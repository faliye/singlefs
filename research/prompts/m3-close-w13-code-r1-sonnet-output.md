# m3-close-w13-code-r1 正推腿报告（K1–K5，2026-10-21）

立场：正推。从已定条款与 `crates/` 现行实现推出「应该是什么」，再逐格与被判的 7 份文件比。没起编译与测试，没读攻方腿这一轮的任何东西。

## 0 开工核对

被判的 7 份文件与开工快照逐份对（命令原样输出）：

```
$ sha256sum -c research/prompts/m3-close-w13-code-r1-snapshot/code-sha256.txt
crates/singlefs-core/src/make_filesystem.rs: OK
crates/singlefs-core/src/extent_tree.rs: OK
crates/singlefs-core/src/allocator.rs: OK
crates/singlefs-core/src/allocation_record_tree.rs: OK
crates/singlefs-core/src/records.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/mounted_read.rs: OK
```

用到的条款（行号都是此刻在 kb 文件里现查的）：

- D22 已定项 8 第 3 条：`.claude/kb/decisions/22-单元原子性怎么合成.md:179`。
- D22 已定项 8 射程（中途崩溃的结局）：`.claude/kb/decisions/22-单元原子性怎么合成.md:181`。
- I-1.10：`crates/singlefs-checker/invariants/01-块自描述.md:16`。
- mkfs 段序列登记行：`.claude/kb/layout/01-first-txn.md:410`。
- 欠账 C867、C780：`.claude/kb/checks-owed.md:575`、`.claude/kb/checks-owed.md:525`。
- 格式冻结政策里「加宽」与「预留 / 尾部」两行：`.claude/kb/decisions/15-格式冻结政策.md:189`、`.claude/kb/decisions/15-格式冻结政策.md:187`。

两个方便称呼（报告里不用序数代称段）：作废段 = 两块盘的 [0, 根环基址) 清零各一写；清零单元段 = 其余八次清零加四个单元写；三个根段 = 三条第 0 代根的 FUA 写各自成段；系统配置段 = 四个系统配置槽写。

## K1 两趟写序与条文逐句对

**结论：一致（代码逐条照条文做）；另有一处条文自身没对齐（条文与登记行冲突，已知，写回另派）。**

条文写序（D22 已定项 8 第 3 条，整句摘引）：
「写任何元数据之前，先把每块盘的 [0, 根环基址) 整段写 0」（`.claude/kb/decisions/22-单元原子性怎么合成.md:179`）
「每块盘的 [0, 根环基址) 清零之后两块盘各过一道屏障（作废），之后才写别的；其余清零与单元写之间不加屏障。」（`.claude/kb/decisions/22-单元原子性怎么合成.md:179`）

正推出的写序：先几何判定（零写）→ 每块盘一次 [0, 根环基址) 清零 → 两块盘各一道屏障 → 每块盘根环三区各一次清零、journal 环一次清零（共 4 次）与两个单元写，中间不加屏障 → 登记行里的屏障 → 三条根 FUA → 系统配置槽 → 屏障。

代码逐条对：

- 几何判定在任何写之前：「let unit_area_start = check_geometry(parameters, &sizes)?;」（`crates/singlefs-core/src/make_filesystem.rs:548`）。
- 作废段：「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」（`crates/singlefs-core/src/make_filesystem.rs:625`），区间右端是「let root_ring_base = region_start(0);」（`crates/singlefs-core/src/make_filesystem.rs:623`）；`region_start(0)` 的值是 `ROOT_RING_BASE_SLOT * SLOT_BYTES`（`crates/singlefs-core/src/root_ring.rs:96`，第一版 1 MiB），所以清的就是 [0, 1 MiB)。
- 作废屏障：「device_whose_system_configuration_area_was_zeroed.barrier()?;」（`crates/singlefs-core/src/make_filesystem.rs:629`），循环在 628–630，发在两块盘的清零循环（624–626）全部发完之后；先两个清零写、再两道屏障，与条文「两块盘各过一道屏障」一致。条文没规定「d0 清零 → d0 屏障 → d1 清零 → d1 屏障」还是「两清零 → 两屏障」，代码取后者，由检查档用例钉住（见下）。
- 其余清零：「device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;」（`crates/singlefs-core/src/make_filesystem.rs:633`）与「device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;」（`crates/singlefs-core/src/make_filesystem.rs:635`），每块盘 3 + 1 = 4 次；加作废段每盘 1 次，每盘 5 次、两盘 10 次，与条文「每块盘 5 步、两块盘共 10 步」相同。
- 单元写与清零同段：单元写在 `crates/singlefs-core/src/make_filesystem.rs:638` 与 `crates/singlefs-core/src/make_filesystem.rs:643`（两个 `write_at`，`WriteDurability::Plain`），位于清零循环（631–636）之后、屏障（`crates/singlefs-core/src/make_filesystem.rs:650`）之前，其间没有屏障。
- 根 FUA：「device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;」（`crates/singlefs-core/src/make_filesystem.rs:701`），在 650 的屏障之后。
- 系统配置槽：「device.write_at(offset, &slot, WriteDurability::Plain)?;」（`crates/singlefs-core/src/make_filesystem.rs:732`），收尾屏障 `crates/singlefs-core/src/make_filesystem.rs:736`。

