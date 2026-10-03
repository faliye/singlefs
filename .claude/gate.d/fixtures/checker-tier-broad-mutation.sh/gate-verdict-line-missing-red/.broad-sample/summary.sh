#!/usr/bin/env bash
# 样本：加剪枝之前的驱动只打八栏汇总、不打提交门禁的判定行，退 0
cat <<'EOF'
BROAD_MUTATION_SUMMARY fingerprint=970a802431217260 commit=0123456789ab total=6 caught=5 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0
  抓到 5、没红 0、无效 1、超时 0、内存撞顶 0、只过了第一段 0、驱动错 0、这一版还没跑 0；清单 6 条
EOF
exit 0
