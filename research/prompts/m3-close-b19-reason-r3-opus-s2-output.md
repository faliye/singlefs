# m3-close-b19-reason-r3 攻方腿第二样本（Opus，2026-11-03）

正文 `research/prompts/_m3-close-b19-reason-r3-body.md`，攻击面是正文第三节 Q1–Q5。没读第一样本与正推腿的报告。被判对象按开工快照 `research/prompts/m3-close-b19-reason-r3-snapshot/crates-sha256.txt` 读：开工时 36 份逐份 `sha256sum -c` 全过（这一次真跑、没有一行不是 OK）。本报告引的代码行号都在主工作区现取，与快照同一份字节。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-b19-reason-r3-opus-s2-model/`（纯算术，不读 `crates/`、不读盘；没跑，读数待主 agent 跑出）：

| 文件 | sha256 |
|---|---|
| `q3_zero_slot_merge_blind.py` | `b0d821e9e1d30b8045f5abfb52e2853dca5d651c7d3ac01ff50e61ffe30cfba6` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`） | `28681b0454c20c1406cdc8af06f8d2836df4fefcdfd779e2dd29578047070ec3` |

复跑整条见末尾「要主 agent 跑的命令」。`out/` 是空的：腿不跑模型，参照输出要等主 agent 跑出之后才有。

## 各格判定一览

| 格 | 结论 | 一句话 | 量的 / 推的 |
|---|---|---|---|
| Q1-1 误红（合法镜像） | 造不出 | 第五判只在 I-3.18 判红的盘上才走得到；合法镜像上 I-3.18 不红（根槽写不撕、单元写在根之前落盘） | 推的 |
| Q1-1 漏判（两条都已释放而相交） | 造不出 | 两条已释放记录相交在记录自述那一份里就相交，I-5.4 逐棵树判、不分释放与否 | 推的（按代码） |
| Q1-1 附带 | 打中（措辞） | 第二轮判决点名「不算病」的 18 个隔离形（两条都带记号），今天的第五判照红 | 推的 |
| Q1-2 与 I-5.4 | 造得出（重复报，不矛盾） | 一条已释放记录在记录自述那一份里就压在一条未释放记录上，同盘另有一条 I-3.18 判红的记录时，I-5.4 与 I-3.1 第五判同红一对，I-3.1 的说明说是「改记之后」相交 | 推的（构造法给出，没造盘） |
| Q1-3 说明里的数 | 造不出 | 说明只在第五判通过时打；第二轮那几形今天都判红、不进说明 | 推的 |
| Q2-1 「0 条不适用」 | 打中（字面）＋推的漏 | 树表指针全零也走进「0 条」那一支报不适用，条款与 I-5.4 都把它当读不出报部分没判；树表 0 条写过行的那一版上，写行发布自己写的单元缺记录这一形，I-3.21 不适用、I-3.1 也没有记账可比 | 前者按代码，后者推的 |
| Q2-2 记录集合 | 误红造不出；漏判造不出 | 每条候选根与由记录施加出来的那一版 txg 都不低于 θ_high，它们走到的单元在最新根里要么未释放、要么释放代高于 θ_high；第二轮 1195 个 C944 形若用一条已释放记录顶替，就是两条记录相交、I-5.4 红 | 推的 |
| Q2-3 部分没判比例 | 造不出 | 合法写者写不出「有条目而没有种类 3」与「树表读不出」 | 推的 |
| Q3-1 两处同批改 | 没复核数；从部分没判变红造不出 | +7、+3、−55、−370 没在今天的实现上复核（要跑重型或另写端口）；合法环里槽 0 不会空着，真环仍在枚举里 | 推的 |
| Q3-2 I-7.9 上限不跟 D16 改 | **造得出（坏写者越过字面上限而 checker 放过）** | 写者不并读回全零的槽、F 抬过它能住的最大 txg，镜像上那一槽仍全零、checker 看得见却不并，I-7.9 判成立；实现员「不算违反式子」那一句与 D16 已定项 1 第 38 行字面不符 | 最小历史手算＋模型（待跑） |
| Q3-2 写者与 checker 上限不同而误红 | 造不出（无读故障时） | 写者并全零槽只会把自己的上限压低 | 推的 |
| Q4-1 全零并 | 相符 | 并的是挂载那一刻读回全零、之后没写过的槽，能住的最大 txg 与读坏槽同一算法；回收门槛那一边补丁里只换了取值的写法、取的仍是最旧 txg | 按代码与补丁 |
| Q4-2 推空 | 相符 | 「已推 + 1 + 1 ≤ 8」与抬 F 那一串同一个 B 记法；带现行 F、txg 加一；写掉那一槽之后下一轮照抬 F 那一串抬 | 按代码 |
| Q4-3 按槽认盘 | 真写者的准入路径上走不到（推的）；另两处仍按 txg 认盘、结局相同 | 每一次准入抬 F 都在写行与暖机之后，暖机已在每块盘上落了本实例 txg > 0 的根；卸载那一处与抬 F 那一遍取最旧有效根那一处仍按落点公式认盘 | 推的 |
| Q4-4 理想模型 | 会报假红（推的） | 故障注入的历史有重挂；被吞的根槽写在年轻环里留下全零槽，重挂之后写者并它、推空，模型与「去掉被吞的根重算」都不并，`left_by_the_lying_device` 认不下 | 推的 |
| Q5 计数分配器 | 打中三句 | E197 判的数是服务的 `memory.peak`，分配器只互核、离得开；放开表今天的格式表达不了 bin 根与 `unsafe impl`；E183 的 SAFETY 注释「计数是原子的」在这一批加了线程本地计数之后只说了一半 | 按代码与 kb |

