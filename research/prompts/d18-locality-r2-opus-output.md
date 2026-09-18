# D18（块里携带什么信息） 已定项 3 那条提议（自描述头加 `locality_id`） 第二轮：攻方腿（Opus）报告

立场：反推 / 攻。假设丙′（倾向）是错的，也假设甲、乙′ 各有死角；换第一轮没攻过的面：分批重新聚簇的中间态、D27（小数据打包容器） 的槽、加密开启时的重建路径、「后台重新聚簇」的两种定义、克隆之后的重新聚簇。
模型：`research/prompts/d18-locality-r2-opus-model/model.py`（只用 std 的 Python，没有 Rust、没有 target 目录；第一轮的 `research/prompts/d18-locality-r1-opus-model/` 只读过、没改、没拷），原始输出同目录 `output.txt`。复跑命令与 sha256 在第十节，判决摘要在第十一节。
引 kb 一律写 kb 文件名加那份文件自己的行号，行号 2026-09-14 在 `.claude/kb/` 下现查，不从背景材料里数。这一轮别的腿的产出一份都没打开。

## 一、失败条款先核：前提表一至十六对 kb 原文

十六条逐条对过附录与 kb 原文，都对得上，失败条款不触发。两处写下来备查。

**前提六这一轮改对了。** `14-双轨大小文件-持久临时.md:281` 起那条被判掉的候选是「判据取 D8（核心索引结构） 已定项 3 的 `locality_id`」，`:291` 那句「加密开启时它新增第一处……泄漏」的「它」指这条候选；`:295`–`:297` 整三行：

> ⚠️ 而 `locality_id` 至今**没有**盘上的明文表达（D18（块里携带什么信息） 已定项 3 的自描述头五元组不含它，
> 加它的提议 2026-09-03 起待用户定案且逐字「动它是一次永久契约变更」）
> ⇒ 按 locality 聚簇等于不加那个字段就把同一个信号搬进物理排布。

前提六写的正是「按 locality 聚簇」这条候选的问题，主语没再换。

**前提十五最后一分句比原文宽一格，推得出，不触发。** `27-小数据打包容器.md:384` 写的是「一个对象连同它的槽被搬到另一个容器时 AAD 变了，要重新加密」（容器到容器）。从码 1 单元打包进容器也要重加密，是同一条机制推出来的：码 1 的 AAD 是五元组（`09-加密.md:524`），容器的 AAD 绑容器身份（`27-小数据打包容器.md:381`），AAD 变了就得重加密。

**前提表之外、这一轮承重而材料里没有的条款**（按 `.claude/rules/three-way-inference.md`「正文没点名的枢纽条款」列出，交主 agent 核）：

| 条款 | kb 文件与行号 | 为什么承重 |
|---|---|---|
| I-1.8（归并后版本全序）：「码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，同 key 的各组两两全序键不等——码 1 的键是写序」 | `invariants.md:28`（长行，只摘到这里，全文在那一行） | 扫描重建怎么择版本。甲把 locality 放进身份之后，「同 key」若读成「要放回去的那条 extent key」，重编前的旧版与重编后的新版不在一组 ⇒ 第三节打中 A |
| D16（发布语义） 已定项 5 的切分纪律：「一个事务最多写一个单元的用户数据」，`:186` 用户 2026-09-13 重申「切分纪律是一事务一单元」 | `16-发布语义.md:185`（长行，只指路）、`:186` | 甲的重编要重写单元，一个单元一个事务 |
| 有效 `T_dirty` 默认 256 MiB | `16-发布语义.md:181` | 一次发布装得下多少脏字节 ⇒ 多大的对象的重编必然跨发布（第五节） |
| D23（journal 的角色与格式） 前缀判定第六条：「施加的单位是一次发布」 | `23-journal的角色与格式.md:1224` | 同一次发布里的中间态不会成为恢复结果，发布之间的会；模型因此把「每一批 = 一次发布」的每个盘面都当成可达 |
| D27（小数据打包容器） 定死的五处：容器写出后不可变、改对象走「搬出去」、死槽标记不许住容器、回收死槽只能整个重写 | `27-小数据打包容器.md:240`–`:244` | 扫描读得到死槽与打包前那个码 1 单元，它们看上去都是活的 |
| D27（小数据打包容器） 已定项 7 / 9：「AAD 绑容器身份（取这条）」 | `27-小数据打包容器.md:381` | 甲把 locality 放进 AAD，罩的只是没打包的码 1 单元；槽里那 8 字节不被任何 AAD 绑 |
| E114（小数据打包容器的总账） 口径：容器头 107、每槽全额自描述 47 = 五元组 33 + 写序 10 + 槽表 4 | `27-小数据打包容器.md:69`、`experiments/114-小数据打包容器的总账.md:16` | 槽 +8 的代价（第五节） |
| D18（块里携带什么信息） 已定项 5 纪律 3：「一条纪律：该状态下**任何权威态结构不得由重建结果就地修复**」 | `18-块里携带什么信息.md:521` | S1（幸存 key 优先于记录）在两者不一致时让重建视图里的记录与盘上那条权威记录不同（第三节「丙′ 与乙′」末） |

## 二、模型怎么建的（口径）

在第一轮模型的形状上加了四样，其余口径同第一轮（对象 = (树 ID, inode)；记录住容器、每容器 2 条（真值 233），改记录就 COW 容器、旧版留在盘上）：

- **盘上保留每一个写出过的物理件**：被覆写的旧单元、甲重编时被换掉的旧单元、先抄后切抄出来的新单元、打包前那个码 1 单元、死槽、搬走前的旧落点，都能被扫描读到，都判已发布（写序 ≤ T_pub，`invariants.md:22` I-1.2（块头写序已发布））。归属当神谕：这个头的这个对象曾经引用过的全部物理件（C113 与克隆祖先表视为已解）。每写一个单元取一个新写序（一事务一单元）；槽与打包前那个单元同写序（`19-块指针的结构与宽度预算.md:173` 逐字节取）。
- **「后台重新聚簇」两种定义**：
  - **重编 key**，三种协议，每一步之间都发布。协议 A：一次发布里改记录与这个对象全部 key（对象装得进一次发布时）。协议 P：记录先带双值 (L_new, L_old)，每次发布搬一条 key，最后记录收成单值；读者按记录次序两个值都试。协议 C：先抄后切，每次发布给一个 offset 在新段加一条 key，抄齐之后一次发布翻记录，再每次发布删一条旧 key，记录始终单值。协议 N（记录直接换成 L_new、不带旧值）是阳性对照，常规读就该红。甲在三种协议里都按 AAD 重写被搬的单元（协议 C 抄出来的是新头的新单元），乙′ / 丙′ 不动单元。
  - **只搬物理位置**：物理件换落点、头一个字节不变（`26-后台整理与放置回收.md:214` 那句「搬迁走映射不改引用者」），locality 创建之后永不变。
- **打包**：对象只有 offset 0 那一个单元时可以打包进一个新容器，槽 +0 或 +8；可以整容器重写。甲另有三种槽政策：认领不比 locality（槽里那 8 字节成了提示）、认领比 locality 且重编时把对象搬出去、认领比 locality 而重编时不动槽（阳性对照）。
- **克隆**：同第一轮，另允许克隆之后在任一头里用协议 P 重编（有一个头在迁移时不许克隆）。
- **损坏**：目标头每个记录容器 在 / 丢 / 最新 k 版读不出（k = 1、2）；每个对象的 extent 条目 在 / 全丢 / 丢某一个首段的那片叶（对象有两个首段时）/ 丢最低或最高 offset 的全部条目。一个对象的条目一条没丢时，重建不往它的 key 空间里补东西；丢了一部分时，按单元头编 key 的臂只往丢了的那几条原 key 上补（对甲最有利的读法：重建知道丢的是哪几片叶的区间）；全丢时按单元头能放回的都放回。

**十条重建臂**（前五条跑在「乙′ / 丙′ 的写路径」上，后五条跑在「甲的写路径」上；协议 A 那个世界另把甲的五条跑在「不重写单元」的写路径上，代表加密关着、没有 AAD 核它的卷）：

