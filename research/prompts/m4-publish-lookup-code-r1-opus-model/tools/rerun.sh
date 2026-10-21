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
  # 改后 = 冻结副本；改前 = 冻结副本拷一份再把七·6 补丁反向打上；两份各放进差分装置，release 编，跑一组小样本，逐行比。
  local capped="$REPO_ROOT/scripts/hosts/capped.sh" cap="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" copy test binary
  for copy in after before half grow9; do
    mkdir -p "$WORK/$copy/.claude"
    rsync -a --exclude target "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$WORK/$copy/"
    [[ -d "$FROZEN/.cargo" ]] && rsync -a "$FROZEN/.cargo" "$WORK/$copy/"
    cp "$FROZEN/.claude/heavy-thread-default" "$WORK/$copy/.claude/"
    cp "$MODEL/attack_differential.rs" "$WORK/$copy/crates/singlefs-harness/tests/m4_publish_lookup_attack_differential.rs"
    (cd "$WORK/$copy" && find crates -type f -exec touch {} +)
  done
  (cd "$WORK/before" && git apply -R "$MODEL/crates.patch")
  cp "$MODEL/attack_leaf_read.rs" "$WORK/after/crates/singlefs-harness/tests/m4_publish_lookup_attack_leaf_read.rs"
  python3 "$MODEL/tools/apply_once.py" "$WORK/half/crates/singlefs-core/src/mounted_session.rs" \
    '        .iter()\n        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {' \
    '        .iter()\n        .step_by(2)\n        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {'
  python3 "$MODEL/tools/apply_once.py" "$WORK/grow9/crates/singlefs-core/src/mounted_session.rs" \
    '        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {' \
    '        .filter_map(|(_, key)| match crate::transaction::central_mapping_locations_in_the_version(version, key) {'
  for copy in after before; do
    (cd "$WORK/$copy" && CARGO_TARGET_DIR="$WORK/target-$copy" bash "$capped" 31 cargo test --release --offline -p singlefs-harness \
      --test m4_publish_lookup_attack_differential $([[ $copy == after ]] && echo --test m4_publish_lookup_attack_leaf_read) --no-run)
  done
  for copy in half grow9; do
    (cd "$WORK/$copy" && CARGO_TARGET_DIR="$WORK/target-$copy" bash "$capped" 31 cargo test --offline -p singlefs-harness --test publish_lookup_by_identity --no-run)
  done
  run() {  # run <副本> <档> <测试目标> <输出> <参数…>
    local copy=$1 profile=$2 target=$3 out=$4; shift 4
    binary=$(ls "$WORK/target-$copy/$profile/deps/$target"-* | grep -v '\.d$' | head -1)
    (cd "$WORK/$copy/crates/singlefs-harness" && bash "$capped" 31 "$binary" "$@" > "$WORK/$out" 2>&1) || echo "name=rerun_step_exit out=$out rc=$?"
  }
  for copy in before after; do
    M4A_SEED_START=0 M4A_SEEDS=2 M4A_OPERATIONS=60 run "$copy" release m4_publish_lookup_attack_differential "random-$copy.log" --exact random_histories_print_digests --nocapture
    M4B_SHAPES=16x4,48x8 M4B_SEED_START=1 M4B_SEEDS=1 M4B_STEPS=40 run "$copy" release m4_publish_lookup_attack_differential "scripted-$copy.log" --exact scripted_histories_print_digests --nocapture
  done
  for kind in random scripted; do
    if diff <(grep '^M4' "$WORK/$kind-before.log") <(grep '^M4' "$WORK/$kind-after.log") > /dev/null; then same=identical; else same=DIFFERENT; fi
    echo "name=differential kind=$kind lines=$(grep -c '^M4' "$WORK/$kind-after.log") before_vs_after=$same"
  done
  run after release m4_publish_lookup_attack_leaf_read t2.log --nocapture
  echo "name=t2 $(grep '^M4T2 total' "$WORK/t2.log")"
  run half debug publish_lookup_by_identity half.log --nocapture
  echo "name=half_mapping_mutant $(grep -c '^test .* ok$' "$WORK/half.log") tests_ok; $(grep -o 'batches_published=[0-9]* images_sha256=[0-9a-f]*' "$WORK/half.log" | tr '\n' ' ')"
  run grow9 debug publish_lookup_by_identity grow9.log --include-ignored --exact a_compaction_batch_cpu_grows_less_than_the_threshold_from_512_to_2048_data_units --nocapture
  echo "name=growth_row_nine $(grep -o 'growth_in_percent=[0-9]*' "$WORK/grow9.log") $(grep '^test result' "$WORK/grow9.log")"
  echo "比：$MODEL/out/rerun-expected.txt"
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
