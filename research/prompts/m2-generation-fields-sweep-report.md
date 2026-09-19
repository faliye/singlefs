# C374 判据回扫：盘上代字段清单与逐条判定

回扫员（sweep），2026-09-19（JST）。活的种类：新立一条判据。判据原文：`.claude/kb/checks-owed.md` C374（释放代与树表诞生 txg 只有验收断言盯着）已还清那一行；按它立的 I-3.9（释放代落在停止引用它的那一格区间里）（`.claude/kb/invariants.md` 第 131 行）与 I-9.14（树表条目的诞生 txg 跨根不变）（同文件第 275 行）；来历 `.claude/kb/milestone/02-second-txn.md` 第 321 行（收口表第 12 行）。

只判不改。登记给 sweep 的门禁阶段（64-moved-paths.sh）已跑，见第四节。

## 一、它管的对象：盘上格式里每一个代字段——清单来源与现算命令

三处来源，命令与计数如下（全部原样跑出）：

```
$ grep -c -i "txg\|代\b\|instance\|generation\|诞生\|出生" .claude/kb/layout/01-first-txn.md
83
$ grep -c -i "txg\|代\b\|instance\|generation\|诞生\|出生" .claude/kb/layout/02-second-txn.md
6
$ grep -rn "CheckpointTxg\|InstanceGeneration" crates/singlefs-format/src/ crates/singlefs-core/src/*.rs | grep -v "^.*test" | wc -l
290
$ grep -rn "pub [a-z_]*: \(CheckpointTxg\|InstanceGeneration\|u64\|u32\)" crates/singlefs-core/src/*.rs | grep -iE "txg|generation|instance|birth|floor" | grep -v "^.*tests" | wc -l
38
```

第三、四条命令的 290 / 38 行里含大量同一概念在多处（结构体定义、`to_bytes`/`parse`、单测里）重复出现；按"同一物理量在盘上只算一个对象"去重、并按 kb 用词归并后，得到下面第二节的对象清单（逐条给出它在 01/02 两份 layout 与 `crates/singlefs-core/src/*.rs` 里的落点）。裸的"事务号""出生序号"这类严格递增的计数器不算"代字段"（不是"随发布变的号"这个语义，是"同一次发布/同一实例内部的第几个"），本次不列为独立判定对象，只在涉及时标注。

## 二、逐条判定表

判据三问：① 有没有一条在用的不变量说它该取什么值（编号 + 简称）；② 那条不变量 checker 实现了没有（函数名）；③ 还是只有验收断言盯着（点名用例）。