| 臂 | 首段怎么编 | 版本分组键 | 记录取什么 | 对应 |
|---|---|---|---|---|
| `bing_s1` | 这个对象全部 key（幸存的与找回的）编同一个值 | 锚点偏移 | 幸存 key 的众数 > 记录 > 0 | 丙′（S1 + S2），候选原文 |
| `bing_rec` | 同上 | 锚点偏移 | 记录 > 幸存 key > 0 | 丙′ 把 S1 换回「记录优先」 |
| `bing_r1lit` | 幸存 key 原样，找回的编选中值 | 锚点偏移 | 记录 > key > 0 | 第一轮丙的字面读法（阳性对照：第一轮打中 1） |
| `yi_s3` | 全部 key 编同一个值 | 锚点偏移 | key > 记录 > 头提示 > 0 | 乙′（S3），候选原文 |
| `yi_unit` | 按单元头编 | extent key | 记录 > 头 > 0 | 第一轮乙的逐单元读法（对照） |
| `jia_key_rec` | 按单元头编（槽没带就编选中值） | extent key：I-1.8「同 key」按甲的字面读，头里的 locality 是 key 的一段 | 幸存记录；没有才取重建出的首段集合 | 甲字面 |
| `jia_key_der` | 同上 | extent key | 重建出的首段集合，读者逐个试 | 甲 + 记录不信幸存值 |
| `jia_anc_rec` | 同上 | 只按锚点偏移 | 幸存记录优先 | 甲 + 分组不看首段 |
| `jia_anc_der` | 同上 | 只按锚点偏移 | 重建出的首段集合 | 甲的最强读法（我给的收严，第八节） |
| `jia_unify` | 全部 key 编同一个值 | 锚点偏移 | key > 记录 > 0 | 阴性对照：甲照丙重编首段，AAD 必失配 |

**检查**：读者按重建后记录里的 locality 次序去找，第一个找到的算数。找不到（`missing`）、读到的内容不是真值（`misread`：被覆写掉的旧版或死槽，即「读到损坏前（在这个位置）没有的数据」）、甲在加密开着时 AAD 失配（`mac_fail`：查找路径的首段 ≠ 码 1 单元头里的 locality）、槽认领失败（`claim_fail`）、一个对象两个首段（`split_new`：原镜像只有一个首段而重建出两个），任一非 0 记一次失败。另数局部性变化（重建后记录的值集与原值集不同，只丢局部性，不记失败）与冲突数（同一个 extent key 两个内容不同的现行件，按写序择新）。没有损坏时按原记录走一遍常规读（协议本身对不对），并按字面判一遍 I-9.9（locality 三值对照）。
这是确定性模型，跑 N 遍与跑 1 遍信息量相同；证据强度来自手造历史里钉死的绝对值（第三节 A–H、第六节 U4、第五节 U3，全部 `assert … ok`）与三个阳性对照（协议 N 常规读红、甲的槽认领比 locality 而不动槽时常规读红、第一轮丙字面读法在打中 1 上红）、一个阴性对照（甲照丙重编首段 AAD 必失配）。

## 三、U1 重建：各臂最短的反例

### 世界与覆盖

十个世界，每个世界的每条历史 × 每个家族 × 每个头 × 全部损坏形态 × 每条臂都重建一遍再检查。`output.txt` 的 `coverage` 行整行抄（`histories_one_step_longer=0` 表示在这个世界的操作上限之内穷举完了）：

```
coverage atomic max_depth=7 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage perunit max_depth=8 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage naive max_depth=8 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage copy max_depth=9 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage copy_recopy max_depth=10 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage physical max_depth=7 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage perunit2 max_depth=7 histories_one_step_longer=36464 (0 = 在这个世界的操作上限之内穷举完了)
coverage pack max_depth=8 histories_one_step_longer=0 (0 = 在这个世界的操作上限之内穷举完了)
coverage clone max_depth=7 histories_one_step_longer=260596 (0 = 在这个世界的操作上限之内穷举完了)
coverage v1 max_depth=7 histories_one_step_longer=378360 (0 = 在这个世界的操作上限之内穷举完了)
```

| 世界 | 「后台重新聚簇」取哪种定义、哪个协议 | 操作上限（每个对象 offset 0 / 1，locality 1 / 2，每容器 2 条记录） |
|---|---|---|
| `atomic` | 重编 key，协议 A（一次发布做完） | 1 个对象；写 ≤ 3；重编 ≤ 2 |
| `perunit` | 重编 key，协议 P（逐单元跨发布，记录带双值） | 1 个对象；写 ≤ 3；迁移 1 次 |
| `naive` | 协议 N（阳性对照：记录直接换值） | 同 `perunit` |
| `copy` | 重编 key，协议 C（先抄后切）；迁移期间的写两份 key 都改（甲写两份单元） | 1 个对象；写 ≤ 3；迁移 1 次 |
| `copy_recopy` | 同上，迁移期间的写只写主段、另一段作废重抄 | 同上 |
| `physical` | 只搬物理位置 | 1 个对象；写 ≤ 3；搬 ≤ 2 |
| `perunit2` | 协议 P，同一个记录容器里两个对象 | 2 个对象；各写 ≤ 2；各迁移 1 次 |
| `pack` | 打包（槽 +0 / +8、整容器重写）+ 协议 A 或 P | 1 个对象、只写 offset 0；写 ≤ 2；打包 ≤ 2；重编 1 次 |
| `clone` | 克隆之后任一头用协议 P | 每头建 ≤ 1；写 ≤ 2；每头每对象迁移 1 次 |
| `v1` | 第一版：locality 恒 0，打包与克隆都开 | 每头建 ≤ 2；写 ≤ 2；打包 1 次 |

七个世界在操作上限内穷举完；`perunit2` / `clone` / `v1` 到深度 7，再多一步还有 36464 / 260596 / 378360 条历史没跑（上面 `coverage` 行）。家族：`plain` = 乙′ / 丙′ 的写路径（重编不动单元），`jia` = 甲的写路径（重编重写单元、加密开着按 AAD 核），`*_slot8` = 槽 +8，另外三种是甲的槽政策（第二节）。

### 全部结果行

`output.txt` 的 `row` 行整行抄。列：world family arm max_depth histories patterns objects failures missing misread mac_fail claim_fail split_new collisions locality_changed（`failures` 数的是「至少一个失败字段非 0」的损坏形态个数，`objects` 是被检查的对象次数）：

