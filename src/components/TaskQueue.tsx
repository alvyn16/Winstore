import { Download } from 'lucide-react';
import { Dialog } from './Dialog';
import { AppIcon } from './AppCard';
import { activeTask, useTaskQueue } from '../useTaskQueue';
const names = { scoop: 'Scoop', winget: 'Winget', choco: 'Chocolatey' };
const statuses = { queued: 'Queued', running: 'Running', completed: 'Completed', failed: 'Failed', cancelled: 'Cancelled', timed_out: 'Timed out', interrupted: 'Interrupted' };
export function TaskQueue({ queue, onClose }: { queue: ReturnType<typeof useTaskQueue>; onClose: () => void }) {
  return <Dialog title="Downloads & tasks" onClose={onClose} drawer>
    {(queue.error || queue.warning) && <p className="source-warning" role="alert">{queue.error || queue.warning}</p>}
    <p className="source-note">Tasks run one at a time. History is saved on this PC. Stopping a task does not undo changes an installer has already made.</p>
    {!queue.ready ? <p role="status">Loading tasks…</p> : !queue.tasks.length ? <div className="empty-state"><Download size={36} /><h3>All quiet here</h3><p>Install an app and follow its progress here.</p></div> : <div className="task-list">{queue.tasks.map(item => <article className="task" key={item.id}>
      <div className="task-heading"><AppIcon app={item.app} /><div><strong>{item.app.name}</strong><small>{item.action} · {names[item.source]}{item.scope ? ` · ${item.scope}` : ''}</small></div><span className={`task-status status-${item.status}`}>{statuses[item.status]}</span></div>
      <div className="task-meta">{new Date(item.created_at).toLocaleString()} · Attempt {item.attempt} · Timeout {item.timeout_seconds/60} min</div>
      {item.status === 'running' && <div className="indeterminate-progress" role="progressbar" aria-label={`${item.action} ${item.app.name}`} />}
      <p className="task-message" role={item.status === 'running' ? 'status' : undefined}>{item.message}</p>
      {item.output && <details open={activeTask(item)}><summary>Live package manager output <span className="muted">(last 32 KB)</span></summary><pre className="live-output" tabIndex={0}>{item.output}</pre></details>}
      <div className="task-actions">{activeTask(item) ? <button className="secondary-button" onClick={() => void queue.cancel(item.id)}>Cancel task</button> : item.status !== 'completed' && <button className="secondary-button" onClick={() => void queue.retry(item.id)}>Retry task</button>}{!activeTask(item) && <button className="secondary-button" onClick={() => void queue.dismiss(item.id)}>Remove from history</button>}</div>
    </article>)}</div>}
    {queue.tasks.some(item => !activeTask(item)) && <button className="secondary-button clear-queue" onClick={() => void queue.clear()}>Clear finished tasks</button>}
  </Dialog>;
}
