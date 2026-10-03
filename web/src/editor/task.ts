import { syntaxTree } from "@codemirror/language";
import type { Extension, Range } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  type EditorView,
  ViewPlugin,
  type ViewUpdate,
  WidgetType,
} from "@codemirror/view";

class CheckboxWidget extends WidgetType {
  checked: boolean;

  constructor(checked: boolean) {
    super();
    this.checked = checked;
  }

  eq(other: CheckboxWidget) {
    return other.checked === this.checked;
  }

  toDOM(view: EditorView) {
    const box = document.createElement("input");
    box.type = "checkbox";
    box.className = "cm-task-checkbox";
    box.checked = this.checked;
    box.addEventListener("mousedown", (event) => {
      event.preventDefault();
      const pos = view.posAtDOM(box);
      view.dispatch({
        changes: {
          from: pos + 1,
          to: pos + 2,
          insert: this.checked ? " " : "x",
        },
      });
    });
    return box;
  }

  ignoreEvent() {
    return true;
  }
}

function buildDecorations(view: EditorView) {
  const { state } = view;
  const ranges: Range<Decoration>[] = [];
  syntaxTree(state).iterate({
    enter(node) {
      if (node.name !== "TaskMarker") return;
      const touched = state.selection.ranges.some(
        (r) => r.from <= node.to && r.to >= node.from,
      );
      if (touched) return;
      const checked = state.sliceDoc(node.from, node.to) !== "[ ]";
      ranges.push(
        Decoration.replace({ widget: new CheckboxWidget(checked) }).range(
          node.from,
          node.to,
        ),
      );
    },
  });
  return Decoration.set(ranges);
}

/** Renders `[ ]` and `[x]` as checkboxes that toggle on click. */
export function taskCheckboxes(): Extension {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = buildDecorations(view);
      }

      update(update: ViewUpdate) {
        if (
          update.docChanged ||
          update.selectionSet ||
          update.viewportChanged
        ) {
          this.decorations = buildDecorations(update.view);
        }
      }
    },
    { decorations: (plugin) => plugin.decorations },
  );
}