```
row atomic plain bing_s1 7 222 2516 2512 0 0 0 0 0 0 0 412
row atomic plain bing_rec 7 222 2516 2512 0 0 0 0 0 0 0 860
row atomic plain bing_r1lit 7 222 2516 2512 448 576 0 0 0 256 0 860
row atomic plain yi_s3 7 222 2516 2512 0 0 0 0 0 0 0 260
row atomic plain yi_unit 7 222 2516 2512 1212 1564 148 0 0 344 0 864
row atomic plain jia_key_rec 7 222 2516 2512 1124 1284 194 0 0 344 0 764
row atomic plain jia_key_der 7 222 2516 2512 676 252 202 0 0 344 0 428
row atomic plain jia_anc_rec 7 222 2516 2512 1020 1444 0 0 0 172 0 736
row atomic plain jia_anc_der 7 222 2516 2512 504 332 0 0 0 172 0 336
row atomic plain jia_unify 7 222 2516 2512 0 0 0 0 0 0 0 412
row atomic jia jia_key_rec 7 222 2516 2512 1148 876 102 0 0 700 0 832
row atomic jia jia_key_der 7 222 2516 2512 700 0 122 0 0 700 0 700
row atomic jia jia_anc_rec 7 222 2516 2512 640 1152 0 0 0 0 0 640
row atomic jia jia_anc_der 7 222 2516 2512 0 0 0 0 0 0 0 0
row atomic jia jia_unify 7 222 2516 2512 412 0 0 684 0 0 0 412
row perunit plain bing_s1 8 466 5676 5672 0 0 0 0 0 0 0 3148
row perunit plain bing_rec 8 466 5676 5672 0 0 0 0 0 0 0 3640
row perunit plain bing_r1lit 8 466 5676 5672 1268 1528 0 0 0 520 0 3640
row perunit plain yi_s3 8 466 5676 5672 0 0 0 0 0 0 0 2972
row perunit plain yi_unit 8 466 5676 5672 3592 4408 132 0 0 848 0 3584
row perunit jia jia_key_rec 8 466 5676 5672 2306 1944 198 0 0 1212 0 2960
row perunit jia jia_key_der 8 466 5676 5672 1218 0 242 0 0 1212 0 2196
row perunit jia jia_anc_rec 8 466 5676 5672 1456 2400 0 0 0 0 0 2888
row perunit jia jia_anc_der 8 466 5676 5672 0 0 0 0 0 0 0 1776
row perunit jia jia_unify 8 466 5676 5672 1732 0 0 2320 0 0 0 3148
row naive plain bing_s1 8 466 5676 5672 0 0 0 0 0 0 0 1578
row naive jia jia_anc_der 8 466 5676 5672 0 0 0 0 0 0 0 1368
row copy plain bing_s1 9 618 7932 7928 0 0 0 0 0 0 0 2852
row copy plain bing_rec 9 618 7932 7928 0 0 0 0 0 0 0 3396
row copy plain bing_r1lit 9 618 7932 7928 1388 1884 0 0 0 336 0 3396
row copy plain yi_s3 9 618 7932 7928 0 0 0 0 0 0 0 2464
row copy plain yi_unit 9 618 7932 7928 3972 5112 276 0 0 432 0 3208
row copy jia jia_key_rec 9 618 7932 7928 1864 1732 96 0 0 684 0 4364
row copy jia jia_key_der 9 618 7932 7928 696 0 96 0 0 684 0 6060
row copy jia jia_anc_rec 9 618 7932 7928 2556 3632 0 0 0 108 0 3720
row copy jia jia_anc_der 9 618 7932 7928 288 180 0 0 0 108 0 4158
row copy jia jia_unify 9 618 7932 7928 4324 0 0 6346 0 0 0 2852
row copy_recopy plain bing_s1 10 638 7836 7832 0 0 0 0 0 0 0 2730
row copy_recopy plain bing_rec 10 638 7836 7832 0 0 0 0 0 0 0 3204
row copy_recopy plain bing_r1lit 10 638 7836 7832 1320 1784 0 0 0 336 0 3204
row copy_recopy plain yi_s3 10 638 7836 7832 0 0 0 0 0 0 0 2342
row copy_recopy plain yi_unit 10 638 7836 7832 3764 4996 124 0 0 408 0 3104
row copy_recopy jia jia_key_rec 10 638 7836 7832 1886 1660 210 0 0 740 0 4168
row copy_recopy jia jia_key_der 10 638 7836 7832 796 0 234 0 0 740 0 5860
row copy_recopy jia jia_anc_rec 10 638 7836 7832 2232 3472 0 0 0 0 0 3416
row copy_recopy jia jia_anc_der 10 638 7836 7832 0 0 0 0 0 0 0 3632
row copy_recopy jia jia_unify 10 638 7836 7832 3334 0 0 5494 0 0 0 2730
row physical plain bing_s1 7 370 2644 2640 0 0 0 0 0 0 0 368
row physical plain bing_rec 7 370 2644 2640 0 0 0 0 0 0 0 368
row physical plain bing_r1lit 7 370 2644 2640 0 0 0 0 0 0 0 368
row physical plain yi_s3 7 370 2644 2640 0 0 0 0 0 0 0 0
row physical plain yi_unit 7 370 2644 2640 0 0 0 0 0 0 0 0
row physical jia jia_key_rec 7 370 2644 2640 0 0 0 0 0 0 0 0
row physical jia jia_key_der 7 370 2644 2640 0 0 0 0 0 0 0 0
row physical jia jia_anc_rec 7 370 2644 2640 0 0 0 0 0 0 0 0
row physical jia jia_anc_der 7 370 2644 2640 0 0 0 0 0 0 0 0
row physical jia jia_unify 7 370 2644 2640 368 0 0 660 0 0 0 368
row perunit2 plain bing_s1 7 15130 434808 855464 0 0 0 0 0 0 0 537532
row perunit2 plain bing_rec 7 15130 434808 855464 0 0 0 0 0 0 0 538424
row perunit2 plain bing_r1lit 7 15130 434808 855464 110896 124072 0 0 0 15760 0 538424
row perunit2 plain yi_s3 7 15130 434808 855464 0 0 0 0 0 0 0 475540
row perunit2 plain yi_unit 7 15130 434808 855464 246184 294824 1084 0 0 9876 0 477288
row perunit2 jia jia_key_rec 7 15130 434808 855464 104040 73564 1606 0 0 39300 0 274648
row perunit2 jia jia_key_der 7 15130 434808 855464 39300 0 1686 0 0 39300 0 306436
row perunit2 jia jia_anc_rec 7 15130 434808 855464 78856 87768 0 0 0 0 0 284832
row perunit2 jia jia_anc_der 7 15130 434808 855464 0 0 0 0 0 0 0 336552
row perunit2 jia jia_unify 7 15130 434808 855464 250044 0 0 335496 0 0 0 537532
row pack plain bing_s1 8 202 1320 1316 0 0 0 0 0 0 0 598
row pack plain bing_rec 8 202 1320 1316 0 0 0 0 0 0 0 726
row pack plain bing_r1lit 8 202 1320 1316 182 182 0 0 0 0 0 726
row pack plain yi_s3 8 202 1320 1316 0 0 0 0 0 0 0 528
row pack plain yi_unit 8 202 1320 1316 656 310 72 0 0 346 0 732
row pack plain_slot8 bing_s1 8 202 1320 1316 0 0 0 0 0 0 0 598
row pack plain_slot8 bing_rec 8 202 1320 1316 0 0 0 0 0 0 0 726
row pack plain_slot8 bing_r1lit 8 202 1320 1316 182 182 0 0 0 0 0 726
row pack plain_slot8 yi_s3 8 202 1320 1316 0 0 0 0 0 0 0 528
row pack plain_slot8 yi_unit 8 202 1320 1316 656 450 58 0 0 206 0 732
row pack jia jia_key_rec 8 202 1320 1316 720 158 108 0 0 562 0 758
row pack jia jia_key_der 8 202 1320 1316 562 0 131 0 0 562 0 722
row pack jia jia_anc_rec 8 202 1320 1316 230 230 0 0 0 0 0 710
row pack jia jia_anc_der 8 202 1320 1316 0 0 0 0 0 0 0 486
row pack jia jia_unify 8 202 1320 1316 180 0 0 180 0 0 0 598
row pack jia_slot8_claim5 jia_key_rec 8 202 1320 1316 608 186 87 0 0 422 0 732
row pack jia_slot8_claim5 jia_key_der 8 202 1320 1316 422 0 103 0 0 422 0 724
row pack jia_slot8_claim5 jia_anc_rec 8 202 1320 1316 306 306 0 0 0 0 0 652
row pack jia_slot8_claim5 jia_anc_der 8 202 1320 1316 0 0 0 0 0 0 0 422
row pack jia_slot8_claim5 jia_unify 8 202 1320 1316 180 0 0 180 0 0 0 598
row pack jia_slot8_moveout jia_key_rec 8 190 1236 1232 654 146 81 0 0 508 0 674
row pack jia_slot8_moveout jia_key_der 8 190 1236 1232 508 0 96 0 0 508 0 652
row pack jia_slot8_moveout jia_anc_rec 8 190 1236 1232 292 292 0 0 0 0 0 588
row pack jia_slot8_moveout jia_anc_der 8 190 1236 1232 0 0 0 0 0 0 0 324
row pack jia_slot8_moveout jia_unify 8 190 1236 1232 358 0 0 232 126 0 0 574
row pack jia_slot8_claim6_leave jia_anc_der 8 202 1320 1316 128 0 0 0 128 0 0 422
row clone plain bing_s1 7 52121 900030 1183264 0 0 0 0 0 0 0 576772
row clone plain bing_rec 7 52121 900030 1183264 0 0 0 0 0 0 0 577732
row clone plain bing_r1lit 7 52121 900030 1183264 98520 105468 0 0 0 13448 0 577732
row clone plain yi_s3 7 52121 900030 1183264 0 0 0 0 0 0 0 424932
row clone plain yi_unit 7 52121 900030 1183264 215396 242324 592 0 0 5296 0 426480
row clone jia jia_key_rec 7 52121 900030 1183264 67448 47824 880 0 0 24212 0 251472
row clone jia jia_key_der 7 52121 900030 1183264 24224 0 932 0 0 24212 0 306796
row clone jia jia_anc_rec 7 52121 900030 1183264 51752 56392 0 0 0 0 0 257424
row clone jia jia_anc_der 7 52121 900030 1183264 0 0 0 0 0 0 0 324800
row clone jia jia_unify 7 52121 900030 1183264 322252 0 0 391072 0 0 0 576772
row v1 plain_slot8 bing_s1 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 plain_slot8 bing_rec 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 plain_slot8 bing_r1lit 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 plain_slot8 yi_s3 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 plain_slot8 yi_unit 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 jia_slot8_claim5 jia_key_rec 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 jia_slot8_claim5 jia_key_der 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 jia_slot8_claim5 jia_anc_rec 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 jia_slot8_claim5 jia_anc_der 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
row v1 jia_slot8_claim5 jia_unify 7 54924 1137941 1780792 0 0 0 0 0 0 0 0
```

读法：`bing_s1` / `bing_rec` / `yi_s3` 在每一行都是 0 失败，而每一行被检查的对象次数从 1316 到 1780792，不是空扫；三个阳性对照都非 0（`bing_r1lit`、`naive` 的常规读、`jia_slot8_claim6_leave`）；阴性对照 `jia_unify` 在有重编的世界里 AAD 失配非 0。甲的四种读法各自在某些世界里非 0，最强读法 `jia_anc_der` 只在 `copy` 一行非 0（288），`copy_recopy` 那一行是 0。

### 有失败的格：最短历史

`output.txt` 的 `shortest` / `normal_shortest` 行里与判臂有关的整行抄（其余格都是 `none_through_depth=N`；`jia_unify` 各世界的最短都是两步的阴性对照，不抄）：

