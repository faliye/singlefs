#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: research 里的实验号在 kb 里有没有正文
#
# 判据：`research/` 下（逐层往下扫，`target/` 编译目录不扫）凡是以 `eNN` 命名的东西（提示、产物、源码、变异表、数据），
# 以及住在 `crates/singlefs-harness/src/bin/` 下的实验装置（`eNN_*.rs`），
# `kb/experiments/` 里就必须有对应编号的正文文件。没有 = 干了活但没入库，
# 而**跑过的东西没入库比没跑更危险**——它会以「我们量过」的形式活在对话里，谁也复核不了。
#
# ⚠️ **这条是实测出来的**：2026-08-29 复跑轮现查，`research/prompts/e34-rootring-geometry-local.md`
# 与 `research/results/e34-rootring-local.out` 都在，而 kb 里没有 E34。
# 已有的阶段 `40-results-cited.sh` 抓不到它，因为那个阶段把文件名含 `local` 的一律当本地腿问答排除掉了
# ——**排除规则正好盖住了这一个**。
set -uo pipefail
EXP_DIR=.claude/kb/experiments
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
[[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，本阶段跳过"; exit 77; }

have=$(ls "$EXP_DIR" | grep -oE '^[0-9]+' | sed 's/^0*//' | sort -un)
# 射程逐个目录报：不在的列进成功行，读不了的（find 出错）判红——射程静默缩小与判过了在输出里长得一样
SCOPE_DIRS=(research crates/singlefs-harness/src/bin)
absent=(); names_file="$(mktemp)"; errors_file="$(mktemp)"
trap 'rm -f "$names_file" "$errors_file"' EXIT
for d in "${SCOPE_DIRS[@]}"; do
  if [[ ! -d "$d" ]]; then absent+=("$d"); continue; fi
  if ! find "$d" -name target -prune -o -regextype posix-extended -regex '.*/e[0-9]+[^/]*' -printf '%f\n' >>"$names_file" 2>>"$errors_file"; then
    echo "  ✗ 扫 $d 时 find 出错，这个目录里的实验号没扫全："
    sed 's/^/      /' "$errors_file"
    echo "  → 怎么办：按上面的报错修好（读不了的子目录、权限），再跑；扫不全的目录会让孤儿实验号静默漏掉。"
    exit 1
  fi
done
want=$(grep -ohE '^e[0-9]+' "$names_file" | sed 's/^e//' | sort -un)

orphan=()
for n in $want; do grep -qx "$n" <<<"$have" || orphan+=("E$n"); done

if ((${#orphan[@]})); then
  echo "  ✗ research 里有这些实验号的东西，kb/experiments/ 里却没有正文："
  for e in "${orphan[@]}"; do
    printf '      %-6s' "$e"
    find "${SCOPE_DIRS[@]}" -name target -prune -o \( -name "${e,,}[-_]*" -o -name "${e,,}.*" \) -print 2>/dev/null | head -5 | tr '\n' ' '
    echo
  done
  echo "  → 怎么办：给它建正文（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；"
  echo "    若那轮的结论不打算入库，就把 research 下那些文件删掉——留着等于留一份没人能复核的证据。"
  exit 1
fi
if [[ -z "$want" ]]; then
  echo "  ⊘ 本次无对象可判：${SCOPE_DIRS[*]} 下没有一个以 eNN 命名的东西（不在的目录：${absent[*]:-（没有）}）"
  exit 77
fi
echo "  ✓ research 里的实验号在 kb 里都有正文（$(wc -w <<<"$want") 个）"
echo "    扫了 ${SCOPE_DIRS[*]}（target/ 不扫）；不在的目录 ${#absent[@]} 个：${absent[*]:-（没有）}"
