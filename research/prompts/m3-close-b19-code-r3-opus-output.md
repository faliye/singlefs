# m3-close-b19-code-r3 攻方腿（Opus）报告（2026-10-30）

批 19 验证段代码轮第三轮（最终轮）。立场：攻前两轮站住的形态与第二轮判决的修补；新冒出的零轮形态只列。代码行号取开工快照（主工作区 `crates/` 的 9 份被判文件与 `research/prompts/m3-close-b19-code-r3-snapshot/crates-sha256.txt` 逐份对上，`sha256sum -c` 9 份 OK），kb 行号去 kb 文件现查。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-b19-code-r3-opus-model/`。复跑（本机内存包装排队排不上时经第二台；这一腿交回前就是这样跑的，退 0）：

```
cd <仓根> && nice -n 19 bash scripts/hosts/multi-host-run.sh --peer --memory-cap 4G --label <新标签> --fetch rerun-work/out -- bash research/prompts/m3-close-b19-code-r3-opus-model/tools/rerun.sh --repo . --frozen . --work rerun-work
```

本机能排上时：`bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-code-r3-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>`。冻结副本是含 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 的树（`mount.rs` 前缀 sha256 要是 `61275a92c211254c`，脚本第一行打出来核）。

交回前那一趟（第二台，标签 `m3-close-b19-code-r3-opus-final`）的原样汇总行：

```
name=tree mount_sha256=61275a92c211254c
SAME name=v1-member-slots lines=17
SAME name=v5-r1-opus-probe lines=6
SAME_AS_EARLIER_ROUND name=v5-r1-opus-probe earlier=r1-opus-probe-output-with-the-reorder.txt
SAME name=v5-r2-u2 lines=6
DIFF_FROM_EARLIER_ROUND name=v5-r2-u2 earlier=r2-sonnet-new-u2.txt changed_lines=2
SAME name=v5-r2-u6 lines=4
SAME_AS_EARLIER_ROUND name=v5-r2-u6 earlier=r2-sonnet-new-u6.txt
name=diffs_against_this_leg count=0
MULTI_HOST_RUN_STEP step=peer-command event=end elapsed_seconds=47.6 seconds=37.2 exit=0
```

（`DIFF_FROM_EARLIER_ROUND` 只打信息、不算失败：那一行正是第三轮修补要改的那一行，见 V5。）

| sha256 | 文件 |
|---|---|
| c3061d75458538992f67922b5d1034de7eddd6515b72e1e4d2323e042b1017a7 | earlier-rounds-out/r1-opus-probe-output-with-the-reorder.txt（原样拷自第一轮攻方模型 out/） |
| dbb03c28fa12c81ed2b1d1dfd157486160b0c2cbfa1d59ebcc0bf8fdea2658b9 | earlier-rounds-out/r2-sonnet-new-u2.txt（原样拷自第二轮辩方模型 out/new-u2.txt） |
| 54f53506c2748750d4441a5c91810ed3ba834398eca944a5e91122665e04682d | earlier-rounds-out/r2-sonnet-new-u6.txt（原样拷自第二轮辩方模型 out/new-u6.txt） |
| b474520af8f330c944a97010cfdabb1a35e7141724a58ba983203c7f779cb8d3 | out/v1-member-slots.txt |
| 38cff82f03ee49c6d4875193a5611087b859f3413ea487d01e392da6084ac8b7 | out/v5-r1-opus-probe.txt |
| aa227f8788733bf40c58c27918677cd2306c539861a7ba009e49deadcee1c67e | out/v5-r2-u2.txt |
| 54f53506c2748750d4441a5c91810ed3ba834398eca944a5e91122665e04682d | out/v5-r2-u6.txt |
| 24881305e3a8ce08ff09145874cbe5ab5ac384be5fd5a1dea04c4728fbbc0f51 | tests/attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration.rs（原样拷自第一轮攻方模型） |
| 7b5ebab71674da4a7e4594d4c054d06bf93c44c0964125f9a79155bf11c165a6 | tests/r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal.rs（原样拷自第二轮辩方模型） |
| 400e55d5efdbd383878f1bc3c30632f24b3c2cac8deca7551ff92973af86e791 | tests/r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times.rs（原样拷自第二轮辩方模型） |
| e5e574273a1861a5b5e9cbf0487690a4b440c70622edae72dc1a9ad1dc396ef2 | tests/r3_attack_probe_slots_of_member_devices_in_the_rehearsal_ledgers.rs（本腿探针） |
| 53a200b87f1589f7b259b740e7762462e9ab5e58bdf9fc4d803c8a8d31052892 | tools/rerun.sh（拷自 `scripts/agents/three-way-rerun-template.sh`，只改 rerun_steps 那一段） |

所有数都是副本上量的（第二台那一趟与本机草稿副本那一趟逐行相同），不算入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 剔读坏账的认法 | **打中（剔少）**；剔多没打中 | 认法按「盘号在补写盘表里」剔，补写盘之外的成员盘一槽撕裂（读得出、自证不过）或一槽每次读报错，只进内存那一趟取号照样读坏、读错，进账、排在最前，U2 那一形的拒因照旧被盖成系统配置槽（量过，6 格）。读错账不剔的理由只对补写盘的两槽成立 |
| V2 逐盘核在预演之前（多盘形） | **打中（补写盘与读坏盘不同那一形，与 V1 同一病根）**；「两种拒同时成立」那一判没打中，但射程只到续做预演 | 盘 1 补写、盘 0 一槽写 0、树表每次读坏：第三轮那条回归用例的断言（拒因是树表）红在这一形（量过）。逐盘核排在写行暖机预演与空间准入之后，「报逐盘核那一拒」只对续做预演成立（推的） |
| V3 C929 照今天不判 | **没打中** | 与 C874 五件闸、D22 那一行没有字面矛盾；第一轮落了之后崩溃那一态走同一支，C929 那句只写了「回卷之后」（列进 V4） |
| V4 未定清单 | **不全，补五条** | U2 修补改了用户定的岔路一/二「两本账同一判」却没列写回；`mount.rs:5638` 文档还写「两本账同一判」；逐盘核先后要写明只对续做预演；C929 一句要罩补写中途崩溃；读错账不剔的理由要缩到补写盘 |
| V5 前两轮的数 | **核得上（三个）** | 第一轮攻方五行（改法一那一份）逐行相同；第二轮 U6 的 420 次挂载第 366 次触发逐行相同；第二轮 U2 的拒因前后偏移与长度复现，今天的树上恰好那一行变成树表（第三轮修补生效） |

## V1　剔读坏账的改法撑不撑得住

### 机理（读码）

- 认法只看三样：盘号在补写盘表里、长度是一整槽、偏移是 0 或一个槽距。`crates/singlefs-core/src/mount.rs:5755`：「devices_rewritten_by_the_acquisition.contains(&read_failing_self_verification.device)」。判两本账时只剔读坏账、拿剔完的第一条：`crates/singlefs-core/src/mount.rs:5709`：「.find(|read_failing_self_verification| {」。读错账照旧取第一条、而且排在读坏账之前：`crates/singlefs-core/src/mount.rs:5725`：「(Some(read_failed_on_the_disk), _, true) => {」。
- 只进内存那一趟取号读盘走共用的那一份：`crates/singlefs-core/src/mount.rs:8078`：「ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal,」，`crates/singlefs-core/src/mount.rs:7806`：「ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal => {」。取号先重算号（`crates/singlefs-core/src/transaction.rs:1131`：「let recomputed = next_instance_generation_to_acquire(pool)」），再两次逐盘读两槽（`crates/singlefs-core/src/transaction.rs:933`：「system_configuration_slots_of_one_device_read_once(」）。每块盘两槽都读（`crates/singlefs-core/src/recovery.rs:4914`：「for offset in [0, slot_spacing_in_bytes] {」），读得出而自证不过就报读坏（`crates/singlefs-core/src/recovery.rs:4930`：「) => reader.note_that_the_bytes_read_fail_self_verification(」），共用的那一层记进读坏账（`crates/singlefs-core/src/mounted_session.rs:1177`：「self.reads_failing_self_verification.borrow_mut().push(」）；读错由同一层记进读错账（`crates/singlefs-core/src/mounted_session.rs:964`：「self.reads_failed_on_the_disk」）。
- 于是：任何一块盘上任何一个「读得出、自证不过」或「每次读都报错」的系统配置槽，只进内存那一趟取号必然读到它、每一趟都记进账、排在续做里任何一读之前。按盘号剔只剔掉补写盘那几条。
- 这类盘逐盘核放行：一槽自证过就走落后支，不看另一槽是坏还是读错（`crates/singlefs-core/src/mount.rs:6132`：「let Some(newest_self_verified) = newest_self_verified else {」；读错那一拒只在两槽一份自证过的都没有时判，`crates/singlefs-core/src/mount.rs:6135`：「if slots_reporting_a_read_error > 0 {」）。一槽坏在承诺里：`.claude/kb/decisions/22-单元原子性怎么合成.md:185`：「容错承诺是单故障：每盘一槽系统配置坏在承诺里」。

### 打中的反例（量过，副本；探针 `tests/r3_attack_probe_slots_of_member_devices_in_the_rehearsal_ledgers.rs`，产物 `out/v1-member-slots.txt`）

镜像是第三轮回归用例同一份窄盘镜像（`narrow_image`，续做中途推抬 F，取号之前在只进内存的盘上走一遍），故障与 U2 回归用例同形：只进内存那一趟头一次读的那份树表，从那一次起两块盘上两份每次读都读回坏字节（偏移 25067520、长 16384）。在它上面只多一处成员盘的坏槽：

| 盘面 | 不带树表故障 | 带树表故障的拒因 |
|---|---|---|
| 原镜像（对照） | 做成 | 树表：盘 0 偏移 25067520 长 16384 |
| 盘 0 槽 0 整槽写 0 | 做成，补写盘表空 | **盘 0 偏移 0 长 4096**（读坏） |
| 盘 0 槽 1 整槽写 0 | 做成，补写盘表空 | **盘 0 偏移 4096 长 4096**（读坏） |
| 盘 0 槽 0 每次读报块设备错 | 做成 | **读错：盘 0 偏移 0 长 4096** |
| 盘 0 槽 1 每次读报块设备错 | 做成 | **读错：盘 0 偏移 4096 长 4096** |
| 盘 1 两槽写 0（U2 回归用例那一形） | 做成，补写盘 [1] | 树表（第三轮修补生效） |
| 盘 1 两槽写 0 + 盘 0 槽 0 写 0 | 做成，补写盘 [1] | **盘 0 偏移 0 长 4096** |
| 盘 1 两槽写 0 + 盘 0 槽 1 写 0 | 做成，补写盘 [1] | **盘 0 偏移 4096 长 4096** |

原样产物行（`out/v1-member-slots.txt`，挑三行）：

```
PROBE name=member_device0_slot1_zeroed_with_the_tree_table_reading_back_garbage tree_table_offset=DeviceOffsetInBytes(25067520) result=Refused ReadBackFailingSelfVerificationWhileRehearsing device=DeviceIdentity(0) offset=DeviceOffsetInBytes(4096) length=4096 writes=0
PROBE name=member_device0_slot0_device_error_every_time_with_the_tree_table_reading_back_garbage tree_table_offset=DeviceOffsetInBytes(25067520) result=Refused ReadFailedWhileRehearsing device=DeviceIdentity(0) offset=DeviceOffsetInBytes(0) length=4096 writes=0
PROBE name=device1_both_zeroed_device0_slot0_zeroed_with_the_tree_table_reading_back_garbage tree_table_offset=DeviceOffsetInBytes(25067520) result=Refused ReadBackFailingSelfVerificationWhileRehearsing device=DeviceIdentity(0) offset=DeviceOffsetInBytes(0) length=4096 writes=0
```

（这个镜像的槽距是 4096，所以槽 1 在偏移 4096。）结局都是取号之前拒、一个写都没发，与对照相同；变的是拒因：点名一个本来就坏、重走治不好的成员盘系统配置槽，盖住每次读坏的树表。这正是第二轮判决 U2 判「采」的那一种毛病（拒因被盖），只是来源换成了补写盘之外的盘。

### 读错账不剔这一取舍

修补员的理由（`crates/singlefs-core/src/mount.rs:5705`：「读错账不剔：逐盘核只收两槽都读得出的盘，之后那两槽报读错是新的故障。」）对补写盘的两槽成立：两槽一份自证过的都没有、又有一槽报读错的盘，逐盘核当场拒（6135 那一判），进不了补写盘表，所以补写盘的两槽在预演里报读错一定是逐盘核之后才出的。没打中这一半。
但读错账本身还有一路逐盘核放行的旧故障：成员盘一槽每次读报错、另一槽自证过（6132 那一支放行，R2 也判得出，因为另一块盘两槽都自证过）。上表第 4、5 行量过：这一槽进读错账、读错账排在读坏账之前，拒因成了这一槽的读错。所以「读错账不剔」只在「读错账里只有补写盘的槽」这一前提下无害，这个前提不成立。

### 四句

1. 分不分辨臂：不分辨「今天的乙（按补写盘剔）」与「不剔」——两臂在这几格上都把拒因报成成员盘的槽（不剔那一臂：成员盘的槽照样最先进账，推的）。分辨的是「按盘号剔」与「按逐盘核那一刻已知坏的槽剔」（下面丙）。病根在两臂共用的前提：只进内存那一趟取号那几读走共用的那一份、读坏读错进两本账，重走治不好的旧故障也算「这一趟出过读错读坏」。
2. 系统当时看不看得到判别它的东西：看得到。逐盘核（7146 那一调用）在预演之前把每块盘两槽各读过一次（`system_configuration_slots_of_one_device_read_once`），哪一槽那一刻已坏、已报读错它读到过，只是今天只交回补写盘表、没留下槽级的结论。不是「判别子观测不到」。
3. 满足的是哪一句：第二轮判决 U2 的「用例：辩方那一形断言拒因点名树表，不点名补写盘的系统配置槽」——后半句字面没破（点名的是盘 0、不是补写盘），前半句「点名树表」破了；第三轮回归用例的模块说明 `crates/singlefs-harness/tests/writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device.rs:11`：「预演判两本账时不拿补写盘系统配置槽的读坏判，拒因点名真出故障的那一读。」，「拒因点名真出故障的那一读」破了。C874「运行时看得出」只罩补写那一支，成员盘那四格归不到它名下，归到 U2 判决那一句。代码自己的文档（5638「交最后那一趟最先的那一次读错，没有读错交最先的那一次读坏」）字面没破：成员盘的坏槽确实是最先那一次。
4. 改法在打中的格上还中不中：乙（今天的）还中，量过；第二轮的「不剔」还中，推的；下面丙、丁两个改法推的能修，都没实现、被攻过零轮。

### 能写成的会红用例与变异

- 会红用例（今天红，量过同形）：在 `writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device.rs` 那条 U2 用例旁加一条，镜像多一处「盘 0 一个系统配置槽整槽写 0」，其余照抄，断言拒因 `(offset, length_in_bytes) == (树表偏移, BYTES_OF_A_TREE_TABLE)`；今天红在这一条，交的是 `(DeviceOffsetInBytes(0), 4096)`。补写盘之外再加两条：只有盘 0 一槽写 0（不补写），只有盘 0 一槽每次读报错，同一断言，今天各红在拒因。
- 变异形态（改法落地之后才有意义）：判两本账时「已知坏槽一条都不剔」、「只剔读坏账不剔读错账」两行，各该把上面对应的那几条打红。

### 几个改法各修哪一格（只在我的模型上量过的只有「今天的乙」那一列；丙、丁被攻过零轮）

| 格 | 今天的乙（按补写盘剔读坏） | 不剔（第二轮之前） | 丙：剔逐盘核那一刻已坏、已报读错的那几槽，两本账都剔 | 丁：只进内存那一趟取号那几读改走穿到盘上（与真做取号写之前那几读同一路，不进账） |
|---|---|---|---|---|
| 补写盘两槽读坏（U2 那一形） | 修，量过 | 不修，第二轮量过 | 修，推的（补写盘两槽逐盘核时都坏） | 修，推的 |
| 成员盘一槽写 0 | 不修，量过 | 不修，推的 | 修，推的 | 修，推的 |
| 成员盘一槽每次读报错 | 不修，量过 | 不修，推的 | 修，推的 | 修，推的 |
| 逐盘核之后才坏的槽（真新故障） | 补写盘上的被剔（推的，见「剔多」）、成员盘上的照算 | 照算 | 照算，推的 | 不进账：真做取号写之前会再读盘，推的 |
| 成员盘根环槽、journal 环槽每次读报错 | 不修，推的 | 不修，推的 | 不修，推的（不是系统配置槽） | 只修取号那一段里读的（重算号读根环），推的 |

丙要逐盘核把槽级结论交出来（今天的 `DevicesCheckedBeforeTheAcquisition` 只有两张盘表）；丁动的是只进内存那一趟取号读的路，会让那一趟取号读不再替真做把字节记进共用的那一份（真做取号写之前那几读本来就穿到盘上，推的不碍事）。两样都改了用户定的岔路一「也记读坏」在系统配置槽上的判法，要连同 V4 第一条一起交用户。

### 剔多：没打中（攻了这几路）

- 偏移撞别的结构：认法按（盘、偏移 ∈ {0, 槽距}、长 4096）认。会报读坏的读只有三类：系统配置槽（`recovery.rs:4930`）、按位置条目读单元（`crates/singlefs-core/src/recovery.rs:710`：「reader.note_that_the_bytes_read_fail_self_verification(」）、释放之前核单元的两份（`crates/singlefs-core/src/transaction.rs:5239`：「reader.note_that_the_bytes_read_fail_self_verification(」）；后两类都在单元区里，偏移不会是 0 或一个槽距。根环槽读坏不报（`read_root_ring_slot` 自证不过那一支不调报读坏）。没造出撞上的。
- 补写盘上的真新故障：只进内存那一趟取号写过之后，补写盘两槽读的是内存里写下的字节（`DeviceKeepingWritesInMemory` 写过的块不交给底下那一层，也不往下报读坏，`mounted_session.rs` 785–814 那一段），读不坏；取号写之前那几读读到的就是逐盘核判过的那两个坏槽。补写盘两槽在「逐盘核之后、只进内存那一趟取号写之前」新坏（例如从读得出、自证不过变成别的坏字节）本来也是坏，剔掉不丢信息。没造出剔掉一条真新故障的盘面。
- 几何：槽距取 `pool.parameters.geometry`，与逐盘核、取号读同一个来源；物理块大于 4096 的几何下整槽读本来过不了对齐判据（报读错、不进读坏账），不归这一认法（推的，没造这种几何）。

## V2　逐盘核在预演之前，多盘形

### 补写盘与读坏盘不同：打中（与 V1 同一病根，不另算一格）

上表末两行就是这一形：盘 1 两槽写 0（补写盘）、盘 0 一槽写 0（成员盘、读得出自证不过）、树表每次读坏。逐盘核照判、把盘 1 交进补写盘表，不带树表故障时挂载做成、补写盘 [1]；带树表故障时拒因点名盘 0 的槽（量过，`out/v1-member-slots.txt` 里 `device1_both_zeroed_device0_slot0_zeroed_with_the_tree_table_reading_back_garbage` 那一行）。第三轮那条回归用例断言的是 `(offset, length) == (树表偏移, 16384)`（`crates/singlefs-harness/tests/writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device.rs:179`：「"拒因点名出故障的那份树表，不点名补写盘的系统配置槽：交的是盘 {:?} 偏移 {:?} 长 {}",」），这一形下它红。

### 逐盘核读盘的读错读坏不进两本账：没打中

两道独立地挡着：逐盘核在 `with_reads_straight_to_the_disk` 里跑，穿到盘上那一路直接读盘、不碰两本账（`crates/singlefs-core/src/mounted_session.rs:1015`：「ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),」），报读坏在这一路什么都不做（1170 那一臂）；预演每一趟开头还清两本账（`crates/singlefs-core/src/mount.rs:5682`：「reads_failing_self_verification.borrow_mut().clear();」）。修补那句「逐盘核只读盘、不写，排在前面不改盘上的结局」（`crates/singlefs-core/src/mount.rs:7144`：「逐盘核只读盘、不写，排在前面不改盘上的结局。」）也站得住：直读那一路不改共用的那一份，挪前挪后共用的那一份里记下的字节一样（推的，没造对照）。

### 两种拒同时成立报逐盘核那一拒：射程只到续做预演（推的，没造盘面）

逐盘核在 7146 调，排在它前面的还有三道取号之前的拒：要取的号到顶（`crates/singlefs-core/src/mount.rs:7089`：「let instance_to_acquire = next_instance_generation_to_acquire(&pool).map_err(」）、上一版树表 0 条时的空间准入（`crates/singlefs-core/src/mount.rs:7120`：「let admission_before_acquisition = admission_of_the_writable_mount_before_acquisition(」）、写行与暖机那一串的预演（`crates/singlefs-core/src/mount.rs:7131`：「} = publishes_after_acquisition_rehearsed(」）。这三道与逐盘核同时成立时报的是它们，不是逐盘核。第二轮判决定的「两种拒同时成立时报逐盘核那一拒」，量的是逐盘核与续做预演（C841）同时成立；写进 D22 那一行与 C874 的条文要写成「逐盘核排在续做预演之前、写行暖机预演与空间准入之后」，写成「逐盘核在预演之前」就是外推。今天没有盘面可量：窄盘镜像上写行暖机预演不拒；要造的话是「盘 1 整块写 0」加一个写行那次落点取不到的镜像。归 V4 第三条。

### 两块以上补写盘：没造（推的）

窄盘镜像只有两块盘，两块都两槽全废时读阶段一份系统配置都择不到，挂不上；仓里 `singlefs-harness/tests` 没找到三块盘、续做推抬 F 的现成镜像（`DeviceIdentity(2)` 只出现在三份与准入、盘表有关的用例里）。读码：认法按 `contains` 认整张补写盘表；取号读见证时补写盘表里的盘交空的一项（`crates/singlefs-core/src/transaction.rs:957`：「if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {」）；逐盘核逐盘追加、遇到第一块过不了闸的就整核交 `Err`。没看到只在两块以上补写盘时才出的分支。按 V1 的机理，两块补写盘之外只要还有一块盘带一个坏槽，同一种拒因被盖照样出。

## V3　C929 照今天不判：没打中

拿 C874 已定的五件闸、D22 那一行逐句对回卷之后那一态：

- D22 那一行的补写条款以「两槽全废」为前提：`.claude/kb/decisions/22-单元原子性怎么合成.md:185`：「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒」。回卷之后那块盘两槽都自证得过（`crates/singlefs-harness/tests/acquisition_rewriting_a_device_without_a_self_verified_system_configuration_rolls_back_on_a_write_failure.rs:188`：「盘 1 的回卷写世代号 2、落槽 0、带回卷之前的号 1，槽 1 留着补写那一写的号 2」），前提不成立，条款不管它；同一行「补写失败照取号每块盘回卷一次」今天照做。字面没有矛盾。
- 五件闸防的都是「这块盘的内容与所选那一版合不上」（缺单元、环里有更新的、单元区有后来实例写的、根槽缺、环槽读错）。闸在回卷之前那一次挂载判过；到下一次挂载之间要让这块盘的内容变，得有别的写（克隆那边挂过、别的进程写过），这对任何有自证槽的成员盘都一样，不是回卷独有——第二轮辩方的论证照样站得住。没造出只在回卷之后才有、普通成员盘没有的差别。
- 回卷写带旧号、第一轮那一槽带新号：下一次取号的 max 收新号，不撞号（188 那一行「下一次取号跳过 2、取 3」）。
- 攻了一路相邻的态：补写盘第一轮落了之后崩溃（进程没了，没有回卷）。盘 1 一槽带新号、自证过，另一槽仍坏；下一次挂载照样走落后支、不过补写闸（`mount.rs` 6132 那一支只看有没有一份自证过的）。结局与回卷之后同一支，不矛盾，但 C929 的题面只写了「回卷之后」：`.claude/kb/checks-owed.md:592`：「补写盘第一轮落了、第二轮报错回卷之后两槽都自证得过，下一次挂载把它当落后的盘走落后支、不再过补写闸」。C874 要的补写中途崩溃枚举流一落地，这一态就进枚举，条款那一句要罩它（列进 V4 第四条）。
- 第一轮那一写就在补写盘上报错：那块盘一写没落、照旧两槽全废，下一次挂载照旧过闸、被补写（同一份用例 235–236 行的说明），与 C929 无关。

会推翻「没打中」的观测：造出一块回卷之后的盘，下一次挂载走落后支放行，而同样内容、两槽由普通发布轮换写成的成员盘会被拒（或反过来）。

## V4　未定的清单完不完整：不全，补五条

正文列的七句（D16 槽 0 钉值与平局、D23 已定项 14 两条前提与隐含前提、C874 取法与字段名、C690 越过不扫三形、C928 解冻后、逐盘核与预演的先后、C929 一句）我没逐句复核写回措辞（归正推腿）；下面是这七句之外、前两轮改了行为或改了判法却没列进写回的：

1. **第三轮修补改了用户定的岔路一、二的判法，没列写回、也没标要交用户。** 岔路单已归档（`git show 59ce5a865^:research/prompts/m3-b44-code-r3-forks.md`），第 7 行「也记读坏」的定义是「取号之前那一判照读错账与读坏账两本判」，第 9 行「两样都做」状态「用户已定（2026-10-18 选这一条）」；第 16 行「维持偏拒」的定义是「两趟预演里出过读错、又没走到结局就在取号之前拒，被另一份兜住的读错也算」，第 17 行「没走完就重走」定的是「再没走完照「维持偏拒」」。第三轮修补让补写盘两槽的读坏不算「出过读坏」：预演没走到结局、账里只有补写盘两槽那几条时，修补之前取号之前拒，修补之后留给续做、真做（推的，没造出一个只靠非读类错停下的盘面）。这是把用户定的一条判据收窄。第 7 行那一条的翻面观测原文还点了这一形：「量出它在系统配置槽坏字节那一形上不让预演在取号之前拒，或在合法挂载上多拒的比例高到不可接受」。第二轮判决的「回看决策」只列 D22，「被攻过零轮的」只说 U2 改法进第三轮。要补：这一收窄进写回（落 D16 或 C874 哪一处由主 agent 定），连同 V1 的丙、丁一起交用户。
2. **`mount.rs` 自己的文档没跟着改。** `crates/singlefs-core/src/mount.rs:5638`：「交最后那一趟最先的那一次读错，没有读错交最先的那一次读坏（两本账同一判」（那一行接着写「岔路一「两样都做」那一行」）——修补之后读坏账剔、读错账不剔，「两本账同一判」不再成立；紧接着的 5639–5641 写了剔法，两句并排自相矛盾。写回时一起改。
3. **「逐盘核与预演的先后」要写明是哪一个预演。** 见 V2：逐盘核排在续做预演之前，排在号到顶（7089）、空间准入（7120）、写行暖机预演（7131）之后。条文写「逐盘核在预演之前」会把后三道也算进去。
4. **C929 那一句要罩补写中途崩溃。** 见 V3：第一轮落了之后崩溃那一态与回卷之后走同一支；今天 C929 题面只写回卷。写成「补写盘有了一份本池自证过的系统配置槽（回卷写的旧号，或第一轮落了之后崩溃留下的新号）之后，下一次挂载走落后支、不再过补写闸」，C874 的补写中途崩溃枚举流落地时有条可依。
5. **「读错账不剔」的理由与成员盘坏槽盖拒因，都没进任何清单。** 5705 那句理由只对补写盘两槽成立（V1）；成员盘一槽撕裂或读错盖住拒因是这一轮新冒出的形态（量过），按「第三轮之后停」只列、标零轮，交判决的交用户表。

会推翻第 1 条的观测：岔路单在 2026-10-18 之后另有一次用户定案改过「两本账两本判」的口径（我只读了归档的那一份）。

## V5　前两轮的数能不能核：挑三个，都核得上

都在开工快照那份代码（`mount.rs` 前缀 `61275a92c211254c`）的副本上、经第二台跑；前两轮的探针源码原样拷进本腿 `tests/`，前两轮存下的产物原样拷进 `earlier-rounds-out/`，比法见 `tools/rerun.sh`（两边按行排序、取 PROBE 起的那一截）。

| 前两轮的数 | 出处 | 今天的树上 | 判 |
|---|---|---|---|
| 第一轮攻方探针五行（改法一那一份：逐盘核挪到预演之前之后，盘 1 两槽写 0 挂得上、补写盘 [1]） | 第一轮攻方模型 `out/probe-output-with-the-reorder.txt` | 五行逐行相同（`SAME_AS_EARLIER_ROUND name=v5-r1-opus-probe`） | 核得上 |
| 第一轮判决引的 `name=probe_output_identical lines=5`（改法一之前那一份） | 同上 `out/probe-output.txt` | 今天的树已带改法一，那五行是旧次序的结局，这棵树上本来就不出；没还原旧次序重跑 | 核不了（没做） |
| 第二轮 U6：420 次挂载、清扫第一次在第 366 次、之后行数 2、全程不超过一片 | 第二轮辩方模型 `out/new-u6.txt` | 逐行相同（`SAME_AS_EARLIER_ROUND name=v5-r2-u6`） | 核得上 |
| 第二轮 U2：拒因修补前点名盘 1 槽 0（偏移 0、长 4096） | 第二轮辩方模型 `out/new-u2.txt` | 只差一行，正是修补要改的那一行：今天点名盘 0 偏移 25067520、长 16384（树表）；另外五行相同 | 核得上；修补生效 |
| 第二轮辩方其余 11 步（old-u1、new-u1、new-u4、new-u5、变异三步等） | 第二轮辩方模型 `out/` | 没跑 | 没核 |

U2 那一行的原样差（`DIFF_FROM_EARLIER_ROUND name=v5-r2-u2 … changed_lines=2`）：

```
-test u2_zeroed_slots_and_a_tree_table_reading_back_garbage_every_time_names_which_read ... PROBE name=zeroed_garbage_every_time result=ReadBackFailingSelfVerification device=DeviceIdentity(1) offset=DeviceOffsetInBytes(0) length=4096
+test u2_zeroed_slots_and_a_tree_table_reading_back_garbage_every_time_names_which_read ... PROBE name=zeroed_garbage_every_time result=ReadBackFailingSelfVerification device=DeviceIdentity(0) offset=DeviceOffsetInBytes(25067520) length=16384
```

U6 的原样行（`out/v5-r2-u6.txt`）：

```
PROBE name=u6_rows_deleted mount_number=366 deleted=364 rows_after_the_mount=2
PROBE name=u6_summary mounts=420 not_swept=419 rows_deleted=1 no_row_deleted=0 other=0 max_rows_at_a_not_swept_decision=365 max_rows_after_a_mount=365 mounts_with_rows_over_one_page=0
```

攻这几个数本身：U6 的「全程不超过一片」只在这一个池、每次挂载写一行的历史上量（第二轮判决已写「仅 12 槽环一个池」）；420 次里清扫只做成一次，「之后行数 2」之后再没到过阈值，所以第二次清扫的行为这份产物里没有取样点（推的，没另扫）。U2 的偏移 25067520 是这份窄盘镜像上那份树表的落点，换镜像就变；它支撑的是「拒因点名树表」这一定性，不是这个数本身。

## 这一轮新冒出的零轮形态（只列，不要求实现）

| 形态 | 状态 |
|---|---|
| 成员盘一个系统配置槽读得出自证不过，或每次读报错，只进内存那一趟取号读到它、记进两本账，盖住真出故障那一读的拒因（V1、V2） | 量过 6 格，副本；零轮 |
| 同一机理推到成员盘的根环槽、journal 环槽每次读报错（只进内存那一趟重算号读根环） | 推的，没造 |
| 同一机理在「只靠非读类错停下」的预演上把结局从「留给续做」翻成「取号之前拒」（多拒） | 推的；窄盘镜像上没有只靠非读类错停下的形，没造出来 |
| 改法丙（剔逐盘核那一刻已坏、已报读错的那几槽，两本账都剔）、改法丁（只进内存那一趟取号那几读改走穿到盘上、不进账） | 只在我的模型上推过、没实现、被攻过零轮；都收窄用户定的岔路一、二，要交用户 |

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| V1 剔多 | 认法的（盘、偏移、长）撞上单元读、释放核验读；补写盘两槽在逐盘核之后新坏；几何换槽距、物理块 | 读码；会报读坏的三类读逐一对过偏移来源；没造盘面 |
| V1 读错账（补写盘那两槽） | 补写盘两槽在预演里报读错是不是旧故障 | 读码（6135 那一判）；成立 |
| V2 直读不进账 | 逐盘核那几读进不进两本账、挪前挪后改不改共用的那一份 | 读码两道（1015 那一路、5681–5682 清账） |
| V2 两块以上补写盘 | 认法、取号见证、逐盘核循环在两块以上时有没有专属分支 | 读码；窄盘镜像只两块盘，没造 |
| V3 | 回卷之后与普通成员盘的差；回卷后撞号；第一轮在补写盘上就报错；第一轮落了之后崩溃 | 读码与用例说明（188、235–236 行）；没跑新盘面 |
| V5 | 三个数逐行复现 | 一个镜像、一个池；U6 只一条历史 |

成员盘坏槽那一机理的取样：一个镜像（窄盘、两块盘、槽距 4096）、盘 0 的两个槽各一次、两种坏法（整槽写 0、每次读报错）、加不加盘 1 补写共 6 格带树表故障、6 格不带；没扫盘 1 上的成员槽（盘 1 是读坏账里树表那一份的第二份所在，推的结局同形）、没扫别的镜像与几何。

## 这条腿自己的限度

- 打中那一格的结局（取号之前拒）与对照相同，变的只是拒因；要两处故障（成员盘一槽坏，加树表两份每次读坏），与第二轮 U2 那一形同一个故障数。多拒那一种（结局翻面）只推了、没造出来。
- 丙、丁两个改法只推、没实现，表里「修」全是推的；它们都动用户定的岔路一、二，判不判要交用户。
- V4 第 1 条依据的岔路单是归档版，只读了那一份；没去 records 里逐条找 2026-10-18 之后有没有再改口径（`grep -rn 两本账 .claude/kb records` 只命中两处，都不是改口径的）。
- 正文 V4 列的七句的写回措辞没逐句核（归正推腿）。
- 所有数都是副本上量的，不算入库装置上的数；入库形态是 V1「会红用例」那三条。
- 冻结副本取的是主工作区此刻的 `crates/`：9 份被判文件与快照清单 sha256 逐份对上；别的 crates 文件快照没记 sha256，没法核是不是开工那一刻的样子（`rerun.sh` 第一行打 `mount.rs` 前缀给复跑的人核）。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；没跑重型测试、没起 checker 档的 cargo、没碰 GPU。
- 本机内存包装排队（heavy slice 被别的会话的 checker 档门禁占满）：本机草稿副本上 V1 探针与第一轮探针跑成了（`capped.sh 31`、上限 4G）；第二轮 U2 探针在本机排队 300 多秒，我停了自己那条链（`proc.py stop` 两个进程，排队那一步退 143、命令一行没跑），改经 `scripts/hosts/multi-host-run.sh --peer --memory-cap 4G` 在第二台跑整份 `rerun.sh` 两趟（先出基线、再整份复跑退 0）。
- 清理：删了草稿目录里的冻结副本 `frozen/`（44M）与编译目录连同副本 `run1/`（2.5G），删了 `multi-host-run` 取回的两个目录（68K、64K）。草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-code-r3-opus/` 只剩几份日志与 `rerun.sh` 的草稿（44K），没有仓副本与编译目录。

## 要主 agent 跑的命令

无（这一腿没改仓里的代码）。要复核模型时照开头那条复跑命令。
