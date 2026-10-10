#!/usr/bin/env bash
# 正推腿（m3-batch28-code-r1-sonnet）的复跑脚本：一次只起一条 cargo，只跑 harness 档，GPU 一概不跑。
#   SINGLEFS_REPO=<冻结副本树根（有 Cargo.toml 与 crates/）> [SINGLEFS_TOOLS=<有 research/scripts 的仓根，默认从本脚本位置推>] \
#     [RERUN_BUDGET_SECONDS=<墙钟预算，默认 0 = 不限>] bash rerun.sh <目录> all
#       目录第一次要空；预算用完在单元边界退 75，再跑同一条命令续（已完成的单元不重做）；全做完退 0 并打印汇总。
#   bash rerun.sh compare <summary-a> <summary-b>     # 两份 summary 各自排序后逐行比，自己不依赖 SINGLEFS_REPO
# 单元（按次序）：
#   truth   h-v-truth-table.py：H-V 在两份副本全部状态上 CPU 新写法 / GPU 新写法（照 wgsl 抄）/ 条款规格 / CPU 旧写法的真值表（不碰盘）
#   new     新代码上 19 条 harness 测试（13 条新坏镜像 + 6 条第 26 批的陈旧副本那几条）逐条 ok / FAILED
#   old     把本批对 walk.rs 的改动反向打回去（walk-rs-of-batch28.patch，就地、跑完还原）再跑同样 19 条
#   rows    本批 crates/mutations.tsv 里 20 行 CPU 变异（第 2883、2930–2935、2960–2972 行）逐条施加、跑、还原（第 2973–2976 行要 GPU，记 SKIP）
#   extra   extra-mutations.tsv 的变异逐条施加、跑、还原（X11–X14 起整份 checker_known_bad_images 二进制，慢）
#   legit   合法流上的 9 个 harness 测试二进制（新池建文件、删除、回退、容器树长高）整份跑，LEGIT 行记每个二进制过了几条
# 编译经 run-with-memory-cap.sh 6G，跑经它 4G，线程 RERUN_THREADS（默认 15，capped.sh）；一份树副本、一个编译目录，都在 <目录> 里。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${1:-}" == compare ]]; then
    a="$(mktemp)"; b="$(mktemp)"
    LC_ALL=C sort "$2" > "$a"; LC_ALL=C sort "$3" > "$b"
    if diff "$a" "$b"; then echo "compare: 两份 summary 排序后逐行相同（$(wc -l < "$a") 行）"; rc=0; else rc=1; fi
    rm -f "$a" "$b"; exit "$rc"
fi

out="${1:?用法：rerun.sh <目录> all}"; mode="${2:-all}"
[[ "$mode" == all ]] || { echo "只认 all" >&2; exit 2; }
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO=<冻结副本树根>}"
tools="${SINGLEFS_TOOLS:-$(cd "$here/../../.." && pwd)}"
[[ -f "$tools/research/scripts/run-with-memory-cap.sh" ]] || { echo "找不到 $tools/research/scripts/run-with-memory-cap.sh，设 SINGLEFS_TOOLS" >&2; exit 2; }
mkdir -p "$out"
out="$(cd "$out" && pwd)"
budget="${RERUN_BUDGET_SECONDS:-0}"
threads="${RERUN_THREADS:-15}"   # 线程上限（派发提示给的数；2026-10-06 主 agent 改成 15）
started="$(date +%s)"
deadline=0; (( budget > 0 )) && deadline=$(( started + budget ))
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-1500}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='上限按派发提示给的编译 6G、跑 4G，高于峰值表里同键实测的 2 倍也照给'
tree="$out/tree"; target="$out/target"; pristine="$out/pristine"; walk=crates/singlefs-checker/src/walk.rs
filters=(carries_only_a is_one_off_over agreeing_birth_trees verifying_as_a_node_on_one_device stale_copy_of_the_child child_that_does_not_read_back)
mkdir -p "$out/done" "$out/mut"

out_of_time() { (( deadline > 0 )) && (( $(date +%s) > deadline )); }
stop_if_out_of_time() { if out_of_time; then echo "预算 ${budget}s 用完，在单元边界停；再跑同一条命令续" >&2; exit 75; fi; }

# 树副本只建一次；断过的话 walk.rs 先还原成冻结副本里的原样
if [[ ! -d "$tree" ]]; then
    [[ -z "$(ls -A "$out" | grep -v -E '^(done|mut)$' || true)" ]] || { echo "$out 不是空目录" >&2; exit 2; }
    cp -a "$SINGLEFS_REPO" "$tree"; mkdir -p "$pristine"; cp -a "$SINGLEFS_REPO/$walk" "$pristine/walk.rs"
fi
cp "$pristine/walk.rs" "$tree/$walk"; touch "$tree/$walk"
cmp -s "$tree/$walk" "$SINGLEFS_REPO/$walk" || { echo "树副本里的 walk.rs 与冻结副本不同" >&2; exit 3; }

# 还原用新 mtime 的拷贝（cp -a 会带回旧 mtime，cargo 按 mtime 判新旧、会把变异那一版的编译产物当成现成的）
restore_walk() { cp "$pristine/walk.rs" "$tree/$walk"; touch "$tree/$walk"; }

