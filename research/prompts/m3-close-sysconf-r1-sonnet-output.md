# m3-close-sysconf-r1 云端正推腿报告（S1–S6）

日期：2026-10-19。腿：three-way-forward（Sonnet）。立场：从条款原文与 `crates/` 今天的代码推，不带倾向。
代码引用的是只读树副本 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree` 里的路径与行号（核引文要带 `--root` 指到这份副本）；kb 与不变量引用的是仓里此刻的行号。
没写模型，没编译，没跑重活。凡写「推的」的，没有造镜像、没有量过。

## 零、六格共用的事实

0.1　第一版就是两块盘：「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」（`crates/singlefs-core/src/make_filesystem.rs:479`）。

0.2　R2 判不出谓词的两个合取项：「pub fn system_configuration_witness_is_undecidable(」（`crates/singlefs-core/src/recovery.rs:4306`），第一项是「.any(|slots| slots.slots_reporting_a_read_error > 0)」（`crates/singlefs-core/src/recovery.rs:4311`）。
两盘下「没有一块盘两槽都自证过」等于两块盘各至少有一槽不是自证过的，再加至少一槽报读错。所以 S1 的盘面至少是每盘一个槽出问题。

0.3　D22 已定项 8 射程把承诺写成：「容错承诺是单故障：每盘一槽系统配置坏在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`），
接着写「每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。
R2 之后判不出的盘面里有一部分正落在第一句的字面里：一盘一槽报读错、另一盘一槽读回全零，用例头里写着「混合形（一盘报读错、另一盘读回全零）：判不出、拒可写」（`crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs:9`）。
条款没写「承诺」是否含可写挂载。这是 S1 与 S5 都要碰的一处空白。

0.4　系统配置槽写的是哪一槽：世代号 = 这块盘两槽里自证过的最大世代号 + 1，两槽都读不出时从 1 起，槽 = 世代号 mod 2（`crates/singlefs-core/src/transaction.rs:559` 起的 `write_system_configuration_slot`，槽号一行是「slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」，`crates/singlefs-core/src/transaction.rs:625`）。
条款侧：「取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1」（`.claude/kb/decisions/22-单元原子性怎么合成.md:333`）。
所以只要这块盘上还有一槽自证过，一次系统配置写就把另一槽（坏的那一槽）重写掉；这是 S1、S4 两种修复共用的写。

