<script lang="ts">
  import { videoFilePath, videoSrc, isProxying, proxyProgress, proxyEta, videoMetadata, currentTime, isPlaying, volume, muted, seekRequest, exportRequest, trimStart, trimEnd, videoDuration, videoZoom, videoPanX, videoPanY, exportHistory, openVideoRequest, isFullscreen } from '$lib/store';
  import FullscreenHud from './FullscreenHud.svelte';
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import { invoke } from '@tauri-apps/api/core';
  import { FilmStrip, MagnifyingGlassPlus, Folder, Clock, FileVideo } from 'phosphor-svelte';
  
  function formatRelativeDate(iso: string) {
    const date = new Date(iso);
    const now = new Date();
    const isToday = date.getDate() === now.getDate() && date.getMonth() === now.getMonth() && date.getFullYear() === now.getFullYear();
    const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (isToday) return `Today, ${time}`;
    return `${date.toLocaleDateString()} ${time}`;
  }
  
  function triggerOpen() {
    $openVideoRequest = Date.now();
  }

  let containerEl: HTMLDivElement;
  let player: any = null;
  let videojs: any = null;
  let mounted = false;
  let isFullscreenLocal = false;

  function toggleFullscreen() {
    if (!player) return;
    if (player.isFullscreen()) {
      player.exitFullscreen();
    } else {
      player.requestFullscreen();
    }
  }

  function portalTo(node: HTMLElement, target: HTMLElement | null) {
    if (!target) return;
    target.appendChild(node);
    return {
      update(newTarget: HTMLElement | null) {
        if (newTarget && newTarget !== target) {
          newTarget.appendChild(node);
          target = newTarget;
        }
      },
      destroy() {
        if (node.parentNode) {
          node.parentNode.removeChild(node);
        }
      }
    };
  }

  function createCustomBtnClass(ButtonClass: any, tooltip: string, name: string, clickHandler: () => void) {
    return class extends ButtonClass {
      constructor(player: any, options: any) {
        super(player, options);
        this.controlText(tooltip);
      }
      buildCSSClass() {
        return `vjs-custom-btn vjs-${name.toLowerCase()} ${super.buildCSSClass()}`;
      }
      handleClick() {
        clickHandler();
      }
    };
  }

  let unsubSeek: () => void;
  
  let isPanning = false;
  let startPanX = 0;
  let startPanY = 0;
  let lastPanX = 0;
  let lastPanY = 0;
  let showZoomIndicator = false;
  let zoomTimeout: any;

  function handleWheel(e: WheelEvent) {
    if (!$videoSrc) return;
    const target = e.target as HTMLElement;
    if (!containerEl.contains(target)) return;

    e.preventDefault();
    const delta = e.deltaY * -0.002;
    const newZoom = Math.min(Math.max(1, $videoZoom + delta), 5);
    $videoZoom = newZoom;
    
    if ($videoZoom === 1) {
      $videoPanX = 0;
      $videoPanY = 0;
    }

    showZoomIndicator = true;
    clearTimeout(zoomTimeout);
    zoomTimeout = setTimeout(() => showZoomIndicator = false, 1500);
  }

  function handleMouseDown(e: MouseEvent) {
    if ($videoZoom <= 1 || e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (!containerEl.contains(target)) return;

    isPanning = true;
    startPanX = e.clientX;
    startPanY = e.clientY;
    lastPanX = $videoPanX;
    lastPanY = $videoPanY;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isPanning) return;
    $videoPanX = lastPanX + (e.clientX - startPanX);
    $videoPanY = lastPanY + (e.clientY - startPanY);
  }

  function handleMouseUp() {
    isPanning = false;
  }

  function handleDoubleClick() {
    if ($videoZoom > 1) {
      $videoZoom = 1;
      $videoPanX = 0;
      $videoPanY = 0;
      showZoomIndicator = true;
      clearTimeout(zoomTimeout);
      zoomTimeout = setTimeout(() => showZoomIndicator = false, 1500);
    } else {
      toggleFullscreen();
    }
  }

  $: if (browser && containerEl && player) {
    const tech = containerEl.querySelector('.vjs-tech') as HTMLElement;
    if (tech) {
      tech.style.transform = `translate(${$videoPanX}px, ${$videoPanY}px) scale(${$videoZoom})`;
      tech.style.transition = isPanning ? 'none' : 'transform 0.1s ease-out';
      tech.style.cursor = $videoZoom > 1 ? (isPanning ? 'grabbing' : 'grab') : 'default';
    }
  }

  let isFastForwarding = false;
  let isRewinding = false;
  let rewindInterval: any;
  let wasPlayingBeforeFF = false;

  function handleKeydown(e: KeyboardEvent) {
    if (e.target && ['INPUT', 'TEXTAREA', 'SELECT'].includes((e.target as HTMLElement).tagName)) return;
    if (!player) return;

    if (e.key === ' ') {
      e.preventDefault();
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
    } else if (e.key.toLowerCase() === 'i') {
      e.preventDefault();
      $trimStart = Math.min($currentTime, $trimEnd - 0.1);
    } else if (e.key.toLowerCase() === 'o') {
      e.preventDefault();
      $trimEnd = Math.max($currentTime, $trimStart + 0.1);
    } else if (e.key === 'ArrowLeft' && e.shiftKey) {
      e.preventDefault();
      $seekRequest = Math.max(0, $currentTime - 5);
    } else if (e.key === 'ArrowRight' && e.shiftKey) {
      e.preventDefault();
      $seekRequest = Math.min($videoDuration, $currentTime + 5);
    } else if (e.key === 'ArrowRight' && !e.shiftKey) {
      e.preventDefault();
      if (e.repeat && !isFastForwarding) {
        isFastForwarding = true;
        wasPlayingBeforeFF = !player.paused();
        player.playbackRate(4.0);
        player.play();
      } else if (!e.repeat) {
        $seekRequest = Math.min($videoDuration, $currentTime + 0.1);
      }
    } else if (e.key === 'ArrowLeft' && !e.shiftKey) {
      e.preventDefault();
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
    } else if (e.key.toLowerCase() === 'f' && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      toggleFullscreen();
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

  function handleBlur() {
    if (isFastForwarding) {
      isFastForwarding = false;
      if (player) {
        player.playbackRate(1.0);
        if (!wasPlayingBeforeFF) player.pause();
      }
    }
    if (isRewinding) {
      isRewinding = false;
      clearInterval(rewindInterval);
      if (player && wasPlayingBeforeFF) player.play();
    }
    isPanning = false;
  }

  let lastLoadedFilePath = '';
  $: if ($videoFilePath !== lastLoadedFilePath) {
    lastLoadedFilePath = $videoFilePath || '';
    $videoZoom = 1;
    $videoPanX = 0;
    $videoPanY = 0;
  }

  onMount(async () => {
    const mod = await import('video.js');
    videojs = mod.default;
    mounted = true;

    if ($videoSrc) {
      createPlayer($videoSrc);
    }

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

  $: if (containerEl && $videoSrc && browser && videojs) {
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

    const Button = videojs.getComponent('Button');
    
    const registerCustomBtn = (name: string, tooltip: string, clickHandler: () => void) => {
      if (!videojs.getComponent(name)) {
        const CustomBtn = createCustomBtnClass(Button, tooltip, name, clickHandler);
        videojs.registerComponent(name, CustomBtn);
      }
    };

    registerCustomBtn('FrameBack', 'Previous Frame', () => {
      if (player) {
        const cur = player.currentTime() || 0;
        const target = Math.max($trimStart, cur - (1/30));
        player.currentTime(target);
      }
    });
    registerCustomBtn('SkipBack', 'Skip Backward 5s', () => {
      if (player) {
        const cur = player.currentTime() || 0;
        const target = Math.max($trimStart, cur - 5);
        player.currentTime(target);
      }
    });
    registerCustomBtn('SkipForward', 'Skip Forward 5s', () => {
      if (player) {
        const cur = player.currentTime() || 0;
        const maxBoundary = $trimEnd > $trimStart ? $trimEnd : player.duration();
        const target = Math.min(maxBoundary, cur + 5);
        player.currentTime(target);
      }
    });
    registerCustomBtn('FrameForward', 'Next Frame', () => {
      if (player) {
        const cur = player.currentTime() || 0;
        const maxBoundary = $trimEnd > $trimStart ? $trimEnd : player.duration();
        const target = Math.min(maxBoundary, cur + (1/30));
        player.currentTime(target);
      }
    });

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
          'FrameBack',
          'SkipBack',
          'playToggle',
          'SkipForward',
          'FrameForward',
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

    let rangeWatcherId: number | null = null;
    function startRangeWatcher() {
      stopRangeWatcher();
      function checkRange() {
        if (player && !player.paused()) {
          const cur = player.currentTime() || 0;
          if ($trimEnd > $trimStart && cur >= $trimEnd) {
            player.pause();
            player.currentTime($trimStart);
            $currentTime = $trimStart;
            return;
          }
          rangeWatcherId = requestAnimationFrame(checkRange);
        }
      }
      rangeWatcherId = requestAnimationFrame(checkRange);
    }

    function stopRangeWatcher() {
      if (rangeWatcherId !== null) {
        cancelAnimationFrame(rangeWatcherId);
        rangeWatcherId = null;
      }
    }

    let lastTimeUpdate = 0;
    player.on('timeupdate', () => {
      if (player) {
        const cur = player.currentTime() || 0;
        if ($trimEnd > $trimStart && cur >= $trimEnd) {
          player.pause();
          player.currentTime($trimStart);
          $currentTime = $trimStart;
          return;
        }
        const now = performance.now();
        if (now - lastTimeUpdate > 60) {
          $currentTime = cur;
          lastTimeUpdate = now;
        }
      }
    });

    player.on('play', () => {
      $isPlaying = true;
      if (player) {
        const cur = player.currentTime() || 0;
        if ($trimEnd > $trimStart && (cur >= $trimEnd - 0.05 || cur < $trimStart)) {
          player.currentTime($trimStart);
          $currentTime = $trimStart;
        }
      }
      startRangeWatcher();
    });

    player.on('pause', () => {
      $isPlaying = false;
      stopRangeWatcher();
    });

    player.on('volumechange', () => {
      if (player) {
        $volume = player.volume() || 1;
        $muted = player.muted() || false;
      }
    });

    player.on('fullscreenchange', () => {
      isFullscreenLocal = player.isFullscreen();
      $isFullscreen = isFullscreenLocal;
    });
  }

  function guessType(src: string): string {
    if (src.includes('.webm')) return 'video/webm';
    if (src.includes('.ogv') || src.includes('.ogg')) return 'video/ogg';
    return 'video/mp4';
  }

  onDestroy(() => {
    if (unsubSeek) unsubSeek();
    if (rewindInterval) clearInterval(rewindInterval);
    if (zoomTimeout) clearTimeout(zoomTimeout);
    if (player) {
      try { player.dispose(); } catch (_) { }
      player = null;
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup} on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} on:blur={handleBlur} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div 
  class="w-full h-full bg-black relative overflow-hidden"
  class:cursor-default={!$videoSrc}
  class:cursor-zoom-in={$videoSrc && $videoZoom === 1}
  class:cursor-grab={$videoZoom > 1}
  class:cursor-grabbing={isPanning}
  on:wheel|nonpassive={handleWheel}
  on:mousedown={handleMouseDown}
  on:dblclick={handleDoubleClick}
>
  <div class="relative z-10 w-full h-full flex flex-col items-center justify-center">
    
    {#if $isProxying}
      <div class="absolute inset-0 flex flex-col items-center justify-center bg-black/70 backdrop-blur-md z-10 p-8">
        <div class="w-full max-w-lg p-8 rounded-2xl flex flex-col items-center text-center shadow-2xl relative overflow-hidden bg-[var(--bg-elevated)] border border-[var(--border-base)]">
          <div class="w-10 h-10 border-2 border-[var(--border-base)] border-t-[var(--accent)] rounded-full animate-spin mb-6"></div>
          <h2 class="text-[var(--text-primary)] font-semibold text-[16px] mb-2">Optimizing Video Preview</h2>
          
          {#if $videoMetadata}
            <p class="text-[var(--text-secondary)] text-[12px] mb-6 font-mono bg-[var(--bg-deep)] px-3 py-1.5 rounded-md border border-[var(--border-base)]">
              {$videoMetadata.width}x{$videoMetadata.height} • {$videoMetadata.codec} • {$videoMetadata.frame_rate}
            </p>
          {/if}

          <div class="w-full bg-[var(--bg-deep)] rounded-full h-2 mb-2 overflow-hidden border border-[var(--border-base)]">
            <div class="h-full bg-[var(--accent)] transition-all duration-300 ease-out" style="width: {$proxyProgress}%"></div>
          </div>
          
          <div class="flex justify-between w-full text-[11px] text-[var(--text-muted)] font-mono mb-6">
            <span>{Math.round($proxyProgress)}%</span>
            <span>{$proxyEta ? `ETA: ${$proxyEta}` : 'Calculating...'}</span>
          </div>

          <button class="btn-ghost px-6 text-[13px] font-medium" on:click={() => invoke('cancel_proxy')}>
            Skip Proxy & Edit Directly
          </button>
        </div>
      </div>
    {/if}

    <div 
      bind:this={containerEl} 
      class="w-full h-full video-container" 
      class:hidden={!$videoSrc}
    ></div>

    {#if player && isFullscreenLocal}
      <div use:portalTo={player.el()}>
        <FullscreenHud {player} onExitFullscreen={toggleFullscreen} />
      </div>
    {/if}

    {#if !$videoSrc}
      <div class="absolute inset-0 flex flex-col w-full h-full p-8 overflow-y-auto no-scrollbar gap-8">
        
        <!-- Dashboard Dropzone -->
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div class="w-full shrink-0 h-[220px] rounded-2xl bg-[var(--bg-elevated)] border-2 border-dashed border-[var(--border-strong)] flex flex-col items-center justify-center gap-4 cursor-pointer hover:bg-[var(--bg-surface)] hover:border-[var(--accent)] transition-colors group" on:click={triggerOpen}>
          <div class="w-16 h-16 rounded-2xl bg-[var(--bg-surface)] border border-[var(--border-base)] flex items-center justify-center text-[var(--text-muted)] group-hover:text-[var(--accent)] group-hover:scale-110 transition-all duration-300">
            <FilmStrip size={32} weight="regular" />
          </div>
          <div class="text-center">
            <p class="text-[15px] font-semibold text-[var(--text-primary)]">Drop a video to begin</p>
            <p class="text-[12.5px] text-[var(--text-muted)] mt-1">or click here to open a file</p>
          </div>
        </div>

        <!-- History Section -->
        <div class="flex-1 flex flex-col gap-4">
          <div class="flex items-center justify-between">
            <h3 class="text-[13px] font-semibold tracking-wide uppercase text-[var(--text-muted)]">Recent Cuts</h3>
          </div>
          
          {#if $exportHistory.length === 0}
            <div class="flex-1 flex flex-col items-center justify-center gap-2 opacity-50 py-10">
              <Folder size={32} weight="thin" class="text-[var(--text-muted)]" />
              <p class="text-[12px] text-[var(--text-muted)] font-medium">No recent cuts found</p>
            </div>
          {:else}
            <div class="grid grid-cols-2 gap-4 pb-8">
              {#each $exportHistory as record}
                <div class="bg-[var(--bg-elevated)] border border-[var(--border-base)] rounded-xl p-4 flex flex-col gap-3 hover:border-[var(--border-strong)] transition-colors">
                  <div class="flex items-start justify-between gap-3">
                    <div class="flex-1 min-w-0">
                      <p class="text-[13px] font-medium text-[var(--text-primary)] truncate" title={record.outputPath.split(/[\\/]/).pop()}>
                        {record.outputPath.split(/[\\/]/).pop()}
                      </p>
                      <p class="text-[11.5px] text-[var(--text-muted)] truncate flex items-center gap-1.5 mt-0.5">
                        <FileVideo size={12} />
                        from {record.sourcePath.split(/[\\/]/).pop()}
                      </p>
                    </div>
                  </div>
                  
                  <div class="flex items-center gap-4 text-[11px] font-mono text-[var(--text-secondary)] mt-auto pt-2 border-t border-[var(--border-subtle)]">
                    <span class="flex items-center gap-1"><Clock size={12} /> {record.duration.toFixed(2)}s cut</span>
                    <span>{formatRelativeDate(record.date)}</span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      </div>
    {/if}

    {#if showZoomIndicator}
      <div class="absolute top-3 right-3 z-50 backdrop-blur-md border border-[var(--border-base)] rounded-full px-3 py-1 flex items-center gap-1 opacity-100 transition-opacity" style="background: var(--bg-tooltip);">
        <MagnifyingGlassPlus size={11} color="var(--text-secondary)" />
        <span class="font-mono text-[10px] text-[var(--text-secondary)]">{Math.round($videoZoom * 100)}%</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .video-container { position: relative; width: 100%; height: 100%; }

  .video-container :global(.video-js) {
    width: 100% !important;
    height: 100% !important;
    background-color: transparent !important;
    font-family: var(--font-body) !important;
    font-size: 12px !important;
  }
  .video-container :global(.vjs-tech) {
    object-fit: contain;
  }

  /* Glass Floating Control Bar */
  .video-container :global(.vjs-control-bar) {
    position: absolute !important;
    bottom: 16px !important;
    left: 50% !important;
    transform: translateX(-50%) !important;
    width: calc(100% - 48px) !important;
    max-width: 720px !important;
    height: 48px !important;
    border-radius: var(--radius-2xl) !important;
    background: rgba(10, 10, 12, 0.70) !important;
    backdrop-filter: blur(28px) saturate(180%) !important;
    -webkit-backdrop-filter: blur(28px) saturate(180%) !important;
    border: 1px solid rgba(255, 255, 255, 0.07) !important;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset, 0 4px 6px rgba(0, 0, 0, 0.4), 0 16px 40px rgba(0, 0, 0, 0.3) !important;
    display: flex !important;
    align-items: center !important;
    padding: 0 8px !important;
    gap: 4px !important;
    transition: opacity 150ms ease, transform 300ms ease !important;
  }

  :global(.light) .video-container :global(.vjs-control-bar) {
    background: rgba(255, 255, 255, 0.85) !important;
    border-color: rgba(0, 0, 0, 0.08) !important;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 1) inset, 0 4px 12px rgba(0, 0, 0, 0.1), 0 16px 40px rgba(0, 0, 0, 0.1) !important;
  }

  /* Fullscreen: hide default video.js control bar so our Studio Fullscreen HUD takes over */
  :global(.video-js.vjs-fullscreen .vjs-control-bar) {
    display: none !important;
  }


  .video-container :global(.vjs-has-started.vjs-user-inactive.vjs-playing .vjs-control-bar) {
    opacity: 0 !important;
    transform: translate(-50%, 10px) !important;
    pointer-events: none !important;
  }

  /* Progress Scrubber */
  .video-container :global(.vjs-progress-control) {
    flex: 1 1 auto !important;
    display: flex !important;
    align-items: center !important;
    min-width: 100px !important;
    margin: 0 12px !important;
    cursor: ew-resize !important;
  }
  .video-container :global(.vjs-progress-holder) {
    height: 3px !important;
    border-radius: 9999px !important;
    margin: 0 !important;
    width: 100% !important;
    background: rgba(255,255,255,0.12) !important;
    transition: height 120ms ease !important;
    cursor: ew-resize !important;
  }
  :global(.light) .video-container :global(.vjs-progress-holder) {
    background: rgba(0,0,0,0.1) !important;
  }
  .video-container :global(.vjs-progress-control:hover .vjs-progress-holder) {
    height: 5px !important;
  }
  .video-container :global(.vjs-play-progress) {
    background-color: var(--accent) !important;
    background-image: var(--accent-gradient) !important;
    border-radius: 9999px !important;
  }
  .video-container :global(.vjs-play-progress::before) {
    content: '' !important;
    position: absolute !important;
    right: -6px !important;
    top: 50% !important;
    margin-top: -6px !important;
    width: 12px !important;
    height: 12px !important;
    background: white !important;
    border-radius: 50% !important;
    box-shadow: 0 0 0 3px var(--accent-dim) !important;
    font-size: 0 !important;
    opacity: 0 !important;
    transform: scale(0) !important;
    transition: all var(--dur-fast) var(--ease-spring) !important;
  }
  .video-container :global(.vjs-progress-control:hover .vjs-play-progress::before) {
    opacity: 1 !important;
    transform: scale(1) !important;
  }

  /* Default Button Styling */
  .video-container :global(.vjs-control-bar .vjs-button) {
    width: 30px !important;
    height: 30px !important;
    border-radius: var(--radius-md) !important;
    background: transparent !important;
    color: rgba(255,255,255,0.65) !important;
    transition: all 100ms var(--ease-out) !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    cursor: pointer !important;
  }
  :global(.light) .video-container :global(.vjs-control-bar .vjs-button) {
    color: rgba(20,20,30,0.75) !important;
  }
  .video-container :global(.vjs-control-bar .vjs-button:hover) {
    background: rgba(255,255,255,0.08) !important;
    color: white !important;
  }
  :global(.light) .video-container :global(.vjs-control-bar .vjs-button:hover) {
    background: rgba(0,0,0,0.05) !important;
    color: rgba(20,20,30,1) !important;
  }
  .video-container :global(.vjs-control-bar .vjs-button:active) {
    transform: scale(0.96) !important;
  }
  .video-container :global(.vjs-icon-placeholder::before) {
    line-height: 1 !important;
    font-size: 15px !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
  }
  .video-container :global(.vjs-custom-btn .vjs-icon-placeholder::before) {
    content: '' !important;
  }

  /* Play Button (Center Hero in Control Bar) */
  .video-container :global(.vjs-play-control) {
    width: 38px !important;
    height: 38px !important;
    border-radius: var(--radius-md) !important;
    background-color: var(--accent) !important;
    background-image: var(--accent-gradient) !important;
    color: #fff !important;
    box-shadow: 0 2px 10px rgba(var(--accent-rgb), 0.35) !important;
    transition: all 120ms var(--ease-out) !important;
    margin: 0 4px !important;
    overflow: hidden !important;
    position: relative !important;
  }
  :global(.light) .video-container :global(.vjs-play-control) {
    color: #fff !important; /* Make sure it stays white against accent */
  }
  :global(.light) .video-container :global(.vjs-play-control:hover) {
    color: #fff !important;
  }
  .video-container :global(.vjs-play-control:hover) {
    filter: brightness(1.08) !important;
    transform: translateY(-1px) !important;
    box-shadow: 0 4px 14px rgba(var(--accent-rgb), 0.45) !important;
  }
  .video-container :global(.vjs-play-control:active) {
    transform: scale(0.96) translateY(0) !important;
    transition-duration: 60ms !important;
  }

  /* Navigation SVG Icons (monoline Phosphor style via CSS Masks) */
  .video-container :global(.vjs-frameback .vjs-icon-placeholder),
  .video-container :global(.vjs-skipback .vjs-icon-placeholder),
  .video-container :global(.vjs-skipforward .vjs-icon-placeholder),
  .video-container :global(.vjs-frameforward .vjs-icon-placeholder) {
    width: 16px !important; 
    height: 16px !important;
    background-color: currentColor !important;
  }

  .video-container :global(.vjs-frameback .vjs-icon-placeholder) {
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M160,208L80,128L160,48'/%3E%3C/svg%3E") no-repeat center center / contain;
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M160,208L80,128L160,48'/%3E%3C/svg%3E") no-repeat center center / contain;
  }
  
  .video-container :global(.vjs-skipback .vjs-icon-placeholder) {
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M200,208L120,128L200,48M112,208L32,128L112,48'/%3E%3C/svg%3E") no-repeat center center / contain;
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M200,208L120,128L200,48M112,208L32,128L112,48'/%3E%3C/svg%3E") no-repeat center center / contain;
  }
  
  .video-container :global(.vjs-skipforward .vjs-icon-placeholder) {
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M56,208L136,128L56,48M144,208L224,128L144,48'/%3E%3C/svg%3E") no-repeat center center / contain;
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M56,208L136,128L56,48M144,208L224,128L144,48'/%3E%3C/svg%3E") no-repeat center center / contain;
  }
  
  .video-container :global(.vjs-frameforward .vjs-icon-placeholder) {
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M96,208L176,128L96,48'/%3E%3C/svg%3E") no-repeat center center / contain;
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'%3E%3Cpath fill='none' stroke='black' stroke-linecap='round' stroke-linejoin='round' stroke-width='16' d='M96,208L176,128L96,48'/%3E%3C/svg%3E") no-repeat center center / contain;
  }

  /* Time display */
  .video-container :global(.vjs-time-control) {
    font-family: var(--font-mono) !important;
    font-size: 10.5px !important;
    color: rgba(255,255,255,0.55) !important;
    letter-spacing: 0.04em !important;
    white-space: nowrap !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    padding: 0 !important;
    min-width: auto !important;
  }
  :global(.light) .video-container :global(.vjs-time-control) {
    color: rgba(20,20,30,0.65) !important;
  }
  .video-container :global(.vjs-time-divider) {
    color: rgba(255,255,255,0.30) !important;
    padding: 0 4px !important;
    display: flex !important;
    align-items: center !important;
  }
  :global(.light) .video-container :global(.vjs-time-divider) {
    color: rgba(20,20,30,0.40) !important;
  }

  /* Volume Panel */
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
    background: rgba(10, 10, 12, 0.70) !important;
    backdrop-filter: blur(24px) saturate(180%) !important;
    border: 1px solid rgba(255, 255, 255, 0.09) !important;
    border-radius: 12px !important;
    padding: 12px 0 !important;
    height: 100px !important;
    width: 36px !important;
    box-shadow: 0 4px 20px rgba(0,0,0,0.5) !important;
    margin-bottom: 8px !important;
    z-index: 50;
  }
  .video-container :global(.vjs-volume-level) {
    background-color: var(--accent) !important;
    width: 100% !important;
  }
  .video-container :global(.vjs-volume-bar) {
    width: 3px !important;
    margin: 0 auto !important;
    border-radius: 9999px !important;
    background: rgba(255,255,255,0.2) !important;
    height: 100% !important;
    cursor: ns-resize !important;
  }

  /* Playback rate menu */
  .video-container :global(.vjs-playback-rate) {
    width: 44px !important;
  }
  .video-container :global(.vjs-playback-rate .vjs-playback-rate-value) {
    font-family: var(--font-mono) !important;
    font-size: 11px !important;
    font-weight: 500 !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    height: 100% !important;
  }

  /* Menus */
  .video-container :global(.vjs-menu) {
    border-radius: 12px !important;
    overflow: hidden !important;
  }
  .video-container :global(.vjs-menu-content) {
    background: rgba(10, 10, 12, 0.70) !important;
    backdrop-filter: blur(24px) saturate(180%) !important;
    border: 1px solid rgba(255,255,255,0.09) !important;
    border-radius: 12px !important;
    box-shadow: 0 4px 20px rgba(0,0,0,0.5) !important;
    bottom: 48px !important;
  }
  .video-container :global(.vjs-menu-item) {
    font-size: 12px !important;
    padding: 8px 16px !important;
    transition: background 100ms !important;
    cursor: pointer !important;
    border-radius: 8px !important;
    margin: 2px !important;
  }
  .video-container :global(.vjs-menu-item:hover) {
    background: rgba(255,255,255,0.1) !important;
    color: #fff !important;
  }
  .video-container :global(.vjs-menu-item.vjs-selected) {
    color: var(--accent) !important;
    background: var(--accent-dim) !important;
  }

  /* Big play button (Center screen empty state) */
  .video-container :global(.vjs-big-play-button) {
    background: rgba(10, 10, 12, 0.65) !important;
    backdrop-filter: blur(16px) !important;
    border: 1px solid rgba(255,255,255,0.10) !important;
    border-radius: var(--radius-lg) !important; /* 12px square */
    width: 56px !important;
    height: 56px !important;
    line-height: 56px !important;
    font-size: 24px !important;
    color: #fff !important;
    transition: all 200ms var(--ease-spring) !important;
    margin: 0 !important;
    top: 50% !important;
    left: 50% !important;
    transform: translate(-50%, -50%) !important;
    cursor: pointer !important;
  }
  .video-container :global(.vjs-big-play-button:hover) {
    background: var(--accent) !important;
    border-color: var(--accent-bright) !important;
    transform: translate(-50%, -50%) scale(1.10) !important;
    box-shadow: 0 8px 32px var(--glow-accent) !important;
  }
  .video-container :global(.vjs-big-play-button:active) {
    transform: translate(-50%, -50%) scale(0.94) !important;
    transition-duration: 80ms !important;
  }
  .video-container :global(.vjs-big-play-button .vjs-icon-placeholder::before) {
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    position: static !important;
  }
</style>
