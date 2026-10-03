#!/usr/bin/env bash
# 复跑这条腿的四个探针。用法：rerun.sh <草稿目录>（在里面建仓拷贝 repo，不碰主工作区；仓拷贝删掉 multi-host.env，免得连第二台）。
# 复跑挂钟（2026-10-01 本机）：拷仓约 1 分钟；hook-probes 约 20 秒；entry-probe 几秒；mutate-probe 约 15 秒；
# light-selftest-scan 并行 6 件约 8 分钟（72 件各自挂钟合计 470 秒）。scan 之前删掉仓拷贝里 samples-heavy 三道的样本目录，让样本档那一件走得通。
set -uo pipefail
S="$1"; here="$(cd "$(dirname "$0")" && pwd)"; main="$(cd "$here/../../.." && pwd)"
mkdir -p "$S"
rsync -a --exclude target --exclude .git "$main/" "$S/repo/"
rm -f "$S/repo/multi-host.env"
bash "$here/hook-probes.sh" "$S/repo" "$S" > "$S/hook-probes.out" 2>&1
bash "$here/entry-probe.sh" "$S/repo" > "$S/entry-probe.out" 2>&1
bash "$here/mutate-probe.sh" "$S/repo" > "$S/mutate-probe.out" 2>&1
rm -rf "$S/repo/.claude/gate.d/fixtures/checker-tier-crates-mutation-replay.sh" "$S/repo/.claude/gate.d/fixtures/checker-tier-formal-verification.sh" \
       "$S/repo/.claude/gate.d/fixtures/checker-tier-research-build-and-replay.sh"
bash "$here/light-selftest-scan.sh" "$S/repo" "$S/scan" 6 > "$S/scan-table.txt" 2>&1
python3 "$here/scan-summary.py" "$S/scan" "$S/scan-table.txt" > "$S/scan-summary.txt"
