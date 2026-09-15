# D18（块里携带什么信息） 已定项 3 那条提议（自描述头加 `locality_id`） 第一轮：攻方腿（Opus）报告

立场：反推 / 攻。假设主 agent 倾向的丙（不加）是错的，也假设甲、乙各有死角，去造重建历史把它们打穿。
模型：`research/prompts/d18-locality-r1-opus-model/model.py`（只用 std 的 Python），原始输出同目录 `output.txt`，复跑命令与 sha256 在第十节，判决摘要在第十一节。
引 kb 一律写 kb 文件自己的行号，行号在 `.claude/kb/` 下逐行现查，不从背景材料里数。

## 一、失败条款先核：前提表一至十对 kb 原文

十条逐条对过，八条对得上。两处要报：都不改这一轮要用的命题，但按失败条款的字面交主 agent 判。

**① 前提六的主语换了。** 前提表写「加密开启时它会新增第一处「用户对象语义类别」的泄漏」，读下来主语是 `locality_id` 这个字段。原文 `.claude/kb/decisions/14-双轨大小文件-持久临时.md:291` 整行：

> ② **加密开启时它新增第一处「用户对象语义类别」的泄漏**——D18（块里携带什么信息） 已定项 14 那一轮列的

这一行的「它」是同文件 :280–281 那条被判掉的分配器提示候选（「判据取 D8（核心索引结构） 已定项 3 的 `locality_id`」），泄漏的载体是按 locality 聚簇的物理排布。同文件 :295–297 整三行：

> ⚠️ 而 `locality_id` 至今**没有**盘上的明文表达（D18（块里携带什么信息） 已定项 3 的自描述头五元组不含它，
> 加它的提议 2026-09-03 起待用户定案且逐字「动它是一次永久契约变更」）
> ⇒ 按 locality 聚簇等于不加那个字段就把同一个信号搬进物理排布。

⇒ 前提六把讲「按 locality 聚簇」的一句挪到了「字段本身」上。这一轮要用的命题（乙把 locality 明文写进每个数据单元，就把同一个信号直接写上盘）从 :295–297 直接推得出来，比聚簇更直接。我判它不让前提作废；按字面是一处转述对不上，交主 agent 定。

**② 前提一只带了已定项 3 的「明文」，没带已定项 13。** `.claude/kb/decisions/18-块里携带什么信息.md:602–604` 整三行：

> ⚠️ **加密开启时这张类身份段表里只有 fsid 与载荷 CRC 留明文**（已定项 13，2026-09-07 用户定案取臂乙′）：
> 五元组 / 诞生代号 / 写序就地变密文，位置与宽度一个都没动，明文头总宽仍 105。
> 加密关闭时整张表都是明文，第一个事务写出的字节逐字节不变。

这不算转述错（附录里的已定项 3 自己也点了已定项 13），但它是这一轮的枢纽条款，没进前提表：候选乙那句「加密开启时它是明文」正撞在它上面（第四节），而加密开启时重建本来就要密钥、要从单元明文头拿完整 nonce（同文件 :148，已定项 14，长行只指路），这决定了「乙 + 进密文」那条出路要付什么（第五节）。

**前提表之外、这一轮用到而材料里没有的条款**（按 `.claude/rules/three-way-inference.md`「正文没点名的枢纽条款」那条列出来，交主 agent 核）：

| 条款 | 行 | 为什么承重 |
|---|---|---|
| D8（核心索引结构） 已定项 3「退化由 D3 修复」与「批量改名」第 3 条 | `08-核心索引结构.md:137`、`:198` | 全仓唯一一个会改已有对象 `locality_id` 的动作；甲的 U2 / U3 全压在它身上 |
| D26（后台整理与放置回收） 的搬迁「走映射不改引用者」 | `26-后台整理与放置回收.md:218`（长行，只指路） | 物理搬迁不碰 extent key ⇒ 修得了物理落点，修不了 key 序局部性 |
| D27（小数据打包容器） 槽自带五元组 | `27-小数据打包容器.md:281–282` | ≤ 4 KiB 对象进容器之后，甲 / 乙加在码 1 头上的 8 字节罩不到它们 |
| D19（块指针的结构与宽度预算） 已定项 6 的收严 ⑩㈠ | `19-块指针的结构与宽度预算.md:173` | 槽里 key 各段逐字节取打包前那一版单元头 ⇒ 甲 / 乙要罩小对象，槽也得跟着 +8 |
| D1（数据可移动性 / 反向索引） 已定项 2 | `01-数据可移动性-反向索引.md:19` | 「反向映射的主体就在单元里」⇒ D9 那条「三份口径」里的反向索引就是单元头的身份 |
| D8（核心索引结构） 已定项 6 克隆水位 | `08-核心索引结构.md:386` | 「两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)」⇒ 丙那句「这个对象」按 inode 号读会中 |
| D8（核心索引结构） 已定项 6 重建射程 | `08-核心索引结构.md:424` | 多头镜像的 inode 树重建今天没有输入；模型因此把单元归属当神谕，只量 locality 这一维 |
| D18（块里携带什么信息） 已定项 11「表不可读时」 | `18-块里携带什么信息.md:879`（长行，只指路） | 实例表不可读时全部旧实例的单元判已发布 ⇒ 第三节 G 的孤儿世界 |
| E98（inode 记录与 inode 树的几何） | `experiments/98-inode记录与inode树的几何.md:65–66` | 记录丢了算不回来的十一个字段里 `locality_id` 只是其中一个 |
| D3（空间分配） 第 4 条 | `03-空间分配.md:17–18` | 加密开启时无密钥侧只读只检测 ⇒ 重建与重新聚簇都发生在有密钥的一侧 |

## 二、模型怎么建的（口径）

