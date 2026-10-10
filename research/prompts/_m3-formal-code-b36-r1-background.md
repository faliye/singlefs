# 第 36 批代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 R1 R2 R3 -->

## 一、题面

第 36 批（`records/2026-10-01-里程碑三形式化接入.md#36`）照第 35 批代码轮判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第三节第 1 条改，改动全文 `research/prompts/_m3-formal-code-b36-r1-diff.md`：

- 36-1b：`crates/singlefs-checker/src/walk.rs`——R1：「下一次挂载先施加的那一版」改取 I-4.1 那份重放 `replay_the_next_mount_would_do` 施加的第一次发布、取它末条那一份新根段（`ReplayTheNextMountWouldDo` 的 `Applied` / `Stopped` 各多带第一次发布的末条；一次都不施加、锚点认不定、所选根 txg 没有后继时不认），I-3.10 与 I-5.4 共用；R3：FL 那段说明照 R1 重写，I-5.5 新理由收窄到「池里的盘上都没有数据单元」。`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`——R2：「拿掉根槽写」两条流改成合法造法录流。`crates/singlefs-harness/tests/checker_known_bad_images.rs` 两条新用例（断号；后面一次发布停在点名单元上仍取第一次）。`crates/mutations.tsv` 追加 2、替换 2。
- 36-2：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 走读内核的重放改名 `replay_the_next_mount_would_do`，多交回第一次施加的发布的末条，经交接段空着的第 31 个字交给扫描内核；`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `record_row_of_the_version_the_next_mount_applies_first` 读它，I-3.10、I-5.4 同用；`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 随改；对拍用例加两条流；`mutations.tsv` 追加 2、替换 2。

条款：`.claude/kb/invariants.md` I-3.10、I-4.1、I-5.4、I-5.5 几行；`.claude/kb/decisions/23-journal的角色与格式.md`「#### 已定项 14」「#### 已定项 15」；`.claude/kb/checks-owed.md` C735。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree/`（第 36 批合入后，清单同目录 `crates-sha256.txt`，316 行）。树里另有删除线第 28–29 批与根环线 10o / 10p，不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的工作目录（`~/.local/share/singlefs/work/<腿名>/`），每份副本用自己的编译目录，编完核日志里被测包确有 `Compiling`。
- 合入验证 `~/.local/share/singlefs/work/merge-and-verify/2026-10-06-1923410/`：harness 170 个测试二进制全绿，fmt、clippy 过；mutation-tables 只红 `crates/mutations.tsv` 第 2919 行（删除线第 26 批那一行，不归这一轮）。
- 实现员在副本上证红：36-1b harness 档 2 行抓到；36-2 的 2 行 GPU 变异与 4 行替换没证红（checker 档）；36-2 换基之后只跑了 fmt、clippy，合入验证补齐了编译与 harness 测试。GPU 没跑。
- 根环线报：各线合入后扫描内核 `crash-judge-checker-scan` 展开 700 783 个表达式、超预算 400 000，GPU 对拍编之前就拒；谁推过线根环线在量，不归这一轮判，涉及 GPU 的结论一律「推的」。

方案按 `crates/` 今天的实现来谈。

## 三、格

### P1　R1（CPU）
`version_the_next_mount_applies_first` 取的第一次发布与 core 恢复真施加的第一次发布是不是同一份（读 `crates/singlefs-core/src/recovery.rs` 施加那一段对照）：断号、点名单元停在第一次或后面某一次、锚点认不定、所选根 txg 没有后继、几个实例并存、回退之后几种历史各走哪一支；`Stopped` 带第一次的那一支在什么历史上走到；I-3.10 与 I-5.4 是否仍同一个认法；合法历史上多报少判。

### P2　R1（GPU）
36-2 选的乙（经交接段第 31 个字传下标）：写那个字的路径是不是先于扫描内核开工、所有能置 `H_VALID == 1` 的路径都写了它；走读内核 I-4.1 那一处行为是否不变；扫描内核读到的那一行与 CPU 认的是不是同一条记录（GPU 跑不了的读码，写「推的」）。

### P3　R2
两条流的新造法与合法历史逐字节一致吗（b35 攻方量过的 W 与系统配置槽两处）；末态判定与原造法相同吗；两条流名字说的那一形还在不在。

### P4　R3
三处说明与 I-5.5 理由的措辞与判定是否一致。

### P5　变异行
追加 4 行、替换 4 行：锚点各恰好命中一次、点名的测试跑得到、合入后基线绿、红在断言上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | P1–P5 | 从判决与条款正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | P1–P5，重点 P1、P2 | 找反例：一段合法历史让新认法误红或少判，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮；条款没写、要用户定的，交用户。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b36-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b36-r1-<腿名>-model/`，带一份 `tools/rerun.sh`（每条 cargo 经 `research/scripts/run-with-memory-cap.sh`，上限按峰值表给或写 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`，同一时刻只起一条）；草稿放 `~/.local/share/singlefs/work/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
### 小节清单：`.claude/kb/invariants.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 不变量清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （不变量清单 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 状态 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-1 块自描述 —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-7 根环 —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-8 journal —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-2 校验和 —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-3 空间记账 —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-4 事务与崩溃一致性 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-5 分配器 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-6 加密不变量 —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## 待补 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |
| ## I-9 inode 树结构 —— 类 | 不抄 | 这一轮只引 I-4、I-5 两类；I-3.10 一行用 --extra 按行取 |

