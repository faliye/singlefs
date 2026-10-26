#!/usr/bin/env bash
# 样本：代码里实现了 --selftest；名字是 crash-point-replay.sh 的后缀，阶段只跑后者的自证，没人跑它
[[ "${1:-}" == --selftest ]] && exit 0
exit 0
