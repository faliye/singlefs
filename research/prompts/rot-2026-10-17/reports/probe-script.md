# 第二轮探针脚本（批 6）报告

没建副本，没起编译，没跑门禁，没改仓内任何文件（脚本只用 `git ls-files` 与 `git grep` 读仓）。

## 交付

- 脚本：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-probe/round2-probe.py`（草稿目录，不进仓）
- 真仓干跑候选表：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-probe/probe-dry.tsv`（五列，首行表头，共 811 行数据）
- P3 收窄后的预览（只供看效果，不是正式口径）：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-probe/p3-narrowed-preview.txt`

## 命令与原样输出

自证：`nice -n 19 python3 round2-probe.py --selftest`，末六行：

```
  ✓ 弄坏 P1：自证红在 P1
  ✓ 弄坏 P3：自证红在 P3
  ✓ 弄坏 P4：自证红在 P4
  ✓ 弄坏 P5：自证红在 P5
  ✓ 弄坏 P7：自证红在 P7
  ✓ 弄坏 P9：自证红在 P9
✓ 自证绿：每个探针该报的恰好报出，六个弄坏开关各红在点名的探针
```

`--selftest` 先在临时小仓里断言「每个探针各一条该报的与一条不该报的，报出的恰好是该报的」，再对六个探针各起一个子进程设 `ROUND2_PROBE_BREAK=<探针号>`，要求自证退非 0 且只红在点名的那一个。手设 `ROUND2_PROBE_BREAK=P7 python3 round2-probe.py --selftest` 实测：打 `✗ P7 报的与该报的不一致`，退出码 1。

真仓干跑：`python3 round2-probe.py /home/fy5090/code/singlefs --out .../probe-dry.tsv`（`nice -n 19`），挂钟约 1.5 分钟，末行原样：

```
✓ 候选 811 行：P1 84、P3 626、P4 59、P5 12、P7 6、P9 24；扫了 297 份文档
```

组名认得：`python3 scripts/kb/stale-candidates.py --check-report probe-dry.tsv <空判定表> --groups P10001,P30001,P40001,P50001,P70001,P90001` 与 `--groups P10001-P10005` 都走到「没判全」（退 3），没有报「组号区间写不认得」或「候选表没有的组」。

## 口径上我补的几处（脚本里的实现细节，不改探针本身）

- 围栏代码块（```）里的行不扫；`kb` 文件 `## 历史版本` 之后不扫。
- 句子切分用 `。；|`；P4、P5 的「原文」列写命中的那一句（表格行是那一格），其余探针超过 500 字时取命中处前后 500 字的原样子串（不加省略号），不让命中处被截掉。
- P1：反引号串按空白拆词逐词判；跳过 `../` 开头（相对命令目录）、`-` `/` `~` `$` 开头、带 `<>*…{}$?|` 的；去掉 `:行号`、`#锚`、`「…」`、`@小节`、`::函数`、`(…)`、git pathspec 的 `:!`、`HEAD~1:` 这类版本前缀；以 `-` 或 `_` 结尾的（截半的名字）跳过。不以仓内目录起头、只因含 `/` 且带扩展名入选的，另认「相对本文档目录」与「某个跟踪文件以它结尾」。「新事实」栏列同名跟踪文件（最多三个）。
- P3：反引号整串是 Rust 标识符（可带 `::` 路径、`()` `!` 尾巴，逐段判）；全大写不带小写的单词（如 `SINGLEFS`）不入选，驼峰要求带小写。排除门禁格名与登记表列名（所有被扫文档的表头、跟踪的 `.tsv` 首个非注释行）。UPPER_SNAKE 在同行写了「环境变量」、或在非 md 的跟踪文件（不含 records、research、briefs）里命中的当环境变量排除。
- P4：先把编号后面括着的简称（`C85（发布计数器的四个等号没写死）`）去掉再找字样，否则简称里的「等号」「没写死」会自己命中；`C11b` 这种带字母尾巴的不当 C11；已还清表自己那一段不扫；已还清集合减去欠的表里还在的编号。「旧事实」栏写命中的字样。
- P5：一个编号（`I-x.y`、`D<n> 已定项 k`）只看它后面到下一个别的编号之前的那一段，中间只隔「、，/ 和 与 及」的几个编号算一组；裸写的「已定项 k」继承同句前面的 D 号，只当边界用。决策分项的已实现/未实现取自 `.claude/kb/decisions/*.md` 里「### 已定项」表每行的 `**进度：…。**`（`decisions.md` 索引表只有计数，没有分项）。
- P7：「派 X」排除「指派、分派、委派、调派」；内置类型 `general-purpose`、`explore`、`plan`、`statusline-setup`、`claude-code-guide` 当存在；阶段名只要和任一跟踪文件或目录的名字（去扩展名）相同就算存在（`doc-lint`、`code-discipline` 这类）；`term-renames.md` 「旧」列按它的「匹配」列当整串或正则排除。
- P9：每行最多一条。

