#!/usr/bin/env bash
# 正推腿（m3-batch29-code-r1-sonnet）的复跑脚本：一次只起一条 cargo，只跑 harness 档，checker 档（含 GPU）一概不跑。
#   SINGLEFS_REPO=<冻结副本树根（有 Cargo.toml 与 crates/）> [SINGLEFS_TOOLS=<有 scripts 的仓根，默认从本脚本位置推>] \
#     [RERUN_THREADS=<线程上限，默认 15>] [RERUN_BUDGET_SECONDS=<墙钟预算，默认 0 = 不限；用完在单元 / 变异行边界退 75，再跑同一条命令续>] bash rerun.sh <目录> all
#       目录第一次要空；全做完退 0 并打印汇总（summary.txt）。
#   bash rerun.sh compare <summary-a> <summary-b>     # 两份 summary 各自排序后逐行比，自己不依赖 SINGLEFS_REPO
# 单元（按次序）：
#   static  crates-mutation-rows.py static-check：第 2977–2989 行（本批追加的 13 行）有没有锚点 / 名字问题；第 2935 行锚点命中几次
#   sites   判不了原因字的站点号：有没有重复，37、38 各出现几次
#   rowcheck 13 行的锚点命中数、点名的测试在测试文件里有没有、参数筛不筛得到
#   consts  两个内核里 `const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;` 各恰好一处（对拍里那条「不用卡」的快测试判的就是这个）
#   new     checker_known_bad_images 整份二进制（含本批 5 条新坏镜像与第 27 批那一条）：每条点名的 ok / FAILED，总数
#   rows    mutations.tsv 本批 7 行 CPU 变异逐条施加、跑、还原（第 2984–2989 行要 checker 档，记 SKIP）
#   extra   extra-mutations.tsv 的 6 行变异（只判第一份、只看最后一份、登记过的类放宽、没登记的只按 32 KiB、读不出算对得上、走读失败取反）
#   legit   合法流（新池建文件、删除、回退、容器树长高）上的 9 个 harness 测试二进制整份跑，LEGIT 行记每个二进制过了几条
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
[[ -f "$tools/scripts/run-with-memory-cap.sh" ]] || { echo "找不到 $tools/scripts/run-with-memory-cap.sh，设 SINGLEFS_TOOLS" >&2; exit 2; }
mkdir -p "$out"
out="$(cd "$out" && pwd)"
threads="${RERUN_THREADS:-15}"
budget="${RERUN_BUDGET_SECONDS:-0}"   # 墙钟预算（秒），0 = 不限；用完在单元 / 变异行边界退 75，再跑同一条命令续
started="$(date +%s)"; deadline=0; (( budget > 0 )) && deadline=$(( started + budget ))
out_of_time() { (( deadline > 0 )) && (( $(date +%s) > deadline )); }
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-1500}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='上限按派发提示给的编译 6G、跑 4G，高于峰值表里同键实测的 2 倍也照给'
tree="$out/tree"; target="$out/target"; walk=crates/singlefs-checker/src/walk.rs
mkdir -p "$out/done" "$out/mut"

if [[ ! -d "$tree" ]]; then
    [[ -z "$(ls -A "$out" | grep -v -E '^(done|mut)$' || true)" ]] || { echo "$out 不是空目录" >&2; exit 2; }
    cp -a "$SINGLEFS_REPO" "$tree"; mkdir -p "$out/pristine"; cp -a "$SINGLEFS_REPO/$walk" "$out/pristine/walk.rs"
fi
cp "$out/pristine/walk.rs" "$tree/$walk"; touch "$tree/$walk"
cmp -s "$tree/$walk" "$SINGLEFS_REPO/$walk" || { echo "树副本里的 walk.rs 与冻结副本不同" >&2; exit 3; }
restore_walk() { cp "$out/pristine/walk.rs" "$tree/$walk"; touch "$tree/$walk"; }

cargo_step() {   # cargo_step <上限> <键> <cargo 参数…>
    local cap="$1" key="$2"; shift 2
    CARGO_TARGET_DIR="$target" RUN_WITH_MEMORY_CAP_KEY="m3b29-sonnet-$key" \
        nice -n 19 bash "$tools/scripts/capped.sh" "$threads" bash "$tools/scripts/run-with-memory-cap.sh" "$cap" \
        cargo test --offline --manifest-path "$tree/Cargo.toml" "$@"
}

