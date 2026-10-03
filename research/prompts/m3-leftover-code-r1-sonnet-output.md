# m3-leftover-code-r1 云端正推腿报告（Sonnet，2026-10-03）

格：K1、K3、K4、K5（正文第三节）。立场：从条款或定案原文推到冻结副本的行，再推到钉它的测试与变异行。读代码、引行号一律对着冻结副本 `/tmp/claude-1000/m3-leftover-code-r1/tree/`（引文里写绝对路径）；引 kb 与 records 的行号是仓里今天的文件。模型与探针在 `research/prompts/m3-leftover-code-r1-sonnet-model/`。

## 0　先说三件和背景材料对不上的事（全部是现查的）

1. **M1 ① 的条款今天已不是「取甲」。** 背景材料第一节 M1 行与 K1 把 ① 写成取甲。kb 已在 2026-10-03 改成 R2、实现留在取甲，kb 自己标了欠：
   - 条款：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）
   - 条款：「这里的实现还是 2026-09-28 取甲的那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
   - 欠账行：「定案改取 R2、实现在途：见证读判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过」（`.claude/kb/checks-owed.md:272`）
2. **C600 的种子今天复现不了。** 在 `dbcc0d2f`（C600 记账时的提交）上单跑这个种子（登记行里那组环境变量，SEEDS=1），新发现 1 条，与 C600 登记一致；在冻结副本上跑同一组，新发现 0 条（命令与输出在 K3）。同一个种子在今天生成的是另一段历史，「把种子 7463871032432355222 那段钉成一条 harness 用例」按种子钉不出来，要直接造盘面。
3. **C600 拒的不是「更新的发布读不出」。** 拒因成员名叫 PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion，C600 复现时里面的读数是 witnessed_journal_counter 0、比较格 SomeSystemConfigurationSlotUnreadOrUnverified：没有见证到任何发布，是取甲「缺一槽」那一支。背景材料 K3 把它写成「见证读里有更新的发布、选中的那一版读不出」，读数不是这样（K3 有输出）。

## 1　K1　五条定案：条款对实现（M1）

### 1.0　九月二十八日之后别的批有没有改掉定案的行为

把 `research/prompts/_m2-five-decisions-code-r1-diff.md`（09-28 整段抽录）里的 14 项逐项与冻结副本里同名项比（`k1/cmp.py`，输出 `k1/cmp.out`、`k1/cmp2.out`）。去掉空白、行尾逗号与 `//` 注释后：

