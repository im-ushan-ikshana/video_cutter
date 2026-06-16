<script lang="ts">
  import { trimStart, trimEnd, selectedDuration, videoFilePath, videoDuration, videoMetadata, exportFormat } from '$lib/store';
  import { invoke } from '@tauri-apps/api/core';

  let isExporting = false;

  // Editable time inputs
  let startInput = '00:00:00.000';
  let endInput = '00:00:00.000';

  $: startInput = formatTime($trimStart);
  $: endInput = formatTime($trimEnd);

  function formatTime(seconds: number): string {
    if (!seconds || isNaN(seconds)) return '00:00:00.000';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 1000);
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(ms).padStart(3, '0')}`;
  }

  function parseTime(input: string): number | null {
    // Expected format: HH:MM:SS.MS or MM:SS.MS or SS.MS
    const parts = input.split(':');
    let seconds = 0;
    if (parts.length === 3) {
      seconds = parseInt(parts[0]) * 3600 + parseInt(parts[1]) * 60 + parseFloat(parts[2]);
    } else if (parts.length === 2) {
      seconds = parseInt(parts[0]) * 60 + parseFloat(parts[1]);
    } else if (parts.length === 1) {
      seconds = parseFloat(parts[0]);
    } else {
      return null;
    }
    return isNaN(seconds) ? null : seconds;
  }

  function onStartChange(event: Event) {
    const val = parseTime((event.target as HTMLInputElement).value);
    if (val !== null && val >= 0 && val < $trimEnd) $trimStart = val;
    else startInput = formatTime($trimStart);
  }

  function onEndChange(event: Event) {
    const val = parseTime((event.target as HTMLInputElement).value);
    if (val !== null && val > $trimStart && val <= $videoDuration) $trimEnd = val;
    else endInput = formatTime($trimEnd);
  }

  let dialogType: 'warning' | 'success' | 'error' | null = null;
  let dialogTitle = '';
  let dialogMessage = '';

  function showDialog(type: 'warning' | 'success' | 'error', title: string, message: string) {
    dialogType = type;
    dialogTitle = title;
    dialogMessage = message;
  }

  function closeDialog() {
    dialogType = null;
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
    
    const error = validateTrimRange($trimStart, $trimEnd, $videoDuration);
    if (error) {
      showDialog('warning', 'Invalid Trim Region', error);
      return;
    }

    isExporting = true;
    try {
      const outPath = await invoke('cut_video', {
        inputPath: $videoFilePath,
        startTime: $trimStart,
        endTime: $trimEnd,
        targetFormat: $exportFormat
      });
      showDialog('success', 'Export Successful!', `Your video clip was saved to:\n\n${outPath}`);
    } catch (e) {
      showDialog('error', 'Export Failed', `An error occurred during export:\n\n${e}`);
    } finally {
      isExporting = false;
    }
  }

  import { exportRequest } from '$lib/store';
  import { onDestroy } from 'svelte';
  
  const unsubExport = exportRequest.subscribe((val) => {
    if (val > 0) exportClip();
  });

  onDestroy(() => {
    unsubExport();
  });
</script>

<div class="w-full h-full flex flex-col gap-3 overflow-y-auto">

  <!-- Cut Settings -->
  <div class="modern-card shrink-0">
    <div class="modern-card-inner p-4 relative">
      <div class="noise-overlay"></div>
      <div class="relative z-10">
        <h3 class="section-title mb-3">Cut Settings</h3>

        <div class="grid grid-cols-2 gap-3 mb-3">
          <div class="flex flex-col gap-1">
            <span class="field-label">Start</span>
            <input type="text" bind:value={startInput} on:change={onStartChange} class="time-input text-center bg-black/5 dark:bg-black/20 border-black/10 dark:border-white/10 text-textPrimary placeholder-textMuted focus:border-accent" />
          </div>
          <div class="flex flex-col gap-1">
            <span class="field-label">End</span>
            <input type="text" bind:value={endInput} on:change={onEndChange} class="time-input text-center bg-black/5 dark:bg-black/20 border-black/10 dark:border-white/10 text-textPrimary placeholder-textMuted focus:border-accent" />
          </div>
        </div>

        <div class="flex justify-between items-center bg-black/5 dark:bg-black/20 rounded-lg px-3 py-2 border border-black/10 dark:border-white/5">
          <span class="text-[11px] text-textSecondary">Selected Duration</span>
          <span class="text-[13px] text-accent font-mono font-semibold">{formatTime($selectedDuration)}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Video Info & Export -->
  <div class="modern-card flex-1 min-h-0">
    <div class="modern-card-inner p-4 relative flex flex-col h-full">
      <div class="noise-overlay"></div>
      <div class="relative z-10 flex flex-col h-full">
        <h3 class="section-title mb-3">Video Info</h3>

        <div class="flex flex-col gap-2 mb-4">
          <div class="flex justify-between items-center text-[12px] h-[18px]">
            <span class="text-textSecondary">Format</span>
            <span class="text-textPrimary font-medium">{$exportFormat === 'ORIGINAL' ? 'Original Copy' : $exportFormat.replace('_', ' ')} (Target)</span>
          </div>
          <div class="flex justify-between items-center text-[12px] h-[18px]">
            <span class="text-textSecondary">Resolution</span>
            {#if $videoMetadata}
              <span class="text-textPrimary font-mono">{$videoMetadata.width}x{$videoMetadata.height}</span>
            {:else}
              <div class="shimmer-line w-[80px]"></div>
            {/if}
          </div>
          <div class="flex justify-between items-center text-[12px] h-[18px]">
            <span class="text-textSecondary">Codec</span>
            {#if $videoMetadata}
              <span class="text-textPrimary font-mono">{$videoMetadata.codec}</span>
            {:else}
              <div class="shimmer-line w-[50px]"></div>
            {/if}
          </div>
          <div class="flex justify-between items-center text-[12px] h-[18px]">
            <span class="text-textSecondary">Framerate</span>
            {#if $videoMetadata}
              <span class="text-textPrimary font-mono">{$videoMetadata.frame_rate}</span>
            {:else}
              <div class="shimmer-line w-[60px]"></div>
            {/if}
          </div>
          <div class="flex justify-between items-center text-[12px] h-[18px]">
            <span class="text-textSecondary">Bitrate</span>
            {#if $videoMetadata}
              <span class="text-textPrimary font-mono">{$videoMetadata.bit_rate !== 'N/A' ? `${Math.round(parseInt($videoMetadata.bit_rate) / 1000)} kbps` : 'N/A'}</span>
            {:else}
              <div class="shimmer-line w-[70px]"></div>
            {/if}
          </div>
          <div class="flex justify-between items-center text-[12px] h-[18px]">
            <span class="text-textSecondary">Start Timestamp</span>
            {#if $videoMetadata}
              <span class="text-textPrimary font-mono">{formatTime(parseFloat($videoMetadata.start_time))}</span>
            {:else}
              <div class="shimmer-line w-[60px]"></div>
            {/if}
          </div>
        </div>

        <div class="mt-auto pt-2">
          <button
            class="modern-btn primary-variant w-full {isExporting ? 'exporting' : ''}"
            on:click={exportClip}
            disabled={!$videoFilePath || isExporting}
          >
            <span class="flex justify-center transition-all duration-300">
              {#if isExporting}
                <svg class="animate-spin h-5 w-5 text-current" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
              {:else}
                <svg class="mr-2" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                Export Clip
              {/if}
            </span>
            <div class="button-overlay"></div>
          </button>
        </div>
      </div>
    </div>
  </div>

</div>

<!-- Generic Modal (floating overlay) -->
{#if dialogType}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-[100] flex items-center justify-center" on:click|self={closeDialog}>
    <div class="absolute inset-0 bg-black/60 backdrop-blur-md pointer-events-none transition-opacity"></div>
    <div class="relative bg-surface border rounded-xl p-6 w-[360px] shadow-2xl animate-in fade-in zoom-in-95 flex flex-col items-center text-center {dialogType === 'success' ? 'border-green-500/30' : 'border-red-500/30'}">
      
      <div class="w-12 h-12 rounded-full flex items-center justify-center mb-4 {dialogType === 'success' ? 'bg-green-500/20 text-green-500' : 'bg-red-500/20 text-red-500'}">
        {#if dialogType === 'warning'}
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
        {:else if dialogType === 'error'}
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>
        {:else if dialogType === 'success'}
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
        {/if}
      </div>

      <h2 class="text-[16px] font-semibold text-textPrimary mb-2">{dialogTitle}</h2>
      <p class="text-[13px] text-textSecondary mb-6 leading-relaxed whitespace-pre-wrap break-all">{dialogMessage}</p>
      
      <button 
        class="w-full h-10 rounded-lg text-[13px] font-semibold text-white transition-all active:scale-[0.98] {dialogType === 'success' ? 'bg-green-500 hover:bg-green-600 shadow-[0_4px_14px_rgba(34,197,94,0.3)]' : 'bg-red-500 hover:bg-red-600 shadow-[0_4px_14px_rgba(239,68,68,0.3)]'}" 
        on:click={closeDialog}
      >
        {dialogType === 'success' ? 'Awesome' : 'Acknowledge'}
      </button>
    </div>
  </div>
{/if}

<style>
  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .field-label {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
  }
  .time-input {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 6px 10px;
    font-size: 13px;
    color: var(--text-primary);
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    outline: none;
    transition: border-color 120ms, box-shadow 120ms;
  }
  .time-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(76,141,255,0.12);
  }

  .modern-btn {
    font-size: 14px;
    border-radius: 12px;
    background: linear-gradient(180deg, var(--btn-border-grad-1) 0%, var(--btn-border-grad-2) 66%, var(--btn-border-grad-3) 100%);
    color: var(--btn-text);
    border: none;
    padding: 2px;
    font-weight: 700;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    transition: all 0.3s ease;
    transform-origin: center;
    box-shadow: 0 4px 10px var(--btn-shadow);
  }
  .modern-btn span {
    border-radius: 10px;
    padding: 0.8em 1.2em;
    text-shadow: 0px 0px 20px rgba(0,0,0,0.3);
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: inherit;
    transition: all 0.3s ease;
    background-color: var(--btn-bg);
    background-image: radial-gradient(at 95% 89%, var(--btn-inner-glow) 0px, transparent 50%), radial-gradient(at 0% 100%, var(--btn-inner-glow) 0px, transparent 50%), radial-gradient(at 0% 0%, var(--btn-bg) 0px, transparent 50%);
  }
  .modern-btn.primary-variant span {
    background-color: var(--btn-primary-bg);
    color: var(--btn-primary-text);
    background-image: radial-gradient(at 95% 89%, var(--btn-primary-glow) 0px, transparent 50%), radial-gradient(at 0% 100%, var(--btn-primary-glow) 0px, transparent 50%), radial-gradient(at 0% 0%, var(--btn-primary-bg) 0px, transparent 50%);
  }
  .modern-btn:hover span { background-color: var(--btn-bg-hover); }
  .modern-btn.primary-variant:hover span { background-color: var(--btn-primary-bg-hover); }
  .modern-btn.exporting span { background-color: var(--btn-primary-bg); opacity: 0.8; }
  .modern-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .button-overlay {
    position: absolute; inset: 0; pointer-events: none;
    background: repeating-conic-gradient(var(--btn-overlay) 0.0000001%, var(--btn-bg) 0.000104%) 60% 60%/600% 600%;
    filter: opacity(10%) contrast(105%); -webkit-filter: opacity(10%) contrast(105%);
  }
  .modern-btn::after {
    content: ""; position: absolute; top: 50%; left: 50%; width: 0; height: 0; border-radius: 50%;
    background: radial-gradient(circle, var(--btn-primary-glow) 0%, rgba(76, 141, 255, 0) 70%);
    transform: translate(-50%, -50%) scale(0); transition: transform 0.6s ease, opacity 0.8s ease; opacity: 0; pointer-events: none;
  }
  .modern-btn:hover::after { width: 200%; height: 200%; transform: translate(-50%, -50%) scale(1); opacity: 1; }
  .modern-btn:active::before {
    content: ""; position: absolute; top: 50%; left: 50%; width: 40px; height: 40px; border-radius: 50%;
    background: var(--btn-ripple); transform: translate(-50%, -50%) scale(0);
    animation: ripple-click 0.5s ease-out forwards; pointer-events: none;
  }
  .modern-btn:active { transform: scale(0.97); filter: brightness(1.1); }
  .modern-btn.primary-variant:hover { box-shadow: 0 0 15px rgba(76, 141, 255, 0.2); }
  @keyframes ripple-click { 0% { transform: translate(-50%, -50%) scale(0); opacity: 1; } 100% { transform: translate(-50%, -50%) scale(3); opacity: 0; } }

  /* Smart targeted reveal shimmer based on Uiverse.io by Nawsome */ 
  .shimmer-line {
    position: relative;
    height: 10px;
    background-color: var(--border);
    border-radius: 4px;
    overflow: hidden;
  }
  .shimmer-line:after {
    content: "";
    position: absolute;
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    background: linear-gradient(110deg, transparent 0%, transparent 40%, rgba(255, 255, 255, 0.1) 50%, transparent 60%, transparent 100%);
    animation: gradient-animation_2 1.2s linear infinite;
  }

  /* Adjust gradient brightness for light theme */
  :global(:root) .shimmer-line:after {
    background: linear-gradient(110deg, transparent 0%, transparent 40%, rgba(255, 255, 255, 0.6) 50%, transparent 60%, transparent 100%);
  }
  :global(:root.dark) .shimmer-line:after {
    background: linear-gradient(110deg, transparent 0%, transparent 40%, rgba(255, 255, 255, 0.05) 50%, transparent 60%, transparent 100%);
  }

  @keyframes gradient-animation_2 {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
</style>
