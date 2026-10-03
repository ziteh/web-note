import { A, useNavigate } from "@solidjs/router";
import { useQueryClient } from "@tanstack/solid-query";
import { createEffect, createSignal, For, on, Show } from "solid-js";
import { toast } from "solid-sonner";
import type { NoteMeta } from "~/api/client";
import { api, unwrap } from "~/api/client";
import {
  useNotebooks,
  useNotes,
  useRefreshNotes,
  useStatus,
} from "~/api/queries";
import { useFilters } from "~/lib/filters";
import {
  DEFAULT_NOTEBOOK,
  formatDate,
  noteTitle,
  setDeletedId,
} from "~/lib/note";
import { type Sort, setSort, sort, sortNotes } from "~/lib/sort";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from "./ui/context-menu";
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
  const navigate = useNavigate();
  const refresh = useRefreshNotes();
  const client = useQueryClient();
  const [pendingDelete, setPendingDelete] = createSignal<{
    id: string;
    backlinks: number;
  }>();
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

  /** New notes start empty, in the current notebook filter if there is one. */
  const create = async () => {
    try {
      const created = await unwrap(
        api.POST("/api/notes", {
          body: {
            notebook: filters().notebook ?? DEFAULT_NOTEBOOK,
            body: "",
            tags: [],
          },
        }),
      );
      await refresh();
      navigate(`/n/${created.meta.id}${location.search}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const dismissDuplicates = async () => {
    try {
      await unwrap(api.POST("/api/system/duplicates/dismiss"));
      await client.invalidateQueries({ queryKey: ["status"] });
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const move = async (id: string, notebook: string) => {
    try {
      const moved = await unwrap(
        api.POST("/api/notes/{id}/move", {
          params: { path: { id } },
          body: { notebook },
        }),
      );
      client.setQueryData(["note", id], moved);
      await refresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const remove = async (id: string) => {
    setDeletedId(id);
    try {
      await unwrap(api.DELETE("/api/notes/{id}", { params: { path: { id } } }));
      client.removeQueries({ queryKey: ["note", id] });
      await refresh();
      if (id === props.selectedId) {
        navigate(`/${location.search}`, { replace: true });
      }
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  /** Deleting is recoverable, so only notes that others link to ask first. */
  const requestDelete = async (id: string) => {
    try {
      const links = await unwrap(
        api.GET("/api/notes/{id}/backlinks", { params: { path: { id } } }),
      );
      if (links.length > 0) setPendingDelete({ id, backlinks: links.length });
      else await remove(id);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

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
          <Button
            variant="outline"
            size="icon"
            aria-label="新增筆記"
            onClick={create}
          >
            +
          </Button>
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
        <Show when={status.data?.duplicates.length}>
          <li class="flex flex-col gap-2 border-b bg-accent p-3 text-sm">
            <div class="font-medium">
              發現重複的筆記 id，已為較新的檔案重新指派
            </div>
            <ul class="text-xs text-muted-foreground">
              <For each={status.data?.duplicates}>
                {(d) => <li class="truncate">{d.path}</li>}
              </For>
            </ul>
            <Button size="sm" variant="outline" onClick={dismissDuplicates}>
              知道了
            </Button>
          </li>
        </Show>
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
              onMove={move}
              onDelete={requestDelete}
            />
          )}
        </For>
        <Show when={notes.isSuccess && sorted().length === 0}>
          <li class="p-4 text-center text-sm text-muted-foreground">
            沒有符合的筆記
          </li>
        </Show>
      </ul>

      <AlertDialog
        open={pendingDelete() !== undefined}
        onOpenChange={(open) => !open && setPendingDelete(undefined)}
      >
        <AlertDialogContent>
          <AlertDialogTitle>刪除筆記</AlertDialogTitle>
          <AlertDialogDescription>
            有 {pendingDelete()?.backlinks}{" "}
            則筆記連結到這則筆記，刪除後那些連結會變成失效連結。
            刪除的筆記會移到垃圾桶。
          </AlertDialogDescription>
          <div class="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={() => setPendingDelete(undefined)}
            >
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                const id = pendingDelete()?.id;
                setPendingDelete(undefined);
                if (id) void remove(id);
              }}
            >
              刪除
            </Button>
          </div>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function NoteRow(props: {
  note: NoteMeta;
  selected: boolean;
  onSelect?: (id: string) => void;
  onMove: (id: string, notebook: string) => void;
  onDelete: (id: string) => void;
}) {
  const notebooks = useNotebooks();
  const href = () => `/n/${props.note.id}${location.search}`;
  return (
    <ContextMenu>
      <ContextMenuTrigger as="li" class="border-b">
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
              {(s) => (
                <Snippet text={s().text} start={s().start} end={s().end} />
              )}
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
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuSub>
          <ContextMenuSubTrigger>移動到筆記本</ContextMenuSubTrigger>
          <ContextMenuSubContent>
            <For each={notebooks.data}>
              {(n) => (
                <ContextMenuItem
                  disabled={n.name === props.note.notebook}
                  onSelect={() => props.onMove(props.note.id, n.name)}
                >
                  {n.name}
                </ContextMenuItem>
              )}
            </For>
          </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuItem
          class="text-destructive"
          onSelect={() => props.onDelete(props.note.id)}
        >
          刪除
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
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
