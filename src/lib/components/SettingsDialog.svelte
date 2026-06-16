<script lang="ts">
  import { showSettings, previewQuality, isDarkMode, exportFormat } from '$lib/store';

  function closeSettings() {
    $showSettings = false;
  }
</script>

{#if $showSettings}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center" on:click|self={closeSettings}>
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/40 backdrop-blur-sm"></div>

    <!-- Panel -->
    <div class="relative glass-dialog w-[480px] max-h-[80vh] overflow-y-auto p-6 animate-in">

      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <h2 class="text-[18px] font-semibold text-textPrimary">Settings</h2>
          <a href="https://github.com/im-ushan-ikshana" target="_blank" rel="noopener noreferrer" class="github-btn" title="Developer GitHub Profile">
            <span class="svgContainer">
              <svg fill="currentColor" viewBox="0 0 496 512" height="1em"><path d="M165.9 397.4c0 2-2.3 3.6-5.2 3.6-3.3.3-5.6-1.3-5.6-3.6 0-2 2.3-3.6 5.2-3.6 3-.3 5.6 1.3 5.6 3.6zm-31.1-4.5c-.7 2 1.3 4.3 4.3 4.9 2.6 1 5.6 0 6.2-2s-1.3-4.3-4.3-5.2c-2.6-.7-5.5.3-6.2 2.3zm44.2-1.7c-2.9.7-4.9 2.6-4.6 4.9.3 2 2.9 3.3 5.9 2.6 2.9-.7 4.9-2.6 4.6-4.6-.3-1.9-3-3.2-5.9-2.9zM244.8 8C106.1 8 0 113.3 0 252c0 110.9 69.8 205.8 169.5 239.2 12.8 2.3 17.3-5.6 17.3-12.1 0-6.2-.3-40.4-.3-61.4 0 0-70 15-84.7-29.8 0 0-11.4-29.1-27.8-36.6 0 0-22.9-15.7 1.6-15.4 0 0 24.9 2 38.6 25.8 21.9 38.6 58.6 27.5 72.9 20.9 2.3-16 8.8-27.1 16-33.7-55.9-6.2-112.3-14.3-112.3-110.5 0-27.5 7.6-41.3 23.6-58.9-2.6-6.5-11.1-33.3 2.6-67.9 20.9-6.5 69 27 69 27 20-5.6 41.5-8.5 62.8-8.5s42.8 2.9 62.8 8.5c0 0 48.1-33.6 69-27 13.7 34.7 5.2 61.4 2.6 67.9 16 17.7 25.8 31.5 25.8 58.9 0 96.5-58.9 104.2-114.8 110.5 9.2 7.9 17 22.9 17 46.4 0 33.7-.3 75.4-.3 83.6 0 6.5 4.6 14.4 17.3 12.1C428.2 457.8 496 362.9 496 252 496 113.3 383.5 8 244.8 8zM97.2 352.9c-1.3 1-1 3.3.7 5.2 1.6 1.6 3.9 2.3 5.2 1 1.3-1 1-3.3-.7-5.2-1.6-1.6-3.9-2.3-5.2-1zm-10.8-8.1c-.7 1.3.3 2.9 2.3 3.9 1.6 1 3.6.7 4.3-.7.7-1.3-.3-2.9-2.3-3.9-2-.6-3.6-.3-4.3.7zm32.4 35.6c-1.6 1.3-1 4.3 1.3 6.2 2.3 2.3 5.2 2.6 6.5 1 1.3-1.3.7-4.3-1.3-6.2-2.2-2.3-5.2-2.6-6.5-1zm-11.4-14.7c-1.6 1-1.6 3.6 0 5.9 1.6 2.3 4.3 3.3 5.6 2.3 1.6-1.3 1.6-3.9 0-6.2-1.4-2.3-4-3.3-5.6-2z"></path></svg>
            </span>
            <span class="BG"></span>
          </a>
        </div>
        <button aria-label="Close Settings" on:click={closeSettings} class="w-8 h-8 rounded-lg flex items-center justify-center text-textSecondary hover:text-textPrimary hover:bg-bg transition-all">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>



      <!-- Preview Quality Section -->
      <div class="mb-6">
        <h3 class="text-[13px] font-semibold text-textSecondary uppercase tracking-widest mb-3">Performance</h3>
        <div class="panel-border bg-bg/50 p-4 rounded-lg space-y-4">
          <div>
            <div class="flex justify-between items-center mb-2">
              <div>
                <span class="text-[14px] text-textPrimary font-medium">Preview Quality</span>
                <p class="text-[12px] text-textMuted mt-0.5">Lower values = faster loading, less detail</p>
              </div>
              <span class="text-[13px] text-accent font-mono font-semibold">{$previewQuality}%</span>
            </div>
            <input
              type="range"
              min="10"
              max="100"
              step="5"
              bind:value={$previewQuality}
              class="settings-slider w-full"
            />
            <div class="flex justify-between mt-1">
              <span class="text-[10px] text-textMuted">Fast</span>
              <span class="text-[10px] text-textMuted">High Quality</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Export Settings Section -->
      <div class="mb-6">
        <h3 class="text-[13px] font-semibold text-textSecondary uppercase tracking-widest mb-3">Export Options</h3>
        <div class="panel-border bg-bg/50 p-4 rounded-lg">
          <div class="flex justify-between items-center">
            <div>
              <span class="text-[14px] text-textPrimary font-medium">Format</span>
              <p class="text-[12px] text-textMuted mt-0.5">Container and codec for final export</p>
            </div>
            <div class="relative w-[180px]">
              <select bind:value={$exportFormat} class="w-full bg-surface border border-borderBase rounded-lg px-3 py-2 text-[13px] text-textPrimary focus:border-accent outline-none appearance-none cursor-pointer transition-colors hover:border-borderHover shadow-sm">
                <option value="ORIGINAL">Original Format (Fast Cut - Instant)</option>
                <option value="MP4_H264">MP4 (H.264 Universal - Playable everywhere)</option>
                <option value="MP4_HEVC">MP4 (HEVC/H.265 - High Quality/Low Size)</option>
                <option value="WEBM_VP9">WebM (VP9 - Web Optimized)</option>
              </select>
              <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-textSecondary">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Keyboard Shortcuts Section -->
      <div>
        <h3 class="text-[13px] font-semibold text-textSecondary uppercase tracking-widest mb-3">Keyboard Shortcuts</h3>
        <div class="panel-border bg-bg/50 p-4 rounded-lg">
          <div class="grid grid-cols-2 gap-y-2 gap-x-6 text-[12px]">
            <div class="flex justify-between"><span class="text-textSecondary">Play / Pause</span><kbd class="kbd">Space</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Set In Point</span><kbd class="kbd">I</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Skip Back 5s</span><kbd class="kbd">Shift+←</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Set Out Point</span><kbd class="kbd">O</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Frame Step ←</span><kbd class="kbd">←</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Export</span><kbd class="kbd">Ctrl+E</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Frame Step →</span><kbd class="kbd">→</kbd></div>
            <div class="flex justify-between"><span class="text-textSecondary">Save Project</span><kbd class="kbd">Ctrl+S</kbd></div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .animate-in {
    animation: dialogIn 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }
  @keyframes dialogIn {
    from { opacity: 0; transform: scale(0.96) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .kbd {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 10px;
    color: var(--text-primary);
  }

  .settings-slider {
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    outline: none;
  }
  .settings-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    background: var(--accent);
    border-radius: 50%;
    cursor: ew-resize;
    box-shadow: 0 0 8px rgba(76,141,255,0.4);
    transition: transform 120ms ease;
  }
  .settings-slider::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  .github-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background-color: transparent;
    position: relative;
    border-radius: 7px;
    cursor: pointer;
    transition: all .3s;
    text-decoration: none;
    color: var(--text-primary);
  }

  .github-btn .svgContainer {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    backdrop-filter: blur(0px);
    letter-spacing: 0.8px;
    border-radius: 8px;
    transition: all .3s;
    border: 1px solid var(--border);
  }

  .github-btn .BG {
    position: absolute;
    content: "";
    width: 100%;
    height: 100%;
    background: color-mix(in srgb, var(--text-primary) 5%, transparent);
    z-index: -1;
    border-radius: 8px;
    pointer-events: none;
    transition: all .3s;
  }

  .github-btn:hover .BG {
    transform: rotate(35deg);
    transform-origin: bottom;
  }

  .github-btn:hover .svgContainer {
    background-color: color-mix(in srgb, var(--text-primary) 10%, transparent);
    backdrop-filter: blur(4px);
  }
</style>
