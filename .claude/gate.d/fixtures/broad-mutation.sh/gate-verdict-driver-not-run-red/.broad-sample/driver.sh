#!/usr/bin/env bash
# 样本驱动：没跑成（退 1）。门禁要判红、不往下读 summary.sh；不起驱动的门禁会读到全绿的 summary.sh 判绿
echo "样本驱动：没跑成" >&2
exit 1
