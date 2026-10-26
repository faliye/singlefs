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
  # 1) 搭两棵树：tree-orig = 冻结副本里的三份钩子把 own.diff 反向打回（改前），tree = 冻结副本原样（改后）；其余钩子库、脚本、定义、门禁表从 $REPO_ROOT 拷
  local side
  for side in tree-orig tree; do
    mkdir -p "$WORK/$side/.claude" "$WORK/$side/scripts" "$WORK/$side/crates"
    cp -r "$REPO_ROOT/.claude/hooks" "$WORK/$side/.claude/hooks"
    cp -r "$REPO_ROOT/.claude/scripts" "$WORK/$side/.claude/scripts"
    cp -r "$REPO_ROOT/.claude/agents" "$WORK/$side/.claude/agents"
    mkdir -p "$WORK/$side/.claude/singlefs-ai-sop"
    cp -r "$REPO_ROOT/.claude/singlefs-ai-sop/scripts" "$WORK/$side/.claude/singlefs-ai-sop/scripts"
    cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/$side/.claude/heavy-thread-default"
    local d
    for d in hosts gate mutation crash agents devices kb; do cp -r "$REPO_ROOT/scripts/$d" "$WORK/$side/scripts/$d"; done
    mkdir -p "$WORK/$side/scripts/experiment-apparatus" "$WORK/$side/.claude/gate.d"
    cp "$REPO_ROOT"/scripts/experiment-apparatus/*.sh "$REPO_ROOT"/scripts/experiment-apparatus/*.py "$WORK/$side/scripts/experiment-apparatus/" 2>/dev/null || true
    cp "$REPO_ROOT"/.claude/gate.d/*.sh "$REPO_ROOT"/.claude/gate.d/*.tsv "$WORK/$side/.claude/gate.d/" 2>/dev/null || true
    for f in bash-command-detector.sh heavy-test-guard.sh lib_shell_words.py lib_heavy_tests.py; do
      rm -f "$WORK/$side/.claude/hooks/$f"; cp "$FROZEN/.claude/hooks/$f" "$WORK/$side/.claude/hooks/$f"
    done
  done
  python3 -I - "$FROZEN/own.diff" "$WORK/tree-orig/.claude/hooks" <<'PY'
import re, subprocess, sys, os
diff_path, hooks = sys.argv[1], sys.argv[2]
for part in re.split(r"(?m)^(?=diff --git )", open(diff_path, encoding="utf-8").read()):
    if not part.strip():
        continue
    name = re.search(r"\+\+\+ b/own/(\S+)", part).group(1)
    subprocess.run(["patch", "-R", "-s", "-r", "/dev/null", os.path.join(hooks, name)], input=part, text=True, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
PY
  echo "name=orig_tree_brace_expand_mentions $(grep -c brace_expand "$WORK/tree-orig/.claude/hooks/lib_shell_words.py" || true)"
  echo "name=new_tree_brace_expand_mentions $(grep -c brace_expand "$WORK/tree/.claude/hooks/lib_shell_words.py" || true)"
  # 2) 探针的当前目录：s.sh 的第 4 行是 cargo test --all，light.sh 只有 echo
  mkdir -p "$WORK/scratch/sub"
  printf '#!/usr/bin/env bash\n# 验证链\nset -euo pipefail\ncargo test --all\n' > "$WORK/scratch/s.sh"
  printf '#!/usr/bin/env bash\necho hi\n' > "$WORK/scratch/light.sh"
  cp "$WORK/scratch/s.sh" "$WORK/scratch/s01.sh"; cp "$WORK/scratch/s.sh" "$WORK/scratch/s02.sh"
  mkdir -p "$WORK/out"
  export SW_ROOT="$WORK"
  # 3) 每个家族文件：两棵树各一个进程（进程内喂钩子），再并排成一行一条的矩阵
  local fam name
  local -a families=()
  for fam in "$MODEL"/families/*.json; do families+=("$fam"); done
  # 家族文件逐个、两棵树各一个进程，全部并行起，最后单独一个不带参数的 wait 等齐；每件的输出写进自己的文件，等齐后按次序读
  for fam in "${families[@]}"; do
    name="$(basename "$fam" .json)"
    { nice -n 19 python3 -I "$MODEL/tools/fam4.py" "$fam" tree-orig "$WORK/out/$name-orig.json"; echo "$?" > "$WORK/out/$name-orig.rc"; } &
    { nice -n 19 python3 -I "$MODEL/tools/fam4.py" "$fam" tree "$WORK/out/$name-new.json"; echo "$?" > "$WORK/out/$name-new.rc"; } &
  done
  wait
  [[ "$(ls "$WORK"/out/*.rc | wc -l)" -eq $((2 * ${#families[@]})) ]] || { echo "name=family_runs_missing" >&2; return 1; }
  cat "$WORK"/out/*.rc | sort | uniq -c | sed 's/^/name=family_run_exit_codes /'
  : > "$WORK/out/matrix.tsv"
  for fam in "${families[@]}"; do
    name="$(basename "$fam" .json)"
    python3 -I "$MODEL/tools/matrix.py" "$fam" "$WORK/out/$name-orig.json" "$WORK/out/$name-new.json" >> "$WORK/out/matrix.tsv"
  done
  LC_ALL=C sort -o "$WORK/out/matrix.tsv" "$WORK/out/matrix.tsv"   # 家族文件的 glob 次序随 locale 变，矩阵按字节序排死
  echo "name=matrix_rows $(wc -l < "$WORK/out/matrix.tsv")"
  if diff -q "$MODEL/out/matrix.tsv" "$WORK/out/matrix.tsv" > /dev/null; then echo "name=matrix_vs_model identical"; else echo "name=matrix_vs_model DIFFERENT"; { diff "$MODEL/out/matrix.tsv" "$WORK/out/matrix.tsv" || true; } | head -20; fi
  # 3b) 关键反例：真的钩子入口逐条喂（子进程，原样退出码与第一条 ✗ 行），路径里的草稿目录换成 <work> 再比
  SW_ROOT="$WORK" bash "$MODEL/tools/keyfeeds.sh" 2>&1 | sed "s#$WORK#<work>#g" > "$WORK/out/keyfeeds.txt" || true
  echo "name=keyfeeds_lines $(wc -l < "$WORK/out/keyfeeds.txt")"
  if diff -q "$MODEL/out/keyfeeds.txt" "$WORK/out/keyfeeds.txt" > /dev/null; then echo "name=keyfeeds_vs_model identical"; else echo "name=keyfeeds_vs_model DIFFERENT"; { diff "$MODEL/out/keyfeeds.txt" "$WORK/out/keyfeeds.txt" || true; } | head -20; fi
  # 4) 花括号：与真 bash 逐词对照
  python3 -I "$MODEL/tools/brace_cases.py" > "$WORK/out/brace-cases.txt" 2>&1 || true
  tail -n 1 "$WORK/out/brace-cases.txt" | sed 's/^/name=brace_cases /'
  python3 -I "$MODEL/tools/brace_oracle4.py" 11 8000 2>&1 | cut -c1-300 > "$WORK/out/brace-oracle.txt" || true
  head -n 1 "$WORK/out/brace-oracle.txt" | sed 's/^/name=brace_oracle /'
  if diff -q "$MODEL/out/brace-cases.txt" "$WORK/out/brace-cases.txt" > /dev/null; then echo "name=brace_cases_vs_model identical"; else echo "name=brace_cases_vs_model DIFFERENT"; fi
  if diff -q "$MODEL/out/brace-oracle.txt" "$WORK/out/brace-oracle.txt" > /dev/null; then echo "name=brace_oracle_vs_model identical"; else echo "name=brace_oracle_vs_model DIFFERENT"; fi
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
