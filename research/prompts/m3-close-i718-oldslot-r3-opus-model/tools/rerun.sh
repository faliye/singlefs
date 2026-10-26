#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  # 拷冻结树里要编的那几样，walk.rs 依次装四道开关（第一轮两道、第二轮一道、第三轮一道），探针包放进副本的工作区，编 release。
  local tree="$WORK/tree" target="$WORK/target" probe piece item slots mode name
  mkdir -p "$tree/.claude" "$WORK/logs" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$tree/"
  cp "$FROZEN/.claude/heavy-thread-default" "$tree/.claude/"
  for switch in walk-candidate-switch.py walk-candidate-switch-ding.py walk-candidate-switch-ding-round-two.py walk-candidate-switch-round-three.py; do
    python3 "$MODEL/tools/$switch" "$tree/crates/singlefs-checker/src/walk.rs"
  done
  mkdir -p "$tree/crates/i718-round-three-probe"
  cp -r "$MODEL/probe/Cargo.toml" "$MODEL/probe/src" "$tree/crates/i718-round-three-probe/"
  python3 - "$tree/Cargo.toml" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
anchor = '    "crates/singlefs-unit-graph",\n'
assert text.count(anchor) == 1
open(path, "w", encoding="utf-8").write(text.replace(anchor, anchor + '    "crates/i718-round-three-probe",\n'))
PY
  (cd "$tree" && BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include CARGO_TARGET_DIR="$target" \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo build --offline --release -p i718-round-three-probe) > "$WORK/logs/build.log" 2>&1
  probe="$target/release/i718-round-three-probe"
  # 各件单线程并行：「S 模式」，输出一件一份；退出码各写一份，最后一个不带参数的 wait 等齐。
  local -a items=("4 z1" "4 z2-gen0" "4 z2-holes" "4 z2-floor" "4 z2-alternating" "4 z2-p" "4 z3-b" "4 z3-f" "4 z3-g" "4 z3-br" "4 counts"
    "5 z2-gen0" "8 z2-gen0" "5 z3-g" "8 z3-g" "16 z3-g" "5 z3-br" "8 z3-br" "16 z3-br")
  rm -f "$WORK"/b1-*.rc
  piece=0
  for item in "${items[@]}"; do
    slots=${item%% *}; mode=${item#* }
    { I718_SLOTS_PER_REGION=$slots nice -n 19 "$probe" "$mode" > "$WORK/out/$mode-S$slots.txt" 2> "$WORK/logs/$mode-S$slots.err"; echo "$?" > "$WORK/b1-$piece.rc"; } &
    piece=$((piece + 1))
  done
  wait
  [[ "$(ls "$WORK"/b1-*.rc | wc -l)" -eq "$piece" ]] || { echo "退出码文件数对不上（派了 $piece 件）" >&2; return 1; }
  for ((piece = 0; piece < ${#items[@]}; piece++)); do
    [[ "$(cat "$WORK/b1-$piece.rc")" == 0 ]] || { echo "件 $piece（${items[$piece]}）退 $(cat "$WORK/b1-$piece.rc")" >&2; return 1; }
  done
  python3 -I "$MODEL/tools/tally.py" "$WORK"/out/z*.txt > "$WORK/out/tally.txt"
  python3 -I "$MODEL/tools/recount_keys.py" "$REPO_ROOT/research/prompts/m3-close-i718-oldslot-r2-sonnet-model/out/raw" > "$WORK/out/recount-keys-sonnet-r2.txt"
  python3 -I "$MODEL/tools/recount_extra.py" "$REPO_ROOT/research/prompts/m3-close-i718-oldslot-r2-sonnet-model/out/raw" > "$WORK/out/recount-extra-sonnet-r2.txt"
  for name in "${items[@]}"; do
    name="${name#* }-S${name%% *}"
    if diff -q "$MODEL/out/$name.txt" "$WORK/out/$name.txt" > /dev/null; then
      echo "name=$name same_as=out/$name.txt lines=$(wc -l < "$WORK/out/$name.txt")"
    else
      echo "name=$name DIFFERS from out/$name.txt（diff \"$MODEL/out/$name.txt\" \"$WORK/out/$name.txt\"）"
    fi
  done
  python3 -I "$MODEL/tools/p_scan_summary.py" "$WORK/out/z2-p-S4.txt" > "$WORK/out/p-scan-summary.txt"
  # 第二轮判决复跑辩方那一份的输出在主 agent 的草稿目录里；还在才比（不在就跳过、打一行说明）。
  local verdict_rerun="${RERUN_R2_VERDICT_RERUN:-$HOME/.local/share/singlefs/work/m3-close-b16-verify/i718-r2-rerun/sonnet-work/out}"
  if [[ -d "$verdict_rerun" ]]; then
    python3 -I "$MODEL/tools/compare_runs.py" "$REPO_ROOT/research/prompts/m3-close-i718-oldslot-r2-sonnet-model/out/raw" "$verdict_rerun" > "$WORK/out/compare-sonnet-r2-leg-vs-verdict-rerun.txt"
    if diff -q "$MODEL/out/compare-sonnet-r2-leg-vs-verdict-rerun.txt" "$WORK/out/compare-sonnet-r2-leg-vs-verdict-rerun.txt" > /dev/null; then
      echo "name=compare-sonnet-r2-leg-vs-verdict-rerun same_as=out/compare-sonnet-r2-leg-vs-verdict-rerun.txt"
    else
      echo "name=compare-sonnet-r2-leg-vs-verdict-rerun DIFFERS"
    fi
  else
    echo "name=compare-sonnet-r2-leg-vs-verdict-rerun skipped: $verdict_rerun 不在"
  fi
  for name in tally p-scan-summary recount-keys-sonnet-r2 recount-extra-sonnet-r2; do
    if diff -q "$MODEL/out/$name.txt" "$WORK/out/$name.txt" > /dev/null; then
      echo "name=$name same_as=out/$name.txt lines=$(wc -l < "$WORK/out/$name.txt")"
    else
      echo "name=$name DIFFERS from out/$name.txt（diff \"$MODEL/out/$name.txt\" \"$WORK/out/$name.txt\"）"
    fi
  done
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--work 非空都退 2 并说明"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
