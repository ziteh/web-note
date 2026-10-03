import { For, Show } from "solid-js";
import { useNote } from "~/api/queries";
import { formatDate, noteTitle } from "~/lib/note";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";

interface Props {
  id?: string;
  onBack?: () => void;
}

/** Read-only for now; the editor replaces the body view. */
export default function NotePane(props: Props) {
  const note = useNote(() => props.id);

  return (
    <div class="flex h-full flex-col">
      <Show when={props.onBack}>
        <div class="border-b p-2">
          <Button variant="ghost" onClick={props.onBack}>
            ← 返回
          </Button>
        </div>
      </Show>
      <div class="min-h-0 flex-1 overflow-y-auto p-4">
        <Show
          when={props.id}
          fallback={<p class="text-muted-foreground">選擇一則筆記</p>}
        >
          <Show when={note.isError}>
            <p class="text-destructive">{note.error?.message}</p>
          </Show>
          <Show when={note.data}>
            {(data) => (
              <article class="flex flex-col gap-3">
                <h1 class="text-2xl font-bold">{noteTitle(data().meta)}</h1>
                <div class="flex flex-wrap items-center gap-1 text-xs text-muted-foreground">
                  <span>{data().meta.notebook}</span>
                  <span>·</span>
                  <span>{formatDate(data().meta.updated)}</span>
                  <For each={data().meta.tags}>
                    {(tag) => <Badge variant="secondary">{tag}</Badge>}
                  </For>
                </div>
                <pre class="whitespace-pre-wrap break-words font-sans text-sm">
                  {data().body}
                </pre>
              </article>
            )}
          </Show>
        </Show>
      </div>
    </div>
  );
}
