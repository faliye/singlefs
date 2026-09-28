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

---

### 小节清单：`.claude/rules/implementation-workflow.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑 | 抄 | 主 agent 点名要全文（正文第三节引其中三处小节：「重型测试只在提交时跑」「复用上一次全量门禁的判定」「门禁管哪一半」） |
| （实现改动的流程：写代码 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 随（与标题合并抄，checklist-specs.py 自动把 H1 与紧邻的引言段合成一条 :1-5） |
| ## 三步，缺一步就不算做完 | 抄 | 全文范围内 |
| ## 改 agent 定义与共用约束，走同一条三步 | 抄 | 全文范围内 |
| ## 代码轮派腿之前记一份开工快照 | 抄 | 全文范围内 |
| ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判 | 抄 | 全文范围内；标题里的「：」是全角字符（U+FF1A），@标题取法不按 ASCII 冒号切分，已用 quote-kb.py 试抽验证回读一致 |
| ## 重型测试只在提交时跑 | 抄 | 全文范围内，也是正文第三节点名的小节之一 |
| ## 测试与崩溃检测优先多线程 | 抄 | 全文范围内 |

### 小节清单：`.claude/agents/crash-verifier.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 崩溃一致性验证员（crash-verifier） | 抄 | 主 agent 点名要全文 |
| （崩溃一致性验证员（crash-verifier） 标题之下、第一个下级标题之前的正文：第 11-16 行） | 抄 | 随（与标题合并抄） |
| ## 输入（主 agent 必须给） | 抄 | 全文范围内 |
| ## 做什么 | 抄 | 全文范围内 |
| ## 写范围 | 抄 | 全文范围内 |
| ## 产出 | 抄 | 全文范围内 |
| ## 没做什么（固定会有的） | 抄 | 全文范围内 |

### 小节清单：`research/prompts/m2-layer0-scale-r2-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 层 0 规模第二轮判决（m2-layer0-scale-r2，2026-09-26） | 不抄 | 主 agent 只点名第二、三节，标题未点名 |
| （层 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 主 agent 只点名第二、三节，这段开篇未点名 |
| ## 一、这一轮 | 不抄 | 主 agent 只点名第二、三节，这一节未点名 |
| ## 二、逐格判 | 抄 | 主 agent 点名第二节；用 @标题 取（等价于按行区间 12-25 取，已用 quote-kb.py 试抽核对回读一致；不用「不抄 + --extra」是因为这一行标题本身落在要取的区间里，那样标会被 C320 反向核对判红，quote-kb.py 自己的出路提示是改成「抄」） |
| ## 三、这一轮的判定（打中的先挂起，第三轮再攻） | 抄 | 主 agent 点名第三节；用 @标题 取，理由同上一行 |
| ## 四、交用户的（第三轮判完之后交） | 不抄 | 主 agent 只点名第二、三节，这一节未点名 |
| ## 五、第三轮 | 不抄 | 主 agent 只点名第二、三节，这一节未点名 |
| ## 回看决策 | 不抄 | 主 agent 只点名第二、三节，这一节未点名 |

### 小节清单：`research/prompts/m2-layer0-scale-r3-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 层 0 规模第三轮判决（m2-layer0-scale-r3，2026-09-26） | 不抄 | 主 agent 只点名第二至四节，标题未点名 |
| （层 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 主 agent 只点名第二至四节，这段开篇未点名 |
| ## 一、这一轮 | 不抄 | 主 agent 只点名第二至四节，这一节未点名 |
| ## 二、逐格判 | 抄 | 主 agent 点名第二节；用 @标题 取（等价于按行区间 12-21 取，已用 quote-kb.py 试抽核对回读一致；理由同 r2 文件同名行） |
| ## 三、定下来的 | 抄 | 主 agent 点名第三节；用 @标题 取，理由同上一行 |
| ## 四、交用户的（现在交） | 抄 | 主 agent 点名第四节；用 @标题 取，理由同上一行 |
| ## 五、到此停 | 不抄 | 主 agent 只点名第二至四节，这一节未点名 |
| ## 回看决策 | 不抄 | 主 agent 只点名第二至四节，这一节未点名 |

