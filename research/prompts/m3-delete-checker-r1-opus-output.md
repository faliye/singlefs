# m3-delete-checker-r1 攻方腿（Opus）报告：K2 / K3（2026-10-05）

攻击面：正文第三节 K2、K3。被判对象：F1–F5 每个候选。冻结副本 `/tmp/claude-1000/m3-delete-checker-r1/tree/`（与仓里 `crates/` 被引的 13 个文件逐字节相同，引代码的行号两边一样）。副本上的数只在副本上量过，不算入库装置的数。

## 复跑

模型目录 `research/prompts/m3-delete-checker-r1-opus-model/`，在空目录里起（`/tmp/claude-1000/m3-delete-checker-r1-opus/rerun-empty/` 已从空目录跑通一次，汇总与 `expected-summary.txt` 逐字相同，sha256 4bcce6cf…30b7）：

```
mkdir 空目录 && cd 空目录
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON=… RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON=…
bash <仓>/scripts/run-with-memory-cap.sh 5G bash <仓>/scripts/capped.sh 10 \
  bash <仓>/research/prompts/m3-delete-checker-r1-opus-model/rerun.sh all     # 约 6 分钟；也可按 prepare / legal <批> / random / mut <批> / extra / summary 分段起
```

```
fe3c9b0b1c6d42dae563999e7c912a690e5d3d3508fbc715c69eb31c34a383fa  copy.patch            副本对冻结副本的全部改动（含原型用例）
4bcce6cf2d6e04ea638f633f24b060eb5f1d3c9a88a1b5145785d7a8c17c30b7  expected-summary.txt  汇总的原样输出
aeeadd1c943a4db0deecd066404fbb698304dbbcd36308a327e6324a776a72f5  m3dc_candidates.rs    原型用例（harness 档，名字不带 layer0）
a6a061797c7fccf6c0d966dbd1ea4f0a970c48f0fa3bed5c3c355bdc0d0facd7  rerun.sh
bffa55182fdcee9d1279b37ee0d597ee19857b36b2a51da474835998f3e131ab  summarize.py
```

装置是什么：副本里 `singlefs-harness` 的历史执行器跑一段历史，每一步之后拿镜像，原型按字节自己读根环里每一条读得出的根（树表、inode 树、`deleted_inodes`、extent 树每个 inode 的单元数、记账标签 4、logged_ops 根、墓碑记录），把每个候选的判法在每一版上判一次；同一镜像另跑一次池级 checker `check_pool_image`。副本改动三样（都只由环境变量开）：产品批大小可压小（`M3DC_BATCH`，模型那一侧同步）；核心层 11 种改坏（`M3DC_MUT`）；执行器在模型对不上的那一步也把镜像交给观察者。

取样（用户决定的几步放开扫）：
- 内容（文件单元数）：批 144 取 Empty（size 0、1 个单元）、Small（3000 字节以内一个单元）、Exact（恰一个净荷）、145、290 个单元；批 1、2、3、5 取 Empty、Small、Exact、2、3、4、7 个单元。
- 批大小：1、2、3、5、144。
- 尾巴：无；冷启动；挂载（续做）；建 inode→挂载→建 inode；建 inode 之后回退到环里第 k 新的根（k = 1..12），再接 {无、挂载续做、再删、覆盖写、建 inode、冷启动再挂载、挂载再删} 之一。
- 另加随机历史：快档比重 `GenerationWeights::BROAD`、种子 1000..1599、每段 30 步，留下带 unlink 的 176 段。
- 合计合法历史 3080 段，全部 `Completed`；判过的版本 199 660 个（最新根 23 045、候选集里别的根 169 314、F 之下 7 301）。

崩溃状态没枚举（照定义第 3b 条「一段历史自己就超过约 10⁶ 的，那段不跑」）：每段历史的覆盖写那一步与 unlink 那一步，第一段就有 24 次以上的写。原型只数状态数（`M3DC_COUNT_ONLY=1`）的原样输出：

```
M3DC_PROGRESS Small/none op0:Some(PublishOverwrite)->… writes=29 segments=[24, 2, 1, 2] closed_form=16777223
M3DC_PROGRESS Small/none op1:Some(Unlink)->… writes=56 segments=[26, 2, 1, 2, 20, 2, 1, 2] closed_form=68157453
M3DC_SUMMARY plan=legal batch=1 mutation="" histories=616 crash=true states=24855468569362 record_applied_states=0 elapsed_seconds=22.9
```

批 144 的计划数状态数时 panic：145 个单元以上的那一步一段写超过 63 次，`crates/singlefs-harness/src/memory_pool.rs:614`：「.map(|segment| (1u64 << segment.len()) - 1)」左移溢出（状态数超过 2⁶⁴）。合法计划里 ≤ 10⁶ 的步只有挂载那几步（327 696、393 238 个状态），所在的历史整段都超过 10⁶，按定义整段不跑。这条腿跑的崩溃状态：0 个。

## 各格判定一览

