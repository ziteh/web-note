import { syntaxTree } from "@codemirror/language";
import type { EditorState } from "@codemirror/state";

const CODE_NODES = new Set(["FencedCode", "CodeBlock", "InlineCode"]);

export function inCode(state: EditorState, pos: number) {
  let node: ReturnType<ReturnType<typeof syntaxTree>["resolveInner"]> | null =
    syntaxTree(state).resolveInner(pos, 1);
  while (node) {
    if (CODE_NODES.has(node.name)) return true;
    node = node.parent;
  }
  return false;
}