「有没有一条写落在条文没说的段里」：整个 `make_filesystem.rs` 里的设备调用只有 `write_zeroes_at` 3 处（625、633、635）、`write_at` 4 处（638、643、701、732）、`barrier` 3 处（629、650、736），上面已全部落位；其余是纯函数拼字节（`build_packed_unit`、`build_index_node`、`location_entries`）。命令：

```
$ grep -nE 'write_at\(|write_zeroes_at\(|barrier\(\)' crates/singlefs-core/src/make_filesystem.rs
625:        device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;
629:        device_whose_system_configuration_area_was_zeroed.barrier()?;
633:            device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
635:        device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;
638:        device.write_at(
643:        device.write_at(
650:        device.barrier()?;
701:        device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;
732:            device.write_at(offset, &slot, WriteDurability::Plain)?;
736:        device.barrier()?;
```

（上面是此刻真跑的命令与原样输出，全文件共 10 行，没有别的设备调用。）

段序列的钉子：检查档用例断言段长「vec![2, 12, 1, 1, 1, 4],」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:108`），harness 用例断言「"2+12+1+1+1+4",」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:339`）。2 = 作废段的两个清零写；12 = 清零单元段（2 盘 × 4 次清零 + 4 个单元写）。我没跑这两条用例（检查档是重型，按定义不跑），段数是读码推出的 2 + 12 + 1 + 1 + 1 + 4，与用例断言的数相同。

**条文与登记行冲突（整行并排，不是代码错）：**

- 条文这一侧：「段序列与步数以 [layout/01-first-txn.md](../layout/01-first-txn.md) 八「mkfs 种根」那一行为准」（`.claude/kb/decisions/22-单元原子性怎么合成.md:179`）。
- 登记行那一侧：「⇒ `14+1+1+1+4`，25 次操作（21 次写，两道池屏障按设备记、每道两块盘各一步）、16402 个崩溃状态」（`.claude/kb/layout/01-first-txn.md:410`）。
- 代码这一侧：段序列 `2+12+1+1+1+4`、27 次操作、闭式 4117（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:370` 的 `(27, "2+12+1+1+1+4".to_string(), 4117)`；`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:336` 的注释「layout/01-first-txn.md 八 mkfs 那一行要跟着改（写回另派）。」）。
- 同形的没跟着改的还有：`scripts/crash/check-segment-registry-product-sample.txt:5`（`operations=25 segments=14+1+1+1+4`）与 `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` 里的 14+1+1+1+4 若干行（命令：`grep -rn '14+1+1+1+4' scripts .claude/kb/layout/01-first-txn.md`，此刻 7 行命中：e142 装置 4 行、登记样本 1 行、layout 406 与 410 两行，都是旧数）。这些我只数、不判。

推翻 K1「代码一致」的现象：把 `make_filesystem.rs:629` 的屏障拿掉后，`recorded_stream_matches_the_registered_mkfs_segment_sequence` 不红；或录制流里出现清零单元段以外的第二个原地写落在作废段与清零单元段之间；或段长不是 `[2, 12, 1, 1, 1, 4]`。推翻「登记行未对齐」：登记行 410 的段序列被改成 `2+12+1+1+1+4`（那一行此刻读到的仍是 14+1+1+1+4）。

## K2 fsid：对外入口、随机源、内层入口

**结论：对外入口与条文一致；读不出随机源怎么办——条文没说，代码拒且在任何写之前；内层入口照条文留着，crates 里没有生产调用者，调用方可以给出与旧池前 8 字节相同的 fsid（条文允许，用例有意用）。**

条文（整句摘引）：「fsid 由 mkfs 的对外入口生成，前 8 字节取自随机源；内层入口可给 fsid，给测试、装置与崩溃点重放用。」（`.claude/kb/decisions/22-单元原子性怎么合成.md:179`）

正推出的实现：对外入口在任何写之前从随机源取 fsid，把它填进参数，调内层入口；内层入口按调用方给的写；单元头只带 fsid 的前 8 字节。

代码链：

- 随机源是内核设备：「pub const RANDOM_SOURCE_PATH: &str = "/dev/urandom";」（`crates/singlefs-core/src/make_filesystem.rs:224`）。
- 取法：「std::fs::File::open(RANDOM_SOURCE_PATH)」（`crates/singlefs-core/src/make_filesystem.rs:235`）、「.and_then(|mut random_source| random_source.read_exact(&mut filesystem_identifier))」（`crates/singlefs-core/src/make_filesystem.rs:236`）、「.map_err(MakeFilesystemError::RandomSourceUnreadable)?;」（`crates/singlefs-core/src/make_filesystem.rs:237`）。取 16 字节整个 fsid，所以前 8 字节也来自随机源；`read_exact` 遇中断重试，读不满报 `UnexpectedEof`。
- 取在写之前、失败就返回：「let drawn_filesystem_identifier = filesystem_identifier_from_the_random_source()?;」（`crates/singlefs-core/src/make_filesystem.rs:259`）先于「let output = make_filesystem(&parameters, devices)?;」（`crates/singlefs-core/src/make_filesystem.rs:265`）；`?` 一返回，内层入口一个字节都没碰盘。没有退回别的来源（时间、计数器、固定值）的分支：全文件只有这一处生成。
- 前 8 字节怎么进盘：「u64::from_le_bytes(filesystem_identifier[..8].try_into().expect("切了 8 字节"))」（`crates/singlefs-core/src/unit.rs:45`）。条文说的「前 8 字节」就是这一切片。
- 内层入口照调用方给的写：「let filesystem_identifier = parameters.filesystem_identifier;」（`crates/singlefs-core/src/make_filesystem.rs:543`），单元头（573、584）、根（654）、系统配置（707）四处都用这一个局部量，没有第二个来源。

**读不出 / 读不满 / 退回别的来源：规则没说。** 条文那一句只定「取自随机源」，没定失败怎么办；代码的选择是拒（`RandomSourceUnreadable`），不退回。三方判决里时间当源「同秒必撞」（`research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S1 那一行）是不退回时间的旁证，但它不是条款。