### 小节清单：`records/2026-09-24-里程碑二收尾调度.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 里程碑二收尾调度（2026-09-24） | 不抄 | 主 agent 只点名第三节里「崩溃枚举的跑法」那一行，标题未点名 |
| （里程碑二收尾调度（2026-09-24） 标题之下、第一个下级标题之前的正文：第 2-6 行） | 不抄 | 主 agent 只点名第三节那一行，这段开篇未点名 |
| ## 零、收尾出口（用户 2026-09-24 定的九步，次序照做） | 不抄 | 主 agent 只点名第三节那一行，这一节未点名 |
| ## 一、批次 | 不抄 | 主 agent 只点名第三节那一行，这一节未点名 |
| ## 二、交用户的 | 不抄 | 主 agent 只点名第三节那一行，这一节未点名 |
| ## 三、用户 2026-09-24 第二次定案（两批问答） | 不抄 | 分项索引表：整节是逐条决策的登记表（每行一条决策），主 agent 只点名其中「崩溃枚举的跑法（用户 2026-09-26」那一行；用 --extra 按行区间取（190-190 行，不含本标题所在行 61，不触发 C320 反向核对） |
| ## 历史版本 | 不抄 | 主 agent 只点名第三节那一行，这一节未点名 |
| ### 2026-09-24 | 不抄 | 主 agent 只点名第三节那一行，这一节未点名（历史版本节里的子节） |

### 小节清单：`.claude/main-agent.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 主 agent：任务入口与职责 | 不抄 | 正文 J5 只点名第 61 行的引句，标题未点名 |
| （主 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 正文 J5 只点名第 61 行，这段开篇未点名 |
| ## 职责 | 不抄 | 正文 J5 只点名第 61 行，这一节未点名 |
| ## 禁止 | 不抄 | 正文 J5 只点名第 61 行，这一节未点名 |
| ## 一轮怎么开、怎么收 | 不抄 | 正文 J5 只点名第 61 行，这一节未点名 |
| ## 派出去之后 | 不抄 | 正文 J5 只点名第 61 行，这一节未点名 |
| ## 交回怎么读 | 不抄 | 正文 J5 只点名第 61 行，这一节未点名 |
| ## 派发提示怎么写 | 不抄 | 正文 J5 只点名第 61 行，这一节未点名 |
| ## 什么时候派哪个 agent | 不抄 | 分项索引表：整节是按场景派 agent 的登记表（每行一个场景），正文 J5 只点名第 61 行「判绿按输入哈希写全绿标记」那一句；用 --extra 按行区间取（61-61 行，不含本标题所在行 52，不触发 C320 反向核对） |

### 小节清单：`.claude/agents/gate-triage.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁分诊（gate-triage） | 不抄 | 正文 J5 只点名第 29 行，标题未点名 |
| （门禁分诊（gate-triage） 标题之下、第一个下级标题之前的正文：第 11-16 行） | 不抄 | 正文 J5 只点名第 29 行，这段开篇未点名 |
| ## 输入（主 agent 必须给） | 不抄 | 正文 J5 只点名第 29 行，这一节未点名 |
| ## 做什么 | 不抄 | 正文 J5 只点名第 29 行；这一节是编了号的步骤列表（1-4 步），只要第 3 步（第 29 行）；用 --extra 按行区间取（29-29 行，不含本标题所在行 24，不触发 C320 反向核对） |
| ## 写范围 | 不抄 | 正文 J5 只点名第 29 行，这一节未点名 |
| ## 产出 | 不抄 | 正文 J5 只点名第 29 行，这一节未点名 |
| ## 没做什么（固定会有的） | 不抄 | 正文 J5 只点名第 29 行，这一节未点名 |

