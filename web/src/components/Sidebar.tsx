import { For, Show } from "solid-js";
import { useNotebooks, useStatus, useTags } from "~/api/queries";
import { useFilters } from "~/lib/filters";
import { Button } from "./ui/button";

export default function Sidebar(props: { onPick?: () => void }) {
  const notebooks = useNotebooks();
  const tags = useTags();
  const status = useStatus();
  const { filters, update } = useFilters();

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

  return (
    <nav class="flex h-full flex-col gap-4 overflow-y-auto p-2">
      <section class="flex flex-col gap-1">
        <h2 class="px-2 text-xs font-semibold text-muted-foreground">筆記本</h2>
        <Button
          variant={filters().notebook ? "ghost" : "secondary"}
          class="justify-start"
          onClick={() => pick({ notebook: undefined })}
        >
          全部
        </Button>
        <For each={notebooks.data}>
          {(nb) => (
            <Button
              variant={filters().notebook === nb.name ? "secondary" : "ghost"}
              class="justify-between"
              onClick={() => pick({ notebook: nb.name })}
            >
              <span class="truncate">{nb.name}</span>
              <span class="text-xs text-muted-foreground">{nb.count}</span>
            </Button>
          )}
        </For>
      </section>

      <section class="flex flex-col gap-1">
        <h2 class="px-2 text-xs font-semibold text-muted-foreground">標籤</h2>
        <For each={tags.data}>
          {(t) => (
            <Button
              variant={filters().tags.includes(t.tag) ? "secondary" : "ghost"}
              class="justify-between"
              onClick={() => toggleTag(t.tag)}
            >
              <span class="truncate">{t.tag}</span>
              <span class="text-xs text-muted-foreground">{t.count}</span>
            </Button>
          )}
        </For>
      </section>

      <Show when={status.data?.errors.length}>
        <p class="px-2 text-xs text-destructive">
          {status.data?.errors.length} 個檔案無法載入
        </p>
      </Show>
    </nav>
  );
}
