import { Route, Router } from "@solidjs/router";
import Layout from "./components/Layout.tsx";
import SettingsPage from "./components/SettingsPage.tsx";
import TrashPage from "./components/TrashPage.tsx";

export default function App() {
  return (
    <Router>
      <Route path="/trash" component={TrashPage} />
      <Route path="/settings" component={SettingsPage} />
      <Route path={["/", "/n/:id"]} component={Layout} />
    </Router>
  );
}
