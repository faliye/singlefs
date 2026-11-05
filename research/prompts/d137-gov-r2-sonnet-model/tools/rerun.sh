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

# ── 这一腿要跑的步骤 ──
# 复跑：两棵树（now = 仓里此刻的钩子，base = 修补前备份换回）、日常命令语料、cd 语料、第一轮 G1 喂法与真 bash 对照、本仓 shell 脚本、可选的第一轮模型电池与真实语料。
# 输出全写 $WORK/out/；摘要行打在标准输出。真实语料与第一轮电池要额外的输入，没给就跳过并打出来。
rerun_steps() {
  local tools="$MODEL/tools" cases="$MODEL/cases" snapshot="$REPO_ROOT/research/prompts/d137-gov-r2-snapshot/files-sha256.txt"
  if [[ ! -f "$tools/mkroots.py" ]]; then   # 只拷了这一份脚本（模板自证）：照模板的默认步骤
    mkdir -p "$WORK/tree"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
    echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
    return 0
  fi
  mkdir -p "$WORK/out" "$WORK/cwd" "$WORK/sbx/sub"
  local path want got changed=0
  # 1 被判文件此刻的 sha256 对快照
  if [[ -f "$snapshot" ]]; then
    while read -r want path; do
      [[ "$path" == .claude/* || "$path" == scripts/* ]] || continue
      got="$(sha256sum "$REPO_ROOT/$path" 2>/dev/null | cut -d' ' -f1)"
      if [[ "$got" != "$want" ]]; then changed=$((changed + 1)); echo "name=snapshot_differs path=$path now=${got:0:8} snapshot=${want:0:8}"; fi
    done < "$snapshot"
    echo "name=snapshot_check differing_files=$changed"
  fi
  # 2 两棵树
  python3 -I "$tools/mkroots.py" --repo "$REPO_ROOT" --model "$MODEL" --work "$WORK"
  # 3 样本目录（带 heavy.sh / benign.sh 的相对脚本场景与真 bash 对照）
  printf 'cargo test --all\n' > "$WORK/sbx/heavy.sh"; printf 'cargo test --all\n' > "$WORK/sbx/e.sh"; printf 'echo hi\n' > "$WORK/sbx/benign.sh"; printf 'hello\n' > "$WORK/sbx/in.txt"
  printf 'echo sub\n' > "$WORK/sbx/sub/benign.sh"; printf 'cargo test --all\n' > "$WORK/sbx/sub/heavy.sh"
  chmod +x "$WORK/sbx/heavy.sh" "$WORK/sbx/benign.sh" "$WORK/sbx/sub/benign.sh" "$WORK/sbx/sub/heavy.sh"
  local probe=(env PROBE_THREADS=12 nice -n 19 python3 -I "$tools/probe.py")
  # 4 喂钩子
  "${probe[@]}" "$cases/build.py" --work "$WORK" --cwd "$WORK/now" > "$WORK/out/run-everyday-1.txt" 2>&1
  "${probe[@]}" "$cases/build2.py" --work "$WORK" --cwd "$WORK/now" > "$WORK/out/run-everyday-2.txt" 2>&1
  "${probe[@]}" "$cases/cdcases.py" --work "$WORK" --cwd "$WORK/sbx" > "$WORK/out/run-cd.txt" 2>&1
  "${probe[@]}" "$cases/r1-hits.py" --work "$WORK" --cwd "$WORK/sbx" > "$WORK/out/probe-r1-hits.txt" 2>&1
  "${probe[@]}" "$cases/r1-extra.py" --work "$WORK" --cwd "$WORK/sbx" > "$WORK/out/probe-r1-extra.txt" 2>&1
  REPO="$REPO_ROOT" "${probe[@]}" "$cases/repo-scripts.py" --work "$WORK" --cwd "$WORK/now" > "$WORK/out/run-repo-scripts.txt" 2>&1
  # 5 真 bash 对照（假 cargo 只记日志；每条样本 setsid --wait 起、超时对整组发 KILL）
  rm -f "$WORK/sbx/mysh" "$WORK/sbx/mysh2" "$WORK/sbx/myls" "$WORK/sbx/d7" "$WORK/sbx/m7"
  nice -n 19 python3 -I "$tools/truth.py" "$cases/r1-hits.py" "$WORK/sbx" 2>&1 | awk '!seen[$0]++' > "$WORK/out/truth-r1.txt"
  nice -n 19 python3 -I "$tools/truth.py" "$cases/r1-extra.py" "$WORK/sbx" 2>&1 | awk '!seen[$0]++' > "$WORK/out/truth-extra.txt"
  rm -f "$WORK/sbx/mysh" "$WORK/sbx/mysh2" "$WORK/sbx/myls" "$WORK/sbx/d7" "$WORK/sbx/m7"
  # 6 摘要
  local file
  for file in run-everyday-1 run-everyday-2 run-cd; do
    echo "name=summary file=$file $(python3 -I "$tools/analyze.py" "$WORK/out/$file.txt" 2>&1 >/dev/null | tr '\n' ' ')"
  done
  echo "name=summary file=run-repo-scripts rows_with_changed_exit_code=$(grep -c CHANGED "$WORK/out/run-repo-scripts.txt" || true)"
  echo "name=summary file=probe-r1-hits rows_base_allowed_now_refused=$(grep -c 'now:rc=2.*base:rc=0' "$WORK/out/probe-r1-hits.txt" || true)"
  echo "name=summary file=truth-r1 cargo_called_yes=$(grep -c 'cargo_called=yes' "$WORK/out/truth-r1.txt" || true) no=$(grep -c 'cargo_called=no' "$WORK/out/truth-r1.txt" || true)"
  # 7 可选：第一轮正推腿的电池（需要它的模型目录与桩目录）
  local r1="$REPO_ROOT/research/prompts/d137-gov-r1-sonnet-model" r1stubs="${R1_STUBS_DIRECTORY:-}" tree
  if [[ -d "$r1/tools" && -n "$r1stubs" && -d "$r1stubs" ]]; then
    for tree in now base; do
      mkdir -p "$WORK/r1leg/$tree/workroot"; ln -sfn "$WORK/$tree" "$WORK/r1leg/$tree/tree"; cp -r "$r1stubs" "$WORK/r1leg/$tree/stubs"
      bash "$r1/tools/setup_sandbox.sh" "$WORK/r1leg/$tree"
      D137_LEG_DIR="$WORK/r1leg/$tree" nice -n 19 python3 "$r1/tools/run_heavy.py" > "$WORK/out/heavy-$tree.tsv" 2> "$WORK/r1leg/heavy-$tree.err" || true
      D137_LEG_DIR="$WORK/r1leg/$tree" nice -n 19 python3 "$r1/tools/run_fuzz.py" > "$WORK/out/fuzz-$tree.tsv" 2> "$WORK/r1leg/fuzz-$tree.err" || true
      D137_LEG_DIR="$WORK/r1leg/$tree" nice -n 19 python3 "$r1/tools/dispatch_probe.py" > "$WORK/out/dispatch-$tree.txt" 2>&1 || true
    done
    echo "name=r1_batteries heavy_rows=$(wc -l < "$WORK/out/heavy-now.tsv") fuzz_rows=$(wc -l < "$WORK/out/fuzz-now.tsv")"
  else
    echo "name=r1_batteries skipped=true（要 R1_STUBS_DIRECTORY 指到第一轮正推腿的 stubs 目录）"
  fi
  # 8 可选：真实命令语料（会话记录里抽，含私有路径，只放 $WORK）
  if [[ -n "${REAL_COMMANDS:-}" && -f "${REAL_COMMANDS:-}" ]]; then
    REAL_CORPUS="$REAL_COMMANDS" "${probe[@]}" "$cases/real.py" --work "$WORK" --cwd "$WORK/now" > "$WORK/out/run-real.txt" 2>&1
    python3 -I "$tools/analyze-real.py" "$WORK/out/run-real.txt" "$REAL_COMMANDS" "$WORK/out/real-analysis.json" | head -1
    python3 -I "$tools/classify-sed.py" "$WORK/out/run-real.txt" | head -5
  else
    echo "name=real_corpus skipped=true（要 REAL_COMMANDS 指到 tools/extract-commands.py 抽出的 JSON 行文件）"
  fi
  # 9 收尾核：$WORK 下起的进程一个不剩
  echo "name=leftover_processes count=$(ps -eo pid,args | awk -v w="$WORK" 'index($0, w) && !/awk -v/ && !/ps -eo/ && !/rerun.sh/ && !/run-with-memory-cap/ && !/timeout /' | wc -l)"
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
