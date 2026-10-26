# m4-publish-trigger-r1 云端正推腿报告（2026-10-26）

立场：从 D16 各分项、截断与删除的同事务约束、`crates/` 今天的实现（冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen`）推每一格的候选与它要满足的条件，每格给推荐与它依赖的前提；P4 数出假设「返回即落盘」的流与用例。这一份没读攻方腿的任何产出。

## 复跑

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 3G bash research/prompts/m4-publish-trigger-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

自己在新 shell 里整份跑过一次，退 0；末尾原样输出：

```
name=refs_selftest broken_python_differs_from_real=4745 of 32152
name=refs_real_vs_python lines=32152 identical=1
name=same_as_stored file=count_assumptions.txt
name=same_as_stored file=model_window.txt
name=same_as_stored file=model_dirty.txt
name=same_as_stored file=model_firstcross.txt
name=same_as_stored file=refs_cases_summary.txt
name=same_as_stored file=cases.sha256
name=rerun_summary mismatches=0
```

模型目录里每个文件的 sha256（`sha256sum`，在仓根起）：

```
f9553acf640fbb281862c88ceba6c2f6b40ff971c0b17eb6be6edc29d25dddcf  research/prompts/m4-publish-trigger-r1-sonnet-model/tools/count_assumptions.sh
88643c000eb68438c55f9613a0ce3c7a7bfe517791d1d18009f8ea0d21b9e248  research/prompts/m4-publish-trigger-r1-sonnet-model/tools/rerun.sh
5f92df7521a466c104423b098827958856cbc26cec9256be5d00e3c99ea9ed76  research/prompts/m4-publish-trigger-r1-sonnet-model/model/trigger_model.py
c5e70e6c10d106d4ef6a5767f7a0e86cb989de32d7fd5adadad7e5eef9cb09ad  research/prompts/m4-publish-trigger-r1-sonnet-model/model/refs_cross.rs
3379d38254b319bfe5b5521c138694b6c80290e5436e902434b0ca6cc4a68e49  research/prompts/m4-publish-trigger-r1-sonnet-model/out/cases.sha256
aa468c4362beb58a60630d8bf6bedfa16ec91ec50d0e10b1181a5a6779f2fa5a  research/prompts/m4-publish-trigger-r1-sonnet-model/out/count_assumptions.txt
b73400a681991d2210f03802a52885b52d143050d8465760c39549232ede96df  research/prompts/m4-publish-trigger-r1-sonnet-model/out/model_dirty.txt
93679c5c5fbb48308df3a6bdca274ac96bcce38d75e7c4e0901496fa57a32d91  research/prompts/m4-publish-trigger-r1-sonnet-model/out/model_firstcross.txt
bbdd57928b00176e29f44214a2745590483c8e84ac90e7e5eb17e2a334c0657e  research/prompts/m4-publish-trigger-r1-sonnet-model/out/model_window.txt
dba41852a5a194198c189543b5441349fce75d40e50e0369c72d1e350d6922d9  research/prompts/m4-publish-trigger-r1-sonnet-model/out/refs_cargo.txt
124cebe4fb68ff08a32c198edb2950c4e88ce459ccaf756fb346949c5ff7ca30  research/prompts/m4-publish-trigger-r1-sonnet-model/out/refs_cases_summary.txt
6de697b5f37229700c2feb41d425715300f812967a91ca05a2eb6b2d684314b2  research/prompts/m4-publish-trigger-r1-sonnet-model/out/rerun_stdout.txt
```

模型三样互不共用代码：命令行计数（grep/awk）、Python 小模型（触发与脏量与 RefFS 允许集合的口径）、真 RefFS（冻结副本里的 `singlefs-reference-filesystem`，Rust，只依赖 std）。三万多条用例两边逐条判，判相同；把 Python 口径弄坏（发布不抬下界）后 4745 条对不上，所以「相同」不是空话。

## 推荐的条款集（各格的细节在后面，这里只列名字）

- 〔入口〕会话在原 `publish_user_change` 旁新增三个入口：收数据写类改动但不立刻发布的 `accept_data_change(change, now)`，调用方按周期调的 `publish_when_due(now)`，立刻排空的 `publish_pending_now()`；原入口语义不变（同步发布），只多一条「先排空待发」。
- 〔触发式〕触发 ⟺ 待发非空 ∧（now − 上次发布的读数 ≥ T_time ∨ 待发脏量 ≥ T_dirty）；accept 进门后判一次，`publish_when_due` 里判一次；`now` 全由调用方给，核心层不读钟。
- 〔脏量〕脏量 = 折叠后待发写的「将写数据单元字节」；T_dirty 有效值照 D16 已定项 5 字面夹取。
- 〔栅栏〕删除类、截断、建删快照、整理、回退、推环、卸载先排空待发；排空一次发布全部待发，不切前缀。
- 〔开关〕阈值开关装在分配器上、只住内存，产品挂载入口恒取有效值。
- 〔RefFS 接法〕RefFS 本体不改；翻译把会话报告的每次发布翻成一个显式发布点。
- 〔窗口公式〕D16 已定项 5 的丢失窗口加两项：调用方轮询周期 P、发布耗时 d。

## P1　时间那一支由谁判、空闲时怎么触发

**只在请求进门时判，空闲负载下窗口无界，打中已定项 2 的依据。** 依据原话：「只按脏数据量，空闲系统上最后一批写可以无限期不落、崩溃丢窗口无界」（`.claude/kb/decisions/16-发布语义.md:112`）。触发式是「触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)」（`.claude/kb/decisions/16-发布语义.md:106`），式子里没有谁在什么时候求值。今天核心层没有钟：`grep -rn 'Instant::now' crates/singlefs-core/src | wc -l` 得 0，`SystemTime`、`thread::spawn` 也是 0（`out/count_assumptions.txt` 的 C1 一段）。只在进门求值时，最后一条改动之后再没有请求，条件没人判：模型 `name=window_entry_only pending_after_idle_forever=2 publishes=0`。

**推荐：两处求值，钟由调用方给。**
- accept 进门、把这条改动收下之后判一次；
- 调用方按自己的周期调 `publish_when_due(now)`，里面判一次。

契约（对照已有先例写，不另造）：
- `now` 的类型直接用「pub struct CallerMonotonicClockReading(pub Duration);」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:991`），文档已写「核心层不起线程、不读钟」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:989`）。
- 钟往回走按没过间隔算，沿用「Some(last_check) => now.0.saturating_sub(last_check.0) >= interval,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:745`）；模型 `name=clock_backwards poll_at_100=None`。
- 守门：待发非空才判。没有守门时，空闲且无待发的会话在 100 个 T_time 里发 99 次空发布（`name=idle_publishes guard_nonempty=False publishes_in_100_t_time=99`），有守门是 0。已定项 9 说「空发布也是发布」（`.claude/kb/decisions/16-发布语义.md:250`），空发布要重写记账行与连带的单元，不是免费的。触发式本身没写这道守门，是条款空白，要补进已定项 2。
- 「上次 checkpoint 的时刻」：会话第一次见到的读数当基线，之后每次发布取触发那次调用交进来的 `now`，不再为「发布完成」多读一次钟。已定项 5 的「在重发时取重发那一刻」（`.claude/kb/decisions/16-发布语义.md:164`）与这条同向。基线若改成挂载入口给读数，空闲很久之后的第一笔写会在 accept 里立刻触发（`name=baseline mount_reading_given=True ... triggered_at_accept=time`）；两种基线都不破窗口上界，取前一种少发一次。
- 返回值要说清是哪一支触发（时间、脏量、两者）和发了哪些版本，给测试和 RefFS 翻译用。

