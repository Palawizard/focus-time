import { render, screen } from "@testing-library/react";

import { App } from "../app/App";

vi.mock("../lib/tauri", () => ({
  getRuntimeHealth: vi.fn().mockResolvedValue({
    productName: "Focus Time",
    desktopShell: "Tauri v2",
    platform: "windows-x86_64",
    persistenceMode: "sqlite-planned",
    workspaceCrates: [
      "focus-domain",
      "focus-persistence",
      "focus-stats",
      "focus-tracking",
    ],
  }),
}));

describe("App", () => {
  it("renders the Focus Time shell", async () => {
    render(<App />);

    expect(screen.getByText("Focus Time")).toBeInTheDocument();
    expect(
      screen.getByText(/Pomodoro, app tracking et statistiques/i),
    ).toBeInTheDocument();
  });
});
