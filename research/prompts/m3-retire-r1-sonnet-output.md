# m3-retire-r1 云端正推（Sonnet）报告：退役树与退役记录的盘上形态

日期 2026-10-09（`TZ=Asia/Tokyo date +%F`）。立场：从条款与冻结树推「对的形态」，每格给能照着写代码的定义、落 `crates/` 哪几处、改哪些钉值与哪几条条款。不判别的格不替主 agent 判。

## 0　读法、结论速览、复核

**路径约定**：`crates/…` 指冻结树 `~/.local/share/singlefs/work/m3-retire-r1/tree/` 下的相对路径（全路径 = 该根 + 相对路径），行号是冻结树里的；kb 引文写 kb 文件自己的行号。核引文：`python3 research/scripts/cite-check.py <本报告> --root ~/.local/share/singlefs/work/m3-retire-r1/tree --root <仓根>`（`crates/…` 在前一个根下找，`.claude/kb/…` 与 `research/…` 在仓根下找；只给一个根会把另一类引文列成「没判」）。「方案按 `crates/` 今天的实现来谈」：下面每一处落点都是先读了冻结树的代码再写的。

**结论速览**（细节在各节，推翻条件在各节末）：

| 格 | 一句话 |
|---|---|
| K1 | 种类码 13、树 ID 65548、树表排最后（第 12 条）、头 ID 取 inode 树的树 ID、day-1 零根注册（水位 65548 → 65549、新池新建文件 11 → 12 条）、不加 incompat 位 |
| K2 | 记录 34 字节：身份引用 26（出生树 8、类型 2、容器号 8、出生代 8）+ 死亡 txg 8，一容器 959 条；不加头 ID、flags、预留；退役集合 = 上一版叶身份集 − 这一版叶身份集（集合差），死亡 txg = 这次发布的 checkpoint_txg |
| K3 | 四条照搬全部成立；唯一要改的是追加接口要能一次追加多条；管理员回退退役树照抄 cur 对，R_old 之后才死的容器靠「记录的死亡 txg 不在父链上」判成活；崩溃恢复靠「先按实例表滤版本、再读那一版的记录」，读不到被抛弃段里写的记录 |
| K4 | 一次发布至多 1 条（最后一批只删一条记录、一次删除至多一个合并动作）；退役角色归普通分配（同墓碑），不进 ckpt_cost，D28 已定项 4 原文不改；要改 D3 已定项 2 的标签 6 式子和 D28 已定项 1 的 W 式子 |
| K5 | 立 5 条：条目身份与子头相符（类型段 ∈ {0, 3}）、记录合法、谱系上不重身份、谱系上的记录不指本版活叶、相邻版本之间消失的叶身份与死亡 txg = 本版 txg 的谱系上记录一一对应；前三条单靠它们抓不到「漏写」，漏写只有第 5 条抓得到；GPU 先同判树表那几处，记录级回落 CPU |
| K6 | M3 那句话对「今天 `crates/` 里有写者」的形态完整；扫描重建读侧记欠账，欠账必须连「候选单元的诞生 txg 也要在父链上」一起记（C615），否则按 kb 字面读出来回退之后复活 |

**复核（这一次真跑的原样输出）**：

```
$ grep -rn "退役记录" crates --include=*.rs; echo "exit=$?"
exit=1
$ grep -rlin "RetirementRecord\|TREE_KIND_RETIREMENT\|PACKED_TYPE_RETIREMENT\|retirement_tree" crates --include=*.rs --include=*.wgsl; echo "exit=$?"
exit=1
```

两条 grep 在冻结树上都是 0 命中：`crates/` 里没有退役树、没有打包记录类型 3 的写者与读者（含英文名）。

**独立小模型**：`research/prompts/m3-retire-r1-sonnet-model/`（纯 Python，不读 `crates/`；`SINGLEFS_REPO=<仓根> bash rerun.sh <空目录> all`；产物 `expected/summary.txt`）。它不是对实现的核对，只证形态的自洽：300 段历史 × 60 步 × 三种合并政策 × 有无回退 × 有无崩溃（崩溃分「txg 重发」与「txg 跳号」）。下面引它的数都写「模型里」。

## K1　登记：种类码、树 ID、树表、新池新建文件

### K1.1　应该是什么

1. **种类码 13**（`TREE_KIND_RETIREMENT`）。依据：D8 已定项 9 逐字「新树只追加不重用码」（`.claude/kb/decisions/08-核心索引结构.md:295`），今天最大是 12：「pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;」（`crates/singlefs-core/src/records.rs:385`）。登记表末尾追加一行 `| 13 | 退役树 | D18 已定项 11 |`。
2. **树 ID 65548**（`TREE_IDENTIFIER_RETIREMENT`），树 ID 水位第一个文件版本之后 65549。依据：树 ID 从水位起连号发，今天最后一个是「pub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 65547;」（`crates/singlefs-format/src/lib.rs:204`），水位是「pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65548;」（`crates/singlefs-format/src/lib.rs:207`）；水位等于最后一个号 + 1 由测试钉死（lib.rs:683-686）。
3. **树表排最后**（第 12 条）。I-9.16 ② 按发号次序要求相邻两棵树 ID 严格升序，发号次序只追加，所以退役树排在 logged_ops 之后；盘上次序（① 树 ID 升序）与发号次序一致。
4. **头 ID = 这张树表里 inode 树的树 ID**（单头时 65537），关系判据同墓碑树。依据：D8 已定项 8 逐字「deleted_inodes 与墓碑树写 inode 树的树 ID（65537），谱系树写 0（全池一份）」（`.claude/kb/decisions/08-核心索引结构.md:262`）。退役记录点名的是某个头的叶容器身份，死活是相对头的；挂「全池一份」的谱系树那一类（写 0）配不上 I-9.33 要的「退役树 ↔ inode 树」配对（见 K5）。
5. **day-1 零根注册**：第一个文件版本那次发布就把退役树写进树表，根指针全零（`NodePointer::empty_root()`），不写任何单元。先例：D8 已定项 9 射程写「day-1 注册进树表比第一次建快照时惰性创建少一条」（`.claude/kb/decisions/08-核心索引结构.md:315`，句子在此处接着写「首次建树」路径），D18 已定项 24 第 6 条给墓碑树、谱系树同样 day-1 零根。
6. **不加 incompat 位**，并进第一条布局线（incompat 位 1）。依据：「不加新 incompat 位，并进第一条布局线。」（`.claude/kb/decisions/18-块里携带什么信息.md:580`）与「类型 3 / 4 按 D15（格式冻结政策） 默认档 incompat，冻结前零代价」（`.claude/kb/decisions/18-块里携带什么信息.md:301`）；打包记录类型默认 incompat 的理由是「新增打包记录类型默认也是 incompat」（`.claude/kb/decisions/18-块里携带什么信息.md:323`），这一条由布局线 incompat 位 1 兜住：旧读者不认布局线就挂不上，没有「认布局线却不认类型 3」的读者。格式还软：「第一个外部用户出现之前格式是软的」（`.claude/kb/decisions/15-格式冻结政策.md:3`）。

### K1.2　落 `crates/` 哪几处（只列登记相关；记录、发布、准入、checker 的落点在 K2–K6）

