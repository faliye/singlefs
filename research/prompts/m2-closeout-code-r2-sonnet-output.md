# m2-closeout-code-r2 云端正推（Sonnet）：Y3、Y5

轮名：m2-closeout-code-r2。立场：云端正推，逐格核「代码做的是不是条款说的」。
代码快照：`refs/sop/m2-closeout-code-r2-snapshot`（提交 `30c084138f7da7488f27874417902caeb5272a15`），
在 `/tmp/claude-1000/three-way-forward-r2/snapshot/` 展开；下文所有 `crates/...:行号` 均指这份快照，
不是主工作区（主工作区在被改，本报告没读它）。kb 引文行号是仓里 kb 文件此刻自己的行号（现查，不从背景材料数）。

## Y3　准入按最坏情况（A4c、A4d、A4e）

### 问 1：K1 整棵树与逐设备合取在两块盘答不同落点时是不是仍是上界

**结论：兑现了条款。**

D28（挂载期承诺量） 已定项 1 逐字：「逐设备算，每块盘各自满足（逐设备合取）：式子对每块盘 d 各算一份，准入要每一块盘都够，
一块盘不够就拒，不拿别的盘的富余补……后三项——待删占用、已承诺预留、checkpoint 保留池——是全池的承诺量，按全部副本之和记字节……
摊到每块盘各扣」（`.claude/kb/decisions/28-挂载期承诺量.md:24`）。
已定项 4 逐字：「分配记录树这一项按 K1 算：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数
= ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1，S_L = 812 × 169^L）；根层级是根节点码 2 头里现读的层级……不读这一版的状态、不依赖 ckpt_cost，
恒是上界」（`.claude/kb/decisions/28-挂载期承诺量.md:107`）。

代码：`allocation_record_tree_positions_over_the_unit_areas` 对 `allocator.devices` 逐盘累加、对每一层累加位置数
（`for device_map in &allocator.devices {` `crates/singlefs-core/src/admission.rs:743`，起步 `positions: u64 = THE_ROOT_COVERING_THE_WHOLE_KEY_SPACE`
`crates/singlefs-core/src/admission.rs:741`），得到的 K1 是**对全部设备求和后的单一标量**；`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`
把它并进 `MetadataBlocks`（`crates/singlefs-core/src/admission.rs:717`）。这个单一标量随后经 `units_of_a_publish_to_land`
被加进 `commit_generated_slots`（`.checked_add(checkpoint_cost.0)` `crates/singlefs-core/src/admission.rs:533`），
再由 `admit_the_units_landing_on_every_device`——它的文档注释自己写「逐设备合取（D28（挂载期承诺量） 已定项 1「逐设备算，每块盘各自满足」；
D3（空间分配） 已定项 7 合取表第 1 条）」（`crates/singlefs-core/src/admission.rs:413`）——**逐盘各自**与同一个 `commit_generated_slots`
比较（`crates/singlefs-core/src/admission.rs:576-586`）。

这样做在数学上仍是上界：K1（对全部设备求和）恒 ≥ 任一单个设备自己实际可能改动的节点数（求和多加了非负项，不会更小）；
把同一个更大的数当成每块盘各自的需求，只会让某些盘被更保守地扣，不会漏扣。两块盘几何不同（不等大池）时同理——K1 仍按「这块盘自己的
单元区 [s, e)」逐盘累加后求总和，不因盘几何不同而失效。

**代码上的实测确认**（不只是推理）：`second_transaction_supplement_two_unequal_devices.rs` 的
`devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written`
（`crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs:353`）构造了两块盘对提交内生块给出
不同落点（大盘开新段、小盘回落到最低空槽）的场景：断言 `short_devices` 只报盘 1，`demand: BytesOnOneDevice(7_553_024)`
（`crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs:399`）——461 槽 = extent 根 1 + inode 叶容器 2 +
inode 根 1 + ckpt_cost 457，这个 demand 是同一个 K1 派生值，对两块盘（大小盘）**同样**成立，只是大盘段外有富余、没被列进
`short_devices`。这条用例在实审 A4e 报告里实测通过（`test result: ok. 3 passed; 0 failed`，
`research/prompts/m2-rev-a4e-implementer-report.md` 第 8 行）。

