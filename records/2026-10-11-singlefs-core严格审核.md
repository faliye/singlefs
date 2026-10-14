# 2026-10-11 `crates/singlefs-core` 严格审核：按功能与潜在问题整理

范围：`crates/singlefs-core/src/` 全部 30 份源文件，约 52600 行，其中产品代码约 43000 行（`#[cfg(test)]` 之前的部分），逐行读完。方法：只读代码与注释，对照 `.claude/rules/fs-design.md`、`.claude/singlefs-ai-sop/rules/code-discipline.md` 与各条款判；没有跑测试、没有跑门禁；每一条「代价」都是按代码路径推的，没量过。没有派子 agent（用户要求）。

去向：严重与中等的条目归进 `.claude/kb/milestone/03-third-txn.md`「待优化」表第 28 行起；与已登记欠账重叠的（C567（分配与发布的代价随盘容量线性）、C476（盘上内容可控时走得到的 panic 有 21 处）、C553（盘在两次挂载之间改大小没有条款）、C549（分配记录树固定点 64 轮的上限没有依据））在那一行的「现状」列点名，不另立欠账。

## 一、按功能模块的审核结论

| 模块 | 职责 | 结论 |
|---|---|---|
| `address.rs` / `bytes.rs` / `checksum.rs` | 地址 newtype、字节读写器、CRC-32C | 正确。读写器越界靠调用方先判长度；`wide_checksum_with_field_zeroed`（`checksum.rs:73`）每次把整段复制一份再算 |
| `pointer.rs` / `unit.rs` / `records.rs` | 指针、三类单元、各类记录的编解码 | 解析侧判得严。写侧两处松：位置项断言是 `<=`（`pointer.rs:324`、`pointer.rs:399`）而读侧 I-2.5（位置条目按设备身份升序） 是严格升序；`build_packed_unit` 装记录没有容量预判，溢出时在 `ByteWriter::put` 里 panic（`build_index_node`、`build_data_unit` 有预判） |
| `block_device.rs` | 块设备 trait、文件后端、O_DIRECT 后端 | 可用。文件后端把 FUA 实现成 write 加 fdatasync，比真 FUA 强；O_DIRECT 后端每次 I/O 分配一块对齐缓冲；sysfs 探测用路径末段文件名（`block_device.rs:546`），符号链接路径探不到 |
| `system_configuration.rs` / `root_record.rs` / `root_ring.rs` / `journal.rs` | 固定结构 | 格式层正确。`parse_slot`（`system_configuration.rs:559` 一带）只核校验和、incompat 位与 S 的范围，compat_ro 位与写者身份不核；`record_offset`（`journal.rs:204`）对 counter 为 0 下溢 panic |
| `make_filesystem.rs` | mkfs 与几何检查 | 大体正确。`check_geometry` 声称任何写之前全判，但没判 `physical_block_size` 整除 4096 与 16384（`make_filesystem.rs:357` 一带只判根槽宽），pbs 大于 4096 时清零之后中途报 Unaligned |
| `allocator.rs` | 位图、聚簇段、bump、释放与延迟回收、影子账、根环占用表 | 账目逐盘对称，功能完整。代价随盘容量与记录数增长、落在运行时决策路径上（问题 1） |
| `allocation_record_tree.rs` / `extent_tree.rs` / `inode_tree.rs` / `code_two_tree.rs` / `tombstone_tree.rs` / `lineage_tree.rs` / `retirement_tree.rs` / `container_tree.rs` | 各树的规划、装节点、读回 | 算法正确：inode 叶借用算术、码 2 树中切与收缩、容器树长高都核过。分配记录树的层数由当前设备大小现算（`AllocationRecordTreeGeometry::of_reader`），不落盘 |
| `recovery.rs` | 择配置、择根、扫环、重放、重建版本、F2 退回、各类重读一次 | 判据层层设防，与条款对得上。重建版本时把第一个文件的全部数据单元读进内存（`recovery.rs:2630`）；数据单元读不出时静默替换成空字节（`recovery.rs:2639`） |
| `mount.rs` | 可写挂载全流程、影子账隔离、抬 F、取号前预演、删除续做、实例表清扫、管理员回退 | 逻辑自洽，每一步在落盘前拒的纪律守得很好。复杂度极高：影子账四层回落分支，`rebuilt_allocator`（`mount.rs:2760`）约 458 行、`establish_instance`（`mount.rs:5942`）约 460 行 |
| `transaction.rs` | 提交状态机、五种提交步骤、取号、暖机、写行、带文件发布、冻结重发、删除三步 | 核心设计干净：封闭枚举 `CommitStep`、一条持久路径 `persist_publish_writes`、冻结重发。`publish_admitted`（`transaction.rs:8099`）约 1228 行；每次发布克隆整个分配器、克隆全部数据单元字节 |
| `admission.rs` | 空间准入式子、检查点成本、环窗口 | 算术一致。检查点成本按整棵分配记录树最坏几何取，切换预留再乘约 12 倍，环窗口项 (3S+1)×W 在小盘上占主导 |
| `mounted_session.rs` / `mounted_read.rs` | 会话层、只读挂载 | 可用。会话每次用户改动重读全部系统配置槽与根槽；只读侧中央映射查找是线性 `Vec`（`mounted_read.rs:832`），inode 记录也线性 |
| `instance_table.rs` / `instance_table_sweep.rs` / `unlink.rs` / `deleted_inodes.rs` / `write_accounting.rs` / `write_request_split.rs` | 实例表链、清扫、删除分批、队列、写账、切分 | 正确，没有发现问题 |

