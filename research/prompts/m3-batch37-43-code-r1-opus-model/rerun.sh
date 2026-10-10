#!/usr/bin/env bash
# 复跑 m3-batch37-43-code-r1 攻方（Opus）模型。用法：rerun.sh <空工作目录> [阶段…]（不给阶段就按次序串行跑全部；每个阶段前台都在 4 分钟内）。
# 环境变量：
#   SINGLEFS_REPO   被判的树根（有 Cargo.toml、Cargo.lock、crates/；本轮是冻结副本 ~/.local/share/singlefs/work/m3-batch37-43-code-r1/tree）。只读，拷进 <工作目录>/repo 再改。
#   SINGLEFS_TOOLS  仓根：只取 research/scripts/run-with-memory-cap.sh 与 research/scripts/capped.sh 两个包装脚本。
#   THREADS         线程上限（默认 15，经 capped.sh 设给 cargo、层 0 枚举与探针的工作线程）。
# 阶段：prep（拷树、patch_arms.py 加臂、放进探针 crate）→ build（编探针 release、编 8 份入库 harness 用例 debug，都 6G）→ swow → swsw3 → sc → l0 → f2 → exist → summary。
#   跑都经 run-with-memory-cap.sh（跑 4G，编 6G），同一时刻只起一条 cargo；编译目录 <工作目录>/target。
#   臂由环境变量选（patch_arms.py 文件头）：不设 = 冻结副本原样。
#   层 0 枚举（sc、l0 两段）照 singlefs-checker-tier 的 enumerate_layer0_selecting_versions_observing_each_state，每一段都展开；每次跑之前探针按
#   closed_form_state_count 算全量，超过 OPUS_L0_CAP（默认 10^6）的那一段不跑、打 SKIP。本模型全部层 0 合计 4 次 × 约 2.05 万状态。
# 输出：<工作目录>/out/<阶段名>.log；summary 阶段用 summarize.py 收进 <工作目录>/summary.txt（排好序、不带耗时）。
# 比对：compare.sh <工作目录>/summary.txt（与本目录 expected-summary.txt 比）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:?要一个工作目录（第一次跑要空的）}"; shift || true
: "${SINGLEFS_REPO:?SINGLEFS_REPO 指被判的树根}"
: "${SINGLEFS_TOOLS:?SINGLEFS_TOOLS 指仓根（取包装脚本）}"
threads="${THREADS:-15}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"
cap="$SINGLEFS_TOOLS/research/scripts/run-with-memory-cap.sh"
capped="$SINGLEFS_TOOLS/research/scripts/capped.sh"
export CARGO_TARGET_DIR="$work/target"
stages=("$@")
[ "${#stages[@]}" -gt 0 ] || stages=(prep build swow swsw3 sc l0 f2 exist summary)
existing=(
  recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed
  writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing
  writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list
  writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero
  writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version
  writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault
  writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten
  writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image
)
probe() { # probe <日志名> <用例过滤> <环境变量赋值…>
  local name="$1" filter="$2"; shift 2
  ( cd "$work/repo" && env "$@" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test --release --offline -q -p opus-probe --test opus_b3743_probe -- --nocapture --test-threads=1 "$filter" ) > "$work/out/$name.log" 2>&1 || true
  echo "done $name" >> "$work/progress.txt"
}
sweep() { local name="$1"; shift; probe "$name" opus_sweep OPUS_ROOT_HIDDEN=false OPUS_RECMASKS=all OPUS_DEPTH=2 "$@"; }
for stage in "${stages[@]}"; do
  case "$stage" in
    prep)
      [ ! -e "$work/repo" ] || { echo "工作目录不空：$work/repo 已在" >&2; exit 2; }
      mkdir -p "$work/repo" "$work/out"
      cp "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$work/repo/"
      cp -r "$SINGLEFS_REPO/crates" "$work/repo/crates"
      python3 -B "$here/patch_arms.py" "$work/repo"
      mkdir -p "$work/repo/crates/opus-probe"
      cp -r "$here/probe/." "$work/repo/crates/opus-probe/"
      ;;
    build)
      ( cd "$work/repo" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo test --release --offline -q -p opus-probe --test opus_b3743_probe --no-run ) > "$work/build-probe.log" 2>&1
      targets=()
      for t in "${existing[@]}"; do targets+=(--test "$t"); done
      ( cd "$work/repo" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo test --offline -q -p singlefs-harness "${targets[@]}" --no-run ) > "$work/build-harness.log" 2>&1
      ;;
    swow)
      sweep sw-ow-k135-sub2-base OPUS_KINDS=ow OPUS_KS=1,3,5 OPUS_MAXSUB=2
      sweep sw-ow-k135-sub2-arm2 OPUS_KINDS=ow OPUS_KS=1,3,5 OPUS_MAXSUB=2 OPUS_ARM_REFUSE_UNMAPPED=2
      sweep sw-ow-k5-sub3-base OPUS_KINDS=ow OPUS_KS=5 OPUS_MAXSUB=3
      ;;
    swsw3)
      sweep sw-sw3-k5-sub2-base OPUS_KINDS=sw3 OPUS_KS=5 OPUS_MAXSUB=2
      sweep sw-sw3-k5-sub2-arm2 OPUS_KINDS=sw3 OPUS_KS=5 OPUS_MAXSUB=2 OPUS_ARM_REFUSE_UNMAPPED=2
      ;;
    sc)
      probe sc-ow-k5-rootrec opus_second_crash OPUS_KINDS=ow OPUS_KS=5 OPUS_M1_HIDE=root+rec
      probe sc-sw3-k5-rootrec opus_second_crash OPUS_KINDS=sw3 OPUS_KS=5 OPUS_M1_HIDE=root+rec
      ;;
    l0)
      probe l0-hit-ow-k5 opus_sweep OPUS_L0=1 OPUS_KINDS=ow OPUS_KS=5 OPUS_UNITS=6+8 OPUS_RECMASKS=1 OPUS_ROOT_HIDDEN=false OPUS_DEPTH=0
      probe l0-ctrl-ow-k5 opus_sweep OPUS_L0=1 OPUS_L0_ALWAYS=1 OPUS_KINDS=ow OPUS_KS=5 OPUS_UNITS=8 OPUS_RECMASKS=1 OPUS_ROOT_HIDDEN=false OPUS_DEPTH=0
      ;;
    f2)
      probe f2-base opus_f2_newest OPUS_NONE=1
      probe f2-noskip1 opus_f2_newest OPUS_ARM_F2_NO_SKIP=1
      probe f2-noskip2 opus_f2_newest OPUS_ARM_F2_NO_SKIP=2
      ;;
    exist)
      for arm in base:OPUS_NONE=1 arm1:OPUS_ARM_REFUSE_UNMAPPED=1 arm2:OPUS_ARM_REFUSE_UNMAPPED=2 noskip1:OPUS_ARM_F2_NO_SKIP=1 noskip2:OPUS_ARM_F2_NO_SKIP=2; do
        for t in "${existing[@]}"; do
          ( cd "$work/repo" && env "${arm#*:}" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test --offline -q -p singlefs-harness --test "$t" -- --test-threads=1 ) > "$work/out/exist-${arm%%:*}-$t.log" 2>&1 || true
        done
        echo "done exist-${arm%%:*}" >> "$work/progress.txt"
      done
      ;;
    summary)
      python3 -B "$here/summarize.py" "$work"/out/*.log > "$work/summary.txt"
      ;;
    *) echo "不认识的阶段：$stage" >&2; exit 2 ;;
  esac
done
