# m4-capacity-r1 正推腿报告

日期：2026-11-17。腿：three-way-forward（Sonnet）。方案按 `crates/` 今天的实现来谈。
没有写模型：这一轮的八格都是格式与处置的设计判断，没有要量的数；唯一要实测才能坐实的一句（盘变大跨层级边界今天挂不上）列在「要主 agent 跑的命令」一节，标「推的」。
没有读禁读清单里的文件。

## 〇、八格方案一页

| 格 | 选 | 一句话 |
|---|---|---|
| 1 落点 | 逐盘记本盘 | 系统配置字段表末尾（偏移 489）加「本盘整槽数」8 字节，进系统不可变配置；不写设备描述符表，不每份记全池 |
| 2 宽度量纲 | 整槽数（16 KiB 槽），8 字节小端，值 = `⌊探测字节数 ÷ 16384⌋` | 与分配器、分配记录树用的是同一个量（`absolute_slot_count_of_device`），不记盘尾不足一槽的零头 |
| 3 变小 | 拒绝挂载，可写与只读都拒；探测整槽数小于记录值即拒 | 与 `physical_block_size` 那一行同一处置 |
| 4 变大 | 放行、不写盘、不算发布；有效容量恒取记录值，多出的部分不分配；记新值归 D2（RAID 条带策略） 已定项 3 的几何再求值事务 | 「放行并记」里的「记」不能在挂载时做：记录值同时是分配记录树根层级的输入 |
| 5 记录值读不出 / 为 0 / 越界 | 为 0 或不在读者收的范围里：整池拒绝（与别的池级不收之值同一类）；盘上两槽都没有自证过的：不比，有效容量取探测值，走已有的「这块盘没有可择槽」那条路 | 零填充的旧镜像同时被这一判拒掉，不用另开 incompat 位 |
| 6 条款 | D12（目标介质） 已定项 4 的表一行与射程一句、已定项 5 定案一句，D22（单元原子性怎么合成） 已定项 15 定案一句，D2（RAID 条带策略） 已定项 2 射程补一句（定案句不用改）；连带 D22 已定项 9 与已定项 26 的字节数和「唯一的盘级字段」一句 | 改前改后见第六节 |
| 7 checker 与 GPU 判器 | 池级 checker：I-7.13 加一项（范围）、新立一条不变量（记录值不超过实际）、容量读数改取记录值；GPU 判器两份都同判 | 见第七节 |
| 8 `unequal_devices.rs:183` | 绿 | 两盘各记各的，重挂的盘与记录一致 |

## 一、实现今天的样子（我读到的，量的 = 读源码与 grep 所得）

1. 容量进地址算术只有一个口：`crates/singlefs-core/src/allocator.rs:528`：「device_bytes / SLOT_BYTES」，所在函数 `absolute_slot_count_of_device`（`crates/singlefs-core/src/allocator.rs:527`）；盘尾不足一槽的零头不算。
2. 分配记录树的根层级由「各盘整槽数」现算：`crates/singlefs-core/src/allocation_record_tree.rs:106` 的 `.find(|candidate_root_level| {` 是找最小的、使各盘叶格数之和不超过扇出 169 的层级；写侧 `of_allocator`（`crates/singlefs-core/src/allocation_record_tree.rs:134`）与读侧 `of_reader`（`crates/singlefs-core/src/allocation_record_tree.rs:148`）都从探测字节数起算，盘上不记。读到的节点层级对不上就拒：`crates/singlefs-core/src/allocation_record_tree.rs:709` 起的判，报「分配记录树节点的层级不是它的位置规定的那一层」（`crates/singlefs-core/src/allocation_record_tree.rs:711`）。
   叶 812 槽、扇出 169（`crates/singlefs-format/src/lib.rs:163`、`crates/singlefs-format/src/lib.rs:169`）。我按 `allocation_record_tree.rs` 106 行那段算式在草稿里独立重算（推的，不是装置）：两盘等大，175 GiB 时根层级 2，176 GiB 时 3。
   ⇒ 今天一块盘变大（或变小）跨过这条边界，旧树的根层级与读者现算的对不上，挂不上，报的是层级错，不是容量变了。
3. 挂载今天不比容量：只比调用方交进来的 mkfs 参数（`crates/singlefs-core/src/mount.rs:7640` 的 `MakeFilesystemParameterField::PhysicalBlockSize,`）与盘表的设备数、本盘设备号（`crates/singlefs-core/src/mount.rs:7554` 的 `device_table_disagreeing_with`）。容量唯一的护栏是盘末尾不在单元区起点之前（`crates/singlefs-core/src/recovery.rs:2880` 的 `every_device_reaches_the_unit_area_start`，可写挂载在 `crates/singlefs-core/src/mount.rs:9885` 调它）与分配记录越过单元区末尾（`crates/singlefs-core/src/recovery.rs:2837` 的 `placement_lies_in_the_unit_area_of_its_device`）。
4. mkfs 手里有每块盘的字节数（`crates/singlefs-core/src/make_filesystem.rs:611` 起的 `sizes`）却只拿最小的那块判几何（`crates/singlefs-core/src/make_filesystem.rs:462`：「let smallest = devices.iter().map(|(_, bytes)| *bytes).min().unwrap_or(0);」）；逐盘写系统配置的循环在 `crates/singlefs-core/src/make_filesystem.rs:774`，本盘设备号在 `crates/singlefs-core/src/make_filesystem.rs:778`。mkfs 断言恰好两块盘（`crates/singlefs-core/src/make_filesystem.rs:607`）。
5. 发布路径每次重写系统配置槽：`crates/singlefs-core/src/transaction.rs:671` 的 `write_system_configuration_slot` 用池级参数加这块盘的身份现拼一份（`crates/singlefs-core/src/transaction.rs:722` 的 `this_device: identity,`），世代号来自这一刻直接从盘上读的两槽（`crates/singlefs-core/src/transaction.rs:680`）。`PoolWriter::new(` 在 `crates/` 下有 573 处调用（`grep -rn 'PoolWriter::new(' --include=*.rs crates | wc -l` 的输出），所以新字段不能走 `PoolWriter::new` 的新参数，只能从盘上现读的槽里带下去。
6. 系统配置每盘两槽、字段表 489 字节、盘级字段只有本盘设备号：`crates/singlefs-core/src/system_configuration.rs:301`：「字段表里唯一的盘级字段：别的都是池级、两盘同内容」；尾部偏移 `crates/singlefs-core/src/system_configuration.rs:80` 的 `TAIL_OFFSET: u64 = 469`，回退下界 F 偏移由总长减 8 推出（`crates/singlefs-core/src/system_configuration.rs:85`）。469 与 F 之间没有别的认购：journal tail 8 + 实例代号 4 = 12，F 在 481（`.claude/kb/layout/01-first-txn.md` 字段表偏移那一段，见第六节引文）。
7. 择系统配置：`crates/singlefs-core/src/recovery.rs:1835` 的 `choose_system_configuration_among_the_slots_of` 取第一块有可择槽的盘里世代号大的那一份，别的盘只比 fsid 与设备数；所以「所选那一份」里的本盘字段只属于一块盘。逐盘读自证槽的现成做法：`crates/singlefs-core/src/mount.rs:7418` 的 `own_device_numbers_on_disk_differing_from_the_identities_handed_in`（读每块盘两槽比本盘设备号）。
8. 只读挂载入口 `crates/singlefs-core/src/mounted_read.rs:651` 的 `pub fn mount_read_only(reader: &dyn PoolReader)`（今天没有探测）；它读字节丢了的那一版会退到旧根：`crates/singlefs-core/src/mounted_read.rs:641`：「施加之后那一版三样读者的并集里有字节丢了、没有内容拒收、系统配置没见证它时」。
9. 设备描述符表在 `crates/` 里零命中（主 agent 正文已列命令，我没重跑，照引）。

