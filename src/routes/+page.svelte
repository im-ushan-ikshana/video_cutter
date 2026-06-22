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
  import { videoFilePath, videoSrc, isProxying, proxyProgress, proxyEta, videoDuration, trimStart, trimEnd, previewQuality, videoMetadata } from '$lib/store';

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
        $videoFilePath = paths[0];
        await loadMetadata();
        await updateProxy();
      }
    }));
  });

  onDestroy(() => {
    unlistens.forEach(fn => fn());
  });

  async function selectVideo() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Video Files',
          extensions: ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'ogg', 'vob', 'ts', 'm2ts', 'mts', 'rm', 'rmvb', 'asf', '3gp', 'm4v', 'mpg', 'mpeg', 'dav', 'h264', 'h265', 'hevc', 'av1', 'braw', 'r3d', 'mxf']
        },
        {
          name: 'All Files',
          extensions: ['*']
        }
      ]
    });
    if (selected) {
      $videoFilePath = selected as string;
      await loadMetadata();
      await updateProxy();
    }
  }

  async function loadMetadata() {
    try {
      const metadata: any = await invoke('get_video_metadata', { path: $videoFilePath });
      $videoMetadata = metadata;
      $videoDuration = metadata.duration;
      $trimStart = 0;
      $trimEnd = $videoDuration; // Select full video by default
    } catch (e) {
      console.error("Failed to load metadata", e);
    }
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
      $videoFilePath = path;
      await loadMetadata();
      await updateProxy();
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

<div class="flex flex-col w-full h-full p-5 gap-3">

  <!-- Toolbar -->
  <div class="flex items-center justify-between shrink-0 mb-1">
    <div class="flex items-center gap-3">
      <button class="modern-btn primary-variant" on:click={selectVideo}>
        <span>
          <svg class="mr-1" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
          Choose Video
        </span>
        <div class="button-overlay"></div>
      </button>

      <button class="modern-btn secondary-variant" on:click={pasteVideo}>
        <span>
          <svg class="mr-1" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>
          Paste Path
        </span>
        <div class="button-overlay"></div>
      </button>
    </div>
  </div>

  <!-- Main Layout -->
  <div class="flex flex-1 gap-3 min-h-0">
    <div class="w-[65%] h-full flex flex-col min-h-0">
      <VideoPreview />
    </div>
    <div class="w-[35%] h-full min-h-0">
      <ControlPanel />
    </div>
  </div>

  <!-- Timeline -->
  <div class="h-[130px] shrink-0 w-full">
    <Timeline />
  </div>
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
    <div class="relative glass-dialog w-[400px] p-6 animate-in fade-in zoom-in-95 duration-200 border-error/30">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-full bg-error/10 flex items-center justify-center shrink-0">
          <svg class="w-5 h-5 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <h2 class="text-[16px] font-semibold text-textPrimary">Error</h2>
      </div>
      
      <p class="text-[13px] text-textSecondary mb-6 leading-relaxed whitespace-pre-wrap">{appError}</p>
      
      <div class="flex justify-end">
        <button class="px-5 py-2.5 bg-error/20 hover:bg-error/30 text-error font-medium rounded-lg text-[13px] transition-colors" on:click={closeAppError}>
          Dismiss
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Global Drag & Drop Overlay -->
{#if isDraggingOver}
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 dark:bg-black/70 backdrop-blur-md pointer-events-none transition-all duration-200">
    <div class="bg-white dark:bg-[#1A1D21] p-12 rounded-[32px] shadow-[0_0_60px_rgba(76,141,255,0.3)] flex flex-col items-center border-[4px] border-dashed border-accent animate-in zoom-in-95 transform scale-100">
      <div class="w-24 h-24 bg-gradient-to-br from-[#4c8dff] to-[#0055ff] rounded-full flex items-center justify-center mb-6 animate-bounce shadow-[0_10px_30px_rgba(76,141,255,0.4)]">
        <svg class="w-12 h-12 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
      </div>
      <h2 class="text-[24px] font-extrabold text-textPrimary mb-2">Drop Video File Here</h2>
      <p class="text-[15px] text-textSecondary font-medium">Release to load into Universal Video Cutter</p>
    </div>
  </div>
{/if}

<style>
  /* From Uiverse.io by alaetr_2429 */ 
  .modern-btn {
    font-size: 13px;
    border-radius: 12px;
    background: linear-gradient(
      180deg,
      var(--btn-border-grad-1) 0%,
      var(--btn-border-grad-2) 66%,
      var(--btn-border-grad-3) 100%
    );
    color: var(--btn-text);
    border: none;
    padding: 2px;
    font-weight: 600;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    transition: all 0.3s ease;
    transform-origin: center;
    box-shadow: 0 4px 10px var(--btn-shadow);
  }

  .modern-btn span {
    border-radius: 10px;
    padding: 0.6em 1.2em;
    text-shadow: 0px 0px 20px rgba(0,0,0,0.3);
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: inherit;
    transition: all 0.3s ease;
    background-color: var(--btn-bg);
    background-image: radial-gradient(
        at 95% 89%,
        var(--btn-inner-glow) 0px,
        transparent 50%
      ),
      radial-gradient(at 0% 100%, var(--btn-inner-glow) 0px, transparent 50%),
      radial-gradient(at 0% 0%, var(--btn-bg) 0px, transparent 50%);
  }

  /* Primary variant uses solid full background color */
  .modern-btn.primary-variant span {
    background-color: var(--btn-primary-bg);
    color: var(--btn-primary-text);
    background-image: radial-gradient(
        at 95% 89%,
        var(--btn-primary-glow) 0px,
        transparent 50%
      ),
      radial-gradient(at 0% 100%, var(--btn-primary-glow) 0px, transparent 50%),
      radial-gradient(at 0% 0%, var(--btn-primary-bg) 0px, transparent 50%);
  }

  /* Secondary variant is purely background color based */
  .modern-btn.secondary-variant span {
    color: var(--text-primary);
  }

  .modern-btn:hover span {
    background-color: var(--btn-bg-hover);
  }

  .modern-btn.primary-variant:hover span {
    background-color: var(--btn-primary-bg-hover);
  }

  .button-overlay {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: repeating-conic-gradient(
        var(--btn-overlay) 0.0000001%,
        var(--btn-bg) 0.000104%
      )
      60% 60%/600% 600%;
    filter: opacity(10%) contrast(105%);
    -webkit-filter: opacity(10%) contrast(105%);
  }

  /* 🔥 Circular hover effect */
  .modern-btn::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    width: 0;
    height: 0;
    border-radius: 50%;
    background: radial-gradient(
      circle,
      var(--btn-ripple) 0%,
      rgba(255, 255, 255, 0) 70%
    );
    transform: translate(-50%, -50%) scale(0);
    transition:
      transform 0.6s ease,
      opacity 0.8s ease;
    opacity: 0;
    pointer-events: none;
  }

  .modern-btn.primary-variant::after {
    background: radial-gradient(
      circle,
      var(--btn-primary-glow) 0%,
      rgba(76, 141, 255, 0) 70%
    );
  }

  .modern-btn:hover::after {
    width: 200%;
    height: 200%;
    transform: translate(-50%, -50%) scale(1);
    opacity: 1;
  }

  /* 🌊 Click ripple effect */
  .modern-btn:active::before {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--btn-ripple);
    transform: translate(-50%, -50%) scale(0);
    animation: ripple-click 0.5s ease-out forwards;
    pointer-events: none;
  }

  .modern-btn:active {
    transform: scale(0.97);
    filter: brightness(1.1);
  }

  .modern-btn:hover {
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.08);
  }

  .modern-btn.primary-variant:hover {
    box-shadow: 0 0 15px rgba(76, 141, 255, 0.15);
  }

  @keyframes ripple-click {
    0% {
      transform: translate(-50%, -50%) scale(0);
      opacity: 1;
    }
    100% {
      transform: translate(-50%, -50%) scale(3);
      opacity: 0;
    }
  }
</style>
