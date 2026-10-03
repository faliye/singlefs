#!/usr/bin/env bash
# 样本：这一版清单 0 条，判定行写 green、退 0
cat <<'EOF'
BROAD_MUTATION_SUMMARY fingerprint=970a802431217260 commit=0123456789ab total=0 caught=0 missed=0 unviable=0 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0
  清单 0 条
BROAD_MUTATION_GATE total=0 concluded=0 carried=0 fresh=0 missed=0 without_conclusion=0 flipped=0 verdict=green
EOF
exit 0
