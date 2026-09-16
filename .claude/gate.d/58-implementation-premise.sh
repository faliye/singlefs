#!/usr/bin/env bash
# gate-stage: 三方论证材料有没有「实现今天的样子」
#
# 管的是 `.claude/rules/implementation-first.md` 第 3 条：实现已经在 `crates/` 里（里程碑一 2026-09-14 出口），
# 三方论证的背景材料必须有一行前提写明读过的 `crates/` 路径与看到的事实；没有相关实现时写 grep 命令与零命中。
# 判据只看形式：标题日期在 CUTOFF 及以后的正文（`research/prompts/_*-body.md`）里必须出现 `crates/`。
# CUTOFF 取规范落地的次日：2026-09-16 当天那几轮有的写在用户指示之前，那批证据不回改（evidence-discipline「原样保存的证据不许事后改」）。
#
# ⚠️ 它管不到读没读对、改法是不是真按实现写的——那一半靠攻方腿与人。
set -uo pipefail
ROOT="${1:-.}"
CUTOFF="2026-09-17"
DIR="$ROOT/research/prompts"
shopt -s nullglob
bodies=("$DIR"/_*-body.md)
checked=0; missing=()
for body in "${bodies[@]}"; do
  heading="$(head -n 1 "$body")"
  date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"
  [[ -n "$date" ]] || continue
  [[ "$date" < "$CUTOFF" ]] && continue
  checked=$((checked+1))
  grep -qF 'crates/' "$body" || missing+=("${body#"$ROOT"/}（$date）")
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
  exit 1
fi
echo "  ✓ 三方论证正文都写了实现今天的样子（检查了 $checked 份，标题日期 ≥ $CUTOFF）"
