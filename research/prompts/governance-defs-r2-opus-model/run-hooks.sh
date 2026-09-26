#!/usr/bin/env bash
# 用法：bash run-hooks.sh <草稿目录>
# 每条 cases/*.cmd 按它的前缀（g1-triage → gate-triage，g2-verifier → crash-verifier，g1-main → 主 agent）喂钩子，前台、后台各一次。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
draft="$1"
for case_file in "$here"/cases/*.cmd; do
  name="$(basename "$case_file" .cmd)"
  case "$name" in
    g1-triage-*) agent=gate-triage ;;
    g2-verifier-*) agent=crash-verifier ;;
    g1-main-*|g5-main-*) agent=- ;;
    *) agent=- ;;
  esac
  for background in 0 1; do
    echo "== $name agent=$agent run_in_background=$background"
    bash "$here/feed.sh" "$draft" "$agent" "$background" "$case_file"
  done
done
