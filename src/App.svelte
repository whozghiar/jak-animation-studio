<script lang="ts">
  import { onMount } from "svelte";
  import Header from "./components/Header.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Studio from "./components/Studio.svelte";
  import SettingsPage from "./components/Settings.svelte";
  import Guide from "./components/Guide.svelte";
  import Toasts from "./components/Toasts.svelte";
  import HelpNote from "./components/HelpNote.svelte";
  import { help } from "$lib/help.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getSettings, takePlay } from "$lib/rpc";
  import { app } from "$lib/settings.svelte";
  import { i18n, LANGS, type Lang } from "$lib/i18n.svelte";

  let page = $state<"studio" | "guide" | "settings">("studio");
  // a control's "See more" opens its section of the guide
  $effect(() => {
    if (help.open) page = "guide";
  });

  async function play() {
    const request = await takePlay();
    if (!request) return;
    page = "studio";
    app.play = request;
  }

  // the window's script errors go to the error log, next to the program's
  const log = (message: string) =>
    invoke("log_window_error", { message }).catch(() => {});
  window.addEventListener("error", (e) =>
    log(`${e.message} (${e.filename}:${e.lineno})`),
  );
  window.addEventListener("unhandledrejection", (e) =>
    log(
      e.reason instanceof Error
        ? (e.reason.stack ?? e.reason.message)
        : (e.reason?.message ?? String(e.reason)),
    ),
  );

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
        <!-- the studio stays: its models, lists and view are there when coming back -->
        <div class="h-full" class:hidden={page !== "studio"}>
          <Studio />
        </div>
        {#if page === "guide"}
          <Guide />
        {:else if page === "settings"}
          <SettingsPage />
        {/if}
      {/if}
    </main>
  </div>
  <Toasts />
  <HelpNote />
</div>
