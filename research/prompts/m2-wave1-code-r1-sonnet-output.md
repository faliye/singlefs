# `m2-wave1-code-r1` 正推腿报告（Sonnet，Y5 / Y6）

分工：本报告只判 Y5（代码与条款）、Y6（写回）。Y1/Y2/Y3 归云端攻方，Y4 归本地攻方，未碰、未替攻方找反例。
行号一律现查 `crates/`、`.claude/kb/` 里对应文件自己的行号（`grep -n` / `sed -n 'NR==...'`），不从背景材料或 diff 附录里数。

## 一、Y5：代码与条款

### U2　回落「同样排除 `R`」——一致

D3（空间分配） 已定项 8 ②，`.claude/kb/decisions/03-空间分配.md:255` 逐字：
「聚簇段只给提交内生块……提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`——C146（无空段时的回落政策全仓无定义） 第 ② 条（回落落到哪）由此有了政策……」

C369，`.claude/kb/checks-owed.md:344`：「D3（空间分配） 已定项 8 ② 逐字『提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`』」——与已定项 8 原文逐字相同（不是转述）。

代码：`crates/singlefs-core/src/allocator.rs:436-451` `lowest_commit_generated_fallback_slot` 的文档注释（432 行）原样引用同一句「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」；实现用 `is_blocked_for_commit_generated`（315-318 行：`self.allocated[index] || self.isolated[index] || self.held_until_floor_takes_effect[index]`）逐槽判「不被挡」，第一版 `R` 为空（434 行注释原样写「第一版 `R` 为空」）。

**判定：一致。** 条款原文、C369 的转述、代码注释与实现逐字对得上。
推翻条件：`is_blocked_for_commit_generated` 之外还有一条独立于「已分配/隔离/扣住」的排除逻辑把 `R` 单独摘出来处理（今天没有，`R` 第一版恒空，这一支代码路径事实上没有被真正跑出非平凡值）。

### U3　按设备取「在每一块被选中的设备上各自取」——一致

D3（空间分配） 已定项 8 第 1 条，`.claude/kb/decisions/03-空间分配.md:253` 逐字：「设备集合由 D2（RAID 条带策略） 已定项 8 选定之后，在每一块被选中的设备上各自取该设备内**起点槽号最小、起点 32768 对齐……**的落点……**不选设备、不轮转设备**」。

C368，`.claude/kb/checks-owed.md:343`：「D3（空间分配） 已定项 8 逐字『在每一块被选中的设备上各自取该设备内』，D2（RAID 条带策略） 已定项 2『各盘不必等大』」——与已定项 8 原文逐字相同。

D2（RAID 条带策略） 已定项 2 标题，`.claude/kb/decisions/02-RAID条带策略.md:101`：「已定项 2（2026-08-30）：各盘不必等大」，与 C368 引用一致。

代码：`crates/singlefs-core/src/allocator.rs:8`（文件头注释）「落点按设备取（D3（空间分配） 已定项 8 第 1 条『在每一块被选中的设备上各自取』）：每块盘按自己的空闲图各答一个，各盘一致才分配」；`try_allocate_user_data`（647-678 行）与 `try_allocate_commit_generated`（695-752 行）都是 `self.devices.iter().map(|device| ...)` 逐盘各答一个、`agreement_across_devices`（147-174 行）判一致，三种拒绝（`NoFreeSlotOnAnyDevice`、`SomeDevicesFullDeviceSetSelectionUndefined`、答案不同）都在 `record` / `record_bumped`（真正改状态）之前返回。

**判定：一致。**
推翻条件：找到一条落点分配路径（用户数据或提交内生块）绕过 `agreement_across_devices`、直接拿盘 0 的答案给全部盘用（C368 描述的那种旧毛病）；今天读到的三处分配入口都没有这种写法。

### U4　tail 存「jsn 的 48 位计数器」——一致

