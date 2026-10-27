use std::{
    io::Read,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

pub struct ProcessResult {
    pub success: bool,
    pub status: String,
    pub stopped: Option<&'static str>,
    pub code: Option<i32>,
}

#[cfg(windows)]
mod windows_job {
    use std::{io, mem, os::windows::io::AsRawHandle, process::Child};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        System::{Diagnostics::ToolHelp::*, JobObjects::*, Threading::*},
    };
    pub struct Job(HANDLE);
    impl Drop for Job {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    impl Job {
        pub fn attach_and_resume(child: &Child) -> io::Result<Self> {
            // The child starts suspended: attach before it can spawn descendants.
            unsafe {
                let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if handle.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let job = Self(handle);
                let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = mem::zeroed();
                limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    &limits as *const _ as _,
                    mem::size_of_val(&limits) as _,
                ) == 0
                    || AssignProcessToJobObject(handle, child.as_raw_handle() as _) == 0
                {
                    return Err(io::Error::last_os_error());
                }
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if snapshot == INVALID_HANDLE_VALUE {
                    return Err(io::Error::last_os_error());
                }
                let mut entry: THREADENTRY32 = mem::zeroed();
                entry.dwSize = mem::size_of_val(&entry) as _;
                let mut found = Thread32First(snapshot, &mut entry) != 0;
                let mut resumed = false;
                while found {
                    if entry.th32OwnerProcessID == child.id() {
                        let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                        if !thread.is_null() {
                            resumed = ResumeThread(thread) != u32::MAX;
                            CloseHandle(thread);
                        }
                        break;
                    }
                    found = Thread32Next(snapshot, &mut entry) != 0;
                }
                CloseHandle(snapshot);
                if !resumed {
                    return Err(io::Error::other("Could not resume the managed process"));
                }
                Ok(job)
            }
        }
        pub fn terminate(&self) {
            unsafe {
                TerminateJobObject(self.0, 1);
            }
        }
        pub fn release_children(&self) -> io::Result<()> {
            // A successful installer can launch the newly installed app. Leave
            // that app running after its package manager has finished normally.
            unsafe {
                let limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = mem::zeroed();
                if SetInformationJobObject(
                    self.0,
                    JobObjectExtendedLimitInformation,
                    &limits as *const _ as _,
                    mem::size_of_val(&limits) as _,
                ) == 0
                {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        }
    }
}

pub fn run(
    command: &mut Command,
    timeout: Duration,
    cancel: &AtomicBool,
    mut output: impl FnMut(&str),
) -> Result<ProcessResult, String> {
    if timeout.is_zero() {
        return Ok(ProcessResult {
            success: false,
            status: "Timed out before launch".into(),
            stopped: Some("timed_out"),
            code: None,
        });
    }
    if cancel.load(Ordering::SeqCst) {
        return Ok(ProcessResult {
            success: false,
            status: "Cancelled before launch".into(),
            stopped: Some("cancelled"),
            code: None,
        });
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000004);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(windows)]
    let job = match windows_job::Job::attach_and_resume(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("Could not manage process safely: {error}"));
        }
    };
    let (sender, receiver) = mpsc::sync_channel::<String>(64);
    let readers: Vec<Box<dyn Read + Send>> = vec![
        Box::new(child.stdout.take().unwrap()),
        Box::new(child.stderr.take().unwrap()),
    ];
    for mut reader in readers {
        let sender = sender.clone();
        std::thread::spawn(move || {
            let mut bytes = [0u8; 4096];
            loop {
                match reader.read(&mut bytes) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => {
                        if sender
                            .send(String::from_utf8_lossy(&bytes[..count]).into_owned())
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });
    }
    drop(sender);
    let started = Instant::now();
    let mut stopped = None;
    let status = loop {
        match receiver.recv_timeout(Duration::from_millis(80)) {
            Ok(text) => output(&text),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(_) => (),
        }
        for text in receiver.try_iter().take(64) {
            output(&text);
        }
        if stopped.is_none() && (cancel.load(Ordering::SeqCst) || started.elapsed() >= timeout) {
            stopped = Some(if cancel.load(Ordering::SeqCst) {
                "cancelled"
            } else {
                "timed_out"
            });
            #[cfg(windows)]
            job.terminate();
            #[cfg(not(windows))]
            {
                let _ = child.kill();
            }
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => (),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    };
    // Cancel/failure stops descendants; normal completion leaves launched apps alive.
    #[cfg(windows)]
    {
        if status.success() && stopped.is_none() {
            job.release_children().map_err(|e| e.to_string())?;
        }
        drop(job);
    }
    for _ in 0..20 {
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(text) => output(&text),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(_) => (),
        }
    }
    Ok(ProcessResult {
        success: status.success() && stopped.is_none(),
        status: status.to_string(),
        stopped,
        code: status.code(),
    })
}

pub fn capture(command: &mut Command, timeout: Duration) -> Result<String, String> {
    let (result, text) = capture_output(command, timeout)?;
    if !result.success {
        return Err(format!("{}\n{text}", result.status));
    }
    Ok(text)
}
pub fn capture_output(
    command: &mut Command,
    timeout: Duration,
) -> Result<(ProcessResult, String), String> {
    let mut text = String::new();
    let result = run(command, timeout, &AtomicBool::new(false), |chunk| {
        if text.len() < 2_000_000 {
            text.push_str(chunk);
        }
    })?;
    if result.stopped.is_some() {
        return Err("The package manager query timed out.".into());
    }
    Ok((result, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streams_output_before_exit_and_enforces_timeout_and_cancellation() {
        let mut command = crate::command("powershell.exe");
        command.args([
            "-NoProfile",
            "-Command",
            "[Console]::Out.WriteLine('ready'); Start-Sleep -Seconds 3",
        ]);
        let cancel = AtomicBool::new(false);
        let mut text = String::new();
        let result = run(&mut command, Duration::from_secs(10), &cancel, |chunk| {
            text.push_str(chunk);
            if text.contains("ready") {
                cancel.store(true, Ordering::SeqCst);
            }
        })
        .unwrap();
        assert!(text.contains("ready"));
        assert_eq!(result.stopped, Some("cancelled"));
        let mut command = crate::command("powershell.exe");
        command.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 3"]);
        let start = Instant::now();
        let result = run(
            &mut command,
            Duration::from_millis(200),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert_eq!(result.stopped, Some("timed_out"));
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
