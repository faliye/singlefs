#!/usr/bin/env bash
# b30 第一轮云端正推腿的复跑脚本：只跑 harness 档测试二进制与池级 checker 的探针，不跑 checker 档包、GPU、变异整表。
# 用法（主 agent 在后台起；整条脚本经内存包装跑，线程上限 7）：
#   REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b30-r1/tree WORK=<会话草稿>/b30-sonnet-rerun RERUN_ALREADY_WRAPPED=1 \
#     bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 7 \
#     bash research/prompts/m3-formal-code-b30-r1-sonnet-model/tools/rerun.sh <段名…>
# 段名：setup anchors m1-tests m1-rows m1-grid m1-extra m1-extra-wide m2-rows m2-sweep m2-scan m2-heads m2-kind m2-i54 m3-pins m3-433 m3-rows m3-greps m4-keys all
# 每一段原样输出落 $WORK/out/<段名>.log，末尾打 RESULT 行；各段的预期读数写在报告 research/prompts/m3-formal-code-b30-r1-sonnet-output.md。
# 三棵树：tree（冻结副本原样 + 探针）、tree-old（只把 30-2 对 walk.rs 的改动倒回去）、tree-secs（只把 core 的 T_time 倒回写 5）。
set -uo pipefail
REPO=${REPO:?REPO 要设成主仓根}; FROZEN=${FROZEN:?FROZEN 要设成冻结副本根}; WORK=${WORK:?WORK 要设成草稿目录}
MODEL=$REPO/research/prompts/m3-formal-code-b30-r1-sonnet-model
OUT=$WORK/out; mkdir -p "$OUT"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-600}
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="b30 正推腿复跑，上限取派发给的 8G"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的几个 agent 各跑各的，这条是 b30 正推腿的复跑"
CAPPED="nice -n 19 bash $REPO/scripts/hosts/run-with-memory-cap.sh 8G bash $REPO/scripts/hosts/capped.sh 7"
if [[ -n "${RERUN_ALREADY_WRAPPED:-}" ]]; then CAPPED="nice -n 19"; fi
PROBES=$MODEL/probes
# 用哪棵树跑：cargo_in <树名> <目标目录名> cargo 参数…
cargo_in() { local tree=$1 target=$2; shift 2; (cd "$WORK/$tree" && CARGO_TARGET_DIR=$WORK/$target $CAPPED cargo test --offline "$@"); }
mut() { REPO=$REPO CARGO_TARGET_DIR=$WORK/target MUTLOG=$OUT MUT_TIMEOUT=${MUT_TIMEOUT:-900} python3 "$MODEL/tools/mutants.py" run "$1" "$WORK/tree" "${@:2}"; }
grid() { REPO=$REPO CARGO_TARGET_DIR=$WORK/target GRID_TIMEOUT=${GRID_TIMEOUT:-900} python3 "$MODEL/tools/grid_variants.py" "$1" "$WORK/tree" "$2" "${@:3}"; }