**什么现象会推翻它**：若某段历史里某块盘实际改动的分配记录树节点数超过了 K1（`allocation_record_tree_positions_over_the_unit_areas`
算出的那个数），K1 就不再是上界；A4d 报告的量表测试（`admission_checkpoint_cost_per_device_paths.rs`）钉住了 8 格的多扣区间，
任一格报「少扣」（多扣为负）即推翻（`research/prompts/m2-rev-a4d-implementer-report.md` 第 13 行「量表那份测试任何一格报少扣也推翻它」）。
这一条我没有另外造历史去测，是对已有实测（A4d 报告第二节量表 8 格与两段历史重放，`research/prompts/m2-rev-a4d-implementer-report.md`
第 24-73 行）与代码结构（求和是上界）的复核，不是新的观测。

### 问 2：C545 准入先拒之后有没有一条合法写被拒（假性 ENOSPC），是不是已登记在 C571 / C576

**结论：兑现了条款（这一格已经被登记，不是漏记）。**

有：A4c 报告量到「384 槎，崩了再挂，80 次空发布，顺序写到 6 个单元；写第 7 个单元时每块盘段外成对空槽 0 对、段内 79 对，
按字节算的式子每块盘可用 49 槎（需求上界 33 槎），数据单元 0 落点被拒」（`research/prompts/m2-rev-a4c-implementer-report.md` 第 14 行）。
这是一次「盘上物理确有空间（段内 79 对）却仍被拒」的写，钉进了用例
`a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_the_space_admission_before_any_write_while_the_byte_formula_admits`
（`crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`，
按 `crates/mutations.tsv` 里 C545 那一行点名，见 `.claude/kb/checks-owed.md:597`）。

这一格的假性 ENOSPC 疑虑已登记为 C571：「这次挂载里，D3（空间分配） 已定项 8 第 2 条为聚簇提交对用户数据关着的段内空槎
（A4c 报告量到一段 79 对），`df` 报的可用空间要不要把这些槎算进去没有条款；若算，用户可能看着 `df` 报有空间却写入 ENOSPC，
那会是 D3（空间分配） 已定项 9 第 1 条说的假性 ENOSPC 那一类」（`.claude/kb/checks-owed.md:484`）。C571 的判据栏明写「先要有
「段内关着的空槎计数」这个读数……才谈得上量 `df` 会不会把它们算进去」——`crates/` 里没有 `df` 的实现
（A4d 报告实测 `grep -rnE 'fn ... df|statfs|reported_free' crates` 零命中，`research/prompts/m2-rev-a4d-implementer-report.md` 第 190 行），
所以这一格今天仍是「待判」而不是「已判红」：不是漏登，是还没到能判的那一步。

C576 登记的是另一件、相关但不同的事：三条**既有**用例（数据单元落点差异、小盘写满、提交内生块各盘落点不同）因为准入先拒改变了它们的交回
成员或数值，「改法两条路（装只供测试的开关跳过准入、或改钉成 `SpaceAdmissionRefused`）与它们点名的第 12、76、77、78、80、145、149 行变异
要不要换靶子没定」（`.claude/kb/checks-owed.md:489`）。C576 管的是**测试期望值与变异靶子的一致性**（后来已由 A4e 批实现员按第二条路改钉，
见 `research/prompts/m2-rev-a4e-implementer-report.md`），不是「合法写被假性拒绝」本身——这三条用例里的盘都是真的写满或真的落点不同，
拒绝本身是合理的，只是拒绝的**成员**（`PlacementRefused` 还是 `SpaceAdmissionRefused`）变了。所以：**C545 那次「有真实空闲却被拒」的
唯一具体实测在 C571，C576 是另一类问题（测试期望值同步），两者都在册，没有第三种被拒的合法写落在册外。**

