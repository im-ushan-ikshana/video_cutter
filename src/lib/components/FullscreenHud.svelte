<script lang="ts">
  import MiniTimeline from './MiniTimeline.svelte';
  import {
    isPlaying,
    currentTime,
    videoDuration,
    trimStart,
    trimEnd,
    selectedDuration,
    volume,
    muted,
    seekRequest,
    exportRequest,
    videoFilePath,
    isUserInteracting,
    isDarkMode
  } from '$lib/store';
  import { onDestroy } from 'svelte';
  import { formatTimecode, formatTime } from '$lib/utils';
  import {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    CaretLeft,
    CaretRight,
    SpeakerHigh,
    SpeakerSimpleX,
    ArrowCounterClockwise,
    UploadSimple,
    CornersIn,
    Spinner
  } from 'phosphor-svelte';

  export let player: any = null;
  export let onExitFullscreen: () => void;

  let showVolumeSlider = false;
  let isExporting = false;

  let hudVisibility: 'full' | 'dimmed' | 'hidden' = 'full';
  let isHoveringHud = false;
  let dimTimer: any = null;
  let hideTimer: any = null;
  let wasPlaying = false;

  function clearFadeTimers() {
    if (dimTimer) { clearTimeout(dimTimer); dimTimer = null; }
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
  }

  function updateCursor(hidden: boolean) {
    if (player && player.el()) {
      if (hidden) {
        player.el().classList.add('hud-cursor-hidden');
      } else {
        player.el().classList.remove('hud-cursor-hidden');
      }
    }
  }

  function wakeUp() {
    clearFadeTimers();
    hudVisibility = 'full';
    updateCursor(false);

    if ($isPlaying && !isHoveringHud && !$isUserInteracting) {
      dimTimer = setTimeout(() => {
        if ($isPlaying && !isHoveringHud && !$isUserInteracting) {
          hudVisibility = 'dimmed';
        }
      }, 600);

      hideTimer = setTimeout(() => {
        if ($isPlaying && !isHoveringHud && !$isUserInteracting) {
          hudVisibility = 'hidden';
          updateCursor(true);
        }
      }, 2000);
    }
  }

  $: if ($isPlaying && !wasPlaying) {
    wasPlaying = true;
    clearFadeTimers();
    if (!isHoveringHud && !$isUserInteracting) {
      // When playing in fullscreen: immediately 50% visibility, then completely invisible after 2s
      hudVisibility = 'dimmed';
      hideTimer = setTimeout(() => {
        if ($isPlaying && !isHoveringHud && !$isUserInteracting) {
          hudVisibility = 'hidden';
          updateCursor(true);
        }
      }, 2000);
    }
  } else if (!$isPlaying && wasPlaying) {
    wasPlaying = false;
    clearFadeTimers();
    hudVisibility = 'full';
    updateCursor(false);
  }

  function onHudMouseEnter() {
    isHoveringHud = true;
    clearFadeTimers();
    hudVisibility = 'full';
    updateCursor(false);
  }

  function onHudMouseLeave() {
    isHoveringHud = false;
    if ($isPlaying && !$isUserInteracting) {
      dimTimer = setTimeout(() => {
        if ($isPlaying && !isHoveringHud && !$isUserInteracting) {
          hudVisibility = 'dimmed';
        }
      }, 600);
      hideTimer = setTimeout(() => {
        if ($isPlaying && !isHoveringHud && !$isUserInteracting) {
          hudVisibility = 'hidden';
          updateCursor(true);
        }
      }, 2000);
    }
  }

  onDestroy(() => {
    clearFadeTimers();
    updateCursor(false);
  });

  function togglePlay() {
    if (!player) return;
    if (player.paused()) {
      const cur = player.currentTime() || 0;
      if ($trimEnd > $trimStart && (cur >= $trimEnd - 0.05 || cur < $trimStart)) {
        player.currentTime($trimStart);
        $currentTime = $trimStart;
      }
      player.play();
    } else {
      player.pause();
    }
  }

  function frameBack() {
    if (player) {
      const cur = player.currentTime() || 0;
      const target = Math.max($trimStart, cur - (1 / 30));
      player.currentTime(target);
      $currentTime = target;
    }
  }

  function frameForward() {
    if (player) {
      const cur = player.currentTime() || 0;
      const maxBoundary = $trimEnd > $trimStart ? $trimEnd : $videoDuration;
      const target = Math.min(maxBoundary, cur + (1 / 30));
      player.currentTime(target);
      $currentTime = target;
    }
  }

  function skipBack() {
    if (player) {
      const cur = player.currentTime() || 0;
      const target = Math.max($trimStart, cur - 5);
      player.currentTime(target);
      $currentTime = target;
    }
  }

  function skipForward() {
    if (player) {
      const cur = player.currentTime() || 0;
      const maxBoundary = $trimEnd > $trimStart ? $trimEnd : $videoDuration;
      const target = Math.min(maxBoundary, cur + 5);
      player.currentTime(target);
      $currentTime = target;
    }
  }

  function toggleMute() {
    if (player) {
      const newMuted = !player.muted();
      player.muted(newMuted);
      $muted = newMuted;
    }
  }

  function handleVolumeChange(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    if (player) {
      player.volume(val);
      $volume = val;
      if (val > 0 && player.muted()) {
        player.muted(false);
        $muted = false;
      }
    }
  }

  function setIn() {
    if ($currentTime < $trimEnd) $trimStart = $currentTime;
  }

  function setOut() {
    if ($currentTime > $trimStart) $trimEnd = $currentTime;
  }

  function goIn() {
    $currentTime = $trimStart;
    $seekRequest = $trimStart;
    if (player) player.currentTime($trimStart);
  }

  function goOut() {
    $currentTime = $trimEnd;
    $seekRequest = $trimEnd;
    if (player) player.currentTime($trimEnd);
  }

  function resetTrim() {
    $trimStart = 0;
    $trimEnd = $videoDuration;
  }

  function handleExport() {
    $exportRequest = Date.now();
  }