| 格 | 判定 | 依据（详见对应小节） |
|---|---|---|
| F1 A（nlink = 0 的记录） | 合法历史上没打中误红；**打中漏判**两种：孤儿单元（号有单元、没有记录、不在队列）与僵尸记录（nlink = 0、单元全放、不在队列） | 量过：改坏 orphan、zombie |
| F1 B（号有单元而没有活记录） | 「活记录」有两个读法 B1（nlink ≥ 1）/ B2（有记录），判法不同；B2 在合法镜像上恒不适用，漏 noqueue；B1、B2 都漏僵尸 | 量过：改坏 noqueue、zombie |
| F1 丙（A ∪ B） | **与 B1 是同一个候选**：每张镜像上判法相同（证明 + 全部取样逐格相同）；漏僵尸 | 量过 + 推的 |
| F1 三个候选共有 | **打中不分辨臂**：僵尸记录满足 I-4.7 字面的「或其 extent 已全部释放」，三个候选与现有 checker 全放过；病根在定义字面，不在读法 | 量过：zombie（Empty 内容时零候选红） |
| F2 甲 | 合法历史 0 误红；条目宽那一半的判别力落在 I-1.10（改坏 qwidth16：I-1.10 同红）；logged_ops 那一半只红自己，但判的是「有没有 logged_ops 意图」，不是「增量字段」 | 量过：qwidth16、loggedops |
| F2 丙 | 与 F3 乙1 是同一个判法（正文表里自己写了重合）；两条一起实现时两条同红，都做不到只红自己；连带乙1 的误红 | 量过（同乙1） |
| F2 乙 | 不判镜像，没有可攻的镜像；交用户（跑前条款） | — |
| F3 甲 | 「只红自己」按定义做不到：甲红 ⇔ 另有一条红 | 推的（逻辑） |
| F3 乙1 | **打中误红**：空的第一个文件（size 0、1 个数据单元）unlink 的意图那一版，⌈0 ÷ 32634⌉ = 0 ≠ 1 | 量过：2449 个版本红，全是这一形 |
| F3 乙2 | **打中误红**，比乙1 宽：不删也红，任何一版只要第一个文件是空内容 | 量过：9258 个版本红 |
| F3 丙 | 连带乙的误红；乙那一半能只红自己（stale 改坏） | 量过 |
| F4 a | 只判最新根：放过 size 与标签 4 这两类「批中途坏、最后一批恢复」的中间态（非崩溃历史里 0 次红）；I-9.5、I-4.7 那几类坏态因为续做由队列驱动而停在最新根，只判最新根也抓得到 | 量过：stale、tag4 / noqueue、nlink1、keepq |
| F4 b | 两个配对法在第一版每张镜像上同一个（树表里每种树至多一条）；配不上时 I-9.27 已红 | 推的（代码） |
| F4 c | 两种报法在层 0 判红上同形；只在坏盘战役的「checker 一条都没判」那一格分叉 | 推的（代码） |
| F5 a | 「三句都落」做不到只红自己：第 1 句落在 I-3.1 / I-3.11、第 3 句落在 I-5.2 | 推的（代码）；量过 checker 那三条会红（allocbump） |
| F5 b | 「缺行不适用」放过缺行（改坏 notag4row：别的条与准入都看不见，核心层冷启动与挂载拒）；「缺行判红」在 199 660 个合法版本上 0 误红 | 量过 |
| F5 c | 同 F4 a：标签 4 写坏只在中间版时，只判最新根在非崩溃历史里 0 次红 | 量过：tag4 |
| I-9.5（正文没列候选，K2 第 4 问） | nlink ≠ 0 那一半与 I-9.19 / I-9.18 必有一条同红（这两条也归三·3 实现），只红自己只剩「队列里的号没有记录」一形，且要 I-9.18 对没有记录的号报判不了 | 量过：nlink1、keepq；推的（抽屉论证） |

## F1：I-4.7「处于删除中的 inode」指谁

定义原文 `.claude/kb/invariants.md:183`：「任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在」。原型里四个读法的判法（`m3dc_candidates.rs` 的 `candidate_verdicts`）：

- A：每条 nlink = 0 的记录，号在队列里或单元数为 0。
- B1：每个「单元数 > 0 而没有 nlink ≥ 1 的记录」的号，在队列里。B2：每个「单元数 > 0 而没有记录」的号，在队列里。（B 的字面「inode 树里没有活记录」，「活」读成 nlink ≥ 1 是 B1，读成「有记录」是 B2。）
- 丙：A 的集合 ∪ B1 的集合，判法同上。

合法历史上：四个读法 0 次红（I-4.7/A 判过 26 167 个有 nlink = 0 记录的版本，B2 在 199 660 个合法版本上全是不适用——合法镜像上它的对象集合恒空）。

### 打中 1（漏判，三个候选共有）：僵尸记录

历史（改坏 zombie：最后一批照删队列那条 key、照放单元，只是不删 inode 记录；副本 `transaction.rs` 里 `remove_record_from_leaf_containers` 那一处换成照抄叶容器）：起点 AfterFirstFile → 覆盖写 Empty（size 0）→ unlink。每一步许可它的那一句：
1. 覆盖写空内容写出 size 0、1 个数据单元：`crates/singlefs-core/src/write_request_split.rs:61`：「content_length_in_bytes.div_ceil(payload_capacity).max(1)」。
2. 意图：nlink = 0、size 不变、队列加 1（`.claude/kb/decisions/08-核心索引结构.md:464`：「意图那次发布把 inode N 的记录写成 nlink = 0（size 不变）」）。
3. 最后一批被改坏成不删记录：本该照 `.claude/kb/decisions/08-核心索引结构.md:465`：「最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布」。

原样输出（`mut-zombie-b144.log`）：