**什么现象会推翻它**：若能造出一次写，盘上**没有**任何段内/段外的聚簇段争用、K1 保留池也没有被过度扣（即真正的普通空闲空间），
仍被 `SpaceAdmissionRefused` 拒绝，且这个场景不落在 C571（段内关着）与 C545 现有用例的射程内，那就是一条不在 C571/C576 射程内的
新假性 ENOSPC，需要另立欠账。我没有构造这类历史去测；这是对 A4c/A4d/A4e 三份报告已交代的量测与 checks-owed 现状的复核，不是新观测。

## Y5　checker 新判定与树表排序（A3-checker、A3-checker-2、I-7.13 坏镜像、I-9.16）

### 问 1：每条新判定是不是只在它该有对象的状态上有对象（不适用与判绿分得开）

**结论：兑现了条款。**

checker 的判定累加器 `Judgements` 在汇总时按优先级取值：有第一处违例就报违例，否则若该条「射程里有一部分判不了」就报不适用，
否则若评估次数 > 0 就报成立，否则才落到 `into_report` 兜底分支返回的显式登记不适用（`crates/singlefs-checker/src/image.rs:172-176`）。
`judge()` 只在真的评估过一次时才计入 `evaluated`（`crates/singlefs-checker/src/image.rs:120-130`），`not_applicable()` 只在
`evaluated` 从未被写入时才生效（因为 `into_report` 先查 `first_violation`、再查 `part_of_the_range_not_judged`、再查 `evaluated`，
`not_applicable` 兜底最后一档）。这条机制不是这一轮新写的，是这一轮三个新判定沿用的既有约定。

- I-1.10 的扩展（条目宽为 0 时条目数须为 0）只在「解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表
（`EntryFieldTableRegistration::Registered`）」时才报违例（`.claude/kb/invariants.md:33`），树表条目与中央映射树因为没有登记条目字段表，
走的是另一条「只记走读失败」的路径，不会被这条判定错误标成「不适用」或「成立」。
- I-2.4 的扩展（格式版本、29 字节预留位、指针 MAC/nonce）在单元头总是存在、被跟随的指针总是存在的前提下无条件 `judge()`
（`crates/singlefs-checker/src/walk.rs:695`、`:730`；四个入口 `crates/singlefs-checker/src/walk.rs:790`、`:1440`、`:1766`、`:1916`），
没有可能落进「不适用」的分支。
- I-7.13 本身：`judge_system_configuration_values_the_reader_accepts` 对每个可择的槎 `judge(..., true, ...)`，
对每个带越界值的槎 `judge(..., false, ...)`（`crates/singlefs-checker/src/image.rs:466-485`），只有当**一块盘都交不出**任何自证过的
系统配置时才落到「池里没有一块盘交得出有效的系统配置」（`crates/singlefs-checker/src/walk.rs:5689-5696`）这一支显式不适用——这与
「判定该有对象」的字面要求一致：没有对象（没有一个可读的系统配置槎）才报不适用，有对象（至少一个可读的槎）就真的判一次。

### 问 2：I-7.13「一槎越界，其余全报不适用」会不会盖掉别的不变量本该报的违例

**结论：兑现了条款（这正是 I-7.13 已经落成的字面文本，不是隐藏的副作用）。**

I-7.13 的条款原文明写：「任一盘任一槎不满足其中一项即判红：checker 报违例、不作保，**且该池其余不变量一律报「不适用」**，与实现整池拒绝
挂载一致（用户 2026-09-27 定系统配置越界整池拒）」（`.claude/kb/invariants.md:64`）。代码：`check_pool_image` 判出
`any_system_configuration_slot_carries_a_refused_value` 之后，`for invariant in crate::image::IMPLEMENTED_INVARIANTS { root_ring_judgements.not_applicable(invariant, ...); }`
后立即 `return root_ring_judgements.into_report();`（`crates/singlefs-checker/src/walk.rs:5680-5687`）——**整棵走读一步都不会跑**，
不是某个判定跑完了又被覆盖成不适用。I-7.13 自己不会被这个循环盖掉：它在这个循环之前已经 `judge()` 过（`crates/singlefs-checker/src/image.rs:476-484`），
`into_report` 的优先级（先查 `first_violation`）保证它仍报违例，不被随后对同一编号调用的 `not_applicable` 冲掉。

