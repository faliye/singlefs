# m3-close-b19-reason-r3 正推腿（Sonnet）报告

日期：2026-11-03。立场：正推。逐句对 `crates/` 今天的实现核「实现做的是不是判决与条款说的」。

## 开头说明

- 没写模型，没起脚本，没跑 cargo。凡标「推的」的，是读代码与条款推出来的；凡标「待主 agent 跑」的，命令在「要主 agent 跑的命令」一节。
- 先核了开工快照：`sha256sum -c research/prompts/m3-close-b19-reason-r3-snapshot/crates-sha256.txt` 此刻 36 份全 OK，行号照此刻的文件。
- 没读禁读清单（攻方腿的报告与模型、`records/` 下的调度记录）。读了第二轮两腿留在 `research/prompts/m3-close-b19-reason-r2-sonnet-model/` 里的辩方模型输出（`out/*.txt`，冻结证据，只读）。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号都是此刻在 kb 文件里现查的。
- 每一问一个结论，用三个词：造得出 / 造不出 / 推的。

## Q1　I-3.1 取宽第五判与判决措辞的更正

### Q1 第 1 问　「至少一条未释放就比、两条都已释放不比」

**措辞更正站得住。** 判决写的是「改记之后各条未释放记录罩住的槽区间两两不相交」（`research/prompts/m3-close-b19-reason-r2-main-verification.md:17`），照字面只比「未释放对未释放」。
但判决自己的验收镜像之一是「隔离单元的尾槽压在一条释放代落在 (t, θ_high] 的已释放记录上」（`crates/singlefs-checker/invariants/05-空间记账.md:8`）：压住的那一条是已释放记录，字面读法抓不到它。
kb 条文已经照更准的读法写了：「改记之后每条未释放记录罩住的槽区间不与这块盘上别的记录（未释放或已释放）罩住的槽区间相交」（`crates/singlefs-checker/invariants/05-空间记账.md:8`），
实现与它逐句相符：`(true, true) => continue,`（`crates/singlefs-checker/src/walk.rs:10595`）跳过两条都已释放的，别的三种组合都比。
说明一处：实现员说「两张镜像都要拿未释放记录去比已释放记录」，只有第一张（攻方的隔离尾槽）必然如此；辩方那张（窄记录加尾槽上另一条记录）的另一条可以是未释放的（判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md:16` 只写「另一条记录」）。更正站住靠第一张就够。

**误红：造不出（推的）。** 挡住它的是进门条件，不是第五判自己：
- 第五判只在 `records_on_this_device_with_the_spans_set_to_the_unit_classes` 非空时才有对象（`crates/singlefs-checker/src/walk.rs:21691`「let records_on_this_device_with_the_spans_set_to_the_unit_classes = if lies_within_its_bounds {」：三判成立就是 `None`）。
- 该集合非空 ⇔ 这块盘上至少有一条未释放记录的跨度与它起点单元那一类的槽数不符（`crates/singlefs-checker/src/walk.rs:10523`「some_span_was_set.then_some(set_to_the_unit_classes)」；`crates/singlefs-checker/src/walk.rs:10508`「if record.is_released {」那一支原样返回已释放记录，不改记）。这正是 I-3.18 判红的那一类。
- 判到相交才红（`crates/singlefs-checker/src/walk.rs:21740`「if records_overlapping_in_the_unit_class_reading.is_empty() {」的 else 支）。
所以第五判能红的盘面，I-3.18 先红。合法写者写的跨度等于单元类，I-3.18 绿，第五判不进。崩溃态、隔离记号、读坏槽钉门槛三形都不改记录的跨度，进不了这一支。
第二轮判决同一格也是这个读法：「这一支只在按记录自述三判不成立时才进，进来的盘面 I-3.18 都红」（`research/prompts/m3-close-b19-reason-r2-main-verification.md:19`）。
推翻它的现象：一张 I-3.18 绿的合法镜像上 `walk::unreleased_records_overlapping_another_record_in_the_unit_class_reading` 返回非空，或 I-3.1 因第五判而红。

**漏判（两条都已释放而相交）：造不出（推的）。** 已释放记录在改记那一份里原样不动（`crates/singlefs-checker/src/walk.rs:10508`），两条已释放记录的相交与记录自述那一份相同。
I-5.4 判的正是这个：「同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交」（`crates/singlefs-checker/invariants/07-分配器.md:9`），最新根在它判的候选里。第五判不比这一对，没有丢信息。
一处缝（不是漏判）：第五判不进的盘面（三判在记录自述那一份就成立）上，改记后的相交没人看；那样的盘面 I-3.18 已红，整张镜像照红，只是红的集合少一项。判决验收镜像要求「只红 I-3.18 与新条」那一类断言在这样的盘面上会少一项，写测试时别拿它当判据。
推翻它的现象：一张两条已释放记录在记录自述下相交、而 I-5.4 与第五判都没报的盘面。

### Q1 第 2 问　第五判与 I-5.4 重复报或互相矛盾

**矛盾：造不出（推的）。** 两条判的是不同的读法：I-5.4 按记录自述的跨度，第五判按改记后的跨度（单元类），而且只判最新根一棵树。同一对记录在两种读法下可以：
- 两种读法都相交：I-5.4 红、第五判红，同一对点名两次，编号不同。**重复，但不矛盾**，各自说各自的读法。
- 只在自述读法下相交：I-5.4 红，第五判不红（改记后不相交）。
- 只在改记读法下相交：I-5.4 绿，第五判红，I-3.18 红。这正是第五判存在的理由。
没有一种组合是「一条说这对记录没问题、另一条在同一读法下说有问题」。
重复报的出处：违例文字点名每一对的两条记录与相交的槽（「fn allocated_statistic_violated_by_records_overlapping_in_the_unit_class_reading(」，`crates/singlefs-checker/src/walk.rs:10706`），I-5.4 点名的是同一对起点槽，读的人会看到两条红指向同一对。可以接受，不改。

### Q1 第 3 问　部分没判说明的数能不能分出「改记后重叠」与第二轮那几形

**能，由构造分开（推的）。** 改记后相交的盘面走红的那一支：`judgements.judge_naming_the_slots("I-3.1", false, …)`（`if records_overlapping_in_the_unit_class_reading.is_empty() {` 的 else 支，`crates/singlefs-checker/src/walk.rs:21740`），不进部分没判。
部分没判说明（「fn allocated_statistic_left_to_the_span_invariant(」，`crates/singlefs-checker/src/walk.rs:10635`）只在相交列表为空时才写（`crates/singlefs-checker/src/walk.rs:21740`「if records_overlapping_in_the_unit_class_reading.is_empty() {」的 then 支）。
所以读到这份说明的人已经知道没有改记后重叠；说明里的「被改记的记录」与「两种读法差在哪几槽」描述的是 I-3.18 红的那几个单元「真占几槽说不准」。不需要从数里再分。
推翻它的现象：说明文字出现在一张改记后确有相交的盘面上。

## Q2　I-3.21 与「0 条不适用」的更正

### Q2 第 1 问　「0 条不适用」：真病会不会藏在这一条之外

**造不出（推的），另有一处措辞要补。**

实现的分法（`crates/singlefs-checker/src/walk.rs:10770`「tree_table_entries: 0,」那一支报不适用，理由常量 `crates/singlefs-checker/src/walk.rs:10774`「NO_ALLOCATION_RECORD_TREE_IN_THE_TREE_TABLE_OF_THE_NEWEST_ROOT,」；
有条目而没有种类 3 的那一支 `crates/singlefs-checker/src/walk.rs:10778`「allocation_record_tree_entries: 0,」报部分没判）与 kb 条文一致：「最新根的树表 0 条（mkfs 的第 0 代、照抄它的暖机根、树表 0 条写过行的那一版）没有对象、报不适用」（`crates/singlefs-checker/invariants/05-空间记账.md:28`）。
判决原文写的是「前置是最新根的树表带种类 3 记录树且读全，不满足报这一条判不了」（`research/prompts/m3-close-b19-reason-r2-main-verification.md:22`），把三种不满足并成一个「判不了」。实现把它拆成「无从判」与「判不了」两个口径，拆法有理由：
0 条的那几版没有种类 3 的条目，也就没有可比的记录，「判不了」在这里会让每张 mkfs 之后的镜像都带一条部分没判，把部分没判的计数冲掉意义。拆开之后部分没判只留给真读不出与畸形。更正站得住。

真病会不会藏在 0 条版本里、又没有别的不变量罩着？
- 这几版没有记录可比，I-3.21 问的「起点有没有自己的记录」在它们身上是空问题。它们的单元（树表单元、实例表单元）落点互不重叠由 I-5.1 判，从 mkfs 水位建树那一版的 t1 落点由 I-5.5 判（编号与射程我只读了 kb 表头，没逐条核它们对 0 条版本的射程）。这是推的，没造盘面。
- 一个容易混的口子：`allocation_records_of_a_root_record_without_judging` 对「树表指针整条全零」也交 `tree_table_entries: 0`（`crates/singlefs-checker/src/walk.rs:10331`「if tree_table_pointer.all_zero {」），I-3.21 会把这一形一并报不适用。但判决列的三个合法来源里没有它：mkfs 第 0 版等三形的树表指针非零。
  全零指针在别处是红的：「这一版的走读记一次失败（最新根红 I-7.2、I-4.8、I-7.4」（`crates/singlefs-checker/src/walk.rs:2698`，函数 `fn follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(`，`crates/singlefs-checker/src/walk.rs:2701`）。
  所以这一形不会因为 I-3.21 报不适用而放过。建议 kb 条文的「不适用」括注补一句「指针全零不在其内，那一形由走读失败判红」，免得读的人以为它被豁免。这是建议，不是实现缺陷。
推翻它的现象：一张树表 0 条、但 walk 记到非豁免单元引用（`walk.references` 里有码 1、2、3 的引用）的镜像，I-3.1、I-5.1、I-5.5 都没报。

### Q2 第 2 问　「未释放或释放代高于 θ_high」这一记录集合

**措辞更正站得住，而且比正文列的多一处。** 判决写的是「都是最新根 E(θ_high) 里某条未释放记录的起点」（`research/prompts/m3-close-b19-reason-r2-main-verification.md:22`）。
前半句 E(θ_high) 本来包含「已释放而释放代高于 θ_high」的记录（`crates/singlefs-checker/invariants/05-空间记账.md:8` 的定义），后半句「未释放记录」把它窄回去了。字面读法会在每一段有覆盖写的合法历史上误红：
- 覆盖写换下的旧数据单元，被更早的候选根引用着；它在最新根的账里已释放，释放代 g 大于那条候选根的 txg。
- 门槛 θ_high 是 `oldest_valid_root_in_the_ring.max(effective_rollback_floor)`（`crates/singlefs-checker/src/walk.rs:9826`），候选根都是有效根且 txg 不低于 F，所以 θ_high 不高于任何一条候选根的 txg；g > 候选根 txg ≥ θ_high，这条记录在 E(θ_high) 里，但它不是未释放记录。
- kb 条文正是这么写的：「只被更早候选根走到的单元在最新根里已释放、释放代高于走到它的那条候选根的 txg，而 θ_high 不高于任何一条候选根的 txg」（`crates/singlefs-checker/invariants/05-空间记账.md:28`）。
实现的过滤是 `!record.is_released || record.generation > reclaim_threshold_computed_by_the_checker`（`crates/singlefs-checker/src/walk.rs:10790`），与 kb 条文相符，与判决字面不符；以 kb 条文与实现为准。

合法镜像误红：**造不出（推的）。** 在 I-3.18 绿、I-5.1/I-5.4 绿的镜像上，I-3.1 的逐槽下沿 (P)「遍历走到的每一槽都有 E(θ_high) 里的记录罩着」若成立，罩着它的记录必须以这一槽起头：
否则两个单元重叠（I-5.1；被罩的是没人引用的单元时归 I-3.15）或一条记录比它的单元宽（I-3.18）。所以在这三条绿时 (P) 蕴含 I-3.21；I-3.21 只在 (P) 本身红、或有宽记录罩住邻居时多红。合法写者写不出后者。
两处需要合入后验证、本腿核不了：（一）I-3.21 不看 `accounting_seen`（`crates/singlefs-checker/src/walk.rs:21639`「judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root(」那一次调用前没有 `if accounting_seen`），而 (P) 所在的 I-3.1 读记录要过这一道门（「let records_of_the_newest_root_read_in_full = if accounting_seen {」，`crates/singlefs-checker/src/walk.rs:21648`）；崩溃点重放里有没有「走到引用、却没有记账行」的合法崩溃态，要看崩溃点重放快档的结果；
（二）「由记录施加出来的那几版」新分配的单元，kb 条文说它们有起点对齐的记录（`crates/singlefs-checker/invariants/05-空间记账.md:28` 的「已知边角」），两腿「逐种推的」，没造合法盘面量。
这两处正文自己也写了「崩溃点重放快档里的崩溃态上不红由合入后验证判」，我同意：待主 agent 跑。

坏镜像漏判：**造不出（推的）。** 攻方修正模型上的 1195 个 C944 形（`research/prompts/m3-close-b19-reason-r2-main-verification.md:22` 的「holds_c944_nothing_else_red=1195」）共同点是：走到的单元起点没有 E(θ_high) 里自己的记录。
I-3.21 对每个走到的 (盘, 起点槽) 查「是否在 E(θ_high) 记录的起点集合里」，这个集合不含它就红。邻居记录被拿掉、或它的记录已释放而释放代不高于 θ_high（可回收），都在红的一边。
一处我没法排除：邻居的自己的记录在，起点对，但记录跨度错——那是 I-3.18 的事，I-3.21 不管。这不是漏。
推翻它的现象：1195 个形里有一个，在今天的实现上跑 I-3.21 不红。（待主 agent 复跑攻方的扫描，本腿没拿到那份模型。）

### Q2 第 3 问　「有条目而没有种类 3」「树表读不出」报部分没判会不会让这一条形同虚设

**两形合法写者写不出；比例不会因它们变大（推的）。**
- 有条目而没有种类 3：kb 把它派给 I-9.16 ④ 判畸形（`crates/singlefs-checker/invariants/05-空间记账.md:28`「树表非空而一条种类 3 的条目都没有（I-9.16 ④ 判那块畸形）」）。合法写者每次发布都重写记账树（`crates/singlefs-checker/src/walk.rs:9830` 一带的说明：带单元的发布每次都重写记账树），带树的版本一定有种类 3。
- 树表读不出：只在读故障下发生；故障注入的一次性读错有重读收，持续读坏的盘不是合法历史。
真正会让这一条「看上去形同虚设」的是 0 条的那一类 n/a：mkfs 之后到第一个文件版本之前的几个根，在崩溃点重放里按状态数不一定少（流起点是 mkfs 的那几族）。这是覆盖稀释，不是部分没判比例问题；比例没量，要看崩溃点重放各族里最新根树表为 0 条的状态占多少。
本腿没有这个数：**没量**，不写成已知。推翻它的现象：某一族崩溃状态里这两形的部分没判占了大头。

## Q3　第 0 代根两处与 I-7.9 的上限

### Q3 第 1 问　ABSENT 加「槽 0 而 F 为 0 不记空着」：第二轮数能不能复核、有没有合法环从部分没判变红

**今天的实现与第二轮辩方 `today_both_fixed` 那一档同语义（读代码对出来的）；数是冻结输出里的，复跑待主 agent。**

对照（Rust 在左，辩方端口在右）：
- Rust：槽 0 且抬之前 F 为 0 时只记第 0 代根，别的槽记空着（`crates/singlefs-checker/src/walk.rs:9029`「&& floor_before_the_raise == GENESIS_CHECKPOINT_TXG」；非槽 0 的 `possible_roots.push(None)`），第 0 代根的树表按 `UserVisibleTreeRootPointers::ABSENT` 记（已知的空树状态）；窗口外那一停仍在（`crates/singlefs-checker/src/walk.rs:9044`「if overwritten_txg < floor_before_the_raise {」）；每一槽的盘按它所在的区域认（`crates/singlefs-checker/src/walk.rs:9007`「let device = geometry.region_devices[usize::try_from(region)」）；镜像上整槽全零交一种空着的可能（`crates/singlefs-checker/src/walk.rs:9013`「RingSlotOfTheImageSeenFromTheRaise::AllZero => Ok(vec![None]),」）。
- 端口：`'today_both_fixed': dict(none_with_genesis=False, genesis_state=0, window_out_stop=True),`（`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/ding_today.py:28`）：只记第 0 代根（`none_with_genesis=False`）、第 0 代根状态已知（`genesis_state=0`）、窗口外一停（`window_out_stop=True`）。三个开关与上面三句一一对上。
所以辩方在这一档上量的数就是今天实现的数，条件是端口忠于 Rust（我逐支读过，没发现不同；没拿端口去对拍 Rust）。

冻结输出里的数（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/`，J = 128，各 20000 段）：
- 坏写者 S = 4：`today` 档「final[holds=0 red=13225 partial=838」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:1`），`today_both_fixed` 档「final[holds=0 red=13232 partial=831」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-4-128.txt:5`）：红 +7。
- 坏写者 S = 8：「final[holds=0 red=22765 partial=333」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:1`）到「final[holds=0 red=22768 partial=330」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-bad-8-128.txt:5`）：红 +3。
- 合法写者 S = 4：部分没判「final[holds=13922 red=0 partial=1348」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:1`）到「final[holds=13977 red=0 partial=1293」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:5`）：降 55。
- 合法写者 S = 8：「final[holds=24402 red=0 partial=976」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:1`）到「final[holds=24774 red=0 partial=604」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:5`）：降 372。判决写的 370（`research/prompts/m3-close-b19-reason-r2-main-verification.md:25`）是从 `today_genesis_only` 档的 974 量起的（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-128.txt:4`），不是从 `today` 档；两个基线不同，差 2，不改结论。
  （上面的 `out/…` 都是 `research/prompts/m3-close-b19-reason-r2-sonnet-model/out/…`。）
- 合法红恒为 0：六档、四格（S = 4 / 8，J = 128 / 32）里 `legal` 的 `red=0`，J = 32 也一样（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:1`「final[holds=13627 red=0 partial=1643」到 `research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-32.txt:5`「final[holds=13681 red=0 partial=1589」；S = 8：`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:1`「final[holds=16082 red=0 partial=9296」到 `research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-8-32.txt:5`「final[holds=16351 red=0 partial=9027」）。

**有没有合法环因此从部分没判变红：造不出（推的，模型上）。** 两处改法都只把「空着」换成「第 0 代根」：第 0 代根的树表是 0 条，按 ABSENT 记；它参与上限式子时可以把上限压到 0。
合法写者的上限式子里同样有这条第 0 代根（写者读环读到它，`crates/singlefs-core/src/mount.rs:4024` 起按槽的区域认盘），所以写者抬到的 F 不高于它，checker 的上限与写者对齐，合法写者不会红。
冻结输出里合法红恒为 0 支持这一点。

缺的一格：端口的「根槽写失败而本实例接着发」档（`p_rootfail = 0.03`，今天的写者走不到）只跑了 `today` 与 `today_no_stop` 两档：「final[holds=13446 red=216」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-4-128.txt:1`）与「final[holds=24077 red=246」（`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-rootfail-8-128.txt:1`）。
`today_both_fixed` 在那一档上没跑，那一档里红的 216 / 246 条在改法之后变多还是变少没有数。它不是合法写者能写出的历史，所以不挡结论，但「有没有合法环变红」这句话的覆盖要写明不含这一档。已写进「要主 agent 跑的命令」。

### Q3 第 2 问　实现员「I-7.9 的上限不用跟 D16 新条款改」的理由

**站得住（推的）。** 理由的骨架：D16 新条款是写者的**保守**规则（读回全零的槽「当作可能是一条非空有效根」，F 不越过它：`.claude/kb/decisions/16-发布语义.md:38`「读坏槽与读回全零的槽都并进上限」），
checker 只能看镜像：镜像上整槽全零就是没有持久根（`crates/singlefs-checker/invariants/02-根环.md:16`「全零的槽那一刻空着」）。

两个方向各核一遍：
1. **合法历史让 checker 判红**：造不出。合法写者把全零槽并进上限，写者的上限 = min(c, h)（c 是读得出的根算的上限，h 是那些槽能住的最大 txg，`crates/singlefs-core/src/mount.rs:4418`「(ring_slot, txgs_it_can_hold.highest_txg_it_can_hold())」）；
   checker 的上限 = c'（全零槽不进）。合法历史里 c' = c，所以写者抬到的 F ≤ min(c, h) ≤ c = c'。写者比 checker 严，checker 不会因为这一点红。
2. **坏写者越过 D16 字面上限而 checker 放过**：字面上存在（F 在 (min(c,h), c'] 里 checker 不红），但我造不出一段**有害**的：
   - 该槽真空：F 越过一个不存在的根，没有候选被挡，不是病。
   - 该槽读回全零是假象（设备说谎或一次性读错）、盘上其实有根：镜像再读时根在槽里，checker 把它算进环里，F 越过它就红 I-7.9；
     如果之后这个进程把那一槽盖掉，checker 走 `OverwrittenSinceTheRaise` 那条路用 journal 认抬 F 那一刻住着的根（`crates/singlefs-checker/src/walk.rs:9044` 往上的逐层认），仍算进环里。
   所以 checker 看得见的有害形都在，只有「槽真空、F 越过它」这种无害形被放过。
一处残留：journal 里没有那一层带提交标记的记录（转出 journal 环了）时 checker 报「环重建不出」、部分没判，这时真有害的形也只是没判（`crates/singlefs-checker/src/walk.rs:9044` 附近的 `Err`）。这是丁一向的缺口，不是这一批带来的。
推翻它的现象：一张镜像，写者读回全零并过的槽上盘里有根、F 抬过它，而 I-7.9 既不红也不报「环重建不出」。

## Q4　代码轮：写者一侧做的是不是 D16 已定项 1 与 C953 说的

条款原文，先摘本问要对的几句（`.claude/kb/decisions/16-发布语义.md` 现查的行号）：
- 第 38 行（抬 F 的上限）：「读坏槽与读回全零的槽都并进上限：挂载那一刻读回全零、之后这个进程没写过的槽，同读坏的槽一样当作可能是一条非空有效根」（`.claude/kb/decisions/16-发布语义.md:38`）；
  「推一次不抬 F 的空发布（带现行 F、txg 加一，算进这次准入的已推次数」（同一行）；「预算放不下这一次，或这一次取不到落点，就拒这次请求，下一次请求从零预算接着推」（`.claude/kb/decisions/16-发布语义.md:38`）。
- 第 39 行（根槽这一次读坏）：「读坏的与读回全零的都并进准入抬 F 的上限，上限卡在它们上时推空发布」（`.claude/kb/decisions/16-发布语义.md:39`）；「重读读回全零照旧钉着」（同一行）。
- 第 44 行（准入）：「一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8；上限卡在读坏槽或读回全零的槽上推的空发布也算在内」（`.claude/kb/decisions/16-发布语义.md:44`）。

### Q4 第 1 问　全零并

**一致（逐句对过）。**
- 并的是哪类槽：`ring_slots_bad_or_reading_back_all_zero_at_mount` 把 `BadAtMount(txgs_it_can_hold)` 与 `AllZeroAtMount { txgs_it_can_hold }` 两类取在一起（`crates/singlefs-core/src/allocator.rs:1694`「| RootRingOccupant::AllZeroAtMount { txgs_it_can_hold } => {」，函数 `crates/singlefs-core/src/allocator.rs:1687`），准入取这一份（`crates/singlefs-core/src/mount.rs:4407`「pub fn ring_slots_bad_or_reading_back_all_zero_at_mount_by(」）。
  这与第 38 行「挂载那一刻读回全零、之后这个进程没写过的槽」对上：「之后这个进程没写过」由根环表做到——进程内记一条根时，那一槽被换成 `ValidRoot`（`crates/singlefs-core/src/allocator.rs:2797`「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」），不再在两类之内。
- 「这一遍又读坏」：算上限那一遍读到全零，在 `read_root_ring_slot` 里归 `Bad`（`crates/singlefs-core/src/recovery.rs:1764`「None => RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),」：全零解析不出根），所以全零槽进「这一遍读坏、当没有根」那一集（`crates/singlefs-core/src/mount.rs:3815`「let mut ring_slots_bad_on_this_read_taken_as_holding_no_root = BTreeSet::new();」），
  再按 `.filter_map(|ring_slot| ring_slots_bad_since_the_mount.get(ring_slot))`（`crates/singlefs-core/src/mount.rs:4086`）取挂载时记的最大 txg，滤掉低于现行 F 的（`crates/singlefs-core/src/mount.rs:4088`「.filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)」）。
- 能住的最大 txg 记得对不对：读回全零的槽与读坏的槽走同一个函数、同一个窗口锚、同一个 `first_txg`（`crates/singlefs-core/src/mount.rs:3387`「let ring_slots_reading_back_all_zero_at_mount = txgs_ring_slots_bad_at_mount_can_hold(」），
  所以最大 txg 的取法就是第 38 行说的「同读坏的槽（「环里最旧有效根」那一行「挂载时读不出的槽」那一句）」：N 取读得出的最新有效根，区间 (N − 3S, first_txg − 1]。我没有另做算术核这个函数本身（它对读坏槽早已有单测在 `mount.rs` 的 `txgs_ring_slots_bad_at_mount_can_hold_tests`）。
- 重读读回全零：`AllZero` 的重读结局什么都不改（`crates/singlefs-core/src/allocator.rs:2945`「| RereadOfARootRingSlotBadAtMount::AllZero => {}」），照旧钉着、照旧并，与第 39 行「重读读回全零照旧钉着」对上。
- 回收门槛有没有被一起改动：**没有。** 补丁前读回全零的槽经同一个 `txgs_ring_slots_bad_at_mount_can_hold` 算、再取 `oldest_txg_it_can_have_held()` 存成单个 txg；补丁后改成存整个 `TxgsARingSlotBadAtMountCanHold`、回收门槛处取 `oldest_txg_it_can_have_held()`（`crates/singlefs-core/src/mount.rs:3440`「.map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),」）。同一个函数、同一个取值，值不变。
  `git diff HEAD -- crates/singlefs-core/src/mount.rs` 里被删的那几行是 `.map(|(ring_slot, txgs_it_can_hold)| { (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held()) })`，新增的是不取 `.map` 直接留下整个值，前后对照一致。
推翻它的现象：同一段历史上补丁前后 `reclaim_floor` 不同；或读回全零的槽并进上限时取的不是能住的最大 txg。

### Q4 第 2 问　推空

**一致（逐句对过），次序没有错。**
- 「已推 + 1 + 1 ≤ 8」是不是 B 的意思：是。第 44 行 B = 8 数的是这次准入的发布数，含「推的空发布」与「这次准入为之判的那一次发布」（那一次也算在 8 里）。代码 `publishes_pushed + 1 + 1 > PUBLISHES_PER_ADMISSION_AT_MOST;`（`crates/singlefs-core/src/mount.rs:4810`，常量 `crates/singlefs-core/src/mount.rs:4540`「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」），
  抬 F 那一串用的是同一个形：`publishes_pushed + publishes_of_the_next_raise + 1`（`crates/singlefs-core/src/mount.rs:4811` 的 `a_raise_exceeds_the_budget`）。推空是一串长 1 的抬 F，两条式子同形。推空计入 B：调用方每推成一次都把 `raised.publishes.len()` 加进已推数（`crates/singlefs-core/src/mount.rs:8592`「publishes_pushed += raised.publishes.len();」；挂载续做 `crates/singlefs-core/src/mount.rs:8052`；会话那一处同形）。
- 「卡在这两类槽上」：`self.not_passing_the_ring_slots_bad_since_the_mount <= floor`（`crates/singlefs-core/src/mount.rs:3975`）且 `&& self.from_the_roots_read > floor`（`crates/singlefs-core/src/mount.rs:3976`），与第 38 行括注「并进它们之后上限不高于现行 F，不并它们时上限高于现行 F」逐字对上。
- 推空发布带现行 F、txg 加一：计划取 `empty_publish_plan_after_file_version(current, current.record.instance)`（`crates/singlefs-core/src/mount.rs:4899`），其中 txg 取 `checkpoint_txg_of_the_planned_publish_after(current_file_version.root.checkpoint_txg)`（`crates/singlefs-core/src/mount.rs:5610`，加一），F 取 `rollback_floor: current_file_version.root.rollback_floor,`（`crates/singlefs-core/src/mount.rs:5624`），
  发完还断言带的 F 等于现行 F（`crates/singlefs-core/src/mount.rs:4912`「不抬 F 的空发布带的 F 就是现行那一版的 F」）。
- 是不是 E173 登记「推空B」的定义：对。E173 页的臂定义「推空B 在上限卡在读坏槽上（C并 ≤ F 而 C0 > F）时推一次不抬 F 的空发布、算进已推次数」（`.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:359`）。
  原登记（已归档，现在不在工作区）的行更全，我从历史里取出核了：`git show 04ceaa904^:research/prompts/e173-preregistration.md | grep -n '推空B'`，第 228 行原样是：
  `| 推空B | 上限取 C并；C并 > F 照今天推；C并 ≤ F 而 C0 > F（卡在读坏槽上）时推一次不抬 F 的空发布（带现行 F、txg 加一，算进这次准入的已推次数），回到判够不够；预算放不下这一次（已推 + 1 + 1 > B）或取不到落点就拒。下一次请求从零预算接着推 | …`。
  「带现行 F、txg 加一」「已推 + 1 + 1 > B」都与代码一致；「回到判够不够」由调用方的循环做（下一条）。
- 推完写掉那一槽之后照抬 F 那一串的次序：三处调用方都是每一轮重新调一次 `push_one_floor_raise_within_the_admission_budget`（可写挂载写行之后 `crates/singlefs-core/src/mount.rs:8592`；挂载续做 `crates/singlefs-core/src/mount.rs:8052`；会话 `crates/singlefs-core/src/mounted_session.rs:4619`），
  该函数每次先判预算（`crates/singlefs-core/src/mount.rs:4813`「if an_empty_publish_keeping_the_floor_exceeds_the_budget {」），再重算两个上限（`crates/singlefs-core/src/mount.rs:4818`），卡着就推空、没卡且放得下就抬 F（`crates/singlefs-core/src/mount.rs:4844`「} else if !a_raise_exceeds_the_budget {」）。
  写掉那一槽之后，那一槽在根环表里换成 `ValidRoot`，`ring_slots_bad_since_the_mount` 里没有它，下一轮 `from_the_roots_read == not_passing…`，`are_held` 为假，走抬 F 那一支。次序对。
  测试 `admission_pushes_empty_publishes_keeping_the_floor_until_the_slot_holding_the_ceiling_is_written_then_raises_the_floor`（`crates/singlefs-harness/tests/admission_pushes_an_empty_publish_keeping_the_floor_while_the_ceiling_is_held_by_a_slot_unreadable_since_the_mount.rs:171`）就是照这个次序断言的；我没有跑它。
一处注意（不是错）：预算判在算上限之前只有「连一次空发布都放不下」这一支；「放得下一次空发布、放不下抬 F 那一串」时，现在会先读一遍根环算上限再报预算用完（补丁前这一支在读盘之前就返回，`git show HEAD:crates/singlefs-core/src/mount.rs` 里 `publishes_of_this_admission > PUBLISHES_PER_ADMISSION_AT_MOST` 紧跟着 `return`）。
这时算上限的读盘错（`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread` 等）会盖过「预算用完」。函数文档写了这一取舍（`crates/singlefs-core/src/mount.rs:4755` 一带「放得下一次而放不下抬 F 那一串时算上限」），所以是照设计，只提醒读报错的人。
推翻它的现象：一次准入里推空超过 7 次（含为之判的那一次超过 8 次）；或推空发布带的 F 与现行 F 不同；或写掉那一槽之后下一轮仍推空。

### Q4 第 3 问　按槽认盘：别的读环路径还有没有按 txg 认盘、真写者上走不走得到

**别的读环路径：除一处外都按槽认盘；那一处在真写者上走不到（推的）。**
- 按槽认盘的：算上限（`crates/singlefs-core/src/mount.rs:4023`「// 按根所在那一槽的区域认盘，不按落点公式从 txg 推：第 0 代根三份 txg 都是 0，各在各的区域里。」）；
  重建分配器与判每块盘是否都读出了有效根（`crates/singlefs-core/src/mount.rs:3237`「fn roots_on_the_devices_they_were_read_from(」，文档第 3233 行「不按 txg 的落点公式」）；
  F_生效 的算法（`crates/singlefs-core/src/recovery.rs:2438`「let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];」）；挂着时回退判候选读根环走同一族读函数（`crates/singlefs-core/src/recovery.rs:1745` 的 `read_root_ring_slot` 按槽的区域取盘）。
- 还按 txg 认盘的一处：卸载抬 F 那一格当场放开的那一截，`roots_on_the_devices_of_their_txgs` 用 `device_of_txg(root.checkpoint_txg)` 认盘（`crates/singlefs-core/src/mount.rs:5275`「.map(|root| (device_of_txg(root.checkpoint_txg), *root))」）。
  与按槽认盘只在第 0 代根上不同；那一格的文档自己写了卸载接在可写挂载之后、暖机先在每块盘上落了新实例的根（`crates/singlefs-core/src/mount.rs:4255`「卸载那一格滤不滤结局相同：卸载接在一次可写挂载之后」），每块盘最新的根都不是第 0 代根，所以取每盘最新时第 0 代根归哪块盘不改结果。**走不到不同结果**，但这是代码里残留的一处按 txg 认盘，下一次改这一带时顺手改成按槽更干净。
- 真写者上按槽认盘走得到吗：**走不到（推的）**，论证：
  1. 准入推抬 F 的入口三处都在写行与暖机之后：可写挂载先 `for planned_txg in &warm_up_publishes_planned {`（`crates/singlefs-core/src/mount.rs:8342`），后 `push_floor_raises_after_the_row_publish(`（`crates/singlefs-core/src/mount.rs:8415`）；挂载续做、会话在这之后。
  2. 暖机按落点公式推到每块盘都有本实例的根为止（区域 r 的槽 0 被 txg r 盖掉：txg 1 盖区域 1 槽 0，txg 2 盖区域 2 槽 0），到算上限时区域 1、区域 2 槽 0 里的第 0 代根已被盖掉，只剩区域 0 槽 0 的那一份，而它按槽与按 txg 认盘同归区域 0。两种认法在已盖掉的第 0 代根上无从分歧。
  3. 要让它走到，需要「区域 1 或 2 的槽 0 在算上限时仍是第 0 代根」，也就是 txg 1 或 2 的根槽写没落盘而写者继续发。今天的写者落盘失败就冻结原样重发（「今天的写者落盘失败就冻结、原样重发」，`crates/singlefs-checker/invariants/02-根环.md:16`，这是 I-7.9 行自己的前提句，我没有另核写者代码），所以只有说谎的设备（吞写）才造得出，那时按槽认盘是 D16 字面「每块盘上最新的持久有效根」要的，上限落 0。
  4. 公开入口 `rollback_floor_ceiling` 收调用方拼的分配器与盘面，这一形在测试里直接钉：`crates/singlefs-harness/tests/admission_floor_ceiling_counts_each_genesis_root_on_the_device_of_its_ring_slot.rs`（攻方那段历史，期望上限 0），没有跑。
推翻它的现象：一段合法历史（无说谎设备）里，算上限时区域 1 或 2 的槽 0 里还是第 0 代根。

### Q4 第 4 问　理想模型不认读坏槽、全零槽与推空

**随机历史对拍不会因此假红；故障注入的「被吞的根槽写」解释口径有一处可能假红（推的，待跑）。要不要同批补模型：不必同批，但要记账。**
- 对拍：理想模型的环里只有读得出的根（`crates/singlefs-harness/src/model.rs:3943`「.entry(device_holding_the_root_of(root.key.checkpoint_txg))」按 txg 认盘），模型里第 0 代根只有一份、在区域 0（真池 mkfs 写三份）。
  不注入读故障的随机历史没有挂载时读坏或读回全零的「可能住过根」的槽（落点公式在 [0, first_txg) 里能落到的槽，写者都写落过），所以实现的上限与模型的上限同值，`reported_ceiling` 对拍不会因新规则报假红。
  暖机之后每块盘都有本实例的根（Q4 第 3 问），模型一块盘上没有有效根时不算那块盘（文档「一块盘上没有有效根时不算它」，`crates/singlefs-harness/src/model.rs:3923`；式子「let newest_on_every_device = newest_valid_per_device.values().copied().min()?;」，`crates/singlefs-harness/src/model.rs:3947`）这一格也不影响。
- 故障注入的缝：说谎设备吞掉根槽写时，harness 用「去掉被吞的根重算的模型上限」去解释实现报的上限：要 `reported == recomputed` 才算被解释（`crates/singlefs-harness/src/fault_injection.rs:2705`「(Some(reported), Some(Some(recomputed))) if reported == recomputed」，重算在 `crates/singlefs-harness/src/model.rs:3985`「pub fn rollback_floor_ceiling_without_the_roots(」）。
  补丁之后：首圈里被吞的根槽写让那一槽保持全零，之后重挂载（新进程）读到「挂载时读回全零」，写者把它按能住的最大 txg 并进上限，实现报的上限变成 min(c, h)；模型只去掉那条根，上限仍是 c。h 低于 c 时两者不等，解释口径判「没解释」。
  需要的条件：被吞的是首圈（环还没转满）里的根槽写、重挂载、之后再抬 F 或推空、并且被吞的根比第 4 新的非空根还旧。随机故障历史里这一串是否出现过，我没有核：**待主 agent 跑**（下一节第 3 条）。
- 要不要补模型：推空与读坏、全零并进上限是实现专有的「写者保守」规则，理想模型本来不建模（模型的上限等于理想算法）。补进模型就等于把实现的保守规则抄一份，对拍变成自己对自己。
  比较合适的补法是不改理想模型，而改解释口径：被吞的根槽写落在首圈时，解释口径除了去掉那条根，还要允许上限低到该槽能住的最大 txg。这不必同批做，但这一批该在 checks-owed 记一笔。
推翻它的现象：一次不注入读故障的随机历史里，实现报的上限与模型不等，且挂载时根环表里有读坏或读回全零的槽。

## Q5　两份装置的计数分配器

两份的 unsafe 数：E183 九处，行是 207、208、210、217、219、226、228、232、234（`unsafe impl` 一处，四个 `unsafe fn` 各一处，四处函数体里的 `unsafe { System.… }` 各一处）；
E197 九处，行是 140、141、143、150、152、159、161、165、167，同形。`grep -n unsafe` 两份文件的输出就是这些行。

### Q5 第 1 问　量的是什么、离不离得开、替代与口径

**E183 离不开；E197 的判的数离得开、登记里的互核与停机条款离不开（推的）。**

E183：量的是单次调用在发起量窗的那条线程里分出去的 `Layout` 字节。例：`measure_in_a_window(&COUNTING_ALLOCATOR, || mounted.allocator.clone());`（`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:3371`），读数 `copy_bytes`（`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:3373`「let copy_bytes = copy_window」）。
登记在第 2 次重跑里把量窗收成线程本地：「计数分配器在全进程原子记账之外，每条线程另记自己分出、放掉的字节（线程本地）」（`research/prompts/e183-r2-prereg.md:384`）。一次 `clone()` 的字节是 KB 到 MB 级、且要排除并行用例的串入，页粒度、进程粒度的计数给不出这个量。

E197：判的数是统一任务服务记的 cgroup 峰值，分配器不判：「进程内的分配器峰值与 `VmHWM` 只用来互核，不判」（`.claude/kb/experiments/197-扫环胶水内存随盘数.md:35`）。但登记里有四处读分配器：
A7（推算值对分配器峰值 ±5%）、A8（measurement_pc 的分配器峰值落 [268 435 456, 268 500 992]：「A8：measurement_pc 格分配器峰值减起点落在 [268 435 456, 268 500 992]」，`research/prompts/e197-preregistration.md:218`）、
A9（split 与 baseline 在 1 盘上分配器峰值差 ≤ 1%：`research/prompts/e197-preregistration.md:224`），以及停机条款 S3「A7 出 ±5%；A9 出界；或两条量法互不包含」（`research/prompts/e197-preregistration.md:292`）。
另外每块盘最后一槽被读到那一刻的分配器在用字节是轨迹的来源（`research/prompts/e197-preregistration.md:229`）。所以 E197 的结论（随盘数怎么涨）不依赖分配器，它的作废与停机依赖。

不用 unsafe 的替代，逐个对照：

| 替代 | 能不能办到 | 对登记里的量改不改口径 | 代价 |
|---|---|---|---|
| 读 `/proc/self/status` 的 VmHWM / VmRSS（两份装置已经在读：`crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:1054`「let status = std::fs::read_to_string("/proc/self/status").ok()?;」） | E197：能。峰值能读，进到每块盘的轨迹可在盘边界读 VmRSS；窗头的基线可写 `/proc/self/clear_refs` 重置 VmHWM（内核文档的做法，本机没试）。E183：不能：KB–MB 级的一次 clone 淹没在页粒度与 malloc 的 arena 复用里，还是进程粒度，12.2 要的线程本地做不到 | E197：改。A7 的式子是 `Layout` 字节加 B 树每叶按 6 键摊的口径，RSS 含 malloc 块头、页取整、没写到的页不算，±5% 要重标；A8 的区间、A9 的 1%、S3 的两条量法互不包含都要重新写。E183：量的定义（X7、X8、E30 与 12.2）整个换，等于另一个量 | E197 的产物一份都没出（kb 页状态「装置已写、产物未跑」），现在改登记的成本最低；E183 的第一段、第三段已有产物，改量法要对已判的读数交代 |
| cgroup `memory.peak`（服务记） | 只有进程级峰值，已是 E197 判的数。给不出窗内、线程内的量 | E183 的窗口量不能用它顶 | 对 E197 没有新增；对 E183 无用 |
| 换一个已有的安全计数库（`stats_alloc`、`peak_alloc`、`dhat`、`cap` 这类） | 此刻办不到：`Cargo.lock` 里不在（`grep -n '^name = "\(peak_alloc\|stats_alloc\|cap\|dhat\)"' Cargo.lock` 零命中），本机 cargo 缓存里也不在（`ls ~/.cargo/registry/cache/*/` 过滤同名零命中）。就算加进来：它们是全进程口径，12.2 与 E197 的单测各在自己线程里量的口径做不到 | 全进程口径与登记的线程本地口径不同 | 要联网加依赖、改锁文件；unsafe 只是挪进依赖里，门禁数的是 `crates/` 下的源码，所以门禁是绿的，这一条不能当作「没有 unsafe」的理由写进理由栏 |
| 自己写线程本地计数、但用安全封装 | 办不到：`GlobalAlloc` 是 unsafe trait，在本仓代码里写 `impl GlobalAlloc` 就必须 `unsafe impl`，它的四个方法是 `unsafe fn` 而且要调 `System` 的 unsafe 方法。只要想在分配调用上挂钩，这九处就是下限 | — | — |
| 两份合并成一个共用的小 crate（`unsafe` 只在一处，gate 登记一组行，两份装置引用它） | 办得到 | 不改口径（E183 的全进程原子 + 线程本地，E197 只要线程本地，小 crate 都给得出） | 新 crate 要进登记表（`crates/` 下出现登记表里没有的 crate 判红，`.claude/gate.d/unsafe-allowances.tsv:21`），根写 deny；两份登记里引的 `e183…rs:150-206` 一类行号要改；已出产物的二进制被钉的 sha256 不再对得上重编的 |

这一问的结论分开写：
- E183：造不出替代（推的，挡住它的是 `GlobalAlloc` 的 unsafe 契约与登记要的线程本地口径）。
- E197：判的数能不用分配器（它已经是 cgroup 峰值）；登记的互核与停机条款要改口径才能不用。这是**能换、但要先改登记**，不是「离不开」。

### Q5 第 2 问　登记几行、理由与出处怎么写才站得住

先说门禁今天怎么数，因为它决定行数。登记表头写的是一处放开一行、每个放开的函数一行：「4 放开 unsafe 的函数名；没有放开写 -；同一个 crate 里一个函数只登一行」（`.claude/gate.d/unsafe-allowances.tsv:8`），
第 5 段是函数里 unsafe 关键字几处，第 7 段理由「为什么非放开不可、不放开的替代为什么不行；放开了的行必填、不少于 20 个字」（`.claude/gate.d/unsafe-allowances.tsv:12`）。门禁认的单位是函数，而且「在函数体里」按字符位置判：函数体的起点是函数名之后第一个 `{`（`.claude/gate.d/code-source-discipline.sh:2605`「start = cleaned.find("{", declaration.end())」），一个 `unsafe` 关键字属于登记行，当且仅当它的位置落在这个区间里（`.claude/gate.d/code-source-discipline.sh:2640`「if path == row["file"] and span is not None and span[0] <= position < span[1]:」）；放开的 `allow` 要紧挨着一个 `fn`（`.claude/gate.d/code-source-discipline.sh:2594`「FUNCTION_AFTER_ATTRIBUTES = re.compile(」）。

按这个判法，今天**连四行都登不过**（推的，读门禁源码得到，没起门禁）：
1. 每份装置的九处 unsafe 里，只有四处函数体里的 `unsafe { System.… }` 落在 `{` 之后。另外五处——四个 `unsafe fn alloc(`、`alloc_zeroed(`、`dealloc(`、`realloc(` 签名里的 `unsafe`，和 `unsafe impl GlobalAlloc for CountingAllocator` 那一行——位置都在对应函数体的 `{` 之前或在任何函数之外，
   `registered_row_holding` 找不到持有它们的登记行，③ 判红；②（unsafe 关键字总数要等于各行第 5 段之和，`.claude/gate.d/code-source-discipline.sh:2767`「expected_unsafes = sum(row["count"] for row in allowance_rows)」）也对不上（实有 9，登记最多 4）。
2. `unsafe fn` 签名里的 `unsafe` 是 `GlobalAlloc` 方法签名要求的，去不掉；`unsafe impl` 同理。所以这不是登记表怎么写的问题，要工具员那一格改门禁：函数体的起点取到声明行（含 `unsafe`、`pub` 一类限定词），并让 `impl` 项也能登记（函数名栏写 `impl GlobalAlloc for CountingAllocator`、第 5 段 1）。
   改成这样之后的行数：每份 `alloc`、`alloc_zeroed`、`dealloc`、`realloc` 四行（每行第 5 段 2：签名一处加函数体一处）、`impl` 一行（第 5 段 1），**每份 5 行、两份 10 行**。这与「一处一行」同义，没有更少的合法写法。
   我没有改门禁的权限，也不替工具员定怎么改。
3. **根要改**：`singlefs-checker-tier` 今天是 `forbid`（`.claude/gate.d/unsafe-allowances.tsv:31`「singlefs-checker-tier	forbid」，根 `crates/singlefs-checker-tier/src/lib.rs:14`「#![forbid(unsafe_code)]」），根是 forbid 时 allow 与 unsafe 都必须 0 处（判法 `.claude/gate.d/unsafe-allowances.tsv:16`）。
   登记要把第 2 段改成 `forbid,deny`、lib.rs 改 `#![deny(unsafe_code)]`。`src/bin/` 每个文件是自己的 crate 根，lib.rs 的 forbid 本来就管不到它们，所以这个改动在编译器层面不减弱什么，在门禁层面是把这个 crate 从「0 处」换成「登记几处恰好几处」。
4. **SAFETY 注释要补**：门禁要求每个 `unsafe` 所在语句上方连着的注释里有 `// SAFETY:`（`.claude/gate.d/code-source-discipline.sh:2441`）。今天的 `// SAFETY:` 只在 `unsafe impl` 上方（E183 `…rs:206`「// SAFETY: 每个方法原样转给 `System`，只在成功之后改计数；计数是原子的」，E197 `…rs:139`「// SAFETY: 每个方法原样转给 `System`，只在成功之后改线程局部计数」）和函数体里的 `unsafe { }` 上方；
   四个 `unsafe fn` 那一行的上方是 `unsafe impl … {` 或空行，不连注释。`alloc_zeroed` 函数体里的注释只写「同 `alloc`。」，第 6 段若登记 `layout` 一类的词它就不含。所以两份各要补四条 `// SAFETY:`、把「同 `alloc`」写开。纯注释改动，但会让下面所有行的行号后移；panic 位置带行号，重编的二进制多半与已被钉的 sha256 对不上（推的，没重编）。
所以要动的有四样：门禁的函数体起点与 `impl` 项（工具员那一格）、根改 deny、补注释、写十行。

理由栏怎么写才站得住（≥ 20 字，写「非放开不可」与「替代为什么不行」）：
- 站得住的句子要有可核的事实：`GlobalAlloc` 是 unsafe trait，实现与调 `System` 都必须 unsafe，标准库没有安全的写法；
  替代在这个项目里逐个不行：页粒度的 VmHWM 给不出 E183 一次 clone 的字节、线程本地口径来自 E183 第 2 次重跑登记 12.2（`research/prompts/e183-r2-prereg.md:384`）；仓里 `Cargo.lock` 与本机缓存里没有安全计数库，库也是全进程口径。
- **站不住的一句：把 E197 与 E183 写成同一个理由。** E197 判的数是 cgroup 峰值，分配器只做互核与停机，登记那边已有「分配器峰值与 VmHWM 只用来互核，不判」（`.claude/kb/experiments/197-扫环胶水内存随盘数.md:35`）。
  理由栏若写「E197 离开分配器就量不了」，读的人对着这一行就能指出它不成立：量得了，只是 A7–A9、S3 要改口径。E197 的行要老实写成「保留是为了不改已定的互核与停机条款、产物未出之前不改登记」，这是成本理由，不是不可能理由；放开表接不接受成本理由是用户的事。
- 另一句要避开：「安全封装的库没有」。库没有是此刻的事实（上面的 grep），但库就算有，unsafe 也只是挪进依赖；写成「线程本地口径的库没有」才对。

出处栏：表头要「谁哪天定的」，且是仓里真有的路径或「用户 YYYY-MM-DD 弹窗定」加记录路径（`.claude/gate.d/unsafe-allowances.tsv:13`「8 出处：谁哪天定的；放开了的行必填，写仓里真有的文件路径」）。现有的 GPU 行写「用户 2026-11-03 弹窗定 records/2026-10-19-里程碑四GPU垃圾回收开工.md」。
本问的三方判决不是用户定案，所以出处要等用户那一次弹窗；判决文件 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 可以当路径，但那一行的「谁定的」必须是用户。我不替它写日期。

**摆给用户的三条路（没有倾向，代价如上表）：**（甲）登十行，改根、补注释、门禁认 `unsafe fn` 签名与 `impl` 项，E197 的理由写成成本理由；（乙）合成一个小 crate，登一组行；（丙）E197 先改登记不用分配器，E183 一份登四到五行。丙把需要放开的缩到一份，E197 登记要改。

## 要主 agent 跑的命令

本腿没写模型，没有 `<模型目录>/tools/rerun.sh`，没有 sha256 要报。下面是核第 3 问与第 4 问要的命令，一条都没有交；`task-server/task submit` 整条由主 agent 交。范围声明、内存、线程、台数都不写，由服务定。

1. **Q3 第 1 问：复跑第二轮辩方模型，核「红 +7 / +3、合法部分没判降 55 / 372」**（纯 python，九个并行段各 20000 段；是否算重型由主 agent 定，我标「较重」）。用的是第二轮留在仓里的模型目录与它自带的 `rerun.sh`（它会逐文件与 `out/` 里的冻结输出 `cmp`，一致打 `output_identical`）：
   `task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model -- bash tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-snapshot --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r3-sonnet-rerun-r2-sonnet`
   看哪几行：九行 `name=…_output_identical`，没有 `output_differs`；输出里 `today` 与 `today_both_fixed` 两档的 red / partial 与本报告第 3 节引的行相同。
   判决要引这些数，照 `.claude/rules/three-way-inference.md` 由主 agent 在草稿目录的副本里复跑。这条只复跑**端口**，端口对今天 Rust 的忠实度是我读代码得到的，没拿端口去对拍 Rust。
2. **Q3 第 1 问：补那一格——`p_rootfail = 0.03` 档上跑 `today_both_fixed`**（原来只跑了 `today` 与 `today_no_stop`）。纯 python，两个并行：
   `task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model/q3 -- python3 -B q3_today_run.py 20000 admission 4 128 0.03 today,today_both_fixed`
   `task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model/q3 -- python3 -B q3_today_run.py 20000 admission 8 128 0.03 today,today_both_fixed`
   看哪几行：`cfg=today` 一行与冻结输出里的 `red=216`（S = 4）、`red=246`（S = 8）相同（先证复跑是那份数据）；`cfg=today_both_fixed` 一行的 `red=` 与 `partial=`。这一档不是今天的写者能写出的历史，读数只用来回答「红有没有变多」。
3. **Q4 第 4 问：被吞的根槽写在首圈里 + 重挂载 + 再抬 F，实现的新上限与模型「去掉被吞的根」的上限会不会不等。** 命令我写不出：需要造一段这样的历史，现成测试里没有（`grep -rn ceiling_without_the_swallowed_roots crates` 只命中 `fault_injection.rs` 与快档夹具 `crates/singlefs-harness/tests/common_images_for_the_checker_tier/fault_injection_fast_tier.rs:785` 的 `None`）。
   最近的现成入口是故障注入快档，重型，标「重型」，只能证明补丁后已有的那些故障集没有新的未解释分歧，不能证明我说的那一形走得到：
   `task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-harness --test fault_injection_fast_tier`
   这一格写「待主 agent 跑」，需要时由实现员造那一段历史。
4. **Q4 的三个用例（我没有跑）**，重型：
   `task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-harness --test admission_pushes_an_empty_publish_keeping_the_floor_while_the_ceiling_is_held_by_a_slot_unreadable_since_the_mount --test admission_floor_ceiling_counts_each_genesis_root_on_the_device_of_its_ring_slot --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount`
   看哪几行：三个目标的 `test result: ok`，尤其攻方那段合法历史（期望上限 0）那一条。这是「合入后验证」的一部分，如果主 agent 已经在跑就不必重复。
5. **Q2 第 2 问：崩溃点重放快档上 I-3.21 不红**，正文自己写了归合入后验证，命令是 checker 档的（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 等），我不写，也不碰 GPU。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1.1 误红 | 造不出（推的） | 第五判只在 I-3.18 红的盘面里进；合法镜像 I-3.18 绿 |
| Q1.1 漏判（两条都已释放） | 造不出（推的） | 已释放记录在改记里原样不动，I-5.4 判任意两条 |
| Q1.1 措辞更正 | 站得住 | 判决字面「未释放对未释放」抓不到判决自己的隔离尾槽镜像；kb 条文与实现已按更准的读法 |
| Q1.2 与 I-5.4 | 重复、不矛盾 | 读法不同；只在改记读法下相交时 I-5.4 绿、I-3.18 与第五判红 |
| Q1.3 说明里的数 | 能分开（推的） | 改记后重叠走红的支，部分没判说明只在相交为空时写 |
| Q2.1 0 条不适用 | 造不出（推的） | 全零指针另有走读失败红；其余几版没有记录可比。建议 kb 括注补一句 |
| Q2.2 记录集合（措辞） | 站得住，且比正文列的多一处 | 判决「未释放记录」字面会在覆盖写历史上误红，实现取 E(θ_high) |
| Q2.2 合法误红 | 造不出（推的），两处待合入后验证 | I-3.18、I-5.1 绿时 (P) 蕴含 I-3.21；`accounting_seen` 门与「由记录施加出来的版」没核 |
| Q2.2 漏判 | 造不出（推的） | 走到的起点不在 E(θ_high) 记录起点集里就红；1195 形没拿到模型复跑 |
| Q2.3 两形比例 | 推的 | 合法写者写不出这两形；崩溃点重放各族 n/a 占比没量 |
| Q3.1 数能不能复核 | 待主 agent 跑 | 端口 `today_both_fixed` 与今天 Rust 逐支同语义；冻结输出：红 +7/+3，部分没判 −55/−372（判决写 370） |
| Q3.1 合法环变红 | 造不出（模型上） | 冻结输出合法红恒为 0；`p_rootfail` 档缺 `today_both_fixed`，待跑 |
| Q3.2 I-7.9 不改的理由 | 站得住（推的） | 写者上限不高于 checker 上限；有害的越界镜像上根在，checker 看得见 |
| Q4.1 全零并 | 一致 | 同一个函数同一个窗口锚；回收门槛值不变 |
| Q4.2 推空 | 一致，次序对 | 已推 + 1 + 1 > 8 与登记 A1 同形；带现行 F、txg 加一；写掉那一槽后下一轮走抬 F。注意：放不下抬 F 那一串时现在先读环 |
| Q4.3 按槽认盘 | 一处残留按 txg（卸载那一格），真写者上走不到（推的） | 暖机在准入之前覆盖每块盘 |
| Q4.4 理想模型 | 随机对拍不假红；故障注入解释口径有一处可能假红（推的，待跑） | 不必同批补模型，记 checks-owed |
| Q5.1 E183 | 离不开 | `GlobalAlloc` 契约与线程本地口径；没有可换的安全库 |
| Q5.1 E197 | 判的数离得开，互核与停机条款离不开 | 要放开分配器前先改登记 A7–A9、S3 |
| Q5.2 行数 | 每份 5 行、共 10 行，且门禁今天登不了 | `unsafe fn` 签名与 `unsafe impl` 落在函数体之外，要工具员那一格改；另要根改 deny、补注释 |
| Q5.2 理由 | E197 的理由最弱 | 写成成本理由，不写「离不开」 |

## 没做什么

- 不判别的腿的格：Q1–Q5 里不属于「逐句对代码核实现」「措辞更正站不站得住」「I-7.9 理由站不站得住」「Q5 代价对照」的，如攻方腿造反例、造合法历史，我没做；没碰禁读清单。
- 不替主 agent 采纳或出判决；Q5 的三条路只摆代价，没有倾向；没有往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没写模型，没起任何脚本、cargo、GPU；Q3.1 的数是第二轮冻结输出里的，Q4 的用例没跑。
- 没核：C944 的 1195 形（没拿到攻方模型）；`accounting_seen` 门与「由记录施加出来的那几版」在崩溃态上的 I-3.21（归合入后验证）；`txgs_ring_slots_bad_at_mount_can_hold` 本身的算术；GPU 对拍两张缺镜像；门禁今天对 `unsafe fn` 的反应没起门禁实测，只读了源码。
