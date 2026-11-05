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
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
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
  if [[ -n "${RERUN_TEMPLATE_SELFTEST_MARK:-}" ]]; then   # 模板自证里拷出去起的就是模板自带的那一步
    mkdir -p "$WORK/tree"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
    echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
    return 0
  fi
  # 正推腿 d137-gov-r1：钩子与真 bash 的差分。--frozen 给一棵带 .claude/、scripts/、records/ 的树（主工作区或开工快照树）；
  # 七步各用自己的一份拷贝与沙箱并行跑（每步一个 python 进程，样本进程各在自己的进程组里，超时杀整组），退出码各写进自己的文件。
  local steps=(heavy fuzz opts hits battery g2 garbage overblock) step leg
  mkdir -p "$WORK/out" "$WORK/base"
  rsync -a --exclude kb --exclude worktrees --exclude '.git' "$FROZEN/.claude" "$WORK/base/"
  rsync -a --exclude experiment-apparatus --exclude __pycache__ "$FROZEN/scripts" "$WORK/base/"
  mkdir -p "$WORK/base/scripts/experiment-apparatus" "$WORK/base/records"
  cp "$FROZEN/scripts/experiment-apparatus/replay.sh" "$WORK/base/scripts/experiment-apparatus/"
  rsync -a --include '*/' --include 'Cargo.toml' --exclude '*' "$FROZEN/crates" "$WORK/base/"
  cp "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$WORK/base/"
  cp -r "$FROZEN/.cargo" "$WORK/base/.cargo"
  cp "$FROZEN"/records/2026-10-29-一百三十七节那一轮调度记录.md "$WORK/base/records/"
  echo "base_tree_bytes=$(du -sb "$WORK/base" | cut -f1)"
  for step in "${steps[@]}"; do
    leg="$WORK/$step"
    mkdir -p "$leg/sandbox" "$leg/workroot" "$leg/stubs"
    cp -a "$WORK/base" "$leg/tree"
    bash "$MODEL/tools/setup_sandbox.sh" "$leg"
    local code
    for code in 0 1 2 3 4 5 77 124 250; do printf 'import sys\nsys.exit(%s)\n' "$code" > "$leg/stubs/adm$code.py"; done
    printf 'import time\ntime.sleep(60)\n' > "$leg/stubs/admhang.py"
  done
  rm -f "$WORK"/out/*.rc
  { D137_LEG_DIR="$WORK/heavy"   python3 -W ignore "$MODEL/tools/run_heavy.py"      > "$WORK/out/heavy.tsv"   2> "$WORK/out/heavy.err";   echo "$?" > "$WORK/out/heavy.rc"; } &
  { D137_LEG_DIR="$WORK/fuzz"    python3 -W ignore "$MODEL/tools/run_fuzz.py"       > "$WORK/out/fuzz.tsv"    2> "$WORK/out/fuzz.err";    echo "$?" > "$WORK/out/fuzz.rc"; } &
  { D137_LEG_DIR="$WORK/opts"    python3 -W ignore "$MODEL/tools/opts_fuzz.py"      > "$WORK/out/opts.tsv"    2> "$WORK/out/opts.err";    echo "$?" > "$WORK/out/opts.rc"; } &
  { D137_LEG_DIR="$WORK/hits"    python3 -W ignore "$MODEL/tools/hits_g1.py"        > "$WORK/out/hits.tsv"    2> "$WORK/out/hits.err";    echo "$?" > "$WORK/out/hits.rc"; } &
  { D137_LEG_DIR="$WORK/battery" python3 -W ignore "$MODEL/tools/battery_other.py"  > "$WORK/out/battery.tsv"  2> "$WORK/out/battery.err";  echo "$?" > "$WORK/out/battery.rc"; } &
  { D137_LEG_DIR="$WORK/g2"      python3 -W ignore "$MODEL/tools/battery_g2.py"     > "$WORK/out/g2.tsv"      2> "$WORK/out/g2.err";      echo "$?" > "$WORK/out/g2.rc"; } &
  { D137_LEG_DIR="$WORK/garbage" python3 -W ignore "$MODEL/tools/garbage_input.py"  > "$WORK/out/garbage.tsv" 2> "$WORK/out/garbage.err"; echo "$?" > "$WORK/out/garbage.rc"; } &
  { D137_LEG_DIR="$WORK/overblock" python3 -W ignore "$MODEL/tools/overblock.py"   > "$WORK/out/overblock.tsv" 2> "$WORK/out/overblock.err"; echo "$?" > "$WORK/out/overblock.rc"; } &
  wait
  local rc_files failed=0
  rc_files=$(ls "$WORK"/out/*.rc 2>/dev/null | wc -l)
  echo "rc_files=$rc_files (派出 ${#steps[@]})"
  [[ "$rc_files" -eq "${#steps[@]}" ]] || { echo "  ✗ 退出码文件数与派出去的步数对不上，整批作废" >&2; return 1; }
  for step in "${steps[@]}"; do
    if [[ "$(cat "$WORK/out/$step.rc")" != 0 ]]; then echo "  ✗ 步骤 $step 退 $(cat "$WORK/out/$step.rc")：$(tail -n 3 "$WORK/out/$step.err")" >&2; failed=$((failed + 1)); fi
  done
  local defs=".claude/agents/tooling-writer.md .claude/agents/implementation-writer.md .claude/agents/experiment-runner.md .claude/agents/investigator.md .claude/agents/three-way-forward.md .claude/agents/three-way-attack.md .claude/agents/three-way-defense.md .claude/agents/three-way-materials.md .claude/agents/mutation-triage.md"
  # shellcheck disable=SC2086
  python3 "$MODEL/tools/chkrefs.py" "$REPO_ROOT" $defs > "$WORK/out/refs.txt" 2>&1 || true
  local file
  : > "$WORK/out/fragments.txt"   # 本轮新句子（git diff HEAD 的词级新增），只作旁证，不进摘要
  for file in .claude/agent-common.md .claude/main-agent.md $defs .claude/rules/verification.md .claude/skills/crash-test/commit-stages.md CLAUDE.md; do
    { echo "=== $file"; python3 "$MODEL/tools/addfrag.py" "$REPO_ROOT" "$file"; } >> "$WORK/out/fragments.txt" 2>&1 || true
  done
  python3 "$MODEL/tools/summarize.py" "$WORK/out" > "$WORK/summary.txt"
  if diff "$MODEL/out/expected-summary.txt" "$WORK/summary.txt" > "$WORK/summary.diff"; then
    echo "  ✓ 摘要与 $MODEL/out/expected-summary.txt 逐行相同（$(wc -l < "$WORK/summary.txt") 行）"
  else
    echo "  ✗ 摘要与 expected-summary.txt 有差异（钩子或脚本在复跑之间变了，或负载下偶发）：" >&2; head -n 20 "$WORK/summary.diff" >&2; failed=$((failed + 1))
  fi
  for step in "${steps[@]}"; do du -sh "$WORK/$step/tree" 2>/dev/null | sed "s#^#删前 #"; rm -rf "$WORK/$step/tree"; done
  rm -rf "$WORK/base"
  local leftover
  leftover=$(ps -o pid,args -u "$(id -u)" | grep -E 'mysh_d137|sandbox/(heavy|g1)\.sh|dd[0-9]+\.sh' | grep -v grep | wc -l || true)
  echo "留下的样本进程=$leftover"
  [[ "$leftover" -eq 0 ]] || failed=$((failed + 1))
  return "$failed"
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  export RERUN_TEMPLATE_SELFTEST_MARK="rerun-template-selftest"   # 自证里拷出去起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
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

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RERUN_TEMPLATE_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${RERUN_TEMPLATE_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${RERUN_TEMPLATE_SELFTEST_MARK:-}" != rerun-template-selftest ]]; then
  echo "  ✗ RERUN_TEMPLATE_BREAK=${RERUN_TEMPLATE_BREAK} 是弄坏开关，只在自证（bash <这份脚本> --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 RERUN_TEMPLATE_BREAK 再跑；要证红就跑 RERUN_TEMPLATE_BREAK=<项> bash scripts/agents/three-way-rerun-template.sh --selftest" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