| 文件:行（冻结树） | 改法 |
|---|---|
| `crates/singlefs-format/src/lib.rs:188` | `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES` 11 → 12 |
| `crates/singlefs-format/src/lib.rs:204` 之后 | 加 `TREE_IDENTIFIER_RETIREMENT: u64 = 65548` |
| `crates/singlefs-format/src/lib.rs:207` | `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH` 65548 → 65549；同文件测试 `lib.rs:630-690`（连号表、水位 = 最后一个号 + 1、条数 11）跟着改 |
| `crates/singlefs-core/src/records.rs:385` 之后 | 加 `TREE_KIND_RETIREMENT: u16 = 13`；`records.rs:560-603` 的值域测试两张表各加一项 |
| `crates/singlefs-core/src/unit.rs:25` 之后 | 加 `PACKED_TYPE_RETIREMENT: u16 = 3`（类型 3 在 D18 已定项 11 里登记过、谱系记录因此取了 5，`unit.rs:23-25` 的注释同形） |
| `crates/singlefs-core/src/transaction.rs:2668`（`FileVersionTreeIdentifiers`） | 加字段 `retirement`；`transaction.rs:2692` 的 `OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER: [u64; 12]` → 13 项，下面的解构（`issued_from_watermark`）与「十二个号」的注释跟着改 |
| `crates/singlefs-core/src/transaction.rs:8340-8400` | 树表条目数组末尾加 `table_entry(TREE_KIND_RETIREMENT, trees.retirement, <根或 empty_root>, trees.inode.0)`；`transaction.rs:8402` 树表节点的最大 key 由 `trees.logged_operations` 改 `trees.retirement` |
| `crates/singlefs-core/src/transaction.rs:6297` | 没有上一版的发布断言「计划写的新水位必须就是 65548」，常量变了断言文字跟着变（它读常量，不用改逻辑） |
| `crates/singlefs-core/src/recovery.rs:2372` | `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 11]` → 12，末尾加 13；`recovery.rs:2580` 加 `retirement: tree_of(TREE_KIND_RETIREMENT)?` |
| `crates/singlefs-core/src/mount.rs:7811-7818` | **不改**：回退只有 extent、inode、`deleted_inodes` 三种取 R_old 的根，别的（含退役树）落在 else 支照抄 cur |
| `crates/singlefs-harness/src/mount_transparency.rs:591` | `TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES: [u16; 5]` 加 13 变 6（退役树根只会被删除续做 / 会话 unlink 改，挂载本身不改） |
| `crates/singlefs-checker/src/walk.rs:88-110` | 种类码常量加 13，`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`（`walk.rs:93`）与 `TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`（`walk.rs:4345`）`[u16; 11]` → 12；`walk.rs:4244` 的头 ID 分支把 13 并进「等于同表 inode 树的树 ID」那一组 |
| `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:139`、`crash_judge_checker_common.wgsl:402-406` | `TREE_KINDS_OF_THE_TREE_TABLE` 11 → 12；`C_MAX_TREES_PER_TABLE` 11 → 12，`C_CANDIDATE_WORDS` 50 → 54，其后 31 个草稿区偏移常量各 +192，`crash_judge_gpu.rs:134` 的 `SCRATCH_WORDS` 315_376 → 315_568 |

### K1.3　day-1 零根 vs 第一次有退役记录才建：各碰什么

先量触点（原样命令输出，冻结树）：

```
$ grep -rn -w '65548' crates --include=*.rs --include=*.wgsl --include=*.tsv | wc -l   → 57
$ grep -rln -w '65548' crates --include=*.rs --include=*.wgsl --include=*.tsv | wc -l  → 16
$ grep -rn 'NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES' crates --include=*.rs | wc -l   → 5
$ awk 'NR>=410 && /^const [A-Z_]+: u32 = [0-9]+u;/ {n++; if ($5+0>=168816) m++} END{print n, m}' crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl   → 94 31
```

| 项 | day-1 零根（推荐） | 第一次有退役记录才建 |
|---|---|---|
| 新池新建文件的字节 | 变：树表单元条目 11 → 12（声明长度 2200 → 2400，D8 已定项 8 原文「声明长度（11 × 200 = 2200）」，`.claude/kb/decisions/08-核心索引结构.md:277`）、根记录与 journal 新根段里的水位 65548 → 65549；不多写任何单元（零根不写单元） | 不变 |
| 发布路径新分支 | 无 | 有：第一次含退役记录的那次最后一批要发树 ID、写树表条目、抬水位；`crates/` 里水位只在第一个文件版本那次动（`transaction.rs:6297` 的断言），要加第二处 |
| 读树表 | `tree_of` 恒找得到（`recovery.rs:2580`） | 找不到要变成合法（今天缺一种报「树表里没有那棵树」），I-9.16 ④ 要从十一种各恰好一条改成十一种恰好一条、退役树至多一条（`crates/singlefs-checker/src/walk.rs:4345`、`crates/singlefs-core/src/recovery.rs:2372`、`.claude/kb/invariants.md:304`） |
| 崩溃点 | 新池新建文件那次的单元数不变 ⇒ 崩溃状态数不变；含退役记录的删除类发布每个多 2 + H 个单元（容器 1 个 32 KiB、根 1 个、H 个内部节点；折成 16 KiB 块是 3 + H）⇒ 删除线崩溃流里这些发布的写点按单元数多 | 同左，另加：首次建树那一发布的树表条目数变化、回退候选里有的根没有这一条（回退照抄 cur 的条目，R_old 的树表没有，`mount.rs:7805-7818`）、被抛弃的首次建树发布发出的树 ID 崩溃后可重发（D8 已定项 8 射程「丙-2 唯一盖不住的那一格」同形） |
| GPU 树表容量 | 11 → 12 的重排一次做完 | **一样要做**：第 12 条树表条目一出现，「第 12 棵起判不了、交 CPU」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:405`）就让每个这样的状态回落 CPU；惰性不省这一项 |
| 钉值 | 16 个文件 57 行带 65548 字面量、5 处 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`、`crates/mutations.tsv` 5 行（1593、1600、1601、1602、1606） | 约 0（新池字节不变），但 I-9.16 ④ 的坏镜像与测试要改 |

判：day-1。先例两条（D8 已定项 9 射程、D18 已定项 24 第 6 条）、少一条首次建树路径、惰性在 GPU 上一分钱省不下；代价是新池字节与 16 个文件的钉值，这些都是机械的同值替换。

### K1.4　要改的条款原文（整行并排；动 D8 / D18 / D3 / D28 的写回之前交用户）

| 现行（文件:行） | 应为 |
|---|---|
| 登记表最后一行是码 12「logged_ops 树」（`.claude/kb/decisions/08-核心索引结构.md:311`） | 其后加 `| 13 | 退役树 | D18 已定项 11 |`；`:313` 「新池新建文件写出的 11 条树表条目照此表」改 12 条（码 1..4、6..13） |
| 「deleted_inodes 与墓碑树写 inode 树的树 ID（65537），谱系树写 0（全池一份）」（`.claude/kb/decisions/08-核心索引结构.md:262`） | 「deleted_inodes、墓碑树与退役树写 inode 树的树 ID（65537）」 |
| 「声明长度（11 × 200 = 2200）与 11 条条目的排布」（`.claude/kb/decisions/08-核心索引结构.md:277`） | 12 × 200 = 2400 与 12 条 |
| 「树表 7 → 11 条（deleted_inodes、墓碑树、谱系树、logged_ops 零根，最后一棵见 D8（核心索引结构） 已定项 5），树 ID 水位 65544 → 65548（推的：四棵各占一个树 ID；树 ID 从 65536 起编，D8（核心索引结构） 已定项 11）；不加新 incompat 位，并进第一条布局线。」（`.claude/kb/decisions/18-块里携带什么信息.md:580`） | 加一句：退役树（D18 已定项 11 类型 3）同样 day-1 零根，11 → 12 条、水位 65548 → 65549，不加 incompat 位 |
| 「④ 树表非空时进树表的十一种（② 那一串）各恰好一条」（`.claude/kb/invariants.md:304`） | 十二种，② 那一串末尾加退役树；`:315` I-9.27 的「种类 1（extent 树）、9（deleted_inodes）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID」加 13 |
| 「树 ID 水位在 mkfs 时写 65536、第一次发布后写 65548」（`.claude/kb/layout/01-first-txn.md:49`） | 65549；`:375`「mkfs 65536，发布后 65548」同改，`:325`、`:353` 新根段同改 |

### K1.5　判定与推翻条件

- 与现行条款逐格：种类码追加、树 ID 连号、day-1 零根、不加 incompat 位 = **一致**（各引见上）；头 ID 的值、树表第 12 条 = **规则没说**（D18 已定项 11 写「头 ID、在树表里的次序……没定」，`.claude/kb/decisions/18-块里携带什么信息.md:301`）；上表六行 = **冲突**（条数 / 水位 / 种类数的字面值要改）。
- 推翻 day-1：若主 agent 量到「新池新建文件的字节」不能再变（有外部用户的镜像了）——D15 开头一段「格式还软」的前提不成立；或 16 个文件的钉值替换有一处不是同值替换而是逻辑改动。
- 推翻头 ID = inode 树 ID：若用户定退役树「全池一份」跨头共用，则每条记录要加头 ID（K2 的候选，记录 34 → 42 字节、一容器 959 → 776 条）。

