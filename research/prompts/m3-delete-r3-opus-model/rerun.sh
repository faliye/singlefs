#!/usr/bin/env bash
# m3-delete-r3 攻方腿复跑（副本专用，被攻过零轮）。用法（从仓根，整条脚本经内存包装与线程上限）：
#   nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 5 \
#     bash research/prompts/m3-delete-r3-opus-model/rerun.sh /tmp/claude-1000/m3-delete-r2/tree <还不存在的草稿目录>
# 冻结副本清单 research/prompts/m3-delete-r3-snapshot/crates-sha256.txt。做的事：拷冻结副本的 crates/，打上 m3-delete-r2 攻方的
# core-prototype-r2.diff（这一轮没改 core，只加用例），放进 m3_delete_r3_attack_prototype.rs，跑 harness 档这一个测试目标的全部用例；
# 再跑纯 python 模型 s2_lineage_model.py。输出与本目录 run-e-lines.txt（排序后）、model-output.txt 逐行比。
# 不枚举任何崩溃状态（层 0 枚举函数一次都不调）。草稿目录跑完留着仓副本与编译目录，用完自己删。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repository="$(cd "$here/../../.." && pwd)"
source_root="$1"
scratch="$2"
if [ -e "$scratch" ]; then echo "草稿目录已存在：$scratch" >&2; exit 2; fi
mkdir -p "$scratch/tree"
cp "$source_root/Cargo.toml" "$source_root/Cargo.lock" "$scratch/tree/"
rsync -a --exclude target "$source_root/crates/" "$scratch/tree/crates/"
(cd "$scratch/tree" && patch -p1 < "$repository/research/prompts/m3-delete-r2-opus-model/core-prototype-r2.diff" > /dev/null)
cp "$here/m3_delete_r3_attack_prototype.rs" "$scratch/tree/crates/singlefs-harness/tests/"
(cd "$scratch/tree" && CARGO_TARGET_DIR="$scratch/target" nice -n 19 cargo test --offline -p singlefs-harness --test m3_delete_r3_attack_prototype -- --nocapture --test-threads 1) > "$scratch/cargo.log" 2>&1
grep -o 'E name=.*' "$scratch/cargo.log" | sort > "$scratch/run-e-lines.txt"
M="$here/s2_lineage_model.py"
{
  python3 "$M" named
  FILE_UNITS=2 python3 "$M" g2 LC,LT
  python3 "$M" matrix 6 LC,LO,LT
  CHECK_CAP=product python3 "$M" matrix 6 LC none,close_early
  CAP=1 ROOTCAP=3 python3 "$M" matrix 6 LC,LT none
  python3 "$M" matrix 7 LC,LO none
} > "$scratch/model-output.txt"
sort "$here/run-e-lines.txt" | diff - "$scratch/run-e-lines.txt" && echo "run-e-lines 相同"
diff "$here/model-output.txt" "$scratch/model-output.txt" && echo "model-output 相同"