place_probes() {   # 探针放进某棵树
  local t=$1
  cp "$PROBES/b30_m1_rollback_grid.rs" "$PROBES/b30_m2_checker_sweep.rs" "$PROBES/b30_m2_pointer_head_predicate.rs" "$t/crates/singlefs-harness/tests/"
  local tests=$t/crates/singlefs-harness/tests
  cat "$PROBES/b30_m2_pointer_scan_wrapper.rs" >> "$tests/common_pointer_head_corruption/mod.rs"
  cp "$tests/checker_known_bad_images.rs" "$tests/b30_m2_pointer_scan.rs"
  cat "$PROBES/b30_m2_pointer_scan_append.rs" "$PROBES/b30_m3_checker_433_append.rs" >> "$tests/b30_m2_pointer_scan.rs"
  cp "$tests/checker_known_bad_images.rs" "$tests/b30_m2_i54_units.rs"; cat "$PROBES/b30_m2_i54_units_append.rs" >> "$tests/b30_m2_i54_units.rs"
  cp "$tests/checker_known_bad_images.rs" "$tests/b30_m2_kind.rs"; cat "$PROBES/b30_m2_kind_append.rs" >> "$tests/b30_m2_kind.rs"
  cp "$tests/system_configuration_mutability_classes.rs" "$tests/b30_m3_slot_dump.rs"; cat "$PROBES/b30_m3_slot_dump_append.rs" >> "$tests/b30_m3_slot_dump.rs"
  cp "$tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs" "$tests/b30_m3_images.rs"; cat "$PROBES/b30_m3_images_append.rs" >> "$tests/b30_m3_images.rs"
}
seg_setup() {
  rm -rf "$WORK/tree" "$WORK/tree-old" "$WORK/tree-secs"; mkdir -p "$WORK/tree"
  rsync -a --exclude target "$FROZEN"/ "$WORK/tree"/
  (cd "$WORK/tree" && sha256sum -c "$(dirname "$FROZEN")/crates-sha256.txt" | grep -vc ': OK' || true) > "$OUT/setup-sha-notok.count"
  rsync -a --exclude target "$WORK/tree"/ "$WORK/tree-old"/; rsync -a --exclude target "$WORK/tree"/ "$WORK/tree-secs"/
  (cd "$WORK/tree-old" && patch -R -p1 < "$MODEL/data/walk-30-2.patch")
  python3 - "$WORK/tree-secs/crates/singlefs-core/src/system_configuration.rs" <<'PY'
import sys
p = sys.argv[1]; s = open(p, encoding='utf-8').read()
old = 'const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;'
assert s.count(old) == 1
open(p + '.new', 'w', encoding='utf-8').write(s.replace(old, 'const TIME_THRESHOLD_MILLISECONDS: u32 = 5;'))
PY
  mv "$WORK/tree-secs/crates/singlefs-core/src/system_configuration.rs.new" "$WORK/tree-secs/crates/singlefs-core/src/system_configuration.rs"
  for t in tree tree-old tree-secs; do place_probes "$WORK/$t"; done
  echo "RESULT setup sha_not_ok=$(cat "$OUT/setup-sha-notok.count")"
}
seg_anchors() {
  python3 "$MODEL/tools/mutants.py" anchors "$MODEL/data/mutants.tsv" "$FROZEN" | tail -1
  (cd "$REPO" && python3 scripts/mutation/crates-mutation-rows.py static-check "$FROZEN" | tail -3)
}
seg_m1_tests() { cargo_in tree target -p singlefs-harness --test model_compares_the_third_and_fourth_inode_leaf_containers 2>&1 | grep -E 'test |result'; }
seg_m1_rows() { mut "$MODEL/data/mutants.tsv" 0 1 2 3; }
seg_m1_grid() {
  local GA="700:2:3,702:2:3,703:2:3,703:1:3,703:3:3,703:4:3,703:6:3" GB="933:2:3,935:2:3,936:3:3,936:5:3,936:6:3,940:2:3,936:2:0"
  local GC="936:8:3,936:9:3,936:12:3,936:16:3,936:40:3,936:300:3,703:8:3" GX="703:1:3,703:2:3,934:2:3,935:3:3,936:2:3,936:5:3,933:2:3" GY="701:2:0,701:2:1,701:2:10,471:2:0,471:2:3,471:1:3,471:3:3"
  grid "$MODEL/data/mutants.tsv" "$GA" base 0 1 2 3
  grid "$MODEL/data/mutants.tsv" "$GB" base
  grid "$MODEL/data/mutants.tsv" "$GC" base
  grid "$MODEL/data/extra.tsv" "$GX" base 100 102 103 104
  grid "$MODEL/data/mutants.tsv" "$GY" base 2 3
}
seg_m1_extra() { mut "$MODEL/data/extra.tsv" 100 101 102 103 104; }
seg_m1_extra_wide() { mut "$MODEL/data/extra-wide.tsv" 100 101 102 103; }
seg_m2_rows() {
  mut "$MODEL/data/mutants.tsv" 4 5 6 16
  cargo_in tree target -p singlefs-harness --test checker_known_bad_images 2>&1 | grep -E 'test result|FAILED|container_pointer|newest_version_that'
}
sweep_one() {  # sweep_one <树名> <目标目录名> <输出目录名>：两轮（种子 60–66 各 1 个、6 步；种子 70–82 各 2 个、8 步）
  local tree=$1 target=$2 out=$3; mkdir -p "$OUT/$out"
  cargo_in "$tree" "$target" -p singlefs-harness --test b30_m2_checker_sweep --no-run >/dev/null 2>&1
  TREE=$WORK/$tree OUT=$OUT/$out CARGO_TARGET_DIR=$WORK/$target NAME=b30_m2_checker_sweep OPS=6 SEEDS=1 CAP=400 EVERY=100 SL=3 STARTS="60 61 62 63 64 65 66" bash "$MODEL/tools/probe_batch.sh" | grep -E '^start|histories'
  TREE=$WORK/$tree OUT=$OUT/$out CARGO_TARGET_DIR=$WORK/$target NAME=b30_m2_checker_sweep OPS=8 SEEDS=2 CAP=400 EVERY=100 SL=3 STARTS="70 72 74 76 78 80 82" bash "$MODEL/tools/probe_batch.sh" | grep -E '^start|histories'
  cat "$OUT/$out"/*.log | grep '^PROBE_CHECK' | sort > "$OUT/$out-checks.txt"
  echo "$out checks=$(wc -l < "$OUT/$out-checks.txt") sha256=$(sha256sum "$OUT/$out-checks.txt" | cut -d' ' -f1)"
  grep -oE 'I-2.4=[A-Za-z]+ I-5.4=[A-Za-z]+ violated=\[[^]]*\]' "$OUT/$out-checks.txt" | sort | uniq -c
}
seg_m2_sweep() { sweep_one tree target sw-new; sweep_one tree-old target-old sw-old; cmp "$OUT/sw-new-checks.txt" "$OUT/sw-old-checks.txt" && echo "SWEEP_OLD_NEW_IDENTICAL"; }
scan() { B30_ONLY=$3 cargo_in "$1" "$2" -p singlefs-harness --test b30_m2_pointer_scan -- --nocapture b30_m2_pointer_scan 2>&1 | grep -E '^B30SCAN|^PTR|^ZEROING|test result|^error'; }
seg_m2_scan() {
  for img in overwrite rollback deletion-intent deletion-first-batch three-unit-file two-page-instance-table normal-unmount new-inodes-300; do scan tree target "$img"; done
  for img in deletion-intent rollback; do scan tree-old target-old "$img"; done
}
seg_m2_heads() { cargo_in tree target -p singlefs-harness --test b30_m2_pointer_head_predicate -- --nocapture 2>&1 | grep -E 'B30HEADS|test result'; }
seg_m2_kind() { cargo_in tree target -p singlefs-harness --test b30_m2_kind -- --nocapture b30_m2_duplicate_kind 2>&1 | grep -E 'B30KIND|test result' | cut -c1-330; }
seg_m2_i54() {
  for pair in "tree target new" "tree-old target-old old"; do set -- $pair
    cargo_in "$1" "$2" -p singlefs-harness --test b30_m2_i54_units -- --nocapture b30_m2_i54 2>&1 | grep -E 'B30I54|test result' > "$OUT/i54-$3.txt"; done
  diff <(grep B30I54 "$OUT/i54-old.txt") <(grep B30I54 "$OUT/i54-new.txt") | cut -c1-200; echo "diff 行数 $(diff <(grep B30I54 "$OUT/i54-old.txt") <(grep B30I54 "$OUT/i54-new.txt") | grep -c '^>')"
}
seg_m3_pins() {
  cargo_in tree target -p singlefs-harness --test system_configuration_mutability_classes --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over --test mount_transparency_judged_field_by_field_after_a_writable_mount 2>&1 | grep -E 'Running|test result|FAILED' | cut -c1-120
  for pair in "tree target new" "tree-secs target-secs old"; do set -- $pair
    cargo_in "$1" "$2" -p singlefs-harness --test b30_m3_slot_dump --test b30_m3_images -- --nocapture b30_m3 > "$OUT/m3-pins-$3.log" 2>&1
    grep -E '^B30IMAGES|^B30SECTORS|test result' "$OUT/m3-pins-$3.log"; done
  python3 "$MODEL/tools/slot_check.py" "$OUT/m3-pins-old.log" "$OUT/m3-pins-new.log" \
    e5cc28abb94e3e1739b85dc7abc2191a1b120908b72a5ab3cb03588feb01c28f 144c1785dd57555c8539963909817a597c5a32d2d4b8fc3e61f7a8f6233f299e
}
seg_m3_433() { cargo_in tree target -p singlefs-harness --test b30_m2_pointer_scan -- --nocapture b30_m3_the_pool_checker 2>&1 | grep -E '^B30433|test result'; }
seg_m3_rows() { mut "$MODEL/data/mutants.tsv" 17 18; }
seg_m3_greps() {
  cd "$REPO"
  echo "== crates 里 T_time 的全部引用（不含变异表）"; grep -rniE 'time_threshold|TIME_THRESHOLD' crates --include='*.rs' --include='*.wgsl' | grep -v 'crates/mutations.tsv' | cut -c1-170
  echo "== crates 里偏移 433 字面"; grep -rnE '\b433\b' crates --include='*.rs' --include='*.wgsl' | grep -v 'b30_m' | cut -c1-170
  echo "== research 装置里写 / 解 T_time 盘上字段的地方"; grep -rnIE 'put_u32\(5\)|put_u32\(5000\)|T_time (秒|毫秒)|"checkpoint_time_threshold"' scripts/experiment-apparatus --include='*.rs' --include='*.py' | cut -c1-200
  echo "== 字段表与 D16"; grep -n '| T_time |' .claude/kb/layout/01-first-txn.md; sed -n 140p .claude/kb/decisions/16-发布语义.md | cut -c1-120
}
seg_m4_keys() {
  cd "$REPO"; python3 - "$FROZEN" "$MODEL" "$OUT" <<'PY'
import hashlib, importlib.util, sys
frozen, model, out = sys.argv[1:]
spec = importlib.util.spec_from_file_location('cmr', 'scripts/mutation/crates-mutation-rows.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
rows, bad, mem, to = m.read_table(f'{frozen}/crates/mutations.tsv')
byname = {}
for r in rows: byname.setdefault(r.name, []).append(r)
mine = [l.split('\t') for l in open(f'{model}/data/mutants.tsv', encoding='utf-8').read().split('\n') if l]
import os
for f in mine:
    i, name = int(f[0]), f[3]; log = f'{out}/mut-{i}.log'
    if not os.path.exists(log): continue
    cands = byname.get(name, [])
    if len(cands) != 1: print('NAME_NOT_IN_TABLE', i, name[:40], len(cands)); continue
    print('ROW', i, 'table_line', cands[0].line_number, 'verdict', m.verdict_of_run(cands[0], 101, open(log, encoding='utf-8').read(), '8G', 900)[0])
r = [x for x in rows if x.line_number == 2636][0]
def key(fields): return hashlib.sha256(("\t".join(fields) + "\nmemory_max=8G\ntimeout_seconds=900\n").encode()).hexdigest()[:12]
b29 = ['第28批 G1 攻方 MUT-A：core 的 T_time 内置默认 5 改 4（挂载透明「R2 = 内置默认」的期望照条款写死才抓得到）', r.fields[1], 'const TIME_THRESHOLD_SECONDS: u32 = 5;', 'const TIME_THRESHOLD_SECONDS: u32 = 4;', r.fields[4], r.fields[5]]
patch = list(r.fields); patch[0] = b29[0]
print('MUT-A 行键 b29 行', key(b29), '| 补丁原样（新锚点旧名）', key(patch), '| 入树现行', key(list(r.fields)))
PY
}
for seg in "$@"; do
  [[ $seg == all ]] && set -- setup anchors m1-tests m1-rows m1-grid m1-extra m1-extra-wide m2-rows m2-sweep m2-scan m2-heads m2-kind m2-i54 m3-pins m3-433 m3-rows m3-greps m4-keys && break
done
for seg in "$@"; do
  fn="seg_${seg//-/_}"
  echo "===== $seg ====="; "$fn" 2>&1 | tee "$OUT/$seg.log"; echo "RESULT $seg rc=${PIPESTATUS[0]}"
done
