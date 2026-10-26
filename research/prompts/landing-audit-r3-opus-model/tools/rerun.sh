#!/usr/bin/env bash
# landing-audit-r3 攻方腿的复跑脚本（拷自 scripts/agents/three-way-rerun-template.sh，只改了 rerun_steps 那一段）。
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
  # 被判的脚本（agent-cost.py、派发闸、dispatch-prompt.py、cite-check.py）从 $FROZEN 读（这一轮没有冻结的仓副本，--frozen 给仓根）；
  # 会话记录读 ~/.claude/projects/<仓根换 - 的目录>；git 读 $REPO_ROOT。会话记录与工作树会长，往后复跑的数会变，最后逐份 diff 给出差在哪。
  local sessions="$HOME/.claude/projects/$(echo "$REPO_ROOT" | sed 's#/#-#g')" probes="$MODEL/probes" out="$WORK/out"
  mkdir -p "$out" "$WORK/tree"
  echo "== 结局账闭窗口：今天的 agent-cost.py 按东京日期切 / 第七批开工前那一份按 UTC 切"
  nice -n 19 python3 -I "$probes/outcome_dump.py" "$REPO_ROOT" "$FROZEN/scripts/agents/agent-cost.py" tokyo 2026-10-01 2026-10-21 4 "$out/today-tokyo"
  nice -n 19 python3 -I "$probes/outcome_dump.py" "$REPO_ROOT" "$MODEL/inputs/agent-cost-before-r2fix.py" utc 2026-10-01 2026-10-21 4 "$out/before-r2fix-utc"
  python3 -I "$probes/decompose.py" "$out/before-r2fix-utc-agents.jsonl" "$out/today-tokyo-agents.jsonl" > "$out/decompose.txt"
  { cat "$out/before-r2fix-utc-summary.txt"; echo "  ✓ 查了 2144 个子 agent（--since 2026-10-01、--until 2026-10-21（含那一天），按首条记录的时刻筛），逐个归了结局桶"; } | sha256sum > "$out/archive-sha256.txt"
  { cat "$out/today-tokyo-summary.txt"; echo "  ✓ 查了 2125 个子 agent（--since 2026-10-01、--until 2026-10-21（含那一天），日期按东京日期（Asia/Tokyo）切天，按首条记录的时刻筛），逐个归了结局桶"; } | sha256sum >> "$out/archive-sha256.txt"
  python3 -I "$probes/threshold_sweep.py" "$out/today-tokyo-agents.jsonl" > "$out/threshold-sweep-today.txt"
  python3 -I "$probes/threshold_sweep.py" "$out/before-r2fix-utc-agents.jsonl" > "$out/threshold-sweep-before-r2fix.txt"
  echo "== 派发闸报告行三种认法在真派发提示上"
  local guard="$FROZEN/.claude/hooks/runner-dispatch-guard.sh"
  nice -n 19 python3 -I "$probes/report_line_corpus.py" "$guard" "$REPO_ROOT" "$sessions" "$out/report-line-corpus" > /dev/null
  nice -n 19 python3 -I "$probes/report_line_examples.py" "$guard" "$REPO_ROOT" "$sessions" > "$out/report-line-examples.txt"
  nice -n 19 python3 -I "$probes/sentence_start_extra.py" "$guard" "$sessions" 2026-10-14 > "$out/sentence-start-extra.txt"
  nice -n 19 python3 -I "$probes/negation_corpus.py" "$guard" "$sessions" > "$out/negation-corpus.txt"
  python3 -I "$probes/write_refusals.py" "$sessions" a9717cf07fb6e7c21 a3343943b7a6b25ee a0a2dfee322e26786 a789e7091d91430aa a8afb2185cfa840cc > "$out/write-refusals.txt"
  python3 -I "$probes/name_verdict_variants.py" "$guard" implementation-writer "$MODEL/prompts/aa9ec3423c1f54bba-own-result.md" > "$out/name-verdict-variants.txt"
  python3 -I "$probes/name_verdict_variants.py" "$guard" experiment-runner "$MODEL/prompts/a4737c6f3660f4f7e-own-result.md" >> "$out/name-verdict-variants.txt"
  python3 -I "$probes/name_verdict_variants.py" "$guard" kb-scribe "$MODEL/prompts/a28fe10b58d88beec.md" >> "$out/name-verdict-variants.txt"
  for prompt in a3652270eb80e4fd0 a303d33b0ccd5e68f a3652270eb80e4fd0-delivery-colon a3652270eb80e4fd0-section-colon aa9ec3423c1f54bba-own-result a4737c6f3660f4f7e-own-result; do
    echo "== $prompt"; (cd "$FROZEN" && python3 scripts/agents/dispatch-prompt.py --check "$MODEL/prompts/$prompt.md" 2>&1 | grep -E '^✗|会拒|放行|认领行' | cut -c1-160) || true
  done > "$out/dispatch-check.txt"
  echo "== 三方腿自己的报告与认到的判决；Write 名字与工具层拒写；交回形状"
  nice -n 19 python3 -I "$probes/leg_own_report.py" "$out/today-tokyo-agents.jsonl" "$sessions" "$REPO_ROOT" > "$out/leg-own-report.txt"
  nice -n 19 python3 -I "$probes/write_name_refusals.py" "$sessions" > "$out/write-name-refusals.txt"
  nice -n 19 python3 -I "$probes/handback_shapes.py" > "$out/handback-shapes-today.txt"
  python3 -I "$probes/fix_word_end.py" "$sessions" > "$out/fix-word-end.txt"
  echo "== 第二轮快照重建与 cite-check"
  local r2="$WORK/r2tree" archive="research/prompts/subagent-landing-audit-2026-10-22"
  mkdir -p "$r2"
  python3 -I "$probes/rebuild_snapshot.py" "$REPO_ROOT" "$REPO_ROOT/research/prompts/landing-audit-r2-snapshot" "$r2" > "$out/r2-rebuild.txt"
  (cd "$r2" && git apply --unsafe-paths --whitespace=nowarn -p1 "$REPO_ROOT/research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff" 2>/dev/null)
  for name in blind-rework.md outcome-summary-since-2026-10-01.txt r1fix-definitions.md r1fix-gates.md r1fix-hooks.md tw-gate-scripts.md; do cp "$REPO_ROOT/$archive/$name" "$r2/$archive/$name"; done
  { cat "$out/before-r2fix-utc-summary.txt"; echo "  ✓ 查了 2144 个子 agent（--since 2026-10-01、--until 2026-10-21（含那一天），按首条记录的时刻筛），逐个归了结局桶"; } > "$r2/$archive/outcome-summary-2026-10-01-to-2026-10-21.txt"
  (cd "$r2" && sha256sum -c "$REPO_ROOT/research/prompts/landing-audit-r2-snapshot/judged-sha256.txt" 2>/dev/null | grep -c ': OK$') >> "$out/r2-rebuild.txt" || true
  (cd "$REPO_ROOT" && python3 "$FROZEN/scripts/agents/cite-check.py" research/prompts/landing-audit-r2-sonnet-output.md research/prompts/landing-audit-r2-opus-output.md --root "$r2" --background research/prompts/_landing-audit-r2-background.md 2>&1 | tail -1) > "$out/r2-cite-check-rebuilt.txt" || true
  echo "== 与 $MODEL/out/ 逐份比（会话记录与工作树会长，差在哪一行就是往后长出来的）"
  for f in "$out"/*.txt; do
    base="$(basename "$f")"
    if [[ -f "$MODEL/out/$base" ]]; then
      if cmp -s "$f" "$MODEL/out/$base"; then echo "  same $base"; else echo "  diff $base（$(diff "$f" "$MODEL/out/$base" | grep -c '^[<>]') 行不同）"; fi
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
