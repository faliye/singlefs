# d13-item4-fua-r1 云端正推（Sonnet）判定：Q1、Q2

攻击面：正文第五节 Q1（契约读对了没有）、Q2（改后的措辞说全了没有）。
不碰 Q3、Q5（云端攻方）、Q4（本地辩方）。行号均现查，未现查的标「行号待查」。

## 一、Q1：契约读对了没有

### 1.1 kb 决策今天的行号（现查）

`.claude/kb/decisions/13-验证路线.md`：

```
$ grep -n "带 FUA 的写也是段边界" .claude/kb/decisions/13-验证路线.md
71:**定案**：崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。……
```

行 71 与背景材料第 9 行抄的一致，不是转述。

### 1.2 两段块层原文，现查行号

`/home/fy5090/code/fs-refs/linux-6.17/Documentation/block/writeback_cache_control.rst`（Linux 6.17，本机路径）：

```
$ grep -n "REQ_PREFLUSH flag can be\|REQ_FUA flag can be" Documentation/block/writeback_cache_control.rst
23:The REQ_PREFLUSH flag can be OR ed into the r/w flags of a bio submitted from
36:The REQ_FUA flag can be OR ed into the r/w flags of a bio submitted from the
```

原文整段（23-27 行，PREFLUSH）：

> The REQ_PREFLUSH flag can be OR ed into the r/w flags of a bio submitted from
> the filesystem and will make sure the volatile cache of the storage device
> has been flushed before the actual I/O operation is started.  This explicitly
> guarantees that previously completed write requests are on non-volatile
> storage before the flagged bio starts.

原文整段（36-38 行，FUA）：

> The REQ_FUA flag can be OR ed into the r/w flags of a bio submitted from the
> filesystem and will make sure that I/O completion for this request is only
> signaled after the data has been committed to non-volatile storage.

**判定：这两段原文只支持窄读法。** PREFLUSH 段明写「previously completed write requests are on non-volatile storage **before the flagged bio starts**」——这句话把「之前已完成的写持久」这个承诺**明确挂在 PREFLUSH 头上**，而 FUA 段整段只说「this request」（它自己）的数据何时算持久，一个字都没提前面的写。两段分工清楚：PREFLUSH 管「之前」，FUA 管「这一次」。**能推翻这个判定的现象**：如果 FUA 段原文里出现类似「and any preceding write is also guaranteed durable」这样的从句——今天没有，95 行全文只有这两段谈持久化时序（`.claude/kb/decisions/20-承重面单元的原子性与自包含.md:137` 已现查过同一份文档「全文没有 atomic / torn / tear」，本轮现查确认这两段也没有把 FUA 与「之前的写」连起来的从句）。

### 1.3 两个标志可以同设一个 bio，不改变上面的分工

```
$ sed -n '44,47p' Documentation/block/writeback_cache_control.rst
44:Filesystems can simply set the REQ_PREFLUSH and REQ_FUA bits and do not have to
45:worry if the underlying devices need any explicit cache flushing and how
46:the Forced Unit Access is implemented.  The REQ_PREFLUSH and REQ_FUA flags
47:may both be set on a single bio.
```

这句只说「可以同设」，不是「FUA 单独设置时也顺带具备 PREFLUSH 的效果」。要让 FUA 那次写连带把前面的写也变持久，必须**额外**设 PREFLUSH——这恰好印证窄读法：两个标志各管各的，混在一起用才是「这次连同前面一起」。


### 1.4 virtio-blk 规范：查得到，给得比块层契约更少，不是更多

外部规范原文不在本仓、也不在 kb，本轮现取（原样贴出，未转述）：

```
$ curl -sL -A "Mozilla/5.0" -o /tmp/virtio-spec.html \
    https://docs.oasis-open.org/virtio/virtio/v1.3/csd01/virtio-v1.3-csd01.html
$ grep -io -c "FUA" /tmp/virtio-spec.html
0
$ grep -io -n "force unit access" /tmp/virtio-spec.html
(无命中)
```

virtio 1.3 规范（OASIS 正式稿，2026-09-20 下载）全文对「FUA」「Force Unit Access」**零命中**。命令类型清单原文（对应本机 `/home/fy5090/linux-bug-fix/linux/include/uapi/linux/virtio_blk.h:163-211` 的内核实现，两处逐字对得上——只有 `VIRTIO_BLK_T_IN` / `_OUT` / `_SCSI_CMD` / `_FLUSH` / `_DISCARD` / `_WRITE_ZEROES` / `_GET_ID` / `_SECURE_ERASE` 等，没有任何 FUA 类型）：

