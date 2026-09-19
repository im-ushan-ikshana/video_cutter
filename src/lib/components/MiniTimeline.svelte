<script lang="ts">
  import { trimStart, trimEnd, videoDuration, currentTime, seekRequest, autoSnap, isUserInteracting, isDarkMode } from '$lib/store';
  import { formatTimecode, formatTime } from '$lib/utils';

  let trackEl: HTMLDivElement;
  let trackWidth = 0;

  const HANDLE_WIDTH = 12;

  $: pxPerSec = $videoDuration > 0 && trackWidth > 0 ? trackWidth / $videoDuration : 0;
  $: startX = $videoDuration > 0 ? ($trimStart / $videoDuration) * trackWidth : 0;
  $: endX = $videoDuration > 0 ? ($trimEnd / $videoDuration) * trackWidth : 0;
  $: playheadX = $videoDuration > 0 ? ($currentTime / $videoDuration) * trackWidth : 0;

  let isHovering = false;
  let hoverX = 0;
  let hoverTime = 0;

  function handleTrackMousemove(e: MouseEvent) {
    if (!trackEl || !$videoDuration) return;
    const rect = trackEl.getBoundingClientRect();
    hoverX = Math.max(0, Math.min(rect.width, e.clientX - rect.left));
    hoverTime = (hoverX / rect.width) * $videoDuration;
  }

  function handleTrackClick(e: MouseEvent) {
    if (!trackEl || !$videoDuration || !pxPerSec) return;
    const target = e.target as HTMLElement;
    // Don't seek if clicking directly on a handle
    if (target.closest('.mini-bracket') || target.closest('.mini-range-body') || target.closest('.mini-playhead')) {
      return;
    }
    const rect = trackEl.getBoundingClientRect();
    const clickX = Math.max(0, Math.min(rect.width, e.clientX - rect.left));
    let time = (clickX / rect.width) * $videoDuration;

    if ($autoSnap) {
      const snapThreshold = 8 / pxPerSec;
      if (Math.abs(time - $trimStart) < snapThreshold) time = $trimStart;
      else if (Math.abs(time - $trimEnd) < snapThreshold) time = $trimEnd;
    }

    $currentTime = time;
    $seekRequest = time;
  }

  function draggable(node: HTMLElement, type: 'start' | 'end' | 'range' | 'playhead') {
    function handleMousedown(e: MouseEvent) {
      if (e.button !== 0 || !$videoDuration || !pxPerSec) return;
      e.stopPropagation();
      e.preventDefault();

      $isUserInteracting = true;
      document.body.style.userSelect = 'none';

      const startClientX = e.clientX;
      const initialTrimStart = $trimStart;
      const initialTrimEnd = $trimEnd;
      const initialCurrentTime = $currentTime;
      let lastSeekTime = 0;

      function onMousemove(event: MouseEvent) {
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
          let target = Math.max(0, Math.min($videoDuration, initialCurrentTime + dt));
          if ($autoSnap) {
            const snapThreshold = 8 / pxPerSec;
            if (Math.abs(target - $trimStart) < snapThreshold) target = $trimStart;
            else if (Math.abs(target - $trimEnd) < snapThreshold) target = $trimEnd;
          }
          $currentTime = target;
          const now = performance.now();
          if (now - lastSeekTime > 50) {
            $seekRequest = target;
            lastSeekTime = now;
          }
        }
      }

      function onMouseup() {
        document.body.style.userSelect = '';
        $isUserInteracting = false;
        if (type === 'playhead') {
          $seekRequest = $currentTime;
        }
        window.removeEventListener('mousemove', onMousemove);
        window.removeEventListener('mouseup', onMouseup);
      }

      window.addEventListener('mousemove', onMousemove);
      window.addEventListener('mouseup', onMouseup);
    }

    node.addEventListener('mousedown', handleMousedown);
    return {
      destroy() {
        node.removeEventListener('mousedown', handleMousedown);
      }
    };
  }
</script>

