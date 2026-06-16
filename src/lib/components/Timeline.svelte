<script lang="ts">
  import { videoSrc, videoDuration, trimStart, trimEnd, currentTime, seekRequest } from '$lib/store';
  import { generateThumbnails, type ThumbnailResult } from '$lib/thumbnails';
  import { browser } from '$app/environment';

  let containerWidth = 0;
  let thumbnails: ThumbnailResult[] = [];
  let isGenerating = false;
  let thumbnailError = false;

  // Convert time to pixels
  $: pxPerSec = $videoDuration > 0 ? containerWidth / $videoDuration : 0;

  // Handle positions
  $: startX = $trimStart * pxPerSec;
  $: endX = $trimEnd * pxPerSec || containerWidth;
  
  let isDraggingPlayhead = false;
  let dragPlayheadTime = 0;
  let lastSeekTime = 0;

  $: playheadX = (isDraggingPlayhead ? dragPlayheadTime : $currentTime) * pxPerSec;


  // Time ruler tick marks
  $: tickInterval = computeTickInterval($videoDuration, containerWidth);
  $: ticks = generateTicks($videoDuration, tickInterval);

  function computeTickInterval(duration: number, width: number): number {
    if (duration <= 0 || width <= 0) return 1;
    // Target roughly 1 tick every 80-120px
    const approxTicks = width / 100;
    const rawInterval = duration / approxTicks;

    // Snap to human-friendly intervals
    const intervals = [0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600];
    for (const iv of intervals) {
      if (rawInterval <= iv) return iv;
    }
    return 600;
  }

  function generateTicks(duration: number, interval: number): { time: number; label: string; major: boolean }[] {
    if (duration <= 0 || interval <= 0) return [];
    const result = [];
    for (let t = 0; t <= duration; t += interval) {
      result.push({
        time: t,
        label: formatTimecode(t),
        major: t % (interval * 2) === 0 || t === 0
      });
    }
    return result;
  }

  function formatTimecode(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
    return `${m}:${String(s).padStart(2, '0')}`;
  }

  let lastGeneratedSrc = '';

  // Thumbnail generation
  $: if (browser && $videoSrc && $videoDuration > 0) {
    // Only generate once per video. Prevent window resizing from triggering infinite decoding loops!
    if (lastGeneratedSrc !== $videoSrc && containerWidth > 0) {
      lastGeneratedSrc = $videoSrc;
      // Dynamically calculate a healthy amount of thumbnails for the current screen size (cap at 30)
      const count = Math.max(8, Math.min(30, Math.floor(containerWidth / 60)));
      generateThumbs($videoSrc, $videoDuration, count);
    }
  }

  let abortController: AbortController | null = null;

  async function generateThumbs(src: string, duration: number, count: number) {
    if (abortController) {
      abortController.abort();
    }
    abortController = new AbortController();
    const signal = abortController.signal;

    isGenerating = true;
    thumbnailError = false;
    thumbnails = [];
    try {
      await generateThumbnails(src, duration, count, 60, (thumb) => {
        if (!signal.aborted) {
          thumbnails = [...thumbnails, thumb];
        }
      }, signal);
      
      if (!signal.aborted) {
        thumbnailError = thumbnails.length === 0;
      }
    } catch {
      if (!signal.aborted) {
        thumbnailError = true;
      }
    } finally {
      if (!signal.aborted) {
        isGenerating = false;
      }
    }
  }

  // Dragging logic
  function draggable(node: HTMLElement, type: 'start' | 'end' | 'playhead') {
    let startClientX: number;

    function handleMousedown(event: MouseEvent) {
      event.preventDefault();
      event.stopPropagation();
      startClientX = event.clientX;
      if (type === 'playhead') {
        isDraggingPlayhead = true;
        dragPlayheadTime = $currentTime;
      }
      document.body.style.cursor = 'ew-resize';
      document.body.style.userSelect = 'none';
      window.addEventListener('mousemove', handleMousemove);
      window.addEventListener('mouseup', handleMouseup);
    }

    function handleMousemove(event: MouseEvent) {
      if (!$videoDuration || !pxPerSec) return;
      const dx = event.clientX - startClientX;
      startClientX = event.clientX;
      const dt = dx / pxPerSec;

      if (type === 'start') {
        let v = $trimStart + dt;
        $trimStart = Math.max(0, Math.min(v, $trimEnd - 0.1));
      } else if (type === 'end') {
        let v = $trimEnd + dt;
        $trimEnd = Math.min($videoDuration, Math.max(v, $trimStart + 0.1));
      } else if (type === 'playhead') {
        dragPlayheadTime = Math.max(0, Math.min($videoDuration, dragPlayheadTime + dt));
        // Throttle actual video player seeking to avoid locking the UI thread
        const now = performance.now();
        if (now - lastSeekTime > 80) { // 80ms throttle (~12fps update rate for seeking)
          $seekRequest = dragPlayheadTime;
          lastSeekTime = now;
        }
      }
    }

    function handleMouseup() {
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      if (type === 'playhead') {
        isDraggingPlayhead = false;
        $currentTime = dragPlayheadTime;
        $seekRequest = dragPlayheadTime;
      }
      window.removeEventListener('mousemove', handleMousemove);
      window.removeEventListener('mouseup', handleMouseup);
    }

    node.addEventListener('mousedown', handleMousedown);
    return { destroy() { node.removeEventListener('mousedown', handleMousedown); } };
  }

  // Click to seek
  function handleTimelineClick(event: MouseEvent) {
    if (!$videoDuration || !pxPerSec) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;
    const time = Math.max(0, Math.min($videoDuration, x / pxPerSec));
    $currentTime = time;
    $seekRequest = time;
  }
