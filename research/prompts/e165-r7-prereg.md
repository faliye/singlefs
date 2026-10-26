# E165 第 7 次重跑登记（r7）：C743 之后对抗根环照落点公式摆根（新域 KL），KJ2 在它上面判什么，择根两族的产物重出

<!-- doc-lint:not-numbers Q1 Q2 K K9 KW KA KL KF KF3 KF9 KFW KG KJ0 KJ1 KJ1T KJ2 KJ3 KF1 KF2 TQ6 TQ10 TF3 TF5 TF7 TF8 TV4 TV10 TH2 TH4 L1 L2 L3 L4 LQ1 LQ2 LQ3 LQ4 LQ5 LQ6 LQ7 LQ8 LF1 LF2 LF3 LF4 LV1 LV2 LV3 LV4 LV5 LS1 LS2 LS3 H1 H2 H3 H4 H5 H6 H7 H8 H9 -->

写于 2026-10-23，装置改动之前、任何新产物之前。不占号。上几次登记：r3 `research/prompts/e165-r3-prereg.md`（sha256 `1859fe965ad926d89e67cd7a00d5bc1d56b975b7f665d61d0991456528f0e4ba`，KA 的出处：5.4、5.7、第七节乙类 KA、9.1）；r4 `research/prompts/e165-r4-prereg.md`（sha256 `6a07eddef0a0de0f405bd11202880f8e638198c2f7d6daad3c8215d62fbb9156`，带读故障一族里「KA 是 r3 原样」）；r5 `research/prompts/e165-r5-prereg.md`（sha256 `02d6a539ad1f0bcb5c8d55101a618af1a95d50d8265bc95fa05f33b2f292bf17`，修订 1 的 R2 闸一族）。第 6 次登记（`research/prompts/e165-r6-prereg.md`）已归档（`git log` 里由 `04ceaa904` 删），r3 档合法一族，与择根两族无关，没读。调查报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-checker-tier-pins/result.md`（sha256 `c594587504890416d3713008adaa0c0540c9b2f4a074654651d4ee5d65faa2cd`）第三节 3.5、第六节第 11 条只当线索。开工时 HEAD `841b3beab`。

这一次不是「输入变了、同臂重跑」的差分登记：输入变了（C743 落点核），而且一个域的定义要重写（用户 2026-10-23 定），所以各节照写；r3、r4 里没变的条目写成「同 r3 第 X 节（第 A–B 行）」，以上面那个 sha256 的版本为准。

## 一、问题

英文名：replay_bounded_enumeration（照抄 r5 第 13 行；装置仍是 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`，不另起；驱动仍是 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh`）

### 1.1 主 agent 给的问题（派发原文，逐字）

> **e165 KA 域改登记**：用户 2026-10-23 弹窗定「照落点公式摆根（推荐）」——KA 改成照 C743 的落点公式摆根、走真 `choose_root` 三步；同 txg 两条根不能同在，KJ2 的平局那半没了。要改的是 `research/prompts/e165-r3-prereg.md` 5.4 与第七节，r4 档的 KA（装置 `e165…rs` 第 13046 行一带是 r3 原样）一起。验收：写一份重跑登记，写明 KA 新域怎么造（照落点公式摆哪几条根、域多大）、KJ2 那一判从此判什么或不判、钉表重放 `.claude/gate.d/e165-pinned-red-examples.tsv` 受不受影响（调查员现查 385 行里没有 KA 的行）。

被测条款（派发原文）：根环一簇 C743（读根环核落点公式，D22（单元原子性怎么合成） 里「一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根」那一句）。岔路单（派发原文）：重跑。

### 1.2 问题单

派发提示没有另给问题单。下面四行按 1.1 的验收写，**待主 agent 认**（文末「待主 agent 认的」第 1 条），认之前不派执行员。四行叫 L1–L4。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| L1 | 对抗根环照落点公式摆根之后（新域 KL，1.3），今天的三步在 KL 上：状态数、根环全空的态数、KJ0、KJ2 与本地择根序的 KJ2 读数，等于独立锚点吗；r3 档（`--family root_choice`）与 r4 档（`--family root_choice_faults`，含 `--gate r2`）两处 KL 一样吗 | 都等 / 哪几格不等 | 任一格与第七节不等 | 三份产物的 KL 行逐格比完 | 开着 |
| L2 | KA 换成 KL 之后，三份择根产物里 KA / KL 之外的每一行与上一次的产物逐字节相同吗 | 相同 / 哪几行不同 | 任一行（照 1.3「哪些行算 KL 行、哪些算汇总行」分）不同 | 三份产物逐行比完 | 开着 |
| L3 | 改坏择根序的六个变体（v1822–v1827）在 KL 上的 KJ2 读数等于第七节的钉值吗；v1823、v1825、v1827 在 KL 上读 0 之后，它们的变体臂阳性对照由谁担 | 等 / 不等；对照改由变异表那一行点名的 crates 测试担 / 别的 | 任一变体读数不等于钉值；或那条 crates 测试在变体上不红 | 第二段跑完（附带，主 agent 判续不续） | 开着 |
| L4 | 钉住红例表重放（`.claude/gate.d/e165-pinned-red-examples.tsv`，门禁 checker-tier-formal-verification 的 pinned-red-examples-replay 格）受不受 KA → KL 影响 | 不受 / 受 | 表里有一行的域是 KA，或重放用法认 `--family root_choice` / `root_choice_faults` | 设计时数表、读用法（第四节 4.6） | 设计时够判（不受），待主 agent 认 |

### 1.3 读法写死（装置改之前定下，改了就是新的一次）

- **新域 KL**（对抗根环，根照落点公式摆）：txg 取 100、101、102 三个，每个 txg 只在它的落点槽里摆根——落点照 D22 已定项 16（第二节）的「区域内槽位 = (txg div R) mod S」与区域 = txg mod R（R = 3、S = 8，实现 `crates/singlefs-core/src/root_ring.rs:167` `target_for_publish`）：txg 102 → 区域 0 槽 2，txg 100 → 区域 1 槽 1，txg 101 → 区域 2 槽 1（读根环次序里依次是第 2、9、17 个槽，`recovery.rs:1699–1708`「区域号升序、区域内槽号升序」）。每个落点槽取空、或那个 txg 的实例 1、实例 2 两条根之一：3³ = **27 态**。其余根槽全 0（不摆第 0 代根）、journal 环全 0、S = 8、journal 环 64 槽，同 r3 的 KA。新根段六条各不相同（同 r3 KA「新根段各不相同」）。走真三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`），同 r3 5.5。
- **为什么三个 txg**：两个 txg（100、101）给 9 态，读根环次序里旧根（txg 100）总在新根（txg 101）前面，只有一个方向；加 txg 102（区域 0，读在最前）就有了「新根在前」的方向，同一个域里两种次序都在（第八节几何敏感性的两点由同一个域给出）。只取 100、101 的 9 态读法也写进第七节，主 agent 要它就在装置写之前改，改了是新的一次。
- **名字**：定义变了，起新名 KL（照 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）；KA 这个名字留给 r3 的旧定义——旧产物与冻结根上的驱动仍按 KA 读。装置里 `RootChoiceDomain::AdversarialRootRing` 的标签由 KA 换成 KL，`--only` 认 KL、不再认 KA。
- **KJ2 在 KL 上判什么**：判据字面同 r3 5.5（「落到的版本 ≥ 真相里根环每个槽现存的根」）。KL 里每条根都住在它的落点槽，C743 的落点核不挡任何一条，真相里的根就是择根看得到的全部根；同 txg 两条根不能同在，「同 txg 不同实例的平局」那一半没有对象（不判，不是判为 0）。KJ2 在 KL 上判的是剩下的两样：实例代号更高而 txg 更低的根不压过 txg 更高的根（KL 里 7 态有这样一对），读根环次序靠前的旧根不压过靠后的新根（KL 里 4 态读在最前的那条不是最新）。KJ0 照判（根环全空那 1 态记一条，同 r3）；KJ1、KJ3 同 r3 修订 2 第 5 条不出行。合法历史三域 K、K9、KW 上 KJ2 的定义不变。
- **哪些行算 KL 行、哪些算汇总行（L2 的比法）**：一行里有字段 `domain=KA`（旧产物）或 `domain=KL`（新产物），或 `name=control` 行的 `measured=` 里含 `domain=KA` / `domain=KL` 的，算 KL 行，归 L1 判、不进 L2；`name=config` 行只许 `only=` 等值里 KA → KL 那一处标签不同；不带 `domain=` 的别的行（收尾、全族合计、判决汇总）是汇总行，逐字段比，不同的字段必须恰好等于「KL 的那一项减 KA 的那一项」（例：全族状态数差 27 − 125 = −98，KJ2 被判态数差 26 − 124 = −98），别的差都算 L2 翻面；其余行逐字节比。
- **「上一次的产物」**：第十一节比对对象表三行（今天 `replay.sh` 第 1679、1681、1691 行指的那三份）。
- **线程上限与内存上限**：线程取登记值（第十四节）；内存照比对对象各自的跑法（`replay.sh` 注释：r3 那一份第二段峰值 446 MiB，r4 那一份约 1.7 GB，`REPLAY_MEMORY_CAP` 默认 8G）。

### 1.4 装置写在哪（写死）

**入库装置**，改原装置 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`：L1、L2 问的是 `crates/` 今天的 `choose_root`（连同 C743 的落点核）在这一族上的结局，另写一份独立模型答不了；独立的那一半是第七节的两份脚本（不读装置、不调 crates）。

要读 `crates/` 的常量：不加新的。KL 摆根的落点用装置已有的本地算法 `root_choice_target_of_publish`（今天第 9805 行，区域 = txg mod 3、槽 = (txg div 3) mod S），每摆一条根都过装置已有的 TH2 回比（第 9844–9860 行：本地 (区域, 槽) 与 `root_ring::target_for_publish` 相等），不直接调 `target_for_publish` 去摆根。第 0 代根的落点（`make_filesystem.rs:825` `genesis_root_ring_slots`）KL 用不到。

### 1.5 重型测试与谁跑

装置的内联测试与 `crates/mutations.tsv` 里这只装置的行属 checker 档，执行员不跑，写成 5.8 的「要主 agent 跑的命令」。三份产物是 `cargo run` 起装置本体：`.claude/hooks/lib_heavy_tests.py` 第 77 行写「`cargo run` 起 checker 档包自己的装置二进制也照旧不判」，由执行员经内存包装跑。改装置的 `.rs` 走 `scripts/hosts/compile-then-swap.py`（执行员在主工作区写 `crates/` 下的 `.rs` 被写闸拒）；主 agent 改派实现员也行，5.6 的表照用。第二段的驱动编变体、跑变体的装置本体，由执行员起；钩子拒了就写进交回。

## 二、被测条款与它引的定义

被测条款是 D22（单元原子性怎么合成） 已定项 7 实例代号那一行里 C743 那几句（「读根环时每条根照落点公式（已定项 16）核槽位，第 0 代根照 mkfs 种根的槽另认，一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根、取读根环次序最前的」）与它引的已定项 16 的落点公式。以下由 `scripts/kb/quote-kb.py` 整段抄进草稿目录的 `e165-section2-quote.md`、回读逐字节一致之后原样追加（四段：索引表第 7、16 行用 `~正则` 取整行，已定项 7、已定项 16 用 `@标题` 取）。KJ0–KJ3 的判据出处（D13 已定项 1、D22 已定项 24 等）同 r3 第二节（第 39–317 行），这一次不重抄。
**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:18-18`（整段抄，未转述）**

