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
  # m3-close-sysconf-r3 攻方腿：冻结副本拷进草稿目录，打取号写两轮补丁与落后支各臂补丁，加探针 crate，几个大包在 release 下降到 opt-level 1；
  # 编探针（cargo build，经内存包装直接执行编出来的二进制）与 harness 档测试二进制（--no-run），跑探针各份与 harness 的红名单（两臂），逐份比 out/。
  # RERUN_HARNESS_ALL=1 时 harness 档 243 个目标在取号写两轮那一臂下全跑（慢，debug 下几个目标单个就超过 4 分钟）。
  mkdir -p "$WORK/tree/.claude" "$WORK/target/release/build" "$WORK/target/release/deps" "$WORK/target/release/.fingerprint"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/tree/"
  rsync -a "$REPO_ROOT/.cargo" "$REPO_ROOT/litmus" "$WORK/tree/"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/tree/.claude/"
  (cd "$WORK/tree" && patch -p1 --quiet < "$MODEL/two-rounds.patch")
  python3 "$MODEL/tools/patch_behind_arms.py" "$WORK/tree/crates/singlefs-core/src/mount.rs"
  mkdir -p "$WORK/tree/crates/opus-r3-probe/src"
  cp "$MODEL/probe-crate/Cargo.toml" "$WORK/tree/crates/opus-r3-probe/"
  cp "$MODEL"/probe-crate/src/*.rs "$WORK/tree/crates/opus-r3-probe/src/"
  python3 - "$WORK/tree/Cargo.toml" <<'ADD_MEMBER'
import sys
path = sys.argv[1]
text = open(path).read()
anchor = '    "crates/singlefs-unit-graph",\n]'
assert text.count(anchor) == 1, "工作区成员表的写法变了"
open(path, "w").write(text.replace(anchor, '    "crates/singlefs-unit-graph",\n    "crates/opus-r3-probe",\n]'))
ADD_MEMBER
  { echo; cat "$MODEL/release-profile-overrides.toml"; } >> "$WORK/tree/Cargo.toml"
  if [[ -n "${PROBE_SEED_TARGET:-}" ]]; then
    local entry kind
    for kind in build .fingerprint; do
      (cd "$PROBE_SEED_TARGET/release/$kind" && ls | grep -v '^singlefs' | grep -v '^opus' | while read -r entry; do cp -a "$entry" "$WORK/target/release/$kind/"; done)
    done
    (cd "$PROBE_SEED_TARGET/release/deps" && ls | grep '^lib' | grep -v '^libsinglefs' | grep -v '^libopus' | while read -r entry; do cp -a "$entry" "$WORK/target/release/deps/"; done)
  fi
  (cd "$WORK/tree" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo build --offline --release -p opus-r3-probe) > "$WORK/build-probe.log" 2>&1
  (cd "$WORK/tree" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --no-run) > "$WORK/build-harness.log" 2>&1
  local bin="$WORK/target/release/opus-r3-probe" two="SINGLEFS_PROBE_ACQUISITION=two_rounds SINGLEFS_PROBE_ROLLBACK=per_device"
  local one="SINGLEFS_PROBE_ACQUISITION=one_round SINGLEFS_PROBE_ROLLBACK=per_written"
  export SINGLEFS_CRASH_POINT_REPLAY_THREADS=31
  (cd "$WORK" && {
    for arm in "$one" "$two"; do env $arm "$bin" r3 counts2 counts3 2>&1; env $arm PROBE_COUNT_ONLY=1 "$bin" acq k0 k2 r1 k0_pub; env $arm PROBE_DEVICES=3 PROBE_COUNT_ONLY=1 "$bin" acq k0 2>&1; done
    for behind in tail record_units record_or_tail_units; do env SINGLEFS_PROBE_BEHIND=$behind $two "$bin" r3 snapshot0 snapshot1; done
  } | grep -o 'probe=.*' | grep -v 'kinds=' > counts-and-snapshot.txt)
  (cd "$WORK" && { for arm in "$one" "$two"; do env $arm "$bin" alpha_prime cells_k2 scan_cost; done; env $two "$bin" alpha c555; env $two "$bin" rollback k0 r1; } | grep -o 'probe=.*' > alpha-prime-and-rollback.txt)
  (cd "$WORK" && { env $two PROBE_CONFIRMED_TXG=5 "$bin" acq k2; env $two PROBE_CONFIRMED_TXG=3 "$bin" acq k0; } | grep -o 'probe=.*' > s3-two_rounds.txt)
  (cd "$WORK" && { env $one PROBE_CONFIRMED_TXG=5 "$bin" acq k2; env $one "$bin" alpha_prime c555_scan; } | grep -o 'probe=.*' > s3-one_round-and-scan.txt)
  echo "name=rerun_done work=$WORK compare_with=$MODEL/out/<名>.txt"
  local name
  for name in counts-and-snapshot alpha-prime-and-rollback s3-two_rounds s3-one_round-and-scan; do
    if diff -q "$WORK/$name.txt" "$MODEL/out/$name.txt" > /dev/null 2>&1; then echo "run=$name same"; else echo "run=$name differs"; fi
  done
  # harness：红名单（取号写两轮那一臂下红的 20 个目标）两臂各跑一次，比退出码；落后支那份用例两种乙' 读法各跑一次
  grep -v '^publish_order_matches_litmus' "$MODEL/out/harness-red-assertions.txt" | sed -n 's/^== \([^ ]*\) .*/\1/p' > "$WORK/red-targets.txt"
  sed "s#^cd .*repo || exit 2#cd $WORK/tree || exit 2#; s#^export CARGO_TARGET_DIR=.*#export CARGO_TARGET_DIR=$WORK/target#" "$MODEL/tools/harness-slice.sh" > "$WORK/harness-slice.sh"
  local count; count=$(wc -l < "$WORK/red-targets.txt")
  RUST_TEST_THREADS=2 bash "$WORK/harness-slice.sh" "$WORK/red-targets.txt" 1 "$count" 10 "$WORK/harness-baseline"
  env $two RUST_TEST_THREADS=2 bash "$WORK/harness-slice.sh" "$WORK/red-targets.txt" 1 "$count" 10 "$WORK/harness-two_rounds"
  echo entries_after_mount_refuse_swapped_or_behind_devices > "$WORK/behind.txt"
  for behind in record_units record_or_tail_units; do env SINGLEFS_PROBE_BEHIND=$behind $two bash "$WORK/harness-slice.sh" "$WORK/behind.txt" 1 1 1 "$WORK/harness-behind-$behind"; done
  local target
  while read -r target; do echo "harness=$target baseline_rc=$(cat "$WORK/harness-baseline/$target.rc") two_rounds_rc=$(cat "$WORK/harness-two_rounds/$target.rc")"; done < "$WORK/red-targets.txt"
  for behind in record_units record_or_tail_units; do echo "harness=behind arm=$behind rc=$(cat "$WORK"/harness-behind-$behind/*.rc)"; done
  if [[ "${RERUN_HARNESS_ALL:-}" == 1 ]]; then
    ls "$WORK/tree/crates/singlefs-harness/tests/"*.rs | xargs -n1 basename | sed 's/\.rs$//' > "$WORK/all-targets.txt"; echo lib >> "$WORK/all-targets.txt"
    env $two RUST_TEST_THREADS=2 bash "$WORK/harness-slice.sh" "$WORK/all-targets.txt" 1 "$(wc -l < "$WORK/all-targets.txt")" 16 "$WORK/harness-all-two_rounds"
    while read -r target; do echo "$target two_rounds_rc=$(cat "$WORK/harness-all-two_rounds/$target.rc" 2>/dev/null || echo missing)"; done < "$WORK/all-targets.txt" > "$WORK/harness-two_rounds-per-target.txt"
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
