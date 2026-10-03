# m3-formal-r4 正推腿报告（Sonnet，2026-10-02）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 Y2 Y7 K7 O1 O2 O3 M2 M3 M4 P Q S Sd H1 H2 H3 H4 C191 C447 R1 R2 R3 -->

轮名 m3-formal-r4（第一轮），立场正推：从已定条款与 `crates/` 今天的实现推 T1–T6 每一格「应该是什么」。被判对象读冻结副本 `/tmp/claude-1000/m3-formal-r4/tree/`（清单与主仓 `research/prompts/m3-formal-r4-snapshot/crates-sha256.txt` 220 份核过 0 份不符），没读主仓 `crates/`、没读禁读清单。kb 引文写 kb 文件自己的行号，代码引文写副本里那份文件自己的行号，都是这一次 `grep -nF` 现查的。

记号：**[读]** 读 kb、代码；**[推]** 推的，没量；**[量]** 我这一次真跑的（命令与原样输出在第九节，脚本与产物在 `research/prompts/m3-formal-r4-sonnet-model/`）；**[译]** 在我写的玩具模型上量的，不是 Rust 上量的。

**一句话**：
- T1 一种读法与条款相容（fsync 之后「那一刻之前的全部操作」），候选句写在第一节；弱读法（只 f 的内容、或 f 加目录项）是「承诺更少」的另一条路，Y2 把定义权交给 RefFS，所以带数列出。
- T2 是岔路：「按前缀丢」（P）、「按发布边界为单位的前缀」（Q）、「按任意子集丢」（S）在一段短历史上给出不同集合；S 与 D16 已定项 12 的「根 + journal 任意前缀」不相容，P 与 Q 都相容、差在边界谁来告诉 RefFS，带代价数交用户。
- T3 O1 今天与实现共用的只有 `singlefs_format` 的常量，但那一条只写在一行注释里、没有门禁盯；O1 的崩溃判官今天从共用的录制流里取「盘上已持久的最新根槽」当下界，RefFS 不许这样。
- T4 比的是「恢复出的 POSIX 状态」是否落在 RefFS 允许集合里，接在 `crash_recovery_disagreement` 同一处调用点；C191 第一对「O3 对 RefFS」今天出不了事件（`crates/` 里没有 O3、没有 RefFS）。
- T5 逐字段的表在第五节：必须相同的只有 POSIX 状态（与 fsid、不可变系统配置）；实例代号、txg、根槽位置、卸载记号、树指针、分配记录是「模掉、改判关系」；实例表行、F、树 ID 水位是「单调关系」。
- T6 比 POSIX 状态（RefFS 判）加根的关系（记录核对器判），射程限在不带故障的崩溃状态；它是独立的一条（挂载透明），不是 M3 的第三读、也不是 I-8.4；一个今天就会漏的地方：`mount_and_publish_once_on_the_crash_image` 的那一次发布整份覆盖内容，发布之后再恢复只能验到新内容，验不到挂载有没有动过旧状态。

## 零、实现今天的样子（我读到的）

