# c673-clauses-r1 云端攻方（Opus）报告

日期 2026-10-04。被判对象：正文 `research/prompts/_c673-clauses-r1-body.md` 第三节 K1、K2、K3 的候选。代码引文一律指冻结副本 `/tmp/claude-1000/c673-clauses-r1/tree/` 里的路径（快照清单 `research/prompts/c673-clauses-r1-snapshot/crates-sha256.txt`，开工时与仓里 `crates/` 逐文件相同）；kb 行号是 kb 文件自己的行号。下面凡写「量过」的数都是我在冻结副本的拷贝上量的，**副本上的数**，主 agent 要在入库装置上重做才能引。

## 复跑

```
bash research/scripts/run-with-memory-cap.sh 8G bash research/prompts/c673-clauses-r1-opus-model/tools/rerun.sh /tmp/claude-1000/<新草稿目录>
```

它把冻结副本拷进草稿目录、按快照清单核 sha256、加仓根 `Cargo.toml` / `Cargo.lock` / `.cargo/config.toml` 与原型用例，跑原型（对 `out/probe.out`），再逐条跑变异表（对 `out/mutations-*.out` 的 `name=mutation_result` 行）。只编 `singlefs-harness` 的测试目标，不跑 checker 档；cargo 线程 8。原型那一半我自测过一遍：新编译目录从头跑，32 秒，`probe.out` 去掉耗时之后与入库那份逐行相同。全程约 7 分钟（变异 M4 一条 2 分 15 秒）。

| 文件 | sha256 |
|---|---|
| `probe/c673_opus_probe.rs` | 3757ff96fd69fe5a49c6e7ee37d8198f3f38ebcbe60e1f26699b486d5f792bbe |
| `mutations.tsv` | dff29d43e9a1b15eef41b01a7481d0c2dd75816d3b01e59a3fbf6059d8c98078 |
| `tools/mutate_and_run.py` | 6c1b726ef63b4820b694a5e0e17915637130e55ba59d13eb4baaf2944aaa84cb |
| `tools/rerun.sh` | 2bbd413096b61dfa1947f2e7ece8c6f689e60b05b776cbac4549d4f78e30c0da |
| `out/probe.out` | f64f0f590fa21cac04bde9bd604e019aa3fcd6a84040cbdf4fa4b7141635e3b0 |
| `out/mutations-M1.out` | 8b18f042506a34e5e0886862e770266e1a096ffcb6004bad050db4c08376803e |
| `out/mutations-M23.out` | f233146fb59d2355e800b5f37f32269bd5f2cc17f98c9f724c1a26bb33542f71 |
| `out/mutations-M4.out` | 1b7543a4d35bd5e1bc71b9553f5804f58c58b3838b797dbaeec443a6fba43838 |
| `out/mutations-M5.out` | e55dd6f48028ef744eee50223341b83b862484b132a4472f25f48c3db35b9bea |

`out/mutations-*.out` 是加 `k1_stream_state_count` 那一行之前的原型版本跑的：复跑时原型那几个目标会多出这一行，`name=mutation_result` 行不受影响。路径都相对 `research/prompts/c673-clauses-r1-opus-model/`。

## 各格判定一览

| 格 | 被攻的候选 | 判定 | 依据性质 |
|---|---|---|---|
| K1-1 | 字面 65537 | **打中（误红）**：合法历史上 extent / inode / `deleted_inodes` / 墓碑树四条全红，而池级 checker 在同一镜像上 0 违例 | 量过（副本） |
| K1-1 | 字面 65537 | **打中（放行坏实现）**：把头 ID 写死成 65537 的实现在字面读法下 0 红；仓里现有 5 个相关 harness 目标全绿 | 量过（副本） |
| K1-1 | 同表 inode 树 ID | 没打中误红：同样的历史 0 红；对写死 65537 的坏实现 4 红 | 量过（副本） |
| K1-2 | 同表读法写成「inode 树条目恰好一条」 | **打中（误红，按构造）**：环里 mkfs 根、暖机根的树表 0 条，字面「恰好一条」在它们上面红 | 推的 |
| K1-2 | 多于一条 | 池级 checker 今天哪一条都不判「同一种两条」，新判据要自己表态 | 读码 |
| K1-3 | 按条目判 | **打中**：同表读法要整张表，现有逐条目入口拿不到 inode 树那条 | 读码 |
| K1-4 | GPU 同判 | 代价小；顺带看到 GPU 的 I-9.16 只判 7 种、CPU 判 11 种 | 读码，推的 |
| K1 附 | 同一段历史中途的镜像 | 池级 checker I-7.4 红（越出攻击面，交主 agent） | 量过（副本） |
| K2 甲 | 认法里的「单元区起点 50176」 | **打中（空真）**：非默认环长 4 种几何 16 格认不出、全都不判；「起点 + 4」在 24 格里全成立 | 量过（副本） |
| K2 甲 | 认法与「新池新建文件」的定义 | **打中（判别子观测不到）**：同进程那条流与「mkfs 后重开、实例 1 写第一个文件」逐字节相同，kb 却把后者划成合法可偏离 | 读码 + kb 原文，推的 |
| K2 甲 | 照 layout / D3 ⑤ 逐字段认 | **打中**：树表 7 条（实现 11 条）认法下空真；照 D3 已定项 10 ⑤ 的表判 t2–t12 在每个合法新池上都红 | 量过（副本） + kb 原文 |
| K2 甲 | 只判 t1 | **打中（放行）**：只挪 t2–t12 的变异 M3 甲放行，丙那几条钉写用例红 | 量过（副本） |
| K2 甲 | 零长文件、多数据单元 | 零长照样写 1 个数据单元、落 50180，被认、判绿；第一个文件版本装不下就拒 | 量过（副本） |
| K2 乙 | O1 答落点 | 不比丙多抓到东西，要另写一份分配器模型 | 推的 |
| K2 丙 | 钉写用例 | E171 第 15 行与 M3 都红；只钉一种几何、一种长度 | 量过（副本）；漏的那几族是推的 |
| K3 | 补句的取值 | 各写路径的取值量过：建新 / 写内容取这次发布的参数，未改的记录、删除步、回退照抄 | 量过（副本） |
| K3 | 与字节表、实现对不上 | **打中三处**：layout 写「mkfs 参数」、纳秒写「写入时刻」而实现恒 0；实现注释说「记录里没有 ctime 字段」与 D8 字段表矛盾 | 读码 + kb 原文 |
| K3 | 哪条 oracle 判得到 | 三条都判不到；harness 用例钉住了四条写路径（覆盖写、顺序写、建 inode、unlink） | 量过（副本）：M4、M5 各被一条用例杀 |

