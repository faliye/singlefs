# m2-closeout-code-r1 云端正推腿报告（Sonnet）

判据格：Z2（准入）、Z4（checker 新判定）、Z6（崩溃注入与对拍）、Z7（头宽三方各算）。
立场：逐格核「代码做的是不是条款说的」。代码一律读快照 `refs/sop/m2-closeout-code-r1-snapshot`
（提交 `67f447de9761f826711565032d8ce16fbf44b902`），取到草稿目录
`/tmp/claude-1000/m2-closeout-code-r1-sonnet/tree`；下面「代码位置」一律是这份快照里的文件与行号。
kb 引文的行号在 kb 文件里现查（`grep -nF`），不从背景材料数。

## Z2　准入（实五、实审 A4、A4b）

### Z2-1　D28 已定项 1：准入不等式的权威形式与逐设备合取 —— 兑现了条款

kb 原文：「可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数」（`.claude/kb/decisions/28-挂载期承诺量.md:19`），
「**逐设备算，每块盘各自满足**（逐设备合取）：式子对每块盘 d 各算一份，准入要每一块盘都够，一块盘不够就拒，不拿别的盘的富余补」（`.claude/kb/decisions/28-挂载期承诺量.md:24`）。

代码：`DeviceAdmissionTerms`（`crates/singlefs-core/src/admission.rs:222-239`）与 `PoolWideCommitments`（`crates/singlefs-core/src/admission.rs:242-250`）分别装五项自己盘的值与三项全池量；`available_on_each_device`（`crates/singlefs-core/src/admission.rs:356-380`）逐盘算「容量 − (自己四项 + 全池三项各摊份额)」；`admit_on_every_device`（`crates/singlefs-core/src/admission.rs:415-457`）逐盘比可用与需求，任一盘不够即整体拒、不拿别的盘补——与「逐设备算，每块盘各自满足」逐字对应。

推翻条件：若 `admit_on_every_device` 在某一盘不够时仍放行（例如把各盘可用求和后与需求求和比较），或 `available_on_each_device` 把三项全池量算成「取任意一盘的值」而非「按副本数摊份额」，这一判即倒向「和条款说反话」。

### Z2-2　D28 已定项 3：实例切换的预留 —— 兑现了条款

kb 原文：「预留 = (N_switch + 1) × 一次切换的最坏量，按设备算；一次切换的最坏量 = 实例表链重写 + 暖机空发布。」（`.claude/kb/decisions/28-挂载期承诺量.md:82`），
「**链重写**：副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节」（`.claude/kb/decisions/28-挂载期承诺量.md:84`），
「**发布路径用 rows0 + 1**：发布准入里的切换预留按下一次可写挂载的 rows0 算（这次挂载的 rows0 + 1，下一次挂载必写一行）；可写挂载自己判时照旧按这次的 rows0」（`.claude/kb/decisions/28-挂载期承诺量.md:88`）。

代码：`instance_switch_reserve_on_one_device`（`crates/singlefs-core/src/admission.rs:190-218`）按 `pages_of_the_chain = max(1, ⌈(rows0+N_switch)/369⌉)`、`chain_rewrite = 32768 × pages_of_the_chain`、`warm_up = c_max字节 × R`、`one_switch = chain_rewrite + warm_up`、`总预留 = one_switch × (N_switch+1)` 逐项对应（式子里的「副本数 × 32768」在按设备摊之后就是这一块盘自己那一份 32768 × 片数，与 kb 原文括注「两份落两块盘，所以按设备各算一份 32768 × 片数」一致）。`InstanceRowsOfTheSwitchReserve` 枚举（`crates/singlefs-core/src/admission.rs:810-830`）区分 `OfThisMount` 与 `OfTheNextWritableMount`（`rows_of_this_mount + 1`），`admission_reading_before_a_publish`（`crates/singlefs-core/src/admission.rs:860-869`）用后者、`admission_reading_of_a_writable_mount`（`crates/singlefs-core/src/admission.rs:874-883`）用前者，与「发布路径用 rows0 + 1；可写挂载自己判时照旧按这次的 rows0」逐字对应。

