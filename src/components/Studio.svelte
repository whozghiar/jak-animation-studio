<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { ask, open as pickFiles } from "@tauri-apps/plugin-dialog";
  import { join } from "@tauri-apps/api/path";
  import { listen } from "@tauri-apps/api/event";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { Spinner } from "flowbite-svelte";
  import IconPlay from "~icons/mdi/play";
  import IconPause from "~icons/mdi/pause";
  import IconBone from "~icons/mdi/bone";
  import IconFolder from "~icons/mdi/folder-open-outline";
  import IconPlus from "~icons/mdi/plus";
  import IconImport from "~icons/mdi/tray-arrow-down";
  import IconChevron from "~icons/mdi/chevron-right";
  import IconPencil from "~icons/mdi/pencil-outline";
  import IconDelete from "~icons/mdi/delete-outline";
  import IconExport from "~icons/mdi/export-variant";
  import IconGame from "~icons/mdi/gamepad-variant";
  import IconError from "~icons/mdi/close-circle";
  import IconWarning from "~icons/mdi/alert";
  import IconOk from "~icons/mdi/check-circle";
  import NewAnimation from "./NewAnimation.svelte";
  import KeyEditor from "./KeyEditor.svelte";
  import Looks from "./Looks.svelte";
  import IconEditKeys from "~icons/mdi/movie-edit-outline";
  import { Viewer, type Keys } from "$lib/viewer";
  import type { EditStart } from "$lib/keys";
  import { variantTags, type Tag } from "$lib/variants";
  import {
    addAnimations,
    deleteAnimation,
    exportForGame,
    GAMES,
    importModel,
    inspectModel,
    listModels,
    modified,
    playInGame,
    readModel,
    renameAnimation,
    stopGameTest,
    testInGame,
    twinAnimations,
    type Export,
    type Game,
    type ModelEntry,
    type ModelInfo,
    type PlayRequest,
    type Twin,
  } from "$lib/rpc";
  import { app, save } from "$lib/settings.svelte";
  import { coreText, errorText, i18n, t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  const settings = app.settings!;
  const game = $derived<Game>(settings.game ?? "jak1");
  let tab = $state<"models" | "creations">("models");
  let models = $state<ModelEntry[]>([]);
  let creations = $state<ModelEntry[]>([]);
  let filter = $state("");
  let current = $state<ModelInfo | null>(null);
  let loading = $state(false);
  /** other files with this model's skeleton and animations it lacks */
  let twins = $state.raw<Twin[]>([]);
  let playing = $state<string | null>(null);
  let time = $state(0);
  let duration = $state(0);
  let paused = $state(false);
  let speed = $state(1);
  let skeleton = $state(false);
  let creating = $state(false);
  let editing = $state.raw<EditStart | null>(null);
  let keys = $state.raw<Keys>({ all: [], joint: [] });
  let renaming = $state<string | null>(null);
  let newName = $state("");
  let exported = $state<Export | null>(null);
  let canvas: HTMLCanvasElement;
  let viewer: Viewer;
  let ready = $state(false);
  let lastModified = 0;
  let poll: ReturnType<typeof setInterval>;

  /** the official name in the interface's language, else the file's */
  const label = (m: ModelEntry) =>
    (i18n.lang === "fr" && m.title?.fr) || m.title?.en || m.name;
  const shown = $derived(
    (tab === "models" ? models : creations)
      .filter((m) =>
        `${label(m)} ${m.name}`.toLowerCase().includes(filter.toLowerCase()),
      )
      .sort((a, b) => label(a).localeCompare(label(b), i18n.lang)),
  );
  const entry = $derived(
    [...models, ...creations].find((m) => m.path === current?.path),
  );
  /** the entities shown, the variants of one (its official name) together */
  const groups = $derived.by(() => {
    const by = new Map<string, ModelEntry[]>();
    for (const m of shown) {
      const key = m.title?.en ?? `file:${m.name}`;
      by.set(key, [...(by.get(key) ?? []), m]);
    }
    return [...by].map(([key, entries]) => {
      // the main one first: its name is the shortest
      entries.sort(
        (a, b) => a.name.length - b.name.length || a.name.localeCompare(b.name),
      );
      return { key, entries, tags: variantTags(entries.map((m) => m.name)) };
    });
  });
  const currentTags = $derived(
    groups
      .filter((g) => g.entries.length > 1)
      .flatMap((g) =>
        g.entries.map((m, i) => (m.path === current?.path ? g.tags[i] : [])),
      )
      .find((tags) => tags.length),
  );
  let expanded = $state<Record<string, boolean>>({});
  /** a group opens by itself on the model shown, and while searching */
  const isOpen = (g: (typeof groups)[number]) =>
    expanded[g.key] ??
    (!!filter || g.entries.some((m) => m.path === current?.path));
  const tagText = (tag: Tag) =>
    "kind" in tag ? t(`tag_${tag.kind}`) : tag.text;
  const isCreation = $derived(
    !!current &&
      !!settings.creationsDir &&
      current.path.startsWith(settings.creationsDir),
  );
  const errors = $derived(
    current?.issues.filter((i) => i.level === "error").length ?? 0,
  );

  async function refresh() {
    const [g, m, c] = [
      game,
      settings.modelsDirs?.[game],
      settings.creationsDir,
    ];
    const found = [
      m ? await listModels(m, g, false) : [],
      c ? await listModels(await join(c, g), g, true) : [],
    ];
    // a list of the game left meanwhile would land over the new game's
    if (g === game) [models, creations] = found;
  }

  $effect(() => {
    refresh();
  });

  async function setGame(g: Game) {
    if (g === game) return;
    settings.game = g;
    current = null;
    creating = false;
    editing = null;
    exported = null;
    playing = null;
    viewer.unload();
    await save();
  }

  /** what the MCP server asked to play */
  async function show(request: PlayRequest) {
    app.play = null;
    await setGame(request.game);
    await refresh();
    tab = creations.some((c) => c.path === request.model)
      ? "creations"
      : "models";
    await open(request.model, request.animation);
  }

  $effect(() => {
    const request = app.play;
    if (request && ready) untrack(() => show(request));
  });

  /** the name of the creation a file makes, as rules.rs's goal_name gives it */
  const goalName = (file: string) =>
    file
      .split(/[\\/]/)
      .pop()!
      .replace(/\.glb$/i, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");

  async function importModels() {
    const files = await pickFiles({
      multiple: true,
      filters: [{ name: "glTF", extensions: ["glb"] }],
    });
    let last: string | null = null;
    for (const file of files ?? []) {
      const name = goalName(file);
      const replace = creations.some((c) => c.name === name);
      if (
        replace &&
        !(await ask(t("importReplace", { name }), { kind: "warning" }))
      )
        continue;
      try {
        last = await importModel(file, game, replace);
      } catch (e) {
        toast(errorText(e), "error");
      }
    }
    if (last) await edited(last);
  }

  $effect(() => {
    const state = { speed, paused, skeleton };
    if (viewer) Object.assign(viewer, state);
  });

  function play(name: string | null) {
    viewer.play(viewer.clips.find((c) => c.name === name) ?? null);
    playing = viewer.duration ? name : null;
    paused = false;
  }

  async function open(path: string, anim?: string | null) {
    loading = true;
    creating = false;
    editing = null;
    exported = null;
    twins = [];
    try {
      const info = await inspectModel(path);
      await viewer.load(await readModel(path), info.joints);
      lastModified = (await modified(path)) ?? 0;
      current = info;
      const keep = anim && info.animations.some((a) => a.name === anim);
      play(keep ? anim : (info.animations[0]?.name ?? null));
      twinAnimations(path, game).then((found) => {
        if (current?.path === path) twins = found;
      });
    } catch (e) {
      toast(errorText(e), "error");
    } finally {
      loading = false;
    }
  }

  /** after a change: the creation it went to is shown */
  async function edited(path: string, anim?: string | null) {
    await refresh();
    tab = "creations";
    await open(path, anim);
  }

  async function rename(name: string) {
    const to = newName.trim();
    renaming = null;
    if (!current || !to || to === name) return;
    try {
      await edited(await renameAnimation(current.path, name, to), to);
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  async function remove(name: string) {
    if (
      !current ||
      !(await ask(t("deleteConfirm", { name }), { kind: "warning" }))
    )
      return;
    try {
      await edited(await deleteAnimation(current.path, name), playing);
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  /** the step of the try in the game going on, the model the game shows */
  type Step = "export" | "compiler" | "build" | "start" | "load" | "play";
  let trying = $state<Step | null>(null);
  let inGame = $state<string | null>(null);
  const animationIndex = () =>
    Math.max(0, current?.animations.findIndex((a) => a.name === playing) ?? 0);

  async function tryInGame() {
    if (!current || trying) return;
    trying = "export";
    try {
      await testInGame(current.path, game, animationIndex());
      inGame = current.path;
    } catch (e) {
      inGame = null;
      toast(errorText(e), "error");
    } finally {
      trying = null;
    }
  }

  async function closeGame() {
    inGame = null;
    await stopGameTest();
  }

  // the animation picked plays in the game too
  $effect(() => {
    const anim = playing;
    if (!anim || untrack(() => trying || inGame !== current?.path)) return;
    playInGame(untrack(animationIndex)).catch((e) => {
      inGame = null;
      toast(errorText(e), "error");
    });
  });

  async function exportGame() {
    if (!current) return;
    try {
      exported = await exportForGame(current.path, i18n.lang);
      await refresh();
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  onMount(() => {
    listen<Step>("game-test", (e) => {
      if (trying) trying = e.payload;
    });
    viewer = new Viewer(canvas);
    viewer.onTime = (t, d) => {
      time = t;
      duration = d;
    };
    viewer.onKeys = (k) => (keys = k);
    ready = true;
    // the MCP server edits the files: what it changes shows here
    poll = setInterval(async () => {
      if (!current || creating || editing || loading) return;
      const m = await modified(current.path);
      if (m && m !== lastModified) {
        await refresh();
        await open(current.path, playing);
      }
    }, 1500);
  });

  onDestroy(() => {
    clearInterval(poll);
    viewer?.dispose();
  });
</script>

<div class="flex h-full">
  <aside
    class="flex w-72 shrink-0 flex-col border-r border-white/5 bg-[#101010]"
  >
    <div class="flex gap-1 border-b border-white/5 p-2">
      {#each Object.entries(GAMES) as [id, label] (id)}
        <button
          class="flex-auto rounded-md px-2 py-1.5 text-xs whitespace-nowrap {game ===
          id
            ? 'bg-orange-500/15 text-white'
            : 'text-neutral-400 hover:bg-white/5 hover:text-neutral-200'}"
          title={label}
          onclick={() => setGame(id as Game)}>{label}</button
        >
      {/each}
    </div>
    <div class="flex border-b border-white/5 text-sm">
      {#each ["models", "creations"] as const as id (id)}
        <button
          class="flex-1 py-3 {tab === id
            ? 'border-b-2 border-orange-500 text-white'
            : 'text-neutral-400 hover:text-neutral-200'}"
          onclick={() => (tab = id)}
        >
          {t(id === "models" ? "tabModels" : "tabCreations")}
          <span class="text-xs text-neutral-500"
            >{(id === "models" ? models : creations).length}</span
          >
        </button>
      {/each}
    </div>
    <div class="m-3 flex gap-2">
      <input
        class="min-w-0 flex-1 rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-1.5 text-sm placeholder:text-neutral-500 focus:border-orange-500 focus:ring-0"
        placeholder={t("search")}
        bind:value={filter}
      />
      <button
        class="rounded-md bg-white/10 px-2.5 text-lg hover:bg-white/20"
        onclick={importModels}
        aria-label={t("import")}
        title={t("import")}><IconImport /></button
      >
    </div>
    <ul class="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
      {#each groups as g (g.key)}
        {#if g.entries.length === 1}
          {@const m = g.entries[0]}
          <li>
            <button
              class="flex w-full flex-col rounded-md px-3 py-2 text-left {current?.path ===
              m.path
                ? 'bg-orange-500/15 text-white'
                : 'text-neutral-300 hover:bg-white/5'}"
              onclick={() => open(m.path)}
            >
              <span class="truncate text-sm">{label(m)}</span>
              <span class="truncate text-xs text-neutral-500"
                >{m.title ? `${m.name} · ` : ""}{t("joints", {
                  n: m.joints,
                })} · {t("anims", { n: m.animations })}</span
              >
            </button>
          </li>
        {:else}
          <li>
            <button
              class="flex w-full items-center gap-1 rounded-md py-2 pr-3 pl-1 text-left text-neutral-300 hover:bg-white/5"
              onclick={() => (expanded[g.key] = !isOpen(g))}
              aria-expanded={isOpen(g)}
            >
              <IconChevron
                class="shrink-0 text-neutral-500 transition-transform {isOpen(g)
                  ? 'rotate-90'
                  : ''}"
              />
              <span class="flex min-w-0 flex-col">
                <span class="truncate text-sm">{label(g.entries[0])}</span>
                <span class="text-xs text-neutral-500"
                  >{t("variants", { n: g.entries.length })}</span
                >
              </span>
            </button>
            {#if isOpen(g)}
              <ul class="mb-1 ml-3 flex flex-col border-l border-white/10 pl-2">
                {#each g.entries as m, i (m.path)}
                  <li>
                    <button
                      class="flex w-full flex-col gap-1 rounded-md px-2 py-1.5 text-left {current?.path ===
                      m.path
                        ? 'bg-orange-500/15 text-white'
                        : 'text-neutral-300 hover:bg-white/5'}"
                      onclick={() => open(m.path)}
                    >
                      <span class="flex flex-wrap gap-1">
                        {#each g.tags[i] as tag, k (k)}
                          <span
                            class="rounded px-1.5 py-0.5 text-[11px] {'kind' in
                            tag
                              ? 'bg-orange-500/20 text-orange-200'
                              : 'bg-white/10 font-mono text-neutral-300'}"
                            >{tagText(tag)}</span
                          >
                        {/each}
                      </span>
                      <span class="truncate text-xs text-neutral-500"
                        >{m.name} · {t("joints", { n: m.joints })} · {t(
                          "anims",
                          { n: m.animations },
                        )}</span
                      >
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </li>
        {/if}
      {:else}
        <li class="px-3 py-4 text-xs text-neutral-500">
          {#if tab === "models"}
            {settings.modelsDirs?.[game]
              ? t("noModels")
              : t("noModelsDir", { game: GAMES[game] })}
          {:else}
            {t("noCreations")}
          {/if}
        </li>
      {/each}
    </ul>
  </aside>

  <section class="flex min-w-0 flex-1 flex-col">
    <div class="relative min-h-0 flex-1">
      <canvas bind:this={canvas} class="absolute inset-0 h-full w-full"
      ></canvas>
      {#if !current && !loading}
        <p
          class="pointer-events-none absolute inset-0 flex items-center justify-center px-8 text-center text-neutral-500"
        >
          {t("pickModel")}
        </p>
      {/if}
      {#if loading}
        <div
          class="absolute inset-0 flex items-center justify-center bg-black/30"
        >
          <Spinner color="orange" />
        </div>
      {/if}
    </div>
    <div
      class="flex items-center gap-3 border-t border-white/5 bg-[#0b0b0b] px-4 py-2"
    >
      <button
        class="text-2xl text-neutral-200 hover:text-orange-500 disabled:opacity-30"
        disabled={!duration}
        onclick={() => (paused = !paused)}
        aria-label={paused ? t("play") : t("pause")}
      >
        {#if paused}<IconPlay />{:else}<IconPause />{/if}
      </button>
      <div class="relative flex flex-1 flex-col justify-center">
        <!-- the keyframes: every key time, and the picked joint's ones, over the time bar -->
        <div class="pointer-events-none absolute inset-x-2 -top-2 h-3">
          {#each keys.all as k (k)}
            <span
              class="absolute top-1 h-2 w-0.5 -translate-x-1/2 bg-neutral-300"
              style="left: {(k / (duration || 1)) * 100}%"
            ></span>
          {/each}
          {#each keys.joint as k (k)}
            <span
              class="absolute top-0 h-3 w-1.5 -translate-x-1/2 rounded-sm bg-orange-400"
              style="left: {(k / (duration || 1)) * 100}%"
            ></span>
          {/each}
        </div>
        <input
          type="range"
          class="w-full accent-orange-500"
          min="0"
          max={duration || 1}
          step="0.001"
          value={time}
          disabled={!duration}
          oninput={(e) => {
            paused = true;
            viewer.seek(+e.currentTarget.value);
          }}
          aria-label={t("keyframes", { n: keys.all.length })}
          title={t("keyframes", { n: keys.all.length })}
        />
      </div>
      <span class="w-32 text-right font-mono text-xs text-neutral-400"
        >{time.toFixed(2)} / {duration.toFixed(2)} s</span
      >
      <select
        class="rounded-md border border-white/10 bg-[#1b1b1b] py-1 pr-7 pl-2 text-xs"
        bind:value={speed}
        aria-label={t("speed")}
      >
        {#each [0.25, 0.5, 1, 2] as s (s)}
          <option value={s}>{s}x</option>
        {/each}
      </select>
      <button
        class="text-xl {skeleton
          ? 'text-orange-500'
          : 'text-neutral-400 hover:text-neutral-200'}"
        onclick={() => (skeleton = !skeleton)}
        aria-label={t("skeleton")}
        title={t("skeleton")}
      >
        <IconBone />
      </button>
    </div>
  </section>

  <aside
    class="w-96 shrink-0 overflow-y-auto border-l border-white/5 bg-[#101010] p-4"
  >
    {#if current && editing}
      <KeyEditor
        {viewer}
        model={current}
        start={editing}
        bind:paused
        onclose={() => {
          editing = null;
          play(playing);
        }}
        onsaved={(path, name) => edited(path, name)}
      />
    {:else if current && creating}
      <NewAnimation
        {viewer}
        model={current}
        onclose={() => {
          creating = false;
          play(playing);
        }}
        onsaved={(path, name) => edited(path, name)}
        onedit={(start) => {
          creating = false;
          editing = start;
        }}
      />
    {:else if current}
      <div class="flex flex-col gap-5">
        <div class="flex items-start justify-between gap-2">
          <div class="min-w-0">
            <h2 class="font-title truncate text-lg">
              {entry ? label(entry) : current.name}
            </h2>
            {#if currentTags}
              <p class="my-1 flex flex-wrap gap-1">
                {#each currentTags as tag, k (k)}
                  <span
                    class="rounded px-1.5 py-0.5 text-[11px] {'kind' in tag
                      ? 'bg-orange-500/20 text-orange-200'
                      : 'bg-white/10 font-mono text-neutral-300'}"
                    >{tagText(tag)}</span
                  >
                {/each}
              </p>
            {/if}
            {#if entry?.title}
              <p class="truncate font-mono text-xs text-neutral-500">
                {current.name}
              </p>
            {/if}
            <p class="text-xs text-neutral-400">
              {t("joints", { n: current.joints.length })} · {t("anims", {
                n: current.animations.length,
              })}
            </p>
          </div>
          <span
            class="shrink-0 rounded px-2 py-0.5 text-xs {isCreation
              ? 'bg-orange-500/20 text-orange-300'
              : 'bg-white/10 text-neutral-300'}"
            >{isCreation ? t("creation") : t("original")}</span
          >
        </div>
        <button
          class="flex items-center gap-2 self-start text-xs text-neutral-400 hover:text-neutral-200"
          onclick={() => revealItemInDir(current!.path)}
        >
          <IconFolder />{t("openFolder")}
        </button>

        <section class="flex flex-col gap-2">
          <h3 class="text-sm font-semibold text-neutral-200">
            {t("animations")}
          </h3>
          <ul class="flex max-h-80 flex-col gap-1 overflow-y-auto">
            {#each current.animations as anim (anim.name)}
              <li
                class="group flex items-center gap-2 rounded-md px-2 py-1.5 {playing ===
                anim.name
                  ? 'bg-orange-500/15'
                  : 'hover:bg-white/5'}"
              >
                {#if renaming === anim.name}
                  <!-- svelte-ignore a11y_autofocus -->
                  <input
                    class="min-w-0 flex-1 rounded border border-orange-500 bg-[#1b1b1b] px-2 py-0.5 text-sm focus:ring-0"
                    bind:value={newName}
                    autofocus
                    onkeydown={(e) => {
                      if (e.key === "Enter") rename(anim.name);
                      if (e.key === "Escape") renaming = null;
                    }}
                    onblur={() => rename(anim.name)}
                  />
                {:else}
                  <button
                    class="min-w-0 flex-1 truncate text-left text-sm"
                    onclick={() => play(anim.name)}>{anim.name}</button
                  >
                  <span class="font-mono text-xs text-neutral-500"
                    >{anim.duration.toFixed(2)} s</span
                  >
                  <button
                    class="text-neutral-500 opacity-0 group-hover:opacity-100 hover:text-neutral-200"
                    onclick={() => {
                      const clip = viewer.clips.find(
                        (c) => c.name === anim.name,
                      );
                      if (!clip) return;
                      creating = false;
                      editing = {
                        name: anim.name,
                        channels: viewer.channelsOf(clip),
                        duration: clip.duration,
                      };
                    }}
                    aria-label={t("editKeys")}
                    title={t("editKeys")}><IconEditKeys /></button
                  >
                  <button
                    class="text-neutral-500 opacity-0 group-hover:opacity-100 hover:text-neutral-200"
                    onclick={() => {
                      renaming = anim.name;
                      newName = anim.name;
                    }}
                    aria-label={t("rename")}
                    title={t("rename")}><IconPencil /></button
                  >
                  <button
                    class="text-neutral-500 opacity-0 group-hover:opacity-100 hover:text-red-400"
                    onclick={() => remove(anim.name)}
                    aria-label={t("delete")}
                    title={t("delete")}><IconDelete /></button
                  >
                {/if}
              </li>
            {:else}
              <li class="text-xs text-neutral-500">{t("noAnimation")}</li>
            {/each}
          </ul>
          {#if twins.length}
            <button
              class="flex items-center justify-center gap-2 rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20"
              title={t("twinAnimationsHint", {
                files: twins.map((w) => w.path.split(/[\\/]/).pop()).join(", "),
              })}
              onclick={async () => {
                try {
                  await edited(
                    await addAnimations(
                      current!.path,
                      twins.map((w) => w.path),
                    ),
                    playing,
                  );
                } catch (e) {
                  toast(errorText(e), "error");
                }
              }}
            >
              <IconPlus />{t("twinAnimations", {
                n: twins.reduce((n, w) => n + w.animations.length, 0),
              })}
            </button>
          {/if}
          <button
            class="flex items-center justify-center gap-2 rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500"
            onclick={() => (creating = true)}
          >
            <IconPlus />{t("newAnimation")}
          </button>
        </section>

        {#key current}
          <Looks
            {viewer}
            model={current}
            {game}
            onapplied={(path) => edited(path, playing)}
          />
        {/key}

        <section class="flex flex-col gap-2">
          <h3 class="text-sm font-semibold text-neutral-200">
            {t("gameCheck")}
          </h3>
          {#if errors === 0}
            <p class="flex items-center gap-2 text-xs text-green-400">
              <IconOk />{t("allGood")}
            </p>
          {/if}
          <ul class="flex flex-col gap-2">
            {#each current.issues as issue, i (i)}
              <li class="flex gap-2 text-xs">
                <span
                  class="mt-0.5 shrink-0 {issue.level === 'error'
                    ? 'text-red-400'
                    : 'text-amber-400'}"
                >
                  {#if issue.level === "error"}<IconError />{:else}<IconWarning
                    />{/if}
                </span>
                <span class="text-neutral-300">
                  {#if issue.animation}<b class="text-neutral-100"
                      >{issue.animation}:</b
                    >
                  {/if}{coreText(issue)}
                </span>
              </li>
            {/each}
          </ul>
          <button
            class="mt-1 flex items-center justify-center gap-2 rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20"
            onclick={exportGame}
          >
            <IconExport />{t("exportGame", { game: GAMES[game] })}
          </button>
          <button
            class="flex items-center justify-center gap-2 rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20 disabled:opacity-60"
            title={t("tryInGameHint", { game: GAMES[game] })}
            disabled={!!trying}
            onclick={tryInGame}
          >
            {#if trying}
              <Spinner size="4" color="orange" />{t(`tryStep_${trying}`)}
            {:else}
              <IconGame />{t("tryInGame")}
            {/if}
          </button>
          {#if inGame === current.path && !trying}
            <div
              class="flex items-center justify-between gap-2 rounded-md border border-white/10 bg-[#1b1b1b] p-3 text-xs"
            >
              <span class="text-neutral-300">{t("inGame")}</span>
              <button
                class="shrink-0 text-orange-400 hover:text-orange-300"
                onclick={closeGame}>{t("closeGame")}</button
              >
            </div>
          {/if}
          {#if exported}
            <div
              class="flex flex-col gap-2 rounded-md border border-white/10 bg-[#1b1b1b] p-3 text-xs"
            >
              <p class="break-all text-neutral-300">
                {t("exportedTo", { path: exported.glb })}
              </p>
              {#each exported.notes as note (note.message)}
                <p class="text-neutral-400">{coreText(note)}</p>
              {/each}
              {#each exported.issues.filter((i) => i.level === "error") as issue, i (i)}
                <p class="text-red-400">{coreText(issue)}</p>
              {/each}
              <div class="flex gap-3">
                <button
                  class="text-orange-400 hover:text-orange-300"
                  onclick={() => openPath(exported!.readme)}
                  >{t("openReadme")}</button
                >
                <button
                  class="text-orange-400 hover:text-orange-300"
                  onclick={() => revealItemInDir(exported!.glb)}
                  >{t("openFolder")}</button
                >
              </div>
            </div>
          {/if}
        </section>
      </div>
    {/if}
  </aside>
</div>
