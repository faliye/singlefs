# D18（块里携带什么信息） 已定项 3 那条提议（自描述头加 `locality_id`） 第三轮：攻方腿（Opus）报告

立场：反推 / 攻。假设倾向（丙′-记录优先）是错的，也假设甲-S、乙′、丙′-S1 各有死角；换前两轮都没建的面：删除与截断之后留在盘上的旧版、被抛弃时间线的孤儿（管理员回退与崩溃，实例表 / 祖先表读不出）、打包之后重编 / 搬出去 / 整容器重写、克隆之后两个头各自重编同一个共享对象、甲-S 的 S6–S8 本身、丙′ 两种取值次序本身。
模型：`research/prompts/d18-locality-r3-opus-model/model.py`（只用 std 的 Python，`nice -n 19` 跑，没编 Rust、没留 target），原始输出同目录 `output.txt`，复跑命令与 sha256 在第九节，判决摘要在第十节。前两轮的模型目录只读过，没改、没拷；这一轮别的腿的产出一份都没打开。
引 kb 一律写 kb 文件名加那份文件自己的行号，行号 2026-09-14 在 `.claude/kb/` 下现查，不从背景材料里数；长行只指路，不摘句。

## 一、失败条款先核：前提表一至二十

二十条逐条对过附录与 kb 原文，对得上，失败条款不触发。一处备查：前提十五末分句「打包进容器本来就要重加密」比 `27-小数据打包容器.md:384` 宽一格（原文说的是对象连同槽搬到另一个容器时），第二轮攻方腿已判它由同一条机制推得出，这一轮同意。

**前提表之外、这一轮承重的条款**（按 `.claude/rules/three-way-inference.md`「正文没点名的枢纽条款」列出，交主 agent 核；右栏写的是模型拿它做什么，不是条款的转述）：

| 条款 | kb 文件与行号 | 模型拿它做什么 |
|---|---|---|
| 实例表的行怎么写（恢复 / 回退 / 切换）、全局已发布谓词、「表不可读时」那一句 | `18-块里携带什么信息.md:879`（长行，只指路） | 孤儿判不判已发布；实例表读不出时旧实例的单元怎么判 |
| I-1.2（块头写序已发布） | `invariants.md:22`（长行，只指路） | 码 1 与码 3 两套已发布判法 |
| 管理员回退 = 一次恢复，回退行与中间实例的行 | `23-journal的角色与格式.md:1206`（长行，只指路） | 模型的回退操作 |
| 回退新根的 txg 取法 | `23-journal的角色与格式.md:1234`（长行，只指路） | 回退之后 txg 不重发（崩溃之后才重发） |
| C143（inode 号水位在回退后会退回去重发） | `checks-owed.md:146`（长行，只指路） | 回退与崩溃之后号被重发 |
| 现行版本由写序、实例表、祖先表三样判 | `11-索引节点要不要留消息缓冲区.md:197`（长行，只指路） | 扫描候选按祖先表过滤 |
| 墓碑区间记录的字段 | `18-块里携带什么信息.md:749`（长行，只指路） | 墓碑按对象 ID + 对象出生代 + 区间杀，不带树 ID、不带 locality |
| 类型 1 容器归写者树独占、别的头靠祖先表读到它们 | `18-块里携带什么信息.md:869`（长行，只指路） | 克隆头看不看得见 origin 的墓碑 |
| I-1.9（清扫水位内无可读垃圾） | `invariants.md:29` | 整行见下 |
| D27（小数据打包容器） 定死的四条 | `27-小数据打包容器.md:241`–`:244` | 整行见下 |
| D18（块里携带什么信息） 已定项 5 纪律 3 | `18-块里携带什么信息.md:521` | 整行见下；R-disk 那种读者读法的依据（第二节） |

`invariants.md:29` 整行：

> | I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |

`27-小数据打包容器.md:241`–`:244` 整四行：

> | 改一个已打包对象 | 走**未改动的**独立单元写路径写一个新单元（「搬出去」），容器一个字节不动 |
> | 已定项 5（活槽计数住哪）| **不能住容器里**——住容器里就要在对象被搬出后回头改容器里那个死槽标记 |
> | 死槽标记 / liveness 位 / 引用计数 | 一概**不许住容器**，同上一条同一个理由 |
> | 整理怎么回收死槽 | 只能**整个重写**（写一个新容器、旧容器整体释放），**不许就地压紧** |

`18-块里携带什么信息.md:521` 整行：

> | 3 | 一条纪律：该状态下**任何权威态结构不得由重建结果就地修复** | 它是 I-6.10（已停用·明文映射层不存在了） 逐字「差异一律判红，**不许就地修复**」扩到重建产物上，不是新机制 |

## 二、模型怎么建的（口径）

**与前两轮模型的差别**：前两轮把单元归属当神谕（这个头的这个对象引用过的物理件），孤儿、墓碑、祖先表都没建。这一轮拿掉神谕，扫描候选按条款算：

- **扫描候选** = 对象号相同、按祖先表对这个头可见、按实例表判已发布（码 1 单元按码 1 的判法，槽按它所在容器的码 3 判法）、对象出生代对得上（记录读得出取记录的，读不出取写序最大那个候选的）、没被死亡写序更大的可见墓碑盖住的物理件；每个锚点在候选里取写序最大的。
- **实例表读不出**：一条行都没有，旧实例的单元一律判已发布，被回退抛弃的整段时间线也在内（这是我对 `18-块里携带什么信息.md:879` 那句的读法）。**祖先表读不出**：克隆头拿不到克隆点，我取最宽的读法——origin 的全部件与全部墓碑都当可见；条款没写这种情形怎么办，是我定的。
- **删除与截断**写一条区间墓碑（死亡写序 = 做这件事的那个事务的写序），旧单元留在盘上；删除后记录留在 inode 树里（nlink = 0）。迁移进行中的对象不删、不截。
- **管理员回退**：回到当前时间线上 1 或 2 次发布之前的根，取新实例代号，写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，新根 txg = 最大 + 1；回退之后的逻辑态（记录、key、水位）照那个根恢复，所以 inode 号会被重发（C143）。**崩溃**：最后一次发布的根没落盘，它写出的单元成孤儿；恢复给所选根那个实例写 (i, 所选根 txg, 0)，取新实例代号，被抛弃的 txg 重发（恢复那次挂载的第一次发布写行、不带用户数据）。**实例切换没建**：它按新写序重做同一批事务，孤儿与重做件同内容、同首段，写序更小，碰不到 locality 这一维（推理，没建模）。
- **盘上保留每一个写出过的物理件与记录容器的每一版**；记录容器损坏为「当前链上最新 k 版读不出」时，在读得出、判已发布的版本里取 (诞生代号, 写序) 最大的那一版——实例表读不出时被抛弃的版本也在候选里。
- **重建规则**：对象的记录没坏、extent 条目一条没丢 ⇒ 原样不动。否则按臂重建：全部 key 编一个值的臂把幸存 key 也重编到选中值（S9）；按单元头编的臂留幸存 key 原来的首段（甲不能改：AAD）；没有幸存 key 的锚点从候选里择新补上。去掉 S6 的对照按 (头里的 locality, 锚点) 分组，只往丢了的原 key 上补、全丢时全补（对字面甲最有利的读法，同第二轮）。择新键是写序；槽与打包前那个单元写序逐字节相同（`19-块指针的结构与宽度预算.md:173`），并列时单元排前——这是甲-S「按单元头编」的最弱读法。

**两个写路径家族**加一个对照：`plain` = 乙′ / 丙′（重编 key 不动单元）；`jia` = 甲-S（不论加不加密，重编 key 都重写码 1 单元；槽不带 locality、不动；协议 C 期间的写只写主段，S8）；`jia_both` = 甲-S 去掉 S8（迁移期间一次写两个首段各一个单元）。

**臂**：

| 臂 | 首段怎么编 | 版本分组 | 选中值次序 | 重建视图里的记录 | 对应 |
|---|---|---|---|---|---|
| `bing_rec` | 这个对象全部 key 编一个值 | 锚点 | 记录 > 幸存 key > 0 | 选中值 | 丙′-记录优先（倾向） |
| `bing_s1` | 同上 | 锚点 | 幸存 key 众数 > 记录 > 0 | 选中值 | 丙′-S1 |
| `yi_s3` | 同上 | 锚点 | 幸存 key > 记录 > 头提示 > 0 | 选中值 | 乙′（S2 + S3，槽 +0） |
| `jia_s` | 按单元头编；被选中的是槽时按丙′-记录优先编 | 锚点（S6） | —— | 重建出的首段集合（S7） | 甲-S，打包规则读成 p2 |
| `jia_s_p1` | ≤ 4 KiB 的对象全部 key 按丙′-记录优先编 | 锚点 | —— | 丙′ 的值 | 甲-S，打包规则读成 p1（只在 `pack3` 跑） |
| `jia_noS6` | 按单元头编 | 要放回的 extent key（字面） | —— | S7 | 甲-S 去掉 S6（对照） |
| `jia_noS7` | 按单元头编 | 锚点 | —— | 记录读得出就用记录 | 甲-S 去掉 S7（对照，= 第二轮 `jia_anc_rec`） |

**甲-S 打包规则的两种读法**：候选原文是「≤ 4 KiB 的对象重建时按丙′ 的规则把全部 key 编同一个值」。p1 = 字面：凡是 ≤ 4 KiB 的对象都按丙′ 编（重建时对象大小从单元头的声明长度就知道）；p2 = 只有被选中的版本是槽时才按丙′ 编。

**读者两种读法**（这一轮新加，承重）：R-view = 读者用重建视图里的记录值；R-disk = 记录容器没坏时读者读盘上那条权威记录，容器坏了才用重建出的值。依据是纪律 3（上面整行）：inode 记录住码 3 权威态单元，不许由重建结果就地修复；视图要盖过一条完好的权威记录，读路径得多一条「字段取视图、不取盘」的规定，而三条候选的原文里只有甲-S 的 S7 写了「重建视图里的多值……读者逐个试」，丙′ 原文写「给记录与这个对象的全部 extent key……编同一个首段」，没写读者读哪一份，乙′ 原文只写 key 的取值次序、根本没提记录。

