#!/usr/bin/env bash
# 样本：包装调了 heavy_thread_default，但取不出时照打 1（不在弄坏开关护着的那一支里、也不写到标准错误）
padded_default_jobs() {
  source "$(dirname "${BASH_SOURCE[0]}")/heavy-threads.sh"
  heavy_thread_default || printf '1\n'
}
