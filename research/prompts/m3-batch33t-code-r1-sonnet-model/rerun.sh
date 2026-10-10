#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<主仓根，取 research/scripts 的内存包装与 diff 文件> [SINGLEFS_FROZEN_TREE=<冻结副本根，默认 ~/.local/share/singlefs/work/m3-batch33t-code-r1/tree>] bash rerun.sh <空工作目录>
# 在空工作目录里：拷冻结副本成仓副本、放进探针测试、经 run-with-memory-cap.sh（编译与跑各 4G 以内、线程 10）跑探针 p1 p1b p1c p23 p4 p5 p6、
# 把追加的 18 行变异逐行打进副本再还原、静态核 38 行变异行；再做一次对照（拿掉 C 的候选豁免，control.py）；输出 <工作目录>/out/{probes,mutations,control,static}.txt。一次只起一条 cargo。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（主仓根）}"
tree="${SINGLEFS_FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-batch33t-code-r1/tree}"
work="${1:?要给一个空工作目录}"
if [ -e "$work" ] && [ -n "$(ls -A "$work")" ]; then echo "工作目录不空：$work" >&2; exit 2; fi
mkdir -p "$work/repo" "$work/out"
cp -r "$tree/." "$work/repo/"
cp "$here/probe/sonnet_probe_b33t.rs" "$work/repo/crates/singlefs-harness/tests/sonnet_probe_b33t.rs"
export CARGO_TARGET_DIR="$work/target"
wrap=(nice -n 19 bash "$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh")
capped=(bash "$SINGLEFS_REPO/research/scripts/capped.sh" 10)
cd "$work/repo"
"${wrap[@]}" 6G "${capped[@]}" cargo test -p singlefs-harness --test sonnet_probe_b33t --no-run > "$work/build.log" 2>&1
"${wrap[@]}" 4G "${capped[@]}" cargo test -p singlefs-harness --test sonnet_probe_b33t -- --nocapture --test-threads=1 > "$work/probes.log" 2>&1
grep -E '^(test .* PROBE|PROBE)' "$work/probes.log" | sed -E 's/^test [^ ]+ \.\.\. //' | sed -E 's/InstanceGeneration\(([0-9]+)\)/i\1/g; s/CheckpointTxg\(([0-9]+)\)/t\1/g' > "$work/out/probes.txt"
python3 -I "$here/mutrun.py" "$work/repo" "$SINGLEFS_REPO" 第33批T > "$work/out/mutations.txt"
python3 -I "$here/control.py" "$work/repo" "$SINGLEFS_REPO" | sed -E "s/InstanceGeneration\(([0-9]+)\)/i\1/g; s/CheckpointTxg\(([0-9]+)\)/t\1/g" > "$work/out/control.txt"
python3 -I "$here/static_rows.py" "$SINGLEFS_REPO/research/prompts/_m3-batch33t-code-r1-diff.md" "$tree" > "$work/out/static.txt"
echo "探针 $(wc -l < "$work/out/probes.txt") 行；变异 $(grep -c '^ROW ' "$work/out/mutations.txt") 行，其中 RED(expected) $(grep -c 'RED(expected test failed)' "$work/out/mutations.txt")；静态核：$(tail -n 1 "$work/out/static.txt")"
