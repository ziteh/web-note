import { syntaxTree } from "@codemirror/language";
import type { Extension, Range } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  type EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";

const mark = Decoration.mark({ class: "cm-inline-code" });
const blockLine = Decoration.line({ class: "cm-code-block" });

function buildDecorations(view: EditorView) {
  const ranges: Range<Decoration>[] = [];
  for (const { from, to } of view.visibleRanges) {
    syntaxTree(view.state).iterate({
      from,
      to,
      enter(node) {
        if (node.name === "InlineCode") {
          ranges.push(mark.range(node.from, node.to));
        } else if (node.name === "FencedCode" || node.name === "CodeBlock") {
          const { doc } = view.state;
          const last = doc.lineAt(node.to).number;
          for (let n = doc.lineAt(node.from).number; n <= last; n++) {
            ranges.push(blockLine.range(doc.line(n).from));
          }
        }
      },
    });
  }
  return Decoration.set(ranges, true);
}

/** Marks inline code spans and code block lines so the theme can give them a background. */
export function codeBackground(): Extension {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = buildDecorations(view);
      }

      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = buildDecorations(update.view);
        }
      }
    },
    { decorations: (plugin) => plugin.decorations },
  );
}