**检查**：读者按记录里的 locality 次序去找每个写过的 offset，第一个找到的算数。`missing` 找不到真值；`misread` 读到的不是真值，或在已删 / 已截掉 / 从没写过的位置读到数据（读到被删掉的版本、被抛弃时间线上的版本都记这一格）；`mac_fail` 甲的码 1 单元头里的 locality ≠ 查找路径的首段；`split_new` 原镜像只有一个首段而重建出两个。任一非 0 记一次失败。另数局部性变化、S10 判定（第六节）、S6 / S7 两条检查的红数与 S8 的「一次写两份」次数（第七节）。
这是确定性模型：无随机源、无 I/O，跑 N 遍与跑 1 遍信息量相同；证据强度来自手造历史里钉死的绝对值（第三节，全部 `assert … ok`）、阳性对照（墓碑丢了读到被删版本、实例表读不出读到被抛弃版本、S6 / S7 / S8 各自的去掉对照）与阴性对照（同一历史表读得出、墓碑在时 0）。

## 三、U1 重建：各臂最短的反例

### 世界与覆盖

十个世界，每个世界的每条历史 × 每个家族 × 每个头 × 全部损坏形态 × 每条臂 × 两种读者读法都重建一遍再检查。`output.txt` 的 `coverage` 行整行抄：

```
coverage copy_r max_depth=8 histories_one_step_longer=100 (0 = 在这个世界的操作上限之内穷举完了)
coverage tomb_a max_depth=7 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage tomb_c max_depth=8 histories_one_step_longer=124 (0 = 在这个世界的操作上限之内穷举完了)
coverage rollback_a max_depth=7 histories_one_step_longer=123536 (0 = 在这个世界的操作上限之内穷举完了)
coverage rollback_p max_depth=8 histories_one_step_longer=600 (0 = 在这个世界的操作上限之内穷举完了)
coverage rollback_c max_depth=8 histories_one_step_longer=664 (0 = 在这个世界的操作上限之内穷举完了)
coverage crash_a max_depth=7 histories_one_step_longer=68464 (0 = 在这个世界的操作上限之内穷举完了)
coverage pack3 max_depth=8 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage clone3 max_depth=7 histories_one_step_longer=369050 (0 = 在这个世界的操作上限之内穷举完了)
coverage clone_p max_depth=7 histories_one_step_longer=238188 (0 = 在这个世界的操作上限之内穷举完了)
```

| 世界 | 建了什么 | 操作上限 |
|---|---|---|
| `copy_r` | 协议 C（先抄后切、记录单值），两种读者读法 | 1 个对象；写 ≤ 3 |
| `tomb_a` | 截断、删除（写墓碑）、协议 A 重编；墓碑容器 在 / 丢 | 1 个对象；写 ≤ 2；截断 ≤ 1；删除 ≤ 1 |
| `tomb_c` | 截断 + 协议 C；墓碑容器 在 / 丢 | 1 个对象；写 ≤ 3；截断 ≤ 1 |
| `rollback_a` | 管理员回退 + 协议 A 重编；实例表 读得出 / 读不出 | 2 个对象；各写 ≤ 2；回退 ≤ 1 |
| `rollback_p` | 管理员回退 + 协议 P | 1 个对象；写 ≤ 2 |
| `rollback_c` | 管理员回退 + 协议 C | 1 个对象；写 ≤ 2 |
| `crash_a` | 崩溃（最后一次发布没落、txg 与 inode 号重发）+ 协议 A | 2 个对象；各写 ≤ 2；崩溃 ≤ 1 |
| `pack3` | 打包、整容器重写、打包之后重编、打包之后写（搬出去）、删除；对象 ≤ 4 KiB | 1 个对象、只写 offset 0；打包 ≤ 2 |
| `clone3` | 克隆之后两个头各自协议 A 重编、各自截断；祖先表 读得出 / 读不出 | 每头 1 个对象；各写 ≤ 2 |
| `clone_p` | 克隆之后任一头协议 P 重编；祖先表 读得出 / 读不出 | 同上 |

`tomb_a` 与 `pack3` 在操作上限之内穷举完了；其余八个到上面那个深度，再多一步还剩 `coverage` 行里那个数没跑。`copy_r` / `tomb_c` / `rollback_c` 的 `jia` 家族有一批历史在甲的写路径上不合法（S8 把「迁移期间的写」改成只写主段，后面能接的操作跟着变），`family` 行的 `skipped` 就是它们。

### 全部结果行（`row`）

列：world family arm reader max_depth histories patterns objects failures missing misread mac_fail split_new locality_changed s6_red s7_red。`failures` 数的是「至少一个失败字段非 0」的损坏形态个数，`objects` 是被检查的对象次数。

```
row copy_r plain bing_rec view 8 518 6732 6728 0 0 0 0 0 2896 0 0
row copy_r plain bing_rec disk 8 518 6732 6728 0 0 0 0 0 2896 0 0
row copy_r plain bing_s1 view 8 518 6732 6728 0 0 0 0 0 2552 0 0
row copy_r plain bing_s1 disk 8 518 6732 6728 576 1088 0 0 0 1976 0 0
row copy_r plain yi_s3 view 8 518 6732 6728 0 0 0 0 0 2212 0 0
row copy_r plain yi_s3 disk 8 518 6732 6728 576 1088 0 0 0 1636 0 0
row copy_r jia jia_s view 8 386 5004 5000 0 0 0 0 0 1844 0 0
row copy_r jia jia_s disk 8 386 5004 5000 444 540 0 0 0 1208 0 0
row copy_r jia jia_noS6 view 8 386 5004 5000 452 0 182 0 396 3324 2752 0
row copy_r jia jia_noS6 disk 8 386 5004 5000 430 0 114 0 396 2152 2752 0
row copy_r jia jia_noS7 view 8 386 5004 5000 1484 2232 0 0 0 1980 0 2764
row copy_r jia jia_noS7 disk 8 386 5004 5000 1484 2232 0 0 0 1980 0 2764
row copy_r jia_both jia_s view 8 518 6732 6728 60 0 0 0 60 3350 0 0
row copy_r jia_both jia_s disk 8 518 6732 6728 816 970 0 0 60 2282 0 0
row tomb_a plain bing_rec view 7 144 1916 1896 342 0 468 0 0 840 0 0
row tomb_a plain bing_rec disk 7 144 1916 1896 342 0 468 0 0 840 0 0
row tomb_a plain bing_s1 view 7 144 1916 1896 342 0 468 0 0 728 0 0
row tomb_a plain bing_s1 disk 7 144 1916 1896 342 0 468 0 0 728 0 0
row tomb_a plain yi_s3 view 7 144 1916 1896 342 0 468 0 0 610 0 0
row tomb_a plain yi_s3 disk 7 144 1916 1896 342 0 468 0 0 610 0 0
row tomb_a jia jia_s view 7 144 1916 1896 342 0 468 0 42 438 0 0
row tomb_a jia jia_s disk 7 144 1916 1896 342 0 456 0 42 426 0 0
row tomb_a jia jia_noS6 view 7 144 1916 1896 516 0 483 0 424 820 492 0
row tomb_a jia jia_noS6 disk 7 144 1916 1896 516 0 466 0 424 694 492 0
row tomb_a jia jia_noS7 view 7 144 1916 1896 472 256 370 0 42 652 0 354
row tomb_a jia jia_noS7 disk 7 144 1916 1896 472 256 370 0 42 652 0 354
row tomb_c plain bing_rec view 8 914 20128 20120 1060 0 1060 0 0 8760 0 0
row tomb_c plain bing_rec disk 8 914 20128 20120 1060 0 1060 0 0 8760 0 0
row tomb_c plain bing_s1 view 8 914 20128 20120 1060 0 1060 0 0 7704 0 0
row tomb_c plain bing_s1 disk 8 914 20128 20120 2518 2752 990 0 0 6176 0 0
row tomb_c plain yi_s3 view 8 914 20128 20120 1060 0 1060 0 0 6490 0 0
row tomb_c plain yi_s3 disk 8 914 20128 20120 2518 2752 990 0 0 4962 0 0
row tomb_c jia jia_s view 8 712 15736 15728 844 0 844 0 150 5780 0 0
row tomb_c jia jia_s disk 8 712 15736 15728 2066 1640 716 0 150 3830 0 0
row tomb_c jia jia_noS6 view 8 712 15736 15728 1871 0 1033 0 1556 10174 8258 0
row tomb_c jia jia_noS6 disk 8 712 15736 15728 1798 0 782 0 1556 6626 8258 0
row tomb_c jia jia_noS7 view 8 712 15736 15728 5032 6416 648 0 150 6228 0 8076
row tomb_c jia jia_noS7 disk 8 712 15736 15728 5032 6416 648 0 150 6228 0 8076
row tomb_c jia_both jia_s view 8 914 20128 20120 1180 0 1060 0 270 9310 0 0
row tomb_c jia_both jia_s disk 8 914 20128 20120 3061 2598 888 0 270 6378 0 0
```

