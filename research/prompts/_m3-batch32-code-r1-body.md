# 删除批第 32 批合入的代码：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 P1 P2 P3 P4 P5 P6 P7 F-KEY F-ID-b F-L0 F-W F-IT G1 G2 G3 G4 S1 S2 S4 M3 O3 A3 -->

## 一、题面

删除批第 32 批 P 一份补丁 2026-10-07 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-07 那几条）：关 m3-batch31-code-r1 判决第三节第 2–4 条、m3-batch30-code-r1 判决第三节第 1–5 条、m3-batch29-code-r1 判决第三节第 2 条（F-零槽）、扫描内核那句说明、根环线托的钉值与对拍文件三处。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch32-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch32-code-r1/tree/`，被判的 9 份文件的 sha256 在 `research/prompts/m3-batch32-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch32/p/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| P1 | F-KEY：容器、inode 孩子、extent 数据单元三处「走过」按读回那一份的落点认（共用 `read_a_unit_to_walk_and_the_placement_it_was_read_from`）；父单元、诞生代号那张表、合并照旧按第 0 条；GPU 同改（`w_visit_the_copy_read_back`） | `crates/singlefs-checker/src/walk.rs` 第 1248 行；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 1318 行 | `.claude/kb/invariants.md` I-1.7；m3-batch31-code-r1 判决第二节 K2 观察 O3 那一行、第三节第 2 条 |
| P2 | F-ID-b：墓碑、谱系类型段 0 那一支认「孩子是码 2」只认按自己那一类长度整份自证过的副本；两份都没自证过时照改之前的读法 | `walk.rs` 第 4511 行与调用处；走读内核同改 | I-9.17、I-9.23；m3-batch31-code-r1 判决第二节 A3 那一行、第四节（用户 2026-10-07 定「只认自证过的副本」） |
| P3 | 测试：层级 0 的根下 F-L0 一形、G1（三棵树类型段 0 的 F-W）、G2（头里类标签是同长度别类）、G3（层级 2）、G4（F-W 五形加 I-2.1 一族断言）；变异 S1、S2、S4、M3、F-IT 两行、`UNDECIDABLE_TOO_DEEP` 一行 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` 第 13278 行起；对拍文件第 10278 行起；`crates/mutations.tsv` 第 3078–3089 行 | m3-batch30-code-r1 判决第三节第 1–5 条；m3-batch31-code-r1 判决第三节第 4 条 |
| P4 | F-零槽：GPU `read_at` 起点在单元区里、不是落点起点、整段没写过时交回全零单元；表里加 32 KiB 与 16 KiB 两条全零单元，`JUDGE_TABLES_VERSION` 3 → 4 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 第 450、502 行；`crates/singlefs-checker-tier/src/crash_judge_tables.rs` 第 41、1279 行一带 | I-2.1；m3-batch29-code-r1 判决第二节 K2 L1「位置条目指向没写过的槽」那一行、第三节第 2 条 |
| P5 | 扫描内核第 12 行一带说明改成与 m3-batch29-code-r1 判决 L2 格一致 | `crash_judge_checker_scan.wgsl` | m3-batch29-code-r1 判决第二节 K1 L2 那一行 |
| P6 | 钉值：恢复 319127、扫描 380969、走读 329198 | `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 第 1347–1357 行 | 根环线托 |
| P7 | 对拍文件：删设了额度变量就判红那一支与辅助函数、改文件头那半句、常量改引 `gpu_test_start::CARD_QUOTA_ENVIRONMENT_VARIABLE` | 对拍文件 | 根环线托 |

**这一轮不判的**（各有去处，腿不攻）：
- `read_index_node` 的「走过」键没跟着改（要连两个调用方）：第 33 批。
- F-ID-b 两份都没自证过那一形：用户 2026-10-07 定「当读不出、这一小句不判」，第 33 批改。
- 三·4 删除流的 oracle 缺口、续做全量每状态核、C692：第 33 批。
- GPU 对拍、GPU 变异、钉值那条库单测：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 合入验证 `merge-and-verify.sh --memory 8G --threads 7`「✓ 测试全绿（170 个测试二进制）」「✓ fmt 过」「✓ clippy 过」，`checker_known_bad_images` 一份「188 passed; 0 failed」（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-07-246606/`）；`crates-mutation-rows.py static-check .` 退 0。合后 4 份 crates 源码与实现员改后那一版逐份相同。
- 实现员报告停下交主 agent 的第 1 条：F-KEY 之后谱系 loc0 那一形多红 I-4.9——诞生代号那张表（`birth_txg_of_walked_units`，`walk.rs` 第 1021 行插入）仍按第 0 条位置条目记，新容器的诞生代号盖掉先走过的 inode 树根那一条；回退目标那条根再走到 inode 树根时按这个数判 I-4.9 红。GPU 同形。实现员报告推的另一处同源分叉：第 0 条那一份对不上、没有别的单元占着它的落点时，CPU 把诞生代号记在第 0 条落点上，GPU 走过表里没有那个落点、记不下。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（P1–P7）
- P1：CPU 与 GPU 三处「走过」的键是不是同一个落点；没被改的那几把键（父单元、诞生代号、合并、inode 叶序）各自该不该跟着改（逐处说理由）。
- P2：CPU 与 GPU 认「自证过」用的是不是同一个判据（类标签登记、按那一类的长度读回的整单元校验和对得上），与 F-W 认别类那一处是不是同一个判据。
- P3：每份坏镜像与对拍流的期望写全没有；S2、S4、G4 各自钉不钉得住它要钉的那一读；变异行点名的测试与参数筛不筛得到。
- P4：GPU 交回全零单元的条件与 CPU 稀疏读者读出全零的条件是不是一一对应；表的两条全零单元的检查字算得对不对；实现员列的「改了之后还留着的分叉」两形对不对。
- P5–P7：照字面核。

### K2　哪一格会错（重点 P1、P4）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- P1：I-4.9 那一形——诞生代号与合并该按哪个落点认（「一个单元的身份是哪个落点」），给出改法候选与各自在攻方 loc0 / loc1 那几形上的结局；F-KEY 会不会让合法状态多红或漏判（盘 0 那份没落、盘 1 落了的合法状态）。
- P4：GPU 读出全零之后，三个内核里各个调用点的结论与 CPU 逐一对；实现员推「合法流与现有对拍流结论不变」，造一形推翻它就算打中。
- P2：两份副本的组合（自证过 / 只 16 KiB 对得上 / 校验和坏 / 读不出）两两扫，两种盘序。
- CPU 与 GPU 判得不一样的一形。

### K3　被判的文件名单
`research/prompts/m3-batch32-code-r1-snapshot/crates-sha256.txt` 里那 9 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 P1、P4 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch32-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch32-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`；挂钟预算若有就从环境变量取；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
