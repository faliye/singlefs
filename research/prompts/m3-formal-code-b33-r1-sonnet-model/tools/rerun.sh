#!/usr/bin/env bash
# m3-formal-code-b33-r1 正推腿的复跑入口。用法：
#   REPO=<主工作区根> FROZEN=/tmp/claude-1000/m3-formal-code-b33-r1/tree WORK=<自己的草稿目录> [CAP=10] [MEM=8G] bash rerun.sh <段名…|all>
# 段名：setup static base p1 p2 p3 p4 reuse sweep sweep-slow
# 全是 harness 档：不跑 checker 档包 singlefs-checker-tier 的任何东西。cargo 与编出来的二进制都经内存包装与线程上限（lib.sh）。
# 每份副本用自己的 CARGO_TARGET_DIR（$WORK/target）；每段日志在 $WORK/log/<名>.log，汇总行打到终端；
# 「编完核日志里被测包确有 Compiling」：lib.sh 的 cg 在每段末尾打 compiled: 行，没有 Compiling singlefs-* 的那一趟不作数。
# 段内顺序都是先基线再变异；变异经 mut.py 施加与撤回（撤回后 touch，cargo 才重编）。
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$HERE/lib.sh"
: "${FROZEN:?FROZEN 是冻结副本根}"
row() { # row <行号> [日志后缀]：用表里这一行自己的 cargo 参数与变异
    local n=$1 args
    args=$(awk -F'\t' -v n="$n" 'NR==n{print $5}' "$TREE/crates/mutations.tsv")
    # shellcheck disable=SC2086
    HEAD=4 mutrun "row$n${2:-}" "$n" test --offline $args
}
var() { local v=$1; shift; HEAD=4 mutrun "var-$v" "$MODEL_DIR/variants/$v.spec" test --offline "$@"; }

