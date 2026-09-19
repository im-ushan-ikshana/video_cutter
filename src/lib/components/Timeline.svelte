<script lang="ts">
  import { videoSrc, videoDuration, trimStart, trimEnd, currentTime, seekRequest, zoomLevel, autoSnap } from '$lib/store';
  import { generateThumbnails, type ThumbnailResult } from '$lib/thumbnails';
  import { formatTime, formatTimecode } from '$lib/utils';
  import { browser } from '$app/environment';
  import { onDestroy } from 'svelte';
  import { MagnifyingGlassMinus, MagnifyingGlassPlus } from 'phosphor-svelte';

  const HANDLE_WIDTH = 10; // Inward bracket width (px)

  let viewportWidth = 0;
  $: containerWidth = Math.max(viewportWidth, viewportWidth * ($zoomLevel / 10));
  
  let thumbnails: ThumbnailResult[] = [];
  let isGenerating = false;
  let thumbnailError = false;

  $: pxPerSec = $videoDuration > 0 ? containerWidth / $videoDuration : 0;

  $: startX = Math.max(0, Math.min(containerWidth, $trimStart * pxPerSec));
  $: endX = Math.max(startX, Math.min(containerWidth, $trimEnd * pxPerSec || containerWidth));
  
  let isDraggingPlayhead = false;
  let dragPlayheadTime = 0;
  let lastSeekTime = 0;

  $: playheadX = (isDraggingPlayhead ? dragPlayheadTime : $currentTime) * pxPerSec;

  $: tickInterval = computeTickInterval($videoDuration, containerWidth);
  $: ticks = generateTicks($videoDuration, tickInterval);

  function computeTickInterval(duration: number, width: number): number {
    if (duration <= 0 || width <= 0) return 1;
    const approxTicks = width / 100;
    const rawInterval = duration / approxTicks;

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
        label: formatRulerTick(t),
        major: t % (interval * 2) === 0 || t === 0
      });
    }
    return result;
  }

  function formatRulerTick(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
    return `${m}:${String(s).padStart(2, '0')}`;
  }

  let lastGeneratedSrc = '';

  $: if (browser && $videoSrc && $videoDuration > 0) {
    if (lastGeneratedSrc !== $videoSrc && viewportWidth > 0) {
      lastGeneratedSrc = $videoSrc;
      const count = Math.max(8, Math.min(30, Math.floor(viewportWidth / 60)));
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

  function draggable(node: HTMLElement, type: 'start' | 'end' | 'range' | 'playhead') {
    let startClientX: number;
    let initialTrimStart = 0;
    let initialTrimEnd = 0;

    function handleMousedown(event: MouseEvent) {
      if (event.button !== 0) return;
      event.preventDefault();
      event.stopPropagation();
      startClientX = event.clientX;
      initialTrimStart = $trimStart;
      initialTrimEnd = $trimEnd;

      if (type === 'playhead') {
        isDraggingPlayhead = true;
        dragPlayheadTime = $currentTime;
        document.body.style.cursor = 'ew-resize';
      } else if (type === 'range') {
        document.body.style.cursor = 'grabbing';
      } else {
        document.body.style.cursor = 'ew-resize';
      }
      document.body.style.userSelect = 'none';

      window.addEventListener('mousemove', handleMousemove);
      window.addEventListener('mouseup', handleMouseup);
    }

    function handleMousemove(event: MouseEvent) {
      if (!$videoDuration || !pxPerSec) return;
      const dx = event.clientX - startClientX;
      const dt = dx / pxPerSec;

      if (type === 'start') {
        let v = initialTrimStart + dt;
        $trimStart = Math.max(0, Math.min(v, $trimEnd - 0.1));
        $currentTime = $trimStart;
        $seekRequest = $trimStart;
      } else if (type === 'end') {
        let v = initialTrimEnd + dt;
        $trimEnd = Math.min($videoDuration, Math.max(v, $trimStart + 0.1));
        $currentTime = $trimEnd;
        $seekRequest = $trimEnd;
      } else if (type === 'range') {
        const span = initialTrimEnd - initialTrimStart;
        let newStart = initialTrimStart + dt;
        if (newStart < 0) {
          newStart = 0;
        } else if (newStart + span > $videoDuration) {
          newStart = Math.max(0, $videoDuration - span);
        }
        $trimStart = newStart;
        $trimEnd = newStart + span;
        $currentTime = newStart;
        $seekRequest = newStart;
      } else if (type === 'playhead') {
        let targetTime = Math.max(0, Math.min($videoDuration, dragPlayheadTime + (event.clientX - startClientX) / pxPerSec));
        startClientX = event.clientX;
        if ($autoSnap) {
          const snapThreshold = 8 / pxPerSec;
          if (Math.abs(targetTime - $trimStart) < snapThreshold) targetTime = $trimStart;
          else if (Math.abs(targetTime - $trimEnd) < snapThreshold) targetTime = $trimEnd;
        }
        dragPlayheadTime = targetTime;
        const now = performance.now();
        if (now - lastSeekTime > 60) {
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
      } else if (type === 'start' || type === 'range' || type === 'end') {
        $currentTime = $trimStart;
        $seekRequest = $trimStart;
      }
      window.removeEventListener('mousemove', handleMousemove);
      window.removeEventListener('mouseup', handleMouseup);
    }

    node.addEventListener('mousedown', handleMousedown);
    return {
      destroy() {
        node.removeEventListener('mousedown', handleMousedown);
        window.removeEventListener('mousemove', handleMousemove);
        window.removeEventListener('mouseup', handleMouseup);
      }
    };
  }

  onDestroy(() => {
    if (abortController) {
      abortController.abort();
    }
  });

  let trackEl: HTMLDivElement;

  function handleTrackMousedown(event: MouseEvent) {
    if (event.button !== 0 || !$videoDuration || !pxPerSec || !trackEl) return;
    const rect = trackEl.getBoundingClientRect();
    const startClientX = event.clientX;
    const initialTrackX = event.clientX - rect.left;
    const dragStartTime = Math.max(0, Math.min($videoDuration, initialTrackX / pxPerSec));
    let hasDragged = false;

    function onTrackMousemove(e: MouseEvent) {
      const deltaX = Math.abs(e.clientX - startClientX);
      if (!hasDragged && deltaX > 4) {
        hasDragged = true;
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
      }

      if (hasDragged) {
        const currentTrackX = e.clientX - rect.left;
        const currentMouseTime = Math.max(0, Math.min($videoDuration, currentTrackX / pxPerSec));
        const newStart = Math.min(dragStartTime, currentMouseTime);
        const newEnd = Math.max(dragStartTime, currentMouseTime);

        if (newEnd - newStart >= 0.05) {
          $trimStart = newStart;
          $trimEnd = newEnd;
        }
        $currentTime = currentMouseTime;
        $seekRequest = currentMouseTime;
      }
    }

    function onTrackMouseup(e: MouseEvent) {
      window.removeEventListener('mousemove', onTrackMousemove);
      window.removeEventListener('mouseup', onTrackMouseup);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';

      if (!hasDragged) {
        // Simple click without drag: seek playhead
        const clickTrackX = e.clientX - rect.left;
        let time = Math.max(0, Math.min($videoDuration, clickTrackX / pxPerSec));

        if ($autoSnap) {
          const snapThreshold = 8 / pxPerSec;
          if (Math.abs(time - $trimStart) < snapThreshold) time = $trimStart;
          else if (Math.abs(time - $trimEnd) < snapThreshold) time = $trimEnd;
        }

        $currentTime = time;
        $seekRequest = time;
      }
    }

    window.addEventListener('mousemove', onTrackMousemove);
    window.addEventListener('mouseup', onTrackMouseup);
  }

  function handleTrackDblClick() {
    $trimStart = 0;
    $trimEnd = $videoDuration;
  }

  function handleRulerClick(event: MouseEvent) {
    if (!$videoDuration || !pxPerSec) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const clickX = event.clientX - rect.left;
    let time = Math.max(0, Math.min($videoDuration, clickX / pxPerSec));

    if ($autoSnap) {
      const snapThreshold = 8 / pxPerSec;
      if (Math.abs(time - $trimStart) < snapThreshold) time = $trimStart;
      else if (Math.abs(time - $trimEnd) < snapThreshold) time = $trimEnd;
    }

    $currentTime = time;
    $seekRequest = time;
  }
</script>

<div class="w-full h-[160px] flex flex-col relative select-none bg-[var(--bg-surface)] border-t border-[var(--border-base)] shrink-0 z-20">
  
  <!-- Header (28px) -->
  <div class="flex justify-between items-center px-3 h-[28px] shrink-0 border-b border-[var(--border-base)] bg-[var(--bg-elevated)]">
    <div class="flex items-center gap-2">
      <span class="text-[9px] font-bold text-[var(--text-muted)] uppercase tracking-[0.10em]">Timeline</span>
      <span class="text-[9px] text-[var(--text-muted)] opacity-60 hidden sm:inline">&middot; Drag brackets or drag track to select</span>
    </div>
    
    {#if $videoSrc}
      <div class="flex items-center gap-4">
        {#if thumbnailError}
          <span class="text-[9px] text-[var(--warning)] uppercase tracking-wide">⚠ Thumbnails N/A</span>
        {/if}
        <div class="flex items-center gap-1.5 font-mono text-[9px] text-[var(--text-muted)]">
          <span class="text-[var(--accent-bright)] font-semibold">{formatTimecode($trimStart)}</span>
          <span class="text-[var(--text-muted)] opacity-50">→</span>
          <span class="text-[var(--accent-bright)] font-semibold">{formatTimecode($trimEnd)}</span>
          <span class="px-1.5 py-0.5 rounded bg-[var(--bg-deep)] border border-[var(--border-subtle)] text-[var(--text-secondary)] ml-1">
            {formatTime($trimEnd - $trimStart)}
          </span>
        </div>
      </div>
    {/if}
  </div>

  {#if $videoSrc}
    <!-- Ruler (24px) -->
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="h-[24px] shrink-0 relative bg-[var(--bg-base)] border-b border-[var(--border-subtle)] overflow-hidden cursor-pointer"
      on:click={handleRulerClick}
    >
      {#each ticks as tick}
        <div
          class="absolute top-0 h-full flex flex-col items-center"
          style="left: {tick.time * pxPerSec}px"
        >
          <div
            class="w-px"
            class:h-[14px]={tick.major}
            class:bg-[var(--border-base)]={tick.major}
            class:h-[6px]={!tick.major}
            class:bg-[var(--border-subtle)]={!tick.major}
          ></div>
          {#if tick.major}
            <span class="absolute bottom-0 text-[8px] text-[var(--text-muted)] font-mono whitespace-nowrap translate-x-[2px] leading-[1]">
              {tick.label}
            </span>
          {/if}
        </div>
      {/each}

      <!-- Playhead marker on ruler -->
      <div
        class="absolute top-0 h-full w-[2px] bg-[var(--danger)] z-10 shadow-[0_0_6px_var(--danger-glow)]"
        style="left: {playheadX}px"
      ></div>
    </div>

    <!-- Track Area (~100px) -->
    <div class="flex-1 px-3 py-2 relative bg-[var(--bg-base)] overflow-hidden">
      <div
        class="relative w-full h-full overflow-x-auto overflow-y-hidden no-scrollbar cursor-crosshair active:cursor-crosshair"
        bind:clientWidth={viewportWidth}
      >
        <div class="relative h-full" style="width: {containerWidth}px; min-width: 100%;">
          
          <!-- Track Content (Thumbnails & Click/Drag target) -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div
            bind:this={trackEl}
            class="absolute inset-0 rounded-[6px] overflow-hidden border border-[var(--border-subtle)] bg-[var(--bg-deep)]"
            on:mousedown={handleTrackMousedown}
            on:dblclick={handleTrackDblClick}
          >
            <!-- Thumbnails Strip -->
            <div class="absolute inset-0 flex overflow-hidden opacity-65 pointer-events-none">
              {#if isGenerating && thumbnails.length === 0}
                <div class="w-full h-full bg-[var(--bg-elevated)] animate-pulse"></div>
              {:else if thumbnails.length > 0}
                {#each thumbnails as thumb}
                  <img
                    src={thumb.dataUrl}
                    alt=""
                    class="flex-1 min-w-0 h-full object-cover animate-in fade-in duration-300 border-r border-[var(--border-subtle)] last:border-r-0"
                    draggable="false"
                  />
                {/each}
                {#if isGenerating}
                  <div class="flex-1 bg-[var(--bg-elevated)] animate-pulse"></div>
                {/if}
              {:else}
                <div class="w-full h-full flex items-center justify-center bg-[var(--bg-deep)]">
                  <span class="text-[10px] text-[var(--text-muted)]">Preview unavailable — video can still be cut</span>
                </div>
              {/if}
            </div>

            <!-- Dark overlays for unselected regions -->
            <div
              class="absolute top-0 bottom-0 left-0 bg-[rgba(0,0,0,0.72)] pointer-events-none transition-all duration-75"
              style="width: {startX}px;"
            ></div>
            <div
              class="absolute top-0 bottom-0 right-0 bg-[rgba(0,0,0,0.72)] pointer-events-none transition-all duration-75"
              style="left: {endX}px;"
            ></div>

            <!-- Selection highlight borders (top & bottom) -->
            <div
              class="absolute top-0 h-[2px] bg-[var(--accent)] pointer-events-none shadow-[0_0_8px_var(--glow-accent)] z-10"
              style="transform: translate3d({startX}px, 0, 0); width: {Math.max(0, endX - startX)}px;"
            ></div>
            <div
              class="absolute bottom-0 h-[2px] bg-[var(--accent)] pointer-events-none shadow-[0_0_8px_var(--glow-accent)] z-10"
              style="transform: translate3d({startX}px, 0, 0); width: {Math.max(0, endX - startX)}px;"
            ></div>
          </div>

          <!-- Middle Range Body (Slide entire window) -->
          {#if endX - startX > HANDLE_WIDTH * 2}
            <div
              use:draggable={'range'}
              class="range-body absolute top-[2px] bottom-[2px] z-10 cursor-grab active:cursor-grabbing rounded-[2px]"
              style="transform: translate3d({startX + HANDLE_WIDTH}px, 0, 0); width: {Math.max(0, endX - startX - (HANDLE_WIDTH * 2))}px;"
            >
              <div class="range-body-tooltip">
                Drag to slide window ({formatTime($trimEnd - $trimStart)})
              </div>
            </div>
          {/if}

          <!-- Handles (Inward Pro Brackets) -->
          <!-- Trim Start Handle (IN Bracket) -->
          <div
            use:draggable={'start'}
            class="trim-bracket trim-bracket-in absolute top-0 bottom-0 z-20 flex items-center justify-center"
            style="transform: translate3d({startX}px, 0, 0); width: {HANDLE_WIDTH}px;"
          >
            <div class="bracket-grip">
              <div class="grip-line"></div>
              <div class="grip-line"></div>
            </div>
            <div class="trim-tooltip">IN {formatTimecode($trimStart)}</div>
          </div>

          <!-- Trim End Handle (OUT Bracket) -->
          <div
            use:draggable={'end'}
            class="trim-bracket trim-bracket-out absolute top-0 bottom-0 z-20 flex items-center justify-center"
            style="transform: translate3d({Math.max(startX + HANDLE_WIDTH, endX - HANDLE_WIDTH)}px, 0, 0); width: {HANDLE_WIDTH}px;"
          >
            <div class="bracket-grip">
              <div class="grip-line"></div>
              <div class="grip-line"></div>
            </div>
            <div class="trim-tooltip">OUT {formatTimecode($trimEnd)}</div>
          </div>

          <!-- Playhead Needle -->
          <div
            use:draggable={'playhead'}
            class="absolute -top-[5px] -bottom-[5px] w-[1.5px] bg-[var(--danger)] shadow-[0_0_8px_var(--danger-glow)] cursor-ew-resize z-30 group pointer-events-auto"
            style="transform: translate3d(calc({playheadX}px - 0.75px), 0, 0);"
          >
            <!-- Diamond heads -->
            <div class="absolute -top-[4px] left-1/2 -translate-x-1/2 w-[10px] h-[10px] bg-[var(--danger)] rotate-45 border border-[rgba(255,255,255,0.25)] group-hover:scale-[1.2] transition-transform cursor-ew-resize rounded-[2px] shadow-sm"></div>
            <div class="absolute -bottom-[4px] left-1/2 -translate-x-1/2 w-[7px] h-[7px] bg-[var(--danger)] rotate-45 border border-[rgba(255,255,255,0.25)] group-hover:scale-[1.2] transition-transform cursor-ew-resize rounded-[2px] shadow-sm"></div>
          </div>

        </div>
      </div>
    </div>
    
    <!-- Zoom Slider Widget -->
    <div class="zoom-widget">
      <MagnifyingGlassMinus size={11} class="text-[var(--text-secondary)] opacity-80" />
      <input type="range" bind:value={$zoomLevel} min="10" max="100" class="sleek-slider w-[80px]" />
      <MagnifyingGlassPlus size={11} class="text-[var(--text-secondary)] opacity-80" />
      <div class="w-px h-3 bg-[var(--border-base)] mx-1"></div>
      <span class="font-mono text-[9px] text-[var(--text-secondary)] w-5 text-right">{Math.round($zoomLevel)}</span>
    </div>

  {:else}
    <div class="flex-1 flex items-center justify-center border border-[var(--border-subtle)] border-dashed text-[var(--text-muted)] text-[12px] m-4 rounded-xl">
      Select a video to interact with the timeline
    </div>
  {/if}
</div>

<style>
  /* Inward Pro Bracket Handles */
  .trim-bracket {
    background: var(--accent);
    cursor: ew-resize;
    user-select: none;
    transition: background-color var(--dur-fast, 120ms), box-shadow var(--dur-fast, 120ms);
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.4);
  }

  .trim-bracket:hover,
  .trim-bracket:active {
    background: var(--accent-bright, #94a3b8);
    box-shadow: 0 0 12px var(--glow-accent);
  }

  .trim-bracket-in {
    border-top-left-radius: 4px;
    border-bottom-left-radius: 4px;
    border-right: 1px solid rgba(0, 0, 0, 0.25);
  }

  .trim-bracket-out {
    border-top-right-radius: 4px;
    border-bottom-right-radius: 4px;
    border-left: 1px solid rgba(0, 0, 0, 0.25);
  }

  /* Extended invisible hit area for effortless grabbing */
  .trim-bracket::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: -8px;
    right: -8px;
    cursor: ew-resize;
  }

  /* Bracket Grip Lines */
  .bracket-grip {
    display: flex;
    gap: 2px;
    align-items: center;
    justify-content: center;
    height: 14px;
    pointer-events: none;
    opacity: 0.85;
  }

  .grip-line {
    width: 1px;
    height: 12px;
    background-color: white;
    border-radius: 1px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  /* Draggable Range Body */
  .range-body {
    background: rgba(var(--accent-rgb), 0.08);
    transition: background-color 150ms;
  }

  :global(.dark) .range-body {
    background: rgba(var(--accent-rgb), 0.10);
  }

  .range-body:hover {
    background: rgba(var(--accent-rgb), 0.18);
  }

  :global(.dark) .range-body:hover {
    background: rgba(var(--accent-rgb), 0.20);
  }

  .range-body-tooltip {
    position: absolute;
    bottom: calc(100% + 4px);
    left: 50%;
    transform: translateX(-50%);
    font-size: 8.5px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: white;
    background: var(--bg-tooltip, rgba(18, 18, 18, 0.85));
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-subtle);
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 120ms ease;
    z-index: 30;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .range-body:hover .range-body-tooltip {
    opacity: 1;
  }

  .trim-tooltip {
    position: absolute;
    bottom: calc(100% + 4px);
    left: 50%;
    transform: translateX(-50%);
    font-size: 8px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: white;
    background: var(--bg-tooltip, rgba(18, 18, 18, 0.85));
    padding: 2px 5px;
    border-radius: 4px;
    border: 1px solid var(--border-subtle);
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 100ms ease;
    z-index: 30;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .trim-bracket:hover .trim-tooltip {
    opacity: 1;
  }

  .zoom-widget {
    position: absolute;
    bottom: 12px;
    right: 16px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-base);
    border-radius: var(--radius-full);
    padding: 4px 10px;
    box-shadow: var(--shadow-md);
    z-index: 30;
  }
</style>
