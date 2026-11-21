# m4-publish-without-whole-version-copy-design-r2 辩方腿报告

日期：2026-11-18。设计推论轮第二轮，辩方腿：复核第一轮判决 `research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`（下称第一轮判决），替被判打中的乙、丙辩护。方案按 `crates/` 今天的实现谈，没改 `crates/`。

标记：【读】现读文件或命令输出，给文件与行；【量】命令此刻数出来的；【推】推的，没量过，写明依据与推翻条件。
没写模型、没起探针：下文所有代价式子都是【推】，没有一个数出自核心层。复跑命令：无。探针的内存与时间：无。
行号是主工作区此刻的树；`crates/singlefs-core/src/mount.rs` 在工作区里有没提交的改动，行号按此刻的文件。
外部包的数据（索引、许可、源码）是 2026-11-18 现取的，原样存在草稿目录 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-sonnet/`（`index/` 是索引与 crates.io 接口的返回，`crates-dl/` 是四个包的 `.crate` 与解开的源码）；四个 `.crate` 的 sha256：
```
d0acd33ff0285af998aaf9b57342af478078f53492322fafc47450e09397e0e9  im-15.1.0.crate
46bad832b9b463ed9398b8506488cc2e3b897a9d44f13af115f382aac71f4fec  imbl-7.0.2.crate
0f5098ef24cf336da7cdf71d9e5aa8ba82aaef01b7413d754c11b6857c2434c6  immutable-chunkmap-2.1.5.crate
e025feb26210bc196b908e72deb063b1b4000754304341cbc168a1e72c857ebc  rpds-1.2.1.crate
```
副本里的造池提速补丁 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/patch/crates.patch`（sha256 `90360588d7362cb99278490b27966645a720687344dc1b0cb308c0da0c926878`）只读；下文引它的行号是补丁文件自己的行。

## 判定一览

结论词只用三个：「够得着」（判决的依据站得住）、「够不着」（依据没打到那个形态）、「推不出」（缺的东西写明）。

| 条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| 乙「一次调用交回几版、返回后逐版读全版」 | 对纯差量形态够得着；对「撤销日志加版本句柄」（下称乙强）够不着 | 一次交回几版属实：`crates/singlefs-core/src/mounted_session.rs:2040`；产品侧读这些版本的几处只取 `writes`、`root`、条数、`Arc::ptr_eq`：`crates/singlefs-core/src/mount.rs:8325`、`:8326`、`:8396`、`crates/singlefs-mount-runtime/src/lib.rs:523`、`:657`；逐版读全表的是验证包 `crates/singlefs-verification/src/history.rs:6042` 一带（二·1） |
| 乙「8 处 `assert_eq!(pool.output, current_before, …)` 会变成比差量」 | 8 处存在（量）；「会空转」只在版本句柄浅拷贝时成立，对深拷快照够不着，代价是改这 8 处与同形写法 | `crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:375`、`:429`；被测调用拿 `&mut`：`crates/singlefs-core/src/mount.rs:10904`（二·2） |
| 乙「冻结发布要发布前后两份全态并存」 | 对今天的写法够得着；对「必须两份全态」够不着；空闲图六种改动没有撤销步这一半同样打中甲与丙 | `crates/singlefs-core/src/transaction.rs:1580`、`:1583`、`:1663`；背景材料「问题」第 3 条写的空闲图撤销步缺口（`research/prompts/_m4-publish-without-whole-version-copy-design-r2-background.md:17`）（二·3） |
| 丙「冷键沿链走，链长乘在发布路径对上一版的取值上」 | 对不压平的链够得着；对「定期压平」与「几何合并」（下称丙强）够不着「无界」，剩的是常数因子与均摊 | 攻方原文「要止住得定期压平…这是丙自己要加的一条路」：`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:85`；正推腿已写压平摊销：`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:70`（二·4） |
| 丙「只留最新一版时整条链被拽着」 | 对不压平的链够得着；对丙强够不着【推】 | 同上（二·4） |
| 判决「两腿一致打中」（乙、丙两行） | 只对没加强的形态一致；「丙强仍中」这一说，正推腿自己没打（它给压平标了均摊式子） | 同上（二·5） |
| 第一轮的打中是不是同样打中甲 | 逐条：同样打中三条共有格、`&mut` 走全版的「读满」、`units` 次序与下标、相等的实现、冻结里的空闲图；只打甲的是容器；不打甲的是多版读与链长 | 三·1 的逐条表 |
| 乙强、丙强补完之后与甲还差什么 | 乙强差：值语义从编译器保证换成撤销完整性、读过去的版本要倒推、里程碑六读写并发时要锁或物化；乙强占的便宜：容器零行、与分配器合一。丙强差：取值的常数因子、压平均摊与尖峰。全是【推】 | 三·2、三·3 |
| 整表断言：三种并存读法各丢什么 | 「恒开并存」合两个前提的字面、丢 O(R) 不降；「只留测试构建」丢前提二，且今天没有现成的「测试构建」开关；「恒开增量加测试参照」（副本补丁的写法）合前提二的位置与恒开（消息措辞变了），前提一的参照只在核心包单测里编，「照原失败路径」读成哪一种判决没写：推不出 | `crates/singlefs-core/src/transaction.rs:16244`–`:16254`；仓根 `Cargo.toml:28`；补丁 `crates.patch:7590`（四） |
| 容器：自己写还是引包 | 推不出（缺用户口径与依赖规则）；给出自写的行数估计与验法，给出四个候选各自的依赖、许可与缺的接口 | 五 |

