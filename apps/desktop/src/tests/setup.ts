// noinspection JSUnusedGlobalSymbols
import "@testing-library/jest-dom/vitest";

Object.defineProperty(window, "localStorage", {
  writable: true,
  value: {
    getItem: () => null,
    setItem: () => undefined,
    removeItem: () => undefined,
    clear: () => undefined,
  },
});

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: query.includes("dark"),
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  }),
});

Object.defineProperty(window, "Notification", {
  writable: true,
  value: class NotificationMock {
    static permission = "granted";

    static requestPermission() {
      return Promise.resolve("granted");
    }
  },
});