1. **用户面极窄** [读]。发布入口 `publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:243`：「pub fn publish_user_change<Device: BlockDevice>(」）只收三种改动：覆盖写、顺序写、建空 inode（同文件第 52 行起的 `UserChange`）。「第一版没有目录，活着的文件 nlink 恒 1」（`crates/singlefs-core/src/records.rs:32`），D17 已定项 5「第一版没有挂载：FUSE 是后来的一层薄适配，等目录与 POSIX 面出现之后再加」（`.claude/kb/decisions/17-实现分层与第三方管道.md:82`）。`crates/` 里没有 rename、mkdir、readdir，也没有发 unlink 意图的发布函数：`grep -rn 'unlink_batch_for_tests' crates` 只命中 `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:321` 一行注释。⇒ T2「rename、unlink 与数据写的相对次序」今天没有可对的实现，下面按「条款面先写、实现面还没有」处理。
2. **没有「写缓冲 + 稍后 fsync」** [读]。`grep -rn 'fn [a-z_]*fsync' crates` 只命中三个测试函数名；每次 `publish_*` 调用就是一次发布，返回即 fsync 返回（持久顺序见 `.claude/kb/decisions/16-发布语义.md:179`：「fsync 等系统配置轮换持久之后才返回」）。D16 已定项 10 的攒批（「攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次」，`.claude/kb/decisions/16-发布语义.md:233`）今天没有代码；今天一次 `publish_sequential_write` 把一个写请求切成 N 个一单元事务放进同一次发布（`crates/singlefs-core/src/transaction.rs:4800`：「pub fn publish_sequential_write<Device: BlockDevice>(」）。
3. **O1 今天不是 HashMap 理想文件系统** [读]。`crates/singlefs-harness/src/model.rs` 的 `IdealModel`（第 708 行）记的是「每一版提交的内容、每条根属于哪个实例、实例表的行、回退下界 F」（第 1 行文档注释），一版至多一个数据单元的文件（`ModelFileVersion` 第 180 行起，`pub content: Rc<[u8]>,` 在第 182 行），没有目录树、没有 inode 集合。kb 把它叫作「HashMap 理想文件系统」（`.claude/kb/decisions/13-验证路线.md:125`），代码已经是池级版本模型。「RefFS 由 O1 扩成」要扩的是这一个东西：从版本表扩到操作序列上的 POSIX 状态机。
4. **崩溃判官今天已有一份，且它的下界取自录制流** [读]。`crash_recovery_disagreement`（`crates/singlefs-harness/src/model.rs:2246`）收三样：模型提交过的全部版本、「盘上已持久的最新根槽」、实现读回的结局；四条判据第三条是「`if chosen < persisted {`」（第 2263 行）。「盘上已持久的最新根槽」由 `newest_persisted_root`（`crates/singlefs-harness/src/memory_pool.rs:812`：「pub fn newest_persisted_root(」）从录制流与持久集合里数出来，调用点在 `crates/singlefs-checker-tier/src/crash_injection.rs:834`（「let newest_persisted = newest_persisted_root(&writes, &persisted)」）。这是 T2、T3 要对着的现状。

## 一、T1　fsync 之后什么算持久

**一种读法与条款相容：那一刻之前已接受的全部操作。**

推导（[读] + [推]）：
1. 「fsync = 提前发布」（`.claude/kb/decisions/16-发布语义.md:156`：「fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号」）：fsync 不是单文件的刷盘，是触发一次发布；一次发布的东西是一条根，根指着整棵树的一个版本（`crates/singlefs-core/src/root_record.rs:52` 起的 `RootRecord`：树表指针、实例表指针、映射根都是整池的）。
2. 持久顺序恒为「单元 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」，fsync 等系统配置轮换持久之后才返回（`.claude/kb/decisions/16-发布语义.md:179`）。返回时这一代的根已在盘上，根覆盖的是整棵树 ⇒ 返回之前已接受的全部操作都在这一版里。
3. 事务层先串行（`.claude/kb/decisions/13-验证路线.md:53`：「是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A」，射程句反过来说串行下前缀就是全序上的前缀）：没有「f 的操作先于 g 的操作持久、却没有被 f 的 fsync 带上」这种中间形态。
4. 所以「只有 f 的内容与元数据」「f 加所在目录项」不是实现的机制，是更弱的承诺。

**条款候选句（T1-甲）**：
> fsync(f) 返回之后，调用开始之前已被接受的全部操作（不分文件、不分目录项与数据）崩溃之后必须仍在，即崩溃之后允许看到的状态至少是 fsync 调用开始那一刻的 POSIX 状态；fsync(f) 在途（已调用、未返回）时它不抬下界；fsync 报错返回时同样不抬下界，那次发布的数据可以出现也可以不出现（`.claude/kb/decisions/16-发布语义.md:336`：「fsync 报错之后，这次发布的数据**可以出现也可以不出现**」）。

**与攒批发布怎么对上**（问题第二句）：一次发布里含多个 fsync 之前与之后的操作，对下界没有影响，只对上界宽了：
- 下界只由「已返回的 fsync」抬：fsync 的调用就是一次发布的触发点（`.claude/kb/decisions/16-发布语义.md:137`：「fsync 密集负载下窗口由 fsync 自身触发」），一次攒批发布里 fsync 之前的操作一定在里面，fsync 之后才发出的操作可能在里面也可能不在；
- 上界是「崩溃时刻已发出的操作」，发布边界之内的次序 RefFS 不需要知道（见 T2 的 P 与 Q）；
- 新实例的暖机（「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回」，`.claude/kb/decisions/16-发布语义.md:196`）只会让 fsync 更晚返回，不会让「已返回」的集合变大，RefFS 不用建模它。

**弱读法的量**（Y2 把定义权交给 RefFS，所以不把弱读法当错）[译]：`research/prompts/m3-formal-r4-sonnet-model/t1_fsync_reading.py` 在历史 `create a; write a; create c; write c`、`fsync(a)` 之后：读法 c（全部）允许 1 个状态；读法 a（只 a 的内容与元数据）与读法 b（a 加目录项）各 3 个。坏恢复「只剩 a、c 全丢」：c 查出，a 与 b 放过。会放过它的实现长这样：将来为了降延迟把 fsync 改成「只发 f 那一棵子树」（D16 已定项 10 的攒批下很自然）。成本：读法 c 要求实现「fsync 带上整棵树」，是现在串行下免费、将来并行与局部发布要主动守的一条。

**什么现象会推翻**：找到实现里一处路径，fsync 返回之后恢复出的状态缺了 fsync 之前已接受的另一个文件的操作，而池级 checker、记录核对器都绿——那说明 D16 已定项 6 的「fsync = 提前发布」在代码里不成立，T1-甲会把它判红，这不是 T1-甲 错；反过来，在今天的实现上找到一个合法崩溃状态（不带故障）里恢复到比最近一次已返回发布更旧的版本，T1-甲 在今天的实现上就是假的（M4 一行要排除的就是「已确认的完整发布被恢复丢掉」（`.claude/kb/decisions/13-验证路线.md:34`））。


## 二、T2　崩溃之后允许看到的状态集合（岔路）

### 2.1　记号与三种丢法

RefFS 收一段操作历史（操作按 1..n 编号）与一个崩溃时刻，交一个 POSIX 状态集合。崩溃时刻只用 RefFS 自己的词：`hi` = 崩溃时刻已发出的操作数，`lo` = 最后一次已返回的 fsync 覆盖到的操作数（T1-甲：fsync 调用开始那一刻已接受的全部）；在途 fsync 与报错的 fsync 不抬 `lo`。`S_j` = 前 j 个操作施加出来的 POSIX 状态。

- **P　按前缀丢（单元级）**：{ S_j : lo ≤ j ≤ hi }，只丢一个后缀；
- **Q　按发布为单位的前缀**：{ S_j : j = lo，或 j 恰是某次发布的末尾 }，发布边界由 RefFS 之外的人给；
- **S　按任意子集丢**：{1..lo} 并上 (lo, hi] 里任一子集施加出来的状态。Sd 是 S 里再加一条「同路径的前一个操作也得在」。

### 2.2　量 [量][译]

脚本 `research/prompts/m3-formal-r4-sonnet-model/t2_allowed_sets.py`（只用标准库，迷你 POSIX 状态机，不是实现的代码），三段短历史：H1 一个文件的内容写；H2 目录操作与数据写（`create a; write a; create c; write a; rename a→b; write c`，目录操作今天实现里没有）；H3 一次写请求切三个一单元事务。表里是「已发出 hi 个操作、已返回的 fsync 覆盖到 lo」那一刻、按内容去重后的允许状态个数：

| 历史与时刻 | lo | hi | P | Q | S | Sd |
|---|---|---|---|---|---|---|
| H1 发出 4 个操作后 | 2 | 4 | 3 | 2 | 4 | 3 |
| H1 发出 5 个操作后 | 2 | 5 | 4 | 2 | 8 | 4 |
| H2 发出 5 个操作后 | 2 | 5 | 4 | 2 | 8 | 6 |
| H2 发出 6 个操作后 | 2 | 6 | 5 | 2 | 12 | 9 |
| H3 发出 3 个操作后 | 1 | 3 | 3 | 1 | 4 | 3 |
| H3 发出 4 个操作后 | 1 | 4 | 4 | 2 | 8 | 4 |

Q 的边界：H1 {2,4}、H2 {2,5}、H3 {1,4}（H3 的边界就是写请求两端）。三个坏恢复，在「全部操作已发出、最后一个 fsync 已返回」那一刻判（`查出` = 状态不在集合里）：

| 坏恢复 | P | Q | S | Sd |
|---|---|---|---|---|
| 坏1　丢了已返回 fsync 的最后一个操作 | 查出 | 查出 | 查出 | 查出 |
| 坏2　乱序：前面某个操作没落、后面的落了（H2 是「rename 落了、写 a 的数据没落」，恢复成 `b:x, c:z`；H1 `ADDB`；H3 单元 2 没落、单元 3 落了） | 查出 | 查出 | **放过** | 查出 |
| 坏3　切开一次整体边界（H1 落在第 3 个操作后；H2 第 3、4 个；H3 第 2、3 个） | **放过** | 查出 | **放过** | **放过** |

自检（`python3 t2_allowed_sets.py --selftest`，输出 `selftest ok`）：把下界 lo 强行取 0，坏1 由「查出」变「放过」，证明这一行的「查出」真靠下界；封闭式 |P| ≤ hi − lo + 1、|Q| ≤ |P| ≤ |S| 在全部时刻成立。

### 2.3　哪几种读法与条款相容 [读][推]

1. **S 与条款不相容。** D16 已定项 12：「根 + journal 任意前缀，replay 之后合法」（`.claude/kb/decisions/16-发布语义.md:262`）；持久顺序里 journal 记录先于根槽（`.claude/kb/decisions/16-发布语义.md:179`），恢复只施加严格大于所选根的连续前缀（`.claude/kb/decisions/23-journal的角色与格式.md:356`：「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」）。从「根 + 连续前缀」出发，落不到「后面的落了、前面的没落」那样的子集状态；串行提交下「前缀」就是全序上的前缀（`.claude/kb/decisions/13-验证路线.md:53`）。S 放过坏2，而坏2 正是 rename 落了数据没落（经典的零长度文件）；Sd 补一条同路径次序能挡住坏2，但 Sd 是为了补 S 的洞才要的规则，在前缀读法下它是多余的。
2. **「同一句话在 POSIX 面上的投影」是 Q 带边界，P 是它不要边界的放宽。** D16 已定项 12 说的前缀按记录截，再加第五条「施加的单位是一次发布」（`.claude/kb/decisions/23-journal的角色与格式.md:362`）与 D16 已定项 4「一次发布整体施加或整体不施加」（`.claude/kb/decisions/16-发布语义.md:111`）。投到 POSIX 面上恰是 Q（前缀 + 只停在发布边界）。P 是 Q 把「边界」放宽到每一个操作：Q ⊆ P。
3. **粒度**：一个写请求被切成 N 个一单元事务；「崩溃原子性的单位是一个单元、不是一次写请求」（`.claude/kb/decisions/23-journal的角色与格式.md:389`）。所以 RefFS 的「操作」要取到一个数据单元的写，不能取整个写请求，否则 P 会把「单元 1 落了、2 与 3 没落」这个将来合法的状态（攒批发布把一个请求拦腰切开时）判坏。今天的实现里一个写请求的 N 个事务在同一次发布里（`crates/singlefs-core/src/transaction.rs:4800`），这个状态今天走不到，所以今天两种取法在实现上给同样的结论，差别只在将来。
4. **目录操作与数据写的相对次序**：前缀读法下不用另写条款，全序自动给出。今天实现里没有 rename 与 unlink 的发布路径（零节第 1 条），这一句是条款面先写；要用它的那一天，RefFS 的操作面里加 rename/unlink 不需要新增丢法规则。

### 2.4　岔路 T2-F1：RefFS 的前缀粒度取 P 还是 Q（两条都与条款相容，在短历史上给不同集合）

| 候选 | 定义 | 代价（数） | 放过什么 |
|---|---|---|---|
| P | 单元级前缀，边界 = 每一个单元写 | 不要 RefFS 之外的输入；集合大：H3 末 4 个（Q 是 2 个） | 坏3：切开一次发布。那一类归记录核对器与 M2（M2 的名字就是「M2 前缀不切发布」，`.claude/kb/decisions/13-验证路线.md:32`：「M2 前缀不切发布」） |
| Q | 发布级前缀，边界由外面给 | 要么 RefFS 自己建攒批的触发规则（D16 已定项 10 的「攒够一批脏节点（或到时间）」，`.claude/kb/decisions/16-发布语义.md:233`，需要时间与脏量阈值），要么吃实现的发布日志（来源是实现，等于共用）。今天一次 `publish_*` 调用一次发布，边界恰是调用边界，所以今天免费；攒批一来就失效 | 无（坏1–3 全查出） |

我的推（没被攻过）：取 P，把「不切发布」留给 M2 与记录核对器——理由是 Q 的收益今天已有别的判官拿着，而 Q 的边界输入要么复制 D16 已定项 10 的阈值逻辑、要么向实现借，与 D13 已定项 3「不许共用」的方向相反。判据一（只有一种读法相容）不满足，所以按判据二写成岔路交用户，不替主 agent 采纳。

### 2.5　条款候选句（T2-乙，取 P 的写法）

> 崩溃之后允许看到的 POSIX 状态集合 = { S_j : lo ≤ j ≤ hi }。操作取到一个数据单元的写（一个写请求按 RefFS 自己的切分规则切成 ⌈长度 ÷ 单元净荷⌉ 个子操作，偏移升序）；hi 取崩溃时刻已发出的操作数（没有发出的操作一个都不许出现）；lo 取最后一次已返回的 fsync 的调用开始时刻已接受的操作数；崩溃之后恢复出的状态 s1 在下一次 fsync 返回之前已经算持久：之后再崩，允许集合的下界是 max(s1, 新抬的 lo)。比较按 POSIX 状态（内容、存在性），不按根的身份。

下界里 s1 那一句 [推]：M3 的单调推论（`.claude/kb/decisions/13-验证路线.md:38`：「M3 的单调推论只在串行提交、不带故障的崩溃状态里成立」）在不带故障的域里保证第二次崩溃恢复到的版本不早于第一次；今天的崩溃注入已经这样做了：第二次崩溃判官取「第一次崩溃前盘上最新根槽」与「挂载写表里最新根槽」的较大者（`crates/singlefs-checker-tier/src/crash_injection.rs:1610`：「let newest_persisted = newest_persisted_before_the_mount.max(newest_persisted_by_the_mount);」），RefFS 版用 s1 取代前一项。

**上界今天没有** [读]：`crash_recovery_disagreement` 收的是整段历史提交过的全部版本（`crates/singlefs-harness/src/model.rs:2277`：「let Some(committed) = committed_versions.get(&chosen) else {」），不按崩溃时刻截。流是整段录下来的，更晚的版本在崩溃状态里一个写也没落，上界在今天的流上不加判别力；RefFS 的崩溃时刻不看流，所以 hi 是 RefFS 自己的东西。

**什么现象会推翻**：(a) 在入库的 Rust 流上找到一个不带故障的崩溃状态，恢复出的 POSIX 状态既不是任何 S_j（lo ≤ j ≤ hi）又被池级 checker 与记录核对器判绿——P 就太紧；(b) 反过来找到恢复落在 S 集合里却不在 P 里、且实现的 journal 路径真能产生（例如由记录重建的根跳过中间一次发布）——说明「根 + 前缀」不等于 POSIX 前缀，T2-乙就是错的；(c) 脚本里 H3 的结论依赖「一个写请求的子操作全进一次发布」，攒批代码一来就要重量坏3。


## 三、T3　RefFS 与实现、与 O1 的独立性

### 3.1　O1 今天与实现共用什么 [读][量]

- **import 层**：`crates/singlefs-harness/src/model.rs` 从 `singlefs_format` 取 18 个常量（第 22–30 行的 `use singlefs_format::{…}`，数命令见第九节），其余只用 `std`。`grep -n 'singlefs_core\|singlefs_checker\|segments\|memory_pool' model.rs` 只命中第 3、5 两行，都是注释。第 5 行就是那条自我约束：「//! 只 `use singlefs_format`，不 `use singlefs_core` / `singlefs_checker`」（`crates/singlefs-harness/src/model.rs:5`）。
- **这一条没有门禁盯**：`grep -rn 'harness/src/model\|model\.rs' .claude/gate.d/*.sh` 零命中（第九节）。门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格只判四件：checker 与实现的依赖闭包求交（「按 crates/*/Cargo.toml 建内部依赖图，singlefs-checker 与 singlefs-core 各取传递闭包求交，减去 singlefs-format 必须为空」，`.claude/gate.d/checker-independence-and-sync.sh:19`）、checker 零引 core、format 无分支、harness 不依赖 checker 档（第 24 行）；它不看 harness 档里 `model.rs` 用了谁。
- **调用点层（更要紧的共用）**：O1 的崩溃判官 `crash_recovery_disagreement` 的第二个入参「盘上已持久的最新根槽」由 `newest_persisted_root`（`crates/singlefs-harness/src/memory_pool.rs:812`）从**与枚举同一份**录制流和持久集合里数出来：它内部调 `publishes_in`（`crates/singlefs-harness/src/memory_pool.rs:961`：「pub fn publishes_in(」），而崩溃点就是按这份流切段摆出来的（`crates/singlefs-checker-tier/src/crash_injection.rs:1670`：「fn draw_crash_points(」）。⇒ 今天的判官下界与崩溃点同源：录制器或切段把某一次发布的根槽归错位置，崩溃点与下界一起歪，判官发现不了。这正是 D13 已定项 3 防的东西，缩小了的一份：「最危险的落点是」「RefFS 与实现是否共享同一个崩溃语义的错误理解」（`.claude/kb/decisions/13-验证路线.md:65`，原句整行见 kb）。
- 第二次崩溃判官更进一步：允许版本表里加「挂载写出的每条根」，内容取第一次崩溃之后**实现**读回的内容（`crates/singlefs-checker-tier/src/crash_injection.rs:1568`：「versions_allowed.insert(model_root_key(instance, checkpoint_txg), content.clone());」）。这一格是实现自己的读回当期望值，Y7 的第二次恢复用它当基准之前要想清楚（第六、七节）。

### 3.2　「那段代码」具体指哪些 [读]

D13 已定项 3 的定案句是「RefFS 与实现不许共用崩溃点枚举的那段代码」（`.claude/kb/decisions/13-验证路线.md:65`），没有列文件。从「什么决定『有哪些崩溃状态、每个状态里哪些写持久了』」往回数，三层：

| 层 | 文件与入口 | 为什么算 |
|---|---|---|
| 录制 | `crates/singlefs-harness/src/lib.rs:179` 的 `push`（把同盘连着的屏障并成一道，D13 已定项 4 点名 `.claude/kb/decisions/13-验证路线.md:78`）、`RecordingBlockDevice` / `SharedStream` | 崩溃状态的原料；并屏障改变操作串，不改变状态集合 |
| 切段与摆状态 | `crates/singlefs-harness/src/segments.rs:159` 的 `split_into_segments` 与关段规则；`crates/singlefs-harness/src/memory_pool.rs:517` 起的 `writes_and_segments*`、`newest_persisted_root`、`some_publish_persisted_without_its_root`、`publishes_in`；`crates/singlefs-checker-tier/src/crash.rs:1531` 起的 `enumerate_layer0*` 与 `torn_image_of_in_place_overwrite`；`crates/singlefs-checker-tier/src/crash_injection.rs:1670` 的 `draw_crash_points` 与 `draw_second_crash_points` | 定义崩溃状态集合 |
| 步号归属 | `crates/singlefs-checker-tier/src/crash_injection.rs:704` 的 `StreamMarks::step_containing`（流下标 → 历史第几步） | 把流上的崩溃点翻成「历史里崩在第几个操作」 |

RefFS 的允许集合不许调这三层里任何一个函数，也不许吃它们算出来的「最新持久根槽」。**允许的接口只有一个：一对整数 (lo, hi)**，由 harness 的步号表给出（第三层那个翻译），RefFS 自己不碰流。D16 已定项 4 ① 另有一条「不与 RefFS 共享恢复代码」（`.claude/kb/decisions/16-发布语义.md:115`）：RefFS 没有恢复函数，这一条对它天然成立，但要写进门禁的禁引清单里：`recover`、`replay_journal`、`choose_root`、`scan_journal`。

### 3.3　RefFS 自己的崩溃时刻从哪来 [推][译]

从 RefFS 自己的操作时间线上数，不从流上数：对 n 个操作与若干 fsync 位置，时刻 = 「发出 k 个操作之后」(k = 0..n) + 每个 fsync 的「在途」与「已返回」。`t2_allowed_sets.py` 的 `moments()` 就是这个定义（H1 共 8 个时刻，H2 9 个，H3 6 个）。流上的崩溃点到时刻的翻译（hi = 持久的写里最晚属于第几个操作，加一；lo = 整个已关段里最后一个整次发布属于第几个操作）住在 harness 一侧，不进 RefFS。

两个可核的东西，各自给推翻现象：
1. **覆盖检查（RefFS 一侧自己的、与枚举独立的完整性）**：每一类 RefFS 时刻都要至少被一个流上的崩溃点命中；某一类时刻一个都没命中，就是枚举漏了一段时间窗。今天 `CrashInjectionTally` 有字段 `crash_points_by_operation_kind`（`crates/singlefs-checker-tier/src/crash_injection.rs:385`：「pub crash_points_by_operation_kind: BTreeMap<HistoryOperationKind, u64>,」）但按操作种类数，不按「第 k 个操作在途」数；要加的是一个按 k 数的计数器。
2. **「一段不跨两次发布」要有数** [推，没量]：翻译里用「第一个没持久的写所属的步」当 hi 的依据，是每次发布末尾的屏障按盘各一道，段在发布末尾关上（`.claude/kb/decisions/16-发布语义.md:179` 的持久顺序），所以一段里的写同属一个步。这一条没有计数撑着。要量：对一段历史的录制流，数「同一段里写分属 ≥ 2 个不同步」的段数，非 0 就要把 hi 改成「持久的写里最晚属于的步」。

### 3.4　条款候选句（T3-甲）与实现落点

> RefFS 的源码只许 `use std`（与按名字登记的格式常量，只许数据单元净荷这一类行为上限）；不许 `use` `singlefs_core`、`singlefs_checker`、`singlefs_checker_tier`，不许引 harness 的录制器与切段（`push`、`RecordingBlockDevice`、`SharedStream`、`split_into_segments`、`writes_and_segments*`、`publishes_in`、`newest_persisted_root`、`some_publish_persisted_without_its_root`），不许引 `recover`、`replay_journal`、`choose_root`、`scan_journal`。它与崩溃枚举之间的接口是整数对 (lo, hi) 与一个「状态」类型（内容、存在性），别无其它。

落点两种写法（成本不同，交主 agent 判）：
- **(a) 一个模块**：`crates/singlefs-harness/src/` 下新文件，由 O1 扩出；门禁在 checker-independence-and-sync 里加一格，只对这一个文件做源码禁引的 grep；必红样本是一份带 `use singlefs_core` 的假文件。成本小；独立性靠文本 grep，与 `model.rs` 今天同一档，只是多了门禁。
- **(b) 单独一个包**（如 `crates/singlefs-reffs`，依赖表空或只有 `singlefs-format`）：依赖图上的独立性，那一格的①可以照写「RefFS 与 core 传递闭包求交，减去 format 必须为空」。成本：该格文件头写着「五个 crate 路径写死，改名或搬家时判红、不跳过」（`.claude/gate.d/checker-independence-and-sync.sh:30`），加包要改这一格；`.claude/rules/verification.md` 的三个包名表要补一行。

**什么现象会推翻**：找到 RefFS 的允许集合要用到某个只有录制流才有的量才算得出（比如「这一发布的单元写与记录写的相对次序」）——那说明 (lo, hi) 这一对接口不够，T3-甲 的「别无其它」要重开。

## 四、T4　RefFS 怎么接到验证上

### 4.1　比什么、在哪条流、哪一档 [读]

- **比的对象**：实现恢复出的 POSIX 状态（`RecoveryOutcome::FileRead{content}` / `NoFile`，`crates/singlefs-core/src/recovery.rs:288` 起；经 `observed_read_back_after_a_crash`，`crates/singlefs-harness/src/model_comparison.rs:940`）是否落在 RefFS 的 { S_j : lo ≤ j ≤ hi } 里。**按内容比，不按根的身份比**：今天的判官按「根身份 → 提交过的版本」查表（`crates/singlefs-harness/src/model.rs:2277`），而根身份是实现的输出；按内容比，RefFS 就不需要知道实现给每一版起了什么号。代价：两个不同的版本内容相同时分不开（H1 里 S_j 内容相同的前缀会合并），分不开的那一部分归 M4 与 E165（恢复重放有界穷举与变异杀伤界）：「的最远一版判据在界内判」（`.claude/kb/decisions/13-验证路线.md:34`）。
- **流**：崩溃注入（`crates/singlefs-checker-tier/src/crash_injection.rs`）的三个截，入口都有现成的 `recover` 与判官调用点：
  1. 崩溃镜像本身：第 843 行 `recover`，第 855 行 `crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);`；
  2. 可写挂载 + 一次发布之后：第 1371 行 `checker_violations_of(&mount_run.pool_after)` 只跑池级 checker 与记录核对器，**没有 `recover`**（这一截 Y7 要补的位置，第七节）；
  3. 二次崩溃：第 1600 行 `recover`，第 1612 行 `crash_recovery_disagreement(`。
  RefFS 的判官与旧判官并排在这三个位置各加一次调用，旧判官不拆（它还判「走读不许失败」「根不许比盘上最新根槽旧」这类 RefFS 不管的东西）。
- **档**：判官要 `recover` 一份崩溃镜像，崩溃枚举与造镜像是「checker 档」（`.claude/rules/verification.md:16`：「归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档」），所以接到验证上的那一步只在 checker 档跑；RefFS 自己（允许集合的计算、必红自检、手写历史上的单测）不枚举崩溃状态，归 harness 档、随时跑。这与 O1 今天的分法一致（`model.rs` 在 harness 档，`crash_recovery_disagreement` 在 checker 档被调）。
- 层 0（`crates/singlefs-checker-tier/src/crash.rs:951` 的 `oracle_violation_for_versions`）的版本表由固定脚本手写；RefFS 可以把手写表换成「RefFS 对固定脚本算出的允许集合」，这一档作第二步，不在 T4 的最小落点里。

### 4.2　C191 第一对「O3 对 RefFS」的事件从这里怎么出 [读][推]

- **O3 与 RefFS 今天都零代码**：`grep -rn 'RefFs\|RefFS' crates` 零命中，`grep -rn '规约执行器\|\bO3\b' crates` 命中的两处都是别的意思（`crates/mutations.tsv` 一行变异描述里的「实四 O3」与 `checker_known_bad_images.rs` 的一行注释，见第九节）。C191 原文：「今天只对 O3（独立规约执行器） × RefFS 一对立了格」（`.claude/kb/checks-owed.md:168`）。⇒ 这一对的不一致事件今天**出不了**，D13 已定项 13 自己也写了「的事件从哪来没定，计数器没有代码」（`.claude/kb/decisions/13-验证路线.md:301`）。
- **条款候选句（T4-甲，事件契约，不依赖 O3 是否存在）**：
> 对同一个（历史、崩溃状态、崩溃时刻），RefFS 的「恢复状态在不在允许集合里」判定与对方判官的判定不同，即产生一行事件：「哪两方、哪个种子与崩溃点、判谁错、日期」；「一次不一致先在前提成立的域里重判，还在的才记」（`.claude/kb/decisions/13-验证路线.md:299`），三格：RefFS 错 / 对方错 / 条款欠一句，最后一格先记账、不计入「累计 ≥ 5 次」。
- **替身对（条款欠一句，交主 agent）**：在 O3 有代码之前，同一批崩溃状态上唯一的第二个判官是今天的 `crash_recovery_disagreement`。RefFS 与它的不一致是「O1 判官 × RefFS」，不是 C191 点名的三对里的任何一对（O3 × RefFS、crash refinement 的 spec × O1 的 model、M2 的 spec × 记录核对器）。我的推：把它登记成同一张表里的一对「替身对」，不计入 C191 的判红，只用来让事件管道有行可记（必红自检：喂一个「RefFS 放行、旧判官判红」的状态，管道必须记一行）；写不写进 C191，是用户与主 agent 的事。
- 同源性的另一面：O1 就是 RefFS 的前身，两份判官的不一致既可能是「RefFS 对」也可能是「同一个误解被扩出来了」，前一类没法区分后一类（`.claude/kb/decisions/13-验证路线.md:36`：「拿它当 spec 会让 refinement 恒真」是同一类担心）。⇒ 在替身对里「RefFS 错」的次数为 0 不构成独立性证据，只构成管道通。

**什么现象会推翻**：(a) 在今天的崩溃注入三个截里，RefFS 判官与旧判官对同一批状态判得逐个相同、而人为放一个 RefFS 一侧的小错（把 lo 多抬一格）后旧判官不变、RefFS 判官变红——那就是管道通了；这个现象**没出现**就说明接口没接对。(b) 找到一个崩溃状态，RefFS 放行、池级 checker 与记录核对器判红：不是推翻，是 RefFS 与另两个判官分担的那条线被你找到了，要记事件。


## 五、T5　挂载后再恢复：模掉什么（逐字段）

**设定**：v1 = 对崩溃镜像做只读恢复（`recover(&image, JournalPolicy::Consult)`，`crates/singlefs-core/src/recovery.rs:4177`：「pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {」，收的是只读的 `&dyn PoolReader`，不改盘）；v2 = 在同一份崩溃镜像上起可写挂载（取号、写行、暖机），再恢复。两种 v2：**v2a** 只挂载；**v2b** 挂载之后再发一次布（今天 `mount_and_publish_once_on_the_crash_image` 做的，第 1095 行）。设 s1 = v1 的 POSIX 状态。字段取自 `RootRecord`（`crates/singlefs-core/src/root_record.rs:52` 起的 10 个字段，第 53–70 行）、`RecoveryReport` 与系统配置槽。**[读]** 每一行都读过码，**[推]** 没有一行在入库代码上量过 v1、v2 的值——量那一遍是攻方腿的活（`重点 T5`），我给「会让这一行翻面的观测」。

| # | 字段 | v1 与 v2 的关系 | 判 | 依据（指到下面的编号） |
|---|---|---|---|---|
| 1 | POSIX 状态（内容、存在性、inode 记录里挂载不碰的字段） | v2a = s1；v2b = RefFS 把 s1 与那次发布合成的状态 | **必须相同**（v2b 是「相同于预测值」） | A1 |
| 2 | 池身份（`filesystem_identifier`）与系统配置的不可变部分（区域归属、尺寸） | 相等 | **必须相同** | A2 |
| 3 | 实例代号 `instance` | v2 > v1，且 = 盘上系统配置与根环里见过的最大实例代号 + 1 | 模掉，改判关系 | B1 |
| 4 | `checkpoint_txg` | v2a ≥ v1 + 2；v2b ≥ v1 + 3（两块盘及以上的池） | 模掉，改判关系 | B2 |
| 5 | 根槽位置 | 是 txg 的函数（区域 = txg mod 3，槽 = (txg ÷ 3) mod S），不是独立字段 | 不另判（跟 4） | B3 |
| 6 | 卸载记号 `unmount_marker` | v1 ∈ {卸载写的, 不是}；v2 恒「不是卸载那一串写下的」 | 模掉，改判 v2 的值 | B4 |
| 7 | `tree_table`、`mapping_root`、`allocation_record_tree_root` 三个指针 | 一般不同（暖机的空发布重写这几棵树） | 模掉；它们指着的内容归池级 checker 逐镜像判 | B5 |
| 8 | 分配记录（已分配 / 已释放 + 释放代） | 一般不同 | 模掉，不比 | B5 |
| 9 | `instance_table` 指针与它的行 | v2 的行集 ⊇ v1 的行集；多出的行正好是写行那次发布按 `instance_rows_to_write` 列的 | **单调关系**，可逐行判 | C1 |
| 10 | 回退下界 `rollback_floor` F | v2 ≥ v1（F 只升不降；准入不够时挂载还会推抬 F） | **单调关系** | C2 |
| 11 | 树 ID 水位 `tree_identifier_watermark` | v2 ≥ v1 | **单调关系** | C3 |
| 12 | journal 记录计数器（jsn 的计数器） | v2 的新记录从「环里最大 + 1」接着走 | 单调关系 | C4 |
| 13 | 恢复报告里的 journal 扫描：`prefix_applied`、`above_water` | v2：两者为 0（v2 的根比环里全部记录新） | 模掉，改判 v2 的值 | B6 |
| 14 | 系统配置槽的代号、journal tail | 取号写会改（实例号、世代号、tail 取见证值） | 模掉，归池级 checker 逐镜像判 | B7 |

### 依据

**A1**　v2a 必须等于 s1：可写挂载做的是「取号 → 写行发布 → 暖机」（`crates/singlefs-core/src/mount.rs:3864` 的 `establish_instance` 文档注释：「取号 → 写行发布 → 暖机到本实例的根覆盖每块盘」），这几次发布都不带用户改动：写行那次的发布种类是「写行」（重写实例表、分配记录树、记账树、映射树、树表，不写数据、不写 inode 树与 extent 树，`crates/singlefs-harness/src/model.rs:98` 的 `ModelPublishKind` 与第 142 行的 `rewritten_roles`）；暖机是空发布。⇒ 用户可见的两棵树（inode、extent）与数据单元不动。D13 已定项 1 的 M2 对「结果」的写法是同一个模式：「是根的投影加盘，不是整条根」（`.claude/kb/decisions/13-验证路线.md:32`），其余字段「等于所选根，卸载记号除外」同一行。v2b：今天那次发布是整份覆盖（`crates/singlefs-core/src/transaction.rs:4738`：「同一个实例里接在上一次发布之后再发布一版同一个文件」，内容装在一个数据单元里），发布之后 v2b 的内容 = 新内容，与 s1 无关；所以 v2b 只能对 RefFS 预测值比（见第六节）。
**A2**　写入口按挂载认下的参数建（`crates/singlefs-core/src/mounted_session.rs:11`：「写入口只按那份参数建」），池身份与区域归属、尺寸不在任何写里被改。判「相等」。

**B1**　取号 = 盘上最大实例代号 + 1：「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`crates/singlefs-core/src/transaction.rs:710`）；条款：「第一次可写挂载按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 取 max(系统配置, 根环) + 1 = 1」（`.claude/kb/decisions/23-journal的角色与格式.md:461`）。实例代号是时间线标识，两次恢复本来就取不同的代号；比大小，不比相等。
**B2**　新实例的第一次发布的 txg = 根环与环里全部自证记录的 txg 最大值 + 1：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1165`；条款 `.claude/kb/decisions/23-journal的角色与格式.md:370`：「第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」）；暖机再推「根覆盖每块盘」所需的次数，两块盘至少一次（`crates/singlefs-core/src/mount.rs:3105`：「fn warm_up_publish_txgs(」，循环条件「还有盘没覆盖」；条款 `.claude/kb/decisions/16-发布语义.md:196`）。⇒ v2a 的 txg ≥ v1 的有效根 txg + 1（写行）+ 1（暖机）。
**B3**　`target_for_publish`（`crates/singlefs-core/src/root_ring.rs:109`）把 txg 映射成「区域 = txg mod 3、槽 = (txg ÷ 3) mod S」；条款「区域 = txg mod R」（`.claude/kb/decisions/16-发布语义.md:156`）。不是独立字段，判了 txg 就判了位置。
**B4**　「flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退，以及由记录施加出来的那一版。」（`crates/singlefs-core/src/root_record.rs:26`）。v2 的最新根是写行、暖机或发布写的，一律是 0；v1 选到的根可以是卸载那一串写的（`crates/singlefs-core/src/root_record.rs:24`）或 0。条款面前例：M2 已经把卸载记号单列：「卸载记号除外」（`.claude/kb/decisions/13-验证路线.md:32`：「其余字段等于所选根，卸载记号除外」）。
**B5**　这三个指针与分配记录在 M2 里本来就不是被比的东西：「拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行」（`.claude/kb/decisions/13-验证路线.md:32`）。它们的正确性由池级 checker 对 v2 这份镜像自己判（I-3 一族、I-9 一族），跨 v1/v2 比没有意义：两份镜像里同一棵树的根必然落在不同的单元上。
**B6**　v2 的根是新实例的根，环里全部旧记录的 (实例代号, txg) 水位都不高于它（水位按实例代号为主比：`.claude/kb/decisions/23-journal的角色与格式.md:369` 注 2），所以 `above_water` 与 `prefix_applied` 都落在 0。[推，没量]
**B7**　取号那一写改系统配置里的实例号与世代号，tail 改写成取号那一刻读到的见证值（`.claude/kb/decisions/23-journal的角色与格式.md:391`：「取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0」）。归逐镜像判，不跨镜像比。

**C1**　写行规则：`let first_row_instance = previous_row.instance.0.max(1);`（`crates/singlefs-core/src/mount.rs:3141`）：给 [max(上一个实例, 1), 要取的号) 里每个实例各写一行；上一个实例那一行的内容是 (实例, 有效根 txg, 恢复施加的最大事务号)：`selected_root_txg: effective_root.checkpoint_txg,`（`crates/singlefs-core/src/mount.rs:4708`）、`applied_transaction_high_water: journal.maximum_applied_transaction,`（`crates/singlefs-core/src/mount.rs:4709`），中间实例 (k, 0, 0)，实例 0 不写行。⇒ v2 的行集 = v1 的行集 ∪ 这几行，v1 的行逐行不动。**这条比「实例表的行允许不同」强：v1 的恢复报告（`effective_root` 与 `journal.maximum_applied_transaction`，`crates/singlefs-core/src/recovery.rs:322`、`:314`）能直接算出 v2 里多出来的那一行**，两次恢复之间因此有一条可核的等式，不用比整张表。行格式归 D18 已定项 11，不在本轮条款清单里，我没读。
**C2**　写行与暖机的根带「恢复后生效的 F」：`rollback_floor: start.effective_floor,`（`crates/singlefs-core/src/mount.rs:2950`）；生效值「根上带的与系统配置里读得出的取最大」且「F 的生效值只增不减」（`crates/singlefs-core/src/mount.rs:129`）；用户发布照抄上一版（`crates/singlefs-core/src/transaction.rs:4729`：「// F 照抄建在上面的那一版（D16（发布语义） 已定项 1：F 只升不降）」）。v1 的根上带的 F ≤ 生效值 ≤ v2 的 F。
**C3**　`tree_identifier_watermark: version_to_build_on` 取「那一版的」与「本次发出的最高树 ID + 1」的较大者（`crates/singlefs-core/src/transaction.rs:4726`），挂载一侧再与环里全部根与记录的水位取最大（`crates/singlefs-core/src/mount.rs:1184` 的 `tree_identifier_watermark_of_the_ring`）。只升。
**C4**　计数器全池接着走：「计数器全池接着走：新实例从前缀末 + 1 接着写、不归零」（`.claude/kb/decisions/23-journal的角色与格式.md:370`）。

### 两处我没读透的、交攻方腿去量的

1. **行里的 W 取的是恢复「施加的」最大事务号，不是有效根覆盖的最大事务号** [读]：`applied_transaction_high_water: journal.maximum_applied_transaction,`。有效根已经覆盖了事务号 x、而这次恢复一条记录也没施加时，行里写 0。条款与实例表读者怎么用 W，我读到的范围里没有写「取覆盖的」还是「取施加的」。观测：崩溃状态在「根槽已持久、记录一条都不用施加」时，对比 v2 的行 W 与 v1 有效根覆盖的最大事务号；不相等就是一个「模掉」要不要允许的问题，不能照 C1 的等式直接判。
2. **「第一次恢复」与「挂载读阶段」在不带故障的域里取同一个根** [推]：C1 的等式靠的是两次读阶段选同一条有效根。只有在系统配置没见证到的撕裂、或读错重读（`crates/singlefs-core/src/mount.rs:4645` 读阶段那一段的注释：「系统配置见证过比所选那一版新的发布就重读一遍」）时才分叉，那已经出了不带故障的域（T6 射程）。

**什么现象会推翻这张表**：在入库代码的拷贝上对一批崩溃状态取 v1、v2（经 `research/scripts/run-with-memory-cap.sh`，harness 档的测试目标）：(1) 行 1、2 任何一个状态上不相等；(2) 行 3 出现 v2 ≤ v1；(3) 行 4 v2a − v1 < 2；(4) 行 6 的 v2 出现「卸载那一串写下的」；(5) 行 9 的 v2 行集不包含 v1 行集；(6) 行 10、11 出现 v2 < v1。每一条都会让对应那一行从「模 / 单调」改成别的。


## 六、T6　挂载后再恢复：比的对象与射程

### 6.1　比什么：两样都比，各归各的判官

1. **POSIX 状态**（RefFS 判）：v2a 必须等于 s1（T5 行 1）；v2b 必须等于 RefFS 把 s1 与那次发布合成的状态。s1 本身先要在第一截里被 RefFS 判进允许集合（T2-乙），之后 RefFS 从 s1 起开一条新时间线：「已确认」的东西是 s1，不是第一次恢复之前的任何别的状态。
2. **根的关系**（记录核对器判，不是 RefFS 判）：T5 表里「单调关系」「模掉改判」那几行（行 3、4、6、9、10、11、13）。理由：记录核对器的入参已经是「崩溃态镜像 + 记录流 + 实现恢复后的镜像」三份（D16 已定项 4 ②：「**入参拆成三份**：崩溃态镜像（择根与前缀判定看它）、记录流（崩溃态镜像环里的 journal 记录）、实现恢复后的镜像（比对的对象）」，`.claude/kb/decisions/16-发布语义.md:116`），今天崩溃注入第二截已经这样调它（`crates/singlefs-checker-tier/src/crash_injection.rs:938`：「.record_check_after_the_writable_mount(&image, &mount_run.pool_after);」）；根的关系是两张镜像之间的关系，POSIX 状态机不该背。
   不比的：行 7、8、14（指针、分配记录、系统配置槽），它们由池级 checker 对 v2 这一份镜像自己判。
3. **不是「只比根」**：根的相等在这里不成立（T5 行 3、4），只比根会把整个检查退化成「v2 的根比 v1 新」，POSIX 状态错了照样绿；也不是「只比 POSIX 状态」：只比状态看不见实例表行、F、水位这几个单调字段退回去，而那几个字段退回去之后 POSIX 状态可以暂时正确（F 退回去要等下一次回收才出事）。

### 6.2　射程

- **崩溃状态**：D13 已定项 4 的枚举域（不带故障的崩溃状态，含「新旧都读不出」的原地覆写第三态）里，**可写挂载做成了**的那些。挂载被拒的（今天分两类：单元区墙，只计数；别的理由，判红，`crates/singlefs-checker-tier/src/crash_injection.rs:1334` 起的 `judge_the_writable_mount_on_the_crash_image`）不进 Y7——没有 v2。
- **带故障的状态**（介质损坏、丢盘、读错）**不进这一条**：M3 的单调推论就只在不带故障的域里成立（`.claude/kb/decisions/13-验证路线.md:38`：「M3 的单调推论只在串行提交、不带故障的崩溃状态里成立」）；带故障时可写挂载可以拒绝（C554 乙、C331 取甲，`.claude/kb/decisions/23-journal的角色与格式.md:391` 一带），v1 与 v2 取同一根的前提（T5 末「第二处」）不成立。带故障域里 Y7 要不要有、取哪一个弱形态，是另一题，我没推。
- **第二次崩溃镜像要不要也做 Y7**（岔路 T6-F1，量过代价）：R1 只做第一次崩溃镜像；R2 加上二次崩溃镜像。`WritableMountPhase::ALL` 有三个相位（`crates/singlefs-checker-tier/src/crash_injection.rs:263`：「pub const ALL: [WritableMountPhase; 3] = [」），`draw_second_crash_points` 每个相位抽一个点，所以 R2 每个第一次崩溃状态多 ≤ 3 次「挂载 + 恢复 + 判」，R1 是每状态 1 次（v2a）或 2 次（v2a 加 v2b）。R2 的好处：挂载在「被第二次崩溃撕过的镜像」上也透明；坏处：挂载次数按 (1+3)× 算。我没有 R2 的价值证据，只给代价；没被攻过。

### 6.3　与 M3 不动点、I-8.4 的关系：独立的第四条，不是第三读

| 性质 | 对象 | 输入 | 出处 |
|---|---|---|---|
| I-8.4 重放幂等 | 一条 journal 记录 | 同一份镜像，记录施加两次对一次 | `.claude/kb/invariants.md` 那一行：「任一 journal 记录被重放两次与重放一次，产生的读出内容相同」（`.claude/kb/invariants.md:93`） |
| M3 不动点 | `replay_journal` 这个函数 | 把重建出的根当所选根、**同一张记录表** | 「把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变」（`.claude/kb/decisions/13-验证路线.md:33`） |
| Y7 挂载透明 | 「可写挂载」这个盘上状态变换 | **一份新镜像**：崩溃镜像上做完取号、写行、暖机（、一次发布）之后的盘 | 本题 |

D13 已定项 1 自己写过「「就是 I-8.4（重放幂等）」不成立」（`.claude/kb/decisions/13-验证路线.md:33`：「不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质」）。Y7 的输入不是同一张记录表而是一份新镜像（环里多了新实例的记录与根、实例表多了行），所以：M3 的不动点成立不蕴含 Y7（挂载那几次发布写错一个字段，函数层照样不动点）；Y7 成立也不蕴含 M3（Y7 看不到「同一张表」）。**独立一条，叫「挂载透明」**。
I-8.4 的实现计划里有一句「未实现（实现在途：取 R-换，在崩溃注入的二次崩溃上判恢复加写行那一整段」（`.claude/kb/invariants.md:93`）：它判的是二次崩溃镜像上「恢复加写行」；今天第三截已经在每个二次崩溃镜像上跑 `recover` 与旧判官（`crates/singlefs-checker-tier/src/crash_injection.rs:1600`、`:1612`）。Y7 补的是同一族的**端点**：所有挂载写都持久了的那一个状态。今天这个端点哪里都没有：`draw_second_crash_points` 每抽一个点就扣下一个写（`crates/singlefs-checker-tier/src/crash_injection.rs:1510`：「std::cmp::Ordering::Equal => false,」），全持久的状态摆不出来；第二截的 `pool_after` 是全持久的、但只跑池级 checker 与记录核对器，不 `recover`（`crates/singlefs-checker-tier/src/crash_injection.rs:1371`：「let violations = checker_violations_of(&mount_run.pool_after);」）。

## 七、对 Y7 落点的两个缺口 [读]

1. **没有「只挂载、未发布」的镜像**。`mount_and_publish_once_on_the_crash_image`（`crates/singlefs-checker-tier/src/crash_injection.rs:1095`：「pub fn mount_and_publish_once_on_the_crash_image(」）把挂载与发布在同一个函数里做完，交回的 `pool_after` 是发布之后的池，挂载那一段只交了录制流。要 v2a，要么在挂载成功之后、发布之前复制一份设备镜像，要么用已有的挂载录制流在崩溃镜像上整段 `apply_writes`（`crates/singlefs-harness/src/memory_pool.rs:321`：「pub fn apply_writes(」）。前者改这个函数一处，后者不改它。
2. **发布的内容没有保证与 s1 不同**。第 1085 行：「let length = 97 + crash_point.segment_index % 64;」，内容由种子低字节与下标算出（第 1083–1090 行），长度 97–160，没有一条保证它异于历史里任何一版。若它碰巧等于 s1，v2b 比的就分不开「发布落了」与「发布没落、读回的是旧内容」。要加一条：内容的前几个字节写崩溃点的唯一编号，历史生成器的内容不写这个前缀（我没读生成器，这一句要对着 `crates/singlefs-harness/src/history.rs` 核，今天没核）。
3. **整份覆盖使发布之后的 POSIX 比对看不到「挂载动没动旧状态」**：`publish_overwrite` 整份换内容（A1），v2b 的内容与 s1 无关。所以 Y7 要两个点：v2a 判透明、v2b 判「发布本身落了」；只做 v2b 是今天的形态，它判不出挂载有没有改动旧状态。
4. 在 GPU 的崩溃放量判器里做不了：`GPU 判器用 WGSL 另写一份 replay_journal，验不到 core`（`.claude/kb/decisions/13-验证路线.md:31`：「GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core」），Y7 要的是 core 的真 `mount_writable` 加真 `recover`，只能住在 CPU 一侧的崩溃注入里。

**什么现象会推翻 T6**：(1) 在不带故障的崩溃状态里找到一个可写挂载成功、v2a 的 POSIX 状态异于 s1 而池级 checker 与记录核对器都绿——T6 的「挂载透明」就是真缺口，不是多余；找不到就说明它在今天实现上是恒真的（要用必红自检证：摘掉写行或让暖机带一个用户可见的改动，它必须红）。(2) 找到一个状态 v2a = s1 与 M3 不动点都成立、而 T5 行 9 的行集等式不成立——说明 Y7 的根的关系那一半不能并进 M3。(3) 「带故障域不进」这一条的推翻：在带故障的状态里可写挂载成功了且 v2 与 v1 取同一根——那就可以把射程扩进去。

## 八、判定一览

判定三档：**一致**（实现今天的样子与我推出的条款相容）、**冲突**（两边不一样，并排抄在下面）、**规则没说**（条款文字不够推出一条，写成候选句或岔路）。

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 | 一致（POSIX 面规则没说） | 候选句 T1-甲：fsync 返回之后调用开始时刻已接受的全部操作必须在；今天的发布返回条件与它一致，实现里没有 POSIX 面的 fsync；弱读法 a/b 带数列出（1 对 3 个状态） |
| T2 | 规则没说（岔路 T2-F1） | S 读法与 D16 已定项 12 不相容；P 与 Q 都相容、短历史上集合 4 对 2（H3）、5 对 2（H2）；我推 P，交用户；今天的判官有下界（取自录制流）没有上界 |
| T3 | 冲突（并排见下）+ 规则没说 | O1 今天的崩溃判官下界取自与枚举同一份录制流；条款没列「那段代码」是哪些文件，我列了三层；O1 只引格式常量只写在注释里、门禁不看 |
| T4 | 规则没说 | 比的是恢复状态（按内容）是否在 RefFS 允许集合里，接在崩溃注入的三个调用点；C191 第一对今天出不了事件（O3、RefFS 都零代码），替身对条款欠一句 |
| T5 | 一致 + 规则没说两处 | 14 行字段表，POSIX 状态与池身份必须相同、实例代号等六类模掉、实例表行 / F / 水位判单调；行 W 的取值与行格式条款没说 |
| T6 | 规则没说（岔路 T6-F1）+ 实现缺一点 | 比 POSIX 状态加根的关系，射程限不带故障的挂载成功状态；独立的第四条（挂载透明）；今天缺「只挂载」镜像与挂载后的 `recover` |

**冲突并排**：

| 条款（整行） | 实现（整行） |
|---|---|
| `13-验证路线.md:65`：**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS）。这条定案接受「工程量接近再写一个文件系统」这个量级，把 RefFS 从「储备候选」升为**要做的东西**。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。 | `crash_injection.rs:834`：`let newest_persisted = newest_persisted_root(&writes, &persisted)`；`model.rs:2246`：`pub fn crash_recovery_disagreement(`（以它的第二个入参接那个数）。O1 判官不是 RefFS，所以不是违反条款；是 RefFS 由 O1 扩成时最容易沿用下来的一条边，要先写进禁引清单。 |
| `13-验证路线.md:125` 的「（只住内存、只用 `singlefs_format` 的常量、不建崩溃）」 | `model.rs:8`：`//! 不建崩溃：第 1 件的历史里没有断电、没有设备错，每一步的写都落完，所以环里最新的那条根就是最后写出的那一条、journal 里没有要施加的记录` 与同一文件 `model.rs:2246`：`pub fn crash_recovery_disagreement(`——文件头说不建崩溃，文件里已经有崩溃判官。两边都对：不枚举崩溃状态，但判崩溃之后的恢复。文字要改的是 kb 一侧与文件头一侧。 |


## 九、交用户的岔路（每条带代价数；都没被攻过）

| 编号 | 候选 | 代价数（本报告里量到的） |
|---|---|---|
| T1-F0 | fsync 的「必须持久」取全部（T1-甲）还是只取 f 的内容 / 加目录项 | 允许状态数 1 对 3；坏恢复「只剩 a、c 全丢」前者查出、后两者放过（`t1_fsync_reading.py`）。读法 a 与 b 在这段历史上不分，分得开的历史（目录项早于内容写）我没造 |
| T2-F1 | RefFS 前缀粒度：P（单元级前缀）还是 Q（发布级前缀） | H1 末 4 对 2、H2 末 5 对 2、H3 末 4 对 2；P 放过「切开一次发布」，Q 要吃发布边界（复制 D16 已定项 10 的阈值或向实现借） |
| T3-F2 | RefFS 落在哪：(a) harness 档一个模块加禁引门禁，(b) 单独一个包加依赖图门禁 | (a) 新增一格 grep 加一份必红样本；(b) 另需改 checker-independence-and-sync 文件头那句「五个 crate 路径写死」与 `verification.md` 三个包名表 |
| T4-F3 | O3 没有代码时，「RefFS × O1 判官」算不算 C191 里的一对 | 我推「替身对」不计入判红；条款欠一句（`.claude/kb/checks-owed.md:168` 只列三对）；记账成本 = 一张表加一个必红自检 |
| T6-F1 | Y7 只做第一次崩溃镜像（R1）还是加二次崩溃镜像（R2） | 每个第一次崩溃状态：R1 一次或两次（v2a、v2a 加 v2b）「挂载 + 恢复 + 判」，R2 再加 ≤ 3 次（三个相位各一个）；没有 R2 的价值证据 |

## 十、命令与原样输出

下面是 `/tmp/claude-1000/m3-formal-r4-sonnet/cmds.sh` 这一次真跑的输出（脚本与输出都在草稿目录，没有入库：它们只是一串 grep 与 sha256，复跑一遍两秒）。模型脚本与产物在 `research/prompts/m3-formal-r4-sonnet-model/` 四个文件，sha256 在输出末四行。

```
$ sha256sum -c research/prompts/m3-formal-r4-snapshot/crates-sha256.txt | grep -c ": OK$"   (在副本根目录下)
220
$ wc -l research/prompts/m3-formal-r4-snapshot/crates-sha256.txt
220
$ grep -n 'singlefs_core\|singlefs_checker\|segments\|memory_pool' crates/singlefs-harness/src/model.rs
3://! `model_comparison.rs`，那里可以用 `singlefs_core` 的类型）。
5://! 只 `use singlefs_format`，不 `use singlefs_core` / `singlefs_checker`（D13（验证路线） 已定项 5：只共享一份从 kb 生成的常量）：代号、实例、
$ sed -n 22,30p crates/singlefs-harness/src/model.rs | grep -o '[A-Z][A-Z0-9_]\{3,\}' | wc -l
18
$ grep -rn 'harness/src/model\|model\.rs' .claude/gate.d/*.sh | wc -l   (主仓根目录)
0
$ grep -rn 'RefFs\|RefFS' crates | wc -l
0
$ grep -rn '规约执行器\|\bO3\b' crates | cut -d: -f1,2
crates/mutations.tsv:540
crates/singlefs-harness/tests/checker_known_bad_images.rs:5399
$ grep -rn 'unlink_batch_for_tests' crates | cut -d: -f1,2
crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:321
$ grep -rn 'fn [a-z_]*fsync' crates | cut -d: -f1,2
crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1034
crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1039
crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1045
$ nice -n 19 python3 t2_allowed_sets.py --selftest
selftest ok
$ nice -n 19 python3 t2_allowed_sets.py | cmp - t2_allowed_sets.out.json && echo 复跑逐字节相同
复跑逐字节相同
$ nice -n 19 python3 t1_fsync_reading.py | cmp - t1_fsync_reading.out.json && echo 复跑逐字节相同
复跑逐字节相同
$ sha256sum t1_fsync_reading.py t1_fsync_reading.out.json t2_allowed_sets.py t2_allowed_sets.out.json
3147bb450d9305961c2168ee7c7f23a54a6f5f10359bee268aa00d076e42a1f4  t1_fsync_reading.py
704029222381fb382a4cd955fb633002ed2459723f20281484a07305dd52d87d  t1_fsync_reading.out.json
39321e08d1c24a5ca2cf485ca27f252abb40e5d62cc1f27a007531cfeafb77f8  t2_allowed_sets.py
3b6efe80f7caf54f438322e733c7e139dbfdf010336856b39ab501c34180ffe3  t2_allowed_sets.out.json
```

## 十一、没做什么

- 没在入库代码的拷贝上量 v1、v2 的字段值：重型测试不跑、定义不要求编译 Rust；T5 的 14 行都是读码与读条款推的，每行的翻面观测写在表后。量那一遍是攻方腿（T5）的活，量到的数要在入库装置上重做一次才能引。
- 没读 D18 已定项 11（实例表行的格式与 W 的语义），不在分工表点名的条款里；T5 行 9 的行格式与第一处「W」疑点因此只标了「没读」。
- 没做 T2 的模型对真实崩溃枚举的校验：`t2_allowed_sets.py` 是玩具 POSIX 状态机，量的是「几种读法在条款面前给不同集合」的大小，不是实现的崩溃状态；H2 里的 rename、unlink 今天实现里没有，这一历史只证条款面。
- 没判「一段不跨两次发布」（T3 第 3.3 节第 2 条）：没有数撑着，标了 [推]，要量。
- 没写 RefFS 的代码、没改任何条款文字、没替主 agent 采纳或出判决：不判别的腿的格（没有）与判决归主 agent。
- 带故障的崩溃状态上的 Y7（T6 射程）没有推。
- 没跑任何 `cargo`、没起任何重型测试；`gate.sh` 与 lint 没跑（不归本腿）。

## 十二、cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-formal-r4-sonnet-output.md --root /tmp/claude-1000/m3-formal-r4/tree --root . --background research/prompts/_m3-formal-r4-background.md` 末行（交回前最后一次跑）：

```
  ✓ 核了 69 处引文，对上 69 处，没判 0 处（逐处列在上面）
```