- **对象** = (树 ID, inode)。inode 记录住容器（每容器 2 条；真值 233，取 2 让「丢一个容器连带丢邻居」在短历史里出现），每次改记录都 COW 容器，旧版留在盘上，可以被「回到上一版」那种损坏选中。
- **数据单元**写出后不可变，头里带 (出生树, 对象 ID, 对象出生代, 锚点偏移) 与写那一刻记录里的 locality（甲 / 乙的第六个字段；丙的世界里这一格照样生成，但重建不读它）。
- **操作**：建对象（父目录给 locality 1 或 2；第一版世界恒 0）、写一个 offset、克隆（记录容器与 extent 条目按物理 id 共享，水位取 origin 那一刻）、重新聚簇（D8 已定项 3 第 3 条设想的动作：记录与这个对象全部 extent key 在同一事务里换首段，单元不动；甲的世界里 AAD 逼着把这些单元重写）。改名按 `08-核心索引结构.md:196` 不碰 locality，模型里是空操作，不列。删除与墓碑只按 (对象 ID + 出生代 + 区间) 指认死者，与 locality 无关，不建。
- **损坏**：目标头的每个记录容器 在 / 丢 / 回到上一版；每个对象的 extent 条目 全在 / 全丢 / 只丢最低 offset 那条。丢了条目的单元由扫描按单元头找回，**单元归属与现行版本判定当神谕**（C113 与克隆祖先表视为已解）——量到的差别只来自 locality 这一维。
- **重建规则**：甲与乙的逐单元读法把找回的单元编在它自己头里的 locality 下（甲是 AAD 逼的，乙是「从单元头读 key 首段」的最弱读法）；乙的按对象读法与丙给每个对象选一个值，全部 extent 编同一个首段。记录的 locality 按两种次序各跑一遍：record_first（各臂字面：记录在就信记录）与 keys_first（收严：幸存的 extent key 优先）。丙另跑一条最弱读法 bing_inode_only：「这个对象」按 inode 号认，不按 (树 ID, inode)。
- **检查**：读者按重建后记录里的 locality 去查每个本该有数据的 offset。找不到、找错、甲的 MAC 失配、读到损坏前没有的数据、一个对象两个首段，任一非 0 记一次失败；另数「重建后 locality 与原值不同」（只丢局部性，不记失败）。
- **穷举**：按深度宽搜全部历史，每条历史 × 每个头 × 全部损坏形态 × 每条臂两种次序。这是确定性模型，跑 N 遍与跑 1 遍信息量相同；证据强度来自手造历史里钉死的绝对值（第三节 A–H 全部 `assert … ok`）与两个阳性对照（B：甲不重写就重新聚簇，无损坏时 MAC 败 2；H：往记录里注入一个错的 locality，未经重建的 I-9.9 由绿转红）。

## 三、U1 重建：各臂最短的反例

三个世界：`rekey_on`（有 D8（核心索引结构） 已定项 3 第 3 条那个重新聚簇动作）、`rekey_off`（没有）、`v1_all_zero`（第一版，locality 恒 0）。`output.txt` 里穷举那一段的全部 row 行，整行抄（列：world arm precedence max_depth histories patterns objects failures ambiguous locality_changed forced_rewrites unshared stale_hints）：

```
row rekey_on jia record_first 6 38593 866498 1212564 55620 0 55620 16132 3272 0
row rekey_on jia keys_first 6 38593 866498 1212564 0 0 0 16132 3272 0
row rekey_on yi_per_unit record_first 6 38593 866498 1212564 91142 16560 108552 0 0 14456
row rekey_on yi_per_unit keys_first 6 38593 866498 1212564 13440 16560 100404 0 0 14456
row rekey_on yi_per_object record_first 6 38593 866498 1212564 21784 16560 89088 0 0 14456
row rekey_on yi_per_object keys_first 6 38593 866498 1212564 0 16560 67304 0 0 14456
row rekey_on bing record_first 6 38593 866498 1212564 21784 0 396724 0 0 14456
row rekey_on bing keys_first 6 38593 866498 1212564 0 0 374940 0 0 14456
row rekey_on bing_inode_only record_first 6 38593 866498 1212564 29400 39232 349876 0 0 14456
row rekey_on bing_inode_only keys_first 6 38593 866498 1212564 19616 39232 368748 0 0 14456
row rekey_off jia record_first 6 26109 617750 872232 0 0 0 0 0 0
row rekey_off jia keys_first 6 26109 617750 872232 0 0 0 0 0 0
row rekey_off yi_per_unit record_first 6 26109 617750 872232 0 0 0 0 0 0
row rekey_off yi_per_unit keys_first 6 26109 617750 872232 0 0 0 0 0 0
row rekey_off yi_per_object record_first 6 26109 617750 872232 0 0 0 0 0 0
row rekey_off yi_per_object keys_first 6 26109 617750 872232 0 0 0 0 0 0
row rekey_off bing record_first 6 26109 617750 872232 0 0 263476 0 0 0
row rekey_off bing keys_first 6 26109 617750 872232 0 0 263476 0 0 0
row rekey_off bing_inode_only record_first 6 26109 617750 872232 1120 4928 213716 0 0 0
row rekey_off bing_inode_only keys_first 6 26109 617750 872232 2464 4928 219956 0 0 0
row v1_all_zero jia record_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero jia keys_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero yi_per_unit record_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero yi_per_unit keys_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero yi_per_object record_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero yi_per_object keys_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero bing record_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero bing keys_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero bing_inode_only record_first 7 24501 900259 1576098 0 0 0 0 0 0
row v1_all_zero bing_inode_only keys_first 7 24501 900259 1576098 0 0 0 0 0 0
```

每一行被检查的对象次数都在 87 万以上，不是空扫。有失败的那几格，最短历史整行抄（其余格都是 `none_through_depth=6` 或 `=7`）：

