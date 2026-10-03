#!/usr/bin/env bash
# 样本：汇总行 total=5091、not_run=5085，判定行 total=6 写 green、退 0（两行判的不是同一份清单）
cat <<'EOF'
BROAD_MUTATION_SUMMARY fingerprint=970a802431217260 commit=0123456789ab total=5091 caught=5 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=5085
  抓到 5、没红 0、无效 1、超时 0、内存撞顶 0、只过了第一段 0、驱动错 0、这一版还没跑 0；清单 6 条
BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green
EOF
exit 0
