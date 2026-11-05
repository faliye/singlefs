# m4-admission-delta-r2 攻方腿报告（Opus，2026-11-05）

攻击面：第二轮正文第五节分给攻方的三样——第一轮的修补本身（B2 补上固定点之后、B3、B4、B5）/ 产生第一轮结论的方法（格六）/ 从未被看过的地方（墓碑树退役树整份照抄、`unit()` 线性找根头、聚簇段求和，B2–B5 各罩不罩得到）。不重攻第一轮已坐实的两样（A3 改拒绝次序、字面上界不是上界）。代码读冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/`，代码引文都写它的绝对路径；kb 与规则写仓内路径、kb 文件自己的行号。没读禁读清单里的那一份。不下「选哪条」的结论。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-admission-delta-r2-opus-model/`，四份文件的 sha256（`sha256sum` 量的）：

```
a9c4a8987d339a140a036566a9f65ce2d09afd093c947510c4244078c2ff459b  scans_and_copies.rs
27868c7dda2c22f9a5638fc7407ac80bee3613d33469da6dee913938866506a0  settle_round_placements.rs
2b4f419a8afb05faeb91085d59632a0864dd92f43065ed89e2b8c1af68d23fcb  tools/apply_settle_trace.py
8e9e83b841bddf10c0473e7328942b7e729c1839446f632b3f8ceab1c374c92a  tools/rerun.sh
```

- `scans_and_copies.rs`：harness 档测试目标（不进 checker 档包），四条用例，盘是内存稀疏盘，在今天的代码上量：根头在 `units` 里排在第几项、墓碑条数随 unlink 次数怎么涨、取用户数据落点要走过几个候选与这次挂载开过几个聚簇段、拷一份分配器随单元区槽数要多久。
- `settle_round_placements.rs`：harness 档测试目标，一条用例。跑之前 `tools/apply_settle_trace.py` 只给副本的 `settle_the_allocation_record_tree` 套上逐轮记录（三处插记录、文件末尾加一个只读出的模块，不改任何判定），数固定点相邻两个取落点的轮之间同一个角色换没换槽、新进来的角色排没排在前一轮已有角色前面。
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：拷冻结副本（连 `prune-server`）、补仓里的 `.cargo` 与 `.claude/heavy-thread-default`，先在今天的代码上跑第一个目标，再套记录、跑第二个目标，打出 `name=` 行与用例结局，删掉拷出的树。换了 `rerun_steps` 之后模板自带的 `--selftest` 不再成立（它造的假冻结副本里起不了 cargo），只用下面那条真跑。
- 复跑命令照「要主 agent 跑的命令」一节。这一腿一条命令都没交、模型一个都没编过：报告里凡是模型的数都写「推的，待主 agent 跑」。
- 崩溃状态：两份模型都不枚举崩溃状态。它们数的是今天代码在准入那一段走过的东西（项数、条数、槽号、挂钟），不判任何一版落盘之后读回什么，历史里没有一次崩溃。定义第 3b 步「每段历史的崩溃状态一个不落」按这一点没照做，偏离写在这里交主 agent 判；状态数 0 个（没调枚举函数）。
- 估时（推的，没跑）：两次编译各约 5–10 分钟（第二次只重编 singlefs-core 往上），第一个目标里最慢的是 256 GiB 单元区那一个池与 96 次建再删，合计估 2–8 分钟；第二个目标不到 1 分钟。合计远低于 40 分钟，没缩取样。

## 各格判定一览

| 编号 | 打的是哪条 / 哪一格 | 判定 | 分不分辨候选 | 依据的种类 |
|---|---|---|---|---|
| G1 | B2 的固定点（格一）、B2 与 B3 的分界（格二） | 打中：B2 定义里「每一轮只处理这一轮新进出的分配记录」两种读法——每轮从同一个底子重放全部释放与取落点、只是不整份拷（那就是 B3），或在上一轮的预演上只补新进出的那几条（那与今天不逐个相同：后一轮新进来的分配记录树节点在 bump 次序里排在上一轮已取过槽的记账树、中央映射树、树表节点前面，后者的槽跟着挪） | 分辨 B2 与 B3 | 代码现查；模型 `settle_round_placements.rs` 数挪了几个（推的，待主 agent 跑） |
| G2 | B2、B3 的固定点（格一、格二），连带 B4 | 打中：固定点每一轮的预演里，取落点与释放都在分配器本体的线性结构上找：用户数据从单元区起点两槽一步往上找、每个候选再把这次挂载开过的聚簇段逐个比一遍；记一条分配要在全部分配记录上过滤一遍；释放与释放判定按 (盘, 槽) 在全部记录上线性找。这几样不在「拷不拷分配器」里，差量视图下照样走底子，B2、B3 的定义都没罩 | 不分辨 B2、B3（两条都中）；B4 照定义留着它 | 代码现查；模型 `scans_and_copies.rs` 第三条量候选数与挂钟（推的，待主 agent 跑） |
| G3 | B3 的差量视图（格二） | 线索：差量视图要跟着改的东西有十几样（见第二节）；它与本体不一致时，唯一会叫的是真发时那一句断言，它只比叶的集合罩不罩得住，槽号不同而叶相同的不一致谁都看不见 | 只关 B3 | 代码现查 |
| G4 | B4 改射程（格三） | 打中：B4 改的不止「准入之前」那半句。射程后半句只把「准入之后装单元、拷字节」排出运行时决策路径，两道之间的固定点是分配预演、释放判定与快照生死判定，三样都不是装单元、拷字节，fs-design 那张表又把分配与生命周期判定点名成运行时决策路径；照 B4 做完，这一段要另改后半句才不算违条 | 只关 B4 | kb 与规则现查 |
| G5 | B5 的代价（格四） | 打中：B5 上界放不下时走今天整条路，放不下的那一带至少宽「K1 − 这次真写的分配记录树节点数」，随单元区线性涨；落进这一带的每一次写准入之前照今天随总数涨。上界自己还要现算 ckpt_cost 与普通分配那一半，那两样今天就随总数涨（G7、G8） | 只关 B5 | 代码现查；带宽用第一轮复跑的 K1 |
| G6 | B5 的断言（格四） | 条件式：断言合规只在上界对一切走得到的输入（含从盘上重建的形状、只供测试的压小容量）都成立时；上界若照搬 ckpt_cost 里「空发布」那一份中央映射树式子，写入类发布会把它超过（第一轮第二次抽样 H3 的形状），那就是走得到的输入撞断言 | 只关 B5 | 代码现查（推的） |
| G7 | 从未被看过的地方：`unit()` 线性找根头 | 打中：落点那一判的参数与式子那一判的读数各现算一次 ckpt_cost，每次按角色在 `units` 上线性找分配记录树根、extent 上段根、extent 下段末节点、中央映射树根，`units` 按 bump 次序把每个数据单元（连第一个文件之外的文件的）排在最前；落点那一判之前每次发布至少走两遍全部数据单元 | B4 必须修，B2、B3、B5 的定义都没罩 | 代码现查；模型第一条量下标（推的，待主 agent 跑） |
| G8 | 从未被看过的地方：墓碑树、退役树、谱系树整份照抄 | 打中：普通分配那一半（落点那一判之前调一次）与固定点每一轮的 `resolve`（再调一次）都把上一版全部墓碑容器、全部退役容器的内容逐片 `clone`；`resolve` 里谱系容器另拷一遍。墓碑第一版不回收，所以每次发布在两道准入之前拷的条数随 unlink 过的总数涨 | B4 只修得到落点那一判之前那一次，B2、B3、B5 的定义都没罩 | 代码现查；模型第二条量条数（推的，待主 agent 跑） |
| G9 | 从未被看过的地方：聚簇段求和 | 打中一半：落点那一判逐个走这次挂载开过的聚簇段（第一轮 H8 报过口径那一半）；这一轮另报同一个集合在取用户数据落点时被每个候选再走一遍（G2），那一处乘上了被占的槽数 | 四条都不罩 | 代码现查；模型第三条 |
| G10 | 格五 记账树台阶行 | 有做法、不动 D8（核心索引结构） 已定项 1：求值的代价可以降到这次碰到的路径（节点数随一版带着、规划只走这次删插的十几行与新台阶那一条路）；要让 ckpt_cost 那一项的值本身不随台阶行涨，改的是 D28（挂载期承诺量） 已定项 4「记账树每发布的节点数」的读法 | 不分辨（四条共用） | kb 与代码现查（推的） |
| G11 | 格六 第一轮的方法 | 没打中：第一轮 A3 的两处打中（拒绝换种类、字面上界被超）都是按构造给的反例，不靠那 5×5 的取样；「拒绝次序与今天相同、上界又是真上界」的 A3 变体就是 B5 本身（上界只当放行快路，放不下回今天），G5 那一带的代价它照样有 | — | 代码与第一轮模型现查 |

