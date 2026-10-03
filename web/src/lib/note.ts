import { createSignal } from "solid-js";
import type { NoteMeta } from "~/api/client";
import { dateLocale, t } from "./i18n";

export const DEFAULT_NOTEBOOK = "Inbox";

/** Lets the editor drop local state for a note deleted from the list. */
export const [deletedId, setDeletedId] = createSignal<string>();

export function noteTitle(note: Pick<NoteMeta, "title">) {
  return note.title ?? t("untitled");
}

export function formatDate(value: string | null | undefined) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString(dateLocale(), {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

export function formatDateFull(value: string | null | undefined) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString(dateLocale(), {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
