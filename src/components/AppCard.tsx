import { useState } from 'react';
import { Package, Check } from 'lucide-react';
import type { AppItem } from '../types';

const iconsNeedingLightSurface = new Set([
  'GitHub.cli', 'GitHub.GitHubDesktop', 'Schniz.fnm', 'JesseDuffield.lazygit',
  'eloston.ungoogled-chromium', 'Hashicorp.Vagrant',
]);

export function AppIcon({ app }: { app: AppItem }) {
  const [failed, setFailed] = useState(false);
  return <span className={`app-icon ${iconsNeedingLightSurface.has(app.id) ? 'app-icon-backed' : ''}`}>{failed || !app.icon
    ? <Package aria-hidden="true" />
    : <img src={app.icon} alt="" loading="lazy" onError={() => setFailed(true)} />}</span>;
}

export function AppCard({ app, onOpen, className = '' }: {
  app: AppItem; onOpen: (app: AppItem) => void; className?: string;
}) {
  return <button className={`app-card ${className}`} onClick={() => onOpen(app)}>
    <AppIcon app={app} />
    <span className="app-card-copy">
      <span className="app-name">{app.name}</span>
      <span className="app-description">{app.description}</span>
    </span>
    {app.is_installed && <Check className="installed-mark" aria-label="Installed" size={15} />}
  </button>;
}
