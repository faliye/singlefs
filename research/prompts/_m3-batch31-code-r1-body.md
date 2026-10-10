# 删除批第 31 批合入的代码：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 N1 N2 N3 N4 N5 N6 F-L0 F-W F-IT X3 X4 X5 O3 -->

## 一、题面

删除批第 31 批一份补丁 2026-10-07 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-07 那几条）：n 关 m3-batch29-code-r1 判决第三节第 1、3、4 条，第 30 批 M 交回留下的两处对拍，根环线报的 GPU 读 extent 上段零条目根缺 CPU 那条豁免。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch31-code-r1-diff.md`。冻结副本 `~/.local/share/singlefs/work/m3-batch31-code-r1/tree/`，被判的 5 份文件的 sha256 在 `research/prompts/m3-batch31-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`~/.local/share/singlefs/work/m3-batch31/n/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| N1 | F-L0：墓碑、谱系两棵树层级 ≤ 1 节点里类型段 0 的条目，孩子不往下走、记走读失败之前，每一份副本照判 I-2.1（共用函数 `judge_every_copy_of_a_child_the_walk_does_not_go_into`），引用照旧不登记；GPU 同改 | `crates/singlefs-checker/src/walk.rs` 第 2500 行；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 3183 行 | `.claude/kb/invariants.md` I-2.1、I-4.8、I-7.2、I-7.4、I-9.17、I-9.23；m3-batch29-code-r1 判决第二节 K2 L1 第三支漏判那一行、第三节第 1 条 |
| N2 | GPU extent 上段根豁免：`w_read_index_node` 加 `key_range_reading == 3u`（层级 0、零条目只判 key 宽），`w_walk_extent_tree` 读根时传 3、读孩子传 2 | 走读内核第 1515、2192 行 | I-1.1；D8（核心索引结构） 已定项 16；CPU `walk.rs` 的 `PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller` |
| N3 | 站点 37 / 38 兜底那一形：候选根自己那张实例表链 257 片、最新根的链途经它第 100 片；变异把扫描内核两个原因字常量写成 0 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 10270 行一带；`crates/mutations.tsv` 第 3065 行 | I-9.29；m3-batch29-code-r1 判决第二节 K1 L2 那一行、第三节第 3 条 |
| N4 | 测试尖度：正推三形 X3（登记过的类也按两种长度比）、X4（没登记只按 32 KiB 比）、X5（读不出算对得上）各一份坏镜像；GPU 容器树四处 `w_judge_every_copy_…` 调用各一条对拍流 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` 第 13069 行起；对拍文件第 9662 行起 | I-2.1；m3-batch29-code-r1 判决第三节第 4 条 |
| N5 | 第 30 批留下的：GPU 容器树两处 F-W 各一条流；inode 类型段 0 而另一份按码 2 自证过一条流（期望 GPU 判不了、原因字 4、回落 CPU 同判） | 对拍文件第 9662 行起 | I-9.2、I-9.17、I-9.23（2026-10-07 写进的「孩子头里的类与类型段说的那一类不一致时怎么往下认」读法）；`research/prompts/m3-batch30-code-r1-main-verification.md` |
| N6 | 变异表追加 12 行（CPU 4 行实现员证红过，GPU 8 行只核过原文命中一次） | `crates/mutations.tsv` 第 3054–3065 行 | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- F-零槽（GPU `read_at` 对没写过的单元区槽交回全零切片）：要改 `crash_judge_common.wgsl` 与 `crash_judge_tables.rs`，第 32 批。
- 扫描内核第 12 行那句说明与 m3-batch29-code-r1 判决 L2 格不符：第 32 批改。
- `crash_judge_dispatch.rs` 里钉的走读内核展开数（323759 → 326147）：第 32 批改钉值，期间那一条红是预期的。
- m3-batch30-code-r1 判决第三节（G1–G4、F-IT 另两条流的变异、写回）：第 32 批。
- GPU 对拍与 GPU 那几行变异：提交时由主 agent 跑。

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 合入验证 `merge-and-verify.sh --memory 8G --threads 10`「✓ 测试全绿（170 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（clippy 带 `--all-targets --all-features`，日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-07-1500192/`）；合后 4 份源码与实现员改后的那一版逐份 `cmp` 相同，`crates-mutation-rows.py static-check .` 退 0。
- 实现员用 naga 30.0.1 量的展开数：走读内核 326147、扫描 378232、恢复 316712，预算 400000。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（N1–N6）
- N1：CPU 与 GPU 是不是同一支（层级 ≤ 1、类型段 0、孩子不是另一类或另一份按码 2 自证过）；判 I-2.1 用的长度读法与 I-2.1 那一行写的是不是一致；引用登记、走读失败次数改没改。
- N2：GPU 走「读根」那一种读法的条件是不是与 CPU `walk_extent_upper_node` 只在 `position` 为 `None` 时那一种读法一一对应；层级 0 非空的根、层级 ≥ 1 的根是不是照旧判分隔 key。
- N3–N6：每份坏镜像与对拍流的期望写全没有；变异行点名的测试与参数筛不筛得到；第 3065 行把常量写成 0 与「删掉那两句」差在哪。

### K2　哪一格会错（重点 N1、N2）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- N1：合法状态上会不会多红 I-2.1；层级 0 的根下（不是层级 1）类型段 0 那一形；孩子一份读不出、一份对得上；盘 0 / 盘 1。
- N2：`pending_level == NONE` 是不是只在读根那一次成立；extent 上段根层级 ≥ 1 而零条目、层级 0 而 key 宽不对时 CPU 与 GPU 判什么。
- N3：那条流里实例 3 写行那条候选根在不在回退候选集、走读内核会不会先记站点 61 让 37 / 38 走不到。
- 观察 O3（m3-batch30-code-r1 判决第二节）：F-W 之后三棵树「走过」的键取 `child.locations[0]`，读回的可能是位置条目 1 那一份；造得出盘序不同判得不同的一形就算打中。
- CPU 与 GPU 判得不一样的一形。
- 第二次抽样：m3-batch30-code-r1 的 F-W CPU（inode / 墓碑 / 谱系，层级 1，盘 0 / 盘 1）、F-W 与 F-IT 的 GPU 读码；m3-batch29-code-r1 的 L1 一份读不出、L2 链长与候选集里外（合法状态多红那一格要层 0，归提交时，不攻）。

### K3　被判的文件名单
`research/prompts/m3-batch31-code-r1-snapshot/crates-sha256.txt` 里那 5 份；冻结副本里每一份都要有一格判到。

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
- 腿的报告写进 `research/prompts/m3-batch31-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch31-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `research/scripts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；同一时刻只起一条 cargo；`compare` 比之前排序、去掉耗时字段、自己不依赖 `SINGLEFS_REPO`；一次调用跑不完的不设挂钟预算（或预算从环境变量取）；用自己的编译目录。草稿放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
