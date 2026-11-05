#!/usr/bin/env bash
# 样本：照规范取法写的跑法，一处都不点名
peer_address="$(python3 scripts/hosts/multi_host_addresses.py peer)"
peer_cap="$(grep -m1 '^SLICE_MEMORY_CAP_BY_HOST=' "$SINGLEFS_MULTI_HOST_CONFIG" | cut -d= -f2- | tr ' ' '\n' | sed -n "s/^${peer_address}=//p")"
bash scripts/hosts/multi-host-run.sh --peer --label sample -- bash scripts/hosts/run-with-memory-cap.sh "$peer_cap" true
hosts=1
emit_hosts=1
two_hosts=0
peer_count=2
peer_count="$(bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts | grep -c '^HOST_')"
bash scripts/hosts/run-with-memory-cap.sh --status
