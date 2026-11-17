# m4-capacity-r3：正推腿报告（2026-11-17）

正文 `research/prompts/_m4-capacity-r3-body.md`，前两轮判决 `research/prompts/m4-capacity-r1-main-verification.md`、`research/prompts/m4-capacity-r2-main-verification.md`。方案按 `crates/` 今天的实现谈。没写模型；唯一的脚本是只读算术 `/home/fy5090/.local/share/singlefs/work/m4-capacity-r3-forward/numbers.py`（sha256 `84d2a3ac99e5f9a79c5d9a01821be0dfdb85743e4adace57dbc93a0296a07eee`），复跑 `python3 -I /home/fy5090/.local/share/singlefs/work/m4-capacity-r3-forward/numbers.py /home/fy5090/code/singlefs`，输出存同目录 `numbers.out`（sha256 `71cd9af13a4c3024e26b271a96a2c5569397dca77519a63cb15942de1d8b806e`），原样贴在第三节。

## 一句话结论

- 第 1 格：形态九句里没有做不出的。三处与已定条款或现有用例硬冲突，要连带改：C323 的「不加字段」定案与碎片度那条必红用例（本轮新找出，前两轮都没提）；D22 已定项 26「之后不可改」与「盘变大走事务」要改同一张表；写者的 `devs` 取盘表项数，抄表必须连 `devs` 一起抄。另有四处缺口（表项数没有独立计数、F 偏移由总长倒推、设备号落在 0 到 devs − 1 没人直接判、出口比正文列的多）。
- 第 3 格：505 / 513、489 与余量 23、175 到 176 GiB 三个数都核对上了，算式与原样输出在第三节；175 到 176 GiB 对应的边界是每盘 175.89 GiB（只对两盘等大成立）。
- 第 4 格：N1 取「按交进来的盘算」，容量读数换成记录值，集合不动；它不动任何现有用例，只多钉一条（第四节）。
- 第 5 格：改句终稿与两个新不变量在第五节；交用户的两条各给两版。

## 一、形态逐句过（第 1 格）

判定记法：「做得出」= 在今天的实现上能写出来；「冲突」= 与点名的条款或现有用例对不上，附两边原句。

### 落点句：每块盘的系统配置带一张逐盘容量表，按设备号 0 到 devs − 1 排，每项 8 字节整槽数

判定：做得出；与条款字面有两处冲突，见下。