这确实意味着：如果这块池**同时**存在一个系统配置槎越界**和**另一处与它无关的真实损坏（例如某个根记录的校验和对不上），checker 在
这次判定里只会报 I-7.13 违例，另一处真实损坏完全不会被走读到、更不会被报出来。但这正是条款要的（不是代码额外做的选择）：
条款给的理由是「与实现整池拒绝挂载一致」——真实的 `mount_writable` 在这种镜像上直接返回 `SystemConfigurationValueRefused`，
根本不会尝试走到别的检查（recovery.rs 的相应错误处理在越界即返回，不继续往下解析），所以「checker 在这张镜像上还能不能看见另一处
损坏」这件事，在生产路径里同样看不见——checker 的行为只是精确复现了这一点。

**什么现象会推翻它**：若能找到一次真实的 `mount_writable`（不是 checker），在同一张系统配置越界的镜像上，**没有**整池拒绝、
而是继续走到了别的检查并报出了别的错误，那 checker「其余全报不适用」就与实现不一致，需要重开。我没有跑这个对照（会涉及重型的可写
挂载路径），是对 A3a、A3-checker-2 两份报告已交代的实现行为（`RecoveryFailure::SystemConfigurationValueRefused` 整池拒绝挂载）
的复核，不是新观测。

### 问 3：I-9.16 两道合一、只比树表里有的种类、排在水位与缺树判定之前，各是条款字面后果还是选择

**结论：三条分开判——两条「替没写的条款做了选择」，一条是「兑现了条款」（次序）；此外 I-9.16 这个编号本身还没进 kb/invariants.md。**

先确认背景事实：`kb/invariants.md` 现在最高到 I-9.15，「现共 80 条在用（编号至 I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉），
另有六条退役，编号不回收）」（`.claude/kb/invariants.md:17`），全文档 `grep -n I-9.16` 零命中于该文件；但代码里
`IMPLEMENTED_INVARIANTS` 已经是 48 项、含 `"I-9.16"`（`crates/singlefs-checker/src/image.rs:57` 起，`pub const IMPLEMENTED_INVARIANTS: [&str; 48]`,
第 105 行是 `"I-9.16",`），`recovery.rs` 也已把这个字符串当不变量名用（`pub const TREE_TABLE_ENTRIES_ORDERING_CONTRACT: &str = "I-9.16";`
`crates/singlefs-core/src/recovery.rs:1865`）。实现员报告自己交代「立号来由：实审 A3b Q2……用户 2026-09-27 JST 17:4x 弹窗定「立不变量并同步」」，
「不变量原句（交 kb 第八批，我没写 kb）」（`research/prompts/m2-rev-tree-table-ordering-implementer-report.md` 第 30-34 行）——
这是用户已经批准要立、但 kb-scribe 那一批还没写回的中间状态，不是代码在没有批准的情况下自行加了一个不变量；不算「与条款说反话」，
但在「invariants.md 每加一条，checker 就要加一个检查……两者不同步的 commit 一律不收」（`.claude/kb/invariants.md:3-4`）这条纪律下，
这批代码若原样提交，会撞上这条纪律（本报告只指出这一事实，判不判红交主 agent 与门禁）。

**（a）两道要不要连中央映射树的号一起比：替没写的条款做了选择。**