> The type of the request is either a read (VIRTIO_BLK_T_IN), a write
> (VIRTIO_BLK_T_OUT), a discard (VIRTIO_BLK_T_DISCARD), a write zeroes
> (VIRTIO_BLK_T_WRITE_ZEROES), a flush (VIRTIO_BLK_T_FLUSH), a
> get device ID string command (VIRTIO_BLK_T_GET_ID), a secure erase
> (VIRTIO_BLK_T_SECURE_ERASE), or a get device lifetime command
> (VIRTIO_BLK_T_GET_LIFETIME).

**判定**：virtio-blk 协议本身没有 FUA 概念，谈不上「给得比块层契约更多」——它连块层契约给的那一点（FUA 只保证自己）都没有独立表达能力。Linux 客户端驱动 `drivers/block/virtio_blk.c` 印证这一点：全文件 grep `FUA`/`BLK_FEAT_FUA` 零命中（只设 `BLK_FEAT_WRITE_CACHE`），所以块层从不把 `REQ_FUA` 透传给 virtio-blk 设备，而是走「写完成之后块层补发一个 `REQ_OP_FLUSH`」这条模拟路径（`writeback_cache_control.rst:92-95`，背景材料第 58 行已引）。这与 `d13-item4-fua-r1-device-model-readings.md` 现测的 `fua=0`（virtio-blk-pci、virtio-scsi-pci 均 0，`-device nvme` 才是 1）完全对得上：**virtio-blk 上今天观测到的「FUA 顺带把前面的写也刷了」，是块层对「设备不支持原生 FUA」的一种补偿性模拟，不是 virtio-blk 规范写的承诺**——这正是背景材料 E77（发布的持久顺序）「它答不了的」一节点名的「真实设备 FLUSH 是否生效归 C6」那句话的一个具体落点，但这属于 Q5 的射程（真机观测该取哪一半），本节只回答「virtio-blk 规范文本本身给不给得更多」：不给。


### 1.5 NVMe 规范、SCSI SBC：查不到，如实报告，不臆造

尝试现查两份官方规范原文，均被访问闸挡住，本轮没有继续：

- NVMe Base Specification（nvmexpress.org）：`curl` 拿到的是一个 Gravity Forms 网关页（`<!doctype html>` 起手，非 PDF），要求先填表单才放行下载，没有绕过。
- SCSI SBC（t10.org）：`https://www.t10.org/cgi-bin/ac.pl?t=f&f=sbc4r22.pdf` 返回的是 t10.org 自己的「File Access Monitor」访客登记页，要求填姓名、机构、邮箱等个人信息才放行（Guest Access 表单）。**没有提交**：这需要把用户的个人信息交给一个与本轮任务无关的外部机构，而任务给的约束是「用户邮箱只用于识别用户本人，不发给无关服务」，填这张表等同把使用者的身份信息注册进 T10 的访客名单，未获用户明确同意，不做。

**这是如实的「查不到」，不是「两家没有更强的承诺」**——按 `verify-before-claiming.md`「陈述外部状态之前现查一次」，查不到就不能把「举不出」写成「不存在」。可核的间接证据只有 Linux 驱动源码（`/home/fy5090/linux-bug-fix/linux/drivers/nvme/host/core.c:1025-1026` 把 `REQ_FUA` 翻成 `NVME_RW_FUA` 控制位；`drivers/scsi/sd.c:1475` 把 `REQ_FUA` 翻成 CDB 里的 `fua` 位、`:3231` 读 mode page 的 `DPOFUA`）——这是**内核对协议的翻译代码**，不是规范原文，按 `implementation-first.md`「别的项目怎么做，是线索不是证据」不能当规范原文引用，也答不出「NVMe/SCSI 是否给得比块层契约更多」这句话。**结论：Q1 这一格对 NVMe、SCSI SBC 两家，判「查不到，未验证」，不判「没有更强承诺」。**

### 1.6 一个不在 Q1 字面问法之内、但直接顶着 Q1 前提的现查发现

**Q1 问的是「块层契约」这份文档给不给得更多；这里现查到的是：`crates/singlefs-core` 的「FUA 写」这一步，在真实块设备上今天根本不经过 `REQ_FUA`。**

