# 里程碑三二·1 不变量补齐：实现侧事实表（2026-09-30）

批次：records/2026-09-30-里程碑三开工第一轮.md#1。读的是 HEAD fcc4d340（`git status --short crates | wc -l` 输出 `0`，crates 无未提交改动）。只读，没编译、没跑测试。
行号都是这一次现查的；「推的」是从代码读出、没跑过的推断。

## 零、先要知道的三件事

1. **条数是 49，不是 48。** `grep -c '^| I-.*已实现' .claude/kb/invariants.md` 输出 `49`；`crates/singlefs-checker/src/image.rs:68` 是 `IMPLEMENTED_INVARIANTS: [&str; 49]`；invariants.md 第 12 行也写「判 49 条」（2026-09-28 加 I-7.14）。`.claude/kb/milestone/03-third-txn.md` 第 60 行写的「判 48 条」过时了。
2. **checker 里已经有「由记录施加出来的那一版」的走读，但不是一次完整的只读重放。** `walk.rs:5577` `versions_applied_only_by_records` 只认已被上一次恢复施加过（实例表有行）且根槽读不出的那几版；`walk.rs:3549` `tree_table_pointer_of_the_version_the_next_mount_applies_first` 只取下一次挂载先施加的那一版的树表指针，并且它自己的注释（`walk.rs:3546-3548`）写明恢复的前缀规则只复刻了反向链一道，「断号即止、点名单元验证、发布边界不复刻」。core 的 `replay_journal`（`crates/singlefs-core/src/recovery.rs:3057`）是纯内存的：入参是 `&dyn PoolReader` 与根，交回重建的 `RootRecord`，不写盘。
3. **C395 草稿写于 2026-09-24（提交 67f2c785），它说只在「工作区」的改动今天全在 HEAD**，但草稿里有一批偏移、槽号与原句已经对不上今天的代码与 kb（第八节逐条列）。

## 一、I-4.1（重放后树可遍历）

invariants.md 第 165 行：「任一崩溃点重放后，系统配置指向的树全部可完整遍历」，状态「未实现」。checks-owed.md 里没有一行以它为主语（`grep -c 'I-4\.1（' .claude/kb/checks-owed.md` 输出 `0`）。

| 问 | 事实 | 依据 |
|---|---|---|
| (a) checker 判到哪 | 没有判定函数，也不在 `IMPLEMENTED_INVARIANTS` 里。最接近的是 I-7.2（最新根可完整遍历）：只走盘上根环里 (txg, 实例) 最大的那条根，不先施加 journal | `image.rs:68-118`；`walk.rs:5864-5878`、`walk.rs:6060-6062` |
| (a) 缺哪一判 | 「重放之后那一版」这个对象 checker 自己不算：没有按 D23 已定项 14 五条前缀规则取前缀、换新根段四个字段、再走读的函数 | `walk.rs:3546-3548` 注释原文「只接这一步……断号即止、点名单元验证、发布边界不复刻」 |
| (a) 措辞 | 行里写「系统配置指向的树」；今天根住根环，系统配置不指向树（推的，按 `image.rs:198-214` 系统配置里 checker 读的字段没有树指针） | invariants.md:165 |
| (b) 输入拿不拿得到 | 拿得到：记录的新根段 188 字节原样（`ScannedJournalRecord::new_root_segment`，`walk.rs:4899`）、实例、计数器、txg、提交标记、本次发布内序号、记录标志、反向链（`walk.rs:4893-4914`）；两盘各一份按计数器索引（`walk.rs:4918`）；「恢复眼里的环」取法（`walk.rs:5028`）；反向链那一道前缀判法（`walk.rs:5046`）。缺：点名项的出生树/出生 txg/key 尾段被 `scanned_journal_records_of_device` 丢掉，只留位置条目（`walk.rs:4950-4962`）；「所选根覆盖的最后一条」的锚点规则 checker 没有写 | 同左 |
| (b) 走读入口 | 走一版的函数现成：`walk_version_applied_only_by_records`（`walk.rs:890`）从新根段走树表与映射根，实例表与树表 0 条那一版的分配记录树照它施加在其上那条根取；`walk_tree_table_and_central_mapping_root`（`walk.rs:925`） | 同左 |
| (c) core 写读 | 恢复：`recover`（`recovery.rs:4222`）= 择系统配置 → `choose_root` → `scan_journal`（`recovery.rs:2891`）→ `replay_journal`（`recovery.rs:3057`）→ `walk_to_file_in_the_unit_area_starting_at(effective_root)`（`recovery.rs:3909`，调用在 `recovery.rs:4279`）。可写挂载的读阶段 `read_stage`（`mount.rs:4352`）同样 `scan_journal` + `replay_journal`。core 的走读只走到第一个文件（`extents_of_the_first_file`，`recovery.rs:3347`），不是整棵树 | 同左 |
| (c) 层 0 现状 | 层 0 每个崩溃状态跑 core 的 `recover`（Consult 与 Ignore 各一次）与 oracle，池级 checker 判的是**没重放的**崩溃镜像 | `crates/singlefs-checker-tier/src/crash.rs:1226-1240` |

## 二、I-8.4（重放幂等）

invariants.md 第 87 行：「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」，「未实现」。checks-owed.md 只在 C77 那一行顺带提到它（`grep -c 'I-8\.4（' .claude/kb/checks-owed.md` 输出 `1`，那一行说「I-8.4 管不到这一类」）。来源是 D23 已定项 3 第 2 条硬要求（`decisions/23-journal的角色与格式.md:128-130`）。

