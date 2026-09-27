#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 第一个事务的每个字段都要指到一条真实存在的分项
#
# **判据**：`kb/layout/01-first-txn.md` 每一行的「指向」列，凡是写了
# 「D<n>（简称） 已定项 <k>」或「…… 未定项 <k>」的（简称可以不写，也可以套一层括号），那条分项必须在
# `kb/decisions/` 那条决策「### 已定项」/「### 未定项」的索引里（第一个 `####` 之前）真实存在；指不到就判红。
#
# **为什么要有它**：那张表是 C66（格式级空白没有分项号）欠的东西，
# 它的全部价值在于「每个字段都指得到一条分项」这个性质**持续成立**。
# 表是手写的，而分项会改号、会从未定项挪进已定项、会被合并——
# 一旦某一行指向的分项不存在了，那一行就退化成一句没人管的话，
# 而读表的人不会发现：他看到的仍然是一个格式正确的编号。
#
# ⚠️ **它和「分项引用的状态与正文相符」（`22-item-ref-status.sh`）不是一条。**
# 那个阶段判的是「说已定的是不是真已定」，前提是那条分项存在；
# 本阶段判的是**分项存不存在**。一个指向 `D22 已定项 99` 的字段，
# 在 22 阶段眼里没有对象可比，会安静地通过。
#
# ⚠️ **写了「无分项」的行是合法的**，那正是这张表要暴露的格式级空白，不判红。

set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-.}"
KB="$ROOT/.claude/kb"
TABLE="$KB/layout/01-first-txn.md"

# 无对象可判退 77，门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
[[ -f "$TABLE" ]] || { echo "  ! 没有 $TABLE，本阶段无对象可判"; exit 77; }
[[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，本阶段无对象可判"; exit 77; }

bad=0
checked=0

# 抽出表里每一处「D<n>（…） 已定项 <k>」/「未定项 <k>」引用，连同行号
while IFS=$'\t' read -r lineno dnum kind item; do
  [[ -z "${dnum:-}" ]] && continue
  checked=$((checked + 1))

  # 找到那条决策的正文文件：文件名以两位编号打头
  file="$(ls "$KB/decisions/" 2>/dev/null | grep -E "^0*${dnum}-" | head -1)"
  if [[ -z "$file" ]]; then
    echo "  ✗ $TABLE:$lineno 指向 D$dnum，而 decisions/ 里没有这条决策"   # gate-lint:detail
    bad=$((bad + 1))
    continue
  fi

  # 那条分项要在对应小节的索引表里有一行，形如 `| <k> | …`
  # 已定项与未定项各有一张表，取「### 已定项」/「### 未定项」之后到下一个任意级标题之前：
  # 只看索引，`#### 已定项 N` 之下的论证里另有编号列表、「## 历史版本」里另有表格，那些不是分项
  # （与 20-kb-shape 第 5 段、24 号同一条边界）
  section="$(awk -v want="$kind" '
    /^### 已定项[[:space:]]*$/ { cur="已定项"; next }
    /^### 未定项[[:space:]]*$/ { cur="未定项"; next }
    /^#+ / { cur="" }
    cur == want { print }
  ' "$KB/decisions/$file")"

  # 分项行有两种写法：表格行 `| k | …`（D18 / D22 一类），
  # 或编号列表 `k. **…**`（D16 / D19 / D23 一类）。两种都认。
  if ! grep -qE "^(\| *${item} *\||${item}\. )" <<<"$section"; then
    echo "  ✗ $TABLE:$lineno 指向 D$dnum（$file）的「$kind $item」，而那张表里没有这一条"   # gate-lint:detail
    bad=$((bad + 1))
  fi
done < <(
  grep -n '指向\|已定项\|未定项' "$TABLE" 2>/dev/null \
    | grep -oE '^[0-9]+:.*' \
    | while IFS= read -r line; do
        n="${line%%:*}"
        body="${line#*:}"
        # 一行里可能有多处引用，逐个抠出来：简称可以不写（「D22 已定项 99」），也可以里面再套一层括号
        # （「D14（双轨（大小文件 / 持久临时）） 已定项 9」）；编号左边不许紧挨字母数字
        grep -oE '(^|[^A-Za-z0-9])D[0-9]+(（([^（）]|（[^（）]*）)*）)? *(已定项|未定项) *[0-9]+' <<<"$body" \
          | grep -oE 'D[0-9]+.*' \
          | while IFS= read -r ref; do
              d="$(grep -oE '^D[0-9]+' <<<"$ref" | tr -d 'D')"
              k="$(grep -oE '(已定项|未定项)' <<<"$ref" | head -1)"
              i="$(grep -oE '[0-9]+$' <<<"$ref")"
              printf '%s\t%s\t%s\t%s\n' "$n" "$d" "$k" "$i"
            done
      done
)

if ((bad)); then
  echo "  ✗ 第一个事务的字段表里有 $bad 处指向了不存在的分项"
  echo "     → 怎么办： 每一行的「指向」列要么写一条真实存在的分项（编号改过就跟着改），"
  echo "                要么写「无分项」——后者是这张表要暴露的格式级空白，不判红。"
  echo "                分项现在叫什么，看 .claude/kb/decisions/ 里那条决策的两张索引表。"
  exit 1
fi

if ((checked == 0)); then
  echo "  ! $TABLE 里一处「D<n>（简称） 已定项 / 未定项 k」引用都没有，本阶段无对象可判"
  exit 77
fi
echo "  ✓ 第一个事务的字段表指向都成立（查了 $checked 处引用）"