## K2　退役记录的字段表

### K2.1　应该是什么：34 字节，小端

| 偏移 | 字段 | 宽 | 说明 |
|---|---|---|---|
| 0 | 被退役身份·出生树 | 8 | 身份引用的第 1 段，单头时 = inode 树的树 ID |
| 8 | 被退役身份·打包记录类型 | 2 | 第一版恒 2（只有 inode 叶容器会退役；类型 1 容器归写者树独占、从不退役，D18 已定项 11 的「类型 1 容器归写者树独占」那一条） |
| 10 | 被退役身份·容器号 | 8 | |
| 18 | 被退役身份·容器出生代 | 8 | |
| 26 | 死亡 txg | 8 | 身份第一次不在后继版本里的那次发布的 checkpoint_txg |
| 34 | （记录结束） | | 无填充、无 flags、无预留 |

26 字节身份引用与 inode 树内部节点条目里的身份引用同序同宽：「writer.put_u64(child_identity.birth_tree.0);」（`crates/singlefs-core/src/records.rs:222`）紧接「writer.put_u16(child_identity.record_type);」（`crates/singlefs-core/src/records.rs:223`），再容器号、容器出生代。读写共用 `PackedIdentity` 的序列化，不另立一套字节次序。

- 记录宽 34；一容器 ⌊(32768 − 136) ÷ 34⌋ = **959** 条（记录区起点 136 = 头 107 + 预留位 29，D18 已定项 16）。与 E178 模型同值：「身份消失（删空、被吸收、N2 借）那次发布追加一条 34 字节退役记录，退役树一容器 959 条、有新记录那次重写开放容器与根」（`.claude/kb/experiments/178-叶容器与记录死亡的计数.md:29`）。
- 登记：D18 已定项 11 的打包记录类型表里「3 退役记录」补宽 34、一容器 959；I-1.7 的「记录宽 == 登记表给该类型的宽」那一串补类型 3 = 34（现行写「类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16」，`.claude/kb/invariants.md:32`）。
- 代码：`crates/singlefs-format/src/lib.rs:113-122` 一带加 `RETIREMENT_RECORD_BYTES: u64 = 34`、`RETIREMENT_CONTAINER_RECORDS: u64 = 959`（带 `format-const` 注释）；`crates/singlefs-core/src/records.rs:184` 的 `LineageRecord` 旁加 `RetirementRecord { retired: PackedIdentity, death_txg: CheckpointTxg }` 与 `to_bytes` / `parse`（长度不是 34 交回 `None`，同谱系记录）；`crates/singlefs-checker/src/walk.rs:189` 的 `registered_record_width` 加 `PACKED_TYPE_RETIREMENT => Some(34)`；GPU 扫描内核 `crash_judge_checker_scan.wgsl:2169` 旁加同名常量（谱系那条是 `DELETION_LINEAGE_RECORD_BYTES: u32 = 16u`）。

### K2.2　要不要别的段：每段写它防哪一形，不防就不加

| 候选段 | 防的形 | `crates/` 里这一形有产生者吗 | 判 |
|---|---|---|---|
| 头 ID（8） | 克隆头共享 origin 的叶，一个头摘掉一片、另一个头还在用 | 没有：`grep -rn -i 'clone_head\|克隆头' crates --include=*.rs` 0 行（这一次真跑） | **不加**。头范围由「每头一棵退役树 + 树表条目头 ID = 该头 inode 树的树 ID」（K1）承担，与墓碑树、`deleted_inodes` 同法 |
| 写序（10） | 同一 checkpoint 内平局、择新 | 容器自己的头里有写序（码 3 类身份段），记录在容器里按追加先后排；D18 已定项 11 已定「不带写序」 | **不加** |
| flags / 预留 | 前向兼容 | 记录内任何赋义本来就 incompat：「预留 20 不承重：记录内任何赋义都是 incompat，宽度只决定那次要不要连记录宽一起改」（`.claude/kb/decisions/08-核心索引结构.md:199`）。`.claude/rules/fs-design.md` 的「不为省空间牺牲自包含」只管「已经论证过有用的字段」，这里没有论证过有用的字段 | **不加** |
| 退役原因（删空 / 吸收 / 重生） | 读者区分死法 | 读者不区分：三种都是「这个身份从这次发布起不在」 | **不加** |
| 被退役容器最后一版的诞生代号 | 读者「版本诞生 txg ≤ 死亡 txg 才杀」那条规则的输入（E178 读者） | 读者读容器单元头就有 | **不加** |

### K2.3　死亡 txg 与退役集合：取集合差，不逐动作

**定义**：一次发布 P 写的退役集合 = { 上一版这个头的 inode 树叶容器身份 } − { 这一版这个头的 inode 树叶容器身份 }；每条记录的死亡 txg = P 的 checkpoint_txg；P 里的记录按（出生树、类型、容器号、出生代）升序追加，保证「同一个参数两次发布逐字节相同」。

落点：`crates/singlefs-core/src/inode_tree.rs` 加 `pub fn retired_identities(before: &[InodeLeafContainer], after: &[InodeLeafContainer]) -> Vec<PackedIdentity>`；`InodeLeafContainersAfterThisPublish`（`inode_tree.rs:118`）加字段 `retired: Vec<PackedIdentity>`（`containers_dropped` 保留，`inode_tree.rs:125`）；`transaction.rs:5091` 的 `ordinary_allocation_part` 在 inode 树结果算出之后（`transaction.rs:5119` 删除支、`:5123` 写入支）算一次集合差，交给退役树的追加（照墓碑树 `transaction.rs:5193-5210` 那一段）。

为什么是集合差而不是「每个动作各写一条」：

1. **同一次发布里「分裂 → 合并 → 再分裂」**（转述 D8 已定项 6 射程，`.claude/kb/decisions/08-核心索引结构.md:207`，同一次发布内造不出同身份两版）：只在这次发布里存在过的身份没落过盘，不在「上一版」集合里，集合差里没有它，不写；再分裂出的同号同出生代身份在「这一版」集合里，也不写。逐动作写会为一个从没落盘的身份写一条无处可用的记录。
2. **借**：右片保留身份，在两个集合里都有，集合差自然没有它；「借不写」不是要单独遵守的规则，是结果。
3. **以后的克隆头首次 COW 重生**：旧身份在这个头的上一版、不在这一版，集合差自动写；新身份在这一版里，不写。K6 的「漏掉的形态」靠这一条闭合。
4. 一次发布的记录数 = 集合差的大小，K4 的上界直接读它。

**同一个身份会不会退役两次**：一条时间线上不会。退役之后它不在任何后继版本里；它也生不回来——容器号是触发那次分裂的新记录的 inode 号，出生代是那次发布的 txg，inode 号「同一条时间线上 inode 号不复用」（`.claude/kb/decisions/08-核心索引结构.md:207`）。**跨管理员回退会**：缺口里退役过（死亡 txg 在 (T_old, T_rb) 之间），回退之后它又活着，再被摘掉时第二条记录的死亡 txg > T_rb。两条都留着，缺口里那条靠「死亡 txg 不在这一版父链上」不生效（K3、K5）。**跨崩溃恢复**：被抛弃段里写的记录不在所选根指着的退役树里；新实例重发同一个 txg 时写的是它自己的记录。

模型里（`m3-retire-r1-sonnet-model`，3 种合并政策 × 有无回退 × 有无崩溃共 12 格、每格 300 段 × 60 步）：单次发布的退役记录数最大值 `max_retire_records_in_one_publish=1`（12 格都是 1）；读者 B 在 12 格的复活与误判死亡全 0；集合差不是模型的前提，是 `finish()` 里算出来的（`model.py` 的 `diff = sorted(before - st.ids())`）。

### K2.4　判定与推翻条件

