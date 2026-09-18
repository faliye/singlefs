# E142 第十一次跑 · 重跑登记：第一个事务的改动计数从 1 改成 3，盘上还有哪些字节跟着变

写于 2026-09-18 08:57 JST（本机 UTC 2026-09-17 23:57），装置改动之前。
重跑对象：E142（第一个事务的干跑）；次数按实验页 `.claude/kb/experiments/142-第一个事务的干跑.md` 的「历史版本」最末一节（2026-09-17（第十次跑））+ 1 = **第十一次**。
原登记 `research/prompts/e142-preregistration.md`（判据怎么定的，读过）；本轮不占新实验号，不跑 `claim-experiment.sh`。

## 一、问题

主 agent 给的问题单（不是为岔路建的实验，一行一个要回答的判断），逐行抄：

| # | 问题 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 1 | 第一个事务的 inode 记录「改动计数」字段从 1 改成 3 之后，盘上还有哪些字节跟着变 | 干跑算出的字节与 `crates/` 实装写出的逐字节比对，出现任何一处不等 | 偏移 88 那 8 字节、inode 叶载荷 CRC 与头校验和、指向 inode 叶的指针里的单元校验和、inode 树根、树表单元、根记录校验和六处逐一给出新值，且 `name=width`、`name=segments` 两类结果行与上一次跑逐字相同 |

**读法写死**（装置照这几句写，不许在跑之后改）：

1. 「盘上还有哪些字节跟着变」= 在同一次进程里把整条路（mkfs → 取号 → 暖机两次 → 第一个事务）跑两遍，
   一遍 inode 记录偏移 88 写 1、一遍写 3，其余输入一字不差（同 fsid `FIXED_FSID`、同写入时刻 `FIXED_WRITE_TIME_SECONDS`、同 3000 字节内容、同 2 盘 4 GiB 稀疏镜像），
   把两遍**落盘后的整份镜像逐字节相减**，报出每一段连续差异的六元组：(设备, 盘上绝对偏移, 长度, 旧字节串, 新字节串, 所属结构 + 结构内偏移)。
   **分母是两块盘的全部字节（2 × 4 GiB），不是只看那 8 个单元**：只比点名的几个槽，「还有哪些字节跟着变」就问不出来了。
2. 「`crates/` 实装写出的」= `crates/singlefs-harness/src/scenario.rs` 的 `run_first_transaction` 在内存盘上跑同一条路（同参数）之后的镜像。
3. 「逐字节比对」= 装置镜像与实装镜像在**区域清单**上按字节全等。区域清单写死为：第一个事务写到的 8 个单元的落点（两盘各一份）、jsn 3 那条 journal 记录的两个落点、这次发布的根槽、这次发布写的超级块槽。
   清单之外对不上不作数：装置与实装是两份代码（`.claude/rules/implementation-first.md` 第 4 条），清单外的差异不是这一行问题问的事，按第十一节的停机条款只记不判。
4. 「值」一律报成小端十六进制字节串，不报十进制，免得「8 字节字段」与「变了几个字节」混成一句。

这一行问题**只有一种读法**：改动计数这一格是定长字段（D8 已定项 6 字段表偏移 88 宽 8），改值不改宽，所以「哪些字节跟着变」= 差异段清单，没有第二种问法。

## 二、被测条款与它引的定义

**出处 `.claude/kb/decisions/08-核心索引结构.md:391-410`（整段抄，未转述）**

```markdown
**记录字段表（定长 140，小端；对齐口径是记录内偏移，容器内第 n 条起点 135 + 140n（码 3 头 107 + nonce / MAC 预留位 28，D18（块里携带什么信息） 已定项 14）不按 8 对齐，按非对齐读）**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| 偏移 | 字段 | 宽 |
|---|---|---|
| 0 | inode | 8 |
| 8 | 对象出生代（创建那次发布的 checkpoint_txg；inode 号不复用后它不再承担复用判别，按五元组 / AAD 的 day-1 契约保留，扫描时与该对象每个数据单元头逐字段对照） | 8 |
| 16 | locality_id（第一版无父目录取 0） | 8 |
| 24 | mode / uid / gid / nlink，各 4 | 16 |
| 40 | size / blocks / rdev，各 8 | 24 |
| 64 | atime / mtime / ctime 秒，各 8 | 24 |
| 88 | 改动计数（最后一次改动所在发布的 checkpoint_txg；提交锚 = 叶容器头的诞生代号；同一发布窗口内多次改动只有最后一版落盘——内部节点不缓冲，E98（inode 记录与 inode 树的几何） 判据 4 的答复） | 8 |
| 96 | atime / mtime / ctime 纳秒，各 4 | 12 |
| 108 | 填充（恒 0） | 4 |
| 112 | flags（恒 0） | 8 |
| 120 | 预留（恒 0） | 20 |

不放 extent 指针（D14（双轨（大小文件 / 持久临时）） 已定项 3 内联阈值 0），btime 不进记录。
**flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ 该记录 EIO（对象级，不拒整个容器）**——前向兼容政策不是损坏检出（载荷校验和照样过），落成 I-9.7（记录三段恒零），在 I-1.7（打包容器合法与判定顺序） 的中止链之外。
```

**出处 `.claude/kb/decisions/08-核心索引结构.md:429-430`（整段抄，未转述）**

```markdown
**第一个事务写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 1、指针）；一个码 3 叶（类型 2、容器号 1、出生代 1、记录数 1、记录宽 140）；
一条记录（inode 1、出生代 1、locality 0、nlink 1、改动计数 1、三段 0）；记账里一条「inode 号水位 = 2」。
```

**出处 `.claude/kb/layout/01-first-txn.md:245-245`（整段抄，未转述）**