`crates/singlefs-core/src/block_device.rs:245-247`（`FileBackedBlockDevice::write_at`）与 `:449-451`（`DirectInputOutputBlockDevice::write_at`，O_DIRECT 那一份，真实部署走这条）逐字相同：

```rust
WriteDurability::ForceUnitAccess => {
    self.file.sync_data().map_err(BlockDeviceError::InputOutput)
}
```

`:251-253` 与 `:455-457` 的 `barrier()` 实现：

```rust
fn barrier(&mut self) -> Result<(), BlockDeviceError> {
    self.file.sync_data().map_err(BlockDeviceError::InputOutput)
}
```

**两者调的是同一个 syscall（`fdatasync(2)`）。** `crates/singlefs-core/src/transaction.rs:158-165` 确认根槽那一步就是 `device.write_at(..., WriteDurability::ForceUnitAccess)`；`:566` 起 `CommitStep::Barrier` 调的是 `device.barrier()`——两条路径最终都落到 `sync_data()`。

对一个真实块设备节点（`/dev/xxx`），Linux 的 `fdatasync` 走 `.fsync = blkdev_fsync`（本机内核树 `/home/fy5090/linux-bug-fix/linux/block/fops.c:937`），其实现（`:590-609`）：

```c
static int blkdev_fsync(struct file *filp, loff_t start, loff_t end, int datasync)
{
	...
	error = file_write_and_wait_range(filp, start, end);
	if (error) return error;
	error = blkdev_issue_flush(bdev);
	if (error == -EOPNOTSUPP) error = 0;
	return error;
}
```

`blkdev_issue_flush`（`/home/fy5090/linux-bug-fix/linux/block/blk-flush.c:470-476`）：

```c
int blkdev_issue_flush(struct block_device *bdev)
{
	struct bio bio;
	bio_init(&bio, bdev, NULL, 0, REQ_OP_WRITE | REQ_PREFLUSH);
	return submit_bio_wait(&bio);
}
```

**这是一个零长度、只带 `REQ_PREFLUSH` 的 bio，没有 `REQ_FUA` 位。** 也就是说：真实部署下，`WriteRootRecordForceUnitAccess` 这一步在设备上实际发生的是「[普通 WRITE][空 PREFLUSH]」两个请求，与 `CommitStep::Barrier` 在设备上产生的请求**完全同形**——都是那一个 `blkdev_issue_flush`。

**这对 Q1 的意义**：writeback_cache_control.rst 那两段原文确实只给窄读法，但那两段原文管的是 `REQ_FUA` 这个标志本身；本工程「带 FUA 的写」这一步今天根本没有让任何一次写请求带上 `REQ_FUA` 标志——它是「写 + 空 PREFLUSH」。空 PREFLUSH 恰恰是**宽读法**那一段原文管的东西（1.2 节已引：「previously completed write requests are on non-volatile storage before the flagged bio starts」），只是这里的「flagged bio」是**下一次**发布最前面还没发生的操作，而不是这次根槽写本身。**换句话说：按今天代码的物理机制，「根槽这一步」享有的持久承诺，不是 FUA 段的窄承诺，而是 PREFLUSH 段的承诺**——它逼着自己之前所有已完成的写都持久，这与候选甲的枚举域描述（FUA 与上一道屏障之后的普通写同段、任意子集持久）不是同一件事：候选甲允许「根槽持久而它前面同段的普通写没持久」，但按这份物理机制推，如果 `blkdev_issue_flush` 已经返回（根槽这一步的 `sync_data()` 完成），那么它前面同一段里那些普通写此刻也必然已经持久（因为它们与根槽这次的 flush 用的是同一次 `blkdev_issue_flush` 调用序列——写先完成、flush 后发出，flush 返回时设备缓存已空，之前提交的写不可能还留在缓存里）。