`grep -rn '设备描述符表\|device_descriptor' crates/singlefs-core/src | wc -l` 我重跑的输出：0。

## 二、第 1 格：落点

**选：逐盘记本盘。** 在系统配置字段表最末（偏移 489，F 之后）加一行「本盘整槽数」，宽 8；归系统不可变配置（改它 = 重建文件系统）。

判据照 `.claude/rules/fs-design.md`@格式分支的判据：能不能把「用错了」变成「挂不上」。把「用错了」逐种列出来，看三个候选各买到哪种：

| 用错了的情形（我列的，推的） | 逐盘记本盘 | 每份记全池 | 设备描述符表 |
|---|---|---|---|
| 一块盘被截小或换成更小的盘，尾部没数据 | 挂不上 | 挂不上 | 挂不上 |
| 同上，尾部有数据 | 挂不上（今天靠分配记录越界或层级错才挂不上，报错不说容量） | 同 | 同 |
| 一块盘变大 | 放行，有效容量取记录值（第 4 格） | 同 | 同 |
| 两盘接线对调、大小不同 | 本盘设备号那一判已拒，不新增 | 多一个独立来源，但这个情形已拒 | 同左 |
| 一块盘两槽都废、同时尾部被截 | 判不了（没有记录），落回已有的补写闸与分配记录越界判 | 判得了（别的盘的副本里有） | 判得了 |

逐盘记本盘买到前两行与第三行，后两个候选只多买最后一行；最后一行的危害面已有护栏：这块盘两槽都废时可写挂载要过补写闸，闸要求这块盘带所选那一版的全部单元（`crates/singlefs-checker/invariants/03-journal.md:12` 的 I-8.5 一行写了这条闸），被截掉的若是有数据的尾部，闸过不了；被截掉的若是空尾巴，没有任何数据受损（推的）。

依据（条款）：
- D12（目标介质） 已定项 3 的判据：`.claude/kb/decisions/12-目标介质.md:56`「一个几何量能不能在池内异构，看用到它的地址算术是否已经携带设备身份」；位置条目带 dev（D19（块指针的结构与宽度预算） 已定项 1），所以容量可以逐盘不同，必须逐盘记。
- D12（目标介质） 已定项 4：`.claude/kb/decisions/12-目标介质.md:69`「凡是进入地址算术的几何量都要逐设备比对，不进地址算术的不比」。
- 设备描述符表第一版不写，用户定过两次：`.claude/kb/decisions/12-目标介质.md:89` 的已定项 5 定案句与 `.claude/kb/decisions/22-单元原子性怎么合成.md:337` 的已定项 15；`.claude/kb/decisions/22-单元原子性怎么合成.md:224` 再写一遍「第一版不留间接目录单元指针、不写设备描述符表，`devs` 留 4 字节」。表的条目宽、表单元取哪个码都没定：`.claude/kb/checks-owed.md:170` 的 C218。
- 本盘设备号是现成的先例：`crates/singlefs-core/src/system_configuration.rs:302` 的 `pub this_device: DeviceIdentity,`，逐盘写（`crates/singlefs-core/src/make_filesystem.rs:778`）、逐盘读（`crates/singlefs-core/src/mount.rs:7418`）、池级 checker 逐盘判（I-7.14，`crates/singlefs-checker/invariants/02-根环.md:21`）。「元数据块必须自描述」（`.claude/rules/fs-design.md`@元数据块必须自描述）：这块盘自己说「我是第几号、我在 mkfs 时有多少槽」。

为什么不选每份记全池：要变长（盘数 × 8），而 mkfs 断言恰好两块盘（`crates/singlefs-core/src/make_filesystem.rs:607`）、`region_devices` 是定长 3 项；固定记 2 项等于把盘数写进格式，记变长项等于提前写出设备描述符表的一半，两样都要翻用户已定的「第一版不写」。它的另一个好处（池里只有一个来源，几何不用逐盘凑）在第 5 格的落回路径里付得起：凑不齐的盘取探测值，退回今天的行为。
为什么不选设备描述符表：C218 没答（表单元取哪个码、`devs` 去留），表一写出就要连带定条目宽与单元类，是单独一条决策的量，不是这一格的量；本字段随表写出那天并进表里（格式锁定之前没有旧盘，`.claude/rules/format-evolution.md`），不留兼容。