```markdown
| inode 树根（码 2） | 单元类型标签恒 = 2；类身份段 = key 宽 1 + 树 ID + 层级 + key 区间 [1, 1] + 诞生代号 + fsid + 写序 4（只实例代号）+ 出生序号 4 + 载荷 CRC 4 + 预留 2 + 条目数 2 + 条目宽 2（偏移表见 D18（块里携带什么信息） 已定项 18） | 头 **131** = 86 + 2 × 8 + 29（明文头末尾 102，头校验和到 101；含预留位 29 的头 131，条目区从 131 起） | 树 ID 12、层级 1、区间 [1, 1]、诞生代号 3、写序 1、key 宽 8、条目数 1、条目宽 120 | D8（核心索引结构） 已定项 6（根恒码 2、带 key 区间）/ 已定项 11；D18（块里携带什么信息） 已定项 7 / 已定项 12 / 已定项 16 / 已定项 18 | 已定 |
```

**出处 `.claude/kb/layout/01-first-txn.md:249-249`（整段抄，未转述）**

```markdown
| inode 叶容器头 | 类身份段（标签 3、出生树、类型 = 2、容器号 = 1、出生代、记录数 = 1、记录宽 = 140、诞生代号、fsid、载荷校验和、写序、出生序号）；含 29 字节预留位的头 136，记录区从 136 起 | 107 + 29 | 出生树 12、容器号 1、容器出生代 3、记录数 1、记录宽 140、诞生代号 3、写序 (1, 1)、出生序号 0（该树该 checkpoint 第一个码 2 / 码 3 单元） | D18（块里携带什么信息） 已定项 11 / 已定项 16；容器号规则与类型 2 的启用是 D8（核心索引结构） 已定项 6；序号 D19（块指针的结构与宽度预算） 已定项 9 | 已定 |
```

**出处 `.claude/kb/layout/01-first-txn.md:257-257`（整段抄，未转述）**

```markdown
| inode 记录（偏移 88） | 改动计数 | 8 | 1 | D8（核心索引结构） 已定项 6 | 已定 |
```

**出处 `.claude/kb/milestone/02-second-txn.md:320-320`（整段抄，未转述）**

```markdown
| 11 | 改动计数：第一个事务写 1，D8（核心索引结构） 已定项 6 字段表按定义该是 3；2026-09-16 核出，没立欠账 | 2026-09-18 用户定案：改代码写 3、重跑 E142（第一个事务的干跑）（字段表是定义，实现跟定义走）；同日代码已改（`publish_first_file` 写 `FIRST_TRANSACTION_TXG`，`records.rs` 与两处用例跟着改，变异「换回 1」红在「逐字段回读等于写入」） | 不等 | 还要：[layout/01-first-txn.md](../layout/01-first-txn.md) 改动计数那一格改成 3（书记员）；E142（第一个事务的干跑） 留存产物按偏移 88 那 8 字节重跑（inode 叶载荷 CRC、指针里的单元校验和、inode 根、树表单元、根记录校验和都跟着变；`name=width` 与 `name=segments` 不变）；层 0 全量重跑 | 步 1 决策点 |
```

## 三、实现今天的样子

`crates/`（2026-09-18 08:57 JST 现查，行号取自各文件本身）：

| 文件:行 | 看到的 |
|---|---|
| `crates/singlefs-format/src/lib.rs:200-201` | `/// 第一个事务的 checkpoint_txg 是 3：两次暖机空发布之后（D16（发布语义） 已定项 6 / 已定项 8）。format-const: FIRST_TRANSACTION_TXG` / `pub const FIRST_TRANSACTION_TXG: u64 = 3;` |
| `crates/singlefs-core/src/transaction.rs:1133`、`:1141` | `pub fn publish_first_file<Device: BlockDevice>(`；`let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);` |
| `crates/singlefs-core/src/transaction.rs:1168-1172` | 注释「改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88 的字段表定义）……写 1 是 2026-09-18 之前的老样子」，字段 `change_count: FIRST_TRANSACTION_TXG,` |
| `crates/singlefs-core/src/transaction.rs:1209` | 覆盖写（`publish_overwrite`）那一路写 `change_count: txg.0,` |
| `crates/singlefs-core/src/transaction.rs:1450` | 装记录时 `change_count: file.change_count,`（`FileVersionPlan` 的字段，`:1029` 声明） |
| `crates/singlefs-core/src/records.rs:30-39`、`:59-60` | `pub struct InodeRecord { … pub change_count: u64, … }`；`writer.assert_position(88, "改动计数"); writer.put_u64(self.change_count);` ⇒ 偏移 88、宽 8 由断言钉住 |
| `crates/singlefs-core/src/records.rs:380`、`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:692` | 两处用例已按 `change_count: 3` 钉住 |
| `crates/singlefs-harness/src/scenario.rs:71` | `pub fn run_first_transaction<`：mkfs → 取号 → 暖机 → `publish_first_file` 整条路，参数照 E142 装置取（同 fsid `E142_FILESYSTEM_IDENTIFIER:20`、同写入时刻 `FIXED_WRITE_TIME_SECONDS:24`、同 3000 字节 `FIRST_FILE_BYTES:26`）——本轮实装臂就从这里取字节 |

装置（`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`，本轮要改的那一行与它牵动的几处）：

