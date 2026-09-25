# E43 重跑登记（记录头 307 → 311）：扩展点字节上限的自证单元档在头 311 下还判不判出原来的结论

写于 2026-09-24 20:50 JST，装置改之前、这一次的任何产物之前。原判据在 `.claude/kb/experiments/43-扩展点字节上限.md`「### 要测什么、判据、失败条款」（E43 没有单独的跑前登记文件，`git log --all --name-only` 里没有 `e43-preregistration.md`，原判据就是实验页这一节与装置文件头那张三段表，第二节整段抄）。

判据、门槛、作废条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**装置写在哪（写死）：`research/e7-index-bench/src/bin/e43_extension_point_budget.rs`（独立手写模型，不与 `crates/` 共用代码），不是入库装置。** 这条问题问的是「E43 跑前写死的判据在头 311 下判不判得出原结论」，不是「`crates/` 今天这份代码的性质」；`crates/` 里扩展点只有一个写 0 的声明值（第三节），E43 的模型答不了也不需要驱动它。变异表 `research/mutations/e43_extension_point_budget.tsv`，复跑行 `research/scripts/replay.sh` 的 E43 那一行。

文件名取 `r1`：派发提示点的；E43 在仓里（含 git 历史）没有任何 `e43-r*-prereg.md`。仓里另有「`r<n>` = 第 n 次跑」的用法（`e156-r2-prereg.md` 标题「第 2 次」），按那个用法 E43 这一次不是第 1 次；文件名要不要改由主 agent 定，本文正文不自引文件名。

## 一、问题

**主 agent 给的问题（逐字）**：记录头 `JOURNAL_HEADER_BYTES` 从 307 改成 311 之后，每个实验跑前写死的判据在 311 下还判不判出原来的结论。

**读法写死**：

| 名字 | 写死成 |
|---|---|
| 头宽 `H` | 装置里 `const JOURNAL_HEADER_BYTES` 的值。这一次只有两个被判的取值：`H = 307`（今天仓里的装置，改之前先复跑一遍当基线）与 `H = 311`（`crates/singlefs-format/src/lib.rs:165` 与 D23 已定项 4 今天的值）。第八节另有只为几何敏感性而取的点，不参与「变不变」的判定 |
| 「原来的结论」 | 不读实验页的结论节。读作：**原判据的每一格在 `H = 307` 的产物上判出的那个值**。`H = 307` 的产物由这一次改装置之前现跑（`replay.sh E43` 逐字节一致才算基线立住，第十一节 S1），不引旧产物里的数 |
| 「还判不判出」 | 第六节每个判定格在 `H = 307` 与 `H = 311` 两份产物上各判一次；两次相同记「不变」，不同记「变」并写明是哪一格、两边各是什么 |
| 一格 | 第六节的一行。一行一个判定，不合取 |
| 取值范围 | 原装置的全部格（段一、段二、段三、自证单元档、挂载时求值、阳性对照）照跑；只有自证单元档与挂载时求值里「自证溢出」那一支读 `H`（第三节末尾那张表逐处列了），其余格要求逐字节不变（第六节 Q43.1） |

**问题单（`research/prompts/m2-header311-rerun-questions.md`，表头与第 1、7 行，quote-kb 机械抄）**：

**出处 `research/prompts/m2-header311-rerun-questions.md:5-6`（整段抄，未转述）**

```markdown
| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m2-header311-rerun-questions.md:7-7`（整段抄，未转述）**

```markdown
| 1 | E43（扩展点字节上限） 的自证单元余量与上界在头 311 下结论变不变 | 不变 / 变 | 512 与 4096 两档上的余量换成 311 算出之后，跨过这个实验跑前写死的阈值 | 装置常量改成 311、单测与变异照改、重跑，对着原判据逐格判 | 开着 |
```

**出处 `research/prompts/m2-header311-rerun-questions.md:13-13`（整段抄，未转述）**

```markdown
| 7 | E23、E39、E42、E49、E61、E75 几份实验页引「现行头 307」的现状句 | 只改句子 / 要重跑 | 那句话参与了该实验的判据（例如 E39 的四档头宽、E49 的 base 口径），换成 311 之后判据要重算 | 逐句判：只是引用现行值的改句子；参与判据的写明要重跑、另列 | 开着 |
```

第 7 行不是重跑，是逐句判的清单，附在本文末尾「附：问题单第 7 行的逐句判定」，不占第六节的判定格。第 6 行（E142）等前置，不在这一份里。

## 二、被测条款与它引的定义

被测条款是 D23 已定项 4（头宽）与已定项 17（点名项 56 字节）；E43 的原判据是这一次拿来判的尺子，一并整段抄。三段都用 `research/scripts/quote-kb.py` 抄进草稿目录、回读逐字节一致之后追加进来。

### 2.1 D23 已定项 4、已定项 17（被测条款）

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:139-165`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 311 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants.md](../invariants.md) I-8.2（记录头不跨原子单元））。头 **311 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到序号 0 或一次发布之内跳号，当那条记录损坏、断链即止（用户 2026-09-24 定，不变量随 invariants 那一批另写）。
- **已定项 15 的新根段 188 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 311（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 311 占 512 扇区的 61%，其后还余 **201 字节**（装得下 3 个点名项，每项 56 字节）；4096 上余 **3785**（67 个点名项，⌊(4096 − 311) ÷ 56⌋ = ⌊3785 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 311 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋ -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/27-format-constants.sh`）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。

**欠**：C94（登记的格式常量与后来的定案对不上）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:440-459`（整段抄，未转述）**

```markdown
#### 已定项 17：点名项 56 字节的字段表

**定案**：**一项一个单元（第一个事务 8 项）：位置条目 14 × 2（偏移 0）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）= 56；不另带载荷 CRC（位置条目各带 4 字节校验和）。重放从点名项直接凑出映射 key。**

- **记录点名的每一项自带校验和**：D4（校验和位置） 已定校验和内联进父指针，而记录点名的东西在记录持久时它的父还没发出 ⇒ 记录临时充当父，否则重放时那些单元没有任何完整性凭据；恢复重放施加前必须逐项验证点名单元（已定项 14 第四条）。
- **一次发布切成 N 个事务、N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名**，前面几条不重复点名它们。
- **共享的提交内生块多于最后一条记录装得下（4096 记录 67 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第六条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改第一个事务的字节（w1、w4、t9 三条记录的这一字节与头部校验和）。用户 2026-09-24 定。

**射程**：管点名项的字节；位置条目的构成归 D19（块指针的结构与宽度预算） 已定项 4 / 11，映射 key 的形态归 D19（块指针的结构与宽度预算） 已定项 6 / 10。⚠️ 共享的提交内生块只在最后一条点名时，崩在最后一条持久之前，那些单元没有任何记录点名——重放的回收路径要认得出这一格，层 0 的小负载要覆盖它（C491（多条记录时共享内生块在哪条点名没定））。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：记录点名的东西其父此刻还没写到盘上，记录必须临时充当父——逼出「点名的每一项自带校验和」。
- E77（发布的持久顺序）：点名项自带校验和、施加前逐项验证，是单元与记录之间那道屏障得以省掉的交换条件。
- 用户定案，原话在变更史。
- 用户定案 2026-09-23：共享的提交内生块只在最后一条记录里点名，原话在变更史。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6：维持拒绝时拒不拒取决于池的历史，按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。

**欠**：C491（多条记录时共享内生块在哪条点名没定）；末条标志的写路径、恢复按标志认边界、checker 与层 0 流 L8 随实现：一条用例写出两条记录、断言共享单元只在最后一条里点名。

```

### 2.2 E43 的原判据（实验页「要测什么、判据、失败条款」整节）

**出处 `.claude/kb/experiments/43-扩展点字节上限.md:26-55`（整段抄，未转述）**

```markdown
### 要测什么、判据、失败条款

- **测什么**（三段，都不需要文件系统，纯算术与计数模型）：
  1. **上界之一，空间对照**：扫 `N ∈ {0, 16, 32, 64, 128, 256, 512}` × 单元 ∈ {4, 16, 64, 128} KiB，
     算每单元元数据字节数与占比，与 ext2 / ext4 / btrfs / ZFS 四行同口径对照。
     ⚠️ **两种记法都要报**：把扩展点**计入**「为管理一个单元而多付的字节」，和**不计入**（记为开线方净荷）。
  2. **上界之二，扇出与树高**：扩展点吃掉节点可用字节 ⇒ 扇出下降。
     按 E29（坏一个节点的爆炸半径） 的口径（1600 万叶、指针 40 字节）算每个 `N` 下的扇出与树高。
  3. **下界**：扩展点至少要放得下**一个合法指向 + 它的配额边界**，
     位置条目宽度由 D19（块指针的结构与宽度预算） 已定的字段独立算出（dev 1 + 物理偏移 6，加长度与边界描述）。
     小于这个值的 `N` ⇒ 扩展点在结构上放不下任何指向 ⇒ **该功能等于不存在**，而 D21（权威态与派生态的分界） 已定它的形态就是「放一个指向」。
