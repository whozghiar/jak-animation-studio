<script lang="ts">
  import { Tooltip } from "flowbite-svelte";
  import IconStudio from "~icons/mdi/human-handsup";
  import IconCog from "~icons/mdi/cog";
  import { t } from "$lib/i18n.svelte";

  type Page = "studio" | "settings";
  let { page = $bindable() }: { page: Page } = $props();

  const items = [
    { id: "studio", icon: IconStudio, label: "navStudio" },
    { id: "settings", icon: IconCog, label: "navSettings" },
  ] as const;
</script>

<div
  class="z-10 flex w-18 shrink-0 flex-col border border-white/5 bg-[#0b0b0b]"
>
  <ul class="flex h-full flex-col items-center gap-6 px-2 py-4">
    {#each items as item, i (item.id)}
      <li
        class="relative flex w-full justify-center {i === items.length - 1
          ? 'mt-auto'
          : ''}"
      >
        {#if page === item.id}
          <div
            class="absolute top-1/2 -left-2 h-8 w-0.75 -translate-y-1/2 rounded-r bg-orange-500"
          ></div>
        {/if}
        <button
          id="nav-{item.id}"
          class="relative flex h-12 w-12 items-center justify-center rounded-lg text-3xl transition-all duration-200 select-none {page ===
          item.id
            ? 'text-orange-500 opacity-100'
            : 'text-neutral-300 opacity-45 hover:opacity-80'}"
          onclick={() => (page = item.id)}
          aria-label={t(item.label)}
        >
          <item.icon />
        </button>
        <Tooltip triggeredBy="#nav-{item.id}" placement="right" type="dark">
          {t(item.label)}
        </Tooltip>
      </li>
    {/each}
  </ul>
</div>
