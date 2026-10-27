import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Search, Download, RefreshCw, Sun, Moon, Layers, Library, X, Menu, ArrowLeft, ChevronRight, Package, AlertCircle, Check, LoaderCircle, Bookmark } from 'lucide-react';
import type { AppItem, PackageSource } from './types';
import { AppCard } from './components/AppCard';
import { AppDetails } from './components/AppDetails';
import { FeaturedTiles, Showcase, categories } from './components/Showcase';
import { Dialog } from './components/Dialog';
import { useCatalog } from './useCatalog';
import './App.css';
import { useAppUpdates } from './useAppUpdates';
import { useTaskQueue, activeTask } from './useTaskQueue';
import { TaskQueue } from './components/TaskQueue';
import { usePreferences, type SortOrder } from './usePreferences';
import { TitleBar } from './components/TitleBar';

const sourceNames = { winget: 'Winget', scoop: 'Scoop', choco: 'Chocolatey' };
type Tab = 'explore' | 'installed' | 'updates' | 'saved';

export default function App() {
  const prefs = usePreferences();
  const store = useCatalog(prefs.preferences.sources);
  const appUpdates = useAppUpdates(prefs.preferences.autoCheckUpdates);
  const taskQueue = useTaskQueue(() => { void store.refresh(); });
  const [tab, setTab] = useState<Tab>('explore');
  const [category, setCategory] = useState('All');
  const [slide, setSlide] = useState(0);
  const [showAll, setShowAll] = useState(false);
  const [selected, setSelected] = useState<AppItem | null>(null);
  const [source, setSource] = useState<PackageSource>('scoop');
  const [overlay, setOverlay] = useState<'sources' | 'queue' | null>(null);
  const [searchOpen, setSearchOpen] = useState(false);
  const [dark, setDark] = useState(() => localStorage.getItem('winstore-theme') !== 'light');
  const searchRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    localStorage.setItem('winstore-theme', dark ? 'dark' : 'light');
    void invoke('set_window_appearance', { dark }).catch(error => {
      console.error('Could not update the native window theme', error);
    });
  }, [dark]);
  useEffect(() => { if (searchOpen) searchRef.current?.focus(); }, [searchOpen]);
  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
        event.preventDefault(); setSearchOpen(true); searchRef.current?.focus();
      }
      if (event.key === 'Escape' && !selected && !overlay) { store.setQuery(''); setSearchOpen(false); }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selected, overlay, store.setQuery]);

  const selectedApp = selected && (store.apps.find(app => app.id === selected.id) ?? store.catalog.find(app => app.id === selected.id) ?? selected);
  const busy = selectedApp && taskQueue.tasks.some(item => item.app.id === selectedApp.id && activeTask(item));
  const home = tab === 'explore' && !store.query.trim() && category === 'All';
  const savedCollection = prefs.savedIds.map(id => store.catalog.find(app => app.id === id) ?? prefs.savedApps.find(app => app.id === id)).filter((app): app is AppItem => !!app);
  const collection = tab === 'saved' ? savedCollection.filter(app => !store.query.trim() || [app.name, app.publisher, app.description].join(' ').toLowerCase().includes(store.query.trim().toLowerCase())) : store.apps;
  const filtered = collection.filter(app => {
    if (tab === 'saved') return true;
    if (tab === 'installed') return app.is_installed;
    if (tab === 'updates') return app.has_update;
    if (category === 'Artcraft') return app.publisher === 'Artcraft';
    if (app.id.startsWith('installed:')) return false;
    return category === 'All' || app.category === category;
  });
  const collectionCategory = slide === 1 ? 'Gaming' : 'Development';
  const recommended = home && dark && slide !== 0
    ? [...filtered].sort((a, b) => Number(b.category === collectionCategory) - Number(a.category === collectionCategory)) : filtered;
  const ordered = prefs.preferences.sort === 'recommended' ? recommended : [...filtered].sort((a,b) => {
    const order = prefs.preferences.sort;
    return (order === 'updates' ? Number(b.has_update)-Number(a.has_update) : order === 'publisher' ? a.publisher.localeCompare(b.publisher) : 0)
      || a.name.localeCompare(b.name) * (order === 'name-desc' ? -1 : 1);
  });
  const visible = home && !showAll ? ordered.slice(0, 12) : ordered;
  const activeCount = taskQueue.tasks.filter(activeTask).length;
  const heading = store.query.trim() ? `Results for “${store.query.trim()}”` : tab === 'saved' ? 'Saved apps' : tab === 'installed' ? 'Installed' : tab === 'updates' ? 'Updates' : category !== 'All' ? category : dark && slide === 1 ? 'Play your way' : dark && slide === 2 ? 'Developer essentials' : 'Trending';
  function openApp(app: AppItem) {
    setSelected(app);
    const managed = app.installations?.find(item => store.sources[item.source] && store.enabled[item.source]);
    const preferred = (['scoop', 'winget', 'choco'] as const).find(source =>
      store.sources[source] && store.enabled[source] && app.sources.some(item => item.source === source));
    setSource(managed?.source ?? preferred ?? app.default_source ?? 'scoop');
  }
  function chooseCategory(value: string) { setCategory(value); setShowAll(true); }
  function changeTab(value: Tab) { setTab(value); setCategory('All'); setShowAll(false); store.setQuery(''); }

  return <div className="store-shell">
    <TitleBar />
    <header className="toolbar">
      <div className="toolbar-start">
        <button className="brand-button" title="Winstore home" aria-label="Winstore home" onClick={() => changeTab('explore')}><img src="/winstore.png" alt="" width={25} height={25} /></button>
        <button className={`icon-button ${searchOpen ? 'is-active' : ''}`} title="Search apps (Ctrl+F)" aria-label="Search apps" aria-expanded={searchOpen} onClick={() => setSearchOpen(value => !value)}><Search size={19} /></button>
        <button className="icon-button refresh-button" title="Refresh catalog" aria-label="Refresh catalog" disabled={store.loading} onClick={() => void store.refresh()}><RefreshCw size={18} className={store.loading ? 'spin' : ''} /></button>
      </div>
      <nav className="navigation" aria-label="Main navigation">
        <button aria-current={tab === 'explore' ? 'page' : undefined} onClick={() => changeTab('explore')}><Layers size={17} />Explore</button>
        <button aria-current={tab === 'installed' ? 'page' : undefined} onClick={() => changeTab('installed')}><Library size={17} />Installed</button>
        <button aria-current={tab === 'updates' ? 'page' : undefined} onClick={() => changeTab('updates')}><RefreshCw size={16} />Updates</button>
        <button aria-current={tab === 'saved' ? 'page' : undefined} onClick={() => changeTab('saved')}><Bookmark size={16} />Saved</button>
      </nav>
      <div className="toolbar-end">
        <button className="icon-button queue-button" aria-label="Download queue" title="Download queue" onClick={() => setOverlay('queue')}><Download size={19} />{activeCount > 0 && <span className="queue-count">{activeCount}</span>}</button>
        <button className="icon-button" aria-label={dark ? 'Switch to light theme' : 'Switch to dark theme'} title={dark ? 'Light theme' : 'Dark theme'} onClick={() => setDark(value => !value)}>{dark ? <Sun size={19} /> : <Moon size={19} />}</button>
        <button className="icon-button" aria-label="Package sources" title="Package sources" onClick={() => setOverlay('sources')}><Menu size={20} /></button>
      </div>
    </header>
    {searchOpen && <div className="search-strip"><div className="search-field"><Search size={18} /><input ref={searchRef} aria-label="Search software" placeholder="Search apps, tools and games…" value={store.query} onChange={event => store.setQuery(event.target.value)} /><button className="icon-button" aria-label="Close search" onClick={() => { store.setQuery(''); setSearchOpen(false); }}><X size={17} /></button></div></div>}
    <main className="main-content">
      {prefs.storageError && <p className="error-banner" role="alert">{prefs.storageError}</p>}
      {appUpdates.info?.version && <div className="inventory-notice" role="status">Winstore {appUpdates.info.version} is available. <button className="secondary-button" onClick={() => setOverlay('sources')}>View update</button></div>}
      {store.error && <div className="error-banner" role="alert"><AlertCircle size={18} /><span>{store.error}</span><button onClick={() => void store.refresh()}>Retry</button><button className="icon-button" aria-label="Dismiss error" onClick={() => store.setError('')}><X size={16} /></button></div>}
      {store.loading && !store.catalog.length ? <div className="loading-catalog" role="status"><div className="skeleton skeleton-hero" /><div className="app-grid">{Array.from({ length: 12 }, (_, index) => <div key={index} className="skeleton skeleton-card" />)}</div><span className="sr-only">Loading catalog</span></div> : <>
        {home && (dark ? <Showcase apps={store.catalog} slide={slide} setSlide={setSlide} onOpen={openApp} onCategory={chooseCategory} /> : <FeaturedTiles apps={store.catalog} onOpen={openApp} />)}
        {tab === 'explore' && !store.query.trim() && <nav className="catalog-filters" aria-label="App categories">
          <button aria-pressed={category === 'All'} onClick={() => { setCategory('All'); setShowAll(false); }}>All apps</button>
          <button aria-pressed={category === 'Artcraft'} onClick={() => chooseCategory('Artcraft')}>Artcraft</button>
          {categories.map(({ name, icon: Icon }) => <button key={name} aria-pressed={category === name} onClick={() => chooseCategory(name)}><Icon size={14} />{name}</button>)}
        </nav>}
        <section className="app-section" aria-labelledby="apps-heading" aria-busy={store.searching}>
          <div className="section-heading"><div className="section-title">{category !== 'All' && tab === 'explore' && <button className="icon-button" aria-label="All categories" onClick={() => { setCategory('All'); setShowAll(false); }}><ArrowLeft size={20} /></button>}<h2 id="apps-heading">{heading}</h2>{!store.searching && <span className="result-count">{filtered.length}</span>}</div><label className="sort-control">Sort <select aria-label="Sort apps" value={prefs.preferences.sort} onChange={event => prefs.setPreferences(previous => ({ ...previous, sort: event.target.value as SortOrder }))}><option value="recommended">Recommended</option><option value="name">Name A–Z</option><option value="name-desc">Name Z–A</option><option value="publisher">Publisher</option><option value="updates">Updates first</option></select></label>{home && filtered.length > 12 && <button className="secondary-button" onClick={() => setShowAll(value => !value)}>{showAll ? 'Show Less' : 'Show More'}</button>}</div>
          {tab === 'saved' && !filtered.length && !store.query.trim() && <p className="inventory-notice">Save apps from their details page to keep them here.</p>}
          {tab !== 'explore' && store.notice && <p className="inventory-notice" role="status">{store.notice}</p>}
          {(tab !== 'saved' && store.searching) || ((tab === 'installed' || tab === 'updates') && store.inventoryLoading) ? <div className="empty-state" role="status"><LoaderCircle className="spin" /><h3>{store.searching ? 'Searching your sources…' : 'Checking your apps…'}</h3><p>{store.searching ? 'Finding software that matches your search.' : 'Reading installed packages and available updates.'}</p></div> : visible.length ? <div className={`app-grid ${home && dark && slide === 1 ? 'gaming-grid' : ''}`}>{visible.map(app => <AppCard key={app.id} app={app} onOpen={openApp} />)}</div> : <div className="empty-state" role="status">{tab === 'updates' ? <Check size={34} /> : <Package size={34} />}<h3>{store.query ? 'No matching apps' : tab === 'updates' ? 'No updates found' : tab === 'installed' ? 'No catalog apps detected' : 'No apps in this category'}</h3><p>{store.query ? 'Try another name or enable a package source.' : tab === 'installed' ? 'Installed apps from the curated catalog appear here when detected.' : tab === 'updates' ? 'Refresh to check the apps in this catalog.' : 'Explore another category to find more software.'}</p><button className="secondary-button" onClick={() => { if (tab === 'explore') { setCategory('All'); store.setQuery(''); } else void store.refresh(); }}>{tab === 'explore' ? 'Browse all apps' : 'Refresh'}</button></div>}
        </section>
        {home && <section className="category-section" aria-labelledby="categories-heading"><h2 id="categories-heading">Categories</h2><div className="category-grid">{categories.map(({ name, icon: Icon, style }) => <button key={name} className={`category-tile category-${style}`} onClick={() => chooseCategory(name)}><Icon size={23} /><span>{name}</span><ChevronRight size={16} /></button>)}</div></section>}
        {home && <footer className="catalog-footer"><Layers size={14} /><span>Good software. One place.</span><button onClick={() => setOverlay('sources')}>Manage sources<ChevronRight size={13} /></button></footer>}
      </>}
    </main>
    {selectedApp && <AppDetails key={selectedApp.id} app={selectedApp} source={source} setSource={setSource}
      sources={store.sources} enabled={store.enabled} busy={!!busy} dark={dark} saved={prefs.savedIds.includes(selectedApp.id)} onToggleSaved={() => prefs.toggleSaved(selectedApp)}
      onToggleTheme={() => setDark(value => !value)} onClose={() => setSelected(null)}
      onAction={(action, scope) => { void taskQueue.enqueue(selectedApp, source, action, scope, prefs.preferences.timeoutMinutes); setSelected(null); setOverlay('queue'); }} />}
    {overlay === 'sources' && <Dialog title="Package sources" className="package-sources-dialog" onClose={() => setOverlay(null)}>
      <p className="muted">Choose which package managers Winstore can use.</p>
      <div className="source-settings">{(['winget', 'scoop', 'choco'] as const).map(key => <label key={key}><span><strong>{sourceNames[key]}</strong><small>{key === 'winget' ? 'Windows Package Manager' : key === 'scoop' ? 'Apps installed in your user folder' : 'Community package repository'}</small></span><span className={`availability ${store.sources[key] ? 'available' : ''}`}>{store.sources[key] ? 'Available' : 'Not installed'}</span><input type="checkbox" aria-label={`Enable ${sourceNames[key]}`} disabled={!store.sources[key]} checked={prefs.preferences.sources[key]} onChange={event => prefs.setPreferences(previous => ({ ...previous, sources: { ...previous.sources, [key]: event.target.checked } }))} /></label>)}</div>
      <p className="source-note">The curated catalog stays browsable with sources disabled. Live search queries Winget and your configured Scoop buckets. Run scoop update to refresh Scoop’s local manifests. Chocolatey supports catalog packages and installed-package detection.</p>
      <div className="preference-settings"><h3>Appearance & tasks</h3><label>Text size <select aria-label="Text size" value={prefs.preferences.fontScale} onChange={event => prefs.setPreferences(previous => ({ ...previous, fontScale: Number(event.target.value) }))}>{[1,1.1,1.2,1.3].map(size => <option key={size} value={size}>{Math.round(size*100)}%</option>)}</select></label><label>Task timeout <select aria-label="Task timeout" value={prefs.preferences.timeoutMinutes} onChange={event => prefs.setPreferences(previous => ({ ...previous, timeoutMinutes: Number(event.target.value) }))}>{[5,15,30,60,120].map(minutes => <option key={minutes} value={minutes}>{minutes} minutes</option>)}</select></label></div>
      <div className="preference-settings"><h3>Winstore {appUpdates.info?.current_version ?? ''}</h3><label>Check for Winstore updates at startup<input type="checkbox" aria-label="Check for Winstore updates at startup" checked={prefs.preferences.autoCheckUpdates} onChange={event => prefs.setPreferences(previous => ({ ...previous, autoCheckUpdates: event.target.checked }))} /></label><p className="source-note">{appUpdates.info?.configured ? 'Updates download in your browser. Open the unsigned installer to update Winstore.' : 'This build has no update feed configured. Installer downloads are manual.'}</p><button className="secondary-button" disabled={!appUpdates.info?.configured || appUpdates.checking} onClick={() => void appUpdates.check()}>{appUpdates.checking ? 'Checking…' : 'Check for Winstore updates'}</button>{appUpdates.checked && !appUpdates.info?.version && <p role="status">Winstore is up to date.</p>}{appUpdates.info?.version && <><p>Winstore {appUpdates.info.version} is available.</p><p className="release-notes">{appUpdates.info.notes}</p><button className="secondary-button" onClick={() => void appUpdates.download()}>Download Winstore update</button></>}{appUpdates.error && <p role="alert" className="source-warning">{appUpdates.error}</p>}</div>
      {prefs.storageError && <p role="alert" className="source-warning">{prefs.storageError}</p>}
      {store.notice && <p className="source-warning">{store.notice}</p>}
      <p className="source-note">App icons belong to their projects. Additional icons: <button className="icon-credit-link" onClick={() => void openUrl('https://selfh.st/icons').catch(() => store.setError('Could not open icon credits.'))}>selfh.st/icons</button> (CC BY 4.0).</p>
    </Dialog>}
    {overlay === 'queue' && <TaskQueue queue={taskQueue} onClose={() => setOverlay(null)} />}
  </div>;
}
