#!/usr/bin/env bash
# gate-stage: 样本：只跑 crash-point-replay.sh 的自证
bash scripts/crash/crash-point-replay.sh --selftest || exit 1