放在尾部而不挨着本盘设备号的理由：尾部追加不挪任何已有偏移。checker 自己的偏移（`crates/singlefs-checker/src/lib.rs:413` 的 `SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET: usize = SYSTEM_CONFIGURATION_TAIL_OFFSET + 8 + 4;`）、GPU 判器的 `c.floor = rd64(base, 481u);`（`crates/singlefs-verification/src/crash_judge_common.wgsl:1062`）、E 系列装置里的偏移，一个都不动。代价：F 不再是「字段表最后一行」，`crates/singlefs-core/src/system_configuration.rs:85` 的 `ROLLBACK_FLOOR_OFFSET` 从「总长减 8」改成 `TAIL_OFFSET + 12`。字段表行序即盘上顺序、分档只管内存表示（`crates/singlefs-core/src/system_configuration.rs` 文件头第 4 行起写了四类在槽里交错），把不可变档的一个字段放在槽尾不违背这条。合计 489 → 497，仍小于 512，D22（单元原子性怎么合成） 已定项 9 说的「真加字段跨过 512 那天要多枚举一个撕裂维度」不触发。

改法（对实现的改动）：
- `crates/singlefs-format/src/lib.rs:276` 的 `SYSTEM_CONFIGURATION_BYTES` 489 → 497（kb 字段表那一行的 `format-const` 标记同改，`.claude/kb/layout/01-first-txn.md:158`）。
- `crates/singlefs-core/src/system_configuration.rs`：`SystemImmutableConfiguration`（定义在 299 行）加字段 `this_device_whole_slots: u64`，`FIELD_TABLE_BYTES` 389 → 397（`crates/singlefs-core/src/system_configuration.rs:315`）；`to_slot` 在偏移 489 写 8 字节小端并断言位置，`parse_slot`（714 行）读回；`SystemImmutableConfiguration {` 的构造点 11 处（`grep -rn 'SystemImmutableConfiguration {' --include=*.rs crates` 命中 15 行，其中 2 处是 `mount_transparency.rs:1594`、`:1603` 的整个解构、2 处是定义与 impl，其余是构造）加字段，漏一处编译不过。
- `crates/singlefs-core/src/make_filesystem.rs:774` 的逐盘循环里填 `absolute_slot_count_of_device(device.size_in_bytes())`。
- `crates/singlefs-core/src/transaction.rs:671` 的 `write_system_configuration_slot`：值取这块盘上一刻直接读出的、世代号最大的自证槽的本盘整槽数（`verified_on_this_device`，680 行已经读了）；这块盘没有一槽自证过时取 `absolute_slot_count_of_device(self.devices[index].1.size_in_bytes())`（这是补写两槽全废的盘那一条路，见第 5 格）。不能取「写的当时再探测一次」：那样每次发布都悄悄把变大记了，第 4 格说它为什么不行。

推翻它的观测：
- 如果让「每份记全池」去过第 5 格的双故障行与第 2 行，发现它们在真实路径上要比本方案多挡掉一类不被现有护栏罩住的损坏（例如补写闸放过了一块被截掉有数据尾部的盘），本格改选每份记全池。
- 如果池要在格式锁定之前支持多于两块盘，逐盘记本盘与设备描述符表之间不再有「第一版不写」可守，改选设备描述符表，本字段并进表。

## 三、第 2 格：宽度与量纲

**选：整槽数（16 KiB 槽），8 字节小端，值 = 探测字节数 ÷ 16384 取整。** 取 mkfs 实际用到的那一段，不是探测字节数。

- 量纲：地址算术只用整槽数：`crates/singlefs-core/src/allocator.rs:528`「device_bytes / SLOT_BYTES」（`absolute_slot_count_of_device` 是这个量唯一的一处定义，注释 `crates/singlefs-core/src/allocator.rs:522` 起写明）。记字节数会把盘尾零头也记进去：盘尾差几百字节的重挂（分区边界对齐、镜像文件扩了半个槽）会被拒，而这个零头不进任何地址算术；D12（目标介质） 已定项 4 射程明说「比对它们只会制造无法挂载的假红」（`.claude/kb/decisions/12-目标介质.md:78`）。现成用例钉着零头不算：`crates/singlefs-core/src/allocation_record_tree.rs:976` 的 `half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary`。
- 「mkfs 探测的值」与「mkfs 实际用到的那一段」：mkfs 没有「只用盘的前一段」的参数，单元区末尾就是整槽数（`crates/singlefs-core/src/allocator.rs:545` 起的 `unit_area_slots_of_device_starting_at` 从 `absolute_slot_count_of_device` 减单元区起点），所以两者相等；写成「整槽数」是把这条等式落在盘上，不是多一个选择。
- 宽度：8 字节。槽号在分配记录里是 6 字节（`crates/singlefs-core/src/allocation_record_tree.rs:41` 的 `LARGEST_SLOT_NUMBER: u64 = (1 << 48) - 1`），6 字节够用，但 `.claude/rules/fs-design.md`@不为省空间牺牲自包含 的默认是不省；`journal_ring_bytes` 同样是 8 字节。超出 2⁴⁸ 的值属于「读者不收」（第 5 格）。字节序小端（D22（单元原子性怎么合成） 已定项 13）。
- 买到的失败模式改善：把「盘缩小」这一类从「靠分配记录越界或根层级错间接露出来（尾部没数据时静默）」变成「挂不上，报的就是容量」。

推翻它的观测：任何一处地址算术用的是字节数而不是整槽数（`grep -rn 'size_in_bytes()' crates/singlefs-core/src` 里有一处把字节数直接当地址上界且与整槽数不等价），则该处要么也改走整槽数，要么本格改记字节数。我核了 `crates/singlefs-core/src/mounted_session.rs:943` 一类（读写范围检查用真实设备字节数），它们判的是 I/O 请求有没有越过真实设备，不是分配地址，不在这条射程里（推的）。

## 四、第 3 格：变小的处置与射程

**选：拒绝挂载，可写与只读都拒；探测整槽数小于这块盘的记录值即拒。**

