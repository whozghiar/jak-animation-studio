<script lang="ts">
  import { onMount } from "svelte";
  import Header from "./components/Header.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Studio from "./components/Studio.svelte";
  import SettingsPage from "./components/Settings.svelte";
  import Guide from "./components/Guide.svelte";
  import Toasts from "./components/Toasts.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getSettings, takePlay } from "$lib/rpc";
  import { app } from "$lib/settings.svelte";
  import { i18n, LANGS, type Lang } from "$lib/i18n.svelte";

  let page = $state<"studio" | "guide" | "settings">("studio");

  async function play() {
    const request = await takePlay();
    if (!request) return;
    page = "studio";
    app.play = request;
  }

  onMount(async () => {
    const settings = await getSettings();
    const lang = settings.lang?.split("-")[0];
    if (lang && lang in LANGS) i18n.lang = lang as Lang;
    if (!Object.keys(settings.modelsDirs ?? {}).length) page = "settings";
    app.settings = settings;
    await listen("play", play);
    await play();
  });
</script>

<div class="flex h-screen flex-col overflow-hidden bg-[#141414]">
  <Header />
  <div class="flex min-h-0 flex-1">
    <Sidebar bind:page />
    <main class="relative min-w-0 flex-1 overflow-hidden">
      {#if app.settings}
        {#if page === "studio"}
          <Studio />
        {:else if page === "guide"}
          <Guide />
        {:else}
          <SettingsPage />
        {/if}
      {/if}
    </main>
  </div>
  <Toasts />
</div>
