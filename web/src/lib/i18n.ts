import { createSignal } from "solid-js";
import en from "~/locales/en";
import zhHant from "~/locales/zh-Hant";

const KEY = "locale";

const MESSAGES = { "zh-Hant": zhHant, en };
const DATE_LOCALES = { "zh-Hant": "zh-TW", en: "en-US" };

export type Locale = keyof typeof MESSAGES;
export type MessageKey = keyof typeof zhHant;

const isLocale = (value: string | null): value is Locale =>
  value !== null && value in MESSAGES;

const initial = (): Locale => {
  const stored = localStorage.getItem(KEY);
  if (isLocale(stored)) return stored;
  return navigator.language.startsWith("zh") ? "zh-Hant" : "en";
};

const [locale, setLocale] = createSignal<Locale>(initial());

const apply = (value: Locale) => {
  document.documentElement.lang = value;
};

apply(locale());

export { locale };

export const dateLocale = () => DATE_LOCALES[locale()];

export function toggleLocale() {
  const next: Locale = locale() === "en" ? "zh-Hant" : "en";
  setLocale(next);
  localStorage.setItem(KEY, next);
  apply(next);
}

export function t(key: MessageKey, params?: Record<string, string | number>) {
  const message: string = MESSAGES[locale()][key];
  if (!params) return message;
  return message.replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? ""));
}
