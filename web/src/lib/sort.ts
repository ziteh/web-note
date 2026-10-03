import { createSignal } from "solid-js";
import type { NoteMeta } from "~/api/client";
import { noteTitle } from "./note";

export type SortKey = "updated" | "created" | "title";

export interface Sort {
  key: SortKey;
  desc: boolean;
}

const STORAGE_KEY = "sort";
const DEFAULT_SORT: Sort = { key: "updated", desc: true };

function load(): Sort {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "");
    if (["updated", "created", "title"].includes(saved.key)) {
      return { key: saved.key, desc: Boolean(saved.desc) };
    }
  } catch {
    // Missing or malformed value falls back to the default.
  }
  return DEFAULT_SORT;
}

const [sort, setSortSignal] = createSignal(load());

export { sort };

export function setSort(next: Sort) {
  setSortSignal(next);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
}

function compareCodePoints(a: string, b: string) {
  const left = Array.from(a);
  const right = Array.from(b);
  const len = Math.min(left.length, right.length);
  for (let i = 0; i < len; i++) {
    const diff = (left[i].codePointAt(0) ?? 0) - (right[i].codePointAt(0) ?? 0);
    if (diff !== 0) return diff;
  }
  return left.length - right.length;
}

/** Titles compare as NFC, case-insensitive code point order. */
function titleKey(note: NoteMeta) {
  return noteTitle(note).normalize("NFC").toLowerCase();
}

function time(value: string | null | undefined) {
  return value ? Date.parse(value) : 0;
}

export function sortNotes(notes: NoteMeta[], { key, desc }: Sort) {
  const compare = (a: NoteMeta, b: NoteMeta) => {
    if (key === "title") return compareCodePoints(titleKey(a), titleKey(b));
    return time(a[key]) - time(b[key]);
  };
  return [...notes].sort((a, b) => (desc ? compare(b, a) : compare(a, b)));
}