## K1　树表条目的头 ID

### K1-1a　字面 65537 在合法历史上误红（量过，副本）

历史（现成用例 `writable_mount_of_a_formatted_pool.rs` 那一段的内存盘版，原型 `k1_recovered_first_file_version_issues_trees_from_the_ring_watermark`），每一步许可它的条款：

1. mkfs → 可写挂载（实例 1，只暖机）→ 再挂一次（实例 2）→ 实例 2 发第一个文件版本 txg 5，十二棵树 65536..65547，记录带水位 65548。
2. 崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前。下次可写挂载时 txg 5 的根槽与它点名的一个数据单元一时读不出（暂时故障）：择根落到 (2, 4)，那条记录施加前验点名单元验不过、不施加。用例文件头写的就是这一段：「择根落到实例 2 的暖机根 (2, 4)（树表 0 条、根上水位 65536），txg 5 那条记录施加前验点名单元验不过、不施加」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:1452`）。
3. 新实例的水位取根环与环里记录的 max（D8 已定项 8 ②「根环里全部根记录」并上记录新根段带的那个字段），= 65548。
4. 实例 3 在树表 0 条的一版上发第一个文件版本：「从 mkfs 之外的水位发第一个文件版本时，十二棵树的号从水位起连号发」（`.claude/kb/decisions/08-核心索引结构.md:243`），实现照做：「这一版的十二棵树从 `version_to_build_on` 的树 ID 水位起连号发」（`crates/singlefs-core/src/transaction.rs:5557`）。inode 树 = 65549。
5. 头 ID 照 D5 已定项 9 的定义写成 inode 树的树 ID：「inode 树条目写自己的树 ID（新池新建文件是 65537）」（`.claude/kb/decisions/05-快照-空间记账机制.md:227`）。

镜像上读出的树表（从镜像字节经只读挂载择根再读树表单元，不经发布路径交回的内存结构），故障解除之后再覆盖写到根环转过被丢的 (2, 5) 那一槽：

`research/prompts/c673-clauses-r1-opus-model/out/probe.out:17`：「name=k1_heads history=instance3_after_the_ring_turned_past_the_dropped_root root=(3,29) entries=11 kind/tree/head=1/65548/65549,2/65549/65549,3/65550/0,4/65551/0,6/65553/0,7/65554/0,8/65555/0,9/65556/65549,10/65557/65549,11/65558/0,12/65559/0 literal_red=4」

同一行末尾 checker 0 违例：`research/prompts/c673-clauses-r1-opus-model/out/probe.out:17`：「same_table_red=0 same_table_detail= checker_violated=[]」。被丢的根槽一直读不出（永久故障）那一形同样：`research/prompts/c673-clauses-r1-opus-model/out/probe.out:15`：「literal_red=4」。

⇒ 字面读法在一个池级 checker 65 条全绿的镜像上红四条。字面 65537 与 D8 已定项 8 自己的 ② 和 D5 已定项 9 的定义句冲突：D8 第 250 行的 65537 是新池新建文件那一格的值（D5 第 227 行在同一处括了「新池新建文件是 65537」），不是取值规则。

四句：① 分辨臂：分辨（字面 4 红、同表 0 红）。② 被判对象看得到判别它的东西：看得到，同一张树表里 inode 树那条就在。③ 满足的是 D8 第 250 行「头 ID 8 字节的取值」那一分句的字面（「inode 树条目写自己的树 ID（65537）」，`.claude/kb/decisions/08-核心索引结构.md:250`）。④ 改法：取「同表 inode 树的 ID」在这几格上不红（量过）。

要几个故障：纯崩溃造不出这一形（推的）。持久顺序是「单元与节点 → 屏障 → journal 记录 → 屏障 → 根槽 FUA」（`.claude/kb/layout/01-first-txn.md:84`），记录持久就说明它点名的单元都持久，恢复会施加它、第一个文件版本保住，树 ID 不会从 65548 再发；记录没持久，水位留在 65536，崩溃孤儿那几个号被重发（I-7.8 那一行说的就是这一形），inode 树仍是 65537。要「记录在、点名单元读不出」才走得到，这要一次读故障。这条流的层 0 全量我没跑：`research/prompts/c673-clauses-r1-opus-model/out/probe.out:10`：「segment_lengths=[2, 2, 1, 2, 2, 1, 2, 2, 12, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2] closed_form_state_count=16781352」，一段历史就超过约 10⁶，照定义不跑，交主 agent。

### K1-1b　字面读法放行「头 ID 写死 65537」的坏实现（量过，副本）

变异 M1：树表条目构造处把非 0 的头 ID 一律换成 `singlefs_format::TREE_IDENTIFIER_INODE`（65537）。同一段历史上：

`research/prompts/c673-clauses-r1-opus-model/out/mutations-M1.out:10`：「literal_red=0 literal_detail= same_table_red=4」

仓里与头 ID、从水位发树相关的 5 个 harness 目标全绿（`mutations-M1.out` 第 11–15 行；`new_pool_file_creation_publish` 8 过、`writable_mount_of_a_formatted_pool` 15 过、`mounted_read_by_offset` 12 过、`rollback_by_a_forward_publish` 1 过 22 跳过、`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over` 24 过）。

⇒ 字面读法不只误红，还把这个坏实现判成正确。同表读法抓得到它，**但只在树 ID 不是 65536.. 的镜像上**：常规历史里 inode 树恒是 65537（`out/probe.out` 第 1、4、7 行），两种读法逐字相同。仓里能产出「从高水位发树」镜像、又会跑池级 checker 的用例我没找到：`writable_mount_of_a_formatted_pool.rs` 那一条只读回内容、不跑 checker（推的，按 grep 看到的调用）。改成同表读法之后若不加一条在高水位镜像上跑 checker 的用例，M1 照样活着。

四句：① 分辨：字面 0 红 / 同表 4 红。② 看得到。③ 同 K1-1a。④ 同表读法在这一格红（量过）；只采纳读法、不加镜像，在仓里的装置上不红（量过：5 个目标全绿）。

### K1-2　0 条与多于 1 条

- **0 条（按构造，推的）**：mkfs 写的第 0 版树表 0 条（「树表单元第 0 版（0 条条目）」，`.claude/kb/layout/01-first-txn.md:57`），暖机根照抄它；池级 checker 判根环里每一个候选版本，带文件的镜像里通常还留着这些根。同表读法若写成「inode 树条目恰好一条」，在这些版本上恒红——原型里 `judge_same_table` 就是这么写的，我只拿它判了带树的版本，所以没量到。条款要写成「树表有条目时」。删空的一版（D8 已定项 16 第 5 条）树表照样 11 条、inode 树零条目根，头 ID 不变，不受影响。
- **多于 1 条（读码）**：核心层拒（「树表里同一种树有两条」，`crates/singlefs-core/src/recovery.rs:2277`），池级 checker 不判：「同一种有两条时取先出现的那条（核心层把它判成损坏，这里不另立判定）」（`crates/singlefs-checker/src/walk.rs:2985`）。I-9.16 ① 只看树 ID 升序，两条种类 2、树 ID 不同的条目照样过。所以「多于一条」今天没有一条不变量会红。新判据要么自己判「种类 2 至多一条」，要么写明交给另立的「同种至多一条」。射程要写一句：有克隆头（D6 已定项 1，每头一棵自己的 inode 树）之后一张表里会有几条种类 2，「恰好一条」变成假，到时候要换成「每条 extent / `deleted_inodes` / 墓碑树条目的头 ID ∈ 同表种类 2 条目的树 ID 集合，且每个头各有一条」。

### K1-3　按条目判还是按表判，放哪一条（读码）

同表读法要先知道这张表里 inode 树那条的树 ID，逐条目入口拿不到：`walk_tree_table_entry` 一次只收一条（`crates/singlefs-checker/src/walk.rs:1452`：「fn walk_tree_table_entry(&mut self, entry: &[u8]) {」），I-9.25 就住在那里。整张表的入口是 I-9.16 那一处：`crates/singlefs-checker/src/walk.rs:1102`：「judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);」。所以按表判，挨着 I-9.16 调。不并进 I-9.16：坏镜像要「只红它自己那一条」（`checker_known_bad_images.rs` 的写法），并进去以后「头 ID 写 0」与「两条种类互换」红同一个 ID，分不开；建议新立 I-9.27（I-9 今天编到 26）。I-9.25 那条「树表被几版共享时只在第一个走到它的那一版判」的毛病（`.claude/kb/invariants.md:311`）对头 ID 不咬：被几版共享的是同一份字节，判出来一样。

### K1-4　GPU 同判（读码，推的）

- 违例位容量 `POOL_CHECKER_VIOLATED_WORDS` = 4（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:39`：「pub const POOL_CHECKER_VIOLATED_WORDS: usize = 4;」），即 128 位，今天 65 条，加一条放得下；覆盖表与 `IMPLEMENTED_INVARIANTS` 同长的编译期断言（同文件第 72 行）逼着内核同时补。
- 写法：照 `w_judge_tree_table_entries_ordering` 那样整张表两遍循环——第一遍找种类 2 那条、数条数，第二遍逐条读偏移 116 的 8 字节比。每版每张表多一次 ≤ 81 条的循环，与走读比可以忽略（推的，没量）。
- 顺带看到的不一致（越出攻击面，推的，没造坏镜像）：GPU 的 I-9.16 ② 只循环 7 种（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1430`：「for (var k = 0u; k < 7u; k = k + 1u) {」，k ≥ 4 时种类 = k + 2，即 1、2、3、4、6、7、8），CPU 循环 11 种（`crates/singlefs-checker/src/walk.rs:73`：「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 11] = [」），kb 的 I-9.16 写 7 种。种类 9..12 的发号次序被打乱的镜像，CPU 红、GPU 不红；合法状态上两边都不红，逐位对拍看不出。新判据的 GPU 版要防同一种走样：内核里的种类表与 CPU 的那张由同一处常量生成，或者用坏镜像对拍。

### K1 附　同一段历史中途的镜像上 I-7.4 红（越出攻击面，量过，副本）

实例 3 挂载时读不出 (2, 5) 那条根，影子账隔离不了它引用的槽，第一个文件的数据单元又落回 50180；故障解除后 (2, 5) 读得出、按实例表是被抛弃的根，它指的那一份已被覆盖：

`research/prompts/c673-clauses-r1-opus-model/out/probe.out:13`：「被抛弃的根（实例 2、txg 5）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50180 的那一份与位置条目里的校验和对不上」

I-7.4 的条款把这种复用判成违例（「它们引用的块在离开根环之前同样不许重新分配、不许抹头」，`.claude/kb/invariants.md:58`），而 `writable_mount_of_a_formatted_pool.rs` 那条用例把这段历史当合法走、不跑 checker。根环转过那一槽之后（第 17 行）或那一槽一直读不出（第 15 行）checker 才全绿。这是实现、条款与用例三方之间的一处空白（挂载看不见的被抛弃根挡不住复用），不属于 C673；交主 agent 判要不要另立一笔。它不影响 K1-1a 的结论：K1-1a 引的是 checker 全绿的那两版。

## K2　新池新建文件的落点

### K2-1　甲的认法里「单元区起点 50176」让它在非默认几何上空真（量过，副本）

原型 `k2_new_pool_file_placement_matrix` 在 6 种几何（环长 128 / 256 / 768 / 1024 MiB 配 4 GiB 盘，768 / 1984 MiB 配 8 GiB 盘）× 5 种文件长度（0、1、3000、32634、32768）上跑「mkfs 同一个进程里那条流」。24 格发成，6 格被拒（32768 > 净荷 32634，`ContentExceedsDataUnit`）。发成的 24 格里：

- 根恒是 (1, 3)，数据单元恒 1 个、inode 记录恒 1 条，数据单元恒在「单元区起点 + 4」：例 `research/prompts/c673-clauses-r1-opus-model/out/probe.out:20`：「ring_mib=128 image_gib=4 length=3000 outcome=published root=(1,3) unit_area_start=9216 data_slot=9220 data_minus_start=4」，`research/prompts/c673-clauses-r1-opus-model/out/probe.out:45`：「ring_mib=1984 image_gib=8 length=3000 outcome=published root=(1,3) unit_area_start=128000 data_slot=128004 data_minus_start=4」。
- 甲的认法（实例 1、txg 3、起点 50176、一个数据单元、一条记录）只认得出默认环长那 8 格；非默认环长的 16 格 `recognized_by_jia=false`，一条都不判。
- 「起点 + 4」判法在 24 格里全绿（`judge_start_plus_4_red=false`）。

⇒ 认法里写死 50176 等于把新立的这条不变量限在一种几何上；条款若写成「数据单元落在单元区起点 + 4」（相对单元区起点），24 格都判、都不误红（量过）。D3 已定项 10 ④ 自己就说起点是按环长现算的（「默认 768 MiB 环是 50176」，`.claude/kb/decisions/03-空间分配.md:205`），字节表的 50180 是默认几何的值。

四句：① 分辨：甲（写死 50176）在 16 格空真，甲改相对写法在 24 格都判。② 看得到：系统配置偏移 417 存着单元区起点。③ 满足的是正文 K2 候选甲「认法」那一分句里的「系统配置里单元区起点 50176」。④ 改法「相对起点」在这 16 格判得到（量过）；用户定案的字面是「t1@50180」，改成相对写法要用户认一句。

### K2-2　甲认得出的那一类，kb 把其中一部分划成「可以偏离」（读码 + kb 原文，推的）

`NEW_POOL_FILE_CREATION_TXG` 的射程只管同一个进程里那一串：「`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（mkfs → 取号 → 暖机两次 → 新池新建文件，同一个进程里那一串）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:339`）。D3 已定项 8 把其余情形的落点划成运行时政策：「其余的写（覆盖写、之后的发布、重挂之后）上违反它的实现写出的镜像合法」（`.claude/kb/decisions/03-空间分配.md:171`）。而「mkfs 之后进程退出、重开可写挂载（实例 1）、在同一次挂载里写第一个文件」写出的字节与同一个进程那条流相同：「mkfs 之后的整条录制流与 mkfs 同一个进程里跑新池新建文件逐字节相同」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:4`）。

