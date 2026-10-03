import {
  defaultHighlightStyle,
  HighlightStyle,
  syntaxHighlighting,
} from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags as t } from "@lezer/highlight";

const markdownStyle = HighlightStyle.define([
  { tag: t.heading1, fontSize: "1.6em", fontWeight: "700" },
  { tag: t.heading2, fontSize: "1.35em", fontWeight: "700" },
  { tag: t.heading3, fontSize: "1.15em", fontWeight: "700" },
  { tag: [t.heading4, t.heading5, t.heading6], fontWeight: "700" },
  { tag: t.strong, fontWeight: "700" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.strikethrough, textDecoration: "line-through" },
  { tag: t.quote, color: "hsl(var(--muted-foreground))" },
  { tag: t.processingInstruction, color: "hsl(var(--muted-foreground))" },
  { tag: [t.link, t.url], color: "hsl(var(--primary))" },
  {
    tag: t.monospace,
    fontFamily: "'Sarasa Fixed TC', ui-monospace, monospace",
  },
]);

const base = EditorView.theme({
  "&": { height: "100%", backgroundColor: "transparent" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "1.7" },
  ".cm-content": { padding: "1rem" },
  ".cm-selectionBackground, &.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, ::selection":
    { backgroundColor: "hsl(var(--muted-foreground) / 0.3)" },
  ".cm-cursor, .cm-dropCursor": {
    borderLeftColor: "hsl(var(--foreground))",
  },
  ".cm-wikilink": {
    color: "hsl(var(--primary))",
    textDecoration: "underline",
    cursor: "pointer",
  },
  ".cm-wikilink-broken": {
    color: "hsl(var(--destructive))",
    textDecoration: "line-through",
  },
  ".cm-task-checkbox": { margin: "0 0.25rem 0 0", verticalAlign: "middle" },
  ".cm-image": { display: "block", maxWidth: "100%", margin: "0.5rem 0" },
});

export const theme = [
  base,
  syntaxHighlighting(markdownStyle),
  syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
];
