# 代码轮第三轮（m3-formal-code-r3）云端辩方腿报告

日期：2026-10-03。腿：云端辩方（Sonnet），格 K1–K4。被复核的判决：`research/prompts/m3-formal-code-r2-main-verification.md`。
代码读冻结副本 `/tmp/claude-1000/m3-formal-code-r3/tree/`，下文代码引文的路径都相对它；kb 引文写 kb 文件自己的行号。没编译、没跑任何测试。

<!-- doc-lint:not-numbers K1 K2 K3 K4 L1 L2 L3 L4 L5 L6 H1 G1 U1 U2 U5 O1 O2 O3 F6 R2 KF KG KJ1T C331 C662 -->

## 一、开工核对（命令与原样输出）

- 清单核对：在冻结副本里 `sha256sum -c --quiet research/prompts/m3-formal-code-r3-snapshot/crates-sha256.txt`，无输出、退出码 0。
- 变异表静态检查：`nice -n 19 python3 research/scripts/crates-mutation-rows.py static-check /tmp/claude-1000/m3-formal-code-r3/tree`，末行 `CHECKED	2020	228	9`。
  ANCHOR 红 13 行：1369（冻结副本里没有 `litmus/` 目录，是拷贝范围造成的，不是仓里的问题）、E168 的 1829、1830、1839–1842、1894、1895（别的会话）、E165 r3 的 1898、1903（命中 2 次，正文已记）。
  本轮新增的变异行（RefFS H1 一族、13-3、13-4、辩方 D3、K4-1、E165 r4、E170、E171）一行 ANCHOR 红都没有。
- 读到的范围：正文、背景材料的小节清单、改动全文里 L1–L4 全部、L5 的 E165 r4 登记第三节与 5.3、L6 的 E170 全部与 E171 的观测与关一条那一段；
  E165 与 E171 两个装置的万行级源码只读了与问题直接相关的函数，没逐行过。

## 二、一览表

「站得住」= 复核后判决条目与落地的改法都对得上；「够不着」= 替旧写法辩护的理由够不到它要推翻的那一句；
「同样打中别的臂」= 判决条目打中的那个形状，在判决没有点名的别的臂上同样成立。

