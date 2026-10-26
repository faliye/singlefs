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
  # 攻方腿 m4-publish-lookup-code-r2-opus：冻结副本拷成「改后」「变异」「改前」三份（改前 = crates.patch 反向打上）。
  # 变异段：逐条打 mutants.tsv 里的变异（MB 不在内：它让慢档用例跑十几分钟以上，报告里只引它停之前那份日志），
  #   debug 编 singlefs-harness 的两个测试目标、直接跑测试二进制（publish_lookup_by_identity 连 ignored 一起），每条的目标与过滤串照下表。
  # 差分段：改前、改后两份的快照测试文件末尾各接上 snapshot_differential_probe.rs，各跑一遍探针与两档等价用例，摘要行逐行比。
  # 打出的 name= 行去掉「finished in …」之后与 $MODEL/out/rerun-expected.txt 逐行比（最后一行报对上几行）。
  local copy name targets filter row
  for copy in after mut before; do
    mkdir -p "$WORK/$copy/.claude"
    rsync -a --exclude target "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/$copy/"
    cp "$FROZEN/.claude/heavy-thread-default" "$WORK/$copy/.claude/"
  done
  (cd "$WORK/before" && git apply -R "$MODEL/crates.patch")
  for copy in after mut before; do (cd "$WORK/$copy" && find crates -type f -exec touch {} +); done
  local sixteen='every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units --exact'
  local both='publish_lookup_by_identity snapshot_release_and_destroy_through_the_deadlist'
  {
    for row in "baseline|$both|" "MA-index-unit-linear|$both|" "MC-leaf-redecoded-per-key|publish_lookup_by_identity|" \
      "MD-mapping-key-of-linear|$both|" "ME-registered-reader-per-key|publish_lookup_by_identity|$sixteen" \
      "MF-fresh-reader-per-key-keeping-the-count|publish_lookup_by_identity|" \
      "MG-deadlist-loop-reverted-to-linear|snapshot_release_and_destroy_through_the_deadlist|" \
      "MH-deadlist-key-of-the-next-replaced-role|snapshot_release_and_destroy_through_the_deadlist|" \
      "MI-first-entry-of-walks-every-entry-without-comparing|publish_lookup_by_identity|" \
      "P1-probe-relocation-reads-the-mapping-key-by-key|publish_lookup_by_identity|$sixteen"; do
      IFS='|' read -r name targets filter <<< "$row"
      bash "$MODEL/tools/run_mutant.sh" "$REPO_ROOT" "$WORK/after" "$WORK/mut" "$WORK/target" "$MODEL/mutants.tsv" "$name" build "$WORK/logs" \
        | sed -E 's/ rc=0 .*/ rc=0/'
      RUN_TARGETS="$targets" RUN_FILTER="$filter" bash "$MODEL/tools/run_mutant.sh" "$REPO_ROOT" "$WORK/after" "$WORK/mut" "$WORK/target" \
        "$MODEL/mutants.tsv" "$name" run "$WORK/logs" | sed -E 's/; finished in [0-9.]+s//'
    done
    # 差分段：改后 = 变异副本恢复到冻结副本那两份产品文件（baseline），改前 = 反向补丁；两份的快照测试文件接上同一段探针。
    for copy in mut before; do
      cat "$MODEL/snapshot_differential_probe.rs" >> "$WORK/$copy/crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs"
    done
    bash "$MODEL/tools/run_mutant.sh" "$REPO_ROOT" "$WORK/after" "$WORK/mut" "$WORK/target" "$MODEL/mutants.tsv" baseline build "$WORK/logs-after" > /dev/null
    (cd "$WORK/before" && CARGO_TARGET_DIR="$WORK/target-before" nice -n 19 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 12G \
      bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test publish_lookup_by_identity \
      --test snapshot_release_and_destroy_through_the_deadlist --no-run) > "$WORK/before-build.log" 2>&1
    local side target_directory binary
    for side in after before; do
      if [[ $side == after ]]; then target_directory="$WORK/target"; copy=mut; else target_directory="$WORK/target-before"; copy=before; fi
      mkdir -p "$WORK/differential-$side"
      binary=$(ls -t "$target_directory"/debug/deps/snapshot_release_and_destroy_through_the_deadlist-* | grep -v '\.d$' | head -1)
      (cd "$WORK/$copy/crates/singlefs-harness" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 snapshot_release_and_destroy_through_the_deadlist; 范围=1 个测试目标; 为什么=攻方腿复跑：改前改后差分探针逐行比" \
        nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 12G "$binary" --nocapture attack_probe_snapshot_history_digests) \
        > "$WORK/differential-$side/probe.log" 2>&1
      binary=$(ls -t "$target_directory"/debug/deps/publish_lookup_by_identity-* | grep -v '\.d$' | head -1)
      (cd "$WORK/$copy/crates/singlefs-harness" && SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity; 范围=1 个测试目标; 为什么=攻方腿复跑：改前改后两档等价用例镜像摘要逐行比" \
        nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 12G "$binary" --nocapture every_lookup_by_identity_agrees) \
        > "$WORK/differential-$side/equivalence.log" 2>&1
      grep -h 'attack_probe\|images_sha256' "$WORK/differential-$side/probe.log" "$WORK/differential-$side/equivalence.log" | grep -v '^test ' > "$WORK/differential-$side/lines"
    done
    if diff "$WORK/differential-before/lines" "$WORK/differential-after/lines" > "$WORK/differential.diff"; then
      echo "name=differential before_vs_after=identical lines=$(wc -l < "$WORK/differential-after/lines")"
    else
      echo "name=differential before_vs_after=different lines=$(wc -l < "$WORK/differential-after/lines")"
    fi
  } | tee "$WORK/rerun.txt"
  echo "name=rerun_compare $(grep -cFxf "$MODEL/out/rerun-expected.txt" "$WORK/rerun.txt") of $(wc -l < "$WORK/rerun.txt") lines found in out/rerun-expected.txt"
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
