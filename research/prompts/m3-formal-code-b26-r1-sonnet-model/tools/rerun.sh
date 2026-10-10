#!/usr/bin/env bash
# 第 26 批代码轮第一轮、云端正推腿：复跑脚本。
# 用法（工作目录随意，路径都写绝对）：
#   FROZEN=/tmp/claude-1000/m3-formal-code-b26-r1/tree \
#   REPO=/home/fy5090/code/singlefs \
#   WORK=<草稿目录>  bash <本目录>/tools/rerun.sh <static|probes|rows|all>
# static：纯读与纯算——#8 的 69 条期望值对日志、#9 的 58 行 SHA-256、变异行锚点、F3-e 种子模型（几秒）。
# probes：把 probe/ 下两份探针追加进冻结副本的拷贝，经 capped.sh 7 与 run-with-memory-cap.sh 12G 跑 harness 档测试（几分钟，不跑 checker 档）。
# rows：把改动全文里 harness 档的 38 条变异行逐条打进拷贝、跑那一行点名的测试（几分钟）。
# 不跑：checker 档测试、crates/mutations.tsv 整表、GPU。
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-code-b26-r1/tree}"
REPO="${REPO:-/home/fy5090/code/singlefs}"
WORK="${WORK:?要给 WORK=<草稿目录>}"
DIFF_MD="$REPO/research/prompts/_m3-formal-code-b26-r1-diff.md"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-草稿副本里单跑取宽上限}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-同一会话下几条腿各跑各的}"
mode="${1:-all}"
mkdir -p "$WORK/logs"

static_part() {
    echo "== #8：69 条不变量的期望评估数对日志 CHECKER 行"
    python3 "$HERE/tools/check_pin8.py" "$FROZEN" "$HERE/in/checker-line-410.txt"
    echo "== #9：打印的 58 行 SHA-256（钉值 7e64a883…）"
    sha256sum "$HERE/in/printed58-golden-lines.txt"
    echo "== 变异行锚点（47 行：append 与 replacements）"
    python3 "$HERE/tools/check_anchors.py" "$DIFF_MD" "$FROZEN" | tail -1
    echo "== F3-e 二次崩溃种子模型"
    python3 "$HERE/tools/f3e_model.py"
}

probes_part() {
    rm -rf "$WORK/probe-tree"; mkdir -p "$WORK/probe-tree"
    rsync -a --exclude target/ "$FROZEN/" "$WORK/probe-tree/"
    cat "$HERE/probe/mount_transparency_probe.rs.append" >> "$WORK/probe-tree/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs"
    cat "$HERE/probe/known_bad_probe.rs.append" >> "$WORK/probe-tree/crates/singlefs-harness/tests/checker_known_bad_images.rs"
    run() { # run <日志名> <cargo test 参数…>
        local log="$1"; shift
        nice -n 19 bash "$REPO/research/scripts/capped.sh" 7 timeout 230 \
            bash "$REPO/research/scripts/run-with-memory-cap.sh" 12G \
            cargo test --offline --manifest-path "$WORK/probe-tree/Cargo.toml" -p singlefs-harness "$@" > "$WORK/logs/$log" 2>&1 || true
        grep -E 'PROBE|test result' "$WORK/logs/$log"
    }
    echo "== 挂载透明：整挂载（7 种底盘面 × 只挂载 / 挂载加发布）与二次崩溃逐写前缀"
    run probe-mt.log --test mount_transparency_judged_field_by_field_after_a_writable_mount probe_b26 -- --nocapture --test-threads=2
    echo "== I-5.4：分配记录树读不出时的判定（新代码）"
    run probe-i54.log --test checker_known_bad_images probe_b26_allocation -- --nocapture
    echo "== I-9.16 ④：树表缺种类 8 / 12 时 checker 报什么、core 可写挂载报什么"
    run probe-i916.log --test checker_known_bad_images probe_b26_tree_table -- --nocapture
    run probe-core.log --test checker_known_bad_images probe_b26_core -- --nocapture
}

rows_part() {
    rm -rf "$WORK/rows-tree"; mkdir -p "$WORK/rows-tree"
    rsync -a --exclude target/ "$FROZEN/" "$WORK/rows-tree/"
    # md 行号：26-2 的 470（C686）；26-3 的 3664-3686；26-1r 的 6084-6092、6106-6110；其余行点名 checker 档，不跑
    python3 "$HERE/tools/run_rows.py" "$DIFF_MD" "$WORK/rows-tree" "$WORK/logs/rows" \
        470 $(seq 3664 3686) $(seq 6084 6092) $(seq 6106 6110) | tee "$WORK/logs/rows-summary.txt"
}

case "$mode" in
    static) static_part ;;
    probes) probes_part ;;
    rows) rows_part ;;
    all) static_part; probes_part; rows_part ;;
    *) echo "用法：rerun.sh <static|probes|rows|all>"; exit 2 ;;
esac
