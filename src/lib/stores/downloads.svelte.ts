// stores/downloads.svelte.ts — Download manager store (Svelte 5 Runes)
import { listen } from '@tauri-apps/api/event';

export type DownloadStatus = 'downloading' | 'extracting' | 'done' | 'error' | 'cancelled';

export interface DownloadItem {
  mod_id: number;
  filename: string;
  mod_name?: string;
  hero_name?: string | null;
  thumbnail_url?: string;
  status: DownloadStatus;
  progress: number;        // 0.0 – 1.0
  message: string;
  speed_mbps: number;
  total_bytes?: number;
  downloaded_bytes?: number;
  started_at: number;      // Date.now()
}

let _downloads = $state<DownloadItem[]>([]);
let _listenerSetup = false;

export function getDownloads(): DownloadItem[] {
  return _downloads;
}

export function getActiveDownloads(): DownloadItem[] {
  return _downloads.filter((d) => d.status === 'downloading' || d.status === 'extracting');
}

export function getCompletedDownloads(): DownloadItem[] {
  return _downloads.filter((d) => d.status === 'done' || d.status === 'error' || d.status === 'cancelled');
}

export function clearCompleted() {
  _downloads = _downloads.filter(
    (d) => d.status === 'downloading' || d.status === 'extracting'
  );
}

export function upsertDownload(item: Omit<DownloadItem, 'started_at'> & { started_at?: number }) {
  const existing = _downloads.findIndex((d) => d.mod_id === item.mod_id && d.filename === item.filename);
  const entry: DownloadItem = { started_at: Date.now(), ...item };
  if (existing >= 0) {
    _downloads[existing] = { ..._downloads[existing], ...entry };
  } else {
    _downloads = [entry, ..._downloads];
  }
}

export function cancelDownload(mod_id: number, filename: string) {
  const idx = _downloads.findIndex((d) => d.mod_id === mod_id && d.filename === filename);
  if (idx >= 0) {
    _downloads[idx] = { ..._downloads[idx], status: 'cancelled' };
  }
}

/** Call once in app bootstrap to listen to Tauri events. */
export async function setupDownloadListener() {
  if (_listenerSetup) return;
  _listenerSetup = true;

  await listen<{
    mod_id: number;
    filename: string;
    state: string;
    progress: number;
    message: string;
    speed_mbps: number;
    downloaded_bytes?: number;
    total_bytes?: number;
  }>('mod_download_progress', (event) => {
    const {
      mod_id,
      filename,
      state,
      progress,
      message,
      speed_mbps,
      downloaded_bytes,
      total_bytes,
    } = event.payload;
    upsertDownload({
      mod_id,
      filename,
      status: state as DownloadStatus,
      progress,
      message,
      speed_mbps,
      downloaded_bytes,
      total_bytes,
    });
  });
}

/** ETA computation in human-readable form */
export function computeEta(item: DownloadItem): string {
  if (!item.total_bytes || !item.downloaded_bytes || item.speed_mbps === 0) return '—';
  const remaining = item.total_bytes - item.downloaded_bytes;
  const bytesPerSec = item.speed_mbps * 1_048_576;
  const seconds = Math.ceil(remaining / bytesPerSec);
  if (seconds < 60) return `${seconds}с`;
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}м ${secs}с`;
}
