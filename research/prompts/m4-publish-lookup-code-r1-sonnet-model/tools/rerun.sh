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
  mkdir -p "$WORK/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  if [[ ! -f "$FROZEN/crates/singlefs-core/Cargo.toml" ]]; then
    echo "name=rerun_skipped reason=冻结副本里没有 crates/singlefs-core（自证用的假冻结副本）"
    return 0
  fi
  # RERUN_STEPS 点名跑哪几步（空格隔）；默认只跑确定性的两步。growth-* 每步 100 到 220 秒、数带负载噪声，要跑就点名。
  local steps="${RERUN_STEPS:-digests-new digests-old}"
  local step
  for step in $steps; do
    case "$step" in
      digests-new) step_digests new "$WORK/tree" ;;
      digests-old) prepare_old_tree; step_digests old "$WORK/tree-old" ;;
      growth-new) step_growth new "$WORK/tree" ;;
      growth-old) prepare_old_tree; step_growth old "$WORK/tree-old" ;;
      growth-m9) prepare_mutated_tree "增长：整理读版本退回逐 key 查映射"; step_growth m9 "$WORK/tree-mutated" ;;
      growth-m8) prepare_mutated_tree "增长：按身份查的索引退回逐项找"; step_growth m8 "$WORK/tree-mutated" ;;
      *) echo "name=rerun_unknown_step step=$step" >&2; return 2 ;;
    esac
  done
  echo "name=rerun_expected digests_16=87addacdc948bbbc00b1e0f2894f5ffe45f1e93a24f7645bd4092edcb3f2bf75 digests_48=3344bce43504554dd5e0ca94c286b1a3b70f9ac362a374898760405876761b19 growth_percent_seen_in_the_report=new:310,287 old:379,377 m9:359,357 m8:609 (这几个数带负载噪声，逐个比不逐字)"
}

# 改前的发布路径：冻结副本的核心层，调用点还原成改前的写法（反向打 diff/callsites.diff），mounted_session.rs 换回改前的 git blob；新接口留着，测试文件原样能编
prepare_old_tree() {
  [[ -d "$WORK/tree-old" ]] && return 0
  mkdir -p "$WORK/tree-old"
  rsync -a --exclude target --exclude .git --exclude research "$WORK/tree/" "$WORK/tree-old/"
  git -C "$REPO_ROOT" cat-file -p 75fd509 > "$WORK/tree-old/crates/singlefs-core/src/mounted_session.rs"
  (cd "$WORK/tree-old/crates/singlefs-core/src" && patch -R --no-backup-if-mismatch transaction.rs < "$MODEL/diff/callsites.diff")
}

prepare_mutated_tree() {
  rm -rf "$WORK/tree-mutated"
  mkdir -p "$WORK/tree-mutated"
  rsync -a --exclude target --exclude .git --exclude research "$WORK/tree/" "$WORK/tree-mutated/"
  python3 -I "$MODEL/tools/apply_mutation.py" "$WORK/tree-mutated" "七·6 $1"
}

build_test_binary() {   # build_test_binary <树> <编译目录>：只编不跑，交回测试二进制的路径（打在最后一行）
  local tree="$1" target="$2"
  find "$tree/crates" -type f -exec touch {} +
  (cd "$tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity（只编不跑）; 范围=1 个测试目标; 为什么=复跑三方正推腿的量：在自己的副本里编出测试二进制再取镜像摘要与增长涨幅' \
    CARGO_TARGET_DIR="$target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" default \
    bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G cargo test --offline -p singlefs-harness --test publish_lookup_by_identity --no-run) > "$WORK/build-$(basename "$tree").log" 2>&1
  ls -t "$target"/debug/deps/publish_lookup_by_identity-* | grep -v '\.d$' | head -1
}

step_digests() {   # step_digests <new|old> <树>
  local label="$1" tree="$2" binary
  binary="$(build_test_binary "$tree" "$WORK/target-$label")"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity 里十六个四单元与四十八个八单元两个等价用例; 范围=2 个测试; 为什么=复跑三方正推腿取改前与改后写出的镜像摘要，两档逐个比' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" default \
    bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 3G "$binary" sixteen_files_of_four forty_eight_files_of_eight --nocapture > "$WORK/run-digests-$label.log" 2>&1 || true
  grep -E 'images_sha256' "$WORK/run-digests-$label.log" | sed -E "s/^publish_lookup_by_identity shape=PoolOfFiles \{ files: ([0-9]+), .*images_sha256=([0-9a-f]+)$/name=digest variant=$label files=\1 sha256=\2/"
}

step_growth() {   # step_growth <名> <树>
  local label="$1" tree="$2" binary
  binary="$(build_test_binary "$tree" "$WORK/target-$label")"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity 里增长慢档用例; 范围=1 个测试; 为什么=复跑三方正推腿量一批整理线程 CPU 从 512 到 2048 个数据单元涨了百分之几' \
    nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" default \
    bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 6G "$binary" a_compaction_batch_cpu_grows --include-ignored --nocapture > "$WORK/run-growth-$label.log" 2>&1 || true
  grep -E 'growth_in_percent' "$WORK/run-growth-$label.log" | sed -E "s/^.*growth_in_percent=([0-9]+)$/name=growth variant=$label percent=\1/"
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
