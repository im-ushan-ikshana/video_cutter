/**
 * Generates thumbnail data URLs from a video source using <video> + <canvas>.
 * This runs entirely in the browser — no FFmpeg needed.
 */

export interface ThumbnailResult {
  time: number;
  dataUrl: string;
}

export async function generateThumbnails(
  videoSrc: string,
  duration: number,
  count: number,
  height: number = 60,
  onProgress?: (thumbnail: ThumbnailResult, index: number, total: number) => void,
  signal?: AbortSignal
): Promise<ThumbnailResult[]> {
  return new Promise((resolve) => {
    if (signal?.aborted) {
      resolve([]);
      return;
    }

    const video = document.createElement('video');
    video.crossOrigin = 'anonymous';
    video.preload = 'auto';
    video.muted = true;
    video.src = videoSrc;

    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      resolve([]);
      return;
    }

    const thumbnails: ThumbnailResult[] = [];
    const interval = duration / count;
    let currentIndex = 0;

    signal?.addEventListener('abort', () => {
      video.src = '';
      video.removeAttribute('src');
      video.load();
      resolve(thumbnails);
    });

    video.addEventListener('error', () => {
      // Video can't be decoded by browser (e.g., CCTV .dav format)
      console.warn('Thumbnail generation failed: browser cannot decode this video format.');
      video.src = '';
      video.removeAttribute('src');
      video.load();
      resolve([]);
    });

    video.addEventListener('loadedmetadata', () => {
      if (signal?.aborted) return;
      // Calculate aspect ratio to determine canvas width
      const rawRatio = video.videoHeight > 0 ? (video.videoWidth / video.videoHeight) : (16 / 9);
      const aspectRatio = isFinite(rawRatio) && rawRatio > 0 ? rawRatio : (16 / 9);
      canvas.height = height;
      // Cap max width to 200 to prevent VRAM spikes on ultra-wide content
      canvas.width = Math.max(10, Math.min(Math.round(height * aspectRatio), 200));

      seekToNext();
    });

    function seekToNext() {
      if (currentIndex >= count || signal?.aborted) {
        video.src = ''; // Release video resources
        video.removeAttribute('src');
        video.load();
        resolve(thumbnails);
        return;
      }

      const targetTime = currentIndex * interval;
      video.currentTime = targetTime;
    }

    video.addEventListener('seeked', () => {
      if (signal?.aborted) return;
      if (!ctx) return;

      try {
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
        // Use WebP instead of JPEG for much faster encoding and smaller memory footprint
        const dataUrl = canvas.toDataURL('image/webp', 0.5);
        const thumb = {
          time: video.currentTime,
          dataUrl
        };
        thumbnails.push(thumb);
        if (onProgress) {
          onProgress(thumb, currentIndex, count);
        }
      } catch (e) {
        // Canvas tainted or draw failed — skip this frame
        console.warn('Failed to capture frame at', video.currentTime);
      }

      currentIndex++;
      seekToNext();
    });
  });
}