```
shortest atomic plain bing_r1lit depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, ('rolled_back', 1))] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest atomic plain jia_key_rec depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest atomic plain jia_key_der depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,1) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_offset', 0))] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest atomic plain jia_anc_rec depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest atomic plain jia_anc_der depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,1) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_offset', 0))] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest atomic jia jia_key_rec depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest atomic jia jia_key_der depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest atomic jia jia_anc_rec depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, ('rolled_back', 1))] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest perunit plain bing_r1lit depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; p_begin(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest perunit jia jia_key_rec depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; p_begin(11,1,2) ; write(11,1,1)] containers=[(0, ('rolled_back', 1))] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest perunit jia jia_key_der depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; p_begin(11,1,2) ; m_step(11,1)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest perunit jia jia_anc_rec depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; p_begin(11,1,2) ; write(11,1,1)] containers=[(0, ('rolled_back', 1))] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest copy plain bing_r1lit depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_segment', 1))] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest copy jia jia_key_rec depth=5 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2) ; c_flip(11,1) ; write(11,1,1)] containers=[(0, ('rolled_back', 1))] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest copy jia jia_key_der depth=5 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2) ; c_flip(11,1) ; c_drop(11,1)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest copy jia jia_anc_rec depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest copy jia jia_anc_der depth=7 head=11 history=[create(11,2) ; write(11,1,0) ; c_step(11,1,1) ; write(11,1,0) ; c_flip(11,1) ; write(11,1,1) ; c_drop(11,1)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest copy_recopy plain bing_r1lit depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, ('lose_segment', 1))] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
shortest copy_recopy jia jia_key_rec depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2) ; write(11,1,0)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest copy_recopy jia jia_key_der depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2) ; write(11,1,0)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=0,misread=0,mac_fail=0,claim_fail=0,split_new=1
shortest copy_recopy jia jia_anc_rec depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; c_step(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=1,misread=0,mac_fail=0,claim_fail=0,split_new=0
normal_shortest naive plain depth=3 history=[create(11,1) ; write(11,1,0) ; n_begin(11,1,2)] tally=claim_fail=0,mac_fail=0,misread=0,missing=1
normal_shortest naive jia depth=3 history=[create(11,1) ; write(11,1,0) ; n_begin(11,1,2)] tally=claim_fail=0,mac_fail=0,misread=0,missing=1
normal_shortest pack jia_slot8_claim6_leave depth=4 history=[create(11,1) ; write(11,1,0) ; pack(11,1) ; rekey(11,1,2)] tally=claim_fail=1,mac_fail=0,misread=0,missing=0
```

### 打中 A：甲按字面分组——重编之前的旧版被当成另一条 key 放回来（3 步两个首段，4 步读到旧版；只打甲）

手造 A：建对象（目录 1）→ 写 offset 0 → 重编到 2（甲重写成新单元，旧单元留在盘上、判已发布）→ 再写 offset 0；损坏：extent 全丢，记录完好。按 I-1.8「同 key」的字面读法（放回去的 extent key = 头里的 locality + 锚点），旧单元（头里 1）自成一组、组内最新，被放回 (1, 1, 0)；最新的单元放回 (2, 1, 0)。重建出的记录取首段集合 (1, 2)，读者先试 1，读到重编之前那一版的内容。`output.txt` 手造 A 的断言：读到旧版 1、两个首段 1、记录 (1, 2)；记录优先的 `jia_key_rec` 读者按 2 读、读得对，两个首段照样 1；同一历史 `jia_anc_der` / `jia_anc_rec` / `bing_s1` / `bing_rec` / `yi_s3` 全 0。

- **分不分辨臂**：分辨。协议 A 世界 `jia` 家族 `jia_key_der` 700、`jia_key_rec` 1148 个失败形态，同一世界丙′ / 乙′ 0。
- **系统在做决定那一刻看不看得到判别子**：看得到。两个单元锚点相同、写序一新一旧，择新本来只要比写序；是分组键把它们分成了两组。
- **满足哪条判据的哪个分句**：U1 触发观测的第二个分句「同一个对象被编进两个不同的 key 首段」；读到旧版那一格是任务书第 1 问列的「读到损坏前没有的数据」，U1 触发观测的两个分句里没有这一类，照实记。
- **收严 S6（线索）**：甲的码 1 版本分组只看 (对象, 锚点偏移)，不看头里的 locality。`jia_anc_*` 就是它。

### 打中 C：甲记录优先——记录回到上一版，而甲不能把幸存 key 编回去（3 步；第一轮打中 1 落在甲身上的形状）

手造 C：建对象（1）→ 写 → 重编到 2；损坏：记录容器最新一版读不出、回到 1，extent 完好。丙′ 把幸存 key 编回记录的值 1，读者找得到（`bing_rec` 0，局部性变化 1）；甲不能这么编：幸存 key (2, 1, 0) 指的单元头里是 2，编到 1 下查找路径的首段与头不等、AAD 失配（`jia_unify` 那几行的 `mac_fail` 就是这个）。记录优先的甲读者按 1 找不到。断言：`jia_anc_rec` 找不到 1，`bing_r1lit`（第一轮丙的字面读法，幸存 key 原样）找不到 1，`jia_anc_der` / `bing_rec` / `bing_s1` / `yi_s3` 0。

- **分辨臂**：分辨。`jia_anc_rec` 在 `atomic` 640、`perunit` 1456、`copy` 2556、`pack` 230、`perunit2` 78856、`clone` 51752 个失败形态；丙′ / 乙′ 在同样的历史上 0。
- **判别子**：看得到（幸存 key 的首段与记录不等）。
- **分句**：第一个分句「读者按记录里的 `locality_id` 找不到某段数据」。
- **收严 S7（线索）**：甲重建出的记录取重建出来的首段集合（重建视图里的多值，不落盘），读者逐个试。`jia_anc_der` 在 `atomic` / `perunit` / `perunit2` / `pack` / `clone` / `copy_recopy` 全 0。丙′ 不需要它，因为丙′ 能把全部 key 编到一个值；甲不能，AAD 把每个单元钉在它头里的首段上。

### 打中 B：甲只按锚点分组、记录优先——先抄后切抄了一份（3 步）

手造 B：建对象（1）→ 写 offset 0 → 先抄后切给 offset 0 在 2 抄一份（甲抄出一个头里是 2 的新单元，记录仍是 1）；损坏：extent 全丢，记录完好。按锚点择新，抄出来的那份写序最新，只放回 (2, 1, 0)；记录说 1，读者找不到。断言：`jia_anc_rec` 找不到 1；`jia_anc_der` 0（记录 (2,)）；`jia_key_rec` 0（两份都放回）；丙′ / 乙′ 0。

- **分辨臂**：分辨。乙′ / 丙′ 的新 key 与旧 key 指同一个物理件，盘上没有第二个版本。
- **判别子**：看得到（记录说 1，重建出的 key 在 2）。
- **分句**：第一个分句。修法同 S7。

### 打中 B′：甲的最强读法（S6 + S7）——先抄后切期间一次写两份（7 步，只在一种写法下）

手造 B′：建对象（目录 2）→ 写 offset 0 → 给 offset 0 在 1 抄一份 → 再写 offset 0（这时 offset 0 有两条 key，甲给两段各写一个新单元，头 1 的写序在前、头 2 的在后）→ 翻记录到 1 → 写 offset 1 → 删旧段那条 key (2, 1, 0)。offset 0 的四个版本里写序最大的是头 2 那个，而它的 key 刚被删掉。损坏：丢 offset 0 那片叶。按锚点择新选中头 2 那个，它该放回的 (2, 1, 0) 不在读不出的区间里，不补；真正活着的头 1 那个比它旧，被择新淘汰——读者找不到。extent 全丢时它被放回 (2, 1, 0)，读者找得到（内容相同），但一个对象两个首段。断言：丢 offset 0 找不到 1、记录 (1,)；全丢 (找不到 0, 两个首段 1)；同一历史 `bing_s1` / `bing_rec` / `yi_s3` 两种损坏都 0。`copy` 世界 `jia_anc_der` 288 个失败形态（找不到 180、两个首段 108）。

- **分辨臂**：分辨。乙′ / 丙′ 一次写只写一个单元，两条 key 都指它。
- **判别子**：记录还在时，「记录说 1」看得到，但择新规则不看它；只看单元头与写序看不到——两个单元内容相同、都已发布，「哪一个的 key 被删了」不在任何单元上。记录丢了时哪里都看不到。这不是判据的错：歧义是甲自己一次写两份造出来的。
- **分句**：丢一片叶时第一个分句，全丢时第二个分句。
- **收严 S8（线索）**：先抄后切期间的写只写主段、另一段那条 key 作废重抄。`copy_recopy` 世界 638 条历史穷举完，`jia_anc_der` 0。

### 甲在不加密的卷上：头里的 locality 没有 AAD 核它

