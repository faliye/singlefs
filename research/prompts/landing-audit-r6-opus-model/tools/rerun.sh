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
  # 冻结副本（判决时给仓根或主 agent 建的副本）里的钩子与脚本拷成三棵树：tree-now（冻结副本此刻的）、tree-v35（$MODEL/orig/v35，任务 35 之后、任务 37 之前）、
  # tree-v10（$MODEL/orig/v10，第十批，第十一批之前；开工时取暂存区那一版），sha256 见 $MODEL/out/orig-sha256.txt。tree-r5orig 是 tree-v10 再盖上第二轮攻方 orig/ 里第十批之前的五份。
  # 真会话记录只读（$RERUN_TRANSCRIPTS 可改）；仓根 $REPO_ROOT 只读（定义 frontmatter、git worktree list、merge-and-verify.sh、第二轮攻方的 orig 与 corpus-agents.txt）。
  local projects="${RERUN_TRANSCRIPTS:-$HOME/.claude/projects/-home-fy5090-code-singlefs}" probes="$MODEL/probes" r5="$REPO_ROOT/research/prompts/landing-audit-r5-opus-model" name original
  bash "$MODEL/tools/make-trees.sh" "$FROZEN" "$MODEL" "$WORK" > "$WORK/trees.out"
  cp -a "$WORK/tree-v10" "$WORK/tree-r5orig"
  for original in "$r5/orig/"*.orig; do cp "$original" "$WORK/tree-r5orig/.claude/hooks/$(basename "$original" .orig)"; done
  rm "$WORK/tree-r5orig/.claude/hooks/lib_prompt_scratch_line.py"
  nice -n 19 python3 -I "$MODEL/tools/extract_corpus.py" "$projects" "$WORK/corpus.jsonl" 12
  local corpus="$WORK/corpus.jsonl" H=.claude/hooks
  nice -n 19 python3 -I "$probes/leg_paths.py" "$corpus" "$REPO_ROOT" v10="$WORK/tree-v10" v35="$WORK/tree-v35" now="$WORK/tree-now" > "$WORK/leg_paths.out" 2>&1
  nice -n 19 python3 -I "$probes/watch_progress_all.py" "$corpus" "$WORK/tree-v10/scripts/agents/agent-watch.py" "$WORK/tree-now/scripts/agents/agent-watch.py" 2026-10-22 > "$WORK/watch_progress_all.out" 2>&1
  nice -n 19 python3 -I "$probes/naming_e2e_all.py" "$corpus" "$projects" "$WORK" "$REPO_ROOT" v10="$WORK/tree-v10" now="$WORK/tree-now" > "$WORK/naming_e2e_all.out" 2>&1
  nice -n 19 python3 -I "$probes/naming_forms.py" "$WORK" "$REPO_ROOT" v10="$WORK/tree-v10" now="$WORK/tree-now" > "$WORK/naming_forms.out" 2>&1
  nice -n 19 python3 -I "$probes/leg_prefix_variants.py" "$WORK/tree-now/$H" "$REPO_ROOT" "$WORK" > "$WORK/leg_prefix_variants.out" 2>&1
  nice -n 19 bash "$probes/rerun_command_feed.sh" "$REPO_ROOT" "$FROZEN/$H" > "$WORK/rerun_command_feed.out" 2>&1
  nice -n 19 python3 -I "$probes/naming_result_shapes.py" "$projects" > "$WORK/naming_result_shapes.out" 2>&1
  (cd "$WORK" && {
    echo "== 交回闸 v10→now"; python3 -I "$probes/function_diff.py" tree-v10/$H/handback-guard.sh tree-now/$H/handback-guard.sh scribe_spec_entries missing_targets_sections missing_targets_listed scribe_target_problems prompt_scratch_directories first_prompt prompt_label_paths citation_checked_agents is_project_agent green_count_problems background_problems decide naming_source_tree
    echo "== 派发闸 v10→now"; python3 -I "$probes/function_diff.py" tree-v10/$H/runner-dispatch-guard.sh tree-now/$H/runner-dispatch-guard.sh report_file_name_verdict three_way_report_verdict prompt_scratch_directories report_scratch_directories three_way_report_types own_report_token
    echo "== 库 v10→now"; python3 -I "$probes/function_diff.py" tree-v10/$H/lib_three_way_legs.py tree-now/$H/lib_three_way_legs.py frontmatter_fields three_way_leg_types three_way_leg_types_scanned is_leg_report_path leg_report_mentions
    echo "== 草稿目录库 v10→now 逐行差几行"; diff tree-v10/$H/lib_prompt_scratch_line.py tree-now/$H/lib_prompt_scratch_line.py | grep -c '^[<>]' || true
  } > "$WORK/function_diff.out" 2>&1)
  # 第二轮判决的 402 份与 4 份：照第二轮攻方的两份探针（原样拷进 probes/r5_*.py）、它的 3343 份清单，tree-cur 取第十批（v10）
  nice -n 19 python3 -I "$probes/r5_report_verdicts.py" "$corpus" "$WORK/tree-v10/$H" "$WORK/tree-r5orig/$H" "$REPO_ROOT" "$r5/out/corpus-agents.txt" > "$WORK/r5_report_verdicts.out" 2>&1
  nice -n 19 python3 -I "$probes/r5_naming_real.py" "$corpus" "$WORK/tree-v10/$H" "$WORK/tree-r5orig/$H" "$REPO_ROOT" "$r5/out/corpus-agents.txt" > "$WORK/r5_naming_real.out" 2>&1
  rm -rf "${WORK:?}/tree-now" "${WORK:?}/tree-v35" "${WORK:?}/tree-v10" "${WORK:?}/tree-r5orig" "${WORK:?}/corpus.jsonl" "${WORK:?}/e2e"
  for name in leg_paths naming_e2e_all naming_forms leg_prefix_variants rerun_command_feed function_diff r5_report_verdicts r5_naming_real naming_result_shapes watch_progress_all; do
    if diff -q "$MODEL/out/$name.out" "$WORK/$name.out" > /dev/null; then echo "name=$name 与 out/ 逐行相同"; else echo "name=$name 与 out/ 不同（新落的会话记录与此刻的文件系统会让份数变，比数要对着 out/ 逐行看）"; fi
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
