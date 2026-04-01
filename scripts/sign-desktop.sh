#!/usr/bin/env bash
set -euo pipefail

target_path="${1:-}"

if [[ -z "$target_path" ]]; then
  echo "sign-desktop.sh expects the bundle path as its first argument" >&2
  exit 1
fi

if [[ -z "${FOCUS_TIME_SIGN_COMMAND:-}" ]]; then
  echo "No FOCUS_TIME_SIGN_COMMAND configured. Skipping signing for: $target_path"
  exit 0
fi

if [[ "$FOCUS_TIME_SIGN_COMMAND" != *"%1"* ]]; then
  echo "FOCUS_TIME_SIGN_COMMAND must contain %1 as the bundle path placeholder" >&2
  exit 1
fi

resolved_command="${FOCUS_TIME_SIGN_COMMAND//%1/$target_path}"

echo "Running signing command for: $target_path"
sh -lc "$resolved_command"
