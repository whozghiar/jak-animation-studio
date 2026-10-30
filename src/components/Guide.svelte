<script lang="ts">
  import { GAMES, gameNames, type GameNames } from "$lib/rpc";
  import { app } from "$lib/settings.svelte";
  import { t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  // the guide's example until a model is opened: Jak 3's Marauder with a wave made in the studio
  const marauder = [
    "idle0",
    "idle1",
    "walk0",
    "run0",
    "run1",
    "jump-start",
    "jump",
    "jump-land",
    "jump-out-car",
    "jump-out-car-land",
    "attack-run0-start",
    "attack-run0",
    "attack-jump0-start",
    "attack-jump0",
    "attack-jump0-land",
    "celebrate0",
    "ragdoll",
    "knocked-back0",
    "knocked-back0-land",
    "knocked-death",
    "knocked-death-land",
    "knocked-forward0",
    "knocked-forward0-land",
    "knocked-forward-death",
    "knocked-forward-death-land",
    "drown-lava",
    "run2",
    "attack-shoot0-start",
    "attack-shoot0-shoot",
    "attack-shoot0-end",
    "turn",
    "die",
    "wave",
  ];
  const EXAMPLE: GameNames = {
    actor: "marauder-male-lod0",
    art: marauder.map((a) => `${a}-ja`),
    slots: marauder.map((a, i) => ({
      slot: i + 2,
      animation: a === "wave" ? a : `marauder-male-${a}`,
      symbol: `marauder-male-lod0-${a}-ja`,
    })),
  };

  let opened = $state<GameNames | null>(null);
  let chosen = $state<string | null>(null);
  $effect(() => {
    const model = app.model;
    if (!model) return;
    gameNames(model)
      .then((n) => (opened = n.slots.length ? n : null))
      .catch(() => (opened = null));
  });

  const names = $derived(opened ?? EXAMPLE);
  const game = $derived(opened ? (app.settings?.game ?? "jak1") : "jak3");
  const actor = $derived(names.actor);
  /** the entity's type: "my-" and the model's first word (my-marauder) */
  const type = $derived(`my-${actor.split("-")[0]}`);
  const idle = $derived(names.slots[0].symbol);
  const play = $derived(
    names.slots.find((s) => s.symbol === chosen) ?? names.slots.at(-1)!,
  );
  /** the event and the state that play it: the animation's short name */
  const event = $derived.by(() => {
    const short = play.symbol.slice(actor.length + 1, -"-ja".length);
    return short === "idle" ? "play" : short;
  });
  const code = $derived({
    model: `custom_assets/${game}/models/custom_levels/${actor}.glb`,
    build: `(build-actor "${actor}")`,
    models: `"custom_models": ["${actor}"],`,
    entity: `;;-*-Lisp-*-
(in-package goal)

;; the art group of the model, its animations in the file's order
(def-actor ${actor}
  :bounds (0 0 0 4)
  :art (${names.art.join(" ")}))

;; the entity: an animated model with two states
(deftype ${type} (process-drawable)
  ()
  (:state-methods
   idle
   ${event}))

;; its first animation again and again; the event '${event} plays the other one
(defstate idle (${type})
  :virtual #t
  :event
    (behavior ((proc process) (argc int) (message symbol) (block event-message-block))
      (case message
        (('${event}) (go-virtual ${event}))))
  :code
    (behavior ()
      (loop
        (ja-no-eval :group! ${idle} :num! (seek!) :frame-num 0.0)
        (until (ja-done? 0)
          (suspend)
          (ja :num! (seek!)))))
  :post ja-post)

;; the animation once, then idle again
(defstate ${event} (${type})
  :virtual #t
  :code
    (behavior ()
      (ja-no-eval :group! ${play.symbol} :num! (seek!) :frame-num 0.0)
      (until (ja-done? 0)
        (suspend)
        (ja :num! (seek!)))
      (go-virtual idle))
  :post ja-post)

;; the level makes one for each of its actors whose etype is ${type}
(defmethod init-from-entity! ((this ${type}) (e entity-actor))
  (set! (-> this root) (new 'process 'trsqv))
  (process-drawable-from-entity! this e)
  (initialize-skeleton this *${actor}-sg* '())
  (logclear! (-> this mask) (process-mask actor-pause))
  (go-virtual idle :proc this)
  (none))`,
    // the engine file the code needs, as for test-zone's own code
    compile: `(goal-src "levels/test-zone/${type}.gc" "${game === "jak1" ? "process-drawable" : "process-focusable"}")`,
    gd: `("TSZ.DGO"
 ("${type}.o"
  ...
  "${actor}-ag.go"
  "test-zone.go"
 ))`,
    actor: `{
  "trans": [0, 3.5, 4],
  "etype": "${type}",
  "game_task": 0,
  "quat": [0, 0, 0, 1],
  "bsphere": [0, 3.5, 4, 10],
  "lump": { "name": "${type}-1" }
}`,
    play: `(mi)
(lt)
(start 'play (get-continue-by-name *game-info* "test-zone-start"))
(send-event (process-by-ename "${type}-1") '${event})`,
    vis: "(set! (-> *pc-settings* ps2-actor-vis?) #f)",
  });

  const steps = $derived([
    { title: "guideModel", text: "guideModelText", code: code.model },
    { title: "guideBuild", text: "guideBuildText", code: code.build },
    { title: "guideMesh", text: "guideMeshText", code: code.models },
    {
      title: "guideCode",
      text: "guideCodeText",
      code: code.entity,
      pick: true,
    },
    { title: "guideCompile", text: "guideCompileText", code: code.compile },
    { title: "guideLoad", text: "guideLoadText", code: code.gd },
    { title: "guidePlace", text: "guidePlaceText", code: code.actor },
    { title: "guidePlay", text: "guidePlayText", code: code.play },
  ] as const);

  const args = $derived({
    game: GAMES[game],
    gameId: game,
    model: actor,
    type,
    anim: play.animation,
    event,
    level: `custom_assets/${game}/levels/test-zone`,
  });

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    toast(t("copied"));
  }