```markdown
| 7 | **根记录的字段表** | 恒走间接层：固定头 + 一条树表单元指针，树根不平铺进根槽；17 个字段合计 466 字节 **状态：已定。** **进度：未实现。** |
```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:28-28`（整段抄，未转述）**

```markdown
| 16 | **系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位** | 系统配置每盘 2 槽按世代号 mod 2 轮换（mkfs 两槽都是 1）；根环起点是 16 KiB 槽号；区域内槽位 = (txg div R) mod S；归属 0 / 1 / 0 **状态：已定。** **进度：已实现。** |
```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:132-176`（整段抄，未转述）**

```markdown
#### 已定项 7：根记录的字段表

**定案**：**根记录恒走间接层：固定头 + 一条指向「树表单元」的指针，树根一律住树表单元，不平铺进根槽。** **不做「树少时平铺、树多时间接」的分支**：分支变量是树数，而树数在挂载期间会变（建一个可写头就加一棵树，D6（快照实现模型）已定项 1）——撞 `.claude/rules/fs-design.md` 硬要求 5「分支变量不许在操作中途改变」。恒一种形态。

| 字段 | 宽 | 出处 |
|---|---|---|
| magic | 4 | — |
| fsid | 16 | I-1.4（块头 fsid 一致）同口径 |
| flags | 4 | 位 0 = 卸载记号：正常卸载那一串写的根置 1（D16（发布语义） 已定项 1「正常卸载」），其余根写 0，池级 checker 按它给 I-7.9（回退下界 F 不高于抬 F 的上限） 分支；其余位第一版恒 0、非 0 拒收（D22（单元原子性怎么合成） 已定项 17）。位 n 住第 n div 8 个字节的第 n mod 8 低位（已定项 13） |
| **实例代号** | **4** | D16（发布语义）已定项 6 骑手 1：与 D23（journal 的角色与格式）已定项 9 的 journal 实例代号**同一个计数**；择新在 checkpoint_txg 平局时按它高者赢（读根环时每条根照落点公式（已定项 16）核槽位，第 0 代根照 mkfs 种根的槽另认，一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根、取读根环次序最前的（已定项 24）；公式核还没实现，欠在 [checks-owed.md](../checks-owed.md) C743（重建分配器记新类时不核根住不住这一槽）） |
| **checkpoint_txg** | 8 | D16（发布语义）已定项 6：发布计数、轮转键、记账代、重放水位是同一个数——**不另设「发布代号」字段** |
| 树表单元指针 | **86** | 指针头部 50 + 位置条目 **14** × 2 副本 + 实例代号 4 + 出生序号 4（D19（块指针的结构与宽度预算） 已定项 7 / 8；位置条目 = 设备 ID 4 + 16 KiB 槽号 6 + 密文校验和 4，D19（块指针的结构与宽度预算） 已定项 4）；自举豁免，位置条目是权威 |
| **树 ID 水位** | **8** | D8（核心索引结构） 已定项 8 ②：`max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，每次发布随根记录重写 |
| **回退下界 F** | **8** | D16（发布语义） 已定项 1：可再分配的界 max(F_生效, 环里最旧有效根)；平时不动，准入不够时抬，正常卸载时抬到现行那一版的 txg；系统配置另记一份（已定项 9），恢复后生效值取两处读得出的最大值（D16（发布语义） 已定项 1「生效」） |
| 自证校验和 | 32 | 与 E23（journal 几何）的 `header_csum` 同宽 |
| **实例表单元指针** | **86** | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2：实例表单元（D18（块里携带什么信息） 已定项 11 打包记录类型 4）不进容器索引，由根记录直接持有；结构与树表单元指针同型 |
| **中央映射树根指针** | **86** | D19（块指针的结构与宽度预算） 已定项 11：中央映射树的根住根记录、不进树表，与树表单元指针同形（自举豁免）；mkfs 与暖机写全 0 |
| **分配记录树根指针** | **86** | D16（发布语义） 已定项 9：树表 0 条的一版上写行那次发布写的那一片分配记录节点；只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0；与树表单元指针同型 |
| **回收门槛 t** | **8** | D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」：写这条根时的回收门槛，取 max(这条根自己带的 F, 分配器记这条根时的回收门槛)，让读坏或读回全零的根槽钉住的门槛随根落盘，不只在这个进程的内存里；零单元发布照抄上一版的值，mkfs 的第 0 代写 0；池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 判 |
| **钉槽 p** | **1** | D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」：建这一版的账时还钉着的根环槽里钉值最小的那个，钉值 = 3·槽 + 区域，就是那一槽可能住过的最旧 txg（小于环长 3S）；没有钉着的槽写 255（`ROOT_RECORD_NO_RING_SLOT_PINNED`）；钉着的槽在这个进程写掉它时必解，那一槽已住着本实例写的根而 p 还指着它，池级 checker 的 I-7.19（钉着的槽不住本实例写的根） 判红 |
| **算法类型** | **1** | D9（加密） 已定项 12 第 1 项（射程扩到自证单元）：0 = 未加密；登记表在 D9（加密），D22（单元原子性怎么合成） 已定项 17 |
| **nonce** | **12** | D9（加密） 已定项 12 第 1 项；与单元头那 12 字节同口径（D18（块里携带什么信息） 已定项 14） |
| **MAC** | **16** | D9（加密） 已定项 8 同口径；根记录是自证结构，加密开启时由它自己的 MAC 罩住 |

行序即盘上顺序，偏移按行累加：magic 0 / fsid 4 / flags 20 / 实例代号 24 / checkpoint_txg 28 / 树表单元指针 36 / 树 ID 水位 122 / 回退下界 F 130 / 自证校验和 138 / 实例表单元指针 170 / 中央映射树根指针 256 / 分配记录树根指针 342 / 回收门槛 t 428 / 钉槽 p 436 / 算法类型 437 / nonce 438 / MAC 450。

<!-- format-const: ROOT_RECORD_BYTES = 466 stale=根记录 371 字节|根记录合计 371|14 个字段合计 371 字节|合计 **371 字节**，512 槽内余量 141|3 × 371|合计 **457 字节**|根记录宽 457|15 个字段合计 457|457 / 512 槽|3 × 457|ROOT_RECORD_BYTES=457|ROOT_RECORD_BYTES = 457|`ROOT_RECORD_BYTES` 是 457|根记录（457 字节|根槽偏移 428..457|15 个字段合计|512 槽内余量 55|根记录合计 457|根记录 **457 字节**|根记录 [0, 457)|[457, 槽宽) -->
合计 **466 字节**，512 槽内余量 46——槽宽是运行时探测值，余量给将来字段。

**树表单元**：16 KiB 元数据单元，装 **81 棵/层**，超过再嵌一层（81² = 6561 棵）；条目宽取 D8（核心索引结构） 已定项 8 定的 **200 字节**（树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + 头 ID 8 + 预留 76），头宽取树表单元自己那个码 2 头 **131**（key = 树 ID、key 宽 8 ⇒ 115 + 16，D18（块里携带什么信息） 已定项 16 / 已定项 18）：⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81**。⚠️ **分母只有一种读法**：16384 减含预留位的码 2 头，不减别的数（C157（树表容量在两处按不同条目宽算） 要的正是这个统一口径；D8（核心索引结构） 已定项 8 那一处逐字相同）。树表单元的头字段按 D18（块里携带什么信息）已定项 7 的索引节点类身份段走；它是权威态走读的一部分（I-7.2（最新根可完整遍历）的「走完全部权威态」经它下去）。

**射程**：只管根记录与它指向的树表单元；系统配置不带树表单元指针（已定项 12）。⚠️ **重放水位的读法**（D23（journal 的角色与格式）已定项 14）：恢复只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录；tail 只是扫描起点的优化。间接层的代价是每次发布多 COW 一个树表单元，知情接受；树多过一层嵌第二层之后每次发布多 COW 几个单元没算成绝对值（C148（树表条目变宽把间接层门槛压到二层））。

**依据**：

- E79（根记录的容量）：根槽宽 = `physical_block_size` × 树表 day-1 开放（D15（格式冻结政策） 已定项 2）× 每头一棵树（D6（快照实现模型） 已定项 1）三条合起来，平铺装不下这个设计自己已定的东西 ⇒ 间接层结构上躲不开。
- 末尾 29 字节（算法类型 + nonce + MAC）：根记录与 journal 记录头是自证结构，没有父指针带 MAC 与 nonce ⇒ 不 day-1 留位，开加密那天它们要么整条走明文豁免（泄漏 `checkpoint_txg`、树表指针、映射根指针、树 ID 水位），要么就是一次布局变更；D9（加密） 已定项 3 的原则是为加密留的位所有卷都留、不挪作他用。
- 用户定案（间接层与字段表、树 ID 水位、实例表单元指针、回退下界 F、中央映射树根指针、三条指针 86、末尾 29 字节），原话在变更史；三条指针变宽与末尾 29 字节没有走三方论证，可推翻。
- 用户定案 2026-09-23：加分配记录树根指针 86 字节（C512（树表 0 条的一版上被换下的实例表记在哪没有条款）），合计 371 → 457，原话在变更史。
- 用户定案 2026-10-20：根记录加回收门槛 t 8 字节与钉槽 p 1 字节（C773（读坏与全零根槽的回收门槛钉只在内存里）；D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」），合计 457 → 466，原话在变更史。t 在偏移 428、p 在 436、算法类型 nonce MAC 往后挪是主 agent 定的，照根环第二轮正推腿 Y4.2（`research/prompts/m3-close-rootring-r2-sonnet-output.md`「Y4.2 字段」），没被攻过，零轮；无实验：补的是两个字段的宽度与偏移，没立实验号。
- 用户定案 2026-09-26：flags 位 0 当卸载记号（I-7.9（回退下界 F 不高于抬 F 的上限） 取 (b)+(c) 合用），原话在变更史；无实验：三方原型 `research/prompts/m2-rollback-forward-r3-opus-model/` 量了三条改法的判别力（第三轮判决 K6：(a) 丢「准入抬过头」整类出局；(b) 丢「准入抬过头却带了卸载记号」那一格；(c) 池级镜像不判），没立实验号；合用是主 agent 推的，被攻过零轮。
- 实例代号那一行的平局：根环三方第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C743（重建分配器记新类时不核根住不住这一槽） 那一格（同 txg 两条根并存与已定项 16 的落点公式冲突，改的是这一行的平局括号；第 0 代例外是第一轮打中裸公式之后补的，第二轮攻过没打中，第三轮辩方站住）。

**欠**：C88（根环的时间线判别未实现）；C148（树表条目变宽把间接层门槛压到二层）；C308（自证单元的字段偏移没定）；C743（重建分配器记新类时不核根住不住这一槽）：读根环时的落点公式核。（C78（根记录没有字段表） 2026-10-11 已还清，并入 C66（格式级空白没有分项号）；C157（树表容量在两处按不同条目宽算） 2026-10-11 已还清，并入 C148（树表条目变宽把间接层门槛压到二层），见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:345-372`（整段抄，未转述）**

```markdown
#### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位

**定案**：系统配置槽：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距（D2（RAID 条带策略） 已定项 19），槽世代号从 1 起（mkfs 写下的两槽世代号都是 1）、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）。读槽 1 的偏移按这个次序取：本盘槽 0 可用（自证过、几何字段都在这一版读者收的范围里）时取它自述的槽距；否则取池里按设备次序第一块可用槽 0 自述的槽距；全池没有可用的系统配置槽 0 时，读者在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096），只收自证过、且自述的固定结构槽距等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；逐档读到自证过而池级字段越界的槽整池拒（第 5 句「越界即拒」），incompat 位不认得的槽不可择、不核偏移。逐档读到自证过的槽先判它带不带这一版读者不收之值（含 I-8.1（环几何够大） 三个 journal 环几何字段），带的整池拒、不再核偏移；不带的再核自述槽距等于所在偏移。根环：`r × P × chunk` 是设备内偏移，基址 = 系统配置「根环起点」字段（第一版 1 MiB），区域 r 的起点 = 基址 + r × P × chunk；区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定，升格进已定项 2 的参数表。五句写死：