D23（journal 的角色与格式） 已定项 18，`.claude/kb/decisions/23-journal的角色与格式.md:1250` 逐字：「在飞记录数上限 = 环槽数 ÷ F……默认环下 196608 ÷ 3 = **65536**，写进超级块；**tail 8 字节存 jsn 的 48 位计数器**，靠超级块自己的两槽轮换取冗余，不另设 tail 槽；记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`。」

C366：代码注释直接点名它（见下），本轮没有单独去 checks-owed.md 复核这条编号的原文（未影响判定，C366 的题面在代码注释里逐字复现）。

代码：`crates/singlefs-core/src/transaction.rs:418-420`（`warm_up_after_journal_counter` 文档注释）：「暖机的两次空发布……jsn 与超级块里的 tail 按记录号接着数、不取 txg（D23（journal 的角色与格式） 已定项 18：tail 存 jsn 的 48 位计数器；已定项 14 第 3 条：计数器全池接着走；C366（暖机路径把 txg 写进计数器与 tail））。」
实现：`publish_without_units`（`transaction.rs:490-544`）里 `record.counter = plan.counter`（jsn）、`CommitStep::RotateSuperblockSlots { journal_tail: plan.counter, ... }`（532-534 行）——tail 写的是 `plan.counter`（jsn 计数器），不是 `plan.txg`；`write_superblock_slot`（`transaction.rs:185-224`）把这个 `journal_tail: u64` 原样放进 `Superblock` 结构体的 `journal_tail` 字段再落盘。

**判定：一致。** tail 字段确实存的是 jsn 计数器（`plan.counter`），不是 checkpoint_txg；条款「48 位」这半句是格式常量层面的位宽（本轮代码里没有单独去比对 `Superblock::to_slot()` 对 `journal_tail` 编码用了几位，`singlefs-format` 不在本轮判的文件清单里，标记「复核不了」——它不属于派发提示列出的被判对象文件表）。
推翻条件：`warm_up_after_journal_counter` 之外还有一条写 tail 的路径（比如 `publish_admitted` 里正常发布也会 `RotateSuperblockSlots`）把 `checkpoint_txg` 传成 `journal_tail`；本轮读到的 `transaction.rs` 里 `RotateSuperblockSlots` 的两处调用点（`publish_without_units` 与 `publish_admitted`）传的都是 jsn 计数器变量，没找到反例。

### U5　出生序号「作用域换到下一个 checkpoint 时清零」——一致

D19（块指针的结构与宽度预算） 已定项 9，`.claude/kb/decisions/19-块指针的结构与宽度预算.md:409` 逐字：「出生序号从 0 起；同一棵树内每写出一个码 2 或码 3 单元加 1（码 2 与码 3 共用一个计数）；**作用域换到下一个 checkpoint 时清零**；同一个 checkpoint 里同一个节点被固定点重写第二次换新号。」

代码：`crates/singlefs-core/src/transaction.rs:947-948`（`BirthSequenceAllocator` 文档注释）：「出生序号：同一 (树, txg, 实例) 里每写出一个码 2 / 码 3 单元加 1，从 0 起（D19（块指针的结构与宽度预算） 已定项 9）。作用域是一次 checkpoint：`publish_admitted` 每次发布建一个……」；实现 `next()`（955-965 行）用 `BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>` 计数、每次取号 `*counter += 1`，不做「同号复用」判断（即同一 checkpoint 内同一节点被固定点重写第二次也会拿到新号，与条款「换新号」一致）；`publish_admitted`（`transaction.rs:1450`）每次调用都 `BirthSequenceAllocator::default()` 现建一个新实例，一次调用只对应一个固定的 `txg`，作用域天然是一次 checkpoint。

**判定：一致。** 用 `(树, txg, 实例)` 三元组做 key 比条款字面的「同一棵树内」更宽，但因为分配器实例本身只活一次发布（一个固定 `txg`、`instance`），同一次调用里这三元组退化成只随「树」变化，语义上与条款等价；「换新号」也逐字实现（不检测重复、恒递增）。
推翻条件：找到一次发布内 `BirthSequenceAllocator` 被跨 txg 复用（同一个实例装两个不同 `txg` 的对象各自取号），今天读到的三处构造点（`transaction.rs:1450, 2046, 2100`）都是每次调用现建，没找到跨 txg 复用的路径。

### U6　记账树行数超过节点容量报错——一致（Y5 判据未给具体引文，按「U2–U7 每条做的是不是条款逐字说的」通判）

里程碑步 3 决策点，`.claude/kb/milestone/02-second-txn.md:162` 逐字：「其它树装不下也要报错不 panic：记账树 477 条（80 块盘）、树表 81 棵（第 82 棵树）、`FirstFile.content` 已判，分裂随树的种类一起做（第二轮攻方腿 Y3）」。

D5（快照 / 空间记账机制） 已定项 4 登记表，`.claude/kb/decisions/05-快照-空间记账机制.md:358-371`（14 行表格，第 1 行「已分配字节……**带**（设备维）」在 358 行，第 4 行「待删除但意图未完成占用……不带」在 361 行，第 6 行「已承诺预留量……不带」在 363 行，第 7 行「扩展点配额已用量……不带」在 364 行，第 12 行「inode 号水位……**带**（树维）」在 369 行，第 13、14 行在 370-371 行）：14 项统计量里带设备维（每盘各一行）的是第 1、2、3、5、10、11 项共 6 项，不带设备维（全池一行）的是第 4、6、7、12、13、14 项；其中第 13、14 项「有过一次推进之后才随每次发布重写……第一个事务这次发布两项一行都不写」（`05-快照-空间记账机制.md:383`，「第 13 / 14 项」那段），第 7 项（扩展点配额已用量）`crates/` 里没有任何实现（`grep -rn '扩展点\|ExtensionPoint' crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/write_accounting.rs` 零命中）。

代码：`crates/singlefs-core/src/transaction.rs:1220` `POOL_WIDE_ACCOUNTING_ENTRIES = 3`（注释「inode 号水位、待删占用、已承诺预留（D5（快照 / 空间记账机制） 已定项 8）」，对应登记表第 12、4、6 项）、`1222` 行 `ACCOUNTING_ENTRIES_PER_DEVICE = 6`（注释「已分配、空闲、不可回收、defer 待释放、碎片段数、全空聚簇段数」，对应登记表第 1、2、3、5、10、11 项）、`1225-1227` 行 `accounting_entry_count(device_count) = 3 + 6 × device_count`；准入检查在 `try_allocate_*` 与 `publish_admitted` 之前（`transaction.rs:1191-1196` 分配记录树同款逻辑、`1204-1209` 记账树）：`accounting_entries_of_this_publish > accounting_node_capacity`（1204 行）时返回 `PublishError::AccountingEntriesExceedOneNode`，早于「拷分配器、动分配器」（1210-1211 行注释与 `allocator_before_this_publish`）。

**判定：一致。** `3 + 6N` 与登记表里「今天真的每次发布都写」的 6 项带设备维 + 3 项池级项完全对应；第 13、14 项按登记表自己的规则第一版不写行，第 7 项全仓未实现，两者都没有被错误地计进 `3` 这个常量、也没有被遗漏一个「本该写却漏计」的项。
推翻条件：`crates/` 某处开始给扩展点配额或清扫水位 / 根销毁代号写行，而 `accounting_entry_count` 没有跟着改成 4 项或更多——今天没有这类写路径。

### U7　层 0 两条新流与 C42 / C77 的「怎么拦」——不是一回事（残留记录）／基本对应但范围窄于条款（陈旧 tail）

**残留记录**：C42，`.claude/kb/checks-owed.md:52`「怎么拦」列逐字：「崩溃点重放里造一次『空洞 → 恢复 → 少量续写 → 再崩』的序列，重放结果里出现**属于已丢弃时间线的记录**即判红；判别力自证：把残留记录改成与新时间线**同源**（反向链一致），该检查必须变绿」——它要测的是一条**属于被丢弃时间线**的记录被错误当合法前缀重放。

代码：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:805-807`（测试自己的文档注释）：「步 0『预想的细节』第三条的**正例**（C42（残留记录冒充合法前缀）、E32（上一条时间线的残留）；验收第 2 条的**正例**那一半）：基镜像里预置一条 jsn 连续、校验和过、点名单元也在、**属于所选根实例**、所选根的实例表里**没有回退行**的记录……⇔ 恢复落在 (1, 5)、读出它那一版的内容」（测试函数名 `residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it` 在第 809 行）；测试断言 `(tally.states, states_whose_chain_reaches_the_residual_record) == (31, 19)`（第 902-906 行，断言消息「跑到的：B 与取号的超级块槽段 15 + 写行发布的记录段 3 + 写行发布的根槽段 1；跑不到的 12 个是实例 2 的某条根已持久」）。这条记录**属于当前（所选根）实例**、没有被丢弃，测的是「该施加就施加」，不是 C42 要的「不该施加的（属于已丢弃时间线的）记录不许被当合法前缀」。

