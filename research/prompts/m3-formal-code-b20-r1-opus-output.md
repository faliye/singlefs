# 第 20 批代码轮第一轮 攻方（Opus）报告（2026-10-04）

攻击面：K1（F1、F2、F5 的用例与点名的接错；靶向二次崩溃摆法；薄壳改造）、K2（发布臂的判法与写入口的对应；nlink = 0 那一支直接建 inode；固定历史用例的态）。
代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b20-r1/tree/`（开工时按 `research/prompts/m3-formal-code-b20-r1-snapshot/crates-sha256.txt` 285 行逐个核过，0 行对不上），路径写那棵树里的。
量的都在我自己拷出的副本上（`/tmp/claude-1000/m3-formal-code-b20-r1-opus/repo/`），只跑 harness 档：checker 档的胶水（调用点、计数、取条件的那段字符串处理）照 `crash_injection.rs` 抄进 harness 档探针，checker 档的用例与 6 行 checker 档变异一行都没跑，凡是说到「checker 档用例红 / 绿」的都是推的。副本上的数要主 agent 在入库装置上重做才能引。

## 复跑

```
REPO=/home/fy5090/code/singlefs FROZEN=/tmp/claude-1000/m3-formal-code-b20-r1/tree THREADS=10 CAP=2G \
  bash research/prompts/m3-formal-code-b20-r1-opus-model/tools/rerun.sh
```

工作目录：仓根。脚本拷冻结副本与仓根的 `Cargo.toml`、`Cargo.lock` 到 `WORK`（缺省新建 `/tmp/claude-1000/b20-opus-rerun.XXXXXX`），把 `probes/` 下三份探针放进副本的 `crates/singlefs-harness/tests/`，release 编译，经 `scripts/hosts/run-with-memory-cap.sh` 与 `scripts/hosts/capped.sh` 顺序跑三个测试目标（我这边分片并行跑过，合起来约 6 分钟；顺序跑估 8 到 10 分钟，推的）。跑完看脚本末尾打出的 F1 / F2 / K2 fixed / F5 行、SUMMARY 与 ARM 计数、三档盘宽的 TALLY。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
af39f3b502fcedc742003306933ca3c8336f00e78b0a2e7eeeac3a0b0ac583cd  ./outputs/k1-run3.log
9e4d373c7fffb030ab1798612fde4c191dc75d92d0912b567bd1afe73827ef24  ./outputs/k2a-final.log
5545a2626837c2d71c4902a25e03b8a8fe5ece89d83f7c90d1456c68673ffc31  ./outputs/k2b-bndwall-240.log
fd80bff57d0890ccfe421e4699a93a2b54d3e66dcd3d014dd418c5d4ebb68612  ./outputs/k2b-bndwall-256.log
49cae083dda7ef926de8ce9e36570d5b7f2a9518d9fed461e5aceed932ac7acf  ./outputs/k2b-bndwall-384.log
40a96f136e7e1cc12d967aa33c34877b04bb4af2da547af02b83cbcaf8c4f009  ./outputs/k2b-est-384.log
a922c0b549c03a07793bf50d9a022a85d810351722a2108994f6e247c208fe85  ./outputs/k2b-est4g.log
a5ed899fdca2dd93cab05c6b15681e7920c22851379a4941ffb794cdf9884911  ./outputs/k2b-map-240.log
899d2b1c0e84123ce45bdd2445ec0c1800a2fb6aa42c881529d5b4a964151694  ./outputs/k2b-map-256.log
be26fc61e38a8d39f59e2c395f83a587395cac712c7c71c79a4251f9e965d570  ./outputs/k2b-map-384.log
df515db0c8eaf3098f64e4123ee56e3710c78adf76f33bf1098e96f96755e08c  ./probes/b20_opus_k1_targeted_states.rs
9585bf7be32829b47b426345ca6479c649757e21f715b48745ea7e1acdbf03f4  ./probes/b20_opus_k2_histories_on_narrow_widths.rs
0f2a162c78fd8041476f1317cb021cbd01514974edd310a54da38c99cc8770d0  ./probes/b20_opus_k2_short_continuation.rs
a5147f61f1af1f2a45479e2eacf7ac15bd4325f667ca12bbd598a7c829b2301b  ./tools/rerun.sh
```

