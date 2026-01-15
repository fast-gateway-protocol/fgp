<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import "../app.css";

  // Types
  interface DaemonInfo {
    name: string;
    status: string;
    version: string | null;
    uptime_seconds: number | null;
    is_running: boolean;
    has_manifest: boolean;
  }

  // State
  let daemons = $state<DaemonInfo[]>([]);
  let loading = $state(true);
  let loadingDaemon = $state<string | null>(null);

  // Fetch daemons on mount
  onMount(async () => {
    await refreshDaemons();

    // Listen for real-time updates from Rust backend
    const unlisten = await listen<DaemonInfo[]>("daemons-updated", (event) => {
      daemons = event.payload;
    });

    return unlisten;
  });

  // Refresh daemon list
  async function refreshDaemons() {
    loading = true;
    try {
      daemons = await invoke<DaemonInfo[]>("list_daemons");
    } catch (e) {
      console.error("Failed to list daemons:", e);
    } finally {
      loading = false;
    }
  }

  // Toggle daemon state
  async function toggleDaemon(daemon: DaemonInfo) {
    loadingDaemon = daemon.name;
    try {
      if (daemon.is_running) {
        await invoke("stop_daemon", { name: daemon.name });
      } else {
        await invoke("start_daemon", { name: daemon.name });
      }
      await refreshDaemons();
    } catch (e) {
      console.error(`Failed to ${daemon.is_running ? "stop" : "start"} daemon:`, e);
    } finally {
      loadingDaemon = null;
    }
  }

  // Format uptime
  function formatUptime(seconds: number | null): string {
    if (!seconds) return "-";
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
    return `${Math.floor(seconds / 86400)}d`;
  }

  // Get status indicator class
  function getStatusClass(status: string): string {
    switch (status) {
      case "running":
      case "healthy":
        return "bg-green-500 pulse";
      case "stopped":
        return "bg-gray-400";
      case "degraded":
      case "not_responding":
        return "bg-yellow-500 pulse";
      default:
        return "bg-red-500 pulse";
    }
  }

  // Calculate summary
  $effect(() => {
    // This will re-run when daemons changes
  });

  let runningCount = $derived(daemons.filter(d => d.is_running).length);
  let stoppedCount = $derived(daemons.length - runningCount);

  // Open marketplace window
  async function openMarketplace() {
    // Hide popover
    const currentWindow = getCurrentWindow();
    await currentWindow.hide();

    // Check if marketplace window already exists
    const existing = await WebviewWindow.getByLabel("marketplace");
    if (existing) {
      await existing.show();
      await existing.setFocus();
      return;
    }

    // Create new marketplace window
    const marketplace = new WebviewWindow("marketplace", {
      url: "/marketplace",
      title: "FGP Marketplace",
      width: 800,
      height: 600,
      resizable: true,
      center: true,
    });

    marketplace.once("tauri://created", () => {
      console.log("Marketplace window created");
    });

    marketplace.once("tauri://error", (e) => {
      console.error("Failed to create marketplace window:", e);
    });
  }
</script>

<div class="popover-container">
  <!-- Header -->
  <header class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
    <h1 class="text-sm font-semibold text-gray-900 dark:text-gray-100">FGP Manager</h1>
    <div class="flex items-center gap-1">
      <button
        onclick={openMarketplace}
        class="p-1.5 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title="Marketplace"
        aria-label="Open Marketplace"
      >
        <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </button>
      <button
        onclick={refreshDaemons}
        class="p-1.5 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title="Refresh"
        aria-label="Refresh daemon list"
      >
        <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" class:animate-spin={loading} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>
    </div>
  </header>

  <!-- Daemon List -->
  <main class="flex-1 overflow-y-auto px-3 py-2">
    {#if loading && daemons.length === 0}
      <div class="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
        <svg class="animate-spin w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Loading...
      </div>
    {:else if daemons.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400">
        <svg class="w-12 h-12 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 12h14M12 5l7 7-7 7" />
        </svg>
        <p class="text-sm">No daemons installed</p>
        <p class="text-xs mt-1">Check the Marketplace to get started</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each daemons as daemon}
          <div
            class="flex items-center justify-between px-3 py-2.5 bg-white/50 dark:bg-gray-800/50 rounded-lg hover:bg-white/80 dark:hover:bg-gray-800/80 transition-all"
            class:border-l-2={daemon.is_running}
            class:border-green-500={daemon.is_running}
          >
            <div class="flex flex-col gap-0.5">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full {getStatusClass(daemon.status)}"></span>
                <span class="font-medium text-sm text-gray-900 dark:text-gray-100">{daemon.name}</span>
              </div>
              <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                {#if daemon.version}
                  <span>v{daemon.version}</span>
                {/if}
                {#if daemon.uptime_seconds}
                  <span class="font-mono">{formatUptime(daemon.uptime_seconds)}</span>
                {/if}
              </div>
            </div>

            <!-- Toggle Switch -->
            <button
              onclick={() => toggleDaemon(daemon)}
              disabled={loadingDaemon === daemon.name}
              class="toggle-switch {daemon.is_running ? 'enabled' : 'disabled'}"
              class:opacity-50={loadingDaemon === daemon.name}
              aria-label={daemon.is_running ? `Stop ${daemon.name}` : `Start ${daemon.name}`}
            >
              <span class="toggle-switch-knob"></span>
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </main>

  <!-- Footer -->
  <footer class="flex items-center justify-between px-4 py-2.5 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 dark:text-gray-400">
    <span>
      {#if daemons.length > 0}
        {runningCount} running · {stoppedCount} stopped
      {:else}
        No daemons
      {/if}
    </span>
    <button
      onclick={openMarketplace}
      class="flex items-center gap-1 hover:text-blue-500 transition-colors"
    >
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Get more
    </button>
  </footer>
</div>
