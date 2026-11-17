#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条交 task-server 跑，工作目录随意；不套内存包装、不写线程：上限、线程与机器都由服务定）：
#   task-server/task submit -- bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 task-server/task（判是不是本仓）
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
#   RERUN_TEMPLATE_BREAK=root-check-by-wrapper  判是不是本仓看 scripts/hosts/run-with-memory-cap.sh 在不在（不看 task-server/task）
#       「✗ 自检：--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2」那一格红
#       连带红的格（自证里的假仓只有 task-server/task、没有内存包装，被当成不是本仓）：「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」、「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条交 task-server/task submit -- bash <这份脚本> … 跑" >&2
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
  local repository_marker="task-server/task"
  [[ "${RERUN_TEMPLATE_BREAK:-}" != root-check-by-wrapper ]] || repository_marker="scripts/hosts/run-with-memory-cap.sh"
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/$repository_marker" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 $repository_marker）"
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

# ── 这一腿要跑的步骤（device-library-write-many-code-r3 攻方腿）──
# 每个变体一棵各自的树（从冻结副本拷）、一个各自的编译目录；同一个变体的几趟用同一个编译目录（树相同，二进制相同）。
# 每棵树：判定库口子 lib.rs 的 mod tests 末尾接上 $MODEL/drafts/appended-tests.rs 的四条草稿测试，变体另换上一处原文（$MODEL/variants/<变体>/）。
# 输出（标准输出）：
#   name=round variant=<变体> round=<趟> rc=<cargo 退出码> compiled=<这一趟日志里 Compiling singlefs-device-library 的行数> result=<test result 那一行> failed=<红了的测试>
#   name=test variant=<变体> round=<趟> test=<关心的测试> outcome=<ok|FAILED|missing>
#   name=resident variant=<变体> round=<趟> <草稿测试打的 announced= growth= 一行，没打出写 missing>
#   name=variant_summary variant=<变体> rounds=<趟数> original_failed_rounds=<原有 21 条里有测试红的趟数> <关心的测试>=<红的趟数>/<趟数> …
WATCHED_TESTS="in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code read_answered_failed_exits_with_the_code_in_the_reply read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes read_answered_found_touches_no_more_memory_than_the_contents_that_arrived a_write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply"

run_variant() {   # run_variant <变体名> <变体目录或 -> <趟数>
  local slug="$1" variant="$2" rounds="$3"
  local tree="$WORK/$slug/tree" target="$WORK/$slug/target"
  mkdir -p "$tree" "$WORK/logs"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
  python3 "$MODEL/tools/apply_variant.py" "$tree" "$MODEL/drafts/appended-tests.rs" "$variant"
  local round original_failed_rounds=0 test
  declare -A red_rounds=()
  for test in $WATCHED_TESTS; do red_rounds[$test]=0; done
  for ((round = 1; round <= rounds; round++)); do
    local log="$WORK/logs/$slug-round-$round.log" rc=0
    (cd "$tree" && CARGO_TARGET_DIR="$target" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标（攻方腿草稿测试接进去的树，没改坏或换上一处变体）; 范围=1 个测试目标; 为什么=量现有 21 条测试抓不抓得到这几处变体、草稿测试抓不抓得到' cargo test -p singlefs-device-library --lib -- --show-output) > "$log" 2>&1 || rc=$?
    local result failed compiled original
    result="$(grep -E '^test result:' "$log" | tail -n 1 || true)"
    failed="$(grep -E '^test tests::[A-Za-z0-9_]+ \.\.\. FAILED$' "$log" | sed -E 's/^test tests::([A-Za-z0-9_]+) .*/\1/' | sort | paste -sd, - || true)"
    compiled="$(grep -c 'Compiling singlefs-device-library' "$log" || true)"
    echo "name=round variant=$slug round=$round rc=$rc compiled=$compiled result=${result:-none} failed=${failed:--}"
    original="$(grep -E '^test tests::[A-Za-z0-9_]+ \.\.\. FAILED$' "$log" | sed -E 's/^test tests::([A-Za-z0-9_]+) .*/\1/' \
      | grep -vxE 'in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code|read_answered_failed_exits_with_the_code_in_the_reply|read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes|read_answered_found_touches_no_more_memory_than_the_contents_that_arrived' || true)"
    if [[ -n "$original" || -z "$result" ]]; then original_failed_rounds=$((original_failed_rounds + 1)); fi
    for test in $WATCHED_TESTS; do
      local outcome
      if grep -qE "^test tests::$test \.\.\. FAILED$" "$log"; then
        outcome=FAILED; red_rounds[$test]=$((red_rounds[$test] + 1))
      elif grep -qE "^test tests::$test \.\.\. ok$" "$log"; then
        outcome=ok
      else
        outcome=missing
      fi
      echo "name=test variant=$slug round=$round test=$test outcome=$outcome"
    done
    local resident
    resident="$(grep -oE 'name=resident_growth_before_contents announced=[0-9]+ growth=[0-9]+' "$log" | head -n 1 || true)"
    echo "name=resident variant=$slug round=$round ${resident:-missing}"
  done
  local summary="name=variant_summary variant=$slug rounds=$rounds original_failed_rounds=$original_failed_rounds"
  for test in $WATCHED_TESTS; do summary="$summary $test=${red_rounds[$test]}/$rounds"; done
  echo "$summary"
  rm -rf "$tree" "$target"
}

rerun_steps() {
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  echo "name=frozen_library sha256=$(sha256sum "$FROZEN/crates/singlefs-device-library/src/lib.rs" | cut -d' ' -f1)"
  run_variant unmodified - 3
  run_variant write-key-checked-in-the-relay-arm-only "$MODEL/variants/write-key-checked-in-the-relay-arm-only" 3
  run_variant read-answered-failed-taken-as-absent "$MODEL/variants/read-answered-failed-taken-as-absent" 3
  run_variant contents-not-utf8-decoded-lossily "$MODEL/variants/contents-not-utf8-decoded-lossily" 3
  run_variant write-lock-released-between-header-and-contents "$MODEL/variants/write-lock-released-between-header-and-contents" 5
  run_variant read-contents-without-filling-zeros-first "$MODEL/variants/read-contents-without-filling-zeros-first" 3
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
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/task-server" "$scratch/old-repo/.git" "$scratch/old-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/task-server/task"
  : > "$scratch/old-repo/scripts/hosts/run-with-memory-cap.sh"
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
  if bash "$copy" --repo "$scratch/old-repo" --frozen "$scratch/frozen" --work "$scratch/work6" > "$scratch/run6.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run6.out"; then
    failures+=("--repo 指到只有内存包装、没有 task-server/task 的旧仓应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run6.out")")
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
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--repo 是没有 task-server/task 的旧仓、--work 非空都退 2 并说明"
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
