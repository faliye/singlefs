# 判决：并行线二 / 三 / 四带进来的 22 个 `crates/` 文件（代码轮第一轮）

主 agent 写，2026-09-23。正文 `_m2-lines234-code-r1-body.md`，三条腿的报告在同目录。
判决只引产物与代码；腿的结论句一律当线索，进 kb 的每一条由主 agent 现查坐实。

## 一、这一轮交了什么

| 腿 | 分到的格 | 产物 |
|---|---|---|
| 云端攻方（Opus） | X1、X2、X6 | `m2-lines234-code-r1-opus-output.md`（344 行）；副本上的两份改动 `x1-x2-experiment.patch`、`x6-five-kind-reading.patch` |
| 云端正推（Sonnet） | X3、X4、X5 | `m2-lines234-code-r1-sonnet-output.md`（122 行）；kb 引文附录 7 段 |
| 本地攻方（英文，两份干净样本加一次作废） | 22 个文件的测试覆盖 | `-local-attack.md`、`-output-s1.md`、`-output-s2.md`、`-output-void1.md`、`-runlog.md`、`-translation-audit.md` |

⚠️ **这一轮的攻方腿真跑了**（上一轮那条自陈一行没跑）：副本 `/tmp/claude-1000/lines234-opus/repo`，`nice -n 19 -j 6`，X6 两串字节、X1 的容量数、X2 的 13 条用例都是跑出来的。**副本上的数不入库**——要引就由主 agent 在入库装置上重做一次。

⚠️ **本地腿第一次调用被字词损坏闸误判作废**：`research/scripts/corruption-check.py` 的粘连判据把 Rust 的 `crate::module` 里第二个冒号当成了「标点粘死后词」。账已立 C505（字词损坏闸把 Rust 的 `::` 误判成粘连）。

## 二、跑前写死的四条判据，各触发没触发

| 判据 | 触发了吗 | 在哪一格 |
|---|---|---|
| 1 抄不出整段原文一律算「推不出」 | **触发** | X3、X4（两问）、X5 四格全部抄不出 |
| 2 「和条款说反话」要指出哪个字节 / 哪条历史 | **触发** | X6 给了同一次 mkfs 两种读法的两串字节，第一段的种类集合从 `{zero_fill,unit_write,barrier}` 变成 `{unit_write,journal_record,root_record_fua,barrier}` |
| 3 「替没写的条款做选择」要两个选择在盘上分得开 | **触发** | X1、X2、X6 内部那两处 |
| 4 本地攻方缺行号那一行作废 | **没触发** | 22 行四列都给了判据来源，0 行作废 |

## 三、六格逐格判决

### X1　挂载态整片读中央映射　⇒ **替没写的条款做了选择（两处），其中一处守的是走不到的那一侧**

**主 agent 现查坐实**：`crates/singlefs-core/src/transaction.rs` 的 `admission_of_one_publish` 给**分配记录树**与**记账树**各写了一条容量准入（`index_node_entry_capacity` 比一次，超了报错），**映射树一条都没有**。

攻方腿在副本上量出的数（**副本值，入库装置上没重做**）：映射节点容量 294、inode 根 135、一次发布的映射条目上界 140。⇒ 今天不翻车靠的是 **140 ≤ 294 这个两棵无关的树之间的数字巧合**，没有任何东西钉住它；第 295 条时 `build_index_node` 断言 panic。

读侧那条「映射树根不是根兼叶就拒绝打开挂载态」**从今天的写侧走不到**：层级写死字面常量 `0`、没有分裂逻辑。⇒ 正文问的「今天那条拒绝够不够」，答案是**不够，而且不够的方式与正文预想不同**——它不是拦得松，是拦在一条走不到的路上；真正该有的守卫在写侧，而写侧没有。等它走得到那天，它的语义是「整个池挂不上」，把一处可用性悬崖装在 D19（块指针的结构与宽度预算） 已定项 5 的「唯一入口」上。「提示过期的一次解引用发 3 次设备读」这个读数的适用域只写在注释里，不在任何会红的东西里。

