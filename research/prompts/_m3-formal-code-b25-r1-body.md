# 第 25 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 V4 V11 V17 V18 V19 S1 S13 Q5 -->

## 一、题面

第 25 批（`records/2026-10-01-里程碑三形式化接入.md#25`）合入了两份补丁，另有 25-3 执行员在合入之后改的装置收尾；改动全文 `research/prompts/_m3-formal-code-b25-r1-diff.md`。

- 25-1：O1 理想模型与胶水罩 inode 树第二片及以后的叶容器。`crates/singlefs-harness/src/model_comparison.rs` 删掉「实现交回的叶容器只许最左那片」的断言，`TransactionUnit::InodeLeafContainer(index)` 映到 `ModelUnitRole::inode_leaf_container_at(index.0)`；`crates/singlefs-harness/src/model.rs` 记 inode 树里几条，第 k 条落第 ⌊k ÷ 233⌋ 片，跨片时哪几片记成这次重写；新用例 `crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs`；变异 3 行（「O1 跨叶容器」开头）。实现员自报两处没罩到：叶容器超过 135 片（建满 31455 个）时模型 `assert!` panic、实现拒；建到第二片之后再管理员回退那一支有代码没测试。
- 25-3a 与 25-3：E171（三oracle合谋测试） 第 1 次重跑的装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`（整份不在 HEAD 里；`_…-diff.md` 第三节是暂存区那一版到冻结树那一版的差）：RefFS 由 `#[path]` 改成 checker 档的正式依赖（`crates/singlefs-checker-tier/Cargo.toml`）；V4（panic 那一支）、V11（判不出那一支）；Q5 候选二：144 个等价类加 288 态交叉核对、类核对 ①–④、停机条件 S13、`class_landing_mismatches` 计进 S13、`q5_partition` 行与 `s13=` 字段、没有根写可判时记 `q5_not_run=no_root_write` 退 3；S1 回比加候选二的 5 个常量与 6 条；第 1618、8729 行一带注释；内联用例 40 → 44。变异：装置 28 行（「E171R1 装置」开头，含 V4、V11、V17–V19）。
- 被判的条款：跑前登记 `research/prompts/e171-r1-prereg.md`（修订 2 第 680–848 行、修订 3、主 agent 认定、修订 4）；D13（验证路线） 已定项 6（三个互相独立的 oracle）与已定项 3（RefFS）。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本：E171 第一段新冻结树 `scripts/experiments/target/e171-r1-frozen-tree/`（快照摘要 4d071c12…，284 个文件；crates 的 sha256 清单另放 `/tmp/claude-1000/m3-formal-code-b25-r1/crates-sha256.txt`，291 行，含 Cargo.toml 与 Cargo.lock）。那棵树正被 E171 第一段的长跑脚本用着：**只读，不在里面编、不往里写**；要编就 rsync 一份到自己的草稿目录。代码引文读冻结副本、写 `crates/…` 相对路径与那棵树里的行号。
- 合入验证（删除线那一趟，日志 `/tmp/claude-1000/merge-and-verify/2026-10-05-2930043/`）：harness 档 151 个测试二进制全绿，fmt、clippy 补跑退 0。25-1 新用例在里面。
- 装置 44 条内联单测与 28 行装置变异属 checker 档，只编过、没跑（25-3a 只编不跑，变异没证红；25-3 只核了 28 行锚点各恰命中 1 次）。25-3 在冻结树上跑过前台自检：S1 71 条全 holds（K = 234），A12 12 项全过。
- 主仓 `grep -c '模型今天只罩一片叶容器的那几种发布'` 为 0。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　25-1：O1 跨叶容器
模型「第 k 条落第 ⌊k ÷ 233⌋ 片」与实现把 inode 记录分进叶容器的规则（读 `crates/singlefs-core/` 里切叶容器那几行）是不是同一个规则：删除之后条数变少、续做、回退、多实例几种历史上还对不对得上；胶水的映射有没有一种实现交回的叶容器编号模型认错了而 `agrees` 仍为真（放过），或合法历史上 `agrees` 为假（误红）。实现员自报的两处（超过 135 片模型 panic、第二片之后回退那一支没测试）该怎么处置：模型答拒、加用例、还是记账，各自的依据。3 行变异各只红点名那条吗。

### K2　E171 装置：Q5 候选二
144 个等价类的分法站不站得住：同一类里的崩溃态，三条 oracle 的判定是不是真的相同（找一对同类而判定不同的态）；288 态交叉核对与类核对 ①–④ 分不分得出一个分错的类（V17–V19 之外，还有没有一种分错它们全放过）；S13 与 `class_landing_mismatches` 的计法；`q5_not_run=no_root_write` 那一支退 3 是不是只在登记写的那种情形下走到。

### K3　E171 装置：其余改动
RefFS 改正式依赖之后依赖方向对不对（`.claude/rules/verification.md`「定义与名字」RefFS 那一行与门禁 checker-independence-and-sync 的 ⑤⑥⑦）；V4、V11 两支改得是不是照登记修订 2 第 2.4 节；第 1618、8729 行一带注释与代码是否一致；28 行装置变异各自是不是只让点名的那条红、有没有等价或取样点不敏感的（`.claude/rules/mutation-sampling.md` 三分）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K2、K3 | 从登记与条款正推每处改动做的是不是条款说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | K1、K2、K3，重点 K1、K2 | 找反例：一段合法历史让 25-1 的模型误红、或一个坏实现让它放过；一对同类而判定不同的崩溃态；一种分错的类让交叉核对与类核对全放过。在自己拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试（`cargo test -p singlefs-checker-tier`，含装置内联单测）、`crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」。harness 档测试可以跑；装置二进制在自己的副本里编出来、经 `scripts/run-with-memory-cap.sh` 跑小规模（流一、流二或自造的小历史）可以，跑不起来写明。
- 不读 E171 第 1 次重跑这一段正在写的产物（`research/results/e171-r1-*2026-10-05*`）。

交付：腿的报告写 `research/prompts/m3-formal-code-b25-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b25-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