**丢失窗口多一项。** 已定项 5 的式子是「由此崩溃最多丢的是 T_time + N_switch × (T_retry + T_redo)，不是恰好 5 秒」（`.claude/kb/decisions/16-发布语义.md:164`），没有调用方轮询周期。模型 2000 条随机排布的最长待发年龄 4997 ms，最坏排布（T_time=4001、周期=2000，上一次发布恰在一次轮询上，下一毫秒收一条改动）到 5999 ms，界 T_time + P = 6001 ms（`name=window_worst_layout ... age_ms=5999 bound_t_time_plus_period=6001`）。式子要加 P 和发布耗时 d 两项；今天的式子在 P、d 上不成立。

**与第 39 批「整理挪出调用线程」同形。** `run_compaction_in_the_background` 收 `&mut self`、在调用方自己的线程上跑（「pub fn run_compaction_in_the_background<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:2160`）），发布路径只记「该跑了」（「返回之前一批整理都不发（整理挪出了调用线程」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:1910`）），定时兜底读调用方交的读数（「.check_the_timer_fallback(pacing.timer_fallback, caller.clock_reading())」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:2205`））。时间入口同样收 `&mut self`，所以它和整理入口由借用互斥，不会并发，不需要锁。关系取三条：
- 复用读数类型，不复用 trait：`CallerOfTheBackgroundCompaction` 还带 `foreground_fsync`（「pub trait CallerOfTheBackgroundCompaction {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:1073`）），里程碑四没有这个对象；时间入口直接收 `now` 值，测试不需要 trait 对象。
- 一个调用方周期里先 `publish_when_due`、后 `run_compaction_in_the_background`。整理的发布建在 `current` 上，待发改动是逻辑改动而不是准备好的版本，所以整理入口发批之前要先排空待发（〔栅栏〕）。
- 实现排在第 39 批实现员交回之后（`mounted_session.rs` 在改）；这一轮只定条款。

**可控时钟下先到哪个（模型，`out/model_firstcross.txt`）：** 时间先到：脏量压在阈值下，T−1 处 `None`，T 处 `time`（`≥`）。脏量先到：时间阈值给足，钟停在 5 ms，第 4 次 accept 起 `cause=dirty`。同时到：`both`。

**依赖的前提：** 调用方愿意按周期调轮询入口（FUSE 层没有定时器就做不到）；当前没有 fsync 入口（`grep -rn 'fn fsync' crates/singlefs-core/src | wc -l` 得 0）。
**什么现象推翻它：** 找到核心层里已有的定时线程机制（现在 `thread::spawn` 零命中）；或用户要求不靠调用方轮询也保时间窗口，那就要核心层起线程，与「核心层不起线程」冲突，要另走一轮。

## P2　「脏字节」按什么算（C348）

C348 原话：「T_dirty 按什么算没有口径」（`.claude/kb/checks-owed.md:249`），两种口径都挂在 write buffer 前端上，里程碑四没有前端。没有前端时今天能算的是「已收下、还没发布」的写，下面五种读法逐个比（负载与读数在 `out/model_dirty.txt`）：