| 问 | 事实 | 依据 |
|---|---|---|
| (a) checker 判到哪 | 没有判定函数；清单里没有 | `image.rs:68-118` |
| (a) 缺哪一判 | 「施加两次 = 施加一次」要在同一镜像上各算一次施加结果再比；checker 没有施加函数（同第一节 (e)） | `walk.rs:3546-3548` |
| (b) 输入 | 同第一节：新根段四个字段的原样字节都在 `ScannedJournalRecord` 里 | `walk.rs:4893-4914` |
| (c) core：施加的形状 | 施加一条记录 = 把所选根的四个字段（树表指针、映射根指针、树 ID 水位、F）换成记录新根段里的，实例表指针与分配记录树根照被施加的那条根；`rebuilt` 按值覆写，同一条记录施加两次得到同一个 `RootRecord`（推的，按 `recovery.rs:3210-3226`） | `recovery.rs:3210-3226`；D23 已定项 15（`decisions/23-journal的角色与格式.md:443`） |
| (c) core：重放写不写盘 | `replay_journal` 只读（入参 `&dyn PoolReader`），恢复本身不落盘；盘上状态的变化来自之后的可写挂载第一次发布（写行）。所以「盘上状态相同」今天的对象是「重放 + 写行」那一整段，不是 `replay_journal` 自己（推的） | `recovery.rs:3057-3063`；`mount.rs:4352-4368` |
| (c) 现有用例 | 按函数名搜 `idempot`、`幂等`、`replay…twice` 没有重放幂等的用例（`grep -rln '幂等\|idempot' --include=*.rs crates` 无输出） | 本次 grep |

## (e) 对象 1、2：checker 能不能在一份镜像上做一次只读重放再走读

| 问 | 答 | 依据 |
|---|---|---|
| 有没有现成函数把 journal 前缀施加到内存再走读 | 半有。① `versions_applied_only_by_records`（`walk.rs:5577`）：四条同时成立才算一版——恢复施加过（实例表有行且 txg ≤ Ti、带提交标记、过反向链）、根槽读不出、txg ≥ F、环里有比它老的有效根（`walk.rs:5561-5575`）。它针对的是 I-3.1 并集，不是「下一次重放会得到什么」。② `tree_table_pointer_of_the_version_the_next_mount_applies_first`（`walk.rs:3549`）：只接一步、只交树表指针、只给 I-3.10 用 | 同左 |
| 缺什么 | D23 已定项 14 前缀五条里 checker 只复刻了反向链一道（`record_can_enter_a_replay_prefix`，`walk.rs:5046`）；锚点（带末条标志那条，core `counter_of_the_last_record_the_root_covers`，`recovery.rs:2962`）、断号即止、在飞上限、首条序号为 1、换 txg 断、提交标记、末条之后还有记录、点名单元校验和验证（core `recovery.rs:3109-3197`）都没有 | 同左 |
| 走读那一半 | 现成：`Walk::walk_version_applied_only_by_records`（`walk.rs:890`）按新根段 + 施加在其上那条根的实例表与分配记录树走，判定与候选根同一套 | `walk.rs:882-920` |
| 能不能直接调 core | 不能：checker 与实现只共享 `singlefs-format`（`crates/singlefs-checker/src/lib.rs:1-2` 文件头；D13 已定项 5） | `lib.rs:1-2` |

## 三、I-8.1（环几何够大），欠账 C28

