# 里程碑二收尾：定义改动一轮三方，第三轮正文（2026-09-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 H1 H2 H3 H4 H5 H6 H7 H8 H9 K1 K2 K3 -->

## 一、这一轮要判什么

第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停）。第二轮判决 `research/prompts/defs-m2-closeout-r2-main-verification.md` 整份进材料；它第三节的改法 G1–G9 已由修定义的 agent 改进文件（被攻过零轮），这一轮只攻 G1–G9 的改后字面，攻击面不重复前两轮。第三轮里新冒出来的改法不再为它开一轮，写进判决的交用户表、标「零轮」。

| 格 | 被攻的 | 问题 |
|---|---|---|
| K1 | **G1**（撤回整条包 `gate.sh`；15、74 号在阶段里面经内存包装，上限变量 `GATE_RESEARCH_BUILD_MEMORY_MAX`、`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`，没设取 8G）与 **G9**（崩溃验证员 54、57 号的上限每道一个数） | 阶段里面包之后，包装自己的退出码 250–254 与阶段判红分得开吗；实现员、崩溃验证员、门禁分诊员照各自定义跑 15、74 号时各是什么结局；15 号的 8G 没量过，峰值表里没有 research 整工作区 cargo test 的行——撞了是什么样子；`.claude/gate.d/stage-inputs.tsv` 的 74 号那一行主 agent 已补上包装脚本，15 号有没有同样的缺口 |
| K2 | **G2、G5、G7**（执行员：产物一个不删、`replay.sh` 改指新文件；判决行点名的对象；读实验页或索引的十道阶段挪到写完实验页之后） | G7 的十道（27、34、40、69、75、84、85、86、88、99）是修定义的 agent 照「凡是读实验页的」现查扩出来的，比判决括注里的三道多：扩得对不对、有没有漏一道、挪了之后第 6 步还剩什么；G2 与 40 号跳过 `*.r[0-9].out` 的命名规则怎么交叉；G5 的「名字表示违例 / 不匹配 / 歧义 / 失败」在今天全部判决行里各落哪一边 |
| K3 | **G3、G4、G6、G8**（攻方原型的流只许自己造、一轮合计 10⁷；弹窗闸收严；退出码文件带批号件号、数件数；本地辩方样本前缀） | 照字面还有没有绕得过去的写法；G4 收严之后有没有把一个带出处的合法断言误拒；G6 的数件数在检出 hook 的出路里没有写（`bash-command-detector.sh` ④ 的出路只写 `a & b & wait`），照 hook 的出路做会不会又回到吞退出码 |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-09-26）

- 被判的改动：修定义的 agent 报告 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md` 与它的 `my-changes-final.diff`（258 行，比的是开工时 `cp -p` 的备份）；材料员把 diff 原样放进附录二，把报告的逐条表、探针与门禁两节抄进附录。开工快照 `research/prompts/defs-m2-closeout-r3-snapshot/sha256sums.txt`。
- 这一轮被改的文件：`.claude/agents/three-way-attack.md`、`three-way-local-defense.md`、`experiment-runner.md`、`crash-verifier.md`、`gate-triage.md`、`implementation-writer.md`，`.claude/agent-common.md`，`.claude/main-agent.md`，`.claude/hooks/ask-user-claim-guard.sh`（自证 30 → 36 格），`.claude/gate.d/74-model-differential.sh`、`.claude/gate.d/15-research-build.sh`（阶段里的 cargo 经包装），另有主 agent 补的 `.claude/gate.d/stage-inputs.tsv` 74 号那一行（加 `research/scripts/run-with-memory-cap.sh` 与 `research/scripts/capped.sh`）。
- 两道阶段在极小的假根上跑过三种结局（默认 8G 绿；200M 上限下分配 400 MiB 包装退 250、走新的红出路；测试 panic 走原来的红出路），251–254 没造出来；15 号没有样本、也没有自证。
- 这一轮不碰 `crates/`。

## 三、条款（材料员整段抄进附录）

- 第二轮背景材料 `research/prompts/_defs-m2-closeout-r2-background.md` 的条款照旧；
- 第二轮判决整份；
- `research/scripts/run-with-memory-cap.sh` 文件头（退出码 250–254 的含义、嵌套那一句）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K2 | 逐条核 G2、G5、G7 改后的字面是不是第二轮判决第三节要的：G7 扩成十道扩得有没有依据、有没有做过头 |
| 云端攻方（Opus） | K1、K2、K3 | 造派发情形与命令攻改后的字面；要喂 hook 的只喂 JSON、只看退出码，被判的命令一条都不执行 |
| 本地攻方 | K2 的 G7 | 按事实表逐格填：十道阶段各自读不读实验页或索引（修定义的 agent 报告第一节表下的逐道 grep 行号）、登记给不给执行员（`.claude/gate.d/stage-owners.tsv`）；每行写来源文件与行号，每张表不超过 6 行 |

两条攻方腿不重叠：Opus 造情形攻语义，本地只逐格核 G7 那张名单的字面依据。

## 五、交付

- 腿的报告写 `research/prompts/defs-m2-closeout-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/defs-m2-closeout-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，引的行是那句原文自己所在的行。
- 不跑重型测试；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知，不用 `true` 或 `sleep` 空转；交回之前后台不许留着跑的东西。


### 小节清单：`.claude/rules/implementation-workflow.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑 | 抄 | 本文件是这一轮附录二 diff 范围内被改的定义文件，先确立它的项目本地权威与适用范围（与下一行的引言段合并抽取） |
| （实现改动的流程：写代码 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间，checklist-specs.py 自动处理） |
| ## 三步，缺一步就不算做完 | 抄 | 「改 agent 定义与共用约束」一节题述「走同一条三步」，需带着这条三步表才能核 K1/K2/K3 涉及的流程合规性 |
| ## 改 agent 定义与共用约束，走同一条三步 | 抄 | 这一轮就是在执行这一节定义的三步（第 2 步三方对抗、第 3 步 checker），是这一轮的元流程依据 |
| ## 代码轮派腿之前记一份开工快照 | 抄 | body 末尾「快照」一节直接对应这里的开工快照机制 |
| ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判 | 抄 | K1 问 15、74 号 `stage-inputs.tsv` 缺口，此节定义阶段复用判定必须按 `stage-inputs.tsv` 逐路径核对 |
| ## 重型测试只在提交时跑 | 抄 | K1 直接问「实现员、崩溃验证员、门禁分诊员照各自定义跑 15、74 号时各是什么结局」，此节的表逐条定义了这三者各自的职责边界 |
| ## 测试与崩溃检测优先多线程 | 不抄 | 与这一轮 K1/K2/K3 三格问题（内存包装分界、执行员产物删留、弹窗闸与退出码计数、G7 名单扩法）无关，这一轮不碰 crates/ 测试代码 |