| 行 | 看到的 |
|---|---|
| `:2213` | `let inode_record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: txg, size: file_bytes.len() as u64, change_count: 1 };` ⇒ **装置今天写 1** |
| `:1611`、`:1632-1633` | `change_count: u64,`；`writer.assert_position(88, "改动计数"); writer.put_u64(self.change_count);` |
| `:646-659` | `struct LocationEntry { device, slot, unit_checksum: u32 }`，宽 `LOC_ENTRY = 14`（设备 4 + 槽 6 + 校验和 4） |
| `:1856-1859` | `fn location_entries(&self, slot, unit)`：`let unit_checksum = castagnoli_crc32(unit);` 两条位置条目共用它 ⇒ 单元字节一变，指向它的每一个指针都跟着变 |
| `:835-880` | `build_packed_unit`：记录区从 `107 + 29 = 136` 起；载荷 CRC 在头内偏移 89（罩 107 到单元末尾）；`seal_header_checksum` 在偏移 10 写 32 字节头校验和 |
| `:1140` | `const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + NODE_POINTER_BYTES as usize + 8 + 8;`（= 138，第七节核过），根记录自证校验和罩整 512 槽、自身按 0 参与 |
| `:1359-1378` | `struct NamedUnit`：journal 点名项里装着**两条位置条目**，每条带 `unit_checksum` |
| `:1404-1418` | `struct JournalRecord`：头里有 `new_tree_table` / `new_mapping_root` 两个指针（D23 已定项 15 的新根段） |
| `:1520-1540` | `struct TreeTableEntry`：每条树表条目里装着该树根的指针（带两条位置条目） |
| `:1796-1804` | `build_mapping_entry`：中央映射条目 = key 27 + **两条位置条目 28** |
| `research/scripts/replay.sh:157` | `E142|e142-first-txn-dry-run||e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out|exact` ⇒ 复跑按 `exact` 与留存产物逐字比 |

⇒ 装置与实装今天在改动计数这一格上**不同**（装置 1、实装 3），这正是本轮要重跑的理由；其余各处由本轮的量 5 逐字节核。

## 四、跑之前已经存在的数

照实列（读条款、判问法时已经知道或算出的），不删：

| 来源 | 已经存在的数 / 话 | 对判据的影响 |
|---|---|---|
| `.claude/kb/milestone/02-second-txn.md:320`（第二节整段抄了） | 用户定案那一行已经写出**预期**：「inode 叶载荷 CRC、指针里的单元校验和、inode 根、树表单元、根记录校验和都跟着变；`name=width` 与 `name=segments` 不变」 | 这是**预期**不是量出来的数，与问题单的够判条件同源。⇒ 判据里**不许**把「只有这几处变」当通过条件；第六节量 4 专门收「预期之外还变了哪些结构」，它非空**不算失败** |
| `.claude/kb/milestone/02-second-txn.md:113` | 「⚠️ 第一个事务写的是 1……代码与 E142 都按 1，改它要重跑 E142，2026-09-16 核出、没改」 | 只说明旧口径的来历，不进判据 |
| `.claude/kb/decisions/08-核心索引结构.md:430`、`.claude/kb/layout/01-first-txn.md:257` | 两处条款字面写「改动计数 1」 | 它们与 D8:403 的定义（= checkpoint_txg）打架 ⇒ 第十节失败条款 F4，本轮按定义算、把冲突报成 gap 行，不改条款 |
| 我为定次数在实验页上 grep `第.次跑\|复跑\|重跑` 时，命中行把上一次跑的结果带进来了（照实列）：层 0 状态数 262165 / 违例 0 / `journal_effect` 差异态 3 / 阳性对照 1020 of 2048 / `verification_ran` 6 / gap 25 行 / 变异 67 条 0 无效 0 没红 / `defer_queue_per_device` 16384 / `mkfs_generation_records` 2 / 走读失败报的槽号 50248 | 上一次跑（第十次）的计数 | 影响两处：① 第六节量 7 因此**不写成「必须等于 262165」**，写成「与第十次跑的产物逐行比，变化的行逐行列出」——拿记住的数当门槛就是自己给自己发标准答案；② 这些数一个都不进第七节的锚点 |
| 我自己算的（第七节全部列出，命令核过） | 224 / 823165152 / 138 / `03 00 00 00 00 00 00 00` 等 | 它们是锚点，不符即作废（第十一节） |

**没读**：`.claude/kb/experiments/142-第一个事务的干跑.md` 的「判决」「空白清单」「它答不了的」三节正文（除上面 grep 命中的那几行），`research/results/` 下任何产物。

## 五、臂、阳性对照、真实基线

| 臂 | 定义 | 谁认它 |
|---|---|---|
| **臂 A（装置·新口径）** | E142 装置 `:2213` 的 `change_count: 1` 改成 `change_count: FIRST_TRANSACTION_TXG`（= 3），**其余一个字不改**；同一次进程跑整条路，落盘镜像记为 M_A | D8 已定项 6 字段表偏移 88 的定义（`最后一次改动所在发布的 checkpoint_txg`），2026-09-18 用户定案 |
| **臂 B（装置·旧口径 = 真实基线）** | 同一次进程里再跑一遍整条路，`change_count` 取 1，镜像记为 M_B。这就是第十次跑的口径 | 支持它的人认的样子：**改动计数 = 这个对象被改过的次数，第一次落盘的版本就是第 1 次** ⇒ 写 1。kb 里有两处字面支持（`decisions/08:430`「一条记录（inode 1、出生代 1、locality 0、nlink 1、改动计数 1、三段 0）」、`layout/01-first-txn.md:257` 那一格的「1」），不是稻草人：真要按这条读法，该改的是 D8:403 的定义那句 |
| **臂 C（实装）** | `crates/singlefs-harness` 的 `run_first_transaction`（`scenario.rs:71`）在内存盘上跑同一条路，镜像记为 M_C。今天它写 3（`transaction.rs:1172`） | `crates/` 今天的实现 |

**真实基线** = 臂 B：本轮真正要问的是「从 1 改到 3，盘上还有哪些字节跟着变」，没有 M_B 就只有一份镜像、没得相减。
**阳性对照（三条臂各跑一条，不许只跑第一条）**：对每条臂的镜像各注入**一处已知的单字节改动**——把该镜像里 inode 叶（两盘各一份）偏移 88 那一格的首字节改成 `02`——再跑同一套比对器：

