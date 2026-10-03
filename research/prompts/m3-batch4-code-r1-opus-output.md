# m3-batch4-code-r1 攻方腿（Opus）报告（2026-10-02）

攻击面：正文第四节分工表攻方那一行——C2 四处已知分歧、C5，兼攻 C1。判据是正文第五节三句。被判对象读冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`（`sha256sum -c` 全部 OK）；下面引的 `crates/`、`.claude/kb/` 行号在冻结副本与主工作区两边逐字节相同（`cmp` 过的十份文件），按仓根写路径。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-batch4-code-r1-opus-model/`。复跑：`bash research/prompts/m3-batch4-code-r1-opus-model/rerun.sh <空的草稿目录>`（拷冻结副本、打 checker 的差分口子、放进原型测试；只跑 `singlefs-harness` 的测试二进制，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 7`）。

```
0851486e663ea893cbbaabd3f8f7c0e38232ceaaa47db28e8bf21d28d4d131a8  attack_b4_opus.rs
477eb1e2e6eeeb2c60f3bcd6f9cff087b8fc651cf4ca2931a5182773ca448c77  fix-f-equals-format-constant.patch
794a65ef8535a29d72bbdcd51a66a7b8ded1ff07a21ef1b371e34363b69e9310  rerun.sh
41ac76b317f17f6f32d5b74bad463c9109c244e3b1bf9f9f19ad5864facd78c7  run-c21-c22.log
205a447e5458a48b4736ad1b4decdbadb46c4d1901e70261319f42d439acedef  run-fix-f.log
7a66607c5d1cfca3abe12c03d5662077eb595cee324ceb1a1d6837accb545aa6  run-geom.log
202259e31b144085d2cc7e8665c57567f3d57f0a437c83c0c8bb77852a40bd76  run-h1.log
5b8203a6c60c666572133d964be61268b46c18214e01b8046e2e378142cde7fb  run-h2.log
8d1e60d13ec435fceb08a4cac87e87e66ec6ecbc48af469fe40f199bc35346e4  run-rh-allocation-wall.log
47fb659d417962b3a909e77ca280afa2b9ce89260989906bc80d546e441ee857  run-rh-fast.log
9afdda296993b2c6efd0c23a917b66291e6f9d72f39cc1ca2e15191f735e3449  run-rh-reuse.log
669dd2f011c2ab5a47755d7d92234cb508635b2a4deb98cc78bf73395c8d17f3  run-rh-rollback.log
ed04a29215f5f239c41b40da53ba487764221fd17d0f0cd64c8721018e0dbd81  run-rh-unit-area-wall-admission.log
e90111a69893821e1d16905eb115d748374af22dc37472f631e591ed40880827  run-rh-unit-area-wall.log
86ee65ff37ba9669975784284a3adde1b5c560ef5676d9b7a84e84581ceed23c  run-seed.log
64846759178946b8845dbfaf1c64083b5892f8871fc6370972f2951e85534147  walk-differential-hook.patch
```

- `attack_b4_opus.rs`：原型测试，放进副本的 `crates/singlefs-harness/tests/`。盘全是内存稀疏盘（`SparseBlockDevice`），起点池建一次、之后从内存拷。
- `walk-differential-hook.patch`：只在副本里给 `crates/singlefs-checker/src/walk.rs` 末尾加一个 `pub fn attack_replay_outcome_for_differential`，照 `check_pool_image` 里 I-4.1 那一段的取法调私有的 `replay_the_next_mount_would_do`，交回它落到的 (实例, txg)。不改任何判定。
- `fix-f-equals-format-constant.patch`：C2-1 改法甲，只在副本上量过（下文）。量完已还原。
- 日志都是这一次真跑出来的原样输出。

## 各格判定一览

| 格 | 判定 | 判据分句 | 一句话 |
|---|---|---|---|
| C2-1 I-8.1 的 F | **打中** | 第五节第 3 句（同一张镜像 checker 与 core 判得不同）；按「一处 = 一个槽」读也满足第 2 句 | 四槽 F 改成 2（或 4）、在飞上限与最坏占用按槽里的 F 算：checker 65 条一条不红，core 只读恢复与可写挂载都整池拒（`JournalSafetyFactorNotTheFirstVersionValue`）。条款字面站在 core 一边 |
| C2-1 附带 | 打中（分歧，交主 agent） | 第 3 句 | I-8.1 三个字段不等时 core 走 `SystemConfigurationValueRefused` 整池拒；checker 只红 I-8.1、其余照判（不适用 6 条，不是整池不作保） |
| C2-2 I-4.1 读环 | **打中，镜像手造** | 第 3 句 | 盘 0 计数器 4 换成实例 2 的记录、盘 1 仍是 B：checker 的重放一条都不施加（I-4.1 不适用、0 红），core 施加 B；再把盘 1 那份 B 的新根段树表校验和改坏，core 恢复失败（`UnitUnreadable { slot: 50264 }`），checker 照旧 0 红。可达性：第一版两盘池上推的不可达 |
| C2-3 R-独 对 `effective_root` | 没打中（一次抽样） | — | 两段历史、400 个崩溃状态逐个差分，400 个一致；读代码只找到一处择根平局的次序差，按 I-7.17 推的不可达 |
| C2-4 R-换 三处读法 | 误红没打中（一次抽样）；「放过」那一支是推的 | — | 384 个二次崩溃状态再挂一次，读出的内容 384 个相同、再挂被拒 0、做一次读不出 0。读代码：二次崩溃之后再挂被拒只计数，而第二截对第一次挂载被拒是按模型判的，两截不对称 |
| C5 测试助手与两条改期望 | 没打中（一次抽样） | — | 助手只抄根的四段与重算反向链，不碰点名项；调用点里没有一张坏镜像在调它之前改过记录。两条改期望按条款推得出 |
| C1 合法镜像误红、坏镜像漏红 | 没打中（一次抽样），C2-1 那一格除外 | — | mkfs 收下的 87 组几何、两段历史 400 个崩溃状态、随机历史六档 288 段，checker 0 红 |

腿的改法都只在我的模型上量过、被攻过零轮。

## C2-1　I-8.1 的 F：checker「F ≥ 2」对 core「== 3」

### 条款与两边的代码

- 条款：`.claude/kb/invariants.md:90`「安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，读者要求 F ≥ 2），任一不等判红」。「任一不等」罩着三个字段，F 那一格的等式另一边是格式常量；括号里那句说的是常量本身要 ≥ 2。按字面，core 对着条款。
- checker：`crates/singlefs-checker/src/image.rs:838`「safety_factor >= JOURNAL_SAFETY_FACTOR_LOWER_BOUND,」；在飞上限按槽里的 F 算，`crates/singlefs-checker/src/image.rs:846`「(geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).checked_div(safety_factor);」。
- core：`crates/singlefs-core/src/recovery.rs:847`「if u64::from(recorded.safety_factor) != JOURNAL_SAFETY_FACTOR {」，经 `crates/singlefs-core/src/recovery.rs:917`「.map_err(|value| RecoveryFailure::SystemConfigurationValueRefused {」整池拒。

### 镜像

新池新建文件之后的干净镜像（内存盘，mkfs → 可写挂载 → 第一个文件），改系统配置槽、重封整槽校验和：
- 四槽 F 改成 2，在飞上限改成 ⌊196608 ÷ 2⌋，最坏占用 = 上限 × 4096（三个字段按槽里的 F 自洽）；
- 同上，F = 4；
- 只改盘 1 槽 1（不是择到的那一槽）：F = 2，另两个字段按它算；
- 对照：只改盘 0 槽 0：F = 1。

### 输出（`run-c21-c22.log` 原样）

```
C2-1 clean: checker violated=[] core=(1,3) file 3000 bytes
C2-1 F=2 four slots: checker violated=[] I-8.1=Holds I-7.13=Holds not_applicable=6 | core recover=none failed SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } } | core mount_writable=Some("Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } })")
C2-1 F=4 four slots: checker violated=[] I-8.1=Holds I-7.13=Holds not_applicable=6 | core recover=none failed SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 4, first_version_safety_factor: 3 } } | core mount_writable=Some("Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 4, first_version_safety_factor: 3 } })")
C2-1 F=2 dev1 slot1 only: checker violated=[] I-8.1=Holds I-7.13=Holds not_applicable=6 | core recover=none failed SystemConfigurationValueRefused { device: DeviceIdentity(1), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } } | core mount_writable=Some("Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(1), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } })")
C2-1 F=1 dev0 slot0 only: checker violated=[("I-8.1", "盘 0 系统配置槽 0 的安全系数 F = 1，读者要求 F ≥ 2")] I-8.1=Violated("盘 0 系统配置槽 0 的安全系数 F = 1，读者要求 F ≥ 2") I-7.13=Holds not_applicable=6 | core recover=none failed SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 1, first_version_safety_factor: 3 } } | core mount_writable=Some("Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 1, first_version_safety_factor: 3 } })")
```

两件事：
1. F = 2、F = 4、只坏一个非择到的槽：checker 65 条一条不红，core 两个入口都整池拒。条款按字面要 checker 红 I-8.1。
2. 附带：F = 1（以及既有坏镜像 ①②③ 那种三个字段之一不等）时 checker 只红 I-8.1，其余照判，不适用只有 6 条；core 这时整池拒。I-7.13 那一行对 core 整池拒的值写的是 `.claude/kb/invariants.md:67`「与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）」——checker 对 I-7.13 那八项是「其余一律报不适用」，对 I-8.1 这三项不是。core 把这三项并进了同一个「读者不收」的出口，checker 没并，两边对「这张镜像挂不挂得上」判得不同。

### 四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨：两条臂是「F ≥ 2、上限按槽里的 F 算」与「F == 格式常量」，F = 2 / 4 的镜像上一条判成立、一条判拒，`F = 1` 的对照上两条一起红 |
| 系统当时看不看得到判别它的东西 | 看得到：F 是系统配置槽偏移 357 的 4 字节（`crates/singlefs-core/src/system_configuration.rs:54`「const JOURNAL_SAFETY_FACTOR_OFFSET: u64 = 357;」），两边都读 |
| 满足判据字面的哪一句 | 第 3 句「checker 与 core 在同一张镜像上对同一件事判得不同」整句满足。第 2 句「只改一处的坏镜像」：只改一个槽、三个字段一起改；只改 F 一个字段时 checker 的在飞上限那一格会先红，所以按字段数它不是「一处」，按槽数是 |
| 改法在打中的那几格上还中不中 | 下表 |

### 改法（都只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：checker 的 F 那一格改成 `safety_factor == JOURNAL_SAFETY_FACTOR`（`fix-f-equals-format-constant.patch`，一行） | F = 2 / 4 / 只坏一个槽那三格变红；既有 I-8.1 坏镜像照旧只红 I-8.1。**附带那一格（core 整池拒、checker 其余照判）不起作用** | 量过，副本上的输出见下 |
| 乙：checker 把 I-8.1 三个字段不等也归进「这一版读者不收的值」，其余不变量整池报不适用，与 I-7.13 那八项同一个出口 | 附带那一格；与甲一起才罩全 | 推的，没实现。改了之后 I-8.1 的四份既有坏镜像都会多出「其余全不适用」，那几条用例的断言要跟着改 |
| 丙：core 退回「F ≥ 2」、在飞上限按槽里的 F 算 | 让两边在 F = 2 / 4 上一起判成立 | 推的、不建议：违条款字面；core 的重放与发布都按格式常量算在飞上限（`recovery.rs` 的 `replay_journal` 调 `journal_in_flight_record_limit(ring_bytes)`），槽里的 F 只是被核的值 |

甲在副本上的原样输出（`run-fix-f.log`，消息文字没跟着改，仍写「F ≥ 2」）：
```
C2-1 F=2 four slots: checker violated=[("I-8.1", "盘 0 系统配置槽 0 的安全系数 F = 2，读者要求 F ≥ 2")] I-8.1=Violated("盘 0 系统配置槽 0 的安全系数 F = 2，读者要求 F ≥ 2") I-7.13=Holds not_applicable=6 | core recover=none failed SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } } | core mount_writable=Some("Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } })")
test each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant ... ok
```

推翻条件：主 agent 判定 I-8.1 那一行的「读者要求 F ≥ 2」是对盘上 F 的要求（F 可以不等于常量）——那时打中的是 core（拒了合法镜像），交回的分歧不变、改法换成丙。

## C2-2　I-4.1 读环：checker 按计数器，core 按 (实例, 计数器)

### 两边的代码

- checker：`crates/singlefs-checker/src/walk.rs:6198`「first_copies.entry(*counter).or_insert(record);」——每个计数器只留按盘次序第一块盘上读得出的那一份，别的实例同一计数器的那份被丢掉。反向链找前一条也走这张表：前一条若是别的实例就放行，`crates/singlefs-checker/src/walk.rs:5281`「Some(_record_of_another_instance) => true,」。
- core：`crates/singlefs-core/src/recovery.rs:3066`「records.insert((record.instance, record.counter), record);」；反向链按同实例找，`crates/singlefs-core/src/recovery.rs:3101`「let Some(previous) = records.get(&(record.instance, previous_counter)) else {」。

### 镜像（手造）

起点「B 的记录两盘都落了、根槽没落」（A 之后覆盖写一次，写表里根槽 FUA 之前的写全落）。core 的 `acquire_instance` 先把两盘系统配置取号到实例 2（I-7.7 不先红）；盘 0 计数器 4 那一格换成一条实例 2、txg 5 的记录（B 那条改实例、txg、反向链 0、事务号 1、带提交标记与末条标志、点名项清空，`to_bytes` 重封）；盘 1 那一格仍是 B。变体：再把盘 1 那份 B 的新根段里树表指针两条位置条目的校验和各加 1、重封记录头（同 `checker_known_bad_images.rs` 里 I-4.1 坏镜像的改法，只改盘 1 那份）。对照：不造实例 2 的记录，两盘的 B 都这样改坏。

### 输出（`run-c21-c22.log` 原样）

```
C2-2 B-record-only (txg 4, counter 4): checker replay=(1,4) I-4.1=Holds violated=[] | core=(1,4) file 4100 bytes
C2-2 forged (dev0 counter 4 = instance 2 txg 5; dev1 = B): checker replay=(1,3) I-4.1=NotApplicable("所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2") violated=[] | core=(1,4) file 4100 bytes
C2-2 forged + dev1 B new tree table unreadable: checker replay=(1,3) I-4.1=NotApplicable("所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2") violated=[] | core=(1,4) failed UnitUnreadable { slot: SlotNumber(50264) }
C2-2 control (no forging, both copies of B unreadable tree table): checker replay=(1,4) I-4.1=Violated("所选根（实例 1、txg 3）之上重放 1 次发布、1 条记录（末条计数器 4）施加出的那一版走不完：树表单元 两份都读不到对得上的") violated=[("I-4.1", "所选根（实例 1、txg 3）之上重放 1 次发布、1 条记录（末条计数器 4）施加出的那一版走不完：树表单元 两份都读不到对得上的")] | core=(1,4) failed UnitUnreadable { slot: SlotNumber(50264) }
```

两边分开：同一张镜像上 checker 的重放落在 (1,3)、core 落在 (1,4)；变体上 core 读不出（`UnitUnreadable`），checker 65 条 0 红，而对照（不造实例 2 的记录）checker 照常红 I-4.1。65 条里也没有一条判「两块盘同一计数器的两份不是同一条记录」。

### 可达性（推的，没量）

第一版两盘池上造不出：同一计数器要被两个实例各写一次，后一个实例取号时得看不见前一个实例在那个计数器上的记录。下一个计数器取环里全部读得出的记录的最大值加一（`crates/singlefs-core/src/mount.rs:4679`「let next_counter = records」那一段），可写挂载要两块盘都在（盘数与系统配置的设备数不等就拒，`crates/singlefs-core/src/mount.rs:590`「DeviceCountDiffersFromTheSystemConfiguration {」）。三块盘以上、或允许降级可写挂载之后，一块盘缺席时写下的记录在它回来后就会形成这个形状。两段历史 400 个崩溃状态里 R-独 与 core 一个都没分开（C2-3），与这个推断相容。

### 四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨：「按计数器取第一份」与「按 (实例, 计数器) 取第一份」在这张镜像上落到不同的版本 |
| 系统看不看得到 | 看得到：两份记录都自证过、fsid 是本池的 |
| 判据哪一句 | 第 3 句。变体也满足「坏镜像该红不红」的意思，但改了两处（造记录、改坏 B），不是第 2 句字面的「只改一处」 |
| 改法 | 甲：`records_the_replay_reads` 改成按 (实例, 计数器) 建键，重放只看所选根那个实例的；反向链前一条按同实例找（与 core 同一个读法）。推的，没实现。乙：不改读法，改法写进 I-4.1 的射程注「两盘同一计数器坐着不同实例时不判」——这一格就从「漏红」变成「不适用」。推的 |

## C2-3　I-4.1（R-独）与 core `effective_root` 的逐状态差分

### 做了什么

副本里给 checker 开了一个口子（`walk-differential-hook.patch`），对每个崩溃状态：checker 的 R-独 落到哪个 (实例, txg) 对 core `recover(…, JournalPolicy::Consult)` 交回的 `effective_root`。崩溃状态照层 0 的枚举域取（更早的段整段落、当前段每个非空子集、更晚的段一个不落），段由 harness 的 `writes_and_segments` 切：

- H1：A 之后覆盖写 B（实例 1）。B 从第一条记录那一段起的每个状态（8 个，第一次崩溃）；每个状态上起一次可写挂载（实例 2，取号、写行、暖机）并录下，挂载那一段的每个状态（二次崩溃）。
- H2：B 崩在根槽之前 → 实例 2 可写挂载做完（重放 B、写行、暖机）→ 实例 2 覆盖写 C。C 从第一条记录那一段起每个状态（8 个），每个上起实例 3 的挂载、挂载那一段每个状态。

### 输出（`run-h1.log`、`run-h2.log` 原样摘行）

```
H1 FIRST first differential agree: 8  例: none: (1,3)
H1 SECOND second differential agree: 216  例: first none second none: (1,3)
H2 FIRST first differential agree: 8  例: none: (2,7)
H2 SECOND second differential agree: 168  例: first none second none: (2,7)
```

400 个状态全一致，`DIFFER` 一行都没有。其中 I-4.1 真被评估（重放施加了一次发布）的有 3 + 69 + 3 + 51 = 126 个，都判成立。

### 读代码找到的一处差（推的不可达）

择根平局：checker `crates/singlefs-checker/src/walk.rs:6667`「.max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))」——`max_by_key` 在几条键相等时取最后一条；core 的择根核 `crates/singlefs-replay-prefix/src/lib.rs:4002`「Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0」只在严格更新时换，取第一条。两个槽坐着 (txg, 实例) 相同而内容不同的根才分得开；I-7.17 把根槽钉在 txg 算出的位置上，同一个 txg 在 R × S 个槽里只有一个位置，合法镜像上造不出（推的）；mkfs 种的第 0 代三条根量过逐字节相同（`run-seed.log`：「distinct record bytes 1」，checker 与 core 都落在 (0,0)），「全是第 0 代」那一态上平局不改结局。只在 I-7.17 已经红的镜像上两边才可能选出不同的根，没造。

推翻条件：层 0 全量上 R-独 与 `effective_root` 有一个状态不一致（命令见「要主 agent 跑的命令」第 2 条）。

## C2-4　I-8.4 R-换 的三处读法

### 做了什么

C2-3 那两段历史里每个二次崩溃状态上，照 `crash_injection.rs` 的 R-换 自己做一遍：「做一次」= 第一次崩溃镜像 + 挂载那一段的写全落、再只读恢复（同 `crates/singlefs-checker-tier/src/crash_injection.rs:1257`「pool_after_the_mount.apply_writes(mount_writes);」那一读法）；「做两次」= 二次崩溃镜像上再起一次可写挂载、再只读恢复。比读出的内容；再挂被拒、做一次读不出各记一笔。与 `crash_injection.rs` 的差别：它在挂载的每一段只摆一个二次崩溃（`crates/singlefs-checker-tier/src/crash_injection.rs:1679`「挂载的每一段各摆一个二次崩溃」），我把挂载那一段的崩溃状态枚举全（全是单元写的段除外，见「限度」第 1 条）；空间准入两边都关。

### 输出（`run-h1.log`、`run-h2.log` 原样摘行）

```
H1 SECOND R-换 same content: 216  例: first none second none: once=Some(Some(3000)) twice=Some(Some(3000))
H2 SECOND R-换 same content: 168  例: first none second none: once=Some(Some(4100)) twice=Some(Some(4100))
```

两份日志里没有 `R-换 DIFFERENT content`、`R-换 remount REFUSED`、`R-换 once read nothing` 任何一行：384 个二次崩溃状态读出的内容都与做一次相同，再挂一次都挂上了。

### 三处读法逐条

| 读法 | 会不会误红 | 会不会放过 |
|---|---|---|
| 基准取「挂载写全落在第一次崩溃镜像上再只读恢复」 | 384 个状态没误红（量过） | 推的：基准不含挂载之后那次发布，二次崩溃也只摆在挂载那一段里，两边对得齐，没找到放过的形状 |
| 做一次读得出、做两次恢复失败判红（`compare_content_read_once_and_twice`） | 没误红（量过） | 无 |
| 三种只计数不判：没崩过的挂载被拒、之后只读恢复失败、二次崩溃之后再挂被拒 | — | **推的、有一格**：二次崩溃之后再挂被拒只计数（`crates/singlefs-checker-tier/src/crash_injection.rs:1910`「.entry(RemountIdempotenceNotJudged::TheRemountWasRefused)」之后直接返回），第三截里别的判法（只读恢复对模型、池级 checker、记录核对器）都不起可写挂载，所以「崩在挂载半途之后再也挂不上」这一类实现错在崩溃注入里不会红。第二截对第一次挂载被拒是按模型判的：`crates/singlefs-checker-tier/src/crash_injection.rs:1558`「Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))」，模型在单元区墙之外不许拒。两截不对称。另外两种只计数的情形，前一种第二截已经判，后一种（做一次读不出）之后那次发布会盖过去，推的，没造出反例 |

四句：没打中误红，不适用。「放过」那一格是读代码推的，没造出一个实际被放过的实现错；改法（推的、没量）：二次崩溃之后再挂被拒按第二截同一条规则判——单元区墙之外报「模型说该成、实现拒了」；或至少把 `TheRemountWasRefused` 的计数在快档报告里设成必须为 0。

推翻条件：主 agent 带前缀跑崩溃注入快档（「要主 agent 跑的命令」第 1 条），报告里「二次崩溃之后再挂」那一行的「不同」不是 0。

## C5　测试助手同步 journal 新根段、两条改期望

### 同步助手会不会把坏镜像修好

- 助手 `crates/singlefs-harness/tests/common/mod.rs:504`（`copy_each_root_into_the_new_root_segment_of_its_journal_records`）只改两样：同 (实例, txg) 有自证根的记录，新根段四段照根抄；前一条被改过、同实例的，反向链重算。点名项、事务号、标志、序号一个字节不动。
- 调用点：`common_tree_split/mod.rs` 的 `propagate_the_new_checksum` 末尾、`tree_table_ordering_…rs` 一处、`checker_known_bad_images.rs` 六处（238、1436、3139、4151、5180、6526 行）。逐处看了调用之前改的是什么：都是改单元或改根（树表指针、映射根指针的出生树、F、树 ID 水位），没有一处在调它之前改过记录。所以今天的语料里没有一张坏镜像的故障被它抹掉。
- 会抹掉的形状（推的，今天没有用例走到）：先改记录新根段、再调助手（I-8.10 的坏镜像若哪天改走 `propagate`，就会被修回成立）；环转过一圈之后，助手按 `crates/singlefs-harness/tests/common/mod.rs:530`「for counter in 1..=record_slots_in_the_ring {」按槽位读、不核读到的记录计数器是不是这一个，又在第一个读不出的槽就停（`crates/singlefs-harness/tests/common/mod.rs:532`「let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {」），跨圈的记录会被按槽位相邻重算反向链或漏同步。今天调它的镜像环里都只有几条记录，走不到。环长写死默认值（`crates/singlefs-harness/tests/common/mod.rs:524`「let record_slots_in_the_ring = JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES;」），非默认环长的镜像上它读错位置。

### 两条改期望

| 用例 | 新期望 | 依据站不站得住 |
|---|---|---|
| `chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant` | 多 I-1.2、I-3.9 | 站得住（按条款推）：坏链指针让每条候选根又经第 1 片引用 mkfs 那一片（槽 50176），它带着写行那次发布记下的释放代；I-3.9 那一行「若每一条有效根都还引用它，它不许带已释放标志」照字面红。I-1.2：指针抄第二片的出生身份、被指单元头是 mkfs 那一片的，按收窄后的「实例表各片与指针比出生身份」红。两样都不是这条用例的目标、也不盖住目标 I-1.1 |
| `c533_row_publish_record_persisted_without_its_root_…` | I-8.11 由成立变不适用 | 站得住：两份孤单元都改坏之后，点名项的位置条目没有一条校验和对得上，I-8.11 那一行「校验和不相符的位置条目不判」，一条都判不到就是不适用。I-4.1 在改坏前后都应是不适用（写行记录是实例 2 的，所选根是实例 1 的；推的，这条用例我没跑），「其余逐条相等」那条断言不受 I-4.1 影响 |

四句：没打中，不适用。

线索（不在 C5 的两条改期望里，交主 agent，推的）：同批新加的 `rewriting_an_older_root_rechains_the_later_record_of_its_instance_and_every_verdict_stays_as_on_the_clean_image` 把「A 的根（非最新的候选根）水位压到 mkfs 时的水位」这张镜像钉成一条都不红，依据是 `crates/singlefs-harness/tests/checker_known_bad_images.rs:6501`「B 的根是最新那条、水位照旧盖得住全池出现过的树 ID」。按 I-7.8 今天的字面（全部根取 max、另判最新那条）它确实不红；但它是回退候选集里一条水位低于自己发过的树 ID 的根，回退到它之后会重发号。这一格条款没写，用例把「不红」钉成了期望，以后补判法时这条用例会挡路。

## C1　合法镜像误红、坏镜像漏红（兼攻）

| 取样 | 量 | 结果 |
|---|---|---|
| mkfs 几何扫描：pbs ∈ {512, 1024, 2048, 4096} × io_min ∈ {0, 512, 1024, 2048, 3072, 4096, 6144, 8192, 12288, 16384} × 环长 ∈ {1 MiB, 3 MiB, 默认}，槽距取 4096 向上取到判定宽度的整数倍 | 120 组：mkfs 收 87、拒 33（都是 `FixedStructureSlotNotAlignedToTheDecisionWidth`） | 收下的 87 组 mkfs 之后与新池新建文件之后 checker 0 红，core 都读出文件（`run-geom.log`） |
| C2-3 那两段历史的崩溃状态 | 第一次 16、二次 384 | 0 红（`run-h1.log`、`run-h2.log` 没有 `RED` 行） |
| 随机历史六档（`random_histories` 的六条慢档，`--include-ignored --exact`，副本里带合入后的 checker） | 96 + 48 + 48 + 32 + 32 + 32 = 288 段 | 「新发现 0」六档都是（`run-rh-*.log`）；新加的 16 条每条都真被评估过，I-4.1 除外：快档里它「判绿 0 次、不适用 1745 次」（随机历史每一步都落在有根的版本上，最新根之上没有记录） |

四句：没打中，不适用。C1 里唯一的打中是 C2-1 那一格（I-8.1 的 F）。

## 要主 agent 跑的命令（checker 档，这条腿不跑）

1. R-换 在崩溃注入上的误红与「再挂被拒」计数（checker 档报告「交主 agent 带前缀跑」第 5 条同一条），工作目录仓根、打好第 4 批补丁之后：
   `SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker-tier --test crash_injection_campaign -- --nocapture crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed`
   看报告里「二次崩溃之后再挂 … 次（I-8.4 的 R-换）：读出的内容与没崩过的那一次相同 a、不同 b；判了的按段 …；没判的 …」那一行：b 要等于 0（不是 0 推翻 C2-4 的「没误红」）；「没判的」里 `TheRemountWasRefused` 的数若不是 0，C2-4「放过」那一格就有了实例。
2. I-4.1 R-独 与 core `effective_root` 的层 0 逐状态差分（判决 `research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行要的，checker 报告第五节第 6 条说没做）：今天没有现成的命令。要先在 checker 里开一个与 `walk-differential-hook.patch` 同样的口子（不改判定），再在层 0 观察者里每个状态比它与 `recover(…, JournalPolicy::Consult).effective_root`。这是实现活，不是这一轮能交的命令；做出来之后跑层 0 快档（`SINGLEFS_HEAVY_TESTS=commit`）看不一致的个数，不是 0 推翻 C2-3。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| R-独 与 core 在合法崩溃态上落到不同的根 | H1、H2 两段历史：第一次崩溃 16 个、二次崩溃 384 个（全是单元写的段整段落） | 400 个全一致 |
| 择根平局次序（`max_by_key` 取最后、核取第一） | mkfs 刚写完那一态（只有第 0 代三条根） | 三条逐字节相同，两边都落在 (0,0) |
| R-换 在合法二次崩溃上误红 | 同上 384 个 | 内容全相同，再挂被拒 0、做一次读不出 0 |
| 新判法在合法镜像上误红 | 上面 400 个、mkfs 收下的 87 组几何、随机历史 288 段 | 0 |
| 同步助手抹掉坏镜像的故障 | 八个调用点逐处读 | 调之前都不改记录，抹不到 |
| 两条改期望依据不成立 | 两条用例按条款推 | 推得出 |
| I-7.5 mkfs 收下而 checker 红（上一轮 A3 那一形） | 120 组几何 | mkfs 先拒的 33 组之外 0 红 |

