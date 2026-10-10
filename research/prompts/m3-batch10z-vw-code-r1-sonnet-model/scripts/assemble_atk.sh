#!/usr/bin/env bash
set -euo pipefail
W=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet
t=$W/tree-b/crates/singlefs-harness/tests
cat "$t/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs" "$W"/src/atk/atk_part*.rs > "$t/atk_pqrs_opus.rs.tmp"
mv "$t/atk_pqrs_opus.rs.tmp" "$t/atk_pqrs_opus.rs"