推翻条件：若发布路径与可写挂载路径调换（发布用 `OfThisMount`、挂载用 `OfTheNextWritableMount`），或 `pages_of_the_chain` 漏了 `max(1, …)` 下限，即倒向「和条款说反话」。


### Z2-3　D28 已定项 4：checkpoint 保留池的 ckpt_cost —— kb 原文与代码不一致，但去向已知（不算新打中）

**两边并排**：

- kb 已定项 4 原文（`.claude/kb/decisions/28-挂载期承诺量.md:106`）：「**形态**：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数 + 1（树表），每次发布按当时的树高重算」——按树高求和，不提「按盘分路」「每块盘两条叶路径」。
- 代码（`crates/singlefs-core/src/admission.rs:476-478`）：「一律按最坏情况计（用户 2026-09-27 定：游标跨叶、中央映射树多层都不许少扣；此前按盘分路的那一版只罩「每块盘一片叶、中央映射树 1 层」）：- 分配记录树：每块盘两条从叶到根之下那一层的路径加共用的根，盘数 × 2 × (高 − 1) + 1」——按盘分路、每块盘两条叶路径。

两者字面不同：kb 原文是「Σ 树高」的单一求和，代码是「盘数 × 2 × (高−1) + 1」的按盘展开式，在盘数 > 1 时两式取值不同（两盘高 3 时 kb 原文读法给 3、代码给 9）。

**为什么不算「和条款说反话」**：这一处不一致的根源是一条同日（2026-09-27）的用户定案还没写回 kb——`records/2026-09-27-代码审阅38条去向.md:50`：「20 | 「改条款按盘分路计并改实现」：D28（挂载期承诺量） 已定项 4 改成按实写计，先量清少扣多少、再改实现与用例 | A2 批之前先量（计数实验或实现员的量），改法走最终代码三方；kb 写回 D28 已定项 4」——这一条明写「改法走最终代码三方」，即这一轮代码三方本身就是 kb 写回之前要走的那一步；代码是在兑现这条尚未写回的用户定案，不是自行替条款做选择。判定按「兑现了条款」记（对象是记录的用户定案，不是 kb 现在的字面），同时点名 kb 待更新。

**已知、不重报的那一半**：body 二节已点名「ckpt_cost「每块盘两条叶路径」可能没算全」（`m2-rev-a4b-implementer-report.md` 第八节第 2 条）——代码自己在 `crates/singlefs-core/src/admission.rs:494`「⚠️ 射程：分配记录树的「每块盘两条路径」罩的是 bump 游标在一个开放段里走、跨过一片叶的末槽那一次」也点名同一处缺口（一次发布里开放段装不下、开新段或回落到最低空槽时，一块盘上改的叶可以多于两片，条款没写怎么计），去向 A4c，不在这里重报。

推翻条件：若 kb 已定项 4 写回之后仍保留「Σ 树高」的旧式子（与代码继续不一致且没有后续用户定案支持），或 `records/2026-09-27-代码审阅38条去向.md` 第 20 条被查明是杜撰、代码里「用户 2026-09-27 定」的引用找不到对应的定案，则应改判「和条款说反话」。

### Z2-4　D2 已定项 13 的例外 / C565：挂载处推满仍不够 —— 兑现了条款

kb 原文（`.claude/kb/decisions/02-RAID条带策略.md:224`，例外那一句）：「推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（用户 2026-09-26 定，被攻过零轮）」；`checks-owed.md` 已还清表（`.claude/kb/checks-owed.md:507`）：「C565 | 挂载处推满仍不够怎么收尾没定 | 用户 2026-09-26 定挂载照样做成（D2（RAID 条带策略） 已定项 13 那一格的例外）；实五实现」。