</script>

<div class="w-full h-full modern-card select-none">
  <div class="modern-card-inner flex flex-col relative">
    <div class="noise-overlay"></div>
    <div class="relative z-10 flex flex-col h-full w-full">
      <!-- Header -->
      <div class="flex justify-between items-center px-4 py-2 shrink-0 border-b border-borderBase">
        <span class="text-[11px] font-semibold text-textSecondary uppercase tracking-widest">Timeline</span>
    {#if $videoSrc}
      <div class="flex items-center gap-4">
        {#if thumbnailError}
          <span class="text-[10px] text-warning">⚠ Thumbnails N/A</span>
        {/if}
        <div class="flex items-center gap-2 bg-bg rounded-md px-2 py-0.5 border border-borderBase">
          <span class="text-[10px] text-accent font-mono font-semibold">{formatTimecode($trimStart)}</span>
          <span class="text-[10px] text-textMuted">→</span>
          <span class="text-[10px] text-accent font-mono font-semibold">{formatTimecode($trimEnd)}</span>
          <span class="text-[10px] text-textMuted ml-1">({formatTimecode($trimEnd - $trimStart)})</span>
        </div>
      </div>
    {/if}
  </div>

  {#if $videoSrc}
    <!-- Time Ruler -->
    <div class="h-[22px] shrink-0 relative bg-bg/50 border-b border-borderBase overflow-hidden">
      {#each ticks as tick}
        <div
          class="absolute top-0 h-full flex flex-col items-center"
          style="left: {tick.time * pxPerSec}px"
        >
          <div
            class="w-px bg-borderBase"
            class:h-full={tick.major}
            class:h-[10px]={!tick.major}
            class:bg-textMuted={tick.major}
          ></div>
          {#if tick.major}
            <span class="absolute bottom-[2px] text-[9px] text-textMuted font-mono whitespace-nowrap translate-x-[2px]">
              {tick.label}
            </span>
          {/if}
        </div>
      {/each}

      <!-- Playhead marker on ruler -->
      <div
        class="absolute top-0 h-full w-px bg-accent z-10"
        style="left: {playheadX}px"
      ></div>
    </div>

    <!-- Track Wrapper with horizontal padding so handles don't hit the screen edge -->
    <div class="flex-1 px-5 pb-5 pt-2">
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="relative w-full h-full" bind:clientWidth={containerWidth}>

        <!-- Track Content (Clipped) -->
        <div class="absolute inset-0 rounded-[8px] overflow-hidden border border-borderBase cursor-crosshair shadow-inner" on:click={handleTimelineClick}>
          
          <!-- Thumbnails -->
          <div class="absolute inset-0 flex overflow-hidden">
            {#if isGenerating && thumbnails.length === 0}
              <div class="w-full h-full bg-card animate-pulse"></div>
            {:else if thumbnails.length > 0}
              {#each thumbnails as thumb}
                <img src={thumb.dataUrl} alt="" class="flex-1 min-w-0 h-full object-cover opacity-70 animate-in fade-in duration-300 border-r border-borderBase last:border-r-0" draggable="false" />
              {/each}
              {#if isGenerating}
                <!-- Fill the remaining space with a placeholder that flexes to fill the gap -->
                <div class="flex-1 bg-card animate-pulse"></div>
              {/if}
            {:else}
              <div class="w-full h-full bg-gradient-to-r from-card via-surface to-card flex items-center justify-center">
                <span class="text-[11px] text-textMuted">Preview unavailable — video can still be cut</span>
              </div>
            {/if}
          </div>

          <!-- Dark overlays for unselected regions -->
          <div class="absolute top-0 bottom-0 left-0 bg-black/60 pointer-events-none" style="width: {startX}px; will-change: width;"></div>
          <div class="absolute top-0 bottom-0 right-0 bg-black/60 pointer-events-none" style="left: {endX}px; will-change: left;"></div>

          <!-- Selection border (top & bottom accent lines) -->
          <div class="absolute top-0 h-[2px] bg-accent pointer-events-none" style="transform: translate3d({startX}px, 0, 0); width: {endX - startX}px; will-change: transform, width; left: 0;"></div>
          <div class="absolute bottom-0 h-[2px] bg-accent pointer-events-none" style="transform: translate3d({startX}px, 0, 0); width: {endX - startX}px; will-change: transform, width; left: 0;"></div>

          <!-- Time separators (vertical lines at tick positions) -->
          {#each ticks as tick}
            {#if tick.major}
              <div
                class="absolute top-0 bottom-0 w-px bg-black/10 dark:bg-white/10 pointer-events-none"
                style="left: {tick.time * pxPerSec}px"
              ></div>
            {/if}
          {/each}
        </div>

        <!-- Handles (Unclipped - safe to render outside the track) -->
        
        <!-- Trim Start Handle -->
        <div
          use:draggable={'start'}
          class="absolute top-0 bottom-0 w-[10px] bg-accent hover:bg-accent-hover cursor-ew-resize flex items-center justify-center z-10 rounded-l-[4px] hover:bg-opacity-90 shadow-[2px_0_12px_rgba(76,141,255,0.4)]"
          style="transform: translate3d(calc({startX}px - 100%), 0, 0); will-change: transform; left: 0;"
        >
          <div class="w-[2px] h-[20px] bg-white/70 rounded-full"></div>
        </div>

        <!-- Trim End Handle -->
        <div
          use:draggable={'end'}
          class="absolute top-0 bottom-0 w-[10px] bg-accent hover:bg-accent-hover cursor-ew-resize flex items-center justify-center z-10 rounded-r-[4px] hover:bg-opacity-90 shadow-[-2px_0_12px_rgba(76,141,255,0.4)]"
          style="transform: translate3d({endX}px, 0, 0); will-change: transform; left: 0;"
        >
          <div class="w-[2px] h-[20px] bg-white/70 rounded-full"></div>
        </div>

        <!-- Playhead -->
        <div
          use:draggable={'playhead'}
          class="absolute -top-[4px] -bottom-[4px] w-[2px] bg-black dark:bg-white cursor-ew-resize z-20 group"
          style="transform: translate3d(calc({playheadX}px - 1px), 0, 0); will-change: transform; left: 0;"
        >
          <!-- Triangle head -->
          <div class="absolute -top-[1px] left-1/2 -translate-x-1/2 w-0 h-0 border-l-[6px] border-r-[6px] border-t-[8px] border-l-transparent border-r-transparent border-t-black dark:border-t-white drop-shadow-md group-hover:scale-110 transition-transform cursor-ew-resize"></div>
          <!-- Bottom triangle -->
          <div class="absolute -bottom-[1px] left-1/2 -translate-x-1/2 w-0 h-0 border-l-[6px] border-r-[6px] border-b-[8px] border-l-transparent border-r-transparent border-b-black dark:border-b-white drop-shadow-md group-hover:scale-110 transition-transform cursor-ew-resize"></div>
        </div>

      </div>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center bg-bg/50 border border-borderBase border-dashed text-textMuted text-[13px] m-3 rounded-lg">
      Select a video to interact with the timeline
    </div>
      {/if}
    </div>
  </div>
</div>