```
shortest rekey_on jia record_first depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'rolled_back')] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=0
shortest rekey_on yi_per_unit record_first depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'all_lost')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=0
shortest rekey_on yi_per_unit keys_first depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; write(11,1,1) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'first_lost')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=1
shortest rekey_on yi_per_object record_first depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'rolled_back')] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=0
shortest rekey_on bing record_first depth=3 head=11 history=[create(11,1) ; write(11,1,0) ; rekey(11,1,2)] containers=[(0, 'rolled_back')] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=0
shortest rekey_on bing_inode_only keys_first depth=4 head=11 history=[create(11,1) ; write(11,1,0) ; clone(11,21) ; rekey(11,1,2)] containers=[(0, 'intact')] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=0
shortest rekey_off bing_inode_only record_first depth=5 head=21 history=[clone(11,21) ; create(11,1) ; write(11,1,0) ; create(21,2) ; write(21,1,0)] containers=[(0, 'lost')] extents=[(1, 'intact')] tally=missing=1,misread=0,mac_fail=0,resurrected=0,split_objects=0
```

### 打中 1：「记录在就信记录」+ 记录容器回到上一版 + 重新聚簇（3 步，四条读法一起中）

历史：建对象（目录 1）→ 写 offset 0 → 重新聚簇到目录 2；损坏：记录容器最新版读不出、上一版可读，extent 树完好。重建选中旧版记录（locality 1），而 extent key 在 2 ⇒ 读者按 1 找不到。手造 D 把五条臂各钉一遍：record_first 都是 1，keys_first 都是 0。
- **分不分辨臂**：不分辨。丙与乙按对象读法失败数都是 21784，甲 55620（甲的找回单元编在头里的 2 下，旧记录 1 一个都找不到，所以更多）。病根在共同前提：各臂字面都写「记录在就信记录」，而重建能选中一个旧版记录。
- **系统看不看得到判别子**：看得到。同一对象幸存的 extent key 首段与选中的记录不等，重建那一刻两样都在手里。
- **满足哪条判据的哪个分句**：U1 触发观测的第一个分句「读者按记录里的 `locality_id` 找不到某段数据」。
- **收严（线索，只在本模型上量过，被攻过零轮）**：重建时幸存的 extent key 优先于记录（`keys_first`）。三条臂深度 6 全部 0 失败。它要写进重建规则里，与臂无关。
- **前置**：要有一个会改已有对象 locality 的动作。`rekey_off` 世界里四条读法 record_first 全是 0。

### 打中 2：丙的最弱读法——「这个对象」按 inode 号认（5 步，丙独有）

历史：先克隆，再在两个头里各建一个对象（目录 1、目录 2），两个头发出同一个 inode 号 1（`08-核心索引结构.md:386` 逐字「两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)」）；损坏：克隆头的记录容器丢了。按 inode 号去所有幸存 extent 条目里找首段，拿到 {1, 2} 两个值，取错一个 ⇒ 读者找不到。有重新聚簇时连损坏都不需要（4 步：origin 重新聚簇之后，克隆头那份旧 key 仍在 1）。
- **分不分辨臂**：分辨。甲、乙的单元头带出生树，按 (出生树, 对象 ID) 认领不撞；`rekey_off` 里甲、乙四格全 0，只有 `bing_inode_only` 非 0。
- **系统看不看得到判别子**：看得到。条目住在哪棵树里，重建那一刻就知道。
- **满足哪个分句**：U1 第一个分句（找不到）。
- **收严（线索，零字节，被攻过零轮）**：丙那句「这个对象还有幸存的 extent key 就取它们的首段」写成「(树 ID, inode) 在本头 extent 树里幸存的 key」。按树认的 `bing` 在 `rekey_off` 两格都是 0，`rekey_on` 的 keys_first 也是 0。
- ⚠️ 射程：多头镜像的重建今天本来就没有输入（`08-核心索引结构.md:424`），这个缺口要等克隆祖先表落地才够得着；但丙的条款文本得在那之前写对。

### 打中 3：乙的逐单元读法——头里的提示过期（3 步，记录还在也中）

历史：建对象（目录 1）→ 写 offset 0 → 重新聚簇到目录 2（单元不动，头里还写着 1）；损坏：extent 树丢了、记录完好。按单元头编首段 ⇒ 单元落在 (1, 1, 0)，记录说 2 ⇒ 找不到。加上 keys_first 收严照样中（4 步：只丢 offset 0 那条，幸存的 offset 1 在 2，找回的 offset 0 按头编到 1 ⇒ 一个对象两个首段）。手造 C / H′ 钉了绝对值：记录与 extent 全丢时找不到 1、两个首段 1；记录在时找不到 1。
- **分不分辨臂**：分辨。同一历史甲 0（重新聚簇时 AAD 逼着重写了单元），丙 0。
- **系统看不看得到判别子**：看得到不一致，分不出原因。「头提示 ≠ 记录」在乙的定义下可以是合法过期，也可以是坏了；H′ 在一个完全合法的镜像上把 I-9.9 判红，是误报。所以重建只能不信它。
- **满足哪个分句**：U1 两个分句都满足（找不到；一个对象两个首段）。
- **收严（线索，被攻过零轮）**：按对象只取一个值，幸存 key 与记录都优先于头提示（`yi_per_object keys_first` 深度 6 为 0）。收严之后乙就是「丙 + 记录与 key 都丢时用头提示代替 0」。
- **前置**：同打中 1，`rekey_off` 里乙两种读法全 0。

### 丙的收严读法：没打中