- 条款：`.claude/kb/decisions/12-目标介质.md:74` 的比对表一行「变小 ⇒ **拒绝**」没有可写 / 只读的限定；`physical_block_size` 一行逐字写「可写与只读都拒」（`.claude/kb/decisions/12-目标介质.md:73`），C1040 的出路写「对不上照 D12（目标介质） 已定项 4 的处置拒」（`.claude/kb/checks-owed.md:642`）。两行同一处置，不单开一个只拒可写的分支。
- 只读也拒的实际理由：一块被截小的盘，尾部的单元读不到，只读挂载会把它当「字节丢了」，按 `crates/singlefs-core/src/mounted_read.rs:641` 那段的口径退到旧根打开（这条是读注释推的，没造镜像实测）；结果是静默读到一个旧版本。拒绝把它变成挂不上，正是 `.claude/rules/fs-design.md`@格式分支的判据 那条判据。只拒可写的话，只读这一支还要自己单独验（五条硬要求第 3 条）。
- 代价（交代清楚）：被截小的池不能再只读挂载来捞数据。出路是把盘恢复到原大小，或离线用 `recover` 与池级 checker 读镜像；「降级只读」的口径归 D2（RAID 条带策略） 已定项 13，不在这一格里开。
- 落点：一个函数放在 `crates/singlefs-core/src/recovery.rs:2880` 的 `every_device_reaches_the_unit_area_start` 旁边，逐盘取这块盘上世代号最大的自证槽（`crates/singlefs-core/src/recovery.rs:5149` 的 `verified_system_configuration_slots`），拿它的本盘整槽数比 `absolute_slot_count_of_device(reader.device_size_in_bytes(device))`，小于记录值就报新增的拒绝成员（带盘号、记录值、探测值，三个数）。三处调用：可写挂载（`crates/singlefs-core/src/mount.rs:9885` 那一句之前）、只读挂载（`crates/singlefs-core/src/mounted_read.rs:680` 的 `mount_read_only_from_the_system_configuration_choice`）、冷恢复入口 `recover`。另一个会话给 `mount_read_only` 加的 `physical_block_size` 探测比对放进同一个函数、同一个拒绝枚举，不另起第二个入口。
- 强制进入：用例造一块比记录小一槽的盘，三个入口都要拒；造等大的、造一块大的，都要挂上。变异表加行（`crates/mutations.tsv` 的登记形态）：比较写反、比较换成字节数、只读入口不调。

买到什么：把「缩盘」从「尾部有数据才在分配记录越界或层级错那里间接露出来」变成「恒挂不上，报容量」；尾部没数据的缩盘今天静默挂上，之后分配器按新大小建空闲图，层级还可能变，这一类也一并被拦。

推翻它的观测：造一个合法的、健康的池，在某个正常运维动作之后探测整槽数小于 mkfs 时记的值（例如块设备层对同一块盘前后报不同容量）。出现了，说明「探测值」不够稳，要先定探测口径，再谈拒绝。

## 五、第 4 格：变大的处置

**选：放行；不写盘、不算发布；这次挂载与以后一律按记录值用盘（有效容量 = 记录值，多出的部分不分配、不进分配记录树层级的算式）；挂载结果带「哪些盘比记录大、各大多少槽」。记新值归 D2（RAID 条带策略） 已定项 3 的显式几何再求值事务，第一版没有这条路。**

这一格改的是条款里的一个词：D12（目标介质） 已定项 4 的「变大 ⇒ 放行并记」里的「记」。理由三条，都是实现里的事实：

1. 记录值同时是分配记录树根层级的输入（第一节第 2 条）。挂载时把更大的值写下去，而树不同时重发布，下一次读按新值现算的层级与盘上那棵旧树对不上，挂不上（`crates/singlefs-core/src/allocation_record_tree.rs:709` 那一判）。记新值只能和重发布树顶、改系统配置一起做成一次原子的事；而系统配置与根是两个落盘点，中间崩溃会落在「新记录 + 旧树」或「旧记录 + 新树」，两种读者今天都读不了。这是几何再求值事务（D2（RAID 条带策略） 已定项 4 的 b 形态：排空、原子发布新几何）要答的问题，不是挂载时顺手写一笔。
2. 只读挂载不写盘。「放行并记」在只读那一支本来就做不到；统一成「不记」，两支一个行为。
3. 变大后若仍按探测值用盘，记录值就不再是「这块盘上分配过的最大槽号的上界」：用了尾部、再缩回记录值，探测值 ≥ 记录值，第 3 格的比对放行，而数据已经落在记录值之外。第 3 格拒绝的完备性靠「分配永远不越过记录值」；所以有效容量取记录值。

崩溃在中间会怎样：没有写，没有中间态。回退：本字段不可变，所有世代的槽里值相同，旧槽被拿回来也是同一个数。

落点：有效容量的两个消费口：恢复侧所有走 `PoolReader::device_size_in_bytes`（`crates/singlefs-core/src/recovery.rs:132`）的判（`AllocationRecordTreeGeometry::of_reader`、`placement_lies_in_the_unit_area_of_its_device`、`every_device_reaches_the_unit_area_start`），与可写挂载建空闲图那一句（`crates/singlefs-core/src/mount.rs:3242`：「DeviceFreeMap::with_unit_area_start(*identity, device.size_in_bytes(), unit_area_start)」）。做法：选完系统配置、比完容量之后，读者外面包一层「容量换成记录值」的套子（`PoolReader` 已有多个套子先例，如 `PoolReaderCountingUnitReadsWithOneCopyFailing`），空闲图改吃记录值乘 `SLOT_BYTES`。真实 I/O 范围检查（`crates/singlefs-core/src/mounted_session.rs:943`）仍看真实设备大小，不换。没有记录的盘取探测值（第 5 格）。

可观测（`.claude/rules/fs-design.md`@五条硬要求 第 4 条）：挂载结果里带 `devices_larger_than_their_recorded_capacity`，形状照 `crates/singlefs-core/src/mounted_read.rs` 里 `devices_without_a_mountable_system_configuration_slot` 那个字段。强制进入：用例把镜像文件扩一截再挂，可写与只读各一次，断言（a）挂上、（b）该盘有效容量仍是记录值、（c）新分配不落在记录值之外、（d）镜像字节与挂载前逐字节相同（不写盘）、（e）扩过层级边界（175 → 176 GiB 同类大小，用小 mkfs 参数缩比例造）的那一档也挂得上。

买到什么：今天一块盘变大跨过根层级边界就挂不上（报层级错）；之后变大一律挂得上、记录值恒是分配上界。

