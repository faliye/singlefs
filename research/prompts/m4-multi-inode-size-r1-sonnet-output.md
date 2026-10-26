# m4-multi-inode-size-r1 正推腿报告（云端正推，Sonnet，2026-10-26）

立场：从截断两轮判决、D8 已定项 5 与 14 和冻结副本 `crates/` 今天的实现，推正文第三节 S2–S7 每一格的候选、要满足的条件、推荐与依赖的前提；S7 给清单。不判攻方腿的格，不替主 agent 出判决。读过：背景材料正文与小节清单（附录 1.5 MB 没整读，kb 条款一律回 kb 文件现查）；禁读清单里攻方腿的产出没读。

## 0 复跑与产物

- 引代码写冻结副本里的路径与行号。1.2 节的 `cmp` 是开工时跑的（被引的 22 个文件里 20 个相同）；写完报告前再跑一次，被引文件里与冻结副本不同的有三个：`crates/singlefs-checker/src/walk.rs`（现行仓多约 20 行，行号整体后移）、`crates/mutations.tsv`（5684 行之后不同）、`crates/singlefs-core/src/mounted_session.rs`（另一会话正在改，现行仓比冻结副本多约 500 行）。这三个文件我写成「冻结副本 文件名 第 N 行」，不用路径加冒号的写法，免得 cite-check 拿现行仓去判；mutations.tsv 引的三行在 4456、4462、4469，两边相同，照用路径加冒号。
- 自己量的数：一份探针测试（3 条），在草稿目录里的副本上跑（只拷冻结副本的 `singlefs-core`、`singlefs-format`、`singlefs-replay-prefix` 三个包，探针放进 `singlefs-core/tests/`），经 `capped.sh 31` 与 `run-with-memory-cap.sh` 跑，没起别的 cargo。没给模型目录，所以不进 `research/prompts/…-sonnet-model/`；探针与输出留在草稿目录，原样输出贴在 2.3、3.2、4.2 节。
  - 草稿目录：`~/.local/share/singlefs/work/m4-multi-inode-size-r1-sonnet/`
  - `probe_multi_inode_size.rs`（152 行）sha256 `988aca960a5cfd15393c1c4dc3f974fd93522132ba967fa5b936754335be64fb`
  - `rerun.sh` sha256 `5333766dde402e6173cf685129fa2b2e84ec30339b84c03ac64e8c19f1f4ad5a`
  - `probe-final-out.txt` sha256 `84de6a5ec0de1231516c351e9e33b2401cff19f7d772ab6db2b5a3e93e00757f`（输出里 panic 行带线程号，每次不同，复跑时 sha256 会变，逐行内容不变）
  - `evidence.txt`（1.2 节的命令与输出）
  - 复跑：`bash rerun.sh ~/.local/share/singlefs/work/m4-close-rounds-frozen <不存在的新目录> probe_multi_inode_size.rs`（脚本里已写好声明、线程上限 31、内存包装 8G）。
- 为什么不入库：探针只回答这一轮的几个读码疑问（计划函数在别的文件上的结局、读侧矩阵、回退后条目数），不是装置；要不要拷进 `research/prompts/` 由主 agent 定。草稿副本与编译目录交回前删。

## 1 对正文第二节「实现今天的样子」逐条核

### 1.1 逐条

