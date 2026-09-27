#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 状态一致性：未定项有没有被别处定了
#
# 还 checks-owed.md C31（未定项被别处定了却没回收）。
#
# **判据**：一个还开着的未定项，如果它的正文点名了另一个决策，
# 而那个决策的**状态行**在它最后一次被改动之后又变过 —— 就要求复核。
# 「它的正文」是整个条目块：首行加它下面的续行，到下一条分项（编号列表项或「| k |」表格行）或下一个标题之前、至多 20 行，
# 与 lib-open-item-review.py 的条目块同一个口径——列表式条目常把「依赖 Dn」写在续行里，只看首行就漏了。
#
# ⚠️ **纯文本的依赖图抓不到这一类**：实测 D22 的三个陈旧未定项里，
# 两个根本没有「前置是某某」这种标记（一个是「本事务内释放的块不得重用」
# 与 D16 新规则 2 撞了，一个是「D2 的口径要扩」而 D2 早就扩完了）。
# 能抓住它们的只有「谁比谁新」这个时间判据。
# ⚠️ **本检查对 2026-08-29 决策文档拆分之前的历史无效**：
# 拆分把一份 4830 行的 decisions.md 变成 25 个文件，逐行历史在那里断了，
# 两侧的时间戳都塌到拆分那一次提交。**它管的是今后**。
# 判别力在一次性合成仓里双向证过：D2 后定而 D22 的项没动 ⇒ rc=1；
# 把那条项改写成「已定」之后 ⇒ rc=0。
# git 取不到时间（对象缺了、仓坏了）判红，不当「这条决策从没定过」：那样点名它的未定项一条都不比，照样报绿。
# 样本：fixtures/60-stale-open-items.sh/red 另造一个缺了旧版本对象的仓：一份决策的 git log -G 读不到，
# 一份未定项的条目块 git log -L 读不到，两处都要报出来。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
REVIEW_LIB="$(cd "$(dirname "$0")" && pwd)/lib-open-item-review.py"
DEC=.claude/kb/decisions
# 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
[[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，本阶段无对象可判"; exit 77; }
git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，本阶段跳过"; exit 77; }

# 每个决策状态行最后一次变动的提交时间；取不到（git 失败）记 -1，点名它的未定项逐条报「没比过」
declare -A st_time
time_unavailable=()
for f in "$DEC"/*.md; do
  n=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}') || continue
  [[ -n "$n" ]] || continue
  # ⚠️ **不能拿标题行当「这个决策定了新东西」的信号**——标题行几乎不变，
  # 实测在已知有三处陈旧未定项的历史点上一条都不报。
  # 改用「这个文件里新增过『已定』标记」的那次提交时间。
  # ⚠️ **必须匹配「小节标题」，不能匹配正文里的「已定」二字。**
  # 用 -S'—— 已定' 时，正文里一句「D21 已定索引是派生态」也会命中，
  # 于是任何一次给决策补实验结论都会把引用它的未定项全报一遍（实测三处假阳性）。
  # -G 加行首锚定只认 `## …—— 已定` / `### …—— 已定` 这种标题行。
  # git 自己的退出码先判，再取第一行：写成「git log … | head -1」时管道只交 head 的退出码，git 失败被读成「从没定过」
  if ! t_out="$(git log -1 --format=%ct -G'^#{2,4} .*—— 已定' -- "$f")"; then
    st_time[$n]=-1
    time_unavailable+=("$n（$(basename "$f")）")
    continue
  fi
  t="$(head -n 1 <<<"$t_out")"
  # ⚠️ **没有兜底。** 第一版取不到时退化成「文件最后修改时间」，
  # 于是任何一次给决策补内容都会把引用它的未定项全报一遍。
  # 从没加过「—— 已定」小节标题 = 它没定过任何东西 = 不该触发任何复核。
  [[ -n "$t" ]] || t=0
  st_time[$n]=$t
done

flagged=0
if ((${#time_unavailable[@]})); then
  flagged=1
  echo "  ✗ 这些决策的「—— 已定」小节最近一次变动时间取不到（git log 失败），点名它们的未定项这一轮没法比：${time_unavailable[*]}"
  echo "     → 怎么办：按上面 git 的报错修好仓库（git fsck 看缺了哪些对象，从远端或备份找回）再跑；取不到时间不是「这条决策从没定过」，不是通过。"
fi
open_items=0     # 扫到的未定项条数
judged_items=0   # 其中点名了「定过东西的别的决策」、真拿去比过复核时间的条数
for f in "$DEC"/*.md; do
  self=$(grep -m1 -oE '^## (D[0-9]+)' "$f" | awk '{print $2}')
  # 未定项行：编号开头且标着未定
  while IFS=: read -r ln text; do
    [[ -n "$ln" ]] || continue
    open_items=$((open_items + 1))
    # ⚠️ **要看整个条目块的最新改动，不是首行。**
    # 复核通常写在条目下面，首行的 blame 时间不动——只看首行会让
    # 复核过的条目永远红着，检查退化成噪声（实测踩过）。
    # ⚠️ **行号必须先映射回 HEAD 的版本，不能直接拿工作区的行号喂 `git log -L`。**
    # `-L` 的行范围是按**历史里的文件**解释的：工作区一旦有未提交改动，
    # 上方插几行就让整段偏移，`-L` 去历史里看的是另一段内容（实测：工作区 140 行、
    # HEAD 131 行，140 在 HEAD 里是空行）⇒ 取不到时间戳 ⇒ **把复核过的条目报成陈旧**。
    # 这是假红，而假红压倒真红的检查等于没有检查
    # （`.claude/singlefs-ai-sop/rules/show-me-test.md`）。
    # 判别力：改前在「工作区有未提交改动且条目上方插过行」时必红，改后转绿。
    # ⚠️ **「条目块动过」不等于「复核过」**（2026-09-12 改）：此前拿条目块最后一次改动的时间当复核时间，
    # 于是**任何**一次改动都能把红消掉——2026-09-11 往 D19 未定项 6 那一格补性能数（与 D16 无关），
    # 它对 D16 状态变动的那一红就这样没了，D16 那两句还成不成立是事后手核的，门禁没逼。
    # ⇒ 复核时间按「被点名的那条决策」逐条算：条目块历史里最近一次让它的点名次数变多的那次提交
    #   （条目诞生那次也算）；工作区里条目块比 HEAD 多点了它一次，算刚复核过。
    #   复核写一句「（YYYY-MM-DD 复核 Dn：……）」就满足；改别的地方不算。判据住在 lib-open-item-review.py。
    deps=()
    # 点名按整个条目块认（首行加续行，到下一条分项或下一个标题之前、至多 20 行），与 lib-open-item-review.py 的 block_end 同一个口径
    block_text="$(awk -v s="$ln" 'NR == s { print; next }
        NR > s && NR <= s + 20 { if (/^[[:space:]]*[0-9]+\. / || /^##+ / || /^\|[[:space:]]*[0-9]+[[:space:]]*\|/) exit; print }' "$f")"
    # ⚠️ D 编号前面要是非字母数字：「RAID5」里的「D5」不是在点名 D5（实测：D2 未定项 15 因此被报成依赖 D5，
    # 改判据之前那一红被任何一次改动顺手消掉，改判据之后永远复核不掉）。
    for d in $(grep -oE '(^|[^A-Za-z0-9])D[0-9]+' <<<"$block_text" | grep -oE 'D[0-9]+' | sort -u); do
      [[ "$d" == "$self" ]] && continue
      dt=${st_time[$d]:-0}
      if (( dt < 0 )); then
        echo "  ✗ $(basename "$f"):$ln 的未定项点名了 $d，而 $d 的变动时间取不到，这一条对 $d 没比过"   # gate-lint:detail
        continue
      fi
      (( dt > 0 )) && deps+=("$d:$dt")
    done
    (( ${#deps[@]} > 0 )) || continue
    judged_items=$((judged_items + 1))
    # 复核判据的输出先落到变量、判过退出码再读：接进 `< <(…)` 时它崩了只是少打几行，这一条被读成「复核过了」
    review_out="$(python3 "$REVIEW_LIB" "$f" "$ln" "${deps[@]}")" || {
      review_rc=$?
      echo "  ✗ $(basename "$f"):$ln 的复核判据没跑成（lib-open-item-review.py 退出码 $review_rc），这一条没比过"
      echo "     → 按上面的报错修 .claude/gate.d/lib-open-item-review.py 或这一份决策正文；判据没跑成不是通过。"
      exit 1
    }
    while IFS= read -r d; do
      [[ -n "$d" ]] || continue
      echo "  ✗ $(basename "$f"):$ln 的未定项点名了 $d，而 $d 的状态行在它之后变过"
      echo "     ⇒ 复核这一项是不是已经被 $d 定掉了；复核完在这一条里写一句点名 $d 的复核记录。原文：${text:0:60}"
      flagged=1
    done <<<"$review_out"
  # ⚠️ **列表式未定项的行内不含「未定」二字**——那两个字在小节标题上。
  # 第一版按行内关键字过滤，把 D22 那三条陈旧项全滤掉了，于是检查恒绿。
  # 改成：取「### 未定项」小节内的条目行，再排掉已经标了「已定」的。
  done < <(awk -F: '
      # ⚠️ 只认**光秃秃的**「### 未定项」标题。
      # 「### 未定项 3 —— 已定」也以它开头，第一版把已定案小节里的表格行
      # 全当成未定项抓了进来，报出两处假阳性。
      # ⚠️ **在第一个 `####` 处也收口**：小节里索引表之后是各分项各自的论证，
      # 那里面另有编号列表，不收口会把论证的第 1/2/3 条当成分项。
      /^### 未定项[[:space:]]*$/  { inside=1; next }
      /^### /              { inside=0 }
      /^#### /             { inside=0 }
      /^## /               { inside=0 }
      inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
    ' "$f")
  # ⚠️ **不再按行内关键字滤掉「已定」**：已定的分项现在住在「### 已定项」小节里，
  # 这一节按定义全是未定的。老版本那道 `grep -v 已定` 有个静默盲区——
  # 一条**未定**分项只要正文里提到别处的「已定」（例：D18 的「与 D16 已定的
  # checkpoint 序号怎么共存」）就会被滤掉，从此不被本阶段看一眼。
done

if ((flagged)); then
  echo "     → 怎么办：逐条复核；已被别处定掉的就改写成「已定，权威记录在 XX」，"
  echo "               仍然开着的就把点名改成不构成依赖的写法。"
  exit 1
fi
# 没有一条未定项点名定过东西的别的决策，这一轮一次复核时间都没比：退 77，不报绿
# （`.claude/singlefs-ai-sop/rules/show-me-test.md`「扫到 0 项也不是通过」）。
if ((judged_items == 0)); then
  echo "  ! 查了 $open_items 条未定项，没有一条点名定过东西的别的决策，本阶段无对象可判"
  exit 77
fi
echo "  ✓ 没有未定项被别处的更新甩在后面（查了 $open_items 条未定项，其中 $judged_items 条点名了定过东西的别的决策、逐条比过复核时间）"