## 各探针行数与前 5 行样例（原文列只贴前 150 字，全文在候选表里）

### P1：84 行

| 组 | 载体 | 旧事实 | 原文（前 150 字） |
|---|---|---|---|
| P10001 | .claude/kb/checks-owed.md:22 | 路径 scripts/gate/change-touches-crates.sh 在工作区与 git 都不存在 | 1` 却有 5 个 `crates/` 路径，四道重阶段白跑 40 分钟）。**已有（按判定算单元键）**：七道重型与 harness-model-differential-and-scenarios 的每条判定按单元键入库（D13（验证路线） 已定项 17），一轮里第二、三次只现判键对不上的；按树 |
| P10002 | .claude/kb/checks-owed.md:22 | 路径 scripts/gate/stage-must-run.sh 在工作区与 git 都不存在 | ates/` 改动还会重新落进这一次门禁（暂存区零行 `crates/`，相对 `HEAD~1` 却有 5 个 `crates/` 路径，四道重阶段白跑 40 分钟）。**已有（按判定算单元键）**：七道重型与 harness-model-differential-and-scenarios 的每条 |
| P10003 | .claude/kb/checks-owed.md:102 | 路径 format-spec/inode-record.toml 在工作区与 git 都不存在 | ／ C119 ／ 冻结组件没有 spec 文件 ／ D15（格式冻结政策） 2026-09-05 立了四层之外的两个独立冻结组件（类型 2 的记录格式、统计量登记表含合并规则），C9（冻结后偷改格式） 要按组件记 `spec_hash`，而 `format-spec/` 目录与两个组件的 spec  |
| P10004 | .claude/kb/checks-owed.md:102 | 路径 format-spec/statistic-registry.toml 在工作区与 git 都不存在 | ／ C119 ／ 冻结组件没有 spec 文件 ／ D15（格式冻结政策） 2026-09-05 立了四层之外的两个独立冻结组件（类型 2 的记录格式、统计量登记表含合并规则），C9（冻结后偷改格式） 要按组件记 `spec_hash`，而 `format-spec/` 目录与两个组件的 spec  |
| P10005 | .claude/kb/checks-owed.md:107 | 路径 research/prompts/c126-r1-opus-probe/ 在工作区与 git 都不存在 | d 上 rows0 无界（C121（zoned 上实例表链长无上界））是显式例外。② 固定点重做**不另要空间**：切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；施加语义在 D28（挂载期承诺量） 已定项 2（施加登记码 1、所选根之后的码 2 / 3  |

### P3：626 行

