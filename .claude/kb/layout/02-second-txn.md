# 覆盖写、释放、回退与复用写出哪些字节（新池新建文件没有的五种盘上形态）

**里程碑「覆盖写、释放、回退与复用」步 7 的登记表**（[02-second-txn.md](../milestone/02-second-txn.md)），2026-09-17 立。新池新建文件的每一段字节在 [layout/01-first-txn.md](01-first-txn.md)；这里只登记覆盖写、释放、回退与复用**新写出**的形态——同一个字段的宽度与落点仍以那张表为准、不另抄，这里写的是「第一次不为 0 / 第一次出现」的取值与它压着的决策分项。登记位放哪（另开一份还是并进 layout/01-first-txn.md 加一节）是预想、等用户定；每一行都指得到里程碑的一步与它的「写出的字节」行，字节的实况由 `crates/singlefs-harness/tests/file_overwrite_step_*.rs` 的用例钉住（新池新建文件那张表由 E142（新池新建文件的干跑） 的产物钉，覆盖写、释放、回退与复用没有干跑产物）。

⚠️ 表里前两种形态（已释放的分配记录、defer 待释放行不为 0）2026-09-17 起在第一个文件版本 A（txg 3）里就已出现：A 重写树表时释放 mkfs 那片第 0 版树表单元（D5（快照 / 空间记账机制） 已定项 8 那张表 defer 行 16384，字节在 [layout/01-first-txn.md](01-first-txn.md)）。它们留在这张表里，登记的是 B 之后的样子；标题说的「新池新建文件没有」对这两种只在立表时成立。

<!-- doc-lint:not-numbers P1 -->

| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 | 钉住它的用例 |
|---|---|---|---|---|---|
| 已释放的分配记录 | 第一个文件版本（A，txg 3）换下 mkfs 的第 0 版树表单元；发布 B 换下 A 的八个单元 | 那条记录原地改写：跨度段最高位 = 已释放，代字段 = 释放代（= 这次发布的 txg）；条目不删、槽仍占着 | D3（空间分配） 已定项 7；D19（块指针的结构与宽度预算） 已定项 5 | 步 2（[layout/01-first-txn.md](01-first-txn.md) 五那一行的字段宽度） | `overwrite_in_one_instance.rs` `release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue` |
| 记账的 defer 待释放行第一次不为 0 | A（16 384：mkfs 树表那 1 槽）；B 之后 11 槽 | 第 5 项逐盘一行，值 = defer 队列里的槽数 × 16384；已分配（第 1 项）不减、空闲（第 2 项）不加——回收那一刻才动 | D5（快照 / 空间记账机制） 已定项 4 / 已定项 8；D16（发布语义） 已定项 1 | 步 2、步 5 | `overwrite_in_one_instance.rs` `release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue`；`reuse_after_raising_the_floor.rs` `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` |
| 实例表 `kind` 0 行 | 第二次可写挂载的写行发布（txg 5）：(1, 4, 0)；步 4 的管理员回退（发布 D，txg 9）不写行，实例表照 C 的（同一条指针、同一份字节） | 88 字节一行：kind 1 + 实例 4 + T 8 + W 8 + flags 1 + 预留 66，行在链指针记录之前；T = 那次恢复生效的根的 txg，W = 那次施加的最大事务号；flags 第一版恒 0，读者见非 0 拒收（位 0 曾标回退行，退役、不回收） | D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14；C330（中间实例那一行的 T_pub 取所选根的 txg） | 步 3、步 4 | `writable_mount_takes_a_second_instance.rs`；`rollback_by_a_forward_publish.rs` `rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content`（D 的实例表与 C 的同一条指针） |
| 根记录的回退下界 F 第一次不为 0 | 抬 F 的空发布（txg 15、16）：F = 11；之后每条根照抄；重开之后新实例的根写 F_生效（D16（发布语义） 已定项 1「生效」：根上与系统配置里读得出的最大值） | 根记录偏移 130 起 8 字节；带 F = X 的根，它的记账行是回收过释放代 ≤ X 之后的账 | D16（发布语义） 已定项 1；D22（单元原子性怎么合成） 已定项 7 | 步 5 | `reuse_after_raising_the_floor.rs` `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`（根记录读回 F = 11）；`floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it`（盘 1 上带 F 11 的根改坏之后重开，F_生效 仍是 11） |
| 根记录的分配记录树根指针第一次不为 0 | 只做过 mkfs 的池第二次可写挂载的写行发布（建池 → 挂载 → 退出 → 再挂载写行） | 根记录偏移 342 起 86 字节，指写行那次发布写的那一片分配记录节点（单元头树 ID 写 0）；那次发布写实例表与这片节点两个单元，被换下的上一版实例表与上一片节点记成已释放 | D16（发布语义） 已定项 9；D22（单元原子性怎么合成） 已定项 7 | 增补 2（收口表第 ④ 行） | `writable_mount_of_a_formatted_pool.rs` `the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool` |
| 复用后改写的分配记录；回退那次复活的分配记录 | 复用：E（txg 17）拿到 50176（mkfs 实例表那 2 槽）；复活：步 4 的管理员回退（发布 D，txg 9）把 A 引用、C 的账记成已释放的四个用户可见单元的记录改回已分配 | 同一条记录原地改写、同盘同槽仍只一条：复用时代 = 这次发布的 txg（17）、已释放位清；复活时代写回 R_old 那一版账里的分配代（A 的 3，不是 D 的 txg）、已释放位清，两块盘各一条。同一次发布里 C 引用、D 不引用的用户可见单元记成已释放、释放代 9（与步 2 登记的已释放分配记录同形）；D 的 journal 记录 jsn 接 C 那一条、事务号 0 | D3（空间分配） 已定项 7；D23（journal 的角色与格式） 已定项 14（管理员回退那一段表里「复活」「释放」两格） | 步 4、步 5 | `reuse_after_raising_the_floor.rs` `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`（复用：50176、代 17）；`rollback_by_a_forward_publish.rs` `rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content`（复活：分配代 3；释放：释放代 9；jsn 接 C、事务号 0） |
| inode 叶容器装多条记录、第二片容器、内部节点多条条目 | 并行线三建 233 个 inode 的那次发布（一片容器装满 233 条）；第 234 个 inode 让它在末尾分裂出第二片 | 码 3 容器的记录区装 233 条 140 字节记录、按 inode 号升序；分裂出的右半容器号 = 触发分裂那条新记录的 inode 号、出生代 = 本次发布的 checkpoint_txg、出生树 = 执行这次分裂的那棵树，左半保留全部原有记录与四段身份；根（码 2）的条目数从 1 涨到 2，每条 120 字节，分隔 key = 该孩子创建时的最小 key | D8（核心索引结构） 已定项 6；D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 11 | 并行线三 | `many_inodes_write_and_read.rs`；另有 inode_tree 模块的 10 条单测判三条分裂纪律（源码在 crates/singlefs-core/src/inode_tree.rs，路径不加反引号：门禁 76 号把反引号里以 .rs 收尾的记号当集成测试文件名，去 crates/*/tests/ 下找） |
| inode 记录偏移 48 的 `blocks` 第一次不是 64 | 并行线三建 N 个 inode 的那次发布：建出来还没写过数据的 inode 写 0 | 8 字节，按 D8（核心索引结构） 已定项 6 的口径值 = ⌈size ÷ 512⌉（逻辑长度的 512 字节块数，不表示分到的空间）：新建的 0；`crates/` 现按 ⌈size ÷ 512⌉ 填（C480（inode 记录的 blocks 怎么算全仓没有条款） 已还清，checker 判 I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）） | D8（核心索引结构） 已定项 6 | 并行线三 | `many_inodes_write_and_read.rs`；字段宽与取值另由 records 模块的 widths_of_every_record_match_the_byte_table 单测钉住（源码在 crates/singlefs-core/src/records.rs，路径与函数名不加反引号：门禁 76 号按反引号里的记号去 crates/*/tests/ 下找文件与函数） |
| 记账里 inode 号水位第一次大于 2 | 并行线三建 N 个 inode 的那次发布：N + 2 | 第 12 项，不带设备维（设备段写保留值 0xFFFF_FFFF）；每次发布重写、合并取最大值 | D5（快照 / 空间记账机制） 已定项 4；D5（快照 / 空间记账机制） 已定项 10 | 并行线三 | `many_inodes_write_and_read.rs`（水位 = 235）；坏镜像 `checker_known_bad_images.rs` 的 I-9.6（水位大于两处最大号） 那一格 |

