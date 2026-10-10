#!/usr/bin/env bash
# b29 第一轮正推腿的复跑脚本：只跑 harness 档与池级 checker 的测试二进制，不跑 checker 档包、GPU、变异整表。
# 用法（主 agent 在后台起；整条脚本经内存包装跑，线程上限 10）：
#   REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b29-r1/tree WORK=<会话草稿>/b29-sonnet-rerun \
#     bash run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 \
#     bash research/prompts/m3-formal-code-b29-r1-sonnet-model/tools/rerun.sh <段名…>
# 段名：setup anchors baseline l3 l2 l1-mutants l1-grid l1-extra l2-bigtree l4 l3-positive all
# 每一段把原样输出落 $WORK/out/<段名>.log，末尾打 RESULT 行；各段的预期读数写在报告 research/prompts/m3-formal-code-b29-r1-sonnet-output.md。
set -uo pipefail
REPO=${REPO:?REPO 要设成主仓根}; FROZEN=${FROZEN:?FROZEN 要设成冻结副本根}; WORK=${WORK:?WORK 要设成草稿目录}
MODEL=$REPO/research/prompts/m3-formal-code-b29-r1-sonnet-model
TREE=$WORK/tree; OUT=$WORK/out; mkdir -p "$OUT"
export CARGO_TARGET_DIR=$WORK/target
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-600}
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="b29 正推腿复跑，上限取派发给的 8G"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="派出去的几个 agent 各跑各的，这条是 b29 正推腿的复跑"
CAPPED="nice -n 19 bash $REPO/research/scripts/run-with-memory-cap.sh 8G bash $REPO/research/scripts/capped.sh 10"
# 脚本整条已经在包装里时，这里的命令直接 nice + cargo（外面那层包装已给内存上限；脚本里不再嵌套起第二层）
if [[ -n "${RERUN_ALREADY_WRAPPED:-}" ]]; then CAPPED="nice -n 19"; fi
cargo_test() { (cd "$TREE" && $CAPPED cargo test --offline "$@"); }