### 小节清单：`.claude/kb/decisions/23-journal的角色与格式.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D23 journal 的角色与格式 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D23 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：journal 的角色本身 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 2：journal 是定长环还是链式 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 3：tail 住系统配置槽，改 jbd2 两处 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 4：记录头约束在单个原子单元内，头 487 字节 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 5：O2（独立解析器 + checker） 是一元谓词，逐字节比对归记录核对器 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 6：目标负载优先级移出 journal 这条线 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 7：一个事务可以跨多条记录，记录头带事务边界字段 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 8：记录头带一条 32 位反向链 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 10：`previous_hash` 覆盖前一条记录的整个记录头 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 11：反向链的 hash 取完整 32 位 CRC32C | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 12：记录尺寸取 4 KiB | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 13：载荷完整性，`header_csum` 罩整条记录 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：mkfs 写实例代号 0，第一次可写挂载取 1 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 17：点名项 56 字节的字段表 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 18：在飞记录数上限的取值与语义，tail 字段的编码 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 20：记录承载指针层的目标态，分配先于 journal | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 21：世代号：记录粒度的全局单调号，不是 D5（快照 / 空间记账机制） 的 txg | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 22：前缀判定：严格连续、断号即止 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 23：三个死锁各自怎么破 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 24：死锁 2 与假性 ENOSPC 是两种故障，要分治 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 25：D16（发布语义）「不绕过 journal」取读法甲 | 不抄 | 这一轮只引已定项 14、15 |
| #### 已定项 26：记录头的两条解析纪律 | 不抄 | 这一轮只引已定项 14、15 |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/checks-owed.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 欠的检查清单 | 抄 | 随表头：标题与第 1–16 行由 --extra 一并取 |
| （欠的检查清单 标题之下、第一个下级标题之前的正文：第 2-599 行） | 不抄 | 这一轮只引表头第 1–16 行与 C735，用 --extra 按行取 |
| ### 已还清 | 不抄 | 这一轮只引表头第 1–16 行与 C735，用 --extra 按行取 |
**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天报它的不止一条，逐条见各行（例：I-5.4（分配记录罩住的槽互不相交）、I-5.5（从 mkfs 水位建树那一版的 t1 落点）、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:172-190`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-05，在清单里的第一版恒报不适用：池级 checker 在 `walk::check_pool_image` 里对它调 `not_applicable`，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`；不要求坏镜像，`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名；GPU 判器下标 73，内核不置位）。第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是中间态特有的一格：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs` 第 61 行 `div_ceil(...).max(1)`）。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条 nlink = 0 的记录判一次，一条都没有时报不适用并带理由；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；射程只对 nlink = 0 的记录，每条记录都判的那一形不挂在这一行名下，另记欠账 C711（单元数对 size 只判了 nlink = 0 的记录）） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:191-213`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 判的版本是回退候选集里每条根，加由记录施加出来、根槽从没落盘的那几版（已被一次恢复施加过、实例表里有行，树表指针与「树表 0 条那一版的分配记录树根」取自记录的新根段）与下一次挂载先施加的那一版（记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根）；这些版本的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽。同一条下另判一格：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节必须全零（D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行），非零就是这一版有两棵分配记录树，判违例。这一格成立的那一次，只在这一版自己的分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4 的评估，没走到叶的版本不计。部分没判（逐版列出，不判红；判出违例照报违例）：树表指针非零而树表读不出的版本、树表指针全零的版本（core 的根记录读者照它读盘 0 槽 0、读不出），都认不出带不带树，根记录里分配记录树根全零那一格与它自己的分配记录树都没判；树表读得出、0 条而分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））、那棵树读不出的版本；带树一版自己的分配记录树没走到叶的版本。候选集里没有一棵分配记录树走得到叶（第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上）报不适用。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，吃一张版本表（`VersionToJudgeTheAllocationRecordsOf`：称呼、树表指针、树表 0 条那一版的分配记录树根），表里是回退候选集里每条根、由记录施加出来的那几版与下一次挂载先施加的那一版（`check_pool_image` 拼表）；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_allocation_records_disjoint_of_a_version`，候选根、由记录施加出来的版本、下一次挂载先施加的那一版各调一次，只有违例位、部分没判不置位；这两类版本上的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant` 与 `two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant`，读不出的几格各有用例（`the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged` 等）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠；带树一版分配记录树根全零那一格的挂法见判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法」那一行与 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「成立要至少一棵树走到叶」那一行，树表读不出与根记录直接持有的树读不出两形见 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行与 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 第一行，射程含两类由记录造出来的版本见同一节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」那一行，树表指针全零的版本见同一节 P2「树表指针全零的根」那一行与第五节第 1 条） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排：这要求这一版发布时每块盘上都没有全空的 64 槽段，今天的写侧在「只暖机 / 只写行的挂载接发第一个文件」这一族里量到 0 次回落（单元区 240 / 256 / 384 槽，连续挂载 k ≤ 60，一次抽样、不是证明）；分配器上留了回落计数，`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs` 断言它是 0，哪天走进回落、用例红，回来重看开放段认法）；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；判不了的几格报部分没判（`InvariantVerdict::PartiallyJudged`，没有违例时整条报它，逐版列出实例、txg 与为什么，形态同 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））：这一版的树表读不出（含树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出）或条目宽不到 200 字节，认不出这一版是不是这一类；认出来的那一版 extent 树读不全（数据单元数不全）、分配记录树读不全（落点算不出）；种类 1 条目不止一条、种类 3（分配记录树）条目不是恰好一条，这两格的树表本身由 I-9.16（树表条目按树 ID 严格升序且合发号次序） ③ ④ 判红、这里只记没判；有数据单元而池里某块盘上一个都没有，那块盘没有对象，跳过、不记（CPU 与 GPU 一致）；GPU 判器只有违例位，这几格不置位；走读到的版本里一版都不是从 mkfs 水位第一次建树的那一版时，整条报专门的不适用理由（`walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK`，`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里那条用例的 I-5.5 一支只认它），认出来而没有数据单元那一形仍落缺省理由；三格的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:1-13`（整段抄，未转述）**

```markdown
## D23 journal 的角色与格式 —— 已定

<!-- doc-lint:not-numbers CRC-64 SSE4.2 CRC32C SHA-256 CJ2 -->

D23（journal 的角色与格式） 管 journal 是什么、写什么、崩溃之后怎么用它：记录承载什么与分配的先后（已定项 20）、每次 fsync 写什么（已定项 1）、环的几何、记录尺寸与在飞上限（已定项 2 / 12 / 18 / 19）、tail（已定项 3 / 18）、记录头与点名项的字节（已定项 4 / 7 / 8 / 9 / 10 / 11 / 13 / 15 / 17 / 21）、前缀判定与重放下界（已定项 14 / 22）、管理员回退、失败处置与实例切换（已定项 14 / 16）、与 journal 相关的三个死锁和保留池（已定项 23 / 24 / 25）、记录头的解析纪律（已定项 26）。不管的：发布的持久顺序、checkpoint 的触发与暖机归 D16（发布语义）；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11；根记录与根环归 D22（单元原子性怎么合成）；保留池与切换预留怎么进准入不等式归 D28（挂载期承诺量）；checker 与记录核对器各是什么归 D13（验证路线） 已定项 6 / 7；目标负载归 D25（目标负载优先级）。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| T6 | 维持拒绝时拒不拒取决于池的历史 |
| L8 | 并行线三跨记录层0流 |

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:355-449`（整段抄，未转述）**

```markdown
#### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换

