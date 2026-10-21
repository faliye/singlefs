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
  local threads="${RERUN_THREADS:-31}"
  local target="$WORK/target"
  mkdir -p "$WORK/tree" "$WORK/logs" "$WORK/out" "$WORK/bin" "$WORK/tree/.claude"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  cp -r "$REPO_ROOT/.cargo" "$WORK/tree/.cargo"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/tree/.claude/heavy-thread-default"
  cp "$MODEL"/probes/*.rs "$WORK/tree/crates/singlefs-harness/tests/"
  if [[ -n "${RERUN_TARGET_SEED:-}" ]]; then cp -r "$RERUN_TARGET_SEED" "$target"; fi
  find "$WORK/tree/crates" "$WORK/tree/Cargo.toml" -type f -exec touch {} +
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  # 被判代码上的唯一一处改动：只打印的跟踪行（整理一批的 planned / relocated / region_empty_after）。
  (cd "$WORK/tree" && patch -s crates/singlefs-core/src/mounted_session.rs "$MODEL/patches/defense-trace-zero-move-batches.diff")
  export CARGO_TARGET_DIR="$target" REPO_ROOT WORK
  build_the_probes "$threads" r2
  local probes="$WORK/bin/probes-r2" sweeps="$WORK/bin/sweeps-r2" tenth="$WORK/bin/t10-r2"
  [[ "${RERUN_STOP_AFTER:-}" == build ]] && { echo "name=rerun_stopped_after step=build"; return 0; }
  # 步 1：T4、F1、F1 小池、F5 各组（每组按场景下标分片并行）。
  bash "$MODEL/tools/probe_group.sh" "$probes" t4 3 t4 > "$WORK/logs/t4.summary" 2>&1
  if [[ "${RERUN_STOP_AFTER:-}" == t4 ]]; then
    normalized_probe_lines "$WORK"/logs/t4-[0-9]*.log > "$WORK/out/t4-r2-checker.txt"
    if cmp -s "$WORK/out/t4-r2-checker.txt" "$MODEL/out/t4-r2-checker.txt"; then echo "name=rerun_compare file=t4-r2-checker.txt identical=true"; else echo "name=rerun_compare file=t4-r2-checker.txt identical=false"; fi
    echo "name=rerun_stopped_after step=t4"; return 0
  fi
  bash "$MODEL/tools/probe_group.sh" "$probes" f1 25 f1 > "$WORK/logs/f1.summary" 2>&1
  bash "$MODEL/tools/probe_group.sh" "$probes" f1small 30 f1small > "$WORK/logs/f1small.summary" 2>&1
  bash "$MODEL/tools/probe_group.sh" "$probes" f5 6 f5 > "$WORK/logs/f5.summary" 2>&1
  normalized_probe_lines "$WORK"/logs/t4-[0-9]*.log > "$WORK/out/t4-r2-checker.txt"
  normalized_probe_lines "$WORK"/logs/f1-[0-9]*.log > "$WORK/out/f1-r2-core.txt"
  normalized_probe_lines "$WORK"/logs/f1small-[0-9]*.log > "$WORK/out/f1small-r2-core.txt"
  normalized_probe_lines "$WORK"/logs/f5-[0-9]*.log > "$WORK/out/f5-r2-core.txt"
  # 步 2：RefFS 翻译层、算术、T10。
  run_one_test "$probes" defense_probe_reference_filesystem_snapshot_sets | grep '^PROBE ' | LC_ALL=C sort > "$WORK/out/f6-reference-filesystem-snapshot-sets.txt"
  run_one_test "$probes" defense_probe_arithmetic | grep '^PROBE ' | LC_ALL=C sort > "$WORK/out/arithmetic.txt"
  run_one_test "$tenth" defense_t10_a_bad_logged_operations_entry_through_each_read_path | grep '^PROBE ' > "$WORK/out/t10.txt"
  # 步 3：四个固定种子（F2 的同形另一支、抬 F 上限）。
  bash "$MODEL/tools/seed_runs.sh" "$sweeps" > "$WORK/out/seeds.txt" 2>&1
  # 步 4：F1 变异（mount.rs 里那个函数改成返回空表）：同一组探针要红。
  (cd "$WORK/tree" && patch -s crates/singlefs-core/src/mount.rs "$MODEL/patches/mutation-f1-no-mapping-keys.diff")
  build_the_probes "$threads" f1-neutered
  (cd "$WORK/tree" && patch -s -R crates/singlefs-core/src/mount.rs "$MODEL/patches/mutation-f1-no-mapping-keys.diff")
  bash "$MODEL/tools/probe_group.sh" "$WORK/bin/probes-f1-neutered" f1 25 f1mut > "$WORK/logs/f1mut.summary" 2>&1
  normalized_probe_lines "$WORK"/logs/f1mut-[0-9]*.log > "$WORK/out/f1-r2-core-with-f1-neutered.txt"
  # 步 5：T11（变异表第 4346 行）：原样绿，按第 4346 行改坏红。
  find "$WORK/tree/crates" -type f -exec touch {} +
  local t11='model::tests::two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots'
  : > "$WORK/out/t11.txt"
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness --lib 里点名的一个单测; 范围=1 个测试; 为什么=复跑辩方腿 T11：原样绿、按变异表第 4346 行改坏红，只跑那一个测试' \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --offline -p singlefs-harness --lib -- "$t11" --exact 2>&1 | grep -E '^test |test result' | sed -E 's/finished in .*//' | sed 's/^/original: /' >> "$WORK/out/t11.txt")
  (cd "$WORK/tree" && patch -s crates/singlefs-harness/src/model.rs "$MODEL/patches/mutation-4346-snapshot-roots-ignored.diff")
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness --lib 里点名的一个单测; 范围=1 个测试; 为什么=复跑辩方腿 T11：原样绿、按变异表第 4346 行改坏红，只跑那一个测试' \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --offline -p singlefs-harness --lib -- "$t11" --exact 2>&1 | grep -E '^test |test result' | sed -E 's/finished in .*//' | sed 's/^/mutated: /' >> "$WORK/out/t11.txt" || true)
  (cd "$WORK/tree" && patch -s -R crates/singlefs-harness/src/model.rs "$MODEL/patches/mutation-4346-snapshot-roots-ignored.diff")
  # 比对：每份逐行相同才算同。
  local same=0 differ=0 name
  for name in t4-r2-checker.txt f1-r2-core.txt f1-r2-core-with-f1-neutered.txt f1small-r2-core.txt f5-r2-core.txt \
      f6-reference-filesystem-snapshot-sets.txt arithmetic.txt t10.txt seeds.txt t11.txt; do
    if cmp -s <(LC_ALL=C sort "$WORK/out/$name") <(LC_ALL=C sort "$MODEL/out/$name"); then
      echo "name=rerun_compare file=$name identical=true"; same=$((same + 1))
    else
      echo "name=rerun_compare file=$name identical=false"; differ=$((differ + 1))
    fi
  done
  echo "name=rerun_total same=$same differ=$differ"
}

