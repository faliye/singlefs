#!/usr/bin/env bash
# gate-stage: 项目本地门禁阶段自己会不会红
#
# 拿一组「本该红」和「本该绿」的样本喂给每个 `.claude/gate.d/*.sh`，看它判得对不对。
#
# ⚠️ **这条是实测出来的，不是想出来的**：2026-08-29 复跑复核轮发现
# `30-decision-history.sh`（决策变更留痕）**恒绿**——两条路径塞进一个变量再加引号，
# `git diff -- "$KB"` 拿一个「两条路径粘成的字符串」当单个 pathspec，匹配不到任何文件。
# 它在门禁输出里与一条真的在跑的检查**长得一模一样**。
# 上游的「门禁判别力」阶段只覆盖 `doc-lint.sh`，够不着项目本地这一批。
#
# ⚠️ **没有样本的阶段一律显式列成「未自检」**，不许当成通过
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
#
# 加一个样本：`.claude/gate.d/fixtures/<阶段文件名>/{red,green}/`，
# 里面按仓库结构摆好被判的文件，再写一个 `expect`：
#   exit=1
#   want=失败信息里必须出现的片段     # 红样本至少一条；防「因为别的原因红了」也算过
# 要 git 仓之类的现场，在样本目录里放一个 `setup.sh`——样本先被拷进临时目录，
# setup.sh 在那里跑，所以不会把 `.git` 塞进本仓。
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
GD="$ROOT/.claude/gate.d"
FX="$GD/fixtures"
SELF="$(basename "$0")"

[[ -d "$FX" ]] || { echo "  ✗ 缺样本目录 $FX"; echo "     → 样本要随仓走，否则下一个改门禁的人无从复跑。"; exit 1; }

pass=0; fail=0; nocase=()
for stage in "$GD"/*.sh; do
  name="$(basename "$stage")"
  [[ "$name" == "$SELF" ]] && continue
  if [[ ! -d "$FX/$name" ]]; then nocase+=("$name"); continue; fi
  for kind in red green; do
    d="$FX/$name/$kind"
    [[ -d "$d" ]] || continue
    want_exit="$(sed -n 's/^exit=//p' "$d/expect")"
    # 样本先拷进临时目录再跑：有 setup.sh 的（例如要 git 仓的阶段）在那里建现场，
    # 免得把 .git 之类的东西塞进本仓。
    work="$(mktemp -d)"
    cp -a "$d/." "$work/"
    if [[ -f "$work/setup.sh" ]]; then
      ( cd "$work" && bash setup.sh >/dev/null 2>&1 ) || { printf '  ✗ %-28s %-5s setup.sh 没跑成\n' "$name" "$kind"; fail=$((fail+1)); rm -rf "$work"; continue; }
    fi
    out="$(cd "$work" && bash "$stage" "$work" 2>&1)"; got=$?
    rm -rf "$work"
    okc=1
    [[ "$got" == "$want_exit" ]] || { printf '  ✗ %-28s %-5s 期望退出 %s，实测 %s\n' "$name" "$kind" "$want_exit" "$got"; okc=0; }
    while IFS= read -r w; do
      [[ -z "$w" ]] && continue
      grep -qF -- "$w" <<<"$out" || { printf '  ✗ %-28s %-5s 输出里找不到「%s」\n' "$name" "$kind" "$w"; okc=0; }
    done < <(sed -n 's/^want=//p' "$d/expect")
    if ((okc)); then printf '  ✓ %-28s %-5s 判得对\n' "$name" "$kind"; pass=$((pass+1)); else fail=$((fail+1)); fi
  done
done

if ((${#nocase[@]})); then
  echo "  ! 这些阶段还没有判别力样本，本阶段**没有验过它们**："
  printf '      %s\n' "${nocase[@]}"
  echo "    → 一条永远不红的检查与没有这条检查，在门禁输出里长得一模一样。落点：kb/checks-owed.md"
fi
((fail == 0)) || exit 1
echo "  ✓ 有样本的阶段判得都对（$pass 个样本），$((${#nocase[@]})) 个阶段仍未自检"