**这一格是唯一建议交用户的岔路**（改了一条已定条款里的词，且今天不跨边界的变大是能用上新空间的，这一格会把它收回）：
- 候选甲（本方案）：有效容量取记录值，变大不记。便宜，不碰树；代价是扩出来的部分在几何再求值事务有之前用不上。
- 候选乙：让分配记录树根层级不再由现算的盘大小决定（节点自述层级、读者从根节点头读层级），变大在下一次发布里记。代价大（读写两侧、checker、GPU 判器的树几何全改），收益是扩盘即用；而系统配置与根仍是两个落盘点，要另证崩溃点顺序。
- 翻面观测：量一次「扩盘即用」在用户场景里值多少（没有外部用户之前量不出），或乙的改动量比估计小得多。我没量过，没有代价数；按 `.claude/rules/three-way-inference.md`@交岔路时写岔路单，派实验时带上它 要先有代价数才交，所以主 agent 若要交用户，先决定要不要建计数模型。

推翻甲的观测：造一份「变大后用探测值、记录值不动」的实现，在 mkfs → 写 → 扩盘 → 再写 → 缩回记录值 → 挂载这条历史上，第 3 格的比对放行且分配记录越过记录值。能造出来，说明我对条件 3 的推理对；造不出来（例如分配器在别处已经拦住），条件 3 这一条理由作废，甲只剩前两条理由。

## 六、第 5 格：记录值读不出、为 0、越界

**选：**

| 情形 | 处置 | 依据 |
|---|---|---|
| 槽自证过、值为 0，或小于 mkfs 要求的下界，或超过 2⁴⁸ | 整池拒绝挂载，新增 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 一个成员；放在 `crates/singlefs-core/src/recovery.rs:1395` 的 `system_configuration_values_this_reader_accepts` 里单元区起点那一判之后 | 与别的池级不收之值同类；`crates/singlefs-core/src/system_configuration.rs:833` 起 `RootRingSlotsPerRegionOutOfRange` 的注释写明「整池拒绝挂载，不换一槽再试」，C663 的定案（`.claude/kb/checks-owed.md:391`）。本字段不可变，换槽读到的还是它 |
| 下界取多少 | 单元区起点槽号加 mkfs 写在单元区里的两个单元的末尾（`crates/singlefs-core/src/make_filesystem.rs:342` 的 `tree_table_genesis_placement_at` 之后一槽）；这是 mkfs 自己要求的最小盘（`UnitAreaBeyondDevice` 那一判） | 记录值小于它的盘 mkfs 不会写出来 |
| 这块盘两槽都没有自证过的（读不出、全 0、坏） | 不比；有效容量取探测值；仍走今天的「没有可择槽」那条路（只读挂得上、列进 `devices_without_a_mountable_system_configuration_slot`；可写要过补写闸，补写时写探测值，见第 1 格改法） | 没有记录就没有比对对象，硬拒会把今天挂得上的降级读也拒掉；补写是安全的，因为那次挂载已经按探测值过了分配记录越界判与树层级判（过不了就没有补写） |
| 同一块盘两个自证槽的本盘整槽数不同 | 整池拒绝（同 `SystemConfigurationsDisagree` 一类，`crates/singlefs-core/src/recovery.rs:1835` 那个函数里已有这个报错） | 不可变字段，写者不会写出两个值；两个都校验过又不同，是写者坏了 |
| 记录值 ≥ 探测值以外的所有情形（等于、变大、变小） | 第 3、4 格 | |

零填充的旧镜像：偏移 489 起是补齐的 0，第一行判它不收，整池拒；这就是「旧镜像挂不上」，不用新开 incompat 位（D15（格式冻结政策） 已定项 4 的登记表写「每开一条布局线追加一位」，`.claude/kb/decisions/15-格式冻结政策.md:83`；这不是新布局线；里程碑 17 之前不留旧版本分支，`.claude/agent-common.md` 那一条）。出路是重建，不迁移。

买到什么：写者写坏、或字段被改成不可能的值时，「用错了」变成「挂不上」；没有记录的盘不因为这一格比今天更难挂。

推翻它的观测：某个合法历史（崩溃点枚举、随机历史）里出现自证过的槽带 0 或越界值。出现了，说明有写者在写字段之前就发布了槽，要先修写者。

## 七、第 6 格：条款怎么改

改前一律是仓里此刻的原文（行号现查），改后是建议的句子。三条被点名的条款各有改动，其中 D2（RAID 条带策略） 已定项 2 的定案句不用改、射程补一句。

### D12（目标介质） 已定项 4

比对表容量那一行，`.claude/kb/decisions/12-目标介质.md:74`。
改前：
`| 设备容量 | 变小 ⇒ **拒绝**；变大 ⇒ 放行并记 | 变小意味着可能有已分配落点掉到设备之外，而分配记录逐落点（D3（空间分配） 已定项 1）判不出「这个落点还在不在盘上」 |`
改后：
`| 设备容量（整槽数：探测字节数 ÷ 16384 取整，比本盘系统配置里的「本盘整槽数」） | 变小 ⇒ **拒绝挂载**（可写与只读都拒）；变大 ⇒ **放行、不写盘**，有效容量仍取记录值、多出的部分不分配，记新值走显式几何再求值事务（D2（RAID 条带策略） 已定项 3 的形态，第一版没有这条路） | 变小意味着可能有已分配落点掉到设备之外，而分配记录逐落点（D3（空间分配） 已定项 1）判不出「这个落点还在不在盘上」；变大不记，因为记录值同时是分配记录树根层级的输入，挂载时改它而树不同时重发布，下一次读现算的层级与盘上那棵树对不上 |`

射程那一句，`.claude/kb/decisions/12-目标介质.md:78`。
改前：`第一版两盘同构、设备描述符表不写，这四条里容量与布局位退化成各盘互比（已定项 5）。`
改后：`第一版不写设备描述符表：布局位比对退化成各盘互比（只有一套布局，已定项 5）；`physical_block_size` 与容量各比本盘系统配置里的记录值（D22（单元原子性怎么合成） 已定项 9 字段表的「mkfs 时的 `physical_block_size`」与「本盘整槽数」），不退化成各盘互比。`

索引表第 4 行（`.claude/kb/decisions/12-目标介质.md:16`）里「探测到的 physical_block_size 不与记录比、两盘容量不互比（C1040…）」那句是进度说明，代码落地时由那次改动改成现值，不在这一次写。

