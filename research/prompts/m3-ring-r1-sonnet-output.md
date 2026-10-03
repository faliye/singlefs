# m3-ring-r1 正推腿报告（Sonnet，H1–H6）

日期 2026-09-30。批次 `records/2026-09-30-里程碑三开工第一轮.md#2`。立场：正推，从 kb 条款与冻结副本 `/tmp/claude-1000/m3-ring-r1/tree/crates/` 推每格该是什么，再与被判对象（kb 条款、事实表 `research/prompts/m3-ring-r1-facts.md`）逐格比。没读禁读清单里的任何文件。

我写的小模型都在 `research/prompts/m3-ring-r1-sonnet-model/`（五个脚本加各自的产物）；它们与 `crates/` 不共用代码，只按 `crates/` 今天的规则写死。

## 〇 核对与四条事实表之外的观测

核对命令与原样输出：

```
$ cd /tmp/claude-1000/m3-ring-r1/tree && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-ring-r1-snapshot/crates-sha256.txt 2>&1 | awk '{print $NF}' | sort | uniq -c
    205 OK
$ cd /home/fy5090/code/singlefs && sha256sum -c research/prompts/m3-ring-r1-snapshot/kb-sha256.txt 2>&1 | awk '{print $NF}' | sort | uniq -c
      7 OK
```

事实表的行号没有整表复核；本报告用到的每一处引文都在冻结副本或仓里 `grep -nF` 现取过，并过了 cite-check（各段末行见「交回闸」一节）。下面四条是事实表没写、而后面每一格都依赖的观测，都是读代码得出的（没编译、没跑）。

