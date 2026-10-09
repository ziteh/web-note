import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import type { Extension } from "@codemirror/state";
import {
  drawSelection,
  EditorView,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { codeBackground } from "./codebackground";
import { imageSupport } from "./image";
import { codeLanguages } from "./languages";
import { taskCheckboxes } from "./task";
import { faintMarks, theme } from "./theme";
import { type NoteLookup, wikilinkCompletion, wikilinks } from "./wikilink";

interface Options {
  lookup: NoteLookup;
  onOpenNote: (id: string) => void;
  onError: (message: string) => void;
}

export function createExtensions(options: Options): Extension[] {
  return [
    history(),
    lineNumbers(),
    drawSelection(),
    EditorView.lineWrapping,
    keymap.of([...defaultKeymap, ...historyKeymap]),
    markdown({
      base: markdownLanguage,
      codeLanguages,
      extensions: { props: [faintMarks] },
    }),
    theme,
    wikilinks(options.lookup, options.onOpenNote),
    wikilinkCompletion(options.lookup),
    taskCheckboxes(),
    codeBackground(),
    imageSupport(options.onError),
  ];
}