### 小节清单：`.claude/singlefs-ai-sop/rules/command-safety.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 进程与命令规范 | 不抄 | 只是章节范围声明，不直接回答 K1/K2/K3 |
| （进程与命令规范 标题之下、第一个下级标题之前的正文：第 3-10 行） | 不抄 | 同上，列的是本文件哪几条是会红的检查，不涉及这一轮的具体判据 |
| ## 退不回去的操作，动手前先想一遍 | 不抄 | 这一轮不做破坏性操作，与 K1/K2/K3 无关 |
| ## `pkill -f` / `killall` 一律禁用 | 不抄 | 与 K1/K2/K3 无关 |
| ## 起了后台任务和子 agent，就要定时复检，不强制结束 | 不抄 | 讲后台任务监控与处置分离，不是 G6 的退出码计数机制本身 |
| ## 一个脚本里的检测项，能并行就并行 | 不抄 | 讲要不要并行的判据，不是 G6 的数件数机制 |
| ## 并行不许把失败吃掉 | 抄 | 是 G6（退出码文件带批号件号、数件数）判据的来源之一：「每项把退出码落进自己的文件…按固定顺序逐个读」「派出去多少项，就要收回来多少项」 |
| ## 别用 echo 假装命令成功 | 不抄 | 与 K1/K2/K3 无关 |
| ## 结果抓取要有完整性闸 | 抄 | 「总共发了多少条，抓取方比对条数，对不上就整轮作废」是 G6「数件数」判据的另一半来源 |
| ## 带闸的脚本，判完了才交出产物 | 不抄 | 与这一轮无关 |
| ## 子 shell 里的赋值传不回父进程 | 不抄 | 与这一轮无关 |
| ## 脚本改文件之后要回读确认，警告是免费的信号 | 不抄 | 与这一轮无关 |
| ## 进程边界上的三种静默失效 | 不抄 | 与这一轮无关 |
| ## 管道里的退出码不是你想要的那个 | 不抄 | G6 问的是 `&` / `wait` 场景，不是管道 |
| ## 测试镜像一律放临时目录 | 不抄 | 与这一轮无关 |
| ## 破坏性操作先看清楚再动 | 不抄 | 与这一轮无关 |

### 小节清单：`.claude/singlefs-ai-sop/rules/rules-discipline.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 规则文件铁律 | 不抄 | 只是范围声明，不直接回答 K1/K2/K3 |
| （规则文件铁律 标题之下、第一个下级标题之前的正文：第 3-8 行） | 不抄 | 同上 |
| ## 1. 正文只写四样 | 抄 | 这一轮改的 `agent-common.md`、`main-agent.md`、`agents/*.md` 都受它管（`implementation-workflow.md` 明文点出 rules-lint 扫这三处），是判断 G 系列改法字面写得对不对的通用判据 |
| ## 2. 判据留下，论证搬走 | 不抄 | 与 K1/K2/K3 的具体判据无直接关系 |
| ## 3. 搬出来的东西直接删掉 | 不抄 | 同上 |
| ## 4. 指向历史的链接要带劝阻句 | 不抄 | 同上 |
| ## 5. 规则文件不留历史节 | 不抄 | 同上 |
| ## 6. 立一条新规则之前，先问它能不能变成会失败的检查 | 不抄 | 同上 |
| ## 7. 正文不许用位置指代，也不许自称 | 抄 | G2/G5/G7 都是改写 agent 定义里具体步骤的措辞，此节判定这些改写有没有引入位置指代或自称 |
| ## 8. 正文不写某一个使用者的东西 | 不抄 | 这几份被改的文件本来就是给 singlefs 项目自己用的，不是共享规范正文 |
| ## 门禁管哪一半 | 不抄 | 讲这条规则本身由哪个脚本机检、哪半靠人，不是这一轮的判据来源 |

### 小节清单：`.claude/rules/three-way-inference.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 推论要三方独立论证 | 抄 | 与引言段合并，确立这一轮走三方推论的项目本地依据 |
| （推论要三方独立论证 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间） |
| ## 适用范围 | 不抄 | 定义「推论」的射程，这一轮已经在执行三方，不需要重新判定该不该走三方 |
| ## 各条腿必须互不重复 | 抄 | body 分工节「两条攻方腿不重叠」直接落实此节判据；本节也含「本地腿只问能落成数、能逐格判的题」——K2 派本地攻方逐格填 G7 事实表正是这一句 |
| ## 引 kb 里的条目要整行抄，不许摘句——三处都管 | 不抄 | 管材料员自己抽取材料的纪律，不是这一轮 K1/K2/K3 要判的问题 |
| ## 判决由主 agent 做，不由投票做 | 不抄 | 管主 agent 判决环节；背景材料是发给腿的，不是发给主 agent 判决用的 |
| ## 多轮：一次打穿不算数，三轮里多数打穿才算 | 抄 | body 第一段明文引用此节「第三轮之后停」 |
| ## 一条腿只抽一次样不算一次观测——否定结论尤其不算 | 抄 | 被「各条腿必须互不重复」节引用（本地腿按此节抽样），K2 的本地攻方"没打中"判定要照此节的门槛 |
| ## 云端腿的报告要分段落盘 | 抄 | body 五「交付」一节「分段落盘」的要求源自此节，云端正推与攻方两条腿都要照办 |
| ## 本地腿缺席时必须显式报告 | 不抄 | 管本地腿缺席时的应急流程，不是这一轮 K1/K2/K3 要判的问题 |
| ## 给本地腿的提示一律用英文 | 抄 | 这一轮要派本地攻方（K2 的 G7），提示必须用英文写，此节是唯一权威规定，含四类字词损坏签名 |


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

**出处 `.claude/rules/implementation-workflow.md:46-80`（整段抄，未转述）**

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

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:99-131`（整段抄，未转述）**

```markdown
## 并行不许把失败吃掉

**不带参数的 `wait` 退出码恒为 0。** 后台那一批里红了几个，它一个字都不说。


| 写法 | 父进程看到的 |
|---|---|
| `for …; do 检查 & done; wait` | **0** |
| 后台体里写 `\|\| bad=1`，父进程读 `$bad` | **0**。后台是子 shell，赋值传不回来，就是「子 shell 里的赋值传不回父进程」那一条 |
| 起的时候记 `pids+=($!)`，收的时候逐个 `wait "$pid"` | **7** |
| 每项把退出码写进自己的文件，收的时候逐个读 | 读得到 |

所以收束只有两种写法：逐个 `wait "$pid"` 取退出码，或者每项把退出码落进自己的文件、
收的时候按固定顺序逐个读。后一种的写法：并行的每一项各写一个退出码文件，
收的时候按派活时那张表的顺序逐个判。

`scripts/shell-lint.sh` 的 S6 判这一条：命令位置上不带参数的 `wait` 判红。
退出码确实在别处收了的，在那一行写 `# shell-lint:exit-collected <怎么收的>`，理由不许省——
跟 `.claude/abbreviations` 与 `.claude/naming-lint-exclude` 同规矩，要放行就把退出码的去向写出来。

**输出不许直接往 stdout 写。** 两个后台作业同时打印，超过管道缓冲区的那些行会互相切断，
`✗` 和跟在它后面的出路会被拆开，而 `sop-first.md` 要的就是每条拒绝都带下一步。
顺序还每跑一次变一个样，同一份输入两次跑输出不同，谁也说不清哪一版是真的。
所以每项写自己的文件，收的时候按派活的顺序回读。

**派出去多少项，就要收回来多少项。** 并行之后少跑一项是不报错的：那一项的文件不存在，
循环少转一圈，末尾照样报绿。收的时候数一遍，与派出去的条数对不上就整道红
（「结果抓取要有完整性闸」那一条管的是同一件事）。

**改成并行之后要重新证明它红得出来。** 并行化本身会让一个原本会红的检查变绿——
不带参数的 `wait`、后台体里写 `|| bad=1` 这两种写法就是这么变绿的。照 `show-me-test.md` 做：拿一个必红的输入喂进去，
看并行版本还红不红。没重新证过的并行化，等于把这道检查关掉了。

```

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:140-149`（整段抄，未转述）**

```markdown
## 结果抓取要有完整性闸

被测程序打印结果，外面的脚本抓回来——**这一路上丢一行是不会有任何提示的**。
输出里混进别的东西（别的进程写进来的字、转义序列、没换行的前一句），
用 `grep '^ANCHOR'` 按行首去抓，行首被顶掉的那一行就悄悄漏了。
外面看到的是「少了一项」，不是「出错了」。

**做法**：被测程序在最后一行报出它总共发了多少条，抓取方比对条数，对不上就整轮作废。
这道闸自己也要先证明会红：喂一个声称发 N 条、实际只发 N−1 条的假程序，它必须判红。

```

