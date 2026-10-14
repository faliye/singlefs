#!/usr/bin/env bash
# 复跑前两轮腿的模型目录（T6：判决里写的「八行 same」「五行 same」「九行八 same」）。
# 模型目录先拷进草稿目录再跑（拷贝不在这里做）；每条经内存包装；三条各自的 --work 要是新目录。
# 用法：bash rerun-legs.sh <仓根> <工作根 ~/.local/share/singlefs/work> <模型拷贝所在目录> <新草稿目录>
set -euo pipefail
repo="$1"; work_root="$2"; models="$3"; draft="$4"
cap="bash $work_root/m3-defs-b3-r3/repo/scripts/hosts/run-with-memory-cap.sh 4G"
$cap bash "$models/m3-defs-b3-r1-opus-model/tools/rerun.sh" --repo "$repo" --frozen "$work_root/m3-defs-b3-r1/repo" --work "$draft/rr-r1-opus"
FROZEN_ROUND1="$work_root/m3-defs-b3-r1/repo" $cap bash "$models/m3-defs-b3-r2-opus-model/tools/rerun.sh" --repo "$repo" --frozen "$work_root/m3-defs-b3-r2/repo" --work "$draft/rr-r2-opus"
BEFORE_FROZEN="$work_root/m3-defs-b3-r1/repo" $cap bash "$models/m3-defs-b3-r2-sonnet-model/tools/rerun.sh" --repo "$repo" --frozen "$work_root/m3-defs-b3-r2/repo" --work "$draft/rr-r2-sonnet"
