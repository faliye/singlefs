# m3-close-b20-code-r1 攻方腿（Opus）：C971 乙入判不出与 C973 第 0 代根判据

日期：2026-11-07。立场：假设实现与条款都错，造盘面打穿。方案按 `crates/` 今天的实现（主工作区，补丁已打进、未提交）。

## 复跑命令与模型文件

模型只有一份纯算术模型（C973 第 0 代根判据，不调实现、不读仓里代码），没写 Rust 原型，没调崩溃枚举函数（这一腿的攻击面里没有要枚举崩溃态的盘面，见「这条腿自己的限度」）。

| 文件 | sha256 |
|---|---|
| `research/prompts/m3-close-b20-code-r1-opus-model/genesis_criterion_model.py` | `db33449f6fb1c25b06e786586182c35b3c5de8bd4c866d1cfc503a58fcb00278` |
| `research/prompts/m3-close-b20-code-r1-opus-model/tools/rerun.sh` | `b783e0908b11cac1889afbef88e2916212707b6bfc76564ad34ad45cecb92ed6` |

复跑命令与读法在「要主 agent 跑的命令」一节。模型的读数一个都还没跑出，下文凡引模型的都写「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 落点 |
|---|---|---|
| 面一：一块盘两槽都自证过、另一块盘一槽读回全零，会不会误拒 | 成立（不拒） | `mount.rs:9241`；用例 `writable_mount_counts_…_as_read_errors.rs:721` |
| 面二：三块盘以上、一块盘的池 | 成立（判法与盘数无关）；用例只罩两块盘 | `mount.rs:9167`、`recovery.rs:5193` |
| 面三：重读那一次本身读故障 | 设备报读错走 R2 本身、同一拒因：成立。**打中（低，保守方向）**：重读是整遍重读、按重读那一遍单独判，条款写的是「重读那一槽一次」 | `mount.rs:9334`、`mount.rs:9336` |
| 面四甲：第 0 代根读自别的区域 | 成立（区域 1、2 的槽 0 一律拦；区域 0 的槽 0 在发布没写到时合法内容就是同字节的第 0 代根） | `instance_table_sweep.rs:611`、`root_ring.rs:172-173` |
| 面四乙：环里还没有根写进那一槽 | 成立（清扫在任何一槽全零时先停，判据走不到） | `instance_table_sweep.rs:576` |
| 面四丙：落点公式算出的 txg 恰好等于 H | 成立，而且这一格走不到：读回第 0 代根的那一槽，H 不可能恰好落进它；上界 `..=` 与 `..` 不分辨，是等价变异 | `instance_table_sweep.rs:611` |
| 面四丁：C973 条款字面 | **打中（判据字面写错）**：C973 那一行「怎么拦」按字面永远不成立，实现取的是另一个读法 | `.claude/kb/checks-owed.md:613`、`instance_table_sweep.rs:286` |
| 面四戊：丢了一次写进槽 0 的根槽写 | **打中（低，保守方向，活性）**：真住着第 0 代根的槽被判自证不过，清扫停到下一圈写到那一槽 | `instance_table_sweep.rs:527`、`instance_table_sweep.rs:541` |
| 面五：错位读回别槽的非第 0 代根，两条判据叠加 | 成立（落点核先拦，叠加只多停、不多删、不读第三次） | `instance_table_sweep.rs:583`、`instance_table_sweep.rs:527` |
| 面五（系统配置槽的错位读） | 不是这一批的新缺陷：系统配置槽不核本盘设备号与槽世代号，已登记 C864，去向里程碑十六 | `recovery.rs:5152`、`.claude/kb/checks-owed.md:523` |
| 检查单八格 | 逐格见「检查单」一节：七格成立，「类型与名字」一格附一条现状说明 | — |

## C971 乙入判不出

### 面一：一块盘两槽都自证过、另一块盘一槽读回全零

盘面：两块盘；盘 1 两槽都自证过；盘 0 最新那一槽读回全零（`parse_slot` 拒成 `NotSelfDescribing`，读的一方报自证不过），盘 0 另一槽自证过；设备都不报读错。

走向（代码逐行）：
- 读全零那一槽报自证不过：「) => reader.note_that_the_bytes_read_fail_self_verification(」（`crates/singlefs-core/src/recovery.rs:5163`），外面那一层把那一读改记读坏。
- R2 本身不成立（没有槽设备报读错）：「.any(|slots| slots.slots_reporting_a_read_error > 0)」（`crates/singlefs-core/src/recovery.rs:5193`）。
- 新判法前一半成立、后一半不成立：盘 1 两槽都自证过，「.any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)」（`crates/singlefs-core/src/mount.rs:9241`）为真，整句为假。
- 往下照常取 c_见证 比，不拒。

