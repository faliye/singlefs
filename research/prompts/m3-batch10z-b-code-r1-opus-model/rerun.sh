#!/usr/bin/env bash
# 复跑攻方原型（m3-batch10z-b-code-r1 攻方腿）。用法：
#   SINGLEFS_FROZEN=<冻结副本> SINGLEFS_TOOLS=<仓根> bash rerun.sh <工作目录>
# 在工作目录里拷冻结副本、打 src/opus-arms-and-mutations.patch（环境变量切臂与变异：OPUS10ZB_ARM=today|slot|cring，OPUS_MUT=m6|m7|m11；
# 不设时与冻结副本同行为）、放进两份探针、编，按 scripts/ 里的规格分批跑，最后出 out/counts.txt 与 out/k3.txt，与模型目录里的比。
# 跑的都经 run-with-memory-cap.sh 8G 与 capped.sh 8；每批 6 件（X1、F1，单件单线程）或 4 件（K3，每件 2 线程）并行。
set -euo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
tools=${SINGLEFS_TOOLS:?仓根}
frozen=${SINGLEFS_FROZEN:?冻结副本}
mkdir -p "$work/out"
rsync -a --exclude target --exclude .git "$frozen/" "$work/repo/"
(cd "$work/repo" && patch -p0 < "$here/src/opus-arms-and-mutations.patch")
cp "$here/src/opus10zb_probe_abandoned_slot.rs" "$here/src/opus10zb_probe_form1.rs" "$work/repo/crates/singlefs-harness/tests/"
targets="--test opus10zb_probe_abandoned_slot --test opus10zb_probe_form1 $(cat "$here/scripts/k3-targets.txt")"
(cd "$work/repo" && nice -n 19 bash "$tools/research/scripts/capped.sh" 8 cargo test --no-run --offline -p singlefs-harness $targets --target-dir "$work/target")
# 规格里的二进制名带编译散列：换成这一次编出来的。
for spec in "$here"/scripts/spec-*.tsv; do
  out="$work/$(basename "$spec")"
  : > "$out"
  while IFS=$'\t' read -r name bin rest; do
    stem=${bin%-*}
    real=$(ls -t "$work/target/debug/deps/" | grep -E "^${stem}-[0-9a-f]+$" | head -1)
    printf '%s\t%s\t%s\n' "$name" "$real" "$rest" >> "$out"
  done < "$spec"
done
sed "s#^D=.*#D=$work#" "$here/scripts/batch.sh" > "$work/batch.sh"
sed "s#^D=.*#D=$work#" "$here/scripts/batch2.sh" > "$work/batch2.sh"
run() { nice -n 19 bash "$tools/research/scripts/run-with-memory-cap.sh" 8G bash "$tools/research/scripts/capped.sh" 8 bash "$@"; }
cd "$work"
split -l 6 -d spec-b6.tsv spec-b6-part
for p in spec-b6-part*; do run "$work/batch.sh" "${p#spec-}" "$work/$p"; done
for b in b1 b2 b3 b4; do run "$work/batch.sh" "$b" "$work/spec-$b.tsv"; done
for m in base m6 m7 m11; do
  split -l 4 -d "spec-k3-$m.tsv" "spec-k3-$m-part"
  for p in spec-k3-"$m"-part*; do run "$work/batch2.sh" "${p#spec-}" "$work/$p"; done
done
python3 "$here/scripts/count.py" "$work" > "$work/out/counts.txt"
for f in "$work"/out/k3-*.log; do echo "$(basename "$f") $(grep -h 'test result' "$f" | sed 's/; finished.*//')"; done > "$work/out/k3.txt"
diff "$work/out/counts.txt" "$here/out/counts.txt" && echo "counts.txt 全同"
diff "$work/out/k3.txt" "$here/out/k3.txt" && echo "k3.txt 全同"
