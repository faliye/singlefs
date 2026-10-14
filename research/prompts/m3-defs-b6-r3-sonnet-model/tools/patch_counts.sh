#!/usr/bin/env bash
# Y6：批 6 至批 8 补丁的文件数与加减行数、sha256 前后缀，以及集成副本里 applied 提交的文件数。用法：patch_counts.sh <工作根> <集成副本仓>
set -u
work="${1:?工作根}"; repo="${2:?集成副本仓}"
for d in m3-tooling-b6 m3-tooling-b6-fixup m3-tooling-b7-dispatch m3-tooling-b7-gates m3-tooling-b7-defs m3-tooling-b7-fixup m3-tooling-b8-dispatch m3-tooling-b8-gates m3-tooling-b8-defs m3-tooling-b8-fixup; do
  p="$work/$d/patch.diff"
  stat=$(git apply --numstat "$p" | awk '{a+=$1;b+=$2;n++} END{printf "files=%d plus=%d minus=%d", n, a, b}')
  sum=$(sha256sum "$p" | awk '{print $1}')
  echo "name=patch dir=$d $stat sha256=${sum:0:8}..${sum: -4}"
done
cat "$work/m3-tooling-b8-dispatch/patch.diff" "$work/m3-tooling-b8-gates/patch.diff" "$work/m3-tooling-b8-defs/patch.diff" | grep '^diff --git' | awk '{print $3}' | sort -u | wc -l | sed 's/^/name=b8_union_of_three_patches files=/'
for c in 9407d9a 2b18900 a013292 d9c9859 80e5f38 309071b; do echo "name=commit $(git -C "$repo" show --shortstat --format='%h %s' "$c" | tr '\n' ' ')"; done
for t in after-b6 after-b6-fixup after-b7 after-b7-fixup after-b8 after-b8-fixup; do
  echo "name=exempt_rows tag=$t heavy_thread_exempt=$(git -C "$repo" show "$t:.claude/heavy-thread-exempt" | grep -vc '^\s*#\|^\s*$') long_run_live_hosts_exempt=$(git -C "$repo" show "$t:.claude/long-run-live-hosts-exempt" | grep -vc '^\s*#\|^\s*$')"
done
