#!/usr/bin/env bash
# 把一份 cargo 日志摘成排序后的几行：PROBE 行、每条用例的 ok / FAILED、test result（去掉耗时）。用法：summarize.sh <日志>
set -u
grep -E '^PROBE_|^test [A-Za-z0-9_:]+ \.\.\. (ok|FAILED)$|^test result' "$1" | sed -E 's/; finished in .*//' | sort