```
row rollback_a plain bing_rec view 7 40810 1327760 2407936 103096 7040 104792 0 0 750912 0 0
row rollback_a plain bing_rec disk 7 40810 1327760 2407936 103096 7040 104792 0 0 750912 0 0
row rollback_a plain bing_s1 view 7 40810 1327760 2407936 103096 7040 104792 0 0 538476 0 0
row rollback_a plain bing_s1 disk 7 40810 1327760 2407936 103096 7040 104792 0 0 538476 0 0
row rollback_a plain yi_s3 view 7 40810 1327760 2407936 103096 7040 104792 0 0 221596 0 0
row rollback_a plain yi_s3 disk 7 40810 1327760 2407936 103096 7040 104792 0 0 221596 0 0
row rollback_a jia jia_s view 7 40810 1327760 2407936 107796 7040 100564 0 17964 46788 0 0
row rollback_a jia jia_s disk 7 40810 1327760 2407936 112416 18440 96072 0 17964 33196 0 0
row rollback_a jia jia_noS6 view 7 40810 1327760 2407936 424036 7056 126318 0 383672 387400 442084 0
row rollback_a jia jia_noS6 disk 7 40810 1327760 2407936 424036 7056 116994 0 383672 288156 442084 0
row rollback_a jia jia_noS7 view 7 40810 1327760 2407936 411448 500104 93692 0 17964 368096 0 497956
row rollback_a jia jia_noS7 disk 7 40810 1327760 2407936 411448 500104 93692 0 17964 368096 0 497956
row rollback_p plain bing_rec view 8 2038 37672 37600 2996 0 2996 0 0 26056 0 0
row rollback_p plain bing_rec disk 8 2038 37672 37600 2996 0 2996 0 0 20344 0 0
row rollback_p plain bing_s1 view 8 2038 37672 37600 2996 0 2996 0 0 24652 0 0
row rollback_p plain bing_s1 disk 8 2038 37672 37600 2996 0 2996 0 0 18940 0 0
row rollback_p plain yi_s3 view 8 2038 37672 37600 2996 0 2996 0 0 23644 0 0
row rollback_p plain yi_s3 disk 8 2038 37672 37600 2996 0 2996 0 0 17932 0 0
row rollback_p jia jia_s view 8 2038 37672 37600 2398 0 2270 0 804 15854 0 0
row rollback_p jia jia_s disk 8 2038 37672 37600 2406 24 2202 0 804 12072 0 0
row rollback_p jia jia_noS6 view 8 2038 37672 37600 9662 0 3402 0 8728 14696 10368 0
row rollback_p jia jia_noS6 disk 8 2038 37672 37600 9648 0 3028 0 8728 11600 10368 0
row rollback_p jia jia_noS7 view 8 2038 37672 37600 9528 10540 2014 0 804 17798 0 10720
row rollback_p jia jia_noS7 disk 8 2038 37672 37600 9528 10540 2014 0 804 17798 0 10720
row rollback_c plain bing_rec view 8 2058 42632 42560 3992 0 3992 0 0 16732 0 0
row rollback_c plain bing_rec disk 8 2058 42632 42560 3992 0 3992 0 0 16732 0 0
row rollback_c plain bing_s1 view 8 2058 42632 42560 3992 0 3992 0 0 16808 0 0
row rollback_c plain bing_s1 disk 8 2058 42632 42560 7335 5864 3651 0 0 13124 0 0
row rollback_c plain yi_s3 view 8 2058 42632 42560 3992 0 3992 0 0 13960 0 0
row rollback_c plain yi_s3 disk 8 2058 42632 42560 7335 5864 3651 0 0 10276 0 0
row rollback_c jia jia_s view 8 1790 37480 37408 3032 0 2944 0 692 16136 0 0
row rollback_c jia jia_s disk 8 1790 37480 37408 6990 5518 2444 0 692 10376 0 0
row rollback_c jia jia_noS6 view 8 1790 37480 37408 4506 0 2640 0 3344 26544 20724 0
row rollback_c jia jia_noS6 disk 8 1790 37480 37408 4465 0 2213 0 3344 16880 20724 0
row rollback_c jia jia_noS7 view 8 1790 37480 37408 11998 12994 2200 0 692 14328 0 17056
row rollback_c jia jia_noS7 disk 8 1790 37480 37408 11998 12994 2200 0 692 14328 0 17056
row rollback_c jia_both jia_s view 8 2058 42632 42560 3648 0 3560 0 388 20912 0 0
row rollback_c jia_both jia_s disk 8 2058 42632 42560 8501 6809 2907 0 388 13859 0 0
```

```
row crash_a plain bing_rec view 7 26958 1074128 2039984 57560 5072 52856 0 0 645424 0 0
row crash_a plain bing_rec disk 7 26958 1074128 2039984 57560 5072 52856 0 0 645424 0 0
row crash_a plain bing_s1 view 7 26958 1074128 2039984 57560 5072 52856 0 0 449428 0 0
row crash_a plain bing_s1 disk 7 26958 1074128 2039984 57560 5072 52856 0 0 449428 0 0
row crash_a plain yi_s3 view 7 26958 1074128 2039984 57560 5072 52856 0 0 199300 0 0
row crash_a plain yi_s3 disk 7 26958 1074128 2039984 57560 5072 52856 0 0 199300 0 0
row crash_a jia jia_s view 7 26958 1074128 2039984 60924 5072 49948 0 11272 25776 0 0
row crash_a jia jia_s disk 7 26958 1074128 2039984 64888 11656 48692 0 11272 18460 0 0
row crash_a jia jia_noS6 view 7 26958 1074128 2039984 357916 5072 74230 0 342008 343992 394664 0
row crash_a jia jia_noS6 disk 7 26958 1074128 2039984 357916 5072 66818 0 342008 257140 394664 0
row crash_a jia jia_noS7 view 7 26958 1074128 2039984 343148 463720 46832 0 11272 329940 0 460864
row crash_a jia jia_noS7 disk 7 26958 1074128 2039984 343148 463720 46832 0 11272 329940 0 460864
row pack3 plain bing_rec view 8 144 918 908 0 0 0 0 0 418 0 0
row pack3 plain bing_rec disk 8 144 918 908 0 0 0 0 0 418 0 0
row pack3 plain bing_s1 view 8 144 918 908 0 0 0 0 0 366 0 0
row pack3 plain bing_s1 disk 8 144 918 908 0 0 0 0 0 366 0 0
row pack3 plain yi_s3 view 8 144 918 908 0 0 0 0 0 328 0 0
row pack3 plain yi_s3 disk 8 144 918 908 0 0 0 0 0 328 0 0
row pack3 jia jia_s view 8 144 918 908 0 0 0 0 0 286 0 0
row pack3 jia jia_s disk 8 144 918 908 14 14 0 0 0 272 0 0
row pack3 jia jia_noS6 view 8 144 918 908 156 0 35 0 156 414 156 0
row pack3 jia jia_noS6 disk 8 144 918 908 156 0 25 0 156 362 156 0
row pack3 jia jia_noS7 view 8 144 918 908 104 104 0 0 0 362 0 104
row pack3 jia jia_noS7 disk 8 144 918 908 104 104 0 0 0 362 0 104
row pack3 jia jia_s_p1 view 8 144 918 908 142 0 0 142 0 418 0 0
row pack3 jia jia_s_p1 disk 8 144 918 908 142 0 0 142 0 418 0 0
row clone3 plain bing_rec view 7 63718 1723124 2432176 164214 22374 155620 0 0 743864 0 0
row clone3 plain bing_rec disk 7 63718 1723124 2432176 164214 22374 155620 0 0 743864 0 0
row clone3 plain bing_s1 view 7 63718 1723124 2432176 164214 22374 155620 0 0 598350 0 0
row clone3 plain bing_s1 disk 7 63718 1723124 2432176 164214 22374 155620 0 0 598350 0 0
row clone3 plain yi_s3 view 7 63718 1723124 2432176 164214 22374 155620 0 0 245926 0 0
row clone3 plain yi_s3 disk 7 63718 1723124 2432176 164214 22374 155620 0 0 245926 0 0
row clone3 jia jia_s view 7 63718 1723124 2432176 168028 19470 156352 0 37262 141300 0 0
row clone3 jia jia_s disk 7 63718 1723124 2432176 166744 32750 137074 0 37262 120076 0 0
row clone3 jia jia_noS6 view 7 63718 1723124 2432176 392024 19510 165480 0 298006 377018 347536 0
row clone3 jia jia_noS6 disk 7 63718 1723124 2432176 391306 19584 150975 0 298006 290726 347536 0
row clone3 jia jia_noS7 view 7 63718 1723124 2432176 387788 356498 134230 0 37262 344694 0 348760
row clone3 jia jia_noS7 disk 7 63718 1723124 2432176 387788 356498 134230 0 37262 344694 0 348760
row clone_p plain bing_rec view 7 44268 1350864 1941360 127456 0 137696 0 0 854924 0 0
row clone_p plain bing_rec disk 7 44268 1350864 1941360 127456 0 137696 0 0 745412 0 0
row clone_p plain bing_s1 view 7 44268 1350864 1941360 127456 0 137696 0 0 840584 0 0
row clone_p plain bing_s1 disk 7 44268 1350864 1941360 127456 0 137696 0 0 731072 0 0
row clone_p plain yi_s3 view 7 44268 1350864 1941360 127456 0 137696 0 0 576204 0 0
row clone_p plain yi_s3 disk 7 44268 1350864 1941360 127456 0 137696 0 0 466692 0 0
row clone_p jia jia_s view 7 44268 1350864 1941360 128564 0 137408 0 9864 434148 0 0
row clone_p jia jia_s disk 7 44268 1350864 1941360 128696 2336 134496 0 9864 355566 0 0
row clone_p jia jia_noS6 view 7 44268 1350864 1941360 157262 0 136666 0 42908 408120 48012 0
row clone_p jia jia_noS6 disk 7 44268 1350864 1941360 157004 0 134274 0 42908 337296 48012 0
row clone_p jia jia_noS7 view 7 44268 1350864 1941360 195192 76668 134464 0 9864 398370 0 78162
row clone_p jia jia_noS7 disk 7 44268 1350864 1941360 195192 76668 134464 0 9864 398370 0 78162
```

### 怎么读这张表

- **丙′-记录优先（`bing_rec`）**：`view` 与 `disk` 两行逐字相同——记录容器没坏时它选的值就是记录里的值，两种读者读法看到的是同一份。它在 `copy_r` 与 `pack3` 两个世界零失败；别的世界里它的失败全部落在三条臂共同的格上（墓碑丢了、实例表读不出、祖先表读不出），见下面「共同前提上的洞」。
- **丙′-S1（`bing_s1`）与乙′（`yi_s3`）**：`view` 行与丙′-记录优先同数，`disk` 行在跑协议 C 的三个世界里多出一批 `missing`——打中 1。
- **甲-S（`jia_s`）**：`view` 行在孤儿世界里多出一批 `split_new`（`rollback_a` 17964、`crash_a` 11272、`clone3` 37262、`clone_p` 9864、`rollback_p` 804、`rollback_c` 692）——打中 3；`disk` 行另外多一批 `missing`——打中 2（= 第二轮打中 B / C 在 S7 的最弱读法下回来）。
- **`jia_s_p1`** 只在 `pack3` 跑：142 个 `mac_fail`，同世界 `jia_s` 的两行都是 0——打中 4。
- **`jia_noS6` / `jia_noS7`** 是去掉 S6 / S7 的对照，它们的 `s6_red` / `s7_red` 非 0（第六节的判别力自证）；**`jia_both`** 是去掉 S8 的对照，`copy_r` 的 `view` 行 60 个 `split_new`，而带 S8 的 `jia` 家族同一世界 0。
- 三个阳性对照都非 0：墓碑丢了之后 `tomb_a` / `tomb_c` 全部臂的 `misread`、实例表读不出之后 `rollback_*` / `crash_a` 全部臂的 `misread`、`pack3` 的 `jia_s_p1` 的 `mac_fail`。阴性对照：同一批历史在墓碑在、表读得出时，丙′-记录优先与甲-S 都 0（手造 C1 阴性对照、G1 / G2 的 intact 行，第三节末）。