1. **系统配置「根环起点」字段是 16 KiB 槽号**，与 D3（空间分配） 已定项 7 的落点槽号、D19（块指针的结构与宽度预算） 已定项 4 的位置条目物理偏移同一个坐标系；第一版写 **64**（= 1 MiB ÷ 16384）。基址（设备内字节偏移）= 该字段 × 16384；区域 r 的起点 = 基址 + r × P × chunk。⚠️ 字段里写的是 64，不是 1048576。读者解槽时读这个字段与 journal 环起点字段（偏移 325），与第一版常量（64、1024）不等一律整池拒；根环区域数 R（偏移 361 那 1 字节）同样读，不等于 3 整池拒，每区槽数 S 越界照已定项 2 整池拒；素数步长 P（偏移 363 那 4 字节）与 chunk（偏移 367 那 4 字节）同样读，与第一版常量（3、1 MiB）不等整池拒，两者都等于常量之后再判 S × 固定结构槽距 ≤ P × chunk（区域不越过下一个区域的起点，写 ≤），越界与 S 越界同一个处置（用户 2026-10-22 定，原话在变更史）；一槽 incompat 位不认得而格式版本又不是 1 时先按格式版本整池拒，不当不可择（C663（系统配置槽越界与不收之值两读者分类不同），用户 2026-10-20 定，原话在变更史；core 是 `crates/singlefs-core/src/recovery.rs` 的 `RootRingRegionCountNotTheFirstVersionValue` 与 `mountable_slot_or_refusal`，checker 是 `crates/singlefs-checker/src/image.rs` 的 `value_refused_by_this_reader`）；判固定结构槽距的上界用同一槽自述的根环起点，checker 的 I-7.13（系统配置池级字段在读者收的范围里） 与 core 收同一张表（用户定，起因与依据在代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`「core 与 checker 对系统配置几何字段的读法」那一格）。已实现：core 的 `crates/singlefs-core/src/system_configuration.rs` `parse_slot` 按偏移 325、371 读进 `SystemImmutableConfiguration` 的 `journal_ring_start_slot`、`root_ring_base_slot`；`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 择系统配置时判，不等报 `SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot` / `JournalRingStartNotTheFirstVersionSlot`，槽距上界取 `largest_fixed_structure_slot_spacing_under_the_root_ring_base`（根环起点 × 16384 − 4096；逐档找槽 1 那一处还没有哪一槽可读，仍用第一版常量）；checker 的 `crates/singlefs-checker/src/image.rs` `geometry_of` 判同一张表（325、371 等于第一版常量，417 = 环长现算的起点且在 64 槽段边界上）。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write`（攻方五格只读与可写都拒、盘上不变）；`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`；两边在同一份镜像上一起判的是 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs`（五格各一条：core 两种挂载都整池拒、盘上不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 并点名它拒的成员；根环起点写 0 那一格两边报的成员不同——core 先判槽距上界、报槽距越界，checker 先判两个起点、报根环起点）（`m2-impl-r2-fixes-b-implementer-report.md` 第三节第 2 行、`m2-impl-r2-fixes-a-implementer-report.md`「结论」第 1 行）。
2. **根槽序号 j 的设备内偏移 = 区域起点 + j × 固定结构槽距**（槽距 = max(4096, mkfs 时探测的 io_min)，D2（RAID 条带策略） 已定项 19）。按已定项 2 的字面取槽距 512 也合规，而字节表要新池新建文件的根落在 1 MiB + 4096，这一句把序号到偏移的换算钉死。
3. **P、chunk 是第一版常量（P = 3、chunk = 1 MiB），多于两块盘换 P 要另立格式决策**；读者按第 1 句判它们。mkfs 要校验 `基址 + (R−1) × P × chunk + S × 槽距 ≤ 设备可用字节`，越界拒绝 mkfs。第一版 mkfs 取 S = min(8, ⌊P × chunk ÷ 固定结构槽距⌋)，结果小于 5 拒绝 mkfs，所以默认值不取两条边；显式给 S 的建池入口照旧，读者照第 1 句判区间与 S × 槽距。取 3 是第一版两盘下满足「P 素数且 P > devs」的最小素数，chunk 取 1 MiB 与基址同档 ⇒ 三个区域起点落在 1 / 4 / 7 MiB，与 [layout/01-first-txn.md](../layout/01-first-txn.md) 的字节表一致。⚠️ **「P > devs」那条约束已经没有理由**（它唯一的理由是 `gcd(P, devs) = 1` 的双射，已随 D2（RAID 条带策略） 已定项 7 把归属改成 mkfs 逐区域写下的设备身份而作废），账在 C217（P 大于设备数这条约束的理由没了而约束还在）；取 3 不依赖那条约束。
4. **根环三个区域的设备归属第一版写死 0 / 1 / 0**（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域把设备身份写进系统配置的「根环逐区域设备身份」12 字节（D2（RAID 条带策略） 已定项 7）。⚠️ **它不是 mkfs 参数**：第一版 devs = 2、R = 3，按 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界（取满 min(R, devs) = 2 个不同值、没有一块盘背超过 ⌈3/2⌉ = 2 个区域），只有「两个区域在一块盘、一个在另一块」这一种形状，区域编号一定之后 0 / 1 / 0 是唯一的规范写法。把它做成 mkfs 参数，买到的是「哪块盘背两个」这一格自由度，而那一格在两盘同构下没有可观测差别，代价是多一个要在挂载时校验、要在加盘时重算的参数。
5. **暖机空发布 2 次、新池新建文件 txg 3 是格式常量**，不是运行时谓词（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、NEW_POOL_FILE_CREATION_TXG = 3，登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 八）。D16（发布语义） 已定项 8 要求「推到本实例的根覆盖两块盘为止」，而在 0 / 1 / 0 的归属下 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定，写成常量比写成运行时谓词少一条只有一个取值的分支。⚠️ 归属改了这两个常量要重算（加盘或开条带表时）。`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（mkfs → 取号 → 暖机两次 → 新池新建文件，同一个进程里那一串），不管任何池的第一个文件版本：只做过 mkfs 的池重开之后再写第一个文件版本，txg 取现行根 + 1。读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前；`physical_block_size` ∈ [根记录宽 466, 槽距]。越界即拒：checker 对越界槽报 I-7.13（系统配置池级字段在读者收的范围里）、该池其余不变量报不适用，实现整池拒挂载（用户 2026-09-27 定「整池拒」）。

**射程**：把区域公式换成「mkfs 逐区域写起点」那条留到 zoned 开线前再走三方。全池没有可用槽 0 时按第一版常量 4096 读槽 1 的那一读（用户 2026-09-14 定的「甲」）由用户 2026-10-02 推翻、改成逐档，原话在变更史；「世代号相同取偏移小」那一句在 mkfs 清零 [0, 根环基址)（已定项 8 第 3 条）之后只在介质损坏造出的残留上有对象。池级 checker、恢复与 GPU 判器的两份按同一句读（[layout/01-first-txn.md](../layout/01-first-txn.md) 一「系统配置槽 0 与槽 1 的位置」那一段）。

**依据**：

- E142（新池新建文件的干跑）：按这套取法写出新池新建文件的全部固定结构字节（系统配置槽 0 / 4096、根环 1 / 4 / 7 MiB、新池新建文件的根落区域 0 槽 1），与 `crates/` 实装按 21 个区域比对全等（根记录、journal 记录与系统配置逐字节全比）。
- E87（固定结构的放置）：两盘上 R = 3 必有一盘背两个区域——0 / 1 / 0 是唯一规范写法的那一半。
- 逐盘计那一读法：C322（取号那一步的屏障怎么放没有条款） 三轮三方之后的用户定案（全池最大 + 1 会让滞后盘一次跳 2、落回它择到的那一槽），判决在 `research/prompts/c322-r3-main-verification.md`。
- 用户定案（按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定；第 4 / 5 句是之后的弹窗定案），原话在变更史；除逐盘计那一句，都没有走三方论证，可推翻。
- 用户定案 2026-09-23：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（`research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节判它射程的那一格），原话在变更史。
- 用户定案 2026-10-01：「mkfs 两槽世代号都是 1」写进这一项，I-7.16（系统配置槽世代号与槽号的关系） 一并判上（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节第 1 条），原话在变更史。
- 全池没有可用槽 0 时逐档、平局取法与两个出口：三方判决 `research/prompts/m3-c651-r1-main-verification.md` 第二节（常量与整池拒两读出局、逐档读法站住、平局取法站住）与第四节第 1、2 条，`research/prompts/m3-formal-r6-main-verification.md` 第二节条款句两个出口那一格；用例 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`（恢复与池级 checker 在同一份镜像上判逐档、自述槽距不等于偏移不收、取世代号最大、同号取偏移小、越界整池拒、incompat 不认得择不到）。
- 逐档先判不收之值、再核偏移：代码轮判决 `research/prompts/m3-formal-code-r2-main-verification.md` 第二节「K3-a」一行（checker 改前先核偏移，丢掉的槽 I-8.1（环几何够大） 不判，与实现分叉；第 13 批改成与实现同序）与 `research/prompts/m3-formal-code-r3-main-verification.md` 第二节「K3」一行（辩方 Z5：条款缺这一句的字面）。
- 用户定案 2026-10-02：逐档连同前两步与平局取法写回；全池没有可用槽 0 时改成逐档，推翻 2026-09-14 收尾弹窗定的「甲」（按最小槽距 4096 读槽 1），原话在变更史。
- 根环区域数 R 读并整池拒、每区槽数 S 越界整池拒、incompat 不认得与格式版本不是 1 同在时格式版本先：用户定案 2026-10-20（C663（系统配置槽越界与不收之值两读者分类不同）），原话在变更史。
- 读者加 P、chunk 与 S × 固定结构槽距两判，第一版 mkfs 取值式与「多于两块盘换 P 要另立格式决策」：三轮三方判决 `research/prompts/m3-close-c506-r3-main-verification.md` 第二节 X1 与第三节「交用户」四件，用户定案 2026-10-22（弹窗「四件照写回并实现（推荐）」），原话在变更史。

**欠**：C217（P 大于设备数这条约束的理由没了而约束还在）。

```

## 三、实现今天的样子

行号都是这一次现查的。HEAD `841b3beab`；下表几份文件工作区另有别的会话暂存的改动（`git diff --cached --numstat`：装置 +24 / −7，`recovery.rs` +1610 / −659，`root_ring.rs` +137 / −10，`transaction.rs` +944 / −192），行号按工作区读。