- 逐字相同 11 项：WitnessedCounterComparison、witnesses_a_publish_newer_than_the_selected_version、newer_publish_witness、StillUnreadableAfterOneReread、placements_referenced_by_root_computed_from_the_mapping、raise_the_floor_through、self_verified_system_configurations_of_every_device、InstanceAcquisitionFailed、ExpectedInstanceAcquisitionFailed、effective_rollback_floor_of_the_roots_read、deferred_slots_referenced_only_below_the_floor_per_device。
- 有差：`isolate_slots_referenced_only_by_abandoned_roots`（只是 `return Err` 的写法换成 `None => return Err(...)`，我逐行对过，两边的控制流相同）；`rebuilt_allocator`（多一个匹配臂 `PreviousVersion::WithTreesAndNoFile`，别的批加的，与 M1 无关）；`check_pool_image`（别的批加了若干别的检查，`k1/cmp2.out` 逐段列出；F_生效 与 I-3.1 的并集那几段 `cmp2.py` 逐段对过没动；根环最新根的 `max_by_key` 换成了 `fold`，平局取读根环次序里最前的那条，与 「pub fn choose_root(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:1265`） 的取法一致）。
- 结论：定案的行为没有被别的批改掉。**推翻它的现象**：对 HEAD 以外的某个提交重跑 `cmp.py` 在上面 11 项里报 DIFF。

### 1.1　① C331 取甲（可写挂载见证读缺一槽判不出；取号那一刻缺一槽在第一个取号写之前拒）

- 用户 2026-09-28 的话：「选甲 但是不需要再开三轮了 等之后开」（`records/2026-09-28-收口表对齐与三笔欠账复查.md:51`）。
- 条款（今天）：D23 已定项 14 的 N-配置续一段：「判不出就在第一个取号写之前拒」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
- 代码（见证读）：判「缺一槽」的两行与它们的出口：
  - 「< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4676`）
  - 「let comparison = if some_system_configuration_slot_unread {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4678`）
  - 「| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:409`）
- 代码（取号）：「< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/transaction.rs:697`），并且读在第一道屏障之前：「self_verified_system_configurations_of_every_device(pool)?;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/transaction.rs:875`）
- **判定：对 09-28 的取甲定案，一致。对今天 kb 的 R2，冲突。** 并排两句：
  - 条款：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）
  - 代码：每盘按「自证过的槽数 < 2」判（「< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4676`）），不分读错、空白、撕裂。分不开是因为入口函数只交回自证过的槽：「pub fn verified_system_configuration_slots<Reader: PoolReader + ?Sized>(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:3232`），它的读是 「.filter_map(|offset| reader.read(device, DeviceOffsetInBytes(offset), slot_bytes))」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:3241`），读错和自证不过一起被丢掉。要改成 R2，三个调用方（见证读、取号、F_生效）都得拿到每个槽的三态（自证过 / 读得出而自证不过 / 读错）。
- 逐盘核那一支：「let devices_without = devices_without_the_selected_version(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4047`） 之后，`Err(device)` 出口 「return Err(device);」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:3294`） 并进取甲的拒因 「fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:3331`）。`SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration` 那一支今天不存在：`grep -rn 'SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration' crates | wc -l` 输出 0，`grep -rn 'NoSelfVerifiedSystemConfiguration' crates | grep -v 'OnTheDeviceHandedIn' | wc -l` 输出 0（剩下的 `…OnTheDeviceHandedIn` 是 `DeviceTableDisagreement` 的成员，是挂载之后各入口的核，不在可写挂载路径上）。经可写挂载走得到的只有「读阶段两槽都读得出、逐盘核或取号再读时缺」（两次读之间的瞬时读错），不是那一支。
- 钉它的：变异 「C331 甲（用户 2026-09-28 定）：可写挂载见证读缺一槽不判不出」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1413`），红的用例 `a_writable_mount_refuses_before_acquisition_when_the_first_devices_newest_system_configuration_slot_does_not_verify`（造法 `/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs:18`：清零世代 5 那一槽）；取号：「C331 甲 V4（用户 2026-09-28 定）：取号那一刻缺一槽照样取号」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1414`）。
- 条款有而没有变异行钉的：第二块盘那一格（用例 `…second_devices…` 在，变异命令只点名第一块盘）；读错形（用例 `mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write` 在 `fsync_drop_and_devices_without_the_selected_version.rs`，变异表里没有以它为命令的行）；逐盘核的 `Err(device)` 出口 「return Err(device);」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:3294`）：`grep -c 'return Err(device)' crates/mutations.tsv` 输出 0。
- **推翻「一致」的现象**：有一块盘只缺一槽、两遍读又都读得出，可写挂载却放行；或取号写之前有任何一个写落盘。

### 1.2　② C393 取 (b)-从映射现算

- 条款（D23 已定项 14 影子账一句，今天仍在）：「被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）
- 条款后半：「映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）
- 代码，逐句对：
  - 账读不出就不跳过、计数：「unreadable += 1;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:1351`）（函数 「fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:1312`）），现算：「fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:1414`）。
  - 只在内存里隔离、不写盘：`isolate_slots_referenced_only_by_abandoned_roots`（1312–1411 行）与现算函数（1414–1475 行）的函数体里 `write_at`、`write_zeroes`、`barrier` 三个词各数 0 次（`awk` 截行段再 `grep -c`），隔离走 `allocator.isolate_abandoned`。
  - 计进 abandoned_roots_unreadable 往上报：影子账算完交回的 `ShadowLedgerComputed.abandoned_roots_unreadable`，经 `rebuilt_allocator` 进挂载输出的 「pub abandoned_roots_unreadable: u64,」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:890`）。
  - 现算不成就重读一次再拒：重读钩子之后 `or_else` 再现算，仍不成交 「StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:1371`）。
  - 取号之前拒：影子账在 `rebuilt_allocator` 里算，调用点 「} = rebuilt?;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4923`） 早于 「establish_instance(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4930`）（建立实例即取号）。
  - 抬 F 路径重算影子账失败，分配器换回原样：「*allocator = allocator_before_the_raise;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:2562`）（该分支在 「Ok(computed) => computed.abandoned_roots_unreadable,」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:2560`） 之后），条款没写这一句，是实现补的。
- **判定：一致**（条款没写抬 F 路径要把分配器换回原样，实现多做了这一句，算「条款没说」）。
- 钉它的：变异 「C393 (b)-从映射现算（用户 2026-09-28 定）：从映射一律现算不成」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1415`）（红在 `c393_tree_table_unreadable_on_every_read`）；不计数那一行在 09-28 之前就有（`mutations.tsv` 第 730 行起头「实三（影子账…」账读不出的被抛弃根不计数）。
- 没有变异行钉的：最后一支「仍不成就拒」。用例有：「fn an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:565`），但 `grep -n 'AccountOfAnAbandonedRoot' crates/mutations.tsv` 零命中。这个用例断言拒因点名那条根，没断言「盘上逐字节不变」。

### 1.3　③ 被抛弃根带的 F 算进 F_生效（实现与 checker 各一份）

- 用户 2026-09-28 的话：「算进（推荐）」（`records/2026-09-28-收口表对齐与三笔欠账复查.md:55`）。条款（D16 已定项 1 依据一条）：「被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定」（`.claude/kb/decisions/16-发布语义.md:55`）
- 代码（实现）：「let highest_on_the_abandoned_roots = roots_with_ring_slots」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:1864`），并进最大值：「.chain(highest_on_the_abandoned_roots)」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:1875`）。代码（checker）：「let highest_floor_on_the_abandoned_roots = roots」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:6956`），并进最大值：「.max(highest_floor_on_the_abandoned_roots);」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:6968`）。「被抛弃」两边都按最新根指着的实例表判：核心读行里的 「pub selected_root_txg: CheckpointTxg,」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/instance_table.rs:23`）（按 `to_bytes` 的次序是 kind 1 字节、实例 4 字节之后的 8 字节），checker 读 「published_checkpoint_txg: u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:4130`）（偏移 5 起的 8 字节），是同一个量。
- **判定：一致。另有一处条款自己不一致（kb 内部，不归代码）：** D16 同一项的「生效」表格行写的是
  - 「恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 系统配置里读得出的 F 的最大值)」（`.claude/kb/decisions/16-发布语义.md:39`）
  - 这一行没有被抛弃根；同一项依据里那一条（上面第一条引文）才有。代码照后者做，表格行是旧的。
- 钉它的：变异 「收口表第 ② 行算进（用户 2026-09-28 定）：实现的 F_生效 不算被抛弃根带的 F」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1417`）（实现）与 「收口表第 ② 行算进（用户 2026-09-28 定）：checker 的 F_生效 不算被抛弃根带的 F」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1418`）（checker），各点名一个用例。
- **推翻「一致」的现象**：造一个被抛弃根带最高 F 的镜像，实现与 checker 算出的 F_生效 不等于 max(含被抛弃根)。
- 实例表读不出时（`newest_roots_table` 为 None）核心不按表滤、所有根按有效根取最新，checker 同（实例表行一行都没有）：这一格两边相同，条款没说。

