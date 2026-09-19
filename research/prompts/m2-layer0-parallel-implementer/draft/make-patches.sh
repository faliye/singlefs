#!/usr/bin/env bash
# make-patches.sh：拿 original/（主工作区开工时的快照）与 copy/ 生成两份补丁（-p1，a/ b/ 前缀），再在主工作区今天内容的新副本上试打一遍。
set -euo pipefail
base=/tmp/claude-1000/m2-layer0-parallel
stage="$base/draft/patch-stage"
rm -rf "$stage"; mkdir -p "$stage/a" "$stage/b"
crates_files=(crates/singlefs-harness/src/crash.rs crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs crates/mutations.tsv)
for file in "${crates_files[@]}"; do
  mkdir -p "$stage/a/$(dirname "$file")" "$stage/b/$(dirname "$file")"
  cp "$base/original/$file" "$stage/a/$file"
  cp "$base/copy/$file" "$stage/b/$file"
done
( cd "$stage" && diff -ruN a/crates b/crates >"$base/crates-layer0-parallel.patch" || [[ $? == 1 ]] )
mkdir -p "$stage/a/.claude/gate.d" "$stage/b/.claude/gate.d"
cp "$base/original/gate.d/54-layer0-replay.sh" "$stage/a/.claude/gate.d/54-layer0-replay.sh"
cp "$base/copy/.claude/gate.d/54-layer0-replay.sh" "$stage/b/.claude/gate.d/54-layer0-replay.sh"
( cd "$stage" && diff -ruN a/.claude b/.claude >"$base/gate54-layer0-parallel.patch" || [[ $? == 1 ]] )
sha256sum "$base/crates-layer0-parallel.patch" "$base/gate54-layer0-parallel.patch"
