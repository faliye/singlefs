# 删除批第 18 批合入的代码：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 N9 N10 K1 K2 K3 F1 F3 F4 F5 -->

## 一、题面

删除批第 18 批五份补丁 2026-10-05 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）：里程碑三 三·3 的池级 checker（P1）与 GPU 判器（P2）、P1 连带红的一条用例与一处 lint（P1 小修）、三·6 干跑与 E142 第二个几何要的导出 bin（X）、E161 第 2 次重跑的装置（Y）。这一轮判两件事：代码做的是不是条款、判决与用户定案说的；哪一格会错。

三·3 的读法以设计轮判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节为准，用户 2026-10-05 的两项定案在岔路单 `research/prompts/m3-delete-checker-r1-forks.md`（I-4.4 第一版报不适用、I-4.6 只判单元数对 size）；五条的读法写回在 `.claude/kb/invariants.md` 那五行（写回 8、写回 9）。

改动全文在 `research/prompts/_m3-batch18-code-r1-diff.md`（五份补丁与变异追加、替换行原样）。冻结副本 `/tmp/claude-1000/m3-batch18-code-r1/tree/`（合入之后主工作区的 `crates/` 与 `Cargo.*`，另含根环线 10c、10d 与形式化线已合的改动），被判的 19 份文件的 sha256 在 `research/prompts/m3-batch18-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索，以代码为准）：`/tmp/claude-1000/m3-batch18/{x,y,p1,p1fix,p2}/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | I-9.5 两个方向（队列里的号在同一版 inode 树里有记录且 nlink = 0；nlink = 0 的记录的号在队列里） | `crates/singlefs-checker/src/walk.rs` `judge_the_deletion_queue_against_the_inode_records`（第 8409 行） | `invariants.md` I-9.5 那一行；D8（核心索引结构） 已定项 16 第 3 条；设计轮判决第二节最后一行；C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） |
| N2 | I-4.7 对象取丙；I-4.6 乙（nlink = 0 的记录，size 0 时 0 或 1 个单元、size > 0 时 ⌈size ÷ 净荷容量⌉）；I-3.4 只落「记账标签 4 恒 0」、缺行判红 | `walk.rs` `judge_deletion_state_of_the_versions`（第 8178 行）起那一组只读函数 | `invariants.md` I-4.7、I-4.6、I-3.4 三行；设计轮判决第二节 F1、F3、F5 三格；用户 I-4.6 乙 |
| N3 | 判哪几版、怎么配、读不全怎么报：回退候选集每一版加由记录施加出来的几版；经树表条目偏移 116 的头 ID 配对；读不全报 `part_of_the_range_not_judged`；根指针全零当没有对象 | 同 N2；`extent_tree_with_the_inode_of_each_data_pointer_without_judging`（第 4114 行，`extent_tree_without_judging` 改成包它） | 设计轮判决第二节 F4 三格；`invariants.md` 文件头「判定有四种」那一段 |
| N4 | I-4.4 第一版恒报不适用并带理由；`IMPLEMENTED_INVARIANTS` 69 → 74，下标 69–73 依次 I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，只往末尾加 | `crates/singlefs-checker/src/image.rs` 第 88、234–237 行 | 用户 I-4.4 乙；`invariants.md` I-4.4 那一行 |
| N5 | I-9.6 墓碑那一半：水位 > 所选根墓碑树里已发布墓碑记录的对象 ID 最大值 | `walk.rs` `largest_published_tombstone_object_identifier`（第 8533 行） | `invariants.md` I-9.6 那一行 |
| N6 | 坏镜像与用例：每条一份只红自己的坏镜像；中间版写坏、最新根写对的用例；读不全报不适用的用例；干净镜像与不适用表跟着改 | `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`、`common_deletion_in_progress/mod.rs`、`checker_known_bad_images.rs`、`writable_mount_of_a_formatted_pool.rs` | 里程碑三 三·3 那一行的验收「四条每条只红自己、`crates/mutations.tsv` 各一行」 |
| N7 | P1 连带红：坏镜像里 2 号有数据单元没有记录，断言改成 I-5.1 与 I-4.7 都红、测试改名、变异表第 842 行换测试名；`shadow_unrelated` 改名 | `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`、`an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` | 主 agent 判（记录 2026-10-05 P1 交回那一条第 ① 项） |
| N8 | GPU 判器：五条下标常量与覆盖表、扫描内核照 P1 的 CPU 判法、I-9.6 墓碑那一半；草稿区 290 544 → 298 736；对拍加 8 条，经过 nlink = 0 的 6 条只比违例位 | `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`（第 152–160 行）、`crash_judge_checker_scan.wgsl`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs`、`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 用户 2026-10-03 定 GPU 判器全覆盖（`records/2026-09-30-里程碑三开工第一轮.md` 第 498 行一带）；C712（GPU 恢复内核不判 nlink = 0） |
| N9 | E142 导出 bin 加 `--device-bytes`（不带参数与只收零个参数时逐字节相同）；新 bin `deletion_write_dump.rs`；`scenario.rs` 公开 `run_deletion_history`（H1、H2）；harness 用例 `deletion_scenario_writes.rs` | `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs`、`…/bin/deletion_write_dump.rs`、`crates/singlefs-harness/src/scenario.rs` 第 548 行、`crates/singlefs-harness/tests/deletion_scenario_writes.rs` | E142 第 22 次重跑登记 `research/prompts/e142-r22-prereg.md` 5.4；E172 跑前登记 `research/prompts/e172-preregistration.md` 5.3 |
| N10 | E161 装置照第 2 次重跑登记 5.6 I-1 到 I-12 改（段形、取样段 11–26、写数阈值 24、域守卫、内联测试 14 条）；变异 M13–M16 | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` | `research/prompts/e161-r2-prereg.md` 5.6、读法甲 |

**这一轮不判的**（各有去处，腿不攻）：
- 同一棵冻结树里根环线 10c、10d 与形式化线 26-3 的改动：各由那两条线的代码轮判。
- `invariants.md` 里写「三·3 实现」的另外九格（I-1.7、I-9.10、I-9.17、I-9.19–I-9.24）：用户 2026-10-05 定并进三·3、下一批做，这一轮不判它们缺。
- C708 还差的「队列里的号在树里 nlink ≠ 0」那一形坏镜像：已进第 19 批清单。
- E142 第 22 次第二段、E161 第 2 次重跑第三段的结论：由各自的实验页与岔路表判。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本取自合入之后的主工作区。合入验证：`merge-and-verify.sh --skip-apply`「✓ 测试全绿（159 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-05-413718/`），fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装 `auto` 单跑都退 0；checker 档没跑（GPU 对拍、P2 的 11 行变异都归提交时）。
- 实现员 P1 自报（线索）：12 条新测试证红，8 个变异全抓；harness 整包 1100 过、1 红（N7 那一条）；没有删除的版本上 I-9.5 报成立、I-4.7 与 I-4.6 报不适用；根指针全零的三种当没有对象；同号两条取后读到的那条。
- 实现员 P2 自报：输入表不加，内核从单元原始字节现读；带 `gpu,verdict-store` 与不带特性两种编得过；GPU 恢复内核找第一个文件只认号、不看 nlink（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 找第一个文件那一段），与 `crates/singlefs-core/src/recovery.rs` 第 5076–5079 行 CPU 答「没有文件」不一致，主 agent 现查属实，记 C712。
- E142 第 22 次第二段执行员用 N9 的 `--device-bytes 8589934592` 跑出 P8G 导出，与装置逐区域「matched=29 equal=29 unequal=0」；E161 第三段执行员核 N10 装置的域与锚点，行 1、2 相等。两份都是实验侧的读数，不替这一轮判代码。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N10）
逐条从条款原文、设计轮判决格与用户定案推到冻结副本里的行，再推到钉它的测试与变异行。要看的有这几处：
- N1–N3：判法的每一个分支与 `invariants.md` 那几行、设计轮判决 F1–F5 各格逐句对；「判哪几版」的版本清单与 I-5.5 那一判是不是同一份；读不全、根指针全零、没有删除这三种情形各报什么，与 `invariants.md` 文件头那四种判定对得上没有。
- N4：下标次序、I-4.4 理由句与 `invariants.md` 那一行逐字对。
- N5：「已发布」用的是哪个谓词，与 I-9.6 那一行「被抛弃时间线的墓碑不算」对得上没有。
- N6：每份坏镜像是不是真的只红目标那一条（断言写的是不是「恰好」）；变异改坏的是不是名字说的那一处。
- N8：GPU 内核的每一支与 P1 的 CPU 判法逐支对；六条只比违例位的用例覆盖了哪几格、漏了哪几格。
- N9：`--device-bytes` 的解析与缺省；`run_deletion_history` 的 H1、H2 与 E172 登记 5.3 写的历史逐步对。
- N10：装置改动与 E161 登记 5.6 I-1 到 I-12 逐条对。

