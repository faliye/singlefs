#!/usr/bin/env bash
# landing-audit-r5 辩方腿的 tools/rerun.sh：由 scripts/agents/three-way-rerun-template.sh 拷来，只改了 rerun_steps 那一段（模板自带的自证测的是模板那一段，这份不再跑它）。
# 会话目录与峰值表是本机的活数据，rerun_steps 用截止秒数 LANDING_AUDIT_R5_CUTOFF（纪元秒，默认 1792722600，在这一腿被派出去之前）把记录时刻不早于它的子 agent 与派发丢掉。
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
  local probes="$MODEL/probes" cutoff="${LANDING_AUDIT_R5_CUTOFF:-1792722600}" projects="${CLAUDE_PROJECTS_DIR:-$HOME/.claude/projects/-home-fy5090-code-singlefs}"
  local peaks="${MEMORY_PEAKS_TABLE:-$HOME/.local/share/singlefs/memory-peaks.tsv}"
  mkdir -p "$WORK/trees" "$WORK/extracted" "$WORK/out"
  # 三棵交回闸与派发闸的树：.claude/hooks 各是各自那一版，.claude 的其余项与 scripts 符号链接回冻结副本（钩子要读登记与门禁表）
  local name entry
  for name in old new only0; do
    mkdir -p "$WORK/trees/tree_$name/.claude"
    cp -a "$FROZEN/.claude/hooks" "$WORK/trees/tree_$name/.claude/hooks"
    for entry in "$FROZEN"/.claude/* "$FROZEN"/.claude/.[a-z]*; do
      [[ -e "$entry" && "$(basename "$entry")" != hooks ]] && ln -s "$entry" "$WORK/trees/tree_$name/.claude/$(basename "$entry")"
    done
    ln -s "$FROZEN/scripts" "$WORK/trees/tree_$name/scripts"
  done
  cp "$MODEL"/inputs/pre-b10-hooks/* "$WORK/trees/tree_old/.claude/hooks/"          # 第九批的四份（落地审计第十批开工前的原文）
  rm -f "$WORK/trees/tree_old/.claude/hooks/lib_prompt_scratch_line.py"             # 第十批新建的库，第九批没有
  sed -i 's/^NAMING_COMPLETED_EXIT_CODES = (0, 1)/NAMING_COMPLETED_EXIT_CODES = (0,)/' "$WORK/trees/tree_only0/.claude/hooks/handback-guard.sh"
  grep -q '^NAMING_COMPLETED_EXIT_CODES = (0,)$' "$WORK/trees/tree_only0/.claude/hooks/handback-guard.sh"
  run_probe() {   # run_probe <名> <命令…>：输出落 $WORK/out/<名>.out，再打到屏幕
    local probe_name="$1"; shift
    nice -n 19 "$@" > "$WORK/out/$probe_name.out" 2>&1
    cat "$WORK/out/$probe_name.out"
  }
  run_probe extract python3 -I "$probes/extract_prompts.py" "$projects" "$WORK/extracted" "$cutoff"
  run_probe scratch_line_stats python3 -I "$probes/scratch_line_stats.py" "$WORK/extracted" "$FROZEN"
  run_probe guard_compare python3 -I "$probes/guard_compare.py" "$WORK/extracted" "$WORK/trees" "$REPO_ROOT" 2026-10-09
  run_probe handback_compare python3 -I "$probes/handback_compare.py" "$WORK/extracted" "$WORK/trees" "$REPO_ROOT"
  run_probe naming_stats python3 -I "$probes/naming_stats.py" "$WORK/extracted"
  run_probe shapes python3 -I "$probes/shapes.py" "$WORK/trees" "$REPO_ROOT" "$WORK/shapes-run"
  run_probe misc_facts python3 -I "$probes/misc_facts.py" "$REPO_ROOT" "$peaks"
  # 第一轮攻方模型里那十种命名检查写法（research/prompts/landing-audit-r4-opus-model/probes/naming_probe.py），分别喂新与只认退 0 两棵交回闸
  for name in new only0; do
    mkdir -p "$WORK/opus_$name"
    ln -s "$WORK/trees/tree_$name" "$WORK/opus_$name/tree"
    run_probe "opus_naming_probe_$name" python3 -I "$REPO_ROOT/research/prompts/landing-audit-r4-opus-model/probes/naming_probe.py" "$WORK/opus_$name" "$REPO_ROOT"
  done
  for name in extract scratch_line_stats guard_compare handback_compare naming_stats shapes misc_facts opus_naming_probe_new opus_naming_probe_only0; do
    if diff -q "$MODEL/out/$name.out" "$WORK/out/$name.out" > /dev/null; then echo "name=$name 与 out/ 逐行相同"; else echo "name=$name 与 out/ 不同"; fi
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
