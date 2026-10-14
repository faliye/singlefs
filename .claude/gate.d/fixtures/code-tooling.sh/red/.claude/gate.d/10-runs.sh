#!/usr/bin/env bash
# gate-stage: 样本：真的跑 ran.py 的自证
python3 scripts/gate/ran.py --selftest || exit 1
