import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
interface UpdateInfo { current_version: string; configured: boolean; version: string | null; download_url: string | null; notes: string | null }
export function useAppUpdates(autoCheck: boolean) {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const [checked, setChecked] = useState(false);
  const [error, setError] = useState('');
  const started = useRef(false);
  async function check() {
    setChecking(true); setError('');
    try { setInfo(await invoke<UpdateInfo>('check_winstore_update')); setChecked(true); }
    catch (cause) { setError(String(cause)); }
    finally { setChecking(false); }
  }
  useEffect(() => { let stopped = false; void invoke<UpdateInfo>('get_update_configuration').then(info => { if (!stopped) setInfo(info); }).catch(cause => { if (!stopped) setError(String(cause)); }); return () => { stopped = true; }; }, []);
  useEffect(() => { if (info?.configured && autoCheck && !started.current) { started.current = true; void check(); } }, [info?.configured, autoCheck]);
  async function download() { if (info?.download_url) try { await openUrl(info.download_url); } catch { setError('Could not open the Winstore installer download.'); } }
  return { info, checking, checked, error, check, download };
}
