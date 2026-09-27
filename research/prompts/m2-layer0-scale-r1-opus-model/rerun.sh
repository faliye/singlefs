#!/usr/bin/env bash
# 复跑 m2-layer0-scale-r1 云端攻方腿（Opus）的探针。全部跑在冻结副本的拷贝上（草稿副本），不是入库装置；数不进 kb。
# 用法：bash rerun.sh [冻结副本根] [草稿目录]
#   冻结副本根默认 /tmp/claude-1000/l0scale-r1-frozen（133 个文件的 sha256 在 /tmp/claude-1000/l0scale-r1-frozen-sha256.txt）
#   草稿目录默认 /tmp/claude-1000/m2-layer0-scale-r1-opus-rerun（每一格一个副本：base、m304、msl、l7；日志落在草稿目录）
# 环境变量：THREADS（线程上限，默认 10）。只跑本目录的探针二进制 opus_scale_probe，不跑名字带 layer0 的目标、不跑 54 号。
# 挂钟（2026-09-26 JST 本机、10 线程、nice 19 量的）：p7 三格合计约 24 分钟，p11 约 20 分钟，其余每格不到 3 分钟。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../.." && pwd)"
frozen="${1:-/tmp/claude-1000/l0scale-r1-frozen}"
scratch="${2:-/tmp/claude-1000/m2-layer0-scale-r1-opus-rerun}"
threads="${THREADS:-10}"
mkdir -p "$scratch"

make_copy() { # <名字> <探针文件> [补丁…]：拷冻结副本、放探针、打补丁
  local name="$1" probe="$2"; shift 2
  local dir="$scratch/$name"
  mkdir -p "$dir"
  rsync -a --exclude target --exclude .git "$frozen/" "$dir/"
  cp "$here/$probe" "$dir/crates/singlefs-harness/tests/opus_scale_probe.rs"
  local p
  for p in "$@"; do patch -s -d "$dir" -p1 < "$here/$p"; done
}
run() { # <副本名> <日志名> <用例> [环境变量=值…]
  local name="$1" log="$2" test_name="$3"; shift 3
  if ( cd "$scratch/$name" && env "$@" nice -n 19 bash "$repo_root/research/scripts/run-with-memory-cap.sh" 12G \
      bash "$repo_root/research/scripts/capped.sh" "$threads" \
      cargo test --release -p singlefs-harness --test opus_scale_probe -- --exact "$test_name" --nocapture ) > "$scratch/$log" 2>&1; then
    echo "$log exit=0"
  else
    echo "$log exit=$?"
  fi
}

make_copy base opus_scale_probe.rs
make_copy m304 opus_scale_probe.rs mutation-304-no-barrier-after-acquisition.patch
make_copy msl opus_scale_probe.rs mutation-shadow-ledger-off.patch
make_copy l7 opus_scale_probe_with_l7.rs l7-resume-prototype-crash-rs.patch

# L1：段序列、单元写盖不盖旧扇区、抽样对照、三臂
run base p1.log p1_fixed_script_streams_and_overwrites
run base p2.log p2_premise_sampled_on_fixed_scripts PROBE_RANDOM=8
for w in C D E18 H1 H2; do run base "p6-$w.log" p6_three_arms_on_fixed_scripts_and_histories PROBE_WHICH=$w PROBE_RANDOM=32; done
# L1：≤ 10^6 个状态的穷举对照（三格合计 786423 个状态）
for plan in C13 E18_54_none E18_54_all; do run base "p7-$plan.log" p7_exhaustive_control PROBE_PLAN=$plan; done
# 用户动作放开扫（前缀 A、B，可选挂载时读故障；之后 ≤ 3 步）
run base p11.log p11_user_action_sweep PROBE_SWEEP_LEN=3 PROBE_RANDOM=4
# L3：变异 304（取号之后没有屏障）与影子账关掉
run m304 p6-C-m304.log p6_three_arms_on_fixed_scripts_and_histories PROBE_WHICH=C PROBE_RANDOM=32
run msl p6-D-msl.log p6_three_arms_on_fixed_scripts_and_histories PROBE_WHICH=D PROBE_RANDOM=32
# L7：续跑原型
run l7 p8.log p8_resume_every_byte_prefix_and_line_boundary PROBE_SCRATCH="$scratch/l7-scratch"
run l7 p10.log p10_weak_keys_threads_versions_observer PROBE_SCRATCH="$scratch/l7-scratch"
run l7 p9.log p9_kill_and_resume PROBE_SCRATCH="$scratch/l7-scratch"

grep -hE '^(STREAM|PREMISE_TOTAL|ARMS|ARMSEG|CONTROL_RESULT|SWEEP_TOTAL|L7|H1 mount)' "$scratch"/*.log
