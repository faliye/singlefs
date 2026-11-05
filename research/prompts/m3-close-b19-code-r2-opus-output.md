# m3-close-b19-code-r2 攻方腿（opus）报告（2026-10-30）

正文 `research/prompts/_m3-close-b19-code-r2-body.md`；第一轮判决 `research/prompts/m3-close-b19-code-r1-main-verification.md`。开工时 `research/prompts/m3-close-b19-code-r2-snapshot/` 的 `crates-sha256.txt`、`kb-sha256.txt` 两份清单对主工作区逐份 `sha256sum -c`，16 份全部 OK，下文代码行号取主工作区（与快照同 sha256）。禁读清单里的三份没读。

这一轮没建模型、没编没跑（理由见末节「这条腿自己的限度」）：全部结论是读码与手推，标「推的」。模型目录 `research/prompts/m3-close-b19-code-r2-opus-model/` 没建。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| U1 逐盘核挪到预演之前 | 没打中 | 逐盘核读盘走直读那一路，不进共用的那一份、不记两本账，预演每趟开头又清账；两拒同时成立时改报逐盘核那一拒，D22 已定项 8 与 C874 对先后没有约束；钉死的读序数 307、308 在两段之前，其余按无故障那一遍现算 |
| U2 改法二要不要 | **打中（推的）** | 补写盘在预演每一趟取号都读坏两槽、记两条读坏账；预演只要停在 C841 之外的错上，两本账就不空，重走到上限拒成「预演读坏」、点名补写盘的系统配置槽——「两本账都空照旧留给续做」那一支在有补写盘的池上永远走不到；盘面要一份能让续做确定地停在非 C841 错上的坏镜像 |
| U3 T2 断言的放处 | 没打中 | 新放处罩着两次算上限（`raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk` 那一次与 `raise_rollback_floor` 里那一次），抬 F 那一串别处不产生 `Recovery(NoValidRoot)`；`raise_rollback_floor` 一并交回是共享函数去掉断言的必然结果，与判决 T2 同向；文档漏列第三个调用处（取号之前只进内存那一趟） |
| U4 C928 解冻用例 | 没打中（丢 discard 那一问） | 三个调用点同进一个函数、冻结判在函数里；冻结期间再冻结走不到（发布路径第一道拒）；重发失败原样再冻结、待 discard 表不动。另记一处与冻结无关的：判孤儿头而扣下的段被取走后不回表 |
| U5 C929 | 推荐「照今天不判」 | 回卷之后那块盘的两槽字节与普通成员盘分不开（判别子观测不到），候选二在读的一侧做不出；两次挂载之间取号写与回卷只写系统配置槽，闸三四五读的环与单元区不变；候选一只在盘被池外改写时错，那与任何成员盘同一个暴露面 |
| U6 第一轮的方法 | 没打中（读码复核） | 挑 T3；读码核实「core 产品代码里 `discard_at` 只在一处调」，T3、T5 两格今天各有会红用例与变异行；探针没建 |
| P1 C934 | 没打中（第二次抽样） | 换角度：journal 记录写失败而部分落盘、冻结之后进程崩溃不重发、清扫删行、管理员回退；都推不出误红 |
| P2 C937 | 没打中（第二次抽样） | 换角度：跨版本的记录、改记取头读的是此刻镜像、(U) 取大；没找到整张镜像无红的形 |
| P3 RCW 拿掉 F_生效 例外 | 没打中（第二次抽样） | 换角度：抬 F 那一串崩在系统配置写完之后，下一次挂载写行那条根带的 F 取到 F_生效，上限不再落在两者之间 |
| P4 Q9 三处认法 | 没打中（读完 I-9.16 ④、I-3.20） | I-3.20 认「I-9.16 ④ 判它」与 I-9.16 ④ 的触发条件一致；I-9.16 提前出口两处各自已红或另有红；I-3.1 与 I-3.18 读分配记录的两条路有一处差（根记录直接持有的那一棵），推不出「两边都不红」 |

## U1　逐盘核挪到预演之前：没打中

