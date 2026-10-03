#!/bin/bash
# 复跑：在一个新的草稿目录里拷冻结副本、装上原型、编译、跑全部产物。
# 用法：bash rebuild-and-run.sh <新草稿目录> [冻结副本根，默认 /tmp/claude-1000/m3-sweep-and-tree-id-r1/tree]
# 线程上限 6，编译内存上限 3G，每次跑 2G；输出在 <新草稿目录>/out/。
set -u
D=$1; SRC=${2:-/tmp/claude-1000/m3-sweep-and-tree-id-r1/tree}
MODEL=$(cd "$(dirname "$0")/.." && pwd)
REPO_SCRIPTS=/home/fy5090/code/singlefs/research/scripts
mkdir -p "$D/repo" "$D/out"
rsync -a --exclude target --exclude .git "$SRC/" "$D/repo/"
mkdir -p "$D/repo/crates/sweep-proto"
cp -r "$MODEL/sweep-proto/." "$D/repo/crates/sweep-proto/"
python3 - "$D/repo/Cargo.toml" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
if '"crates/sweep-proto"' not in s:
    s=s.replace('    "crates/singlefs-reference-filesystem",\n]','    "crates/singlefs-reference-filesystem",\n    "crates/sweep-proto",\n]')
open(p+'.tmp','w').write(s)
PY
mv "$D/repo/Cargo.toml.tmp" "$D/repo/Cargo.toml"
cd "$D/repo"
RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿原型复跑，与同会话别的腿各跑各的" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1800 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑原型从头编译，峰值随机器负载不同" CARGO_TARGET_DIR="$D/target" nice -n 19 bash $REPO_SCRIPTS/run-with-memory-cap.sh 3G bash $REPO_SCRIPTS/capped.sh 6 cargo build --offline --release -p sweep-proto > "$D/build.log" 2>&1
echo "build exit=$?"
run() { local out=$1; shift; local bin=$1; shift
  ( cd "$D" && RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方攻方腿原型复跑，与同会话别的腿各跑各的" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1800 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一二进制换臂峰值不同" nice -n 19 bash $REPO_SCRIPTS/run-with-memory-cap.sh 2G bash $REPO_SCRIPTS/capped.sh 1 "$D/target/release/$bin" "$@" > "out/$out.out" 2> "out/$out.err"; echo "rc=$?" >> "out/$out.out" ); }
for a in control high foreign same65536 inode; do run fh-$a fake_header_tree_id $a; done
for arm in sweep noerase carry; do run sc-probe-$arm sweep_crash 4 5 $arm probe; run sc-states-$arm sweep_crash 4 5 $arm states; done
for pl in live freed; do for rd in all skipalloc claim ringref; do run sf-$pl-$rd sweep_fake $pl $rd; done; done
for k in 1 2 4 8; do run ss-$k-reverify sweep_stale $k reverify; done
for k in 1 2 4 5 6 7 8 9 10 12 16; do run ss-$k-stale sweep_stale $k stale; done
for a in orphan_ff applied_ff rollback fake_live fake_freed c342; do run th-$a tree_id_histories $a; done
C342_ROLL_BACK_SYSCONF=1 run th-c342-sysconf tree_id_histories c342
for e in 6 9; do C342_ROLL_BACK_SYSCONF=1 C342_EXTRA=$e run th-c342-sysconf-x$e tree_id_histories c342; done
echo "runs=$(ls "$D"/out/*.out | wc -l)"
