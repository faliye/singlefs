# 删除批第 33 批 R 合入的代码：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 R1 R2 R3 R4 -->

## 一、题面

删除批第 33 批先派的一块 R 2026-10-07 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-07 那几条）：让里程碑三 三·4 的删除流在 CPU 这一侧跑通层 0 的 oracle。这一轮判两件事：代码做的是不是条款与主 agent 定的读法说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch33r-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch33r-code-r1/tree/`（R 合后、根环线 10w 合入之前），被判的 4 份文件的 sha256 在 `research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch33/r/report.md`；前任 Q 的报告 `~/.local/share/singlefs/work/m3-batch32/q/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定的读法 |
|---|---|---|---|
| R1 | oracle 认「这一版起没有文件」：`Layer0VersionTable`（带文件的一组、没有文件的一组）与 `classified_oracle_violation_for_the_version_table`；根在表里看那一版，根不在表里只看比它旧的最新一项；旧函数签名不变、转调 | `crates/singlefs-checker-tier/src/crash.rs`（`newest_older_version_if_it_has_a_file` 第 1046 行起）；`crates/singlefs-harness/src/memory_pool.rs`（`PublishedVersionWithoutAFile`） | D8（核心索引结构） 已定项 16 第 4 条；主 agent 2026-10-07 定的 ①（调度记录「Q 交回」那一条） |
| R2 | 续做走到空全量每状态核：观察者 `OnTheWorkerThreads`、新入口 `enumerate_layer0_against_the_version_table_or_one_shard`；1 / 4 线程与调用线程三种跑法 `Layer0Tally` 逐项相等的对拍 | `crash.rs` | D8 已定项 5；主 agent 定的 ②；`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」 |
| R3 | 新流：多记录流 A、B、C 之后 unlink 那个文件（意图、两批、最后一批），快档 189 态、全量 #[ignore]；版本表 A、B、C 带文件，意图那一版（第 6 代）进「没有文件」组，各批（第 7–9 代）不进表 | `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` | `.claude/kb/milestone/03-third-txn.md` 三·4；用户 2026-10-01 定 K-U6「F1：接在多记录流后面」 |
| R4 | 变异 8 行（第 3102–3109 行）；主 agent 改第 16 行锚点 | `crates/mutations.tsv` | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- 新流的崩溃放量那条（`crash_amplification.rs` 的 `CrashFlow` 加「没有文件的那几版」）与 GPU 输入表、恢复内核那一侧：第 33 批后一块。
- 全量每状态挂载的挂钟（R 推两台 13–65 天，debug、机器忙时量的）：提交时快档 release 下量单次耗时再重算。
- 层 0 全量与 checker 档测试：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 合入验证「✓ 测试全绿（170 个测试二进制）」「✓ fmt 过」「✓ clippy 过」；打补丁那一刻 code-source-discipline 报第 16 行锚点（R 删了旧的「任一更旧版」比较），主 agent 已把锚点改到新判法同一处，`crates-mutation-rows.py static-check .` 退 0（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-07-976401/`）。
- 主 agent 审了新判法的五支（根在表里有文件 / 没有文件、根不在表里、读出内容、内容不对），与定的 ① 一致。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（R1–R4）
- R1：「根不在表里只看比它旧的最新一项」在现有流上与旧的「任一更旧版有文件」等价——钉住等价的那条单测取的表够不够（现有流里有没有「同 txg 两实例」「回退」这类版本表，等价在它们上成立吗）；新流的版本表（各批不进表）与 D8 已定项 16 第 4 条对不对得上。
- R2：工作线程上的挂载核与调用线程上的是不是同一个判法；三种跑法逐项相等那条对拍能不能分辨「工作线程上漏核了一部分状态」。
- R3：快档与全量的期望写全没有；崩在意图那次发布中间、某一批中间、最后一批之后，各读回哪一版、挂完是不是走到空，与 D8 已定项 5、16 对得上。
- R4：8 行变异与第 16 行改后锚点筛不筛得到、各自点名的测试会不会红（读码推，checker 档不跑）。

### K2　哪一格会错
在冻结副本的拷贝上攻，只跑 harness 档（checker 档的用例不跑；要判的用 harness 档自己造镜像、调 `crash.rs` 里公开的判法）：
- R1：一张版本表让新判法与旧判法给出不同结论、而按 D8 已定项 16 第 4 条旧的才对（或反过来）；「没有文件」组里的版本被一次读出内容的恢复判成立的一形。
- R2：观察者挪进工作线程之后判定次序、分片、续跑（断点）会不会让某个状态没核或核两次。
- R3：删除流里 oracle 放过、而按条款该红的状态（例：最后一批之后读出了内容；意图之前就报没有文件）。

### K3　被判的文件名单
`research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt` 里那 4 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、定的读法推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch33r-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与脚本放 `research/prompts/m3-batch33r-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`，用法写在文件头第一行；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
