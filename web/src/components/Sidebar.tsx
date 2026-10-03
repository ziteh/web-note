import { A } from "@solidjs/router";
import { createSignal, For, Show } from "solid-js";
import { toast } from "solid-sonner";
import { api, unwrap } from "~/api/client";
import {
  useNotebooks,
  useRefreshNotes,
  useStatus,
  useTags,
} from "~/api/queries";
import MoonIcon from "~/assets/moon.svg";
import PlusIcon from "~/assets/plus.svg";
import SettingsIcon from "~/assets/settings.svg";
import SunIcon from "~/assets/sun.svg";
import TrashIcon from "~/assets/trash.svg";
import { useFilters } from "~/lib/filters";
import { t } from "~/lib/i18n";
import { isDark, toggleTheme } from "~/lib/theme";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Button, buttonVariants } from "./ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "./ui/context-menu";
import { TextField, TextFieldInput } from "./ui/text-field";

type Dialog =
  | { kind: "create" }
  | { kind: "rename"; name: string }
  | { kind: "renameTag"; name: string }
  | { kind: "delete"; name: string };

export default function Sidebar(props: { onPick?: () => void }) {
  const notebooks = useNotebooks();
  const tags = useTags();
  const status = useStatus();
  const refresh = useRefreshNotes();
  const { filters, update } = useFilters();
  const [dialog, setDialog] = createSignal<Dialog>();
  const [draft, setDraft] = createSignal("");

  const pick = (patch: Parameters<typeof update>[0]) => {
    update(patch, "/");
    props.onPick?.();
  };

  const toggleTag = (tag: string) => {
    const current = filters().tags;
    pick({
      tags: current.includes(tag)
        ? current.filter((t) => t !== tag)
        : [...current, tag],
    });
  };

  const openDialog = (next: Dialog) => {
    setDraft(
      next.kind === "rename" || next.kind === "renameTag" ? next.name : "",
    );
    setDialog(next);
  };

  const run = async (action: () => Promise<unknown>) => {
    setDialog(undefined);
    try {
      await action();
      await refresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const submitName = () => {
    const current = dialog();
    const name = draft().trim();
    if (!current || current.kind === "delete" || !name) return;
    if (current.kind === "renameTag") {
      const from = current.name;
      void run(async () => {
        await unwrap(
          api.POST("/api/tags/rename", { body: { from, to: name } }),
        );
        const active = filters().tags;
        if (active.includes(from)) {
          update({
            tags: [
              ...new Set(active.map((tag) => (tag === from ? name : tag))),
            ],
          });
        }
      });
      return;
    }
    if (current.kind === "create") {
      void run(() => unwrap(api.POST("/api/notebooks", { body: { name } })));
      return;
    }
    const from = current.name;
    void run(async () => {
      await unwrap(
        api.PATCH("/api/notebooks/{name}", {
          params: { path: { name: from } },
          body: { name },
        }),
      );
      if (filters().notebook === from) update({ notebook: name });
    });
  };

  const confirmDelete = (name: string) =>
    run(async () => {
      await unwrap(
        api.DELETE("/api/notebooks/{name}", { params: { path: { name } } }),
      );
      if (filters().notebook === name) update({ notebook: undefined });
    });

  return (
    <nav class="flex h-full flex-col gap-4 overflow-y-auto p-2">
      <section class="flex flex-col gap-1">
        <div class="flex items-center justify-between">
          <h2 class="px-2 text-xs font-semibold text-muted-foreground">
            {t("notebooks")}
          </h2>
          <Button
            variant="ghost"
            size="icon"
            class="size-6"
            onClick={() => openDialog({ kind: "create" })}
            aria-label={t("newNotebook")}
          >
            <PlusIcon />
          </Button>
        </div>
        <Button
          variant={filters().notebook ? "ghost" : "secondary"}
          class="justify-start"
          onClick={() => pick({ notebook: undefined })}
        >
          {t("all")}
        </Button>
        <For each={notebooks.data}>
          {(nb) => (
            <ContextMenu>
              <ContextMenuTrigger
                as={Button}
                variant={filters().notebook === nb.name ? "secondary" : "ghost"}
                class="justify-between"
                onClick={() => pick({ notebook: nb.name })}
              >
                <span class="truncate">{nb.name}</span>
                <span class="text-xs text-muted-foreground">{nb.count}</span>
              </ContextMenuTrigger>
              <ContextMenuContent>
                <ContextMenuItem
                  onSelect={() => openDialog({ kind: "rename", name: nb.name })}
                >
                  {t("rename")}
                </ContextMenuItem>
                <ContextMenuItem
                  class="text-destructive"
                  onSelect={() => openDialog({ kind: "delete", name: nb.name })}
                >
                  {t("delete")}
                </ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
          )}
        </For>
      </section>

      <section class="flex flex-col gap-1">
        <h2 class="px-2 text-xs font-semibold text-muted-foreground">
          {t("tags")}
        </h2>
        <For each={tags.data}>
          {(tag) => (
            <ContextMenu>
              <ContextMenuTrigger
                as={Button}
                variant={
                  filters().tags.includes(tag.tag) ? "secondary" : "ghost"
                }
                class="justify-between"
                onClick={() => toggleTag(tag.tag)}
              >
                <span class="truncate">{tag.tag}</span>
                <span class="text-xs text-muted-foreground">{tag.count}</span>
              </ContextMenuTrigger>
              <ContextMenuContent>
                <ContextMenuItem
                  onSelect={() =>
                    openDialog({ kind: "renameTag", name: tag.tag })
                  }
                >
                  {t("rename")}
                </ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
          )}
        </For>
      </section>

      <div class="mt-auto flex items-center justify-between">
        <A
          href="/trash"
          class={buttonVariants({ variant: "ghost", size: "icon" })}
          aria-label={t("trash")}
        >
          <TrashIcon />
        </A>
        <div class="flex items-center">
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleTheme}
            aria-label={t("toggleTheme")}
          >
            {isDark() ? <MoonIcon /> : <SunIcon />}
          </Button>
          <A
            href="/settings"
            class={buttonVariants({ variant: "ghost", size: "icon" })}
            aria-label={t("settings")}
          >
            <SettingsIcon />
          </A>
        </div>
      </div>

      <Show when={status.data?.errors.length}>
        <p class="px-2 text-xs text-destructive">
          {t("loadErrors", { count: status.data?.errors.length ?? 0 })}
        </p>
      </Show>

      <AlertDialog
        open={dialog() !== undefined}
        onOpenChange={(open) => !open && setDialog(undefined)}
      >
        <AlertDialogContent>
          {(() => {
            const current = dialog();
            if (current?.kind === "delete") {
              return (
                <>
                  <AlertDialogTitle>{t("deleteNotebook")}</AlertDialogTitle>
                  <AlertDialogDescription>
                    {t("deleteNotebookConfirm", { name: current.name })}
                  </AlertDialogDescription>
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="outline"
                      onClick={() => setDialog(undefined)}
                    >
                      {t("cancel")}
                    </Button>
                    <Button
                      variant="destructive"
                      onClick={() => void confirmDelete(current.name)}
                    >
                      {t("delete")}
                    </Button>
                  </div>
                </>
              );
            }
            return (
              <>
                <AlertDialogTitle>
                  {current?.kind === "rename"
                    ? t("renameNotebook")
                    : current?.kind === "renameTag"
                      ? t("renameTag")
                      : t("newNotebook")}
                </AlertDialogTitle>
                <TextField value={draft()} onChange={setDraft}>
                  <TextFieldInput
                    autofocus
                    onKeyDown={(e: KeyboardEvent) =>
                      e.key === "Enter" && !e.isComposing && submitName()
                    }
                  />
                </TextField>
                <div class="flex justify-end gap-2">
                  <Button
                    variant="outline"
                    onClick={() => setDialog(undefined)}
                  >
                    {t("cancel")}
                  </Button>
                  <Button disabled={!draft().trim()} onClick={submitName}>
                    {current?.kind === "create" ? t("create") : t("rename")}
                  </Button>
                </div>
              </>
            );
          })()}
        </AlertDialogContent>
      </AlertDialog>
    </nav>
  );
}
