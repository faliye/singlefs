# 删除批第 33 批 S 合入的代码：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 S1 S2 S3 S4 S5 S6 S7 S8 E1 E2 E4 E6 E7 Z1 Z2 Z3 F-KEY F-ID-b F-W -->

## 一、题面

删除批第 33 批 S 2026-10-07 合进主工作区（`records/2026-10-01-删除与并发事务.md` 同日那几条）：关第 32 批代码轮打中的几处（`research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1–4 条）、F-ID-b 两份都没自证过的读法（用户 2026-10-07 定「当读不出、这一小句不判」）、删除流进崩溃放量与 GPU 一侧（第 33 批 R 那一轮判决 `research/prompts/m3-batch33r-code-r1-main-verification.md` 正文第一节「这一轮不判的」第一条）、C744 ①、三个内核钉值。这一轮判两件事：代码做的是不是条款与判决定的改法；哪一格会错。

改动全文在 `research/prompts/_m3-batch33s-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch33s-code-r1/tree/`：S 合后、并且根环线 10x（去掉跨进程显存账）已合之后的主工作区（10x 改了 `crash_amplification.rs` 的产品路径借卡几处、`gpu_unit_checks.rs`、删了 `gpu_memory_ledger.rs`，不在这一轮被判之列，腿不攻 10x 的改动）。被判的 18 份文件的 sha256 在 `research/prompts/m3-batch33s-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch33/s/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定的改法 |
|---|---|---|---|
| S1 allrbmap | 一个单元在走读里的各把键（走过、诞生代号、并进前一版、父单元、inode 叶序、I-3.5 折叠表）一律按读回那一份的落点认；`read_index_node` 交回 `IndexNodeReading` 三种结局；实例表链同改；GPU 走读内核同改 | `crates/singlefs-checker/src/walk.rs`；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_read_unit_to_walk` 与各处键 | 第 32 批判决第三节第 1 条；I-4.9、I-3.5、I-2.1 |
| S2 P4 | GPU 全零单元守卫改按 `offset.y >> SLOT_SHIFT`，`first_slot + n` 改成先判起点再拿差比；F-α 选 Z3（比 32 KiB 时另按 32 KiB 读） | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 的 `zero_unit_version_over_a_slot_nothing_wrote` 与 `read_at`；`crash_judge_checker.wgsl` F-α 那一处 | 第 32 批判决第三节第 2 条 |
| S3 P2 | 「自证过」三份 CPU 抄写合成 `class_of_a_copy_verifying_at_the_length_of_its_class`；两份都没自证过时当读不出、「孩子是码 2」那一小句不判（CPU、GPU 同改） | `walk.rs`、`crash_judge_checker.wgsl` | 第 32 批判决第三节第 3 条；`.claude/kb/invariants.md` I-9.17、I-9.23 那两行的 F-ID-b 读法 |
| S4 P3 | G4 那一读（E1）、F-KEY 数据单元（E2）、F-KEY inode 内部孩子（E4）、F-ID-b 回落（E6）、I-4.9 那一形各补坏镜像；E7 停下（造不出能区分的形） | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 第 32 批判决第三节第 4 条 |
| S5 删除流进崩溃放量与 GPU | `CrashFlow` 加 `versions_without_a_file` 与 `version_table()`；判一块走 `crash::judge_layer0_state_range_against_the_version_table`；复用键只在有「没有文件那几版」时追加；GPU 期望版本表第 5 字记有没有文件、`JUDGE_TABLES_VERSION` 4 → 5；恢复内核 `classify_oracle` 照 CPU 改；新流补崩溃放量那条；六份测试各加一行 | `crates/singlefs-checker-tier/src/crash_amplification.rs`、`crash_judge_tables.rs`、`crash_judge_recovery.wgsl`、`crash.rs`、`tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` 与六份测试 | D8（核心索引结构） 已定项 16 第 4 条；第 33 批 R 判决 |
| S6 C744 ① | 由记录施加出来的版与下一次挂载先施加的那一版同在的一条对拍流 | `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | `.claude/kb/checks-owed.md` C744 |
| S7 钉值 | 恢复 320015、扫描 382259、走读 337872 | `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | 预算 400000 |
| S8 变异 | 追加 19 行，按名字整行替换 25 行 | `crates/mutations.tsv` | — |

**这一轮不判的**（腿不攻）：根环线 10x 的改动；C744 ② ③（实现员停下，进下一批清单）；E7（同 C739 ① 开着，要先有条款）；checker 档测试与 GPU 对拍上卡（提交时由主 agent 跑）。

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 合入验证「✓ 测试全绿（170 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-07-3760584/`）；合入之前主 agent 在副本里另跑了 `cargo build --offline -p singlefs-checker-tier --all-targets` 与加 `--features verdict-store` 两趟（都退 0、0 行 warning）与 harness `checker_known_bad_images --include-ignored`「203 passed; 0 failed」；全表 `crates-mutation-rows.py static-check .` 退 0。
- 主 agent 审 S2：守卫与差比改法对；守卫上方注释仍写旧界「高 32 位 < 2^(32 − SLOT_SHIFT)」，对的是 `offset.y < 2^SLOT_SHIFT`（`crash_judge_common.wgsl` 里 `zero_unit_version_over_a_slot_nothing_wrote` 上方那三行注释）——已知，腿不必再报。
- 主 agent 审 S3：`walk.rs` 里 `class_of_the_first_copy_verifying_at_the_length_of_its_class` 两份都没自证过交 None，调用方 `is_none_or` 当这一小句成立、不判。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（S1–S8）
- S1：各把键是不是都按同一个落点认、CPU 与 GPU 是不是同一组键；合法崩溃状态上判定集合变没变（实现员推的是不变：键只在第 0 条那一份校验和对不上时才变，那时 I-2.1 已红）；第 25 批 H-b、第 28 批 H-V、第 30 批 F-W 那几条已有坏镜像结局变没变。
- S2：守卫的界、差比在空段两端与槽号 2³² 附近；F-α 选 Z3 之后 CPU 与 GPU 在攻方第 32 批那一形上是不是同判、别的读法有没有跟着变。
- S3：合成之后三处调用的语义与合成之前逐处相同吗；两份都没自证过时 CPU 与 GPU 判得一样吗。
- S4：每条坏镜像钉的是不是判决点名的那一形，变异行改坏之后红的是不是点名那一句。
- S5：带「没有文件」的版本表在崩溃放量 CPU 判器、GPU 输入表、恢复内核三处判法一致吗（与 `crash.rs` 的 `classified_oracle_violation_for_the_version_table`）；复用键对已有流一字不变吗；`JUDGE_TABLES_VERSION` 与判法版本闭包换了之后旧判定不被复用吗。
- S6、S7：对拍流与钉值。
- S8：变异行锚点、替换的 25 行名字与必须红的测试照旧吗。

### K2　哪一格会错
在冻结副本的拷贝上攻，只跑 harness 档（checker 档的用例与 GPU 一律不跑；GPU 一侧按 wgsl 读码推、或在 harness 档用 CPU 判法造对照）：
- S1：一份合法或坏镜像，CPU 新键与 GPU 新键判得不同；或改按读回落点认之后一形本该红的漏判（例：两份副本都读得出而内容不同、第 1 条那一份被走过）。
- S2：槽号、长度、空段端点的一个取值让守卫放过不该放的或拒了该放的。
- S3 / S5：两份都没自证过那一句在 CPU 与 GPU 判得不同的一形；带「没有文件」的版本表在崩溃放量与层 0 判得不同的一形。

### K3　被判的文件名单
`research/prompts/m3-batch33s-code-r1-snapshot/crates-sha256.txt` 里那 18 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决定的改法推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch33s-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与脚本放 `research/prompts/m3-batch33s-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G；harness 的 `checker_known_bad_images` 跑给 8G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`，用法写在文件头第一行；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
