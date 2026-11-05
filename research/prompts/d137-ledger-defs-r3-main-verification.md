# d137-ledger-defs-r3 判决（最终轮，2026-11-02）

被判：`research/prompts/_d137-ledger-defs-r3-body.md` 第一节——前两轮判决（`research/prompts/d137-ledger-defs-r1-main-verification.md`、`research/prompts/d137-ledger-defs-r2-main-verification.md`）落地之后的样子。腿：辩方 `research/prompts/d137-ledger-defs-r3-sonnet-output.md`（a2e05f8c），攻方 `research/prompts/d137-ledger-defs-r3-opus-output.md`（2e30226c），攻方模型 `research/prompts/d137-ledger-defs-r3-opus-model/`。被判文件：`.claude/agent-common.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`、`.claude/main-agent.md`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/rules/mutation-sampling.md`、`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/crash-test/SKILL.md`、`scripts/hosts/compile-then-swap.py`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/doc-experiments.sh`。

## 核查怎么做的

- 开工快照倒推（任务 5157）：「清单点名 19 份，倒推出 19 份，sha256 对上 19 份」。
- 引文：辩方「核了 5 处引文、96 处裸行号，对上 5 处」，攻方「核了 30 处引文、13 处裸行号，对上 30 处」；下面「采」的每一条由主 agent 在主工作区现查。
- 攻方模型（任务 5158 造世界与变异体，5161 判世界，5159、5162、5163 探针）：世界十一行 rc 依次 0、1、1、0、0、1、0、0、0、0、1，与 `out/expected.txt` 第二节第三列逐行相同，四个对照（slice 新入口红、slice 路径上绿、新入口写在前面红、围栏里注释红）对上；探针照真脚本与两个变异体的每一行与第三、四、五节逐行相同。
- 新格 judge-entries-in-sync 在真树上（任务 5149）：「13 个名字与两份判段源文件里认出的公开判段入口逐个相同（扫了 2 份、459 个函数单元…）」。

## 逐条判

| 条 | 腿 | 判 | 主 agent 现查 |
|---|---|---|---|
| 前两轮判「采」的各条落地做到了判决那一句（九栏、三种出路、三处要卡、按判段入口认、续行与注释、特性取并集、读不出与不是成员都照跑、按小节点名） | 辩方 | 站得住 | 辩方逐条引文与快照 19 份对上 |
| F1：`mutation-triage.md` 第 32、41 行与 `mutation-sampling.md` 第 79 行还写「清场没过没跑」一栏，暂存区那一版 `crates-mutation-rows.py` 已不打这一行（与拆分提案一百八十节 ① 同一处） | 辩方 | 采 | `git show :scripts/mutation/crates-mutation-rows.py \| grep -c 清场没过没跑` 得 0 |
| F2：E171 两份装置源文件头没有 `// task-server-gpu-arguments:` | 辩方 | 采，归做 E171 重跑的会话 | 它 2026-11-02 回已派实现员补；在补上之前那两份裸交服务退 78（`gpu_card_configuration.rs` 没清场时退 78 那一支），不静默自己挑卡 |
| F3：`verification.md` 第 35 行仍写「特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起」，与同一份第 63 行相反 | 辩方 | 采 | 那一句此刻在 |
| F4：`mutation-triage.md:32` 与 `mutation-sampling.md:79` 有大段相同文字 | 辩方 | 不采 | 两处说同一件事、一份是定义一份是规则，F1 改完两处一起改；门禁不判跨文件抄段，零轮记下 |
| F5：`judge_not_run_without_the_gpu_feature` 是公开函数而不在入口表 | 辩方 | 不采 | 任务 5149 judge-entries-in-sync 按「带判段去路记号」认，判它不是判段入口；辩方也写明没找到装置直接调它 |
| C1：judge-entries-in-sync 切函数单元从上一个分号算表头，签名里带 `[u8; N]` 的函数不进单元——新入口漏报、路径上的私有函数断边误报 | 攻方 | 采 | 世界 sync-array-parameter-new-entry rc 0、sync-array-parameter-on-the-path rc 1，两个对照对上 |
| C2：环上截断时「走不到」被记住，新入口与表里入口互相调用、写在后面时漏报 | 攻方 | 采 | 世界 sync-cycle-new-entry-after rc 0，对照 rc 1 |
| C3：preregistration-judge-features 行尾注释只在围栏里截，列表项与表格整格里的注释里写的 `--features gpu` 被算作带了 | 攻方 | 采 | 两个世界 rc 0，围栏对照 rc 1 |
| C4：`compile-then-swap.py` 23 项自证没有一项给 `--release`，带 `--release` 时丢特性、不交 `--release` 两个变异体自证照样全过 | 攻方 | 采 | 探针 5162、5163 与期望逐行相同，两次 `selftest_in_process rc=0` |
| C5：执行员第 4c 步 `task log <号> --all` 只给最后一次尝试，重排或续跑之后前几次的 NOT_RUN 数不到 | 攻方 | 采 | `task-server/task` 第 25 行「打那一次尝试（默认最后一次）的日志」，`--attempt <第几次>` 逐次取 |
| C6：定义与门禁没有要求新入库装置写文件头要卡声明；`scripts/experiment-apparatus/` 那一半服务认不到（那棵树里没有 `src/bin/`） | 攻方 | 采前一半；后一半交做统一任务服务的会话 | 设计员与执行员定义补「走判段的入库装置在源文件头写 `// task-server-gpu-arguments:`」 |
| C7：`tooling-writer.md` 点名 path-moves「怎么做」，那一节第 2 步是 `git mv`，共用约束第 61 行不许子 agent 做 git 写操作 | 攻方 | 采 | 两处原文此刻在；点名改成只取不含 git 写操作的那几步，`git mv` 与登记写明交主 agent |
| C8：`kb-scribe.md` 点名的 path-moves「改一个全仓术语」要求的跟改多在写回员写范围外 | 攻方 | 采 | 点名改成写回员只做 kb 里那几处、写范围外的列给主 agent |

## 交用户表（零轮）

按 `.claude/rules/three-way-inference.md`「第三轮之后停，不开第四轮」：下面这一轮的改法都没被攻过，标「零轮」，改完由提交前的整轮门禁判形式，判不了的靠 review。

| 改法 | 零轮 | 去向 |
|---|---|---|
| F1、F3、C5、C6 前一半、C7、C8 的定义与规则字句 | 零轮 | 交工具员改，提交前门禁 agent-def-adversarial-review 格认这三份判决 |
| C1、C2、C3 的门禁认法、C4 的自证 | 零轮 | 交工具员改，配样本、自证与开关，复跑对上 |
| F4（大段相同文字）、F5（公开函数不在入口表） | 零轮、不采 | 记下，不改 |
| C6 后一半（`scripts/experiment-apparatus/` 下的装置服务认不到要卡） | 交做统一任务服务的会话 | 主 agent 发消息 |

## 回看决策

不涉及决策：这一轮判的是 agent 定义、规则、skill 的字句与配套脚本，没有动 `.claude/kb/decisions/` 里的任何分项。