- 与现行条款逐格：四元组 + 死亡 txg、不带写序、住自己的容器、容器号单调分配 = **一致**（「记录 = 被退役的 (出生树, 类型, 容器号, 出生代) 四元组加死亡 txg（身份消失那次发布的 checkpoint_txg），不带写序；住自己的容器，容器进一棵专用的退役树，容器号按类型 1 那套单调分配」，`.claude/kb/decisions/18-块里携带什么信息.md:301`）；逐字节次序、记录宽、一容器条数、头 ID / flags / 预留的取舍 = **规则没说**（同一行：「退役树的种类码、树 ID、头 ID、在树表里的次序与退役记录的逐字节字段表没定，随 crates/ 那一批的设计定」），本节定。
- 推翻：① 用户定退役树全池一份 ⇒ 要头 ID（记录 42 字节、一容器 776 条）；② 任一合法历史里出现「身份消失而集合差里没有它」——例如一个动作改了叶容器身份又没经过上一版 / 这一版的叶容器集合（`crates/` 今天没有这种动作）；③ 主 agent 在入库装置上量到单次发布的退役记录数 > 1 而 M3 政策声称一次删除只做一个合并动作。

## K3　退役树在发布、回退、崩溃恢复里随哪一版

### K3.1　墓碑树的五条纪律，退役树上哪一条不成立

| 墓碑树的纪律（出处） | 退役树 | 说明 |
|---|---|---|
| 「容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1」（`.claude/kb/decisions/18-块里携带什么信息.md:575`） | 成立 | 容器身份 = (退役树 ID, 3, 容器号, 这次发布的 txg)；与 D18 已定项 11 登记的「容器号按类型 1 那套单调分配」同一句 |
| 「有新记录的那次发布才重写，不删除的发布照抄；满了才关」（`.claude/kb/decisions/18-块里携带什么信息.md:577`） | 成立，换主语 | 「有新退役记录的那次发布才重写」；其余发布走 `ContainerTreeAfterThisPublish::carried`。**一处接口要改**：`container_tree.rs:423` 的 `append_record_to_the_open_container` 一次只追加一条（「rewritten_containers: vec![rewritten_position],」，`crates/singlefs-core/src/container_tree.rs:469`），退役记录一次发布可以不止一条（K4 的 c(P) ≤ 2 那一格） ⇒ 加 `append_records_to_the_open_container(… records: &[Record] …)`：逐条追加、开放容器装满就开新容器、`rewritten_containers` 升序去重、形状逐个容器按 `after_a_container_is_opened_at_the_right_end` 折；单条版改成它的一元素调用，一套实现 |
| 「第一版不回收墓碑」（`.claude/kb/decisions/18-块里携带什么信息.md:578`） | 成立，且更要紧 | 退役记录要活到被退役身份的所有旧版本单元都不可读为止；第一版没有回收单元的清扫 ⇒ 退役记录永久。10⁶ 次退役 = 1043 个容器 = 34 177 024 字节（两份 68 354 048），与 10⁶ 次 unlink 的墓碑 1227 个容器同量级（`.claude/kb/decisions/18-块里携带什么信息.md:578` 引的 76.7 MiB）。统计量 13、14 第一版不登记，退役树也不登记 |
| 根满长高 | 成立 | 共用 `container_tree.rs`，条目 120 字节同形，类型段 ∈ {0, 3} |
| 写进树表、随现行那一版 | 成立 | 见 K3.2 |

没有哪一条不成立。**多出来的一条代价**：可写挂载重建上一版时整棵读回来（`recovery.rs:2790-2830` 读墓碑树、谱系树的整棵容器，退役树加第三棵），挂载读量随退役条数线性涨，与墓碑树同形（今天已有的 O(条数) 项，退役树把它加倍而不是新建）。

### K3.2　管理员回退：照抄 cur

1. **代码**：`mount.rs:7811-7818` 只有 extent、inode、`deleted_inodes` 三种取 R_old 的根（「|| entry.kind == TREE_KIND_DELETED_INODES」，`crates/singlefs-core/src/mount.rs:7814`），退役树落在 else 支照抄 cur，不用改这一段。要改的是：`mount.rs:7862-7866` 墓碑 / 谱系两行旁加 `retirement_containers: current.retirement_containers.clone()`、`retirement_tree_index_nodes: …`；`is_a_user_visible_unit`（`mount.rs:7447`）与 `is_a_unit_taken_from_the_rollback_target`（`mount.rs:7481`）两个穷尽 match 各加三个退役变体（前者 `false`，后者经 `is_a_user_visible_unit` 得 `false`）——编译器强制，漏一个编不过。
2. **条款**：D23 已定项 14 现写「墓碑树、谱系树随现行那一版，不取 R_old」（`.claude/kb/decisions/23-journal的角色与格式.md:378`），补成「墓碑树、谱系树、退役树随现行那一版」。D18 已定项 11 已写「管理员回退那次退役树随现行那一版、不取 R_old（不改 D23（journal 的角色与格式） 已定项 14）」（`.claude/kb/decisions/18-块里携带什么信息.md:301`）——**一致**；C760 也是这一句：「读侧另有一条约束：随现行的树（退役树、墓碑树、谱系树）不按谱系父链滤、只按 C113」（`.claude/kb/checks-owed.md:475`，句子在此处接着写「定案 P2 择版本」）。
3. **R_old 之后才死、cur 里已退役、回退后在 R_old 那一版 inode 树里又是活叶的容器**：它的记录死亡 txg d 满足 T_old < d < T_rb。回退那次发布的谱系记录 (T_rb, T_old) 使这一版父链 = [T_rb] ∪ [0, T_old]（`walk.rs:10134` 的 `lineage_chain_of_a_version`：T_rb ≤ 此刻的记录 [T_rb, 此刻] 入链、跳到 T_old）。d 不在链上 ⇒ 记录不生效 ⇒ 判成活。**模型里**（`m3-retire-r1-sonnet-model`，borrow 政策、有回退有崩溃 300 段）：读者 C（退役记录不看谱系）误判死亡 1438 次，读者 A、B（按死亡 txg 滤谱系）0 次；无回退的两格 C 也是 0——误判死亡只来自回退，滤掉它的就是记录级的谱系判。
4. **常规挂载路径会不会读退役树**：读，但不用它判活。`rebuild_version` 要把退役树整棵读回来才能接着发布（追加、照抄容器、算内部层数），这是写侧的延续，同墓碑树、谱系树；常规路径的活死由 inode 树的可达性定，不看退役记录。读侧第一个真正用记录判活的是扫描重建（今天 `crates/` 没有，K6），其次是 checker 的 K5 几条。

### K3.3　崩溃恢复：被抛弃段里写的退役记录读不到

**机制**：退役树随所选根指的那一版走。被抛弃段（恢复前老实例发布了单元、没发布成根的那些）里写的退役记录落在两类单元里：老退役容器的新版本（身份不变、诞生代号 = 被抛弃的 txg、实例 = 老实例），或被抛弃段新开的容器。扫描方向看到它们时，**必须先**按实例表的已发布谓词滤版本（C113 定案 P2：老实例有行 (i, T_pub)，「码 2 / 3 ⇒ b ≤ T_pub」，`.claude/kb/decisions/18-块里携带什么信息.md:315`；诞生代号 > T_pub 判未发布），**再**对每个退役容器身份取 P3 最大的那一版，**只读那一版里的记录**；不许把各版的记录并起来。

**为什么顺序承重**：新实例会重发同一个 txg：「崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上」（`.claude/kb/decisions/08-核心索引结构.md:207`）；被抛弃段里的死亡 txg 于是正好落在新父链上（「其余 txg 的父是 txg − 1」，`.claude/kb/invariants.md:306`），只靠「按死亡 txg 滤谱系」滤不掉它。**模型里**读者 E（= B，但退役记录不先按已发布谓词滤版本、各版并起来）：

| 历史形（borrow 政策，各 300 段 × 60 步） | 读者 B 误判死亡 | 读者 E 误判死亡 |
|---|---|---|
| 有回退有崩溃（崩溃 905 次：txg 重发 591、跳号 314） | 0 | 300 |
| 无回退有崩溃（崩溃 880 次） | 0 | 742 |
| 有回退无崩溃 | 0 | 0 |
| 无回退无崩溃 | 0 | 0 |

E 只在有崩溃的格里误判，正是「被抛弃段里死掉、所选根里还活着」那一形；B 的顺序（P2 → P3 → 读那一版 → 死亡 txg 滤谱系）在这些格里全 0。drop、absorb 两种政策各格同形（`expected/summary.txt`）。

