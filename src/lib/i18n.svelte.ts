// The interface's texts, by the OpenGOAL Launcher's library (svelte-i18n) and with its file
// layout: one JSON file per language in src/assets/translations/, named by locale (en-US, fr-FR),
// in ICU message format ({name} for a value; '' for an apostrophe before a brace).

import { addMessages, format, init } from "svelte-i18n";
import { get } from "svelte/store";
import en from "../assets/translations/en-US.json";
import fr from "../assets/translations/fr-FR.json";

export type Key = keyof typeof en;
// every text in French too
const _complete: Record<Key, string> = fr;

/** the interface's languages, by the code the Launcher gives (--lang) */
const LOCALES = { en: "en-US", fr: "fr-FR" } as const;
addMessages(LOCALES.en, en);
addMessages(LOCALES.fr, _complete);
init({ fallbackLocale: LOCALES.en, initialLocale: LOCALES.en });

export const LANGS = { en: "English", fr: "Français" } as const;
export type Lang = keyof typeof LANGS;

export const i18n = $state({
  lang: (navigator.language.startsWith("fr") ? "fr" : "en") as Lang,
});

export function t(key: Key, vars: Record<string, string | number> = {}) {
  // read here, so that a text follows the language as it changes
  const locale = LOCALES[i18n.lang];
  return get(format)(key, { locale, values: vars });
}

/** A text of the core (a game check issue, an export note, an error): its code picks the words in
 * the interface's language; its message, in English, stands in for a code missing here. */
export function coreText(text: {
  code?: string;
  args?: Record<string, string>;
  message?: string;
}) {
  const key = `core_${text.code}`;
  return key in en ? t(key as Key, text.args) : (text.message ?? "");
}

/** What went wrong, from the core or the interface, in the interface's language. */
export function errorText(e: unknown) {
  return e && typeof e === "object" && "message" in e
    ? coreText(e as { message: string })
    : String(e);
}