**出处 `.claude/singlefs-ai-sop/rules/rules-discipline.md:9-20`（整段抄，未转述）**

```markdown
## 1. 正文只写四样

| 写 | 不写 |
|---|---|
| 怎么做：步骤、次序、判据、阈值 | 为什么这么定：论证、取舍、推导 |
| 做什么、不做什么：射程与例外 | 这条怎么来的：实测、日期、谁定的、踩过的坑 |
| 推荐怎么做：拿不准时的默认选项 | 当时是什么样：旧值、旧做法、改动经过 |
| 什么规则在哪里：指到别的规则、脚本或 kb | 门禁判别力的经过与数 |

规则会被整篇读进每一轮工作的上下文。论证与经过挤在里面，执行的人和模型要先分辨
哪一句是命令、哪一句是背景，分辨错了就照着背景办事。

```

**出处 `.claude/singlefs-ai-sop/rules/rules-discipline.md:60-79`（整段抄，未转述）**

```markdown
## 7. 正文不许用位置指代，也不许自称

指文档里位置的说法一律不写：「上一节」「下面那张表」「前面提到」「上述」「如下：」「同上」「见下节」，
以及「本条」「本节」「本表」「本文档」这类自称。

把位置写成名字：小节写成它的标题，表写成它判的那件事，别处的事实链到它所在的文件。

射程是 `rules/*.md`、`CLAUDE.md`、`agents/*.md`、`skills/*/SKILL.md`。
位置指代的判据与 `kb/*.md` 同一套，逐词的射程以 `kb-discipline.md` 第 1 条为准。

自称只判指文档结构的那几个词：「本节」「本章节」「本表」「本文档」「本条」。
「这个实验」「这条决策」「本实验」这类领域对象词**不判**——规范文本讲的就是怎么处理一条决策、
一个实验，那里的「这个实验」指的是工作对象，不是文档位置。
时间指代（「本轮」「这一轮」）也不判：规则对每一轮都生效，那里的「本轮」是泛指。
这两样在 `kb/*.md` 里照判。

规则文件带 `<!-- doc-lint:rule-definition -->` 牌时，反引号与「」里的举例不判，正文照判。

由 `scripts/doc-lint.sh` 强制。

```

**出处 `.claude/rules/three-way-inference.md:1-5`（整段抄，未转述）**

```markdown
# 推论要三方独立论证

**这是 singlefs 的项目本地规则，而且只在本机生效——不进上游 SOP。**
共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/three-way-inference.md:19-37`（整段抄，未转述）**

```markdown
## 各条腿必须互不重复

「三方」是这套流程沿用的名字；一轮派三条推论腿：云端攻方（Opus）、云端正推或云端辩方（Sonnet，一轮一条）、本地攻方或本地辩方（一轮一条，派这一轮缺的那一侧）。

| 腿 | 用什么 | 怎么调 |
|---|---|---|
| 本地攻方 | Qwen3-Next-80B-A3B-Thinking-AWQ-4bit，经 `~/code/ai-center` 网关（`:8200`，模型 id `local`） | `bash research/scripts/ask-local.sh <提示文件>`，**提示用英文写** |
| 本地辩方 | 同一个本地模型，另一份提示 | 同上 |
| 云端 A | Sonnet | Agent 工具，`model: sonnet` |
| 云端 B | Opus | Agent 工具，`model: opus` |

**「不能重复」指的是各条腿要拿到不同的切入角，不是只换个模型名。**
每条腿要被指定一个不同的立场（例如：正推 / 反推 / 找反例），
对齐 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 的三步。
这一轮派本地攻方时，它与云端攻方同为攻方，**两条攻方腿的攻击面要在正文的分工表里分开写**；写不出两组不重叠的攻击面，这一轮的正文就还没写完。
本地辩方替被判出局或被攻的一方辩护。本地腿按「一条腿只抽一次样不算一次观测」那一节抽样。

**本地腿只问能落成数、能逐格判的题。** 提示里给写死的事实表（每行在核对表里写来源文件与行号），要模型按表格逐格填，不许只答 yes / no。

```

**出处 `.claude/rules/three-way-inference.md:129-146`（整段抄，未转述）**

```markdown
## 多轮：一次打穿不算数，三轮里多数打穿才算

一轮攻击打中的东西先挂起，不直接写进正文；同一个结论再攻两轮，三轮里多数打穿才算打穿。三轮各换一组攻击面，提示里明令不许重复前几轮攻过的角度；其中至少一轮派一条辩方腿去复核前一轮的判决——判它够不够得着、是不是同样打中所有替代方案。攻击腿撤回自己上一轮给的方案，比它打中新东西更有价值。

**第三轮之后停，不开第四轮。** 腿提的收严、主 agent 判决里推的组合都算被攻过零轮；第三轮只攻前两轮站住的形态，这一轮里新冒出来的零轮形态不再为它开一轮，写进判决的交用户表、标「零轮」，要么登记实验量代价，要么另立一题。

**核查员按轮派。** 这一轮有腿交了模型、产物或复跑命令，就派 `three-way-verifier`；只有辩方复核、没有新产物的轮可以不派，判决里写明没派、为什么。

实测有效的三轮分工：第一轮过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱；第二轮第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方；第三轮「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。

攻击结果要主 agent 逐条现查再采纳，不照单全收。

**结论从一边翻到另一边时（两边互调），要三条互不共享前提的验证路径。** 一次算术复核不够，翻回来可能只是换了个方向错；三条路径共用同一个前提时，逐格相等也不构成证据。

**岔路交用户定之前，每条路要有代价数。** 交岔路之前先问「每条路的代价我有数吗」，没有就建一个计数模型实验（纯算术、钉绝对值断言、变异表、进 `replay.sh`，形态照 E109（位置权威三臂的运行时代价）、E110（条带表在连续发布下的期望写放大））跑完再交。模型要在两条臂真的不同的取样点上取样。

**交岔路时写岔路单，派实验时带上它；每段交回对着岔路单判够不够。** 判决里要交用户的岔路另写一份 `research/prompts/<轮>-forks.md`，每条岔路一行，四列：候选（各自的定义）、翻面观测（量出什么数会让选择从一个候选换到另一个）、够判条件（量到哪一步这条岔路就能交用户定）、状态（开着 / 够判 / 用户已定）。岔路单只写问题与候选定义，不写倾向、不写已有的数。可以原样发给实验设计员：「设计时不看已有结论」挡的是判决里的结论，挡不到岔路单。实验每段交回，主 agent 逐行更新状态；一行开着的都不剩就停，交岔路表，不再续派。

```

**出处 `.claude/rules/three-way-inference.md:147-162`（整段抄，未转述）**

```markdown
## 一条腿只抽一次样不算一次观测——否定结论尤其不算

模型的答复是**有变化的观测**，因此 `.claude/singlefs-ai-sop/rules/test-discipline.md`
「单次观测不算数」那条对它成立：同一份提示、同一个模型，两次可以给出方向相反的答案。

⚠️ **两个方向的门槛不一样**，与那条规则同形：

| 结论 | 采信条件 |
|---|---|
| **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
| **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |

⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的。

```

**出处 `.claude/rules/three-way-inference.md:163-174`（整段抄，未转述）**

```markdown
## 云端腿的报告要分段落盘

云端腿按提示先把报告写进 `research/prompts/*-output.md`、最后才回复。**提示里还要写明分段写**：
每一次工具调用写进文件的内容不超过 150 行，模型源码与报告都分段追加，第一段排他新建，回复只写短句。
⇒ 看到「超过输出上限」的失败，先 `ls` 产物（多半是空的），按分段写的提示重派，不要续那条腿——它的上下文里没有能用的东西。

**撞了限额、被停或报错的腿，新开一条接着做，不续原来那条。** 失败通知里写着 failed 或 killed 时，先 `ls -la` 看它的报告与模型落了多少：腿在最后一步（把全文当回复返回）才撞限额的，报告文件多半早已完整落盘，完整就直接读、不用接着做。没做完的，跑 `python3 research/scripts/agent-handover.py --agent <它的 id> --out <交接摘要>`，从它的会话记录里机械地抽出派发提示、续做消息、写过的文件与现状、跑过的命令与输出、它最后说的话，再派一条同立场的新腿，派发提示指到交接摘要与已落盘的产物：第一步核现场，已落盘的不重做，接着做完。交接摘要只给新腿读，主 agent 不读它；接续非得经过主 agent 的上下文，就干脆从头重派。续做闸 `.claude/hooks/continuation-guard.sh` 拒绝给最近一次任务通知是 failed 或 killed 的子 agent 续做。**上下文多大不是停、不续的理由**：看门狗报「上下文过大」时，主 agent 看它是不是在原地打转或做派发之外的事，不是就接着跑。

**提示里还要写明：草稿与临时文件放腿自己的目录**（`/tmp/claude-1000/<腿名>/`），不放会话共用的暂存目录。

**提示里还要写明：引 kb 条款时写 kb 文件自己的行号。** 云端腿读的是拼好的背景材料，它顺手记下的行号是背景材料里的行号，贴在 kb 文件名后面就指向一个不存在的位置。⇒ 提示里写「引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数」。

```

**出处 `.claude/rules/three-way-inference.md:180-215`（整段抄，未转述）**

```markdown
## 给本地腿的提示一律用英文

本地那条腿是 4-bit 量化模型，**中文输出会退化性复读**（「有效的有效性」「恢复恢复」），
**英文不会**。数据与口径见 `.claude/kb/tooling.md`。

⇒ **提示用英文写，答案也让它用英文回。**

⚠️ **英文提示里的每一句转述，写完都要对着原文核一遍。** 本地腿读不到中文附录，条款只能译成英文转述，而转述正是「引 kb 里的条目要整行抄」那条纪律够不着的缺口：译的人觉得忠实，丢的往往是一个限定词。⇒ 写完英文提示，把每一句转述与附录里的原文并排放一遍，缺一个限定词就补上。**多出来的也要列**：英文比原文多一个限定词、多一个括注，同样在核对表里单列一行、写明为什么加。

**这条不靠自觉**：`ask-local.sh` 里有一道会拒绝的闸——输出判定为字词损坏时
**退出码 5**，并打印下一步。

⚠️ **损坏有四类，签名互不相同，一个检测器只查得了一类**：复读（多吐了）、
成对标记落单（整段掉了，`**Attack P1**:it impossible.**`）、
拼接（两词粘死，`configurationing` `batchinggroup`；**后一个词还可以缺头**——
`inaccessibleisabled` 缺一个字母、`cryptographicord`（cryptographic + [Rec]ord）缺三个，
缺到尾巴不成词时前几条规则全切不开，靠「长前缀 + 既不成词也不是后缀的短尾巴」这条才抓得到；**粘上的还可以是这个词自己尾部的一截**（`reachableachable`，短到 6 个字母的 `anewew`），或者前缀是词表外的派生词（`observationalomputational`）；闸判绿之后照样通读，闸只认得登记过的形态）。第四类是**实词自复读**
（`resetting resetting`，同一个 ≥5 字母的词连着出现两次、中间隔一个空格）——
闸只查长实词（短虚词的自复读在正常英文里合法），不收窄就误报。
`ask-local.sh` 串跑 `corruption-check.py` 与 `oov-check.py`，任一判红即拒绝。
**只跑一个等于对另外三类判绿。**

⚠️ **提示里不许用 markdown 强调。**

⚠️ **本地腿要前台跑，不要 `setsid ... & disown`。**

**闸判红之后**：那一轮**作废重跑**，不许记成「三方不一致」——
否则每次都会不一致，这条规则就退化成了摆设。
⚠️ **这不是洁癖**：**损坏的不只是词。**
确有必要采用带损坏的输出时，设 `ASK_LOCAL_ALLOW_CORRUPT=1`，
并在结论里写明这一票带瑕疵。

⚠️ **闸报「没跑成」时不是通过。** 检测器自己出错（退出码 2）与判红（退出码 1）
是两件事，`ask-local.sh` 分开报。看到「没做字词损坏检查」就是**这一项没验**，
不许当成验过了。

```

**出处 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md:9-30`（整段抄，未转述）**

```markdown
## 一、G1–G9 逐条

| G | 文件与小节（改后行号） | 改了什么 |
|---|---|---|
| G1 | `.claude/agents/gate-triage.md`「输入」、第 1b 步（:26）、第 2 步（:27） | 撤回 F9：删「`gate.sh` 整条经内存包装的上限」这一项输入；1b、2 还原成 F9 之前的写法（草稿 `/tmp/claude-1000/defs-closeout-r1-fixes/before/.claude/agents/gate-triage.md`），命令回到 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`。与那份草稿逐行 diff 只差一处：1b 的括注从「87 号经 `research/scripts/replay.sh` 逐条套了」变成「87 号经 `research/scripts/replay.sh` 逐条套了，15、74 号在阶段里面经包装」 |
| G1 | `.claude/gate.d/74-model-differential.sh`（:39–43 注释与变量；cargo 那一支） | `cargo test --release -p singlefs-harness --test "$TEST_BINARY"` 改成经 `research/scripts/run-with-memory-cap.sh` 跑（包装路径按阶段文件自己所在的仓根取，与同一文件取 `stage-must-run.sh` 的写法相同）。上限取 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`，没设取 8G（`research/scripts/replay.sh:24` 的 `REPLAY_MEMORY_CAP` 默认值）。包装退 250–254 单独判红、出路指到包装文件头「退出码」一段；普通判红那一支的单跑命令改用同一个上限变量的值 |
| G1 | `.claude/gate.d/15-research-build.sh`（:25–38） | `cd research && cargo test --release` 改成经同一个包装跑；上限取 `GATE_RESEARCH_BUILD_MEMORY_MAX`，没设取 8G；峰值表的键写死成 `gate 15-research-build: cargo test --release (research)`（不写死的话键是「cargo test --release」，分不出工作区）；包装退 250–254 单独判红 |
| G1 | `.claude/agent-common.md`「不做」一节「跑编译出来的代码经内存包装」那一条（:48） | 「`mutate.sh`、`replay.sh` 与门禁 59 号在里面逐条套了」后面加「门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`）」 |
| G1 | `.claude/agents/crash-verifier.md` 第 1b 步（:26） | 「59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层」 |
| G1 | `.claude/agents/implementation-writer.md` 第 1b 步（:26） | 末尾加「登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层」 |
| G1、G9 | `.claude/main-agent.md`「派发提示怎么写」（:48） | 删「派门禁分诊员时给 `gate.sh` 整条经内存包装的上限」；改成「派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个数，照 crash-verifier「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个数，没给用 `replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值」 |
| G9 | `.claude/agents/crash-verifier.md`「输入」（:20） | 「每道一个数：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值）还是推的；55 号的不小于……」（55 号照 F8 原样） |
| G2 | `.claude/agents/experiment-runner.md` 第 4 步（:31）、第 4c 步（:33） | 删「逐字节一致就删掉这一次的新文件、不新存」。改成「这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重」。4c 里「逐字节一致没新存的，读 `research/results/` 里那一份」那半句随之删掉 |
| G3 | `.claude/agents/three-way-attack.md` 第 3b 步第 4 条（:33） | 加「原型里的流只许在原型里自己造，不从名字带 `layer0` 的用例里拷（拷文件、拷代码段、`include!`、`mod` 引进来都算拷）」与「这一轮全部原型跑的全量合起来不超过约 10⁷ 个状态」；结尾改成「一段历史自己就超过约 10⁶ 的，那段不跑；再跑就要越过约 10⁷ 的，剩下的不跑；两样都写进报告交主 agent」 |
| G4 | `.claude/hooks/ask-user-claim-guard.sh`：`FILE_AND_LINE`（:57–59）、`FILE_NAME_CHARACTER` 上的注释、文件头判法、自证 | 第一支 `/` 之后换成攻方 `f13-fix-g1.sh` 的写法 `(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))`（逐字相同）；自证加 6 格必拒；文件头判法与成功行跟着改（第二节） |
| G5 | `.claude/agents/experiment-runner.md` 第 4c 步（:33） | 点名对象改成表示「没过」的字段：取值是布尔 `false` 的、取值是 `not_run` 的、取值是大于 0 的整数而名字表示违例 / 不匹配 / 歧义 / 失败的计数；取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。「一个都没有」那句改成「……违例、不匹配、歧义、失败类的整数计数都是 0」 |
| G6 | `.claude/agents/three-way-attack.md` 第 3c 步（:38）；`.claude/agent-common.md` ④（:64） | 3c：`.rc` 文件名带批号与件号（`<草稿目录>/b<批号>-<件号>.rc`），每批开跑前 `rm -f <草稿目录>/b<批号>-*.rc`；`wait` 之后数一遍这一批的 `.rc`，与派出去的件数对不上整批作废，对得上再按起的次序读。④：同样三句（文件名带批号与件号、每批开跑前先删这一批的文件、读之前数一遍、对不上整批作废） |
| G7 | `.claude/agents/experiment-runner.md` 第 6 步（:35）、第 7 步（:36） | 第 6 步「84 号除外」改成「读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）」，放到第 7 步写完实验页之后跑；不写实验页的在第 6 步最后跑、红了照写，并加「产物照第 4 步一个不删（86 号出路里『把 research 下那些文件删掉』那一句不照做，写进报告交主 agent）」。第 7 步末尾改成「跑第 6 步留下的那几道，各贴原样末行与退出码」。名单为什么是十道见本节表下 |
| G8 | `.claude/agents/three-way-local-defense.md`「文件名形态」（:19） | 样本不再写死成 `research/prompts/<轮>-local-defense-output-s<n>.md`：「样本照攻方写成 `<前缀>-output-s<n>.md`，前缀取派发提示给的（与攻方「输入」那一项同），不写死」；提示、核对表、运行记录三个名字不变 |

**G7 名单为什么是十道**：判决第三节 G7 写的判据是「登记给它的阶段里凡是读实验页的（84、40、86 号）」。照这个判据现查了登记给 experiment-runner 的 14 道（`awk` 取法同共用约束「门禁」一节）：`27 33 34 40 52 80 84 85 86 88 69 75 96 99`。逐道 `grep -n 'kb/experiments\|experiments\.md\|\.claude/kb\|KB=\|kb_dir'`，读实验页或索引的是 27（`:56` `glob('.claude/kb/**/*.md')`）、34（`:30–31` `IDX=.claude/kb/experiments.md`、`EXP=.claude/kb/experiments`）、40（`:4` 判据点名 `kb/experiments.md`）、69（`:119` `os.path.join(kb_dir, "experiments")`）、75（`:40` `.claude/kb/experiments`）、84、85（`:14` `EXP_DIR=.claude/kb/experiments`）、86、88（`:54` `.claude/kb/**/*.md`）、99（`:33` `EXPERIMENTS=.claude/kb/experiments`）；不读的是 33、52（读 `.claude/kb/layout/01-first-txn.md`，不是实验页）、80、96（读 `.claude/kb/checks-owed.md`）。判决括注只列了会因「页还没写」判红的三道；另外七道在页写出来之前跑是空判（新实验的页不在，判不到它），照「凡是」一起挪。这是按判据字面扩的名单，不是判决括注的原样，第三轮可以攻。

