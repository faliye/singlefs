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
  # 步骤可以用 RERUN_ONLY=<逗号隔开的步骤名> 只跑几步（check,baseline,rows,extra,probe,equal），默认全跑。
  local only="${RERUN_ONLY:-check,baseline,rows,extra,probe,equal}"
  local diff_file="$REPO_ROOT/research/prompts/_m4-publish-lookup-followup-code-r1-diff.md"
  local cap="$REPO_ROOT/scripts/hosts/capped.sh" wrap="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" twologs="$REPO_ROOT/scripts/hosts/two-logs.sh"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  copy_source_tree() {   # copy_source_tree <目标>：只拷要编的（crates、Cargo.toml、Cargo.lock、.cargo、heavy-thread-default），拷完 touch
    mkdir -p "$1/.claude"
    cp -r "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$1/"
    cp "$FROZEN/.claude/heavy-thread-default" "$1/.claude/"
    find "$1/crates" -type f -exec touch {} +
  }
  cargo_test() {   # cargo_test <树> <编译目录> <日志名> <声明的范围> <cargo test 后面的参数…>
    local tree="$1" target="$2" log="$3" scope="$4"; shift 4
    (cd "$tree" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity; 范围=$scope; 为什么=三方正推腿复跑自己的读数与变异，只点名要的用例" \
      nice -n 19 env CARGO_TARGET_DIR="$target" bash "$twologs" --quiet "$WORK/$log" -- bash "$cap" 31 bash "$wrap" 16G \
      cargo test -p singlefs-harness --test publish_lookup_by_identity --offline -- "$@") || true
  }
  if [[ ",$only," == *,check,* ]]; then
    python3 -I "$MODEL/tools/check_mutation_rows.py" "$FROZEN" "$WORK/before" 2>/dev/null || python3 -I "$MODEL/tools/check_mutation_rows.py" "$FROZEN"
  fi
  copy_source_tree "$WORK/after"
  if [[ ",$only," == *,baseline,* ]]; then
    cargo_test "$WORK/after" "$WORK/target-after" run-baseline "5 个测试" --exact \
      a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site \
      a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once \
      relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice \
      first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table \
      lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search
    grep -E '^test |test result' "$WORK/run-baseline.log" | sed 's/^/name=baseline /'
  fi
  if [[ ",$only," == *,rows,* ]]; then
    copy_source_tree "$WORK/mut"
    (cd "$WORK/mut" && python3 -I "$MODEL/tools/run_mutants.py" --repo "$REPO_ROOT" --tree "$WORK/mut" --target "$WORK/target-mut" \
      --tsv "$WORK/mut/crates/mutations.tsv" --out "$WORK/rows-19.tsv" --prefix '七·6 第三轮' --baseline)
    awk -F'\t' '{printf "name=row idx=%s outcome=%s\n", $1, $2}' "$WORK/rows-19.tsv"
  fi
  if [[ ",$only," == *,extra,* ]]; then
    [[ -d "$WORK/mut" ]] || copy_source_tree "$WORK/mut"
    (cd "$WORK/mut" && python3 -I "$MODEL/tools/run_mutants.py" --repo "$REPO_ROOT" --tree "$WORK/mut" --target "$WORK/target-mut" \
      --tsv "$MODEL/mutants/extra.tsv" --out "$WORK/rows-extra.tsv" --prefix '正推')
    awk -F'\t' '{printf "name=extra idx=%s outcome=%s result=%s\n", $1, $2, $4}' "$WORK/rows-extra.tsv"
  fi
  if [[ ",$only," == *,probe,* ]]; then
    copy_source_tree "$WORK/inst"
    python3 -I "$MODEL/tools/instrument.py" --tree "$WORK/inst"
    cargo_test "$WORK/inst" "$WORK/target-inst" run-probe "2 个测试" --exact \
      probe_linear_finds_on_the_publish_path_by_file_size probe_linear_finds_on_a_relocation_batch_by_pool_size --nocapture --test-threads=1
    grep -E '^PROBE' "$WORK/run-probe.log" | sed 's/^/name=probe /'
  fi
  if [[ ",$only," == *,equal,* ]]; then
    copy_source_tree "$WORK/before"
    (cd "$WORK/before" && sed -n 6,1245p "$diff_file" > "$WORK/followup.patch" && patch -R -p1 < "$WORK/followup.patch" > "$WORK/patch-R.out")
    local equal_tests=(every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units
      every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units)
    cargo_test "$WORK/after" "$WORK/target-after" run-equal-after "2 个测试" --exact "${equal_tests[@]}" --nocapture
    cargo_test "$WORK/before" "$WORK/target-before" run-equal-before "2 个测试" --exact "${equal_tests[@]}" --nocapture
    grep -h 'images_sha256' "$WORK/run-equal-after.log" | sed 's/^/name=equal side=after /'
    grep -h 'images_sha256' "$WORK/run-equal-before.log" | sed 's/^/name=equal side=before /'
  fi
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