D8（核心索引结构） 已定项 8 ② 逐字：「从 mkfs 之外的水位发第一个文件版本时，**八棵树**的号从水位起连号发，次序照格式常量 11..18 那一组
（extent、inode、分配记录、记账、**中央映射**、livelist、稀疏旁表、deadlist）」（`.claude/kb/decisions/08-核心索引结构.md:240`）——
中央映射树字面在这个「连号发、有次序」的名单里，是第 5 个。而 I-9.16 的实现只覆盖树表里的七棵（`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`
只有 7 个成员：`crates/singlefs-core/src/recovery.rs:1869-1877`），不比中央映射树的号是否落在这条序列该在的位置上。实现员自己把这一点
列成停下交主 agent 的问题：「② 管不管中央映射树的号。D8 已定项 8 ② 的发号次序里有中央映射树（第 5 个，第一版恒为 15），它不进树表。
……要不要把它纳进 I-9.16，条款没写，我没定」（`research/prompts/m2-rev-tree-table-ordering-implementer-report.md` 第 103 行）。
这与「已定项 8 ② 的字面名单包含中央映射树」直接对照，可以看出：**已定项 8 ② 定的是「八棵树按这个次序连号发」的写路径承诺，
没有定「I-9.16 这个读路径判定该不该核到中央映射树的号」**——I-9.16 是这一轮新造出来的、比已定项 8 ② 窄的一个可判定形式，
它的射程要不要与已定项 8 ② 的字面范围（八棵）对齐，是实现员在没有写好的条款上做的选择，而且他明确没有替用户拍板。

**（b）只比树表里有的种类：替没写的条款做了选择（但选择本身站得住）。**

D8 已定项 8 的排序契约段只写「条目按树 ID 升序排」（`.claude/kb/decisions/08-核心索引结构.md:245`），没有写「缺一种树的树表该怎么判」。
实现员的选择是：「缺哪一种、有没登记的种类，在核心层照旧由后面读那一棵的那一步报……checker 不另立判定。同一种出现两条时，checker 取
先出现的那条来比。核心层在这之前已经把「同一种两条」判成损坏了」（`research/prompts/m2-rev-tree-table-ordering-implementer-report.md`
第 104 行）。这不是条款字面写好的分工，是实现员按「不重复判定同一件事」的考虑做的选择——但它有一个可核的支撑事实：核心层确实在
`tree_table_entries_each_kind_at_most_once` 里已经把「同一种两条」判成损坏（该函数在 `rebuild_version` 里排在 I-9.16 判定之前调用，
`crates/singlefs-core/src/recovery.rs:2014-2015`），所以这个选择没有留下一个真正的判定空白，只是没有把它归进 I-9.16 这一个编号名下。

**（c）排在水位与缺树判定之前：兑现了条款（「在任何写之前拒」这条纪律的直接推论，不是自由选择）。**

`rebuild_version` 里的次序是：`tree_table_entries_each_kind_at_most_once` → `tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order`
（I-9.16，`crates/singlefs-core/src/recovery.rs:2019`）→ 逐条目判水位（`tree_identifier_of_the_entry_is_below_the_watermark`，
`crates/singlefs-core/src/recovery.rs:2020-2022`）→ `pointer_of`（缺树判定，`crates/singlefs-core/src/recovery.rs:2023-2030`）。
这个次序本身是 D28（挂载期承诺量） 已定项 1「接线」段那句「拒了在任何写之前返回：发布路径报 `PublishError::SpaceAdmissionRefused`，
可写挂载报 `MountError::SpaceAdmissionRefusedBeforeAcquisition`」（`.claude/kb/decisions/28-挂载期承诺量.md:33`）同样的纪律，与「可写挂载接着这一版发布，写者按种类装树表条目、断言树 ID
升序……走得到那条断言的镜像在任何写之前拒」（`crates/singlefs-core/src/recovery.rs:1879-1881`，函数文档）共同要求的结果：I-9.16 判定
本身就是为了在**任何**基于这个树表的读之前先把排序坏了的镜像挡住，挪到最前面是这条判定「存在的理由」的字面兑现，不是众多同样合理的
选项之一。实现员报告把这一点写成后果、不是问题：「**多处违例同时存在时，报哪一个错变了**。`rebuild_version` 里原来 ② 排在水位判定、
缺树判定之后，现在挪到它们之前。只违反其中一条的镜像，报错不变；同时违反 ② 和「缺树」或「水位」的镜像，现在报的是 I-9.16。这是
「在任何往下走之前拒」的直接后果」（`research/prompts/m2-rev-tree-table-ordering-implementer-report.md` 第 105 行）——他自己把这一点
归为「直接后果」而不是列进「停下交主 agent 的设计问题」那三条，与（a）（b）两条的处理方式不同，佐证了这里是三条里唯一「兑现了条款」的。

