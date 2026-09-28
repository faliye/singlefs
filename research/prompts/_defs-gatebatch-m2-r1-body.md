# 里程碑二收尾：门禁批（层 0 续跑接入、崩溃枚举按用例复用）一轮三方，第一轮正文（2026-09-26）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 -->

## 一、这一轮要判什么

门禁批改了门禁、研究脚本、hook 与崩溃验证员定义（`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。被判的改动是它交回时的 diff `/tmp/claude-1000/gate-batch-m2/my-changes.diff`（2543 行、8 个文件，材料员原样放进附录二），另加主 agent 同日对 `research/scripts/check-segment-registry.py`（门禁 52 号）的两处修补（数字间下划线、认不出第二条流登记句就判红）。出处与用户定案见规格 `/tmp/claude-1000/gate-batch-m2/spec.md` 与 `records/2026-09-24-里程碑二收尾调度.md` 第三节「崩溃枚举的跑法」「实六交回」两行。

| 格 | 被攻的 | 问题 |
|---|---|---|
| J1 | **54 号 `--full` 逐条跑、逐条标记、续跑**（`.claude/gate.d/54-layer0-replay.sh`） | 有没有一条路径：某条用例的输入变了而它的旧标记照样作数（假复用）；跑了一部分、被打断或判红之后留下一个作数的标记；续跑的进度目录跨输入串用；`--start-over` 没传到、或别处漏清 `SINGLEFS_LAYER0_START_OVER` |
| J2 | **按排除法算每条用例的输入**（`research/scripts/admission.py` 的崩溃枚举那一节：「别的所有测试目标」自动减掉，在别处 `.rs` 或 `Cargo.toml` 里以整词出现的不减，有 `build.rs` / `[[test]]` / autotests 的包整份不减） | 造一处改动：它改变了某条登记用例的行为，而被算成「与它无关」减掉了（宏、`include!`、`#[path]`、按目录读 `tests/`、拼接出来的名字、共用模块改名）；另看会不会减得太少，让无关改动把 2.8 天的第二条流拖去重跑 |
| J3 | **构建环境进指纹**（`build_environment_lines`；`CARGO_BUILD_JOBS` 故意不进；行名只写类别不写绝对路径） | 还有什么改了会改测试二进制的行为而不进指纹；`CARGO_BUILD_JOBS` 不进会不会让一个真有差别的构建被当成同一个；主工作区与 HEAD + 暂存区的 worktree 算出的指纹是不是一定相同 |
| J4 | **重型测试闸认崩溃枚举用例**（`.claude/hooks/lib_heavy_tests.py` 的 `classify`、`.claude/hooks/heavy-test-guard.sh`） | 子 agent 有没有一种写法绕得过去（脚本里运行时读参数、别名、`cargo nextest`、直接起测试二进制加 `--ignored` 的别的拼法）；有没有把只跑同一目标里快用例的合法命令误拒 |
| J5 | **崩溃验证员定义与相关文字**（`.claude/agents/crash-verifier.md`、`.claude/rules/implementation-workflow.md`、`stage-inputs.tsv` / `stage-owners.tsv`；门禁批没改的 `.claude/main-agent.md` 第 61 行「判绿按输入哈希写全绿标记」与 `.claude/agents/gate-triage.md` 第 29 行） | 照改后的字面，崩溃验证员在提交时做不做得对；几份文字之间有没有互相矛盾、有没有哪一处仍按「整批一格标记」写；用户原话「改了只跑改了的部分」在字面上兑现了没有 |

**共用问句**：照改后的字面与代码，哪一步会做错、放过、或误拒；给具体的命令、改动或派发情形。

## 二、实现今天的样子（主 agent 的观测，2026-09-26）

- 登记的四条崩溃枚举用例（`python3 research/scripts/admission.py crash-cases .` 原样）：`layer0-first-stream`（`first_transaction_step_seven_layer0`）、`layer0-second-stream`（`second_transaction_step_zero_layer0`）、`floor-raise-pushed-by-the-session`、`c561-sigma-full`，都在 `singlefs-harness` 包里。
- 门禁批自报：`admission.py --selftest` 140 格、54 号自证 13 格加 10 个变异全抓、`lib_heavy_tests.py --selftest` 55 种加 6 个变异、`heavy-test-guard.sh --selftest` 580 种；62、63、doc-lint、gate-lint、shell-lint 全绿；47 号在主 agent 修了 52 号脚本之后退 0。
- 这一轮不碰 `crates/`（实七在改，与这一轮无关）；开工快照 `research/prompts/defs-gatebatch-m2-r1-snapshot/sha256sums.txt`（11 个文件）。

## 三、条款（材料员整段抄进附录）

- `.claude/rules/implementation-workflow.md` 全文（「重型测试只在提交时跑」「复用上一次全量门禁的判定」「门禁管哪一半」）；
- `.claude/agents/crash-verifier.md` 全文；
- 层 0 规模三轮判决 `research/prompts/m2-layer0-scale-r2-main-verification.md` 第二、三节与 `-r3-main-verification.md` 第二至四节（乙、U1–U4、用户四问）；
- `records/2026-09-24-里程碑二收尾调度.md` 第三节「崩溃枚举的跑法」那一行（用户原话）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | J5，兼核 J1 的字面 | 逐条核改后的字面是不是规格与用户原话要的，几份文字之间对不对得上 |
| 云端攻方（Opus） | J1、J2、J3、J4 | 造改动、命令与派发情形攻改后的代码；只在临时拷贝上跑 `admission.py`、54 号的自证与 hook 的自证，喂 hook 只喂 JSON 看退出码，被判的重型命令一条都不执行 |
| 本地攻方 | J2 的清单 | 按事实表逐格核：四条用例各自保留、减去的文件数与几个抽样文件的去留，照 `admission.py crash-case-manifest` 的原样输出与规则逐条判对不对；事实表每行写来源（命令与输出行），每张表不超过 6 行 |

两条攻方腿不重叠：Opus 造情形攻语义，本地只逐格核排除清单的字面。

## 五、交付

- 腿的报告写 `research/prompts/defs-gatebatch-m2-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/defs-gatebatch-m2-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，引的行是那句原文自己所在的行。
- 不跑重型测试：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的测试目标、全量 `cargo test` 一律不跑；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知；交回之前后台不许留着跑的东西。
