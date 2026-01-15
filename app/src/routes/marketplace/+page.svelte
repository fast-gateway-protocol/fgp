<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  // Types
  interface RegistryPackage {
    name: string;
    display_name: string;
    version: string;
    description: string;
    icon: string;
    author: string;
    repository: string;
    methods_count: number;
    featured: boolean;
    official: boolean;
    category: string;
    installed: boolean;
    installed_version: string | null;
    update_available: boolean;
  }

  interface Registry {
    schema_version: number;
    updated_at: string;
    packages: RegistryPackage[];
    categories: { id: string; name: string; icon: string }[];
  }

  interface InstallProgress {
    package: string;
    step: string;
    progress: number;
    total: number;
  }

  // State
  let registry = $state<Registry | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedCategory = $state<string | null>(null);
  let installingPackage = $state<string | null>(null);
  let installProgress = $state<InstallProgress | null>(null);

  // Computed
  let filteredPackages = $derived(
    registry?.packages.filter(
      (p) => !selectedCategory || p.category === selectedCategory
    ) ?? []
  );

  let featuredPackages = $derived(
    registry?.packages.filter((p) => p.featured) ?? []
  );

  // Fetch registry on mount
  onMount(async () => {
    await fetchRegistry();

    // Listen for install progress
    const unlistenProgress = await listen<InstallProgress>(
      "install-progress",
      (event) => {
        installProgress = event.payload;
        if (event.payload.progress >= 100) {
          setTimeout(() => {
            installingPackage = null;
            installProgress = null;
            fetchRegistry(); // Refresh
          }, 1000);
        }
      }
    );

    // Listen for daemon updates
    const unlistenDaemons = await listen("daemons-updated", () => {
      fetchRegistry();
    });

    return () => {
      unlistenProgress();
      unlistenDaemons();
    };
  });

  async function fetchRegistry() {
    loading = true;
    error = null;
    try {
      registry = await invoke<Registry>("fetch_registry");
    } catch (e) {
      error = String(e);
      console.error("Failed to fetch registry:", e);
    } finally {
      loading = false;
    }
  }

  async function installPackage(name: string) {
    installingPackage = name;
    try {
      await invoke("install_package", { name });
    } catch (e) {
      console.error("Failed to install package:", e);
      error = String(e);
      installingPackage = null;
      installProgress = null;
    }
  }

  async function uninstallPackage(name: string) {
    if (!confirm(`Are you sure you want to uninstall ${name}?`)) return;
    try {
      await invoke("uninstall_package", { name });
      await fetchRegistry();
    } catch (e) {
      console.error("Failed to uninstall package:", e);
      error = String(e);
    }
  }

  // Icon mapping
  function getIcon(iconName: string): string {
    const icons: Record<string, string> = {
      globe: "M12 21a9.004 9.004 0 0 0 8.716-6.747M12 21a9.004 9.004 0 0 1-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 0 1 7.843 4.582M12 3a8.997 8.997 0 0 0-7.843 4.582m15.686 0A11.953 11.953 0 0 1 12 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0 1 21 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0 1 12 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 0 1 3 12c0-1.605.42-3.113 1.157-4.418",
      mail: "M21.75 6.75v10.5a2.25 2.25 0 0 1-2.25 2.25h-15a2.25 2.25 0 0 1-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0 0 19.5 4.5h-15a2.25 2.25 0 0 0-2.25 2.25m19.5 0v.243a2.25 2.25 0 0 1-1.07 1.916l-7.5 4.615a2.25 2.25 0 0 1-2.36 0L3.32 8.91a2.25 2.25 0 0 1-1.07-1.916V6.75",
      calendar: "M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 0 1 2.25-2.25h13.5A2.25 2.25 0 0 1 21 7.5v11.25m-18 0A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75m-18 0v-7.5A2.25 2.25 0 0 1 5.25 9h13.5A2.25 2.25 0 0 1 21 11.25v7.5",
      code: "M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5",
      cloud: "M2.25 15a4.5 4.5 0 0 0 4.5 4.5H18a3.75 3.75 0 0 0 1.332-7.257 3 3 0 0 0-3.758-3.848 5.25 5.25 0 0 0-10.233 2.33A4.502 4.502 0 0 0 2.25 15Z",
      database: "M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125",
      triangle: "M12 3L2 21h20L12 3z",
    };
    return icons[iconName] || icons.globe;
  }
</script>