**定案**：**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**

**前缀判定的完整口径是五条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引）：
jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、
提交标记齐全的事务才施加（D23（journal 的角色与格式）已定项 7）、
施加前逐项验证点名单元（D16（发布语义）已定项 7，E77（发布的持久顺序）证明承重）——**一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加**，有一份读不出或对不上，这条记录与它之后的都不施加。
⚠️ **第五条：施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加，前四条判出来的前缀再按发布边界截短一次；引用这四条时连这一条一起引。**（D16（发布语义） 已定项 4）**发布边界怎么认**：一次发布的末条 = 记录标志位 0 为 1 的那一条（已定项 4、17）；前缀里一次发布的末条没到，那次发布整体不施加。

两份镜像何时算「记录在」：**任一份自证校验和过即在**（崩溃点重放里的检查由层 0 全量做出——jsn 3 只有一份持久的两个状态都读回文件）。点名单元不同：**两份都验过才施加**（前缀第四条）。代价：两块盘的池上一块盘整块掉了，记录在幸存盘上、点名单元有一份在掉了的盘上，那条记录不施加——恢复退到更早那一版，已确认（fsync 已返回）的最后一版丢掉；D16（发布语义） 已定项 8 的暖机管不到这一格。用户知情认下（C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格：「任一份验过」会不会放过 E77（发布的持久顺序） 要挡的一份新一份旧的嫁接没量过）。

**跨实例边界、链首、比较序、计数器与切换时的 W**（注 1–4）：

1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（五条口径第一条原样）。**所选根覆盖的最后一条** = 与所选根同实例、同 `checkpoint_txg` 的记录里 jsn 最大的那条——一次发布切成多条记录时它们共享一个 `checkpoint_txg`（D16（发布语义） 已定项 6），那次发布的最后一条才是根覆盖到的末端。**那条读不出时**，链首接水位之上第一条可读记录，前提是它的「本次发布内序号」为 1（已定项 4）、`checkpoint_txg` = 所选根的 txg + 1；序号不是 1 说明下一次发布的开头缺了，断号即止。**「那条」按末条标志认**——所选根那次发布读得出的几条里没有一条带末条标志，就算「那条读不出」，走「序号为 1 的第一条可读记录当链首」那一支；不取读得出的同 txg 记录里 jsn 最大的那条（用户 2026-09-24 定，读法乙；三方 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 那一支四条臂同中、没分出候选，这一读法被攻过零轮）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
2. **记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主比**，与 jsn 同序；择根照旧 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7），两处比的东西不同，各守各的序。按 txg 为主时，新实例的第一个根取「根环最大 + 1」，而被抛弃实例开放 checkpoint 里已提交的记录 txg 可以更大，会被当成水位之后施加到新实例的根上。
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。前缀末的计数器取环里读得出的最大那一个：所选根自己那条记录两份都读不出时，新实例的第一条编在读得出的最大号 + 1，落回那条读不出的记录原来的槽（被重写的本来就读不出）；计数器从记录头读，不按 txg 推。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的 48 位本来就按全卷寿命算。⚠️ **checkpoint_txg 也一样**：新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1；普通挂载本来就全环扫描，切换不为算这个下界重扫 journal 环（内存里的 txg 已不小于这次挂载见过的一切；切换的所选根要读盘择，见实例切换那一句）。它与 C331（择根倒挂压过已确认的写） 的修法候选「记录扫描水位」是同一个量，C331（择根倒挂压过已确认的写） 还清时一起写。代价：跳号比今天大（最多到被抛弃时间线的长度）；记账不受影响：D5（快照 / 空间记账机制） 已定项 2 每条根的记账树只装这条根那一代的行，不点删旧代，跳号不留要删的行（C341（K 代点删在 txg 跳号时留下删不掉的行） 题面消失同理）。
4. **切换时的 W 取被重发的那个 checkpoint 里的最大事务号**：开放 checkpoint 里已完成的事务全部 > W、按新写序重做。按第 2 条，它们的记录不在所选根之后；W 若取字面的「最后施加的事务号」，它们的单元会被谓词判成已发布而谁都不施加它们（C287（切换收养开放 checkpoint 的事务后再崩））。被重发的 checkpoint 里没有事务号非 0 的记录（重发的是空发布）时 W = 0（已定项 19 ①：事务号 0 不进 W 的 max）；W 写在每个被照旧事务的写序实例那一行，切换的所选根按实例切换那一句取。

**第一版没有干净关闭标记**：每次挂载一律走恢复（全环扫描、逐条验证、择根，按五条口径与注 1–4 施加），不按上一次是不是干净关闭分支；准入、暖机、回退候选集、F 的生效都不读这样一个标记。

**第一个文件版本接在哪条记录之后**：写第一个文件版本之前，要接的那一版之后的上一条记录必须解得出、且它的 `checkpoint_txg` 等于那一版的 txg——新记录的计数器取它的计数器 + 1、反向链取它的记录头（已定项 8、已定项 9）；不满足就在任何写之前拒绝（`crates/singlefs-core/src/transaction.rs` 的 `publish_first_file` 返回 `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`），一个字节不写。

**管理员回退是挂着时的一次向前发布**：管理员在一次可写挂载里从回退候选集选一个旧根 R_old，文件系统发一次普通发布（D16（发布语义） 已定项 7 的持久顺序），新根的用户可见树（inode 树、extent 树）与 `deleted_inodes` 的根指针取 R_old 那一版的；墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)（D18（块里携带什么信息） 已定项 24）；树表单元、分配记录树、中央映射树与记账统计量（D5（快照 / 空间记账机制） 已定项 4）按这次发布照调整之后的分配器现算，不从 R_old 那棵账载入（inode 号与树 ID 水位见表里「水位」一格）；实例表照 cur 的；checkpoint_txg 照常加一，**不新增实例、不取号、不写实例表行、不施加也不删除任何记录**。记号：cur = 现行那一版，new = 回退发出的新根，用户可见单元 = 数据单元与 extent 树、inode 树的单元。