- `bing keys_first`（对象按 (树 ID, inode) 认、重建时幸存 key 优先于记录、两样都没有取 0）三个世界 0 失败：rekey_on 深度 6（38593 条历史、866498 个损坏形态、1212564 次对象检查）、rekey_off 深度 6、v1_all_zero 深度 7；ambiguous 恒 0——一个头里同一对象的幸存 key 从来没有两个首段（模型里重新聚簇按对象在一个事务里做完，合法镜像上 I-9.9 成立）。
- 丙付的是局部性：rekey_off 里 872232 次对象检查有 263476 次重建后 locality 与原值不同（记录与 key 都丢时取 0），甲 / 乙 0；rekey_on 里丙 374940、乙按对象 67304、甲 0。这是甲 / 乙在 U1 上唯一买到的东西，而且只是局部性：重建产物落在 D18 已定项 5 的级 1，`18-块里携带什么信息.md:509` 整行「| 1 | 只有校验和 / MAC 自洽，没有分配记录 | **只读挂载** |」。
- 第一版：v1_all_zero 十格全 0 失败、全 0 局部性变化（深度 7，24501 条历史）。丙的「没有就取 0」在第一版恢复的就是原值；甲 / 乙的 8 字节在第一版每个单元都写 0，携带 0 bit（手造 F 钉了）。

### 共同前提上的洞 G：实例表不可读时的孤儿（不分辨臂，不拿它判臂）

被抛弃的时间线里建过同号对象、写过 offset 0；崩溃后号与出生代都重发（D8 已定项 6：崩溃后 checkpoint 号会重发），新对象只写 offset 1。实例表不可读时孤儿单元判已发布（`18-块里携带什么信息.md:879`，长行只指路）。`output.txt` 的 G 行整行抄：

```
G abandoned=1 reissued=2 jia              resurrected=0 split=1 missing=0
G abandoned=1 reissued=2 yi_per_unit      resurrected=0 split=1 missing=0
G abandoned=1 reissued=2 yi_per_object    resurrected=1 split=0 missing=0
G abandoned=1 reissued=2 bing             resurrected=1 split=0 missing=0
G abandoned=1 reissued=2 bing_inode_only  resurrected=1 split=0 missing=0
G abandoned=1 reissued=1 jia              resurrected=1 split=0 missing=0
G abandoned=1 reissued=1 yi_per_unit      resurrected=1 split=0 missing=0
G abandoned=1 reissued=1 yi_per_object    resurrected=1 split=0 missing=0
G abandoned=1 reissued=1 bing             resurrected=1 split=0 missing=0
G abandoned=1 reissued=1 bing_inode_only  resurrected=1 split=0 missing=0
G abandoned=0 reissued=0 jia              resurrected=1 split=0 missing=0
G abandoned=0 reissued=0 yi_per_unit      resurrected=1 split=0 missing=0
G abandoned=0 reissued=0 yi_per_object    resurrected=1 split=0 missing=0
G abandoned=0 reissued=0 bing             resurrected=1 split=0 missing=0
G abandoned=0 reissued=0 bing_inode_only  resurrected=1 split=0 missing=0
```

- **分不分辨臂**：基本不分辨。第一版（0, 0）与同目录（1, 1）五条臂一起读到孤儿的数据；只有被抛弃那次创建与重发那次创建落在不同目录时，甲与乙逐单元读法碰巧把孤儿编到另一个首段、读者看不见（代价是一个对象两个首段）。
- **系统看不看得到判别子**：看不到。实例表丢了之后重建没有东西区分孤儿与现行单元，甲躲过去靠两次创建碰巧不同目录。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判别子观测不到」那一行，不拿它判臂。
- **满足哪个分句**：不是 U1 的字面（找得到、只有一个首段），是「读到损坏前没有的数据」——反向接受条款说的正确性后果，但病根在已发布谓词，不在 locality。记录还在时 I-9.10（对象出生代三处一致，`invariants.md:266`）能挡住出生代不同的孤儿；模型取的是最坏情况（出生代也相同）。

### 另一处共同前提：「增量、可中断」的重新聚簇与 I-9.9 在大对象上不能同真（推理，模型没建）

`08-核心索引结构.md:198` 整行「3. **后台重新聚簇必须是增量、可中断、可续做的**，且随时可以停掉不做。」；`invariants.md:265` 要记录的 `locality_id` == 该对象每条 extent key 的首段。一个 extent 很多的对象若分几个事务重编，中间每次发布的镜像都合法地有两个首段：I-9.9 在合法镜像上判红，丙的「取幸存 key 首段」拿到两个值，甲的读者按记录只找得到一半。三条臂一起中。出路要么是「一个对象的重编在一个事务里做完」（大对象上是无界操作），要么 I-9.9 与读者都认两个首段。模型里重新聚簇按对象原子完成，这一格没量；推翻它的观测是给出一个分批重编、每批之间都满足 I-9.9 的做法。

## 四、U2 契约

### 甲：前提三那句「取值精确且定义性」顶不顶得住前提五那条排除——两句说的不是同一对东西