`outputs/` 里的日志是我这一次跑出来的原样：`k1-run3.log`（K1 三条靶向态与 F5 射程，四个测试一次跑）、`k2a-final.log`（短续做形状扫描，三片合并）、`k2b-map-*.log`（三档窄盘、偏删除比重、段边界态，带写入口对应）、`k2b-bndwall-*.log`（同上但偏单元区墙比重；这三份是加对应核对之前那一版探针跑的，只有分支计数）、`k2b-est-384.log`（层 0 全域状态数估算）、`k2b-est4g.log`（4 GiB 估时）。

## 各格判定一览

| 格 | 判定 | 依据 | 量过 / 推的 |
|---|---|---|---|
| K1-F1 第二截两条断言 | 站住：正确接线两条都成立，「所选那一版交 None」两条都不成立 | 一·1 | harness 档复做量过；checker 档用例本身推的 |
| K1-F2 第三截靶向用例 | 站住：正确接线 (1, 1)；三种点名接错 (1, 0) / (0, 0) / (0, 0)；未点名的「所选那一版交 None」也是 (1, 0) | 一·2 | 同上 |
| K1-F5 纯函数单测 | 站住（对点名的那一种接错红）；旁注：今天的历史里读故障那一支走不到，见一·3 | 一·3 | 单测推的；射程量过（0 / 122） |
| K1 靶向摆法摆的态 | 站住：摆出来的就是文档说的那个态 | 一·4 | 量过 |
| K1 薄壳改造 | 站住：原入口的种子、摆法、计数没变 | 一·5 | 推的（按改动全文逐段核） |
| K2-a 判法与写入口一一对应 | 站住 | 二·1 | 量过（15948 个段边界态 + 250 个短续做态 + 固定态） |
| K2-b nlink = 0 那一支直接建 inode | 没打中假发现、没打中漏报；**打中一处判别力缺口（H2）**：判法里 nlink 那一半没有任何用例走到 | 二·2 | 结局量过（250 态）；用例红绿推的 |
| K2-c 固定历史用例的态 | 态与名字相符；**打中一处判别力缺口（H1）**：用例不核「建出了 inode」 | 二·3 | 量过（态、两条臂、建 0 个那一臂）；用例红绿推的 |

H1、H2 都是「用例分不出两种实现」的缺口，不是实现错。四句见三节。

## 一、K1

### 一·1 F1：第二截那两条断言

复做的态与用例同形：第一个文件之后一次覆盖写（`UnitAreaOf384Slots`、不逐步跑 checker、准入判着），覆盖写那次发布只落两槽系统配置轮换。只读恢复落在 (实例 1, txg 3)。在这份盘上用崩溃注入第二截那一种调法挂一次（被拒），再把拒绝交给 harness 档判官 `judge_writable_mount_refusal_of_a_crash_state`，所选那一版分别交 `recover` 的 `effective_root` 与 None；判官的结论照 `crash_injection.rs` 第 2289 行那句格式拼成实现那一半，再照用例的取法（`split_once("拒得不合法（")` 再 `split_once("）：")`）取出条件、照用例的两条 `contains` 判。原样输出（`outputs/k1-run3.log`）：

```
F1 正确接线 report.effective_root: verdict=Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterAboveTheLastRecordOfTheSelectedVersion)] }
   parsed=Some("[WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterAboveTheLastRecordOfTheSelectedVersion)]") 断言一(含 C670)=true 断言二(不含复算为假)=true
F1 变异 None: verdict=Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [CriterionFalseWhenRecomputedOnTheCrashImage] }
   parsed=Some("[CriterionFalseWhenRecomputedOnTheCrashImage]") 断言一(含 C670)=false 断言二(不含复算为假)=false
```

- 正确接线是 `crates/singlefs-checker-tier/src/crash_injection.rs:1384`：「version_landed_on_after_the_crash: report.effective_root,」；变异把它换成 None，判官那一支在 `mount_transparency.rs` 里先交回没有所选那一版（`crates/singlefs-harness/src/mount_transparency.rs:1457`：「let selected_version = selected_version?;」），C670 那一条不进，复算为假那一条顶上来。两条断言（`crates/singlefs-checker-tier/src/crash_injection.rs:5761`：「conditions_not_met.contains(&witness_criterion_true_without_read_faults),」，`crates/singlefs-checker-tier/src/crash_injection.rs:5765`：「!conditions_not_met.contains(&criterion_false_when_recomputed),」）在变异下各自都红，不是只红一条。
- 用例取条件要第二截恰好一条新发现：已知红清单是空的（`crates/singlefs-harness/src/history.rs:2038`：「pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];」），拒得不合法那一条一定进新发现。两个变体名互不为子串，`contains` 不会误中。
- 没量的：checker 档用例那一整条链（`inject_crashes_into_history` → 计数 → `render`）没跑；正确接线下它绿是推的。

