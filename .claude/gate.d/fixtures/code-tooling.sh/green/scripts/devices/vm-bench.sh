#!/usr/bin/env bash
# 样本：代码里实现了 --selftest，登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里
# （只在注释里提 --selftest 的不算实现了，那一格会把 NOT_RUN_HERE 这一行判成过期）
[[ "${1:-}" == --selftest ]] && exit 0
exit 0
