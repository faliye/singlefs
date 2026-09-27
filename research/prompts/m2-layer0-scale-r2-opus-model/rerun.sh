#!/usr/bin/env bash
# 复跑 m2-layer0-scale-r2 云端攻方腿（Opus）的探针。全部跑在冻结副本的拷贝上（草稿副本），不是入库装置；数不进 kb。
# 用法：bash rerun.sh [冻结副本根] [草稿目录]
#   冻结副本根默认 /tmp/claude-1000/l0scale-r1-frozen；草稿目录默认 /tmp/claude-1000/m2-layer0-scale-r2-opus-rerun
# 环境变量：THREADS（线程上限，默认 10）。只跑本目录的两个探针二进制（名字不带 layer0），不跑名字带 layer0 的目标、不跑 54 号。
# 挂钟（2026-09-26 JST 本机、10 线程、nice 19）：q3 约 6 分钟，q1 随机 120 条约 16 分钟，q6 约 19 分钟，q9 约 4 分钟，其余每格不到 1 分钟。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../.." && pwd)"
frozen="${1:-/tmp/claude-1000/l0scale-r1-frozen}"
scratch="${2:-/tmp/claude-1000/m2-layer0-scale-r2-opus-rerun}"
threads="${THREADS:-10}"
r1="$repo_root/research/prompts/m2-layer0-scale-r1-opus-model"
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/repo/"
cp "$here/opus_r2_probe.rs" "$scratch/repo/crates/singlefs-harness/tests/"
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/repo-l7/"
patch -s -d "$scratch/repo-l7" -p1 < "$r1/l7-resume-prototype-crash-rs.patch"
cp "$here/opus_r2_resume_probe.rs" "$scratch/repo-l7/crates/singlefs-harness/tests/"
run() { # <副本> <测试目标> <日志名> <用例> [环境变量=值…]
  local copy="$1" target="$2" log="$3" test_name="$4"; shift 4
  if ( cd "$scratch/$copy" && env PROBE_SCRATCH="$scratch/l7-scratch" "$@" nice -n 19 bash "$repo_root/research/scripts/run-with-memory-cap.sh" 8G \
      bash "$repo_root/research/scripts/capped.sh" "$threads" \
      cargo test --release -p singlefs-harness --test "$target" -- --exact "$test_name" --nocapture ) > "$scratch/$log" 2>&1; then
    echo "$log exit=0"
  else
    echo "$log exit=$?"
  fi
}
P=opus_r2_probe
# M1：静态扫豁免链候选（3000 条随机历史）、候选坐实与缩小、整条流的甲二与 σ 段全枚举、真值表、固定脚本上有没有、按扇区判的改法
run repo $P q3-sweep1.log q3_static_exemption_chains_over_sampled_histories PROBE_N=3000 PROBE_MAX_LEN=14 PROBE_ALPHABET=OUMRS PROBE_SEED=7
seqs="$(grep 'chain_candidates=[1-9]' "$scratch/q3-sweep1.log" | awk '{print $3}' | sed 's/seq=//' | paste -sd,)"
run repo $P q5-confirm21.log q5_confirm_and_shrink PROBE_SEQS="$seqs"
run repo $P q5-shrink.log q5_confirm_and_shrink PROBE_SHRINK=1 PROBE_SEQS=RUSUSMUSOUS,UOOUSUSOMMSU,SOMSMOUSSSSOMU,MMUSOUSMUSU
run repo $P q6-1.log q6_whole_stream_arms_and_segment_enumeration PROBE_SEQS=UOOUOMSU,USOUSMSU,USUSUSOUS
run repo $P q7_exemption_chains_on_fixed_scripts.log q7_exemption_chains_on_fixed_scripts
run repo $P q8_four_write_truth_table.log q8_four_write_truth_table PROBE_SEQS=UOOUOMSU
run repo $P q9.log q9_sector_wise_exemption_on_sigma PROBE_SEQS=UOOUOMSU
run repo $P q1-mirror.log q1_search_non_monotone_cow_subsets_over_histories PROBE_SEQS=UOOUOMSU,O,UO PROBE_VERIFY=0 PROBE_ALL_IMASKS=1
run repo $P q1-sweep120.log q1_search_non_monotone_cow_subsets_over_histories PROBE_N=120 PROBE_MAX_LEN=10 PROBE_SEED=11 PROBE_ALPHABET=OUMRS PROBE_ALL_IMASKS=1 PROBE_VERIFY=0
run repo $P q10.log q10_forced_zero_reuse_window_arms PROBE_SEQS=ZOOOO,OZOOO,UZOOOSO,ZSOSOS,MZOOO,UOZOSOS,ZOUZOO
# M2：R1 一份指纹一个进度文件、R4 观察者里的断言
run repo-l7 opus_r2_resume_probe r2a.log r2a_one_file_per_input_fingerprint_lets_the_first_stream_wipe_the_second
run repo-l7 opus_r2_resume_probe r2b.log r2b_observer_assertion_is_lost_on_resume
# M3：tests/common.rs 与 tests/common/mod.rs 并存
: > "$scratch/repo/crates/singlefs-harness/tests/common.rs"; touch "$scratch/repo/crates/singlefs-harness/tests/$P.rs"
( cd "$scratch/repo" && nice -n 19 bash "$repo_root/research/scripts/run-with-memory-cap.sh" 8G bash "$repo_root/research/scripts/capped.sh" "$threads" \
    cargo test --release -p singlefs-harness --test $P --no-run ) > "$scratch/m3-common-rs-2.log" 2>&1 || echo "m3-common-rs-2.log exit=$?"
rm "$scratch/repo/crates/singlefs-harness/tests/common.rs"
grep -hE '^(Q1_TOTAL|Q3_TOTAL|Q5 |  CONFIRMED|  SHRUNK|Q6 |  ARM_A2 |  SIGMA_FULL|Q7 |Q9 |Q10 |ARMS |R2A|R2B|error\[E0761\])' "$scratch"/*.log
