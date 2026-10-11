#!/usr/bin/env bash
# m3-batch10z-jk-code-r1 攻方 s2 复跑。前置：把冻结副本 rsync 一份到 $REPO（--exclude target .git），
# 打上 ../m3-r1/batch10z-j/keep/measure-arms-on-top-of-the-patch.diff 与本目录 model-arms-on-mount.diff（后者含 s2fb 臂），
# 把 opus_s2_probe.rs、opus_s2_device_lock_stress.rs 放进 $REPO/crates/singlefs-harness/tests/，
# common_unwitnessed_publish-mod.rs 已是副本里现成的（起点）。每件直接经内存包装起，不经 shell 函数包 env。
set -euo pipefail
REPO=${REPO:?set REPO to the rsynced copy}
CAP=${CAP:-8G}
CARGO_TARGET_DIR="$REPO/../target" bash scripts/hosts/run-with-memory-cap.sh "$CAP" bash scripts/hosts/capped.sh 10 cargo build --release -p singlefs-harness --test opus_s2_probe
CARGO_TARGET_DIR="$REPO/../target" bash scripts/hosts/run-with-memory-cap.sh "$CAP" bash scripts/hosts/capped.sh 10 cargo build --release -p singlefs-harness --test opus_s2_device_lock_stress
B=$(ls -t "$REPO/../target"/release/deps/opus_s2_probe-* | grep -v '\.d$' | head -1)
# K2-3 可达面（384 与 1536 两档，五组故障，补法臂）：
env S2_SLOTS=384 S2_PREFIXES=oooo S2_RS="b1;b3" "S2_FAULTS=racct;racct,rlin;rall;racct,oldroot;racct,oldacct" S2_ALPHABET=onsFb S2_COUNT=4 S2_SEED=31 S2_SHORTEST=20 S2_LONGEST=34 S2_PREROOT=1 S2_LABEL=k23 \
  bash scripts/hosts/run-with-memory-cap.sh "$CAP" bash scripts/hosts/capped.sh 1 "$B" --nocapture s2_sweep
# 认不出回退的 s2fb 退路臂（把 SINGLEFS_PROBE_ARM=s2fb 放最前）：
env SINGLEFS_PROBE_ARM=s2fb S2_SLOTS=384 S2_PREFIXES=oooo S2_RS=b1 "S2_FAULTS=rall;racct,oldroot" S2_ALPHABET=onsFb S2_COUNT=4 S2_SEED=31 S2_SHORTEST=20 S2_LONGEST=34 S2_PREROOT=1 S2_LABEL=k23fb \
  bash scripts/hosts/run-with-memory-cap.sh "$CAP" bash scripts/hosts/capped.sh 1 "$B" --nocapture s2_sweep
# 阳性对照（今天的代码，回退复活应红）：
env SINGLEFS_PROBE_ARM=today S2_SLOTS=384 S2_PREFIXES=oooo S2_RS=b1 S2_FAULTS=racct S2_ALPHABET=onsFb S2_COUNT=4 S2_SEED=31 S2_SHORTEST=20 S2_LONGEST=34 S2_PREROOT=1 S2_LABEL=posctl \
  bash scripts/hosts/run-with-memory-cap.sh "$CAP" bash scripts/hosts/capped.sh 1 "$B" --nocapture s2_sweep
# K4 两把锁并发压测：
L=$(ls -t "$REPO/../target"/release/deps/opus_s2_device_lock_stress-* | grep -v '\.d$' | head -1)
env S2_THREADS=10 S2_ROUNDS=5000 S2_DROP_MS=0 bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 10 "$L" --nocapture stress_two_locks
env S2_THREADS=10 S2_ROUNDS=800 S2_DROP_MS=3 bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 10 "$L" --nocapture stress_two_locks
