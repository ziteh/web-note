import { css } from "@codemirror/lang-css";
import { html } from "@codemirror/lang-html";
import { javascript } from "@codemirror/lang-javascript";
import { markdown } from "@codemirror/lang-markdown";
import {
  LanguageDescription,
  LanguageSupport,
  StreamLanguage,
  type StreamParser,
} from "@codemirror/language";

function legacy<T>(parser: StreamParser<T>) {
  return new LanguageSupport(StreamLanguage.define(parser));
}

// Only languages listed here are bundled, which keeps the embedded assets small.
// The markdown package already bundles html, css and javascript, so those load
// statically.
export const codeLanguages = [
  LanguageDescription.of({
    name: "JavaScript",
    alias: ["js", "mjs", "cjs", "node"],
    load: async () => javascript(),
  }),
  LanguageDescription.of({
    name: "TypeScript",
    alias: ["ts"],
    load: async () => javascript({ typescript: true }),
  }),
  LanguageDescription.of({
    name: "JSX",
    load: async () => javascript({ jsx: true }),
  }),
  LanguageDescription.of({
    name: "TSX",
    load: async () => javascript({ jsx: true, typescript: true }),
  }),
  LanguageDescription.of({
    name: "Python",
    alias: ["py"],
    load: () => import("@codemirror/lang-python").then((m) => m.python()),
  }),
  LanguageDescription.of({
    name: "Rust",
    alias: ["rs"],
    load: () => import("@codemirror/lang-rust").then((m) => m.rust()),
  }),
  LanguageDescription.of({
    name: "Go",
    alias: ["golang"],
    load: () => import("@codemirror/lang-go").then((m) => m.go()),
  }),
  LanguageDescription.of({
    name: "C",
    load: () => import("@codemirror/lang-cpp").then((m) => m.cpp()),
  }),
  LanguageDescription.of({
    name: "C++",
    alias: ["cpp", "cc", "cxx"],
    load: () => import("@codemirror/lang-cpp").then((m) => m.cpp()),
  }),
  LanguageDescription.of({
    name: "Java",
    load: () => import("@codemirror/lang-java").then((m) => m.java()),
  }),
  LanguageDescription.of({
    name: "Kotlin",
    alias: ["kt", "kts"],
    load: () =>
      import("@codemirror/legacy-modes/mode/clike").then((m) =>
        legacy(m.kotlin),
      ),
  }),
  LanguageDescription.of({
    name: "Shell",
    alias: ["bash", "sh", "zsh"],
    load: () =>
      import("@codemirror/legacy-modes/mode/shell").then((m) =>
        legacy(m.shell),
      ),
  }),
  LanguageDescription.of({
    name: "SQL",
    load: () => import("@codemirror/lang-sql").then((m) => m.sql()),
  }),
  LanguageDescription.of({
    name: "JSON",
    load: () => import("@codemirror/lang-json").then((m) => m.json()),
  }),
  LanguageDescription.of({
    name: "YAML",
    alias: ["yml"],
    load: () => import("@codemirror/lang-yaml").then((m) => m.yaml()),
  }),
  LanguageDescription.of({
    name: "TOML",
    load: () =>
      import("@codemirror/legacy-modes/mode/toml").then((m) => legacy(m.toml)),
  }),
  LanguageDescription.of({
    name: "HTML",
    load: async () => html(),
  }),
  LanguageDescription.of({
    name: "CSS",
    load: async () => css(),
  }),
  LanguageDescription.of({
    name: "Markdown",
    alias: ["md"],
    load: async () => markdown(),
  }),
  LanguageDescription.of({
    name: "diff",
    alias: ["patch"],
    load: () =>
      import("@codemirror/legacy-modes/mode/diff").then((m) => legacy(m.diff)),
  }),
  LanguageDescription.of({
    name: "Dockerfile",
    alias: ["docker"],
    load: () =>
      import("@codemirror/legacy-modes/mode/dockerfile").then((m) =>
        legacy(m.dockerFile),
      ),
  }),
  LanguageDescription.of({
    name: "Lua",
    alias: ["lua"],
    load: () =>
      import("@codemirror/legacy-modes/mode/lua").then((m) => legacy(m.lua)),
  }),
  LanguageDescription.of({
    name: "Haskell",
    alias: ["hs"],
    load: () =>
      import("@codemirror/legacy-modes/mode/haskell").then((m) =>
        legacy(m.haskell),
      ),
  }),
  LanguageDescription.of({
    name: "R",
    load: () =>
      import("@codemirror/legacy-modes/mode/r").then((m) => legacy(m.r)),
  }),
  LanguageDescription.of({
    name: "Swift",
    alias: ["swift"],
    load: () =>
      import("@codemirror/legacy-modes/mode/swift").then((m) =>
        legacy(m.swift),
      ),
  }),
  LanguageDescription.of({
    name: "PowerShell",
    alias: ["ps1", "posh"],
    load: () =>
      import("@codemirror/legacy-modes/mode/powershell").then((m) =>
        legacy(m.powerShell),
      ),
  }),
];