- `08-核心索引结构.md:207–208` 整两行「⚠️ 「它是提示、可以是错的」准许的是**不信它的含义**，不是**不存它的位**：」「作为地址首段它的取值精确且定义性。」——精确的是**记录 ↔ extent key 首段**这一对：读者按记录里那个值去找数据，I-9.9 查的也是这一对。
- `08-核心索引结构.md:196` 整行「1. **`locality_id` 永远不许被改名更新。** 它是**提示，不是正确性的一部分**——可以是错的、可以过期。」——可以是错的，是 **locality ↔ 对象现在住的目录**这一对。
- 甲往 AAD 里放它，绑的是第三对：**单元 ↔ 它被编在哪个首段下**。`09-加密.md:586` 整行「| **链 B：逻辑身份 ↔ 指针** | **只有 AAD**。nonce 只说「这块配这个指针」，不说「这个指针该挂在哪个 key 下」 | **本工程自己的实现 bug**——btree split/merge、write buffer 的 seq 去重、快照 key 改写 |」。这一对 D8 从没说过要永远不变。
- ⇒ 剩下一个事实问题：**有没有合法操作不重写单元就改掉它的首段？** 仓里点了一个：`08-核心索引结构.md:137` 整行「- 退化由 D3（空间分配） 已承诺的常驻后台整理修复，不是新增负担」与 `:198` 的「后台重新聚簇」。有它，甲要么 MAC 失配（手造 B 阳性对照：2 次），要么把单元全重写（手造 A：2 个单元强制重写，其中 2 个本与 origin 共享、被拆开；rekey_on 深度 6 累计 16132 个强制重写、3272 个被拆开的共享）。形状与 `09-加密.md:555` 排除「读者当前所处的快照 ID」相同：绑一个在共享者之间可以不同的值。
- ⇒ **前提三与前提五不互相顶着**。D9 那条排除的机制对，措辞弱：它引的是语义上「可以是错的」，真正承重的是「可以不重写单元就改掉」。
- 甲要自洽只剩一条路：规定 locality 一经写出永不改变，连后台重新聚簇也不改。那样 `08-核心索引结构.md:137` 就落空：D26 的搬迁走映射、不改引用者（`26-后台整理与放置回收.md:218`，只指路），动不了 key；而 E9 量的收益在遍历读到的索引叶上（`08-核心索引结构.md:159` 整行「**加上 `cache_leaves = 8`**。」），key 不动，叶里装谁就不动（推理，未量）。
- 三句：分辨臂（同一历史只有甲强制重写，乙 / 丙 0）；判别子看得到（重新聚簇那一刻系统知道自己在换首段）；满足 U3 触发观测「写出一条臂在某个操作上要改写已发布单元」，也即 U2 问的「甲撞不撞前提五」。
- ⚠️ 条件式：「后台重新聚簇 = 重编 key」全仓没有定义，D26 没实现它，D8 只在第 198 行给了约束。推翻它的观测：一条已定条款写明 locality 永不改变，并给出一个不重编 key 的局部性修复机制。

### 甲还要面对的冻结分层

- 甲把 locality 放进第 1 层（`15-格式冻结政策.md:58` 整行「第 1 层：超级块 + 块头（自身地址 / generation / 树 ID / fsid / magic）」）与永久的 AAD，而它的取值规则今天没写：`08-核心索引结构.md:209` 整行「⚠️ 连带：第一版没有目录，而这个值按本项从父目录继承 ⇒ **没有父目录时取什么，也要一起定**。」；「从父目录继承」继承的是父目录的哪个值，全仓也没写（`grep -n "从父目录继承" 08-核心索引结构.md` 只命中 :132 与 :209）。今天它只在第 3 层（`15-格式冻结政策.md:62`）与 inode 记录格式那个独立冻结组件（`:78`，冻结时刻是第一个写出类型 2 容器的镜像）里。
- 有一处缓冲：AAD 的永久性从加密开机那天起算（`09-加密.md:552`，长行只指路），加密不进第一版（D9 已定项 10）⇒ 甲不必现在定，也不丢任何东西（推理）。

### 乙：原文「加密开启时它是明文」与已定项 13 逐字冲突

- `18-块里携带什么信息.md:413` 整行「⚠️ **2026-09-07 用户定案：这个泄漏不接受。** 怎么处置归 D18（块里携带什么信息） 已定项 13」，落地在 `:602–604`（第一节已整行抄）：加密开启时码 1 的明文只剩 fsid 与载荷 CRC。乙按原文是往那张白名单里加第三项，候选没写改已定项 13 的计划 ⇒ **U2 触发**。三句：分辨臂（只有乙原文中）；是静态条款冲突，不涉及运行时判别子；满足 U2 触发观测「写出一条臂与一条已定条款逐字冲突、而没有改那条条款的计划」。
- 反向接受条款里「乙 + 加密开启时这 8 字节进密文」与已定项 13 同形（位置宽度不动、内容变密文），不冲突。
- 乙还要改 `18-块里携带什么信息.md:394–395` 那句「字段取 D9（加密） 已定项 6 已定的那一组」：头里的字段从此不等于 AAD 集合。

### 丙

- 与 `18-块里携带什么信息.md:400` 整行「一旦 inode 记录丢了，扫描找到的数据单元**放不回去**——单元头给得出对象 ID 与锚点偏移，」直接冲突：丙的规则就是把它们放回去、首段重新编。那句是提议自己的论据，丙要连带改写它（事实性改写，不是契约变更）。候选写「零契约变更」对，写成「零条款改动」就不对。
- `08-核心索引结构.md:206` 整行「扫描重建时也没法重新编 key。落点是已定项 6（inode 记录的盘上字段表）。」按最弱读法（重编 key 必须知道原值）与丙冲突；按原意（重编后的值要有地方存）不冲突。丙要把这句改成后一个意思。
- `09-加密.md:659` 那条「三份口径同一份定义」：字面读法下三条臂一起不满足（extent key 是 (locality, inode, offset)，没有哪条臂让它与 AAD 逐字段相同）；按「每一份都算得出同一份身份」读，三条臂都满足（丙：key + 所在树 + 记录 + 指针算得出五元组，locality 那段不参与）。不分辨臂。D19 已定项 6 ⑨㈥（`19-块指针的结构与宽度预算.md:196`，长行只指路）已按后一种读法读过。

## 五、U3 代价

