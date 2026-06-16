import { writable, derived } from 'svelte/store';

// File & Source
export const videoFilePath = writable<string | null>(null);
export const videoSrc = writable<string | null>(null);
export const isProxying = writable<boolean>(false);
export const proxyProgress = writable<number>(0);
export const proxyEta = writable<string>('');

// Playback State
export const videoDuration = writable<number>(0);
export const currentTime = writable<number>(0);
export const isPlaying = writable<boolean>(false);

// Timeline & Trimming
export const trimStart = writable<number>(0);
export const trimEnd = writable<number>(0);
export const zoomLevel = writable<number>(10);

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
if (typeof window !== 'undefined') {
  exportFormat.subscribe(value => localStorage.setItem('exportFormat', value));
}

// Preview quality
const initialQuality = typeof window !== 'undefined' && localStorage.getItem('previewQuality') 
  ? parseInt(localStorage.getItem('previewQuality') as string) 
  : 50;
export const previewQuality = writable<number>(initialQuality);
if (typeof window !== 'undefined') {
  previewQuality.subscribe(value => localStorage.setItem('previewQuality', value.toString()));
}

// Video Metadata
export const videoMetadata = writable<any>(null);

// Derived Data
export const selectedDuration = derived(
  [trimStart, trimEnd],
  ([$start, $end]) => Math.max(0, $end - $start)
);