| 项 | 定案 |
|---|---|
| 候选集 | 根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义） 已定项 1）∧ 带文件（那一版写过文件，不是还没写过文件的那一版；删完之后有树、文件集合为空的那一版写过文件，算带文件）的根：(i, T) 可选 ⟺ cur 的实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被崩溃恢复抛弃的时间线上的根一条都选不中 |
| 释放 | cur 那一版的账里仍分配、而 new 不引用的落点，释放代 = new 的 txg；隔离单元（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：任一份核出对不上、两份都留在已分配的）除外，留在已分配（用户 2026-09-27 定） |
| 复活 | R_old 引用的用户可见单元里，cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的分配代 |
| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |
| 记录 | 回退那次发布的记录事务号取 0、不推进实例内计数（与空发布同：不承载用户事务，D23（journal 的角色与格式） 已定项 19 ①）；新根带的 F 取现行那一版根上的，不算抬 F；jsn 接现行那一条（主 agent 2026-09-26 定，推的，被攻过零轮） |
| 准入与记账 | 回退照旧不判准入式子；谱系容器归固定点角色、进 ckpt_cost 的 Σ 名单；记账标签 4、6 按新根这一版现算（inode 树与 `deleted_inodes` 取 R_old 的，所以被带回来的对象的墓碑份额照抬；标签 6 按新根现算时，n_live（D3（空间分配） 已定项 2）= R_old 那一版 inode 树各叶容器头的记录数之和减 `deleted_inodes` 的 key 数，第一版 inode 树一个根、至多 135 个容器），不判（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V4；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G3-回退；用户定案 K-W5「谱系记录」、K-W5b） |
| 在任何写之前拒 | R_old 引用的用户可见单元在 cur 的账里没有记录，或仍分配而分配代与 R_old 那一版账里的不同；R_old 那一版或 cur 那一版的账有一个节点读不出；复活集里的每个单元（cur 的账记成已释放、这一次改回已分配的那些）在每块盘上的那一份都要读得出、校验和对得上，有一份不行就拒——它们的槽在 cur 的时间线上释放过，释放之后可能被一次没发布成的写盖掉一块盘上的那一份；cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内（与前缀第四条「两份都验过才施加」同一方向；主 agent 2026-09-26 定，推的，被攻过零轮） |

崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同；回退的根持久之后才向管理员确认（D16（发布语义） 已定项 7）。回退深度由候选集定：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。**被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内——「有效根」指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效），每次挂载与每次抬 F 按当时的候选集重算；被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报，映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮，判决 `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西（影子账；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 同一次定案（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。

**可写挂载读到的更新状态读不出（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，用户 2026-09-27 定；判据 N-配置续 照 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行）**：读阶段（择根、扫 journal、重放）读完，读池里每块盘两槽的系统配置：判不出 ⇒ 按真（判不出取 R2，权威写法在 D22（单元原子性怎么合成） 已定项 8：有槽设备报读错，且没有一块盘两槽都自证过；用户定「改取 R2」；实现取 R2：`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_witness_is_undecidable`，读阶段与取号同用；欠 C331（择根倒挂压过已确认的写） 的是多次挂载崩溃点重放里的注入）；不是判不出时取全部自证过的槽 journal tail 的最大值 c_见证，0 ⇒ 不判；所选那一版那次发布带末条标志的记录读得出（任一份）⇒ c_见证 > 它的计数器即为真；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) 大于所选那一版即为真；两条都读不出按真。为真就在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍（R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等），重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常。重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）。重建分配器读根环两遍并起来一条不被抛弃的自证有效根都没有，同样在取号之前拒可写挂载，报 `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（D16（发布语义） 已定项 1「根槽这一次读坏」那一行），这一拒在单子上；挂载透明（`crates/singlefs-harness/src/mount_transparency.rs`）在镜像上复算「按最新那条根的实例表，环里一条不被抛弃的自证有效根都没有」，没注读故障时复算为真判不合法、注过读故障的态不判、实例表读不出时不按表滤。**「续」**：取号那一写（`crates/singlefs-core/src/transaction.rs` 的 `write_acquired_instance`）不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证（用户 2026-09-27 定「乙-配置续（推荐）」；实现员 2026-09-27 交回并打上，落点是 `write_acquired_instance` 里的 `highest_system_configuration_journal_tail`：每块盘两槽里全部自证过、fsid 与本池相同的系统配置槽 journal_tail 取最大，一份都没有时 0，在第一道屏障之后、第一个取号写之前读一次）。取号失败的回卷写（D18（块里携带什么信息） 已定项 11）带同一个见证值，不退回 0。mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变。取号那一刻按同一个判不出（R2，见 D22（单元原子性怎么合成） 已定项 8）判：判不出就在第一个取号写之前拒（实现落点 `crates/singlefs-core/src/transaction.rs` 的 `self_verified_system_configurations_of_every_device`，已取 R2；这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着），欠 C331（择根倒挂压过已确认的写） 的是多次挂载崩溃点重放里的注入）。系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；这一形记欠见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 Q1。

**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写不进去 ⇒ 持续失败，走**转只读到下次挂载**；写得进去，再**对这次失败的那个落点做一次只读复核**：读得出 ⇒ 瞬时失败，走**实例切换**；读不出 ⇒ 落点级的持续坏，同样走**转只读到下次挂载**。⚠️ 复核这一步**只读不写**：写探针进失败落点会抹掉那里的权威数据（8 个失败点里 4 格的落点是根槽或系统配置槽），而固定落点这一步不许改——`D2`（RAID 条带策略） **已定项 13** 逐字「与 `D23`（journal 的角色与格式） 已定项 14 的两支判别子同一个动作」靠它成立（主 agent 现查：那句话在 `.claude/kb/decisions/02-RAID条带策略.md:225`，归属已定项 13「降级期间只读」，不是已定项 15）。**失败点是屏障时没有落点可复核**，那几格只有固定落点探针写这一步。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写加落点只读复核。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份系统配置、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。