| 东西 | 在哪 | 今天是什么样 |
|---|---|---|
| 落点公式 | `crates/singlefs-core/src/root_ring.rs:167–175` `target_for_publish`：`region: checkpoint_txg.0 % ROOT_RING_REGIONS`、`slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count()` | 同 D22 已定项 16 |
| C743 落点核 | `root_ring.rs:177–193` `root_sits_in_the_ring_slot_the_formula_gives`（txg 是 `GENESIS_CHECKPOINT_TXG` 的照 `genesis_root_ring_slots()` 另认，别的只认公式那一槽）；`crates/singlefs-core/src/recovery.rs:1732–1765` `read_root_ring_slot`：自证得过而不住在公式那一槽的，记 `BadRootRingSlotReading::NotSelfVerified`（错位读当这一槽这一次读坏） | 已实现。D22 已定项 7 那一行仍写「公式核还没实现，欠在 C743」（第二节），kb 与实现对不上，这一次不改 kb，写进交回 |
| 读根环次序与择根 | `recovery.rs:1699–1708` `every_root_ring_slot`（区域号升序、区域内槽号升序）；`:1772` 起 `choose_root`（读每个槽、摊成择新键、交给 `singlefs_replay_prefix::choose_root_slot`） | KL 里三个落点槽的读次序是 2、9、17 |
| 第 0 代根的槽 | `crates/singlefs-core/src/make_filesystem.rs:56` `GENESIS_CHECKPOINT_TXG = CheckpointTxg(0)`；`:825–827` `genesis_root_ring_slots`（三个区域各自的槽 0） | KL 不摆第 0 代根 |
| 装置：本地常量与落点 | `e165_replay_bounded_enumeration.rs`（下同）`:9637–9659`（区域数 3、每区 8 / 4 槽……）；`:9805–9810` `root_choice_target_of_publish`；`:9817–9821` `root_choice_slot_index_of_publish`；`:9844–9860` TH2 回比；`:11818` `cross_check_root_choice_local_constants_against_the_implementation` | 落点本地算法与回比已在，K 族的生成器在用 |
| 装置：r3 档 KA | `:11584–11603` `ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS`（四条根）与 `ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS = 3`；`:11605–11659` 生成 125 态（区域 0 槽 0、1、2 各取空或四条之一，见证串 `ka_region0_slots012=[…]`）；`:11738–11748` 本地择根序钉值 0、36、18、18、52；`:11751` `PINNED_ROOT_CHOICE_ADVERSARIAL_STATES = 125`；`:12719–12731` TV10（KA 要「同 txg 不同实例同在」与「实例代号更高而 txg 更低同在」都 > 0）；`:12950–12962` TV4 两条断言 | 旧定义。C743 之后 125 态里四条根全是错位读（调查报告 3.5 量的：`criterion=KJ0 violations=125 judged=125`），第七节的脚本在旧 KA 上加落点核同样给 125 态全空 |
| 装置：r4 档 KA | `:13046`（「KA 是 r3 原样的对抗根环」）；`:16479–16488` `PINNED_KA_KJ2_OF_THE_LOCAL_RULES`（六种：0、36、18、18、52、0）；`:16490–16534` `ka_kj2_violations_of_a_local_rule`（自己另摆一遍 125 态，不经 r3 那个函数）；`:17163–17190` `run_the_adversarial_domain_of_the_faults_family`（三步与行、判决用 r3 的写法） | 同上 |
| 装置：标签、命令行 | `:9672`、`:13053` 标签 `"KA"`；`:12825`、`:16875` 不认识的域的报错文字；`:17432` `USAGE`；`:12883`、`:17229` config 行（`only=`） | 换名要动这几处 |
| 装置：内联测试 | `:19199` `the_adversarial_root_rings_have_125_states_one_of_them_without_a_self_verified_root_and_zero_kj2_violations_today`（125、1、1、124、0、0、两条 `> 0`）；`:19268`、`:19642`「在 KA 上每种本地择根序一条」；`:19297–19303`、`:19518–19523` 命令行解析（`--only K9,KA`、`KF,KA`）；`:19431–19433` r4 本地钉值循环 | 旧定义的数 |
| 变异表 | `crates/mutations.tsv:1712`（判据的版本序写成实例代号为主，点名 `root_versions_compare_the_txg_first_and_the_instance_breaks_ties`）、`:1713`（KA 本地「平局取低」写成「平局取高」，名字里写「KA 本地那一份 36 变 0」，点名 `the_local_root_choice_rules_pick_as_the_registered_comparisons_on_hand_made_rings`）；crates 一侧的六个变体 `:1634–1639`（v1822–v1827 依次；前五行改 `crates/singlefs-replay-prefix/src/lib.rs` 的择根核、点名那个包的 `--lib` 测试，`:1635` 与 `:1639` 点名 harness 测试 `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot`——那份测试工作区暂存的改法用第 0 代根的槽摆两条 txg 0、实例不同的根，绕开 C743） | 1713 那一行的点名测试用手摆根环，不经 KA；名字里的数是旧 KA 的 |
| 驱动 | `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` 第 23–24、33–35 行（KA 钉值 18、52、36、18，推的 0、18）、第 335 行 `R3_ALL_DOMAINS`、第 378 行 `R4_ALL_DOMAINS`、第 1463–1506、1625–1669 行（读 KA 的 KJ2、核钉值） | 只在冻结根上跑（r3、r4、r5 那几份根上的旧装置与旧 crates），那几份根里仍是旧 KA，钉值对它们仍成立 |
| 复跑 | `scripts/experiment-apparatus/replay.sh:1679` `driver_e165_r3_segment2_root_choice`、`:1681` `driver_e165_r4_segment1_root_choice_faults`、`:1691` `driver_e165_r5_segment5_r2_gate_root_choice_faults`：三个都在主工作区上 `cargo run`（`:2123–2125`、`:2157–2159`、`:2240–2242`），跑的是今天的 crates 与装置；`:1680`、`:1682`、`:1684`、`:1685`、`:1692` 五个变异杀伤界驱动都要 `--root` 指冻结根 | 前三个的产物会变（KA → KL）；后五个在冻结根上跑，不受这一次影响 |

## 四、跑之前已经存在的数

读条款、实现、调查报告与判问法时已经知道或自己算出的，照实列；判据在第六节，不照这些数挪。三份比对对象只取了 sha256 与行数，内容没读；实验页的结果与结论各节没读。

| # | 数 | 出处 | 对判据的影响 |
|---|---|---|---|
| 4.1 | 调查员在副本上 `--bounds light --profile r3 --family root_choice --only KA`：今天的树 `criterion=KJ0 violations=125 judged=125`、`criterion=KJ2 violations=0 judged=0`；HEAD 的树 `KJ0 violations=1 judged=125`、`KJ2 violations=0 judged=124`；把 C743 的落点核短路之后回到 HEAD 的数 | 调查报告 3.5（报告转述，原件 `e165-{now,head}.out` 没读） | 旧 KA 在 C743 之后失去对象，这是这一次的起因；不进门槛 |
| 4.2 | 我算的：旧 KA 不核落点 125 态、全空 1、同 txg 不同实例 48、实例高而 txg 低 24，本地择根序 KJ2 依次 0、36、18、18、52、0（与 r3 第七节乙类 KA 的钉值逐个相同，这一格自证脚本的六种择根序写对了）；旧 KA 核落点：125 态全空 | 第十三节 `anchors_e165_r7_ka.py` | 旧定义加 C743 只剩 KJ0，与 4.1 相容 |
| 4.3 | 我算的 KL：27 态，根环全空 1，同 txg 不同实例 0，实例高而 txg 低 7，两个以上实例 12，读根环次序里第一个自证过的根不是最新 4；本地择根序 KJ2：今天的序 0、平局取低 0、实例代号为主 6、比较不看实例 0、取第一个有效者 4、平局取后者赢 0 | `anchors_e165_r7_ka.py`、`anchors_e165_r7_kl_counts.py` | 进第七节乙类 |
| 4.4 | 我算的另两种摆法：只取 txg 100、101（9 态）：全空 1、实例高而 txg 低 1，KJ2 依次 0、0、1、0、4、0；只取 txg 101、102（读根环次序里新根在前，9 态）：KJ2 依次 0、0、1、0、0、0 | `anchors_e165_r7_ka.py` | 「取第一个有效者」那一格随读根环次序翻面（4 / 0），是第八节几何敏感性的两点；KL 把两种次序放进同一个域 |
| 4.5 | r4 第四节推 s2（r4 第 498 行）：v1823、v1825 只在根环里有两条同 txg 不同实例的根时与原式择得不同，根槽落点只由 txg 定 | r4 登记 | 与 4.3 的 0、0 一致：KL 上这两条（与 v1827）读 0 是落点公式的结果，不是变体没改到择根 |
| 4.6 | 钉住红例表 `.claude/gate.d/e165-pinned-red-examples.tsv`：385 行 = 文件头注释 25 行 + 数据 360 行；数据行第 2 段（域）的分布 A 36、A6 42、AC 38、C 24、C6 30、E1 25、E2 4、L 24、L6 30、R 26、U_verify_off 26、U_verify_on 29、X 26，没有 K、K9、KW、KA；`--pinned-cases` 走 `--replay-state` 同一段代码（装置第 30–33 行），r3 修订 2 第 8 条写「逐态行、单状态重放与钉住红例重放这一族不支持」 | 第十三节命令 | L4 设计时够判：KA → KL 不碰这张表与它的重放。别的改动（工作区暂存的记录头 487 → 496 字节等，4.8）碰不碰它，不在这份登记里 |
| 4.7 | 三份比对对象：r3 择根 `20ca1851…`、379 行；r4 带故障择根 `10ff3260…`、3309 行；r5 第五段 R2 闸 `55993278…`、3359 行 | `sha256sum`、`wc -l` | L2 的比对对象（第十一节） |
| 4.8 | 工作区暂存的装置改动里有「487 字节头 → 496 字节头」与根记录新字段（`reclaim_threshold`、`pinned_ring_slot`）的接线（`git diff --cached` 装置那一份的 `+`/`−` 行） | `git diff --cached` | 这些改动与 C743、取号写两轮、C755 一样可能让 KL 之外的行变；L2 翻面时按 LS3 停下交回，不归因成 KL |

## 五、臂、阳性对照、真实基线

### 5.1 档

不加新档。r3 档的 `--family root_choice` 与 r4 档的 `--family root_choice_faults` 里 KA 那个域原地换成 KL（用户定：KA 改成照落点公式摆根）；两档别的族、别的域一个字不改。旧产物留着不删；三个复跑驱动的期望产物换成这一次的新产物（5.6 H9）。

### 5.2 臂（每一行的对面写成支持它的人认的样子）

| 行 | 被判的一方 | 对面 |
|---|---|---|
| L1 | 今天的三步在 KL 上：落到的版本 ≥ 根环里每条根（KJ2），根环全空那一态记 KJ0 | 「C743 之后对抗根环只能摆在落点上，KJ2 在它上面没有可判的东西」：支持它的人会说同 txg 平局没了，KL 只是合法摆法的一个子集。KL 里有实例高而 txg 低（7 态）与读在前的不是最新（4 态）两种形状，本地「实例代号为主」「取第一个有效者」两种序在 KL 上读 6、4（第七节）——这两格不是 0，就是 KL 上 KJ2 还有对象的证据 |
| L1（域的取法） | KL：三个 txg、27 态 | 只取原来的两个 txg（100、101，9 态，第七节乙类那一行）：支持它的人会说它与旧 KA 的根最接近、域更小；它只给一种读根环次序（4.4） |
| L2 | KL 之外的行与上一次逐字节相同 | 「C743、取号写两轮、C755 与工作区暂存的格式改动也会动合法历史一族」：这正是 L2 要量的，不翻就记「只有 KL 行变」 |
| L3 | 变体在 KL 上的读数等于第七节 | 「v1823、v1825、v1827 在 KL 上读 0，等于没有对照」：r3 TF8、r4 TF7 正是这样读 0 的（「变体没改到择根」）；这一次照 4.5 把它们在 KL 上的 0 钉成「这一族没有取样点」，臂的阳性对照改由 crates 变异表那一行点名的测试担（待主 agent 认） |
| L4 | 钉表重放不受影响 | 「表里有择根族的行」：4.6 数过，没有 |

### 5.3 阳性对照（每条判据都跑，每条臂都跑）

