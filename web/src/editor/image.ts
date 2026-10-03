import { syntaxTree } from "@codemirror/language";
import {
  type EditorState,
  type Extension,
  type Range,
  StateField,
} from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import { uploadAsset } from "~/api/client";

const ASSET_PATH = /^\.\.\/\.assets\/([0-9a-f]+\.[a-z]+)$/;

class ImageWidget extends WidgetType {
  src: string;

  constructor(src: string) {
    super();
    this.src = src;
  }

  eq(other: ImageWidget) {
    return other.src === this.src;
  }

  toDOM() {
    const img = document.createElement("img");
    img.className = "cm-image";
    img.src = this.src;
    img.alt = "";
    return img;
  }
}

function buildDecorations(state: EditorState) {
  const ranges: Range<Decoration>[] = [];
  syntaxTree(state).iterate({
    enter(node) {
      if (node.name !== "Image") return;
      const url = node.node.getChild("URL");
      if (!url) return;
      const match = ASSET_PATH.exec(state.sliceDoc(url.from, url.to));
      if (!match) return;
      ranges.push(
        Decoration.widget({
          widget: new ImageWidget(`/api/assets/${match[1]}`),
          block: true,
          side: 1,
        }).range(state.doc.lineAt(node.to).to),
      );
    },
  });
  return Decoration.set(ranges, true);
}

/** Shows images stored under `../.assets/` below the line that references them. */
const imageWidgets = StateField.define({
  create: buildDecorations,
  update: (value, tr) => (tr.docChanged ? buildDecorations(tr.state) : value),
  provide: (field) => EditorView.decorations.from(field),
});

function imageFiles(files: FileList | null | undefined) {
  return Array.from(files ?? []).filter((f) => f.type.startsWith("image/"));
}

async function insertImages(
  view: EditorView,
  files: File[],
  at: number | null,
  onError: (message: string) => void,
) {
  for (const file of files) {
    try {
      const { path } = await uploadAsset(file);
      const pos = Math.min(
        at ?? view.state.selection.main.head,
        view.state.doc.length,
      );
      const insert = `![](${path})`;
      view.dispatch({
        changes: { from: pos, insert },
        selection: { anchor: pos + insert.length },
      });
      at = null;
    } catch (e) {
      onError(e instanceof Error ? e.message : String(e));
    }
  }
}

/** Pasting or dropping images uploads them and inserts a relative link. */
export function imageSupport(onError: (message: string) => void): Extension {
  return [
    imageWidgets,
    EditorView.domEventHandlers({
      paste(event, view) {
        const files = imageFiles(event.clipboardData?.files);
        if (files.length === 0) return false;
        event.preventDefault();
        void insertImages(view, files, null, onError);
        return true;
      },
      drop(event, view) {
        const files = imageFiles(event.dataTransfer?.files);
        if (files.length === 0) return false;
        event.preventDefault();
        const at = view.posAtCoords({ x: event.clientX, y: event.clientY });
        void insertImages(view, files, at, onError);
        return true;
      },
    }),
  ];
}