| 读法 | 同一文件 1 KiB 覆盖 1000 次 | 1000 个文件各 1 KiB | 一个 70000 B 文件写 3 次 | 10 万次建空 inode |
|---|---|---|---|---|
| 收下字节（不折叠） | 1 024 000 | 1 024 000 | 210 000 | 0 |
| 折叠后收下字节 | 1 024 | 1 024 000 | 70 000 | 0 |
| 将写数据单元字节（折叠后，取整到 32 KiB 单元） | 32 768 | 32 768 000 | 98 304 | 0 |
| 发布时弄脏节点字节 | 要读树，进门时算不出 | 同左 | 同左 | 同左 |
| 日志占用字节 | 单元数 × 4096 | 同左 | 同左 | 同左 |

单元数 32 768 B 来自「pub const DATA_UNIT_BYTES: u64 = 32768;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-format/src/lib.rs:20`），日志记录 4096 B 来自「pub const JOURNAL_RECORD_BYTES: u64 = 4096;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-format/src/lib.rs:241`）。

**第三种读法（今天能算的）算不算：算，取「将写数据单元字节（折叠后）」。** 理由三条：
- 它是内存里实占的上界。已定项 5 说 T_dirty「它就是脏页内存预算本身」（`.claude/kb/decisions/16-发布语义.md:162`）；实现若在后到的整份写盖掉先到的时丢掉旧缓冲，内存实占是折叠后收下字节，单元取整只会更大，用它当预算不会被突破。
- 同一个文件整份重写 1000 次，「收下字节」读 1 024 000，但这次发布只写 1 个单元（32 768）；按「收下字节」读会过早触发，按「折叠后收下字节」读又把 1 KiB 的文件少算成 1 KiB 而真写一个 32 KiB 单元。折叠的依据是 D8 已定项 13 的「落叶只做一次：叶里这个 key 的条目直接写成最终态 E」（`.claude/kb/decisions/08-核心索引结构.md:414`）。
- 它在进门时就算得出，不要读树；「发布时弄脏节点字节」要先算出这次发布的样子，进门时不可得。

