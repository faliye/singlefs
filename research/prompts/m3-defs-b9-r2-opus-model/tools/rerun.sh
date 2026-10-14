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
# 第二题第二轮攻方腿（m3-defs-b9-r2-opus）。两份冻结副本都硬链接拷进 $WORK（拷贝里删掉 multi-host.env 那一条链接，冻结副本不被碰），
# 钩子与门禁格的判法只在拷贝上跑；第一轮冻结副本默认取 --frozen 同级的 m3-defs-b9-r1/repo（R1_FROZEN 可改）。
# RERUN_PARTS 选跑哪几组（默认 "s1 s3 s5 s6 s7"）；RERUN_CORPUS=1 另跑 S1 的现有派发提示那一组（读工作根下各草稿目录的 prompt.md，
# 那批文件不在仓里、会变，输出不与 out/ 比，只打合计行）。末尾逐份打 same / DIFF / 新出（先把 $WORK、$MODEL 换成占位再比）。
rerun_steps() {
  local r1_frozen="${R1_FROZEN:-$(dirname "$FROZEN")/../m3-defs-b9-r1/repo}" parts=" ${RERUN_PARTS:-s1 s3 s5 s6 s7} " tools="$MODEL/tools" out="$WORK/out"
  [[ -d "$r1_frozen" ]] || { echo "  ✗ 第一轮冻结副本 $r1_frozen 不在（用 R1_FROZEN 指过去）" >&2; return 2; }
  mkdir -p "$out"
  cp -al "$FROZEN" "$WORK/r2"; cp -al "$r1_frozen" "$WORK/r1"
  rm -f "$WORK/r2/multi-host.env" "$WORK/r1/multi-host.env"
  if [[ "$parts" == *" s1 "* ]]; then
    python3 -m venv --without-pip "$WORK/venv-no-markdown"
    nice -n 19 python3 -B "$tools/s1_fence_feed.py" --root r2="$WORK/r2" --root r1="$WORK/r1" --work "$WORK/s1" > "$out/s1-fence.txt"
    S1_VENV="$WORK/venv-no-markdown" nice -n 19 bash "$tools/s1_hosts_and_fallback.sh" "$WORK/r2" "$WORK/r1" "$WORK/s1h" > "$out/s1-hosts-fallback.txt"
    nice -n 19 python3 -B "$tools/s1_ambiguity_prototype.py" > "$out/s1-ambiguity-prototype.txt"
    if [[ "${RERUN_CORPUS:-0}" == 1 ]]; then
      nice -n 19 python3 -B "$tools/s1_corpus.py" --root "$WORK/r2" --work-root "${CORPUS_WORK_ROOT:-$HOME/.local/share/singlefs/work}" --skip m3-defs-b9-r2-sonnet > "$WORK/s1-corpus.txt"
      tail -n 1 "$WORK/s1-corpus.txt"
    fi
  fi
  if [[ "$parts" == *" s3 "* ]]; then
    local gate fixture
    for gate in r2 r1; do bash "$MODEL/feeds/s3/make_tree.sh" "$WORK/$gate" "$WORK/s3/tree-$gate"; done
    mkdir -p "$WORK/s3/patched/.claude/gate.d" "$WORK/s3/patched/scripts/hosts"
    cp "$WORK/r2/scripts/hosts/heavy_threads.py" "$WORK/s3/patched/scripts/hosts/"
    python3 -B "$tools/s3_patch.py" "$WORK/r2/.claude/gate.d/code-tooling.sh" "$WORK/s3/patched/.claude/gate.d/code-tooling.sh"
    for gate in r2 r1 patched; do
      local gate_root="$WORK/$gate" tree="$WORK/s3/tree-$gate"
      [[ "$gate" == patched ]] && gate_root="$WORK/s3/patched" tree="$WORK/s3/tree-r2"
      echo "== 门禁 $gate 判本腿样本"
      nice -n 19 python3 -B "$tools/s3_htd_cell.py" "$gate_root" "$tree" 2>&1 | grep -E '^ +scripts/|退出=|✓' || true
    done > "$out/s3-samples.txt"
    for gate in r2 patched; do
      local gate_root="$WORK/r2"; [[ "$gate" == patched ]] && gate_root="$WORK/s3/patched"
      for fixture in heavy-thread-defaults-var-source-green heavy-thread-defaults-var-source-red heavy-thread-defaults-per-line-red heavy-thread-defaults-per-line-green heavy-thread-defaults-green heavy-thread-defaults-red; do
        cp -r "$WORK/r2/.claude/gate.d/fixtures/code-tooling.sh/$fixture" "$WORK/s3/fixture-$gate-$fixture"
        printf '%s\t%s\t%s\n' "$gate" "$fixture" "$(nice -n 19 python3 -B "$tools/s3_htd_cell.py" "$gate_root" "$WORK/s3/fixture-$gate-$fixture" 2>&1 | grep -E '退出=|处自己定' | sed 's/（每台默认.*//' | tr '\n' ' ')"
      done
      printf '%s\t真树（这一轮冻结副本拷贝）\t%s\n' "$gate" "$(nice -n 19 python3 -B "$tools/s3_htd_cell.py" "$gate_root" "$WORK/r2" 2>&1 | grep -E '退出=|^  [✓✗]' | sed 's/，可疑写法.*//' | tr '\n' ' ')"
    done > "$out/s3-fixtures-and-real-tree.txt"
  fi
  if [[ "$parts" == *" s5 "* ]]; then
    local version
    for version in r2 r1; do nice -n 19 bash "$tools/s5_real_misread.sh" "$version=$WORK/$version" "$WORK/s5"; done > "$out/s5-real-misread.txt"
    nice -n 19 python3 -B "$tools/s5_combos.py" --lib r2="$WORK/r2/.claude/hooks/lib_abandoned_rounds.py" --lib r1="$WORK/r1/.claude/hooks/lib_abandoned_rounds.py" --work "$WORK/s5c" > "$out/s5-combos.txt"
  fi
  if [[ "$parts" == *" s6 "* ]]; then
    nice -n 19 python3 -B "$tools/s6_mask_shapes.py" "$WORK/r2" > "$out/s6-mask-shapes.txt"
  fi
  if [[ "$parts" == *" s7 "* ]]; then
    nice -n 19 bash "$tools/s7_user_per_host.sh" r2="$WORK/r2" r1="$WORK/r1" -- "$WORK/s7" > "$out/s7-user-per-host.txt"
  fi
  local file name
  for file in "$out"/*.txt; do
    name="$(basename "$file")"
    sed -e "s|$WORK|<WORK>|g" -e "s|$MODEL|<MODEL>|g" -e "s|$FROZEN|<FROZEN>|g" "$file" > "$file.placeholder" && mv "$file.placeholder" "$file"
    if [[ ! -f "$MODEL/out/$name" ]]; then echo "新出 $name"
    elif cmp -s "$file" "$MODEL/out/$name"; then echo "same $name"
    else echo "DIFF $name"; fi
  done
  rm -rf "$WORK/r2" "$WORK/r1"
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
