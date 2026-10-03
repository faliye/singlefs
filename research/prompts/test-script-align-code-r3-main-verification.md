# test-script-align-code-r3 判决（2026-10-02，代码轮第三轮，最终轮）

正文 `research/prompts/_test-script-align-code-r3-body.md`；腿：正推 `research/prompts/test-script-align-code-r3-sonnet-output.md`（L2、L3、L5）、攻方 `research/prompts/test-script-align-code-r3-opus-output.md`（L1、L4 兼 L2），模型目录各自 `-model/`。前两轮判决 `research/prompts/test-script-align-code-r1-main-verification.md`、`research/prompts/test-script-align-code-r2-main-verification.md`。

被判的 `crates/*/src/*.rs`：`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/numeric_environment_variable.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/model_comparison.rs`。

## 一、核查

- 开工快照：冻结副本 `/tmp/claude-1000/test-script-align-code-r3/tree/` 对 `research/prompts/test-script-align-code-r3-snapshot/crates-sha256.txt` `sha256sum -c` 全对。
- `cite-check.py <两份报告> --root . --background research/prompts/_test-script-align-code-r3-background.md`：「核了 88 处引文，对上 88 处，没判 0 处」。
- 主 agent 读冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 390–410 行 `JudgeInputsOfTheFlow` 的文档：B0 的上界取「基镜像里本来就有的根、写表里到这个点为止的根槽写、没做完那次发布」三者最大，文档自己写明前提「基镜像里是 mkfs 的第 0 代根」。攻方 L1 造的基镜像是崩溃状态（记录落了、根没落），journal 恢复会落到基镜像里没有根的那一版——前提不成立，机制成立。攻方的数是它草稿副本上原型量的，不进 kb。
- checker 档：主 agent 在冻结副本上带 `SINGLEFS_HEAVY_TESTS=commit` 实跑过（`scratchpad/r3b-libint.log`），这一批碰到的用例全绿；剩下 8 条红不归这一批（GPU 判器 5 条是别的会话暂存的内核改动，`crash_enumeration_*` 3 条在 HEAD 上同样红）。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| L1 基镜像是崩溃状态时 B0 截掉 journal 够得着的那一版，身份、复用键、计划哈希相同而 8 态判定不同 | 打中（潜伏：今天登记的流都从 mkfs 起） | 第三轮新冒出的改法零轮，交用户表第 1 行 |
| L1 今天的流上块键相同的点 | 没打中（展开 6 两流 44 态、到 D 对到卸载 115 态；展开 16 到 D 对到卸载 131185 态，判定与核对位 0 态不同） | 维持；只抽一次，不作「没打中」用 |
| L1 txg 不严格递增那条新用例、M3 拆成的两条断言 | 没打中（各红在点名的那一条） | 维持 |
| L4 变异表 42 行 | 没打中（静态核全过；原型与 prove-red 证了 13 行） | 维持；第 1736 行只在并行度大于 1 的机器上红，记一笔，提交时整表复跑照它的出路判 |
| L2 前后空白口径两处不一（共用解析去空白，`history.rs` 第 4095 行一带的 `Result` 版不去） | 打中（两边都报错，不静默） | 交用户表第 2 行（零轮） |
| L2 种子数写 0：整批 0 个崩溃状态，按只判清单外失败那一组判成绿 | 打中 | 交用户表第 2 行（零轮）；照上游 `show-me-test.md`「扫到 0 项也不是通过」 |
| L2 非 UTF-8 报错两种写法；`crash_amplification.rs` 的 SHARE、主机权重、IMPORT 三处非 UTF-8 当没设 | 线索 | 记一笔，不在这一批 |
| L2 依赖判法只看第一个引到 `history` 的文件 | 线索 | 记一笔 |
| L3 C610 回扫与术语改名 | 没打中（83 处逐处核，没删现状句与指路；按字面认的脚本与门禁全认新串） | 维持；`tests/common_crash_points/mod.rs:2` 还有一处「用户 2026-09-28 定」（在 `crates/*/tests/*.rs` 通配之外），交用户表第 2 行一并做 |
| L5 交来的四样 | 没有一样会让红变绿；`history.rs` 留的 `pub use` 让 `c561-sigma-full` 判法闭包从 58 份多到 63 份（多重判方向） | 交用户表第 2 行一并做（改引新模块、删转出） |

## 三、交用户（零轮）

| # | 事 | 候选 | 状态 |
|---|---|---|---|
| 1 | L1：基镜像不是 mkfs 时 B0 可能让判定不同的两条流共用块键 | A：B0R，上界再并上「对基镜像本身照 journal 恢复后实际落到的根」（攻方只在自己模型上量过，三条点名用例照样绿）；B：计划时断言基镜像只有 mkfs 的第 0 代根、journal 恢复不越过它，不满足就拒（把潜伏的错复用变成显式拒绝，前缀共享不受影响）；C：只记欠账 | 交用户 |
| 2 | 小项一批：`Result` 版解析也去前后空白；种子数（与同类规模变量）写 0 判红；`tests/common_crash_points/mod.rs:2` 那一处来历删掉；`record_checker_judges_absence_by_the_persisted_set.rs` 改引新模块、删 `history.rs` 的 `pub use` | 主 agent 照判决当场派一件实现员做，改完带前缀实跑 checker 档核 | 主 agent 定 |

第三轮之后不再开轮（`.claude/rules/three-way-inference.md`「多轮」）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-02 不受影响：这一轮判的是实现 |
