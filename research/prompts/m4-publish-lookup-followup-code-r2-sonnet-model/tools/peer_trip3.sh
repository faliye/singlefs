#!/usr/bin/env bash
# peer_trip3.sh：第二台上补四样：构建脚本几种编法（改过的 t4_cells2.sh）、三棵树照抄节点数（deadlist 池）、
# 产品构建（不开特性）里 Drop 的代价（t3_drop.sh）、没点名那一格里每个单元现建索引的变异（整份用例文件都跑）。
set -u
root=$(pwd); threads=${THREADS:-31}
echo "name=trip3 start nproc=$(nproc)"
bash model/tools/t4_cells2.sh
cp "$root/model/tests/carried_counts.rs" "$root/after/crates/singlefs-harness/tests/carried_counts.rs"; find "$root/after/crates" -type f -exec touch {} +
( cd "$root/after" && SINGLEFS_RUN_SCOPE='种类=items; 跑什么=singlefs-harness 的 carried_counts 里 deadlist 池那一条; 范围=1 个测试; 为什么=第二轮正推腿看 deadlist 池两档各照抄几个节点' bash "$root/scripts/hosts/capped.sh" "$threads" cargo test --offline -p singlefs-harness --test carried_counts -- --exact rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart --nocapture > "$root/carried-deadlist.txt" 2>&1 )
echo "name=carried_deadlist_pool rc=$? $(grep -c deadlist_pool "$root/carried-deadlist.txt") lines"
grep -E 'deadlist_pool|test result|panicked' "$root/carried-deadlist.txt" | cut -c1-700
bash model/tools/t3_drop.sh
bash model/tools/peer_mutants.sh alt4-mutations.tsv 1
echo "name=trip3 done"