### 1.4　④ I-3.1 的遍历并上 F_生效 之下仍在 defer 的槽（丁-defer）

- 用户 2026-09-28 的话：「丁-defer（推荐）」（`records/2026-09-28-收口表对齐与三笔欠账复查.md:55`）。条款（invariants.md I-3.1 行）：「F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于 F_生效 的」（`.claude/kb/invariants.md:143`）
- 代码，逐个限定词对：
  - 「F_生效 之下」且「没被抛弃」：「&& root.checkpoint_txg < effective_rollback_floor」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:7274`）（同一个过滤里还有 `!abandoned_by_the_newest_roots_table(root)`，7273 行）。
  - 「候选集那一遍没走到」：「if !walked_slots.contains(&key) && released_above_the_floor.contains(&key) {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:4670`）
  - 「最新根的账里是已释放、释放代高于 F_生效」：「if record.is_released && record.generation > effective_rollback_floor {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:4649`）
  - 并进和：「+ deferred_below_the_floor;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:7317`），函数头 「fn deferred_slots_referenced_only_below_the_floor_per_device(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:4616`）。
- **判定：一致。** 条款自己留了一个口：「那一类错，没拿变异试过」（`.claude/kb/invariants.md:143`）。09-28 起没有补。变异 「收口表第 43 行丁-defer（用户 2026-09-28 定）：I-3.1 的遍历不并 F 之下仍在 defer 的槽」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1416`） 把这一项乘 0（整项去掉），只证「并上它」承重，不证它「不盖住该回收而没回收的错」。
- 钉它的：上面那行变异的用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green`。

### 1.5　⑤ C379 取 B

只动 kb、没有代码，背景材料第一节 M1 行也没列它；本腿不判。

### 1.6　K1 的整体结论与能推翻它的现象

- ②③④ 与 09-28 的定案及今天的条款一致，行为没被别的批改掉（1.0）。① 对 09-28 定案一致、对今天 kb 冲突。
- 条款里有而没有变异行钉的：① 的第二块盘与读错形、逐盘核 `Err(device)` 出口；② 的「仍不成就拒」；④ 的「不盖住该回收而没回收」。这四处是漏洞候选，不是判错。

## 2　K3　C600：设备吞掉一次系统配置槽写之后的可写挂载

只交两边的结局与依据，不替用户选。

### 2.1　事实（复跑过）

- 在 `dbcc0d2f` 的 `git archive` 副本里单跑登记行的种子：`SINGLEFS_FAULT_INJECTION_FIRST_SEED=7463871032432355222 SINGLEFS_FAULT_INJECTION_SEEDS=1 SINGLEFS_FAULT_INJECTION_OPERATIONS=30 SINGLEFS_FAULT_INJECTION_FAULTS=6 cargo test --release --offline -p singlefs-harness --test fault_injection_fast_tier -- --include-ignored --exact fault_injection_large_tier_from_the_environment`，经 `capped.sh 6` 与 `run-with-memory-cap.sh`。输出（`k3/seed222-dbcc0d2f-debug.log` 第 48–50 行）：「种子 7463871032432355222 write_is_swallowed：整池第 19 次write，落在起点段（mkfs、取号、暖机、第一个文件）：ModelDisagreement { aspect: "模型说该成、实现拒了" }」，重开「走到实例 0 第 0 代根、没有文件」。这份副本我在 `mount_error_member` 与 `refusal_reason_of_mount_error` 前加了 `eprintln!`，把拒因打全：
  - `selected_version: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }`，`witnessed_journal_counter: 0`，`comparison: SomeSystemConfigurationSlotUnreadOrUnverified`，第一遍与重读一遍相同。
- 同一组环境变量在冻结副本上：`k3/seed222-today.log` 末段「以「已知红」收尾 {}、新发现 0」，注入点的分布也变了（`dbcc0d2f` 上那份是 TheStartingPoint/system_configuration_slot，今天是 CloseAndMountWritable/system_configuration_slot）。同一个种子今天生成另一段历史（加了新的历史操作种类，抽样序列变了；这一句是推的，没去二分哪一次提交改的）。
- 记录里的复现描述：「种子 7463871032432355222 吞掉起点段的一次系统配置槽写（整池第 19 次写）」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:36`）
- 把 C600 的形直接造出来（`k3/k3_probe.rs`，我在冻结副本的拷贝里加的 harness 档用例，不入库）：mkfs 之后一个系统配置槽清成空白（读得出、自证不过），四种位置各做一遍。输出（`k3/k3_probe.out`）：
  - `mount_writable` 四格都拒，拒因同上（`witnessed_journal_counter: 0`、`SomeSystemConfigurationSlotUnreadOrUnverified`）；
  - mkfs 同一个进程里的 `acquire_instance` 四格都拒，成员 `DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`；
  - 对照（不清槽）`mount_writable` 成功。
- 为什么会有空白槽：mkfs 两个槽写同一份字节，各是一次普通写（「for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/make_filesystem.rs:651`））。mkfs 里被吞的那一次写，那个槽就一直是空白。C600 复现的第 19 次写落在 mkfs 的什么位置我没有逐写核对（只看到 injection 点的名字是系统配置槽写、读数是 counter 0 与缺一槽），这一句是推的。
- 模型一边：发现原文（`k3/seed222-dbcc0d2f-debug.log` 第 50 行）写的是模型 MountWritable 该成、「允许拒的只有容量墙区间与 []」。把设备吞写造成的拒豁免掉的函数只认一种：根环槽重读仍坏、且那个槽是被吞的根槽写——函数 「pub fn left_by_the_lying_device(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/fault_injection.rs:2218`），豁免条件读的是 「.implementation_reported_root_ring_slot_still_bad_after_one_reread」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/fault_injection.rs:2245`）。

### 结局表（两块盘、每盘两槽；「今天的实现」就是出路 A，出路 B 一列是我按 kb 的 R2 定义推的）

| 盘面 | 今天的实现 = 出路 A | 出路 B（R2，推） | 依据 |
|---|---|---|---|
| 每盘两槽都自证过 | 比较格定 | 同 | 对照格 `a_writable_mount_with_every_system_configuration_slot_verified_goes_ahead` |
| 一槽空白或撕裂（读得出），其余自证 | 可写挂载拒、取号拒 | 可写 | 我跑的 4 格（mkfs 之后）；`a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs` 那一族（世代 5 清零，两块盘各一格） |
| 一槽读错，另一盘两槽自证 | 拒 | 可写 | 代码：读错的槽被 `filter_map` 丢掉、该盘自证槽数 < 2；整盘读错的用例在 `fsync_drop…`（拒，没重跑）；R2 的形式定义 |
| 每盘各一槽读错 | 拒 | 拒（判不出） | R2 的形式定义；C596 行 |
| 每盘各一槽空白（读得出） | 拒 | 可写（没有读错） | R2 的形式定义；这一形若同时最新根读不出，条款明文不罩 |

### 2.2　出路 A：模型认这种拒（挂载的判法不动，仍是取甲）

- 改动落点（`crates/` 里）：`singlefs-harness/src/fault_injection.rs` 的 `left_by_the_lying_device`，在 「ModelDisagreementAspect::RefusedWhenModelRequiresSuccess」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/fault_injection.rs:2243`） 那一臂里加一条：实现的拒因比较格是 `SomeSystemConfigurationSlotUnreadOrUnverified`，且被吞的写里有这块盘的系统配置槽写。拒因对应的模型理由今天是 「StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/model_comparison.rs:796`）（成员名叫「更新的发布」，C600 的读数里没有任何发布被见证）。
- 在哪些合法历史上给出什么结局（取甲，已有用例或我跑过的）：
  - 一槽空白（mkfs 里被吞、或撕裂的系统配置轮换）：可写挂载拒、取号拒，之后每次可写挂载都拒，只读挂载照常。我跑的 4 格（上面）和 「fn pool_with_one_system_configuration_slot_zeroed(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs:18`） 那一族（世代 5 那一槽清零，两块盘各一格）。
  - 读错形：`fsync_drop_and_devices_without_the_selected_version.rs` 的 `mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write`：拒（我没重跑）。
  - 两槽都自证过（含「被吞的是轮换写、旧槽仍自证」的陈旧形）：不缺槽，结局由 counter 比较格定，本格不拒（对照格 `a_writable_mount_with_every_system_configuration_slot_verified_goes_ahead` 是未损坏的池，不罩陈旧形；陈旧形我没跑）。
- 依据与代价：条款里这条代价早已写明，用户 2026-09-28 选甲时知情：「撕裂的系统配置轮换（普通断电就造得出，被撕裂的那次发布没确认过）不再因判不出拒可写挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:408`）。池一旦有一个槽空白，永远只读，出路没有条款：「R2 之后仍判不出的那一形（有槽读错、没有一块盘两槽都自证过）一次可写挂载都做不成」（`.claude/kb/checks-owed.md:459`）。从映射现算不成的那一形同样：「C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:408`）
- 会不会漏判倒挂：取甲的拒绝集合包含 R2 的拒绝集合（推：R2 判不出要求有读错且没有一块盘两槽自证过，这时必有一块盘自证槽数 < 2，取甲也拒），所以取甲相对 R2 只会多拒、不会多漏；09-28 的判决记录也写「甲 唯一丢写 0」：「甲 唯一丢写 0 但在撕裂系统配置轮换这种普通断电造出的合法崩溃状态上拒可写」（`records/2026-09-28-收口表对齐与三笔欠账复查.md:70`）。会不会把别的形压成永远只读：不新增，已有的全部保留（包括 R2 会放开的那几类）。

### 2.3　出路 B：改挂载的判法（kb 今天写的是 R2）

- 条款：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。丢写的界：「丢写的界按类写：全池读回一个值的纯形丢写 120 格，一盘报读错、另一盘读回全 0 的混合形丢写 0 格」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。单故障承诺与双故障：「容错承诺是单故障：每盘一槽系统配置坏在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）；「每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）
- 按 R2 推（推的，没实现；实现要先把 `/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:3232` 改成三态）：
  - 一槽空白或撕裂、读得出：没有读错，不是「判不出」，取其余自证过的槽的 tail 最大值去比，C600 的形（counter 0）⇒ 没见证 ⇒ 可写；取号写重写那个槽：「一槽撕裂、另一槽自证过的盘面放开拒可写的两道，取号写重写坏的那一槽」（`.claude/kb/checks-owed.md:459`）模型的「该成」成立，C600 自然消失。
  - 一槽读错、另一盘两槽自证过：有读错但有一块盘两槽自证过 ⇒ 判得出 ⇒ 可写。D23 另一句的括注把「一槽系统配置持续读错」写成仍判不出的那一形：「仍判不出的是有槽设备报读错、没有一块盘两槽都自证过那一形（一槽系统配置持续读错），可写挂载每次都拒」（`.claude/kb/decisions/23-journal的角色与格式.md:408`），字面与形式定义不完全同义（形式定义下另一盘两槽自证过即判得出），我按形式定义推；这是 kb 里两句的出入，要主 agent 或用户认哪一句。
  - 每盘各一槽读错：有读错且没有一块盘两槽自证过 ⇒ 判不出 ⇒ 拒（永远只读，出路仍没有条款：「R2 之后仍判不出的那一形（有槽读错、没有一块盘两槽都自证过）一次可写挂载都做不成」（`.claude/kb/checks-owed.md:459`））。
- 量过的数（kb 里，不是我量的）：「混合形（拒可写 / 可写不丢 / 可写丢）甲-读错-每盘 888 / 831 / 240，R2 1244 / 715 / 0」（`.claude/kb/checks-owed.md:272`）；D22 依据一条：「纯形 120、混合形 0，混合形多拒 116 格」（`.claude/kb/decisions/22-单元原子性怎么合成.md:187`）。
- 会不会漏判倒挂：丢写 0 格（混合形）、120 格（纯形，都落在「每盘各坏一槽系统配置同时最新根一时读不出」的双故障形，条款明文不罩：「每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`））。会不会把别的形压成永远只读：比取甲少，剩下两类（每盘一槽读错且无一盘两槽自证）。
- 另一个牵连：C662 登记了带故障的合法历史一族照取甲生成，R2 进代码之后要重跑，读数不能当 R2 的依据：「R2 进代码之后这一族生成的历史与条款不再是同一套写路径」（`.claude/kb/checks-owed.md:524`）。

### 2.4　K3 里我没法判的

- 「模型认这种拒」与「改判法」各自在随机历史上改变多少段的结局：要跑 512 段大档（原跑 debug 4722 秒），没跑。
- 出路 B 的实现与它在 harness 档上的结局：没实现，没跑。
- C600 条款的现状：「没有条款，用户 2026-09-29 定先记欠账」（`.claude/kb/checks-owed.md:463`）；里程碑三项五·6 还写着「条款先定」：「| 五·6 | 大档故障注入 2 条新发现」（`.claude/kb/milestone/03-third-txn.md:117`）。今天 kb 里 R2 已经是 C331 的定案，按 R2 字面 C600 的形不属「判不出」，出路 A 与出路 B 是否还是二选一要主 agent 判。

## 3　K4　写零折叠（M2，`device_log.rs`）

### 3.1　分块的切法与 `write_zeroes_in_chunks` 逐字一致吗

- 后端的循环：「let this_chunk = (length - written).min(length_of(zeros));」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/block_device.rs:364`）。暂存区长度是整段与块大小取小：「fn zero_fill_chunk_length(length: u64) -> usize {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/block_device.rs:344`）；常量：「pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/block_device.rs:112`）。检查所在的虚机里跑的是 O_DIRECT 后端，它的暂存区也是这一个长度：「let scratch = AlignedScratch::new(zero_fill_chunk_length(length));」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/block_device.rs:717`）；装置里用的就是这个后端：「let mut devices: Vec<(DeviceIdentity, CountedDevice<DirectInputOutputBlockDevice>)> =」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1419`）。
- 宿主检查的切法：「let chunk_index = (position - fill_start) / ZERO_FILL_CHUNK_BYTES;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:281`）（块起点从**这一段的起点**起算，块长同一个常量：「use singlefs_core::block_device::ZERO_FILL_CHUNK_BYTES;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:12`）；末块 `min(…, fill_end)`）。
- **判定：一致。** 两边都从声明的 `offset` 起、每 `ZERO_FILL_CHUNK_BYTES` 一块、末块到段尾为止。例外一条：文件后端的打洞路径（「HolePunchFailure::PunchingUnsupported => {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/block_device.rs:479`））不分块，但虚机里的来宾装置走 O_DIRECT 后端，不走它。
- 复核不了：用例注释里「来宾内核把后端一块拆成两个同时在途的请求（2032 + 6160 扇区）、blklogwrites 按数据盘写完的先后记日志」要虚机；我没有读到产物，只读到注释。**推翻这个前提的现象**：虚机里出现一块之内的两部分以外的乱序（例如下一块的写排在上一块的写之前）。

### 3.2　「块内不论次序」会不会把真缺了中间一块的写零折成整段

- 机制：已铺满的末端 `covered_up_to` 只因两种事增长——一条写正好接在它后面，或先到的若干条在它之后接上；中间缺任何一截，`covered_up_to` 停住，下一条不接得上就返回 None（「let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:320`）、「piece.start > covered_up_to && piece.end <= chunk_of_cursor.end;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:322`）），日志用完也返回 None。重叠条目被拒：「.any(|earlier| earlier.start < piece.end && piece.start < earlier.end);」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:328`）。
- 实测（`k4/model.rs` 是 `device_log.rs` 第 238–354 行逐字，加桩与对拍；块大小取 3、4 两档，段起点取 0、1、2、5，段长 1–8，序列长 ≤5（段长 ≤5）或 ≤4，每一条写取段内所有起点与长度，加四种越界/非全 0 的条目（段前一格、段后一格、伸出段尾、非全 0））：对一个我写的精确口径（不重叠、全在段内、字节数恰为段长、块与块之间按序）`oracle_strict` 逐序列对拍，**59 539 320 条序列，不一致 0**（`k4/model.out`）；再随机 4000 万条（块 2–6、段长 1–14，随机打乱块内次序、删一片、复制一片、加长一片、缩短一片、改成非全 0），**不一致 0**（`k4/model2-random.out`）。口径是我按用例注释里的规则写的，与实现不共用代码；规则本身若错（例如不该许跨块的一条写）这个对拍抓不到。
- 命令：`bash research/scripts/capped.sh 6 bash research/scripts/run-with-memory-cap.sh 2G /tmp/.../k4/model`，退出码 0；包装起初退 252（同一个会话已有别的腿在跑），设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 之后照常。
- **判定：不会。** 推翻它的现象：对拍里出现「实现折了、精确口径不折」的序列，或相反。

### 3.3　按序到达的一条跨块仍折，旧行为与新规则会不会在某种到达次序上打架

- 旧规则：diff 里被删的那几行（按序、首尾相接、每条全 0）。`k4/model3.rs` 把它逐字写成 `old_tile`，与新实现在同一批序列上对拍（`k4/old-vs-new.out`）：**40 749 960 条序列里旧规则折的 2 116 840 条，新实现全部折且条数相同；新实现多折 69 816 条（旧规则折不了的）**。没有一条「旧折、新不折」或「条数不同」。
- 多段声明同时能铺满同一串日志时，新实现取声明表里靠前的（「.find_map(|(declared_offset, declared_length)| {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:272`）），旧规则取起点等于首条写起点的那一段。我没造出会改变折叠结果的输入，也没去证没有。
- **判定：不打架（旧行为是新规则的子集）。**

### 3.4　长度 0 不折之外，还有没有不终止的输入

- 守卫：「if declared_length == 0 {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:295`）。去掉它（`k4/mut_noguard.rs`），声明长度 0 时 `entries_tiling_one_zero_fill` 交回 `Some(0)`，`fold_declared_zero_fills` 里 `index += 0`、不停地 push：在 1G 的内存包装里被 oom-kill，退出码 250（`k4/mut_noguard.out`；250 是包装自己的结局，输出不算结果，这里只用来证「去掉守卫就不终止」，我没有绕开包装重跑）。
- 其余输入都终止：每一轮循环吃掉日志里的一条（`consumed` 加一，`observed.get(index + consumed)` 取不到就 `return None`），`covered_up_to` 只增；长度为 0 的**日志条目**不终止之忧：模型里「段头夹一条长度 0 的全 0 写」被并进整段（`k4/model.out` 末两行），不挂。
- 两个 `assert` 能不能被输入打到：段尾那条 「"每一条都不越过段尾（接在末端的查了段尾、先到的查了所在那一块的块尾），已铺满的末端停在段尾"」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:350`） 等于号成立靠「接在末端的一条查了段尾、先到的查了块尾」；去掉接在末端那一条的 `piece.end <= fill_end`（`k4/mut_noend.rs`），一条越过段尾的接续写让它 panic（退出码 101，`k4/mut_noend.out`）而不是返回 None。今天这句在，所以不 panic。
- **没有用例钉的**：长度 0 守卫（库内用例里没有声明长度 0 的）；接续写越过段尾（用例里只有「先到的那一条伸出段尾」一个：`zero_fill_whose_short_last_chunk_gets_a_piece_running_past_the_end_does_not_fold`）；两个 `assert`。变异表第 1562–1569 行的 8 行罩的是别的事（例如 「设备侧日志折清零段时不查每条是不是全 0」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1562`）、「设备侧日志折清零段时最后一块不截到段尾」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1569`）），没有一行动这三处。
- **推翻「没有不终止的输入」的现象**：某个 `declared` 段在 `entries_tiling_one_zero_fill` 里让 `consumed` 不增长而循环不退。

## 4　K5　对接上游第三轮之后（M3）

### 4.1　窄一断言是不是用户定的那一句

- 用户定案的转述（`records` 2026-10-01 对接上游那份第 679 行）：「（只断言对基镜像照 journal 恢复落到的就是它里面最新的根）；第 3 条取」（`records/2026-10-01-对接上游测试脚本规范.md:679`），第 1 条那句整句是「只断言对基镜像照 journal 恢复落到的就是它里面最新的根」。
- 代码：断言 「fn assert_journal_recovery_of_the_base_image_lands_on_its_newest_root(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:453`）；落到的根取判器同一条恢复：「let landed_root = recover(&base_with_no_write_persisted, JournalPolicy::Consult)」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:462`）；「最新的根」取基镜像里魔数起头的扇区、不验自证：「fn newest_root_in_the_base_image(base: &MemoryPool) -> Option<(CheckpointTxg, InstanceGeneration)> {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:429`）、「bytes.len() >= ROOT_RECORD_IDENTITY_END && bytes[..ROOT_MAGIC.len()] == ROOT_MAGIC」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:434`），注释自述：「不验自证校验和：判器输入只要知道恢复最多落到哪一代」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:428`）；排计划时调用：「let newest_root_of_the_base = newest_root_in_the_base_image(flow.base);」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:529`）。`(txg, 实例)` 的次序与 `choose_root` 一致：「三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/recovery.rs:1260`）。
- **判定：一致，有一处偏严。** 断言的是「落到的根 = 基镜像里最新的根」，没有多断言别的。偏严的是「最新」按魔数认、不验自证：基镜像里有一个魔数起头而自证不过的根槽（坏根、撕裂）且它的 (txg, 实例) 最大时，`recover` 会落到更旧的有效根，断言报「越过」而拒。这种基镜像是不是合法的、登记的流里有没有，我没查到有；它只会多拒、不会漏拒（B0 的上界取高了只是多保留一些版本）。
- 钉它的：一条 `#[should_panic]` 用例 「fn a_flow_whose_base_image_is_a_crash_state_with_a_journal_record_past_its_newest_root_is_refused_at_planning(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:1028`）；变异 「崩溃放量 B0 前提 排计划时不断言基镜像照 journal 恢复落在它里面最新的根」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1768`）。放行的一面没有专门的用例，靠别的两流用例在同一个入口上不拒来罩。
- 会不会误拒今天登记的流：**复核不了**（要跑 checker 档）。主 agent 记的一次实跑：「依赖用例 `crash_identity::tests::every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries ... ok`」（`records/2026-10-01-对接上游测试脚本规范.md:683`）。
- **推翻「一致」的现象**：登记流里某条的基镜像照 journal 恢复落到的根 ≠ 基镜像最新魔数根，规划就 panic。

### 4.2　白名单表与「逐个引入者判」会不会被绕过

- 表：「pub const RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED: &[RandomHistoryThroughALibraryModuleAllowed] = &[」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:890`）（两行，都是 `crash_injection_campaign`）。死条目拦：「登记了而那条用例的闭包里没有这一引的是死条目，同一核也拦」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:889`）。
- 逐个引入者的判法：库模块 = 路径在 `crates/<包>/src/` 下（「fn is_a_library_module(file: &str) -> bool {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:938`）），不在的一律当「测试目标自己的文件」放行（「return RandomHistoryIntroducerVerdict::TestTargetFile;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:971`））。所以 `#[path]` 共用的别的包 `tests/` 下的搭建模块直接引随机历史脚手架，不用登记、不拦（用户 2026-10-02 定的是「按登记用例白名单」；库模块与测试文件这样分是实现的分法，不是用户那句话说的，这一句是推的）。例：`singlefs-harness/tests/common_admission/mod.rs` 今天就直接引 `history`（`HistoryDeviceWidth`）。
- 绕过实验（`k5/model.rs`，`crash_identity.rs` 第 273–397 行与 460–768 行逐字（398–459 行是类型定义，换成桩）加一个合成小仓，输出 `k5/model.out`）：被引文件列表
  - `use singlefs_harness::history::foo;` ⇒ `history.rs`
  - 全路径表达式 `singlefs_harness::history::foo()` ⇒ `history.rs`
  - `use singlefs_harness as h; h::history::foo();` ⇒ 只有 `lib.rs`，**没有 `history.rs`**
  - `extern crate singlefs_harness as hh;` ⇒ 空
  - `use singlefs_harness::*; history::foo();` ⇒ 只有 `lib.rs`
  - `use singlefs_harness::{self as sh}; sh::history::foo();` ⇒ 只有 `lib.rs`
  引入者表就是靠这个解析填的，别名写法下 `history.rs` 的引入者看不见。后果：没登记的库模块用别名引 `history`，不被拦，`history.rs` 也不进那条用例的判法摘要（复用键对随机历史的改动不敏感）。路径解析的依据：「name if crates.contains_key(name) => {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:680`）（第一段必须是工作区 crate 名）。今天仓里这类写法 0 处：`grep -rnE 'use singlefs_[a-z_]+ as |extern crate|singlefs_[a-z_]+::\{self as|use singlefs_[a-z_]+::\*;' crates` 零命中。
- 另一处只认测试 crate 的 `mod` 声明：「if text_at(position) == Some("mod") && text_at(position + 2) == Some(";") && is_test_crate {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:723`）；库 crate 里的 `mod x;` 加 `#[path]` 不当引用。今天登记的库模块里没有这样的写法（没有逐个核）。
- **判定：能被绕过，要用别名这类写法；今天仓里没有。** 我没把它接到真正的 `random_history_scaffolding_introducers_judged` 上跑（那要编 checker 档），用的是它下面 `references_of` 的逐字拷贝。**推翻「能绕过」的现象**：在真实仓里加一个 `use singlefs_harness as h; h::history::…` 的库模块，依赖用例红。

### 4.3　去空白与写 0 判红在四个入口上是不是同一个口径

- 数值的去空白：共用解析 「match text.trim().parse::<Number>() {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/numeric_environment_variable.rs:51`），`Result` 版 「match text.trim().parse::<Number>() {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:4735`），同一句 `text.trim().parse`。
- 写 0 判红：计数用 NonZero 读，各入口：崩溃注入种子数 「let seed_count = count_from_reproduction_environment_panicking_when_unreadable(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3085`）（只抽样时每段崩溃点 「crash_points_per_history: std::num::NonZeroUsize::new(points)」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3101`）、短段写数上限 「segment_writes: positive_count_from_the_variable_text(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3112`））、坏盘输入 「let seed_count = count_from_reproduction_environment(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/bad_disk_input.rs:911`）、故障注入每段注入次数 「let faults_given = count_from_reproduction_environment(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/fault_injection_fast_tier.rs:147`）、随机历史种子数 「number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/random_histories.rs:1889`）、每段步数四个入口共用 「let operations_per_history_given = count_from_reproduction_environment(」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:4849`）；种子起点读非负整数（0 合法）。
- **判定：数值一致。** 三处同一入口内不是一个口径，都不是静默放行：
  - 选项型变量不去空白：「.find(|choice| environment_value_of(*choice) == value)」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:4778`）（逐字比较；读代码，没跑），`" broad"` 拒、数值 `" 11"` 收。
  - 崩溃注入的短段写数与 `none` 全等比较（「let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3099`）），`" none"` 不当 `none`、掉进正整数解析、panic 认不出。
  - 两种消息形态（panic 文本、`Result` 的 `render`），用例各钉各的串。
- 另：只抽样时每段崩溃点写 0 判红、短段整段枚举时长段另抽 0 个合法（`zero_crash_points_beside_whole_short_segments_reads_back_as_short_segments_only`），是定好的区别，不是漏洞。
- **推翻「数值一致」的现象**：某个数值变量写 `" 11"` 在一个入口收、另一个入口拒。

### 4.4　抬 F 搭建拆分之后，使用者拿到的池几何与拆分前逐项相同吗

- 使用者数：`mod common_admission;` 的 harness 档文件 10 个（`for f in $(grep -l 'mod common_admission;' crates/singlefs-harness/tests/*.rs)` 数出 10 个），其中起池的 9 个（调 `start_after_the_first_file` 的 4 个、调 `start_plain` 的 5 个，两者有重叠；另有一个窄盘起池函数 `start_on_narrow_devices_…` 是别的批加的，HEAD 里没有），只借 `content_of` 等小函数、不起池的 1 个（`admission_checkpoint_cost_per_device_paths.rs`）；checker 档 1 个（抬 F 那条，经 `common_admission_pool`）。实现员说的「9 个使用者」是起池的那 9 个。
- 起池的 9 个：`start_plain` 是 `start_after_the_first_file(device_width, plain_device)`，里头仍取 `HistoryDeviceWidth` 的参数与盘宽：「device_width.device_bytes(),」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission/mod.rs:44`）；池的起池函数体与 HEAD 的 `start_after_the_first_file` 去空白后逐字相同（`git show HEAD:…` 与冻结副本的比较，我拿 python 比的，1517 个字符对 1517 个字符）；`start_plain`、`space_admission_of_a_crash_remount_on_a_copy` 挪回了 `common_admission`，函数体逐字相同；`plain_device`、`plain_devices_on`、`checker_violations_on` 挪进池文件，函数体逐字相同。**这 9 个的池几何与拆分前相同**（路径多一层 `#[path]`：「#[path = "../common_admission_pool/mod.rs"]」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission/mod.rs:8`））。
- checker 档那 1 个用的是抄的数，**抄了三个不是两个**：「const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission_pool/mod.rs:50`）、「const RING_WINDOW_ALLOWANCE_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 815;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission_pool/mod.rs:55`）、「const JOURNAL_RING_BYTES_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 6 << 20;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission_pool/mod.rs:59`）；原件：「const SMALL_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 384;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:133`）、「const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 815;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:146`）、「const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 6 << 20;」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:158`）。三对逐字相同。
- 抄本与原件的核对：harness 档用 384 槽那一档起池时调用 「assert_the_384_slot_copy_matches_the_random_history_width();」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission/mod.rs:36`），比较 mkfs 参数、盘宽、按落点分写的几何三样；变异 「准入池抄的 384 槽那一档单元区槽数与 history.rs 那一档对不上」（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/mutations.tsv:1772`） 只改了 384 那一个数。815 与 6 MiB 没有变异行（断言比盘宽与参数，会红，我没跑）。
- **判定：起池的 9 个 harness 使用者一致；checker 档那 1 个与原件一致，但靠一个只在 harness 档起 384 槽的用例时才跑的断言守着。** 没有 harness 档用例按 384 槽起池，或那条断言被改，checker 档的抄本会悄悄漂移。**推翻「一致」的现象**：`history.rs` 里那三个常量之一改了而 `common_admission_pool` 没改，同时 harness 档仍有 384 槽用例绿。

## 5　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 后续批 | 一致 | 11 项逐字相同，3 项只是格式或别的批加的臂，定案行为没被改 |
| K1 ① C331 取甲 | 对 09-28 定案一致；对今天 kb 冲突 | kb 已改 R2、实现留在取甲，kb 自己标了欠；R2 要三态槽读法 |
| K1 ① NoSelfVerified 一支 | 不存在 | 今天 0 命中，逐盘核该支并进甲的拒因 |
| K1 ② C393 (b) | 一致 | 逐句对上；「仍不成就拒」没有变异行 |
| K1 ③ F 算进 | 一致（kb 内部有旧表格行） | 实现与 checker 各一份；D16 生效表格行没有被抛弃根 |
| K1 ④ 丁-defer | 一致 | 限定词逐个对上；条款自己的缺口「没拿变异试过」仍开 |
| K1 ⑤ C379 B | 不判 | 只动 kb，没有代码 |
| K3 事实 | 与背景材料不同 | 种子今天复现不了；拒因是取甲的缺一槽，counter 0 |
| K3 出路 A（模型认拒） | 给了结局表 | 改 `left_by_the_lying_device`；永远只读保留 |
| K3 出路 B（R2） | 给了结局表（推） | 空白槽放开；剩每盘一槽读错；数引 kb |
| K4 切法 | 一致 | 起点与块长相同，虚机拆法的前提复核不了 |
| K4 缺一块 | 不会折 | 5954 万 + 4000 万条序列对精确口径不一致 0 |
| K4 新旧 | 旧是新的子集 | 4075 万条序列，旧折的新都折且条数相同 |
| K4 终止 | 没找到不终止的输入 | 长度 0 守卫承重（去掉即 oom），三处没有用例钉 |
| K5 窄一 | 一致，偏严一处 | 最新根按魔数认；对登记流误拒复核不了 |
| K5 白名单 | 能绕过（别名写法），今天无 | 别名下 `history.rs` 的引入者看不见；`tests/` 下共用模块不受表管 |
| K5 口径 | 数值一致，选项与 `none` 不去空白 | 都是响的拒 |
| K5 池几何 | 一致（9 + 1） | 抄了三个数不是两个；核对断言只在 harness 档 384 槽用例里跑 |

## 6　没做什么，与复核不了的

- 没跑 checker 档任何测试（窄一断言对登记流的放行、依赖用例、GPU）。窄一的「不误拒」只有主 agent 的一次实跑可引。
- 没起虚机：K4 里「来宾内核拆成同时在途的请求」「blklogwrites 按完成先后记日志」只读到注释。
- K3 出路 B 没实现，R2 一列是按 kb 的形式定义推的；没跑 512 段大档，没量两条出路对随机历史结局的改变。
- 没去二分是哪次提交让同一个种子生成另一段历史。
- K1 ⑤ 没判；K2 不归我。
- 3.4 里去掉守卫的那次命令退出码 250，是内存包装自己的结局（oom），输出不算结果；只用来证「去掉守卫就不终止」。
- 运行环境：内存包装头几次因同一个会话已有别的腿在跑退 252，之后设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与（峰值表只有一次样本时）`RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`；线程数 6，均经 `capped.sh 6`。冻结副本拷贝、`dbcc0d2f` 的 `git archive` 与编译目录在交回前删除，源与输出留在 `research/prompts/m3-leftover-code-r1-sonnet-model/`。