invariants.md 第 84 行：「`环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。最坏占用与 F 写进系统配置，mkfs 时算、mount 时复算比对」，「未实现」，口径注：mkfs 声明的最坏占用取上界还是挂载期内复算没定。C28（checks-owed.md 第 36 行）写的前置里也记着这一格还开着。

| 问 | 事实 | 依据 |
|---|---|---|
| (a) checker 判到哪 | 没有 I-8.1 的判定。最低那一格（环装得下 F 条记录，在飞上限 ≥ 1）在 I-7.13 里判：`journal_ring_bytes_lie_in_the_supported_range`，F 取格式常量，注释原文「不读槽里那 4 字节，那一格没有上下界可判」 | `image.rs:333-352`、`image.rs:274-280` |
| (b) 输入 | 系统配置里有三个字段：在飞记录数上限 4（偏移 345）、journal 最坏占用 8（偏移 349）、安全系数 F 4（偏移 357）——偏移是按 `system_configuration.rs:422-435` 的写序从 317 往后数的（推的），字段名与值在 `.claude/kb/layout/01-first-txn.md` 第 134-136 行。checker 的 `PoolGeometry` 没有这三个字段，`check_system_configuration_slot` 也不读（`grep -rn '\b34[15]\b\|\b349\b\|\b357\b' crates/singlefs-core/src crates/singlefs-checker/src crates/singlefs-format/src` 无输出） | `image.rs:198-214`；`lib.rs:245-289` |
| (b) 环长 | checker 读环长（偏移 333）进 `PoolGeometry.journal_ring_bytes` | `image.rs:258` |
| (c) mkfs 写 | 写在飞上限 = 环长 ÷ 4096 ÷ F、最坏占用 = 在飞上限 × 4096、F = `JOURNAL_SAFETY_FACTOR`（3）。最坏占用是由环长倒算出来的，所以「环 ≥ F × 最坏占用」按构造恒成立（推的） | `system_configuration.rs:428-434`；`crates/singlefs-format/src/lib.rs:221` |
| (c) mkfs 判 | `check_geometry` 拒环短于一条记录、拒在飞上限为 0（「判的是恢复真正用的那个数，不另写一遍『环槽数 < F』」）、拒在飞上限装不进 4 字节 | `make_filesystem.rs:306`、`make_filesystem.rs:321-345` |
| (c) 挂载读 | core 解析系统配置不读 345 / 349 / 357 三个字段（`system_configuration.rs:517-523` 读完 `physical_block_size` 跳到环长）；恢复按环长现算在飞上限（`recovery.rs:781-782` 判它非 0 且装得进 4 字节；`recovery.rs:3084-3085` 取前缀上限）。mount 时「复算比对」这一步今天不存在 | 同左 |
| (c) 一次发布的记录数上限 | 发布路径按 `journal_record_limit_of_one_publish` = min(在飞上限, 2³² − 1) 拒超出的发布 | `transaction.rs:5387-5389`、`transaction.rs:5396` |
| (c) 「任一事务的最坏占用」 | core 里没有一个按事务结构算出的「最坏占用」量；ckpt_cost / c_max 是 D28 已定项 4 的空间块数，单位是 16 KiB 元数据块，不是 journal 记录数（推的，按 `admission.rs:18-21`、`admission.rs:133`） | 同左 |

## 四、I-7.5（根槽按判定宽度对齐），欠账 C47

invariants.md 第 56 行：「每个根槽的起始偏移是挂载时探测的 `physical_block_size` 的整数倍。逐槽判」，「未实现」，射程注：`io_min` 大于 `physical_block_size` 时按 `io_min` 判。C47（第 48 行）说「三样 2026-09-21 现查都齐了，这一条可以开工」，欠「会红的判定本身」。

| 问 | 事实 | 依据 |
|---|---|---|
| (a) checker 判到哪 | 没有判定。I-7.13 只判 `physical_block_size` ∈ [457, 槽距]（`physical_block_size_fits_a_root_slot`），不判对齐 | `image.rs:326-331` |
| (b) 逐槽偏移 | 现成：`root_slot_positions` 交 (区域, 槽, 盘, 偏移)，偏移 = 基址字段 × 16384 + 区域 × P × chunk + 槽 × 槽距，全从系统配置读 | `image.rs:593-609` |
| (b) 判定宽度 | 只有系统配置的声明值（偏移 317，`SystemConfigurationView.declared_physical_block_size`，`lib.rs:281-284`）。`io_min`（偏移 425）不进 `PoolGeometry`（`image.rs:201-214` 没有这个字段，注释 `image.rs:198` 列了它）。`ImageReader` 没有探测口子（`image.rs:19-28`）。`DecisionWidth::Probed` 只在测试里构造（`lib.rs:846-847`，`grep -rn DecisionWidth` 只命中 `lib.rs`）——这就是 C388 | 同左 |
| (c) core 写 | 根槽偏移 = `region_start(r)`（基址与 P、chunk 取编译期常量）+ 槽 × 槽距（`root_ring.rs:94-106`）；区域起点 1 / 4 / 7 MiB。槽距 = 4096 向上取整到 io_min 的整数倍（`system_configuration.rs:156-162`）。根槽宽 = 参数里的 `physical_block_size`（`make_filesystem.rs:555`；C47 行里的「第 278 行」今天在第 555 行） | 同左 |
| (c) mkfs / 恢复的对齐判定 | 没有：`grep -n 'is_multiple_of\|% ' make_filesystem.rs recovery.rs` 无输出。mkfs 只拒 `physical_block_size` < 457 与 > 槽距（`make_filesystem.rs:313`、`make_filesystem.rs:378`）。推的：io_min = 3072、pbs = 4096 时槽距 6144，槽 1 偏移不是 4096 的整数倍，mkfs 不拒 | 同左 |
| (c) 写路径兜底 | 块设备每次读写判「偏移与长度是物理块整数倍」，不齐报 `Unaligned`（`block_device.rs:219-233`）——这是运行时拒写，不是盘上可判的不变量 | 同左 |
| (c) 探测 | `probe_physical_block_size()` 在 core 非测试代码里没有调用点（唯一命中 `block_device.rs:795` 在 `#[cfg(test)]` 模块里，模块起于第 554 行）；mkfs 与挂载用调用方传进来的参数，挂载比对盘上与参数的 pbs / io_min 相等（`mount.rs:3532-3566`） | 同左 |

## 五、I-8.5（tail 槽有效集非空），欠账 C367（前置 C245 已还）

invariants.md 第 88 行：「全部 tail 槽位中自证校验和通过的记录集合非空，且其中除代号最大者外至少还有一条更早的」，「未实现」。C367（第 305 行）：后半句在合法镜像上不成立，改写后判「每块盘两个系统配置槽里自证过的集合非空」。C245 在已还清表（第 612 行），同日 D23 已定项 3 射程句已改。

