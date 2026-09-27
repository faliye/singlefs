#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 状态一致性：定了新东西之后有没有回头看同文件的未定项
#
# 还 checks-owed.md C36（未定项检查的三个盲区）的前两条。
#
# **它与 60 阶段是两把不同的尺子，不是加强版**：
#   60：**跨文件 + 看历史**——未定项点名了别的决策，而那个决策后来定过东西。
#   61：**同文件 + 看本次 diff**——本次 diff 在某个决策里新增了一个「已定」小节，
#       而同一个文件里还开着的未定项**这次一个都没碰**。
#
# ⚠️ **为什么必须看 diff 而不是看历史**：按历史判会在每个「有已定小节又有未定项」的
# 决策上恒红，而 D13（验证路线） 已定项 9（.claude/kb/decisions/13-验证路线.md）写过
# 「一个不可复现的夜间失败价值接近 0，甚至为负，它训练人忽略红灯」——一个恒红的检查比没有检查更坏。
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
# 「碰没碰」按条目块判：表格行一行就是一条分项，列表式条目从首行到下一条分项或下一个标题之前（与 lib-open-item-review.py 同一个口径）；
# 索引页的「待议」按那一节判（从 `## 待议` 到下一个 `## ` 之前），整份 decisions.md 别处动过不算碰了它。
# 这份文件跟踪没有，按 `git ls-files --error-unmatch` 的退出码判：1 是未跟踪（整份算碰过），别的非 0 是 git 自己出错，判红。
# 新增了已定小节的那几份里一条未定项都没有、索引页也没有待议节时，什么都没查，退 77。
# 样本：fixtures/61-settled-same-file.sh/red 另放一份两行表格的决策（只改了第 2 行，第 1 行要报）与一节没动的待议（同一次只改了索引表那一行）。
set -uo pipefail
DEC=.claude/kb/decisions
IDX=.claude/kb/decisions.md
# 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
[[ -d "$DEC" ]] || { echo "  ! 没有 $DEC，本阶段无对象可判"; exit 77; }
git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，本阶段跳过"; exit 77; }

# 改动范围取共用脚本 research/scripts/changed-paths.sh（门禁 64 号判阶段里不另算一份）：
# 基准是 gate_diff_base gate，名单带未跟踪文件——新写的决策文件在 git add 之前也算这次改动。
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
BASE="$(gate_diff_base gate)"

# 本次改动碰过的决策文件。名单先落到变量、判过退出码再读：git 失败时名单是空的，会被读成「本次没有新增已定小节」。
all_changed="$(gate_changed_paths "$BASE" untracked)" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $BASE），这次改了哪些决策文件没取到"
  echo "     → 按上面 git 的报错修好仓库状态（基准 $BASE 要存在；仓里还没有提交就先提交一次）再跑；git 失败时这一阶段什么都没比，不是通过。"
  exit 1
}
changed_names="$(grep "^\.claude/kb/decisions/" <<<"$all_changed" || true)"

# 本次 diff 里**新增**了「已定」小节标题的决策文件
settled_files=()
while IFS= read -r f; do
  [[ -n "$f" ]] || continue
  # 只认新增行（+），且必须是小节标题：`## D1 简称 —— 已定` / `### 未定项 3 —— 已定` / `### 已定（…）`
  # ⚠️ **这个正则第一版写坏过，而且是「写坏了但恒绿」那种坏法**：
  # 写成 '^\+#{2,4} .*(—— 已定|^\+### 已定)' 时，第二个分支里的 ^ 在组内永远匹配不上，
  # 于是 `### 已定（…）` 这种最常见的定案小节标题一个都抓不到，检查恒绿。
  # 合成仓双向验的时候当场红——**这就是「新增的检查必须先证明它会红」拦下来的那一次**。
  # 同 10-kb-rot.sh 那条：pipefail + `grep -q` 提前退出 ⇒ 前段 SIGPIPE ⇒ 命中被读成没命中。
  # `git diff` 的输出可以很大，这里比那条更容易撞上。
  # 新增行按共用脚本取（未跟踪的文件整份算新增）；取不到就判红，不当「没有新增」
  added_out="$(gate_added_lines "$BASE" "$f")" || {
    echo "  ✗ 取不到 $f 这次新增了哪些行（基准 $BASE）"
    echo "     → 按上面 git 的报错修好仓库状态再跑；取不到新增行时这一阶段什么都没比，不是通过。"
    exit 1
  }
  added_text="$(cut -f2- <<<"$added_out")"
  if grep -qE '^#{2,4} .*—— 已定|^#{2,4} 已定[（(]' <<<"$added_text"; then
    settled_files+=("$f")
  fi
done <<<"$changed_names"

