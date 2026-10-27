use crate::{process_runner, AppItem};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::Emitter;

const OUTPUT_LIMIT: usize = 32_768;
const HISTORY_LIMIT: usize = 100;
static IDS: AtomicU64 = AtomicU64::new(1);
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn active(status: &str) -> bool {
    matches!(status, "queued" | "running")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub app: AppItem,
    pub package_id: String,
    pub source: String,
    pub action: String,
    pub scope: Option<String>,
    pub timeout_seconds: u64,
    pub status: String,
    pub output: String,
    pub message: String,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub attempt: u32,
    pub revision: u64,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub tasks: Vec<Task>,
    pub revision: u64,
    #[serde(default)]
    pub warning: String,
}
struct Inner {
    data: Mutex<Snapshot>,
    worker: AtomicBool,
    cancel: Mutex<Option<(String, Arc<AtomicBool>)>>,
    app: tauri::AppHandle,
    path: PathBuf,
    _lock: std::fs::File,
}
#[derive(Clone)]
pub struct TaskManager(Arc<Inner>);

fn persist(path: &std::path::Path, data: &Snapshot) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec(data).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
    {
        use std::io::Write;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(&temp, path).map_err(|e| e.to_string())
}
fn recover(data: &mut Snapshot) {
    for task in &mut data.tasks {
        if active(&task.status) {
            task.status = "interrupted".into();
            task.message="Winstore closed before this task finished. Refresh your apps, then retry if needed.".into();
            task.finished_at = Some(now());
            task.revision += 1;
        }
    }
}
fn append_output(output: &mut String, text: &str) {
    output.push_str(text);
    if output.len() > OUTPUT_LIMIT {
        let mut start = output.len() - OUTPUT_LIMIT;
        while !output.is_char_boundary(start) {
            start += 1;
        }
        output.drain(..start);
    }
}
impl TaskManager {
    pub fn load(app: tauri::AppHandle, path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let history_lock = options.open(path.with_extension("lock")).map_err(|e| {
            format!("Task history is in use by another Winstore window or cannot be opened: {e}")
        })?;
        let mut data = match std::fs::read(&path) {
            Ok(bytes) if bytes.len() <= 8_000_000 => serde_json::from_slice::<Snapshot>(&bytes)
                .unwrap_or_else(|_| Snapshot {
                    warning: "Task history is unreadable. The original file has been preserved."
                        .into(),
                    ..Default::default()
                }),
            Ok(_) => Snapshot {
                warning: "Task history is too large to load. The original file has been preserved."
                    .into(),
                ..Default::default()
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Snapshot::default(),
            Err(error) => Snapshot {
                warning: format!("Could not read task history: {error}"),
                ..Default::default()
            },
        };
        if !data.warning.is_empty() && path.is_file() {
            let backup = path.with_extension(format!("corrupt-{}.json", now()));
            if let Err(error) = std::fs::copy(&path, backup) {
                return Err(format!(
                    "Could not preserve unreadable task history: {error}"
                ));
            }
        }
        recover(&mut data);
        if let Err(error) = persist(&path, &data) {
            data.warning = format!("Task history could not be saved: {error}");
        }
        Ok(Self(Arc::new(Inner {
            data: Mutex::new(data),
            worker: AtomicBool::new(false),
            cancel: Mutex::new(None),
            app,
            path,
            _lock: history_lock,
        })))
    }
    fn snapshot(&self) -> Snapshot {
        self.0.data.lock().unwrap().clone()
    }
    fn update(&self, id: &str, save: bool, update: impl FnOnce(&mut Task)) {
        let mut data = self.0.data.lock().unwrap();
        data.revision += 1;
        let revision = data.revision;
        let Some(task) = data.tasks.iter_mut().find(|task| task.id == id) else {
            return;
        };
        update(task);
        task.revision = revision;
        let task = task.clone();
        if save {
            if let Err(error) = persist(&self.0.path, &data) {
                data.warning = format!("Task history could not be saved: {error}");
            }
        }
        let warning = data.warning.clone();
        drop(data);
        let _ = self.0.app.emit(
            "task-changed",
            serde_json::json!({"task":task,"warning":warning}),
        );
    }
    fn enqueue(
        &self,
        app: AppItem,
        source: String,
        action: String,
        scope: Option<String>,
        timeout_seconds: u64,
        attempt: u32,
    ) -> Result<Task, String> {
        if !["install", "update", "uninstall"].contains(&action.as_str())
            || !["scoop", "winget", "choco"].contains(&source.as_str())
        {
            return Err("Invalid task action or source.".into());
        }
        if !(60..=7200).contains(&timeout_seconds) {
            return Err("Task timeout must be between 1 and 120 minutes.".into());
        }
        let option = app
            .sources
            .iter()
            .find(|option| option.source == source)
            .ok_or("This app has no selected package source.")?;
        let package_id = if action == "install" {
            option.package_id.clone()
        } else {
            app.installations
                .iter()
                .find(|item| {
                    item.source == source
                        && Some(&item.scope) == scope.as_ref()
                        && !["ambiguous", "overlap"].contains(&item.scope.as_str())
                        && (action != "update" || !item.held)
                })
                .ok_or("Choose a detected installation that can be managed.")?
                .package_id
                .clone()
        };
        if !crate::valid_package_id(&package_id) {
            return Err("Invalid package ID.".into());
        }
        let mut data = self.0.data.lock().unwrap();
        if data
            .tasks
            .iter()
            .any(|task| task.app.id == app.id && active(&task.status))
        {
            return Err("This app already has a queued or running task.".into());
        }
        if data
            .tasks
            .iter()
            .filter(|task| active(&task.status))
            .count()
            >= 20
        {
            return Err("The queue is full. Wait for tasks to finish.".into());
        }
        data.revision += 1;
        let task = Task {
            id: format!("{}-{}", now(), IDS.fetch_add(1, Ordering::SeqCst)),
            app,
            source,
            package_id,
            action,
            scope,
            timeout_seconds,
            status: "queued".into(),
            output: String::new(),
            message: "Waiting for earlier tasks.".into(),
            created_at: now(),
            started_at: None,
            finished_at: None,
            attempt,
            revision: data.revision,
        };
        data.tasks.push(task.clone());
        while data.tasks.len() > HISTORY_LIMIT {
            if let Some(index) = data.tasks.iter().position(|task| !active(&task.status)) {
                data.tasks.remove(index);
            } else {
                break;
            }
        }
        if let Err(error) = persist(&self.0.path, &data) {
            data.tasks.retain(|item| item.id != task.id);
            return Err(format!("Could not save the queued task: {error}"));
        }
        drop(data);
        let _ = self.0.app.emit(
            "task-changed",
            serde_json::json!({"task":task,"warning":""}),
        );
        self.start();
        Ok(task)
    }
    fn start(&self) {
        if self
            .0
            .worker
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let manager = self.clone();
            std::thread::spawn(move || manager.worker());
        }
    }
    fn worker(&self) {
        loop {
            let cancel = Arc::new(AtomicBool::new(false));
            let next = {
                let mut data = self.0.data.lock().unwrap();
                let Some(index) = data.tasks.iter().position(|task| task.status == "queued") else {
                    self.0.worker.store(false, Ordering::SeqCst);
                    return;
                };
                *self.0.cancel.lock().unwrap() =
                    Some((data.tasks[index].id.clone(), cancel.clone()));
                data.revision += 1;
                let revision = data.revision;
                let task = &mut data.tasks[index];
                task.status = "running".into();
                task.started_at = Some(now());
                task.message = "Checking the package source…".into();
                task.revision = revision;
                let task = task.clone();
                if let Err(error) = persist(&self.0.path, &data) {
                    data.warning = format!("Task history could not be saved: {error}");
                }
                let warning = data.warning.clone();
                drop(data);
                let _ = self.0.app.emit(
                    "task-changed",
                    serde_json::json!({"task":task,"warning":warning}),
                );
                task
            };
            let task = next;
            let started = Instant::now();
            let command = crate::prepare_action(
                task.package_id.clone(),
                task.source.clone(),
                &task.action,
                task.scope.clone(),
            );
            let result = match command {
                Ok(mut command) => {
                    self.update(&task.id, false, |item| {
                        item.message =
                            "Package manager is running. Installer windows may appear.".into()
                    });
                    let remaining =
                        Duration::from_secs(task.timeout_seconds).saturating_sub(started.elapsed());
                    let mut flushed = Instant::now();
                    let mut first_output = true;
                    process_runner::run(&mut command, remaining, &cancel, |chunk| {
                        let save = first_output || flushed.elapsed() >= Duration::from_secs(1);
                        first_output = false;
                        self.update(&task.id, save, |item| {
                            append_output(&mut item.output, chunk)
                        });
                        if save {
                            flushed = Instant::now();
                        }
                    })
                }
                Err(error) => Err(error),
            };
            self.update(&task.id,true,|item|{item.finished_at=Some(now());match result{
            Ok(result)=>{item.status=result.stopped.unwrap_or(if result.success{"completed"}else{"failed"}).into();item.message=match result.stopped{Some("cancelled")=>"Stopped the package manager process tree. Refresh apps to check any changes already made.".into(),Some("timed_out")=>"The task exceeded its timeout and was stopped. Refresh apps before retrying.".into(),_=>format!("{} via {} ({}).",if result.success{"Package manager finished"}else{"Package manager failed"},task.source,result.status)};},
            Err(error)=>{item.status=if cancel.load(Ordering::SeqCst){"cancelled"}else{"failed"}.into();item.message=error;}
        }});
            *self.0.cancel.lock().unwrap() = None;
        }
    }
    fn cancel(&self, id: &str) -> Result<(), String> {
        let mut data = self.0.data.lock().unwrap();
        let index = data
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or("Task was not found.")?;
        if !active(&data.tasks[index].status) {
            return Ok(());
        }
        data.revision += 1;
        let revision = data.revision;
        let task = &mut data.tasks[index];
        task.revision = revision;
        if task.status == "queued" {
            task.status = "cancelled".into();
            task.message = "Cancelled before launch.".into();
            task.finished_at = Some(now());
        } else {
            if let Some((active, flag)) = self.0.cancel.lock().unwrap().as_ref() {
                if active == id {
                    flag.store(true, Ordering::SeqCst);
                }
            }
            task.message =
                "Cancellation requested. Waiting for the source check or process to stop…".into();
        }
        let task = task.clone();
        if let Err(error) = persist(&self.0.path, &data) {
            data.warning = format!("Task history could not be saved: {error}");
        }
        let warning = data.warning.clone();
        drop(data);
        let _ = self.0.app.emit(
            "task-changed",
            serde_json::json!({"task":task,"warning":warning}),
        );
        Ok(())
    }
}