| # | 对象（盘上落点） | ①有效不变量 | ②checker 实现 | ③验收断言（点名） | 判定 | 理由 |
|---|---|---|---|---|---|---|
| 1 | 超级块槽世代号 `Superblock.slot_generation`（`crates/singlefs-core/src/superblock.rs:71`；layout 01 第 91/95/113 行"槽世代号"） | 无 | 无 | 单测按往返断言，未见专门验收 | 分不清 | 它是"哪个槽更新"的择槽计数器，语义只要求严格递增、不要求取到某个具体值；`invariants.md` 全文 0 命中"槽世代号"。是否该算这类判据管的"代字段"存疑，列出交人看 |
| 2 | 超级块 `journal_instance`（实例代号，`superblock.rs:75`） | I-7.7（超级块实例代号不低于根环） | 已实现，`walk::judge_instance_carriers`（`crates/singlefs-checker/src/walk.rs:647`） | — | 过 | `judge_instance_carriers` 直接读各盘超级块槽的 `journal_instance` 参与比对 |
| 3 | 根记录 `instance`（`crates/singlefs-core/src/root_record.rs:18`；layout 01 第 60/355 行） | I-7.7 | 已实现，`judge_instance_carriers`（`instance_carriers` 里 `roots.iter().map(... root.instance)`，`walk.rs:612`） | — | 过 | 同上，根记录的实例代号是 `instance_carriers` 扫描对象之一 |
| 4 | 根记录 `checkpoint_txg`（自身的 txg，`root_record.rs:19`） | 无（它是别的判据的自变量，如 I-3.5 的 `R.txg`） | — | 大量脚本按具体 txg 断言（如各步 `_publish.rs`） | 分不清 | 没有一条不变量说"这条根自身的 checkpoint_txg 该取什么值"——它是被比较的坐标轴，不是被约束的对象；I-7.3（环健康性）只从"存不存在更早代号"侧面涉及、未实现，且不判具体值 |
| 5 | 根记录 `rollback_floor` F（`root_record.rs:22`；layout 02 第 14 行） | 无（只被 I-3.1/I-7.4/I-4.8/I-5.4 当输入使用） | — | `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`（`one_device_carrying_the_floor_alone_does_not_take_effect_on_remount` 等钉 F=11 一类具体值） | 不过 | `invariants.md` 全文没有一条约束 F 自身该取什么值（例如"非递减，除非新实例重开允许回落"这类）；只有验收断言盯着固定脚本上的具体 F 值，checker 一处未判 F 本身对不对 |
| 6 | journal 记录 `checkpoint_txg`（`crates/singlefs-core/src/journal.rs:84`） | 无 | — | 各 `_publish.rs`/`second_transaction_step_*.rs` 断言具体 txg | 分不清 | 与 #4 同一物理量（该次发布的 txg），理由同 #4 |
| 7 | journal 记录 `new_rollback_floor`（`journal.rs:92`） | 无 | — | `second_transaction_step_five_reuse.rs` | 不过 | 与 #5 同源（写入新 F 的 journal 侧字段），理由同 #5 |
| 8 | jsn 的实例代号（`JournalRecord.instance`，`journal.rs:82`；`JournalSequenceNumber.instance_generation`，`address.rs:35`） | I-7.7 | 已实现，`judge_instance_carriers`（扫描 journal 环记录的 instance，`walk.rs:623-627`） | — | 过 | `instance_carriers` 对 journal 环内 fsid 相符的记录同样取出 `record.instance` 参与比对 |
| 9 | 单元写序里的实例代号（`WriteOrder.instance`、`NodePointer.instance`、各单元头 write_order 内的 instance，`unit.rs:29/147/340`，`pointer.rs:82`） | I-7.7 | 已实现，`judge_instance_carriers`（`unit_write_order_instance` 扫单元区头，`walk.rs:576-596`） | — | 过 | 覆盖码 1/2/3 三类单元头写序里的实例代号；判的是"不超过各盘超级块最大者"这个全局上界，不判"这个单元的实例代号是否与它实际所属的那次发布一致"，那一半见 #20 |
| 10 | 单元头诞生代号 `birth_txg`（`PackedUnitHeader.birth_txg` `unit.rs:411`、`DataUnitHeader.birth_txg` `unit.rs:474`、`IndexNodeHeader.birth_txg` `unit.rs:338`、块指针缓存 `PointerHead.birth_txg` `pointer.rs:49`、journal 点名项 `NamedUnit.birth_txg` `journal.rs:30`；layout 01 第 175/177/200 行「诞生代号」「出生 txg」） | I-1.2（块头写序已发布，未实现）、I-1.8（归并后版本全序，未实现）、I-3.5（引用区间的精确性，未实现）、I-3.6（deadlist 紧界，未实现）——四条均以"诞生代号该取什么值 / 该落在什么区间"为内容 | **全部未实现**（`invariants.md` 第 25/31/126/127 行状态列逐条写"未实现"） | `crates/singlefs-harness/tests/first_transaction_step_two_data_unit.rs`、`first_transaction_step_five_publish.rs`、`second_transaction_step_one_overwrite.rs`、`second_transaction_step_three_second_instance.rs`、`second_transaction_supplement_two_accounting_node_full.rs`、`second_transaction_supplement_two_row_publish_admission.rs` 等逐脚本断言诞生代号 = 3 / 4 / 5 这类具体值 | **不过** | 与 C374 立条前"只有验收断言盯着"同型：四条能约束该字段取值的不变量一条都没实现，checker 里除 I-9.14 判的是树表条目自己那份（#13，另一物理落点）之外，`walk.rs` 全文没有任何函数读取并判定单元头自身的 `birth_txg`（只在 I-7.8 的水位扫描里当"是否早于当前根"这个过滤条件用，`walk.rs:538`，不判它本身对不对） |
| 11 | 对象出生代 `object_birth`（`DataUnitIdentity.object_birth` `unit.rs:211`、`InodeRecord.object_birth` `records.rs:34`；layout 01 第 175/252 行「对象出生代」） | I-9.10（对象出生代三处一致）；I-1.1（块头自述逻辑地址）把它纳入五元组自描述 | 已实现，`check_pool_image` 内联判定（`walk.rs:1464-1468`，比 `data_unit_objects` 与 `inode_object_birth`） | — | 过 | 三处（数据单元头、inode 记录、墓碑）里前两处已判，墓碑那处功能未上线报「不可判定」，符合 I-9.10 正文口径 |
| 12 | 容器出生代 `container_birth`（`PackedIdentity.container_birth` `unit.rs:65`；layout 01 第 247 行「容器出生代」） | I-9.2（条目身份与子头相符）、I-1.1 | 已实现，`walk_inode_root`（`walk.rs:338-386`，四元组含 offset 26 的 container_birth 与子头偏移 61 逐字比对） | — | 过 | 内部条目引用的容器出生代与子容器头实际值逐字比对，判定函数已实现 |
| 13 | 树表条目诞生 txg `TreeTableEntry.birth_txg`（`records.rs:267`） | I-9.14（树表条目的诞生 txg 跨根不变） | 已实现，`walk::judge_release_generation_and_tree_table_birth` / `judge_tree_table_birth_txg`（`walk.rs:1050`、`1104`） | — | 过 | C374 本条已还清的核心成果，本次复核确认函数与坏镜像用例仍在（详见第三节） |
| 14 | 树表条目 `previous_snapshot_txg`（`records.rs:285` 恒写 0，`records.rs:318` 读出即丢：`let _previous_snapshot_txg = reader.get_u64();`） | 无（`invariants.md` 全文 0 命中"previous_snapshot_txg"/"前驱快照"） | 无 | 无——连往返之外的语义断言都没有 | **不过** | 比"只有验收断言盯着"更弱：这个字段目前连一条盯着它语义（非 0 时该等于 origin 快照 txg，`.claude/kb/decisions/05-快照-空间记账机制.md` 第 253 行已定项 9）的验收断言都没有，因为代码里读出来直接丢弃、写出去恒为常量 0（day-1 无第二个可写头，功能未上线） |
| 15 | 分配记录 `generation`（`AllocationRecordView.generation` checker 侧 `walk.rs:719`；实现侧 `allocator.rs:34`；已释放标志区分分配代/释放代，layout 01 第 285 行、layout 02 第 11 行） | 释放代：I-3.9；分配代（未释放时）：**无** | 释放代：已实现，`judge_release_generations`（`walk.rs:976`，`.filter(\|record\| record.is_released)`）——**只处理 `is_released` 为真的记录** | 分配代无判据，只有各脚本断言分配代=3/4/5 这类值；执行器侧已知欠账 `crates/singlefs-harness/src/history.rs:914` 原话："盘上 checker 判不出分配代没改（29 条不变量里只有 I-3.9 看代，只看已释放的；增补 2 收口表第 44 行）" | 释放代：过；分配代：**不过**（本轮已知样本） | `judge_release_generations` 的 `filter` 在源头就把未释放记录排除在外，I-3.9 正文与状态列都只提"释放代"，没有一条不变量管仍处于已分配状态的记录的分配代；与收口表第 44 行、`history.rs` 注释完全对应，确认清单没有列窄 |
| 16 | 记账代 `AccountingEntry.generation`（`records.rs:131`；layout 01 第 290 行「记账 key ... 代」） | 无（`invariants.md` 全文 0 命中"记账代"/"AccountingEntry"） | 无（`walk.rs` 未见任何函数比对记账条目的 generation 字段） | `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:926-927`：`assert!(rows.iter().all(\|row\| row.sequence == 1 && row.generation == CheckpointTxg(3)));` | **不过** | 与已知样本第 44 行同型：缺不变量本身，只有固定脚本上的验收断言钉住具体值，checker 一处未判 |
| 17 | inode 记录改动计数 `InodeRecord.change_count`（`records.rs:19`，类型是裸 `u64` 不是 `CheckpointTxg`；语义按 `.claude/kb/decisions/08-核心索引结构.md` 已定项 6 等于"最后一次改动所在发布的 checkpoint_txg"） | 无（`invariants.md` 全文 0 命中"改动计数"/"change_count"） | 无 | `first_transaction_step_five_publish.rs:692`：`change_count: 3,`；另 `first_transaction_region_bytes.rs`、`second_transaction_supplement_two_accounting_node_full.rs`、`second_transaction_step_one_overwrite.rs` | **不过** | 与已知样本同型；且这个字段在代码里连专属 newtype（`CheckpointTxg`）都没套，只是裸 `u64`——比其余代字段更弱一档 |
| 18 | 实例表行「所选根 txg」`InstanceRow.selected_root_txg`（T，`instance_table.rs:14`；layout 02 第 13 行） | I-1.2（提到 `(i, T_pub, W)` 发布判定谓词），I-3.8 只判 instance 唯一性与低于挂载根实例，不判 T | I-1.2 未实现；I-3.8 已实现但不含 T（`judge_instance_table_rows`，`walk.rs:426-461`，只读 `instances`/`below_mount_root`/`chain_record_last`，未比对 `selected_root_txg`） | `second_transaction_step_three_second_instance.rs:114/400`、`second_transaction_step_four_rollback.rs:170/176` 各按具体脚本断言 T 的值 | **不过** | 唯一能约束 T 取值的 I-1.2 未实现；I-3.8 的实现虽然把 T 读进 `self.instance_table_rows`（`walk.rs:441`），但函数本身没有拿它去判定任何东西 |
| 19 | 实例表行「已施加最大事务号」`applied_transaction_high_water`（W，`instance_table.rs:15`） | 同 #18，只在 I-1.2 未实现的谓词里出现 | 未实现 | 同 #18 两份测试 | 分不清 | W 本质是"事务号"计数器而非"txg/代"，是否算本判据管的"代字段"存疑；若算，判定与 #18 相同（不过），交人看是否入选 |
| 20 | 块指针缓存的出生 txg / 写序（`PointerHead.birth_txg` 与 `DataPointer.write_order`、`NodePointer.instance`/`birth_sequence` 相对**被指对象实际头字段**是否一致） | 无——`invariants.md` 没有一条登记"父级指针缓存值必须与子单元实际头字段一致"这条判据（区别于 #10/#12：那两条判的是字段自身取值范围，这一条问的是同一份数据在父子两处是否一致） | 无：`read_referenced_unit`（`crates/singlefs-checker/src/image.rs:301`）只判 I-2.1 校验和；`walk_extent_leaf` 的 `five_tuple_holds`（`walk.rs:523-528`）只比对树/对象/锚点三项，不含指针自带的 `birth_txg`；`walk_inode_root` 的 I-9.2 只比对四元组身份，不含指针自己的 `instance`/`birth_sequence` | 未见专门的验收断言，大概率只有 `pointer.rs`/`unit.rs` 内部的序列化往返单测 | **不过** | 缺不变量本身（未登记）也缺 checker；指针里这份"缓存值"若被单独篡改成一个仍然合法但错误的代号，位置条目校验和不受影响（校验和罩的是被指单元内容，不是指针头部字段自身），I-1.1/I-9.2 现有判定範围都够不着这一项 |