没打中的角度与取样范围在「没打中的形状」一节；各候选一行代价量级在「代价量级」一节。

## 一、攻击面一：第一轮的修补本身

### G1　B2 的「每一轮只处理新进出的那几条」：要么就是 B3，要么与今天不逐个相同（格一、格二）

今天的固定点每一轮都从分配器的一份新拷贝起：「let mut rehearsal = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13098`），把这次的释放全做一遍，再按 `resolved.rewritten_roles` 的次序给每个角色重新取一遍落点：「            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13117`）。次序是 bump 次序，分配记录树的节点在固定点那几族里排最前：「是罩住它们的那一版 bump 次序里第一个取不到的（分配记录树的节点排在前面）」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13103`），`units` 的说明同一个次序：「固定点那几族（分配记录树、记账树、中央映射树各连同根之下的节点，加树表单元）」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:3206`）。

B2 的定义是「分配记录树的固定点每一轮只处理这一轮新进出的分配记录与它们碰到的叶，不克隆整份分配器」（正文第三节 B2 那一行，`research/prompts/_m4-admission-delta-r2-body.md:26`）。这句话有两种做法：

| 读法 | 怎么做 | 与今天逐个相同吗 |
|---|---|---|
| 甲 | 每一轮仍从同一个底子起，把这次全部释放与全部角色重放一遍，只是不整份拷：改动记在一层只记「这次取走与放回的槽」的视图里，轮末丢掉 | 相同（视图与拷贝语义一样时）。可这就是 B3 的定义「固定点里要预演的分配走分配器的差量视图（只记这一次取走与放回的槽），不拷整份分配器」（`research/prompts/_m4-admission-delta-r2-body.md:27`），B3 比 B2 多省下的是零 |
| 乙 | 第 k + 1 轮接着第 k 轮的预演走，只给这一轮新进来的角色补取落点、只释放新换下的 | 不相同：第 k + 1 轮新并进重写集的分配记录树节点，在今天的次序里排在第 k 轮已经取过槽的记账树、中央映射树、树表节点前面；今天从新拷贝重取，后面那几个的槽整串往后挪，乙只把新进来的接在末尾，同一个角色落在另一个槽上 |

