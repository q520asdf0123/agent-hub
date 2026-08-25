#!/usr/bin/env bash
# 兼容旧入口；实际部署统一由 PowerShell 完成，避免跨 shell 组合 Windows 进程和文件操作。
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -W)"
if command -v pwsh.exe >/dev/null 2>&1; then
  shell=pwsh.exe
else
  shell=powershell.exe
fi

exec "$shell" -NoLogo -NoProfile -File "$script_dir/deploy-local.ps1" "$@"
