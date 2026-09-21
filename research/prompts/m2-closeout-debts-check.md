# checks-owed.md 七行今天能否移「已还清」—— 只读核查

范围：C22、C29、C42、C77、C80、C281、C314。判法：把每行「怎么拦（会红的形态）」拆成独立分句，逐句核对 `crates/`（含今天未提交的新用例：`second_transaction_step_zero_layer0.rs` 两条新测试、`crates/mutations.tsv` 95–100 行）里是否有对应的用例 / 变异行 / 门禁阶段，并核实它判的确实是那一句。**只读，未编译、未跑门禁、未跑 cargo test/mutate.sh**——凡涉及“运行结果”的判断，都是读代码推出来的，已在各条注明置信度与需要补跑之处。

---

## C22（刚释放的块立即重分配）

原文「怎么拦」：两条：① 镜像侧由 invariants.md I-4.8（近 K 代根校验和自洽） 判（从最近 K 代任一根遍历，校验和必须全对）；② 代码侧门禁做故障注入自证——把「延迟重用」的窗口临时改成 0，I-4.8 必须变红。不红说明这条约束根本没被检查

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| ① 镜像侧由 I-4.8 判（从最近 K 代任一根遍历，校验和必须全对） | `crates/singlefs-harness/tests/checker_known_bad_images.rs:253-262`：坏镜像把「第 0 代根引用的 mkfs 树表单元头抹成零、不重封」，登记名就是 `"I-4.8"`（255 行）；驱动它的用例是 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`（同文件）。另有 `crates/mutations.tsv:59-61` 三行把 `walk.rs` 里 `judge("I-4.8", …)` / `judge("I-7.4", …)`（`crates/singlefs-checker/src/walk.rs:785,826`）改成恒真或互换，目标测试仍是 `checker_known_bad_images` | 是——坏镜像的登记名与断言消息（`checker_known_bad_images.rs:603-605` 同型的 I-4.8 分支）都直接点名 I-4.8，走的是「候选根遍历时单元读不出」这条路径，与不变量定义一致 | 有 |
| ② 故障注入自证：把延迟重用窗口改 0，I-4.8 必须变红 | `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:483-511` 的 `reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`：不抬 F（`newest_root_floor == 0`，即窗口=0）直接 `reclaim_released_up_to(11, Immediately)` 再覆盖写，断言 `violated.contains(&"I-2.1")`（507-509 行） | **部分**——`walk.rs:806-828` 的实现里，候选根 A（未抛弃、未低于 floor）被走读时，`walked_into_reused_or_erased_unit` 由 `violation_count("I-2.1")` 的增量算出（815 行），同一个布尔值同时喂给 `judge("I-7.4", …)`（822 行）与 `judge("I-4.8", …)`（826 行）——按代码读法，这个测试跑起来时 I-4.8 **必然**也被判红，但测试本身只断言 `"I-2.1"` 在 `violated` 里（508 行），从未点名 `"I-4.8"`。若以后有人把 I-4.8 的计算从这条共享逻辑里拆开、单独出一个 bug，这个测试**不会**发现，因为它没有钉住 I-4.8 | 部分 |

**结论：不能销账。** 差的是②：现有用例的断言没有点名 I-4.8（只断言 I-2.1），虽然按今天的实现代码，I-4.8 与 I-2.1 在这个场景里必然同时变红，但这是靠读代码推出来的，不是断言钉住的——回归会漏判。建议：在该测试里补一行 `assert!(violated.contains(&"I-4.8"))`，或在 `mutations.tsv` 里补一条把 826 行单独改坏、由这个测试抓到的变异行。

## C29（恢复先信 tail 会丢数据）

原文「怎么拦」：E24（恢复算法：先信 tail 会不会丢记录）已把机制做成可执行模型并证实；欠的是把它接进真实恢复路径的门禁阶段

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| E24 已把机制做成可执行模型并证实（模型层，非 crates/） | `.claude/kb/experiments/24-恢复算法先信tail会不会丢记录.md`；代码在 `research/e7-index-bench/src/bin/e24_recovery.rs`，纯计数模型，不在 `crates/` 下 | 是（模型层确实做过），但这一句本身承认它不在真实实现里 | 有（但不算这次核查的范围——它是 research/ 模型，不是 crates/） |
| 欠的是把它接进真实恢复路径的门禁阶段（即：crates/ 里要有故障注入证明「若真实 `scan_journal`/recovery 信了 tail，会丢窗口内记录」） | **没找到**。现查 `crates/singlefs-core/src/recovery.rs:786-815` 的 `scan_journal`：它对每块盘固定扫描 `(0..ring_bytes/JOURNAL_RECORD_BYTES)` 整个环（`796-804` 行），函数体内一次都没有引用 `superblock.journal_tail`；全仓 `grep -n journal_tail crates/singlefs-core/src/*.rs` 命中的 9 处全在 `crates/singlefs-core/src/system_configuration.rs`/`transaction.rs`（写入侧），`recovery.rs` 里 0 处读它。`crates/mutations.tsv` 里唯一带 "tail" 字样的变异是第 97 行，但它模拟的是「从 tail 起逐条验证、失配即中止」（自我中止形态，属于 C77/E78 的失败模式），不是「信了 tail 就往前走、丢记录」这个 C29 特有的失败模式；该行本身也是这次未提交改动的一部分 | 否——真实实现本来就是「不先信 tail，全环扫描」（`recovery.rs:6` 的设计注释也这么写），所以没有、也无法构造出「real code 信了 tail」这条分支去故障注入；这条债务原文自己写的就是「仍欠」，现查坐实：crates/ 里确实没有把 E24 接进真实路径的检查 | 没有 |

**结论：不能销账。** 这行「怎么拦」列本身承认是未完成状态（模型层已证、实现层未接），现查确认到今天为止 `crates/` 里仍然没有对应的检查——与用户给的提示（“C29 据报告没有会红的形态”）一致。

## C42（残留记录冒充合法前缀）

原文「怎么拦」：崩溃点重放里造一次「空洞 → 恢复 → 少量续写 → 再崩」的序列，重放结果里出现属于已丢弃时间线的记录即判红；判别力自证：把残留记录改成与新时间线同源（反向链一致），该检查必须变绿

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| 崩溃点重放造「空洞→恢复→续写→再崩」，重放结果里出现**属于已丢弃时间线**的记录即判红 | `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:805-923` 的 `residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it`：确实构造了「基镜像里种一条残留记录 → 恢复施加 → 续写 C」的序列（672-803 行的 `prepare_with_residual_record_seeded`），并逐崩溃状态判定「链能不能接到残留记录」与「恢复实际是否施加」是否一致（843-901 行） | **否，判的是反面场景**——该用例自己的文档注释（805-807 行）与 `.claude/kb/milestone/02-second-txn.md`（约 73、80 行）都明确写这是**正例**：残留记录属于**所选根自己的实例**、实例表里**没有回退行**，因此“应该被施加”且断言它确实被施加。C42 真正要防的是**反例**——记录属于**已被回退抛弃的时间线**却被误当合法前缀重放。milestone 文档原文（`02-second-txn.md`）写明反例前置是步 4（回退），且截至目前**未见对应的 layer0/崩溃点重放级负例用例**。唯一相关但级别不同的是 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:404-444` 的 `the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water`——它直接调用 `replay_journal` 传入不同 `high_water`，断言前缀在回退行处被正确封顶（438-442 行），但这是**函数级直调**，不是崩溃点枚举，也没有点名「残留记录」这个具体故事 | 部分（只做了正例；负例只有非崩溃点重放级别的函数级用例） |
| 判别力自证：把残留记录改成与新时间线同源（反向链一致），该检查必须变绿 | `crates/mutations.tsv:95-96` 两行都是拿 `residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it` 当靶子，但改法分别是「崩溃镜像的记录提示漏掉基镜像里的记录」与「恢复把没有回退行的所选根也按 W=0 封顶」——两者都是让**该被接受的记录变得不被接受**（绿转红的方向相反），不是「把一条本来因为不同源而被拒的记录改成同源、使检查从红变绿」这个具体形态 | 否——没找到匹配这句判别力自证具体描述的用例或变异 | 没有 |

**结论：不能销账。** 差两句：① 负例（残留记录属于被抛弃时间线、恢复必须拒绝）在崩溃点重放/layer0 层面没有用例，只有一条函数级直调测试覆盖了同一机制的一部分；② 「改成同源、检查变绿」这个具体判别力自证形态没有对应实现。

## C77（重放起点未定义）

原文「怎么拦」：**仍欠**：崩溃点重放造「tail 陈旧 + 窗口内块已复用」的镜像：恢复必须完成且终态与真值逐格相等，中止即红；撕裂注入必须被旗标（另一句「恢复算法已定案」是决策层陈述，非检查分句，不列）

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| 崩溃点重放造「tail 陈旧 + 窗口内块已复用」的镜像 | `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:925-1097` 的 `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`：`prepare(…, Script::ReuseOfTheFirstDataUnitSlotAfterE)` 构造「A 记录点名的单元在 txg 18 被合法复用」的流，`writes_with_stale_journal_tail`（929-956 行）把每次超级块槽写的 tail 重写成 `STALE_JOURNAL_TAIL=2`（926 行），再用 `enumerate_layer0_selecting_versions_observing_each_state` 枚举 txg 18 单元写持久之后的 8 个崩溃状态（1021-1058 行） | 是，场景与原文完全对应 | 有 |
| 恢复必须完成且终态与真值逐格相等，中止即红 | 同用例 1068-1089 行：`assert!(states_differing_from_the_true_tail.is_empty())`（陈旧 tail 下的恢复结果与 tail 未改坏时逐状态相同）、`assert_eq!(tally.failed_states, 0, "恢复在每个状态上都完成")` | 是，断言直接钉住这两点 | 有 |
| 撕裂注入必须被旗标 | 同文件 1099-1141 行：把 txg 18 已持久的数据单元两份都改坏一个字节（`torn_writes`），断言 `torn_report.journal.verification_failed == 1` 且 `torn_report.journal.prefix_applied == 0`、`effective_root == Some((3,17))`（E 那一版），即撕裂记录被旗标、不施加，恢复正确停在 E | 是 | 有 |

**旁证（未跑，仅代码读出的推断）**：`crates/mutations.tsv:97` 把「陈旧失配即自我中止」（E78 的自我中止形态）这段代码直接注入 `recovery.rs::scan_journal` 调用处，目标测试正是本条上面这个 `stale_tail_…` 用例——若真的中止，`tally.failed_states` 会从 0 变正、`states_differing_from_the_true_tail` 也会非空，理论上会被现有断言抓到，但**这条变异行本身是本轮未提交改动，没有实跑 `mutate.sh`/`mutation-triage` 确认过「抓到」**，只是读代码认为它会被抓。

**结论：这一行三句在 `crates/` 里都能找到对应、且断言判的确实是原文那几句，已接入门禁 54 号**（`.claude/gate.d/54-layer0-replay.sh:39-53` 跑 `second_transaction_step_zero_layer0` 全部用例，含 `--include-ignored`，本用例未标 `#[ignore]`，常规 `cargo test` 也会跑）。**建议销账，但有一个前提缺口**：`mutations.tsv:97` 那条判别力自证是本轮未提交、未实跑确认的，若要在提交前写「已还清」，应先跑一次 `mutate.sh` 把这条变异坐实（抓到/无效/没红三选一），不要照抄「有」。若只看 `crates/` 里已经写好且可读断言这一层，本行**可以**移，但需在 kb-scribe 环节备注这条变异待验证。

## C80（记账更新必须原子地随根发布）

原文「怎么拦」：崩溃点重放里每个发布点后跑 I-3.1（已分配统计对得上） 必须绿；故障注入自证——把记账排空从发布路径摘掉，同一检查必须红

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| 崩溃点重放里每个发布点后跑 I-3.1 必须绿 | I-3.1 确实在 layer0 每个崩溃状态都被评估（`second_transaction_step_zero_layer0.rs:372-373` 的 `must_evaluate` 列表含 `"I-3.1"`）。但**它今天不是恒绿**：同文件 918-922 行的注释与 922 行的断言 `assert_checker_and_record_checker_counts(&tally, &[("I-3.1", 12)], 0)` 明确容忍「I-3.1 在 12 个状态上判红」，理由写着**"口径未定（2026-09-17 写这条用例时发现）"**——即今天（写这份报告当天）代码里就存在一个被主动放行、尚未解决的 I-3.1 红：残留记录那一版的四个固定点单元被写行发布释放进 defer，checker「已分配=候选根引用并集」与分配器「defer 里的仍算已分配」在这一情形上口径分歧 | 否——原文要求「必须绿」，现状是「已知有 12 个状态红，且被当作现状放行，不是被检查拦下」，这恰好命中了 C80 描述的失败模式本身（记账与根发布之间不同步），只是它现在被记成"待设计"而非"违反" | 没有 |
| 故障注入自证——把记账排空从发布路径摘掉，同一检查必须红 | 全仓搜索 `crates/singlefs-core/` 未找到任何「write buffer 排空到叶」的独立实现：`grep -rn "排空到叶\|drain\|前端\|WriteBuffer" crates/singlefs-core/src/*.rs` 零命中；`grep -rln accounting crates/singlefs-core/src/*.rs` 只命中 `allocator.rs`（记账增量在分配器里直接维护，`982` 行 `accounting_deltas_are_maintained_incrementally`）、`records.rs`（`AccountingEntry` 结构体）、`recovery.rs`、`transaction.rs`（`accounting_entry_count`），没有一处是独立于事务提交路径之外、可以被"摘掉"的排空步骤；`crates/mutations.tsv` 里也没有任何一行以"排空"/"drain"/记账相关字样匹配这个故障注入描述 | 否——今天代码里记账似乎是与其它结构在同一次事务提交里同步生成的（没有独立的前端 write buffer 排空阶段），因此这条「摘掉排空、检查必须红」的故障注入**没有对象可摘**，也没有对应用例 | 没有 |

**结论：不能销账，而且现状比"仍欠"更值得警惕。** 不仅两句原文分句在 `crates/` 里都找不到对应的会红检查，现查还发现今天（2026-09-17）代码里已经有一个与 C80 主题直接相关、被显式容忍的 I-3.1 红（12 个崩溃状态，`.claude/kb/invariants.md:120` 关于 I-3.1 的说明与用例注释都指向这同一处"口径未定"）。这不是"没有检查"，是"有检查，而且它已经在报警，只是被记成现状而不是违规"——建议连带核实是否要新开一条 checks-owed 记这个具体口径分歧，而不是简单把 C80 移入已还清。

## C281（回退后第一次发布会复用被抛弃时间线还引用的块）

原文「怎么拦」：检查形态：崩溃点重放在回退后第一次发布的每个崩溃点上恢复，I-7.2（最新根可完整遍历） 必须绿；判别力自证：去掉这条约束必须由绿转红（前面「二选一」那句是决策选项，非检查分句，不列）

前置列写的是（截至该行上次更新）：「决策侧已还…检查仍欠：事务层、管理员回退实现、崩溃点重放 harness」——这三样在 `crates/` 里今天都已存在（`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mount.rs::mount_rollback`、`second_transaction_step_zero_layer0.rs` 的 layer0 枚举），说明前置列本身已经过时，值得连带更新。

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| 崩溃点重放在回退后第一次发布的每个崩溃点上恢复，I-7.2 必须绿 | `second_transaction_step_zero_layer0.rs:469-515` 的 `full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`：穷举整条固定脚本（mkfs→取号→暖机→A→B→回退 D→四次覆盖写→抬 F→重用 E）共 2,104,413 个崩溃状态，`assert_checker_and_record_checker_clean`（514 行）要求全部 26 条不变量（含 I-7.2）在每个状态上零违例，且 `must_evaluate` 列表（339-402 行范围内定义于 372-373 行）强制 I-7.2 至少被真评估过一次 | 是——覆盖范围比原文要求的"回退后第一次发布的每个崩溃点"更宽（整条脚本全穷举），I-7.2 被点名要求全程为绿 | 有 |
| 判别力自证：去掉这条约束必须由绿转红 | `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:338-400` 的 `without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit`：用 `ShadowLedger::Off` 强制去掉影子账保护，断言恢复挂上被抛弃根 C 后读到"撕裂"（`assert_ne!` 396 行），`ShadowLedger::On` 时同一构造能正确读回（`assert_eq!` 387 行）；对应 `crates/mutations.tsv:34` 把 `mount.rs` 里 `isolate_abandoned(...)` 调用整行删掉，目标测试即此用例 | **部分**——功能上确实验证了"去掉保护就从能读回（视为绿）变成读不回/撕裂（视为红）"，但断言对象是 `recover(...).outcome`（恢复结果内容），全程没有调用 `check_pool_image`，因此**没有点名 I-7.2**这个不变量本身；原文写的判别力自证钉的是 I-7.2，这里钉的是恢复内容正确性，两者语义相关但不是同一句 | 部分 |

**结论：不能直接销账，但已非常接近。** 第一句已经很扎实地满足（甚至超出范围）；第二句的判别力自证在功能层面成立，但断言对象与原文点名的 I-7.2 不是同一件事，且原文的"前置"描述已经过时（应更新）。另外，C281 的"二选一"决策内容已被 C314 的影子账方案吸收，若销账建议与 C314 一并处理、或在此行注明"决策内容并入 C314，检查随 C314 一并核"。

## C314（回退可以复用被抛弃的根引用的单元）

原文「怎么拦」：崩溃点重放造两格：回退那次发布崩在单元之后、记录之前；回退确认之后复用、再让回退实例的根全读不出。断言恢复挂上的根引用的单元一个都没被复用或抹头。判别力自证：放开对被抛弃的根的保护，两格都必须由绿转红

前置列写的是（截至该行上次更新）：「检查仍欠：崩溃点重放 harness（多次挂载的录制流）」——`second_transaction_step_zero_layer0.rs` 的固定脚本本身就是多次挂载/重开的录制流（进程退出、重开走回退，见该文件 57-156 行的 `prepare` 构造），这条前置描述同样已过时。

| 分句 | 做成会红的东西 | 判的是不是这一句 | 结论 |
|---|---|---|---|
| 格 1：回退那次发布崩在单元之后、记录之前 | **没有专门点名的用例**。`full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`（同上）穷举了整条脚本的全部持久子集，逻辑上必然覆盖"回退发布 D 的单元已持久、记录未持久"这一具体子集作为 2,104,413 个状态之一，但没有任何断言单独把这个状态挑出来命名或验证 | 只是隐含子集，不是被专门断言的场景 | 部分 |
| 格 2：回退确认之后复用、再让回退实例的根全读不出 | `second_transaction_step_four_rollback.rs:338-400`（同 C281 引用的用例）：`rollback_to_first_root` 之后再发两版文件（占用被抛弃槽或安全槽，取决于 `ShadowLedger` 开关），再把 4 个 txg 的根槽全部改坏（375-378 行的 `flip_byte` 循环），强制"回退实例的根全读不出"，断言恢复结果 | 是，场景描述与用例构造一致 | 有 |
| 断言恢复挂上的根引用的单元一个都没被复用或抹头 | `full_enumeration…` 的 `assert_checker_and_record_checker_clean` 要求 I-2.1、I-4.8、I-7.4 全程零违例（这三条正是"单元被复用/抹头"的判定依据，见 `walk.rs:779-828` 的注释） | 是 | 有 |
| 判别力自证：放开保护，两格都必须由绿转红 | 只找到格 2 的判别力自证（`mutations.tsv:34` + `without_the_shadow_ledger…` 用例，见上）。**格 1 没有对应的判别力自证**——没有变异行把"回退发布崩在单元之后、记录之前"这个具体崩溃点从受保护变成不受保护并断言由绿转红 | 只覆盖了两格中的一格 | 部分 |

**结论：不能销账，但比 checks-owed.md 当前记录的状态新得多。** `crates/` 里今天已经有：① 覆盖回退全程的 210 万+ 状态穷举、全部相关不变量断言为绿；② 针对"回退确认后复用+根全读不出"这一格的定向判别力自证（含 mutations.tsv 条目）。差的是：格 1（回退发布自身崩在单元与记录之间）既没有被专门断言点名，也没有专门的判别力自证——它目前只是穷举覆盖里"顺带路过"的一个状态，回归如果只破坏这一个具体子情形（而不影响其它状态），穷举测试未必能把它从背景噪音里分辨出来（穷举断言是"全部为 0"，任何一个状态红都会报错，所以理论上能抓到——但没有专门测试把这一格单独钉住、也没有变异行验证过"抓到"）。建议：① 更新本行"前置"列（harness 已具备）；② 补一条格 1 的定向变异或至少一条命名断言，再销账。

## 汇总

| 编号 | 能否移入已还清 | 一句话 |
|---|---|---|
| C22 | 不能 | ①有；②只钉了 I-2.1，没点名 I-4.8（虽按代码逻辑必然同时红） |
| C29 | 不能 | 原文自认仍欠，现查坐实：真实 `scan_journal` 全环扫描且不读 tail，没有故障注入把它接进真实路径 |
| C42 | 不能 | 只做了正例（残留记录合法、应施加）；负例（残留记录属于被抛弃时间线、应拒绝）在崩溃点重放层没有用例；判别力自证的具体形态未找到 |
| C77 | **接近可以**，但有一处未坐实 | 三句在 crates/ 都有对应且已接入门禁 54 号；唯一缺口是判别力用的 mutations.tsv:97 是本轮未提交、未实跑确认的变异 |
| C80 | 不能，且发现新问题 | 今天代码里就有一个与本行主题直接相关、被容忍的 I-3.1 红（12 个状态，口径未定）；且没有可摘的"排空"实现可做故障注入 |
| C281 | 不能，但接近 | I-7.2 全绿已被穷举覆盖；判别力自证功能上成立但断言对象是恢复内容而非 I-7.2 本身；决策内容已被 C314 吸收 |
| C314 | 不能，但接近 | 两格中格 2 齐全（用例+判别力自证），格 1 只是穷举的隐含子集，没有专门断言或专门判别力自证；本行"前置"描述已过时 |

**没有一行可以在今天、仅凭只读核查就移入「已还清」。** 最接近的是 C77（差一次变异跑实确认），其次是 C281/C314（`crates/` 里的证据已远超 checks-owed.md 当前文字描述，但逐句核对后仍各差一处）。C80 核查过程中意外发现一个当前被容忍、与本行主题直接相关的 I-3.1 红（`second_transaction_step_zero_layer0.rs:918-922`，2026-09-17 当天写用例时发现），建议主 agent 另行核实是否要为它单独立一条 checks-owed 记录。

