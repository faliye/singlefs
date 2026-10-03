#!/usr/bin/env bash
# 复跑差分：从两份冻结副本生成核，装进 <工作树>（第三轮冻结副本的一份拷贝，rsync 来、不带 target）当工作区成员，编 release，跑四个族。
# 用法：rerun.sh <第二轮冻结副本根> <第三轮冻结副本根> <工作树> <出口目录>；内存包装 512M、线程 10。
set -eu
old_root=$1; new_root=$2; tree=$3; out=$4
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../../.." && pwd)
mkdir -p "$out"
rsync -a --exclude kernels "$here/" "$tree/crates/r3-attack-kernel-diff/"
python3 "$here/make-kernels.py" "$old_root" "$new_root" "$tree/crates/r3-attack-kernel-diff/src/kernels"
grep -q '"crates/r3-attack-kernel-diff"' "$tree/Cargo.toml" || python3 - "$tree/Cargo.toml" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
marker = '    "crates/singlefs-checker-tier",\n]'
assert text.count(marker) == 1
open(path, "w", encoding="utf-8").write(text.replace(marker, '    "crates/singlefs-checker-tier",\n    "crates/r3-attack-kernel-diff",\n]', 1))
PY
( cd "$tree" && nice -n 19 bash "$repo/research/scripts/capped.sh" 10 cargo build --offline --release -p r3-attack-kernel-diff )
for family_arguments in "exhaustive 0 0" "random 5000000 1" "chain 5000000 2" "drops 5000000 3"; do
    set -- $family_arguments
    nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" 512M bash "$repo/research/scripts/capped.sh" 10 \
        "$tree/target/release/r3-attack-kernel-diff" "$1" "$2" "$3" 10 > "$out/$1.txt" 2> "$out/$1.err"
done