- 切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **(N_switch + 1) × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵；多的一份给这次挂载写行那次发布的元数据）；**这个量的口径与式子的权威正文在 D28（挂载期承诺量） 已定项 3**：一次切换的最坏量 = 实例表链重写（副本数 2 × 32768 × 片数，按设备算）+ 暖机空发布（至多 R 次 × 现算 c_max），**固定点重做与号 > W 的数据单元重写不另要空间**——切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；引这一段时引 D28（挂载期承诺量） 已定项 3。
- **预留挡的是分配失败那一路，不挡写失败那一路**——写失败由探针写与取号全或无挡。
- **「分配失败」按它发生在哪里拆三支**（不拆的话池一满就整卷只读，而释放空间要写盘、重挂又被预留合取挡住，撞了 `.claude/rules/fs-design.md` 那句「释放空间这个操作本身不需要申请空间」）——前台事务的用户数据分配失败 ⇒ **向调用者返回 ENOSPC**，checkpoint 照常发布、不动挂载状态；后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**，不动挂载状态（`.claude/rules/fs-design.md` 第三格「无戒律」、D2（RAID 条带策略） 已定项 4 c「前台不停」）；**切换自己的预留拿不到** ⇒ 才转只读。
- 根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；⚠️ 由此「同一个根槽连续失败」不再是一条判据——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着，留着它读起来像一道闸、实际是空的。
- **实例切换** = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint——所选根**重新读盘择根**：对着仍打开的设备句柄按恢复择根的同一套规则现选（`crates/singlefs-core/src/recovery.rs` 的 `choose_root`），不取调用方内存里记的根——根已落盘而系统配置槽那一步失败时，内存里记的根比盘上的现行根旧（C458（实例切换取内存里的根，不重新读盘））；行只落在 [这个根的实例, 新实例)，每个被照旧事务的写序实例 k 写 (k, k 最后发布的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)，其余按 D18（块里携带什么信息） 已定项 11（旧实例发布过根、被照旧的事务都是它自己的时候，就是 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号) 这一行），号 ≤ W 的事务照旧（开放 checkpoint 里已完成的事务全部 > W，C287（切换收养开放 checkpoint 的事务后再崩） 按这个 W 封死，注 4）、号 > W 的按新写序重做或向还没返回的调用者报错，固定点单元全部按新实例重写；不转只读、不等下次挂载，代价 = 一条行 + 整条实例表链重写 + 号 > W 的数据单元重写 + 一次固定点重做，频率未测。

**这一版的失败处置**：发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置与字节都不变），重发成功才建下一次发布；于是合法历史里同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录，恢复遇到它就是盘坏了，在任何写之前停下（`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做。原样重发在串行提交下会把一次失败放大成整条发布流阻塞，欠 C541（原样重发把一次失败放大成整条发布流阻塞）。

**射程**：管恢复施加哪一段、回退与切换怎么做、失败怎么分流；实例表的行格式与写行规则归 D18（块里携带什么信息） 已定项 11，切换预留的量归 D28（挂载期承诺量） 已定项 3，defer 窗口与 F 归 D16（发布语义） 已定项 1。已知边角：

- ⚠️ 它对已定项 3 是收窄不是推翻：「先全环扫描、逐条验证、不许先信 tail」原样成立——变的是验证出的合法前缀里**哪一段被施加**。
- ⚠️ **影子账与按实例表判抛弃（`abandoned_by_table`）留着，理由是崩溃恢复，不是管理员回退**：一段没有任何管理员回退的历史里，崩溃恢复因暂时读错落到旧根、实例表抛弃了较新的根，关掉影子账，被抛弃的根引用的数据单元被复用，之后恢复落在它上面读不出（第一轮 H6）。「只在崩溃恢复落到旧根的那次挂载里隔离」挡不住它：被复用的写发生在下一次干净的挂载里（第二轮辩方 G6，读代码的判断、没量）。落 [checks-owed.md](../checks-owed.md) C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）。 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续（用户 2026-09-27 定）之后，系统配置见证过的较新的根暂时读不出不再被抛弃：可写挂载重读一次，仍读不出就拒可写；影子账留着的理由收窄成系统配置没见证到的那一形（那次发布的系统配置轮换没落盘）。
- ⚠️ **C331（择根倒挂压过已确认的写） 判不出取 R2 之后的代价（取 R2 的权威写法在 D22（单元原子性怎么合成） 已定项 8；取甲时量的数在 `research/prompts/unreadable-at-mount-r2-main-verification.md` L1）**：撕裂的系统配置轮换（普通断电就造得出，被撕裂的那次发布没确认过）不再因判不出拒可写挂载；仍判不出的是有槽设备报读错、没有一块盘两槽都自证过那一形（一槽系统配置持续读错），可写挂载每次都拒，只读挂载不写盘，那一槽没有机会被重写——池只剩只读挂载；「只读挂载不写盘」对今天的代码量实（2016 条轨迹），条款上没有，见 C32（无密钥的一侧写不写得出字节没有闸）、C33（scrub 检出之后做什么没有条款）、C75（降级挂载这道闸自己不会红）。一次可写挂载都做不成时怎么走出来，没有条款（C596（撕裂的系统配置轮换之后只剩只读挂载））。C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载。
- ⚠️ **挂着时回退的已知边角三样**：① 「R_old 仍分配的落点在 cur 的账里是同槽同分配代」这道核不在定案里，也补不上——已释放的记录记的是释放代（`crates/singlefs-core/src/allocator.rs:38`），核看不出「这个槽被复用过又释放了」；它放过的读不对的回退只在 F 回落时走得到（第三轮判决第二节 K1：F 生效取最大值的最小值时 15 格中 12 格，取 SysPre 时 0），靠 F 不回落挡。② 崩溃恢复抛弃的根在重开时全部暂时读不出，影子账读不到它们的账、隔离 0，之后别的实例复用了它们的槽，撤故障再把新实例的根全改坏，恢复落在被抛弃的根上读不出——与回退形态无关的多重故障，见 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）——乙-配置续之后这一形只剩被抛弃根是系统配置没见证到的那一种：见证过的在崩溃恢复那次挂载里重读一次仍读不出就拒可写，抛弃不了。③ 两个状态来回回退超过根环一圈时，可退到的不同状态少于 4 个（D16（发布语义） 已定项 1 的根环容量边界）。
- 实现的链首锚在所选根覆盖的最后一条（`crates/singlefs-replay-prefix/src/lib.rs` 的 `counter_of_the_last_record_the_root_covers`，`crates/singlefs-core/src/recovery.rs` 的 `replay_journal` 经 `replay_core` 取前缀），对着它的会红检查欠在 C365（恢复路径的链首不锚在所选根覆盖的最后一条）。
- 回退时标签 6 的 n_live 现算读法（「准入与记账」一格）被攻过零轮。
- ⚠️ **只读复核买的是落点持续坏那一类，代价是它自己零轮。** 固定落点探针写单独用时，落点持续坏那一类会反复白切换（攻方在副本装置上量到 24 个持续故障场景合计 46 次，那个数没在入库装置重做过、不进 kb）；只读复核能分辨它。这条改法是攻方腿自己提的，**被攻过零轮**，用户 2026-09-21 知情接受。
- **切换重新读盘择根带出的三件**：读盘选出的根比内存里记的新时，不重发在飞 checkpoint，以读盘选出的根为准接着走；切换写的实例表行与 W 按读盘选出的根写；回退那次发布里，管理员选的 R_old 必须在「以读盘选出的根为最新根」算出的回退候选集里，不在就拒绝、盘上逐字节不变。挂载内实例切换的实现随里程碑「覆盖写、释放、回退与复用」收口表第 16、40 行在后面的里程碑（C458（实例切换取内存里的根，不重新读盘））。
- ⚠️ **零单元发布这一支不冻结**：发布末尾系统配置轮换之后那道屏障报错时，带单元 / 带记录的发布与轮换报错同一处置（冻结、原样重发）；零单元发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`，末尾走 `persist_the_root_then_rotate_the_system_configuration` 那一路）这一支不冻结、整个挂载返回错误（失败经 `PublishError::BlockDevice`），下一次发布落在下一次挂载、换了实例代号；这是替条款没写的一支做的选择（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「最新根读不出、见证与屏障」那一格；`research/prompts/m2-impl-c577-barrier-implementer-report.md` 第 106–111 行「停下交主 agent 的设计问题」第 2 条）。钉它的用例是 `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs` 的 `a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error`：只做过 mkfs 的池第一次可写挂载，写行那次零单元发布在盘 1 轮换之后那道屏障报错一次 ⇒ 挂载报 `MountError::Publish`、原因 `PublishError::BlockDevice`，写账已落盘的发布 0 份、失败的 1 份；录制流落到盘 0 那道屏障为止，盘 1 那一道没进录制流；换回不报错的设备之后再可写挂载，取号 2、择到 (实例 1, txg 1)。它只钉文件镜像上的结局：harness 的文件设备没有写缓存，盘 1 的轮换写在屏障报错之前已经进了镜像，「屏障报错时轮换其实没持久」那一种崩溃状态它不罩（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md`「结论」第 4 行、「停下交主 agent 的」第 5 条）。
- ⚠️ **C366（暖机路径把 txg 写进计数器与 tail） 那条定案在可写挂载这一层没有可达盘面**：注 3「前缀末的计数器取环里读得出的最大那一个」只在所选根自己那条记录两份都读不出时与 txg 分得开；那一形在可写挂载的读阶段被 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙判成「两条都读不出按真」，重读一次仍为真就在取号之前拒可写（C579（判据在单故障合法状态上也拒可写） 用户定维持乙字面），走不到取号、写行与暖机。今天钉住的是：函数层 `crates/singlefs-harness/tests/warm_up_journal_counter.rs` 的 `warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two`（环里末条 jsn 40 时暖机写 jsn 41、42，txg 仍是 1、2）；同一份文件的 `c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root`：可写挂载拒在任何写之前、两块盘逐字节不变，只读挂载照常择 (实例 1, txg 4)、环里读得出的 jsn 1–3 一条不施加（`research/prompts/m2-investigate-three-reds-report.md` 第一节第 2 条、第七节；`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节 5c 行）。
- 实例表删行的清扫（D18（块里携带什么信息） 已定项 11 的「回收」一条）保护被抛弃根引用的单元：读成功的全零根槽与读不出同待遇（按「可能有该实例的根」算），这一次不删；其余的槽须自证过，否则重读一次、仍不过就这次不删（零轮，推的，没造盘面）。

