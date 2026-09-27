#!/usr/bin/env bash
# defs-m2-closeout-r3 云端攻方 K1：派发给的上限写法包装不认（例 12GiB）时，15、74 号阶段各报成什么。
# 在临时目录里搭一个镜像根：.claude/gate.d/ 放今天仓里那两份阶段的拷贝，research/scripts 是指到仓里 research/scripts 的符号链接，
# 另有一个空的 Cargo 工作区（research/Cargo.toml、Cargo.toml、crates/singlefs-harness/）。上限写法错时包装在起 scope 之前就退 2，
# cargo 一行都不跑；不带单位的「16」照 systemd 写法是 16 字节，包装起 scope 时外壳自己就被停掉，同样一行 cargo 都不跑。
# 峰值表指到临时目录里的私有一份（RUN_WITH_MEMORY_CAP_PEAKS），不碰主仓那一份。
# 用法：bash stage-cap-syntax-demo.sh [临时目录]
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
scratch="$(mktemp -d "${1:-${TMPDIR:-/tmp}}/cap-syntax-demo.XXXXXX")"
mirror="$scratch/mirror"
mkdir -p "$mirror/.claude/gate.d" "$mirror/research" "$mirror/crates/singlefs-harness"
cp "$root/.claude/gate.d/15-research-build.sh" "$root/.claude/gate.d/74-model-differential.sh" "$mirror/.claude/gate.d/"
ln -s "$root/research/scripts" "$mirror/research/scripts"
printf '[workspace]\nmembers = []\n' > "$mirror/research/Cargo.toml"
printf '[workspace]\nmembers = []\n' > "$mirror/Cargo.toml"
export RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv"
for cap in 12GiB 16GB 8g 16; do
  echo "== 15 号，GATE_RESEARCH_BUILD_MEMORY_MAX=$cap"
  GATE_RESEARCH_BUILD_MEMORY_MAX="$cap" bash "$mirror/.claude/gate.d/15-research-build.sh" "$mirror" 2>&1
  echo "exit=$?"
  echo "== 74 号，GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=$cap"
  GATE_MODEL_DIFFERENTIAL_MEMORY_MAX="$cap" SINGLEFS_GATE_FULL=1 bash "$mirror/.claude/gate.d/74-model-differential.sh" "$mirror" 2>&1
  echo "exit=$?"
done
for cap in 12GiB 16; do
  echo "== 包装本身，上限 $cap（命令是 true）"
  bash "$root/research/scripts/run-with-memory-cap.sh" "$cap" true; echo "exit=$?"
done
rm -rf -- "${scratch:?}"
