#!/usr/bin/env bash
# m4-publish-lookup-code-r3 攻方腿的复跑脚本（拷自 scripts/agents/three-way-rerun-template.sh，只改了 rerun_steps 那一段）。
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
rerun_measurements() {
  local capped="$REPO_ROOT/scripts/hosts/capped.sh" cap="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" two_logs="$REPO_ROOT/scripts/hosts/two-logs.sh"
  local tree binary
  local -a crash_entries=(singlefs_checker_tier::crash::judge_crash_image
    singlefs_checker_tier::crash_judge_conclusions::StateCountsOfABlock::count_the_cpu_judgement
    singlefs_checker_tier::crash_judge_conclusions::StateCountsOfABlock::into_count_vector
    singlefs_checker_tier::crash_judge_conclusions::encoded_verdict_of_the_red_items)
  mkdir -p "$WORK/patch" "$WORK/mutruns"
  # 改后 = 冻结副本原样；改前 = 第三批 crates.patch 反向打回（补丁取自背景材料的改动全文）
  for tree in after before; do
    mkdir -p "$WORK/$tree/.claude"
    rsync -a --exclude target "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/$tree/"
    cp "$FROZEN/.claude/heavy-thread-default" "$WORK/$tree/.claude/"
  done
  awk '/^````diff$/{f=1;next} /^````$/{f=0} f' "$REPO_ROOT/research/prompts/_m4-publish-lookup-code-r3-diff.md" > "$WORK/patch/crates.patch"
  (cd "$WORK/before" && git apply -R "$WORK/patch/crates.patch")
  echo "name=trees after_transaction=$(sha256sum < "$WORK/after/crates/singlefs-core/src/transaction.rs" | cut -c1-16) before_transaction=$(sha256sum < "$WORK/before/crates/singlefs-core/src/transaction.rs" | cut -c1-16)"
  rsync -a "$WORK/after/" "$WORK/mut/"
  rsync -a "$WORK/after/" "$WORK/featured/"
  for tree in after before; do cp "$MODEL/tests/r3_opus_differential.rs" "$WORK/$tree/crates/singlefs-harness/tests/"; done
  cp "$MODEL/tests/r3_opus_coverage.rs" "$WORK/after/crates/singlefs-harness/tests/"
  for tree in after before mut featured; do (cd "$WORK/$tree" && find crates -type f -exec touch {} +); done
  # 一、改前改后差分：六组比重 × 两种盘宽 × 16 个种子，每段 40 步
  for tree in before after; do
    (cd "$WORK/$tree" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$two_logs" --quiet "$WORK/build-diff-$tree" -- bash "$capped" 31 bash "$cap" 16G cargo test -p singlefs-harness --test r3_opus_differential --no-run) || return 1
    binary="$(grep -o '/[^ ]*r3_opus_differential-[0-9a-f]*' "$WORK/build-diff-$tree.log" | head -n 1)"
    R3_DIFF_SEEDS=16 R3_DIFF_STEPS=40 nice -n 19 bash "$capped" 31 bash "$cap" 16G "$binary" --nocapture --exact r3_opus_before_and_after_histories > "$WORK/diff-$tree.txt" 2> "$WORK/diff-$tree.err" || return 1
  done
  if diff <(grep '^name=differential' "$WORK/diff-before.txt") <(grep '^name=differential' "$WORK/diff-after.txt") > "$WORK/diff.out"; then
    echo "name=differential_compare histories=$(grep -c '^name=differential' "$WORK/diff-after.txt") before_vs_after=identical endings=$(grep -o 'ending=[a-z_0-9]*' "$WORK/diff-after.txt" | sort | uniq -c | awk '{print $2 ":" $1}' | paste -sd, -)"
  else
    echo "name=differential_compare before_vs_after=different lines_differing=$(grep -c '^[<>]' "$WORK/diff.out")"
  fi
  # 二、差分那一组历史（每组 4 个种子）在改后那棵上的覆盖：各处按身份查了几次
  (cd "$WORK/after" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$two_logs" --quiet "$WORK/build-cov" -- bash "$capped" 31 bash "$cap" 16G cargo test -p singlefs-harness --test r3_opus_coverage --no-run) || return 1
  binary="$(grep -o '/[^ ]*r3_opus_coverage-[0-9a-f]*' "$WORK/build-cov.log" | head -n 1)"
  R3_DIFF_SEEDS=4 R3_DIFF_STEPS=40 nice -n 19 bash "$capped" 31 bash "$cap" 16G "$binary" --nocapture --exact r3_opus_coverage_of_the_differential_histories > "$WORK/cov.txt" 2> "$WORK/cov.err" || return 1
  python3 -I - "$WORK/cov.txt" <<'PY'
import collections, re, sys
total, histories = collections.Counter(), collections.Counter()
for line in open(sys.argv[1], encoding="utf-8"):
    if line.startswith("name=coverage"):
        for site, lookups, builds, decodes in re.findall(r"(\w+):lookups=(\d+),builds=(\d+),leaf_decodes=(\d+)", line):
            total[site, "lookups"] += int(lookups); total[site, "builds"] += int(builds); total[site, "leaf_decodes"] += int(decodes)
            histories[site] += int(lookups) > 0
for site in sorted(histories):
    print(f"name=coverage_sum site={site} histories_reaching_it={histories[site]} lookups={total[site, 'lookups']} builds={total[site, 'builds']} leaf_decodes={total[site, 'leaf_decodes']}")
PY
  # 三、改前那棵上两档等价用例打出的镜像摘要（与改后基线那一趟打出的比）
  (cd "$WORK/before" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity（改前拷贝）; 范围=1 个测试目标; 为什么=取改前两档等价用例打出的镜像摘要，与改后基线逐行比，核整理发批核源落点换查法前后写出同样镜像' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$two_logs" --quiet "$WORK/before-pli" -- bash "$capped" 31 bash "$cap" 16G cargo test -p singlefs-harness --test publish_lookup_by_identity -- --nocapture) || return 1
  grep -oE 'shape=PoolOfFiles \{ files: [0-9]+, data_units_per_file: [0-9]+ \} batches_published=[0-9]+ images_sha256=[0-9a-f]+' "$WORK/before-pli.log" | sed 's/^/name=images tree=before /'
  # 四、变异：基线、攻方腿自己的几条、第三批追加表里的 MA 作对照、只改测试的探针
  local mutant
  for mutant in baseline RA RB RC RD RE RF RG RG+MA MA PR; do
    bash "$MODEL/tools/run_mutant.sh" "$REPO_ROOT" "$WORK/after" "$WORK/mut" "$WORK/target" "$MODEL/mutants/mutants.tsv" "$mutant" "$WORK/mutruns" | grep -vE '^applied '
    if [[ "$mutant" == baseline ]]; then grep -oE 'shape=PoolOfFiles \{ files: [0-9]+, data_units_per_file: [0-9]+ \} batches_published=[0-9]+ images_sha256=[0-9a-f]+' "$WORK/mutruns/mutant-baseline.log" | sed 's/^/name=images tree=after /'; fi
  done
  # 五、checker 档判法单元键：单独编 checker 档与连 harness 测试一起编时 singlefs-core 开不开计数特性，键随不随它变
  (cd "$WORK/featured" && python3 -I - <<'PY'
path = "crates/singlefs-checker-tier/Cargo.toml"
text = open(path, encoding="utf-8").read()
old = 'singlefs-core = { path = "../singlefs-core" }\n'
assert text.count(old) == 1
open(path, "w", encoding="utf-8").write(text.replace(old, 'singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters"] }\n'))
PY
  )
  (cd "$WORK/after" && echo "name=cargo_tree checker_tier_alone_lookup_work_counters=$(cargo tree --offline -p singlefs-checker-tier -e features -i singlefs-core 2>/dev/null | grep -c 'lookup-work-counters') checker_tier_with_harness_lookup_work_counters=$(cargo tree --offline -p singlefs-checker-tier -p singlefs-harness -e features -i singlefs-core 2>/dev/null | grep -c 'lookup-work-counters') workspace_lookup_work_counters=$(cargo tree --offline --workspace -e features -i singlefs-core 2>/dev/null | grep -c 'lookup-work-counters')")
  (cd "$WORK/after" && CARGO_TARGET_DIR="$WORK/target-unit-graph" nice -n 19 bash "$capped" 31 cargo build -q -p singlefs-unit-graph --bin unit-graph) || return 1
  for tree in before after featured; do
    nice -n 19 bash "$capped" 31 bash "$cap" 8G "$WORK/target-unit-graph/debug/unit-graph" closure "$WORK/$tree" "${crash_entries[@]}" --target lib --no-default-features \
      --cfg 'feature="default"' --cfg 'feature="verdict-store"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --lock-file "$WORK/$tree/Cargo.lock" --threads 31 > "$WORK/unit-graph-$tree.txt" 2> "$WORK/unit-graph-$tree.err" || return 1
    echo "name=unit_key group=judgement_of_the_crash_images tree=$tree $(grep -E '^key' "$WORK/unit-graph-$tree.txt" | tr '\t' '=') units=$(grep -vc '^key' "$WORK/unit-graph-$tree.txt") lookup_work_units=$(grep -c 'lookup_work' "$WORK/unit-graph-$tree.txt")"
  done
}

rerun_steps() {
  rerun_measurements | tee "$WORK/rerun-output.txt"
  local expected="$MODEL/out/rerun-expected.txt" found=0 total=0 line
  [[ -f "$expected" ]] || { echo "name=rerun_compare no_expected_file"; return 0; }
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    total=$((total + 1))
    if grep -qxF -- "$line" "$WORK/rerun-output.txt"; then found=$((found + 1)); else echo "name=rerun_missing $line"; fi
  done < "$expected"
  echo "name=rerun_compare $found of $total lines of out/rerun-expected.txt found"
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