「什么现象会推翻」逐条写在各节末尾；总的一条：若第三轮的核心层探针量出「一次发布改到的条数 d 接近全版条数」，乙、丙、甲在这一张表上都买不到东西，本表各行的比较一并作废。

## 一、两个加强形态的定义（我给的，下文按它们判）

乙强（写者持有的可变索引，加撤销日志与版本句柄）：
- 写者持有一份有序的可变索引（今天的 `Vec` 换成标准库的 `BTreeMap`，或原样的 `Vec`），每次发布记一份正反两向的差量（每个改到的键的旧值与新值）；
- 交回的「版本」是句柄（索引的引用、纪元号、本次的正反差量）；读最新一版直接读索引；读过去的某一版 = 把索引按日志从新到旧倒推到那一版，惰性做、做完缓存；
- 相等 = 两边各自物化再逐项比；失败换回与预演撤回 = 倒推本次发布的日志；冻结 = 留住正向日志、调用方一侧倒推回发布前，重发时正向重放。
- 用户给的另一个形态「调用返回的每一版都物化成全版」我判它不是加强而是回到今天：物化的单价就是 `admitted.units_assembly`、`admitted.allocation_records_sorted_copy` 那两段的内容，每次发布至少交回一版，每次发布仍付 O(N)，「一次发布的代价」那一格仍中。只有惰性形态补得上那一格，下文说乙强都指惰性形态。

丙强有两种，都只用标准库：
- 定期压平：版本 = 差量加指向上一版的 `Arc`，链长到 L 时合成一份平的基、新版不再挂旧链；L 取 √N 时取一键最坏 √N 跳。
- 几何合并：版本 = 若干不可变有序段的列表，每次发布压入一个 d 条的新段，相邻的同量级段合并（日志结构合并树的写法，删除用墓碑）；取一键看至多 log N 个段，均摊 O(d·log N)，最大一次合并的尖峰 O(N)，段不可变所以尖峰可以放后台。

## 二、对第一轮判决逐条：乙、丙的依据够不够得着

### 二·1 乙「一次调用交回几版、返回后逐版读全版」

【读】一次调用交回几版属实：`pub versions_published: Vec<Arc<TransactionOutput>>,`（`crates/singlefs-core/src/mounted_session.rs:2040`）。
【读】产品侧读这几版的几处只取写账、根的事务号、条数、指针相同：`.flat_map(|raised| raised.publishes.iter())`（`crates/singlefs-core/src/mount.rs:8325`）后接 `.map(|publish| publish.writes.clone()),`（`crates/singlefs-core/src/mount.rs:8326`）；`persisted.sort_by_key(|publish| publish.root.checkpoint_txg);`（`crates/singlefs-core/src/mount.rs:8396`）；挂载运行库只数条数：`.map(|raised| raised.publishes.len())`（`crates/singlefs-mount-runtime/src/lib.rs:523`）、`let publishes = u64::try_from(run.versions_published.len())`（`crates/singlefs-mount-runtime/src/lib.rs:657`）。
【读】逐版读全表的是验证包：`let outputs: Vec<&singlefs_core::transaction::TransactionOutput> = match published {`（`crates/singlefs-verification/src/history.rs:6042`），随后 `records_of_the_previous_publish = &output.allocation_records;`（`crates/singlefs-verification/src/history.rs:6097`）；模型对拍读每一版：`pub fn observed_root_of_file_version(output: &TransactionOutput) -> ObservedRoot {`（`crates/singlefs-verification/src/model_comparison.rs:181`）。
我数的范围：`singlefs-core/src`、`singlefs-mount-runtime/src`、`singlefs-background-gpu/src` 里 `versions_published|steps_published|\.publishes` 的命中，逐行看了带遍历与下标的那几处；验证包与两个测试包里同名命中共 598 行【量】（命令在「没做什么」一节），没逐行读。

判：「返回后逐版读全版」在产品侧不成立（读的是写账、根、条数），只在验证包成立。验证包读的是惰性倒推出来的过去版本，换成「按需物化」不改它的判法：`roots: vec![observed_root_of_file_version(version)],`（`crates/singlefs-verification/src/history.rs:6073`）交给 `judge_by_model` 的是对独立理想模型的比对，写者的日志倒推错了会在这里红，不是丢了检查。对纯差量形态够得着，对乙强够不着。
什么会推翻：若产品侧有我没数到的路径在调用返回之后读过去某一版的单元表（例如我没读的验证包之外的包），乙强要为它付 O(Σd) 的倒推，这一条改判「够得着」。

### 二·2 乙「8 处 `assert_eq!(pool.output, current_before, …)` 会变成比差量」