| 问 | 事实 | 依据 |
|---|---|---|
| tail 住哪 | D23 已定项 3 标题「tail 住系统配置槽，改 jbd2 两处」：≥ 2 槽轮换 + 代号、随每次发布写一次、是发布序列最后一步 | `decisions/23-journal的角色与格式.md:116-124` |
| (a) checker 判到哪 | 没有 I-8.5。每盘两槽已经读、每槽判 I-7.13（`judge_system_configuration_values_the_reader_accepts`，`image.rs:510`）与 I-7.14（`image.rs:549`）。**一块盘两槽都无效时 checker 不早退、也不报违例**；`walk.rs:5737-5741` 注释原文「『哪块盘的系统配置全废了』今天没有编号报得出来，仍欠在 C461」 | 同左 |
| (b) 输入 | 现成：`system_configuration_slot_readings`（`image.rs:456`）交每盘 `[槽 0, 槽 1]` 两格，每格是 `Usable` / `CarriesAValueThisReaderRefuses` / `NotChoosable`（`image.rs:372-386`），带槽号位置；`verified_system_configuration_slots`（`image.rs:487`）交每盘可择槽的 Vec，丢了槽号。tail 字段已解（`SystemConfigurationView.journal_tail`，偏移 469，`lib.rs:200`、`lib.rs:285`），世代号已解（偏移 147，`lib.rs:280`）；walk.rs 不读 `journal_tail`（`grep -n journal_tail walk.rs` 无输出） | 同左 |
| (c) mkfs 写 | 两槽同内容、世代号 `SYSTEM_CONFIGURATION_GENERATION_AT_MKFS` = 1、tail 0 | `make_filesystem.rs:46`、`make_filesystem.rs:582-583` |
| (c) 发布写 | 每次发布写一槽，tail = 这次发布最后一条记录的计数器（`transaction.rs:1877`、`transaction.rs:6805`；取号写 `transaction.rs:1329`） | 同左 |
| (c) 读 | 恢复择系统配置：一块盘两槽都无效时跳过这块盘，全池都无效才失败（`recovery.rs:952-958` 文档，函数 `recovery.rs:964`）。可写挂载读最新自证槽的 tail 当见证（`mount.rs:3262-3266`） | 同左 |
| 张力（推的） | C367 提的改写「每块盘非空」与 core 的读法不一致：一块盘两槽全坏的镜像 core 照样择得出系统配置，按改写判会是违例。它正是 C461 仍欠的那一格（哪块盘全废没有编号），两件可以一起定 | `recovery.rs:954-955`；checks-owed.md C461 行 |

## 六、树 ID 水位（C147；对照 I-9.6）

C147（checks-owed.md 第 133 行）标题「树 ID 水位的不变量还没有」，而同一行写着 2026-09-06 已把不变量写成 **I-7.8（根记录树 ID 水位不低于全池最大树 ID）**、2026-09-14 已实现；它剩下的只有判别力第 ② 格——「根轮出环」钉住累计高水位的坏镜像还没有。I-9.6（第 280 行）的写法是「每个可写头的 inode 号水位 > 该树 inode 树内最大 key，> 全部已发布墓碑的对象 ID」，状态列注明只判了前一半。

| 问 | 事实 | 依据 |
|---|---|---|
| (a) checker 判到哪 | I-7.8 已实现：根环全部自证过的根的水位取 max，> 「出现过的最大树 ID」= 扫描方向码 2 头里的树 ID（排除两类孤儿）∪ 走过的树表条目的树 ID | `walk.rs:6342-6365`；`scanned_tree_identifiers` `walk.rs:2396`；孤儿排除 `walk.rs:2377`、`walk.rs:2419-2425`；树表条目收集 `walk.rs:1303` |
| (a) 与 I-9.6 的对照 | I-9.6 的两处是「树内最大 key」与「墓碑」；I-7.8 的两处是「单元头」与「树表条目」。I-7.8 只取根环的水位，**不取 journal 记录新根段里的水位**（偏移 99 + 172，`lib.rs:833` 已解成 `new_tree_identifier_watermark`，walk.rs 不读：`grep -n new_tree_identifier_watermark walk.rs` 无输出） | 同左 |
| (a) 缺哪一判 | C147 行说缺「连发 R + 1 个 checkpoint 各建一棵树」的坏镜像。按函数名搜 `watermark`，harness 与 checker-tier 测试里的七个函数都不是这个形态（`checker_known_bad_images.rs:5004`、`:5029`、`:5082`、`crash_points_tree_identifier_watermark_orphans.rs:312` 等）；这一条只按名字搜过，没读用例体 | 本次 grep |
| (b) 输入 | 根记录水位（偏移 122，`lib.rs:347`）；码 2 头树 ID（偏移 42，`walk.rs:2427`）；树表条目树 ID（条目偏移 0）；记录新根段水位已解未用 | 同左 |
| (c) core 读 | 挂载取水位 = max(根环全部自证过的根, 环里全部自证过的记录新根段) ——`highest_tree_identifier_watermark_in_the_ring`（`recovery.rs:1203`，文档 `recovery.rs:1195-1201`），C342 的载体；可写挂载再与所选那一版取 max（`mount.rs:1133-1147`）；管理员回退取 `.fold(current.root.tree_identifier_watermark, u64::max)`（`mount.rs:5193-5196`，C586 说这一半没有用例） | 同左 |
| (c) core 判 | 冷走读判树表条目的树 ID < 所选根水位，报 `InvariantViolated { invariant: "I-7.8" }` | `recovery.rs:3535-3547` |
| (c) core 写 | 发布把水位写进根记录与记录新根段（记录新根段 `transaction.rs:1299`、`:1821`、`:6769`；根记录 `transaction.rs:1312`、`:1841`、`:6783`） | 同左 |
| 与 checker 的差（推的） | core 取 max 时并上 journal 记录，checker 的 I-7.8 只取根：根槽读不出而记录在的镜像上，checker 的水位可能低于 core 实际用的水位；方向是 checker 更严（多红），不会漏红 | 两处算式对比 |

## 七、指针里的 key 各段与被指单元头相等（C294）

C294（checks-owed.md 第 252 行）：每条指针带被指单元映射 key 的各段（码 1：出生树、出生 txg、写序；码 2 / 码 3：出生树、出生 txg、实例代号、出生序号），invariants.md 没有一条要求与单元头相等；判法「按指针 key 经映射取到的单元，头里对应各段逐一等于 key」。

