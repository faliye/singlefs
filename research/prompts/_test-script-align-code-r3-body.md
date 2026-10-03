# 对接上游测试脚本规范：代码轮第三轮正文（2026-10-02）

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 J1 J2 J3 J4 J5 B0 -->

## 一、题面

`.claude/rules/implementation-workflow.md` 三步里的第 2 步，第三轮，最终轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停；这一轮新冒出来的零轮形态写进判决的交用户表、标「零轮」，不再开第四轮）。前两轮判决 `research/prompts/test-script-align-code-r1-main-verification.md`、`research/prompts/test-script-align-code-r2-main-verification.md` 整份进材料；前两轮四条腿的报告与模型目录可读。这一轮只攻第二轮站住、写回的形态，不重复前两轮攻过的角度。

第二轮之后写回的改动：两份补丁 `/tmp/claude-1000/impl-code-r2-{crash-amp,parse-gpu}/patch/`（报告在各自目录 `report.md`）与两件直接改主工作区的收尾 `/tmp/claude-1000/impl-pre-r3/report.md`、`/tmp/claude-1000/impl-pre-r3-2/report.md`；主 agent 另做了全仓术语改名（「轻用例 / 耗时用例」→「快档用例 / 慢档用例」，`crates/` 里 9 份只改 `#[ignore]` 文字与注释）。第二轮冻结副本到这一轮冻结副本的整份 diff 在 `research/prompts/test-script-align-code-r3-snapshot/r2-to-r3.diff`，**混着别的会话同期的改动**（核心层、GPU 判器的 wgsl 内核与 `crash_judge_gpu.rs` 一带），不判那几处；判法：只判上面几份报告写明改了的地方与术语改名。

被判的 `crates/*/src/*.rs`：`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/numeric_environment_variable.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/model_comparison.rs`；连同各报告点名的测试文件、C610 回扫改过的测试文件头与 `crates/mutations.tsv` 的新增与改写行。

## 二、实现今天的样子（主 agent 的观测，2026-10-02）

- 冻结副本 `/tmp/claude-1000/test-script-align-code-r3/tree/`（`crates/`、`Cargo.*`、`.cargo/`，另带 `.claude/gate.d/stage-inputs.tsv`），sha256 清单 `research/prompts/test-script-align-code-r3-snapshot/crates-sha256.txt`（225 份）；读冻结副本，不读主仓 `crates/`。
- 主 agent 在冻结副本上带 `SINGLEFS_HEAVY_TESTS=commit` 实跑 checker 档 `--features verdict-store,gpu --lib --test '*'`（日志 `/tmp/claude-1000/-home-fy5090-code-singlefs/1b1be0b2-61aa-47ec-b6e0-aa04cd899d35/scratchpad/r3b-libint.log`）：两流用例 7 条与依赖用例 `crash_identity::tests::every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries` 都绿；剩下 8 条红都不归这一批——GPU 判器 5 条（「表越过了内核的容量」，HEAD 上绿，这一批没动内核与表），`crash_enumeration_{findings_log_…,new_pool_file_creation_stream,sharded_across_processes}` 3 条（HEAD 上同样红）。harness 档合入后 105 个测试二进制全绿；`cargo fmt --check`、clippy 带两个特性退 0。
- 实现员留下、主 agent 判为不再派、交这一轮攻的四样（`/tmp/claude-1000/impl-pre-r3-2/report.md`「要你定的」）：取样参数的每段步数仍走 `history` 的 `Result` 版解析；`history.rs` 留一行 `pub use` 转出；`numeric_environment_variable` 模块装了读非数值变量原文的函数；E158/E161/E165/E168 装置与 `layer0_progress.rs` 分片变量的同形副本没改。

## 三、格

### L1　B0 截版本表与 M3 重造
B0（够得着的根取基镜像里的根、点之前写过的根槽写、没做完那次发布的根槽写三者最大）：有没有一对点块键相同而某个状态判定不同（攻方第二轮只在改前的代码上量过）；有没有一对判器输入真相同而块键不同（前缀共享该保没保）；txg 不严格递增那一支的新用例是不是真走到那一支。M3 那一对拆成两条断言后，两条各拦的变异是不是真拦得住。

### L2　数值环境变量解析收成一份之后
`numeric_environment_variable.rs` 与它的调用方：写错、写 0、空串、前后空白、非 UTF-8 各自的结局是不是全仓一致；`crash_injection_campaign_from_environment` 改成全 panic 之后，复现行喂回入口那一路（C607、S2）有没有因此丢了「写错报给人看」的那一层；依赖判法（随机历史脚手架不许经 impl 头带进来）在新模块下是不是照样成立。

### L3　C610 回扫与术语改名
测试文件头删掉的来历里，有没有删掉了说「文件测什么」的现状句、或删了指到 kb 的指路；剩下 7 行带日期的是不是都是路径名的一部分；术语改名后，按 `#[ignore = "harness 慢档用例：…"]` 字面认的脚本与门禁（`research/scripts/crash-case-check.py`、`research/scripts/broad-mutation.py`、门禁 code-source-discipline 与 harness-model-differential-and-scenarios 的样本）是不是全认得新串、没有一处还按旧串认。

### L4　变异表
第二轮之后新增与改写的行（崩溃放量 10 加 4、解析与 GPU 8、收尾 6 加 7、7 加 4）：锚点唯一、替换文改的是那一判、点名的测试跑得到、在今天的代码上推得出会红。

### L5　交来的四样（第二节末）
四样各自会不会让一个该红的东西绿着过去；哪一样其实是正确性问题、不只是风格。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | L2、L3、L5 | 从条款与报告推代码该有什么，核代码是不是做到；读冻结副本 |
| 云端攻方（Opus） | L1、L4，兼 L2 | 造反例、造流、造变异，在冻结副本的拷贝上量；只跑 harness 档与纯 Rust 小程序，checker 档包的测试不跑 |

## 五、跑前写死的判据

- 造出一对点块键相同而判定不同，或判器输入相同而块键不同 ⇒ 打中 L1。
- 造出一个输入让改后的解析给错结局（该 panic 不 panic、口径两处不一）⇒ 打中 L2。
- 找到一处删掉的现状句或指路、或一处还按旧串认的脚本与样本 ⇒ 打中 L3。
- 一条变异行锚点不唯一、替换文改的不是那一判、或点名的测试推不出会红 ⇒ 打中 L4。
- 四样里有一样会让该红的东西绿着过去 ⇒ 打中 L5。

## 六、交付

- 腿的报告写 `research/prompts/test-script-align-code-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/test-script-align-code-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件名加那份文件自己的行号（去 kb 文件里现查）；引代码写冻结副本里的路径与行号。
- 方案按 `crates/` 今天的实现来谈。不跑重型测试；不改主工作区；只停自己起的进程。