```

**出处 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md:79-200`（整段抄，未转述）**

````markdown
## 三、G1：74、15 号的探针、假根演示与样本

### 三 a、喂 `heavy-test-guard.sh` 的探针（只喂 JSON、只看退出码，一条都没执行）

用攻方的 `probe.py`，用例 `/tmp/claude-1000/defs-closeout-r2-fixes/probe/cases-r2-fixes.json`，输出 `probe/r2-fixes-out.txt`，原样：

```
G1-01	heavy	gate-triage	fg	exit=0	
G1-02	heavy	gate-triage	fg	exit=0	
G1-03	heavy	gate-triage	fg	exit=2	✗ 重型测试被拒：gate.sh --staged（整轮门禁）：gate-triage 跑「整轮门禁」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
G1-04	heavy	gate-triage	fg	exit=0	
G1-05	heavy	gate-triage	fg	exit=0	
G1-06	heavy	implementation-writer	fg	exit=0	
G1-07	heavy	implementation-writer	fg	exit=0	
G1-08	heavy	crash-verifier	fg	exit=0	
G1-09	heavy	crash-verifier	fg	exit=0	
G1-10	heavy	主 agent	fg	exit=0	
G1-11	heavy	implementation-writer	fg	exit=0	
G1-12	heavy	implementation-writer	fg	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（implementation-writer 不经 run-with-memory-cap.sh）
G1-13	heavy	gate-triage	fg	exit=0	
G1-14	heavy	crash-verifier	fg	exit=0	
G1-15	heavy	gate-triage	fg	exit=0	
G1-16	heavy	gate-triage	fg	exit=0	
G6-01	detector	three-way-attack	rib	exit=0	
G6-02	detector	three-way-attack	fg	exit=0	
G6-03	heavy	three-way-attack	rib	exit=0	  ! heavy-test-guard.sh：bash / sh 起的脚本 /tmp/claude-1000/defs-closeout-r2-fixes/probe/w/a.sh 不存在，里面的命令没判；这条命令照常执行，记了一条检出
```