</script>

{#snippet block(text: string)}
  <div class="relative">
    <pre
      class="overflow-x-auto rounded-md border border-white/10 bg-[#1b1b1b] px-4 py-3 font-mono text-xs leading-relaxed text-neutral-200">{text}</pre>
    <button
      class="absolute top-2 right-2 rounded-md bg-white/10 px-3 py-1 text-xs hover:bg-white/20"
      onclick={() => copy(text)}>{t("copy")}</button
    >
  </div>
{/snippet}

<div class="h-full overflow-y-auto p-8">
  <div class="mx-auto flex max-w-3xl flex-col gap-8">
    <h1 class="font-title text-2xl">{t("guideTitle")}</h1>
    <p class="text-sm text-neutral-300">
      {t(opened ? "guideIntro" : "guideIntroExample", args)}
    </p>
    <p class="text-sm text-neutral-400">{t("guideNeed", args)}</p>

    {#each steps as step, i (step.title)}
      <section class="flex flex-col gap-2">
        <h2 class="text-sm font-semibold text-neutral-200">
          {i + 1}. {t(step.title)}
        </h2>
        <p class="text-sm text-neutral-400">{t(step.text, args)}</p>
        {#if "pick" in step}
          <label class="flex items-center gap-2 text-sm text-neutral-300">
            {t("guideEventAnim")}
            <select
              class="rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-1.5 text-sm"
              value={play.symbol}
              onchange={(e) => (chosen = e.currentTarget.value)}
            >
              {#each names.slots.slice(1) as s (s.symbol)}
                <option value={s.symbol}>{s.animation}</option>
              {/each}
            </select>
          </label>
        {/if}
        {@render block(step.code)}
      </section>
    {/each}

    <section
      class="flex flex-col gap-2 rounded-lg border border-orange-500/40 bg-orange-500/10 p-4"
    >
      <h2 class="text-sm font-semibold text-neutral-200">
        {t("guideNotBorn")}
      </h2>
      <p class="text-sm text-neutral-300">{t("guideNotBornText")}</p>
      {@render block(code.vis)}
      {#if game === "jak1"}
        <p class="text-sm text-neutral-300">{t("guideJak1")}</p>
      {/if}
    </section>
  </div>
</div>
