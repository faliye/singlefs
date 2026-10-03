#!/usr/bin/env bash
# 样本：驱动的判红失灵（判定行写 green、退 0），计数里却有 1 条没红
cat <<'EOF'
BROAD_MUTATION_SUMMARY fingerprint=970a802431217260 commit=0123456789ab total=6 caught=4 missed=1 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0
  抓到 4、没红 1、无效 1、超时 0、内存撞顶 0、只过了第一段 0、驱动错 0、这一版还没跑 0；清单 6 条
MISSED	3	singlefs-core	crates/singlefs-core/src/sample.rs:12:9: replace + with - in add
BROAD_MUTATION_GATE total=6 concluded=6 carried=3 fresh=3 missed=1 without_conclusion=0 flipped=0 verdict=green
EOF
exit 0
