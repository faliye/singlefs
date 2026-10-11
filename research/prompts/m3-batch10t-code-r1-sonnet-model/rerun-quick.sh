#!/usr/bin/env bash
# 正推腿 m3-batch10t-code-r1 的快速复跑（harness 档，不要显卡）。草稿副本在 $S/repo（冻结副本去掉 target 的拷贝，
# 加了 crates/singlefs-harness/tests/ 下的几份探针与 scan_copy.rs，walk.rs 末尾加了 chain_record_view 的垫片）。
# 经 `bash scripts/hosts/run-with-memory-cap.sh 4G bash <这份脚本>` 跑；输出落到调用方给的文件。
set -uo pipefail
S="$HOME/.local/share/singlefs/work/m3-batch10t-code-r1-sonnet"
export TMPDIR="$S/tmp" CARGO_TARGET_DIR="$S/target"
R=/home/fy5090/code/singlefs/scripts/kb/replace-once.py
cd "$S/repo" || exit 2
run() { nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 8 cargo test --offline -p singlefs-harness "$@" 2>&1 | grep -E 'PROBE|test result|FAILED|panicked at' | cut -c1-230; }
echo "== A 基线：harness 档 gpu_memory_ledger 单测"; run --lib -- gpu_memory_ledger
LEDGER=crates/singlefs-harness/src/gpu_memory_ledger.rs
cp "$LEDGER" "$S/ledger.orig.rs"
echo "== B 变异：容量那一项「估计减驱动」换成 u64::MAX（变异表里没有点名它的行）"
python3 "$R" "$LEDGER" '                .min(
                    memory_all_registered_processes_may_hold_mebibytes
                        .saturating_sub(mebibytes_left_for_the_driver),
                ),
        ),
        None =>' '                .min(u64::MAX),
        ),
        None =>' | tail -1
echo "-- B1 变异后：gpu_memory_ledger 单测"; run --lib -- gpu_memory_ledger
echo "-- B2 变异后：探针 probe_f"; run --test k_forward_ledger_probe -- probe_f
cp "$S/ledger.orig.rs" "$LEDGER"; sha256sum "$LEDGER" | cut -c1-64
WALK=crates/singlefs-checker/src/walk.rs
cp "$WALK" "$S/walk.with-shim.rs"
echo "== C K3-1 CPU 差分：实现 InstanceTableChainRecord::parse 对 checker chain_record_view"; run --test k3_chain_record_differential_probe -- --nocapture
echo "-- C2 变异（checker 不看 extent 偏移）后同一探针"
python3 "$R" "$WALK" '        && pointer.is_extent_offset_zero
}' '        && true
}' | tail -1
run --test k3_chain_record_differential_probe -- --nocapture
cp "$S/walk.with-shim.rs" "$WALK"
echo "== D K4-2 扫源码探针"; run --test k4_scan_evasions_probe -- probe_ --nocapture --test-threads 1
