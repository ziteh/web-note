import {
  autocompletion,
  type CompletionContext,
} from "@codemirror/autocomplete";
import {
  type EditorState,
  type Extension,
  type Range,
  StateEffect,
} from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  type EditorView,
  ViewPlugin,
  type ViewUpdate,
  WidgetType,
} from "@codemirror/view";
import type { NoteMeta } from "~/api/client";
import { formatDate, noteTitle } from "~/lib/note";
import { inCode } from "./syntax";

export interface NoteLookup {
  get(id: string): NoteMeta | undefined;
  all(): NoteMeta[];
}

/** Dispatch when the lookup's contents changed so titles are redrawn. */
export const lookupChanged = StateEffect.define<null>();

const WIKILINK = /\[\[([0-9a-z]+)(?:\|([^[\]\n]*))?\]\]/g;
const UNSAFE_ALIAS = /[[\]|\n]/g;

class WikiLinkWidget extends WidgetType {
  id: string;
  text: string;
  broken: boolean;

  constructor(id: string, text: string, broken: boolean) {
    super();
    this.id = id;
    this.text = text;
    this.broken = broken;
  }

  eq(other: WikiLinkWidget) {
    return (
      other.id === this.id &&
      other.text === this.text &&
      other.broken === this.broken
    );
  }

  toDOM() {
    const el = document.createElement("span");
    el.className = this.broken
      ? "cm-wikilink cm-wikilink-broken"
      : "cm-wikilink";
    el.textContent = this.text;
    el.dataset.noteId = this.id;
    return el;
  }

  ignoreEvent() {
    return false;
  }
}

function buildDecorations(view: EditorView, lookup: NoteLookup) {
  const { state } = view;
  const text = state.doc.toString();
  const ranges: Range<Decoration>[] = [];
  for (const match of text.matchAll(WIKILINK)) {
    const from = match.index;
    const to = from + match[0].length;
    if (touchesSelection(state, from, to) || inCode(state, from)) continue;
    const note = lookup.get(match[1]);
    // Broken links keep the alias text they were written with.
    const label = note ? noteTitle(note) : (match[2] ?? match[1]);
    ranges.push(
      Decoration.replace({
        widget: new WikiLinkWidget(match[1], label, !note),
      }).range(from, to),
    );
  }
  return Decoration.set(ranges);
}

function touchesSelection(state: EditorState, from: number, to: number) {
  return state.selection.ranges.some((r) => r.from <= to && r.to >= from);
}

/** Shows `[[id|alias]]` as the note's current title; Ctrl/Cmd-click opens it. */
export function wikilinks(
  lookup: NoteLookup,
  onOpen: (id: string) => void,
): Extension {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = buildDecorations(view, lookup);
      }

      update(update: ViewUpdate) {
        const refreshed = update.transactions.some((tr) =>
          tr.effects.some((e) => e.is(lookupChanged)),
        );
        if (update.docChanged || update.selectionSet || refreshed) {
          this.decorations = buildDecorations(update.view, lookup);
        }
      }
    },
    {
      decorations: (plugin) => plugin.decorations,
      eventHandlers: {
        mousedown(event) {
          const target = event.target as HTMLElement;
          const el = target.closest<HTMLElement>(".cm-wikilink");
          const id = el?.dataset.noteId;
          if (!id || !(event.metaKey || event.ctrlKey)) return false;
          event.preventDefault();
          onOpen(id);
          return true;
        },
      },
    },
  );
}

/** Typing `[[` offers notes; picking one inserts `[[id|title]]`. */
export function wikilinkCompletion(lookup: NoteLookup): Extension {
  const marker = "[[".length;
  return autocompletion({
    override: [
      (context: CompletionContext) => {
        const match = context.matchBefore(/\[\[[^[\]|\n]*/);
        if (!match) return null;
        return {
          from: match.from + marker,
          validFor: /^[^[\]|\n]*$/,
          options: lookup.all().map((note) => ({
            label: noteTitle(note),
            detail: `${note.path} · ${formatDate(note.created)}`,
            info: note.summary,
            apply(
              view: EditorView,
              _completion: unknown,
              from: number,
              to: number,
            ) {
              const alias = noteTitle(note).replace(UNSAFE_ALIAS, " ");
              const link = `[[${note.id}|${alias}]]`;
              const closing = view.state.sliceDoc(to, to + 2) === "]]" ? 2 : 0;
              view.dispatch({
                changes: {
                  from: from - marker,
                  to: to + closing,
                  insert: link,
                },
                selection: { anchor: from - marker + link.length },
              });
            },
          })),
        };
      },
    ],
  });
}
