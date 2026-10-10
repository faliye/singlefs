#!/usr/bin/env bash
# 复跑 m3-batch35-36-code-r1 攻方（Opus）模型。用法：rerun.sh <空工作目录> [阶段…]（不给阶段就按次序串行跑全部；单个阶段前台都在 4 分钟内）。
# 环境变量：
#   SINGLEFS_REPO   被判的树根（有 Cargo.toml、Cargo.lock、crates/；本轮是冻结副本 ~/.local/share/singlefs/work/m3-batch35-36-code-r1/tree）。只读，拷进 <工作目录>/repo 再改。
#   SINGLEFS_TOOLS  仓根：只取 research/scripts/run-with-memory-cap.sh 与 research/scripts/capped.sh 两个包装脚本。
#   THREADS         线程上限（默认 7，经 capped.sh 设给 cargo 与测试）。
# 阶段：prep（拷树、patch_arms.py 加臂开关、放进探针）→ build（编，6G）→ h1 → ow → sw3vis → sw3hid → sw3f1 → sw3both → sw3all3 → sw3single → w4 → exist → v2model → summary。
#   跑都经 run-with-memory-cap.sh（跑 4G，编 6G），同一时刻只起一条 cargo；编译目录 <工作目录>/target。
#   臂由环境变量选（patch_arms.py 文件头）：不设 = 冻结副本原样；OPUS_C_NEWER_ROOTS / OPUS_F1_AFTER_F2 / OPUS_C_MAPPING / OPUS_F2_NEWEST 各开一臂。
# 输出：<工作目录>/out/<名>.log；summary 阶段用 summarize.py 把各 log 的判定行、TALLY 与 test result 行（去掉耗时）收进 <工作目录>/summary.txt，排好序。
# 比对：compare.sh <工作目录>/summary.txt（与本目录 expected-summary.txt 比）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:?要一个工作目录（第一次跑要空的）}"; shift || true
: "${SINGLEFS_REPO:?SINGLEFS_REPO 指被判的树根}"
: "${SINGLEFS_TOOLS:?SINGLEFS_TOOLS 指仓根（取包装脚本）}"
threads="${THREADS:-7}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"
cap="$SINGLEFS_TOOLS/research/scripts/run-with-memory-cap.sh"
capped="$SINGLEFS_TOOLS/research/scripts/capped.sh"
export CARGO_TARGET_DIR="$work/target"
stages=("$@")
[ "${#stages[@]}" -gt 0 ] || stages=(prep build h1 ow sw3vis sw3hid sw3f1 sw3both sw3all3 sw3single w4 exist v2model summary)
all3=(OPUS_C_NEWER_ROOTS=1 OPUS_F1_AFTER_F2=1 OPUS_C_MAPPING=1)
ptest() { # ptest <日志名> <测试目标> <过滤或 -> <环境变量赋值…>
  local name="$1" target="$2" filter="$3"; shift 3
  local args=(--nocapture --test-threads=1)
  [ "$filter" = "-" ] || args+=("$filter")
  ( cd "$work/repo" && env OPUS_RUN=1 "$@" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test -q -p singlefs-harness --test "$target" -- "${args[@]}" ) > "$work/out/$name.log" 2>&1 || true
  echo "done $name" >> "$work/progress.txt"
}
cb() { ptest "$1" opus_b3536_probe opus_c_blind_spot_sweep "${@:2}"; }
existing=(
  writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten
  writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero
  writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero
  recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed
  read_recovery_fields_follow_the_fallback_to_the_witnessed_version
)
for stage in "${stages[@]}"; do
  case "$stage" in
    prep)
      [ ! -e "$work/repo" ] || { echo "工作目录不空：$work/repo 已在" >&2; exit 2; }
      mkdir -p "$work/repo" "$work/out"
      cp "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$work/repo/"
      cp -r "$SINGLEFS_REPO/crates" "$work/repo/crates"
      python3 -B "$here/patch_arms.py" "$work/repo"
      cp "$here/probe/opus_b3536_probe.rs" "$work/repo/crates/singlefs-harness/tests/"
      ;;
    build)
      targets=(--test opus_b3536_probe)
      for t in "${existing[@]}"; do targets+=(--test "$t"); done
      ( cd "$work/repo" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo test -q -p singlefs-harness "${targets[@]}" --no-run ) > "$work/build.log" 2>&1
      ;;
    h1)
      ptest h1-base opus_b3536_probe opus_h1_
      ptest h1-newer opus_b3536_probe opus_h1_ OPUS_C_NEWER_ROOTS=1
      ;;
    ow)
      for k in 1 3 5 7; do cb "cbd2-ow-k$k-base" OPUS_KINDS=ow OPUS_KS=$k OPUS_DEPTH=2; done
      cb cbd2-ow-k5-newer OPUS_KINDS=ow OPUS_KS=5 OPUS_DEPTH=2 OPUS_C_NEWER_ROOTS=1
      cb cbd2-ow-k5-all3 OPUS_KINDS=ow OPUS_KS=5 OPUS_DEPTH=2 "${all3[@]}"
      ;;
    sw3vis)
      cb cbd2-sw3-k5-rootvisible-base OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=2 OPUS_ROOT_HIDDEN=false
      cb cb-sw3-k5-rootvisible-newer OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=1 OPUS_ROOT_HIDDEN=false OPUS_C_NEWER_ROOTS=1
      ;;
    sw3hid)
      cb cbd2-sw3-k5-roothidden-base OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=2 OPUS_ROOT_HIDDEN=true
      cb cb-sw3-k5-roothiddentrue-all3 OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=1 OPUS_ROOT_HIDDEN=true "${all3[@]}"
      ;;
    sw3f1)
      cb cb-sw3-k5-rootvisible-f1 OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=1 OPUS_ROOT_HIDDEN=false OPUS_F1_AFTER_F2=1
      ;;
    sw3both)
      cb cb-sw3-k5-rootvisible-both OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=1 OPUS_ROOT_HIDDEN=false OPUS_F1_AFTER_F2=1 OPUS_C_NEWER_ROOTS=1
      ;;
    sw3all3)
      cb cb-sw3-k5-roothiddenfalse-all3 OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=1 OPUS_ROOT_HIDDEN=false "${all3[@]}"
      ;;
    sw3single)
      cb cb-sw3-k5-roothidden-singlecopy-base OPUS_KINDS=sw3 OPUS_KS=5 OPUS_DEPTH=1 OPUS_ROOT_HIDDEN=true OPUS_SINGLE_COPY=1
      ;;
    w4)
      ptest w4-base opus_b3536_probe opus_w4_
      ptest w4-f2newest opus_b3536_probe opus_w4_ OPUS_F2_NEWEST=1
      ;;
    exist)
      for t in "${existing[@]}"; do
        ptest "exist-$t-base" "$t" -
        ptest "exist-$t-newer" "$t" - OPUS_C_NEWER_ROOTS=1
        ptest "exist-$t-f1" "$t" - OPUS_F1_AFTER_F2=1
        ptest "exist-$t-f2newest" "$t" - OPUS_F2_NEWEST=1
        ptest "exist-$t-all3" "$t" - "${all3[@]}"
      done
      ;;
    v2model)
      for config in "4 3 1" "5 3 1"; do
        # shellcheck disable=SC2086
        nice -n 19 python3 -B "$here/v2_fold_model.py" $config > "$work/out/v2model-${config// /-}.log" 2>&1
      done
      ;;
    summary)
      python3 -B "$here/summarize.py" "$work"/out/*.log > "$work/summary.txt"
      ;;
    *) echo "不认识的阶段：$stage" >&2; exit 2 ;;
  esac
done
