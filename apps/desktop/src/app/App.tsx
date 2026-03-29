import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HashRouter } from "react-router-dom";

import { AppShell } from "./AppShell";
import { useApplyTheme } from "../hooks/useApplyTheme";

const queryClient = new QueryClient();

function AppRoot() {
  useApplyTheme();

  return <AppShell />;
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <AppRoot />
      </HashRouter>
    </QueryClientProvider>
  );
}