### 分辨臂：与丙′-记录优先逐格比

`common` 行是「这一格上每条臂都失败」的格数，`dist` 行是逐格与丙′-记录优先（同一读法）比。`output.txt` 的 `common` 行整行抄：

```
common copy_r view patterns=6732 all_arms_fail=0
common copy_r disk patterns=6732 all_arms_fail=0
common tomb_a view patterns=1916 all_arms_fail=296
common tomb_a disk patterns=1916 all_arms_fail=296
common tomb_c view patterns=20128 all_arms_fail=702
common tomb_c disk patterns=20128 all_arms_fail=683
common rollback_a view patterns=1327760 all_arms_fail=98548
common rollback_a disk patterns=1327760 all_arms_fail=98548
common rollback_p view patterns=37672 all_arms_fail=2240
common rollback_p disk patterns=37672 all_arms_fail=2242
common rollback_c view patterns=42632 all_arms_fail=2010
common rollback_c disk patterns=42632 all_arms_fail=2044
common crash_a view patterns=1074128 all_arms_fail=54652
common crash_a disk patterns=1074128 all_arms_fail=54652
common pack3 view patterns=918 all_arms_fail=0
common pack3 disk patterns=918 all_arms_fail=0
common clone3 view patterns=1723124 all_arms_fail=147728
common clone3 disk patterns=1723124 all_arms_fail=147728
common clone_p view patterns=1350864 all_arms_fail=123912
common clone_p disk patterns=1350864 all_arms_fail=123844
```

`dist` 行（只抄与判臂有关的五条臂；`jia_noS6` / `jia_noS7` 是收严的对照，数在 `output.txt` 里）：

```
dist copy_r plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r plain bing_s1 disk fails_where_bing_rec_ok=576 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r plain yi_s3 disk fails_where_bing_rec_ok=576 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r jia jia_s view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r jia jia_s disk fails_where_bing_rec_ok=444 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r jia_both jia_s view fails_where_bing_rec_ok=60 ok_where_bing_rec_fails=0 both_fail=0
dist copy_r jia_both jia_s disk fails_where_bing_rec_ok=816 ok_where_bing_rec_fails=0 both_fail=0
dist tomb_a plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=342
dist tomb_a plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=342
dist tomb_a plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=342
dist tomb_a plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=342
dist tomb_a jia jia_s view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=342
dist tomb_a jia jia_s disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=342
dist tomb_c plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=1060
dist tomb_c plain bing_s1 disk fails_where_bing_rec_ok=1458 ok_where_bing_rec_fails=0 both_fail=1060
dist tomb_c plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=1060
dist tomb_c plain yi_s3 disk fails_where_bing_rec_ok=1458 ok_where_bing_rec_fails=0 both_fail=1060
dist tomb_c jia jia_s view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=844
dist tomb_c jia jia_s disk fails_where_bing_rec_ok=1274 ok_where_bing_rec_fails=52 both_fail=792
dist tomb_c jia_both jia_s view fails_where_bing_rec_ok=120 ok_where_bing_rec_fails=0 both_fail=1060
dist tomb_c jia_both jia_s disk fails_where_bing_rec_ok=2078 ok_where_bing_rec_fails=77 both_fail=983
dist rollback_a plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=103096
dist rollback_a plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=103096
dist rollback_a plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=103096
dist rollback_a plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=103096
dist rollback_a jia jia_s view fails_where_bing_rec_ok=8544 ok_where_bing_rec_fails=3844 both_fail=99252
dist rollback_a jia jia_s disk fails_where_bing_rec_ok=13740 ok_where_bing_rec_fails=4420 both_fail=98676
dist rollback_p plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=2996
dist rollback_p plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=2996
dist rollback_p plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=2996
dist rollback_p plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=2996
dist rollback_p jia jia_s view fails_where_bing_rec_ok=128 ok_where_bing_rec_fails=726 both_fail=2270
dist rollback_p jia jia_s disk fails_where_bing_rec_ok=136 ok_where_bing_rec_fails=726 both_fail=2270
dist rollback_c plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=3992
dist rollback_c plain bing_s1 disk fails_where_bing_rec_ok=3343 ok_where_bing_rec_fails=0 both_fail=3992
dist rollback_c plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=3992
dist rollback_c plain yi_s3 disk fails_where_bing_rec_ok=3343 ok_where_bing_rec_fails=0 both_fail=3992
dist rollback_c jia jia_s view fails_where_bing_rec_ok=88 ok_where_bing_rec_fails=432 both_fail=2944
dist rollback_c jia jia_s disk fails_where_bing_rec_ok=4082 ok_where_bing_rec_fails=468 both_fail=2908
dist rollback_c jia_both jia_s view fails_where_bing_rec_ok=88 ok_where_bing_rec_fails=432 both_fail=3560
dist rollback_c jia_both jia_s disk fails_where_bing_rec_ok=5088 ok_where_bing_rec_fails=579 both_fail=3413
dist crash_a plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=57560
dist crash_a plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=57560
dist crash_a plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=57560
dist crash_a plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=57560
dist crash_a jia jia_s view fails_where_bing_rec_ok=6272 ok_where_bing_rec_fails=2908 both_fail=54652
dist crash_a jia jia_s disk fails_where_bing_rec_ok=10236 ok_where_bing_rec_fails=2908 both_fail=54652
dist pack3 plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 jia jia_s view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 jia jia_s disk fails_where_bing_rec_ok=14 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 jia jia_s_p1 view fails_where_bing_rec_ok=142 ok_where_bing_rec_fails=0 both_fail=0
dist pack3 jia jia_s_p1 disk fails_where_bing_rec_ok=142 ok_where_bing_rec_fails=0 both_fail=0
dist clone3 plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=164214
dist clone3 plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=164214
dist clone3 plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=164214
dist clone3 plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=164214
dist clone3 jia jia_s view fails_where_bing_rec_ok=14416 ok_where_bing_rec_fails=10602 both_fail=153612
dist clone3 jia jia_s disk fails_where_bing_rec_ok=15346 ok_where_bing_rec_fails=12816 both_fail=151398
dist clone_p plain bing_s1 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=127456
dist clone_p plain bing_s1 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=127456
dist clone_p plain yi_s3 view fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=127456
dist clone_p plain yi_s3 disk fails_where_bing_rec_ok=0 ok_where_bing_rec_fails=0 both_fail=127456
dist clone_p jia jia_s view fails_where_bing_rec_ok=2504 ok_where_bing_rec_fails=1396 both_fail=126060
dist clone_p jia jia_s disk fails_where_bing_rec_ok=2796 ok_where_bing_rec_fails=1556 both_fail=125900
```

### 打中 1：丙′-S1 与乙′ 在 R-disk 下找不到数据（3 步；丙′-记录优先同一段历史 0）

`output.txt` 的 `shortest` 与手造断言整行抄：

```
shortest copy_r plain bing_s1 disk depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_segment', 1))] tally=missing=1,misread=0,mac_fail=0,split_new=0
shortest copy_r plain yi_s3 disk depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_segment', 1))] tally=missing=1,misread=0,mac_fail=0,split_new=0
shortest rollback_c plain bing_s1 disk depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_segment', 1))] tally=missing=1,misread=0,mac_fail=0,split_new=0
shortest rollback_c plain yi_s3 disk depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_segment', 1))] tally=missing=1,misread=0,mac_fail=0,split_new=0
assert A1 丙′-记录优先：先抄后切抄了一份、丢首段 1 那片叶，R-view                                     got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert A1 同上 R-disk                                                             got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert A1 丙′-S1 R-view（第二轮模型的读法）                                                got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert A1 丙′-S1 R-disk：幸存 key 在 2、盘上记录说 1，找不到                                   got=(1, 0, 0, 0) want=(1, 0, 0, 0) ok
assert A1 乙′（S3）R-view                                                          got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert A1 乙′（S3）R-disk：同上，找不到                                                   got=(1, 0, 0, 0) want=(1, 0, 0, 0) ok
assert A2 翻记录之后丢首段 2 的叶：丙′-记录优先 R-disk                                          got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert A2 同上 丙′-S1 R-disk：幸存 key 在 1、记录说 2                                      got=(1, 0, 0, 0) want=(1, 0, 0, 0) ok
assert A2 同上 乙′ R-disk                                                          got=(1, 0, 0, 0) want=(1, 0, 0, 0) ok
```

历史：建对象（目录 1）→ 写 offset 0 → 先抄后切给 offset 0 在首段 2 抄一条 key（记录仍是 1）；损坏：丢首段 1 那片叶，记录容器完好。幸存的 key 只剩 (2, 1, 0)，S1 与 S3 按「幸存 extent key 优先」选 2，把这个对象的 key 全编到 2；而记录容器没坏，盘上那条权威记录还是 1 ⇒ 读者按 1 找不到数据。翻记录之后丢首段 2 那片叶是镜像对称的另一半（手造 A2）。

- **分不分辨臂**：分辨。丙′-记录优先在同一格选 1、把 key 全编到 1，读者找得到：`dist` 行里「它失败而丙′-记录优先不失败」的格 `copy_r` 576、`tomb_c` 1458、`rollback_c` 3343，反向一格没有。
- **系统在做决定那一刻看不看得到判别子**：看得到——幸存 key 的首段与那条完好的记录不等，重建那一刻两样都在手里；这两条臂是规则上选择不看记录。
- **满足哪条判据的哪个分句**：U1 触发观测第一个分句「读者按记录里的 `locality_id` 找不到某段数据」。
- **对乙′ 是按候选原文打中**：乙′ 的文本只写 key 的取值次序（S3），没写记录；记录完好时读者读的就是盘上那条。**对丙′-S1 是最弱读法打中**：它的文本写「给记录与这个对象的全部 extent key……编同一个首段」，而没写读者读视图还是读盘；按纪律 3，那个「给记录编」的值只能活在视图里。两条臂要站住都得补 S11（第八节），丙′-记录优先不需要。
- 第二轮把 S1 判成局部性偏好，是在 R-view 下量的；这一轮 R-disk 那一列把它变回正确性后果。按反向接受条款「丙′ 的某一种取值次序在 U1 被一个正确性后果打中 ⇒ 那一种取值次序出局」，出局的是 S1 那一种次序，记录优先那一种不受影响。

