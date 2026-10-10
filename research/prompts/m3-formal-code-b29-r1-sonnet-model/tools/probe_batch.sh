#!/usr/bin/env bash
# 探针分批跑：几个进程并行，每个进程一段种子；输出落 $OUT/<NAME>-<起点>.log。
# 用法：TREE=… OUT=… CARGO_TARGET_DIR=… NAME=b29_l2_checker_sweep STARTS="1 4 7 10" SEEDS=3 OPS=6 CAP=400 EVERY=150 bash probe_batch.sh
set -uo pipefail
TREE=${TREE:?}; OUT=${OUT:?}; NAME=${NAME:?}; OPS=${OPS:-6}; SEEDS=${SEEDS:-3}; CAP=${CAP:-400}; EVERY=${EVERY:-200}; SL=${SL:-3}; STARTS=${STARTS:-"0 3 6 9"}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:?}
cd "$TREE"
BIN=$(ls -t $CARGO_TARGET_DIR/debug/deps/${NAME}-* | grep -v '\.d$' | head -1)
for s in $STARTS; do
  { PROBE_SEED_START=$s PROBE_SEEDS=$SEEDS PROBE_OPS=$OPS PROBE_STATE_CAP=$CAP PROBE_CHECK_EVERY=$EVERY PROBE_SECOND_LEVEL=$SL "$BIN" --nocapture > "$OUT/$NAME-$OPS-$s.log" 2>&1; echo "$?" > "$OUT/$NAME-$OPS-$s.rc"; } &
done
wait
for s in $STARTS; do echo "start=$s rc=$(cat $OUT/$NAME-$OPS-$s.rc)"; grep -E '^PROBE' "$OUT/$NAME-$OPS-$s.log"; done
