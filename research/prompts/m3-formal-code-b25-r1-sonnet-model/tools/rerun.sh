#!/usr/bin/env bash
# 正推腿 m3-formal-code-b25-r1 的复跑脚本：在草稿目录里从冻结树拷一份副本、装上探针、复现报告里的读数。
# 不在冻结树里编、不往里写；只跑 harness 档测试与装置二进制的小规模模式（都经 capped.sh 与 run-with-memory-cap.sh），
# 不跑 checker 档测试、不跑 crates/mutations.tsv 整表。
#
# 用法（工作目录不限）：
#   bash tools/rerun.sh setup     拷副本、装探针、编（约 1 分钟）
#   bash tools/rerun.sh k1        K1：新用例、3 行变异、附加变异、三片叶探针、挂载插进序列的探针、边界探针（共约 8 分钟，分几条命令各 ≤ 4 分钟）
#   bash tools/rerun.sh k2        K2：主取样点冻结二进制一遍、探针二进制（4896 个附加态）、自证会红、V17 至 V19 与告警变异的运行期读数
#   bash tools/rerun.sh k3        K3：28 行锚点、流三的模型判定直方图
# 环境变量：SCRATCH（默认 /tmp/claude-1000/m3-formal-code-b25-r1-sonnet-rerun）、SINGLEFS_REPO（默认 /home/fy5090/code/singlefs）
set -euo pipefail
repo="${SINGLEFS_REPO:-/home/fy5090/code/singlefs}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scratch="${SCRATCH:-/tmp/claude-1000/m3-formal-code-b25-r1-sonnet-rerun}"
frozen="$repo/scripts/experiments/target/e171-r1-frozen-tree"
tree="$scratch/tree"
device="crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="正推腿复跑，与别的会话同时在跑各跑各的"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="草稿副本里的小规模复跑，上限取 8G，峰值表是别处测的"
capped() { nice -n 19 bash "$repo/scripts/hosts/capped.sh" 6 "$@"; }
capped_run() { capped bash "$repo/scripts/hosts/run-with-memory-cap.sh" 8G "$@"; }

setup() {
    mkdir -p "$scratch/out"
    [ -d "$tree" ] || rsync -a --exclude=target "$frozen/" "$tree/"
    cp -n "$here/probes/k1_probe.rs" "$here/probes/k1_boundary_core.rs" "$here/probes/k1_three_leaves.rs" "$tree/crates/singlefs-harness/tests/"
    if ! grep -q 'mod k1_boundary_probe' "$tree/crates/singlefs-harness/src/model.rs"; then
        cat "$here/probes/k1_boundary_probe_appended_to_model_rs.txt" >> "$tree/crates/singlefs-harness/src/model.rs"
    fi
    if ! grep -q 'PROBE_UNIT' "$tree/$device"; then
        patch "$tree/$device" < "$here/probes/e171_device_probe.patch"
    fi
    [ -f "$scratch/e171-frozen-bin" ] || cp "$frozen/target/release/e171_three_oracle_collusion_after_the_oracle_fixes" "$scratch/e171-frozen-bin"
    (cd "$tree" && mkdir -p target && [ -d target/release ] || cp -a "$frozen/target/release" target/)
    (cd "$tree" && capped cargo build --release --offline -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes)
    (cd "$tree" && capped cargo build --offline -p singlefs-harness --tests)
    cp "$tree/target/release/e171_three_oracle_collusion_after_the_oracle_fixes" "$scratch/e171-probe-bin"
}