### 打中 2：甲-S 在 R-disk 下找不到数据（3 步；= 第二轮打中 B / C 在 S7 的最弱读法下回来）

```
shortest copy_r jia jia_s disk depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=1,misread=0,mac_fail=0,split_new=0
assert A3 甲-S 同一段历史 extent 全丢 R-view（S7 管用）                                     got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert A3 甲-S R-disk（= 第二轮打中 B 的记录优先读法）                                         got=(1, 0, 0, 0) want=(1, 0, 0, 0) ok
```

S7 写的是「重建出的记录取重建出来的首段集合（重建视图里的多值，不落盘），读者逐个试」。记录容器完好时没有「重建出的记录」这回事，按最弱读法读者读盘上那条 ⇒ 第二轮打中 B（先抄后切抄了一份）与打中 C（记录回退）原样回来：`copy_r` 444、`tomb_c` 1274、`rollback_c` 4082 个分辨得出臂的格（`dist` 行）。R-view 那一列全 0，所以这一条与打中 1 是同一件事的两端：**三条候选里只有丙′-记录优先不需要「读者取视图里的记录值」这条额外规定**。
三句：分辨臂（丙′-记录优先同格 0）；判别子看得到（记录与单元头不等）；满足第一个分句。

### 打中 3：甲-S 在实例表 / 祖先表读不出时把一个对象编进两个首段（5 步；丙′ 与字面分组同一段历史 0）

`output.txt` 的 `dist_shortest` 与手造断言整行抄：

```
dist_shortest rollback_a jia jia_s view depth=5 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,1) ; rekey(11,1,2) ; rollback(1)] containers=[(0, 'intact')] extents=[(1, ('lose_offset', 0))] table=unreadable
dist_shortest crash_a jia jia_s view depth=5 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,1) ; rekey(11,1,2) ; crash()] containers=[(0, 'intact')] extents=[(1, ('lose_offset', 0))] table=unreadable
dist_shortest clone3 jia jia_s view depth=5 head=21 history=[create(11,1) ; write(11,1,0) ; write(11,1,1) ; clone(11,21) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_offset', 0))] ancestor=unreadable
assert H 回退抛弃了一次重编、丢 offset 0 那片叶、{'table': 'unreadable'}：丙′-记录优先               got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 同上 甲-S R-view：写序更大、头里是 2 的那个单元胜出，一个对象两个首段                              got=(0, 0, 0, 1) want=(0, 0, 0, 1) ok
assert H 同上 甲去掉 S6：只往丢了的原 key (1, 1, 0) 上补，补回现行版本                               got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 阴性对照：表读得出时甲-S                                                          got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 崩溃丢了一次重编、丢 offset 0 那片叶、{'table': 'unreadable'}：丙′-记录优先                got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 同上 甲-S R-view：写序更大、头里是 2 的那个单元胜出，一个对象两个首段                              got=(0, 0, 0, 1) want=(0, 0, 0, 1) ok
assert H 同上 甲去掉 S6：只往丢了的原 key (1, 1, 0) 上补，补回现行版本                               got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 阴性对照：表读得出时甲-S                                                          got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 克隆之后 origin 重编、丢 offset 0 那片叶、{'ancestor': 'unreadable'}：丙′-记录优先       got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 同上 甲-S R-view：写序更大、头里是 2 的那个单元胜出，一个对象两个首段                              got=(0, 0, 0, 1) want=(0, 0, 0, 1) ok
assert H 同上 甲去掉 S6：只往丢了的原 key (1, 1, 0) 上补，补回现行版本                               got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert H 阴性对照：表读得出时甲-S                                                          got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
```

历史：建对象（目录 1）→ 写 offset 0 → 写 offset 1 → 重编到 2（甲把两个单元都重写，头里写 2）→ 回退到重编之前（或崩溃丢掉那次发布，或这一切发生在克隆的 origin 侧）；损坏：丢 offset 0 那片叶，记录容器完好，实例表（或祖先表）读不出。重编写出的那两个单元是被抛弃的，写序却比现行版本大；表读不出时它们判已发布，S6 按锚点择新把 offset 0 那个选中 ⇒ 这个对象的 key 一条在 2、一条在 1，而原镜像只有一个首段。内容没错（重编不改内容），错的是**一个对象两个首段**。

- **分不分辨臂**：分辨。丙′ 与乙′ 的重编不写单元，盘上根本没有这批「同内容、不同首段」的孤儿；字面分组（去掉 S6）只往丢了的那条原 key 上补，补回的是现行版本。三条对照在同一格上都是 0（手造 H 的五条断言）。
- **系统在做决定那一刻看不看得到判别子**：看不到。实例表读不出时，孤儿与现行件在盘上逐条相同（都判已发布，只差写序大小，而写序大的那个恰好是错的）；祖先表读不出时克隆头拿不到克隆点。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判别子观测不到」那一行，这一格**不能拿来判丙′ 与乙′**——但它分辨得出甲：这批候选是甲自己的写路径造出来的，另两条臂根本没有。
- **满足哪条判据的哪个分句**：U1 触发观测第二个分句「同一个对象被编进两个不同的 key 首段」。
- **它是 S6 换来的**：S6 修的是第二轮打中 A（字面分组把重编前的旧版当另一条 key 放回），代价是在「写序最大的那个件本不该胜」的那几格上把过期的首段带进来。两者不在同一段历史上，所以不是「S6 修错了」，是 S6 与字面分组各有各的死角。按反向接受条款「甲-S 在 U1 被一段分辨臂的历史打中 ⇒ 甲出局（第二轮已中一次，这一轮再中就是三轮里的多数）」，这一条与打中 4 都落在那一格上，照实交主 agent。

### 打中 4：甲-S 的打包规则按字面（p1）读时，没打包的 ≤ 4 KiB 对象 AAD 失配（2 步）

```
shortest pack3 jia jia_s_p1 view depth=2 head=11 history=[create(11,1) ; write(11,1,0)] containers=[(0, 'lost')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=1,split_new=0
assert D1 甲-S 打包规则读成 p1：没打包的 ≤ 4 KiB 对象、记录与 extent 都丢，AAD 失配                    got=(0, 0, 1, 0) want=(0, 0, 1, 0) ok
assert D1 同上读成 p2                                                               got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert D2 p1：重编之后记录容器回到上一版，AAD 失配                                               got=(0, 0, 1, 0) want=(0, 0, 1, 0) ok
```

候选文本：「打包对象：槽不带 locality（槽 +0），≤ 4 KiB 的对象重建时按丙′ 的规则把全部 key 编同一个值」。按字面，这句话罩住的是**所有** ≤ 4 KiB 的对象，包括还没被整理搬进容器、仍然是码 1 单元的那些；而它们头里的 locality 在甲下进了 AAD，一旦丙′ 的规则给出的值与单元头里的值不等（记录与 extent 都丢时取 0，记录回到上一版时取旧值），读路径的查找路径首段 ≠ 单元头里的 locality ⇒ AAD 失配。`pack3` 世界 142 个失败形态全是这一类，同世界 `jia_s`（读成 p2）两行都是 0。这是第二轮阴性对照 `jia_unify`（甲照丙重编首段必失配）落在小对象上的形态。

- **分不分辨臂**：分辨。丙′ 与乙′ 没有 AAD 绑这个字段，同一格 0。
- **判别子**：看得到（单元头里的值与查找路径的首段不等，那正是 AAD 要报的东西），但它报出来的是「读不了」，不是「重建规则选错了」。
- **分句**：不在 U1 触发观测的两个分句里（既不是找不到，也不是两个首段），是读路径失败，照实记。
- **修法**：把候选写成 p2（第八节 S12）。p2 在 R-view 下 0，但它自带一个要写明的并列：槽与打包前那个码 1 单元写序逐字节相同（`19-块指针的结构与宽度预算.md:173`），并列时取单元还是取槽——模型取单元（对「按单元头编」最弱的读法），于是 `pack3` 的 `jia_s` 在 R-disk 下有 14 个 `missing`（手造 E1）。

### 打中 5：甲去掉 S8 时，先抄后切期间一次写两份仍然出两个首段（`jia_both` 对照）

```
shortest copy_r jia_both jia_s view depth=7 head=11 history=[create(11,2) ; write(11,1,0) ; c_step(11,1,1) ; write(11,1,0) ; c_flip(11,1) ; write(11,1,1) ; c_drop(11,1)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,split_new=1
shortest rollback_c jia_both jia_s view depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,0) ; rollback(1)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] table=unreadable tally=missing=0,misread=1,mac_fail=0,split_new=0
assert S8 检查（迁移期间一次写在两个首段各写一个码 1 单元的次数）：(甲-S, 去掉 S8)                            got=(0, 1)     want=(0, 1)     ok
```

`copy_r` 的 `jia_both`（去掉 S8）`view` 行 60 个 `split_new`，带 S8 的 `jia` 家族同一世界 0；`rollback_c` 是 388 对 692（那个世界 `jia` 家族另有孤儿带来的 692，两者不是同一批格）。⇒ 第二轮的 S8 这一轮又被攻了一轮，它修的那一格还在，去掉它就复现。

### S6 与字面分组各修各的：两段方向相反的历史

```
assert C1 被抛弃的时间线重编后又写、实例表读不出：丙′-记录优先读到被抛弃的版本                                   got=(0, 1, 0, 0) want=(0, 1, 0, 0) ok
assert C1 同上 甲-S R-view（S6 让写序最大的孤儿胜出）                                          got=(0, 1, 0, 0) want=(0, 1, 0, 0) ok
assert C1 同上 甲去掉 S6（字面分组）：两个首段、读者按 1 读到现行版本                                     got=(0, 0, 0, 1) want=(0, 0, 0, 1) ok
assert C1 阴性对照：实例表读得出时甲-S                                                       got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert G1 截掉 offset 1、重编、再写回 offset 1，墓碑 intact：丙′-记录优先                         got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert G1 同上 甲-S                                                                got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert G1 同上 甲去掉 S6（被截掉的旧单元按头里的 1 自成一组放回）                                       got=(0, 0, 0, 1) want=(0, 0, 0, 1) ok
assert G1 截掉 offset 1、重编、再写回 offset 1，墓碑 lost：丙′-记录优先                           got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert G1 同上 甲-S                                                                got=(0, 0, 0, 0) want=(0, 0, 0, 0) ok
assert G1 同上 甲去掉 S6（被截掉的旧单元按头里的 1 自成一组放回）                                       got=(0, 1, 0, 1) want=(0, 1, 0, 1) ok
```