**谁在调内层入口。** 命令与此刻命中：

```
$ grep -rnE '\bmake_filesystem\(' --include=*.rs crates scripts | grep -v '^crates/singlefs-core/src/make_filesystem.rs' | wc -l
137
$ grep -rlE '\bmake_filesystem\(' --include=*.rs crates scripts | grep -v '^crates/singlefs-core/src/make_filesystem.rs' | wc -l
81
$ grep -rlE '\bmake_filesystem\(' --include=*.rs crates scripts | grep -v '/tests/' | grep -v '^crates/singlefs-core/src/make_filesystem.rs' | wc -l
15
```

81 个文件里 66 个在 `tests/` 目录下；其余 15 个是：`crates/singlefs-checker-tier/src/` 的库文件 3 个（`crash_state_enumeration.rs`、`crash_judge_tables.rs`、`crash_amplification.rs`）、`bin/` 的实验装置 8 个（e156、e158、e168、e180、e181、e182、e183、e185）、`crates/singlefs-harness/src/` 的 `history.rs`、`scenario.rs`、`crates/singlefs-core/src/mount.rs`（测试模块里，11070、11523 两行）、`scripts/experiment-apparatus/e164_crash_amplification_design_function_test/probe_recording.rs`。`crates/` 里没有任何产品二进制（命令行、FUSE）调 mkfs：`ls crates/*/src/bin` 只有检查档的实验装置与 `unit_graph`。对外入口的调用方只有一处：`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:173` 的「let made = make_filesystem_drawing_the_filesystem_identifier(」（命令：`grep -rn 'make_filesystem_drawing_the_filesystem_identifier' --include=*.rs crates scripts | grep -v '^crates/singlefs-core/src/make_filesystem.rs'`，命中 2 行：13 行的 `use` 与 173 行的调用）。

**调用方会不会给出与旧池前 8 字节相同的 fsid：会，且是有意的，用例各自标明。**
- 同前缀有意：C659 的坏镜像用例用「filesystem_identifier[8..].copy_from_slice(b"remade-x");」（`crates/singlefs-harness/tests/checker_a_pool_remade_over_an_old_pool_sharing_the_first_eight_identifier_bytes.rs:51`）造前 8 字节相同、后 8 字节不同；C484 / C651 的回归用例按 `parameters()` 的固定 fsid 重做（读码；没逐个核每条用例的调用行）。
- 前 8 字节不同：检查档新用例「filesystem_identifier[..8].copy_from_slice(b"remade-x");」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:72`），GPU 对照用例同（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_a_make_filesystem_over_a_used_pool.rs:86`），判表单测「make_filesystem_parameters_with_a_small_journal_ring(*b"remade-tables!!!")」（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:3517`）。
- 对外入口自己：两次取到的前 8 字节相等的概率按 2⁻⁶⁴（推的，没量）；用例「"对外入口两次 mkfs 的 fsid 前 8 字节相同：fsid 不是从随机源取的"」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:201`）钉住成功分支。

**没有任何用例钉住失败分支。** 命令 `grep -rn 'RandomSourceUnreadable\|filesystem_identifier_from_the_random_source\|RANDOM_SOURCE_PATH' --include=*.rs crates scripts | grep -v '^crates/singlefs-core/src/make_filesystem.rs' | wc -l` 输出 0。`RANDOM_SOURCE_PATH` 是 `pub const`，用例改不了路径，也就造不出「读不出」。变异表只覆盖成功分支（`crates/mutations.tsv:5084` 把取随机源换成固定值、`:5085` 把内层入口改成取随机源）。「失败在任何写之前交回」这一句今天只靠读码成立（259 先于 265）。

推翻 K2：对外入口的两次返回里前 8 字节相同并非碰撞（例如读到的是固定值）；或有一条走到 `make_filesystem` 内部写盘之后才可能出现的 `RandomSourceUnreadable`（今天没有这样的路径）。

## K3 mkfs 中途崩溃的结局

**结论：条文要求的两类结局，代码的写序在读码上推得出，没有留出第三类；用例里的态数（8 / 31 122 / 65，合 31 195）是主 agent 与实现员量的，我复核不了（检查档用例是重型，按定义不跑）。**

条文（`.claude/kb/decisions/22-单元原子性怎么合成.md:181`，整句摘引）：
「两块盘是同一个池时，作废持久之前崩溃：旧池读回最后确认的一版；可写挂载的结局按现行补写条款。」
「作废持久之后、最后一道屏障之前崩溃：下一次挂载只有两种结局——整池拒（`NoValidSystemConfiguration`），或恢复读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置。」
「设备守屏障时，作废持久之后任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存」

正推（读码，每步引代码）：

1. 作废持久之前（作废段内的崩溃态）。作废段之前代码没有任何写（K1：几何判定在 548，首写在 625）；作废段里只有两个写零，目标是各盘 [0, 1 MiB)，它们只能抹掉（或撕裂地抹掉）系统配置槽，不会往盘上写任何新字节。所以此时恢复的输入 = 旧池的盘面减去若干系统配置槽：根环、journal 环、单元区一字未动。恢复择系统配置的逻辑是每盘取可择槽里世代号大的、一块盘两槽全废就跳过：「// 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上；记下来交给调用方。」（`crates/singlefs-core/src/recovery.rs:1525`）。只抹掉一块盘时另一块盘的旧系统配置仍在、旧根都在 ⇒ 读回旧池最后确认的一版；撕裂（槽 0 抹了、槽 1 在）落在条文已有的单故障承诺里：「容错承诺是单故障：每盘一槽系统配置坏在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:181`）。用例的判法：作废那两次写没有都持久 ⇒ 必须是旧池那一版，否则 panic（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:168` 的 `if *root == (old_version.instance, old_version.checkpoint_txg)`；`:186` 的 panic 文案）。
2. 作废持久之后、系统配置段之前。作废段两块盘的屏障（629）返回后，两块盘 [0, 1 MiB) 全 0，旧池的系统配置一槽都没了；清零单元段（根环、journal 环清零、单元写）与三个根段都不写 [0, 1 MiB)（根环从 1 MiB 起，单元区在 journal 环之后）。所以这一段里任何崩溃态都没有可择的系统配置：两块盘都落「两槽全废」，最后落到「(None, Some(device)) => Err(RecoveryFailure::NoValidSystemConfiguration {」（`crates/singlefs-core/src/recovery.rs:1569`）= 整池拒。此时旧池的根环可能被清了一半，但没有系统配置，读不到，不会「读回旧池的一版」。
3. 系统配置段。新系统配置槽在这一段才第一次落盘（732），前面已经有：根环与 journal 环清零持久（650 的屏障）、三条第 0 代根各自 FUA 成段（701）。所以只要有一个新槽自证过，盘上自证过的根就只有三条新的第 0 代根，旧根已清 ⇒ 恢复读回 (实例 0, txg 0) 的空池；没有槽自证过 ⇒ 回到整池拒。用例对应：「(true, RecoveryOutcome::NoFile { root })」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:180`）那一支认 (实例 0, txg 0) 的第 0 代空池。只读挂载读这种空池：「return Err(OpenPoolForReadFailure::TreeTableHasNoEntries);」（`crates/singlefs-core/src/mounted_read.rs:442`）；条文已把「挂载的结局」两个分句删掉，只写恢复，所以这一条是现状、不是条文要求。
4. 「新系统配置与旧池的根并存」要成立，需要新槽先于旧根被清落盘。代码里新槽的写（732）在清根环的屏障（650）之后，两者之间还隔着三个根段，所以不会并存；把 732 提前到 650 之前，这一类态才摆得出来。
5. 「屏障只到一块盘归 C6」：代码对每块盘各调一次 `barrier()`（629 的循环），接口的约定是「屏障：之前发出的写全部持久之后才返回。」（`crates/singlefs-core/src/block_device.rs:218`），签名「fn barrier(&mut self) -> Result<(), BlockDeviceError>;」（`crates/singlefs-core/src/block_device.rs:219`）；设备不守屏障的情形条文已划出射程。

用例里报的数（引自测试注释，我没复跑）：「assert_eq!(tally.states, 31_195, "判决 S2：1 + 8 + 31 103 + 3 + 80");」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:343`）与「assert_every_state_lands_in_an_allowed_outcome(&tally, 8, 65);」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:344`）。我从读码推出的分类与它一致：旧池 8 态来自作废段里「两个清零写没有都持久」的子集，65 态来自系统配置段，其余整池拒。我没有独立数这三个数。

可写挂载的结局：条文写「按现行补写条款」，不在这一轮的 7 份文件里（`crates/singlefs-core/src/mount.rs:5851` 的 `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`）。我没读它的全部分支，不判。

推翻 K3：任何一个崩溃态 —— 作废持久之后恢复读回旧池 (实例 1, txg 3)（或旧池任一版）；或系统配置段之前恢复读回第 0 代空池；或有新槽与旧根同时自证过。能直接量的办法是用例里的观察者（`observe_the_outcome_allowed_by_the_clause`）在 31 195 态上不 panic；它任一态 panic 就推翻。

## K4 六处读者「不等于就拒」与同形读者有没有漏

**结论：六处都是真的「不等于就拒」，两边是同一个量（盘上条目宽对字段表宽，单位字节）；全仓另有同形读者没改：确定的一处（extent 上段叶条目 113，与 I-1.10 冲突，已知、轮外），有意宽松的两处（分配记录树的影子账走读、中央映射树条目），树表条目本来就是「不等于」。**

条款：
「**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）」（`crates/singlefs-checker/invariants/01-块自描述.md:16`）。
欠账 C867 点名的六处：「extent 内部、分配记录树、inode 内部、extent 叶、deadlist、树表条目」（`.claude/kb/checks-owed.md:575`），要求「六处读者各改成不等于字段表宽就拒」（`.claude/kb/checks-owed.md:575`）。

**两边是不是同一个量。** 左边：`parse_index_node` 先核「if entry_count * entry_width != declared_length {」（`crates/singlefs-core/src/unit.rs:430`），再按「.chunks(entry_width.max(1))」（`crates/singlefs-core/src/unit.rs:445`）切，所以每条条目的 `len()` 恰好等于头里的 `entry_width`（`entry_width` 是 `usize::from(u16)`，字节）。内部节点读者比 `header.entry_width`，叶读者比单条 `bytes.len()`，是同一个数。右边：都是 `singlefs-format` 的字节常量，转成 `usize`。

六处逐个对：

| 读者 | 比较行 | 右边常量 | 判 |
|---|---|---|---|
| 分配记录树内部条目 | 「if header.entry_width != internal_width {」（`crates/singlefs-core/src/allocation_record_tree.rs:757`） | 「usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96");」（`crates/singlefs-core/src/allocation_record_tree.rs:754`）；常量「pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;」（`crates/singlefs-format/src/lib.rs:166`） | 不等就拒，同量 |
| 分配记录 20（叶） | 「if bytes.len() != usize::try_from(ALLOCATION_RECORD_BYTES).expect("20") {」（`crates/singlefs-core/src/allocator.rs:81`） | 「pub const ALLOCATION_RECORD_BYTES: u64 = 20;」（`crates/singlefs-format/src/lib.rs:148`） | 不等就拒，同量 |
| extent 树内部条目 | 「if header.entry_width != internal_width {」（`crates/singlefs-core/src/extent_tree.rs:908`） | 「let internal_width = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");」（`crates/singlefs-core/src/extent_tree.rs:905`）；常量「pub const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = 110;」（`crates/singlefs-format/src/lib.rs:173`） | 不等就拒，同量；上下两段的内部节点都经这一个函数（`internal_entries`，调用在 `extent_tree.rs` 1054、1237、1308） |
| deadlist 条目 | 「if bytes.len() != DEADLIST_ENTRY_KEY_BYTES {」（`crates/singlefs-core/src/records.rs:640`） | 「pub const DEADLIST_ENTRY_KEY_BYTES: usize = 8 + 8 + 27;」（`crates/singlefs-core/src/records.rs:587`），value 空，条目就是 key | 不等就拒，同量 |
| inode 树内部条目 | 「if bytes.len() != usize::try_from(INODE_INTERNAL_ENTRY).expect("120") {」（`crates/singlefs-core/src/records.rs:806`） | 「pub const INODE_INTERNAL_ENTRY: u64 = 120;」（`crates/singlefs-format/src/lib.rs:107`） | 不等就拒，同量 |
| extent 叶记录 | 「if bytes.len() != usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {」（`crates/singlefs-core/src/records.rs:832`） | 「pub const EXTENT_LEAF_RECORD_BYTES: u64 = 112;」（`crates/singlefs-format/src/lib.rs:101`） | 不等就拒，同量 |

对照 C867 的名单：C867 的第六处「树表条目」今天不在 diff 里，因为它本来就判不等于：「if bytes.len() != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")」（`crates/singlefs-core/src/records.rs:737`；开工前的 HEAD 版同样是 `!=`，命令 `git show HEAD:crates/singlefs-core/src/records.rs | grep -n 'bytes.len() != usize::try_from(TREE_TABLE_ENTRY_BYTES)'` 输出 `736:`）。diff 里换进来的第六处是分配记录 20（叶）。两份名单集合不同，意图一致（新用例文件头也写了「树表条目 200 的读者……本来就判「不等于 200」」，`crates/singlefs-harness/tests/tree_readers_refuse_extent_inode_allocation_and_deadlist_entries_wider_than_the_field_table.rs:5`）。

**全仓同形读者，读码查全。** 命令与命中（核心层）：

```
$ grep -nE '(\.len\(\)|entry_width|record_width) *(<|!=|>=|<=) *(u64::try_from\(|usize::try_from\()?[A-Za-z_]*(BYTES|ENTRY|_width|_bytes|width_in_bytes)' crates/singlefs-core/src/*.rs | grep -vE '^\S+:[0-9]+: *///?' | grep -vE '^\S+:[0-9]+: *(assert|let )' | wc -l
30
$ grep -nE '^\s*(<|!=|>=|<=) *(usize::try_from|u64::try_from|[A-Z_]{6,})' crates/singlefs-core/src/*.rs
crates/singlefs-core/src/inode_tree.rs:424:            <= INODE_LEAF_RECORDS
crates/singlefs-core/src/allocation_record_tree.rs:888:                < usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
crates/singlefs-core/src/recovery.rs:5179:                != usize::try_from(to_read.record_width_in_bytes).expect("记录宽是 40 或 16")
```

第一条的 30 行里 22 行是「!=」精确比较（六处、记账条目、码 2 多层树内部条目、logged_ops、树表、各记录类型、节点 / 数据单元整宽等），余下 8 行是 `<` 或 `>=`：`root_record.rs:129`（根槽缓冲对根记录）、`system_configuration.rs:142`、`:647`、`journal.rs:286`（槽或记录缓冲区，缓冲区本来就比记录宽，不是索引节点条目）、`unit.rs:225`（写侧断言）、`unit.rs:437`（通用头判，只判条目宽不小于 key 宽，条款原话「`index_node_view` 只判了它 ≥ key 宽」）、`extent_tree.rs:515` 与 `records.rs:853`（下面第 1、3 处）。第二条（换行写法，第一条抓不到）多出 `allocation_record_tree.rs:888` 一处（下面第 2 处）。三处要单独说：

1. **extent 上段叶条目 113（第七处）：同形、没改，与 I-1.10 冲突。** 读者：「if bytes.len() < usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113") {」（`crates/singlefs-core/src/extent_tree.rs:515`），错误成员叫 `NarrowerThanItsFieldTable`（`extent_tree.rs:432`），生产路径上唯一的调用点是 `upper_leaf_entries` 里的「ExtentUpperLeafEntry::parse(entry_bytes).map_err(|malformed| match malformed {」（`crates/singlefs-core/src/extent_tree.rs:1118`）。对着条款：池级 checker 对上段叶按「==」判 I-1.10（上段叶 113 的条目宽由 `crates/singlefs-checker/src/walk.rs:4490` 取，判在「.judge("I-1.10", view.entry_width == entry_width, || {」，`crates/singlefs-checker/src/walk.rs:4311`）。所以一个上段叶条目宽 114 的镜像：checker 红 I-1.10，恢复与读者照读并忽略多出的 1 字节——正是 C867 要消的「宽于字段表的照读」那一形，只是这一处不在 C867 的名单里。主 agent 的背景材料已写「extent 上段叶第七处」在这一轮之外（实现员报的）。
2. **分配记录树的影子账走读（第八处）：同形、没改，是有意宽松。** 「|| header.entry_width」（`crates/singlefs-core/src/allocation_record_tree.rs:887`）换行接「< usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")」（`crates/singlefs-core/src/allocation_record_tree.rs:888`），在 `node_pointers_as_far_as_readable`；条件成立时是 `continue`（不再往下走），不是拒。它的文档写明目的：「从根指针往下，读得出多少就认多少」（`crates/singlefs-core/src/allocation_record_tree.rs:859`），后文写少认一个单元，被抛弃的根就不隔离。条目宽大于 96 时它仍按 `entry[..key]` 与偏移 10 起的子指针读，多出的字节被忽略。条款没说这条走读也要拒；它与「拒」的读者方向相反（认得多比认得少安全），我把它标为「规则没说」，由主 agent 判。
3. **中央映射树叶条目 55：同形、没改，条款明确不在射程内。** 「if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {」（`crates/singlefs-core/src/records.rs:853`）；I-1.10 正文「中央映射树还不判」，账在 C307（`crates/singlefs-checker/invariants/01-块自描述.md:16`）。一致。

池级 checker 自己还有 43 处「小于」的走读下限守卫，命令 `grep -rnE '(entry_width|\.len\(\)) *(<|>=) *(inode_internal_entry_bytes|allocation_record_bytes|internal_entry_bytes|mapping_entry_bytes|tree_table_entry_bytes|extent_leaf_record_bytes|accounting_entry_bytes|deadlist_entry_bytes|bytes_the_walk_needs)' crates/singlefs-checker/src | wc -l` 输出 43。它们是「走读要几个字节」的守卫，I-1.10 的判在别处按「==」做（`crates/singlefs-checker/src/walk.rs:2554` 的「.judge("I-1.10", entry_width == field_table_bytes, || {」），不是读者拒绝，不算漏。唯一值得点的是：checker 的 I-1.10 不含 deadlist 树：「条目宽同中央映射树只守走读要几个字节（I-1.10 的射程里没有它）」（`crates/singlefs-checker/src/walk.rs:2688`）。所以 deadlist 条目宽 44 的镜像：恢复今天拒（C867 的新行为），checker 的 I-1.10 不红。这条不是代码错，是 checker 与恢复在这一格的射程差，C867 行没提 checker 要不要跟。

零条目叶：叶读者按条逐条判，一个零条目的叶（上段根兼叶可以零条目：「零条目只许上段的根兼叶」，`crates/singlefs-core/src/extent_tree.rs:892`）没有条目可判，头里自述的条目宽不被任何读者核；I-1.10 判的是节点头。这不是这一轮新引入的（改前改后都如此），只记一笔。

推翻 K4「六处都是不等于就拒」：构造一条宽 +1 的条目放进上面六种节点之一，读者照读不拒（新用例文件有六条，主 agent 称改回只拒窄于时各红，`crates/mutations.tsv:5080` 起的 C867 行）。推翻「第七处」：upper leaf 宽 114 的镜像上 `ExtentUpperLeafEntry::parse` 返回 `Err`。

## K5 改名与拒因的处置

**结论：改名在每个调用方的处置与改名前相同（只换了匹配的成员名）；拒因的名字对宽条目是旧名（名实不符，轮外已知）；没有算术依赖「条目窄」；这个读者拒不掉一个本写者写出的盘；将来加尾随字段的盘归格式冻结政策管，条款没写读者怎么处置，属规则没说。**

**改名。** 旧名 `PointerCarryingEntryRefusal::NarrowerThanItsFieldTable` 在 crates 与 scripts 里命中 0 次，新名命中 11 次（命令：`grep -rn 'PointerCarryingEntryRefusal::NarrowerThanItsFieldTable' --include=*.rs crates scripts | wc -l` 输出 0；`grep -rn 'WidthOtherThanItsFieldTable' --include=*.rs crates scripts | wc -l` 输出 11）。定义一处「WidthOtherThanItsFieldTable,」（`crates/singlefs-core/src/records.rs:785`）；返回它的两处是 `records.rs:807`、`:833`；匹配它的调用方四处，各自的处置在 diff 里只换了匹配行，函数体没动：

```
$ grep -nE '^[-+] *PointerCarryingEntryRefusal::(Narrower|WidthOther)ThanItsFieldTable =>' research/prompts/m3-close-w13-code-r1-snapshot/product-code.diff
72:-                    PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
73:+                    PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable => {
290:-            PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
291:+            PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable => {
388:-                            PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
389:+                            PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable => {
415:-            PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
416:+            PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable => {
```

四个调用方现在的位置与处置：`crates/singlefs-core/src/extent_tree.rs:1014`（extent 叶记录 → `RecoveryFailure::EntryNarrowerThanItsFieldTable`，`what` 为「extent 叶记录」）、`crates/singlefs-core/src/mounted_read.rs:502`（→ `OpenPoolForReadFailure::RecordMalformed`，`what` 为「inode 树内部条目」）、`crates/singlefs-core/src/recovery.rs:5051`（容器树逐层走读 → `EntryNarrowerThanItsFieldTable`）、`crates/singlefs-core/src/recovery.rs:5968`（→ `EntryNarrowerThanItsFieldTable`，`what` 为「inode 树内部条目」）。两种拒因的第二支 `PointerHeadFieldOutsideTheFirstVersion` 没动。records 单测里的两条断言（`records.rs:1452`、`:1457` 的 `Err(PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable)`）随名字换了；`parse_inode_internal_entry` 的 `Option` 包装（`crates/singlefs-core/src/records.rs:796`）不看成员名，不受影响。

**拒因的名字。** 宽条目报的仍是 `RecoveryFailure::EntryNarrowerThanItsFieldTable`，成员名对宽条目不实，变体文档也还写「窄于」：「/// 索引节点头自述的条目宽窄于这棵树的条目字段表：条目宽是**盘上的值**，」（`crates/singlefs-core/src/recovery.rs:531`）。主 agent 背景材料已列这一处在轮外。我核了会不会因此出错：

- 有没有拿 `entry_bytes` 与 `field_table_bytes` 做减法：命令 `grep -rnE 'field_table_bytes *-|- *entry_bytes|entry_bytes *<' --include=*.rs crates` 输出为空 ⇒ 没有，宽条目不会引起整数下溢。
- 有没有按成员名分流并假设窄：只有两处按名匹配：GPU 判表的失败点映射「RecoveryFailure::EntryNarrowerThanItsFieldTable { .. } => {」（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:317`）与 harness 历史标签「format!("RecoveryFailure::EntryNarrowerThanItsFieldTable（{what}）")」（`crates/singlefs-harness/src/history.rs:3953`）。两处都用 `{ .. }` 或只取 `what`，不看宽窄 ⇒ 宽条目与窄条目走同一个结局。

**拒宽条目会不会拒掉一个合法的盘。** 先看本写者写出的盘：六种条目的写侧都用同一批常量 ——
extent 叶「u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),」（`crates/singlefs-core/src/extent_tree.rs:631`）；
extent 内部「u16::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"),」（`crates/singlefs-core/src/extent_tree.rs:664`，下段 736 同）；
inode 内部「u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),」（`crates/singlefs-core/src/transaction.rs:12357`，墓碑、谱系、退役三棵树根的 12557 同）；
deadlist「MultiLevelCodeTwoTree::Deadlist => DEADLIST_ENTRY_KEY_BYTES,」（`crates/singlefs-core/src/transaction.rs:2601`）；
分配记录叶「ALLOCATION_RECORD_BYTES,」（`crates/singlefs-core/src/allocation_record_tree.rs:533`）与内部「ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,」（`crates/singlefs-core/src/allocation_record_tree.rs:543`），都经「u16::try_from(entry_width).expect("条目宽 2 字节"),」写进头（`crates/singlefs-core/src/allocation_record_tree.rs:570`）。
写侧宽 = 读侧宽 = 常量，所以本写者写出的盘不会被「不等于」拒掉；「写出」的实际字节读回由已有的往返用例管，我没复跑。

「将来某类条目合法地带尾随字段」：条款没有给读者一条「忽略尾随字段」的路。I-1.10 写的是「等于」（`crates/singlefs-checker/invariants/01-块自描述.md:16`）。格式冻结政策里相关两行：加宽走不兼容位：「| 扩大已有字段宽度 | 按旧宽度解析，越界读 / 错位 | incompat | 是 | 宽度是布局的一部分，旧代码的偏移计算全错 |」（`.claude/kb/decisions/15-格式冻结政策.md:189`）；预留区 / 尾部的新增才是兼容：「| 新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否 |」（`.claude/kb/decisions/15-格式冻结政策.md:187`），但政策自己限了射程：「第一行那条（预留区里的新增是 compat）只在真有预留区时成立，索引节点头只有 2 字节预留，装得下的字段很少。」（`.claude/kb/decisions/15-格式冻结政策.md:197`）。定宽条目没有预留区或尾部。推：给条目加尾随字段是「扩大已有字段宽度」，该设不兼容位；不认识的不兼容位在择系统配置时就整池拒（「RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {」，`crates/singlefs-core/src/recovery.rs:1564`），到不了条目读者，所以「等于」拒绝不会先于不兼容位把一个合法的新盘拒成数据损坏。成立的前提是将来的改动老老实实走 189 行那一类；如果有人把某类条目的加宽当成 187 行的兼容改动，本读者就会拒掉它的盘 —— 这一点是格式冻结政策与 I-1.10 之间没写明的缝，条款没说，主 agent 判是否要在 D15 或 I-1.10 里补一句。

推翻 K5：任何一个四处调用方在改名前后对同一个镜像给出不同的错误成员或 `what`（diff 上看不出，除非函数体被改，8 行 diff 只换了匹配行）；或写侧某个写入点写出与读侧常量不等的条目宽（上面列的写入点之外的 `build_index_node` 调用：`unit.rs:641`、`:720`、`region_scan.rs:558`、`transaction.rs` 的 12547、12653、12694、14005、14818 我只核了与六种条目相关的，没有逐个对到常量）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 | 代码一致；条文与登记行冲突 | 写序、分段、屏障逐条照条文，设备调用共 10 处无遗漏；`.claude/kb/layout/01-first-txn.md:410` 仍是 `14+1+1+1+4`、25 次操作，D22 条文却写「以那一行为准」，写回另派 |
| K2 | 一致；失败分支规则没说 | 对外入口读 16 字节 `/dev/urandom`，失败在写之前返回、无退回；条文没定失败怎么办；失败分支无用例；内层入口 137 行 / 81 文件在调，无产品调用者 |
| K3 | 读码一致；态数复核不了 | 作废段只抹系统配置，之后无可择系统配置直到系统配置段，根环此时已清，故两类结局之外没有第三类；8 / 31 122 / 65 是别人量的 |
| K4 | 六处一致；有漏 | 六处都是同量的「不等于就拒」；extent 上段叶 113 仍只拒窄于，与 checker 的 I-1.10 冲突（已知、轮外）；影子账走读与映射条目有意宽松；checker 的 I-1.10 不含 deadlist |
| K5 | 一致；名实不符轮外 | 四个调用方处置未变；宽条目仍报「窄于」的旧名，无算术依赖；本写者写出的盘不被拒；将来加字段的缝是 D15 与 I-1.10 之间没写明的 |

## 没做什么

- 没跑任何编译与测试。K3 的态数（8 / 31 122 / 65、合 31 195）、段序列 `2+12+1+1+1+4`、27 次操作、4117 的闭式，都是引用已入库用例与注释里的数，我只读码推了分类，没复算：检查档用例是重型，按定义不跑；harness 档用例（`mkfs_bytes_judged_by_the_checker`、`tree_readers_refuse_extent_inode_allocation_and_deadlist_entries_wider_than_the_field_table`、`checker_a_pool_remade_over_an_old_pool_sharing_the_first_eight_identifier_bytes`）派发提示说不起编译与测试，也没跑。
- 没判可写挂载在作废之前 / 之后的结局（条文写「按现行补写条款」，实现在 `mount.rs`，不在被判的 7 份里）。
- 没核 D22 的英文、日文两份译文与条文是否同步，也没核 layout 的英文、日文两份。
- 没逐个核每条同 fsid 重做 mkfs 的用例（C484 / C651）的调用行，只核了 C659 用例与新增的三处。
- 没对 `build_index_node` 的全部 14 个调用点逐个对常量，只核了六种相关条目的写入点。
- 不替主 agent 采纳或出判决；三条「规则没说」（随机源失败怎么办、影子账走读要不要也拒、D15 与 I-1.10 的加宽缝）只摆出来。
- 没写模型目录（没有需要独立复跑的模型；所有命令都是 grep）。

## 要主 agent 跑的命令

（都归主 agent；检查档是重型，按 `.claude/agent-common.md` 的重型测试一条由主 agent 带 `SINGLEFS_HEAVY_TESTS` 起，范围声明与内存包装照那一条的写法。我只列要看哪一行判结果。）

1. 检查档用例 `crash_enumeration_make_filesystem_over_a_used_pool_stream`（快档与 `--ignored` 全量各一条）：看 `tally.states` 是否 31 195、三种结局是否 8 / 31 122 / 65，以及观察者是否零 panic（用例自己的断言）。
2. harness 档 `mkfs_bytes_judged_by_the_checker`：看 `recorded_stream_matches_the_registered_mkfs_segment_sequence` 与 `the_outer_entry_draws_filesystem_identifiers_whose_first_eight_bytes_differ_between_two_runs` 绿。
3. 探 K4 第七处：造一个上段叶条目宽 114 的镜像（可抄 `tree_readers_refuse_…_wider_than_the_field_table.rs` 的 `widen_every_entry_of_the_node_by_one_byte`，把目标换成 extent 树上段叶），看恢复是否拒、池级 checker 是否红 I-1.10。我读码推的是「恢复照读、checker 红」，没有跑过。
