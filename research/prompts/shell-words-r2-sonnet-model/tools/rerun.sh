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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；不起 cargo；输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。
# 两棵树：tree = 冻结副本里的四份钩子原样（修补后），tree-orig = 同四份把 fix.diff 反向打回（修补前）；pre/ 里再往前一步（r1.diff 也反向打回，第一轮判决之前）。
rerun_steps() {
  local side d f name
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  for side in tree-orig tree; do
    mkdir -p "$WORK/$side/.claude/singlefs-ai-sop" "$WORK/$side/scripts" "$WORK/$side/crates" "$WORK/$side/.claude/gate.d" "$WORK/$side/scripts/experiment-apparatus"
    cp -r "$REPO_ROOT/.claude/hooks" "$REPO_ROOT/.claude/scripts" "$REPO_ROOT/.claude/agents" "$WORK/$side/.claude/"
    cp -r "$REPO_ROOT/.claude/singlefs-ai-sop/scripts" "$WORK/$side/.claude/singlefs-ai-sop/"
    cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/$side/.claude/heavy-thread-default"
    for d in hosts gate mutation crash agents devices kb; do cp -r "$REPO_ROOT/scripts/$d" "$WORK/$side/scripts/"; done
    cp "$REPO_ROOT"/scripts/experiment-apparatus/*.sh "$REPO_ROOT"/scripts/experiment-apparatus/*.py "$WORK/$side/scripts/experiment-apparatus/" 2>/dev/null || true
    cp "$REPO_ROOT"/.claude/gate.d/*.sh "$REPO_ROOT"/.claude/gate.d/*.tsv "$WORK/$side/.claude/gate.d/" 2>/dev/null || true
    for f in bash-command-detector.sh heavy-test-guard.sh lib_shell_words.py lib_heavy_tests.py; do cp -f "$FROZEN/.claude/hooks/$f" "$WORK/$side/.claude/hooks/$f"; done
  done
  (cd "$WORK/tree-orig" && patch -R -p1 -s < "$FROZEN/fix.diff")
  mkdir -p "$WORK/pre/det" "$WORK/pre/scratch/sub"
  cp -r "$WORK/tree-orig" "$WORK/pre/tree-orig"
  ln -s "$WORK/tree" "$WORK/pre/tree"
  python3 -I - "$FROZEN/r1.diff" "$WORK/pre/tree-orig/.claude/hooks" <<'PY'
import re, subprocess, sys, os
diff_path, hooks = sys.argv[1], sys.argv[2]
for part in re.split(r"(?m)^(?=diff --git )", open(diff_path, encoding="utf-8").read()):
    if not part.strip():
        continue
    name = re.search(r"\+\+\+ b/own/(\S+)", part).group(1)
    subprocess.run(["patch", "-R", "-s", "-r", "/dev/null", os.path.join(hooks, name)], input=part, text=True, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
PY
  printf '#!/usr/bin/env bash\n# 验证链\nset -euo pipefail\ncargo test --all\n' > "$WORK/pre/scratch/s.sh"
  printf '#!/usr/bin/env bash\necho hi\n' > "$WORK/pre/scratch/light.sh"
  cp "$WORK/pre/scratch/s.sh" "$WORK/pre/scratch/s01.sh"; cp "$WORK/pre/scratch/s.sh" "$WORK/pre/scratch/s02.sh"
  for side in tree tree-orig; do
    (cd "$WORK/$side/.claude/hooks" && sha256sum lib_shell_words.py bash-command-detector.sh heavy-test-guard.sh lib_heavy_tests.py | sed "s#^#name=fingerprint_$side #")
  done
  (cd "$WORK/pre/tree-orig/.claude/hooks" && sha256sum lib_shell_words.py bash-command-detector.sh heavy-test-guard.sh | sed "s#^#name=fingerprint_pre_r1 #")
  # 2) 草稿根：假 cargo、探针脚本（模板）、检出记录目录
  mkdir -p "$WORK/scratch/sub" "$WORK/det" "$WORK/oracle/bin" "$WORK/out" "$WORK/cases"
  cp -r "$MODEL/template/." "$WORK/scratch/"
  cp "$MODEL/oracle/bin/cargo" "$WORK/oracle/bin/cargo"
  export SW_ROOT="$WORK" SINGLEFS_WORK_ROOT=/var/empty/swr
  # 3) 真钩子入口逐条喂（第一轮的关键反例，加第二轮追加的）
  SW_ROOT="$WORK" bash "$MODEL/tools/keyfeeds.sh" 2>&1 | sed "s#$WORK#<work>#g" > "$WORK/out/keyfeeds.txt" || true
  echo "name=keyfeeds_lines $(wc -l < "$WORK/out/keyfeeds.txt")"
  # 4) 花括号词表与整数序列对真 bash（修补前后各一棵树）
  : > "$WORK/out/totals.txt"
  for side in tree-orig tree; do
    python3 -I "$MODEL/tools/brace_diff2.py" "$WORK/$side" 1 3000 > "$WORK/out/brace-diff-$side-1.txt"
    head -n 1 "$WORK/out/brace-diff-$side-1.txt" | sed "s#^#brace_diff2 $side seed1 #" >> "$WORK/out/totals.txt"
    python3 -I "$MODEL/tools/seq_diff.py" "$WORK/$side" 3 1500 > "$WORK/out/seq-diff-$side.txt"
    tail -n 1 "$WORK/out/seq-diff-$side.txt" | sed "s#^#seq_diff #" >> "$WORK/out/totals.txt"
  done
  python3 -I "$MODEL/tools/brace_diff2.py" "$WORK/tree" 2 3000 > "$WORK/out/brace-diff-tree-2.txt"
  head -n 1 "$WORK/out/brace-diff-tree-2.txt" | sed "s#^#brace_diff2 tree seed2 #" >> "$WORK/out/totals.txt"
  # 5) 命令清单：按族造，加两份随机的（种子写死）
  python3 -I "$MODEL/tools/mkcases.py" "$WORK/cases" "$MODEL/families" > "$WORK/out/mkcases.txt"
  python3 -I "$MODEL/tools/fuzz2.py" 2 3000 "$WORK/cases/cases-fuzz2.json" > /dev/null
  python3 -I "$MODEL/tools/fuzzhead.py" 5 3000 "$WORK/cases/cases-head1.json" > /dev/null
  # 6) 逐族：真 bash 当裁判，修补前后两棵树各判一遍；每族分 12 块并行（pjudge.sh 里 wait 等齐）
  : > "$WORK/out/flags.txt"
  for name in key cpforms brace heredoc redirect ln env shell builtin function fncall empties hwrite ansic r1-575 fuzz2 head1; do
    bash "$MODEL/tools/pjudge.sh" "$WORK" "$WORK/cases/cases-$name.json" 12 "$WORK/out/p-$name" > "$WORK/out/p-$name.log" 2>&1 || { echo "name=pjudge_failed $name" >&2; return 1; }
    echo "$name $(tail -n 1 "$WORK/out/p-$name.txt")" >> "$WORK/out/totals.txt"
    case "$name" in   # 大族只留「修补后漏拒」与「前对后误拒」两种，别的差异看每族的合计行
      shell|function|fncall|fuzz2|head1) grep '<==' "$WORK/out/p-$name.txt" | grep -E '后=[0-9]+\(漏拒\)|前=[0-9]+\(对\)\|后=[0-9]+\(误拒\)' | sed "s#^#[$name] #" >> "$WORK/out/flags.txt" || true ;;
      *) grep '<==' "$WORK/out/p-$name.txt" | sed "s#^#[$name] #" >> "$WORK/out/flags.txt" || true ;;
    esac
  done
  # 7) 第一轮 575 条：第一轮判决之前的树（pre/tree-orig）→ 修补后的树，沿用第一轮的草稿根形状（探针目录里没有 scripts/hosts）
  bash "$MODEL/tools/runfams.sh" "$WORK/pre" "$MODEL/families" "$WORK/pre/out" > "$WORK/out/pre-matrix.txt" 2>&1
  sed -n '/matrix_rows/,$p' "$WORK/out/pre-matrix.txt" | sed 's/^/r1_575_pre_vs_fixed /' >> "$WORK/out/totals.txt"
  # 7b) 上面「第一轮判决之前拒、修补后放」的那几条，交真 bash 裁判（改前 = 修补前那棵树）：放得对不对
  python3 -I "$MODEL/tools/mk2to0.py" "$WORK/pre" "$WORK/cases/cases-r1-2to0.json" > "$WORK/out/mk2to0.txt"
  bash "$MODEL/tools/pjudge.sh" "$WORK" "$WORK/cases/cases-r1-2to0.json" 4 "$WORK/out/p-r1-2to0" > "$WORK/out/p-r1-2to0.log" 2>&1 || { echo "name=pjudge_failed r1-2to0" >&2; return 1; }
  echo "r1-2to0 $(tail -n 1 "$WORK/out/p-r1-2to0.txt")" >> "$WORK/out/totals.txt"
  grep '<==' "$WORK/out/p-r1-2to0.txt" | sed "s#^#[r1-2to0] #" >> "$WORK/out/flags.txt" || true
  # 7c) 小事：内建表对 bash、运算符对象的隐患、版本；长命令的挂钟（不进 totals）
  python3 -I "$MODEL/tools/small_checks.py" "$WORK/tree" | sed 's/^/small_checks /' >> "$WORK/out/totals.txt"
  python3 -I "$MODEL/tools/perf.py" tree-orig tree > "$WORK/out/perf.txt" 2>&1 || true
  echo "name=totals_lines $(wc -l < "$WORK/out/totals.txt")"
  echo "name=flags_lines $(wc -l < "$WORK/out/flags.txt")"
  cat "$WORK/out/totals.txt"
  for name in totals.txt flags.txt keyfeeds.txt; do
    if diff -q "$MODEL/out/$name" "$WORK/out/$name" > /dev/null 2>&1; then echo "name=${name}_vs_model identical"; else echo "name=${name}_vs_model DIFFERENT"; { diff "$MODEL/out/$name" "$WORK/out/$name" 2>&1 || true; } | head -n 12; fi
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
