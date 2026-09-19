import { writable, derived } from 'svelte/store';

// File & Source
export const videoFilePath = writable<string | null>(null);
export const videoSrc = writable<string | null>(null);
export const isProxying = writable<boolean>(false);
export const proxyProgress = writable<number>(0);
export const proxyEta = writable<string>('');

// Proxy configuration (Disabled by default, opt-in via Settings)
const initialEnableProxy = typeof window !== 'undefined' && localStorage.getItem('enableProxy') === 'true';
export const enableProxy = writable<boolean>(initialEnableProxy);
if (typeof window !== 'undefined') {
  enableProxy.subscribe(value => localStorage.setItem('enableProxy', value.toString()));
}

// Playback State
export const videoDuration = writable<number>(0);
export const currentTime = writable<number>(0);
export const isPlaying = writable<boolean>(false);
export const isFullscreen = writable<boolean>(false);

// Playback Interaction State
export const isUserInteracting = writable<boolean>(false);

// Timeline & Trimming
export const trimStart = writable<number>(0);
export const trimEnd = writable<number>(0);
export const zoomLevel = writable<number>(10);

// Video Canvas (Zoom/Pan)
export const videoZoom = writable<number>(1);
export const videoPanX = writable<number>(0);
export const videoPanY = writable<number>(0);

// Audio Controls
export const volume = writable<number>(1);
export const muted = writable<boolean>(false);

// Seek request: Timeline writes this, VideoPreview reads & seeks
export const seekRequest = writable<number | null>(null);

// Trigger export globally
export const exportRequest = writable<number>(0);

// Theme
const initialTheme = typeof window !== 'undefined' && localStorage.getItem('isDarkMode') !== 'false';
export const isDarkMode = writable<boolean>(initialTheme);
if (typeof window !== 'undefined') {
  isDarkMode.subscribe(value => localStorage.setItem('isDarkMode', value.toString()));
}

// Settings panel visibility
export const showSettings = writable<boolean>(false);

// Export Settings
const initialFormat = (typeof window !== 'undefined' && localStorage.getItem('exportFormat')) || 'ORIGINAL';
export const exportFormat = writable<string>(initialFormat);

const initialExportQuality = typeof window !== 'undefined' && localStorage.getItem('exportQuality') 
  ? parseInt(localStorage.getItem('exportQuality') as string) 
  : 100;
export const exportQuality = writable<number>(initialExportQuality);

const initialAutoSnap = typeof window !== 'undefined' && localStorage.getItem('autoSnap') !== 'false';
export const autoSnap = writable<boolean>(initialAutoSnap);

if (typeof window !== 'undefined') {
  exportFormat.subscribe(value => localStorage.setItem('exportFormat', value));
  exportQuality.subscribe(value => localStorage.setItem('exportQuality', value.toString()));
  autoSnap.subscribe(value => localStorage.setItem('autoSnap', value.toString()));
}

// Preview quality
const initialPreviewQuality = typeof window !== 'undefined' && localStorage.getItem('previewQuality') 
  ? parseInt(localStorage.getItem('previewQuality') as string) 
  : 50;
export const previewQuality = writable<number>(initialPreviewQuality);
if (typeof window !== 'undefined') {
  previewQuality.subscribe(value => localStorage.setItem('previewQuality', value.toString()));
}

// Video Metadata
export const videoMetadata = writable<any>(null);

// Trigger project loading globally
export const projectLoadRequest = writable<any>(null);

// Trigger Open/Paste Video Dialogs
export const openVideoRequest = writable<number>(0);
export const pasteVideoRequest = writable<number>(0);

// Derived Data
export const selectedDuration = derived(
  [trimStart, trimEnd],
  ([$start, $end]) => Math.max(0, $end - $start)
);

// History Dashboard
export type ExportRecord = {
  id: string;
  sourcePath: string;
  outputPath: string;
  duration: number;
  date: string;
};

const initialHistory = typeof window !== 'undefined' && localStorage.getItem('exportHistory') 
  ? JSON.parse(localStorage.getItem('exportHistory') as string) as ExportRecord[]
  : [];
export const exportHistory = writable<ExportRecord[]>(initialHistory);

if (typeof window !== 'undefined') {
  exportHistory.subscribe(value => localStorage.setItem('exportHistory', JSON.stringify(value)));
}

// Session & Workspace Management
export async function closeVideoSession() {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('cancel_proxy');
  } catch (_) {}
  isProxying.set(false);
  proxyProgress.set(0);
  proxyEta.set('');
  isPlaying.set(false);
  videoSrc.set(null);
  videoFilePath.set(null);
  videoMetadata.set(null);
  videoDuration.set(0);
  currentTime.set(0);
  trimStart.set(0);
  trimEnd.set(0);
  zoomLevel.set(10);
  videoZoom.set(1);
  videoPanX.set(0);
  videoPanY.set(0);
  seekRequest.set(null);
}
