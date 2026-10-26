#!/usr/bin/env bash
# probe.sh <仓根> <草稿目录>：探针副本（probe）里编 tests/attack_followup_probe.rs，量几处没改的线性查找比较了几次（release 档；六个场景各一条线程；池 16、64、256 个八单元的文件；
# 快照下整份重写 8、32、128 个单元的文件），打出 name=probe 那几行。
set -u
repo=$1; work=$2
( cd "$work/probe" && CARGO_TARGET_DIR="$work/target-probe" nice -n 19 bash "$repo/scripts/hosts/two-logs.sh" --quiet "$work/probe-build" -- \
    bash "$repo/scripts/hosts/capped.sh" 31 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G \
    cargo test --offline --release -p singlefs-harness --test attack_followup_probe --no-run ) || { echo "name=probe build_failed"; exit 1; }
binary=$(grep -o "$work/target-probe/release/deps/attack_followup_probe-[0-9a-f]*" "$work/probe-build.log" | tail -1)
( cd "$work/probe/crates/singlefs-harness" && nice -n 19 bash "$repo/scripts/hosts/capped.sh" 31 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G \
    "$binary" --exact attack_followup_probe_of_the_unchanged_linear_lookups --nocapture ) > "$work/probe.txt" 2>&1
echo "name=probe_run rc=$? $(grep -h "^test result" "$work/probe.txt" | tail -1 | sed "s/; finished in [0-9.]*s//")"
grep -h '^name=probe ' "$work/probe.txt"