## 三、另判一件：I-3.9 与 I-9.14 自己的射程对已在册的镜像形态说了什么

读 `crates/singlefs-checker/src/walk.rs` 的 `judge_release_generation_and_tree_table_birth`（第 1104 行起）与其两个子函数 `judge_release_generations`（976 行）、`judge_tree_table_birth_txg`（1050 行）。

**结论：有一类会被判「不适用」而其实该判。**

`judge_release_generation_and_tree_table_birth`（`walk.rs:1104`）依次调用 `judge_release_generations`（判 I-3.9）与 `judge_tree_table_birth_txg`（判 I-9.14）两个子函数。逐句核过控制流，两条不变量各自被挡住「不适用」的条件并不相同：

- **候选集只剩一条根**（`candidate_indexes.len() < 2`，`walk.rs:1112-1121`）：这一处两条不变量**一起**报「不适用」并整体 `return`，`judge_tree_table_birth_txg` 都不会被调用。
- **候选集里有根的引用集合走不完**（`references.is_complete` 有假，`judge_release_generations` 内部 `walk.rs:998-1003`）：这一处只让 **I-3.9** 报「不适用」并从 `judge_release_generations` 内部 `return`——这个 `return` 只跳出这一个函数，调用方 `judge_release_generation_and_tree_table_birth` 紧接着仍会照常调用 `judge_tree_table_birth_txg`（`walk.rs:1157`）。再核 `references_of_root`（`walk.rs:837-895`）：树表条目的 `birth_txg` 在遍历树表本身那一层就写进 `tree_table_birth_txg`（`walk.rs:871-873`），发生在深入某棵树内部、可能把 `is_complete` 置假的那一步**之前**；只要树表单元本身可读，`is_complete` 因为某棵树内部（extent/inode/分配记录/记账）走不完而变假，不影响树表条目 `birth_txg` 已经被记下来。**所以 I-9.14 不受这一处「引用集合走不完」影响，仍会正常判定。**

