# 这一轮的用户定案（2026-09-20）

判决正文写 `research/prompts/d13-item4-fua-r1-main-verification.md`，这一份只原样记用户当场定的几条，供写回 kb 时引。

## 一、实现要不要真的发 `REQ_FUA`：先按保守的走

**用户原话**：

> 实现要不要真的发 REQ_FUA。  先按保守的策略走。优化的事情我们后面再说。

**背景**（主 agent 2026-09-20 现查，正推腿的附带发现引出）：`crates/singlefs-core/src/block_device.rs` 两份 `BlockDevice` 实现里，`WriteDurability::ForceUnitAccess`（`:245`、`:449`）与 `barrier()`（`:251`、`:455`）调的是同一个 `file.sync_data()`。块设备 fd 上 `sync_data` 走 `blkdev_fsync` → `blkdev_issue_flush`，发出的是一个零长度的 `REQ_OP_WRITE | REQ_PREFLUSH` bio（`block/fops.c:590`、`:605` 与 `block/blk-flush.c:474`，读的是本机 `/home/fy5090/linux-bug-fix/linux`，`git describe` 报 v7.3-rc1；契约那份文档读的是 6.17 树 `/home/fy5090/code/fs-refs/linux-6.17/Documentation/block/writeback_cache_control.rst`，两棵树不是一棵）。`write_all_at` 走 `pwrite`，不设 `REQ_FUA`。

所以本工程至今没有向任何设备发出过一次 `REQ_FUA` 写：`ForceUnitAccess` 的实际行为是**写完再整盘刷一次**，与屏障同形，比真 FUA 强（真 FUA 只保它自己那一次写，整盘刷把此前已完成的写也一起保了）。

**定案**：维持今天的做法（写 + `sync_data`），不改成发真 `REQ_FUA`。

**代价，知情接受**：每次发布是「刷 → 写根 → 再刷」两次整盘刷；真 FUA 只要「刷 → 带 FUA 写根」一次。省掉的那一次整盘刷是性能项，归以后的优化，不在这一轮。E77（发布的持久顺序） 那句「要不要为此付第二道屏障，是决策不是实验」说的是同一条线上的事。

**这条定案对 D13（验证路线） 已定项 4 的直接后果**：那半句写的是「带 FUA 的写」，而代码里没有这种写。保守策略定下来之后，条款要描述的对象就是 `WriteDurability::ForceUnitAccess` 这种「写 + 整盘刷」，措辞按它写。

## 二、测试周期的边界（同日定，记在这里便于一起写回）

**用户原话**（分两次）：

> 我说的随机是 多次实验的随机  比如 里程碑1 和里程碑2  而不是 每个里程碑里面都要随机

> 崩溃重放 只要在一个测试周期内一致就可以了 没有必要里程碑5和里程碑1的一致，这个也不现实

**定案**：一个测试周期从「开一个新里程碑」与「用户显式说从零开始测试 / 重建测试」两件里**先到的那一件**开始。周期之内种子基写死，周期之间重抽。

**周期开始要做三样**：重抽种子基；把以前的数据盘、虚拟盘镜像全删掉重建；变异表整表在新基上重验一遍。

**这一周期（里程碑「第二个事务」）的基**：`7463871032432355113`，2026-09-20 用 `python3 -c "import secrets; print(secrets.randbits(63))"` 抽，落在 `crates/singlefs-harness/src/crash_injection.rs:70` 的 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`。

## 三、可以拿真设备

**用户原话**：

> 可以拿真设备 没什么问题

射程：C6（块层语义假设写错） 欠的那一半（拿虚机的设备侧日志重建崩溃状态再跑记录核对器）可以动用真设备。读数与设备型号对比在 `d13-item4-fua-r1-device-readings.md` 与 `d13-item4-fua-r1-device-model-readings.md`。

⚠️ 主 agent 当时据此说过「换成 `-device nvme` 那个危险状态就变得可达」，**这句说宽了**，已在判决里更正：程序不发 `REQ_FUA`，换设备型号不改变它发出去的请求流。设备型号的两处读数本身照旧为真。