| 判决条目（第二轮判决） | 辩方判定 | 证据 |
|---|---|---|
| K3-a 第 1 种（I-8.1 三字段只在逐档路上分叉）→ 改法 K3-1（L4：`image.rs` 与 `crash_judge_checker.wgsl`） | **站得住**：落地与实现同序同判，没改过头 | 判决引的条款句在 kb 里：`.claude/kb/decisions/22-单元原子性怎么合成.md:333`「逐档读到自证过而池级字段越界的槽整池拒」；实现先判值：`crates/singlefs-core/src/recovery.rs:921`「system_configuration_values_this_reader_accepts(&bytes, &system_configuration)」、之后才核偏移 `crates/singlefs-core/src/recovery.rs:991`「if recorded_spacing != candidate_spacing {」；checker 现在同序：`crates/singlefs-checker/src/image.rs:664`「if let Some(disagreeing_field) =」先于 `crates/singlefs-checker/src/image.rs:674`「if geometry.slot_spacing != candidate_spacing {」；GPU 同序：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:221`「if (s.reading == SLOT_USABLE && !journal_ring_geometry_fields_agree(s)) {」。三字段等式两边同一组（F = 3、在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096），实现把它们放在值表最后：`crates/singlefs-core/src/recovery.rs:845`「放在 [`system_configuration_values_this_reader_accepts`] 的最后」 |
| 辩方立场：替「checker 逐档先核偏移」辩护 | **够不着**，但读法有一处真空，见第三节 K3 | 旧 checker 对 `value_refused_by_this_reader` 那几个成员本来就不核偏移：`crates/singlefs-checker/src/image.rs:643`「与槽 0、槽 1 一个处置；它记的槽距可能正是越界的那一格，不核偏移。」；旧写法只在 I-8.1 三格上破例。条款句里「不核偏移」只点在 incompat 上：`.claude/kb/decisions/22-单元原子性怎么合成.md:333`「incompat 位不认得的槽不可择、不核偏移」，这是旧写法唯一的文字抓手（只提 incompat 就是只在 incompat 上免核），但「越界整池拒」那一句没带偏移条件，两种读法都读得通，判决取的是字面更近的一种 |
| K3-a 第 2、3 种（S/R 越界、incompat 加格式版本）记欠账、不改 | **站得住**；同样打中别的臂：不管改 checker 还是改实现，这几种都还在 | 这几种的病根是两个读者对字节的分类先后不同（见 `research/prompts/m3-formal-code-r2-opus-output.md:168`：「R、S 越界照旧是这一槽不可择」），与「逐档先核偏移还是先判值」无关：把实现改成先核偏移，S 越界那一格在普通读法上照样分叉 |
| 辩方 D3（`walk.rs` 择根键相同取读根环次序最前）→ 落地 | **站得住**（CPU 一臂）；**同样打中别的臂**：GPU 判器择根仍取后一个 | `crates/singlefs-checker/src/walk.rs:6722`「Some((_, newest_key)) if choice_key <= newest_key => newest_so_far,」只在严格更大时才换；与核同序：`crates/singlefs-replay-prefix/src/lib.rs:4005`「Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0」；GPU 那份没跟着改：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2869`「// 择根：(txg, 实例) 最大，相等取后面的（max_by_key 取最后一个最大者）」，比较式在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2874`「if (!root_key_less(r.txg, r.instance, newest_root.txg, newest_root.instance)) {」 |
| K4-a 借槽距用例加问 checker + 变异行 | **站得住** | 变异行在 `crates/mutations.tsv:2014`（名字「K4-1：checker 槽 0 无效的盘不借别的盘槽 0 记的槽距、落到逐档…」），静态检查原文命中一次；用例在 core 一臂与 checker 一臂各问一次（改动全文 `core_review_tree_table_duplicates_and_slot_one_search.rs` 一节）。GPU 一臂仍没有变异行，判决已记进 K3-2 那笔欠账 |
| K2-c（暖机与抬 F 拆开）→ 改法 K2-1（L3） | **站得住**，有一处没量：改后在那 4 107 态上「F 不变」没有复量 | 界按内容认，今天与按段位置认同值，因为抬 F 那一串第一个系统配置写之前恒有一道屏障：`crates/singlefs-core/src/transaction.rs:566`「// 发布末尾已有一道屏障（C577）：紧跟在同一个写入口的发布之后时下面这一道前面没有写，写入口不发它；留着它，不靠调用方前面是什么。」。判不判按持久集合：`crates/singlefs-harness/src/mount_transparency.rs:313`「.get(first_write..)」 |
| 辩方 D4（推论体注释） | **站得住**，已落 | 改动全文 `crates/singlefs-replay-prefix/src/lib.rs` 一节只改注释一处，把「≤ 落点（J1）」改成由 M2 推得 |
| K4-b（E165 r3 锚点 1812） | **站得住**，但同一形在 r4 又出现 | 静态检查 1898、1903 命中 2 次（正文已记，排第 14 批），1812 不在红单里 |

## 三、逐格复核

### K1　RefFS 接调用点（L1、L2）

**H1 各条在包里的落点与钉它的变异行**（只读码核，没跑）：

| H1 条目 | 包里落点 | 钉它的变异行（`crates/mutations.tsv`） |
|---|---|---|
| a 返回成功的发布调用、fsync 抬 lo，lo = 它自己的末尾 | `crates/singlefs-reference-filesystem/src/lib.rs:219`「operation.outcome == EnteredOutcome::Succeeded && operation.kind.ends_a_publish()」 | 1965、1968 |
| b、c 回退返回成功抬 lo，回退算发布末尾 | `crates/singlefs-reference-filesystem/src/lib.rs:164`「EnteredKind::RollBack => true,」 | 1969、1858 |
| d 原样重发结局 | `crates/singlefs-reference-filesystem/src/lib.rs:342`「OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting => {」 | 1970 |
| e 版本号、起点是第 1 版 | `crates/singlefs-reference-filesystem/src/lib.rs:272`「versions_up_to_the_last_recovery: vec![VisibleState::NoFile],」 | 1966、1967、1971 |
| f 恢复读出、重新起头的那一份是一版 | `crates/singlefs-reference-filesystem/src/lib.rs:465`「.push(recovered.clone());」 | 1972 |
| U2 抛弃最新根的那次恢复不判、重新起头 | `crates/singlefs-reference-filesystem/src/lib.rs:440`「pub fn restart_after_a_recovery_that_abandoned_the_newest_root(」 | 1973 |
| 翻译层：归属 / 冷启动 / 根映版本 / 二次崩溃 | `crates/singlefs-harness/src/reference_filesystem_translation.rs:678`「.find(|(_, stream_length)| *stream_length > closing_stream_index)」 | 1974、1975、1976、1977 |

一处没钉：`ends_a_publish` 的发布调用与 fsync 那一臂（`crates/singlefs-reference-filesystem/src/lib.rs:162`「EnteredKind::ExplicitPublish | EnteredKind::Fsync => true,」）没有「改成 false」的变异行
（`grep -nF 'ExplicitPublish | EnteredKind::Fsync' crates/mutations.tsv` 零命中）；单测会红（`after_a_successful_fsync_its_own_index_…` 在 lib.rs 里），只是这一臂没进变异表。低。

**崩溃时刻归属在每一种步上是不是只有一种读法**：是。归属只看「第一个跑完时流长大于关段下标的步」（上表 678 行），
而步分三类：①发布、覆盖写、零单元发布、回退——交 `Submit` 事件，崩溃算在最后进序列的那个操作里（发布调用或回退自己，没返回、不抬 lo）；
②挂载、抬 F——不交事件（`crates/singlefs-harness/src/reference_filesystem_translation.rs:381`「HistoryOperation::CloseAndMountWritable | HistoryOperation::RaiseRollbackFloor(_) => {」），崩溃算在「序列里的操作都返回过之后」；
③冷启动恢复不写盘，流长不涨，关段查找永远选不到它；抛弃最新根的那次挂载整步不判（`crates/singlefs-harness/src/reference_filesystem_translation.rs:427`「crash_inside: CrashInsideTheStep::NotJudgedBecauseItsRecoveryHadReadFaults,」）。
起点段（含 mkfs 之后的取号、暖机、第一个文件）整段算一步：崩在暖机里时允许集合是 {没有文件、第一个文件}，比真能读到的松一格，只漏、不误红，O1 一侧同样松。

**RefFS ∧ O1 在合法历史上 RefFS 会不会先误红**：读不出内容两边同红——`crates/singlefs-harness/src/model.rs:2252`「ObservedReadBack::Failed { what } => {」（O1 判官）与 `crates/singlefs-harness/src/reference_filesystem_translation.rs:822`「ObservedReadBack::Failed { what } => Err(what.clone()),」（RefFS 判红）；
lo 只被「返回过」的操作抬，崩溃号取没返回的那个；崩溃状态的候选段要求整段的写都落在最后一个跑完的操作之前：
`crates/singlefs-checker-tier/src/crash_injection.rs:2592`「(marks.after_make_filesystem..marks.after_the_last_finished_step)」，所以崩溃注入里翻译层没被喂过「停下那一步」——
`note_the_step_the_history_stopped_in` 只有故障注入调（全仓 grep 两处：定义 `crates/singlefs-harness/src/reference_filesystem_translation.rs:141`、调用 `crates/singlefs-harness/src/fault_injection.rs:2358` 一带）。
没找到合法历史上 RefFS 红而 O1 绿的路径；要推翻它，需要一个「已返回的发布之后、读回比 lo 更旧的内容、而 O1 的持久根判据放行」的合法崩溃状态——D16 已定项 7 的持久顺序（轮换持久之后才返回）排除它。

**坏实现上会不会漏（该判的弃权）**：崩溃注入里可达的弃权只有：回退目标根不在 m 表（`crates/singlefs-harness/src/reference_filesystem_translation.rs:348`「return Err(TranslationStopReason::TheRollbackTargetRootHasNoVersion {」，正好是实现报了一条没写过的根；这一步由历史执行器自己的模型判过，O1 在）；
抛弃最新根那一步里的崩溃；第一次崩溃没判绿时的二次崩溃。三者都对着「RefFS 判不了」的真空，不是对着「该判而没判」。

**故障注入一律弃权之后 O1 一票撑不撑得住**：替旧写法「读故障档照判」辩护，够不着：读故障（以及翻位、写被吞这些）让较新的根暂时读不出时，
实现今天仍按择新取读得出的最新根，落到比已确认更旧的版本是 C331 那一形，kb 记着没修：`.claude/kb/checks-owed.md:272`「择根倒挂压过已确认的写」，改取 R2 还没进代码 `.claude/kb/checks-owed.md:525`「带故障的合法历史一族照取甲生成」。
照判会把这类合法旧读判红（执行员 13-4 报告里读故障档那条变异的红例：读回发布之前那一版，RefFS 红；腿外读到，非入库证据）。O1 一票今天的判法保持：`crates/singlefs-harness/src/model.rs:2262`「if let Some(persisted) = newest_persisted_root {」那条「不许恢复到比盘上已持久的最新根更旧」仍在，
漏掉的是「已返回的写在故障后读不回」那一类，它本来就是 C331 的未定义射程。推翻它的现象：E165 r4 的 KF 族合法历史（承诺内标签）上出现「落到比最后一次已确认更早」的态——那样「读故障档该照判」就有了抓手。

### K2　二次崩溃 F 的界（L3）

**界按内容认会不会在合法历史上认错位置**：不会错成误红，错也只往「漏判」一边偏。界 = 第一个「系统配置槽写、解析出的 F 等于第一串抬 F 第一次发布根里的 F」：
`crates/singlefs-harness/src/mount_transparency.rs:292`「write.kind == StepKind::SystemConfigurationSlot」。取号与暖机轮换写的是池生效 F（抬 F 之前的），抬 F 才写 `raised_floor.max(pool_effective_floor)`（`crates/singlefs-core/src/transaction.rs:388`「raised_floor.max(pool_effective_floor)」），
只有新 F 恰等于生效 F 时两者才相同（入口只拒新 F 低于生效 F 的：`crates/singlefs-core/src/mount.rs:2450`「if new_floor < effective_floor {」，相等放行）；那时取号那一写就带这个 F，界提前到挂载第一个系统配置写，F 判不判只往「不判」偏，而 F 本来也没变。找不到写时是 `expect` 的 panic，不是静默：`crates/singlefs-harness/src/mount_transparency.rs:298`「.expect("抬 F 那一串第一次发布之前先把新 F 写进每块盘的系统配置，那几个写在这次挂载的写表里");」。
按段位置认也成立：抬 F 第一个系统配置写之前有屏障（`crates/singlefs-core/src/transaction.rs:566`那句 C577 注释与 `crates/singlefs-core/src/transaction.rs:569`「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」），
所以内容界与段界今天同点（执行员 13-4 报告第 4 条担心的「要假定恒有一道屏障」，今天的写入口满足它）。

**按持久集合判比「抬 F 第一个写起」细的那一格在撕裂第三态上同不同**：二次崩溃的态只有「更早的段整段持久、当前段按子集、更晚的段一个都没持久」
（`crates/singlefs-checker-tier/src/crash_injection.rs:2360`「persisted_in_the_mount[..first_write_of_the_segment].fill(true);」），没有撕裂第三态——撕裂只在第一次崩溃的态里；
一个镜像上没有任何带新 F 的写持久，F 就只能是抬 F 之前的，所以「照 U5 前半句判不变」是由构造得出的，不是推的。判法 `crates/singlefs-checker-tier/src/crash_injection.rs:2148`「let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);」。

**摆点三段不变、抬 F 的挂载上暖机只摆一点，缺口多大**：摆点每相位一个（`crates/singlefs-checker-tier/src/crash_injection.rs:2258`「let picked = usize::try_from(」），暖机与抬 F 同一相位。
这一改只增加被判的态：旧写法下这一相位的 F 一个都不判，新写法下落在抬 F 第一个写之前的点判「不变」。相对于「拆成四相位」的理想缺口：执行员 13-4 报告第 155 行按攻方 fms12 那次挂载写的是暖机 18 个写、抬 F 34 个写
（腿外读到，非入库证据），按均匀挑点约 35% 的点落在暖机里被判 F。没拆第四相位的理由也在那份报告：`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs` 与两条库单测对每个相位要求至少一点，加第四相位要连它们一起改。

**一处没量**：改后「那 4 107 态 F 不变」没有在改后的实现上复量（执行员报告自己写了没复量；正文第二节也写 checker 档没跑）。判决把它当作「按 U5 前半句判」，构造上成立（上一段），但提交时的 checker 档快档没有一个钉死「推了抬 F 的挂载」的用例（今天快档多跑 4 GiB 盘、不推抬 F，执行员报告第 161 行）。
推翻它的现象：提交时的快档或主 agent 复跑攻方 `k2one` 原型在这 4 107 态上读到 F 生效值 ≠ R1 的；那是实现的发现，不是补丁错。

### K3　checker 逐档与择根（L4）

**第二轮攻方那 13 种手放槽还剩哪几种结局不同**（读码加对第二轮攻方表，没复跑；复跑要编译，按定义不做）：
表在 `research/prompts/m3-formal-code-r2-opus-output.md:149-163`。L4 之后两行「在飞上限 + 1」（表第 154、155 行，标注「只在逐档路上分叉」）收成两边都整池拒：checker 逐档读到可用槽先判三字段、当场交回
（上面一览表第一行的 `image.rs:664` 与 `crash_judge_checker.wgsl:221`），实现本来就先判值；用例按这两种槽钉在同一份镜像上恢复与 checker 各问一次（改动全文 `system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 一节，两条用例）。
还分叉的是原表的 S = 17 三行（第 151–153 行，「同样分叉：恢复整池拒，checker 全绿」）、R = 4 一行（第 157 行，「同样分叉：恢复择世代号 9，checker 择世代号 1」）、incompat 加格式版本一行（第 159 行，「同样分叉（checker 报 I-7.13，恢复照择）」），
共 5 行，都是第二轮已记欠账的那几类（S、R 越界与 incompat），没有一行是 L4 新带进来的。**L4 没有制造新分叉**：它只把「checker 读成 Usable 而 I-8.1 三字段不等」的槽挪成「带不收之值」，实现这一边本来就在值表最后一格整池拒
（`crates/singlefs-core/src/recovery.rs:840`「journal_ring_geometry_fields_recompute_from_the_ring_length(slot, sizes.journal_ring_bytes)」）；同一个槽上更前面的格（格式版本、槽距上界、环长等）两边都先判，结局相同。

**同一种坏槽普通读法报 I-8.1、逐档读法报 I-7.13，会不会让 I-7.13 的计数或依据它的判法出错**：不会。依据 I-7.13 的判法只有两处：
`crates/singlefs-checker/src/image.rs:766`「pub fn judge_system_configuration_values_the_reader_accepts(」把一个 `Usable` 槽记成成立一格、一个带不收之值的槽记成违例一格并交回「有没有」（第 783、799 行），
`crates/singlefs-checker/src/walk.rs:6505`「checker 报违例、别的不变量同「挂不上的镜像」一样不作保」那一支；I-8.1 那一支与它同口径：`crates/singlefs-checker/src/walk.rs:6524`「checker 与上面 I-7.13 那一支同口径——报 I-8.1 违例、别的不变量一律不作保」。
所以「这个池整池不作保、其余不变量报不适用」两种读法一样，差的只是红的那条叫 I-7.13 还是 I-8.1。靠不变量名分类的下游（已知红形态、E171 的 O2 列）只关心非空与否；要撞上这个差别，镜像得同时满足「每块盘的槽 0 都不可用」与「槽 1 在自己的偏移上带不等的 I-8.1 字段」，合法历史走不到。

**`walk.rs` 新 fold 与 `choose_root` 在所有入参次序上同值**：「同值」的含义只能是「同一个入参次序下同值」（键相同取前一个，本来就依赖次序）。次序相同：checker 的根表 `crates/singlefs-checker/src/image.rs:1136`「pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {」
按 `root_slot_positions`（区域主序、区域内槽序）过滤自证过的根，实现的 `crates/singlefs-core/src/recovery.rs:1200`「pub fn every_root_ring_slot(immutable_sizes: &SystemImmutableSizes) -> Vec<RootRingSlot> {」也是区域主序、区域内槽序，
Verus 核 `crates/singlefs-replay-prefix/src/lib.rs:4005` 只在严格更大时换。三处同「第一个最大者」。变异行 2015 钉住 fold 的 `<=`。**GPU 判器那一份只跟着改了 I-8.1，没跟着改择根平局**，见第四节 Z1。

**替旧写法「checker 逐档先核偏移」辩护的读法真空**：`.claude/kb/decisions/22-单元原子性怎么合成.md:333` 一行里有两句话，
先说收槽条件「只收自证过、且自述的固定结构槽距等于所在偏移的槽」，后说「逐档读到自证过而池级字段越界的槽整池拒」，再单独给 incompat 加上「incompat 位不认得的槽不可择、不核偏移」。
旧写法的读法：「整池拒」只对「越界」的值，I-8.1 三条是一致性等式、不是范围，位置不在自己槽距上的槽先因不合收槽条件而不收；这与第二轮攻方自己写的话同向：「D22 已定项 16 第 5 句点名的是槽距与 `physical_block_size`」（`research/prompts/m3-formal-code-r2-opus-output.md:175`）。
判决取的读法：实现的值表里本来就有这三格（`.claude/kb/decisions/22-单元原子性怎么合成.md:339` 第 5 句「越界即拒」与 m3-inv-a 判决把 I-8.1 三格放进实现表），整池拒不挑偏移。
两种读法都读得通；L4 取第二种，理由是它与实现今天的行为一致、而且 checker 自己的文档早写了同一处置（`crates/singlefs-checker/src/image.rs:643`「与槽 0、槽 1 一个处置；它记的槽距可能正是越界的那一格，不核偏移。」）。
**同样打中别的臂**：把实现改成「先核偏移再判值」同样要改 S、R 那几格才算收齐，没有一条更省；所以替旧写法辩护不能推翻 K3-1，只是提醒这一句条款应补一个「先判值还是先核偏移」的字面（写回时的一个字，不是代码问题）。
推翻 K3-1 的现象：一个合法历史（mkfs 之后的盘）上，[4096, 1 MiB) 里出现自证过、带不收之值、又不在自己记的槽距上的槽——mkfs 对该段整段写 0（`.claude/kb/decisions/22-单元原子性怎么合成.md:176`「先把每块盘的 [0, 根环基址) 整段写 0」），今天没有这样的写者。

### K4　三个实验装置（L5、L6）

**E165 r4 生成器造的「带故障合法历史」今天写路径写不写得出**：登记明说生成器是可信件、不调 `mount.rs`、`transaction.rs`，只许照条款文字写（`research/prompts/e165-r4-prereg.md` 5.3 第 6 条），所以逐条对 `crates/` 今天的样子，对得上的：

| 生成器的写法（登记 5.3） | 今天的写路径 | 对得上？ |
|---|---|---|
| N-配置：任一块盘两槽读得出的不足 2 槽 ⇒ 真 | `crates/singlefs-core/src/mount.rs:4502`「let comparison = if some_system_configuration_slot_unread {」 | 对上 |
| 一次系统配置写：世代号 = 这块盘读得出的最大世代号 + 1，落 世代号 mod 2 | `crates/singlefs-core/src/transaction.rs:366`「let slot_generation = verified_on_this_device」、落槽 `crates/singlefs-core/src/transaction.rs:414`「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」 | 对上 |
| 取号那一刻有一槽读不出或撕裂就在第一个取号写之前拒 | `crates/singlefs-core/src/transaction.rs:839`「self_verified_system_configurations_of_every_device(pool)?;」（屏障之前的一核） | 对上 |
| 写行 txg = max(根 txg, 记录 txg) + 1，条款允许的最小值 | `crates/singlefs-core/src/mount.rs:1148`「fn first_txg_of_new_instance<Device: BlockDevice>(」；条款 `.claude/kb/decisions/23-journal的角色与格式.md:371` 注 3 的下界；登记第三节自己写了生成器取最小值、比实现可能小（`research/prompts/e165-r4-prereg.md:457`「取条款允许的最小值」） | 条款内对上，实现可更大，只在「最新根与它的全部记录同时读不出又是 T1」那一种组合里分得开，这一族造不出（登记自述） |
| 取甲，不取 R2 | `.claude/kb/decisions/23-journal的角色与格式.md:392`「这里的实现还是 2026-09-28 取甲的那一版」，欠账 `.claude/kb/checks-owed.md:525` | 对上 |

一处建模选择要点明：生成器把 T1（第一次读读不出、之后都读得出）落在 N-配置那一遍读系统配置槽上；今天的挂载在读阶段之前已经选过一次系统配置
（`crates/singlefs-core/src/mount.rs:4593`「let reread = read_stage(&cache, devices, system_configuration)?;」收的是已选好的 `system_configuration`），所以真设备上这次「第一次读失败」会先落在选系统配置那一读上。
对盘面的影响为零——槽读不出不影响根表与版本，往下走用哪一遍都是同一版——所以造出的历史在今天的写路径上照样写得出（任意瞬时读故障图样都是合法环境），只是计数 `reread_mounts` 的口径是生成器的、不是实现的。推翻它：用一个只让「选系统配置那一读」失败一次的设备跑真挂载，盘面与生成器的对应态不同。

**KG 拿掉的两道保护是不是条款里那两道**：登记 KG 一行（`research/prompts/e165-r4-prereg.md:542`「**KG**：不判 N-配置、不判取号那一刻」）拿掉的是 D23 已定项 14「可写挂载读到的更新状态读不出」那一整段
（它自己就把 N-配置与「取号那一刻按同一个判不出」写在一段里：`.claude/kb/decisions/23-journal的角色与格式.md:392`「取号那一刻按同一个判不出（R2，见 D22（单元原子性怎么合成） 已定项 8）判：判不出就在第一个取号写之前拒」）与注 3 的写行 txg 看记录。
两道 = 闸（含取号那一刻）+ 写行 txg；闸里「重读一次」同属第一道。对得上。

**E170 新模式与旧模式输出不变**：旧词 `bad-states` 取 `BadTableVersion::FirstRun`：P8 期望仍是 `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:2019`「Self::FirstRun => "not_in",」、不造 P10（`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:2937`「BadTableVersion::FirstRun => None,」）、
`config` 行 `mode=` 仍写 `bad_states`；P10 排在表末尾、不抽随机数，单测对同一种子逐行比两版（改动全文单测 `the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base`）。站得住。

**E171 逐行打变异时基线、阳性对照、关一条开关会不会让某一列静默不判**：关一条写成明字而不是「没红」：O2 列 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:2229`「None => ("off", "-".to_string()),」，O1 列走 `CrashJudgeVerdict::Off`（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1952`「let crash_judge = if !model_is_enabled {」），
不适用也是独立的一格 `na`，不是绿。O1-崩溃判官的阳性对照只在翻到一次之后才清「待翻」标志（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1851`「controls.crash_judge_control_pending = false;」），中途遇到不适用的态不会白耗它。
一处低风险的口径：O2 列只数 `Violated`（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1777`「fn violated_invariant_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {」），
一个镜像上若每条不变量都是「不适用」而没有一条违例，O2 列会写成绿；我读过的整池不作保的几支（`crates/singlefs-checker/src/walk.rs:6505` 起）都伴着一条违例，没找到这种镜像。推翻它：任一镜像上 `check_pool_image` 全不适用且零违例。

## 四、判决没点名、这一轮新冒出的形态（零轮，不再开轮，交判决的交用户表）

- **Z1　GPU 判器择根平局仍取后一个，CPU 的 checker 已改成取前一个**（同一形，D3 打中 `walk.rs` 一臂，没打到 GPU 那一臂）：
  `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2869`「// 择根：(txg, 实例) 最大，相等取后面的（max_by_key 取最后一个最大者）」，比较式 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2874`「if (!root_key_less(r.txg, r.instance, newest_root.txg, newest_root.instance)) {」键相等时换；
  kb 的字是前一个：`.claude/kb/decisions/22-单元原子性怎么合成.md:34`「同键取读根环次序最前的」。今天够不着：键相同要两个自证过的根槽 (txg, 实例) 完全相同，而根的槽位由 txg 定（I-7.17），层 0 与崩溃放量的合法流造不出，只有手造镜像才有；
  所以不是误红、是「GPU 与 CPU 在手造平局镜像上走读的根不同」。改法（零轮、被攻过零轮）：比较式换成严格更新才换，即把条件写成 `root_key_less(newest_root.txg, newest_root.instance, r.txg, r.instance)`，注释随改；再加一条手造同键两槽的 GPU 与 CPU 同判用例。
  推翻它：在 GPU 判器与 CPU checker 上放同一份同键异内容的镜像，两边择的根一致（那我读错了 wgsl 的比较方向）。
- **Z2　同形的几处测试侧读法**：`crates/singlefs-harness/src/history.rs:2620`「/// 按 checker 的读法：根环里最新那条根（(txg, 实例) 最大）带的 F；读不到时 None。」下面 `crates/singlefs-harness/src/history.rs:2628`「.max_by_key(|view| (view.checkpoint_txg, view.instance))」键相同取后一个，注释却写「按 checker 的读法」。
  同形还有 `crates/singlefs-harness/src/mount_transparency.rs:201` 与 `crates/singlefs-checker-tier/src/bad_disk_input.rs:1537` 两处（`grep -rn max_by_key` 命中）。都是测试支撑、只在同键手造镜像上才分得出，低。
- **Z3　翻译层把「没返回的步」当「成功返回」，注释写的是「按在途算」**：`crates/singlefs-harness/src/reference_filesystem_translation.rs:299`「| StepEndingSeenByTheTranslation::NeverReturned => OperationOutcome::Succeeded,」，
  而 `crates/singlefs-harness/src/reference_filesystem_translation.rs:140`「/// 没有结局（panic）的按在途算。」。今天被遮住：崩溃注入从不把停下那一步交给翻译（上文 K1，候选段全在最后一个跑完的操作之前），故障注入的重开一律弃权。
  一旦第 14 批把「注入点没撞上的历史照判」落了，这一步在 panic 的历史上会让 RefFS 抬 lo、把没返回的发布当已确认，合法重开读回旧版本就会判红。改法：`NeverReturned` 与 `ReturnedAnErrorAfterItMayHaveWritten` 同取 `FailedAfterWriting`（两可）。零轮。
  推翻它：第 14 批的改法让 `NeverReturned` 不再走到这一臂，或有一条用例在 panic 的历史上照判仍绿。
- **Z4　变异表没有 `ends_a_publish` 的发布调用与 fsync 那一臂**（K1 一节已写），单测在，低。
- **Z5　D22 已定项 16 的字面缺一个「先判值还是先核偏移」**：L4 取了先判值，条款 `.claude/kb/decisions/22-单元原子性怎么合成.md:333` 没把次序写出来（K3 一节「读法真空」）。写回时补一个字，不是代码问题；与第二轮判决第三节第 6 条的新欠账（S、R 越界、incompat 先后）同一笔里写最省。

## 五、没做什么

- 没编译、没跑任何测试与装置，包括 harness 档；K3 的 13 种槽与 K2 的 4 107 态只对着第二轮攻方的表和代码读，没复跑。
- 没读完的：`e165_replay_bounded_enumeration.rs`（17 625 行）与 `e171_three_oracle_collusion.rs`（5 670 行）的生成器与驱动主体，只对着登记第三节的七条事实与观测/关一条那一段逐条对了代码；E165 r4 各格的钉绝对值（9401、10517 这些数）没核。
- 没读完的测试：`reference_filesystem_translation_of_histories.rs`、`mount_transparency_judged_field_by_field_after_a_writable_mount.rs`、`fault_injection_fast_tier.rs` 的用例体只看了名字与变异行对应关系。
- K1 里「RefFS 与 O1 合起来不先误红」是按代码读出来的论证，没有一个现跑的合法历史样本；推翻条件写在各段末尾。
- GPU 判器只读了逐档函数、I-8.1 三字段等式与择根平局那几处，别的段没读。
- 不替主 agent 撤销或维持判决；Z1–Z3 是零轮形态，由主 agent 判进不进交用户表。
- 没写任何仓内文件，除这份报告。