- **判据**：三段各给一个边界，取**交集**。
  - **对照翻转点**：`N` 使本工程在 128 KiB 单元上的每单元元数据字节数超过 ZFS 的 128.0 B 或 btrfs 的 207.3 B ⇒
    D21（权威态与派生态的分界） 正文「本工程在 128 KiB 单元上比 ZFS 省、比 btrfs 省一半」那句话**不再成立**。
    ⇒ 要么把 `N` 收窄到翻转点以内，要么改写那句话并写明改的是哪种记法。
  - **树高**：`N` 不得使任一单元档的树高比 `N = 0` 时**多一层**。
  - **下界**：`N ≥` 第 3 段算出的最小可用值。
  - 三段的上界互相矛盾时**取最紧的那个**，并写明是哪一段在约束，**不许平均、不许挑宽的那个**。
- **阳性对照**（**四个单元档各跑一次，不是只跑第一档**）：每一档都把 `N` 抬到该档单元大小的一半，
  占比与扇出必须显著变化且等于独立算出的值——否则模型根本没在把扩展点算进去，**整轮作废**。
- **失败条款**：
  - 阳性对照测不出变化 ⇒ **实现有问题，整轮作废**。
  - 三段的交集为空 ⇒ **合法结果，如实记录**：结论是「这个数在现有口径下定不了」，
    要改的是口径或 D21（权威态与派生态的分界） 那句对照结论，**不许换一种记法把 128 坐实**。
  - 「全单元带」与「仅数据单元带」两档结论不同 ⇒ 如实记录两个，并把那个岔路退回 D21（权威态与派生态的分界）。
- ⚠️ **必须带绝对值断言**（这三段全是臂间比较，只互比测不出四条臂一起错）：
  - `N = 0` 档必须逐字复现 D21（权威态与派生态的分界） 对照表：158.0 B、4 KiB 3.857%、128 KiB 0.121%、4+2 条带 202.0 B（数值随对照表的现行口径走）。
  - `N = 0` 档的扇出必须等于 E29（坏一个节点的爆炸半径） 的 49 / 100 / 408 / 1636，树高必须等于 5 / 4 / 3 / 3。
  - 每个 `N` 的占比由 `(158 + N) / 单元字节数` 独立算出，**不许从被测代码读回来**。

```

### 2.3 装置里读 `H` 的那一段（自证单元档与挂载判定，装置第 167–206 行）

原判据（2.2）里没有自证单元档：这一档与「挂载时求值一次」是后加进装置的（实验页「### 挂载时求值一次」写的是用户 2026-08-30 定案之后加的判定函数），它们的判定写在装置里，没有另一份跑前文字。这一次对它们的判据在第六节写死。

**出处 `research/e7-index-bench/src/bin/e43_extension_point_budget.rs:167-206`（整段抄，未转述）**

```markdown
    node - NODE_HEADER_BYTES - POINTER_BYTES
}

// ── 自证单元那一档（D20 推论三：根槽、journal 记录头）──
// 它们没有带校验和的父指针，原子宽度**等于运行时探测到的 `physical_block_size`**。
// ⇒ 扩展点在这一档的余量由**原子宽度**夹，不由「省不省」夹。
const JOURNAL_HEADER_BYTES: u64 = 307; // D23 已定项 4：头 307 字节 = 十个字段 78 + 已定项 7 / 8 / 13 三笔已定增量 17 + 已定项 15 新根段 188 + fsid 8 + MAC 16
const ROOT_SLOT_CANDIDATE: u64 = 256; // D22 已定项 2 的候选槽宽

/// 一个自证单元的头部落进一个原子单元之后，还剩多少字节。
/// D23 已定项 4 逐字：「头是 307 字节（……），占 512 扇区的 60%，其后还余 205 字节（装得下 3 个点名项，
/// 每项 56 字节）；4096 上余 3789（67 个点名项）」。
fn self_witness_room(header_bytes: u64, atomic: u64) -> u64 {
    atomic - header_bytes
}

/// 一个原子单元里挤进几个槽。E34 主张一：槽宽 256、原子宽度 512 ⇒ **2 个**，
/// 于是写一个槽可能撕裂邻槽。撕裂隔离要求这个数为 1。
fn slots_per_atomic(slot: u64, atomic: u64) -> u64 {
    (atomic / slot).max(1)
}

/// 挂载时求值一次的几何判定。**返回「挂得上 / 挂不上」，不返回「慢一点」**——
/// 这是 `.claude/rules/fs-design.md`「能不能把『用错了』变成『挂不上』」在本项上的形态。
/// 输入全部是**操作期间不会变的量**：系统配置里声明的 N、格式里的单元与节点大小、
/// 运行时探测到的 `physical_block_size`。⇒ 挂载时算一次，运行时只查不算。
fn mount_verdict(extension_point_bytes: u64, unit: u64, node: u64, slot: u64, atomic: u64) -> &'static str {
    if extension_point_bytes > maximum_extension_point_bytes_keeping_payload_positive(unit) {
        return "reject_no_payload"; // 净荷被挤成 0
    }
    if extension_point_bytes > maximum_extension_point_bytes_keeping_one_pointer_in_node(node) {
        return "reject_no_pointer"; // 节点里放不下一个指针
    }
    if slots_per_atomic(slot, atomic) != 1 {
        return "reject_slot_shares_atomic"; // 撕裂隔离失效：一个原子单元里不止一个槽
    }
    if extension_point_bytes > self_witness_room(JOURNAL_HEADER_BYTES, atomic) {
        return "reject_self_witness_overflow"; // 自证单元的头顶不住一个原子宽度
    }
    "ok"
```

## 三、实现今天的样子

### 3.1 `crates/`（2026-09-24 现查，行号是那一刻的）

| 处 | 文件与行 | 今天是什么样 |
|---|---|---|
| 头宽 | `crates/singlefs-format/src/lib.rs:164-165` | `pub const JOURNAL_HEADER_BYTES: u64 = 311;`，注释列出 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16 |
| 头宽的分项和 | `lib.rs:303-315`（单测 `assert_eq!` 把 311 写成十个字段加八项之和）、`:323`（`assert_eq!(JOURNAL_HEADER_BYTES, 311, …)`） | 311 由单测钉住 |
| 记录宽、点名项、一条装几项 | `lib.rs:156`（`JOURNAL_RECORD_BYTES = 4096`）、`:168-169`（`JOURNAL_NAMED_ENTRY_BYTES` = 14 × 2 + 1 + 8 + 8 + 10 + 1）、`:172-173`（`JOURNAL_NAMED_ENTRIES_PER_RECORD = (4096 − 311) / 56`）、`:324-325`（钉 56 与 67） | 4096 / 56 / 67 |
| 写头的路径 | `crates/singlefs-core/src/journal.rs:151`（「记录标志 1」那个字节今天写 `0`，注释仍叫「对齐填充」）、`:165-169`（本次发布内序号落在 87）、`:184`（`assert_position(JOURNAL_HEADER_BYTES, "记录头")`） | 已定项 17 的末条标志还没写进代码（问题单第 6 行等的「实二十」）；**E43 不建头里的逐字节布局，只用头宽 `H` 这一个数，标志位不进本实验的任何一格** |
| 扩展点 | `crates/singlefs-core/src/system_configuration.rs:348`（`writer.put_u32(0); // 扩展点声明值 N：第一版 0`）、`crates/singlefs-harness/src/model.rs:251` | `crates/` 里扩展点只有一个恒 0 的声明值；E43 算的 N 的上下界与挂载判定函数 `crates/` 里都没有：`grep -rn 'extension_point\|扩展点\|ExtensionPoint' crates/ --include=*.rs` 只命中这两行 |
| 根槽 | `lib.rs:152-153`（`ROOT_RECORD_BYTES = 457`） | ⚠️ E43 装置里的 `ROOT_SLOT_CANDIDATE = 256`（装置第 174 行，「D22 已定项 2 的候选槽宽」）比今天的根记录 457 还窄。这与头宽无关、不在这一次的比较范围里，**不改、不判**，只记在这里交主 agent（第十一节 S3） |