乙下的结局与今天不同的是：这次重写的每个固定点角色落在哪个槽（盘上字节跟着不同）、分配记录里哪几片叶变了（重写集跟着可能不同）。真发时 `publish_admitted` 照今天的次序从头取落点，再用断言核真发的记录与固定点说的是不是同一棵树：「准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15225`）。乙要过这一句，真发也得改成乙的次序；不改，叶集合一不同就撞这一句断言。

最小反例（形状，推的，待主 agent 跑模型坐实）：两块内存盘、单元区 16384 槽，第一个文件之后覆盖写一个单元。我推的轮次（按 `settle_the_allocation_record_tree` 的注释「产品路径上三四轮就定」与上面几行代码推，没量）：第一轮只释放、没取落点就回头（释放弄脏的叶没被罩住，`continue`）；第二轮取落点，取到的槽弄脏开放段那一片叶；第三轮重写集多了那片叶与它的祖先，这几个节点排在记账树、中央映射树、树表之前取槽，排在后面的那几个取到的槽跟着变。挪了几个、挪到哪，等模型的数。模型 `settle_round_placements.rs` 在 24 次用户改动上逐次打 `name=settle_rounds … roles_shifted_between_rounds=… new_roles_placed_before_a_role_of_the_earlier_round=…`，末行 `name=settle_rounds_summary`；我推的是两个数都大于 0。若两个数都是 0（每次都只有一个取落点的轮、或新角色总在末尾），乙就与今天相同，G1 的乙那一半不成立。

四句：
- 分不分辨候选：分辨 B2 与 B3。按甲读，B2 = B3；按乙读，B2 的结局与今天不同、B3 不受影响。B4、B5 不动固定点，不中。
- 被判的系统当时看不看得到判别它的东西：看得到，两轮的角色清单与取到的槽都在预演手里。
- 满足的是判据字面的哪一个分句：格一「与今天整份比对的结局逐个相同吗（收敛到同一个固定点、轮数上限 64 照旧够）」里的「逐个相同」。
- 改法在这一格上还中不中：把 B2 的定义写成甲，这一格不中，代价是 B2 与 B3 合成一条；写成乙，要另证乙的落点也合法、并把真发改成同一个次序，那是改了今天的落点规则，不再是「只换算法」。

能不能做成会红的东西：能。模型那一条就是计数；要做成会红的测试，可以在副本里另写一个乙那样的固定点，与今天的结局逐个比，今天的形状上就红。没写这一份（只数了今天的轮间挪动）。

### G2　B2、B3 都没罩到分配器本体里的线性找（格一、格二，连带 B4）

固定点每一轮的预演除了那一次整份拷，还在分配器本体（拷贝出来的那一份，与本体同一套结构）上做三样线性的找：

1. 取用户数据落点：每块盘从单元区起点两槽一步往上找，「let mut candidate = self.unit_area_start.slot().0;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:971`），每个候选再把这次挂载开过的全部聚簇段逐个比：「let inside_cluster_segment = cluster_segments.iter().any(|segment| {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:974`）。一次取落点 Ω(第一对空槽之下被占的槽对 × 这次挂载开过的聚簇段)。聚簇段集合挂载里只增不减：「这次挂载开过的聚簇段的起点，开过就一直算数」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1872`）。开新段时另有一遍从头扫全部段：「let full_segments = self.allocated.len() / per_segment;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1004`）起的那一段。
2. 记一条分配：每块盘在全部分配记录上过滤一遍找相交的：「let intersecting_record_slots: Vec<SlotNumber> = records」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1152`），记录表是一张平的 `Vec`：「    records: Vec<AllocationRecord>,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1877`）。一个角色 Ω(分配记录总数)。
3. 释放与释放判定：释放时每块盘线性找那条记录「.find(|record| record.device == device.device && record.slot == placement.slot)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:2265`）；固定点每一轮先跑的释放判定（`placements_released_by_this_publish`，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13073` 那一句）里逐盘核「.record_for(device, slot)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:5448`），它也是线性找：「.find(|record| record.device == device && record.slot == slot)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:2211`）。

这三样都在式子那一判之前、每一轮各走一遍，代价随分配记录总数（第 2、3 样）与被占的槽、开过的聚簇段（第 1 样）涨。第一轮攻方 H4 只报了整份拷与按叶分组两遍，没报这三样。

B2 的定义管「不克隆整份分配器」与「只处理新进出的分配记录与它们碰到的叶」；B3 的定义管「只记这一次取走与放回的槽」。两条都没说取落点怎么找、记录怎么查：差量视图只记这次改了什么，要取下一个落点、要查一条记录，照样落到底子上那张位图从头扫、那张 `Vec` 逐条找。所以照字面做完 B2 或 B3，这三样原样留在两道准入之间。B4 不动两道之间，留着。B5 上界放得下时把它们挪到准入之后，放不下时照今天（G5）。

最小历史（推的，待主 agent 跑模型）：两块内存盘、单元区 16384 槽，第一个文件之后建一个 inode、给它顺序写 B 个数据单元（压在单元区底下、一直活着），再写一个单元。模型第三条在 B = 0、128、400、1200 上打 `name=lowest_free_pair_search … candidates_walked_before_the_free_pair=… cluster_segments_opened_in_the_mount=… one_user_data_placement_ns_median=…`，断言候选数随 B 严格涨；另在同一次挂载里连发 8、64、256 次覆盖写，打 `name=cluster_segments_in_one_mount`。我推的是候选数约等于 B 加常数、聚簇段数随覆盖写次数涨、取一次落点的挂钟跟着候选数涨。

四句：
- 分不分辨候选：不分辨 B2 与 B3（两条都中）；B4 照它的定义本来就留着两道之间；只有另加「分配器本体按槽索引」那一件才不中。
- 看不看得到：看得到，位图、聚簇段集合、记录表都在手里，换成按段的空闲摘要、按起点排序的聚簇段集合（`range` 一次查）与按 (盘, 槽) 的记录索引，每一次找都降到对数级。
- 满足的是判据字面的哪一个分句：D8（核心索引结构） 已定项 16 射程「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:516`）里的「记录总数」；第 1 样另随被占的槽涨，也碰 fs-design 那一格的「不许随盘容量增长」。
- 改法：见第五节改法表的「B6」一行（攻方提，零轮）。

能不能做成会红的测试：能。模型第三条的断言反过来写——「取落点要走过的候选不随压在底下的数据单元数涨」——今天红，B2、B3 照定义做完照样红。

### G3　B3 的差量视图要跟着改什么，不一致时谁先叫（格二，线索）

预演里会动分配器的四件事（释放、取用户数据落点、取提交内生块落点、搬迁取目的地，`settle_the_allocation_record_tree` 里那几行）各自改这些东西（按 `allocator.rs` 的字段读，推的，没逐个跑）：

| 视图要叠一层的 | 谁改它 |
|---|---|
| 每槽三张位图：已分配、影子账隔离、抬 F 扣住（「    allocated: Vec<bool>,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:313`）、「    isolated: Vec<bool>,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:336`）、「    held_until_floor_takes_effect: Vec<bool>,」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:344`）） | 取落点置已分配；释放改 defer 计数；只供测试的复用窗口置 0 时释放当场回收 |
| 按段的五张计数表与七个总数（占用、未释放、隔离、扣住、两槽没挡的槽对；已分配、空闲、defer、没挡、槽对、全空段、空闲连续段） | 取落点、释放、回收都按段增量改 |
| 分配记录表（改写成已释放、新增、从已回收改写、删相交的已回收记录）与已回收集合 | 释放、取落点（`make_room_for_record_on_device`） |
| 开放段与 bump 游标、这次挂载开过的聚簇段（「self.cluster_segments.insert(segment_start);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:2484`））、回落计数 | 取提交内生块落点开新段、回落 |
| 整理意图的排他区（只读）、隔离（预演不做，真发时读盘核出对不上的那一份留在已分配） | 预演只读；真发才改 |

取落点那几个找（G2）还要穿过这一层看底子：视图里刚置上的已分配位、刚开的段都要被下一次找看见，否则同一轮里两个角色会取到同一个槽。

不一致谁先看见：真发时那一句断言（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:15225` 那一行的消息）比的是两样：真发后内容变了的叶都在固定点的重写集里、装着记录的节点集合等于固定点说的那一版。视图与本体在某个角色的槽号上不一致、而叶集合恰好相同时，两边都不叫，盘上写的是真发那一份，固定点那一份只是没用上；叶集合不同才在那一句上 panic。这一句在 `publish_admitted` 里，准入两判之后（是不是已经发了写，我没查到底，推的是还没有）。这一格我没造出不一致的历史，只列出要维护的面，交下一轮或实现时核。

B3 比 B2 多省下的（量级，推的）：按甲读 B2 = B3，零；按乙读，B3 省的是每一轮那一次整份拷——按类型宽度算每块盘每槽约 3.6 字节（三张逐槽一字节的位图加五张每 64 槽一个 u64 的表），两块 1 TiB 盘一轮约 0.5 GiB，再加全部分配记录（满盘时每块盘约 2^25 条）。模型第四条量的是同一件事随单元区怎么涨（2^14 到 2^24 槽四档，打 `name=allocator_copy_by_unit_area … allocator_clone_ns_median=…`，推的，待主 agent 跑）。G2 那三样它省不下。

### G4　B4 改的不止「准入之前」那半句（格三）

射程那一句整句是「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:516`）。后半句排出运行时决策路径的只有「装单元、拷字节」。用户 2026-10-31 的原话是「  - **用户原话**：条款读法「管，只管准入之前」；排法「五处加准入现在改，根源先量」。」（`.claude/kb/decisions-history.md:3514`）：原话没说准入是哪一判，也没说两道之间算哪一边。

B4 把「准入之前」读成「落点判之前」，于是两道之间那一段落进「准入之后」。可那一段不是装单元、拷字节：

- 它是分配：固定点在分配器拷贝上把释放与取落点走一遍，定这次写哪几个节点、分配记录树重写哪几片叶；
- 它是生命周期判定：`resolve` 收 `snapshot_lifetime_judgement`，按快照生死定换下的用户可见单元进 deadlist 还是释放；
- 它出拒绝：多层码 2 树拒、释放判定的错、固定点走不完、记录超上限都在这一段交回（第一轮判决 A3 那一格坐实过这几种的次序）。

fs-design 那张表把分配与生命周期判定点名为运行时决策路径：「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」（`.claude/rules/fs-design.md:23`）。D8 已定项 16 那一句只把这一格「读成只管准入之前」，没把分配与生命周期判定挪出这一格。所以照 B4 做完，两道之间那一段既不在「准入之前」、也不在「装单元、拷字节」里，仍被 fs-design 那一格管着、仍不合。要让 B4 合条款，得同时把后半句改成「准入之后的一切不算」或「落点判之后的规划与预演不算」——那是第二处条款变更，B4 的定义只写了第一处。

用户那句话够不够得着两道之间：判不出。用户定那一句时的案子是判决 m4-close-code-r1 T7 的容器小树代价（`ancestors_of_containers` 按全部容器数开标记表），容器树的规划今天在普通分配那一半里，落点那一判之前、固定点每一轮的 `resolve` 里各走一次（G8 同一处），两种读法下它都在「准入之前」，所以那个案子不分辨两种读法。

按 B4 做完两道之间的代价（量级，推的；模型第三、四条量随参数怎么涨）：每一轮 Ω(单元区槽数 + 分配记录总数) 的整份拷，加每个数据角色 Ω(被占槽对 × 开过的聚簇段) 的取落点、每个角色 Ω(分配记录总数) 的记一条、每次释放 Ω(分配记录总数) 的找，三四轮。两块 1 TiB 盘写满时，一次一个单元的覆盖写在两道之间约拷 2–2.5 GiB 每轮、过滤记录约 10^8 条每个角色（按类型宽度与满盘记录数推，没量）。

四句：不分辨（B4 一条的事）；看得到（条款文字）；满足的是格三「这是条款变更」那一问，另加「要改两处」；改法——B4 的定义补上第二处条款变更，或者不改条款、把两道之间也做成差量（那就是 B2/B3 加 G2 那一件）。

### G5　B5 放不下的那一带照今天，带宽随单元区线性涨；上界自己也要先算随总数涨的两样（格四）

B5 的定义：「准入先判一个按差量算的上界，上界放不下才走今天的全量规划与两道判；上界放得下时照样做全量规划，用断言核全量结果不超上界。拒绝的次序与种类保持今天的样子」（`research/prompts/_m4-admission-delta-r2-body.md:29`）。

1. 上界要罩得住今天两判各自的需求，结局才逐个相同。落点那一判今天的固定点那一截已经拿 ckpt_cost 顶（分配记录树那一项是 K1 整棵树，「一次发布改几片叶的不动点收不了口，拿得准的上界只有几何上整棵树。代价随单元区线性涨（两块 4 GiB 盘 529 个节点）。」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1193`））。B5 的上界若比它小，上界放得下而今天落点那一判不够时，今天推抬 F、B5 直接走下去——拒成哪一种就不同了（多放行，不是「相同或更严」）。所以 B5 的落点上界至少是今天那一份：普通分配那一半加 ckpt_cost。式子那一判这一侧，分配记录树能拿得准的只有 K1（第一轮两次抽样都坐实了这一点），于是需求上界里再含一份 K1，可用那一侧的保留池里已经扣过一份。
2. 放不下的那一带：可用落在 [今天的真需求, B5 的上界) 里的每一次带普通分配的发布，B5 走今天整条路。带宽至少 K1 − 这次真写的分配记录树节点数。第一轮复跑（任务 6265）打过 K1：「name=k1_band geometry=2x4GiB slots_per_device=262144 root_level=2 … k1=529 … band_blocks_at_least=512 band_per_device_mib=8.0」（`research/prompts/m4-admission-delta-r1-main-verification.md:12`）；第一轮攻方报告按同一个式子算的两块 1 TiB 盘 K1 是 166159、每盘约 2.5 GiB（算的，第一轮那一份）。盘越大这一带越宽，而盘快满时每一次写都落在带里。D8（核心索引结构） 已定项 16 射程那一句没有「通常情况下」的口子：带里那几次照今天，准入之前仍随总数涨。
3. 上界本身要现算的：普通分配那一半（`ordinary_allocation_part`，G8 的整份照抄在它里面）与 ckpt_cost（G7 的线性找根头、格五的记账树全量规划在它里面）。这两样今天就在落点那一判之前、随总数涨，B5 的定义没说它们怎么变成差量。
4. 总代价：放得下时 = 上界 + 今天的全量规划（挪到准入之后）；放不下时 = 上界 + 今天整条路。两种都 ≥ 今天。它省下的只是「放得下时，两道之间那一段不算准入之前」。

四句：
- 分不分辨候选：只关 B5。
- 看不看得到：看得到，可用与上界都在准入那一刻算得出；带里那一次 B5 自己知道自己走了慢路。
- 满足的是判据字面的哪一个分句：D8 已定项 16 射程「准入之前的步代价不随……涨」，在带里那几次上不成立。
- 改法：慢路也做成差量（就是 B2/B3 加 G2 那一件），这一格不中；那时上界那一判多余。

能不能做成会红的测试：能，与第一轮 H2 那条钉今天行为的用例同形——窄池上把可用压进那一带，数这一次发布准入之前做了几次整份拷（或走了几个记录）；B5 照定义实现之后照样不为零。没写。

### G6　B5 的断言：合规只在上界对一切走得到的输入都成立时（格四，条件式）

`.claude/singlefs-ai-sop/rules/code-discipline.md` 那一节：「- **不变量被破坏是 bug，用断言**，不带着坏状态往下跑、更不往盘上写。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:150`），前面一条是「（I/O 错、盘上数据损坏、资源不够）」走 `Result`（`.claude/singlefs-ai-sop/rules/code-discipline.md:145`）。

- 断言「全量结果不超上界」是不变量，只要上界是对的：分配记录树取 K1（几何上整棵树，任何形状都罩得住），码 2 树取逐层「min(这一层节点数, 删的 + 插的) + 切出的 + 长高的」那一类不看形状的式子，就成立，panic 只在代码写错时出。
- 不成立的一种：上界照搬 ckpt_cost 里中央映射树那一项。那一项只数空发布会改的 key（代码注释「一次空发布在中央映射树里改的条目是这两棵树的节点」），写入类发布另插这次每个新数据单元、extent 与 inode 节点的 key；第一轮第二次抽样 H3 推过这一形（压小容量下顺序写 40、85 个单元，推的，那一腿模型的那一格主 agent 判决里没单列读数）。那时超上界是一条普通的大写走得到的，断言就打在走得到的输入上，按上面那一节应当走 `Result`、不是 panic。
- 「从盘上重建的树」那一类：形状由盘上读来（D8（核心索引结构） 已定项 11 那几种拒之外的都收），上界式子若依赖「节点都至少半满」之类规划器自己造的性质，重建出来的形状就可能不满足。我没核规划器在重建的形状上有没有这种依赖，只列出要核的面。
- 降级（断言打中时回今天的路）比 panic 合规：结局与今天相同、代价照 G5 的带。B5 的定义说「用断言核」，没说哪一种。

四句：只关 B5；看得到；满足格四「合不合 code-discipline」；改法——定义写明上界式子不看形状、写入类发布另数插入的 key，或打中时降级。

### G10　格五：记账树台阶行不动 D8（核心索引结构） 已定项 1 也有做法

D8 已定项 1 管的是行留不留：「整批换代：每条根的记账树只装这条根那一代的行，不留多代（除 inode 号水位的台阶行随根保留外，D5（快照 / 空间记账机制） 已定项 2）」（`.claude/kb/decisions/08-核心索引结构.md:32`）。台阶行随根保留、随历史涨，这是它定下的，不改它台阶行就照涨。ckpt_cost 那一项随台阶行涨的有两件，分开看：

1. 求值的代价。今天逐行过滤全部记账行「.filter(|entry| entry.generation == version_txg)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1339`），把全部台阶行收进 key 集合「.chain(version_to_build_on.inode_number_watermark_steps())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1349`），交全量规划器、取整棵树的节点数「Ok(plan) => plan.shape.nodes().len(),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1363`）。接在后面的那次发布只删插这一代的十几行（两块盘 15 行），台阶行照抄、key 不变，至多多一行新台阶。所以「那次之后的节点数 = 这一版的节点数 + 这次删插的十几把 key 那几条路上长出与摘掉的节点数」，只走那十几条路就算得出，与今天的全量规划逐个相同（推的：码 2 规划器的重写只由删插的 key 定，第一轮攻方「没打中的形状」第 1 条按同一条走过）。要的两样：节点数随一版带着（与 `InodeRecordCounts` 同一个做法），这一代的行与最后一级台阶随一版另记一份、不在全部行上过滤（`inode_number_watermark_steps` 今天也逐行过滤「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < version_txg」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4249`））。这两样都只改实现，不动 D8 已定项 1 的哪一个字。B2 的差量规划若把「下一次发布」的这一份也算进「多层码 2 树的规划」，就罩到了；B2 的定义只说「这一次进出的 key」，没点名它（第一轮两次抽样同报）。
2. 那一项的值。ckpt_cost 取的是那次之后整棵记账树的节点数，不是那次真重写的节点数，所以保留池随台阶行涨、可用随历史缩。这一件是 D28（挂载期承诺量） 已定项 4 的口径：「ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数 + 1（树表）」（`.claude/kb/decisions/28-挂载期承诺量.md:116`）。「每发布的节点数」读成「每发布重写的」，值就降到那十几条路的节点数、不随台阶行涨；读成「每发布之后有的」就是今天。要改的是这一句的读法，不是 D8 已定项 1。改读法之前要先核：整批换代下一次空发布到底重写不重写台阶行所在的叶（代码说照抄；D8 已定项 1「整批换代」字面像整棵重写），这一点我只看了规划器，没查条款当时怎么定的。

四句：不分辨（四条候选共用，同第一轮判决「另立一笔账」那一格）；看得到（行与形状都在这一版里）；满足格五「有没有一种做法」与「要改的是哪一句」；改法如上两条，第一条只改实现、第二条改 D28 已定项 4 的读法。

## 二、攻击面二：产生第一轮结论的方法（格六）

### G11　第一轮 A3、A4 的打中不靠那 5×5 的取样；「次序相同、上界是真上界」的 A3 变体就是 B5（没打中）

- 字面上界被超：第一轮的根据是按构造的反例，根叶装满再插一把重写 3 个、式子给 1（任务 6265 打了 6 个叶容量，判决第 12 行「rewritten=3 literal_bound=1」）。一个反例就够推翻「它是上界」，`keys_times_height_sweep` 那 9975 次只是另外数了被超的频率，结论不靠它。
- 那一份取样真正撑着的是另一句：攻方自己推的「插的 key 数 × 2 × 之后的高 + 删的 key 数 × 之前的高」在 9975 次里一次没被超（`corrected_bound_exceeded=0`）。只扫了叶与内部同容量的 5 档、每档 5 段历史，这一句只算「相容」。我按规划器的代码推它是真上界：删一把只重写它那一条路（≤ 之前的高），插一把重写那一条路、每层至多切出一个、根切开长一层（≤ 2 × 之后的高）；摘空与降高只少不多。推的，没另取样。
- A3 改拒绝次序：第一轮第二次抽样只在一处挪了式子判（挪到 `settle_the_allocation_record_tree` 之前），坐实的是记录超上限那一种被换成空间拒。别的几种「推抬 F 也没用」的拒（多层码 2 树拒、固定点走不完、释放判定的错）同样只在走完固定点之后才知道，挪到哪一处之前都一样被抢先；合取 7 判不判只看有没有普通分配，第一步就知道，挪得到式子之前，不算。所以那一处模拟足够撑「A3 改了拒成哪一种」，换别的挪法不改结论（推的，读代码次序）。
- 有没有 A3 变体次序相同、上界是真上界：有，就是 B5 去掉断言那一形——上界放得下就放行、把全量规划挪到准入之后照今天的次序交回那几种拒；放不下不拒、回今天整条路。放得下时今天两判本来也都过（上界罩住两判各自的需求，G5 第 1 条），走得到的拒只剩固定点那几种与记录超上限，次序与今天同。它的代价就是 G5 那一带。我没找到第三种。

推翻 G11 的现象：有人造出一段历史，规划器一次发布的重写数超过「插 × 2 × 之后的高 + 删 × 之前的高」；或者 A3 有一种挪法，在全量规划之前就判得出多层码 2 树拒与记录条数。

## 三、攻击面三：从未被看过的地方

### G7　`unit()` 线性找根头：每次发布在落点那一判之前至少走两遍全部数据单元

`unit` 在 `units` 上线性找：「.find(|unit| unit.identity == identity)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4351`）。读树高的两个函数都经它取根单元：「let root_header = parse_index_node(&self.unit(root_role).bytes)」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4410`，同一句在 4426 行又一次）。4426 那一处取分配记录树根、extent 上段根与第一个文件下段的末节点，4410 那一处取中央映射树根。ckpt_cost 调这两样：「.position_addressed_tree_heights_read_from_the_root_node_headers()」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1221`）、「file_version.height_read_from_the_root_node_header(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1263`）。ckpt_cost 在落点那一判的参数里算一次（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12527` 那一行），式子那一判的读数里再算一次：「let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1719`）。

`units` 的次序把每个数据单元放在最前（第一个文件的，再是别的文件的，`transaction.rs` 里 `files_other_than_the_first` 那一段的说明），固定点那几族在后（G1 引的 3206 行）。所以一次找分配记录树根、一次找中央映射树根，各要先走过这一版的全部数据单元：每次发布落点那一判之前 Ω(数据单元总数)。墓碑树、退役树的根已经改成按记下的下标取（`container_tree_root_unit_positions`，注释写明「准入读它（运行时决策路径）」），这四个根头没改。

最小历史（推的，待主 agent 跑）：第一个文件之后顺序写 N 个数据单元。模型第一条在 N = 1、16、128、400 四个新池上打 `name=root_heads_behind_the_data_units data_units_in_the_version=… allocation_tree_at=… mapping_tree_at=…`，断言分配记录树根的下标不小于这一版的数据单元数、随 N 严格涨。

四句：
- 分不分辨候选：分辨。B4 要「落点判之前不随总数涨」，必须修它；B2、B3 的定义只管规划与固定点，这一步既不是规划也不在固定点里，没罩；B5 的上界要现算 ckpt_cost（G5 第 3 条），没罩。
- 看得到：看得到，修法有先例（墓碑、退役两个根已经按下标取），把这四个根的下标同样随一版记下，O(1)。
- 满足：D8 已定项 16 射程「记录总数」那一分句（数据单元是记录）；落点那一判之前，两种读法下都是「准入之前」。
- 改法：「根头按下标取」一行，与选哪条候选无关，四条都要另加。

能不能做成会红的测试：能，模型第一条把断言反过来（「根头下标不随数据单元数涨」）今天就红；或者打开 `lookup-work-counters` 那一族计数，给 `unit` 记一格走过几项。

### G8　墓碑树、退役树、谱系树整份照抄：每次发布在两道准入之前拷 (1 + 轮数) 遍

普通分配那一半里：「let tombstone_containers_before: Vec<TombstoneContainer> = previous」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8601`）、「let retirement_containers_before: Vec<RetirementContainer> = previous」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8646`），两处都是逐片「.map(|container| container.contents.clone())」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8606`，退役那一处在 8651 行），不是 unlink 意图的发布再整份交给「TombstoneTreeAfterThisPublish::carried(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8638`）。这一半在落点那一判之前调一次：「.ordinary_allocation_part(previous, trees, deletion_and_lineage)?」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12511`），固定点每一轮的 `resolve` 里再调一次：「let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8729`）。`resolve` 里谱系容器另拷一遍（同一句 `.map(|container| container.contents.clone())` 在 8939 行）。

墓碑第一版不回收（D8 已定项 16 射程「这一读法依赖墓碑第一版不回收」，`.claude/kb/decisions/08-核心索引结构.md:516` 那一行），每次 unlink 的意图追加一条；退役记录每消失一片 inode 叶容器追加一条，也只增不删。所以一次与删除无关的覆盖写，在两道准入之前拷的墓碑条数 = 历史上 unlink 过的总数，乘 (1 + 固定点轮数)。第一轮攻方 H8 指过前两处的位置，没量、没算轮数；第一轮判决把它列在 A1 那一格里。

最小历史（推的，待主 agent 跑）：反复「建一个空 inode、unlink 它」k 次，再覆盖写一次。模型第二条在 k = 1、8、32、96 上打 `name=tombstones_every_publish_copies unlinked=… tombstone_records_before_an_overwrite=… tombstone_records_carried_by_the_overwrite=…`，断言覆盖写之后墓碑条数不变（整份照抄过来）、条数随 k 严格涨。

四句：
- 分不分辨候选：分辨。B4 修得到落点那一判之前那一次（它的目标管这一次），修不到每一轮 `resolve` 里那几次（两道之间照今天）。B2 的定义是「三棵容器树的规划只读这一次进出的 key」，照抄不是规划：不是 unlink 意图的发布进出的墓碑 key 是零，规划本来就不读它们，拷的是新一版的内存态。第一轮 A2 的定义写过「上一版的形状按引用读、不收全量 key」（`research/prompts/_m4-admission-delta-r1-body.md:37`），B2 把这半句丢了，所以 B2 照字面罩不到；B3 同。B5 的上界要普通分配那一半，罩不到。
- 看得到：看得到，上一版的容器在手里，按引用借（形状那一半今天已经这样做：「上一版的形状借过来用，不先照抄一份」那一行注释），新一版的整份挪到准入之后装。
- 满足：D8 已定项 16 射程「盘上容器总数」（墓碑容器）与「记录总数」（墓碑条数）两个分句。
- 改法：「上一版的容器按引用读、新一版的照抄挪到准入之后」一句，B2、B3、B5 都要另加；B4 只在落点那一判之前做。

### G9　聚簇段求和：落点那一判走一遍，取落点时每个候选再走一遍

落点那一判的槽对数逐个走这次挂载开过的聚簇段：「let pairs_inside_the_cluster_segments: u64 = cluster_segments」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:578`）。第一轮攻方 H8 报过它，判在口径（D8 已定项 16 的「容器总数、记录总数」管不管「这次挂载开过的段数」）上没定。这一轮补一件第一轮没报的：同一个集合在取用户数据落点时被每个候选再走一遍（G2 第 1 样，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:974`），那一处是 O(被占槽对 × 段数)，乘上了被占的槽，两种口径下都随记录总数涨。

四句：不分辨（四条候选的定义都没提）；看得到（集合按起点排序，`BTreeSet` 上一次 `range` 就判得出候选在不在某一段里；求和那一处改成随开段、关段增量维护）；满足「记录总数」（取落点那一处）与口径未定（求和那一处）；改法与选哪条候选无关。

### 没点名的几处，B2–B5 各罩不罩得到

「罩」= 照候选的定义字面做完，这一处不再随总数涨。

| 地方 | 在哪一段 | B2 | B3 | B4 | B5 |
|---|---|---|---|---|---|
| 墓碑、退役、谱系容器整份照抄（G8） | 落点判之前一次，两道之间每轮一次 | 不罩（照抄不是规划，定义丢了「按引用读」） | 不罩 | 罩落点判之前那一次；两道之间照今天 | 不罩（上界要普通分配那一半） |
| `unit()` 线性找四个根头（G7） | 落点判参数里与式子读数里各一次 ckpt_cost | 不罩 | 不罩 | 罩（目标管得到，要另加「按下标取」） | 不罩（上界要 ckpt_cost） |
| 落点判的聚簇段求和（G9） | 落点判 | 不罩 | 不罩 | 看口径（段数算不算总数，第一轮没定） | 不罩 |
| 取用户数据落点逐对扫、每候选比全部聚簇段（G2 第 1 样） | 两道之间每轮每个数据角色 | 不罩 | 不罩（视图下照样扫底子） | 不罩（两道之间照今天） | 放得下时挪到准入之后；带里照今天 |
| 分配记录表线性过滤与找（G2 第 2、3 样） | 两道之间每轮每个角色、每次释放 | 不罩 | 不罩 | 不罩 | 同上 |
| ckpt_cost 的记账树全量规划（格五，G10） | 落点判参数里与式子读数里各一次 | 看读法（「下一次发布」的规划算不算） | 同 B2 | 罩（要另加 G10 第 1 条） | 不罩 |

### 格一另一问：L1 里 W 的「每层 2·(K + 26)」与 K1 那句注释，哪一处对

- W：「W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))」（`.claude/kb/decisions/28-挂载期承诺量.md:31`），代码里的常量「pub const DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_UNITS_BEYOND_THE_BATCH: u64 = 26;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:474`）。它是「一次删除类发布每盘新占槽数的上界」，射程自己写着这个上界没被攻过：「待删占用恒 0、W 上界、管理员回退不判式子与谱系容器进 Σ 名单、删除队列非空时挂载前把环窗口算作可动用被攻过零轮」（`.claude/kb/decisions/28-挂载期承诺量.md:42`）。2·(K + 26) 按「每条变了的记录在每块盘上至多弄脏一片叶」数，K + 26 是这次变了的记录条数；固定点自己重写的分配记录树节点各换下一份旧副本、各改一条记录，这几条不在 K + 26 里，除非它们的旧副本恰好都落在已经变了的叶里。
- K1 注释点名的第三种少扣（换下很久以前落在别处的节点）对删除类发布同样成立，所以两句放在一起，W 那一句缺一段「固定点自己引出的叶有多少」的论证；K1 那一句是保守的一边。我没造出一段让删除类发布超过 W 的历史（要让几百片叶的现行副本按一条长链散开），只能说两句对不上，判不出 W 错。
- 对格一本身：B2 若按甲读（G1）求的是精确固定点，不用任何上界，链长就是这次真要重写的节点数，代价只随这次碰到的东西涨，K1 与 W 哪一句对都不改 B2 的成败。上界只对 B5（与任何「先判上界」的变体）要紧：B5 取 W 那一类按差量的式子，得先证它；取 K1，就是 G5 那一带。
- 轮数上限 64：今天的注释只对产品路径给了单调性论证（「多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减」），只供测试的复用窗口置 0 那一格推不出来（「只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13007`））。B2 按甲读不改轮的走法，64 照旧；按乙读走法变了，单调性要重证。没量。