⇒ 单镜像上两者分不开（判别子观测不到）。今天的实现两者逐字节相同，所以今天不误红；但按 kb 的字面，一个只在「重挂之后」那一形把数据放到别处的实现写出的镜像是合法的，甲会判它红——甲比条款严。出路两条，都要用户认：把「新池新建文件」的格式级约束扩到「实例 1、txg 3 那一版」这个单镜像看得见的类（不论是不是同一个进程）；或者在条款里明写甲判的是这个更大的类。

四句：① 分辨：只分辨「甲」与「丙」（丙不判镜像，不受影响）。② 看不到：进程边界不进盘。③ 满足的是 D3 第 171 行「重挂之后」那一分句。④ 两个出路都在这一格上不红（推的）。

### K2-3　照 layout / D3 ⑤ 逐字段认、逐字段判（量过 + kb 原文）

- 树表条目数：layout 写 7（「⇒ 树表第 1 版 **7 条**」，`.claude/kb/layout/01-first-txn.md:49`；t12「树表单元第 1 版（7 条条目」，同文件第 78 行；D8「新池新建文件写出的 **7** 条树表条目照此表」，`.claude/kb/decisions/08-核心索引结构.md:300`），实现写 11（`research/prompts/c673-clauses-r1-opus-model/out/probe.out:1`：「name=k1_heads history=new_pool_file root=(1,3) entries=11」）。认法里带「树表 7 条」就永远认不出，整条不变量空真。layout 第 49 行同一句里还写着水位 65548（十二棵），与「7 条」自相矛盾。
- 提交内生块：D3 已定项 10 ⑤ 的表只有 7 行、树 ID 写 11..15、t8 树表落 50248（「| t2 | extent 树根兼叶（码 2，16 KiB） | 11 | 50240 | 0 |」，`.claude/kb/decisions/03-空间分配.md:211`），实现与 layout 第 67–78 行是 11 个单元、树 ID 65536.. 、树表落 50252（`research/prompts/c673-clauses-r1-opus-model/out/probe.out:48`：「name=k2_matrix_default_internal_minus_start value=Some([64, 66, 68, 69, 70, 71, 72, 73, 74, 75, 76])」，即 50240、50242、50244..50252）。照 D3 ⑤ 的表判 t2–t12，每个合法新池都红。
- 提交内生块的个数随几何变：环长 1024 MiB 时是 13 个（`research/prompts/c673-clauses-r1-opus-model/out/probe.out:35`：「internal_minus_start=[64, 66, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78]」），分配记录树多出节点（推的，没拆是哪两个）。t2–t12 若要按相对写法判，个数不能写死 11。

