import type { NoteMeta } from "~/api/client";

export const UNTITLED = "未命名";

export function noteTitle(note: Pick<NoteMeta, "title">) {
  return note.title ?? UNTITLED;
}

export function formatDate(value: string | null | undefined) {
  return value ? new Date(value).toLocaleDateString() : "";
}