<div class="marketplace-container">
  <!-- Header -->
  <header class="sticky top-0 z-10 bg-white/80 dark:bg-gray-900/80 backdrop-blur-lg border-b border-gray-200 dark:border-gray-700">
    <div class="px-6 py-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Marketplace</h1>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Discover and install FGP daemons
          </p>
        </div>
        <button
          onclick={fetchRegistry}
          class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Refresh"
        >
          <svg class="w-5 h-5 text-gray-500 dark:text-gray-400" class:animate-spin={loading} fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>

      <!-- Category Filter -->
      {#if registry?.categories}
        <div class="flex gap-2 mt-4 overflow-x-auto pb-2">
          <button
            onclick={() => (selectedCategory = null)}
            class="px-3 py-1.5 text-sm rounded-full whitespace-nowrap transition-colors {selectedCategory === null
              ? 'bg-blue-500 text-white'
              : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'}"
          >
            All
          </button>
          {#each registry.categories as category}
            <button
              onclick={() => (selectedCategory = category.id)}
              class="px-3 py-1.5 text-sm rounded-full whitespace-nowrap transition-colors {selectedCategory === category.id
                ? 'bg-blue-500 text-white'
                : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'}"
            >
              {category.name}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </header>

  <!-- Content -->
  <main class="p-6">
    {#if loading && !registry}
      <div class="flex items-center justify-center h-64">
        <svg class="animate-spin w-8 h-8 text-blue-500" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
    {:else if error}
      <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p class="text-red-700 dark:text-red-400">{error}</p>
        <button
          onclick={fetchRegistry}
          class="mt-2 text-sm text-red-600 dark:text-red-400 hover:underline"
        >
          Try again
        </button>
      </div>
    {:else if registry}
      <!-- Featured Section -->
      {#if !selectedCategory && featuredPackages.length > 0}
        <section class="mb-8">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Featured</h2>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each featuredPackages as pkg}
              <div class="relative overflow-hidden rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 p-[1px]">
                <div class="h-full rounded-xl bg-white dark:bg-gray-900 p-4">
                  <div class="flex items-start gap-3">
                    <div class="p-2 rounded-lg bg-blue-100 dark:bg-blue-900/30">
                      <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getIcon(pkg.icon)} />
                      </svg>
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <h3 class="font-semibold text-gray-900 dark:text-white truncate">{pkg.display_name}</h3>
                        {#if pkg.official}
                          <span class="px-1.5 py-0.5 text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded">Official</span>
                        {/if}
                      </div>
                      <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">{pkg.description}</p>
                    </div>
                  </div>
                  <div class="flex items-center justify-between mt-4 pt-4 border-t border-gray-100 dark:border-gray-800">
                    <span class="text-xs text-gray-500 dark:text-gray-400">{pkg.methods_count} methods</span>
                    {#if pkg.installed}
                      <span class="px-2 py-1 text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded-full">Installed</span>
                    {:else}
                      <button
                        onclick={() => installPackage(pkg.name)}
                        disabled={installingPackage !== null}
                        class="px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:opacity-50"
                      >
                        Install
                      </button>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- All Packages -->
      <section>
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          {selectedCategory ? registry.categories.find(c => c.id === selectedCategory)?.name : 'All Packages'}
        </h2>
        <div class="space-y-3">
          {#each filteredPackages as pkg}
            <div class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600 transition-colors">
              <div class="flex items-center gap-4">
                <div class="p-2.5 rounded-lg bg-gray-100 dark:bg-gray-700">
                  <svg class="w-5 h-5 text-gray-600 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getIcon(pkg.icon)} />
                  </svg>
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <h3 class="font-medium text-gray-900 dark:text-white">{pkg.display_name}</h3>
                    {#if pkg.official}
                      <span class="px-1.5 py-0.5 text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded">Official</span>
                    {/if}
                    {#if pkg.installed && pkg.update_available}
                      <span class="px-1.5 py-0.5 text-xs bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 rounded">Update</span>
                    {/if}
                  </div>
                  <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{pkg.description}</p>
                  <div class="flex items-center gap-3 mt-1.5 text-xs text-gray-400 dark:text-gray-500">
                    <span>v{pkg.version}</span>
                    <span>{pkg.methods_count} methods</span>
                    <span>{pkg.author}</span>
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2">
                {#if installingPackage === pkg.name}
                  <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
                    <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <span class="max-w-32 truncate">{installProgress?.step || 'Installing...'}</span>
                  </div>
                {:else if pkg.installed}
                  <a
                    href={pkg.repository}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                    title="View on GitHub"
                  >
                    <svg class="w-4 h-4 text-gray-400" fill="currentColor" viewBox="0 0 24 24">
                      <path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" />
                    </svg>
                  </a>
                  <button
                    onclick={() => uninstallPackage(pkg.name)}
                    class="px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  >
                    Uninstall
                  </button>
                {:else}
                  <button
                    onclick={() => installPackage(pkg.name)}
                    disabled={installingPackage !== null}
                    class="px-4 py-1.5 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Install
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  </main>
</div>

<style>
  .marketplace-container {
    min-height: 100vh;
    background-color: #f9fafb;
  }

  @media (prefers-color-scheme: dark) {
    .marketplace-container {
      background-color: #111827;
    }
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
