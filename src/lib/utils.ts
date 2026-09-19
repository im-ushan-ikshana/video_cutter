export function formatTimecode(seconds: number): string {
  if (isNaN(seconds) || seconds < 0) return "00:00:00.000";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  const ms = Math.floor((seconds % 1) * 1000);
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(ms).padStart(3, '0')}`;
}

export function parseTimecode(tc: string): number | null {
  const parts = tc.split(':');
  if (parts.length === 3) {
    const [h, m, s_ms] = parts;
    const sParts = s_ms.split('.');
    const s = sParts[0];
    const ms = sParts.length > 1 ? sParts[1].padEnd(3, '0').slice(0, 3) : '000';
    return parseInt(h) * 3600 + parseInt(m) * 60 + parseInt(s) + parseInt(ms) / 1000;
  } else if (parts.length === 2) {
    const [m, s_ms] = parts;
    const sParts = s_ms.split('.');
    const s = sParts[0];
    const ms = sParts.length > 1 ? sParts[1].padEnd(3, '0').slice(0, 3) : '000';
    return parseInt(m) * 60 + parseInt(s) + parseInt(ms) / 1000;
  }
  return null;
}

export function formatTime(seconds: number): string {
    if (isNaN(seconds) || seconds < 0) return "0s";
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}h ${m}m ${s}s`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
}
