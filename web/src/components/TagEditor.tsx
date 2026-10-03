import { createSignal, For } from "solid-js";
import { Badge } from "./ui/badge";
import { TextField, TextFieldInput } from "./ui/text-field";

interface Props {
  tags: string[];
  onChange: (tags: string[]) => void;
}

function normalize(tag: string) {
  return tag.trim().normalize("NFC").toLowerCase();
}

export default function TagEditor(props: Props) {
  const [draft, setDraft] = createSignal("");

  const add = () => {
    const tag = normalize(draft());
    setDraft("");
    if (tag && !props.tags.includes(tag)) props.onChange([...props.tags, tag]);
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
      <TextField value={draft()} onChange={setDraft}>
        <TextFieldInput
          class="h-7 w-28 text-xs"
          placeholder="新增標籤"
          onKeyDown={(e: KeyboardEvent) => {
            if (e.key === "Enter" && !e.isComposing) {
              e.preventDefault();
              add();
            }
          }}
          onBlur={add}
        />
      </TextField>
    </div>
  );
}