代码：`push_floor_raises_after_the_row_publish`（`crates/singlefs-core/src/mount.rs:3367-3422`）循环推抬 F 空发布，推满仍不够时返回 `Ok(MountSpaceAdmission::StillShortAfterTheFloorRaises { .. })`（`crates/singlefs-core/src/mount.rs:3404`）——是 `Ok`，挂载成功交回，不是错误；函数头注释（`crates/singlefs-core/src/mount.rs:3353-3355`）逐字写明「推满仍不够时怎么收尾条款没定（C565）：这里取实现员提的那一种——挂载照样交回做成」。

推翻条件：若推满仍不够时函数返回 `Err`（挂载被拒），或之后的发布不走发布准入而是无条件放行/无条件拒绝，即倒向「和条款说反话」。

### Z2-5　D16 已定项 1「准入」那一行：先推空发布抬 F 再判、B = 8 —— 兑现了条款

kb 原文（`.claude/kb/decisions/16-发布语义.md:42`）：「准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判……一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC」。

代码：`PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8`（`crates/singlefs-core/src/mount.rs:1648`），`push_one_floor_raise_within_the_admission_budget` 在 `publishes_of_this_admission > PUBLISHES_PER_ADMISSION_AT_MOST` 时停（`crates/singlefs-core/src/mount.rs:1834-1836`）；`admission.rs` 模块文档（`crates/singlefs-core/src/admission.rs:28-29`）「准入不够、或准入放行而落点取不到时先推空发布抬 F 再判」与 kb 原文逐字对应。

推翻条件：若 `PUBLISHES_PER_ADMISSION_AT_MOST` 不是 8，或写行那次发布之前也推了抬 F（kb 原文明写「写行那次发布之前不推」），即倒向「和条款说反话」。

## Z4　checker 新判定（实审 B1、B2、B2b、I-1.11；实四乙、实六、实八的 checker 部分）

### Z4-1　I-1.11（映射 key 与单元头相符） —— 兑现了条款

kb 原文（`.claude/kb/invariants.md:34`）：「任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段……分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树……不等判损坏」，实现状态列同一行：「已实现（2026-09-27，池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`……」。

代码：`walk_central_mapping_entries`（`crates/singlefs-checker/src/walk.rs:859-966`）逐条判：① 类标签不在登记表（`MappingKeyUnitClass::Unregistered`）判违例并跳过读单元（`walk.rs:871-878`）；② 读出单元后比头里类标签与 key 类标签（`walk.rs:895-904`）；③ 比出生 txg / 实例代号 / 尾段（`walk.rs:920-932`）；④ 码 2/3 尾段后补零 2 字节（`walk.rs:936-946`）；⑤ 码 1/3 的出生树等于单元头偏移 43（`walk.rs:948-964`，码 2 因 C289 未定不判）。每一步都用同一个判定名 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，与 kb 原文逐句对应，且 kb 原文自己也写明「码 2 的出生树随 C289 定」不判——代码同样跳过（`walk.rs:963`，`BirthTreeInTheHeader::FieldStillOpenUnderC289 => {}`）。

推翻条件：若代码在类标签不在登记表时仍继续按某个默认类型读单元（而不是判违例后跳过），或码 2 出生树被错误地拿去比对（凭空替 C289 做了选择而不承认），即倒向「替没写的条款做选择」或「和条款说反话」。

### Z4-2　I-8.6（反向链算法）计数器 0 判违例 —— 兑现了条款

kb 原文（`.claude/kb/invariants.md:87`）：「jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）」，实现状态列同一行：「计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`……）」。

代码：`ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`（`crates/singlefs-checker/src/walk.rs:4713-4715`），`expected_back_chain_of` 在 `counter == 0` 时返回它（`walk.rs:4725-4726`），`judge_journal_back_chain` 在拿到这个变体时判 `"I-8.6"` 为 `false`（`walk.rs:4822-4831`）——与 kb 原文「计数器为 0 的记录判违例」逐字对应；旧行为（判不了跳过）在注释里留痕：「实审 B1 把它当判不了跳过，用户 2026-09-27 定判违例」（`crates/singlefs-checker/src/walk.rs:4821`）。

