#!/usr/bin/env bash
# gate-stage: 样本：不带编号的阶段，真的跑 ran-unnumbered.sh 的自证（阶段只认两位数开头时它被漏掉，ran-unnumbered.sh 就被报成没人跑）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
bash research/scripts/ran-unnumbered.sh --selftest || exit 1