seg_setup() {
  rm -rf "$TREE"; mkdir -p "$TREE"
  rsync -a --exclude target "$FROZEN"/ "$TREE"/
  (cd "$TREE" && sha256sum -c "$(dirname "$FROZEN")/crates-sha256.txt" | grep -vc ': OK' || true) > "$OUT/setup-sha-notok.count"
  for f in b29_l1_rollback_grid b29_l2_big_tree b29_l2_checker_sweep b29_l3_zero_slot_probe; do cp "$MODEL/probes/$f.rs" "$TREE/crates/singlefs-harness/tests/"; done
  cp "$TREE/crates/singlefs-harness/tests/checker_known_bad_images.rs" "$TREE/crates/singlefs-harness/tests/b29_l4_probe.rs"
  cat "$MODEL/probes/b29_l4_append.rs" >> "$TREE/crates/singlefs-harness/tests/b29_l4_probe.rs"
  echo "RESULT setup tree=$TREE sha_not_ok=$(cat "$OUT/setup-sha-notok.count")"
}
seg_anchors() {
  python3 "$MODEL/tools/mutants.py" anchors "$MODEL/data/mutants.tsv" "$FROZEN" | tail -1
  python3 "$MODEL/tools/mutants.py" anchors "$MODEL/data/extra.tsv" "$FROZEN" | tail -1
  (cd "$REPO" && python3 research/scripts/crates-mutation-rows.py static-check "$FROZEN" | tail -3)
}
seg_baseline() {
  cargo_test -p singlefs-harness --lib --test checker_known_bad_images --test mount_transparency_judged_field_by_field_after_a_writable_mount \
    --test writable_mount_refusal_on_a_crash_state_without_read_faults \
    --test writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion \
    --test model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers \
    --test model_compares_inode_creations_into_the_third_inode_leaf_container \
    --test inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds 2>&1 | grep -E 'Running|test result'
}
seg_l3() {  # 8 个进程各 3 个种子 × 8 组比重，6 步；再加「读回全 0」那一组 8 步（读故障那一类）
  cargo_test -p singlefs-harness --test b29_l3_zero_slot_probe --no-run >/dev/null 2>&1
  TREE=$TREE OUT=$OUT NAME=b29_l3_zero_slot_probe OPS=6 SEEDS=3 CAP=800 SL=3 STARTS="0 3 6 9 12 15 18 21" bash "$MODEL/tools/probe_batch.sh"
  PROBE_WEIGHTS="读回全 0" TREE=$TREE OUT=$OUT NAME=b29_l3_zero_slot_probe OPS=8 SEEDS=4 CAP=600 SL=25 STARTS="100 104 108 112" bash "$MODEL/tools/probe_batch.sh"
}
seg_l2() {
  cargo_test -p singlefs-harness --test b29_l2_checker_sweep --no-run >/dev/null 2>&1
  TREE=$TREE OUT=$OUT NAME=b29_l2_checker_sweep OPS=6 SEEDS=3 CAP=400 EVERY=150 STARTS="30 33 36 39 42 45 48 51" bash "$MODEL/tools/probe_batch.sh"
}
seg_l2_bigtree() { cargo_test -p singlefs-harness --test b29_l2_big_tree -- --nocapture 2>&1 | grep -E 'BIGTREE|test result'; }
mutate() {  # 变异行表 行序号…
  local table=$1; shift
  REPO=$REPO MUTLOG=$OUT MUT_TIMEOUT=${MUT_TIMEOUT:-900} python3 "$MODEL/tools/mutants.py" run "$table" "$TREE" "$@"
}
seg_l1_mutants() {  # 抽出的 61 行里 harness 档的 24 行（2、3 号是改名之前的旧名，换成 23、24）
  mutate "$MODEL/data/mutants.tsv" 0 1 4 5 18 20 21 22 23 24 25 26 27 28 29 30 31 32 36 44 45 46 48
}
seg_l1_extra() {  # 取样点变体：min(2) 两条与 retain / extend 只处理前两片（入库用例上预期都活下来），G1 两条
  mutate "$MODEL/data/extra.tsv" 102 103 104 105 110 111
}
seg_l1_grid() {  # T13 换取样点：基线与两个变异，格子里 NewFinding 的是抓到
  cargo_test -p singlefs-harness --test b29_l1_rollback_grid --no-run >/dev/null 2>&1
  export GRID="468:1:3,468:2:3,468:3:3,468:6:3,468:2:0,468:2:1,236:3:3,236:2:3,470:2:3,703:4:3,703:6:3"
  echo "== baseline"; cargo_test -p singlefs-harness --test b29_l1_rollback_grid -- --nocapture 2>&1 | grep CELL
  for row in 100 101 106 107 108 109; do mutate "$MODEL/data/extra.tsv" $row; grep CELL "$OUT/mut-$row.log" | sed -E 's/NewFinding \{ signature: ModelDisagreement \{ aspect: "([^"]*)".*position: (Operation\([0-9]+\)).*/NewFinding(\1 @\2)/' | cut -c1-160; done
}
seg_l4() {
  cargo_test -p singlefs-harness --test b29_l4_probe -- --nocapture b29_l4_duplicate_kind_probe 2>&1 | grep -E 'L4PROBE|test result'
  cargo_test -p singlefs-harness --test b29_l4_probe -- --nocapture b29_l2_i54 2>&1 | grep -E 'L2PROBE|test result'
}
seg_l3_positive() {  # 阳性对照：改过的核心（写完新槽紧跟着把同盘另一槽清 0）上，同一个探针必须数到零槽的态
  rm -rf "$WORK/tree-mut"; mkdir -p "$WORK/tree-mut"; rsync -a --exclude target --exclude 'crates/singlefs-harness/tests/b29_l4_probe.rs' "$TREE"/ "$WORK/tree-mut"/
  python3 - "$WORK/tree-mut/crates/singlefs-core/src/transaction.rs" <<'PY'
import sys
p=sys.argv[1]; s=open(p,encoding='utf-8').read()
old="""            &slot_bytes,
            WriteDurability::Plain,
        )?;
        self.writes_by_structure_kind
            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);
        Ok(())"""
new="""            &slot_bytes,
            WriteDurability::Plain,
        )?;
        let other_slot = ((slot_generation + 1) % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing;
        self.devices[index].1.write_at(
            DeviceOffsetInBytes(other_slot),
            &vec![0u8; slot_bytes.len()],
            WriteDurability::Plain,
        )?;
        self.writes_by_structure_kind
            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);
        Ok(())"""
assert s.count(old)==1
open(p+'.new','w',encoding='utf-8').write(s.replace(old,new))
PY
  mv "$WORK/tree-mut/crates/singlefs-core/src/transaction.rs.new" "$WORK/tree-mut/crates/singlefs-core/src/transaction.rs"
  (cd "$WORK/tree-mut" && CARGO_TARGET_DIR=$WORK/target-mut PROBE_SEEDS=1 PROBE_OPS=4 PROBE_STATE_CAP=800 PROBE_SECOND_LEVEL=2 $CAPPED cargo test --offline -p singlefs-harness --test b29_l3_zero_slot_probe -- --nocapture 2>&1 | grep -E 'PROBE (histories|first_cut|second_cut)|test result')
}
for seg in "$@"; do
  [[ $seg == all ]] && set -- setup anchors baseline l3 l2 l2-bigtree l1-mutants l1-extra l1-grid l4 l3-positive && break
done
for seg in "$@"; do
  fn="seg_${seg//-/_}"
  echo "===== $seg ====="; "$fn" 2>&1 | tee "$OUT/$seg.log"; echo "RESULT $seg rc=${PIPESTATUS[0]}"
done