## Q1　I-3.1 取宽第五判

### Q1-1 误红：造不出（推的）

第五判前面有两道门：这块盘上要有 I-3.18 判红的未释放记录，改记那一份才存在：「some_span_was_set.then_some(set_to_the_unit_classes)」（`crates/singlefs-checker/src/walk.rs:10523`）；改记那一份三判与逐槽两判都过了才问相交：「if the_misfit_lies_in_units_the_span_invariant_reddens {」（`crates/singlefs-checker/src/walk.rs:21739`）、「if records_overlapping_in_the_unit_class_reading.is_empty() {」（`crates/singlefs-checker/src/walk.rs:21740`）。所以第五判在一张镜像上判红，先要 I-3.18 在那张镜像上红。按点名的几形逐个看：

| 形 | I-3.18 在合法写者的镜像上红不红 | 依据 |
|---|---|---|
| 崩溃点重放的崩溃态 | 不红：未释放记录罩住的单元在根（或记录）落盘之前已落盘；单元写是 COW、根槽写不撕 | 「根槽写是一个物理块，照旧两态」（`crates/singlefs-checker-tier/src/crash.rs:2175`）；D16（发布语义） 已定项 7 的持久顺序 |
| 隔离记号 | 不红：带记号的记录是未释放的（I-3.17 ①，`crates/singlefs-checker/invariants/05-空间记账.md:24`），I-3.18 判它；它罩的是 journal 点名、在记录之前已落盘的单元，头的类别与跨度相符 | 推的，没造盘 |
| 读坏槽钉门槛 | 不涉及：钉只把 t 压低，第五判只比记录区间，不读 t | 同上两行代码 |

推翻条件：任何一张合法写者写出的镜像上 I-3.18 判红。

### Q1-1 漏判（两条都已释放而相交）：造不出（按代码）

第五判确实不比两条都已释放的：「(true, true) => continue,」（`crates/singlefs-checker/src/walk.rs:10595`）。但改记只改未释放的记录，已释放的原样留着：「if record.is_released {」（`crates/singlefs-checker/src/walk.rs:10508`）。所以改记那一份里两条已释放记录相交，就是记录自述那一份里它们相交，I-5.4 逐棵树判、不看释放与否：「.find(|pair| pair[0].0 + pair[0].1 > pair[1].0)」（`crates/singlefs-checker/src/walk.rs:9672`）。I-5.4 按一棵分配记录树判，一版要是有两棵种类 3 的树、两条记录分在两棵里，I-5.4 看不见；那一形是 I-9.16 ③ ④ 判红的畸形树表。推翻条件：合法写者写出一版带两棵种类 3 树的镜像。

### Q1-1 附带：第二轮判决点名「不算病」的隔离形，今天判红（推的，没造盘）

第二轮判决 Q1 第 1 问那一格的末句：「改宽罩进隔离豁免的槽（18 个）两条记录都带记号、不复用，不算病」（`research/prompts/m3-close-b19-reason-r2-main-verification.md:16`）。今天的第五判只放过两条都已释放的那一对，带记号的记录都是未释放的，两条都带记号的一对照比，相交就把 I-3.1 判红（`crates/singlefs-checker/src/walk.rs:10592`–`10596` 那一段 match，没有隔离记号的分支）。

四句：不涉及判臂；判别子在镜像上（记号在跨度段里）；满足的是 I-3.1 状态格第五判那一分句「不与这块盘上别的记录（未释放或已释放）罩住的槽区间相交」（`crates/singlefs-checker/invariants/05-空间记账.md:8`），条款字面判红是对的，对不上的是判决那一句「不算病」；改法两个都能让两边一致：甲，判决那一句改成「这 18 个今天判红」；乙，第五判对两条都带隔离记号的一对不比。两个都只在我的推演上看过、被攻过零轮。这 18 个都是 I-3.18 已红的坏镜像，红多一条不放过坏写者，甲的代价是 I-3.1 多报一条；乙要先答「两条隔离记录相交是不是病」，判决只推了一句。

### Q1-2 第五判与 I-5.4：造得出重复报，不矛盾（推的，构造法，没造盘）

第五判拿每条未释放记录跟别的每条记录比，不限于被改记的那几条。于是一处在记录自述那一份里就有的相交（I-5.4 的病），只要同一块盘上另有一条 I-3.18 判红的记录让 I-3.1 走进取宽那一支，就被 I-3.1 再报一次，说明却写成「按 I-3.18 判红的记录改记成单元那一类之后三判与逐槽两判都成立，而改记之后未释放的记录罩住的槽与别的记录相交」（`crates/singlefs-checker/src/walk.rs:10734`），两条记录其实都没被改记。

最小镜像（构造法）：

1. 起点是 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_unreleased_allocation_record_spanning_one_slot_of_a_two_slot_data_unit_reddens_only_the_span_invariant`（数据单元那条未释放记录跨度改 1，`crates/singlefs-checker/invariants/05-空间记账.md:25` 点名它只红 I-3.18）。这张上 I-3.1 走进取宽那一支、报部分没判。
2. 在同一块盘上取一条已释放记录 R（新池新建文件之后有：第 0 版树表单元那一条，`crates/singlefs-checker/invariants/05-空间记账.md:8`「已释放、在 defer 队列里」）与起点紧挨在它后面的一条未释放记录 X（R 的起点槽 + R 的跨度 = X 的起点槽），把 R 的跨度加 1、重封那个节点。
3. 预期：I-5.4 红（R 罩 X 的起点）；I-3.1 改记那一份的 E(θ_high)、E(t)、Ω ∪ U 都不变（R 多罩的那一槽本来就在 X 的 E 里、被遍历走到，不进 U），三判与逐槽两判照过，第五判找到 (X, R) 这一对，I-3.1 判红。两条不变量红在同一对上。镜像上要是没有紧挨的 R、X，这一张就不成，推的。

四句：不分臂；判别子观测得到（哪几条记录被改记，改记那一份与记录自述那一份逐条比得出）；I-3.1 第五判那一分句字面是「每条未释放记录」，红是字面许可的，归错的是说明那一句；改法：第五判只比「至少一条跨度被改记」的那几对，没改记的两条相交留给 I-5.4（只在推上、被攻过零轮）。

矛盾找不到：第五判看得见、I-5.4 看不见的，只有改记之后才相交的那几对，两条判的是两种读法，不是同一件事给相反的结论；两条都看得见的那几对，两条同红。

### Q1-3 说明里的数：造不出（推的）

部分没判的说明只在第五判没找到相交时才打（`crates/singlefs-checker/src/walk.rs:21740` 那一支），第二轮那两形（隔离单元尾槽压在释放代落在 (t, θ_high] 的记录上、窄记录尾槽上另有一条记录）都是改记之后相交，今天走 `crates/singlefs-checker/src/walk.rs:21784` 那一支判红、打的是点名每一对的违例说明，读说明的人不用在部分没判的说明里分它们。说明里 U 是用记录自述那一份的界相减得出的（「let still_deferred_outside_the_walk = as_recorded」（`crates/singlefs-checker/src/walk.rs:10652`）），U 只由已释放记录定，改记不碰已释放记录，两种读法的 U 相同，这一减不会报错数。

## Q2　I-3.21 与「0 条不适用」

### Q2-1 打中（字面）：树表指针全零也报「不适用」，条款要的是部分没判（按代码）

读最新根的记录时，树表指针全零直接交回 0 条：「tree_table_entries: 0,」（`crates/singlefs-checker/src/walk.rs:10334`，它上面一行是 `if tree_table_pointer.all_zero {`），I-3.21 把 0 条这一支报成不适用：「NO_ALLOCATION_RECORD_TREE_IN_THE_TREE_TABLE_OF_THE_NEWEST_ROOT,」（`crates/singlefs-checker/src/walk.rs:10774`）。GPU 同一个读法：「c_newest_tree_table_entry_count = 0u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2251`，上一行是全零那一判）、「if (c_newest_tree_table_entry_count == 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3103`，直接返回、不置位）。

条款 I-3.21 状态格的不适用只给「最新根的树表 0 条（mkfs 的第 0 代、照抄它的暖机根、树表 0 条写过行的那一版）」，「树表读不出 …… 射程里判不了、报部分没判」（`crates/singlefs-checker/invariants/05-空间记账.md:28`）；树表指针全零在 I-5.4 那边是用户 2026-10-05 定甲的「读不出」：「树表指针全零的那几版：core 照它读盘 0 槽 0、读不出（用户 2026-10-05 定甲），与树表读不出同一处置；此前当这一版没有树表。」（`crates/singlefs-checker/src/walk.rs:8033`）。I-3.21 这里还是「此前」那个读法。

四句：不涉及臂；判别子在镜像上（根记录树表指针那 86 字节）；满足的是条款「树表读不出」那一分句，却报成了「0 条」那一分句的不适用（归错分句）；改法：`allocation_records_of_a_root_record_without_judging` 的全零那一支交回读不出（`None`），I-3.21 随之报部分没判，GPU 把全零那一支的 `c_newest_tree_table_entry_count` 记成 `NONE`（只在我的推演上看过、被攻过零轮；同一个函数还被别处调用，`crates/singlefs-checker/src/walk.rs:10292` 那一处要一并看它对全零是怎么用的，我没核）。这一形是坏镜像（主走读另判损坏），不放过病，错的是部分没判的计数与理由。

### Q2-1 推的漏：树表 0 条写过行的那一版上，写行发布自己写的单元缺记录，没有一条不变量罩着

那一版的树表是 0 条，分配记录树由根记录直接持有（C512），写行那次发布把自己新写的实例表片与分配记录树节点都记进这棵树：「这些节点自己的分配记录也要进这棵树，所以落点都取完才装节点。」（`crates/singlefs-core/src/transaction.rs:2095`）。坏写者在这棵树里漏记一片新实例表：

- I-3.21：树表 0 条，报不适用（上一节同一支）。
- I-3.1：「为什么是这两样：树表 0 条 ⇒ 这一版没有记账树」（`crates/singlefs-core/src/transaction.rs:1798`），没有记账可比。
- I-3.18、I-3.15、I-3.10、I-5.4：都从记录出发看单元，缺的那条记录不在，它们没有对象。

镜像在下一次可写挂载之前停在这一版（写完行、崩在第一个文件之前），这一形就留在 checker 之外；重挂时分配器按这棵树重建，那一片成了空闲槽（推的，没造盘、没跑）。条款把整版划成不适用的理由是 mkfs 写的两个单元在第一个文件之前合法地没有记录（第二轮判决 Q2「C944 的合法来源」那一格），这个理由只罩 mkfs 的单元，不罩写行发布自己写的那几个——两者分得开：后者的单元头诞生代号就是这一版的 txg。改法（只在推上、被攻过零轮）：树表 0 条而根记录持有分配记录树的那一版上，诞生代号等于这一版 txg 的走到的起点照 I-3.21 比根记录持有的那棵树；mkfs 的单元照旧不比。推翻条件：真写者的写行发布不把新实例表片记进那棵树（那样这一形是合法来源，改法会误红）。

### Q2-2 记录集合「未释放或释放代高于 θ_high」：误红造不出、漏判造不出（推的）

误红那一边：遍历的版本是 txg ≥ F_生效、不被抛弃的根，与由记录施加出来的那几版；θ_high = max(F_生效, 环里最旧有效根)（`crates/singlefs-checker/src/walk.rs:9811` `deferred_slots_reclaim_threshold`）。由记录施加出来的那一版按 F_生效 滤：「let versions_applied_only_by_records = versions_applied_only_by_records(」（`crates/singlefs-checker/src/walk.rs:21007`，第三个实参是 `effective_rollback_floor`），且要求环里有一条有效根比它旧（第 12666 行 ④）。所以每个被走的版本 txg 都不低于 θ_high；它走到、而最新根已不再引用的单元，释放代大于那个版本的 txg，就大于 θ_high。第二轮两腿列的合法来源（覆盖写换下的、映射换位的源槽、由记录施加出来的那几版新分配的、第 0 版树表单元）逐种落在这一句里。附带：那个函数的文档注释第 ③ 条写的是「③ **不低于回退下界**：txg ≥ 最新根带的 F，与根环里的候选根同一条（D16（发布语义） 已定项 1）。」（`crates/singlefs-checker/src/walk.rs:12665`），调用处传的是 F_生效，注释与代码不一致；按注释实现的话，F_生效 高于最新根带的 F（抬 F 先写了系统配置、带新 F 的根没落）时会多走一版、I-3.21 与 I-3.1 (P) 同红。今天按代码不红。

漏判那一边：第二轮修正模型上的 1195 个 C944 形是「一条宽的未释放记录罩住一个走到、没有自己记录的邻居」。改取 E(θ_high) 之后能漏的，只能是那个邻居的起点恰好是一条已释放、释放代高于 θ_high 的记录的起点；那条记录与宽记录罩同一槽，记录自述那一份里就相交，I-5.4 红（`crates/singlefs-checker/src/walk.rs:9672`）。所以「除 I-3.18 外都不红」的那 1195 个不会因为放宽到 E(θ_high) 漏掉。推翻条件：一张坏镜像上 I-3.21 成立、I-5.4 成立、走到的某个单元起点只有一条已释放而释放代高于 θ_high 的记录、最新根却还引用它——那是 I-3.9 的病（每条有效根都还引用它就不许带已释放标志），不归这一条。

### Q2-3 两形部分没判：合法写者写不出（推的）

「有条目而没有种类 3」是 I-9.16 ④ 的畸形，合法写者带文件的一版都有种类 3；「树表读不出」在合法镜像上要最新根的树表单元坏掉，它在根落盘之前已落盘、被最新根引用、不会被回收复用。读故障不留在镜像上。所以部分没判在合法写者的镜像上只来自坏写者或介质故障，比例不会让 I-3.21 形同虚设。没有量。

## Q3　第 0 代根两处与 I-7.9 的上限

### Q3-2 打中：不并读回全零槽的坏写者越过 D16 字面上限，I-7.9 判成立（最小历史手算；模型待跑）

D16（发布语义） 已定项 1「抬 F 的上限」那一行（`.claude/kb/decisions/16-发布语义.md:38`）这一批起写着读坏槽与读回全零的槽都并进上限：挂载那一刻读回全零、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它，能住的最大 txg 同读坏的槽。I-7.9 的条文说上限照这一行（`crates/singlefs-checker/invariants/02-根环.md:16`「上限照 D16（发布语义） 已定项 1「抬 F 的上限」那一行」），而 checker 两遍都不并全零槽：重建那一遍「RingSlotOfTheImageSeenFromTheRaise::AllZero => Ok(vec![None]),」（`crates/singlefs-checker/src/walk.rs:9013`）。实现员不改的理由里有一句：「只有那一槽真是全零时，漏了新条款的写者 checker 看不出来；这时盘上没有一条根被越过，不算违反式子，是写者那一侧的处置，归写者的单测。」（`/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-impl-checker/result.md:61`）。「不算违反式子」与第 38 行字面不符：并全零槽就是那一行的式子的一部分，不是式子之外的写者处置。

最小历史（R = 3、S = 4、区域 0 / 1 / 2 在盘 0 / 1 / 0，F 起点 0；每一步都是真写者走得到的：崩在记录之后、根槽之前是 D16 已定项 7 的持久顺序许可的崩溃点）：

| 步 | 发生的事 | 根环（槽 = (区域, 槽号)） |
|---|---|---|
| 1 | mkfs | (0,0) (1,0) (2,0) 各一份第 0 代根 txg 0 |
| 2 | 实例 A 可写挂载：写行 txg 1（盘 1）、暖机 txg 2（盘 0）；写文件 txg 3、4 | (1,0)=1、(2,0)=2、(0,1)=3、(1,1)=4 |
| 3 | 写文件 txg 5：记录落盘，根槽 (2,1) 没落，崩溃 | (2,1) 从没写过，读回全零 |
| 4 | 实例 B 可写挂载：恢复施加 5；N = 4、first = 6，(2,1) 记成挂载时读回全零，能住的最大 txg = (4 − 12, 5] 里落到它的 = 5 | — |
| 5 | B 写行 txg 6（盘 0）、暖机 txg 7（盘 1）；写文件 txg 8、9、10 | (0,2)=6、(1,2)=7、(2,2)=8、(0,3)=9、(1,3)=10 |
| 6 | txg 10 之后准入不够、抬 F | 有效根全在；状态去重从新到旧 10、9、8、6（6 承 5 的状态、与 4 不同），第 4 新是 6；盘 0 最新 9、盘 1 最新 10 |

不并时的上限 = min(9, 10, 6) = 6；照第 38 行并进 (2,1) 之后 = min(6, 5) = 5。照字面的写者（今天的实现）抬到 5；不并的坏写者抬到 6。抬 F 那一串 txg 11（(2,3)）、12（(0,0)）、13（(1,0)），(2,1) 要到 txg 17 才被盖，判的时候镜像上仍全零。checker 判 txg 11 那条根：窗口锚在 13，窗口里 txg 2–10 的根两块盘都有、去掉最旧那条之后不同状态 6 个 ≥ 4，走窗口那一判，上限 = min(9, 10, 6) = 6，F = 6 判成立。坏写者越过字面上限 1 个 txg，I-7.9 放过。

四句：

1. 分不分辨臂：分辨「并」与「不并」两种写者——前者 F = 5，后者 F = 6，镜像不同；checker 两张都判成立，它分不开这两臂。
2. 判别子看不看得到：看得到。(2,1) 整槽全零留在镜像上；它能住的最大 txg 要 B 挂载那一刻的 N 与 first，这两个数从镜像推得出（B 的第一条根 txg 6、实例表行 (A, 5)、journal 里 txg 5 那条记录）。这与读坏槽不同：读坏「读故障不留在盘上」（I-7.9 条文那一句只说读不出、自证不过的槽，`crates/singlefs-checker/invariants/02-根环.md:16`），全零留在盘上。
3. 满足的分句：D16 第 38 行「读坏槽与读回全零的槽都并进上限」那一句；I-7.9 条文里「窗口里挂载那一刻读不出或自证不过、之后这个进程没写过的槽 …… checker 看不见」那一句没有把全零槽列进去，条文与代码都只按读得出的根判。
4. 改法在打中的格上中不中：实现员给的「归写者的单测」——单测钉的是一张固定镜像（`crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs`，没洞那一份 F 停在 17）与变异表一行（`crates/mutations.tsv:6527`「D16（发布语义） 已定项 1 全零并：读回全零的槽不并进准入抬 F 的上限」），崩溃点重放里别的流、别的形状上坏写者不会被 checker 抓到；对这一格只部分起作用。checker 侧的改法（零轮、只在推上）：判抬 F 的根时，镜像上整槽全零、落点式在 (那个实例挂载时环里读得出的最新有效根 − 3S, 那个实例第一个 txg − 1] 里落得到、能住的最大 txg 不低于抬之前 F 的槽，把那个 txg 并进上限下沿；N 或 first 推不出时报部分没判。代价与误红面我没量。

模型 `q3_zero_slot_merge_blind.py` 把上表这一形按 S = 4 / 8 / 16、洞的 txg h、抬 F 的 txg c 扫一遍（要求抬 F 那一串最后一个 txg 小于 h + 3S、洞那一槽判的时候仍全零），打出每个 S 下历史条数、不并的写者越过字面上限的条数与其中 checker 判成立的条数。模型里 checker 那一栏用的就是不并的式子（按 walk.rs 重建那一遍「同一实例的那一版认作落了盘」，在这段历史上重建出的环就是真环，推的），所以「checker 判成立」那一栏按构造等于越过那一栏：模型量的是这一形有多常见，不是独立地再证 checker 判不出。读数待主 agent 跑（命令在末尾），上表那一条（S = 4、h = 5、c = 10）应是第一条例子之一（推的）。

### Q3-2 另一半：写者与 checker 上限不同而误红：造不出（无读故障时，推的）

写者并全零槽只压低自己的上限（`crates/singlefs-core/src/mount.rs:4088` 只并能住的最大 txg 不低于 F 的槽，再取 min），F ≤ 写者上限 ≤ 不并的上限 = checker 的上限，checker 不误红。按槽认盘之后写者与 checker 认盘一样（`crates/singlefs-core/src/mount.rs:4024`、`crates/singlefs-checker/src/walk.rs:8888`）。带读故障的那几形（写者挂载时读坏一槽、那一槽其实住着它那块盘上唯一的根）会让写者按能住的最大 txg 抬过那条根、checker 看见那条根判红，这是 I-7.9 条文已记的「读故障不留在盘上」那一类，不是这一批新出的，不算。

### Q3-1 第 0 代根两处同批改

数（+7、+3、−55、−370）：没在今天的实现上复核。今天的实现是池级 checker 与 GPU 判器，复核要在崩溃点重放或第二轮模型的端口上跑；第二轮辩方模型（`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/`）是端口不是实现，重跑它只能复现第二轮的数。记「没复核」。

逐行对过的是两处改动的形：CPU 在落点式住进这一槽的正 txg 认完、槽 0 而抬之前 F 为 0 时记一条第 0 代根，「&& floor_before_the_raise == GENESIS_CHECKPOINT_TXG」（`crates/singlefs-checker/src/walk.rs:9029`），树表按 ABSENT 记；GPU 同判（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1876` 判槽 0 且 F 为 0，基记 NONE，`c_user_visible_pointers` 对 NONE 记成两棵都没有、判得了）。实例号两边都取 0，与 core 的 `MKFS_INSTANCE_GENERATION` 相同（`crates/singlefs-core/src/make_filesystem.rs:52`）。

合法环从部分没判变成红：造不出（推的）。改动只把「槽 0、F 为 0、往下认完」那一种可能从「空着」换成「第 0 代根」。合法写者的镜像上槽 0 在抬 F 那一刻不会空着：mkfs 在三个区域的槽 0 都种了第 0 代根，根槽写不撕（`crates/singlefs-checker-tier/src/crash.rs:2175`），盖它的只能是一条完整的根。所以真环仍在枚举出的那几种里；红要每一种都红，真环那一种按写者自己的式子不红（写者按槽认盘之后两边式子相同）。推翻条件：一张合法写者的镜像，抬 F 那一刻某个区域的槽 0 既不是第 0 代根、也不是任何一条完整的根。

## Q4　代码轮：写者一侧

### Q4-1 全零并：相符（按代码与补丁）

| 条款（`.claude/kb/decisions/16-发布语义.md`） | 代码 | 对上没有 |
|---|---|---|
| 第 38 行：挂载那一刻读回全零、之后这个进程没写过的槽并进上限，能住的最大 txg 同读坏的槽 | 「pub fn ring_slots_bad_or_reading_back_all_zero_at_mount(」（`crates/singlefs-core/src/allocator.rs:1687`）把 `BadAtMount` 与 `AllZeroAtMount` 并起来；两类的数都出自 `txgs_ring_slots_bad_at_mount_can_hold`（`crates/singlefs-core/src/mount.rs:2937`），窗口锚 N 与 first 同一份 | 对上 |
| 第 39 行：重读读回全零照旧钉着、照旧并进上限；写掉那一槽、或进程内重读读出有效根或被抛弃的根才解开 | `AllZeroAtMount` 的文档与解开时机同 `BadAtMount`（`crates/singlefs-core/src/allocator.rs:1263`–`1270`） | 对上 |
| 回收门槛那一边不动 | 补丁 `~/.local/share/singlefs/work/m3-close-b19-d16-impl-writer/patch/crates.patch` 里重建那一处只把全零那一类的元素从「最旧 txg」换成「两个数都带」，回收门槛照旧取 `oldest_txg_it_can_have_held()`（补丁第 244–252 行那一块） | 对上 |
| 只并能住的最大 txg 不低于现行 F 的 | 「*highest_txg_it_can_hold >= current_floor)」（`crates/singlefs-core/src/mount.rs:4088`） | 条款没写这一滤；那一槽里的根 txg ≤ 能住的最大 txg < F 就不是有效根，滤掉与「F 不越过它」同义（推的） |

### Q4-2 推空：相符（按代码）

- 卡住的判法：「self.not_passing_the_ring_slots_bad_since_the_mount <= floor」（`crates/singlefs-core/src/mount.rs:3975`），下一行要求不并时高于 F，就是 E173 登记「推空B」那一臂「C并 ≤ F 而 C0 > F」（`.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:359`）。因为 C并 只并能住的最大 txg ≥ F 的槽（上一节最后一行），卡住时那个 txg 恰好等于 F（推的，算术）。
- 预算：「publishes_pushed + 1 + 1 > PUBLISHES_PER_ADMISSION_AT_MOST;」（`crates/singlefs-core/src/mount.rs:4810`）。抬 F 那一串在同一个函数里是「已推 + 这一串的次数 + 1」，末尾的 + 1 一直是「这次准入为之判的那一次发布」；推空用同一个记法，与第 44 行「一次准入最多 B …… 上限卡在读坏槽或读回全零的槽上推的空发布也算在内」一致。
- 发的是什么：「empty_publish_plan_after_file_version(current, current.record.instance),」（`crates/singlefs-core/src/mount.rs:4899`），那份计划带现行那一版的 F、txg 是现行加一（`crates/singlefs-core/src/mount.rs:5610`、`crates/singlefs-core/src/mount.rs:5624`）。
- 次序：卡住就先推空（`crates/singlefs-core/src/mount.rs:4835`），推到写掉那一槽之后那一槽不再记在根环表的两类里，下一轮 `from_the_roots_read` 与并进之后的上限相同、高于 F，走 `raise_to_the_admission_ceiling_judged`。没找到次序错。

### Q4-3 按槽认盘：真写者准入路径上走不到（推的）；两处仍按 txg 认盘，结局相同

走不到的论证：写者算抬 F 上限的三个调用处都在写行与暖机之后。可写挂载那一处：「写完行（与暖机）之后推抬 F 的空发布」（`crates/singlefs-core/src/mount.rs:7496`）与 `push_floor_raises_after_the_row_publish` 的文档「写行与暖机之后按」（`crates/singlefs-core/src/mount.rs:8531`）；挂载续做在暖机之后；会话那一处（`crates/singlefs-core/src/mounted_session.rs:4619`）在挂载做成之后。暖机推到每块盘上都有一条本实例的根（`warm_up_publish_txgs`），那几条 txg > 0、不被抛弃、不低于 F。于是每一次准入抬 F 时每块盘上最新的有效根都是 txg > 0 的根，第 0 代根不会是哪块盘上最新的那一条，按槽认盘与按落点公式认盘算出同一个上限。破这条论证要「暖机那几次根槽写 FUA 返回成功而没落盘」，那是说谎的设备（下一节）。第二轮那段历史只在 `admission_floor_ceiling_counts_each_genesis_root_on_the_device_of_its_ring_slot.rs` 手拼的根环上成立，那份用例文件头自己也写了「这一形在真写者的可写挂载里走不走得到没有核」。

仍按落点公式认的两处（逐处看了，结局都不变）：

| 处 | 代码 | 为什么结局相同 |
|---|---|---|
| 卸载那一串当场放开那一截取每块盘上最新有效根 | 「(device_of_txg(root.checkpoint_txg), *root))」（`crates/singlefs-core/src/mount.rs:5275`） | 文档已写：卸载接在一次可写挂载之后，暖机已在每块盘上落了根（`crates/singlefs-core/src/mount.rs:4254`）；与上一段同一个论证 |
| 抬 F 那一遍取环里最旧有效根时认「这一遍读出了根的槽」 | 「target_for_publish(root.checkpoint_txg, slots_per_region))」（`crates/singlefs-core/src/mount.rs:5044`） | 区域 1、2 槽 0 读出的第 0 代根被认成 (0,0)，那一槽要是挂载时读坏、钉在 0，滤不掉、走重读；重读结局与不重读都取 0（第 0 代根的 txg 就是 0）。只读、不改分配器 |

挂载择根、回退判候选、算 F_生效：这几处取的是 txg、F 的最大值或候选集，不按盘取最小，第 0 代根记在哪块盘不改结局（推的，没逐处现读）。

### Q4-4 理想模型不认读坏、全零与推空：故障注入会报假红（推的）

- 随机历史不建崩溃，挂载时没有全零槽（txg < first 的每一版根槽都落了），也没有读坏，两边一致。
- 故障注入有说谎的设备吞掉根槽写，历史里有重挂（`crates/singlefs-harness/src/fault_injection.rs:2789`「实现交回的每一份隔离，跨重挂累积」）。年轻的环里被吞的那一槽从没写过、重挂时读回全零，写者照新规则并它：上限低于模型、可能推空。`left_by_the_lying_device` 只认「实现报的上限等于去掉被吞的根之后模型重算的那一个」：「(Some(reported), Some(Some(recomputed))) if reported == recomputed」（`crates/singlefs-harness/src/fault_injection.rs:2705`），重算用的是不并的式子；推空多出来的根落在 `PublishCount`，那一格不在认得下的几格里（`crates/singlefs-harness/src/fault_injection.rs:2734`）。两种都照报新发现。要让它成假红，被吞的那一次要在中间（之后还有根落了），撑上限的四个状态与每块盘最新根都在它之后——与 Q3-2 那张历史同形。
- 模型的盘按 txg 认（「.entry(device_holding_the_root_of(root.key.checkpoint_txg))」（`crates/singlefs-harness/src/model.rs:3943`）），写者按槽认；按 Q4-3 的论证准入时走不到分歧。
- 要不要同批补：要补的是故障注入那一层的解释（上限比较取「去掉被吞的根重算、再并上重挂时读回全零的槽」），或模型的上限函数；推空那一格要模型知道哪几槽在重挂时读回全零。我没跑故障注入，假红出不出得来、多少，是推的。

## Q5　两份装置的计数分配器

### E197 判的数不是分配器量的：离得开（按代码与 kb）

E197 装置文件头：「判的数是统一任务服务记下的那一次尝试的 `memory_peak`（scope 的 `memory.peak`），驱动」（`crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:17`）；实验页口径：「进程内的分配器峰值与 `VmHWM` 只用来互核，不判」（`.claude/kb/experiments/197-扫环胶水内存随盘数.md:35`）。`VmHWM` 已经不经 unsafe 读到了：「let status = std::fs::read_to_string("/proc/self/status").ok()?;」（`crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:1054`）。所以登记理由里写「E197 量分配字节、非放开不可」那一类句子站不住：判的数在服务的 cgroup 峰值上，拿掉分配器判定行不变。拿掉要动的是登记第七节的两项自核（A7 分配器峰值与推算值 5% 容差、A8 量窗对照落在登记区间），它们换成 `VmHWM`（页粒度、含分配器自身开销）之后容差要重登，属自核口径的改动，不是被判量的口径。E197 一个进程一格（文件头第 9 行起那一段），按进程读 `VmHWM` 够用，不需要线程本地计数。

### E183 离不开（按代码）：量的是一次克隆留下的 `Layout` 字节

「measure_in_a_window(&COUNTING_ALLOCATOR, || mounted.allocator.clone());」（`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:3371`）之后 `copy_bytes` 取量窗末尾的在用字节减起点，是分配器口径的字节数。不经 unsafe 的替代我只找得到 `/proc/self/status` 的 `VmRSS` / `VmHWM`（可用 `/proc/self/clear_refs` 写 5 重置峰值，纯 std）——页粒度、进程级、受 malloc 复用影响，换它就是改登记里 R2 那几个量的口径。仓里 `Cargo.lock` 与本机 cargo 缓存里没有现成的安全计数库（`grep` 两处，只有 `libc`），离线加不进来。服务的 cgroup 峰值是整进程整段的，分不出量窗。

### 「两份各登几行」：今天放开表的格式表达不了这两份（按表头）

- 第 2 段是「crate 根（src/lib.rs）允许写的级别」（`.claude/gate.d/unsafe-allowances.tsv:6`），`singlefs-checker-tier` 那一行写 `forbid`、没有放开（`.claude/gate.d/unsafe-allowances.tsv:31`）。两份装置是 `src/bin/` 下各自的 crate 根，不归 lib.rs 管，这一段没有它们的位置。
- 键是（crate，函数）：「# 同一个（crate，函数）登两行、同一个 crate 的几行第 2 段不一样，判红。」（`.claude/gate.d/unsafe-allowances.tsv:20`）。两份的 unsafe 都在 `alloc`、`alloc_zeroed`、`dealloc`、`realloc` 四个同名方法里，同在 `singlefs-checker-tier` 目录下，按今天的键登两份就撞。
- 第 5 段数的是「这个函数里 unsafe 关键字几处」。每份 9 处里有 1 处是 `unsafe impl GlobalAlloc` 本身，不在任何函数里；其余 8 处是 4 个 `unsafe fn` 与 4 个 `unsafe { … }`，按函数是 4 行、各 2 处。「每份 1 行、9 处」不合这一段的数法；「每份 4 行、各 2 处」又剩下 `unsafe impl` 那 1 处没处登。
- 这三条是用户 2026-11-03 定的「一处一行」改格式要先答的；登记理由写得再好，格式不改就登不进去、门禁照红。

### E183 的 SAFETY 注释只说了一半（按代码）

E183 的 `unsafe impl` 上方：「// SAFETY: 每个方法原样转给 `System`，只在成功之后改计数；计数是原子的，不碰被分配的内存。」（`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:206`）。这一批加了线程本地的那一份：「let thread_in_use_now = THIS_THREAD_IN_USE_BYTES.with(」（`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:142`）那一份改的是 `Cell`，不是原子量。分配器里读写线程本地量要成立的前提是「常量初始化、没有析构、不再分配」，这一句写在 `thread_local!` 的文档注释里（第 122–123 行），不在 SAFETY 注释里；表头第 6 段要求从 SAFETY 注释里取它真写着的前提，照今天的注释取出来的是「原子」，罩不住线程本地那一份。E197 的同位置写的是「只在成功之后改线程局部计数；计数不碰被分配的内存，也不再分配」（第 139 行），是对的。另：E183 用 `with`、E197 用 `try_with`，线程拆掉之后再分配时前者会 panic、后者不会；常量初始化、没有析构的线程本地量在 Linux 上 `with` 不会失败（推的，没核标准库的保证写在哪）。

## 没打中的形状

| 问 | 试过的形 | 范围 | 结局 |
|---|---|---|---|
| Q1-1 误红 | 合法镜像上让 I-3.18 红：崩溃态撕根槽、撕单元、隔离单元头与跨度不符、钉门槛改 t | 按代码与崩溃枚举的撕裂规则逐形推 | 都到不了第五判 |
| Q1-1 漏判 | 两条已释放记录相交；分在两棵种类 3 树里；跨度 0 的记录（第五判会报一段空的「相交」[s, s)，只在坏镜像上） | 按代码 | I-5.4 罩着，或是 I-9.16 的畸形；跨度 0 那一形只是说明难看 |
| Q2-2 误红 | 由记录施加出来的那一版 txg 低于 θ_high；管理员回退复活的单元；第一个文件版本上 mkfs 的两个单元；映射换位的源槽 | 按代码与 D16 的回收谓词逐形推 | 每个被走的版本 txg ≥ θ_high，走到的单元在最新根里都计入 E(θ_high) |
| Q3-1 | 第 0 代根那一种让每种可能的环都红；CPU 与 GPU 对第 0 代根的实例号、树表记法不同 | 读 `crates/singlefs-checker/src/walk.rs:9022-9043` 与 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1871-1887`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1606-1611` | 两边同记；真环仍在枚举里 |
| Q4-2 | 推空不计 B、B 记法与抬 F 那一串不同；能住的最大 txg 低于 F 的槽滤掉与 E173 C并 不同；推完之后不抬 | 按代码与 E173 页第 359 行 | 都对上（C并 的滤法按「照今天」那一臂推的，没回读 E173 原登记） |
| Q4-3 | 写行之后、暖机之前就算上限；挂载续做在暖机之前；会话路径上某块盘只剩第 0 代根 | 读 `establish_instance` 那一段次序与三个调用处 | 都在暖机之后 |
| Q5 | 离线可用的安全计数库 | `Cargo.lock` 与本机 cargo 缓存 | 只有 `libc` |

## 这条腿自己的限度

- 没跑任何东西：除了只读命令与 `cite-check.py`，读数全是推的；唯一的模型没跑、没有参照输出。Q3-2 那张最小历史是照式子手算的，没在真写者上走一遍、没造镜像。
- Q3-2 的模型里 checker 那一栏按构造等于不并的式子，它不独立证明 checker 判不出，只量这一形在三种 S 下多常见。
- Q1-2 的镜像是构造法，依赖新池新建文件那张镜像上有一对紧挨的「已释放、未释放」记录，没核。
- Q3-1 第二轮的四个数没复核。
- Q4-4 只读了故障注入的认法与「跨重挂」那一句，没核故障注入的历史里被吞的根槽写之后真的会重挂、会抬 F。
- 没读第一样本与正推腿的报告（禁读），也没读第二轮两腿报告的正文，只读了第二轮判决。

## 要主 agent 跑的命令

1. 模型复跑（不重型，纯 Python）。在仓根起：

   `task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r3-opus-s2-model -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r3-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r3-snapshot --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r3-opus-s2/rerun-1`

   看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里三行 `E7RESULT name=zero_slot_merge_blind S=…`（每行 `drop_writer_exceeds_the_literal_ceiling` 大于 0 就是 Q3-2 那一形在那个 S 下取得到）、一行 `E7RESULT name=first_example`、一行 `name=rerun_compare … result=no_reference_in_the_model_directory`（第一次跑没有参照）。`--frozen` 只是给模板的必填参数，这个模型不读它。跑完把 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r3-opus-s2/rerun-1/out/q3_zero_slot_merge_blind.txt` 拷进模型目录的 `out/` 当参照，再跑一次应打 `result=_output_identical`。读数要接着解读的，新派同立场的腿带「承接：」。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没造任何坏镜像、没改 `crates/`；改法都只在推演上、被攻过零轮。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r3-opus-s2` 里没建仓副本、编译目录与库拷贝，不用删；复跑的 `rerun-1/` 由主 agent 的那一次跑建。
