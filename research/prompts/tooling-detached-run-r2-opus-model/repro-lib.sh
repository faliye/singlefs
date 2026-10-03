# 复现共用（主 agent 带前缀跑；起真单元，单元名都是 singlefs-detached-opusr2-*，登记目录都是临时的）。source 这一份。
# opusr2_tree <仓根> <工作目录> <脚本文件>：在 <工作目录>/tree 里摆一份 research/scripts/detached-run.sh（就是给的脚本文件），
#   .claude 指到仓里的 .claude（preflight 从脚本旁边的 ../../.claude/scripts/preflight.sh 找）；打出摆好的脚本路径。
opusr2_tree() {
  mkdir -p "$2/tree/research/scripts"
  [[ -e "$2/tree/.claude" ]] || ln -s "$1/.claude" "$2/tree/.claude"
  cp -- "$3" "$2/tree/research/scripts/detached-run.sh"
  printf '%s\n' "$2/tree/research/scripts/detached-run.sh"
}
# opusr2_stop_unit <单元名>：只按名停这一个、清 failed
opusr2_stop_unit() {
  if [[ "$(systemctl --user show -p LoadState --value "$1" 2>/dev/null)" == loaded ]]; then systemctl --user stop "$1" 2>/dev/null; fi
  systemctl --user reset-failed "$1" 2>/dev/null
  return 0
}
# opusr2_residue：打出 singlefs-detached-opusr2-* 还剩几个单元（期望 0）
opusr2_residue() {
  local left
  left="$(systemctl --user list-units --all --plain --no-legend 'singlefs-detached-opusr2-*' 2>&1)"
  echo "残留单元 $(printf '%s' "$left" | grep -c .) 个${left:+：$left}"
}
