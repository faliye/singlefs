# m4-trunc-r2 正推腿（Sonnet）报告：截在单元边界再变长，扫描重建认回放掉的单元（2026-10-17）

正文 `research/prompts/_m4-trunc-r2-body.md`，格 W1–W5。前一轮判决 `research/prompts/m4-trunc-r1-main-verification.md`（K 项、S1–S5）。开工快照 `research/prompts/m4-trunc-r2-snapshot/kb-sha256.txt`，`sha256sum -c` 全部 OK（开工时与交回前各核一次）。
这一轮没有起 cargo、没有编译、没有跑任何测试、没有起 GPU；量的只有一份 python3 纯算术小模型（`research/prompts/m4-trunc-r2-sonnet-model/`，经 `scripts/hosts/run-with-memory-cap.sh 2G` 起）。下面「量的」指这份模型上的输出，「读码的」指我读 `crates/` 与 kb 得来的，「推的」指没有量。

## 0　共用事实（读码的；行号都是工作树 `~/.local/share/singlefs/work/m4-predev` 里的，写进报告前逐句 grep 过）

- 题面那条红的形状在模型里复现：K6（对象出生代、size 之内、写序三条）在「截在边界、变长」一步多认领 3 个单元，与题面「中央映射差 3 处」同数（量的，`out/named.txt` 的 H1 一节 K6 一行）。写死历史里第 1 步是 「truncate(TruncationTarget::ToADataUnitBoundary, 0),」（`crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs:61`），第 2 步变长到 3 × 净荷 + 1。
- 扫描重建认领的现行代码：size 那一条是 「if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {」（`crates/singlefs-checker/src/rebuild.rs:778`），写序那一条是 「.is_some_and(|newest_below| *newest_below > birth_txg_of(version))」（`crates/singlefs-checker/src/rebuild.rs:783`）。当前版本只在谱系上出生的版本里取：「.filter(|(_, version)| txgs_on_the_lineage.holds(birth_txg_of(version)))」（`crates/singlefs-checker/src/rebuild.rs:717`）。inode 记录来自同一份当前版本，不读快照那一版的旧容器。
- 遍历一侧只按 size 滤 size 之外挂着的单元：「>= data_units_the_size_asks_for(read_u64(record, INODE_RECORD_SIZE_OFFSET))」（`crates/singlefs-checker/src/walk.rs:13615`）。放掉的单元不在 extent 树里，遍历一侧本来就看不见它们，所以三条臂都不动遍历一侧（推的，没有哪条臂需要它改）。
- 记录第 108–139 字节今天的读法：「|| bytes[108..].iter().any(|byte| *byte != 0)」（`crates/singlefs-core/src/records.rs:93`）。字段表是 `| 108 | 填充（恒 0） | 4 |`、`| 112 | flags（恒 0） | 8 |`、`| 120 | 预留（恒 0） | 20 |`（`.claude/kb/decisions/08-核心索引结构.md:191`、`:192`、`:193`）。
- 整理搬迁不改出生 txg：「搬迁不改 birth」（`.claude/kb/decisions/26-后台整理与放置回收.md:123`）；代码一侧「原样字节写到目的地（头不盖戳、birth 不变）」（`crates/singlefs-core/src/transaction.rs:7846`）。
- 回退时 inode 记录取 R_old、墓碑树取现行：「inode_record: target_version.inode_record,」（`crates/singlefs-core/src/mount.rs:8200`），「tombstone_containers: current.tombstone_containers.clone(),」（`crates/singlefs-core/src/mount.rs:8206`）。
- 截断意图只在意图那次发布写 inode 记录，批不碰它：「意图那次一次发布写记录的 size = N」（`research/prompts/m4-trunc-r1-main-verification.md:34`）；今天的记录构造在 `crates/singlefs-core/src/transaction.rs:8600`（`let record_after_the_change = InodeRecord {`）。
- 写死历史第 3 步 「truncate(TruncationTarget::BackIntoTheHole, 0),」（`crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs:63`）选择子是 0，截到的位置就是存在的单元末尾（`crates/singlefs-harness/src/history.rs:651-658`），也就是回到第 1 步的 N，所以这条固定历史分不出「每次截断都盖章」与「放了单元才盖章」两种字段更新规则（读码的；随机历史的选择子不为 0 时才分得出）。

## 1　W1　载体：截断的信息记在哪

### 1.1　判定

- 甲（inode 记录加字段）能落地成一个 16 字节、0 个新单元、0 个新槽的形态；在 W2 列的每一种历史上，下面写的那个更新规则在模型里到深度 5（2 564 298 个状态）0 条不同（量的，`out/search-depth5.txt`）。前提是更新规则是「放了单元的截断才盖章」，不是「每次截断都盖章」：后者在 W2 的 H4 上就红（量的，`out/named.txt` H4 一节「甲last」一行）。
- 乙（区间墓碑）在同一批历史上同样 0 条不同，多罩了 W5 的 pwrite / punch，多付了永久增长和一条用户 2026-10-08 定下的条款要翻。
- 丙（尾单元一律重写）在现有入口下同样 0 条不同（量的），代价是边界截断每次多写一个数据单元和一串 extent 节点，并且它整个站在「写序那一条」上，W5 一来写序先塌（量的）。
- 条款间一处看着冲突、实际 D8 已点名排除：`.claude/kb/decisions/15-格式冻结政策.md:185` 的表第一行是「新增字段在预留区 / TLV 尾部」判 compat，而 `.claude/kb/decisions/08-核心索引结构.md:199` 写着「记录内不存在 compat 扩展路径」，并且点名「D15（格式冻结政策） 判定表第一行够不着」。按 D8 已定项 6 这句更具体的，甲给预留字节赋义是 incompat；已定项 6 同一行还写着「将来给任一位 / 任一字节赋义按 D15（格式冻结政策）」「判 incompat，退役的字节永久报废」。
- 推翻它的现象：若主 agent 在 `crates/` 上复现，发现 2 个以上的 W2 历史在甲下仍红，或「放了单元才盖章」的条件在真实发布路径里取不到（比如 `data_units_existing` 在意图那次发布里拿不到），则甲的判定作废。

### 1.2　三条臂各一句定义

- 甲：在 inode 记录偏移 120 起放两段 8 字节：「最近一次放了单元的截断的 txg」T 与「那次截断留下的单元数」K；读法 Z 多判一句，偏移 ≥ K 个单元且出生 txg < T 的数据单元不认领。
- 乙：截断意图那次往墓碑树的开放容器追加一条区间记录（对象 ID、对象出生代、区间 [K × 净荷, 2⁶⁴ − 1]、死亡 txg = 意图 txg），扫描重建认区间墓碑。
- 丙：截断意图那次，留下的最后一个单元只要存在就重写（截在边界也重写），读法 Z 与格式都不动。

### 1.3　甲写成可落地的形态

字段（小端，都在 140 字节记录之内，记录宽、一叶 233 条都不变）：

| 偏移 | 宽 | 内容 | 值域 |
|---|---|---|---|
| 108 | 4 | 填充 | 恒 0（不变） |
| 112 | 8 | flags | 恒 0（不变） |
| 120 | 8 | T：最近一次放了单元的截断的 checkpoint_txg | 0 = 没有；否则 对象出生代 ≤ T ≤ 改动计数 |
| 128 | 8 | K：那次截断留下的单元数 = ⌈N ÷ 净荷⌉.max(1)（即 `truncate.rs` 的 `data_units_a_size_needs(N)`） | T = 0 时恒 0；T ≠ 0 时 ≥ 1 |
| 136 | 4 | 预留 | 恒 0（不变；单测 「tampered[139] = 1」（`crates/singlefs-core/src/records.rs:841`）与已知坏镜像用例的「预留段最后一个字节」都落在这里，照红） |

为什么放 120 起、不放 108 或 112：现有的两个「最后一个字节非零就红」的用例（`crates/singlefs-core/src/records.rs:841`：「tampered[139] = 1;」 和 `crates/singlefs-harness/tests/checker_known_bad_images.rs:13419`：「const INODE_RECORD_LAST_RESERVED_BYTE_OFFSET: usize = 139;」 那一形）落在偏移 139，留 136–139 恒 0 它们不用改（读码的）。存 K 不存 N：判法只用 K，N 多存 8 字节也装不下（120–139 只有 20 字节，T + N + K 要 24）；用户原话是「txg 与 size」，K 是 size 折成的单元数，同一信息的另一个写法，列进岔路表第 2 行。

什么时候写（照 K2「先落目标」，与 size = N 同一条记录同一次写）：
- 截断意图那次发布：若这次放单元之前存在的单元数 `data_units_existing` 大于 K（有单元要放），T := 这次发布的 checkpoint_txg，K := `data_units_a_size_needs(N)`；否则抄上一版的 (T, K)。
- 各批、最后一批：不碰记录（K2），字段不动。
- 变长、N = S 只改记录：抄上一版的 (T, K)。
- 整份覆盖写、顺序写：清零（「inode_record_of_file_version」（`crates/singlefs-core/src/transaction.rs:7314`）按零构造，零行 plumbing）；抄上一版也行，两种在模型里都到深度 5 无差异（量的，`甲rel清`、`甲rel留` 两行）。清零成立的理由：整份写把 0..n−1 号单元全用新 txg 重写，放掉的更早的单元出生 txg 都小于新写的单元 0，写序那一条判得死（H5b 一节）。
- unlink 的意图与各批：抄上一版的字段（「record_with_nlink_zero」（`crates/singlefs-core/src/transaction.rs:8443`）、「record_after_the_batch」（`crates/singlefs-core/src/transaction.rs:8506`）原样带过去），为的是回退到意图那一版时记录与今天只差 nlink 与 size。
- 管理员回退：不用写代码。字段在记录里，记录随 inode 树取 R_old（`crates/singlefs-core/src/mount.rs:8200`），字段跟着回到 R_old 的值。

读法 Z 多判的那一句（逐字）：

> 有同一对象出生代的 inode 记录、且该记录偏移 120 的 T 不为 0 时：数据单元的偏移 ÷ 净荷容量不小于该记录偏移 128 的 K、并且单元头出生 txg 小于 T 的，不认领。

四条一并读：对象出生代相同；偏移 ÷ 净荷 < ⌈size ÷ 净荷⌉.max(1)；没有偏移更小、出生 txg 更大的单元；（新）不落在「偏移 ÷ 净荷 ≥ K 且出生 txg < T」里。四条都满足才认领。没有同一对象出生代的记录的单元照旧认领（S3 不动）。写序那一条留着：整份覆盖写变短再变长（H5b）只有它罩得住；去掉它要让整份覆盖写也盖章，那时到 W5 又不够（见 W5）。

### 1.4　三条臂各要动 `crates/` 哪几个文件的哪一步、新写出哪些字节、代价

| | 要动的文件与那一步（读码的） | 新写出的字节 | 代价 |
|---|---|---|---|
| 甲 | `crates/singlefs-core/src/records.rs`：`InodeRecord` 结构（:50）加两个字段；`to_bytes` 把 `writer.skip(4 + 8 + 20);`（:85）拆成跳 12、写 T、写 K、跳 4；`parse` 把 `|| bytes[108..].iter().any(|byte| *byte != 0)`（:93）拆成 108–119 与 136–139 恒 0、两段合法性（T = 0 ⇔ K = 0）。`crates/singlefs-core/src/truncate.rs`：加一个纯函数算盖不盖章与新值（参数：`data_units_existing`、目标 size、上一版的 (T, K)、本次 txg）。`crates/singlefs-core/src/transaction.rs`：截断意图那次（:8600）按纯函数写字段，其余记录构造点抄或清零。 | 每个放了单元的意图发布，已经要重写的那条 140 字节记录里多 16 个非零字节。 | 新单元 0 个、新槽 0 个；记录宽、一叶 233 条、准入 W（`crates/singlefs-core/src/admission.rs:384` 的 24）都不变；长期占用不增。 |
| 乙 | `crates/singlefs-core/src/records.rs`：`TombstoneRecord` 旁加一个按尾部区间构造的入口（现有 `of_the_whole_object` 在 :137）。`crates/singlefs-core/src/transaction.rs`：截断意图那一步（`DeletionStepOfThePublish::TruncationIntent`）像 unlink 意图那样带一条墓碑（unlink 的写法在 :8451：`tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),`），复用 `crates/singlefs-core/src/tombstone_tree.rs:117`：「append_tombstone_to_the_open_container」。`crates/singlefs-checker/src/rebuild.rs`：墓碑那一判（:697 的 `ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,` 要改成认尾部区间）与数据单元认领（:778 一带）多判区间。`crates/singlefs-checker/src/walk.rs` 的 I-9.19、I-9.20 与 GPU 扫描内核的同两条（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:4262`：「judge(INVARIANT_I_9_19, d_inode_record_nlink(record_row) == 0u);」 一带）要从「整对象」放宽到「整对象或尾部区间」。 | 每次截断意图：40 字节墓碑记录，连带重写开放墓碑容器（32 KiB，每盘 2 槽）和墓碑树根（16 KiB，1 槽）：每盘约多新占 3 槽（推的）。 | 永久增长：40 字节 × 截断次数，第一版不回收（`.claude/kb/decisions/18-块里携带什么信息.md:579` 的「第一版不回收墓碑」）；10⁶ 次截断 1227 个容器、两份 76.69 MiB（量的，`out/costs.txt`），与 D18 写的 10⁶ 次 unlink 约 76.7 MiB 同数；一根满 135 个容器（`crates/singlefs-core/src/tombstone_tree.rs:106`）即 110 025 条之后墓碑树长高，每次意图多重写一层节点（推的）。要翻 D18 已定项 10 的「区间墓碑不做」，那是用户 2026-10-08 定的（`.claude/kb/decisions/18-块里携带什么信息.md:249`），要用户再定。 |
| 丙 | `crates/singlefs-core/src/truncate.rs`：`tail_data_unit_rewrite_of_a_truncation`（:94–110）去掉 `(declared_length_within_a_size(unit, size_before_in_bytes) != declared_length_in_bytes)`（:104）这个条件，k 存在就返回 `Some`。`crates/singlefs-core/src/transaction.rs`：不用改（尾单元重写、`payload_of_the_rewritten_tail_data_unit` 读旧字节的路径截到单元中间时已经走）。`crates/singlefs-harness/src/model.rs`：K8（`research/prompts/m4-trunc-r1-main-verification.md:40`）里模型对「是否重写尾单元」的取值，边界那一格由否改是（推的）。 | 每次边界截断：多写 1 个数据单元（32 KiB，每盘 2 槽）+ 罩着它的下段叶到根 + 上段这个文件那一串：每盘约多新占 4–6 槽（节点层数是推的，`out/costs.txt`）。 | 不超 W 的 24：截到单元中间那一格今天每盘新占 11 槽就钉在 `truncation_intent_batches_and_final_batch.rs`（`the_truncation_intent_newly_takes_eleven_slots_per_device_outside_the_allocation_record_tree`），边界在丙下与它同形。新的失败面：边界截断要读尾单元的旧字节，读不出就拒（今天边界截断不读它）。有快照时旧尾单元多进一条 deadlist（`.claude/kb/decisions/05-快照-空间记账机制.md:302`：「birth > previous_snapshot_txg」 判）。 |

### 1.5　推荐与依据

推荐甲，字段取 (T, K)，更新规则取「放了单元的截断才盖章」，整份覆盖写清零，写序那一条留着。依据：
- 零新槽、零长期占用，是三条臂里唯一不加写放大也不加永久增长的；
- 字段住记录，随 inode 树回退（`crates/singlefs-core/src/mount.rs:8200`）自动一致，不靠谱系过滤；
- 与用户选的方向一致，W2 列的历史在模型里到深度 5 无差异；
- 不翻用户 2026-10-08 的条款（乙要翻）。
弱处写在 W5：甲只记一对 (T, K)，一进 pwrite / punch 就不够。这条推荐的「推翻」：R2 的攻方在甲上找到一段 W2 里没有的、只用今天的写入口的历史，让模型红。

### 1.6　要交用户的岔路（不带倾向）

| # | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 1 | 载体：甲 / 乙 / 丙 / 甲加丙叠用 | ① pwrite / punch 入口会不会在里程碑十一之前落地（落了，W5 的重开发生；不落，不发生）；② 截断次数 n 是多少量级（乙长期占用 = 40 B × n × 2 份）；③ 边界截断占截断的比例（丙每次多 4–6 槽） | ① 主 agent 对里程碑十一的排期给一句；② 用 E109 / E110 形的计数模型（纯算术）按用户给的 n 量一遍；没做，我这里只有 10⁶ 次一个点 |
| 2 | 字段内容：(T, K) / (T, N) | 无可量的翻面观测：判法只用 K，N 多一个「截到的 size」的人读信息；两者同宽 16 字节，不能三样都存（装不下 20 字节） | 用户拍 |
| 3 | 整份覆盖写对字段：清零 / 抄上一版 / 放了单元也盖章（后者可去掉写序） | 写序那一条留不留：留则 pwrite 来时它自己先塌（K6 已记欠账）；去则要整份写也盖章，到 pwrite 仍不够（W5 模型） | 用户定「现在留不留写序」 |
| 4 | 要不要为这个字节新开 incompat 位：并进第一条布局线 / 新开一位 | 有没有不能重 mkfs 的、里程碑三之后写出的镜像要给旧读者挂（零值字节读成「没截断过」，新读者读旧镜像没问题；问题只在旧读者读新镜像：遇到截断过的文件的记录判该记录 EIO）；先例是「不加新 incompat 位，并进第一条布局线」（`.claude/kb/decisions/18-块里携带什么信息.md:581`） | 用户说这批镜像是否算已发布 |

## 2　W2　判法：读法 Z 多判的一句，与 K6 三条怎么并

### 2.1　判定

- 甲的那一句（1.3 里逐字写的那句）与 K6 三条是合取的第四条：四条都满足才认领。写序那一条要留：整份覆盖写变短再变长（H5b）只有写序罩得住，字段不起作用。
- 「只记最近一次」够不够：够，但「最近一次」要读成「最近一次放了单元的截断」，并且新值整个替换旧值（T 取这次、K 取这次，不取 min）。两个反例都在模型里量到：每次截断都盖章（`甲last`）在 H4 的中间态红；盖章但不替换（旧章不被新章盖掉）在 H8、H9 要红（推的：这两个历史里旧章的 K 比放掉的单元偏移大，旧章判不到它们；模型里没有单列这一臂，只量了替换的臂）。
- 为什么一对就够（推的，模型到深度 5 与之相容）：盖章之后，今天的写入口没有一个能在偏移 ≥ K 的位置留下出生 txg ≥ T 的活单元，除了整份写（它把 0..n−1 号全用新 txg 重写，旧章不再需要；放到 n 之外的单元由写序判死）。一旦出现「只写中间一段」的入口，这个前提就没了（W5）。
- 推翻它的现象：一段只用今天入口（整份覆盖写、顺序写、截断、变长、回退、unlink）的历史，使模型里「甲rel清」或「甲rel留」多认领或漏认领。我在深度 5、字母表 W（整份覆盖写成 0..8 字节）/ S（改 size 到 0..8）/ R（回退到前面任一状态）里穷举了 2 564 298 个状态，没找到（量的，一次抽样）。

### 2.2　每种历史逐条

模型用 2 字节净荷、4 个单元的文件（size 8），S2 的意思是「改 size 到 2」；真实尺寸下把 2 换成 32 634 字节、4 个单元换成 300 个即题面。下表「放」是被放掉而盘上字节还在的单元，「认」是该认领的单元。

| 历史（模型写法） | 放：满足什么 | 认：满足什么 | 甲那一句分不分得开 | K6（量的） | 乙 / 丙（量的） |
|---|---|---|---|---|---|
| H1 截在边界再变长：S2 S7 | 单元 1..299；偏移 ≥ P；出生 4；size 之内；偏移更小的单元 0 出生也是 4，写序放过（平局） | 单元 0；偏移 0；出生 4 | 分得开：T ≥ 5、K = 1；放的是「偏移 ÷ P ≥ 1 且 4 < T」，认的是「偏移 ÷ P = 0 < K」 | 红：多认领 3 个（单元 1、2、3） | 都绿 |
| H2 截到单元中间再变长：S3 S8 | 单元 2..299；出生 4；单元 1 在意图那次重写、出生 T > 4，写序已判死 | 单元 0（4）、单元 1（T，变长补零后再重写） | 分得开；这里甲与写序重复，都判死 | 绿 | 都绿 |
| H3 截到 0 再变长：S0 S8 | 单元 1..299；出生 4；单元 0 重写成声明长度 0（K3），出生 T | 单元 0；K = 1 | 分得开：K = ⌈0 ÷ P⌉.max(1) = 1，写序也判死，重复 | 绿 | 都绿 |
| H4 截到 N1 → 变长 → 截到 N2 > N1（落在洞里）→ 变长：S2 S5 S4 S8 | 单元 1..299；出生 4；第二次截断放 0 个单元 | 单元 0 | 分得开，条件是第二次不盖章（字段仍是第一次的 (T1, 1)）；若第二次也盖 (T2, ⌈N2 ÷ P⌉ = 2)，单元 1 偏移 ÷ P = 1 < 2，甲放过它 | 红（S5 多认领单元 1、2） | 乙绿、丙绿；甲last 红（S4 意图后多认领单元 1） |
| H5 截断后整份覆盖写成更长、再截短、再变长：S2 W8 S2 S8 | 单元 1..；出生 = 整份写那次的 Tw；偏移更小的单元 0 出生也是 Tw，写序放过（平局） | 单元 0（Tw） | 分得开：第二次截断放了单元、盖新章 (T2, 1)，Tw < T2；旧章 (T1, 1) 判不到它们（Tw > T1），所以新章必须替换旧章 | 红（S8 多认领单元 1、2、3） | 都绿 |
| H5b 整份覆盖写变短再变长（无截断）：W4 S8 | 单元 2、3；出生 4；单元 0 出生 Tw > 4 | 单元 0、1（Tw） | 甲不参与（T = 0）；写序判死 | 绿 | 乙「只在截断写墓碑、无写序」红（多认领 2、3）；其余绿 |
| H6 截断中间态（意图之后、批之前，包括批与批之间崩溃）：S2 的意图后 | 单元 1..；挂在外面，出生 < T，仍在 extent 树里 | 单元 0 | 分得开；size 那一条已判死，甲重复；遍历一侧按 size 滤（`crates/singlefs-checker/src/walk.rs:13615`），两侧一致 | 绿 | 都绿；乙要让 `crates/singlefs-checker/src/rebuild.rs:768`：「return Err(ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild);」 只对整对象墓碑成立，区间墓碑不能触发它（推的） |
| H7 截断后回退到截断之前那一版，再截再变长：S2 R0 S2 S8 | 回退后再截那一次放掉的单元，出生 4 | 回退目标那一版的单元 | 分得开，因为字段随记录回到 R_old（`crates/singlefs-core/src/mount.rs:8200`）；抛弃那段里出生的版本不在谱系上（`crates/singlefs-checker/src/rebuild.rs:717`） | 红（S8 多认领 1、2、3，与 H1 同一个缺口） | 都绿；乙靠墓碑的死亡 txg 不在谱系上被忽略（`crates/singlefs-checker/src/rebuild.rs:700`：「if txgs_on_the_lineage.holds(read_u64(record, TOMBSTONE_RECORD_DEATH_TXG_OFFSET)) {」） |
| H8 截深、变长、再截回刚才的边界、变长：S4 S8 S2 S8 | 单元 1（第二次放的）、2、3（第一次放的）；出生 4 | 单元 0 | 分得开，条件：第二次盖 (T2, 1) 替换 (T1, 2)；留着 (T1, 2) 则单元 1 偏移 ÷ P = 1 < 2 放过 | 红（多认领 2、3） | 都绿 |
| H9 截到边界、变长、再截到更小的边界、变长：S6 S8 S2 S8 | 同上，K1 = 3，K2 = 1 | 单元 0 | 同 H8 | 红（多认领 3） | 都绿 |

H1 到 H9 在 `out/named.txt` 的原样输出里按「臂」分行，此表是它的整理。深度 4 的穷举（十种臂全跑）在 `out/search-depth4.txt`：K6 有 1 804 条最短不同的历史，甲last 302 条，乙「截断写墓碑、无写序」6 730 条（都是整份覆盖写变短再变长，H5b 一类），其余臂 0 条。

### 2.3　甲与丙、乙的差别在这一格的表现

- 丙在 W2 全部历史上 0 条不同（量的）。它的工作方式是让尾单元的出生 txg 变成放掉的单元里最大的，于是写序那一条判得死；它不引入新条款，但把正确性押在「每次放单元的截断都重写尾单元」这一处。
- 乙把所有历史上的每一次截断都记成一条独立记录，不存在「最近一次」够不够的问题；H5b 里它也需要写序那一条，除非整份覆盖写变短也写区间墓碑（`乙全释放无写序` 一行，量的，0 条不同）。

## 3　W3　快照、回退与删除

### 3.1　快照下截断

- 机制不变：截断换下的单元经 「deadlist_changes_of_this_publish」（`crates/singlefs-core/src/transaction.rs:5038`）按「取诞生代号 + deadlist 模型（ZFS 的 birth txg + deadlist）」（`.claude/kb/decisions/05-快照-空间记账机制.md:299`）判：「birth > previous_snapshot_txg」（`.claude/kb/decisions/05-快照-空间记账机制.md:302`）的直接释放，否则进活头的 deadlist。
- 今天扫描重建按哪一版的记录判（读码的）：只按「谱系上出生的版本里取最新」的那一版 inode 叶容器（`crates/singlefs-checker/src/rebuild.rs:720`：「let Some(version) = latest_of_the_ordered_versions(born_on_the_lineage)? else {」）里的记录判，不读快照那一版。快照共享的叶被头 COW 重写之后，头的叶是同身份的新版本，旧版本被取代；遍历一侧也不收最新根 deadlist 点名的映射条目（`crates/singlefs-checker/src/walk.rs:13679`：「mapping_keys_named_by_the_deadlist_of_the_newest_root」）。于是被快照钉住、头已放掉的单元，在重建看来就是「头放掉的单元」，按头的记录和字段判死；两侧一致。
- 三条臂各自：甲的字段只在头的记录里，快照那一版的旧记录没有它、重建也不读它，不需要为快照多写什么；乙的区间墓碑在墓碑树，同样与快照无关，但将来回收要照「墓碑记录随快照 deadlist 同一套节奏清」（`.claude/kb/decisions/18-块里携带什么信息.md:238`），而回收门是承重的（「墓碑不按门回收时旧版本复活——回收门承重」，`.claude/kb/decisions/18-块里携带什么信息.md:246`），乙要多立一道区间墓碑的回收门；丙在有快照时，被重写掉的旧尾单元如果出生 txg ≤ 上一个快照的 txg，就多进一条 deadlist、多钉一个 32 KiB 单元到快照销毁（同一个快照间隔里第二次边界截断时尾单元已是新版，直接释放）。
- 推翻它的现象：一段「建快照 → 截断 → 变长」的历史，重建把快照钉住的单元认回（这条我没有建模，模型里没有快照一项；读码得出重建不读快照的叶，是推的）。

### 3.2　回退（K1 回退那一段）

- 字段跟着退，是构造出来的：记录随 inode 树取 R_old（`inode_record: target_version.inode_record,`，`crates/singlefs-core/src/mount.rs:8200`；`inode_leaf_containers` 同取 R_old，`:8215`），字段在记录里，没有第二份要同步。
- 「留着」是错的（推的，没量）：若实现把现行版本的字段并回 R_old 的记录，R_old 里活着、出生 txg 小于现行 T 的单元全被判死，I-1.13 红在「重建少认」一侧。写死历史第 8 步回退到的是第 7 步的意图那一版（那一版自带字段），分不出「跟着退」与「留着」，必红世界要另加一步「回退到盖章之前的版本」。
- 乙的墓碑树随现行（`tombstone_containers: current.tombstone_containers.clone(),`，`crates/singlefs-core/src/mount.rs:8206`），回退抛弃段里写的区间墓碑还在树里，靠死亡 txg 不在谱系上被忽略（`crates/singlefs-checker/src/rebuild.rs:700`）。模型里这一过滤是 H7 乙绿的原因（量的）。I-9.20 现在对「不论在不在谱系上」的每条墓碑都要求整对象（`crates/singlefs-checker/invariants/09-inode树结构.md:32` 的「第一版只许整对象 [0, 2⁶⁴ − 1]」），乙要改它。
- 回退到截断中间态那一版：截断条目取 R_old（`research/prompts/m4-trunc-r1-main-verification.md:33` 的「管理员回退时 logged_ops 按种类分来源」一句），续做把挂着的单元放掉；甲的字段在 R_old 的记录里，已经是意图那次盖的章，不用再写（模型里 `R` 的实现就是这样，量的）。

### 3.3　unlink

- 整对象墓碑压住 (inode, 对象出生代) 的全部单元（`crates/singlefs-checker/src/rebuild.rs:766`：「if tombstoned_objects.contains(&(inode, object_birth)) {」），字段不参与。字段随记录抄到各批，最后一批删记录时一起没了（「最后一批释放剩下全部单元、删 inode 记录、删 deleted_inodes 那条 key，同一次发布」，`.claude/kb/decisions/08-核心索引结构.md:482`）。号重发后的新对象对象出生代不同，旧对象的单元过不了第一条（对象出生代相同），字段对它不适用。
- 乙：整对象墓碑与区间墓碑并存无害，但 I-9.19（谱系上的墓碑指认的 (号, 对象出生代) 在 inode 树里有记录时，那条记录的 nlink = 0，`crates/singlefs-checker/invariants/09-inode树结构.md:31`）会把每个截断过的活文件判红，要限定在整对象墓碑上；GPU 扫描内核同一条在 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:4262`。
- 丙：无交互。

### 3.4　整理（compaction）

- 不改出生 txg（`.claude/kb/decisions/26-后台整理与放置回收.md:123`、`:124` 的「目的地那条分配记录的分配代写单元的诞生代（不写搬迁那次发布的 txg）」；代码 `crates/singlefs-core/src/transaction.rs:7846`），所以四条判据里的「出生 txg 比较」搬过之后照旧；搬迁只搬已分配（含被快照钉住）的单元，放掉的字节不搬。
- 承重：整理一旦改出生 txg（比如搬完盖新戳），被快照钉住而头已放掉的单元出生 txg 会越过 T，写序、甲、乙、丙全部要重判。现在没有任何一条检查直接钉「搬迁不改 birth」对扫描重建的影响，只有分配代 = 诞生代那条（I-3.10，D26 :124 提到）间接钉；我没有再查这一条的实现（没做）。

## 4　W4　每一处要改的地方

行数全是推的（没有编译、没有试改）；「抄」指机械地带过去，不算判断。`grep -rn 'InodeRecord {' crates --include=*.rs`，去掉定义、impl 与 `fn inode_record` 之后是 19 处构造（`crates/singlefs-core/src/transaction.rs` 6 处、`records.rs` 3 处、`inode_tree.rs` 2 处、`crates/singlefs-checker-tier/src/crash_injection.rs:6623` 1 处、harness 用例 7 处），给 `InodeRecord` 加字段，这 19 处都得补。

### 4.1　核心层

| 处 | 甲 | 乙 | 丙 |
|---|---|---|---|
| `records.rs`（`InodeRecord` 结构 :50、`to_bytes` :85、`parse` :93；`TombstoneRecord` :137） | 加两个字段，`to_bytes` / `parse` 按 1.3 的表拆段，`parse` 判 T = 0 ⇔ K = 0；单测补一行「T ≠ 0 而 K = 0 拒」「136–139 非零拒」。约 +25 | 加按尾部区间构造墓碑的入口 +10；`InodeRecord` 不动 | 不动 |
| `truncate.rs`（:94–110 截断意图重写尾单元的判法） | 加盖章的纯函数（参数：已存在单元数、目标 size、上一版 (T, K)、本次 txg）+ 单测钉绝对值，约 +25 | 不动 | 去掉 :104 的声明长度条件；原单测里「边界不重写」那几行改成「重写」，约 −3 +10 |
| `transaction.rs`（19 处构造里 6 处在此；意图那次 :8600；unlink 意图与批 :8443、:8506；整份写 :7314） | 6 处补字段（整份写清零，unlink 抄，变长与 N = S 抄），意图那次调盖章函数，约 +25 | `DeletionStepOfThePublish::TruncationIntent` 带一条墓碑，复用 unlink 意图的墓碑路径（:8451），publish_deletion_step 里按「有墓碑」分支，墓碑树与容器的重写角色、journal 点名项、准入的需求都要把它算上，约 +80（推的，最大不确定处） | 不动（`payload_of_the_rewritten_tail_data_unit` 路径截到中间时已走） |
| `admission.rs`（:384 的 W 上界 24） | 不动：新槽 0 | 要核一次意图那次多出的约 3 槽仍在 24 之内（单测钉绝对值），并核「墓碑预留额」（D3（空间分配） 已定项 2 对墓碑的预留）是否按截断也预付，约 +15 | 不动；边界那一格的槽数用例（`truncation_intent_batches_and_final_batch.rs` 里 11 槽那条）多一个边界用例，约 +20 |
| `mount.rs`（回退 :8200、:8206）、`mounted_read.rs`（:544 `InodeRecord::parse`、:215 注释）、`recovery.rs`（:5370） | 不动代码，注释 :215 改字；回退自动一致 | 回退要靠谱系过滤（不改代码）；`crates/singlefs-core/src/mount.rs:8206` 不动 | 不动 |
| 其余 13 处构造的测试与辅助（harness 用例、`records.rs` 与 `inode_tree.rs` 的单测、`crates/singlefs-checker-tier/src/crash_injection.rs:6623`） | 每处补两个字段，约 +15 | 无 | 无 |

### 4.2　池级 checker

| 处 | 甲 | 乙 | 丙 |
|---|---|---|---|
| `crates/singlefs-checker/src/lib.rs`（常量，:785 旁） | 加 T、K 两个偏移常量，+4 | 加区间墓碑要用的常量（已有区间起止偏移，几乎不加），+2 | 不动 |
| `rebuild.rs` 认领（:778、:783）与白名单（`ITEMS_THE_REBUILD_MAY_USE` :827，:847 旁） | 认领加那一句（读记录偏移 120 / 128），白名单加两项，+10 | 墓碑那一段（:697 的 Err 改成认区间，:700 谱系过滤保留）和认领处的区间判，白名单加项，+35；`DeletionInProgressNotWrittenForTheScanRebuild`（:768）改成只对整对象墓碑触发 | 不动 |
| `rebuild.rs` 单测（手造单元头世界，不开池） | 手造 H1 的头：单元 0 与单元 1 出生同为 4、记录带 (5, 1)，认领 {0}；再各造 H4（不盖章）与 H8（替换）世界，+60 | 手造同样几个 + 回退抛弃段墓碑，+60 | 手造「尾单元出生 txg 大于放掉的」世界，+20 |
| `walk.rs` I-9.7（:3697，今天是 108 到 140 全零的一个 `all`） | 改成 108–119、136–139 恒 0 加两段合法性，+12；新判「T ≠ 0 时存在的单元里偏移 ÷ 净荷 ≥ K 的出生 txg 都 < T」（需要 extent 树里数据指针的出生 txg，遍历一侧本来就读），+40 | I-9.19、I-9.20（:11549 一带 `self.range_start == TOMBSTONE_WHOLE_OBJECT_RANGE_START`）放宽到「整对象或尾部区间」，I-9.19 只对整对象墓碑要求 nlink = 0，+30 | 不动 |
| `walk.rs` I-1.13 遍历一侧（:13615 一带的 size 滤） | 不动：放掉的单元不在 extent 树里 | 不动 | 不动 |
| 不变量清单（`crates/singlefs-checker/invariants/`） | `crates/singlefs-checker/invariants/09-inode树结构.md:19` I-9.7 改字；加一行新判；`crates/singlefs-checker/invariants/01-块自描述.md:18` I-1.13 与 `:14` I-1.9 里「被压住」的定义补第四类 | I-9.19、I-9.20 行改字；I-1.13、I-1.9 补区间墓碑 | 不动 |

### 4.3　GPU 判器

- 甲：I-9.7 在两处：检查内核 「w_judge(INVARIANT_I_9_7, all_zero(p.base, ro + 108u, 32u));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3189`）和恢复内核 `if (!all_zero(leaf.base, ro + 108u, 32u)) {`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3753`，读到非零就 `walk_failed`）。两处都得改成「108–119 与 136–139 恒 0、T / K 合法」，否则带字段的合法镜像在 GPU 上要么违例要么恢复失败，对拍必红（推的）。各 +8。新判（T、K 与单元出生 txg 的交叉）CPU 一侧立，GPU 内核不置位、占覆盖表末尾一个下标，照 S4 的先例（`research/prompts/m4-trunc-r1-main-verification.md:84` 「I-1.13 留在内核不置位」）。I-1.13 本身不上 GPU（S4）。
- 乙：墓碑扫描内核 I-9.19、I-9.20（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:4175-4262`）放宽，约 +30；恢复内核不动（墓碑不进恢复）。
- 丙：不动。

### 4.4　理想模型、RefFS、layout 表

- 理想模型 `crates/singlefs-harness/src/model.rs`：今天按单元号记出生 txg（`pub data_unit_written_at: Rc<Vec<ModelCheckpointTxg>>,`，`:1077`）。甲要多记「最近一次放了单元的截断」(txg, K) 随发布种类更新，并在对拍里比，约 +40（推的）；乙要多记墓碑多重集，约 +30；丙只把截断意图「是否重写尾单元」边界那一格由否改是，约 +3，模型对丙是最便宜的。
- RefFS：三条臂都不动（读者答案只看内容；截断后变长读 0 与洞同）。pwrite / punch 才要动（W5）。
- layout 表：`.claude/kb/layout/` 下没有里程碑四那一份（现有 01、02、03），甲要在 `04-fourth-txn.md` 登记「inode 记录偏移 120 / 128」并把 `.claude/kb/layout/01-first-txn.md:270` 那一行（「inode 记录（偏移 108 / 112 / 120）」）的「恒 0」写成只剩 108–119、136–139；乙登记一条区间墓碑的样例字节；丙只登记「边界截断重写尾单元」。

### 4.5　变异与必红世界：「改坏那一句，写死历史第 2 步镜像 I-1.13 红」怎么造

写死历史今天在第 2 步红（题面），也就是「把新那一句整个删掉」这个变异已经由今天的树自己立着；加上那一句之后，下面这些变异各自要红在指定的历史上。
- 甲：① 删掉认领处新的一句 → 写死历史第 2 步红；② 意图那次不盖章（盖章函数恒返回上一版） → 同一处红；③ 去掉「`data_units_existing > K`」这个条件、每次截断都盖章 → 写死历史分不出（第 3 步选择子是 0，截到的位置就是第 1 步的 N，`crates/singlefs-harness/src/history.rs:651-658`），要在写死历史里加一步选择子大于 0 的 `BackIntoTheHole`（H4），或靠随机采样；④ 盖章但不替换（取旧 K、旧 T） → H8 形的一步（截深、变长、再截回）；⑤ 回退把现行字段并回 R_old → 写死历史第 8 步分不出，要加一步回退到盖章之前的版本；⑥ 比较符 `<` 改 `<=` 或 K 差一 → rebuild.rs 单测里手造「出生 txg 恰为 T、偏移恰为 K」的头。
- 乙：删掉区间判；意图那次不写墓碑；死亡 txg 取对象出生代；去掉谱系过滤（H7 红：回退抛弃段的墓碑把 R_old 里活的单元压死）。
- 丙：把 :104 的条件放回去 → 写死历史第 2 步红；另加 harness 绝对值用例「边界截断后尾单元的出生 txg 等于意图那次的 txg」。

## 5　W5　以后的写入口（pwrite、punch，C778 挂里程碑十一）

`.claude/kb/milestone/04-fourth-txn.md:118` 写着「punch（不改 size 的部分删除）不在这一份」，C778 去向里程碑十一。K6 自己记了前提：「前提是活单元的出生 txg 随偏移不降」（`research/prompts/m4-trunc-r1-main-verification.md:38`）。下面模型里 pwrite 是「只重写第 i 个单元、别的不动、size 不到就撑到它为止」，punch 是「只放掉第 i 个单元」；COW 下单元内部分覆盖也是重写整个单元，所以同形（推的）。

### 5.1　每条臂在 pwrite / punch 进来时要不要重开

| 臂 | 要不要重开 | 模型里的表现（量的） | 重开要加什么 |
|---|---|---|---|
| 丙 | 要。它的保证全站在写序那一条上，写序先塌 | `K6`、`丙` 都在最短的历史 `PW0` 一步就漏认领单元 1、2、3（`out/w5-depth3.txt`）：pwrite 把偏移 0 的单元写成新 txg，偏移更大的活单元就被写序判死 | 去掉写序之后丙一条保证都没有：它只在「放掉的单元出生更早」时有用，需要另一条臂接手 |
| 甲 | 要。一对 (T, K) 是一个后缀水位，表达不了中间一段，也表达不了「先放、后在洞里写、再放」的两段不同 T | 去掉写序、整份写变短也盖章的甲（`甲rel覆盖也盖章无写序`）在 `W0 PW2 S3` 的截断意图之后的中间态多认领单元 1（深度 4 共 1 436 条不同；加上 punch 共 26 068 条，`out/w5-depth4.txt`） | 阶梯：一串 (T_i, K_i)，记录里 20 字节只装一对，要换处放，换到别的树就是乙的形态 |
| 乙 | 不用改字节格式（40 字节记录里本来有区间起止），要加四件，见下 | 去掉写序、每种释放（截断、整份写变短、punch）都写区间墓碑，pwrite + punch 到深度 4（423 565 个状态）0 条不同（`乙全释放无写序`，`out/w5-depth4.txt`） | ① punch 写 [a, b) 区间；② 整份覆盖写变短也要写区间墓碑（这是去掉写序的代价，整份写路径多一条墓碑）；③ 立区间墓碑的回收门（「墓碑不按门回收时旧版本复活——回收门承重」，`.claude/kb/decisions/18-块里携带什么信息.md:246`）；④ D18 已定项 10 里「punch 这类不改 size 的部分删除怎么记，判决没涉及、没定」（`.claude/kb/decisions/18-块里携带什么信息.md:240`）改成定；I-9.19、I-9.20 与 GPU 扫描内核按 4.2、4.3 放宽 |

那时仍成立的只有乙（加上面四件）。甲与丙要加的东西，加完就是乙的形态或乙加另一条保证。若用户判断 pwrite / punch 一定进里程碑十一，现在选甲或丙是在给里程碑十一欠一笔重做。
- 推翻它的现象：把 pwrite 的模型换成真实入口（单元内部分覆盖、跨单元写）后，`乙全释放无写序` 出现不同的历史；或者发现 punch 之后再 `Truncate` 的真实路径不写区间墓碑也认得对。这两样我都没有建模到入口层。
- 没量：punch 之后 size 不变的文件上，walk 一侧是不是还能靠 extent 树里缺席表示洞（`.claude/kb/decisions/08-核心索引结构.md:419` 的 extent 树一条写着「洞就是缺席」）；若能，乙那一半对得上；这是读码得来的、没量。

## 6　量的、推的、复核不了的

- 量的：`research/prompts/m4-trunc-r2-sonnet-model/out/` 下六份，`tools/rerun.sh` 在新目录经 2G 包装复跑，六份 `same_as_model_out=yes`（`~/.local/share/singlefs/work/m4-trunc-r2-sonnet/rerun-1/`）。校验路径会红：K6 一臂复现题面（H1 多认领 3 个单元），甲last、乙「截断写墓碑、无写序」两臂各被穷举抓到最短历史；模型只有一份，我没有第二个独立模型，两条「路径」（读码推出的 H1–H9 与穷举）共用同一个对「谱系、写序、尾单元」的理解，一致不算独立证据。
- 复核不了：① 写死历史第 2 步今天红（我没有跑测试，红的事实来自题面和模型 H1 的同数）；② 真实的 `data_units_existing` 在意图那次发布里是不是拿得到（`crates/singlefs-core/src/transaction.rs:8593`：「u64::try_from(previous.data_pointers.len()).expect("单元数")」（`publish_size_change` 里已有这一句），没编译）；③ 跨崩溃重发 txg 的情形（D8 已定项 6 射程写的「跨崩溃不成立：崩溃后 checkpoint 号会被重发」）下，`出生 txg < T` 只比 txg 数字、不比实例代号，与 K6 写序那一条同一处暴露，我没有建模；④ GPU 一侧的覆盖表下标与对拍，没起；⑤ 各格的行数估计。
- 推的：所有「行数」「每盘约多新占 N 槽」「乙在 110 025 条之后长高每次多一层」。

## 7　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W1 载体 | 甲可落地 | 记录偏移 120 起放 T、K 各 8 字节，意图那次放了单元才盖章，零新槽；incompat 属性按 D8 已定项 6 自己写的；乙多罩 W5 但要翻 2026-10-08 的条款并永久增长；丙零格式改动但每次边界截断多约 4–6 槽、押在写序上 |
| W2 判法 | 甲分得开 | 读法 Z 第四条 = 偏移 ÷ 净荷 ≥ K 且出生 txg < T 不认领；只记最近一次够，条件是「最近一次放了单元的截断」且新值整个替换；写序留着；十种历史、深度 5 的穷举在模型里 0 条不同 |
| W3 快照、回退、删除 | 一致 | 重建只读头的当前版本记录；字段随记录回退；unlink 抄；整理不改出生 txg；乙的正确性押在谱系过滤，丙在有快照时多钉一个旧尾单元 |
| W4 改哪里 | 列了 | 甲：records.rs、truncate.rs、transaction.rs、rebuild.rs、walk.rs（I-9.7 与新判）、两处 GPU I-9.7、model.rs、layout；约 +270 行（推的）；乙约 +300 且最大不确定在 transaction.rs；丙约 +50 |
| W5 以后的入口 | 都要重开 | 丙随写序先塌，甲的一对不够，乙加四件后在模型里成立 |

## 8　没做什么

- 没有起 cargo、没有编译、没有跑任何测试（含写死历史那条红的用例）、没有起 GPU、没有跑门禁阶段；本机在跑别的会话的崩溃点重放。
- 没有改 `crates/` 的任何文件，没有改 kb；写了的只有本报告、`research/prompts/m4-trunc-r2-sonnet-model/` 和草稿目录。
- 没有读攻方腿这一轮的产出（`research/prompts/m4-trunc-r2-opus-output.md`、`research/prompts/m4-trunc-r2-opus-model/`）。
- 不替主 agent 采纳或出判决：1.5 的推荐是正推腿的依据，1.6 的岔路表不带倾向。
- 模型是单对象、单实例、单元级的；没有建模快照、跨崩溃重发 txg、多文件、加密、副本落点；W3 里关于快照的结论是读码推的。
- 「没找到不同的历史」都只抽了一次样（穷举一遍），照 `.claude/rules/three-way-inference.md:142`：「一条腿只抽一次样不算一次观测」那一节，不能单独支撑「甲罩得住」。

交回前 `python3 scripts/agents/cite-check.py` 的末行：  ✓ 核了 37 处引文、38 处裸行号，对上 37 处，没判 38 处（其中裸行号没有引文可比 38 处；逐处列在上面）
