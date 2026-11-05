#!/usr/bin/env bash
# task-server-shards: *
# 样本：照分片协议写的驱动（注释里提到 multi-host-run.sh --peer 不算）
case "${TASK_SERVER_SHARD_PHASE:-}" in
  plan) printf '{"shards": [{"label": "一半"}, {"label": "另一半"}]}\n' > "$TASK_SERVER_SHARD_PLAN_DIRECTORY/plan.json" ;;
  shard) mkdir -p -- "$TASK_SERVER_SHARD_OUTPUT_DIRECTORY" ;;
esac
echo "  → 怎么办：不要直接起 multi-host-run.sh --peer，交服务分片"
bash scripts/crash/crash-amplification-two-hosts.sh some-target --peer-memory-cap-is-not-the-same-word
