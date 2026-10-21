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
  # 第三轮辩方腿：前两轮四条腿的复跑 + 判决第二节引的数 + 脚本缺的几步。
  #   RERUN_ONLY  要跑哪几步（空格隔开，默认全跑）：legs numbers batches tests
  #   RERUN_FROZEN_R1  第一轮冻结副本（默认：--frozen 的上一级旁边的 m4-segment-code-r1/repo）；--frozen 给第二轮冻结副本
  #   RERUN_THREADS    线程数，默认 31（派发提示的线程上限）
  #   RERUN_TARGET_SEED_DEBUG  可选：已编过的 debug 编译目录，拷来当 tests 一步的编译目录种子（省首次全量编译）；batches 一步由第二轮攻方的 rerun.sh 在 $WORK/orig/tree 里 release 从零编
  # 四条腿各自的 rerun.sh 原样起（--work 各用 $WORK/legs/<腿>），它们自己与自己的 out/ 比；这里再比三样它们缺的：
  #   numbers 判决引的数对着腿交的 out 文件与冻结副本现算（tools/check-numbers.py），与 $MODEL/out/check-numbers.txt 逐行比
  #   batches 第二轮攻方「4 GiB with-rollback 310 段、3 段不合」：原样模型上重跑种子下标 [400,555) 与 [600,755)，与 $MODEL/out/with-rollback-310.txt 逐行比
  #   tests   第二轮辩方「必红与已知坏镜像 309 条全绿」：六个 harness 测试目标在第二轮冻结副本上重跑，与 $MODEL/out/f9-checker-tests.txt 逐行比；
  #           另从辩方腿的种子日志里抽出 950093 那一步的两个上限（9 与 11），与 $MODEL/out/seed950093-ceiling.txt 比（腿交的 out/seeds.txt 在 330 字处截断，没有这两个数）
  local only="${RERUN_ONLY:-legs numbers batches tests}" threads="${RERUN_THREADS:-31}"
  local prompts="$REPO_ROOT/research/prompts" cap="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" capped="$REPO_ROOT/scripts/hosts/capped.sh"
  local frozen_r2="$FROZEN" frozen_r1="${RERUN_FROZEN_R1:-$(dirname "$(dirname "$FROZEN")")/m4-segment-code-r1/repo}"
  local same=0 differ=0 leg name
  mkdir -p "$WORK/out" "$WORK/logs" "$WORK/legs"
  wants() { [[ " $only " == *" $1 "* ]]; }
  compare() {   # compare <名> <新文件> <模型目录里的基线>
    if cmp -s "$2" "$3"; then echo "name=rerun_compare file=$1 identical=true"; same=$((same + 1)); else echo "name=rerun_compare file=$1 identical=false"; differ=$((differ + 1)); fi
  }
  echo "name=rerun_roots repo=$REPO_ROOT frozen_r2=$frozen_r2 frozen_r1=$frozen_r1 model=$MODEL work=$WORK"
  if wants legs; then
    for leg in r1-sonnet r1-opus r2-sonnet r2-opus; do
      local leg_frozen="$frozen_r2"
      [[ $leg == r1-* ]] && leg_frozen="$frozen_r1"
      RERUN_THREADS="$threads" nice -n 19 bash "$cap" 8G bash "$prompts/m4-segment-code-$leg-model/tools/rerun.sh" \
        --repo "$REPO_ROOT" --frozen "$leg_frozen" --work "$WORK/legs/$leg" --model "$prompts/m4-segment-code-$leg-model" > "$WORK/logs/leg-$leg.log" 2>&1 || true
      echo "name=rerun_leg leg=$leg $(grep -E 'name=rerun_total|name=sonnet_forward_rerun_matches_expected' "$WORK/logs/leg-$leg.log" | tail -n 1)"
    done
    { grep -h 'Some(9)（D16' "$WORK/legs/r2-sonnet/logs/seed-rollback-950093.log" | grep -o '上限 Some([0-9]*)（D16（发布语义） 已定项 1）' | head -n 1
      grep -h '实现报上限' "$WORK/legs/r2-sonnet/logs/seed-rollback-950093.log" | grep -o '实现报上限 [0-9]*' | head -n 1; } > "$WORK/out/seed950093-ceiling.txt" || true
    compare seed950093-ceiling.txt "$WORK/out/seed950093-ceiling.txt" "$MODEL/out/seed950093-ceiling.txt"
  fi
  if wants numbers; then
    python3 -I "$MODEL/tools/check-numbers.py" --prompts "$prompts" --frozen-r1 "$frozen_r1" --frozen-r2 "$frozen_r2" --repo "$REPO_ROOT" > "$WORK/out/check-numbers.txt"
    compare check-numbers.txt "$WORK/out/check-numbers.txt" "$MODEL/out/check-numbers.txt"
  fi
  if wants batches; then
    # 原样模型的树：第二轮攻方的 rerun.sh 只做拷贝、打补丁、编译（RERUN_ONLY=prepare build），不跑模型那一步；随机历史由 random-batch.sh 分两段各 31 件跑。
    RERUN_THREADS="$threads" RERUN_ONLY="prepare build" nice -n 19 bash "$cap" 8G bash "$prompts/m4-segment-code-r2-opus-model/tools/rerun.sh" \
      --repo "$REPO_ROOT" --frozen "$frozen_r2" --work "$WORK/orig" --model "$prompts/m4-segment-code-r2-opus-model" > "$WORK/logs/orig-build.log" 2>&1
    bash "$MODEL/tools/random-batch.sh" "$REPO_ROOT" "$WORK/orig/tree" "$WORK/logs/batch-a" a with-rollback 400 31 5 > "$WORK/logs/batch-a.out" 2>&1 || true
    bash "$MODEL/tools/random-batch.sh" "$REPO_ROOT" "$WORK/orig/tree" "$WORK/logs/batch-b" b with-rollback 600 31 5 > "$WORK/logs/batch-b.out" 2>&1 || true
    python3 -I - "$WORK/logs" > "$WORK/out/with-rollback-310.txt" <<'PY'
