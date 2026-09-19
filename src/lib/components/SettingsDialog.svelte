<script lang="ts">
  import { showSettings, previewQuality, exportFormat, exportQuality, autoSnap, enableProxy, isDarkMode } from '$lib/store';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { invoke } from '@tauri-apps/api/core';
  import { GearSix, X, GithubLogo, CaretDown, Check, Palette, FilmStrip, Keyboard, Export, Desktop, HardDrives, Trash, ArrowsClockwise, Lightning, Sun, Moon } from 'phosphor-svelte';

  function closeSettings() {
    $showSettings = false;
  }

  let isDropdownOpen = false;
  let activeTab = 'appearance';

  let proxyStats: { fileCount: number; totalBytes: number } | null = null;
  let isLoadingStats = false;
  let isClearingCache = false;
  let clearSuccessMsg = '';

  async function refreshProxyStats() {
    try {
      isLoadingStats = true;
      proxyStats = await invoke<{ fileCount: number; totalBytes: number }>('get_proxy_stats');
    } catch (e) {
      console.error('Failed to get proxy stats', e);
    } finally {
      isLoadingStats = false;
    }
  }

  async function handleClearCache() {
    try {
      isClearingCache = true;
      const freedBytes = await invoke<number>('clear_proxy_cache');
      await refreshProxyStats();
      clearSuccessMsg = `Freed ${formatBytes(freedBytes)}`;
      setTimeout(() => {
        clearSuccessMsg = '';
      }, 3000);
    } catch (e) {
      console.error('Failed to clear proxy cache', e);
    } finally {
      isClearingCache = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  $: if ($showSettings && activeTab === 'preview') {
    refreshProxyStats();
  }

  const exportOptions = [
    { value: "ORIGINAL", label: "Original Format (Instant Cut)" },
    { value: "MP4_H264", label: "MP4 (H.264 Universal)" },
    { value: "MP4_HEVC", label: "MP4 (HEVC/H.265)" },
    { value: "WEBM_VP9", label: "WebM (VP9 Web Optimized)" }
  ];

  const tabs = [
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'preview', label: 'Preview Engine', icon: Desktop },
    { id: 'export', label: 'Export', icon: Export },
    { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard }
  ];
</script>

{#if $showSettings}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 flex items-center justify-center" style="z-index: var(--z-modal)" on:click|self={closeSettings}>
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-[var(--bg-modal)] backdrop-blur-[12px] transition-all duration-300"></div>

    <!-- Main Panel: 890x560px Landscape with ample width -->
    <div class="relative bg-[var(--bg-deep)] w-[890px] h-[560px] max-w-[95vw] max-h-[92vh] flex border border-[var(--border-strong)] rounded-[var(--radius-2xl)] shadow-[var(--shadow-xl)] animate-in overflow-hidden">
      
      <!-- Left Sidebar (Nav) -->
      <div class="w-[230px] bg-[var(--bg-surface)] border-r border-[var(--border-base)] flex flex-col shrink-0">
        
        <!-- Sidebar Header -->
        <div class="px-6 h-[64px] flex items-center gap-2.5 text-[var(--text-primary)]">
          <GearSix size={18} class="text-[var(--accent)] shrink-0" weight="fill" />
          <h2 class="text-[14px] font-semibold tracking-tight">Preferences</h2>
        </div>

        <!-- Sidebar Navigation -->
        <nav class="flex-1 px-3 py-2 space-y-1">
          {#each tabs as tab}
            <button 
              class="w-full flex items-center gap-3 px-3.5 py-2.5 rounded-[var(--radius-lg)] text-[13px] font-medium transition-all duration-150 whitespace-nowrap {activeTab === tab.id ? 'bg-[var(--accent-dim)] text-[var(--accent)]' : 'text-[var(--text-secondary)] hover:bg-[var(--bg-elevated)] hover:text-[var(--text-primary)]'}"
              on:click={() => activeTab = tab.id}
            >
              <svelte:component this={tab.icon} size={17} weight={activeTab === tab.id ? 'bold' : 'regular'} class="shrink-0" />
              <span class="truncate">{tab.label}</span>
            </button>
          {/each}
        </nav>

        <!-- Sidebar Footer -->
        <div class="p-4 border-t border-[var(--border-base)] flex items-center justify-between">
          <button class="btn-ghost w-full justify-center gap-2 min-h-[34px] px-3 whitespace-nowrap" on:click={() => openUrl('https://github.com/im-ushan-ikshana')}>
            <GithubLogo size={15} weight="bold" class="shrink-0" />
            <span class="whitespace-nowrap">Developer</span>
          </button>
        </div>
      </div>

      <!-- Right Content Area -->
      <div class="flex-1 flex flex-col relative overflow-hidden bg-[var(--bg-deep)]">
        
        <!-- Header (Close button) -->
        <div class="absolute top-4 right-4 z-10">
          <button aria-label="Close Settings" on:click={closeSettings} class="w-8 h-8 rounded-full bg-[var(--bg-elevated)] border border-[var(--border-base)] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)] hover:border-[var(--border-strong)] transition-all">
            <X size={14} weight="bold" />
          </button>
        </div>

        <!-- Content Scroller -->
        <div class="flex-1 overflow-y-auto no-scrollbar px-8 py-8 relative">
          
          <div class="w-full max-w-[560px] transition-all duration-300">
            
            {#if activeTab === 'appearance'}
              <div class="animate-tab space-y-6">
                <div>
                  <div class="label mb-4">Theme Appearance</div>
                  <div class="bg-[var(--bg-elevated)] border border-[var(--border-base)] rounded-[var(--radius-xl)] p-6 space-y-4">
                    <div>
                      <span class="text-[14px] text-[var(--text-primary)] font-medium">Interface Color Theme</span>
                      <p class="text-[12px] text-[var(--text-muted)] mt-1">Choose between Studio Dark (Obsidian & Amber Gold) and Studio Light.</p>
                    </div>

                    <div class="grid grid-cols-2 gap-3 pt-2">
                      <button 
                        type="button"
                        class="flex items-center justify-center gap-2.5 min-h-[46px] px-4 rounded-[var(--radius-lg)] border text-[13px] font-medium transition-all cursor-pointer whitespace-nowrap
                        {!$isDarkMode ? 'bg-[var(--accent-dim)] border-[var(--accent)] text-[var(--accent-bright)] shadow-sm' : 'bg-[var(--bg-surface)] border-[var(--border-base)] text-[var(--text-secondary)] hover:bg-[var(--bg-deep)] hover:text-[var(--text-primary)]'}"
                        on:click={() => $isDarkMode = false}
                      >
                        <Sun size={18} weight={!$isDarkMode ? 'fill' : 'regular'} class="shrink-0 text-[var(--accent)]" />
                        <span class="whitespace-nowrap font-semibold">Studio Light</span>
                      </button>

                      <button 
                        type="button"
                        class="flex items-center justify-center gap-2.5 min-h-[46px] px-4 rounded-[var(--radius-lg)] border text-[13px] font-medium transition-all cursor-pointer whitespace-nowrap
                        {$isDarkMode ? 'bg-[var(--accent-dim)] border-[var(--accent)] text-[var(--accent-bright)] shadow-sm' : 'bg-[var(--bg-surface)] border-[var(--border-base)] text-[var(--text-secondary)] hover:bg-[var(--bg-deep)] hover:text-[var(--text-primary)]'}"
                        on:click={() => $isDarkMode = true}
                      >
                        <Moon size={18} weight={$isDarkMode ? 'fill' : 'regular'} class="shrink-0 text-[var(--accent)]" />
                        <span class="whitespace-nowrap font-semibold">Studio Dark</span>
                      </button>
                    </div>

                    <div class="flex items-center gap-3 p-3 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] mt-3">
                      <Palette size={18} class="text-[var(--accent)] shrink-0" />
                      <span class="text-[12px] text-[var(--text-secondary)]">You can also quickly toggle between themes anytime using the Sun/Moon icon in the top right titlebar.</span>
                    </div>
                  </div>
                </div>
              </div>

            {:else if activeTab === 'preview'}
              <div class="animate-tab space-y-6">
                <div>
                  <div class="label mb-4">Preview Engine</div>
                  <div class="bg-[var(--bg-elevated)] border border-[var(--border-base)] rounded-[var(--radius-xl)] p-6 space-y-6">
                    
                    <!-- Proxy Generation Toggle -->
                    <div class="flex items-center justify-between gap-4">
                      <div class="flex-1 min-w-0">
                        <span class="text-[14px] text-[var(--text-primary)] font-medium">Generate Video Proxies</span>
                        <p class="text-[12px] text-[var(--text-muted)] mt-1">
                          Creates lightweight preview proxies for ultra-smooth scrubbing on heavy or 4K/8K codecs.
                        </p>
                      </div>
                      <label class="toggle shrink-0" class:on={$enableProxy}>
                        <input type="checkbox" bind:checked={$enableProxy} class="hidden">
                        <div class="toggle-thumb"></div>
                      </label>
                    </div>

                    {#if $enableProxy}
                      <div class="pt-4 border-t border-[var(--border-subtle)]">
                        <div class="flex justify-between items-center mb-4 gap-3">
                          <div class="flex-1 min-w-0">
                            <span class="text-[14px] text-[var(--text-primary)] font-medium">Render Quality</span>
                            <p class="text-[12px] text-[var(--text-muted)] mt-1">Lower values load proxies much faster with lower resource usage.</p>
                          </div>
                          <span class="text-[13px] text-[var(--accent)] font-mono font-medium px-2.5 py-1 bg-[var(--accent-dim)] rounded-md min-w-[54px] text-center shrink-0">{$previewQuality}%</span>
                        </div>
                        <input
                          type="range"
                          min="10"
                          max="100"
                          step="5"
                          bind:value={$previewQuality}
                          class="sleek-slider w-full mt-2"
                        />
                        <div class="flex justify-between mt-3 px-1">
                          <span class="text-[10px] font-medium text-[var(--text-muted)] uppercase tracking-wide whitespace-nowrap">Performance</span>
                          <span class="text-[10px] font-medium text-[var(--text-muted)] uppercase tracking-wide whitespace-nowrap">Quality</span>
                        </div>
                      </div>
                    {:else}
                      <div class="p-3.5 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] text-[12.5px] text-[var(--text-secondary)] flex items-start gap-2.5">
                        <Lightning size={16} class="text-[var(--accent)] shrink-0 mt-0.5" weight="fill" />
                        <span>Direct playback enabled. Videos play immediately with zero proxy generation and no temporary file creation.</span>
                      </div>
                    {/if}

                    <hr class="border-[var(--border-subtle)]" />
                    
                    <div class="flex items-center justify-between gap-4">
                      <div class="flex-1 min-w-0">
                        <span class="text-[14px] text-[var(--text-primary)] font-medium">Auto-Snap Playhead</span>
                        <p class="text-[12px] text-[var(--text-muted)] mt-1">Magnetically snap to trim boundaries.</p>
                      </div>
                      <label class="toggle shrink-0" class:on={$autoSnap}>
                        <input type="checkbox" bind:checked={$autoSnap} class="hidden">
                        <div class="toggle-thumb"></div>
                      </label>
                    </div>
                  </div>
                </div>

                <!-- Proxy Cache Management Section -->
                <div>
                  <div class="label mb-4">Temporary Storage & Cache</div>
                  <div class="bg-[var(--bg-elevated)] border border-[var(--border-base)] rounded-[var(--radius-xl)] p-6 space-y-4">
                    <div class="flex items-center justify-between gap-4">
                      <div class="flex-1 min-w-0">
                        <span class="text-[14px] text-[var(--text-primary)] font-medium">Proxy Cache Storage</span>
                        <p class="text-[12px] text-[var(--text-muted)] mt-1">
                          Temporary proxies are auto-evicted after 2 hours and capped at 500 MB.
                        </p>
                      </div>
                      <button 
                        class="btn-ghost text-[12.5px] font-medium h-[34px] px-4 min-w-[130px] shrink-0 whitespace-nowrap gap-2 border border-[var(--border-base)] hover:border-red-500/50 hover:text-red-400 hover:bg-red-500/10 transition-all rounded-[var(--radius-lg)]"
                        on:click={handleClearCache}
                        disabled={isClearingCache || (proxyStats !== null && proxyStats.fileCount === 0)}
                      >
                        <Trash size={15} class="shrink-0" />
                        <span class="whitespace-nowrap">{isClearingCache ? 'Clearing...' : 'Clear Cache'}</span>
                      </button>
                    </div>

                    <div class="flex items-center justify-between p-3.5 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] gap-3">
                      <div class="flex items-center gap-3 flex-1 min-w-0">
                        <HardDrives size={20} class="text-[var(--accent)] shrink-0" />
                        <div class="flex-1 min-w-0">
                          <div class="text-[13px] font-medium text-[var(--text-primary)] truncate">
                            {#if proxyStats}
                              {formatBytes(proxyStats.totalBytes)} ({proxyStats.fileCount} {proxyStats.fileCount === 1 ? 'file' : 'files'})
                            {:else if isLoadingStats}
                              Scanning cache...
                            {:else}
                              0 B (0 files)
                            {/if}
                          </div>
                          <div class="text-[11px] text-[var(--text-muted)] truncate">LRU auto-pruning active &middot; Safe for AppData/Temp</div>
                        </div>
                      </div>

                      <button 
                        class="btn-ghost h-9 w-9 p-0 shrink-0 flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] rounded-[var(--radius-md)]"
                        title="Refresh cache statistics"
                        on:click={refreshProxyStats}
                      >
                        <ArrowsClockwise size={16} class={isLoadingStats ? 'animate-spin' : ''} />
                      </button>
                    </div>

                    {#if clearSuccessMsg}
                      <div class="text-[12px] text-emerald-400 font-medium px-1 flex items-center gap-1.5 animate-in fade-in">
                        <Check size={14} weight="bold" class="shrink-0" />
                        <span>{clearSuccessMsg}</span>
                      </div>
                    {/if}
                  </div>
                </div>
              </div>

            {:else if activeTab === 'export'}
              <div class="animate-tab">
                <div class="label mb-4">Export Configuration</div>
                <div class="bg-[var(--bg-elevated)] border border-[var(--border-base)] rounded-[var(--radius-xl)] p-6 space-y-8">
                  
                  <!-- Format Dropdown -->
                  <div class="flex flex-col gap-3">
                    <div>
                      <span class="text-[14px] text-[var(--text-primary)] font-medium">Format Container</span>
                      <p class="text-[12px] text-[var(--text-muted)] mt-1">Select the output codec. Original format is instant via stream copying.</p>
                    </div>
                    
                    <div class="relative w-full">
                      <button 
                        class="w-full flex items-center justify-between border border-[var(--border-base)] bg-[var(--bg-surface)] min-h-[42px] px-4 py-2 rounded-[var(--radius-lg)] hover:bg-[var(--bg-elevated)] hover:border-[var(--border-strong)] transition-all text-left"
                        on:click={() => isDropdownOpen = !isDropdownOpen}
                      >
                        <span class="text-[13.5px] text-[var(--text-primary)] font-medium truncate pr-3 whitespace-nowrap">
                          {exportOptions.find(o => o.value === $exportFormat)?.label || 'Original Format'}
                        </span>
                        <CaretDown size={15} class={`shrink-0 transition-transform duration-200 text-[var(--text-secondary)] ${isDropdownOpen ? 'rotate-180 text-[var(--accent)]' : ''}`} />
                      </button>
                      
                      {#if isDropdownOpen}
                        <!-- svelte-ignore a11y-click-events-have-key-events -->
                        <!-- svelte-ignore a11y-no-static-element-interactions -->
                        <div class="fixed inset-0" style="z-index: var(--z-popover)" on:click={() => isDropdownOpen = false}></div>
                        <div class="absolute left-0 right-0 mt-2 bg-[var(--bg-popover)] backdrop-blur-xl border border-[var(--border-strong)] rounded-[var(--radius-lg)] shadow-[var(--shadow-lg)] overflow-hidden py-1.5 animate-dropdown" style="z-index: calc(var(--z-popover) + 1)">
                          {#each exportOptions as option}
                            <!-- svelte-ignore a11y-click-events-have-key-events -->
                            <!-- svelte-ignore a11y-no-static-element-interactions -->
                            <div 
                              class="px-4 min-h-[38px] py-2 flex items-center gap-3 cursor-pointer transition-colors
                              {option.value === $exportFormat ? 'bg-[var(--accent-dim)] text-[var(--accent)]' : 'text-[var(--text-secondary)] hover:bg-[rgba(255,255,255,0.05)] hover:text-[var(--text-primary)]'}"
                              on:click={() => { $exportFormat = option.value; isDropdownOpen = false; }}
                            >
                              <div class="w-4 h-4 shrink-0 flex items-center justify-center">
                                {#if option.value === $exportFormat}
                                  <Check size={14} weight="bold" />
                                {/if}
                              </div>
                              <span class="text-[13px] font-medium leading-normal whitespace-nowrap">{option.label}</span>
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>
                  </div>
                  
                  <hr class="border-[var(--border-subtle)]" />
                  
                  <!-- Quality Slider -->
                  <div>
                    <div class="flex justify-between items-center mb-4 gap-3">
                      <div class="flex-1 min-w-0">
                        <span class="text-[14px] text-[var(--text-primary)] font-medium">Re-encoding Quality</span>
                        <p class="text-[12px] text-[var(--text-muted)] mt-1">Only applies if a new format is selected.</p>
                      </div>
                      <span class="text-[13px] text-[var(--accent)] font-mono font-medium px-2.5 py-1 bg-[var(--accent-dim)] rounded-md min-w-[54px] text-center shrink-0">{$exportQuality}%</span>
                    </div>
                    <input
                      type="range"
                      min="1"
                      max="100"
                      step="1"
                      bind:value={$exportQuality}
                      class="sleek-slider w-full mt-2"
                    />
                    <div class="flex justify-between mt-3 px-1">
                      <span class="text-[10px] font-medium text-[var(--text-muted)] uppercase tracking-wide whitespace-nowrap">Compressed</span>
                      <span class="text-[10px] font-medium text-[var(--text-muted)] uppercase tracking-wide whitespace-nowrap">Visually Lossless</span>
                    </div>
                  </div>
                </div>
              </div>

            {:else if activeTab === 'shortcuts'}
              <div class="animate-tab">
                <div class="label mb-4">Keyboard Shortcuts</div>
                <div class="bg-[var(--bg-elevated)] border border-[var(--border-base)] rounded-[var(--radius-xl)] overflow-hidden">
                  <div class="grid grid-cols-1">
                    
                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Play / Pause</span>
                      <kbd class="kbd">Space</kbd>
                    </div>
                    
                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Export Video</span>
                      <kbd class="kbd">Ctrl + E</kbd>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Set In Point</span>
                      <kbd class="kbd">I</kbd>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Set Out Point</span>
                      <kbd class="kbd">O</kbd>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Skip Back 5s</span>
                      <div class="flex gap-1.5"><kbd class="kbd">Shift</kbd><kbd class="kbd">←</kbd></div>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Skip Forward 5s</span>
                      <div class="flex gap-1.5"><kbd class="kbd">Shift</kbd><kbd class="kbd">→</kbd></div>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Fast Forward (4x)</span>
                      <div class="flex gap-1.5"><kbd class="kbd">Hold</kbd><kbd class="kbd">→</kbd></div>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Rewind</span>
                      <div class="flex gap-1.5"><kbd class="kbd">Hold</kbd><kbd class="kbd">←</kbd></div>
                    </div>

                    <div class="flex items-center justify-between p-4 border-b border-[var(--border-subtle)] hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Frame Step</span>
                      <div class="flex gap-1.5"><kbd class="kbd">←</kbd><kbd class="kbd">→</kbd></div>
                    </div>

                    <div class="flex items-center justify-between p-4 hover:bg-[var(--bg-surface)] transition-colors">
                      <span class="text-[13px] text-[var(--text-secondary)] font-medium">Save Project</span>
                      <kbd class="kbd">Ctrl + S</kbd>
                    </div>

                  </div>
                </div>
              </div>
            {/if}

          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .animate-in {
    animation: dialog-in 300ms var(--ease-spring) forwards;
  }
  @keyframes dialog-in {
    from { opacity: 0; transform: scale(0.96) translateY(12px); }
    to   { opacity: 1; transform: scale(1) translateY(0); }
  }

  .animate-tab {
    animation: fade-slide-up 200ms var(--ease-out) forwards;
  }
  @keyframes fade-slide-up {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .animate-dropdown {
    animation: drop-in 150ms var(--ease-out) forwards;
    transform-origin: top;
  }
  @keyframes drop-in {
    from { opacity: 0; transform: translateY(-4px) scaleY(0.96); }
    to   { opacity: 1; transform: translateY(0) scaleY(1); }
  }

  .kbd {
    background: var(--bg-surface);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    padding: 3px 10px;
    min-width: 28px;
    min-height: 25px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-primary);
    box-shadow: 0 1px 1px rgba(0,0,0,0.1), 0 1px 0 inset rgba(255,255,255,0.05);
    letter-spacing: 0.02em;
    white-space: nowrap;
    flex-shrink: 0;
  }

  :global(.light) .kbd {
    background: var(--bg-base);
    box-shadow: 0 2px 0 var(--border-strong);
  }

  /* Custom Toggle Switch */
  .toggle {
    width: 40px;
    height: 22px;
    border-radius: 11px;
    cursor: pointer;
    background: var(--bg-surface);
    border: 1px solid var(--border-strong);
    transition: all 200ms var(--ease-out);
    position: relative;
    box-shadow: 0 1px 4px rgba(0,0,0,0.15) inset;
    flex-shrink: 0;
  }
  
  .toggle.on {
    background: var(--accent);
    border-color: var(--accent-bright);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.1);
  }
  
  .toggle-thumb {
    width: 16px;
    height: 16px;
    border-radius: 8px;
    background: white;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 250ms var(--ease-spring);
    box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  }
  
  .toggle.on .toggle-thumb {
    transform: translateX(18px);
  }
</style>