- 整槽数就是 `absolute_slot_count_of_device` 的值，它的文档注释自称「这是这个量唯一的一处定义」（`crates/singlefs-core/src/allocator.rs:522`：「**这是这个量唯一的一处定义**」），函数在 `crates/singlefs-core/src/allocator.rs:527`：「pub fn absolute_slot_count_of_device(device_bytes: u64) -> u64 {」。记整槽数，出口读回来不用再换算。
- 表是池级、每盘每槽同内容，不冲突「唯一的盘级字段是「本盘设备号 4」」（`.claude/kb/decisions/22-单元原子性怎么合成.md:228`）。
- 放法：追加在回退下界 F 之后、偏移 489 起。不能插进字段表中间，也不能把 489 那个常量加长：F 的偏移是由总长倒推的（`crates/singlefs-core/src/system_configuration.rs:85`：「const ROLLBACK_FLOOR_OFFSET: u64 = SYSTEM_CONFIGURATION_BYTES - ROLLBACK_FLOOR_FIELD_BYTES;」），常量本身是 489（`crates/singlefs-format/src/lib.rs:276`：「pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;」），kb 那一行钉着它。所以 489 留作「定长部分」不动，表另立一个长度（8 × devs）。
- 容量：槽里最多 450 项（第三节输出）；分配记录树的扇出 169（`crates/singlefs-format/src/lib.rs:169`：「pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;」）先挡在前面，450 不是实际的盘数上限。
- 要动的实现：`SystemConfiguration` 与 `SystemImmutableConfiguration` 现在都是 `Copy`（`crates/singlefs-core/src/system_configuration.rs:484`、`:298` 的 derive 行），装一张随 devs 变长的表就不能再 Copy；`to_slot` 收尾的四档记账断言系统不可变配置恰好 389 字节（`crates/singlefs-core/src/system_configuration.rs:315`：「pub const FIELD_TABLE_BYTES: u64 = 389;」）要变成 389 + 8 × devs。编译会逐个报出来，不是设计问题。
- 冲突一·C323「不加字段」：`.claude/kb/checks-owed.md:229`：「2026-09-14 用户定案：不加字段」，原话是单元区末端 = 设备容量、由设备探测得出，系统配置不存「单元区槽数」这一行。逐盘整槽数就是那一行（每盘的单元区槽数 = 整槽数 − 单元区起点）。2026-11-17 的用户定案（`records/2026-11-17-里程碑四收口续做.md:31`：「C1040 容量那一半选「记容量、动格式」」）在后，但没提到 C323；那一行与它的判别用例要连带改（出口句下面写用例）。前两轮没有任何一份报告提到 C323（我在四份腿报告里按 `C323` 与用例文件名各数了一遍，0 处）。
- 冲突二·D22 已定项 26「之后不可改」：同一份文件里「| 系统不可变配置 | 389 字节 | 重建文件系统 |」（`.claude/kb/decisions/22-单元原子性怎么合成.md:246`），而变大句要求「盘变大走 D2 已定项 3 事务」改这张表。表归「系统不可变配置」时，这一档的「之后不可改」就有了例外，与 `devs` 是同一个例外（`devs` 在 D2 加盘时变大，第二轮判决记欠账）。表的归档与 `devs` 一起收，不另开第五档。

### 只在 mkfs 写句

判定：做得出。mkfs 手里已有每块盘的字节数：`crates/singlefs-core/src/make_filesystem.rs:613`：「.map(|(identity, device)| (*identity, device.size_in_bytes()))」，系统配置在 `crates/singlefs-core/src/make_filesystem.rs:794` 写：「let slot = system_configuration.to_slot();」。

- 缺口·设备号：表按设备号下标取项，mkfs 今天不直接判盘表的设备号恰好是 0 到 devs − 1，只靠区域归属写死 0 / 1 / 0 间接带出（`crates/singlefs-core/src/make_filesystem.rs:607`：「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」断言两盘）。要加：mkfs 拒设备号不是 0 到 devs − 1 的盘表（新错误成员）。
- 缺口·读侧：读者也要判「本盘设备号 < devs」。`own_device_numbers_on_disk_differing_from_the_identities_handed_in` 只核盘上写的号与交进来的身份一致（`crates/singlefs-core/src/mount.rs:7554`：「fn device_table_disagreeing_with<Device: BlockDevice>(」），不核它落在表里；盘上写 5、devs = 2 且交进来也叫 5 的盘表今天过得了，查表时越界。进 I-7.13 那张「读者收的值」表（`crates/singlefs-checker/src/image.rs:61`：「pub const SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS: &str = "I-7.13";」）。

### 抄表句：之后每条写系统配置槽的路都抄择到的那一份表，不抄探测值

判定：做得出，单点。生产代码里写系统配置槽的只有两处：mkfs（上面）与 `PoolWriter::write_system_configuration_slot`（`crates/singlefs-core/src/transaction.rs:671`：「fn write_system_configuration_slot(」），轮换、取号、回卷、补写都经它，离线修复经 `write_system_configuration_slot_carrying_the_pool_effective_floor`（`crates/singlefs-core/src/system_configuration_repair.rs:415`：「.write_system_configuration_slot_carrying_the_pool_effective_floor(」）调它。

- 表从哪来：写者每次已经现读本盘两槽（`crates/singlefs-core/src/transaction.rs:680`：「let verified_on_this_device = verified_system_configuration_slots(」）。本盘有自证槽就抄其中世代号最大那一槽的表；本盘一个自证槽都没有（补写、离线修复）才读别的盘的，取按设备次序第一块有自证槽的盘。不加 `PoolWriter::new` 的参数（调用 573 处，`grep -rn 'PoolWriter::new(' --include=*.rs crates | wc -l` 的输出），也不往 `MakeFilesystemParameters` 加字段（花括号构造约 110 处）。
- 冲突三·`devs` 也要抄：写者的设备数取 `self.devices.len()`（`crates/singlefs-core/src/transaction.rs:723`：「device_count: u32::try_from(self.devices.len()).expect("设备数"),」）。盘表与 devs 对得上的路径上两者相等；离线修复只验了池的 fsid、没核盘数（`crates/singlefs-core/src/system_configuration_repair.rs:399`：「let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());」），交一部分盘进去会写出「devs = 1、表 2 项」的槽。形态里只说抄表，没说抄 devs。改成：devs 与表同源、同长。
- `crates/singlefs-core/src/mount.rs:737` 的注释「系统配置里只有设备数与本盘设备号这两样是按盘表写的」随之要改成：只有本盘设备号按盘表写。

### 相等句：跨盘、跨槽的容量表不等整池拒

判定：做得出，两处各加一道比较。

- 跨盘：择系统配置时每盘取世代号大的那一槽，之后跨盘只比 fsid 与设备数（`crates/singlefs-core/src/recovery.rs:1901`：「|| previous.immutable.device_count != best_on_device.immutable.device_count」，不等报 `crates/singlefs-core/src/recovery.rs:1903`：「return Err(RecoveryFailure::SystemConfigurationsDisagree);」）。在这里再比表。
- 跨槽：同一块盘两槽都可择时只取世代号大的，不比内容（`crates/singlefs-core/src/recovery.rs:1888`：「(Some(zero), Some(one)) => {」）。在这个分支比两槽的表；只比 fsid 相同的两槽，别的池的旧槽留给 fsid 那一判，不改今天的错误先后。
- 错误成员另立（不并进 `SystemConfigurationsDisagree`），否则用例分不出是哪一格拒的。
- 合法状态下两槽的表恒等：写路抄表、mkfs 两槽写同一份。mkfs 写到一半崩溃（一块盘新表、另一块盘旧表）从此拒，与今天 fsid 对不上同一待遇；两盘同大小时两张表本来相等，不受影响（崩溃点重放只造等大的盘，第二轮判决 N4，推的）。

### 比对句：每块盘拿表里它那一项与探测值比；小于拒，大于放行、有效容量取记录值

判定：做得出。

- 探测值 = 读者报的字节数除以 16384 取整，与记录同一个函数；比的位置要在包上「记录值」那一层之前（取原始探测）。可写挂载放在择系统配置之后、`every_device_reaches_the_unit_area_start` 那一步之前（`crates/singlefs-core/src/mount.rs:9885`：「every_device_reaches_the_unit_area_start(&*devices, unit_area_start)?;」），只读挂载放在 `mount_read_only` 的择配置之后（`crates/singlefs-core/src/mounted_read.rs:651`：「pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {」）。所有写（取号、补写）之前，所以补写不会把一块缩了的盘洗成「记录 = 探测」。
- 只读拒不拒：形态没写，是交用户那一条，见第五节。
- 交进来的盘身份不在 0 到 devs − 1：整池拒（新错误成员），否则查表越界。

### 无自证盘句：两槽都不自证的盘照别的盘择到的表比

判定：做得出。这些盘已经在择系统配置时被点名收集（`crates/singlefs-core/src/recovery.rs:1884`：「devices_without_a_mountable_system_configuration_slot.push(device);」），比对句对它们照用择到的那张表、用盘表交进来的身份当下标。一块盘换成更小的空盘 → 小于记录 → 拒；换成更大的空盘 → 放行，之后补写把表抄进去，有效容量仍是记录值。

### 值域句：表项为 0、越界、表项数与 devs 不等整池拒

判定：做得出，第三项要改字面。

- 「0」与「越界」：合法区间取 0 < 项 ≤ 2^48（槽号 6 字节，`crates/singlefs-core/src/allocation_record_tree.rs:41`：「const LARGEST_SLOT_NUMBER: u64 = (1 << 48) - 1;」，最后一槽的号 ≤ 2^48 − 1）。小于单元区起点的项由今天的 `every_device_reaches_the_unit_area_start` 接（有效容量 = 记录之后它判的就是记录），不另立。归 I-7.13 那张「读者收的值」表，核心与 checker 各写一份、同一张表（`crates/singlefs-core/src/recovery.rs:1395`：「fn system_configuration_values_this_reader_accepts(」）。
- 「表项数与 devs 不等」：按位置排的表没有独立的计数字段，这个状态今天造不出来。只能落成两条可判的：devs 大于槽内放得下的 450 项整池拒（归 I-7.13）；表之后到槽末恒 0，由 I-2.4 判，端点从 489 改成 489 + 8 × devs（`crates/singlefs-checker/src/image.rs:1510`：「pub fn judge_system_configuration_slot_bytes_past_the_field_table_are_zero(」）。不建议另加 4 字节计数字段：它与 devs 冗余，只多出一个新的可不一致状态。
- 今天写 489 之后的字节是 0 且有用例钉着：`crates/singlefs-core/src/system_configuration.rs:911`：「fn system_configuration_is_489_bytes_in_a_4096_slot_and_round_trips() {」断言 `slot[489..]` 全 0；checker 那条 `crates/singlefs-checker/src/image.rs:2410`：「fn non_zero_byte_right_past_the_system_configuration_field_table_reddens_header_checksum_coverage(」往偏移 489 写 1 期待 I-2.4 红。表占了 489 起的字节，这两条的取点都要挪到表之后。

### 出口句：容量读数的出口全部取记录值

判定：做得出，但出口比正文列的六类多，漏一处那一处就比今天松（第二轮判决辩方条件一）。我逐个 `grep` 出的生产代码出口（测试代码与 `src/bin` 里的实验装置另算）：

- 经 `PoolReader::device_size_in_bytes`（trait 声明 `crates/singlefs-core/src/recovery.rs:132`：「fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64>;」；`singlefs-core/src` 里带这个函数名的有 20 处，含转发壳与测试读者）读的：落点在不在盘上 `crates/singlefs-core/src/recovery.rs:2847`、单元区起点判 `:2886`、journal 环扫描夹到盘内 `:5274`（`scan_journal` 在 `:5262`）、每个落点每盘一条 `:6779`、可写挂载 journal 环那一段 `crates/singlefs-core/src/mount.rs:7000`、实例表扫描 `crates/singlefs-core/src/instance_table_sweep.rs:417`、离线修复扫单元区 `crates/singlefs-core/src/system_configuration_repair.rs:59`、分配记录树读侧 `crates/singlefs-core/src/allocation_record_tree.rs:156`。正文没列的有三类：journal 环扫描（`:5274` 与 `mount.rs:7000` 两处）、实例表扫描、离线修复的单元区扫描。
- 直接问块设备、不经读者的：建空闲图 `crates/singlefs-core/src/mount.rs:3242`：「DeviceFreeMap::with_unit_area_start(*identity, device.size_in_bytes(), unit_area_start)」。mkfs 自己建的（`crates/singlefs-core/src/make_filesystem.rs:866`）取探测值是对的：它就是写表的那一边。
- checker 自己的出口：`reader.device_bytes(..)` 在 `crates/singlefs-checker/src/walk.rs` 里 8 处（`:5274`、`:5734`、`:5851`、`:7156`、`:11592`、`:20550`、`:20722`、`:22288`），checker 照约定不引 core，要自己择表、自己比。
- 验证装置里照抄的几处：`crates/singlefs-verification/src/memory_pool.rs:700`、`:738`、`:852`（候选 journal 环长）、`crates/singlefs-verification/src/bad_disk_input.rs:2137`、`crates/singlefs-verification/src/on_device_modes.rs:195`、`crates/singlefs-verification/src/verification_read.rs:693`。
- 不改的：IO 越界判（`check_aligned_and_in_range` 取块设备自己的大小，`crates/singlefs-core/src/mounted_session.rs:943` 等）。落点已由记录值的出口挡在记录之内，探测值只是更宽的外框。
- 防漏的做法有两种，都做得出：(a) 便宜：在择系统配置之后把读者包一层，`device_size_in_bytes` 返回记录值，漏掉的只有直接问块设备的 `mount.rs:3242` 一处；缺点是漏一处不报错。(b) 贵但编译器强制：把 `device_size_in_bytes` 从 `PoolReader` 里拿掉，容量只从比对句通过之后才造得出的「记录容量」类型里取。(b) 要删 `singlefs-core/src` 里约 20 个转发函数并给消费方加参数，我没编过，工作量是推的。建议 (b) 做核心与 checker，验证装置里照抄的几处跟着编译走。
- 现有用例必须动的一条（C323）：`crates/singlefs-verification/tests/fragmentation_statistics_follow_the_probed_device_capacity.rs:125`：「fn the_same_pool_bytes_on_devices_probed_three_cluster_segments_larger_redden_the_empty_segment_statistics(」。它同一份盘字节、只把探测容量加长三个整段，期待 I-3.16、I-5.2、I-3.20 红；出口取记录值之后，探测变大是放行，这一格不红。改法：这条改成「记录没变、探测变大 ⇒ 三条都不红」，另造「记录与账不符 ⇒ 红」（改记录那 8 字节再重封，记账的全空段数按原容量）当新的必红世界。同文件里「大三个整段的盘」那条（合法世界）不动，盘 mkfs 时就是那么大，记录与探测相等。

### 变大句：盘变大的部分走 D2 已定项 3 的显式几何再求值事务才用得上，第一版没有

判定：做得出，条款要补一句。D2 已定项 3 的定案句只说「加盘 / 换盘 / 掉盘」（`.claude/kb/decisions/02-RAID条带策略.md:65`：「**定案**：加盘 / 换盘 / 掉盘之后，挂载时算的那份几何由一条**显式事务**重新求值并原子发布」），同一块盘变大不在三者之内，辩方第二轮判它不罩；要在定案句里补进去（改句在第五节）。这条事务一旦有，要改的是这张表与 `devs`，所以与已定项 26「之后不可改」的冲突同源（落点句里的冲突二）。

### 条款字面与现行模型不符的一处：「多枚举一个撕裂维度」

`.claude/kb/decisions/22-单元原子性怎么合成.md:219`：「- 代价知情接受：崩溃点重放要多枚举一个撕裂维度；撕裂那次挂载拿到的是上一代的配置。」把「跨过 512」当成崩溃点重放的新代价。读模型代码：崩溃点重放的口径是每次原地覆写取三态（`crates/singlefs-verification/src/crash.rs:1551`：「/// 崩溃点重放的口径：每次原地覆写（[`TearableInPlaceOverwrites::of`]）取三态，撕裂镜像与重放接在录制流后面。」）；系统配置槽写走非写时复制分支、长 4096 大于扇区、盘上原来有非零字节，三条都成立（`crates/singlefs-verification/src/crash.rs:2343`：「if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {」往下的判据）；撕裂镜像取新旧不同那一截的前一半新、后一半旧，整槽校验和新旧都对不上（`crates/singlefs-verification/src/crash.rs:2368`：「整段校验和罩着的写（系统配置槽、journal 记录）新旧两份校验和都对不上」），与 512 无关。所以 devs = 3 起字段表跨过 512，态数不变。这一句是推的（读码），没数过态，复核不了，原因写在「要主 agent 跑的命令」末一条。推翻条件：某一档崩溃点重放（`TornImagesOfTheInPlaceOverwrites::LeftOut`）不取三态而又要判系统配置槽。

### 现有用例要动的一览（只读 grep 出来的，没编、没跑）

| 用例 | 为什么动 | 怎么动 |
|---|---|---|
| `crates/singlefs-core/src/system_configuration.rs:911` `system_configuration_is_489_bytes_in_a_4096_slot_and_round_trips` | 断言 `slot[489..]` 全 0 | 样例配置带 2 项表；断言 `slot[..489]` 之外、表之后全 0，表本身读回等于样例 |
| `crates/singlefs-core/src/system_configuration.rs:1138` `the_four_mutability_classes_budget_389_4_36_60_and_add_up_to_the_field_table_total` 与 `:1172` 那条逐档字节对账 | 系统不可变档是 389 + 8 × devs | 预算改成 devs 的函数，devs = 2 为 405 |
| `crates/singlefs-verification/tests/system_configuration_mutability_classes.rs:115`：「let padding_start = usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("489");」 | 补齐起点是 489 + 8 × devs | 起点按 devs 现算 |
| `crates/singlefs-checker/src/image.rs:2410` 往偏移 489 写 1 期待 I-2.4 红 | 偏移 489 现在是表的第一个字节 | 取点改成表之后第一个字节 |
| `crates/singlefs-verification/tests/fragmentation_statistics_follow_the_probed_device_capacity.rs:125` | 探测变大不再红（出口句里写了改法） | 反转，另造记录与账不符的必红世界 |
| `crates/singlefs-verification/tests/unequal_devices.rs:183` | 绿，但记录恒等于探测，对新比对没有判别力 | 留着；另造「重开时把一块盘截短 / 加长」两条（文件后端本来就能截） |
| `crates/singlefs-verification/src/mount_transparency.rs:176`：「usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("系统配置字段表 489 字节装得进 usize")」 | 挂载透明逐字段表只盖 [0, 489) | 把表那一段并进去：挂载不许改表 |
| `crates/singlefs-verification/tests/checker_known_bad_images.rs:5886` | 取 F 偏移用 `SYSTEM_CONFIGURATION_BYTES - 8` | 489 留作定长部分，这一条不动，只要常量不被加长 |

不动、且被钉着的：`crates/singlefs-verification/tests/fsync_drop_and_devices_without_the_selected_version.rs:262` 与 `:302`（第四节）。

## 二、未定项清单补漏（第 2 格给主 agent 的线索，攻方腿的格，我只列读码所得）

已知四条之外，正推腿读码读到的、判决里没写的：

- C323「不加字段」与它的必红用例（落点句、出口句）：条款定案与用例要连带改，要问用户的是「2026-09-14 的不加字段随这次定案作废」认不认。我按作废写了第五节的终稿，认与不认交主 agent 判。
- 表怎么随 D2 事务改（落点句冲突二）：与 `devs` 的归档同一笔欠账，建议并进 T1 一起记。
- 写者 `devs` 取盘表项数（抄表句冲突三）。
- 表项数没有独立计数（值域句）。
- 设备号落在 0 到 devs − 1（只靠写死的区域归属、读侧没判；写进 I-7.13）。
- 条款里的「多枚举一个撕裂维度」与现行模型不符（上一小节）。
- 验证装置里的记录取点（出口句末两类）：漏改的话装置按探测值判、真代码按记录值判，两边在盘被加长的镜像上分叉；没有任何今天的用例造这种镜像之外的输入，所以不红，只是对拍盲区（推的）。

## 三、三个数（第 3 格）

`python3 -I numbers.py /home/fy5090/code/singlefs` 的原样输出（`numbers.out`，退出码 0）：

```
字段表行数 46，合计 489 字节，离 512 余 23，离 4096 余 3607
格式常量 SYSTEM_CONFIGURATION_BYTES = 489，与逐行加的是否相等：True
devs = 1：497 字节，没跨 512，装得下 4096 槽
devs = 2：505 字节，没跨 512，装得下 4096 槽
devs = 3：513 字节，跨过 512，装得下 4096 槽
devs = 4：521 字节，跨过 512，装得下 4096 槽
devs = 169：1841 字节，跨过 512，装得下 4096 槽
devs = 450：4089 字节，跨过 512，装得下 4096 槽
devs = 451：4097 字节，跨过 512，装不下 4096 槽
槽内最多容纳 450 项
叶槽数 812，扇出 169，槽宽 16384
两盘各 4 GiB = 262144 槽：根层级 2
两盘各 174 GiB = 11403264 槽：根层级 2
两盘各 175 GiB = 11468800 槽：根层级 2
两盘各 176 GiB = 11534336 槽：根层级 3
两盘各 177 GiB = 11599872 槽：根层级 3
两盘各 1024 GiB = 67108864 槽：根层级 3
两盘等大时层级 2 与 3 的边界：每盘 84 × 137228 = 11527152 槽 = 188860858368 字节 = 175.890380859375 GiB
断言：边界那一槽层级 2、多一槽层级 3；175 GiB 层级 2、176 GiB 层级 3：通过
```

- **字段表今天 489 字节、离 512 余 23**：量的。算式 = kb 字段表 46 行宽度逐行相加（脚本从 `.claude/kb/layout/01-first-txn.md` 现取，合计行在 `.claude/kb/layout/01-first-txn.md:158`：「合计 **489 字节**」），与格式常量一致（`crates/singlefs-format/src/lib.rs:276`），也与现有用例断言一致（`crates/singlefs-core/src/system_configuration.rs:911`）。余量 512 − 489 = 23，4096 − 489 = 3607（`.claude/kb/decisions/22-单元原子性怎么合成.md:209` 写的「槽内余 3607」同数）。主 agent 复核：交 `system_configuration_is_489_bytes_in_a_4096_slot_and_round_trips`（命令在末节），绿即实现里 489 与断言一致。
- **505 / 513**：量的（算术，不是装置）。每项 8 字节、项数 = devs：489 + 8 × 2 = 505（余 7），489 + 8 × 3 = 513（跨 512）。前提是表追加在 489 之后、项宽 8；表项若按设备号带 4 字节号（12 字节一项）devs = 2 就是 513，这一条形态里已定为位置排、不带号。「512」指探测到的 `physical_block_size`（`.claude/kb/decisions/22-单元原子性怎么合成.md:221`：「⚠️ 现值 489 没跨 512，这条是**允许**不是**已经跨**。」）。devs = 3 起的 513 在第一版不付：mkfs 断言恰好两盘（`crates/singlefs-core/src/make_filesystem.rs:607`）。跨 512 的实际代价见上面「撕裂维度」一节：模型里没有新增的态。推翻条件：表项宽不是 8，或表不追加在 489 之后而是插进定长部分。
- **两盘各 175 GiB 到 176 GiB 跨根层级边界**：量的（算术，用的是源文件里的常量与同一条算式）。常量叶 812 槽、扇出 169（`crates/singlefs-format/src/lib.rs:163`、`:169`），根层级取使各盘格数之和 ≤ 169 的最小层级（`crates/singlefs-core/src/allocation_record_tree.rs:106`：「.find(|candidate_root_level| {」）。两盘等大时层级 2 管到每盘 84 × 137228 = 11 527 152 槽 = 175.890 GiB；175 GiB（11 468 800 槽）层级 2，176 GiB（11 534 336 槽）层级 3，对上。**只对两盘等大成立**：不等大时边界取各盘格数之和，与每盘大小的关系变了。交叉核：同一份脚本给出 4 GiB 层级 2（树高 3）、1 TiB 层级 3（树高 4），与现有用例 `two_four_gibibyte_devices_give_a_tree_of_height_three`、`two_one_tebibyte_devices_give_a_tree_of_height_four` 断言的树高一致（`crates/singlefs-core/src/allocation_record_tree.rs` 的 `mod tests`，我读了断言，没跑）。这个边界为什么要紧：今天一块盘变大跨过它，旧树的根层级与读者现算的对不上，挂载拒（报的是层级错，不是容量变了）；形态里根层级取记录值之后，盘被扩到 176 GiB 不变层级，这一格才有判别力。要钉它：在 `allocation_record_tree` 的 `mod tests` 里加一条「两盘各 175 GiB 层级 2、各 176 GiB 层级 3」（断言就是脚本末行那四条）；我没加。

## 四、第二轮 N1：根层级的设备集合取哪一个（第 4 格）

**取法：按交进来的盘算（读者列出的盘），只把每块盘的字节数换成记录值，集合不动。**

依据：

- 盘数对得上时两边本来同一个集合：可写挂载交进来的盘数不等于 `devs` 就拒（`crates/singlefs-core/src/mount.rs:7554`：「fn device_table_disagreeing_with<Device: BlockDevice>(」的第一项）。写者那一侧由分配器建空闲图时的字节数算（`crates/singlefs-core/src/allocation_record_tree.rs:134`：「pub fn of_allocator(allocator: &PoolAllocator) -> Self {」），读者那一侧由读者列的盘算（`crates/singlefs-core/src/allocation_record_tree.rs:148`：「pub fn of_reader<Reader: PoolReader + ?Sized>(reader: &Reader) -> Self {」），两边同走 `of_device_sizes_in_bytes`，只差字节数从哪来；字节数换成记录值，这个结构不动。
- 按表里的盘算，降级读也读不了：只读挂载本来不读分配记录树（`crates/singlefs-core/src/mounted_read.rs:1518`：「allocation_record_tree_root: NodePointer::empty_root(),」）；离线恢复与可写挂载重建分配器才读，读到缺盘的落点时 `placement_lies_in_the_unit_area_of_its_device` 以「分配记录的设备身份不在池里」拒（`crates/singlefs-core/src/recovery.rs:2847`：「let device_bytes = reader.device_size_in_bytes(device).ok_or(」）。所以改成按表算，只把错误名从分配记录树的 I-1.1 换成「设备身份不在池里」，没多读出任何东西，却要给 `of_reader` 的五处调用（`crates/singlefs-core/src/recovery.rs:2939`、`:2995`、`:4461`、`:7473`、`crates/singlefs-core/src/mount.rs:2759`）送表。
- 没有一句条款要求降级恢复读分配记录树：D2 已定项 13 只管降级期间的可写准入与运行期转只读（`.claude/kb/decisions/02-RAID条带策略.md:229` 附近那一节），第二轮判决写的也只是「保留今天降级读不了分配记录树的行为」。等哪天有这一句再改按表算，记欠账。

动哪条现有用例：**不动**。钉着这一格的是 `crates/singlefs-verification/tests/fsync_drop_and_devices_without_the_selected_version.rs:262`：「fn reading_the_allocation_record_tree_with_the_reader_of_only_device_zero_reports_the_child_key_violation_instead_of_panicking(」，它只给盘 0 的读者读两盘写的树，断言 `RecoveryFailure::InvariantViolated { invariant: "I-1.1", .. }`。按交进来的盘算，读者列 [盘 0]、记录值 = 盘 0 那一项 = 它的探测值，几何同今天，断言原样绿；它是这个取法的判别用例（取「按表里的盘」时这条断言会变：根的内部条目指盘 1 那一格能对上孩子，改报「分配记录的设备身份不在池里」，这是推的，没跑）。同文件 `:302` 的只读挂载只交盘 0（`read_only_mount_with_only_device_zero_handed_in_opens_the_newest_version_on_it_and_reads_it_back`）两个取法下都不读树，不受影响。

要加的（新增，不是改旧）：读者列出的盘的身份不在 0 到 devs − 1 → 整池拒，造法是把盘表里一块盘的身份改成 devs 或更大（可写与只读各一条）；没有这一条，`of_reader` 里的 `expect("读者列出的盘说得出自己的字节数")` 在查表越界时 panic。

推翻条件：找到一条条款或现有用例要求「缺盘的读者也读得出整棵分配记录树」。

## 五、条款改句终稿与 checker 不变量（第 5 格）

引的改前原句行号都在 kb 文件里现查过（`grep -nF`，命中 1 次）。改后是整句替换文本；「〔…〕」里是要用户拍板的两处的两个版本，分别写在本节「交用户的两条」。

### D12（目标介质） 已定项 4

改前（`.claude/kb/decisions/12-目标介质.md:74`）：「| 设备容量 | 变小 ⇒ **拒绝**；变大 ⇒ 放行并记 |」，「为什么」列「变小意味着可能有已分配落点掉到设备之外，而分配记录逐落点（D3（空间分配） 已定项 1）判不出「这个落点还在不在盘上」」。

改后（以「只读也拒」为底，另一版见交用户的「只读拒不拒」）：

> | 设备容量 | 每块盘的探测值（整槽数，字节数 ÷ 16384 取整）与系统配置里逐盘容量表（D22（单元原子性怎么合成） 已定项 9）它那一项比：变小 ⇒ **拒绝挂载**（可写与只读都拒）；相等或变大 ⇒ 放行、**挂载不写盘**，有效容量取记录值，变大的部分走 D2（RAID 条带策略） 已定项 3 的显式事务才用得上（第一版没有）；记录为 0 或大于 2^48、本盘设备号不小于 devs、各盘各槽的表不相等 ⇒ 整池拒绝挂载 | 变小意味着可能有已分配落点掉到设备之外，而分配记录逐落点（D3（空间分配） 已定项 1）判不出「这个落点还在不在盘上」；变大放行而不记：记录只在 mkfs 写、之后不变，分配记录树的根层级与单元区槽数才不随探测值漂 |

改前（`.claude/kb/decisions/12-目标介质.md:78`）：「第一版两盘同构、设备描述符表不写，这四条里容量与布局位退化成各盘互比（已定项 5）。」

改后：

> 第一版两盘同构，这四条里只有布局位退化成各盘互比（已定项 5）；容量按逐盘容量表比（上表），`physical_block_size` 与 mkfs 时记的值比（已定项 5）。

### D22（单元原子性怎么合成） 已定项 9

改前（`.claude/kb/decisions/22-单元原子性怎么合成.md:209`）：「46 行，合计 **489 字节**，住 4096 字节的系统配置槽（已定项 2），槽内余 3607。」；表格那一行（`.claude/kb/decisions/22-单元原子性怎么合成.md:213`）「| **系统不可变配置** |」末列「147 + 114 + 128 = 389」。

改后：

> 46 行定长部分 **489 字节**，其后追加**逐盘容量表**：按设备号 0 到 devs − 1 排，每项 8 字节小端 u64，记 mkfs 时这块盘的整槽数（盘上绝对槽数 = 字节数 ÷ 16384 取整，`absolute_slot_count_of_device` 一处定义），共 8 × devs 字节；devs = 2 时字段表合计 505、槽内余 3591，devs = 3 起合计过 512；槽内最多 450 项。
>
> 「系统不可变配置」一行末列改成「147 + 114 + 128 + 8 × devs = 389 + 8 × devs」，「从哪几段来」末尾加「；逐盘容量表」。

另加三条（接在「加密预留三格」那一条之前）：

> - **逐盘容量表只在 mkfs 写。** mkfs 之外每条写系统配置槽的路（发布末尾的轮换、取号及其回卷、取号补写、离线修复）都逐字抄本池择到的那一份表与 `devs`，不写探测值：本盘有自证槽就抄它世代号最大那一槽的，没有（补写、离线修复）就抄按设备次序第一块有自证槽的盘的。
> - **读者。** 各盘各可择槽的表逐项相等，不等整池拒；每块盘取表里它那一项与探测值比（D12（目标介质） 已定项 4）；两槽都不自证的盘照别的盘择到的表比；记录为 0 或大于 2^48、本盘设备号不小于 devs、devs 大于 450 整池拒（归 I-7.13（系统配置池级字段在读者收的范围里））；表之后到槽末恒 0（I-2.4（头校验和覆盖范围））。
> - **容量读数的出口一律取记录值**：建空闲图、读者的 `device_size_in_bytes`、落点越界判、单元区起点判、journal 环扫描、实例表扫描、离线修复的单元区扫描、分配记录树读侧与根层级；不取探测值。

改前（`.claude/kb/decisions/22-单元原子性怎么合成.md:219` 与 `:221`）：「- 代价知情接受：崩溃点重放要多枚举一个撕裂维度；撕裂那次挂载拿到的是上一代的配置。」「⚠️ 现值 489 没跨 512，这条是**允许**不是**已经跨**。」

改后：

> - 撕裂那次挂载拿到的是上一代的配置；崩溃点重放对每次 4096 字节的系统配置槽写本来就取三态（新、旧、新旧都读不出），态数与槽内字节数无关。
> - ⚠️ 定长部分 489 没跨 512；加上逐盘容量表，devs = 2 为 505 仍没跨，devs = 3 起跨过，跨过之后一次写可能落一半，由整槽校验和判出。

射程（`.claude/kb/decisions/22-单元原子性怎么合成.md:228`）「内容是**池级**的，每盘存一份副本，两槽同内容；唯一的盘级字段是「本盘设备号 4」」：不改，逐盘容量表是池级、两盘四槽同内容。该句之后加：「逐盘容量表与 `devs` 之后只有 D2（RAID 条带策略） 已定项 3 的几何再求值事务能改，第一版没有这条事务。」（与已定项 26 的衔接见本节末「连带」。）

「设备描述符表第一版不写」那一条（`.claude/kb/decisions/22-单元原子性怎么合成.md:224`）按交用户的「表的叫法」写，两版见下。

### D22 已定项 15

改前（`.claude/kb/decisions/22-单元原子性怎么合成.md:337`）：「**定案**：第一版不留间接目录单元指针（显式留白，不是留位），设备描述符表第一版不写」，接「并写明 D12（目标介质） 已定项 4 的容量 / 布局位 / physical_block_size 三条比对在第一版退化成各盘互比」。

改后（两版见交用户的「表的叫法」；两版共同的后半句）：

> 并写明 D12（目标介质） 已定项 4 的布局位比对在第一版退化成各盘互比，容量按逐盘容量表比（已定项 9），`physical_block_size` 与 mkfs 时记的值比；

### D2（RAID 条带策略） 已定项 3

改前（`.claude/kb/decisions/02-RAID条带策略.md:65`）：「**定案**：加盘 / 换盘 / 掉盘之后，挂载时算的那份几何由一条**显式事务**重新求值并原子发布」，接「**卷保持挂载**；不采纳「强制重挂」。」

改后：

> **定案**：加盘 / 换盘 / 掉盘之后、以及某块盘的容量比系统配置里记的变大之后，挂载时算的那份几何由一条**显式事务**重新求值并原子发布，**卷保持挂载**；不采纳「强制重挂」。系统配置里的逐盘容量表与 `devs`（D22（单元原子性怎么合成） 已定项 9）只有这条事务能改；挂载本身不改记录，探测值比记录大时有效容量仍取记录值，变大的部分在这条事务有之前用不上（第一版没有这条事务）。

射程（`.claude/kb/decisions/02-RAID条带策略.md:67`）「**射程**：定的是方向，那条事务长什么样在已定项 4」之后补一句：

> 容量变大本来不在「加盘 / 换盘 / 掉盘」三者之内，这里并进来：不并，改几何的路就多出一条挂载时的（写盘或只改内存），撞 `.claude/rules/fs-design.md`「分支变量不许在操作中途改变」。

### 连带（不在你列的五处里，kb 写回员要同步的，行号现查）

- D22 已定项 26 表里「| 系统不可变配置 | 389 字节 | 重建文件系统 |」（`.claude/kb/decisions/22-单元原子性怎么合成.md:246`）：389 改「389 + 8 × devs」，并写明 `devs` 与逐盘容量表是这一档里唯一由 D2 已定项 3 事务改的两项，其余之后不可改（T1 欠账收时一并归档）。
- `.claude/kb/layout/01-first-txn.md`：字段表末尾加「逐盘容量表 8 × devs」一行；合计行（`:158`）、偏移段（`:160`）、写清单里的「489 / 4096 槽」（`:61`、`:65`、`:68`、`:83`、`:399`）、`:100` 那一行；`format-const` 钉子的 489 留着，改说明成「定长部分」。
- `.claude/kb/freeze-layer-membership.md:19`（「字段表 489 字节」）、`.claude/kb/decisions/22-单元原子性怎么合成.md:20`（索引行「五段 46 行、489 字节」）。
- `.claude/kb/checks-owed.md`：C323（`:229`）「不加字段」那一句改写，I-3.16 取记录容量；C1040（`:642`）；C728（`:430`）的「[489, 4096)」端点。

### checker 新不变量（一句话）

- **I-7.20**（表相等）：池里每块盘每个自证过且可择的系统配置槽记的逐盘容量表逐项相同；不等判红，点名第一处不同的盘、槽、项。
- **I-7.21**（记录不大于现有）：每块盘现有的整槽数（镜像字节数 ÷ 16384 取整）不小于所选系统配置的逐盘容量表里它那一项；两槽都不自证的盘照所选表判；镜像里没有的盘不判。
- 若只许一句：「各盘各可择槽的逐盘容量表逐项相同，且每块盘现有整槽数不小于表里它那一项」。我建议拆成两条：两格独立红才分得出是哪一格坏（`.claude/kb/decisions/12-目标介质.md:82` 引的 E72 就是这条原则：每次只有被改坏的那一条红）。号取 7.20、7.21：checker 现有 I-7.x 最大到 7.19（`crates/singlefs-checker/src/walk.rs` 与 `crates/singlefs-checker/src/image.rs` 里的号我 `grep` 了一遍，没有 7.20 及以上）。
- 不另立：值域（0、大于 2^48、devs 大于 450、本盘设备号不小于 devs）进 I-7.13（`crates/singlefs-checker/src/image.rs:61`），表之后恒 0 仍是 I-2.4（端点移到 489 + 8 × devs）。
- 每条要有已知坏镜像、核心侧与 checker 侧在同一份镜像上一起判的用例（照 `system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 的做法）、变异行。

### 交用户的两条，各按两种答案写两版

**只读拒不拒：容量变小时只读挂载拒不拒。**

- 版本甲（只读也拒，沿用 `physical_block_size` 那一行，`.claude/kb/decisions/12-目标介质.md:73`：「| `physical_block_size` | **拒绝挂载**（可写与只读都拒） |」）：已定项 4 的改后原样（上面「可写与只读都拒」）。D2 已定项 13 射程加一句：
  > 容量变小的盘与整盘不在不同：整盘不在时那块盘读不到、其余盘上两份系统配置与每个落点的镜像副本仍自证，只读挂得上；容量变小的盘仍在池里、它的落点可能落到盘外而分配记录判不出，所以挂载拒、不进只读降级。要读数据的出路是把那块盘按「整盘不在」拔掉再挂。
  - 代价：用户要读数据只能还原盘大小或拔盘；同一块盘「变小」比「整盘不在」更难挂，D2 已定项 13 的单调性不成立。
  - 要加的用例：变小在可写与只读各拒、盘上逐字不变。
- 版本乙（只拒可写；只读放行并报出）：已定项 4 那一行的前半改成
  > 变小 ⇒ **拒绝可写挂载**；只读挂载放行并报出哪块盘小于记录（恢复与只读挂载的报告里各一个字段，形状照 `devices_without_a_mountable_system_configuration_slot`），有效容量仍取记录值，读到盘外的单元按读错处理、镜像副本兜底

  「为什么」列末尾加：「只读不拒：整盘不在也能只读挂（D2（RAID 条带策略） 已定项 13），容量变小的盘至多读不到某些单元，不比整盘不在更糟」。D2 已定项 13 挂载准入那一行的合取（`.claude/kb/decisions/02-RAID条带策略.md:226` 一带）加一项「∧ 每块盘探测容量不小于系统配置记录」，不成立即只读挂载，与其余合取同一处置。
  - 代价：多一个报告字段与一组用例；读到盘外的单元怎么记账（读错计数、见证判据）要走一遍 D23 那一套，我没核它是否改变见证判定（待核）。
  - 要加的用例：变小可写拒、只读做成并点名那块盘、读盘外落点走镜像副本。

**表的叫法：逐盘容量表与「设备描述符表第一版不写」的字面。**

- 版本甲（补一句「逐盘容量表不是设备描述符表」，不改「第一版不写」）：
  - D12 已定项 5 改前（`.claude/kb/decisions/12-目标介质.md:89`）「**设备描述符表**：第一条线两盘同构、第一版不写；加第二套布局前必须写。」，改后：
    > **设备描述符表**：第一条线两盘同构、第一版不写；加第二套布局前必须写。第一版写的逐盘容量表（D22（单元原子性怎么合成） 已定项 9）不是设备描述符表：它只有每块盘的整槽数一列，不记布局位、`physical_block_size`、`io_min`；设备描述符表写出来那天容量一列归它，逐盘容量表随那次格式变更删掉。已定项 4 的布局位比对在第一版退化成各盘互比，容量按逐盘容量表比；（后文「`physical_block_size` 那一条不再只是互比」起不变）
  - D22 已定项 15：「设备描述符表第一版不写（逐盘容量表不是它，见已定项 9），D12（目标介质） 已定项 5 那句写成「第一条线两盘同构、第一版不写；加第二套布局前必须写」并补「逐盘容量表不是设备描述符表」，」接共同的后半句。
  - D22 已定项 9 那一条（`:224`「- 第一版不留间接目录单元指针、不写设备描述符表，`devs` 留 4 字节」）后加「，逐盘容量表（8 × devs 字节，不是设备描述符表）追加在回退下界 F 之后」。
  - 代价：同一个量（容量）将来有两个家，设备描述符表那天要搬；D12 已定项 3 说「设备级几何量住系统配置里的设备描述符表」（`.claude/kb/decisions/12-目标介质.md:56`），字面上容量表就是它的一列，甲只是不叫这个名字。
- 版本乙（改那两句：逐盘容量表就是第一版的设备描述符表，只有容量一列）：
  - D12 已定项 5 改后：
    > **设备描述符表**：第一版写，且只有一列——每块盘的整槽数（D22（单元原子性怎么合成） 已定项 9 的逐盘容量表就是它的第一版）；加第二套布局前补布局位、`physical_block_size`、`io_min` 等列。已定项 4 的布局位比对在第一版退化成各盘互比，容量按它比；（后文不变）
    依据一栏的「设备描述符表第一版不写」（`.claude/kb/decisions/12-目标介质.md:95`）注明被取代，原话留在变更史。
  - D22 已定项 15：「设备描述符表第一版只写容量一列（就是已定项 9 的逐盘容量表），D12（目标介质） 已定项 5 那句写成「第一版写，且只有一列：每块盘的整槽数；加第二套布局前补布局位等列」，」接共同的后半句。
  - D22 已定项 9 那一条改成「第一版不留间接目录单元指针，设备描述符表只写容量一列（追加在回退下界 F 之后，8 × devs 字节），`devs` 留 4 字节」。
  - 代价：推翻用户两次定案里的一次（`.claude/kb/decisions/12-目标介质.md:95` 写着「设备描述符表第一版不写，随 D22（单元原子性怎么合成） 已定项 15 改写」）；与 C218「设备表单元取哪个码没定」的前提不同（乙里表住槽内、不是单元），C218 要改写。好处：一个家，将来补列不搬家。

## 要主 agent 跑的命令

四条都是读现有用例的基线，不改任何东西；判据：`TASK_SERVER_RESULT` 行 `exit=0`、`task-server/task log <号>` 里「test result: ok. N passed」N 等于点名的条数。我没跑，所有格都是「待主 agent 跑」。

- `python3 -I /home/fy5090/.local/share/singlefs/work/m4-capacity-r3-forward/numbers.py /home/fy5090/code/singlefs`，交法 `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/m4-capacity-r3-forward/numbers.py /home/fy5090/code/singlefs`，在仓根起；看 `exit=0` 与日志末行「断言：…通过」。
- `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 库测试里 3 条; 范围=3 个测试; 为什么=核 489 字节、四档预算与两盘树高三个基线数' task-server/task submit -- cargo test -p singlefs-core --lib -- system_configuration::tests::system_configuration_is_489_bytes_in_a_4096_slot_and_round_trips system_configuration::tests::the_four_mutability_classes_budget_389_4_36_60_and_add_up_to_the_field_table_total allocation_record_tree::tests::two_one_tebibyte_devices_give_a_tree_of_height_four --exact`，在仓根起。
- `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 fragmentation_statistics_follow_the_probed_device_capacity 里的 2 条测试; 范围=2 个测试; 为什么=核出口取记录值之后要反转的那条今天是绿的' task-server/task submit -- cargo test -p singlefs-verification --test fragmentation_statistics_follow_the_probed_device_capacity -- the_same_pool_bytes_on_devices_probed_three_cluster_segments_larger_redden_the_empty_segment_statistics new_pool_file_creation_on_devices_three_cluster_segments_larger_holds_the_fragmentation_statistics --exact`，在仓根起。
- `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 fsync_drop_and_devices_without_the_selected_version 里的 2 条测试; 范围=2 个测试; 为什么=核 N1 取法钉着的两条今天是绿的' task-server/task submit -- cargo test -p singlefs-verification --test fsync_drop_and_devices_without_the_selected_version -- reading_the_allocation_record_tree_with_the_reader_of_only_device_zero_reports_the_child_key_violation_instead_of_panicking read_only_mount_with_only_device_zero_handed_in_opens_the_newest_version_on_it_and_reads_it_back --exact`，在仓根起。
- 复核不了：「跨 512 不多枚举态」要数 devs = 2 与 devs = 3 两种槽长下同一格崩溃点重放的态数，今天没有按槽长参数化的入口，我没造；上面「撕裂维度」一节只凭读码。要证它，实现员在带表的实现上枚举一格新池新建文件的崩溃点重放，对比 devs = 2 与改成更长表后的态数。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 落点句 | 做得出，两处条款字面要改 | C323「不加字段」与 D22 已定项 26「之后不可改」都要改；`Copy` 与 389 记账要改 |
| 1 只在 mkfs 写句 | 做得出 | 加 mkfs 拒设备号不是 0 到 devs − 1，读侧判本盘设备号 < devs |
| 1 抄表句 | 做得出，要补 | 单点 `transaction.rs:671`；`devs` 也要抄（今天取盘表项数，`transaction.rs:723`） |
| 1 相等句 | 做得出 | 两处各加比较：`recovery.rs:1901` 跨盘、`:1888` 跨槽，同 fsid 才比 |
| 1 比对句 | 做得出 | 在所有写之前、包记录值那层之前比；只读拒不拒交用户 |
| 1 无自证盘句 | 做得出 | 用已收集的两槽都不可择的盘表，身份当下标 |
| 1 值域句 | 做得出，字面要改 | 「表项数与 devs 不等」造不出来，改成 devs ≤ 450 与表后恒 0 |
| 1 出口句 | 做得出，出口比正文多 | 多出 journal 环扫描、实例表扫描、离线修复扫描；C323 那条碎片度必红用例要反转 |
| 1 变大句 | 做得出 | D2 已定项 3 定案句补「容量变大」 |
| 3 489 / 余 23 / 505 / 513 | 核上 | 量的算术，脚本从 kb 与格式常量现取 |
| 3 175 到 176 GiB | 核上 | 边界每盘 175.89 GiB，只对两盘等大 |
| 4 N1 | 按交进来的盘算 | 不动现有用例，`:262` 那条是判别用例 |
| 5 条款改句 | 终稿已写 | D12 四五、D22 九与十五、D2 三各前后句；交用户两条各两版 |
| 5 checker | 新立 I-7.20、I-7.21 | 值域进 I-7.13，表后恒 0 仍是 I-2.4 |

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数（基线测试）没复跑，命令都在上一节。
- 没建模型、没编译、没跑任何 `cargo`；「要动的实现」「现有用例要动」两张清单都是只读 grep 与读码，编译才能全部暴露。
- 没核 GPU 判器与崩溃放量那一路对系统配置字段偏移的读法（第一轮正文第 5 格问过 GPU 判器，这一轮没分给我）。
- 没核第 2 格（未定项清单完整性）的全部，那是攻方腿的格；第二节只列我读码顺路读到的。
- 没改任何 kb、`crates/`、规则文件；草稿目录里只有 `numbers.py` 与 `numbers.out`，没有仓副本与编译目录。