### K2-4　甲只判 t1：E171 第 15 行那一族抓得到，只挪提交内生块的放过（量过，副本）

- M2（E171 第 15 行那条，`crates/mutations.tsv:1040`：「实审 A2c 第 15 条：用户数据落点不从这张图的单元区起点起扫」）：甲红，丙那三个目标都红。`research/prompts/c673-clauses-r1-opus-model/out/mutations-M23.out:14`：「ring_mib=768 image_gib=4 length=3000 outcome=published root=(1,3) unit_area_start=50176 data_slot=50240」，同行「recognized_by_jia=true judge_50180_red=true」；`new_pool_file_creation_publish` 2 败、`new_pool_file_creation_write_regions` 2 败、`core_review_unit_area_start_and_publish_limits` 1 败（`mutations-M23.out` 第 33–35 行）。
- M3（开放段的起点多跳一段，t1 不动）：甲放行，丙红。`research/prompts/c673-clauses-r1-opus-model/out/mutations-M23.out:49`：「data_slot=50180 data_minus_start=4 data_units=1 records=1 internal_minus_start=[128, 130, 132, 133, 134, 135, 136, 137, 138, 139, 140] recognized_by_jia=true judge_50180_red=false」；同三个目标 2 / 2 / 4 败（第 68–70 行）。