### K3.4　判定与推翻条件

- **一致**：随现行那一版、回退不取 R_old、容器级只按 P2 择版本（18-块里携带什么信息.md 第 301 行、C760）；四条纪律照搬。**规则没说**：读者先 P2 再 P3 再读记录的顺序、追加接口一次多条。**冲突**：无。
- 推翻「照抄 cur 对」：任一合法历史里，回退之后某个活叶被退役记录判死。模型 12 格里读者 B 为 0；C760 描述的另一种形（谱系链上那一版的槽被环再分配、链上的退役记录读不到）模型没建（模型没有分配器与环），`crates/` 里也没有读侧，这一形仍开着，不由本节关。
- 推翻「P2 先于读记录」：模型里读者 E 在有崩溃的格里不误判，或读者 B 在有崩溃的格里误判。

## K4　准入与承诺量

### K4.1　一次发布至多写几条退役记录：r(P) = 集合差的大小

| 发布种类 | r(P) | 依据 |
|---|---|---|
| 意图、批、建 inode、覆盖写、顺序写（`write_records_into_leaf_containers`） | 0 | 分裂是新建，不退役；`InodeLeafContainersAfterThisPublish` 里「这次删空、不再落盘的片数（只有 [`remove_record_from_leaf_containers`] 删空一片时是 1，别的落法都是 0）：」（`crates/singlefs-core/src/inode_tree.rs:123`） |
| 最后一批（删 inode 记录） | ≤ 1 | 「最后一批释放剩下全部单元、删 inode 记录、删 deleted_inodes 那条 key，同一次发布」（`.claude/kb/decisions/08-核心索引结构.md:473`）；代码断言「最后一批只删一条记录，不与别的记录写同一次发布：删除的入口不带文件版本、不建 inode」（`crates/singlefs-core/src/transaction.rs:5117`）；一条记录删掉至多触发一个合并动作（删空 / 吸收 / 借三选一，借不退役）⇒ 集合差 ≤ 1 |
| 管理员回退、空发布、写行、暖机、抬 F | 0 | 不改 inode 树的叶容器集合（回退取 R_old 的集合，不写退役，K6） |
| 以后的克隆头首次 COW | ≤ 该头在这次发布里首次 COW 的共享叶数 ≤ 135 | 今天没有产生者；一个根装得下 135 片叶：`leaf_containers_one_root_node_holds()`（`crates/singlefs-core/src/inode_tree.rs:109`） |

**式子**：r(P) = |ids(上一版) − ids(这一版)|；今天 r ≤ 1。若 M3 的写路径允许一次删除连着做几次合并，上界退成 r ≤ N_叶 − 1 ≤ 134（N_叶 ≤ 135）。**对 M3 写路径的要求（写进 K6 的接口）**：一次删除至多一个合并动作，否则 r 的上界要换。模型里单次发布最大退役记录数 1（12 格全是），单次发布重写的退役容器数最大 1（`max_retire_containers_rewritten_in_one_publish=1`，12 格）。

**退役容器重写数 c(P)**：r = 0 时 0；0 < r ≤ 959 时，开放容器装得下就是 1，装不下（先填满开放容器、余下的开新容器）至多 2 ⇒ c(P) ≤ 1（r = 1）、≤ 2（1 < r ≤ 959）。单位：一个容器 32 KiB = 2 槽（16 KiB），根 1 槽，根之下的内部节点在最右那条路径上 H_r 个（H_r = 退役树根节点码 2 头里现读的层级 − 1，零根与只有根时 0）。一次发布新写的退役单元 ≤ 2c + 1 + H_r + G，G = 这次根满长高多出的右半节点（0 或 1）；r = 1 时 ≤ **4 + H_r** 槽。

### K4.2　归哪一格：普通分配（同墓碑），不进 ckpt_cost

`space_budget_of_role`（`crates/singlefs-core/src/admission.rs:1230`）把重写的角色分三格。墓碑三角色在「普通分配」格：「删除那两棵树的节点只在删除类发布里重写：落点那一判照判（普通分配），式子那一判删除类发布整次不判」（`crates/singlefs-core/src/admission.rs:1238`）；谱系三角色在「checkpoint 保留池」格，因为它只在管理员回退那次写，而回退「回退不判式子，空间从 checkpoint 保留池里出」（`crates/singlefs-core/src/admission.rs:1245`）。

退役记录只在最后一批写，那次发布重写 inode 根（普通分配），所以它的空间走墓碑的路：落点那一判照判，式子那一判删除类发布不判（「删除类发布（意图、各批、挂载里的续做批）不判式子那一判，落点那一判照判」，`.claude/kb/decisions/03-空间分配.md:52`）。放进 ckpt_cost 买不到保证：保留池是给 checkpoint 固定点的一道地板，「保留池是给 checkpoint 开的一道地板，对普通分配是纯税」（`.claude/kb/decisions/23-journal的角色与格式.md:629`），退役单元是普通分配落点，从保留池之外的空闲里出，把它们的 3 + H_r 块与中央映射删插挂进 ckpt_cost 只是每次发布白扣。

**⇒ D28 已定项 4 原文不改**（ckpt_cost 式子、「谱系容器单列一项」都不动）。`space_budget_of_role` 加三个退役变体归 `OrdinaryAllocation`（`admission.rs:1238-1243` 墓碑那三行旁），编译器的穷尽 match 强制。

### K4.3　要改的两个式子（动 D3 / D28 原文，写回前交用户）

**(a) 标签 6（D3 已定项 2）**。现行：「精确式子：标签 6 = 副本数 × ((2 + H + 1) × 16384 + 41 × n_live)」（`.claude/kb/decisions/03-空间分配.md:52`）。应为：

> 标签 6 = 副本数 × ( (2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录 )

35 = ⌈2 × 16384 ÷ 959⌉（墓碑那 41 = ⌈2 × 16384 ÷ 815⌉ 同式，这次真算：⌈32768 ÷ 959⌉ = 35）；H_退役从退役树根节点码 2 头现读，零根与只有根时 0。**为什么退役那一项数 n_记录（inode 树里全部记录，含 nlink = 0 的）而不数 n_live**：墓碑写在意图那次，n_live 在同一次发布里减一，预付恰好转成真写入；退役记录写在最后一批，意图到最后一批之间那个对象 n_live 已不含它，若退役份额也挂 n_live，这份预付在用到之前就释放了。n_记录 在最后一批删掉那条记录的同一次发布里减一，与退役记录同发布，对得上。35 B/记录是上界（一条记录的删除至多引出一条退役，实际每片叶 233 条才退役一次），保守约两个数量级，不量。

落点：`admission.rs:236-300`（常量与 `TombstonePrepayBasis`、`committed_reservation_for_tombstones`：基底加 `retirement_tree_internal_levels` 与 `inode_records`）；`admission.rs:225` 旁加 `inode_records_of_the_inode_tree`；`transaction.rs:4269`（`committed_reservation_for_tombstones_after_this_publish`）与新的 `retirement_tree_internal_levels_read_from_the_root_node_header`（仿 `transaction.rs:4294`）；两个调用点 `transaction.rs:6509`、`:8053`；测试 `admission.rs:1742-1790` 钉绝对值。

**(b) 环窗口 W（D28 已定项 1）**。现行：「W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）。应为 **W_新 = 24 + (4 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))**：多出的 4 + H_退役 = K4.1 末的新写退役单元上界（r = 1）。落点：`admission.rs:298`（常量旁加退役项）与 `admission.rs:311` 的 `slots_one_deletion_publish_newly_takes_on_one_device_at_most` 加参数 `retirement_tree_internal_levels`；调用点 `admission.rs:390`（`ring_window_on_one_device`）；钉值测试 `crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:76`。窗口 Ω = (3S + 1) × W，S = 8 时 25 × (4 + 0) = 100 槽 = 1 638 400 字节每盘（S = 4：13 × 4 = 52 槽）。中央映射树对这几个单元的删插（每单元一把 key）不在 W 的「映射树根以下 4」里重量过：**没量，推的**，要主 agent 在入库装置上量（`unlink_writes_per_transaction_do_not_grow_with_the_file.rs` 加退役写之后看实测最大一批 ≤ W_新）。