| 判据 / 臂 | 对照 | 必须看到 | 看不到时 |
|---|---|---|---|
| KJ0 | 同 r3 5.7 第一行（K 域 M = 1、U = 0 的指定一态 panic） | 那一格违例数 = 1 | 同 r3 TV3 |
| KJ2（KL） | 本地照 r3 9.1 的比较式写的五种择根序（r3 档）与六种（r4 档，多「平局取后者赢」），落点 = 择到的根 | 实例代号为主 6、取第一个有效者 4（这两格是会红的对照，非 0）；今天的序、平局取低、比较不看实例、平局取后者赢都是 0（这四格是等式，不是对照：KL 上没有同 txg 两条根） | 同 r3 TV3；「实例代号为主」或「取第一个有效者」读 0 说明 KL 没摆成第七节的样子（走 LV1） |
| KL 的形状 | 计数：实例高而 txg 低同在的态数、同 txg 不同实例同在的态数 | 7 与 0 | 走 LV1（TV10 的新写法，5.6 H3） |
| 变体臂 v1824、v1826（第二段） | 各自在 KL 上跑 | KJ2 读 4、6 | 走 LF4 |
| 变体臂 v1822、v1823、v1825、v1827（第二段） | KL 上读 0 是等式不是对照；对照改由 `crates/mutations.tsv:1634`、`:1635`、`:1637`、`:1639`（v1822、v1823、v1825、v1827 依次）点名的 crates 测试在变体上红担（提交时门禁 checker-tier-crates-mutation-replay 的那几行「抓到」） | KL 读 0；crates 那几行抓到 | KL 非 0：走 LF4；crates 那一行没红：那条变体臂没有对照，L3 记「没有对照」 |

### 5.4 真实基线

今天的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`，一个字不改），同 r3 5.2 那一行。

### 5.5 域（替换 r3 5.4 第 461 行 KA 那一行与 r4 5.4 第 559 行 KA 那一行；别的域同 r3、r4）

| 域 | 定义 | 状态数 | 跑不跑 |
|---|---|---|---|
| KL | 1.3：txg 100、101、102 各自的落点槽（区域 1 槽 1、区域 2 槽 1、区域 0 槽 2）各取空或那个 txg 的实例 1、2 两条根之一；其余根槽全 0，journal 环全 0，S = 8、journal 环 64 槽；判 KJ0、KJ2 | 27 | r3 档 `--family root_choice` 与 r4 档 `--family root_choice_faults`（含 `--gate r2`）各全跑 |

r3 档这一族状态数由 2 931 574 + 5 472 + 80 + 125 = 2 937 251 变成 2 937 153（KL 比 KA 少 98）；r4 档合计 13 856 755 变成 13 856 657（r4 第 561 行的合计减 98）。

### 5.6 装置要改哪几处（要动的文件：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/mutations.tsv`、`scripts/experiment-apparatus/replay.sh`）

行号是 2026-10-23 工作区的（第三节）。**表里的新值都是执行员要量出来、再与第七节对的数**：先照下表改，再跑 5.8 的三份产物，KL 那几行等于第七节才算钉住；对不上的照第十一节 LV1 / LS1 判，不把量到的数改进常量来让它对上。

| # | 改哪里 | 改成什么 | 验收 |
|---|---|---|---|
| H1 | `:11584–11659`（r3 档 KA 的根与生成） | 六条根 (txg, 实例) ∈ {100, 101, 102} × {1, 2}，新根段各不相同；每个 txg 的候选只摆在 `root_choice_slot_index_of_publish(AdversarialRootRing, txg)` 那一格，摆之前过 TH2 回比（`:9844–9860`）；三个落点槽按读根环次序编号，状态号 = 三位三进制（空、实例 1、实例 2），27 态；见证串改成 `kl_landing_slots=[<txg 102 槽>,<txg 100 槽>,<txg 101 槽>]`（各写 `empty` 或 `(txg,实例)`） | `name=root_choice_states domain=KL … states=27 …` |
| H2 | `:16490–16534`（r4 档自己另摆的那一遍） | 删掉另摆的那一遍，改用 H1 那一个生成函数交出的 27 个根环（收严：两档的 KL 出自同一段代码，不会一边改了一边没改） | r4 档 KL 的本地择根序读数与 r3 档逐个相同 |
| H3 | `:12719–12731`（TV10）与它的消息 | KL：「实例代号更高而 txg 更低同在」> 0，且「同 txg 不同实例同在」= 0（后一半是收严：KL 上同 txg 两条根同在说明摆根没照落点） | 跑完没有 `TV10 KL` 那一条 void |
| H4 | `:11738–11751`、`:12950–12962`、`:17176–17186`（钉值与 TV4 断言） | 本地择根序 r3 档 0、0、6、0、4（次序同 `LocalRootChoiceRule::ALL`）；r4 档 `PINNED_KA_KJ2_OF_THE_LOCAL_RULES`（改名 `PINNED_KL_KJ2_OF_THE_LOCAL_RULES`）0、0、6、0、4、0；状态数 27；根环全空 1；另加（收严）同 txg 不同实例 0、实例高而 txg 低 7、两个以上实例 12、读在最前的不是最新 4 四条断言 | TV4 那几条不 panic |
| H5 | 标签与命令行：`:9672`、`:13053`、`:12825`、`:16875`、`:17432`、文件头 `:13`、`:9667`、`:13046` 的文档 | `"KA"` → `"KL"`；`--only` 认 KL、不认 KA（认 KA 的报「不认识的域」退 2）；文档写 KL 的定义与这份登记的路径 | `--only KL` 跑得出，`--only KA` 退 2 |
| H6 | 内联测试 `:19199` 那条 | 改名 `the_landing_slot_adversarial_root_rings_have_27_states_one_of_them_without_a_self_verified_root_and_zero_kj2_violations_today`；断言 27、`ChoseNothingStates` 1、`Kj0Violations` 1、`Kj2Judged` 26、`Kj2Violations` 0、`SlotReadingsDifferingFromTheTruth` 0、`SameTxgDifferentInstanceRoots` == 0、`HigherInstanceLowerTxgRoots` == 7 | C-test 里它 ok |
| H7 | 内联测试 `:19268`、`:19297–19303`、`:19431–19433`、`:19518–19523`、`:19642` | 「KA」→「KL」，命令行用例 `--only K9,KL`、`KF,KL`；读数条数不变（五种 / 六种本地择根序各一条） | C-test 里它们 ok |
| H8 | `crates/mutations.tsv` 这只装置的行 | `:1713` 名字里「KA 本地那一份 36 变 0」照实改成「手摆根环上那一条变红；KL 上平局取低与平局取高都读 0」（点名的测试不变；别的表若按名字引这一行，先查、同改）；补三行（第九节 KM1–KM3）；每条原文在改后的源码里 `count == 1` | 用 python 数，不跑变异 |
| H9 | `scripts/experiment-apparatus/replay.sh:1679`、`:1681`、`:1691` 与三个驱动上方的注释 | 期望产物换成 5.8 的 P1、P2、P3 新产物；注释补「第 7 次重跑（KA → KL）」与登记路径；驱动命令不改 | 第十一节比对对象表 |

H1–H8 编得过：`cargo build --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`、同参数的 `cargo clippy … -- -D warnings`、`cargo fmt --check`（经 `scripts/hosts/compile-then-swap.py` 换进主工作区时它起的那几条算）。

### 5.7 分段（按岔路排；每段交回之后主 agent 对着 1.2 的四行判续不续，一行开着的都不剩就停）

| 段 | 谁做 | 做什么 | 补哪一行 | 补完够不够判 |
|---|---|---|---|---|
| 第一段 | 执行员 | 5.6 的 H1–H8；B1 编一次；P1、P2、P3 三份产物（执行员自己经内存包装跑本体，1.5）；写比对脚本、先过 LQ8 的自证；照第六节逐格判；落产物、H9、第十一节比对对象表与 `.claude/gate.d/replay-comparison-units.tsv` 的「不登记」三行 | L1、L2 | 两行够判 |
| 第二段 | 主 agent | 5.8 的 T1（这只装置的内联测试，重型） | L1 的内联那一半（H6、H7） | 够判 |
| 第三段（附带，主 agent 判续不续） | 执行员 | 驱动加一套 `--pins r7`：根取第一段开工那一刻主工作区 `Cargo.toml`、`Cargo.lock`、`.cargo/`、`crates/` 的拷贝（清单与 sha256 记进产物头），r3、r4 两个模式的 KA 钉值换成第七节 KL 那一列，`R3_ALL_DOMAINS`、`R4_ALL_DOMAINS` 在 `--pins r7` 下取 KL；冻结根上的 r3、r4、r5 三套钉值一个字不改；跑 r3、r4 两个模式（六个变体各一次） | L3 | 够判 |
| 第四段 | 主 agent 提交时 | `checker-tier-crates-mutation-replay` 里这只装置的行（含 KM1–KM3）与 `crates/mutations.tsv:1634–1639` 六行 | L1 的变异那一半；L3 的「crates 那一行抓到」 | 够判 |

L4 在设计时够判（第四节 4.6），不派段。第一段停在 LS1–LS3 的，后面的段不跑，交回主 agent。

### 5.8 命令

工作目录一律 `/home/fy5090/code/singlefs`；`<草稿目录>` 由主 agent 定；产物名里的日期是登记日 2026-10-23，哪一天跑都照这个名字落，同名文件已在就停下交回。三份产物的计数与线程数无关（`replay.sh` 第 2153 行注释：r4 那一族 6 与 3 线程整份逐字节相同；config 行不记线程，装置第 12883、17229 行），这一次取登记值 31。

| # | 谁跑 | 命令 | 内存上限与理由 | 限时 | 跑完看哪几行 |
|---|---|---|---|---|---|
| B1 | 执行员 | `CARGO_TARGET_DIR=<草稿目录>/target nice -n 19 bash scripts/hosts/capped.sh 31 cargo build --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`（编译目录在仓外，交回前删） | 不跑编出来的代码 | — | 退出码 0 |
| P1 | 执行员 | `{ set -o pipefail; E165_THREADS=31 RUN_WITH_MEMORY_CAP_TIME_LIMIT=3600 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G <草稿目录>/target/release/e165_replay_bounded_enumeration --bounds segment1 --family root_choice --profile r3 \| grep '^E7RESULT ' > research/results/e165-replay-bounded-enumeration-2026-10-23-r7-r3-root-choice.out; echo "exit=${PIPESTATUS[0]}"; } 2> <草稿目录>/p1.err` | 8G：复跑默认上限；r3 第二段峰值 446 MiB（`replay.sh` 注释） | 3600 秒 | `exit=`；`name=root_choice_states domain=KL` 一行；`name=violations domain=KL` 两行（KJ0、KJ2）；`name=control` 里 `domain=KL` 五行；`name=verdict domain=KL` 一行；收尾行 |
| P2 | 执行员 | 同 P1，参数 `--bounds segment1 --family root_choice_faults --profile r4`，产物 `e165-replay-bounded-enumeration-2026-10-23-r7-r4-root-choice-faults.out`，`p2.err` | 8G：r4 那一族整遍峰值约 1.7 GB（`replay.sh` 注释） | 3600 秒 | 同上，`name=control` 里 KL 六行 |
| P3 | 执行员 | 同 P2，另加 `--gate r2`，产物 `e165-replay-bounded-enumeration-2026-10-23-r7-r5-gate-r2-root-choice-faults.out`，`p3.err` | 8G：同 P2（`replay.sh` 第 2240 行那一跑取默认上限） | 3600 秒 | 同 P2 |
| P1b（附带，够判后不跑） | 执行员 | 同 P1，`E165_THREADS=3`，产物 `…-r7-r3-root-choice-threads-second.out` | 8G | 3600 秒 | 与 P1 逐字节相同 |
| T1 | 主 agent | `{ SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker-tier 的装置 e165_replay_bounded_enumeration 的内联测试; 范围=1 个测试目标; 为什么=E165 第 7 次重跑登记 L1：KA 换成照落点公式摆根的 KL 之后这只装置的单测全绿' RUN_WITH_MEMORY_CAP_TIME_LIMIT=3600 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/run-build-directory.sh cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration; echo "exit=$?"; } > <草稿目录>/e165-r7-t1-inline-tests.log 2>&1` | 8G：`memory-peaks.tsv` 里这只装置整份单测 debug 档峰值 1 257 242 624 字节、release 档 `--no-run` 编译 2 976 907 264 字节 | 3600 秒（含编译） | `exit=`；`test result:`（failed 0、ignored 0）；有 `panicked at` 的连同后面三行 |