`atomic` 世界 `plain` 家族那五行甲（甲的五种读法跑在「重编不重写单元」的写路径上）全部非 0，最强读法 `jia_anc_der` 504 个失败形态（找不到 332）。最短：写 offset 0 → 写 offset 1 → 重编到 2 → 丢 offset 0 那片叶；offset 0 的单元头里还是 1，该放回的 (1, 1, 0) 不在读不出的区间（丢的是 (2, 1, 0)），读者找不到。

- 它说的是：加密关着（第一版，以及以后所有不加密的卷）时，「甲的单元头里的 locality 等于 key 首段」只靠写路径自己在重编时重写单元来维持，读路径上没有 AAD 核它。甲的条款要写成「不论加不加密，重编 key 都重写单元」，第五节的代价在不加密的卷上照付；盯着这条纪律的检查只有 I-9.9 按侧判的那一格（第六节）。
- 三句：分辨（同一写路径上丙′ / 乙′ 0）；判别子看得到不一致、分不出是合法过期还是坏了（第一轮 H′ 同形）；满足第一个分句。

### 丙′ 与乙′：没打中

`bing_s1`（丙′ 原文 + S1 + S2）、`bing_rec`（丙′ 记录优先）、`yi_s3`（乙′ + S3）在跑了它们的每个世界里全 0（`naive` 只跑 `bing_s1`，其余九个世界三条都跑）。被检查的对象次数：`atomic` 2512、`perunit` / `naive` 5672、`copy` 7928、`copy_recopy` 7832、`physical` 2640、`perunit2` 855464、`pack` 1316（两个家族各一遍）、`clone` 1183264、`v1` 1780792。

- **试过的形状**：重编 key 的三种协议（协议 C 两种写法）与协议 N 对照、只搬物理位置、同一容器两个对象、打包（槽 +0 / +8、整容器重写、打包之后重编、打包之后写即搬出去）、克隆之后任一头逐单元重编、第一版恒 0；损坏：记录容器 在 / 丢 / 回到上一版 / 回到上上版 × 每个对象的 extent 在 / 全丢 / 丢一个首段的那片叶 / 丢最低或最高 offset；盘上保留每一个写出过的物理件（旧版、被抄的、打包前的、死槽、搬走前的旧落点）。
- **取样范围**：每对象 offset 0、1，locality 1、2（第一版 0），每容器 2 条记录（真值 233）；七个世界在操作上限内穷举完，三个到深度 7。
- **丙′ 付的只有局部性**：`atomic` 2512 次对象检查里 `bing_s1` 412 次重建后 locality 与原值不同，`yi_s3` 260 次，甲最强读法 0（它不重编首段）。乙′ 比丙′ 多保住的局部性：`perunit` 3148 → 2972、`clone` 576772 → 424932、`physical` 368 → 0。重建产物本来就只读（`18-块里携带什么信息.md:509`）。

### 共同前提上的洞（三条臂一起中，不拿来判臂）

1. **分批重编的中间态让 I-9.9 在合法镜像上判红。** 手造 E：写完 / 开始 / 搬一条 / 搬两条 / 结束 五次发布上 I-9.9 字面判红的对象数 (0, 1, 1, 1, 0)。`family` 行的 `i99_legal_red`：`perunit` 232、`copy` 360、`copy_recopy` 352、`pack` 54、`perunit2` 16164、`clone` 39256，两个家族逐个相等。协议 N（记录直接换值，不带旧值）常规读就找不到还没搬的那条（`naive` 世界 100 条历史常规读失败，手造 F）。⇒ 按「重编 key」读，不论哪条臂，都要先定分批协议（一次发布装得下的对象走协议 A，装不下的走 P 或 C），并让 I-9.9 能判多首段的中间态。第一轮第三节末那一格的推理这一轮坐实了。
2. **被抛弃时间线的孤儿**（第一轮 G）这一轮没建，结论不变。

### 加密开启时的重建路径

- 已定项 13 之后，码 1 的五元组在密文里（`18-块里携带什么信息.md:602`–`:604`），甲那里的第六个字段、乙′ 进密文的提示都跟着进；加密开启时无密钥的一侧「只读只检测，既不搬也不回收」（`03-空间分配.md:17`）。⇒ 三条臂的重建都发生在有密钥的一侧，locality 不添新的密钥需求，也不添新的明文泄漏（乙原文那种明文提示除外）。
- 重建路径上甲的 AAD 是自证的（第四节甲第 3 条），读到的与「乙′ + 进密文」相同。模型里甲的家族读路径按查找路径核 AAD：`jia_anc_der` 在每个世界 `mac_fail` 0（按单元头编 key，自然对得上）；照丙′ 重编首段的 `jia_unify` 在 `atomic` 684、`perunit` 2320、`copy` 6346、`perunit2` 335496、`clone` 391072 次 AAD 失配——这就是甲用不了丙′ 那条修法的原因。
- 槽：容器 AAD 绑容器身份（第四节甲第 2 条），加不加密槽里的 locality 都不被绑；`jia_slot8_claim5` 家族（槽 +8、认领不比它）在 `pack` 世界的读数与 `jia` 家族同形。
- 这一面没有打出新的 U1 失败形状：加密只改变谁能重建（有密钥的一侧），以及甲的错在常规读路径上以 AAD 失配的形态出现（第六节 U4）。

## 四、U2 契约

### 甲

**前提三与前提五不互相顶着**（第一轮第四节已论证，两句管的不是同一对东西），**承重的是一个没定义的动作**。这一轮把那个动作的两种定义各跑了一遍：

- 按「只搬物理位置」读：物理件换落点、头一个字节不变，locality 创建之后永不变。`physical` 世界在操作上限内穷举完，甲被迫重写的单元 0 个、五条读法 0 失败（第三节表格与手造 H）。这时 D9（加密） 那条排除的机制没有输入：没有一个合法操作会不重写单元就改掉「单元 ↔ 首段」这一对。
- 按「重编 key」读：甲每搬一条 key 就重写一个单元（手造 E：两单元对象重写 2 个），先抄后切还要多抄一份新头的单元；乙′ / 丙′ 0 个。

⇒ 跑前反向接受条款「甲在 U2 被前提五打中、且前提五的理由今天仍然成立 ⇒ 甲出局」的前件，仍然取决于 `08-核心索引结构.md:198` 那个动作取哪种定义；这一轮没把它变成无条件的，前件怎么读归辩方腿。`09-加密.md:553` 整行：

> | **`locality_id`** | **D8（核心索引结构） 已定项 3 逐字**：「`locality_id` 是提示，不是正确性的一部分——它可以是错的、可以过期」 | 让声明为「可以是错的」的字段承重 |

**这一轮新找到三处。都不是逐字冲突，是甲的候选文本没写、不写就被打中的：**

1. **I-1.8 的「同 key」要改。** `invariants.md:28` 按「类身份段全部字段含写序」归组，「同 key 的各组」按写序择新。甲把 locality 放进类身份段之后，「同 key」若按甲的字面读成「要放回去的那条 extent key」，重编前的旧版与重编后的新版不在一组，重建两个都放回（打中 A）。要让甲过，I-1.8 得写明码 1 的版本分组不看 locality（S6）。
2. **D27 的容器 AAD 罩不到它。** `27-小数据打包容器.md:379` 整行：

   > | AAD 绑**对象**身份 + 整容器一份 MAC | **机制性排除**：N 个 AAD 对 1 个 MAC ⇒ N−1 个对象验不过。这正是 D9（加密） 已定项 6 排除 extent 切分时用的那条机制，逐字「失配成为正常操作的必然结果 ⇒ 排除」 |

   ⇒ 槽里的 locality 不可能被 AAD 绑。≤ 4 KiB 的对象上，甲那 8 字节要么不带（按丙′ 的规则重建），要么带成一个不被绑的提示（按乙′ 的规则）；候选写的「扫描重建时直接从单元头读 key 首段」配上「它进 AAD」，只对没打包的码 1 单元成立。
3. **重建路径上 AAD 不替它作证。** `18-块里携带什么信息.md:428` 整行：

   > | **重建**（索引没了） | **必须参与**——没有查找路径，AAD 只能由块头提供 | 单改字段会 MAC 失配；但**整单元搬运（头 + 密文一起搬）验证完全通过**，而那正是 A6（AAD 期望值不来自明文侧） 要挡的「块级重排」 |

   ⇒ 重建时甲读到的 locality 是这个单元自己头里（解密出来）的值。MAC 过只说明它没被单独改过，不说明它等于这个对象现在的首段（打中 A 的旧版就是一个 MAC 完全正确、首段已经过期的单元）。重建路径上甲与「乙′ + 进密文」拿到的是同一样东西；甲比乙′ 多的只在常规读路径：查找路径的首段 ≠ 单元头里的值时 AAD 失配（第六节 U4 那一格）。