- 拒绝入口：「let is_the_first_file = inode.0 == FIRST_INODE_NUMBER;」（`crates/singlefs-core/src/transaction.rs:10483`）一致；成员定义「SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses { inode: u64 },」（`crates/singlefs-core/src/transaction.rs:5555`）一致。
- 续做只认第一个文件：「let names_the_live_first_file = intent.inode.0 == FIRST_INODE_NUMBER」（`crates/singlefs-core/src/transaction.rs:10604`）一致；同一函数里取单元数也是第一个文件的：「let data_units_existing = u64::try_from(previous.data_pointers.len()).expect("单元数");」（`crates/singlefs-core/src/transaction.rs:10616`），材料没写这一条。
- 别的文件不许有洞：「截断与变长对它拒」（`crates/singlefs-core/src/transaction.rs:3671`）一致；冷恢复那一处拒绝是 5939–5946 行（材料写 5939–5945，差一行）：「|| extents.tail() != TailOfAFile::NoHoleAfterTheLastDataUnit」（`crates/singlefs-core/src/recovery.rs:5940`）。
- unlink 已泛化：取单元数的写法一致，行号是 10398–10404（材料写 10391–10400，偏 7 行）：「let data_units_remaining = if inode.0 == FIRST_INODE_NUMBER {」（`crates/singlefs-core/src/transaction.rs:10398`）。
- 池级 checker 按每条记录判：一致。材料点名的 12804 行是 I-4.6 与 I-4.10 两个判的调用点；判函数本身在冻结副本 walk.rs 第 13016 行（`judge_data_unit_counts_of_the_inodes_against_their_size`）、第 13106 行（`judge_data_units_beyond_the_size_against_the_truncation_intents`）、第 13138 行（`judge_declared_lengths_of_the_data_units_within_the_size`）、第 18100 行（`judge_truncation_intent_against_its_record`），都按记录的 inode 号取，不看号是几。
- harness 模型有按 inode 号写：一致（`crates/singlefs-harness/src/model.rs:1121` 「pub struct ModelInodeOtherThanTheFirst {」）；但模型对截断的答法有材料没写的两个边界，见 5.1 节。
- RefFS 只有第一个文件：一致，而且更彻底：RefFS 整份源码里「inode」一词只出现一行注释（1.2 节输出），它没有文件身份这个概念。
- 「`mutations.tsv` 里锚在拒绝成员与 `is_the_first_file` 上的 3 行」：**冲突**。实测三行（4456、4462、4469）锚在拒绝成员的名字上，落在 harness 的 `model.rs`、`model_comparison.rs`、`history.rs`；锚在 core 的 `is_the_first_file` 上的变异行零条，全表 `is_the_first_file` 只命中 4779 一行，那是 GPU 恢复内核的 `is_the_first_file`（1.2 节输出）。也就是说 core 里这道门（10483–10493）今天没有变异行钉着。
- 「`grep -rn FIRST_INODE_NUMBER crates/singlefs-core/src` 今天 61 行」：一致，分解见 1.2 节：非测试 45、测试 16；非测试里 3 行不是用法（`unlink.rs:22` 与 `recovery.rs:112` 是导入，`transaction.rs:178` 是定义），用法 42 行。

### 1.2 命令与原样输出

```
$ cd 冻结副本; grep -rn FIRST_INODE_NUMBER crates/singlefs-core/src | wc -l
61
$ 非测试 / 测试 行数（测试模块起点：transaction.rs 15283、unlink.rs 333、truncate.rs 315、recovery.rs 8095；各文件第一个 #[cfg(test)] 行）
合计 61 非测试 45 测试 16
transaction.rs 非测试 37 [178, 260, 263, 331, 336, 4007, 4034, 4549, 5852, 5859, 7131, 7152, 7316, 7835, 7855, 7958, 7964, 7993, 8006, 8933, 9049, 9106, 9168, 9279, 9330, 10398, 10483, 10604, 10820, 12179, 12204, 12375, 12378, 13275, 13465, 13497, 13597] 测试 4
recovery.rs 非测试 5 [112, 3805, 5930, 7287, 7468] 测试 0
unlink.rs 非测试 3 [22, 259, 290] 测试 8
truncate.rs 非测试 0 [] 测试 4
$ cmp 被引文件（冻结副本对现行仓）
相同 crates/singlefs-core/src/transaction.rs
相同 crates/singlefs-core/src/recovery.rs
相同 crates/singlefs-core/src/truncate.rs
相同 crates/singlefs-core/src/unlink.rs
相同 crates/singlefs-core/src/extent_tree.rs
相同 crates/singlefs-core/src/logged_operations.rs
相同 crates/singlefs-core/src/mount.rs
相同 crates/singlefs-core/src/mounted_session.rs
相同 crates/singlefs-core/src/mounted_read.rs
相同 crates/singlefs-core/src/write_request_split.rs
相同 crates/singlefs-harness/src/model.rs
相同 crates/singlefs-harness/src/model_comparison.rs
相同 crates/singlefs-harness/src/history.rs
相同 crates/singlefs-harness/src/reference_filesystem_translation.rs
相同 crates/singlefs-harness/src/verification_read.rs
相同 crates/singlefs-checker-tier/src/crash_judge_common.wgsl
相同 crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
相同 crates/singlefs-reference-filesystem/src/lib.rs
相同 crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs
相同 crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs
不同 crates/singlefs-checker/src/walk.rs
不同 crates/mutations.tsv
$ 三行变异（锚在拒绝成员名上）与 is_the_first_file 的行
4456:m4 截断截丁 第一个文件之外的 inode 改 size
4462:m4 截断截丁 胶水：删完之后点名别的 inode
4469:m4 截断截丁 执行器：拒绝的成员名不分第
4779:m4 验证段 GPU 分批读：恢复内核 extent 树那
$ 钉住「别的文件无单元/有洞要拒」的变异行（recovery.rs 与 GPU 恢复内核三处）
0
$ RefFS 里 inode 一词
110:    /// 还没有文件时截断：实现在任何写之前拒（判决 `res
$ 截断条目的签发点
crates/singlefs-core/src/transaction.rs:5820:    /// 截断条目由同一次发布的 [`LoggedOperationsStepO
crates/singlefs-core/src/transaction.rs:5974:    IssueTheTruncationIntent(TruncationIntent),
crates/singlefs-core/src/transaction.rs:6022:            LoggedOperationsStepOfThePublish::IssueTheTruncationI
crates/singlefs-core/src/transaction.rs:10557:                LoggedOperationsStepOfThePublish::IssueTheTrunca
$ 模型里四处「别的文件带数据就不答」
5754:        Self::refuse_to_answer_with_a_file_other_than_the_first_holding_data_units(
5800:        Self::refuse_to_answer_with_a_file_other_than_the_first_holding_data_units(
5935:        Self::refuse_to_answer_with_a_file_other_than_the_first_holding_data_units(
6132:        Self::refuse_to_answer_with_a_file_other_than_the_first_holding_data_units(
6406:    fn refuse_to_answer_with_a_file_other_than_the_first_holding_data_units(
$ 测试里钉「别的文件无单元要拒」的文案
crates/singlefs-core/src/recovery.rs:5944:                what: "extent 树里第一个文件之外的文件没有单元
```

## 2 S2　别的文件的洞怎么表示

### 2.1 条款与实现今天怎么说

- 条款只写了第一个文件。标签 0 的写法：「标签 0 全零：第一个文件一个数据单元都没有」（`.claude/kb/decisions/08-核心索引结构.md:451`）；写者义务：「现有写路径自己写不出来」（`.claude/kb/decisions/08-核心索引结构.md:465`）；两条用户定案：「用户定案 2026-10-22：标签 0 配 size > 0 的第一个文件，写者接得住这一形」（`.claude/kb/decisions/08-核心索引结构.md:477`）、「用户定案 2026-10-24：第一个文件没有数据单元时 size 0 也写标签 0」（`.claude/kb/decisions/08-核心索引结构.md:478`）。别的 inode 条款没写；代码注释替它说了现状：「号 ≥ 2 的空 inode（`transaction::publish_new_inodes` 建出来、一次都没写过内容的）」（`crates/singlefs-core/src/extent_tree.rs:412`）接着「不经这里，照旧没有条目、读者按「没有条目」收」（`crates/singlefs-core/src/extent_tree.rs:413`）。
- 读侧是同一个函数，不看 inode 号：「DataUnitsOfTheInode::NoneWritten => refused,」（`crates/singlefs-core/src/write_request_split.rs:106`）；挂载态读者按条目有无分流，对任何 inode 一样：「None => DataUnitsOfTheInode::NoneWritten,」（`crates/singlefs-core/src/mounted_read.rs:809`）。
- 冷走读、可写重挂与冷恢复、只读快照三处读者共用 `extents_of_the_first_file`：「extent_tree = Some((entry.tree, extents_of_the_first_file(read)?));」（`crates/singlefs-core/src/recovery.rs:7088`，冷走读）、「let first_file_extents = extents_of_the_first_file(extent_tree_read)?;」（`crates/singlefs-core/src/recovery.rs:3333`，重挂与冷恢复）、「let extents = extents_of_the_first_file(read_extent_tree(」（`crates/singlefs-core/src/recovery.rs:7401`，快照）。别的文件在这个函数里被拒的条件是 5939–5940（零单元、尾部有洞），拒绝文案「按 inode 号写数据写不出来，第一版不支持」（`crates/singlefs-core/src/recovery.rs:5944`）；通过之后单元数对 size 的核对还把别的文件的尾部写死成没有洞：「TailOfAFile::NoHoleAfterTheLastDataUnit,」（`crates/singlefs-core/src/recovery.rs:3853`）。
- 写者侧装不下零单元的文件：「只写过建 inode（size 0）的 inode 没有数据单元，不在这里」（`crates/singlefs-core/src/transaction.rs:3181`），别的文件在上段叶里那一条只会出标签 1、2：「let target = match (file_lower_nodes.last(), file_data_pointers.first()) {」（`crates/singlefs-core/src/transaction.rs:12317`），零单元直接 panic：「第一个文件之外的文件只在有数据单元时才在计划里」（`crates/singlefs-core/src/transaction.rs:12323`）。
- 池级 checker：I-4.6 把「没有上段条目」与「标签 0」放在同一支，只要求零单元、不看 size（冻结副本 walk.rs 第 13057 行 `(None | Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (`）；上段规范形 I-9.40 从条目自己的号集合判形状（冻结副本 walk.rs 第 12343 行），所以别的文件多一条标签 0 条目它不会红。GPU 恢复内核有两处和 CPU 一样写死「别的文件」：「let has_no_hole_after_its_last_data_unit = tag == 1u || tag == 2u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3387`）、「let tail_is_a_hole = is_the_first_file && (tag == 0u || tag == 3u || tag == 4u);」（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:432`）。
- 变异表里钉这几处「别的文件无单元、有洞要拒」的行零条（1.2 节输出）；测试里也只有 `recovery.rs:5944` 自己带那句文案（1.2 节输出）。

### 2.2 候选

- **粘住的标签 0**：别的 inode 变长到 size > 0 而一个单元都没有时，上段写它的标签 0 条目；条目此后一直留着（缩短、缩到 0、再变长都不动），到 unlink 最后一批才去掉。`publish_new_inodes` 建出来的 size 0 inode 仍不写条目，字节不变。
- **按单元数与 size 现算的标签 0**：条目有无 = （单元数 ≥ 1 或 size > 0）。零单元的文件缩到 size 0 时去掉条目，size 0 的从没写过的变长时加上。
- **不声明**：变长只改记录，读者把「没有条目而 size > 0」放宽成整个文件是洞。
- **先落补零单元**：变长时发现一个单元都没有，先落单元 0（声明长度 min(N, 净荷)，补零），之后用标签 3 或 4 声明洞，标签 0 不出现在别的文件上。

### 2.3 量：读侧矩阵（探针 `probe`，同一个读侧函数，原样）

```
读侧 无条目 size0 0单元: Ok(0)
读侧 无条目 size100 0单元: Err(DataUnitCountDoesNotMatchTheFileSize { data_units_implied_by_the_file_size: 1 })
读侧 标签0 size100 0单元: Ok(0)
读侧 标签0 size0 0单元: Ok(0)
读侧 标签1/2 size100 1单元: Ok(1)
读侧 标签3/4 size100 1单元: Err(DataUnitCountDoesNotMatchTheFileSize { data_units_implied_by_the_file_size: 1 })
号 2 标签 0 条目 113 字节，来回解析相等: true
of_a_file_without_data_units(Some(0)) = Some(NoDataUnit); (None) = None
```

读法：读者对「没有条目 + size 100」拒、对「标签 0 + size 100」与「标签 0 + size 0」都收；size 0 的文件有没有条目，读出来的内容一样（都是空）。号 2 的标签 0 条目字节格式今天就解得开。

### 2.4 要满足的条件

- 三处读者与挂载态读者对同一镜像同一个答案。今天读侧函数与挂载态读者已经是任意 inode 的；要改的只有 `extents_of_the_first_file` 对别的文件的那两道门（5939–5940）与 3853 写死的尾部。
- 写者对读进来的每一形不丢：D8 已定项 14 对第一个文件已经立了这条义务（465 行），先例是 2026-10-22 调查发现的「标签 0 镜像上重写上段叶的发布丢掉标签 0 条目，写出的那一版三处读者都拒」（`.claude/kb/decisions/08-核心索引结构.md:477`）。别的文件的零单元条目今天在内存里没有位置（`transaction.rs:3181`），重写上段叶的任何发布都会把它丢掉，读回来就是读侧矩阵第二行的拒。
- 标签由「这一版的单元数与 size」现取，别的文件也一样：unlink 的批把 size 改成「剩下单元数 × 净荷」（`.claude/kb/decisions/08-核心索引结构.md:505`），带洞的别的文件被 unlink 的批截过之后标签要从 3、4 换回 1、2，今天构造别的文件条目的地方（`transaction.rs:12317`）只会出 1、2。
- 判别力：条目有无不得成为 checker 看不出的自由度。读者对「没有条目 + size > 0」拒而 I-4.6 绿，是现成的缺口（读侧矩阵第二行对冻结副本 walk.rs 第 13057 行）；别的文件变长之后这个缺口会被写者踩到，所以要加一条 checker 判：别的文件「没有上段条目 ⇒ size 0」。这是**新判**。
- 别的文件的合法形（推荐写进 I-4.6 的别的文件那一支）：标签 0 ⇒ 零单元；标签 1、2、3、4 ⇒ 至少一个单元、从 0 起连号；标签 1、2 ⇒ 单元数不少于 ⌈size ÷ 净荷⌉.max(1)（截断中间态可以多）；标签 3、4 ⇒ 少于它。这与今天对第一个文件的判一字不差，等于把「第一个文件」三个字去掉。
- 建 inode 路径字节不变（`extent_tree.rs:412` 的现状）。
- 既有坏镜像不多红（截断判决 S3 的口径：「照字面读成「没有记录就不认领」会让既有坏镜像多红 I-1.13、破坏「只红自己」」，`research/prompts/m4-trunc-r1-main-verification.md:85`）：读者放宽只会把过去的拒变成收，不新增红；checker 现有判本来按记录判，别的文件多出标签 0、3、4 它们今天就是绿的；新加的那一判只红「没有条目而 size > 0 的别的文件」，这一形没有任何写者写得出，手造镜像上读者本来就拒（读侧矩阵第二行），所以它让 checker 追上读者而不改任何一个挂载结局（推的，没跑镜像）。
- GPU 恢复内核与 CPU 同判：上面两处 wgsl 去掉「第一个文件」限定，与 CPU 的对拍用例（checker 档，归主 agent 跑）补别的文件有洞、标签 0 的世界。

### 2.5 推荐与依赖的前提

推荐「粘住的标签 0」。理由：
- 与已定的两条用户定案落在同一个字节形上，不引入新标签、新读法；
- 不需要改 size 的发布去新建数据单元（「先落补零单元」要在改 size 的发布里写一个今天没有的新单元，而 K5 明写「变长不立意图、一次发布做完……之后到 N 都是洞、不写单元」，`research/prompts/m4-trunc-r1-main-verification.md:39`）；
- 「不声明」丢判别力：截断判决 W5 已经量过「只改 size 不重写尾单元，今天读者拒、池级 checker 一条不红」，用户据此定了「加标签声明洞」（`research/prompts/m4-trunc-r1-main-verification.md:24`）；
- 「现算」与「粘住」对读者等价（读侧矩阵第三、四行），现算多出两个上段形状转移（缩到 0 去条目、从 0 变长加条目），换不来任何 checker 判别力。

依赖的前提：
- 把用户 2026-10-16「加标签声明洞」与 2026-10-24「一律写标签 0」推到别的文件。条款没写别的文件；落地前 D8 已定项 14 的 451、465 两处和 `.claude/kb/layout/04-fourth-txn.md:26` 那一行要改成不限第一个文件（阶段同步），否则代码与条款是两句话。
- 读侧矩阵第一、四行：size 0 的别的文件有没有条目对读者同价，所以「粘住」不会让「缩到 0 的零单元文件」变成读者拒。
- 我没 grep `.claude/gate.d/` 下的钉表：如果有钉表要求「别的文件的上段条目只在有数据单元时出现」，要一起改。

建议随这一格落的必红世界（变异行与用例名交实现员取，这里只写世界）：别的文件零单元变长后重写上段叶丢掉条目；别的文件标签 0 配 size 0 的单元数多于零；标签 3、4 配单元数不少于 size 要的个数；unlink 的批之后标签没从 3、4 换回 1、2；GPU 恢复内核仍把别的文件的洞当坏。

### 2.6 什么现象会推翻

- 找到一份已入库的钉表、用例或条款要求「别的文件的上段条目只在有数据单元时出现」（我没查钉表）；
- 读侧矩阵第三、四行在入库装置上复跑不是 `Ok(0)`；
- 用户不认「别的文件可以有标签 0」，选「先落补零单元」；那时 2.4 节除"标签 0"之外的条件原样成立，只是写者多一条新建单元的路径。

## 3 S3　截断续做与截断计划对别的文件

### 3.1 今天哪几处把截断写死在第一个文件上

- 取单元数：unlink 已经按 inode 取（「let data_units_remaining = if inode.0 == FIRST_INODE_NUMBER {」，`crates/singlefs-core/src/transaction.rs:10398`）。截断路径上同一件事有三处没泛化：「let data_units_existing = if is_the_first_file {」（`crates/singlefs-core/src/transaction.rs:10494`，别的文件取 0）、「let data_units_existing = u64::try_from(previous.data_pointers.len()).expect("单元数");」（`crates/singlefs-core/src/transaction.rs:10616`）、「let data_units_before = u64::try_from(previous_version.data_pointers.len()).expect("单元数");」（`crates/singlefs-core/src/transaction.rs:8943`）。
- 步骤类型没有 inode：「TruncationBatch {」（`crates/singlefs-core/src/transaction.rs:5826`）只带 `data_units_kept`，`data_units_after_this_publish` 把它当第一个文件的：「DeletionStepOfThePublish::TruncationBatch { data_units_kept }」（`crates/singlefs-core/src/transaction.rs:5855`）。
- 计划函数的下段那一半只读第一个文件的下段：调用处写死「FIRST_INODE_NUMBER,」（`crates/singlefs-core/src/transaction.rs:7993`），改 size 前后的样子取自「let record_before = previous_version.inode_record?;」（`crates/singlefs-core/src/transaction.rs:8924`）。
- 别的文件的计划物种没有截断：「pub enum ChangeOfAFileOtherThanTheFirst」（`crates/singlefs-core/src/transaction.rs:7247`）只有整份写、删除的批、删除的最后一批；选择在 7949–7978。
- 尾单元重写的四样写死第一个文件：角色「slot_of(TransactionUnit::Data(rewrite.unit)),」（`crates/singlefs-core/src/transaction.rs:13485`）、单元头里的对象「object: FIRST_INODE_NUMBER,」（`crates/singlefs-core/src/transaction.rs:13465`）、旧净荷来源「let bytes = &previous.unit(TransactionUnit::Data(rewrite.unit)).bytes;」（`crates/singlefs-core/src/transaction.rs:13189`）、映射 key 按「(self.carried_data_units..previous.data_pointers.len())」（`crates/singlefs-core/src/transaction.rs:7752`）与 8538 行起那一段取第一个文件的单元。
- 取号之前的预演只走 `deleted_inodes`：「let Some(inode) = version_rehearsed.deleted_inodes.first().copied() else {」（`crates/singlefs-core/src/transaction.rs:10666`），截断条目和销毁中的快照不在预演里。第一个文件今天就这样，推广不改变它，不属于这一行（记作没动的前提）。

### 3.2 量：别的文件上，删除批计划与改 size 的计划在哪几步一样、哪几步不一样（探针，原样）

文件上一版 300 个单元（下段叶 0 罩 0–143，叶 1 罩 144–287，叶 2 罩 288–299，根在层级 1），号 2。

```
删除批计划 kept=144（不带重写单元）: L0#0=carried
截断意图计划（改 size 的计划，重写单元 143，单元数 300 不变）: L0#0=rewritten L0#1=carried L0#2=carried L1#0=rewritten
删除批计划 kept=288（不带重写单元）: L0#0=carried L0#1=carried L1#0=rewritten
截断意图计划（改 size 的计划，重写单元 287，单元数 300 不变）: L0#0=carried L0#1=rewritten L0#2=carried L1#0=rewritten
删除批计划 kept=150（不带重写单元）: L0#0=carried L0#1=rewritten L1#0=rewritten
截断意图计划（改 size 的计划，重写单元 149，单元数 300 不变）: L0#0=carried L0#1=rewritten L0#2=carried L1#0=rewritten
thread 'probe' (714960) panicked at crates/singlefs-core/src/unlink.rs:191:5:
删除的一批留下至少一个单元、至少截掉一个（留下 0 个是最后一批）
删除批计划 before=144 kept=144（最后一批放 0 个）: panic（断言：留下至少一个、至少截掉一个）
不带重写单元的批（单元数 300 截到 11 个取值）：两份计划的下段节点与来历相同 11 个，不同的 kept = []
```

读法：
- 不带重写单元的批，截断的批与 unlink 的批在 kept 取 1、2、100、143、144、145、150、287、288、289、299 共 11 个值上给出逐节点相同的下段计划，所以截断的批可以直接用 `plan_the_lower_segment_after_a_deletion_batch`（`crates/singlefs-core/src/unlink.rs:191` 的断言要求留下至少一个、至少截掉一个，截断的批满足）。
- 截断意图那次要重写尾单元 k，罩着 k 的下段叶到根恒重写：kept=144 时 k=143，删除批计划把叶 0 照抄、改 size 的计划重写叶 0 与根；kept=288 时叶 1 同理。这就是 K9 的叶边界（「harness 层另加：144 叶边界的计划纯函数用例（留下 144、288 个单元时罩着单元 k 的叶被重写）」，`research/prompts/m4-trunc-r1-main-verification.md:43`）搬到别的文件上的样子：**拿 unlink 的计划去做意图那一步，会漏重写**。
- 最后一批放 0 个单元（单元数不变）时删除批计划断言 panic，变长、N = S 也不在它的定义域里。所以「unlink 已泛化的那一份」只覆盖截断的批，不覆盖意图、最后一批放 0 个、变长。

### 3.3 候选（实现形）

- **抽出下段计划**：把 `plan_the_extent_tree_after_a_size_change` 里算下段节点与换下节点的那一半（`crates/singlefs-core/src/truncate.rs:243-297`）抽成「入参是某个文件上一版下段节点表 + 改 size 前后的样子」的函数，第一个文件与别的文件各调一次；`ChangeOfAFileOtherThanTheFirst` 加改 size 变体，`FileOtherThanTheFirstOrigin` 加对应来历；上段仍走统一计划。
- **适配**：别的文件的下段节点表临时包成一个 `ExtentTreeVersion` 去调现成的 `plan_the_extent_tree_after_a_size_change`，再把结果里下段角色改名成别的文件那一族。改动最小，但角色改名是一道手工转换，错了不会编译失败。
- **并成一个物种**：把第一个文件也并进 `FileOfAnInodeOtherThanTheFirst` 那一族。这是 C618 与里程碑十一·3 的事（`.claude/kb/milestone/10-tenth-txn.md:44`），不在这一行。

### 3.4 要满足的条件

- 叶边界：意图那次罩着被重写单元 k 的下段叶到根恒重写，k 所在叶的整边界（留下 144、288 个单元）也一样；size 之外挂着的单元所在的叶照抄（单元数没变）。
- 批：释放至少一个、不重写，直接复用删除批的下段计划（3.2 节 11/11）；被截掉的单元与不再出现的下段节点没有同名后继、经映射释放或进 deadlist。
- 最后一批放 0 个单元、变长、N = S：下段整棵照抄（变长重写尾单元时只加罩着它的叶到根）；零单元的文件（推荐的标签 0）这几步下段为空、整棵照抄。
- 上段：条目变了的号是被改的那个 X；有条目的号集合只在 X 原来没有条目（零单元文件第一次变长）时变，走统一计划 `extent_tree_plan_with_the_unified_upper_segment`（`crates/singlefs-core/src/transaction.rs:7161`），写别的文件那一支已经这么用（`plan_the_extent_tree_after_writing_a_file_other_than_the_first`，`crates/singlefs-core/src/transaction.rs:7464`）。
- 尾单元重写的角色、单元头对象与对象出生代（取 X 的记录，不取 `record_of_the_first_file_after_this_publish`，`crates/singlefs-core/src/transaction.rs:13458`）、旧净荷来源、映射 key 的事务号，全按 X。
- 快照共享：被重写单元的上一版角色要放进 `replaced_previous_data_roles`（`crates/singlefs-core/src/transaction.rs:7281`），走「.chain(files_other_than_the_first.replaced_previous_roles())」那一条链（`crates/singlefs-core/src/transaction.rs:8280`）；8267 行起的过滤只收第一个文件的 `Data(_)`，别的文件的尾单元放那里会漏判生死；整份写别的文件已经走链、不能再放进过滤（会双算）。读码推，没跑。
- 认领：截断条目按 (inode, 对象出生代, 目标 size) 认，三项原样在条目里（`crates/singlefs-core/src/logged_operations.rs:323` 的 `parse_truncation_intent`）。续做改成取 `previous.inode_record_of(inode)`（`crates/singlefs-core/src/transaction.rs:4087`，已经是任意 inode 的）核：记录在、nlink 不是 0、对象出生代相同、size = 目标；步骤取单元数改走「第一个文件取 `data_pointers`、别的文件取 `file_other_than_the_first(inode)`」那份。
- 回退取目标版的条目（K1）：条目取自目标版（「.find(|entry| entry.operation.kind() == LoggedOperationKind::Truncation),」，`crates/singlefs-core/src/mount.rs:9938`），inode 树取目标版（`crates/singlefs-core/src/mount.rs:9593` 的「inode_record: target_version.inode_record,」并上 `inode_leaf_containers`），别的文件的单元与下段也取目标版（「files_other_than_the_first: target_version.files_other_than_the_first.clone(),」，`crates/singlefs-core/src/mount.rs:9573`）。三者同源，所以目标版的记录 size 恒等于条目的目标 size（意图那次一起写下，K2「先落目标」，批与最后一批不碰记录）。认领条件在回退后的版本上成立，不需要新规则；带快照的回退本来就先拒（`research/prompts/m4-trunc-r1-main-verification.md:23`）。
- 准入：K10 钉的「截断意图那次每盘新占槽数不超过 W 的 24」要对别的文件再钉一次；别的文件上段路径与第一个文件同层数，推的，没量。

### 3.5 推荐与依赖的前提

推荐「抽出下段计划」，步骤类型带 inode（`TruncationBatch`、`TruncationFinalBatch` 各加 `inode` 字段，由 `publish_truncation_step` 从条目填），不让计划层再去猜 inode。依赖的前提：S2 选「粘住的标签 0」（否则零单元文件的变长与缩短在这里没有落点）；K1 至多一条（S4）。

### 3.6 什么现象会推翻

- 探针 3.2 节在入库装置上复跑，不带重写单元的批两份计划有 kept 取值不相同；
- 别的文件的截断意图在 144 或 288 个单元上被删除批计划做出来、池级 checker 与对拍都绿（那就说明 K9 的叶边界在别的文件上不用单独处理）；
- 找到一个把别的文件尾单元的上一版角色留在 8267 行过滤里也不双算的写法（那 3.4 的快照条件要改）。

## 4 S4　「全池活截断条目至多一条」（K1）留不留

### 4.1 今天谁在强制它（都不看 inode 号）

- 条款：「全池活截断条目至多一条」（`research/prompts/m4-trunc-r1-main-verification.md:35`）。
- 读者：「if truncation_intents > 1 {」（`crates/singlefs-core/src/logged_operations.rs:401`）接着 `MoreThanOneTruncationIntent`；`live_truncation_intent_of_the_entries` 里再断言一次：「全池活截断意图至多一条（判决 m4-trunc-r1 K1）：签发入口断言上一版没有截断条目」（`crates/singlefs-core/src/logged_operations.rs:454`）。
- 签发入口：全仓只有一个签发点（1.2 节输出，`IssueTheTruncationIntent` 的构造只在 `transaction.rs:10557`），它前面断言：「发截断意图之前三样队列（deleted_inodes、销毁中的快照、截断条目）都已走空（判决 m4-trunc-r1 K1）」（`crates/singlefs-core/src/transaction.rs:10546`）；条目集合变更处再断言一次（`crates/singlefs-core/src/transaction.rs:6025`）。
- 会话：每次改动先走空再做它自己，走空失败（空间不够）就带着错返回、不发这次改动：「published_in_this_change = walk?;」（冻结副本 mounted_session.rs 第 2321 行）。挂载续做同一个走空（`crates/singlefs-core/src/mount.rs:7442`）。条款：「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的」（`.claude/kb/decisions/08-核心索引结构.md:509`）。
- 回退：条目集合是「删光现行版的截断条目，再放目标版的那一条（至多一条）」，不是并集：「entries.retain(|entry| entry.operation.kind() != LoggedOperationKind::Truncation);」（`crates/singlefs-core/src/transaction.rs:6048`），目标版的条目由调用方取：「.find(|entry| entry.operation.kind() == LoggedOperationKind::Truncation),」（`crates/singlefs-core/src/mount.rs:9938`）。
- checker：冻结副本 walk.rs 第 18385 行起，I-4.4 在有截断条目的那一版判「logged_ops 里有 N 条截断条目，全池至多一条」；每条条目对自己的记录的判（第 18100 行）只按条目里的 inode 号取记录。

### 4.2 量（探针 `probe_s4`，原样）

现行版挂着号 3 的截断条目（签发 txg 9），回退目标版挂着号 2 的（签发 txg 5）：

```
回退取目标版条目：现行 [3] → 回退后 1 条，号 [2]
回退目标版没有截断条目：回退后 0 条
盘上读到两个号各一条截断条目：Err(MoreThanOneTruncationIntent { truncation_intents: 2 })
```

### 4.3 推出来的

- 串行下会话每次改动先走空，推得出任一版最多一条：签发点唯一、签发前断言队列为空、会话与挂载续做都先走空（4.1 节）。这条推导的前提是「一次只有一个写者在发、每次用户改动之前走空」，两个前提都是 D8 已定项 16 第 6 条（509 行）。
- 管理员回退合不出两条（两个不同 inode 各一条）：回退后的条目集合 = 目标版的条目（0 或 1 条），现行版的不并进来（4.2 节第一、二行）。目标版本身至多一条，因为每一版写出来之前都过上面的强制点，盘上读到两条的读者拒（4.2 节第三行）。
- 回退时 inode 树、extent 树与别的文件的单元也取目标版（3.4 节），所以「条目点名的 inode」与「记录」永远来自同一版；K1 当初打中的两个方向（回退到截断做了一半的那一版意图丢、现行版条目带到 R_old 的内容上，`research/prompts/m4-trunc-r1-main-verification.md:20`）在推广到任意 inode 之后原样被这一替换堵住，与 inode 号无关。

### 4.4 候选

- **留，写成「任一版 logged_ops 里种类码 2 的条目不多于一条，不分 inode」**。读者、签发断言、checker I-4.4 今天已经是这个写法，推广不改一行。
- **放开成「每个 inode 至多一条」**。要改：`live_truncation_intent_of_the_entries` 变成按 inode 取（`crates/singlefs-core/src/logged_operations.rs:444`）；`walk_the_deletion_queue_until_it_is_empty` 要选先走哪一条、批之间不得互相放同一个单元（D8 已定项 5 第 4 条「一批里每个动作施加时先核它要删的东西还在」，`.claude/kb/decisions/08-核心索引结构.md:127`，两个 inode 的计划不相交，但这一条要重新论证）；回退取目标版的「全部」截断条目；读者与 I-4.4 的"至多一条"换成"每个号至多一条"；队列空的四处判断不变；GPU 判器的 I-4.4 同改。没有任何一格要求它。

### 4.5 推荐与依赖的前提

推荐留，条款文字不改（K1 那一句本来就没限 inode）；在阶段同步里把「至多一条」的理由补一句：由「签发点唯一 + 前置队列为空 + 回退是替换」三样推出。随这一格落两个世界（实现员取名）：
- 两个 inode 的回退：号 2 截到一半留下回退目标版（有条目 2），之后走空完成号 2、再对号 3 发截断意图并停在批间，管理员回退到号 2 那一版；结果只有号 2 的一条，续做把号 2 截完，号 3 的单元与 size 回到目标版。变异：删去 6048 行的 `retain`，结果两条，下一次挂载读者拒、checker I-4.4 红。
- 反方向：现行版没有条目、目标版有号 2 的：回退后一条，续做走到空。

依赖的前提：会话串行、每次改动先走空。里程碑十一·1 的多会话与 D16 已定项 10 的攒批落地时这个前提失效，截断判决已经为攒批记了欠账（「R7 攒批欠账」，`research/prompts/m4-trunc-r2-main-verification.md:51`）；那时「至多一条」要重开，不属于这一行。

### 4.6 什么现象会推翻

- 找到一个不经 `publish_size_change`（`transaction.rs:10457`）签发截断条目的入口，或一个回退之外把现行版条目带进新版本的入口；
- 一条合法历史（会话、挂载、回退的任意交错）末尾读者拒 `MoreThanOneTruncationIntent`；
- 用户要求两个 inode 的截断可以同时挂着（那时选「放开」，4.4 节的清单是它的最小改动面）。

## 5 S5　对拍对象

### 5.1 今天的对拍对象与边界

- RefFS 没有文件身份：操作集合只有「CreateTheFirstFile」「OverwriteTheFile」等，截断是「Truncate(u64),」（`crates/singlefs-reference-filesystem/src/lib.rs:59`），整份源码里「inode」一词只出现一行注释（1.2 节输出）。
- 翻译层对多文件的先例是停下：按 inode 号写写了盘就停：「HistoryOperation::WriteOfAnInode(_) => match &seen.ending {」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:474`），停因「TheReferenceFileSystemHasNoFileOtherThanTheFirstInTheFirstVersion,」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:481`）；unlink 同（`reference_filesystem_translation.rs:485` 起，停因 `TheReferenceFileSystemHasNoUnlinkInTheFirstVersion`）。截断那一支**不分 inode**：「HistoryOperation::Truncate(_) => match (&seen.ending, seen.size_requested_by_the_truncation) {」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:393`）把执行器交回的目标 size 一律翻成 `Operation::Truncate(n)`。今天不出事，是因为执行器只在第一个文件没有记录、又建过别的 inode 时才点名别的 inode，而且那一格被拒（`crates/singlefs-harness/src/history.rs:5187` 与 `:5209` 两个分支）。推广之后执行器点名别的 inode 做成了，这一支会把别的文件的 size 当第一个文件的喂给 RefFS，**模型与 RefFS 假红**。
- 模型对多文件的答法有两个材料没写的边界：
  - 四个臂在「别的文件带数据单元」时拒答：`refuse_to_answer_with_a_file_other_than_the_first_holding_data_units` 的四个调用点是建快照 5754、删快照 5800、挂着时整理 5935、截断与变长 6132（`crates/singlefs-harness/src/model.rs`，1.2 节输出），拒答成 `NotModeled`。也就是说今天「快照共享」「整理」与别的文件的数据单元同时出现的历史模型答不了。
  - 截断途中的状态只装一个文件：「FileBeingTruncated {」（`crates/singlefs-harness/src/model.rs:1298`）带的是一个 `ModelFileVersion`；别的文件的模型「pub struct ModelInodeOtherThanTheFirst {」（`crates/singlefs-harness/src/model.rs:1121`）只有内容、分配代、对象出生代、活死，没有单元数、标签、截断字段。
- 观测一侧写死「没有洞」：「每个号恰好一个，接出来的长度要恰好是 size」（`crates/singlefs-harness/src/model_comparison.rs:203`），别的文件带洞时观测函数（`crates/singlefs-harness/src/model_comparison.rs:205`）会把它判成 `Undecodable`。
- 崩溃后读回今天只比第一个文件的内容：冷启动答「ModelReadBack::FileRead」只带第一个文件（`crates/singlefs-harness/src/model.rs:2377`）；checker 档的内容读回「singlefs_checker::walk::first_file_content_of_a_root_record(&Reader(reader), &bytes)」（`crates/singlefs-harness/src/verification_read.rs:726`）也只读 inode 1。别的文件的读回走挂载之后的 `ObservedRoot.inodes_other_than_the_first`（`crates/singlefs-harness/src/model_comparison.rs:197`）。

### 5.2 候选

- **只对模型**：模型扩成多文件的截断、快照、整理臂；翻译层在别的 inode 的写（含截断）写了盘时停；RefFS 不动。
- **模型加 RefFS 多文件**：RefFS 先加文件身份、创建、删除，再让翻译层对多文件翻译。
- **模型加把 checker 档的内容读回泛化到任意 inode**：`first_file_content_of_a_root_record` 加 inode 参数，GPU 恢复内核的内容那一段（只第一个文件的数据指针进 R_DATA）同改。

### 5.3 要满足的条件

- 模型要能表达这一行的全部可观测：每个文件的单元数、上段标签（有无条目、1 到 4 或 0）、截断字段、截断条目点名的 inode；观测一侧同样要有，且别的文件带洞时补零到 size。对应 `ModelInodeOtherThanTheFirst`、`FileBeingTruncated`、`ObservedRoot.first_file_record_truncation_field`（`crates/singlefs-harness/src/model_comparison.rs:197` 一带）三处加 per-inode。
- 模型的快照、整理臂要能答「别的文件带数据」：这是「快照共享」验收的**前提**，不是附带；今天它们拒答（5.1 节）。
- 崩溃后读回的允许集合：由理想模型给。K2「先落目标」让任何批边的崩溃读回只有两样（意图之前读截前、意图之后读截后），模型里就是相邻两条根各自的 `ModelRoot`（内容加 `inodes_other_than_the_first`）；冷启动选根按 D23 已定项 14（`answer_cold_start_recover`，`crates/singlefs-harness/src/model.rs:6908`），不需要 RefFS 的 `AllowedSet`。RefFS 的允许集合只管它那一个文件。
- 翻译层与执行器：执行器把每次截断点名的 inode 记下来（`sizes_requested_by_the_truncations` 今天只记 size）；翻译层对 inode ≠ 1 且写了盘的截断停下，复用 `TheReferenceFileSystemHasNoFileOtherThanTheFirstInTheFirstVersion`。
- 随机历史点名谁：按现行版树里活着的号取（同 `WriteOfAnInodeChoice::inode_among`，`crates/singlefs-harness/src/history.rs` 的按 inode 号写那一组），目标 size 的类（缩到 0、单元边界、单元中间、单元内变长、跨单元变长、变长后截回洞里）按被点名文件那一刻的 size 与单元数取。
- 不能把现有拒绝边界直接当回归（`.claude/kb/milestone/04-fourth-txn.md:201`）：新用例的期望值来自条款（K2、K3、K5、K9）与模型，不是把旧的 `expect_err` 翻成 `expect`。

### 5.4 推荐与依赖的前提

推荐「只对模型」，RefFS 扩多文件归里程碑十的一·3 与里程碑十一（目录与 POSIX 面）。理由：
- RefFS 今天没有文件身份（5.1 节），多文件要新造创建、命名、删除，这些在条款里没有定义，不是大小变更这一行能定的；
- 多文件的先例就是只对模型：按 inode 号写与 unlink 都在翻译层停下（5.1 节），一·3 的验收也只写「模型对拍逐项相同」（`.claude/kb/milestone/10-tenth-txn.md:44`）；
- 这一行验收那一句（`.claude/kb/milestone/04-fourth-txn.md:201`）没有点 RefFS。

内容读回泛化到 checker 档只作为单列的待定项：这一行的「崩溃后读回对拍」在 harness 层做（崩溃重挂后的 `ObservedRoot` 逐版比），按记录判的结构判（I-4.6、I-4.10、I-9.38、I-4.4）对每个 inode 今天就有，GPU 恢复内核对别的文件的结构判在 S2 里补。要不要再加一条别的 inode 的截断崩溃流（`research/prompts/m4-trunc-r1-main-verification.md:43` 的 K9 三条流今天只有第一个文件）、连带泛化内容读回，是主 agent 与用户的事；若加，按「崩溃点测试准确率优先」办（`.claude/agent-common.md` 不做一节），代价是 `first_file_content_of_a_root_record` 加参数与 GPU 恢复内核内容段改写。

依赖的前提：
- 用户说的「对拍」指 harness 对拍，不含 RefFS；
- 模型的快照、整理臂能扩到多文件，且扩完不改第一个文件的既有答案（`crates/singlefs-harness/tests/` 里与快照、整理有关的既有用例要原样绿）。

### 5.5 什么现象会推翻

- 用户要求 RefFS 也比别的文件；
- 模型扩多文件时发现快照臂必须知道别的文件在快照里怎么读（今天读快照的入口只有第一个文件：`crates/singlefs-core/src/recovery.rs:7368` 的「pub fn read_the_first_file_of_a_snapshot(」），那时「快照共享」的对拍对象要加一个读别的文件的快照入口（算 S7 的清单项）；
- 执行器点名别的 inode 做成了、翻译层没停而 RefFS 对拍绿（说明翻译层另有保护，5.1 节第二条读错）。

## 6 S6　现有拒绝边界的用例与变异

### 6.1 今天谁在钉这道边界

- core：成员（`crates/singlefs-core/src/transaction.rs:5555`）与门（10483–10493）。core 里这道门没有任何变异行（1.2 节输出：`is_the_first_file` 在全表只命中 GPU 恢复内核那一行 4779）。
- harness 的模型与胶水：模型的拒因「ModelRefusalReason::SizeChangeOfAnInodeOtherThanTheFirstFile」出现在名字表与两处容量墙分支（`crates/singlefs-harness/src/model.rs:2338`、`:2365`、`:6971`）、答法函数 `answer_size_change_of_an_inode_created_after_the_first_file_was_deleted`（`crates/singlefs-harness/src/model.rs:6249`，分支在 6276 行）；胶水「} => explained(ModelRefusalReason::SizeChangeOfAnInodeOtherThanTheFirstFile),」（`crates/singlefs-harness/src/model_comparison.rs:1158`）与它的单测（1940–1956 行）；执行器的成员名表「} => "SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses",」（`crates/singlefs-harness/src/history.rs:3978`）。
- 用例：`a_size_change_of_an_inode_other_than_the_first_file_is_refused_unless_the_size_stays`（`crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs:257`，期望 inode 2 变长被拒、N = S 照做）；固定历史第 21 步断言拒绝：「matches!(&run.outcomes[21], StepOutcome::Refused { member } if member.contains("SizeChangeOfAnInodeOtherThanTheFirstFile")),」（`crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs:72`）。
- 变异三行（锚在拒绝成员名上，不在 `is_the_first_file` 上；正文第二节的"3 行"数对了、锚点说错了）：4456「m4 截断截丁 第一个文件之外的 inode 改 size 溢出时不先按溢出拒」（`crates/mutations.tsv:4456`，用例 `model::tests::a_size_change_of_an_inode_created_after_the_first_file_was_deleted_must_be_refused_before_any_write`）、4462「m4 截断截丁 胶水：删完之后点名别的 inode 改 size 被拒映射成说不清的理由」（`crates/mutations.tsv:4462`，用例 `model_comparison::tests::each_truncation_refusal_the_model_has_a_reason_for_maps_to_its_own_reason`）、4469「m4 截断截丁 执行器：拒绝的成员名不分第一个文件之外的 inode」（`crates/mutations.tsv:4469`，用例 `a_fixed_history_through_every_truncation_target_agrees_with_the_model_at_every_version`）。

### 6.2 候选

- **删成员**：推广之后"别的 inode 改 size"不再有拒绝；剩下的拒绝（不是活记录、N 向上取整溢出）早有各自的成员（「TruncationRefusal::InodeIsNotALiveRecord { inode: inode.0 },」，`crates/singlefs-core/src/transaction.rs:10473`；「TruncationRefusal::TargetSizeRoundedUpToAWholeDataUnitOverflows {」，`crates/singlefs-core/src/transaction.rs:10477`）。
- **留成员给别的情形**：唯一可能的情形是「删完第一个文件之后才建的 inode」（K-U5 的 β 读法，6.3 节）。
- **改名留着**：成员名里的"第一个文件之外"没有对象，不取。

### 6.3 一个要主 agent 或用户先定的前提：β 管不管改 size

D8 已定项 16 的 β 管的是**写数据**：「删完第一个文件之后（K-U5，记欠账）按 inode 号写数据只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode」（`.claude/kb/decisions/08-核心索引结构.md:513`）。删完之后才建的 inode 没有数据单元（写数据被拒），所以对它的大小变更只能是：变长只改记录加一条标签 0 条目，缩短没有单元可放。这两样不写用户数据，也不碰 β 依赖的墓碑死亡 txg。

- **不拒**（推荐）：它们和别的 inode 走同一条路，成员删净；β 的门（`crates/singlefs-core/src/transaction.rs:9373` 起的整份写入口）原样留着，C618 仍归一·3。
- **仍拒**：那要为它新起一个成员（"数据写"的成员名不对，改 size 不是数据写），模型里 `answer_size_change_of_an_inode_created_after_the_first_file_was_deleted` 留下并改名，4456、4462、4469 三行换锚而不删。

### 6.4 钉住它的用例与变异怎么改才不丢判别力

原则：不把 `expect_err` 翻成 `expect`。旧用例钉的是「条款没写所以拒」，新用例钉的是条款推出来的结果，且每条新变异都能让"退回旧边界或写死第一个文件"这个错红在一条用例上。

- 旧用例 `…is_refused_unless_the_size_stays`（257 行）拆成：号 2（建出来、size 0）变长到 N > 0 → 成功，读回 N 个 0，上段有号 2 的标签 0 条目（S2）；号 2 整份写三个单元后缩到单元中间 → 条目点名号 2、走空后单元数 = ⌈N ÷ 净荷⌉.max(1)、尾单元声明长度 = N − k × 净荷（K3）；缩到 0 → 留一个声明长度 0 的单元；N = S → 只改记录；nlink = 0 的号 → `InodeIsNotALiveRecord`；N 溢出 → 溢出成员。期望来自 K2、K3、K5、K12，不来自旧断言。
- 固定历史第 21 步（删完之后点名别的 inode 改成 6 字节）：按 6.3 节"不拒"，该步变成做成，模型逐版对拍；目标 size 表里的 `(21, 6)` 原样留着。
- 三行旧变异：4456 改锚到推广后模型臂里「N 向上取整溢出先于一切」那一句，对所有 inode；4462、4469 随成员一起删（成员没了，映射与名字表没有对象）。删三行的同时必须有下面这几条新行顶上，否则这块的判别力净减少：
  - 退回旧门（`transaction.rs:10485` 恢复"别的文件变长或缩短即拒"）→ 号 2 变长、缩短两条用例红；
  - 别的文件的 `data_units_existing` 仍取 0（`transaction.rs:10494`）→ 号 2 三个单元缩到中间，单元数没减、截断字段没写，红；
  - `names_the_live_first_file` 恢复（`transaction.rs:10604`）→ 号 2 的续做被 `TruncationIntentDoesNotNameTheLiveRecord` 拒，红；
  - 意图计划的下段叶不带"罩着被重写单元"这一条件，用在号 2 的 144、288 个单元上（`truncate.rs` 抽出的函数）→ 红；
  - 尾单元重写的对象、角色回到第一个文件（`transaction.rs:13465`、`:13485`）→ 号 2 的尾单元头里对象不是 2，重建或读回拒；
  - 别的文件的上段条目只出标签 1、2（`transaction.rs:12317`）→ 号 2 变长后重挂拒；
  - 冷恢复对别的文件的放宽退回（`recovery.rs:5940`）→ 号 2 变长后冷恢复拒；
  - 回退的 `retain` 删去（`transaction.rs:6048`，S4 的世界）→ 两个 inode 的回退用例红；
  - 快照下缩短别的文件，尾单元的上一版角色没进 deadlist（8267–8280 的链）→ 快照空间账的用例红。
  GPU 恢复内核两处（`crash_judge_common.wgsl:3387`、`crash_judge_recovery.wgsl:432`）的变异行和 CPU 对拍属 checker 档，归主 agent。
- "必红再改绿"：以上每条新用例先在冻结副本上跑一遍必须红（拒绝、panic 或读回不对），再在实现后绿；这是 C618 验收那一句的做法（`.claude/kb/checks-owed.md:387`「core 泛化到任意 inode」）。

### 6.5 推荐与依赖的前提

推荐：删成员；β 不管改 size（6.3 节"不拒"）；三行旧变异按 6.4 节处理。依赖的前提：6.3 节那一问由主 agent 或用户定；S2 选标签 0；S3 的计划抽出来。

### 6.6 什么现象会推翻

- 用户认为「删完之后才建的 inode 变长」也算 C618 要留给一·3 的"带数据的文件"：那时选"仍拒"；
- 新用例里任一条在冻结副本上不是红（说明旧边界并没有钉住它，要核是不是别处已经放行）。

## 7 S7　射程之外还剩什么只认第一个文件

### 7.1 判法

一处"只认第一个文件"的代码**必须一起改**，当且仅当：存在一条合法历史，在四样（快照共享、意图续做、关闭重挂、崩溃恢复）之一上让号 ≥ 2 的活 inode 的缩短或变长被拒、panic、写出读者拒的镜像或读回不对，而该历史的路径经过这一处。可以留给一·3，当且仅当经过它的历史要么在更早处被拒（例如 β），要么只涉及整份写别的文件、第一个文件、读快照这类本行不改的入口。

机械核法三层，缺一层不算清单成立：
- 行为层：6 条历史，每条在冻结副本上必须红、全部必改项改完后绿——号 2 建出来后变长到洞再重挂读回；号 2 三个单元缩到单元中间再关闭重挂；号 2 在批间停下再重挂续做；建快照后缩短号 2 再删快照（空间账平）；号 2 带洞时被 unlink；两个 inode 的回退（S4）。模型逐版比。
- grep 层（改后应零命中）：`is_the_first_file` 在 `transaction.rs`（改前 3 行）、`names_the_live_first_file`（改前 2 行）、`SizeChangeOfAnInodeOtherThanTheFirstFile` 全仓（改前 22 行）、`tail_is_a_hole = is_the_first_file` 在 `crash_judge_recovery.wgsl`（改前 1 行）、`recovery.rs:3853` 那一行写死的 `NoHoleAfterTheLastDataUnit,`（该文件另一处 5968 是"上段里没有第一个文件"的返回，不动）。命令与改前计数在草稿目录的 `evidence-s7.txt`。
- 范围层：`grep -rn FIRST_INODE_NUMBER crates/singlefs-core/src` 改后的非测试行，每一行都能在 7.3、7.4 节的留下清单里按函数名找到；多出的行说明改动越界或清单漏项。

### 7.2 必须一起改（core，冻结副本行号）

四样一栏写这一处不改时哪一样先坏。

- `transaction.rs:10483`–`10498`（`publish_size_change` 的门与单元数）：四样都在第一步被拒。
- `transaction.rs:10604`–`10616`（`publish_truncation_step` 认领与单元数）：**与上一条必须同批改**。只放开上一条，意图能签发，续做每次被拒成 `TruncationIntentDoesNotNameTheLiveRecord`，队列永远走不空，会话里之后每次改动先走空都失败、挂载续做也失败（意图续做、关闭重挂坏）。
- `transaction.rs:5826`–`5833`、`5855`–`5857`、`7941`–`7946`（截断两种批没有 inode；`data_units_after_this_publish` 把它们当第一个文件的，7941 起用第一个文件的单元数喂它）：意图续做坏。
- `transaction.rs:7247`、`7207`、`7309`–`7440`、`7949`–`7978`（别的文件的计划物种只有整份写与两种删除批；没有改 size、截断的批与最后一批、尾单元重写）：全坏。
- `transaction.rs:7991`–`7996`、`8918`–`8951`、`truncate.rs:226`（计划函数的下段一半与"改 size 前后的样子"取自第一个文件）：全坏，叶边界漏重写（3.2 节）。
- `transaction.rs:3181`、`3673`、`12317`–`12323`、`4001`（`FileOfAnInodeOtherThanTheFirst` 装不下零单元与洞的尾部；上段叶条目构造只出标签 1、2、零单元 panic）：变长之后任何重写上段叶的发布丢条目或 panic，关闭重挂坏（S2）。
- `transaction.rs:13458`–`13485`、`13189`、`7860`、`7752`、`8538`–`8560`（尾单元重写的对象、角色、旧净荷来源、映射 key）：崩溃恢复坏（单元头对象不对，重建时读者拒）。
- `transaction.rs:8267`–`8280`（deadlist 角色选择）：快照共享坏（尾单元与被截掉的单元的上一版没进 deadlist，被快照引用的块被释放）。
- `recovery.rs:5939`–`5940`、`:3853`（三处读者共用的别的文件的收法）：关闭重挂、崩溃恢复坏。
- 同批但不在 core/src：GPU 恢复内核 `crash_judge_common.wgsl:3387`、`crash_judge_recovery.wgsl:432`；池级 checker 的别的文件无条目判（S2 的新判）；模型、胶水、执行器、翻译（S5、S6）；条款与 layout（S2）。

### 7.3 可以留给一·3（有理由，非测试 FIRST_INODE_NUMBER 行）

- `transaction.rs:4034`（`death_txg_of_the_deleted_first_file`，β 读法）、`9373` 起的整份写入口的 β 门：β 管数据写，改 size 不经过它（6.3 节）。前提是 6.3 节选"不拒"。
- `transaction.rs:9279`、`9330`（按 inode 号写，inode 1 转给第一个文件的写入口）；`9049`、`9106`、`9168`（第一个文件的三个写入口）；`10820`（`live_record_of_the_first_file`）：整份写入口，本行不改。
- `transaction.rs:7131`、`7152`（第一个文件整份写的 extent 计划）、`7835`、`7855`（`first_file_version`、`inode_of_the_file_other_than_the_first_written`，由 `self.file` 定，只在整份写时有值）、`4549`（`FileContentOfThePublish::of_the_plan`）、`12179`、`12204`（装第一个文件的下段）、`12375`、`12378`（装上段叶里第一个文件那一条）、`13497`（装别的文件整份写的单元）、`13597`（取第一个文件那条记录）：只与整份写或第一个文件有关。
- `recovery.rs:3805`（重挂校验第一个文件那条记录）、`7287`、`7468`（冷走读与快照读第一个文件的内容）、`recovery.rs:7368` 的 `read_the_first_file_of_a_snapshot`：读第一个文件内容的入口。**条件**：本行"快照共享"只用 deadlist 账与模型的快照臂验证；若要读出快照里别的文件的内容，要加 `read_a_file_of_a_snapshot(inode)`，那一处就从这里搬进 7.2 节。
- `transaction.rs:7316`、`8006`、`7958`、`7964`、`10398`、`unlink.rs:259`、`:290`：unlink 的批与最后一批，已经按 inode 分，不需要动（7958、7964 两处所在的 `match` 因为 7247 的枚举加变体要补臂，但行为不变）。

### 7.4 与本行无关

`transaction.rs:178`（常量定义）、`260`、`263`、`331`、`336`（inode 号分配与水位）、`unlink.rs:22`、`recovery.rs:112`（导入）；测试里 16 行。

### 7.5 不带 `FIRST_INODE_NUMBER` 却同样把第一个文件写死的地方

这是 grep 看不见的一类，必须单列，靠"字段名"找：第一个文件的字段 `previous.data_pointers`、`previous.inode_record`、`previous.extent_tree.lower_nodes`、`TransactionUnit::Data(_)` 这一族。冻结副本里与截断路径有关的命中已并入 7.2 节（8943、10616、7752、8538、13189、13485）；其余命中（4569、4628、8007 起）是整份写或 unlink 已按 inode 分的，不动。

### 7.6 本行没改但行为依赖的前提（读码，没跑）

- 取号之前的预演只走 `deleted_inodes`（`transaction.rs:10666`）。截断续做不在预演里，第一个文件今天就是这样。
- `mount.rs:9573`、`9593`、`9938`（回退取目标版的别的文件单元、inode 树、截断条目）、冻结副本 mounted_session.rs 第 2321 行（先走空）、`mounted_read.rs:809`（按条目有无分流）、`write_request_split.rs:106`（读侧函数）、`logged_operations.rs:401`（至多一条）：已经是任意 inode 的，不用改。

### 7.7 推荐与依赖的前提

推荐 7.2 节为"必须一起改"清单、7.3 节为"留给一·3"清单，判法照 7.1 节三层。依赖的前提：
- S2 选"粘住的标签 0"，S5 选"只对模型"，6.3 节选"不拒"；三者任一改变，7.2、7.3 两张清单各有一两项互相搬动（S2 改选"先落补零单元"：12317 那一项变成"新建单元的路径"；S5 加 RefFS：模型与翻译一栏加项；6.3 选"仍拒"：β 门多一条）。
- 清单里的行号是冻结副本的，实现员改动之后会漂；判法按函数名与 grep 模式，不按行号。

### 7.8 什么现象会推翻

- 6 条历史里任一条在冻结副本上不红，说明那条路径上没有未列的第一个文件耦合（该条对应的清单项可以删）；
- 全部必改项改完，6 条历史里仍有红，说明清单漏项（7.5 节那类 grep 看不见的耦合最可能）；
- 改后非测试的 `FIRST_INODE_NUMBER` 行里出现 7.3、7.4 节没有的函数。

## 8 判定一览

判定三种：一致（推出来的与被判条款一致）、冲突（两句并排）、规则没说。

| 格 | 判定 | 一句话 |
|---|---|---|
| S2 别的文件的洞 | 规则没说；与今天实现冲突 | 条款只写第一个文件的标签 0（`08-核心索引结构.md:451`、`:465`、`:477`、`:478`）；推出的该收别的文件的标签 0、3、4，今天 `recovery.rs:5939`–`5940`、`:3853` 拒。推荐「粘住的标签 0」，要新加一条 checker 判（别的文件无条目 ⇒ size 0）和 GPU 恢复内核两处 |
| S3 续做与计划 | 规则没说；K9 叶边界一致 | K3、K9 的话没限 inode；unlink 的计划只覆盖截断的批（探针 11/11 相同），意图（漏重写叶）、最后一批放 0 个（断言 panic）、变长都不在它的定义域；续做认领与回退取目标版条目不需要新规则 |
| S4 至多一条 | 一致 | 签发点唯一、前置队列空、回退是替换而不是并集（探针：回退后 1 条；盘上两条读者拒），推广不改；放开的改动面列在 4.4 节 |
| S5 对拍对象 | 规则没说 | 验收那一句没点 RefFS；RefFS 没有文件身份，多文件先例是翻译层停下。推荐只对模型，但模型今天对快照、整理、截断三个臂在"别的文件带数据"时拒答，这是前提不是附带；翻译层的截断支不分 inode，推广后会假红 |
| S6 拒绝边界 | 一致（方向）；冲突（前提） | 删成员，β 不管改 size（要主 agent 或用户定）；正文说"锚在拒绝成员与 `is_the_first_file` 上的 3 行"，实测三行都在 harness 的拒绝成员名上，core 的门零变异行 |
| S7 清单 | 规则没说 | 42 个用法行里必须一起改的核心项 9 组（7.2 节），留给一·3 的 7.3 节，无关 7.4 节；grep 看不见的第一个文件耦合 7.5 节；判法三层 7.1 节 |

冲突两句并排（S6 的前提）：
- 正文第二节：「钉住边界的用例与变异：`crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs`、`truncation_random_histories_against_the_model.rs`；`crates/mutations.tsv` 里锚在拒绝成员与 `is_the_first_file` 上的 3 行。」（`research/prompts/_m4-multi-inode-size-r1-body.md:22`）
- 实测：`crates/mutations.tsv` 全表 `is_the_first_file` 只命中 4779 行（GPU 恢复内核），拒绝成员名命中 4456、4462、4469 三行，都在 harness（1.2 节输出）。

冲突两句并排（S2 的实现一边）：
- 条款推出：别的文件的标签 0 配 size > 0、零单元，读者应收（读侧矩阵：标签 0 + size 100 → `Ok(0)`）。
- 今天 `extents_of_the_first_file`：「|| extents.tail() != TailOfAFile::NoHoleAfterTheLastDataUnit」（`crates/singlefs-core/src/recovery.rs:5940`），零单元与带洞的别的文件被拒成 `UnitMalformed`。

另有一处 kb 文字过期（规则没说的一类，不是这一行的前提）：D8 已定项 16 射程仍写「第一版按 inode 号删任一 inode、无目录、不带 truncate」（`.claude/kb/decisions/08-核心索引结构.md:513`），而截断判决回看表写明"射程里「不带 truncate」随阶段同步改写"（`research/prompts/m4-trunc-r1-main-verification.md:73`）；今天仓里这句还在。

## 9 要主 agent 跑的命令

- 复跑探针（轻型，不是 checker 档）：见 0 节，`bash rerun.sh …`；复跑时输出里线程号会变。
- 实现之后属 checker 档、归主 agent 的：GPU 恢复内核与 CPU 恢复的对拍（别的文件有洞、标签 0 的世界）、变异表里新行的 `--quick` 复跑、崩溃点重放里是否加一条别的 inode 的截断流（5.4 节待定项）。我没有起这些。

## 10 没做什么

- 没判攻方腿的格，没采纳或否决任何一条攻击，没出判决；推荐只是正推腿的推荐，前提都写在每节的"依赖的前提"里。
- 没读背景材料附录（1.5 MB）；kb 条款与判决原文回 kb 文件现查，代码读冻结副本。
- 没跑任何门禁、checker 档测试、GPU；没起 QEMU。探针只在 `singlefs-core` 三个包的副本上跑（经 `capped.sh 31` 与内存包装，一次只起一条 cargo），没动冻结副本。
- 没 grep `.claude/gate.d/` 下的钉表（2.5 节）；没读 `crates/singlefs-checker/invariants/` 的不变量清单文字，I-4.6 等的行为按冻结副本 walk.rs 的函数读。
- 没在入库装置上复跑探针的数；3.2、4.2 的数是副本上量的，按三方规则不能直接进 kb，只当线索。
- 读码推的（没跑）：3.4 的快照角色条件、3.4 的准入槽数、2.4 的"新判不改任何挂载结局"、7.2 节各项"哪一样先坏"的归因。
- 草稿目录里留下 `probe_multi_inode_size.rs`、`rerun.sh`、`probe-final-out.txt`、`evidence.txt`、`evidence-s7.txt`、`ln.sh` 与几份编译日志；仓副本与编译目录（草稿目录下的 `probe-tree/`，删之前 `du -sh` 是 535M）交回前已删。
