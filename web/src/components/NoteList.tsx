import { A } from "@solidjs/router";
import { createEffect, createSignal, For, on, Show } from "solid-js";
import type { NoteMeta } from "~/api/client";
import { useNotes, useStatus } from "~/api/queries";
import { useFilters } from "~/lib/filters";
import { formatDate, noteTitle } from "~/lib/note";
import { type Sort, setSort, sort, sortNotes } from "~/lib/sort";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { TextField, TextFieldInput } from "./ui/text-field";

const SEARCH_DEBOUNCE_MS = 250;

interface SortOption extends Sort {
  label: string;
}

const SORT_OPTIONS: SortOption[] = [
  { key: "updated", desc: true, label: "更新時間：新到舊" },
  { key: "updated", desc: false, label: "更新時間：舊到新" },
  { key: "created", desc: true, label: "建立時間：新到舊" },
  { key: "created", desc: false, label: "建立時間：舊到新" },
  { key: "title", desc: false, label: "標題：升冪" },
  { key: "title", desc: true, label: "標題：降冪" },
];

interface Props {
  selectedId?: string;
  onOpenSidebar?: () => void;
  /** Overrides link navigation, used by the stacked mobile layout. */
  onSelect?: (id: string) => void;
}

export default function NoteList(props: Props) {
  const { filters, update } = useFilters();
  const notes = useNotes(filters);
  const status = useStatus();
  const [query, setQuery] = createSignal(filters().q ?? "");

  createEffect(
    on(
      query,
      (q) => {
        const timer = setTimeout(
          () => update({ q: q || undefined }),
          SEARCH_DEBOUNCE_MS,
        );
        return () => clearTimeout(timer);
      },
      { defer: true },
    ),
  );

  const sorted = () => sortNotes(notes.data ?? [], sort());
  const selectedSort = () =>
    SORT_OPTIONS.find((o) => o.key === sort().key && o.desc === sort().desc);

  return (
    <div class="flex h-full flex-col">
      <div class="flex flex-col gap-2 border-b p-2">
        <div class="flex items-center gap-2">
          <Show when={props.onOpenSidebar}>
            <Button variant="outline" size="icon" onClick={props.onOpenSidebar}>
              ☰
            </Button>
          </Show>
          <TextField class="flex-1" value={query()} onChange={setQuery}>
            <TextFieldInput type="search" placeholder="搜尋標題與內文" />
          </TextField>
        </div>
        <Select
          options={SORT_OPTIONS}
          optionValue={(o) => `${o.key}-${o.desc}`}
          optionTextValue="label"
          value={selectedSort()}
          onChange={(o) => o && setSort({ key: o.key, desc: o.desc })}
          disallowEmptySelection
          itemComponent={(p) => (
            <SelectItem item={p.item}>{p.item.rawValue.label}</SelectItem>
          )}
        >
          <SelectTrigger>
            <SelectValue<SortOption>>
              {(state) => state.selectedOption().label}
            </SelectValue>
          </SelectTrigger>
          <SelectContent />
        </Select>
      </div>

      <ul class="min-h-0 flex-1 overflow-y-auto">
        <For each={status.data?.errors}>
          {(e) => (
            <li class="border-b p-3 text-sm text-destructive">
              <div class="truncate font-medium">{e.path}</div>
              <div class="text-xs">{e.message}</div>
            </li>
          )}
        </For>
        <For each={sorted()}>
          {(note) => (
            <NoteRow
              note={note}
              selected={note.id === props.selectedId}
              onSelect={props.onSelect}
            />
          )}
        </For>
        <Show when={notes.isSuccess && sorted().length === 0}>
          <li class="p-4 text-center text-sm text-muted-foreground">
            沒有符合的筆記
          </li>
        </Show>
      </ul>
    </div>
  );
}

function NoteRow(props: {
  note: NoteMeta;
  selected: boolean;
  onSelect?: (id: string) => void;
}) {
  const href = () => `/n/${props.note.id}${location.search}`;
  return (
    <li class="border-b">
      <A
        href={href()}
        onClick={(e) => {
          if (props.onSelect) {
            e.preventDefault();
            props.onSelect(props.note.id);
          }
        }}
        class="block p-3 hover:bg-accent"
        classList={{ "bg-accent": props.selected }}
      >
        <div class="truncate font-medium">{noteTitle(props.note)}</div>
        <div class="line-clamp-2 text-sm text-muted-foreground">
          <Show when={props.note.snippet} fallback={props.note.summary}>
            {(s) => <Snippet text={s().text} start={s().start} end={s().end} />}
          </Show>
        </div>
        <div class="mt-1 flex flex-wrap items-center gap-1 text-xs text-muted-foreground">
          <span>{props.note.notebook}</span>
          <span>·</span>
          <span>{formatDate(props.note.updated)}</span>
          <For each={props.note.tags}>
            {(tag) => <Badge variant="secondary">{tag}</Badge>}
          </For>
        </div>
      </A>
    </li>
  );
}

function Snippet(props: { text: string; start: number; end: number }) {
  const chars = () => Array.from(props.text);
  const part = (from: number, to: number) => chars().slice(from, to).join("");
  return (
    <>
      {part(0, props.start)}
      <mark class="bg-yellow-200 text-foreground">
        {part(props.start, props.end)}
      </mark>
      {part(props.end, chars().length)}
    </>
  );
}