#[tauri::command]
pub fn get_tasks(manager: tauri::State<'_, TaskManager>) -> Snapshot {
    manager.snapshot()
}
#[tauri::command]
pub fn enqueue_task(
    manager: tauri::State<'_, TaskManager>,
    app: AppItem,
    source: String,
    action: String,
    scope: Option<String>,
    timeout_seconds: u64,
) -> Result<Task, String> {
    manager.enqueue(app, source, action, scope, timeout_seconds, 1)
}
#[tauri::command]
pub fn cancel_task(manager: tauri::State<'_, TaskManager>, id: String) -> Result<(), String> {
    manager.cancel(&id)
}
#[tauri::command]
pub fn retry_task(manager: tauri::State<'_, TaskManager>, id: String) -> Result<Task, String> {
    let task = manager
        .snapshot()
        .tasks
        .into_iter()
        .find(|task| task.id == id)
        .ok_or("Task was not found.")?;
    if active(&task.status) || task.status == "completed" {
        return Err("Only unsuccessful tasks can be retried.".into());
    }
    manager.enqueue(
        task.app,
        task.source,
        task.action,
        task.scope,
        task.timeout_seconds,
        task.attempt + 1,
    )
}
#[tauri::command]
pub fn clear_task_history(manager: tauri::State<'_, TaskManager>) -> Result<Snapshot, String> {
    let mut data = manager.0.data.lock().unwrap();
    let mut next = data.clone();
    next.tasks.retain(|task| active(&task.status));
    next.revision += 1;
    persist(&manager.0.path, &next)?;
    *data = next;
    let result = data.clone();
    drop(data);
    let _ = manager.0.app.emit("tasks-reset", &result);
    Ok(result)
}

