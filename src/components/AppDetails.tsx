import { useEffect, useState } from 'react';
import { ArrowLeft, X, Sun, Moon, Bookmark, ExternalLink, Download, Monitor, Package, Layers, CircleCheck, Tag, AlertCircle, LoaderCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { AppItem, PackageSource, SourceStatus, PackageDetails } from '../types';
import { appMedia } from '../appMedia';
import { AppIcon } from './AppCard';
import { Dialog } from './Dialog';
import { ScreenshotGallery } from './ScreenshotGallery';
import './AppDetails.css';

const sourceNames = { winget: 'Winget', scoop: 'Scoop', choco: 'Chocolatey' };
type PackageAction = 'install' | 'update' | 'uninstall';

export function AppDetails({ app, source, setSource, sources, enabled, busy, dark, saved, onToggleSaved, onToggleTheme, onClose, onAction }: {
  app: AppItem; source: PackageSource; setSource: (source: PackageSource) => void;
  sources: SourceStatus; enabled: SourceStatus; busy: boolean; dark: boolean; saved: boolean; onToggleSaved: () => void;
  onToggleTheme: () => void; onClose: () => void; onAction: (action: PackageAction, scope?: string) => void;
}) {
  const [error, setError] = useState('');
  const [downloadOpened, setDownloadOpened] = useState(false);
  const [scope, setScope] = useState('');
  const media = appMedia(app.id, app.website, app.screenshots);
  const publisherOnly = app.sources.length === 0 && !!app.publisher_download;
  const option = app.sources.find(item => item.source === source);
  const detailsKey = publisherOnly ? 'publisher:' + app.id : source + ':' + option?.package_id;
  const [loadedDetails, setLoadedDetails] = useState<{ key: string; data?: PackageDetails; error?: string } | null>(null);
  const details = loadedDetails?.key === detailsKey ? loadedDetails : null;
  useEffect(() => {
    let cancelled = false;
    if (!publisherOnly && !option) return;
    void invoke<PackageDetails>('get_package_details', { source: publisherOnly ? 'publisher' : source, packageId: publisherOnly ? app.id : option!.package_id })
      .then(data => { if (!cancelled) setLoadedDetails({ key: detailsKey, data }); })
      .catch(cause => { if (!cancelled) setLoadedDetails({ key: detailsKey, error: String(cause) }); });
    return () => { cancelled = true; };
  }, [detailsKey]);
  const downloadBytes = details?.data?.size_bytes ?? (publisherOnly ? app.publisher_download?.size_bytes : null);
  const availableVersion = details?.data?.version ?? (publisherOnly ? app.publisher_download?.version : installationVersion());
  function installationVersion() { return (app.installations?.find(item => item.source === source)?.available_version) ?? (option?.version !== 'Latest' ? option?.version : undefined); }
  async function openReleaseNotes() { const url = details?.data?.release_notes_url ?? (publisherOnly ? app.publisher_download?.release_url : undefined); if (url) try { await openUrl(url); } catch { setError('Could not open release notes.'); } }

  const available = sources[source] && enabled[source];
  const copies = app.installations?.filter(item => item.source === source) ?? [];
  const installation = copies.find(item => item.scope === scope) ?? copies[0];
  const ambiguous = installation?.scope === 'ambiguous' || installation?.scope === 'overlap';
  const action: PackageAction = installation?.available_version && !installation.held ? 'update' : installation ? 'uninstall' : 'install';
  const facts = [
    { icon: Download, value: downloadBytes ? `${(downloadBytes / 1024 / 1024).toFixed(1)} MB` : 'Not reported', label: 'Download size' },
    { icon: Package, value: publisherOnly ? app.publisher : sourceNames[source], label: publisherOnly ? 'Publisher' : 'Package source' },
    { icon: Tag, value: installation?.version ?? details?.data?.version ?? (publisherOnly ? app.publisher_download?.version : availableVersion) ?? (details ? 'Not reported' : 'Checking…'), label: installation ? 'Installed version' : 'Version' },
    { icon: Monitor, value: details?.data?.architectures.length ? details.data.architectures.join(', ') : publisherOnly ? 'x64' : 'Not reported', label: 'Architecture' },
    { icon: Layers, value: String(app.sources.length + (app.publisher_download ? 1 : 0)), label: 'Download options' },
    { icon: CircleCheck, value: installation?.held ? 'Held' : installation?.available_version ? 'Update ready' : installation ? 'Installed' : 'Available', label: 'Status' },
  ];
  async function visitWebsite() {
    if (!media.website) return;
    try { await openUrl(media.website); }
    catch { setError('Could not open the publisher’s website. Please try again.'); }
  }
  async function downloadInstaller() {
    if (!app.publisher_download) return;
    setError('');
    try {
      await openUrl(app.publisher_download.url);
      setDownloadOpened(true);
    } catch { setError('Could not open the standalone download. Try the publisher’s website.'); }
  }
  return <Dialog title={app.name} className="app-details-dialog" onClose={onClose} header={
    <header className="details-navigation">
      <button className="icon-button" aria-label="Back to store" title="Back to store" onClick={onClose}><ArrowLeft size={23} /></button>
      <div><button className="icon-button" aria-label={dark ? 'Switch details to light theme' : 'Switch details to dark theme'} title={dark ? 'Light theme' : 'Dark theme'} onClick={onToggleTheme}>{dark ? <Sun size={19} /> : <Moon size={19} />}</button><button className="icon-button details-close" aria-label="Close app details" onClick={onClose}><X size={20} /></button></div>
    </header>
  }>
    <div className="details-identity">
      <AppIcon app={app} />
      <div className="details-title"><h1>{app.name}</h1><p>{app.publisher}</p><div className="details-chips">
        {media.website && <button className="website-chip" onClick={() => void visitWebsite()}><ExternalLink size={15} />Website</button>}
        <button className="save-chip" aria-pressed={saved} onClick={onToggleSaved}><Bookmark size={16} fill={saved ? 'currentColor' : 'none'} />{saved ? 'Saved' : 'Save'}</button>
      </div></div>
      <div className="details-install"><button className={`details-install-button ${action === 'uninstall' ? 'details-uninstall-button' : ''}`} disabled={busy || ambiguous || (!publisherOnly && (!available || !option))} onClick={() => { if (publisherOnly) void downloadInstaller(); else onAction(action, installation?.scope); }}>{busy ? <><LoaderCircle className="spin" size={19} />Working…</> : publisherOnly ? 'Download' : action === 'update' ? 'Update' : action === 'uninstall' ? 'Uninstall' : 'Install'}</button><span>{busy ? 'A package task is running' : publisherOnly ? 'Standalone Windows x64 installer' : !available ? 'Choose an available source below' : `via ${sourceNames[source]}${installation ? " · " + installation.scope : ""}`}</span>{downloadOpened && <span role="status">Download opened in your browser</span>}</div>
    </div>
    {error && <p className="details-error" role="alert"><AlertCircle size={16} />{error}</p>}
    <dl className="details-facts">{facts.map(({ icon: Icon, value, label }) => <div key={label}><dt>{label}</dt><dd><Icon size={18} /><span>{value}</span></dd></div>)}</dl>
    <ScreenshotGallery previews={media.previews} name={app.name} />
    <div className="details-about">
      <section className="details-description"><h2>About {app.name}</h2><p>{app.description}</p>{app.publisher === 'Artcraft' && <p className="early-release-note">Early release. Some features are still in development.</p>}<div className="version-information"><h3>Versions & release notes</h3><dl><div><dt>Installed</dt><dd>{installation?.version ?? (app.is_installed ? 'Installed through another source' : 'Not detected')}</dd></div><div><dt>Available from {publisherOnly ? app.publisher : sourceNames[source]}</dt><dd>{availableVersion ?? (details ? 'Not reported' : 'Checking…')}</dd></div></dl>{!details && <p role="status">Checking package details…</p>}{details?.error && <p className="source-warning">Package details are unavailable: {details.error}</p>}{details?.data?.warning && <p className="source-warning">{details.data.warning}</p>}{details?.data?.origin && <small>{details.data.origin}</small>}{(details?.data?.release_notes_url || publisherOnly) && <button className="secondary-button" onClick={() => void openReleaseNotes()}><ExternalLink size={14} />{source === 'choco' && !publisherOnly ? 'Package release information' : 'Release notes'}</button>}{details?.data?.release_notes ? <details><summary>Read release notes</summary><pre className="release-notes">{details.data.release_notes}</pre></details> : details && !details.data?.release_notes_url && !publisherOnly && <p className="source-note">Release notes were not provided by this source.</p>}</div><div className="details-tags"><span>{app.category}</span>{app.tags.filter(tag => tag.toLowerCase() !== app.category.toLowerCase()).map(tag => <span key={tag}>{tag}</span>)}</div></section>
      <section className="details-package"><h2>{publisherOnly ? 'Publisher download' : 'Installation source'}</h2>{!publisherOnly && <><p>Choose where to get this app.</p><div className="source-options">{app.sources.map(item => <button key={item.source} aria-pressed={source === item.source} className={source === item.source ? 'selected' : ''} onClick={() => setSource(item.source)}>{sourceNames[item.source]}<small>{item.source === source && availableVersion ? availableVersion : item.version === 'Latest' ? 'Check version' : item.version}</small></button>)}</div><code>{installation?.package_id ?? option?.package_id}</code>
        {copies.length > 1 && <label>Installation <select aria-label="Installation scope" value={installation.scope} onChange={event => setScope(event.target.value)}>{copies.map(copy => <option key={copy.scope} value={copy.scope}>{copy.scope} · {copy.version}</option>)}</select></label>}
        {installation && <p className="source-note">{installation?.scope === 'overlap' ? 'This registration overlaps a managed copy. Use its Scoop or Chocolatey source.' : ambiguous ? 'Multiple registrations detected. Manage these copies in Windows Settings; Winstore cannot safely choose one.' : source === 'winget' ? 'Detected by Winget. The original installer is unknown.' : 'Managed by ' + sourceNames[source] + ' · ' + installation.scope}{installation.available_version && <> · Available: {installation.available_version}</>}{installation.held && <> · Updates held in your package manager.</>}</p>}
        {action === 'update' && <button className="secondary-button" disabled={busy || ambiguous || !available} onClick={() => onAction('uninstall', installation?.scope)}>Uninstall this copy</button>}
        {!installation && app.is_installed && <p className="source-note">This source has no detected installation. Install adds another copy.</p>}<p className="source-note">{source === 'scoop' ? 'Installs in your user folder by default.' : 'The installer may request administrator permission.'}</p>{!available && <p className="source-warning"><AlertCircle size={15} />{sourceNames[source]} is {sources[source] ? 'disabled. Enable it in Package sources.' : 'not installed on this PC. Choose another source.'}</p>}</>}{app.publisher_download && <div className="publisher-download"><p>Standalone installer. No Artcraft launcher.</p><button className="secondary-button" onClick={() => void downloadInstaller()}><Download size={14} />Download v{app.publisher_download.version}</button><p>Windows x64 · {Math.ceil(app.publisher_download.size_bytes / 1024 / 1024)} MB</p><p>Open the downloaded installer to install this app.</p></div>}</section>
    </div>
  </Dialog>;
}
