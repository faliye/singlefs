#!/usr/bin/env bash
# 攻方 J2-c 旁支（不是这一批引入的）：一条无界分配的变异按峰值表里这个键上一条变异记的小峰值排进来，scope 上限却是 --memory-cap；
# slice 里另有一条更大的常驻（别的会话的重活）时，slice 撞总上限，内核杀的是谁。临时 slice 总上限 256M：常驻 150M（上限 160M），
# 峰值表里键 r3-coll 先记 20M（上一条变异的峰值），再起上限 200M 的无界分配。用法：j2c-collateral.sh <树根> <草稿目录>
set -uo pipefail
tree="$1"; work="$2"; wrap="$tree/research/scripts/run-with-memory-cap.sh"
rm -rf "${work:?}/j2c-coll"; mkdir -p "$work/j2c-coll"; cd "$work/j2c-coll"
export RUN_WITH_MEMORY_CAP_SLICE=singlefs-r3attack.slice RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$work/j2c-coll/state" \
       RUN_WITH_MEMORY_CAP_PEAKS="$work/j2c-coll/peaks.tsv" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30 RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿在自己的临时 slice 里量撞总上限时杀谁" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="驱动真跑带前缀不判这一条，这里照它"
RUN_WITH_MEMORY_CAP_KEY=r3-coll bash "$wrap" 64M python3 -c 'b=bytearray(16<<20); b[::4096]=b"\x01"*(len(b)//4096)' 2>/dev/null; echo "记小峰值那一趟 exit=$?"
grep -P '\tr3-coll' peaks.tsv | cut -f1,3,4
rm -f b3-*.rc
{ RUN_WITH_MEMORY_CAP_KEY=r3-resident bash "$wrap" 160M python3 -c 'import time
b=bytearray(150<<20); b[::4096]=b"\x01"*(len(b)//4096); time.sleep(20)' 2> resident.err; echo "$?" > b3-1.rc; } &
sleep 4
RUN_WITH_MEMORY_CAP_KEY=r3-coll bash "$wrap" 200M python3 -c 'c=[]
while True:
    b=bytearray(8<<20); b[::4096]=b"\x01"*(len(b)//4096); c.append(b)' 2> hog.err; echo "无界分配那一条 exit=$?"; grep -o 'run-with-memory-cap: [^（]*' hog.err | grep -v 照理由 | head -2
wait
n=$(ls b3-*.rc 2>/dev/null | wc -l); [[ $n -eq 1 ]] || echo "常驻那一条的退出码文件 $n 个，不是 1：这一批作废"
echo "常驻那一条（别的会话的重活）exit=$(cat b3-1.rc)"; grep -o 'run-with-memory-cap: [^（]*' resident.err | grep -v 照理由 | head -2