**依据**：

- E78（重放的起点）：陈旧 tail 叠上块重用之后，「全环扫描 + 断号即止 + 幂等」三件套不闭合——失配即中止的恢复在健康镜像上自我中止，失配跳过的撕裂判别力归零；水位臂与尾删臂都过全部判据，取水位臂是因为水位就是根记录已有的 `(实例代号, checkpoint_txg)`（D22（单元原子性怎么合成） 已定项 7），零新增格式字段，而尾删臂要在恢复路径里加一个重试环。
- E77（发布的持久顺序）：拿掉施加前的点名单元验证，单元与记录同段那一臂当场出静默嫁接——第四条承重。
- E104（扫描重建的现行版本判定）：新根取 T_old + 1 时压不过被抛弃的根——注 2、注 3 新实例的第一次发布从环里最大 + 1 起靠它；它的另一半「不写回退行时被回退抛弃的单元在全规则臂上复活」仍承重——今天的设计就是不写回退行（墓碑第一轮判决 `research/prompts/m3-tomb-r1-main-verification.md` 第二节、`research/prompts/m3-delete-r3-main-verification.md` 第三节「墓碑第一轮「一轮」各行」）；扫盘重建认「挂载根是一次回退」靠谱系记录，读侧怎么用记欠账（K-W5b，与 C113（扫描重建时多版单元的现行版本判定无输入） 同组）。
- E150（回退复用被抛弃的根引用的单元）：影子账在模型里两格零违例，多隔离的单元到被抛弃的最新根被轮转覆写为止。模型里的被抛弃时间线是管理员回退造出来的；今天影子账留着的理由是崩溃恢复抛弃的时间线（第一轮 H6），机理相同，模型没重跑那一形。
- E142（新池新建文件的干跑）：干跑时装置取「两份镜像任一份合法即在」，改坏一份记录仍读到全部合法记录——「记录在」的口径由此收口。
- 三方：`research/prompts/c199-r1-main-verification.md`、`research/prompts/c199-r2-main-verification.md`（注 1–4）；`research/prompts/c322-r3-main-verification.md`（取号屏障与跳过的号）；`research/prompts/c329-c330-r3-main-verification.md`（切换的所选根与 W 那一行）；`research/prompts/c143-r3-main-verification.md`（第一个新根的 txg 取 CJ2）；`research/prompts/m2-r1-main-verification.md` 第三节与 `research/prompts/m2-step45-code-r3-main-verification.md` 第六节 5（影子账查谁的账、有效根）；C113（扫描重建时多版单元的现行版本判定无输入） 定案（`_c113-r9-final.md`）。
- 用户定案（重放下界、回退、失败表、切换预留口径、两份镜像的口径各一次），原话在变更史。
- `research/prompts/c381-r3-main-verification.md` 第三节：候选乙（探针写改成写失败的那个落点）被三处独立打中——屏障上没有落点、8 个失败点里 4 格的落点是权威结构、与用户第二轮定的第 1 题（落点进地址空间表，买的就是「写坏了也不心疼」）对撞、`D2`（RAID 条带策略） 已定项 13 那句话会当场为假。
- 用户定案 2026-09-21（改法 B），原话在变更史。
- 用户定案 2026-09-23：切换的所选根重新读盘择根（`research/prompts/c381-r3-main-verification.md` 第二节记的零轮形态、第四节第 3 题），原话在变更史；被攻过零轮。
- 用户定案 2026-09-23：注 3 的前缀末计数器取环里读得出的最大那一个（C366（暖机路径把 txg 写进计数器与 tail） 实现员查出所选根自己那条记录读不出时条款没写），原话在变更史；这条定案在可写挂载这一层没有可达盘面（射程「C366（暖机路径把 txg 写进计数器与 tail） 那条定案在可写挂载这一层没有可达盘面」那一条），C579（判据在单故障合法状态上也拒可写） 用户 2026-09-27 定维持乙字面、拒可写（`research/prompts/m2-investigate-three-reds-report.md`），原话在变更史。
- 用户定案 2026-09-23：第五条的发布边界按已定项 17 那条约定认末条（C245（系统配置槽的写频率，两条条款说反话） 第二轮 `research/prompts/c245-r2-main-verification.md` 第四节列的三个判别量之一，被攻过零轮），原话在变更史。末条按记录标志位 0 认（已定项 17）；一次发布的记录里没读到带标志的那条，整次不施加。
- 用户定案 2026-09-23：注 1 的锚点取同 txg 里 jsn 最大的那条、锚点读不出时靠本次发布内序号接链首，第一版没有干净关闭标记（三方第一轮 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K3、K4；K4 的两个改法都被攻过零轮，本次发布内序号那一个是推的、没实现），原话在变更史。
- 用户定案 2026-09-24：第一个文件版本写之前核上一条记录，照今天的实现写成条款（C495（第一个文件版本核上一版记录没有条款），三方 `research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节 R5：拒之前零字节写、被拒那一族零故障不可达），原话在变更史。
- 用户定案 2026-09-24：切换重新读盘择根带出的三件照主 agent 推荐定（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N5–N7，原话在变更史）；无实验：三件都不改盘上格式、实现不在这个里程碑，差别只在语义。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。
- 用户定案 2026-09-24：末条按标志认、注 1 的锚点读法乙（原话在变更史；实十三报告 `research/prompts/m2-p6-header-implementer-report.md` 第四节 ① 列的两种读法）。
- 用户定案 2026-09-24：这一版发布失败原样重发（实二十报告 `research/prompts/m2-lastflag-implementer-report.md` 第六节 1：同一实例失败后接着发，会留下两条同 (实例代号, checkpoint_txg) 都带末条标志的记录），原话在变更史；无实验：只定失败之后发什么，不改盘上格式。
- `research/prompts/m2-final-code-r2-main-verification.md` 第三节 Z8（这一版发布失败原样重发，攻方没打中）；那一份的 Z9 与第四节改法 1–4 是回退见证的，随见证删掉不再承重。
- 管理员回退改成挂着时的一次向前发布（候选集、释放、复活、水位、拒）、删前缀第五条与回退见证、影子账依据改指 H6：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`。水位取 max(内存, 环) 的收严（第三轮判决第二节 K1 的水位分句）被攻过零轮；用户定案 2026-09-25（向前发布、只在挂着的时候做）与 2026-09-26（第三轮交用户的四问），原话在变更史。
- 主 agent 2026-09-26 定：回退那次发布之前逐盘验复活集里的每个单元（「在任何写之前拒」那一格最后一句），照用户 2026-09-25 原话「1 就是fsync失败后 两个盘掉一个盘。这个不能认，这个违背我们数据安全的承诺」（`records/2026-09-24-里程碑二收尾调度.md` 第三节那一时刻的一行），与 C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格「两份都验过才施加」同一方向；无实验，推的，被攻过零轮；多读的量随 R_old 之后释放的用户可见单元数涨，没量。
- 用户定案 2026-09-25：点名单元两份都验过才施加、认下丢一整块盘时丢掉刚确认的那一版（C519（丢一整块盘时恢复丢掉刚确认的那一版） 实一复现：`crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs`），原话在变更史；无实验：照今天的实现写成条款，另一读法「任一份验过」会不会放过 E77（发布的持久顺序） 那种嫁接没量。
- 用户定案 2026-09-27（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续，先选乙、再补选「配置续（推荐）」）：可写挂载读到系统配置见证过的更新状态读不出，重读一次仍读不出就拒可写；取号那一写不再写 tail = 0，改写取号那一刻按同一取法读到的见证值（实现已合入，`crates/singlefs-core/src/transaction.rs` 的 `highest_system_configuration_journal_tail`）；判据 N-配置续 定义在 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行，原话在变更史。E158（择根与修复四岔路） 第 4 次跑：第三段 H1g 把乙-配置与乙-配置续分开（16/32 对 0/32），第二段（以装置撞键修过的 seg2b 为准）四族两者逐族相同，页上「2026-09-27：第 4 次跑」一节的岔路表第 1、2 行。
- 回退时标签 6 的 n_live 现算读法：`research/prompts/m3-blanks-r1-main-verification.md` 第四节「回退那一刻 n_live 怎么现算」那一行（零轮；主 agent 采纳正推腿的读法）。
- 三方判决 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 9 行：清扫把读成功的全零根槽当没有根，会抹掉仍在环里的被抛弃根引用的单元，违反这一项正文明文的保护；全零根槽不当「没有根」、每槽自证过或全零否则重读一次仍不过就不删，两处收严采纳（零轮，推的，没造盘面）。无实验：保护的对象是被抛弃时间线的残余，不是活数据。
- 回退到删完的一版（有树、文件集合为空）算候选：主 agent 2026-10-02 定「Q-K 照收」，候选集原文括注「那一版写过文件」的读法（调度记录 `records/2026-10-01-删除与并发事务.md` 第 320 行）；2026-10-03 主 agent 照收第九位实现员的同一选择（「回退到删完的一版照收，与 Q-K 一致」）。无实验。
- 重建分配器读根环两遍并起来没有有效根的拒另起拒因、单子加一行，与候选集去掉一圈窗口那一个合取：用户定案 2026-10-05（Q3「另起拒因、单子加一行」与按槽并两遍），原话在变更史；判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节（`Recovery(NoValidRoot)` 不在单子上那一行）与第六节、`research/prompts/m3-batch10-code-r2-main-verification.md` 第六节；复算判据取自 `crates/singlefs-harness/src/mount_transparency.rs` 今天的复算。候选集那一个合取是 D16（发布语义） 已定项 1 的六处退回环里最旧有效根（用户 2026-10-04 定）之后漏改的残留，调度记录第 601 行点名。无实验：数在三方腿的副本上量，没立实验号。

