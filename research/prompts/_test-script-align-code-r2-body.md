# 对接上游测试脚本规范：代码轮第二轮正文（2026-10-01）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R1 L1 G1 S2 M3 J1 J2 J3 J4 J5 -->

## 一、题面

`.claude/rules/implementation-workflow.md` 三步里的第 2 步，第二轮。第一轮判决 `research/prompts/test-script-align-code-r1-main-verification.md` 整份进材料，第一轮两条腿的报告与模型目录可读（`research/prompts/test-script-align-code-r1-{sonnet,opus}-output.md`、`…-model/`）。这一轮攻第一轮打中之后写回的代码（改法都被攻过零轮），不重复第一轮攻过的角度（`.claude/rules/three-way-inference.md`「多轮」）。

写回的改动：三份补丁 `/tmp/claude-1000/impl-code-r1-{crash-amp,gpu,repro}/patch/`（报告在各自目录 `report.md`）与合入后收尾 `/tmp/claude-1000/impl-post-merge-code-r1/report.md`，主 agent 另加一行豁免注释（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` 大档入口上面的 `crash-case-check:not-a-crash-case`）。第一轮冻结副本到这一轮冻结副本的整份 diff 在 `research/prompts/test-script-align-code-r2-snapshot/r1-to-r2.diff`；**里面混着别的会话同期在这几份文件里的改动**（`crash_identity.rs`、`crash_injection.rs`、`crash_judge_dispatch.rs`、`crash_judge_gpu_agrees_…rs` 一带，合入前工作区就有），不判那几处，判法：只判四份报告各自写明改了的那几处。

被判的 `crates/*/src/*.rs`：`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/crash_identity.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/bad_disk_input.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-core/src/admission.rs`；连同各报告点名的测试文件与 `crates/mutations.tsv` 新增与改锚的行。

## 二、实现今天的样子（主 agent 的观测，2026-10-01，合入与收尾之后）

- 冻结副本 `/tmp/claude-1000/test-script-align-code-r2/tree/`（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo/`），sha256 清单 `research/prompts/test-script-align-code-r2-snapshot/crates-sha256.txt`（223 份）；读冻结副本，不读主仓 `crates/`。
- 主 agent 现跑：`merge-and-verify.sh` 合三份补丁，harness 档「测试全绿（105 个测试二进制）」；收尾之后实现员报 harness 档 lib 66、`fault_injection_fast_tier` 10、`random_histories` 15、core lib 141 全过，checker 档两种特性组合 `--no-run` 退 0；`research/scripts/crash-case-check.py --only placement,modules` 两样绿。checker 档一条没跑（重型，归提交时）；崩溃放量判定库合入后每块都不再复用（计划哈希变了）。
- 实现员推的、没量过的前提（报告原话的出处）：R1「点之后的写读不到」靠根槽写 txg 沿写表严格递增、基镜像只有 mkfs 第 0 代根（`/tmp/claude-1000/impl-code-r1-crash-amp/report.md`「停下交主 agent」第 4 条）；第 ② 段核对块用同一个块键、读的事实表是整条流抽的（同一节第 5 条）。

## 三、格

### J1　R1 的改法够不够、会不会伤到别处
`JudgeInputsOfTheFlow` 进计划哈希之后：判器读到的输入是不是都进了（版本表、`judged_root_index`、点之后的根槽写、起点盘……），有没有一样漏了；那两个推的前提在哪条流上不成立（txg 不严格递增、多代根），退成整段写表的那一支是不是真的会走到；第 ② 段核对块共用块键、读整条流的事实表，会不会让两条只在点后不同的流误复用。

### J2　改法本身的新缺口
L1 照实报之后按行相加的复用总计会不会重复计（`research/scripts/crash-amplification.sh` 的汇总）；G1 PCI 判定（只数 0x10DE 且基类 0x03、读不出判红）在虚机、容器、多卡、非 NVIDIA 独显上的结局；S2 重放行带 `SINGLEFS_CRASH_INJECTION_ASSERTIONS` 之后，「复现这一段」只判新发现那一支照抄还复现不复现；H9 `computation_routes` 在「登记了却没卡」「没编 gpu 特性」「GPU 跑法下没登记 crash_verify_gpu 改在 CPU 上核」几种组合下的结局；同族环境变量抽成一个解析函数后，线程数设 0 现在 panic，有没有调用方拿 0 当「取默认」。

### J3　变异表
新增的行（崩溃放量 25、GPU 6、复现行 9、收尾 8）与改锚的行（第 1585、1587、1590、1679、1708、1712 行一带）：锚点唯一、替换文改的是那一判、点名的测试跑得到；M3 新用例让第 1587 行会红的推断（改坏根槽写的 magic 之后那个点有状态判红）成不成立；C604 没留变异那一条的理由（有卡的机器上改回去走同一条路）站不站得住。

### J4　编码纪律
`enum_listing_every_member!` 宏挪进 `crates/singlefs-core/src/admission.rs`、harness 留转发宏：核心层多了一个只为测试生成清单的宏，合不合 `.claude/singlefs-ai-sop/rules/code-discipline.md`「重复要生成」与 D17（实现分层与第三方管道）的分层；`crash_amplification.rs` 第 85 行一带展开上限的解析与 `history.rs` 新抽的共用解析函数是不是同一件事写了两份；新名字、`#[should_panic]` 用例、环境守卫。

### J5　第一轮判决（辩方复核）
第一轮判决里「可议，不采」（K4 三处名字、错误口径不一）与「只抽一次」（K2 没打中的几格）判得对不对；第一轮采的改法是不是同样打中了被攻的形态、有没有同样打中别的写法。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | J5 为主，J3、J4 | 复核第一轮判决够不够得着、是不是同样打中所有替代方案；替被判「不采」的一方辩护；读冻结副本 |
| 云端攻方（Opus） | J1、J2 为主，J3 | 攻写回的每一处改法：造反例、造流，在冻结副本的拷贝上量；只跑 harness 档与纯 Rust 小程序，checker 档包的测试不跑 |

第二轮派辩方不派正推（`.claude/rules/three-way-inference.md`「多轮」：至少一轮派辩方复核前一轮判决）。

## 五、跑前写死的判据

- 造出两条流，判器输入不同而块键相同（同一判法版本下会互相复用），或判器输入相同而块键不同（前缀共享丢了而判决说保留）⇒ 打中 J1。
- 造出一个输入让改后的判法给错结局（该红不红、该绿不绿、该报本次未跑而判绿、该报错而悄悄退回）⇒ 打中 J2。
- 一条变异行锚点不唯一、替换文改的不是那一判、点名的测试跑不到或推不出会红 ⇒ 打中 J3。
- 第一轮判决的某一格去向判错（采错、漏采、「不采」理由不成立）⇒ 打中 J5。

## 六、交付

- 腿的报告写 `research/prompts/test-script-align-code-r2-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/test-script-align-code-r2-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件名加那份文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的路径与行号。
- 方案按 `crates/` 今天的实现来谈。不跑重型测试；不改主工作区；只停自己起的进程。