### D12（目标介质） 已定项 5

定案句，`.claude/kb/decisions/12-目标介质.md:89`。
改前：`已定项 4 的容量 / 布局位 / `physical_block_size` 三条比对在第一版退化成各盘互比；`physical_block_size` 那一条不再只是互比——系统配置几何段加了「mkfs 时的 `physical_block_size`」4 字节（D22（单元原子性怎么合成） 已定项 9，第一版 512），条款要挂载时探到的值与它比对、第一版就有第二个独立输入；代码还没做这一比（C1040（挂载不比对探测到的物理块大小与两盘容量））。`
改后：`已定项 4 的布局位比对在第一版退化成各盘互比；`physical_block_size` 与容量两条不是互比：各有一个系统配置里的记录值——几何段「mkfs 时的 `physical_block_size`」4 字节与字段表末尾「本盘整槽数」8 字节（D22（单元原子性怎么合成） 已定项 9，第一版 `physical_block_size` 写 512、整槽数各盘写各盘自己的），条款要挂载时探到的值与它们比对、第一版就有第二个独立输入；代码还没做这两比（C1040（挂载不比对探测到的物理块大小与两盘容量））；设备描述符表写出那天，「本盘整槽数」并进表里。`
同一句前面的「设备描述符表：第一条线两盘同构、第一版不写；加第二套布局前必须写」不动。

### D22（单元原子性怎么合成） 已定项 15

定案句，`.claude/kb/decisions/22-单元原子性怎么合成.md:337`。
改前：`并写明 D12（目标介质） 已定项 4 的容量 / 布局位 / physical_block_size 三条比对在第一版退化成各盘互比；`
改后：`并写明 D12（目标介质） 已定项 4 的布局位比对在第一版退化成各盘互比、physical_block_size 与容量两条各比本盘记录值；`
「只定这四格」不变：新字段本身是字段表的事，加在已定项 9（下面「连带」）。

### D2（RAID 条带策略） 已定项 2

定案句 `.claude/kb/decisions/02-RAID条带策略.md:51`「池内各盘不必等大。池成员集合与各盘几何在**挂载时**校验，判不出安全组合就**拒绝挂载**。」不用改：容量是 D12（目标介质） 已定项 4 列的几何量之一，已在「各盘几何在挂载时校验」之内；「各盘不必等大」说的是 mkfs 时各盘大小可以不同，本方案每块盘各记各的，与它一致。
射程（`.claude/kb/decisions/02-RAID条带策略.md:53`）末尾补一句，防止「不必等大」被读成「探测容量不受约束」：
补：`各盘容量逐盘记在各盘自己的系统配置里（D22（单元原子性怎么合成） 已定项 9 字段表「本盘整槽数」）：挂载时探测整槽数小于记录值拒绝挂载，大于记录值照常挂载、按记录值用盘；不等大的池每块盘各记各的（D12（目标介质） 已定项 4）。`
这条补句里「记新值走几何再求值事务」没有写进 D2（RAID 条带策略） 已定项 3：已定项 3 说的是设备集合变化（加盘、换盘、掉盘），容量变大是否也归它，条款里没有写；这是 D2（RAID 条带策略） 已定项 3 射程里还缺的一句，列在「没做什么」，不替主 agent 定。

### 连带（不是被点名的三处，但不改就与新字段自相矛盾）

- D22（单元原子性怎么合成） 已定项 9：`.claude/kb/decisions/22-单元原子性怎么合成.md:209` 的「46 行，合计 **489 字节**……槽内余 3607」→「47 行，合计 **497 字节**……槽内余 3599」；`.claude/kb/decisions/22-单元原子性怎么合成.md:228` 的「唯一的盘级字段是「本盘设备号 4」，不是每块盘能配不同的值。」→「盘级字段只有两个：「本盘设备号 4」与字段表末尾的「本盘整槽数 8」（mkfs 逐盘现填），其余都是池级、两盘同内容。」；系统不可变配置那一行 389 → 397（`.claude/kb/decisions/22-单元原子性怎么合成.md:213` 与已定项 26 表 `.claude/kb/decisions/22-单元原子性怎么合成.md:246`）。
- `.claude/kb/layout/01-first-txn.md`：字段表在 F 行（`.claude/kb/layout/01-first-txn.md:156`）之后加一行「不可变 | 本盘整槽数 | 8 | 各盘各写 ⌊字节数 ÷ 16384⌋（4 GiB 的盘 262144） | D12（目标介质） 已定项 4；D22（单元原子性怎么合成） 已定项 9」；合计行（`.claude/kb/layout/01-first-txn.md:158`）的 `format-const` 标记 489 → 497；偏移段（`.claude/kb/layout/01-first-txn.md:160`）补「本盘整槽数 489，合计 497」与 checker 读的偏移名单加 489。D15（格式冻结政策） 已定项 7 的归层：系统配置在第 1 层（尚未冻结，里程碑 17 之前不存在格式锁定），这一改动落在第 1 层，`spec_hash` 随字段表重算。
- 代码注释里同一句话的三处：`crates/singlefs-core/src/system_configuration.rs:301`（「字段表里唯一的盘级字段」）、`crates/singlefs-core/src/system_configuration.rs:247`、`crates/singlefs-core/src/system_configuration.rs:833` 起的「字段表里只有本盘设备号是盘级的」。
- 含 489 的断言：`crates/singlefs-core/src/system_configuration.rs:915`、`:1013`、`:1165` 与 `crates/singlefs-verification/tests/system_configuration_mutability_classes.rs:115`、`:147`（`grep -rn '\b489\b' crates --include=*.rs` 的命中，我挑的是与字段表总长有关的几处）。F 区间 `481..489` 的断言不动。
- 登记表：不新开 incompat 位（第 5 格）；`.claude/kb/checks-owed.md:642` 的 C1040 在代码落地后改写；C218（`.claude/kb/checks-owed.md:170`）与 C180（`.claude/kb/checks-owed.md:138`）不因本方案关闭：本字段不是设备内禀身份，也没有回答设备表单元取哪个码。

## 八、第 7 格：checker 与 GPU 判器

**核心实现（同第 3 到 5 格）判什么，池级 checker 与两份 GPU 判器就各自再判一遍；三者不共用代码。**

