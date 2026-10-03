import { A, useNavigate } from "@solidjs/router";
import { useQueryClient } from "@tanstack/solid-query";
import { createSignal, For, Show } from "solid-js";
import { toast } from "solid-sonner";
import { api, unwrap } from "~/api/client";
import { useOrphanAssets, useRefreshNotes, useTrash } from "~/api/queries";
import { dateLocale, t } from "~/lib/i18n";
import { noteTitle } from "~/lib/note";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";

function report(e: unknown) {
  toast.error(e instanceof Error ? e.message : String(e));
}

export default function TrashPage() {
  const trash = useTrash();
  const orphans = useOrphanAssets();
  const client = useQueryClient();
  const refresh = useRefreshNotes();
  const navigate = useNavigate();
  const [confirmEmpty, setConfirmEmpty] = createSignal(false);

  const reload = async () => {
    await Promise.all([
      refresh(),
      client.invalidateQueries({ queryKey: ["trash"] }),
      client.invalidateQueries({ queryKey: ["orphan-assets"] }),
    ]);
  };

  const restore = async (key: string) => {
    try {
      const note = await unwrap(
        api.POST("/api/trash/{key}/restore", { params: { path: { key } } }),
      );
      await reload();
      navigate(`/n/${note.meta.id}`);
    } catch (e) {
      report(e);
    }
  };

  const purge = async (key: string) => {
    try {
      await unwrap(
        api.DELETE("/api/trash/{key}", { params: { path: { key } } }),
      );
      await reload();
    } catch (e) {
      report(e);
    }
  };

  const empty = async () => {
    setConfirmEmpty(false);
    try {
      await unwrap(api.DELETE("/api/trash"));
      await reload();
    } catch (e) {
      report(e);
    }
  };

  const trashOrphans = async () => {
    try {
      const { count } = await unwrap(api.POST("/api/assets/orphans/trash"));
      toast.success(t("orphansTrashed", { count }));
      await reload();
    } catch (e) {
      report(e);
    }
  };

  return (
    <div class="mx-auto flex h-full max-w-2xl flex-col gap-6 overflow-y-auto p-4">
      <header class="flex items-center gap-2">
        <A href="/" class="text-sm text-muted-foreground hover:underline">
          {t("back")}
        </A>
        <h1 class="text-xl font-bold">{t("trash")}</h1>
        <Button
          class="ml-auto"
          variant="destructive"
          size="sm"
          disabled={!trash.data?.length}
          onClick={() => setConfirmEmpty(true)}
        >
          {t("emptyTrash")}
        </Button>
      </header>

      <ul class="flex flex-col divide-y rounded-md border">
        <For each={trash.data}>
          {(item) => (
            <li class="flex items-center gap-2 p-3">
              <div class="min-w-0 flex-1">
                <div class="truncate font-medium">{noteTitle(item)}</div>
                <div class="text-xs text-muted-foreground">
                  {item.notebook} ·{" "}
                  {new Date(item.trashed_at_millis).toLocaleString(
                    dateLocale(),
                  )}
                </div>
              </div>
              <Button
                size="sm"
                variant="outline"
                onClick={() => restore(item.key)}
              >
                {t("restore")}
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => purge(item.key)}
              >
                {t("permanentDelete")}
              </Button>
            </li>
          )}
        </For>
        <Show when={trash.isSuccess && trash.data?.length === 0}>
          <li class="p-4 text-center text-sm text-muted-foreground">
            {t("trashEmpty")}
          </li>
        </Show>
      </ul>

      <section class="flex flex-col gap-2">
        <h2 class="font-semibold">{t("unusedImages")}</h2>
        <p class="text-sm text-muted-foreground">{t("unusedImagesHint")}</p>
        <div class="flex items-center gap-2">
          <span class="text-sm">
            {t("unusedImagesCount", { count: orphans.data?.length ?? 0 })}
          </span>
          <Button
            size="sm"
            variant="outline"
            disabled={!orphans.data?.length}
            onClick={trashOrphans}
          >
            {t("moveToTrash")}
          </Button>
        </div>
      </section>

      <AlertDialog open={confirmEmpty()} onOpenChange={setConfirmEmpty}>
        <AlertDialogContent>
          <AlertDialogTitle>{t("emptyTrash")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("emptyTrashConfirm")}
          </AlertDialogDescription>
          <div class="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setConfirmEmpty(false)}>
              {t("cancel")}
            </Button>
            <Button variant="destructive" onClick={empty}>
              {t("permanentDelete")}
            </Button>
          </div>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