**判定：不是一回事。** C42 的「怎么拦」列要求一次「空洞→恢复→续写→再崩」并验证**丢弃时间线的记录不会被冒认**，代码写的是同一实例内合法记录被正确施加的**正例**；两者的对象不同（前者的记录该来自被抛弃的时间线，后者来自当前时间线）。这与背景材料 U7 自己的措辞「残留记录只做正例」及里程碑收口表第 23 行「残留记录正例 31 个状态里 19 个跑到」是同一件事的两种说法，代码、用例与背景材料对这一点相互一致（自报的缺口是真的缺口，不是被背景材料盖过去的假一致）。
推翻条件：`second_transaction_step_zero_layer0.rs` 里另有一条从「被抛弃时间线」造记录并验证它不会被当合法前缀的用例——现查该文件只有这一条残留记录测试（`fn residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it`，第 809 行），没有第二条。

**陈旧 tail**：C77，`.claude/kb/checks-owed.md:87`「怎么拦」列逐字：「**已还一半**（2026-09-02）：恢复算法已定案……**仍欠**：崩溃点重放造『tail 陈旧 + 窗口内块已复用』的镜像：恢复必须完成且终态与真值逐格相等，中止即红；撕裂注入必须被旗标」。

代码：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:994`（函数 `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state` 起点）：`assert!(states_differing_from_the_true_tail.is_empty(), ...)`（1068-1071 行，终态与「tail 没改坏」的同一状态逐项相等）、`assert_eq!(tally.failed_states, 0, "恢复在每个状态上都完成")`（1089 行）、末尾再注入一次真撕裂并断言 `(torn_report.effective_root, torn_report.journal.verification_failed, torn_report.journal.prefix_applied) == (Some((InstanceGeneration(3), CheckpointTxg(17))), 1, 0)`（1126-1135 行，撕裂的那条被旗标、不施加）。三件事（恢复完成、终态相等、撕裂被旗标且不与陈旧失配混计——1090-1093 行 `verification_failed_states == 0`「陈旧失配不进施加前验证的计数器」）与 C77 的「怎么拦」逐句对应。

但里程碑收口表第 23 行，`.claude/kb/milestone/02-second-txn.md:333`（现查见下）自陈：「陈旧 tail 那格只造得出**跨实例**的一格，同一实例内的 E78（重放的起点） 形态这条脚本造不出」——C77 依据栏点名的 E78（重放的起点） 讲的是同一实例内块复用导致的自我中止，代码测的是跨越到下一个实例之后的复用（测试里 `Script::ReuseOfTheFirstDataUnitSlotAfterE` 覆盖写发生在 txg 18，晚于陈旧 tail 所在窗口对应的实例边界）。

**判定：基本对应，但范围窄于条款自陈要覆盖的形态。** C77「怎么拦」列要的三件事（完成、终态相等、撕裂旗标）代码都做了；但 C77 依据栏的 E78 特指同实例内复用，代码目前只能造跨实例复用，这一点收口表自己已经写明，不是被背景材料掩盖的缺口。
推翻条件：`second_transaction_step_zero_layer0.rs` 里出现一条同一实例内块复用（不跨实例边界）的陈旧 tail 用例；现查该文件里陈旧 tail 相关函数只有 `writes_with_stale_journal_tail`、`records_after_tail_naming_a_mismatched_unit`、`stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state` 三个，没有第二条覆盖同实例形态的用例。

## 二、Y6：写回（收口表与步 6 现状）

逐行核对里程碑 `.claude/kb/milestone/02-second-txn.md`（现查行号，非背景材料行号）与 `crates/`、测试、`crates/mutations.tsv`。

### 第 19 行（`02-second-txn.md:328`）——一致

行文「作用域 2026-09-17 实现员交回：发号器每次发布建一个，一个文件对象的单元抽成 `build_file_version_units`，单文件发布字节不变」。
代码：`BirthSequenceAllocator::default()` 在 `publish_admitted` 每次调用现建（`transaction.rs:1450`，同 U5 节现查），`build_file_version_units` 存在（`transaction.rs:1272`）。「单文件发布字节不变」这半句要靠字节表复跑核，本轮没有复跑权限也没在派发的判据格里（Y3 归攻方），标记**复核不了**（不在 Y5/Y6 分到的格里，也没有可静态核的产物）。

### 第 20 行（`02-second-txn.md:329`）——一致

行文「回落与 bump 共用『已分配、隔离、扣住』三样挡位；三处落点由每块盘各答一个、一致才分配；新用例 `second_transaction_supplement_two_commit_generated_fallback.rs`、`second_transaction_supplement_two_unequal_devices.rs`，变异 9 条」。
代码现查：两个测试文件都存在（`crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs`、`.../second_transaction_supplement_two_unequal_devices.rs`）；`is_blocked_for_commit_generated`（`allocator.rs:315-318`）正是「已分配、隔离、扣住」三个位的析取，`lowest_commit_generated_fallback_slot`（回落）与 `bump_slot_in_open_segment`（bump）都调用它，「共用」成立。
变异计数：`awk -F'\t' 'NR>=77 && NR<=106' crates/mutations.tsv | grep -c 'unequal_devices\|commit_generated_fallback'` → **9**，与行文「变异 9 条」逐字对上。

### 第 20′ 行（`02-second-txn.md:330`）——一致（含一处需要说明的细节）

行文「运行时计数『用户数据的落点与政策函数不一致的次数』（`.claude/kb/verification-build.md` 事务层第一版那张表）按新实现恒 0，口径要改」。
`grep -nF '用户数据的落点与政策函数不一致的次数' .claude/kb/verification-build.md` → 命中 `verification-build.md:76`，与背景材料引用的度量名逐字相同。
`grep -rn 'fallback_policy_mismatches' crates/` → **零命中**：这个计数器在 `crates/` 里今天根本没有实现（不是「实现了、跑出来恒等于 0」，是从未落地成一个真的运行时计数器）。「按新实现恒 0」这句话按字面读像是一次测量结果，但现查下来它是一个设计推论（新实现按盘各答一个、不一致就在动状态前拒绝，所以这类「不一致」不可能表现成运行时的一次不一致计数，而是表现成一次拒绝）——不是恒 0 的观测，是这个计数器的落点已经不存在的推论。
**判定：这句话本身与代码不冲突（`grep` 找不到反例），但「恒 0」写法容易被读成『量出来是 0』，而实际是『这个计数器没有被实现』**。推翻条件：若以后有人给这个计数器补实现并跑出非 0 读数，这句「恒 0」就被推翻；今天它是「未实现」而非「测过是 0」。

### 第 21 行（`02-second-txn.md:331`）——一致

行文「暖机拆成 `warm_up`……与 `warm_up_after_journal_counter`，tail 跟记录号走……用例只在函数层给不相等的起点」。
代码：`warm_up`（`transaction.rs:405-416`）调用 `warm_up_after_journal_counter`（`transaction.rs:426-461`），两个函数都存在；`journal_tail: plan.counter`（`transaction.rs:533`）证实 tail 跟记录号。
用例：`crates/singlefs-harness/tests/second_transaction_supplement_two_warm_up_counter.rs:25`（测试函数起点），`LAST_JOURNAL_COUNTER_BEFORE_WARM_UP = 40`（第 22 行），断言 `txg_and_counter == [(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)]`（第 55-59 行）——jsn 从 41 起接着数、txg 仍 1、2，正是「函数层给不相等的起点」。

### 第 23 行（`02-second-txn.md:333`）——一致

行文「2026-09-17 实现员交回：`second_transaction_step_zero_layer0.rs` 两条新用例、`crash.rs` 按不变量报『评估过 / 判违例 / 不适用』、变异 6 条」。
两条新用例现查存在：`residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it`（`second_transaction_step_zero_layer0.rs:809`）、`stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`（同文件 `:994`）。
`crash.rs` 的 `Layer0Tally` 结构体（`crates/singlefs-harness/src/crash.rs:505`）里 `checker_evaluated_states`（525 行）、`checker_violated_states`（526 行）、`checker_not_applicable_states`（530 行）三个字段（评估过 / 判违例 / 不适用）都在，`checker_counts_by_invariant()`（`crash.rs:536-548`）按这三个字段拼成 `I-x.y=评估过/判违例/不适用` 的字符串。
变异计数：`awk -F'\t' 'NR>=77 && NR<=106' crates/mutations.tsv | grep -c 'second_transaction_step_zero_layer0'` → **6**，与「变异 6 条」逐字对上。
本行「已实现待代码三方」括注里的「残留记录正例……陈旧 tail 那格只造得出跨实例的一格……『先信 tail』……仍没有会红的检查」，与 U7 节的判定一致（一个正例、一个范围窄于条款、一个完全没做）。

### 第 23′ 行（`02-second-txn.md:334`）——一致

行文「残留记录那条流 12 个状态差 65 536 字节」「用例按现状钉成 12 个红」。
代码：`assert_checker_and_record_checker_counts(&tally, &[("I-3.1", 12)], 0)`（`second_transaction_step_zero_layer0.rs:922`），第二个参数把 I-3.1（已分配统计对得上） 的判违例状态数钉成 12，与行文「12 个状态」逐字对上；65 536 字节 = 一个数据单元（32768）+ 三个元数据单元中的一个的量级组合，本轮未逐字节复核这个具体数值的算术（不在 Y5/Y6 分到的算术核对范围，Y4 的本地腿另算三张数表），标记**复核不了**（超出派给我的格）。

### 第 23″ 行（`02-second-txn.md:335`）——一致

行文「陈旧 tail 那条流 8 个状态都误报」「用例按现状钉成 8」。
代码：`assert_checker_and_record_checker_counts(&tally, &[], 8)`（`second_transaction_step_zero_layer0.rs:1097`），第三个参数 `known_record_claimed_state_missing_unit` 钉成 8，函数签名（`second_transaction_step_zero_layer0.rs:341-344`）确认第三个参数就是 `record_claimed_state_missing_unit` 这条判据，与行文「记录核对器第二条判据……8 个状态都误报」逐字对上。

### 第 28 行（`02-second-txn.md:340`）——一致

行文「记账树报错成员 2026-09-17 已实现待代码三方（`AccountingEntriesExceedOneNode`，79 块盘 477 行照常、80 块盘报错且盘上不变）；树表那一半没做：发布路径里树表条目写死 7 条」。
`crates/mutations.tsv` 第 77-106 行两条含 `accounting_node_full`：`eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written`、`seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish`，测试文件 `second_transaction_supplement_two_accounting_node_full.rs` 现查存在。树表条目：`crates/singlefs-core/src/transaction.rs:767` `tree_birth_txg` 里 `.expect("树表第 1 版起恒有七条")`，`grep -n "PublishPlan" crates/singlefs-core/src/transaction.rs` 现查其定义没有树清单字段，与行文「没有任何输入能让它变」一致。

### 第 30 行（`02-second-txn.md:342`）——一致

行文「二进制那一半 2026-09-17 已实现待代码三方（`second-instance` 模式：发布 B 之后冷重开、可写挂载、发布 C）；还要改 55 号的 `MODES` 与按模式取 `recover_cold`，以及 `first_transaction_device_log_check` 加模式参数……」。
`crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 现查：`second-instance` 模式存在（第 526 行 `"second-instance" => RunMode::SecondInstance,`，第 898 行宿主态测试 `second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches`）。
`.claude/gate.d/55-qemu-first-transaction.sh:43`：`MODES=(direct skip-first-transaction-barrier page-cache)`——确实不含 `second-instance`，与行文「还要改」一致（这半是未完成项，行文没有说它已经改了）。
`crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs`：`grep -n 'mode' ...` 零命中，没有模式参数，同样与「还要改」一致。

