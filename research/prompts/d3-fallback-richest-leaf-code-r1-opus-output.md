# d3-fallback-richest-leaf-code-r1 攻方腿（Opus）报告（2026-11-12）

立场：攻方，找「哪一格会错」。被判对象是 `research/prompts/_d3-fallback-richest-leaf-code-r1-diff.md` 里的块与它们和周围代码的交叉，文件:行写主工作区今天的行号（`crates/singlefs-core/src/allocator.rs`、`transaction.rs` 的 sha256 与开工快照 `research/prompts/d3-fallback-richest-leaf-code-r1-snapshot/crates-sha256.txt` 逐字相同，现跑 `sha256sum` 核过）。
这一条腿没往任务服务交任何任务、没编没跑项目代码：下面凡是模型读数都是「推的，待主 agent 跑」，代码事实是读码与 `grep -nF` 现查的。

## 复跑命令与模型文件

模型目录 `research/prompts/d3-fallback-richest-leaf-code-r1-opus-model/`（一个验证包测试目标的源码、装臂脚本、复跑脚本），冻结副本是我用 `minimal-copy.py` 建的 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-attack/copy`（两份被判文件的 sha256 与开工快照相同，`.deliverables` 写成「交接」）。复跑命令（主 agent 交，在新 shell 里起；外层只先删掉那一个 `-run` 运行目录，再调同目录的 `tools/rerun.sh`）：

`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/d3-fallback-richest-leaf-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-attack/copy /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-attack-run /home/fy5090/code/singlefs/research/prompts/d3-fallback-richest-leaf-code-r1-opus-model`

| 文件 | sha256 |
|---|---|
| `probe/fallback_settling_on_forged_disks_without_a_rich_leaf.rs` | `f2939ad1983c401235435fac3eae72ca50bc2fe659dae812847c2b4d3727ada0` |
| `tools/patch_the_copy.py` | `c77aa9d6763bc9a453065d602eac7751066946d1f7d83eabe2194aeb75fbf9a6` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh` | `330dbcf861dd51d77af1fce6bbdb5e58a233ff47f52f873ae9f0d5ca36b60e3e` |

模型做什么：在副本里依次装三条臂（`patch_the_copy.py` 从冻结副本读 `allocator.rs` 原文、按臂做定点替换，旧串与替换文逐字取自 `crates/mutations.tsv` 第 7659、7663 行那两条变异），每条臂编一次验证包测试目标、跑五条用例，每条用例在锻造盘（照实现员新目标的锻造写法：只动空闲图、不写分配记录）上覆盖写第一个文件一次，打一行 `PROBE_RESULT shape=… outcome=… rounds=…`。三条臂：

- as-coded：今天的实现。
- prefer-then-lowest：先重写集的叶，不够就退回各盘槽号最小的空槽（第 7663 行变异）。
- lowest-only：改前的政策，各盘槽号最小的空槽（第 7663、7659 两行变异合起来）。

五个形状与跑前写下的预测（推的，跑前写死；量出相反的数，对应那一格的结论照下面「推翻条件」改）：

| 形状 | 单元区槽数 | as-coded | prefer-then-lowest | lowest-only |
|---|---|---|---|---|
| sparse-leaves-below-a-rich-region（实现员的形状，对照） | 2¹⁸ | 定下、4 轮、被合取 7 拒 | 走满 64 轮（实现员探针 `prefer_then_lowest prefix=14 outcome=did_not_settle_limit_64`） | 推不出 |
| an-aligned-pair-in-every-leaf | 2¹⁸ | 几轮定下 | 几轮定下 | 几轮定下 |
| one-unaligned-slot-in-every-leaf-and-two-pairs-in-the-top-leaf | 2²⁰ | `placement-refused unit=InodeLeafContainer…` | 走满 64 轮 | 走满 64 轮 |
| pair-at-the-bottom-of-the-richest-leaf | 2¹⁸ | `placement-refused unit=InodeLeafContainer…` | 定下、被合取 7 拒 | 定下、被合取 7 拒 |
| pair-at-the-bottom-of-the-lowest-leaf | 2¹⁸ | `placement-refused`（第二个取落点的轮） | `placement-refused`（第一个取落点的轮） | `placement-refused` |