if ((${#settled_files[@]} == 0)); then
  # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  echo "  ! 本次 diff 没有新增「已定」小节，本阶段无对象可判"
  exit 77
fi

# 这次改动碰过的行号（新文件侧），一行一个。未跟踪的新文件整份都是这次写的，每一行都算碰过。
# 跟踪没有按 ls-files --error-unmatch 的退出码判：0 跟踪、1 未跟踪，别的码是 git 自己出错——返回 2，调用方判红，
# 不当「未跟踪」（那样整份每行都算碰过，这一份一条都不报）。
touched_lines_of() {
  local file="$1" tracked_rc=0
  git -c core.quotepath=false ls-files --error-unmatch -- "$file" >/dev/null 2>&1 || tracked_rc=$?
  case "$tracked_rc" in
    0) git diff --no-color --no-ext-diff -U0 "$BASE" -- "$file" \
         | awk 'match($0,/^@@ .* \+([0-9]+)(,([0-9]+))? @@/,m){s=m[1]; n=(m[3]==""?1:m[3]); for(i=0;i<n;i++) print s+i}' ;;
    1) seq 1 "$(wc -l < "$file")" ;;
    *)
      echo "  判不了 $file 跟踪没有：git 退出码 $tracked_rc" >&2
      return 2 ;;
  esac
}

flagged=0
open_checked=0      # 查过的未定项条数（新增了已定小节的那几份里）
pending_checked=0   # 查过的索引页「待议」节数

# ── ① 同文件里还开着、而本次一个都没碰的未定项 ──────────────────
for f in "${settled_files[@]}"; do
  # 本次 diff 在这个文件里碰过的行号（新文件侧）；未跟踪的新文件整份都算碰过
  touched="$(touched_lines_of "$f")" || {
    echo "  ✗ 判不了 $f 有没有被 git 跟踪（上面是 git 的退出码），这一份的未定项这一轮没比"
    echo "     → 怎么办：按 git 的报错修好仓库状态（索引坏了就 git status 看、必要时重建索引）再跑；判不了不是「未跟踪、整份都碰过」。"
    exit 1
  }
  while IFS=: read -r ln text; do
    [[ -n "$ln" ]] || continue
    open_checked=$((open_checked + 1))
    # 条目块：表格行一行就是一条，列表项到下一条分项（编号列表项或「| k |」表格行）或任一级标题之前（与 lib-open-item-review.py 同一个口径）
    end=$(awk -v s="$ln" 'NR>s && (/^[[:space:]]*[0-9]+\. /||/^\|[[:space:]]*[0-9]+[[:space:]]*\|/||/^##+ /){print NR-1; exit}' "$f")
    [[ -n "$end" ]] || end=$((ln+20))
    # 这个条目块里有没有任何一行在本次 diff 里被碰过
    hit=0
    for t in $touched; do (( t>=ln && t<=end )) && { hit=1; break; }; done
    (( hit )) && continue
    echo "  ✗ $(basename "$f"):$ln 本次新增了「已定」小节，而这条未定项一个字都没动"   # gate-lint:detail
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
  # 按这一节判「动没动」：索引页别处（索引表的一行）改了不算回头看过这一节待议
  idx_touched_lines=""
  if grep -qxF "$IDX" <<<"$all_changed"; then
    idx_touched_lines="$(touched_lines_of "$IDX")" || {
      echo "  ✗ 判不了 $IDX 有没有被 git 跟踪（上面是 git 的退出码），索引页的待议节这一轮没比"
      echo "     → 怎么办：按 git 的报错修好仓库状态再跑；判不了不是「整份都碰过」。"
      exit 1
    }
  fi
  while IFS=: read -r ln text; do
    [[ -n "$ln" ]] || continue
    grep -qE '已回收|已收摊|已并入' <<<"$text" && continue
    pending_checked=$((pending_checked + 1))
    section_end=$(awk -v s="$ln" 'NR>s && /^## /{print NR-1; exit}' "$IDX")
    [[ -n "$section_end" ]] || section_end=$(wc -l < "$IDX")
    section_touched=0
    for t in $idx_touched_lines; do (( t>=ln && t<=section_end )) && { section_touched=1; break; }; done
    (( section_touched )) && continue
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
if ((open_checked == 0 && pending_checked == 0)); then
  echo "  ! 本次无对象可判：新增了已定小节的 ${#settled_files[@]} 份决策里没有一条未定项，索引页也没有悬着的待议节，什么都没查"
  exit 77
fi
echo "  ✓ 本次定案之后，同文件的未定项与索引页的待议节都被回头看过（新增已定小节的决策 ${#settled_files[@]} 份，查了 $open_checked 条未定项、$pending_checked 节待议）"