| 对照 | 必须看到 | 看不到时 |
|---|---|---|
| 对照 A | 比对器报出差异，且报出的位置正是注入点（设备 0 / 1、绝对偏移 823165152，见第七节） | **整轮作废**：比对器分不出「字节变了」 |
| 对照 B | 同上（注入在 M_B 上） | 整轮作废 |
| 对照 C | 同上（注入在 M_C 上，跑量 5 的那套装置↔实装比对） | 整轮作废 |

**为什么三条都要跑**：量 1（A 减 B）与量 5（A 比 C）用的是两套不同的比对路径，只证明其中一套分得出差别，另一套就没过闸（`test-discipline.md`「阳性对照必须对**每一条**被测的臂都跑」）。

**分段**（一个执行员一次做不完时按这个次序做；每段交回，主 agent 对着问题单判续不续，第 1 行够判就停）：

- **第一段（补问题单第 1 行，补完即够判）**：装置改口径；加 M_A / M_B 两份镜像与全量逐字节相减的输出行；量 1、2、3、4、6；实装臂 M_C 与量 5（逐字节比对——它就是问题单的翻面观测，不许挪到后面）；三条阳性对照；变异 V1–V4。
- **第二段（第一段交回后主 agent 判要不要）**：几何敏感性两点（量 8）、变异 V5、整份产物重跑留存（层 0 全量 = 量 7）与 `research/scripts/replay.sh:157` 改指新产物。
  ⚠️ 这一段**不是为了让第 1 行够判**：`.claude/gate.d/52-segment-registry.sh`、`54-layer0-replay.sh`、`55-qemu-first-transaction.sh` 与 `replay.sh:157` 都指着第十次跑的产物，不重跑整份产物它们会红——那是工程需要，由主 agent 单独派，不算这一行问题的证据。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，不合成一个 true / false。这个实验只有问题单第 1 行，所以每一行都写它对那一行的翻面取值。

| # | 量（产物里的行） | 怎么算 | 门槛 / 判定 | 对问题单第 1 行的作用与翻面取值 |
|---|---|---|---|---|
| 1 | `name=change_count_diff`（每段差异一行） | M_A 与 M_B 在 2 × 4 GiB 上逐字节相减，合并相邻差异字节成段，每段报 `device= offset= length= old= new= structure= offset_in_structure=` | 差异段数 ≥ 1；**每一段都必须给出六元组**，缺任一列判装置输 | 这是答案本身。**够判 = 问题单点名的六处各至少出现一段**。某一处一段都没有 ⇒ 那一处不随改动计数变 ⇒ 够判条件那一句翻面，如实记、不作废 |
| 2 | `name=change_count_value`（两份 inode 叶各一行） | 从 M_A 读 inode 叶记录区偏移 88 起的 8 字节 | 必须等于 `03 00 00 00 00 00 00 00`；M_B 那一份必须等于 `01 00 00 00 00 00 00 00` | 不等 ⇒ 装置写错，查装置、改完算这一次跑的重做，不改判据 |
| 3 | `name=diff_explained` | 逐段给出「谁的校验和 / 指针罩到它」，注明依据的 kb 条款（哪一份文件哪一节） | 归不了因的段数 = 0 | > 0 ⇒ 每一段记一行 `name=gap`，**不作废**：那是「盘上还有字节跟着变而没有条款解释它」，正是这一行问题要的产出 |
| 4 | `name=extra_structures` | 差异段所属结构去重之后，减去问题单点名的六处 | 如实列，**空与非空都是正当结果** | 非空 ⇒ 「六处」那句要扩（按第三节，journal 记录的点名项与新根段、中央映射条目都装着单元校验和，它们变不变由这一格回答，不由我在跑前定）。为空也照记：说明级联止于那六处 |
| 5 | `name=impl_bytes_equal`（每个区域一行） | M_A 与 M_C 在第一节第 3 条那张**区域清单**上按字节 `cmp`，报 `region= equal= first_diff_offset= mismatch_bytes=` | 清单上每个区域 `equal=1` | **这是问题单的翻面观测**。任一区域不等 ⇒ 第十一节的**停机条款**：两边都查，既不作废也不当结果 |
| 6 | `name=width` 全部行、`name=segments` 全部行 | 与第十次跑的留存产物 `research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out` 里同名的行逐字比 | 两类行**逐字相同**，一行不差 | 这是够判条件的后半句。任一行不同 ⇒ 第十节 F2（改动计数动了宽度或写路径），先查装置 |
| 7 | 其余结果行（`name=layer0`、`name=layer0_control`、`name=journal_effect`、`name=probe`、`name=recover_full`、`name=accounting`、`name=verdict`……） | 与第十次跑的产物**逐行比，变化的行逐行列出** | 不设数值门槛（上一次跑的计数我已经在第四节列过，拿它当门槛就是发标准答案） | 有行变化 ⇒ 第十节 F5：改动计数进了恢复 / 层 0 / 探针的判定，逐行记、不作废 |
| 8 | `name=geometry_sensitivity`（两行） | 见第八节的两个取样点，各报一次差异段的**位置集合**（不含值） | 位置集合与主取样点相同 ⇒ 记两点；不同 ⇒ 记「不稳定」并列出差在哪 | 第 1 行的答案是不是只在这一个几何上成立 |
| 9 | 附带，**够判后不跑**：改动计数取 5、7 两个值再各跑一次差异 | 同量 1 | — | 够判之后不跑；量 8 的第二个取样点已经覆盖「换一个值」这件事 |