## 四、改法各修哪一格

全是推的（按代码推、没实现没跑），只在我的推理上成立、被攻过零轮；量过的一格都没有（模型量的是今天的代码，不是改法）。

新候选 **B6 差量规划连可索引的分配器**（攻方提，零轮）：B3 之外另加五件——(1) 分配器本体的取落点改成按段的空闲摘要找（不从单元区起点逐对扫），聚簇段集合按起点查一次（`range`）不逐个比；(2) 分配记录按 (盘, 槽) 建索引，记一条、释放、释放判定都按索引找；(3) 四个根头的下标随一版记下（照墓碑、退役两个根的先例）；(4) 上一版的墓碑、退役、谱系容器按引用读，新一版的整份照抄挪到两道准入之后装；(5) ckpt_cost 的记账树一项按 G10 第 1 条只走删插的那几条路、节点数随一版带着。与 B3 的区别就是这五件；与 B2 的区别是这五件加 G1 的甲读法。

| 改法 | G1 | G2 | G5 | G7 | G8 | G9 | G10 |
|---|---|---|---|---|---|---|---|
| B2 定义写成甲读法（每轮从同一底子重放、视图不整份拷） | 推的：修（与 B3 合一） | 推的：不修 | — | 推的：不碰 | 推的：不碰 | 推的：不碰 | 推的：不碰 |
| B6（上面五件加 B3） | 推的：修 | 推的：修 | 推的：绕开（不走上界） | 推的：修 | 推的：修 | 推的：取落点那一处修，求和那一处看口径 | 推的：修求值代价，值照旧 |
| 只加「根头按下标取」 | — | — | — | 推的：修 | — | — | — |
| 只加「容器按引用读、照抄挪到准入之后」 | — | — | — | — | 推的：修 | — | — |
| B4 的定义补上第二处条款变更（射程后半句改成「落点判之后的规划与预演不算」） | — | 推的：绕开（改条款） | — | — | 推的：两道之间那几次绕开 | — | — |
| B5 的慢路也做成差量 | — | 推的：要先有 B6 的 (1)(2) | 推的：修（那时上界多余） | — | — | — | — |
| D28（挂载期承诺量） 已定项 4 的「记账树每发布的节点数」读成「每发布重写的」 | — | — | — | — | — | — | 推的：修值，求值代价不碰 |

