#!/usr/bin/env bash
# 复跑 m2-layer0-scale-r3 云端攻方腿（Opus）的探针。全部在冻结副本的拷贝上跑（草稿副本），不是入库装置；数不进 kb。
# 用法（在仓根下）：bash research/prompts/m2-layer0-scale-r3-opus-model/rerun.sh [冻结副本根] [草稿目录]
#   冻结副本根默认 /tmp/claude-1000/l0scale-r1-frozen；草稿目录默认 /tmp/claude-1000/m2-layer0-scale-r3-opus-rerun
# 环境变量 THREADS（线程上限，默认 6）。只跑本目录的两个测试目标（名字不带 layer0），不跑名字带 layer0 的目标、不跑 54 号；
# 不做层 0 的崩溃状态枚举，只评手搭的单个状态（每条历史的闭式打在 R3A / R3B 行里，都远超 10^6）。
# 挂钟（2026-09-26 JST 本机、6 线程、nice 19）：编译约 1 分钟；r3a 23 条历史约 40 秒；r3b 不到 1 秒；③ 的场景约 1 分钟。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../.." && pwd)"
frozen="${1:-/tmp/claude-1000/l0scale-r1-frozen}"
scratch="${2:-/tmp/claude-1000/m2-layer0-scale-r3-opus-rerun}"
threads="${THREADS:-6}"
cap() { nice -n 19 bash "$repo_root/research/scripts/run-with-memory-cap.sh" 8G bash "$repo_root/research/scripts/capped.sh" "$threads" "$@"; }
mkdir -p "$scratch"
# ① 副本：两处 helper 改成 pub（逻辑不动），放探针
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/repo/"
patch -s -d "$scratch/repo" -p1 < "$here/crash-rs-pub-helpers.patch"
cp "$here/opus_r3_probe.rs" "$scratch/repo/crates/singlefs-harness/tests/"
P=opus_r3_probe
seqs21=OSMUMUOURSMMUR,OURSMUSRUSUUR,SUOSSOUSMRUSUR,RUOUOMUSRUSUOU,USUOOSSUURMSR,SOMSMOUSSSSOMU,USRMSSUSUSRORU,SSUURMSMUOOOU,MRSSUSROORSUU,RUSUSMUSOUS,SSUMOOUSSROOMU,MMUSOUSMUSU,MUMUSRUORUMUR,UURUSOURSURS,UUOSUSMRROURS,SMUUOUSSOUUMUR,MSSSUMOOSRMORO,UOOUSUSOMMSU,MMOUOMSMUOUU,SUSOMUSORUSUR,OUROMUOUOSSURU
( cd "$scratch/repo" && PROBE_SEQS=UOOUOMSU cap cargo test --release -p singlefs-harness --test $P -- --exact r3a_misaligned_c507_cell_against_the_m1_false_red --nocapture ) > "$scratch/r3a-uooouomsu.log" 2>&1
( cd "$scratch/repo" && PROBE_SEQS="$seqs21,USUSUSOUS,USOUSMSU" cap cargo test --release -p singlefs-harness --test $P -- --exact r3a_misaligned_c507_cell_against_the_m1_false_red --nocapture ) > "$scratch/r3a-sweep23.log" 2>&1
( cd "$scratch/repo" && cap cargo test --release -p singlefs-harness --test $P -- --exact r3b_c513_illegal_reuse_under_sector_wise_rules --nocapture ) > "$scratch/r3b.log" 2>&1
# ③ 副本：只放一个打印编译环境的测试（singlefs-format 下），外加三个包装脚本
mkdir -p "$scratch/cfgprobe"
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/cfgprobe/repo3/"
mkdir -p "$scratch/cfgprobe/repo3/crates/singlefs-format/tests"
cp "$here/opus_r3_build_env_probe.rs" "$scratch/cfgprobe/repo3/crates/singlefs-format/tests/"
cp "$here/rustc-wrapper.sh" "$here/rustc-as-RUSTC.sh" "$here/runner.sh" "$scratch/cfgprobe/"
chmod +x "$scratch/cfgprobe/"*.sh
CFGPROBE_DIR="$scratch/cfgprobe" cap bash "$here/build-env-scenarios.sh" > "$scratch/build-env-scenarios.log" 2>&1
# 汇总
grep -hE '^(R3A seq=.*(closed_form|images_identical)|  R3A seq=.*images_identical|R3B |  R3STATE C513|SCENARIO |test result)' "$scratch"/r3a-uooouomsu.log "$scratch"/r3b.log "$scratch"/build-env-scenarios.log || true
for st in X_only_y Y_u2 Yp_only C_u_and; do
  for r in Today SecPers SecPers513 SecDisk SecDisk513 SecDiskRec SecDiskRec513; do
    printf '%s:%s=%s ' "$st" "$r" "$(grep -E "R3RULE ${st}[^ ]* ${r} missing" "$scratch/r3a-sweep23.log" | grep -vc '=\[\]' || true)"
  done
  printf '%s:layer0_red=%s\n' "$st" "$(grep -E "R3STATE ${st}" "$scratch/r3a-sweep23.log" | grep -c 'red=true' || true)"
done
echo "images_identical(X,Y)=true in $(grep -c 'images_identical(X,Y)=true' "$scratch/r3a-sweep23.log") of $(grep -c 'images_identical(X,Y)=' "$scratch/r3a-sweep23.log") histories with a u2"
