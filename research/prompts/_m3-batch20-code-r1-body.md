# 删除批第 20 批合入的代码：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 N9 K1 K2 K3 L1 L1a L1b L1c L2 H1 H2 H3 -->

## 一、题面

删除批第 20 批四份补丁 2026-10-05 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）：A 关 m3-batch18-code-r1 判决第三节第 1–3、6 条与 C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差的那一形；G 关 C712（GPU 恢复内核不判 nlink = 0） 与判决第三节第 4 条（对拍覆盖）；X、X2 关判决第三节第 5 条、两个 bin 各抄的函数挪进 harness、E172 第三段要的 H3。这一轮判两件事：代码做的是不是条款、判决与用户定案说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch20-code-r1-diff.md`（四份补丁与变异追加、删除行原样）。冻结副本 `/tmp/claude-1000/m3-batch20-code-r1/tree/`（合入之后主工作区的 `crates/` 与 `Cargo.*`，另含形式化线第 30 批已合的改动：`walk_container_tree_root` 多一行 extent 偏移判定、T_time 盘上改毫秒等），被判的 12 份文件的 sha256 在 `research/prompts/m3-batch20-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索，以代码为准）：`/tmp/claude-1000/m3-batch20/{a,g,x,x2}/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | L1 改法 A：`judge_deletion_queue_of_a_version` 反方向也配，每棵 inode 树要恰好一条 `deleted_inodes` 条目指着它，配不上对 I-9.5、I-4.7、I-4.6 记判不了（新成员 `InodeTreeNamedByNoSingleDeletedInodesEntry`） | `crates/singlefs-checker/src/walk.rs` 第 7942、7963、8347 行一带 | 设计轮判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F4-b；m3-batch18-code-r1 判决第二节 L1 那一行 |
| N2 | L2：I-4.6 不适用的理由串换成配不上那一句 | 同上 | m3-batch18-code-r1 判决第二节 L2 那一行 |
| N3 | 未钉格：I-4.6 size = 0 那一支、中间版写坏补 I-9.5 与 I-4.7、`DeletionScanGap` 七个零用例的变体、墓碑「未发布」一支；C708 还差的「队列里的号在树里 nlink ≠ 0」那一形 | `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`、`common_deletion_in_progress/mod.rs` | `.claude/kb/invariants.md` I-9.5、I-4.7、I-4.6、I-9.6 四行；`.claude/kb/checks-owed.md` C708 那一行 |
| N4 | `image.rs` 过期注释 | `crates/singlefs-checker/src/image.rs` | — |
| N5 | C712：GPU 恢复内核找第一个文件那一段补读 nlink、为 0 答没有文件（常量 `INODE_RECORD_NLINK_OFFSET`、`NLINK_OF_AN_INODE_BEING_DELETED`） | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 第 134–136 行与找第一个文件那一段 | `crates/singlefs-core/src/recovery.rs` 第 5076–5079 行；D8（核心索引结构） 已定项 16 第 4 条 |
| N6 | 对拍覆盖：新用例按「只差一个写」逐写切段；经过 nlink = 0 的 6 条换回整份比 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 5253 行一带 | m3-batch18-code-r1 判决第二节「K2 N8 对拍覆盖」那一行；用户 2026-10-03 定 GPU 判器全覆盖 |
| N7 | E142 导出 bin：mkfs 拒的 `--device-bytes` 值打原因退 1 | `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | m3-batch18-code-r1 判决第二节 K2 N9 那一行 |
| N8 | 两个 bin 各抄的 `kind_name` / `operation_bytes` 挪成 `RecordedOperationKind::name` 与 `RetainedOperation::written_bytes_rebuilding_zero_fills`；变异表删两行失效的、追加接住的两行 | `crates/singlefs-harness/src/lib.rs` | `.claude/singlefs-ai-sop/rules/code-discipline.md`「重复要生成，不许手抄」 |
| N9 | H3（先建 size 0 的 2 号 inode、意图时间 = 记录现有时间 + 60 秒）；删除导出不带参数照旧只导 H1、H2，`--history <名>` 只导点名那一段 | `crates/singlefs-harness/src/scenario.rs`、`crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs`、`crates/singlefs-harness/tests/deletion_scenario_writes.rs` | E172 跑前登记 `research/prompts/e172-preregistration.md` 第五节 5.3 与第三段 |