## 五、各候选的代价量级（一行一条，全是推的）

记号：S = 每块盘单元区槽数，R = 分配记录总数，U = 这一版数据单元总数，T = 墓碑与退役记录总数，G = 这次挂载开过的聚簇段数，P = 第一对空槽之下被占的槽对数，r = 固定点轮数（产品路径三四轮），k = 这次进出的 key 与角色数，h = 树高。

| 候选 | 两道准入都判完之前，一次单单元覆盖写的代价 | 别的 |
|---|---|---|
| B1 今天 | Ω(U)（G7）+ (1 + r)·Ω(T)（G8）+ Ω(台阶行)（G10）+ r·[Ω(S + R) 整份拷 + 每个数据角色 Ω(P·G) + 每个角色与每次释放 Ω(R) + 全量规划] | 基线；两块 1 TiB 盘写满时一轮约拷 2–2.5 GiB、每个角色过滤约 10^8 条记录 |
| B2（照定义） | 甲读：同 B3；乙读：去掉整份拷，其余照 B1，结局与今天不同（G1） | 改规划器（第一轮正推那一格） |
| B3 | B1 去掉 r·Ω(S + R) 的整份拷；G7、G8、G10 与 r·[Ω(P·G) + Ω(R)] 照旧 | 视图要叠十几样状态（G3） |
| B4 | 落点判之前降到 O(k·h)（要另加根头下标、容器按引用、记账树差量三件才做得到）；两道之间照 B1 | 改 D8 已定项 16 两处（G4） |
| B5 | 放得下：上界（含 G7、G8、G10 那几样，除非另改）；放不下（带宽 ≥ K1 − 真写数，两块 1 TiB 盘每盘约 2.5 GiB）：上界 + B1 | 总工作 ≥ B1；断言的合规看上界式子（G6） |
| B6（攻方提，零轮） | O(k·h·r·log R)，加按段摘要找空槽 O(log S)（推的式子，没实现） | 分配器两个新索引要随每次分配、释放、回收增量维护，checker 那一侧要另有不共享的重算（fs-design 那一节「运行时与 checker 必须用不同的算法」） |