另：甲把一个取值规则还没写的字段（`08-核心索引结构.md:209`）冻进第 1 层（`15-格式冻结政策.md:58`）与永久 AAD（`09-加密.md:552` 那一行的「AAD 的字段组成是 day-1 永久契约」），第一轮已记，这一轮没变。

### 乙′

- 进密文与已定项 13 同形：`18-块里携带什么信息.md:602`–`:604` 那条注只要把 locality 列进「就地变密文」那一串。
- 要改 `18-块里携带什么信息.md:394`：头里的字段从此不等于 AAD 那一组。
- I-1.8 同样要补一句：乙′ 的提示住类身份段，副本逐字节相同不影响归组，但「同 key」不许把它算进去——按单元头编 key 的对照 `yi_unit` 在协议 A 世界 1212 次失败，`yi_s3` 0 次（第三节表格）。
- 不进 AAD，没有 D9（加密） 冲突。

### 丙′

- 零契约变更，要改的是事实性措辞（第七节）。
- **一处要写明的取舍：S1 与 `18-块里携带什么信息.md:521` 那条纪律。** S1（幸存 key 优先于记录）在两者不一致时，重建视图里的记录值与盘上那条权威记录不同。那条纪律整行是「| 3 | 一条纪律：该状态下**任何权威态结构不得由重建结果就地修复** | …」，视图不写盘，按字面不撞；但它是「重建结果替权威态报了一个不同的值」。把 S1 换回「记录优先」（`bing_rec`）在跑了它的九个世界里同样 0 失败，因为丙′ 本来就把这个对象的全部 key 重编到同一个值，差别只剩局部性：协议 A 世界 `bing_s1` 局部性变化 412 次、`bing_rec` 860 次（同样 2512 次对象检查）。⇒ **S1 不是正确性要求，是局部性偏好**。第一轮判它必需，是因为第一轮模型没把幸存 key 也重编；这一轮的 `bing_r1lit` 对照（幸存 key 原样）复现了那一击（手造 C）。

### 三条臂与冻结分层（前提七）

| 层 | 甲 | 乙′ | 丙′ |
|---|---|---|---|
| 第 1 层（块头，`15-格式冻结政策.md:58`） | +8，而且进 AAD（加密开机那天起永久） | +8 | 不动 |
| 第 3 层（key 编码，`:62`） | 不动 | 不动 | 不动 |
| inode 记录组件（`:78`，冻结时刻是第一个写出类型 2 容器的镜像） | 不动 | 不动 | 不动 |

「重编 key」按协议 P 分批时，三条臂的常规读都要记录带双值，只能从记录那 20 字节预留里取（`08-核心索引结构.md:405`），赋义是 incompat，而且落在那个第一个镜像就冻结的组件上；协议 C 不要这个字段。这是「重编 key」这个定义自带的账，不分辨臂。

## 五、U3 代价

| 项 | 甲 | 乙′ | 丙′ |
|---|---|---|---|
| 每个码 1 单元 | 类身份段 105 → 113，含预留位的头 134 → 142（`18-块里携带什么信息.md:592` 的 `DATA_UNIT_HEADER_BYTES = 105`、`:596` 五元组 33 那一行） | 同甲 | 0 |
| AAD 规范编码 | 33 → 41 | 不变 | 不变 |
| 打包容器的槽 | 要保住「重建读头里的值」就得 +8，而那 8 字节不被 AAD 绑（`27-小数据打包容器.md:379`）；不加，≤ 4 KiB 的对象按丙′ 的规则走 | 可选 +8，按 S3 排最后，只在记录与 key 都丢时有用 | 0 |
| 第一个事务 | 数据单元头 +8，两盘各一份，五元组之后的偏移整体后移（`layout/01-first-txn.md:172`–`:176` 及其后各行） | 同甲 | 0 |
| 第一版这 8 字节带的信息 | 0 bit（`v1` 世界 54924 条历史全 0 失败、全 0 局部性变化） | 0 bit | — |
| 重编 key | 每搬一条 key 重写一个 32 KiB 单元、换新 nonce（手造 E：两单元对象 2 个）；先抄后切在翻记录之前两份单元并存 | 0 个单元，只改 key 与记录 | 0 个单元 |
| 克隆之后重编 | 还要拆开共享：`clone` 世界 `jia` 家族 `unshared=1950`（`forced_rewrites=7424`） | 0 | 0 |
| 打包对象的重编 | 认领比 locality 时要把对象搬出去：`pack` 世界 `jia_slot8_moveout` 家族 `move_outs=52`（每个 ≤ 4 KiB 对象换一个 32 KiB 码 1 单元，之后再打包）；认领不比它时槽里的值过期（`jia_slot8_claim5` 家族 `stale_hints=64`） | 槽 +8 时提示过期（`plain_slot8` `stale_hints=158`），不重写 | 0 |
| 只搬物理位置 | 0（`physical` 世界 `forced_rewrites=0`） | 0 | 0 |
| 重建规则 | S6 + S7，协议 C 时再加 S8；重建视图里记录多值 | S3 | 全部 key 编一个值，S1 或记录优先 |
| 加密开启后的新泄漏 | 0（第六个字段在密文里） | 0（进密文） | 0 |

### 打包容器槽要不要跟着 +8

E114（小数据打包容器的总账） 口径推算（没重跑 E114），`output.txt` 的断言整行抄：

```
assert U3   64 字节对象：一个容器装几个（今天, 槽 +8）                                     got=(294, 274) want=(294, 274) ok
assert U3  512 字节对象：一个容器装几个（今天, 槽 +8）                                     got=(58, 57) want=(58, 57) ok
assert U3 1024 字节对象：一个容器装几个（今天, 槽 +8）                                     got=(30, 30) want=(30, 30) ok
assert U3 4096 字节对象：一个容器装几个（今天, 槽 +8）                                     got=(7, 7)   want=(7, 7)   ok
```

- **甲**：加，重建时 ≤ 4 KiB 的对象才读得到值，但那个值不被任何 AAD 绑，性质上就是乙′ 的提示；不加，≤ 4 KiB 的对象按丙′ 的规则重建。两种都得在候选文本里写明。加的代价：64 字节对象每容器 294 → 274 个（少 6.8%），512 字节 58 → 57，1 KiB 与 4 KiB 不变。
- **乙′**：可加可不加。加了只在「记录与 key 都丢」时给 ≤ 4 KiB 对象多保住局部性；不加，这一档按丙′ 的规则走，正确性一样。
- **丙′**：不加。
- ⚠️ 没算的一格：D27（小数据打包容器） 的档表按槽宽等比切（`27-小数据打包容器.md:28`：`W_min` = 64、公比 1.125、36 档），而每槽的自描述算不算在槽宽 `W` 里，我读到的 D27 那几行没写；算在里面的话，+8 会把挨着档界的对象推进下一档，这笔没量。

### 分批重新聚簇下甲每批要重写多少单元

一批 = 一次发布。甲每搬一条 extent key 就重写一个单元，一个单元一个事务（`16-发布语义.md:185`）：一批 b 条 key = b 个单元、b 个事务、b × 32 KiB 用户数据重写、b 个新 nonce、b 条新映射条目，旧的 b 个单元进释放路径；被快照或克隆共享的还要拆开。乙′ / 丙′ 一批只改 b 条 key 与一条记录。

一次发布装得下多大的对象的重编（推算），`output.txt` 整行抄：

```
U3 extent_leaf_records=144 per_extent_metadata_bytes=227.6 per_unit_jia_bytes=33106.3 extents_per_publish_bing=1179648 units_per_publish_jia=8108
assert U3 extent 叶每叶条数                                                    got=144      want=144      ok
assert U3 一次发布里乙′ / 丙′ 能重编的 extent 数（推算）                                  got=1179648  want=1179648  ok
assert U3 一次发布里甲能重编的单元数（推算）                                               got=8108     want=8108     ok
```

⇒ 默认有效 `T_dirty` 256 MiB（`16-发布语义.md:181`）下，甲一次发布最多重编约 8108 个单元（约 253 MiB 的对象），乙′ / 丙′ 约 1179648 条 extent（约 36 GiB 的对象），差约 145 倍。超过的对象必须跨发布，落进第三节「共同前提上的洞」第 1 条；甲在 253 MiB 以上就落进去，乙′ / 丙′ 在 36 GiB 以上。口径：重编一条 key 的元数据按「删一处、插一处各摊一片 16 KiB extent 叶，每叶 144 条」估，甲再加映射条目的两份摊销（映射叶扇出 296）；journal 记录、deadlist、分配记录的写没算；是推算不是实测。

**U3 触发观测**：「写出一条臂在某个操作上要改写已发布单元」——甲在「重编 key」下触发（手造 E，条件同第一轮：那个动作要取重编 key），在「只搬物理位置」下不触发；乙′ / 丙′ 两种定义下都不触发。「代价随盘容量涨」：三条臂都不随盘容量涨，甲的重编代价随被重编的数据量涨。