推翻条件：若 `expected_back_chain_of` 在 `counter == 0` 时返回 `PredecessorNotOnThisDevice`（判不了、跳过）而不是 `CounterZeroHasNoLogicalPredecessor`（判违例），即倒向「和条款说反话」。

### Z4-3　I-5.1（物理范围不重叠）引用按版判、共享子树合并 —— 兑现了条款

kb 原文（`.claude/kb/invariants.md:181`）：「引用按版判（2026-09-27……）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红……别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）」。

代码：`note_references_of_a_pointer_followed_by_the_walk`（`crates/singlefs-checker/src/walk.rs:591-…`）注释写明「判 **I-5.1（物理范围不重叠）** 的「同一版里第二次引用同一个落点」那一格——两个 inode 的 extent 指着同一个数据单元、两条子指针指着同一个节点，都在这里红」（`walk.rs:588-589`）；`merge_placements_below_a_unit_walked_by_an_earlier_version`（`walk.rs:567-583`）对已走过的单元不重走、把它下面的落点从缓存表 `placements_referenced_below_a_walked_unit` 展开逐个并进这一版再判——函数名与 kb 原文括注里点名的函数名字面相同。

推翻条件：若合并共享子树时漏了某一层孩子（`expanded` 集合插入判断写反、或 `pending` 没有把新展开的孩子继续压栈），会让老一版共享子树之外又指了子树里面某个单元的情形判不出来——这正是代码注释自己点名的「实审 B1 留下的缺口」（`crates/singlefs-checker/src/walk.rs:566`），本轮判为「兑现了条款」是指判定逻辑与主检查存在、字面对应，不代表这一处已知缺口被补全。

### Z4-4　I-7.4（近 K 代块未被复用）被抛弃根那一半 —— 兑现了条款

kb 原文（`.claude/kb/invariants.md:55`）：「被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根……各用一份自己的走读走一遍，只看读回来对不对……根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立」。

代码：`judge_blocks_referenced_by_abandoned_roots`（`crates/singlefs-checker/src/walk.rs:5364-…`），函数前注释（`walk.rs:5356-5362`）：「I-7.4（近 K 代块未被复用） 被抛弃根那一半（代码审阅 6c 第 7 条，用户 2026-09-27 定「checker 现在就补被抛弃根那一半」）：……只看它引用的块读回来对不对——有一个单元校验和对不上或头用不了，就是它指着的块被重新分配或抹头了，I-7.4 按这条根判一格红……根环有读不出的槽、最新根的实例表里又有行……这一半判不了，I-7.4 没有违例时整条报「不适用」、不报成立」——与 kb 原文逐句对应。

推翻条件：若该函数在根环有读不出的槽、实例表有行、没有违例时仍报「成立」而不是「不适用」，即倒向「和条款说反话」。

### Z4-5　I-7.7（系统配置实例代号不低于根环）有读不出的槽时两句各怎么判 —— 兑现了条款

kb 原文（`.claude/kb/invariants.md:58`）：「① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号……不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」」。