【量】`grep -rnE 'assert_eq!\(pool\.output, current_before' crates` 命中 8 行，全在 `crates/singlefs-verification/tests`（主 agent 已核同数）。
【读】8 处同形：先 `let current_before = pool.output.clone();`（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:375`），再调拿 `&mut` 的被测函数（`current: &mut TransactionOutput,`，`crates/singlefs-core/src/mount.rs:10904`），期望被拒，最后 `assert_eq!(pool.output, current_before, "{breakage:?}");`（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:429`）；同一个测试在 `pool.output.tree_identifiers.extent = TreeIdentifier(999);`（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:371`）自己改字段造故障。分配器那一半另有独立的一判：`let records_before = pool.allocator.records().to_vec();`（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:376`）对 `records_before.as_slice(),`（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs:432`）。

判：「会空转」只在版本句柄的 `clone()` 是浅拷贝时成立：`current_before` 与 `pool.output` 指同一份索引，被拒的调用把索引改了半截时两边仍相等，判别子观测不到。若这 8 处的快照改成物化出来的深拷（O(N) 只落在测试里），比较仍是「独立深拷对现行」，判别力回来；分配记录那一半本来就有独立的 `to_vec()` 快照（`:376`），不依赖这一条。所以对浅句柄够得着，对深拷快照够不着；乙强要付的是改这 8 处与同形写法（攻方数到的 15 行是上界，`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:220`）。
什么会推翻：若 `Clone` 必须保持深拷才能让产品里隐式 `clone()` 的几处（`crates/singlefs-core/src/mounted_session.rs:5371`、`crates/singlefs-core/src/transaction.rs:15822`）语义不变，则「句柄浅拷」本身就不成立，乙强要先把这几处改成显式的撤销记录（见三·2 第一条），这一条的代价大于改 8 处测试。

### 二·3 乙「冻结发布要发布前后两份全态并存」

【读】`version: PoolVersion,`（`crates/singlefs-core/src/transaction.rs:1580`）、`allocator_after_the_publish: PoolAllocator,`（`crates/singlefs-core/src/transaction.rs:1583`）；落盘失败时 `std::mem::replace(allocator, allocator_before_this_publish);`（`crates/singlefs-core/src/transaction.rs:1663`）：调用方手里回到发布之前，冻结的是发布之后。「发布路径冻结期间一个都不许进来」（`crates/singlefs-core/src/transaction.rs:1582`）。
判：「两份全态并存」是今天写法的事实，不是需求。需求是：调用方一侧回到发布之前、重发时能得到发布之后。记正反两向的差量能给出两份，各 O(d)。第一轮判决引的「回滚日志只给得出之前那一份」（`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md:64`）说的是只记撤销的日志，对正反两向的日志够不着。
真缺口是背景材料「问题」第 3 条那一句：实现员报空闲图（每盘位图）的六种改动没有撤销步（`research/prompts/_m4-publish-without-whole-version-copy-design-r2-background.md:17`，我没复核）。这一半对三条同样：正推腿自己写位图「不在这一题里，仍整份拷」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:164`），甲也不解。归「三条共有格」，不拿来判臂。
我没核的：冻结期间会不会有读路径同时要发布前与发布后的分配器（`refuse_while_a_publish_is_frozen`，`crates/singlefs-core/src/transaction.rs:1631`，拒的是发布，读没核）。这一点判「推不出」。
什么会推翻：冻结期间有读路径同时读两份状态，乙强要么整份拷（只在失败路径）要么放弃这一格。

### 二·4 丙「冷键沿链走，链长乘在发布路径对上一版的取值上」「只留最新一版时整条链被拽着」

【读】攻方自己写：「要止住得定期压平（压平那一次是 O(N)），这是丙自己要加的一条路，背景材料没写。」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:85`）。它的读数（4096 跳、16517088 步、8192 条）是没有压平的链上的，而且是腿自写的小模型、没在核心层量过（第一轮判决已写：`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md:41` 的「推的，攻方模型量的是自写模型」）。我没复跑那份模型，下面不引它的读数。
【读】正推腿的丙格已给压平定价：「每 L 版压平一次 O(N)，摊到每次 O(N/L)，不压平则读代价无界」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:70`）；读全版格写「要有界就要压平，压平版就是甲的丑版」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:71`）。即正推腿没有判「丙强仍中」，它判的是「丙强退化成一个差一点的甲」。
【读】发布路径今天对上一版是整版建索引再取：`FirstEntryOfEachIdentity::of_the_table(&self.units, |unit| unit.identity)`（`crates/singlefs-core/src/transaction.rs:4540`）。所以「链长乘在取值次数上」里的取值次数今天是 U，不是 d；要把它降到 d，要改发布路径的取值方式，这一改三个候选一样要做（正推腿的分步第 5 步写「推不出」，`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:143`）。链长乘数只是系数（L 对 h），量级的分界不在这里。

判：
- 对不压平的丙：够得着。
- 对丙强（定期压平）：「无界」够不着；剩下的是系数加均摊：取一键最坏 L 跳，L 取 √N 时是 √N（甲是 log N，这一格量级不同）；压平均摊 N/L，单次尖峰 O(N)（基不可变，可放后台，推的）。
- 对丙强（几何合并）：取一键至多 log N 段，量级与甲同；均摊写放大 log N 倍；整版遍历 O(N·log 段数)（整理与验证包才整版读）；删除要墓碑；尖峰同上。
- 「整条链被拽着」：压平后的新基不引用旧链，旧链只被仍拿着旧句柄的读者保活；产品里长期活着的只有 `current` 一版（`crates/singlefs-core/src/mounted_session.rs:132` 的 `pub current: PoolVersion,`），多版列表一次调用返回后即丢【推，没数过所有持有者】。
什么会推翻：若发布路径对上一版的取值次数降不到 d（即那一改做不成），丙强与甲同样买不到东西，这一格对三条平手；若核心层量出 L 取到使均摊项 N/L 与取值项 d·L 之和仍随 N 线性涨（例如 d 本身接近 N），丙强也中。

### 二·5 判决「乙、丙两腿一致打中」

第一轮判决那张表：「两腿一致打中」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md:40`、`:41`）。对乙：正推与攻方都指出多版读、8 处断言、冻结，这三条我在二·1 到二·3 里判过：对纯差量形态成立，对乙强不成立（冻结里的空闲图那一半对三条同样）。对丙：攻方打的是不压平的链，正推腿打的是「读代价无界」并同时给出压平的均摊，没有打「丙强」。所以「一致」只在没加强的两个形态上成立；判决那一行写成「打中，挂起」对没加强的形态够得着，对加强形态是没被攻过（零轮）。

