#!/usr/bin/env bash
# 取不出每台默认重活线程数就红：基准里一份记录，这一轮加一份干净的记录（本来判绿）；样本根放 .sample-thread-cap-invalid，
# date-once 取并行路数时按 SINGLEFS_THREAD_CAP 设成了不是正整数的值答，取不出登记值，整格判红、一份都没判，不退回按核数或一路。
# 弄坏开关 date-once-jobs-unregistered-passes 下退回一路、照常判，这份记录干净，退 0。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records
printf '%s\n' '# 基准记录' '' '- 基准里就有的一行。' > records/2026-10-22-基准记录.md
git add -A && git commit -qm base
printf '%s\n' '# 干净样本' '' '- 这一行没有日期。' > records/2026-10-23-干净样本.md