**门槛不是臂定义的同义反复**：量 1 的门槛是「六处各至少一段」，臂 A 的定义只说「偏移 88 写 3」，推不出别的五处会变；量 5 的门槛（装置↔实装全等）也推不出——两份代码各写各的。

## 七、钉绝对值的断言

臂跟臂比出来的「都变了」很弱（三条臂共用同一套校验和代码时，一起错也长成相等的样子），所以每一条互比旁边钉一条绝对值。分两类：

**（甲）出自被测条款本身**——不符走第十节「条款可能错」，不作废：

| 断言 | 出处 |
|---|---|
| 改动计数在 inode 记录内偏移 **88**、宽 **8** | `decisions/08:403`（第二节整段抄） |
| 它的值 = **最后一次改动所在发布的 checkpoint_txg**，第一个事务这次发布是 **3**（两次暖机之后） | `decisions/08:403` + `crates/singlefs-format/src/lib.rs:200-201` 的注释所引 D16 已定项 6 / 已定项 8 |
| 容器内第 n 条记录起点 = **135 + 140n** ⇒ 改动计数在叶内偏移 **135 + 88 = 223** | `decisions/08:391`（码 3 头 107 + 预留 **28**） |
| 记录区从 **136** 起 ⇒ 改动计数在叶内偏移 **136 + 88 = 224** | `layout/01-first-txn.md:249`（107 + 预留 **29**） |

⚠️ 这两条条款自己差 1（28 对 29 的预留位）。本轮**不挑一边当对**：装置与实装今天都按 136（第三节 `:835-880`），产物照 136 算并把这一处报成一行 `name=gap`；哪一边错由条款侧处置（第十节 F3）。

**（乙）独立算出、用命令核过**——不符即作废（第十一节）：

| 断言 | 值 |
|---|---|
| `1` 与 `3` 的 u64 小端表示 | `01 00 00 00 00 00 00 00` / `03 00 00 00 00 00 00 00`；**只差 1 个字节**（低位） |
| ⇒ 改动计数这一格本身在 M_A 与 M_B 之间的差异字节数 | 每份 inode 叶 **1** 字节；两盘各一份 ⇒ **2** 字节。其余差异字节**全部**是校验和 / 指针里的校验和 |
| 改动计数首字节的盘上绝对偏移（按记录区 136 起） | 50242 × 16384 + 136 + 88 = **823165152**，设备 0 与设备 1 同偏移 |
| 同一格按条款 135 起算 | 823165151（差 1，甲类那条冲突的盘上形态） |
| inode 叶载荷 CRC / 头校验和的绝对偏移 | 823165017（宽 4）/ 823164938（宽 32） |
| inode 树根（槽 50244，首字节 **823197696**）里子指针的两个单元校验和，结构内偏移 | **225** 与 **239**（条目区 131 起 + 分隔 key 8 + 身份引用 26 + 指针头 50 + 位置条目内 10；`LOC_ENTRY = 14`） |
| inode 树根自己的载荷 CRC / 头校验和，结构内偏移 | **92**（= 76 + 2 × 8）/ **10**（宽 32） |
| 根记录自证校验和，结构内偏移 | **138**（= 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8），宽 32，罩整 512 槽、自身按 0 参与 |
| 中央映射根 / 树表单元的落点首字节 | 823246848 / 823263232 |

核算命令与原样输出见第十三节。根槽与 journal 记录的绝对偏移由环几何算出，装置在产物里报出来，不在这里钉。

## 八、轨迹与几何敏感性

**轨迹这一格不适用，写明为什么**：这一轮没有任何量被条款当停机 / 起跑 / 准入谓词的输入——被问的是一次确定性发布的字节，装置纯确定性（fsid、写入时刻、内容都是固定参数），**没有轮次、没有时间序列**，「吃光初始供给之后的峰值 / 为正的轮数 / 期末值」三样在这里没有对应物。正文里因此也不许写「从来」「恒」这类说轨迹的词，只许写「这一次发布里」。

**几何敏感性（`mutation-sampling.md` 第六类）**：跑前写死的判决（「这六处跟着变」）只在一个取样点上量过——2 块盘、物理块 512、槽 16384、文件 3000 字节、改动计数 1 → 3。至少再取两点：

| 取样点 | 怎么取 | 报什么 |
|---|---|---|
| **G1（方向相反：盘数 2 → 1）** | 装置已有的 `PoolParameters::control_one_device_no_barriers`（`:1853` 一带）：两条位置条目都指盘 0 | 差异段的**位置集合**（设备 + 结构 + 结构内偏移，不含值）。与主取样点相比：段数应随副本数变，结构集合不应变 |
| **G2（换一个值：1 → 4）** | 臂 A 的 `change_count` 取 4 再与 M_B 相减 | 位置集合必须与主取样点**逐项相同**，只有 `new=` 变 |

判定：任一取样点的**结构集合**与主取样点不同 ⇒ 记「不稳定」，并列出差在哪一处；都相同 ⇒ 记两点（不写成「与几何无关」）。
**判别力自证**：把量 8 的判据从「位置集合相等」挪到「位置集合相差 ≤ 1 段」，G1 那一格必须由红转绿（或反向：故意让比对器漏掉一段，必须由绿转红）——见第九节 V6。

## 九、变异（给执行员，每条写明它在哪个取样点上改变输出，防等价变异）

写进 `research/mutations/e142_first_transaction_dry_run.tsv`（三段制表符分隔），用 `bash research/scripts/mutate.sh e142-first-txn-dry-run <源文件> <变异表>` 跑；实装侧那一条写进 `crates/mutations.tsv`（那份表的跑法照它自己的表头）。

