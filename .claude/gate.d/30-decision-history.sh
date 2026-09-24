#!/usr/bin/env bash
# gate-stage: 决策变更留痕
#
# 改了决策正文却没在决策变更史（`kb/decisions-history/<年-月>.md`）留条目 ⇒ 判红。
#
# 为什么判红而不是提醒：`.claude/rules/format-evolution.md` 已定
# 「决策变更**必须记进** decisions-history.md，含推翻依据」；
# 而 kb-discipline 又要求正文只写现状 ⇒ **被推翻的那句话会被直接改掉**，
# 不留条目就等于它从未存在过，三个月后没人知道它为什么不算数了。
#
# ⚠️ 判据是「有没有新增条目」，不判「条目写得对不对」——后者只有人能判。
#
# ⚠️ **本阶段自己恒绿过一段时间**（2026-08-29 复跑复核轮查出）：两个路径被塞进一个变量再加引号，
# `git diff -- "$KB"` 于是拿一个「两条路径粘成的字符串」当单个 pathspec，**匹配不到任何文件**。
# 加上历史外置到 decisions-history.md 之后仍在决策正文里找 `### 2026-`，判据也早已错位。
# ⇒ 路径改成数组，历史条目改到正确的文件里数。**双向证过会红**，见文末注释。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
KB=(.claude/kb/decisions.md .claude/kb/decisions)
# 变更史 2026-09-12 起按月拆档：条目住 decisions-history/<年-月>.md，decisions-history.md 放快查表。
# 新条目可能落在还没进 git 的新月份文件里，git diff HEAD 看不见它 ⇒ 那几份里的条目整份算新增。
shopt -s nullglob
HIST=(.claude/kb/decisions-history.md .claude/kb/decisions-history/*.md)
shopt -u nullglob
HIST_THIS_MONTH=".claude/kb/decisions-history/$(date +%Y-%m).md"
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

# 不在 git 仓里没有「改了什么」可比：退 77（本次无对象可判）。老写法把 git 的报错丢进 /dev/null，
# 每一道 git 都失败、行数读成 0，于是「只改了 0 行，按小改动放行」报绿。
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "  ! 不在 git 仓库里，本阶段无对象可判"
  exit 77
fi
# 在仓里时每一道 git 都要成功：失败了这一阶段什么都没比，判红，不许读成「没改」。
git_failed() {
  bad "git $1 跑不起来（退出码 $2），决策正文改了多少、变更史加了几条都没数出来"
  howto "按上面 git 的报错修好仓库状态再跑（常见是还没有任何提交、HEAD 不存在：先提交一次）；"
  howto "  git 失败时这一阶段什么都没比，不是通过。"
  exit 1
}
diff_rc=0
git diff --quiet HEAD -- "${KB[@]}" || diff_rc=$?
case "$diff_rc" in
  0) echo "  ! 决策正文与 HEAD 无差异，本阶段无对象可判"; exit 77 ;;
  1) ;;
  *) git_failed "diff --quiet HEAD" "$diff_rc" ;;
esac

# 决策正文改了多少行（增 + 删，各文件相加）
numstat="$(git diff HEAD --numstat -- "${KB[@]}")" || git_failed "diff --numstat" "$?"
changed=$(awk '{n+=$1+$2} END{print n+0}' <<<"$numstat")
# 本次 diff 往变更史里加了几条日期标题
added=0
if ((${#HIST[@]})); then
  hist_diff="$(git diff HEAD -- "${HIST[@]}")" || git_failed "diff（变更史）" "$?"
  added=$(grep -c '^+### 20[0-9][0-9]-' <<<"$hist_diff" || true)
  untracked="$(git -c core.quotepath=false ls-files --others --exclude-standard -- "${HIST[@]}")" || git_failed "ls-files --others" "$?"
  while IFS= read -r untracked_history; do
    [[ -n "$untracked_history" ]] || continue
    untracked_count=$(grep -c '^### 20[0-9][0-9]-' "$untracked_history" || true)
    added=$((added + untracked_count))
  done <<<"$untracked"
fi

if [[ "$added" -gt 0 ]]; then
  ok "决策正文改了 $changed 行，变更史新增 $added 条条目"
  exit 0
fi
if [[ "$changed" -le 4 ]]; then
  ok "决策正文只改了 $changed 行，按小改动放行"
  exit 0
fi

bad "决策正文改了 $changed 行，却没往变更史新增任何条目"
howto "若这次改动推翻或定下了任何结论，在 $HIST_THIS_MONTH 的「## 历史版本」下最上面加一条"
howto "  （这个月的文件还没有就新建，文件头照上个月那份写），标题下写两行快查，再跑 49 号 --write："
howto "  ### $(date +%F)  —— 曾经 X / 现在 Y / 依据 Z"
howto "纯排版改动可以拆成单独一个提交，那时这一项就无对象可判了。"
exit 1