cargo_step() {   # cargo_step <上限> <键> <cargo 参数…>
    local cap="$1" key="$2"; shift 2
    CARGO_TARGET_DIR="$target" RUN_WITH_MEMORY_CAP_KEY="m3b28-sonnet-$key" \
        nice -n 19 bash "$tools/research/scripts/capped.sh" "$threads" bash "$tools/research/scripts/run-with-memory-cap.sh" "$cap" \
        cargo test --offline --manifest-path "$tree/Cargo.toml" "$@"
}

run_suite() {   # run_suite <标签>：19 条测试逐条列 ok / FAILED
    local label="$1"
    cargo_step 6G compile -p singlefs-harness --test checker_known_bad_images --no-run > "$out/$label.build.log" 2>&1
    set +e
    cargo_step 4G run -p singlefs-harness --test checker_known_bad_images -- "${filters[@]}" > "$out/$label.run.log" 2>&1
    rc=$?; echo "$rc" > "$out/$label.rc"
    set -e
    if (( rc >= 250 && rc <= 254 )); then echo "包装退 $rc（那一次的输出不算结果）" >&2; exit 4; fi
    grep -E '^test .* \.\.\. (ok|FAILED)$' "$out/$label.run.log" | sed -E "s/^test (\S+) \.\.\. (ok|FAILED)$/SUITE\t$label\t\1\t\2/" | LC_ALL=C sort > "$out/$label.suite.txt"
}

unit_truth() { python3 "$here/h-v-truth-table.py" > "$out/truth-table.txt"; }
unit_new() { run_suite new; }
unit_old() {
    (cd "$tree" && patch -R -p1 -s < "$here/walk-rs-of-batch28.patch")
    run_suite old
    restore_walk
}
repo_rows="2883,2930,2931,2932,2933,2934,2935,2960,2961,2962,2963,2964,2965,2966,2967,2968,2969,2970,2971,2972,2973,2974,2975,2976"
nextra="$(grep -c . "$here/extra-mutations.tsv")"
extra_rows="$(seq -s, 1 "$nextra")"
mutrun() { python3 "$here/mutrun.py" --tree "$tree" --target "$target" --tools "$tools" --outdir "$out/mut" --deadline "$deadline" --threads "$threads" "$@"; }
unit_rows() { mutrun --table "$tree/crates/mutations.tsv" --rows "$repo_rows"; }
unit_extra() { mutrun --table "$here/extra-mutations.tsv" --rows "$extra_rows"; }

legit_tests=(new_pool_file_creation_publish unlink_single_file_publishes pool_checker_judges_the_deletion_state_of_every_walked_version rollback_lineage_record_and_its_checkpoint_cost_term container_trees_grow_a_level_when_the_root_is_full deletion_scenario_writes unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches unlink_mount_continuation_and_the_emptied_version test_only_switches_force_every_branch)
unit_legit() {   # 合法流（新池建文件、删除、回退、容器树长高）上的 9 个 harness 测试二进制，全绿 = 新写法没有在合法状态上误红
    local flags=() t
    for t in "${legit_tests[@]}"; do flags+=(--test "$t"); done
    cargo_step 6G compile -p singlefs-harness "${flags[@]}" --no-run > "$out/legit.build.log" 2>&1
    set +e
    cargo_step 4G run -p singlefs-harness "${flags[@]}" > "$out/legit.run.log" 2>&1
    rc=$?; echo "$rc" > "$out/legit.rc"
    set -e
    if (( rc >= 250 && rc <= 254 )); then echo "包装退 $rc（那一次的输出不算结果）" >&2; exit 4; fi
    awk '/Running tests\// {name=$2} /^test result:/ {printf "LEGIT\t%s\t%s %s %s %s %s\n", name, $3, $4, $5, $6, $7}' "$out/legit.run.log" | LC_ALL=C sort > "$out/legit.txt"
}

for unit in truth new old rows extra legit; do
    [[ -f "$out/done/$unit" ]] && continue
    stop_if_out_of_time
    echo "== 单元 $unit"
    set +e; "unit_$unit"; rc=$?; set -e
    restore_walk
    (( rc == 75 )) && exit 75
    (( rc == 0 )) || { echo "单元 $unit 退 $rc" >&2; exit "$rc"; }
    touch "$out/done/$unit"
done

{
    cat "$out/new.suite.txt" "$out/old.suite.txt"
    cat "$out"/mut/*.txt | grep '^ROW' || true
    cat "$out/legit.txt"
} | LC_ALL=C sort > "$out/summary.txt"
cat "$out/truth-table.txt"
echo "SUITE new: $(grep -c 'SUITE.new.*ok$' "$out/summary.txt") ok / $(grep -c 'SUITE.new.*FAILED$' "$out/summary.txt") FAILED；old: $(grep -c 'SUITE.old.*ok$' "$out/summary.txt") ok / $(grep -c 'SUITE.old.*FAILED$' "$out/summary.txt") FAILED"
grep -h '^BASE' "$out"/mut/BASE-*.txt | cut -f1-4 | sort
grep '^ROW' "$out/summary.txt" | cut -f2,3,5,6,7 | sort -k1,1 -k2,2n
echo "summary: $(wc -l < "$out/summary.txt") 行 -> $out/summary.txt"