| # | 变异 | 在哪个取样点上改变输出 | 该红在哪 |
|---|---|---|---|
| V1 | 装置 `change_count: FIRST_TRANSACTION_TXG` → `1` | 主取样点（2 盘、3000 字节）：M_A 的偏移 88 那 8 字节 | 量 2（`03 00…` 那一格）与量 5（装置↔实装全等）同时红 |
| V2 | 比对器的扫描范围从「两块盘全部字节」缩成「inode 叶那两个槽」 | 主取样点：量 1 的差异段只剩 inode 叶那几段 | 量 1 的「六处各至少一段」红（inode 树根 / 树表 / 根记录三处缺席） |
| V3 | `location_entries` 里 `let unit_checksum = castagnoli_crc32(unit);` → `let unit_checksum = 0;` | 主取样点：指针里的单元校验和不再随被指单元变 | 量 1 里 inode 树根、树表单元、根记录那几段消失 ⇒ 量 1 红；量 5 也红 |
| V4 | 实装 `crates/singlefs-core/src/transaction.rs:1172` 的 `change_count: FIRST_TRANSACTION_TXG` → `change_count: FIRST_TRANSACTION_TXG + 1` | 主取样点：M_C 的偏移 88 那 8 字节 | 量 5 红（装置↔实装第一处不等就在 823165152） |
| V5 | 装置 `NamedUnit::write_to` 里写位置条目改成写全零位置条目 | 主取样点：journal 记录里点名项那几段 | 若量 4 本轮报出了 journal 记录，这一条让它消失 ⇒ 量 4 与量 1 的段数同时变；若量 4 本轮为空，这一条**必须**改动 `name=recover_full` / `name=layer0`（点名项是恢复的输入），红在那里 |
| V6 | 量 8 的判据从「位置集合逐项相同」放宽成「相差 ≤ 1 段」 | G1 / G2 两个取样点 | 量 8 那一格必须由红转绿——这是几何敏感性那条判据自己的判别力自证 |

每条变异跑完必须还原并确认基线全绿（`mutate.sh` 自带）。**等价变异的防法**：上面每一条都写出了「在主取样点上哪一行输出变了」；写不出那一行的变异不进表。

⚠️ **V4 的锚点不唯一**（`mutation-sampling.md` 第七类）：`change_count: FIRST_TRANSACTION_TXG,` 这一串在 `crates/singlefs-core/src/transaction.rs` 里出现两次（`:1172` 生产路径、`:2234` 单测里的样例），
照抄进 `crates/mutations.tsv` 会被「原文必须恰好命中一次」判红（门禁 59 号）。执行员要把锚点加长到唯一命中（连上 `:1169-1171` 那三行注释里的任一整行），并在产物里贴出命中数。
`crates/mutations.tsv:22` 已有一条「步 1 变异：改动计数留 1」锚在 `change_count: txg.0,`（`publish_overwrite`，`:1209`）上——那是覆盖写那一路，与 V4 是两处，不许合并。

## 十、失败条款（每条紧跟「什么观测会让它触发」）

| # | 条款 | 什么观测会让它触发 | 触发时怎么办 |
|---|---|---|---|
| F1 | 装置与 `crates/` 实装在区域清单上对不上 | 量 5 任一行 `equal=0`（产物里给出 `region=`、`first_diff_offset=`、`mismatch_bytes=`） | **停机**（第十一节）：两边都查，既不作废也不当结果 |
| F2 | 改动计数动了宽度或写路径 | 量 6：`name=width` 或 `name=segments` 任一行与第十次跑的产物不逐字相同 | 先查装置；装置没错则是条款有洞（定长字段改值不该动宽度与段序列），如实记，不改判据 |
| F3 | 条款可能错（记录区起点 135 对 136） | 量 2 报出的叶内偏移是 224，而 `decisions/08:391` 算出 223 —— **这一条在跑之前就已经不等**，所以产物里**必须**有那一行 `name=gap`；产物里没有这一行 ⇒ 判装置输（它没查这一处） | 报成 gap 行，按定义（136，装置与实装今天的写法）出产物，不作废、不由本实验改条款 |
| F4 | 条款自己不一致（值写 1） | `decisions/08:430` 的「改动计数 1」与 `layout/01-first-txn.md:257` 那一格的「1」，同 `decisions/08:403` 的定义（= checkpoint_txg = 3）不符 —— 同样是跑前就成立，产物里必须有对应的 `name=gap` 行 | 本轮按定义算；kb 正文的改由书记员做（`milestone/02-second-txn.md:320` 已把它派给书记员），不由本实验改 |
| F5 | 改动计数进了恢复 / 层 0 / 探针的判定 | 量 7：`name=layer0`、`name=layer0_control`、`name=journal_effect`、`name=probe`、`name=recover_full`、`name=verdict` 任一行与第十次跑的产物不同 | **不作废**，是正当结果：逐行列出变化，并说明改动计数怎么进的判定 |
| F6 | 差异里出现没有条款解释的字节 | 量 3 的「归不了因的段数」> 0 | **不作废**：每段记一行 `name=gap`，这正是这一行问题要的产出 |

**反面自查（前件写反的检查）**：上面六条，F1 / F2 / F5 / F6 的前件都是「量出某个非零 / 不相同」，F3 / F4 的前件在跑之前就成立（因此写成「产物里必须有那一行，没有就判装置输」）。
没有任何一条写成「测不出效应 ⇒ 实现有 bug ⇒ 整轮作废」：本轮「测不出效应」的形态是「某一处根本没跟着变」，它按第六节量 1 记成**正当结果**（够判条件那一句翻面），不作废。

## 十一、作废条款与停机条款

**作废（产物留成 `-void.out`，不入库）**：