---

**出处 `.claude/rules/implementation-workflow.md:1-5`（整段抄，未转述）**

```markdown
# 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/implementation-workflow.md:6-15`（整段抄，未转述）**

```markdown
## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，正推腿核「代码做的是不是条款说的」、反推腿攻「哪一格会错」、本地腿找反例；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

```

**出处 `.claude/rules/implementation-workflow.md:16-23`（整段抄，未转述）**

```markdown
## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

```

**出处 `.claude/rules/implementation-workflow.md:24-29`（整段抄，未转述）**

```markdown
## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、派核查员之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再交核查员——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、几点、被改了几处、腿引的行落没落在那几处。

```

**出处 `.claude/rules/implementation-workflow.md:30-45`（整段抄，未转述）**

```markdown
## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

```

**出处 `.claude/rules/implementation-workflow.md:46-81`（整段抄，未转述）**

```markdown
## 重型测试只在提交时跑

**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：

- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
- 全部实验复跑：87 号。
- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标，libtest 参数带 `--ignored` 或 `--include-ignored` 的——`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制。不带这两个参数、只跑那个测试目标里快用例的不算。

只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。

| 场合 | 跑不跑 |
|---|---|
| 每次提交代码 | **必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；git 的 pre-commit hook 照旧跑整轮门禁 |
| 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
| 其余任何时候 | 不跑 |
| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员跑 54、55、57、59 号那几道；门禁分诊员跑门禁其余阶段并分诊，54、55、57、59 靠「输入没变就复用上一次全绿判定」不重跑。谁都不把整轮全量从头跑一遍 |
| 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |

由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

```

**出处 `.claude/rules/implementation-workflow.md:82-90`（整段抄，未转述）**

```markdown
## 测试与崩溃检测优先多线程

- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。

**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。

```

**出处 `.claude/agents/crash-verifier.md:1-16`（整段抄，未转述）**

```markdown
---
name: crash-verifier
description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
tools: Read, Bash
model: opus
effort: high
omitClaudeMd: true
---

# 崩溃一致性验证员（crash-verifier）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「重型测试只在提交时跑」里最重的那几道。
开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`「判读纪律」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「崩溃一致性只能靠崩溃点重放验证」「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」。

```

**出处 `.claude/agents/crash-verifier.md:17-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
- 54、55、57 号各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
- 54 号 `--full` 在哪棵 worktree 里跑：HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建。只核标记、不跑全量的，写明「54 号不带 --full」；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
- 报告路径与草稿目录。

```

**出处 `.claude/agents/crash-verifier.md:25-34`（整段抄，未转述）**

```markdown
## 做什么

1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
   1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认带 `--full`、在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`，经内存包装照第 1b 步）：它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；派发提示写明「54 号不带 --full」时才只跑快档（逐条核全绿标记），写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

```

**出处 `.claude/agents/crash-verifier.md:35-38`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件、草稿目录，与为 54 号 `--full` 建的临时 worktree（跑完删）。阶段自己用的临时目录、编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）与 54 号写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；你不改仓里任何文件。

```

**出处 `.claude/agents/crash-verifier.md:39-42`（整段抄，未转述）**

```markdown
## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行；末尾「没做什么」。

```

**出处 `.claude/agents/crash-verifier.md:43-47`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没修任何一处红，也不判红是不是这一轮的改动造成的（交主 agent 或 `gate-triage`）。
- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 54 号头部与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。

```

**出处 `research/prompts/m2-layer0-scale-r2-main-verification.md:12-25`（整段抄，未转述）**

```markdown
## 二、逐格判