unit_static() {
    set +e
    python3 "$tools/scripts/crates-mutation-rows.py" static-check "$tree" > "$out/static.raw" 2>&1
    set -e
    local row
    for row in $(seq 2977 2989); do
        printf 'STATIC\trow %s\t%s\n' "$row" "$(grep -c -P "^(ANCHOR|NAME|MALFORMED|STRAY|ROW)\t$row\t" "$out/static.raw" || true) 个问题行"
    done
    printf 'STATIC\trow 2935\t%s\n' "$(grep -P '^ANCHOR\t2935\t' "$out/static.raw" | sed -E 's/^ANCHOR\t2935\t[^\t]*\t//' || echo 无)"
    grep -P '^CHECKED\t' "$out/static.raw" | sed 's/^/STATIC\t/'
}
unit_sites() { python3 "$here/sites.py" "$tree"; }
unit_rowcheck() { python3 "$here/rowcheck.py" "$tree"; }
unit_consts() {
    local k
    for k in crash_judge_checker.wgsl crash_judge_checker_scan.wgsl; do
        printf 'CONSTS\t%s\tconst INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u; 命中 %s 处\n' "$k" \
            "$(grep -c -F 'const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;' "$tree/crates/singlefs-checker-tier/src/$k")"
    done
}
unit_new() {
    cargo_step 6G compile -p singlefs-harness --test checker_known_bad_images --no-run > "$out/new.build.log" 2>&1
    set +e
    cargo_step 4G run -p singlefs-harness --test checker_known_bad_images > "$out/new.run.log" 2>&1
    rc=$?; echo "$rc" > "$out/new.rc"
    set -e
    if (( rc >= 250 && rc <= 254 )); then echo "包装退 $rc（那一次的输出不算结果）" >&2; exit 4; fi
    grep -E '^test .* \.\.\. (ok|FAILED)$' "$out/new.run.log" | grep -E 'location_checksum|unregistered_type_over_a_child' | sed -E "s/^test (\S+) \.\.\. (ok|FAILED)$/NEW\t\1\t\2/"
    grep -E '^test result:' "$out/new.run.log" | sed -E 's/^/NEW\t整份二进制\t/; s/; finished in .*//'
}
mutrun() { python3 "$here/mutrun.py" --tree "$tree" --target "$target" --tools "$tools" --outdir "$out/mut" --deadline "$deadline" --threads "$threads" "$@"; }
unit_rows() { local rc=0; mutrun --table "$tree/crates/mutations.tsv" --rows "$(seq -s, 2977 2989)" > "$out/rows.log" || rc=$?; (( rc == 0 )) || return "$rc"; grep '^ROW' "$out"/mut/mutations.tsv-*.txt | sed 's/^[^:]*://' ; }
unit_extra() { local rc=0; mutrun --table "$here/extra-mutations.tsv" --rows "$(seq -s, 1 "$(grep -c . "$here/extra-mutations.tsv")")" > "$out/extra.log" || rc=$?; (( rc == 0 )) || return "$rc"; grep -h '^ROW' "$out"/mut/extra-mutations.tsv-*.txt; }

legit_tests=(new_pool_file_creation_publish unlink_single_file_publishes pool_checker_judges_the_deletion_state_of_every_walked_version rollback_lineage_record_and_its_checkpoint_cost_term container_trees_grow_a_level_when_the_root_is_full deletion_scenario_writes unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches unlink_mount_continuation_and_the_emptied_version test_only_switches_force_every_branch)
unit_legit() {
    local flags=() t
    for t in "${legit_tests[@]}"; do flags+=(--test "$t"); done
    cargo_step 6G compile -p singlefs-harness "${flags[@]}" --no-run > "$out/legit.build.log" 2>&1
    set +e
    cargo_step 4G run -p singlefs-harness "${flags[@]}" > "$out/legit.run.log" 2>&1
    rc=$?; echo "$rc" > "$out/legit.rc"
    set -e
    if (( rc >= 250 && rc <= 254 )); then echo "包装退 $rc（那一次的输出不算结果）" >&2; exit 4; fi
    awk '/Running tests\// {name=$2} /^test result:/ {printf "LEGIT\t%s\t%s %s %s %s %s\n", name, $3, $4, $5, $6, $7}' "$out/legit.run.log"
}

for unit in static sites rowcheck consts new rows extra legit; do
    [[ -f "$out/done/$unit" ]] && continue
    if out_of_time; then echo "预算 ${budget}s 用完，在单元边界停；再跑同一条命令续" >&2; exit 75; fi
    echo "== 单元 $unit"
    set +e; "unit_$unit" > "$out/$unit.txt"; rc=$?; set -e
    restore_walk
    (( rc == 75 )) && { echo "预算用完（单元 $unit 在变异行边界停）；再跑同一条命令续" >&2; exit 75; }
    (( rc == 0 )) || { echo "单元 $unit 退 $rc" >&2; exit "$rc"; }
    touch "$out/done/$unit"
done

{ for unit in static sites rowcheck consts new rows extra legit; do cat "$out/$unit.txt"; done; grep -h '^BASE' "$out"/mut/BASE-*.txt | cut -f1-4; } | LC_ALL=C sort > "$out/summary.txt"
cat "$out/summary.txt"
echo "summary: $(wc -l < "$out/summary.txt") 行 -> $out/summary.txt"
