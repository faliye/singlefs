#!/usr/bin/env bash
# 用法：bash run-hooks.sh <草稿目录>；cases/<agent>--<名>.cmd 以 <agent> 为 agent_type，前台、后台各喂一次。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
draft="$1"
for case_file in "$here"/cases/*.cmd; do
  name="$(basename "$case_file" .cmd)"; agent="${name%%--*}"
  for background in 0 1; do
    echo "== $name agent=$agent run_in_background=$background"
    bash "$here/feed.sh" "$draft" "$agent" "$background" "$case_file"
  done
done
echo "检出记录条数：$(cat "$draft/detections.jsonl" 2>/dev/null | wc -l)"
