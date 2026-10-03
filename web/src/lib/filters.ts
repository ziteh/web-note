import { useLocation, useNavigate } from "@solidjs/router";
import { createMemo } from "solid-js";

export interface Filters {
  notebook?: string;
  tags: string[];
  q?: string;
}

/** Filters live in the URL query string so they survive reloads and links. */
export function useFilters() {
  const location = useLocation();
  const navigate = useNavigate();

  const filters = createMemo<Filters>(() => {
    const params = new URLSearchParams(location.search);
    return {
      notebook: params.get("notebook") ?? undefined,
      tags: params.getAll("tag"),
      q: params.get("q") ?? undefined,
    };
  });

  const update = (patch: Partial<Filters>, to = location.pathname) => {
    const next = { ...filters(), ...patch };
    const params = new URLSearchParams();
    if (next.notebook) params.set("notebook", next.notebook);
    for (const tag of next.tags) params.append("tag", tag);
    if (next.q) params.set("q", next.q);
    const qs = params.toString();
    navigate(qs ? `${to}?${qs}` : to, { replace: true });
  };

  return { filters, update };
}