### K2　哪一格会错（重点 N1–N3、N8）
在冻结副本的拷贝上攻，只跑 harness 档：
- 合法状态上误红：删除中间态的每一个已发布状态（意图、每一批、续做批、挂载续做、批中途崩、回退之后）上，五条与 I-9.6 有没有报红；造历史时把用户决定的那几步放开扫（文件单元数、批大小、unlink 时机、崩在哪一批、续做在会话里还是挂载里、调用方给的时间）。
- 漏判：每条各造一个该红而判成立或不适用的镜像，尤其是只在中间版写坏、读不全的那一版夹在中间、两棵树配错头 ID、墓碑树里有被抛弃时间线的墓碑。
- 「只红自己」：每份坏镜像换一种造法再造一次，看是不是仍只红目标那一条。
- N8：P2 的判法有没有一支与 CPU 不同（读代码对，造得出就在 harness 档里用 CPU 判法算出期望、写清 GPU 那一侧会给什么；GPU 不跑）；只比违例位的那 6 条有没有漏掉恢复结局之外的分叉。
- N9：`--device-bytes` 给非 4 GiB 时导出的每一个区域是不是都随盘大小算（不只 P8G 一点）。

### K3　被判的文件名单
`research/prompts/m3-batch18-code-r1-snapshot/crates-sha256.txt` 里那 19 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格与用户定案推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1–N3 与 N8 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- 误红判据：一个从入口走得到的已发布状态上，五条或 I-9.6 有一条报不成立，就是误红；读不全报部分没判不算误红。
- 漏判判据：镜像违反了 `invariants.md` 那一行的字面，checker 报成立或不适用。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch18-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch18-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的文件:行号。
- 方案按 `crates/` 今天的实现来谈。
