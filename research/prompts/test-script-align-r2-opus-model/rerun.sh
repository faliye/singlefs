#!/usr/bin/env bash
# 复跑第二轮攻方（Opus）U1/U2 探针。用法：rerun.sh <草稿目录>
# 只喂本仓 .claude/hooks/heavy-test-guard.sh 的 JSON、看退出码；被判命令一条都不起，主工作区只读。
# 坏树（改过的 stage-selftest.sh 拷贝、假 rustc 的 54 样本根）都建在 <草稿目录> 下。
set -uo pipefail
S="$1"; here="$(cd "$(dirname "$0")" && pwd)"; mkdir -p "$S"
bash "$here/probes.sh" "$S/run" | tee "$S/probes.out"
bash "$here/wild-probes.sh" "$S/wild" | tee "$S/wild-probes.out"