k1() {
    cd "$tree"
    # 25-1 新用例，三片叶的变体，挂载与冷启动恢复插进序列，边界（实现的纯函数与模型的式子）
    capped_run cargo test --offline -p singlefs-harness --test model_compares_inode_creations_across_two_inode_leaf_containers
    capped_run cargo test --offline -p singlefs-harness --test k1_three_leaves
    capped_run cargo test --offline -p singlefs-harness --test k1_probe -- --nocapture --test-threads=2
    capped_run cargo test --offline -p singlefs-harness --test k1_boundary_core -- --nocapture
    capped_run cargo test --offline -p singlefs-harness --lib k1_boundary_probe -- --nocapture
    # 3 行变异（只跑点名的测试），再加我自造的 KX1–KX8（锚点与替换文在 k1-extra-mutations*.tsv）
    export LOG_DIR="$scratch/logs"
    python3 "$here/tools/mutate_in_scratch.py" "$tree" "$tree/crates/mutations.tsv" 'O1 跨叶容器'
    python3 "$here/tools/mutate_in_scratch.py" "$tree" "$here/tools/k1-extra-mutations.tsv" KX
    python3 "$here/tools/mutate_in_scratch.py" "$tree" "$here/tools/k1-extra-mutations-three-leaves.tsv" KX
    python3 "$here/tools/mutate_in_scratch.py" "$tree" "$here/tools/k1-extra-mutations-leftmost-span.tsv" KX
    # 3 行变异对别的 harness 档测试（--lib、random_histories、unlink_in_histories_judged_by_the_model）红不红（每行约 2 分钟）
    for prefix in 'O1 跨叶容器（理想模型自身）：第二片晚' 'O1 跨叶容器（胶水）' 'O1 跨叶容器（理想模型自身）：新记录落在第二片时'; do
        python3 "$here/tools/mutate_in_scratch.py" "$tree" "$here/tools/k1-rows-collateral.tsv" "$prefix"
    done
}

k2() {
    cd "$scratch"
    capped_run ./e171-frozen-bin stream-constants --streams 1,2 > out/stream-constants-12.txt
    capped_run ./e171-frozen-bin confirmed-loss-form --output-directory out/main-point > out/main-point.txt
    grep -E 'name=(q5 |q5_equivalence|q5_partition|verdict)' out/main-point.txt | cut -c1-400
    # 探针二进制：主取样点真流上 4896 个附加态（每种 K 取法：C 单个落、C 缺一个、20 个随机子集）对类里推出的列
    capped_run ./e171-probe-bin confirmed-loss-form --output-directory out/main-point-probe 2> out/main-point-probe.err > /dev/null
    grep PROBE out/main-point-probe.err
    # 自证会红：把探针里的类键抹掉一位，必须报出不同
    PROBE_BREAK_KEY=1 PROBE_RANDOM_PER_ASSIGNMENT=2 capped_run ./e171-probe-bin confirmed-loss-form --output-directory out/main-point-probe-break 2> out/main-point-probe-break.err > /dev/null
    grep PROBE out/main-point-probe-break.err
    # 内联用例口径的复刻（288 态那一形）：基线
    PROBE_UNIT=1 capped_run ./e171-probe-bin stream-constants > out/probe-unit-baseline.txt
    cat out/probe-unit-baseline.txt
    # V17–V19 在装置运行期的读数（主取样点）
    for v in V17 V18 V19; do
        python3 "$here/tools/runtime_mutation.py" "$tree" "$tree/crates/mutations.tsv" "E171R1 装置 $v" out/main-point.txt confirmed-loss-form --output-directory @OUT@
    done
    # 同三条与告警变异 KM1–KM7 在内联用例口径复刻上的读数
    for v in V17 V18 V19; do
        PROBE_UNIT=1 python3 "$here/tools/runtime_mutation.py" "$tree" "$tree/crates/mutations.tsv" "E171R1 装置 $v" out/probe-unit-baseline.txt stream-constants
    done
    for k in KM1 KM2 KM3 KM4 KM5 KM6 KM7; do
        PROBE_UNIT=1 python3 "$here/tools/runtime_mutation.py" "$tree" "$here/tools/e171-class-alarm-mutations.tsv" "$k" out/probe-unit-baseline.txt stream-constants
    done
}

k3() {
    python3 "$here/tools/check_e171r1_anchors.py" "$frozen"
    cd "$scratch"
    capped_run ./e171-probe-bin stream-constants --streams 1,2,3 2>&1 >/dev/null | grep PROBE
    (cd "$repo" && python3 - <<'PY'
import json, subprocess
m = json.loads(subprocess.check_output(['cargo', 'metadata', '--offline', '--format-version', '1'], cwd='scripts/experiments/target/e171-r1-frozen-tree'))
ws = {p['name']: p for p in m['packages'] if p['source'] is None}
for n in sorted(ws):
    print(n, sorted({d['name'] for d in ws[n]['dependencies']} & set(ws)))
PY
    )
}

case "${1:-}" in
    setup) setup ;;
    k1) k1 ;;
    k2) k2 ;;
    k3) k3 ;;
    *) echo "用法：rerun.sh setup|k1|k2|k3" >&2; exit 2 ;;
esac
