import { useEffect, useRef, type ReactNode } from 'react';
import { X } from 'lucide-react';
import { TitleBar } from './TitleBar';
import { lockDocumentScroll } from './dialogScrollLock';

export function Dialog({ title, onClose, children, drawer = false, className = '', header }: {
  title: string; onClose: () => void; children: ReactNode; drawer?: boolean;
  className?: string; header?: ReactNode;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const opener = useRef(document.activeElement instanceof HTMLElement ? document.activeElement : null);
  useEffect(() => {
    const element = ref.current;
    const unlockScroll = lockDocumentScroll();
    element?.showModal();
    return () => {
      element?.close();
      unlockScroll();
      if (opener.current?.isConnected) opener.current.focus({ preventScroll: true });
    };
  }, []);
  return <dialog ref={ref} className={`dialog ${drawer ? 'drawer' : ''} ${className}`} aria-label={title}
    onCancel={event => { event.stopPropagation(); onClose(); }} onClick={event => { if (event.target === event.currentTarget) onClose(); }}>
    <TitleBar />
    <div className="dialog-body">
      {header ?? <div className="dialog-heading"><h2>{title}</h2><button className="icon-button" aria-label="Close dialog" onClick={onClose}><X size={19} /></button></div>}
      {children}
    </div>
  </dialog>;
}
