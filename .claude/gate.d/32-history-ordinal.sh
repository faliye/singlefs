#!/usr/bin/env bash
# gate-stage: 本次新增的历史条目有没有撞号
#
# 变更史的条目标题形如 `### 2026-09-01（其十六）：……`，同一天按「其 N」顺序编号。
# **这个编号是先到先得的公共资源**：并发会话各写各的，取号前不查最大号就会撞。
#
# ⚠️ **实测代价**（2026-08-30）：两个会话同一天在同一个仓里工作，
# 两条历史条目都取了「其十三」，后发现的那个会话把自己那条改成「其十四」避让。
# 撞号的后果不是难看——`records/` 里有用「2026-08-28（其十一）」当锚点的引用，
# 一个日期下有两条同号条目时，那个锚点指向哪一条无从判断。
# 依据：`.claude/singlefs-ai-sop/rules/session-wrapup.md` 第 4 节「公共编号先到先得」。
#
# **判据只看本次 diff 新增的条目**，理由是历史条目一旦提交就被别处引用，
# 回头改号会把那些引用一起改坏。⇒ 拦的是「取号的那一刻」，不是既成事实。
# ⚠️ 因此**存量撞号不判红，只如实报数**——把它读成「存量是干净的」是错的。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
# 决策变更史 2026-09-12 起按月拆档（<年-月>-decisions-history.md），几份都要查
shopt -s nullglob
HIST=(.claude/kb/*decisions-history.md .claude/kb/experiments-history.md)
shopt -u nullglob
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

# 一行标题里取出「日期（其 N）」这把钥匙；取不到的（没带序号的条目）不参与判定。
key_of() { grep -o '^### 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]（其[^）]*）'; }

hits=(); legacy=0; checked=0
# HEAD 里各份变更史已有的条目标题（整行）。按月拆档时条目整份搬进 HEAD 里还没有的新文件，
# 搬过去的不是新取的号——不排掉的话，存量撞号会在搬家那一次被当成新撞号判红。
# ⚠️ 按整行比，不按「日期（其 N）」比：新写的一条若又取了已有的号，钥匙与 HEAD 里那条相同，
# 按钥匙排会把它一起排掉，这一路就永远不红（2026-09-12 写这段时实测）。
head_headings="$(for f in "${HIST[@]}"; do git show "HEAD:$f" 2>/dev/null; done | grep '^### 20' || true)"
for f in "${HIST[@]}"; do
  [[ -f "$f" ]] || continue
  checked=$((checked + 1))
  # 存量：以 HEAD 那份为准数重复，只报数不判红
  if git rev-parse --verify HEAD >/dev/null 2>&1; then
    n=$(git show "HEAD:$f" 2>/dev/null | key_of | sort | uniq -d | wc -l)
    legacy=$((legacy + n))
  fi
  # 本次新增的条目标题
  if git cat-file -e "HEAD:$f" 2>/dev/null; then
    mapfile -t added < <(git diff HEAD -- "$f" 2>/dev/null | sed -n 's/^+//p' | key_of)
  else
    # HEAD 里还没有的新文件（暂存了没暂存都算）：HEAD 里哪一份变更史都没有的标题，才是本次新写的条目
    mapfile -t added < <(grep '^### 20' "$f" | grep -vxF -f <(printf '%s\n' "$head_headings") | key_of || true)
  fi
  ((${#added[@]})) || continue
  # 撞号有两种：与文件里已有的条目撞，或本次新增的两条自己撞
  all="$(key_of < "$f")"
  for k in "${added[@]}"; do
    cnt=$(grep -cxF "$k" <<<"$all")
    ((cnt > 1)) && hits+=("$f  $k  在该文件里出现 $cnt 次")
  done
done

if ((checked == 0)); then
  ok "没有变更史文件，本阶段无对象可判"
  exit 0
fi

if ((${#hits[@]})); then
  bad "本次新增的历史条目里有 ${#hits[@]} 处撞号"
  printf '     %s\n' "${hits[@]}"
  howto "取号前先查该文件里当天的最大号： grep -o '^### <日期>（其[^）]*）' <变更史文件> | sort -u"
  howto "把自己这条改成下一个没被占的号——先到先得，改别人已提交的那条会改坏引用它的锚点。"
  howto "并发会话的完整对法见 .claude/singlefs-ai-sop/rules/session-wrapup.md 第 4 节。"
  exit 1
fi
ok "本次新增的历史条目没有撞号（存量 $legacy 处撞号不在本阶段射程，见文件头）"