### 一·2 F2：第三截靶向用例与点名的三种接错

复做：第一次崩溃摆在覆盖写那次发布的第一个写上（盘面就是覆盖写之前），在这份盘上带录制流可写挂载一次，挂载写表 38 个写；照用例的摆法（`crates/singlefs-checker-tier/src/crash_injection.rs:6032`：「writes_from_the_segment_on: last_rotation + 1」，扣下 `crates/singlefs-checker-tier/src/crash_injection.rs:6035`：「root_of_the_row_publish - first_write_of_the_segment_of_the_root,」）造二次崩溃镜像，在它上面恢复、再挂、交判官，计数照第三截那段取舍（N-配置 判过记一次，判官列出 C670 那一条再记一次不合法）。原样输出：

```
F2 mount writes=38 root=18 rotation=[19, 20] root segment=3 [18] next segment Some([19, 20]) kinds in window [RootRecordFua, SystemConfigurationSlot, SystemConfigurationSlot]
F2 second crash effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) (first crash Some((InstanceGeneration(1), CheckpointTxg(3))))
F2 正确接线: (判过, 不合法)=(1, 1) verdict=Some(Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterAboveTheLastRecordOfTheSelectedVersion)] })
F2 变异 读故障恒交 Injected: (判过, 不合法)=(1, 0) verdict=Some(Legal(NewerPublishWitnessedBySystemConfiguration))
F2 变异 再挂的盘交第一次崩溃镜像: (判过, 不合法)=(0, 0) verdict=None
F2 变异 再挂的盘交 second_base: (判过, 不合法)=(0, 0) verdict=None
F2 未点名 所选那一版交 None: (判过, 不合法)=(1, 0) verdict=Some(Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [CriterionFalseWhenRecomputedOnTheCrashImage] })
```

用例的末条断言要 (1, 1)：正确接线下取到 (1, 1)，三种点名的接错都不是 (1, 1)，在第三截调用点把所选那一版交成 None（F1 在第三截的同形，没登记变异）也不是。verdict=None 那两行是再挂做成了、没有拒绝可判。

### 一·3 F5：纯函数单测，与它在今天的历史里的射程

单测对点名的那一种接错（交写表下标、不交录制流下标）一定红：`crates/singlefs-checker-tier/src/crash_injection.rs:5498`：「stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 2, 3),」 期望 6，变异交 4（推的，纯函数，按代码算）。

射程：读故障的认定只在「崩溃恢复抛弃根」那一步往录制流里发过东西时才交 `Injected`（`crates/singlefs-checker-tier/src/crash_injection.rs:1069`：「HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,」，第 1019 行的文档注释也写了那一步常态下一个都不发）。我数了随机历史里这一步发没发：三组比重（BROAD、DELETING_FILES_OF_SEVERAL_LOWER_LEAVES、ROLLBACK_AFTER_RAISING_THE_FLOOR）× 两档窄盘 × 种子 0–79、每段 24 步，原样输出：

```
F5 abandoning-recovery steps observed=122 steps_that_sent_something=0
```

所以在这批历史上，第二截（`crates/singlefs-checker-tier/src/crash_injection.rs:1390`：「read_faults: read_faults_in_the_crash_state,」）与第三截（`crates/singlefs-checker-tier/src/crash_injection.rs:3011`：「read_faults: read_faults_in_the_first_crash_state,」）拿到的认定恒是 NoneInjected。推出来的两件事：F5 那一处修法今天改变不了任何一个崩溃态的判定；第三截调用点写死成 `NoneInjected` 这一种没登记的接错，没有用例会红，今天的历史上也判不出（与它对称的「恒交 Injected」有变异行 `crates/mutations.tsv:2363`）。这一条不算打中 K1（F5 的单测对它点名的接错是红的），记在这里给主 agent 判要不要另立一笔账：C670 读故障那一支的判别力，要等有一种历史步在读故障下真的往盘上发写才量得到。

