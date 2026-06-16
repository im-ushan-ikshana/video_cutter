<script lang="ts">
  import { videoFilePath, videoSrc, isProxying, proxyProgress, proxyEta, videoMetadata, currentTime, isPlaying, volume, muted, seekRequest, exportRequest, trimStart, trimEnd, videoDuration } from '$lib/store';
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';

  import { invoke } from '@tauri-apps/api/core';

  let containerEl: HTMLDivElement;
  let player: any = null;
  let videojs: any = null;
  let mounted = false;

  let unsubSeek: () => void;
  
  let isFastForwarding = false;
  let isRewinding = false;
  let rewindInterval: any;
  let wasPlayingBeforeFF = false;

  function handleKeydown(e: KeyboardEvent) {
    if (e.target && ['INPUT', 'TEXTAREA', 'SELECT'].includes((e.target as HTMLElement).tagName)) return;
    if (!player) return;

    if (e.key === ' ') {
      e.preventDefault();
      if (player.paused()) player.play(); else player.pause();
    } else if (e.key.toLowerCase() === 'i') {
      $trimStart = Math.min($currentTime, $trimEnd - 0.1);
    } else if (e.key.toLowerCase() === 'o') {
      $trimEnd = Math.max($currentTime, $trimStart + 0.1);
    } else if (e.key === 'ArrowLeft' && e.shiftKey) {
      $seekRequest = Math.max(0, $currentTime - 5);
    } else if (e.key === 'ArrowRight' && !e.shiftKey) {
      if (e.repeat && !isFastForwarding) {
        isFastForwarding = true;
        wasPlayingBeforeFF = !player.paused();
        player.playbackRate(4.0);
        player.play();
      } else if (!e.repeat) {
        $seekRequest = Math.min($videoDuration, $currentTime + 0.1);
      }
    } else if (e.key === 'ArrowLeft' && !e.shiftKey) {
      if (e.repeat && !isRewinding) {
        isRewinding = true;
        wasPlayingBeforeFF = !player.paused();
        player.pause();
        rewindInterval = setInterval(() => {
          $seekRequest = Math.max(0, $currentTime - 0.2);
        }, 50);
      } else if (!e.repeat) {
        $seekRequest = Math.max(0, $currentTime - 0.1);
      }
    } else if (e.key.toLowerCase() === 'e' && e.ctrlKey) {
      e.preventDefault();
      exportRequest.set(Date.now());
    } else if (e.key.toLowerCase() === 's' && e.ctrlKey) {
      e.preventDefault();
    }
  }

  function handleKeyup(e: KeyboardEvent) {
    if (!player) return;
    if (e.key === 'ArrowRight') {
      if (isFastForwarding) {
        isFastForwarding = false;
        player.playbackRate(1.0);
        if (!wasPlayingBeforeFF) player.pause();
      }
    } else if (e.key === 'ArrowLeft') {
      if (isRewinding) {
        isRewinding = false;
        clearInterval(rewindInterval);
        if (wasPlayingBeforeFF) player.play();
      }
    }
  }

  onMount(async () => {
    const mod = await import('video.js');
    videojs = mod.default;
    await import('video.js/dist/video-js.css');
    mounted = true;

    if ($videoSrc) {
      createPlayer($videoSrc);
    }

    // Watch for seek requests from Timeline
    unsubSeek = seekRequest.subscribe((time) => {
      if (time !== null && player) {
        player.currentTime(time);
        $seekRequest = null;
      }
    });
  });

  let canPlayNatively = false;

  $: if ($videoFilePath) {
    const ext = $videoFilePath.split('.').pop()?.toLowerCase() || '';
    const mimeTypes: Record<string, string> = {
      mp4: 'video/mp4',
      webm: 'video/webm',
      ogg: 'video/ogg',
      mov: 'video/quicktime',
      mkv: 'video/x-matroska',
      avi: 'video/x-msvideo',
      flv: 'video/x-flv',
      wmv: 'video/x-ms-wmv',
    };
    const mime = mimeTypes[ext] || `video/${ext}`;
    const testVideo = document.createElement('video');
    canPlayNatively = testVideo.canPlayType(mime) !== '';
  }

  $: if (containerEl && $videoSrc && browser) {
    setTimeout(() => createPlayer($videoSrc!), 0);
  }

  function createPlayer(src: string) {
    if (!videojs || !containerEl) return;

    if (player) {
      player.src({ src, type: guessType(src) });
      player.load();
      return;
    }

    const videoEl = document.createElement('video');
    videoEl.className = 'video-js vjs-big-play-centered';
    videoEl.setAttribute('playsinline', '');
    containerEl.innerHTML = '';
    containerEl.appendChild(videoEl);

    player = videojs(videoEl, {
      controls: true,
      autoplay: false,
      preload: 'auto',
      fluid: false,
      fill: true,
      responsive: true,
      playbackRates: [0.25, 0.5, 1, 1.5, 2],
      controlBar: {
        volumePanel: { inline: false, vertical: true },
        children: [
          'playToggle',
          'volumePanel',
          'currentTimeDisplay',
          'timeDivider',
          'durationDisplay',
          'progressControl',
          'playbackRateMenuButton',
          'fullscreenToggle'
        ]
      },
      sources: [{ src, type: guessType(src) }]
    });

    let lastTimeUpdate = 0;
    player.on('timeupdate', () => {
      if (player) {
        const now = performance.now();
        // Throttle Svelte store reactivity to ~60ms (approx 16fps) to save CPU
        if (now - lastTimeUpdate > 60) {
          $currentTime = player.currentTime() || 0;
          lastTimeUpdate = now;
        }
      }
    });
    player.on('play', () => { $isPlaying = true; });
    player.on('pause', () => { $isPlaying = false; });
    player.on('volumechange', () => {
      if (player) {
        $volume = player.volume() || 1;
        $muted = player.muted() || false;
      }
    });
  }

  function guessType(src: string): string {
    if (src.includes('.webm')) return 'video/webm';
    if (src.includes('.ogv') || src.includes('.ogg')) return 'video/ogg';
    return 'video/mp4';
  }

  onDestroy(() => {
    if (unsubSeek) unsubSeek();
    if (player) {
      try { player.dispose(); } catch (_) { /* ignore */ }
      player = null;
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup} />

<div class="w-full h-full modern-card group">
  <div class="modern-card-inner flex flex-col items-center justify-center relative { $videoSrc ? '!bg-black !bg-none' : '' }">
    <div class="noise-overlay" class:hidden={$videoSrc}></div>
    <div class="relative z-10 w-full h-full flex flex-col items-center justify-center">
  {#if $isProxying}
    <!-- Background Dimmer -->
    <div class="absolute inset-0 flex flex-col items-center justify-center bg-black/40 dark:bg-black/60 backdrop-blur-sm z-10 p-8">
      
      <!-- Premium Theme-Aware Glass Card -->
      <div class="w-full max-w-lg p-8 rounded-2xl flex flex-col items-center text-center shadow-2xl relative overflow-hidden bg-white/85 dark:bg-[#1D2125]/85 backdrop-blur-2xl border border-black/10 dark:border-white/10">
        
        <div class="cube-spinner mb-6">
          <div></div>
          <div></div>
          <div></div>
          <div></div>
          <div></div>
          <div></div>
        </div>
        
        <h2 class="text-textPrimary font-semibold text-[18px] mb-2">Optimizing Video Preview</h2>
        
        {#if $videoMetadata}
          <p class="text-textSecondary text-[12px] mb-6 font-mono bg-black/5 dark:bg-black/40 px-3 py-1.5 rounded-md border border-black/10 dark:border-white/5 shadow-inner">
            {$videoMetadata.width}x{$videoMetadata.height} • {$videoMetadata.codec} • {$videoMetadata.frame_rate}
          </p>
        {/if}

        <div class="w-full bg-black/10 dark:bg-black/40 rounded-full h-3 mb-2 overflow-hidden border border-black/10 dark:border-white/10 relative shadow-inner">
          <div class="h-full bg-accent transition-all duration-300 ease-out relative" style="width: {$proxyProgress}%">
            <div class="absolute inset-0 bg-white/20 w-full opacity-50" style="background-image: linear-gradient(90deg, transparent, rgba(255,255,255,0.4), transparent); animation: shimmer 2s infinite linear;"></div>
          </div>
        </div>
        
        <div class="flex justify-between w-full text-[12px] text-textSecondary font-mono mb-6">
          <span>{Math.round($proxyProgress)}%</span>
          <span>{$proxyEta ? `ETA: ${$proxyEta}` : 'Calculating ETA...'}</span>
        </div>

        <div class="bg-warning/10 border border-warning/20 text-warning px-4 py-3 rounded-xl text-[12px] w-full mb-6 text-left flex gap-3 shadow-sm">
          <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
          <p class="leading-relaxed">Large files (4K/HEVC) may take several minutes to optimize. You can skip this step to edit immediately, but timeline scrubbing may be laggy.</p>
        </div>

        {#if canPlayNatively}
          <button 
            class="px-8 py-3 bg-black/5 dark:bg-white/10 hover:bg-black/10 dark:hover:bg-white/15 border border-black/10 dark:border-white/10 rounded-xl text-[13px] font-semibold text-textPrimary transition-all active:scale-[0.98] shadow-sm"
            on:click={() => invoke('cancel_proxy')}
          >
            Skip Proxy & Edit Now
          </button>
        {:else}
          <button 
            class="px-8 py-3 bg-black/5 dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl text-[13px] font-semibold text-textMuted cursor-not-allowed opacity-50 shadow-sm"
            title="Your browser cannot natively decode this format. Proxy generation is required."
            disabled
          >
            Skip Proxy (Format requires proxy)
          </button>
        {/if}
      </div>
    </div>
  {/if}

  {#if $videoSrc}
    <div bind:this={containerEl} class="w-full h-full video-container"></div>
  {:else}
    <div 
      class="file-upload-form"
      role="button"
      tabindex="0"
      on:click={() => {
        const chooseBtn = document.querySelectorAll('button.primary-variant')[0];
        if (chooseBtn) (chooseBtn as HTMLButtonElement).click();
      }}
      on:keydown={(e) => {
        if (e.key === 'Enter') {
          const chooseBtn = document.querySelectorAll('button.primary-variant')[0];
          if (chooseBtn) (chooseBtn as HTMLButtonElement).click();
        }
      }}
    >
      <div class="file-upload-label">
        <div class="file-upload-design">
          <svg viewBox="0 0 640 512" height="1em">
            <path d="M144 480C64.5 480 0 415.5 0 336c0-62.8 40.2-116.2 96.2-135.9c-.1-2.7-.2-5.4-.2-8.1c0-88.4 71.6-160 160-160c59.3 0 111 32.2 138.7 80.2C409.9 102 428.3 96 448 96c53 0 96 43 96 96c0 12.2-2.3 23.8-6.4 34.6C596 238.4 640 290.1 640 352c0 70.7-57.3 128-128 128H144zm79-217c-9.4 9.4-9.4 24.6 0 33.9s24.6 9.4 33.9 0l39-39V392c0 13.3 10.7 24 24 24s24-10.7 24-24V257.9l39 39c9.4 9.4 24.6 9.4 33.9 0s9.4-24.6 0-33.9l-80-80c-9.4-9.4-24.6-9.4-33.9 0l-80 80z"></path>
          </svg>
          <p class="font-semibold text-[16px] mb-1">Drag and Drop</p>
          <p class="text-[12px] opacity-70 mb-2">or</p>
          <span class="browse-button text-[13px] font-medium">Browse file</span>
        </div>
      </div>
    </div>
  {/if}
    </div>
  </div>
</div>

<style>
  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }

  .video-container { position: relative; }

  /* === FULL CUSTOM VIDEO.JS THEME === */
  .video-container :global(.video-js) {
    width: 100% !important;
    height: 100% !important;
    background-color: transparent !important; /* Let the container bg show */
    font-family: 'Inter', 'Segoe UI', system-ui, sans-serif !important;
    font-size: 13px !important;
  }
  .video-container :global(.vjs-tech) {
    object-fit: contain;
  }

  /* Control bar - Modern Floating Island */
  .video-container :global(.vjs-control-bar) {
    background: rgba(23, 26, 29, 0.3) !important;
    backdrop-filter: blur(8px) !important;
    border: 1px solid rgba(255, 255, 255, 0.1) !important;
    border-radius: 12px !important;
    height: 48px !important;
    padding: 0 12px !important;
    margin: 0 20px 20px 20px !important; /* Float above the bottom edge */
    width: auto !important; /* Let it shrink or fill */
    left: 0 !important;
    right: 0 !important;
    bottom: 0 !important;
    display: flex !important;
    align-items: center !important;
    gap: 4px !important; /* Gap between controls */
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2) !important;
    transition: background 0.3s ease, opacity 0.3s, transform 0.3s !important;
  }
  .video-container :global(.vjs-control-bar:hover) {
    background: rgba(23, 26, 29, 0.85) !important;
    backdrop-filter: blur(12px) !important;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4) !important;
  }
  
  /* Hide control bar when inactive/playing */
  .video-container :global(.vjs-has-started.vjs-user-inactive.vjs-playing .vjs-control-bar) {
    opacity: 0 !important;
    transform: translateY(10px) !important;
  }

  /* Progress Control (Timeline in player) */
  .video-container :global(.vjs-progress-control) {
    flex: 1 1 auto !important; /* Take remaining space */
    height: 100% !important;
    display: flex !important;
    align-items: center !important;
    min-width: 100px !important;
    margin: 0 8px !important;
  }
  .video-container :global(.vjs-progress-holder) {
    height: 4px !important;
    border-radius: 2px !important;
    margin: 0 !important;
    width: 100% !important;
    transition: height 120ms ease !important;
  }
  .video-container :global(.vjs-progress-control:hover .vjs-progress-holder) {
    height: 6px !important;
  }
  .video-container :global(.vjs-play-progress) {
    background-color: var(--accent) !important;
    border-radius: 2px !important;
  }
  .video-container :global(.vjs-play-progress::before) {
    /* Thumb dot */
    content: '' !important;
    position: absolute !important;
    right: -6px !important;
    top: -3px !important;
    width: 12px !important;
    height: 12px !important;
    background: var(--accent) !important;
    border-radius: 50% !important;
    border: 2px solid #fff !important;
    box-shadow: 0 0 6px rgba(76,141,255,0.6) !important;
    font-size: 0 !important;
    transition: transform 120ms ease !important;
    animation: thumbPulse 2s infinite ease-in-out;
  }
  
  @keyframes thumbPulse {
    0% { box-shadow: 0 0 0 0 rgba(76,141,255,0.7); }
    70% { box-shadow: 0 0 0 6px rgba(76,141,255,0); }
    100% { box-shadow: 0 0 0 0 rgba(76,141,255,0); }
  }

  .video-container :global(.vjs-progress-control:hover .vjs-play-progress::before) {
    transform: scale(1.25) !important;
    animation: none;
  }

  /* Buttons */
  .video-container :global(.vjs-control-bar .vjs-button) {
    width: 36px !important;
    height: 36px !important;
    border-radius: 8px !important;
    transition: background 120ms ease, transform 120ms ease !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    cursor: pointer !important;
  }
  .video-container :global(.vjs-control-bar .vjs-button:hover) {
    background: rgba(255,255,255,0.12) !important;
  }
  .video-container :global(.vjs-control-bar .vjs-button:active) {
    transform: scale(0.92) !important;
  }
  .video-container :global(.vjs-icon-placeholder::before) {
    font-size: 18px !important;
    line-height: 1 !important;
    position: static !important; /* Fix alignment */
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
  }

  /* Play button icon size */
  .video-container :global(.vjs-play-control .vjs-icon-placeholder::before) {
    font-size: 22px !important;
  }

  /* Time display */
  .video-container :global(.vjs-time-control) {
    font-size: 12px !important;
    font-family: 'JetBrains Mono', 'Fira Code', monospace !important;
    color: rgba(255,255,255,0.8) !important;
    line-height: 1 !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    padding: 0 !important;
    min-width: auto !important;
  }
  .video-container :global(.vjs-time-divider) {
    color: rgba(255,255,255,0.4) !important;
    padding: 0 4px !important;
    display: flex !important;
    align-items: center !important;
  }

  /* Volume Panel (Vertical Popup) */
  .video-container :global(.vjs-volume-panel) {
    display: flex !important;
    align-items: center !important;
    margin-right: 4px !important;
  }
  .video-container :global(.vjs-volume-panel.vjs-volume-panel-vertical .vjs-volume-control) {
    position: absolute !important;
    bottom: 100% !important;
    left: 50% !important;
    transform: translateX(-50%) !important;
    background: rgba(23,26,29,0.95) !important;
    backdrop-filter: blur(12px) !important;
    border: 1px solid rgba(255,255,255,0.1) !important;
    border-radius: 8px !important;
    padding: 12px 0 !important;
    height: 100px !important;
    width: 36px !important;
    box-shadow: 0 4px 20px rgba(0,0,0,0.5) !important;
    margin-bottom: 8px !important;
    z-index: 50;
    transition: opacity 0.2s, transform 0.2s;
  }
  .video-container :global(.vjs-volume-level) {
    background-color: var(--accent) !important;
    width: 100% !important;
  }
  .video-container :global(.vjs-volume-bar) {
    width: 4px !important;
    margin: 0 auto !important;
    border-radius: 2px !important;
    background: rgba(255,255,255,0.2) !important;
    height: 100% !important;
  }

  /* Playback rate menu */
  .video-container :global(.vjs-playback-rate) {
    width: 44px !important;
  }
  .video-container :global(.vjs-playback-rate .vjs-playback-rate-value) {
    font-size: 12px !important;
    font-weight: 600 !important;
    line-height: 1 !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    height: 100% !important;
  }

  /* Menus */
  .video-container :global(.vjs-menu) {
    border-radius: 8px !important;
    overflow: hidden !important;
  }
  .video-container :global(.vjs-menu-content) {
    background: rgba(23,26,29,0.95) !important;
    backdrop-filter: blur(12px) !important;
    border: 1px solid rgba(255,255,255,0.1) !important;
    border-radius: 8px !important;
    box-shadow: 0 4px 20px rgba(0,0,0,0.5) !important;
    bottom: 48px !important; /* Push above control bar */
  }
  .video-container :global(.vjs-menu-item) {
    font-size: 12px !important;
    padding: 8px 16px !important;
    transition: background 100ms !important;
  }
  .video-container :global(.vjs-menu-item:hover) {
    background: rgba(76,141,255,0.15) !important;
    color: #fff !important;
  }
  .video-container :global(.vjs-menu-item-text) {
    font-family: 'Inter', sans-serif !important;
  }

  /* Big play button (Center screen) */
  .video-container :global(.vjs-big-play-button) {
    background: rgba(23, 26, 29, 0.6) !important;
    backdrop-filter: blur(8px) !important;
    border: 1px solid rgba(255, 255, 255, 0.1) !important;
    border-radius: 50% !important;
    width: 72px !important;
    height: 72px !important;
    line-height: 72px !important;
    font-size: 32px !important;
    color: #fff !important;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3) !important;
    transition: transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275), background 0.2s, box-shadow 0.2s !important;
    margin: 0 !important;
    top: 50% !important;
    left: 50% !important;
    transform: translate(-50%, -50%) !important;
  }
  .video-container :global(.vjs-big-play-button:hover) {
    background: rgba(76,141,255,0.9) !important;
    border-color: rgba(255,255,255,0.2) !important;
    transform: translate(-50%, -50%) scale(1.1) !important;
    box-shadow: 0 12px 40px rgba(76,141,255,0.4) !important;
  }
  .video-container :global(.vjs-big-play-button:active) {
    transform: translate(-50%, -50%) scale(0.95) !important;
  }
  .video-container :global(.vjs-big-play-button .vjs-icon-placeholder::before) {
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    position: static !important;
  }

  /* Loading spinner */
  .video-container :global(.vjs-loading-spinner) {
    border-color: rgba(255,255,255,0.1) !important;
    border-top-color: var(--accent) !important;
    border-radius: 50% !important;
  }

  /* Load progress (buffered) */
  .video-container :global(.vjs-load-progress) {
    background: rgba(255,255,255,0.1) !important;
    border-radius: 2px !important;
  }
  .video-container :global(.vjs-load-progress div) {
    background: rgba(255,255,255,0.2) !important;
  }

  /* Slider focus */
  .video-container :global(.vjs-slider:focus) {
    box-shadow: none !important; /* Removing default ugly ring, using thumb scale instead */
  }

  /* Tooltips */
  .video-container :global(.vjs-time-tooltip),
  .video-container :global(.vjs-mouse-display .vjs-time-tooltip) {
    background: rgba(23,26,29,0.95) !important;
    border: 1px solid rgba(255,255,255,0.1) !important;
    border-radius: 6px !important;
    font-size: 11px !important;
    font-family: 'JetBrains Mono', monospace !important;
    padding: 4px 8px !important;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4) !important;
  }

  /* From Uiverse.io by AqFox - Theme Aware 3D Cube */
  .cube-spinner {
    width: 44px;
    height: 44px;
    animation: spinner-y0fdc1 2.5s infinite ease;
    transform-style: preserve-3d;
  }

  .cube-spinner > div {
    /* Mix 20% of the active accent color with transparent to get a dynamic theme-aware translucent blue */
    background-color: color-mix(in srgb, var(--accent) 20%, transparent);
    height: 100%;
    position: absolute;
    width: 100%;
    border: 2px solid var(--accent);
    box-shadow: 0 0 15px color-mix(in srgb, var(--accent) 40%, transparent);
  }

  .cube-spinner div:nth-of-type(1) {
    transform: translateZ(-22px) rotateY(180deg);
  }

  .cube-spinner div:nth-of-type(2) {
    transform: rotateY(-270deg) translateX(50%);
    transform-origin: top right;
  }

  .cube-spinner div:nth-of-type(3) {
    transform: rotateY(270deg) translateX(-50%);
    transform-origin: center left;
  }

  .cube-spinner div:nth-of-type(4) {
    transform: rotateX(90deg) translateY(-50%);
    transform-origin: top center;
  }

  .cube-spinner div:nth-of-type(5) {
    transform: rotateX(-90deg) translateY(50%);
    transform-origin: bottom center;
  }

  .cube-spinner div:nth-of-type(6) {
    transform: translateZ(22px);
  }

  @keyframes spinner-y0fdc1 {
    0% {
      transform: rotate(45deg) rotateX(-25deg) rotateY(25deg);
    }
    50% {
      transform: rotate(45deg) rotateX(-385deg) rotateY(25deg);
    }
    100% {
      transform: rotate(45deg) rotateX(-385deg) rotateY(385deg);
    }
  }

  /* File Upload Design */
  .file-upload-form {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    outline: none;
  }

  .file-upload-label {
    cursor: pointer;
    background-color: color-mix(in srgb, var(--surface) 50%, transparent);
    padding: 30px 70px;
    border-radius: 40px;
    border: 2px dashed var(--border);
    box-shadow: 0px 0px 100px -50px rgba(0, 0, 0, 0.5);
    color: var(--text-primary);
    transition: all 0.3s;
  }

  .file-upload-form:hover .file-upload-label {
    border-color: var(--accent);
    background-color: color-mix(in srgb, var(--surface) 80%, var(--accent) 5%);
    transform: scale(1.02);
  }

  .file-upload-label svg {
    height: 50px;
    fill: var(--text-secondary);
    margin-bottom: 20px;
    transition: all 0.3s;
  }
  
  .file-upload-form:hover .file-upload-label svg {
    fill: var(--accent);
  }

  .file-upload-design {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 5px;
  }

  .browse-button {
    background-color: var(--border);
    padding: 5px 15px;
    border-radius: 10px;
    color: var(--text-primary);
    transition: all 0.3s;
    margin-top: 8px;
  }

  .file-upload-form:hover .browse-button {
    background-color: var(--accent);
    color: white;
  }
</style>
