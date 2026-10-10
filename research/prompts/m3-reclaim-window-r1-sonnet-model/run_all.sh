#!/usr/bin/env bash
# 复现这份报告里的全部数。在本目录里跑：nice -n 19 bash run_all.sh   （约 1 分钟，纯 Python，无 numpy）
# 输出落进 out/；out/ 下已有的文件会被 `>` 覆盖，所以先 `rm -rf out` 再跑。
set -u
export PYTHONDONTWRITEBYTECODE=1
cd "$(dirname "$0")"
mkdir -p out
J=16
P="python3 e139_port.py --jobs $J"
echo "== 一、复现 E139 第 1 次重跑产物里的行（G2SW、G2SW_ringgate 的 w_activation 各 60 行）"
$P --cfg gate=window > out/base_window.txt;  python3 cmp_rows.py out/base_window.txt G2SW_tightened_states_window | tail -n 1
$P --cfg gate=ring   > out/base_ring.txt;    python3 cmp_rows.py out/base_ring.txt G2SW_ringgate | tail -n 1
echo "== 二、W2：装置上一次只改一处（改成条款 / 实现的形）"
for c in p2=True p3=True p3c=True p3s=True p4=True "p2=True,p3=True,p4=True" failure=resend recover=replay; do
  n=$(echo "$c" | tr ',=' '__'); $P --cfg "gate=window,$c" > "out/flip_$n.txt"; echo "$c : $(tail -n 1 out/flip_$n.txt)"
done
echo "== 三、W3：回收门槛候选在 D23 目标形（推进 txg）的活化一族上"
for g in window ring alow alowr ext2 ext5 ext8 holes; do
  for c in "" ",p3c=True"; do n=$(echo "$g$c" | tr ',=' '__'); $P --cfg "gate=$g$c" > "out/gate_$n.txt"; echo "gate=$g$c : $(tail -n 1 out/gate_$n.txt)"; done
done
echo "== 四、W3：近满盘两个世界（drain_failure、broken_slot）上的代价"
for g in window ring alow alowr holes ext5 ext8; do python3 run_nearfull.py gate=$g > out/nf_$g.txt; done
python3 cmp_nf.py out/nf_window.txt G2SW_tightened_states_window | tail -n 1
python3 cmp_nf.py out/nf_ring.txt G2SW_ringgate | tail -n 1
echo "== 五、W1：实现形的 K1 世界（崩在记录之后、根槽之前，可写挂载由记录施加，再在每个发布点崩 + 丢盘）"
for dh in 0 1; do for g in window ring alow alowr ext2 ext5 holes; do
  python3 k1_world.py --cfg gate=$g --double $dh > out/k1_double${dh}_$g.txt; echo "double_hole=$dh gate=$g : $(grep '^# total' out/k1_double${dh}_$g.txt)"
done; done
echo "== 六、探针"
python3 probe_p2_p4_effect.py > out/probe_p2_p4_effect.txt; cat out/probe_p2_p4_effect.txt
python3 probe_p2_ceiling_calls.py > out/probe_p2_ceiling_calls.txt; cat out/probe_p2_ceiling_calls.txt
python3 probe_unrecoverable_cases.py > out/probe_unrecoverable_cases.txt; tail -n 3 out/probe_unrecoverable_cases.txt
python3 probe_gate_cost_after_hole.py > out/probe_gate_cost_after_hole.txt; sed -n 1,4p out/probe_gate_cost_after_hole.txt