| 格 | 判 | 依据 |
|---|---|---|
| M1 甲二的等价 | **打中，分辨臂：甲二不等价** | 历史 `UOOUOMSU`（A、B 之后卸载重挂、覆盖写两次、卸载重挂、覆盖写、进程重开、写 16 字节小文件、卸载重挂）txg 25 那一段 2 个原地写、16 个 COW 写，全枚举 32768 个状态被记录核对器第二条判红，COW 取 ∅ 或全集时一个都不红——甲二在整条流 334 个状态上 0 个红。打中的是**判据自己的一处漏洞**：冻结副本 `crash.rs:706`、`:714` 的复用豁免只认整份还在位的后写，跨度不对齐的两代复用（两个 16K 节点 → 一个 32K 数据单元 → 两个 16K 节点）只落一半时被当成洞；同一段 oracle 与 checker 违例都是 0。它说明甲二会把「只在 COW 部分落盘时才显出来」的判定差异整类藏起来，不管那个差异是实现的错还是判据的错 |
| M1 另三处（I-7.7、I-7.8、I-1.8） | 没打中，不构成证明 | 随机 120 条历史 51720 次判定不单调点 0 个；攻方自己算过按豁免链出现的比例这 120 条里本该只有约 0.84 条，这个 0 不是证明 |
| M1 今天 54 号的固定脚本 | 冻结形状上不受影响 | B、C、D、E、E18、H1、H2 上候选都是 0；这一形要两次卸载，实三之后固定脚本加了卸载，新形状上有没有，没量 |
| M2 R1 | 打中两格 | 一个指纹只对应一个进度文件时两条流互相作废（量过：第二条流杀前跑完的 21 / 43 片全丢）；`.cargo/config.toml`、`RUSTFLAGS`、`CARGO_PROFILE_*` 不进指纹（推的） |
| M2 R3 | 没打中 | 前提是合并前核「文件里的片 + 这次跑的片 = 总片数」，R3 字面没写这一道，照加 |
| M2 R4 | 打中（量过） | 观察者里有断言时，第一趟断言 panic、进度文件已写满，续跑那一趟观察者一个状态都没看到、结果变绿 |
| M2 R5 | 打中一格（推的） | 报续跑片数挡不住 R4 那一格；R5 没写判红的那一趟要不要删进度文件 |
| M3 乙的清单 | 打中 | 按流拆的清单会漏新加的 `tests/common.rs`（量过：与 `tests/common/mod.rs` 并存编不过，cargo 不重编时还假绿退 0）、新加的 `build.rs`、新拆出的 crate（推的）；`.cargo/config.toml` 与 `src/` 下叫 `target` 的目录（被 `.gitignore:3` 挡在输入清单外）今天整份 `crates/` 的清单也漏 |
| M4 第一轮判决复核 | 甲出局站得住（第 304 行那一类落在固定脚本第 12 段，早于回退那一步，推的，等实三之后的新形状核）；其余三处更正见第一节 | — |

```

**出处 `research/prompts/m2-layer0-scale-r2-main-verification.md:26-33`（整段抄，未转述）**

```markdown
## 三、这一轮的判定（打中的先挂起，第三轮再攻）

