#!/usr/bin/env bash
# 样本：格并行的缺省路数的包装：只经 heavy_thread_default 取，取不出的原因写到标准错误，弄坏开关护着的那一支才打 1
sample_default_jobs() {
  local library status
  library="$(dirname "${BASH_SOURCE[0]}")/heavy-threads.sh"
  if source "$library"; then
    heavy_thread_default && return 0
    status=$?
  else
    status=$?
    printf '  source 不了 %s\n' "$library" >&2
  fi
  if [[ "${SAMPLE_STAGE_BREAK:-}" == jobs-as-one ]]; then
    printf '1\n'
    return 0
  fi
  return "$status"
}
