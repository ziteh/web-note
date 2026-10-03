import { A } from "@solidjs/router";
import { For, Show } from "solid-js";
import { useNotebooks, useStatus, useTags } from "~/api/queries";
import MoonIcon from "~/assets/moon.svg";
import SunIcon from "~/assets/sun.svg";
import TrashIcon from "~/assets/trash.svg";
import { useFilters } from "~/lib/filters";
import { locale, t, toggleLocale } from "~/lib/i18n";
import { isDark, toggleTheme } from "~/lib/theme";
import { Button, buttonVariants } from "./ui/button";

export default function Sidebar(props: { onPick?: () => void }) {
  const notebooks = useNotebooks();
  const tags = useTags();
  const status = useStatus();
  const { filters, update } = useFilters();

  const pick = (patch: Parameters<typeof update>[0]) => {
    update(patch, "/");
    props.onPick?.();
  };

  const toggleTag = (tag: string) => {
    const current = filters().tags;
    pick({
      tags: current.includes(tag)
        ? current.filter((t) => t !== tag)
        : [...current, tag],
    });
  };

  return (
    <nav class="flex h-full flex-col gap-4 overflow-y-auto p-2">
      <section class="flex flex-col gap-1">
        <h2 class="px-2 text-xs font-semibold text-muted-foreground">
          {t("notebooks")}
        </h2>
        <Button
          variant={filters().notebook ? "ghost" : "secondary"}
          class="justify-start"
          onClick={() => pick({ notebook: undefined })}
        >
          {t("all")}
        </Button>
        <For each={notebooks.data}>
          {(nb) => (
            <Button
              variant={filters().notebook === nb.name ? "secondary" : "ghost"}
              class="justify-between"
              onClick={() => pick({ notebook: nb.name })}
            >
              <span class="truncate">{nb.name}</span>
              <span class="text-xs text-muted-foreground">{nb.count}</span>
            </Button>
          )}
        </For>
      </section>

      <section class="flex flex-col gap-1">
        <h2 class="px-2 text-xs font-semibold text-muted-foreground">
          {t("tags")}
        </h2>
        <For each={tags.data}>
          {(tag) => (
            <Button
              variant={filters().tags.includes(tag.tag) ? "secondary" : "ghost"}
              class="justify-between"
              onClick={() => toggleTag(tag.tag)}
            >
              <span class="truncate">{tag.tag}</span>
              <span class="text-xs text-muted-foreground">{tag.count}</span>
            </Button>
          )}
        </For>
      </section>

      <div class="mt-auto flex items-center justify-between">
        <A
          href="/trash"
          class={buttonVariants({ variant: "ghost", size: "icon" })}
          aria-label={t("trash")}
        >
          <TrashIcon />
        </A>
        <Button
          variant="ghost"
          size="icon"
          onClick={toggleLocale}
          aria-label={t("toggleLanguage")}
        >
          {locale() === "en" ? "EN" : "中"}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={toggleTheme}
          aria-label={t("toggleTheme")}
        >
          {isDark() ? <MoonIcon /> : <SunIcon />}
        </Button>
      </div>

      <Show when={status.data?.errors.length}>
        <p class="px-2 text-xs text-destructive">
          {t("loadErrors", { count: status.data?.errors.length ?? 0 })}
        </p>
      </Show>
    </nav>
  );
}
