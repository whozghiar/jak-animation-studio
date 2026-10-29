<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import IconClose from "~icons/mdi/close";
  import IconRotate from "~icons/mdi/rotate-3d-variant";
  import IconMove from "~icons/mdi/cursor-move";
  import IconKey from "~icons/mdi/key-plus";
  import IconKeyRemove from "~icons/mdi/key-remove";
  import IconPrev from "~icons/mdi/skip-previous";
  import IconNext from "~icons/mdi/skip-next";
  import IconUndo from "~icons/mdi/undo";
  import IconRest from "~icons/mdi/human-handsdown";
  import {
    finished,
    snap,
    withKey,
    withoutKey,
    type EditStart,
  } from "$lib/keys";
  import { setAnimation, type Channel, type ModelInfo } from "$lib/rpc";
  import type { Viewer } from "$lib/viewer";
  import { errorText, t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  let {
    viewer,
    model,
    start,
    paused = $bindable(),
    onclose,
    onsaved,
  }: {
    viewer: Viewer;
    model: ModelInfo;
    start: EditStart;
    paused: boolean;
    onclose: () => void;
    onsaved: (path: string, name: string) => void;
  } = $props();

  type Path = "rotation" | "translation";

  // svelte-ignore state_referenced_locally
  let name = $state(start.name);
  // svelte-ignore state_referenced_locally
  let duration = $state(Math.max(+start.duration.toFixed(2), 0.1));
  // svelte-ignore state_referenced_locally
  let channels = $state.raw<Channel[]>(start.channels);
  let history: Channel[][] = [];
  let joint = $state<number | null>(null);
  let mode = $state<"rotate" | "translate">("rotate");
  let saving = $state(false);

  /** the joints' depth in the armature, to indent the list */
  const depth = $derived(
    model.joints.map(function d(j): number {
      return j.parent === null ? 0 : 1 + d(model.joints[j.parent]);
    }),
  );
  const jointKeys = $derived(
    new Set(
      channels
        .filter((c) => c.joint === joint)
        .flatMap((c) => c.times.map((time) => time.toFixed(3))),
    ).size,
  );
  const keyedJoints = $derived(new Set(channels.map((c) => c.joint)).size);

  const length = () =>
    Math.max(duration, ...channels.map((c) => c.times[c.times.length - 1]));

  function refresh(time = viewer.time) {
    viewer.show(viewer.clipOf(channels, length(), name || "edit"), time);
  }

  function change(next: Channel[], time?: number) {
    history.push(channels);
    if (history.length > 200) history.shift();
    channels = next;
    refresh(time);
  }

  function undo() {
    const previous = history.pop();
    if (!previous) return;
    channels = previous;
    refresh();
  }

  /** a key at the time shown with the joint as it is in the view */
  function key(j: number, path: Path) {
    const time = snap(viewer.time);
    const o = viewer.joints[j];
    const value =
      path === "rotation" ? o.quaternion.toArray() : o.position.toArray();
    change(withKey(channels, j, path, time, value), time);
  }

  const path = (): Path => (mode === "rotate" ? "rotation" : "translation");

  function pick(j: number | null) {
    joint = j;
    viewer.select(j, mode);
  }

  function setMode(m: typeof mode) {
    mode = m;
    if (joint !== null) viewer.select(joint, m);
  }

  function removeKey() {
    if (joint === null) return;
    const time = snap(viewer.time);
    change(withoutKey(channels, joint, time), time);
  }

  function rest() {
    if (joint === null) return;
    const time = snap(viewer.time);
    const r = viewer.restOf(joint);
    let next = withKey(channels, joint, "rotation", time, r.rotation);
    if (channels.some((c) => c.joint === joint && c.path === "translation"))
      next = withKey(next, joint, "translation", time, r.translation);
    change(next, time);
  }

  /** to the joint's next or previous key (any joint's when none is picked) */
  function step(direction: 1 | -1) {
    const keys = viewer.keyTimes();
    const list = joint !== null && keys.joint.length ? keys.joint : keys.all;
    const now = viewer.time;
    const time =
      direction > 0
        ? list.find((k) => k > now + 1e-3)
        : list.findLast((k) => k < now - 1e-3);
    if (time === undefined) return;
    paused = true;
    viewer.seek(time);
  }

  function keydown(e: KeyboardEvent) {
    if ((e.target as HTMLElement).closest("input, select, textarea")) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      undo();
    } else if (e.key === "r") setMode("rotate");
    else if (e.key === "t") setMode("translate");
    else if (joint === null) return;
    else if (e.key === "k") key(joint, path());
    else if (e.key === "Delete") removeKey();
  }

  async function save() {
    const anim = name.trim();
    if (!anim || !channels.length) return;
    if (
      anim !== start.name &&
      model.animations.some((a) => a.name === anim) &&
      !(await ask(t("replaceConfirm", { name: anim }), { kind: "warning" }))
    )
      return;
    saving = true;
    try {
      const saved = await setAnimation(
        model.path,
        anim,
        finished(channels, length()),
      );
      toast(t("savedToCreations"));
      onsaved(saved, anim);
    } catch (e) {
      toast(errorText(e), "error");
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    paused = true;
    viewer.setEditing(true);
    viewer.onPick = (j) => pick(j);
    viewer.onJointMoved = (j) => key(j, path());
    refresh(0);
  });

  onDestroy(() => {
    viewer.onPick = undefined;
    viewer.onJointMoved = undefined;
    viewer.setEditing(false);
  });
