# 删除批第 28 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 H-B H-V T1 T2 T3 -->

## 一、题面

删除批第 28 批一份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：k 关 m3-batch26-code-r1 判决第三节第 1–3 条。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch28-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch28-code-r1/tree/`，被判的 5 份文件的 sha256 在 `research/prompts/m3-batch28-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`/tmp/claude-1000/m3-batch28/k/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| H-B | 孩子两份都读不出时照判「子指针头部的出生树 == 身份引用的出生树」：inode 树 `judge_an_inode_entry_whose_child_does_not_read_back` 类型段 2 那一支、墓碑与谱系容器读不出那一支；GPU 两支同改 | `crates/singlefs-checker/src/walk.rs` 第 4200 行起、第 2394 行一带；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | `.claude/kb/invariants.md` I-9.2、I-9.17、I-9.23；m3-batch26-code-r1 判决第二节 H-B 那一行、第三节第 1 条 |
| H-V | `child_unit_class_other_than_the_entry_type_names` 两份都看，有一份自证成别类就交回那一类；GPU 同改 | `walk.rs` 第 4161 行起；走读内核第 2986 行起 | 同上；m3-batch26-code-r1 判决第二节 H-V 那一行、第三节第 2 条 |
| T1–T3 | 正推腿三条测试尖度：GPU 第二份副本那一支对拍与变异；读不出孩子坏镜像补「只改身份一段」三形；类型段 2 读不出孩子不过判 | `crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | m3-batch26-code-r1 判决第三节第 3 条 |
| 变异 | 追加 17 行、替换 7 行（2 行锚点断、4 行在两份都看之后成了等价变异、1 行 GPU） | `crates/mutations.tsv` | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch27-code-r1 判决第三节（类型段不认得 / 孩子是另一类两支不判 I-2.1、扫描内核两处链循环上限、关门流补变异）：第 29 批。
- 两份副本各自证成两种不同的别类时取哪一类、一份本类一份别类时按哪一类登记槽数：已记 C739。
- GPU 对拍与 GPU 那几行变异：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」「✓ clippy 过」（`/tmp/claude-1000/merge-and-verify/2026-10-06-3197393/`）；fmt（`cargo fmt --all -- --check`）与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0；`crash_enumeration_sharded_across_processes` 整份「10 passed」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（H-B、H-V、T1–T3）
- H-B：三棵树与 GPU 读不出孩子那几支，判的是不是 I-9.2 / I-9.17 / I-9.23 里只比父条目两段的那一小句；其余要看孩子头的小句是不是照旧不判。
- H-V：CPU 与 GPU 两份都看的次序、长度、两份都自证成本类 / 一份本类一份别类 / 两份各是别类时交回什么，是不是同一个。
- 替换的 7 行：4 行「等价变异」是不是在所有输入上都与新写法同值（照 `.claude/rules/mutation-sampling.md` 分「等价」与「取样点不敏感」），换上的写法是不是真改行为；13 条坏镜像与 4 条对拍的期望写没写全。

### K2　哪一格会错（重点 H-B、H-V）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- H-V：两份都看之后，有没有一形本来判对的现在判错（一份是本类且完好、另一份是别类的陈旧单元或被换过的块；条目不改与条目改坏；盘 0 / 盘 1）；合法流的崩溃状态里，一份没落盘、另一份落了旧单元时会不会误红。
- H-B：孩子读不出时照判的那一小句，会不会在合法状态上误红（例如子指针出生树与身份出生树在某种合法写法下本来就不同）。
- CPU 与 GPU 判得不一样的一形。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch27-code-r1 的 J1、CPU 与 GPU。

### K3　被判的文件名单
`research/prompts/m3-batch28-code-r1-snapshot/crates-sha256.txt` 里那 5 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 H-B、H-V | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch28-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch28-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比之前排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
