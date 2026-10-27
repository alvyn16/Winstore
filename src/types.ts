export type PackageSource = 'winget' | 'scoop' | 'choco';

export interface SourceOption {
  source: PackageSource;
  package_id: string;
  version: string;
}

export interface PublisherDownload {
  url: string;
  release_url: string;
  version: string;
  size_bytes: number;
}

export interface AppItem {
  id: string;
  name: string;
  publisher: string;
  description: string;
  category: string;
  icon: string;
  banner?: string;
  website?: string;
  publisher_download?: PublisherDownload | null;
  screenshots?: { src: string; caption: string }[];
  is_verified: boolean;
  is_installed: boolean;
  has_update: boolean;
  installations?: Installation[];
  default_source: PackageSource | null;
  sources: SourceOption[];
  tags: string[];
}

export interface Installation {
  source: PackageSource;
  package_id: string;
  version: string;
  available_version: string | null;
  scope: string;
  held: boolean;
}

export interface PackageDetails {
  version: string | null;
  architectures: string[];
  size_bytes: number | null;
  release_notes: string | null;
  release_notes_url: string | null;
  origin: string;
  warning: string;
}

export interface SourceStatus {
  winget: boolean;
  scoop: boolean;
  choco: boolean;
}

export interface InstallResult {
  success: boolean;
  message: string;
  output: string;
}

export interface QueueItem {
  id: string;
  action: 'install' | 'uninstall' | 'update';
  app: AppItem;
  source: PackageSource;
  package_id: string;
  scope: string | null;
  timeout_seconds: number;
  status: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'timed_out' | 'interrupted';
  message: string;
  output: string;
  created_at: number;
  started_at: number | null;
  finished_at: number | null;
  attempt: number;
  revision: number;
}
