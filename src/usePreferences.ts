import { useEffect, useState } from 'react';
import type { AppItem, SourceStatus } from './types';

export type SortOrder = 'recommended' | 'name' | 'name-desc' | 'publisher' | 'updates';
interface Preferences {
  sources: SourceStatus;
  sort: SortOrder;
  fontScale: number;
  timeoutMinutes: number;
  autoCheckUpdates: boolean;
}
const defaults: Preferences = { sources: { scoop: true, winget: true, choco: true }, sort: 'recommended', fontScale: 1, timeoutMinutes: 30, autoCheckUpdates: true };
function readPreferences(): Preferences {
  try {
    const value = JSON.parse(localStorage.getItem('winstore-preferences') ?? '{}');
    return {
      sources: Object.fromEntries(Object.entries(defaults.sources).map(([key, fallback]) => [key, typeof value.sources?.[key] === 'boolean' ? value.sources[key] : fallback])) as unknown as SourceStatus,
      sort: ['recommended', 'name', 'name-desc', 'publisher', 'updates'].includes(value.sort) ? value.sort : defaults.sort,
      fontScale: [1, 1.1, 1.2, 1.3].includes(value.fontScale) ? value.fontScale : 1,
      timeoutMinutes: [5, 15, 30, 60, 120].includes(value.timeoutMinutes) ? value.timeoutMinutes : 30,
      autoCheckUpdates: typeof value.autoCheckUpdates === 'boolean' ? value.autoCheckUpdates : true,
    };
  } catch { return defaults; }
}
export function usePreferences() {
  const [preferences, setPreferences] = useState(readPreferences);
  const [savedIds, setSavedIds] = useState<string[]>(() => {
    try { const ids = JSON.parse(localStorage.getItem('winstore-saved-apps') ?? '[]'); return Array.isArray(ids) ? ids.filter((id): id is string => typeof id === 'string') : []; }
    catch { return []; }
  });
  const [savedApps, setSavedApps] = useState<AppItem[]>(() => {
    try { const apps = JSON.parse(localStorage.getItem('winstore-saved-metadata') ?? '[]'); return Array.isArray(apps) ? apps.filter(app => app && typeof app.id === 'string' && typeof app.name === 'string' && Array.isArray(app.sources) && Array.isArray(app.tags)).map(app => ({ ...app, installations: [], is_installed: false, has_update: false })) : []; }
    catch { return []; }
  });
  const [storageError, setStorageError] = useState('');
  useEffect(() => {
    document.documentElement.style.setProperty('--font-scale', String(preferences.fontScale));
    try { localStorage.setItem('winstore-preferences', JSON.stringify(preferences)); setStorageError(''); }
    catch { setStorageError('Your settings could not be saved on this device.'); }
  }, [preferences]);
  function toggleSaved(app: AppItem) {
    const removing = savedIds.includes(app.id);
    const ids = removing ? savedIds.filter(id => id !== app.id) : [...savedIds, app.id];
    const metadata = removing ? savedApps.filter(item => item.id !== app.id) : [...savedApps.filter(item => item.id !== app.id), { ...app, installations: [], is_installed: false, has_update: false }];
    try {
      localStorage.setItem('winstore-saved-metadata', JSON.stringify(metadata));
      localStorage.setItem('winstore-saved-apps', JSON.stringify(ids));
      setSavedApps(metadata); setSavedIds(ids); setStorageError('');
    } catch { setStorageError('Could not save this app on this device.'); }
  }
  return { preferences, setPreferences, savedIds, savedApps, toggleSaved, storageError };
}