## 三、第一轮的打中是不是同样打中甲；补完之后与甲还差什么

### 三·1 逐条

| 第一轮的打中 | 乙 | 丙 | 甲 | 证据与判 |
|---|---|---|---|---|
| 一次调用交回几版、逐版读全版 | 纯差量中，乙强不中 | 不中（沿链各版都在） | 不中（各版一个根） | 二·1；分辨臂，但只对没加强的乙 |
| 版本相等：8 处断言 | 浅句柄中，深拷快照不中 | O(N) 合出再比，不中 | 要按内容比：`TransactionOutput` 今天 `#[derive(Clone, Debug, PartialEq, Eq)]`（`crates/singlefs-core/src/transaction.rs:3374`），自写树的形状随插入历史变，照抄 `derive` 会比形状；攻方也写了这一条（`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:125`），所以同样打中甲的一半 | 二·2；不分辨臂 |
| 冻结要两份全态 | 记录与版本那一半不中，空闲图中 | 同乙 | 记录那一半不中，位图仍整份拷（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:164`） | 二·3；空闲图同样打中甲 |
| 链长乘在取值上 | 不适用 | 不压平的中，丙强剩系数 | 取值 h 层，不适用 | 二·4；只打丙 |
| `&mut` 走全版（`take_the_mapping_positions_over_the_location_hints`，`crates/singlefs-core/src/transaction.rs:3992`） | 读满 N 中；原地改 K 项，写不中 | 读满中 | 读满中；「照现写法直接换容器要把每个节点拷一遍」只对没改写法的甲 | 攻方自己写「读满这一半不分辨臂」：`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:68`；判决把它只列在甲的行里，同样打中另两条 |
| `units` 的次序与下标 | 中：键要另立 | 中：攻方把它列在丙下（`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:161`） | 中 | `ContainerTreeRootUnitPositions` 存 `units` 下标：`pub container_tree_root_unit_positions: ContainerTreeRootUnitPositions,`（`crates/singlefs-core/src/transaction.rs:3479`）。三条都要处理，判决只列在甲的行里 |
| 仓里没有持久共享容器 | 不适用（标准库 `BTreeMap`） | 不适用（标准库有序表与 `Vec`） | 中：只打甲 | 五 |
| 三条共有：分配器克隆、结算、整表拷出 | 乙强的索引就是分配器那份，天然合一（攻方原文：`research/prompts/m4-publish-without-whole-version-copy-design-r1-opus-output.md:57`） | 同甲（要把分配器的表也换） | 要把分配器的 `records` 也换成同一种结构 | 不分辨臂；这一格里乙强便宜 |

判：第一轮判决把「三处要解的点」全写在甲的行里、把「乙、丙打中」写在乙、丙的行里，但三处点里两处（`&mut` 走全版的读满、`units` 次序与下标）对乙、丙同样成立，第三处（容器）只打甲、乙丙没有。表的记法对甲偏宽：甲的缺点不比乙丙少，只是被记成「要解的点」而不是「打中」。这是记法问题，不是说甲该出局：甲在「版本是值」那一格上确实是三条里最干净的（见三·2 第一条）。

### 三·2 乙强补完之后与甲还差什么（全部【推】）

- 值语义由编译器保证，换成撤销完整性：今天「拷一份再改」靠 `clone()` 是深拷来保证上一版不被改：`let mut relocated_version = previous_version.clone();`（`crates/singlefs-core/src/transaction.rs:15822`）、`Arc::make_mut(version).writes = writes;`（`crates/singlefs-core/src/transaction.rs:2685`）、`Arc::make_mut(current),`（`crates/singlefs-core/src/mounted_session.rs:6332`）、`versions_published.extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));`（`crates/singlefs-core/src/mounted_session.rs:4644`）。乙强里 `clone()` 是句柄浅拷，这几处不改就是悄悄改到共享索引；编译器看不出，要靠人逐处换成「记录了再改」。甲里这几处照写、写时复制保证。对 `.claude/singlefs-ai-sop/rules/code-discipline.md@类型：让非法状态写不出来` 的读法是推的：那一节的字面说「验没验过、落没落盘做成类型」，没有一句说版本不可变。
- 预演：`current: &TransactionOutput,`（`crates/singlefs-core/src/mounted_session.rs:1850`）后接 `let mut rehearsed_version = PoolVersion::of_a_version_of_trees(current.clone());`（`crates/singlefs-core/src/mounted_session.rs:1866`），借用只给不可变引用、拷一份来演练。乙强要改成 `&mut` 加撤销，「现行版本不动」从借用检查变成撤销是否做全，由 8 处断言与新写的用例钉。
- 读过去的版本倒推 O(Σd)，产品用不到（二·1），验证包要。
- 并发：今天一把锁罩着会话，GPU 在锁内等：`locked: Mutex<SessionAndDevices<Device>>,`（`crates/singlefs-mount-runtime/src/lib.rs:396`）与「一批之内 GPU 仍在锁内等」（`crates/singlefs-mount-runtime/src/lib.rs:5`）；乙强的单索引与这把锁同构，今天不冲突。里程碑四的口径写「原来的项六（并行提交）与读写并发都归里程碑六」（`.claude/kb/milestone/04-fourth-txn.md:5`）：到那时读者要么拿锁要么要物化快照，甲不用。这一条是对将来的推，「按 `crates/` 今天的实现」下不成立。
- 乙强占的便宜：容器零行；与分配器合一，射程不必越出 `TransactionOutput`；`take_the_mapping_positions…` 的 `&mut` 在单索引上原地改 K 项，甲要对 K 项各做路径复制。
- 乙强没占到的便宜：想留 `&[AllocationRecord]` 切片就得让索引保持排好序的 `Vec`，而版本里那份按 (设备, 槽号) 升序、分配器里那份是插入序（`records: Vec<AllocationRecord>,`，`crates/singlefs-core/src/allocator.rs:2294`；`pub fn records(&self) -> &[AllocationRecord] {`，`crates/singlefs-core/src/allocator.rs:2743`）；中间插入要 O(N) 的搬移，d 条就是 d·N 次搬移；改 `BTreeMap` 就和甲一样丢切片、读者改动面（正推腿数的 112 处签名）不省。

### 三·3 丙强补完之后与甲还差什么（全部【推】）

- 取值系数：丙强（压平）每次取值最坏 L 跳（每跳一次有序表查找）对甲的 h 层；丙强（几何合并）至多 log N 段，与甲同量级。
- 均摊与尖峰：压平的均摊 N/L、单次 O(N)；几何合并的写放大 log N、最大一次合并 O(N)。甲没有尖峰，每次发布 O(d·h·F)（正推的式子，没量）。
- 版本相等与换回：丢掉新头就是换回，同甲；相等不同源时 O(N)，同甲。
- 按序号取：标准库的有序表没有按序号取（`BTreeMap` 没有这个接口）；`.units[i]` 的读者（正推腿数 16 处）与 `ContainerTreeRootUnitPositions` 的下标，在丙强与乙强里要改成按角色的键取，甲自写带条数的树才给得出第 i 条；但「另立角色键」三条都要做（三·1），所以这一条只在读者改动面上差一点。
什么会推翻三·2、三·3：核心层探针量出的 d 与 N 的分布（攻方腿的活）；若 d 经常接近 N，三条的差别都被抹平。

## 四、整表断言：三种并存读法各丢什么

先整节读了 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节。Q2.3 ② 那一行的判决写：「做得到（推的），两个前提：全量遍历保留为 checker 一侧的参照（`.claude/rules/fs-design.md` 第 34 行「审计与被审计用同一段代码」被禁止）；`assert!` 照原失败路径留着，不挪」（`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md:63`）；这一轮的第一轮判决把两个前提复述为「全量遍历保留为参照；`assert!` 照原失败路径留着、不挪」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md:32`），并记下正推的提法与它「相反」（`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md:52`）。那一行引的 `fs-design.md` 第 34 行今天在第 22 行：`- 被禁止的是两件事：需要答案的那一刻还没有答案；审计与被审计用同一段代码。**运行时与 checker 必须用不同的算法。**`（`.claude/rules/fs-design.md:22`）。

【读】今天的断言（`crates/singlefs-core/src/transaction.rs:16244`–`:16254`）：恒开的 `assert!`，两个合取：`nodes_whose_contents_changed(` 的结果（`crates/singlefs-core/src/allocation_record_tree.rs:368` 定义，读上一版整张记录表与这一版整张）是重写集的子集，且 `nodes_holding_records(`（`crates/singlefs-core/src/allocation_record_tree.rs:338`）算出的装着记录的节点集等于 `resolved.allocation_record_tree.node_set()`，失败消息「准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树：两边走的是同一串释放与取落点」（`crates/singlefs-core/src/transaction.rs:16253`）。它读的 `allocation_records` 是上面 `let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();`（`crates/singlefs-core/src/transaction.rs:16232`）整表拷出再排序的那份。
【读】它在 `publish_admitted` 里、准入之后，结果不被任何决策消费。照第一轮（2026-11-06）判深拷组的同一个字面（「不产出被决策消费的数」，`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md:72`），它也不落 `.claude/rules/fs-design.md` 里「运行时决策路径」那一行（`.claude/rules/fs-design.md:16`）。这一句是类比，推的。

三种读法（名字用内容命名）：
- 恒开并存：增量比对进发布路径，整表断言原样留着、恒开。
- 只留测试构建：发布路径走增量，整表断言只在测试构建里跑（正推腿的提法）。
- 恒开增量加测试参照：恒开的 `assert!` 留在原处，改喂增量算出的结果，整表算法只作为参照在测试构建里并排比（造池提速副本补丁的写法）：`allocator.start_noting_the_record_keys_changed();`（`crates.patch:7000`）、`comparison_with_the_reference_in_tests::compare_the_admitted_tree_with_grouping_every_record(`（`crates.patch:7034`）、`&& nodes_after_this_publish.holding_records`（`crates.patch:7048`）、消息改成「准入之前在分配器上演练走到的固定点，与真发时取完落点的记录说的是同一棵树：两边走的是同一串释放与取落点」（`crates.patch:7050`）。

| 读法 | 丢什么 | 合前提一（全量遍历留作参照） | 合前提二（`assert!` 照原失败路径留着，不挪） |
|---|---|---|---|
| 恒开并存 | 每次发布仍付 O(R)：断言读 `&allocation_records`，所以 `crates/singlefs-core/src/transaction.rs:16232` 那一份整表拷出与排序也要留着（它是断言的输入）；这一格的目标买不到 | 合：参照就是原断言 | 合 |
| 只留测试构建 | 前提二；发布路径在非测试构建里一旦固定点与真发说的不是同一棵树，不再当场失败，不一致的分配记录树可能落盘（推的）。另外「测试构建」今天没有现成的开关：`cfg(test)` 只在核心包自己的单测里为真，验证包把核心包当普通依赖编，它的集成测试跑不到（补丁自己的注释：「只在本包的单测里编」，`crates.patch:7590`）；`debug_assert!` 也不行，本仓 release 档照开：`debug-assertions = true`（`Cargo.toml:28`），E202 的 release 运行照跑、不省 | 合，条件是开关覆盖验证包的发布 | 不合（字面：断言移出了恒开路径） |
| 恒开增量加测试参照 | 整表参照只在核心包单测里并排比，验证包里绝大多数发布只过增量断言；增量的输入（`take_the_record_keys_changed`）若漏记某个改动的键，恒开的断言与它同源，不会红，只有整表参照会红 | 在核心包单测内合；在验证包的发布上不合 | 合位置与恒开；断言的条件换成了增量结果、消息措辞也变了（`:16253` 对 `crates.patch:7050`），算不算「照原失败路径」：判决第二节没写：推不出 |

仓里已有的「测试开关」做法：核心包的默认关特性，加验证包的 dev 依赖打开它：`lookup-work-counters = []`（`crates/singlefs-core/Cargo.toml:34`）与 `singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "test-only-publish-thresholds"] }`（`crates/singlefs-verification/Cargo.toml:55`）。「只留测试构建」与「恒开增量加测试参照」要让参照在验证包的发布上跑，照这个先例加一个特性，参照写在特性之下；这样两者都补上前提一。

判：
- 字面上合两个前提的只有「恒开并存」，代价是这一格不降。
- 「恒开增量加测试参照」加特性之后合前提一、合前提二的位置与恒开两面，「原失败路径」读成同一处 panic、release 下也开则合，读成同一个判断则不合：推不出，要主 agent 读定 Q2.3 ② 的意图。
- 「只留测试构建」不合前提二。
- 这一格不分辨臂：乙强里改到的键就是日志，甲里是 `ptr_eq` 跳过共享子树的差，丙强里是新段；整表参照在三条里都是「物化再逐项比」的测试侧算法。
什么会推翻：若 Q2.3 ② 前提二的意图只是「release 下不许把这道失败变没」，则第三种读法合；若意图是「断言的内容不许动」，只有第一种合，而「分配记录从分配器整表拷出」那一格在三条候选下都买不到。

## 五、容器：自己写，还是引外部包

### 五·1 仓里今天的依赖与规则（读）

- 核心包的正常依赖：`libc`、`singlefs-format`、`singlefs-replay-prefix`、`vstd`（`crates/singlefs-core/Cargo.toml` 的 `[dependencies]`）；池级 checker 只有 `singlefs-format`（`crates/singlefs-checker/Cargo.toml:9`）。`Cargo.lock` 里已有 `arrayvec`（`:30`）、`equivalent`（`:216`）、`smallvec`（`:911`），没有 `im`、`imbl`、`rpds`、`archery`、`immutable-chunkmap`（`grep -nE '^name = "(im|imbl|rpds|archery|immutable-chunkmap)"' Cargo.lock` 零命中）。
- 先例：核心包加 `libc` 时写「与 Cargo.lock 里已有的 libc 同一份，不新增包」（`crates/singlefs-core/Cargo.toml:13`），验证包同（`crates/singlefs-verification/Cargo.toml:24`）。我在 `.claude/rules`、`.claude/singlefs-ai-sop/rules`、`.claude/kb/decisions.md` 里搜「新增依赖、引入依赖、新依赖、第三方依赖、外部依赖、新增包」没找到依赖规则（只命中 `.claude/rules/test-script-discipline.md:65` 的「外部依赖的预编库」，不相关）。所以：有「非必要不新增包」的做法，没有写成规则的条款。
- 外部包进判法单元键：依赖行收窄「每个包一行（名、版本、来源、checksum）进键」（`crates/singlefs-unit-graph/src/lock_file.rs:3`）。引外部包与自写容器首次落地，都让闭包含核心发布路径的判法单元键变一次（自写是源码进键）；我没核哪些单元的闭包会到这个容器，这一次性成本两种做法共有，不分辨。区别在之后：外部包升版本改键（核心包对 `vstd` 钉精确版本、不许 cargo 自己升，`crates/singlefs-core/Cargo.toml:21`，同一做法要照搬），自写容器只在我们改它时改键。
- 变异表覆盖：变异表每行的「文件」是仓内路径（`crates/mutations.tsv:1`「变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名」），外部包源码不在仓里，变异行罩不到它，只能罩适配层。

### 五·2 自己写：行数估计与验法

行数都是【推】，没写过，依据是接口清单的分解：
- 二层分块持久表（正推腿的丁，只用标准库）：结构、`Clone`、`Debug`、`PartialEq` 约 40；`get`、`len`、按序号取约 40；`insert`、`remove` 含块分裂与并块约 90；`iter`、`range` 约 50；按序建表约 20；按块 `Arc::ptr_eq` 找变了的块约 30，合计约 270 非测试行；正推腿估 250（`research/prompts/m4-publish-without-whole-version-copy-design-r1-sonnet-output.md:76`），量级对得上。改一条付 O(B + N/B)，N=43861、B=256 时脊 172 个指针，仍随 N 线性、系数小。
- 完整的持久 B 树（甲的原形，带子树条数）：正推腿估 400 至 700（同一行）；我不另估，只给量级参照：下载的 `imbl-7.0.2` 里 `src/nodes/btree.rs` 1351 行、`src/ord/map.rs` 2994 行（功能全）；`immutable-chunkmap-2.1.5` 的 `src/avl.rs` 2163 行加 `src/chunk.rs` 826 行【量，`wc -l`】。这些是功能全的实现，不是最小子集的估计依据。
- 乙强、丙强不需要持久容器：标准库的 `BTreeMap`、`Vec`、`Arc` 够用，容器行数是零；它们要写的是日志与句柄（几十到一百多行，推的，没分解）。

验法（写完之后由实现员与主 agent 做，现在没东西可跑）：
- 随机操作对拍 `BTreeMap`：插入、删除、克隆之后再改（旧克隆必须不变，这条持久性检查是自写持久结构最容易错的一处）、范围、条数、按序号取、变了的键与暴力差集相同；
- 变异行进 `crates/mutations.tsv`：跳过路径复制原地改、分裂偏一格、`ptr_eq` 快路径不看内容、范围上界差一；
- 行数验收：写完用 `wc -l` 数非测试行，与上面的估计对表，超过估计一倍停下回报。
什么会推翻估计：若按序号取与「变了的块」比较要带第二份索引，行数上去；若实现员写出的并块路径需要借块，丁就不再是「没有重平衡」。

### 五·3 引包：候选、依赖、许可（读，2026-11-18 现取）

来源：`https://index.crates.io/<路径>` 的返回（正常依赖不含 `optional` 与 `dev`、`build`），`https://crates.io/api/v1/crates/<包名>` 的返回（许可、最近更新），下载 `.crate` 后在草稿目录解开 grep 源码。本机 `rustc 1.98.0`、`cargo 1.98.0`【量】，各候选的 `rust-version` 都不高于 1.85。

| 包与最新版 | 许可 | 正常依赖（非可选） | 有序表里读到的接口 | 缺的 |
|---|---|---|---|---|
| `imbl` 7.0.2（更新 2026-09-09） | `MPL-2.0+` | `archery ^1.2.1`、`equivalent ^1.0.2`、`imbl-sized-chunks ^0.2.0`（`MPL-2.0+`）、`rand_core ^0.9`、`rand_xoshiro ^0.7`、`wide ^0.7`；构建期 `version_check` | `OrdMap`：`ptr_eq`（`src/ord/map.rs:158`）、`len`、`get`、`range`（`:266`）、`insert`、`remove`、`diff`（文档写「会避开两边共用的节点」，`src/ord/map.rs:288`–`:291`）；默认指针 `ArcK`（`src/shared_ptr.rs:16`） | 按序号取（`pub fn nth` 一类，`src/ord` 零命中）；`unsafe` 在 `src/nodes/btree.rs` 有 4 处（`grep -c unsafe`） |
| `im` 15.1.0（更新 2022-04-29） | `MPL-2.0+` | `bitmaps ^2`、`rand_core ^0.6`、`rand_xoshiro ^0.6`、`sized-chunks ^0.6.4`、`typenum ^1.12`；构建期 `version_check` | 与 `imbl` 同源的 `OrdMap` | 最近更新是 2022 年，`rand_core` 停在 0.6 |
| `rpds` 1.2.1（更新 2026-05-15） | `MIT`（crates.io 接口返回的字段） | `archery ^1.2.2`、`smallvec ^1.15.1` | `RedBlackTreeMap`：`get`、`insert`、`remove`、`range`、`iter`、`ptr_eq`（只比根指针，`src/map/red_black_tree_map/mod.rs:902`）；默认指针 `RcK`（`:103`，不是 `Send`），要用 `RedBlackTreeMapSync`（`:111`） | 按序号取；找「变了的键」的差（`diff` 零命中） |
| `immutable-chunkmap` 2.1.5（更新 2026-10-28） | `Apache-2.0 OR MIT` | `arrayvec ^0.7`（关默认特性；锁里已有 0.7.8） | `Map`：`insert_cow`、`remove_cow`、`update_cow`、`range`、`get`、`len`；`diff` 是带回调的集合差（`src/map.rs:617`），不是「哪些键变了」 | 非测试源码里 `ptr_eq` 零调用；按序号取零命中 |

新增进 `Cargo.lock` 的包（量，对着现有的锁）：`imbl` 路径新增 `imbl`、`imbl-sized-chunks`、`archery`、`rand_core`、`rand_xoshiro`、`wide`（还有 `wide` 自己的依赖，我没查）；`rpds` 路径新增 `rpds`、`archery`；`immutable-chunkmap` 路径新增它一个（`arrayvec` 已在锁里）。

### 五·4 判

- 自己写还是引包：推不出。缺两样：用户口径（背景材料「交用户的」已知这一条）与依赖规则（上面搜过，没有）。
- 事实上会影响口径的几条：只有 `imbl` 带「跳过共用节点」的差，别的包要自己找变了的键；四个包都没有按序号取，`.units[i]` 与下标的读者无论哪条路都要改成按键取；`imbl`、`im` 的许可是文件级弱 copyleft 的 `MPL-2.0+`，仓根放的是 `LICENSE-APACHE` 与 `LICENSE-MIT` 两份，两者并存是否可接受没读到本仓的许可政策，也推不出；外部包罩不到变异表，只能黑盒对拍加适配层变异；`rpds` 默认 `Rc` 指针，不用 `Sync` 别名会在 `Mutex` 里编不过（推的：未编过）。
- 乙强、丙强不需要这个决定。

## 要主 agent 跑的命令

无。没写模型、没起探针、没有要经任务服务交的命令。判决要引的数里属于核心层的（一次发布改到的条数 d、各段占比）是攻方腿的探针，不在这一腿；本腿的数都是读出来或数出来的，下一节列出命令，主 agent 可直接复核。

## 核对用的只读命令与此刻输出（本腿跑的，原样）

```
$ grep -rnE 'assert_eq!\(pool\.output, current_before' crates | wc -l
8
$ grep -rnE '(versions_published|steps_published|\.publishes\b)' --include=*.rs singlefs-core/src singlefs-mount-runtime/src singlefs-background-gpu/src | wc -l
113
$ grep -rnE '(versions_published|steps_published|\.publishes\b)' --include=*.rs singlefs-verification/src singlefs-verification/tests singlefs-background-gpu-tests singlefs-mount-runtime/tests | wc -l
598
$ grep -nE '^name = "(im|imbl|rpds|archery|immutable-chunkmap)"' Cargo.lock | wc -l
0
$ rustc --version; cargo --version
rustc 1.98.0 (88d9e12ae 2026-08-18)
cargo 1.98.0 (797e8a9bc 2026-08-05)
```
第二条在 `crates/` 下跑，其余在仓根。外部包的索引、许可与源码在草稿目录（开头列了四个 `.crate` 的 sha256），原样的返回在 `index/`。

## 没做什么

- 没替主 agent 撤销或维持第一轮判决，只交复核结果；没往统一任务服务交任务；没复跑攻方腿的计数模型（`m4-publish-without-whole-version-copy-design-r1-opus-model/`），报告里不引它的读数。
- 没写模型、没量任何核心层的读数：乙强、丙强、甲的代价式子、行数估计、均摊与尖峰全是【推】。d 的分布（一次发布改到的条数）不在这一腿，量出来之前第三轮的比较不能引任何一格的代价。
- 没逐行读验证包与两个测试包里 598 行 `versions_published|steps_published|\.publishes` 的命中，二·1 的「产品侧不读过去版本的单元表」限在核心包 src、挂载运行库 src、产品 GPU 包 src 那 113 行。
- 没核冻结期间的读路径会不会同时要发布前与发布后的分配器（二·3），没核空闲图六种改动没有撤销步这句话（背景材料转述实现员）。
- 没读攻方腿本轮的报告、模型与草稿目录（在禁读清单里）；第一轮两腿的报告与第一轮判决读了。
- 没核引的外部包是否在本仓的工具链与 `vstd` 钉版下编得过：一个包也没加进本仓、没编过；`wide` 的依赖没查；`imbl` 的 `unsafe` 4 处只数了个数、没读。
- `.claude/singlefs-ai-sop/rules/code-discipline.md@类型：让非法状态写不出来` 与「版本不可变」的联系是我的读法，那一节的字面没有这句。
- 造池提速副本补丁（`crates.patch`）只读了断言与参照那几处，没读全；补丁在副本里、没进主工作区，里面引的行号是补丁文件自己的行号。
- 草稿目录 `~/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r2-sonnet/` 里只有索引与四个外部包的下载物（未当代码跑过，没有仓副本、没有编译目录）。
