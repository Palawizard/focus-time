import { invoke } from "@tauri-apps/api/core";

import type { RuntimeHealth } from "../types/runtime";

export function getRuntimeHealth() {
  return invoke<RuntimeHealth>("get_runtime_health");
}
