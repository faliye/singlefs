#!/usr/bin/env bash
# m3-formal-r4 攻方腿原型的复跑：在冻结副本的一份拷贝上加一个 crate（r4probe），编 release，逐段历史全量跑。
# 用法：bash rerun.sh <冻结副本 tree 目录> <草稿目录>
# 例：bash rerun.sh /tmp/claude-1000/m3-formal-r4/tree /tmp/claude-1000/<自己的草稿目录>
set -euo pipefail
frozen="$1"; scratch="$2"
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/tree/"
mkdir -p "$scratch/tree/crates/r4probe/src"
cp "$here/Cargo.toml" "$scratch/tree/crates/r4probe/Cargo.toml"
cp "$here/main.rs" "$scratch/tree/crates/r4probe/src/main.rs"
python3 - "$scratch/tree/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = '    "crates/singlefs-checker-tier",\n]'
assert old in s
open(p + '.new', 'w').write(s.replace(old, '    "crates/singlefs-checker-tier",\n    "crates/r4probe",\n]'))
PY
mv "$scratch/tree/Cargo.toml.new" "$scratch/tree/Cargo.toml"
cd "$scratch/tree"
CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" auto \
  bash "$repo/research/scripts/capped.sh" 10 cargo build --offline --release -p r4probe
cd "$scratch"
for h in s1 s5 s3 s2 s6 s10; do
  rc=0
  nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" 6G ./target/release/r4probe t5 "$h" 10 > "t5b-$h.log" 2>&1 || rc=$?
  echo "$h exit=$rc"
done
