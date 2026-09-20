# 背景材料：D13 已定项 4 里「带 FUA 的写自成一段」这半句，第一轮

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 -->

判的是一条已定项的措辞要不要改。2026-09-20 记。这一轮**不改代码**：代码今天的切法就是候选甲，要判的是条款该不该跟着它改。

## 一、条款今天的原文

`.claude/kb/decisions/13-验证路线.md:71`（D13（验证路线） 已定项 4 的定案句，整行抄）：

> **定案**：崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。撕裂子集**不枚举**，一次写的任何撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像要真的生成出来喂给 checker，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。

要判的只有其中两句：「带 FUA 的写也是段边界：**它自成一段**，它之后的写不与它同段。」别的句子不在这一轮射程里。

## 二、实现今天的样子（主 agent 的观测，读的是工作区那一版）

**发布路径发出的步骤**（`crates/singlefs-core/src/transaction.rs:548-557`）：

```
Barrier                            ← 刷这次事务的 COW 数据与元数据
WriteJournalRecordToEveryDevice    ← 往日志环追加记录，普通写
Barrier                            ← 记录先于根落盘靠这一道
WriteRootRecordForceUnitAccess     ← 写根槽，带 FUA
RotateSuperblockSlots
```

`CommitStep::Barrier` 落到设备上是 `device.barrier()`（`transaction.rs:181`），文件后端实现成 `sync_data()`（`crates/singlefs-core/src/block_device.rs:251`）。根槽那一步用 `WriteDurability::ForceUnitAccess`（`transaction.rs:165`）。

**崩溃点重放的切段**只有一份实现，两个调用方共用（`crates/singlefs-harness/src/crash.rs:327-366`）：屏障把当前段关掉；写与 FUA 写都先推进当前段，**FUA 推进去之后再把段关掉**（`crash.rs:356-358`）。所以 FUA 写是它那一段的最后一个写，与它前面、上一道屏障之后的普通写**同段**。

**段序列登记那一份**（`crates/singlefs-harness/src/segments.rs:71-74`）的文档注释逐字写着：

> FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）

`segments.rs:266-274` 有一条钉住它的断言：一个普通写后面跟一个 FUA 写，段大小是 `2`（同一段），注释写着「FUA 不替它前面的普通写做持久：普通写与 FUA 同段」。

**所以条款字面与代码今天的做法不一致**：条款说 FUA 自成一段，代码让它与前面的普通写同段、再关段。

**变异表第 67 行**（`crates/mutations.tsv:67`）拿掉的正是 `transaction.rs:553` 那道屏障（「步 3：零单元发布在记录与根之间少一道屏障」）。

**主 agent 2026-09-20 让实现员在副本上实测过两种切法**（副本 `/tmp/claude-1000/m2-supp3-item3-r1-fix/drafts/copy-a`，逐条日志在那个目录下）：

| 切法 | 施加变异表第 67 行之后 |
|---|---|
| 代码今天的（FUA 与前面的普通写同段） | `the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence` 判红；崩溃注入里 `every_crash_state_of_a_written_out_history_recovers_into_a_committed_version` 判红，断言是 `record_root_without_record == 0` |
| 照条款字面（FUA 自成一段） | 两条都**判绿**。机理：有屏障时 `[记录写][根槽]`，没屏障时 FUA 先关掉当前段、再推一段，还是 `[记录写][根槽]`，段序列一模一样 |

## 三、外部事实（主 agent 2026-09-20 现查，未在本项目验证）

**块层契约**，`Documentation/block/writeback_cache_control.rst`（本机 `/home/fy5090/code/fs-refs/linux-6.17/`，Linux 6.17），两段原文：

> The REQ_PREFLUSH flag can be OR ed into the r/w flags of a bio submitted from the filesystem and will make sure the volatile cache of the storage device has been flushed before the actual I/O operation is started.  This explicitly guarantees that previously completed write requests are on non-volatile storage before the flagged bio starts.

> The REQ_FUA flag can be OR ed into the r/w flags of a bio submitted from the filesystem and will make sure that I/O completion for this request is only signaled after the data has been committed to non-volatile storage.

同一份文档还写着两个标志可以同时设在一个 bio 上（"The REQ_PREFLUSH and REQ_FUA flags may both be set on a single bio."），以及 blk-mq 那一段：