池级 checker（`crates/singlefs-checker`，独立解槽）：
1. 改格式同步更新不变量与 checker：`.claude/rules/format-evolution.md`@硬约束。
2. I-7.13（系统配置池级字段在读者收的范围里，`crates/singlefs-checker/invariants/02-根环.md:20`）加一项：本盘整槽数 ≥ 下界（checker 自己用「环长现算的单元区起点 + mkfs 写在单元区里的两个单元」的算式）且 ≤ 2⁴⁸；越界与 core 同类，整池不作保、其余不变量报「不适用」。偏移 489 加进 `crates/singlefs-checker/src/lib.rs:410` 附近那组 `SYSTEM_CONFIGURATION_*_OFFSET` 常量；其余偏移不动。
3. **新立一条不变量**（编号取现有最大 I-7.19 之后的 I-7.20，`grep -rn 'I-7\.20'` 在 `crates/singlefs-checker/invariants/*.md` 与 `.claude/kb/*.md` 零命中）：「系统配置记的本盘整槽数不超过盘的实际整槽数」——每块盘每个自证过、fsid 同池的槽，记录值 ≤ ⌊这块盘实际字节数 ÷ 16384⌋。立它的理由：不立，core 拒挂的镜像（盘比记录小）池级 checker 判绿，两边对同一镜像分类不一，这正是 I-7.13 当初立项的缘由（那一行写「与实现整池拒绝挂载一致」）。红时与 I-7.13 同口径：该池其余不变量报「不适用」。形态照 I-7.14（`crates/singlefs-checker/invariants/02-根环.md:21`）：登记名常量、列进 `IMPLEMENTED_INVARIANTS`、坏镜像进 `crates/singlefs-verification/tests/checker_known_bad_images.rs`（盘比记录小一槽只红 I-7.20）、变异表加行。
4. 容量读数改取记录值：`crates/singlefs-checker/src/walk.rs` 里按 `device_bytes(...) / SLOT_BYTES` 算单元区末尾的 7 处（5274、5734、5851、7156、11592、20722、22288 行，我读了每一处的那一行）。它们不改，则「盘变大、记录值不变」的镜像会在 I-3.20（`crates/singlefs-checker/invariants/05-空间记账.md:27` 的容量等式）这类判上误红。做法：`ImageReader::device_bytes`（`crates/singlefs-checker/src/image.rs:22`）保留交真实字节数，I-7.20 用它；另给一个由 checker 自己从槽里解出的「使用中的整槽数」，walk 里这 7 处改读它；没有记录的盘取真实字节数。

GPU 判器（`.claude/rules/verification.md:25` 要求判器调用点合规；对拍照 `.claude/rules/verification-reference.md:153`）：**要同判**，两份都动：
- 恢复判器 `crates/singlefs-verification/src/crash_judge_common.wgsl`：解槽（1062 行附近 `c.floor = rd64(base, 481u);` 那段）读偏移 489 的 8 字节进配置结构；`allocation_root_level`（2630 行）与分配记录越界判（3896 行 `let device_slots = shr64(device_bytes(), SLOT_SHIFT);`）改用记录值；加三个失败点：记录值不收、盘小于记录值、两槽记录值不同，与 core 的拒绝成员一一对应。
- checker 判器 `crates/singlefs-verification/src/crash_judge_checker.wgsl`：I-7.13 加一项、I-7.20 一格、容量读数同上。
- 要说明白的一点：崩溃枚举里每个镜像的盘大小恒等于 mkfs 记的值，这些新分支在枚举里永远不会触发，GPU 与 CPU 的对拍不会因它们分叉。所以不同判也不会被现有对拍抓到，必须同判的理由是「每条分支能被测试强制进入」（`.claude/rules/fs-design.md`@五条硬要求 第 2 条）：补对拍用例，手工造的镜像（记录值为 0、越界、比盘大、比盘小、两槽不同）每个喂 CPU `recover` 与两份 GPU 判器，分类必须一致。
- 现状的局限：GPU 判器里 `device_bytes()` 是全池一个数（`crates/singlefs-verification/src/crash_judge_common.wgsl:795` 取头里两个字），表示不了不等大的盘；本方案不扩它，不等大的池的对拍只在 CPU 一侧覆盖（见「没做什么」）。

买到什么：把「core 拒、checker 不知道」与「CPU 判器拒、GPU 判器不知道」两类分叉在新字段上提前堵住。

推翻它的观测：对拍用例里任一手工镜像，CPU 与 GPU 分类不一致；或者池级 checker 在一份合法历史（随机历史、崩溃枚举）上判 I-7.20 红 / I-7.13 新项红。前者说明同判没做对，后者说明写者在写字段上有错。

## 九、第 8 格：`unequal_devices.rs:183` 在新判法下是绿是红

**绿。** `crates/singlefs-verification/tests/unequal_devices.rs:183`：「mount_writable(&parameters(), &mut remounted_devices).expect("盘不等大的可写挂载今天接受")」。

推（读代码，没跑）：
- 两盘大小是 4 GiB 与 3 GiB 加 33 槽（`crates/singlefs-verification/tests/unequal_devices.rs:48` 的 `SMALLER_DEVICE_BYTES`）；mkfs 在逐盘循环（`crates/singlefs-core/src/make_filesystem.rs:774`）里各写各的整槽数：262144 与 196641。
- 之后 `acquire_instance`、`warm_up`、`publish_first_file` 经 `write_system_configuration_slot` 重写槽，字段从盘上现读的自证槽带下去（第 2 节改法），两块盘的值不变。
- 用例重开的是同样的镜像文件（`ImageFileOpening::ReopenExisting`），探测整槽数等于记录值，第 3 格不拒、第 4 格不触发，有效容量与今天相同，空闲图与今天逐槽相同，`expect` 通过。
- 这条用例同时是第 1 格「每盘各记各的」的活证据：若 mkfs 写的是池里最小的那一个数（取 min），4 GiB 那块盘就成了「变大」，有效容量被压到 3 GiB 加 33 槽，两盘变成等大，这个文件要钉的「盘不等大」就不再被钉住（推的，没跑）。

