# m4-admission-delta-r3 攻方腿报告（Opus，2026-11-05）

攻击面（第三轮正文第五节分给攻方的）：第二轮判 B2、B4、B5 打中的依据撑不撑得住 / 随总数涨的步的清单完不完整（格三）/ 前两轮报出的数能不能核（格四）；B6、B7 各找一个反例。不重攻前两轮已坐实的角度（A3 改拒绝次序、字面上界不是上界、B2 乙读法、G7–G9 三处本身）。代码读冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/`，代码引文写它的绝对路径；kb、规则与前两轮报告写仓内路径。禁读清单里那一份没读。不下「选哪条」的结论。前两轮三个复跑任务的日志（6265、6394、6553）用 `task-server/task log <号> --all` 现读，落在草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r3-attack/task<号>.log`。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-admission-delta-r3-opus-model/`，三份文件的 sha256（`sha256sum` 量的）：

```
b7a175fa7e04fbbf19b4ad045dbc9dc8a317dc270f739069c7738604f05d1b9c  index_builds_and_allocator_copies.rs
aa5b94abd48de6c49d5b1fefb88dd209a00a5c188f628f73139b34888059dfed  tools/apply_copy_counters.py
a8ca1b6a5d678ef77a7d3d5700dd48fe5f3bd022e27bb469aa2aca17a6672110  tools/rerun.sh
```

- `index_builds_and_allocator_copies.rs`：harness 档测试目标（不进 checker 档包），三条用例，盘是内存稀疏盘，搭池沿用第二轮攻方模型那一份 `common_admission_pool` 的写法：
  1. 第一个文件顺序写 1、16、128、400 个数据单元之后覆盖写一个单元，用 harness 档已开的 `lookup-work-counters` 逐处数查找（`lookup_work_while`），打每一处的 `name=lookup_work … site=…` 与一行 `name=release_judgement_indexes`（释放判定那一处进了几轮、建了几份索引、查过的索引最大几项）；
  2. 单元区 2^14、2^17、2^20 槽三个池上各做覆盖写、建 inode、给它写 3 个单元、删它，打 `name=allocator_copies`（两道准入之前那一判进了几次、判之前整份拷了几次分配器、固定点每一轮又拷了几次）；
  3. 一次建 1、8、32 个 inode，打 `name=committed_reservation_of_an_inode_creating_publish`（这一版记下的标签 6 比上一版多多少）。
- `tools/apply_copy_counters.py`：只在拷出的副本里给 `transaction.rs` 三个函数体各插一行计数、文件末尾加一个只读出的模块，不改任何判定；锚点在函数体里命中不是一次就退 3、一个字节不写。`python3 -m py_compile` 过。
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`（`diff` 核过前 76 行与末段与模板逐行相同，`bash -n` 过）；换了之后模板自带的 `--selftest` 不再成立，只用「要主 agent 跑的命令」那一条真跑。
- 这一腿一条命令都没交、模型没编过：报告里模型的数都写「推的，待主 agent 跑」。
- 崩溃状态：模型不枚举崩溃状态，数的是今天代码在两道准入之前走过的东西（拷贝次数、索引大小、标签 6 的值），历史里没有一次崩溃，不判任何一版落盘后读回什么。定义第 3b 步「每段历史的崩溃状态一个不落」按这一点没照做，偏离写在这里交主 agent 判；调枚举函数的状态数 0 个。
- 估时（推的，没跑）：编译一次 harness 档测试目标约 5–10 分钟；用例里最慢的是 400 个单元的顺序写与 2^20 槽的池，合计估 2–6 分钟。远低于 40 分钟，没缩取样。

## 各格判定一览

| 编号 | 打的是哪条 / 哪一格 | 判定 | 分不分辨候选 | 依据的种类 |
|---|---|---|---|---|
| J1 | 格三：从用户改动到式子判之间没列的步 | 打中：每次发布在 `prepare_the_version_publish_relocating_units` 之前整份拷一次分配器（`transaction.rs:12314`，失败就换回去那一份），排在两道准入之前；前两轮的表一、表二从 `prepare_the_version_publish` 起数，它在调用方里，没进任何一张表；B6、B7 的定义都不碰它 | 不分辨（B1、B6、B7 都有） | 代码现查；模型第 2 条数次数（推的，待主 agent 跑） |
| J2 | B7 的反例（某一步仍随总数涨） | 打中：固定点每一轮的释放判定在第一次查映射时，把上一版整张 `mapped_units` 与整张 `units` 各建一份按身份的索引（第一轮表二 #28 列过、没量）；B7 的五件只改分配器里的三样找、根头、容器与记账树，不碰这两份索引；另有全记录分叶、分配记录树全节点集合两样也不在五件里 | 只关 B7（B1 同样有） | 代码现查；模型第 1 条数索引大小（推的，待主 agent 跑） |
| J3 | B6 的反例（某一步仍随总数涨） | 打中：B6 的定义出处（第二轮辩方 3.1）把删除类与删快照整类排出快路，unlink 的意图、每一批、截断各批、删快照各批不论空间多宽都走今天的整条路；加上 J1 那一份拷贝排在 B6 的上界之前 | 只关 B6 | 定义与代码现查 |
| J4 | B6 的反例（放行而真实不够），条件式 | 条件式打中：正文的 B6 只写「这一次分配量的上界」，式子那一判另一侧的已承诺预留按这次之后那一版的标签 6 读；建 inode 的发布之后标签 6 涨副本数 × 76 × k 字节，上界快路若照装置的读法取上一版记下的那一份，可用落在这一截里时快路放行、今天拒 | 只关 B6 | 代码现查；模型第 3 条量差值（推的，待主 agent 跑） |
| J5 | B6 定义自己（上界逐项），条件式 | 线索：辩方 3.1 的 U_固定点 不列 deadlist 树，3.3 列了；今天落点那一判同样不数 deadlist 树的节点（注释写「落点那一判照判」）；照 3.1 做，活快照下覆盖写一个被钉住的单元，deadlist 树重写的节点只能被 K1 的余量吞掉，窄盘上余量可能为零、断言打中 | 只关 B6 | 代码现查（推的，没造出历史） |
| J6 | 「已定」格：第二轮判 B4「要改两处」 | 打中一半：「是条款变更」站得住；「要改两处」靠的是把射程后半句读成穷举的豁免单，而射程前半句字面是「只管准入之前」，照它读后半句只是举例，B4 改的只有「准入指哪一判」一处 | 只关 B4 | kb 与规则现查 |
| J7 | 「已定」格：第二轮判 B2、B5 | 没打中：B2 靠任务 6553 那一段历史一次挪动就够（存在性）；B5 的带靠构造，不靠 K1 的具体数 | — | 代码与日志现查 |
| J8 | 格四：任务 6553 三列读数 | 打中：`candidates_walked_before_the_free_pair`、`allocation_tree_at`、`tombstone_records_carried_by_the_overwrite` 三列量的是「答案在哪 / 有几条」，不是「走了多少」；判决用它们写「线性扫」「拷」是把代码读出来的那一半记成了现量。撑「线性扫」的是同一行的挂钟那一列（判决没引）；另两处没有挂钟，「拷」只有代码一条路。三列在 B7 对应那几件做完之后读数不变，做不成会红的测试 | 不分辨 | 日志与模型源码现查 |
| J9 | 格四：任务 6553 的整份拷贝挂钟 | 线索：覆盖写整次发布的挂钟随单元区涨的斜率约是一份拷贝的 1.9 倍，按代码数每次覆盖写要拷 3 份（J1 一份、固定点两轮各一份）；两者对不上，判决「克隆分配器随单元区涨」不受影响，「每轮拷一份」的次数要计数才定 | 不分辨 | 日志算术（推的）；模型第 2 条数次数 |
| J10 | 格四：任务 6265 的 `k1_band` | 打中：四行是 Python 照公式复算（不调 Rust 那个函数），单元区起点四种几何都写死成 4 GiB 那一份，「今天实写的分配记录树节点数」是假定的 17 / 25；`band_blocks_at_least` 不是下界。只有 2×4GiB 的 529 有 Rust 注释旁证。判决里「多拒一带」「随单元区线性涨」的结构结论不受影响，带宽的数要标「推的」 | 不分辨 | 模型源码与日志现查 |
| J11 | 格四：任务 6394 里判决没引的一行 | 打中：压小映射树容量（叶 4、内部 8）、顺序写 85 个单元那一格「excess_slots=14」——今天落点那一判拿 ckpt_cost 当固定点的「最坏数」，写入类发布真写的固定点超过它；第二轮攻方 G6 写的「判决里没单列读数」其实日志里有；同一份模型预言 40 个单元那一格为正，实测 −4 | 不分辨（今天就有） | 日志现查 |