代码（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_carriers`，diff 里 `346f5e6` → 快照 一段）：`instance_carriers` 改交回 `InstanceCarriersOnDisk { carriers, some_slot_is_unreadable }`；`some_slot_is_unreadable` 为真时①这一半整段走 `judgements.part_of_the_range_not_judged("I-7.7", …)` 报「读不全」，不判成立也不判违例；②这一半只在「各盘的号不等」（`disks_disagree = pool_lowest < pool_highest`）时才有对象，仍在读得出的载体上继续判——代码里这一处的注释逐字写明分工：「① 条款逐字「任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」」：读不全时这一半不判，整条不报成立。② 的不适用条件是另一件事（镜像记不下这次挂载独占打开的集合），读不全时照样在读得出的载体上判：找到较大者就红」——与 kb 原文①②两支的射程逐字对应。

推翻条件：若某一块盘读不出时①②被合并成同一个「不适用」（旧行为，kb 原文已改判两支各自的射程），或②在「各盘号相等」时仍去找「较大者」（kb 原文明写②只在不等时有对象），即倒向「和条款说反话」。

## Z6　崩溃注入与对拍（实七、实八、实审 B3b、B3c-1）

### Z6-1　D13 已定项 7：三截判定（崩溃镜像 / 可写挂载后 / 挂载途中二次崩溃三段） —— 兑现了条款

kb 原文（`.claude/kb/decisions/13-验证路线.md:134`）：「每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）」；`.claude/kb/decisions/13-验证路线.md:140`：「层 0 不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」）」。

代码：`CrashStateStage` 三个变体（`crates/singlefs-harness/src/crash_injection.rs:294-301`）——`CrashImage`（注释：「崩溃镜像本身：只读恢复（看 journal）、问模型、池级 checker、记录核对器」，`crash_injection.rs:295`）、`AfterTheWritableMountAndOnePublish`（注释：「在那份崩溃后镜像上可写挂载（取号、写行、暖机）再发一次布之后的池：池级 checker」，`crash_injection.rs:297`）、`SecondCrashInsideTheWritableMount(WritableMountPhase)`（注释：「可写挂载途中再崩一次（二次崩溃），崩在挂载的这一段里：只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）」，`crash_injection.rs:299`）；`WritableMountPhase` 恰有三个变体 `Acquisition`（取号，`crash_injection.rs:267-268`）、`RowPublish`（写行，`crash_injection.rs:269-270`）、`WarmUp`（暖机，`crash_injection.rs:271-272`），与 kb 原文「取号、写行、暖机三段各摆一个二次崩溃状态」逐字对应。

推翻条件：若 `CrashStateStage` 少了某一截、或 `WritableMountPhase::ALL`（`crash_injection.rs:276-280`）不是恰好这三个变体，或 `SecondCrashInsideTheWritableMount` 那一截的记录核对器读了整条流而不是「只核挂载自己写出的那几次发布」，即倒向「和条款说反话」。

### Z6-2　model.rs / model_comparison.rs：ObservedRoot 与分配记录两个方向都比 —— 兑现了条款

代码：`judge_allocation_generations` 注释（`crates/singlefs-harness/src/model.rs:2044`）：「两个方向都比（代码审阅第 12 条）：实现交回的每个角色这一版都要有，模型这一版的每个角色实现都要交回」；实现（`model.rs:2077-2091`）先检查模型期望的每个角色是否都在实现交回的集合 `handed_in` 里（`model.rs:2077-2091`），再检查实现交回的每条记录对应的角色是否都在模型的 `role_written_at` 里（`model.rs:2092-2098`）——两个方向各有一段独立的检查，与注释「两个方向都比」对应。`ObservedRoot`（`model.rs:420-427`）今天有 `key`、`journal_counter`、`rollback_floor`、`file`、`instance_table`、`unit_allocation_records` 六个字段；对照 `346f5e6` 到快照的 diff，这一轮新增/改类型的是 `file`（原 `has_file: bool` 换成 `ObservedFile` 枚举）、`instance_table`（新增字段）、`unit_allocation_records`（类型从 `Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)>` 换成 `ObservedUnitAllocationRecords` 枚举）三项，body「`ObservedRoot` 三个字段」指的是这三个新增/改类型的字段，不是结构体今天字段总数。

推翻条件：若「两个方向都比」的第二段检查（`model.rs:2092-2098`）被删掉、或只保留第一段（只查模型期望的角色在不在实现里，不反过来查实现多交回的角色），即倒向「替没写的条款做选择」（模型对拍这条纪律没有 kb 已定项文字钉死「两个方向」，这一处的依据是「代码审阅第 12 条」这一条用户定案，本轮判它兑现了这条定案，而非 kb 里某条已定项的字面）。

## Z7　头宽三方各算（实审 C11）

### Z7-1　D13 已定项 5：三份独立实现 + 交叉断言 + 6 个 format-const —— 兑现了条款

kb 原文（`.claude/kb/decisions/13-验证路线.md:86`）：「checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改……**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空」；`.claude/kb/decisions/13-验证路线.md:89`：「生成器拒绝发射没有 kb 落点的常量，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量」。

代码：`crates/singlefs-format/src/lib.rs:52`（`index_node_header_bytes` 之外，模块文档）：「含 key 区间与预留位的头宽 `86 + 2 × key 宽 + 29` 这条式子**不在这里**：这里只放标量，core（`singlefs_core::unit::index_node_header_bytes`）……」；format 只发射三个标量 `NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29`、`INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86`、`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2`（`crates/singlefs-format/src/lib.rs:30,47,56`），不发射算式。core `index_node_header_bytes`（`crates/singlefs-core/src/unit.rs:140-142`）、checker `index_node_header_bytes`（`crates/singlefs-checker/src/lib.rs:301-308`）、模型 `index_node_header_bytes`（`crates/singlefs-harness/src/model.rs:267-270`）各自独立地把这三个标量代入 `86 + 2×key宽 + 29`，互不调用（三处文档注释各自写「不调实现、也不调……」：`crates/singlefs-core/src/unit.rs:136-137`、`crates/singlefs-checker/src/lib.rs:297-298`、`crates/singlefs-harness/src/model.rs:263-264`）。singlefs-format 自己另有一份 `index_node_header_bytes`（`crates/singlefs-format/src/lib.rs:466-470`），但它在 `#[cfg(test)] mod tests`（`crates/singlefs-format/src/lib.rs:286` 起）里面，只给 format 自己的单测用，注释写明「只在这份单测里用，核下面那几个字面量；core、checker、模型各自的那一份不调它」（`crates/singlefs-format/src/lib.rs:464-465`）——不是发射物，不构成「共享一份算式」。

