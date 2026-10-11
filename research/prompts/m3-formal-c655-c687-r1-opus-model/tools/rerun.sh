#!/usr/bin/env bash
# m3-formal-c655-c687-r1 攻方腿原型的复跑。在冻结副本的一份拷贝上：打探针补丁（probe-core.patch：recovery.rs 加一个只读函数、allocator.rs 回落那一支加计数、
# 工作区加 c655probe）、把 harness-probe-tests.rs 接到 checker_known_bad_images.rs 末尾、放进 c655probe，编、逐件跑，日志写进 $WORK/logs，
# 再与模型目录里同名日志比（去掉耗时、排队、编译那几行）。
# 环境变量（都可覆盖）：REPO 仓根（默认按本脚本位置推）、FROZEN 冻结副本 tree 目录、WORK 草稿目录、THREADS 线程数（默认 10）。
# 用法：bash tools/rerun.sh [件名…]；件名：harness y1-4g y1-384-check y1-384 y1s2 fallback（不给就全跑，合计十几分钟，视负载）。
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
REPO="${REPO:-$(cd "$here/../../.." && pwd)}"
FROZEN="${FROZEN:-/tmp/claude-1000/m3-formal-c655-c687-r1/tree}"
WORK="${WORK:-/tmp/claude-1000/m3-formal-c655-c687-r1-opus-rerun}"
THREADS="${THREADS:-10}"
items=("$@"); [ ${#items[@]} -eq 0 ] && items=(harness y1-4g y1-384-check y1-384 y1s2 fallback)
W="$REPO/scripts/hosts/run-with-memory-cap.sh"
C="$REPO/scripts/hosts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-复跑 m3-formal-c655-c687-r1 攻方腿原型 c655probe}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-复跑攻方腿原型，盘宽与历史随参数变，峰值表那一次不代表这一次}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-1800}"
mkdir -p "$WORK/logs"
rsync -a --delete --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
cd "$WORK/tree"
patch -p1 < "$here/probe-core.patch"
mkdir -p crates/c655probe/src
cp "$here/Cargo.toml" crates/c655probe/Cargo.toml
cp "$here/main.rs" crates/c655probe/src/main.rs
cat "$here/harness-probe-tests.rs" >> crates/singlefs-harness/tests/checker_known_bad_images.rs
export CARGO_TARGET_DIR="$WORK/target"
nice -n 19 bash "$W" 4G bash "$C" "$THREADS" cargo build --offline --release -p c655probe
P="$WORK/target/release/c655probe"
L="$WORK/logs"
run() { local log="$1"; shift; local rc=0; nice -n 19 "$@" > "$L/$log" 2>&1 || rc=$?; echo "$log exit=$rc"; }
for item in "${items[@]}"; do
  case "$item" in
    harness)
      for pair in y3-cells.log:c687_probe_cells_on_bad_images y3-cells-f.log:c687_probe_duplicate_kinds_on_a_version_not_built_from_the_mkfs_watermark \
                  y3-cells-e45.log:c687_probe_tree_table_without_the_allocation_entry y1-root-fields.log:c655_probe_root_fields_outside_the_table_written_wrong_on_a_file_version \
                  y1-root-fields-not-newest.log:c655_probe_root_fields_written_wrong_on_a_version_that_is_not_the_newest y1-sc-fields.log:c655_probe_system_configuration_runtime_quantities_written_wrong; do
        run "${pair%%:*}" bash "$W" 6G bash "$C" "$THREADS" cargo test --offline -p singlefs-harness --test checker_known_bad_images "${pair#*:}" -- --exact --nocapture
      done ;;
    y1-4g) for h in s1 ul@1-4 ul@5-8; do run "y1-$h.log" env C655_CHECK_AFTER_MOUNT=1 bash "$W" 6G bash "$C" "$THREADS" "$P" y1 "$h" "$THREADS"; done ;;
    y1-384-check) for h in ul@1-4 ul@5-8 ow@1-4 rb@1-4 rbow@1-4 ci@1-4; do run "y1w384-$h.log" env C655_WIDTH=384 C655_CHECK_AFTER_MOUNT=1 bash "$W" 3G bash "$C" "$THREADS" "$P" y1 "$h" "$THREADS"; done ;;
    y1-384) for h in s1 s2@0-1 s2@2-5 s2@6-9 fr@0-1 fr@2-5 fr@6-9 fr s2; do run "y1w384-$h.log" env C655_WIDTH=384 bash "$W" 3G bash "$C" "$THREADS" "$P" y1 "$h" "$THREADS"; done ;;
    y1s2) for spec in 'ul@1-4 last' 's1 0,last' 'rb@1-4 last' 'fr last' 's2 0'; do set -- $spec
            run "y1s2w384-$1-$2.log" env C655_WIDTH=384 bash "$W" 3G bash "$C" "$THREADS" "$P" y1s2 "$1" "$2" "$THREADS"; done ;;
    fallback)
      run fallback-240-a.log bash "$W" 3G "$P" fallback 240 m1z0,m2z0,m4z0,m8z0,m16z0,m1z4,m1z16,m4z4,m8z8
      shapes=$(python3 -c "print(','.join(f'm{n}z{z}' for n in [3,5,6,7,9,10,11,12,13,14,15,18,20,24,28,32,40,48,64] for z in [0,1,2,3,8]))")
      for w in 240 256 384; do run "fallback-$w-b.log" bash "$W" 3G "$P" fallback "$w" "$shapes"; done
      run fallback-control.log bash "$W" 3G "$P" fallback-control ;;
    *) echo "未知件 $item"; exit 2 ;;
  esac
