# 删除批第 25 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers N1 N2 N3 N4 K1 K2 K3 -->

## 一、题面

删除批第 25 批一份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：N 关 m3-batch23-code-r1 判决第三节第 1、2 条——I-9.29 判不了的门改成看主走读有没有沿最新根的实例表链走到末片（CPU 与 GPU 同改），走读内核两处写死 `ro + 8u` 改命名常量，扫描内核删除态头注释补 I-9.24。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch25-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch25-code-r1/tree/`，被判的 7 份文件的 sha256 在 `research/prompts/m3-batch25-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`/tmp/claude-1000/m3-batch25/n/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| N1 | CPU：`Walk` 加 `abandoned_roots_filtering`，最新根那一遍沿实例表链走到末片才记滤全；I-9.29 判不了的门直接读它，不再另调 `instance_table_rows_of_root_without_judging` | `crates/singlefs-checker/src/walk.rs` | `.claude/kb/invariants.md` I-9.29「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；m3-batch23-code-r1 判决第二节 K2 N5 |
| N2 | GPU：走读内核把「走到末片」写进交接段第 30 个字（原来空着，`SCRATCH_WORDS` 不变），扫描内核 I-9.29 读它 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_checker_common.wgsl` | 同上；用户 2026-10-03 定 GPU 全覆盖 |
| N3 | 坏镜像：N5-a 转成立、N5-c 只红 I-2.3、N5-b 仍部分没判；对拍补 N5-a 一条流 | `crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 同上 |
| N4 | 两处 `ro + 8u` 改用 `INODE_RECORD_OBJECT_BIRTH_OFFSET`；扫描内核头注释补 I-9.24；变异表追加、替换 3 行（锚点随改动变了） | 走读内核、扫描内核、`crates/mutations.tsv` | m3-batch23-code-r1 判决第三节第 2 条 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch24-code-r1 判决第三节（读孩子类标签时验校验和、inode 树读不出孩子时 I-9.2）：第 26 批。
- C726（读不出时同一实例的版本之间也不比）、C727（GPU I-9.29 跨版本表 2048 格）：已记欠账。
- GPU 对拍与 GPU 那两行变异：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-06-1756899/`）；fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0；`crash_enumeration_sharded_across_processes` 整份「10 passed」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N4）
- N1、N2：「走到末片」在 CPU 与 GPU 上是不是同一个条件（链指针记录合不合法、载荷 CRC、读不出、跨片上限各怎么算）；门开、门关两个方向各由哪条用例或变异钉住。
- N3、N4：每份坏镜像是不是只红目标；三行替换后点名的测试与参数是不是没变。

### K2　哪一格会错（重点 N1、N2）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- 找一形主走读「走到末片」而候选集其实没滤全（被抛弃的根留在候选集里、I-9.29 误红），或没走到末片而滤全了（I-9.29 无谓地判不了）；用户决定的那几步放开扫（崩溃恢复落到哪条根、实例表长到几片、回退、再挂）。
- CPU 与 GPU 判得不一样的一形。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch24-code-r1 的 N1 单故障、N3。

### K3　被判的文件名单
`research/prompts/m3-batch25-code-r1-snapshot/crates-sha256.txt` 里那 7 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch25-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch25-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比之前排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
