#!/usr/bin/env bash
# 攻方 J2-c：真包装（research/scripts/run-with-memory-cap.sh）在自己的临时 slice 里（总上限 256M、账与锁、峰值表都在草稿目录），量两件事：
#  ① 包装真打的「超过限时 N 秒」「撞了内存上限 X」两句，喂给驱动的 escalation_evidence_problems（升档证据那一判）认不认；
#  ② 升档撞顶那一趟：排队按峰值表里这个键上一次记的量（头一趟撞顶记成它的上限）算，scope 的上限却是 1.5 倍；slice 里另有一条常驻 120M 时，
#     升档那趟涨过 slice 总上限，退几。用法：j2c-wrapper-probe.sh <树根> <草稿目录>
set -uo pipefail
tree="$1"; work="$2"; wrap="$tree/research/scripts/run-with-memory-cap.sh"; driver="$tree/research/scripts/broad-mutation.py"
rm -rf "${work:?}/j2c"; mkdir -p "$work/j2c"; cd "$work/j2c"
export RUN_WITH_MEMORY_CAP_SLICE=singlefs-r3attack.slice RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$work/j2c/state" \
       RUN_WITH_MEMORY_CAP_PEAKS="$work/j2c/peaks.tsv" RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿在自己的临时 slice 里量升档撞顶那一趟" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30
hog='import time
c=[]
while True:
    b=bytearray(8<<20); b[::4096]=b"\x01"*(len(b)//4096); c.append(b)'
run() {   # <标签> <上限> [环境…] -- <命令…>：打退出码与 stderr 里包装那几行
  local label="$1" cap="$2"; shift 2; local -a envs=(); while [[ "$1" != "--" ]]; do envs+=("$1"); shift; done; shift
  env "${envs[@]}" bash "$wrap" "$cap" "$@" 2> "$label.err"; local rc=$?
  echo "$label exit=$rc"; grep -o 'run-with-memory-cap: [^（]*' "$label.err" | head -3
}
echo "== ① 包装真打的两句"
run time-first 64M RUN_WITH_MEMORY_CAP_TIME_LIMIT=2 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 RUN_WITH_MEMORY_CAP_KEY=r3-time -- sleep 6
run time-escalated 64M RUN_WITH_MEMORY_CAP_TIME_LIMIT=4 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 RUN_WITH_MEMORY_CAP_KEY=r3-time -- sleep 9
run memory-first 64M RUN_WITH_MEMORY_CAP_KEY=r3-mem -- python3 -c "$hog"
run memory-escalated 96M RUN_WITH_MEMORY_CAP_KEY=r3-mem -- python3 -c "$hog"
python3 - "$driver" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("d", sys.argv[1]); d = importlib.util.module_from_spec(spec); spec.loader.exec_module(d)
read = lambda name: open(name + ".err", encoding="utf-8", errors="replace").read()
print("超时两趟（2 → 4 秒、基线 0.1 秒）证据缺：", d.escalation_evidence_problems(253, 253, 0.1, 2, 4, "2", "4", read("time-first"), read("time-escalated")) or "无（证据齐）")
print("撞顶两趟（64M → 96M、基线 32 MiB）证据缺：", d.escalation_evidence_problems(250, 250, 32 << 20, 64 << 20, 96 << 20, "64M", "96M", read("memory-first"), read("memory-escalated")) or "无（证据齐）")
PY
echo "== 峰值表里键 r3-mem 记的（头一趟、升档那趟撞顶都记成各自的上限）"; grep -P '\tr3-mem' peaks.tsv | cut -f1,3,4
echo "== ② slice 里另有一条常驻 120M（经包装、上限 130M）时，撞顶的头一趟 100M、升档那趟 150M"
rm -f b2-*.rc
{ bash "$wrap" 130M python3 -c 'import time
b=bytearray(120<<20); b[::4096]=b"\x01"*(len(b)//4096); time.sleep(25)' 2> filler.err; echo "$?" > b2-1.rc; } &
sleep 4
run slice-first 100M RUN_WITH_MEMORY_CAP_KEY=r3-slice -- python3 -c "$hog"
grep -P '\tr3-slice' peaks.tsv | cut -f1,3,4
run slice-escalated 150M RUN_WITH_MEMORY_CAP_KEY=r3-slice -- python3 -c "$hog"
wait
n=$(ls b2-*.rc 2>/dev/null | wc -l); [[ $n -eq 1 ]] || echo "常驻那一条的退出码文件 $n 个，不是 1：这一批作废"
echo "常驻那一条 exit=$(cat b2-1.rc)"; grep -o 'run-with-memory-cap: [^（]*' filler.err | head -2
