#!/usr/bin/env bash
# 攻方腿 m3-batch10z-jk-code-r1-opus 的复跑：K5 判别力的独立变异-复跑核对。
# 没有新写的模型源码：这条腿在冻结副本的拷贝上，对入库的 mount.rs 逐条应用 mutations-append.tsv 里第十块那 10 行的改法，
# 跑点名的测试，看每一条都红（判别力）。库级 4 条跑 singlefs-core --lib，harness 级 6 条跑那份 harness 测试。
# 用法：bash rerun.sh <冻结副本tree路径> <一个空的CARGO_TARGET_DIR>
set -u
TREE="${1:?冻结副本 tree 路径}"; TGT="${2:?空 target 目录}"
SFS=/home/fy5090/code/singlefs
cd "$TREE"; export CARGO_TARGET_DIR="$TGT"
M=crates/singlefs-core/src/mount.rs
PRISTINE="$TGT/mount.pristine"; cp "$M" "$PRISTINE"
runlib () { bash "$SFS/research/scripts/run-with-memory-cap.sh" 8G bash "$SFS/research/scripts/capped.sh" 15 \
  cargo test -p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests 2>&1 | grep 'test result:'; cp "$PRISTINE" "$M"; }
runh () { bash "$SFS/research/scripts/run-with-memory-cap.sh" 8G bash "$SFS/research/scripts/capped.sh" 15 \
  cargo test -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- "$1" 2>&1 | grep -E 'test result:|panicked at.*\.rs:[0-9]' | head -2; cp "$PRISTINE" "$M"; }
echo "== baseline lib =="; runlib
echo "== L4 take(1) =="; sed -i 's/\.take(ring_length_in_txgs)/.take(1)/' "$M"; runlib
echo "== L5 dedup-off =="; sed -i 's/if already_there.insert((placement.device, placement.slot, placement.span)) {/if already_there.insert((placement.device, placement.slot, placement.span)) || true {/' "$M"; runlib
echo "== L10 from-selected =="; sed -i 's/(selected_version_txg.0 + 1/(selected_version_txg.0/' "$M"; runlib
T=units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
echo "== L3 chain+2 =="; sed -i 's/(selected_version_txg.0 + 1/(selected_version_txg.0 + 2/' "$M"; runh units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written
echo "== L7 resurrection filter inverted =="; sed -i 's/\.filter(|(device, slot, _)| released_in_the_current_account.contains(&(\*device, \*slot)))/.filter(|(device, slot, _)| !released_in_the_current_account.contains(\&(*device, *slot)))/' "$M"; runh "$T"
echo "== L8 LINEAGE->!LINEAGE (== 认不出回退：谱系片读不出的等价) =="; sed -i 's/== crate::unit::PACKED_TYPE_LINEAGE)/!= crate::unit::PACKED_TYPE_LINEAGE)/' "$M"; runh "$T"
echo "== L9 !is_abandoned->is_abandoned =="; sed -i 's/\.filter(|root| !is_abandoned(root) \&\& root.checkpoint_txg == rollback_target_txg)/.filter(|root| is_abandoned(root) \&\& root.checkpoint_txg == rollback_target_txg)/' "$M"; runh "$T"
echo "全部应回到基线绿、每条变异红。"
