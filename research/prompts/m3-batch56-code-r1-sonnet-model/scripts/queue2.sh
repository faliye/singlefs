#!/usr/bin/env bash
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
cd $W
echo "== c789 (product-mode-only weakenings, combined)"
bash mutm.sh c789 crates/singlefs-checker/src/walk.rs c7,c8,c9 -p singlefs-harness --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full -- tombstone lineage retirement container 2>&1 | cut -c1-200
echo "== ca"; bash mut.sh ca crates/singlefs-core/src/recovery.rs $W/mutdefs/ca.old $W/mutdefs/ca.new -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking 2>&1 | grep -E '^test .*FAILED|^test result|exit=|ANCHOR|^error' | cut -c1-200
echo "== fuzz new"; bash fuzzrun.sh fuzz-new new
echo "== fuzz old"; bash fuzzrun.sh fuzz-old old
echo "== old rows"
for pre in "第52批 冷走读记账条目数排回条目宽之前" "第52批 重建记账条目数排回条目宽之前" "C586：回退那次的树 ID 水位只取环里" "第24批 K M5"; do
  bash runprefix.sh "old_$(echo "$pre" | cksum | cut -d' ' -f1)" "$pre" 2>&1 | grep -E '^== |^test .*FAILED|^test result|exit=|ANCHOR|^error' | cut -c1-200
done
echo "== kb2"; bash mut.sh kb2 crates/singlefs-core/src/mount.rs $W/mutdefs/kb2.old $W/mutdefs/kb2.new -p singlefs-core --lib -- tree_identifier_watermark_of_the_rollback_tests 2>&1 | grep -E '^test .*FAILED|^test result|exit=|ANCHOR|^error' | cut -c1-200