按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，上表每一行都只抽了一次样，不支撑「没有」。

## 这条腿自己的限度

1. **崩溃状态没枚举全，缩了的是历史起点**：B、C 那次发布从第一条记录那一段起取（之前的单元写段整段落）；挂载那一段里全是单元写的两段（写行 18 个单元写、暖机 16 个单元写）整段落、不取子集（开关 `ATTACK_SKIP_UNIT_ONLY_SEGMENTS=1`）。缩前：H1 挂载那一段的闭式全量合计 3 080 362、H2 34 144 390（日志里「挂载那一段的闭式全量合计」那一行）；缩后真跑了 216 与 168。估时：带单元段时一个第一次崩溃状态下的挂载那一段是 327 696 个状态，量到每个二次崩溃状态（checker + core 恢复 + 再挂 + 再恢复）约 0.5 秒（`PROGRESS first 1 second 20 elapsed 9.757214208s`，草稿目录 `run-enum-2.log`），一个第一次崩溃状态就要约 45 小时，超过 40 分钟的线，也超过单次约 10⁶ 个状态的上限。这些单元写子集上读出的内容、R-独 的落点按推理与「单元写都落了」那一态相同（单元写之后才有记录与根），没量。这一条腿全部原型跑的崩溃状态合计不到 0.2 万个（含被停掉的那一次试跑），远在 10⁷ 之下。
2. 空间准入一律关（`SpaceAdmission::SkippedByTheTestOnlySwitch`）；准入判的分支没进取样。
3. C2-2 的镜像是手造的；可达性只读代码推。
4. 只用了默认几何（512 / 4096、默认环长）跑历史；几何扫描只到新池新建文件那一版。
5. 随机历史六档用的是副本里合入后的 checker 加我的口子；口子只加函数、不改判定（`walk-differential-hook.patch`）。
6. C2-1 改法甲只在副本上量了两条用例；乙、丙没实现。

