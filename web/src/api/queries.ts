import { createQuery, useQueryClient } from "@tanstack/solid-query";
import type { Accessor } from "solid-js";
import type { Filters } from "~/lib/filters";
import { api, unwrap } from "./client";

export function useNotes(filters: Accessor<Filters>) {
  return createQuery(() => ({
    queryKey: ["notes", filters()],
    queryFn: () =>
      unwrap(
        api.GET("/api/notes", {
          params: {
            query: {
              notebook: filters().notebook,
              tag: filters().tags,
              q: filters().q,
            },
          },
        }),
      ),
  }));
}

/** Unfiltered list backing wikilink titles and completion. */
export function useAllNotes() {
  return createQuery(() => ({
    queryKey: ["notes", "all"],
    queryFn: () => unwrap(api.GET("/api/notes")),
  }));
}

export function useBacklinks(id: Accessor<string | undefined>) {
  return createQuery(() => ({
    queryKey: ["notes", "backlinks", id()],
    enabled: id() !== undefined,
    queryFn: () =>
      unwrap(
        api.GET("/api/notes/{id}/backlinks", {
          // biome-ignore lint/style/noNonNullAssertion: guarded by `enabled`
          params: { path: { id: id()! } },
        }),
      ),
  }));
}

export function useTrash() {
  return createQuery(() => ({
    queryKey: ["trash"],
    queryFn: () => unwrap(api.GET("/api/trash")),
  }));
}

export function useOrphanAssets() {
  return createQuery(() => ({
    queryKey: ["orphan-assets"],
    queryFn: () => unwrap(api.GET("/api/assets/orphans")),
  }));
}

export function useNote(id: Accessor<string | undefined>) {
  return createQuery(() => ({
    queryKey: ["note", id()],
    enabled: id() !== undefined,
    queryFn: () =>
      unwrap(
        api.GET("/api/notes/{id}", {
          // biome-ignore lint/style/noNonNullAssertion: guarded by `enabled`
          params: { path: { id: id()! } },
        }),
      ),
  }));
}

export function useNotebooks() {
  return createQuery(() => ({
    queryKey: ["notebooks"],
    queryFn: () => unwrap(api.GET("/api/notebooks")),
  }));
}

export function useTags() {
  return createQuery(() => ({
    queryKey: ["tags"],
    queryFn: () => unwrap(api.GET("/api/tags")),
  }));
}

export function useStatus() {
  return createQuery(() => ({
    queryKey: ["status"],
    queryFn: () => unwrap(api.GET("/api/system/status")),
  }));
}

/** Re-walks the notes directory, then refreshes everything shown. */
export function useRescan() {
  const client = useQueryClient();
  return async () => {
    await unwrap(api.POST("/api/system/rescan"));
    await client.invalidateQueries();
  };
}

/** Refreshes everything that derives from note contents after a change. */
export function useRefreshNotes() {
  const client = useQueryClient();
  return () =>
    Promise.all(
      ["notes", "tags", "notebooks"].map((key) =>
        client.invalidateQueries({ queryKey: [key] }),
      ),
    );
}
