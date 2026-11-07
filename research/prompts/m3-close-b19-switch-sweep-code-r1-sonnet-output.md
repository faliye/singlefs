# m3-close-b19-switch-sweep-code-r1 正推腿报告（Sonnet）

立场：正推，代码做的是不是条款说的。方案按 `crates/` 今天的实现来谈。只答 Q1 三问。
读的文件（sha256 与开工快照一致）：`crates/singlefs-core/src/mount.rs` 7aeaa0980972…、`crates/singlefs-core/src/instance_table_sweep.rs` bbff4180df82…（`sha256sum` 与 `crates-sha256.txt` 前两行逐位相同）。

标记：【读码】= 读代码行坐实；【推】= 由读码推出，没跑过；【条款没说】= 条款里没有这一句。

## 背景材料里一处与代码相反的话（先说）

背景正文第二节写「取 N = 读得出的根里最小的 txg」（`research/prompts/_m3-close-b19-switch-sweep-code-r1-body.md:27`）。代码取的是最大：「.max()」（`crates/singlefs-core/src/mount.rs:1576`），登记也写最大：「的根的最大 txg」（`research/prompts/e205-preregistration.md:43`）。下面按代码与登记逐句比，不沿用「最小」。

## Q1.1 开关两档的算法与登记「甲并全零」「甲只并读坏」逐句对

登记定义原文在 `research/prompts/e205-preregistration.md` 第 41、42、43 行（甲两档各一行，第 43 行是两档共用的取法）。一句一格：