**什么现象会推翻它们**：
- （a）：若能找到 D8 已定项 8 或别的已定分项里，明写「I-9.16（或它的前身判定）只管树表里的七棵、不管中央映射树」，那这一格就该改判「兑现了条款」。
- （b）：若某个合法的、只缺一种树的树表镜像走到 `walk_to_file` 或 `rebuild_version` 时，没有被「后面读那一棵的那一步」报出任何错误（即真的漏判），
  那这个选择就不成立，需要改判「与条款说反话」（条款要求排序契约覆盖全部树表条目，缺项被放过）。
- （c）：若能构造一个次序颠倒（水位判定排在 I-9.16 之前）但仍满足「在任何写之前拒」的实现，且它与今天代码在某个多重违例镜像上给出的报错
  不同，就说明这个次序不是「拒在任何写之前」唯一能推出的形态，需要改判为选择。

### 问 4：checker 与 core 除常量模块之外有没有共用判定

**结论：没有——兑现了条款。**

D13（验证路线） 已定项 4 的射程句：「checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）」
（`.claude/kb/invariants.md:12`，「状态」一节引言段）。门禁 94 号（`94-checker-implementation-disjoint.sh`）在这一轮涉及 checker 的
三份实现员报告里都判过且都是绿：
- A3-checker：「✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个……）；
  checker 的 4 份源码零处引 `singlefs_core`」（`research/prompts/m2-rev-a3-checker-implementer-report.md` 第 136 行）；
- A3-checker-2：同一句原样（`research/prompts/m2-rev-a3-checker-2-implementer-report.md` 第 130 行）；
- 树表排序：「94 号绿」（`research/prompts/m2-rev-tree-table-ordering-implementer-report.md` 第 9 行），核心层的判定与走读那一份分开写
  （`recovery.rs` 里的 `tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order` 与 `walk.rs` 里独立的
  `judge_tree_table_entries_ordering`），文档注释自己写明「checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5）」
  （`crates/singlefs-checker/src/walk.rs:2608`）。

代码层面另一处直接证据：分配记录条目的字段偏移常量在 checker 里各写一份，注释写「checker 按字段表自己解一份，不与实现共用代码
（D13（验证路线） 已定项 5）」（`crates/singlefs-checker/src/walk.rs:2595`），与 recovery.rs 那边各自的常量（本报告未逐一列出，
门禁 94 号的判据正是逐个 crate 算依赖闸合子集为空）互不引用。

**什么现象会推翻它**：若门禁 94 号在打上这一轮全部补丁之后的主工作区上报非 0（例如 checker 出现一处 `use singlefs_core::...`），
就说明「只共享常量模块」不再成立。我没有在主工作区现状上重跑 94 号（主工作区在被改，禁读），是对三份实现员报告各自在自己副本上
跑出的 94 号结果的复核。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y3-1（K1 + 逐设备合取，两盘不同落点时） | 兑现了条款 | K1 是对全部设备求和的单一标量，恒 ≥ 任一单个设备的实际值，逐设备合取用同一个数比对仍是安全上界；D28 已定项 1「逐设备合取」与已定项 4「K1」两句字面就是这个做法，A4e 报告的不等大盘测试也实测到这一点 |
| Y3-2（C545 准入先拒之后的假性 ENOSPC） | 兑现了条款 | 唯一具体的「真有空闲却被拒」实测（384 槎段内 79 对）已登记在 C571；C576 管的是另一件事（既有用例的期望值同步），没有第三种漏登的情形 |
| Y5-1（新判定的不适用/判绿分得开） | 兑现了条款 | `Judgements::into_report` 的既有优先级（违例 > 部分判不了 > 已评估 > 显式不适用）保证只在真有对象时才 `judge()`，三条新判定都遵循这条既有约定 |
| Y5-2（I-7.13 盖掉别的不变量违例） | 兑现了条款 | 「其余一律报不适用」是 I-7.13 条款自己的字面文本（用户 2026-09-27 定），且与真实的 `mount_writable` 整池拒绝挂载一致；I-7.13 自己不会被这个循环盖掉 |
| Y5-3a（I-9.16 要不要连中央映射树的号） | 替没写的条款做了选择 | D8 已定项 8 ② 的发号名单字面包含中央映射树，I-9.16 只覆盖树表七棵；实现员自己列为未定、没拍板 |
| Y5-3b（I-9.16 只比树表里有的种类） | 替没写的条款做了选择 | 条款没写缺项怎么判，选择本身有 `tree_table_entries_each_kind_at_most_once` 兜底，不留判定空白 |
| Y5-3c（I-9.16 排在水位/缺树判定之前） | 兑现了条款 | 是「拒在任何写之前」这条既有纪律对这个新判定的直接推论，不是众多同样合理选项之一 |
| Y5-4（checker/core 共用判定） | 兑现了条款 | 门禁 94 号在三份相关实现员报告里都判绿，D13 已定项 4/5 的「只共享常量模块」字面成立 |

