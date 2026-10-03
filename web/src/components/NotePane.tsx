import { A, useNavigate } from "@solidjs/router";
import { useQueryClient } from "@tanstack/solid-query";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { toast } from "solid-sonner";
import { api, type Note, unwrap } from "~/api/client";
import {
  useAllNotes,
  useBacklinks,
  useNote,
  useNotebooks,
  useRefreshNotes,
} from "~/api/queries";
import { createExtensions } from "~/editor/extensions";
import { EditorSession, type SaveStatus } from "~/editor/session";
import { lookupChanged } from "~/editor/wikilink";
import { noteTitle } from "~/lib/note";
import TagEditor from "./TagEditor";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

interface Props {
  id?: string;
  onBack?: () => void;
}

const STATUS_LABEL: Record<SaveStatus, string> = {
  saved: "已儲存",
  dirty: "尚未儲存",
  saving: "儲存中",
  conflict: "有衝突",
  error: "儲存失敗",
};

export default function NotePane(props: Props) {
  const note = useNote(() => props.id);
  const allNotes = useAllNotes();
  const notebooks = useNotebooks();
  const backlinks = useBacklinks(() => props.id);
  const client = useQueryClient();
  const refresh = useRefreshNotes();
  const navigate = useNavigate();

  const [status, setStatus] = createSignal<SaveStatus>("saved");
  const [tags, setTags] = createSignal<string[]>([]);
  const [backlinkCount, setBacklinkCount] = createSignal<number>();

  const byId = createMemo(
    () => new Map((allNotes.data ?? []).map((n) => [n.id, n])),
  );

  let host!: HTMLDivElement;
  let session: EditorSession | undefined;

  const saved = (saved: Note) => {
    client.setQueryData(["note", saved.meta.id], saved);
    void refresh();
  };

  onMount(() => {
    session = new EditorSession(
      host,
      createExtensions({
        lookup: { get: (id) => byId().get(id), all: () => allNotes.data ?? [] },
        onOpenNote: (id) => navigate(`/n/${id}${location.search}`),
        onError: (message) => toast.error(message),
      }),
      {
        onStatus: setStatus,
        onTags: setTags,
        onSaved: saved,
        onError: (message) => toast.error(message),
      },
    );

    const flushOnLeave = () => void session?.flush({ keepalive: true });
    const onVisibility = () => {
      if (document.visibilityState === "hidden") flushOnLeave();
    };
    document.addEventListener("visibilitychange", onVisibility);
    window.addEventListener("beforeunload", flushOnLeave);
    onCleanup(() => {
      document.removeEventListener("visibilitychange", onVisibility);
      window.removeEventListener("beforeunload", flushOnLeave);
      flushOnLeave();
      session?.destroy();
    });
  });

  createEffect(() => {
    const data = note.data;
    if (data && data.meta.id === props.id) session?.open(data);
  });

  createEffect(() => {
    if (props.id === undefined) session?.close();
  });

  createEffect(() => {
    byId();
    session?.editorView.dispatch({ effects: lookupChanged.of(null) });
  });

  const move = async (notebook: string) => {
    const id = props.id;
    if (!id || notebook === note.data?.meta.notebook) return;
    try {
      const moved = await unwrap(
        api.POST("/api/notes/{id}/move", {
          params: { path: { id } },
          body: { notebook },
        }),
      );
      saved(moved);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const remove = async () => {
    const id = props.id;
    if (!id) return;
    session?.forget(id);
    try {
      await unwrap(api.DELETE("/api/notes/{id}", { params: { path: { id } } }));
      client.removeQueries({ queryKey: ["note", id] });
      await refresh();
      navigate(`/${location.search}`, { replace: true });
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  /** Deleting is recoverable, so only notes that others link to ask first. */
  const requestDelete = async () => {
    const id = props.id;
    if (!id) return;
    try {
      const links = await unwrap(
        api.GET("/api/notes/{id}/backlinks", { params: { path: { id } } }),
      );
      if (links.length > 0) setBacklinkCount(links.length);
      else await remove();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const resolve = async (choice: "discard" | "overwrite") => {
    const id = props.id;
    if (!id || !session) return;
    try {
      if (choice === "discard") await session.discard(id);
      else await session.save(id, { force: true });
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div class="flex h-full flex-col">
      <Show when={props.id}>
        <div class="flex flex-col gap-2 border-b p-2">
          <div class="flex items-center gap-2">
            <Show when={props.onBack}>
              <Button variant="ghost" onClick={props.onBack}>
                ← 返回
              </Button>
            </Show>
            <Select
              options={(notebooks.data ?? []).map((n) => n.name)}
              value={note.data?.meta.notebook}
              onChange={(name) => name && void move(name)}
              disallowEmptySelection
              itemComponent={(p) => (
                <SelectItem item={p.item}>{p.item.rawValue}</SelectItem>
              )}
            >
              <SelectTrigger class="h-8 w-40">
                <SelectValue<string>>
                  {(state) => state.selectedOption()}
                </SelectValue>
              </SelectTrigger>
              <SelectContent />
            </Select>
            <span class="ml-auto text-xs text-muted-foreground">
              {STATUS_LABEL[status()]}
            </span>
            <Button variant="outline" size="sm" onClick={requestDelete}>
              刪除
            </Button>
          </div>
          <TagEditor tags={tags()} onChange={(t) => session?.setTags(t)} />
          <Show when={status() === "conflict"}>
            <div class="flex flex-wrap items-center gap-2 rounded-md border border-destructive p-2 text-sm">
              <span class="flex-1">這則筆記在其他地方被修改過。</span>
              <Button
                size="sm"
                variant="outline"
                onClick={() => resolve("discard")}
              >
                載入新版本（放棄我的修改）
              </Button>
              <Button
                size="sm"
                variant="destructive"
                onClick={() => resolve("overwrite")}
              >
                覆蓋
              </Button>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={props.id === undefined}>
        <p class="p-4 text-muted-foreground">選擇一則筆記</p>
      </Show>
      <Show when={note.isError}>
        <p class="p-4 text-destructive">{note.error?.message}</p>
      </Show>
      <div
        ref={host}
        class="min-h-0 flex-1 overflow-hidden"
        classList={{ hidden: props.id === undefined || note.isError }}
      />

      <Show when={backlinks.data?.length}>
        <details class="border-t p-2 text-sm">
          <summary class="cursor-pointer text-muted-foreground">
            反向連結（{backlinks.data?.length}）
          </summary>
          <ul class="mt-1 flex flex-col">
            <For each={backlinks.data}>
              {(link) => (
                <li>
                  <A
                    href={`/n/${link.id}${location.search}`}
                    class="block truncate rounded px-2 py-1 hover:bg-accent"
                  >
                    {noteTitle(link)}
                    <span class="ml-2 text-xs text-muted-foreground">
                      {link.path}
                    </span>
                  </A>
                </li>
              )}
            </For>
          </ul>
        </details>
      </Show>

      <AlertDialog
        open={backlinkCount() !== undefined}
        onOpenChange={(open) => !open && setBacklinkCount(undefined)}
      >
        <AlertDialogContent>
          <AlertDialogTitle>刪除筆記</AlertDialogTitle>
          <AlertDialogDescription>
            有 {backlinkCount()}{" "}
            則筆記連結到這則筆記，刪除後那些連結會變成失效連結。
            刪除的筆記會移到垃圾桶。
          </AlertDialogDescription>
          <div class="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={() => setBacklinkCount(undefined)}
            >
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                setBacklinkCount(undefined);
                void remove();
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