### 步 6 现状「必红计数」那句——内部自相矛盾，后半句与代码不符

原句，`.claude/kb/milestone/02-second-txn.md:222`（现查，整句很长，只摘与本判据相关的一段，原文一个字不改）：

> 「……必红八条里五条各有先红后绿的用例或变异（摘根槽前的屏障、关掉点名验证并改坏单元、复用窗口置 0、记账重写摘掉、影子账关掉——含层 0 之外那一格的故障注入）；**残留记录与陈旧 tail + 已复用的块 2026-09-17 补进层 0**（残留记录只做正例）；『先信 tail』没有：`crates/` 里只有 `first_transaction_step_six_recovery.rs` 一个 tail 陈旧一格的探针，没有任何改坏形态让它红（2026-09-17 层 0 装置实现员核，此前这句写七条已有、含残留记录与先信 tail），**『陈旧 tail + 已复用的块』那一条还没做**；checker 新接了……」

同一句话里先说「陈旧 tail + 已复用的块……补进层 0」，隔了一个插入语之后又说「『陈旧 tail + 已复用的块』那一条还没做」——**两句字面互相矛盾**。

代码现查：`stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:994`）确实实现并跑通了「陈旧 tail + 已复用的块」这个场景（本报告 U7 节已核：恢复完成、终态相等、撕裂注入被旗标）——**代码支持前半句「补进层 0」，与后半句「还没做」不符**。