### 3.2 装置里读 `H` 的每一处（`research/e7-index-bench/src/bin/e43_extension_point_budget.rs`，`grep -n 'JOURNAL_HEADER_BYTES\|307\|205\|3789' ` 现查）

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 173 | `const JOURNAL_HEADER_BYTES: u64 = 307;` 与行尾注释（分项里没有「本次发布内序号 4」） | 改成 311，注释照 D23 已定项 4 今天的分项写 |
| 177–178 | 文档注释逐字引 D23「头是 307 字节（……），占 512 扇区的 60%，其后还余 205 字节……4096 上余 3789」 | 换成 D23 已定项 4 今天的原句（第二节 2.1 里那一句），逐字 |
| 203 | `mount_verdict` 里 `extension_point_bytes > self_witness_room(JOURNAL_HEADER_BYTES, atomic)` | 不动代码，值随常量走 |
| 339–348 | 主函数 `name=self_witness kind=journal_record hdr=… atomic=… room=…` 两行 | 不动代码，值随常量走 |
| 373–382 | 主函数 `name=self_witness_bound n_max_journal_512=… n_max_if_self_witness_carries=…` 一行 | 不动代码，值随常量走 |
| 519–523 | 单测 `self_witness_room_matches_the_decision_23_number`：钉 205 / 3789 | 改成钉 201 / 3785（第七节 A1、A2）；注释照改 |
| 535–542 | 单测 `the_self_witness_bound_is_205_not_864`：钉 205、`bound < 864`、注释「紧 4.2 倍（864 ÷ 205）」 | 钉 201；注释的倍数换成第七节 B2 算出的那个数；函数名里的 205 换成 201 |
| 579–582 | 单测 `every_rejection_reason_is_reachable`（第 570 行起）里 `mount_verdict(500, 8192, 8192, 512, 512) == "reject_self_witness_overflow"` | 不动：500 > 512 − 311，第七节 B3 核过 |

除上表之外，装置里没有别处读 `H`：段一（空间对照）、段二（扇出与树高）、段三（下界）、根槽行、阳性对照都不经过 `JOURNAL_HEADER_BYTES`（`grep -n 'JOURNAL_HEADER_BYTES'` 在装置里命中 173、203、343、345、378、380、522、523、540，全在上表里）。


## 四、跑之前已经存在的数

读条款、读装置、判问法时已经撞见或自己算出的数，照实列，不删。**这一次的答案在跑之前大半已经算得出来**：头宽只进自证单元档那几格，而那几格的翻面点（下表末几行，第十三节命令二）全都离 307 与 311 很远。所以这一次重跑的用处不在「算出新答案」，而在三件事：装置、单测、变异、产物跟着格式常量一起动（D23 已定项 4 射程「改它要三处一起动」）；核实头宽没有从别处漏进其它格（Q43.1）；产物与我这边的算术对不上时停下来两边都查（第十一节 S2）。判定以产物为准，不以下表为准。

| 数 | 出处 | 对判据的影响 |
|---|---|---|
| 205、3789（`H = 307` 时 512 / 4096 上的余量），「紧 4.2 倍（864 ÷ 205）」、`min(205, 255) = 205`，「夹住这一档的是 journal 记录头那一侧」 | 装置第 177–178、519–542 行的单测与注释；回扫报告 `research/prompts/m2-header311-sweep-report.md` 第二节第 2 小节 E43 那一行也引了 205 / 3789 / 864 ÷ 205 | 是 `H = 307` 那一份的值，这一次改装置之前现跑复现它（S1），不拿这里的数当基线 |
| 201、3785、「3 个点名项」「67 个点名项」「占 512 扇区的 61%」 | 被测条款 D23 已定项 4（第二节 2.1） | 第七节 A 类锚点，出自条款本身 |
| 864（段二全单元带那一档的树高上界）、255（根槽 256 − 1）、下界 7 / 9 / 11 / 13、净荷上界 3990 / 130966、指针上界 1944 / 65432 | 装置单测（第 430 行起的 `mod tests`：512–515、535–542、591–594、615 行） | 第六节门槛的来源：它们是**别的段**给的数，不是自证单元档自己的定义，所以不是同义反复；它们都不读 `H`，两次跑必须逐字节相同（Q43.1） |
| 实验页标题「没夹出有约束力的上界，上界 A 作废、上界 B 是软线」；「实测四条拒绝理由都够得到」；「同一份声明（槽宽 512）在 512 上判 ok、在 4Kn 上判 `reject_slot_shares_atomic`」 | `grep -n '^#'` 看小节结构时看到标题行；读「### 挂载时求值一次」（第 97–112 行，为了知道挂载判定是怎么来的） | 是结论。第六节的判据不照它写：判据是「两次跑判出的值相同与否」，不是「判出上界 A 作废」 |
| 回扫报告说「未见 e43 专属 mutations 文件」 | 回扫报告第二节第 2 小节 | **与仓里对不上**：`research/mutations/e43_extension_point_budget.tsv` 在，19 行（M1–M19），第九节照它办 |
| **翻面点（我写登记时算出来的）**：自证上界的绑定侧换成根槽 255 要 `H ≤ 257`（`H = 257` 两侧相等）；下界 13 / 11 / 9 / 7 放不进 `min(512 − H, 255)` 要 `H ≥ 500 / 502 / 504 / 506`；D21 举例的 128 放不进要 `H ≥ 385`；挂载判定 N = 128、原子 512 转成自证溢出要 `H ≥ 385`，N = 512、原子 4096 要 `H ≥ 3585`，N = 128、原子 4096 要 `H ≥ 3969`；`864 ÷ 201 = 4.2985` | 第十三节命令二原样输出 | ⚠️ 跑之前就存在的答案：按它，307 与 311 在第六节每一格的翻面点同一侧。照纪律不删、不回改判据；第八节的敏感性取样点就取在这些翻面点两侧，让判据的判别力在产物里看得见；产物算出的翻面点与这里不同 ⇒ 停机 S2 |
| E23 / E39 / E42 / E49 / E61 / E75 实验页里的数：E23「现行头是 307、512 上余 205、4096 上余 3789」；E39「3961 vs 3906、16 vs 15.3」「四档 304 / 305 / 307 / 311、+0.33% / +0.66% / +1.32% / +2.64%」；E42「差额与头宽无关，结论不动」；E49 标题「64 优于 32，但只在记录核对器那个职责上」「代价侧那条结论 2026-09-07 起不再适用」「base = 78 那一档 2 vs 4 有 31 处分岔」；E61「甲在三档上一律满秩、丙在三档上一律不满」；E75 标题「4 KiB 那个定案站得住，但它引的头部口径是旧的」、容量表 1 KiB 16 / 2 KiB 35·34·34·34 / 4 KiB 71；E39 历史节「8 位 3961 ppm、16 位 16 ppm、32 位 0」 | 为判问题单第 7 行读了这几页里回扫报告点名的那几行（第十三节列了行号）与 E39、E75 的判据节 | 只用于本文末尾第 7 行的逐句判定；不进 E43 的任何判据 |

## 五、臂、阳性对照、真实基线

### 5.1 臂

- **原实验的臂照旧，一个字不改**：段一的「计入 / 不计入」两种记法、段二的 `Carrier::AllUnits` 与 `Carrier::DataOnly` 两档载体、段三的四档下界、自证单元档的 journal 记录头与根槽两种自证单元（第二节 2.2、2.3，装置第 75–80、167–206 行）。这一次不加臂、不删臂。
- **这一次新增的只有一个旋钮：头宽 `H`，两个被判的取值**：
  - **`H = 307`**：怎么做——今天仓里的装置不改一个字节，`bash research/scripts/replay.sh E43` 现跑；做完会怎样——输出与 `replay.sh` 登记的留存产物逐字节一致（这是 S1 要核的，不是假设）。
  - **`H = 311`**：怎么做——只按第三节 3.2 那张表改常量、注释与钉值的单测，别的一行不动，重新出产物；做完会怎样——第三节 3.2 列的三行输出（两行 `name=self_witness kind=journal_record`、一行 `name=self_witness_bound`）的值换成 `H = 311` 的值，其余输出行逐字节不变。「其余逐字节不变」是从「装置里只有那几处读 `H`」推出来的（第三节 3.2 的 grep），它本身由 Q43.1 核，不当判据用。
- 对面那条臂写成支持它的人认的样子：这一次没有「对面」的方案，两个取值都是真实发生过的格式（307 是 2026-09-14 起到 2026-09-24 的现行值，311 是今天的现行值），不存在稻草人。

### 5.2 阳性对照（每一条臂都跑）