命令（cwd 仓根）：G1-01 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（门禁分诊员还原后的第 2 步）；G1-02 同一条不带 `--staged`；G1-03 对照：不带前缀；G1-04 `nice -n 19 bash .claude/gate.d/15-research-build.sh`；G1-05 前面加 `GATE_RESEARCH_BUILD_MEMORY_MAX=12G`；G1-06、G1-08 `nice -n 19 bash .claude/gate.d/74-model-differential.sh`（实现员、崩溃验证员）；G1-07、G1-09 前面加 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`；G1-10 主 agent 直接跑 74 号；G1-11 74 号普通判红时打印的单跑命令 `bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture`；G1-12 对照：同一条不经包装；G1-13 `bash research/scripts/capped.sh 8 bash .claude/gate.d/15-research-build.sh`；G1-14 崩溃验证员 1b 的 54 号写法（没变，核一次）；G1-15 分诊员单跑 87 号；G1-16 F9 那一版的整条包装写法（hook 照放，定义里已经不叫人这么跑）。G6-01/02/03：3c 新写法（`rm -f …/b2-*.rc; { …; echo "$?" > …/b2-1.rc; } & { …; echo "$?" > …/b2-2.rc; } & wait; n=$(ls …/b2-*.rc | wc -l); if [ "$n" -ne 2 ]; then echo void; else cat …; fi`）在 run_in_background 与前台喂检出 hook、在 run_in_background 喂重型闸，都放行；G6-03 那一行 `!` 是 `a.sh` 在探针里不存在，只记检出。检出记录 3 条（G1-03、G1-12、G6-03 各一条），在 `probe/probe-detections-63sz6yhu.jsonl`。

重型闸按名字判门禁阶段、不读正文（`.claude/rules/implementation-workflow.md:59`），所以 74、15 号阶段里面加了包装不改变它对这两道的判定：两道对各身份照旧放行。

### 三 b、改后的两道阶段在假根上跑（经包装那一支的三种结局）

两道都没在仓上真跑（原因见第五节）。改用两个只有几行 Rust 的假根：`fake74`（工作区成员 `crates/singlefs-harness`，测试目标就叫 `second_transaction_supplement_three_random_history`，打出六段标题与「模型对拍 N 步」）与 `fake15`（`research/` 下一个带两个单测的 crate）；两者的测试在设了 `FAKE_ALLOCATE_MIB` 时先摸满那么多内存。先 `cargo test --offline --release … --no-run` 只编不跑；阶段从一个镜像目录起（`mirror/.claude/gate.d/` 放改后的两份脚本，`mirror/research/scripts` 是指到仓里 `research/scripts` 的符号链接，这样 `$(dirname "$0")/../..` 取到的包装就是仓里那一份），峰值表指到草稿目录里的私有一份（`RUN_WITH_MEMORY_CAP_PEAKS`，不碰主仓的 `memory-peaks.tsv`），`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=300`。假根的全部源码存在 `g1/fake-roots-sources.txt`。

绿（默认上限）：

```
== 74 green (default cap)
  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：
      随机历史快档：模型对拍 10 步：假根替身
      随机历史：偏向抬 F 之后复用的取样点：模型对拍 11 步：假根替身
      随机历史：偏向抬 F 之后回退的取样点：模型对拍 12 步：假根替身
      随机历史：越过原分配记录墙的取样点：模型对拍 13 步：假根替身
      随机历史：小盘上逼近单元区墙的取样点：模型对拍 14 步：假根替身
      随机历史：小盘上逼近单元区墙的取样点（空间准入判着）：模型对拍 15 步：假根替身