| 组 | 载体 | 旧事实 | 原文（前 150 字） |
|---|---|---|---|
| P30001 | .claude/agent-common.md:47 | 代码名 stream_0 在 crates、scripts、.claude/gate.d、.claude/hooks、.claude/scr | - 名字、注释、文档、输出文字都不用序数或位置代称，也不用缩写与「a / b / c」这类略列：不写 `layer0`、`stream_0`、`first_stream`、「层 0」「第一条流」「第一项」，写它是什么、做什么（`enumerate_crash_states`、「崩溃点重放」「新池新建 |
| P30002 | .claude/agent-common.md:62 | 代码名 KS_THREADS 在 crates、scripts、.claude/gate.d、.claude/hooks、.claude/s | 户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），主 agent 新派的写别的数，并带一行「线程上限已判：<理由>」；不为改线程上限给在跑的子 agent 发消息，在跑的照原来的 N 跑完。登记文件头「`SINGLEFS_THREAD_CAP` 设了照它」只管 `capped.s |
| P30003 | .claude/kb/checks-owed.md:25 | 代码名 REQ_HARDBARRIER 在 crates、scripts、.claude/gate.d、.claude/hooks、.cla | 点重放 harness 里造（截断一条记录下来的写请求）；归第三方管道（`milestone/02-second-txn.md`「不收口的」一串里点名本行的那一条：要至少两条管道，第三方管道不在这个里程碑；`milestone/03-third-txn.md` 作业点五·5） ／ 依据是现役对照组（ |
| P30004 | .claude/kb/checks-owed.md:25 | 代码名 REQ_PREFLUSH 在 crates、scripts、.claude/gate.d、.claude/hooks、.claude | 02-second-txn.md`「不收口的」一串里点名本行的那一条：要至少两条管道，第三方管道不在这个里程碑；`milestone/03-third-txn.md` 作业点五·5） ／ 依据是现役对照组（2026-08-27 现查 Linux 6.17）：允许后端返回 `-EOPNOTSUPP`  |
| P30005 | .claude/kb/checks-owed.md:25 | 代码名 drop_writes 在 crates、scripts、.claude/gate.d、.claude/hooks、.claude/ | harness、至少两条管道、失败模型这根轴先定下来（[decisions.md](decisions.md) D17（实现分层与第三方管道） 已定项 1）。⚠️ **撕裂注入不能靠 dm-flakey**（2026-08-28 现查 `Documentation/admin-guide/devic |

### P4：59 行

| 组 | 载体 | 旧事实 | 原文（前 150 字） |
|---|---|---|---|
| P40001 | .claude/kb/checks-owed.md:22 | C14 已在 checks-owed.md「已还清」表，这一句把它写成开着（命中字样：欠） | 布局清单文件随 C14（格式变了 checker 没跟） 有了：`.claude/gate.d/layouts.tsv` 一套布局一行，登记位号、格式定义路径与 checker 判定路径，门禁 checker-independence-and-sync 的 layout-checker-sync 格 |
| P40002 | .claude/kb/checks-owed.md:262 | C14、C20 已在 checks-owed.md「已还清」表，这一句把它写成开着（命中字样：前置） | 布局清单与目录约定，与 C8（门禁范围判不出来）、C635（上一次全量没有机器可读的标记）（C20（罕见布局无人跑） 已并进它） 同一份前置（布局清单随 C14（格式变了 checker 没跟） 有了：`.claude/gate.d/layouts.tsv`，一套布局一行、位号引 D15（格式冻结政 |
| P40003 | .claude/kb/checks-owed.md:270 | C634 已在 checks-owed.md「已还清」表，这一句把它写成开着（命中字样：还没） | 真正缺的是 D16（发布语义） 已定项 1 那一套「可分配」与 `df` 没有设备维，以及写入时要预付的「解开自己」没有量（D3（空间分配） 已定项 2 那一节自陈「量还没算」，C634（单次 unlink 的墓碑字节没有 trace 检查） 同域）——第二轮攻方腿的模型里只有「两边都逐设备取小、且 |
| P40004 | .claude/kb/checks-owed.md:275 | C374 已在 checks-owed.md「已还清」表，这一句把它写成开着（命中字样：等） | 2026-09-18 C374（释放代与树表诞生 txg 只有验收断言盯着） 的 checker 实现：主 agent 原措辞「释放代等于最早不再引用它的有效根的 txg」被残留记录那条流的 12 个合法状态证伪，改判区间 |
| P40005 | .claude/kb/checks-owed.md:298 | C12 已在 checks-owed.md「已还清」表，这一句把它写成开着（命中字样：仍） | 取得到 `crates/singlefs-checker` 的调用图——这一条要的是**同一个 crate 内部**两个方向的符号集合，C12（增量语义共用） 的门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |

### P5：12 行

| 组 | 载体 | 旧事实 | 原文（前 150 字） |
|---|---|---|---|
| P50001 | .claude/kb/checks-owed.md:122 | I-7.8 被写成未实现 | ⚠️ **2026-09-06 当日已把不变量写下来：I-7.8（根记录树 ID 水位不低于全池最大树 ID）**，欠的收窄成「它还没实现」（checker 不存在） |
| P50002 | .claude/kb/checks-owed.md:235 | I-2.2 被写成已实现 | ① 读槽时算法标识读到 0 或未登记的码一律拒绝挂载，checker 同判，I-2.2（校验和算法全盘一致） 转已实现 |
| P50003 | .claude/kb/checks-owed.md:343 | D22 已定项 1 被写成未实现 | ② 上界仍是写死的 16（lib.rs 里标着 placeholder），D22（单元原子性怎么合成） 已定项 1 要的「由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算」没实现，那个数算不出来（I-7.4（近 K 代块未被复用） 的 K 代指回退候选集本身，候选集按 F 与实例表现算） |
| P50004 | .claude/kb/checks-owed.md:343 | I-7.4 被写成未实现 | ② 上界仍是写死的 16（lib.rs 里标着 placeholder），D22（单元原子性怎么合成） 已定项 1 要的「由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算」没实现，那个数算不出来（I-7.4（近 K 代块未被复用） 的 K 代指回退候选集本身，候选集按 F 与实例表现算） |
| P50005 | .claude/kb/decisions.md:392 | I-7.6 被写成未实现 | - 14. devs < R 时 I-7.6（根环区域落在互不相同的盘上） 判什么 —— 已定 · 未实现 |

### P7：6 行

| 组 | 载体 | 旧事实 | 原文（前 150 字） |
|---|---|---|---|
| P70001 | .claude/kb/checks-owed.md:387 | 门禁格名 status-sync 不在 # gate-cell: 登记里 | `--write`（decision-items-sync）读 `decisions.md`（`.claude/gate.d/doc-decisions.sh` 第 1640、1653 行）到整份写回（第 1644、1679 行）之间不再核一次文件，也没有锁，另一个会话同一时刻改了 `decisio |
| P70002 | .claude/kb/checks-owed.md:405 | 门禁格名 file-names 不在 # gate-cell: 登记里 | 0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs` 数；同一条命令现数 95 行、59 份文件），其中同一行没有 kb 指路（D、E、C、I 编号带括号简称、`.claude/kb/` 路径、已定项、未定项，再接一道 `grep -vE` 去掉这几样）的  |
| P70003 | .claude/kb/checks-owed.md:425 | 门禁阶段名 harness-test-granularity 不在 .claude/gate.d/*.sh 里 | ting_the_system_configuration`；正推探针 `one-scenario-on-tier.py`）。`crates/*/src/` 里的 `#[cfg(test)]` 单测两档都不在射程 ／ one-scenario 格把 `crates/singlefs-checker- |
| P70004 | .claude/kb/checks-owed.md:774 | 门禁格名 missing-commands 不在 # gate-cell: 登记里 | ／ C642 ／ 单机配置跑不了崩溃放量全量 ／ 单机配置只写清场命令就能跑崩溃放量全量：清场包装 `scripts/crash/gpu-quiesce-run.sh` 直接读 `QUIESCE_STOP_COMMAND`、`QUIESCE_STOPPED_CHECK_COMMAND`，不读双机开关 |
| P70005 | .claude/kb/checks-owed.md:796 | 门禁格名 outer-memory-cap-to-driver 不在 # gate-cell: 登记里 | ／ C796 ／ 崩溃点重放全量外层上限小于双机驱动里层上限 ／ `scripts/crash/crash-point-replay-shard-run.sh` 的 `run_merged_log` 读外层包装暴露的 `RUN_WITH_MEMORY_CAP_OUTER_CAP`、写法认得就交给驱动 |