### X2　树节点那一侧没有经映射的回退　⇒ **替没写的条款做了选择**（C483（挂载态读映射与树节点回退都没有条款） 第 ② 条）

攻方腿在副本上跑出：同一种损伤（提示指空槽、映射里是真落点），**数据单元**多跳一次读回正确字节，**extent 树根**则让 `open_pool_for_read` 返回 `Walk(UnitUnreadable { … })`、**整池挂不上**。同一份字节在两处结局相反。

映射根与树表不回退**与 D19（块指针的结构与宽度预算） 已定项 8 的自举豁免三类相符**；extent 根 / inode 根 / inode 叶容器不回退，条款里找不到许可它的那一句，但条款也只写了「地位」没写「动作」⇒ 判**替没写的条款做选择**，不是说反话。

### X6　mkfs 清零的动作与步骤种类　⇒ **和条款说反话**（判据第 2 条）＋ 内部另有两处没人定的选择

**主 agent 现查坐实**：`.claude/kb/decisions/17-实现分层与第三方管道.md` 已定项 2 第 1 条逐字写「每段里出现的步骤种类**集合**相同（写单元 / 写 journal 记录 / 根槽 FUA 写 / 系统配置槽原地覆写 / 屏障**五种**）」，而 `crates/singlefs-harness/src/segments.rs` 的 `StepKind` 是**六个成员的封闭枚举**（`ZeroFill`、`UnitWrite`、`JournalRecord`、`RootRecordFua`、`SystemConfigurationSlot`、`Barrier`），`.claude/kb/layout/01-first-txn.md` 的段序列也按六种写。

**但这一格主 agent 要还的是「写回」，不是「重判」**：第六种有用户 2026-09-19 的定案撑着（`records/2026-09-19-里程碑二遗留收拢.md`），只是**没写回 D17（实现分层与第三方管道） 已定项 2**，而同一条分项第 3 条又写着「登记位只有一处」。⇒ 这是一次**定案没落到登记位**，不是设计分歧。

⚠️ **这一格不分辨臂**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中之后先判是哪一种」第一形态）：五种读法与六种读法一起被打中，病根在「没写回」这个共用前提上 ⇒ 不拿它判臂，另立一笔账。

第六种**里面**另有两个没人定的选择，各自在盘上分得开：**(a)** 按动作归类还是按落点归类（同一张表里今天混了两套归类法，后果落在等价类数上）；**(b)** 「清零算一次整写」——D13（验证路线） 已定项 4 把撕裂并进「没持久」的理由是「差别只有 CRC32C 碰撞概率」，而**被清的那一段没有校验和罩着**（`crates/singlefs-core/src/make_filesystem.rs` 自己的注释就是这么说的）。

### X3　inode 树根的 key 区间取分隔 key 的首尾　⇒ **推不出**

`separator_key()`（分隔 key = 创建时最小 key）兑现 D8（核心索引结构） 已定项 6，但根头 `[min_key, max_key]` **取条目键区间而非子树覆盖区间没有任何已定分项支持**，C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） 明登为待定。

### X4　inode 记录 `blocks` 与建 inode 的四个固定值　⇒ **两问都推不出**

`blocks`：字段表只给宽度不给算法，`crates/singlefs-core/src/records.rs` 自述「没写怎么算」，与 C480（inode 记录的 blocks 怎么算全仓没有条款） 逐字吻合；`crates/mutations.tsv` 里只钉住「不能是常数 64」，钉不住该按什么口径算。
mode / uid / gid / nlink 四个写死值：D8（核心索引结构） 已定项 6 字段表只给宽度不给默认值，字节表那句「已定」说的是第一个事务那次发布写出的字节、不是通用策略；这四个值**全仓没有编号登记、也没有任何变异钉着**。

### X5　一次发布只重写被改到的那几片叶容器　⇒ **推不出**

与正文观测一致：并行线三的现状原句「没有条款也没有不变量」。`crates/singlefs-core/src/inode_tree.rs` 与 harness 单测有断言，但 `crates/mutations.tsv` 里 6 条点名 `inode_tree.rs` 的变异**都不改这一处**，没进复跑门禁。