### 一·4 靶向摆法摆的态

`exactly_one_second_crash_point` 的持久集合从段首起数、崩在哪一段取第一个被扣下的写（`crates/singlefs-checker-tier/src/crash_injection.rs:2780`：「phase: phase_of_each_write[first_write_of_the_segment + first_withheld],」）。一·2 的输出里挂载写表第 18 个写（写行的根槽 FUA）单独成一段，第 19、20 个（两槽轮换）是下一段；用例的窗口是第 18–20 个、扣下第 18 个。于是二次崩溃镜像 = 第一次崩溃镜像 + 挂载写表第 0–17 个（取号、写行的单元与记录）+ 两槽轮换，根槽没落、之后一个都没落；扣下的第一个写是写行的根槽，它在 `writable_mount_phase_of_each_write` 里落在写行（数根槽之前判段，第一个根槽之前都是写行）。与 `SecondCrashPointDraw::ExactlyOneSecondCrashState` 的文档逐句相符。第三截按窗口长度切持久集合（`persisted_within_the_segment.len()`），越过段尾的窗口照样逐条对上，读码核过。

### 一·5 薄壳改造

改动全文里 `inject_crashes_into_history` 的函数体只有四处动了：签名拆成薄壳（`crates/singlefs-checker-tier/src/crash_injection.rs:1100`：「SecondCrashPointDraw::OneInEachMountPhaseBySeed,」）；F5 那一处把原来的下标表达式换成同值的纯函数（`writes_in_the_segment` 为 0 时原表达式在 `overflow-checks = true` 下同样 panic）；`SecondCrashInputs` 多一个字段；第三截的 `match` 在按种子那一支原样调原来那一句（`crates/singlefs-checker-tier/src/crash_injection.rs:2910`：「draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write)」）。种子、摆法、计数都没变（推的，按改动全文逐段核；没有跑两版对比，那要跑 checker 档）。

## 二、K2

### 二·1 判法与写入口一一对应