对 `candidate_indexes.len() < 2`：已被 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 显式钉住预期（`checker_known_bad_images.rs` 里 I-9.14 在第一个事务之后的干净镜像上判 `NotApplicable`）；核实 mkfs 的树表创世单元传入空条目数组（`crates/singlefs-core/src/make_filesystem.rs:213-225`，`&[]`），mkfs 版本的树表没有任何树 ID，与第一个事务写出的 7 条不会在「同一树 ID 出现在两个不同树表单元里」这个判据上产生交集——这一类「不适用」是设计使然，不是漏判。

对「候选根引用集合走不完」只挡 I-3.9 这一处：已注册的坏镜像（`known_bad_images`、`known_bad_images_after_the_overwrite`、`known_bad_images_of_overlapping_allocation_records`）里没有一份构造过「某条候选根（不一定是最新根）的分配记录树或树表读不全，同时镜像上确有一处释放代真的错了」这种组合；层 0 崩溃点重放（到 E 的固定脚本）挑的 newest 根按恢复语义必然完整，较早的候选根一旦已提交不会再被后续崩溃撕裂，所以目前的固定脚本大概率触碰不到这个分支——但这只是「现有脚本没有构造出这种组合」，不是这条判据把它排除在射程外。**结论：这是一类真实存在、目前没有任何已注册坏镜像覆盖到的空当，只影响 I-3.9，不影响 I-9.14。** **这一条的推翻条件**：造一份镜像，回退候选集≥2 条根，某条非最新候选根的分配记录树或树表读不全（触发 `references.is_complete=false`），而最新根身上确有一处真实的释放代错误——若 I-3.9 报"不适用"而不是"违例"，此判定成立；若门禁/checker 之后改成按可读候选子集降级判定，此判定被推翻。

