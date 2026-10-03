import { A, useNavigate } from "@solidjs/router";
import { useQueryClient } from "@tanstack/solid-query";
import { createSignal, For, Show } from "solid-js";
import { toast } from "solid-sonner";
import { api, unwrap } from "~/api/client";
import { useOrphanAssets, useRefreshNotes, useTrash } from "~/api/queries";
import { UNTITLED } from "~/lib/note";
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
      toast.success(`已將 ${count} 個未使用的圖片移到垃圾桶`);
      await reload();
    } catch (e) {
      report(e);
    }
  };

  return (
    <div class="mx-auto flex h-full max-w-2xl flex-col gap-6 overflow-y-auto p-4">
      <header class="flex items-center gap-2">
        <A href="/" class="text-sm text-muted-foreground hover:underline">
          ← 返回
        </A>
        <h1 class="text-xl font-bold">垃圾桶</h1>
        <Button
          class="ml-auto"
          variant="destructive"
          size="sm"
          disabled={!trash.data?.length}
          onClick={() => setConfirmEmpty(true)}
        >
          清空垃圾桶
        </Button>
      </header>

      <ul class="flex flex-col divide-y rounded-md border">
        <For each={trash.data}>
          {(item) => (
            <li class="flex items-center gap-2 p-3">
              <div class="min-w-0 flex-1">
                <div class="truncate font-medium">{item.title ?? UNTITLED}</div>
                <div class="text-xs text-muted-foreground">
                  {item.notebook} ·{" "}
                  {new Date(item.trashed_at_millis).toLocaleString()}
                </div>
              </div>
              <Button
                size="sm"
                variant="outline"
                onClick={() => restore(item.key)}
              >
                還原
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => purge(item.key)}
              >
                永久刪除
              </Button>
            </li>
          )}
        </For>
        <Show when={trash.isSuccess && trash.data?.length === 0}>
          <li class="p-4 text-center text-sm text-muted-foreground">
            垃圾桶是空的
          </li>
        </Show>
      </ul>

      <section class="flex flex-col gap-2">
        <h2 class="font-semibold">未使用的圖片</h2>
        <p class="text-sm text-muted-foreground">
          沒有任何筆記（含垃圾桶內的筆記）引用的圖片，可以移到垃圾桶資料夾。
        </p>
        <div class="flex items-center gap-2">
          <span class="text-sm">{orphans.data?.length ?? 0} 個</span>
          <Button
            size="sm"
            variant="outline"
            disabled={!orphans.data?.length}
            onClick={trashOrphans}
          >
            移到垃圾桶
          </Button>
        </div>
      </section>

      <AlertDialog open={confirmEmpty()} onOpenChange={setConfirmEmpty}>
        <AlertDialogContent>
          <AlertDialogTitle>清空垃圾桶</AlertDialogTitle>
          <AlertDialogDescription>
            垃圾桶內的所有筆記會被永久刪除，無法還原。
          </AlertDialogDescription>
          <div class="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setConfirmEmpty(false)}>
              取消
            </Button>
            <Button variant="destructive" onClick={empty}>
              永久刪除
            </Button>
          </div>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
