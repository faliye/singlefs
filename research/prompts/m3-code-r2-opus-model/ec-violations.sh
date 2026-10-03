#!/usr/bin/env bash
# EC 下转圈那几格（txg 27、28、29，切在根槽之前，卸载、准入抬 F、回退到前一版）入口之后池级 checker 报的是哪几条。
set -u
D=/tmp/claude-1000/m3-code-r2-opus; W=$D/work; F=/tmp/claude-1000/m3-code-r2/tree
export CARGO_TARGET_DIR=$D/target
python3 $D/model/wrongrun.py apply $W $F EC_any_self_verified_root_in_the_three_other_entries || exit 3
(cd $W && nice -n 19 cargo test --offline -p singlefs-harness --test r2opus_entries_proto --no-run > $D/logs/ec-violations-build.log 2>&1); echo "build rc=$?"
BIN=$(ls -t $CARGO_TARGET_DIR/debug/deps/r2opus_entries_proto-* | grep -v '\.d$' | head -1)
cd $W/crates/singlefs-harness
R2_CUTS=before R2_TXGS=27,28,29 R2_ENTRIES=unmount,raise,rollback R2_ROLLBACK_PREV=1 nice -n 19 $BIN r2opus_y3 --nocapture --test-threads 1 > $D/logs/ec-violations.log 2>&1; echo "run rc=$?"
python3 $D/model/wrongrun.py restore $W $F EC_any_self_verified_root_in_the_three_other_entries
diff -rq --exclude=target $F/crates $W/crates | grep -v r2opus
