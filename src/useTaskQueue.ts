import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppItem, PackageSource, QueueItem } from './types';

interface Snapshot { tasks: QueueItem[]; revision: number; warning: string }
export const activeTask = (task: QueueItem) => task.status === 'queued' || task.status === 'running';
export function useTaskQueue(onFinished: () => void) {
  const [tasks, setTasks] = useState<QueueItem[]>([]);
  const [warning, setWarning] = useState('');
  const [error, setError] = useState('');
  const [ready, setReady] = useState(false);
  const resetRevision = useRef(-1);
  useEffect(() => {
    let stopped = false;
    const unlisten: (() => void)[] = [];
    const merge = (task: QueueItem) => setTasks(previous => {
      if (task.revision < resetRevision.current) return previous;
      const known = previous.find(item => item.id === task.id);
      if (known && known.revision > task.revision) return previous;
      return [...previous.filter(item => item.id !== task.id), task].sort((a,b) => b.created_at-a.created_at);
    });
    void (async () => {
      try {
        const taskListener = await listen<{ task: QueueItem; warning: string }>('task-changed', ({ payload }) => {
          if (stopped || payload.task.revision < resetRevision.current) return;
          merge(payload.task); setWarning(payload.warning); if (!activeTask(payload.task)) onFinished();
        });
        if (stopped) taskListener(); else unlisten.push(taskListener);
        const resetListener = await listen<Snapshot>('tasks-reset', ({ payload }) => {
          if (stopped || payload.revision < resetRevision.current) return;
          resetRevision.current = payload.revision; setTasks(payload.tasks); setWarning(payload.warning);
        });
        if (stopped) resetListener(); else unlisten.push(resetListener);
        const data = await invoke<Snapshot>('get_tasks');
        if (!stopped) { if (data.revision >= resetRevision.current) { for (const task of data.tasks) merge(task); setWarning(data.warning); } setReady(true); }
      } catch (cause) { if (!stopped) { setError(String(cause)); setReady(true); } }
    })();
    return () => { stopped = true; unlisten.forEach(stop => stop()); };
  }, []);
  async function action(command: string, args?: Record<string, unknown>) {
    setError('');
    try { await invoke(command, args); }
    catch (cause) { setError(String(cause)); }
  }
  async function enqueue(app: AppItem, source: PackageSource, taskAction: string, scope: string | undefined, timeoutMinutes: number) {
    await action('enqueue_task', { app, source, action: taskAction, scope: scope ?? null, timeoutSeconds: timeoutMinutes*60 });
  }
  return { tasks, warning, error, ready, enqueue,
    dismiss: (id: string) => action('dismiss_task', { id }),
    cancel: (id: string) => action('cancel_task', { id }), retry: (id: string) => action('retry_task', { id }),
    clear: () => action('clear_task_history') };
}
