import { Route, Router } from "@solidjs/router";
import Layout from "./components/Layout.tsx";

export default function App() {
  return (
    <Router>
      <Route path={["/", "/n/:id"]} component={Layout} />
    </Router>
  );
}