## 六、U4 检查：I-9.9 在各臂下判得了什么

1. **合法镜像上判红（三条臂共同）**：第三节「共同前提上的洞」第 1 条。I-9.9（`invariants.md:265`）要能表达分批重编的中间态（例如按记录的双值判，或给迁移中的对象另立一条），三条臂是同一件事。
2. **「记录丢了、extent 树没丢」这一格，I-9.9 的两个条件同时成立。** 这时丙′、乙′、甲的最强读法都从幸存 key 编出记录，两侧同源。I-9.9 整行里「extent 树未经重建时输出红 / 绿」按字面该出红 / 绿（而且必然是绿），同时「超级块 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」」也成立；字面没说哪个优先，取前者就是 I-9.9 自己禁止的「通过」。三条臂一起要改成按侧判：一侧是从另一侧编出来的，就输出不可判定。
3. **判别力自证**：往 extent key 里注入一个写路径 bug（key 编到 2，记录与单元头都说 1），`output.txt` 整行抄：

```
U4 plain bing_s1     before_normal_missing=1 before_i99_red=1 rebuilt_record=(2,) after_missing=0 after_mac_fail=0 record_from_keys=True header_vs_key_mismatch=n/a
U4 plain yi_s3       before_normal_missing=1 before_i99_red=1 rebuilt_record=(2,) after_missing=0 after_mac_fail=0 record_from_keys=True header_vs_key_mismatch=1
U4 jia   jia_anc_der before_normal_missing=1 before_i99_red=1 rebuilt_record=(2,) after_missing=0 after_mac_fail=1 record_from_keys=True header_vs_key_mismatch=1
assert U4 丙′：(注入后常规读找不到, I-9.9 字面红, 重建后找不到, AAD 失配, 记录取自 key, 见证)         got=(1, 1, 0, 0, True, 'n/a') want=(1, 1, 0, 0, True, 'n/a') ok
assert U4 乙′：同上（见证 = 单元头 vs key）                                          got=(1, 1, 0, 0, True, 1) want=(1, 1, 0, 0, True, 1) ok
assert U4 甲：同上（重建后读路径 AAD 失配，见证 = 单元头 vs key）                             got=(1, 1, 0, 1, True, 1) want=(1, 1, 0, 1, True, 1) ok
```

- 注入之后三条臂的常规读都找不到、I-9.9 字面都判红：未经重建时三条臂都写得出自证（干净时绿，第三节各世界 `atomic` / `physical` 的 `i99_legal_red=0`；注入后红）。
- 记录丢了、按幸存 key 重建之后：丙′ 与乙′ 的读者都找得到了，bug 被静默抹掉，I-9.9 只能不可判定；甲的读路径 AAD 失配 1，bug 仍看得见。
- 盘上多出来的那一格见证（单元头 vs key 首段）：甲 1、乙′ 1、丙′ 没有这一格。乙′ 那一格在合法重编之后会误报（第一轮 H′：合法镜像上判红），只能当提示报；甲那一格在「重编都重写」的纪律下是精确的，而且它就是盯住那条纪律（第三节「甲在不加密的卷上」）的检查。
- ⇒ 与第一轮同向：甲在 U4 上比丙′ 多一格判别力，乙′ 多半格（只在没被重编过的对象上可信）。按 U4 触发观测「写不出判别力自证的臂记一次」：丙′ 在这一格写不出，记一次；那一格是 I-9.9 自己写明的「不可判定」，算不算输交主 agent。

## 七、各臂要改哪几句 kb（kb 文件自己的行号）

| 臂 | 要改的句子 | 改成什么 |
|---|---|---|
| 甲 | `09-加密.md:524`（AAD 的定义） | 六元组，规范编码 41 字节；同一节那张「为什么必须在里面 / 谁提供期望值」的表补 locality 一行 |
| 甲 | `09-加密.md:553`（「一定不能进 AAD」表里 `locality_id` 那一行） | 删行并写撤销依据；前置是 `08-核心索引结构.md:198` 那个动作取「只搬物理位置」，或写明取「重编 key」时甲重写全部被重编单元（第五节的代价） |
| 甲 | `invariants.md:28`（I-1.8（归并后版本全序）） | 码 1 的版本分组不看 locality（S6） |
| 甲 | 重建规则：D18（块里携带什么信息） 已定项 5，或 `08-核心索引结构.md:422` 那段「重建」 | S6 + S7：只按锚点择新；重建出的记录取重建出的首段集合，读者逐个试 |
| 甲 | `08-核心索引结构.md:198` 那个动作若取重编 key、协议取先抄后切 | 迁移期间的写只写主段，另一段作废重抄（S8） |
| 甲 | `27-小数据打包容器.md:281`（槽自带五元组） | 写明槽带不带 locality；带的话写明它不被 AAD 绑（`27-小数据打包容器.md:379`），是提示 |
| 甲 / 乙′ | `18-块里携带什么信息.md:394`、`:396`、`:596`（五元组那一组、33、类身份段表五元组那一行） | 甲：六个字段、41；乙′：五元组照旧，类身份段另加一个 8 字节字段。两条都让类身份段 105 → 113（`:592` 的 format-const 跟着改） |
| 甲 / 乙′ | `18-块里携带什么信息.md:398`–`:404`（那条 ⚠️ 提议） | 改成定案 |
| 甲 / 乙′ | `layout/01-first-txn.md:172`–`:176` 及其后各行 | 五元组之后的偏移整体后移 8 |
| 乙′ | `18-块里携带什么信息.md:394` | 头里的字段从此不等于 AAD 那一组 |
| 乙′ | `18-块里携带什么信息.md:602`–`:604` | 把 locality 列进加密开启时就地变密文的那一串 |
| 乙′ | `invariants.md:28` | 注明提示字段不进「同 key」 |
| 丙′ | `18-块里携带什么信息.md:398`–`:404` | 提议不收；`:400`「放不回去」改成「放得回去，首段按规则重编，丢的是局部性」 |
| 丙′ | `08-核心索引结构.md:206` | 「扫描重建时也没法重新编 key」改成「重编之后的值要有地方存」 |
| 丙′ | D18（块里携带什么信息） 已定项 5，或 `08-核心索引结构.md:422`–`:424` | 重建规则：对象按 (树 ID, inode) 认、只看本头的 extent 树；这个对象全部 extent key（幸存的与找回的）编同一个首段（S9）；取值次序二选一（S1 幸存 key 优先 / 记录优先），写明只是局部性偏好 |
| 三臂共同 | `08-核心索引结构.md:198` | 定义「后台重新聚簇」是只搬物理位置还是重编 key；重编 key 时写明分批协议（装得进一次发布的对象一次做完，装不下的走 P 或 C；P 要记录的双值字段） |
| 三臂共同 | `invariants.md:265`（I-9.9） | ① 分批重编的中间态怎么判；② 「extent 树未经重建」与「map_provenance 标 rebuilt」同时成立、记录取自 key 时输出不可判定（S10） |

## 八、我提的收严（都只在本模型上量过，被攻过零轮）

| # | 收严 | 修哪一格 | 本模型上的读数（`output.txt` 的 `row` 行） |
|---|---|---|---|
| S6 | 甲的码 1 版本分组只看 (对象, 锚点偏移)，不看头里的 locality | 打中 A | `atomic` `jia` 家族：`jia_key_der` 700 → `jia_anc_der` 0 |
| S7 | 甲重建出的记录取重建出的首段集合（视图里的多值，不落盘），读者逐个试 | 打中 B、打中 C（手造 C / E′） | `atomic`：`jia_anc_rec` 640 → `jia_anc_der` 0；`perunit` 1456 → 0；`clone` 51752 → 0 |
| S8 | 甲在先抄后切期间的用户写只写主段，另一段那条 key 作废重抄 | 打中 B′ | `copy` 的 `jia_anc_der` 288 → `copy_recopy` 0 |
| S9 | 丙′ 把这个对象的全部 extent key（含幸存的）重编到同一个值——候选原文本来就这么写，第一轮模型没这么建 | 让 S1 从必要条件变成局部性偏好 | `atomic`：`bing_r1lit` 448 → `bing_rec` 0 |
| S10 | I-9.9 按侧判：一侧是从另一侧编出来的就输出不可判定 | 第六节第 2 条 | 手造 U4 |

- 按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算「没被攻过」」：S6–S10 交用户时写明被攻过零轮；S6 / S7 / S8 各自要一条会红的检查（把手造 A / C / B′ 做成模型对拍里的反例），检查另记账。
- 第一轮的 S1–S5 这一轮又被攻了一轮：S1 与 S3 没被打中（`bing_s1`、`yi_s3` 在跑了它们的世界里 0），S1 的角色改判成局部性偏好（S9）；S2 这一轮没另建「按 inode 号认对象」的读法（归属神谕本身按 (树 ID, inode)），不算又攻了一轮；S4（乙′ 进密文）没有模型读数，第三节「加密开启时的重建路径」与第四节核了条款；S5 被第六节第 2 条补了另一半（S10）。

