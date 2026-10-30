import { saveSettings, type PlayRequest, type Settings } from "./rpc";

/** The settings, loaded at start, shared by the pages, what the MCP server asked to play, and the
 * model last opened in the studio (the guide's lines are made for it). */
export const app = $state<{
  settings: Settings | null;
  play: PlayRequest | null;
  model: string | null;
}>({ settings: null, play: null, model: null });

export async function save() {
  if (app.settings) await saveSettings($state.snapshot(app.settings));
}
