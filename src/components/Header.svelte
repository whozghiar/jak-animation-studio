<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import IconWindowMinimize from "~icons/mdi/window-minimize";
  import IconWindowMaximize from "~icons/mdi/window-maximize";
  import IconWindowClose from "~icons/mdi/window-close";
  import logo from "../../src-tauri/icons/128x128.png";
  import { app } from "$lib/settings.svelte";
  import { GAMES } from "$lib/rpc";
  import { t } from "$lib/i18n.svelte";
  const appWindow = getCurrentWindow();
  let version = $state("");

  onMount(async () => {
    version = `v${await getVersion()}`;
  });
</script>

<header
  data-tauri-drag-region
  class="flex h-10 shrink-0 items-center gap-4 border-b border-white/5 bg-[#0b0b0b] pl-3 select-none"
>
  <div class="pointer-events-none flex items-center gap-2">
    <img src={logo} alt="" class="h-6 w-6" />
    <span class="font-title text-sm text-neutral-100">Jak Animation Studio</span
    >
  </div>
  <div class="h-6 shrink-0 border-l border-neutral-800"></div>
  <div
    class="pointer-events-none flex items-center gap-2 font-mono text-sm whitespace-nowrap"
  >
    <span class="text-neutral-400">{version}</span>
  </div>
  <div class="h-6 shrink-0 border-l border-neutral-800"></div>
  <div
    class="pointer-events-none flex items-center gap-2 font-mono text-sm whitespace-nowrap"
  >
    <span class="text-neutral-400">{t("game")}</span>
    <span class="text-neutral-500">{GAMES[app.settings?.game ?? "jak1"]}</span>
  </div>
  <div class="ml-auto flex shrink-0 self-stretch">
    <button
      class="flex h-full w-12 items-center justify-center hover:bg-white/10"
      onclick={() => appWindow.minimize()}
      aria-label={t("minimize")}
    >
      <IconWindowMinimize />
    </button>
    <button
      class="flex h-full w-12 items-center justify-center hover:bg-white/10"
      onclick={() => appWindow.toggleMaximize()}
      aria-label={t("maximize")}
    >
      <IconWindowMaximize />
    </button>
    <button
      class="flex h-full w-12 items-center justify-center hover:bg-red-600"
      onclick={() => appWindow.close()}
      aria-label={t("close")}
    >
      <IconWindowClose />
    </button>
  </div>
</header>