功能边界（里程碑三的既定射程，不算缺陷）：只有 inode 1 能带数据（`recovery::extents_of_the_first_file` 拒别的）、没有目录、两块盘镜像写死（mkfs 断言恰好 2 块）、顺序写只能从偏移 0 整份写、加密不进第一版。

## 二、潜在问题（按严重度）

### 严重：会随规模变坏，且落在条款不许遍历的那一格

1. **分配器的决策路径代价随盘容量与记录数线性增长**，与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」第一格冲突。`allocator.rs:1810` 的 `record_for` 在 `Vec<AllocationRecord>` 上线性查找，每次发布对每个被换下角色、每块盘各调一次（`transaction.rs:3789` 的 `placement_registered_unreleased_on_every_device`）；`allocator.rs:792`、`allocator.rs:836` 取落点从单元区起点逐槽扫，每个候选还拿全部开放聚簇段比一遍；三张 `Vec<bool>` 位图每槽约 3 字节；已释放、永不复用的记录一直留在 `records` 里。改法方向：记录改成按 (设备, 槽) 索引的 map，位图改成压缩位图并维护「第一个空槽」游标。已登记 C567（分配与发布的代价随盘容量线性），这次补了 `record_for` 与释放清单两处。

2. **每次发布与挂载的内存量随文件大小线性增长。** `recovery.rs:2630` 到 `recovery.rs:2647` 重建版本时把第一个文件的每个 32 KiB 数据单元读进 `data_unit_contents_in_file_order`，再塞进 `TransactionOutput.units`（`recovery.rs:3065`）。`transaction.rs:8071` 的 `carried_file_version_units` 对不改文件内容的每一次发布（建 inode、unlink 意图与各批、写行、空发布）克隆全部数据单元字节，`transaction.rs:8783` 再装进这一版的 `units`；`transaction.rs:9191` 的 `PublishWrites.units` 又克隆一份重写的单元。一个 1 GiB 的文件意味着每次发布搬 1 GiB 内存。改法方向：`units` 只装提交内生块，数据单元只留指针与校验和。

3. **每次发布整个克隆分配器，且不止一次。** `transaction.rs:6618` 发布前克隆一次（注释自己量到约 450 KiB，这个数随盘容量增长）；固定点每轮再克隆（`transaction.rs:6929`，上限 64 轮）；冻结的发布里再存一份（`transaction.rs:1291` 的 `allocator_after_the_publish`）；`mount.rs` 4 处、`mounted_session.rs` 2 处。这是用拷贝换「失败不留半新状态」。改法方向：变更日志加撤销。

4. **每次系统配置槽写都重读整条根环与最新根的实例表。** `transaction.rs:534` 的 `write_system_configuration_slot` 调 `recovery.rs:1734` 的 `effective_rollback_floor_rereading_unreadable_reads_once` 现算整池 F，每次发布末尾的轮换对每块盘各做一次，即每次发布多读 2 轮根环（3 区 × S 槽）加实例表链；会话层 `publish_user_change` 每次用户改动还重读系统配置槽与根槽。发布路径的读放大是推的，没量过。改法方向：F 的生效值由分配器增量维护，系统配置写只读它。

5. **分配记录树的层数不落盘，由当前设备大小现算。** `AllocationRecordTreeGeometry::of_reader` 按读者报的盘大小算层数，设备被扩容、缩容或任何一块盘尺寸报告不同，整棵树被判损坏。按位置寻址的树应把几何写进根或系统配置。与 C553（盘在两次挂载之间改大小没有条款） 是同一件事的格式侧。

### 中：正确性边角、panic 面与可维护性

6. **数据单元读不出被静默替换成空字节。** `recovery.rs:2639` 到 `recovery.rs:2644` 把映射里缺失或两份都读不出的数据单元存成 `Vec::new()`，这一版 `units` 里 Data 角色字节为空，碰到它的是对拍、checker 与照抄逻辑，不是报错。应作为 `Result` 交出或在版本里显式标记。

7. **mkfs 几何检查缺一条。** 见模块表 `make_filesystem.rs` 那一行。

