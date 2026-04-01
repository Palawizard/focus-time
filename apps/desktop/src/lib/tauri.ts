import type { RuntimeHealth } from "../types/runtime";
import { desktopInvoke } from "./desktop-api";

export function getRuntimeHealth() {
  return desktopInvoke<RuntimeHealth>("get_runtime_health");
}
