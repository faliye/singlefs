#!/usr/bin/env bash
# 样本：登记成 heavy-if-set:SINGLEFS_HEAVY_TESTS 的自证。变量设着就是真家伙会起重活的那一支：写标记文件 heavy-if-set-ran.marker、退 1；没设走假驱动，退 0
[[ "${1:-}" == --selftest ]] || exit 2
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  : > "$PWD/heavy-if-set-ran.marker"
  echo "  ✗ 样本自证看到了 SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS}：真驱动会在这里起 checker 档 cargo test 与 multi-host-run.sh --peer"
  exit 1
fi
echo "  ✓ 样本自证按假驱动跑完，没起重活"
exit 0
