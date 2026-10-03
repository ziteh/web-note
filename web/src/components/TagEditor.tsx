import * as ComboboxPrimitive from "@kobalte/core/combobox";
import { createMemo, createSignal, For } from "solid-js";
import { useTags } from "~/api/queries";
import { Badge } from "./ui/badge";

interface Props {
  tags: string[];
  onChange: (tags: string[]) => void;
}

function normalize(tag: string) {
  return tag.trim().normalize("NFC").toLowerCase();
}

export default function TagEditor(props: Props) {
  const known = useTags();
  const [draft, setDraft] = createSignal("");

  /** Existing tags not yet on the note, plus the draft when it would be new. */
  const options = createMemo(() => {
    const names = (known.data ?? [])
      .map((t) => t.tag)
      .filter((t) => !props.tags.includes(t));
    const typed = normalize(draft());
    if (typed && !props.tags.includes(typed) && !names.includes(typed)) {
      names.unshift(typed);
    }
    return names;
  });

  const add = (tag: string | null) => {
    setDraft("");
    const value = tag && normalize(tag);
    if (value && !props.tags.includes(value)) {
      props.onChange([...props.tags, value]);
    }
  };

  const remove = (tag: string) =>
    props.onChange(props.tags.filter((t) => t !== tag));

  return (
    <div class="flex flex-wrap items-center gap-1">
      <For each={props.tags}>
        {(tag) => (
          <Badge variant="secondary" class="gap-1">
            {tag}
            <button
              type="button"
              aria-label="移除標籤"
              onClick={() => remove(tag)}
            >
              ×
            </button>
          </Badge>
        )}
      </For>
      <ComboboxPrimitive.Root<string>
        options={options()}
        value={null}
        onChange={add}
        onInputChange={setDraft}
        allowsEmptyCollection
        placeholder="新增標籤"
        itemComponent={(p) => (
          <ComboboxPrimitive.Item
            item={p.item}
            class="flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground"
          >
            <ComboboxPrimitive.ItemLabel>
              {p.item.rawValue}
            </ComboboxPrimitive.ItemLabel>
          </ComboboxPrimitive.Item>
        )}
      >
        <ComboboxPrimitive.Control>
          <ComboboxPrimitive.Input class="h-7 w-28 rounded-md border border-input bg-transparent px-2 text-xs placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring" />
        </ComboboxPrimitive.Control>
        <ComboboxPrimitive.Portal>
          <ComboboxPrimitive.Content class="relative z-50 min-w-32 overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md animate-in fade-in-80">
            <ComboboxPrimitive.Listbox class="m-0 p-1" />
          </ComboboxPrimitive.Content>
        </ComboboxPrimitive.Portal>
      </ComboboxPrimitive.Root>
    </div>
  );
}