只住内存、不写字节的两样，登记在这里是为了别让人来找：影子账（被抛弃根独占量，D28（挂载期承诺量） 已定项 1 第九项）与回收过还没复用的落点集合（`PoolAllocator` 的 `reclaimed`）。

## E152（按里程碑对比六家文件系统的文件性能） 第三次跑量到的（2026-09-17）

发布 B 在真设备（两块 16 GiB virtio 盘）上 5 轮逐字相同：两盘合计 21 次写调用（来宾块层记 26 次写请求：每道屏障与每次 FUA 各多记一次）、344 576 字节（2 × 172 032 + 根槽 512）、4 次屏障、1 次 FUA，段序列 16+2+1+2，与新池新建文件的 `path=transaction` 同型——产物 `e152-file-system-benchmark-file-overwrite-2026-09-17.out` 的 `name=summary` 行，表在 `research/perf-by-milestone.md` 第三·二节。回退、抬 F、复用那几次发布的字节只有层 0 与用例钉着，真设备上没量。

## 历史版本

### 2026-09-26
- 管理员回退改成挂着时的一次向前发布之后按实现改（D23（journal 的角色与格式） 已定项 14；实三交回，`records/2026-09-24-里程碑二收尾调度.md` 第三节「实三交回」那一行）：「实例表 `kind` 0 行」改前写回退的发布 D（txg 9）写回退行 (1, 3, 0) flags bit0 = 1 与中间实例行 (2, 0, 0)、回退行的 W 恒 0，改后写 D 不写行、实例表照 C 的，flags 第一版恒 0；「复用后改写的分配记录；回退之后新实例的第一条记录接在环里最大 jsn 之后」改名「复用后改写的分配记录；回退那次复活的分配记录」，改前写 E 拿到 50178、回退之后新实例的第一条记录 jsn = 环里最大 + 1（C340（回退之后记录链从哪条之后接没有定义） 取 P2），改后写 E 拿到 50176、D 把 A 的单元的记录改回已分配（分配代写回 3）、D 的记录 jsn 接 C、事务号 0；「系统配置槽的回退见证表第一次有条目」那一行删掉：见证表删了，系统配置字段表之后是补齐 0（D22（单元原子性怎么合成） 已定项 9），它点名的 `file_overwrite_supplement_two_rollback_witness.rs` 随实三删掉。
- 「根记录的回退下界 F 第一次不为 0」点名的用例改前是 `one_device_carrying_the_floor_alone_does_not_take_effect_on_remount`（实二改名改写、断言翻面），改后点 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 与 `floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it`。

### 2026-09-17
- 发布 B 真设备那段：改前写「两盘合计 21 次写请求」；改后写 21 次写调用、来宾块层记 26 次写请求。依据：E152（按里程碑对比六家文件系统的文件性能） 实验页「这几个数说明什么（覆盖写、释放、回退与复用，与两盘镜像的六家比）」一节第 5 条的块层差分。
- 立表：五种形态各指到决策分项、里程碑那一步与钉住它的用例。登记位放哪（另开一份还是并进新池新建文件那份）标预想、等用户定。
- 用户定案：按里程碑放进 `.claude/kb/layout/` 目录，一个里程碑一份、与 `milestone/` 同号；这份从 `.claude/kb/second-txn-layout.md` 搬到 `.claude/kb/layout/02-second-txn.md`，全仓现行文件里的引用一并改写（`research/prompts/` 下冻结的论证材料不改，里面的旧路径照旧）。
