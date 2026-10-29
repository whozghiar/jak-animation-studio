export type Toast = { id: number; text: string; level: "info" | "error" };

let next = 0;
export const toasts = $state<Toast[]>([]);

export function toast(text: string, level: Toast["level"] = "info") {
  const id = next++;
  toasts.push({ id, text, level });
  setTimeout(
    () => {
      const i = toasts.findIndex((t) => t.id === id);
      if (i >= 0) toasts.splice(i, 1);
    },
    level === "error" ? 8000 : 4000,
  );
}
