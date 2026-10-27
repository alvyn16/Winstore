import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppItem } from './types';

const requests = new Map<string, { expires: number; result: Promise<string | null> }>();
let running = 0;
const waiting: (() => void)[] = [];
async function limited(request: () => Promise<string | null>) {
  if (running >= 4) await new Promise<void>(resolve => waiting.push(resolve));
  else running++;
  try { return await request(); }
  finally { const next = waiting.shift(); if (next) next(); else running--; }
}
function resolve(app: AppItem, key: string, skipBundled: boolean) {
  const cached = requests.get(key);
  if (cached && cached.expires > Date.now()) return cached.result;
  if (requests.size >= 512) requests.delete(requests.keys().next().value!);
  const result = limited(() => invoke<string | null>('resolve_app_icon', { app, skipBundled }).catch(() => null));
  requests.set(key, { expires: Date.now() + 5*60*1000, result });
  return result;
}
export function useAppIcon(app: AppItem) {
  const ref = useRef<HTMLSpanElement>(null);
  const [visible, setVisible] = useState(false);
  const [failed, setFailed] = useState<{ key: string; sources: string[] } | null>(null);
  const [resolved, setResolved] = useState<{ key: string; src: string | null } | null>(null);
  const key = JSON.stringify([app.id, app.name, app.icon, app.is_installed, app.sources.map(option => [option.source, option.package_id])]);
  const bundledFailed = !!app.icon && failed?.key === key && failed.sources.includes(app.icon);
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    if (typeof IntersectionObserver === 'undefined') { setVisible(true); return; }
    const observer = new IntersectionObserver(entries => {
      if (entries.some(entry => entry.isIntersecting)) { setVisible(true); observer.disconnect(); }
    }, { rootMargin: '160px' });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    if (!visible || (app.icon && !bundledFailed)) return;
    let stopped = false;
    void resolve(app, key + ':' + bundledFailed, bundledFailed).then(src => { if (!stopped) setResolved({ key, src }); });
    return () => { stopped = true; };
  }, [key, visible, bundledFailed]);
  const src = app.icon && !bundledFailed ? app.icon : resolved?.key === key ? resolved.src : null;
  return { ref, src: src && !(failed?.key === key && failed.sources.includes(src)) ? src : null,
    onError: () => { if (src) setFailed(previous => ({ key, sources: [...(previous?.key === key ? previous.sources : []), src] })); } };
}