**这一发现是否推翻 D13 已定项 4 或候选甲，不在 Q1/Q2 的判据射程内**（Q1 判的是「原文支不支持窄读法」，不是「本工程的实现是不是窄读法」；这属于「模型是不是忠实刻画了今天的实现」，落在 Q3「枚举域会不会太宽」与 Q5「本机观测的射程」——两者都派给了云端攻方）。**这里如实记下，供主 agent 与云端攻方核实**：`crash.rs`/`segments.rs` 的段模型把「FUA 写」当成一个独立的 `RecordedOperationKind::WriteForceUnitAccess`，这个记法只反映调用方**声明**了 `WriteDurability::ForceUnitAccess`，不反映它在真实块设备上到底发出的是不是 `REQ_FUA`——今天两种真实机制（`FileBackedBlockDevice`、`DirectInputOutputBlockDevice`）都不发 `REQ_FUA`，发的是「写 + 空 PREFLUSH」。**推翻条件**：如果以后 `block_device.rs` 改用 `pwritev2(..., RWF_DSYNC)` 或某种直接标记 `REQ_FUA` 的写路径（不再依赖事后 `fdatasync`），这条发现自动失效，候选甲的窄读法与真实机制才会重新对齐。


## 二、Q2：改后的措辞说全了没有

候选甲原句（背景材料第 73 行，`_d13-item4-fua-r1-background.md` 是背景材料不是 kb，这里只按主 agent 给的候选表复述一次，不当引用）：「带 FUA 的写也是段边界：它关掉自己所在的那一段，它之后的写不与它同段。」

### 2.1 `crash.rs:327-366` 的切法，现查

```rust
pub fn writes_and_segments_with_stream_indexes(
    operations: &[RetainedOperation],
    geometry: &FixedGeometry,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<usize>) {
    ...
    for (stream_index, retained) in operations.iter().enumerate() {
        match retained.operation.kind {
            RecordedOperationKind::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                writes.push(...);
                stream_indexes.push(stream_index);
                current.push(writes.len() - 1);
                if retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    ...
}
```

（原样摘自 `crates/singlefs-harness/src/crash.rs:327-366`，只删了与切段逻辑无关的字段赋值。）

`RecordedOperationKind` 是封闭枚举，只有三个成员（`crates/singlefs-harness/src/lib.rs:30-34`）：`Write`、`WriteForceUnitAccess`、`Barrier`——**没有第四个成员表示「一次写同时带 PREFLUSH 与 FUA」**。

### 2.2 逐个形态：候选甲的措辞该切成什么、代码实际切成什么

| 形态 | 候选甲的措辞该切成什么 | `crash.rs` 实际切成什么 | 对不对得上 |
|---|---|---|---|
| 一次写同时带 PREFLUSH 与 FUA | 候选甲的措辞完全没有涉及这种写——它只谈「FUA 写」，没有谈「同时带 PREFLUSH 的 FUA 写」。若按最接近的读法把它硬套进 `WriteForceUnitAccess`：该写与「上一道屏障之后的普通写」同段、任意子集持久 | `RecordedOperationKind` 只有三个成员，这种写**没有编码方式**——记录层写不出「这一次写既是 Barrier 又是 WriteForceUnitAccess」这句话，`writes_and_segments_with_stream_indexes` 根本没有分支能处理它。如果调用方把它记成 `WriteForceUnitAccess`（唯一能落地的选择），效果与「候选甲该切成什么」一致：同段、任意子集持久 | **对不上，是模型的表达力缺口，不是候选甲这句话本身写错**：PREFLUSH 分量要求「它之前已完成的写在它开始之前必须持久」（1.2 节的窄/宽读法之分），这等价于在这次写之前隐式插了一道 Barrier；但记录层没有位置存这道隐式屏障，候选甲的措辞也没有点名这种写，逐字对不上 `crash.rs` 能表达的东西——因为 `crash.rs` 表达不了它 |
| 一道屏障紧跟一个 FUA 写 | 屏障先关掉它之前的段（若非空）；FUA 写单独进入一个新段，因为屏障已经清空了 `current` | 同左：`Barrier` 分支先 `if !current.is_empty()` 关段，`current` 归零；下一步 `WriteForceUnitAccess` 把自己 push 进 `current` 后立刻 `segments.push`，得到一个只含它自己的段 | **对得上**：`crash.rs:341` 关屏障前的段，`:356-358` 让 FUA 独立关自己的段，两步顺序执行，候选甲「它关掉自己所在的那一段」在这里的「自己所在的那一段」恰好只有它自己 |
| 流首就是 FUA 写 | `current` 从空开始，FUA 写进去后立刻关段，得到一个只含它自己的段，作为整条流的第一段 | 同左：循环第一次迭代命中 `WriteForceUnitAccess` 分支，`current` 此前是空 `Vec::new()`，push 后 `segments.push`，产出 `segments = [[F]]` | **对得上**，无需前面有屏障：候选甲的措辞没有预设「一定先有屏障」，这一步验证了它在没有屏障时依然成立 |
| 连着两个 FUA 写 | 第一个 FUA 写关掉只含它自己的段；`current` 归零；第二个 FUA 写同样只含它自己，各自成一个独立的段 | 同左：两次 `WriteForceUnitAccess` 各自触发一次 `segments.push`，`segments = [[F1], [F2]]`，`F2` 不进 `F1` 的段 | **对得上**，候选甲「它之后的写不与它同段」这句直接覆盖了这个形态：F2 是「它之后的写」，确实不与 F1 同段 |
| 流尾是 FUA 写 | FUA 写自己触发关段，循环结束时 `current` 已经是空的，末尾的 `if !current.is_empty()` 不会再补一段 | 同左：`WriteForceUnitAccess` 分支已经在循环体内部把 `current` 清空，循环结束后的收尾判断因此不生效 | **对得上**，候选甲没有额外要求「流尾要不要再关一次」，代码也确实不需要——FUA 自己已经处理干净 |

