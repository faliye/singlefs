# m2-layer0-scale-r1 本地攻方腿：逐句核转述

按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」：每一句转述与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注单列一行并写明为什么加。

列：英文项（用于提示里的原样英文）/ 原文文件:行 / 首稿缺的 / 定稿说明。

## 一、Table 1（发布顺序与写倍数的一般事实）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 1 Row 1: "The persistence order of one publish is always: COW units/nodes, then a barrier, then a journal record, then a barrier, then the root slot (an FUA write), then the system-configuration slot; fsync does not return until the root slot is persistent. The system-configuration slot is updated only after the root slot is persistent, once per checkpoint." | `.claude/kb/decisions/16-发布语义.md:176`「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽；fsync 等根槽持久之后才返回。」+「系统配置槽在根槽持久之后再更新，每个 checkpoint 一次」 | 无遗漏：「恒为」译作 always、「才返回」译作 does not return until、「每个 checkpoint 一次」译作 once per checkpoint，逐一对应 | 未加字；两句原文本不相邻（同一行内的两个分句），拼接时用句号断开，未改变原意 |
| Table 1 Row 2: "Once it returns, that generation is guarded by that one root slot alone (the root slot is not mirrored)." | `.claude/kb/decisions/16-发布语义.md:180`（节选）「返回之后这一代只由那一个根槽罩着（根槽不镜像）。」 | 无遗漏：「只由」译作 alone | 未加字 |
| Table 1 Row 3: "A mirrored double write (the general w >= 2 mirroring rule, and specifically journal-record mirroring) counts as one write per device in this harness's own write stream." | `.claude/kb/decisions/13-验证路线.md:71`（节选）「镜像双写（D2 已定项 6 的 w ≥ 2、D22 已定项 8 的 journal 镜像）按设备各算一次写，harness 的状态数按自己的写流另算闭式」 | 首稿漏了原句后半「harness 的状态数按自己的写流另算闭式」——与本轮问题无关（那半句讲 harness 自己另算闭式，本提示已经给了自己的闭式定义），核对后判定为可省，不算遗漏限定词，只是省了不相关的后半句 | 未加字；省略的半句已在核对表本行写明理由 |

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 1 Row 4: "The barrier at the instance-number-taking step: after instance-number-taking's two system-configuration writes and before this instance's first non-system-configuration write, at least one completed barrier is required. On the first-mount path, warm-up's own opening barrier counts for this, and no separate barrier is issued. Before that barrier completes, none of this instance's writers issue any unit, record, or root write. If that barrier reports an error on any device, instance-number-taking fails all-or-nothing, and it is not permitted to just reissue the barrier and continue." | `.claude/kb/decisions/23-journal的角色与格式.md:449`「取号那一步的屏障：取号那两次系统配置写之后、本实例第一个非系统配置写之前至少一道完成了的屏障，首次挂载路径上暖机开场那道就算、不另发；那道屏障完成之前本实例的全部写者都不发单元、记录、根；那道屏障在任一块盘上报错 ⇒ 取号全或无失败，不许只重发屏障就继续。」 | 首稿漏「至少」（at least）与「完成了的」（completed）两个限定词，第二稿补上「at least one completed barrier」 | 加了一处：「暖机开场那道」译成「warm-up's own opening barrier」，多出的 own 是为了英文可读性加的所有格，不改变「那道屏障是暖机自己开场那一道」的原意，不是新增事实 |
| Table 1 Row 5: "An empty publish is also a publish. For the first writable mount's warm-up, the tree table has zero entries at that point, therefore the warm-up writes zero units of its own." | `.claude/kb/decisions/16-发布语义.md:215`「空发布也是发布」+「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元」 | 无遗漏 | 加了「of its own」与「at that point」两处，用于把「树表 0 条」明确成「在暖机那一刻」，把「零单元」明确成「暖机自己写的单元」，是为消除本地模型可能把「零单元」误读成「全池零单元」而加的限定词，不是新增事实（原文本身在「已定项 9」整条里就是这个意思：暖机写自己的单元，写行那次才写别的单元） |
| Table 1 Row 6: "D2 (RAID striping strategy) item 9: the first version of this system runs exactly 2 devices." | `.claude/kb/decisions/22-单元原子性怎么合成.md:70`（节选）「D2（RAID 条带策略） 已定项 9 第一版跑 2 块盘」 | 无遗漏 | 加了「exactly」，原文「2 块盘」本身就是确数，加 exactly 只是英文里把「就是 2」说得更明确，不改变原意 |