exit=0
== 15 green (default cap)
  ✓ research 构建通过，2 个测试批次、共 2 个单测全绿
exit=0
== peaks
9437184	2026-09-26	8G	cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
28311552	2026-09-26	8G	gate 15-research-build: cargo test --release (research)
```

私有峰值表里两行的上限列都是 8G、键是设计的那两个：两条 cargo 确实在包装里、按默认上限跑的。

红一：撞上限（`FAKE_ALLOCATE_MIB=400`，上限 200M）：

```
== 74 red (cap 200M, allocates 400 MiB)
    Finished `release` profile [optimized] target(s) in 0.00s
     Running tests/second_transaction_supplement_three_random_history.rs (target/release/deps/second_transaction_supplement_three_random_history-c6b70fa26f432660)

running 1 test
error: test failed, to rerun pass `-p singlefs-harness --test second_transaction_supplement_three_random_history`

Caused by:
  process didn't exit successfully: `/tmp/claude-1000/defs-closeout-r2-fixes/g1/fake74/target/release/deps/second_transaction_supplement_three_random_history-c6b70fa26f432660 --nocapture` (signal: 9, SIGKILL: kill)
run-with-memory-cap: 撞了内存上限 200M（scope singlefs-memory-cap-1468639-87326325 的 Result=oom-kill，命令退出码 101）
  ✗ 随机历史的测试二进制没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 250（上限 200M），这一次的输出不算判定
     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；
                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。
exit=1
== 15 red (cap 200M, allocates 400 MiB)
         Finished `release` profile [optimized] target(s) in 0.00s
          Running unittests src/lib.rs (target/release/deps/fake_research-073f73e920db56df)
     
     running 2 tests
     test tests::second_test_is_counted ... ok
     run-with-memory-cap: 撞了内存上限 200M（scope singlefs-memory-cap-1468695-1038813893 的 Result=oom-kill，命令退出码 143）
  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 250（上限 200M），这一次的输出不算判定
     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；
               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。
exit=1
```

红二：测试自己失败（`FAKE_ALLOCATE_MIB=not-a-number`，解析 panic，cargo 退 101，走原来那一支；74 号只贴了末 6 行）：

```
== 74 red (test panics)

error: test failed, to rerun pass `-p singlefs-harness --test second_transaction_supplement_three_random_history`
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节（经内存包装，上限同这一道）：
                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
exit=1
== 15 red (test panics)
  ✗ research 的构建或单测没过（cargo test 退出码 101）
     test tests::allocates_when_asked ... FAILED
     thread 'tests::allocates_when_asked' (1469116) panicked at src/lib.rs:7:54:
     test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     error: test failed, to rerun pass `--lib`
     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。
exit=1
```

251、252、253、254 这四个码没在假根上造出来（要让 systemd-run 起不来、让 slice 占满或设限时），它们与 250 走同一个分支（74 号 `case 250|251|252|253|254`、15 号 `(( rc >= 250 && rc <= 254 ))`），这一格是按代码推的。

### 三 c、74 号的红绿样本

`stage-selftest.sh` 整道会把 `.claude/gate.d/` 下每个有样本的阶段都跑一遍，没整道跑；照它对单个样本做的几步（拷进临时目录、清掉 `GATE_BASE` 三个变量、以样本目录为根跑阶段、比退出码与 `want=` 行）写了 `g1/run-fixture.sh`，只跑 74 号的两份样本。改后的脚本在镜像目录里跑一次、装进仓后再跑一次，两次原样相同：

```
74-model-differential.sh/green: exit=0 ok
74-model-differential.sh/red: exit=1 ok
```

样本走「没有 Cargo.toml、有录好的输出」那一支，碰不到经包装的 cargo 那一支，所以样本没改；那一支靠第三 b 节的假根演示。15 号没有样本目录、也没有 `--selftest`（`ls .claude/gate.d/fixtures/ | grep '^15'` 空），没有可单跑的自证；假根演示是它唯一跑过的一次。

````

**出处 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md:201-214`（整段抄，未转述）**