**这一轮不判的**（各有去处，腿不攻）：
- 形式化线第 30 批与根环线的改动：由那两条线的代码轮判。
- 形式化线第 31 批 31-1（`build_pool` 装根环表、分配器没装根环表而记根就断言）还没合进来：合入后可能改删除线文件的搭建，那时另记。
- G 交回时提的几件已进下一批清单：6 条对拍用例改名连带 8 行变异、扫描内核与共用内核两个同值 nlink 常量合一、`e142_new_pool_file_creation_write_dump_one_device.rs` 里还抄着的一份种类名。
- E172 第 2 次跑（H2 换判据重判、H3、T_time 改毫秒后重导）：由实验那一侧判。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本取自合入之后、形式化线第 30 批合入之后的主工作区。第 20 批合入验证：`merge-and-verify.sh`「✓ 测试全绿（163 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-05-2255273/`）；fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0；四份打完之后变异表原文各恰好命中一次（主 agent 现数）。checker 档没跑：GPU 对拍、G 的 4 行与 X 的 2 行变异、新用例的耗时都归提交时。
- 实现员 A 的三行变异由主 agent 补证：「✓ 点名 3 条：跑了 3 条，跳过 0 条，跑的都抓到了」。
- 实现员 X2：不带参数的删除导出与改之前逐字节相同（sha256 5a9b3fa2…）；`--history h3` 与 X 的 H3 那 149 行逐字节相同。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N9）
逐条从条款、判决格与用户定案推到冻结副本里的行，再推到钉它的测试与变异行：
- N1、N2：配对两个方向与设计轮判决 F4-b 逐句对；配不上时三条各报什么，与 `invariants.md` 文件头「判定有四种」对得上没有。
- N3：每份新坏镜像是不是只红目标（断言写的是不是恰好）；`DeletionScanGap` 每个变体的用例是不是真走到那一支。
- N5、N6：GPU 恢复那一判与 CPU `recovery.rs` 那几行逐支对；新对拍用例取的状态是不是登记的「只差一个写、之前段全落」。
- N7–N9：退出码、挪出去的函数与原来逐字节等价、H3 与 E172 登记逐步对。

### K2　哪一格会错（重点 N1、N3、N5）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读代码、用 CPU 算期望）：
- N1：配对改了之后，合法删除中间态上有没有误报判不了或误红；有没有一种树表形态让两个方向都配上、而违例照旧漏判；用户决定的几步（文件单元数、批大小、unlink 对象与次序、进程里 / 会话 / 挂载续做、管理员回退）放开扫。
- N3：每份坏镜像换一种造法再造，是不是仍只红目标。
- N5：GPU 恢复补了 nlink 之后，还有没有一形 CPU 答没有文件而 GPU 不答（同号多条记录、多片 inode 叶、nlink 读的偏移与 CPU 同一处）。
- N9：H3 的形状与登记有没有出入；不带参数的导出在今天的树上是不是仍逐字节等于入库那一份（形式化线已核出系统配置两槽因 T_time 改毫秒而不同，那一处不算这一轮的打中）。

### K3　被判的文件名单
`research/prompts/m3-batch20-code-r1-snapshot/crates-sha256.txt` 里那 12 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格与用户定案推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N3、N5 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- 误红：一个从入口走得到的已发布状态上报不成立。漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立，或报不适用而镜像上有它的对象；配不上的版报部分没判不算漏判。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch20-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch20-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
