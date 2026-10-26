#!/usr/bin/env bash
# 样本：照规范取法写的跑法，一处都不点名
MEMORY_CAP="${SAMPLE_MEMORY_CAP:-auto}"
bash scripts/hosts/run-with-memory-cap.sh auto "$BINARY" list-items
bash scripts/hosts/run-with-memory-cap.sh "$MEMORY_CAP" "$BINARY" run
peer_cap="$(grep -m1 '^PEER_MEMORY_CAP=' "$SINGLEFS_MULTI_HOST_CONFIG" | cut -d= -f2)"
bash scripts/hosts/multi-host-run.sh --peer --label sample -- bash scripts/hosts/run-with-memory-cap.sh "$peer_cap" true
hosts=1
emit_hosts=1
two_hosts=0
peer_count=2
peer_count="$(bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts | grep -c '^HOST_')"
RUN_WITH_MEMORY_CAP_RESERVE=20G bash scripts/hosts/run-with-memory-cap.sh --status