D3 已定项 10 的射程只写了「七个提交内生块的落点整体重排」（`.claude/kb/decisions/03-空间分配.md:221`），全 D3 文件里「不合法」只出现在已定项 8 射程那一句（第 171 行，按 `grep -nF 不合法` 数，1 处）。所以 t2–t12 偏离算不算不合法，kb 没有一句话；甲只判 t1 与条款字面一致，但 M3 这一族今天只有丙抓。

四句：① 分辨：M3 上甲 0 红、丙红。② 看得到（落点在镜像上）。③ 满足的是正文 K2「t2–t12 要不要一起判」那一问，不是任何已定判据。④ 甲加判 t2–t12 在 M3 上会红，但前提是先把 D3 ⑤ 的表改成实现的 11 / 13 个单元（K2-3），否则先误红。

### K2-5　零长文件、多个数据单元、第一次发布写几个单元（量过，副本）

- 零长：照样写一个数据单元、落起点 + 4、认得出、判绿（`research/prompts/c673-clauses-r1-opus-model/out/probe.out:28`：「ring_mib=768 image_gib=4 length=0 outcome=published root=(1,3) unit_area_start=50176 data_slot=50180」）。
- 多单元：第一个文件版本装不下一个单元就拒（`research/prompts/c673-clauses-r1-opus-model/out/probe.out:32`：「outcome=refused error=publish:ContentExceedsDataUnit { bytes: 32768, capacity: 32634 }」）。所以 (1, 3) 那一版恒是一个数据单元一条记录，甲认法里的这两个条件在这一类上恒真、可以去掉（只要认 (1, 3) 且树表非空）。
- 别的历史上的第一个文件版本（不是新池新建文件，甲不认）：实例 2 只暖机过之后写的，t1 仍是 50180，提交内生块落到 [50247..]（`research/prompts/c673-clauses-r1-opus-model/out/probe.out:8`：「data_slot=50180 internal_slots=[50247, 50248, 50250, 50251, 50252, 50253, 50254, 50255, 50256, 50257, 50258]」）；K1-1a 那段历史的实例 3 落到 [50311..]（第 12 行）。layout 的 t2–t12 只在同一个进程那条流上成立。

