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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  # 1. 把冻结副本里要编的部分拷进草稿目录，打上主 agent 已有的 C860 镜像补丁（实现员 sha256 d5ac0e9a…）与本腿的读法开关补丁，再附上矩阵用例
  mkdir -p "$WORK/tree/.claude"
  rsync -a "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/tree/"
  cp "$FROZEN/.claude/heavy-thread-default" "$WORK/tree/.claude/"
  rm -rf "$WORK/tree/crates/"*/target
  ( cd "$WORK/tree" && git init -q . && git apply "$MODEL/patch/c860-writer-crates.patch" && git apply "$MODEL/patch/recognition-variants.patch" )
  cat "$MODEL/patch/matrix.rs" >> "$WORK/tree/crates/singlefs-harness/tests/scan_rebuild_reads_lineage_and_retirement_records.rs"
  sed -i 's/^mod common_pointer_head_corruption;$/mod common_pointer_head_corruption;\nmod common_snapshot_prefix;/' "$WORK/tree/crates/singlefs-harness/tests/scan_rebuild_reads_lineage_and_retirement_records.rs"
  export CARGO_TARGET_DIR="$WORK/target"
  cd "$WORK/tree"
  # 2. 编（只编这一个测试目标）
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 scan_rebuild_reads_lineage_and_retirement_records; 范围=1 个测试目标只编不跑; 为什么=复跑正推腿的矩阵先编副本里的读法开关' \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" default cargo test --offline --no-run -p singlefs-harness --test scan_rebuild_reads_lineage_and_retirement_records
  # 3. 五种读法各跑一遍世界矩阵与随机历史计数（SONNET_RECOGNITION 空 = 甲，今天的读法）
  for variant in jia yi bing ding ding2; do
    if [ "$variant" = jia ]; then export SONNET_RECOGNITION=; else export SONNET_RECOGNITION="$variant"; fi
    run_matrix > "$WORK/matrix-$variant.log" 2>&1
    run_random_histories > "$WORK/random-$variant.log" 2>&1
  done
  if [ "${SONNET_RERUN_REGRESS:-}" = 1 ]; then
    for variant in jia yi bing ding ding2; do
      if [ "$variant" = jia ]; then export SONNET_RECOGNITION=; else export SONNET_RECOGNITION="$variant"; fi
      for chunk in 1 2 3; do
        run_regression_chunk "$chunk" > "$WORK/regress-c$chunk-$variant.log" 2>&1 || true
      done
    done
  fi
  unset SONNET_RECOGNITION
  echo "name=rerun_matrix_lines $(cat "$WORK"/matrix-*.log | grep -c '^MATRIX')"
  grep -h 'SCAN_REBUILD_TALLY_SUMMARY' "$WORK"/random-*.log
}

run_matrix() {
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 scan_rebuild_reads_lineage_and_retirement_records 的矩阵用例; 范围=1 个测试; 为什么=复跑正推腿的世界矩阵，按环境变量选读法' \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G \
    cargo test --offline -p singlefs-harness --test scan_rebuild_reads_lineage_and_retirement_records -- --ignored --exact sonnet_matrix::matrix --nocapture
}

run_random_histories() {
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 scan_rebuild_reads_lineage_and_retirement_records 的随机历史慢档用例; 范围=1 个测试; 为什么=复跑正推腿的随机历史计数，按环境变量选读法' \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G \
    cargo test --offline -p singlefs-harness --test scan_rebuild_reads_lineage_and_retirement_records -- --ignored --exact the_scan_rebuild_is_never_violated_and_holds_after_deletions_over_random_histories --nocapture
}

# 已有用例回归（30 个调池级 checker 的测试目标，三段各 10 个）：SONNET_RERUN_REGRESS=1 时才跑，每段每种读法一次，报末尾的 test result
run_regression_chunk() {
  local chunk="$1"
  case "$chunk" in
    1) SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 里调池级 checker 的 10 个测试目标; 范围=10 个测试目标; 为什么=复跑正推腿的已有用例回归，第一段' \
         bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 12G \
         cargo test --offline --no-fail-fast -p singlefs-harness --test checker_compaction_intent_and_relocation_must_red_worlds --test checker_cross_links_malformed_nodes_and_mapping_entries --test checker_judges_reference_intervals_and_every_new_root_segment_field --test checker_judges_reserved_bytes_and_widths_it_used_to_skip --test checker_known_bad_images --test checker_narrow_invariants_and_abandoned_roots --test checker_relocated_images_judged_by_the_mapping_position --test checker_snapshot_space_accounting_must_red_worlds --test checker_truncation_field_must_red_worlds --test checker_truncation_holes_and_intent_must_red_worlds ;;
    2) SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 里调池级 checker 的 10 个测试目标; 范围=10 个测试目标; 为什么=复跑正推腿的已有用例回归，第二段' \
         bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 12G \
         cargo test --offline --no-fail-fast -p singlefs-harness --test compaction_relocation_batches_publish_through_the_mapping --test pool_checker_appearance_rule_labels_each_version_by_its_own_user_visible_trees --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window --test pool_checker_gap_row_conditions_and_incomplete_reference_pairs --test pool_checker_judges_the_deletion_state_of_every_walked_version --test pool_checker_judges_the_snapshot_entries_of_the_tree_table --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement --test pool_checker_reddens_only_the_tail_slot_invariant_when_every_system_configuration_slot_has_a_foreign_node_size --test pool_checker_scan_of_tree_identifiers_claims_whole_units --test pool_checker_shared_subtree_below_a_snapshot_only_unit_keeps_the_children_it_shares_with_the_head ;;
    3) SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 里调池级 checker 的 10 个测试目标; 范围=10 个测试目标; 为什么=复跑正推腿的已有用例回归，第三段' \
         bash "$REPO_ROOT/scripts/hosts/capped.sh" default bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 12G \
         cargo test --offline --no-fail-fast -p singlefs-harness --test pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance --test scan_rebuild_agrees_with_the_walked_index --test scan_rebuild_reads_lineage_and_retirement_records --test snapshot_and_compaction_over_a_file_other_than_the_first --test unlink_batches_of_an_inode_other_than_the_first_follow_its_own_data_units --test unlink_of_files_other_than_the_first --test writes_by_inode_number_after_rollbacks_past_the_deletion_of_the_first_file --test writes_by_inode_number_in_the_interleaved_histories_of_the_attack --test writes_by_inode_number_of_several_files ;;
  esac
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
