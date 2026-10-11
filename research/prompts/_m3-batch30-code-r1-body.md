# 删除批第 30 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 M1 M2 M3 M4 F-W F-IT X2 X13 X14 -->

## 一、题面

删除批第 30 批两份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：m 关 m3-batch28-code-r1 判决第三节第 1–3 条与第 2919 行锚点，fix 是合入时与根环线 10r 叠出编译错的补修（两条新对拍用例改用卡池取卡）。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch30-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch30-code-r1/tree/`，被判的 5 份文件的 sha256 在 `research/prompts/m3-batch30-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch30/m/report.md`、`~/.local/share/singlefs/work/m3-batch30/fix/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| M1 | F-W：墓碑、谱系、inode 三棵树「孩子是另一类」那一支，另有一份按类型段那一类自证过的，判条目那一条红后照类型段那一支往下走（槽数按类型段那一类）；CPU 两处、GPU 三处 | `crates/singlefs-checker/src/walk.rs` 第 4447 行 `some_copy_of_the_child_verifies_as_the_class_the_entry_type_names` 与调用方；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 3061 行同名 w_ 函数与调用方 | `.claude/kb/invariants.md` I-9.2、I-9.17、I-9.23、I-9.7、I-9.15、I-1.7；m3-batch28-code-r1 判决第二节 H-V 子树那一行、第三节第 1 条 |
| M2 | F-IT：`w_walk_instance_table_chain` 每一片在 I-1.12 之后判 I-1.2 | 走读内核第 1641 行一带 | I-1.2；m3-batch28-code-r1 判决第二节实例表链 I-1.2 那一行、第三节第 2 条 |
| M3 | 测试尖度：X2（inode 读不出孩子、子指针出生树加 1）、X13、X14（第一份头类标签不登记 / 头读不出、第二份自证成别类）各补一形 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | m3-batch28-code-r1 判决第二节 K1 测试那一行、第三节第 3 条 |
| M4 | 变异表追加 10 行、替换第 2919 行锚点；补修把两条新对拍用例改用 `the_card_pool_or_report_not_run` | `crates/mutations.tsv`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch29-code-r1 判决第三节（F-L0、F-零槽、L2 兜底那一形、测试尖度与 GPU 容器树三处）与 GPU 读 extent 上段零条目根缺豁免：第 31 批。
- 实现员报告「要你定的」第 1、2 条（F-W 之后别类那一份按类型段长度读对不上时 I-2.1 一族照红；GPU inode 类型段 0 而有一份按码 2 自证过的条目落「判不了」）：主 agent 按字面与偏保守认了，随 C739 写回；第 3 条（GPU 容器树两处 F-W 没对拍流）进第 31 批。
- `crash_enumeration_sharded_across_processes` 的 GOLDEN_TALLY 钉值变了：根环线说是 10o 加 I-3.14 带来的、由它重取，删除线调查员在二分，腿不攻。
- GPU 对拍与 GPU 那几行变异：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」「✓ fmt 过」，clippy 第一次红在两条新对拍用例调已删掉的取卡函数（与根环线 10r 叠起来才有），补修之后 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑退 0，fmt 退 0，`crates-mutation-rows.py static-check` 退 0。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（M1–M4）
- M1：CPU 与 GPU「另有一份按类型段那一类自证过」的判法是不是同一个（次序、长度、两份都自证过 / 一份自证过时）；往下走那一支与类型段那一支是不是同一段代码（槽数、读孩子、判身份、走子树）；没认出别类的条目是不是一份都不多读。
- M2：GPU 判 I-1.2 用的偏移与 CPU 对实例表单元判的是不是同一半（不判 I-4.2）。
- M3、M4：每份坏镜像与对拍流的期望写全没有；变异行点名的测试与参数筛不筛得到；第 2919 行替换是不是只换锚点。

### K2　哪一格会错（重点 M1）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- M1：往下走之后，别类那一份的引用登记、I-2.1 一族、子树里的不变量，有没有一形本来判对的现在判错；两份都按类型段那一类自证过而头里类不同时；盘 0 / 盘 1；三棵树层级 1 / 2。
- M2：实例表单元在根环里不同根之间共享、链途经已走过的片时，I-1.2 会不会重复判或判错片。
- CPU 与 GPU 判得不一样的一形。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch29-code-r1 的 L1 合法状态多红、L1 一份读不出、L2 链长与候选集里外。

### K3　被判的文件名单
`research/prompts/m3-batch30-code-r1-snapshot/crates-sha256.txt` 里那 5 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 M1 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch30-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch30-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