估时（推的）：三次增量编译验证包测试目标各几分钟；用例里最重的是 2²⁰ 那一形在两条退回最低空槽的臂上，每个落点逐槽从单元区起点扫，64 轮累计约 2.5 万个落点、每个至多扫 100 万槽 × 2 盘，推的一两分钟。全量估不到 40 分钟，没缩。这一模型不枚举崩溃状态（攻的是发布前的固定点与取落点，不造崩溃历史），定义第 3b 步的崩溃状态数上界不适用，全量崩溃状态数为 0。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 攻击面 1 越位：各盘答案不同而有共同空槽时成功 | 没打中 | 落在用户那句「每块盘都答回落时，池里先取…每块盘都空的…」字面里，末段「各盘落点答案必须相同」由池级落点满足 |
| 攻击面 1 越位：交集、R 排除、两槽跨度在叶边界上 | 没打中 | 交集、两步排除 R、两槽不跨叶都读码核过 |
| 攻击面 1 新打中：单槽的提交内生块先进空槽最多的叶，拆掉全池最后一对，排在后面的两槽 inode 叶容器取不到 | 打中（推的，模型待跑），不分辨臂 | 改前的政策在镜像形状上一样拆对；病根是准入 ② 只数槽不数对、取落点政策都不看会不会拆掉最后一对 |
| 攻击面 2 走满 64 轮 | 对改前两条臂构造得出（推的）；对 as-coded 构造不出 | 2²⁰ 单元区每叶一个单槽、顶叶两对：as-coded 在第一个取落点的轮先撞上拆对拒绝 |
| 攻击面 2 来回换 | 构造不出（推的） | 跨轮空闲图不变、重写集只增，单叶与成对叶的进出推下来都单调；无记录叶那一角没核完 |
| 攻击面 2 演练与真发 | 没打中 | 真发的隔离释放不改挡位，两边读同一串角色、同一份空闲图 |
| 攻击面 2 `recovery.rs` 预演与真发 | 没打中（产品路径） | 预演会落到回落的几何上，真发的第一个文件先被合取 7 拒 |
| 攻击面 3 绕开 `set_blocking_bit` 的路径 | 没打中 | 三张位图的赋值在 `allocator.rs` 里只有 `set_blocking_bit` 那三行，字段私有、子模块只有单测 |
| 攻击面 4 钉值与上界 | 打中（推的，待跑）三处 | A4c 随机历史那条断言没在新政策上跑过；`model.rs` 上界文档与 `admission.rs` 两处注释已成假话 |
| 检查单 8 格 | 成立 4、不成立 2、推不出 2 | 见「检查单」一节 |

## 攻击面 1：过度外推与越位

### 1a 各盘自己答的回落落点不同而池里有共同空槽：没打中

逐句对用户那句话与代码（kb 字面还是改前的 `.claude/kb/decisions/03-空间分配.md:168`：「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」，写回随判决，这一格判代码对不对得上用户的新句）：

