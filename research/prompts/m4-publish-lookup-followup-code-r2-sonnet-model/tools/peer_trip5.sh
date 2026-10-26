#!/usr/bin/env bash
# peer_trip5.sh：只在探针副本里跑 probe_scan_counts（场景 D 带恢复路径上 UnitsOfTheMappedRolesTakingTheirMappingPositions 那一处的次数）。
set -u
root=$(pwd); threads=${THREADS:-31}
cp "$root/model/tests/probe_scans.rs" "$root/probe/crates/singlefs-harness/tests/probe_scans.rs"; find "$root/probe/crates" -type f -exec touch {} +
( cd "$root/probe" && bash "$root/scripts/hosts/capped.sh" "$threads" cargo test --offline --release -p singlefs-harness --test probe_scans --no-run 2>&1 | tail -2 )
bin=$(ls -t "$root"/probe/target/release/deps/probe_scans-* 2>/dev/null | grep -v '\.d$' | head -1)
( cd "$root/probe/crates/singlefs-harness" && bash "$root/scripts/hosts/capped.sh" "$threads" "$bin" --exact probe_scan_counts --nocapture 2>&1 | grep -E '^name=probe|test result|panicked' | cut -c1-1200 )