8. **panic 面大。** 产品代码里 `expect` / `assert` / `panic` 约 600 处：`transaction.rs` 171、`allocator.rs` 74、`recovery.rs` 66、`admission.rs` 58、`mount.rs` 45、`unit.rs` 35。多数带着不变量说明，但有几处依赖「盘上数据已核过」而核的那一道在别处：`journal.rs:204` 对 counter 为 0 下溢；`unit.rs` 的 `build_packed_unit` 没有容量预判；`unlink::plan_the_extent_tree_after_a_deletion_batch` 对上一版位置直接断言（合法但非最小形态的盘上树会 panic）；`allocation_record_tree::records_of_each_leaf` 对未验证记录断言。C476（盘上内容可控时走得到的 panic 有 21 处） 普查过 21 处，这 4 处不在那张清单上，要现核。

9. **函数体量与路径数。** `publish_admitted` 约 1228 行、`rebuild_version_in_the_unit_area_starting_at`（`recovery.rs:2519`）约 758 行、`establish_instance` 与 `rebuilt_allocator` 各约 460 行，两处挂 `#[allow(clippy::too_many_lines)]`。code-discipline 允许长函数的前提是「路径数说得出」，这几个函数的提前出口数十个，路径数说不出。

10. **只供测试的开关渗进产品类型。** `PoolWriter` 带 5 个测试开关（节点容量、记录点名容量、容器容量、inode 号水位压回，加分配器上的空间准入、影子账、删除开关），可写挂载有 6 个变体入口，`InodeNumberWatermarkSource` 开关给错直接 panic（`transaction.rs:275`、`transaction.rs:297`）。产品路径与测试路径共用一个类型。改法方向：开关收进一个 `cfg(feature = "test-switches")` 的结构，产品入口只剩一个。

11. **封装泄漏。** `PoolAllocator.devices`（`allocator.rs:1568`）、`PoolWriter.devices` 与 `parameters`（`transaction.rs:309`）、`MountedSession` 8 个字段、`TransactionOutput` 全部字段都是 `pub`。带不变量的字段应私有（code-discipline「类型」一节）。

12. **写侧与读侧对位置项顺序的判据不一致。** `pointer.rs:324` 写侧断言 `locations[0].device <= locations[1].device`，读侧 I-2.5（位置条目按设备身份升序） 要求严格升序。写出两条同设备位置项不会被写侧拦住，只会在下一次读时被判损坏。

13. **布尔参数与字符串错误身份。** `recovery.rs:3678` 的 `replay_journal(..., verify_named_units: bool)` 违反「布尔参数换枚举」；`UnitError::Structure(&'static str)` 与 `RecoveryFailure::UnitMalformed { what: &'static str }` 用字符串当错误身份，调用方按字符串匹配。

14. **错误枚举按审阅轮次长大。** `PublishError` 25 个成员、`MountError` 与 `RollbackError` 更多，成员注释里带「实审 A1b Q5」「代码轮 U9」这类来历。code-discipline 要求错误成员按调用方要做的决定分，这里已经退化成按发现问题的轮次分；决策或轮次改号时注释批量腐烂，而门禁只盯格式常量的派生形态。

15. **核心库生成中文 UI 文案。** `mounted_session.rs:304` 的 `ways_out_after` 返回 `Vec<String>` 中文出路说明，核心层承担了呈现职责。

16. **准入预留在小盘上过重。** `admission.rs` 的检查点成本按整棵分配记录树最坏几何取，切换预留再乘约 12 倍，环窗口项 (3S+1)×W 在小盘上占主导。这是设计取舍，但小盘几乎不可写，且这个数没量过。

17. **系统配置槽校验不全。** 见模块表 `system_configuration.rs` 那一行；另外 device_count 与交进来的盘数在择配置那一步（`recovery::choose_system_configuration`）不比，可写挂载靠后面 `caller_inputs_agreeing_with_the_disk` 补核，只读挂载没有这一道。

### 低

18. 块设备文件后端的 FUA 比真 FUA 强，可能掩盖顺序错误（崩溃枚举另有录制层，影响有限）；O_DIRECT 后端每次 I/O 分配对齐缓冲；sysfs 探测用 `file_name()`。
19. 只读挂载的中央映射查找是 `Vec` 线性查找，每次读整个 32 KiB 单元。
20. `wide_checksum_with_field_zeroed` 每次 `to_vec` 复制整段。
21. `address.rs` 乘法风格不一（有的 `checked_mul`、有的裸 `*`；release 开了溢出检查，都安全）。

## 三、总体评价

做得好的：提交步骤是封闭枚举，三条发布路径共用一条持久函数；落盘失败冻结原样重发；每一种拒绝都在任何写之前返回并说明盘上不变；释放前逐盘核三样、读盘核校验和；取号前预演并断言预演落点与真发相同；所有「重读一次」的口径一致；newtype 与穷举 match 执行得彻底。崩溃一致性那一层的设计纪律是这份代码最强的部分。

最弱的是**规模可扩展性**：问题 1 到 5 都是「第一版两块小盘一个文件」下看不出来、换成真实盘容量与文件大小就会变坏的代价，其中两条直接违反本仓自己的设计规则。第二弱的是**可维护性**：函数体量、测试开关、错误枚举与注释里的审阅编号，让下一个人改任何一处都要先读几百行来历。

没做的事：没有跑测试与门禁；全部「代价」都是按代码路径推的，没量过。
