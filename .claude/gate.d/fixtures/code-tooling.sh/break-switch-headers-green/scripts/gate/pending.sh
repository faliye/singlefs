#!/usr/bin/env bash
# 样本脚本：读了开关，还没改成那种写法，登记在豁免表
if [[ "${PENDING_BREAK:-}" == skip ]]; then exit 0; fi
