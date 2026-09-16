# 发布 B（覆盖写 + 释放）那批代码三方对抗第二轮：主 agent 核实（2026-09-16）

第一轮判决 `research/prompts/m2-code-r1-main-verification.md` 第四节按跑前条款要「X1 / X2 / X4 打中 ⇒ 再攻一轮」。第二轮换攻击面：攻方腿只攻四处改法本身（`m2-code-r2-opus.md`），辩方腿复核第一轮每一格的判决（`m2-code-r2-sonnet.md`），本地腿攻空闲独立计数与释放经映射两处（`m2-code-r2-local.md`，英文，两次抽样）。材料：`_m2-code-r2-diff.md`（从提交 d2aeb7d 起 crates/ 的全部 diff + 两个新测试全文，2167 行）；条款附录沿用 `_m2-code-r1-background.md`。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-harness/src/crash.rs`；用例 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`。

## 一、本地腿（s1 215 词、s2 275 词；字词损坏闸两次都过）

| 问 | s1 | s2 | 主 agent 核 |
|---|---|---|---|
| 1（空闲独立计数能不能骗过 I-5.2 与 I-3.1） | NO HIT：「the sum consistently equals the unit area size … because the checker directly uses these counters」 | HIT：假设释放路径「incorrectly increments free_slots and decrements allocated_slots」，两个计数一起漂而「I-3.1 holds」 | 两次相反。s2 的构造是一条对称变异（两个计数一加一减），I-5.2 确实骗得过——但它说 I-3.1 仍成立是错的：I-3.1 拿记账的已分配与环里全部有效根引用的并集比，已分配少 10 槽当场红（第一轮攻方腿的 M-X2 正是这条变异，副本上 I-3.1 红）。s1 的「the checker directly uses these counters」也不准：checker 读的是盘上记账行，不是内存计数。记「不稳定、没打中」；有用的一句是「对称变异只靠 I-3.1 拦」，写进判决第三节 |
| 2（经上一版映射查落点会不会查错） | NO HIT：上一版的映射节点是同一次发布写的，释放时那些单元仍分配着 | HIT：假设「the mapping node was written with an incorrect slot for a key」（写者 bug）⇒ 释放错的槽 | 两次相反。s2 的前提是映射节点本身写错，那是写者的 bug、不是释放路径的；而且这种镜像 checker 会红（映射条目的位置项带单元校验和，指错槽读不到对应的单元）。记「不稳定、没打中」 |
| 3（两盘不同槽的合法历史） | NO HIT | NO HIT | 两次一致：D2（RAID 条带策略） 已定项 10 两盘同槽，第一版没有两盘异槽的形态。没打中 |

## 二、辩方腿（Sonnet，`m2-code-r2-sonnet-output.md` 81 行；六条变异在副本上各跑一遍）

| 格 | 报告怎么判 | 主 agent 核 | 处置 |
|---|---|---|---|
| 第一轮判决一、二、三节的二十格技术判断 | 全部「判对」，X2 那一格「比判决更强」：摘掉减法之后 checker 的 I-5.2 与空闲绝对值断言两条路都红 | 与入库装置上的读数一致（`I-5.2 … Violated("盘 0：空闲 Some(3472883712) + 已分配 Some(376832) ≠ 单元区 Some(3472883712)")`） | 无 |
| X1-① 与本地腿 X2 第二次抽样的处置 | **处置不合条款**：条款「X1–X4 打中 ⇒ 改代码、补一条会红的用例、再攻一轮」，第四节只写了字、引了既有的 C283（准入失败时不先推发布就报 ENOSPC） / I-5.3（报出的空闲都兑现得了），没有新代码、没有新用例，也没有明写偏离 | **核实，判对**：条款原文没有例外；第一轮判决第四节确实没写「这里按 X6 待遇」 | 第一轮判决补第五节明写偏离与理由（谓词要步 5 的两样东西）；补弱形态用例 `released_placements_are_not_handed_out_again_before_reclaim_exists`，变异「`mark_released` 清位图」会红；步 5 还 |
| X5-① 的处置（立 C374（释放代与树表诞生 txg 只有验收断言盯着） + 收窄措辞） | 「门槛较低，勉强够」：没有新增独立对照，只是账记得更准 | 与第一轮判决说的一致，条款对 X5 只要「补对照」，立不立不变量步 3 前定 | 无 |
| 六条变异 | 各红在判决说的那条断言 / checker 上 | 与入库装置上的六次变异读数逐条相同 | 无 |
| `newest_persisted_root_txg` 改名 | 报告核出攻方腿写的旧名是修前的，改名是改法的一部分 | 对 | 无 |

辩方腿小结：技术判断零判错；打中的是处置与跑前条款之间的落差，已按上表补齐。

## 三、攻方腿（Opus，`m2-code-r2-opus-output.md` 351 行；副本装置上跑，数不入库）

