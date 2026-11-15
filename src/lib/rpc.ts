import { invoke } from "@tauri-apps/api/core";

export type Game = "jak1" | "jak2" | "jak3";

export const GAMES: Record<Game, string> = {
  jak1: "Jak and Daxter",
  jak2: "Jak II",
  jak3: "Jak 3",
};

export type Settings = {
  modelsDirs?: Partial<Record<Game, string>>;
  creationsDir?: string | null;
  /** each game's installation: the OpenGOAL Launcher's, an OpenGOAL release or a jak-project */
  gameDirs?: Partial<Record<Game, string>>;
  game?: Game | null;
  lang?: string | null;
};

export type ModelEntry = {
  name: string;
  /** the entity's official name, when the game gives it one */
  title?: { en: string; fr: string | null } | null;
  path: string;
  joints: number;
  animations: number;
};

export type Joint = {
  node: number;
  name: string;
  parent: number | null;
  translation: number[];
  rotation: number[];
  scale: number[];
};

export type AnimationInfo = {
  name: string;
  duration: number;
  keyframes: number;
  /** Jak's: the game has it, it is neither checked nor exported */
  inGame?: boolean;
};

/** Words of the core: `code` and `args` pick the interface's text (i18n's coreText), `message` is
 * the English one. */
export type CoreText = {
  code: string;
  args?: Record<string, string>;
  message: string;
};

export type Issue = CoreText & {
  level: "error" | "warning";
  animation?: string;
};

export type ModelInfo = {
  name: string;
  path: string;
  joints: Joint[];
  animations: AnimationInfo[];
  issues: Issue[];
};

/** Keys of one joint channel, in the joint's parent space: 3 numbers per time, 4 for a rotation. */
export type Channel = {
  joint: number;
  path: "translation" | "rotation" | "scale";
  times: number[];
  values: number[];
};

/** A part of a model's skinned meshes (primitives drawing the same vertices) and its textures. */
export type Part = { mesh: number; primitives: number[]; materials: string[] };
export type Looks = {
  parts: Part[];
  targets: number;
  segments: number[][];
};

/** What the MCP server asked the window to play, the movement to put on the model first, and
 * whether to try it in the game and check what the game plays. */
export type PlayRequest = {
  model: string;
  animation: string | null;
  game: Game;
  at: number;
  mocap?: {
    file: string;
    name: string;
    fps: number;
    inPlace: boolean;
    holdWeapon: boolean;
  };
  check?: boolean;
};

/** What the game of a try shows of an animation (game_test::Check). */
export type Check = {
  animation: string;
  slot: number;
  frames: number;
  screenshots: string[];
};

/** The window's answer to the MCP server: the creation and the joint of each body part of a
 * movement, what the game plays of a check, or what went wrong. */
export type McpAnswer = {
  at: number;
  creation?: string;
  animation?: string;
  duration?: number;
  mapping?: Record<string, string>;
  check?: Check;
  error?: string;
};

/** What the game's code calls a model's art: its art group, the `:art` of its def-actor, each
 * animation's slot and symbol. */
export type GameNames = {
  actor: string;
  art: string[];
  slots: { slot: number; animation: string; symbol: string }[];
};

export type Export = {
  glb: string;
  readme: string;
  notes: CoreText[];
  issues: Issue[];
};

export const getSettings = () => invoke<Settings>("get_settings");
export const saveSettings = (settings: Settings) =>
  invoke<void>("save_settings", { settings });
/** `objects`: the game's objects (props, platforms, vehicles...) too, not only its living entities. */
export const listModels = (dir: string, game: Game, objects: boolean) =>
  invoke<ModelEntry[]>("list_models", { dir, game, objects });
export const inspectModel = (path: string) =>
  invoke<ModelInfo>("inspect_model", { path });
export const readModel = (path: string) =>
  invoke<ArrayBuffer>("read_model", { path });
export const modified = (path: string) =>
  invoke<number | null>("modified", { path });
export const mcpCommand = () => invoke<string>("mcp_command");
/** The studio's folder of settings and logs (errors.log, game-test/). */
export const prefsFolder = () => invoke<string>("prefs_folder");

/** These change the model's creation (copied from the model the first time) and return its path. */
export const setAnimation = (
  model: string,
  name: string,
  channels: Channel[],
) => invoke<string>("set_animation", { model, name, channels });
export const deleteAnimation = (model: string, name: string) =>
  invoke<string>("delete_animation", { model, name });
export const renameAnimation = (model: string, name: string, newName: string) =>
  invoke<string>("rename_animation", { model, name, newName });
/** Copies an entity into the creations of `game` and returns its path there. */
export const importModel = (path: string, game: Game, replace: boolean) =>
  invoke<string>("import_model", { path, game, replace });
export const takePlay = () => invoke<PlayRequest | null>("take_play");
export const answerMcp = (result: McpAnswer) =>
  invoke<void>("answer_mcp", { result });
/** The .png files of the textures the game draws the model's eyes with, when its art group says
 * them: left then right, each iris, pupil, lid. */
/** One eye at one frame (eye.gc): x, y, lid, then the iris, pupil and lid sizes. */
export type EyeFrame = [number, number, number, number, number, number];
/** Each animation's eye frames, by name: per frame, the left eye then the right. */
export const modelEyeFrames = (path: string, game: Game) =>
  invoke<Record<string, [EyeFrame, EyeFrame][]>>("model_eye_frames", {
    path,
    game,
  });
export const modelEyes = (path: string, game: Game) =>
  invoke<[[string, string, string], [string, string, string]] | null>(
    "model_eyes",
    { path, game },
  );
export const modelLooks = (path: string, game: Game) =>
  invoke<Looks>("model_looks", { path, game });
export const findModel = (near: string, game: Game, name: string) =>
  invoke<string | null>("find_model", { near, game, name });
/** Another model with the same skeleton, and the animations it would add. */
export type Twin = { path: string; animations: string[] };
export const twinAnimations = (path: string, game: Game) =>
  invoke<Twin[]>("twin_animations", { path, game });
/** The animations of `sources`, twins of the model, added to its creation. */
export const addAnimations = (model: string, sources: string[]) =>
  invoke<string>("add_animations", { model, sources });
/** The game's textures of the page holding `beside`, one of the model's, then of common. */
export const gameTextures = (game: Game, near: string, beside: string) =>
  invoke<string[]>("game_textures", { game, near, beside });
export const gameNames = (model: string) =>
  invoke<GameNames>("game_names", { model });
/** The README is written in `lang`. */
export const exportForGame = (model: string, lang: string) =>
  invoke<Export>("export_for_game", { model, lang });
/** The model tried in the game: built as a custom actor, shown in the game's test level playing its
 * animation `animation`; the steps come as "game-test" events. */
/** The model tried in the game, playing the animation `animation` again and again. */
export const testInGame = (model: string, game: Game, animation: string) =>
  invoke<void>("test_in_game", { model, game, animation });
/** False when the try did not build that animation (a level holds a few of Jak's). */
export const playInGame = (animation: string) =>
  invoke<boolean>("play_in_game", { animation });
export const checkInGame = (game: Game, animation: string, seconds: number) =>
  invoke<Check>("check_in_game", { game, animation, seconds });
export const stopGameTest = () => invoke<void>("stop_game_test");
/** Why `game` cannot be started for a try now (no installation, another game on its port...). */
export const tryBlocked = (game: Game) =>
  invoke<CoreText | null>("try_blocked", { game });
