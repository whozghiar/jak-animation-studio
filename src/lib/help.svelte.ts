import type { Key } from "./i18n.svelte";

/** The sections of the guide's Using the studio tab, by theme. */
export const THEMES = [
  { id: "find", title: "themeFind", sections: ["browse", "watch"] },
  { id: "animate", title: "themeAnimate", sections: ["anims", "new", "keys"] },
  { id: "looks", title: "themeLooks", sections: ["looks"] },
  { id: "game", title: "themeGame", sections: ["check", "try"] },
  { id: "setup", title: "themeSetup", sections: ["settings", "mcp", "log"] },
] as const;
export type Section = (typeof THEMES)[number]["sections"][number];

/** A control's note: its words (a guide's `ctl_` text), the guide's section that says more. */
export type Help = { note: Key; section: Section; label?: string };

/** The note shown, by the control under the mouse; the guide's section asked for by "See more". */
export const help = $state<{
  shown: (Help & { label: string; rect: DOMRect }) | null;
  open: Section | null;
}>({ shown: null, open: null });

let hiding: ReturnType<typeof setTimeout> | undefined;

/** The note goes after a moment, unless the mouse comes onto it (its link). */
export function hideHelp(now = false) {
  clearTimeout(hiding);
  if (now) help.shown = null;
  else hiding = setTimeout(() => (help.shown = null), 250);
}

export function keepHelp() {
  clearTimeout(hiding);
}

/** `use:helpOn={{ note, section }}`: the control's note shows when the mouse stays on it. Its label
 * is `label`, else the control's aria-label, else its text. */
export function helpOn(node: HTMLElement, value: Help) {
  let h = value;
  let showing: ReturnType<typeof setTimeout> | undefined;
  const enter = () => {
    keepHelp();
    showing = setTimeout(() => {
      help.shown = {
        ...h,
        label:
          h.label ??
          node.getAttribute("aria-label") ??
          node.textContent?.trim() ??
          "",
        rect: node.getBoundingClientRect(),
      };
    }, 450);
  };
  const leave = () => {
    clearTimeout(showing);
    hideHelp();
  };
  node.addEventListener("mouseenter", enter);
  node.addEventListener("mouseleave", leave);
  return {
    update(next: Help) {
      h = next;
    },
    destroy() {
      clearTimeout(showing);
      node.removeEventListener("mouseenter", enter);
      node.removeEventListener("mouseleave", leave);
    },
  };
}