1. 三条阳性对照（A / B / C）任一条报不出注入的那一个字节 ⇒ 比对器分不出「字节变了」，整轮作废。触发观测：对照那三行里 `detected=0`，或报出的位置不是注入点。
2. 第七节（乙）里任一条「独立算出、命令核过」的锚点与产物不符（`03 00…` 那一格、823165152、叶载荷 CRC / 头校验和的偏移、结构内 225 / 239 / 92 / 10 / 138、差异字节数 2）⇒ 装置或比对器的偏移算错，整轮作废。

**停机（F1，`implementation-first.md` 第 4 条）**：装置镜像与实装镜像在区域清单上对不上 ⇒ **停下来交主 agent**，两边都查，不许默认装置对，也不许默认实现对；这一格既不作废也不当结果，产物里如实留 `name=impl_bytes_equal` 那几行。

**够判停机**：问题单只有第 1 行。量 1–6 出齐、点名的六处各给出新值、`name=width` 与 `name=segments` 两类行逐字相同 ⇒ **第 1 行够判，当场停**。
没跑的量逐条标「够判后未跑」：量 8（几何敏感性两点）、量 9（取值 5 / 7）、变异 V5 / V6、整份产物重跑与 `replay.sh:157` 改指——它们要不要跑由主 agent 按第五节的分段另判（门禁 52 / 54 / 55 的需要不是这一行问题的证据）。

## 十二、修订

写于 2026-09-18（本机 UTC），装置改完、产物之前。两条都是执行记录，不改第六/七/九节任何一条判据。

1. **变异 V4 未追进 `crates/mutations.tsv`，V1 的锚点从主调用点改锚在共用函数内部**：`publish_first_file` 加了 `change_count: u64` 形参（不再是字面 `1`），
   主调用点（`main()` 里那一行）因此没有单测直接盖到（`main()` 不可测）；把 M_A / M_B / 取样点 G1 / G2 都经由的共用函数 `run_full_pipeline` 的转发行
   （`e142_first_transaction_dry_run.rs` 里 `let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), change_count);`）
   作为 V1（`research/mutations/e142_first_transaction_dry_run.tsv` 的 `M68_run_full_pipeline_ignores_requested_change_count`）的锚点，
   新增单测 `run_full_pipeline_writes_the_requested_change_count_into_the_inode_leaf_record` 钉住（对 1 / `FIRST_TRANSACTION_TXG` / 4 三个值逐一读盘核对），
   跑变异表时红在这条与 `run_full_pipeline_matches_the_manually_assembled_pool_for_the_same_change_count`（`mutate-full.log` 原样：`✅ [M68_run_full_pipeline_ignores_requested_change_count] 红：...`）。效果与登记原意等价，只是锚点位置变了。
2. **量 5（装置↔`crates/` 实装区域清单逐字节比对）、阳性对照 C、变异 V4 三项做不了，按第十一节 F1 的「停机条款」在产物之前停下**：
   `experiment-runner` 的写范围（`.claude/hooks/agent-write-scope.tsv` 第 4 行、第 9-19 行）不含 `crates/**`——`crates/**` 只登记给 `implementation-writer`；
   要跑量 5 需要在 `crates/` 侧新建一个能把 `singlefs-harness::scenario::run_first_transaction` 的镜像按第一节第 3 条那份区域清单转成字节的产出（新测试/新二进制/新依赖），
   这三样都落在 `crates/**` 或本包 `[dependencies]`（同样没登记给这次跑），改不了、加不了。这不是「量做不完」（第五节的分段问题），是「这一条真跑不动」——
   已按 `experiment-runner.md` 第 3b 步的停机处置：不把产物写进 `research/results/`、不写实验页，草稿产物（含量 1-4、6、8、阳性对照 A/B、量 5 的 21 行区域清单原样标 `equal=unknown`）留在
   `/tmp/claude-1000/e142-r11-runner/e142-r11-draft.out`，交主 agent 定要不要派 `implementation-writer` 补上 `crates/` 那一侧再续跑。
