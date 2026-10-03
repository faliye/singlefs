#!/usr/bin/env bash
# 攻方 J2-c：基线那一步写盘多（页缓存）时，包装记的峰值顶到它的上限；升档撞顶的上限 = max(1.5 倍原上限, 2 倍基线峰值) = 2 倍原上限，
# 放不进 slice 总上限就只剩 254。临时 slice 总上限 160M：① 上限 96M 里写 400M 的文件（盘在 ext4，每 8M fsync 一次，页缓存回收得掉，命令退 0），看记下的峰值；
# ② 同一个上限里无界分配（撞顶 250）；③ 照驱动的式子算升档上限，用它再起一次。用法：j2c-pagecache.sh <树根> <草稿目录>
set -uo pipefail
tree="$1"; work="$2"; wrap="$tree/research/scripts/run-with-memory-cap.sh"; driver="$tree/research/scripts/broad-mutation.py"
rm -rf "${work:?}/j2c-pc"; mkdir -p "$work/j2c-pc"; cd "$work/j2c-pc"
export RUN_WITH_MEMORY_CAP_SLICE=singlefs-r3attack.slice RUN_WITH_MEMORY_CAP_SLICE_TOTAL=160M RUN_WITH_MEMORY_CAP_STATE_DIR="$work/j2c-pc/state" \
       RUN_WITH_MEMORY_CAP_PEAKS="$work/j2c-pc/peaks.tsv" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="照驱动真跑带前缀时不判这一条"
RUN_WITH_MEMORY_CAP_KEY="broad-mutation baseline two-test-slow-harness" bash "$wrap" 96M python3 -c '
import os
with open("big.bin", "wb") as handle:
    for _ in range(50):
        handle.write(os.urandom(8 << 20)); handle.flush(); os.fsync(handle.fileno())
' 2> baseline.err; echo "① 基线（写 400M 的文件）exit=$?"
peak=$(grep -P '\tbroad-mutation baseline two-test-slow-harness' peaks.tsv | cut -f1); echo "   记下的峰值 $peak 字节（上限 96M = $((96 << 20)) 字节）"; rm -f big.bin
hog='c=[]
while True:
    b=bytearray(8<<20); b[::4096]=b"\x01"*(len(b)//4096); c.append(b)'
RUN_WITH_MEMORY_CAP_KEY="broad-mutation two-test-slow-harness" bash "$wrap" 96M python3 -c "$hog" 2> first.err; echo "② 变异那一趟 exit=$?"
up=$(python3 - "$driver" "$peak" <<'PY'
import importlib.util, math, sys
spec = importlib.util.spec_from_file_location("d", sys.argv[1]); d = importlib.util.module_from_spec(spec); spec.loader.exec_module(d)
first = d.memory_cap_bytes("96M"); peak = int(sys.argv[2])
wanted = max(d.ESCALATION_MEMORY_FACTOR * first, d.ESCALATION_MEMORY_BASELINE_MULTIPLE * peak)
print(f"{int(math.ceil(wanted / d.MEMORY_CAP_UNITS['M']))}M")
PY
)
echo "   照 escalate_step 的式子升档上限 = $up"
RUN_WITH_MEMORY_CAP_KEY="broad-mutation two-test-slow-harness" bash "$wrap" "$up" python3 -c "$hog" 2> escalated.err; echo "③ 升档那一趟 exit=$?"
grep -o 'run-with-memory-cap: [^（]*' escalated.err | head -1
