import { A, useNavigate } from "@solidjs/router";
import { useQueryClient } from "@tanstack/solid-query";
import {
  type Component,
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { Dynamic } from "solid-js/web";
import { toast } from "solid-sonner";
import { api, type Note, unwrap } from "~/api/client";
import {
  useAllNotes,
  useBacklinks,
  useNote,
  useRefreshNotes,
} from "~/api/queries";
import CheckIcon from "~/assets/check.svg";
import InfoIcon from "~/assets/info.svg";
import MagicWandIcon from "~/assets/magic-wand.svg";
import UpdateIcon from "~/assets/update.svg";
import WarningIcon from "~/assets/warning.svg";
import XIcon from "~/assets/x.svg";
import { createExtensions } from "~/editor/extensions";
import { EditorSession, type SaveStatus } from "~/editor/session";
import { lookupChanged } from "~/editor/wikilink";
import { type MessageKey, t } from "~/lib/i18n";
import { deletedId, formatDateFull, noteTitle } from "~/lib/note";
import TagEditor from "./TagEditor";
import { Button } from "./ui/button";

interface Props {
  id?: string;
  onBack?: () => void;
}

const STATUS_ICON: Record<SaveStatus, Component> = {
  saved: CheckIcon,
  dirty: UpdateIcon,
  saving: UpdateIcon,
  conflict: WarningIcon,
  error: XIcon,
};

const STATUS_LABEL: Record<SaveStatus, MessageKey> = {
  saved: "statusSaved",
  dirty: "statusDirty",
  saving: "statusSaving",
  conflict: "statusConflict",
  error: "statusError",
};

export default function NotePane(props: Props) {
  const note = useNote(() => props.id);
  const allNotes = useAllNotes();
  const backlinks = useBacklinks(() => props.id);
  const client = useQueryClient();
  const refresh = useRefreshNotes();
  const navigate = useNavigate();

  const [status, setStatus] = createSignal<SaveStatus>("saved");
  const [showInfo, setShowInfo] = createSignal(false);
  const [tags, setTags] = createSignal<string[]>([]);

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

  createEffect(() => {
    const id = deletedId();
    if (id) session?.forget(id);
  });

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

  const formatNote = async () => {
    if (!session) return;
    const body = session.editorView.state.doc.toString();
    try {
      const res = await unwrap(api.POST("/api/format", { body: { body } }));
      // Typing during the request would be overwritten by a stale result.
      if (session.editorView.state.doc.toString() === body) {
        session.replaceDoc(res.body);
      }
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
                {t("back")}
              </Button>
            </Show>
            <div class="min-w-0 flex-1">
              <TagEditor tags={tags()} onChange={(t) => session?.setTags(t)} />
            </div>
            <span
              class="inline-flex size-4 shrink-0 text-muted-foreground [&>svg]:size-full"
              classList={{
                "animate-spin": status() === "saving",
                "text-destructive":
                  status() === "conflict" || status() === "error",
              }}
              role="img"
              aria-label={t(STATUS_LABEL[status()])}
              title={t(STATUS_LABEL[status()])}
            >
              <Dynamic component={STATUS_ICON[status()]} />
            </span>
            <Button
              variant="ghost"
              size="icon"
              class="size-6 shrink-0 [&>svg]:size-4"
              aria-label={t("formatNote")}
              title={t("formatNote")}
              onClick={formatNote}
            >
              <MagicWandIcon />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="size-6 shrink-0 [&>svg]:size-4"
              classList={{ "bg-accent": showInfo() }}
              aria-label={t("noteInfo")}
              title={t("noteInfo")}
              onClick={() => setShowInfo(!showInfo())}
            >
              <InfoIcon />
            </Button>
          </div>
          <Show when={showInfo()}>
            <div class="flex flex-col gap-2 rounded-md border p-2 text-sm">
              <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                <dt class="text-muted-foreground">ID</dt>
                <dd class="select-text break-all">{props.id}</dd>
                <dt class="text-muted-foreground">{t("created")}</dt>
                <dd>{formatDateFull(note.data?.meta.created)}</dd>
                <dt class="text-muted-foreground">{t("updated")}</dt>
                <dd>{formatDateFull(note.data?.meta.updated)}</dd>
              </dl>
              <div>
                <div class="text-muted-foreground">
                  {t("backlinks", { count: backlinks.data?.length ?? 0 })}
                </div>
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
              </div>
            </div>
          </Show>
          <Show when={status() === "conflict"}>
            <div class="flex flex-wrap items-center gap-2 rounded-md border border-destructive p-2 text-sm">
              <span class="flex-1">{t("conflictMessage")}</span>
              <Button
                size="sm"
                variant="outline"
                onClick={() => resolve("discard")}
              >
                {t("loadLatest")}
              </Button>
              <Button
                size="sm"
                variant="destructive"
                onClick={() => resolve("overwrite")}
              >
                {t("overwrite")}
              </Button>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={props.id === undefined}>
        <p class="p-4 text-muted-foreground">{t("selectNote")}</p>
      </Show>
      <Show when={note.isError}>
        <p class="p-4 text-destructive">{note.error?.message}</p>
      </Show>
      <div
        ref={host}
        class="min-h-0 flex-1 overflow-hidden"
        classList={{ hidden: props.id === undefined || note.isError }}
      />
    </div>
  );
}