| 对照 | 对哪条臂 | 怎么做 | 该看到什么 | 看不到时 |
|---|---|---|---|---|
| **P0 原实验的阳性对照** | `H = 307` 与 `H = 311` 各跑 | 不改：`name=poscontrol_space`（四个单元档）、`name=poscontrol_geom`（四个节点档）、`name=poscontrol_slot`，与单测 `positive_control_every_unit_arm`、`positive_control_every_node_arm`、`two_256_byte_slots_share_one_512_byte_atomic_unit` | 两次跑都绿，这几行两次逐字节相同 | 任一次红 ⇒ 作废 V1 |
| **P1 头宽这个旋钮读没读到** | 两个取值成对跑 | 比 `H = 307` 与 `H = 311` 两份产物里 `name=self_witness kind=journal_record atomic=512` 那一行 | 两行**不同**，且差正好是 4（205 → 201 这一类的差，不看具体值只看差） | 两行相同 ⇒ 装置没读到常量（改错了地方或二进制没重编）⇒ 作废 V2 |
| **P2 判别力（第八节敏感性行）** | `H = 311` 那一次的产物里 | 第八节 8.2 的每一对取样点 | 每一对翻面点两侧的判定不同 | 某一对两侧相同 ⇒ 那一格的判据在这个旋钮上没有判别力 ⇒ 作废 V3（那一格），修好重跑 |

### 5.3 真实基线

**真实基线是 `H = 311`**：它是 `crates/` 今天写的头宽（第三节 3.1）。`H = 307` 是比较的另一侧，不是基线。「两次判出的值全都相同」是正当结果，如实记「不变」；不许写成装置坏了（装置有没有读到 `H` 由 P1 单独判）。

### 5.4 实现量与分段

量很小：改 1 个常量、2 段注释、2 个单测的钉值与 1 个单测名；加第八节的敏感性输出与它的单测（约 40 行）；变异表加 3 条（第九节）；`replay.sh` 的 E43 一行换产物文件名。编译一次、单测一次、变异表两次（`H = 307` 改之前一次、`H = 311` 改之后一次，第九节要比三个数）、产物一次。**一段做完**，不分段；第 7 行的清单不在执行员的活里（它由主 agent 交 kb 写手或自己改句子，第 7 行附在本文末尾）。E43 实验页里引 307 的 7 行（回扫报告列的第 88、188、193、194、200、220、252 行，我没读，写登记时不看结果节）由执行员在产物出来之后按第六节各格的判定逐句改，每句里的数先对上新产物的整行再改。

## 六、报哪些量与各自的判据

全部对应问题单第 1 行（E43）。每格各报各的判定，不合取；「变」「不变」的合取留给主 agent。写门槛之前把臂的定义并排放一遍：`H = 307` 与 `H = 311` 的差只在 `512 − H`、`4096 − H` 两个数上（第五节 5.1）。下面每个门槛都来自**别的段或别的条款**（根槽 255、下界 9 / 13、D21 举例 128、挂载扫描的 N），不来自这两个数的定义。

| 格 | 怎么算 | 门槛 | 为什么不是同义反复 | 判定 | 问题单第 1 行：取什么值会让它翻面 |
|---|---|---|---|---|---|
| **Q43.1 头宽没漏进别的格** | `diff` 两份产物（`H = 307` 现跑那份、`H = 311` 那份），列出不同的行的 `name=` | 允许不同的只有：`name=self_witness kind=journal_record`（两行）、`name=self_witness_bound`、第八节新加的 `name=header_sensitivity_*` 行、`name=done` 的 `emitted=` | 装置有没有别处悄悄读 `H`（或改装置时手滑改了别处），只有 diff 看得见 | 超出允许集合 ⇒ **不是结果**，停机 S2；在允许集合之内 ⇒ 记「只动了自证单元档」 | 不直接翻面；它保证「原判据其余各格两次判出同值」这句话是量到的，不是推的 |
| **Q43.2 自证上界由哪一侧夹** | `name=self_witness_bound` 行：`n_max_journal_512` 与 `n_max_root_slot_256` 比大小 | journal 侧 `<` 255 记「journal 侧」，`=` 记「两侧相等」，`>` 记「根槽侧」 | 255 是根槽候选宽 256 减 1（D22 已定项 2 候选，装置第 174 行），与头宽无关 | 两次记的侧相同 ⇒ 不变；不同 ⇒ 变 | 换成根槽侧或两侧相等：`512 − H ≥ 255` 即 `H ≤ 257`（257 时两侧相等） |
| **Q43.3a 自证上界放不放得下主函数用的下界** | `n_max_if_self_witness_carries ≥ n_min_quota`（`name=bounds` 行，9） | `≥` 记「放得下」，`<` 记「区间空」 | 9 是段三的下界（dev 1 + 偏移 6 + 长度 2），不读 `H` | 同上 | `min(512 − H, 255) < 9` 即 `H ≥ 504` |
| **Q43.3b 自证上界放不放得下最宽的下界** | 同上，比 `n_min_full`（13） | 同上 | 13 是段三四档下界里最宽的一档 | 同上 | `H ≥ 500` |
| **Q43.4 D21 举例的 128 在自证单元带扩展点时放不放得下** | `n_max_if_self_witness_carries ≥ 128` | `≥` 记「放得下」 | 128 出自 D21 正文举例（第二节 2.2「现在那个 128 不算证据」那一句的对象），不读 `H` | 同上 | `H ≥ 385` |
| **Q43.5 挂载判定网格** | `name=mount_eval` 18 行（原子 {512, 4096} × 槽宽 {256, 512, 4096} × N {0, 128, 512}）的 `verdict=` 逐行比 | 字符串相等 | N 的三档与槽宽是原装置写死的扫描点，不读 `H`；只有「自证溢出」那一支读 `H` | 18 行逐行相同 ⇒ 不变；任一行不同 ⇒ 变，写明是哪一行 | N = 128 原子 512：`H ≥ 385`；N = 512 原子 4096 槽宽 4096：`H ≥ 3585`；N = 128 原子 4096：`H ≥ 3969`；N = 0 任何 `H` 都不翻（这几个翻面点只在「前面三条拒绝理由都没挡住」的那几行上起作用） |
| Q43.6 4096 上的余量（只报数） | `name=self_witness kind=journal_record atomic=4096` 的 `room=` | 不设门槛 | 它就是 `4096 − H`，从定义直接推出 | 只报数，与第七节 A2 对拍 | — |
| Q43.7 自证上界比段二上界紧几倍（只报数） | `864 ÷ n_max_if_self_witness_carries`，保留 4 位小数 | 不设门槛 | `min(512 − H, 255) ≤ 255 < 864` 由定义直接推出，「更紧」恒真 | 只报数；实验页与装置注释里「紧 4.2 倍」那一句照它改 | — |

**够判点**：`H = 307` 与 `H = 311` 两份产物都出来、Q43.1 在允许集合之内、Q43.2–Q43.5 各判完一次，第 1 行就够判。第八节的敏感性行是判据的判别力自证，不是够判条件的一部分；它红了按 V3 处理。

## 七、钉绝对值的断言

Q43.2–Q43.5 都是「两次比」的判定；两次一起错（比如常量改了、`self_witness_room` 同时被改坏）时靠下面这些绝对值发现。

### 7.1 出自被测条款本身的（不符走第十节 F1「条款可能错」，不作废）

| # | 断言 | 出处（第二节 2.1 已整段抄） | 装置里由谁钉 |
|---|---|---|---|
| A1 | `H = 311` 时 512 上余 **201** | D23 已定项 4「311 占 512 扇区的 61%，其后还余 **201 字节**」 | 单测 `self_witness_room_matches_the_decision_23_number` 改成钉 201 |
| A2 | `H = 311` 时 4096 上余 **3785** | 同一句「4096 上余 **3785**」 | 同一个单测钉 3785 |
| A3 | 余量 201 装得下 **3** 个点名项、3785 装得下 **67** 个（每项 56） | 同一句「装得下 3 个点名项，每项 56 字节」「67 个点名项」；已定项 17 的 56 | 新加一条单测：`201 / 56 == 3`、`3785 / 56 == 67` |
| A4 | 头 311 = 十个字段 78 + 三笔已定增量 17 + 本次发布内序号 4 + 新根段 188 + fsid 8 + MAC 16 | D23 已定项 4 分项 | 常量行注释照写；新加一条单测钉这个和（常量不从 `crates/` 引，第十一节 S3 另核两边相等） |

### 7.2 独立算出、用命令核过的（第十三节命令二；不符先核我这边的算术，核完仍不符 ⇒ 停机 S2）

| # | 断言 | 值 |
|---|---|---|
| B1 | `n_max_if_self_witness_carries` = `min(512 − 311, 256 − 1)` | **201**，journal 侧夹（201 < 255） |
| B2 | `864 ÷ 201` | **4.2985**（装置注释与实验页的「紧 4.2 倍」换成「紧 4.3 倍（864 ÷ 201）」） |
| B3 | `mount_verdict(500, 8192, 8192, 512, 512)` 在 `H = 311` 下仍是 `reject_self_witness_overflow` | 500 > 201 |
| B4 | 第八节敏感性行在各取样点上的值 | 见第八节 8.2 的「该看到」一列 |
| B5 | 不读 `H` 的各格照旧：`N = 0` 的 158.0 B / 3.857% / 0.121% / 202.0 B、扇出 49 / 100 / 408 / 1636、树高 5 / 4 / 3 / 3 | 原装置单测，这一次不改，两次跑都绿（它们是原实验的绝对值锚） |