推翻它的观测：把本方案按改法实现之后跑 `cargo test -p singlefs-verification --test unequal_devices`，有任一测试红（命令在「要主 agent 跑的命令」）。

会因本方案改字节而要同步的断言（不是回归，是规格变了）：第六节「连带」列的含 489 的几处，与 `crates/singlefs-verification/src/mount_transparency.rs:1594` 起那个整个解构 `SystemImmutableConfiguration`、判「除本盘设备号之外相同」的函数（编译会指出，加「本盘整槽数」也进例外里）。

## 十、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 落点 | 选逐盘记本盘 | 偏移 489 追加「本盘整槽数」8 字节、进系统不可变配置；不写设备描述符表、不每份记全池；双故障那一行（两槽全废加缩盘）判不了，落回补写闸与分配记录越界判 |
| 2 宽度量纲 | 选整槽数、8 字节小端 | 与 `absolute_slot_count_of_device` 同一个量；mkfs 实际用到的就是整槽数；零头不记 |
| 3 变小 | 可写与只读都拒 | 与 `physical_block_size` 同一处置；只读不拒会静默退到旧根；代价是被截小的池不能只读挂载捞数据 |
| 4 变大 | 放行、不写盘、不算发布、有效容量取记录值 | 改的是 D12（目标介质） 已定项 4 里「并记」的「记」：记录值是分配记录树根层级的输入，挂载时记会让树与记录对不上；唯一建议交用户的岔路，交之前缺代价数 |
| 5 读不出 / 0 / 越界 | 0 与越界整池拒；没有自证槽的盘不比、取探测值；两槽值不同整池拒 | 零填充旧镜像自然被拒，不新开 incompat 位 |
| 6 条款 | D12（目标介质） 已定项 4（表一行、射程一句）与已定项 5（定案一句）、D22（单元原子性怎么合成） 已定项 15（定案一句）要改；D2（RAID 条带策略） 已定项 2 定案句不用改、射程补一句；连带 D22（单元原子性怎么合成） 已定项 9 / 已定项 26 与字段表 | 改前改后见第七节 |
| 7 checker 与 GPU 判器 | checker：I-7.13 加一项、新立 I-7.20、容量读数改取记录值；GPU 两份判器同判 | 新分支在崩溃枚举里不触发，靠手工镜像对拍强制进入 |
| 8 `unequal_devices.rs:183` | 绿（推的，没跑） | 每盘各记各的；取池内最小值的方案会让它不再钉住不等大 |

## 十一、要主 agent 跑的命令

没有起过任何项目命令。下面三条是核我这份报告里「读代码推的」那几处的，都是点名测试函数或目标的指定范围命令，我没提交。

1. 在仓根起，核根层级随盘整槽数变化的边界用例今天是绿的（第一节第 2 条的依据之一）。看末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，`exit=0` 且 `task-server/task log <号>` 里该测试 `ok` 算过：
`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试 half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary; 范围=1 个测试函数; 为什么=核分配记录树根层级随盘整槽数变化的边界用例今天绿，支撑变大跨边界今天挂不上的推断' task-server/task submit -- cargo test -p singlefs-core --lib half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary`
2. 在仓根起，核第 8 格的基线：新判法落地之前，这个用例今天是绿的。同样看 `exit=0` 与日志里的 `test result: ok`；若范围声明闸说这个目标会走崩溃枚举，按它的拒绝说明二选一再交：
`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 unequal_devices; 范围=1 个测试目标; 为什么=核盘不等大的可写挂载今天接受这一条基线，第 8 格的推断以它为前提' task-server/task submit -- cargo test -p singlefs-verification --test unequal_devices`
3. 「盘变大跨过根层级边界今天挂不上」这一条（第一节第 2 条的结论、第 4 格理由 1）我只读代码推，没有现成用例能直接坐实它：复核不了，原因是要新写一个用例。用例规格（交实现员，写进副本）：用小环长的 mkfs 参数造两块等大的盘，各 68208 个整槽（812 × 84，`crates/singlefs-core/src/allocation_record_tree.rs:976` 那条用例钉的边界），写一个文件版本；把两块镜像文件各扩一个整槽（16384 字节）；`recover` 与 `mount_read_only` 各一次。现在的代码应报「分配记录树节点的层级不是它的位置规定的那一层」。出了别的结果，第 4 格理由 1 的实证部分作废（理由 3 不依赖它）。

## 十二、没做什么

- 没起任何编译、测试、门禁、脚本，没往任务服务交任务；上面三条判决要引的数与「绿」都没复跑，一律当「推的」。
- 没写模型，没造镜像，没量代价数。第 4 格的岔路没有代价数，交用户之前要先建（`.claude/rules/three-way-inference.md`@交岔路时写岔路单，派实验时带上它）。
- 没改 `crates/`、kb、checker 不变量、`.claude/` 下任何文件；第七节的改前改后只是建议句，没写进仓。
- 没判别别的腿的格：攻方腿分的三件（第 1 格三个候选各自哪里会错、第 4 格崩溃与回退下的漏洞、第 6 格改条款会不会与别的已定项打架）我只给了自己的推翻观测，没替它们攻自己。
- 没读禁读清单里的文件（攻方腿的报告、实现员的副本与报告）。另一个会话正在给 `mount_read_only` 加 `physical_block_size` 探测比对（正文写的），我没看它的改动，第 3 格只写了「放进同一个函数、同一个拒绝枚举」，具体接口以它的实现为准。
- 条款里没有写、我也没替主 agent 定的两处：（一）容量变大后记新值算不算 D2（RAID 条带策略） 已定项 3 的几何再求值事务（已定项 3 只说设备集合变化）；（二）D12（目标介质） 已定项 3 在索引表里标「进度：已实现」（`.claude/kb/decisions/12-目标介质.md:15`），而 `grep -rn '设备描述符表\|device_descriptor' crates/singlefs-core/src | wc -l` 输出 0，这两句是不是同一件事的两面，我没判。
- GPU 判器的盘大小是全池一个数，表示不了不等大的池；本方案没有扩它，不等大池在 GPU 一侧没有覆盖。
- 第 5 格里「两槽值不同整池拒」与「没有自证槽的盘取探测值」两条是我定的小处置，没有条款依据，只有实现里的相邻先例，交攻方腿攻。
