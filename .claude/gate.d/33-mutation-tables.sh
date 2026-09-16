#!/usr/bin/env bash
# gate-stage: 每个实验二进制都要有变异表
#
# 判据：`research/e7-index-bench/src/bin/` 下每个 `*.rs` 在 `research/mutations/`
# 下都要有**同名** `.tsv` 变异表，且表里至少一条成形的变异（三段制表符分隔）。
#
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现（C40（变异表没存）——
# 结论留下了而产生结论的装置没留下，正是 2026-08-29 审计实测过的失败形态；
# 2026-09-03 把最后 19 张欠表补齐后，用这条阶段拦住它再欠回去）。
#
# ⚠️ **这条阶段不跑变异**（跑一遍全部表要逐条重编译，量级是小时）。
# 「表今天还会不会红」由每轮改动实验代码时手跑 `research/scripts/mutate.sh` 证明，
# 复跑记录见各实验正文的「口径与复跑」——本阶段只保证装置在、形状对，不冒充跑过。
set -uo pipefail
BIN_DIR=research/e7-index-bench/src/bin
MUT_DIR=research/mutations
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，本阶段跳过"; exit 77; }

missing=(); malformed=()
for src in "$BIN_DIR"/*.rs; do
  stem="$(basename "$src" .rs)"
  tsv="$MUT_DIR/$stem.tsv"
  if [[ ! -f "$tsv" ]]; then missing+=("$stem"); continue; fi
  # 至少一条成形的变异行：非注释、非空、恰好三段
  ok_rows=$(awk -F'\t' '!/^#/ && NF==3 && $1!="" && $2!="" {n++} END{print n+0}' "$tsv")
  bad_rows=$(awk -F'\t' '!/^#/ && NF!=3 && $0!="" {n++} END{print n+0}' "$tsv")
  if [[ "$ok_rows" -eq 0 || "$bad_rows" -gt 0 ]]; then malformed+=("$stem(成形 $ok_rows 条/坏 $bad_rows 行)"); fi
done

# ── 锚点还对得上吗（C327（变异表的锚点腐化没有会红的检查））──
# 成形不等于替换得上：`mutate.sh` 要求每条「原文」在对应源码里**恰好命中一次**，
# 命中 0 次或多次就退出码 3、**后面的条目一条都不跑**，而那张表对上面那几项检查是绿的。
# 2026-09-14 现查有 4 条这样的锚点（e143 两条、e67 一条、e79 一条），而它们所在的实验页
# 都写着「N 条变异全抓」——那句话当时已经复跑不出来了。
# ⚠️ 这里仍然**不跑变异**，只做子串计数，代价是毫秒级。
anchor_report="$(BIN_DIR="$BIN_DIR" MUT_DIR="$MUT_DIR" python3 - <<'PY'
import os, glob
bin_dir = os.environ["BIN_DIR"]; mut_dir = os.environ["MUT_DIR"]
bad = []; checked = 0
for tsv in sorted(glob.glob(os.path.join(mut_dir, "*.tsv"))):
    stem = os.path.basename(tsv)[:-4]
    src_path = os.path.join(bin_dir, stem + ".rs")
    # 没有同名二进制的表（shell 探针的变异表）不在本检查射程：它们的被测对象不是 .rs
    if not os.path.exists(src_path):
        continue
    src = open(src_path, encoding="utf-8").read()
    for lineno, line in enumerate(open(tsv, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        name, frm, _ = parts
        checked += 1
        # 口径与 mutate.sh 一致：表里的 \n 先还原成换行，再数子串
        hits = src.count(frm.replace("\\n", "\n"))
        if hits != 1:
            bad.append((stem, name, lineno, hits))
print("CHECKED", checked)
for b in bad:
    print("BAD", *b, sep="\t")
PY
)"
anchor_checked="$(sed -n 's/^CHECKED //p' <<<"$anchor_report")"
mapfile -t anchor_bad < <(grep '^BAD' <<<"$anchor_report")

if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]})); then
  if ((${#missing[@]})); then
    echo "  ✗ 这些实验二进制没有同名变异表："   # gate-lint:detail
    printf '      %s\n' "${missing[@]}"
  fi
  if ((${#malformed[@]})); then
    echo "  ✗ 这些变异表不成形（要求每行三段制表符分隔，至少一条）："
    printf '      %s\n' "${malformed[@]}"
    echo "    → 每行写成「变异名<TAB>原文<TAB>替换文」三段，坏行补齐或删掉，至少留一条成形的。"
  fi
  if ((${#anchor_bad[@]})); then
    echo "  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 会退出码 3，后面的条目一条都不跑）："
    while IFS=$'\t' read -r _ stem name lineno hits; do
      printf '      %s 的 %s（表第 %s 行，命中 %s 次）\n' "$stem" "$name" "$lineno" "$hits"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_bad[@]}")
    echo "    命中 0 次：源码改过而表没跟；命中多次：原文要多带一行上下文才唯一。"
  fi
  echo "  → 怎么办：缺表的写 research/mutations/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文）；"
  echo "    锚点对不上的把「原文」改成今天源码里逐字存在、且只出现一次的那一段，"
  echo "    改完跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
  exit 1
fi
n=$(ls "$BIN_DIR"/*.rs | wc -l)
echo "  ✓ $n 个实验二进制都有成形的变异表，${anchor_checked} 条变异的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上）"