</script>

<svelte:window on:mousemove={wakeUp} on:keydown={wakeUp} />

<!-- Fullscreen Studio HUD Wrapper -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="fullscreen-hud-container fixed bottom-6 left-1/2 -translate-x-1/2 z-[9999] w-[calc(100%-48px)] max-w-[1100px] flex flex-col gap-2.5 transition-all duration-300 ease-out"
  class:opacity-100={hudVisibility === 'full'}
  class:opacity-50={hudVisibility === 'dimmed'}
  class:opacity-0={hudVisibility === 'hidden'}
  class:pointer-events-none={hudVisibility === 'hidden'}
  on:mouseenter={onHudMouseEnter}
  on:mouseleave={onHudMouseLeave}
>
  <!-- Main Glassmorphic Dock Container (Theme Aware) -->
  <div
    class="hud-glass-dock w-full flex flex-col gap-2 p-3.5 rounded-2xl transition-colors duration-200"
    class:light={!$isDarkMode}
    class:dark={$isDarkMode}
  >
    
    <!-- Top: Mini-Timeline (without thumbnails, inward handles, draggable range) -->
    <div class="w-full px-1">
      <MiniTimeline />
    </div>

    <!-- Bottom: Integrated Cutting Bar & Playback Controls -->
    <div class="flex items-center justify-between gap-4 pt-1">
      
      <!-- Left: Playback & Audio Controls -->
      <div class="flex items-center gap-1">
        <button class="hud-btn tooltip-host" on:click={frameBack} data-tooltip="Previous Frame">
          <CaretLeft size={16} />
        </button>

        <button class="hud-btn tooltip-host" on:click={skipBack} data-tooltip="Skip Back 5s">
          <SkipBack size={15} />
        </button>

        <!-- Hero Play/Pause Button -->
        <button class="hud-play-btn tooltip-host" on:click={togglePlay} data-tooltip={$isPlaying ? 'Pause (Space)' : 'Play (Space)'}>
          {#if $isPlaying}
            <Pause size={17} weight="fill" />
          {:else}
            <Play size={17} weight="fill" class="ml-0.5" />
          {/if}
        </button>

        <button class="hud-btn tooltip-host" on:click={skipForward} data-tooltip="Skip Forward 5s">
          <SkipForward size={15} />
        </button>

        <button class="hud-btn tooltip-host" on:click={frameForward} data-tooltip="Next Frame">
          <CaretRight size={16} />
        </button>

        <div class="hud-divider h-4 w-px mx-1"></div>

        <!-- Volume Controller -->
        <div
          class="relative flex items-center"
          on:mouseenter={() => showVolumeSlider = true}
          on:mouseleave={() => showVolumeSlider = false}
        >
          <button class="hud-btn tooltip-host" on:click={toggleMute} data-tooltip={$muted ? 'Unmute' : 'Mute'}>
            {#if $muted || $volume === 0}
              <SpeakerSimpleX size={16} class="text-[var(--danger)]" />
            {:else}
              <SpeakerHigh size={16} />
            {/if}
          </button>

          {#if showVolumeSlider}
            <div class="hud-volume-popover absolute bottom-full left-1/2 -translate-x-1/2 mb-2 p-2 rounded-lg shadow-xl flex items-center animate-in fade-in zoom-in-95 duration-150">
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={$muted ? 0 : $volume}
                on:input={handleVolumeChange}
                class="sleek-slider w-[75px]"
              />
            </div>
          {/if}
        </div>
      </div>

      <!-- Center: Full Cutting Bar (Set In/Out, Range Markers, Chips) -->
      <div class="flex items-center gap-2">
        <div class="hud-action-group flex items-center gap-1 rounded-lg p-1">
          <button class="hud-action-btn tooltip-host" on:click={setIn} data-tooltip="Set In Point (I)">
            <span class="font-bold mono text-[13px] leading-none">[</span>
          </button>

          <button class="hud-action-btn tooltip-host" on:click={setOut} data-tooltip="Set Out Point (O)">
            <span class="font-bold mono text-[13px] leading-none">]</span>
          </button>

          <div class="hud-divider h-3 w-px mx-0.5"></div>

          <button class="hud-action-btn tooltip-host" on:click={goIn} data-tooltip="Jump to In">
            <span class="text-[10px] font-mono font-semibold">|&lt;</span>
          </button>

          <button class="hud-action-btn tooltip-host" on:click={goOut} data-tooltip="Jump to Out">
            <span class="text-[10px] font-mono font-semibold">&gt;|</span>
          </button>

          <button class="hud-action-btn group tooltip-host" on:click={resetTrim} data-tooltip="Reset Range">
            <ArrowCounterClockwise size={13} class="group-hover:text-[var(--danger)] transition-colors" />
          </button>
        </div>

        <!-- Cutting Chips: IN, OUT, CLIP DURATION -->
        <div class="flex items-center gap-1.5 font-mono text-[11px]">
          <div class="hud-chip px-2 py-1 rounded flex items-center gap-1">
            <span class="hud-chip-label text-[9px] uppercase font-bold">IN</span>
            <span>{formatTimecode($trimStart)}</span>
          </div>

          <div class="hud-chip px-2 py-1 rounded flex items-center gap-1">
            <span class="hud-chip-label text-[9px] uppercase font-bold">OUT</span>
            <span>{formatTimecode($trimEnd)}</span>
          </div>

          <div class="hud-cut-chip px-2 py-1 rounded font-semibold flex items-center gap-1">
            <span class="text-[9px] uppercase font-bold text-[var(--accent)]">CUT</span>
            <span>{formatTime($selectedDuration)}</span>
          </div>
        </div>
      </div>

      <!-- Right: Playback Time, Direct Export, Fullscreen Exit -->
      <div class="flex items-center gap-2.5">
        <!-- Live Playhead Timecode -->
        <div class="hud-timecode font-mono text-[11.5px] whitespace-nowrap">
          <span class="hud-time-cur font-medium">{formatTime($currentTime)}</span>
          <span class="hud-time-divider mx-0.5">/</span>
          <span class="hud-time-total">{formatTime($videoDuration)}</span>
        </div>

        <!-- Direct Export Button -->
        <button
          class="hud-export-btn tooltip-host"
          on:click={handleExport}
          disabled={!$videoFilePath || isExporting}
          data-tooltip="Export Cut Clip (Ctrl+E)"
        >
          {#if isExporting}
            <Spinner size={14} class="animate-spin" />
            <span>Exporting...</span>
          {:else}
            <UploadSimple size={14} weight="bold" />
            <span>Export</span>
          {/if}
        </button>

        <div class="hud-divider h-4 w-px"></div>

        <!-- Exit Fullscreen Button -->
        <button class="hud-btn tooltip-host" on:click={onExitFullscreen} data-tooltip="Exit Fullscreen (F / Esc)">
          <CornersIn size={16} />
        </button>
      </div>

    </div>

  </div>
</div>

<style>
  /* Base / Dark Dock Styles */
  .hud-glass-dock {
    background: rgba(15, 23, 42, 0.84);
    backdrop-filter: blur(24px) saturate(180%);
    -webkit-backdrop-filter: blur(24px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.15);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7), inset 0 1px 0 rgba(255, 255, 255, 0.12);
  }

  .hud-divider {
    background: rgba(255, 255, 255, 0.15);
  }

  .hud-btn {
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    background: transparent;
    color: rgba(255, 255, 255, 0.75);
    border: 1px solid transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease-out);
    flex-shrink: 0;
  }

  .hud-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.15);
  }

  .hud-btn:active {
    transform: scale(0.96);
  }

  .hud-volume-popover {
    background: rgba(15, 23, 42, 0.90);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }

  .hud-action-group {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.10);
  }

  .hud-action-btn {
    width: 26px;
    height: 26px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.10);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .hud-action-btn:hover {
    background: rgba(255, 255, 255, 0.16);
    color: #ffffff;
    border-color: rgba(255, 255, 255, 0.25);
  }

  .hud-action-btn:active {
    transform: scale(0.95);
  }

  .hud-chip {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.10);
    color: rgba(255, 255, 255, 0.85);
  }

  .hud-chip-label {
    color: rgba(255, 255, 255, 0.40);
  }

  .hud-cut-chip {
    background: rgba(var(--accent-rgb), 0.18);
    border: 1px solid var(--accent);
    color: var(--accent-bright);
  }

  .hud-timecode {
    color: rgba(255, 255, 255, 0.80);
  }

  .hud-time-cur {
    color: #ffffff;
  }

  .hud-time-divider {
    color: rgba(255, 255, 255, 0.40);
  }

  .hud-time-total {
    color: rgba(255, 255, 255, 0.60);
  }

  /* Light Mode Overrides */
  .hud-glass-dock.light {
    background: rgba(255, 255, 255, 0.94);
    border: 1px solid rgba(148, 163, 184, 0.38);
    box-shadow: 0 20px 50px -10px rgba(15, 23, 42, 0.16), 0 4px 12px rgba(15, 23, 42, 0.06), inset 0 1px 0 rgba(255, 255, 255, 0.95);
  }

  .hud-glass-dock.light .hud-divider {
    background: rgba(148, 163, 184, 0.32);
  }

  .hud-glass-dock.light .hud-btn {
    color: var(--text-secondary);
  }

  .hud-glass-dock.light .hud-btn:hover {
    background: rgba(15, 23, 42, 0.06);
    color: var(--text-primary);
    border-color: rgba(148, 163, 184, 0.35);
  }

  .hud-glass-dock.light .hud-volume-popover {
    background: rgba(255, 255, 255, 0.96);
    border: 1px solid rgba(148, 163, 184, 0.35);
    color: var(--text-primary);
    box-shadow: var(--shadow-lg);
  }

  .hud-glass-dock.light .hud-action-group {
    background: rgba(15, 23, 42, 0.04);
    border: 1px solid rgba(148, 163, 184, 0.30);
  }

  .hud-glass-dock.light .hud-action-btn {
    background: #FFFFFF;
    color: var(--text-primary);
    border: 1px solid rgba(148, 163, 184, 0.35);
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
  }

  .hud-glass-dock.light .hud-action-btn:hover {
    background: rgba(37, 99, 235, 0.08);
    color: var(--accent);
    border-color: var(--accent);
  }

  .hud-glass-dock.light .hud-chip {
    background: #FFFFFF;
    border: 1px solid rgba(148, 163, 184, 0.35);
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
  }

  .hud-glass-dock.light .hud-chip-label {
    color: var(--text-muted);
  }

  .hud-glass-dock.light .hud-cut-chip {
    background: rgba(37, 99, 235, 0.08);
    border: 1px solid rgba(37, 99, 235, 0.35);
    color: #1D4ED8;
  }

  .hud-glass-dock.light .hud-timecode {
    color: var(--text-secondary);
  }

  .hud-glass-dock.light .hud-time-cur {
    color: var(--text-primary);
  }

  .hud-glass-dock.light .hud-time-divider {
    color: rgba(148, 163, 184, 0.6);
  }

  .hud-glass-dock.light .hud-time-total {
    color: var(--text-muted);
  }

  /* Shared Interactive Buttons */
  .hud-play-btn {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    background: var(--accent-gradient);
    color: #ffffff;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 10px rgba(var(--accent-rgb), 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease-out);
    flex-shrink: 0;
    margin: 0 2px;
  }

  .hud-play-btn:hover {
    filter: brightness(1.08);
    transform: translateY(-1px);
    box-shadow: 0 4px 14px rgba(var(--accent-rgb), 0.45);
  }

  .hud-play-btn:active {
    transform: scale(0.96);
  }

  .hud-export-btn {
    height: 30px;
    padding: 0 12px;
    border-radius: var(--radius-md);
    background: var(--accent-gradient);
    color: #ffffff;
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: -0.01em;
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease-out);
  }

  .hud-export-btn:hover:not(:disabled) {
    filter: brightness(1.08);
    transform: translateY(-0.5px);
    box-shadow: 0 4px 12px rgba(var(--accent-rgb), 0.45);
  }

  .hud-export-btn:active:not(:disabled) {
    transform: scale(0.97);
  }

  .hud-export-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  :global(.video-js.vjs-fullscreen.hud-cursor-hidden),
  :global(.video-js.vjs-fullscreen.hud-cursor-hidden *) {
    cursor: none !important;
  }
</style>
