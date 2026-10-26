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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引。步骤：计数脚本、三个小模型、用例生成、真 RefFS 逐条判、两边判相同、与 $MODEL/out/ 逐行比。
rerun_steps() {
  local capped="$REPO_ROOT/scripts/hosts/capped.sh"
  local out="$WORK/out"
  mkdir -p "$out"
  local format_lib="$FROZEN/crates/singlefs-format/src/lib.rs"
  number_of() { grep -oE "$1: u64 = [0-9]+" "$format_lib" | grep -oE '[0-9]+$' | head -1; }
  local unit_bytes payload_offset record_bytes factor ring_bytes
  unit_bytes="$(number_of 'pub const DATA_UNIT_BYTES')"
  payload_offset="$(number_of 'pub const DATA_UNIT_PAYLOAD_OFFSET')"
  record_bytes="$(number_of 'pub const JOURNAL_RECORD_BYTES')"
  factor="$(number_of 'pub const JOURNAL_SAFETY_FACTOR')"
  ring_bytes="$(grep -oE 'JOURNAL_RING_DEFAULT_BYTES = [0-9]+' "$REPO_ROOT/.claude/kb/layout/01-first-txn.md" | head -1 | grep -oE '[0-9]+$')"
  echo "name=constants data_unit_bytes=$unit_bytes payload_offset=$payload_offset record_bytes=$record_bytes factor=$factor default_ring_bytes=$ring_bytes"
  local -a flags=(--data-unit-bytes "$unit_bytes" --payload-bytes "$((unit_bytes - payload_offset))" --record-bytes "$record_bytes" --factor "$factor" --ring-bytes "$ring_bytes")

  bash "$MODEL/tools/count_assumptions.sh" "$FROZEN" > "$out/count_assumptions.txt"
  local sub
  for sub in window dirty firstcross; do
    python3 -I "$MODEL/model/trigger_model.py" "${flags[@]}" "$sub" > "$out/model_$sub.txt"
  done
  python3 -I "$MODEL/model/trigger_model.py" refs-cases "$out/cases.txt" > "$out/refs_cases_summary.txt"
  python3 -I "$MODEL/model/trigger_model.py" refs-py-verdicts "$out/cases.txt" > "$out/refs_python_verdicts.txt"
  sha256sum "$out/cases.txt" | cut -d' ' -f1 > "$out/cases.sha256"
  # 真 RefFS：只拷冻结副本里的这一个包（只依赖 std），把用例判一遍
  mkdir -p "$WORK/refs-copy/src" "$WORK/refs-copy/tests"
  cp "$FROZEN/crates/singlefs-reference-filesystem/Cargo.toml" "$WORK/refs-copy/"
  cp "$FROZEN/crates/singlefs-reference-filesystem/src/lib.rs" "$WORK/refs-copy/src/"
  cp "$MODEL/model/refs_cross.rs" "$WORK/refs-copy/tests/cross.rs"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-reference-filesystem 副本的测试目标 cross; 范围=1 个测试目标; 为什么=用真 RefFS 判正推腿生成的用例，核 Python 口径的允许集合' \
    REFS_CASES="$out/cases.txt" REFS_VERDICTS="$out/refs_real_verdicts.txt" CARGO_TARGET_DIR="$WORK/refs-target" \
    bash "$capped" default cargo test --manifest-path "$WORK/refs-copy/Cargo.toml" --test cross > "$out/refs_cargo.txt" 2>&1
  rm -rf "$WORK/refs-target" "$WORK/refs-copy"
  # 自证：把 Python 口径弄坏（发布不抬 lo）后，它与真 RefFS 的判必须对不上，否则上面那一格「相同」什么都没证明
  TRIGGER_MODEL_BREAK=publish-does-not-raise-lo python3 -I "$MODEL/model/trigger_model.py" refs-py-verdicts "$out/cases.txt" > "$out/refs_python_verdicts_broken.txt"
  if cmp -s "$out/refs_real_verdicts.txt" "$out/refs_python_verdicts_broken.txt"; then
    echo "name=refs_selftest broken_python_still_identical=1 (自证失败)" >&2
    return 1
  fi
  echo "name=refs_selftest broken_python_differs_from_real=$(paste -d"|" "$out/refs_real_verdicts.txt" "$out/refs_python_verdicts_broken.txt" | awk -F"|" "\$1!=\$2" | wc -l) of $(wc -l < "$out/cases.txt")"
  if cmp -s "$out/refs_real_verdicts.txt" "$out/refs_python_verdicts.txt"; then
    echo "name=refs_real_vs_python lines=$(wc -l < "$out/refs_real_verdicts.txt") identical=1"
  else
    echo "name=refs_real_vs_python identical=0" >&2
    return 1
  fi
  # 与模型目录里存的那一份逐行比（输出里不含路径与时刻的文件）
  local mismatches=0 file
  for file in count_assumptions.txt model_window.txt model_dirty.txt model_firstcross.txt refs_cases_summary.txt cases.sha256; do
    if diff -q "$out/$file" "$MODEL/out/$file" > /dev/null; then
      echo "name=same_as_stored file=$file"
    else
      echo "name=differs_from_stored file=$file" >&2
      mismatches=$((mismatches + 1))
    fi
  done
  echo "name=rerun_summary mismatches=$mismatches"
  [[ "$mismatches" -eq 0 ]]
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