| 问 | 事实 | 依据 |
|---|---|---|
| (a) 出生 txg、实例、尾段 | **已判**，挂在 I-1.2 遍历方向那一半：`judge_birth_identity_of_a_referenced_unit` 比头里的诞生代号、写序实例、尾段（码 1 事务号 / 码 2、3 出生序号）与引用它的那条指针 | `walk.rs:482-525`；调用点 `walk.rs:825`（码 2）、`walk.rs:1591`（inode 叶）、`walk.rs:1682`（inode 内部）、`walk.rs:1979`（数据单元） |
| (a) 出生树（指针偏移 34） | **大半没判**。`PointerView.birth_tree` 在 walk.rs 里只用了两处：inode 内部条目的叶容器孩子（I-9.2，`walk.rs:1510-1512`）与中央映射树根指针的出生树当作 I-1.3 的期望树（`walk.rs:958`）。树表条目的根指针、码 2 子指针、extent 数据指针、实例表指针的出生树都不与头比。I-1.3 比的是头里的树 ID 与「引用它的树」（树表条目里的树 ID，`walk.rs:832`、`walk.rs:1588`、`walk.rs:1981`），不是指针偏移 34 | `grep -n birth_tree` 在 checker 四个源文件里的全部命中：`lib.rs:643,682,712,723,816`、`image.rs:645,671,685`、`walk.rs:390,958,1076,1077,1080,1083,1086,1512,1522,1524,1527,2921` |
| (a) 码 2 的出生树 | 头里取哪个字段没定（C289，checks-owed.md 第 247 行）；映射那一边照样不比（`BirthTreeInTheHeader::FieldStillOpenUnderC289`，`walk.rs:384-386`、`walk.rs:1091`） | 同左 |
| (a) 实例表指针 | 根记录偏移 170 那条指针指着的实例表单元不调 `judge_birth_identity_of_a_referenced_unit`（`walk.rs:480-481` 注释：I-1.2 的例外）。C395 草稿主表第 10 行提的正是这一处 | 同左 |
| (a) 「经映射取到」 | checker 从不按指针的 key 查映射：它按位置条目读单元（`read_referenced_unit`，`image.rs:734`），映射条目单独逐条走、判映射 key 与它指的单元头（I-1.11，`walk.rs:987-1094`）。`NamedEntryView::mapping_key`（`lib.rs:720`）在 walk.rs 里没有调用点 | `grep -n 'mapping_key()' walk.rs lib.rs` 只命中 `lib.rs:720` |
| (b) 输入 | 指针各段已解：`parse_node_pointer` / `parse_data_pointer`（出生树 34、出生 txg 42、位置条目 50 / 64、实例 78、尾段 82，`image.rs:669-693`）；头里码 1 / 码 3 出生树在偏移 43（`walk.rs:393-396`）；映射条目 key 各段偏移（`walk.rs:371-378`） | 同左 |
| (c) core 读 | 位置提示读不出时按指针造映射 key 查映射：`mapping_key_for_node`（出生树取 `pointer.head.birth_tree`，`records.rs:202-212`）、`mapping_key_for_data`（`records.rs:190-199`），调用 `recovery.rs:416`、`recovery.rs:468`。C294 说的「出生树错成另一棵树、映射查到别人的单元」就在这条回退路上 | 同左 |
| (c) core 判 | 冷走读：树根判出生序号与指针（`recovery.rs:3314-3319`），不判出生树；inode 叶判 `child.head.birth_tree != identity.birth_tree`（`recovery.rs:3709`，比的是指针与内部条目身份，不是指针与头）；数据单元判诞生代号、写序、fsid 与指针（`recovery.rs:3790-3799`），出生树不比（头里五元组的树与 extent 树比，`recovery.rs:3775-3781`） | 同左 |

## 八、代字段（C395），底稿 `research/prompts/c395-invariants-draft.md`

C395（checks-owed.md 第 324 行）仍在未还表里，正文没随 HEAD 更新：还写着「I-1.2、I-1.8、I-3.5、I-3.6 在册而没实现」「已分配记录的分配代」。草稿入库提交 67f2c785（2026-09-24）；它的「HEAD」是 3b60f09（2026-09-22）。

### (a)(b)(c) 今天的总账