- **甲二被打中，挂起**（第二轮 M1 分辨臂，核查员复跑坐实；打中一轮）。甲打中两轮（第一轮攻方、第二轮辩方复核站住），出局。第三轮也打穿甲二的话，约简候选一个不剩，**层 0 全量照今天的枚举域跑**（D13（验证路线） 已定项 4 的段模型），用户定的「崩溃点不为省时间缩范围」照旧。
- **乙（按流复用）打中，挂起**：按流拆的输入清单一碰新文件就漏（M3），今天 54 号按整批输入哈希记一格全绿标记的做法照留；另把 `.cargo/`、`RUSTFLAGS`、`CARGO_PROFILE_*` 与 `CARGO_BUILD_*` 这几个环境变量补进 54 号的输入指纹，`.gitignore` 挡住的 `crates/**/target/` 这类源码目录名在清单里单独核（推的，被攻过零轮）。
- **记录核对器的复用豁免漏洞**（M1 打中的判据错）：立 C561（记录核对器的复用豁免在复用只落一半时假红）、实六修——豁免按扇区判，只把被后写覆盖到的那几个扇区当成合法复用；攻方在打中的那一段上量过 32768 个红 → 0（只在这一段上，被攻过零轮）。修完之前，若第二条流的新形状走到这一形，全量层 0 会在这里报红，那是判据的错、不是实现的错，照红照查。
- **断点续跑收严**（实六实现，被攻过零轮）：R1 的键带流的名字（一条流一个进度文件），指纹补进 `.cargo/`、`RUSTFLAGS` 这几样；R3 合并前核「文件里的片 + 这次跑的片 = 总片数」，对不上整份作废；R4 片行在观察者看完那一片之后才写，任何一片判红，这一趟的进度文件删掉、下一趟从头跑；R5 报续跑片数与强制从头跑的开关照旧。
- **L6 的 6 条没人跑的全量**：状态数照旧没量，归交用户的那一问（第四节）。

```

**出处 `research/prompts/m2-layer0-scale-r3-main-verification.md:12-21`（整段抄，未转述）**

```markdown
## 二、逐格判

| 格 | 判 | 依据 |
|---|---|---|
| N1 甲二（辩方） | **辩住了那一格**：第二轮打中甲二的只是 C561（记录核对器的复用豁免在复用只落一半时假红） 那一处判据错。按扇区豁免打进 `crash.rs` 的副本重编之后，M1 那一段 262144 个状态 32768 → 0、甲二红 0；两条流（41 写 / 10 段、425 写 / 55 段）、270 条随机历史（种子 11 与 53）、7 条回收窗口置 0 的历史上甲二与全量判得一样，真错误那 7 条的红数与补丁前逐字相同。没证明「只在 COW 部分落盘时显出来」这一类没有别的成员（没给 I-7.7、I-7.8、I-1.8 造定向变异） | 辩方报告，核查员复跑 |
| N2 ① 按扇区豁免（攻方） | **打中（量过），形态是「判别子观测不到」**：M1 那一形里「只落 y」的状态 X（今天判红，是假红）与「B 的节点 u2 和盖住它的数据单元 l 都没落」的状态 Y（C507 那一格的洞）崩溃镜像逐字节相同（21 条历史 21 条，差异扇区 0）。记录核对器按 D13（验证路线） 已定项 7 只拿三样入参，判不出两者的差别，所以「修掉 M1 假红」与「C507 那一格仍红」在三样入参下互相矛盾。另量到今天的判法自己有一处假绿：l 没落、y 只盖住它一半，今天把 l 整份开脱、整份层 0 判绿（21/21），七种按扇区判法都判它缺席。C513（复用豁免不判那次复用合不合法） 那一道：按扇区判的字面不带回收谓词，那三个状态上核对器不红，但同状态上 checker 另有 6–7 条违例，整份层 0 照样红 | 攻方报告，核查员复跑 |
| N2 ② 续跑 R1–R5（攻方） | 没找到新的「只跑一部分却报全量」读法（推的，续跑没实现）；四处规格缺口：键「指纹 + 流名」加片头认不出同流同状态、只换了版本表的两次枚举（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 第 456 行与第 526 行就是这样一对）；原型读回一行时缺的字段取 0；收严后的 R4 丢了第一轮「观察者累计进进度文件」那一半，第一条流的全量用例续跑后必红（第 466 行）；后两格叠起来会真的假绿 | 攻方报告第二节 |
| N2 ③ 54 号输入指纹（攻方） | **打中（量过 7 个场景）**：`cargo -V && rustc -V` 的哈希不变而测试二进制行为变了的：仓根上层目录的 `.cargo/config.toml`（54 号 `--full` 的 worktree 建在 `mktemp -d` 下，上一层是 `/tmp`）、`CARGO_HOME` 下的 `config.toml`、`CARGO_TARGET_<三元组>_RUSTFLAGS`、`CARGO_TARGET_<三元组>_RUNNER`、`RUSTC_WORKSPACE_WRAPPER`、`RUSTC_WRAPPER`（空 target 目录上）、`RUSTC`（指纹跑的是 PATH 里的 `rustc`）；推的一格：`crates/` 下指向目录的符号链接被 `research/scripts/admission.py` 第 276 行的 `isfile` 滤掉 | 攻方报告，核查员复跑 |
| N3 代价数（本地） | 两份样本一致：5575606380 与 5575802973 各自按闭式算得对；速率 4194320 ÷ 302.2 ≈ 13880；32 线程 2.3 天是线性外推；210 是闭式算的、没跑枚举器；32768 ÷ 262144 = 1/8。更正见第一节 | 本地两份样本 |