### 2.3 只有一处对不上：一次写同时带 PREFLUSH 与 FUA

**结论**：候选甲这句话对着 `crash.rs:327-366` 能表达的四种边界形态（屏障紧跟 FUA、流首 FUA、连续 FUA、流尾 FUA）逐字对得上，这四种不构成「措辞没说全」。**唯一说不全的是「一次写同时带 PREFLUSH 与 FUA」**——这不是候选甲选错了字，是承载候选甲这句话的底层类型（`RecordedOperationKind` 三元封闭枚举）今天写不出这种写，候选甲的措辞因此对它保持沉默；沉默的后果是：如果调用方真的发出这样一次写（比如把「屏障 + FUA 写」优化成一次 `pwritev2(..., RWF_DSYNC)`），录制层只能把它降级记成 `WriteForceUnitAccess`，而候选甲「它关掉自己所在的那一段」这句话会被套用到一个本该更强（前面的写也必须整段持久，等价于该写自带一道屏障）的操作上，产生一个模型本不该允许的「FUA 已持久、它前面同段的写没持久」状态。**这与 1.6 节现查到的发现是同一处缺口在两个层面的表现**：1.6 节说的是「代码今天用的真实机制（写 + fdatasync）在物理上已经隐含了这道屏障」，这里说的是「录制模型的类型系统压根表达不出这种写」——两者共同指向候选甲这句话没有覆盖到的同一类操作。**能推翻这条判定的现象**：如果背景材料能证明 `transaction.rs` 或任何调用方今天、或计划中会发出一次同时带 PREFLUSH 与 FUA 语义的写（本轮现查 `transaction.rs:548-559` 的五步 `CommitStep` 序列，`Barrier` 与 `WriteRootRecordForceUnitAccess` 始终是分开的两步，不存在合并成一步的调用点），这条判定不成立。


### 2.4 `segments.rs:71-74` 与它的断言（`:266-274`）：与 `crash.rs` 是不是同一件事

`crates/singlefs-harness/src/segments.rs:71-74` 文档注释（现查行号）：

```
71: /// 切段（与 E142（第一个事务的干跑） 的切法同一条规则，产物 `name=segments` 就是按它切的）：
72: /// 屏障关掉它之前那一段（屏障算在被关掉的那一段里）；段里还一个写都没有时（流首那道屏障）它并进即将开始的那一段；
73: /// FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）；
74: /// 流尾只有屏障没有写的那一串并进上一段——录到的每一步都恰好落在一个段里。
```

`split_into_segments`（`:76-106`）与 `writes_and_segments_with_stream_indexes`（`crash.rs:327-366`）逐句对照：

