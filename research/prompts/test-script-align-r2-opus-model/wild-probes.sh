#!/usr/bin/env bash
# U1 核心打中：命令词带通配（*、?、[）时，重型测试闸整条看不见——classify 按 basename 认不出、script_path_of 不读正文——
# 一律放行（退 0）。执行时 bash 把通配展开成真的重型目标跑起来。只喂 JSON、看退出码，被判命令一条都不起。
# 用法：wild-probes.sh <草稿目录>
set -uo pipefail
S="$1"; here="$(cd "$(dirname "$0")" && pwd)"; R="$(cd "$here/../../.." && pwd)"
D="$S/wild.jsonl"; mkdir -p "$S"; : > "$D"
p(){ python3 "$here/hook-probe.py" "$D" implementation-writer "$R" "$1"; }
echo "## 通配命令词放行真重型（退 0 = 放行；bash 展开后跑的是括号里那一份真的）"
p "bash .claude/gate.d/checker-tier-layer0-replay.s? $R"                      # → checker-tier-layer0-replay.sh：真层 0
p "bash .claude/gate.d/checker-tier-qemu-device-streams.s? $R"      # → 真 QEMU 那一道
p "bash research/scripts/mutate.s? crates/mutations.tsv"            # → mutate.sh 跑 crates 变异整表
p "bash research/scripts/vm-bench*.sh --selftest"                  # → vm-bench.sh --selftest：起虚机
p "bash research/scripts/crash-amplification-two-host?.sh --selftest"  # → 本机两份真 checker 档用例
p "python3 research/scripts/compile-then-swap.p? --selftest"       # → 真 cargo build / cargo test --no-run
echo "## 非通配对照（应拒）"
p "bash .claude/gate.d/checker-tier-layer0-replay.sh $R"
p "bash research/scripts/vm-bench.sh --selftest"
p "cargo test -p singlefs-checker-tier"