```

**出处 `research/prompts/m2-layer0-scale-r3-main-verification.md:22-35`（整段抄，未转述）**

```markdown
## 三、定下来的

- **甲二**：第二轮打中一次，这一轮辩住（打中的只是判据错）。三轮里没有多数打穿，它不出局；但它只取 COW 全落或全不落两点，部分落盘才显出来的问题它整类看不见，只能在 C561 修好之后当平时快档，替不了全量——用户定的「崩溃点不为省时间缩范围」下，提交时照旧全量。当不当快档交用户（第四节）。
- **乙（按流复用）**：第二轮打中，这一轮没人替它辩，按两轮里一轮打穿、另一轮没攻计，维持挂起、不采纳；54 号照旧按整批输入哈希记一格全绿标记。
- **C561 的修法**：按扇区判这一条被打中，判据本身在三样入参下矛盾，交用户定两条出路（第四节）。
- **实六的规格**（零轮，最终代码三方攻）：
  - U1 续跑的键加版本表的哈希（或整份枚举计划的哈希），认得出同流同状态、换了版本表的两次枚举；
  - U2 读回进度行缺任何一个字段整份作废，不取 0；
  - U3 观察者累计的状态进进度文件（第一轮 R4 那一半），续跑之后观察者看到的状态数 = 闭式，否则判红；
  - U4 54 号输入指纹补：仓根往上各层与 `CARGO_HOME` 下的 cargo 配置、`CARGO_TARGET_*_RUSTFLAGS`、`CARGO_TARGET_*_RUNNER`、`RUSTC_WRAPPER`、`RUSTC_WORKSPACE_WRAPPER`、`RUSTC`（取 `$RUSTC -V`），外加第二轮判决第三节那几样；`admission.py` 对 `crates/` 下的符号链接照链接指向的内容计入，不滤掉；
  - U5 记录核对器照第四节用户定的那条出路改，今天那一处假绿（l 没落、y 只盖一半被整份开脱）照改后的判法各一条用例钉住；
  - U6 辩方的 `rerun.sh` 默认参数跑不通（要 `-p4`）：冻结证据不改，判决里记下这一格，以后腿交回的 `rerun.sh` 由核查员照默认参数跑一次。
- **kb 腐烂**：C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑） 那一行引的 `crates/mutations.tsv:306-308` 今天在第 268–270 行，归第 6 步腐烂治理。

```

**出处 `research/prompts/m2-layer0-scale-r3-main-verification.md:36-50`（整段抄，未转述）**

```markdown
## 四、交用户的（现在交）

