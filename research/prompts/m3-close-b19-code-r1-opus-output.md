# m3-close-b19-code-r1 攻方腿（opus）报告（2026-10-29）

正文 `research/prompts/_m3-close-b19-code-r1-body.md`；开工快照 `research/prompts/m3-close-b19-code-r1-snapshot/` 两份 sha256 清单开工时对主工作区逐份核过、全部 OK，下文引的代码行号取主工作区（与快照同 sha256）。

## 复跑命令与文件 sha256

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-code-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <snapshot-tree.py 倒推出的树> --work <新的草稿目录>
```

它把冻结副本里的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 拷进草稿目录，放进探针，先按快照那一份代码跑、再打上改法一跑，各与 `out/` 里那一份逐行比，打 `name=probe_output_identical lines=5` 与 `name=probe_output_with_the_reorder_identical lines=5`。交回前在第二台跑过一次（本机内存 slice 排满、排不上）：`bash scripts/hosts/multi-host-run.sh --peer --memory-cap 4G --label m3-close-b19-code-r1-opus-rerun -- bash research/prompts/m3-close-b19-code-r1-opus-model/tools/rerun.sh --repo . --frozen . --work rerun-work`，退 0，两行 `name=` 都打出、diff 无输出（`--frozen .` 是第二台那一份主工作区拷贝，不是倒推的树）。

```
24881305e3a8ce08ff09145874cbe5ab5ac384be5fd5a1dea04c4728fbbc0f51  research/prompts/m3-close-b19-code-r1-opus-model/attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration.rs
5e358e3a69a1ab2486952b5c677f419d9bf4497119c1c40a435d5953bdb88101  research/prompts/m3-close-b19-code-r1-opus-model/fix-reorder-the-device-check-before-the-rehearsal.diff
ae9f5a9fbdc4c4068c934c5d79ff87367f970bf1b54aa8fcefc09294d3660274  research/prompts/m3-close-b19-code-r1-opus-model/out/probe-output.txt
c3061d75458538992f67922b5d1034de7eddd6515b72e1e4d2323e042b1017a7  research/prompts/m3-close-b19-code-r1-opus-model/out/probe-output-with-the-reorder.txt
bb11138e14fe590986ff15756ce357978a1879f488e82304ea3e9f0a5d37000e  research/prompts/m3-close-b19-code-r1-opus-model/tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 C773 写者（钉值, 槽） | 没打中 | 四处写根取法、平局、读坏 / 全零 / 读错三形、年轻环、checker 与 GPU 还原都对得上；一条可能的变异（只比钉值）等价 |
| T2 C724 两处 NoValidRoot | 没打中 | 第二处结构上走不到；第一处两条前提在单故障下成立；另记一处越出判决：共享函数的 `NoValidRoot` 在本进程路径上改成 panic，判决只要求断言前提 b |
| T3 C928 冻结着不发 discard | 没打中 | discard 只有一处、第一道判冻结；冻结期间整理的别的写都在发布路径第一道被拒；重发之后待 discard 表随分配器换回、不丢 |
| T4 C874 补写闸原因与补写盘 | **打中一处**（副本量过） | 续做要推抬 F 的池上，逐盘核排在续做预演之后、预演按空的补写盘表走：两槽全废的盘在预演取号时被记成读坏，重走到上限在取号之前拒成「预演读坏」，补写闸与 C874 的载荷都走不到；同一条池历史上续做走完再挂就补写成了。改法一（逐盘核挪到预演之前）在副本上让这两格做成；只做它不够的那一半是推的 |
| T5 C690 阈值 365 | 没打中 | 第 366–369 行没有「写不进也不清扫」的形：预留按 ⌈(rows0 + 3) ÷ 369⌉ 算；文档「链在一片之内收回」一句只在每次写一行、清扫做成时成立 |
| T6 其余 mount.rs 改动 | 没打中 | 逐处压着 C773 / C724 / C874；没压着条款的只有 T2 那一处 panic |
| P1 C934 u 落了盘 | 没打中 | 写失败即冻结、零单元发布失败即实例结束、T 由择根定，同一实例或 u < T 时 u 的根槽一定写成过；唯一反例形要 FUA 被丢，且只让 checker 少红 |
| P2 C937 取宽两个方向 | 没打中 | 取宽只在 I-3.18 已红时生效；造出的「I-3.1 部分没判而少记一槽没有别处红」那一形与单缺陷镜像逐字节相同，判别子观测不到；只放一个方向会与 Q9 前半句冲突 |
| P3 RCW 拿掉 F_生效 例外 | 没打中 | θ_high 只升不降、U 按 t 取，三判不因 F_生效 升高误红；误红要两处读不出 |
| P4 Q9 三处认法 | 没打中（一处残余没核） | I-3.1 认 I-3.18 与 I-3.18 自己调同一个函数；两边读分配记录的路径不同，「两边都不红」那一形没核到底；I-9.16 ④、I-3.20 两处没读 |

