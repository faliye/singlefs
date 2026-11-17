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
# 甲臂（不打补丁）与乙臂（打 patches/ 里那一份补丁）各从冻结副本经 minimal-copy.py 拷一份副本，先核六份被判产品文件与开工快照的 sha256；
# 乙臂打补丁之后再核打完的 mounted_session.rs 与 patches/patched-sha256.txt 相等。快照或补丁核不过的那一臂不编不跑（免得把没打上的代码当乙臂读）。
# 两份副本里放进同一份探针当一个新测试目标，只跑它。观测行（m4-close-batch2-r2-ab: 开头）前面加上 arm=<臂> 落进 $WORK/observations.txt。
# 探针只断言前提，两臂的读数看观测行；最后两行按跑前写下的期望对一遍（name=m4_close_batch2_r2_ab_verdict、name=m4_close_batch2_r2_ab_mount_budget）。
# 服务撞内存上限重起同一条命令时 --work 要仍是空的（模板拒非空的 --work）：每次尝试写在 $WORK.attempts/attempt.* 里，跑完才把日志与观测拷进 $WORK。
run_one_ab_arm() {   # run_one_ab_arm <臂：without-the-patch 或 with-the-patch> <这一次尝试的目录>；三步都过退 0
  local arm="$1" run_dir="$2"
  local copy="$run_dir/copy-$arm"
  python3 "$REPO_ROOT/scripts/hosts/minimal-copy.py" "$FROZEN" "$copy" --with-test-inputs > "$run_dir/minimal-copy-$arm.log" 2>&1
  local snapshot_rc=0
  (cd "$copy" && sha256sum -c "$REPO_ROOT/research/prompts/m4-close-batch2-code-r2-snapshot/crates-sha256.txt") > "$run_dir/snapshot-check-$arm.txt" 2>&1 || snapshot_rc=$?
  cat "$run_dir/snapshot-check-$arm.txt"
  echo "name=m4_close_batch2_r2_ab_snapshot arm=$arm rc=$snapshot_rc expected=0"
  local patch_rc=0
  if [[ "$arm" == with-the-patch && $snapshot_rc -eq 0 ]]; then
    (cd "$copy" && patch -p1 --forward --no-backup-if-mismatch < "$MODEL/patches/stopped-drop-reads-the-mount-budget-and-judges-the-gate.patch" && sha256sum -c "$MODEL/patches/patched-sha256.txt") > "$run_dir/patch-$arm.txt" 2>&1 || patch_rc=$?
    cat "$run_dir/patch-$arm.txt"
    echo "name=m4_close_batch2_r2_ab_patch arm=$arm rc=$patch_rc expected=0"
  fi
  if [[ $snapshot_rc -ne 0 || $patch_rc -ne 0 ]]; then
    echo "name=m4_close_batch2_r2_ab_probes arm=$arm rc=skipped expected=0"
    rm -rf "$copy"
    return 1
  fi
  cp "$MODEL/probes/attack_ab_stopped_drop_judges_the_rollback_gate.rs" \
    "$copy/crates/singlefs-verification/tests/attack_ab_stopped_drop_judges_the_rollback_gate.rs"
  local probes_rc=0
  (cd "$copy" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_ab_stopped_drop_judges_the_rollback_gate（攻方 A/B 探针，副本里新加）; 范围=1 个测试目标; 为什么=判决第三节第 1 条零轮改法的两臂读数' cargo test -p singlefs-verification --test attack_ab_stopped_drop_judges_the_rollback_gate -- --nocapture) \
    > "$run_dir/probes-$arm.log" 2>&1 || probes_rc=$?
  grep -aF "m4-close-batch2-r2-ab:" "$run_dir/probes-$arm.log" | sed "s/^/arm=$arm /" >> "$run_dir/observations.txt" || true
  grep -aF "arm=$arm " "$run_dir/observations.txt" || true
  grep -aE "^test result:|^test .* \.\.\. |^error(\[E[0-9]+\])?:|panicked at" "$run_dir/probes-$arm.log" || true
  echo "name=m4_close_batch2_r2_ab_probes arm=$arm rc=$probes_rc expected=0"
  rm -rf "$copy"
  [[ $probes_rc -eq 0 ]]
}

ab_field() {   # ab_field <观测文件> <臂> <观测行名> <字段>：那一臂那一行里那个字段的值（数或 true/false），没有就空
  grep -aF "arm=$2 m4-close-batch2-r2-ab: $3 " "$1" | sed -n "s/.* $4=\([0-9a-z]*\).*/\1/p" | head -n 1 || true
}

rerun_steps() {
  local attempts="$WORK.attempts" run_dir arms_failed=0
  mkdir -p "$attempts"
  rm -rf "$attempts"/attempt.*/copy-*
  run_dir="$(mktemp -d "$attempts/attempt.XXXXXX")"
  : > "$run_dir/observations.txt"
  run_one_ab_arm without-the-patch "$run_dir" || arms_failed=$((arms_failed + 1))
  run_one_ab_arm with-the-patch "$run_dir" || arms_failed=$((arms_failed + 1))
  cp "$run_dir"/probes-*.log "$run_dir"/snapshot-check-*.txt "$run_dir/observations.txt" "$WORK/" 2>/dev/null || true
  cp "$run_dir"/patch-*.txt "$WORK/" 2>/dev/null || true
  local observations="$run_dir/observations.txt" summary=stopped_drop_gate_ab_summary
  local judged_without published_without contrast_without
  local judged_with at_closed_with published_with holding_with left_with gone_after_held_with
  judged_without="$(ab_field "$observations" without-the-patch "$summary" geometries_judged)"
  published_without="$(ab_field "$observations" without-the-patch "$summary" first_wakes_publishing_past_a_closed_gate)"
  contrast_without="$(ab_field "$observations" without-the-patch "$summary" contrast_wakes_publishing)"
  judged_with="$(ab_field "$observations" with-the-patch "$summary" geometries_judged)"
  at_closed_with="$(ab_field "$observations" with-the-patch "$summary" first_wakes_at_a_closed_gate)"
  published_with="$(ab_field "$observations" with-the-patch "$summary" first_wakes_publishing_past_a_closed_gate)"
  holding_with="$(ab_field "$observations" with-the-patch "$summary" first_wakes_holding_the_intent_at_a_closed_gate)"
  left_with="$(ab_field "$observations" with-the-patch "$summary" histories_left_with_the_intent)"
  gone_after_held_with="$(ab_field "$observations" with-the-patch "$summary" histories_with_the_intent_gone_after_a_wake_held_at_the_gate)"
  local reproduces=no holds=no drops=no
  # 跑前写下的期望：甲臂三档都判到、门关着的头一次叫起每档至少一次越门发了、对照臂不发（照上一份复现）；
  # 乙臂三档都判到、门关着的头一次叫起一次都不发且意图留着；乙臂每段切换的历史建 inode 到门开都删掉了意图、至少一段是先被门挡住再删的。
  if [[ "$judged_without" == 3 && -n "$published_without" && "$published_without" -ge 3 && "$contrast_without" == 0 ]]; then reproduces=yes; fi
  if [[ "$judged_with" == 3 && -n "$at_closed_with" && "$at_closed_with" -ge 3 && "$published_with" == 0 && "$holding_with" == "$at_closed_with" ]]; then holds=yes; fi
  if [[ "$left_with" == 0 && -n "$gone_after_held_with" && "$gone_after_held_with" -ge 1 ]]; then drops=yes; fi
  echo "name=m4_close_batch2_r2_ab_verdict without_the_patch_reproduces=$reproduces with_the_patch_holds_at_a_closed_gate=$holds with_the_patch_drops_once_the_gate_opens=$drops arms_failed=$arms_failed"
  local mount_line=mount_budget_first_stopped_call
  echo "name=m4_close_batch2_r2_ab_mount_budget counted_at_the_mount_without_the_patch=$(ab_field "$observations" without-the-patch "$mount_line" publishes_counted_at_the_mount) after_the_first_call_without_the_patch=$(ab_field "$observations" without-the-patch "$mount_line" publishes_since_the_last_user_change_after_the_first_call) counted_at_the_mount_with_the_patch=$(ab_field "$observations" with-the-patch "$mount_line" publishes_counted_at_the_mount) after_the_first_call_with_the_patch=$(ab_field "$observations" with-the-patch "$mount_line" publishes_since_the_last_user_change_after_the_first_call)"
  ((arms_failed == 0))
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
