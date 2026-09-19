<script lang="ts">
  import { trimStart, trimEnd, videoDuration, selectedDuration, currentTime, seekRequest, videoMetadata, videoFilePath, exportHistory, exportFormat, exportRequest } from '$lib/store';
  import type { ExportRecord } from '$lib/store';
  import { invoke } from '@tauri-apps/api/core';
  import { formatTime, parseTimecode, formatTimecode } from '$lib/utils';
  import { message } from '@tauri-apps/plugin-dialog';
  import { SkipBack, SkipForward, ArrowCounterClockwise, UploadSimple, Spinner } from 'phosphor-svelte';
  import { onDestroy } from 'svelte';

  let startInput = "00:00:00.000";
  let endInput = "00:00:00.000";
  let isExporting = false;

  const unsubExport = exportRequest.subscribe((val) => {
    if (val > 0) exportClip();
  });

  onDestroy(() => {
    unsubExport();
  });

  $: {
    startInput = formatTimecode($trimStart);
    endInput = formatTimecode($trimEnd);
  }

  function onStartChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const time = parseTimecode(target.value);
    if (time !== null && time <= $trimEnd && time >= 0) {
      $trimStart = time;
    }
    target.value = formatTimecode($trimStart);
  }

  function onEndChange(e: Event) {
    const target = e.target as HTMLInputElement;
    const time = parseTimecode(target.value);
    if (time !== null && time >= $trimStart && time <= $videoDuration) {
      $trimEnd = time;
    }
    target.value = formatTimecode($trimEnd);
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
  }
  function goOut() {
    $currentTime = $trimEnd;
    $seekRequest = $trimEnd;
  }
  function resetTrim() {
    $trimStart = 0;
    $trimEnd = $videoDuration;
  }

  function validateTrimRange(start: number, end: number, duration: number): string | null {
    if (start < 0) return "Start time cannot be negative.";
    if (end > duration + 0.5) return "End time exceeds the total video duration.";
    if (start >= end) return "Start time cannot be equal to or greater than the end time.";
    if (end - start < 0.1) return "The selected clip is too short. Minimum duration is 0.1 seconds.";
    return null;
  }

  async function exportClip() {
    if (!$videoFilePath || isExporting) return;

    const validationError = validateTrimRange($trimStart, $trimEnd, $videoDuration);
    if (validationError) {
      await message(validationError, { title: 'Invalid Trim Region', kind: 'warning' });
      return;
    }

    try {
      isExporting = true;
      const res = await invoke<string>('cut_video', {
        inputPath: $videoFilePath,
        startTime: $trimStart,
        endTime: $trimEnd,
        targetFormat: $exportFormat
      });
      
      const record: ExportRecord = {
        id: Date.now().toString(),
        sourcePath: $videoFilePath,
        outputPath: res,
        duration: $trimEnd - $trimStart,
        date: new Date().toISOString()
      };
      
      $exportHistory = [record, ...$exportHistory].slice(0, 50); // Keep last 50
      
      await message(`Clip exported successfully!\n\nSaved to: ${res}`, { title: 'Export Complete', kind: 'info' });
    } catch (e) {
      await message(`${e}`, { title: 'Export Failed', kind: 'error' });
    } finally {
      isExporting = false;
    }
  }
</script>

<div class="panel-bg w-full h-full flex flex-col shrink-0">
  
  <!-- Section: Cut Range -->
  <div class="pt-2">
    <div class="label px-[14px] pb-1.5">Cut Range</div>
    
    <div class="grid grid-cols-2 gap-2 px-[14px]">
      <div class="flex flex-col min-w-0">
        <span class="text-[9px] font-semibold tracking-[0.08em] text-[var(--text-muted)] uppercase mb-[4px] pl-1">Start</span>
        <input type="text" bind:value={startInput} on:change={onStartChange} class="input-tc w-full" />
      </div>
      <div class="flex flex-col min-w-0">
        <span class="text-[9px] font-semibold tracking-[0.08em] text-[var(--text-muted)] uppercase mb-[4px] pl-1">End</span>
        <input type="text" bind:value={endInput} on:change={onEndChange} class="input-tc w-full" />
      </div>
    </div>
    
    <div class="px-[14px] mt-2 w-full">
      <div class="timecode-chip w-full flex justify-between px-3">
        <span class="text-[9px] text-[var(--accent-bright)] tracking-wide uppercase font-semibold">Duration</span>
        <span>{formatTime($selectedDuration)}</span>
      </div>
    </div>
  </div>

  <hr class="divider" />

  <!-- Section: Actions -->
  <div>
    <div class="label px-[14px] pb-1.5">Actions</div>
    <div class="flex gap-1.5 px-[14px]">
      <!-- Quick Action Buttons -->
      <button class="quick-btn tooltip-host" on:click={setIn} data-tooltip="Set In (I)">
        <span class="font-bold mono text-[13px] leading-none">[</span>
      </button>
      <button class="quick-btn tooltip-host" on:click={setOut} data-tooltip="Set Out (O)">
        <span class="font-bold mono text-[13px] leading-none">]</span>
      </button>
      <button class="quick-btn tooltip-host" on:click={goIn} data-tooltip="Go to In">
        <SkipBack size={15} />
      </button>
      <button class="quick-btn tooltip-host" on:click={goOut} data-tooltip="Go to Out">
        <SkipForward size={15} />
      </button>
      <button class="quick-btn group tooltip-host" on:click={resetTrim} data-tooltip="Reset Trim">
        <ArrowCounterClockwise size={15} class="group-hover:text-[var(--danger)] transition-colors" />
      </button>
    </div>
  </div>

  <div class="flex-1"></div>

  <hr class="divider" />

  <!-- Section: Metadata -->
  <div>
    <div class="meta-bar">
      {#if $videoMetadata}
        <span>{$videoMetadata.width}×{$videoMetadata.height} &middot; {$videoMetadata.codec} &middot; {$videoMetadata.frame_rate} &middot; {$videoMetadata.bit_rate !== 'N/A' ? Math.round(parseInt($videoMetadata.bit_rate)/1000)+'k' : 'N/A'}</span>
      {:else}
        <div class="shimmer-placeholder w-full h-[8px] rounded-full mx-4"></div>
      {/if}
    </div>
  </div>

  <hr class="divider" />

  <!-- Export -->
  <div class="p-[14px] pt-0">
    <button class="btn-action w-full" on:click={exportClip} disabled={!$videoFilePath || isExporting}>
      {#if isExporting}
        <Spinner size={16} class="animate-spin" />
        <span>Exporting...</span>
      {:else}
        <UploadSimple size={16} />
        <span>Export Clip</span>
      {/if}
    </button>
  </div>

</div>

<style>
  .quick-btn {
    flex: 1;
    height: 30px;
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    border: 1px solid var(--border-base);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--dur-fast) var(--ease-out);
    box-shadow: var(--shadow-sm);
  }

  .quick-btn:hover {
    background: var(--hover-bg-strong);
    color: var(--text-primary);
    border-color: var(--border-strong);
    transform: translateY(-0.5px);
  }

  .quick-btn:active {
    transform: scale(0.97);
    transition-duration: var(--dur-micro);
  }

  .meta-bar {
    height: 24px;
    padding: 0 14px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .shimmer-placeholder {
    background: linear-gradient(90deg, var(--border-subtle) 25%, var(--border-base) 50%, var(--border-subtle) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite linear;
  }

  @keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
  }
</style>
