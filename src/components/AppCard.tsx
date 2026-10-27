import { Check } from 'lucide-react';
import type { AppItem } from '../types';
import { useAppIcon } from '../useAppIcon';

const iconsNeedingLightSurface = new Set([
  'GitHub.cli', 'GitHub.GitHubDesktop', 'Schniz.fnm', 'JesseDuffield.lazygit',
  'eloston.ungoogled-chromium', 'Hashicorp.Vagrant',
]);

export function AppIcon({ app }: { app: AppItem }) {
  const icon = useAppIcon(app);
  const initials = app.name.match(/[\p{L}\p{N}]+/gu)?.slice(0,2).map(word => word[0]).join('').toUpperCase() || '?';
  const backed = iconsNeedingLightSurface.has(app.id) || app.sources.some(option => iconsNeedingLightSurface.has(option.package_id));
  return <span ref={icon.ref} className={`app-icon ${backed ? 'app-icon-backed' : ''}`}>{icon.src
    ? <img src={icon.src} alt="" loading="lazy" onError={icon.onError} />
    : <span className="app-icon-initials" aria-hidden="true">{initials}</span>}</span>;
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