**欠**：C77（重放起点未定义）；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）；C331（择根倒挂压过已确认的写）；C334（切换的所选根没有会红的检查）；C365（恢复路径的链首不锚在所选根覆盖的最后一条）；C287（切换收养开放 checkpoint 的事务后再崩）；C126（切换预留的最坏量没有口径）；C493（回退候选集条文与实现说反话）；C381（根已落盘之后发布失败，分配器仍退回）——三方第二、三轮打中失败表那两支判别子（探针写在盘上没有落点、屏障类失败点连「失败的那个落点」都没有定义），用户已定「落点进地址空间表、射程逐个失败点列一张表」，第 2、4 题已定（`D16`（发布语义） 新立一条已定项、失败表改两步判别子），「实例切换取的是内存里的根」那一题定成切换重新读盘择根、带出的三件待定（`research/prompts/c381-r2-main-verification.md`、`c381-r3-main-verification.md`）；代码那一半（探针写、只读复核、实例切换、转只读）跟里程碑「覆盖写、释放、回退与复用」收口表第 16 行一起挪到后面的里程碑；C458（实例切换取内存里的根，不重新读盘）：切换重新读盘择根那条会红的检查；C539（锚点读得出时下一次发布的首条序号不是 1）；C540（末条标志坏在一次发布中间，读者切出两次发布）；C541（原样重发把一次失败放大成整条发布流阻塞）；C558（回退目标是环里最旧的根时回退那次发布会写坏它）；C596（撕裂的系统配置轮换之后只剩只读挂载）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:450-464`（整段抄，未转述）**

```markdown
#### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根