stage_setup() {
    if [[ ! -d "$TREE" ]]; then mkdir -p "$WORK" && cp -a "$FROZEN" "$TREE"; fi
    # 冻结副本只有 crates/ 与两份 Cargo 文件；有几条用例读 ../../litmus，从主工作区拷一份（litmus 不在这一轮的改动里）
    [[ -d "$TREE/litmus" ]] || cp -a "$REPO/litmus" "$TREE/litmus"
    ( cd "$TREE" && sha256sum -c "$(dirname "$FROZEN")/crates-sha256.txt" 2>&1 | grep -vc ': OK$' | sed 's/^/sha_not_ok=/' )
}
stage_static() {
    ( cd "$REPO" && python3 research/scripts/crates-mutation-rows.py static-check "$TREE" | tail -3 )
    python3 "$MODEL_DIR/tools/p6_rows.py" "$TREE" "$REPO/research/prompts/_m3-formal-code-b33-r1-diff.md"
}
stage_base() {
    cg base-lib test --offline -p singlefs-harness --lib
    cg base-kbi test --offline -p singlefs-harness --test checker_known_bad_images
    cg base-overwrite test --offline -p singlefs-harness --test overwrite_in_one_instance
    cg base-turn test --offline -p singlefs-harness --test root_ring_turn_in_one_mount
}
stage_p1() { # B3 / B4：入库三行 + 取样点之外的变体（封顶 / 只取回 / 只保留 / 分配代错一版 / 只在装片数 60–118 时错）
    row 2774; row 2775; row 2776
    for v in glue-cap-5 glue-cap-6 glue-cap-20 glue-cap-60 glue-cap-100 glue-cap-133 glue-cap-134 glue-swap-77-78 glue-mod-64 \
             b4-take-up-to-5 b4-take-up-to-6 b4-take-up-to-20 b4-take-up-to-100 b4-take-up-to-133 b4-take-up-to-134 \
             b4-keep-current-from-5 b4-keep-current-from-6 b4-keep-current-from-100 b4-written-at-wrong-at-3 b4-count-window-60-118; do
        var "$v" -p singlefs-harness --lib -- every_inode_leaf_container a_rollback_takes_exactly
    done
}
stage_p2() { for n in 2777 2778 2779 2780 2781 2782 2783 2821 2822; do row "$n"; done; }
stage_p3() {
    for n in 2788 2789 2790 2791 2793 2794; do row "$n"; done
    # U11 那一行在 checker 档、不跑；同一处变异（2792 的原文与替换文）在 harness 档上量机理：哪些断言红
    HEAD=6 mutrun row2792-harness-probe 2792 test --offline --no-fail-fast -p singlefs-harness --test overwrite_in_one_instance --test root_ring_turn_in_one_mount
    # R4、d 在 812 那份用例下：R4 全绿（只靠 root_ring_turn 那条抓），d 红在 txg 4
    for n in 2790 2791; do HEAD=3 mutrun "r$n-on-812" "$n" test --offline --no-fail-fast -p singlefs-harness --test overwrite_in_one_instance; done
}
stage_p4() { # 33-b 新造法：把用例文件拷成 harness 档探针，基线 + 四行变异
    python3 "$MODEL_DIR/probes/make_probe_rollback_floor.py" "$TREE"
    cg probe-p4-base-all test --offline -p singlefs-harness --test probe_b33_rollback_floor
    for n in 2796 2797 720 1376; do
        HEAD=3 mutrun "probe-p4-row$n" "$n" test --offline -p singlefs-harness --test probe_b33_rollback_floor -- rolling_back_below_a_floor_only_the_system_configuration
    done
    # 旧第 38 行（HEAD 的 crates/mutations.tsv）的替换文在今天的 core 上编不过：应见 error[E0425] cannot find value `newest_root`
    HEAD=5 mutrun var-old-row38-compile "$MODEL_DIR/variants/old-row38-newest-root.spec" build --offline -p singlefs-core
}
stage_reuse() { # 抬 F 慢档那条：整份（含 --ignored）与单条
    cg base-reuse test --offline -p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored
    cg base-reuse-one test --offline -p singlefs-harness --test reuse_after_raising_the_floor -- --ignored --exact \
        raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
}
stage_sweep() { # 合法历史上 F1 / 33-c / 二末节的探针：整个 harness 套件（非慢档）。一次调用有 BUDGET 秒，没跑完再调用一次接着跑
    if [[ ! -f "$WORK/.instrumented" ]]; then
        python3 "$MODEL_DIR/probes/instrument_i54.py" "$TREE" && touch "$WORK/.instrumented"
        cg instr-build test --offline -p singlefs-harness --no-run
        ( cd "$TREE" && cargo test --offline -p singlefs-harness --no-run --message-format=json 2>/dev/null ) | python3 -c "
import sys, json
rows = []
for line in sys.stdin:
    try: m = json.loads(line)
    except ValueError: continue
    if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile'].get('test'):
        rows.append((m['target']['name'], m['executable']))
print(''.join(f'{n}\t{e}\n' for n, e in sorted(rows)), end='')" > "$WORK/bins.tsv"
        wc -l < "$WORK/bins.tsv" | sed 's/^/test_binaries=/'
    fi
    P=${P:-3} CAP=${CAP:-3} MEM=${MEM_SWEEP:-3G} bash "$MODEL_DIR/tools/sweep.sh"
}
stage_sweep_slow() { # 慢档：一个测试一个任务；jobs-ignored.tsv 由 --list --ignored 现生成
    : > "$WORK/jobs-ignored.tsv"
    while IFS=$'\t' read -r name exe; do
        ( cd "$TREE/crates/singlefs-harness" && "$exe" --list --ignored 2>/dev/null ) | grep ': test$' | sed 's/: test$//' \
            | while read -r t; do printf '%s\t%s\t%s\n' "$name" "$exe" "$t" >> "$WORK/jobs-ignored.tsv"; done
    done < "$WORK/bins.tsv"
    P=${P:-1} CAP=${CAP:-10} MEM=${MEM_SWEEP:-6G} BUDGET=${BUDGET:-3000} bash "$MODEL_DIR/tools/sweep-ignored.sh"
}
for s in "$@"; do
    if [[ "$s" == all ]]; then set -- setup static base p1 p2 p3 p4 reuse sweep sweep-slow; break; fi
done
for s in "$@"; do echo "######## $s"; "stage_${s//-/_}"; done