# 在 $WORK/tree 上编三个探针二进制并拷进 $WORK/bin（$1 线程数，$2 标签）。
build_the_probes() {
  local threads="$1" label="$2" deps
  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的 3 个辩方探针目标（只编不跑）; 范围=3 个测试目标; 为什么=复跑辩方腿：在冻结副本的拷贝上编探针二进制，只编不跑' \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" "$threads" cargo test --offline --no-run -p singlefs-harness \
      --test defense_probes --test defense_random_sweeps --test defense_t10_bad_logged_operations_entry > "$WORK/logs/build-$label.log" 2>&1)
  deps="$CARGO_TARGET_DIR/debug/deps"
  cp "$(ls -t "$deps"/defense_probes-* | grep -v '\.d$' | head -1)" "$WORK/bin/probes-$label"
  cp "$(ls -t "$deps"/defense_random_sweeps-* | grep -v '\.d$' | head -1)" "$WORK/bin/sweeps-$label"
  cp "$(ls -t "$deps"/defense_t10_bad_logged_operations_entry-* | grep -v '\.d$' | head -1)" "$WORK/bin/t10-$label"
}

# 跑二进制里的一个 ignored 测试（整条经内存包装）。
run_one_test() {
  nice -n 19 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 4G "$1" --ignored --nocapture --exact "$2" 2>&1
}

normalized_probe_lines() {
  grep -h '^PROBE ' "$@" | sed -E 's/ first_reds=.*//' | LC_ALL=C sort
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