### K4.4　同一次发布里已经存在、退役记录不引入但会放大的一格（观察，不属本题）

D28 已定项 1 给 W 的前提写明「这个上界在删除批只改尾叶时成立（D8（核心索引结构） 已定项 16）」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）。最后一批删空一片叶时，`remove_record_from_leaf_containers` 把它右边的每一片位置左移、「挪了位置的那几片这次全部重写」（`crates/singlefs-core/src/inode_tree.rs:278`，代码 `let rewritten = (index.position()..containers.len())`，`inode_tree.rs:309`）。一个根装得下 135 片叶 ⇒ 最左一片删空时重写至多 134 片 × 2 槽 = 268 槽每盘，而 W 里「普通分配」那一项量到的只有「量到的普通分配只有 3 到 5 槽」（`research/prompts/m3-delete-r3-sonnet-output.md:94`）。268 是算的、没量；M3 的左吸收右同样把右邻之后全部左移。两条出路：① 写路径不重写只是换了位置的叶（角色按身份而非位置排，内部节点条目前移即可），W_新 不变；② 保留重写，W_新 加 2 × (N_叶 − 1)，窗口 S = 8 时多 25 × 268 = 6 700 槽 ≈ 105 MiB 每盘。这一格归 M3 的写路径设计，本腿只标出来，给攻方扫。

### K4.5　判定与推翻条件

- 与现行条款逐格：墓碑走「解开自己」的预付、删除类发布不判式子 = **一致**（03-空间分配.md 第 52 行、D3 已定项 17「墓碑树根满时长高（D8（核心索引结构） 已定项 6），不因结构拒删除」，`.claude/kb/decisions/03-空间分配.md:318`）；退役项的两个式子 = **冲突**（03-空间分配.md 第 52 行 与 28-挂载期承诺量.md 第 29 行 两行少一项）；D28 已定项 4 = 不改；ckpt_cost 是否含退役 = **规则没说**，本节定为不含。
- 「释放空间的操作不能因写退役记录被准入拒」：成立的推理是——退役单元在删除类发布里、式子那一判整次不判，落点那一判只拒物理空槽，而窗口 W_新 与标签 6 已为这些槽预留。**这条推理没有量**：量它要在入库装置上跑 E168 一类的满池删除（重型，归主 agent）。推翻：满池删除里最后一批被落点那一判拒，且拒因是退役单元的槽。
- 推翻「n_记录 而非 n_live」：任一合法历史里意图到最后一批之间，标签 6 的退役份额小于最后一批要写的退役记录的摊分。推翻「不进 ckpt_cost」：退役记录出现在一个没有任何普通分配角色的发布里（今天没有）。

## K5　checker 判什么

### K5.1　立 5 条（编号接 I-9.29 往后；记录级的几条照 I-9.18–I-9.20 按谱系判）

| 编号（建议） | 名 | 判什么 | 在合法镜像上误报吗 |
|---|---|---|---|
| I-9.30 | 退役树条目身份与子头相符 | 退役树每个内部条目的类型段 ∈ {0, 3}：0 ⇒ 子单元是码 2（根长高之后），身份引用的出生树、容器号、出生代三段为 0；3 ⇒ 子单元偏移 6 = 3，子头的（出生树、打包记录类型、容器号、容器出生代）与条目的身份引用四段逐字相等，子指针头部出生树 == 身份引用出生树，且子容器打包记录类型 == 3。孩子头里的类与类型段不一致时怎么往下认，逐字照 I-9.17 / I-9.23（同一段话，类型段的值换成 3）。现行 I-9.17 写「类型段 ∈ {0, 5}」（`.claude/kb/invariants.md:305`），I-9.23 写 {0, 1} | 不会：与已实现的两条同形，实现是 `walk_container_tree_root`（`crates/singlefs-checker/src/walk.rs:2397`）再多一次调用 |
| I-9.31 | 退役记录合法 | 退役树里**每条**记录（不论在不在谱系上）：被退役身份的打包记录类型 == 2、出生树 ≠ 0；容器出生代 < 死亡 txg ≤ 这一版的 txg；每个退役容器记录数 ≤ 959。记录宽 34 由 I-1.7 的登记宽判 | 不会：死亡 txg 来自发布 txg，回退版的 cur 记录 ≤ T_c < T_rb；身份只在「上一版有」时才被退役，所以出生代严格小于死亡 txg（比 I-9.20 对墓碑的 ≤ 严一格：退役的身份必须在上一版里，出生早于死亡发布）。与 I-9.20 同句式：「每条记录的对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」（`.claude/kb/invariants.md:308`） |
| I-9.32 | 谱系上的退役记录按身份不重复 | 退役树里死亡 txg 落在这一版父链上的记录，按（出生树、类型、容器号、出生代）不重复；只判在谱系上的 | 不会：回退后同一身份可再被摘一次，两条里只有一条在父链上；缺口里那条被滤掉 |
| I-9.33 | 谱系上的退役记录不指本版活叶 | 在谱系上的退役记录指认的身份，不出现在同一版、头 ID 配得上的那棵 inode 树的叶容器身份集合里（读 inode 树内部节点条目的身份引用即可，不开叶） | 不会：借时右片保留身份、不退役；回退版的缺口记录死亡 txg 不在父链上；跨回退后又活的身份只有缺口里那条记录 |
| I-9.34 | 退役完备 | 相邻两版（本版 txg 恰是某条谱系记录的 T_rb 时父版 = 那条记录的 T_old，否则父版 = txg − 1；父版在候选集里读得出才判）：父版叶身份集 − 本版叶身份集 = 本版谱系上、死亡 txg == 本版 txg 的退役记录的身份集（两个方向各报：漏记 `d_missing`、多记 `d_extra`）。最新根的实例表读不出、滤不掉被崩溃恢复抛弃的根时记判不了（照 I-9.29 的门，`walk.rs:10380` 一带的 `FilteredByTheNewestRootsInstanceTable`） | 不会（见下）：回退版的父版是 R_old 而不是 cur，两边叶身份集相等；崩溃跳号时父版 txg − 1 不在候选集里，不判 |

引文：照 I-9.18 写的理由「不按谱系过滤的写法放过改坏」（`.claude/kb/invariants.md:306`）——前三条里凡涉及记录的都只判在谱系上的。

### K5.2　这 5 条各自抓什么，漏了哪条会放过什么（模型）

模型 `m3-retire-r1-sonnet-model`：合法历史 12 格（3 合并政策 × 有无回退 × 有无崩溃）共判 219 462 个版本，a b c d 全部 0 次红。植入五种写者错误（borrow 政策、有回退有崩溃，每种 18 276 个版本）：

| 植入的写者错误 | a | b | c | d 漏记 | d 多记 | 红的版本 | 读者 B 复活 / 误判死亡 |
|---|---|---|---|---|---|---|---|
| 删空的忘了写 | 0 | 0 | 0 | 427 | 0 | 427 | 9456 / 0 |
| 被吸收的忘了写 | 0 | 0 | 0 | 806 | 0 | 806 | 12890 / 0 |
| 借时给右片也写了 | 0 | 2029 | 2705 | 0 | 319 | 4106 | 0 / 1420 |
| 死亡 txg 少一 | 3375 | 588 | 760 | 1233 | 0 | 4985 | 0 / 243 |
| 同一条写两次 | 0 | 11583 | 0 | 0 | 0 | 11583 | 0 / 0 |

读：**漏写（两行）只有 I-9.34 抓得到**，a b c 全 0；而漏写正是 E177 的复活形态（读者复活 9456、12890 次）。所以 I-9.34 不能省，也是风险最高的一条（它是跨版本的、依赖「叶身份集合只经退役记录消失」这条写者纪律；任何以后新增的、不经集合差的叶身份消失动作都会让它在合法镜像上红）。同一条写两次读者不受影响，只有 I-9.32 红——它抓的是写者 bug，不是读侧后果。

### K5.3　GPU 判器：同判什么、回落什么