**判定：条款自相矛盾，且矛盾的那一半（「还没做」）与代码不符。** 后半句更像是修订前的残留措辞（插入语「此前这句写七条已有、含残留记录与先信 tail」提示这句在 2026-09-17 被改写过，「陈旧 tail + 已复用的块那一条还没做」疑似是改写时没清理干净的旧半句）——但这是我对成因的推测，不是现查到的事实，写回时按「两边并排抄」处理，不替主 agent 下笔改哪一半。
推翻条件：若「还没做」实际指的是收口表第 23 行已经写明的那个更窄的缺口（「只造得出跨实例的一格，同一实例内的 E78 形态这条脚本造不出」），那这句不是矛盾而是省略了限定语——但字面上「那一条还没做」与「已经补进层 0」仍然是两个相反的判断，我判定为矛盾，供主 agent 复核。

「必红八条里五条各有先红后绿的用例或变异」这半句：本轮只在 `crates/mutations.tsv` 里直接按关键词命中「复用窗口置 0」（第 38 行）与「影子账关掉」（`grep -n '影子账' crates/mutations.tsv` 命中第 34、45、48、50、52、53、57、93 行，均属「影子账」类）；「摘根槽前的屏障」「关掉点名验证并改坏单元」「记账重写摘掉」三项按同样的关键词 `grep` 在 `mutations.tsv` 全文零命中，与括注「（任一发布，**已有**）」「（**已有**）」（`02-second-txn.md:228`「必红八条与它们挂在哪次发布上」那句原文）一致——这三项不是靠 `mutations.tsv` 变异表实现的，是更早、独立于变异表的专用测试（这三项不在本轮被判对象文件表内，不属于这一轮改动，只做了关键词存在性核查，**没有**去找它们各自的专用测试文件逐一确认「先红后绿」，标记**复核不了**：不在本轮 30 行 diff 范围内，找它们的成本超出 Y5/Y6 分到的格）。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y5 · U2 回落「同样排除 `R`」 | 一致 | 代码注释逐字引用 D3 已定项 8 ②，`is_blocked_for_commit_generated` 实现与 C369 描述一致，`R` 第一版恒空 |
| Y5 · U3 按设备取「各自取」 | 一致 | `try_allocate_user_data` / `try_allocate_commit_generated` 都逐盘各答一个、`agreement_across_devices` 判一致，三种拒绝都在动状态之前 |
| Y5 · U4 tail 存 jsn 的 48 位计数器 | 一致 | `RotateSuperblockSlots { journal_tail: plan.counter }` 写的是 jsn 而非 txg，与 D23 已定项 18 逐字对上；格式层「48 位」编码本身不在被判文件表内，复核不了 |
| Y5 · U5 出生序号「换到下一个 checkpoint 清零」 | 一致 | `BirthSequenceAllocator` 每次 `publish_admitted` 现建、`next()` 恒递增不检测重复 |
| Y5 · U6 记账树行数报错 | 一致 | `POOL_WIDE_ACCOUNTING_ENTRIES = 3` 与 `ACCOUNTING_ENTRIES_PER_DEVICE = 6` 对应 D5 已定项 4 登记表里今天真的每次发布都写的 3 个池级项 + 6 个设备维项 |
| Y5 · U7 残留记录 vs C42「怎么拦」 | 不是一回事 | C42 要「丢弃时间线的记录被误当合法前缀」的负例，代码只测了「当前实例合法记录该施加就施加」的正例；代码、测试文档注释与背景材料对这一点自报一致，不是被掩盖的缺口 |
| Y5 · U7 陈旧 tail vs C77「怎么拦」 | 基本对应，范围窄于条款 | 恢复完成、终态相等、撕裂旗标三件事都做了；但只造得出「跨实例」复用，C77 依据栏点名的 E78 是「同实例内」形态，收口表第 23 行自己写明这个缺口 |
| Y6 · 收口表第 19 行 | 一致（一处复核不了） | 发号器每次发布建一个、`build_file_version_units` 都核实；「单文件发布字节不变」超出本轮判据范围 |
| Y6 · 收口表第 20 行 | 一致 | 两个新用例文件存在、`is_blocked_for_commit_generated` 三位共用、变异数现查恰为 9 |
| Y6 · 收口表第 20′ 行 | 一致（一处措辞值得注意） | 度量名与 `verification-build.md` 逐字对上；但 `fallback_policy_mismatches` 在 `crates/` 零命中——「恒 0」是「计数器未实现」而非「量出来是 0」 |
| Y6 · 收口表第 21 行 | 一致 | `warm_up` / `warm_up_after_journal_counter` 两个函数都在，专用测试给出不相等起点（40 → 41、42） |
| Y6 · 收口表第 23 行 | 一致 | 两条新用例、`Layer0Tally` 三字段、变异数现查恰为 6，均对上 |
| Y6 · 收口表第 23′ 行 | 一致（65536 字节算术复核不了） | `assert_checker_and_record_checker_counts(&tally, &[("I-3.1", 12)], 0)` 钉死 12，与行文一致 |
| Y6 · 收口表第 23″ 行 | 一致 | 第三个参数钉死 8，函数签名确认对应 `record_claimed_state_missing_unit` |
| Y6 · 收口表第 28 行 | 一致 | 两条新用例（79/80 块盘）、`PublishPlan` 无树清单字段，`tree_birth_txg` 恒七条 |
| Y6 · 收口表第 30 行 | 一致 | `second-instance` 模式已实现，55 号 `MODES` 与 `first_transaction_device_log_check` 的模式参数确认仍未加 |
| Y6 · 步 6 现状「必红计数」句 | 内部自相矛盾，后半句与代码不符 | 同一句先说「陈旧 tail + 已复用的块……补进层 0」，后说「那一条还没做」；代码（`stale_tail_with_a_reused_named_unit_...` 测试）支持前半句 |
| Y6 · 步 6 现状「五条已有」那半 | 一致（部分复核不了） | 「复用窗口置 0」「影子账关掉」在 `mutations.tsv` 全文有命中；「摘根槽前的屏障」「关掉点名验证并改坏单元」「记账重写摘掉」标「已有」、不靠变异表，本轮未逐一找到对应专用测试，复核不了 |

