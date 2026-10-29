<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { GAMES, mcpCommand, type Game } from "$lib/rpc";
  import { app, save } from "$lib/settings.svelte";
  import { i18n, LANGS, t, type Lang } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  type Folder = "creationsDir" | "jakProjectDir";
  const settings = app.settings!;
  let command = $state("");

  onMount(async () => {
    command = await mcpCommand();
  });

  async function choose(key: Folder) {
    const dir = await open({
      directory: true,
      defaultPath: settings[key] ?? undefined,
    });
    if (typeof dir !== "string") return;
    settings[key] = dir;
    await save();
  }

  async function clear(key: Folder) {
    settings[key] = null;
    await save();
  }

  async function chooseModels(game: Game) {
    const dir = await open({
      directory: true,
      defaultPath: settings.modelsDirs?.[game],
    });
    if (typeof dir !== "string") return;
    settings.modelsDirs = { ...settings.modelsDirs, [game]: dir };
    await save();
  }

  async function clearModels(game: Game) {
    const { [game]: _, ...rest } = settings.modelsDirs ?? {};
    settings.modelsDirs = rest;
    await save();
  }

  async function setLang(lang: Lang) {
    i18n.lang = lang;
    settings.lang = lang;
    await save();
  }

  const folders: {
    key: Folder;
    label: Parameters<typeof t>[0];
    hint: Parameters<typeof t>[0];
    optional?: boolean;
  }[] = [
    { key: "creationsDir", label: "creationsDir", hint: "creationsDirHint" },
    {
      key: "jakProjectDir",
      label: "jakProjectDir",
      hint: "jakProjectDirHint",
      optional: true,
    },
  ];
</script>

<div class="h-full overflow-y-auto p-8">
  <div class="mx-auto flex max-w-3xl flex-col gap-8">
    <h1 class="font-title text-2xl">{t("navSettings")}</h1>
    {#if !Object.keys(settings.modelsDirs ?? {}).length}
      <p
        class="rounded-lg border border-orange-500/40 bg-orange-500/10 px-4 py-3 text-sm"
      >
        {t("welcome")}
      </p>
    {/if}

    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-semibold text-neutral-200">{t("modelsDirs")}</h2>
      {#each Object.entries(GAMES) as [id, label] (id)}
        {@const dir = settings.modelsDirs?.[id as Game]}
        <div class="flex items-center gap-2">
          <span class="w-32 shrink-0 text-sm text-neutral-300">{label}</span>
          <div
            class="min-w-0 flex-1 truncate rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-2 font-mono text-sm {dir
              ? 'text-neutral-200'
              : 'text-neutral-500'}"
          >
            {dir ?? t("notSet")}
          </div>
          <button
            class="rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500"
            onclick={() => chooseModels(id as Game)}>{t("choose")}</button
          >
          {#if dir}
            <button
              class="rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20"
              onclick={() => clearModels(id as Game)}>{t("clear")}</button
            >
          {/if}
        </div>
      {/each}
      <p class="text-xs text-neutral-400">{t("modelsDirHint")}</p>
    </section>

    {#each folders as f (f.key)}
      <section class="flex flex-col gap-2">
        <h2 class="text-sm font-semibold text-neutral-200">{t(f.label)}</h2>
        <div class="flex items-center gap-2">
          <div
            class="min-w-0 flex-1 truncate rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-2 font-mono text-sm {settings[
              f.key
            ]
              ? 'text-neutral-200'
              : 'text-neutral-500'}"
          >
            {settings[f.key] ?? t("notSet")}
          </div>
          <button
            class="rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500"
            onclick={() => choose(f.key)}>{t("choose")}</button
          >
          {#if f.optional && settings[f.key]}
            <button
              class="rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20"
              onclick={() => clear(f.key)}>{t("clear")}</button
            >
          {/if}
        </div>
        <p class="text-xs text-neutral-400">{t(f.hint)}</p>
      </section>
    {/each}

    <div class="flex gap-8">
      <section class="flex flex-col gap-2">
        <h2 class="text-sm font-semibold text-neutral-200">{t("language")}</h2>
        <select
          class="rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-2 text-sm"
          value={i18n.lang}
          onchange={(e) => setLang(e.currentTarget.value as Lang)}
        >
          {#each Object.entries(LANGS) as [code, label] (code)}
            <option value={code}>{label}</option>
          {/each}
        </select>
      </section>
    </div>

    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-semibold text-neutral-200">{t("mcpTitle")}</h2>
      <p class="text-xs text-neutral-400">{t("mcpHint")}</p>
      <div class="flex items-center gap-2">
        <code
          class="min-w-0 flex-1 truncate rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-2 font-mono text-xs text-neutral-200"
          >{command}</code
        >
        <button
          class="rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20"
          onclick={async () => {
            await navigator.clipboard.writeText(command);
            toast(t("copied"));
          }}>{t("copy")}</button
        >
      </div>
    </section>
  </div>
</div>