### K2-6　乙：O1 答落点（推的）

O1 今天的观测不带落点（E171 第 15 行那一格的原话：「O1：`ObservedRoot` 不带落点，读回同，看不到」，`.claude/kb/experiments/171-三oracle合谋测试.md:86`）。乙要模型自己算 t1，就要在模型里再写一份 D3 已定项 8 / 10 的分配器，而按 D3 第 171 行，除新池新建文件之外的落点都不许拿来判，模型只能在 (1, 3) 那一版上比——等于在模型里钉一个常量，抓到的集合与丙相同（M2、M3 都是同进程那条流上的字节），多付一份分配器模型的维护。没量。

### K2-7　丙：钉写用例漏什么（M2、M3 量过；漏的那几族推的）

丙那三个目标钉的是一个几何（768 MiB 环、4 GiB 盘、3000 字节、固定 fsid）上的整份写清单，`core_review_unit_area_start_and_publish_limits` 另在 128 MiB 环上判「紧接环后发槽」。漏的（推的，没造变异）：只在别的文件长度、别的盘大小、别的环长（除 128 MiB）上走样的落点错误；它们写出的镜像按 K2-1 仍是新池新建文件。甲改成相对写法也只在 checker 真见过的镜像上判：层 0 与 harness 里这几种几何的新池镜像我没数到有几份（推的）。所以「丙」与「甲」的差别不在 E171 第 15 行那一族（两者都抓），在 M3 那一族（只有丙抓）与非默认几何那一族（两者今天都看不到，甲相对写法加上跑那几种几何的用例才看得到）。

## K3　D8 已定项 6 的写入时间

### K3-1　各写路径今天取什么（量过，副本）

原型 `k1_k3_standard_history_heads_and_write_times`：新池新建文件（参数 T0）→ 覆盖写（T0+60）→ 建 3 个 inode（T0+120）→ 顺序写 70000 字节（T0+180），每步之后从镜像经只读挂载读每条记录（号、出生代、改动计数、写入时间秒）：

