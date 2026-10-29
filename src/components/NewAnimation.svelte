<script lang="ts">
  import { onDestroy } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { Spinner } from "flowbite-svelte";
  import IconVideo from "~icons/mdi/video-outline";
  import IconMocap from "~icons/mdi/run";
  import IconHand from "~icons/mdi/gesture-tap-hold";
  import IconClose from "~icons/mdi/close";
  import IconChevron from "~icons/mdi/chevron-down";
  import { autoMap, SLOTS, type Mapping } from "$lib/mapping";
  import { captureMocap, captureVideo, loadMocap, type Mocap } from "$lib/pose";
  import type { Capture } from "$lib/frames";
  import { retarget, type Retarget } from "$lib/retarget";
  import type { EditStart } from "$lib/keys";
  import { setAnimation, type ModelInfo } from "$lib/rpc";
  import type { Viewer } from "$lib/viewer";
  import { errorText, t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  let {
    viewer,
    model,
    onclose,
    onsaved,
    onedit,
  }: {
    viewer: Viewer;
    model: ModelInfo;
    onclose: () => void;
    onsaved: (path: string, name: string) => void;
    /** to the animation editor, with what to start from */
    onedit: (start: EditStart) => void;
  } = $props();

  const slug = (s: string) =>
    s
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");

  let source = $state<"video" | "mocap" | "manual">("video");
  let input = $state<HTMLInputElement>();
  let file = $state.raw<File | null>(null);
  let mocap = $state.raw<Mocap | null>(null);
  let clipIndex = $state(0);
  let name = $state("");
  let fps = $state(30);
  let smoothing = $state(2);
  let inPlace = $state(true);
  let face = $state(true);
  let mapping = $state<Mapping>(autoMap(model.joints));
  let showMapping = $state(false);
  let capture = $state.raw<Capture | null>(null);
  let progress = $state<number | null>(null);
  let controller: AbortController | null = null;
  let result = $state.raw<Retarget | null>(null);
  let saving = $state(false);

  function reset() {
    capture = null;
    result = null;
    viewer.setFigure(null);
  }

  async function pick() {
    const f = input?.files?.[0];
    if (input) input.value = "";
    if (!f) return;
    reset();
    file = f;
    mocap = null;
    name = slug(f.name.replace(/\.[^.]+$/, ""));
    if (source === "mocap") {
      try {
        mocap = await loadMocap(f);
        clipIndex = 0;
      } catch (e) {
        toast(errorText(e), "error");
        file = null;
      }
    }
  }

  async function extract() {
    if (!file) return;
    reset();
    controller = new AbortController();
    progress = 0;
    try {
      capture =
        source === "video"
          ? await captureVideo(
              file,
              fps,
              (p) => (progress = p),
              controller.signal,
            )
          : captureMocap(mocap!, mocap!.clips[clipIndex], fps);
    } catch (e) {
      if ((e as Error).name !== "AbortError") toast(errorText(e), "error");
    } finally {
      progress = null;
      controller = null;
    }
  }

  // the preview follows every change of the options and of the mapping
  $effect(() => {
    const options = { smoothing, inPlace, faceForward: face };
    const map = $state.snapshot(mapping);
    if (!capture) return;
    try {
      const r = retarget(viewer.rig(), map, capture, options);
      result = r;
      viewer.play(r.clip);
      viewer.setFigure(r.frames, capture.fps);
    } catch (e) {
      toast(errorText(e), "error");
    }
  });

  async function save() {
    const anim = slug(name);
    if (!result || !anim) return;
    const taken = model.animations.some((a) => a.name === anim);
    if (
      taken &&
      !(await ask(t("replaceConfirm", { name: anim }), { kind: "warning" }))
    )
      return;
    saving = true;
    try {
      const path = await setAnimation(model.path, anim, result.channels);
      toast(t("savedToCreations"));
      onsaved(path, anim);
    } catch (e) {
      toast(errorText(e), "error");
    } finally {
      saving = false;
    }
  }

  onDestroy(() => {
    controller?.abort();
    viewer.setFigure(null);
  });
</script>

<div class="flex flex-col gap-4">
  <div class="flex items-center justify-between">
    <h2 class="font-title text-lg">{t("newAnimation")}</h2>
    <button
      class="text-xl text-neutral-400 hover:text-neutral-100"
      onclick={onclose}
      aria-label={t("cancel")}><IconClose /></button
    >
  </div>

  <div class="grid grid-cols-3 gap-2">
    {#each [["video", IconVideo, "fromVideo"], ["mocap", IconMocap, "fromMocap"], ["manual", IconHand, "fromManual"]] as const as [id, Icon, label] (id)}
      <button
        class="flex flex-col items-center gap-1 rounded-md border px-3 py-3 text-sm {source ===
        id
          ? 'border-orange-500 bg-orange-500/10'
          : 'border-white/10 hover:bg-white/5'}"
        onclick={() => {
          source = id;
          file = null;
          mocap = null;
          reset();
        }}
      >
        <Icon class="text-2xl" />{t(label)}
      </button>
    {/each}
  </div>
  <p class="text-xs text-neutral-400">
    {t(
      source === "video"
        ? "videoHint"
        : source === "mocap"
          ? "mocapHint"
          : "manualHint",
    )}
  </p>

  {#if source === "manual"}
    <label class="flex items-center justify-between gap-2 text-sm">
      <span class="text-neutral-300">{t("name")}</span>
      <input
        class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 font-mono text-sm focus:border-orange-500 focus:ring-0"
        bind:value={name}
      />
    </label>
    <button
      class="rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500"
      onclick={() => onedit({ name: slug(name), channels: [], duration: 2 })}
      >{t("openEditor")}</button
    >
  {:else}
    <input
      bind:this={input}
      type="file"
      class="hidden"
      accept={source === "video" ? "video/*" : ".bvh,.fbx,.glb,.gltf"}
      onchange={pick}
    />
    <button
      class="truncate rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20"
      onclick={() => input?.click()}>{file?.name ?? t("chooseFile")}</button
    >

    {#if mocap && mocap.clips.length > 1}
      <label class="flex items-center justify-between gap-2 text-sm">
        <span class="text-neutral-300">{t("clip")}</span>
        <select
          class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 text-sm"
          bind:value={clipIndex}
        >
          {#each mocap.clips as clip, i (i)}
            <option value={i}>{clip.name || i}</option>
          {/each}
        </select>
      </label>
    {/if}

    <label class="flex items-center justify-between gap-2 text-sm">
      <span class="text-neutral-300">{t("name")}</span>
      <input
        class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 font-mono text-sm focus:border-orange-500 focus:ring-0"
        bind:value={name}
      />
    </label>
    <label class="flex items-center justify-between gap-2 text-sm">
      <span class="text-neutral-300">{t("fps")}</span>
      <select
        class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 text-sm"
        bind:value={fps}
        disabled={!!capture}
      >
        {#each [15, 24, 30, 60] as f (f)}
          <option value={f}>{f}</option>
        {/each}
      </select>
    </label>

    {#if progress !== null}
      <div class="flex items-center gap-3">
        <Spinner size="5" color="orange" />
        <span class="flex-1 text-xs text-neutral-300"
          >{t("extracting", { p: Math.round(progress * 100) })}</span
        >
        <button
          class="rounded-md bg-white/10 px-3 py-1 text-xs hover:bg-white/20"
          onclick={() => controller?.abort()}>{t("stop")}</button
        >
      </div>
    {:else}
      <button
        class="rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500 disabled:opacity-40"
        disabled={!file || (source === "mocap" && !mocap)}
        onclick={extract}>{t("extract")}</button
      >
    {/if}

    {#if capture}
      <p class="text-xs text-neutral-400">
        {t("frames", {
          n: capture.frames.length,
          s: (capture.frames.length / capture.fps).toFixed(2),
        })}
      </p>
      <label class="flex items-center justify-between gap-2 text-sm">
        <span class="text-neutral-300">{t("smoothing")}</span>
        <input
          type="range"
          class="w-48 accent-orange-500"
          min="0"
          max="6"
          bind:value={smoothing}
        />
      </label>
      <label class="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
          bind:checked={face}
        />
        <span class="text-neutral-300">{t("faceForward")}</span>
      </label>
      {#if capture.absolute}
        <label class="flex items-start gap-2 text-sm">
          <input
            type="checkbox"
            class="mt-1 rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
            bind:checked={inPlace}
          />
          <span class="text-neutral-300"
            >{t("inPlace")}<br /><span class="text-xs text-neutral-500"
              >{t("inPlaceHint")}</span
            ></span
          >
        </label>
      {/if}
    {/if}

    <div class="flex flex-col gap-2 rounded-md border border-white/10">
      <button
        class="flex items-center justify-between px-3 py-2 text-sm text-neutral-200"
        onclick={() => (showMapping = !showMapping)}
      >
        {t("mapping")}
        <IconChevron class={showMapping ? "rotate-180" : ""} />
      </button>
      {#if showMapping}
        <div class="flex flex-col gap-1 px-3 pb-3">
          {#each SLOTS as slot (slot)}
            <label class="flex items-center justify-between gap-2 text-xs">
              <span class="text-neutral-400">{t(`slot_${slot}`)}</span>
              <select
                class="w-44 rounded border border-white/10 bg-[#1b1b1b] px-2 py-0.5 text-xs"
                value={mapping[slot] ?? -1}
                onchange={(e) => {
                  const v = +e.currentTarget.value;
                  if (v < 0) delete mapping[slot];
                  else mapping[slot] = v;
                }}
              >
                <option value={-1}>{t("none")}</option>
                {#each model.joints as joint, i (i)}
                  <option value={i}>{joint.name}</option>
                {/each}
              </select>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    <button
      class="rounded-md bg-white/10 px-4 py-2 text-sm hover:bg-white/20 disabled:opacity-40"
      disabled={!result}
      onclick={() =>
        result &&
        onedit({
          name: slug(name),
          channels: result.channels,
          duration: result.clip.duration,
        })}>{t("refine")}</button
    >
    <button
      class="rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500 disabled:opacity-40"
      disabled={!result || !slug(name) || saving}
      onclick={save}>{t("save")}</button
    >
  {/if}
</div>