## T1　C773 写者带（钉值, 槽）到写根：没打中

攻了的路径：

- 写根四处取 t、p：带单元的发布与写行那次经 `PoolAllocator::record_root_written_by_this_process` 取 `lowest_pin`（`crates/singlefs-core/src/allocator.rs:2793` `.min_by_key(|pin| (pin.pinned_txg, pin.ring_slot));`），编码 `crates/singlefs-core/src/allocator.rs:1813` `ROOT_RING_REGIONS * self.ring_slot.slot + self.ring_slot.region`；零单元发布照抄上一版（`crates/singlefs-core/src/transaction.rs:1725` `pinned_ring_slot: previous_root.pinned_ring_slot,`），它只走树表 0 条的一版，池级 checker 在那一版报不适用，不判 p；mkfs 写 255。
- 平局：钉值相同只可能是三个槽 0（钉值 0；别的槽钉值就是编码 3·槽 + 区域 ≥ 3，互不相同）。`RootRingSlot` 按（区域, 槽）比，三个槽 0 里取区域 0；按编码比也是区域 0。两种破法在所有能出现的平局上取同一槽，推的。
- 读坏、读回全零两形都经 `txgs_ring_slots_bad_at_mount_can_hold` 再经 `crates/singlefs-core/src/mount.rs:2993` `fn oldest_txg_a_ring_slot_bad_at_mount_can_have_held(`，槽 0 取第 0 代根的 txg（`crates/singlefs-core/src/mount.rs:3000` `crate::make_filesystem::GENESIS_CHECKPOINT_TXG`）。读故障（设备报读错）与读坏同一类。
- 年轻环：first_txg ≤ 2 时区域 1、2 的槽 0 公式落不到、不钉。那时区域 0 槽 0 的第 0 代根要么读得出（环里最旧有效根就是 0、门槛 0），要么读坏（钉 0），门槛都是 0，不钉区域 1、2 的槽 0 不放松门槛。推的。
- checker 与 GPU 两边还原：`crates/singlefs-checker/src/walk.rs:9757` `if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION {` 与 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5282` `let pinned_txg = u64_of(select(pinned, 0u, pinned / g.regions == 0u));`，与写者「槽 0 钉 0、别的槽钉值 = 编码」同一个函数关系。

会红的用例形态（补钉用，不是打中）：判决 R1 正推腿那一条变异「把钉值 0 直接当 p 写出」要 first_txg ≥ 3S 且区域 1 或 2 的槽 0 读坏的池才现形；另一条值得加的变异是把 `min_by_key` 的键换成只比钉值（`.min_by_key(|pin| pin.pinned_txg)`）：三个槽 0 同时钉着时 `min_by_key` 取第一个最小值，迭代次序是 BTreeMap 的（区域, 槽）序，结果不变——这条变异等价，不用写进变异表。推的，没跑。

## T2　C724 两处 NoValidRoot：没打中

- 第二处：`crates/singlefs-core/src/mount.rs:4082` `.or(oldest_valid_root)?;` 只在 `oldest_valid_root` 为 `None` 时交 `None`，它取自 `valid` 的最小值；`valid` 为空时 `newest_per_device` 也为空，`crates/singlefs-core/src/mount.rs:3978` `.ok_or(RecoveryFailure::NoValidRoot)?;` 先交回。`expect` 的理由句成立，与盘面、故障无关。
- 第一处在本进程准入路径上：前提 b 由 `crates/singlefs-core/src/mount.rs:4412` `current_root.rollback_floor <= current_root.checkpoint_txg,` 断言。攻了的历史：抬 F 那一串先写系统配置再发空发布、第一次空发布被冻结（F_生效 高于现行根带的 F）——`raise_the_floor_through` 第一道判冻结（`crates/singlefs-core/src/mount.rs:4843` `if let Some(frozen) = allocator.frozen_publish() {`），一串都不动；卸载那一串把 F 抬到卸载开始时现行那一版的 txg，之后写的根 txg 更大；管理员回退是向前发布。这几条都造不出现行根带的 F 高于它自己的 txg。
- 前提 a：现行根的槽在「知道住着根的槽」里，读坏重读仍坏先拒；读得出时 txg ≥ F（前提 b）且按现行实例表不被抛弃（本实例在自己那一张表里没有行）。冻结期间分配器退回发布之前那一份：冻结那次的根若已落盘，盖的是另一槽，现行根那一槽不动，照样读得出。没造出 `valid` 为空的盘面。
- 一处越出判决的行为变化（不算打中，记给主 agent）：判决 R2 只要求给前提 b 补断言；实现另把共享函数交回的 `Recovery(NoValidRoot)` 也改成断言失败（`crates/singlefs-core/src/mount.rs:4430` `Err(MountError::Recovery(RecoveryFailure::NoValidRoot))` 那一处 `assert!`）。原来是挂载错误交给调用方，现在是 panic。前提 a 在单故障下成立（上一条），这一改只在前提 a 破了时生效；它把「写明走不到」从注释变成运行时 panic，判决没写这一样，零轮。

## T3　C928 冻结着时整理不发 discard：没打中

- 发 discard 的只有 `discard_the_segments_reusable_as_a_whole` 一处（core 的产品代码里 `discard_at` 只在 `crates/singlefs-core/src/mounted_session.rs:2988` 它里面调，`block_device.rs` 另几处在测试模块里；它的调用点三处 `crates/singlefs-core/src/mounted_session.rs:2136`、`:2307`、`:2498`），第一道 `crates/singlefs-core/src/mounted_session.rs:2931` `if let Some(frozen) = allocator.frozen_publish() {` 判冻结、整份待 discard 表不取。用户改动那一处在发布报错时经 `?` 先交回，不走到 discard。
- 冻结期间整理那一趟的别的写：搬一批的单元写在 `persist_publish_writes` 里，发布路径第一道已拒；删除续做走发布路径，同拒；抬 F 第一道拒。没找到别的往单元区写、或把段交出去的路径。
- 冻结解除：`crates/singlefs-core/src/transaction.rs:1594` `pub fn resend_the_frozen_publish<Device: BlockDevice>(` 成功后 `crates/singlefs-core/src/transaction.rs:1612` `*allocator = allocator_after_the_publish;`，待 discard 表随这一份换回发布之后的样子（冻结那次落点所在段已因有槽被分配摘掉或留着）；重发之后不当场发 discard，留到下一次发布或整理那一处的 discard 发。不丢（表在内存里一直带着）；卸载前还没等到下一次 discard 的那几段这一进程不再发，这与没冻结时卸载前积下的同一个待遇，不是 C928 新添的。核心层里没有调 `resend_the_frozen_publish` 的会话入口，调用方拆开会话自己重发（`crates/singlefs-core/src/mounted_session.rs:4952` 那一行文档）。
- 顺带看到的（不归本轮射程，`compaction_stays_stopped_pausing_the_live_intent` 是别的会话的改动）：冻结期间整理入口照旧调它，它在退回的那一份分配器上撤活意图的 R 排他；重发成功分配器整份换回发布之后那一份，撤掉的 R 排他回来，而整理仍停着，到下一次整理入口再撤之前 R 压着分配。推的，没量。

## T4　C874 补写闸的原因与补写过的盘：打中（一处）、其余没打中

### 打中：续做要推抬 F 的池上，两槽全废的盘走不到补写闸，挂载按「预演读坏」拒

历史（每一步指到许可它的那一句）：

1. 池里一块盘两个系统配置槽都读得出、都自证不过（整槽写 0；丙' 补写例外就是给这一形开的，`.claude/kb/checks-owed.md:540` C874 一行的造法同此）。
2. 可写挂载走到 `establish_instance`，取号之前的计划里补写盘表先置空：`crates/singlefs-core/src/mount.rs:7107` `devices_rewritten_by_the_acquisition: Vec::new(),`，字段注释 `crates/singlefs-core/src/mount.rs:7693` `/// 取号之前的逐盘核在预演之后才判，预演那一趟按空表走。`
3. 续做预演先于逐盘核：`crates/singlefs-core/src/mount.rs:7113` 调 `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(`，逐盘核在其后 `crates/singlefs-core/src/mount.rs:7125` `publishes_planned.devices_rewritten_by_the_acquisition =`。
4. 分配器拷贝上那一趟续做有一步空间不够（`crates/singlefs-core/src/mount.rs:5752` `Err(refusal) if refusal_is_short_of_space(&refusal) => {`；那一趟扣住位没放开、比真做紧，文档自己写了紧到就转去只进内存那一趟），转去只进内存的盘上走取号连同续做，取号读走共用的那一份（`crates/singlefs-core/src/mount.rs:8037` `ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal,`），补写盘表是空的。
5. 取号读见证：两槽全废的盘读一次两槽，自证不过的报给读者（`crates/singlefs-core/src/recovery.rs:4930` `) => reader.note_that_the_bytes_read_fail_self_verification(`），只进内存的盘交给底下共用的那一份，路由是 `crates/singlefs-core/src/mount.rs:7011` 的 `SharedBetweenTheRehearsalAndTheRealRun`，记进读坏账（`crates/singlefs-core/src/mounted_session.rs:1177` `self.reads_failing_self_verification.borrow_mut().push(`）；空表里没有这块盘，取号交 `DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`（`crates/singlefs-core/src/transaction.rs:957` `if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {` 那一判不过）。
6. 取号失败，那一趟交 `StoppedBeforeTheEndOfTheContinuation`（`crates/singlefs-core/src/mount.rs:8040`）。读坏账不空、没到上限，从头重走；重走那一趟同样读两槽（读坏的块已从共用的那一份丢掉，再读穿到盘上，盘上还是 0），同样记账、同样停。
7. 到上限：`crates/singlefs-core/src/mount.rs:5705` `(None, Some(read_failing_self_verification), true) => {`，在取号之前拒成 `MountError::Publish`，cause 是 `ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition`，点的是那块盘的系统配置槽。逐盘核与补写闸一次都没走到。

量的（第二台上的副本：主工作区那棵树经 `multi-host-run.sh --peer` 拷过去、只加探针这一个测试文件；不是入库装置上的数），`research/prompts/m3-close-b19-code-r1-opus-model/out/probe-output.txt` 原样：

```
PROBE unchanged_image mounted instance=InstanceGeneration(2) devices_rewritten_by_the_acquisition=[] continuation_steps=16
PROBE zeroed_image_space_admission_judged refused Publish cause=ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(ReadFailingSelfVerification { device: DeviceIdentity(1), offset: DeviceOffsetInBytes(0), length_in_bytes: 4096 }) persisted=0 failed=0
PROBE zeroed_image_space_admission_judged devices_unchanged=true
PROBE zeroed_image_space_admission_skipped refused Publish cause=ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(ReadFailingSelfVerification { device: DeviceIdentity(1), offset: DeviceOffsetInBytes(0), length_in_bytes: 4096 }) persisted=0 failed=0
PROBE zeroed_after_the_continuation mounted instance=InstanceGeneration(3) devices_rewritten_by_the_acquisition=[DeviceIdentity(1)] continuation_steps=0
```

读法：没改坏的窄盘镜像挂载做成、续做 16 步；盘 1 两槽写 0 之后同一份镜像在取号之前拒成预演读坏、点的是盘 1 系统配置槽 0，两块盘逐字节不变；关掉空间准入照样拒（只进内存那一趟是分配器拷贝上真缺槽带出来的，不是准入式子）；对照格先挂一次让续做走到空、再把盘 1 两槽写 0 挂第二次，不走只进内存那一趟，补写盘 1、挂载做成——同一条池的历史上，两槽全废的盘补不补得成只看这次挂载续做要不要推抬 F。

后果：这块盘五件闸都过得了也补写不了（对照格补写成了，推的：闸看的单元、环与单元区在两次挂载之间只多了续做那 16 步），盘上逐字节不变，下一次挂载走同一条路、同一拒——这块盘的补写例外在「续做要推抬 F」的池上整个失效；闸有一件不过时，报的也不是 C874 要的载荷（`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition { not_rewritten_because }`），运行时看不出是补写那一支没走成。今天的用例 `crates/singlefs-harness/tests/writable_mount_rehearsal_drops_bytes_failing_self_verification_from_the_shared_reads.rs` 只造了盘 0 两槽「读回坏字节一次、盘上是好的」那一形，判别力注释里写着坏字节留着时「到上限交读坏那一拒」，两槽在盘上真坏的这一形没有用例。

四句：

- 分不分辨臂：不分辨 R4 的臂（载荷用哪种封闭枚举、报第一件第一块盘）。病根在各臂共用的前提「逐盘核在续做预演之后、预演按空的补写盘表走」，与载荷形态无关。
- 系统当时看不看得到判别它的东西：看得到。逐盘核只读盘（`with_reads_straight_to_the_disk`），预演之前就能跑；那一刻两槽全废、五件闸过不过都判得出。
- 满足的是哪一个分句：C874 一行（`.claude/kb/checks-owed.md:540`）「补写闸哪一件没过在运行时有可观测的输出」那一句与判决 R4「补写闸没过的原因进挂载错误成员的载荷」——这一形里补写那一支连同闸都没走到，拒因是预演的读坏；另外丙' 例外本身（闸都过就补写）在这一形上不成立。
- 跑前条款给的改法在这几格上还中不中：R4 没给备选改法；C874 一行列的观察者、补写流崩溃枚举、登记完整性用例都录真实补写流，碰不到这一支（这一支在补写之前就拒）。

自己提的改法（只在我的模型上量过、被攻过零轮）：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 逐盘核挪到续做预演之前，补写盘表填好再交预演（`research/prompts/m3-close-b19-code-r1-opus-model/fix-reorder-the-device-check-before-the-rehearsal.diff`） | 第 5 步取号不再交「一份自证过的都没有」 | 量过（副本，`out/probe-output-with-the-reorder.txt`：写 0 的那两格都做成、补写盘 1，续做 16 步与 15 步） |
| 补写盘表里的盘两槽读坏不记进读坏账（或预演只看读坏账里补写盘之外的块） | 挪了次序之后，预演续做停在别的非 C841 错上时，补写盘两槽那两条读坏账仍会让第 6、7 步重走到上限、拒成读坏 | 推的 |

只做第一条不够：`self_verified_system_configurations_of_every_device` 对补写盘照样先读两槽、报读坏，再交回空的一项（`crates/singlefs-core/src/transaction.rs:957` 那一判在读之后），读坏账照样记两条；只要那一趟没走到 `WalkedToItsEnd`，重走到上限就拒。

会红的用例形态：窄盘镜像（`common_reads_shared_by_the_rehearsal_and_the_real_run::narrow_image`，续做中途推抬 F、取号之前走只进内存那一趟）上把盘 1 两个系统配置槽整槽写 0，可写挂载：断言挂载做成、`MountOutput::devices_rewritten_by_the_acquisition == [盘 1]`；今天推的会红在「取号之前拒、读坏」。对照：同一份镜像关掉空间准入（续做不缺空间、不转只进内存那一趟）应当做成并补写盘 1。探针源码 `research/prompts/m3-close-b19-code-r1-opus-model/attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration.rs`。

改法一的副作用（推的）：逐盘核读盘走 `with_reads_straight_to_the_disk`，挪到预演之前会改动整池读序数，`common_reads_shared_by_the_rehearsal_and_the_real_run` 里钉死的 `READ_OF_SLOT_ZERO_OF_DEVICE_ZERO_INTO_THE_SHARED_READS`（307）这类常量与按读序数注入故障的几条用例要跟着重核。

### T4 其余几问：没打中

- 枚举覆盖：`devices_without_the_selected_version` 对两槽全废的盘只有两处 `return Err`：闸前读错（`SystemConfigurationSlotReportsAReadError`）与补写闸第一件不过（`RewriteGateNotMet(_)`）；补写闸函数五个 `return Some(..)` 各对一个成员，`rings_of_the_device_against_the_selected_version` 先交读错、再交更新。成员一一对上。上一节那一形是这个函数根本没被调到，不是枚举漏成员。
- 「第一件、第一块盘」确定性：外层按 `reader.device_identities()`（交进来的盘表次序）、遇第一块就整个返回；闸内次序固定（闸一、闸五、闸三、闸四、闸二），闸五与闸三在同一遍读里先读完全部槽、再按「读错先于更新」交，不受槽的读序影响。同一盘表次序、同一盘面上确定。盘表次序换了，报的盘可以换，文档写的就是「盘表次序第一块」。多盘多件同时不过时只报一件，C874 文字接受。
- 报的补写盘与真写的盘：取号写两轮对盘表里每块盘都写（`write_the_witnessed_acquisition` 不分补写不补写），任一写或屏障报错整次挂载失败、不交 `MountOutput`；交得出 `MountOutput` 时补写盘表里每块盘都写过两轮。集合相同。
- C929（回卷之后那块盘该不该再过闸）：今天的代码在回卷之后的下一次挂载里，那块盘两槽里至少一份自证过（回卷每盘写一次旧号），`devices_without_the_selected_version` 走落后支（`what_a_device_behind_the_version_lacks`），闸三、闸四、闸五不再看。取号写与回卷写只写系统配置槽，那块盘的根环、journal 环、单元区在两次挂载之间不变，上一次挂载那五件闸都过了，下一次落后支只缺闸三四五的再核；两次挂载之间盘被别处写过（克隆盘那一形）才分得出。条款该怎么定交主 agent；推的，没造盘面。

## T5　C690 清扫阈值 365：没打中（产品代码），注释一句外推

- 触发：`crates/singlefs-core/src/instance_table_sweep.rs:807` `if rows <= configuration.row_count_threshold.0 {`，行数取挂载末尾现行那一版的实例表，366 起扫。
- 切换预留不按 365 写死：`crates/singlefs-core/src/admission.rs:566` 那一行 `.div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)` 按 ⌈(rows0 + 3) ÷ 369⌉ 片算。rows0 = 366 时正好一片；行数越过 369 时预留跟着要两片，准入照式子判（不够就推抬 F 或拒），不会出现「行写不进而准入没留」。
- 能让一次挂载开头行数已越过 365、或一次挂载写多行的几形（都只让「链在一片之内」那句不成立，不让行写不进）：写行那次给 [上一个实例, 要取的号) 里每个实例一行（`crates/singlefs-core/src/mount.rs:5913` `fn instance_rows_to_write(`），中间取号跳过的实例各多一行；上一次挂载写行之后、末尾清扫之前崩溃，下一次挂载开头就是 366；清扫这一轮一行不删的几种（`crates/singlefs-core/src/instance_table_sweep.rs:277` 根环一槽读不出或全零、`:297` 现行那一版树表 0 条、盘不全、读失败）持续成立时行数一直涨。
- 外推的一句：`InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE` 的文档「链在一片之内（一片装 369 行…）收回，切换预留不按两片留」只在「每次挂载写一行、末尾清扫做成」时成立；上面几形里链会长出第二片，预留由式子按两片要。这是文档的射程，不是代码错；建议文档补「清扫一行不删或一次写多行时链照样越过一片，预留按式子算」。

## T6　其余 mount.rs 改动：没打中（一处行为变化越出判决，见 T2）

逐处压着的条款：

| 函数 | 改动 | 压着的条款 |
|---|---|---|
| `txgs_ring_slots_bad_at_mount_can_hold` 与新函数 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held` | 槽 0 的最旧 txg 取第 0 代根的 0；读坏与读回全零两类都经它 | C773 判决 R1 乙 |
| `ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes` | diff 里只是上一个函数的文档行做上下文，函数体没动 | 无变化 |
| `ring_slots_recorded_as_holding_an_abandoned_root_by` 之后的 `rollback_floor_ceiling_of_this_process` | 参数改成整条现行根；两处断言 | C724 判决 R2（第二个断言越出判决，T2） |
| `establish_instance` | 把补写盘表带进 `MountOutput` | C874 判决 R4 |
| `devices_without_the_selected_version`、`RewriteGateNotMet` 改 pub、`DeviceNotRewrittenByTheAcquisition` | 拒因载荷 | C874 判决 R4 |

没压着条款的行为变化：除 T2 那一处 `NoValidRoot` 改 panic，没找到。`oldest_txg_a_ring_slot_bad_at_mount_can_have_held` 也改了读回全零那一类的钉值（全零槽同样经它），C773 一行（`.claude/kb/checks-owed.md:481`）的标题就是「读坏与全零根槽」，在射程里。

## P1　C934 认 u 落了盘的前提：没打中（单故障）

被攻的那一句：`crates/singlefs-checker/src/walk.rs:9033` `let its_root_landed = version.instance == raising_instance` 与随后 `rows_of_its_instance` 全部 `overwritten_txg < row.published_checkpoint_txg` 那一判。

攻了的路径（对着写者）：

- 根槽写失败：带分配器的三条发布路径落盘中途失败就冻结（`persist_the_publish_or_freeze_it`），之后同一实例每一次发布在第一道被拒，只有原样重发清得掉，重发要再写一遍根槽并成功。零单元发布不冻结，但它一失败整个挂载返回错误、实例就此结束。抬 F 那一串第一道判冻结。所以同一实例里 u 之后还有一版（抬 F 那条根，或 T > u），u 的根槽 FUA 一定返回过成功。
- 恢复把 T 推过 u：T 是下一次挂载给那个实例择到的根的 txg；择到的可以是崩在记录之后、根槽之前、由记录重建的那一版（D23 已定项 15），那一版自己的根槽没落盘，但它就是 T，u < T 不含它；比它早的各版各自的发布在它开始之前已返回。实例行 (i, 0, 0)（取号跳过的实例）T = 0，任何 u 都不满足 u < T，报认不出。
- 唯一造得出的反例形要一种「设备报成功而没持久」的故障（FUA 被丢）：u 的根槽没落、那一槽还住着 u − 3S，之后 u + 3S（txg 不小于抬 F 那条根）落了盘、盖掉它。checker 认作抬 F 那一刻住着 u，写者抬 F 时读盘读到的是 u − 3S。把一槽的住户从旧根换成新根，上限式子里每块盘最新、第 4 新的不同状态、最旧有效根都只升不降，checker 算出的上限不低于写者的，只会少红、不会误红（推的）。这一形超出「写失败」单故障，C773 射程里已有「两样故障下钉槽那一条会误红」同类的记账。
- 钉槽 p 认不认得出这一形：p 记的是写者记根那一刻还钉着的、挂载时读坏的槽，与「本进程写的根槽没落盘」无关，认不出，也用不着。

没造盘面：上面每一条都推不出误红。

## P2　C937 I-3.1 照 I-3.18 改记取宽：没打中（打中的那一形判别子观测不到）

被攻的：`crates/singlefs-checker/src/walk.rs:10391` `fn allocated_statistic_lies_within_the_bounds_of_either_span_reading(`，只在 `records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes` 交回 `Some`（至少一条未释放记录被 I-3.18 判红）时才走。所以取宽那一支只在整张镜像已有 I-3.18 红的时候生效，镜像判不成好的。

造的形（记 O 为别的记录罩住、都被遍历走到的槽数）：一条两槽单元的记录跨度写成 3（比单元宽），另有一处「记账少记一槽」的坏写者。按记录自述 |E| = O + 3、改记之后 O + 2；记账 O + 2。自述那一份 (L) 不成立；取宽那一份 (L) 取 O + 2、(P) 单元两槽在改记那一份里、(U) 取大 O + 3 再与 |Ω ∪ U| = O + 2 取小，三判成立 ⇒ I-3.1 报部分没判，「少记一槽」这件事没有别的不变量判红。

四句：分不分辨臂——分辨「两个方向都放」与「只放比单元窄」（只放窄的那一臂在这一形上 I-3.1 照判红）；系统看不看得到判别它的东西——看不到：同一份字节也正是「写者按单元那一类记两槽、记录多写一槽、没有少记」那一形（单缺陷），两种历史落下的镜像逐字节相同，I-3.1 分不出记账少的那一槽是「少记」还是「按单元记」；满足判据字面哪一分句——Q9 条件「那块畸形若没有别的不变量判红，那一处判红」的后半句，但「那块畸形」在镜像上只认得成 I-3.18 那一条；跑前条款的改法——只放「比单元窄」一个方向在这一形上判红，可它在单缺陷那一份同样判红（I-3.18 已红、I-3.1 再红），与 Q9「别的不变量判红就报部分没判」前半句冲突。

结论：判别子观测不到，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 那一形办，不拿它判臂。两个方向都放与 Q9 前半句一致；只放窄的一个方向会在「记录比单元宽、记账按单元」那一形上与 Q9 前半句冲突。另一个方向（记录比单元窄、记账按记录少一槽）同理：I-3.18 红，(P) 只在取宽时成立，报部分没判。没找到取宽之后整张镜像无红的形（取宽那一支的前提就是 I-3.18 红）。

## P3　RCW 拿掉 F_生效 例外：没打中

F_生效 高于最新根带的 F 的合法形：抬 F 那一串先写每块盘系统配置（新 F）、第一次带新 F 的空发布落盘之前崩溃；或带新 F 的根都读不出（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 那一份坏镜像钉的形）。这时 θ_high = max(F_生效, 环里最旧有效根) ≥ F_生效 > 根上 F。

逐判：t = max(根上 F, 写者记根时的门槛)，写者门槛不高于 θ_high 的论证与没有这一例外时同一条（I-7.18 那一格），θ_high 只升不降，所以 t ≤ θ_high、E(t) ⊇ E(θ_high)。(L)：写者记账 = |E(t)|（判决 Q5 坐实的写者口径）≥ |E(θ_high)|。(P)：候选集按 F_生效 滤，只更少；候选根 R（txg ≥ F_生效）走到、在最新根账里已释放的槽，释放代 g > txg_R ≥ θ_high，在 E(θ_high) 里。(U)：U 按 t 取、不按 θ_high，E(t) 的已释放部分要么被走到（Ω）、要么在 U，未释放部分被最新根走到（最新根 txg 不低于 F_生效：抬 F 的新 F 不超过上限、上限不超过环里根的 txg）。三判都不因 F_生效 升高而误红。推的，没造盘面；造不出误红的那一步是「最新根 txg ≥ F_生效」，它要是破了（新 F 写进系统配置而最新可读根比新 F 旧），最新根不在候选集里、(U) 里未释放的那一截不在 Ω，会误红——这一形要两处读不出（带新 F 的根与 txg = 新 F 那一条都读不出），超出单故障。

## P4　Q9 条件三处的认法：没打中（一处残余没核）

- I-3.1 认「I-3.18 判红」：取宽那一支的回调（`crates/singlefs-checker/src/walk.rs:21325` 起）与 I-3.18 自己（`judge_allocation_record_spans_against_the_classes_of_the_units_they_cover`）都调 `crates/singlefs-checker/src/walk.rs:19237` `fn class_and_slots_of_the_unit_whose_usable_header_starts_at(`，跳过已释放记录、头用不了的记录两边一样，同一份判法，不是各写一份近似。
- 一处两边口径不同、没核到底：I-3.18 按版本清单逐版读分配记录（`tree_table_entries_of_a_version_without_judging` 加 `allocation_records_of_a_version_without_judging`），I-3.1 读最新根的「全部分配记录」（`allocation_records_of_the_newest_root_read_in_full_without_judging`）。若有一形前者在最新那一版读不出、报部分没判，而后者读得出、认出跨度不符的记录、报「I-3.18 判它」的部分没判，就是「两边都不红」。两份读法是不是同一棵树、同一个读不出的条件，我没读到底，推的、留给下一轮。
- I-9.16 ④ 与 I-3.20（缺 `deleted_inodes`）那两处这一轮没读到判法正文，没攻。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T1 | 带单元发布、写行、零单元发布、mkfs 四处写根；三个槽 0 同时钉着的平局；first_txg ≤ 2 的年轻环；读坏 / 读回全零 / 设备读错 | 读码，全部写根调用点（`reclaim_floor_when_the_latest_root_was_recorded()` 与 `pinned_ring_slot:` 的非常量赋值，`grep` 全 core） |
| T2 | 抬 F 系统配置写了、空发布冻结；卸载那一串；管理员回退；冻结期间算上限 | 读码，`rollback_floor_ceiling_of_this_process` 两个调用点 |
| T3 | 用户改动、整理显式入口、后台入口三处 discard；冻结期间整理的搬迁、删除续做、抬 F；重发之后的待 discard 表 | 读码，core 产品代码里全部 `discard_at` 与 `frozen_publish()` 调用点 |
| T4 | 闸前读错、五件闸各一；多盘多件同时不过；盘表次序；补写盘与取号写的盘；取号写失败回卷；续做要推抬 F 的预演（打中） | 读码加一个探针（窄盘镜像一份，写 0 盘 1 两槽，空间准入开 / 关两格，续做走完再挂一格） |
| T5 | 一次挂载写多行；写行之后清扫之前崩溃；清扫一行不删的各成员 | 读码，触发判与预留式子 |
| T6 | 五个点名函数逐处 | diff 全文对着改动后的函数体 |
| P1 | 根槽写失败冻结、零单元发布失败、恢复由记录重建、取号跳过实例的行、FUA 被丢 | 读码，写者三条发布路径与 checker 那一判 |
| P2 | 记录比单元宽 / 窄，各配「记账少记一槽」「泄漏一槽」「丢记录」 | 手算，界的取法照 `crates/singlefs-checker/src/walk.rs:10391` 那一个函数 |
| P3 | 抬 F 写了系统配置、空发布没落；带新 F 的根都读不出 | 手算，三判逐一 |
| P4 | I-3.1 认 I-3.18 的回调与 I-3.18 自己的判法 | 读码；I-9.16 ④、I-3.20 没读 |

## 这条腿自己的限度

- T4 那一形只在一份镜像（窄盘、两块盘、续做中途推抬 F）上量过，量在第二台的副本上（主工作区拷贝加探针），不是入库装置；没在别的几何上扫，也没扫「哪块盘写 0」（只写了盘 1）。改法一只在这一份镜像上量过；改法二没实现。
- 「T4 两槽全废的盘在这一份镜像上过得了五件闸」是用「续做走完再挂」那一格旁证的（推的），不是在同一份镜像上直接调闸。
- T1、T2、T3、T5、T6、P1–P4 全是读码与手算，没造盘面；「没打中」是一次抽样，照规则不当结论。
- P4 两处没读、一处读法差异没核到底。
- 用户动作：T4 探针里用户动作只有「挂载」「挂一次再挂」两种，没把挂载之间的用户写放开扫；这一形的触发只看续做要不要推抬 F，与挂载之间写什么无关（推的）。

## 资源与跑在哪

- 开跑前 `ps` 看到别的会话的 crates 变异复跑（`checker-tier-crates-mutation-replay.sh --quick`）、崩溃点重放 `--quick` 与一条 `--check feature-gated-tests` 在跑，没有性能测量。本机 `run-with-memory-cap.sh 4G` 两次排队（slice 已占 58.9–70.4 GiB、总上限 40.1 GiB，各排到 180 多秒与 728 秒没排上），包装提示第二台放得下；第二次排队那一条由我停掉（`proc.py stop` 停排队的那一个，之后核过没有残留）。
- 探针与复跑都改在第二台跑：`multi-host-run.sh --peer --memory-cap 4G`，`--send` 把副本里 `crates/singlefs-harness/tests` 整个目录（主工作区那一份加探针一个文件）送过去，改法一那一趟另送副本里打过补丁的 `crates/singlefs-core/src`；每趟第二台的树用完即删（输出里 ⑦ 那一行）。

## 没做什么

- 没跑重型测试、没起 checker 档的 cargo、没碰 GPU；没跑门禁阶段。
- 没在入库装置上量；副本上的数不进 kb，要引得由主 agent 在入库装置上重做（写一条 harness 用例）。
- 没实现改法二、没跑改法一对现有读序数用例的影响。
- 没攻 P4 的 I-9.16 ④ 与 I-3.20 两处。
- 草稿与副本：仓副本 `~/.local/share/singlefs/work/m3-close-b19-code-r1-opus/repo`（44M）已删；本机编译目录一次都没建成（两次都在排队）；第二台的树每趟已删。草稿目录里留着草稿与第二台的输出（92K）。

## 要主 agent 跑的命令

无必须的。要把 T4 那一形做成入库的会红用例时，把探针里「盘 1 两槽写 0、续做要推抬 F」那一格改成断言挂载做成且 `devices_rewritten_by_the_acquisition == [DeviceIdentity(1)]`，放进 `crates/singlefs-harness/tests/`，在主工作区或集成副本里照 harness 档跑一个测试目标（今天推的红在「取号之前拒、预演读坏」）。
