#!/usr/bin/env bash
# 复跑 m3-batch33t-code-r1 攻方（Opus）模型。用法：rerun.sh <空工作目录> [阶段…]（不给阶段就按次序跑全部；单个阶段前台都在 4 分钟内）。
# 环境变量：
#   SINGLEFS_REPO   被判的树根（有 Cargo.toml、Cargo.lock、crates/；本轮是冻结副本 ~/.local/share/singlefs/work/m3-batch33t-code-r1/tree）。只读，拷进 <工作目录>/repo 再改。
#   SINGLEFS_TOOLS  仓根：只取 scripts/hosts/run-with-memory-cap.sh 与 scripts/hosts/capped.sh 两个包装脚本。
#   THREADS         线程上限（默认 10，经 capped.sh 设给 cargo 与测试）。
# 阶段：prep（拷树、patch_arms.py 加臂开关、放进两份探针）→ build（编，6G）→ t5 → t1 → t1deep1 → t1deep4 → t4 → t7 → t7b → t7big → exist → oracle → summary。
#   跑都经 run-with-memory-cap.sh（跑 4G，编 6G），同一时刻只起一条 cargo 或一个测试进程；编译目录 <工作目录>/target 与 <工作目录>/target-oracle。
#   臂由环境变量选（patch_arms.py 文件头）：不设 = 冻结副本原样；OPUS_F2_OFF / OPUS_F2_EXACT / OPUS_F1_OFF / OPUS_F1_ANY / OPUS_C_ACCOUNT 各开一臂。
# 输出：<工作目录>/out/<名>.log；summary 阶段把各 log 里以 OPUS 开头的判定行与既有用例的 test result 行（去掉耗时）收进 <工作目录>/summary.txt，排好序。
# 比对：compare.sh <工作目录>/summary.txt（与 expected/summary.txt 比）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:?要一个工作目录（第一次跑要空的）}"; shift || true
: "${SINGLEFS_REPO:?SINGLEFS_REPO 指被判的树根}"
: "${SINGLEFS_TOOLS:?SINGLEFS_TOOLS 指仓根（取包装脚本）}"
threads="${THREADS:-10}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"
cap="$SINGLEFS_TOOLS/scripts/hosts/run-with-memory-cap.sh"
capped="$SINGLEFS_TOOLS/scripts/hosts/capped.sh"
export CARGO_TARGET_DIR="$work/target"
stages=("$@")
[ "${#stages[@]}" -gt 0 ] || stages=(prep build t5 t1 t1deep1 t1deep4 t4 t7 t7b t7big exist oracle summary)
ptest() { # ptest <日志名> <测试二进制> <环境变量赋值…> -- <过滤>
  local name="$1" target="$2"; shift 2
  local envs=()
  while [ "$1" != "--" ]; do envs+=("$1"); shift; done
  shift
  ( cd "$work/repo" && env "${envs[@]}" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test -q -p singlefs-harness --test "$target" -- --nocapture --test-threads=1 "$@" ) > "$work/out/$name.log" 2>&1
  echo "done $name" >> "$work/progress.txt"
}
for stage in "${stages[@]}"; do
  case "$stage" in
    prep)
      [ ! -e "$work/repo" ] || { echo "工作目录不空：$work/repo 已在" >&2; exit 2; }
      mkdir -p "$work/repo" "$work/out"
      cp "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$work/repo/"
      cp -r "$SINGLEFS_REPO/crates" "$work/repo/crates"
      python3 -B "$here/patch_arms.py" "$work/repo"
      cp "$here/probe/opus_b33t_probe.rs" "$here/probe/opus_b33t_sweep.rs" "$work/repo/crates/singlefs-harness/tests/"
      mkdir -p "$work/oracle-probe/src"
      cp "$here/oracle-probe/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$work/oracle-probe/"
      cp "$here/oracle-probe/src/main.rs" "$work/oracle-probe/src/"
      ;;
    build)
      ( cd "$work/repo" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo test -q -p singlefs-harness \
          --test opus_b33t_probe --test opus_b33t_sweep \
          --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed \
          --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten \
          --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero --no-run ) > "$work/build.log" 2>&1
      ( cd "$work/oracle-probe" && CARGO_TARGET_DIR="$work/target-oracle" nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo build -q ) > "$work/build-oracle.log" 2>&1
      ;;
    t5)
      ptest t5-base opus_b33t_probe -- opus_t5
      ptest t5-F2_OFF opus_b33t_probe OPUS_F2_OFF=1 -- opus_t5
      ptest t5-F2_EXACT opus_b33t_probe OPUS_F2_EXACT=1 -- opus_t5
      ;;
    t1)
      ptest t1-base opus_b33t_probe OPUS_DEPTH=2 -- opus_t1
      ptest t1-C_ACCOUNT opus_b33t_probe OPUS_DEPTH=2 OPUS_C_ACCOUNT=1 -- opus_t1
      ptest t1-F1_ANY opus_b33t_probe OPUS_F1_ANY=1 -- opus_t1_blind_spot_last
      ;;
    t1deep1)
      ptest t1deep-base-k1 opus_b33t_probe OPUS_DEPTH=3 OPUS_KS=1 -- opus_t1_blind_spot_after
      ;;
    t1deep4)
      ptest t1deep-base-k4 opus_b33t_probe OPUS_DEPTH=3 OPUS_KS=4 -- opus_t1_blind_spot_after
      ;;
    t4)
      ptest t4-base opus_b33t_probe -- opus_t4_crash
      for copies in both one one-unit; do
        ptest "t4l-base-$copies" opus_b33t_probe OPUS_T4L_COPIES=$copies -- opus_t4_permanent
        ptest "t4l-F1_OFF-$copies" opus_b33t_probe OPUS_T4L_COPIES=$copies OPUS_F1_OFF=1 -- opus_t4_permanent
        ptest "t4l-F1_ANY-$copies" opus_b33t_probe OPUS_T4L_COPIES=$copies OPUS_F1_ANY=1 -- opus_t4_permanent
      done
      ptest t4-F1_ANY opus_b33t_probe OPUS_F1_ANY=1 -- opus_t4_crash
      ;;
    t7)
      ptest t7-small-w384 opus_b33t_sweep OPUS_SEEDS=4 OPUS_STEPS=100 OPUS_WIDTHS=w384 -- opus_t7_records
      ptest t7-small-w256 opus_b33t_sweep OPUS_SEEDS=4 OPUS_STEPS=100 OPUS_WIDTHS=w256 -- opus_t7_records
      ;;
    t7b)
      ptest t7-small-w240 opus_b33t_sweep OPUS_SEEDS=4 OPUS_STEPS=100 OPUS_WIDTHS=w240 -- opus_t7_records
      ptest t7-small-narrow opus_b33t_sweep OPUS_SEEDS=4 OPUS_STEPS=100 OPUS_WIDTHS=narrow -- opus_t7_records
      ;;
    t7big)
      ptest t7-4g opus_b33t_sweep OPUS_SEEDS=1 OPUS_STEPS=20 -- opus_t7_four
      ;;
    exist)
      for arm in NONE OPUS_F2_EXACT OPUS_C_ACCOUNT OPUS_F1_ANY; do
        for target in recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed \
                      writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten \
                      writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero; do
          ( cd "$work/repo" && env "$arm=1" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test -q -p singlefs-harness --test "$target" -- --test-threads=4 ) > "$work/out/exist-$arm-$target.log" 2>&1 || true
          echo "done exist-$arm-$target" >> "$work/progress.txt"
        done
      done
      ;;
    oracle)
      nice -n 19 bash "$cap" 4G "$work/target-oracle/debug/opus-b33t-oracle-probe" > "$work/out/oracle.log" 2>&1
      ;;
    summary)
      ( cd "$work/out" && for log in *.log; do
          name="${log%.log}"
          grep -o 'OPUS .*' "$log" | sed "s|^|$name |" || true
          grep -h '^test result:' "$log" | sed -e 's/; finished in .*//' -e "s|^|$name |" || true
        done ) | LC_ALL=C sort > "$work/summary.txt"
      wc -l "$work/summary.txt"
      ;;
    *) echo "不认的阶段：$stage" >&2; exit 2 ;;
  esac
done
