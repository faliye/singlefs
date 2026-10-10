#!/usr/bin/env bash
# usage: run_mut.sh <name> ; applies mutation, runs three test targets, restores pristine inode_tree.rs
W=$HOME/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet
export CARGO_TARGET_DIR=$W/target
cd $W && python3 tools/mut.py "$1" || exit 9
cd $W/repo
R="nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 10G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 12"
echo "== $1 core inode_tree::"
$R cargo test --offline -p singlefs-core --lib inode_tree:: 2>&1 | grep -E '^test result|FAILED$|error(\[|:)' | head -5
echo "== $1 harness ref test"
$R cargo test --offline -p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference 2>&1 | grep -E '^test result|FAILED$|error(\[|:)' | head -5
echo "== $1 harness absorb_and_borrow_on_unlink_path"
$R cargo test --offline -p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path 2>&1 | grep -E '^test result|FAILED$|error(\[|:)' | head -5
cp $W/pristine/inode_tree.rs $W/repo/crates/singlefs-core/src/inode_tree.rs