- **只在 S6 之下才会出的那一段（C1）**：被抛弃的时间线上先重编到 2、又写了一次 offset 0（内容变了），回退之后实例表读不出、extent 全丢、记录完好。S6 按锚点择新选中写序最大的那个孤儿 ⇒ 读到被抛弃时间线上的内容；字面分组把两版分进两个首段，读者按记录里的 1 读到的是现行版本（代价是两个首段）。⚠️ 同一格丙′-记录优先也读到被抛弃的内容（`assert C1 … 丙′-记录优先读到被抛弃的版本 got=(0, 1, 0, 0)`）⇒ 这一段**不分辨甲-S 与丙′**，它分辨的是 S6 与字面分组。
- **方向相反的那一段（G1）**：截掉 offset 1、重编到 2、再把 offset 1 写回来，墓碑容器丢了。字面分组把被截掉的那个旧单元按它头里的 1 自成一组放回，读者先试 1、读到被删掉的版本；S6 按锚点择新选中最新那版，内容对。
- ⇒ S6 不是「更对」也不是「更错」，它换了一组死角。条款只能取一种，取哪种要连这两段历史一起写进去。

### 共同前提上的洞（三条臂一起中，不拿来判臂）

1. **墓碑丢了**：三条臂一起读到被删掉 / 被截掉的版本（`tomb_a` 296、`tomb_c` 702 个每条臂都失败的格；墓碑在时同一批历史全 0，手造 G2 两行）。这是 E59（缓冲消息能不能只从单元重算） 那条老结论的形态，与 locality 这一维无关。
2. **实例表读不出**：被回退或被崩溃抛弃的那段时间线判已发布，写序更大的孤儿在择新里胜出 ⇒ 读到被抛弃时间线上的数据（`rollback_a` 98548、`crash_a` 54652、`rollback_p` 2240、`rollback_c` 2010 个共同格）。`18-块里携带什么信息.md:879` 那句自己写了这一格的处置（只读挂载、扫描重建停在级 1），这一轮把它量了出来：它不止是「记不记歧义」，是**读得到被抛弃时间线上的内容**。第一轮的 G 这一轮坐实了，形状与那一轮一致。
3. **祖先表读不出**：克隆头认领 origin 在克隆点之后写的件，也会施加 origin 在克隆点之后写的墓碑（墓碑记录只有对象 ID + 对象出生代 + 区间，不带树）⇒ 读到别的头的数据，或者自己的数据被别人的墓碑杀掉（`clone3` 147728、`clone_p` 123912 个共同格；手造 F2 两条）。祖先表读不出时克隆头该认领什么、该施加谁的墓碑，全仓没写（第七节）。
4. **「丙′ 失败而甲-S 不失败」的那些格不是甲买到的判别力**。`output.txt` 的 `rdist_shortest` 行整行抄：

```
rdist_shortest rollback_a jia jia_s view depth=5 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,0) ; rollback(1) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] table=unreadable
rdist_shortest crash_a jia jia_s view depth=5 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,0) ; crash() ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] table=unreadable
rdist_shortest clone3 jia jia_s view depth=5 head=21 history=[create(11,1) ; write(11,1,0) ; clone(11,21) ; write(11,1,0) ; rekey(21,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] ancestor=unreadable
```

   读法：回退 / 克隆之后**又重编了一次**，甲把那个单元重写了一遍，新单元的写序比孤儿大 ⇒ 择新选中它、内容是对的；丙′ 不重写单元，择新只能在旧件与孤儿之间选，选中孤儿。甲在这几格逃掉靠的是「它刚好又写了一次」，不是它知道哪个是孤儿——同一个机制在打中 3 那几格上让它自己中。⇒ 按「判别子观测不到」，这一批格（`rollback_a` 3844、`crash_a` 2908、`clone3` 10602、`clone_p` 1396、`rollback_p` 726、`rollback_c` 432）不拿来判臂。

### 丙′-记录优先：在分辨得出臂的格上一次没被打中

- **零失败的两个世界**：`copy_r`（6732 个损坏形态、6728 次对象检查）与 `pack3`（918 / 908）——这两个世界里没有墓碑丢失、没有孤儿、没有祖先表问题，剩下的全是这一轮要攻的重建规则本身。
- **其余八个世界它的失败数与共同格的差**：`tomb_a` 342 对 296、`tomb_c` 1060 对 702、`rollback_a` 103096 对 98548、`rollback_p` 2996 对 2240、`rollback_c` 3992 对 2010、`crash_a` 57560 对 54652、`clone3` 164214 对 147728、`clone_p` 127456 对 123912。差出来的那些格，逐个落在上面第 4 条那个机制上（某条甲的读法碰巧又写了一次、或者字面分组把孤儿编到另一个首段去、读者看不见它），`dist` 行的 `ok_where_bing_rec_fails` 就是它们。
- **它付的只有局部性**：`row` 行最后第三列 `locality_changed`（重建后视图里的记录值与原值不同的对象次数），例如 `copy_r` 2896 次 / 6728 次对象检查、`rollback_a` 750912 / 2407936。重建产物本来就只读（`18-块里携带什么信息.md:509` 整行「| 1 | 只有校验和 / MAC 自洽，没有分配记录 | **只读挂载** |」）。
- **这一轮试过的形状**：删除、截断到 offset 1（都写区间墓碑，墓碑容器 在 / 丢）；管理员回退（回退 1 或 2 次发布，实例表 读得出 / 读不出）；崩溃（最后一次发布没落、txg 与 inode 号重发，实例表 读得出 / 读不出）；打包、整容器重写、打包之后重编、打包之后写（搬出去）、打包对象的删除；克隆之后两个头各自重编（协议 A 与协议 P）、克隆之后一个头截断而另一个头没有（祖先表 读得出 / 读不出）；三种重编协议（A / P / C）与两种写路径（甲重写单元 / 不重写）；S6、S7、S8 各自的去掉对照；读者两种读法。
- **取样范围**：每个对象 offset 0 与 1、locality 取 1 与 2（`pack3` 只有 offset 0），每容器 2 条记录（真值 233），记录容器最多回退 2 版，回退最多 1 次、崩溃最多 1 次，2 个头。`tomb_a` 与 `pack3` 在操作上限内穷举完，另外八个世界到 `coverage` 行写的那个深度。**这些都是小取样点，不是真值**；第二轮的射程注照旧成立。

## 四、U2 契约

U2 的触发观测是「一条臂与一条已定条款逐字冲突、而没有改那条条款的计划」。这一轮新找到的几处都不是逐字冲突，而是候选文本没写、不写就在 U1 被打中；各自落在哪条条款旁边，写在下面。

### 丙′-S1 与前提十四（纪律 3）

- 纪律 3（`18-块里携带什么信息.md:521`，第一节已整行抄）只管「就地修复」，S1 的重建值只活在视图里，按字面不撞。
- 撞的是读路径：记录完好、幸存 key 与它不一致时，S1 给出的值 ≠ 盘上那条权威记录。读者读视图（R-view）就 0，读盘（R-disk）就找不到（第三节打中 1）。候选原文「给记录与这个对象的全部 extent key……编同一个首段」没写读者读哪一份。
- 出路两条：① 补一句「读者取重建视图里的记录值」（我记为 S11，只在本模型量过，被攻过零轮），它多要求一条读路径规定——一个完好的权威字段在读路径上被重建值盖住；② 撤回 S1、取记录优先，不多要求任何东西。第二轮把 S1 判成局部性偏好，是在 R-view 下量的；R-disk 下它是正确性要求的反面。

### 乙′ 与前提十一

- 进密文与已定项 13 同形，第二轮已判，这一轮没变。
- 乙′ 原文只写 key 的取值次序（S3：幸存 extent key > inode 记录 > 单元头提示 > 0），没写记录。按字面，记录完好时读者读的就是盘上那条——R-disk 是乙′ 的**字面**读法，不是最弱读法。所以第三节打中 1 在乙′ 上是按候选原文打中。收严：次序改成「inode 记录 > 幸存 extent key > 单元头提示 > 0」（= 丙′-记录优先，外加两样都丢时用头提示代替 0），或补 S11。

### 甲-S 与前提五

这一轮没有新的条款证据。第一、二轮的判法不变：前件取决于 `08-核心索引结构.md:198` 那个动作取哪种定义，归辩方腿。

### 甲-S 与前提十五（打包对象）

- 候选原文「≤ 4 KiB 的对象重建时按丙′ 的规则把全部 key 编同一个值」按字面（p1）也罩住**还没打包**的 ≤ 4 KiB 对象，而它们是码 1 单元、头里的 locality 进了 AAD：一旦丙′ 给的值 ≠ 单元头里的值，读路径 AAD 失配（第三节打中 4，两步）。那就是第二轮 `jia_unify` 的形状落在小对象上。
- 读成 p2（只罩被选中的版本是槽的）在 R-view 下 0；但候选文本得写明两件事：对象「是不是 ≤ 4 KiB / 是不是打包对象」在重建那一刻从哪个字段知道；槽与打包前那个码 1 单元写序逐字节相同（`19-块指针的结构与宽度预算.md:173`），两者并列时取哪个——取单元时首段是它头里的旧值，R-disk 下找不到（第三节 E1）。
- 槽 +0 与 `27-小数据打包容器.md:379` 自洽：槽里没有 locality，不需要被 AAD 绑。

### 甲-S 与前提十七（I-1.8）

- S6 要把码 1 的「同 key」写成不含 locality，是改 I-1.8（`invariants.md:28`）的文字，第二轮已记。
- 这一轮新找到的是 S6 的代价：它让写序最大的可见件胜出、不看头里的 locality。写序最大的件本不该胜的时候（实例表或祖先表读不出），它把一个过期的首段带进重建（第三节打中 3）；字面分组在同一段历史上只往丢了的原 key 上补，补回的是现行版本。S6 修的是第二轮打中 A，打中 3 是它换来的，两者不在同一段历史上。

### 丙′-记录优先

零契约变更。记录完好时它选的值就是记录的值，视图与盘上那条一致，纪律 3 那条读路径问题不出现；记录坏了时视图里的记录本来就是重建产物。

## 五、U3 代价

