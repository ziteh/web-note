import type { NoteMeta } from "~/api/client";

export const DEFAULT_NOTEBOOK = "Inbox";
export const UNTITLED = "未命名";

export function noteTitle(note: Pick<NoteMeta, "title">) {
  return note.title ?? UNTITLED;
}

export function formatDate(value: string | null | undefined) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString("zh-TW", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}