**互比配的绝对值**：Q43.2 配 B1 + A1；Q43.3a / Q43.3b / Q43.4 配 B1 与原装置钉死的下界 7 / 9 / 11 / 13；Q43.5 配 B3 与原装置 `every_rejection_reason_is_reachable`、`the_same_declaration_can_mount_here_and_be_rejected_there` 两个单测（不改）。

## 八、轨迹与几何敏感性

### 8.1 轨迹

不适用：E43 是纯算术，没有轮次、没有随时间走的量，没有哪个量被条款当停机 / 起跑 / 准入谓词反复消费。挂载判定是挂载时求值一次的函数，不是轨迹。

### 8.2 几何敏感性（`H` 这个旋钮，每个判定格在翻面点两侧各取一点）

`mutation-sampling.md` 第六类要的那一行。307 与 311 两点在每一格上都落在翻面点同一侧（第四节），只比这两点看不出判据有没有判别力。⇒ 执行员在装置里加两种只读 `H` 参数、不读全局常量的输出行，产物里每个取样点一行：

- `name=header_sensitivity_512 h=<H> room512=<512−H> bound=<min(512−H,255)> binding=<journal|tie|root_slot> fits_lb9=<0|1> fits_lb13=<0|1> fits_128=<0|1> self_witness_n128=<ok|overflow>`，`H ∈ {256, 257, 258, 307, 311, 384, 385, 499, 500, 503, 504}`；
- `name=header_sensitivity_4096 h=<H> room4096=<4096−H> self_witness_n128=<ok|overflow> self_witness_n512=<ok|overflow>`，`H ∈ {307, 311, 3584, 3585, 3968, 3969}`。

`self_witness_n*` 只算「N 大于余量就溢出」这一支，不走 `mount_verdict` 前面三条拒绝理由（那三条不读 `H`）。

| 对 | 该看到（B4） | 对应格 |
|---|---|---|
| 256 / 257 / 258 | `binding` = `root_slot` / `tie` / `journal` | Q43.2 |
| 503 / 504 | `fits_lb9` = 1 / 0 | Q43.3a |
| 499 / 500 | `fits_lb13` = 1 / 0 | Q43.3b |
| 384 / 385 | `fits_128` = 1 / 0；`self_witness_n128` = `ok` / `overflow` | Q43.4、Q43.5 |
| 3584 / 3585 | `self_witness_n512` = `ok` / `overflow` | Q43.5 |
| 3968 / 3969 | `self_witness_n128` = `ok` / `overflow` | Q43.5 |
| 307 / 311 | 各格与 Q43.2–Q43.5 在两份主产物上判出的值相同 | 两个被判的点，交叉核一次 |

**判别力自证**：把任一格的门槛挪到那一对取样点之间（例如把 128 换成 129），那一对的判定必须由「1 / 0」变成「0 / 0」；单测里对每一对各写一条这样的断言。挪了不变 ⇒ V3。

## 九、变异

**先后**：改装置之前，在今天的源码（`H = 307`）上跑一遍 `bash research/scripts/mutate.sh e43_extension_point_budget research/e7-index-bench/src/bin/e43_extension_point_budget.rs research/mutations/e43_extension_point_budget.tsv`，留日志；改完之后在 `H = 311` 上用同一张表（加了下面三条之后）再跑一遍。两次各报「抓到 / 无效 / 没红」三个数（`mutation-sampling.md`「改了一个格式常量之后，要看「无效」那一栏有没有变多」）：原有 19 条在两次里的三个数必须相同，任一条从「抓到」变成「无效」或「没红」⇒ V4。

原有 19 条里与 `H` 有关的两条：

| 条 | 在哪个取样点上改变输出（`H = 311`） |
|---|---|
| M16 自证余量算错（`atomic - header_bytes` → `atomic - header_bytes / 2`） | 512 上余量 201 → 357、4096 上 3785 → 3941；A1 / A2 的单测红 |
| M19 挂载判定漏掉自证溢出 | `mount_verdict(500, 8192, 8192, 512, 512)` 从 `reject_self_witness_overflow` 变 `ok`（B3） |

新加三条（锚点由执行员照改完的源码取，必须唯一命中；命中不唯一按第七类处理，不许改成模糊锚点）：

| 条 | 改什么 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M20 头宽退回 307 | `const JOURNAL_HEADER_BYTES: u64 = 311;` → `307` | 512 上余量 201 → 205、4096 上 3785 → 3789 | A1 / A2 / B1 的单测 |
| M21 头宽落到根槽线上 | 同一行 → `257` | 自证上界 201 → 255，`binding` 由 journal 变 tie | B1 的单测（钉 201）与 Q43.2 那一格的单测 |
| M22 敏感性行的 128 判据写成严格小于 | `fits_128` 的比较 `128 <= bound` → `128 < bound` | 只有 `H = 384`（bound 恰好 128）那一格由 1 变 0，其余取样点不变 | 8.2 那一对 384 / 385 的单测 |

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | **条款可能错**：装置确实读的是 311（P1 过、M20 被抓），而 A1–A4 的某一条与产物对不上 ⇒ 不作废，如实记「D23 已定项 4 正文的算术与它自己的头宽不符」，交主 agent | `H = 311` 产物里 `name=self_witness kind=journal_record hdr=311 atomic=512` 那一行 `room=` 不等于 201，或 `atomic=4096` 那一行不等于 3785 |
| F2 | **结论变了（正当结果）**：Q43.2 / Q43.3a / Q43.3b / Q43.4 / Q43.5 任一格两次判出的值不同 ⇒ 第 1 行记「变」，写明哪一格、两边各是什么，实验页对应那一句要按新值改写，不许回头改门槛 | 两份产物里 `name=self_witness_bound` 的两个数的大小关系、`n_max_if_self_witness_carries` 与 9 / 13 / 128 的大小关系、或 18 行 `mount_eval` 的任一 `verdict=` 在两份里不同 |
| F3 | **全部不变（正当结果）**：Q43.2–Q43.5 五格两次都相同 ⇒ 第 1 行记「不变」，不许写成「装置没测出差别所以坏了」（装置有没有读到 `H` 由 P1 单独判） | 上面那些比较在两份产物里全部相同，且 P1 看得到 4 的差 |

## 十一、作废条款与停机条款

### 作废（这一次不出结论，修好重跑）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V1 | 原实验的阳性对照（P0）在任一次跑上红 | `cargo test` 里 `positive_control_*` 或 `two_256_byte_slots_share_one_512_byte_atomic_unit` 失败，或两份产物里 `name=poscontrol_*` 行不同 |
| V2 | 头宽旋钮没读到（P1） | 两份产物里 `name=self_witness kind=journal_record atomic=512` 那一行逐字节相同 |
| V3 | 敏感性行某一对两侧判定相同，或挪门槛之后判定不变 | 8.2 表里任一对的「该看到」不成立 |
| V4 | 改常量之后原有变异有一条从「抓到」变成「无效」或「没红」 | 两次 `mutate.sh` 日志的三个数不同，或逐条对比有一条的状态变了 |

### 停机（既不作废，也不当结果；两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住：改装置之前 `replay.sh E43` 不逐字节一致 | `replay.sh` 报 E43 不一致（这与头宽无关，先查是什么让今天的构建与留存产物不同） |
| S2 | 头宽漏进了别的格，或产物与我这边的算术对不上 | Q43.1 的 diff 超出允许集合；或 A1–A4 之外 B1–B4 任一条与产物不符（核完我这边的算术仍不符） |
| S3 | 装置与 `crates/` 对不上（`implementation-first.md` 第 4 条）：跑的那一刻 `crates/singlefs-format/src/lib.rs` 里 `JOURNAL_HEADER_BYTES` 不是 311、或 `JOURNAL_NAMED_ENTRY_BYTES` 不是 56 | 执行员开跑前 `grep -n 'pub const JOURNAL_HEADER_BYTES\|pub const JOURNAL_NAMED_ENTRY_BYTES' crates/singlefs-format/src/lib.rs` 与 `sed -n '168,169p'` 现查到的值 ≠ 311 / 56。⚠️ 根槽候选 256 与 `crates/` 根记录 457 对不上（第三节 3.1 末行）**不触发 S3**：它与头宽无关、这一次不比，只照报 |

### 够判停机

