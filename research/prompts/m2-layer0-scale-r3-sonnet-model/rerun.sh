#!/usr/bin/env bash
# 复跑 m2-layer0-scale-r3 云端辩方（Sonnet）的探针：N1，替甲二辩护。
# 全部跑在冻结副本的拷贝上（草稿副本），不是入库装置；数不进 kb。
# 用法：bash rerun.sh [冻结副本根] [草稿目录]
#   冻结副本根默认 /tmp/claude-1000/l0scale-r1-frozen；草稿目录默认 /tmp/claude-1000/m2-layer0-scale-r3-sonnet-rerun
# 环境变量：THREADS（线程上限，默认 6）。
# 只跑本目录的一个探针二进制（名字不带 layer0），不跑名字带 layer0 的目标、不跑 54 号。
#
# 两份副本：
#   repo     打了 crash-sector-wise.patch（C561 的按扇区判豁免，辩方原型，不是入库的实六实现）
#   baseline 原样冻结副本（不打补丁），用来独立复现「打补丁前」的数（today_red=32768 那一类）
#
# 挂钟（2026-09-26 JST 本机、6 线程、nice 19）：
#   n13（σ=79 全枚举 262144 个状态 + 甲二全流）约 1371 秒；
#   q1（120 条随机历史，种子 11）约 1270 秒；
#   n12（150 条随机历史，种子 53）约 2054 秒；
#   q10（7 条回收窗口置 0 的历史）约数十秒；
#   baseline 上的 q9（探针侧扇区判变体 vs 今天的判据）约 373 秒。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../.." && pwd)"
frozen="${1:-/tmp/claude-1000/l0scale-r1-frozen}"
scratch="${2:-/tmp/claude-1000/m2-layer0-scale-r3-sonnet-rerun}"
threads="${THREADS:-6}"
mkdir -p "$scratch"

rsync -a --exclude target --exclude .git "$frozen/" "$scratch/repo/"
patch -s -d "$scratch/repo" -p1 < "$here/crash-sector-wise.patch"
cp "$here/n1_sonnet_probe.rs" "$scratch/repo/crates/singlefs-harness/tests/"

rsync -a --exclude target --exclude .git "$frozen/" "$scratch/baseline/"
cp "$here/n1_sonnet_probe.rs" "$scratch/baseline/crates/singlefs-harness/tests/"

run() { # <副本> <日志名> <用例> [环境变量=值…]
  local copy="$1" log="$2" test_name="$3"; shift 3
  if ( cd "$scratch/$copy" && env "$@" nice -n 19 bash "$repo_root/research/scripts/run-with-memory-cap.sh" 8G \
      bash "$repo_root/research/scripts/capped.sh" "$threads" \
      cargo test --release -p singlefs-harness --test n1_sonnet_probe -- --exact "$test_name" --nocapture ) > "$scratch/$log" 2>&1; then
    echo "$log exit=0"
  else
    echo "$log exit=$?"
  fi
}

# N1 补丁生效之后：两条产生 54 号 --full 的流上豁免链候选是不是仍然 0（与打补丁前一样，静态扫不读 check_records）
run repo n11-two-admitted-streams.log n11_exemption_chains_on_the_two_admitted_streams
run repo q7-fixed-scripts.log q7_exemption_chains_on_fixed_scripts
# N1 核心：打了补丁之后，UOOUOMSU 第 79 段全枚举（262144 个状态）与甲二（整条流）是否判得一样
run repo n13-sigma-full-and-arm-a2-patched.log n13_patched_sigma_full_enumeration_and_arm_a2 PROBE_SEQS=UOOUOMSU
# N1：第二轮攻方用过的 120 条随机历史（种子 11）在打了补丁之后还是不是 0 个不单调点
run repo q1-120-seed11-patched.log q1_search_non_monotone_cow_subsets_over_histories \
  PROBE_N=120 PROBE_MAX_LEN=10 PROBE_SEED=11 PROBE_ALPHABET=OUMRS PROBE_ALL_IMASKS=1 PROBE_VERIFY=0
# N1：第二轮攻方用过的 7 条回收窗口置 0 的历史（真 bug 场景），打了补丁之后甲二是不是照样红
run repo q10-forced-zero-reuse-window-patched.log q10_forced_zero_reuse_window_arms \
  PROBE_SEQS=ZOOOO,OZOOO,UZOOOSO,ZSOSOS,MZOOO,UOZOSOS,ZOUZOO
# N1（还有没有别的成员）：换一个第二轮没用过的种子（53），150 条历史，在打了补丁之后的判据上再搜一遍
run repo n12-150-seed53-patched.log n12_non_monotone_fresh_seed_after_patch PROBE_N=150 PROBE_MAX_LEN=10 PROBE_SEED=53 PROBE_ALPHABET=OUMRS

# 基线（不打补丁）：独立复现第二轮攻方的 q9.log（today_red=32768、sector_wise_red=0、differ=32768）
run baseline q9-baseline-unpatched.log q9_sector_wise_exemption_on_sigma PROBE_SEQS=UOOUOMSU

grep -hE '^(N11|Q7 |N13_SIGMA_FULL|N13_ARM_A2|Q1_TOTAL|ARMS |N12_TOTAL|Q9 )' "$scratch"/*.log