</script>

<svelte:window onkeydown={keydown} />

<div class="flex flex-col gap-4">
  <div class="flex items-center justify-between">
    <h2 class="font-title text-lg">{t("editor")}</h2>
    <button
      class="text-xl text-neutral-400 hover:text-neutral-100"
      onclick={onclose}
      aria-label={t("cancel")}><IconClose /></button
    >
  </div>
  <p class="text-xs text-neutral-400">{t("editorHint")}</p>

  <label class="flex items-center justify-between gap-2 text-sm">
    <span class="text-neutral-300">{t("name")}</span>
    <input
      class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 font-mono text-sm focus:border-orange-500 focus:ring-0"
      bind:value={name}
    />
  </label>
  <label class="flex items-center justify-between gap-2 text-sm">
    <span class="text-neutral-300">{t("duration")}</span>
    <input
      type="number"
      min="0.1"
      step="0.1"
      class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 text-sm focus:border-orange-500 focus:ring-0"
      bind:value={duration}
      onchange={() => refresh()}
    />
  </label>

  <label class="flex items-center justify-between gap-2 text-sm">
    <span class="text-neutral-300">{t("joint")}</span>
    <select
      class="w-48 rounded-md border border-white/10 bg-[#1b1b1b] px-2 py-1 text-sm"
      value={joint ?? -1}
      onchange={(e) => {
        const v = +e.currentTarget.value;
        pick(v < 0 ? null : v);
      }}
    >
      <option value={-1}>{t("noJoint")}</option>
      {#each model.joints as j, i (i)}
        <option value={i}>{"  ".repeat(depth[i])}{j.name}</option>
      {/each}
    </select>
  </label>

  <div class="grid grid-cols-2 gap-2">
    {#each [["rotate", IconRotate, "rotateTool"], ["translate", IconMove, "moveTool"]] as const as [id, Icon, label] (id)}
      <button
        class="flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm {mode ===
        id
          ? 'border-orange-500 bg-orange-500/10'
          : 'border-white/10 hover:bg-white/5'}"
        onclick={() => setMode(id)}
      >
        <Icon />{t(label)}
      </button>
    {/each}
  </div>

  <div class="flex flex-wrap gap-2">
    {#each [[IconPrev, "prevKey", () => step(-1), false], [IconKey, "addKey", () => joint !== null && key(joint, path()), joint === null], [IconKeyRemove, "deleteKey", removeKey, joint === null], [IconNext, "nextKey", () => step(1), false], [IconRest, "restPose", rest, joint === null], [IconUndo, "undo", undo, false]] as const as [Icon, label, action, disabled] (label)}
      <button
        class="flex items-center justify-center rounded-md bg-white/10 p-2 text-lg hover:bg-white/20 disabled:opacity-30"
        onclick={action}
        {disabled}
        title={t(label)}
        aria-label={t(label)}><Icon /></button
      >
    {/each}
  </div>

  <p class="text-xs text-neutral-400">
    {#if joint !== null}{t("jointKeys", { n: jointKeys })} ·
    {/if}{t("keyedJoints", { n: keyedJoints })}
  </p>

  <button
    class="rounded-md bg-orange-600 px-4 py-2 text-sm font-semibold hover:bg-orange-500 disabled:opacity-40"
    disabled={!name.trim() || !channels.length || saving}
    onclick={save}>{t("save")}</button
  >
</div>