## 二、Table 2（切段机制与基线闭式）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 2 Row 1: "A barrier closes the segment before it ...; when the segment accumulated so far has zero writes ..., the barrier does not close anything — it is folded into the segment that is about to begin instead." | `crates/singlefs-harness/src/segments.rs:90`「屏障关掉它之前那一段（屏障算在被关掉的那一段里）；段里还一个写都没有时（流首那道屏障）它并进即将开始的那一段」 | 无遗漏 | 加了「instead」一词，是英文行文的自然衔接词，不改变原意（原文本身就是「零写时不关段、并入下一段」这一个意思） |
| Table 2 Row 2: "An FUA write always closes the segment it itself belongs to (...; so those earlier plain writes share its segment, and any subset of them may be the ones that persisted)." | `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」 | 无遗漏 | 加了「may be the ones that persisted」，是把原文「任意子集」这个省略式短语展开成完整从句，讲的是同一件事（哪个子集持久都可能），不是新增事实 |
| Table 2 Row 3: "A trailing run at the very end of the stream that has only barriers and no writes is folded into the previous segment; every recorded step lands in exactly one segment." | `crates/singlefs-harness/src/segments.rs:92`「流尾只有屏障没有写的那一串并进上一段——录到的每一步都恰好落在一个段里。」 | 无遗漏 | 未加字 |
| Table 2 Row 4: "The baseline (un-reduced) closed form is: crash-state count = 1 + the sum over all segments of (2^w_i minus 1), where w_i is segment i's own write count (barriers are not counted as writes)." | `crates/singlefs-harness/src/segments.rs:145`「E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。」 | 无遗漏 | 未加字；把「|段|」换成「w_i」是为了与本提示后文的记号一致，同一个量，不改变定义 |
| Table 2 Row 5: classify() 四类落点顺序的转述 | `crates/singlefs-harness/src/segments.rs:59-83`（代码，非中文原文） | 不适用：这是对 Rust 代码逻辑的转述，代码本身标识符已是英文（SystemConfigurationSlot / RootRecordFua / JournalRecord / UnitWrite），不存在中文原文可供逐句核对 | 已在提示行内注明「this is a paraphrase of Rust code…not a translation of Chinese text」，不进本表逐句核对范围 |

## 三、Table 3（候选甲的定义与本轮问句）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 3 Row 1: "Within one segment, for any write that is not an in-place overwrite (...), if the block it lands on is referenced by no version that recovery might select, and is also not read by journal replay, then in that segment, persisting any subset of these writes yields a recovered visible state identical, item by item, to the state where none of these writes persisted at all." | `research/prompts/_m2-layer0-scale-r1-body.md:13`（L1 格第一句）「一段写里，凡不是原地覆盖的写（COW 新写的单元、分配记录、映射条目等），它落的块不被任何「恢复可能选中的版本」引用、也不被 journal 重放读到 ⇒ 这一段里这些写的任意子集落盘，恢复出的可见状态与「这些写一个都没落盘」逐项相同」 | 无遗漏：「也不被 journal 重放读到」的「也」已译成「and is also not read by」 | 未另加字 |
| Table 3 Row 2: "Only in-place overwrites (the root slot, the system-configuration slot, the record slot inside the journal ring) need to be enumerated subset by subset." | `research/prompts/_m2-layer0-scale-r1-body.md:13`（L1 格第二句）「只有原地覆盖的写（根槽、系统配置槽、journal 环里的记录槽）要逐个子集枚举」 | 无遗漏 | 未加字 |
| Table 3 Row 3: "The question: under candidate A, how many states remain for the second stream's segment sequence as it stands today, and how is the closed form computed." | `research/prompts/_m2-layer0-scale-r1-body.md:17`（L5 格）「甲之下第二条流（今天的段序列）剩多少个状态、闭式怎么算」 | 无遗漏 | 「甲」音译改叫 candidate A，是本地腿英文提示的通用做法（中文候选名一律译成 candidate 加字母），提示开头 Context 段已交代 |

## 四、Table 4 / Table 5（数据与来源说明的转述部分）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 4 各行的数字与总数（423 次写、54 段、闭式 5575802973、5 段 30 写占 96%） | `records/2026-09-24-里程碑二收尾调度.md:162` | 不适用：均为数据整行抄，非叙述性转述 | 数组与总数按行原样抄录，未改写；「5 段 30 写的段占 96%」译作"The 5 segments whose own size is 30 writes account for 96 percent"，逐词对应，无遗漏 |
| Table 5 Row 1 旧数组与旧闭式 2104413 | `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:287-295`（assert_eq! 字面量） | 不适用：数据 | 数组整行抄（旧、右手边 right 那份） |
| Table 5 Row 2「positions differ」列表 | 本 agent 自算（python diff，两份数组逐位比对） | 不适用：算术观测，非转述 | 15 个差异位置 7,10,14,17,20,23,27,30,33,36,39,42,45,48,51 已用 python 现跑核对（见运行记录） |

## 五、Table 6 引的三段测试文件注释（Quote A / B / C）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Quote A: "B's two system-configuration-slot writes merge with instance-number-taking's two into one segment (4); the write-the-row publish issues 10 unit writes (the instance-table row plus four fixed-point units, each on two devices); each warm-up's 8 unit writes merge with the previous publish's two system-configuration-slot writes into one segment (10); C's 16 unit writes merge with txg 7's system-configuration-slot writes into 18." | `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:252-253`（ThirdVersion 注释）「B 的两个系统配置槽写与取号的两个合成一段（4）；写行发布 10 个单元写（实例表 + 四个固定点单元，各两盘）；每次暖机 8 个单元写与上一次发布的两个系统配置槽写合成一段（10）；C 的 16 个单元写与 txg 7 的系统配置槽写合成 18。」 | 无遗漏：四个分句逐一对应（B 的槽写合成 4、写行 10、暖机合成 10、C 合成 18） | 未加字 |
| Quote B: "C's two system-configuration-slot writes merge with the rollback's instance-number-taking's two into one segment (4); D is the publish that writes the rollback row: 10 unit writes (the instance-table row plus four fixed-point units, each on two devices); one warm-up's 8 unit writes merge with D's two system-configuration-slot writes into one segment (10)." | `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:269-270`（RollbackToFirstVersion 注释）「C 的两个系统配置槽写与回退取号的两个合成一段（4）；D 是写回退行的发布：10 个单元写（实例表 + 四个固定点单元，各两盘）；暖机一次 8 个单元写与 D 的两个系统配置槽写合成一段（10）。」 | 无遗漏 | 「回退取号」译作「the rollback's instance-number-taking」，与「取号」统一译法一致，未增删事实 |
| Quote C: "Each of the four overwrite-writes has 16 unit writes merged with the previous publish's two system-configuration-slot writes (18); each of the two empty publishes that raise F has 8 unit writes merged with two system-configuration-slot writes (10); E is the same shape as an overwrite-write." | `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:286`（ReuseAfterRaisingFloor 注释）「四次覆盖写各 16 个单元写并上一次的两个系统配置槽写（18）；抬 F 的两次空发布各 8 个单元写并上两个系统配置槽写（10）；E 同覆盖写。」 | 无遗漏 | 未加字 |

## 六、Table 6 六条规则本身

Table 6 六行是本 agent 自己综合 Table 1–5 与 Quote A/B/C 之后写出的分类规则（「由你照 split_into_segments 与冻结副本的写流登记推」），不是某一句中文原文的逐句转述，因此不逐行列入本核对表；每一行规则在提示里都标了它依据 Table/Quote 的哪一行，供主 agent 与核查员倒查。