## 四、没做什么

- 不判 Y1（回落与挡位）、Y2（拒绝与失败路径）、Y3（写量计数）——归 Opus 攻方；不判 Y4（计数与准入算术）——归本地攻方；没有替任何一方找反例。
- 没有编译 `crates/`、没有跑 `cargo test`、没有跑任何门禁阶段（`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-forward`），全部判定靠静态读码与 `grep -n` 现查行号；U2–U7 的实现语义只读代码与文档注释推断，没有实测跑出一次结果。
- 「单文件发布字节不变」（收口表第 19 行）、残留记录流「12 个状态差 65536 字节」的具体算术（第 23′ 行）：需要跑字节表或复算，超出静态核对能力，标记「复核不了」，不是「一致」也不是「冲突」。
- 「必红八条」里三项（摘根槽前的屏障、关掉点名验证并改坏单元、记账重写摘掉）标「已有」、不在这一轮 30 行 diff 与 mutations.tsv 新增范围内，没有去找它们各自的专用测试文件逐一确认「先红后绿」是否仍然成立。
- U6 的记账行数公式只核了「今天真的每次发布都写的那几项」，没有去反推「如果以后给扩展点配额或清扫水位补实现，`accounting_entry_count` 要不要跟着改」这类前瞻性问题——那不是本轮 Y5 的判定对象。
- `singlefs-format` 里 `journal_tail` 字段本身的位宽编码（是不是真的只用 48 位、高位怎么处理）不在被判对象文件表内，没有去查。