> When the BLK_FEAT_FUA flags is set, the REQ_FUA bit is simply passed on for the REQ_OP_WRITE request, else a REQ_OP_FLUSH request is sent by the block layer after the completion of the write request for bio submissions with the REQ_FUA bit set.

**本机的盘**（现查 sysfs，机器上只有一块）：

```
/sys/block/nvme0n1/queue/write_cache = write back
/sys/block/nvme0n1/queue/fua         = 1
/sys/block/nvme0n1/queue/rotational  = 0
lsblk: nvme0n1  3.6T  KINGSTON SNV3S4000G
```

## 四、候选

| 候选 | 措辞 | 枚举域 |
|---|---|---|
| **甲** | 「带 FUA 的写也是段边界：**它关掉自己所在的那一段**，它之后的写不与它同段。」（＝代码今天的做法） | FUA 与上一道屏障之后的普通写同段，段内任意真子集持久 |
| **乙** | 维持原文「它自成一段」，改代码去迁就 | FUA 单独一段；前面那些普通写在更早的段里，按全序前缀必须整段持久 |
| **丙** | FUA 自成一段，但段不再是全序前缀（允许它与前一段乱序） | 另一套枚举，层 0（门禁 54 号）与增补 3 第 3 件都要重写 |

## 五、要判的问题

**Q1（契约读对了没有）**：第三节那两段原文，是不是只支持「FUA 的持久承诺罩它自己那一次写的数据，不罩它之前已完成的写」这一个读法？virtio-blk 规范、NVMe 规范（Write 命令的 FUA 位）、SCSI SBC 的 FUA，有没有哪一家比块层这份契约给得更多，多到「FUA 返回时它之前的写也一定在非易失存储上」？举得出就整段引原文。

**Q2（改后的措辞说全了没有）**：候选甲那句话，逐字对上 `crash.rs:327-366` 的切法了吗？有没有第三种形态落在措辞之外——例如一次写同时带 PREFLUSH 与 FUA、或者一道屏障紧跟一个 FUA 写、或者流首就是 FUA 写？逐个形态给出按候选甲的措辞该切成什么、代码实际切成什么，对不上就是措辞没说全。

**Q3（枚举域会不会太宽）**：候选甲让「FUA 写没持久，而它前面同段的普通写持久了」也进枚举域。真设备上这个状态可达吗？若不可达，模型多枚举了一批摆不出来的状态——那算不算把这条已定项改错了？给判据，别只答可不可达。

**Q4（替候选乙辩护）**：找出一个读法，让原文「它自成一段」在今天这套模型里仍然成立。比如把「段」读成别的东西、或者认为「自成一段」与「与前面的普通写同段」并不冲突。辩不出来就写明辩不出来，并说清你排除了哪几种读法。

**Q5（本机那个观测的射程）**：`fua = 1` 说的是这一块盘。第三节最后那段 blk-mq 原文说，`BLK_FEAT_FUA` **没**设时块层会在写完成之后补发一个 `REQ_OP_FLUSH` ——那种盘上 FUA 反而顺带把前面的写也刷了。模型该取哪一半：契约给的弱保证，还是某一块盘恰好做到的强保证？两种取法各会让哪一类真实缺陷在崩溃点重放里看不见？这一格要给判据，不要给倾向。

## 六、分工

| 腿 | 攻击面 |
|---|---|
| 云端攻方（Opus） | Q3、Q5：枚举域宽了会不会把这条已定项改错、模型该取契约还是取本机这块盘。攻的是「候选甲更对」这个结论本身，要举得出一个具体的缺陷形态在候选甲下看不见 |
| 云端正推（Sonnet） | Q1、Q2：逐条核契约原文只支持哪个读法、候选甲那句措辞对不对得上 `crash.rs` 的切法。引 kb 条款写 kb 文件自己的行号，行号去 kb 文件里现查 |
| 本地攻方（英文） | Q4：替候选乙辩护，找出让原文成立的读法。按表格逐格填，不许只答 yes / no。与云端攻方不重叠：它攻候选甲的代价，这一条守候选乙的正当性 |

判决写 `research/prompts/d13-item4-fua-r1-main-verification.md`，带一节 `## 回看决策`（门禁 75 号判形式）。这一轮不改 `crates/`，所以不受门禁 56 号约束。