0.5　取号写本身就是逐盘写一次：「for index in 0..pool.devices.len() {」（`crates/singlefs-core/src/transaction.rs:1171`），之后一道屏障。取号式子在「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`crates/singlefs-core/src/transaction.rs:941`）。

0.6　恢复整条链只收只读读者：`scan_journal` 的参数是 `reader: &dyn PoolReader`（`crates/singlefs-core/src/recovery.rs:4322`），`mount_read_only` 也是（`crates/singlefs-core/src/mounted_read.rs:621`）。
所以「在恢复里自愈重写」不是改一处，是给恢复链换一个带写的读者类型；写只能放在拿着 `&mut devices` 的可写挂载一侧。

0.7　`crates/` 里没有面向文件系统的可执行入口：
`grep -rl 'fn main' crates --include=*.rs | grep -v 'singlefs-checker-tier' | grep -v unit-graph | wc -l` 在树副本里输出 `0`。
所以 S1 的「管理员工具」今天只能是 `singlefs-core` 里一个库函数（先例是 `pub fn roll_back_by_a_forward_publish<`，`crates/singlefs-core/src/mount.rs:8878`），命令行包装不在这一轮范围。

0.8　journal 记录与单元都是「写到每块盘」且中间没有屏障：「CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {」（`crates/singlefs-core/src/transaction.rs:495`）、「CommitStep::WriteUnitToEveryDevice {」（`crates/singlefs-core/src/transaction.rs:484`）。
所以一次发布里「记录写到了盘 A、没写到盘 B」与「后一条记录持久、前一条没持久」都是崩溃点重放会枚举到的子集。

0.9　新实例的第一条记录的计数器与反向链：计数器 = 环里读得出的最大计数器 + 1，「let next_counter = records」（`crates/singlefs-core/src/mount.rs:8074`）；
写行那次发布写死反向链 0，「back_chain: 0,」（`crates/singlefs-core/src/mount.rs:5037`）。
条款侧：「前缀末的计数器取环里读得出的最大那一个」（`.claude/kb/decisions/23-journal的角色与格式.md:371`）、「本实例写出的第一条记录恒 0」（`.claude/kb/decisions/23-journal的角色与格式.md:532`）。

0.10　没有「实例切换」的实现：新实例只在可写挂载取号时产生。这一条决定 S6 能不能造（见 S6）。

0.11　牵连面的数（树副本里数的，命令在各格里）：提到 `acquire_instance` 或 `acquire_expected_instance` 的 `.rs` 文件 67 个；提到 `system_configuration_witness_is_undecidable` 的 8 个（含 `crates/singlefs-harness/src/mount_transparency.rs` 那个独立复算的判官）；
提到落后支（`devices_behind_the_current_version` 等三个名字）的 12 个；写死世代号 5 或叫 `NEW_POOL_FILE_CREATION_ROTATION` 的 9 个。

## S1　C596：R2 之后仍判不出的盘面怎么走出只读

盘面（推的，按 0.2 展开）：两盘各至少一槽不是自证过的、至少一槽设备报读错，例如「两块盘世代 5 那一槽都报读错」（`crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs:7`）。
今天的结局：可写挂载在读阶段判据 N-配置 为真，重读一次仍真就拒，「WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots」那一支（`crates/singlefs-core/src/mount.rs:495`）；取号那一刻读见证值判不出也在第一个写之前拒（`crates/singlefs-core/src/transaction.rs:878`）。
只读挂载照常；没有别的入口写系统配置槽：`write_system_configuration_slot(` 在树副本里只有四个调用点（取号、发布末尾轮换、抬 F 先写系统配置、取号回卷；命令 `grep -n 'write_system_configuration_slot(' crates/singlefs-core/src/*.rs` 命中 transaction.rs 的 534、681、791、1172 行）。

为什么不能直接放行（读代码得到的）：见证读判不出时，读错的那一槽可能藏着比可见槽更大的 tail。tail 的作用是证明存在比所选那一版更新的发布（`crates/singlefs-core/src/mount.rs:7607` 起的 `newer_publish_witness`）。
把读错的槽重写掉，等于销毁这条证据。所以任何出路要回答的只有一件事：写之前用什么代替这条证据。

### S1 候选一（α）：离线的定点重写（管理员工具，库函数）

- 怎么触发：调用方调一个新库函数，形如 `repair_system_configuration_slots(parameters, devices: &mut Vec<(DeviceIdentity, Device)>, asserted_version)`，放在 `crates/singlefs-core/src/mount.rs`，挨着 `roll_back_by_a_forward_publish`（0.7）。
  `asserted_version` 是调用方断言的「最后一次确认过的那一版」(实例代号, checkpoint_txg)。`crates/` 里没有这个函数，也没有可执行入口（0.7）。
- 谁有权：条款与代码里都没有「管理员」身份，只有「谁拿着可写的盘」。同一时刻只有一个写者靠 D18 已定项 11「前提」：「同一时刻只有一个可写挂载」（`.claude/kb/decisions/18-块里携带什么信息.md:321`），该前提待 C833 落地，所以这个函数同样不防别的进程同时写盘（推的）。
- 写之前核什么：先按只读口径走一遍恢复读（`choose_system_configuration_naming_devices_without_a_mountable_slot`，`crates/singlefs-core/src/recovery.rs:1387`；`scan_journal`，`crates/singlefs-core/src/recovery.rs:4322`；`replay_journal`，`crates/singlefs-core/src/recovery.rs:4506`），得到所选那一版；与断言不等，一个字节不写就拒。
  再核每盘至少留一槽自证过（两槽都不自证的盘归 S4）。这样「断言」替代了被销毁的那条 tail 证据：写之前由人对着只读报告确认「最新确认过的就是这一版」。
- 写哪几个槽：只写每盘里不自证的槽，最多 4 个；盘上还有自证槽时一次 `write_system_configuration_slot` 就落到坏槽上（0.4）；不改任何已自证的槽，所以每盘自证槽数只增不减。
  带的值：实例代号 = max(各盘自证槽、根环全部根记录、journal 环全部记录的实例代号)，不加一；tail = max(各盘自证槽的 tail, 所选那一版末条记录的计数器)；F 取整池生效值（`RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor`，`crates/singlefs-core/src/transaction.rs:559` 起那个函数的参数）。写完一道池屏障才交回成功。
- 出错留什么：写某槽报设备错，函数返回已写与未写的清单；被写的槽本来就不自证，崩在写中途也只是仍不自证，盘上不比开始更坏；整个函数幂等，可再调。
- 要动哪几个文件的哪一步：`crates/singlefs-core/src/mount.rs`（新函数、报告类型、拒因枚举）；`crates/singlefs-core/src/transaction.rs`（`write_system_configuration_slot` 是 `PoolWriter` 的私有方法，且 journal tail、实例代号由调用点传入，4 个调用点见上面；要给它一个 `pub(crate)` 入口）。
  写出的新形态：没有，盘上格式不变。取号、读阶段、R2 谓词一行不动，所以 `mount_transparency.rs` 的独立判官与 e165 装置不用改（它们提到谓词的 8 个文件见 0.11）。
- 代价：1 个库函数 + 报告/拒因类型 + 约 6 条用例（断言不符拒且字节不变、相符重写、重写后 `mount_writable` 做成且读回确认过的写、写失败、幂等、两盘各坏一槽的盘面）+ `crates/mutations.tsv` 的变异行。运行时代价为 0（不在挂载路径上）。
- 能不能让「之后可写挂载做成，且确认过的写不丢」的用例成立：能造盘面（两盘各一槽报读错或读回全零、其余完好）；「不丢」只在断言等于盘上真正最新那一版时成立；用例必须带反向格（断言错一位 ⇒ 拒）。
- 与条款的关系：D22 已定项 8 射程不用改（承诺句、「取号那一步也拒」都不动）；C583 不动；C331 取甲、R2 不动，因为可写挂载的拒照旧，出路在挂载之外。
- 罩不住的历史：断言者断言了一个不是最新的版本（最新那次发布的根与记录都读不出，tail 又在读错的槽里）；设备写不进坏槽（扇区坏而不重映射）：函数报错，池仍是只读，这一形没有出路。

### S1 候选二（β）：只读挂载下的定点修复

- 今天做不到的原因是类型：`mount_read_only(reader: &dyn PoolReader)`（`crates/singlefs-core/src/mounted_read.rs:621`）拿不到写。「只读挂载不写盘」还有三条欠账名下的承诺（C32、C33、C75，`.claude/kb/checks-owed.md:38`、`.claude/kb/checks-owed.md:39`、`.claude/kb/checks-owed.md:71`）。
- 所以 β 只能是第三种挂载（修复挂载）：读阶段同 α，再同一个写。写的内容、写之前核的、出错留什么、改哪几处，全部同 α；多出来的是一条新的挂载种类条款，和 `MountedReadOnly` 之外的一个结果类型。
- 与 α 的差别只有包装：α 不要求先挂载；β 让调用方拿到挂载报告（`devices_without_a_mountable_system_configuration_slot` 那种字段，`crates/singlefs-core/src/mounted_read.rs:607`）之后在同一个句柄上修。两者共用同一个写函数，所以不是两套代价。

### S1 候选三（γ）：可写挂载在附加条件下放行

- 条件（推的最小可判一组）：判不出，但这次读阶段里每个根环槽要么自证过要么读成全零（没有设备读错、没有读成而自证不过；`crates/singlefs-core/src/mount.rs:2709` 的 `RootRingSlotReadingTellingNeverWrittenApart` 已有这三类），journal 环没有设备读错的记录槽，且每盘至少一槽自证过；满足就把读错的系统配置槽当「丢了」，按可见槽比较（c_见证 > c_E 则照拒）。
- 要动：`crates/singlefs-core/src/mount.rs` 的 `newer_publish_witness`（`crates/singlefs-core/src/mount.rs:7638` 那一支）、读阶段交出「环读干净」这个新字段；`crates/singlefs-core/src/recovery.rs` 的 `scan_journal`（读不出的槽今天静默 `continue`）要多交一个读错计数；取号那一刻的同一判（`crates/singlefs-core/src/transaction.rs:878`）要收同一个标志，否则放过读阶段又被取号拒；`crates/singlefs-harness/src/mount_transparency.rs` 的判官与 e165 装置跟着改。
- 写出的新形态：无；运行时不多读（环在读阶段已全读）。
- 罩不住：设备静默丢写（读回旧内容、不报错）不会让「环读干净」变假。R2 对读错的挡法在这里换成了对「有信号的读错」的挡法；这一步要在 C331 网格上量过才能引（推的，没量）。
- 与条款：C583 的双故障形（最新根一时读不出）会让环读出错，条件不成立，仍拒；与 C331 取甲/R2 的关系是再收窄一步，D22 已定项 8 射程那一句「判不出 ⟺ …」要改成带例外的写法；R2 是三轮判过的，γ 是零轮。

### S1 候选四（δ）：接受永远只读，写成已知代价

- 要动：条款一句；`crates/` 不动。代价：这一形的池永久只读，数据要靠外部拷出重建。
- 与条款的冲突：0.3 说明这一形有一部分在「每盘一槽系统配置坏在承诺里」的字面里，δ 要同时把那一句改成「承诺只含读得出」，否则 δ 与 D22 已定项 8 射程自相矛盾。
- 「之后可写挂载做成」的用例：按定义不成立，无此用例。

### S1 一句话条款（推荐 α）与推翻条件

条款句：R2 判不出的盘面，可写挂载与取号一律拒（不变）；出路是调用 `repair_system_configuration_slots`——入口收盘表与调用方断言的「最后确认过的那一版」(实例代号, checkpoint_txg)，先按只读口径读出所选那一版，与断言不等则一个字节不写地拒；相等则只重写每盘不自证的系统配置槽（每盘至少已留一槽自证，实例代号取各盘自证槽、根环、journal 环的最大值不加一，tail 取各盘自证槽 tail 与所选那一版末条记录计数器的较大者），写完一道池屏障，不发布、不取号、不动根环与 journal；重写之后下一次可写挂载按判得出走。

推荐 α 的理由（推的）：它不动挂载路径上任何已经三轮判过的判据，唯一新增的信任是调用方的断言，这个信任是显式的、可审计的、用例能反向钉住的；γ 要在 C331 网格上证明「环读干净」顶得住 R2，今天没有这个数。
什么现象会推翻它：(1) 用户不接受「靠人断言」，要自动放行——那时 α 出局、γ 须先量；(2) 在 C331 网格（E165 择根与重放合判族）上量出 γ 的放行格里丢确认过的写的格数为 0，则 γ 与 α 的选择只剩偏好；(3) 发现 `write_system_configuration_slot` 在持续读错的槽上写不成（设备拒写），则 α 在这一形上给不出出路。

## S2　C593：落后且缺一份单元的盘

条款今天的字面：「放行两种：只落后、单元都在、根槽那一半不缺」（`.claude/kb/decisions/18-块里携带什么信息.md:320`），
放行的另一种是「系统配置跟得上、只缺几份单元（单份坏，由 D19」（`.claude/kb/decisions/18-块里携带什么信息.md:320`）；两样同时出现的怎么办没写，C593 就是这一格（`.claude/kb/checks-owed.md:385`）。
代码：落后只看系统配置槽 tail，「if newest_system_configuration_journal_tail >= version_journal_position {」（`crates/singlefs-core/src/mount.rs:5802`）直接返回「不缺」；否则逐个读单元，缺一份就记下（`crates/singlefs-core/src/mount.rs:5791` 起的 `units_of_the_version_missing_on_a_device_behind_it`）。
根槽那一半另算一次落后，「let is_behind_the_version = newest_system_configuration_journal_tail < version_journal_position;」（`crates/singlefs-core/src/mount.rs:5851`），挂在 `what_a_device_behind_the_version_lacks`（`crates/singlefs-core/src/mount.rs:5824`）里。
三处入口共用这一个函数：可写挂载取号之前的 `devices_without_the_selected_version`（`crates/singlefs-core/src/mount.rs:5713`）、挂着之后收盘表的入口（`crates/singlefs-core/src/mount.rs:6308`）、会话每次用户改动（`crates/singlefs-core/src/mounted_session.rs:1885`），所以改判法只改那一个函数，调用点与拒因类型不动。

三种「一次正常崩溃」都会让落后成立，而单份坏的单元放行本来就是条款的意思（推的，按 `crates/singlefs-core/src/transaction.rs:484`、`crates/singlefs-core/src/transaction.rs:495` 的写序）：
- 崩点一：根槽已落，轮换没落。本盘有单元、有记录、有根。
- 崩点二：记录只写到了盘 A（逐盘写、中间没有屏障），根没落。盘 B 有单元、没有记录；单元是在记录之前的屏障里持久的，所以盘 B 单元齐全。
- 崩点三：记录两盘都落了，根没落（所选那一版是重放施加出来的）。本盘有单元、有记录。
比对用的单元清单是整一版的全部单元，不只是最后一次发布写的那几个：「带文件的一版就是重建出来的 `units`（全部角色」（`crates/singlefs-core/src/mount.rs:1172`）。
所以崩点一上本盘任何一个单元的一份坏副本，崩点二、三上本盘任何一个旧单元（不是最后那条记录点名的）的一份坏副本，今天都被判成「落后且缺单元」，可写挂载永久拒、挂着之后的入口与会话每次改动也拒。
崩点二、三上最后那条记录点名的单元坏了一份，不落在这里：施加前验证要两份都验过，「一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加」（`.claude/kb/decisions/23-journal的角色与格式.md:362`），那次发布先不施加，所选那一版退回上一版（推的）。

### S2 候选一（甲）：维持拒
- 要动：条款一句（把「两样同时出现判不带」写成已知代价）；`crates/` 不动。
- 代价：上面三个崩点各加单份单元坏，池永久只读，只读挂载读回完好。罩不住：无（拒掉了所有）。
- 与 C585：不动。

### S2 候选二（丁）：落后与否看本盘环槽里有没有那一版的末条记录
- 要动：`units_of_the_version_missing_on_a_device_behind_it`（`crates/singlefs-core/src/mount.rs:5791`）在 tail 不落后的返回之后再加一个返回：本盘上 `record_offset(version_journal_position.counter, newest_self_verified.immutable.sizes.journal_ring_bytes)` 那一槽解得出（按系统配置里的 fsid）且 (实例代号, 计数器) 等于那一版末条，就返回「不缺」。
  计数器为 0（mkfs 的第 0 代根没有记录）要先返回，`record_offset` 里是 `counter - 1`（`crates/singlefs-core/src/journal.rs:204`），0 会溢出。攻方当年的原型是 `yi-j-mount.diff`，在 `git show 31128273b^:research/prompts/c245-r3-opus-model/yi-j-mount.diff` 里取，被攻过零轮（`.claude/kb/checks-owed.md:385`）。
- 根槽那一半必须保持用 tail 判（`crates/singlefs-core/src/mount.rs:5851`），不能跟着换成「本盘有没有记录」：C585 的「旧快照取在发布途中（单元与记录已写、根槽还没写）」靠根槽那一半拒；换了判法，记录在就放行，这一形重新打开。
- 运行时代价：tail 落后的盘才多读一个 4096 字节的记录槽，会话热路径（tail 不落后）不变。写出的新形态：无。
- 覆盖：崩点一、崩点三（本盘有记录）任意多份单元坏都放行；崩点二不覆盖（盘 B 没有那条记录），仍拒。

### S2 候选三（丙）：缺的单元份数不到 2 就不算缺
- 要动：同一个函数的最后一行 `(!units_missing.is_empty()).then_some(...)` 改成「缺的份数不少于 min(2, 这一版单元总数)」。
- 依据是条款自己的话：「单份坏，由 D19」（`.claude/kb/decisions/18-块里携带什么信息.md:320`）指读盘核隔离的单份副本；注意比对的单元是整一版的，同盘旧单元坏两份以上的池，丙在崩点一上会拒，这是丙的代价，丁丙把它交给「本盘有没有那条记录」再判一次；两份以上同盘副本坏已经不是单份坏。2 是从「单份」直接来的，不是量出来的。
- 覆盖：三个崩点各加一份单元坏都放行；同盘两份以上副本坏仍拒（崩点一、三也拒，比丁窄）。
- 罩不住：旧快照缺恰好一份单元的形（单份坏与它分不开）；根槽那一半仍判它（根住在本盘时）。

### S2 候选四（丁丙）：两样都加
- 「不带」的第二种改成：tail 落后，并且本盘环槽里没有那一版末条记录，并且缺的单元份数不少于 min(2, 这一版单元总数)；根槽那一半不变。
- 覆盖：崩点一、三任意份单元坏；崩点二一份单元坏；同盘两份以上单元副本坏且记录也不在的旧快照仍拒。
- 与 C585：旧快照盘「记录不在 ∧ 缺两份以上」仍拒；「记录在、根不在」靠根槽那一半拒；会重新打开的只有「记录在而单元缺两份以上」，这要求快照不是一个写序前缀（单元在记录之前的屏障里持久），超出故障模型（推的，没造过）。
- 入口与会话：调用点不变（上面三处共用函数）；拒因类型里带的 `newest_system_configuration_journal_tail` 等字段不变。

### S2 一句话条款（推荐丁丙）与推翻条件
条款句：取号之前逐盘核与挂着之后收盘表的入口，「不带所选（现行）那一版」的第二种是——这块盘最新那份自证系统配置的 (实例代号, journal tail) 小于那一版末条记录的 jsn，并且本盘环槽里没有那一条记录，并且缺的单元份数不少于 min(2, 这一版单元总数)；或那一版的根写在这块盘上而那一槽读一次、重读一次都不是它（后一半不变）。
推荐丁丙而不是丁的理由：丁漏崩点二，丙漏「同盘两份以上副本坏」，两个窗口都是条款放行语义（单份坏）的一部分，合起来才罩住三个崩点。
什么现象会推翻它：(1) `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 里任一格由拒变放（该文件在落后支名下，12 个文件提到落后支，见 0.11）；(2) 在崩溃点重放的崩点二上量出「记录只在盘 A」的状态并不存在（则丙是多余的，丁够）；(3) 发现旧快照盘可以带着本版末条记录而缺两份以上单元（写序前缀假设不成立）。

## S3　C707：fsid 不符的槽落 R2 哪一边

今天的实现：槽解得出而 fsid 不同，不进自证列表也不计读错，「== *filesystem_identifier」（`crates/singlefs-core/src/recovery.rs:4271`）为真才「self_verified.push(system_configuration);」（`crates/singlefs-core/src/recovery.rs:4273`），没有 else 分支；撕裂的槽走另一条，会记进读坏账。
关键的前置事实（读代码得到的）：R2 之前还有择系统配置这一步。它按每盘取世代号最大的可择槽（不看 fsid），再比各盘之间的 fsid，对不上整池拒：「return Err(RecoveryFailure::SystemConfigurationsDisagree);」（`crates/singlefs-core/src/recovery.rs:1448`）。
所以：整盘换成别池的盘、或只换一槽而那一槽世代号更高，都在 R2 之前就整池拒（只读也拒）；只有「别池的槽世代号不高于本盘自己的槽」才走到 R2（推的，按 `crates/singlefs-core/src/recovery.rs:1427` 起的 `(Some(zero), Some(one))` 选法）。

| 盘面 | 落「自证不过」一边 | 落「读错」一边 |
|---|---|---|
| 两盘世代 5 都读回全零（撕裂的轮换） | 判得出，可写 | 同（不涉及 fsid） |
| 整盘换成别池的盘 | 择系统配置整池拒 | 同 |
| 只换一槽、别池槽世代号更高 | 整池拒 | 同 |
| 只换一槽、世代号不高；另一盘两槽自证过 | 判得出，可写 | 判得出（另一盘两槽自证过，R2 第二项为假），可写 |
| 只换一槽、世代号不高；另一盘一槽读回全零（无设备报错） | 判得出，可写，取号写重写那一槽 | 判不出，落 S1 |

### S3 候选与代价
- 候选一（自证不过，今天的实现）：不改 `crates/`。要补的是用例：「只换一槽、世代号不高」判得出、可写挂载做成、取号写重写那一槽；「别池的槽世代号更高」整池拒（`SystemConfigurationsDisagree`）。今天没有用例（`.claude/kb/checks-owed.md:466`）。
- 候选二（读错）：`crates/singlefs-core/src/recovery.rs:4271` 那个 `if` 加 `else` 计数（新增一个字段，不并进 `slots_reporting_a_read_error` 的名字）；谓词 `system_configuration_witness_is_undecidable`（`crates/singlefs-core/src/recovery.rs:4306`）的第一项收这个字段。谓词的使用点三处：读阶段（`crates/singlefs-core/src/mount.rs:7638`）、取号读见证值（`crates/singlefs-core/src/transaction.rs:878`）、独立判官（`crates/singlefs-harness/src/mount_transparency.rs:2388`），都调同一个函数，所以改一处；判官与 e165 装置的复算要核。翻面的只有上表最后一行，那一行落进 S1。
- 候选三（整池拒）：择系统配置那一步把「任何一槽 fsid 与择到的不同」都报 `SystemConfigurationsDisagree`。连只读挂载也拒，今天能只读挂载的池会挂不上；上表第 4、5 行都变成整池拒。
- 与 S1 怎么配：S1 推荐的 α 把「不自证」的槽（撕裂、全零、别池的槽）一视同仁地重写，所以候选一或二都不改变 α；候选二只是让更多盘面需要走 α。

### S3 一句话条款（推荐候选一）与推翻条件
条款句：系统配置见证读里，读得出、整槽校验和过而 fsid 与本池不同的槽按「读得出而自证不过」算、不算读错，落在「判得出」一边；它若是所在盘世代号最大的可择槽，择系统配置那一步已按 fsid 对不上整池拒。
理由（推的）：R2 把撕裂（读得出、自证不过）放在判得出一边，靠的是「轮换排在根槽 FUA 之后，撕裂的那次发布的根已持久」；别池的写进来的槽没有这条顺序保证，与位腐烂成的垃圾槽是同一类，R2 已经把垃圾槽放在判得出一边，候选一与它一致；整盘换别池的盘根本到不了 R2。
什么现象会推翻它：在 C331 网格式的历史上，把「只换一槽、世代号不高、另一盘一槽读回全零」这一格放行后量出丢了确认过的写（那样候选二才站得住）。

## S4　C463：一块盘两槽全废之后的自愈重写

今天的实况：恢复择系统配置时跳过这块盘，报告点名它（`(None, None) => {`，`crates/singlefs-core/src/recovery.rs:1427`；只读挂载有同名字段，`crates/singlefs-core/src/mounted_read.rs:607`），这是 C463 已做的一半（`.claude/kb/checks-owed.md:323`）。
可写挂载在两处拒：取号之前的逐盘核第一支「return Err(device);」（`crates/singlefs-core/src/mount.rs:5731`），映射成 `SystemConfigurationUnreadableAfterTheReadStage`（`crates/singlefs-core/src/mount.rs:6575`）；取号读见证值时「if slots_of_this_device.self_verified.is_empty() {」（`crates/singlefs-core/src/transaction.rs:893`）。
条款侧两句要连着看：「某块盘一份自证过的系统配置都没有时，取号那一步也拒」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`，用户 2026-10-03 定）；「可见」的定义「= 独占打开成功且系统配置读得通」（`.claude/kb/decisions/18-块里携带什么信息.md:319`）。
拒的理由写在代码里：不拒的话「一块换上去的空盘让这些入口照样做成、把本池的写发到空盘上」（`crates/singlefs-core/src/mount.rs:716`）。所以任何自愈要先分得清「本池的盘、系统配置毁了」与「空盘、别池的盘、对调的盘」。

自愈重写与取号重写是同一个写：取号写本来就逐盘写一次（0.5），在一块没有自证槽的盘上 `write_system_configuration_slot` 取世代号 1、写槽 1（0.4）；另一槽由第一次发布末尾的轮换补上（世代号 2，槽 0）。不需要新的写函数，需要的是放行两处拒。

### S4 候选一：在恢复里写
- 做不到的原因是类型（0.6）：`choose_system_configuration_naming_devices_without_a_mountable_slot(reader: &dyn PoolReader)`（`crates/singlefs-core/src/recovery.rs:1387`）与整条恢复链没有写能力；要写就得给恢复链一个带写的读者类型，或在恢复返回之后另补一段。后一种就是候选二的位置。代价最大，无收益。

### S4 候选二：可写挂载取号时写（推荐）
- 判「这是本池的盘」：把落后支的单元比对搬来用——这块盘没有自证槽，按 tail 为无来判落后，然后要求所选那一版的单元清单非空、全部单元在这块盘上读得到且字节与验得过的那一份相同、根槽那一半不缺（`what_a_device_behind_the_version_lacks`，`crates/singlefs-core/src/mount.rs:5824`，不看 tail 的变体）。空盘、别池的盘缺单元，照拒；单元清单为空（没有可比的东西）照拒。
- 要动：`crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`（`crates/singlefs-core/src/mount.rs:5713`）返回值多一类「可补写」，取号之前那一步（`crates/singlefs-core/src/mount.rs:6566` 起）按这三类分流；`crates/singlefs-core/src/transaction.rs` 的 `self_verified_system_configurations_of_every_device`（`crates/singlefs-core/src/transaction.rs:861`）收一个「已核过可补写的盘」集合，空列表只在集合里放行，并且写之前重算时集合必须不变（照 `acquire_expected_instance`「判定与写读同一份输入」的做法，`crates/singlefs-core/src/transaction.rs:1045`）；`MountOutput`（`crates/singlefs-core/src/mount.rs:982`）加一个「补写过系统配置的盘」字段；`crates/singlefs-harness/src/mount_transparency.rs` 的判官对应的拒因复算。
- 写出的新形态：无（盘上格式不变）；盘上多出来的状态是那块盘槽 1 世代号 1、槽 0 仍坏，直到第一次轮换。
- 写之前核的：见上；还有读阶段判得出（S4 只落在判得出的盘面里：另一盘两槽自证过，或没有设备读错；判不出的归 S1）。
- 写失败怎么报：沿用取号失败的回卷（`crates/singlefs-core/src/transaction.rs:656` 起的 `roll_back_acquisition`），报 `InstanceAcquisitionFailed::Acquisition`；挂载拒可写，调用方转只读挂载；补写失败的盘仍是两槽全废，不比开始更坏。
- 代价：1 个函数的返回值分三类、1 个集合参数贯穿取号三个入口（`crates/singlefs-core/src/transaction.rs:987`、`crates/singlefs-core/src/transaction.rs:1045` 与 `read_the_witness_before_the_expected_acquisition_writes`）、1 个报告字段、约 5 条用例（本盘数据完好的补写做成并读回确认过的写、空盘照拒、别池的盘照拒、对调的盘照拒、写失败回卷）+ 变异行。现有那条断言「可写挂载维持拒」的用例要反过来（`crates/singlefs-harness/tests/system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots.rs` 与 `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs`，是否都断言了拒要现查）。
- 与条款：改动 D22 已定项 8 射程那一句「取号那一步也拒」（用户 2026-10-03 定）与 D18 已定项 11 的「可见」，两句都要带例外重写；这是冲突点，不是补空白。

### S4 候选三：不自愈，出路并进 S1 的 α
- 做法：α 的写函数放宽「每盘至少一槽自证」，对两槽全废的盘先写一槽、屏障、再写另一槽；写之前仍要核上面的「本盘带所选那一版」，否则 α 会把一块空盘登记成池成员。
- 代价：取号路径不动；但每次这样的盘面都要人调用，冗余缺失期间池只能只读。

### S4 与 S1 是不是同一个机制
同一个写（`write_system_configuration_slot` 落到不自证的槽），不同的闸：S4 的闸是「见证判得出 + 本盘带所选那一版」，自动、在取号里；S1 的闸是「见证判不出 + 调用方断言版本」，手动、在挂载之外。库里可以只有一个「重写不自证槽」的内部函数，两个入口各自带闸。

### S4 一句话条款（推荐候选二）与推翻条件
条款句：可写挂载读系统配置的见证判得出时，若某块盘两个系统配置槽都不自证、而这块盘带所选那一版的全部单元（单元清单非空、逐个读到且字节与验得过的那一份相同）且根槽那一半不缺，则取号那一写把它的系统配置补写回去（该盘取世代号 1 写槽 1，另一槽由第一次发布末尾的轮换补上），挂载结果点名补写的盘；带不全所选那一版的盘，取号那一步照拒；补写写失败按取号失败回卷，挂载拒可写。
什么现象会推翻它：(1) 造一块空盘、别池的盘或对调的盘，补写居然做成了（则「带所选那一版」的判不够）；(2) 用户重申 2026-10-03 的「一份自证过的都没有就拒」不含例外；(3) 补写之后到第一次轮换之间的状态，池级 checker 有一条红（I-7.15、I-7.16 是最该先看的两条）。

## S5　C555：两盘最新系统配置槽都坏时取号撞号

条款侧：取号式子是「取新代号 = max(**这次挂载独占打开成功的那个集合**里各系统配置的代号」加根环里全部根记录的实例代号，再加一（`.claude/kb/decisions/18-块里携带什么信息.md:319`）；代码在「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`crates/singlefs-core/src/transaction.rs:941`）。
不变量侧：I-7.7 的前一半已经把 journal 记录与单元写序算进来：「① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者」（`crates/singlefs-checker/invariants/02-根环.md:14`）。
所以式子只保证「根环里的号 < 新号」，不变量要求「journal 记录与单元的号 ≤ 系统配置里的最大号」，两者之间的缝就是 C555：新号只活在各盘最新那一槽里，直到第一次发布末尾的轮换才进另一槽（推的，按 0.4 与 0.5 的写序：取号写落在较旧的那一槽，轮换在根槽之后）。
最小历史的第 3 步翻坏的正是每盘最新那一槽，退回的旧槽还是旧号，第 4 步取到同一个号。行里写的补丁量过：「取号式子多读 journal 环一路的补丁在调查副本上 225 格 0 红，只写了单元就崩的那几格照样撞号」（`.claude/kb/checks-owed.md:362`）。

D22 已定项 8 射程把承诺写成「每盘一槽系统配置坏在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。C555 的第 3 步是每盘一槽坏（最新那一槽），不带「最新根一时读不出」，所以它落在这句承诺的字面里，不像 C583 那样落在被排除的双故障形里。这是候选一要面对的冲突。

### S5 候选一：照 C583 先例写成「不罩」
- 要动：D22 已定项 8 射程那句承诺要加例外，`crates/` 不动。条款句草稿见下。
- 代价：0 行代码；检查器在这一形的镜像上照红（C555 行写的是 I-8.6；I-7.7 前一半按字面也该红，推的），条款要写明这些红是有意的；崩溃点重放不带「取号之后的写落盘后两盘最新槽持续坏」这种故障，所以不会红（推的）。
- 与单故障承诺的关系：例外把「每盘最新一槽」从承诺里拿掉，而「最新一槽」恰是撕裂轮换最常落的那一槽；也就是说承诺的字面从「每盘一槽」缩成「每盘一槽且不是取号之后第一次轮换之前的最新槽」，缩得不小。
- 罩不住：整个这一形，包括没有任何记录、只有单元的状态。

### S5 候选二：取号式子并进 journal 环（以及别的来源）
- 要动：`crates/singlefs-core/src/transaction.rs` 的 `next_instance_generation_to_acquire`（`crates/singlefs-core/src/transaction.rs:931`）多一路 journal 记录的最大实例代号；读阶段已经扫过环（`records`，`crates/singlefs-core/src/mount.rs:8074`），所以把读阶段的最大实例代号当参数传进来，比在取号里再扫一遍 1.5 GiB 便宜。调用点三处：`crates/singlefs-core/src/mount.rs:6415`、写之前重算那一处（`crates/singlefs-core/src/transaction.rs:1063`）、mkfs 同进程那条流（`crates/singlefs-core/src/transaction.rs:987`，环是空的）。
- 写出的新形态：无。
- 代价：1 个参数贯穿取号三个入口；用例约 4 条。运行时不多读。
- 罩不住：只写了单元就崩的历史（单元头上的写序实例代号只在单元里，式子看不到）；把单元写序也并进来要整盘扫单元区，不在读阶段已读的范围内（推的，没量）。所以单做它，C555 最小历史能修掉的是「崩在第一条记录持久之后」那一半。

### S5 候选三（第三种）：取号写两次，两个槽都带新号
- 做法：取号写分两轮，每轮逐盘各写一次：第一轮每盘写世代号 g+1（落较旧那一槽），一道池屏障，第二轮每盘写世代号 g+2（落刚才较新的那一槽，两轮都读盘算世代号，`crates/singlefs-core/src/transaction.rs:559` 起的函数本来就这么算），再一道屏障，然后才交出新号、才动任何单元。
- 为什么两轮之间要屏障：第二轮覆写的是原先最新的那一槽，要「覆写旧槽时新槽已持久」，与取号写之前那道屏障同一个理由（`crates/singlefs-core/src/transaction.rs:1129` 起的注释）。
- 结果：取号返回时每盘两槽都带新号。之后每盘坏一槽，另一槽仍带新号，式子读到的最大号不回退；C555 最小历史第 3 步翻坏每盘最新一槽之后，每盘仍有一槽带 2，第 4 步取 3，不撞。轮换不改实例代号，所以此后两槽一直同号。
- 要动：`crates/singlefs-core/src/transaction.rs` 的 `write_the_witnessed_acquisition`（`crates/singlefs-core/src/transaction.rs:1161`）的逐盘循环（`crates/singlefs-core/src/transaction.rs:1171`）；回卷 `roll_back_acquisition`（`crates/singlefs-core/src/transaction.rs:656`）要回卷两轮，否则每盘一槽仍带较大的新号，I-7.7 的第二句判成立（`crates/singlefs-checker/invariants/02-根环.md:14` 的「取号没做完、全或无失败回卷期间的代号不等由 ② 判成立」）而 D18 已定项 11 的字面是「先把已经写出的那几份回卷成旧代号」（`.claude/kb/decisions/18-块里携带什么信息.md:319`），推荐照字面回卷两轮。
- 写出的新形态：无（同样的槽，多写一份）；每次挂载多 2 次 4096 字节写和 1 道屏障。
- 牵连面（树副本里数的）：每次取号后的世代号整体加 1。写死世代号 5 或叫 `NEW_POOL_FILE_CREATION_ROTATION` 的文件 9 个，提到 `slot_generation` 的 39 个；`.claude/kb/layout/01-first-txn.md` 的字节表里取号、暖机、事务三行的世代号与「屏障 3 道」（该表 a1 行在第 60 行：「a1 | 第一次可写挂载取号」）；E142 的新池新建文件干跑按 21 个区域逐字节比对要重钉；崩溃点重放的状态数变多（不收窄，照多枚举一档加）。
- 罩不住：同一块盘两槽都坏加另一块盘也坏的四故障；取号写两轮之间崩（此时没有单元、记录带新号，写出去的新号无人引用，I-7.7 的第二句已判成立）。

### S5 一句话条款
推荐候选三的条款句：取号写分两轮，每轮逐盘各写一次系统配置、每轮之后一道池屏障，两轮都完成之后才交出新实例代号、才动任何单元；任一轮的写或屏障报错，已写出的各份回卷成取号之前这个集合里全部自证槽中最大的实例代号。
候选一的条款句（交用户时的对照）：D22 已定项 8 射程的承诺句改为「容错承诺是单故障：每盘一槽系统配置坏在承诺里，但每盘最新一槽在取号之后的写落盘之后都持续坏、而新实例代号只在这些槽里的那一形（取号之后、第一次发布末尾的轮换之前崩）不在承诺里，检查器在这一形上红是有意的」。
推荐候选三的理由（推的）：它不改承诺的字面，由构造保证式子读到的最大号不会因为每盘坏一槽而回退；候选二单做罩不住只写单元就崩的格；候选一要缩承诺。
什么现象会推翻它：(1) 取号写两轮之后，造「第一次发布之前崩 + 每盘最新一槽翻坏一字节」的盘面，下一次取号仍与某条记录或某个单元的实例代号相同；(2) 崩溃点重放里取号两轮之间的状态有池级 checker 红；(3) 用户认为承诺本来就不含「每盘最新槽持续坏」，那时候选一最便宜。

## S6　C569：反向链遇上前一槽的残留记录

### 两边今天各判什么（读代码得到的）
- 写路径：新实例的第一条记录反向链写死 0（写行那次发布，「back_chain: 0,」，`crates/singlefs-core/src/mount.rs:5037`；暖机，「back_chain: previous_record_bytes.as_deref().map_or(0, back_chain_of),」，`crates/singlefs-core/src/transaction.rs:1266`），同一次发布里之后每条接前一条的头（「let back_chain = match written_records.last() {」，`crates/singlefs-core/src/transaction.rs:2149`）。写路径不读盘，对上条款「本实例写出的第一条记录恒 0」（`.claude/kb/decisions/23-journal的角色与格式.md:532`）。
- 恢复：前一条按键 (实例代号, 计数器 − 1) 在全池扫环的结果里找，找不到就「不判」，「let Some(previous) = records.get(&(record.instance, previous_counter)) else {」（`crates/singlefs-core/src/recovery.rs:4401`）；计数器 1 也不判（`crates/singlefs-core/src/recovery.rs:4399`）。所以恢复从不核「第一条链为 0」，只在同实例前一条在盘上时核链值。
- 池级 checker：逐盘、按计数器找前一条。「FIRST_JOURNAL_COUNTER => ExpectedBackChain::Value(0),」（`crates/singlefs-checker/src/walk.rs:9050`）；前一条是同实例 ⇒ 链值（`crates/singlefs-checker/src/walk.rs:9055`）；「Some(_record_of_another_instance) => ExpectedBackChain::Value(0),」（`crates/singlefs-checker/src/walk.rs:9058`）；找不到 ⇒ 跳过（`crates/singlefs-checker/src/walk.rs:9059`）。
  它给「前缀进不进得去」另有一个与恢复对齐的函数 `record_can_enter_a_replay_prefix`（`crates/singlefs-checker/src/walk.rs:9089`），按 (实例代号, 计数器 − 1) 在全池结果里找，前一条不在不判。所以 C569 只涉及 I-8.6 本身那一判，不涉及前缀。
- 条款：「本实例写出的第一条记录反向链恒 0」，同一行还有「⚠️ 链按实例算，所以跨实例边界的两条记录之间不判这一条」（`crates/singlefs-checker/invariants/03-journal.md:13`）。两句并排：前一句把「前一槽是别的实例」那一格判成链恒 0，后一句说跨实例边界不判链值；checker 取前一句，恢复取后一句。

### 盘面对照（推的，没造镜像量；Y 是新实例，X 是上一条时间线）

| 盘面（同一块盘） | 恢复 | 池级 checker |
|---|---|---|
| 计数器 1，链不是 0 | 不判 | 红 |
| Y 的第 c 条，同盘第 c−1 条是 Y 的 | 核链值 | 核链值（一致） |
| Y 的第 c 条，第 c−1 条是 Y 的但只在另一块盘上 | 核链值（全池结果里找得到） | 跳过 |
| Y 的第 c 条链不是 0，同盘第 c−1 槽坐着 X 的、计数器也是 c−1 的自证记录（C569） | 不判 | 红 |
| Y 的第 c 条，第 c−1 槽空、读错、或坐着上一圈的记录（计数器对不上） | 不判 | 跳过（一致） |
| 计数器 0 | 进不了前缀 | 红（用户 2026-09-27 定，`crates/singlefs-checker/invariants/03-journal.md:13`「jsn 的计数器为 0 的记录判违例」） |

### 这一格在今天的写路径上造不造得出来（推的）
新实例第一条的计数器是环里读得出的最大计数器加一（`crates/singlefs-core/src/mount.rs:8074`，对上条款「前缀末的计数器取环里读得出的最大那一个」，`.claude/kb/decisions/23-journal的角色与格式.md:371`）；扫环时读不出的槽被静默跳过（`crates/singlefs-core/src/recovery.rs:4348`）。
所以 X 在计数器 f 上留下的自证记录，只要 Y 挂载时读得出，Y 的第一条就编在 f 之后，不会与它同号。盘面第 4 行要成立，X 的那条残留记录在 Y 挂载扫环时必须在每块盘上都读不出（读故障），之后又读得出。
没有读故障的崩溃点重放状态里造不出这一格（推的）。造它至少五步：① X 写下计数器 f 的记录且落盘，但不在前缀里（崩）；② Y 挂载，f 那一槽在两块盘上读错，Y 的第一条编成 f；③ Y 的发布写 f 与 f+1，崩，本盘只持久 f+1（记录逐盘逐条写、中间没有屏障，0.8）；④ 读故障消失；⑤ 对本盘镜像跑池级 checker。第 ② 步挂载会不会在别处被拒（F1 乙、N-配置）要造出来才知道，没造过。
今天没有实例切换的实现（0.10），那条路径上的同号问题不存在；M4 若加实例切换，要回头看这一格。

### 候选一：收
- 做法：checker 前一条的找法改成恢复的找法：按 (实例代号, 计数器 − 1) 在全池结果里找（复用 `records_as_the_recovery_reads_them`，`crates/singlefs-checker/src/walk.rs:9068`），找得到才判链值，找不到一律跳过；计数器 1 仍判恒 0（计数器 1 之前不可能有别的记录，它确是全池第一条）。
- 动：`crates/singlefs-checker/src/walk.rs` 的 `expected_back_chain_of`（`crates/singlefs-checker/src/walk.rs:9043`）与 `judge_journal_back_chain` 的调用（`crates/singlefs-checker/src/walk.rs:9145`）；`crates/singlefs-checker/invariants/03-journal.md` 第 13 行 I-8.6 的文字；`crates/singlefs-harness/tests/checker_known_bad_images.rs` 里钉「别的实例之后的第一条链不为 0」的坏镜像（`fn the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant() {`，`crates/singlefs-harness/tests/checker_known_bad_images.rs:2911`；是否含这一格要现查）与对应变异行。
- 写出的新形态：无。
- 一致性：上表第 3 行也对齐（checker 会判），第 4 行对齐（都不判）；第 1 行仍是恢复不判、checker 红。
- 代价：失去唯一的哨兵——「新实例第一条记录链不为 0」这条条款（`.claude/kb/decisions/23-journal的角色与格式.md:532`）在计数器大于 1 的第一条上没有任何东西再检查（恢复本来就不查）。「按实例表认」不能补：实例表的行记录是「kind 1 | 实例代号 4 | 所选根的 checkpoint」打头的几个字段（`.claude/kb/decisions/18-块里携带什么信息.md:304`），已发布谓词里用的是 (i, T_pub, W)（`.claude/kb/decisions/18-块里携带什么信息.md:315`），都不记第一条的计数器。
- 罩不住：写路径错写非 0 的第一条链（计数器大于 1）。

### 候选二：不收
- 做法：I-8.6 射程写明「前一槽（计数器 − 1）坐着别的实例的自证记录时，本条按『本实例第一条、链恒 0』判，红是有意的」，并写上造得出来的条件（上一节第 ②–③ 步，即读故障加子集持久）。
- 动：`crates/singlefs-checker/invariants/03-journal.md` 第 13 行一句话；`crates/` 不动；补一条钉住这一格结局的用例（造上面五步的镜像，断言 checker 红、恢复不判、两边结局各自如表）。
- 代价：这一格上 checker 对一个恢复读得过去的状态报红，条款要承认这是有意的；哨兵保留。
- 罩不住：读故障历史里造出这一格时，checker 红而池并没有写错（假阳性），要靠条款文字解释。

### S6 一句话条款（推荐候选二）与推翻条件
条款句：I-8.6 的「本实例写出的第一条记录反向链恒 0」按计数器认——计数器 1，或前一槽（计数器 − 1）坐着别的实例的自证记录——并写明：恢复只在同实例前一条在盘上时核链值，不核第一条恒 0，两边在「前一槽坐着别的实例的记录」这一格上判法不同是有意的；这一格只在新实例挂载扫环时那条记录在每块盘上都读不出的历史里才造得出。
推荐候选二的理由（推的）：造得出来只靠读故障加子集持久，没有读故障的崩溃状态里不会红；候选一为了一个读故障才有的假阳性拆掉唯一的哨兵。
什么现象会推翻它：(1) 在没有读故障的崩溃点重放状态里找到第 4 行盘面（则 `next_counter` 的论证不成立，红是真问题，候选一或写路径要改）；(2) M4 加了实例切换并且切换的计数器不取环里最大值加一；(3) 读故障加子集持久的历史被列进要罩的故障模型。

## 岔路单草稿行（候选 / 翻面观测 / 够判条件；不带倾向，不写已有的数）

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| S1 R2 之后仍判不出的盘面怎么走出只读 | α1 管理员定点重写，带调用方断言的版本；α2 同上，工具直接按读出的版本写、无断言；β 修复挂载（读阶段同 α，包装成第三种挂载）；γ 可写挂载在「根环槽自证或全零、journal 环无读错、每盘一槽自证」下放行；δ 接受永远只读 | 在 C331 网格（E165 择根与重放合判族）上，γ 条件放行的格数，以及其中丢确认过的写的格数；一块持续读错的槽被 `write_system_configuration_slot` 写一次的结果（成功 / 设备报错） | 入库装置上量出 γ 的放行格数与丢写格数两个绝对数，量出持续读错槽的写入结果；用户定「出路是否允许靠人断言」 |
| S2 落后且缺一份单元的盘 | 甲 维持拒；丁 落后与否看本盘环槽里有没有那一版末条记录（只改缺单元那一半）；丙 缺的单元份数不少于 min(2, 单元总数) 才算缺；丁丙 两样都加 | `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 全部格在各候选下由拒变放的格数；崩溃点重放里「记录只在盘 A、盘 B 没有」的状态是否存在 | 上面两个数在入库装置上量出；新造「崩点一 / 二 / 三 × 本盘一份单元坏」的盘面，在各候选下的结局表 |
| S3 fsid 不符的槽落 R2 哪一边 | 自证不过一边（今天的实现）；读错一边；整池拒（任何槽 fsid 与择到的不同即拒，只读也拒） | 「只换一槽、世代号不高、另一盘一槽读回全零」这一格放行后，C331 式历史上丢确认过的写的格数 | 该格放行格数与丢写格数两个数；六种盘面（撕裂、整盘换、只换一槽高 / 低世代号、另一盘撕裂 / 读错）在三个候选下的结局表 |
| S4 一块盘两槽全废之后的自愈重写 | 取号时补写（以「本盘带所选那一版的全部单元」为闸）；在恢复里写；不自愈、出路并进 S1 | 空盘、别池的盘、对调的盘三种被闸误放行补写的格数；补写之后到第一次轮换之间池级 checker 红的条数 | 两个数在入库用例里量出；用户定是否给 D22 已定项 8「取号那一步也拒」开例外 |
| S5 两盘最新系统配置槽都坏时取号撞号 | 写成「不罩」（改承诺句）；取号式子并进 journal 环（及单元写序）；取号写两次、两槽都带新号 | C555 最小历史及「只写了单元就崩」那几格在各候选下撞号的格数；取号写两次对崩溃点重放状态数的增量 | 三个候选各在入库装置上跑一次 C555 全格，报撞号格数；用户定承诺是否含「每盘最新一槽取号之后持续坏」 |
| S6 反向链遇上前一槽的残留记录 | 收（checker 的前一条找法改成与恢复同键）；不收（I-8.6 射程写明这一格判违例是有意的） | 没有读故障的崩溃点重放状态里，I-8.6 在「前一槽坐着别的实例的记录」这一格红的状态数；带读故障的历史族里这一格出现的状态数 | 两个数量出；M4 是否加实例切换及其计数器起点 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| S1 | 规则没说；候选 δ 与 D22 已定项 8 射程字面冲突 | R2 之后判不出的盘面至少每盘一槽出问题，其中一部分落在「每盘一槽坏在承诺里」的字面里；出路条款全仓没有；推荐 α（带版本断言的库函数），γ 要先量 |
| S2 | 规则没说 | 「只落后」与「只缺单元」两样同时出现条款没写；三个正常崩溃点各加一份单元坏都被永久拒；推荐丁丙，调用点不动 |
| S3 | 规则没说 | 别池的槽世代号更高或整盘换盘在 R2 之前就整池拒；能走到 R2 的只有世代号不高的单槽；推荐落「自证不过」，补用例 |
| S4 | 规则没说；放行补写与 D22 已定项 8「取号那一步也拒」冲突 | 自愈与取号写是同一个写，闸要换成「本盘带所选那一版」；恢复链类型上写不了，只能放在可写挂载取号里 |
| S5 | 规则没说（式子与 I-7.7 之间有缝）；候选「不罩」与承诺字面冲突 | 这一形不带「最新根读不出」，落在「每盘一槽坏」的字面里；推荐取号写两次，牵连面大 |
| S6 | 规则没说；I-8.6 同一行两句并排，恢复与 checker 各取一句 | 今天的写路径只在带读故障的历史里造得出这一格；推荐不收、射程写明 |

## 复核

- 引文核对（树副本做根）：`python3 scripts/agents/cite-check.py research/prompts/m3-close-sysconf-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree --root /home/fy5090/code/singlefs`。
  不带树副本根时，主工作区 `crates/` 此刻已漂，会有行号错一两行的红，那是主工作区改动造成的，不是引文错。
- 数出来的数（树副本）：`grep -rl 'fn main' crates --include=*.rs | grep -v 'singlefs-checker-tier' | grep -v unit-graph | wc -l` 输出 `0`；
  提到 `acquire_instance` 或 `acquire_expected_instance` 的 `.rs` 文件 67 个；提到 `system_configuration_witness_is_undecidable` 的 8 个；提到落后支三个名字的 12 个；写死世代号 5 或 `NEW_POOL_FILE_CREATION_ROTATION` 的 9 个；提到 `slot_generation` 的 39 个。

## 没做什么

- 没编译、没跑测试、没造镜像、没量任何格数。凡写「推的」的结论都只是读代码与条款推的；岔路单里的「翻面观测」都是还没量的数。
- 没写模型，没有模型目录，没有要主 agent 复跑的命令。
- 没读禁读清单里攻方腿的任何产出。
- 没替主 agent 采纳或出判决；每格的「推荐」只是正推腿从实现推出来的一种落法，用户该定的取舍在岔路单里。
- 没核 `entries_after_mount_refuse_swapped_or_behind_devices.rs` 与 `system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots.rs` 两个文件里每一格现在断言什么（只引了它们的名字与用途），写进「要动哪些用例」的部分是按文件名与注释推的。
- 没核 `RootRingSlotReadingTellingNeverWrittenApart` 的读错与读成垃圾两类能否分开（它把「设备报读错，或读得出、不是全零而自证不过」并成一类，`crates/singlefs-core/src/mount.rs:2711` 的注释），所以 γ 的条件「环读干净」按这一类算，比字面更严。
