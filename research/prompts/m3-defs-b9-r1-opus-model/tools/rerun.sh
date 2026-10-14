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

# ── 这一腿要跑的步骤：m3-defs-b9-r1 攻方腿（Opus） ──
# 三份冻结副本：这一轮 $FROZEN（after-b9-fixup），M 形态之前那一版 R2_FROZEN、N 形态之前那一版 R3_FROZEN（默认取 $FROZEN 同级的 m3-defs-b6-r2/repo、m3-defs-b6-r3/repo）。
# 每份先硬链接拷进 $WORK/copies（钩子与判法只读；要改的文件各脚本先换成新 inode），攻方试改那一份由 make_fix_tree.sh 打 $MODEL/patches/ 的补丁。
# 输出写进 $WORK/out，路径换成 <WORK>、<MODEL>、<FROZEN> 之后与 $MODEL/out/ 同名文件逐字比，末尾逐份打 same / DIFF / 新出。
# RERUN_PARTS 可选 z1 z2 z4 z5 z7 的子集（默认全跑；z2 真树一趟单独约 2 分钟）。不起 cargo、不跑 --selftest、不跑门禁阶段。
rerun_steps() {
  local r2="${R2_FROZEN:-$(dirname "$FROZEN")/../m3-defs-b6-r2/repo}" r3="${R3_FROZEN:-$(dirname "$FROZEN")/../m3-defs-b6-r3/repo}"
  local parts="${RERUN_PARTS:-z1 z2 z4 z5 z7}" tools="$MODEL/tools" C="$WORK/copies" O="$WORK/raw"
  export PYTHONDONTWRITEBYTECODE=1
  mkdir -p "$C" "$O" "$WORK/out"
  cp -al "$FROZEN" "$C/b9"; cp -al "$(cd "$r3" && pwd)" "$C/r3"; cp -al "$(cd "$r2" && pwd)" "$C/r2"
  bash "$tools/make_fix_tree.sh" "$FROZEN" "$MODEL" "$C/fix"
  case " $parts " in *" z1 "*)
    nice -n 19 python3 -B "$tools/z1_fence_feed.py" --root b9="$C/b9" --root r3="$C/r3" --root r2="$C/r2" --work "$WORK/z1" > "$O/z1-fence.txt" 2>&1
    nice -n 19 python3 -B "$tools/z1_commonmark_view.py" "$WORK/z1" > "$O/z1-commonmark.txt" 2>&1
    nice -n 19 python3 -B "$tools/z1_stdin_feed.py" --root b9="$C/b9" --root r3="$C/r3" --root r2="$C/r2" --prompts "$WORK/z1" --work "$WORK/z1-stdin" \
      T1-tilde-lone-fence-16 T2-list-fence-container-end-16 O2-nested-list-fence-example > "$O/z1-stdin.txt" 2>&1
    nice -n 19 python3 -B "$tools/z1_fence_feed.py" --root fix="$C/fix" --root b9="$C/b9" --work "$WORK/z1fix" > "$O/z1-fix.txt" 2>&1 ;;
  esac
  case " $parts " in *" z2 "*)
    local label
    for label in b9 r3 r2; do nice -n 19 python3 -B "$tools/z2_htd_cell.py" "$C/$label" "$MODEL/feeds/z2/sample" > "$O/z2-shell-$label.txt" 2>&1 || true; done
    HTD_ANCHOR_DUMP=1 nice -n 19 python3 -B "$tools/z2_htd_cell.py" "$C/b9" "$C/b9" > "$O/z2-anchors-b9.txt" 2>&1 || true
    { bash "$tools/z2_real_tree.sh" b9="$C/b9" "$WORK/z2-real"; bash "$tools/z2_real_tree.sh" r2="$C/r2" "$WORK/z2-real"; } > "$O/z2-real.txt" 2>&1 ;;
  esac
  case " $parts " in *" z4 "*)
    { bash "$tools/z4_source_drift.sh" b9="$C/b9" "$C/b9" "$WORK/z4" excerpt
      bash "$tools/z4_source_drift.sh" r3="$C/r3" "$C/b9" "$WORK/z4" line
      bash "$tools/z4_source_drift.sh" r3="$C/r3" "$C/b9" "$WORK/z4" excerpt
      bash "$tools/z4_source_drift.sh" fix="$C/fix" "$C/b9" "$WORK/z4" excerpt; } > "$O/z4-drift.txt" 2>&1
    { bash "$tools/z4_real_dispatch.sh" b9="$C/b9" "$WORK/z4-real"; bash "$tools/z4_real_dispatch.sh" r3="$C/r3" "$WORK/z4-real"
      bash "$tools/z4_real_dispatch.sh" fix="$C/fix" "$WORK/z4-real"; } > "$O/z4-real.txt" 2>&1
    bash "$tools/z4_misc.sh" "$WORK/z4-misc" b9="$C/b9" r3="$C/r3" r2="$C/r2" fix="$C/fix" > "$O/z4-misc.txt" 2>&1 ;;
  esac
  case " $parts " in *" z5 "*)
    mkdir -p "$WORK/z5-sample"; ( cd "$WORK/z5-sample" && bash "$MODEL/feeds/z5/setup.sh" )
    for label in b9 r3 r2 fix; do nice -n 19 python3 -B "$tools/z5_cells.py" "$C/$label" "$WORK/z5-sample" clock-times date-once term-renames private-values > "$O/z5-cells-$label.txt" 2>&1; done ;;
  esac
  case " $parts " in *" z7 "*)
    { bash "$tools/z7_exempt_repin.sh" b9="$C/b9" "$WORK/z7"; bash "$tools/z7_exempt_repin.sh" r2="$C/r2" "$WORK/z7"; } > "$O/z7.txt" 2>&1 ;;
  esac
  local file name
  for file in "$O"/*.txt; do
    name="$(basename "$file")"
    sed -e "s#$WORK#<WORK>#g" -e "s#$MODEL#<MODEL>#g" -e "s#$FROZEN#<FROZEN>#g" "$file" > "$WORK/out/$name"
    if [[ ! -f "$MODEL/out/$name" ]]; then echo "新出 $name"
    elif cmp -s "$WORK/out/$name" "$MODEL/out/$name"; then echo "same $name"
    else echo "DIFF $name"; fi
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
