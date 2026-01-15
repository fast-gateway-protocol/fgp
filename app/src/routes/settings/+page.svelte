<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { onMount } from "svelte";

  // Types
  interface AgentInfo {
    name: string;
    id: string;
    installed: boolean;
    config_path: string | null;
    registered: boolean;
  }

  // State
  let agents = $state<AgentInfo[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let actionLoading = $state<string | null>(null);
  let mcpConfig = $state<string | null>(null);
  let copied = $state(false);
  let autostartEnabled = $state(false);
  let autostartLoading = $state(false);

  // Load agents on mount
  onMount(async () => {
    await Promise.all([
      refreshAgents(),
      loadMcpConfig(),
      checkAutostart()
    ]);
  });

  async function checkAutostart() {
    try {
      autostartEnabled = await invoke<boolean>("is_autostart_enabled");
    } catch (e) {
      console.error("Failed to check autostart:", e);
    }
  }

  async function toggleAutostart() {
    autostartLoading = true;
    error = null;
    try {
      if (autostartEnabled) {
        await invoke("disable_autostart");
      } else {
        await invoke("enable_autostart");
      }
      autostartEnabled = !autostartEnabled;
    } catch (e) {
      error = String(e);
      console.error("Failed to toggle autostart:", e);
    } finally {
      autostartLoading = false;
    }
  }

  async function refreshAgents() {
    loading = true;
    error = null;
    try {
      agents = await invoke<AgentInfo[]>("detect_agents");
    } catch (e) {
      error = String(e);
      console.error("Failed to detect agents:", e);
    } finally {
      loading = false;
    }
  }

  async function loadMcpConfig() {
    try {
      mcpConfig = await invoke<string>("get_mcp_config");
    } catch (e) {
      console.error("Failed to get MCP config:", e);
    }
  }

  async function registerAgent(agentId: string) {
    actionLoading = agentId;
    error = null;
    try {
      await invoke("register_mcp", { agentId });
      await refreshAgents();
    } catch (e) {
      error = String(e);
      console.error("Failed to register:", e);
    } finally {
      actionLoading = null;
    }
  }

  async function unregisterAgent(agentId: string) {
    actionLoading = agentId;
    error = null;
    try {
      await invoke("unregister_mcp", { agentId });
      await refreshAgents();
    } catch (e) {
      error = String(e);
      console.error("Failed to unregister:", e);
    } finally {
      actionLoading = null;
    }
  }

  async function copyConfig() {
    if (mcpConfig) {
      try {
        await writeText(mcpConfig);
        copied = true;
        setTimeout(() => (copied = false), 2000);
      } catch (e) {
        // Fallback to navigator.clipboard
        navigator.clipboard.writeText(mcpConfig);
        copied = true;
        setTimeout(() => (copied = false), 2000);
      }
    }
  }

  // Agent icons
  function getAgentIcon(id: string): string {
    const icons: Record<string, string> = {
      "claude-code": "M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z",
      "cursor": "M15.042 21.672L13.684 16.6m0 0l-2.51 2.225.569-9.47 5.227 7.917-3.286-.672zM12 2.25V4.5m5.834.166l-1.591 1.591M20.25 10.5H18M7.757 14.743l-1.59 1.59M6 10.5H3.75m4.007-4.243l-1.59-1.59",
      "claude-desktop": "M9 17.25v1.007a3 3 0 01-.879 2.122L7.5 21h9l-.621-.621A3 3 0 0115 18.257V17.25m6-12V15a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 15V5.25m18 0A2.25 2.25 0 0018.75 3H5.25A2.25 2.25 0 003 5.25m18 0V12a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 12V5.25"
    };
    return icons[id] || icons["claude-code"];
  }
</script>

<div class="settings-container">
  <!-- Header -->
  <header class="sticky top-0 z-10 bg-white/80 dark:bg-gray-900/80 backdrop-blur-lg border-b border-gray-200 dark:border-gray-700">
    <div class="px-6 py-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Settings</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
        Configure FGP integrations
      </p>
    </div>
  </header>

  <main class="p-6 space-y-8">
    <!-- General Settings Section -->
    <section>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">General</h2>

      <div class="space-y-3">
        <!-- Auto-start Toggle -->
        <div class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
          <div class="flex items-center gap-4">
            <div class="p-2.5 rounded-lg bg-purple-100 dark:bg-purple-900/30">
              <svg class="w-5 h-5 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <div>
              <h3 class="font-medium text-gray-900 dark:text-white">Launch at Login</h3>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                Start FGP Manager automatically when you log in
              </p>
            </div>
          </div>

          <button
            onclick={toggleAutostart}
            disabled={autostartLoading}
            class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 {autostartEnabled ? 'bg-blue-500' : 'bg-gray-300 dark:bg-gray-600'}"
            class:opacity-50={autostartLoading}
            aria-label={autostartEnabled ? "Disable auto-start" : "Enable auto-start"}
          >
            {#if autostartLoading}
              <span class="absolute inset-0 flex items-center justify-center">
                <svg class="animate-spin w-4 h-4 text-white" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              </span>
            {:else}
              <span
                class="inline-block h-4 w-4 transform rounded-full bg-white shadow-sm transition-transform {autostartEnabled ? 'translate-x-6' : 'translate-x-1'}"
              ></span>
            {/if}
          </button>
        </div>
      </div>
    </section>

    <!-- AI Agent Integration Section -->
    <section>
      <div class="flex items-center justify-between mb-4">
        <div>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">AI Agent Integration</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
            Connect FGP daemons to your AI coding assistants via MCP
          </p>
        </div>
        <button
          onclick={refreshAgents}
          class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Refresh"
        >
          <svg class="w-5 h-5 text-gray-500 dark:text-gray-400" class:animate-spin={loading} fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>

      {#if error}
        <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-700 dark:text-red-400">{error}</p>
        </div>
      {/if}

      {#if loading && agents.length === 0}
        <div class="flex items-center justify-center h-32">
          <svg class="animate-spin w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
      {:else}
        <div class="space-y-3">
          {#each agents as agent}
            <div class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
              <div class="flex items-center gap-4">
                <div class="p-2.5 rounded-lg {agent.installed ? 'bg-blue-100 dark:bg-blue-900/30' : 'bg-gray-100 dark:bg-gray-700'}">
                  <svg class="w-5 h-5 {agent.installed ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getAgentIcon(agent.id)} />
                  </svg>
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <h3 class="font-medium text-gray-900 dark:text-white">{agent.name}</h3>
                    {#if !agent.installed}
                      <span class="px-1.5 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded">Not Installed</span>
                    {:else if agent.registered}
                      <span class="px-1.5 py-0.5 text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded">Connected</span>
                    {/if}
                  </div>
                  {#if agent.config_path}
                    <p class="text-xs text-gray-400 dark:text-gray-500 mt-0.5 font-mono truncate max-w-xs">
                      {agent.config_path}
                    </p>
                  {/if}
                </div>
              </div>

              <div class="flex items-center gap-2">
                {#if !agent.installed}
                  <span class="text-sm text-gray-400">—</span>
                {:else if actionLoading === agent.id}
                  <svg class="animate-spin w-5 h-5 text-blue-500" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                {:else if agent.registered}
                  <button
                    onclick={() => unregisterAgent(agent.id)}
                    class="px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  >
                    Disconnect
                  </button>
                {:else}
                  <button
                    onclick={() => registerAgent(agent.id)}
                    class="px-4 py-1.5 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
                  >
                    Connect
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Manual Configuration Section -->
    <section>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Manual Configuration</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
        Copy this configuration for other MCP-compatible agents
      </p>

      <div class="relative">
        <pre class="p-4 bg-gray-900 text-gray-100 rounded-xl overflow-x-auto text-sm font-mono">{mcpConfig || 'Loading...'}</pre>
        <button
          onclick={copyConfig}
          class="absolute top-3 right-3 p-2 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors"
          title="Copy to clipboard"
        >
          {#if copied}
            <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          {:else}
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          {/if}
        </button>
      </div>
    </section>

    <!-- Available Tools Section -->
    <section>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Available MCP Tools</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
        Once connected, these tools become available in your AI assistant
      </p>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div class="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <code class="text-sm font-mono text-blue-600 dark:text-blue-400">fgp_list_daemons</code>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">List installed daemons and their status</p>
        </div>
        <div class="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <code class="text-sm font-mono text-blue-600 dark:text-blue-400">fgp_start_daemon</code>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Start a daemon by name</p>
        </div>
        <div class="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <code class="text-sm font-mono text-blue-600 dark:text-blue-400">fgp_browser_*</code>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Browser automation (open, click, fill, screenshot)</p>
        </div>
        <div class="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <code class="text-sm font-mono text-blue-600 dark:text-blue-400">fgp_gmail_*</code>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Email operations (list, read, send, search)</p>
        </div>
        <div class="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <code class="text-sm font-mono text-blue-600 dark:text-blue-400">fgp_github_*</code>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">GitHub operations (issues, PRs, repos)</p>
        </div>
        <div class="p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <code class="text-sm font-mono text-blue-600 dark:text-blue-400">fgp_calendar_*</code>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Calendar operations (list, create, update)</p>
        </div>
      </div>
    </section>

    <!-- Performance Section -->
    <section>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Performance Advantage</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
        FGP daemons eliminate cold-start latency for dramatically faster tool execution
      </p>

      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-gray-200 dark:border-gray-700">
              <th class="text-left py-2 px-3 text-gray-500 dark:text-gray-400 font-medium">Operation</th>
              <th class="text-right py-2 px-3 text-gray-500 dark:text-gray-400 font-medium">MCP Stdio</th>
              <th class="text-right py-2 px-3 text-gray-500 dark:text-gray-400 font-medium">FGP Daemon</th>
              <th class="text-right py-2 px-3 text-gray-500 dark:text-gray-400 font-medium">Speedup</th>
            </tr>
          </thead>
          <tbody class="text-gray-900 dark:text-gray-100">
            <tr class="border-b border-gray-100 dark:border-gray-800">
              <td class="py-2 px-3">Browser navigate</td>
              <td class="py-2 px-3 text-right text-gray-500">2,300ms</td>
              <td class="py-2 px-3 text-right text-green-600 dark:text-green-400">8ms</td>
              <td class="py-2 px-3 text-right font-semibold text-green-600 dark:text-green-400">292x</td>
            </tr>
            <tr class="border-b border-gray-100 dark:border-gray-800">
              <td class="py-2 px-3">Gmail list</td>
              <td class="py-2 px-3 text-right text-gray-500">2,400ms</td>
              <td class="py-2 px-3 text-right text-green-600 dark:text-green-400">35ms</td>
              <td class="py-2 px-3 text-right font-semibold text-green-600 dark:text-green-400">69x</td>
            </tr>
            <tr>
              <td class="py-2 px-3">GitHub issues</td>
              <td class="py-2 px-3 text-right text-gray-500">2,100ms</td>
              <td class="py-2 px-3 text-right text-green-600 dark:text-green-400">28ms</td>
              <td class="py-2 px-3 text-right font-semibold text-green-600 dark:text-green-400">75x</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </main>
</div>

<style>
  .settings-container {
    min-height: 100vh;
    background-color: #f9fafb;
  }

  @media (prefers-color-scheme: dark) {
    .settings-container {
      background-color: #111827;
    }
  }
</style>