```
M3DC_LINE Empty/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-4.7/丁零轮"] | txg=6 F=0 trees=true rec={1: (0, 0, 3)} q=[] qw=None units={} tag4=Value(0) lo0=true tomb=1
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-4.7/丁零轮", "I-4.6/乙1", "I-4.6/乙2", "I-4.6/乙1z零轮", "I-4.6/乙2z零轮"] | txg=6 F=0 trees=true rec={1: (0, 3000, 3)} q=[] qw=None units={} tag4=Value(0) lo0=true tomb=1
```

这一版：记录 nlink = 0、队列空、单元全放。读者答「没有文件」，续做看队列、队列空就什么都不做（`crates/singlefs-core/src/transaction.rs:6004`：「let Some(inode) = current.deleted_inodes.first().copied() else {」），这条记录永远不回收。A、B1、B2、丙全绿，现有 checker 0 条红（26 段历史）。Empty 内容那一形，F3 的乙1 也绿（size 0、0 个单元，⌈0 ÷ 32634⌉ = 0），五个候选里只有本腿零轮的「丁」红。

四问：
- 分不分辨臂：不分辨 F1 的三个候选（A、B、丙一起放过）——**打中不分辨臂**。共用的前提是定义字面的析取「或其 extent 已全部释放」：僵尸满足它。
- 系统看不看得到：看得到，记录、队列、单元都在同一版里。
- 满足的是字面哪一句：不违反 I-4.7 的任一分句；它违反的是 `.claude/kb/decisions/08-核心索引结构.md:466`：「它是派生态：队列 = inode 树里 nlink = 0 的号集合」的反方向（nlink = 0 ⇒ 在队列）。I-9.5 只管正方向（队列 ⇒ nlink = 0），五条里没有一条管反方向。
- 改法在这几格还中不中：A / B / 丙 换哪个都不中；要中得改字面（下面「改法表」的丁、戊），或把反方向另立一条。

核心层对第一个文件这一形另有拦法：重建上一版时单元数对 size 核（`crates/singlefs-core/src/recovery.rs:4572`：「!= data_unit_count_of_a_sequential_write(inode_record.size)」），僵尸 0 个单元、size 0 也要 1 个，挂载拒。这是挂载那一道，不是池级 checker；对建出来的 inode（size 0、0 个单元）这一道不拦（推的：那一判只看 inode 1）。

### 打中 2（漏判，A 独有）：孤儿单元

历史（改坏 orphan：最后一批删记录、删 key，但不截 extent、不放单元；副本里还要关掉发布路径的断言 `crates/singlefs-core/src/transaction.rs:7838`：「"没有第一个文件那条记录的一版没有数据单元：最后一批同一次发布释放剩下的全部单元"」，不关就 panic）：AfterFirstFile → 覆盖写 Small → unlink。原样输出（`mut-orphan-b144.log`）：

```
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-4.7/B1", "I-4.7/B2", "I-4.7/丙"] | txg=6 F=0 trees=true rec={} q=[] qw=None units={1: 1} tag4=Value(0) lo0=true tomb=1
```

A 绿（没有记录，A 的对象集合空）；现有 checker 0 条红：I-9.10 只对记录还在的对象判（`crates/singlefs-checker/src/walk.rs:8369`：「if let Some(record_birth) = walk.inode_object_birth.get(inode) {」），单元照算在已分配里，I-3.1 平。核心层挂载时拒（`crates/singlefs-core/src/recovery.rs:2693`：「inode 树里没有第一个文件那条记录，extent 树里却有它的数据单元」）。

四问：分辨臂（A 漏、B1 / B2 / 丙中）；看得到；满足的是字面「要么在删除中队列里有条目，要么其 extent 已全部释放」两者皆非——前提是把「没有记录、有单元的号」算作「处于删除中」，这正是 A 与 B 的分歧；改法：换 B1 或丙即中。⚠️ 今天的实现到不了这一形（要先关掉上面那条断言），它是「实现坏了 checker 能不能看见」的镜像，不是今天能跑出来的历史。

### 打中 3（漏判，B2 独有）：意图漏写队列

历史（改坏 noqueue：意图照写 nlink = 0、照追墓碑，只是不加队列那条 key）：AfterFirstFile → 覆盖写 Small → unlink。会话走空看队列、队列空就停，unlink 照样报做成。原样输出（`mut-noqueue-b144.log`）：

```
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-4.7/A", "I-4.7/B1", "I-4.7/丙", "I-4.7/丁零轮"] | txg=5 F=0 trees=true rec={1: (0, 3000, 3)} q=[] qw=None units={1: 1} tag4=Value(0) lo0=true tomb=1
```

B2 绿（记录还在）。四问：分辨臂（B2 漏，其余中）；看得到；满足字面「两者皆非」；改法：换 A / B1 / 丙。这一形只红 I-4.7（Small 内容时乙1 不红、现有 checker 0 条红）——A、B1、丙都能用它当「只红自己」的坏镜像。

### 两个候选实为同一个：丙 ≡ B1

证明：A 集合里单元数为 0 的号，判法「在队列里或单元数为 0」恒真；A 集合里单元数 > 0 的号，没有 nlink ≥ 1 的记录（它的记录就是 nlink = 0），已经在 B1 集合里。所以「A ∪ B1 上判」与「B1 上判」逐号同真假。实测：3080 段合法历史与 11 种改坏的每一版上，丙与 B1 的判定逐格相同（`expected-summary.txt` 里两行计数一样）。B 读成 B2 时丙 = A ∪ B2，判法与 B1 也相同（同一证明，B1 \ B2 = 单元数 > 0 的 nlink = 0 记录 ⊆ A）。所以 F1 实际只有三个不同的判法：A、B1（= 丙）、B2。

## F3：I-4.6 判什么

定义原文 `.claude/kb/invariants.md:182`：「存在未完成意图的镜像上，其余全部不变量仍然成立」。

### 打中 4（误红，乙1、乙2、F2 丙）：空的第一个文件

历史：AfterFirstFile → 覆盖写 Empty → unlink（批大小任一）。每一步许可它的那一句：
1. 空内容照样是一个数据单元：`crates/singlefs-core/src/write_request_split.rs:56`：「长度为 0 时是 1」；空内容在随机历史的抽法里有比重（`crates/singlefs-harness/src/history.rs:954`：「(ContentLength::Empty, 10),」）。
2. 意图 size 不变：`.claude/kb/decisions/08-核心索引结构.md:464`：「意图那次发布把 inode N 的记录写成 nlink = 0（size 不变）」。
3. 一个单元不多于一批，意图之后直接是最后一批（`crates/singlefs-core/src/unlink.rs:128` 那一支走不到），所以中间态只有意图那一版：nlink = 0、size 0、1 个单元。

原样输出（`legal2-b144.log`）：

```
M3DC_LINE Empty/none op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-older>=F RED=["I-4.6/乙1", "I-4.6/乙2"] | txg=5 F=0 trees=true rec={1: (0, 0, 3)} q=[1] qw=Some(8) units={1: 1} tag4=Value(0) lo0=true tomb=1
M3DC_LINE Empty/inode-rb2- op3:Some(RollBackWhileMounted)->Some("Applied(RolledBack { target: RollbackTarget { instance: Inst") step-newest RED=["I-4.6/乙1", "I-4.6/乙2"] | txg=8 F=0 trees=true rec={1: (0, 0, 3)} q=[1] qw=Some(8) units={1: 1} tag4=Value(0) lo0=true tomb=1
```

第二行是「删 → 建 inode → 回退到意图那一版」：回退新根取回退目标的用户可见树（`crates/singlefs-core/src/mount.rs:6101`：「/// 回退那次发布接在后面的「上一版」：用户可见的那一半」），误红落在最新根上，只判最新根也躲不开。合计：乙1 在 2449 个合法版本上红（最新根 80、候选集里别的根 2334、F 之下 35），逐个核过全是 `rec={1: (0, 0)} q=[1] units={1: 1}` 这一形；乙2 另在活着的空文件上红（不删也红），合计 9258 个版本：

```
M3DC_LINE Empty/none op0:Some(PublishOverwrite)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-4.6/乙2"] | txg=4 F=0 trees=true rec={1: (1, 0, 3)} q=[] qw=None units={1: 1} tag4=Value(0) lo0=true tomb=0
```

现有 checker 在这些镜像上 0 条红；核心层自己的读法是「单元数 = max(1, ⌈size ÷ 净荷⌉)」（同一个 `div_ceil(...).max(1)`，重建时 `crates/singlefs-core/src/recovery.rs:4572` 拿它核），乙的字面 ⌈size ÷ 净荷容量⌉ 与实现自己的读法差在 size 0 这一点。

四问：
- 分不分辨臂：分辨（乙1、乙2、F2 丙中；甲与本腿的乙1z 不中）。
- 系统看不看得到：看得到。
- 满足的是字面哪一句：乙1「nlink = 0 的记录：数据单元数 = ⌈size ÷ 净荷容量⌉」逐字；乙2「每条记录都判这一式」逐字。
- 改法还中不中：换成核心层的 max(1, ·) 读法在 inode 1 上不中；但建出来的 inode 是 size 0、0 个单元，被 unlink 时意图那一版 nlink = 0、0 个单元，max(1, 0) = 1 ≠ 0 又误红（推的：历史执行器的 unlink 只删 inode 1，这一形没跑到；`publish_unlink_step` 对别的 inode 照走意图 + 最后一批，路径在）。本腿的乙1z（size 0 时允许 0 或 1 个单元）在 199 660 个合法版本上 0 误红（量过）。

### 甲、丙与「只红自己」

- 甲（其余各条的合取）：甲红 ⇔ 有未完成意图 ∧ 另有一条红。按定义它的每一次红都与另一条同红，`.claude/kb/milestone/03-third-txn.md:85` 的「每条只红自己；`crates/mutations.tsv` 各一行」对甲做不到；判别力全部落在被合取的那几条上（推的，逻辑）。
- 乙那一半能只红自己。改坏 stale（批照截单元、不改 size）：中间版 size 停在原值，乙1 / 乙2 / 乙1z / 乙2z 红、现有 checker 0 条红、F1 各候选 0 条红（量过，批 1 / 2 / 3 / 144 合计 23 个版本）。所以「丙 = 甲 + 乙」能有只红自己的坏镜像，来自乙那一半。
- F2 丙（落到 nlink = 0 记录的字段上）与 F3 乙1 是同一个判法：两条都这样实现时，stale 镜像上两条同红，两条都做不到只红自己。

## F2：I-4.4 在第一版判什么

### 打中 5（甲的条目宽那一半做不到只红自己）

改坏 qwidth16：队列条目写成 16 字节（号 + 「还剩几个单元」，就是定义要禁的那种增量字段），批 1 / 2 / 3 / 144、7 种内容各一段。原样输出（`mut-qwidth16-b2.log`）：

```
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") CHECKER-RED I-1.10
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-older>=F RED=["I-4.4/甲"] | txg=5 F=0 trees=true rec={1: (0, 65272, 3)} q=[1] qw=Some(16) units={1: 3} tag4=Value(0) lo0=true tomb=1
```

26 段里 I-1.10 每段都红：池级 checker 已经给 `deleted_inodes` 登记了条目字段表宽 8（`crates/singlefs-checker/src/walk.rs:213`：「TREE_KIND_DELETED_INODES => Some(DELETED_INODES_ENTRY_BYTES),」），每一版都判（`crates/singlefs-checker/src/walk.rs:1912`：「.judge("I-1.10", entry_width == entry_field_table_bytes, || {」）。甲的这一半的判别力落在 I-1.10。理想模型这一形照样对得上（26 段都 Completed）。

### 打中 6（甲的 logged_ops 那一半只红自己，但判的不是 I-4.4）

改坏 loggedops：树表里 logged_ops 那条的根指针指向墓碑树根。原样输出（`mut-loggedops-b144.log`）：

```
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-4.4/甲"] | txg=6 F=0 trees=true rec={} q=[] qw=None units={} tag4=Value(0) lo0=false tomb=1
```

只有甲红：池级 checker 遇到种类 12 直接返回（`crates/singlefs-checker/src/walk.rs:1816`：「let Some(schema) = key_schema_for_tree_kind(kind) else {」，`crates/singlefs-checker/src/lib.rs:602`：「_ => None,」）。但它判的是「有没有 logged_ops 条目」，第一版 unlink 本来就不写 logged_ops（`.claude/kb/decisions/08-核心索引结构.md:108`：「第一版 unlink 不往 logged_ops 树写条目」），整理落地那天第一个写者一来就误红（同一行：「字段表、回退时取哪一版、态别留到第一个写者（整理」）——推的，今天的 crates 里没有写者。

四问（打中 5、6 合起来）：分辨臂（甲 vs 乙：乙不判镜像）；看得到；满足的是甲字面的「条目宽 = 8」与「根指针全零」；改法：甲的条目宽一半换成「I-4.4 在第一版由 I-1.10 代判」不改判法、只改登记，logged_ops 一半要么留着并写明整理落地时撤，要么报不适用。

判别子观测不到（推的，举例）：「字段只描述目标状态、不含增量」要分清一个字段的语义。批之后 size = 剩下单元数 × 净荷容量（`.claude/kb/decisions/08-核心索引结构.md:465`：「每批释放尾部 K 个单元、size 改成「剩下单元数 × 净荷容量」」），这个数同时就是「还剩多少要删」；一个按增量写、写出来字节相同的实现，单张镜像上与它逐字节相同。I-4.4 在单张镜像上能判的只有结构代理（条目宽、有没有 logged_ops），幂等的真判据在 D8 已定项 16 第 7 条（两份盘逐字节相同，要两张镜像）。

## F4：三件共用的事

### a 判哪几版——打中 7（只判最新根放过「中途坏、最后一批恢复」的中间态）

改坏 tag4（意图与每个不是最后一批的删除批把标签 4 写成还没放的单元数 × 32768，最后一批照写 0）与 stale（见 F3）。历史：AfterFirstFile → 覆盖写 3 个单元 → unlink（批大小 2）。原样输出（`mut-tag4-b2.log`、`mut-stale-b2.log`）：

```
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-older>=F RED=["I-3.4/缺行判红", "I-3.4/缺行不适用"] | txg=5 F=0 trees=true rec={1: (0, 65272, 3)} q=[1] qw=Some(8) units={1: 3} tag4=Value(98304) lo0=true tomb=1
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-older>=F RED=["I-3.4/缺行判红", "I-3.4/缺行不适用"] | txg=6 F=0 trees=true rec={1: (0, 32634, 3)} q=[1] qw=Some(8) units={1: 1} tag4=Value(32768) lo0=true tomb=1
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=[] | txg=7 F=0 trees=true rec={} q=[] qw=None units={} tag4=Value(0) lo0=true tomb=1
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-older>=F RED=["I-4.6/乙1", "I-4.6/乙2", "I-4.6/乙1z零轮", "I-4.6/乙2z零轮"] | txg=6 F=0 trees=true rec={1: (0, 65272, 3)} q=[1] qw=Some(8) units={1: 1} tag4=Value(0) lo0=true tomb=1
M3DC_LINE Span(3)/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=[] | txg=7 F=0 trees=true rec={} q=[] qw=None units={} tag4=Value(0) lo0=true tomb=1
```

汇总（`expected-summary.txt`）：tag4 在 26 段历史里最新根红 0 次、别的根红 49 次；stale 去掉空文件那 4 次误红后，最新根红 0 次、别的根红 23 次。unlink 一次调用发完意图与全部批，非崩溃历史里每一步之后的最新根就是最后一批那一版，坏的中间版只在环里当回退候选。现有 checker 的记账只取最新根那一遍（`crates/singlefs-checker/src/walk.rs:7997`：「walk.walk_root(&roots[newest_index].2.record_bytes, true);」之后 `crates/singlefs-checker/src/walk.rs:8026`：「let accounting = walk.accounting.clone();」），标签 4 照这个读法只判最新根，就是这一格的「只最新根」。

对比：I-9.5、I-4.7 那几类坏态（noqueue、nlink1、keepq）在非崩溃历史里都停在最新根——续做由队列驱动（`crates/singlefs-core/src/transaction.rs:5919`：「.find(|record| record.inode == inode.0 && record.nlink == NLINK_OF_AN_INODE_BEING_DELETED)」、`crates/singlefs-core/src/transaction.rs:5921`：「.filter(|_| previous.deleted_inodes.contains(&inode))」），坏了就走不下去，最新根 26 / 26 次红。回退把用户可见树搬到新根（`crates/singlefs-checker/src/walk.rs:386`：「[TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_DELETED_INODES];」），size 那一类回退之后上了最新根；标签 4 不随回退搬（记账由发布路径现算），只能靠判每一版。

四问：分辨臂（「只最新根」漏、「每一版」中）；看得到；满足的是 I-3.4「记账标签 4 … 恒 0」、乙1 的式子；改法：换「每一版」即中。另一条约束：I-9.24 自己写了「每个被判的版本各判一次，不遍历版本之间的历史」（`.claude/kb/invariants.md:311`），同批实现的几条取不同的版本集合，同一张镜像上会出现「I-9.24 在旧版上红、I-9.5 只看最新根」这类不对称。

注意（量过）：判「环里每一条读得出的根」而不是候选集，会碰到 F 之下已被回收的根。合法历史里 7 301 个 F 之下的版本里 1 157 个读不全；第一版原型把读不全的版照样判，B1 / B2 / 丙在这些版上红了 28 次（全是 inode 树读不出、extent 树读得出）。修成「读不全就全部不适用」之后 0 次。所以 a 取「回退候选集」时，c 那一格必须是「判不了」而不是红，不然 B 读法在 F 之下误红。

### b 怎么配对（推的，代码）

第一版每张树表里每种树至多一条：核心层重建拒「同一种树有两条」的树表，`deleted_inodes` 条目的头 ID 等于这张表里唯一那条 inode 树条目的树 ID，不等时 I-9.27 已红。所以「经头 ID 配对、配不上记判不了」与「同一张树表里的 inode 树」在 I-9.27 成立的每张镜像上是同一个配对；两者只在 I-9.27 已红的镜像上分开。原型用的是后者（同一张树表），合法历史 0 次配不上。

### c 读不全怎么报（推的，代码）

层 0 把两种报法算成同一格：`crates/singlefs-checker-tier/src/crash.rs:1493`：「InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {」。唯一分叉在坏盘战役：`crates/singlefs-checker-tier/src/bad_disk_input.rs:1321`：「InvariantVerdict::NotApplicable(_) => true,」只把不适用算进「checker 一条都没判」，部分没判不算，而那一格会让这次坏盘算作看见了，见 `crates/singlefs-checker-tier/src/bad_disk_input.rs:506`：「|| self.checker_judged_nothing」。删除这几条报部分没判时，一张别的条全不适用、只有它们部分判了的坏盘镜像会从看见了翻成没看见。这一形今天没造出来（推的）。

## F5：I-3.4

定义原文 `.claude/kb/invariants.md:148`，三句：待删单元计在已分配里（释放只进 defer）；「记账标签 4（待删除但意图未完成占用）恒 0」；可用空间因此已扣除待删占用。

### a 落哪几句（推的，代码；checker 会红这一半量过）

第 1 句的坏法（待删单元提前从已分配里扣、或提前进 defer）由 I-3.1（已分配 = 遍历全部候选根的引用）与 I-3.11（已分配 − defer = 最新根走读）判：删除中的单元仍被最新根引用。第 3 句由 I-5.2（空闲 + 已分配 = 容量）判。三句都落时这两句的坏镜像至少同红两条，「只红自己」做不到。checker 这三条在原型装置上会红：改坏 allocbump（每版已分配多记一个槽）上 I-3.1、I-3.11、I-5.2 各红 78 次（四个批大小合计）。「提前进 defer」的改坏本腿没造（限度）。

### b 标签 4 那一行缺了——打中 8（「缺行不适用」放过）

改坏 notag4row（每次发布的记账不写标签 4 那一行）。原样输出（`mut-notag4row-b144.log`、`extra-notag4row-mount.log`）：

```
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Applied(Published { reuse: RecordReuse { rewritten_from_rele") step-newest RED=["I-3.4/缺行判红"] | txg=6 F=0 trees=true rec={} q=[] qw=None units={} tag4=Missing lo0=true tomb=1
M3DC_LINE Empty/cold ENDING "NewFinding { signature: HarnessJudgement { judgement: \"checker 判绿的镜像上冷启动恢复报错\" }, observation: FailureObservation { position: Operation(2), operation_kind
M3DC_LINE Empty/mount ENDING "NewFinding { signature: ModelDisagreement { aspect: \"模型说该成、实现拒了\" }, observation: FailureObservation { position: Operation(2), operation_kind: Some(CloseAndMou
```

现有 checker 0 条红；准入不读盘上那一行（读常量：`crates/singlefs-core/src/admission.rs:511`：「pending_delete: PENDING_DELETE_OF_THE_FIRST_VERSION,」）；核心层冷启动与挂载拒（行数核：`crates/singlefs-core/src/recovery.rs:4374`：「detail: "记账条目数不是 3 + 6 × 盘数",」），执行器自己判出「checker 判绿的镜像上冷启动恢复报错」。「缺行不适用」时池级 checker 在一张挂不上的镜像上全绿；「缺行判红」在 199 660 个合法版本上 0 次误红。

四问：分辨臂（缺行不适用漏、缺行判红中）；看得到；满足的是「标签 4 … 恒 0」——「恒 0」要有那一行才说得上；改法：取「缺行判红」即中。

### c 判哪几版

同 F4 a 打中 7：tag4 改坏只在中间版写坏时，只判最新根在非崩溃历史里 0 次红、判每一版 49 次红（量过）。

## I-9.5 能不能只红自己（K2 第 4 问；正文 F1–F5 没列 I-9.5 的候选）

定义 `.claude/kb/invariants.md:292`。它红只有两种：队列里的号 N 有记录而 nlink ≠ 0；或 N 没有记录。三·3 同批要实现的 I-9.18（`.claude/kb/invariants.md:305`）、I-9.19（`.claude/kb/invariants.md:306`）：
- N 有记录、nlink ≠ 0：谱系上有 (N, N 的对象出生代) 的墓碑 ⇒ I-9.19 红；没有 ⇒ I-9.18 红。两种必居其一（推的，抽屉论证）。
- N 没有记录：I-9.18 要的「N 的 inode 记录偏移 8 的对象出生代」不存在，I-9.18 报红还是判不了由它的实现定。

量过（不按谱系过滤的近似写法，`mut-nlink1-b144.log`、`mut-keepq-b144.log`）：

```
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Refused { member: \"PublishError::UnlinkOfAnInodeThatIsNotALi") step-newest RED=["I-9.5", "I-9.19近似"] | txg=5 F=0 trees=true rec={1: (1, 3000, 3)} q=[1] qw=Some(8) units={1: 1} tag4=Value(0) lo0=true tomb=1
M3DC_LINE Small/unlink op1:Some(Unlink)->Some("Refused { member: \"PublishError::UnlinkOfAnInodeThatIsNotALi") step-newest RED=["I-9.5", "I-9.18近似"] | txg=6 F=0 trees=true rec={} q=[1] qw=Some(8) units={} tag4=Value(0) lo0=true tomb=1
```

所以 I-9.5 的坏镜像要「只红自己」，只能取「队列里的号没有记录」，并且 I-9.18 对没有记录的号报判不了。这一行的近似写法不按谱系过滤，在合法回退历史上误红（I-9.19 近似在最新根上红 1616 次，全在带回退的历史里），所以上面两行只当「同红」的证据，不当 I-9.18 / I-9.19 的判法。

## 改法表（每一格：量过 = 本腿副本上的原样输出在上面各节；推的 = 按代码推、没实现没跑）

改法都只在本腿的模型上量过、被攻过零轮。「丁」= I-4.7 去掉析取（每条 nlink = 0 的记录，号在队列里）；「戊」= 丁 ∧ B1；「乙1z」= nlink = 0 的记录，单元数 = ⌈size ÷ 净荷⌉，或 size 0 而单元数 ≤ 1；「乙2z」= 乙1z 对每条记录判。

| 打中的格 | 丁 | 戊 | 乙1z / 乙2z | 每一版（F4 a） | 缺行判红（F5 b） |
|---|---|---|---|---|---|
| 乙1 / 乙2 空文件误红 | 不相干 | 不相干 | 修：0 误红（量过，199 660 个版本） | 不修（误红也在最新根上，量过） | 不相干 |
| 僵尸记录（A / B / 丙 都放过） | 修（量过，26 / 26） | 修（推的：戊含丁） | Small 内容修、Empty 内容不修（量过：22 / 26） | 不修（僵尸在最新根上，量过） | 不相干 |
| 孤儿单元（A 放过） | 不修（量过：丁 0 次红） | 修（推的：戊含 B1，B1 量过 26 / 26） | 不修（量过） | 不修 | 不相干 |
| 意图漏写队列（B2 放过） | 修（量过） | 修（推的） | 不修 | 不修 | 不相干 |
| stale / tag4 只在中间版（只最新根放过） | 不相干 | 不相干 | stale 修、tag4 不修（量过） | 修（量过：23 / 49 次红） | 不相干 |
| 缺标签 4 那一行 | 不相干 | 不相干 | 不相干 | 不修 | 修（量过：最新根 78 次红、合法 0 误红） |
| I-9.5 与 I-9.18 / I-9.19 同红 | 不相干 | 不相干 | 不相干 | 不修 | 不相干；要修得定 I-9.18 对没有记录的号报判不了（推的） |

## K3 岔路单（每处取不下来的一行；候选定义、翻面观测、够判条件、代价）

| 处 | 候选（各自的定义） | 翻面观测 | 够判条件 | 代价 |
|---|---|---|---|---|
| F1 I-4.7 的对象 | A：nlink = 0 的记录；B1（= 丙）：单元数 > 0 而没有 nlink ≥ 1 的记录；B2：单元数 > 0 而没有记录；丁（零轮）：A 去掉「或其 extent 已全部释放」；戊（零轮）：丁 ∧ B1 | ① 孤儿单元要不要归池级 checker 管（今天发布路径断言与挂载重建各拦一道；要 checker 独立看见 ⇒ B1 / 戊）；② 僵尸记录要不要有一条不变量管（要 ⇒ 丁 / 戊，同时 I-4.7 字面或另立一条要改） | 用户答 ①② 即够；不需再量（两形本腿副本上已量） | A、丁：逐版读记录偏移 36 的 nlink，一个字段（推的）；B1、B2、戊：另要逐版、逐号的单元数，今天 extent 走读跨版共享已走过的单元、不按版存（推的，没量过）；丁、戊要改 `invariants.md` 那一行 |
| F2 I-4.4 | 甲：队列条目宽 = 8 ∧ logged_ops 根全零；乙：第一版报不适用、改出口措辞；丙：落到 nlink = 0 记录字段（= F3 乙1） | 用户接不接受「I-4.4 在第一版的判别力 = I-1.10 + logged_ops 是否为空」（量过：条目宽一半与 I-1.10 同红） | 用户定 | 甲：一个判（推的）、整理落地时 logged_ops 那一半要撤；乙：改出口 1 与状态列；丙：与 F3 乙1 同红，取丙则 F3 不能再取乙 |
| F3 I-4.6 | 甲：其余各条合取；乙1 / 乙2：⌈size ÷ 净荷⌉ 字面；乙1z / 乙2z（零轮）：size 0 时容 0 或 1 个单元；丙：甲 + 乙 | 字面乙1 / 乙2 在空文件上误红（量过）；甲做不到只红自己（逻辑） | 用户在「乙1z（或乙2z）」与「甲 + 验收不要求它只红自己」之间定 | 乙1z：同 F1 B 的逐版单元数；乙2z：逐版每条记录；甲：零读盘、但没有自己的变异行 |
| F4 a 判哪几版 | 只最新根；回退候选集每一版加记录施加出来的几版 | 量过：中途坏、最后一批恢复的坏态，只最新根在非崩溃历史里 0 次红；I-9.5 / I-4.7 类坏态停在最新根 | 定三·4 的层 0 流是否让每个删除中间版都当一次最新根（是 ⇒ 只最新根在 checker 档够；否 ⇒ 每一版） | 每一版：记账已逐版走（I-3.13），标签 4 逐版不多读盘（推的）；inode / 队列 / 单元数要按版存（推的，没量过）；层 0 那一路单步 ≥ 16 777 223 个状态（量过的闭式） |
| F5 a | 只落「标签 4 恒 0」；三句都落 | 第 1、3 句的坏镜像同红 I-3.1 / I-3.11 / I-5.2（推的；这三条会红量过） | 用户接不接受「第 1、3 句由 I-3.1 / I-5.2 代判」 | 只落标签 4：一个判（推的） |

F4 b、F4 c、F5 b、F5 c 不列：b 两个配对法在第一版是同一个；c 两种报法层 0 同形，坏盘战役那一格取「不适用」不改今天的「看见了」计数；F5 b「缺行判红」0 误红、「缺行不适用」放过一张挂不上的镜像；F5 c 同 F4 a。

## 没打中的形状

- 合法历史上没误红的：I-9.5、I-4.7 的 A / B1 / B2 / 丙 / 丁、乙1z / 乙2z、I-4.4 甲、I-3.4 两种、I-9.24、I-9.18 近似——199 660 个版本（3080 段历史）0 次红；现有池级 checker 在这些镜像上 0 条红。试过的形状：文件 1 / 2 / 3 / 4 / 7 / 145 / 290 个单元与空文件，批大小 1 / 2 / 3 / 5 / 144，unlink 之后冷启动、挂载续做、建 inode 再挂载，建 inode 之后回退到环里第 1–12 新的根（落在删之前、意图、各批、删完各版），回退之后挂载续做、再删、覆盖写（会话先走空再拒）、建 inode、冷启动再挂载、挂载再删；随机历史 176 段（unlink 做成 155 次、回退做成 328 次、抬 F 做成 259 次）。
- 没跑到的：崩溃状态（0 个，理由见「复跑」一节）；记录施加出来、还没有根的那一版（推的：它的树与同一次发布的根相同，I-8.10 判这一条）；被崩溃恢复抛弃的根（随机历史里那一步 127 次全被拒）；unlink 建出来的 inode（执行器只删 inode 1）；会话走空被空间拒、队列留着的那一形；小盘几何。

## 这条腿自己的限度

- 一次抽样；「没打中」照规则要再抽一次才算。
- 原型读事实时用了核心层的解析函数（`parse_index_node`、`read_extent_tree`、`parse_inode_internal_entry` 等），不经它们的判定，但与实现共用解析代码。
- 版本集合：判环里每一条读得出的根，按 txg ≥ 最新根的 F 标「候选集」；没按实例表分抛弃的根（本腿的历史里没有被抛弃的根）。
- I-9.18 / I-9.19 是不按谱系过滤的近似，只用来看「同红」，不当判法。
- 改坏都是本腿自己写的；孤儿单元要先关掉发布路径的断言才造得出。

## 没做什么

- 不判 K1；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑任何 checker 档测试、层 0、崩溃放量；只跑了副本里 `cargo test -p singlefs-harness --test m3dc_candidates`（经内存包装、线程上限 10）。
- 草稿：`/tmp/claude-1000/m3-delete-checker-r1-opus/out/`、`/tmp/claude-1000/m3-delete-checker-r1-opus/out2/`（原样输出，上面引的每一行都在 `out2/` 的日志里），`/tmp/claude-1000/m3-delete-checker-r1-opus/rerun-empty/out/`（空目录复跑的输出）。仓副本与编译目录交回前删掉。