| 字段 | checker 今天判不判 | 依据 |
|---|---|---|
| 根记录的 F（偏移 130） | 判：I-7.9 抬 F 的那一条根对它的上限（`walk.rs:4024`）；I-7.12 系统配置 F ≥ 同盘根上的 F（`walk.rs:4118`） | `image.rs:99-100` |
| 记录新根段的 F / 水位 / 两条指针 | 不判：`new_rollback_floor`、`new_tree_identifier_watermark` 在 walk.rs 无命中；新根段两条指针只在 `versions_applied_only_by_records` 与 `tree_table_pointer_of_the_version_the_next_mount_applies_first` 里当走读入口 | `lib.rs:833-834`；`walk.rs:5551-5556`、`walk.rs:3578` |
| 单元头诞生代号 | 上界与「与指针相同」判（I-1.2 遍历方向、I-4.2）；下界（I-3.5 ②）不判，I-3.5 仍「未实现」 | `walk.rs:482-525` |
| 点名项的出生 txg / 出生树 / key 尾段 | 不判：`scanned_journal_records_of_device` 只留点名项的位置条目 | `walk.rs:4950-4962` |
| 树表条目 `previous_snapshot_txg`（条目偏移 100） | 不判（只在 `walk.rs:2600` 注释里出现） | `grep -n previous_snapshot walk.rs` |
| 分配记录的分配代（未释放） | 判：I-3.10 | `walk.rs:3634`；`image.rs:85` |
| 记账代（记账 key 偏移 14） | 不判：记账行只取 (偏移 0 统计量, 偏移 10 设备) → 偏移 22 值 | `walk.rs:1344-1347` |
| inode 改动计数（记录偏移 88） | 不判：叶容器逐条只读偏移 0、8、40、48、108..140 | `walk.rs:1601-1621` |
| 实例表行 T | 当谓词输入用（候选集、已发布谓词），不判它自己取什么值；最新根那张表才解行（`walk.rs:1864-1867`）；任一根的表另有只读不判的读法 `instance_table_rows_of_root_without_judging`（`walk.rs:3762`）与公开的 `InstanceTableOfRootRecord::read`（`walk.rs:3834`），2026-09-25 提交 346f5e6c 加的 | 同左 |
| 父指针缓存的出生身份 | 判（I-1.2 遍历方向），实例表指针除外（`walk.rs:480-481`） | 同左 |
| 系统配置槽世代号（偏移 147） | 只用来择槽（`image.rs:571-589`），不判取值 | `lib.rs:280` |
| 根记录自身 txg 与槽位（区域 = txg mod R） | 不判：`grep -n 'mod R\|txg mod' crates/singlefs-checker/src/*.rs` 无输出；C88 仍在未还表（第 88 行） | 本次 grep |
| journal 记录自身 txg（记录偏移 26） | 不判取值；I-8.9 按 (实例, txg) 分组、`versions_applied_only_by_records` 按它认版本 | `lib.rs:826`；`walk.rs:5386` |

core 一侧写这些字段的位置草稿没点、这一轮也没逐个现查（见「没做什么」）；已现查的：mkfs 世代号 1（`make_filesystem.rs:46`），零单元发布 `publish_without_units`（`transaction.rs:1233`）。

### (d) 草稿「口径」三条

| 草稿说 | 今天 | 依据 |
|---|---|---|
| 「HEAD（3b60f09）`IMPLEMENTED_INVARIANTS` 29 条，工作区 40 条；I-1.2、I-1.8、I-4.2、I-3.10 的实现与 I-7.9 那一行都只在工作区」 | 过时。HEAD fcc4d340 是 49 条，五条都在清单里 | `image.rs:68`、`:70`、`:75`、`:85`、`:87`、`:99`；crates 无未提交改动 |
| 「回扫报告读 `git show 3cff909:…m2-generation-fields-sweep-report.md`」 | 没重读（不影响本表） | — |
| 「变异在门禁 59 号里红」 | 门禁改按内容起名，今天是 `.claude/gate.d/checker-tier-crates-mutation-replay.sh` | `ls .claude/gate.d` |
| 第 11 行「C395 那一行有一句在工作区已经过时」 | 仍成立，而且今天在 HEAD 里就过时：I-1.2 遍历方向、I-4.2、I-1.8 码 3 已实现，C395 那一行没改 | invariants.md 第 25、31 行状态列 |

### (d) 主表 10 行

