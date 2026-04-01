import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";

type EventPayload<T> = {
  payload: T;
};

type DesktopListen = <T>(
  event: string,
  handler: (event: EventPayload<T>) => void,
) => Promise<() => void>;

interface DesktopBridge {
  invoke: typeof tauriInvoke;
  listen: DesktopListen;
}

declare global {
  interface Window {
    __FOCUS_TIME_E2E_API__?: DesktopBridge;
  }
}

function getDesktopBridge(): DesktopBridge {
  if (typeof window !== "undefined" && window.__FOCUS_TIME_E2E_API__) {
    return window.__FOCUS_TIME_E2E_API__;
  }

  return {
    invoke: tauriInvoke,
    listen: tauriListen as DesktopListen,
  };
}

export function desktopInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
) {
  return getDesktopBridge().invoke<T>(command, args);
}

export function desktopListen<T>(
  event: string,
  handler: (event: EventPayload<T>) => void,
) {
  return getDesktopBridge().listen<T>(event, handler);
}