表里命令格中的 `\|` 是表格转义，跑时写成 `|`。撞了内存或限时照 LS2 停，不加大上限重跑。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，合取留给人。门槛都不能从 5.2 的臂定义直接推出：KL 的计数与本地择根序读数由第七节的独立算术给，L2 的「相同」由两份产物给，L3 由变体给，L4 由表给。

| 量 | 怎么算 | 门槛 | 判定 | 对应的行与翻面取值 |
|---|---|---|---|---|
| LQ1 KL 的形状 | P1、P2、P3 各自 `name=root_choice_states domain=KL` 那一行的状态数、根环全空、同 txg 不同实例、实例高而 txg 低、两个以上实例、读在最前的不是最新 | 第七节乙类：27、1、0、7、12、4 | 逐格等 / 不等（不等走 LV1） | L1 的前提 |
| LQ2 今天的三步在 KL 上 | 同三份产物 `name=violations domain=KL` 的 KJ0、KJ2 两行 | 钉绝对值：KJ0 violations = 1、judged = 27；KJ2 violations = 0、judged = 26 | 等记「L1 不翻」；KJ2 非 0 走 LF1；KJ0 不是 1 走 LF2 | L1：任一格不等即翻 |
| LQ3 本地择根序 | 同三份产物 `name=control` 里 `domain=KL` 的几行 | r3 档（P1）0、0、6、0、4；r4 档（P2、P3）0、0、6、0、4、0 | 逐格等 / 不等（不等走 LV1） | L1：KL 上 KJ2 还有没有对象——「实例代号为主」「取第一个有效者」非 0 就是有 |
| LQ4 三份产物的 KL 互比 | P1、P2、P3 的 KL 计数行与 KJ0、KJ2 两行，除 `name=` 之外的字段 | 三份相同（旁边钉 LQ1、LQ2 的绝对值） | 相同 / 哪几个字段不同 | L1 的「两处 KL 一样吗」 |
| LQ5 KL 之外不同的行数 | 每份产物与它的比对对象（第十一节）照 1.3 的比法：KL 行不比；config 行只许 KA → KL 标签；汇总行逐字段、差只许等于 KL 减 KA；其余逐字节 | 钉绝对值：三份各 = 0 | = 0 记「只有 KL 行变」；≥ 1 逐行列（两边原文），走 LS3 | L2：任一份 ≥ 1 即翻 |
| LQ6 汇总行的差 | LQ5 里汇总行每个不同字段的差 | 每个差都等于第七节乙类 LQ6 那一列（例：状态数 −98、KJ2 被判态数 −98、KJ0 违例数 0） | 等 / 不等（不等的字段算进 LQ5） | L2 |
| LQ7 变体在 KL 上（第三段，附带） | 驱动 `--pins r7` 的 r3、r4 两个模式，每个变体一行的 `ka_kj2`（`--pins r7` 下读 KL） | 第七节乙类：v1822 0、v1823 0、v1824 4、v1825 0、v1826 6、v1827 0；两个模式相同 | 等 / 不等（不等走 LF4） | L3 |
| LQ8 比对脚本会红 | 执行员写的比对脚本先跑三格：比对对象对它自己（0 行不同）；把比对对象的副本里一行非 KL 行改一个数（恰好 1 行，指到那一行）；把一行汇总行的状态数差改成 −97（恰好 1 行） | 三格都如此 | 不如此：比对脚本作废，L2 不判（LV3） | L2 的前提 |
| LQ9 钉表里择根族的行数 | `.claude/gate.d/e165-pinned-red-examples.tsv` 数据行第 2 段是 K、K9、KW、KA、KL、KF、KF3、KF9、KFW、KG 之一的行数（设计时已数，第四节 4.6） | = 0 | = 0 记「不受影响」 | L4：≥ 1 即翻 |

## 七、钉绝对值的断言

### 7.1 甲类：出自被测条款本身的（不符走 LF3「条款可能错」：两边都查）

| 编号 | 断言 | 出自 |
|---|---|---|
| A1 | KL 上「同 txg 不同实例同在」= 0 | D22 已定项 7「一个 txg 只有一个落点槽」（第二节）：KL 的根都照落点摆，读出来同在的只能是同一个落点槽里的一条 |
| A2 | KL 上根环全空的那 1 态 KJ0 记一条，其余 26 态 `choose_root` 交回一条根 | D13 已定项 1 M1 与 I-7.1（同 r3 5.5 KJ0）；KL 的根都住在落点上，C743 的落点核不挡任何一条（D22 已定项 7、已定项 16） |
| A3 | KL 上 KJ2 违例 = 0 | D22 已定项 7 择新「(checkpoint_txg, 实例代号) 最大」与已定项 24「先逐个验证全部候选、再在有效者中择新」 |

### 7.2 乙类：独立算出、用命令核过的（不符才作废，LV1）

命令是第十三节的 `anchors_e165_r7_ka.py` 与 `anchors_e165_r7_kl_counts.py`：只用条款字面（落点公式、读根环次序、择新序）与 r3 9.1 的比较式，不读装置、不调 crates。前者在旧 KA 上不核落点时给回 r3 第七节乙类 KA 的 125、1、36、18、18、52、0（自证六种择根序写对了）。

| 编号 | 断言 | 值 |
|---|---|---|
| B1 | KL 的形状 | 27 态；根环全空 1；同 txg 不同实例 0；实例高而 txg 低 7；两个以上实例 12；读在最前的不是最新 4 |
| B2 | KL 上本地择根序的 KJ2 违例数 | 今天的序 0、平局取低 0、实例代号为主 6、比较不看实例 0、取第一个有效者 4、平局取后者赢 0 |
| B3 | 变体在 KL 上的 KJ2（第三段；变体与本地择根序的对应同 r3 9.1、r4 9.1） | v1822 0（平局取后者赢）、v1823 0（比较不看实例）、v1824 4（取第一个有效者）、v1825 0（平局取低）、v1826 6（实例代号为主）、v1827 0（胶水摊键不带实例代号，同比较不看实例） |
| B4 | LQ6 的差（KL 减 KA） | 状态数 27 − 125 = −98；KJ2 被判态数 26 − 124 = −98；KJ0 违例 1 − 1 = 0；根环全空 1 − 1 = 0；同 txg 不同实例 0 − 48 = −48；实例高而 txg 低 7 − 24 = −17（旧 KA 的 48、24 是 4.2 那一格算的；汇总行里没有这两样的就不用这两格） |
| B5 | 只取 txg 100、101 的那种 KL（9 态，主 agent 改选时用） | 全空 1；同 txg 不同实例 0；实例高而 txg 低 1；KJ2 依次 0、0、1、0、4、0 |
| B6 | KM3 的取样点 | KL 上「取最后一个自证过的」读 16 |

### 7.3 互比旁边的绝对值

LQ4（三份产物的 KL 互比）旁边钉 LQ1、LQ2、LQ3 的绝对值；LQ5（新旧产物互比）旁边钉 LQ6 的每一个差；LQ7 两个模式的互比旁边钉 B3。三份产物一起错（例如三份都出自同一个摆错了根的生成函数），互比照样相同，B1 会红。

## 八、轨迹与几何敏感性

### 8.1 轨迹

被谓词消费的量（LQ1–LQ9）都是有限确定的枚举数完之后的个数或逐行比的行数，没有「轮」，峰值、为正轮数、期末值三样不适用。随运行变化的只有内存：P1–P3、T1 报 `memory-peaks.tsv` 那一行的峰值。合法历史域上「所选根之上的记录条数」的轨迹量同 r3 第八节，这一次不判（KL 没有 journal 记录）。正文与交回不写「从来」「恒」。

### 8.2 几何敏感性

| 旋钮 | 取样点 | 方向 | 重判哪一格 | 记法 |
|---|---|---|---|---|
| 读根环次序里新旧根的先后 | txg 100、101（旧根在前）与 txg 101、102（新根在前）；KL 把两种都放进同一个域 | 方向相反 | 「取第一个有效者」的 KJ2：4 与 0（`anchors_e165_r7_ka.py` 的 `new_ka_two_txgs`、`opposite_order_two_txgs`，设计时已算） | 翻面：记两点，不合成一个结论；KL 的 4 全来自「txg 102 那一槽空、100 与 101 都在」那 4 态 |
| 域里的 txg 个数 | 两个（9 态）与三个（27 态） | 少一个区域 / 三个区域都有根 | LQ2 的 KJ2 = 0、LQ3 的五格 | 两点都由脚本算（B2、B5）：今天的序都是 0，「实例代号为主」1 与 6，「取第一个有效者」4 与 4 |
| 线程数（附带，够判后不跑） | `E165_THREADS=31` 与 3 | 并发有无 | P1 与 P1b 逐字节相同 | 不同走 LV4 |

## 九、变异

### 9.1 第三段的变体（L3；名单、取法同 r4 9.1，钉值换成 KL 的）

| 变体（r3 的编号当名字；今天 `crates/mutations.tsv` 的行） | 在哪个取样点上改变输出 |
|---|---|
| v1822 平局取后者赢（`:1634`） | KL：0（键完全相同的两个槽只有第 0 代根，KL 不摆）；crates 那一行点名的测试在 `singlefs-replay-prefix` 的手摆键上红 |
| v1823 比较不看实例代号（`:1635`） | KL：0（同 txg 两条根不同在）；crates 那一行点名的 harness 测试用第 0 代根的槽摆两条 txg 0、实例不同的根，在那里红 |
| v1824 取第一个有效者（`:1636`） | KL：txg 102 那一槽空、txg 100 与 101 都在的 4 态，KJ2 红 |
| v1825 平局取低（`:1637`） | KL：0；crates 那一行点名的测试在手摆键上红 |
| v1826 实例代号为主（`:1638`） | KL：实例代号更高而 txg 更低、且实例为主的序择到旧根的 6 态，KJ2 红 |
| v1827 胶水摊键不带实例代号（`:1639`） | KL：0；同 v1823 那条 harness 测试 |

### 9.2 装置这一次新写的代码要过的变异（写进 `crates/mutations.tsv`，点名这只装置的单测；执行员可以加，不许减）

| 编号 | 改什么 | 在哪个取样点上改变输出 | 哪条测试红 |
|---|---|---|---|
| KM1 | KL 的摆根改回读根环次序的前三个槽（区域 0 槽 0、1、2），不按落点 | 27 态全是错位读：`ChoseNothingStates` 1 → 27 | H6 那条 |
| KM2 | KL 的 txg 只取 100、101 | 状态数 27 → 9（B5） | H6 那条 |
| KM3 | 本地「取第一个有效者」写成「取最后一个自证过的」 | KL 上那一格 4 → 16（B6） | `every_root_choice_criterion_has_a_positive_control_and_every_control_is_red`（KL 的对照读数等于钉值那一格）；手摆根环那条也会红，照报 |

`:1713` 那一行（KA 本地「平局取低」写成「平局取高」）在 KL 上 0 → 0，KL 不是它的取样点（`.claude/rules/mutation-sampling.md` 第三类）；它点名的手摆根环测试仍是敏感的取样点，这一行不删、不当等价变异，只改名字里的旧数（H8）。

## 十、失败条款

每条是「量出什么就让 1.2 的某一行翻面」，都是正当结果，照实记，不作废；反过来的结果（不翻）同样接受。

