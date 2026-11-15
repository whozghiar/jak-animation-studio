<script lang="ts">
  import { help, hideHelp, keepHelp } from "$lib/help.svelte";
  import { t } from "$lib/i18n.svelte";

  const WIDTH = 280;
  let height = $state(0);

  /** under the control, else over it; inside the window */
  const place = $derived.by(() => {
    const s = help.shown;
    if (!s) return "";
    const r = s.rect;
    const left = Math.min(
      Math.max(8, r.left + r.width / 2 - WIDTH / 2),
      window.innerWidth - WIDTH - 8,
    );
    const below = r.bottom + 6 + height < window.innerHeight;
    const top = below ? r.bottom + 6 : r.top - 6 - height;
    return `left:${left}px;top:${Math.max(8, top)}px;width:${WIDTH}px`;
  });
</script>

{#if help.shown}
  {@const s = help.shown}
  <div
    bind:clientHeight={height}
    class="fixed z-50 flex flex-col gap-1.5 rounded-md border border-white/15 bg-[#1f1f1f] px-3 py-2 text-xs shadow-lg"
    style={place}
    role="tooltip"
    data-help
    onmouseenter={keepHelp}
    onmouseleave={() => hideHelp()}
  >
    {#if s.label}
      <span class="font-semibold text-neutral-100">{s.label}</span>
    {/if}
    <span class="text-neutral-300">{t(s.note)}</span>
    <button
      class="self-end text-orange-400 hover:text-orange-300"
      onclick={() => {
        help.open = s.section;
        hideHelp(true);
      }}>{t("seeMore")}</button
    >
  </div>
{/if}
