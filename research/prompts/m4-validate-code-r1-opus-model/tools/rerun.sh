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

# ── 这一腿要跑的步骤：m4-validate-code-r1 攻方腿 ──
# 冻结副本拷进 $WORK/tree，原型（harness 测试目标）放进 crates/singlefs-harness/tests/，按组经 cargo test 跑；
# 每组只留 ATTACK- 开头的行与分配失败那一行（排序），末尾一行 exit=<cargo 退出码>，写 $WORK/out/<组>.txt，再与 $MODEL/out/<组>.txt 逐行比。
# ATTACK_RERUN_GROUPS 选组（默认全部）；ATTACK_RERUN_TARGET_DIR 给一个已有的编译目录可省一次全量编译（默认 $WORK/target）。
attack_group_tests() {   # attack_group_tests <组>：打出这一组的 libtest 参数
  case "$1" in
    main) echo "--test-threads 4 --exact a_whole_file_hole_of_tag_zero_shrunk_by_the_session_does_not_panic a_whole_file_hole_of_tag_zero_kept_at_its_size_by_the_session_does_not_panic a_whole_file_hole_of_tag_zero_extended_by_the_session_does_not_panic a_whole_file_hole_of_tag_zero_overwritten_by_the_session_does_not_panic a_pool_without_incompat_bit_two_takes_a_truncation_and_still_mounts the_mounted_reader_reads_the_tail_of_a_one_tebibyte_hole_as_zeros a_one_tebibyte_hole_remounts_writable_and_shrinks_back" ;;
    cold-walk) echo "--exact the_cold_walk_of_a_one_tebibyte_hole_returns" ;;
    cold-recovery) echo "--exact the_cold_recovery_of_a_one_tebibyte_hole_returns" ;;
    reference-filesystem) echo "--exact the_reference_filesystem_truncates_to_one_tebibyte" ;;
    other-changes) echo "--test-threads 3 --exact a_whole_file_hole_of_tag_zero_unlinked_by_the_session_does_not_panic a_whole_file_hole_of_tag_zero_snapshotted_by_the_session_does_not_panic a_whole_file_hole_of_tag_zero_with_new_inodes_by_the_session_does_not_panic" ;;
    slots) echo "--exact the_truncation_intent_slots_outside_the_allocation_record_tree_across_geometries" ;;
    rollback-histories) echo "--exact rolling_back_to_a_truncated_version_and_extending_keeps_every_step_image_clean" ;;
    *) return 1 ;;
  esac
}

rerun_steps() {
  mkdir -p "$WORK/tree" "$WORK/out" "$WORK/tmp"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  find "$WORK/tree/crates" -type f -exec touch {} +
  cp "$MODEL/prototype/truncation_code_round_attack_cases.rs" "$WORK/tree/crates/singlefs-harness/tests/"
  eval "$(python3 "$REPO_ROOT/scripts/hosts/rocksdb-prebuilt.py" env)"
  export CARGO_TARGET_DIR="${ATTACK_RERUN_TARGET_DIR:-$WORK/target}"
  export TMPDIR="$WORK/tmp"
  local group arguments rc
  for group in ${ATTACK_RERUN_GROUPS:-main cold-walk cold-recovery reference-filesystem other-changes slots rollback-histories}; do
    arguments="$(attack_group_tests "$group")" || { echo "认不出组：$group" >&2; return 2; }
    rc=0
    # shellcheck disable=SC2086  # 参数按空白切成 libtest 的几个参数
    (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 truncation_code_round_attack_cases; 范围=1 个测试目标; 为什么=复跑 m4-validate-code-r1 攻方腿的标签 0、大洞、位 2、意图槽数与回退用例' nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test -p singlefs-harness --test truncation_code_round_attack_cases -- $arguments --nocapture) > "$WORK/out/raw-$group.txt" 2>&1 || rc=$?
    { grep -E '^ATTACK-|^memory allocation of' "$WORK/out/raw-$group.txt" | sort; echo "exit=$rc"; } > "$WORK/out/$group.txt"
    if [[ -f "$MODEL/out/$group.txt" ]]; then
      if diff -q "$MODEL/out/$group.txt" "$WORK/out/$group.txt" > /dev/null; then
        echo "name=$group same_as_model_out=yes exit=$rc"
      else
        echo "name=$group same_as_model_out=no exit=$rc（diff $MODEL/out/$group.txt $WORK/out/$group.txt）"
      fi
    else
      echo "name=$group same_as_model_out=absent exit=$rc（模型目录里还没有 out/$group.txt）"
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