## 九、什么现象会推翻每条结论

| 结论 | 推翻它的观测 |
|---|---|
| 丙′（全部 key 编一个值）在 U1 上没被正确性后果打中 | 一段合法历史让 `bing_s1` 或 `bing_rec` 找不到、读到旧版或出现两个首段。模型外最可能的来源：同一个锚点下写序先后与现行关系相反的两个版本（被抛弃时间线的孤儿，第一轮 G），或删除 / 截断之后留在盘上的旧版（这一轮没建墓碑） |
| 乙′（S3）在 U1 上没被打中 | 同上；另加一段历史让头提示排到记录或幸存 key 前面 |
| S1 不是正确性要求 | 一段历史让 `bing_rec` 失败而 `bing_s1` 不失败（九个世界里没有） |
| 甲的字面分组会放回旧版（打中 A） | I-1.8 或 D18（块里携带什么信息） 写明码 1 的「同 key」不含 locality（那就是采纳了 S6），或一条条款写明被重写掉的旧单元在扫描路径上不可读 |
| 甲记录优先在记录回退时找不到（打中 C） | 一条条款允许重建时把幸存 key 编到另一个首段而不核 AAD（重建路径不核 AAD），或规定重建出的记录不取幸存记录（那就是 S7） |
| 甲最强读法只在「一次写两份」的先抄后切里被打中（打中 B′） | `copy_recopy` 世界出现 `jia_anc_der` 失败，或一个不写两份的协议下也出现（`atomic` / `perunit` / `perunit2` / `pack` / `clone` 今天都是 0） |
| 甲在不加密的卷上只靠写路径纪律 | 一条条款让不加密的卷在读路径上也核头里的 locality 与查找路径的首段 |
| 甲进 AAD 罩不到槽里的 locality | `27-小数据打包容器.md:379` 那一行被推翻，改成每槽各自的 AAD |
| 重建路径上甲与「乙′ + 进密文」拿到同一样东西 | 重建路径拿到一个独立于单元头的 AAD 期望值来源（`18-块里携带什么信息.md:428` 那一行不再成立） |
| 分批重编让 I-9.9 在合法镜像上判红（三臂共同） | 一个分批协议让每一次中间发布都满足 I-9.9 字面（记录单值、全部 key 同一首段），且不把大对象的重编变成一次无界的发布 |
| 「只搬物理位置」下甲零重写 | 一条条款要求整理搬迁时改写单元头（`26-后台整理与放置回收.md:214` 写的是搬迁走映射、不改引用者） |
| 甲一次发布装得下的对象比乙′ / 丙′ 小约 145 倍 | 实测一次发布里重编一条 key 的元数据代价远大于 228 字节（journal、分配记录、deadlist 的写把它抬到 KiB 级），比值随之缩小；这是推算，没实测 |
| 槽 +8 在 64 字节对象上少装 6.8% | E114（小数据打包容器的总账） 按 55 字节自描述重跑出不同的数；或自描述不在槽宽里、按档补齐之后 +8 不改变装载数 |
| U4：甲比丙′ 多一格判别力 | 丙′ 下也有一个独立见证，能在「记录丢了、extent 没丢」时判出 key 首段写错（例如别处存了 locality 的第三份） |

## 十、复跑与指纹

```
python3 -B research/prompts/d18-locality-r2-opus-model/model.py > /tmp/claude-1000/d18-locality-r2-opus.out
diff /tmp/claude-1000/d18-locality-r2-opus.out research/prompts/d18-locality-r2-opus-model/output.txt
```

- 默认深度，本机实测 3 分 57 秒（`time`）；`model.py N` 把全部世界改成深度 N，`model.py 0 世界名,世界名` 只跑这几个世界、不跑手造历史。只用 std 的 Python，没有 Rust、没有 target 目录。
- 确定性：无随机源、无 I/O。第二次跑（写在 scratchpad）与留存的 `output.txt` `diff` 逐字节相同，只说明没有隐藏状态，不是统计意义上稳定。输出里 69 条 `assert … ok`、0 条 FAIL（`grep -c`）。
- sha256：
  - `907c9d4eb9e3f584c0218491617dced0f81e43580650a37819a4930f843f3ca6  research/prompts/d18-locality-r2-opus-model/model.py`
  - `70a81045045e4f574121fe43bf0cf3580f78db7a6977e9b67bb006c04b17709e  research/prompts/d18-locality-r2-opus-model/output.txt`
- 射程与没建的：单元归属当神谕（C113 与克隆祖先表视为已解）；删除 / 墓碑、reflink、被抛弃时间线的孤儿（第一轮 G）、同一次发布里几个事务之间的中间态（D23（journal 的角色与格式） 前缀判定第六条之后不会成为恢复结果）没建；每容器 2 条记录、每对象 2 个 offset、locality 两个值是小取样点，不是真值；`perunit2` / `clone` / `v1` 只到深度 7。
- 过程记录：
  - 模型源码 1 次新建、5 次追加、二十多次定点编辑。**两次追加超了每次 150 行的上限**：第二段 166 行、第五段 151 行。两次写入都完整落盘（文件能跑、69 条断言全过、两次跑逐字节相同），之后没有拆成更小的段重写。照实记，交主 agent 判。
  - 报告分 7 次写入（第一段用 noclobber 排他新建），每次都在 150 行以内；第三节一句措辞事后定点改过（「十个世界」改成「跑了它们的每个世界」，因为 `naive` 只跑 `bing_s1`）。
  - 留存的 `output.txt` 之前改过两处模型缺陷：① 第一版重建对 extent 一条没丢的对象也往回补扫描到的版本，改成只对丢了条目的对象补，按单元头编 key 的臂只补丢了的那几条原 key；② 协议 C「只写主段」那一版在删旧段阶段的写带走最后一条旧 key 时没结束迁移，删旧段那一步找不到 key 而崩溃，补了一句。另：起初 `clone` 取深度 8、估计要跑几小时，按 `ps` 列出的写死的 pid 停掉两个在跑的进程，改成 7。
  - 这一轮别的腿的产出一份都没打开；第一轮的模型目录只读、没改、没拷。

## 十一、判决摘要（给主 agent）

- **丙′**：这一轮换的五个面——分批重新聚簇的中间态、D27（小数据打包容器） 的槽、加密开启时的重建路径、「后台重新聚簇」按重编 key 与只搬物理位置各一遍、克隆之后重编——都没打出正确性后果：`bing_s1`、`bing_rec` 在跑了它们的每个世界 0 失败，七个世界在操作上限内穷举完。第一轮的 S1 不是正确性要求：丙′ 原文本来就把这个对象的全部 key 编到同一个值（S9），这时记录优先与幸存 key 优先都是 0，差的只是局部性；S1 与 `18-块里携带什么信息.md:521` 那条纪律有张力，记录优先没有。
- **乙′（S3 + 进密文）**：同样没打中，比丙′ 多保住一部分局部性；U4 多半格（头 vs key 的见证在重编过的对象上误报）；槽 +8 可选，64 字节对象每容器少装 6.8%。
- **甲**：四种读法各被一段分辨臂的历史打中——A：字面分组把重编前的旧版当另一条 key 放回（读到旧版、两个首段）；C / E′：记录优先在记录回退时找不到（第一轮打中 1 落在甲身上，甲不能像丙′ 那样把幸存 key 编回去）；B：先抄后切抄了一份就找不到；B′：最强读法在「一次写两份」的先抄后切里找不到或两个首段。要站住得加 S6 + S7 + S8（零字节、被攻过零轮）并改 I-1.8；它的 AAD 罩不到槽里的值，重建路径上 AAD 不替它作证，不加密的卷上只靠写路径纪律。「只搬物理位置」下甲零重写、全 0 失败；「重编 key」下每搬一条 key 重写一个单元，一次发布装得下的对象比乙′ / 丙′ 小约 145 倍（推算）。
- **三臂共同，不拿来判臂**：「重编 key」下分批的中间态让 I-9.9 在合法镜像上判红，记录不带旧值的协议常规读就坏；I-9.9 在记录取自 key 时两个条件打架（S10）。「后台重新聚簇」取哪种定义仍是全仓空白，它决定甲在 U2 / U3 的出局前件。
- **按跑前条款**：丙′、乙′ 在 U1 都没被正确性后果打中；甲在 U1 被正确性后果打中四次，而反向接受条款没有「甲在 U1 被打中」这一格，照实交主 agent；甲在 U2 的前件仍取决于 `08-核心索引结构.md:198` 的定义，归辩方腿。攻方这条腿没把丙′ 打穿。S6–S10 都只在本模型上量过，被攻过零轮。