第六节「够判点」一到就停：问题单第 1 行记「变」或「不变」交回。没有第二段。没跑的量逐条标「够判后未跑」：无（Q43.6、Q43.7 只报数，随产物一起出，不单跑）。

## 十二、修订

装置写完、H=311 产物产出之前（`nice -n 19 cargo test --release --bin e43_extension_point_budget` 读数：25 单测全绿；`bash .claude/scripts/naming-lint.sh` 读数：本文件 10 处命名违规），两处纯实现层调整，不改判据、不改门槛、不改臂：

1. 第八节敏感性代码里循环变量 `h`（单字母）改名 `header_bytes_sample`；测试里的 `b503`/`b504`/`b499`/`b500`/`b384`/`b385`（单字母加数字）改名 `bound_at_503` 等——naming-lint 判红后原样改，不影响任何输出字段名与判定值。
2. `fits_128` 从「借用通用 `fits(bound, lower_bound_bytes)` 传 128」改成一个专用函数 `fn fits_128(bound: u64) -> u8 { u8::from(bound >= 128) }`——第九节 M22 写的锚点「`128 <= bound` → `128 < bound`」在通用 `fits` 函数上会连带改坏 `fits_lb9`／`fits_lb13`（M22 的失败条款要求「只有 H=384 那一格翻」不成立）；专用函数把 M22 的作用范围收紧到只对 `fits_128` 输出，收严而非放宽，判据与门槛一个字未改。M22 的实际锚点因此是 `research/mutations/e43_extension_point_budget.tsv` 里 `fn fits_128(bound: u64) -> u8 {\n    u8::from(bound >= 128)\n}` → `... bound > 128 ...`，与本文正文第九节文字性描述的语义相同、写法不同。


## 十三、读过的文件与跑过的命令

五份头 311 重跑登记（E43、E116、E155、E157、E159）是同一次派发里一起写的，这一节五份相同：列的是这一次派发里读过的全部文件，不只这一份用到的。行号是读的那一刻（2026-09-24 20:37–21:50 JST）的行号。`research/results/` 下的产物一份都没读（只在 `git log --all --name-only` 的输出里看到过文件名）。

### 13.1 规则、共用约束、门禁与脚本

- `.claude/agent-common.md`：1–71
- `.claude/singlefs-ai-sop/rules/test-discipline.md`：`grep -n '^#'` 全部标题；44–153
- `.claude/rules/three-way-inference.md`：`grep -n '^#'`；128–147
- `.claude/singlefs-ai-sop/rules/evidence-discipline.md`：`grep -n '^#'`；119–190
- `.claude/rules/mutation-sampling.md`：`grep -n '^#'`；45–53、76–95
- `.claude/rules/implementation-first.md`：`grep -n '^#'`；1–25
- `.claude/hooks/agent-write-scope.tsv`：`head -30`（全文）
- `.claude/gate.d/27-format-constants.sh`：60–125
- `.claude/gate.d/69-evidence-in-repo.sh`：`grep -n 'tmp'` 的命中行；20–37、178–200
- `.claude/gate.d/`：`ls`；`grep -ln 'preregistration\|prereg' .claude/gate.d/ -r` 零命中
- `research/scripts/replay.sh`：1–60；`grep -n 'e43\|e116\|e155\|e157\|e159'` 命中 63、77、171–174、176、192
- `research/scripts/quote-kb.py`：1–60；`research/scripts/replace-once.py`：1–30；`research/scripts/mutate.sh`：1–40
- `research/e7-index-bench/src/lib.rs`：67–88
- `research/e7-index-bench/Cargo.toml`：`grep` 命中 317–318、421–438、445–446

### 13.2 派发给的输入、被测条款与原登记

- `research/prompts/m2-header311-rerun-questions.md`：1–15（全文）
- `research/prompts/m2-header311-sweep-report.md`：`grep -n '^#'`；51–73
- `.claude/kb/decisions/23-journal的角色与格式.md`：`grep -n '^#'`；139–165、440–459
- `research/prompts/e157-preregistration.md`：`grep -n '^#'`；1–66、280–425、464–612
- `research/prompts/e159-preregistration.md`：`grep -n '^#'`；`grep -n 'smoke\|冒烟\|anchors'` 命中行；1–46、383–470、501–534、609–645
- `research/prompts/e156-r2-prereg.md`：1–12
- 归档的 E155 四份登记（`git show 3cff909^:research/prompts/e155-preregistration.md`、`git show 3cff909^:research/prompts/e155-r2-prereg.md`、`git show 8186d5b^:research/prompts/e155-r3-prereg.md`、`git show 8186d5b^:research/prompts/e155-r4-prereg.md`，拷进草稿目录读）：各自 `grep -n '^#'` 的前 60 行；`grep -n '307\|3789\|⌈[^⌉]*67\|/ *67\|÷ *67\|\b67\b\|RECORD_HEADER\|记录头宽'` 的命中行（第 1 次 659、720、727、749、769、815–824、888、899、907、912；第 2 次 25–26、33、262、563、736、782–791、826、851、857、882–891、997、1012、1262、1289；第 4 次 266–311、401、403、410、459、469、556、567、584、608、650–651、865、874、913、973、1030、1033、1053、1066）；第 3 次 `grep -n '67'` 的命中行（104、109、244、414、467、572、585、590、592、593）；四份的「六」「十」与 5.x 网格小节的起始行

### 13.3 `crates/`

- `crates/singlefs-format/src/lib.rs`：150–180、295–330、302–316；`grep` 命中 11–12、39–44、74、119、152–153、156、172、175–176、184–198、206、254、286、321–325、340
- `crates/singlefs-core/src/journal.rs`：15–35、140–200、162–170、230–250；`grep` 命中 21–27、113、151、167、169、227、259、283、294–311、360、379
- `crates/singlefs-checker/src/lib.rs`：`grep` 命中 13–15、286、454、467、478、560、590–591
- `crates/singlefs-core/src/transaction.rs`：`grep` 命中 15、2621
- `crates/singlefs-core/src/system_configuration.rs`：`grep` 命中 49、148、150、172、174、348、382、650
- `crates/singlefs-core/src/recovery.rs`：`grep` 命中 1679
- `crates/singlefs-harness/src/model.rs`：`grep` 命中 251
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`grep` 命中 3204–3205、3239、3241
- `crates/singlefs-harness/tests/second_transaction_parallel_line_three_many_inodes.rs`：`grep` 命中 28、373、391
- `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`：`grep` 命中 7、12、230
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`：`grep` 命中 404

### 13.4 实验装置与变异表

- `research/e7-index-bench/src/bin/e43_extension_point_budget.rs`：1–687（全文）
- `research/e7-index-bench/src/bin/e116_pack_settle.rs`：1–504（全文）
- `research/e7-index-bench/src/bin/e155_fsync_write_volume.rs`：18–30、74–78、166；`grep` 命中（`RECORD_HEADER_BYTES\|JOURNAL_HEADER\|307\|3789\|3785\|\b67\b\|NAMED_ENTRY\|RECORD_BYTES`、`HEADER\|NAMED_ITEM_BYTES`）
- `research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs`：36–48、86–90、177；同上 `grep`
- `research/e7-index-bench/src/bin/e155_third_run_release_cascade.rs`：28–38、58–63、152；同上 `grep`
- `research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs`：37–48、87–91、178；同上 `grep`
- `research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs`：36–46、55–80、296–315、495–512；`grep` 命中
- `research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs`：40–50、44–48、90–94、3062–3102（`print_anchors` 里带 `format!` 的行）、3085–3130、3230–3260；`grep` 命中
- `research/mutations/e43_extension_point_budget.tsv`、`e116_pack_settle.tsv`、`e157_parallel_line_one_clauses.tsv`：全文；`e159_fsync_wait_group_commit.tsv`：第一列全部；四张 E155 变异表：`grep -n 'RECORD_HEADER\|307\|NAMED_ITEMS_PER_RECORD\|JOURNAL_HEADER'` 命中行（第 1 次表第 12 行、第 2 次表第 12、29 行、第 4 次表第 4、5 行）与 `wc -l`

### 13.5 kb 实验页（为判问题单第 7 行、为找原判据；结果节没读，撞见的数已列进第四节）

- `.claude/kb/experiments/43-扩展点字节上限.md`：`grep -n '^#'`；2–74、97–112
- `.claude/kb/experiments/116-打包容器的账·补元数据写与整理策略.md`、`155-每次持久化的写量三种fsync形态与反事实上界.md`、`157-并行线一两条条款的计数模型.md`、`159-fsync等待时间随并发数组提交与wal两臂.md`：`grep -n '^#'`（标题行里有结论与变异账，第四节已列）；`159-…` 另有 `grep -n 'smoke\|冒烟\|anchors'` 命中的 5、14、15、32、46、61、62、64、68、73 行
- `.claude/kb/experiments/23-journal几何.md`：63–67
- `.claude/kb/experiments/39-反向链挡不挡得住残留记录.md`：`grep -n '^#'` 的前 20 行；`grep -n '判据'` 命中行；11–27、57–60
- `.claude/kb/experiments/42-一事务几条记录.md`：54–57
- `.claude/kb/experiments/49-反向链宽度32还是64.md`：1、7、80、145
- `.claude/kb/experiments/61-反向链hash算法的均匀性.md`：93–96
- `.claude/kb/experiments/75-记录尺寸与环几何.md`：`grep -n '^#'` 的前 10 行；24–40
- 查 `PACKED_UNIT_HEADER_BYTES` 出处时 `grep -rn` 撞见：`.claude/kb/experiments-history.md:578`、`.claude/kb/experiments/142-第一个事务的干跑.md:157`、`.claude/kb/decisions/18-块里携带什么信息.md:320`

### 13.6 跑过的命令（原样；读文件的 `sed` / `awk` / `grep -n` / Read 在上面按行列过，不重列）

```
$ git log --all --name-only --format='%h %ad' --date=short -- 'research/prompts/e43*' 'research/prompts/e116*' 'research/prompts/e155*' 'research/prompts/e157*' 'research/prompts/e159*'
$ git log --all --name-only --format='' | grep -iE '(^|/)e0?43[-_]|(^|/)e116[-_]|e155|e157|e159' | sort -u
$ git log --all --format='' --name-only | grep -E 'research/prompts/e[0-9]+-r[0-9]+-prereg' | sort -u
$ git log --all --diff-filter=D --format='%h' -- research/prompts/<四份 E155 登记>   # 得 3cff909、3cff909、8186d5b、8186d5b
$ git show 3cff909^:research/prompts/e155-preregistration.md > <草稿目录>/orig/e155-preregistration.md   # 另三份同形
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/quote-d23.md '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 4：记录头约束在单个原子单元内，头 311 字节' '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 17：点名项 56 字节的字段表'
  ✓ 2 段整抄进 …/quote-d23.md，回读逐字节一致
