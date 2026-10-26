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
  # 冻结副本里被判的钩子与脚本拷进草稿目录的 tree-cur/（改后）；tree-orig/ 是同一份再盖上 $MODEL/orig/ 里第十批之前的五份原文（存成 <文件名>.orig，sha256 见 out/orig-sha256.txt）、去掉新库（改前）。
  # 真会话记录只读：照 $MODEL/out/corpus-agents.txt 那 3343 份抽语料（抽不齐退 1）。仓根 $REPO_ROOT 只读（定义 frontmatter、r3 真提示、naming-lint.sh）。
  local projects="${RERUN_TRANSCRIPTS:-$HOME/.claude/projects/-home-fy5090-code-singlefs}" probes="$MODEL/probes" name
  for tree in tree-cur tree-orig; do
    mkdir -p "$WORK/$tree/.claude/singlefs-ai-sop" "$WORK/$tree/.claude/scripts" "$WORK/$tree/scripts"
    cp -a "$FROZEN/.claude/hooks" "$FROZEN/.claude/agents" "$WORK/$tree/.claude/"
    cp "$FROZEN/.claude/heavy-thread-default" "$WORK/$tree/.claude/"
    cp -a "$FROZEN/.claude/singlefs-ai-sop/scripts" "$WORK/$tree/.claude/singlefs-ai-sop/"
    cp "$FROZEN/.claude/scripts/project_preflight.py" "$WORK/$tree/.claude/scripts/"
    cp -a "$FROZEN/scripts/agents" "$FROZEN/scripts/kb" "$FROZEN/scripts/hosts" "$WORK/$tree/scripts/"
  done
  for original in "$MODEL/orig/"*.orig; do cp "$original" "$WORK/tree-orig/.claude/hooks/$(basename "$original" .orig)"; done
  rm "$WORK/tree-orig/.claude/hooks/lib_prompt_scratch_line.py"
  cp -a "$WORK/tree-cur" "$WORK/tree-fix"
  nice -n 19 python3 -I "$probes/make_fix.py" "$WORK/tree-fix" > /dev/null
  nice -n 19 python3 -I "$MODEL/tools/extract_corpus.py" "$projects" "$WORK/corpus.jsonl" 12 --only "$MODEL/out/corpus-agents.txt"
  local corpus="$WORK/corpus.jsonl" ids="$MODEL/out/corpus-agents.txt" cur="$WORK/tree-cur/.claude/hooks" orig="$WORK/tree-orig/.claude/hooks" fix="$WORK/tree-fix/.claude/hooks"
  nice -n 19 python3 -I "$probes/scratch_lines.py" "$corpus" "$cur" "$orig" "$FROZEN/scripts/agents" "$ids" > "$WORK/scratch_lines.out" 2>&1
  nice -n 19 python3 -I "$probes/report_verdicts.py" "$corpus" "$cur" "$orig" "$REPO_ROOT" "$ids" > "$WORK/report_verdicts.out" 2>&1
  nice -n 19 python3 -I "$probes/report_verdicts.py" "$corpus" "$fix" "$orig" "$REPO_ROOT" "$ids" > "$WORK/report_verdicts_fix.out" 2>&1
  nice -n 19 python3 -I "$probes/flip_sample.py" "$corpus" "$cur" "$orig" "$probes" > "$WORK/flip_sample.out" 2>&1
  nice -n 19 python3 -I "$probes/adhoc_recent.py" "$corpus" "$cur" "$orig" "$FROZEN/scripts/agents" "$probes" > "$WORK/adhoc_recent.out" 2>&1
  nice -n 19 python3 -I "$probes/fence_rules.py" "$cur" "$corpus" "$ids" > "$WORK/fence_rules.out" 2>&1
  nice -n 19 python3 -I "$probes/naming_real.py" "$corpus" "$cur" "$orig" "$REPO_ROOT" "$ids" > "$WORK/naming_real.out" 2>&1
  nice -n 19 python3 -I "$probes/echo_status.py" "$corpus" "$cur" "$orig" "$ids" > "$WORK/echo_status.out" 2>&1
  nice -n 19 python3 -I "$probes/dump_naming.py" "$corpus" a7aa1608c74c3e21e aa87dd9fe761d727f aa68d151c2fa72bb7 ab6dd5f35228122b7 > "$WORK/dump_naming.out" 2>&1
  nice -n 19 python3 -I "$probes/naming_e2e.py" "$WORK" "$REPO_ROOT" "$projects" a7aa1608c74c3e21e aa87dd9fe761d727f aa68d151c2fa72bb7 ab6dd5f35228122b7 > "$WORK/naming_e2e.out" 2>&1
  nice -n 19 python3 -I "$probes/naming_masks.py" "$WORK" "$REPO_ROOT" > "$WORK/naming_masks.out" 2>&1
  nice -n 19 bash "$probes/sigpipe.sh" "$WORK" "$REPO_ROOT" > "$WORK/sigpipe.out" 2>&1
  nice -n 19 python3 -I "$probes/leg_forms.py" "$cur" "$REPO_ROOT" > "$WORK/leg_forms.out" 2>&1
  nice -n 19 python3 -I "$probes/leg_handbacks.py" "$corpus" "$cur" "$orig" "$REPO_ROOT" "$ids" > "$WORK/leg_handbacks.out" 2>&1
  nice -n 19 python3 -I "$probes/import_break.py" "$WORK" "$WORK/tree-cur" > "$WORK/import_break.out" 2>&1
  nice -n 19 python3 -I "$probes/real_cells_fix.py" "$cur" "$fix" "$REPO_ROOT" > "$WORK/real_cells_fix.out" 2>&1
  nice -n 19 bash "$probes/real_prompts_dir.sh" "$WORK/tree-cur" "$REPO_ROOT" > "$WORK/real_prompts_dir.out" 2>&1
  (cd "$REPO_ROOT" && nice -n 19 python3 -I "$probes/feed_heavy.py" "$FROZEN/.claude/hooks/heavy-test-guard.sh" "$REPO_ROOT" main     "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 crash-case-replay、feature-gated-tests 格; 范围=2 格; 为什么=合入后验证这几格红了，修完只重跑它们' bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash .claude/gate.d/checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests $REPO_ROOT") > "$WORK/feed_heavy.out" 2>&1
  # 看门狗那一格按此刻的文件系统取 progress.md，复跑的数会随别的目录变，不比 out/
  nice -n 19 python3 -I "$probes/watch_progress.py" "$corpus" "$cur" "$FROZEN/scripts/agents" "$ids" 2026-10-22 > "$WORK/watch_progress.out" 2>&1
  rm -rf "${WORK:?}/tree-cur" "${WORK:?}/tree-orig" "${WORK:?}/tree-fix" "${WORK:?}/corpus.jsonl"
  for name in scratch_lines report_verdicts report_verdicts_fix flip_sample adhoc_recent fence_rules naming_real echo_status dump_naming naming_e2e naming_masks sigpipe leg_forms leg_handbacks import_break real_cells_fix real_prompts_dir feed_heavy; do
    if diff -q "$MODEL/out/$name.out" "$WORK/$name.out" > /dev/null; then echo "name=$name 与 out/ 逐行相同"; else echo "name=$name 与 out/ 不同"; fi
  done
  echo "name=watch_progress 不比（取此刻的文件系统）：$(head -c 200 "$WORK/watch_progress.out")"
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