| 项 | 甲 | 乙 | 丙 |
|---|---|---|---|
| 每个码 1 单元 | 类身份段 105 → 113，含预留位的头 134 → 142，净荷 32768 − 142（`first-txn-layout.md:170–185`；`crates/singlefs-format/src/lib.rs:33` 的 `DATA_UNIT_HEADER_BYTES = 105`） | 同甲 | 0 |
| AAD 规范编码 | 33 → 41 | 不变 | 不变 |
| D27 打包容器的槽 | 槽自带五元组（`27-小数据打包容器.md:281`），槽里 key 逐字节取打包前那一版单元头（`19-块指针的结构与宽度预算.md:173`）⇒ 要罩住 ≤ 4 KiB 的对象，槽也得 +8；不加，这一档照丙的规则走 | 同甲 | 0 |
| 第一个事务 | 数据单元头 +8，五元组之后的偏移全移（`first-txn-layout.md:172–182`）；连带格式常量、27 / 39 号门禁、E142 干跑、54 号层 0 全量重跑 | 同甲 | 0 |
| 第一版这 8 字节带的信息 | 0 bit（每个单元写 0） | 0 bit | — |
| 改名 | 0 | 0 | 0 |
| 整理搬迁（D26，走映射） | 0（AAD 不含物理位置） | 0 | 0 |
| 打包进容器（D27） | 本来就要重加密（`27-小数据打包容器.md:384`），locality 不添代价 | 0 | 0 |
| 重新聚簇（`08-核心索引结构.md:198`） | **被重编对象的每个单元重写一遍**；克隆里还拆开共享（手造 A：2 个重写、2 个被拆开）。整卷重新聚簇的量随数据量涨，不随 extent 数涨（推理） | 0，头提示过期（rekey_on 累计 14456 个） | 0，只改记录与 extent key |
| 重建 | 0 | 0 | 0 |

- 甲的重写必须换新 nonce：同一个 nonce 配两份不同的 AAD 等于把 Poly1305 的一次性密钥用了两次（密码学常识，推理，未在本项目验证）⇒ 不是「补一个 MAC」，是整单元重加密、COW 成新单元。
- U3 触发观测「写出一条臂在某个操作上要改写已发布单元」：甲在重新聚簇上触发（条件同第四节：那个动作要是重编 key）。乙、丙没有触发。

### 乙的明文泄漏具体泄漏什么

- 按乙原文（加密开启时明文）：每个数据单元明文带创建时父目录给的那个值。已定项 13 把「哪些单元属于同一对象、对象多大」藏进密文之后（`18-块里携带什么信息.md:371` 那一行，长行只指路），乙把一个更粗的分组放回明文：**哪些单元出自同一个创建目录**、每个目录有多少个 32 KiB 单元（目录里的数据量，32 KiB 精度）、两次观察之间哪些目录长了新单元（活跃度）。一个目录里只有一个文件时，这就是那个文件大小的 32 KiB 精度——已定项 13 乙′ 买到的「对象大小不漏」在这种目录上退回一半。取值若是父目录的 inode 号（单调不复用，D8 已定项 6），还漏目录的相对创建次序。取值规则全仓没写（`08-核心索引结构.md:209`），这一段按「值是父目录的某个稳定标识」推。
- 第一版恒 0：零泄漏。改名之后值不变，所以它漏的是「创建时在哪」，不是「现在在哪」。

### 加密开启时乙能不能退回丙

能，两条路，都是「位所有卷都留、不挪作他用」：
1. 这 8 字节写 0，重建按丙的次序走（幸存 key > 记录 > 0）。读出 0 与「没有提示」同一个结局，模型里就是 bing 那一行。
2. 进密文：加密开启时重建本来就要密钥、要从明文头拿完整 nonce 解密身份段（`18-块里携带什么信息.md:148`，已定项 14，长行只指路），locality 跟五元组在同一段密文里，重建一步不多、提示照样可用，还顺带被 MAC 罩住（不进 AAD，只是密文的一部分）。

2 严格好于 1。结论：乙在 U3 的泄漏那一格用第 2 条就修掉，反向接受条款那一问（「要核加密开启时重建还读不读得到它」）的答案是读得到，与五元组同一条路径。

## 六、U4 检查：I-9.9 在各臂下判得了什么

手造 H：建对象、写两个 offset，再把记录里的 locality 改错（extent key 仍在原首段），看 I-9.9 在「未经重建」与「extent 树整棵重建、记录在」两种状态下判什么。`output.txt` 的 H 行与断言整行抄：

```
H injected=False jia              intact=green       rebuilt=green       rebuilt_missing=0
H injected=False yi_per_unit      intact=green       rebuilt=green       rebuilt_missing=0
H injected=False yi_per_object    intact=green       rebuilt=undecidable rebuilt_missing=0
H injected=False bing             intact=green       rebuilt=undecidable rebuilt_missing=0
H injected=False bing_inode_only  intact=green       rebuilt=undecidable rebuilt_missing=0
H injected=True  jia              intact=red         rebuilt=red         rebuilt_missing=2
H injected=True  yi_per_unit      intact=red         rebuilt=red         rebuilt_missing=2
H injected=True  yi_per_object    intact=red         rebuilt=undecidable rebuilt_missing=0
H injected=True  bing             intact=red         rebuilt=undecidable rebuilt_missing=0
H injected=True  bing_inode_only  intact=red         rebuilt=undecidable rebuilt_missing=0
assert H 未经重建 jia：干净绿、注入红                                             got=('green', 'red') want=('green', 'red') ok
assert H 未经重建 yi_per_unit：干净绿、注入红                                     got=('green', 'red') want=('green', 'red') ok
assert H 未经重建 yi_per_object：干净绿、注入红                                   got=('green', 'red') want=('green', 'red') ok
assert H 未经重建 bing：干净绿、注入红                                            got=('green', 'red') want=('green', 'red') ok
assert H 未经重建 bing_inode_only：干净绿、注入红                                 got=('green', 'red') want=('green', 'red') ok
assert H extent 树重建后 丙：干净与注入同为不可判定                                    got=('undecidable', 'undecidable') want=('undecidable', 'undecidable') ok
assert H extent 树重建后 甲：干净绿、注入红                                        got=('green', 'red') want=('green', 'red') ok
## H′ 乙逐单元读法：合法的重新聚簇之后（不注入任何错），记录在、extent 树重建
assert H′ 乙逐单元：合法镜像上重建后 I-9.9 判红（误报）                                  got=red    want=red    ok
assert H′ 乙逐单元：记录还在也找不到的 extent 数                                     got=1      want=1      ok
```