| 格 | 报告怎么说（第几节） | 主 agent 核 | 处置 |
|---|---|---|---|
| Y1-①（Z1）映射条目在、落点指错 ⇒ `release` panic | 第二节：「查得到就把 `locations[0].slot` 原样交给 `PoolAllocator::release`」，副本两个探针（指向 60000 / 指向 A 的 extent 根）都 `should panic ... ok`；「同一轮判决里一边把 `build_index_node` 的 panic 换成 `PublishError`，一边新开了两条从映射内容直通 panic 的路」 | **打中，核实**：改动前 `placements_to_release_via_mapping` 只在查不到 key 时报错；`release` 的三条断言（无记录 / 释放两次 / 跨度不符）对映射给的槽号没有错误路径 | **改代码**：释放判定路径在动分配器之前逐个核映射查出的槽——在分配记录里没条目 ⇒ `ReleaseTargetNotAllocated`，已释放 ⇒ `ReleaseTargetAlreadyReleased`，记录的跨度与种类不符 ⇒ `ReleaseSpanMismatch`；`placements_to_release_via_mapping` 多收一个 `&PoolAllocator`。会红：用例 `release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking`（60000 ⇒ `ReleaseTargetNotAllocated`，A 的 extent 根 ⇒ `ReleaseSpanMismatch { recorded_span: 1, expected_span: 2 }`，分配器一条记录没改），把查记录改成「查不到就拿第一条顶」它红在 `allocator.rs` 的 expect 上（旧形态） |
| Y1-②（Z2）跨度不经映射、验收那句在跨度维上是恒等式 | 第二节：`placements()` 与释放路径的跨度都是 `identity.span_slots()`，「这一半是恒等式」 | **打中，核实**：两边确实同一个函数 | **改代码**：跨度改取分配记录里的 `span_slots`，与种类表互核（不符报错）；验收那句现在在槽维上是映射对提示、在跨度维上是记录对种类表，两维都是两条路 |
| Y1-③（Z3）`NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池 | 第二节：「返回时上一版八个落点已经全部改写成已释放 + 释放代 = 这次的 txg……而没有任何一次发布成立」；副本探针「拿同一个 `previous` 发两次」`should panic ... ok`；`NoSpaceFor` 自己没跑到，「记成推理」 | **打中，核实**：`publish_file_version` 的次序逐字如报告；主 agent 在入库装置上造出了 `NoSpaceFor` 那条路（把 A 开的段用到头、单元区别的空槽全标成已分配、只留 50182–50183） | **改代码**：`publish_file_version` 拆成准入（内容长度、分配记录树容量）+ `publish_admitted_file_version`；准入之后整个分配器拷一份，失败就换回去（每次发布约 450 KiB 的拷贝，第一版认）。会红：用例 `publish_running_out_of_space_midway_leaves_the_allocator_as_it_was`（B 拿到数据单元、第一个提交内生块拿不到 ⇒ `NoSpaceFor { ExtentRoot }`，记录、defer、位图、开放段全部退回），把「失败换回去」摘掉它红 |
| Y2（Z4）空闲改法整个撤回，全仓零判红零警告 | 第三节：四处改回 HEAD 的形态，`cargo test --workspace` 全绿、clippy 零 warning；「验收断言把期望写成了「容量 − 已分配」的形状」；「I-5.2 唯一能抓的是那一对语句里改了一条没改另一条」 | **打中，核实**：任何对值的断言都分不出「独立维护」与「由已分配现算」——两者在正确的系统上给同一个数；能分出的只有变异「摘掉减法 ⇒ I-5.2 红」，而那条变异的锚点 `self.free_slots -= span;` 在撤回之后不存在 | **立门禁**：`crates/mutations.tsv`（十三条：三方两轮每一处改法各一条「改回去它就红」的变异）+ `.claude/gate.d/59-crates-mutation-replay.sh`（锚点恰好命中一次；拷仓、逐条改坏、跑点名的测试、要求它红、还原）+ 红绿两份样本进 89 号。撤回空闲改法 ⇒ 那一行的锚点命中 0 次 ⇒ 59 号红。这是 show-me-test.md「存进仓的变异清单交给门禁复跑」在 crates 上的形态 |
| Y3 装不下报错 | 第四节：容量 812、头 135 含 `reserved_bytes`、「+8 × 盘数」在空发布 / 多对象 / 三盘上、报错时点——四问都核过，「没打中」；但同一入口还有三条「装不下 ⇒ panic」，`FirstFile.content` 超过 32 634 字节一行就走得到，副本探针 `should panic ... ok` | **核实**：`build_data_unit` 的断言在，`publish_overwrite` 对内容长度没有前置检查 | **改代码（Z5）**：`unit.rs` 公开 `data_unit_payload_capacity()`（32634），准入里内容超长报 `ContentExceedsDataUnit { bytes, capacity }`。会红：用例 `content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched`（32635 字节 ⇒ Err、分配器没动），把判定摘掉它 panic 在 `unit.rs` 的断言上。记账树 477 条 / 树表 81 条那两条门槛远，随步 3 的树分裂一起做，记进步 3 决策点 |
| Y4 择新序 | 第五节：`newest_persisted_root` / `choose_root` / 池级 checker 三处同序，根记录偏移 24 / 28 对，暖机根不算一版——「没打中」 | 与代码相符 | 无 |
| Y4-①（Z6）oracle 落在没版本的更新根上报「没有文件」一律放过 | 第五节：`(NoFile, None) => None`；副本喂 `effective (1, 5)`、`newest (4, 1)`、versions 3 / 4 ⇒ `None` | **打中，核实**：那一臂逐字如报告；恢复走到比最新持久根还新的根是合法的（journal 前缀施加），这一格可达 | **改代码**：那一臂改成「versions 里有比它旧的版本就判违例」；会红：单测 `newer_root_without_any_version_reporting_no_file_is_violation`（暖机根那一格照旧不违例），把条件改成恒假它红 |
| Y4-②（Z7）`JournalPolicy::Ignore` 那一遍恢复不过 oracle | 第五节：`ignored` 只进 `journal_differing_states`；副本上接上同一个 oracle，26 个状态零新增违例，靶向对照 ② 两遍都违例 | **打中，核实**：`evaluate_state_for_versions` 只判 `consulted` | **改代码**：Ignore 那一遍过同一个 oracle，另计 `ignored_violations` / `first_ignored_violation`，不混进 `violations`；两条流的快用例与全量用例都断言它为 0，靶向对照 ② 断言它为 1。会红：把计数改成 `+= 0`，对照 ② 红 |
| 顺带（Z8）逐盘核用集合比，同盘同槽两条记录放过 | 第六节：集合里没有东西要求一个槽只出现一次，副本探针两盘各多一条 `(50176, 2, 第 9 代)` ⇒ `true`；走读不判 key 严格递增 | **打中，核实** | **改代码**：逐盘先核槽号不重复；会红：单测 `two_records_for_the_same_slot_on_the_same_device_are_rejected`，把唯一性判定摘掉它红 |
| 记一笔 ① 实例维在 524312 个状态里零覆盖 | 第五节末：四条根槽写的实例代号 `[1, 1, 1, 1]` | 对，步 4 才有第二个实例；里程碑没写「层 0 判了实例维」 | 步 0 现状加一句 |
| 记一笔 ② 根环择新（txg 为主）与 journal 水线（实例为主）两序并存，oracle 只跟前者 | 第五节末 | 与代码和各自的条款相符，不是打中；步 4 一开就走得到 | 步 4 决策点加一句 |
| Y1 射程：释放路径读的是内存态、一个 reader 都不带 | 第二节 ⚠️ | 对：步 3 从盘上重建 `previous` 时，映射条目的位置项带单元校验和，释放前要按它核盘上那个单元 | 步 3 决策点加一句 |

攻方腿小结：八处打中全核实——七处改代码各配会红的用例（六条变异各自红在自己那条断言上，Z1 红在旧形态的 expect 上），一处立门禁（59 号）；Y3 / Y4 的算式、时点与择新序没打中。

## 四、判决

1. **本地腿**：三问没打中（两问两次抽样相反、构造都建在假设的写者 bug 上，一问两次一致）。
2. **辩方腿**：第一轮二十格技术判断零判错；两格处置不合条款 ⇒ 第一轮判决补第五节明写偏离、加弱形态用例。
3. **攻方腿**：八处打中 ⇒ 七处改代码 + 一道门禁。改了的文件：`crates/singlefs-core/src/allocator.rs`（`record_for`）、`crates/singlefs-core/src/transaction.rs`（准入 / 退回 / 释放路径三条错误 / 内容长度）、`crates/singlefs-core/src/unit.rs`（`data_unit_payload_capacity`）、`crates/singlefs-core/src/recovery.rs`（同盘槽号唯一）、`crates/singlefs-harness/src/crash.rs`（oracle 一臂、Ignore 那一遍）；用例三份：`second_transaction_step_one_overwrite.rs`（三条新用例）、`second_transaction_step_zero_layer0.rs` 与 `first_transaction_step_seven_layer0.rs`（Ignore 那一遍的断言）。
4. **变异表**：`crates/mutations.tsv` 十三条，59 号门禁复跑；每条的会红读数都在入库装置上跑出来过（本节与第一轮第三节的表）。
5. **再攻一轮**：第二轮打中的七处改法被攻过零轮。按跑前条款的精神它们还要再攻；本轮先把 59 号立起来（改法从此有东西守着），第三轮攻这七处改法的时机交用户定（并行线一开工前、或步 3 开工前）。
6. **不入库的**：攻方腿副本上的全部读数与探针；辩方腿副本上的六次变异读数（入库装置上各自重做过，读数相同）。