done
# 与模型目录里的日志逐行比：去掉耗时、排队、编译、测试框架那几行与取样的例子（例子按线程交回的先后取，随调度变）。
strip() { grep -v -E 'elapsed_secs|per_state_ms|prepare_secs|^run-with-memory-cap|Compiling|Finished|Running|^running|^test |^$|finished in|^    e\.g\.|^      R[12]:' "$1" || true; }
for log in "$L"/*.log; do
  name="$(basename "$log")"
  [ -f "$here/$name" ] || { echo "$name 模型目录里没有"; continue; }
  if diff <(strip "$here/$name") <(strip "$log") > /dev/null; then echo "$name 逐行相同"; else echo "$name 不同"; fi
done
cd "$L"
# 三份汇总要 y1-4g、y1-384-check、y1-384、y1s2 四件的日志都在；少一件就不出汇总。
if ! { [ -f y1-s1.log ] && [ -f 'y1w384-ci@1-4.log' ] && [ -f y1w384-s2.log ] && [ -f y1s2w384-s2-0.log ]; }; then echo "汇总没出：y1-4g、y1-384-check、y1-384、y1s2 四件没全跑"; exit 0; fi
F='mapping|allocation_record|alg_nonce|deleted|tombstone|lineage|sc\.'
python3 "$here/tools/summarize.py" y1-s1.log 'y1-ul@1-4.log' 'y1-ul@5-8.log' $(ls y1w384-*.log) --fields "$F" > summary-y1-first-crash.txt
python3 "$here/tools/summarize.py" y1s2w384-*.log --fields "$F" > summary-y1-second-crash.txt
python3 "$here/tools/summarize.py" y1-s1.log 'y1-ul@1-4.log' 'y1-ul@5-8.log' $(ls y1w384-*.log) y1s2w384-*.log --fields 'tt\.(allocation|accounting|livelist|sparse|deadlist|logged_ops)\.root' > summary-y2-other-tree-roots.txt
for s in summary-y1-first-crash.txt summary-y1-second-crash.txt summary-y2-other-tree-roots.txt; do
  if diff <(grep -v -E 'elapsed_secs' "$here/$s") <(grep -v -E 'elapsed_secs' "$s") > /dev/null; then echo "$s 逐行相同（去掉耗时）"; else echo "$s 不同"; fi
done