| 编号 | 条款 | 什么观测会让它触发 |
|---|---|---|
| LF1 | 今天的择根在照落点摆的对抗根环上没择最大的键（L1 翻）：记见证（`kl_landing_slots=[…]`、所选根、真相里最大的根），先照第十一节两边都查，核过就是 `choose_root` 或胶水的事，交主 agent | 任一份产物 `name=violations domain=KL … criterion=KJ2 violations=` 非 0 |
| LF2 | KJ0 在 KL 上不是恰好根环全空那 1 态（L1 翻）：多出来的态是 C743 把落点上的根也当错位读挡掉了，或摆根没照落点；照 LS1 两边都查 | `criterion=KJ0 violations=` 不是 1 |
| LF3 | 条款可能错：甲类 A2、A3 有一条不成立，而摆根与本地择根序都与乙类对得上（装置摆得对，实现与条款不一致）。条款与实现两边都查（`.claude/rules/implementation-first.md` 第 4 条），不默认哪边对，交主 agent | LF1 或 LF2 触发，同时 LQ1、LQ3 与第七节乙类逐格相等 |
| LF4 | 变体臂的读数不等于 B3（L3 翻）：读 0 而钉值非 0（v1824、v1826）：变体没改到择根，按 `.claude/rules/mutation-sampling.md` 第八类查替换文与选行；非 0 而钉值是 0（v1822、v1823、v1825、v1827）：KL 上有了同 txg 两条根或键相同的两条根，回查摆根；交主 agent，别的变体照判 | 第三段产物某个变体 `ka_pin=mismatch` |
| LF5 | KL 之外的行变了（L2 翻）：不归因，走 LS3 | LQ5 ≥ 1 |

## 十一、作废条款与停机条款

### 11.1 作废

| 编号 | 条件 | 作废什么 |
|---|---|---|
| LV1 | 装置数出的 KL 形状（LQ1）或本地择根序读数（LQ3）与第七节乙类有一格不等（装置的 TV4 断言红，或 H3 的 TV10 void 出现） | 那一份产物的 L1 判定；先查是装置摆错还是脚本算错，改脚本是新的一次登记修订（第十二节，时点在产物之前；产物之后发现的这一次作废、重跑） |
| LV2 | 产物行数不等于收尾行报的行数，或包装退 251、252、254 | 那一份产物 |
| LV3 | LQ8 三格有一格不如此 | L2 的比对 |
| LV4 | P1b 跑了而与 P1 不逐字节相同 | P1 与 L1、L2 里用到 P1 的格 |
| LV5 | 比对对象（11.3）不在、sha256 不等于表里那一格、或行数不等 | 那一行的 L2 不判（照共用约束「找不到历史实验的数据」去 git 里取，取到再判） |

### 11.2 停机（装置与 `crates/` 对不上：两边都查，既不作废也不当结果，写清哪一处对不上，交回主 agent）

| 编号 | 核什么 | 对不上时 |
|---|---|---|
| LS1 | TH2（本地落点与 `root_ring::target_for_publish` 回比，H1 摆 KL 的每条根都过）、TH4（读槽对拍：KL 上每个槽 `read_root_ring_slot` 的结局等于真相）、`cross_check_root_choice_local_constants_against_the_implementation` | 停 |
| LS2 | 包装退 250（撞内存上限）或 253（超时） | 停。输出不算结果；不加大上限、不放长限时重跑；交回里写命令原样、退出码、`memory-peaks.tsv` 那一行、产物最后一行 |
| LS3 | L2 翻（LQ5 ≥ 1） | 停在第一段，不改 `replay.sh`（H9 不做），新产物照落 `research/results/`；交回逐行列两边原文与可能的来处（C743、取号写两轮、C755、工作区暂存的记录头与根记录字段改动，第四节 4.8），不归因，由主 agent 定是另开一次重跑还是并进这一次（并进就是新的一次登记） |

### 11.3 比对对象

