import { saveSettings, type PlayRequest, type Settings } from "./rpc";

/** The settings, loaded at start, shared by the pages, and what the MCP server asked to play. */
export const app = $state<{
  settings: Settings | null;
  play: PlayRequest | null;
}>({ settings: null, play: null });

export async function save() {
  if (app.settings) await saveSettings($state.snapshot(app.settings));
}