| # | 草稿的要点 | 今天是否仍成立 | 依据 |
|---|---|---|---|
| 1 根记录 F | 提新措辞「最新有效根的 F ≤ 抬 F 上限」，挂 I-7.9，说 I-7.9 未实现、「有效」两处写法不一 | **已被取代**。I-7.9 2026-09-24 / 09-26 两支已实现，措辞改成「抬 F 的那一条根……只判抬 F 的那一条，不在后来每张镜像上重算」，「有效」改成「按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F」；草稿引的 QF4（旧 I-7.9 原句）今天不是 invariants.md 的子串；QF1 今天多了「准入抬 F / 卸载抬 F」两支 | invariants.md 第 60 行；`walk.rs:4024`；原句核对见下 |
| 2 记录新根段 F | 同 (实例, txg) 配对的根与计数器最大那条记录，新根段四字段逐字节相等 | 判定仍没有（上表）。**偏移过时**：新根段起点今天是 99（`lib.rs:703`），树表指针 99、映射根 185、水位 271、F 279，草稿写 95 / 181 / 267 / 275；头宽今天 311（`crates/singlefs-format/src/lib.rs:200`），草稿写 307。头校验和 46 仍对（`lib.rs:697`）；`scanned_journal_records_of_device`、`RootView::record_bytes` 仍在 | 同左 |
| 3 单元头诞生代号下界 | 挂 I-3.5 ②；`references_of_root`、`birth_txg_in_a_usable_unit_header` 在；只在最新根有已释放记录时走到叶 | 判定与函数的说法仍对（`walk.rs:2793`、`walk.rs:3523`、`walk.rs:3421-3429`）。**槽号部分过时**：数据单元 50182 仍对（`checker_known_bad_images.rs:3329`）、A 的数据单元 50180 与 extent 叶 50240 仍对（`:100`、`:101`）；草稿说的 B extent 叶 50249、B 分配记录树 50253、A 分配记录树 50245 对不上今天的单元表——分配记录树已是多层，A 写 50245–50249 五个节点（`:45-46` 注释），B 的两片叶是 50257 / 50258（`:95`），50253 今天是 `EMPTY_LEAF_SLOT`（`:2652`） | 同左 |
| 4 点名项诞生代号 | 点名项与被指单元头逐字段相等；「`walk.rs` 一处都没读 `named`」 | 判定仍没有。「一处都没读」**过时**：2026-09-27（45d49aaf）起读点名项的位置条目判 I-2.5（`walk.rs:4950-4962`、`walk.rs:4971`），出生树 / 出生 txg / key 尾段仍不读。「载荷 CRC 偏移 91」**过时**：今天载荷校验和在 95、91 是反向链（`lib.rs:701-702`）；点名项内偏移 37 仍对（`lib.rs:817`） | 同左 |
| 5 `previous_snapshot_txg` | 条目偏移 100 没读；树表槽 50248 | 没读仍对（`walk.rs:2600-2601`，诞生 txg 偏移 108 = 100 + 8）。**槽号过时**：树表今天是 50252（`checker_known_bad_images.rs:107`） | 同左 |
| 6 分配代 | 照 I-3.10，「工作区」已有 | 成立，今天在 HEAD：`walk.rs:3634`，用例 `checker_known_bad_images.rs:3392` | 同左 |
| 7 记账代 | 记账 key 偏移 14 没读；记账树槽 50246 | 没读仍对（`walk.rs:1344-1347`）；`publish_without_units` 在（`transaction.rs:1233`）。**槽号过时**：记账树根今天 50250（`checker_known_bad_images.rs:106`） | 同左 |
| 8 inode 改动计数 | 偏移 88、容器诞生代号 73 没比；说是 `walk_inode_root` 逐条读记录 | 没比仍对；逐条读记录的函数今天是 `walk_inode_leaf_container`（`walk.rs:1568`），`walk_inode_root`（`walk.rs:1405`）只处理根。inode 叶槽 50242 仍对（`checker_known_bad_images.rs:102`） | 同左 |
| 9 实例表行 T | (a) T < 候选根 txg；(b) 同一实例的行在两张表里逐字节相同；依据 QT1–QT4；「别的候选根的表……只在最新根上解行」 | 判定仍没有。「要按偏移 170 各读一次」今天已有只读函数（上表 `walk.rs:3762`、`:3834`）。**依据过时**：QT4「回退写的行」2026-09-26 删了（`decisions/18-块里携带什么信息.md:336`），QT2 改成「只在恢复与实例切换时写」（同文件第 305 行），QW 的 flags bit0 退役（同文件第 299 行）——(b) 里「删行、回退行」那几句要按今天的条款重看。镜像函数 `image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record` 在（`checker_known_bad_images.rs:4213`） | 同左 |
| 10 实例表指针出生身份 | 根记录偏移 170 那条指针与实例表单元头比；`walk_root` 读了、没调 `judge_birth_identity_of_a_referenced_unit` | 成立：`walk.rs:857`、`walk.rs:480-481`；偏移 170 + 42 = 212 与根记录校验和 138（`lib.rs:208`）仍对 | 同左 |

### (d) ②表 3 行（「算」的三项）

| 项 | 今天是否仍成立 | 依据 |
|---|---|---|
| 系统配置槽世代号 | 判定仍没有。「要改一处：`verified_system_configuration_slots` 丢了槽号」**部分过时**：它仍丢槽号（`image.rs:487-506`），但 2026-09-27（c3540c02）加的 `system_configuration_slot_readings`（`image.rs:456`）按 `[槽 0, 槽 1]` 交回，不改也拿得到槽号。偏移 147、校验和 155 仍对（`lib.rs:197`、`lib.rs:280`）；`mutate_system_configuration_slot` 在（`checker_known_bad_images.rs:438`）；首个事务之后槽 1 世代 5、槽 0 世代 4 与字节表一致（`layout/01-first-txn.md` 第 66、81 行） | 同左 |
| 根记录自身 txg（区域 = txg mod R） | 判定仍没有（checker 无 `mod R`）；`valid_roots` 交 (区域, 槽, 根)（`image.rs:613`）仍对；C88 仍未还 | 同左 |
| journal 记录自身 txg | 判定仍没有；`scanned_journal_records_by_device`（`walk.rs:5066`）、偏移 26（`lib.rs:826`）、头校验和 46 仍对 | 同左 |

### (d) 「① I-3.9 那一处空当」

| 草稿说 | 今天 | 依据 |
|---|---|---|
| `judge_release_generations` 里任一候选根 `is_complete` 为假，I-3.9 整条报不适用 | 仍成立 | `walk.rs:3279-3285` |
| 造法第 1 步：B 的分配记录树（槽 50253）16 条已释放记录 4 → 3，`rewrite_released_generations` | 函数在（`checker_known_bad_images.rs:667`）；**槽号过时**（B 的叶是 50257 / 50258，`:95`）；16 这个数没核 | 同左 |
| 造法第 2 步：A 的分配记录树（槽 50245）两盘各翻一字节 | **过时**：50245 今天只是盘 0 那片叶（`:45-46`），每片叶只装它那块盘的记录（`:93-94`），「两盘各翻」要改成翻 50245 与 50246 两片（推的） | 同左 |
| 判据与变异那两段 | 与今天的守卫对得上，可以照用 | `walk.rs:3279-3285` |

### (d) 「另外查到的」5 条