| 问 | 选项 | 代价（写明量过还是推的） |
|---|---|---|
| C561 怎么修 | 甲：按扇区判，只看盘上字节，解释扇区的后写要过回收谓词；D13（验证路线） 已定项 7 不改；内容恰好相同、盘上看不出来的洞（状态 Y）判不红。乙：把枚举器的持久集合当第四样入参交给记录核对器（攻方的 `SecPers513`），改 D13（验证路线） 已定项 7 的入参那一句 | 乙在攻方的四格上都对（量过，副本）；甲在状态 Y 上不红（量过）。乙之下「崩溃后镜像逐字节相同的状态判定必然相同」对记录核对器不再成立，按镜像去重的提速对它失效（推的） |
| 甲二当不当平时快档 | 当（C561 修好之后，平时跑甲二、提交时照旧全量加续跑）/ 不当（平时只跑今天的快档） | 甲二在冻结形状上 210 个状态（推的，闭式）；它看不见部分落盘才显出来的一类（第三节） |
| 等价提速要不要先量 | 实六之后跑一次剖析（重型，要跑名字带 `layer0` 的用例），量每个状态的时间花在哪，再定做不做增量枚举、按镜像去重 / 不做，照今天的写法跑 | 全量第二条流：冻结形状 5575606380 个状态（副本上的重建），32 线程约 2.3 天（推的，线性外推）；新形状（加卸载、挂着时回退）没量 |
| 另外 6 条标了 `#[ignore]` 的层 0 全量 | 提交时一并跑 / 记欠账、这一次不跑 | 状态数没量（要跑名字带 `layer0` 的用例取段序列，重型） |

**用户 2026-09-26 弹窗定**（选项原文照上表）：
- C561 取乙：持久集合当第四样入参交给记录核对器，改 D13（验证路线） 已定项 7 的入参那一句。用户另说（原话）：「乙，可以没有问题。 checker这里加参数不存在问题。并且checker的算法优化可以优先于core进行。core不优化算法，checker可以。」
- 甲二当平时快档，C561 修好之后（实六做）；提交时照旧全量加续跑。
- 等价提速先量：实六之后跑一次剖析（用户许可的重型测试，主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 跑）。用户原话：「可以跑一次刨析，为我们下个里程碑3的优化做准备，里程碑3放量这里会考虑多机分片核cpu核对。」
- 另外 6 条 `#[ignore]` 的层 0 全量：记欠账，这一次不跑。

```

**出处 `records/2026-09-24-里程碑二收尾调度.md:190-190`（整段抄，未转述）**

```markdown
| 崩溃枚举的跑法（用户 2026-09-26） | 用户原话：「这次跑可以 下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」起因：实五按主 agent 规格加了一条在会话推的抬 F 那一串上做小枚举的用例（`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`，20,493 个状态，debug 下单跑约 12 分钟、约 16 核），它是这条新写路径唯一的崩溃证据，层 0 两条流在 4 GiB 盘上碰不到准入被拒（推的）。**定**：① 这一次照跑；实五证红做完后给它加 `#[ignore]`，写明提交时由崩溃验证员按输入哈希跑（release）。② 门禁批（实六之后）把它接进提交时崩溃验证员那一组，在 `.claude/gate.d/stage-inputs.tsv` 登记它读的输入，输入没变复用上一次全绿判定。③ 以后崩溃枚举一律按用例 / 按流各自登记输入、各自复用，改了哪块只重跑读它的那几条——就是层 0 规模第二轮挂起的乙；第二轮打中「按流列的清单漏新加的共用文件」，门禁批照当时的修法做：清单用排除法写（整个 `crates/` 减去别的流自己的用例文件）、指纹补第三轮 U4 那几样；被攻过零轮，改的是门禁与崩溃验证员定义，照「改 agent 定义与共用约束，走同一条三步」走一轮三方 |
```

**出处 `.claude/main-agent.md:61-61`（整段抄，未转述）**

```markdown
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
```

**出处 `.claude/agents/gate-triage.md:29-29`（整段抄，未转述）**

```markdown
3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；✗ 行写着内存包装退 25x 的（15、74 号的「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」）不按它点名的文件判：250 ⇒「环境（上限）」，252 与 254 ⇒「并发」，251 与 253 ⇒「环境」，原样抄阶段的 → 与它上面包装自己打的 ✗、→；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
```