3. **`crates/` 侧补上区域字节产出之后量 5 真跑出触发 F1，查过之后已经收口，装置改了两行**：写于 2026-09-18（本机 UTC），看过 F1 诊断报告之后、重跑产物之前。
   量 5 在 `research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out` 第 233 行 `name=impl_bytes_equal_summary regions=21 equal=10 unequal=11` 触发（第 192–232 行里 11 行 `equal=false`），按第十一节 F1「两边都查，不作废也不当结果」停机；
   诊断报告 `research/prompts/e142-r11-f1-diagnosis.md`（只读诊断，仓里代码当时一个字节没改）查明：11 处不等全部同一根因——本节第 1 条要求的「同 3000 字节内容」（对应本文件第 18 行）与第 21 行「同一条路（同参数）」没有做到：装置 `:3717` 的文件内容公式是 `(index * 7 + 3) % 251`，`crates/singlefs-harness/src/scenario.rs:29-33` 的 `first_file_content()` 是 `index % 251`，两串不同的 3000 字节；诊断报告第六节把任一边内容换成另一边之后 21/21 全等，第三/四节用独立的按位 CRC32C 核对器把两边的校验和逐格核对都算对了（两边各 92 格 `all_ok=True`），排除了覆盖范围、封口时机、比错字段三类候选解释。
   主 agent 判定：改装置这一侧对齐到 `index % 251`（`crates/` 一侧被 QEMU 日志核对器与三份测试用例、层 0 两条流钉着，改动面更大）。已改 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3717`（`main()` 里的 `file_bytes`）与 `:4124` 附近的 `sample_file()`（单测用，同一个式子），50 个单测全绿、75 条变异全抓（跑法与结果见执行员报告）。
   顺带修了诊断报告「顺带发现」第 1 条指出的一处不同步：`name=impl_bytes_equal` 在 `equal=true` 时此前把 `first_diff_offset` 的 `None` 与 `equal=false` 时「差异藏在没抽样的中段」的 `None` 共用同一个 `unknown_middle_of_region` 文案，`equal=true` 时改成固定打 `none`（不是新判据，是把「没有差异」与「有差异但定位不到」这两种不同的 `None` 分开描述）。
   不改第六/七/九节任何一条判据；量 5 的字面判据（21 行 `equal=`、F1 的触发与处置）一字不动，这一条只是记「F1 因为什么触发、改了什么、为什么」。

## 十三、读过的文件与跑过的命令

**读过的文件**（行号区间；grep 命中行也列）：

| 文件 | 读了哪几行 |
|---|---|
| `.claude/agent-common.md` | 1-62（整份） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 35-135 |
| `.claude/rules/three-way-inference.md` | 224（`grep -n 岔路单` 命中行） |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 108-140 |
| `.claude/rules/mutation-sampling.md` | 65-82 |
| `.claude/rules/implementation-first.md` | 1-35（整份） |
| `research/prompts/e142-preregistration.md` | 1-64（整份；原登记的判据怎么定的） |
| `.claude/kb/experiments/142-第一个事务的干跑.md` | 小节标题一览（`grep -n '^#'`）、14-23（复跑命令那一块）、`grep -n '第.次跑\|复跑\|重跑\|跑了\|次跑'` 的命中行（8 / 12 / 14 / 24 / 26 / 36 / 37 / 41 / 49 / 57 / 147 / 149 / 150 / 151 / 158 / 176 / 177 / 186 / 201 / 202 / 204 / 205 / 208 / 220 / 225 / 228 / 233 / 236 / 239 / 241 / 245 / 250）——第四节已照实列出它们带进来的数 |
| `.claude/kb/decisions/08-核心索引结构.md` | 332-440 |
| `.claude/kb/layout/01-first-txn.md` | 28-... 的小节标题一览、237-280、`grep -n` 命中的 89 / 103 / 163 / 243 / 245 / 249 / 251 / 257 / 269 / 281 |
| `.claude/kb/milestone/02-second-txn.md` | `grep -n 改动计数` 命中行 90 / 113 / 320 / 473 / 509 |
| `crates/singlefs-core/src/transaction.rs` | 1125-1200、2195-2245，`grep -n` 命中若干行 |
| `crates/singlefs-core/src/records.rs` | 25-95，`grep -n change_count` 命中行 36 / 60 / 84 / 89 / 380 |
| `crates/singlefs-harness/src/scenario.rs` | 1-140 |
| `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs` | `grep -n` 命中行 2 / 3 / 38 / 54 / 56 / 57 / 60 / 63 / 75 / 174 / 261 / 374 / 692 |
| `crates/singlefs-format/src/lib.rs` | `grep -n` 命中行 200 / 201 |
| `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` | 1-260（常量区，节选自条目一览）、646-760、827-900、1128-1200、1359-1420、1520-1560、1790-1870、2195-2245、3165-3402（emit 行一览） |
| `research/mutations/e142_first_transaction_dry_run.tsv` | 1-8、末 3 行 |
| `crates/mutations.tsv` | 1-3、22 |
| `research/scripts/mutate.sh` | 1-25 |
| `research/scripts/quote-kb.py` | 1-40 |
| `research/scripts/claim-experiment.sh` | 1-30 |
| `research/scripts/replay.sh` | 157（grep 命中行） |

**跑过的命令**（原样；输出见上文各节）：

```bash
nice -n 19 python3 research/scripts/quote-kb.py /tmp/claude-1000/e142-rerun-designer/quotes.md \
  '.claude/kb/decisions/08-核心索引结构.md:391-410' \
  '.claude/kb/decisions/08-核心索引结构.md:429-430' \
  '.claude/kb/layout/01-first-txn.md~^\| inode 树根（码 2）' \
  '.claude/kb/layout/01-first-txn.md~^\| inode 叶容器头' \
  '.claude/kb/layout/01-first-txn.md~^\| inode 记录（偏移 88）' \
  '.claude/kb/milestone/02-second-txn.md~^\| 11 \| 改动计数'
#   ✓ 6 段整抄进 /tmp/claude-1000/e142-rerun-designer/quotes.md，回读逐字节一致
grep -rn 'FIRST_TRANSACTION_TXG\|publish_first_file' crates/ --include=*.rs
grep -rn '改动计数' .claude/kb/ --exclude-dir=experiments --exclude-dir=results
grep -n 'change_count\|改动计数' research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs | head -30
```

第七节（乙）的锚点核算（原样输出贴在这里，行号与值都是这一次跑出来的）：

```
inode 叶首字节 = 823164928
改动计数首字节（记录区 136 起）= 823165152
改动计数首字节（条款 135 起）= 823165151
叶载荷 CRC = 823165017 叶头校验和 = 823164938 宽 32
inode 树根首字节 = 823197696  子指针 = 165  两个单元校验和结构内偏移 = 225 239
inode 树根载荷 CRC 结构内偏移 = 92  头校验和 = 10 宽 32
中央映射根首字节 = 823246848  树表单元首字节 = 823263232
根记录自证校验和偏移 = 138 宽 32
u64 小端 1 / 2 / 3 / 4 = 01 00 00 00 00 00 00 00 | 02 00 00 00 00 00 00 00 | 03 00 00 00 00 00 00 00 | 04 00 00 00 00 00 00 00
1 与 3 的小端表示差几个字节 = 1
```

（算式：叶槽 50242、根槽 50244、映射根 50247、树表 50248，槽宽 16384；记录区 136 = 码 3 头 107 + 预留 29；条目区 131 = 86 + 2 × 8 + 29；子指针 165 = 131 + 8 + 26；位置条目 14 = 设备 4 + 槽 6 + 校验和 4；根记录校验和 138 = 4 + 16 + 4 + 4 + 8 + 86 + 8 + 8。）