新次序（`crates/singlefs-core/src/mount.rs:7113`：「let devices_rewritten_by_the_acquisition =」先算，`crates/singlefs-core/src/mount.rs:7133`：「refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(」后调）。

攻了的三问：

1. 逐盘核读盘会不会改预演看到的盘面。逐盘核整段包在直读那一路里（`crates/singlefs-core/src/mount.rs:7375`：「} = with_reads_straight_to_the_disk(read_route, || {」，`crates/singlefs-core/src/mount.rs:7722`：「let route_before = read_route.replace(ReadRouteOfThisMount::StraightToTheDisk);」）。直读那一路在共用的那一份读者里直接交给底下那块盘、不记字节（`crates/singlefs-core/src/mounted_session.rs:1015`：「ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),」），读坏那一判在直读那一路什么都不做（同文件 `note_the_read_failing_self_verification_and_forget_its_bytes` 的 `StraightToTheDisk => {}` 那一支）。所以逐盘核读到的坏字节、读错既不进共用的那一份，也不进读错账、读坏账；预演每一趟开头又清两本账（`crates/singlefs-core/src/mount.rs:5674`：「reads_failed_on_the_disk.borrow_mut().clear();」）。逐盘核与真做取号之间只有读（预演写只进内存），盘上字节不变。读序数在带故障层里整体后移，只影响「第几次读带故障」这类用例的取样，不改结局。推的。
2. 两拒同时成立报哪一个。今天报逐盘核那一拒（它先跑、`?` 先交回），原来报预演那一拒（C841 或预演读坏）。D22 已定项 8 那一句（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`：「要这几样都成立，有一样不成立就拒」）只要求闸不过就拒，没写与续做预演谁先；C874 一行（`.claude/kb/checks-owed.md:540`）要的是「补写闸哪一件没过在运行时有可观测的输出」，新次序下这一输出更早、更少被遮。两拒都在取号之前、三份写账全空。没找到约束先后的条款。
3. 钉读序数的用例没改。`crates/singlefs-harness/tests/common_reads_shared_by_the_rehearsal_and_the_real_run/mod.rs:72`：「pub const READ_OF_SLOT_ZERO_OF_DEVICE_ZERO_INTO_THE_SHARED_READS: u64 = 307;」那一对钉的是判号那一读（`next_instance_generation_to_acquire`），在 `establish_instance` 开头、逐盘核与预演之前，次序改不动它。按读序数注入故障的另几条用例（`writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs`、`writable_mount_reread_after_a_bad_read_reaches_the_disk_past_the_reads_shared_with_the_rehearsal.rs`）不写死数，先无故障挂一遍、照读账现算窗口，三处取法的文档已按新次序改（`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:434`：「真做取号写之前那几次读（系统配置槽、根环；取号之前的逐盘核排在续做预演之前），所以它是只进内存那一趟读的」）。不是没跑到新次序那一支：每次可写挂载都过逐盘核，是取法本身随次序走。推的（没跑）。

会红的用例形态（补钉用，不是打中）：同一份窄盘镜像盘 1 两槽写 0、退役树那片号改 u64::MAX（C841），断言今天报 C841 还是逐盘核那一拒——两种都合条款，钉哪一个要主 agent 先定先后。

推翻条件：找到一条读在直读那一路里仍写进共用的那一份或两本账（例如某个读者绕过 `read_by_the_route` 直接调了记账的那一层）。

## U2　改法二：打中（推的，没造盘面）

修补员那句「这一形预演只走一趟、读坏账没被看」只在预演走到续做结局时成立。走不到结局时，补写盘的读坏账照样被看。

代码路径（每一步指到许可它的那一句）：

1. 逐盘核交回非空的补写盘表 [D]，计划里带它（`crates/singlefs-core/src/mount.rs:7127`：「devices_rewritten_by_the_acquisition: devices_rewritten_by_the_acquisition.clone(),」）。
2. 预演一趟：分配器拷贝上那一趟缺空间，转去只进内存那一趟，取号走共用的那一份（`crates/singlefs-core/src/mount.rs:8046`：「ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal,」）。
3. 取号先逐盘读两槽，再判补写盘例外（`crates/singlefs-core/src/transaction.rs:933`：「system_configuration_slots_of_one_device_read_once(」在前，`crates/singlefs-core/src/transaction.rs:957`：「if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {」在后）。D 两槽读得出、自证不过，读的一方报读坏，共用的那一份记进读坏账、丢掉那两块（`crates/singlefs-core/src/mounted_session.rs:1177`：「self.reads_failing_self_verification.borrow_mut().push(」）。D 在表里，取号照常往下走——这是改法一修好的那一格。
4. 只进内存那一趟停在续做结局之外：续做里一步报 C841 之外的错（`crates/singlefs-core/src/mount.rs:8066`：「ContinuationRehearsedBeforeAcquisition::StepRefused(failed.cause)」），或取号之后那一串、续做里推的抬 F 报错（`crates/singlefs-core/src/mount.rs:8049`：「return ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation;」）。
5. 这两种都落进「看两本账」那一支（`crates/singlefs-core/src/mount.rs:5693`：「| ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation => {}」），读坏账里至少有 D 的两条，`(None, None, _)` 那一支（`crates/singlefs-core/src/mount.rs:5708`：「(None, None, _) => return Ok(()),」）走不到；没到上限就重走。
6. 重走那一趟 D 两块已被丢掉，再读穿到盘上，盘上还是废的，再记两条；到上限拒（`crates/singlefs-core/src/mount.rs:5714`：「(None, Some(read_failing_self_verification), true) => {」），cause 是 `ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition`，点名的是 D 的系统配置槽 0（读坏账里最先那一条，假定 D 之前没有别的读坏）。

同一份盘面去掉 D（D 两槽好的）时：第 5 步两本账都空，交回 `Ok(())` 照旧留给续做，真做取号、写行、暖机落盘，续做走到同一步报它自己的错（`MountError::Publish`，带已落盘那几次的写账）。

后果（推的）：

- 报错归错了因：拒因是「预演读坏」，点名一块补写闸已判过、取号本来就要补写的盘的系统配置槽；续做那一步真正的错（例如 `DeletedInodesQueueRefused`、`NextCheckpointTxgPastTheTopOfItsRange`，`crates/singlefs-core/src/mount.rs:5822`：「| PublishError::DeletedInodesQueueRefused(_)」那一张「不在取号之前拒」的表里的任何一项）被遮住。
- 「两本账都空照旧留给续做」那一支（预演文档自己写的契约）在有补写盘的池上整个失效：补写盘那两条读坏是逐盘核已知的、取号会补写的，不是「真做再读可能走出别的结局」那一类，拿它重走与拒都没有依据。
- 盘上逐字节不变、D 一直补写不了；每次挂载同一拒。没有补写盘时这份池至少写得进行（续做之前那几次落盘）。两种结局哪个对条款没写，归错因那一格是确定的。

盘面要什么（造法）：续做确定地停在 C841 之外的错上，而只进内存那一趟这一刻读盘没有别的读错、读坏。没有故障的合法池上我没找到这种停法（续做的错清单里能确定触发的都要校验和自洽的坏镜像，与 C841 那份用例的造法同一类）。用例形态：

- 起点 `common_reads_shared_by_the_rehearsal_and_the_real_run::narrow_image`（续做要推抬 F，取号之前走只进内存那一趟），按 `change_the_unit_and_propagate_its_checksum_on` 改一处让续做某一批报非 C841 的错（例如把队首 inode 在分配记录树里某个数据单元的记录标成已释放，释放那一批报 `ReleaseTargetAlreadyReleased`；落点与字段偏移要现查，我没造），再把盘 1 两个系统配置槽整槽写 0。
- 断言：挂载交回的错不是 `ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition`、cause 是那一步的错（或主 agent 定的别的结局）；对照格盘 1 不写 0，断言同一份坏镜像报那一步的错。今天推的会红在第一格「预演读坏、点名盘 1 槽 0」。

四句：

- 分不分辨臂：分辨改法一与「改法一加改法二」。只做改法一在这一格照中；改法二（补写盘两槽的读坏不记账，或判两本账时剔掉补写盘系统配置槽那几条）让第 5 步两本账回到空、交回 `Ok(())`。
- 系统当时看不看得到判别它的东西：看得到。补写盘表在预演之前已由逐盘核交回，读坏账每一条带盘号与偏移，D 的两个系统配置槽偏移是固定的。
- 满足判据字面的哪一个分句：预演文档「两本账都空的照旧留给续做」与 C874 一行「补写闸哪一件没过在运行时有可观测的输出」的反面——这里闸都过了，挂载仍被拒，拒因指向补写盘，运行时看起来像补写盘有读坏。
- 跑前条款给的改法在这一格还中不中：第一轮判决给的两改法里，改法一在这一格还中（打中的就是改法一之后的样子）；改法二修这一格。

改法（只在我的读码里推过、被攻过零轮，没实现、没跑）：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 改法二甲：取号读见证时，补写盘表里的盘两槽自证不过不报读坏（`system_configuration_slots_of_one_device_read_once` 带上补写盘表） | 第 3 步不记账，第 5 步两本账回空 | 推的。副作用：真做那一趟本来走直读、不记账，不受影响；要核同一读者别的调用方（mkfs 流、用例）交空表照旧报 |
| 改法二乙：判两本账之前剔掉补写盘系统配置槽那几条（只在 `refuse_before_acquisition_…` 里过滤） | 同上 | 推的。比甲窄：读坏照记、共用的那一份照丢，只是不拿它们判重走 |

推翻条件：找到 D 两槽读坏不进读坏账的一条路（例如只进内存那一趟的取号读走直读）；或证明续做在只进内存那一趟里除 C841 之外不会停（那样这一形只在不可达盘面上）。

## U3　T2 断言的放处：没打中

- 新放处判的是 `raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk` 整个交回值（`crates/singlefs-core/src/mount.rs:4660`：「let raised_to_the_ceiling =」，`crates/singlefs-core/src/mount.rs:4672`：「Err(MountError::Recovery(RecoveryFailure::NoValidRoot))」）。这个交回值里算上限有两次：先算一次（`crates/singlefs-core/src/mount.rs:4507`：「let ceiling = rollback_floor_ceiling_of_this_process(」），高于 F 就调 `raise_rollback_floor`（`crates/singlefs-core/src/mount.rs:4520`：「raise_rollback_floor(」），后者准入入口再算一次（`crates/singlefs-core/src/mount.rs:4902`：「let ceiling = rollback_floor_ceiling_of_this_process(」）。两次的 `NoValidRoot` 都经 `?` 原样上交，都被新断言罩住，与挪之前（断言在共享的私有函数里、两次都判）罩的范围相同。
- 范围有没有变宽：抬 F 那一串别处会不会交 `Recovery(NoValidRoot)`、被新断言误当成「前提一破了」。`grep -n NoValidRoot crates/singlefs-core/src/*.rs` 产品代码里造这个值的只有算上限的共享函数（`mount.rs` 第 3978 行那一处 `.ok_or`）、读阶段择根（第 8455 行）、只读挂载与恢复报告；抬 F 那一串（系统配置核对、读实例表、算 F 生效值、影子账、回收、空发布）不调读阶段与恢复报告。没变宽。推的。
- 本进程准入路径上还有没有一处 panic 而不交错：`push_one_floor_raise_within_the_admission_budget` 有三个调用处——续做被空间拒（`crates/singlefs-core/src/mount.rs:7605`：「match push_one_floor_raise_within_the_admission_budget(」）、写行之后准入不够（`push_floor_raises_after_the_row_publish` 里，第 8145 行）、会话发布被拒（`crates/singlefs-core/src/mounted_session.rs:4533`）。前两处在取号之前只进内存那一趟里也跑（分配器是拷贝、盘是只进内存的盘）。新文档只列了两种（`crates/singlefs-core/src/mount.rs:4623`：「这是本进程准入路径（可写挂载写行之后推、挂着的会话发布被拒时推，分配器都是这个进程挂载建的那一份）」），漏了预演那一种。预演那一趟前提一照样成立：写行那条根在只进内存那一趟里经拷贝上的分配器记根，读它的槽取内存里写下的字节，总读得出；读坏不会落在它上面。推的。只是文档少一种，不是打中。
- 另一条断言（前提二，F ≤ txg）照旧在共享的私有函数里，公开入口也会 panic；判决 T2 只点名了前提一那一错要交回，前提二那一条照 core 判决 R2 留断言，没越位。
- `raise_rollback_floor` 一并交回：它是 pub、文档写「只供测试强制进入的那一档」，又是产品路径里抬到上限的那一调。断言从共享私有函数里拿掉之后，它的直接调用方（用例、checker 档运行模式与实验装置，`grep -rln 'raise_rollback_floor(' crates` 列出 20 份）拿到的是错而不是 panic；这与判决 T2「共享函数保留这个错，公开入口交回它、不 panic」同一个理由（调用方给的分配器）。算不上越出 T2：不这样改就得在 `raise_rollback_floor` 里另放一份断言，而它同样收调用方给的分配器。

会红的用例形态（补钉用）：给 `raise_rollback_floor` 一个不带根环表的分配器（照 `admission_floor_ceiling_entry_returns_no_valid_root_for_an_allocator_without_the_root_ring_table.rs` 的造法）、根环全读坏，断言交回 `Recovery(NoValidRoot)`；今天推的是绿，拿掉交回、改回断言的变异要红。

推翻条件：抬 F 那一串里找到一处别的函数交 `MountError::Recovery(RecoveryFailure::NoValidRoot)`（单故障下走得到）。

## U4　C928 解冻用例：没打中（丢 discard 那一问）；另记一处与冻结无关的扣下即丢

- 三个调用点（`crates/singlefs-core/src/mounted_session.rs` 第 2136、2307、2498 行：用户改动发成之后、整理显式入口、后台入口）都调同一个 `discard_the_segments_reusable_as_a_whole`，冻结判在函数第一道（`crates/singlefs-core/src/mounted_session.rs:2931`：「if let Some(frozen) = allocator.frozen_publish() {」），判在取表之前（`crates/singlefs-core/src/mounted_session.rs:2936`：「let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();」）。新用例走的是显式入口；另两处与它共用同一判、同一份表，用例钉的是函数的行为，调用点换哪个不改结论。重发之后最先会走到的是哪一处，看调用方先做什么（改动还是整理），核心层没有重发的会话入口，用例取哪一处都行。推的。
- 冻结期间再冻结：发布路径第一道判冻结（`crates/singlefs-core/src/transaction.rs` 的 `refuse_while_a_publish_is_frozen`），冻结着时没有第二次发布能进落盘那一段，造不出「冻结上再冻结」。
- 重发失败后回卷：重发先把冻结的那一次取出（`crates/singlefs-core/src/transaction.rs:1598`：「let Some(frozen) = allocator.take_frozen_publish() else {」），落盘失败原样放回（`crates/singlefs-core/src/transaction.rs:1604`：「allocator.freeze_publish(frozen);」），分配器本身（退回的那一份，连同它的待 discard 表）一个字段都没动；冻结那次成立之后的那一份（`allocator_after_the_publish`）随冻结对象一起放回，它的待 discard 表也不动。下一次重发成功照样整份换上（`crates/singlefs-core/src/transaction.rs:1612`：「*allocator = allocator_after_the_publish;」）。不丢。
- 冻结期间对退回那一份分配器的改动在重发成功时整份丢掉（换成冻结那一刻存下的那一份）：会改它的只有整理入口撤活意图的 R 排他（第一轮攻方 T3 已记，别的会话的代码）与读坏重读记下的根环槽；待 discard 表冻结期间没人取，丢掉的那一份里待 discard 表与存下的那一份只差冻结那次自己回收与占用的段，不丢已积下的。推的。
- 另记（不是 C928，没被看过的地方）：判孤儿头时扣下的段（`withheld_holding_unswept_orphan_heads`，`crates/singlefs-core/src/mounted_session.rs:2971`：「.withheld_holding_unswept_orphan_heads」）与读不出扣下的段，已经从待 discard 表里取走、只记进这一次交回的账，不放回表里；清扫抹掉孤儿头之后、或下一次读得出时，这几段这一进程不再发 discard。discard 只是提示，后果是空间回收少发（SSD 与精简盘），不是正确性。条款（主 agent 2026-10-15 定、零轮的「装着孤儿头的段不发」）没写扣下之后发不发。推的，没量。

会红的用例形态（补钉用）：照新用例的历史，第 8 步换成「原样重发、第一次重发注入写失败、再重发成功」，第 9 步断言同一组段都发了 discard；今天推的绿。扣下即丢那一形：造一段装着未清扫孤儿头的整段可再分配段，整理一次（扣下）、清扫一次、再整理，断言第二次整理发了它；今天推的红（如果条款定成要发）。

推翻条件：找到第四个发 discard 的调用点，或冻结期间有路径调 `take_the_segments_to_discard_on_every_device`。

## U5　C929 回卷之后过不过补写闸：推荐候选一（照今天不判）

回卷之后那块盘 D 盘上是什么样（读码）：取号写两轮，第一轮写每块盘较旧那一槽、第二轮写另一槽；第二轮报错时对写出过的每块盘只回卷一次，回卷写覆写较旧那一槽、另一槽留着新号（`crates/singlefs-core/src/transaction.rs:709`：「回卷写覆写这块盘此刻较旧的那一槽，」，`roll_back_acquisition` 的文档，函数在第 713 行）。D 原来两槽全废：第一轮写进一槽新号（带所选那一版的系统配置），回卷写进另一槽旧号。回卷之后 D 两槽都自证得过，一槽旧号、一槽新号——与同一次取号里一起回卷的别的盘逐槽同一种样子。取号写与回卷写只写系统配置槽；D 的根环、journal 环、单元区在两次挂载之间一个字节没动。崩在两轮之间（不回卷）时 D 一槽新号、一槽仍废，与「每盘一槽系统配置坏」这一承诺内的单故障形同一种样子。

两个候选各在哪些盘面上错：

| 候选 | 错在哪 | 依据 |
|---|---|---|
| 候选一：照今天不判（D 走落后支或当普通成员盘） | 只在两次挂载之间 D 被池外改写（克隆盘、别的进程写了 D 的环或单元区）时，闸三（环里没有更新的根或记录）、闸四（单元区没有写序实例更大的单元）该拒而不拒 | 闸三四五读的东西在上一次挂载取号之前判过且都过了，之后本池只写了 D 的系统配置槽；池外改写这一暴露面任何成员盘都有，闸只给「两槽全废」这一种盘开，不是给「曾经两槽全废」的盘开 |
| 候选二：回卷后的下一次挂载仍判三四五 | 读的一侧做不出：下一次挂载看到的 D 与同次回卷的别的盘逐槽同一种样子，也与「一槽撕裂、另一槽好」的普通盘同一种样子，没有可判的东西告诉它 D 曾经是补写盘（判别子观测不到） | 要做只能在写的一侧：回卷时把 D 恢复成两槽都不自证（两槽都要写，同段两写之间没有屏障就是第二轮判决 T1 那一形，C463 一族），或在系统配置里加一个持久标记（改盘上格式） |

推荐候选一：在单故障承诺里它不错，错的那一形（池外改写）是全部成员盘共有的暴露，不该只罩回卷过的那一块；候选二的代价是改回卷写法或改格式，换来的是对池外改写的部分防护。推的，没造盘面。

条款可以写的一句（给写回参考）：回卷之后（或取号两轮之间崩溃之后）那块盘有自证过的系统配置槽，下一次挂载按普通成员盘判，不再过补写闸；补写闸判过的环与单元区在取号写与回卷写之间不被本池改写。

推翻条件：找到回卷或两轮之间崩溃之后 D 的系统配置槽能与普通成员盘分开的一处（某个字段只在补写写里带）；或找到取号写与回卷写之间本池会写 D 的环或单元区的一条路。

## U6　第一轮的方法：没打中（挑 T3，读码复核；探针没建）

挑 T3 的理由：T1、T5、T6 今天各有用例或已在第一轮之后补了钉（T5 的 365 / 366 边界在 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 的 `the_product_threshold_sweeps_on_the_mount_whose_rows_pass_three_hundred_sixty_five_and_on_no_mount_before`；T1 的钉值 / 钉槽由 core 判决那一批的用例钉），T6 只有 T2 那一处、已改。T3 第一轮「相符」靠的是一句读码：「core 的产品代码里 `discard_at` 只在 `discard_the_segments_reusable_as_a_whole` 里调」。这一句一旦读漏（别处还有一个不判冻结的 discard），整格就翻。

现查（命令与原样输出，这一次跑的）：

```
$ grep -n 'discard_at(' crates/singlefs-core/src/*.rs | head
crates/singlefs-core/src/block_device.rs:382:    fn discard_at(
crates/singlefs-core/src/block_device.rs:935:    fn discard_at(
crates/singlefs-core/src/block_device.rs:1217:    fn discard_at(
crates/singlefs-core/src/block_device.rs:2449:            .discard_at(DeviceOffsetInBytes(8192), 16384)
crates/singlefs-core/src/block_device.rs:2493:            .discard_at(DeviceOffsetInBytes(8192), 16384)
crates/singlefs-core/src/block_device.rs:2519:        let outcome = device.discard_at(DeviceOffsetInBytes(8192), 16384);
crates/singlefs-core/src/block_device.rs:2540:            .discard_at(DeviceOffsetInBytes(8192), 16384)
crates/singlefs-core/src/block_device.rs:2556:            device.discard_at(DeviceOffsetInBytes(100), 512),
crates/singlefs-core/src/block_device.rs:2560:            device.discard_at(DeviceOffsetInBytes(DISCARD_IMAGE_BYTES - 512), 1024),
crates/singlefs-core/src/block_device.rs:2578:            .discard_at(DeviceOffsetInBytes(8192), 0)
$ grep -n '#\[cfg(test)\]' crates/singlefs-core/src/block_device.rs | head -3
1488:#[cfg(test)]
```

`head` 截在第 10 行，`mounted_session.rs` 那一处（第 2988 行，第一轮攻方 T3 引过）在截掉的后面；block_device.rs 里第 382、935、1217 行是 trait 声明与两份实现，2449 起都在第 1488 行的测试模块里。另跑 `find crates/singlefs-core/src -mindepth 2 -name '*.rs' | wc -l` 得 `0`（src 下没有子目录里的 `.rs`），`grep -rn 'discard_at(' crates/singlefs-core/src | grep -v 'block_device.rs'` 只得一行 `crates/singlefs-core/src/mounted_session.rs:2988:            match device.discard_at(`。核过的范围是 core 这一个包的 src。结论：读码那一句在这个范围里成立，T3 不翻。

T3 今天的会红用例与变异行（现查 `grep -n 'C928' crates/mutations.tsv`）：第 6145 行「discard 开头不判冻结」、第 6249 行「重发成功之后清空待 discard 表」两行（各自由哪条用例杀，我没核；推的是冻结着整理一段不发、重发之后整理发出积下的那几段那两条）。这两行的证红没跑过（我不跑变异，交主 agent，命令见末节）。

探针没建：原本想补一条「冻结着整理一个写都不发（不只是不发 discard）」的会红用例核第一轮 T3 的另一句读码（「冻结期间整理那一趟的别的写都在发布路径第一道被拒」），现成的历史（`common::build_pool`）建在盘上的镜像文件上，换成内存稀疏盘要重搭冻结那一段，挂钟里排不下。形态见末节。

## P1　C934 认 u 落了盘的前提：没打中（第二次抽样，换角度）

第一轮攻的是根槽写失败即冻结、零单元发布失败即实例结束、FUA 被丢。这一轮换四个角度，都对着「u 与抬 F 的实例 r 同一实例，或 u 那个实例的行 T > u」推「u 的根槽落了盘」：

- journal 记录写报错而记录其实落了：落盘次序是单元 → 屏障 → 记录 → 根槽 FUA → 轮换。记录那一写报错就冻结在根槽之前；进程若之后断电不重发，下一次挂载可以由这条记录重建出 u 那一版并择它，T = u，「u < T」不成立，认不出（保守）。重发成功则根槽写过。推不出误红。
- 冻结之后进程断电、没重发：冻结那次的根若已落（轮换报错那一形），下一次挂载要么择到它（T = 它的 txg），要么按「系统配置没见证到」判它被抛弃；两种下 T 都不小于 u 之前那几版的 txg，那几版各自的发布已返回成功、根槽写成过。
- 实例表清扫删了 u 那个实例的行：删行之后那个实例没有行，第一轮报告记的「那个实例没有行报认不出」照保守走，不误红。
- 管理员回退（向前发布）：被回退掉的那几版所在实例的行照旧在，回退那一刻之后的根是新的一版，u 若是被回退掉的那一段里的一版，它的发布当时已返回、根槽写成过。

没造盘面。唯一能让「u 的根槽没落而被认成落了」的仍是第一轮那一形（设备报成功而没持久），超出单故障。

## P2　C937 取宽条件：没打中（第二次抽样，换角度）

第一轮攻的是单张镜像上「记录比单元宽 / 窄」配「少记一槽」。这一轮换三处：

- 跨版本：I-3.18 判全部候选版本的未释放记录（`crates/singlefs-checker/src/walk.rs:19256` 起），I-3.1 的取宽只看最新根这一盘上的记录（`records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes` 的调用处在第 21322 行）。取宽要那条记录在最新根里跨度与头不符；同一 (盘, 槽) 的头每一版读的是同一份字节，最新根里不符，I-3.18 在最新根那一版上就判它红（只要那一版的分配记录读得出，见 P4 第三条）。没找到取宽认了而 I-3.18 没判红的记录。
- 改记取的是此刻镜像上的头：未释放记录罩的槽写者不会再分配（释放之后要过 F 才回收），后来崩掉的实例写的孤儿头落不到这些槽上；头读不出时两边都跳过这条记录。
- (U) 取两份 |E(t)| 的大者再与 |Ω ∪ U| 取小：|Ω ∪ U| 不随改记变（Ω 按遍历引用的跨度加），取宽只放宽 (U) 的前一半；配不上的那一块要全落在 I-3.18 判红的单元里才放过，与 Q9 前半句一致。

没找到取宽之后整张镜像一处都不红的形。

## P3　RCW 拿掉 F_生效 例外：没打中（第二次抽样，换角度）

第一轮推出误红要「带新 F 的根与 txg = 新 F 那一条都读不出」两处故障。这一轮换一个单故障的来路：抬 F 那一串先写完每块盘的系统配置（新 F = F1）、第一条带 F1 的空发布落盘之前崩溃。下一次可写挂载写行那条根带的 F 取盘上根与系统配置里的大者（`raise_the_floor_through` 与写者给新根取 F 的那一句），写行之后现行根的 F 就是 F1；准入算出的上限要是不高于 F1，交 `FloorAlreadyAtTheCeiling`、不推，不会出现「上限落在根上 F 与 F_生效 之间、往下抬被拒」。挂载之前的镜像（崩溃态本身）就是第一轮那一形：最新可读根带旧 F、F_生效 = F1、最新根 txg ≥ F1 不一定成立——崩溃态里最新根是抬 F 之前那一版，txg 不低于 F1（F1 不超过当时的上限，上限不超过环里根的 txg），第一轮那一步「最新根 txg ≥ F_生效」照旧成立。推的，没造盘面。

## P4　Q9 三处认法：没打中（I-9.16 ④、I-3.20 两处读完）

- I-3.20 (b) 认「缺 `deleted_inodes`」：`crates/singlefs-checker/src/walk.rs:22097`：「.any(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)」，树表非空而没有种类 9 时交回说明、报部分没判，说明里点名 I-9.16 ④。I-9.16 ④ 在 `judge_tree_table_entry_kind_counts` 里判十二种各至少一条，种类 9 在那十二种里（`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES` 取发号次序那一份），缺种类 9 必红。两边条件一致。
- I-9.16 有两个提前出口不判 ④：① 盘上次序不升序就判红返回（`crates/singlefs-checker/src/walk.rs:5852`：「if let Some((index, pair)) = first_pair_out_of_the_on_disk_order {」）——这时 I-9.16 已红，Q9 后半句不触发；条目宽不到种类那 2 字节就报部分没判、不判 ④——这时 I-3.20 那一边读树表先过宽度判（`crates/singlefs-checker/src/walk.rs:18824`：「return Err(VersionReadingGap::TreeTableEntryNarrowerThanTheFieldTable);」），交回的说明是读不全、不点名 I-9.16 ④，而树表条目那一支走读断了，那一版 I-7.4、I-4.8 红（`crates/singlefs-checker/src/walk.rs:5867`：「树表条目那一支走读断了，那一版的 I-7.4、I-4.8 红」）。两出口都不造出「I-3.20 说 I-9.16 ④ 判它、而 ④ 没判也没别处红」。
- I-9.16 只在一张树表第一次被走到时判（`crates/singlefs-checker/src/walk.rs:2740`：「judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);」，同一落点后来的版本不重判，第 2753 行按落点记下）。同一落点同一张镜像上字节相同，最新根的树表与先走到它的那一版是同一张表。不漏。
- 第一轮留的那一处（I-3.1 与 I-3.18 读分配记录两条路）读到底：I-3.1 走 `allocation_records_of_a_root_record_without_judging`，只读树表里种类 3 那几条指的树、任一棵读不出整条放弃（`crates/singlefs-checker/src/walk.rs:10167`：「AllocationRecordTreeThatDoesNotReadBack::GiveUpOnTheRoot,」）；I-3.18 走 `allocation_records_of_a_version_without_judging`，另读根记录直接持有的那一棵（`crates/singlefs-checker/src/walk.rs:18859`：「parse_node_pointer(version.allocation_record_tree_root_held_outside_the_tree_table);」）。差在这一棵：最新根树表非空、根记录里那根指针又非零且读不出时，I-3.18 那一版整版读不全、报部分没判，I-3.1 照读得出、可能取宽并说「I-3.18 判它」。这一形要根记录在树表非空时还带一根非零的直接持有指针（写者只在树表 0 条那一版写它），属于校验和自洽的坏根记录；那根指针读不出由走读那一侧判红（I-2.1 / I-7.4 一类），整张镜像不判成好的。推的，没造盘面；严格说 Q9 要的是「那块畸形（跨度不符）别的不变量判红」，这一形里跨度不符那一块没有不变量判红，镜像是因为另一块畸形红的——交主 agent 定这算不算 Q9 的漏。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| U1 | 逐盘核的读坏、读错进不进共用的那一份与两本账；两拒同时成立报哪个；钉读序数的用例 | 读码：直读那一路、`read_by_the_route`、读坏记账那一处；三份按读序数注入故障的用例的取窗法 |
| U3 | 新断言罩的两次算上限；抬 F 那一串别处的 `NoValidRoot`；三个调用处（含预演）的前提一 | 读码：core 产品代码里全部 `NoValidRoot` 造处，`push_one_floor_raise_within_the_admission_budget` 全部调用处 |
| U4 | 三个调用点；冻结上再冻结；重发失败再冻结；冻结期间对退回那一份分配器的改动 | 读码：`discard_the_segments_reusable_as_a_whole`、`resend_the_frozen_publish`、`persist_the_publish_or_freeze_it` |
| U5 | 回卷之后、两轮之间崩溃之后 D 的两槽样子；两次挂载之间本池写不写 D 的环与单元区 | 读码：`write_the_witnessed_acquisition`、`roll_back_acquisition` |
| U6 | T3「`discard_at` 只一处调」 | grep core 整个 src |
| P1 | 记录写报错而落盘、冻结后断电、清扫删行、管理员回退 | 读码与手推 |
| P2 | 跨版本、头取此刻镜像、(U) 取大 | 读码：I-3.1 取宽回调与 I-3.18 |
| P3 | 抬 F 系统配置写完、空发布之前崩溃，下一次挂载 | 手推 |
| P4 | I-9.16 两个提前出口、按落点只判一次、I-3.20 宽度判、I-3.1 与 I-3.18 两条读法 | 读码：`judge_tree_table_entries_ordering`、`judge_tree_table_entry_kind_counts`、`committed_reservation_basis_of_a_version_without_judging`、两条分配记录读法 |

## 这条腿自己的限度

- 十格全是读码与手推，没编没跑、没造盘面：U2 的打中是推的，要一份能让续做确定地停在 C841 之外的校验和自洽坏镜像才能做成会红用例，我没造（字段偏移没查）。照规则这一格是线索，由主 agent 现查。
- 没打中的九格都是一次抽样；P1–P3 是第二次抽样，但两次都是读码，没有一次是造盘面跑出来的。
- U2 只推了「续做停在非 C841 错上」这一个入口；只进内存那一趟在取号之后那一串（写行、暖机、写行之后推的抬 F）里因补写盘本身停下的形，我只看了取号读见证那一段（新用例做成说明那一段走得过），别的段没逐行核。
- U5 的推荐只在单故障承诺里成立；池外改写（克隆盘）那一形没造。
- 用户动作：U2、U4 的历史里由用户决定的只有「挂载」「整理还是改动」，这两格的结论与它们无关（推的）；没把挂载之间的用户写放开扫。
- 开跑前看负载（`ps -o pid,args -u "$(id -u)"`）：别的会话的广谱变异覆盖档在跑（harness 与 checker 档两条 `cargo test` 在内存包装里），没有性能测量。我一条重活都没起，没用第二台。

## 没做什么

- 没建模型目录、没写 `tools/rerun.sh`、没建仓副本与编译目录（草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-code-r2-opus` 里只有派发闸建的写者锁）。
- 没跑重型测试、没起 checker 档的 cargo、没碰 GPU、没跑门禁阶段与变异。
- 没判正推、辩方那几格；改法二甲、乙与 U4 的扣下即丢都是零轮，没量代价。

## 要主 agent 跑的命令

可选，核 U6 里 T3 两行变异的证红（轻型，交主 agent，按 `.claude/rules/verification.md`「两档：轻型与重型」起）：

```
SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=crates 变异 C928 两行; 范围=2 行变异; 为什么=核 b19 代码轮第二轮攻方 U6 引的两行 C928 变异证红' bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick /home/fy5090/code/singlefs --item <C928 两行的项名>
```

工作目录仓根；项名照 `crates/mutations.tsv` 第 6145、6249 行的名字填（我没查 `--list-items` 的写法）；线程照登记默认值；看末尾每行变异被杀没有。

要把 U2 做成入库的会红用例：照 U2 一节「用例形态」写进 `crates/singlefs-harness/tests/`，在主工作区或集成副本里照 harness 档跑这一个测试目标，今天推的红在「预演读坏、点名盘 1 槽 0」。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-code-r2-opus-output.md` 末行（交回前这一次跑的）：

```
  ✓ 核了 41 处引文、14 处裸行号，对上 41 处，没判 14 处（其中裸行号没有引文可比 14 处；逐处列在上面）
```