字节与泄漏面第二轮第五节已算（甲 / 乙′ 每个码 1 单元 +8、第一个事务两盘各 +8、丙′ 0；加密开启后三条臂都不新增泄漏），这一轮没有新的字节数。这一轮新量到的是两笔操作代价，都只在甲-S 上：

- **重编 key 在盘上多留一批「同内容、不同首段」的码 1 单元。** 甲-S 不论加不加密都重写被重编的单元，旧单元留到被清扫为止；它们平时无害（写序更小），而一旦实例表或祖先表读不出、或者重编那一次发布本身被回退 / 崩溃抛弃，写序更大的那一批恰好是不该胜的那一批——第三节打中 3 的全部输入就是这批单元。乙′ / 丙′ 的重编不写单元，盘上没有这批东西。
- **一次重编 = 每个被重编单元一个事务**（`16-发布语义.md:185` 的切分纪律），被快照或克隆共享的还要拆开：各世界 `family` 行的 `forced_rewrites` / `unshared` 计数（第三节整行抄）；乙′ / 丙′ 两列恒 0。

U3 触发观测「写出一条臂在某个操作上要改写已发布单元」：甲-S 在「重编 key」下触发（与第一、二轮同条件），乙′ / 丙′ 不触发；「代价随盘容量涨」三条臂都不触发。

## 六、U4：S10 在各臂下判得了什么；S6–S8 各配什么会红的检查

### S10 会不会在合法镜像上判红、会不会把真错判成不可判定

手造两组（`output.txt` 整行抄）：

```
assert U4-1 bing_rec：(注入后常规读找不到, I-9.9 字面红) / 记录丢了之后读者 tally / S10              got=((1, 1), (0, 0, 0, 0), 'undecidable') want=((1, 1), (0, 0, 0, 0), 'undecidable') ok
assert U4-1 yi_s3：(注入后常规读找不到, I-9.9 字面红) / 记录丢了之后读者 tally / S10                 got=((1, 1), (0, 0, 0, 0), 'undecidable') want=((1, 1), (0, 0, 0, 0), 'undecidable') ok
assert U4-1 jia_s：(注入后常规读找不到, I-9.9 字面红) / 记录丢了之后读者 tally / S10                 got=((1, 1), (0, 0, 1, 0), 'undecidable') want=((1, 1), (0, 0, 1, 0), 'undecidable') ok
assert U4-2 合法的协议 P 中间态 vs 写路径 bug，记录都丢了：(key 集合, 按记录取自 key 的字面红, S10)          got=(((1, 1, 1), (2, 1, 0)), True, 'undecidable') want=(((1, 1, 1), (2, 1, 0)), True, 'undecidable') ok
assert U4-2 同上两者的 S10                                                           got=undecidable want=undecidable ok
assert U4-2 甲-S 同两幅镜像：AAD 失配数（合法, bug）——单元头替它作证                                 got=(0, 1)     want=(0, 1)     ok
```

- **U4-1（真错被抹掉）**：写路径 bug 把 key 编到 2、记录与单元头都说 1；注入后常规读找不到、I-9.9 字面红（未经重建时三条臂都判得出）。记录丢了、按幸存 key 重建之后，丙′ 与乙′ 读者都找得到了，bug 被静默抹掉，S10 判不可判定——现行 I-9.9 在这一格（extent 树没经重建、记录取自 key）按字面会给绿，S10 把它改成不可判定，是改好不是改坏。甲-S 同一格读路径 AAD 失配 1，bug 仍看得见（第二轮第六节同向）。
- **U4-2（真错与合法中间态分不开）**：协议 P 搬了一条的合法镜像，与一幅「把 offset 0 的 key 错编到 2」的 bug 镜像，记录都丢了之后，丙′ 看到的输入逐项相同（key 集合都是 (1, 1, 1)、(2, 1, 0)），按「记录取自 key」的字面两幅都红（一幅真红、一幅假红），S10 两幅都判不可判定。判别它们的只有丢掉的那条双值记录——**被判的系统在做决定那一刻看不到判别子**，所以 S10 判不可判定是它能给的最好答案，不是它的错。甲-S 在这两幅上 AAD 失配数是 0 与 1：单元头替它作证，这是甲在 U4 上多出的那一格，与第二轮同向。

### 穷举里 S10 判了什么

`s10` 行的列是「判定 / 这一格重建产物的下场」：`ok` 读者一格没出错，`loc_bad` 有找不到 / 两个首段 / AAD 失配，`content_bad` 只有内容读错（版本选错，I-9.9 这一维本来就看不见）。三个世界的行整行抄（全部 160 行在 `output.txt`）：

```
```

- **R-view 那一列，十个世界、每条臂，一个 `red` 都没有**。原因是结构性的：读者用视图里的记录值时，记录这一侧要么就是选中值（丙′ / 乙′），要么是重建出的首段集合（甲-S 的 S7），两边同源 ⇒ S10 一律判「不可判定」。**代价是它也看不见打中 3**：`jia_s` 的 `undecidable/loc_bad`（`tomb_a` 42、`rollback_a` 24572、`crash_a` 15976、`clone3` 56732、`clone_p` 9864）就是「两个首段」那一批格，S10 在那里说不可判定。⇒ 甲-S 的两个首段要被检查抓住，得另立一条（第二轮记的「甲配 I-9.14」那一格），I-9.9 按侧判罩不到它。
- **R-disk 那一列才有 `red`**，而且分两种：`red/loc_bad` 是真红（重建产物确实坏了，例如打中 1、打中 2、打中 4 那几批格）；`red/ok` 是**合法中间态上的假红**——记录是完好的那一条、key 是按各自规则编出来的，分批重编还没做完时本来就两侧不等（`rollback_p` 的 `bing_s1` / `yi_s3` 各 2924、`clone_p` 各 42068、`jia_s` 98928、`copy_r` 的 `jia_s` 192）。这正是第二轮记的三臂共同的洞 ①（分批重编的中间态让 I-9.9 在合法镜像上判红）在重建产物上的形态，**不是 S10 新引入的**：S10 只在两侧同源时把判定改成不可判定，两侧独立时它退回字面。
- ⇒ 回答任务书第 2 问：S10 **会**在合法镜像上判红，但红的那一格与洞 ① 是同一格，修洞 ① 就修了它；S10 **会**把一个真错判成不可判定，而 U4-2 那两幅镜像说明那一格的判别子在重建那一刻看不到，判不可判定是它能给的最好答案。另外要记一笔：S7 让甲-S 的重建记录成为多值，I-9.9 的「记录只许一个值」对甲-S 的重建产物恒不成立，按 S10 一律不可判定。

### S6、S7、S8 各配一条会红的检查，以及判别力自证

| 收严 | 检查（模型里的形态） | 落到哪 |
|---|---|---|
| S6 | 没有幸存 key 的锚点，重建只许补一条，而且是该锚点可见候选里择新键最大的那个（`s6_red`） | checker 的扫描方向：对重建产物逐 (对象, 锚点) 数放回的件、比择新键；I-1.8（`invariants.md:28`）码 1 那半的可执行形式 |
| S7 | 被重建过的对象，每条 key 的首段都在视图记录的集合里（`s7_red`） | 重建之后、只读挂载之前的自检：读者逐个试找得到每条 key |
| S8 | 迁移期间一次用户写在两个首段各写一个码 1 单元的次数（`double_writes`），非 0 判红 | 写路径计数，形态与 C319 那条「同一请求内取号序与 key 序不一致的次数」相同 |

判别力自证（`output.txt` 整行抄）：

```
assert S6 检查（没有幸存 key 的锚点只补一条、且是择新键最大的）：(甲-S, 去掉 S6)                            got=(0, 1)     want=(0, 1)     ok
assert S7 检查（重建过的对象每条 key 的首段都在视图记录里）：(甲-S, 去掉 S7)                              got=(0, 1)     want=(0, 1)     ok
assert S8 检查（迁移期间一次写在两个首段各写一个码 1 单元的次数）：(甲-S, 去掉 S8)                            got=(0, 1)     want=(0, 1)     ok
```

三条检查都只盯住「甲-S 有没有照 S6 / S7 / S8 做」，不盯「做出来的结果对不对」：第三节打中 3 那几格上 `jia_s` 的 `s6_red` 是 0（它确实把写序最大的那个件补了回去），而结果是一个对象两个首段。穷举里 `jia_s` 各世界的 `s6_red` / `s7_red` 与 `jia` 家族的 `double_writes` 见第三节 `row` 与 `family` 行。

## 七、各臂要改哪几句 kb（kb 文件自己的行号）

第二轮第七节那张表的各行这一轮都还成立，下面只写这一轮新加的与改动的。

| 臂 | 要改的句子 | 改成什么 |
|---|---|---|
| 丙′-记录优先 | `18-块里携带什么信息.md:398`–`:404`（那条提议）与 `:400`「放不回去」 | 提议不收；「放得回去：首段按 inode 记录 > 幸存 extent key > 0 重编，丢的是局部性」 |
| 丙′-记录优先 | `08-核心索引结构.md:206` | 「扫描重建时也没法重新编 key」改成「重编之后的值要有地方存」 |
| 丙′-记录优先 | D18（块里携带什么信息） 已定项 5，或 `08-核心索引结构.md:422` 那段「重建」 | 对象按 (树 ID, inode) 认、只看本头的 extent 树（S2）；这个对象全部 extent key 编同一个首段（S9）；次序记录 > 幸存 key > 0。记录完好时视图值 = 记录值，读者读哪一份不必另写 |
| 丙′-S1 | 同上 | 除 S2、S9 外还要写 S11（读者取视图里的记录值）；不写就在 R-disk 下被打中 1 |
| 乙′ | `18-块里携带什么信息.md:394`、`:396`、`:602`（第二轮已列） | 另把 S3 的次序改成「inode 记录 > 幸存 extent key > 单元头提示 > 0」，或写 S11 |
| 甲-S | `09-加密.md:524`、`:553`、`invariants.md:28`（第二轮已列） | 不变 |
| 甲-S | `27-小数据打包容器.md:281`（槽自带五元组那一条） | 写明槽不带 locality（槽 +0），打包规则写成 p2（S12）：被选中的是槽才按丙′ 的规则编；槽与打包前那个单元写序并列时取槽 |
| 甲-S | D18（块里携带什么信息） 已定项 5 的重建规则 | S6 + S7 + S8 之外，写明实例表或祖先表读不出时「一个对象两个首段」怎么报（打中 3 在这两格上是 S6 的必然结果） |
| 三臂共同 | `invariants.md:265`（I-9.9） | 第二轮的两处（分批中间态、S10）不变；第六节写 S10 的判别力 |
| 三臂共同 | `11-索引节点要不要留消息缓冲区.md:197` 或 D6（快照实现模型） | 祖先表读不出时克隆头的扫描重建认领什么、应用谁的墓碑——全仓没写（我在模型里取最宽的读法，三条臂一起中） |

