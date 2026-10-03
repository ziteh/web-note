import { A } from "@solidjs/router";
import {
  LOCALES,
  type Locale,
  locale,
  localeName,
  setLocale,
  t,
} from "~/lib/i18n";
import {
  SYNTAX_THEMES,
  type SyntaxTheme,
  setSyntaxTheme,
  setUiTheme,
  syntaxTheme,
  UI_THEMES,
  type UiTheme,
  uiTheme,
} from "~/lib/theme";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

const NAMES: Record<string, string> = {
  stone: "Stone",
  blue: "Blue",
  github: "GitHub",
  solarized: "Solarized",
  one: "One",
};

const themeLabel = (value: string) =>
  value === "default" ? t("themeDefault") : NAMES[value];

function OptionSelect<T extends string>(props: {
  title: string;
  label: (value: T) => string;
  options: readonly T[];
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <section class="flex flex-col gap-2">
      <h2 class="font-semibold">{props.title}</h2>
      <Select
        options={[...props.options]}
        value={props.value}
        onChange={(v) => v && props.onChange(v)}
        disallowEmptySelection
        itemComponent={(p) => (
          <SelectItem item={p.item}>{props.label(p.item.rawValue)}</SelectItem>
        )}
      >
        <SelectTrigger>
          <SelectValue<T>>
            {(state) => props.label(state.selectedOption())}
          </SelectValue>
        </SelectTrigger>
        <SelectContent />
      </Select>
    </section>
  );
}

export default function SettingsPage() {
  return (
    <div class="mx-auto flex h-full max-w-2xl flex-col gap-6 overflow-y-auto p-4">
      <header class="flex items-center gap-2">
        <A href="/" class="text-sm text-muted-foreground hover:underline">
          {t("back")}
        </A>
        <h1 class="text-xl font-bold">{t("settings")}</h1>
      </header>
      <OptionSelect<Locale>
        title={t("language")}
        label={localeName}
        options={LOCALES}
        value={locale()}
        onChange={setLocale}
      />
      <OptionSelect<UiTheme>
        label={themeLabel}
        title={t("uiTheme")}
        options={UI_THEMES}
        value={uiTheme()}
        onChange={setUiTheme}
      />
      <OptionSelect<SyntaxTheme>
        label={themeLabel}
        title={t("syntaxTheme")}
        options={SYNTAX_THEMES}
        value={syntaxTheme()}
        onChange={setSyntaxTheme}
      />
    </div>
  );
}
