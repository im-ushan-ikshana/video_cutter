<script lang="ts">
  import VideoPreview from '$lib/components/VideoPreview.svelte';
  import ControlPanel from '$lib/components/ControlPanel.svelte';
  import Timeline from '$lib/components/Timeline.svelte';
  import SettingsDialog from '$lib/components/SettingsDialog.svelte';

  import { open } from '@tauri-apps/plugin-dialog';
  import { readText } from '@tauri-apps/plugin-clipboard-manager';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { videoFilePath, videoSrc, isProxying, proxyProgress, proxyEta, videoDuration, trimStart, trimEnd, previewQuality, videoMetadata, exportFormat, exportQuality, autoSnap, projectLoadRequest, openVideoRequest, pasteVideoRequest, enableProxy } from '$lib/store';
  import { type ProjectData, loadProjectFromFile } from '$lib/project';

  let showPasteDialog = false;
  let pasteInput = "";
  let proxyStartTime = 0;
  let isDraggingOver = false;
  let unlistens: (() => void)[] = [];

  onMount(async () => {
    unlistens.push(await listen('proxy_progress', (event) => {
      const secondsProcessed = event.payload as number;
      if ($videoDuration > 0) {
        $proxyProgress = Math.min((secondsProcessed / $videoDuration) * 100, 100);
        
        // Calculate ETA
        const elapsed = (performance.now() - proxyStartTime) / 1000;
        const rate = secondsProcessed / elapsed;
        if (rate > 0) {
          const remainingSeconds = Math.max(0, ($videoDuration - secondsProcessed) / rate);
          const m = Math.floor(remainingSeconds / 60);
          const s = Math.floor(remainingSeconds % 60);
          $proxyEta = `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
        }
      }
    }));

    unlistens.push(await listen('tauri://drag-enter', () => {
      isDraggingOver = true;
    }));

    unlistens.push(await listen('tauri://drag-leave', () => {
      isDraggingOver = false;
    }));

    unlistens.push(await listen('tauri://drag-drop', async (event: any) => {
      isDraggingOver = false;
      const paths = event.payload?.paths;
      if (paths && paths.length > 0) {
        await openFile(paths[0]);
      }
    }));

    const unsubProject = projectLoadRequest.subscribe(async (data: ProjectData | null) => {
      if (data) {
        if ($isProxying) {
            await invoke('cancel_proxy');
        }
        
        $videoFilePath = data.videoFilePath;
        $exportFormat = data.exportFormat;
        $exportQuality = data.exportQuality;
        $autoSnap = data.autoSnap;
        if (data.previewQuality) $previewQuality = data.previewQuality;

        // Try to load metadata
        const success = await loadMetadata();
        if (success) {
            // Restore trim points AFTER metadata load, otherwise they might get reset
            $trimStart = data.trimStart;
            $trimEnd = data.trimEnd;
            await setupVideoPlayback();
        }
        
        // Reset request
        $projectLoadRequest = null;
      }
    });
    unlistens.push(unsubProject);

    const unsubOpen = openVideoRequest.subscribe((val) => {
      if (val > 0) selectVideo();
    });
    unlistens.push(unsubOpen);

    const unsubPaste = pasteVideoRequest.subscribe((val) => {
      if (val > 0) pasteVideo();
    });
    unlistens.push(unsubPaste);
  });

  onDestroy(() => {
    unlistens.forEach(fn => fn());
  });

  async function openFile(filePath: string) {
    if (!filePath) return;
    const lower = filePath.toLowerCase();
    if (lower.endsWith('.json') || lower.endsWith('.cutterproj')) {
      try {
        const projectData = await loadProjectFromFile(filePath);
        if (projectData) {
          $projectLoadRequest = projectData;
          return;
        }
      } catch (e) {
        console.warn("File was not a valid project, proceeding as video file:", e);
      }
    }

    if ($isProxying) {
      await invoke('cancel_proxy');
    }
    $videoFilePath = filePath;
    const success = await loadMetadata();
    if (success) {
      await setupVideoPlayback();
    }
  }

  async function selectVideo() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Supported Files (Videos & Projects)',
          extensions: ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'ogg', 'vob', 'ts', 'm2ts', 'mts', 'rm', 'rmvb', 'asf', '3gp', 'm4v', 'mpg', 'mpeg', 'dav', 'h264', 'h265', 'hevc', 'av1', 'braw', 'r3d', 'mxf', 'cutterproj', 'json']
        },
        {
          name: 'Video Files',
          extensions: ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'ogg', 'vob', 'ts', 'm2ts', 'mts', 'rm', 'rmvb', 'asf', '3gp', 'm4v', 'mpg', 'mpeg', 'dav', 'h264', 'h265', 'hevc', 'av1', 'braw', 'r3d', 'mxf']
        },
        {
          name: 'Cut Project Files',
          extensions: ['cutterproj', 'json']
        },
        {
          name: 'All Files',
          extensions: ['*']
        }
      ]
    });
    if (selected && typeof selected === 'string') {
      await openFile(selected);
    }
  }

  async function loadMetadata(): Promise<boolean> {
    try {
      const metadata: any = await invoke('get_video_metadata', { path: $videoFilePath });
      $videoMetadata = metadata;
      $videoDuration = metadata.duration;
      $trimStart = 0;
      $trimEnd = $videoDuration; // Select full video by default
      return true;
    } catch (e) {
      console.error("Failed to load metadata", e);
      appError = "Failed to load video metadata: " + e;
      return false;
    }
  }

  async function setupVideoPlayback() {
    if (!$videoFilePath) return;
    if ($enableProxy) {
      await updateProxy();
    } else {
      if ($isProxying) {
        await invoke('cancel_proxy');
        $isProxying = false;
      }
      $videoSrc = convertFileSrc($videoFilePath);
    }
  }

  let previousEnableProxy: boolean | null = null;
  $: if ($videoFilePath) {
    if (previousEnableProxy !== null && previousEnableProxy !== $enableProxy) {
      setupVideoPlayback();
    }
    previousEnableProxy = $enableProxy;
  }

  async function updateProxy() {
    if (!$videoFilePath) return;
    $isProxying = true;
    $proxyProgress = 0;
    $proxyEta = 'Calculating...';
    proxyStartTime = performance.now();
    try {
      const proxyPath: string = await invoke('generate_proxy', {
        inputPath: $videoFilePath,
        quality: $previewQuality
      });
      $videoSrc = convertFileSrc(proxyPath);
    } catch (e) {
      if (e === "CANCELLED") {
        console.log("Proxy generation cancelled. Playing original file.");
        $videoSrc = convertFileSrc($videoFilePath);
      } else {
        appError = "Failed to generate proxy: " + e;
      }
    } finally {
      $isProxying = false;
    }
  }
  async function pasteVideo() {
    try {
      pasteInput = await readText() || "";
    } catch (e) {
      pasteInput = "";
    }
    showPasteDialog = true;
  }

  async function confirmPaste() {
    let path = pasteInput.trim().replace(/^["']|["']$/g, '');
    showPasteDialog = false;
    if (path) {
      await openFile(path);
    }
  }

  function cancelPaste() {
    showPasteDialog = false;
  }
  let appError: string | null = null;

  function closeAppError() {
    appError = null;
  }
</script>

<div class="flex-1 flex flex-col p-5 gap-4 relative">
  {#if !$videoFilePath}
    <!-- Professional Studio Welcome / Initial Window -->
    <div class="flex-1 flex flex-col items-center justify-center p-6 sm:p-10 animate-in fade-in duration-300 zoom-in-95 max-w-4xl mx-auto w-full">
      
      <!-- Studio Header Section (Using Modern Antiqua for the app name) -->
      <div class="flex flex-col items-center text-center mb-7">
        <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--accent-dim)] border border-[var(--accent)]/30 text-[var(--accent-bright)] text-[10.5px] font-mono font-semibold tracking-wider uppercase mb-3.5 shadow-sm">
          <span>Lossless Video Engine</span>
          <span class="opacity-50">&middot;</span>
          <span>Fast Proxy Trimmer</span>
        </div>

        <h1 class="font-brand text-4xl sm:text-5xl lg:text-[52px] font-normal text-textPrimary tracking-normal mb-3 leading-tight">
          Universal Video Cutter
        </h1>

        <p class="text-[14px] text-textSecondary max-w-xl leading-relaxed font-sans font-normal">
          High-performance, frame-accurate video trimming with zero quality loss. Rapidly scrub massive footage, select in/out ranges, and export pristine clips instantly.
        </p>
      </div>

      <!-- Drag & Drop Studio Card -->
      <div 
        class="matte-glass-card rounded-[28px] p-8 sm:p-10 max-w-lg w-full flex flex-col items-center text-center border-dashed border-[2px] border-borderBase hover:border-accent transition-all duration-200 cursor-pointer group shadow-xl relative overflow-hidden" 
        on:click={selectVideo} 
        on:keydown={(e) => e.key === 'Enter' && selectVideo()} 
        role="button" 
        tabindex="0"
      >
        <!-- Subtle Glow backdrop -->
        <div class="absolute -top-24 left-1/2 -translate-x-1/2 w-48 h-48 rounded-full bg-accent/10 blur-3xl pointer-events-none group-hover:bg-accent/20 transition-all duration-500"></div>

        <div class="w-16 h-16 rounded-2xl flex items-center justify-center mb-5 group-hover:scale-110 transition-all duration-300 relative z-10">
          <img src="/player.png" alt="Universal Video Cutter Icon" class="w-16 h-16 rounded-2xl shadow-lg object-contain select-none pointer-events-none ring-1 ring-black/5 dark:ring-white/10" />
        </div>

        <h2 class="text-lg font-semibold text-textPrimary mb-1.5 tracking-tight relative z-10">
          Drop your video file here
        </h2>
        <p class="text-textSecondary mb-5 text-[12.5px] leading-relaxed max-w-sm relative z-10">
          Drag and drop any video from your files, or click to browse.
        </p>

        <div class="flex items-center gap-2.5 relative z-10">
          <button class="btn-primary px-6 h-9 text-[12.5px] rounded-xl shadow-lg shadow-accent/20">
            Select Video File
          </button>
          <span class="text-[11px] text-textMuted font-mono bg-bg border border-borderBase px-2.5 py-1.5 rounded-lg">
            Ctrl+O
          </span>
        </div>

        <!-- Supported Formats Badges -->
        <div class="flex flex-wrap items-center justify-center gap-1.5 mt-6 pt-5 border-t border-borderBase/60 w-full relative z-10">
          {#each ['MP4', 'MKV', 'MOV', 'WebM', 'AVI', 'ProRes', 'HEVC'] as fmt}
            <span class="px-2 py-0.5 rounded-md text-[9.5px] font-mono font-medium text-textMuted bg-bg/70 border border-borderBase">
              {fmt}
            </span>
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <!-- Grid Layout -->
    <div class="flex-1 flex flex-col lg:flex-row gap-4 min-h-0 min-w-0">
      <div class="flex-1 min-w-0 min-h-[300px] lg:min-h-0 matte-glass-card rounded-2xl overflow-hidden relative flex flex-col">
        <VideoPreview />
      </div>
      <div class="w-full lg:w-[280px] shrink-0 lg:h-full flex flex-col gap-4 min-h-0 min-w-0">
        <div class="flex-1 matte-glass-card rounded-2xl overflow-hidden min-h-[300px] lg:min-h-0">
          <ControlPanel />
        </div>
      </div>
    </div>
    
    <!-- Floating Timeline -->
    <div class="h-[160px] shrink min-h-[100px] w-full matte-glass-card rounded-2xl overflow-hidden p-2">
      <Timeline />
    </div>
  {/if}
</div>

<!-- Settings Dialog (floating overlay) -->
<SettingsDialog />

<!-- Paste Path Dialog (floating overlay) -->
{#if showPasteDialog}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-[60] flex items-center justify-center" on:click|self={cancelPaste}>
    <div class="absolute inset-0 bg-black/40 backdrop-blur-sm pointer-events-none transition-opacity"></div>
    <div class="relative glass-dialog w-[400px] p-6 animate-in fade-in zoom-in-95 duration-200">
      <h2 class="text-[16px] font-semibold text-textPrimary mb-2">Paste Video Path</h2>
      <p class="text-[12px] text-textSecondary mb-4">Enter the absolute file path to your video file below.</p>
      
      <!-- svelte-ignore a11y_autofocus -->
      <input 
        type="text" 
        bind:value={pasteInput} 
        class="w-full bg-bg border border-borderBase rounded-lg px-3 py-2.5 text-[13px] text-textPrimary focus:border-accent outline-none mb-5 font-mono shadow-inner transition-colors" 
        placeholder="C:\videos\my_video.mp4" 
        autofocus 
        on:keydown={(e) => e.key === 'Enter' && confirmPaste()}
      />
      
      <div class="flex justify-end gap-2">
        <button class="btn-secondary" on:click={cancelPaste}>Cancel</button>
        <button class="btn-primary px-5" on:click={confirmPaste}>Load Video</button>
      </div>
    </div>
  </div>
{/if}

<!-- App Error Dialog -->
{#if appError}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-[70] flex items-center justify-center" on:click|self={closeAppError}>
    <div class="absolute inset-0 bg-black/40 backdrop-blur-sm pointer-events-none transition-opacity"></div>
    <div class="relative glass-dialog w-[400px] p-6 animate-in fade-in zoom-in-95 duration-200 border-danger/30">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-full bg-danger/10 flex items-center justify-center shrink-0">
          <svg class="w-5 h-5 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <h2 class="text-[16px] font-semibold text-textPrimary">Error</h2>
      </div>
      
      <p class="text-[13px] text-textSecondary mb-6 leading-relaxed whitespace-pre-wrap">{appError}</p>
      
      <div class="flex justify-end">
        <button class="px-5 py-2.5 bg-danger/20 hover:bg-danger/30 text-danger font-medium rounded-lg text-[13px] transition-colors" on:click={closeAppError}>
          Dismiss
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Global Drag & Drop Overlay -->
{#if isDraggingOver}
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 dark:bg-black/70 backdrop-blur-md pointer-events-none transition-all duration-200">
    <div class="bg-white dark:bg-card p-12 rounded-[32px] shadow-[0_0_60px_rgba(var(--accent-rgb),0.25)] flex flex-col items-center border-[3px] border-dashed border-accent animate-in zoom-in-95 transform scale-100">
      <div class="w-24 h-24 rounded-full flex items-center justify-center mb-6 shadow-[0_10px_25px_rgba(var(--accent-rgb),0.35)]" style="background: var(--accent-gradient);">
        <svg class="w-12 h-12 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
      </div>
      <h2 class="text-[24px] font-extrabold text-textPrimary mb-2">Drop Video File Here</h2>
      <p class="text-[15px] text-textSecondary font-medium">Release to load into Universal Video Cutter</p>
    </div>
  </div>
{/if}