## 四、没做什么

- 只判不改：没有改动 `.claude/kb/invariants.md`、`crates/singlefs-checker/`、`crates/singlefs-core/` 或任何测试文件。
- 登记给 sweep 的门禁阶段只有 `64-moved-paths.sh`（现查命令与结果见本节末尾），本轮不改任何文件，该阶段跑出的绿与本轮无因果关系，只是如实交代。
- #1（超级块槽世代号）、#4/#6（根/journal 记录自身 checkpoint_txg）、#19（W）三类判定写的是「分不清」，没有强行归类，交主 agent 或用户定这三类算不算本判据管的"代字段"。
- #20（指针缓存值一致性）与 #14（`previous_snapshot_txg`）是本轮延伸出的新发现，不在 C374 原文点名范围内，只按"它管的对象：盘上格式里每一个代字段"这句原文字面纳入；是否要另开欠账条目（`checks-owed.md`）由主 agent 定，本轮未新增任何 kb 条目。
- 未逐字节验证 #9/#10 里六份验收测试文件断言的每一处具体数值是否与当前代码行为一致（只核实了断言存在、字段名对得上），当前判定只依据"有没有断言"，不依据"断言的数对不对"。
- `.claude/gate.d/stage-owners.tsv` 现查：
  ```
  $ awk -F'\t' -v me="sweep" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv
  64-moved-paths.sh
  $ nice -n 19 bash .claude/gate.d/64-moved-paths.sh
    ✓ 全仓没有指向登记过的旧路径的地方（扫了 4825 个文本文件、6 条搬迁登记）
  exit=0
  ```
  本轮没有发现任何"旧值是路径"的情形（见第二节各行"理由"列，均按值/字段判定，不涉及目录或文件路径搬迁），登记的 6 条搬迁与本轮判定对象无关，该阶段本就该绿。

## 五、判定汇总

过 8 项（#2/#3/#8/#9/#11/#12/#13/#15 释放代那一半）；不过 9 项（#5/#7/#10/#14/#15 分配代那一半/#16/#17/#18/#20）；分不清 4 项（#1/#4/#6/#19）。共 20 个对象、21 条判定（#15 一分为二）。已知样本（#15 分配代）判「不过」，理由与收口表第 44 行、`crates/singlefs-harness/src/history.rs:914` 一致——确认清单没有列窄。新发现的「不过」项（#10 单元头诞生代号、#14 previous_snapshot_txg、#16 记账代、#17 改动计数、#18 实例表行 T、#20 指针缓存一致性）与 C374 立条前的原问题同型：要么缺不变量登记，要么不变量已登记但 checker 未实现，只有各脚本的验收断言按具体数值盯着。