### P9：24 行

| 组 | 载体 | 旧事实 | 原文（前 150 字） |
|---|---|---|---|
| P90001 | .claude/agents/sweep.md:41 | 带日期的现状句（形态命中，kb-discipline 第 8 条说这类是历史快照） | 「仍欠：X」「要等 X」）在结束那一版之后还是不是真话——判的是分句，不是这笔欠账或这条决策还清了没有：欠账照旧开着、而它写的前置或理由已经过时，照样判要改。分句带日期的，把日期删掉再读一遍：读起来是在说今天的样子（有没有、几条、多宽、还欠什么），就当现状句判，日期是过去的不等于这句说的是过去；读起 |
| P90002 | .claude/kb/checks-owed.md:106 | 带日期的现状句（形态命中，kb-discipline 第 8 条说这类是历史快照） | xperiments/replay.sh` 把 E58（校验和粒度的端到端代价） 判成「对不上」而不是「仅计时不同」，2026-09-05 全量复跑逐次如此。**害处不是那一行本身，是它把复跑的红灯变成常态**：一个每次都红、每次都被解释成「哦那个不算」的检查，等于没有检查（`.claude/sin |
| P90003 | .claude/kb/milestone/01-first-txn.md:19 | 带日期的现状句（形态命中，kb-discipline 第 8 条说这类是历史快照） | **现状（2026-09-14 开工）**：根 `Cargo.toml` workspace（排除 `research/`）+ 四个 crate：`crates/singlefs-format`（格式常量，带 `format-const:` 注与 `// placeholder:` 占位）、`cra |
| P90004 | .claude/kb/milestone/01-first-txn.md:43 | 带日期的现状句（形态命中，kb-discipline 第 8 条说这类是历史快照） | **现状（2026-09-14 开工，验收五条全过）**：`crates/singlefs-core` 的 `make_filesystem` 吃两块设备，按字节表零的 m1–m4 写出实例表单元第 0 片（槽 50176）、树表单元第 0 版（槽 50178）、三个区域各自槽 0 的第 0 代根记 |
| P90005 | .claude/kb/milestone/01-first-txn.md:76 | 带日期的现状句（形态命中，kb-discipline 第 8 条说这类是历史快照） | **现状（2026-09-14 开工，验收三条全过）**：`crates/singlefs-core` 的 `allocator` 模块——每盘一张单元区空闲图（已分配位、每段已用槽数、空闲 run 数三样增量维护，运行时不扫盘）、用户数据落点政策（起点最小、偶数槽、两槽都空、不在开放段）、提交内生 |


## 精度看法（样例 5 行里 3 行以上不是腐烂的，写明怎么收窄；口径我没改）

下面是我凭读命中的那一句判的，没逐个去核仓，只用来决定怎么收窄。

| 探针 | 前 5 行里像腐烂的 | 看法 |
|---|---|---|
| P1 | 0 / 5 | 精度低。前 5 行是「两条路已删」「说 format-spec 目录不存在」「引已归档的 research/prompts 目录」，都是句子本来就在说它没了。84 行里 21 行是 `scripts/experiments/target/` 下的转储目录（实验页引的运行产物，不进库），19 行是 research/ 下已归档的路径，19 行是不带仓内前缀的短写。真有价值的是「新事实」栏写着同名文件在活目录的几行，例：`scripts/gate.sh`、`scripts/shell-lint.sh` 的同名文件在 `.claude/scripts/`（搬了家）。 |
| P3 | 0 / 5 | 精度低。626 行（554 个不同的名字）里，`.claude/kb/prior-art.md` 占 180 行（别家实现的函数名与内核标志，如 `REQ_PREFLUSH`、`dsl_deadlist_merge`），`.claude/kb/term-renames.md` 占 177 行（旧名登记表，本来就要留旧名）；`agent-common.md` 的 `stream_0`、`layer0` 是「不许写」的反例。 |
| P4 | 0 或 1 / 5 | 精度低。用「欠」「前置」「等」「仍」同句判，一句里常同时有已还的编号和还开着的别的编号（例「C14 有了……C8 还欠」），字样说的是另一个编号；「等」还会命中「等于」。 |
| P5 | 3 / 5 | 精度中。12 行里我看像真矛盾的约 4 行（`checks-owed.md:122` 的 I-7.8、`:235` 的 I-2.2、`:343` 的 D22 已定项 1 与 I-7.4）；`decisions.md:392` 那行的「未实现」说的是该分项自己的进度而不是 I-7.6；`03-third-txn.md` 三行的「未实现」在 C 编号的简称里（`C28（环几何不变量未实现）`）。 |
| P7 | 0 / 5 | 精度低但只有 6 行。前 5 行里 `status-sync` 是「随 status-sync 格删掉」的事件句，`file-names`、`missing-commands`、`outer-memory-cap-to-driver` 是别的脚本 `--selftest` 的格不是门禁格，`harness-test-granularity` 是用例上的标记注释名。 |
| P9 | 不判 | 24 行都是形态命中：`milestone/02-second-txn.md` 12 行、`milestone/01-first-txn.md` 8 行、`verification-build.md` 2 行、`checks-owed.md` 1 行、`.claude/agents/sweep.md` 1 行（那一行是举例的写法）。 |

### 怎么收窄（要不要采用由主 agent 定）

- P1：先判「新事实」栏不是「无」且同名文件不在 `research/` 下的几行；`scripts/experiments/target/` 下的、`research/` 下的、句子里带「已删」「不存在」「没有」的，可以另成一批或不判。
- P3：不扫 `prior-art.md` 与 `term-renames.md`（253 行）；再排除在 `research/` 的非 md 文件里能命中的名字（共 71 个名字，剩 155 行，名字清单与组号在 `p3-narrowed-preview.txt`）。剩下的仍有一批是 `ds_num_children` 这类别家实现的名字，可以再按「所在文档是 decisions/ 的依据句」另判。
- P4：同句判改成「字样在该编号后面、到下一个编号之前」那一段里（P5 已是这样做的），并去掉单字「等」；或只留「欠」「前置」「要先」「挡」四个字样。
- P5：第一个编号后面的括号简称里的字样不算；「—— 已定 · 未实现」这种分项自己的状态尾巴不算。
- P7：「X 格」只在同句有「门禁」或阶段名时才报；句子里有 `--selftest` 的不报。

### 没拦住的已知漏

- 围栏代码块里的路径与名字不扫（脚本写在开头，改成扫要去掉 `load_documents` 里的 `in_fence` 一段）。
- P5 的决策分项只有「### 已定项」表里带 `**进度：…。**` 的行；「未定项」里的分项、没有进度标记的行不入集合。
- P4 只认 `C\d+` 且同句；跨句、写成「欠账表第 N 行」的不认。

## 没做什么

- 没逐个核候选，也没派回扫员；样例判断只凭句子本身。
- 没跑重型测试、门禁、编译；没改仓内文件；没提交；没碰 `scripts/kb/stale-candidates.py`（它认组名的事只用 `--check-report` 空表试了一次）。
- `--selftest` 只跑了草稿目录里这一份，没有给 `selftest-registry.tsv` 登记（脚本不在仓里）。
- 开跑前后我对仓根跑 `git status --short | md5sum`，两次的值不同（534b5a48… 与 1b6f44da…）；我没写过仓内文件，脚本里没有写仓的调用，应是本机别的会话在改。

## 推翻条件

- 若 `--selftest` 在干净的另一份 Python 3 下红，或对仓根跑出的行数与上面不同（仓有改动除外），说明脚本或本报告有误。
- 若某个探针的「前 5 行」换一批后多数是真腐烂，上面的精度判断就不成立；样例只是前 5 行。