**观测 1：由记录施加出来的那一版，不是当初那次发布会写的根。**
恢复施加一条记录时，实例表指针、分配记录树根指针照抄所选根，卸载记号恒写 0：
`crates/singlefs-core/src/recovery.rs:3221`「instance_table: rebuilt.instance_table,」；
`crates/singlefs-core/src/recovery.rs:3225`「allocation_record_tree_root: rebuilt.allocation_record_tree_root,」；
`crates/singlefs-core/src/recovery.rs:3215`「unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,」。
这是条款定的：D23 已定项 15「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义」（`.claude/kb/decisions/23-journal的角色与格式.md:443`），括注「（实例表单元指针照所选根）」（同一行）。记录里没有这三样：`crates/singlefs-core/src/journal.rs:159-162` 只有 `new_tree_table`、`new_mapping_root`、`new_tree_identifier_watermark`、`new_rollback_floor` 四个新根字段。
而真发布写的根在三种发布上与「施加出来的」不同：
- 写行那次发布：实例表指针取新链首片，`crates/singlefs-core/src/transaction.rs:6208`「let instance_table_pointer = match &plan.instance_table {」，重写那一支 `crates/singlefs-core/src/transaction.rs:6210`「InstanceTablePlan::Rewrite(_) => {」；施加出来的指着旧表。
- 第一个文件版本那次发布：带文件的一版分配记录树根指针恒全零，`crates/singlefs-core/src/transaction.rs:6789`「allocation_record_tree_root: NodePointer::empty_root(),」；接在树表 0 条的一版（写过行、那一项非零）之后时，施加出来的仍是旧的非零指针。
- 正常卸载那一串的每条根：带卸载记号，`crates/singlefs-core/src/mount.rs:1997`「RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,」；施加出来的记号是 0。
普通覆写发布、暖机与准入抬 F 的空发布（这三种各自的根照上一版抄实例表与分配记录树根、记号 0）没有这个差别。
事实表把「能补的那一版」说成「这条根写进去就是有效根」（`research/prompts/m3-ring-r1-facts.md:97`）；对上面三种发布，写进去的字节不是那次发布的字节。

**观测 2：一次崩溃至多留下一个「记录在、根不在」的尖端版本。**
一次发布的落盘次序是单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换 → 屏障，`crates/singlefs-core/src/transaction.rs:1213`「writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {」在 `persist_the_root_then_rotate_the_system_configuration` 里，下一次发布的单元写排在它末尾那道屏障之后（`crates/singlefs-core/src/transaction.rs:1217`「writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {」）。所以一次崩溃留下的「effective 的 txg 大于 chosen 的 txg」至多差一次发布。例外只有「根槽读不出、后面的记录还接得上」那一形（一条以上的记录被施加）。

**观测 3：撕裂的那一槽，取号写本来就在重写它。**
`crates/singlefs-core/src/transaction.rs:361-366` 每块盘的新世代号 = 这块盘自证过的槽里最大世代号 + 1，写到槽 `世代号 % 2`（`crates/singlefs-core/src/transaction.rs:409`「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」）。一槽撕裂、另一槽自证过（世代 g）时，新世代 g + 1 落在撕裂的那一槽；写它的代码不需要新的。今天走不到它，是因为有两道拒挡在前面：读阶段的见证读在取号之前拒（`crates/singlefs-core/src/mount.rs:4304`「let some_system_configuration_slot_unread =」、`crates/singlefs-core/src/mount.rs:4313`「WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified」），取号自己在写之前读一遍每块盘两槽、有一槽读不出或自证不过就报错并点名那块盘（`crates/singlefs-core/src/transaction.rs:640`「fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(」，判在 `crates/singlefs-core/src/transaction.rs:656` 的 `< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`；D23 已定项 14 那一段也写了这一道：`.claude/kb/decisions/23-journal的角色与格式.md:389`「取号那一刻某块盘两槽里有一槽读不出或本池自证不过：在第一个取号写之前拒」）。

**观测 4：可写挂载今天已经拒「一块盘两槽全废」，C463 题面写的「静默成功」只剩只读与恢复路径。**
同一处见证读把「有一块盘自证过的槽少于 2」判成判不出（`crates/singlefs-core/src/mount.rs:4304` 起），逐盘核（`mount.rs:3182` 起的 `devices_without_the_selected_version`）里「一份自证过的都没有」直接交 `Err(那块盘)`（`crates/singlefs-core/src/mount.rs:3198`「let Some(newest_self_verified) = newest_self_verified else {」）。C463 那一行写的是另一个时间点的状态：「2026-09-21 起一块盘两个系统配置槽都无效时恢复改用别的盘那一份、照常挂上」（`.claude/kb/checks-owed.md:383`），对 `choose_system_configuration`（恢复与只读挂载）今天仍成立，对可写挂载已被 2026-09-28 的 C331 甲盖掉。

## 结论先行（每条的依据在各节）

1. H1 推荐挂点 P1（取号之后、写行那次发布之前，多一段 [根槽 FUA][屏障]），补且只补 effective 那一版，不带系统配置轮换；前提是补进去的字节要与那次发布当初会写的一致——今天的 `RootRecord` 施加法在写行、首个文件、卸载三种发布上做不到，要么把这三个字段加进记录的新根段（记录点名项 67 → 64），要么这三种洞不补。P0、P2、P3 各有出局或更弱的理由（H1 表、H2 表）。
2. H2 用小模型枚举补洞那一段的崩溃状态：P1 无红、补完之后不留内部洞；P2 只有「补洞按已生效算回收、补洞写与写行根同段」这一种写法红（5 个状态），改成「两条根都持久之后才记」就不红但内部洞会在一个状态上留下来。
3. H3：I-3.9 仍有对象，区间写法保留；C380 的原文要改写成「候选集里 L 与 T 之间隔着的、不在候选集的 txg」几类，不关；它的判别力自证（把区间收成单值、12 个状态转红）在 P1 落地后不再成立。
4. H4：三条候选的代价数照上一轮判决表；S-修 与 S-辨 是同一个改动（放开拒可写的两道，取号写本来就重写那一槽），不是两条候选；推荐 S-留 加调用方显式放行参数（默认行为逐字节不变，放行后取号写重写缺的那一槽，也覆盖 S-辨 覆盖不了的「一槽持续设备读错」）；S-辨 会把用户 2026-09-28 选的甲改成甲-读错，要用户重新定。D22 已定项 8 射程「每盘一槽系统配置坏在承诺里」没说承诺含不含可写，规则没说。
5. H5：I-3.8 草案的条件在逻辑上够用，但要的输入（整轮清扫）`crates/` 里没有；做不做取决于要不要造清扫器；不做的代价 10⁴ 次挂载 28 片、10⁶ 次 2711 片。推荐第三里程碑不做，条款改写成「等清扫器」并留数。
6. H6：C462 恢复这一层单列错误成员（约 30 行）；C463 只读与恢复路径报出来（新函数加 `MountedReadOnly` 一个字段，不改 44 个文件在用的 `choose_system_configuration` 返回类型）、可写路径今天已拒（三处）；自愈与 H4 的取号重写是同一个写、同一个显式放行参数，合成一条条款，放行范围先只含至少一槽自证过的盘。
7. 旁及一条不归本轮判的：C333（`.claude/kb/checks-owed.md:278`）的前提「记账却是那次发布的」与今天施加记录时分配记录树根照抄所选根（`crates/singlefs-core/src/recovery.rs:3225`）冲突，可能过期，见 H5-c，交主 agent。

## H1　补洞的形态（C566）

被判对象：`.claude/kb/milestone/03-third-txn.md:69` 二·4 那一行的「可写挂载时把洞补上，让环上没有洞」（方向）与事实表 (c) 末表的四个候选挂点（`research/prompts/m3-ring-r1-facts.md:97` 起）。

### H1-a　补哪几个洞

推出：只补这次挂载读阶段施加出来的那一版 h（`effective_root` 的 txg 大于 `chosen_root` 的 txg 时，两者在 `crates/singlefs-core/src/mount.rs:4458` 一带同时交出，观测 2 说明这一差至多一次发布）。补之前先读 slot(h)：
- 自证过且 txg < h（后洞，槽里是上一圈的根）、或自证不过（前洞，mkfs 清零过的槽；`crates/singlefs-core/src/recovery.rs:1090`「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}」）才补；
- 设备读不出的不补（`crates/singlefs-core/src/recovery.rs:1089`「RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue,」）：那一槽可能正住着 R_h 的真根，用合成根盖它是拿不完整的换对的。
更早的洞：事实表说「更早留下的洞，那一版只在当时的记录里，今天挂载手里没有它的根记录字节」（`research/prompts/m3-ring-r1-facts.md:97`）。对**根字节**成立；但产生那一版的记录还在 journal 环里（默认环 805 306 368 字节，`crates/singlefs-format/src/lib.rs:213`「pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;」，÷ 4096 = 196 608 条，环里的记录到被覆盖前一直读得到）。所以旧盘上早就存在的洞技术上补得了：逐个 txg 缺口取前一条根加那一条记录重放出来。它需要在挂载里另写一遍「找缺口、逐个重放」，而 P1 落地之后新洞不会再留下，只剩旧盘上已有的，我不推荐做（零轮）。判定：与事实表这一句**冲突（半句）**——「根记录字节」没有，「那一版」还在。

### H1-b　补进去的字节从哪来（事实表没问、决定 P1 能不能成立的一格）

写到 slot(h) 的字节必须是 h 那次发布当初会写的根，否则它成了一条「有引用错误的合法候选根」：
- 写行那次发布：施加出来的根指着旧实例表链；真根指着新链。旧链首片在 h 的账里已带已释放标志、释放代 h，而补进去的根还引用它。checker 每条候选根都数实例表指针（`crates/singlefs-checker/src/walk.rs:2808`「references.note(&instance_table);」），I-3.9 取「最后一条仍引用它的根」L = h，要求释放代 > L：`crates/singlefs-checker/src/walk.rs:3305`「let in_the_witnessed_interval = record.generation > last_referencing_txg」，h > h 不成立 ⇒ 红（推的）。
- 卸载那一串：施加出来的记号是 0、F 是记录里的（h − 1）；I-7.9 按「不带记号」那一支判，上限 min(每盘最新根, 第 4 新的非空根) 通常低于 h − 1 ⇒ 红（推的，`.claude/kb/invariants.md:60`「**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限」）。
- 第一个文件版本：分配记录树根指针照旧非零，与带文件的一版恒全零（`crates/singlefs-core/src/transaction.rs:6789`）不符。
三条路：

| 路 | 做法 | 补得了 | 代价数 |
|---|---|---|---|
| α | 直接写施加出来的 `effective_root.to_slot(...)`（`crates/singlefs-core/src/root_record.rs:76`） | 覆写、暖机、准入抬 F 的空发布 | 写行 / 首个文件 / 卸载三种洞补出错根（上面三条红）；不许 |
| β | 只在记录看得出是「安全三种」时补，其余不补 | 同 α 的三种 | 记录里认不出卸载那一串（记号不在记录里）；剩下三种洞留着，C566 在这三类上不闭 |
| γ | 记录的新根段（`crates/singlefs-core/src/journal.rs:159-162` 那四个字段）加实例表指针 86、分配记录树根指针 86、记号 1 位，每条记录带 | 全部 | 记录头 311 → 487 字节，一条记录装的点名项 67 → 64（`python3 h5_instance_table_numbers.py` 第一行：「点名项容量 现行 (4096-311)//56 = 67；新根段加 86+86+4 字节后 (4096-311-176)//56 = 64」），一次发布跨记录的起点 24 723 行 → 23 616 行；改 D23 已定项 15 与 `JOURNAL_HEADER_BYTES`（`crates/singlefs-format/src/lib.rs:200`「pub const JOURNAL_HEADER_BYTES: u64 = 311;」）、层 0 字节表重跑；D15：「第一个外部用户出现之前格式是软的」（`.claude/kb/decisions/15-格式冻结政策.md:3`），不走兼容通道 |

推荐 γ（补全部、不留三类洞）；不做格式变动就退到 β 并把 C566 记成「三类洞不闭」。判定：事实表「这条根写进去就是有效根」与 `crates/` 今天的施加法**冲突**，并排：
- 事实表：「这条根写进去就是有效根」（`research/prompts/m3-ring-r1-facts.md:97`）；
- 代码：「instance_table: rebuilt.instance_table,」（`crates/singlefs-core/src/recovery.rs:3221`），条款：「（实例表单元指针照所选根）」（`.claude/kb/decisions/23-journal的角色与格式.md:443`）。

### H1-c　挂点

对每个挂点答四件事。「写几次」指有洞的那次挂载多写几次；没洞的挂载一次都不多写。

| 挂点 | 写几次 / 段序列变化 | 分配器根环表入口 | 撞上的现行条款或代码 | 补洞写的崩溃窗口（H2 详） |
|---|---|---|---|---|
| P0 取号之前 | +1 根槽 FUA、+1 屏障，排在取号前 | 不用另开：装表时 latest 设成 h − 1，补完调 `record_root_written_by_this_process(h)` | 取号在它后面：`crates/singlefs-core/src/mount.rs:3812` 一带写着「取号是这次挂载的第一个写」，取号失败的三种拒因（号变了、号到顶、见证读读不出）都会出现在一次写之后，「盘上逐字节不变」的承诺破；补洞发生在取号那一刻见证读判过之前 | 两态：落 / 没落；没落时与今天相同 |
| P1 取号之后、写行之前 | 同上，多一段 `[root_record_fua]` 与一道屏障，排在取号那道屏障之后、写行的单元写之前；不带系统配置轮换 | 同上 | 无：取号在前（`crates/singlefs-core/src/mount.rs:3838`「let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(」），写行在后（`crates/singlefs-core/src/mount.rs:3870`「let row_publish = match &start.previous {」） | 两态；落了则补完、没落则与今天相同；补洞写之后到写行第一个单元写之前隔一道屏障 |
| P2 并进写行那次发布 | 写行那次的根槽段里多一个 `root_record_fua`，段序列不变、步骤种类不变 | 同上，但记根要等两条根都持久之后（否则见 H2 的 P2-eager） | `PublishWrites` 只装一条 `root_slot`（`crates/singlefs-core/src/transaction.rs:1027-1038` 一带），冻结重发也要带上补洞那条 | 段内两条根写任意子集，4 态；「写行根落、补洞没落」的状态里洞永久留下 |
| P3 暖机之后 | +1 根槽 FUA、+1 屏障，排在最后 | 要另开入口：补的 txg h 小于已记的最新，`record_root_written_by_this_process` 的断言（`crates/singlefs-core/src/allocator.rs:1641`「let expected_txg = CheckpointTxg(occupancy.latest_recorded_txg.0 + 1);」）不许 | 无 | 写行与暖机整个窗口里洞都在；崩在里面，下次挂载 effective = chosen，条件「effective 大于 chosen」不再成立，洞永久留下 |
| P-无 不补，只改门槛 | 0 | 0 | 门槛谓词在 `crates/singlefs-core/src/mount.rs:1158`「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」；候选集的读法在 checker `crates/singlefs-checker/src/walk.rs:5959-5977` 也要跟着改 | 后洞：改门槛就是把「洞之前的根」踢出候选集，回退候选少的比 P1 多得多；前洞：多扣来自那一版没有根，门槛没动它，不变 |

P1 的改法（对 `crates/` 的改动，行数是推的估计，没写过）：
1. `mount.rs` 新增纯函数 `root_to_patch_into_the_ring(chosen_root, effective_root, slot_reading)`，返回要补的 `RootRecord`（γ 之后从记录取三个字段）或不补的理由，约 20 行；
2. `rebuilt_allocator`（`crates/singlefs-core/src/mount.rs:1586` 起装表，`crates/singlefs-core/src/mount.rs:1596`「.checked_sub(1)」）在有补洞计划时把 `latest_txg_before_the_next_publish` 传 first_txg − 2，其余不动；
3. `establish_instance` 取号之后、`row_publish` 之前，`pool.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess { … })` 加 `CommitStep::Barrier`，然后 `allocator.record_root_written_by_this_process(h)`，约 15 行；失败走 `publish_failed_after_acquisition`；
4. `dry_run_of_the_publishes_after_acquisition`（`crates/singlefs-core/src/mount.rs:2874`，拷贝分配器 `crates/singlefs-core/src/mount.rs:2883`「let mut copy = allocator.clone();」）在预演写行之前对拷贝做同样的记根。不加的话，补洞让门槛提前抬高、真发与预演取的落点不同，`crates/singlefs-core/src/mount.rs:3975` 那句 `assert_eq!` 在有洞的挂载上 panic；
5. `MountOutput` 加 `patched_hole: Option<CheckpointTxg>`（五条硬要求第 4 条：分支要可观测）；
6. 层 0：`.claude/kb/layout/01-first-txn.md:400` 的 mkfs 行就是先例——不在 E142 流里的路径由 harness 用例钉录制流（`recorded_stream_matches_the_registered_mkfs_segment_sequence`），门禁按它比；补洞这一行同样登记 `[root_record_fua]`，用一条「预置残留记录」那类基镜像的挂载录制流钉住。D17 已定项 2 的步骤种类不增（还是六种，`root_record_fua` 与 `barrier` 都在里面），等价类数照旧；`registered_segment_sequences_match_every_recorded_path` 不动（`.claude/kb/decisions/17-实现分层与第三方管道.md:42`）。
不带系统配置轮换：补的是 h 那次发布，journal tail 没变（`crates/singlefs-core/src/transaction.rs:1217` 那一步写的 tail 就是记录末条的计数器，这一版的末条在补洞前后同一个）；加上轮换只增加一次原地覆写的三态窗口（C596 那一类），得不到东西。

### H1-d　补完之后回退候选集少了什么

后洞：slot(h) 上 h − 3S 那条根被盖掉，候选集少这一条（环里最旧的）；它正是「txg h 那次发布的根本来要盖掉的那一条」，没洞的历史里同样会丢它。前洞：槽从没写过，候选集不少任何一条。与今天的洞环比，后洞每补一个洞少 1 条回退候选、门槛抬 1 个 txg（模型里 h = 17：门槛 5 → 6，`ring_patch_crash_states_classes_back.out` 的 P1 各行 floor_next 6…9）。

### H1-e　补完之后 C566 的多扣还剩多少（E156 取样点）

先给今天的数，都取产物 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out`，S = 8、ρ = 1、实时点：
- 前洞，h_缺 = 1、2、3、4：峰值 10、18、26、34 槽——`research/results/e156-alloc-basis-counts-2026-09-26-r2.out:3935`「1:peak10:sum314:count38:positive38,2:peak18:sum420:count24:positive24,3:peak26:sum208:count8:positive8,4:peak34:sum102:count3:positive3」；
- 后洞，h_缺 = 4 起：52、66、80、…、366 槽（h_缺 = 24）——`research/results/e156-alloc-basis-counts-2026-09-26-r2.out:3936`「4:peak52:sum156:count3:positive3,5:peak66:sum198:count3:positive3」，末项「24:peak366:sum2404:count7:positive7」同行；
- 环上没有缺 txg 的样本（h_缺 = 0）：172 个，峰值 0——`research/results/e156-alloc-basis-counts-2026-09-26-r2.out:3934`「by_h_que=0:peak0:sum0:count172:positive0」。
补洞挂点（P0、P1、P3）在挂载交回之后的每个取样点上环里没有缺 txg：前洞那一槽补上 h，后洞那一槽的旧根被 h 盖掉，最旧有效根随之前移，环回到没洞历史的样子，所以两类都落进 h_缺 = 0 那一类（推的，把 172 个样本的 0 移到补洞之后的取样点上）。前洞、后洞分开的剩余：前洞 0、后洞 0。

第二条路（不与 E156 装置共用代码）：`research/prompts/m3-ring-r1-sonnet-model/delta_by_ring_model.py` 按环与树表寿命的定义重算 q1a 与 h_缺，与产物里 `name=anchor_dump` 的 264 行逐行比，不符 0 行，并把同一段历史里的洞补上：

```
$ python3 delta_by_ring_model.py research/results/e156-alloc-basis-counts-2026-09-26-r2.out   （产物 delta_by_ring_model.out，sha256 daf75354…）
S= 4 front 洞 txg=  5: 与 anchor_dump 逐行比 15 行、不符 0 行；没补：树表多扣峰值 1、h_缺峰值 1；补了：树表多扣峰值 0、h_缺峰值 0
S= 4 back  洞 txg= 16: 与 anchor_dump 逐行比 25 行、不符 0 行；没补：树表多扣峰值 12、h_缺峰值 12；补了：树表多扣峰值 0、h_缺峰值 0
S= 8 front 洞 txg=  5: 与 anchor_dump 逐行比 27 行、不符 0 行；没补：树表多扣峰值 1、h_缺峰值 1；补了：树表多扣峰值 0、h_缺峰值 0
S= 8 back  洞 txg= 28: 与 anchor_dump 逐行比 49 行、不符 0 行；没补：树表多扣峰值 24、h_缺峰值 24；补了：树表多扣峰值 0、h_缺峰值 0
S=16 front 洞 txg=  5: 与 anchor_dump 逐行比 51 行、不符 0 行；没补：树表多扣峰值 1、h_缺峰值 1；补了：树表多扣峰值 0、h_缺峰值 0
S=16 back  洞 txg= 52: 与 anchor_dump 逐行比 97 行、不符 0 行；没补：树表多扣峰值 48、h_缺峰值 48；补了：树表多扣峰值 0、h_缺峰值 0
```

判别力自证：`delta_mutant_check.py` 把门槛改成「环里最新根」，同样逐行比得到「门槛改成最新根：逐行比 264 行、不符 264 行（必须 > 0）」。「补了」那一列在模型里是把洞那次发布当成有根，没洞历史 k = 0 本来就是 0（产物 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out:516`「a12_k0_max_q1a=0 a13_max_q1b=0」），所以这一列是对一致性的复核，不是独立证据；独立的一半是「没补」那一列与产物 264 行全部相符。
数出来的数不是槽数：模型量的是树表张数；槽数只能引 E156 装置的数，我没有跑它（装置属 checker 档，按规矩不跑）。

剩下的多扣来自三类洞，补洞不碰：洞② 记录自证过、施加前验证失败，那一版根本没法补（`research/prompts/m3-ring-r1-facts.md:29` 那一行标了「推的，没造过」，这一格我也没造）；β 路线里不补的三类洞；旧盘上已有的洞。崩在挂载中间的状态里洞与今天一样在，下次挂载再补（H2）。

### H1 的推翻条件

- 补洞后的取样点上 h_缺 仍 > 0：说明补上的 txg h 的槽位与 `target_for_publish` 不一致（读 `crates/singlefs-core/src/root_ring.rs:114-115`「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」）。
- 在一个写行 / 首个文件 / 卸载类的洞上，补 α 字节后 checker 全绿：则观测 1 的三条红推错了（这一条我没造出镜像验，`crates/singlefs-checker-tier` 的层 0 用例造一次即可判）。
- 有一个 P1 的崩溃状态里 slot(h) 既不是旧根也不是新根：说明根槽写在 harness 里有第三态，与 D13 已定项 4 那句「根槽写长于一个扇区又罩住旧内容时第一版不支持第三态」（`.claude/kb/decisions/13-验证路线.md:74`）的射程不符。

## H2　崩在补洞中间（C566）

被判对象：D13 已定项 4 的枚举域，`.claude/kb/decisions/13-验证路线.md:72`「一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」」。
补洞写是一次根槽 FUA 写。根槽宽等于 physical_block_size（`.claude/kb/decisions/22-单元原子性怎么合成.md:59` 那一行），根记录 457 字节落在一个判定宽度里。D13 已定项 4：原地覆写之外的写，「其余写的撕裂态一律并进」「这次写没持久」这一态（`.claude/kb/decisions/13-验证路线.md:72`），根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（`.claude/kb/decisions/13-验证路线.md:74`「根槽写长于一个扇区又罩住旧内容时第一版不支持第三态」）。所以补洞写只有两态：落 / 没落。取号那一段（两块盘各一次系统配置槽原地覆写，带第三态）的状态与今天逐个相同，补洞写排在取号末尾那道屏障之后（`crates/singlefs-core/src/transaction.rs:868`「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」），所以补洞这一段新增的状态就是这一次写的两态。

### H2-a　小模型：全部状态、下次挂载择到什么、门槛、有没有被复用

`research/prompts/m3-ring-r1-sonnet-model/ring_patch_crash_states.py`：S = 4，每版一个单元，落点取「最低可回收槽」与「单元号最大的可回收槽」两种，按 D13 已定项 4 枚举（前面各段全持久，当前段任意子集）。判红只认一格：回退候选集里有根引用的单元槽已被别的单元盖掉（I-7.4 / I-4.8 同一格）。原样输出（`ring_patch_crash_states.out`，sha256 fa3de81d…）：

```
S=4 环长=12
== 洞位置 back: h=17 slot(h)=pos 9 洞槽原有根=txg 5
  none      段数= 9 | low: 状态= 19 红=  0 内部洞= 13 记录在根不在= 10 | high: 状态= 19 红=  0 内部洞= 13 记录在根不在= 10
  P1        段数=10 | low: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8 | high: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8
  P0        段数=10 | low: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8 | high: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8
  P2-late   段数= 9 | low: 状态= 21 红=  0 内部洞=  1 记录在根不在= 11 | high: 状态= 21 红=  0 内部洞=  1 记录在根不在= 11
  P2-eager  段数= 9 | low: 状态= 21 红=  5 内部洞=  1 记录在根不在= 11 | high: 状态= 21 红=  5 内部洞=  1 记录在根不在= 11
  P3        段数=10 | low: 状态= 21 红=  0 内部洞= 14 记录在根不在= 10 | high: 状态= 21 红=  0 内部洞= 14 记录在根不在= 10
  P1-wipe   段数=10 | low: 状态= 21 红=  0 内部洞= 13 记录在根不在= 12 | high: 状态= 21 红=  0 内部洞= 13 记录在根不在= 12
== 洞位置 front: h=5 slot(h)=pos 9 洞槽原有根=从没写过
  none      段数= 9 | low: 状态= 19 红=  0 内部洞= 13 记录在根不在= 10 | high: 状态= 19 红=  0 内部洞= 13 记录在根不在= 10
  P1        段数=10 | low: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8 | high: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8
  P0        段数=10 | low: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8 | high: 状态= 21 红=  0 内部洞=  0 记录在根不在=  8
  P2-late   段数= 9 | low: 状态= 21 红=  0 内部洞=  1 记录在根不在= 11 | high: 状态= 21 红=  0 内部洞=  1 记录在根不在= 11
  P2-eager  段数= 9 | low: 状态= 21 红=  0 内部洞=  1 记录在根不在= 11 | high: 状态= 21 红=  0 内部洞=  1 记录在根不在= 11
  P3        段数=10 | low: 状态= 21 红=  0 内部洞= 14 记录在根不在= 10 | high: 状态= 21 红=  0 内部洞= 14 记录在根不在= 10
  P1-wipe   段数=10 | low: 状态= 21 红=  0 内部洞= 13 记录在根不在= 12 | high: 状态= 21 红=  0 内部洞= 13 记录在根不在= 12
```

列意：「内部洞」= 下次挂载的环里最旧根与最新根之间有 txg 没有根；「记录在根不在」= 下次挂载会施加记录得到一个没有根的尖端版本（那是今天就有的、下次挂载能再补的洞，不是永久洞）。
按类的明细在 `ring_patch_crash_states_classes_back.out`、`ring_patch_crash_states_classes_front.out`。P1 后洞的类：

| 状态 | 下次挂载择到的根 → 施加后的版本 | 下次挂载门槛 | 候选引用已复用单元 |
|---|---|---|---|
| 补洞没落（2 个：挂载开头、补洞写没持久） | 16 → 17（记录在根不在） | 5 = h − 3S | 无：补洞之前没有任何单元写 |
| 补洞落了、写行还没有任何写持久 | 17 → 17 | 6 = h − 3S + 1 | 无：单元写排在补洞屏障之后，还没发 |
| 之后各段（写行、两次暖机）的状态 | 与没洞的挂载相同 | 7、8、9 逐次前移 | 无（红 0） |

### H2-b　每个挂点在「候选集引用已复用单元」这一判据上的结论

正文第五节的判据：任一挂点有一个崩溃状态让候选集引用已复用的单元 ⇒ 该挂点出局。按上面的输出：

| 挂点 | 红的状态（后洞 / 前洞） | 判据 | 环里留下内部洞的状态（21 个里） |
|---|---|---|---|
| P0、P1 | 0 / 0 | 不出局 | 0（补洞写落了才会有后面的写，屏障隔着） |
| P2，记根在两条根都持久之后（P2-late，也是 `record_root_written_by_this_process` 今天「落点取完之后记」的自然写法，`crates/singlefs-core/src/allocator.rs:1637`「pub fn record_root_written_by_this_process(&mut self, checkpoint_txg: CheckpointTxg) {」） | 0 / 0 | 不出局 | 1（写行的根落了、补洞写没落：洞永久留下，下次挂载 effective = chosen，条件不成立） |
| P2，补洞的记根排在写行的落点之前（P2-eager） | 5 / 0 | **出局**（这一种写法） | 1 |
| P3 | 0 / 0 | 不出局，但窗口最长 | 14 |

P2-eager 红的机理：门槛按补洞已生效算，写行的单元覆盖了 h − 3S 那条根引用的单元槽，而补洞写与写行的根同段、可以没落；那条旧根仍是候选，引用着被盖的槽。它是 D22 已定项 25 那一节 `.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 25：根环深度是块重用延迟的上界，两者不是同一个数`「上一代根能不能用取决于块重用延迟」这句话在补洞上的实例。前洞不红：slot(h) 里没有旧根可被引用。

### H2-c　必红世界（三个，取正文点名的那个在内）

1. **回收先于补洞持久**（P2-eager 的形态；任何让门槛在补洞写落盘之前抬高的写法）⇒ I-7.4：`.claude/kb/invariants.md:55`「回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头」，同一格 I-4.8。模型里是 P2-eager 后洞 5 个状态红，P1 与 P2-late 0 个（判别力：同一模型把记根顺序改成 eager 就红）。
2. **补进去的字节不是那次发布当初的字节**（观测 1 的三种洞、α 路线）⇒ 写行洞红 I-3.9（`crates/singlefs-checker/src/walk.rs:3305` 的区间判：补进去的根引用着已带释放代 h 的旧实例表链，L = h）；卸载洞红 I-7.9（不带记号那一支）。没造镜像，推的。
3. **正文点名的世界：盖掉 slot(h)、不带 txg h 的根字节**（P1-wipe：补洞写把 slot(h) 写成非根字节，分配器仍按补洞成功记根）：模型里红 0、内部洞 13 / 21（后洞与前洞同数）。I-7.4 与 I-4.8 那一格不红——旧根没了，没有候选再引用被盖的槽；这个世界只让「H 那一版永远没有根、候选集悄悄少一条」。今天判器里能推出的红只有一处且带条件：写落了、写行的根还没落的那个状态，最新有根的版本是 h − 1，它的记账是按环里仍有 h − 3S 那条根记的，被它独占的单元仍算已分配，候选集少了它 ⇒ `crates/singlefs-checker/src/walk.rs:6288`「judgements.judge("I-3.1", allocated == Some(walked), || {」红；而 C566 那一行自己写着「记账第 1 项与 I-3.1（已分配统计对得上） 在洞上取什么形态没定」（`.claude/kb/checks-owed.md:445`），所以这一红取决于条款还没定的形态。判定：`.claude/kb/milestone/03-third-txn.md:69` 出口栏的「补洞的必红世界（补洞那次发布崩在中间，判器红）」对世界 1、2 成立，对世界 3 **规则没说**（池级 checker 单镜像判不出，`.claude/kb/invariants.md:167` 写着「池级 checker 手上没有第二个输入」那一类事归记录核对器）。
   世界 3 要靠一条会红的核对：崩溃状态里，只要某次发布的记录被施加过、且之后本实例的第一条根已持久，环里 slot(h) 必须是 txg h 的根。它需要 journal 作第二输入，归记录核对器（D13 已定项 7），不是新加一条单镜像不变量——单镜像不变量「环上没有内部洞」不可行：洞② 与旧盘留下的洞都是合法的实例边界缺口（H3）。这条核对在今天的实现（表里 none）上红：19 个状态里 13 个有内部洞，正好是 C566 那一行要的「判别力自证：今天的实现必须红」（`.claude/kb/checks-owed.md:445`「判别力自证：今天的实现必须红（今天多扣随洞数增长）」）；在 P1 上绿（0 / 21），在 P1-wipe 上红（13 / 21）。

### H2 的推翻条件

- 层 0 或 harness 里造得出一个 P1 的崩溃状态：环里已有 h + 1 的根而没有 h 的根 ⇒ 补洞写与写行的屏障次序有错，模型「内部洞 0」不成立。
- P2-late 在真代码上有红状态：说明分配器记根不是「这次发布的落点全部取完、之后不再分配」（`crates/singlefs-core/src/allocator.rs:1630` 那一段文档写的次序）。
- 模型里一版一个单元的抽象漏了：一版引用多个单元、其中有单元被 h − 3S 那条根独占，也不会改变「补洞落盘之前不发任何单元写」这一点，但会改变 P2-eager 红的个数；个数不进结论。

## H3　洞补上之后的 I-3.9（C380）

被判对象：I-3.9（`.claude/kb/invariants.md:142`）「根环没有洞时这个区间只有一个值」、C380（`.claude/kb/checks-owed.md:313`）「洞很长时区间会松到判不出错写的释放代」，以及事实表 (d) 小结（`research/prompts/m3-ring-r1-facts.md:121`）。

推出：区间要保留，C380 改写、不关。
- 补洞之后，「被施加的记录做的释放写着一个根槽从没写过的 txg」这一类（I-3.9 那一行的例子，残留记录那条层 0 流的 12 个状态）消失：那条流上 P1 把 txg 5 补进 slot(5)，L = 4、T = 5，区间 (4, 5] 只有一个值；写行根（txg 6）落盘时 5 的根一定已持久（补洞写在写行第一个单元写之前隔着屏障）。
- 候选集里 L 与 T 之间仍可能隔着不在候选集的 txg，区间照样不止一个值，对象换成这几类：
  1. 被抛弃的根、低于 F 的根：`crates/singlefs-checker/src/walk.rs:5964`「let abandoned = abandoned_by_the_newest_roots_table(root);」与 `crates/singlefs-checker/src/walk.rs:5965`「let below_floor = root.checkpoint_txg < effective_rollback_floor;」把它们滤出候选集，与洞无关；
  2. 读不出的根槽（`crates/singlefs-checker/src/image.rs:613-624` 的 `valid_roots` 只收自证过的）；
  3. 洞②（记录自证过、点名单元验证失败，施加停在 h − 1，新实例仍从 max(记录) + 1 起，`crates/singlefs-core/src/mount.rs:1114`「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」）：那一版不存在，没有根可补；D23 已定项 14 第 3 条要求这个 + 1（`.claude/kb/decisions/23-journal的角色与格式.md:369`「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」），缺口留在实例边界上；
  4. P1 不补的洞：β 路线里的三类，旧盘上已有的洞，补洞写本身没落的状态之外的所有旧洞；
  5. 补洞那次写本身崩在中间：不产生新的一类（补洞写两态，没落就回到「记录在根不在」，下一次挂载再补）。
- C380 的判别力自证：「造一条环上连着几个 txg 没有根的历史，把释放代写成区间里另一个值，判据要能红——今天判不了；判别力自证：把区间收成单值，那 12 个合法状态必须由绿转红」（`.claude/kb/checks-owed.md:313` 前两栏合读）。P1 落地之后这一句里的 12 个状态在单值判法下也是绿，判别力自证要换成上面 1–4 类里造得出的镜像（例：被抛弃的根夹在 L 与 T 之间）。

判定：
- 与事实表 (d) 小结：**一致**（对象从「从没写过的 txg」换成「滤掉或读不出的根」），我补两类：洞②、P1 不补的洞。
- 与事实表 (d) ①「补不了的旧洞：只能补本次挂载施加出来的那一版」（`research/prompts/m3-ring-r1-facts.md:117`）**冲突（半句）**：旧洞的根字节没有，产生它的记录还在 journal 环里（H1-a），补得了，只是不划算；所以①在 P1 之后指的是「不补的旧洞」，不是「补不了」。
- 与 C380「洞很长时区间会松到判不出错写的释放代」：**规则没说**长到多长算「判不出」；P1 之后洞不再随挂载次数累积，缺口至多一个实例边界一个 txg（洞②），松度有上界（推的）。

H3 的推翻条件：造得出一份 P1 下的合法镜像，候选集 L 与 T 之间隔着不在上面 1–4 类里的 txg；或残留记录那条流在 P1 下仍有状态在单值判法下红。

## H4　撕裂轮换之后永远只读（C596）

被判对象：C596 那一行（`.claude/kb/checks-owed.md:471`）写着「一次可写挂载都做不成时怎么走出来，全仓没有条款」；二·4 那一行说这一格「做不做」（`.claude/kb/milestone/03-third-txn.md:73`「剩 C596（撕裂的系统配置轮换之后只剩只读挂载）、C597（实例表行只增不删，删行的条件没有条款） 做不做」）；用户 2026-09-28 选甲时知情（`.claude/kb/checks-owed.md:471`「用户 2026-09-28 选甲时知情」）。

### H4-a　今天的代码怎么把它拒成「永远」（复核事实表 (e)）

- 拒的那一判是「有一块盘自证过的槽少于 2」：`crates/singlefs-core/src/mount.rs:4304`「let some_system_configuration_slot_unread =」，成了真就是 `crates/singlefs-core/src/mount.rs:404`「| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,」，重读一次仍真就在取号之前拒（`crates/singlefs-core/src/mount.rs:4391`「fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(」）。
- 这一判读的数是 `verified_system_configuration_slots`（`crates/singlefs-core/src/recovery.rs:2866`「pub fn verified_system_configuration_slots<Reader: PoolReader + ?Sized>(」），它对每槽 `filter_map` 两次，把「设备读不出」与「读得出、自证不过」都丢掉，交回的只有自证过的那几份。所以今天的读数里两种坏槽分不开（观测，读代码得出）。
- 撕裂的那一槽取号写自己就会重写：观测 3；写它的世代号是 `crates/singlefs-core/src/transaction.rs:365`「.unwrap_or(0)」那一段算出的自证过最大世代 + 1，槽是世代号 mod 2，落回撕裂的那一槽（另一槽是 g，撕裂的是 g + 1 的位置）。
- 事实表 (e) 那一行（`research/prompts/m3-ring-r1-facts.md:131`「为什么是「永远」」）：**一致**，四处能写那一槽的调用都在拒点之后：取号 `crates/singlefs-core/src/transaction.rs:853`、发布末尾轮换 `crates/singlefs-core/src/transaction.rs:324`、抬 F 先写 `crates/singlefs-core/src/transaction.rs:574`，我另数到第四处回卷写 `crates/singlefs-core/src/transaction.rs:465`（取号失败之后的回卷，不是走出路）；只读挂载只拿 `&dyn PoolReader`（`crates/singlefs-core/src/mounted_read.rs:553`「pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {」）。

### H4-b　三条候选按今天的实现是什么（这里有一处与正文的分法不一致）

代价数都取上一轮判决与岔路单，我没有重量：`research/prompts/unreadable-at-mount-r2-main-verification.md:44-47` 的 L2 表，原样：

```
| 今天 | 240 | — | 可写 | 可写 |
| 甲 | 0 | 719 | 永远只读 | 永远只读 |
| 甲-读错 | 120（全是 `Zeros` 形） | 284（全是 `DeviceError` 形） | 可写 | 永远只读 |
| 甲-读错-每盘（攻方提，零轮） | 120（同上） | 98（全是两盘各坏一槽） | 可写 | 可写 |
```

列头是该文件第 42 行的「| 候选 | 丢写 | 多拒 | 撕裂轮换 | 一槽系统配置持续设备读错（单故障） |」。

| 候选 | 按今天代码怎么落 | 丢写 / 多拒 / 撕裂 / 单槽读错 |
|---|---|---|
| S-留 | 不动 `mount.rs:4304` 那一判；补出路。出路只有一条走得通：显式放行（下面 H4-c） | 0 / 719 / 只读，放行后可写 / 只读，放行后可写 |
| S-辨 | 就是判决表里的「甲-读错」（岔路单 `research/prompts/unreadable-at-mount-r2-forks.md:7` 的定义：「**甲-读错**：只有设备报读错的槽当判不出，自证不过（撕裂、全零、内容坏）照今天」）：把上面那一判从「自证过的少于 2」改成「有槽设备读不出」。要一个新读数把每槽分成读不出 / 自证不过 / 自证过（`verified_system_configuration_slots` 的 `filter_map` 分不开，约 15 行，推的），判据两处改（读阶段、取号） | 120 / 284 / 可写 / 永远只读 |
| S-修 | **不是独立的一支**：它说的「取号写里顺带重写撕裂那一槽」今天的取号写本来就做（观测 3，`transaction.rs:365`），缺的只是拒可写那两道（读阶段 `mount.rs:4304`、取号 `transaction.rs:656`）不再拦它。放开就是 S-辨；不放开，S-修 一行也走不到（取号写在拒点之后）。所以 S-修 的代价数与 S-辨 同一行，重写的代码是 0 行 | 同 S-辨 |

判定：与正文第三节 H4 那句「S-辨 … S-修 …」分成两条候选**冲突（分法）**——两条落在同一处代码改动上，正文并排：
- 正文：「S-辨 见证读把「一槽撕裂、另一槽是上一世代且自证过」与「读不出」分开，前者不当判不出」「S-修 可写挂载在取号写里顺带重写撕裂那一槽」（`research/prompts/_m3-ring-r1-body.md:41` 同一行）；
- 代码：取号写的世代号 =「这块盘两槽里自证过的最大世代号 + 1」（`crates/singlefs-core/src/transaction.rs:344`「两槽都读不出时从 1 起」那一行的文档）。

S-辨 与 C331 甲「丢写 0」冲不冲突：冲突。判决表里甲 的丢写是 0，甲-读错 是 120（`research/prompts/unreadable-at-mount-r2-main-verification.md:46`「| 甲-读错 | 120（全是 `Zeros` 形） | 284（全是 `DeviceError` 形） | 可写 | 永远只读 |」）。我在攻方产物里现查了这 120 格长什么样（不是重量，是读它的样例行）：

```
$ awk 'NR>=14 && NR<=38' research/prompts/unreadable-at-mount-r2-opus-model/c331-summary-with-dev.txt | wc -l
25
$ ... | grep -c 'slots=NewestOnBoth'            → 25
$ ... | grep -c 'hidden=NewestRootAndOneDataCopy' → 25
$ ... | grep -c 'read_back=Zeros'               → 25
$ grep -nF 'lost: fewest_fault_ranges=4' research/prompts/unreadable-at-mount-r2-opus-model/c331-summary-with-dev.txt
13:   lost: fewest_fault_ranges=4; cells with that many (up to 12):
58:   lost: fewest_fault_ranges=4; cells with that many (up to 12):
```

样例行（该文件第 14–38 行，攻方每候选最多列 12 个最少故障的格，我读到的 25 行）每一行都同时是「两盘最新的系统配置槽都读回全零」加「最新根与一份数据副本被藏起来」；攻方自己标的最少故障范围数是 4。也就是说 S-辨 的丢写要多个并发故障，不是撕裂那一个（推的：只看了样例行，没读攻方脚本怎么数「故障范围」）。

### H4-c　S-留 的出路：显式放行（我推荐的形态，零轮）

一句话：可写挂载加一个调用方给的放行参数，默认不放行（今天的行为逐字节不变）；放行时那一判对「读得出但自证不过、或设备读不出」的槽不判真，其余不动，取号写把缺的那一槽补成自证过的。

- 为什么走得通：写那一槽的代码不用新写（`crates/singlefs-core/src/transaction.rs:365` 那段世代号与槽位的算法，观测 3）。要松的拒有两道：读阶段那一判（`mount.rs:4304`），和取号写之前读每块盘两槽的那一道（`crates/singlefs-core/src/transaction.rs:640`「fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(」，判在 `crates/singlefs-core/src/transaction.rs:656`，它已经点名那块盘）；D23 已定项 14 把这两道都写成了条款（`.claude/kb/decisions/23-journal的角色与格式.md:389`）。逐盘核 `crates/singlefs-core/src/mount.rs:3198`「let Some(newest_self_verified) = newest_self_verified else {」只在一份自证过的都没有时出错，缺一槽的盘不受它拦。
- 要改的地方（行数是推的估计，没写过）：读数一处（分辨设备读不出 / 自证不过，同 S-辨，约 15 行）；`read_stage_settled_at_most_on_the_one_reread`（`mount.rs:4391`）到 `crates/singlefs-core/src/mount.rs:4313`「WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified」的参数传递，约 20–30 行；取号那一道（`write_acquired_instance` 里两处调 `self_verified_system_configurations_of_every_device`，`transaction.rs:834` 与 `transaction.rs:851`）按放行参数放过缺一槽的盘，约 15 行；读阶段的拒因不点名盘（转述；成员 `crates/singlefs-core/src/mount.rs:404` 那一行被引用，是个不带盘的单元成员，取号那一道的拒因带盘），管理员看不出该放行哪一块，拒因要带上盘与槽号，约 10 行；用例 4 条（默认拒不变、放行后可写、放行后撕裂那一槽被补成自证过、放行不影响两槽都好的池）。测试造撕裂盘面的现成法在 `crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs`（事实表 (e) 用例行）。
- 它放进来的丢写：放行之后与 S-辨 同一批格（上表 120，需要 ≥ 4 个故障范围，样例）；默认不放行时 0。它比 S-辨 多覆盖一格：「一槽持续设备读错」（S-辨 在判决表里仍是永远只读），因为放行后取号写会去写那一槽，写得下就好了，写不下走回卷（`transaction.rs:465`）、挂载失败、盘上回到旧代号——这一条是读代码推的，没造。
- 代价：调用方要有一个「这是管理员在知情下放行」的入口，第一版是纯用户态库（D17 已定项 5，`.claude/kb/decisions/17-实现分层与第三方管道.md`「已定项 5：第一版是纯用户态库，不挂载」），参数就是入口，命令行与工具不在这一轮。

### H4-d　与 D22 已定项 8 射程的关系（规则没说）

`.claude/kb/decisions/22-单元原子性怎么合成.md:178`「容错承诺是单故障：每盘一槽系统配置坏在承诺里」。C596 的盘面（每盘一槽坏）正好落在「承诺里」，而 S-留 今天的行为是只读挂载照常、可写挂载永远拒。条款没说承诺的是「读得回」还是「读得回且写得下」。判定：**规则没说**；这一句要用户定，否则 S-留 与承诺字面读起来冲突。

### H4 答：做不做

做 S-留 加显式放行（代价上面）；S-辨 与 S-修 是同一个改动，会把用户 2026-09-28 选的甲改成甲-读错，交用户重定，代价数照 L2 表。

推翻条件：
- 造一份「一槽撕裂、另一槽自证过」的盘面，打开放行，取号写之后那一槽仍读不出自证过：说明观测 3 的世代号推错（撕裂的槽不是 (g + 1) mod 2 的那个）；
- 同一盘面默认（不放行）下可写挂载不再拒：说明放行参数改动了默认行为。

## H5　实例表删行（C597）

被判对象：I-3.8 那一行的回收条件（`.claude/kb/invariants.md:141`，整句原文）：「行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）」；C597 那一行的空白（`.claude/kb/checks-owed.md:472`「删行条件的条款（回收条件、谁删、崩在删行那次发布中间怎么办）」）。

### H5-a　这个回收条件成不成立

先弄清行为什么存在，条件才判得了。行 (i, Ti, Wi) 有两个消费者：
1. 判根：`crates/singlefs-core/src/recovery.rs:1403`「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」——有行且 txg > Ti 才是被抛弃的根；D23 已定项 14 的候选集是它的反面（`.claude/kb/decisions/23-journal的角色与格式.md:380`「(i, T) 可选 ⟺ cur 的实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti」）。**删掉 i 的行，i 的被抛弃根就重新可选**——所以条件里要「根环里没有该实例发布的根」。
2. 判单元：扫描重建里判一个实例写的单元算不算数（E104 的谓词，`.claude/kb/experiments/104-扫描重建的现行版本判定.md:19`「谓词按单元类分（第七版 P2）：数据单元 b ≤ T_pub ∨ n ≤ W」）。删掉行，那个实例没发布的单元就没了裁判——所以条件里要「整轮清扫……没有该实例的未发布单元」。
journal 记录不是消费者：恢复只施加与所选根**同一个实例**的记录（`crates/singlefs-core/src/recovery.rs:3079`「record.instance == root.instance && (record.instance, record.checkpoint_txg) > water」），别的实例的行删不删都不改它。

判定：条件写全了，**没看出缺哪一格**（推的，逐个消费者对过，没造盘面）。两个不该删的对象也在条件里挡住：上一个实例的行在新链里才写出来，条件「根环里没有该实例发布的根」对它不成立；I-3.8 自己说行的实例代号低于挂载根实例。

### H5-b　条件今天有没有输入（这才是不成立的一格）

- 「根环里没有该实例发布的根、每个槽都读成功」：挂载手里有。读根环时每槽是自证过 / 自证不过 / 读不出三种（`crates/singlefs-core/src/recovery.rs:1090`「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}」，1089 行是读不出那一支），重建分配器时已经全读一遍（事实表 (c) 第 8 步）。
- 「整轮清扫」：没有。`grep -rlin 'sweep\|清扫' crates/singlefs-core/src crates/singlefs-checker/src` 在冻结副本上零命中（本轮实跑，输出为空）；扫描重建只出现在 checker 的走读与 harness 的一个测试里（`grep -rln 'scan_rebuild\|扫描重建' --include=*.rs` 在冻结副本上只命中 `singlefs-checker/src/walk.rs` 与 `singlefs-harness/tests/checker_known_bad_images.rs`）。I-3.8 那一行自己也写了这一点：「回收那一半没有输入，第一版行只增不删，删行的条件没人判」（`.claude/kb/invariants.md:141`）。
判定：与 I-3.8 状态列**一致**，与事实表 (f)「条款里的删行条件」**一致**。

### H5-c　删行那一写落在哪、崩在中间会怎样

- 落点：写行那次发布。今天它已经把整条链 COW 重写（`crates/singlefs-core/src/mount.rs:1014`「fn instance_table_rewrite_of_the_row_publish(」），删行就是在 `crates/singlefs-core/src/mount.rs:1019`「rows.extend_from_slice(rows_written);」之前对 `rows` 做一次保留可回收行以外的过滤，不多一次发布、不多一段写（推的）。取号之前的准入读的是同一个函数的输出（该函数文档「两处同一份输入」），行数减了预留自动减。
- 崩在中间：这次发布的窗口就是今天写行发布的窗口（单元 → 屏障 → 记录 → 屏障 → 根槽 FUA），删行不新增状态。根落盘之前，旧链在、行全在；根落盘之后新链在、行少了，而条件已保证被删的实例在根环里没有根。记录被施加而根没落的状态：施加出来的版本指着旧链（`crates/singlefs-core/src/recovery.rs:3221`「instance_table: rebuilt.instance_table,」），行是全的，是「删行没发生」，一致。
- 与 C333（`.claude/kb/checks-owed.md:278`）的关系：C333 的推理是「重建态里实例表是旧链、记账却是那次发布的（旧链的片已记成释放、新链的片记成已分配）」。今天施加记录时分配记录树根也照抄所选根、不取记录的（`crates/singlefs-core/src/recovery.rs:3225`「allocation_record_tree_root: rebuilt.allocation_record_tree_root,」，上两行注释写着重建出来的版本「仍指着上一版的分配记录树——那正是「这次写行没有成立」该有的账」）。C333 登记于 2026-09-14，分配记录树根指针是后来加进根的；它的前提今天看**冲突**（推的，没建模）：
  - C333：「记账却是那次发布的」；
  - 代码：`allocation_record_tree_root: rebuilt.allocation_record_tree_root,`。
  这一格不归我判，交主 agent 另判 C333 是否过期；H1 的 γ 路线（记录带实例表指针与分配记录树根指针）落地后，这一格连同 C333 会一起改写。
- 用例：8 条登记流都从 mkfs 起、行数至多个位数，行都在根环里有根，删行的分支一个都走不到（推的，事实表 (g) 的起点表加上 I-3.8 条件）；要一条 S = 4（环长 12）、连发多于 12 个 txg 的多实例录制流，删行才触发。

### H5-d　不做的代价（重跑）

`research/prompts/m3-ring-r1-sonnet-model/h5_instance_table_numbers.py`（式子照 `crates/singlefs-core/src/admission.rs:203`「.div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)」与 `crates/singlefs-core/src/instance_table.rs:122`「pub fn instance_table_pages_for_rows(rows: usize) -> usize {」），本轮重跑与落盘的 `.out` 逐字节相同（`cmp` 无输出）。原样输出：

```
点名项容量 现行 (4096-311)//56 = 67；新根段加 86+86+4 字节后 (4096-311-176)//56 = 64
跨记录起点：现行 67 片 = 24723 行；加宽后 64 片 = 23616 行
M=     369: 片数=    2 一片 2 槽 ⇒ 链      4 槽/每盘；每次挂载重写整条链     0.062 MiB/每盘（两盘各一份）；切换预留里链那一半     16 槽 =     0.250 MiB/每盘
M=     370: 片数=    2 一片 2 槽 ⇒ 链      4 槽/每盘；每次挂载重写整条链     0.062 MiB/每盘（两盘各一份）；切换预留里链那一半     16 槽 =     0.250 MiB/每盘
M=    1000: 片数=    3 一片 2 槽 ⇒ 链      6 槽/每盘；每次挂载重写整条链     0.094 MiB/每盘（两盘各一份）；切换预留里链那一半     24 槽 =     0.375 MiB/每盘
M=   10000: 片数=   28 一片 2 槽 ⇒ 链     56 槽/每盘；每次挂载重写整条链     0.875 MiB/每盘（两盘各一份）；切换预留里链那一半    224 槽 =     3.500 MiB/每盘
M= 1000000: 片数= 2711 一片 2 槽 ⇒ 链   5422 槽/每盘；每次挂载重写整条链    84.719 MiB/每盘（两盘各一份）；切换预留里链那一半  21688 槽 =   338.875 MiB/每盘
```

M 是挂载次数，行数近似取 M（每次可写挂载写一行，取号失败烧掉的号也写行，所以行数 ≥ 成功挂载数）；片数 = ⌈(M + 3) ÷ 369⌉，那个 +3 是 N_switch（`admission.rs:203` 上一行的 `checked_add`）。10⁴ 次 28 片、10⁶ 次 2711 片。「每次挂载重写整条链」是每次写行发布都 COW 整条，这一项是每次挂载都付的写，不只是预留。有回收时行数至多是「在根环里有根的实例数」，环有 3S 条根，所以至多 3S + 1 = 25 行（S = 8，推的），链恒为 1 片。
行的跨记录起点：一次发布的点名项超过一条记录装的 67 个时要跨记录，`M ≥ 24 723` 行（67 片 × 369）起；加宽记录头的 γ 路线把它推到 23 616（上面第二行，H1 的 γ）。

### H5 答：做不做

- 全条件（含整轮清扫）：做不了，输入缺一半（清扫器 `crates/` 里没有）；造清扫器是一个新部件，不在这一轮。
- 只做根环那一半：挂载里约 25 行加用例，但把 I-3.8 那句条件里的清扫半拿掉，那是 C113 定案 P2 的一部分，不是这一轮能改的，交用户。
- 推荐：第三里程碑不做，C597 改写成「等清扫器」，条款写上上面的表（10⁴：28 片、每盘每次挂载多写 0.875 MiB；10⁶：2711 片、84.719 MiB）与「有回收时行数 ≤ 3S + 1」两个数。

推翻条件：造得出一个盘面，删了某实例的行之后（其根环里没有它的根、每槽读成功），恢复或 checker 的结论变了（被抛弃的根重新可选、或扫描重建认了它的未发布单元）：那样条件缺一格，H5-a 的「没看出缺哪一格」就错了。

## H6　系统配置冗余（C462、C463）

被判对象：C462 行（`.claude/kb/checks-owed.md:382`「要先定：单列一个错误成员，还是由上层挡掉空池、恢复这一层不接受零设备。」）与 C463 行（`.claude/kb/checks-owed.md:383`「要先定报出来就够还是当场自愈重写。」）。

### H6-a　C462：零设备在哪一层挡

今天的样子：
- 零设备走到 `crates/singlefs-core/src/recovery.rs:1050`「(None, None) => Err(RecoveryFailure::SystemConfigurationsDisagree),」，它上一行的注释自己写着「池里一块盘都没有：没有哪一块盘点得出名，保持这个函数原先对空池的判定不变」（`crates/singlefs-core/src/recovery.rs:1049`）。`NoValidSystemConfiguration` 带一个必填的盘（`crates/singlefs-core/src/recovery.rs:166`「NoValidSystemConfiguration {」），空池没有盘可填，所以它借不来。
- 可写挂载已经在恢复之前挡了零设备：`crates/singlefs-core/src/mount.rs:4439`「admit_the_writable_device_count(distinct_device_identities_handed_in(devices))?;」，成员是 `WritableDeviceCountBelowTheStripeWidthLowerBound`（`crates/singlefs-core/src/mount.rs:245`），名字与原因相符。
- 只读挂载与直接调恢复的没有这道：`crates/singlefs-core/src/mounted_read.rs:555`「choose_system_configuration(reader).map_err(MountReadOnlyFailure::Recovery)?;」。`choose_system_configuration` 在冻结副本里有 44 个文件、97 处调用（`grep -rl 'choose_system_configuration(' --include=*.rs . | wc -l` 输出 44，`grep -rn` 数行输出 97）。

按 `.claude/singlefs-ai-sop/rules/code-discipline.md:141`「错误枚举的成员**按调用方要做的决定分**」与 `.claude/singlefs-ai-sop/rules/code-discipline.md:144`「**判据是调用方有没有因此失去该有的决定。**」判：`SystemConfigurationsDisagree` 让调用方去查「各盘是不是同一个池」，空池的调用方该做的是「给它盘」。今天两件事同一个成员，调用方失去了这个分辨，判据成立 ⇒ 单列成员。

「上层挡」这一支不合适：恢复这一层是 44 个文件共用的一道口，只读挂载的口在恢复上面一层、没有别的计数，上层挡要每个口各加一份；可写挂载那道已经有，对它没有新东西。

答：恢复这一层单列一个成员（名字取「池里没有一块盘」的意思，不缩写），零设备走它，`SystemConfigurationsDisagree` 只留 fsid 与设备数对不上那一支（`crates/singlefs-core/src/recovery.rs:1027`「return Err(RecoveryFailure::SystemConfigurationsDisagree);」）。代价（行数是推的估计，没写过）：枚举加一个成员带文档约 6 行；`recovery.rs:1049-1050` 改两行；文档里提到空池的话（如 `crates/singlefs-core/src/recovery.rs:961` 那一行）跟着改；harness 里对 `RecoveryFailure` 做穷举匹配的只有一处（`crates/singlefs-harness/src/history.rs:2233`「fn recovery_failure_member(failure: &RecoveryFailure) -> String {」，加一支约 3 行，`grep -rn 'RecoveryFailure::NoValidRoot =>' --include=*.rs .` 只命中这一个文件）；用例一条（C462 行的「拿一个零设备的 `PoolReader` 调恢复」，判别力自证是把成员改回去必红），约 15 行。共约 30 行。
推翻条件：造得出一个不经 `choose_system_configuration` 也把零设备放进去的入口（那时要在那个入口另挡）；或 `RecoveryFailure` 有 `crates/` 外的穷举匹配（那时加成员是破坏性改动，我只查了冻结副本）。

### H6-b　C463：报出来还是自愈

先对事实：C463 题面写「静默成功」，对不同入口不同。
| 入口 | 一块盘两槽全废、别的盘完好 |
|---|---|
| 恢复、只读挂载 | 用别的盘那一份照常挂上：`crates/singlefs-core/src/recovery.rs:1004`「(None, None) => {」那一支只记下第一块这样的盘、`continue`，选出来之后这个盘号丢掉（函数返回值只有系统配置）；`RecoveryReport` 四个字段（`crates/singlefs-core/src/recovery.rs:311`「pub struct RecoveryReport {」起）与 `MountedReadOnly` 四个字段（`crates/singlefs-core/src/mounted_read.rs:535`「pub struct MountedReadOnly {」起）都没有点名它的。用例把这个行为钉住了：`crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs:156`「fn both_system_configuration_slots_unreadable_on_device_zero_still_read_the_file_back() {」。**与 C463 题面一致** |
| 可写挂载 | 拒：读阶段那一判（`mount.rs:4304`）、取号读盘那一道（`transaction.rs:656`，点名那块盘）、逐盘核（`mount.rs:3198`）三处都会拦；题面「静默成功」对这一入口**冲突**（并排：题面「恢复改用别的盘那一份、照常挂上」`.claude/kb/checks-owed.md:383` ｜ 代码 `crates/singlefs-core/src/transaction.rs:656`「< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」的那个 `Err`） |

按 code-discipline.md:144 的判据：只读一侧的调用方今天不知道冗余少了一份，失去该有的决定 ⇒ **报出来**。做法：新增一个只读的函数交回「没有自证过的槽的盘」清单（不改 `choose_system_configuration` 的返回类型：44 个文件 97 处调用），`MountedReadOnly` 加一个字段，约 25 行 + 用例两条（盘 1 两槽全废点名盘 1；完好池清单为空，判别力自证是把字段去掉必红）。写盘一侧的拒因已经点名盘（取号那一道），读阶段那一道不点名，见 H4-c。

自愈重写：
- 重写用的就是同一个函数：`crates/singlefs-core/src/transaction.rs:344`「两槽都读不出时从 1 起」那行文档说的世代号 = 自证过的最大 + 1（一份都没有时从 1 起），落到槽 `世代号 mod 2`，内容由池参数现拼，不读旧槽（`crates/singlefs-core/src/transaction.rs:386`「let system_configuration = SystemConfiguration {」起）。所以「一槽缺」（H4 的 S-修）与「两槽全废」是同一个写，只差 `slot_generation` 从 g + 1 还是从 1 起。
- 但两槽全废的盘比一槽缺的盘多丢一层：它上面两份都没了，这块盘上可能记着池里最新的 journal tail 见证；自愈写回去的 tail 取别的盘的最大值（`transaction.rs:851` 一带），把那个见证抹掉。这就是 C331 甲要防的形（`.claude/kb/decisions/23-journal的角色与格式.md:389`「有一槽读不出或本池自证不过 ⇒ 判不出、按真」）。所以自愈**不能默认开**。
- 三处拒（`mount.rs:4304`、`transaction.rs:656`、`mount.rs:3198`）要一起松；一槽缺的盘只要松前两处。

答：C463 分两半。只读与恢复路径：报出来就够（默认、无风险、不改写盘行为）。可写路径：不自愈；显式放行（H4-c 那个参数）之后才自愈，放行的范围先只含「至少一槽自证过」的盘，两槽全废的盘要不要也放行，是把 D23 已定项 14 那条拒再松一格，交用户。

自愈重写与 H4 的 S-修 是不是同一个写：是（同一个函数、同一个落点算法，取号写里）。能不能合成一条条款：能，写成一句——「系统配置槽缺槽的重写只在取号写里发生；默认遇缺槽拒可写并点名盘与槽；调用方显式放行后取号写把缺的槽补成自证过；一份自证过的都没有的盘不在放行范围内，除非用户另定。」C596、C463 可写半共用这一条，C462 不在里面（它是错误名，不是写）。

推翻条件：
- 造得出「盘 1 两槽全废、盘 0 完好」的池，可写挂载不拒：说明三处拒里有一处我读漏了（`mount.rs:4304`、`transaction.rs:656`、`mount.rs:3198` 是我读到的三处）；
- 造得出一块盘缺一槽、放行后取号写写到了**自证过**的那一槽：说明世代号与槽位算法（`transaction.rs:365`）推错。

## 判定一览

判定只有三种：一致（我推出的与被判对象相同）、冲突（两边整行并排在对应小节里）、规则没说。

| 格 | 判定 | 一句话 |
|---|---|---|
| H1-a 补哪几个洞 | 冲突（半句） | 事实表 (c) 末表说更早的洞手里没有根记录字节；字节确实没有，那一版还在 journal 环里，补得了但不划算，只补 effective 那一版 |
| H1-b 补进去的字节 | 冲突 | 事实表说写进去就是有效根；今天的施加法在写行、首个文件、卸载三种发布上补出错根（推的）；γ 路线把三个字段加进记录，点名项 67 → 64 |
| H1-c 挂点 | 一致 | 事实表 (c) 末表的约束都成立；推荐 P1；P0 破「取号是第一个写」，P3 要另开根环表入口且窗口最长，P2 只有「两条根都持久后才记」不红 |
| H1-d/e 补后候选集与多扣 | 一致（推的） | 后洞每补一个少一条回退候选、门槛抬 1；前后洞多扣都回到 0，剩的是洞②、不补的三类洞、旧盘旧洞；q1a 模型与产物 264 行不符 0 |
| H2 崩在补洞中间 | 一致 | D13 已定项 4 域内 P1、P0、P2-late、P3 红 0，P2-eager 后洞红 5；补洞写只有落 / 没落两态 |
| H2 必红世界（盖 slot(h) 不带根字节） | 规则没说 | 里程碑出口栏要它判器红；I-7.4 与 I-4.8 不红，只有单镜像判不出的「缺一版」，要记录核对器判 |
| H3 I-3.9 收成单值 | 一致 | 仍有对象（被抛弃或低于 F 的根、读不出的根槽、洞②、P1 不补的洞），区间保留，C380 改写不关 |
| H3 事实表 (d) ① 补不了的旧洞 | 冲突（半句） | 根字节没有但记录还在，补得了；① 应写成「不补的旧洞」 |
| H3 C380「洞很长」 | 规则没说 | 没写长到多长算判不出；P1 后缺口至多一个实例边界一个 txg（推的） |
| H4 事实表 (e) 永远只读 | 一致 | 能重写那一槽的写都在两道拒之后（读阶段、取号读盘） |
| H4 正文的 S-辨 / S-修 分法 | 冲突 | 两者落在同一处改动（放开两道拒），取号写本来就重写那一槽 |
| H4 S-辨 与 C331 甲丢写 0 | 冲突 | 判决表甲 0、甲-读错 120（样例行都是双盘最新槽全零加最新根被藏，最少 4 个故障范围） |
| H4 D22 已定项 8 射程「每盘一槽系统配置坏在承诺里」 | 规则没说 | 没说承诺含不含可写；要用户定 |
| H5 I-3.8 回收条件 | 一致（推的） | 逐个消费者（判根、判单元）对过，没看出缺格；缺的是「整轮清扫」这个输入 |
| H5 删行那一写 | 一致（推的） | 落在写行那次发布的链重写里，不多一段写；窗口与今天写行发布相同 |
| H5 C333 前提 | 冲突（推的） | C333 说记账是那次发布的；代码里分配记录树根照抄所选根，可能过期，交主 agent |
| H6 C462 落点 | 规则没说 | 答：恢复这一层单列成员，约 30 行；可写挂载那道已有 |
| H6 C463 只读 / 恢复入口 | 一致 | 静默成功，`RecoveryReport` 与 `MountedReadOnly` 都没点名；答：新函数加一个字段 |
| H6 C463 可写入口 | 冲突 | 题面写「照常挂上」，今天三处拒；自愈与 H4 同一个写、同一个放行参数 |

## 没做什么

- 没造镜像验：H1-b 的三种红（α 路线）、H2 必红世界 2 与 3、H4 放行后行为、H5 删行后的 checker 结论都是读代码推的；小模型（`research/prompts/m3-ring-r1-sonnet-model/`）只覆盖 H1-d/e、H2-a/b 与 H5 的数，抽象是一版一个单元、S = 4，它与 `crates/` 不共用代码。
- 没跑 E156 装置与任何 checker 档测试、没编译 Rust（重型测试：不跑）；「槽数」的数只能引 E156 产物（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out`），我算的是树表张数。
- H4 三条候选的丢写 / 多拒数取自上一轮判决与攻方产物，没重量；读攻方脚本怎么数「故障范围」没做。
- 事实表的行号没有整表复核，只复核用到的那些；C583 没查。
- 攻方腿这一轮的提示与产出、并行四轮（m3-inv-a-r1、m3-inv-b-r1、m3-delete-r1、m3-formal-r1）的腿报告、模型目录与交接摘要，一个都没读。
- 不采纳、不出判决：「推荐」只是这条腿的推，交用户的表由主 agent 定；腿提的新形态（H4 的显式放行参数、H1 的 γ 路线、H2 的记录核对器检查）都是零轮，交用户时按规则标「零轮」。
- 接续说明：上一条同立场的腿撞限额中断，没写出报告；我核了它落盘的四份小模型脚本（本轮重跑，产物与落盘的 `cmp` 逐字节相同）与它草稿目录里的四段草稿，重过 cite-check，改了观测 3（补上取号读盘那道拒）、事实表核对的说法、结论先行第 4、6、7 条；H4、H5、H6 与本表是本轮新写。
- 草稿：`/tmp/claude-1000/m3-ring-r1-sonnet-2/`（分段草稿与几份重跑输出，没建编译目录或仓副本）；上一条腿的草稿目录 `/tmp/claude-1000/m3-ring-r1-sonnet/` 我没动。

## 交回闸

```
$ python3 research/scripts/cite-check.py research/prompts/m3-ring-r1-sonnet-output.md
  ✓ 核了 108 处引文，对上 108 处，没判 0 处（逐处列在上面）
```

各段落盘前逐段跑过同一个脚本（`--root /tmp/claude-1000/m3-ring-r1/tree --root .`），全文落盘后不带 `--root` 再跑一遍，同一行结果。
