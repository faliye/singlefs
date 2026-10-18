#!/usr/bin/env bash
# K3：两个会话同时跑庚二的 gate-staged（落法 A：判的就是开跑时 write-tree 的那棵，基准是开跑时读到的 staged-green），
# 共用一个暂存区与 HEAD，staged-green 后写的覆盖先写的；四个事件之间各插一步别人的暂存区操作（或不插）。
# 用法：k3-concurrent.sh <冻结副本> <工具副本> <草稿根> <事件序列>，事件序列形如 "L.a C.b -.A G.B"：
#   每一项「操作.事件」，操作 - 不做、L 暂存一行错引文（.claude/kb/l.md）、U 把它退出暂存、C 原样提交、G 暂存一行无关的；
#   事件 a / b 是会话甲 / 乙开跑（write-tree、读 staged-green），A / B 是收尾（判那棵树，绿或无对象就前移到它）。
# 末尾再跑一趟不交错的。打一行：staged-green 的树里有没有那行错引文、它有没有在任何一次判里被判红。
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
SEQ="$4"; REPO="$(mktemp -d "$WORK/k3.XXXXXX")"; new_repo "$REPO" >/dev/null
git update-ref refs/sop/staged-green "$(green_tree_commit "$(git write-tree)")"
judged_red=0; declare -A tree base
judge_and_maybe_advance() {   # <树> <基准提交>
  local out rc; out="$(bash -c "$(declare -f judge_tree run_cell); FROZEN='$FROZEN' WORK='$REPO' CELL_PY='$CELL_PY' judge_tree '$1' '$2'" 2>&1)"; rc=$?
  grep -q 'l.md:[0-9]*  E7RESULT name=beta value=2' <<<"$out" && judged_red=1
  if [[ $rc == 0 || $rc == 77 ]]; then git update-ref refs/sop/staged-green "$(green_tree_commit "$1")"; fi
}
for item in $SEQ; do
  op="${item%%.*}"; ev="${item#*.}"
  case "$op" in
    L) printf 'E7RESULT name=beta value=2\n' > .claude/kb/l.md; git add .claude/kb/l.md ;;
    U) git reset -q -- .claude/kb/l.md 2>/dev/null ;;
    C) git commit -qm c --allow-empty ;;
    G) printf 'g\n' >> .claude/kb/g.md; git add .claude/kb/g.md ;;
  esac
  case "$ev" in
    a|b) tree[$ev]="$(git write-tree)"; base[$ev]="$(sg_commit_or_empty)" ;;
    A) judge_and_maybe_advance "${tree[a]}" "${base[a]}" ;;
    B) judge_and_maybe_advance "${tree[b]}" "${base[b]}" ;;
  esac
done
judge_and_maybe_advance "$(git write-tree)" "$(sg_commit_or_empty)"
in_sg="$(git cat-file -e "$(sg_tree):.claude/kb/l.md" 2>/dev/null && git show "$(sg_tree):.claude/kb/l.md" | grep -c 'value=2' || echo 0)"
echo "name=k3 seq=${SEQ// /_} wrong_line_in_staged_green=$in_sg judged_red_ever=$judged_red leak=$([[ $in_sg -gt 0 && $judged_red == 0 ]] && echo yes || echo no)"
cd "$WORK"; rm -rf "$REPO"
