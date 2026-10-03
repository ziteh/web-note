import { createSignal } from "solid-js";

const MODE_KEY = "theme";
const UI_KEY = "ui-theme";
const SYNTAX_KEY = "syntax-theme";

export const UI_THEMES = ["default", "stone", "blue", "one"] as const;
export const SYNTAX_THEMES = ["default", "github", "solarized", "one"] as const;

export type UiTheme = (typeof UI_THEMES)[number];
export type SyntaxTheme = (typeof SYNTAX_THEMES)[number];

const initialMode = () =>
  localStorage.getItem(MODE_KEY) ??
  (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");

const stored = <T extends string>(key: string, options: readonly T[]): T => {
  const value = localStorage.getItem(key);
  return options.find((o) => o === value) ?? options[0];
};

const [isDark, setIsDark] = createSignal(initialMode() === "dark");
const [uiTheme, setUiThemeSignal] = createSignal(stored(UI_KEY, UI_THEMES));
const [syntaxTheme, setSyntaxThemeSignal] = createSignal(
  stored(SYNTAX_KEY, SYNTAX_THEMES),
);

const apply = (dark: boolean) => {
  document.documentElement.classList.toggle("dark", dark);
  document.documentElement.style.colorScheme = dark ? "dark" : "light";
};

const applyAttr = (name: string, value: string) => {
  if (value === "default") delete document.documentElement.dataset[name];
  else document.documentElement.dataset[name] = value;
};

apply(isDark());
applyAttr("theme", uiTheme());
applyAttr("syntax", syntaxTheme());

export { isDark, syntaxTheme, uiTheme };

export function toggleTheme() {
  const next = !isDark();
  setIsDark(next);
  localStorage.setItem(MODE_KEY, next ? "dark" : "light");
  apply(next);
}

export function setUiTheme(value: UiTheme) {
  setUiThemeSignal(value);
  localStorage.setItem(UI_KEY, value);
  applyAttr("theme", value);
}

export function setSyntaxTheme(value: SyntaxTheme) {
  setSyntaxThemeSignal(value);
  localStorage.setItem(SYNTAX_KEY, value);
  applyAttr("syntax", value);
}