- **未经重建**：五条读法都判得了（记录与 extent 树两侧独立），判别力自证全过：干净绿、注入红。
- **extent 树整棵重建、记录在**：丙与乙按对象读法的首段取自记录 ⇒ 两侧同源 ⇒ 不可判定（干净与注入同为 undecidable），与 `invariants.md:265` 写的一致。甲的首段取自单元头 ⇒ 不同源 ⇒ 干净绿、注入红，自证过。乙逐单元读法数字上也是绿 / 红，但 H′ 在一个完全合法的镜像上判红 ⇒ 有重新聚簇时乙这一侧的红不可信，只能输出不可判定。
- **甲那一格抓到的是什么**：一个被写错的记录 locality。丙之下它的后果只有局部性（重建按记录编首段，读者全都找得到：H 注入那一行 `bing ... rebuilt_missing=0`）；甲的字面读法下同一个错让读者找不到 2 个 extent（`jia ... rebuilt_missing=2`），再由 I-9.9 判红报出来。⇒ 甲把一个在丙下无害的错变成有害的错，然后抓住它。甲按 keys_first 收严（单元头优先于记录）时读者全找得到，「记录与头不等」在重建那一刻报出来、按 `18-块里携带什么信息.md:521` 不许写回——这是甲在 U4 上真正多买到的一格。
- **记录 ↔ 单元头之间已经有一条独立见证**：I-9.10（对象出生代三处一致，`invariants.md:266`，已实现）。甲 / 乙是给同一条记录再加一个见证字段，不是从无到有。
- **I-9.9 的条文要不要改**：`invariants.md:265` 把「不可判定」挂在全池的 `map_provenance = rebuilt` 上，不按哪一侧重建过分。甲那一格的判别力要落地，I-9.9 得改成按侧判（记录这一侧没重建、key 那一侧取自单元头 ⇒ 可判）。丙不用改。
- **按 U4 触发观测的字面记账**：重建之前五条读法都写得出自证；重建之后丙没有判别力（I-9.9 自己写明的设计，不是写不出），乙在有重新聚簇时写不出（误报）。字面上各记一次，丙那一次算不算交主 agent 判。

## 七、各臂要改哪几句 kb（kb 文件自己的行号）

| 臂 | 要改的句子 | 改成什么 |
|---|---|---|
| 甲 | `09-加密.md:524`（AAD 的定义） | 六元组，规范编码 41 字节 |
| 甲 | `09-加密.md:553`（「一定不能进 AAD」表里 `locality_id` 那一行） | 删行并写撤销依据；按第四节，前置是先有一条条款定「locality 一经写出永不改变」 |
| 甲 | `08-核心索引结构.md:137`、`:198` | 与上一条相容：写明不再有重新聚簇，或写明重新聚簇 = 重加密全部被重编单元 |
| 甲 / 乙 | `18-块里携带什么信息.md:395`、`:396`（五元组与 33 的宽度） | 甲：六个字段、41；乙：五元组照旧，另加一个头字段 |
| 甲 / 乙 | `18-块里携带什么信息.md:398–404`（那条 ⚠️ 提议） | 改成定案 |
| 甲 / 乙 | 码 1 类身份段表（`18-块里携带什么信息.md:591–600`）、`first-txn-layout.md:170–185` | 头 105 → 113、含预留位 134 → 142，五元组之后的偏移重排 |
| 甲 / 乙 | `27-小数据打包容器.md:281`（槽自带五元组） | 写明槽带不带 locality；不带就写明 ≤ 4 KiB 那一档照丙的规则 |
| 乙 | `18-块里携带什么信息.md:394` | 头字段从此不等于 AAD 那一组 |
| 乙 | `18-块里携带什么信息.md:602–604` | 取原文（明文）就要改已定项 13 的白名单，那是推翻 `:413` 的 2026-09-07 用户定案；取「进密文」只要把 locality 列进「就地变密文」那一串 |
| 丙 | `18-块里携带什么信息.md:398–404` | 提议不收；「放不回去」改成「放得回去，首段按规则重编，丢的是局部性」 |
| 丙 | `08-核心索引结构.md:206` | 「没法重新编 key」改成「重编之后的值要有地方存」 |
| 丙 | D18 已定项 5（`18-块里携带什么信息.md:495` 起）或 D8 已定项 6 的「重建」段（`08-核心索引结构.md:422–424`） | 写进重建规则：对象按 (树 ID, inode) 认；幸存的 extent key 优先于记录；两样都没有取 0 |
| 三臂共同 | `08-核心索引结构.md:198` 与 `invariants.md:265` | 大对象的重新聚簇怎么做才不在合法镜像上违反 I-9.9（第三节末那一格） |

## 八、我提的收严（都只在本模型上量过，被攻过零轮）

| # | 收严 | 修哪一格 | 本模型上的读数 |
|---|---|---|---|
| S1 | 重建时幸存的 extent key 优先于记录（keys_first） | 打中 1，三臂共同 | rekey_on 深度 6 失败：甲 55620 → 0、乙按对象 21784 → 0、丙 21784 → 0 |
| S2 | 丙的「这个对象」= (树 ID, inode)，幸存 key 只看本头的 extent 树 | 打中 2 | bing_inode_only 29400 / 19616（rekey_on）、1120 / 2464（rekey_off）→ bing 全 0 |
| S3 | 乙按对象只取一个值，幸存 key 与记录都优先于头提示 | 打中 3 | yi_per_unit keys_first 13440 → yi_per_object keys_first 0 |
| S4 | 乙在加密开启时这 8 字节进密文 | 第四节乙、第五节泄漏 | 没有模型读数，是条款推理 |
| S5 | I-9.9 的「不可判定」按侧判，不按全池 `map_provenance` | 第六节甲那一格 | 手造 H：甲重建后干净绿、注入红 |

