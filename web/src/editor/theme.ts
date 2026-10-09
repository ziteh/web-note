import {
  defaultHighlightStyle,
  HighlightStyle,
  syntaxHighlighting,
} from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { styleTags, Tag, tags as t } from "@lezer/highlight";

const faintMark = Tag.define();

/** Dims the markers of inline code and emphasis, and whole images. */
export const faintMarks = styleTags({
  "CodeMark EmphasisMark StrikethroughMark": faintMark,
  "Image/...": faintMark,
});

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
  { tag: faintMark, color: "hsl(var(--muted-foreground) / 0.5)" },
  {
    tag: [t.keyword, t.operatorKeyword, t.modifier],
    color: "hsl(var(--syntax-keyword))",
  },
  {
    tag: [t.string, t.regexp, t.special(t.string)],
    color: "hsl(var(--syntax-string))",
  },
  {
    tag: [t.number, t.bool, t.null, t.atom],
    color: "hsl(var(--syntax-number))",
  },
  {
    tag: [t.function(t.variableName), t.function(t.propertyName), t.macroName],
    color: "hsl(var(--syntax-function))",
  },
  {
    tag: [t.typeName, t.className, t.namespace, t.tagName],
    color: "hsl(var(--syntax-type))",
  },
  {
    tag: [t.attributeName, t.propertyName],
    color: "hsl(var(--syntax-function))",
  },
  {
    tag: [t.comment, t.lineComment, t.blockComment],
    color: "hsl(var(--syntax-comment))",
    fontStyle: "italic",
  },
  {
    tag: t.monospace,
    fontFamily: "'Sarasa Fixed TC', ui-monospace, monospace",
  },
]);

const base = EditorView.theme({
  "&": { height: "100%", backgroundColor: "transparent" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "1.7" },
  // 水平 padding 放在 line 而非 content,否則選取範圍的左緣會與文字錯位
  ".cm-content": { padding: "1rem 0" },
  ".cm-line": { padding: "0 1rem" },
  ".cm-gutters": {
    backgroundColor: "transparent",
    border: "none",
    color: "hsl(var(--muted-foreground))",
  },
  ".cm-selectionBackground, &.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, ::selection":
    { backgroundColor: "hsl(var(--muted-foreground) / 0.3)" },
  ".cm-selectionLayer": { zIndex: "150 !important", pointerEvents: "none" },
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
  ".cm-inline-code": {
    backgroundColor: "hsl(var(--muted))",
    borderRadius: "0.25em",
    boxShadow: "0 0 0 0.15em hsl(var(--muted))",
  },
  ".cm-code-block": { backgroundColor: "hsl(var(--muted))" },
  ".cm-task-checkbox": { margin: "0 0.25rem 0 0", verticalAlign: "middle" },
  ".cm-image": { display: "block", maxWidth: "100%", margin: "0.5rem 0" },
});

export const theme = [
  base,
  syntaxHighlighting(markdownStyle),
  syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
];
