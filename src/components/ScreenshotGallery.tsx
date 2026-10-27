import { useState } from 'react';
import { ChevronLeft, ChevronRight, Expand, ImageOff } from 'lucide-react';
import type { AppPreview } from '../appMedia';
import { Dialog } from './Dialog';

export function ScreenshotGallery({ previews, name }: { previews: AppPreview[]; name: string }) {
  const [index, setIndex] = useState(0);
  const [expanded, setExpanded] = useState(false);
  const [failed, setFailed] = useState<string[]>([]);
  const available = previews.filter(preview => !failed.includes(preview.src));
  const currentIndex = Math.min(index, Math.max(available.length - 1, 0));
  const current = available[currentIndex];
  function next(offset: number) { setIndex((currentIndex + offset + available.length) % available.length); }
  return <section className={`screenshot-section ${current ? '' : 'screenshot-section-empty'}`} aria-label={`${name} screenshots`}>
    {current ? <>
      <div className="screenshot-track">
        {[-1, 0, 1].map(offset => {
          const preview = available[(currentIndex + offset + available.length) % available.length];
          return <button key={`${offset}-${preview.src}`} className={`screenshot-frame ${offset === 0 ? 'screenshot-center' : 'screenshot-neighbor'}`} tabIndex={offset === 0 ? 0 : -1}
            aria-label={offset === 0 ? `Enlarge screenshot: ${preview.caption}` : `${offset < 0 ? 'Previous' : 'Next'} screenshot preview`} onClick={() => offset === 0 ? setExpanded(true) : next(offset)}>
            <img src={preview.src} alt={preview.caption} onError={() => setFailed(previous => [...previous, preview.src])} />
          </button>;
        })}
      </div>
      {available.length > 1 && <><button className="gallery-arrow gallery-previous" aria-label="Previous screenshot" onClick={() => next(-1)}><ChevronLeft /></button><button className="gallery-arrow gallery-next" aria-label="Next screenshot" onClick={() => next(1)}><ChevronRight /></button></>}
      <div className="gallery-caption"><span>{current.caption}</span><span>{currentIndex + 1} / {available.length}</span></div>
      <button className="gallery-expand" aria-label="Enlarge screenshot" onClick={() => setExpanded(true)}><Expand size={19} /></button>
      {expanded && <Dialog title={current.caption} className="screenshot-lightbox" onClose={() => setExpanded(false)}><img src={current.src} alt={current.caption} /></Dialog>}
    </> : <div className="screenshots-unavailable"><ImageOff size={17} /><p>Screenshots aren’t available for this app yet</p></div>}
  </section>;
}