**定案**：**由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 实例表单元指针 86 + 分配记录树根指针 86 + 卸载记号 4 = 364 字节 <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 364 stale=新根段 188 字节|JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->，4096 记录装 64 个点名项。施加一条记录 = 把所选根的这七个字段换成记录里的（实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根），不需要树语义。一次发布切成几条记录时，每条记录都带同一份新根段。已定项 14 条 2「严格大于所选根就施加」原样成立。**

**射程**：管「记录在、根槽不在」那一格恢复施加什么；指针的宽度随 D19（块指针的结构与宽度预算） 已定项 7 / 11 走。施加一条记录时指针层上其余的事（分配器登记、overlay）不在这一格，归 C284（施加一条记录在指针层上做什么没有定义） 与 D28（挂载期承诺量） 已定项 2。

**依据**：

- E142（新池新建文件的干跑）：新根段加进记录头之后，「施加一条记录」在指针层上有了定义，忽略 journal 与查 journal 的恢复结果在一批状态上不同——journal 承重。
- 用户定案，原话在变更史。
- 用户定案 2026-10-01：「多条记录的发布每条都带同一份新根段」补进已定项 15（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节第 3 条），原话在变更史。
- 三方判决 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行：施加处的实例表指针、卸载记号、分配记录树根三项改从新根段来（首个文件、卸载两种发布补进去之后施加出来的根与当初会写的逐字节相同，攻方量的；专用清扫发布崩在记录已持久、根槽没持久时，实例表指针照抄所选根会让下一次可写挂载在取号之前被拒，攻方在副本上量的）；用户定案 2026-10-02「回到甲」，原话在变更史。无实验：数都在三方副本上，没立实验号。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/invariants.md:152-152`（整段抄，未转述）**

```markdown
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树（主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定，被攻过零轮；只接这一步——恢复连着施加几条记录时第二条及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
```

**出处 `.claude/kb/checks-owed.md:1-16`（整段抄，未转述）**

```markdown
# 欠的检查清单

**checks-owed.md 放「已经知道要拦什么、但现在还拦不了」的检查。**

与 [invariants.md](invariants.md) 的分界：不变量是**对着一个镜像**可判定的性质，
checker 是它的可执行形式；checks-owed.md 放的是**代码路径的要求**——
它们对着镜像判不出来，只能由门禁在跑的时候拦。

**写进来的门槛**：说得出「拦什么」「怎么拦会红」「现在缺什么前置」。
三样缺一样就是还没想清楚，先别写
（`.claude/singlefs-ai-sop/rules/sop-first.md`：写不出下一步就别加那条检查）。

<!-- doc-lint:registry name-col=2 -->

| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |
|---|---|---|---|---|---|
```

**出处 `.claude/kb/checks-owed.md:584-584`（整段抄，未转述）**

```markdown
| C735 | GPU 判器里由记录施加的版本没有对拍流与变异行 | GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 里 I-5.4（分配记录罩住的槽互不相交） 按版本判的那一圈，第 34 批 34-1 拆出 `judge_allocation_records_disjoint_of_a_version`，候选根、`R_DATA` 里由记录施加出来的那几版、下一次挂载先施加的那一版各调一次。`R_DATA` 里由记录施加出来的那几版那一次调用，没有一条对拍流造得出「由记录施加出来的版本里两条记录重叠」，也没写变异行，去掉它 GPU 判器照绿（变异下会存活）。下一次挂载先施加的那一版有一条对拍流（`the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first`）与一行变异（`crates/mutations.tsv` 里名字带「第34批 34-1 GPU」那一行），两样都在 checker 档、没跑、没证红 | 造一条录制流：一个版本只由记录施加出来（记录已落、根槽没落，或挂载之后只由记录施加），它的分配记录叶里两条记录罩住同一个槽，CPU 判器红，GPU 判器也要红；对拍加这条流，再加一行变异去掉 `R_DATA` 那一次调用，对拍必须由绿转红 | 这一形的录制流今天造不出（34-1 实现员没找到造法）；GPU 对拍与变异在 checker 档（要一张 NVIDIA 独显），归提交时 | 2026-10-06 `records/2026-10-01-里程碑三形式化接入.md` 第 34 批 34-1 交回那一条（「GPU 里由记录施加那一圈没有对拍流与变异行（记欠账）」）；34-1 实现员报告「变异」一节末句与「没做什么」一节最后一条 |
```
