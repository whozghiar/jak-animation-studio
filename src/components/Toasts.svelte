<script lang="ts">
  import { toasts } from "$lib/toast.svelte";
</script>

<div
  class="pointer-events-none fixed right-4 bottom-4 z-50 flex max-w-md flex-col gap-2"
>
  {#each toasts as toast (toast.id)}
    <div
      class="pointer-events-auto rounded-lg border px-4 py-3 text-sm shadow-lg {toast.level ===
      'error'
        ? 'border-red-800 bg-red-950 text-red-100'
        : 'border-white/10 bg-[#222] text-neutral-100'}"
    >
      {toast.text}
    </div>
  {/each}
</div>