<!-- Mini Timeline Track Container -->
<div class="w-full flex flex-col gap-1 select-none">
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    bind:this={trackEl}
    bind:clientWidth={trackWidth}
    class="mini-track-wrapper h-[26px] relative w-full rounded-md overflow-visible cursor-pointer"
    class:light={!$isDarkMode}
    class:dark={$isDarkMode}
    on:mousemove={handleTrackMousemove}
    on:mouseenter={() => isHovering = true}
    on:mouseleave={() => isHovering = false}
    on:mousedown={handleTrackClick}
  >
    <!-- Background Track Bar -->
    <div class="mini-track-bg absolute inset-0 rounded-md backdrop-blur-md overflow-hidden transition-colors duration-200">
      <!-- Unselected Left Shade -->
      <div
        class="mini-unselected-shade absolute top-0 bottom-0 left-0 pointer-events-none transition-all duration-75"
        style="width: {startX}px;"
      ></div>

      <!-- Unselected Right Shade -->
      <div
        class="mini-unselected-shade absolute top-0 bottom-0 right-0 pointer-events-none transition-all duration-75"
        style="left: {endX}px;"
      ></div>

      <!-- Selected Region Glowing Rails -->
      <div
        class="absolute top-0 h-[2px] bg-[var(--accent)] pointer-events-none shadow-[0_0_8px_var(--glow-accent)]"
        style="transform: translate3d({startX}px, 0, 0); width: {Math.max(0, endX - startX)}px;"
      ></div>
      <div
        class="absolute bottom-0 h-[2px] bg-[var(--accent)] pointer-events-none shadow-[0_0_8px_var(--glow-accent)]"
        style="transform: translate3d({startX}px, 0, 0); width: {Math.max(0, endX - startX)}px;"
      ></div>
    </div>

    <!-- Draggable Middle Range Window (Slides the entire cut) -->
    {#if endX - startX > HANDLE_WIDTH * 2}
      <div
        use:draggable={'range'}
        class="mini-range-body absolute top-[2px] bottom-[2px] z-10 cursor-grab active:cursor-grabbing rounded-[2px]"
        style="transform: translate3d({startX + HANDLE_WIDTH}px, 0, 0); width: {Math.max(0, endX - startX - (HANDLE_WIDTH * 2))}px;"
      >
        <div class="mini-tooltip mini-tooltip-range">
          Slide Cut ({formatTime($trimEnd - $trimStart)})
        </div>
      </div>
    {/if}

    <!-- Inward Trim In Handle [ -->
    <div
      use:draggable={'start'}
      class="mini-bracket mini-bracket-in absolute top-0 bottom-0 z-20 flex items-center justify-center cursor-ew-resize"
      style="transform: translate3d({startX}px, 0, 0); width: {HANDLE_WIDTH}px;"
    >
      <div class="mini-grip">
        <div class="mini-grip-line"></div>
      </div>
      <div class="mini-tooltip">IN {formatTimecode($trimStart)}</div>
    </div>

    <!-- Inward Trim Out Handle ] -->
    <div
      use:draggable={'end'}
      class="mini-bracket mini-bracket-out absolute top-0 bottom-0 z-20 flex items-center justify-center cursor-ew-resize"
      style="transform: translate3d({Math.max(startX + HANDLE_WIDTH, endX - HANDLE_WIDTH)}px, 0, 0); width: {HANDLE_WIDTH}px;"
    >
      <div class="mini-grip">
        <div class="mini-grip-line"></div>
      </div>
      <div class="mini-tooltip">OUT {formatTimecode($trimEnd)}</div>
    </div>

    <!-- Draggable Playhead Needle -->
    <div
      use:draggable={'playhead'}
      class="mini-playhead absolute -top-[4px] -bottom-[4px] w-[2px] bg-[var(--danger)] shadow-[0_0_8px_var(--danger-glow)] cursor-ew-resize z-30 pointer-events-auto group"
      style="transform: translate3d(calc({playheadX}px - 1px), 0, 0);"
    >
      <!-- Diamond Indicators -->
      <div class="absolute -top-[3px] left-1/2 -translate-x-1/2 w-[8px] h-[8px] bg-[var(--danger)] rotate-45 rounded-[1px] shadow-sm border border-white/40 group-hover:scale-125 transition-transform"></div>
      <div class="absolute -bottom-[3px] left-1/2 -translate-x-1/2 w-[6px] h-[6px] bg-[var(--danger)] rotate-45 rounded-[1px] shadow-sm border border-white/40 group-hover:scale-125 transition-transform"></div>
    </div>

    <!-- Hover Indicator -->
    {#if isHovering}
      <div
        class="mini-hover-line absolute -top-[2px] -bottom-[2px] w-px pointer-events-none z-15"
        style="transform: translate3d({hoverX}px, 0, 0);"
      >
        <div class="mini-hover-pill absolute -top-[20px] left-1/2 -translate-x-1/2 px-1.5 py-0.5 rounded text-[9px] font-mono whitespace-nowrap shadow-md">
          {formatTime(hoverTime)}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .mini-track-wrapper {
    touch-action: none;
  }

  /* Dark Theme Default */
  .mini-track-bg {
    background: rgba(0, 0, 0, 0.65);
    border: 1px solid rgba(255, 255, 255, 0.12);
  }

  .mini-unselected-shade {
    background: rgba(0, 0, 0, 0.60);
  }

  .mini-range-body {
    background: rgba(var(--accent-rgb), 0.16);
    transition: background-color 120ms ease;
  }

  .mini-range-body:hover {
    background: rgba(var(--accent-rgb), 0.28);
  }

  .mini-hover-line {
    background: rgba(255, 255, 255, 0.40);
  }

  .mini-hover-pill {
    background: rgba(0, 0, 0, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }

  /* Light Theme Overrides */
  .mini-track-wrapper.light .mini-track-bg {
    background: #E2E8F0;
    border: 1px solid rgba(148, 163, 184, 0.45);
    box-shadow: inset 0 1px 2px rgba(15, 23, 42, 0.08);
  }

  .mini-track-wrapper.light .mini-unselected-shade {
    background: rgba(100, 116, 139, 0.28);
  }

  .mini-track-wrapper.light .mini-range-body {
    background: rgba(37, 99, 235, 0.12);
  }

  .mini-track-wrapper.light .mini-range-body:hover {
    background: rgba(37, 99, 235, 0.24);
  }

  .mini-track-wrapper.light .mini-hover-line {
    background: rgba(15, 23, 42, 0.45);
  }

  .mini-track-wrapper.light .mini-hover-pill {
    background: #0F172A;
    border: 1px solid rgba(148, 163, 184, 0.3);
    color: #F8FAFC;
  }

  /* Brackets & Grips */
  .mini-bracket {
    background: var(--accent);
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.4);
    transition: background-color var(--dur-fast), box-shadow var(--dur-fast);
  }

  .mini-bracket:hover,
  .mini-bracket:active {
    background: var(--accent-bright);
    box-shadow: 0 0 10px var(--glow-accent);
  }

  .mini-bracket-in {
    border-top-left-radius: 4px;
    border-bottom-left-radius: 4px;
    border-right: 1px solid rgba(0, 0, 0, 0.35);
  }

  .mini-bracket-out {
    border-top-right-radius: 4px;
    border-bottom-right-radius: 4px;
    border-left: 1px solid rgba(0, 0, 0, 0.35);
  }

  .mini-bracket::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: -6px;
    right: -6px;
    cursor: ew-resize;
  }

  .mini-grip {
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .mini-grip-line {
    width: 1.5px;
    height: 10px;
    background-color: white;
    border-radius: 1px;
    opacity: 0.95;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
  }

  .mini-tooltip {
    position: absolute;
    bottom: calc(100% + 5px);
    left: 50%;
    transform: translateX(-50%);
    font-size: 8.5px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--text-tooltip);
    background: var(--bg-tooltip);
    backdrop-filter: blur(8px);
    padding: 2px 5px;
    border-radius: 3px;
    border: 1px solid var(--border-subtle);
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 120ms ease;
    z-index: 40;
    box-shadow: var(--shadow-md);
  }

  .mini-bracket:hover .mini-tooltip,
  .mini-range-body:hover .mini-tooltip {
    opacity: 1;
  }
</style>
