#!/usr/bin/env bash
set -u
base=/tmp/claude-1000/m2-supp3-item1/r2
cd $base/probe/repo || exit 1
PROBE_SEEDS=96 PROBE_OPS=30 nice -n 19 cargo test --release -p singlefs-harness --test scratch_form0_decidability -- --nocapture > $base/probe/form0-96x30.log 2>&1
echo "probe 96x30 exit $?" >> $base/probe.done
PROBE_SEEDS=1000 PROBE_OPS=40 nice -n 19 cargo test --release -p singlefs-harness --test scratch_form0_decidability -- --nocapture > $base/probe/form0-1000x40.log 2>&1
echo "probe 1000x40 exit $?" >> $base/probe.done
