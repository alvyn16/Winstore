import { useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppItem, PackageSource, SourceStatus } from './types';

const unavailable: SourceStatus = { winget: false, scoop: false, choco: false };
export function useCatalog(sourcePreferences: SourceStatus) {
  const [catalog, setCatalog] = useState<AppItem[]>([]);
  const [results, setResults] = useState<AppItem[]>([]);
  const [sources, setSources] = useState(unavailable);
  const enabled = useMemo(() => ({ scoop: sources.scoop && sourcePreferences.scoop, winget: sources.winget && sourcePreferences.winget, choco: sources.choco && sourcePreferences.choco }), [sources, sourcePreferences]);
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [searching, setSearching] = useState(false);
  const [inventoryLoading, setInventoryLoading] = useState(true);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const refreshId = useRef(0);

  async function refresh() {
    const request = ++refreshId.current;
    setLoading(true);
    setInventoryLoading(true);
    setNotice('');
    setError('');
    try {
      const [items, status] = await Promise.all([
        invoke<AppItem[]>('get_featured_apps'), invoke<SourceStatus>('check_sources'),
      ]);
      if (request !== refreshId.current) return;
      setCatalog(previous => items.map(app => {
        const known = previous.find(item => item.id === app.id);
        return known ? { ...app, installations: known.installations, is_installed: known.is_installed, has_update: known.has_update } : app;
      }));
      setSources(status);
      setLoading(false);
      const inventory = await invoke<{ apps: AppItem[]; warning: string }>('get_catalog_state');
      if (request !== refreshId.current) return;
      setCatalog(inventory.apps);
      setNotice(inventory.warning);
    } catch (cause) {
      if (request === refreshId.current) setError(String(cause));
    } finally {
      if (request === refreshId.current) { setLoading(false); setInventoryLoading(false); }
    }
  }
  useEffect(() => { void refresh(); return () => { refreshId.current++; }; }, []);

  useEffect(() => {
    let cancelled = false;
    if (!query.trim()) { setSearching(false); setResults([]); return; }
    setSearching(true);
    setError('');
    const timer = setTimeout(async () => {
      try {
        const result = await invoke<{ apps: AppItem[]; warning: string }>('search_apps', {
          query: query.trim(), sources: Object.keys(enabled).filter(source => enabled[source as PackageSource]),
        });
        if (!cancelled) { setResults(result.apps); setError(result.warning); }
      } catch (cause) {
        if (!cancelled) { setError(String(cause)); setResults([]); }
      } finally { if (!cancelled) setSearching(false); }
    }, 350);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [query, enabled]);

  const apps = query.trim() ? results.map(app => {
    const known = catalog.find(item => item.id === app.id || item.sources.some(option => app.sources.some(other =>
      option.source === other.source && option.package_id.toLowerCase() === other.package_id.toLowerCase())));
    return known ? { ...app, installations: known.installations, is_installed: known.is_installed, has_update: known.has_update } : app;
  }) : catalog;
  return { apps, catalog, sources, enabled, query, setQuery, loading, searching, inventoryLoading,
    error, setError, notice, refresh };
}
