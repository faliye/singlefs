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
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""; RERUN_PART="${RERUN_PART:-all}"
  while (($#)); do
    case "$1" in
      --part) RERUN_PART="$2"; shift 2 ;;
      --stride) RERUN_STRIDE="$2"; shift 2 ;;
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
  if [[ "$RERUN_PART" == setup || "$RERUN_PART" == all ]] && [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤 ──
# 分块跑：--part setup | L | F | S | D | M | G | report | all；--stride N 每 N 条取 1 条（默认 1 = 全量；机器忙时全量要半小时以上，复核用 --stride 8 约 5 分钟）
# setup：从 $REPO_ROOT 拷出桩树（钩子换成 $FROZEN 里的五份）、仓副本、假家目录、工作根；其余每块跑一族样本，report 汇总。
run_corpus() {   # run_corpus <语料模块> <输出名> [--slice a:b]
  local module="$1" outname="$2"; shift 2
  ORACLE_HOME="$WORK/fakehome" nice -n 19 bash -c "ulimit -v 4194304; python3 '$MODEL/tools/run_shards.py' --work '$WORK' --corpus '$module' --out '$WORK/out/$outname.jsonl' --workers 4 $*"
}
setup_tree() {
  mkdir -p "$WORK/out" "$WORK/oraclelog" "$WORK/wr" "$WORK/u" "$WORK/fakehome/.local/bin" "$WORK/wr/copy/crates/singlefs-checker-tier"
  local R="$WORK/repo"
  mkdir -p "$R/.claude" "$R/scripts/experiment-apparatus" "$R/scripts/kb" "$R/.claude/gate.d"
  cp -r "$REPO_ROOT/.claude/hooks" "$R/.claude/hooks"
  for f in bash-command-detector.sh heavy-test-guard.sh lib_heavy_tests.py lib_shell_words.py lib_workspace_cargo_lock.py; do cp "$FROZEN/.claude/hooks/$f" "$R/.claude/hooks/$f"; done
  cp -r "$REPO_ROOT/.claude/scripts" "$R/.claude/scripts"; cp -r "$REPO_ROOT/.claude/agents" "$R/.claude/agents"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$R/.claude/"; cp -r "$REPO_ROOT/.claude/singlefs-ai-sop" "$R/.claude/singlefs-ai-sop"
  for d in gate hosts crash mutation agents devices; do cp -r "$REPO_ROOT/scripts/$d" "$R/scripts/$d"; done
  cp "$REPO_ROOT/scripts/experiment-apparatus/replay.sh" "$REPO_ROOT/scripts/experiment-apparatus/e165-pin-red-examples.py" "$R/scripts/experiment-apparatus/"
  cp "$REPO_ROOT"/scripts/kb/*.py "$R/scripts/kb/"
  cp "$REPO_ROOT"/.claude/gate.d/*.sh "$R/.claude/gate.d/"; cp -r "$REPO_ROOT/.claude/gate.d/lib" "$R/.claude/gate.d/lib"
  cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$R/"
  for c in "$REPO_ROOT"/crates/*/Cargo.toml; do mkdir -p "$R/crates/$(basename "$(dirname "$c")")"; cp "$c" "$R/crates/$(basename "$(dirname "$c")")/Cargo.toml"; done
  find "$R" -name __pycache__ -prune -exec rm -rf {} \;
  bash "$MODEL/tools/build_tree.sh" "$R" "$MODEL/tools/stublib.sh"
  for i in 2 3 4; do cp -a "$R" "$WORK/repo$i"; done
  cp "$REPO_ROOT/Cargo.toml" "$WORK/wr/copy/"; cp "$REPO_ROOT/crates/singlefs-checker-tier/Cargo.toml" "$WORK/wr/copy/crates/singlefs-checker-tier/"
  printf 'export PATH=%s:$PATH\n' "$MODEL/tools/bin" > "$WORK/fakehome/.profile"; cp "$WORK/fakehome/.profile" "$WORK/fakehome/.bash_profile"
  printf '#!/usr/bin/env bash\nexec %s/claude "$@"\n' "$MODEL/tools/bin" > "$WORK/fakehome/.local/bin/claude"; chmod +x "$WORK/fakehome/.local/bin/claude"
  echo "setup: 桩树 $R（含 repo2..4）、仓副本 $WORK/wr/copy、假家目录 $WORK/fakehome 已建"
}
want() { [[ "$PART" == "$1" || "$PART" == all ]]; }
rerun_steps() {
  PART="${RERUN_PART:-all}"
  STRIDE="${RERUN_STRIDE:-1}"
  if want setup; then setup_tree; fi
  if want L; then run_corpus corpus_launch L-a --stride "$STRIDE"; fi
  if want F; then run_corpus corpus_forms F-a --stride "$STRIDE"; fi
  if want S; then run_corpus corpus_scripts S-a --stride "$STRIDE"; fi
  if want D; then run_corpus corpus_detect D-a --stride "$STRIDE"; fi
  if want M; then run_corpus corpus_misc M-a --stride "$STRIDE"; fi
  if want G; then run_corpus corpus_more G-a --stride "$STRIDE"; fi
  if want report; then
    python3 "$MODEL/tools/report.py" "$WORK/out" "$WORK/summary.txt"
    cat "$WORK/summary.txt"
    # 与随报告带的 out/ 里同 id 的逐条比（该判、钩子判），不一致的件数就是退出码
    python3 "$MODEL/tools/compare.py" "$MODEL/out" "$WORK/out"
  fi
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