- **必须同判（不是新判据，是树表种类变多）**：`crash_judge_common.wgsl:139` 的 `TREE_KINDS_OF_THE_TREE_TABLE` 11 → 12；`crash_judge_checker.wgsl:1861` 头 ID 分支并进种类 13；`:2967`、`:3044-3050` 的 schema 与容器树分支加种类 13（容器记录类型 `PACKED_TYPE_RETIREMENT` 取 3，`:1602`、`:3192` 的记录类型分派、`crash_judge_checker_common.wgsl:189` 的常量）；I-9.30 由走读内核的容器树分支同判。CPU 一侧 `walk.rs:2319` 那一块旁加退役树的 `walk_container_tree_root` 调用，`image.rs:260-275` 一带加常量，`IMPLEMENTED_INVARIANTS` 84 → 89（GPU 违例位 128 位，`.claude/kb/invariants.md:16`，装得下）。
- **首批不写内核的**：I-9.31–I-9.34 四条。做法照先例「GPU 判器走读内核 inode 树只接类型段 2 的层级 1 条目：类型段 0 而有一份按码 2 自证过的条目落判不了（`UNDECIDABLE_TOO_DEEP`），交回 CPU 判」（`.claude/kb/invariants.md:290`）：状态里退役树有 ≥ 1 条记录就置判不了（新理由码），交 CPU 重判；覆盖表照编译期断言列这四个下标。代价：含退役记录的崩溃状态全回落 CPU，层 0 的 GPU 加速对这些状态失效，占比没量。另一条路是同判（写四个内核函数、合并调用点，`crash_judge_dispatch.rs:1212` 的 `SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE` 表再加）——工作量大，先记欠账。
- 草稿区重排：`C_MAX_TREES_PER_TABLE` 11 → 12 让 `C_CANDIDATE_WORDS` 50 → 54，48 行候选根 × 4 字 = 每状态多 192 字，其后 31 个偏移常量各 +192，`SCRATCH_WORDS` 315_376 → 315_568（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:134`）。不做的话，每张 12 条的树表都是「第 12 棵起判不了、交 CPU」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:405`），GPU 在新池新建文件之后的每个状态上全退回 CPU。

### K5.4　判定与推翻条件

- 与现行：条目身份那条、按谱系判记录那几条的写法 = **一致**（I-9.17、I-9.23、I-9.18–I-9.20 的句式）；退役记录本身的判据 = **规则没说**（`.claude/kb/decisions/08-核心索引结构.md:203` 只写「退役记录带死亡 txg、按谱系滤」读法，没有 checker 判据）；`.claude/kb/invariants.md:304` 的「十一种」、`:315` 的头 ID 组、`:32` 的登记宽 = **冲突**（K1、K2 已列）。
- 推翻 I-9.31–I-9.33 不误报：任一合法镜像（回退、崩溃恢复、借之后）上它们红；模型 12 格 0，但模型没有克隆头、没有环再分配（C760）、没有一次删除连做几次合并。推翻 I-9.34：合法镜像上出现「叶身份集合变小而没有退役记录」的发布——除管理员回退外 `crates/` 里没有（`remove_record_from_leaf_containers` 是唯一的删除落法，`inode_tree.rs:17` 的文件头注释把「别的合并（左从右借、两片都不空的左吸收右）」写作没写）。M3 写路径一旦有「不经集合差」的动作，这条先红。

## K6　与 M3、读侧的接口

### K6.1　身份消失的形态清单：「借不写」这一句漏了什么

| 形 | `crates/` 今天谁产生 | 写退役记录 | 集合差里有吗 |
|---|---|---|---|
| 删空摘掉 | `remove_record_from_leaf_containers`（`crates/singlefs-core/src/inode_tree.rs:286`） | 是 | 有 |
| 合并时被吸收的右半 | 没有：「别的合并（左从右借、两片都不空的左吸收右）」在文件头注释里写作没写（`crates/singlefs-core/src/inode_tree.rs:17`），M3 新写 | 是 | 有 |
| 左从右借 | 没有，M3 新写 | 否（右片保留身份、分隔 key 改成它这一版的最小 key） | 无，自然不写 |
| 左半分裂 | `write_records_into_leaf_containers` | 否（左半身份不变） | 无 |
| 码 2 内部节点消失或长高 | M3 / 根满长高 | 否：「内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份」（`.claude/kb/decisions/08-核心索引结构.md:171`） | 无（集合只数码 3 叶容器） |
| 跨头首次 COW 重生的旧身份 | 没有（`grep -rn -i 'clone_head\|克隆头' crates --include=*.rs` 0 行） | 是 | 有（该头的上一版有、这一版没有） |
| 管理员回退让现行独有的叶身份消失 | `version_to_publish_the_rollback_after`（`crates/singlefs-core/src/mount.rs:7770` 一带） | 否 | 父版是 R_old 不是 cur，集合差为空 |
| 崩溃恢复抛弃一段 | 恢复 | 否（靠 P2 实例表滤） | 不是一次发布 |

**结论**：对「`crates/` 今天或 M3 有写者」的形态，「借不写，吸收与删空写」这一句是完整的，因为写的规则不是逐动作列举而是集合差（K2.3）。两处要补：① **一次删除至多一个合并动作**，写进 M3 的写路径要求，否则 K4 的 r ≤ 1 要换成 r ≤ 134；② 管理员回退造成的「现行独有的叶身份没有退役记录就消失」是一个真实的形态——不是 M3 的漏，是 X1 甲定案本来的取舍，下面 K6.2 量它。

### K6.2　回退那次不写退役记录，读侧靠什么挡住回退缺口里出生的叶

**模型里**：1187 次回退共 1886 个现行独有的叶身份没有退役记录就消失（单次最大 9；borrow 政策、有回退有崩溃 300 段）。读者 A（kb 字面：已发布谓词 + 退役记录按死亡 txg 滤谱系）在回退之后的扫描里复活这些叶：12 124 / 18 276 个扫描有错（复活 54 179 个叶、内容错 9 975 个），读者 B（A 再加一条「候选单元的诞生 txg 在被扫那一版的父链上」）0。备选 α（回退那次给现行独有的叶身份补退役记录）只救一半：读者 A 复活降到 19 418、内容错仍 9 975——缺口里重写过的叶身份在 R_old 里是活的，不会被 α 退役，读者仍取到缺口里那一版。**所以挡缺口的是读者的候选谱系滤，不是回退时多写记录**；X1 甲不在回退写记录是对的。

这一条在 E178 的读者里有（候选一条：读得出、已发布、「诞生 txg 在被扫那一版的父链上且不大于它的 txg」，`research/prompts/e178-preregistration.md:472`），**在 kb 条款里没有**：D8 已定项 6「重建」只写「退役记录带死亡 txg、按谱系滤」（`.claude/kb/decisions/08-核心索引结构.md:203`），C615 写明读侧「读侧怎么用没有条款」（`.claude/kb/checks-owed.md:475`）。⇒ **规则没说**，且按 kb 字面读出来的读者就是模型里的 A。

### K6.3　读侧放这一批，还是记欠账

**建议**：这一批做写侧（K2–K4）与 checker 的树走读方向（K5 五条，它们就是今天 `crates/` 里唯一读退役记录的地方）；**扫描重建读侧记欠账**，不在这一批写代码。理由：`crates/` 里没有扫描重建——`scanned_content_units`（`crates/singlefs-checker/src/walk.rs:7385`）的输出只喂 I-1.8（`judge_merged_version_total_order(reader, &content_units, …)`，`walk.rs:11513`），不重建任何东西；给它加「认这一组没了」没有消费者，也没有真值可对（真值要重建视图，今天没有）。先例：谱系记录同样「写侧随谱系落地，读侧记欠账」（`.claude/kb/checks-owed.md:475`）。

**欠账要连着记的三句**（模型各有一格支撑，写回 C615 / C760 一带，编号由主 agent 定）：

1. 读退役容器：先按实例表已发布谓词滤版本（P2），再每个身份取 P3 最大的一版，**只读那一版的记录**，不并集各版——读者 E（并集）有崩溃的格里误判死亡 300 / 742 次（K3.3）。
2. 记录生效：死亡 txg 落在这一版父链上，且被杀的单元版本诞生 txg ≤ 死亡 txg——读者 C（不看谱系）有回退的格里误判死亡 1438 / 1586 次（K3.2）。
3. **候选步先滤诞生 txg 不在父链上的单元**（C615）——否则读者 A，K6.2。

