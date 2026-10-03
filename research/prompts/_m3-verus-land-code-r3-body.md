# Verus 核接回 `crates/`：代码轮第三轮正文（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 J1 J2 J3 J4 J5 S02 S03 S05 S06 S13 S14 K02 K19 K20 -->

## 一、题面

`.claude/rules/implementation-workflow.md` 三步里的第 2 步，第三轮（最终轮，`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不再开轮）。代码轮第二轮判决 `research/prompts/m3-verus-land-code-r2-main-verification.md` 第二节 H2、H4、H5 三行与第四节第 1 条的改法，已由第 5 批实现员 5-1 落进 `crates/` 并合入（`records/2026-10-01-里程碑三形式化接入.md`「第 5 批开工」之后各条）。这一轮只攻 5-1 的改法本身（被攻过零轮），与前两轮站住的判决够不够得着；不重复第一轮（C1–C8）与第二轮（H1 改后核与 fcc4d340 等价、H2 spec 证到了什么、H3 报错路径、H4 第 4 批测试与变异、H5 第 4 批编码纪律、H6 E165 `--pinned-cases`）各自的攻法。

被判的文件（判决逐份写去向）：

| 文件 | 5-1 改了什么 |
|---|---|
| `crates/singlefs-replay-prefix/src/lib.rs` | H5-1：C540 扫描里删掉计数器是 u64 最大值那一支、`next_counter` 与「≥ 下一号」子式，循环不变式里 `next_counter` 那一条一并删，进 if 处补引理调用；H5-2：删掉链首判定里冗余的 `|| record.ordinal_within_publish.0 != 1`；H5-3：核的四个类型改名 `PrefixKernelInstanceGeneration`、`PrefixKernelCheckpointTxg`、`PrefixKernelJournalRecordOrdinalWithinPublish`、`PrefixKernelJournalRecordPlaceInPublish`；H5-8：`CandidateVerdict` 两个成员改名对齐 spec（`FailsACheckBeforeTheNamedUnits`、`NamedUnitsReportedUnverified`）；核头注释第 9 行「读盘次数与次序同原文」改成指到新用例 |
| `crates/singlefs-replay-prefix/verus-mutations.tsv` | 94 行变 93 行：改名 3 行、改锚 2 行、删「下一号改成加二」（它改的那条不变式随 H5-1 删掉）；9 行期望改成实测值（其中 4 行的错误数含一行 rlimit 用尽） |
| `crates/singlefs-core/src/recovery.rs` | H5-3：胶水改用新名字；H5-4：「新根从哪来」由 `Option<usize>` 改成胶水里的两成员枚举 `NewRootSource { ChosenRootItself, NewRootSegmentOfTheRecordAtKeyOrderIndex(usize) }`，注释删掉 |
| `crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs`（新） | H2 / H4-c：水位之上有候选、`mount_read_only`，逐次记录块层读，钉扫完环之后紧接的 50 次读与判完之后环里 0 次读 |
| `crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs` | H5-5：根与记录真共用 fsid，`0x53` 起名 `FILESYSTEM_IDENTIFIER_BYTE` |
| `crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` | H5-6：魔法数起名，`4096` 改取几何里的 `fixed_structure_slot_spacing` |
| `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` | H5-7：两个辅助函数合成一个，参数用枚举 `RollbackFloorHandedToTheWriter` |
| `crates/mutations.tsv` | 整行改写 4 行：K20（锚到扫描起点 `let mut later_index = index + 1;` → `index + 2`）、「步 3 三方第一轮…」「P6 后一半…」（锚点随 H5-2 改）、「实二十：层 0 L8…」（只因改名）；追加 2 行：S03（每个候选调两次反向链）、S05（点名回调提前到所有判定之前），点名新用例 |

H5-3 实现员选了改名、没选一处定义：`singlefs-core` 在 `crates/singlefs-core/src/journal.rs` 给其中两个类型写了固有方法，外 crate 的类型上写不了（实现员报告，主 agent 没核）。

## 二、实现今天的样子（主 agent 的观测，2026-10-01，主工作区合入之后）

- 冻结副本 `/tmp/claude-1000/m3-verus-land-code-r3/tree/`（`crates/`、`Cargo.toml`、`Cargo.lock`），sha256 清单 `research/prompts/m3-verus-land-code-r3-snapshot/crates-sha256.txt`（217 份）；读冻结副本，不读主仓 `crates/`。第二轮的冻结副本 `/tmp/claude-1000/m3-verus-land-code-r2/tree/` 是改前；被判的 diff 在 `research/prompts/_m3-verus-land-code-r3-diff.md`。两份冻结副本之间别的会话还改了 `crates/singlefs-core/src/block_device.rs`、`mount.rs`、`mounted_session.rs` 等十几份，不在这一轮判；主 agent 现查被判的 6 份旧文件把 5-1 的补丁反着打回去与第二轮冻结副本逐字节相同。
- 主 agent 现跑：Verus 0.2026.09.20 `--no-cheating` 在冻结副本的核上「32 verified, 0 errors」（`/tmp/claude-1000/m3-verus-land-code-r3/verus-main.log`）；合入验证 harness 档 101 个测试二进制全绿、clippy 过，fmt 只红在别的会话的 `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs`（`/tmp/claude-1000/merge-and-verify/2026-10-01-1262960/`）；`crates-mutation-rows.py static-check .`「CHECKED 1558 202 9」、没有 ANCHOR；实现员没证的「步 3 三方第一轮…」「P6 后一半…」两行主 agent 在副本上 `prove-red.sh` 证红（`/tmp/claude-1000/m3-prove-red-5-1/tree/prove-red-logs/001.log`、`prove-red-logs-3/001.log`）。`verus-mutations.tsv` 93 行主 agent 逐行复核在跑（`/tmp/claude-1000/m3-verus-rows-5-1/rows.out`），腿交齐之前写进判决。
- 实现员报告：`/tmp/claude-1000/m3-verus-codestyle-5-1/report.md`（sha256 ffb17ed2…7dc7aebc9），逐条写了验收命令与原样输出、`verus-mutations.tsv` 9 行期望的改前改后（`/tmp/claude-1000/m3-verus-codestyle-5-1/verus-expectation-changes.tsv`）。
- 没做的：「实二十：层 0 L8…」那一行的目标在 checker 档，证红留给提交时的崩溃验证员；E165 第一段在改后树上逐字节重跑归第 5 批 3-2，与这一轮并行，不在这一轮判。

## 三、格

### J1　5-1 的改法是不是纯重构
H5-1 删死代码、H5-2 删冗余判定、H5-3 改名、H5-4 换枚举、H5-8 改名都声称结局不变：改后核与第二轮冻结副本的核在任何输入上还给同一个结局吗（返回的根、报告五字段、报错种类与内容、回调调用的次数与次序）；H5-1 删掉「计数器是最大值就交回 false」之后，核的 `requires`（计数器加一不溢出）与胶水的 `counter < u64::MAX` 断言是不是仍罩住原来那一支要挡的输入；H5-2 删掉之后，「首条序号为 1」那一判是不是对每种走到链首的路径都管住（没有锚点、锚点读不出、锚点在别的发布）。

### J2　新用例证不证得到「读盘次数与次序」
`mounted_recovery_reads_each_candidate_once_in_order.rs` 钉的 50 次读与「判完之后环里 0 次读」，是不是它名字说的那件事；第二轮攻方 `research/prompts/m3-verus-land-code-r2-opus-model/call-pattern-rows.tsv` 里 S03–S07 五行（只改回调调几次、次序、对哪条调），套到今天的核上，这条用例逐行红不红；核头注释改后那一句是不是属实（用例罩住的是哪几种候选形状，罩不住的有没有写出来）；`mount_read_only` 走的路径与可写挂载、`recover` 是不是同一条取前缀路径。

### J3　变异表的改动
K20 改锚到 `index + 2` 之后还是不是「只从紧挨着的下一条之后找」那一判的变异；「步 3 三方第一轮…」「P6 后一半…」两行改锚后的替换文还是不是原来那一判；`verus-mutations.tsv` 删掉「下一号改成加二」那一行之后，那一类错（跳过紧挨着的下一条）在 Verus 一侧还有没有一行必红世界罩着；9 行期望改成实测值、其中 4 行错误数含一行 rlimit 用尽：门禁 checker-tier-formal-verification 的 verus-must-red-worlds 格按期望片段判，这 4 行换 rlimit 或换 Verus 版本时会不会误判。

### J4　编码纪律（5-1 新写的代码）
`.claude/singlefs-ai-sop/rules/code-discipline.md`「名字」「注释、断言、常量、重复」：`PrefixKernel…` 前缀之后，同一概念是不是仍有两个类型（核与 core 各一个，胶水手抄映射还在），「一个语义概念全仓一个名字」算不算还清；`NewRootSource` 的成员名与它在核里的角色对不对得上；新用例与 H5-5、H5-6、H5-7 改过的三份测试里的魔法数、注释与代码对不对得上；逐条给文件:行。

### J5　前两轮站住的判决（辩方复核）
第二轮判决里判「没打中」的 H1、H3、H6 与「照字面没打中」的 H2、H4，在今天的代码上还够不够得着：第二轮正推的差分（`research/prompts/m3-verus-land-code-r2-sonnet-model/`）在改后核上重跑还是 0 差异吗；第二轮判 H5 打中的 8 处，有没有一处其实不违例（判错了、或改了反而更差）；两轮报出的数（58 904 864 个产品输入零差异、m1–m4 阳性对照 1581 / 3821 / 545 / 520、Verus 32 verified、必红表逐行红）今天能不能核。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | J5 为主，J1、J2 | 复核第二轮判决够不够得着、是不是同样打中所有替代方案（H5-3 的两条改法、H4-c 的「spec 加读盘日志」与「补用例」）；替被判打中的一方辩护；在冻结副本的拷贝上重跑第二轮正推的差分 |
| 云端攻方（Opus） | J1–J4，重点 J1、J2、J3 | 攻 5-1 每一处改法：造反例、造变异，在冻结副本的拷贝上量；Verus 可以在自己草稿目录的副本上直接起（`~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus`，`--no-cheating`，经内存包装），rlimit 写成 10 |

第三轮：前两轮都派正推，这一轮派辩方不派正推（`.claude/rules/three-way-inference.md`「多轮」那一节：至少一轮派辩方复核前一轮判决）。

## 五、跑前写死的判据

- 改后核与第二轮冻结副本的核在某个输入上结局或回调调用不同 ⇒ 打中 J1，写回代码。
- S03–S07 里有一行套到今天的核上、新用例照样绿，而核头注释说那一类由这条用例钉住 ⇒ 打中 J2。
- 一行变异改锚后不再是原来那一判，或一类错在 Verus 一侧没有必红世界罩着而之前有 ⇒ 打中 J3；期望片段换 rlimit 或换版本会误判，给出复现命令就算打中 J3。
- 编码纪律违例给出文件:行就算打中 J4。
- 第二轮判决的一格在今天的代码上判反，或报的数今天核出不同 ⇒ 打中 J5。
- 腿提的新改法算零轮，标「零轮」；这是最终轮，零轮改法写进判决的交用户表，不再为它开一轮。
- 不提前优化 `crates/` 里 core 的实现代码（用户 2026-10-01 定）：只攻正确性、证明、测试与编码纪律；为了快去改 core 的改法不提，提了判决不采。

交付：腿的报告写 `research/prompts/m3-verus-land-code-r3-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-verus-land-code-r3-<腿名>-model/`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈，读冻结副本。
