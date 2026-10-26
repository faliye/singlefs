# scripts/devices

虚机、QEMU 与真块设备上的装置与探针。起虚机、真编 QEMU 是重型（`.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」）。

| 文件 | 做什么 |
|---|---|
| `vm-bench.sh`、`vm-kernel.sh`、`vm-geom.sh` | 在 QEMU/KVM 虚机里给静态二进制一块真块设备、找可读的内核、造不同几何的 NVMe |
| `build-patched-qemu.sh`、`qemu-blklogwrites-check.sh`、`qemu/` | 编打了补丁的 QEMU（补丁在 `qemu/`），核 blklogwrites 记全多段写 |
| `e152-run.sh`、`e152-stage-root.sh` | E152（按里程碑对比六家文件系统的文件性能）的跑法与 initramfs 附加根 |
| `e34-iomin-probe.sh`、`e72-devtable-probe.sh`、`e125-zoned-wp-probe.sh`、`e129-*.sh`、`stripe-map-probe.sh` | 各实验在真设备上的探针 |
| `snapshot-check.sh` | 三方开工快照的逐文件核对 |
| `dm-owned.sh` | 探针共用的 dm 创建登记与清理，只移除本次成功创建的映射 |
| `tests/test_devices.py` | 设备脚本的失败路径回归测试，使用临时文件和替身命令，不操作真实设备 |

轻量回归：`python3 scripts/devices/tests/test_devices.py`（需要 `dpkg-deb`，只构建和解包临时测试包，不安装）。覆盖创建失败与清理归属、清理失败后的资源保留、写入失败、E152 路径与产物发布、软件包校验范围、读取失败与短读、快照非法记录、参数边界和几何记录完整性；不替代真实设备或 KVM 验证。

`vm-bench.sh` 的 `VM_IOPS` 限速模型只支持单盘，不与 `VM_BLKLOGWRITES_DIR` 组合；不支持的配置在启动前报错。两个 thin 探针清理失败时返回非零并打印保留目录与 loop 句柄，后备文件保留到设备成功释放，不能直接删除仍被设备使用的目录。