### K6.4　接口（给 M3 的写路径）

- `InodeLeafContainersAfterThisPublish` 加 `retired: Vec<PackedIdentity>`，由 `retired_identities(before, after)` 算（K2.3）；M3 的合并 / 借函数只管返回新的叶容器集合，不自己写退役记录。
- 一次删除至多一个合并动作（删空 / 吸收 / 借三选一）；删空优先于合并（`|C| = 0` 不落盘，不与右邻吸收）。
- 退役树追加一次多条：`append_records_to_the_open_container`（K3.1）。
- 位置左移重写右邻片：见 K4.4，M3 要么不重写，要么 W_新 加 2 × (N_叶 − 1)。

### K6.5　判定与推翻条件

- 与现行：「吸收与删空写、借不写、借时分隔 key 取孩子这一版的最小 key」= **一致**（`.claude/kb/decisions/08-核心索引结构.md:171` 的「被吸收的右半走 COW 释放路径，同一次发布给它的身份写一条退役记录」与「删空摘掉的容器同一次发布给它的身份写一条退役记录」）；读侧候选谱系滤 = **规则没说**；08-核心索引结构.md 第 203 行 的读法与 E178 读者（候选一条）并排 = **冲突**（前者少一条）。
- 推翻「读侧记欠账」：用户要这一批就能做扫盘重建（那就要把 K6.3 三句写成代码与会红的检查）；或 M3 的写路径里出现不经集合差的叶身份消失（则 K6.1 的清单要加一行）。
- 推翻 K6.2 的「候选谱系滤必需」：入库装置（E178 一类）里 A 读者在回退之后不复活。模型里 A 复活是回退缺口里出生的叶、没有退役记录挡它，E178 的读者本来就带这一条，所以两边不矛盾，矛盾只在 kb 字面。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 种类码 / 树 ID / 树表次序 | 一致（追加）；字面值冲突 | 13、65548、排最后；D8 登记表、18-块里携带什么信息.md 第 580 行、I-9.16 ④、layout 的 11 / 65548 要改成 12 / 65549 |
| K1 头 ID | 规则没说 | 取 inode 树的树 ID（同墓碑树），配 I-9.33 的配对；全池一份则每条记录要加头 ID |
| K1 day-1 vs 首次建树 | 规则没说（先例一致） | day-1：16 个文件 57 行 65548 字面量、GPU 草稿区 +192 字；惰性省不掉 GPU 那项，还多一条建树路径 |
| K1 incompat 位 | 一致 | 不加，并进第一条布局线 |
| K2 字段表 | 规则没说 | 34 字节：身份引用 26 + 死亡 txg 8，959 条一容器，不加头 ID / flags / 预留 |
| K2 死亡 txg 与退役集合 | 规则没说 | 集合差，死亡 txg = 这次发布的 txg；分裂 → 合并 → 再分裂不写从没落盘的身份 |
| K2 同身份退役两次 | 规则没说 | 一条时间线上不会；跨回退会，缺口里那条不在父链上 |
| K3 四条纪律照搬 | 一致 | 全部成立；只有追加接口要改成一次多条 |
| K3 管理员回退 | 一致 | 照抄 cur；R_old 之后才死的容器靠记录死亡 txg 不在父链上判活（模型：不看谱系的读者误判死亡 1438 / 1586，看的 0） |
| K3 崩溃恢复 | 规则没说 | 先 P2 再 P3 再读那一版的记录；并集各版的读者在有崩溃的格里误判死亡 300 / 742，顺序对的 0 |
| K4 一次发布至多几条 | 规则没说 | r = 集合差大小，今天 ≤ 1，要求 M3 一次删除至多一个合并动作；否则 ≤ 134 |
| K4 ckpt_cost | 规则没说 | 不进；退役角色归普通分配（同墓碑），D28 已定项 4 原文不改 |
| K4 标签 6 与 W | 冲突 | 03-空间分配.md 第 52 行 与 28-挂载期承诺量.md 第 29 行 两个式子各加一项（35 × n_记录 与 2 个 H 项；W_新 = W + 4 + H_退役）；没量映射树项 |
| K4 重写右邻片 | 冲突（已有的，非退役引入） | 28-挂载期承诺量.md 第 29 行「只改尾叶」前提 vs `inode_tree.rs:309`：至多 268 槽每盘；归 M3 写路径，给攻方扫 |
| K5 立哪几条 | 规则没说 | I-9.30–I-9.34 五条；漏写只有 I-9.34 抓得到 |
| K5 误报 | 一致（模型内） | 合法 12 格 219 462 个版本 0 次红；模型无克隆头 / 环再分配 / 多合并动作 |
| K5 GPU | 规则没说 | I-9.30 同判，I-9.31–34 判不了回落 CPU；树表容量 11 → 12 的重排必须做 |
| K6 「借不写」那一句 | 一致 | 对 `crates/` 与 M3 有写者的形态完整；补「一次删除至多一个合并动作」 |
| K6 读侧 | 规则没说 / 冲突 | 扫描重建读侧记欠账（`crates/` 无消费者）；欠账连记三句，其中候选谱系滤 kb 字面没有（08-核心索引结构.md 第 203 行 vs E178 读者），按字面读出来的读者回退后复活 |

## 没做什么

- 没读禁读清单（`research/prompts/m3-retire-r1-opus-output.md`、`m3-retire-r1-opus-model/`）；没读背景材料附录之外的别腿产出。
- 没跑 cargo、没跑重型测试、没编译；没改 `crates/` 与 kb，没碰冻结树。`crates/` 的落点都是读代码得来的，没有一处落点被编译验证过。
- 没量：K4.3 的映射树项（W_新 的中央映射删插）、K4.4 的 268 槽、n_记录 预付的实际保守倍数、挂载读退役树的读量、K5.3 里含退役记录的崩溃状态占比（GPU 回落 CPU 的面积）。这些在判定一览里标了「没量」或「推的」。
- 没建模：克隆头、C760（谱系链上被环再分配的那一版）、一次删除连做几次合并、分配器与准入。模型的数不进 kb；要引，先在入库装置（E178 一类）上重做（`.claude/rules/implementation-first.md` 第 4 条、`three-way-inference.md`「一条腿在副本装置上量出来的数……」）。
- 不判别的腿的格（攻方的 K2–K5 反例）不替主 agent 采纳或出判决；没替主 agent 判「改 D8 / D18 / D3 / D28 原文」，这些写回之前交用户（本报告里标「冲突」的行就是要改的原文）。
- 没写 kb 欠账编号：K6.3 的三句欠账由主 agent 登记编号。

## 要主 agent 复核的命令（都是轻量，纯 Python）

```
cp -r research/prompts/m3-retire-r1-sonnet-model <草稿目录>/copy
SINGLEFS_REPO=<仓根> bash research/scripts/capped.sh 10 bash <草稿目录>/copy/rerun.sh <空目录> all
```

跑完看 `compare：与 expected/summary.txt 逐行相同（排序后）`；报告引用的数都在 `<空目录>/out.txt` 的 `group=legal`（`policy=borrow` 四格）、`group=fault`、`group=alpha_rollback_retires` 三类行里，字段名与报告一一对应（`misdeath_A..E`、`resurrect_A..E`、`wrong_content_A..E`、`inv_red_*`、`rollback_cur_only_ids_*`、`max_retire_*`）。

## 交回前自检（这一次真跑的末行）

```
$ python3 research/scripts/cite-check.py research/prompts/m3-retire-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-retire-r1/tree --root /home/fy5090/code/singlefs
  ✓ 核了 49 处引文，对上 49 处，没判 0 处（逐处列在上面）
$ SINGLEFS_REPO=/home/fy5090/code/singlefs bash research/scripts/capped.sh 10 bash research/prompts/m3-retire-r1-sonnet-model/rerun.sh <空目录> all
compare：与 expected/summary.txt 逐行相同（排序后）
```

草稿与中间产物（不入库，仓外）：`~/.local/share/singlefs/work/m3-retire-r1-sonnet/`（`parts/` 报告分段稿、`report_full.md`、`model.py` 草稿、`run1`–`run3` 模型三次产物、`lq.py` 查行号的小脚本）；没有编译目录、没有仓副本、没有工作树，无需删除。
