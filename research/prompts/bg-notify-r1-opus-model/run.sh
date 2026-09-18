#!/usr/bin/env bash
# bg-notify-r1 攻方腿的模型复跑：仓里的 hook 与本腿提的变体（hook-variant.sh，被攻过零轮）各跑一遍形态表，
# 再比变体与仓里 hook 在附录三 L1–L14 上第三种检出的判定差几条。只写草稿目录，不写仓里别处。
#   bash research/prompts/bg-notify-r1-opus-model/run.sh [草稿目录]
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
draft="${1:-/tmp/claude-1000/bg-notify-r1-opus}"
mkdir -p "$draft"
export BGN_DRAFT="$draft"
echo "== 仓里的 hook（sha256 $(sha256sum "$repo/.claude/hooks/bash-command-detector.sh" | cut -d' ' -f1)）"
nice -n 19 python3 "$here/probe.py" "$repo" "$draft"
variant_root="$(mktemp -d "$draft/variant-root.XXXXXX")"
mkdir -p "$variant_root/.claude/hooks"
cp "$here/hook-variant.sh" "$variant_root/.claude/hooks/bash-command-detector.sh"
echo
echo "== 变体 hook-variant.sh（sha256 $(sha256sum "$here/hook-variant.sh" | cut -d' ' -f1)；本腿提的改法，被攻过零轮）"
nice -n 19 python3 "$here/probe.py" "$variant_root" "$draft"
echo
echo "== 变体自检"
nice -n 19 bash "$variant_root/.claude/hooks/bash-command-detector.sh" --selftest
echo
echo "== 变体与仓里 hook 在 L1–L14 上"
nice -n 19 python3 "$here/l-regress.py" "$repo/.claude/hooks/bash-command-detector.sh" "$variant_root/.claude/hooks/bash-command-detector.sh"
