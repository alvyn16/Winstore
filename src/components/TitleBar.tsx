import { useEffect, useState } from 'react';
import { Minus, Square, Copy, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './TitleBar.css';

export function TitleBar() {
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    const window = getCurrentWindow();
    let disposed = false;
    const update = () => { void window.isMaximized().then(value => { if (!disposed) setMaximized(value); }); };
    update();
    const listening = window.onResized(update);
    return () => { disposed = true; void listening.then(unlisten => unlisten()); };
  }, []);
  return <div className="window-titlebar" role="banner" aria-label="Window title bar" data-tauri-drag-region>
    <div className="window-identity" data-tauri-drag-region><img src="/winstore.png" alt="" data-tauri-drag-region /><span data-tauri-drag-region>Winstore</span></div>
    <div className="window-controls">
      <button aria-label="Minimize window" title="Minimize" onClick={() => void getCurrentWindow().minimize()}><Minus size={14} /></button>
      <button aria-label={maximized ? 'Restore window' : 'Maximize window'} title={maximized ? 'Restore' : 'Maximize'} onClick={() => void getCurrentWindow().toggleMaximize()}>{maximized ? <Copy size={12} /> : <Square size={12} />}</button>
      <button className="window-close" aria-label="Close window" title="Close" onClick={() => void getCurrentWindow().close()}><X size={16} /></button>
    </div>
  </div>;
}