$ （问题单表头 5–6 行与第 1、7 / 2 / 3 / 4 / 5 行各抄一份，五次，都回读逐字节一致）
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/orig-e43.md '.claude/kb/experiments/43-扩展点字节上限.md@### 要测什么、判据、失败条款'
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/orig-e43-src.md 'research/e7-index-bench/src/bin/e43_extension_point_budget.rs:167-206'
$ nice -n 19 python3 research/scripts/quote-kb.py <草稿目录>/orig-e116.md 'research/e7-index-bench/src/bin/e116_pack_settle.rs:34-65'
$ grep -rn 'extension_point\|扩展点\|ExtensionPoint' crates/ --include=*.rs          # 2 行：system_configuration.rs:348、model.rs:251
$ grep -rn 'settle\|整理\|compaction\|搬迁\|repack' crates/ --include=*.rs           # 水位字段、恢复注释、测试注释，没有搬迁写路径
$ ls research/scripts | grep e159                                                    # 零命中
$ TZ=Asia/Tokyo date '+%Y-%m-%d %H:%M JST'                                          # 2026-09-24 20:47 JST（开写前）
```

`<草稿目录>` 是派发提示给的那个目录；命令二的两个脚本全文与原样输出在下面，执行员要复核时可以原样拷进 `research/` 下重跑。

#### 命令二：独立锚点（`anchors_h311.py`，全文）

```python
#!/usr/bin/env python3
# 头 311 重跑登记的独立锚点。常量从 crates/singlefs-format/src/lib.rs（156 / 165 / 168-169 / 172-173）
# 与 D23 已定项 4 / 17 手抄；不 import 任何装置，公式照各装置源码里那一行的算式手写。
from fractions import Fraction as F
from math import floor
REC, H_OLD, H_NEW, ITEM = 4096, 307, 311, 56
print("== 共用：一条记录装几个点名项")
for h in (H_OLD, H_NEW):
    print(f"  H={h}: (4096-H)={REC-h}  floor/56={(REC-h)//ITEM}  余={(REC-h)-((REC-h)//ITEM)*ITEM}  67*56+H={67*ITEM+h}  68*56+H={68*ITEM+h}")
