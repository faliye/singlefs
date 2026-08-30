#!/usr/bin/env bash
# gate-stage: 定了新东西之后有没有回头看同文件的未定项
#
# 还 checks-owed.md C36（未定项检查的三个盲区）的前两条。
#
# **它与 60 阶段是两把不同的尺子，不是加强版**：
#   60：**跨文件 + 看历史**——未定项点名了别的决策，而那个决策后来定过东西。
#   61：**同文件 + 看本次 diff**——本次 diff 在某个决策里新增了一个「已定」小节，
#       而同一个文件里还开着的未定项**这次一个都没碰**。
#
# ⚠️ **为什么必须看 diff 而不是看历史**：按历史判会在每个「有已定小节又有未定项」的
# 决策上恒红（本仓 D9 / D12 / D14 都是这个形状），而 decisions.md D13 自己写过
# 「不可复现的红训练人忽略红灯」——一个恒红的检查比没有检查更坏。
# 看 diff 则只在**真正发生了定案的那一次提交**上说话，其余时候安静。
#
# **它抓的两个实测形态**（2026-08-29 审计轮，一轮之内各撞一次）：
#   ① D25 同一个文件里既写「已定：取粗粒度 8 叶 1 脊柱」，
#      又在「还要回答的」里留着「目标负载的两个数取什么值。这一步只能由人定」。
#   ② decisions.md 索引页末尾「待议：记账与反向索引的隔离纪律」自 2026-08-25 悬着，
#      而 D6 定案取「付 O(N) 次反向索引查找」已经实质选中了它要禁的那件事。
#      60 阶段看不见它，因为 60 只扫 decisions/ 下的正文文件，**不扫索引页**。
# ⚠️ **`git diff --name-only` 对非 ASCII 文件名默认做 C 转义**（`"\347\233\256…"`），
# 而本仓**每一个决策文件名都是中文** ⇒ 拿转义后的串再去 `git diff -- "$f"` 匹配不到任何文件
# ⇒ **检查恒绿**。必须带 `-c core.quotepath=false`。
# 这个 bug 在合成仓的双向验里当场暴露，是「新增的检查必须先证明它会红」抓到的第二个。
set -uo pipefail
DEC=.claude/kb/decisions
IDX=.claude/kb/decisions.md
[[ -d "$DEC" ]] || { echo "  ✓ 没有 $DEC，无对象可判"; exit 0; }
git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，本阶段跳过"; exit 0; }

# diff 基准：与共享门禁的 Show me test 同一套口径
BASE="${GATE_BASE:-}"
if [[ -z "$BASE" ]]; then
  for def in master main; do
    if git rev-parse --verify -q "$def" >/dev/null; then
      BASE="$(git merge-base HEAD "$def" 2>/dev/null)" && break
    fi
  done
fi
[[ -n "$BASE" ]] || BASE=HEAD

# 本次 diff 里**新增**了「已定」小节标题的决策文件
settled_files=()
while IFS= read -r f; do
  [[ -n "$f" ]] || continue
  # 只认新增行（+），且必须是小节标题：`## D1 简称 —— 已定` / `### 未定项 3 —— 已定` / `### 已定（…）`
  # ⚠️ **这个正则第一版写坏过，而且是「写坏了但恒绿」那种坏法**：
  # 写成 '^\+#{2,4} .*(—— 已定|^\+### 已定)' 时，第二个分支里的 ^ 在组内永远匹配不上，
  # 于是 `### 已定（…）` 这种最常见的定案小节标题一个都抓不到，检查恒绿。
  # 合成仓双向验的时候当场红——**这就是「新增的检查必须先证明它会红」拦下来的那一次**。
  if git diff "$BASE" -- "$f" | grep -qE '^\+#{2,4} .*—— 已定|^\+#{2,4} 已定[（(]'; then
    settled_files+=("$f")
  fi
done < <(git -c core.quotepath=false diff --name-only "$BASE" -- "$DEC" 2>/dev/null)

if ((${#settled_files[@]} == 0)); then
  echo "  ✓ 本次 diff 没有新增「已定」小节，本阶段无对象可判"
  exit 0
fi

flagged=0

# ── ① 同文件里还开着、而本次一个都没碰的未定项 ──────────────────
for f in "${settled_files[@]}"; do
  # 本次 diff 在这个文件里碰过的行号（新文件侧）
  touched="$(git diff -U0 "$BASE" -- "$f" \
            | awk 'match($0,/^@@ .* \+([0-9]+)(,([0-9]+))? @@/,m){s=m[1]; n=(m[3]==""?1:m[3]); for(i=0;i<n;i++) print s+i}')"
  while IFS=: read -r ln text; do
    [[ -n "$ln" ]] || continue
    end=$(awk -v s="$ln" 'NR>s && (/^[[:space:]]*[0-9]+\. /||/^### /||/^## /){print NR-1; exit}' "$f")
    [[ -n "$end" ]] || end=$((ln+20))
    # 这个条目块里有没有任何一行在本次 diff 里被碰过
    hit=0
    for t in $touched; do (( t>=ln && t<=end )) && { hit=1; break; }; done
    (( hit )) && continue
    echo "  ✗ $(basename "$f"):$ln 本次新增了「已定」小节，而这条未定项一个字都没动"
    echo "     ⇒ 复核它是不是被这次定案顺带答掉了。原文：${text:0:60}"
    flagged=1
  done < <(awk '
      /^### 未定项[[:space:]]*$/    { inside=1; next }
      /^### 还要回答的[[:space:]]*$/ { inside=1; next }
      /^### /                      { inside=0 }
      /^#### /                     { inside=0 }
      /^## /                       { inside=0 }
      inside && /^[[:space:]]*[0-9]+\. |^\| [0-9]+ \|/ { print NR":"$0 }
    ' "$f")
  # ⚠️ 滤器已撤：已定的分项住「### 已定项」小节，这一节按定义全是未定的。
  # 老版本按行内关键字滤，会把「正文里提到别处已定」的未定分项一起滤掉。
done

# ── ② 索引页里还悬着、而本次一个都没碰的「待议」节 ──────────────
if [[ -f "$IDX" ]]; then
  idx_touched=0
  git -c core.quotepath=false diff --name-only "$BASE" -- "$IDX" 2>/dev/null | grep -q . && idx_touched=1
  while IFS=: read -r ln text; do
    [[ -n "$ln" ]] || continue
    grep -qE '已回收|已收摊|已并入' <<<"$text" && continue
    (( idx_touched )) && continue
    echo "  ✗ $(basename "$IDX"):$ln 本次有决策定案，而这一节「待议」一个字都没动"
    echo "     ⇒ 复核它是不是被这次定案实质回答/否决了。原文：${text:0:60}"
    flagged=1
  done < <(grep -nE '^## 待议' "$IDX")
fi

if ((flagged)); then
  echo "     → 怎么办：被顺带定掉的就改写成「已定，权威记录在 XX」或直接收摊；"
  echo "               仍然开着的就在条目里补一行「YYYY-MM-DD 复核过，仍然开着，因为 …」。"
  echo "               后一种做法本身就是这条检查要的东西——它要的是一次回头看，不是一次沉默。"
  exit 1
fi
echo "  ✓ 本次定案之后，同文件的未定项与索引页的待议节都被回头看过"