按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严……算「没被攻过」」：S1–S5 交用户时要写明被攻过零轮，各自的检查另记账。

## 九、什么现象会推翻每条结论

| 结论 | 推翻它的观测 |
|---|---|
| 丙在收严读法下 U1 不中 | 一段合法历史让同一个头里同一对象的幸存 extent key 有两个首段（模型里 ambiguous 恒 0 被打破），或读者按重建后记录找不到数据；模型外最可能的来源是分批重编（第三节末） |
| 丙字面读法与乙按对象读法同病（打中 1 不分辨臂） | 一条历史让丙 record_first 失败而乙按对象 record_first 不失败（两格今天都是 21784） |
| 甲在重新聚簇上要重写单元 | 一条条款写明 locality 一经写出永不改变、并给出不重编 key 的局部性修复；或一个不换 nonce 也能安全改 AAD 的构造 |
| 前提三与前提五不互相顶着 | D8 里出现一句要求「单元 ↔ 首段」这一对永不变的原文。`grep -n locality 08-核心索引结构.md` 的 15 行（:71、:132、:136、:146、:157、:161、:171、:173、:174、:179、:196、:202、:205、:397、:427）我逐行读过，没有；:196 与 :202 只说不随改名更新 |
| 乙原文与已定项 13 冲突 | 已定项 13 的明文白名单给提示字段留过位（`18-块里携带什么信息.md:602` 写的是只剩两样） |
| 乙进密文在重建上零代价 | 加密开启时有一条重建路径不拿密钥也要读 locality（`03-空间分配.md:17–18` 写无密钥侧只读只检测） |
| 第一版甲 / 乙的 8 字节是 0 bit | 第一版出现 locality ≠ 0 的对象（`08-核心索引结构.md:397` 与 `first-txn-layout.md` 都写 0） |
| G 不分辨臂 | 一条规则让甲在实例表丢失后按头里的 locality 判出孤儿，而不靠两次创建碰巧落在不同目录 |
| U4：甲在重建后多一格判别力，且那一格抓的错在丙下无害 | 找到一个「记录 locality 写错」在丙的重建下也让读者找不到数据的历史（手造 H 里丙 `rebuilt_missing=0`） |

## 十、复跑与指纹

```
python3 research/prompts/d18-locality-r1-opus-model/model.py > /tmp/claude-1000/d18-locality-opus.out
diff /tmp/claude-1000/d18-locality-opus.out research/prompts/d18-locality-r1-opus-model/output.txt
```

- 默认深度 6 / 6 / 7，本机实测 2 分 12 秒（`time`）；`model.py N` 把三个世界都改成深度 N。只用 std 的 Python，没有 Rust、没有 target 目录。
- 确定性：无随机源、无 I/O，复跑一致只说明没有隐藏状态，不是统计意义上稳定。输出里 39 条 `assert … ok`、0 条 FAIL（`grep -c`）。
- sha256：
  - `29b9e238b41d3c66967b84ced2cc0b046ea1abb8cbdcfa06a6059dc010bae126  research/prompts/d18-locality-r1-opus-model/model.py`
  - `c5cfab242f7900b6c91a5b7536fe91c3c0fcd29acc27e35bd1c6f3dd8bcacc9d  research/prompts/d18-locality-r1-opus-model/output.txt`
- 射程与没建的：单元归属与现行版本判定当神谕（C113、克隆祖先表）；删除 / 墓碑、reflink（今天发不出去，`21-权威态与派生态的分界.md:363`）、D27 打包、分批重编都没建；每容器 2 条记录、每头最多建 2 个对象、两个 offset 是小规模取样点，不是真值。
- 过程记录：模型第一段源码一次写了 155 行，超了每次 150 行的上限；当场删掉、分两段重写（95 + 60 行），与原稿 `cmp` 逐字节相同。

## 十一、判决摘要（给主 agent）

- **丙**：U1 没被正确性后果打中。收严读法（S1 + S2，零字节）下 rekey_on / rekey_off 深度 6、第一版深度 7 都是 0 失败。字面读法的反例（打中 1）三臂共同、不分辨；最弱读法的反例（打中 2）零字节可修。丙付的是重建之后的局部性（rekey_off 872232 次对象检查里 263476 次），而重建产物本来就只读（`18-块里携带什么信息.md:509`）。要改的是两句事实性措辞（第七节），不是契约。
- **甲**：U2 / U3 被打中，条件是「后台重新聚簇 = 重编 key」（手造 A：2 个单元强制重写、2 个共享被拆开；B：不重写就 MAC 败 2）。前提三那句顶不掉前提五：两句管的不是同一对东西。那个动作全仓没有定义 ⇒ 这一击是条件式的，条件交主 agent 定；另外甲把一个取值规则还没写的字段冻进第 1 层与永久 AAD。
- **乙**：原文（加密开启时明文）在 U2 被已定项 13 打中；逐单元读法在 U1 被打中（3 步 / 4 步）。按 S3 + S4 收严之后站得住，等于「丙 + 记录与 key 都丢时用头提示代替 0」：每个码 1 单元 +8、第一个事务改字节，第一版携带 0 bit，买到的是重建之后的局部性与 I-9.9 按侧判的那一格。
- **不拿来判臂的两处共同前提洞**：实例表不可读时的孤儿（G）；「增量、可中断」的重新聚簇与 I-9.9 在大对象上不能同真（第三节末，推理）。
- **按跑前条款**：丙没被打中，甲出局有条件，乙原文被泄漏那一格打中 ⇒ 落到「交用户在『乙 + 进密文』与丙之间选」。攻方这条腿没能把丙打穿；如果交用户，丙那一侧要带上 S1 / S2 写进重建规则，乙那一侧要带上 S3 / S4，四条都被攻过零轮。