交叉断言：`crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`——第一条测试在 `key_width_byte in u8::MIN..=u8::MAX`（`:46`）上逐个比三份结果，末尾断言 `key_widths_checked == 256`（`:63-66`）确认 256 个取值一个不落；docstring（`:11-16`）交代取值域上界 255 的推法（key 宽字段偏移 51、宽 1 字节）。第二条测试把 6 棵树登记的 `format-const` 头宽（`INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES` 等，`:79-116`）逐个与三份在该树 key 宽上的值比对——与 body「另钉 6 个 `format-const` 头宽」的数目一致（该文件 `registered` 数组字面量（`:79-116`）里逐个构造 `IndexNodeHeaderWidthRegisteredInTheKnowledgeBase { .. }` 6 次，行号 80、86、92、98、104、110，不含它自己的 struct 定义那一行 70）。

对「三份是不是真独立」的问句作答：三份**互不调用彼此**（各自文档注释与函数体内均无跨 crate 调用），只共享来自 `singlefs-format` 的三个标量常量——这正是 D13 已定项 5 允许共享的那一样东西（「只共享一份从 kb 生成的常量」），不是被明令禁止的「共享格式解析 / 偏移函数」；format 自己额外的测试用副本不被三份调用，也不算破坏独立性。对「扫的取值域是不是全的」作答：0..=255 覆盖了 key 宽字段（1 字节）的全部取值，取值域按字段宽度封顶，是全的。