没打中的角度与取样范围在「没打中的形状」一节；各候选一行代价量级在第七节。

## 一、格三：随总数涨的步的清单（J1）

### J1　两道准入之前每次发布整份拷一次分配器，前两轮的表都从它的下一层数起

正文格三的窗口是「从 `prepare_the_version_publish`（`transaction.rs:12409`）到式子判之间」。可发布路径进 `prepare_the_version_publish_relocating_units` 之前还有一层：「fn publish_version_of_trees_relocating_units_after_a_check<Device: BlockDevice, Refusal, Check>(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12294`），它第一件事是冻结判，接着「    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12313`）、「    let allocator_before_this_publish = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12314`），下一行才「    let built = prepare_the_version_publish_relocating_units(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12315`），落点那一判在「        admit_the_units_landing_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12523`），式子那一判在「        admit_on_every_device(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12621`）。

- 谁走它：`grep` 这一层的调用点，六处入口都经它（`publish_create_snapshot`、`publish_compaction_step_after_its_check`、`publish_deletion_step`、`publish_version_of_trees_holding_one_data_unit`、`publish_version_with_unmount_marker`、`publish_version_appending_a_lineage_record`，后三者经 `publish_version_of_trees` 与 `publish_version_of_trees_relocating_units` 两层转过来）。覆盖写的链：`publish_overwrite` → 「fn publish_version_holding_one_data_unit<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12100`）→ `publish_version_of_trees_holding_one_data_unit` → 「    publish_version_of_trees(」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12119`）→ 这一层。删除的意图与各批走 `publish_deletion_step`，也经它。
- 拷的是什么：`PoolAllocator` 是「#[derive(Clone, Debug)]」整份派生的拷贝（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1866`），里面每块盘逐槽的三张位图（「    allocated: Vec<bool>,」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:313`，另两张同第二轮攻方 G3 那张表）与全部分配记录（「    records: Vec<AllocationRecord>,」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:1877`）。随单元区槽数与分配记录总数涨。那一行注释的「约 450 KiB」只是一种几何上一张位图的数（211968 = 4 GiB 盘单元区 [50176, 262144) 的槽数），三张位图按类型宽度是它的三倍左右（推的），盘大了照比例涨。
- 有多贵：第二轮复跑（任务 6553）在同一个结构上量过一份拷贝的挂钟：「scans name=allocator_copy_by_unit_area unit_area_slots=16777216 unit_area_gib_per_device=256 allocation_records=116 allocator_clone_ns_median=34949849 one_unit_overwrite_publish_ns_median=71945489」（任务 6553 日志，现读）。每块盘 256 GiB 时一份约 35 ms（debug 构建）。
- 前两轮漏它的原因：第一轮正推表一的标题就是「`prepare_the_version_publish` → `prepare_the_version_publish_relocating_units`（12659）→ `judge_the_publish_before_any_read_or_write`（12489）开头到 12523」（`research/prompts/m4-admission-delta-r1-sonnet-output.md:38`），从被调的那一层数起；第二轮 G2–G4 数的是固定点里每一轮那一份（`transaction.rs:13098`）。全仓前两轮的报告与正文里 `allocator_before_this_publish` 一处都没有（`grep -l` 对 `research/prompts/m4-admission-delta-r*.md` 与三份正文，零个文件）。

四句：
- 分不分辨候选：不分辨。B6 的上界照辩方 3.1 算在「在 `judge_the_publish_before_any_read_or_write` 最前（`ordinary_allocation_part` 之前）算四个上界，纯函数、不改任何状态：」（`research/prompts/m4-admission-delta-r2-sonnet-output.md:84`），那已经在这份拷贝之后；B7 的「不拷整份分配器」说的是固定点那一份（正文 B7 定义「分配记录树固定点每轮从同一个底子重放、走分配器的差量视图，不拷整份分配器」）；B4 只动落点判之前的几样。三条都留着它。
- 看不看得到：看得到。失败要换回的只是这一次动过的那几样（两道准入都不动分配器，判完之后的释放与取落点才动），换成撤销日志或差量视图就不必整份拷；或者把这份拷贝挪到两道准入判完之后（准入拒绝的路上分配器还没动，不用换回）。
- 满足的是判据字面的哪一个分句：D8（核心索引结构） 已定项 16 射程「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（`.claude/kb/decisions/08-核心索引结构.md:516`）的前半句（分配记录是记录）；按槽的位图另碰 `.claude/rules/fs-design.md` 那一格「代价不许随盘容量增长」（「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」，`.claude/rules/fs-design.md:23`）。后半句豁免的「拷字节」在「准入之后」，这一份在之前。
- 改法在这一格上还中不中：B6、B7 照定义都还中；「这份拷贝挪到两道准入之后」一句（攻方提，零轮）修它。

能不能做成会红的东西：能。模型第 2 条（`every_publish_copies_the_whole_allocator_before_its_judgement`）数每次用户改动在判之前整份拷了几次，断言「至少一次」今天就成立；把断言反过来（「判之前整份拷贝 0 次」）今天红、B6 与 B7 照定义做完照样红，挪走那一份才绿。

### 格三另几处（对前两轮的表补的，都是推的）

| 步 | 在哪 | 随什么涨 | 前两轮列过没有 |
|---|---|---|---|
| 判之前整份拷分配器 | `transaction.rs:12314`（J1） | 单元区槽数、分配记录总数 | 没有 |
| 建 inode 的发布取起号：`inode_number_watermark` 逐行过滤全部记账行 | 「    let first_new_inode_number = previous.inode_number_watermark();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:9872`），在造计划时、`prepare_the_version_publish` 之前 | 记账行数（台阶行随历史涨） | 第二轮辩方格五提过，两份判决的清单里没有 |
| 会话走空：队列不空时整份拷现行那一版 | 「            let mut walked = current_version_of_trees.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/mounted_session.rs:2620`），排在走空的各批与这次改动的准入之前 | 整份 `TransactionOutput`（单元字节、映射、全部容器） | 没有；它是 `TransactionOutput` 的照抄，算不算进 E199（发布照抄上一版的代价） 的「准入之前照抄」我没核 |
| 释放判定每一轮建两份全版索引 | J2 | 这一版单元数、映射条目数 | 第一轮表二 #28 列过，没量 |

推翻 J1 的现象：模型第 2 条打出 `whole_allocator_copies_before_the_judgement=0`，或者覆盖写的调用链不经 `publish_version_of_trees_relocating_units_after_a_check`。

## 二、B7 的反例（J2）

### J2　B7 的五件都做完，固定点每一轮仍为释放判定把上一版整张 `units` 与 `mapped_units` 各建一份索引

正文 B7 的五件：取空槽与记一条分配按索引找（`allocator.rs:971`、`:974`、`:1152`）、释放按索引找（`:2265`、`:2211`）、根头按下标取（`transaction.rs:4351`）、墓碑退役谱系容器按引用读（`:8601`、`:8646`、`:8729`、`:12511`）、ckpt_cost 的记账树一项按差量规划。另加差量规划（码 2 树与三棵容器树）与差量预演（分配器的差量视图）。

固定点每一轮先调释放判定：「            placements_released_by_this_publish(plan, previous, &resolved, allocator)?;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13074`）。它对换下的每个角色取上一版的映射条目，进门造一份读者：「    let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4613`）。第一次查时建两样：「                mapped_units_by_role: version.mapped_units_by_role(),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:4984`），与中央映射的批量读者，它一造就建「            units_by_role: version.units_by_role(),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:5079`）。两份都经 `FirstEntryOfEachIdentity::of_the_table` 走整张表、排一次序（「    pub fn of_the_table(」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:3671`）。读者是这一轮现造的，下一轮再造一份：一次覆盖写走两轮就建两遍。

- 随什么涨：`units` 每个数据单元一项（第一个文件的与别的文件的），`mapped_units` 每个进映射的单元一项；两样都随这一版的单元总数涨，一轮 Ω(P log P + M log M)。
- 为什么五件碰不到：五件里「释放按索引找」指的是分配器里按 (盘, 槽) 找那条分配记录（`allocator.rs:2265`、`:2211`），不是这一处按角色找上一版的映射条目；「根头按下标取」只管 `unit()` 那四个根；这两份是按角色查映射与单元字节的全版索引，B7 的定义一个字没提。
- 写文件的发布一定建：覆盖写换下的数据单元经映射释放，查一次就建（第一轮正推表二 #28 同判「写文件的发布一定要查」）。

同一类、B7 也没提的两样（代码现查，前两轮表二 #20、#21、#30 列过，B7 的定义没把它们收进去）：

1. 全记录分叶：每一轮两次「            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13120`），里面「    let before = records_of_each_leaf(geometry, previous_records);」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocation_record_tree.rs:372`）与下一行把前后全部记录各分一遍叶；再加 `nodes_holding_records` 一遍与「        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13122`）的整集合比较。B7 的「差量视图」只说「只记这一次取走与放回的槽」，没说这几个比较改成只比被动过的叶；第二轮辩方格一前提 2 指出要做成差量还得随一版带每叶记录计数（叶空了节点就不在了），B7 的五件里没有这一样。
2. 分配记录树的全节点集合：进门一次「            previous_version.allocation_record_tree.node_set(),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:13045`），每一轮拿两个全集合交、算计划。分配记录树按位置寻址，不是码 2 树，B7「多层码 2 树与三棵容器树的规划只读这一次进出的 key」罩不到它。

另有落点那一判的聚簇段求和（「        let pairs_inside_the_cluster_segments: u64 = cluster_segments」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/allocator.rs:578`）：B7 第一件只把「每个候选再逐个比聚簇段」那一处（`:974`）换成按索引，这一处每次判逐段求和照旧；它随这次挂载开过的段数涨，口径（段数算不算「总数」）前两轮都没定。

最小历史（推的，待主 agent 跑模型）：两块内存盘、单元区 16384 槽，第一个文件顺序写 N 个单元，再覆盖写一个单元。模型第 1 条在 N = 1、16、128、400 上打 `name=release_judgement_indexes … rounds_entering_the_release_judgement=… index_builds=… largest_index_looked_up=…`，断言每一轮至少建一份、查过的索引最大项数随 N 严格涨。我推的：`rounds_entering_the_release_judgement` 是 2（任务 6553 的覆盖写是「allocation_rounds_per_call=[1] release_only_rounds=1」两轮，每一轮都先调释放判定）、`index_builds` 至少 4、`largest_index_looked_up` 约等于 `units_in_the_version_before` 或 `mapped_units_in_the_version_before`（两者里大的那一个）。若 `largest_index_looked_up` 不随 N 涨，J2 撤回。

四句：
- 分不分辨候选：只关 B7（B6 不改固定点，本来就留着；B1 今天就有）。
- 看不看得到：看得到。两份索引只依赖上一版，可以随一版建一次、跨轮复用（今天读者的生命周期是一轮），或者随一版带着（与 `container_tree_root_unit_positions` 同一个先例）；那样一轮只剩 Δ × log。
- 满足的是判据字面的哪一个分句：D8（核心索引结构） 已定项 16 射程「准入之前的步代价不随盘上容器总数、记录总数涨」（单元与映射条目是记录）；两道之间照第一轮判决的读法算「准入之前」。
- 改法：B7 的定义加一件「释放判定读上一版的映射与单元按随一版带着的索引，不按轮现建」（攻方提，零轮）；全记录分叶与分配记录树全集合两样要另加「每叶记录计数随一版带着、只比被动过的叶」。

能不能做成会红的东西：能。模型第 1 条把断言反过来（「释放判定查过的索引不随这一版的单元数涨」）今天红，B7 照五件做完照样红。

推翻 J2 的现象：模型第 1 条打出释放判定那一处 `index_builds=0`，或 `largest_index_looked_up` 在四个 N 上不变。

## 三、B6 的反例（J3、J4、J5）

正文 B6 的定义（第三轮正文第三节）只有一句「准入之前先由差量算一个这一次分配量的上界；上界放得下才走快路……上界放不下整条走今天的路」，出处指到第二轮辩方报告第三节。下面三条分别打那一节写下的定义（J3、J5）与正文那一句没写到的一侧（J4）。

### J3　删除类与删快照整类不走快路：unlink 的每一批不论空间多宽都照今天

辩方 3.1：「快路条件，全成立才走：发布不是删除类、不是删快照；落点判在（U_普通，ckpt_cost）上过；U_记录不超上限；式子判在（U_普通 + U_固定点，U_预留）上过。」（`research/prompts/m4-admission-delta-r2-sonnet-output.md:90`）。辩方 3.4 列了 B6 没做到的五样（只救放行路径、带宽、规划挪到准入之后、改 D28 两句、记账树台阶行），没列这一样。

- 被排出去的是哪些发布：`is_a_deletion_class_publish` 认的意图、各批、最后一批，截断的意图与各批，删快照的意图与各批（今天式子那一判对它们整次不判，「                !step.is_a_deletion_class_publish()」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12605`）。它们按 B6 的快路条件第一条就落到慢路，慢路是今天整条路：普通分配那一半（墓碑、退役整份照抄，G8）、ckpt_cost（G7、G10）、固定点每一轮（整份拷、全记录分叶、两份全版索引，J2）。
- 这一类有多常见：被判条款就住在 D8（核心索引结构） 已定项 16「第一版 unlink」名下（`.claude/kb/decisions/08-核心索引结构.md:516` 是它的射程那一行），分批那一条写「对任一 inode N，从尾部按数据单元个数分批」（`.claude/kb/decisions/08-核心索引结构.md:508`），一个大文件删一次是 ⌈单元数 ÷ 144⌉ 次删除类发布；会话里每一次用户改动之前先把队列走空，队列里有东西时这几批排在用户那一次改动前面。
- 最小历史（按构造，不靠取样）：任何一段含一次 unlink 的历史。快路条件第一条对意图那次与每一批都为假，准入之前照 B1 随墓碑总数、单元总数、分配记录总数涨，与池里还剩多少空间无关——不是 G5 那一带（只在快满时），是全部删除类发布。
- 能不能放进快路：删除类今天只判落点那一判，D28（挂载期承诺量） 已定项 1 已有一个按差量写的删除类上界 W（「W 是一次删除类发布每盘新占槽数的上界」那一句在 `.claude/kb/decisions/28-挂载期承诺量.md:31`），可以拿来当删除类的快路上界；可它自己被攻过零轮，第二轮判决列的 L1（W 的每层 2·(K + 26) 隐含一条没写的落点局部性假设）还开着。

四句：只关 B6；看得到（发布种类在计划里，一进门就知道）；满足 D8 已定项 16 射程前半句（删除批是这一条款自己管的发布）；改法——快路条件去掉「不是删除类、不是删快照」，删除类的上界取 W（零轮，要先结 L1），这一格才不中。

推翻 J3 的现象：B6 的定义另有一处把删除类收进快路（我只读了辩方第三节与正文第三节）。

### J4　（条件式）已承诺预留那一侧：建 inode 的发布按这次之后的标签 6 判，取上一版的就少扣一截

式子那一判的可用里扣的已承诺预留按这次之后那一版算：「/// 记账标签 6（已承诺预留里墓碑与退役记录那两份，D3（空间分配） 已定项 2）在这次发布之后那一版的值：」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:6960`），式子是「/// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)：H_墓碑、H_退役是这次之后墓碑树、退役树的内部层数，」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:6961`）。建 k 个 inode 的一次发布让 n_live 与 n_记录 各加 k，标签 6 加副本数 × 76 × k 字节。

代码里另有一份「取上一版记下的标签 6」的读法，装置与用例用：「/// 同那一份，已承诺预留取 `version_to_build_on` 那一版记账行里写下的标签 6（不建 inode 的发布前后相同），」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1765`）。它自己写着只对不建 inode 的发布成立。

- 正文 B6 那一句只说「这一次分配量的上界」，管的是需求那一侧；可用那一侧要的是已承诺预留的上界（从可用里扣得越多越保守）。辩方 3.1 写了「4. U_预留：已承诺预留的上界；写入类发布不重写墓碑树与退役树，读上一版根头与按差量维护的 `InodeRecordCounts`（推；第二次抽样「没打中的形状」同判）。」（`research/prompts/m4-admission-delta-r2-sonnet-output.md:88`），「按差量维护的」读成「这次之后、按计划里新建的那几条改出来的」就对；读成「上一版带着的」就是上面那份装置读法。
- 照后一种读法做的反例（形状，推的）：一次 `NewInodes { count: k }`，可用落在 [需求, 需求 + 副本数 × 76 × k) 字节里：今天式子那一判拒（`SpaceAdmissionRefused`，会话推抬 F），快路放行；B6 在快路上把式子判换成「精确需求 ≤ 上界」的断言，断言只核需求，不核预留，于是照发——「放行而真实不够」那一种。判是逐字节比的（「(available.0 < i128::from(demand.0)).then_some(DeviceShortOfDemand {」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:904`），这一截再窄也有落进去的池。
- 模型第 3 条量这一截：一次建 1、8、32 个 inode，打 `name=committed_reservation_of_an_inode_creating_publish … short_if_the_previous_one_is_read_bytes=… equals_the_expectation=…`；我推的是 152、1216、4864 字节（两份副本 × 76 × k），三行都 `yes`。

四句：只关 B6；看得到（新建几条在计划里）；满足 B6 自己的前提「上界真」那一句（辩方 3.2「全部成立的前提只有一个：上界真」）的预留一侧；改法——B6 的定义写明 U_预留 取「按计划里这次写进、删掉的那几条从上一版改出来的标签 6」（`InodeRecordCounts::after_writing` 那一套），不取上一版记下的那一份。照那样写就不中，所以这一条是条件式：它打的是定义没写到的一侧，不是 B6 做不到。

### J5　（线索）U_固定点 的清单漏了 deadlist 树，今天落点那一判也漏了它

辩方 3.1 的 U_固定点：「2. U_固定点：分配记录树取 K1（无条件上界）；中央映射树取 `admission.rs:1484` 那一族逐层式子，删插数换成这次换下与新写的角色数；记账树取闭式（格五）；谱系、退役沿用 ckpt_cost 里现成的两项；树表 1。」（`research/prompts/m4-admission-delta-r2-sonnet-output.md:86`），没有 deadlist 树；同一节 3.3 的表又列了「| 码 2 树（记账、deadlist、中央映射）的重写节点数 | 推：2.2 的读码推导 | 先删后插、路径加切分加新根；尚无穷举验证 |」（`research/prompts/m4-admission-delta-r2-sonnet-output.md:114`）。两处对不上。

今天的代码里 deadlist 树的节点归普通分配，注释写「        // `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4）：条款没写它归哪一格，照删除那两棵树读——落点那一判照判，」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1604`）。可落点那一判的输入是普通分配那一半的角色（「    let ordinary_allocation_roles = plan」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12510`），那一半的清单是「    /// 这一半重写的角色（`admission::space_budget_of_role` 都是普通分配）：数据单元、extent 树的节点、inode 叶容器与 inode 根，」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8101`）加删除、logged_ops、退役三样，deadlist 树的角色到 `resolve` 里才并进来（「        // deadlist 树（树 65543）按树 ID 排在记账树之后、删除那两棵树之前。」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:8872`）；ckpt_cost 也不含它（同一段注释「不进 ckpt_cost 的 Σ 名单」）。所以今天落点那一判一个 deadlist 树节点都不数，注释说的「照判」没做到；今天由固定点里取不到落点（`PlacementRefused`，同样推抬 F）兜着。

对 B6：照 3.1 做，活快照下覆盖写一个被快照钉住的单元，deadlist 树重写的节点（至少一片叶连到根）不在 U_普通、也不在 U_固定点 的任何一项里，只能被别的项的余量吞掉——主要是 K1 减去这次真写的分配记录树节点数。大盘上这份余量几百到几百万块，断言不会打中；窄盘上 K1 小（冻结副本注释给的例子「单元区 240 / 256 / 384 槽的窄盘每块盘只罩叶 61、62 两片，1 + 2 × 2 = 5」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1376`），一次写把两片叶都弄脏时余量为零，deadlist 树那几个节点就让「精确需求 ≤ 上界」的断言打中（panic，今天是照发或照拒）。这一条我没造出历史，只推到这里，写「推的」。

四句：只关 B6（J5 前半「今天落点那一判不数 deadlist 树」不分辨候选，另立一笔）；看得到（这次换下的用户可见单元被不被快照钉住，`resolve` 里才定，快路要另算一个不看生死判定的上界：换下的用户可见单元数 × deadlist 树每把 key 至多重写的节点数）；满足辩方 3.2「上界真」那一个前提；改法——U_固定点 或 U_普通 补 deadlist 树一项（零轮）。

## 四、「已定」的那几格（J6、J7）

### J6　第二轮判 B4「要改两处」：靠的是一种读法，那种读法照样够到准入之后的真分配

第二轮判决那一格：「| 格三 B4 改射程 | 打中：是条款变更，且要改两处 | 辩方格三与攻方 G4 对上：射程后半句只排出「装单元、拷字节」，两道之间是分配、生命周期判定、会交回拒绝，`.claude/rules/fs-design.md:23` 把它们点名成运行时决策路径；用户「只管准入之前」够不着两道之间 |」（`research/prompts/m4-admission-delta-r2-main-verification.md:29`）。

射程那一句前半说 fs-design 那张表的运行时决策路径一格「在这一格读成只管准入之前」，后半是「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。」（两段都在 `.claude/kb/decisions/08-核心索引结构.md:516`）。用户原话「  - **用户原话**：条款读法「管，只管准入之前」；排法「五处加准入现在改，根源先量」。」（`.claude/kb/decisions-history.md:3514`）。

两种读法：

| 读法 | 后半句是什么 | B4 要改几处 | 推到准入之后的真分配 |
|---|---|---|---|
| 甲：前半句定射程 | 「只管准入之前」已把准入之后整段排出 fs-design 那一格；后半句是在说准入之后剩下的是什么，不是穷举的豁免单 | 一处：D8 已定项 16 射程里「准入」指哪一判（第一轮判决读成两道一起算，B4 改成只算落点判） | 不管 |
| 乙：后半句是穷举的豁免单（判决用的） | 准入之后只有「装单元、拷字节」不算，别的分配、生命周期判定照 fs-design 那一格管 | 两处（判决的读法） | 管：两道准入都判完之后，`publish_admitted`（「fn publish_admitted<Device: BlockDevice>(」，`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:14656`）真取落点「        let placement = allocate_placement_for_role(allocator, *identity, txg)?;」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:14819`），用户数据走「        PlacementRule::UserData => allocator.try_allocate_user_data(txg),」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:2384`），就是第二轮 G2 那条从单元区起点逐对扫的路；释放进 defer 队列也在这一段。这几样是分配、是 defer 窗口，不是装单元、拷字节 |

所以：
- 「是条款变更」两种读法下都站得住（B4 改的是第一轮判决定下的「准入」所指）。
- 「且要改两处」只在乙下成立。乙一旦成立，它管的就不止两道之间：准入之后的真取落点（`:14819`）今天也随被占的槽对、聚簇段数涨，B1、B4、B6 照定义都留着它，只有 B7 的第一、二件（改的是分配器本体，真发也走同一份）碰得到。前三轮没有一份把 `:14819` 这一步列进清单——正文格三的窗口截在式子判，第一轮判决又把「准入之前」读成两道一起算，这一步在两种算法下都落在窗口之外。
- 辩方 3.4 第 3 条「快路之后的全量规划与固定点仍在，总开销没降，只是排到「准入之后」」（`research/prompts/m4-admission-delta-r2-sonnet-output.md:126`）要用户确认免责口径比字面宽，也是乙。照甲，B6 把规划挪到准入之后不需要改条款；照乙，B6 挪过去的规划与固定点、连同今天就在准入之后的真分配，都还被那一格管。

四句：
- 分不分辨候选：分辨。乙下 B6、B4 都多一处条款变更，B1 今天在准入之后也不合，B7 的五件里改分配器本体的两件同时修准入之后那一段；甲下 B4 一处、B6 零处。
- 看不看得到：条款文字在手里；用户那一次弹窗的原问题读不到（第二轮辩方格三同报）。
- 满足的是判据字面的哪一个分句：第二轮判决那一格「且要改两处」那半句。
- 改法：这不是候选的改法，是要用户定的读法：交用户时把「射程后半句是不是穷举的豁免单」单列成一问，两种答案下 B4 与 B6 各要改几处、准入之后的真分配管不管，一并写出。

推翻 J6 的现象：变更史或那次弹窗的原文里写明后半句是穷举的（例如「准入之后只有装单元、拷字节不算」），那乙就是字面，「两处」站住，同时 `:14819` 那一步要补进清单。

### J7　第二轮判 B2、B5 打中：没打穿

- B2（G1）：任务 6553 的 18 次改动里只有一次挪了槽——「settle name=settle_rounds change=sequential_write_of_64_data_units settling_calls=1 allocation_rounds_per_call=[2] release_only_rounds=1 roles_compared=74 roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2 first_shift=AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition { level: 0, device: DeviceIdentity(1), index_in_device: 1 }):1580->1581」（任务 6553 日志，现读）；末行「settle name=settle_rounds_summary changes=18 settling_calls=20 calls_with_two_or_more_allocation_rounds=1 most_allocation_rounds_in_one_call=2 roles_compared=74 roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2」。判的是「乙读法与今天不逐个相同」，一段历史就够；判决写「覆盖写与建 inode 那几次都是 roles_shifted_between_rounds=0、一轮」与日志对得上（unlink 两次也是 0）。我试的角度：乙读法会不会恰好在第二轮把挪过的那 5 个角色重取回同一槽——不会，乙不重取前一轮放下的角色，这正是乙的定义。没打穿。
- B5（G5）：带宽「至少 K1 − 实写数」是按构造：上界里分配记录树取 K1，精确判取固定点真写的；带宽随单元区线性涨靠 K1 的式子（D28 与 Rust 注释）。K1 的四个数怎么来的有问题（J10），可这一格的结论不靠那四个数。读法一「什么都不省」也是按构造。没打穿。

## 五、格四：前两轮报出的数能不能核（J8–J11）

三份日志都现读了（命令在开头那一段），判决第一节抄的每一行在日志里逐字找得到。下面只写「取样点撑不撑得住判决里用它的那一句」。

### J8　任务 6553 有三列量的是「在哪 / 有几条」，判决拿来写「走了多少」

| 列 | 模型怎么算的 | 判决怎么用 | 撑不撑得住 |
|---|---|---|---|
| `candidates_walked_before_the_free_pair` | 「        let candidates_walked = (lowest - start) / 2;」（`research/prompts/m4-admission-delta-r2-opus-model/scans_and_copies.rs:242`）：第一对空槽的槽号减单元区起点除以 2，是答案的位置，不是循环走了几步 | 第二轮判决格二「攻方 G2 现量：找空槽线性扫（`candidates_walked_before_the_free_pair` 3、163、435、1267 随 `data_units_kept_below` 涨，`allocator.rs:971`、`:974`）」（`research/prompts/m4-admission-delta-r2-main-verification.md:28`） | 撑得住，但撑它的是同一行的挂钟列，判决没引：四行的 `one_user_data_placement_ns_median` 是 2830、38612、102946、393066，除以候选数每个约 943、237、237、310 ns，后三点与候选数成正比（第一点只有 3 个候选，常数项占大头）。候选数那一列本身任何「取最低的空对」的分配器都打出同样的数，B7 第一件做完之后照样 3、163、435、1267 |
| `allocation_tree_at` 等四个根头下标 | 「        .position(|unit| unit.identity == role)」（`research/prompts/m4-admission-delta-r2-opus-model/scans_and_copies.rs:72`）：角色在 `units` 里的下标 | 第二轮判决「第一轮正文没点名的」那一格「打中（模型现量）」，G7「根头排在全部数据单元之后」 | 那半句撑得住（下标 6、22、136、411 随数据单元数 1、16、128、400 涨，四点都在日志里）；「每次找根头走全部数据单元」是 `unit()` 线性找这一条代码推出来的，模型没量走了几项、没量挂钟。B7 第三件（根头按下标取）做完，下标不变、读数不变 |
| `tombstone_records_carried_by_the_overwrite` | 覆盖写之后那一版里墓碑条数（同一段打印，`research/prompts/m4-admission-delta-r2-opus-model/scans_and_copies.rs:204`） | 同一格 G8「覆盖写带着全部墓碑」，「打中（模型现量）」 | 「带着」撑得住（1、8、32、96 四点）；「落点判之前拷一次、每轮 resolve 里再拷一次」是代码推出来的：新一版装着全部墓碑这件事，按引用共享的表示下读数一样是 96。B7 第四件做完，读数不变 |

所以：判决这两格写的「模型现量」量到的是前提（根在后面、墓碑都在、空对在上面），代价那一半（走、拷）只有代码一条路；要「现量」到代价，得数走过的项或拷的字节（`lookup-work-counters` 管不到 `unit()` 与容器照抄）。拿这三列做 B7 的验收测试，B7 照定义做完它们照样「红」——测不出 B7。

### J9　任务 6553 的拷贝挂钟与整次发布挂钟对不上「每次拷几份」（线索）

日志四行 `allocator_copy_by_unit_area`：单元区 16384、131072、1048576、16777216 槽，拷一份 2870、17281、1096661、34949849 ns，一次覆盖写整次发布 6928799、4569896、7471424、71945489 ns。后两点之间整次发布多了 71945489 − 7471424 = 64474065 ns，拷一份多了 34949849 − 1096661 = 33853188 ns，比是 1.9（算术，推的）。按代码数，一次覆盖写在两道准入之前拷 3 份：J1 那一份，加固定点两轮各一份（同一份日志「settle name=settle_rounds change=overwrite_0 settling_calls=1 allocation_rounds_per_call=[1] release_only_rounds=1」：一轮只释放、一轮取落点，每轮进门先拷）。debug 构建、几条用例同一个二进制里并行、整次发布只取 3 次的中位数，1.9 与 3 差在噪声还是差在次数，挂钟分不出。判决「克隆分配器随单元区涨」不受影响；要定「每次发布拷几份」，用模型第 2 条的计数（`whole_allocator_copies_before_the_judgement` 与 `whole_allocator_copies_in_settling_rounds`），不用挂钟。

### J10　任务 6265 的 `k1_band` 四行是 Python 照公式复算、带宽用了假定的实写数

- 怎么算的：`research/prompts/m4-admission-delta-r1-opus-model/k1_band.py` 照 Rust 注释的式子另写一份，不调 `allocation_record_tree_positions_over_the_unit_areas`；单元区起点四种几何都取「UNIT_AREA_START = 50176」（`research/prompts/m4-admission-delta-r1-opus-model/k1_band.py:19`，那是 4 GiB 盘的起点，文件头自己写「大盘上的真起点没查（推的）」）；「今天的实写数」取「def todays_settled_allocation_record_nodes(level_of_the_root, leaves_per_device=4):」（`research/prompts/m4-admission-delta-r1-opus-model/k1_band.py:41`），假定每块盘 4 片叶连到根，打出来是 `todays_settled_nodes_assumed=17` 与 `=25`。
- 能核的只有一点：2×4GiB 的 `k1=529` 与 Rust 注释「两块 4 GiB 盘（单元区 [50176, 262144)、根层级 2）：每块盘叶 262 个、层级 1 两个，1 + 2 × 264 = 529；」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/admission.rs:1375`）对得上。另三点（10273、166159、2660309）只有 Python 一条路。
- `band_blocks_at_least` 叫「至少」，算的是 K1 − 假定实写数：实写数比假定多时带更窄，它不是下界。
- 判决怎么用：第一轮判决「判它会多拒一带（任务 6265 的 `k1_band` 四行）」（`research/prompts/m4-admission-delta-r1-main-verification.md:24`），第二轮判决格四「上界放不下那一带至少宽 K1 − 实写数，随单元区线性涨」（`research/prompts/m4-admission-delta-r2-main-verification.md:30`）。「有一带」「随单元区线性涨」两句靠式子，撑得住；带宽的具体数（每盘 8.0 MiB、2595.8 MiB）要标「推的」，大盘那三点的 K1 要用 Rust 那个函数在真几何上重算才算量过。

### J11　任务 6394 里有一行两份判决都没引：写入类发布真写的固定点超过 ckpt_cost

日志「today name=fixed_point_vs_checkpoint_cost capacities=capped_mapping_4_8 data_units=85 checkpoint_cost_of_the_previous_version=29 fixed_point_slots_written=43 excess_slots=14 mapping_tree_nodes_after=38」（任务 6394 日志，现读）；同一组 1、10、20、40 个单元那几行是 −20、−17、−12、−4，产品容量那一组五行全是 −10。

- 第一轮第二次抽样的预言是「我预期压小那一档 40、85 两格为正、产品那一档全不为正（推的）」（`research/prompts/m4-admission-delta-r1-opus-s2-output.md:87`）：85 那一格中了，40 那一格没中（−4）。
- 第二轮攻方 G6 写的是「推的，那一腿模型的那一格主 agent 判决里没单列读数」（`research/prompts/m4-admission-delta-r2-opus-output.md:146`）：日志里其实有这个读数，只是两份判决都没抄。
- 它说明的事（今天就有，不分辨候选）：落点那一判的注释写「数据单元要段外成对的空槽、提交内生块要没挡的槽（普通分配那一半照计划数，固定点按 ckpt_cost 的最坏数），」（`/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo/crates/singlefs-core/src/transaction.rs:12508`），可 ckpt_cost 只数空发布会改的映射 key，写入类发布插进映射树的 key 让真写的固定点超过它（压小容量下 85 个单元多 14 槽）；今天由固定点里 `PlacementRefused` 兜着。产品容量下要多大的写才翻过去没量（第二次抽样推的门槛「N 约 1000 以上」）。对 B6：辩方 3.1 的快路落点那一判照抄今天这一份（「落点判在（U_普通，ckpt_cost）上过」），与今天同错同兜，不分辨；B6 的 U_固定点 若有人图省事直接取 ckpt_cost，这一行就是断言打中的反例。

## 六、改法各修哪一格

全是「推的」（按代码推、没实现没跑），只在我的推理上成立、被攻过零轮；「量过」一格都没有（模型量的是今天的代码，不是改法）。

| 改法 | J1 | J2 | J3 | J4 | J5 | J6 |
|---|---|---|---|---|---|---|
| 判之前那一份整份拷贝挪到两道准入之后（准入拒绝的路上分配器还没动），或换成只记这一次改动的撤销日志 | 推的：修 | — | 推的：删除类那份拷贝一并修，别的照旧 | — | — | — |
| B7 加一件：释放判定读上一版的映射与单元按随一版带着的索引，不按轮现建 | — | 推的：修两份索引那一半 | — | — | — | — |
| B7 加一件：每叶记录计数随一版带着，全记录分叶、全节点集合比较换成只比被动过的叶 | — | 推的：修另两样 | — | — | — | — |
| B6 快路条件去掉「不是删除类、不是删快照」，删除类的上界取 W | — | — | 推的：修，前提是 L1（W 的落点局部性）先结 | — | — | — |
| B6 的 U_预留 写明取「按计划从上一版改出来的这次之后的标签 6」 | — | — | — | 推的：修 | — | — |
| B6 的 U_固定点 补 deadlist 树一项（换下的用户可见单元数 × 每把 key 至多重写的节点数） | — | — | — | — | 推的：修 | — |
| 今天落点那一判把 deadlist 树的节点也数进去（或改注释，说明它由固定点里的 `PlacementRefused` 兜） | — | — | — | — | 推的：修今天那一半 | — |
| 交用户时单列一问：射程后半句是不是穷举的豁免单 | — | — | — | — | — | 推的：把「要改几处」变成用户定的事 |

## 七、各候选的代价量级（一行一条，全是推的）

记号：S = 每块盘单元区槽数，R = 分配记录总数，P = 这一版 `units` 项数（约等于数据单元总数），M = 进映射的单元数，T = 墓碑与退役记录总数，r = 固定点轮数（今天的覆盖写 2 轮，任务 6553），k = 这次进出的 key 与角色数，h = 树高。几何取 E199（发布照抄上一版的代价） 的主几何 u = 10⁶ 个数据单元（「- **G0（主几何）**：u = 10⁶、a_T = 1（一次 unlink 的意图）、R_X = 10³、a_X = 0、R_L = 16、a_L = 0、I = 10³、D = 0、s = 4；乙2 b = 32、g = 64；丁 p = 64、整份新基。」，`research/prompts/e199-preregistration.md:400`），每个数据单元两块盘各占槽，S 与 R 至少是 10⁶ 量级（推的）。

| 候选 | 两道准入都判完之前，一次单单元覆盖写的代价 | 在 u = 10⁶ 上大约（推的） | 要不要另建计数模型 |
|---|---|---|---|
| B1 今天 | (1 + r)·Ω(S + R) 整份拷（J1 加每轮一份）+ r·Ω(P log P + M log M)（J2）+ r·Ω(R) 全记录分叶 + G7、G8、G10 + 每个数据角色 Ω(被占槽对 × 聚簇段) + 全量规划 | 每份至少两块盘的三张位图约 6 MB（S 取 10⁶）再加 R 条分配记录（每条字节数没量），一次覆盖写拷 3 份；两份全版索引每轮各排 10⁶ 项、两轮约 4 × 10⁶ 项的排序 | 拷贝次数、索引大小用模型第 1、2 条数（待主 agent 跑）；拷贝字节要 E199 第二段那种计数分配器 |
| B6（辩方 3.1 那一份） | 快路：Ω(S + R)（J1 那一份在上界之前）+ ckpt_cost（G7 的 Ω(P) 找根头、G10 的 Ω(台阶行)）+ 聚簇段求和；之后照 B1 的整条路挪到准入之后。慢路（G5 那一带，加上 J3 的全部删除类发布）：B1 | 快路准入之前 1 份整份拷（至少 6 MB 加分配记录）加两遍走 10⁶ 项的 `units`；每删一个大文件 ⌈单元数 ÷ 144⌉ 批全走 B1 | 要：「可用落在带里的发布占多少」随池满的程度怎么变，带宽靠 K1 在真几何上重算（J10），删除类那一半不用模型（按构造全走慢路） |
| B7（正文那一份） | Ω(S + R)（J1）+ r·[Ω(P log P + M log M)（J2）+ Ω(R) 全记录分叶 + Ω(分配记录树节点数) 全集合]+ 聚簇段求和；五件与差量规划修掉的那几样降到 O(k·h·log) | 一份整份拷（至少 6 MB 加分配记录）加每轮两份 10⁶ 项的索引；五件修掉的部分降到几十到几百次对数查找 | 要：J2 那几样在 B7 下还剩多少，用模型第 1 条的索引大小作上界（今天的数，B7 不改它们） |

## 八、要主 agent 跑的命令

一条，不是重型（一个 harness 档测试目标，不碰 checker 档包，不调崩溃枚举）。我一条都没交。

1. 模型整份复跑（定义第 9 步）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r3-attack -- bash /home/fy5090/code/singlefs/research/prompts/m4-admission-delta-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r3-attack/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs` 起 submit，`--cwd` 是本腿草稿目录。脚本里那一条 `cargo test -p singlefs-harness --test m4_admission_delta_r3_attack_counts` 自带 `SINGLEFS_RUN_SCOPE='种类=target; …; 范围=1 个测试目标; …'`，在 `--work` 下拷出的 `tree/` 里跑（拷冻结副本连 `prune-server`，补仓里的 `.cargo` 与 `.claude/heavy-thread-default`），跑完 `tree/` 由脚本删掉，留 `patch.out`、`copy_counters.diff`、`counts.out`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志末尾 `name=rerun_summary patch_rc=0 counts_rc=0` 与 `name=rerun_verdict all_ran=yes`；`name=copy_counters_applied edits=3`；`counts test … ok` 三行。读数：`counts name=release_judgement_indexes` 四行（J2：`largest_index_looked_up` 随 `data_units_in_the_version_before` 涨，`index_builds ≥ rounds_entering_the_release_judgement`）、`counts name=lookup_work` 若干行（每一处一行，看哪几处在两道准入之前）、`counts name=allocator_copies` 十二行（J1、J9：`whole_allocator_copies_before_the_judgement` 与 `judgements`、`whole_allocator_copies_in_settling_rounds` 各是几）、`counts name=committed_reservation_of_an_inode_creating_publish` 三行（J4：`short_if_the_previous_one_is_read_bytes` 我推的 152、1216、4864，`equals_the_expectation=yes`）。
   - 编不过、锚点退 3 或某条用例红：整份 `patch.out` / `counts.out` 交承接的腿看，不当「模型不成立」；断言红之前每条用例的行都已打出。
   - 跑完新派同立场的腿，派发提示写「承接：research/prompts/m4-admission-delta-r3-opus-output.md」与任务号，读 `task-server/task log <号> --all`，把 J1、J2、J4、J9 里「推的，待主 agent 跑」换成读到的数；J2 若 `largest_index_looked_up` 不随 N 涨就撤回，J1 若 `whole_allocator_copies_before_the_judgement=0` 就撤回。

## 九、没打中的形状

| 形状 | 攻的是哪条 | 为什么没打中 / 没做到 | 取样范围 |
|---|---|---|---|
| B6 快路放行之后拒绝次序变了（普通分配那一半的 `?` 错误、合取 7、固定点里的几种拒） | B6 | 快路不写盘、不动分配器；上界真时落点、记录条数、式子三判在快路上都过，剩下的拒照今天的先后在放行之后出现；普通分配那一半的错误在快路上由 `resolve` 第一轮先交回，与今天排在落点判之前的那一次是同一种、同一个值；上界放不下时整条走今天的路 | 读 `judge_the_publish_before_any_read_or_write` 全函数与 `resolve` 开头 |
| B6 快路的上界算不出来时（算上界本身报错）抢在今天的错误前面 | B6 | 辩方 3.1 的四个上界都写成纯函数、不报错；只要实现照这样写就没有新的错误种类 | 只读定义 |
| 整理批（`relocations` 非空、没有普通分配）在快路上被错放 | B6 | 今天这类发布不判落点、照判式子；快路上落点那一判过不过都不改结局（过不了走慢路，慢路就是今天），式子那一侧照抄批次最坏占用 | 读 `demand_is_judged` 那一段 |
| 管理员回退带着「不搬而换了位置的映射 key」让 U_固定点 里中央映射树那一项少数 | B6 | 回退照辩方 3.1 不在快路排除之列，可今天回退整次不判式子（`demand_is_judged` 只看普通分配与整理），快路上式子那一侧本来就不判 | 读 `settle_the_allocation_record_tree` 开头与 `demand_is_judged` |
| B7 的「同一个底子重放」下第二轮 G1 那种挪动还在不在 | B7（格二，归正推） | 甲读法下每轮从同一份底子重放，结局按构造与今天相同；我没找到差量视图会让同一轮里两个角色取到同一槽之外的分叉 | 读 `settle_the_allocation_record_tree` 全函数 |
| 会话的盘表与落后核在每次改动之前走全部单元 | 格三 | `devices_behind_the_current_version` 正常的盘只读两个系统配置槽，只有落后的盘才读现行那一版每个单元的落点——那是拒绝路径 | 读 `mount.rs` 那一段 |
| 实例表整条链重写随历次写行涨 | 格三 | 只在写行（挂载）与实例表清扫那两处发生，它们没有普通分配、两道准入都不判 | 读 `InstanceTablePlan::Rewrite` 的三处构造 |
| 第一轮 K1 的四个数在真几何上算错了 | 格四 | 只核了 2×4GiB 那一点（与 Rust 注释相同）；另三点没在 Rust 上重算，算不出对错，只写「推的」 | 读 `k1_band.py` 与 Rust 那个函数 |
| 由用户决定的动作 | — | J1、J2、J3 都是「任何一段含这一步的历史」：J1 每次发布都有，J2 每次写文件的发布都有，J3 每次 unlink 都有，不依赖用户之后做什么；J4、J5 是「存在一个落进那一截的池」，要扫的是几何与快照，不是用户动作。模型里的历史把用户动作写死了（覆盖写、建 inode、顺序写、unlink 的固定次序），没放开扫 | — |

以上「没打中」都只抽了这一次样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不算「没打中」。

## 十、这条腿自己的限度

- 一条命令都没跑：模型与补丁脚本没编过、没跑过，可能编不过（补丁脚本只用 `python3 -m py_compile` 查过语法，`rerun.sh` 只用 `bash -n`）。模型的数都是推的。
- J4、J5 是条件式：J4 打的是「上界快路取了上一版记下的预留」那一种实现，J5 打的是照辩方 3.1 的清单字面实现；照 3.3 的表或照「按计划改出来的预留」实现，两条都不中。J5 的窄盘断言打中没造出历史。
- J6 的两种读法我判不出哪一种是用户的意思；变更史只留了一句原话。
- 只读了 `judge_the_publish_before_any_read_or_write`、`settle_the_allocation_record_tree`、`resolve`、释放判定、调用方那一层与会话改动入口；`publish_admitted` 只看了取落点那一行，`plan_the_multi_level_trees` 与容器树规划内部没读。
- 第二轮辩方报告与第一轮、第二轮攻方报告与模型是这一轮之前的材料，我读了；禁读清单里那一份（这一轮正推）没读，所以与正推重叠的格（格一、格二）我只打反例，不判它推得对不对。

## 十一、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；不下「选哪条」的结论。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在第八节）。副本上量出的数不算入库装置上的数。
- 没写进 `crates/`（写范围不含它）；几条「能不能做成会红的测试」交主 agent 定要不要派人写。
- 定义第 3b 步的崩溃状态枚举没做（偏离写在开头「复跑命令与模型文件」一节）。

## 十二、删了什么

- 这一腿没建仓副本与编译目录。草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-admission-delta-r3-attack` 里只有认领留下的 `.owner`、`.writer-lock` 与三份现读的日志 `task6265.log`、`task6394.log`、`task6553.log`（留着，承接与判决核格四要用）。复跑时 `rerun.sh` 在 `rerun-1/tree/` 拷出的树由脚本自己删。
