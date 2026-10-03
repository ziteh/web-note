/* @refresh reload */
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { render } from "solid-js/web";
import "./app.css";
import App from "./App.tsx";
import { Toaster } from "./components/ui/sonner.tsx";

const root = document.getElementById("root");
if (!root) throw new Error("missing root element");

const queryClient = new QueryClient();

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <App />
      <Toaster />
    </QueryClientProvider>
  ),
  root,
);