## 八、我提的收严（都只在本模型上量过，被攻过零轮）

| # | 收严 | 修哪一格 | 本模型上的读数 |
|---|---|---|---|
| S11 | 读者取重建视图里的记录值：对象被重建过时，哪怕记录容器完好，读路径也用视图里的 locality，不用盘上那条 | 打中 1（丙′-S1、乙′ 在 R-disk 下找不到） | 各世界 `bing_s1` / `yi_s3` 的 `view` 行对 `disk` 行（第三节表格） |
| S12 | 甲-S 的打包规则写成 p2：被选中的版本是槽才按丙′ 的规则编；槽与打包前那个单元写序并列时取槽 | 打中 4（p1 下 AAD 失配）与 E1（并列取单元、R-disk 下找不到） | `pack3` 世界 `jia_s_p1` 对 `jia_s`；手造 D1 / D2 / E1 |

- 丙′-记录优先不需要这两条里的任何一条。
- S11 是一条读路径规定：一个完好的权威字段在读路径上被重建值盖住。纪律 3 按字面只管写盘，S11 不撞它；但它正是「重建结果替权威态报了一个不同的值」在读路径上的形态，第二轮攻方腿第四节已点过这层张力。
- 打中 3 我没给收严：它是 S6 在「写序最大的件不该胜」时的必然结果，而那几格（实例表 / 祖先表读不出）本来就是三条臂共同的洞；给甲-S 单独补一条「记录读得出时只收头里 locality ∈ 记录值集合的候选」能不能修、会不会重新打开第二轮打中 C，这一轮没量。
- 第二轮的 S6–S10 这一轮又被攻了一轮：S6 在打中 3 上付了代价（第三节）；S7 在 R-view 下零失败，R-disk 下就是第二轮打中 B / C；S8 的去掉对照在 `copy_r` 与 `tomb_c`、`rollback_c` 上非 0（第三节表格），带上 S8 时 `jia_s` 在 R-view 下没有一格因「一次写两份」出错；S9 在 R-view 下成立，R-disk 下只对记录优先成立；S10 见第六节。

## 九、什么现象会推翻每条结论

| 结论 | 推翻它的观测 |
|---|---|
| 打中 1：丙′-S1 与乙′ 在 R-disk 下找不到数据，丙′-记录优先同一段历史 0 | 一条条款写明重建之后读者一律读视图里的记录值（那就是采纳了 S11）；或写明协议 C 期间丢了一段 key 的叶时，重建按记录重编（那就是换回记录优先） |
| 乙′ 的 R-disk 是它的字面读法 | 乙′ 的候选文本补了一句「记录也取这个值、读者读视图」 |
| 打中 3：甲-S 在实例表 / 祖先表读不出时出现一个对象两个首段，字面分组与丙′ 同一段历史 0 | 一条条款写明表读不出时扫描重建不收旧实例 / origin 的件（那样三条臂都没有这批候选）；或甲-S 补一条「记录读得出时只收头里 locality ∈ 记录值集合的候选」并在模型里量出零失败且不重开第二轮打中 C |
| 打中 4：甲-S 的打包规则读成 p1 时 AAD 失配 | 候选文本写明只罩被选中的版本是槽的对象（S12） |
| 墓碑、孤儿、祖先表这三面的其余失败三条臂一起中 | 一段历史让丙′-记录优先失败而甲-S 与乙′ 都不失败，且病根在 locality 这一维（本模型里 `dist` 行的 `ok_where_bing_rec_fails` 就是这类格，第三节逐个说了它们为什么是碰巧） |
| 丙′-记录优先在这一轮九个世界里只在三臂共同的格上失败 | 一段合法历史让 `bing_rec` 在实例表、祖先表都读得出、墓碑也在的情况下找不到、读到旧版或出现两个首段 |
| S10 只在「按字面两侧独立而镜像处在合法中间态」时判红 | 一幅合法镜像在 S10 下判红，而它的重建里没有两侧独立的对象；或一个真错在 S10 下判不可判定，而判别它的东西在重建那一刻看得到（第六节 U4-2 说明这一格看不到） |
| S6 / S7 / S8 的检查分得出去掉那一条收严的对照 | 把检查放到去掉收严的对照上仍判绿（手造三条断言现在是红 1） |

## 十、复跑与指纹

```
nice -n 19 python3 -B research/prompts/d18-locality-r3-opus-model/model.py > /tmp/claude-1000/d18-locality-r3-opus/out.txt
diff /tmp/claude-1000/d18-locality-r3-opus/out.txt research/prompts/d18-locality-r3-opus-model/output.txt
```

- 默认深度，本机实测 **768 秒**（`nice -n 19`，跑之前与跑之后的 `date +%s` 之差）。`model.py N` 把全部世界改成深度 N，`model.py 0 世界名,世界名` 只跑这几个世界、不跑手造历史。只用 std 的 Python，没有 Rust、没有 target 目录。
- 确定性：无随机源、无 I/O，跑 N 遍与跑 1 遍信息量相同。输出里 **82 条 `assert … ok`、0 条 FAIL**（`grep -c`）。
- sha256：
  - `09a20142d5d7f32d2a029863766281c9754da7cae73db873af689c2fac99ed86  research/prompts/d18-locality-r3-opus-model/model.py`
  - `73857ad989e5eeeef062cc16136d92a183f352bfb071ec6abd37f5976406fbac  research/prompts/d18-locality-r3-opus-model/output.txt`
- **射程与没建的**：实例切换没建（它按新写序重做同一批事务，孤儿与重做件同内容、同首段、写序更小，碰不到 locality 这一维——这是推理，不是量出来的）；同一次发布里几个事务之间的中间态没建（`23-journal的角色与格式.md` 前缀判定第六条之后它不会成为恢复结果）；reflink、多于两个头、快照销毁都没建；**祖先表读不出时克隆头认领什么、施加谁的墓碑，条款没写，模型取的是最宽的那种读法，是我定的**；实例表读不出时的处置按 `18-块里携带什么信息.md:879` 那句读；每容器 2 条记录（真值 233）、每对象 2 个 offset、locality 两个值、回退与崩溃各至多 1 次是小取样点，不是真值；八个世界没在操作上限内穷举完（`coverage` 行）。
- **过程记录**：模型源码 1 次排他新建 + 10 次追加，每次都在 150 行以内；两处定点编辑修自己写出的语法错与未定义变量；一次用 `research/scripts/replace-batch.py` 做的 4 处定点替换（加 `rdist_shortest` 与三条手造断言），`--dry-run` 先核、写完回读一致。报告分段追加，每次都在 150 行以内（最大一次 81 行），第一段用 `set -o noclobber` 排他新建。`output.txt` 由 6 次 ≤ 150 行的追加拼出，与留存的那次运行输出 `diff` 逐字节相同。中途按 `ps` 列出的写死的 pid 停过两个跑到一半的探索进程（一次是换深度，一次是加 `rdist_shortest` 之后要重跑），没有按名字匹配杀过进程。草稿与临时文件都在 `/tmp/claude-1000/d18-locality-r3-opus/`。这一轮别的腿的产出一份都没打开；前两轮的模型目录只读过、没改、没拷。

## 十一、判决摘要（给主 agent）

- **丙′-记录优先**：这一轮换的六个面——删除与截断留在盘上的旧版与墓碑、管理员回退与崩溃留下的孤儿（实例表读得出 / 读不出）、打包之后重编 / 搬出去 / 整容器重写、克隆之后两个头各自重编同一个共享对象（祖先表读得出 / 读不出）、甲-S 的 S6–S8 本身、丙′ 两种取值次序本身——**没有一段分辨得出臂的历史打中它**。它的失败全落在三条臂一起中的格上；「丙′ 失败而某条甲的读法不失败」的那批格，靠的是甲刚好又重写了一次那个单元（第三节共同前提第 4 条），不是判别力。
- **丙′-S1**：在 R-disk 下被一段 3 步的分辨臂历史打中（打中 1）。按反向接受条款「丙′ 的某一种取值次序在 U1 被一个正确性后果打中 ⇒ 那一种取值次序出局」，出局的是幸存 key 优先这一种，除非补 S11（读者取视图里的记录值）。第二轮把 S1 判成局部性偏好，是在 R-view 那一种读者读法下量的。
- **乙′（S3）**：同一段历史按**候选原文**打中（乙′ 的文本只写 key 的取值次序、没写记录）。次序改成记录优先、或者补 S11 才站得住；改完它就是「丙′-记录优先 + 记录与 key 都丢时用头提示代替 0」。
- **甲-S**：这一轮被三段分辨臂的历史打中——打中 2（R-disk，S7 的最弱读法，等于第二轮打中 B / C 回来）、打中 3（实例表或祖先表读不出时，甲自己重编写出的孤儿把一个对象编进两个首段；5 步，丙′ 与字面分组同格 0）、打中 4（打包规则按字面读时，没打包的 ≤ 4 KiB 对象 AAD 失配；2 步）。加上第二轮那一次，**甲在 U1 上是三轮里的两轮被分辨臂的历史打中**。S6 与字面分组各有各的死角（C1 与 G1 两段方向相反的历史），取哪一种都要把另一段一起写进条款。
- **三臂共同、不拿来判臂**：墓碑丢了复活；实例表读不出时读到被抛弃时间线上的内容；祖先表读不出时克隆头认领什么、施加谁的墓碑全仓没写。三样都要各记一笔账。
- **U4**：S10 在 R-view 下一个红都没有（两侧同源），代价是它看不见甲-S 的两个首段；R-disk 下的假红与三臂共同的洞 ①（分批重编的中间态）是同一格。S6 / S7 / S8 各配了一条会红的检查，判别力自证在第六节。
- **按跑前条款**：丙′-记录优先在 U1 上没被分辨臂的正确性后果打中；丙′-S1 与乙′ 各被打中一次；甲-S 被打中三次。S11、S12 是我提的收严，只在本模型上量过、**被攻过零轮**。这条攻方腿没把丙′-记录优先打穿。