- `research/prompts/c673-clauses-r1-opus-model/out/probe.out:5`：「step=new_inodes parameter=1788000120 records=[(1, 3, 4, 1788000060), (2, 5, 5, 1788000120), (3, 5, 5, 1788000120), (4, 5, 5, 1788000120)]」
- `research/prompts/c673-clauses-r1-opus-model/out/probe.out:6`：「step=sequential_write parameter=1788000180 records=[(1, 3, 6, 1788000180), (2, 5, 5, 1788000120)」

即：建出记录的那次发布、改写这个文件内容的那次发布（覆盖写、顺序写）取这次发布的参数；同一片叶里被连带 COW 的别的记录照抄自己的。删除步照抄（`unlink_mount_continuation_and_the_emptied_version.rs` 那条断言「写入时间不动」，`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:459`）；回退取 R_old 的树、照抄（读码：`mount.rs` 里没有写入时间参数）。三个秒字段写同一个值，三个纳秒字段写 0（`crates/singlefs-core/src/records.rs:83`：「writer.skip(12);」）。

### K3-2　补那一句会与三处已有文字对不上（读码 + kb 原文）

1. **「mkfs 参数」对「发布参数」**：layout 写「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md:267`），里程碑一写「写入时间是发布参数，不取系统时钟」（`.claude/kb/milestone/01-first-txn.md:103`），实现每次发布由调用方给（`FirstFile.write_time_seconds`、`publish_new_inodes` 的参数）。补句取「发布参数」就要同时改 layout 第 267 行，否则字节表与决策正文两说。
2. **纳秒**：layout 写纳秒三段也是「写入时刻」（「atime / mtime / ctime 纳秒 | 12 | 写入时刻」，`.claude/kb/layout/01-first-txn.md:269`），实现恒写 0（`crates/singlefs-core/src/records.rs:83`：「writer.skip(12);」），参数只有秒。补句要写清纳秒取 0（或参数带纳秒、实现跟着改），并改 layout 第 269 行。
3. **ctime 与 unlink**：实现给删除步照抄写入时间的理由里有一句与 D8 字段表矛盾：「写入时间不改：记录里没有 ctime 字段」（`crates/singlefs-core/src/transaction.rs:5836`），而字段表偏移 64 是「atime / mtime / ctime 秒，各 8」（`.claude/kb/decisions/08-核心索引结构.md:188`）。照抄这一做法本身站得住（同一句后半的理由是续做要确定，D8 已定项 16 第 7 条 ①b），但补句要把「删除步（意图与每一批）照抄、三个时间字段都不动」写成条款，不要留着那句错的理由；否则下一个读码的人会照「没有 ctime」去推。

补句放哪：取值规则（建新 / 改内容取这次发布的参数、别的照抄、纳秒 0）是盘上字节的取值，放定案那张字段表的偏移 64 / 96 两行；「删除步与回退照抄」放射程。这是推的建议，被攻过零轮。

### K3-3　哪条 oracle 判得到（量过，副本）

三条都判不到：O1 的模型与对拍里 `write_time` 零命中（`grep -c write_time` 在 `crates/singlefs-harness/src/model.rs` 与 `model_comparison.rs` 都是 0，与 E171 第 92 行那一格的原话一致），O2 单镜像不知道参数，O3 投影不进 inode 树。

但 harness 用例按路径钉住了，四条写路径各有一条改坏就红的用例：

| 写路径 | 改坏 | 谁红 | 依据 |
|---|---|---|---|
| 覆盖写 | 照抄上一版（`crates/mutations.tsv:292` 那条） | `overwrite_in_one_instance` 的 `cold_start_reads_the_second_content_and_the_pool_checker_stays_green` | 变异表点名（没重跑） |
| 顺序写 | 照抄上一版（M4） | `sequential_write_one_record_per_data_unit` 1 败，另 12 个目标全绿 | 量过：`mutations-M4.out` 第 15 行 |
| 建 inode | 参数减 1（M5） | `many_inodes_write_and_read` 1 败，另 10 个目标全绿 | 量过：`mutations-M5.out` 第 13 行 |
| unlink 各步 | 改成新值 | `unlink_mount_continuation_and_the_emptied_version.rs:459` 那条断言 | 读码，没跑变异 |

M4 是被「一个单元的顺序写与同内容的覆盖写逐字节相同」那条间接抓到的：`research/prompts/c673-clauses-r1-opus-model/out/mutations-M4.out:15`：「failed=['sequential_write_of_one_data_unit_writes_the_same_bytes_as_an_overwrite_of_the_same_content']」。只在多单元那一支照抄写入时间的变异，它就抓不到（推的，没造）。没找到钉住的：回退照抄 R_old 的写入时间、同叶连带 COW 的别的记录不许被盖上新时间（M5 的打印里 inode 1 那条保持 T0+60 是现状，没有用例断言它；推的）。

⇒ 补句之后「验不到什么」可以写：三条 oracle 判不到写入时间；由上表 harness 用例按路径钉，多单元顺序写、回退、同叶连带 COW 三格没有用例（记欠）。

## 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 头 ID 判据取「同一张树表里种类 2 那条条目的树 ID」，inode 树条目判等于自己的树 ID，其余八种判 0 | K1-1a、K1-1b | 量过：高水位历史上 0 红（`out/probe.out` 第 13–17 行），写死 65537 的坏实现上 4 红（`out/mutations-M1.out` 第 10 行） |
| 条款写「树表有条目时」，0 条的版本不判 | K1-2 0 条那一格 | 推的 |
| 种类 2 多于一条判红（或另立「同种至多一条」） | K1-2 多条那一格 | 推的 |
| 按表判，挨着 `judge_tree_table_entries_ordering` 调，新立 I-9.27 | K1-3 | 推的 |
| 加一条在高水位历史（K1-1a 那段，故障解除后转过环）上跑池级 checker 的用例与一份头 ID 写死的坏镜像 | K1-1b（不加就杀不掉 M1） | 量过：不加时 5 个目标全绿 |
| GPU 内核的种类表与 CPU 同源，或拿坏镜像对拍 | K1-4 | 推的 |
| 甲的落点判法写成「单元区起点 + 4」，认法去掉 50176、一个单元、一条记录三个条件，只认 (1, 3) 且树表非空 | K2-1、K2-5 | 量过：24 格全判、全绿；M2 上红 |
| 用户认一句「实例 1、txg 3 那一版」这个单镜像看得见的类都受 t1 约束（不论是不是同一个进程） | K2-2 | 推的 |
| D3 ⑤ 的表与 layout「7 条」先改成实现的 11 条 / 11 个单元（且写明个数随几何变），再谈判 t2–t12 | K2-3、K2-4 | 量过：默认 11 个、1024 MiB 环 13 个 |
| 写入时间补句：建新与改内容取这次发布的参数，其余照抄，纳秒 0；同步改 layout 第 267、269 行，删掉实现里「没有 ctime 字段」那句理由 | K3-1、K3-2 | 取值量过；改文字是推的 |

## 没打中的形状

- **同表读法的误红**：试了五段历史——新池新建文件、其后覆盖写 + 建 3 个 inode + 顺序写、实例 2 只暖机之后的第一个文件、K1-1a 那段（故障中、永久故障、转过环三种收尾）——带树的版本上同表读法 0 红（`out/probe.out` 第 1、4、7、9、13–17 行）。没试：删空的一版（读码看头 ID 不变）、克隆 / 快照（今天没有）、回退（回退那次发布照抄树表条目，读码；`rollback_by_a_forward_publish.rs` 里树 ID 不同就在任何写之前拒）。
- **纯崩溃造出高水位的第一个文件**：没造出来，也没跑：那条流的层 0 全量 16781352 个状态，超过一段历史约 10⁶ 的上限（`out/probe.out` 第 10 行）。按持久顺序推的结论是造不出（K1-1a 末段）。
- **甲把合法镜像认成新池而误红**：24 格同一个进程的流、实例 2 与实例 3 的第一个文件版本都没出现「被认出而 t1 ≠ 50180」；今天唯一的误红口子是 K2-2 那个看不见的进程边界（今天字节相同，0 误红）。几何只扫了 6 种、长度 5 种；没扫盘不等大、physical_block_size 4096、fsid 不同。
- **K3 写入时间的盲区**：M4、M5 都被 harness 用例杀了；没造多单元那一支、回退、同叶连带 COW 的变异。

## 这条腿自己的限度

- 全部数是副本上的：冻结副本拷贝 + 我加的一个 harness 测试目标与五条变异，主 agent 要在入库装置上重做。
- 原型里两种头 ID 判法、甲的认法与两种落点判法是我照正文的文字写的模型（`judge_literal`、`judge_same_table`、`recognized`），不是哪个候选实现的代码；认法里的单元区起点、数据单元个数取自发布交回值，不是从镜像读的（落点在镜像上的位置由同一次发布写出，推的一致）。
- 「仓里没有在高水位镜像上跑 checker 的用例」「没找到写入时间回退那一格的用例」是 grep 与读码，没把全部 harness 目标跑一遍。
- K1-1a 的历史要一次暂时的读故障（根槽 + 一份点名数据单元）；它算不算「合法历史」我照仓里现成用例的态度算，附带的 I-7.4 那一格说明这段历史在条款上本身还有空白。
- 用户动作：K1-1a 之后的动作我固定成覆盖写（转过根环）；放开它不影响头 ID（树 ID 只在第一个文件版本那次发，之后每种发布照抄条目，读码），没扫。

## 没做什么

- 没跑重型测试、checker 档、层 0（16781352 个状态那条流越过上限，交主 agent 定要不要跑）。
- 没改仓里任何 crates 文件；改动只在已删掉的草稿副本上。
- 没造 K1-4 GPU 七种对十一种的坏镜像、没造 K2-7 与 K3-3 里标「推的」那几种变异。
- 草稿目录 `/tmp/claude-1000/c673-clauses-r1-opus/` 里的仓副本、两个编译目录（4.2 GiB 与复跑自测那份）已删；留下的是日志、原型源码副本与复跑自测用的 `rerun-probe-only.sh`，没有编译目录与仓副本。

## 引文核对

```
python3 research/scripts/cite-check.py research/prompts/c673-clauses-r1-opus-output.md --root /tmp/claude-1000/c673-clauses-r1/tree --root . --background research/prompts/_c673-clauses-r1-background.md
  ✓ 核了 41 处引文，对上 41 处，没判 0 处（逐处列在上面）
```