## 六、要主 agent 跑的命令

一条，不是重型（两个 harness 档测试目标，不碰 checker 档包，不调崩溃枚举）。我一条都没交。

1. 模型整份复跑（第 9 步）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r2-attack -- bash /home/fy5090/code/singlefs/research/prompts/m4-admission-delta-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r2-attack/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs` 起 submit，`--cwd` 是本腿草稿目录。脚本里两条 `cargo test -p singlefs-harness --test …` 各自带 `SINGLEFS_RUN_SCOPE='种类=target; …; 范围=1 个测试目标; …'`，在 `--work` 下拷出的 `tree/` 里跑，跑完 `tree/` 删掉，留 `scans.out`、`patch.out`、`settle_trace.diff`、`settle.out`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志末尾 `name=rerun_summary scans_rc=0 patch_rc=0 settle_rc=0` 与 `name=rerun_verdict all_ran=yes`；`name=settle_trace_applied edits=3`；`scans test … ok` 四行、`settle test … ok` 一行。读数：`scans name=root_heads_behind_the_data_units` 四行（G7：`allocation_tree_at` 随 `data_units_in_the_version` 涨）、`scans name=tombstones_every_publish_copies` 四行（G8）、`scans name=lowest_free_pair_search` 四行与 `scans name=cluster_segments_in_one_mount` 三行（G2、G9）、`scans name=allocator_copy_by_unit_area` 四行（G3 末段、G4 的量级）、`settle name=settle_rounds` 24 行与 `settle name=settle_rounds_summary` 一行（G1：`roles_shifted_between_rounds` 与 `new_roles_placed_before_a_role_of_the_earlier_round` 我推的都大于 0；两个都是 0 就照 G1 末段把乙那一半撤回）。
   - 编不过或某条用例红：整段 `scans.out` / `settle.out` 交承接的腿看，不当「模型不成立」。
   - 跑完新派同立场的腿，派发提示写「承接：research/prompts/m4-admission-delta-r2-opus-output.md」与任务号，读 `task-server/task log <号> --all`，把各格「推的，待主 agent 跑」换成读到的数。

## 七、没打中的形状

| 形状 | 攻的是哪条 | 为什么没打中 / 没做到 | 取样范围 |
|---|---|---|---|
| 让一次删除类发布的分配记录树新写节点超过 W 的历史 | L1（格一另一问） | 要让几百片叶的现行副本沿一条长链散开，我没造出来 | 只读代码与 D28 那两行 |
| 差量视图与本体不一致、而真发那一句断言不叫的历史 | B3（G3） | 只列了要维护的面，没写视图、没造历史 | 读 `allocator.rs` 的字段与 `settle` 那一段 |
| B2 甲读法与今天不逐个相同 | B2 | 视图语义与拷贝相同时结局相同；我找不到甲读法下会不同的一步 | 读 `settle_the_allocation_record_tree` 全函数 |
| 第一轮攻方推的「插 × 2 × 之后的高 + 删 × 之前的高」被超 | 格六 | 按规划器删插的走法推它是真上界，没另取样 | 读 `code_two_tree.rs` 的删插两段（第一轮模型 9975 次一次没超） |
| B5 放得下那一路拒成别的种类 | B5（格四） | 上界罩住两判各自的需求时，走得到的拒只剩固定点那几种与记录超上限，次序与今天同 | 读 `judge_the_publish_before_any_read_or_write` 全函数 |
| 用户 2026-10-31 那一句够得着两道之间 | B4（格三） | 当时的案子（容器小树代价）两种读法下都在准入之前，分不出 | 读变更史那一节与 m4-close-code-r1 判决 T7 两行 |
| 从盘上重建的码 2 树形状让「不看形状」的上界式子失效 | B5 的断言（G6） | 没核规划器在重建形状上的依赖 | 没读重建那一段 |
| 由用户决定的动作 | — | 模型里的历史把用户动作写死了（覆盖写、建 inode、顺序写、unlink 的固定次序）。G1、G2、G7、G8 都是「存在一段历史」的形状，一段就够，没扫；这一节的「没打中」都不依赖用户动作 | — |

以上「没打中」都只抽了这一次样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不算「没打中」。

## 八、这条腿自己的限度

- 一条命令都没跑：两份模型、补丁脚本都没编过、没跑过，可能编不过（补丁脚本只用 `python3 -m py_compile` 查过语法，`rerun.sh` 只用 `bash -n` 查过）。所有模型的数都是推的。
- 挂钟是 debug 构建、同一个测试二进制里几条用例可能并行时量的，只拿来看随参数的走向；拷贝字节、记录条数、轮数都是按类型宽度与代码推的。
- 两块 1 TiB 盘写满时的那几个量（每轮 2–2.5 GiB、每个角色 10^8 条）按 `AllocationRecord` 约 32 字节、一个数据单元两槽推，没量 `size_of`。
- 没读 `container_tree.rs` 的规划内部与 extent 树规划；没查 `publish_admitted` 里那一句断言之前有没有发写；没查 D28 已定项 4「记账树每发布的节点数」当时怎么定的。
- 冻结副本里的 `allocator.rs` 与主工作区是否逐字节相同我没比（正文第二节只点名了另几份）；引文都写冻结副本的绝对路径。

## 九、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；不下「选哪条」的结论。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在第六节）。副本上量出的数不算入库装置上的数。
- 没写 harness 用例进 `crates/`（写范围不含它）；几条「能不能做成会红的测试」交主 agent 定要不要派人写。

## 十、删了什么

- 这一腿没建仓副本与编译目录：草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r2-attack` 里只有认领留下的 `.owner` 与 `.writer-lock`（`du -sh` 量的 12K）。复跑时 `rerun.sh` 在 `rerun-1/tree/` 拷出的树由脚本自己删。