## 草稿现场

- `/tmp/claude-1000/m3-batch4-code-r1-opus/tree/`（冻结副本的拷贝，含 `target/`，639M）：已删。
- `/tmp/claude-1000/m3-batch4-code-r1-opus/` 下的日志（`run-enum-1.log` 是被我停掉的那一次全量试跑，单元段没缩；`run-enum-2.log` 是估时那一次；`run-enum-3.log` 是 H1 改写之前的同一次枚举，结果与 `run-h1.log` 逐项相同）与 `build*.log`：留着，几十 KB。
- 停过一次自己起的进程：第一次全量试跑（单元段没缩）开跑后看到挂载那一段 327 696 个状态，停了测试二进制（pid 831816，`proc.py stop`），之后列进程确认一个不剩；约 45 小时那个估时是随后 `run-enum-2.log` 量的。

## 没做什么

- 不判正推那几格（C3、C4、C6），不替主 agent 采纳。
- 没跑 checker 档（`singlefs-checker-tier` 的测试、层 0、变异整表、GPU 对拍）；要它的数写在上一节。
- 副本上量出的数不算入库装置上的数。
- 开跑前 `ps` 看到别的会话的 `e168_delete_ring_window_borrow` 装置（约 7.5 GiB 常驻）与几条 `cargo` 在跑，没有 `qemu-system`、`vm-bench.sh`、`e152`、`fio`；内存包装两次退 252（排不上），等了再跑；一次 251（峰值表），按出路设了理由变量。