#[tauri::command]
pub fn dismiss_task(
    manager: tauri::State<'_, TaskManager>,
    id: String,
) -> Result<Snapshot, String> {
    let mut data = manager.0.data.lock().unwrap();
    if data
        .tasks
        .iter()
        .any(|task| task.id == id && active(&task.status))
    {
        return Err("Cancel this task before removing it.".into());
    }
    let mut next = data.clone();
    next.tasks.retain(|task| task.id != id);
    next.revision += 1;
    persist(&manager.0.path, &next)?;
    *data = next;
    let result = data.clone();
    drop(data);
    let _ = manager.0.app.emit("tasks-reset", &result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restart_never_replays_unfinished_actions_and_output_is_bounded() {
        let app = crate::get_featured_apps().remove(0);
        let mut data = Snapshot {
            tasks: vec![Task {
                id: "test".into(),
                app,
                package_id: "test".into(),
                source: "scoop".into(),
                action: "install".into(),
                scope: None,
                timeout_seconds: 60,
                status: "running".into(),
                output: String::new(),
                message: String::new(),
                created_at: 0,
                started_at: Some(0),
                finished_at: None,
                attempt: 1,
                revision: 0,
            }],
            ..Default::default()
        };
        recover(&mut data);
        assert_eq!(data.tasks[0].status, "interrupted");
        assert!(data.tasks[0].finished_at.is_some());
        let mut output = "é".repeat(OUTPUT_LIMIT);
        append_output(&mut output, "latest");
        assert!(output.len() <= OUTPUT_LIMIT);
        assert!(output.ends_with("latest"));
    }
}