```markdown
## 四、门禁判定行（原样，逐道跑，都在最后一处改动之后，`nice -n 19`；日志在 `gates/`）

| 门禁 | 原样判定行 | 退出码 |
|---|---|---|
| 47 | ✓ research 脚本的自证都通过（本阶段跑了 31 条；research/scripts/ 里声称有 --selftest 的 35 份中 34 份有门禁阶段在跑）<br>没跑的 1 份（登记在本阶段的 NOT_RUN_HERE）：research/scripts/vm-bench.sh：自证要连起三次虚机，挂钟太重，不进每轮门禁 | 0 |
| 62 | ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent） | 0 |
| 63 | ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（3 个有 Write 或 Edit 的定义、18 条路径模式），共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 12 个文件），共用重型测试判定模块的 15 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 12 个文件；selftest、load_sibling_module 不算，见 NOT_SHARED_JUDGMENT） | 0 |
| 73 | ✓ 门禁自检通过：84 个脚本（.sh 与 .py）、260 条拒绝都带了出路<br>✓ 查了 72 个脚本：.sh 都可执行，暂存区里的模式与工作区一致<br>✓ 进程安全：查了 168 个脚本（.sh 127 个、.py 41 个），发信号的写法都只打得到点名的一个进程；own-scope 标注放行 1 处（research/scripts/run-with-memory-cap.sh:952）；没判的：.claude/process-safety-pending 里的 2 个文件 research/scripts/agent-watch.py（research/scripts/agent-watch.py:2162）；research/scripts/mutate.sh（research/scripts/mutate.sh:351）<br>✓ shell 纪律检查通过（共 34 个脚本）<br>✓ shell 纪律检查通过（共 8 个脚本）<br>✓ shell 纪律检查通过（共 9 个脚本） | 0 |
| doc-lint（文档铁律，`bash .claude/singlefs-ai-sop/scripts/doc-lint.sh <仓根>`） | ✓ 文档铁律检查通过（检查 485，跳过 0；DOC_LINT_VERBOSE=1 看全部） | 0 |
| 规则纪律（项目本地；照 `gate.sh` 的 `run_rules_lint` 起：`GATE_IN_STAGE=1 RULES_LINT_DIR=<仓根>/.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/skills/*/SKILL.md"`） | ✓ 规则只写怎么做（扫了 29 份文件 1793 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 9 行的日期只在「」或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1670 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=） | 0 |
| gate-overlap（门禁查重） | ✓ 相对 97f5904b44cd：新加的门禁与钩子 1 个（.claude/gate.d/84-verdict-false-named.sh）都写明了比过谁，改过的 15 份脚本对照已有的 132 份没有整段相同<br>没判的：装进来的 SOP 副本 37 份只当对照（.claude/singlefs-ai-sop/scripts） | 0 |

63 号那一行里「弹窗断言闸……自证通过」就是改后弹窗闸的 36 格自证。73 号的 gate-lint 管到改后 74、15 号新加的两处 ✗：每处后面都紧跟 → 行。

```

**出处 `research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md:215-227`（整段抄，未转述）**

```markdown
## 五、没做的与原因

- **74、15 号没在仓上真跑**。15 号是整个 research 工作区 `cargo test --release`（阶段归属表写它「全工作区十几分钟」，按量级是全量测试），不跑；74 号要 release 编整个 `crates/`，这一轮只验了改动那一支（第三 b 节假根上的绿、撞顶、测试失败三种结局）与样本（第三 c 节）。真跑一次归提交时的崩溃验证员 / 门禁分诊员。
- **默认上限 8G 对 15 号没量过**：峰值表里没有 research 整个工作区 `cargo test --release` 的行（`grep` 键含 `research` 的行都是单个 bin 或 `-p singlefs-harness` 的），推的；判决 G1 定的默认就是 `REPLAY_MEMORY_CAP` 的默认值，照写。74 号整道经包装跑过的峰值 4.41–4.70 GiB（攻方报告 H1 节峰值表三行，含编译），在 8G 以下。
- **G1 里「`check.sh` 的 `cargo test --all` 报给 SOP 会话、记进 `records/2026-09-16-subagent拆分提案.md` 第四十节」与 G3 里「派发闸误拒记进第四节」**：`records/` 与上游 SOP 不在放行文件里，归主 agent。
- **包装文件头 `research/scripts/run-with-memory-cap.sh:72`** 讲嵌套时仍拿「整条 gate.sh 经它跑」当例子；机制描述本身还对，但 G1 之后没有定义再叫人这么跑。那份不在放行文件里，没改。
- **`.claude/gate.d/stage-inputs.tsv` 的 74 号那一行**（`crates/ Cargo.toml Cargo.lock`）没把 `research/scripts/run-with-memory-cap.sh` 列进输入：阶段现在读它，包装改了不会让 74 号的复用判定失效。不在放行文件里，没改（推的影响：包装改坏时 74 号可能按「输入没变」复用旧绿）。
- **攻方 H2 附带的一格**（40 号把 `*.r[0-9].out` 当逐轮中间件跳过，G2 之后新文件常写成 `rN`）不在 G1–G9 里，没动。
- **重型闸对 F9 那种整条包装写法照放**（第三 a 节 G1-16 exit=0）：定义里已经不这么写，闸没拦它；闸不在放行文件里。
- **`bash-command-detector.sh` 自己给 ④ 的拒绝出路**没跟着 G6 改（不在放行文件里）：`bash-command-detector.sh:2395–2397` 的出路只写「几条活要并行就在同一条命令里 `a & b & wait`，用不带参数的 `wait` 等齐」，不提退出码文件、批号件号与数件数。
- **G7 名单**是照判据「凡是读实验页的」现查扩出来的十道，不是判决括注里的三道（第一节表下）。
- 没跑重型测试（54、55、57、59、87、层 0、QEMU、herd7、全量 cargo test、整轮门禁）；没跑 72 号（判形式的是主 agent 写的判决，归主 agent）；`stage-selftest.sh` 整道没跑，只跑了 74 号两份样本。

```

**出处 `research/prompts/defs-m2-closeout-r2-main-verification.md:1-57`（整段抄，未转述）**

```markdown
# 定义收尾第二轮判决（defs-m2-closeout-r2，2026-09-26）

<!-- doc-lint:not-numbers E1 E2 E3 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 G1 G2 G3 G4 G5 G6 G7 G8 G9 H1 H2 H3 H4 H5 H6 H7 H8 H9 O3 O5 O8 O9 O11 O12 -->

## 一、这一轮

- 正文 `research/prompts/_defs-m2-closeout-r2-body.md`，背景材料 `_defs-m2-closeout-r2-background.md`，附录二 `_defs-m2-closeout-r2-diff.md`，开工快照 `research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt`（核查员 `sha256sum -c` 24 个全 OK）。
- 腿：辩方 `defs-m2-closeout-r2-sonnet-output.md`；云端攻方 `defs-m2-closeout-r2-opus-output.md`（模型 `defs-m2-closeout-r2-opus-model/`）；本地攻方 `defs-m2-closeout-r2-local-attack-output-s1.md`、`-s2.md` 两份干净。
- 核查员 `defs-m2-closeout-r2-verifier-output.md`：150 处 ✓147 ✗2 核不动 1。攻方 76 处 0 个 ✗，`rerun.sh` 整份复跑承重的探针与演示逐条一致；辩方 1 处行号错位（`:15` 应为 `:25`，文字抄对）；本地攻方运行记录把 s1 的生词数写成 1、应为 2。
- **第一轮判决的两处更正**（原判决照留，更正写在这里）：第一轮判决第一节「96 处 ✓83」应为 ✓82（核查员复核第一轮核查员表三：本地攻方分项 6 个 ✓，不是 7 个）；第一轮把 O9 与 O3、O8 同评「部分」评轻了——O9 的代价（`check.sh`、15 号、74 号起的 cargo 不在任何内存上限里）是现查坐实的现状（辩方 E3）。

## 二、逐格判

| 攻方编号 | 被攻的改法 | 判 | 改法 |
|---|---|---|---|
| H1 | F9（`gate.sh` 整条经内存包装） | **打中，F9 这个方向错**：整轮放进一个 scope，任一阶段撞上限 systemd 就停掉整个 scope、后面的阶段不跑（私有 slice 上缩小版 6 次都退 250）；外层按要的量占账，里层 59 / 87 的包装排不上（252）；上限只能落在一个很窄的窗口里，32 线程时多半是空的（推的） | G1 |
| H2 | F7（输出逐字节不变就删新文件） | 打中：与 69 号绕成死圈 | G2 |
| H3 | F4（原型跑小流不算重型） | 打中：把层 0 流拷成不带 `layer0` 的名字 hook 就放行；上限只限「每次」、分几次能拼回整套；派发闸把主 agent 转述 3b 的自然写法误拒（D1–D3） | G3 |
| H4 | F13（弹窗闸认中文文件名） | 打中：6 句没有出处的断言改后放过；攻方的收严在副本上自证 30 种全过、这 6 句照拒 | G4 |
| H5 | F5（按名字里四个词认违例计数） | 打中：漏 `journal_differing_states` 这类，又把 `*_violations_ok=true`、`layer0_violations=zero` 这类当成「不为 0」要点名 | G5 |
| H6 | F1 × F2（并行收退出码） | 打中：分批时每批复用同一组 `.rc`，某一件在写 `.rc` 之前被停，读到上一批的 0；也没带「派出去多少项就收回多少项」 | G6 |
| H7 | F16（84 号挪到写实验页之后） | 打中：40、86 号同形，只挪了 84 号 | G7 |
| H8 | F10（实现员照跑登记给它的 74 号） | 打中：74 号里面的 `cargo test --release` 不经包装 | G1 一并 |
| H9 | F6（本地辩方改名） | 小：攻方的样本前缀由派发给，辩方写死 | G8 |
| — | F8 的 55 号下限 | 没打中（推的）；真正缺的是 54、57 号的上限算法 | G9 |
| — | F3、F10 的 8G、F11、F12、F14、F15 | 没打中 | — |
| E3 | 第一轮「部分」「小」与延后两件 | O3、O8、O11、O12 判得够；O9 评轻了（第一节更正）；延后两件站得住：提交时都走带 `SINGLEFS_HEAVY_TESTS=commit` 前缀的路径，碰不到这两处 | — |

## 三、改法（被攻过零轮，第三轮攻）

- **G1** 撤回 F9：门禁分诊员照旧直接起 `gate.sh`，不整条包；**起 cargo 的门禁阶段各自在阶段里面经内存包装**：74 号、15 号把阶段里的 `cargo test` 改成经 `research/scripts/run-with-memory-cap.sh`（上限取派发提示或 `REPLAY_MEMORY_CAP` 的默认值），59、87 号今天已经各自逐条包着，照旧。实现员与崩溃验证员跑 74 号时照同一份阶段脚本，里面已经包了，定义里只写「照跑」。`check.sh` 是上游 SOP 的共享脚本，本仓不改，它的 `cargo test --all` 不在内存上限里这一件报给 SOP 会话（第四节）。
- **G2** 撤回 F7 的「逐字节一致就删新文件」：执行员不删任何产物；这一次的输出写新文件名，`research/scripts/replay.sh` 的登记行改指新文件，旧的留着。
- **G3** 攻方第 3b 步：原型里的流只许自己在原型里造，不许从名字带 `layer0` 的用例里拷流；一轮里全部原型跑的全量合起来不超过约 10⁷ 个状态，超过的交主 agent。派发闸对「在原型里调枚举函数跑小流」这类否定或转述写法的误拒，记进第四节。
- **G4** 弹窗闸照攻方的收严改（攻方模型目录里的 `f13fix` 改法），自证照它的 30 种，外加这一轮那 6 句没有出处的断言各一格必拒。
- **G5** 执行员第 4c 步：点名的对象是判决行里表示「没过」的字段——布尔值为 false、取值为 `not_run`、以及数值计数里名字表示违例、不匹配、歧义、失败且值大于 0 的；布尔值为 true、取值为 `zero`、以及名字不表示「没过」的计数（例如 `journal_differing_states`）不点名。拿不准的点名并写「拿不准」。
- **G6** 攻方第 3c 步与共用约束 ④：退出码文件名带批号与件号，每批开跑前清空这一批的文件；收的时候数一遍，文件数与派出去的件数对不上整批作废。
- **G7** 执行员：登记给它的阶段里凡是读实验页的（84、40、86 号）都挪到第 7 步写完实验页之后跑；这一次不写实验页的在第 6 步最后跑，红了照写。
- **G8** 本地辩方的样本前缀与攻方同，取派发给的前缀，不写死。
- **G9** 崩溃验证员「输入」的内存上限：54、57 号各给一个数（没量过的写「推的」），55 号照 F8；主 agent 派发时照给。

## 四、延后与报给别处的

- **上游**：`check.sh`（SOP 共享脚本）里的 `cargo test --all` 不经内存包装，本仓改不了；报给 SOP 会话，并记进 `records/2026-09-16-subagent拆分提案.md` 第四十节。
- **派发闸的误拒**（H3 的 D1–D3：主 agent 用自然写法转述 3b 被拒）：记进第十步「总结与归档」的素材。
- 第一轮第四节延后的两件（重型闸静态分支与版本号白名单、84 号只认 false）照旧延后。

## 五、按路径点名被判的定义与文件（门禁 72 号）

`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-local-defense.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/mutation-triage.md`、`.claude/agent-common.md`、`.claude/main-agent.md`，另有 `.claude/rules/implementation-workflow.md`、`.claude/hooks/ask-user-claim-guard.sh`、`.claude/gate.d/74-model-differential.sh`：第二轮判过，打中的按第三节改，改完第三轮攻；`.claude/gate.d/15-*.sh` 随 G1 改、第三轮一起判。

## 六、第三轮（最后一轮）

只攻 G1–G9 的改后字面；第三轮之后停，第三轮里新冒出来的零轮改法写进判决的交用户表、标「零轮」。

## 回看决策

不涉及决策：被判的是 agent 定义、共用约束、主 agent 定义、规则、hook 与门禁，不改 `.claude/kb/decisions/` 里任何一条决策。
```

**出处 `research/scripts/run-with-memory-cap.sh:17-27`（整段抄，未转述）**

```markdown
# 退出码：
#   0–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 250–254）；
#   250    撞了这一条自己的上限：scope 的 Result 是 oom-kill，scope 自己的 memory.events 里 oom 不为 0（或读不到）。
#          只认 Result、不认退出码：被杀的是 cargo 起的测试二进制时 cargo 退 101 或被停时退 143，被杀的是命令本身时退 137，外面 kill -9 同样退 137；
#   251    slice 的总上限设不上、systemd-run 起不来（没有用户级 systemd、D-Bus 连不上、上限写法 systemd 不认）、账与锁的目录建不了，或查不到 scope 的结局：
#          那条命令一行都没跑（或结局判不了），不退回无上限、无总上限去跑——那正是要防的事；
#   252    内存不够排不上：等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下，或这一条要的量比总上限还大（等多久都放不下，不等）。命令一行都没跑；
#   253    超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）：scope 的 Result 是 timeout，systemd 停掉了 scope 里的进程；
#   254    被总上限挤掉：scope 的 Result 是 oom-kill，而 scope 自己的 oom 计数是 0、oom_kill 不为 0——整个 slice 满了，内核在 slice 里挑了这一条杀，
#          不是它自己撞了它的上限；它的结果不算数，重跑它；
#   2      用法错（上限、余量、总上限不是 正整数[KMGT] 的写法，没给命令）。
```

**出处 `research/scripts/run-with-memory-cap.sh:72-72`（整段抄，未转述）**

```markdown
# 包装里再经包装跑的（整条 gate.sh 经它跑，里面 59 号的每条再经它）：里层另起一个 scope、挪出外层，挂在同一个 slice 里各排各的队；外层照它要的量占着账。
```

**出处 `.claude/hooks/bash-command-detector.sh:2407-2410`（整段抄，未转述）**

```markdown
            print("     → 怎么办：长活直接放 run_in_background，命令里只写那条活本身，不加 `&`"
                  "（几条活要并行就在同一条命令里 `a & b & wait`，用 `wait` 等齐）；"
                  "已经在跑、pid 已知的，另起一条 run_in_background 等它：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。"
                  "这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
```
