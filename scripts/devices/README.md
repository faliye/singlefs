# scripts/devices

虚机、QEMU 与真块设备上的装置与探针。起虚机、真编 QEMU 是重型（`.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」）。

| 文件 | 做什么 |
|---|---|
| `vm-bench.sh`、`vm-kernel.sh`、`vm-geom.sh` | 在 QEMU/KVM 虚机里给静态二进制一块真块设备、找可读的内核、造不同几何的 NVMe |
| `build-patched-qemu.sh`、`qemu-blklogwrites-check.sh`、`qemu/` | 编打了补丁的 QEMU（补丁在 `qemu/`），核 blklogwrites 记全多段写 |
| `e152-run.sh`、`e152-stage-root.sh` | E152（按里程碑对比六家文件系统的文件性能）的跑法与 initramfs 附加根 |
| `e34-iomin-probe.sh`、`e72-devtable-probe.sh`、`e125-zoned-wp-probe.sh`、`e129-*.sh`、`stripe-map-probe.sh` | 各实验在真设备上的探针 |
| `snapshot-check.sh` | 三方开工快照的逐文件核对 |
