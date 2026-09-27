#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 三方论证材料有没有「实现今天的样子」
#
# 管的是 `.claude/rules/implementation-first.md` 第 3 条：实现已经在 `crates/` 里（里程碑一 2026-09-14 出口），
# 三方论证的背景材料必须有一行前提写明读过的 `crates/` 路径与看到的事实；没有相关实现时写 grep 命令与零命中。
# 判据只看形式：标题日期在 CUTOFF 及以后的正文（`research/prompts/_*-body.md`）里必须出现 `crates/`。
# 日期取首行（标题）里的第一个 YYYY-MM-DD；标题里没有日期的，证不出它早于 CUTOFF，照要查的算。
# 成功行报查了几份，并逐个列出因为标题日期早于 CUTOFF 而没查的（现算）。
# CUTOFF 取规范落地的次日：2026-09-16 当天那几轮有的写在用户指示之前，那批证据不回改（evidence-discipline「原样保存的证据不许事后改」）。
#
# 另两条，只管标题日期在 SIDE_AND_SNAPSHOT_CUTOFF 及以后的正文（那天起材料员与主 agent 的定义才要求它们，更早的证据不回改）：
#   ② 正文里有一行「本地腿：攻 …」或「本地腿：辩 …」，冒号后面在攻 / 辩之外还写了理由（这一轮本地腿派哪一侧、为什么）；
#   ③ 这一轮有背景材料（`_<轮>-background.md`）的，要有开工快照目录 `<轮>-snapshot/`，小节清单（`_<轮>-checklist.md`）里每个「### 小节清单：`<文件>`」
#      点名的 `.claude/kb/` 文件在快照目录里某一份 sha256 清单里有一行。
# gate-similar: 72-agent-def-adversarial-review.sh 也读 research/prompts/ 下的三方文件，但它判判决里有没有按路径点名改过的定义；这里判正文与材料的形式
# gate-similar: 56-crates-adversarial-review.sh 同上，判判决点名改过的 crates 源文件
#
# ⚠️ 它管不到读没读对、改法是不是真按实现写的——那一半靠攻方腿与人。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-.}"
CUTOFF="2026-09-17"
SIDE_AND_SNAPSHOT_CUTOFF="2026-09-28"
DIR="$ROOT/research/prompts"
shopt -s nullglob
bodies=("$DIR"/_*-body.md)
checked=0; missing=(); before_cutoff=(); undated=()
side_checked=0; missing_side=(); missing_snapshot=(); failed=0
for body in "${bodies[@]}"; do
  heading="$(head -n 1 "$body")"
  date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"
  # 标题里没有日期：证不出它早于 CUTOFF，照要查的算，不静默跳过
  if [[ -z "$date" ]]; then
    undated+=("${body#"$ROOT"/}")
    date_label="标题没有日期，按要查的算"
  elif [[ "$date" < "$CUTOFF" ]]; then
    before_cutoff+=("${body#"$ROOT"/}（$date）")
    continue
  else
    date_label="$date"
  fi
  checked=$((checked+1))
  grep -qF 'crates/' "$body" || missing+=("${body#"$ROOT"/}（$date_label）")
  if [[ -n "$date" && ! "$date" < "$SIDE_AND_SNAPSHOT_CUTOFF" ]]; then
    side_checked=$((side_checked + 1))
    grep -qE '本地腿[*]*[：:][*]*[[:space:]]*(攻|辩)[^|]{2,}' "$body" || missing_side+=("${body#"$ROOT"/}")
    round="$(basename "$body")"; round="${round#_}"; round="${round%-body.md}"
    if [[ -f "$DIR/_$round-background.md" ]]; then
      if [[ ! -d "$DIR/$round-snapshot" ]]; then
        missing_snapshot+=("$round：没有 research/prompts/$round-snapshot/")
      else
        while IFS= read -r listed; do
          [[ "$listed" == .claude/kb/* ]] || continue
          grep -qsF "$listed" "$DIR/$round-snapshot"/* || missing_snapshot+=("$round：$listed 不在快照清单里")
        done < <(sed -n 's/^### 小节清单：`\([^`]*\)`.*/\1/p' "$DIR/_$round-checklist.md" 2>/dev/null)
      fi
    fi
  fi
done
if ((checked == 0)); then
  echo "  ! 标题日期在 $CUTOFF 及以后的三方论证正文 0 份，本阶段无对象可判"
  exit 77
fi
if ((${#missing[@]})); then
  echo "  ✗ 这些三方论证正文没有「实现今天的样子」——全文一处 crates/ 都没有："   # gate-lint:summary
  printf '     %s\n' "${missing[@]}"
  echo "     → 怎么办：读 crates/ 里与这一问相关的代码路径，在正文前提表里加一行「实现今天的样子」，写文件名与看到的事实并标成观测；"
  echo "               实现里没有相关代码时，写 grep 命令与零命中（.claude/rules/implementation-first.md 第 2、3 条）。"
  failed=1
fi
if ((${#missing_side[@]})) || ((${#missing_snapshot[@]})); then
  if ((${#missing_side[@]})); then
    echo "  ✗ 这些三方论证正文（标题日期 ≥ $SIDE_AND_SNAPSHOT_CUTOFF）没写这一轮本地腿派哪一侧、为什么："   # gate-lint:summary
    printf '     %s\n' "${missing_side[@]}"   # gate-lint:detail
    echo "     → 怎么办：正文分工表里加一行「本地腿：攻（理由）」或「本地腿：辩（理由）」——辩方复核轮派本地辩方，缺哪一侧派哪一侧（.claude/main-agent.md「什么时候派哪个 agent」）"
  fi
  if ((${#missing_snapshot[@]})); then
    echo "  ✗ 这些轮次的开工快照缺了（有背景材料就要有 <轮>-snapshot/，清单点名的 kb 文件都要在快照清单里）："   # gate-lint:summary
    printf '     %s\n' "${missing_snapshot[@]}"   # gate-lint:detail
    echo "     → 怎么办：照 .claude/agents/three-way-materials.md 第 5b 步补 research/prompts/<轮>-snapshot/kb-sha256.txt（sha256sum 原样输出，路径从仓根起）；腿已经开跑的，快照写明补写的时刻"
  fi
  failed=1
fi
((failed)) && exit 1
echo "  ✓ 三方论证正文都写了实现今天的样子（检查了 $checked 份：标题日期 ≥ $CUTOFF 的 $((checked - ${#undated[@]})) 份、标题没有日期的 ${#undated[@]} 份）"
if ((${#undated[@]})); then
  echo "    标题没有日期、照要查的算的 ${#undated[@]} 份：${undated[*]}"
fi
echo "    标题日期 ≥ $SIDE_AND_SNAPSHOT_CUTOFF、另核了本地腿一侧与开工快照的 $side_checked 份"
if ((${#before_cutoff[@]})); then
  echo "    没查 ${#before_cutoff[@]} 份（标题日期早于 $CUTOFF，规范落地之前写的）：${before_cutoff[*]}"
else
  echo "    没查 0 份"
fi
