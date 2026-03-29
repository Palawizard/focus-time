import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { AppShell } from "./AppShell";

const queryClient = new QueryClient();

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppShell />
    </QueryClientProvider>
  );
}
