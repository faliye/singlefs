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

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 直接起（线程由 task-server 给整条套的 capped.sh <N> 定，这里不再套），
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  local library table rc round mutant parallel log
  mkdir -p "$WORK/tree" "$WORK/out"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  library="$WORK/tree/crates/singlefs-device-library/src/lib.rs"
  table="$WORK/tree/crates/mutations.tsv"
  sha256sum "$library" "$table" "$MODEL/attack_tests.rs" "$MODEL/relay_read_smuggle_model.py" "$MODEL/key_rules_cross_check.py" "$MODEL/tools/apply_once.py"
  # 同时跑几份探锁测试：服务给的每台默认重活线程数的三分之一（一份探锁测试三个线程，两个空转）
  parallel=$(( ${SINGLEFS_THREAD_CAP:?服务给的每台默认重活线程数没设} / 3 ))
  (( parallel >= 1 )) || parallel=1
  echo "name=probe_parallel thread_cap=$SINGLEFS_THREAD_CAP concurrent=$parallel"

  # 一、转交一侧（读仓根此刻的 device-code-key.py，输出头一行打它的 sha256）：read 的键里带换行
  rc=0
  python3 -I "$MODEL/relay_read_smuggle_model.py" --repo "$REPO_ROOT" > "$WORK/out/relay.out" 2> "$WORK/out/relay.err" || rc=$?
  echo "name=relay_model rc=$rc"
  cat "$WORK/out/relay.out"

  # 二、装置一侧：拷出来的树里加攻方测试，实现不改，整个 lib 目标跑五趟
  python3 -I "$MODEL/tools/apply_once.py" insert-tests "$library" "$MODEL/attack_tests.rs"
  cp "$library" "$WORK/out/lib-with-attack-tests.rs"
  for round in 1 2 3 4 5; do
    log="$WORK/out/tests-unmutated-$round.log"
    rc=0
    (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标（攻方腿拷出来的树，加了攻方测试，实现不改）; 范围=1 个测试目标; 为什么=基线：原有十六条与攻方加的几条在没改坏的实现上各是什么结局，跑五趟看稳不稳' cargo test -p singlefs-device-library --lib) > "$log" 2>&1 || rc=$?
    echo "name=tests_unmutated round=$round rc=$rc ok=$(grep -cE '^test .* \.\.\. ok$' "$log") failed=$(grep -cE '^test .* \.\.\. FAILED$' "$log")"
    grep -E '^test .* \.\.\. FAILED$|^test result|process didn.t exit successfully' "$log" || true
  done
  grep -E '^test ' "$WORK/out/tests-unmutated-1.log" || true
  sed -n '/^---- tests::read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code stdout ----/,/^$/p' "$WORK/out/tests-unmutated-1.log"

  # 三、两侧的相对键规则各跑各的再比
  rc=0
  (cd "$WORK/tree" && ATTACK_KEY_RULE_LISTING="$WORK/out/rust-key-rule.txt" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标里攻方加的一条 listing_of_what_the_key_rule_accepts; 范围=1 个测试; 为什么=让装置一侧的相对键规则自己跑出它收的全部单字符与边界键，交给转交一侧的正则逐个比' cargo test -p singlefs-device-library --lib -- tests::listing_of_what_the_key_rule_accepts --exact) > "$WORK/out/key-listing.log" 2>&1 || rc=$?
  echo "name=key_listing rc=$rc"
  rc=0
  python3 -I "$MODEL/key_rules_cross_check.py" --repo "$REPO_ROOT" --listing "$WORK/out/rust-key-rule.txt" > "$WORK/out/key-rules.out" 2> "$WORK/out/key-rules.err" || rc=$?
  echo "name=key_rules_cross_check rc=$rc"
  cat "$WORK/out/key-rules.out"

  # 四、探锁测试在没改坏的实现上：一份一份跑 40 次、同时几份跑 10 批（要 0 次红）
  for mutant in unmutated; do
    rc=0
    (cd "$WORK/tree" && ATTACK_PROBE_SAMPLES="1x40" ATTACK_PROBE_OUT="$WORK/out/probe-unmutated-sequential.txt" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标里攻方加的一条 catch_rate_of_the_read_lock_probe; 范围=1 个测试; 为什么=没改坏时多线程探锁测试一份一份连跑 40 次，数它误报红几次' cargo test -p singlefs-device-library --lib -- tests::catch_rate_of_the_read_lock_probe --exact) > "$WORK/out/probe-unmutated-sequential.log" 2>&1 || rc=$?
    echo "name=probe mutant=$mutant mode=sequential rc=$rc $(cat "$WORK/out/probe-unmutated-sequential.txt" 2>/dev/null || echo probe_samples=-)"
    rc=0
    (cd "$WORK/tree" && ATTACK_PROBE_SAMPLES="${parallel}x10" ATTACK_PROBE_OUT="$WORK/out/probe-unmutated-concurrent.txt" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标里攻方加的一条 catch_rate_of_the_read_lock_probe; 范围=1 个测试; 为什么=没改坏时多线程探锁测试同时几份、跑十批，数它在处理器被抢时误报红几次' cargo test -p singlefs-device-library --lib -- tests::catch_rate_of_the_read_lock_probe --exact) > "$WORK/out/probe-unmutated-concurrent.log" 2>&1 || rc=$?
    echo "name=probe mutant=$mutant mode=concurrent rc=$rc $(cat "$WORK/out/probe-unmutated-concurrent.txt" 2>/dev/null || echo probe_samples=-)"
  done

  # 五、变异表那一行「判定库口子 读的锁在请求与回应之间放过」原样施加（从拷出来的树里的 crates/mutations.tsv 取），量探锁测试抓到的机会
  cp "$WORK/out/lib-with-attack-tests.rs" "$library"
  python3 -I "$MODEL/tools/apply_once.py" replace-row "$library" "$table" '判定库口子 读的锁在请求与回应之间放过'
  rc=0
  (cd "$WORK/tree" && ATTACK_PROBE_SAMPLES="1x100" ATTACK_PROBE_OUT="$WORK/out/probe-mutant-sequential.txt" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标里攻方加的一条 catch_rate_of_the_read_lock_probe（施加了变异表「读的锁在请求与回应之间放过」那一行）; 范围=1 个测试; 为什么=变异下多线程探锁测试一份一份连跑 100 次，数它抓到几次，量抓到的机会' cargo test -p singlefs-device-library --lib -- tests::catch_rate_of_the_read_lock_probe --exact) > "$WORK/out/probe-mutant-sequential.log" 2>&1 || rc=$?
  echo "name=probe mutant=read-lock-released-between-request-and-reply mode=sequential rc=$rc $(cat "$WORK/out/probe-mutant-sequential.txt" 2>/dev/null || echo probe_samples=-)"
  rc=0
  (cd "$WORK/tree" && ATTACK_PROBE_SAMPLES="${parallel}x20" ATTACK_PROBE_OUT="$WORK/out/probe-mutant-concurrent.txt" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标里攻方加的一条 catch_rate_of_the_read_lock_probe（施加了变异表「读的锁在请求与回应之间放过」那一行）; 范围=1 个测试; 为什么=变异下多线程探锁测试同时几份、跑二十批，量处理器被抢时抓到的机会' cargo test -p singlefs-device-library --lib -- tests::catch_rate_of_the_read_lock_probe --exact) > "$WORK/out/probe-mutant-concurrent.log" 2>&1 || rc=$?
  echo "name=probe mutant=read-lock-released-between-request-and-reply mode=concurrent rc=$rc $(cat "$WORK/out/probe-mutant-concurrent.txt" 2>/dev/null || echo probe_samples=-)"
  # 照门禁 crates-mutation-replay 那一行的跑法（整个 lib 目标、十几条测试并行）连跑 30 趟，逐趟看点名的那条红没红
  for round in $(seq 1 30); do
    log="$WORK/out/replay-like-$round.log"
    rc=0
    (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标（施加了变异表「读的锁在请求与回应之间放过」那一行）; 范围=1 个测试目标; 为什么=照变异门禁那一行的跑法连跑 30 趟，逐趟看点名的多线程测试红没红，量一次跑抓到的机会' cargo test -p singlefs-device-library --lib) > "$log" 2>&1 || rc=$?
    echo "name=replay_like round=$round rc=$rc named=$(grep -oE '^test tests::a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply \.\.\. (ok|FAILED)$' "$log" | grep -oE '(ok|FAILED)$' || echo missing) caught_at=$(grep -oE 'left: Some\([0-9]+\)' "$log" | head -n 1 || true)"
  done
  echo "name=replay_like_summary runs=30 named_failed=$(cat "$WORK"/out/replay-like-*.log | grep -cE '^test tests::a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply \.\.\. FAILED$')"

  # 六、攻方加的三个变异：found 之后放锁再读内容、单条写与成批写在请求与回应之间放锁；整个 lib 目标各跑五趟，看有没有一条测试红
  for mutant in read-lock-released-before-found-contents write-lock-released-between-request-and-reply write-many-lock-released-between-request-and-reply; do
    cp "$WORK/out/lib-with-attack-tests.rs" "$library"
    python3 -I "$MODEL/tools/apply_once.py" replace "$library" "$MODEL/mutants/$mutant.anchor" "$MODEL/mutants/$mutant.replacement"
    for round in 1 2 3 4 5; do
      log="$WORK/out/tests-$mutant-$round.log"
      rc=0
      (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标（攻方腿拷出来的树，加了攻方测试，再换上攻方的一个放锁变异）; 范围=1 个测试目标; 为什么=看 found 内容那一段与两种写在请求与回应之间放锁时有没有一条测试红，跑五趟' cargo test -p singlefs-device-library --lib) > "$log" 2>&1 || rc=$?
      echo "name=tests_mutant mutant=$mutant round=$round rc=$rc ok=$(grep -cE '^test .* \.\.\. ok$' "$log") failed=$(grep -cE '^test .* \.\.\. FAILED$' "$log")"
      grep -E '^test .* \.\.\. FAILED$|^test result|process didn.t exit successfully|^error' "$log" || true
    done
  done
  cp "$WORK/out/lib-with-attack-tests.rs" "$library"
  echo "name=rerun_done out=$WORK/out"
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