**它与已定项 5 的夹取合不合：字面合，单位上有缝。** 有效值 = 「有效 T_dirty = min(系统配置里那个字段, 环长 ÷ F)」（`.claude/kb/decisions/16-发布语义.md:162`）。模型按冻结副本的常量算（F = 3 来自「pub const JOURNAL_SAFETY_FACTOR: u64 = 3;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-format/src/lib.rs:273`），默认环 805 306 368 B 来自「JOURNAL_RING_DEFAULT_BYTES = 805306368」（`.claude/kb/layout/01-first-txn.md:134`））：

```
name=effective_t_dirty field_bytes=2147483648 ring_bytes=805306368 factor=3 ring_div_factor=268435456 effective_bytes=268435456
name=effective_in_data_units data_unit_bytes=32768 units=8192 journal_bytes_of_those_units=33554432 in_flight_record_limit=65536
name=field_2gib_in_data_units units=65536 journal_bytes=268435456
```

有效值 256 MiB 是 8192 个数据单元，对应日志 32 MiB，只用到在飞记录上限 65 536 条的八分之一。而同一份文件里另一句说「T_dirty 满窗 65 536 条记录、256 MiB」（`.claude/kb/decisions/16-发布语义.md:171`）——65 536 条记录、256 MiB 日志是字段值 2 GiB 对应的，不是夹取后的值。两句并排：

| 句 | 原文 | 说的是哪个量 |
|---|---|---|
| 夹取 | 「有效 T_dirty = min(系统配置里那个字段, 环长 ÷ F)」（`.claude/kb/decisions/16-发布语义.md:162`） | 数据字节与环字节直接比 |
| 满窗 | 「T_dirty 满窗 65 536 条记录、256 MiB」（`.claude/kb/decisions/16-发布语义.md:171`） | 2 GiB 数据对应的日志字节 |

夹取式把数据字节与日志环字节放进同一个 min，单位不同；按 32 KiB 单元算，夹取比在飞记录上限松 8 倍的反向，即比需要的紧 8 倍。这是 C348 之外的新缝，应记欠账；里程碑四取字面（直接比），因为真正的硬界是单次发布的记录数上限（P3）。**判定：冲突（单位）。**

**脏量那一支在测试里跨不跨得了阈值，取决于读法，要写进条款的测试负载：**

```
name=dirty_branch_reachable reading=content threshold_bytes=65536 accepts_until_cross=64 cause=dirty
name=dirty_branch_reachable reading=folded_content threshold_bytes=65536 accepts_until_cross=never(10000) cause=None
name=dirty_branch_reachable reading=units threshold_bytes=65536 accepts_until_cross=never(10000) cause=None
```

折叠后反复整份覆盖同一个文件永远跨不了；测试负载必须写不同的 inode，或内容长到几个单元。建 inode 不占数据单元，脏量恒 0，只有时间一支会触发；但待发建 inode 的个数没有第二道界，10 万次建 inode 脏量仍是 0，所以要靠 P3 的「进门预判合并后能不能发布」作硬界。

**依赖的前提：** 里程碑四没有 write buffer 前端；会话待发队列以逻辑改动存、同 inode 的整份写后到盖先到。
**什么现象推翻它：** write buffer 前端启用（里程碑八）后 C348 的两种口径取代这一种；或用户定「脏字节按日志字节读」，则夹取式单位自洽而字段 2 GiB 要重新解释。

## P3　一次发布里攒什么

已有约束逐条对着今天的发布计划形状：

| 约束 | 出处 | 对攒批的要求 |
|---|---|---|
| 建快照独立发布 | 「独立发布」（`research/prompts/m4-snapshot-r1-main-verification.md:31`） | 建快照前先排空待发（那是一次发布），再独立发快照 |
| 快照点在 checkpoint 全部数据变更之后 | 「快照发布点必须与 checkpoint 边界对齐」（`.claude/kb/decisions/16-发布语义.md:322`） | 同上：待发里的写都在快照之前，不能留到快照之后 |
| 每次用户改动先走空 | 「先把现行那一版的 deleted_inodes 队列走空，再做它自己」（`.claude/kb/decisions/08-核心索引结构.md:509`） | 走空发生在批发布之前，一次，不是每条被攒的改动之前 |
| 删除与截断的最后一批与删意图同事务 | 「删除意图必须与最后一批工作在同一个事务里提交」（`.claude/kb/decisions/08-核心索引结构.md:122`） | 删除类每步自成一次发布，不并别的 |
| 意图那次、每一批各占一个事务号 | 「各占一个事务号」（`.claude/kb/decisions/08-核心索引结构.md:505`） | 同上 |
| 同请求内取号按 key 升序 | 「同一次请求切出的若干事务按其单元的 key 升序取号」（`.claude/kb/decisions/16-发布语义.md:164`） | 跨请求的取号序条款没写（规则没说）；全量排空时取接受序 |

**哪些能并进同一次发布。** 今天发布计划一次装一个文件的改动：「pub file: Option<FileVersionPlan<'content>>,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5780`），第一个文件之外的改动一次一个：「pub enum ChangeOfAFileOtherThanTheFirst {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:7247`）里的 `WholeWrite` 只带一个 inode（「WholeWrite { inode: InodeNumber, data_units: u64 },」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:7251`）），建 inode 的记录列表可以多条：「pub new_inode_records: &'content [InodeRecord],」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5784`）。`crates/` 里没有「同一次发布装两个文件的写」的入口（上面两行是凭据；计划函数一次收一个 `ChangeOfAFileOtherThanTheFirst`）。所以不改发布计划形状时：
- 能并：同一个 inode 的连续整份写（后到的盖先到的，一次 `publish_*`）；多次建 inode（个数相加，时间秒逐条保留在记录里）；一个文件的写加建 inode（计划两个字段都在，入口函数里没有，要新写一个装两样的）。
- 必须单发：建快照、删快照、unlink、截断、整理、回退、推环；不同文件的写（M4 内先各发一次，不改计划形状；改计划形状是要碰 `transaction.rs` 的另一条路，与第 39 批实现员撞文件，留后）。
- 不同文件的写各发一次时，一次触发排空是若干次发布，中间崩出的是「前几个文件的写已落、后几个没落」的状态，不是任何接受序前缀。RefFS 目前只有一个文件（「第一版的操作只碰一个文件：还没有文件，或者有这一个文件与它的全部字节。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:24`）），这个问题在 RefFS 里不出现，在多文件的模型对拍里要另判。

**把删除类并进别的改动，同事务约束的判器还分不分得出。** 机制上分得出：约束被违反的样子是某一版里「意图没了而目标没达成」，判器按版本逐态判，与那一版里还装了别的什么无关。现有变异也是按单独发布设计的，两条会红的：「最后一批不删 deleted_inodes 那条 key（删 key 不与最后一批同一次发布）」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/mutations.tsv:2766`）和「最后一批不同发布删意图」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/mutations.tsv:4210`）。我没有在副本上改坏重跑（变异表是重型，按定义不跑），所以「并了以后这两条仍红」是**复核不了**；能复核的是并了以后「每一版的重写角色集合钉绝对值」那一批用例会因角色集合变了而红，即合法改动误红。推荐不并，理由是这三条：同事务判器按单独发布设计、角色集合钉的是绝对值、预算按类算——「每一次被拒的删除类发布各算一次准入」（`.claude/kb/decisions/08-核心索引结构.md:509`），并进别的改动后一次准入里说不清这是哪一类。

**合起来对准入量与在飞记录上限怎么合算。**
- 准入是只读纯函数：「全是只读的纯函数：不动分配器、不发一个写。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/admission.rs:8`）。需求是这次发布新写的全部槽，不加换下的槽；合并后共享的祖先只写一次，所以合并需求不大于逐个需求之和，不会因合并而多拒。
- 一次发布的记录条数上限：「pub fn journal_record_limit_of_one_publish(journal_ring_bytes: u64) -> u64 {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:11913`）= 环槽数 ÷ F（默认环 65 536，上面模型第三行）。合并后的条数大约是折叠后数据单元数加 ⌈元数据角色数 ÷ 64⌉，8192 个单元的脏量远小于 65 536；小环的测试几何里才会先碰到。
- **新不变量：已 accept 的改动必须发布得出去。** 超过记录上限的错属于「推抬 F 腾不出记录的位置，要把改动切小」那一类（「推抬 F 腾不出记录的位置，要把改动切小」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:1729`）），空间准入不够则要推抬 F；若 accept 已返回成功而后来的排空被拒，调用方手里是一个失败的承诺。所以 accept 进门要预判「现行版 + 待发 + 这一条」能不能发布，不能就先排空（走既有的发布加推抬 F 路径，被拒时拒绝交给这一次调用的调用方）。预判要先算出这次发布的样子，今天的预演是在只进内存的盘与分配器拷贝上走的（「要出路时的预演在只进内存的盘与分配器的拷贝上走同一段」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:2279`）），代价高；**前提待核**：能不能做一个不拷分配器的只读预判，要实现员读 `prepare_the_version_publish` 才定，我没有展开。

**依赖的前提：** 里程碑四数据写类就是整份覆盖/顺序写（按 inode 号的同理）和建 inode，别的写类还没有；全量排空不切前缀。
**什么现象推翻它：** 把删除类并进别的改动后，崩溃流里出现「意图没了、目标没达成」的版本而判器全绿（并入后一定要补这一条用例与变异）；或发现同 inode 折叠让某个版本的内容与接受序前缀不一致。

## P4　攒批之后「返回即落盘」不再成立

**入口形态：** 已定项 11 的原话是「所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏。」（`.claude/kb/decisions/16-发布语义.md:283`），fsync 的效果是提前触发一次发布。里程碑四没有 fsync 入口（`grep -rn 'fn fsync' crates/singlefs-core/src | wc -l` 得 0），它的形态就是〔入口〕里的 `publish_pending_now()`：把全部待发改动做成一次发布，待发为空时返回「没有待发」、不发空发布（已定项 8 取戊「根覆盖两块盘之前不让 fsync 返回」那一套属于里程碑十对外 fsync）。失败的语义照已定项 17：「报错之后不许假设可以照旧复用同一份内存态重试。」（`.claude/kb/decisions/16-发布语义.md:371`）。发布冻结时（`PublishFrozenAfterAWriteFailureIsNotResentYet`，「只有 [`resend_the_frozen_publish`] 清得掉它」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:1471`））accept 与各入口在任何读写之前拒，待发已被那次冻结的发布消耗，不留第二份。

**还有一条不能漏：读路径看不到待发。** 挂载读入口收的是 `PoolReader`、读盘：「pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:656`）。accept 返回之后、发布之前，从盘上读不到这条改动。「返回即落盘」不成立的同时，「返回即读得到」也不成立；里程碑四没有 POSIX 层，所以条款上写成：待发期间盘上读不到待发内容，读者要么先 `publish_pending_now()`，要么由里程碑十的 FUSE 层在读路径上叠待发。这一句今天没有任何条款（规则没说）。

**数出来的（`out/count_assumptions.txt` 的 A 各段，全文在模型目录）：**

| 量 | 命令 | 数 |
|---|---|---|
| 会话入口 `publish_user_change` 的方法调用点 | `grep -rc '\.publish_user_change(' crates --include=*.rs` | 100 处、28 个文件 |
| 其中 harness 历史（`src/history.rs`） | 同上按目录分 | 8 处、1 个文件 |
| 其中 harness 用例 | 同上 | 72 处、18 个文件 |
| 其中 checker 档用例 | 同上 | 7 处、2 个文件 |
| 其中 checker 档装置（`src/bin`） | 同上 | 13 处、7 个文件 |
| harness 历史的操作种类 | `HistoryOperation` 枚举（「pub enum HistoryOperation {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/history.rs:503`））数变体 | 19 种 |
| 其中经会话发布的 | 含方法调用的 fn 对着 `apply_operation`（「fn apply_operation(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/history.rs:7687`））的分派 | 9 种：`PublishOverwrite`、`Unlink`、`CreateInode`、`CreateInodes`、`WriteOfAnInode`、`Truncate`、`CreateSnapshot`、`DestroySnapshot`、`OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable` |
| RefFS 翻译里「内容改动后紧跟显式发布点」 | `grep -c 'publish_with_content('` | 6 处（1 处定义、其余调用）；三处直接 `Operation::ExplicitPublish`（「operation: Operation::ExplicitPublish,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/reference_filesystem_translation.rs:364`）等） |
| 调用方读返回值里 `versions_published` | 同上 core 之外 | 185 处、25 个文件 |
| `roots_carrying_the_change` | 同上 | 14 处、2 个文件（`history.rs`、`reference_filesystem_translation.rs`） |
| 崩溃流 `crash_enumeration_*` 文件 | `ls crates/singlefs-checker-tier/tests/crash_enumeration_*.rs \| wc -l` | 34 个，其中含 `publish_user_change` 的 0 个 |
| 经会话的崩溃相关用例 | `grep -rl publish_user_change crates/singlefs-checker-tier/tests` | `common_crash_points/mod.rs`（一条「经会话 unlink」的流）与 `crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded.rs` |
| 随机历史入口 | `grep -rl 'run_history_campaign\|execute_history_handing_out_every_step_image'` | 6 个文件（`history.rs` 本身、`random_histories.rs`、3 个 checker 档用例、`e158_root_choice_repair.rs`） |

**两种做法对这些数的影响（这是 P4 的核心）：**
- 〔入口〕分开（原入口不动，新增 accept/轮询/立即排空三个入口）：上面所有「返回即落盘」的假设在原入口上仍然为真，**受影响的现有用例是 0 条**；要新增的是三个历史操作变体、翻译里三个新臂、一条「收多笔→排空→全状态扫崩溃」的新崩溃流、模型对拍的待发队列。
- 改原入口为攒批：100 处调用、28 个文件、9 种历史操作、185 处返回值读取、14 处根映射都要改，原来逐步对拍的 9 种操作整套失效。不推荐。

**判法怎么换（〔入口〕分开之下）：** 模型对拍点只放在发布之后；待发期间模型的「磁盘读」仍是上一次发布的版本。翻译对 accept 只交 `Operation::OverwriteTheFile`（结局成），不跟显式发布点；对会话报告的每次发布（轮询触发、accept 里触发、立即排空）翻成一个显式发布点，位置按调用里发布的先后。随机历史新增 `AcceptDataChange`、`PollWithClock`、`PublishPendingNow` 三种操作，以及一条「历史结尾必须排空」的规则：`CloseAndMountWritable` 与 `mount::unmount`（「pub fn unmount<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:4240`））之前待发必须排空。**这条要靠类型保证，不能靠自觉：** 会话的字段全公开（「pub struct MountedSession {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:90`）），卸载与回退是收字段引用的自由函数，历史里直接对 `session.current` 动手；待发队列放进会话就得把它放成私有字段，卸载与回退改成会话方法，否则有待发时卸载会静默丢掉待发。

**不加立即排空入口，「证明先到触发」还能不能验：** 能验触发本身（假钟加轮询，看会话 `current` 的发布号有没有涨，`out/model_firstcross.txt` 的四行），不能验的是崩溃判法：没有立即排空就没有「此刻起前面的都已落盘」的确定点，历史只能靠把钟拨过 T_time 来造发布点，随机历史里每一次要造发布点都得带一次钟前进；卸载前也排不空。所以这个入口是验收要用的，不只是对外 fsync 的预备。

**依赖的前提：** 原入口保持同步发布并先排空待发；会话的待发队列私有。
**什么现象推翻它：** 数出原入口以外还有调用者依赖「调用返回时盘上已有」且不经 `publish_user_change`（例如直接对 `session.current` 读盘），则〔入口〕分开也要改它们；`grep -rn 'session\.current' crates` 之类可以现查。

## P5　RefFS 怎么接自动触发的发布点

**今天会判红，而且是对合法恢复态判红。** RefFS 文件头：「只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。自动触发的攒批发布落地之前只接显式发布点（r6 判决第二节 U2 一行）。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:15`）。允许集合只认下界与显式发布末尾：「Q：j 只取 lo 与恰是某次显式发布末尾的下标」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:377`），内容改动不是发布末尾：「EnteredKind::ContentChange => false,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:232`）。翻译今天对每个内容改动都补一个显式发布点：「operation: Operation::ExplicitPublish,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/reference_filesystem_translation.rs:364`）。两句并排：

| 句 | 原文 |
|---|---|
| RefFS | 「自动触发的攒批发布落地之前只接显式发布点」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:15`） |
| D16 已定项 10 | 「攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。」（`.claude/kb/decisions/16-发布语义.md:267`） |

**实测（真 RefFS，3 万多条用例）：** 用例是「接受若干条整份覆盖、夹着调用方显式发布和会话自己发的发布，然后崩溃，恢复读出某个内容号」。真值取「发布返回即落盘」：恢复只能读最后一次完成的发布之后的内容；崩在一次发布里则前后两个都行。每条用例两个视角：把自动发布也翻成显式发布点（本方案），和只翻调用方点名的（RefFS 今天的接法）。真 RefFS 与 Python 口径在全部 32 152 条上判相同；统计（`out/refs_cases_summary.txt`）：

```
name=refs_cases cases=32152 truth_legal=11562 cur_false_red=3501 cur_missed_red=2084 auto_false_red=0 auto_missed_red=0
```

今天的接法：对合法恢复态判红 3501 条，把本该判红的放过 2084 条（自动发布已返回，恢复却读出更早的内容）；本方案两个数都是 0。最小例子是「接受第 1 号内容，自动发布一次，全部返回之后崩溃」：恢复读出 1 号内容是合法的，今天的 RefFS 判红；恢复读出 0 号（丢了已返回的发布）是非法的，今天的 RefFS 收下。

**落地后允许集合怎么定：** RefFS 本体不改——它的 Q 已经是「下界 lo，加上下界之后每个发布末尾」，缺的只是自动发布没有进操作序列。接法是在翻译里：会话在 accept、轮询、立即排空的返回值里报告「这次调用发了哪几个版本、先后」，翻译把每个版本翻成一个显式发布点，位置按先后；发布返回即抬 lo（自动发布与显式发布同一个返回条件，同 D16 已定项 7 的持久顺序）。崩在自动发布里面（`hi` 是那个发布）时允许集合含发布前后两个状态。`accept` 本身不写盘（待发只在内存），没有屏障，不会是崩溃所在的操作。

**由谁告诉 RefFS：** 会话的返回值（实现侧事实）加录制流对账。对账沿用 `roots_carrying_the_change` 那一套：报告了发布而录制流里没有对应的根，或录制流里多出根而没有报告，翻译判「实现与报告不一致」，不让 RefFS 吞掉。RefFS 不读盘、不读录制流，仍只收翻译交入的操作序列。

**全量排空是这个接法的前提。** 若一次触发只发布待发的一个前缀，该发布的状态是接受序里更早的 S_i，而 RefFS 的显式发布点状态是它之前最后一个操作之后的状态，对不上，需要新增一种「发布到第 i 个操作」。〔栅栏〕里规定排空发布全部待发，并在 accept 进门预判超限就先排空，所以不需要。

**依赖的前提：** RefFS 仍是单文件（一次触发排空是一次发布）。多文件的 RefFS 出现，且不同文件的写要分几次发布时（P3），要另判。
**什么现象推翻它：** 翻译里自动发布的位置错一格（放在下一次 accept 之后），则 3501 条合法恢复态里的一部分会再次判红；用例里加一条这样的变异，rerun 的自证已经覆盖「发布不抬下界」这一种。

## P6　阈值的只供测试开关与「配置生效」

**开关怎么写：照整理开关与删除开关的先例，装在分配器上，只住内存。** 先例原话：「都只住内存、不落盘」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/unlink.rs:28`），「经可写挂载的只供测试的开关链装在分配器上」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/unlink.rs:29`）；分配器字段「deletion_switches: DeletionSwitches,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/allocator.rs:1871`）；汇总入口「fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:8693`）。新开关〔开关〕= 一个只有两个值的结构（时间阈值、脏量阈值）加 `PRODUCT` 常量，作为汇总入口的一个新参数，由一个新的 `..._with_test_only_...` 命名入口给；`PRODUCT` 取挂载时求一次的有效值：时间 = 5000 ms（「const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/system_configuration.rs:102`）），脏量 = min(「const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/system_configuration.rs:103`）, 环长 ÷ F)。

**满足 `.claude/rules/fs-design.md` 的两条：**
- 「2. **每条分支必须能被测试强制进入。**」（`.claude/rules/fs-design.md:122`）：开关把阈值压到测试大小（例如两个数据单元、几毫秒），无视 5 s 与 256 MiB 的判据，两支各能单独跨过去。
- 「5. **分支变量不许在操作中途改变。**」（`.claude/rules/fs-design.md:128`）：阈值挂载时求一次、会话期间恒定；已定项 2 的射程也这样说：「它们在挂载期间恒定」（`.claude/kb/decisions/16-发布语义.md:108`）。变的是 `now` 与待发脏量，不是分支变量。

**不变成第二条配置通道的四条约束：**
- 开关只出现在 `..._with_test_only_...` 命名的入口里，产品入口 `mount_writable` 恒取 `PRODUCT`（现状：「pub fn mount_writable<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:8087`）只传 `SpaceAdmission::JudgedByTheFormula`）。
- 不落盘、不进系统配置字段，不碰 C422 说的「读回来」。
- 会话期间没有任何入口能改它。
- 越界值不警告：开关就是绕过，警告属于将来真正的输入通道。

**要正视的先例风险：** 仓里已有一个绕过准入的只供测试开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`（「只供测试的开关关掉准入」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:1120`）；grep 命中在 `out/count_assumptions.txt` 的 D2 一段）；`mount_writable_with_test_only` 的引用今天出现在 5 个文件，`crates/singlefs-core/src` 之外只有 `singlefs-harness/src/scenario.rs` 与一个装置，没有产品二进制。所以「测试开关成了第二条路」今天的防线只有命名；加一条机械的防线很便宜——门禁里 grep：非测试目录、非装置的代码对 `..._with_test_only_` 名字零引用——这一条建议交主 agent 判要不要立。

**验收「配置生效、无越界误触发」在只有测试开关时验到哪一步：**

| 能验（里程碑四） | 留里程碑十 |
|---|---|
| 阈值 → 行为：T−1 不触发、T 触发（`≥`）；时间先到、脏量先到、同时到（`out/model_firstcross.txt`） | 挂载时传 T_time，重挂载读回新值（C422，「挂载选项通道不存在」（`.claude/kb/checks-owed.md:297`）） |
| 无越界误触发：待发为空时无论钟走多远都不发（守门）；阈值下一格不发 | 盘上与运行时报「这次挂载生效的是多少」（C424，「生效配置不可见」（`.claude/kb/checks-owed.md:299`）） |
| 产品入口的有效值读回：5000 ms、min(2 GiB, 环长 ÷ F)；默认环得 256 MiB，小环几何得环长 ÷ F，夹取这一支强制进入 | 两端值挂载时拒绝（C733，「条款已定：两端值（低于 1 ms 与超过 u32 毫秒上限）挂载时拒绝」（`.claude/kb/checks-owed.md:459`）） |
| | T_time 越出 [200 ms, 5 s] 的警告（已定项 5「只给警告」） |
| | C425 的真机有界断言 |

**越界警告在里程碑四交给谁：没有人。** 里程碑四没有任何输入能产生越界的 T_time——产品入口固定 5000 ms，开关绕过——警告随挂载选项通道一起落到里程碑十。`T_dirty` 的夹取在里程碑四做，但夹取静默（已定项 5 写明两个字段「两种规则」），不产生警告。

**依赖的前提：** 开关结构加进汇总入口的参数表（这改 `mount.rs`，与第 39 批实现员撞文件，排在它之后）。
**什么现象推翻它：** 找到产品路径可以构造该开关的入口（grep 命中产品二进制）；或夹取这一支没有测试能强制进入（默认环一个环长，需要一个小环几何的用例）。

## P7　串行下不该在里程碑四立的

三样逐个对着今天的实现：

| 样 | 串行、在飞深度 1 下触发吗 | 证据 | 去向 |
|---|---|---|---|
| C425：T_time 小于一次发布耗时时触发空转 | 不触发 | C425 的机理是「在飞 checkpoint 深度只有 2（开放 + 发布中）⇒ 触发空转、脏数据一直攒」（`.claude/kb/checks-owed.md:300`）。串行下 accept 与轮询都收 `&mut self`，一次发布在调用内做完，返回前没有第二次触发，不存在「上一次没发完」。返回之后条件立刻又为真的，是**无待发时**的空发布，由 P1 的守门挡掉（模型：有守门 0 次，无守门 99 次）。真机有界断言需要并发，留里程碑六与里程碑十 | 里程碑六；守门条款立在里程碑四 |
| 重做期间开放的 checkpoint 堵不堵写者 | 不触发 | 已定项 5 把它记为口径空白：「也不定重做期间开放的那个 checkpoint 会不会堵住写者」（`.claude/kb/decisions/16-发布语义.md:166`）。会话里没有实例切换入口（`grep -rn 'fn [a-z_]*instance_switch\|fn [a-z_]*switch_instance\|fn redo_' crates/singlefs-core/src \| grep -v admission.rs \| wc -l` 得 0）；T_retry 没做：「块设备报的第一个错就判失败（写路径上的重试全仓都还没有）。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:1001`）；失败的发布冻结，只能显式原样重发。串行下写者就是被阻塞在那个调用里的调用方，没有「开放的 checkpoint 在重做期间继续被写」的场景 | 里程碑六 |
| 丢失窗口 T_time + N_switch × (T_retry + T_redo) 的验证 | 只有 T_time 项可验，P 与 d 两项要补进式子（P1） | N_switch 项没有对象（会话内无切换），T_retry、T_redo 全仓无实现（`grep -rn 'T_redo' crates/singlefs-core/src \| wc -l` 得 0）。里程碑四可验的是 `T_time + P` 这一段：模型最坏排布 5999 ms ≤ 6001 ms | 里程碑四验 T_time + P；N_switch 项里程碑六 |

**串行下仍要立的两条：**
- 〔守门〕触发式只在待发非空时求值（P1）。这一条不是 C425 的有界断言，是串行下就会出现的空转（无待发的空发布），条款空白，应补进已定项 2。
- 〔冻结〕发布冻结着时 accept 在任何读写之前拒；冻结期间待发不再增长。这是现有「冻结着的时候发布路径在任何读写之前拒绝」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:1470`）在新入口上的延伸。

**依赖的前提：** 调用方串行（`&mut self` 持有会话）；会话里没有实例切换。
**什么现象推翻它：** 里程碑四期间有人给会话加了实例切换入口、或让 accept 与轮询跨线程并发调用（要先改成 `Arc<Mutex>` 之类），那三样就不能留给里程碑六。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1 | 规则没说 | 谁来求值、空闲时怎么触发、守门、基线、窗口公式里的 P 与 d 都没写；只在进门判会打中已定项 2 的依据；推荐 accept 加调用方轮询，钟由调用方给，复用 `CallerMonotonicClockReading` |
| P2 | 冲突（单位） | 夹取式把数据字节与日志环字节直比，与「满窗 65 536 条记录、256 MiB」一句对不上；今天能算的第三种读法取折叠后的将写数据单元字节 |
| P3 | 规则没说 | 已定项 10 没限种类；数据写类可并（同 inode 折叠、建 inode 求和），删除类、截断、快照、整理单发；已 accept 的改动必须发布得出去是新不变量；并了以后同事务判器是否仍红复核不了 |
| P4 | 规则没说 | 立即排空入口是验收要用的；入口分开则现有 100 处调用、9 种历史操作零改动；读路径看不到待发是新增的条款空白 |
| P5 | 冲突 | RefFS 只接显式发布点，与已定项 10 的攒批并排；真 RefFS 实测 32 152 条里 3501 条合法恢复态判红、2084 条该红的放过；RefFS 本体不改，翻译把会话报告的发布翻成显式发布点即两个数都归零 |
| P6 | 一致 | 开关照整理与删除开关的先例，满足 fs-design 的第 2、5 条；配置生效的读回、警告、两端拒绝全留里程碑十 |
| P7 | 一致 | 串行下 C425 空转与重做堵写者都不触发，留里程碑六；里程碑四补守门与冻结两条 |

## 要主 agent 跑的命令

无。我没有起任何重型测试，没有跑门禁阶段；真 RefFS 那一步是 std-only 的小 crate 在草稿目录副本里的测试目标，编译目录用完已删（`rerun.sh` 自己删）。

## 没做什么

- 没替主 agent 采纳或出判决；〔推荐的条款集〕是这一条腿的立场，不是判决。
- 没改 `crates/`，没跑变异表：因此「并了以后同事务判器仍红」复核不了（P3），要在副本上改坏一处再跑 `crates/mutations.tsv` 里那两条变异所在的测试，是重型，交主 agent。
- 没读 `prepare_the_version_publish`，所以「accept 进门的只读预判能不能不拷分配器」是待核前提（P3）。
- 没做真机，没量发布耗时 d，没量调用方轮询周期的取值；P 与 d 只写进公式，没给数。
- 没碰攻方腿的任何产出。
- 报告在草稿目录里分四段写好，再一段一次拷进 `research/prompts/`；第一次拷贝是把四段在一次调用里连着写了（290 行），已删掉重写成四次，每次不超过 150 行。
