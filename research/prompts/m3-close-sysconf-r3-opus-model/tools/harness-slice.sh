#!/usr/bin/env bash
# 攻方副本（m3-close-sysconf-r3）：harness 档的测试目标逐个跑（每个目标一条 cargo test --test），一次跑名单里第 <起>–<止> 行，
# 每 <并行> 个一组并行，退出码写进 <输出目录>/<目标>.rc，输出写进 <输出目录>/<目标>.log。臂由调用方设的环境变量定。
# 用法：harness-slice.sh <名单> <起> <止> <并行> <输出目录>
set -u
list="$1"; first="$2"; last="$3"; parallel="$4"; out="$5"
mkdir -p "$out"
cd /home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r3-opus/repo || exit 2
export CARGO_TARGET_DIR=/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r3-opus/target
mapfile -t targets < <(sed -n "${first},${last}p" "$list")
index=0
for target in "${targets[@]}"; do
  rm -f "$out/$target.rc"
  {
    if [[ "$target" == lib ]]; then
      SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的库内单元测试（攻方副本，名单里逐个）; 范围=1 个测试目标; 为什么=攻方腿量取号写两轮在 harness 档现有用例上的牵连面，逐个目标跑、退出码各自落盘' cargo test --offline -q -p singlefs-harness --lib > "$out/$target.log" 2>&1
    else
      SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的一个测试目标（攻方副本，名单里逐个）; 范围=1 个测试目标; 为什么=攻方腿量取号写两轮在 harness 档现有用例上的牵连面，逐个目标跑、退出码各自落盘' cargo test --offline -q -p singlefs-harness --test "$target" > "$out/$target.log" 2>&1
    fi
    echo "$?" > "$out/$target.rc"
  } &
  index=$((index + 1))
  if (( index % parallel == 0 )); then wait; fi
done
wait