| # | 草稿说 | 今天 | 依据 |
|---|---|---|---|
| 1 | I-7.9 的「有效」按字面近乎恒真，定之前别照字面写 checker | **过时**：I-7.9 已改措辞并实现（只判抬 F 的那条根、有效按那条根自己的实例表且 txg ≥ 抬之前的 F，带卸载记号另一支） | invariants.md 第 60 行；`walk.rs:4024-4118` |
| 2 | mkfs 两槽都写世代号 1 只在 layout 字节表，决策正文没有 | 仍成立（只按 grep 核）：`grep -rn '两槽同内容\|两槽都写世代号\|世代号 1、' .claude/kb/decisions/` 唯一命中 D22 第 210 行，那一处的「两槽同内容」说的是池级内容，不是世代号；D22 已定项 16（第 331 行）仍只说「从 1 起、每写一次 +1」。实现侧常量 `SYSTEM_CONFIGURATION_GENERATION_AT_MKFS = 1`（`make_filesystem.rs:46`）。字节表那一行今天是「489 / 4096 槽」（草稿引的是 481） | 同左 |
| 3 | C294 的代字段那一半已被 I-1.2 罩住，只剩实例表指针与出生树 | 成立，且今天在 HEAD；见第七节 | `walk.rs:482-525`、`walk.rs:480-481` |
| 4 | C85 仍欠的会红检查里「区域 = txg mod R」「记账代 = checkpoint 号」就是②表根记录行与主表第 7 行 | 成立：C85 仍在未还表（第 85 行）、说「仍欠：checker 侧的可判形式并入 C88」；checker 没有这两判 | 本次 grep |
| 5 | I-3.6 第一版没有对象，C395 把它列成诞生代号的在册不变量靠不上 | C395 那一行仍列着 I-3.6，I-3.6 仍「未实现」（invariants.md 第 138 行）；「deadlist 树根指针恒零」这一句这一轮没核 | 同左 |

### (d) 附录原句逐字核对

脚本 `/tmp/claude-1000/m3-r1/invariants-facts/quotecheck.py`：取草稿附录里每条 `Q…` 的代码块，判它是不是点名文件的子串。输出里 `SUBSTRING` 24 条（`grep -c SUBSTRING` 输出 `24`），`NOT-FOUND` 9 条，原样：

```text
QF1 .claude/kb/decisions/16-发布语义.md NOT-FOUND
QF4 .claude/kb/invariants.md NOT-FOUND
QJ1 .claude/kb/decisions/23-journal的角色与格式.md NOT-FOUND
QN1 .claude/kb/decisions/23-journal的角色与格式.md NOT-FOUND
QT2 .claude/kb/decisions/18-块里携带什么信息.md NOT-FOUND
QT3 .claude/kb/decisions/23-journal的角色与格式.md NOT-FOUND
QT4 .claude/kb/decisions/18-块里携带什么信息.md NOT-FOUND
QG2 .claude/kb/layout/01-first-txn.md NOT-FOUND
QW .claude/kb/decisions/18-块里携带什么信息.md NOT-FOUND
```

对不上的是怎么变的（现查今天那一行）：

| 编号 | 变化 | 今天在哪 |
|---|---|---|
| QF1 | 抬 F 上限分成「准入抬 F」「卸载抬 F」两支 | `decisions/16-发布语义.md:36` |
| QF4 | I-7.9 整行重写 | invariants.md 第 60 行 |
| QJ1 | 只多了一个 `<!-- format-const -->` 注释，内容没变 | `decisions/23-journal的角色与格式.md:443` |
| QN1 | 「第一个事务 8 项」改成「新池新建文件 8 项」，多了 format-const 注释 | 同文件第 472 行 |
| QT2 | 「只在恢复、实例切换与回退时写」改成「只在恢复与实例切换时写（管理员回退……不写行）」 | `decisions/18-块里携带什么信息.md:305` |
| QT3 | 「新实例（普通挂载、切换、回退）」去掉了「回退」，「普通挂载与回退本来就全环扫描」去掉了「与回退」 | `decisions/23-journal的角色与格式.md:369` |
| QT4 | 「回退写的行」2026-09-26 删掉 | `decisions/18-块里携带什么信息.md:336` |
| QG2 | 「481 / 4096 槽」改成「489 / 4096 槽」 | `layout/01-first-txn.md:59` |
| QW | flags 那一格：bit0 退役、不回收 | `decisions/18-块里携带什么信息.md:299` |

## 九、推翻条件

- 「checker 没有完整的只读重放」：在 `crates/singlefs-checker/src/` 里找到一个函数按 D23 已定项 14 的锚点、断号即止、发布边界、点名单元验证取前缀，这句就不成立（本轮 grep `replay`、`施加` 只找到第一节列的两处）。
- 「checker 不判指针偏移 34 的出生树（除两处）」：`grep -n birth_tree` 在 checker 源里多出一处与头比较的用法就不成立。
- 「I-8.1 那三个系统配置字段两边都不读」：`grep -rn '\b34[15]\b\|\b349\b\|\b357\b'` 或按字段名在 core / checker 解析里找到读它们的地方就不成立。
- 「mkfs 与恢复不判根槽对齐」：`make_filesystem.rs`、`recovery.rs` 里出现按 `physical_block_size` 或 `io_min` 取模的判定就不成立。

## 没做什么

- 没做派发提示之外的活；没编译、没跑任何测试（含重型测试）；没提交；没改仓内文件。
- 第八节 C395 各字段在 core 里写与读的位置没有逐个现查（只查了世代号 1、`publish_without_units`、F / 水位 / tail 的写处）；草稿坏镜像的「推得只红它」没核；「16 条已释放记录」「deadlist 根指针恒零」没核。
- 回扫报告 `m2-generation-fields-sweep-report.md`（草稿口径第 2 条）没重读。
- 第六节「根轮出环」坏镜像只按测试函数名搜，没读用例体。
- 草稿目录里的辅助文件：`quotecheck.py`、`quotecheck.out`、`walk-fns.txt`、`recovery-fns.txt`，都是文本，没有编译目录或仓副本。
