# 删除批第 23 批合入的代码：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 K1 K2 K3 -->

## 一、题面

删除批第 23 批两份补丁 2026-10-05 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-05 那几条）：G 让 GPU 判器跟上第 22 批的九个新下标、I-9.4 第四句、I-9.10 墓碑那一个等号与 I-9.17 / I-9.23 走读内核的改法，并收第 21 批 G 留下的两处坏盘分叉与 size 偏移合一；X 关 I-9.29 在最新根实例表读不出时记判不了、checker 档新池新建文件流的计数分组、E142 导出 `--device-bytes` 退 134、一条用例改名。这一轮判两件事：代码做的是不是条款、判决与用户定案说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch23-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch23-code-r1/tree/`（合入之后的主工作区 `crates/` 与 `Cargo.*`），被判的 13 份文件的 sha256 在 `research/prompts/m3-batch23-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索，以代码为准）：`/tmp/claude-1000/m3-batch23/{g,x}/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或定案 |
|---|---|---|---|
| N1 | GPU：九个新下标里八条由内核置位（走读内核 I-9.17、I-9.23；扫描内核 I-9.24、I-9.18–I-9.21、I-9.29 跨版本出生代表），I-9.22 照 I-4.4 先例恒不置位；超上限记判不了（站点号 55–60）；草稿区 298 736 → 315 376 字 | `crates/singlefs-checker-tier/src/crash_judge_checker*.wgsl`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs` | `.claude/kb/invariants.md` 那几行；用户 2026-10-03 定 GPU 判器全覆盖；CPU 实现 `crates/singlefs-checker/src/walk.rs` |
| N2 | GPU：I-9.4 第四句并进下标 40、I-9.10 墓碑等号并进下标 43 | 同上 | 同上 |
| N3 | GPU：零条目、层级 0 的 extent 上段根与 inode 树根只在有条目时核条目宽；size 偏移合成一处定义 | `crash_judge_common.wgsl`、`crash_judge_checker_scan.wgsl` | `crates/singlefs-core/src/recovery.rs` 删完那一支；m3-batch20-code-r1 判决第三节第 2 条 |
| N4 | 对拍新加 13 条（I-9.17、I-9.18、I-9.19、I-9.21、I-9.4、I-9.29、I-9.10 各一，I-9.23、I-9.20 各二，两处坏盘根各一，「删 → 回退 → 再删」合法流一条），队列 {1, 7} 那条多断言 I-9.24 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 同 N1 |
| N5 | CPU：I-9.29 在最新根实例表链上有一片读不出时报部分没判、一个号都不比 | `walk.rs`；`crates/singlefs-harness/tests/checker_known_bad_images.rs` | `invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」 |
| N6 | checker 档 `layer0_quick_tier_matches_the_full_tally_shape` 的分组：I-9.5、I-3.4、I-9.24、I-9.29 进「根已落」，I-4.7、I-4.6、I-9.18 一组 0，I-9.17、I-9.19–I-9.23 一组 0，I-4.4 单独 0（后几样推的，提交时核） | `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` | 用例自己的判据：阴性结果要能和「代码没跑到」分开 |
| N7 | E142 导出：建盘前 `try_reserve_exact` 试分配、分配不出打原因退 1 | `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | m3-batch20-code-r1 判决第二节 K1 那一行（`--device-bytes 18446744073709551615` 退 134） |
| N8 | 用例改名连带 3 行变异；变异表追加 21 行、替换 4 行 | `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs`、`crates/mutations.tsv` | m3-batch21-code-r1 判决 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch22-code-r1 判决第三节列的改法（I-9.17 / I-9.23 读孩子的方式、覆盖缺口 M1–M8、新九条的专门不适用理由与慢档用例豁免）：第 24 批。
- GPU 侧 I-9.29「实例表读不出记判不了」：第 24 批（G 交回时 X 还没落地）。
- E142 的拒绝判据随机器 overcommit 变：第 24 批改成按算术判。
- 别的线的改动：由那两条线的代码轮判。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（169 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-06-701910/`）；fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0。checker 档没跑：GPU 对拍、G 的 18 行与 X 的 checker 档变异、N6 的分组都归提交时。
- 实现员 G 自报：三个内核源码与 18 行变异各自施加之后的源码都过了 naga 解析与校验；带 `--features gpu` 的 check 与 clippy 过。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N8）
- N1–N4：GPU 每一判与 CPU `walk.rs` 同一不变量那一判逐支对（判什么、判不了什么、置哪一位）；超上限的那几处是不是记判不了而不是判成立；对拍每条新用例覆盖的是不是它名字说的那一判。
- N5：判不了的条件与 `invariants.md` I-9.29 字面、与 CPU 候选集的滤法对不对得上。
- N6：每个分组按代码推的依据对不对（这条流上哪些状态有墓碑树、谱系树、队列）。
- N7、N8：退出码；改名连带的变异行是不是全了。

### K2　哪一格会错（重点 N1、N3、N5）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- N1、N3：找一形 CPU 与 GPU 判得不一样的镜像或流（合法流上的误红、坏镜像上的漏判、判不了的条件两边不同），用户决定的那几步放开扫。
- N5：实例表读得出时有没有被误报判不了；读不出时有没有漏判。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch22-code-r1 的 N4 其余几形、N5、N7、N9、N6。

### K3　被判的文件名单
`research/prompts/m3-batch23-code-r1-snapshot/crates-sha256.txt` 里那 13 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格与用户定案推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N3、N5 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫，不写死 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。
- 腿提的新改法算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-batch23-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch23-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比摘要之前先把并行测试的 `test … ok` 行排序；用自己的编译目录，不与别处共用。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
