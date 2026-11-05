#!/usr/bin/env bash
# 样本：驱动自己连第二台（门禁 code-tooling 的 peer-launch-direct 格判红）
MULTI_HOST_RUN="${SAMPLE_MULTI_HOST_RUN:-scripts/hosts/multi-host-run.sh}"
bash scripts/hosts/multi-host-run.sh --peer --label sample -- bash scripts/experiment-apparatus/driver.sh --share 1/2
bash "$MULTI_HOST_RUN" --peer -- true
