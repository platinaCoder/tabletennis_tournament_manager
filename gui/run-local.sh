#!/usr/bin/env bash
set -euo pipefail

script_directory="$(cd -- "$(dirname -- "$0")" && pwd)"
project_directory="$(dirname -- "$script_directory")"

if ! command -v trunk >/dev/null 2>&1; then
  exec nix-shell "$project_directory/shell.nix" --run "$script_directory/run-local.sh"
fi

cd -- "$script_directory"
unset NO_COLOR
exec trunk serve --open