判法（`crates/singlefs-checker-tier/src/crash_injection.rs:1961`：「fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {」，`crates/singlefs-checker-tier/src/crash_injection.rs:1965`：「.is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)」）与写入口（`crates/singlefs-core/src/transaction.rs:6069`：「let Some(record) = previous.inode_record else {」 交 `WriteToAnInodeThatIsNotInTheTree`，`crates/singlefs-core/src/transaction.rs:6074`：「if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」 交 `WriteToAnInodeBeingDeleted`）读的是同一个字段、同两个条件；覆盖写进门第一句就调它（`crates/singlefs-core/src/transaction.rs:5679`：「let first_file_record = live_record_of_the_first_file(previous)?;」）。从盘上重建的一版，这个字段取 inode 1 那条记录、不看 nlink（`crates/singlefs-core/src/recovery.rs:2685`：「.find(|inode_record| inode_record.inode == FIRST_INODE_NUMBER)」），所以挂载交回的现行那一版上两边看到的是同一个值。覆盖写别的拒法（内容装不进一个数据单元、txg 到顶）在这里走不到：发布臂的内容 97–160 字节。

量：三档窄盘（384 / 256 / 240 槽）、偏删除比重、种子 0–59、每段 24 步，每个段边界态上挂一次、记判法的值，再从同一份镜像现挂一次、直接调 `publish_overwrite` 看写入口说什么（`outputs/k2b-map-*.log`，三档合计 15948 个态、546 次 unlink）。三档相加（命令：`grep -h TALLY k2b-map-*.log | grep mapping`，按键相加）：

```
2960 mapping live=true / overwrite published
285 mapping live=true / overwrite refused SpaceAdmissionRefused
7404 mapping live=false / overwrite refused WriteToAnInodeThatIsNotInTheTree
```

再加二·2 的 250 个短续做态（判法为假、写入口全部交 `WriteToAnInodeBeingDeleted`）与二·3 的固定态。判法为真时写入口一次都没交那两种拒，判法为假时写入口每次都交其中之一：一一对应，站住。

### 二·2 nlink = 0 那一支直接建 inode

**这一支什么时候走得到。** 可写挂载在交回之前把队列走空（`crates/singlefs-core/src/mount.rs:4924`：「if current_version_of_trees.deleted_inodes.is_empty() {」 之后那一圈），只有推满抬 F 仍放不下（`crates/singlefs-core/src/mount.rs:4995`：「DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {」）时挂载交回的那一版里第一个文件还是 nlink = 0。历史里只 unlink 第一个文件、只在它删完之后建 inode，所以「第一个文件活着、队列里有别的号」这一形造不出来。

**量 A：造出来的短续做盘面。** 盘面照 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:322`：「fn pool_crashed_between_deletion_batches(」 的造法（不补环窗口的窄盘、造盘时关准入、第一个文件长到几个单元、直接发意图与 0 或 1 批再崩），再用崩溃注入第二截那一种调法挂（`crates/singlefs-checker-tier/src/crash_injection.rs:1796`：「mount_writable_with_space_admission(parameters, &mut devices, execution.space_admission);」，产品删除开关）。取样网格：单元区 100–320 槽每 2 槽 × 覆盖写 {0,1,2,3,4,6,8} 次 × 文件 {1,2,6,14,30,60} 个单元 × 意图之后 {0,1} 批，造得出的 6604 个；挂载被拒 241、续做走空 6113、推满仍放不下 250。250 个全是队列 [1]、inode 1 的 nlink = 0、挂载之后池级 checker 全绿。每个形状上各从同一份崩溃镜像现挂一次跑五条臂，臂之后的盘跑池级 checker。一个形状的原样输出与五条臂的计数（`outputs/k2a-final.log`，`grep -cF` 数的）：

```
SHAPE Shape { unit_area_slots: 128, overwrites: 3, data_units: 14, batches_after_the_intent: 1 } MOUNT short queue=[InodeNumber(1)] inode1_nlink=Some(0) live=false checker_after_mount=[]
  ARM TwentyOne: 拒 Explained(UnitAreaWall) / SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [DeviceShortOfDemand  | checker []
  ARM BeforeTwentyOne: 拒 Unexplained / WriteToAnInodeBeingDeleted { inode: 1 } | checker []
  ARM SessionNewInode: 拒 NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingTheFloor { last_refusal: SpaceAdmissionRefu；盘动没动 false | checker []
  ARM SessionOverwrite: 拒 NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingTheFloor { last_refusal: SpaceAdmissionRefu；盘动没动 false | checker []
  ARM SessionUnlink: 拒 NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingTheFloor { last_refusal: SpaceAdmissionRefu；盘动没动 false | checker []
MOUNT short queue=[InodeNumber(1)] inode1_nlink=Some(0) live=false checker_after_mount=[]	250
ARM TwentyOne: 拒 Explained(UnitAreaWall) / SpaceAdmissionRefused(	250
ARM BeforeTwentyOne: 拒 Unexplained / WriteToAnInodeBeingDeleted { inode: 1 } | checker []	250
ARM SessionNewInode: 拒 NoSpaceAfterRaisingTheFloor(	250
ARM SessionOverwrite: 拒 NoSpaceAfterRaisingTheFloor(	250
ARM SessionUnlink: 拒 NoSpaceAfterRaisingTheFloor(	250
盘动没动 false | checker []	750
| checker []	1250
```

臂的意思：TwentyOne 是 20-1 的发布臂（这一支直接 `publish_new_inodes(…, 1, …)`）；BeforeTwentyOne 是 20-1 之前「带文件一律覆盖写」；三条 Session 是挂载之后经会话做一次用户改动（D8（核心索引结构） 已定项 16 第 6 条要先走空的那一种，`.claude/kb/decisions/08-核心索引结构.md` 第 469 行；会话那一处是 `crates/singlefs-core/src/mounted_session.rs:392`：「if !current_version_of_trees.deleted_inodes.is_empty() {」）。用户动作放开扫了三种（建 inode、覆盖写、unlink），前缀（形状）与故障（崩在意图之后）固定。

结局：20-1 的臂 250 / 250 被空间准入拒，映射成单元区墙（`crates/singlefs-harness/src/model_comparison.rs:673`：「PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),」），第二截只计数（`crates/singlefs-checker-tier/src/crash_injection.rs:2307`：「tally.publishes_after_the_writable_mount_refused_at_the_unit_area_wall += 1;」），不报；会话三种改动 250 / 250 报空间不够、盘一个字节都没动。两边都是「这一版上这次改动做不成、盘不动」，池级 checker 全绿。20-1 之前那一臂 250 / 250 报 `WriteToAnInodeBeingDeleted`，映射成 Unexplained（`crates/singlefs-harness/src/model_comparison.rs:714`：「| PublishError::WriteToAnInodeBeingDeleted { .. }」），在第二截是一条假的新发现——20-1 把这一格修掉了。

在这 250 个态上，直接建 inode 与「会话先走空」既没让第二截多报（假发现），也没少看一段写（会话那一路一个写都没发，没有要 checker 去看的盘面）：**没打中**。什么现象会推翻它：某个短续做态上式子放行了建 inode（20-1 的臂发成、造出「队列里有号、又建了新 inode」这一版，用户经会话造不出来），或会话那一路走空推得出空间、写了盘——这两样在 250 个态上都是 0。

**量 B：崩溃注入真用的盘面上走不走得到。** 这一支在随机历史的崩溃态上我一次都没走到，但量的不是层 0 的全域，限度如下：
- 层 0 全域（每段的每个写子集）按 `closed_form_state_count` 算，384 槽、偏删除比重、24 步的 20 段历史每段 1.09×10⁶ 到 2.8×10¹⁴ 个状态（`outputs/k2b-est-384.log`），一段都低不到约 10⁶ 的上限，按约束一段都没跑。
- 退而只取每段的段边界态（之前的段整段持久、这一段一个都没落，外加全部持久那一个；**这不是层 0 的枚举域**，只用来看「挂载之后落在哪一支」）：三档窄盘 × 偏删除比重 180 段历史 15948 个态（546 次 unlink），三档窄盘 × 偏单元区墙比重 180 段 17111 个态（123 次 unlink）。nlink = 0 那一支 0 次，续做推满仍放不下 0 次，挂载被拒 0 次（`outputs/k2b-map-*.log`、`outputs/k2b-bndwall-*.log` 的 TALLY 行）。
- 4 GiB（快档战役的盘宽，`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:59`：「device_width: HistoryDeviceWidth::FourGibibytes,」）：3 段 BROAD 历史的段边界态跑了 193 秒、一次 unlink 都没有（`outputs/k2b-est4g.log`），估一段约 64 秒，没再跑。

所以今天这一支只在造出来的窄盘上量得到；随机历史的崩溃态上走不走得到，量过的那部分是 0，全域没量。

### 二·3 固定历史用例的态

用例（`crates/singlefs-checker-tier/src/crash_injection.rs:4623`：「fn the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published(」）的历史是 `crates/singlefs-checker-tier/src/crash_injection.rs:4634`：「operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode],」，摆法是从建 inode 那次根槽所在段的段首起数到根槽之后一个写、扣下那一个。复做的原样输出：

```
K2 fixed root=124 root segment=26 [124] next Some([125, 126]) kinds after root [RootRecordFua, SystemConfigurationSlot, SystemConfigurationSlot] writes total 127
K2 fixed state == 写表前缀到根槽为止：true; effective_root=Some((InstanceGeneration(1), CheckpointTxg(6)))
K2 fixed arm 20-1: current inode_record=None deleted_inodes=[] trees_and_no_file=false
K2 fixed arm 20-1: 发成之后 inode 号 [(2, 1), (3, 1)]
K2 fixed arm 20-1: 发成 | checker []
K2 fixed arm 一律覆盖写: current inode_record=None deleted_inodes=[] trees_and_no_file=false
K2 fixed arm 一律覆盖写: 拒 Unexplained WriteToAnInodeThatIsNotInTheTree { inode: 1 } | checker []
K2 fixed arm 建 0 个 inode: current inode_record=None deleted_inodes=[] trees_and_no_file=false
K2 fixed arm 建 0 个 inode: 发成之后 inode 号 [(2, 1)]
K2 fixed arm 建 0 个 inode: 发成 | checker []
```

- 态与名字相符：根槽（第 124 个写）单独一段，两槽轮换（125、126）是下一段；摆出来的盘就是写表前缀到根槽为止，根落了、两槽轮换都没落，只读恢复落在建 inode 那一版 (1, txg 6)，挂载之后现行那一版是带文件的一版、`inode_record` 是 None、队列空。窗口越过段尾只是为了扣下第 125 个写，这个态与「第 27 段一个都没落」是同一个盘，落在常规枚举域里，不是越过屏障才有的态。
- 20-1 的臂建出 inode 3、池级 checker 绿；「一律覆盖写」被拒成 Unexplained，第二截会多一条发现，用例照它的文档红（推的，checker 档没跑）。
- **H1**：同一个态上「建 0 个 inode」（`publish_new_inodes(…, 0, …)`）也发成、checker 绿，盘上没有新 inode。用例只断言发布做成、没被拒（`crates/singlefs-checker-tier/src/crash_injection.rs:4718`：「(1, 0),」）与第二截没有新发现（`crates/singlefs-checker-tier/src/crash_injection.rs:4722`：「findings_after_the_writable_mount.is_empty(),」），名字里的「creates_an_inode」没有断言罩着：这一支的个数从 1 改成 0、或换成任何一种做得成的发布，用例照绿（推的：第二截还跑记录核对器，我的探针没跑它；0 个 inode 的那次发布它判不判红没量）。

## 三、打中的两处：四句

| | H1 固定用例不核「建出了 inode」 | H2 nlink = 0 那一半没有用例走到 |
|---|---|---|
| 分不分辨臂 | 分辨：区分「这一支建一个 inode」与「这一支发一次别的做得成的发布」（例如建 0 个）。量过：两条臂在同一态上盘面不同（inode 号表 [(2,1),(3,1)] 对 [(2,1)]） | 分辨：区分 20-1 的判法与「只看 `inode_record` 是不是 Some」。量过：250 个短续做态上后者走覆盖写、结局与 20-1 之前那一臂相同（Unexplained，第二截报假发现），前者是单元区墙、不报 |
| 被判的系统当时看不看得到 | 看得到：发布交回的那一版、发布之后的盘上 inode 树都在，用例拿得到 | 看得到：挂载交回的现行那一版带着 `inode_record` 与 `deleted_inodes` |
| 满足判据字面哪一句 | 题面 K2 格「固定历史用例摆的态是不是它名字说的那个」：态是那个态（满足），名字后半「建一个 inode」没有断言（这一句打中的是用例的判别力，不是态） | 题面 K1 / K2 分工里攻方那一句「攻哪一格会错：用例判不出的接错」：判法的 nlink 那一半是 20-1 新加的，没有一条用例让它为假 |
| 跑前条款的改法在这几格上还中不中 | 正文第五节「打中而坐实 ⇒ 改法并进下一批实现员」：下面的改法 a 直接罩这一格 | 同上，改法 b 罩这一格；改法 a 碰不到它（固定态的 `inode_record` 是 None，不是 nlink = 0） |

两处都不改变 20-1 发布臂今天的行为，只是变异「这一支建 0 个」「判法去掉 nlink 那一半」各有一条活路（推的，checker 档变异没跑）。

## 四、我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| a. 固定历史用例加一条：发布之后那份盘（`WritableMountOnTheCrashImage::pool_after`）只读恢复读回的 inode 号集合比崩溃那一版多恰好一个，或比两版的 inode 号水位差 1 | H1 | 推的：没实现、没跑；「建 0 个」那一臂盘上 inode 号表不变是量过的（二·3 输出），所以这条断言在那一臂上会红 |
| b. 加一条 checker 档用例，把二·2 量 A 的一个短续做盘面（例如单元区 128 槽、覆盖写 3 次、14 个单元、意图之后 1 批）交 `mount_and_publish_once_on_the_crash_image`，断言结局是 `PublishRefused` 且理由是单元区墙；登记一行变异「判法只看 `inode_record` 是不是 Some」 | H2 | 推的：没实现、没跑；那个盘面上挂载推满仍放不下、20-1 的臂被单元区墙拒、「只看 Some」那一臂报 Unexplained，是在 harness 档复做的同一条路上量过的 |
| c. 给一·3 记一笔账：读故障那一支在今天的随机历史里一次都没交过 `Injected`（0 / 122），C670 读故障一支的判别力、F5 的修法与第三截「恒交 NoneInjected」那一种接错都要等有一种步在读故障下真的写盘才量得到 | 一·3 旁注 | 射程量过；要不要立账由主 agent 定 |

## 没打中的形状

| 试过的形状 | 取样范围 | 结果 |
|---|---|---|
| F1 两条断言在正确接线下有一条不成立（变体名互为子串、取条件的字符串切法切错、新发现被已知红吃掉） | 一个靶向态；读码核已知红清单与 `ModelDisagreement` 字段次序 | 没中：两条都成立，取出的条件就是判官那一份 |
| F2 点名的三种接错里有一种计数仍是 (1, 1) | 一个靶向态 × 三种接错 + 一种未点名的（所选那一版交 None） | 没中：四种都不是 (1, 1) |
| 靶向二次崩溃摆出来的不是文档说的态（根槽前面还有同段的写、轮换不紧跟根槽、崩在的段取错） | 一个靶向态，挂载写表逐条打出来 | 没中 |
| 薄壳改了按种子那一支的种子、调用或计数 | 改动全文里 `inject_crashes_into_history` 的四处 hunk | 没中（推的） |
| 判法与写入口不对应（判法为真而写入口拒、判法为假而写入口放行） | 15948 个段边界态 + 250 个短续做态 + 固定态 | 没中：0 次 |
| nlink = 0 那一支直接建 inode 让第二截报假发现，或比会话那一路少看一段写 | 250 个短续做态 × 五条臂（三种会话改动放开扫） | 没中 |
| nlink = 0 那一支在随机历史的崩溃态上走得到 | 段边界态 33059 个（两组比重、三档窄盘、360 段历史）；4 GiB 3 段 | 没走到；层 0 全域没跑（见上） |

## 这条腿自己的限度

- checker 档一条都没跑。F1、F2、固定历史用例、F5 单测与 6 行 checker 档变异的红绿都是从 harness 档复做推的；复做抄了 checker 档的胶水（第二截拼报红那一句、第三截的计数取舍、发布臂的分支），抄错的话复做与真用例会分开。第二截与第三截的记录核对器（`HistoryThenWritableMountRecords`，在 checker 档）我没跑，H1 那一句「建 0 个 inode 用例照绿」里记录核对器那一半是推的。
- 短续做的盘面是造出来的：造盘时关了空间准入、不补环窗口（照现有 harness 档用例的造法），不是随机历史在准入判着的盘上走出来的。它说明这一支一旦走到会怎样，不说明崩溃注入会走到。
- 随机历史那一路只取了段边界态，不是层 0 的枚举域；全域每段历史都超过约 10⁶ 个状态，按约束没跑。这一条腿全部原型跑的态：短续做形状 6604 个盘面（每个盘面一个态，再在 250 个上各跑五条臂），段边界态 33059 个，靶向态 4 个；层 0 全域态 0 个，离约 10⁷ 的总量很远。
- 时长：短续做扫描分三片并行，每片约 2 分钟；段边界态每档约 1 分钟；没设批的限时。4 GiB 估一段约 64 秒（3 段 193 秒），没往下跑。
- 只抽了一次样。没打中的几格按规矩不算「没打中」，要主 agent 再派一条同立场腿才算。

## 没做什么

- 没跑 checker 档用例、checker 档变异、任何重型测试；没跑门禁。
- 没判正推那几格（K3 的 18-2 两行变异、注释与常量、重定基），只在一·5 顺带核了薄壳。
- 没核 GPU 判官那一侧。
- 没量 4 GiB 盘宽上带 unlink 的历史（每段约 64 秒，一段都没跑到有 unlink 的）。
- 副本上的数不进 kb，要主 agent 在入库装置上重做。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b20-r1-opus/` 里有仓副本 `repo/` 与编译目录 `target/`，交回前删掉；日志的原样拷在模型目录 `outputs/`。

## 运行环境与中途消息

- 开工时 `ps` 看到别的会话在跑 `cargo test --offline -p singlefs-harness --test probe_k3`（经内存包装），没有性能测量在跑；我的编译与运行都加了 `nice -n 19`，内存包装要求同会话并行时给理由，照它的提示设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`，峰值表里有旧峰值的那几次设了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。包装退 251 / 252 的几次（上限写法与排队）命令一行没跑，照提示改了再跑，那几次不算结果。
- 开工读材料时收到主 agent 的消息「线程上限改成 10」：之后的编译与单条运行都用 `capped.sh 10`，三片并行时每片 `capped.sh 3`。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b20-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b20-r1/tree --root . --background research/prompts/_m3-formal-code-b20-r1-background.md` 末行：

```
  ✓ 核了 33 处引文，对上 33 处，没判 0 处（逐处列在上面）
```