### 测试覆盖那一维（22 个文件）

本地腿逐行判完 22 行，两份干净样本。**主 agent 自己数了一遍变异表那一列**（`awk -F'\t' '$2==路径'`，`crates/mutations.tsv` 共 299 行）：这 22 个文件里 **8 个零变异**——`crates/singlefs-core/src/address.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/pointer.rs`、`crates/singlefs-core/src/write_request_split.rs`、`crates/singlefs-harness/src/read_tally.rs`、`crates/singlefs-harness/src/segments.rs`、`crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs`、`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`，与本地腿数出的 8 个逐个相同。

⚠️ **最值得记的一格**：`crates/singlefs-harness/src/segments.rs` **零变异**，而 X6 打中的那条「条文与实现说反话」正住在它里面（`StepKind` 那个六成员枚举）。归类法改一下，段序列整串就变，而门禁 52 号逐字比的就是那一串——今天没有任何一条变异在盯它。

本地腿另报：22 个文件里 **3 个今天完全没有测试**——`crates/singlefs-core/src/lib.rs`、`crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs`、`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`。

### 这一轮被判过的 22 个文件（门禁 56 号按路径点名）

`crates/singlefs-core/src/address.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/pointer.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/root_ring.rs`、`crates/singlefs-core/src/write_request_split.rs`、`crates/singlefs-harness/src/bad_disk_input.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/crash_injection.rs`、`crates/singlefs-harness/src/device_log.rs`、`crates/singlefs-harness/src/first_transaction_regions.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/read_tally.rs`、`crates/singlefs-harness/src/scenario.rs`、`crates/singlefs-harness/src/segments.rs`、`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs`、`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`。

## 四、这一轮要落的（不需要用户定的）

1. **把第六种步骤写回 D17（实现分层与第三方管道） 已定项 2**：用户 2026-09-19 已定案，只是没落到登记位，而同一分项第 3 条写着登记位只有一处。
2. **给 `crates/singlefs-harness/src/segments.rs` 补变异**：至少钉住「归类法按动作不按落点」与「六个成员的种类串规范序」，否则段序列那一串没有任何东西盯着。
3. **改 `crates/singlefs-core/src/transaction.rs` 那句注释**：它引「I-1.1（key 区间罩住条目）」，而登记位上 I-1.1 的简称是「块头自述逻辑地址」——引对了编号、自造了第二个简称（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 5 条）。正文确实罩着 key 区间（「索引节点是树 ID + 层级 + 该树已定携带的 key 区间」），改简称即可，不改指向。
4. **给映射树的写侧准入立一笔账**：X1 那个 140 ≤ 294 的巧合要有东西钉住。

## 五、要交用户的

X1 的写侧准入怎么加、X2 的回退动作要不要写成条款、X6 内部那两个选择（按动作还是按落点归类、清零算不算一次整写）、X3 / X4 / X5 三格的条款该怎么写。**这一轮一格都没有替用户定**；攻方腿自己提的 5 个改法**被攻过零轮**，交用户时按这个标注。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D17（实现分层与第三方管道） 已定项 2 | 推翻 | 2026-09-23 改了：第 1 条逐字列五种步骤，而活代码与字节表都是六种；用户 2026-09-19 的定案要写回这条分项 |
| D19（块指针的结构与宽度预算） 已定项 5 | 备料 | 2026-09-23 不受影响：它只定「映射是解引用的唯一入口」，整片读与写侧准入都在它射程之外 |
| D19（块指针的结构与宽度预算） 已定项 8 | 支撑 | 2026-09-23 不受影响：映射根与树表不回退正是它的自举豁免三类 |
| D8（核心索引结构） 已定项 6 | 备料 | 2026-09-23 不受影响：字段表只给宽度，不给 key 区间口径与默认值 |
| D13（验证路线） 已定项 4 | 备料 | 2026-09-23 不受影响：撕裂并进「没持久」的理由压在「有校验和罩着」上，而被清零那一段没有 |
| D5（快照 / 空间记账机制） 已定项 8 | 不影响 | 2026-09-23 不受影响：记账树的准入这一轮没被判 |