L2 的「上一次」：今天 `replay.sh` 三行指的那三份。sha256 与行数是 2026-10-23 现算的（`sha256sum`、`wc -l`），内容没读。三条正式跑在装置的单元表（`formal_run_units`，测试 `every_criterion_has_a_positive_control_and_every_control_is_red` 里那张）里都没有单元（C879 那一形），`.claude/gate.d/replay-comparison-units.tsv` 要由执行员加三行「# 不登记的比对对象：research/prompts/e165-r7-prereg.md #<号>：<理由>」，加之前门禁 doc-experiments 的 replay-comparison-units 格判红。

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out` | `20ca1851dea7cc414fa2071f3b7bccc1a29cfe3799dc59a2ef3d036bd07423de` | 379 | `1679` `driver_e165_r3_segment2_root_choice`（`--bounds segment1 --family root_choice --profile r3`） | `e165-replay-bounded-enumeration-2026-10-23-r7-r3-root-choice.out` | r3 档择根；单元表里没有这条正式跑的单元 |
| 2 | `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` | `10ff3260ec21d23b22d2cf74d8a9cba3b2e03519ee9643bd1bae7b9890a9f4d7` | 3309 | `1681` `driver_e165_r4_segment1_root_choice_faults`（`--bounds segment1 --family root_choice_faults --profile r4`） | `e165-replay-bounded-enumeration-2026-10-23-r7-r4-root-choice-faults.out` | r4 档带故障择根（闸取甲）；单元表里没有 |
| 3 | `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out` | `559932786d077529618b1cfc41802bc1653c972b5bf6f85914064e3eff616581` | 3359 | `1691` `driver_e165_r5_segment5_r2_gate_root_choice_faults`（另加 `--gate r2`） | `e165-replay-bounded-enumeration-2026-10-23-r7-r5-gate-r2-root-choice-faults.out` | r5 修订 1 的 R2 闸那一族；单元表里没有 |

不在表里的：五个变异杀伤界驱动（`replay.sh:1680`、`:1682`、`:1684`、`:1685`、`:1692`）在冻结根上跑旧装置，KA 仍是旧定义，这一次不重跑、钉值不改；第三段若跑，它的产物是新的一次（`--pins r7`），不比旧产物。

### 11.4 够判停机

每段交回附 1.2 那张表，L1–L4 逐行写「已够判 / 还差什么 / 剩下的量能不能让它翻面」。L1、L2 在第一段之后够判，L4 设计时够判；L3 只在主 agent 要它时跑（第三段），不跑就标「够判后未跑」——这是合法结局。P1b、第八节附带格同样标「够判后未跑」。

## 十二、修订

（留空：装置写之后、产物之前由执行员写，只许收严或补臂；原判据原样保留，每条写改了什么、依据、时点。）

## 十三、读过的文件与跑过的命令

### 13.1 读过的文件（行号区间；grep 命中行也列）

| 文件 | 读了哪些 |
|---|---|
| `.claude/agent-common.md`；`.claude/singlefs-ai-sop/rules/test-discipline.md`、`.claude/rules/implementation-first.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`.claude/rules/test-script-discipline.md` | 全份 |
| 调查报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-checker-tier-pins/result.md` | 1–142、206–353 |
| `research/prompts/e165-r3-prereg.md` | 标题 grep；9–38、454–498；`KA\|KJ2\|KJ0` grep 命中行（30、33、34、394、403、411、414、425、433、434、461、463、471、473、490–497、506、517、518、527–531、539、545、602、604、635、649、661、682–687、705–708、718、726、727、738–745、759、760、766、807、811–817、823–835、874、927） |
| `research/prompts/e165-r4-prereg.md` | 标题 grep；`KA` grep 命中行（5、31、40、41、481、484、490、498、502、559、561、570–573、591、594、614、615、632、633、643、843、856、902–909、944、967、969、992、1040、1127） |
| `research/prompts/e165-r5-prereg.md` | 标题 grep；13、655–700（比对对象表） |
| `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` | `KA` grep 命中行；9629–9700、9795–9835、10660–10760、11580–11665、11735–11760、11985–11995、12195–12205、12512–12520、12649–12760 的 `format!` 行、12715–12732、12883、12945–12965、16376–16385、16470–16560、16914–16940 的 `name=` 行、17160–17200、17229、17432、17700–17712、18367–18732 的单元名、19195–19240；30–33；`fn` 名 grep |
| `crates/singlefs-core/src/root_ring.rs` | 100–200 |
| `crates/singlefs-core/src/recovery.rs` | 1695–1718、1720–1800 |
| `crates/singlefs-core/src/make_filesystem.rs` | 820–835；grep 56 |
| `crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs` | `C743\|GENESIS\|genesis\|checkpoint_txg` grep 命中行（2、5、12、60、67、69、78、79、87、91、95、98、115） |
| `crates/mutations.tsv` | 1634–1639、1712、1713；`E165\|e165` 与择根变体名 grep |
| `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` | `KA\|ka_` grep 命中行；380–420 |
| `scripts/experiment-apparatus/replay.sh` | `E165` grep；2116–2128、2141–2260 |
| `.claude/gate.d/replay-comparison-units.tsv` | 全份 |
| `.claude/gate.d/doc-experiments.sh` | `preregistration-parallelism\|prereg` grep |
| `.claude/gate.d/e165-pinned-red-examples.tsv` | 文件头 1–20；数据行第 2 段的计数（命令见下） |
| `.claude/kb/decisions/22-单元原子性怎么合成.md` | 标题 grep；141（C743 那一句）；quote-kb 抄的 18、28、132–176、345–372 |
| `.claude/hooks/lib_heavy_tests.py` | 70–82 |
| `~/.local/share/singlefs/memory-peaks.tsv` | 1–3；`root_choice` 与 `--bin e165_replay_bounded_enumeration` 的命中行 |
| `research/results/` | 只对三份比对对象跑 `sha256sum`、`wc -l`，没读内容 |

没读：`research/results/` 下任何内容、`.claude/kb/experiments/` 下 E165 的实验页、`.claude/kb/experiments.md`、`.claude/kb/experiments-history.md`、`records/`、`.claude/kb/decisions-history.md`、`scripts/experiment-apparatus/perf-by-milestone.md`、第 6 次重跑登记（已归档）、调查报告点名的 `e165-*.out` 原件。

### 13.2 跑过的命令（原样）与算术的原样输出

```bash
git status --short -- <几份 crates 文件与驱动>; git diff --cached --numstat -- <同>; git log --all --diff-filter=D --format='%h' -- research/prompts/e165-r6-prereg.md   # 04ceaa904
(cd research/results && for f in <三份比对对象>; do sha256sum "$f"; wc -l < "$f"; done)
T=.claude/gate.d/e165-pinned-red-examples.tsv; wc -l $T; grep -c '\bKA\b' $T; grep -vc '^#' $T; grep -c '^#' $T   # 385；0；360；25
grep -v '^#' $T | awk -F'\t' '{print $2}' | sort | uniq -c   # A 36、A6 42、AC 38、C 24、C6 30、E1 25、E2 4、L 24、L6 30、R 26、U_verify_off 26、U_verify_on 29、X 26
nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/e165-section2-quote.md "$D~^\| 7 \| \*\*根记录的字段表" "$D~^\| 16 \| \*\*系统配置槽轮换规则" "$D@#### 已定项 7：根记录的字段表" "$D@#### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位"   # ✓ 4 段整抄，回读逐字节一致
cat <草稿目录>/e165-section2-quote.md >> research/prompts/e165-r7-prereg.md
nice -n 19 python3 -I anchors_e165_r7_ka.py; nice -n 19 python3 -I anchors_e165_r7_kl_counts.py      # 在草稿目录的 anchors/ 里跑
python3 -I -c '<KL 上「取最后一个自证过的」那一格>'                                                     # last_self_verified_wins_on_KL 16（B6）
nice -n 19 python3 scripts/kb/replace-once.py research/prompts/e165-r7-prereg.md <旧串> <新串>      # 写登记时的定点修正，都在装置改动与产物之前
```

`anchors_e165_r7_ka.py` 全文：

```python
# E165 第 7 次重跑登记：KA 照落点公式摆根之后的算术锚点。独立于装置与 crates 的代码，只用条款字面：
# 落点 D22 已定项 16「区域内槽位 = (txg div R) mod S」、区域 = txg mod R（R = 3），读根环次序「区域号升序、区域内槽号升序」，
# 择新 D22 已定项 7 「(checkpoint_txg, 实例代号) 最大，平局取读根环次序最前」，C743「不住公式那一槽的根当这一槽读坏」（第 0 代根另认，这里不摆第 0 代根）。
# 本地择根序五种照第 3 次登记 9.1 四行替换文的比较式与第 4 次加的「平局取后者赢」。
import itertools
R, S = 3, 8
def slot_of(txg): return (txg % R, (txg // R) % S)
def order_index(slot): return slot[0] * S + slot[1]
RULES = {
    "newest_wins":                  lambda c, k: c[0] > k[0] or (c[0] == k[0] and c[1] > k[1]),
    "tie_takes_the_lower_instance": lambda c, k: c[0] > k[0] or (c[0] == k[0] and c[1] < k[1]),
    "instance_first_then_txg":      lambda c, k: c[1] > k[1] or (c[1] == k[1] and c[0] > k[0]),
    "ignores_the_instance":         lambda c, k: c[0] > k[0],
    "first_self_verified_wins":     lambda c, k: False,
    "tie_takes_the_later":          lambda c, k: c[0] > k[0] or (c[0] == k[0] and c[1] >= k[1]),
}
def choose(rule, ring):
    chosen = None
    for v in ring:
        if v is not None and (chosen is None or RULES[rule](v, chosen)):
            chosen = v
    return chosen
def domain(slots, candidates_per_slot, check_landing):
    """slots：摆根的槽（按读根环次序排好）；candidates_per_slot：每个槽的候选根 (txg, 实例)，另加空。"""
    rows = []
    for combo in itertools.product(*[[None] + c for c in candidates_per_slot]):
        ring = []
        for slot, v in zip(slots, combo):
            if v is not None and check_landing and slot_of(v[0]) != slot:
                v = None   # C743：错位读当这一槽读坏
            ring.append(v)
        rows.append((combo, ring))
    return rows
def report(name, rows):
    n = len(rows)
    empty = sum(1 for _, ring in rows if all(v is None for v in ring))
    same_txg = sum(1 for _, ring in rows if any(a and b and a[0] == b[0] and a[1] != b[1] for a, b in itertools.combinations(ring, 2)))
    inst_high_txg_low = sum(1 for _, ring in rows if any(a and b and ((a[0] < b[0] and a[1] > b[1]) or (b[0] < a[0] and b[1] > a[1])) for a, b in itertools.combinations(ring, 2)))
    kj2 = {}
    for rule in RULES:
        bad = 0
        for _, ring in rows:
            present = [v for v in ring if v is not None]
            if not present:
                continue
            bad += choose(rule, ring) < max(present)
        kj2[rule] = bad
    print(name, "states", n, "ring_empty", empty, "same_txg_other_instance", same_txg, "instance_higher_txg_lower", inst_high_txg_low)
    print(name, "kj2", " ".join(f"{k}={v}" for k, v in kj2.items()))
    return kj2
inst = (1, 2)
# 旧 KA（第 3 次登记 5.4）：区域 0 槽 0、1、2 各取空或四条根之一。不核落点：应给回第 3 次登记第七节乙类的 125、1、36、18、18、52、0（自证这份脚本的择根序）。
old_slots = [(0, 0), (0, 1), (0, 2)]
old_roots = [(t, i) for t in (100, 101) for i in inst]
report("old_ka_without_landing_check", domain(old_slots, [old_roots] * 3, False))
report("old_ka_with_landing_check", domain(old_slots, [old_roots] * 3, True))
# 新 KA 候选甲：txg 100、101 各自的落点槽，各取空或那个 txg 的两条根之一（9 态）
for txgs, label in (((100, 101), "new_ka_two_txgs"), ((100, 101, 102), "new_ka_three_txgs"), ((101, 102), "opposite_order_two_txgs")):
    slots = sorted((slot_of(t) for t in txgs), key=order_index)
    by_slot = {slot_of(t): [(t, i) for i in inst] for t in txgs}
    print(label, "slots_in_ring_order", [(s, order_index(s), [v[0] for v in by_slot[s]][0]) for s in slots])
    report(label, domain(slots, [by_slot[s] for s in slots], True))
```

sha256 `638520dbd8903783dd69961e8befefe815712b9d2699a72d65669b165220d1ae`；原样输出（sha256 `a1aaef359582a12c43d89cc24753fe58de5b0de162776d9debd6f37ad1001299`）：

```text
old_ka_without_landing_check states 125 ring_empty 1 same_txg_other_instance 48 instance_higher_txg_lower 24
old_ka_without_landing_check kj2 newest_wins=0 tie_takes_the_lower_instance=36 instance_first_then_txg=18 ignores_the_instance=18 first_self_verified_wins=52 tie_takes_the_later=0
old_ka_with_landing_check states 125 ring_empty 125 same_txg_other_instance 0 instance_higher_txg_lower 0
old_ka_with_landing_check kj2 newest_wins=0 tie_takes_the_lower_instance=0 instance_first_then_txg=0 ignores_the_instance=0 first_self_verified_wins=0 tie_takes_the_later=0
new_ka_two_txgs slots_in_ring_order [((1, 1), 9, 100), ((2, 1), 17, 101)]
new_ka_two_txgs states 9 ring_empty 1 same_txg_other_instance 0 instance_higher_txg_lower 1
new_ka_two_txgs kj2 newest_wins=0 tie_takes_the_lower_instance=0 instance_first_then_txg=1 ignores_the_instance=0 first_self_verified_wins=4 tie_takes_the_later=0
new_ka_three_txgs slots_in_ring_order [((0, 2), 2, 102), ((1, 1), 9, 100), ((2, 1), 17, 101)]
new_ka_three_txgs states 27 ring_empty 1 same_txg_other_instance 0 instance_higher_txg_lower 7
new_ka_three_txgs kj2 newest_wins=0 tie_takes_the_lower_instance=0 instance_first_then_txg=6 ignores_the_instance=0 first_self_verified_wins=4 tie_takes_the_later=0
opposite_order_two_txgs slots_in_ring_order [((0, 2), 2, 102), ((2, 1), 17, 101)]
opposite_order_two_txgs states 9 ring_empty 1 same_txg_other_instance 0 instance_higher_txg_lower 1
opposite_order_two_txgs kj2 newest_wins=0 tie_takes_the_lower_instance=0 instance_first_then_txg=1 ignores_the_instance=0 first_self_verified_wins=0 tie_takes_the_later=0
```

`anchors_e165_r7_kl_counts.py` 全文：

```python
# KL（照落点公式摆根的对抗根环）的另外几样计数：根环里有两个以上实例的态数、读根环次序里第一个自证过的根不是版本最大那条的态数，
# 以及按 (txg 102 槽, txg 100 槽, txg 101 槽) 的取值逐态列出今天的序与五种变体序择到的根（核 anchors_e165_r7_ka.py 的合计）。
import itertools
R, S = 3, 8
def slot_of(txg): return (txg % R, (txg // R) % S)
txgs = sorted((100, 101, 102), key=lambda t: slot_of(t)[0] * S + slot_of(t)[1])
print("ring_order_txgs", txgs, "slots", [slot_of(t) for t in txgs], "order_index", [slot_of(t)[0] * S + slot_of(t)[1] for t in txgs])
two_or_more_instances = first_not_newest = 0
states = 0
for combo in itertools.product(*[[None, (t, 1), (t, 2)] for t in txgs]):
    states += 1
    present = [v for v in combo if v is not None]
    if len({v[1] for v in present}) >= 2:
        two_or_more_instances += 1
    if present and present[0] != max(present):
        first_not_newest += 1
print("states", states, "two_or_more_instances", two_or_more_instances, "first_self_verified_root_is_not_the_newest", first_not_newest)
```

sha256 `17bc3441b9b0ed0ccec1496c6abf495c2492a088600db1e57384c1630ea783ba`；原样输出（sha256 `15629371ef76640ca8351d81f1466610305a9b21f06ab03ecb1cb20475a44cff`）：

```text
ring_order_txgs [102, 100, 101] slots [(0, 2), (1, 1), (2, 1)] order_index [2, 9, 17]
states 27 two_or_more_instances 12 first_self_verified_root_is_not_the_newest 4
```

## 十四、并行与挂钟

独立单位：P1、P2、P3 三份产物彼此独立（同一个 B1 编出的本体、不同参数）；每份里合法历史各格按工作单元切给 `E165_THREADS` 条线程，KL 27 态单线程走（装置今天的写法，第 11661 行注释）；第二段 T1 与第一段的产物互相独立，可以同时起；第三段要第一段的根与 H1–H8 落地之后才起。
线程：B1 与 P1–P3 取登记值 31（`.claude/heavy-thread-default`，`capped.sh 31`、`E165_THREADS=31`）；三份产物的计数与线程数无关（5.8），31 只影响挂钟；T1 编译 31、测试并行度用 cargo 默认。
串行：P1、P2、P3 依次跑——每份已占满 31 条线程，本机 `nproc` 32，三份并起来跑只是互相抢核；比对（LQ5、LQ6）要等对应那份产物落盘；第三段等第一段（要它的根与改好的装置）。
预计挂钟：第一段约 1 小时（推的：B1 约 10 分钟；P1 是 r3 5.9 推的「几分钟」量级的 294 万态；P2、P3 各约 1 386 万态，按 `replay.sh` 第 2184 行注释里 r4 模式七个变体合计约 15 分钟、每个变体一遍，单遍推的几分钟；改装置、写比对脚本与登记约 30 分钟）；第二段 T1 不超过 60 分钟（限时 3600 秒，含编译）；第三段（附带）约 1 小时（同一条注释：r3 模式约 10 分钟、r4 模式约 15 分钟，加建根与编变体）。
预计在用核数：第一段 B1 与 P1–P3 各约 31 个核（依次），比对 1 个核；第二段编译约 31 个核、跑测试约 31 个核以内；第三段 `--jobs 2 --threads 3` 照旧约 6 个核，编变体时 31。

## 待主 agent 认的

1. **问题单**：派发提示写「岔路单：重跑」，没有另给问题单。1.2 的 L1–L4 是按派发提示第 2 条的验收写的，认之前不派执行员。
2. **域的取法**：KL 取 txg 100、101、102 三个、27 态（1.3「为什么三个 txg」）；要只取 100、101 的 9 态，第七节 B5 已算好，在装置写之前改，改了是新的一次。
3. **新名 KL**：KA 的定义变了，照起名规矩起新名；要沿用 KA 这个标签，在装置写之前说，改动表 H5、H7 与第十一节的 L2 比法跟着改。
4. **v1822、v1823、v1825、v1827 的变体臂对照**：在 KL 上它们读 0（同 txg 两条根不能同在、KL 不摆第 0 代根），这一次把它们的对照改由 `crates/mutations.tsv:1634`、`:1635`、`:1637`、`:1639` 点名的 crates 测试担（5.3 最后一行）。另一条路是在装置里另加一个对抗域，像 harness 测试 `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot` 工作区暂存的那一版那样，在第 0 代根的槽里摆两条 txg 0、实例不同的根——用户这次定的是「KJ2 的平局那半没了」，这一条没登记，要不要另开交主 agent。
5. **第三段跑不跑**：L3 是附带，主 agent 看第一段交回再定。
6. **kb 与实现对不上**：D22 已定项 7 实例代号那一行仍写「公式核还没实现，欠在 C743」，`root_ring.rs:177–193` 与 `recovery.rs:1732–1765` 已经核落点（第三节）。这份登记不改 kb，交主 agent。

## 没做什么（固定会有的）

没写代码、没跑装置；登记在产物之前写死，之后要改由主 agent 按 `evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」那三步处理。准入（`python3 scripts/gate/admission.py experiment <仓根> E165`）没跑，归主 agent。
