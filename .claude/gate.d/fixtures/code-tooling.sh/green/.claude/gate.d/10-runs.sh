#!/usr/bin/env bash
# 样本不是真阶段；stage-owners 格要每个阶段认第一个参数当项目根，这一行只为它：ROOT="${1:-.}"
# gate-stage: 样本：真的跑 ran.py 的自证
python3 research/scripts/ran.py --selftest || exit 1
