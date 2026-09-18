# 第二个事务写出哪些字节（第一个事务没有的五种盘上形态）

**里程碑「第二个事务」步 7 的登记表**（[02-second-txn.md](../milestone/02-second-txn.md)），2026-09-17 立。第一个事务的每一段字节在 [layout/01-first-txn.md](01-first-txn.md)；这里只登记第二个事务**新写出**的形态——同一个字段的宽度与落点仍以那张表为准、不另抄，这里写的是「第一次不为 0 / 第一次出现」的取值与它压着的决策分项。登记位放哪（另开一份还是并进 layout/01-first-txn.md 加一节）是预想、等用户定；每一行都指得到里程碑的一步与它的「写出的字节」行，字节的实况由 `crates/singlefs-harness/tests/second_transaction_step_*.rs` 的用例钉住（第一个事务那张表由 E142（第一个事务的干跑） 的产物钉，第二个事务没有干跑产物）。

<!-- doc-lint:not-numbers P1 -->

| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 | 钉住它的用例 |
|---|---|---|---|---|---|
| 已释放的分配记录 | 第一个文件版本（A，txg 3）换下 mkfs 的第 0 版树表单元；发布 B 换下 A 的八个单元 | 那条记录原地改写：跨度段最高位 = 已释放，代字段 = 释放代（= 这次发布的 txg）；条目不删、槽仍占着 | D3（空间分配） 已定项 7；D19（块指针的结构与宽度预算） 已定项 5 | 步 2（[layout/01-first-txn.md](01-first-txn.md) 五那一行的字段宽度） | `second_transaction_step_one_overwrite.rs` `release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue` |
| 记账的 defer 待释放行第一次不为 0 | A（16 384：mkfs 树表那 1 槽）；B 之后 11 槽 | 第 5 项逐盘一行，值 = defer 队列里的槽数 × 16384；已分配（第 1 项）不减、空闲（第 2 项）不加——回收那一刻才动 | D5（快照 / 空间记账机制） 已定项 4 / 已定项 8；D16（发布语义） 已定项 1 | 步 2、步 5 | `second_transaction_step_one_overwrite.rs` `release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue`；`second_transaction_step_five_reuse.rs` `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` |
| 实例表 `kind` 0 行 | 第二次可写挂载的写行发布（txg 5）：(1, 4, 0)；回退的发布 D（txg 9）：回退行 (1, 3, 0) flags bit0 = 1 与中间实例行 (2, 0, 0) | 88 字节一行：kind 1 + 实例 4 + T 8 + W 8 + flags 1 + 预留 66，行在链指针记录之前；T = 那次恢复生效的根的 txg，W = 那次施加的最大事务号；回退行的 W 恒 0 | D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14；C330（中间实例那一行的 T_pub 取所选根的 txg） | 步 3、步 4 | `second_transaction_step_three_second_instance.rs`、`second_transaction_step_four_rollback.rs` |
| 根记录的回退下界 F 第一次不为 0 | 抬 F 的空发布（txg 15、16）：F = 11；之后每条根照抄；重开之后新实例的根写 F_生效（各幸存盘所带 F 最大值的最小值，可以比上一条根低） | 根记录偏移 130 起 8 字节；带 F = X 的根，它的记账行是回收过释放代 ≤ X 之后的账 | D16（发布语义） 已定项 1；D22（单元原子性怎么合成） 已定项 7 | 步 5 | `second_transaction_step_five_reuse.rs` `one_device_carrying_the_floor_alone_does_not_take_effect_on_remount` |
| 复用后改写的分配记录；回退之后新实例的第一条记录接在环里最大 jsn 之后 | E（txg 17）拿到 50178：那条已释放记录改写成代 17、已释放位清，同盘同槽仍只一条；回退之后新实例的第一条记录 jsn = 环里最大 + 1（C340（回退之后记录链从哪条之后接没有定义） 取 P2，D 的 jsn 9），被抛弃发布的记录原样留在环里 | 分配记录：同一条原地改写；journal：环槽由 jsn 定，新实例的记录接在旧记录之后、反向链 0、事务号 0 | D3（空间分配） 已定项 7；D23（journal 的角色与格式） 已定项 14 第 3 条；C340（回退之后记录链从哪条之后接没有定义）（取 P2，预想） | 步 4、步 5 | `second_transaction_step_five_reuse.rs`（复用）；`second_transaction_step_four_rollback.rs` `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`（jsn 9、被抛弃的记录都在） |

只住内存、不写字节的两样，登记在这里是为了别让人来找：影子账（被抛弃根独占量，D28（挂载期承诺量） 已定项 1 第九项）与回收过还没复用的落点集合（`PoolAllocator` 的 `reclaimed`）。

## E152（按里程碑对比六家文件系统的文件性能） 第三次跑量到的（2026-09-17）

发布 B 在真设备（两块 16 GiB virtio 盘）上 5 轮逐字相同：两盘合计 21 次写调用（来宾块层记 26 次写请求：每道屏障与每次 FUA 各多记一次）、344 576 字节（2 × 172 032 + 根槽 512）、4 次屏障、1 次 FUA，段序列 16+2+1+2，与第一个事务的 `path=transaction` 同型——产物 `research/results/e152-file-system-benchmark-second-transaction-2026-09-17.out` 的 `name=summary` 行，表在 `research/perf-by-milestone.md` 第三·二节。回退、抬 F、复用那几次发布的字节只有层 0 与用例钉着，真设备上没量。

## 历史版本

### 2026-09-17
- 发布 B 真设备那段：改前写「两盘合计 21 次写请求」；改后写 21 次写调用、来宾块层记 26 次写请求。依据：E152（按里程碑对比六家文件系统的文件性能） 实验页「这几个数说明什么（第二个事务，与两盘镜像的六家比）」一节第 5 条的块层差分。
- 立表：五种形态各指到决策分项、里程碑那一步与钉住它的用例。登记位放哪（另开一份还是并进第一个事务那份）标预想、等用户定。
- 用户定案：按里程碑放进 `.claude/kb/layout/` 目录，一个里程碑一份、与 `milestone/` 同号；这份从 `.claude/kb/second-txn-layout.md` 搬到 `.claude/kb/layout/02-second-txn.md`，全仓现行文件里的引用一并改写（`research/prompts/` 下冻结的论证材料不改，里面的旧路径照旧）。