| 环节 | `crash.rs` | `segments.rs` | 是否等价 |
|---|---|---|---|
| 屏障关段 | `current`（只装写下标）非空才关 | `current`（装 `StepKind`，含屏障自己）里 `writes_in_current > 0` 才关 | 等价：判据都是「屏障之前有没有写」，`segments.rs` 多存的屏障本身只进「种类串」显示，不参与计数 |
| FUA 关段 | 写入 `current` 后立即无条件 `segments.push` | 写入 `current` 后立即无条件 `segments.push` | 等价，逐字同构 |
| 流尾收尾 | `current` 非空就整体追加成新段（不区分是不是纯屏障） | 按 `writes_in_current == 0` 与否分流：纯屏障（`writes_in_current == 0`）并进上一段；含写的追加成新段 | **写分区一致，屏障归属处理不同**：`crash.rs` 完全不记录屏障（它的 `current` 里从没放过屏障），所以流尾纯屏障时 `crash.rs` 对应位置的 `current` 本来就是空的，那次 `Barrier` 因为 `current.is_empty()` 直接被跳过、什么也不做；`segments.rs` 则显式把这类屏障追加进上一段的「种类」串里，为了让 `kinds=` 报告里能看到那个屏障。两条路径对**写的分段**给出完全相同的结果，差别只在于要不要把屏障记进报告字符串里 |
| 闭式状态数 | `closed_form_state_count`（`crash.rs:370-375`）：`1 + Σ(2^|segment| − 1)`，`|segment|` 是写下标个数 | `closed_form_state_count`（`segments.rs:126-137`）：同一个公式，`writes` 现过滤掉 `Barrier` 再取长度 | 逐字同一条公式，`segments.rs` 显式过滤屏障、`crash.rs` 天然不含屏障，结果相同 |

**判定：两份实现说的是同一件事**——对「写」这一层的分段结果逐操作等价，唯一差异是 `segments.rs` 额外记录屏障用于生成 `kinds=` 报告字符串，不影响状态枚举。这一等价性也被仓里已有的单测钉住：`registered_segment_sequences_match_every_recorded_path`（背景材料第 521 行引用；本轮未重新执行，属于「能用命令核的事实」但这一次判定不需要重复现跑这一条既有断言）。

`segments.rs:266-274` 的钉子测试（现查行号）：

```
266: // FUA 不替它前面的普通写做持久：普通写与 FUA 同段。
267: let fua_after_plain = vec![
268:     operation(RecordedOperationKind::Write, unit),
269:     operation(RecordedOperationKind::WriteForceUnitAccess, 1 << 20),
270: ];
271: assert_eq!(
272:     segment_sizes_text(&split_into_segments(&fua_after_plain, &geometry)),
273:     "2"
274: );
```

这条断言与候选甲的措辞（「它关掉自己所在的那一段」，而「自己所在的那一段」按 2.2 节的枚举域包含它前面自上一道屏障以来的普通写）逐字对得上：一个普通写 + 一个 FUA 写 = 一段、大小 2。**候选甲这句话与 `segments.rs` 的文档注释、`segments.rs` 的断言、`crash.rs` 的实现，四处相互一致**（2.4 节确认 `segments.rs` 与 `crash.rs` 等价；2.2 节确认候选甲的措辞与 `crash.rs` 在四种可表达形态下逐字对得上）。

## 三、D19、D4、D20 三条已定项自己的定案句（材料员报的空白，按指示直接读 kb 原文核实）

这三条不在 Q1、Q2 的判据射程内（Q1 管契约读法、Q2 管候选甲措辞对不对得上 `crash.rs`），它们出现在已定项 4「射程」段引用的撕裂子集/校验和论证里，落在「撕裂子集不枚举」那半句，正文已明写这半句不在本轮射程。这里只如实记录现查结果，不据此改判 Q1、Q2：

- `.claude/kb/decisions/20-承重面单元的原子性与自包含.md:67`（现查）：「**有父指针的单元**（多数）| **不依赖任何宽度。** 撕裂粒度是 512、4 KiB 还是整块，对父指针校验和这条论证毫无影响」——与已定项 4 射程段「与 D20（承重面：单元的原子性与自包含）『有父指针的单元不依赖任何宽度』一致」逐字对得上，不是摘句、不是转述。
- `.claude/kb/decisions/19-块指针的结构与宽度预算.md:39`（现查）：`#### 已定项 2：密文校验和取 32 位`——已定项 4 射程段引「D19（块指针的结构与宽度预算） 已定项 2」佐证「CRC32C 32 位」的碰撞概率说法，标题与引用的字面「32 位」对得上。
- `.claude/kb/decisions/04-校验和位置.md:18`（现查）：`#### 已定项 1：校验和粒度与随机小读的张力`——已定项 4 的「依据」段引「D4（校验和位置） 已定项 1」佐证「有父指针的靠父指针里的校验和」，但已定项 1 原文（`04-校验和位置.md` 已定项 1 整段，现查过）谈的是「数据 extent 的校验和 / MAC 覆盖 32 KiB」这条粒度定案，**不是**「校验和内联进父指针」这件事本身——后者写在 `04-校验和位置.md` 文件开头的引言段（未编号的正文，「校验和内联进指向该块的父指针」），不在已定项 1 里。**这一处引用的分项号可能指错了位置**（该指向引言段而不是已定项 1），但这属于已定项 4「依据」段的内部索引问题，不在 Q1/Q2 的判据射程内，如实记下供主 agent 核，不在本报告下判定。


## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1：`writeback_cache_control.rst` 两段原文 | 只支持窄读法 | PREFLUSH 段明写「之前已完成的写」，FUA 段只字未提，两段各管各的（1.2 节） |
| Q1：两标志同设一个 bio 那句 | 不改变窄读法 | 「可以同设」不等于「单设 FUA 也顺带有 PREFLUSH 的效果」（1.3 节） |
| Q1：virtio-blk 规范给不给得更多 | 给得更少，不是更多 | 规范全文零处「FUA」，命令类型里没有 FUA，块层靠写后补发 FLUSH 模拟（1.4 节） |
| Q1：NVMe / SCSI SBC 规范给不给得更多 | 查不到，未验证 | 官方原文均被访客登记闸挡住，未提交用户个人信息，如实报告查不到，不判「没有」（1.5 节） |
| Q1：本工程「FUA 写」今天真实机制是不是 REQ_FUA | 不是，是写 + 空 PREFLUSH | `block_device.rs` 两份实现的 `ForceUnitAccess` 与 `barrier()` 调的是同一个 `sync_data()`，块设备上落到 `blkdev_issue_flush`（零长度 `REQ_PREFLUSH`），不带 `REQ_FUA`（1.6 节，供主 agent 与 Q3/Q5 核） |
| Q2：候选甲对屏障紧跟 FUA、流首 FUA、连续 FUA、流尾 FUA 四种形态 | 逐字对得上 | 四种形态下 `crash.rs:327-366` 的切法与候选甲的措辞逐字相符（2.2 节表格） |
| Q2：候选甲对「一次写同时带 PREFLUSH 与 FUA」 | 说得不全 | `RecordedOperationKind` 三元封闭枚举写不出这种写，候选甲的措辞对它保持沉默，若强行降级记成 `WriteForceUnitAccess` 会产生模型不该允许的状态（2.3 节） |
| `segments.rs:71-74` 文档注释与断言、`crash.rs:327-366` | 说的是同一件事 | 对写的分段结果逐操作等价，唯一差异是 `segments.rs` 多记屏障用于报告字符串（2.4 节表格） |
| D19 已定项 2、D20 已定项 4（第 4 行「有父指针的单元」） | 与已定项 4 的引用逐字对得上 | 现查原文见三、（不在 Q1/Q2 判据射程，仅记录） |
| D4 已定项 1 | 引用的分项号可能指错位置 | 「父指针里的校验和」写在 D4 文件引言段，不在已定项 1（不在 Q1/Q2 判据射程，仅记录） |

## 没做什么

- 不判 Q3（枚举域会不会太宽）、Q5（本机观测取哪一半）：派给云端攻方，本报告 1.6 节的现查发现只如实记录、不据此下判。
- 不判 Q4（替候选乙辩护）：派给本地辩方。
- NVMe Base Specification、SCSI SBC 官方原文没有取到：两处都被访客身份登记闸挡住，需要提交姓名、机构、邮箱等个人信息才放行，未获用户明确同意，没有提交，如实报告「查不到」，没有用内核驱动代码或记忆里的说法冒充规范原文。
- 没有重新跑 `registered_segment_sequences_match_every_recorded_path` 或任何 `cargo test`：2.4 节的等价性判定完全靠逐行读两份源码得出，这条既有单测的存在与其名字是从背景材料第 521 行读到的，没有现跑复核（`omitClaudeMd` 下不编译 Rust，也不在这份定义的写范围内新建编译产物）。
- D19、D4、D20 三条的核实只覆盖已定项 4 明确点名的那几个分项号，没有通读这三份决策文件的全文，也没有判断它们自身是否「半定」会不会影响已定项 4——按主 agent 给的指示，这三条只是已定项 4 携带的指针，本轮只核指针指没指对地方。
- 没有对 1.6 节的发现提出候选甲/乙/丙之间该怎么改的建议：那是判决权，不是这条腿的职责（`evidence-discipline.md`「判决由主 agent 做，不由投票做」）。