- 「只有每块盘都答回落时才进池级政策」：`crates/singlefs-core/src/allocator.rs:2859`：「let every_device_falls_back = answer_per_device.iter().all(|(_, answer)| {」，取不到时原样交回各盘答案 `crates/singlefs-core/src/allocator.rs:2886`：「None => answer_per_device,」。一块开段一块回落不进池级政策、照旧在 `record` 之前拒（`crates/singlefs-core/src/allocator.rs:2898`：「DeviceAgreement::AnswersDiffer(answer_per_device) => {」）。
- 各盘自己的答案仍是改前那一判（`crates/singlefs-core/src/allocator.rs:1307`：「match self.lowest_empty_segment_outside_the_region(excluded_region) {」），所以「这块盘答不答回落」与改前逐位相同。
- 推论（读码推的，用来缩攻击面）：**政策只在某一次取落点时每块盘都答回落才起作用**；一段历史里一次回落都没有，改前改后取的每个落点逐个相同。第 4 节挑「会变的测试」就按这一条筛。
- 「各盘答案不同、池里有共同空槽 ⇒ 现在成功、改前拒」：用户那句话的字面是「每块盘都答回落时，池里先取…每块盘都空的最低落点；没有再取每块盘都空的槽最多的那片叶里的最低落点」——条件只有「每块盘都答回落」，不要求各盘自己答得一样，所以这一形在字面之内。已定项 8 末段 `.claude/kb/decisions/03-空间分配.md:173`：「第一版盘不等大时，各盘落点答案必须相同；小盘没有全空段而大盘有时，在动任何状态之前拒绝」——池级落点每块盘同一个槽（`crates/singlefs-core/src/allocator.rs:2880`：「Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(」那一处把每块盘的答案都换成同一个值），「答案相同」成立；「小盘没有全空段而大盘有」那一形大盘答开段、不进池级政策，照拒。我找不到与末段冲突的输入。
- 推的补一句：每块盘都答回落而池级取不到时，各盘自己的最低空槽不可能相同（相同就说明那个槽每块盘都空、池级第二步一定扫得到它所在的叶），所以这条路恒落到 `AnswersDiffer`，报的是 `CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined`——真实原因是「没有每块盘都空的槽」，报的名字说的是段对齐没定。不改结局（拒、在动状态之前），只是原因名不再是实话，归检查单「类型与名字」那一格。

### 1b 交集、R 排除、两槽跨度在叶边界上：没打中

- 交集：`rewrite_set_leaves_of_the_roles` 按每块盘自己的层级 0 节点取叶序号（`crates/singlefs-core/src/transaction.rs:2532`：「Some(position.index_in_device)」），`present_on_every_device` 取交集；叶序号与分配器同一个算法（`allocation_record_tree.rs` 的 `leaf_of_slot` 用 `slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS`，与 `allocator.rs` 的 `slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS` 同式）。
- R：两步都经 `is_whole_span_open_for_commit_generated` 排除活 R（`crates/singlefs-core/src/allocator.rs:3025`：「device.is_whole_span_open_for_commit_generated(」与 `:3050` 同一个判法），R 按槽号不分盘（`logged_operations.rs` 的 `contains_slot` 只看槽号，两盘同槽时罩住每块盘的同号槽），与「各盘答不答回落」那一判（也排除 R）口径一致。合取 5 `.claude/kb/decisions/03-空间分配.md:146`：「| 5 | 不在 D26（后台整理与放置回收） 已定项 1 保护期内的 `R` |」在两步上都守着。
- 两槽跨度：叶罩 812 槽、起点 812 × k 是偶数；单元区起点在 64 槽段边界上也是偶数；两槽单元按 2 步进（`crates/singlefs-core/src/allocator.rs:3058`：「candidate += footprint.alignment_in_slots();」），跨不了叶；单元区末尾是奇数槽时 `candidate + footprint.slots() <= self.unit_area_end_slot()` 挡住。盘不等大时叶序号取各盘并集、小盘外的叶上界读成 0 被跳过。

### 1c 新打中（推的，待模型跑）：空槽最多的叶拆掉全池最后一对，两槽的 inode 叶容器取不到

形状 `pair-at-the-bottom-of-the-richest-leaf`（锻造，见模型）：除叶 12–14 外整片占满，每块盘都没有全空段；叶 12 只留一对（数据单元取走）；叶 13 留 20 个奇数偏移单槽；叶 14 留一对（偏移 400–401）加 21 个奇数偏移单槽，共 23 槽、是空槽最多的叶，它最低的空槽正是全池剩下的唯一一对的起点。

走一遍（as-coded，每一步指到许可它的那一句）：

1. 角色次序里数据单元最前，取叶 12 那一对（用户数据落点，这一轮没改）。
2. 树 ID 升序、extent 树（65536）排在 inode 树（65537）前面：extent 树根兼叶是单槽提交内生块，先取落点。重写集的叶（上一版节点所在的低处叶）全满 ⇒ 取空槽最多的叶 14 里最低的空槽 = 那一对的起点（`crates/singlefs-core/src/allocator.rs:2964`：「fn lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(」按单槽数选叶、在叶里取最低）。
3. inode 叶容器是两槽、要 32768 对齐的一对：重写集的叶没有对；第二步逐叶找「放得下这个跨度」的叶，一片都没有 ⇒ 池级 `None`；各盘自己的最低对也没有 ⇒ `NoAnswerOnAnyDevice` ⇒ `PublishError::PlacementRefused { unit: InodeLeafContainer…, refusal: NoFreeSlotOnAnyDevice }`，在固定点的演练里交回。
4. 同一形上 lowest-only（改前）：单槽取全池最低的空槽 = 叶 13 的单槽，那一对留着，inode 叶容器取到；之后几轮定下、被合取 7 拒（锻造盘上没有全空段）。prefer-then-lowest 同（后几轮重写集里叶 13 序号小，单槽先进叶 13）。

四句：

- 分不分辨臂：这一形分辨（as-coded 拒、改前定下）。但镜像形 `pair-at-the-bottom-of-the-lowest-leaf`（那一对在最低的叶里、空槽最多的叶全是单槽）上推的是改前两条臂第一个取落点的轮就拆对被拒，as-coded 第二个取落点的轮也被拒（那时那片叶已进重写集、重写集那一步按叶序号升序先取它最低的空槽）——两个方向都有被打中的形，**病根在各臂共用的前提**：准入 ② 只数槽、不数对（`crates/singlefs-core/src/admission.rs:1054`：「/// ② 只数槽：两槽的 inode 叶容器要成对、各盘的落点要同槽（开段各盘一致、回落各盘同槽），这两样今天的落点规则里有、这一判不判——」），取单槽落点的政策不看会不会拆掉最后一对，而两槽单元排在单槽单元之后取。按 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」表里「打中不分辨臂」那一行，不拿它判这一轮的臂，另立一笔账。
- 系统当时看不看得到：看得到，空闲图上对与单槽一清二楚。
- 满足的是哪一句：这一形用的是覆盖写（前台写入），改前改后都会被合取 7 拒，不新增用户可见的拒。要成为用户可见的差别，得是合取 7 豁免的发布（删除、整理批、挂载与回退）里也有单槽在两槽单元之前取：删除改写 inode 叶容器与 extent 树，次序同上（推的，没造删除那一形）。那时它碰的是已定项 17「释放空间这个操作本身不需要申请空间」与已定项 9 第 1 条（不许假性 ENOSPC）。
- 改法在打中的格上还中不中：跑前条款没有给改法。我提的（只在我的模型上推过、被攻过零轮、没实现没跑）：单槽在一片叶里取「不是一对空槽前一半」的最低空槽，叶里只剩成对的空槽才拆对；或准入 ② 按对数核两槽单元。前者两条臂都能装，对两种形状都推的不中；后者是准入改动，不在这一轮。

推翻条件：模型 as-coded 臂上这一形报的不是 `placement-refused unit=InodeLeafContainer…`，或 lowest-only 臂上报 `placement-refused`。

## 攻击面 2：固定点终止与演练真发一致

### 2a 走满 64 轮：对改前两条臂构造得出（推的），对 as-coded 构造不出

形状 `one-unaligned-slot-in-every-leaf-and-two-pairs-in-the-top-leaf`（2²⁰ 槽单元区，约 1280 片叶）：叶 12 起每片叶只留偏移 401 一个单槽，单元区最后一片整叶另留偏移 400–403 两对。

- 计数论证（推的）：每一轮演练从同一份空闲图起（`crates/singlefs-core/src/transaction.rs:14075`：「let mut rehearsal = allocator.clone();」，释放只改记录、槽进 defer 仍挡着，`crates/singlefs-core/src/transaction.rs:14077`：「rehearsal.release(*placement, plan.txg);」）；重写集里每片稀疏叶提供 1 个槽、自己又是一个要取落点的角色，外加记账树、映射树、树表、根、上一层节点与上一版那几片满叶这些「只吃不给」的角色 ⇒ 每一轮缺口至少是那几个角色的个数，缺口落进新叶、新叶下一轮进重写集，重写集每轮严格变大、永远定不下来。2²⁰ 单元区上 64 轮用掉的单槽推的不到 1000 个，先撞上 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`（`crates/singlefs-core/src/transaction.rs:14165`：「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」）。这一论证对 lowest-only 与 prefer-then-lowest 两条臂成立（那一对两槽单元每轮都取得到顶叶的第二对：单槽一律先进序号更低的稀疏叶）。
- as-coded 在这一形上先撞上的是 1c 那一种拒：数据单元取走顶叶第一对之后，顶叶剩 2 槽、是空槽最多的叶，extent 树根拆掉第二对，inode 叶容器取不到（推的）。所以对 as-coded 我没造出「走满 64 轮」，造出的是「先拒」。
- 把顶叶换成只留成对空槽、稀疏叶每叶 2 槽（`an-aligned-pair-in-every-leaf`）：每片新叶给 2 槽、只吃 1 个角色，缺口逐轮收窄，推的几轮定下，三条臂都是。
- 四句：不分辨臂（改前两条臂走满 64 轮，as-coded 换成另一种拒，三条都不成功）；系统看得到空闲图；满足的是旧论证「这一半的上界没有推出来」那一句的反面例子，与 64 这个数无关——任何有限轮数上限都会被这一形打穿；跑前没有改法条款。这一形的前提「每一片叶都只剩一个共同空槽、没有一片叶空槽多」在真历史里走不走得到，我没造（锻造只动空闲图、不写记录，记录不在那些叶里，与真盘不同：真盘上那些叶本来就有记录，它们不是「新节点」，但仍进重写集、仍是角色，计数论证不变，推的）。

推翻条件：模型里 lowest-only 或 prefer-then-lowest 在这一形上报 `settled…`，或 as-coded 报 `did-not-settle-after-64-rounds`（那就是 as-coded 也走满了，结论加强）。

### 2b 重写集没变、这一版节点来回换：构造不出（推的）

试过的推导（没跑）：固定点每轮带两样——重写集 `rewritten`（只增）与这一版的节点 `nodes_after`（`crates/singlefs-core/src/transaction.rs:14121`：「nodes_after = holding_records;」）。演练每轮的空闲图相同（见 2a），角色次序相同，所以一轮取的落点只是「重写集的叶（按 `nodes_after` 过滤）」与角色个数的函数。真盘上上一版有记录的叶永远在 `holding_records` 里（记录只改写、不删），能进出 `nodes_after` 的只有「上一版没有记录、这一轮才装进记录」的叶。设这样一片叶 X：X 在计划里时它是重写集的叶，排在它前面的重写集的叶要是装得下全部角色，X 一个落点都分不到、下一轮掉出去；掉出去之后角色少了 X 自己那一个，前面的叶照样装得下，没有缺口落进空槽最多的叶，X 不会回来。X 分得到落点时它留着、下一轮照旧分得到。两种情形都单调，推不出两轮一个周期。我没核完的角：同一轮里两片这样的叶 X、Y，一片的进出改变另一片能不能分到「排在两槽单元之后」的那几个单槽（两槽单元只认成对，单槽数相同而对数不同的两片叶互相顶替）；锻造盘（只动空闲图、不写记录）上每一片叶都是「上一版没有记录」的叶，这一角在锻造盘上比真盘宽。没造模型臂测它。

### 2c 演练与真发取同一串落点：没打中

- 两边读同一串角色：固定点最后一轮 `crates/singlefs-core/src/transaction.rs:14096`：「rewrite_set_leaves_of_the_roles(&resolved.rewritten_roles, allocator);」，真发 `crates/singlefs-core/src/transaction.rs:15758`：「let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(rewritten, allocator);」，`rewritten` 就是定下那一轮的 `resolved.rewritten_roles`；写行那一路同（`:2054` 与 `:2210`）。`allocator` 在这里只用来列盘，不读空闲图。
- 两边的空闲图相同：真发释放时核出对不上的那几份留在已分配（`crates/singlefs-core/src/transaction.rs:15738`：「allocator.release_leaving_the_record_allocated_on(」），演练全部释放；产品路径上释放只改记录、槽进 defer 仍挡着，两边挡位逐位相同，池级政策读的只是挡位与 R。只供测试的复用窗口置 0 时释放当场空出槽，那一格两边会不同（实现员文档已写明，测试开关，不算）。
- 次序：两边都是先释放、再搬迁目的地、再按角色取落点（演练 `crates/singlefs-core/src/transaction.rs:14098`：「allocate_relocation_destination(&mut rehearsal, relocation, &rewrite_set_leaves)?;」在角色之前）。

### 2d `recovery.rs` 预演（空重写集）与真发：没打中（产品路径走不到）

- 预演只建单元区开头两段的空闲图（`crates/singlefs-core/src/recovery.rs:6586`：「let rehearsal_bytes_on_each_device =」），用空重写集取提交内生块（`crates/singlefs-core/src/recovery.rs:6617`：「rehearsing.allocate_commit_generated(footprint, new_pool_file_creation_txg)」），只交回「每盘几条记录」当下限（`:6798` 那一处）。
- 两边能不同的唯一一形（推的）：第二段装不下新池新建文件的提交内生块、预演走回落，且单元区跨两片叶（非默认环，例如 1 GiB 环单元区从 66560 起、叶 81 只罩到 66584）——预演取空槽最多的叶（叶 82）、真发先取重写集里 mkfs 记录所在的叶 81，节点数差一片叶，预演的下限高于真发写出的记录数，恢复会把合法镜像判成「少于这套几何下新池新建文件之后每盘的落点数」。
- 产品路径走不到：这种几何上第二段不是整段，全空聚簇段数是 0，第一个文件是前台写入，合取 7 在固定点之后先拒（`crates/singlefs-core/src/transaction.rs:13367`：「if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {」、`:13485`），第一个文件写不出来，就没有要恢复的那种镜像。只供测试的「准入跳过」开关下走得到，那一格不算。

## 攻击面 3：每片叶没挡槽数的增量维护——没打中

- 三张挡位图在 `allocator.rs` 里的赋值只有 `set_blocking_bit` 那三行：`crates/singlefs-core/src/allocator.rs:658`：「SlotBlockingBit::Allocated => self.allocated[index] = is_set,」、`crates/singlefs-core/src/allocator.rs:659`：「SlotBlockingBit::IsolatedByTheShadowLedger => self.isolated[index] = is_set,」、`crates/singlefs-core/src/allocator.rs:661`：「self.held_until_floor_takes_effect[index] = is_set;」；文档自己也写着 `crates/singlefs-core/src/allocator.rs:653`：「/// 改一槽的一种挡位，同时增量维护没挡的槽数与没挡的槽对数（每段一份与总数）：三张位图只经这里改。」。
- 怎么核的（命令现跑）：`grep -nP '(allocated|isolated|held_until_floor_takes_effect)(\[[^\]]*\])?\s*=[^=]'` 在 `allocator.rs` 上只命中上面三行与一行无关的局部变量（3604，测试里的 `let isolated = …`）；`.fill/.clear/.push/.resize/.iter_mut/.swap/.get_mut/.as_mut/.retain/.drain…` 与 `&mut self.<位图>`、`mem::swap/replace/take`、`*self =`、`unit_area_(slots|start) =` 都零命中（退出码 1）；`DeviceFreeMap { … }` 字面量只在 `with_unit_area_start` 里（叶计数在同一处按单元区现算起步值）；字段私有，`allocator.rs` 里的子模块只有 `mod tests`；别的文件零处碰这几个字段（`grep -rn unblocked_slots_per_allocation_record_tree_leaf crates/` 除 `allocator.rs` 外只命中 `crates/mutations.tsv` 那一行变异）。
- 逐条路径：挂载建空闲图走 `DeviceFreeMap::new` + `mark_allocated`；回收、`mark_reclaimed`、抬 F 的 `hold_until_floor_takes_effect` / `release_holds`、影子账 `isolate*` / `clear_isolation_of_slot`、释放 `mark_released`、回退复活——都落到 `set_blocking_bit`（`isolate_slot_at_index`、`clear_isolation_of_slot`、`hold_until_floor_takes_effect`、`release_holds` 四处调用现查过）。克隆是 `#[derive(Clone)]`，向量整份拷，叶计数与位图同拷。
- 下标：`allocation_record_tree_leaf_position_of_index` 用起点所在叶作 0，起步向量按 `start / W ..= (end − 1) / W` 建，任一单元区下标落得进；`unblocked_slots_in_allocation_record_tree_leaf` 越界读 0。
- 「上界与逐槽数不一致时取到的叶对不对」：上界只用来剪枝（`crates/singlefs-core/src/allocator.rs:2994`：「if upper_bound_of_the_open_slots <= most_open_slots_so_far {」），选叶用的是逐槽数（`:2997` 那一行 `let open_slots =`）；上界取各盘最小值、R 里的槽也算进去，只会偏大，偏大只多数一片叶、不改选中的叶。要它选错叶，得上界偏小——上面没找到能让它偏小的写法。

## 攻击面 4：钉着落点与单元数的测试、上界与注释

筛法：按 1a 的推论，只有走过回落的历史会变；一次回落都没有的用例（例如 `crates/singlefs-verification/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs:5`：「（`PoolAllocator::commit_generated_placements_that_fell_back_to_the_lowest_free_slot`）是 0；哪天走进回落，用例红，回来重看 I-5.5 的开放段认法。」那一族）改前改后逐字节相同，不用重跑。

| # | 位置 | 会红还是已成假话 | 判定 |
|---|---|---|---|
| 1 | `crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs:1096`：「/// - 单元区 2600 槽、逼满、种子 13、200 步：情形 ②（没有全空段，回落到最低空槽、一个单元一处）。A4c 量的那一段单元区从 50176 起，」 | 这段随机历史走回落（文档自述），没在新政策上跑过（背景材料第二节列的已跑目标里没有这个目标）。断言 `crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs:1153`：「more_than_two_leaves_on_one_device > 0,」要「至少一次空发布一块盘改的叶多于两片」——新政策先取重写集的叶、把回落收进少数几片叶，推的这一条可能变红；落点一变，记录口径的全空段数跟着变、合取 7 的拒与不拒跟着变，整段历史分叉，第 1161 行那条「固定点槽数不多于 ckpt_cost」也要重判。文档里的「200 步里少扣 4 次（第 22、58、137、154 步，实测）」是改前的读数，新政策下不再是现状 | 会不会红推不出，待跑（命令见文末第 2 条）；红了是「判别力丢了」（这条用例本来要的那种情形在新政策下不再出现），要找新的种子或改断言，不是政策错 |
| 2 | `crates/singlefs-verification/src/model.rs:4210`：「/// 记录只住在落点上；落点按 D3（空间分配） 已定项 10 从最低处取（开新段取最低的全空段，回落取最低的空槽），每个落点至多多开一个」 | 已成假话（回落不再取最低的空槽）。上界本身（推的）：每块盘都没有全空段时，每段要么有分配位、要么有隔离或扣住位；全靠分配位挡着时 P ≥ 段数、`64 × P` 罩满单元区，上界照旧成立。靠隔离或扣住挡着的段不进 P，那一角上空槽最多的叶可以在 `64 × P` 之外；改前最低空槽要越出 `64 × P` 得有整段被隔离挡满，改后每段一个隔离槽就够——这一角比改前宽。理想模型几乎不模隔离（`model.rs` 里「隔离」相关只 3 处），它的取样史上走不走得到推不出 | 注释不成立；上界在隔离撑起的非空段那一角推不出 |
| 3 | `crates/singlefs-core/src/admission.rs:1188`：「在开放段装不下开新段、没有全空段回落到最低空槽、」与下一行 `crates/singlefs-core/src/admission.rs:1189`：「/// 换下很久之前落在别处的节点这三种情形下少扣（量到一块盘改 3–4 片叶），」 | 前一句已成假话；「量到一块盘改 3–4 片叶」是改前政策下量的数，新政策下没量 | 注释不成立；扣项本身取几何上整棵树（K1），不受落点政策影响（推的），不会因此少扣 |
| 4 | `crates/singlefs-core/src/admission.rs:981`：「开放段 bump、开全空段、回落到最低空槽挡的都只是这三种位」 | 说挡位的那一半仍对，「回落到最低空槽」几个字已不是实话 | 注释措辞过时 |
| 5 | 检查单「带 --item 的命令与项」那一格的漏项（见下一节第 7 格） | 走回落而没列进 8 项 | 见检查单 |
| 6 | checker 一侧 | `grep -rn 回落 crates/singlefs-checker/src` 只命中 `walk.rs` 一处（说的是「F 不回落」，与落点无关）；I-5.5 的近似「开放段拿树表单元所在的段当它（…提交内生块回落时会误排…）」只在从 mkfs 水位建树那一版上判，那一族由上面那条回落计数用例钉成 0 次回落，政策变了也走不进 | 没打中 |

## 检查单八格

| 格 | 判定 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 新加分支里没有 `todo!`、`unreachable!`、产品 `assert!`（diff 里新加的 `assert` 都在 `mod tests`）。两处新 `expect` 的范围成立：`crates/singlefs-core/src/allocator.rs:698`：「.expect("叶下标装得进 usize")」的值不大于起步时按单元区建的向量长度（那个向量已经建得出来）；`slots_open_on_every_device_in_the_leaf` 数的是一片叶的槽，至多 812。顺带核出的一条死组合（不是新加的 `unreachable!`，不算打中）：每块盘都答回落而池级取不到时，各盘答案不可能一致（1a 末条），`crates/singlefs-core/src/allocator.rs:2886`：「None => answer_per_device,」之后走到「一致」那一支的组合走不到 |
| 类型与名字 | 不成立 | ① `crates/singlefs-core/src/allocator.rs:2880`：「Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(」装的是池级取的落点（`:2881` 那一行 `fallback_slot_on_every_device,`），成员名说的「最低空槽」在这里是假话；计数 `crates/singlefs-core/src/allocator.rs:2920`：「self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot += 1;」同。② 叶序号、槽号、槽数共用裸 `u64`：`crates/singlefs-core/src/allocator.rs:203`：「leaf_indices: BTreeSet<u64>,」，`crates/singlefs-core/src/allocator.rs:704`：「pub fn unblocked_slots_in_allocation_record_tree_leaf(&self, leaf_index: u64) -> u64 {」（参数是叶序号、返回是槽数，同一个类型），`crates/singlefs-core/src/allocator.rs:734`：「candidate: u64,」（槽号不用已有的 `SlotNumber`）。③ 1a 末条：池级取不到时报的 `CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined` 说的是段对齐，真实原因是没有每块盘都空的槽 |
| 不兼容旧代码与旧格式 | 成立 | `grep -nE 'legacy|old_format|migrat' research/prompts/_d3-fallback-richest-leaf-code-r1-diff.md` 现跑零命中、退 1。`try_allocate_commit_generated` 有非测试调用方（`recovery.rs` 新池新建文件预演经 `allocate_commit_generated` 调它），文档写的就是这件事：`crates/singlefs-core/src/allocator.rs:2811`：「/// 不在发布路径上取落点的调用方走这里（新池新建文件的预演、测试）。」——是空重写集的入口，不是为旧调用方留的兼容 |
| 不新建判定库 | 成立 | diff 只改 `crates/singlefs-core/src/` 两份；新验证包测试文件里 `prune|rocksdb|/home|std::env|env!(` 零命中（`grep -cE` 现跑，0） |
| 所有任务多机可用 | 成立 | 同上一格的 grep：不读主机路径与环境；分配器单测在进程里造 `PoolAllocator` |
| 测试优先多线程 | 不成立 | `crates/singlefs-core/src/allocator.rs:4745`：「for (footprint, expected) in [」按参数循环，每一轮重建被测状态（`crates/singlefs-core/src/allocator.rs:4761`：「let mut pool = pool_with_free_slots_left_in_three_leaves();」），照 `.claude/rules/test-script-discipline.md:13`：「- 一条用例一个场景。按参数循环、每一轮重建被测状态的，拆成一个带参数的函数加每个取值一条用例」该拆成一槽、两槽各一条；同一条用例后半段又换了重写集（叶 70）再建一次池，是第三个场景 |
| 带 --item 的命令与项 | 推不出（算打中） | 实现员按源码字面挑的 8 项挑不全：`crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs:21`：「//! 前缀到第一批同这条流，之后在会话的分配器上把 R 之外的每一个空槽隔离（不写盘、不进录制流），最后一批在 R 之外取不到落点、暂停，」——R 里还有未释放的记录（同文件第 291 行起那条用例断言），R 之外每个空槽都隔离 ⇒ 每块盘都没有全空段，删意图那次的提交内生块走回落；它所在的合表项 `compaction-paused-short-of-space-dropping-the-intent`（`.claude/gate.d/crash-point-replay-items.tsv:78`：「compaction-paused-short-of-space-dropping-the-intent」）不在 8 项里。推的：R 在默认几何下整段落在叶 61 里，回落取到的仍是 R 里最低的那几槽，录制流可能不变；变没变要跑。更一般地，靠关键字挑不出「这条流里真走过回落」，可靠的挑法是给每个崩溃枚举用例的录制流读一次分配器上的回落计数 |
| 新崩溃流多罩了什么 | 推不出（算打中） | 「回落进空槽最多的叶之后那片叶头一次被重写」那一形，要有一条流在回落时单元区跨两片以上的叶、且空槽最多的叶不是最低空槽所在的叶。384 槽小盘那一档单元区 [50176, 50560) 跨叶 61 与叶 62（叶 62 从 50344 起），近满池那两项正是这一档，回落可能改落叶 62——那就是新形；但这几项在主 agent 的重放里只缩到展开深度 5、三项在合并那一步没判成（背景材料第二节），罩没罩到推不出 |

## 没打中的形状

- 越位：一块盘开段一块回落（照拒，代码没改那一支）；盘不等大、小盘没有全空段而大盘有（大盘答开段，照拒）；盘不等大、两块都没有全空段（池级只在两块盘的单元区交集里取，小盘外的叶上界读 0）；R 罩着重写集叶里全部空槽（两步都排除，实现员单测钉着）；两槽单元在叶边界与单元区末尾奇数槽上（读码，跨不了叶、越不过末尾）；重写集交集只在一块盘上有的叶（交集去掉）。取样范围：读码推导，没有跑。
- 固定点：一片无记录叶进出 `nodes_after` 的两种情形（单调，推不出周期）；`an-aligned-pair-in-every-leaf`（推的几轮定下）。演练与真发：隔离释放、整理搬迁的次序、写行那一路。`recovery.rs` 预演：默认环下单元区要不到 128 槽才会回落、而且整段落在叶 61 里（50176 起、叶 61 到 50344 止），落点同改前；跨叶的几何只在非默认环、且第一个文件被合取 7 先拒。
- 增量计数：赋值、批量改、整份替换、改单元区几何、克隆、越界读，七种写法逐个 grep。
- 钉值：一次回落都不走的用例族（回落计数钉成 0 的那一族、准入几何格那几格大盘）按 1a 的推论不变；`unequal_devices.rs` 第 388 行那个每盘答案 `FallBackToLowestFreeSlot(SlotNumber(196_638))` 是一块开段一块回落那一形的每盘答案，不进池级政策，不变（主 agent 任务 13430 退 0 与此相符）。

## 这条腿自己的限度

- 模型一行都没跑：五个形状、三条臂的结局全是推的；尤其 1c 与 2a 的「extent 树根排在 inode 叶容器前面取落点」「覆盖写会重写 inode 叶容器」两条，我只按 D3（空间分配） 已定项 10 ⑤ 的次序与角色名推，没在代码里逐个角色核 `resolve` 排出的次序。模型跑出来与预测不符，以跑出来的为准、对应格照推翻条件改。
- 锻造只动空闲图、不写分配记录（照实现员的做法）：锻造出来的叶在上一版里没有记录，是「新节点」，与真盘上「有记录的叶」进出重写集的方式不同（2b 末尾那一角）；真历史里走不走得到这几种盘面，我没造历史。
- 1c 的用户可见后果要靠合取 7 豁免的发布（删除）来显现，模型用的是覆盖写，只在固定点演练那一层分得出臂；删除那一形没造。
- 模型源码第一次写成时一次落盘 226 行，越过「每次写进文件不超过 150 行」；之后删掉按两段重写，现在的文件是分段写的。`tools/rerun.sh` 从模板拷来只换了 `rerun_steps`，模板自带的 `--selftest` 看的是 `$WORK/tree`，换了步骤之后那条自证不再适用（模板的自证本来就在模板上跑）。
- 攻击面 4 只挑了「走回落」那一类；落点政策之外（例如准入那一侧）的钉值没扫。

## 要主 agent 跑的命令

每条一行一个交任务的命令，结局看各自任务末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`。

1. 攻方模型复跑（在仓根起；外层先删 `-run` 运行目录再调 `tools/rerun.sh`，三条臂在同一份拷贝里依次装、依次编；cargo 那三行的范围声明写在 `rerun.sh` 里的 `export SINGLEFS_RUN_SCOPE=…`）：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/d3-fallback-richest-leaf-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-attack/copy /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-attack-run /home/fy5090/code/singlefs/research/prompts/d3-fallback-richest-leaf-code-r1-opus-model`
   看：`exit=0`；日志里 `name=exit_codes as_coded=0 prefer_then_lowest=0 lowest_only=0`、`name=probe_lines count=15 expected=15`，15 行 `PROBE_RESULT arm=… shape=… outcome=…` 与上面预测表逐格比；`every_device_lacks_an_empty_segment=true` 每行都要是 true，不是的那一行锻造没造成（作废那一行）。对照形 `sparse-leaves-below-a-rich-region` 在 as-coded 臂上不是 `rounds=4 … outcome=settled-then-refused-by-the-empty-segments-floor` 的，整趟作废（装置不等于实现员的目标）。退 2 是 `patch_the_copy.py` 的旧串命中次数不是 1（冻结副本与开工快照不同）。
2. A4c 随机历史那条（在仓根起，主工作区已打进补丁）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 admission_checkpoint_cost_per_device_paths 里点名的一条; 范围=1 个测试; 为什么=这条用例的单元区 2600 槽那段历史走回落，断言至少一次空发布一块盘改的叶多于两片、固定点槽数不多于 ckpt_cost，新政策上没跑过' task-server/task submit -- cargo test -p singlefs-verification --test admission_checkpoint_cost_per_device_paths -- --exact empty_publishes_of_the_histories_where_one_device_rewrites_more_than_two_leaves_fit_in_the_whole_tree_checkpoint_cost`
   看：`exit=0` 且 `test result: ok. 1 passed`；红了看 `task-server/task log <号> --all` 里 `name=a4d-history history=unit-area-2600-filled-seed-13 … more_than_two_leaves_on_one_device=` 那一行的数（0 就是攻击面 4 第 1 行说的「判别力丢了」）。
3. 只剩 R 有空槽那条流的形状用例（在仓根起，小、不枚举崩溃状态）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream 里的形状用例; 范围=1 个测试; 为什么=这条流删意图那次每块盘都没有全空段、提交内生块走回落，回落政策改了之后它断言的落点在 R 里没在新政策上跑过' task-server/task submit -- cargo test -p singlefs-verification --test crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream -- --exact the_flow_with_only_the_region_free_drops_the_intent_with_its_placements_in_the_region`
   看：`exit=0`、`1 passed`。
4. 那一项的崩溃点重放（在仓根起；状态数大的话主 agent 照它的 `ITEM_PLAN` 定要不要加展开深度）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃点重放入口带 --item; 范围=1 项; 为什么=这一项里只剩 R 有空槽那条流删意图那次走回落，实现员点名的 8 项里没有它' task-server/task submit -- bash scripts/crash/crash-point-replay.sh --item compaction-paused-short-of-space-dropping-the-intent`
   看：`exit=0`，那一项一行的结局；红了先分钉值变了（落点变了）还是 checker 判红。

## 仓副本与草稿

- 没删 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r1-attack/copy`（54M，`du -sh` 量的）：它是第 1 条命令的冻结副本，`.deliverables` 写成「交接」；主 agent 复跑与判决用完再删。没有自己建的编译目录（没编过）。
- 草稿目录里另有 `minimal-copy.txt`（建副本的输出）。

## 没做什么

- 没往任务服务交任何任务，模型、测试、崩溃点重放都没跑（命令在上一节）；判决要引的数一个都没有，全部「推的」。
- 没判正推那几格，没读禁读清单里的正推腿产出；没替主 agent 采纳；没改 kb、代码与测试。
- 1c 删除那一形、2b 两片无记录叶互相顶替那一角、真历史上的可达性，都没造。