推翻条件：若 core / checker / model 三份里任意一份的实现被改成互相调用（例如 checker 直接调 `singlefs_core::unit::index_node_header_bytes`），或交叉测试的取值域收窄到 0..=255 之外的某个子集，即倒向「和条款说反话」。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z2-1 D28 已定项 1 逐设备合取 | 兑现了条款 | `admit_on_every_device` 逐盘比可用与需求，一盘不够即整体拒 |
| Z2-2 D28 已定项 3 切换预留 | 兑现了条款 | 链重写 + 暖机、rows0/rows0+1 的用法与 kb 逐字对应 |
| Z2-3 D28 已定项 4 ckpt_cost | 兑现了条款（对象是尚未写回 kb 的用户定案，非当前 kb 字面） | kb 原文「Σ 树高」与代码「盘数×2×(高−1)+1」不同，去向已知：`records/2026-09-27-代码审阅38条去向.md` 第 20 条要求「kb 写回」 |
| Z2-4 D2 已定项 13 例外 / C565 | 兑现了条款 | 推满仍不够时 `Ok(StillShortAfterTheFloorRaises)`，挂载做成 |
| Z2-5 D16 已定项 1「准入」 | 兑现了条款 | `PUBLISHES_PER_ADMISSION_AT_MOST = 8`，写行之前不推 |
| Z4-1 I-1.11 | 兑现了条款 | `walk_central_mapping_entries` 五步判定与 kb 五句逐一对应 |
| Z4-2 I-8.6 计数器 0 | 兑现了条款 | `CounterZeroHasNoLogicalPredecessor` 判违例，不再判不了跳过 |
| Z4-3 I-5.1 按版判、共享子树合并 | 兑现了条款 | 函数名与 kb 括注点名的函数字面相同；已知缺口未在本轮补全 |
| Z4-4 I-7.4 被抛弃根那一半 | 兑现了条款 | 读不出槽时报不适用、不报成立，与 kb 逐字对应 |
| Z4-5 I-7.7 有读不出的槽 | 兑现了条款 | ①②两支的射程按 kb 拆开判，不再合并成一个「不适用」 |
| Z6-1 D13 已定项 7 三截 | 兑现了条款 | `CrashStateStage` 三变体、`WritableMountPhase` 三变体与 kb 逐字对应 |
| Z6-2 model.rs 双向比 | 兑现了条款 | `judge_allocation_generations` 两段检查各管一个方向 |
| Z7-1 D13 已定项 5 三份独立 + 交叉断言 | 兑现了条款 | 三份各写各的算式、只共享标量；交叉测试覆盖 key 宽 0..=255 全域，钉 6 个 format-const |

## 没做什么

- **不判 Z1、Z3、Z5、Z8**：按分工表归云端攻方（Opus）。
- **不判 Z2 里在飞、不在这份快照里的部分**：A4c（`admission.rs`、`allocator.rs`、`mount.rs` 每块盘改几片叶、C545、压小容量）——按正文二节「在飞、不在快照里、归第二轮」列出，不在这份快照上判。
- **不重报已知缺口**：ckpt_cost「每块盘两条叶路径可能没算全」（Z2-3 已点名去向 A4c）；I-5.1 共享子树合并的「实审 B1 留下的缺口」（Z4-3 已点名，代码注释自己承认未补全）；C554（挂载时读不出的最新根被当成抛弃）——这一形与 Z4 无关（归 C554 修法，用户定，第 43 段等门禁 74 号红在这一形，不算这一轮新打中）。
- **不独立复算准入或头宽的绝对值**：ckpt_cost 在 `two-4GiB`、`two-1GiB`、`two-4GiB-mapping-4-8` 三格的具体数、头宽在 k=8/24 上的数值，分工表交本地攻方按事实表逐格算，本报告只核代码与条款字面是否对应，不重复算术。
- **不判合并次序（Z8）**：`mount.rs`/`transaction.rs` 被多批实现员各自改过之后的合并次序是否与各报告一致，归 Opus。
- **不跑测试**：本轮「重型测试：不跑」，`index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`、`checker_narrow_invariants_and_abandoned_roots.rs` 等用例只读源码确认存在与断言内容，没有 `cargo test` 复跑；若这些用例实际跑不过，会推翻本报告对应格的「兑现了条款」判定。
- **不判 D28 已定项 4 kb 写回之后的最终文字**：那要等主 agent 或 kb-scribe 按本轮判决写回之后才有对象。
