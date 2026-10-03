import { createSignal } from "solid-js";

const KEY = "theme";

const initial = () =>
  localStorage.getItem(KEY) ??
  (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");

const [isDark, setIsDark] = createSignal(initial() === "dark");

const apply = (dark: boolean) => {
  document.documentElement.classList.toggle("dark", dark);
  document.documentElement.style.colorScheme = dark ? "dark" : "light";
};

apply(isDark());

export { isDark };

export function toggleTheme() {
  const next = !isDark();
  setIsDark(next);
  localStorage.setItem(KEY, next ? "dark" : "light");
  apply(next);
}
