# 删除批第 24 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 K1 K2 K3 M1 M2 M3 M4 M5 M6 M7 M8 -->

## 一、题面

删除批第 24 批两份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：K 关 m3-batch22-code-r1 判决第三节第 1–3 条（I-9.17 / I-9.23 读孩子的方式、覆盖缺口 M1–M7、队列同号两条归谁管）、新九条的专门不适用理由与慢档用例豁免、E142 拒绝判据改按算术；G 关同一判决 N4 的 GPU 那一侧与 GPU 侧 I-9.29 实例表读不出那一支。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch24-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch24-code-r1/tree/`（合入之后的主工作区，已含形式化线第 34 批与根环线 10h），被判的 10 份文件的 sha256 在 `research/prompts/m3-batch24-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`/tmp/claude-1000/m3-batch24/{k,g}/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| N1 | CPU：容器树内部条目按孩子单元头的类标签认孩子是哪一类，按那一类的长度读回、整单元校验和对得上才算数；类与类型段不符判 I-9.17 / I-9.23 红、按孩子自己那一类登记引用、不往下走、不记走读失败；inode 树 I-9.2 同法改 | `crates/singlefs-checker/src/walk.rs` | `.claude/kb/invariants.md` I-9.2、I-9.17、I-9.23；m3-batch22-code-r1 判决第二节 N4 |
| N2 | 坏镜像：N4-a、N4-i、N4-f、N4-j 四形加谱系 N4-c 与 inode 两形，各只红目标；M1–M7 各补坏镜像或变异；队列同号两条由 I-1.1 管的坏镜像 | `crates/singlefs-harness/tests/checker_known_bad_images.rs`、`pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | 同上；m3-batch22-code-r1 判决第二节「K1 覆盖缺口」 |
| N3 | 新九条里八条在 walk.rs 写专门不适用理由（`pub const`），I-9.24 那句在形式化线 34-1 改的段里没动；`reuse_after_raising_the_floor.rs` 抬 F 用例给 I-9.18–I-9.23 加豁免、每条只认那一句 | `walk.rs`、`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` | `crates/singlefs-checker/src/image.rs` 缺省理由那一句分不开「没有对象」与「一次都没判」 |
| N4 | E142 导出：拒绝判据改按算术（checked 乘法，两块盘空闲图合起来上界 1 GiB，上界是推的） | `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | m3-batch23-code-r1 正推腿线索（2⁴³ 起试分配过、真分配撞 4G） |
| N5 | GPU：走读内核读孩子类标签同 N1（G 对齐的是 K 副本里 sha256 c9884ecc… 那一版 `walk.rs`，K 之后又改成 2a95615f… 那一版才交回） | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | 同 N1；用户 2026-10-03 定 GPU 全覆盖 |
| N6 | GPU：I-9.29 最新根实例表整条链读不出时不置位、不记判不了；对拍补一形 | `crash_judge_checker_scan.wgsl`、`crash_judge_gpu.rs`、对拍用例 | `invariants.md` I-9.29；CPU 第 23 批 X 那一支 |
| N7 | 变异表：K 追加 21 删 1（第 23 批 X 那行 E142 试分配，锚点随它删了），G 追加 6 | `crates/mutations.tsv` | `.claude/singlefs-ai-sop/rules/show-me-test.md`「新增的测试必须先证明它会红」 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch23-code-r1 判决第三节（I-9.29 判不了的门看主走读有没有走到链的末片）：第 25 批；N6 跟的是今天的门，门改了 GPU 要跟着改。
- M8（去重键里的 txg）：已记欠账 C725。
- GPU 对拍与 G 的 6 行、K 的 checker 档变异：要显卡与 checker 档，提交时由主 agent 跑。
- 别的线的改动：由那两条线的代码轮判。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-06-1324740/`）；fmt 与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0；`reuse_after_raising_the_floor --ignored` 13 条全过。checker 档没跑。
- K 自报：E142 `--device-bytes 8589934592` 输出与 `research/results/e142-r24-p8g-crates-dump-2026-10-05.out` 逐字节相同；不带参数那份只差第 1 行 `E7INPUT`（`scripts/experiments/replay.sh` 比对前删掉这一行）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N7）
- N1、N5：CPU 与 GPU 在读孩子那一判上逐支对（同一张镜像，CPU 判什么、GPU 置哪一位）；G 对齐的那一版与 K 最终那一版之间的差别有没有落在这一判上。
- N2：每份坏镜像是不是只红目标；每行新变异改坏的是不是名字说的那一处、红在哪条断言。
- N3：八句专门理由各在哪一支报；豁免是不是每条只认那一句、没把 I-5.5 那一支改回去。
- N4、N6、N7：退出码与上界；GPU 不置位的条件与 CPU 判不了的条件是不是同一个。

### K2　哪一格会错（重点 N1、N5、N3）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- N1、N5：类标签与类型段的每一种组合（类型段 0 / 1 / 5 / 条款外的值 × 孩子是码 2 节点 / 码 3 容器 / 别的类 / 读不出）、树长到第几层，有没有一形漏判、误红，或 CPU 与 GPU 判得不一样。
- N3：合法历史上新九条有没有落到缺省理由上的（应当报专门理由而报了缺省理由，或该判的没判）；用户决定的那几步放开扫。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch23-code-r1 的 N1、N3（CPU 与 GPU 一致）。

### K3　被判的文件名单
`research/prompts/m3-batch24-code-r1-snapshot/crates-sha256.txt` 里那 10 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 N1、N5、N3 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch24-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-batch24-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比之前把并行测试的 `test … ok` 行排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