对照条款：「乙入判不出：重读之后仍是这两种之一的槽」（`research/prompts/e205-preregistration.md:46`，整行见附录），R2 的另一半照旧：一块盘两槽都自证过就判得出。实现与条款一致，不误拒。用例罩着：「fn a_hand_made_image_with_only_device_zero_failing_self_verification_mounts_writable() {」（`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:721`），变异 `crates/mutations.tsv` 第 7049 行（「R2 的另一半丢了」）由它抓。随机历史新步只藏第一块盘最新那一槽，罩的也是这一形（主 agent 任务 10211 的读数 Ok 23、Err 0，是主 agent 量的，我没复跑）。

反面也试了：这一形不拒，会不会放过一次该拒的倒挂。同段的系统配置轮换（`CommitStep::RotateSystemConfigurationSlots`，`crates/singlefs-core/src/transaction.rs:593`，几块盘之间没有屏障）崩在盘 0 那一写撕裂、盘 1 那一写没落时，盘 1 两槽自证过但都是上一轮的 tail。丢的见证只有正在提交的那一次发布（它没交回、没确认），已确认的发布在上一轮轮换里两块盘都写过。所以这一形不丢已确认的写，只留 R2 本身的前提「两槽都自证过的盘见过最近一轮轮换」不一定成立这一点——那是 R2 的事（D22 已定项 8 射程写 R2「被攻过零轮」），不是这一批的改动。推的，没造盘面。

### 面二：三块盘以上、一块盘的池

- 判法只按交进来的每块盘逐个读（「for device in devices.device_identities() {」，`crates/singlefs-core/src/mount.rs:9167`），两半都是「有没有一块盘」的存在式，与盘数无关。三块盘时：一块盘两槽自证过就不拒；每块盘都缺一槽才拒。与条款同形。
- mkfs 只在两块盘时钉区域归属（「if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {」，`crates/singlefs-core/src/make_filesystem.rs:558`），一块盘与三块盘的池建得出。一块盘的池：任何一次撕裂的轮换（普通断电）之后每次可写挂载都拒，只剩只读挂载与离线出路「/// 离线出路（α'，`system_configuration_repair`）重写一块盘不自证的那一槽：同 [`Self::write_system_configuration_slot`]，F 带整池的生效值。」（`crates/singlefs-core/src/transaction.rs:618`）。这是 C596 已定的冲突格，条款认了，不算误拒。
- 缺口（推的）：这一批的 11 条用例与随机历史都是两块盘（用例文件里「[DeviceIdentity(0), DeviceIdentity(1)]」写死两块盘，`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:503`）。一个只看前两块盘的变异（例如 `.take(2)` 套在 `slots_of_each_device.iter()` 上）没有用例抓。要不要补一条三块盘、只有第三块盘两槽都自证过的用例，交主 agent 定。

### 面三：重读那一次本身读故障

成立的几支（代码逐行）：
- 第一遍判乙入判不出 ⇒ 判据为真 ⇒ 调钩子、整遍重做读阶段：「let reread = read_stage(&cache, devices, system_configuration)?;」（`crates/singlefs-core/src/mount.rs:9334`）。系统配置槽经 `devices` 直接读盘（读阶段交的是盘表本身，`DeviceReadOnceInThisMount` 那一层在读阶段之后才套，`crates/singlefs-core/src/mount.rs:7718`），所以重读真到盘上。
- 重读那一遍有槽设备报读错、没有一块盘两槽都自证过 ⇒ R2 本身先判（`crates/singlefs-core/src/mount.rs:9186`），比法换成 `SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots`，仍按真 ⇒ 同一拒因拒（「if reread_against_the_witness.witnesses_a_publish_newer_than_the_selected_version() {」，`crates/singlefs-core/src/mount.rs:9336`）。错误里带两遍的比法，看得出是哪一形。条款：判不出就拒，读错与自证不过并成一半，两形同处置——成立。
- 重读那一遍择根本身失败（比如根环全读不出）⇒ `read_stage` 的 `?` 原样交回 `NoValidRoot`，拒因换了但仍在第一个写之前——成立。
- 重读那一遍读得出、自证过 ⇒ 按重读那一遍往下走——成立（用例 `system_configuration_slots_failing_self_verification_only_on_the_first_read_let_the_mount_through_on_the_reread`）。

**打中（低，保守方向）：实现按「整遍重读、只看重读那一遍」判，条款写的是「重读那一槽一次、重读后仍是这两种之一的槽」。**

条款原句：「乙入判不出：重读之后仍是这两种之一的槽」（`research/prompts/e205-preregistration.md:46`）；D22 已定项 8 射程「读阶段读回全零或读得出而自证不过的槽重读那一槽一次」（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`，整行见附录）。两句都只重读第一遍坏的那几槽，第一遍好的槽沿用第一遍的结论。实现的新判法只吃这一遍的读数（「some_system_configuration_slot_fails_self_verification_and_no_device_self_verified_both_slots(」，`crates/singlefs-core/src/mount.rs:9233`），重读那一遍把每块盘两槽全部重读。

盘面（两块盘，C971 最小历史即可；故障用只供测试的钩子造，`BeforeTheOneReread` 的文档许可「或在钩子里让重读那一遍也读坏」，「/// 或在钩子里让重读那一遍也读坏，强制进入「重读仍读不出」那一支）。」所在那一段，`crates/singlefs-core/src/mount.rs` 第 566 行）：
1. 第一遍：盘 0 槽 a（最新）、盘 1 槽 a（最新）读回全零；盘 0 槽 b、盘 1 槽 b 自证过。⇒ 没有一块盘两槽都自证过，乙入判不出为真 ⇒ 重读。
2. 钩子里：撤掉盘 0 槽 a 的故障；盘 1 槽 a 照旧全零；新加一处：盘 0 槽 b 读回全零（一次间歇的读故障，只落在重读那一遍）。
3. 实现：重读那一遍盘 0 只剩槽 a 自证过、盘 1 只剩槽 b 自证过，有槽自证不过、没有一块盘两槽都自证过 ⇒ 拒可写。
4. 条款：重读第一遍坏的两槽 ⇒ 盘 0 槽 a 读得出自证过、盘 1 槽 a 仍是全零；盘 0 槽 b 第一遍就自证过，不重读 ⇒ 盘 0 两槽都自证过 ⇒ 判得出 ⇒ 不因这一拒拒。

四句：
- 分不分辨臂：分辨「整遍重读」与「只重读坏槽」两种读法；与乙直拒、乙入判不出的选择无关（两条都写「重读那一槽」）。
- 系统当时看不看得到：看得到。第一遍的 `system_configuration_slots_as_read` 就在 `read_stage` 交回的读数里，重读那一遍只要按槽合并两遍结论。
- 满足字面哪一分句：「重读那一槽一次」与「重读后仍是这两种之一的槽」两个分句；实现多重读了第一遍好的槽，并拿它们重读的结论替掉了第一遍的。
- 跑前条款给的改法：条款没给改法；方向只多拒、不少拒（反方向要第一遍「别的池」或「布局不认」的槽重读成本池自证过，那是错位读，C864 的事），落在 C596 的只读出路里，不丢写。

改法（只在我的推理上量过、被攻过零轮）：重读那一遍只把第一遍结论是读坏的那几槽换成重读结论，其余槽沿用第一遍，再按同一判法判；或者条款收成「重读那一遍整遍重读、按重读那一遍判」，与 C554 乙整遍重做读阶段的口径对齐。哪一个由主 agent 交用户定。

| 改法 | 修打中的那一格吗 | 标 |
|---|---|---|
| 只换坏槽、其余沿用第一遍 | 修（上面第 4 步的结论） | 推的 |
| 改条款为整遍重读 | 不改代码，条款与代码对齐 | 推的 |
| 什么都不改 | 那一格多拒一次，落 C596 只读出路 | 推的 |

用例草稿（没编、没跑；照 `writable_mount_counts_…_as_read_errors.rs` 里 `devices_with_read_faults` 的写法，要一个能在钩子里「撤一处、加一处」的读故障表，现有的只有整表一起撤的 `faults_lifted`）：

```rust
/// 第一遍两块盘最新那一槽读回全零；钩子里撤掉盘 0 那一处、给盘 0 另一槽加一处只在重读那一遍才有的全零。
/// 条款（只重读第一遍坏的槽）判得出、不拒；实现（整遍重读、按重读那一遍判）拒。
#[test]
fn a_slot_self_verified_on_the_first_read_and_failing_only_on_the_reread_does_not_refuse_the_mount() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_of_c971(&parameters).image;
    let newest_of = |device| newest_system_configuration_slot(&image, device);
    let other_of = |device| u64::from(FIXED_STRUCTURE_SLOT_SPACING) - newest_of(device);
    // 读故障表两张：第一遍才有的（钩子里撤），重读才有的（钩子里加）；装法照 devices_with_read_faults，另加一个开关
    let first_read_only = [(DeviceIdentity(0), newest_of(DeviceIdentity(0)))];
    let throughout = [(DeviceIdentity(1), newest_of(DeviceIdentity(1)))];
    let reread_only = [(DeviceIdentity(0), other_of(DeviceIdentity(0)))];
    // … 建 devices、钩子里翻两个开关，经 mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread 挂 …
    // 断言：照条款应当做成、取到实例 2；今天的实现交 NewerStateStillUnreadableAfterOneReread，这一条红
}
```

要主 agent 跑的：这条用例要先由实现员写成（测试文件在验证包，攻方腿不写进验证包），写成之后的命令见「要主 agent 跑的命令」第 2 条。

（上面「读阶段交的是盘表本身」的出处：「读阶段、恢复与重建分配器（这之前）读的不经它：它们自己的重读要打到盘上。」（`crates/singlefs-core/src/mount.rs:7705`）。）

## C973 第 0 代根判据

先把落点算清（`crates/singlefs-core/src/root_ring.rs:172-173`）：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:172`）、「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:173`）。三个区域槽 0 第一次被发布路径写到的 txg：区域 1 是 txg 1，区域 2 是 txg 2，区域 0 是 txg 3S；之后每 3S 个 txg 各写一次。第 0 代根只认三个区域的槽 0（「.any(|genesis_slot| genesis_slot == ring_slot);」，`crates/singlefs-core/src/root_ring.rs:190`）。判据：「(GENESIS_CHECKPOINT_TXG.0 + 1..=highest_txg_of_the_pass.0)」（`crates/singlefs-core/src/instance_table_sweep.rs:611`）接「.take(ring_length_in_slots)」（`crates/singlefs-core/src/instance_table_sweep.rs:612`），等价于「H 不小于这一槽第一次被写到的 txg」（推的，按上面两行算）。

### 面四乙：环里还没有任何根写进那一槽

清扫读一槽读回全零就整轮停：「if bytes.iter().all(|byte| *byte == 0) {」（`crates/singlefs-core/src/instance_table_sweep.rs:576`）。mkfs 把根环三个区域整段写 0、只在槽 0 种第 0 代根（D22 已定项 8 第 3 条，`.claude/kb/decisions/22-单元原子性怎么合成.md:183`），所以只要环里还有一槽一次都没被发布写到，判据根本走不到。txg 逐次加一（`crates/singlefs-core/src/mount.rs:5727` 一带的 `checked_add(1)`）时，走得到判据就要除区域 0 槽 0 之外的 3S − 1 槽都写过，即 H ≥ 3S − 1。推的。

推论：判据真正要分的只有区域 0 槽 0 在 H = 3S − 1 那一刻（合法内容就是第 0 代根）与 H ≥ 3S（合法内容是一条真根）两种；区域 1、2 的槽 0 在走得到判据时一律算发布已写到。附带一句现状（不是这一批的）：从 mkfs 起到写满一圈之前，清扫每一轮都停在全零槽、一行都不删。

### 面四甲：第 0 代根读自别的区域

- 错位读把区域 1 或 2 的槽 0 读成第 0 代根：H ≥ 3S − 1 ≥ 2，判据成立 ⇒ 第一遍就读成的重读一次（「if !read.self_verified_on_the_first_read {」，`crates/singlefs-core/src/instance_table_sweep.rs:527`），重读仍是第 0 代根就停——成立。
- 错位读把区域 0 槽 0 读成第 0 代根：第 0 代根的副本此刻只可能还在区域 0 槽 0 自己（区域 1、2 的槽 0 早被 txg 1、2 盖掉）。要读回一份第 0 代根，源头只能是它自己（不是错位）或一个丢了写、仍留着第 0 代根的区域 1、2 槽 0——后者自己在这一遍里被判据拦、整轮停。所以单次错位读造不出漏删。
- 要漏，得把 H 压低：把最新那条根所在的槽错位读掉，H 就从真值掉成次大的。最新那条在区域 0 槽 0（txg 3S）时，把它读成第 0 代根需要一份第 0 代根的源头，同上，源头只剩会被拦的那几槽。最新那条在别处时，它被错位读成别槽的根，落点核先拦（「RootRecord::parse_slot(&bytes, filesystem_identifier).filter(|root| {」，`crates/singlefs-core/src/instance_table_sweep.rs:583`），第一遍判据之前就重读了，H 里有它。
- 判据之后的重读读出的根不回写 H（「// 迭代上界是根环的槽数；每槽至多再读一次（判据只用上面那一遍读出的最大 txg，重读读出的不改它）；提前出口是这一次不删的那几种。」，`crates/singlefs-core/src/instance_table_sweep.rs:522`）。我试了「两个住第 0 代根的槽，一个拦下重读出更大的 txg，另一个按旧 H 放过」：放过的那一槽要是错位，源头同样只剩会被拦的槽；不是错位，它就真住着第 0 代根，放过是对的。没打中。推的；模型 `deflation-changes-verdict` 那一栏数的就是这一形，待主 agent 跑。

### 面四丙：落点公式算出的 txg 恰好等于 H

走不到。设读回第 0 代根的槽是 X，若 target(H) = X：H 是这一遍读出的根里最大的 txg，那条根过了落点核，住在 target(H) = X；但 X 读出的是第 0 代根，矛盾。所以 H 恰好落进读回第 0 代根的那一槽这一形不存在，上界写 `..=H` 还是 `..H` 结局逐盘面相同——是等价变异，不该进变异表（现有的第 6994 行改的是下界，不是这一格）。推的；模型的 `equality` 一栏应当恒为 0，待主 agent 跑。

### 面四丁：C973 条款字面（打中：判据字面写错）

C973 那一行「怎么拦」一栏写判据的那一分句：「这一遍读出的环里已有落点公式会写进这一槽且 txg 更大的根时，算这一槽自证不过」（`.claude/kb/checks-owed.md:613`，整行见附录）。按字面，要环里有一条根 R：落点公式把 R 写进这一槽、且 R 的 txg 比第 0 代根大。但这一遍交回的每条根都过了落点核（`crates/singlefs-core/src/instance_table_sweep.rs:583`），R 就住在落点公式给的那一槽，也就是这一槽；而这一槽读出的是第 0 代根。所以字面条件在任何盘面上都不成立，判据按字面是空的。

实现取的是另一个读法：「/// 一个槽 0 读出第 0 代根、而同一遍读出的环里最大的 txg 已到发布路径写进这一槽的 txg，也算这一槽自证不过）。」（`crates/singlefs-core/src/instance_table_sweep.rs:286`）——看的是 H 有没有越过这一槽第一次被写到的 txg，不要求环里真有那么一条根。这个读法才拦得住 C973 的用例盘面，代码对，条款的字写错了。

四句：不分辨臂（只有一个改法，没有对照臂）；系统看得到判别它的东西（H 就在这一遍读数里）；满足的是「怎么拦」那一分句的字面；跑前没有反向条款。交主 agent：C973 那一行「怎么拦」与去向两处的这一分句改写成实现那一句的读法（写回归书记员，不归攻方腿）。

### 面四戊：丢了一次写进槽 0 的根槽写（打中：低，保守方向，活性）

说谎的设备吞掉根槽写在故障模型里：「/// 可写挂载，而 `roots_whose_root_slot_is_missing` 那几条根的根槽写没落盘（说谎的设备吞了；它们的记录与单元都落了）：」（`crates/singlefs-verification/src/model.rs:8068`）。

盘面（S = 4）：mkfs → 第一次可写挂载暖机 txg 1、2，txg 1 的根槽写（落区域 1 槽 0）被吞，区域 1 槽 0 仍是 mkfs 种的第 0 代根 → 之后发布到 txg 11 以上（每一槽都写过，清扫走得到判据）→ 某次可写挂载末尾的实例表清扫。

走向：区域 1 槽 0 读出第 0 代根，过落点核；H ≥ 11 ≥ 1，判据成立 ⇒ 重读一次，读出的仍是第 0 代根（「.filter(|root| !genesis_root_reached_by_the_publishes_of_the_pass(root, read.ring_slot));」，`crates/singlefs-core/src/instance_table_sweep.rs:541`）⇒ 这一轮不删（`RootRingSlotIsNotSelfVerifiedAfterOneReread`）。盘上那一槽真住着第 0 代根，没有错位读，判据把真内容当了读坏。每次清扫都停，直到 txg 13 把区域 1 槽 0 盖掉，至多一圈（3S 次发布）。区域 0 槽 0 丢 txg 3S 那一写同形。

四句：不分辨臂；系统当时看不出「丢写留下的第 0 代根」与「错位读回的第 0 代根」——两者在这一槽读出的字节逐位相同（判别子观测不到那一形，只是方向保守）；满足的是 C973「仍不过这一次不删」那一句的结局，条款本身认这一结局；跑前没有改法。只少删、不多删，丢的是清扫的活性一圈；我判它不必改，写进去是让判决知道这一格的代价。推的，模型 `clean` 模式（`lost` 非空）那几行数的就是它，待主 agent 跑。

### 面五：错位读回别槽的非第 0 代根，两条判据会不会叠加误判

- 非第 0 代根错位读进任一槽：它的 txg 的落点是它原来那一槽，不是被读的这一槽，落点核拦（`crates/singlefs-core/src/instance_table_sweep.rs:583`）⇒ 这一读当自证不过，重读一次（「for _reading in 0..2 {」，`crates/singlefs-core/src/instance_table_sweep.rs:483`）。它进不了 H，抬不高也压不低判据。
- 叠加一：第一读错位（落点核拦）、重读又错位读回第 0 代根（这一槽是槽 0）⇒ 第一遍记「没一次读成」，判据成立时直接停，不读第三次（`crates/singlefs-core/src/instance_table_sweep.rs:527`）。
- 叠加二：第一读读出第 0 代根、判据成立、判据重读错位读回别槽的非第 0 代根 ⇒ 落点核拦 ⇒ `None` ⇒ 停。
- 两条判据叠在一起只会让清扫多停，不会让一条错位来的根被当成这一槽的真根，也不会多读。成立。模型 `single` 模式三读各取任一槽、`double` 模式两槽各错第一读，数的就是这几形（推的，待主 agent 跑）。
- C971 那一侧：系统配置槽读得出、fsid 相同就算自证过（「if system_configuration.immutable.filesystem_identifier」，`crates/singlefs-core/src/recovery.rs:5152`），不核本盘设备号与槽世代号。错位读回同盘另一槽或别盘的一槽，会让「这块盘两槽都自证过」凭空成立、把乙入判不出与 R2 一起关掉。这一形已登记：「别的自证结构——系统配置槽、journal 记录、单元——各有自己的位置绑定」（`.claude/kb/checks-owed.md:523`，C864，去向里程碑十六），不是这一批引入的，这里只记它对新判法同样成立。

## 检查单（八格逐格）

我现查用的命令：`git diff HEAD -- crates/singlefs-core/src/mount.rs` 与 `… instance_table_sweep.rs` 落进草稿目录（`diff-mount.txt` 加的行 56 行），再 `grep -n '^+.*\(expect(\|unwrap()\|_ =>\|unreachable!\|todo!\|assert!\)'` 两份，都退 1（没有命中）。

| 格 | 判定 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 两份 diff 加的行里没有 `todo!`、`unreachable!`、`assert!`、`expect(`、`unwrap()`、`_ =>`（上面那条 grep 退 1）。C973 那一段在 HEAD（`git log -S` 落在 b14b6fe92），里面唯一一处「.expect("根环至少三个区域各一槽，每槽都交回了一条根");」（`crates/singlefs-core/src/instance_table_sweep.rs:511`）的理由成立：每一槽要么推进 `roots_read`、要么提前 `Err` 返回，走到这里 `roots_read` 有 3S ≥ 12 条 |
| 类型与名字 | 成立，附一条现状 | txg 全程是 `CheckpointTxg`，偏移是 `DeviceOffsetInBytes`，S 是 `RootRingSlotsPerRegion`；判据里把 txg 拆成裸 `u64` 迭代再包回（`crates/singlefs-core/src/instance_table_sweep.rs:611`），只在函数内部；`RootRingSlot` 的区域号与槽号是裸 `u64`、环长是裸 `usize`，是这一批之前就有的，没有混用出错的地方。新名照内容起，没有序数与缩写 |
| 不兼容旧代码与旧格式 | 成立 | 两份 diff 加的行 grep 旧版、旧布局、迁移、`legacy`、布局号，退 1 |
| 不新建判定库 | 成立 | 产品代码改动不存判定（读过两份 diff） |
| 所有任务多机可用 | 成立 | 新测试文件 grep `std::fs`、`File::`、`temp_dir`、`ExclusiveTemporaryDirectory`、`hostname`、`/home/`，退 1；盘全在内存里 |
| 测试优先多线程 | 成立 | 11 条（`grep -c '#\[test\]'` 数的）各一个盘面一次挂载，测试框架本身按用例并行；单条里没有可切的循环，不该再切 |
| 带 --item 的命令与项 | 成立（行在；`--list-items` 没跑） | 两项在 `.claude/gate.d/crash-point-replay-items.tsv` 第 63、64 行；变异名字带 C971 的 19 行（`grep -c 'C971' crates/mutations.tsv`）加第 1948、2284、2285 行共 22 行，与主 agent 说的 22 条对得上。`--list-items` 是跑门禁入口，子 agent 不跑，交主 agent |
| 新崩溃流多罩了什么 | 成立，附缺口 | 新步只藏第一块盘最新那一槽，罩面一（另一块盘两槽都自证过 ⇒ 不拒）；原来那一步罩拒的那一边。两步都只造「读回全零」，没造「读得出而自证不过（非零）」与一块、三块盘的池；读得出而自证不过那一形由撕裂轮换的崩溃枚举经挂载透明判官罩着（`crates/singlefs-verification/src/mount_transparency.rs` 新加的 `system_configuration_slot_bytes_fail_self_verification`） |

## 要主 agent 跑的命令

1. 模型复跑（C973 第 0 代根判据）。先在仓根跑小段估时，再跑全量：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-legs/attack -- python3 -I /home/fy5090/code/singlefs/research/prompts/m3-close-b20-code-r1-opus-model/genesis_criterion_model.py --quick`
     在哪起：草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-legs/attack`。看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=genesis_criterion_cost S=4 … sweeps=… seconds=…`，按 `seconds / sweeps` 乘全量的段数估挂钟（我按代码推全量约八分钟，推的；估出来超过 40 分钟就不跑全量、回报）。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-legs/attack -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b20-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b20-code-r1-snapshot --work /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-legs/attack/rerun-1`
     看：`exit=0`；`rerun-1/out/full.out` 里每个 `name=genesis_criterion S=… mode=…` 行。判法写在跑之前：`hidden` 一栏任何一行非零 ⇒ 第 0 代根判据被单槽或双槽错位读打穿（推翻面四甲、面五「成立」）；`equality` 非零 ⇒ 推翻面四丙；`deflation-changes-verdict` 非零 ⇒ 推翻面四甲第四条；`clean` 模式 `stop:genesis-still` 非零而 `hidden` 为零 ⇒ 面四戊那一形（预期非零）。`--frozen` 这一份脚本不读，给快照目录只为过参数检查。
2. 面三的用例：先派实现员把「用例草稿」写进 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs`，写成后：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors; 范围=1 个测试目标; 为什么=核攻方腿面三：第一遍自证过、只在重读那一遍坏的槽按条款不该让可写挂载被拒' task-server/task submit -- cargo test -p singlefs-verification --test writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors a_slot_self_verified_on_the_first_read_and_failing_only_on_the_reread_does_not_refuse_the_mount`
   在哪起：仓根。看：今天的实现上这一条红（`✗` 行是那条断言），红了即坐实面三的打中。
3. 「带 --item」一格的列项：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃点重放入口列项; 范围=只列项不跑; 为什么=核两项能被列出' task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items`（在仓根起；看输出里有没有 `read-faulted-states-never-hold-the-witness-criterion` 与 `random-histories-and-fault-injection-given-states` 两行）。范围声明的种类我没把握，被范围声明闸拒就照它的出路改。

## 没打中的形状

| 形状 | 取样范围 | 结论 |
|---|---|---|
| 一块盘两槽自证过 + 另一块盘一槽读回全零或撕裂 | 两块盘、按代码逐行走；三块盘按判法的存在式推 | 不拒，与条款同（面一） |
| 同段轮换一写撕裂一写没落，R2 另一半放过 | 按提交步骤推，没造盘面 | 丢的只是没确认那一次的见证，不丢已确认的写（面一末段） |
| 乙入判不出排在 `NothingWitnessed` 之前（c_见证 = 0 的池也会被拒） | 读 `crates/singlefs-core/src/mount.rs:9186-9193` 的次序 | 与 R2 本身同一次序，条款没给 c_见证 = 0 例外，不算打中 |
| 重读那一遍设备报读错 / 择根失败 / 读得出 | 按代码逐行走 | 都在第一个写之前拒或照常做成（面三前半） |
| 读阶段之后（逐盘核、取号）才出现的自证不过槽 | 读 D22 已定项 8 射程「读阶段」那一句 | 条款只管读阶段，实现也只在读阶段判，一致 |
| C973：第 0 代根错位读进槽 0、H 压低、两次错位叠加 | 推的；模型 `single`（S = 4、5、8、16，单槽三读各取任一槽）与 `double`（S = 4、5，两槽各错第一读），H 取每圈末尾前后五个值，丢写取落进槽 0 的那几次与最后一次 | 推不出漏删；数待主 agent 跑 |
| C973：非第 0 代根错位读与第 0 代根判据叠加 | 同上 | 只多停、不多删、不读第三次（面五） |

崩溃状态这一腿没有枚举：攻的五面都是读故障与错位读下的一次挂载或一次清扫，没有要按写流切段的盘面；`crash_state_count_with_torn_in_place_overwrites` 没算，原型一条都没写，累计状态数 0。

## 这条腿自己的限度

- 所有「成立」都是读代码推的，没有一条在实现上跑过；「没打中」只抽了一次样，照规则不支撑「没打中」的结论，要第二份样本。
- 模型是我照自己读的代码手抄的判据，与实现共用我的读法：读错了，模型照样错。它只能核我的推理在更多盘面上是否自洽，不能当实现的校验路子。
- 面三的打中只在钩子造的「第一遍好、重读才坏」那一形上成立；这一形在真盘上要一次只落在第二次读上的间歇读故障，我没量它有多常见。
- 系统配置槽的错位读（C864）只记了对新判法的影响，没造盘面。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。
- 没往统一任务服务交任务：模型复跑、面三的用例、`--list-items` 都在「要主 agent 跑的命令」一节，判决要引的数一个都没复跑。
- 没写验证包里的测试（写范围不含 `crates/`），面三只给了用例草稿。
- 没建仓副本、没编译；草稿目录里只有三份 diff 文本（`diff-mount.txt`、`diff-sweep.txt`、`diff-mt-cl.txt`），没有副本与编译目录要删。
- `tools/rerun.sh` 照模板只改了 `rerun_steps`；模板自带的 `--selftest` 拿它自己的假仓跑 `rerun_steps`，改过之后那一格会红（假模型目录里没有 `genesis_criterion_model.py`），这一份不该拿去跑自证。

## 附录：引到的 kb 与登记条款整行

`research/prompts/e205-preregistration.md` 第 46 行：

  - 乙入判不出：重读之后仍是这两种之一的槽，并进 R2 的「有槽设备报读错」那一半当读错计，R2 的另一半（没有一块盘两槽都自证过）照旧；判不出就拒可写。

`research/prompts/e205-preregistration.md` 第 45 行：

  - 乙直拒：读阶段读每块盘两槽系统配置，有一槽读回全零或读得出而自证不过就重读那一槽一次，仍是这两种之一就拒可写，不再看 R2 的两半；

`.claude/kb/checks-owed.md` 第 613 行（C973）：

| C973 | 实例表清扫读根环不核落点 | 实例表清扫（`crates/singlefs-core/src/instance_table_sweep.rs` 第 470 行起的 `every_root_of_the_ring_self_verified`）改前逐槽解析根、不调落点核 `root_sits_in_the_ring_slot_the_formula_gives`，而 D16（发布语义） 已定项 6 第 195 行写落点核「恢复、挂载与实例表清扫读根环时都调」（改前那一句名单里没有清扫）（`.claude/kb/decisions/16-发布语义.md`）；后果：一次错位读让清扫删掉了环里还有根的实例行。造出了盘面、是真缺陷（主 agent 任务 8811 证红：「清扫读根环不核落点」那一行抓到，红在 `crates/singlefs-core/src/instance_table_sweep.rs` 测试模块用例 `misplaced_read_replacing_the_only_ring_root_of_an_instance_on_both_reads_deletes_no_row` 的「错位读把实例 1 唯一的根换成了别槽的根，清扫判出要删的行」那一句）；补的条件：`crates/singlefs-core/src/instance_table_sweep.rs` 第 362、367 行只在「实例在根环里没有自证根」时删行，要删掉还有根的行，得是错位读把某槽的真根换成别槽自证过的根，且被换掉的是那个实例在环里唯一的根 | 造一份满足补的条件的盘面（某槽的真根被换成别槽自证过的根，被换掉的是该实例在环里唯一的根），清扫那一遍必须不删那一行。盘面已造出，每种换法各有用例罩着：换成别槽的非第 0 代根，落点核拦下（`crates/singlefs-core/src/instance_table_sweep.rs` 测试模块 `misplaced_ring_reads_tests` 的 `misplaced_read_replacing_the_only_ring_root_of_an_instance_on_both_reads_deletes_no_row`）；换成第 0 代根，落点核认第 0 代根住三个区域的槽 0 任一槽（`crates/singlefs-core/src/root_ring.rs`），核不出，所以清扫另加第 0 代根判据（`crates/singlefs-core/src/instance_table_sweep.rs` 第 601 行的 `genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached`）：区域 r 的槽 0 读到第 0 代根、而这一遍读出的环里已有落点公式会写进这一槽且 txg 更大的根时，算这一槽自证不过，走原有的重读一次，仍不过这一次不删（用例 `misplaced_read_returning_the_genesis_root_over_the_only_ring_root_of_an_instance_on_both_reads_deletes_no_row`：两次读都错位读回第 0 代根，停在那一槽、实例 1 那一行不删；阳性对照 `misplaced_read_returning_the_genesis_root_over_the_only_ring_root_of_an_instance_reread_right_keeps_its_row`：重读读对了照认那条根，`the_genesis_root_left_in_a_ring_slot_no_publish_has_reached_yet_is_self_verified_and_no_row_is_a_candidate`：发布还没写到那一槽时第 0 代根照认） | 无；零轮（攻方第二份样本附带的线索，辩方补条件，之后没有再攻）；去向：未排，改法已写，已打进主工作区。清扫读根环改调读者同一个落点核 `root_ring::root_sits_in_the_ring_slot_the_formula_gives`，错位读算这一槽自证不过，走原有的重读一次，仍不过这一次不删；落点核认第 0 代根住三个区域的槽 0 任一槽，核不出错位读回的第 0 代根，所以清扫另加第 0 代根判据（`crates/singlefs-core/src/instance_table_sweep.rs` 第 601 行的 `genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached`，条件写在「怎么拦」一栏）。这个漏洞是代码轮判决 `research/prompts/m3-close-b19-switch-sweep-code-r1-main-verification.md` 第二节 Q2.3「错位读回第 0 代根」一格在副本上量到的：实例 1 唯一的根在区域 1 槽 0，一次错位读回区域 0 槽 0 的第 0 代根，落点核照收，实例 1 那一行判成要删（任务 9256）。验证（量的）：`singlefs-core` 的 `--lib` 419 passed（任务 9937）、`singlefs-core` 与 `singlefs-verification` 两包 clippy 退 0（任务 9938）、19 行变异 `--item` 全抓到（任务 9753）。第 0 代根判据被攻过零轮，等下一次代码轮攻（判决第四节），暂不销账（判决第二节 Q2.3 与第五节） | 判决 `research/prompts/m3-close-b19-e165-r2-main-verification.md` 第二节「清扫读根环不核落点」一格；判决 `research/prompts/m3-close-b19-e165-r3-main-verification.md` 第二节「清扫不核落点」一格 |

`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 185 行（已定项 8 射程）：

**射程**：系统配置自己的更新协议（槽轮换的代号、校验和多宽）跟着单元头字段表（D18（块里携带什么信息）已定项 7）的口径走，落在已定项 9 与已定项 16；zoned 介质上这些固定结构能不能原地轮转，属于 D22（单元原子性怎么合成） 未定项 6，还没答。容错承诺是单故障：每盘一槽系统配置坏在承诺里，取号之后两块盘最新那一槽持续坏也在承诺里（用户 2026-10-19 定，原话在变更史）：取号写两轮，每轮逐盘一写、轮后一道池屏障，每块盘两槽都带取号时的新号，全或无判失败时每块盘只回卷一次（`crates/singlefs-core/src/transaction.rs` 的 `write_the_witnessed_acquisition`、`roll_back_acquisition`）；每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩），那一形的可写挂载按 R2 的乙入判不出拒，承诺仍不罩它，两条用例当登记，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） 那一行。可写挂载读系统配置的见证按盘读（S-辨：一槽撕裂、另一槽自证过，与设备读不出分开），判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）。读阶段读回全零或读得出而自证不过的槽重读那一槽一次，重读后仍是这两种之一的槽并进 R2「有槽设备报读错」那一半当读错计，R2 的另一半照旧，判不出就拒可写（乙入判不出，用户定，原话在变更史）；「自证过」与 `system_configuration_slots_of_one_device_read_once` 同口径（整槽校验和过、fsid 与本池相同），布局号不认、每区槽数越界的槽不算坏、不并进；乙入判不出只加这一道拒，不拒时其余每一步不变。撕裂的系统配置轮换里，落在 C596（见证判不出的系统配置盘面只剩只读挂载） 已定的冲突格的那些态（设备没报读错、坏的槽是撕裂的、没有一块盘两槽都自证过）照它拒可写、只给只读挂载，其余态不因这一拒被拒；态数在 E205（多处读坏倒挂三条改法的代价） 页「交用户的岔路表」乙入判不出那一行。判不出的盘面有一条离线出路（C596（见证判不出的系统配置盘面只剩只读挂载））：管理员断言一个版本，那一版的根或那次发布的末条记录读得出且是它、各盘单元区里本池单元的 `birth_txg` 最大值不大于它，才只重写不自证的系统配置槽，写的实例代号取各盘自证槽、根环、journal 环与单元写序里的最大值、不加一；否则一个字节不写（用户 2026-10-19 定、2026-10-20 加 `birth_txg` 闸，原话在变更史；`crates/singlefs-core/src/system_configuration_repair.rs` 的 `rewrite_system_configuration_slots_for_the_asserted_version`）。某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒：这块盘带所选那一版的全部单元（清单非空、每个落点读一次、字节与验得过的那一份相同）；这块盘的根环槽与 journal 环槽没有一槽设备报读错；这些槽里没有比所选那一版新的根或记录（按择根的序）；这块盘单元区里没有写序实例大于所选那一版实例的单元；所选那一版的根槽那一半不缺（与落后支缺根那一半同一判）。补写失败照取号每块盘回卷一次（用户 2026-10-03 定这一步拒，2026-10-19 开例外，2026-10-20 加单元区扫描与环槽读错两样，原话在变更史；`crates/singlefs-core/src/mount.rs` 的 `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`，实现只拿两槽都读得出、都自证不过的盘过闸，有一槽设备报读错的照旧拒）。取号之前的逐盘核遇到盘表次序最前的、没被补写的盘就整个返回，只报这一块盘、它按次序先不过的那一样：先判两槽里有没有一槽设备报读错，再按带全部单元、环槽没有读错、环里没有更新的根或记录、单元区没有写序实例更大的单元、根槽那一半不缺的次序判，先不过的一样装进挂载错误成员 `SystemConfigurationUnreadableAfterTheReadStage` 的载荷（`CheckingEachDeviceBeforeTheAcquisition` 的 `not_rewritten_because`），不加挂载错误成员；管理员按报的那一样修好再挂，可能撞上同一块盘的下一样或下一块盘（C874（补写那一支进不了崩溃测试、运行时看不出））。挂载成功时，挂载输出的 `devices_rewritten_by_the_acquisition` 按盘表次序列取号写两轮顺带补写的盘（两槽一份本池自证过的系统配置都没有、过了补写闸的那几块），没有补写是空的。补写盘有了一份本池自证过的系统配置槽之后（补写失败回卷写回的旧号，或取号写两轮只落了先写那一轮之后崩溃留下的新号），这块盘不再是两槽全废：下一次挂载走落后支、不再过补写闸，补写闸各件（带全部单元、根槽那一半在内）都不再判，落后支只在这块盘落后于所选那一版时才看单元与根槽那一半，补写盘表为空；这一形对所有有自证槽的盘同形，不是回卷独有（C929（补写盘回卷之后不再过补写闸））。丢写的界按类写：全池读回一个值的纯形丢写 120 格，一盘报读错、另一盘读回全 0 的混合形丢写 0 格（C331（择根倒挂压过已确认的写） 网格上的格数，副本上量的），丢写格都落在每盘各坏一槽系统配置、同时最新根一时读不出的双故障形里。两块盘是同一个池时，作废持久之前崩溃：旧池读回最后确认的一版；可写挂载的结局按现行补写条款。作废持久之后、最后一道屏障之前崩溃：恢复只有两种结局——整池拒（`NoValidSystemConfiguration`），或读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置。调用方拿自己断言的参数（例如旧池的 fsid）去挂、与盘上择到的系统配置不符而拒（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`），是调用方的断言与盘不符，不算盘面的结局。设备守屏障时，作废持久之后任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存；屏障只到一块盘归 C6（块层语义假设写错），两块盘来自两个池的起点不在射程里。