| # | 登记的一句 | 代码 | 判 |
|---|---|---|---|
| 1 | 「甲并全零：挂载那一刻读不出、读得出不是全零而自证不过、读回全零的根环槽都算」（`research/prompts/e205-preregistration.md:41`） | 档名对到 `RingSlotsTheFirstTxgGoesPast::ReadBadOrReadingBackAllZeroAtMount`（`crates/singlefs-core/src/mount.rs:1565` 起的分派）；选槽：「(BadAndNotAllZero, ReadBadOrReadingBackAllZeroAtMount) => Some(*ring_slot),」（`crates/singlefs-core/src/mount.rs:1612`）与「(AllZeroNeverWritten, ReadBadOrReadingBackAllZeroAtMount) => Some(*ring_slot),」（`crates/singlefs-core/src/mount.rs:1614`）；自证过的槽不算：「(SelfVerified(_), ReadBadAtMount | ReadBadOrReadingBackAllZeroAtMount) => None,」（`crates/singlefs-core/src/mount.rs:1610`）。`BadAndNotAllZero` 的来路：设备读不出（`crates/singlefs-core/src/mount.rs:3021`）、读得出非全零而 `parse_slot` 不过（`crates/singlefs-core/src/mount.rs:3040`）、自证过而落点不合（`crates/singlefs-core/src/mount.rs:3031` 起到 3037）；全零：「if bytes.iter().all(」「== 0) {」（`crates/singlefs-core/src/mount.rs:3023`）。 | 对得上【读码】。「挂载那一刻」这一句见下方 D 项。 |
| 2 | 「甲只并读坏：只算读不出、读得出不是全零而自证不过的根环槽。」（`research/prompts/e205-preregistration.md:42`） | 「(BadAndNotAllZero, ReadBadAtMount) => Some(*ring_slot),」（`crates/singlefs-core/src/mount.rs:1611`），读回全零：「(AllZeroNeverWritten, ReadBadAtMount) => None,」（`crates/singlefs-core/src/mount.rs:1613`）。 | 对得上【读码】。 |
| 3 | 「N = 这次挂载读根环读得出、自证过（含落点公式核）的根的最大 txg（与 `first_txg_of_new_instance` 里 `highest_root_txg` 是同一个量）」（`research/prompts/e205-preregistration.md:43`） | N 取一遍根环读数里 `SelfVerified` 的 `checkpoint_txg` 的最大：「SelfVerified(root) => Some(root.checkpoint_txg),」（`crates/singlefs-core/src/mount.rs:1573`）接「.max()」（`crates/singlefs-core/src/mount.rs:1576`），没有时 0（`crates/singlefs-core/src/mount.rs:1577`）。`SelfVerified` 已含落点核：「if !root_sits_in_the_ring_slot_the_formula_gives(」（`crates/singlefs-core/src/mount.rs:3031`）。今天的 `highest_root_txg`（`crates/singlefs-core/src/recovery.rs:2073`）经 `read_root_ring_slot` 读，那里同样先核落点：「if !root_sits_in_the_ring_slot_the_formula_gives(」（`crates/singlefs-core/src/recovery.rs:2010`）；两边都不按实例表滤被抛弃的根，没有时都 `unwrap_or(0)`（`crates/singlefs-core/src/mount.rs:1511`）。 | 对得上【读码】：取最大、含落点核、不滤被抛弃、空时 0。「同一个量」的前提是两遍读数相同，见 D 项。 |
| 4 | 「H_槽 = [N + 1, N + 3S − 1] 里按 `target_for_publish` 落到这一槽的那个 txg」（`research/prompts/e205-preregistration.md:43`） | 环长：「let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();」（`crates/singlefs-core/src/mount.rs:1636`），`ROOT_RING_REGIONS` 取 3：「pub const ROOT_RING_REGIONS: u64 = 3;」（`crates/singlefs-format/src/lib.rs:291`）；偏移从 1 走到 3S − 1：「for offset_past_the_highest_root in 1..ring_length_in_txgs {」（`crates/singlefs-core/src/mount.rs:1638`），即 txg 从 N + 1 到 N + 3S − 1（Rust 半开区间，上端 3S 不取）；落槽：「if ring_slots_passed.contains(&target_for_publish(CheckpointTxg(txg), slots_per_region)) {」（`crates/singlefs-core/src/mount.rs:1645`）；从小到大走，最后一次命中就是最大：「highest_txg_a_passed_ring_slot_can_hold = Some(CheckpointTxg(txg));」（`crates/singlefs-core/src/mount.rs:1646`）。 | 对得上【读码】。 |
| 5 | 「3S 是环长；这一段里每个槽恰好落一个 txg，`N` 自己那一槽不在里面」（`research/prompts/e205-preregistration.md:43`） | 「每个槽恰好一个」是对公式的断言：`target_for_publish` 取「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」与「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:172-173`），区域是 txg 模 3、槽是 txg 除 3 再模 S，等价于 txg 模 3S 一一对应，所以任意 3S 个连续 txg 恰好各落一槽，窗口取其中去掉偏移 0（N 自己）的 3S − 1 个【推：算术，没跑】。N 自己那一槽不会在算进来的集合里：集合只收非 `SelfVerified` 的槽（`crates/singlefs-core/src/mount.rs:1610`），而住着 N 那条根的槽读数就是 `SelfVerified`【读码】。 | 对得上。N 自己那一槽不在集合里是【读码】，一一对应是【推】。 |
| 6 | 「甲给的第一个 txg = max(今天的值, max H_槽 + 1)，没有算进来的槽时等于今天的值」（`research/prompts/e205-preregistration.md:43`） | 今天的值：「let first_txg_today = first_txg_of_new_instance(devices, system_configuration, records)?;」（`crates/singlefs-core/src/mount.rs:1557`），其式子取「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1519`）；没有命中：「None => Ok(first_txg_today),」（`crates/singlefs-core/src/mount.rs:1650`）；有命中：「Ok(first_txg_today.max(checkpoint_txg_after_the_version(highest_txg)?))」（`crates/singlefs-core/src/mount.rs:1652`）。 | 对得上【读码】。 |
| 7 | 「开关只换新实例第一个 txg 那一个值」（`research/prompts/e205-preregistration.md:49`） | 取值只有一处：「let first_txg = first_txg_of_new_instance_under_the_test_only_switch(」（`crates/singlefs-core/src/mount.rs:9800`），枚举值本身只往后传进 `InstanceStart` 供报告档名：「first_txg_of_the_new_instance,」（`crates/singlefs-core/src/mount.rs:9860`），之后唯一的读是填档名：「first_txg_of_the_new_instance_branch: start.first_txg_of_the_new_instance.branch_name(),」（`crates/singlefs-core/src/mount.rs:7926`）。全文 grep `first_txg_of_the_new_instance` 在 `crates/singlefs-core/src/` 里没有第三处读这个枚举值的点。 | 对得上【读码】（Q2.2 的「中途换档」同源，这里只记事实）。 |

### D 项：登记没钉死、代码替它选了的三处（不算对不上，主 agent 判是否要登记补一句）

- D1 读哪一遍。登记写「挂载那一刻读不出……」，没写是挂载里哪一遍读数。代码在取号点（`crates/singlefs-core/src/mount.rs:9800`）现读一遍：「let ring_slot_readings = every_root_ring_slot_read_once(devices, system_configuration);」（`crates/singlefs-core/src/mount.rs:1569`），N 与「哪些槽算读坏」用这一遍的同一份读数，今天的值读的是另一遍（`highest_root_txg`）。函数文档自己写了这一点：「今天的值读的是另一遍」（`crates/singlefs-core/src/mount.rs:1538`）。读故障在挂载期间不变时，两遍相同，登记「读法写死」第一条把读故障设成「故障只在这次挂载里有」（`research/prompts/e205-preregistration.md:36`）【推：没跑，且取决于装置里读故障按落点还是按读次数计，我没读装置】；瞬时读错让两遍不同时，甲给的值因取 max 不小于今天的值（同一段文档写了，我读了 `crates/singlefs-core/src/mount.rs:1630-1653` 的 max，坐实不小于）。
- D2 N 全空时取 0。登记没写没有自证过的根时 N 是多少。代码取 0（`crates/singlefs-core/src/mount.rs:1577`），与今天 `highest_root_txg` 空时的 0（`crates/singlefs-core/src/mount.rs:1511`）同。条款没说这一格。
- D3 走出 u64 的处置。登记没写；代码：N 加偏移装不下就提前出口（`crates/singlefs-core/src/mount.rs:1641` 的 `checked_add` 之后 `break`），命中的最大 txg 加一装不下报 `SequenceNumberPastTheTopOfItsRange`（`crates/singlefs-core/src/mount.rs:1652` 的 `?`，经 `checkpoint_txg_after_the_version`）。条款没说这一格；与今天那一档对 `u64::MAX` 的处置同形（`crates/singlefs-core/src/mount.rs:1519`）。

### Q1.1 结论

七句里七句对得上，没有一句对不上。两档差别只在 `AllZeroNeverWritten` 一个分支（`crates/singlefs-core/src/mount.rs:1613` 对 `crates/singlefs-core/src/mount.rs:1614`），与登记两条定义的差别一一对应。
什么现象会推翻它：用同一盘面（含一个读回全零的槽、一个设备读不出的槽、一个错位读的槽）分别经两档取第一个 txg，若「只并读坏」档给的值受全零槽影响、或「并全零」档给的值不含全零槽那一圈 H 的最大者加一，就推翻。这是要主 agent 跑的用例（见末尾「要主 agent 跑的命令」）。

## Q1.2 产品入口在任何输入上走今天那一档

要证的命题：从产品入口 `mount_writable` 进，不论盘面、参数怎样，传给取第一个 txg 那一步的档恒是 `PastTheHighestRootAndRecordInTheRing`，而那一档的值与行为和只供测试的开关加进来之前相同。

### 调用点证据链（逐层，【读码】）

1. 产品入口只转一层：「mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)」（`crates/singlefs-core/src/mount.rs:8928`）。
2. 再转一层：「mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)」（`crates/singlefs-core/src/mount.rs:8941`）。
3. 再转一层：「mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(」（`crates/singlefs-core/src/mount.rs:8956`），实参 `BeforeTheOneReread::RereadImmediately`（`crates/singlefs-core/src/mount.rs:8961`）。
4. 这一个入口落到私有的汇总函数，枚举实参写死今天那一档：「FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,」（`crates/singlefs-core/src/mount.rs:9398`），调用在 `crates/singlefs-core/src/mount.rs:9393`。
5. 汇总函数是私有的：「fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(」（`crates/singlefs-core/src/mount.rs:9638`，没有 `pub`），crate 外够不着；crate 内它的全部调用点共 7 处（`crates/singlefs-core/src/mount.rs:8983`、`9393`、`9420`、`9445`、`9473`、`9551`、`9592`）。6 处的枚举实参是同一个字面量：「FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing,」在 `crates/singlefs-core/src/mount.rs:9398`、`9425`、`9450`、`9478`、`9556`、`9597` 各一行（`grep -nF` 命中恰这六行）；剩下的一处是只供测试的入口：「first_txg_of_the_new_instance,」（`crates/singlefs-core/src/mount.rs:8988`），实参来自它自己的形参（`crates/singlefs-core/src/mount.rs:8974` 起的签名）。
6. 其余带别的开关的公开入口（删除开关、整理开关、清扫配置、后台计算设置两种）都在上面那六行里，同一个字面量，没有一个把枚举暴露成形参。
7. 枚举不从输入造出来：派生只有「#[derive(Clone, Copy, Debug, PartialEq, Eq)]」（`crates/singlefs-core/src/mount.rs:919`），没有 `Default`、`From`、解析；`crates/singlefs-core/src/` 里 `std::env` 只出现在 `block_device.rs` 的测试临时目录里（`grep -rn 'std::env' crates/singlefs-core/src/*.rs` 三处命中全在 `block_device.rs`），不读环境变量选档。`FirstTxgOfTheNewInstance` 在 `crates/singlefs-core/src/` 之外只出现在三个验证包的文件（`crates/singlefs-verification/tests/writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero.rs`、`crates/singlefs-verification/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-verification/src/bin/e205_c971_inverted_first_txg_arm_costs.rs`；其中 `e156` 命中的是同名局部变量，不是该枚举——`grep -n 'FirstTxgOfTheNewInstance' crates/singlefs-verification/src/bin/e156_allocation_basis_counts.rs` 零命中）。
8. 取第一个 txg 的唯一调用点：「let first_txg = first_txg_of_new_instance_under_the_test_only_switch(」（`crates/singlefs-core/src/mount.rs:9800`）。

### 今天那一档的行为与加开关之前一样

- 加开关之前，同一个位置是「let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records)?;」（`git show HEAD:crates/singlefs-core/src/mount.rs` 第 8850 行，HEAD 是加开关之前的版本）。
- 加开关之后，今天那一档的分派：先「let first_txg_today = first_txg_of_new_instance(devices, system_configuration, records)?;」（`crates/singlefs-core/src/mount.rs:1557`），同一个函数、同样的三个实参、同样的 `?`；再「return Ok(first_txg_today);」（`crates/singlefs-core/src/mount.rs:1560`），在任何一次多余的盘读之前返回。额外的整环读（`crates/singlefs-core/src/mount.rs:1569`）在这个 `return` 之后，只有甲两档走得到。
- 所以今天那一档的返回值、错误、盘读次数与加开关之前逐项相同【读码】。
- 报告档名多一个字段：「first_txg_of_the_new_instance_branch: start.first_txg_of_the_new_instance.branch_name(),」（`crates/singlefs-core/src/mount.rs:7926`），是 `MountOutput` 上新增的只读字段，产品行为（盘字节、txg）不受影响【读码】。

### Q1.2 结论

对得上：产品入口恒走今天那一档，产品行为不变。依据全是【读码】；「全部 7 个调用点都在列」靠 `grep -n 'mount_writable_with_every_test_only_switch' crates/singlefs-core/src/mount.rs`（命中 8 行：7 处调用、1 处定义）核过。
什么现象会推翻它：存在一条从 `mount_writable` 或任一产品入口到汇总函数、枚举实参不是今天那一档的调用链；或者 `first_txg_of_new_instance_under_the_test_only_switch` 在今天那一档上发出了多余的盘读（例如把 `return` 挪到 `every_root_ring_slot_read_once` 之后）。后者可由主 agent 在记数设备上比加开关前后的读次数（命令见末尾）。

### 与本问相邻、但不在本问验收里的一处观测

- `mount.rs` 的文档注释写「产品路径恒 `PastTheHighestRootAndRecordInTheRing`」，同时开关在发布构建里够得着（`mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance` 是 `pub`，`crates/singlefs-core/src/mount.rs:8974`）。这与 C921（只供测试的开关在发布构建里够得着）同形，是已登记的欠账形，不是这一轮的新问题；我没有判它。

## Q1.3 清扫改法与 D16 已定项 6 逐句对；再核与 `recovery::read_root_ring_slot` 的处置是否同形

### 逐句对：`.claude/kb/decisions/16-发布语义.md` 已定项 6 骑手条款 1（该文件第 195 行）

条款里与读根环有关的几句，一句一格（引文照 kb 原文；骑手条款 1 全文在该行，下面只摘与清扫有关的句子，不另造缩写版）：

| # | 条款一句 | 代码（`crates/singlefs-core/src/instance_table_sweep.rs`） | 判 |
|---|---|---|---|
| 1 | 「读根环时每条根照落点公式」「核槽位」（`.claude/kb/decisions/16-发布语义.md:195`） | 清扫读根环的函数「fn every_root_of_the_ring_self_verified(」（`crates/singlefs-core/src/instance_table_sweep.rs:464`）里，每一次读出的字节先 `parse_slot` 再过落点核：「root_read = RootRecord::parse_slot(&bytes, filesystem_identifier).filter(」（`crates/singlefs-core/src/instance_table_sweep.rs:491`）紧接「root_sits_in_the_ring_slot_the_formula_gives(」（`crates/singlefs-core/src/instance_table_sweep.rs:492`）。清扫读根环只有这一处解析根（`grep -n 'parse_slot' crates/singlefs-core/src/instance_table_sweep.rs` 整个文件只命中第 491 行）。 | 对得上【读码】。 |
| 2 | 「第 0 代根照 mkfs 种根的槽另认」（`.claude/kb/decisions/16-发布语义.md:195`） | 清扫调的是读者同一个核，不自己写一份：这个函数在 `crates/singlefs-core/src/root_ring.rs:183` 定义，第 0 代另认在「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」（`crates/singlefs-core/src/root_ring.rs:188`）与「genesis_slot == ring_slot);」之内（`crates/singlefs-core/src/root_ring.rs:190`）；清扫经 `use` 引入同名函数（`crates/singlefs-core/src/instance_table_sweep.rs:45`）。 | 对得上【读码】。 |
| 3 | 「一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根、取读根环次序最前的」（`.claude/kb/decisions/16-发布语义.md:195`） | 清扫不择根：它把每槽读出的根收进 `roots` 返回（`crates/singlefs-core/src/instance_table_sweep.rs:507`），只用其 `instance` 判「环里有没有该实例的根」（「root.instance).collect();」，`crates/singlefs-core/src/instance_table_sweep.rs:359`）。没有「平局取次序最前」这一步可对。 | 不适用（条款这一句管择根，清扫不择根）【读码】。 |
| 4 | 「恢复与挂载读根环时都调」（`.claude/kb/decisions/16-发布语义.md:195`） | 条款点名的两处是恢复与挂载；清扫是第三处读根环的地方，条款没点它。代码注释把这句当作清扫要调的依据引用：「核落点公式（D16（发布语义） 已定项 6 骑手条款 1「恢复与挂载读根环时都调」」（`crates/singlefs-core/src/instance_table_sweep.rs:488`）。 | 代码做的与条款意思相容；字面上【条款没说】清扫，是 C973 把这句外推到清扫。条款里「都调」的名单该补上清扫（kb 的活，不是代码的活）。 |

清扫「重读一次」那半句出处不是 D16 已定项 6，是 D18 已定项 11 的回收一条：「其余的槽须自证过，否则重读一次，仍不过就这次不删」（`.claude/kb/decisions/18-块里携带什么信息.md:318`）。改法让「自证过」从「`parse_slot` 过」扩成「`parse_slot` 过且落点合」，落点不合的读数落进同一个「重读一次」循环：「for _reading in 0..2 {」（`crates/singlefs-core/src/instance_table_sweep.rs:477`），第一次落点不合时 `root_read` 为 `None`、不 `break`（`crates/singlefs-core/src/instance_table_sweep.rs:498`），第二次再读；两次都不过才返回 `ARootRingSlotIsNotSelfVerifiedAfterOneReread`（`crates/singlefs-core/src/instance_table_sweep.rs:504`），整轮不删。判：与 D18 已定项 11 那一句对得上【读码】。

全零与读不出照旧先于解析处理（`crates/singlefs-core/src/instance_table_sweep.rs:478-487`），读回全零或读不出直接整轮不删，不进重读：这与 D18 同一行「读成功的全零根槽与读不出同待遇」（`.claude/kb/decisions/18-块里携带什么信息.md:318`）对得上，改法没动它。

### 与 `recovery::read_root_ring_slot` 同一形（读得出、`parse_slot` 过、落点不合）的处置

| 读者 | 这一形怎么记 | 重读 |
|---|---|---|
| `recovery::read_root_ring_slot`（`crates/singlefs-core/src/recovery.rs:1995`） | 「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)」（`crates/singlefs-core/src/recovery.rs:2016`），与 `parse_slot` 不过同一个变体：「None => RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),」（`crates/singlefs-core/src/recovery.rs:2022`） | 读者自己不重读：「只读一次，不重试。」（`crates/singlefs-core/src/recovery.rs:1992`） |
| 挂载重建分配器那一遍 `read_root_ring_slot_telling_never_written_apart`（`crates/singlefs-core/src/mount.rs:3009`） | 记成 `BadAndNotAllZero`（`crates/singlefs-core/src/mount.rs:3037`），与 `parse_slot` 不过同一个变体：「None => RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero,」（`crates/singlefs-core/src/mount.rs:3040`） | 读者自己不重读，两遍并槽由调用方做 |
| 清扫（`crates/singlefs-core/src/instance_table_sweep.rs:464`） | `root_read` 为 `None`，与 `parse_slot` 不过同一条路（`crates/singlefs-core/src/instance_table_sweep.rs:491-497` 的 `.filter` 把落点不合并进 `None`） | 重读一次，仍不过整轮不删（`crates/singlefs-core/src/instance_table_sweep.rs:477`、`502-505`） |

三个读者对这一形的分类相同：错位读 = 「这一槽这一次读坏」= 与 `parse_slot` 不过同类（`crates/singlefs-core/src/recovery.rs:1974-1976` 的变体文档也这样写）。重读次数的差异来自各读者自己的条款（清扫：D18 已定项 11；挂载：D16 已定项 1），在 `parse_slot` 不过那一形上三者本来就不同，错位读没有引入新的差异。判：同一形上处置一致【读码】。

### 两处小差别，不影响一致，记下

1. 清扫第二次读若读不出或全零，返回的是 `ARootRingSlotIsUnreadableOrAllZero`（`crates/singlefs-core/src/instance_table_sweep.rs:479`、`484`），不是 `...NotSelfVerifiedAfterOneReread`：错位读加一次读坏，报的成员不同，两个都是整轮不删【读码】。
2. 清扫不调 `note_that_the_bytes_read_self_verify`（读者的自证过且落点对时才调，`crates/singlefs-core/src/recovery.rs:2019`）：这是改前就有的差别，该调用只服务挂载期间的读缓存换字节；清扫在挂载末尾、读阶段之后跑（D18 已定项 11 写「可写挂载末尾……判一次」），我没有读读缓存那一侧去核它到这一步已不在，标【推】。

### 一处过期的注释（代码里的文字，不是行为）

`crates/singlefs-core/src/mount.rs` 里：「别的读者（择根、算上限那一遍、清扫）照旧用 [`read_root_ring_slot`]」（`crates/singlefs-core/src/mount.rs:2991`）。清扫不调 `read_root_ring_slot`（非测试代码），它自己逐槽解析；这句注释对清扫不成立。改前就如此（改前清扫也是自己 `parse_slot`），C973 的改法没有把它改成真，也没有把注释改对。【读码】

### Q1.3 结论

清扫改法与 D16 已定项 6 骑手条款 1 逐句对：条款里管读根环的两句（落点核、第 0 代另认）对得上，择根平局一句不适用，「恢复与挂载都调」一句字面没点清扫（条款没说，C973 外推）。「错位读算自证不过、重读一次」与 `recovery::read_root_ring_slot` 同一形处置一致，重读那一半出处是 D18 已定项 11，也对得上。
什么现象会推翻它：存在一个落点不合而 `parse_slot` 过的槽，清扫在第一次读之后直接收进 `roots`、或第二次读才发现落点不合时没报 `ARootRingSlotIsNotSelfVerifiedAfterOneReread`；或存在第 0 代根住在 mkfs 种根的槽，清扫却当错位读拒掉（会让新池一开始清扫就整轮不删）。后一条可由主 agent 在刚 mkfs 的池上跑清扫核（命令见下）。

## 顺带记一条观测（不在三问验收里，供主 agent 判）

- E205 两处用例对「错位读」这一类读坏零命中：`grep -n -i 'misplaced\|错位'` 在 `crates/singlefs-verification/tests/writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero.rs` 与 `crates/singlefs-core/src/mount.rs` 的 `first_txg_past_the_ring_slots_read_bad_at_mount_tests` 模块（第 14129 行起 200 行内）都无输出；`crates/mutations.tsv` 里 E205 开关那 18 行里也没有一行含「落点」（`grep -n 'E205' crates/mutations.tsv | grep -c 落点` 得 0）。登记第 43 行 N 的定义写了「含落点公式核」，代码做到了（`crates/singlefs-core/src/mount.rs:3031`），但这一半没有 E205 自己的用例或变异罩着；它与重建分配器共用同一个读者函数（`crates/singlefs-core/src/mount.rs:3009`），别的用例（C743）是否罩着我没查。这是 Q2.4 的材料，留给攻方腿与主 agent。
- 背景里「取 N = 读得出的根里最小的 txg」那一句，与 `crates/mutations.tsv` 里 E205 的一行变异名同形（「甲的 N 取读得出的根里最小的 txg」，该文件第 6874 行）：那是变异的内容，不是代码。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1.1 第 1 条 甲并全零（登记第 41 行） | 一致【读码】 | 读不出、非全零自证不过（含错位读）、全零三类槽都算，`crates/singlefs-core/src/mount.rs:1612`、`1614` |
| Q1.1 第 2 条 甲只并读坏（登记第 42 行） | 一致【读码】 | 全零槽不算，`crates/singlefs-core/src/mount.rs:1613` |
| Q1.1 N 的取法（登记第 43 行） | 一致【读码】；两遍读数相同是【推】 | 取最大、含落点核、空时 0；N 与读坏槽用开关自己读的一遍，今天的值读另一遍，故障稳定时相同 |
| Q1.1 H_槽 窗口 [N+1, N+3S−1] | 一致【读码】 | `crates/singlefs-core/src/mount.rs:1638` 偏移 1 到 3S−1；一一对应是【推】 |
| Q1.1 max(今天, max H + 1)，无槽等于今天 | 一致【读码】 | `crates/singlefs-core/src/mount.rs:1650`、`1652` |
| Q1.1 开关只换那一个值 | 一致【读码】 | 取值一处（`crates/singlefs-core/src/mount.rs:9800`），枚举之后只填档名 |
| Q1.1 登记没写的三处（读哪一遍、N 空取 0、走出 u64） | 条款没说 | 代码各选了一个，与今天那一档同形，要不要登记补一句主 agent 判 |
| Q1.2 产品入口恒走今天那一档 | 一致【读码】 | 一串转发写死字面量（`crates/singlefs-core/src/mount.rs:9398`），汇总函数私有，7 个调用点 6 处字面量 1 处测试入口 |
| Q1.2 产品行为不变 | 一致【读码】 | 今天那一档与 HEAD 同一函数、同实参，`return` 在多余读之前（`crates/singlefs-core/src/mount.rs:1560`）；盘读次数未在记数设备上跑，是读码推出 |
| Q1.3 清扫 vs D16 已定项 6：落点核、第 0 代另认 | 一致【读码】 | 调读者同一个核（`crates/singlefs-core/src/instance_table_sweep.rs:492`） |
| Q1.3 清扫 vs D16 已定项 6：择根平局 | 不适用 | 清扫不择根 |
| Q1.3 清扫 vs D16 已定项 6：「恢复与挂载都调」 | 条款没说清扫 | 外推，意思相容；kb 名单该补，不是代码的活 |
| Q1.3 重读一次的出处 D18 已定项 11 | 一致【读码】 | 落点不合并进同一个重读循环（`crates/singlefs-core/src/instance_table_sweep.rs:477`） |
| Q1.3 与 `recovery::read_root_ring_slot` 同形处置 | 一致【读码】 | 错位读 = 与 `parse_slot` 不过同类，三个读者都如此 |
| 附：`crates/singlefs-core/src/mount.rs:2991` 注释 | 过期 | 称清扫用 `read_root_ring_slot`，实际清扫自己解析 |

## 要主 agent 跑的命令

以下都写成整条，我一条都没交。判决要引的数请主 agent 复跑，我这里没有跑出读数，所有「读码」都是只读命令（`grep -nF`、`sed`、`git show`）看到的。

1. 开关 12 条验证用例（Q1.1 推翻观测里的三类槽盘面、产品入口对拍，用例 `first_txg_taken_by_the_product_entry` 在第 610 行）：
   `task-server/task submit -- cargo test -p singlefs-verification --test writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero`
2. `mount.rs` 里甲窗口三条边角用例（N 自己那一槽不越过、顶端拒、窗口上方的槽不越过）：
   `task-server/task submit -- cargo test -p singlefs-core --lib first_txg_past_the_ring_slots_read_bad_at_mount_tests`
3. 清扫两条用例（两次都错位不删、只第一次错位重读读对不删；Q1.3 的「重读一次」与「第 0 代根不被拒」前半）：
   `task-server/task submit -- cargo test -p singlefs-core --lib misplaced_ring_reads_tests`
4. 第 0 代根不被清扫误拒（Q1.3 推翻观测的后一条）：复核不了。我没有找到一条现成的「刚 mkfs 的池上跑清扫、第 0 代根住 mkfs 种根的槽」的用例，不自己编命令；要主 agent 指一条或另写。
5. 今天那一档的盘读次数加开关前后相同（Q1.2 推翻观测的后一条）：复核不了。要在加开关之前的副本上用记数设备量一次作基线，我不做这类还原操作；Q1.2 的「相同」目前只是读码推出。

## 没做什么

- 没判 Q2 的四问与 Q3：不是这条腿的格。
- 没替主 agent采纳或出判决；没往统一任务服务交任务，判决要引的数（用例结果、盘读次数）没有复跑，命令在上一节。
- 没读禁读清单里的两份文件（同轮攻方腿的报告与本轮判决），也没读装置 `e205_c971_inverted_first_txg_arm_costs.rs` 里读故障按落点还是按读次数计的那部分，所以 D1 里「两遍读数相同」只写成【推】。
- 没写模型，没起任何脚本，也没有留下进程。草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-sonnet` 里没有产物。
