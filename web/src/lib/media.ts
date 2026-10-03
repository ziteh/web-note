import { createSignal, onCleanup } from "solid-js";

const DESKTOP_QUERY = "(min-width: 768px)";

export function createIsDesktop() {
  const query = window.matchMedia(DESKTOP_QUERY);
  const [isDesktop, setIsDesktop] = createSignal(query.matches);
  const onChange = (e: MediaQueryListEvent) => setIsDesktop(e.matches);
  query.addEventListener("change", onChange);
  onCleanup(() => query.removeEventListener("change", onChange));
  return isDesktop;
}
