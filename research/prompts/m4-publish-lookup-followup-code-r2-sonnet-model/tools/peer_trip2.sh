#!/usr/bin/env bash
# peer_trip2.sh：第二台上，在树根（有 before/ after/ probe/ 三份副本与 model/）依次做：构建脚本的几种编法（t4_cells.sh）、
# 改前改后随机历史差分（含重新挂载）、探针的线性查找比较次数、小发布与重新挂载的挂钟。
# 外面已套内存包装。每份副本用自己的编译目录 <副本>/target。
set -u
root=$(pwd)
threads=${THREADS:-31}
prep() { # <副本名> <测试名>
  cp "$root/model/tests/$2.rs" "$root/$1/crates/singlefs-harness/tests/$2.rs"
  find "$root/$1/crates" -type f -exec touch {} +
}
build() { # <副本名> <测试名>
  ( cd "$root/$1" && bash "$root/scripts/hosts/capped.sh" "$threads" cargo test --offline --release -p singlefs-harness --test "$2" --no-run 2>&1 | tail -3 )
}
binary_of() { ls -t "$root/$1"/target/release/deps/"$2"-* 2>/dev/null | grep -v '\.d$' | head -1; }
echo "name=trip2 start nproc=$(nproc)"
bash model/tools/t4_cells.sh
for copy in after before; do
  prep $copy diff_history_remount; build $copy diff_history_remount
  bin=$(binary_of $copy diff_history_remount)
  ( cd "$root/$copy/crates/singlefs-harness" && DIFF_SEED_FIRST=1 DIFF_SEEDS=${DIFF_SEEDS:-256} DIFF_STEPS=${DIFF_STEPS:-64} bash "$root/scripts/hosts/capped.sh" "$threads" "$bin" --exact diff_history_remount_history --nocapture > "$root/diff-$copy.txt" 2>&1 )
  echo "name=diff_run copy=$copy rc=$? $(grep -h '^test result' "$root/diff-$copy.txt" | tail -1)"
  grep '^seed=' "$root/diff-$copy.txt" > "$root/diff-$copy.lines"
done
if cmp -s diff-after.lines diff-before.lines; then v=identical; else v=different; fi
echo "name=differential seeds=${DIFF_SEEDS:-256} steps=${DIFF_STEPS:-64} lines=$(wc -l < diff-after.lines) lines_before=$(wc -l < diff-before.lines) before_vs_after=$v"
diff diff-before.lines diff-after.lines | head -6 | sed 's/^/name=differential_first_difference /' | cut -c1-300
for op in new_inodes write overwrite create_snapshot destroy_snapshot unlink compaction relocate raise_floor remount; do
  echo "name=differential_coverage op=$op steps=$(grep -c " $op" diff-after.lines) ok=$(grep -cE " $op[^ ]* ok" diff-after.lines) err=$(grep -cE " $op[^ ]* err" diff-after.lines)"
done
echo "name=differential_coverage panics_after=$(grep -c ' panic:' diff-after.lines) panics_before=$(grep -c ' panic:' diff-before.lines) remount_ok=$(grep -cE ' remount ok' diff-after.lines)"
# 探针
prep probe probe_scans; build probe probe_scans
bin=$(binary_of probe probe_scans)
( cd "$root/probe/crates/singlefs-harness" && bash "$root/scripts/hosts/capped.sh" "$threads" "$bin" --exact probe_scan_counts --nocapture 2>&1 | grep -E '^name=probe|test result|panicked' | cut -c1-900 )
# 小发布与重新挂载（改前改后）
for copy in before after; do
  prep $copy bench_positions; build $copy bench_positions
  bin=$(binary_of $copy bench_positions)
  echo "name=bench copy=$copy"
  ( cd "$root/$copy/crates/singlefs-harness" && bash "$root/scripts/hosts/capped.sh" "$threads" "$bin" --exact bench_positions_and_remount --nocapture 2>&1 | grep -E '^name=bench|test result|panicked' | cut -c1-400 )
done
# 三棵多层码 2 树各照抄了几个节点（整数遍判法空不空真）：无快照的整份重写用例与 deadlist 池用例
cp "$root/model/tests/carried_counts.rs" "$root/after/crates/singlefs-harness/tests/carried_counts.rs"; find "$root/after/crates" -type f -exec touch {} +
( cd "$root/after" && SINGLEFS_RUN_SCOPE='种类=items; 跑什么=singlefs-harness 的 carried_counts 里无快照整份重写那一条; 范围=1 个测试; 为什么=第二轮正推腿看三棵多层码2树各照抄几个节点，判整数遍判法空不空真' bash "$root/scripts/hosts/capped.sh" "$threads" cargo test --offline -p singlefs-harness --test carried_counts -- --exact a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site --nocapture 2>&1 | grep -E '^name=carried|test result' | cut -c1-400 )
( cd "$root/after" && SINGLEFS_RUN_SCOPE='种类=items; 跑什么=singlefs-harness 的 carried_counts 里 deadlist 池那一条; 范围=1 个测试; 为什么=第二轮正推腿看 deadlist 池两档各照抄几个节点' bash "$root/scripts/hosts/capped.sh" "$threads" cargo test --offline -p singlefs-harness --test carried_counts -- --exact rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart --nocapture 2>&1 | grep -E 'deadlist_pool|test result' | cut -c1-600 )
# 第 11 批整行替换的两行，改前（before 副本、旧锚点）红在哪条断言：对照用
mkdir -p "$root/.pristine-before"; cp -a "$root/before/crates" "$root/.pristine-before/"; mkdir -p "$root/mutout"
python3 -I "$root/model/tools/mutate_run.py" --repo "$root" --pristine "$root/.pristine-before" --work "$root/before" --target "$root/before/target-mut" --out "$root/mutout" --tsv "$root/model/mutations/old-rows.tsv" --rows 1,2 --direct --tag "old-before." 2>&1 | cut -c1-500
bash model/tools/peer_mutants.sh alt3-mutations.tsv 1,2,3
echo "name=trip2 done"