hs67 = [h for h in range(0, REC) if (REC-h)//ITEM == 67]
print(f"  装 67 项的 H 区间：[{min(hs67)}, {max(hs67)}]；H={min(hs67)-1} 装 {(REC-min(hs67)+1)//ITEM} 项，H={max(hs67)+1} 装 {(REC-max(hs67)-1)//ITEM} 项")
print("== E43 自证单元档")
for h in (H_OLD, H_NEW):
    r512, r4096 = 512-h, 4096-h
    print(f"  H={h}: room512={r512} (点名项 {r512//ITEM})  room4096={r4096} (点名项 {r4096//ITEM})  占512={F(h,512)*100:.4f}%  bound=min(room512,255)={min(r512,255)}  864/bound={864/min(r512,255):.4f}")
side = [h for h in range(1, 512) if 512-h >= 255]
print(f"  绑定侧换成根槽 255 的 H：H <= {max(side)}")
for lb in (7, 9, 11, 13):
    ok = [h for h in range(1, 512) if min(512-h, 255) >= lb]
    print(f"  下界 {lb} 仍放得进 min(512-H,255) 的 H：H <= {max(ok)}")
ok128 = [h for h in range(1, 512) if min(512-h, 255) >= 128]
print(f"  D21 举例 128 仍放得进的 H：H <= {max(ok128)}")
# 挂载判定只在 N > 512-H（atomic 512）或 N > 4096-H（atomic 4096）处与 H 有关
for n in (0, 128, 512):
    for atomic in (512, 4096):
        flip = [h for h in range(0, atomic) if n > atomic - h]
        print(f"  mount N={n} atomic={atomic}: 自证溢出判拒的 H 从 {min(flip) if flip else '无'} 起")
print("== E116 journal 那一项（N = 100000，映射条目 61）")
N, UNIT, NODE, W = 100_000, 32768, 16384, 2
data_w = 1725 * UNIT * W
saved = N * UNIT - 1725 * UNIT
for h in (H_OLD, H_NEW):
    j = N * (h + 61)
    key = data_w + 375 * NODE * W + 113 * NODE * W + j
    fill1 = data_w + 100_000 * NODE * W + 101_725 * NODE * W + j
    filln = data_w + 375 * NODE * W + 226 * NODE * W + j
    print(f"  H={h}: journal_w={j}  key512.move_write={key}  key512.payback={key/saved:.6f}  fill512.b1.move_write={fill1}  fill512.b1.payback={fill1/saved:.6f}  fill512.bN.move_write={filln}")
base_key = data_w + 375 * NODE * W + 113 * NODE * W
base_fill1 = data_w + 100_000 * NODE * W + 101_725 * NODE * W
print(f"  saved={saved}")
print(f"  回本比 = 1 的 H*：key512 {F(saved - base_key, N) - 61}  fill512.b1 {F(saved - base_fill1, N) - 61}")
print("== E39 四档（行 7）")
for base in (303, 307):
    print(f"  base={base}: 档 {[base+k for k in (1,2,4,8)]}  增量 {[f'{k/base*100:.2f}%' for k in (1,2,4,8)]}")
print("== 十字段与增量拆分（D23 已定项 4 正文）")
print(f"  78+9+4+4+4+188+8+16={78+9+4+4+4+188+8+16}  78+17={78+17}  311-4(反向链)={311-4}")
```

```
$ cd <草稿目录> && nice -n 19 python3 anchors_h311.py
== 共用：一条记录装几个点名项
  H=307: (4096-H)=3789  floor/56=67  余=37  67*56+H=4059  68*56+H=4115
  H=311: (4096-H)=3785  floor/56=67  余=33  67*56+H=4063  68*56+H=4119
  装 67 项的 H 区间：[289, 344]；H=288 装 68 项，H=345 装 66 项
== E43 自证单元档
  H=307: room512=205 (点名项 3)  room4096=3789 (点名项 67)  占512=59.9609%  bound=min(room512,255)=205  864/bound=4.2146
  H=311: room512=201 (点名项 3)  room4096=3785 (点名项 67)  占512=60.7422%  bound=min(room512,255)=201  864/bound=4.2985
  绑定侧换成根槽 255 的 H：H <= 257
  下界 7 仍放得进 min(512-H,255) 的 H：H <= 505
  下界 9 仍放得进 min(512-H,255) 的 H：H <= 503
  下界 11 仍放得进 min(512-H,255) 的 H：H <= 501
  下界 13 仍放得进 min(512-H,255) 的 H：H <= 499
  D21 举例 128 仍放得进的 H：H <= 384
  mount N=0 atomic=512: 自证溢出判拒的 H 从 无 起
  mount N=0 atomic=4096: 自证溢出判拒的 H 从 无 起
  mount N=128 atomic=512: 自证溢出判拒的 H 从 385 起
  mount N=128 atomic=4096: 自证溢出判拒的 H 从 3969 起
  mount N=512 atomic=512: 自证溢出判拒的 H 从 1 起
  mount N=512 atomic=4096: 自证溢出判拒的 H 从 3585 起
== E116 journal 那一项（N = 100000，映射条目 61）
  H=307: journal_w=36800000  key512.move_write=165840384  key512.payback=0.051499  fill512.b1.move_write=6759974400  fill512.b1.payback=2.099192  fill512.bN.move_write=169543168
  H=311: journal_w=37200000  key512.move_write=166240384  key512.payback=0.051623  fill512.b1.move_write=6760374400  fill512.b1.payback=2.099316  fill512.bN.move_write=169943168
  saved=3220275200
  回本比 = 1 的 H*：key512 96410463/3125  fill512.b1 -4386249/125
== E39 四档（行 7）
  base=303: 档 [304, 305, 307, 311]  增量 ['0.33%', '0.66%', '1.32%', '2.64%']
  base=307: 档 [308, 309, 311, 315]  增量 ['0.33%', '0.65%', '1.30%', '2.61%']
== 十字段与增量拆分（D23 已定项 4 正文）
  78+9+4+4+4+188+8+16=311  78+17=95  311-4(反向链)=307
```

#### 命令二 b：E116 第八节敏感性两点（`anchors_h311_b.py`，全文）

```python
#!/usr/bin/env python3
# E116 第八节敏感性两点的回本比（公式照 e116_pack_settle.rs 第 156 行与 B2 手写，不 import 装置）
base_key = 1725 * 32768 * 2 + 375 * 16384 * 2 + 113 * 16384 * 2
saved = 100_000 * 32768 - 1725 * 32768
print("base_key(不含 journal) =", base_key, " saved =", saved)
for h in (307, 311, 30851, 30852):
    p = (base_key + 100_000 * (h + 61)) / saved
    print(f"  H={h}: payback={p:.7f}  ge1={int(p >= 1.0)}  ge1.0001={int(p >= 1.0001)}")
```

```
$ cd <草稿目录> && nice -n 19 python3 anchors_h311_b.py
base_key(不含 journal) = 129040384  saved = 3220275200
  H=307: payback=0.0514988  ge1=0  ge1.0001=0
  H=311: payback=0.0516230  ge1=0  ge1.0001=0
  H=30851: payback=0.9999892  ge1=0  ge1.0001=0
  H=30852: payback=1.0000202  ge1=1  ge1.0001=0
```

## 附：问题单第 7 行的逐句判定（不是重跑，不占第六节的判定格）

问题单第 7 行：「E23、E39、E42、E49、E61、E75 几份实验页引「现行头 307」的现状句」，候选「只改句子 / 要重跑」，翻面观测「那句话参与了该实验的判据……换成 311 之后判据要重算」。判法写死：**一句话里的 307（或由 307 推出的数）是该实验某一判据格的输入 ⇒「要重跑」；只是在引今天的头宽、该实验的判据格不用它 ⇒「只改句子」**。判据格以各页「判据」节或那句话自己写明的口径为准。句子是回扫报告第二节第 2 小节点名的那几行，逐句现读（行号是 2026-09-24 现查的）。

改句子时的数（第十三节命令二）：头 311 = 十个字段 78 + 三笔已定增量 17 + 本次发布内序号 4 + 新根段 188 + fsid 8 + MAC 16；512 上余 201（3 个点名项）、4096 上余 3785（67 个点名项）；不含反向链的头 = 311 − 4 = 307。**「三笔已定增量」仍是三笔、仍是 17**：D23 已定项 4 今天的原句把本次发布内序号单列一条，没有并进「已定增量」（回扫报告 E75 那一格写「三笔已定增量也要变四笔」，与 D23 原句不符，照 D23 改）。

| # | 位置（文件:行）与句中涉及头宽的片段（片段，不是整行；整行以原文件为准） | 它在说什么 | 判定 | 改成什么 / 为什么 |
|---|---|---|---|---|
| 1 | `.claude/kb/experiments/23-journal几何.md:65`：「……**现行头是 307 字节、512 上余 205**（3 个点名项）、4096 上余 3789（67 个点名项）。」 | 引今天的头宽与余量；E23 自己量的是那 10 个字段的旧头（同段第 63–64 行） | **只改句子** | 链上补「本次发布内序号 4」；307 → 311、205 → 201、3789 → 3785；「3 个」「67 个」不变 |
| 2 | `23-journal几何.md:66`：「……引用头部字节数时用 307（D23 已定项 4）。」 | 引用约定 | **只改句子** | 307 → 311 |
| 3 | `39-反向链挡不挡得住残留记录.md:59`：「……现行头不含反向链是 303……⇒ 四档是 304 / 305 / 307 / 311 字节、+0.33% / +0.66% / +1.32% / +2.64%。**宽度选择不变**——选的依据是误接受那两列，与头宽无关。」 | 表里末两列（头宽与涨幅）的现行换算。E39 的判据节（第 11–27 行）三条判据是有效性、不误杀、宽度（误接受率为 0 的最小宽度），都不用头宽 | **只改句子，但不是替换一个数**：四档要按不含反向链的头 307 重新枚举 | 「现行头不含反向链是 307（头 311 里含反向链 4）⇒ 四档是 308 / 309 / 311 / 315 字节、+0.33% / +0.65% / +1.30% / +2.61%」（命令二 E39 那两行）；「宽度选择不变」那半句不动 |
| 4 | `42-一事务几条记录.md:56`：「（正文里那个 84 是 2026-08-29 跑时的头宽；现行 307（十个基础字段 78 加后定的增量）……差额与头宽无关，结论不动。）」 | 引今天的头宽；句子自己写明差额与头宽无关 | **只改句子** | 307 → 311 |
| 5 | `49-反向链宽度32还是64.md:1、7、80、145`：「……现行不含反向链的记录头是 303……代价侧要按 303 重算，重算之前不许把「两侧都免费」当结论引用」 | E49 代价侧的判据是「代价在 base = X 这一档为零」，base 就是不含反向链的头宽；这四句说的正是「今天的 base 不在 E49 那张表里、代价侧要按今天的 base 重算」 | **参与判据，要重跑**（另列，不在这一次） | 四句里的 303 → 307（头 311 里另含反向链 4），「按 303 重算」→「按 307 重算」。⚠️ 这笔重算**本来就欠着**（`.claude/kb/checks-owed.md` C200，头 307 时就没重算），不是 311 这一次新造的；回扫报告还指出 C200 的还账计划写死的「base = 78 与 base = 91」已不对应任何现行口径，要人重定 |
| 6 | `61-反向链hash算法的均匀性.md:95`：「三档头宽取自 2026-08-31 跑时的 86 / 95 / 99；现行头是 **307**（……十个字段 78 + 三笔已定增量 17 + 新根段 188 + fsid 8 + MAC 16）……源码与产物停在旧档，引用前先复跑。」 | 引今天的头宽与分项；E61 的判定格是 86 / 95 / 99 三档，今天的头宽不是其中一档 | **只改句子** | 307 → 311，分项加「本次发布内序号 4」。⚠️ 句子自己写着「源码与产物停在旧档，引用前先复跑」：E61 在今天的头宽上从没量过，307 时也没有，这笔账与这一次改动无关，照旧挂着 |
| 7 | `75-记录尺寸与环几何.md:34`：「……三笔已定增量把它抬到 **95**（现行头 307 另含新根段 188、fsid 8 与 MAC 16，D23 已定项 4）。」 | 括号里引今天的头宽；E75 判据 1 的容量表四个口径是 78 / 91 / 95 / 99（第 36 行起那张表的表头），今天的头宽不是其中一个 | **只改句子** | 「现行头 311 另含本次发布内序号 4、新根段 188、fsid 8 与 MAC 16」；「三笔已定增量把它抬到 95」不动。E75 标题自己写着「它引的头部口径是旧的」，那是已有的状态，不是这一次造成的 |

**要重跑的另列**：只有第 5 行 E49 的代价侧（base 改按 307）。它不在这一次跑；照 C200 的现状，还账计划本身要先由人重定 base 口径。

**没判的**：回扫报告同一小节里另外几处（`.claude/kb/experiments.md:70` E49 索引行的「现行记录头 78」、E91 的 `RECORD_HEADER_BYTES = 78`、E143 的 `JOURNAL_HEADER_BYTES_TODAY = 95`）不在问题单第 7 行的名单里，这一次没读也没判。