import glob, re, sys
mine = {}
for path in glob.glob(sys.argv[1] + "/batch-?/b*.log"):
    for line in open(path, errors="replace"):
        if line.startswith("ATTACK_SUMMARY"):
            d = dict(re.findall(r"(\w+)=(\S+)", line))
            mine[int(d["seed_start"])] = (int(d["seeds"]), int(d["steps_checked"]), int(d["problems"]))
for key in sorted(mine):
    print("seed_start=%d seeds=%d steps_checked=%d problems=%d" % ((key,) + mine[key]))
print("total seeds=%d steps_checked=%d problems=%d" % (sum(v[0] for v in mine.values()), sum(v[1] for v in mine.values()), sum(v[2] for v in mine.values())))
PY
    compare with-rollback-310.txt "$WORK/out/with-rollback-310.txt" "$MODEL/out/with-rollback-310.txt"
  fi
  if wants tests; then
    mkdir -p "$WORK/checker-tree/.claude"
    if [[ -n "${RERUN_TARGET_SEED_DEBUG:-}" ]]; then cp -r "$RERUN_TARGET_SEED_DEBUG" "$WORK/checker-target"; fi
    rsync -a --exclude target --exclude .git "$frozen_r2/" "$WORK/checker-tree/"
    cp -r "$REPO_ROOT/.cargo" "$WORK/checker-tree/.cargo"
    cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/checker-tree/.claude/heavy-thread-default"
    find "$WORK/checker-tree/crates" "$WORK/checker-tree/Cargo.toml" -type f -exec touch {} +
    (cd "$WORK/checker-tree" && CARGO_TARGET_DIR="$WORK/checker-target" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的 6 个池级 checker 测试目标; 范围=6 个测试目标; 为什么=第三轮辩方腿复跑第二轮辩方 F9 条（309 条必红与已知坏镜像测试）' \
      nice -n 19 bash "$cap" 8G bash "$capped" "$threads" cargo test --offline -p singlefs-harness \
      --test checker_compaction_intent_and_relocation_must_red_worlds --test checker_judges_reference_intervals_and_every_new_root_segment_field \
      --test checker_known_bad_images --test checker_snapshot_space_accounting_must_red_worlds \
      --test pool_checker_judges_the_snapshot_entries_of_the_tree_table \
      --test pool_checker_shared_subtree_below_a_snapshot_only_unit_keeps_the_children_it_shares_with_the_head) > "$WORK/logs/checker-tests.log" 2>&1 || true
    grep -E '^running|test result' "$WORK/logs/checker-tests.log" | sed -E 's/; finished in .*//' > "$WORK/out/f9-checker-tests.txt"
    compare f9-checker-tests.txt "$WORK/out/f9-checker-tests.txt" "$MODEL/out/f9-checker-tests.txt"
    # 第二轮辩方报告里「既有用例绿」那几条：F2 删意图、F3 的 segs0、F4 强制开关、合取 7 的默认拒绝。
    (cd "$WORK/checker-tree" && CARGO_TARGET_DIR="$WORK/checker-target" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的 4 个测试目标（F2、F3、F4、合取 7 的既有用例）; 范围=4 个测试目标; 为什么=第三轮辩方腿复跑第二轮辩方报告里「既有用例绿」那几条' \
      nice -n 19 bash "$cap" 8G bash "$capped" "$threads" cargo test --offline -p singlefs-harness \
      --test compaction_intent_issue_writes_the_empty_segments_of_its_own_accounting --test compaction_batch_emptying_the_region_deletes_the_intent_in_the_same_publish \
      --test snapshot_lifetime_judgement_forced_onto_either_arm_by_the_test_only_switch --test commit_generated_units_fall_back_when_the_segment_runs_out) > "$WORK/logs/existing-fix-tests.log" 2>&1 || true
    grep -E '^running|test result' "$WORK/logs/existing-fix-tests.log" | sed -E 's/; finished in .*//' > "$WORK/out/existing-fix-tests.txt"
    compare existing-fix-tests.txt "$WORK/out/existing-fix-tests.txt" "$MODEL/out/existing-fix-tests.txt"
  fi
  echo "name=rerun_total same=$same differ=$differ"
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