## 没做什么

- 不判 Y1、Y2、Y4、Y6、Y7、Y8（分给云端攻方与本地攻方的格）。
- 没有另起副本、另跑测试或崩溃枚举：本报告的证据全部来自快照代码现读与已交回的实现员报告（A4c、A4d、A4e、A3-checker、A3-checker-2、
  树表排序）里已经跑过的实测，我只做了交叉核对与文本-代码对照，没有产生新的实验产物。
- 没有判 I-9.16 这个编号本身该不该在这一轮的代码状态下就落进 `IMPLEMENTED_INVARIANTS`（那是「kb 与代码是否同步」的准入问题，不是
  「Y5 问的三条选择对不对」；本报告只在问 3 的开头指出这个事实，供主 agent 与门禁判断，不替它们下结论）。
- 没有跑 `cargo test`、`cargo build`、门禁阶段：这一轮我只读代码与 kb，不复跑（派发提示没有要求这条腿跑重型测试，且主工作区正被改，
  不应在上面跑测试）。
- 没有替不归 Y3、Y5 的欠账（C572、C578、C579、C573-C576 除已在 Y3 问 2 里核对过的部分）另行判定；这些在背景材料「四、已知」一节
  已标「主 agent 接受」或「记欠账」，不在这两格的问句射程内。

## 引文核对说明

交回前跑 `python3 research/scripts/cite-check.py research/prompts/m2-closeout-code-r2-sonnet-output.md --root /tmp/claude-1000/three-way-forward-r2/snapshot --root /home/fy5090/code/singlefs`：

```
✓ 核了 3 处引文，对上 3 处，没判 0 处（逐处列在上面）
```

**它只核到 3 处，不是这份报告只有 3 处引文**：`citations_in()` 按报告的**单一物理行**扫描正文，本报告里多数引文（正文为可读性手工换行、以及所引 kb 原句自身含嵌套「」引号）的「引文」与`（`路径:行号`）`落在不同物理行、或引文内部还有一层「」，两种情形都不会被它的正则捕捉到，是脚本的已知覆盖缺口，不是这些引文没被人核过。**每一处这份报告里出现的「引文」（`文件:行号`）在写进正文之前，我都用 `grep -n` 或 `sed -n` 现读过那个文件那一行/那一段，逐字核对过**（过程见本会话对 `crates/singlefs-core/src/admission.rs`、`crates/singlefs-checker/src/{image,walk}.rs`、`crates/singlefs-core/src/recovery.rs`、`.claude/kb/invariants.md`、`.claude/kb/decisions/{08,13,28}-*.md`、`.claude/kb/checks-owed.md` 的多次 `Read`/`grep -n` 调用）；机器复核只覆盖了脚本正则能捕捉到的那 3 处，其余靠人工，写在这里让主 agent 知道机器核过的射程有多窄。
