# 门禁收缩与提炼：gate-shrink-r1（2026-09-28）

**实现今天的样子**（`.claude/rules/implementation-first.md` 第 3 条）：这一轮判的是门禁与协作定义本身（`.claude/gate.d/`、`research/scripts/`、`.claude/hooks/`、`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/`），不改文件系统的设计与实现。`crates/` 只作被判的对象出现：代码类、harness 类、checker 类、checker-tier 类门禁读 `crates/` 的源码与测试，这一轮不改 `crates/` 里任何文件（第一轮的合并也没改，`git diff --stat HEAD -- crates/` 里的改动归别的会话的 KV 与测试工作）。

**用户定案**（2026-09-28，原话）：「门禁部分 可以合成的要合成 而不是互相引用」「你最好按照下面的分类来 文档类门禁 代码类门禁 harness类门禁 checker类门禁 checker-tier类门禁这样组织 每个门类下面几个门禁」「做完后 门禁取消编号 不再以编号命名 以后记录门禁按照全名记录」「继续 门禁需要继续收缩 需要继续分类和精华 同时针对门禁 进行三方和测试」；另把「扫仓的门禁只在提交前由 `gate-staged.sh` 起的整轮跑」定成一条规矩（第一轮 Q9，调度记录 `records/2026-09-28-门禁59号提速与双机分片.md`）。

## 一、现状（事实，出处都能现查）

- 第一轮把本地门禁从 79 道合成 23 道：`ls .claude/gate.d/*.sh`；`wc -l .claude/gate.d/*.sh` 合计 15276 行。
- 逐格盘点（只读调查交回，拷进仓里）：`research/prompts/gate-shrink-r1-inventory/cells.tsv`（84 行，sha256 `74d4bd10ea93787a255cef5ee23b0a41443149f45c7808bcdd0043eba71ea79c`）与同目录 `report.md`（sha256 `69c494d285d9e40f35e95bfbe9e0cff052942542eec6106b1f8ca8a7a4bea898`）。每格十列：判什么、读什么、单格实跑耗时、与共享门禁或别的格是否重复、依附的规矩在不在、记录里抓到过没有、今天对工作区的判定。材料员把整张表原样放进附录。
- 今天对工作区（几个会话未提交的改动都在里面）：绿 54、红 17、退 77 有 4、不跑 7（重型或要编译）、没跑完 2。
- 耗时：84 格里多数不到 1 秒；超过 10 秒的只有 12 号 term-renames（>5 分钟）、47 号 research-script-selftests（>5 分钟）、12 号 clock-times（37.8 秒）、10 号 citations（16.9 秒）、33 号 reading-discipline（13.6 秒）。
- 盘点报的四处同一件事跑两遍：33 号 mutation-tables 与 59 号用同一份 `row_problems`；47 号 agent-write-scope 与共享 `hooks-registered` 各跑一遍同一批钩子自证；43 号 table-shape 与 doc-lint F 都判「同一编号不许两处登记」；54 号快档与 74 号的用例已在共享「构建与单测」的 `cargo test --all`（debug）里跑过一遍。
- 盘点报的次序依赖：`gate.sh` 按文件名排序跑本地阶段（`gate.sh` 第 636 行一带），77 号文件头写「这一阶段排在最后」而 87、92、94 号在它后面；去掉编号改成 `<类>-<内容>.sh` 之后次序按类名字母序（checker、checker-tier、code、doc、harness）。
- 47 号 research-script-selftests 报 4 份「声称有 `--selftest` 却没阶段在跑」：认法是文件里出现 `--selftest` 这串字就算（47 号第 241 行一带）；4 份里只有 `rerun-failed-tests.py` 真有自己的 `--selftest`；原 47 号（HEAD 版）对同一份工作区报同样 4 份。
- 54、59 号与双机分片、按输入复用与全绿标记，用户已定由 singlefs-29 的里程碑三第二轮按「先录入（KV）、后核对」重设计；这一轮对它们只改名、改开关，不加功能。

## 二、方案（要腿攻的）

### A 再合：按五类，每类几道

| 类 | 合成后 | 由哪几道并成（第一轮合出的名） | 格数 |
|---|---|---|---|
| 文档类 | doc-kb | 20 doc-decision-documents、30 decision-history-entries、32 doc-field-and-layout-registry、10 references-and-invariants、43 checks-owed-and-closeout：都判 `.claude/kb/` 里决策、变更史、布局表、欠账表与引用 | 37 |
| 文档类 | doc-experiments | 34 doc-experiment-pages-and-products | 10 |
| 文档类 | doc-process-records | 11 review-and-sync-records | 7 |
| 文档类 | doc-text | 12 doc-forbidden-notations-and-old-terms | 3 |
| 代码类 | code-source-discipline | 13 code-vague-names-and-test-file-names、27 code-constants-enums-bits-match-kb、33 code-experiment-and-mutation-source-discipline | 8 |
| 代码类 | code-tooling | 47 code-tooling-selftests-and-registries | 8 |
| 代码类 | code-research-build | 15 research-build（要编译） | 1 |
| harness 类 | harness-tests | 14 harness-one-scenario-per-test、74 model-differential | 2 |
| checker 类 | checker-independence-and-sync | 92 layout-checker-sync、94 checker-implementation-disjoint | 2 |
| checker-tier 类 | 各自一道 | 54、55、57、59、87 不并（重型，各有复用标记与内存上限，54、59 在重设计）；77 test-environment 见 B6 | 6 |

合计 15 道。文件名去编号：`<类>-<内容>.sh`。

### B 砍、改（「精华」）

| # | 对象 | 方案 | 依据 |
|---|---|---|---|
| B1 | 43 号 row27-preconditions | 里程碑二提交之后删这一格 | 只盯里程碑二收口表第 27 行那几笔欠账的前置，今天退 77 |
| B2 | 47 号 agent-write-scope 里对钩子自证的调用 | 删掉与共享 `hooks-registered` 重复的那一段，只留表与定义双向一致 | 盘点第 7 列 |
| B3 | 43 号 table-shape 与 doc-lint F 重叠的那一半 | 删重叠的一半，留 doc-lint 不判的部分 | 盘点第 7 列 |
| B4 | 74 号 model-differential 与「构建与单测」重复跑用例 | 待腿判：留 74 号（它带内存包装、逐段判过）还是靠「构建与单测」 | 盘点第 7 列 |
| B5 | 47 号 research-script-selftests 的认法 | 只认自己实现了 `--selftest` 入口的脚本，不认文字里提到别人的 | 盘点报告 |
| B6 | 77 号 test-environment | 必须排在全部跑测试的阶段之后：去编号之后次序按类名，要么改名让它排最后，要么证明它不依赖次序 | 盘点报告 |
| B7 | 12 号 term-renames、47 号 research-script-selftests 超过 5 分钟 | 查慢在哪，能收就收；不为省时砍格 | 盘点第 6 列 |
| B8 | 10 号 experiment-refs、decision-refs 与 doc-lint H、`number-name-sync` 部分重叠 | 待腿判重叠到什么程度、能不能删 | 盘点第 7 列 |
| B9 | 长期红的格：11 号 batch-scope（149 个触发文件没登记）、knowledge-sync（134 个没点名）、34 号 archive-past-rounds（1816 份没归档） | 待腿判：判据对、是流程欠账，还是判据本身不合用 | 盘点第 10 列 |

### C 第一轮改过的定义与共用约束

第一轮为「扫仓的门禁只在提交时跑」（Q9）、「看门狗放行样本自检」（Q15）、doc-lint 挪到提交时（Q8）与各组合并改了下面这些文件的文字；按合并后 11 号 agent-def-adversarial-review 那一格的要求走这一轮三方：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/kb-scribe.md`、`experiment-runner.md`、`tooling-writer.md`、`three-way-materials.md`、`prior-art.md`、`crash-verifier.md`、`implementation-writer.md`、`investigator.md`、`kb-spec-drafter.md`，规则 `.claude/rules/implementation-workflow.md`、`implementation-first.md`、`three-way-inference.md`、`format-evolution.md`、`mutation-sampling.md`、`verification.md`、`path-moves.md`，以及 `CLAUDE.md`。材料员照 `git diff HEAD --` 这些路径出 diff。不审：`three-way-verifier.md` 的删除与 `three-way-local-attack.md`（会话开始前已在暂存区）、`.claude/rules/changelog-format.md`（singlefs-0a 那一轮，它另开了 changelog-format-r1）。

## 三、要腿答的

1. **A 的合并丢不丢覆盖**：每一道被并的门禁、每一格，在合成后的哪一道、哪一格里；有没有一格在合并里消失、或两格合成一格后判据变宽变窄。拿 `cells.tsv` 逐行对。
2. **合并后单道的格数与失败信息**：doc-kb 37 格，一格红让整道红；按 `sop-first.md`「每一条拒绝都必须给出下一步」，出路还能不能指到那一格、那条规则？`--check <格名>` 够不够用？有没有该拆回来的。
3. **B1–B9 每一条**：删了或改了之后，什么违规会从此没人判？举一个具体的输入。B4、B8、B9 给出判断与依据。
4. **B6 次序**：去编号之后按类名排序，哪些阶段的判法依赖前面的阶段跑过（盘点报了 77 号、20 号 `--write` 先于 30 号 `--write`、55/57/59/74/87 看 staged-green）；给出不靠字母序偏门的办法，或证明不依赖。
5. **C 定义改动**：逐份核 diff 是不是只做了 Q8、Q9、Q15 与改门禁号要的改动；有没有改出互相矛盾的两句（例：一处说「交回前不跑门禁」、另一处还要求跑某一道）；阶段归属表第二列改成「判红时派给谁修」之后，`crash-verifier.md` 靠这张表挑提交时跑哪几道，还说得通吗。
6. **「只在提交时跑」这条规矩的代价**：书记员、执行员写错的 kb 格式，从此要到提交时才红、再派人修。有没有该留在写入那一刻就拦的（像 write-guard 那样的钩子），列出来并说理由。

## 四、分工

本地腿：攻（A 的逐格映射：给一张写死的事实表——被并门禁的每一格与方案里的合成后门禁——要模型逐格填「落在哪一道」「判据是否原样」，数出消失的格与重复的格；攻击面与云端攻方不重叠）。没有前一轮判决要复核，辩方这一轮没有对象。

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | 第 1、2、5、6 问 | 从 `cells.tsv` 与合成后的脚本出发，给出一套完整的合并与改名方案：每道合成后门禁的格表、出路、`gate-category`、跑的次序；定义改动逐份判 |
| 云端攻方（Opus） | 第 3、4、6 问与 A 里 doc-kb | 专找「合了、砍了之后从此没人判」的违规：对 B1–B9 每条造一个具体的违规输入；对 doc-kb 37 格造「一格红把别的格的出路淹掉」的场景；对次序依赖造「按类名排序后判错」的场景 |
| 本地攻方 | 第 1 问 | 按事实表逐格填：旧门禁、旧格名、方案里的去处、判据原样与否；报消失的格数与重复的格数。事实表每行写来源文件与行号，一张表不超过 6 行，分几张 |

## 五、交付

- 腿的报告写 `research/prompts/gate-shrink-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建），回复只写短句。
- 模型放 `research/prompts/gate-shrink-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb、规则、门禁脚本写那份文件自己的行号，行号去文件里现查。
- 方案按 `crates/` 与 `.claude/gate.d/` 今天的样子来谈。
- **资源**：线程上限 4；跑门禁只跑非重型的格（`--check <格名>`），一律经 `bash research/scripts/run-with-memory-cap.sh 8G <命令…>`；不跑 54、55、57、59、87 号，不跑 `gate.sh` 整轮与 `gate-staged.sh`；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；交回之前后台不许留着跑的东西。
- 不写时刻，时区名也不写；变体起新名字，不用撇号角标。

---

### 小节清单：`.claude/singlefs-ai-sop/rules/sop-first.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # SOP 优先于代码 | 不抄 | 派发只点名「加门禁或钩子之前，先找已有的」与「每一条拒绝都必须给出下一步」两节，这一级标题本身没被点名 |
| （SOP 标题之下、第一个下级标题之前的正文：第 3-6 行） | 不抄 | 派发未点名这段引言，正文在别处也没引到 |
| ## 推论 | 不抄 | 派发未点名这一节 |
| # 门禁是教学工具，不是筛子 | 不抄 | 派发只点名它下面两个二级小节，这一级标题本身没被单独点名 |
| ## 每一条拒绝都必须给出下一步 | 抄 | 派发正文明确点名的两节之一 |
| ## 好门禁的三个性质 | 不抄 | 派发未点名这一节 |
| ## 边界 | 不抄 | 派发未点名这一节 |
| ## 加门禁或钩子之前，先找已有的 | 抄 | 派发正文明确点名的两节之一 |
| ### 谁来查 | 抄 | 随「加门禁或钩子之前，先找已有的」一并抄出，不单独取法 |

### 小节清单：`.claude/rules/implementation-workflow.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实现改动的流程：写代码 → 三方对抗 → checker，checker 档只在提交时跑 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （实现改动的流程：写代码 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |
| ## 三步，缺一步就不算做完 | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |
| ## 改 agent 定义与共用约束，走同一条三步 | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |
| ## 代码轮派腿之前记一份开工快照 | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |
| ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判 | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |
| ## checker 档只在提交时跑，harness 随时跑 | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |
| ## 测试与崩溃检测优先多线程 | 抄 | 整份需要：这一轮方案与「开工快照」「checker 档只在提交时跑」等条款相关，C 节列出的规则改动之一 |

### 小节清单：`.claude/rules/verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 验证纪律：harness 档随时跑，checker 档默认只在提交时跑 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （验证纪律：harness 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 定义与名字 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## harness 档里再分轻用例与耗时用例 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## checker 档自己分快档与全量 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 崩溃枚举用例住哪、怎么登记 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 函数名与类型名不许只由空泛词拼成 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 崩溃一致性只能靠崩溃点重放验证 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 功能正确性靠模型对拍 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## checker 即规范 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 文件系统特有的反推缺口 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |
| ## 门禁管哪一半 | 抄 | 整份需要：定义 harness / checker 分档口径，是方案 A 五类分档的依据 |

### 小节清单：`.claude/rules/path-moves.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 文件与目录的路径不在冻结范围内 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （文件与目录的路径不在冻结范围内 标题之下、第一个下级标题之前的正文：第 2-7 行） | 抄 | 整份需要：变体起新名字与「文件名去编号」相关，C 节列出的规则改动之一 |
| ## 怎么做 | 抄 | 整份需要：变体起新名字与「文件名去编号」相关，C 节列出的规则改动之一 |
| ## 按里程碑递增的文件放进同一个目录 | 抄 | 整份需要：变体起新名字与「文件名去编号」相关，C 节列出的规则改动之一 |
| ## 改一个全仓术语：正文之外还有五处会红 | 抄 | 整份需要：变体起新名字与「文件名去编号」相关，C 节列出的规则改动之一 |
| ## 变体起新名字，不用角标 | 抄 | 整份需要：变体起新名字与「文件名去编号」相关，C 节列出的规则改动之一 |
| ## 门禁管哪一半 | 抄 | 整份需要：变体起新名字与「文件名去编号」相关，C 节列出的规则改动之一 |

### 小节清单：`.claude/rules/format-evolution.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 格式演进纪律 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （格式演进纪律 标题之下、第一个下级标题之前的正文：第 3-14 行） | 抄 | 整份需要：与合并里的 27 号 format-constants 相关，C 节列出的规则改动之一 |
| ## 硬约束 | 抄 | 整份需要：与合并里的 27 号 format-constants 相关，C 节列出的规则改动之一 |
| ## 决策正文只写现状，依据写成指针；决策与实验双向登记 | 抄 | 整份需要：与合并里的 27 号 format-constants 相关，C 节列出的规则改动之一 |
| ## 让格式永久化的那个动作，触发点必须与拦它的闸在同一条时间线上 | 抄 | 整份需要：与合并里的 27 号 format-constants 相关，C 节列出的规则改动之一 |
| ## 改一个格式常量：旧值的派生形态要逐类搜，改完登记进 `stale=` | 抄 | 整份需要：与合并里的 27 号 format-constants 相关，C 节列出的规则改动之一 |
| ## 从写开始，不从读开始 | 抄 | 整份需要：与合并里的 27 号 format-constants 相关，C 节列出的规则改动之一 |

### 小节清单：`.claude/rules/mutation-sampling.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 变异没被抓时，先分清是哪一类 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （变异没被抓时，先分清是哪一类 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 没红的三类，判据不同 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 第三类不许当等价变异留档 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 第四类：取样点不敏感还有一种形态——它不判绿，它当场炸 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 第五类：两个口径算同一个量，而它们在基线那一臂上恰好同值 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 第六类：跑前写死的判决只在一个几何 / 旋钮取样点上量过 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 第七类：变异压根没跑——锚点在源码里不再唯一命中 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 第八类：变异跑了但无效——替换编不过，那条行为零覆盖 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 改了一个格式常量之后，要看「无效」那一栏有没有变多 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |
| ## 判据 | 抄 | 整份需要：与合并里的 33 号 mutation-tables 相关，C 节列出的规则改动之一 |

### 小节清单：`.claude/rules/three-way-inference.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 推论要三方独立论证 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （推论要三方独立论证 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 适用范围 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 各条腿必须互不重复 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 引 kb 里的条目要整行抄，不许摘句——三处都管 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 判决由主 agent 做，不由投票做 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 多轮：一次打穿不算数，三轮里多数打穿才算 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ### 交岔路时写岔路单，派实验时带上它 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 一条腿只抽一次样不算一次观测——否定结论尤其不算 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 云端腿的报告要分段落盘 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 本地腿缺席时必须显式报告 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |
| ## 给本地腿的提示一律用英文 | 抄 | 整份需要：这一轮三方论证本身依据的规则，且是 C 节要核的规则改动之一 |

### 小节清单：`.claude/rules/implementation-first.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 方案与探讨建在 `crates/` 的实现上 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （方案与探讨建在 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 整份需要：「实现今天的样子」判据出处，C 节列出的规则改动之一 |
| ## 规矩 | 抄 | 整份需要：「实现今天的样子」判据出处，C 节列出的规则改动之一 |
| ## 门禁管哪一半 | 抄 | 整份需要：「实现今天的样子」判据出处，C 节列出的规则改动之一 |

### 小节清单：`.claude/agent-common.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 项目 subagent 的共用约束 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （项目 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |
| ## 派发 | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |
| ## 规则怎么读 | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |
| ## 写 | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |
| ## 不做 | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |
| ## 门禁 | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |
| ## 报告 | 抄 | 整份需要：C 节点名的共用约束改动，逐份核 diff 要用 |

### 小节清单：`.claude/main-agent.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 主 agent：任务入口与职责 | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （主 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 职责 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 禁止 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 一轮怎么开、怎么收 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 派出去之后 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 交回怎么读 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 派发提示怎么写 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 什么时候派哪个 agent | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/kb-scribe.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 书记员（kb-scribe） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （书记员（kb-scribe） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/experiment-runner.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实验执行员（experiment-runner） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （实验执行员（experiment-runner） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/tooling-writer.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 工具实现员（tooling-writer） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （工具实现员（tooling-writer） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/three-way-materials.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 材料员（three-way-materials） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （材料员（three-way-materials） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/prior-art.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 调研员（prior-art） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （调研员（prior-art） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/crash-verifier.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 崩溃一致性验证员（crash-verifier） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （崩溃一致性验证员（crash-verifier） 标题之下、第一个下级标题之前的正文：第 11-16 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/implementation-writer.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实现员（implementation-writer） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （实现员（implementation-writer） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/investigator.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 调查员（investigator） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （调查员（investigator） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`.claude/agents/kb-spec-drafter.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 写回规格起草员（kb-spec-drafter） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （写回规格起草员（kb-spec-drafter） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 输入（主 agent 必须给） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 做什么 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 写范围 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 产出 | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |
| ## 没做什么（固定会有的） | 抄 | 整份需要：C 节点名的 agent 定义改动，逐份核 diff 要用 |

### 小节清单：`CLAUDE.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # singlefs | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （singlefs 标题之下、第一个下级标题之前的正文：第 2-9 行） | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 任务从哪进 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 经验与约定写在哪 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 规则（始终生效） | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 规范从哪来 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 项目本地规则 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 项目本地事实 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 本项目的特殊性 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |
| ## 一句话版本 | 抄 | 整份需要：C 节点名的定义改动，逐份核 diff 要用 |


### 小节清单：`research/prompts/gate-shrink-r1-inventory/report.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁逐格盘点（第二轮第 1 步） | 不抄 | 标题行本身不含正文，引言段与各节各自整段抄出，不重复抄标题行 |
| （门禁逐格盘点（第二轮第 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 交付 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 一、总数 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 二、怎么跑的 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 三、今天的判定 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 四、47 号 research-script-selftests 的四份 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 五、跑的先后有依赖的 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 六、重复与重叠（cells.tsv 第 7 列的汇总，只列事实） | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 七、依附的对象过期或已关的（第 8 列里挑出来的事实） | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 八、什么现象会推翻这份表 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |
| ## 九、没做什么 | 抄 | 整份需要：派发提示要求 report.md 整份进附录 |

### 小节清单：`records/2026-09-28-门禁59号提速与双机分片.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁 59 号提速与双机分片：调度记录 | 不抄 | 派发只点名下面两节，这一级标题本身没被点名 |
| （门禁 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 派发未点名这段引言 |
| ## 课题与出口 | 不抄 | 派发未点名这一节 |
| ## 依赖表 | 不抄 | 派发未点名这一节 |
| ## 全量合并方案（Q14） | 抄 | 派发正文明确点名的两节之一 |
| ## 第二轮：门禁收缩、提炼、三方与测试 | 抄 | 派发正文明确点名的两节之一 |
| ## 顺带发现、不在这一轮改的 | 不抄 | 派发未点名这一节 |
| ## 历史版本 | 不抄 | 派发未点名这一节 |
| ### 2026-09-28 | 不抄 | 派发未点名这一节，且不随「历史版本」带出（父节标不抄） |

### 非 markdown 文件，整份用 --extra 带入附录（过不了 kb-sections.py）

- `research/prompts/gate-shrink-r1-inventory/cells.tsv`：`.tsv` 文件，`kb-sections.py` 处理不了；用 `--extra research/prompts/gate-shrink-r1-inventory/cells.tsv:1-85` 整份原样带进附录，不过滤、不改列（sha256 `74d4bd10ea93787a255cef5ee23b0a41443149f45c7808bcdd0043eba71ea79c`，与派发提示给的一致）。

---

**出处 `.claude/singlefs-ai-sop/rules/sop-first.md:18-32`（整段抄，未转述）**

```markdown
## 每一条拒绝都必须给出下一步

**拒绝的时候必须说清下一步做什么。** `scripts/gate-lint.sh` 每一种拒绝形态都查：

| 形态 | 出路写在哪 | 判据 |
|---|---|---|
| `bad` | 紧随其后的 `howto` | 从 `bad` 那行起 5 行内要有 `howto`（它自己 + 后面 4 行；注释行不占名额）|
| `die` | **它自己的第二个参数**（`die "卡在哪" "下一步"`）| 只带一句话的 `die` 判红 |
| 直接打印的 `✗`（`echo`，或内嵌 python 的 `print`）| 到下一处拒绝之前的那行 `→` | 中间一行 `→` 都没有就判红 |

直接打印的 `✗` 有两种豁免，都要**显式写出来**：循环里逐条列明细的行标
`# gate-lint:detail`（出路写在它的汇总上），汇总行标 `# gate-lint:summary`。

**写不出 `howto` 就先别加这条检查。**

```

**出处 `.claude/singlefs-ai-sop/rules/sop-first.md:44-66`（整段抄，未转述）**

```markdown
## 加门禁或钩子之前，先找已有的

新加一道门禁或一个钩子之前，先查已有的里有没有管同一件事、同一批对象、同一个触发点的：

1. 跑 `python3 scripts/gate-overlap.py --list`（项目里跑装进来的副本：`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`），看已有的门禁与钩子各判什么、钩子挂在哪个触发点上。
2. 有管同一件事的，把新判据追加进它；两份判同一批对象、读同一份输入的，合并成一份。
3. 两份只是共用一段逻辑（读同一张表、用同一套判据），抽成共用的库，两边都调它，不抄第二份。
4. 非单独加不可，在新文件里逐个写明比过哪些、为什么不并进去：`# gate-similar: <已有的文件名> <为什么不并进它>`；一个像的都没有，写 `# gate-similar: 无 <查过哪些>`。
   同一事件上、matcher 有交集的已有钩子，与字面上很像的已有门禁或钩子，每个都要点名，写「无」不算。
5. 确实要留两份相同的一段，在其中一份里写 `# gate-overlap:copy-kept <另一份的文件名> <为什么不抽成共用>`。
6. 新钩子在文件头写 `# hook-events: <事件> …`，列出它要挂的每个事件（只该在某个工具上触发的写成 `<事件>:<工具名>`），并在 `.claude/settings.json` 里每个事件各注册一次，写了工具名的注册在认得它的 matcher 上。

### 谁来查

- **收工时**：收工钩子 `scripts/claude-hooks/gate-reuse-check.sh` 查这个 agent 自己新建或改动的门禁与钩子。没写 `gate-similar` 或 `hook-events`、该点名的没点全、或者整段抄了已有的一份，就拦下收工，要它先自检。
  同一份判定结果，在被拦回来的续跑里只拦一次；结果变了照样再拦。
  主 agent 与子 agent 都要管：项目在 `.claude/settings.json` 的 `Stop` 与 `SubagentStop` 上各注册一次，写法在那个钩子的文件头。
- **提交前**：门禁阶段「门禁查重」（`scripts/gate-overlap.py`）判 diff 窗口里新加与改动的门禁与钩子：写没写 `gate-similar` 与 `hook-events`、点名的是不是已有的门禁或钩子、理由够不够长、该点名的点全没有、加进来的行有没有与已有的整段相同、`copy-kept` 写得对不对。
  「整段相同」的门槛以那个脚本的 `CLONE_MINIMUM_LINES` 为准。
- 门禁阶段「工具层的闸」（`scripts/hooks-registered.sh`）判钩子注册着没有、`hook-events` 里的事件挂全没有、写了工具名的挂没挂在认得它的 matcher 上。

点名的那一份是不是真的最像、不并进去的理由成不成立，门禁判不了，靠 review。

```

**出处 `.claude/rules/implementation-workflow.md:2-5`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/implementation-workflow.md:6-15`（整段抄，未转述）**

```markdown
## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」、本地腿（`three-way-local-attack` 或 `-defense`，主 agent 按 `.claude/rules/three-way-inference.md`「各条腿必须互不重复」定）按分到的那一面找反例或辩护；打中的写回代码，再攻一轮 | 门禁 11 号的 crates-adversarial-review 格判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（checker 档快档，全量按 `.claude/rules/verification.md`）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

```

**出处 `.claude/rules/implementation-workflow.md:16-23`（整段抄，未转述）**

```markdown
## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 11 号的 agent-def-adversarial-review 格判形式（形态照同一道的 crates-adversarial-review 格）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

```

**出处 `.claude/rules/implementation-workflow.md:24-29`（整段抄，未转述）**

```markdown
## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），主 agent 写判决时拿它核腿引的行；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、写判决之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再核——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、被改了几处、腿引的行落没落在那几处。

```

**出处 `.claude/rules/implementation-workflow.md:30-45`（整段抄，未转述）**

```markdown
## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；54、55、57、59 号都登记着。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树；只在 `research/scripts/gate-staged.sh` 起的那一趟里比，直接跑 `gate.sh` 一律照跑），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

```

**出处 `.claude/rules/implementation-workflow.md:46-68`（整段抄，未转述）**

```markdown
## checker 档只在提交时跑，harness 随时跑

两档各是什么、谁跑、带什么前缀，在 `.claude/rules/verification.md`；这里只写实现改动流程里的落点：

| 场合 | 跑不跑 |
|---|---|
| 实现员交回前 | harness：自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装；checker 档一样都不跑 |
| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 54 号（快档：`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 55、57、59 号；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
| 用户要求，或夜间 | checker 档全量：54 号 `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）、其余重型测试；命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |
| 其余任何时候 | checker 档不跑；子 agent 只跑 harness |

**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 checker 档包 `singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而包的范围含它、直接执行它的测试二进制）、54 / 55 / 57 / 59 / 87 号、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-device-streams.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

```

**出处 `.claude/rules/implementation-workflow.md:69-78`（整段抄，未转述）**

```markdown
## 测试与崩溃检测优先多线程

- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。54 号 `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 54 号的那一层内存包装管。驱动只由 54 号 `--full` 调。门禁 59 号照同一份配置按行分：配置判得过、`GATE_MUTATION_ACROSS_MACHINES` 不是 0、没设 `SINGLEFS_GATE_FULL=1` 与 `GATE_MUTATION_START_OVER=1` 时，先交 `research/scripts/mutation-shard-run.sh`：本机没有作数按条记录的行按用时表两堆均分，本机与第二台各跑一趟只判自己那一堆的 59 号，第二台那一份整份经 `research/scripts/run-with-memory-cap.sh` 起、每条变异在里面照 59 号各自经包装，第二台的按条记录拷回本机，经 `research/scripts/crates-mutation-rows.py import` 核过底座指纹与行键再导入；驱动退出之后 59 号在本机整张判一遍（命中记录的复用、缺的现跑），判定输出与单机逐字相同，全绿标记由本机写。这个驱动只由 59 号调。

**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求或夜间，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。

```

**出处 `.claude/rules/verification.md:2-5`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/verification.md:6-20`（整段抄，未转述）**

```markdown
## 定义与名字

三个包，三个名字，全仓只用这三个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **harness 档** | `crates/singlefs-harness` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景 | 改了代码随时跑 |
| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对与运行模式、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例；连同 QEMU（55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被两档调用；它自己的库单测归 harness 档，随时跑 |

归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档，再由代码三方判要不要挪回。
依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 94 号判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；不带里程碑、步号、增补号、并行线号、欠账号（以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`），来历写进文件头的文档注释。`research/scripts/crash-case-check.py` 的 file-names 那一样判，门禁 13 号的 test-file-names 格在真仓上跑它。
不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。

```

**出处 `.claude/rules/verification.md:21-28`（整段抄，未转述）**

```markdown
## harness 档里再分轻用例与耗时用例

- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
- 红了只重跑红的那几条，不整份重跑，轻用例与耗时用例一样：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`，红的是耗时用例也跑得到），经 Bash 起、加内存包装；修完代码也只重跑红的那几条；整份重跑只在上一趟的日志不全时（脚本判红、一条命令都不打），完整的一遍归提交时的整轮门禁。
- 耗时用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 耗时用例：…"]`；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。单条用时看平常跑的时候量到的，不另外单独量。
- 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里判红，门禁 54 号开跑前在真仓上跑它。
- 一条用例一个场景：按参数循环、每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的，拆成一个带参数的函数加每个取值一条 `#[test]`，红了只重跑那一条；确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 的 one-scenario 那一样判，门禁 14 号在真仓上跑它。

```

**出处 `.claude/rules/verification.md:29-37`（整段抄，未转述）**

```markdown
## checker 档自己分快档与全量

| 档 | 命令 | 什么时候 |
|---|---|---|
| 快档 | `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（库与集成测试；装置二进制 `src/bin/` 的内联单测不在快档里，归它们的变异表与实验复跑）；门禁 54 号跑它，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红 | 整轮门禁与每次提交 |
| 全量 | 54 号 `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用；分层照 D13（验证路线） 已定项 9；GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU） | 用户要求或夜间 |

55、57、59、87 号照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。

```

**出处 `.claude/rules/verification.md:38-46`（整段抄，未转述）**

```markdown
## 崩溃枚举用例住哪、怎么登记

- checker 档每个测试文件第一行写它测哪几个模块：`//! checker 档模块：<模块，按 crash、layer0_progress、crash_injection、bad_disk_input、device_log、on_device_modes 的次序用、隔开>`，与它从 `singlefs_checker_tier::` 导入的模块逐个相同；一个都不导入的写 `无（为什么）`。按模块找用例：`grep -l '^//! checker 档模块：.*crash_injection' crates/singlefs-checker-tier/tests/*.rs`。`research/scripts/crash-case-check.py` 的 modules 那一样判，门禁 54 号开跑前在真仓上跑它。

- 写在 `crates/singlefs-checker-tier/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的快档用例不标。
- 共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness 档的 `tests/common*/mod.rs`，不抄第二份。
- 全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`，第三列 `test=singlefs-checker-tier:<测试目标>:<用例函数>`，计数行、`exhaustive=`、`threads=` 按 `research/scripts/admission.py` 文件头的写法；用例打不出的那一项不登记、在注释里写明。
- 池级 checker 与实现的依赖闭包（dev-dependencies 也算）除 `singlefs-format` 之外不相交，池级 checker 的源码零处引 `singlefs_core`（D13（验证路线） 已定项 5，门禁 94 号判）；它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句 94 号不单判。

```

**出处 `.claude/rules/verification.md:47-50`（整段抄，未转述）**

```markdown
## 函数名与类型名不许只由空泛词拼成

`run_cell`、`CellRun`、`check_cell`、`fn run`、`fn get` 这一类名字判违规：每一个词都在 `.claude/naming-vague-words` 里，名字就什么都没说。改成说出跑的是什么、判的是哪条、取的是哪个量（`evaluate_stream_domain`、`count_named`、`perform_commit_step`）。`impl <trait> for <类型>` 块里 trait 规定的方法名不判。门禁 13 号判；还没改完名的文件登记在 `.claude/naming-vague-exclude`，只缩不涨。

```

**出处 `.claude/rules/verification.md:51-54`（整段抄，未转述）**

```markdown
## 崩溃一致性只能靠崩溃点重放验证

把块层所有写请求记下来，在**每一个**可能的崩溃点截断、重放、跑池级 checker 与记录核对器。**没跑过这个的写路径就不算验过**：单测全绿说明不了崩溃一致性。

```

**出处 `.claude/rules/verification.md:55-58`（整段抄，未转述）**

```markdown
## 功能正确性靠模型对拍

在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统（`crates/singlefs-harness/src/model.rs`），把同一串随机操作分别施加到模型和实现上，比结果。这是这个项目唯一的功能对照物，没有现成实现可以拿输出当标准答案。

```

**出处 `.claude/rules/verification.md:59-62`（整段抄，未转述）**

```markdown
## checker 即规范

不变量清单（`.claude/kb/invariants.md`）每加一条，池级 checker 就加一个检查。**「这个格式到底是什么」，答案以 checker 的源码为准，不是文档。**

```

**出处 `.claude/rules/verification.md:63-69`（整段抄，未转述）**

```markdown
## 文件系统特有的反推缺口

这个项目测的多半是对还是不对这种二选一的东西，风险在覆盖够不够：

- **「测试全绿」不等于「实现正确」。** 崩溃窗口可能只有一次写那么宽，没撞上也许只是没遍历到那个崩溃点。要说「崩溃一致性成立」，先说清这一趟本来撞不撞得上：枚举了多少个崩溃点，是不是全部。
- **「checker 没报错」不等于「镜像是好的」。** 也可能是 checker 还没实现那条检查。说这句话之前，先看 `.claude/kb/invariants.md` 里对应那条的实现状态。

```

**出处 `.claude/rules/verification.md:70-85`（整段抄，未转述）**

```markdown
## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 54 号；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 模型对拍跑了、每段都判过 | 74 号；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:`；checker 档测试文件声明模块 | 54 号开跑前跑 `research/scripts/crash-case-check.py --only placement,modules`，红了不起 cargo；47 号跑那份脚本的自证 |
| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 13 号：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `research/scripts/crash-case-check.py --only file-names` |
| 池级 checker 库不依赖实现；harness 档不依赖 checker 档 | 94 号 |
| harness 档一条用例一个场景 | 14 号（跑 `research/scripts/crash-case-check.py --only one-scenario`） |
| 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 33 号 |
| 每道阶段认第一个参数当项目根 | 62 号 |
| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、55 / 57 / 59 / 87 号、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |

**它们管不到的**：harness 耗时用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。

```

**出处 `.claude/rules/path-moves.md:2-7`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则**，不在共享 SOP 里。共享规则在 `.claude/singlefs-ai-sop/rules/`。

`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」冻结的是**内容**：发给模型的提示说了什么、腿答了什么、产物量到什么数。
**文件与目录的路径不在冻结范围内**：一个文件或目录搬了家、改了名，全仓指向它的地方一次改成新路径——`research/prompts/`、`research/results/`、`records/`、变更史与历史版本节、别的会话的记录都算，不留一处旧路径。

```

**出处 `.claude/rules/path-moves.md:8-19`（整段抄，未转述）**

```markdown
## 怎么做

搬迁是低频操作，核验是一次性的，**没有专管搬迁的门禁**：Markdown 相对链接由共享 `gate.sh` 的「链接指向」阶段、治理文档反引号里的路径由门禁 10 号第 4 段兜一部分，`Cargo.toml`、`replay.sh` 登记表、代码与脚本里的路径没人兜，每一步都要自己跑出来看。

1. **搬之前先把旧路径的全部写法搜一遍，把清单留在手边**：整条路径、`kb/` 起的写法、光文件名、不带 `.md` 的名字，四种各搜一次，输出不许截断（`grep -c` 先数、数对上了再看内容）。
2. `git mv` 搬。
3. 按第 1 步那份清单逐处改成新路径。Markdown 链接要按**每个文件自己的位置**重算相对路径，不是照抄仓库根路径——同一条新路径在 `.claude/kb/` 与 `research/prompts/` 下写法不同。
4. **改完按同一份清单再搜一遍，必须 0 命中**；然后逐个打开被改的文件确认新路径指得到东西。⚠️ 「旧路径搜不到了」**不等于**「新路径是对的」。按各自的解析基准逐类核一遍：`Cargo.toml` 的 `path =` 按 crate 根解析，`replay.sh` 登记表的产物列要纯文件名，这两处写成仓库根路径都搜不出来，一个在 `cargo build` 报 can't find bin、一个在复跑报对不上。
5. **说搬迁这件事本身的那一行保留旧路径**：同一行里旧、新路径都在，或带「改前」「搬到」「搬进」「搬迁」「改名为」这类记变更的词——变更史里「改前：链接指向旧路径」照旧是真话。⚠️ 这条要人判，别做成正则：按词判会把含「搬到」二字的**现状句**一并跳过，旧路径就留在正文里了。
6. 判决或核对表里记过的 sha256 会因此对不上：只差路径的不算证据被改，在那一轮的记录里写明哪个文件、改了几处。产物里带路径的，装置源码与留存产物一起改，复跑仍要逐字相同。
7. **门禁、钩子与脚本的自检样本和用例跟着搬**：被搬的路径、包名或目录布局出现在哪道门禁、哪个钩子、哪个脚本的判据里，就把它们的样本（`fixtures/` 下的 red 与 green、自检里造的临时仓）与自检用例一起改成新布局，改完跑一遍自检，每个弄坏开关仍要判红。搬迁让判据的某一支不再有对象（按旧名字、旧目录认的那一支），就把那一支连同它的自检用例、弄坏开关一起删掉，不留绿着的旧用例；要查就去 git 历史里找。

```

**出处 `.claude/rules/path-moves.md:20-23`（整段抄，未转述）**

```markdown
## 按里程碑递增的文件放进同一个目录

同一类随里程碑一份一份长出来的 kb 文件（字节布局表之类）放进一个目录，文件名 `NN-简称.md` 与 `.claude/kb/milestone/` 同号，不在 kb 根目录下并排起 `first-…`、`second-…`。要的是放进一个文件夹，不是把几份合成一份。第一例是 `.claude/kb/layout/`（`01-first-txn.md`、`02-second-txn.md`）。新立这类文件时直接建目录；已经并排起了名的，按「怎么做」那一节搬。

```

**出处 `.claude/rules/path-moves.md:24-41`（整段抄，未转述）**

```markdown
## 改一个全仓术语：正文之外还有五处会红

路径搬迁之外的另一类是**术语改名**（把一个概念的中文名全仓换掉）。它登记在 `.claude/kb/term-renames.md`、由门禁 12 号的 term-renames 格判全仓不再出现旧名，落地时同样要一次做完：正文改完之后还有五处会红。

| 跟着要改的 | 哪道门禁会红 | 怎么做 |
|---|---|---|
| **编号的简称** | kb 里由 doc-lint、kb 之外的 D / E 编号由共享 `gate.sh` 的「编号与简称」阶段判（引用与登记位要逐字一致）；C / I 编号在 kb 之外没有检查 | 简称里含旧名的编号全仓一起改，kb 之外的 C / I 引用要手工全仓搜；**历史类文件里的引用也要改**——引用与登记位不一致就红，与那句话是不是历史无关 |
| **简称的宽度上限 48** | doc-lint | 新名比旧名长时可能撞上限，要当场缩简称并全仓同步 |
| **文件名与指向它的链接** | 共享 `gate.sh` 的「链接指向」阶段 | 文件名含旧名的，按本规则「怎么做」那一节搬；只改链接文字不改文件名，链接当场失效 |
| **决策分项清单** | 20 / 21 号 | 清单是正文的投影且按宽度截断，名字变长会让截断位置变 ⇒ 跑 `21-decision-items-sync.sh --write` 重新生成 |
| **实验页的「影响的决策」回看** | 75 号 | 它的判据是「正文有新增行、而影响的决策表一行没动」⇒ 纯改名也触发。逐行回看、写当天日期与「不受影响：改名只动措辞，结论、数与产物不变」 |

**留存产物分两类，别一把梭**：装置源码在树里、`replay.sh` 判 `exact` 的产物，跟着源码一起换名，换完复跑必须仍然逐字节相同——那是同一份源码重新生成，不是改证据；已归档、要虚机或真设备、别人的产物，一个字节都不许动，连同引它的正文整段留旧名，并登记进 `.claude/term-rename-exempt`。引它的文件名本身不用登记：`research/scripts/sweep-term.py` 把文件名形态、历史里 `research/results/` 下出现过而树里没有的串当已归档产物的名字，不换也不报；改名时把这类串一起换掉了的，换回旧名，40 号才在历史里找得到。分不清两类时看一句话：**这份产物今天跑得出来吗**。跑不出来就不许改它，也不许改引它的那一行——改了那句「整行抄自产物」当场成假话，而门禁 88 号要到下一次跑才说。

**历史类文件同样换名**（`.claude/kb/decisions-history/`、`experiments-history.md`、`records/`）；只有换了就成假话的那一句留旧名，判据与「说搬迁这件事本身的那一行保留旧路径」那一步同形：把旧名换成新名之后那句话还是不是真话（整行抄自已归档产物的、别家的术语与原文引文、记「这个词被改掉了」这件事本身的）。留旧名的逐文件登记进 `.claude/term-rename-exempt`、写明为什么，不整个目录豁免；变更史里要有一条改名条目当索引，写明旧名指的就是今天的新名。

⚠️ **英文标识符与中文术语可以分批改**：只改中文时 `format-const` 标记名与源码里的 `const` 名仍成对，门禁 27 号不红；而 `crates/` 正被别的会话改时，先改中文不会让它编译失败。

```

**出处 `.claude/rules/path-moves.md:42-52`（整段抄，未转述）**

```markdown
## 变体起新名字，不用角标

候选、臂、方案、命题、提问编号有了变体，给变体起一个新名字，不在原名后面加撇号类字符（U+2032、U+2033、U+2034、U+02B9、U+02BA），也不用 ASCII 单引号代替它：

1. 先定这个名字属于哪一族、那一族在哪几份文件里定义与被引用（同一个字母加数字，在两个轮次、两个实验里可能是两件事）。
2. 在那一族里取下一个没用过的号；取号会造成误读的，起一个不超过 8 个汉字的描述性名字。
3. 撞号先查：新名字在那一族的全部文件里零命中。
4. 变体与原名的关系写在新名字的定义那一句里，不写进名字。

门禁 12 号的 prime-marks 格扫全仓（冻结证据也在内），撇号类字符出现一处就判红；ASCII 单引号当角标它判不了，靠写的人与 review。

```

**出处 `.claude/rules/path-moves.md:53-60`（整段抄，未转述）**

```markdown
## 门禁管哪一半

**搬迁这一半没有专管它的阶段**：搬迁是低频操作、核验是一次性的，不值得挂一道常驻阶段。「链接指向」阶段与门禁 10 号第 4 段顺带罩住 Markdown 链接与治理文档里的路径，其余落点靠人。
所以「怎么做」那七步全靠人跑（第 7 步里的自检由各道门禁、钩子与脚本自己的自检跑，改没改全靠人），第 4 步那条「旧路径搜不到了不等于新路径是对的」尤其要自己打开文件看。

**门禁管的是另一半**：术语改名归 `.claude/gate.d/12-doc-forbidden-notations-and-old-terms.sh` 的 term-renames 格，它按 [.claude/kb/term-renames.md](../kb/term-renames.md) 的登记表查全仓不再出现旧名——
那一类是高频、会静静错在几千处、而且肉眼扫不出来，与搬迁不同。

```

**出处 `.claude/rules/format-evolution.md:3-14`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——磁盘格式怎么演进只有本工程需要。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

**在第一个外部用户出现之前，磁盘格式是软的**——随时可以拆了重做，不需要向后兼容。
这是个人项目相对上游项目最大的自由度，要用足。

但**概念模型要慢**：代码和字节扔了不心疼，「空间怎么记账」「快照是什么」
这类心智模型一旦错了，会在被发现之前渗进上百个决定里。

**推论：真正要慎重的是 `kb/decisions/` 下的决策正文，不是 `.rs` 文件。**

```

**出处 `.claude/rules/format-evolution.md:15-22`（整段抄，未转述）**

```markdown
## 硬约束

- 改格式**必须同步更新** `.claude/kb/invariants.md` 和 checker。三者不同步的 commit 一律不收。
- 决策变更**必须记进**决策变更史（原文写进 `.claude/kb/decisions-history.md` 对应决策的节里，
  组织形态见 `.claude/rules/changelog-format.md`），含推翻依据；正文改 `.claude/kb/decisions/` 下对应文件。
- 一旦有外部用户，本文作废，改为严格兼容——那时直接改本文，
  并把推翻依据写进决策变更史。

```

**出处 `.claude/rules/format-evolution.md:23-55`（整段抄，未转述）**

```markdown
## 决策正文只写现状，依据写成指针；决策与实验双向登记

**决策正文长什么样**（`.claude/kb/decisions/NN-简称.md`）：

- 首行 `## D<n> 简称 —— 状态`，状态后面不加括注：分项计数在索引页，日期与来历在变更史。半定 / 待定的决策例外，要在状态后写「（N 项未定）」（例：「—— 半定（三项未定）」），N 与「### 未定项」的条数一致，只许这一种括注；写了括注时 20 号判条数对不对、34 号的 decision-links 格判形态，没写括注两道都不红，靠写的人守。
- 开头一段写射程：管什么、不管什么。
- `### 已定项` / `### 未定项` 的索引表每行一句话定案，不抄分项正文，不带日期，不超过 100 字；行末照旧带「**状态：已定。**」（20 号要的规范标记，不算字数）。未定项的登记行里还要带两句判定，写在表下那一段里门禁 20 号格「未定项判过改不改新池新建文件的字节」定位不到：「改新池新建文件的字节：否（YYYY-MM-DD，依据：…）」与「动不动格式：不动（YYYY-MM-DD，依据：…）」。两句都是规范标记、不算字数，那两个日期也不算「带日期」。**两把尺量的不是一个集合，不许拿一句的结论填另一句**：前一句量「这一版写不写出不同字节」，取值是 是 / 否 / 无对象；后一句量「它的答案将来会不会改动任何盘上字节，或改动已有字节的解释口径」（D15（格式冻结政策） 已定项 1），取值逐字是「**动 / 不动 / 界不定（视同动）**；**没写过视同动**」——两处都视同动，`界不定` 不是中立的第三档。
- 每个分项一节 `#### 已定项 N：名字`，标题不带日期与来历，正文四块，各以粗体标签开头：
  - `**定案**：` 现行规则，不带日期；
  - `**射程**：` 管到哪、不管什么、已知边角，各一两句；论证、推导、「那个实验量不准」这类说明不进射程——进实验页的「它答不了的」或三方判决；
  - `**依据**：` 只写指针，每条一行——`E<n>（简称）` 证明了什么（一句话，实验页上已有的数不抄）、三方判决文件、用户定案（原话在变更史）。**有实验才有决策**：依据里至少引一个实验；纯政策、没有可量的量的，写「无实验：理由」，门禁把这类分项的数报出来；
  - `**欠**：` `C<n>（简称）`，没有写「无」。
- 移出正文的东西：定案经过、改前原文、「此前……」「立项那天……」这类只在写下那天成立的句子，进当月变更史；推导过程与中间数进依据指到的实验页，实验页里已有的不重复；被否掉的候选的论证留在三方判决或 `records/`。
- 挪的时候不许丢内容：旧正文里删掉的每个数、式子、代码片段，都要在新正文、变更史或被指到的实验页里找得到。改动前后逐分项对一遍，腐烂（与代码、实验、别的决策对不上的现状句）按现查结果改成现值；两条已定条款互相矛盾的，不在瘦身时顺手判，记进 [checks-owed.md](../kb/checks-owed.md) 交用户。

**实验页的「影响的决策」**：每个实验页有一节 `### 影响的决策`，一张表 `| 决策分项 | 关系 | 回看 |`：

- 决策分项写 `D<n>（简称） 已定项 k` 或 `未定项 k`，没有分项的决策、或只作背景提到的，写 `D<n>（简称）`；实验正文（历史版本与这一节之外）提到的每条决策都要有一行，**一条决策撑着几条分项就写几行**——门禁 34 号的 decision-links 格按分项查依据（`decisions[decision]["basis"].get(item)`），一条决策收成一行会让它判不出哪条分项有那个实验。
- **实验必须对应决策**：表里至少一行关系是支撑、推翻或备料；结论作废或退役的实验在标题状态里写明，不判这一条。
- 关系是 支撑 / 推翻 / 备料 / 不影响 之一。支撑、推翻要写到分项，那条分项的 `**依据**：` 要引回这个实验；反过来，分项依据里引的每个实验，实验页里都要有这一行、关系是支撑或推翻。
- 回看写 `YYYY-MM-DD 改了` 或 `YYYY-MM-DD 不受影响：理由`。实验出了新结论（页内历史节或 `experiments-history.md` 记了新条目、改了正文、换了产物）之后，每一行都要重新回看，日期不早于那次变动；写「改了」的，那条决策文件要在同一次改动里。
- 新写的三方判决（`research/prompts/*-main-verification.md`）带一节 `## 回看决策`，同样的表；一条决策都不涉及的，写一行「不涉及决策：理由」。

**实验页的「路径与结论登记」**：一个实验若**同一个量用两条或以上互不共享代码的路径各算一遍再比对**，就要有一节 `### 路径与结论登记`，一张表 `| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |`：

- 路径写它在正文里的名字，逐字一致。**按名字认，不按函数名猜**：路径的名字说的是它怎么算（「字节级摆放」「闭式算术」），不含 `verify`、`check` 这类词，靠函数名扫不出来。
- 源码落点写 `路径:行号 函数名`，路径要在仓里现存。只活在正文散文里、源码里没有对应实现的，写「无实现：理由」——那本身是一条要么补实现、要么别说成「一条路径」的账。
- 共用项那一列写这条路径读了哪些常量、哪些规则、哪份规范，**一个都不许省**：多条路径逐格相等不构成证据，当它们共用同一个错误前提时。写完对每一条共用项问一句「它错了，这几条会不会一起错」。
- 跨路径的比对断言本身也要钉在源码里（`assert_eq!` 之类把两条路径的输出直接连起来比），只是各自算得出、没有一处断言连起来的，在表下写明还欠这一条。
- 结论写成全称的（「对所有 X，A 与 B 相同 / 不同」），那一行的怎么算一栏要写明 X 的取值是**扫遍定义域**还是**几个枚举点**；写枚举点的要给理由。变异测试拦不住量词写错，它只证明「已写下的断言所覆盖的函数会红」，欠账在 C49（全称断言只在抽样点验过）。

**门禁管哪一半**：`.claude/gate.d/34-doc-experiment-pages-and-products.sh` 的 decision-links 格判表的形状、双向对不对得上、回看过没过期、这次改动该回看的回看了没有。还没回填的实验页与决策记在 `.claude/decision-links-pending`，只减不增。「路径与结论登记」一节由同一道的 multipath-registry 格判形状、源码落点存在、共用项不空，还没补的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，只缩不涨。**它们管不到的**：关系判得对不对、回看的理由站不站得住、瘦身时丢没丢内容——这几样靠逐分项对照与抽查。

```

**出处 `.claude/rules/format-evolution.md:56-61`（整段抄，未转述）**

```markdown
## 让格式永久化的那个动作，触发点必须与拦它的闸在同一条时间线上

⇒ **写一条「什么时候起这块字节不能再改」的条款时，同时写清谁在那一刻会被叫醒。**
叫不醒任何人的，要么把时刻挪回提交侧，要么另给一道运行期的闸
（例如写路径第一次写该结构时自检 spec 在不在、指纹对不对，缺就拒绝写）。

```

**出处 `.claude/rules/format-evolution.md:62-74`（整段抄，未转述）**

```markdown
## 改一个格式常量：旧值的派生形态要逐类搜，改完登记进 `stale=`

`.claude/gate.d/27-code-constants-enums-bits-match-kb.sh` 的 format-constants 那一格只核 `const 名字 = 值` 那一行。同一个值在源码与 kb 里还有一串派生形态，常量改了它们不会跟着红：
消息串（`expect("148")`）、按预留宽写的读写（`skip(24)`、`len() - 24`）、下标（`[147]`，即值 − 1）、倍数（`7 × 148 = 1036`）、
商与平方（`⌊16253 / 148⌋ = 109`、`109²`）、测试里钉住的产物值（随字节变的校验和），以及门禁脚本里抄过去的数。

⇒ 改一个格式常量时：

1. 动手前按「字面量、消息串、预留宽的读写、下标、倍数、商与平方、测试与门禁里钉的产物值」逐类全仓搜一遍，输出不截断；
2. 改完把**只可能指旧值**的那几个串登记进它 `format-const` 标记的 `stale=`（`|` 分隔），27 号从此在 kb 正文与 `research/`、`crates/` 下的 `.rs` 里替你盯着；它只扫这两处，别处一概不扫（门禁脚本、`research/results/` 下的产物、规则与 agent 定义、`records/`、`research/prompts/`、kb 的历史节与变更史，以及别的 `.md`、`.tsv`、`Cargo.toml`、`litmus/` 这类），都照第 1 步手工全仓搜。
   裸数字（`148`）不许登记：E145（码 2 自描述头与映射树 key 宽的代价） 的 extent 扇出也是 148，登记了就误拒。登记之前现扫一遍，kb 正文与源码里命中 0 次才登记；
3. 标记写在表格单元格里的，挪到表后单独一行：`stale=` 的分隔符 `|` 会把表格切断。

```

**出处 `.claude/rules/format-evolution.md:75-82`（整段抄，未转述）**

```markdown
## 从写开始，不从读开始

先做只读实现会让你设计出一个「读起来优雅、写起来要命」的格式。
**怎么写决定怎么读**：分配策略、事务边界、COW 顺序都是写路径的产物，
读路径只是跟着指针走。

难题全在写这一侧，先做读是把简单的一半当成进度。

```

**出处 `.claude/rules/mutation-sampling.md:2-10`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则**，不在共享 SOP 里。
共享规则在 `.claude/singlefs-ai-sop/rules/`，变异测试的总纲在那里的 `test-discipline.md`。

`test-discipline.md` 写了两类：**被抓**，和**等价变异**（在所有输入上都跟原式同值，永远抓不到，
判定为等价的把等价性写成一条测试留档）；它在「变异测试证明的是断言会红，不是覆盖」一节第 1 条还提到减法断言编译期溢出会被记成「无效变异」。

这里的编号共八类：第一至三类是变异没红时的三种（「没红的三类，判据不同」那张表的三行），第四至八类是同一个病的另外五种形态，各占一节。

```

**出处 `.claude/rules/mutation-sampling.md:11-18`（整段抄，未转述）**

```markdown
## 没红的三类，判据不同

| 变异没红 | 是什么 | 怎么处置 |
|---|---|---|
| 断言压根没覆盖那段代码 | **真盲区** | 补断言 |
| 在**所有**输入上与原式同值 | **等价变异** | 把等价性写成一条测试留档，再换一个真会改行为的变异 |
| 在**我选的取样点上**恰好同值，换一个取样点就不同 | **取样点不敏感** | **补一个敏感的取样点**，不是留档 |

```

**出处 `.claude/rules/mutation-sampling.md:19-23`（整段抄，未转述）**

```markdown
## 第三类不许当等价变异留档

它有断言、变异也有效，**看起来就是一条抓不到的变异**——最省事的处置是把它记成等价变异留档，
而那一步一做，那个常量从此没有任何东西盯着。

```

**出处 `.claude/rules/mutation-sampling.md:24-33`（整段抄，未转述）**

```markdown
## 第四类：取样点不敏感还有一种形态——它不判绿，它当场炸

前三类说的都是「变异没被抓」。**同一个病还有一种反过来的形态**：
取样点选窄了，**单测全绿、变异也全抓，而实验一跑起来就 panic**。

⇒ **判据与第三类同，只是问句换一个方向**：单测取的那几个点，
**覆不覆盖实验自己会跑到的全部取值**？
`.claude/singlefs-ai-sop/rules/test-discipline.md` 那条「只让多条臂互相比」管的是
**断言之间**的盲区；这一条管的是**取样点与实际取值域之间**的盲区。

```

**出处 `.claude/rules/mutation-sampling.md:34-46`（整段抄，未转述）**

```markdown
## 第五类：两个口径算同一个量，而它们在基线那一臂上恰好同值

这一类连变异都没有，它出现在**两份算术**之间：仓里对同一个物理量有两个口径，
写新算术的人挑了其中一个，而两个口径**在他用来对答案的那一臂上恰好给同一个数**。
于是「基线对上了」这件事本身成了掩护——**没有人会再去查另一臂**。

⇒ **判据**：**把一个数写进产物之前，问一句「这个量仓里是不是已经有人算过；如果算过，他用的分母是不是我这个」。**
- 两个口径都在 ⇒ **不许自己挑一个**：要么按仓里那个算并注明出处，要么**干脆不报这个数**，把这一格留给已有的欠账。
- 只在基线那一臂上对过账 ⇒ **等于没对**。对账要对在**被判的那一臂**上。

⚠️ 与第三类同形（都是取样点恰好同值），差别是它**不需要变异也会发生**，
所以 `research/scripts/mutate.sh` 的输出里一个字都不会说。

```

**出处 `.claude/rules/mutation-sampling.md:47-55`（整段抄，未转述）**

```markdown
## 第六类：跑前写死的判决只在一个几何 / 旋钮取样点上量过

对象不是变异，是**判决本身**：一个实验跑前写死的判据（阈值、翻不翻面）只在装置取的那一个几何或旋钮上被检验过，
而那个几何 / 旋钮不是任何已定条款给的。变异表钉的是单测的取样点（第四类），没有一条变异动几何，判决对几何的敏感性没有任何东西在看。

⇒ **判据**：每个带跑前判决的计数实验报一行「几何敏感性」：至少在一个方向相反的几何 / 旋钮取样点上重跑判决那一格，
翻面就记「不稳定」、不翻就记两点；写「这是政策的性质」「是数不是取法」之前，把那条政策依赖的每个旋钮扫一遍。
判别力自证：把判据阈值挪到两点之间，检查必须由绿转红。欠账在 [checks-owed.md](../kb/checks-owed.md) C317（判决只在一个几何取样点上量过没有检查）。

```

**出处 `.claude/rules/mutation-sampling.md:56-66`（整段抄，未转述）**

```markdown
## 第七类：变异压根没跑——锚点在源码里不再唯一命中

前六类都预设**变异真的被施加了**。还有一类连这一步都没发生：变异表里那一条的「原文」
在对应源码里命中 0 次或多次，`mutate.sh` 在派活之前逐条核锚点，当场退出码 3、**这一轮一条变异都不跑**，
而实验页仍旧写着「N 条变异全抓」——那句话在锚点腐化的那一天就已经复现不出来了。

⇒ **判据**：读 `mutate.sh` 的输出之前，先确认它**跑完了整张表**——
收尾没有「已还原，基线仍全绿」就是中途退出，那一轮的「全抓」一个字都不算数；有这一行而退出码是 5（跑的时候源码被改过），同样不算数，改完重跑。
门禁形态：不跑变异，只对每张表逐条做子串计数，命中不是 1 次就判红并列出表名与条目名
（`.claude/gate.d/33-code-experiment-and-mutation-source-discipline.sh` 的 mutation-tables 那一格，C327（变异表的锚点腐化没有会红的检查） 已还清）。

```

**出处 `.claude/rules/mutation-sampling.md:67-77`（整段抄，未转述）**

```markdown
## 第八类：变异跑了但无效——替换编不过，那条行为零覆盖

`.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」一节第 1 条已经把编译期溢出记作「无效变异」而不是「被抓到」；推开来，替换之后编不过的变异都是这样：既不算被抓、也不算没抓到。
它没写下一步：**一条无效变异等于它本要证明的那个行为今天零变异覆盖**，
要换一条真会改行为的补上，而不是让实验页继续按表里的条数写「N 条全抓」。

**门禁形态**：反斜杠转义这一种成因由门禁 33 号静态判红——两侧变异表的原文与替换文里只有 `\n` 会被还原成换行，
出现别的反斜杠转义就判红，那一段按字面写进源码、编不过。跑起来才发现的编不过由门禁 59 号单独报成「无效」，
与「没红」分成两栏、各带各的出路：无效先按「改了一个格式常量之后，要看「无效」那一栏有没有变多」一节列的来源分——编不过改那一行替换文，点名的名字认不出改那一行的测试名，测试进程被杀先查是谁杀的；没红才是去补一条会红的用例。
读 59 号的输出时按两栏分开数，无效的条数不算进「全抓」。

```

**出处 `.claude/rules/mutation-sampling.md:78-84`（整段抄，未转述）**

```markdown
## 改了一个格式常量之后，要看「无效」那一栏有没有变多

变异表里一条本来好好的条目，会因为**别处改了一个常量**而从「被抓」变成「无效」，而没有任何东西报警。

⇒ **改完格式常量，重跑受影响的每张变异表，比对「抓到 / 无效 / 没红」三个数**，不能只看「没红」是不是 0（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`💥` 不判失败，照样单列、逐条交出去；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；门禁 59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏，每一条按这个次序判：点名的测试那一行是 `FAILED` 记抓到；那一行出现了（跑了没红，或跑到它、跑完它之后进程被杀）记没红；那一行没出现、而输出里有一行 `error:` 开头或 `could not compile` 记无效——替换文编不过，或测试进程在跑到点名的测试之前就被杀（尾巴带 `process didn't exit successfully` 与信号），或点名的名字认不出（名字里带 `$` 这类字符，59 号按字面去找）；都没有记「没跑到」，算没红；无效、没红都让整道判红；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列；`crates/mutations.tsv` 整表复跑是重型，由提交时的崩溃验证员跑 59 号）：
无效那一栏变多，等于有断言被关掉了，而它与「这一条本来就抓不到」在输出里长得一模一样。

```

**出处 `.claude/rules/mutation-sampling.md:85-97`（整段抄，未转述）**

```markdown
## 判据

> **一条变异没被抓时，问一句：换一个取样点，它还同值吗？**

- 答不上来 ⇒ **去找一个会让它不同值的取样点**。找得到就是取样点不敏感，补那一格。
- 确认在所有输入上同值 ⇒ 才是等价变异，按 `test-discipline.md` 留档。

**怎么找那个取样点**：变异改的若是一个出现在**除法或取整**里的常量，就去解
「让两个式子跨过整数边界」的那个输入；改的若是加法项，任何输入都敏感，那说明是真盲区。

⚠️ **这条只在有变异 harness 的仓里生效**，判据落在 `research/scripts/mutate.sh` 的输出上：
它报「一个测试都没红」的那些条目，逐条按「没红的三类，判据不同」那一节三分，**不许直接记成盲区或等价**。

```

**出处 `.claude/rules/three-way-inference.md:2-5`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则，而且只在本机生效——不进上游 SOP。**
共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/three-way-inference.md:6-18`（整段抄，未转述）**

```markdown
## 适用范围

**凡是「推论」，都要走三方。** 推论指：设计判断、取舍结论、
以及任何用来推翻或确立 `.claude/kb/decisions.md` 里某一条的依据。

**不适用**：查一个事实（跑命令、读文件、看源码）、算术、
以及本次对话里刚确认过的东西。那些是观测，不是推论。

⚠️ **最容易漏掉的一类：从自己跑的实验里推出的结论。**
跑实验、读数字是**观测**，不用三方；但**「这个数说明了什么」是推论**——
一旦它被用来推翻或确立 `.claude/kb/decisions.md` 里的某一句话，就必须走三方。
⇒ **「数字是我自己跑的」不构成免走三方的理由，反而是最需要的场合。**

```

**出处 `.claude/rules/three-way-inference.md:19-37`（整段抄，未转述）**

```markdown
## 各条腿必须互不重复

「三方」是这套流程沿用的名字；一轮派三条推论腿：云端攻方（Opus）、云端正推或云端辩方（Sonnet，一轮一条）、本地攻方或本地辩方（一轮一条，派哪一条由主 agent 按这一轮的攻击面与要复核的判决定，在正文分工表里写明理由）。

| 腿 | 用什么 | 怎么调 |
|---|---|---|
| 本地攻方 | Qwen3-Next-80B-A3B-Thinking-AWQ-4bit，经 `~/code/ai-center` 网关（`:8200`，模型 id `local`） | 派 `three-way-local-attack`，由它写提示、调 `bash research/scripts/ask-local.sh <提示文件>`，**提示用英文写** |
| 本地辩方 | 同一个本地模型，另一份提示 | 派 `three-way-local-defense`，同上 |
| 云端正推或云端辩方 | Sonnet | 派 `three-way-forward` 或 `three-way-defense`（定义里 `model: sonnet`） |
| 云端攻方 | Opus | 派 `three-way-attack`（定义里 `model: opus`） |

**「不能重复」指的是各条腿要拿到不同的切入角，不是只换个模型名。**
每条腿要被指定一个不同的立场（例如：正推 / 反推 / 找反例），
正推腿与攻方腿分担 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 三步里的正推与反推，第三步校验由主 agent 判决时逐条现查（「判决由主 agent 做，不由投票做」一节）。
这一轮派本地攻方时，它与云端攻方同为攻方，**两条攻方腿的攻击面要在正文的分工表里分开写**；写不出两组不重叠的攻击面，这一轮的正文就还没写完。
本地辩方替被判出局或被攻的一方辩护。本地腿按「一条腿只抽一次样不算一次观测」那一节抽样。

**本地腿只问能落成数、能逐格判的题。** 提示里给写死的事实表（每行在核对表里写来源文件与行号），要模型按表格逐格填，不许只答 yes / no。

```

**出处 `.claude/rules/three-way-inference.md:38-114`（整段抄，未转述）**

```markdown
## 引 kb 里的条目要整行抄，不许摘句——三处都管

**射程是三处，不止背景材料**：

| 引到哪 | 例子 |
|---|---|
| 三方论证的背景材料 | 发给各条腿的那份材料 |
| **一份 kb 文件引另一份里的条款** | 落地计划复述一条已定项的口径、决策 A 的正文引决策 B 的分项 |
| **给用户或别的会话的转述** | 收尾报告里复述一条已定项 |

三处的失效机理是同一个，判据也是同一条。

⇒ **做法**：凡是引 kb 的条目（决策分项、挡路条、判据表格、不变量），
**整条抄进去，连同它的 ⚠️ 与括注**。嫌长有两条出路：把整条放进末尾的附录、正文指过去，
或者**只写指路不写数**（「合计见 D22（单元原子性怎么合成） 已定项 7 的字段表」）——
**不许在正文里给它做一个更短的版本**。

**判据**：把引用处那一条与仓里那一条并排放，**有没有一个字是引用处没有的**？
有就是摘句了。

**别用手抄**：`research/scripts/quote-kb.py` 按「文件:行区间 / 文件@标题 / 文件~正则」把条款整段抽出来，
写完**回读产物、与源文件逐字节比对**，对不上退出码 3。
自证会红：`python3 research/scripts/quote-kb.py --selftest` 会强制走一遍「抄漏一行」那条分支。

⚠️ **它保证「抄的没错」，不保证「该抄的都抄了」——这两件事差得很远。**
按行区间取的时候，区间的两端是**写材料的人自己挑的**，脚本只核「挑出来那一段有没有抄漏」。
⇒ **用 `@标题` 取，不用 `:行区间` 取**；例外是分项索引表：`### 已定项` 那一节在多数决策文件里是 `#### 已定项 N` 的父节，`@标题` 会把名下全部分项一起带进来，`quote-kb.py --checklist` 因此退出码 6——索引表在清单里标「不抄」并写明「用 --extra 按行区间取」，行区间从标题的下一行起、到第一个 `####` 之前；被父节带出的子节标「抄」、理由以「随」开头。非用行区间不可时，抄完**看一眼被抄内容的最后一行**
（不是产物最后那行外层围栏——外层围栏的长度按正文里最长的一串反引号定，是变的）：
以 `|---|` 收尾、或内层代码围栏没闭合，就是少了一行。欠账落
[checks-owed.md](../kb/checks-owed.md) C244（机械抽取选错行区间，产物本身没有断裂痕迹）。

**材料要带一张小节清单，让漏掉的那一节留下可见的空格。**
⇒ **写材料时，先把全部小节列成一张表，逐节标「抄 / 不抄 / 为什么不抄」，这张表本身进材料。**
判据是：漏掉的那一节在材料里要留下一行「不抄，因为……」，而不是无声消失。

⚠️ **清单要用 `research/scripts/kb-sections.py` 生成，不要临时用 awk 抓标题行。**
抓标题行的清单里，**文档开头那段正文没有自己的行**——它住在 二级标题之下、第一个三级标题之前，
只表现为那条 二级标题行，而 二级标题行看着就是个标题，随手标「不抄」几乎不需要理由。
脚本给那段正文单独发一行并标出行区间，
自证会红：`kb-sections.py --selftest`，再用 `KB_SECTIONS_NO_PREAMBLE=1` 强制走回旧行为。
一级标题同样要有自己的行：顶上是一级标题的文件（`checks-owed.md`、`invariants.md`、`layout/01-first-txn.md` 等），不给它单独一行，那个一级标题下的正文在清单上就一行都没有——`checks-owed.md` 整张欠账表就住在那里（C273（小节清单不认一级标题），自证用 `KB_SECTIONS_NO_H1=1`）。

⚠️ **附录用 `research/scripts/quote-kb.py --checklist 清单.md 出口.md 取法…` 抽，清单与附录由脚本绑住。**
`--checklist` 逐行核清单里标「抄」的每一节真的在附录里，
对不上退出码 4；反向也核——清单标「不抄」的小节，若它自己的标题行出现在附录里同一个源文件的抄录块中（多半是 `--extra 文件:A-B` 整段带进来的），退出码 6（C320（小节清单标「不抄」而附录里有，没人核） 的检查）。
取法**从清单里标「抄」的行自动生成**，别手写第二份。用 `research/scripts/checklist-specs.py 清单.md --cited 正文.md --out 出口.md [--extra 文件:行区间 …]`，它生成取法后用参数列表直接调 `quote-kb.py`；自证 `checklist-specs.py --selftest`，`CHECKLIST_SPECS_SPLIT_ON_SPACE=1` 强制走回拆词的旧毛病、自检必须判红。
再加 `--cited 正文.md`（`quote-kb.py --checklist 清单.md --cited 正文.md 出口.md 取法…`）：正文里提到的每个 D / E / C / I 编号所在的 kb 文件都要有一张清单，缺一张退出码 5——这是 C234（三方材料的小节清单只覆盖被判那一项所在的文件） 第 ① 条的会红检查。⚠️ 它只抓正文里**字面出现**的编号：闸门问题住在一个正文从没提到的文件里时，它一个字都说不出来。

⚠️ **机器生成的清单不许再过滤。**
⇒ 清单要么整份进材料，要么不叫机器生成。**「生成了」与「没被自己删过」是两件事**，
与 [checks-owed.md](../kb/checks-owed.md) C244（机械抽取选错行区间，产物本身没有断裂痕迹） 同族：
两者都是机械抽取之后**人又插了一手**，而产物本身看不出被插过。

⚠️ **射程按「这一轮要引的每个 kb 文件」算，不是「被判那一项所在的那一个文件」。**
一轮论证的闸门问题常常一半住在别的决策里，而那一半没有清单罩着。

⚠️ **正文没点名的枢纽条款，`--cited` 看不见。** 它只抓正文里字面出现的编号；一句写成「X 已经如何」的括注，
若 X 其实是一笔欠账的题面，正文里不会出现那个编号，清单与附录都不会有它，三条腿只读附录就一起继承。
⇒ **写材料时把正文里每一句「已经如何」当成待查的引用**：拿它的动词全仓 grep 一遍，命中的欠账行进 `--extra`，命中的条款进清单。

⚠️ **「不抄」的理由要能被核。**
⇒ 凡是标「不抄，因为它的正文在别处抄了」的行，**写之前去数一眼那个小节在不在**。

⚠️ **判据表也要照这条办：被判那一项自己交代过的问题，必须各占一格。**
⇒ **写判据表之前，先把被判那一项的正文从头读一遍，把每一句「要连同 X 一起答」都摘成一格。**

⚠️ **最容易漏的两种，都不长成「我删了半句」的样子**：

1. **只读结论句、不读它自己的射程句。**
   ⇒ **引一条已定项之前，把它那一节从头读到下一个标题为止**，
   ⚠️ 开头的行一句都不许跳。
2. **条款自己写了「引用时要连这句一起引」，而引用处没连。**
   它们是条款作者留下的显式警告，**优先级高于「嫌长」**。

**门禁拦不住这一条，要说清为什么**（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）：
欠账落 [checks-owed.md](../kb/checks-owed.md) C192（摘句引用没有会红的形式）。

```

**出处 `.claude/rules/three-way-inference.md:115-129`（整段抄，未转述）**

```markdown
## 判决由主 agent 做，不由投票做

- **三方一致 → 继续。** 一致不等于对，但可以往下走。
- **不一致 → 先核实，再讨论。** 核实指去查能分辨它们的那个事实，
  不是再加一个模型来投票。分歧点本身要写进结论里。
- **本地腿交回的每一份样本都读，判决里逐份按路径列出**：干净样本、通读判带损坏的样本、损坏闸留下的作废副本（`-output-void<n>.md`），各标干净或参考；判决里不许只写「作废」「带损坏」「不稳定不采」而不再提它的内容。参考样本与「不稳定」格怎么用，在「一条腿只抽一次样不算一次观测——否定结论尤其不算」与「给本地腿的提示一律用英文」两节。门禁 11 号的 verdict-names-local-samples 格（`.claude/gate.d/11-review-and-sync-records.sh`，新写的判决有没有按路径点名那一轮本地腿的每一份样本）判点名这一半。
- **一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引。** 攻方腿常拷一份装置改臂实测（允许，而且值钱），
  但副本上的数不进 kb：主 agent 把它做成入库装置的新一次跑（跑前登记写明是核实哪条腿的哪句话），逐字段坐实了才引。
- **攻方腿自己提的收严，只在它自己的模型上量过，算「没被攻过」。**
  ⇒ 这类改法可以写回（它补的常是一个没有定义的前提，没有定义比任何一个定义都糟），但交用户时要写明「被攻过零轮」，并给它自己的检查记一笔账。
- **攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的。** 攻方腿的脚本常把故障之外的那几步也固定住（回退之后立刻建几个对象、写几次），一条臂在那几段历史上「一次都不中」，可能只是那个固定的后缀恰好避开了它的落点。
  ⇒ 判一条臂「在同一段历史上不中」之前，把历史里由用户决定的那几步放开扫一遍，只固定前缀与故障；腿的模型只读导入、核对脚本与产物随判决一起入库（例：`c143-r3-main-checks/`，已归档，取法见 `.claude/agent-common.md`「规则怎么读」一节「找不到历史实验的数据」那一条）。

主 agent 的职责是**评判**，不是再当一条推论。

```

**出处 `.claude/rules/three-way-inference.md:130-155`（整段抄，未转述）**

```markdown
## 多轮：一次打穿不算数，三轮里多数打穿才算

一轮攻击打中的东西先挂起，不直接写进正文；同一个结论再攻两轮，三轮里多数打穿才算打穿。三轮各换一组攻击面，提示里明令不许重复前几轮攻过的角度；其中至少一轮派一条辩方腿去复核前一轮的判决——判它够不够得着、是不是同样打中所有替代方案。攻击腿撤回自己上一轮给的方案，比它打中新东西更有价值。

**第三轮之后停，不开第四轮。** 腿提的收严、主 agent 判决里推的组合都算被攻过零轮；第三轮只攻前两轮站住的形态，这一轮里新冒出来的零轮形态不再为它开一轮，写进判决的交用户表、标「零轮」，要么登记实验量代价，要么另立一题。

**核查与判决由主 agent 做，不派子 agent 去核子 agent。** 腿交齐之后、写判决之前，主 agent 自己逐条核：

1. 先跑 `python3 research/scripts/cite-check.py <全部腿报告> --root <快照根> --background <背景材料>`：它判了的（对不上、指到标题行、写成背景材料的行号）照它的判定写进判决；它列成「没判」的与认不出写法的引文，逐处人工核。
2. 腿引的每一行产物，在产物文件里逐字找。
3. 判决要引的复跑数：把腿的模型目录拷到草稿目录，在副本里经 `research/scripts/run-with-memory-cap.sh` 跑，比输出与报告里抄的、比 sha256，不在腿的原目录里跑。
4. 本地腿的译文核对表逐条核：行号在不在、抄的是不是原文、英文丢没丢或多没多限定词与括注。
5. 核不动的（要虚机、要网络、内存包装起不来）在判决里写明是哪一条、为什么，不当核过。

实测有效的三轮分工：第一轮过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱；第二轮第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方；第三轮「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。

攻击结果要主 agent 逐条现查再采纳，不照单全收。

**结论从一边翻到另一边时（两边互调），要三条互不共享前提的验证路径。** 一次算术复核不够，翻回来可能只是换了个方向错；三条路径共用同一个前提时，逐格相等也不构成证据。

**岔路交用户定之前，每条路要有代价数。** 交岔路之前先问「每条路的代价我有数吗」，没有就建一个计数模型实验（纯算术、钉绝对值断言、变异表、进 `replay.sh`，形态照 E109（位置权威三臂的运行时代价）、E110（条带表在连续发布下的期望写放大））跑完再交。模型要在两条臂真的不同的取样点上取样。

### 交岔路时写岔路单，派实验时带上它

**每段交回对着岔路单判够不够。** 判决里要交用户的岔路另写一份 `research/prompts/<轮>-forks.md`，每条岔路一行，四列：候选（各自的定义）、翻面观测（量出什么数会让选择从一个候选换到另一个）、够判条件（量到哪一步这条岔路就能交用户定）、状态（开着 / 够判 / 用户已定）。岔路单只写问题与候选定义，不写倾向、不写已有的数。可以原样发给实验设计员：「设计时不看已有结论」挡的是判决里的结论，挡不到岔路单。实验每段交回，主 agent 逐行更新状态；一行开着的都不剩就停，交岔路表，不再续派。

```

**出处 `.claude/rules/three-way-inference.md:152-155`（整段抄，未转述）**

```markdown
### 交岔路时写岔路单，派实验时带上它

**每段交回对着岔路单判够不够。** 判决里要交用户的岔路另写一份 `research/prompts/<轮>-forks.md`，每条岔路一行，四列：候选（各自的定义）、翻面观测（量出什么数会让选择从一个候选换到另一个）、够判条件（量到哪一步这条岔路就能交用户定）、状态（开着 / 够判 / 用户已定）。岔路单只写问题与候选定义，不写倾向、不写已有的数。可以原样发给实验设计员：「设计时不看已有结论」挡的是判决里的结论，挡不到岔路单。实验每段交回，主 agent 逐行更新状态；一行开着的都不剩就停，交岔路表，不再续派。

```

**出处 `.claude/rules/three-way-inference.md:156-171`（整段抄，未转述）**

```markdown
## 一条腿只抽一次样不算一次观测——否定结论尤其不算

模型的答复是**有变化的观测**，因此 `.claude/singlefs-ai-sop/rules/test-discipline.md`
「单次观测不算数」那条对它成立：同一份提示、同一个模型，两次可以给出方向相反的答案。

⚠️ **两个方向的门槛不一样**，与那条规则同形：

| 结论 | 采信条件 |
|---|---|
| **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
| **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |

⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」。
**几份样本一不一致，按每一格答的是哪件事判，不按行号、行标签、措辞、排版与作答次序判**：先把各份样本按内容对到同一件事上再比；只有一份样本答到的那件事，照只有一份样本办。对齐之后结论不同的格记「不稳定」（`.claude/singlefs-ai-sop/rules/test-discipline.md`「单次观测不算数」）：它不支撑「没打中」，也不算几份样本一致；格里任何一份样本给出的反例、矛盾与算出的数，照「打中了」那一行当线索，由主 agent 现查，判决里逐条写去向（现查坐实而采、现查推翻而不采、核不动），不整格写「不稳定不采」。
云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的：云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告写成 `<轮>-<模型>-s2-output.md`（例 `-opus-s2-output.md`），模型目录写成 `<轮>-<模型>-s2-model/`，不盖掉第一条腿的。

```

**出处 `.claude/rules/three-way-inference.md:172-183`（整段抄，未转述）**

```markdown
## 云端腿的报告要分段落盘

云端腿按提示先把报告写进 `research/prompts/*-output.md`、最后才回复。**提示里还要写明分段写**：
每一次工具调用写进文件的内容不超过 150 行，模型源码与报告都分段追加，第一段排他新建，回复只写短句。
⇒ 看到「超过输出上限」的失败，先 `ls` 产物（多半是空的），按分段写的提示重派，不要续那条腿——它的上下文里没有能用的东西。

**撞了限额、被停或报错的腿，新开一条接着做，不续原来那条。** 失败通知里写着 failed 或 killed 时，先 `ls -la` 看它的报告与模型落了多少：腿在最后一步（把全文当回复返回）才撞限额的，报告文件多半早已完整落盘，完整就直接读、不用接着做。没做完的，跑 `python3 research/scripts/agent-handover.py --agent <它的 id> --out <交接摘要>`，从它的会话记录里机械地抽出派发提示、续做消息、写过的文件与现状、跑过的命令与输出、它最后说的话，再派一条同立场的新腿，派发提示指到交接摘要与已落盘的产物：第一步核现场，已落盘的不重做，接着做完。交接摘要只给新腿读，主 agent 不读它；接续非得经过主 agent 的上下文，就干脆从头重派。续做闸 `.claude/hooks/continuation-guard.sh` 拒绝给最近一次任务通知是 failed 或 killed 的子 agent 续做。**上下文多大不是停、不续的理由**：看门狗报「上下文过大」时，主 agent 看它是不是在原地打转或做派发之外的事，不是就接着跑。

**提示里还要写明：草稿与临时文件放腿自己的目录**（`/tmp/claude-1000/<腿名>/`），不放会话共用的暂存目录。

**提示里还要写明：引 kb 条款时写 kb 文件自己的行号。** 云端腿读的是拼好的背景材料，它顺手记下的行号是背景材料里的行号，贴在 kb 文件名后面就指向一个不存在的位置。⇒ 提示里写「引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数」。

```

**出处 `.claude/rules/three-way-inference.md:184-188`（整段抄，未转述）**

```markdown
## 本地腿缺席时必须显式报告

`ask-local.sh` 取不到 key、网关不通、或正文为空时，**一律报错退出，不许静默跳过**。
少了腿就说少了哪几条腿（网关不通时本地攻方、本地辩方一起缺），然后由人决定要不要在剩下的腿上继续。

```

**出处 `.claude/rules/three-way-inference.md:189-222`（整段抄，未转述）**

```markdown
## 给本地腿的提示一律用英文

本地那条腿是 4-bit 量化模型，**中文输出会退化性复读**（「有效的有效性」「恢复恢复」），
**英文不会**。数据与口径见 `.claude/kb/tooling.md`。

⇒ **提示用英文写，答案也让它用英文回。**

⚠️ **英文提示里的每一句转述，写完都要对着原文核一遍。** 本地腿读不到中文附录，条款只能译成英文转述，而转述正是「引 kb 里的条目要整行抄」那条纪律够不着的缺口：译的人觉得忠实，丢的往往是一个限定词。⇒ 写完英文提示，把每一句转述与附录里的原文并排放一遍，缺一个限定词就补上。**多出来的也要列**：英文比原文多一个限定词、多一个括注，同样在核对表里单列一行、写明为什么加。

**这条不靠自觉**：`ask-local.sh` 里有一道会拒绝的闸——输出判定为字词损坏时
**退出码 5**，并打印下一步。

⚠️ **损坏有四类，签名互不相同，分在两个检测器里**（`corruption-check.py` 查复读、成对标记落单、实词自复读，`oov-check.py` 查拼接）：复读（多吐了）、
成对标记落单（整段掉了，`**Attack P1**:it impossible.**`）、
拼接（两词粘死，`configurationing` `batchinggroup`；**后一个词还可以缺头**——
`inaccessibleisabled` 缺一个字母、`cryptographicord`（cryptographic + [Rec]ord）缺三个，
缺到尾巴不成词时前几条规则全切不开，靠「长前缀 + 既不成词也不是后缀的短尾巴」这条才抓得到；**粘上的还可以是这个词自己尾部的一截**（`reachableachable`，短到 6 个字母的 `anewew`），或者前缀是词表外的派生词（`observationalomputational`）；闸判绿之后照样通读，闸只认得登记过的形态）。第四类是**实词自复读**
（`resetting resetting`，同一个 ≥5 字母的词连着出现两次、中间隔一个空格）——
闸只查长实词（短虚词的自复读在正常英文里合法），不收窄就误报。
`ask-local.sh` 串跑 `corruption-check.py` 与 `oov-check.py`，任一判红即拒绝。
**只跑一个就有一到三类没人查。**

⚠️ **提示里不许用 markdown 强调。**

⚠️ **本地腿用 Bash 的 `run_in_background` 起、结束本轮等完成通知（单次请求最长 900 秒，超过前台上限），命令里不加 `setsid`、`&`、`disown`。**

**闸判红之后**：那一份不算干净样本，照样重跑取下一份，不许记成「三方不一致」——
否则每次都会不一致，这条规则就退化成了摆设。
⚠️ **损坏的不只是词**，带损坏的只当参考样本：`ask-local.sh` 留下的作废副本（`-output-void<n>.md`）与通读判「带损坏」的样本都交主 agent 逐份读；损坏处所在的那一句不用，其余部分给出的反例、矛盾与算出的数，照「一条腿只抽一次样不算一次观测——否定结论尤其不算」那一节「打中了」那一行当线索现查，判决里引它时写明出自参考样本；参考样本不算进「没打中」要的两份，也不拿来与干净样本比一致不一致。

⚠️ **闸报「没跑成」或「没做」时不是通过。** 检测器自己出错（退出码 2）或找不到，与判红（退出码 1）
是两件事，`ask-local.sh` 分开报：前者退出 6、不打正文、正文留成作废副本，判红退出 5。退出 6 就是**这一项没验**，
不许当成验过了。

```

**出处 `.claude/rules/implementation-first.md:2-8`（整段抄，未转述）**

```markdown

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的实现住在哪、设计讨论从哪起步。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

盘上格式、分配器、发布路径、恢复与 checker 都有真代码（`crates/` 下五个 crate，三个验证用的包怎么称呼见 `.claude/rules/verification.md`「定义与名字」）。
除了独立验证这类确有需要的场合，一般的方案和探讨都建在实现上。

```

**出处 `.claude/rules/implementation-first.md:9-16`（整段抄，未转述）**

```markdown
## 规矩

1. **谈方案之前先读实现**：候选臂、改法、实验要建的模型，先读 `crates/` 里对应的代码路径；引用写文件名加行号。
2. **改法写成对实现的改动**：「改 `crates/` 里哪个文件的哪一步、加什么」。实现里还没有的，写明「`crates/` 里没有 X」并附 grep 命令与零命中，候选写成以后加在哪一处、要与已经实现的哪几样对得上。
3. **三方论证的背景材料必须有一行「实现今天的样子」前提**，列出读过的 `crates/` 路径与看到的事实，标成主 agent 的观测；没有相关实现时写 grep 命令与零命中。发给各条腿的提示里写明「方案按 `crates/` 今天的实现来谈」。
4. **`research/` 下的装置与模型只用来独立验证**：它们不与实现共用代码，用来核实现、核推论。装置的数与实现对不上时两边都查，不许默认装置对，也不许默认实现对。
5. **例外**：独立验证本身（checker 与核心层分开写、计数模型核实现给出的数、攻方腿自己写的小模型）；实现还没覆盖、也不打算近期覆盖的推演（照第 2 条写明没有）；`research/prompts/` 下的冻结证据不回改。

```

**出处 `.claude/rules/implementation-first.md:17-21`（整段抄，未转述）**

```markdown
## 门禁管哪一半

`.claude/gate.d/11-review-and-sync-records.sh` 的 implementation-premise 格（单跑：`--check implementation-premise`）查第 3 条的形式：标题日期在 `2026-09-17` 及以后的三方论证正文（`research/prompts/_*-body.md`）必须出现 `crates/`。
**它管不到的**：读没读对、改法是不是真按实现写的、实验跑前登记里的模型该不该复用实现——这几样靠人与三方论证的攻方腿。

```

**出处 `.claude/agent-common.md:2-5`（整段抄，未转述）**

```markdown

`.claude/agents/` 下每个定义开工前先读这一份；与定义冲突时以定义为准。
这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。上游门禁的「规则纪律（项目本地）」阶段判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。

```

**出处 `.claude/agent-common.md:6-14`（整段抄，未转述）**

```markdown
## 派发

- **简单任务不派发，自己直接改**，例如修改一下、查询一下、计划一下等，任务单一流程简单的任务。
- **派发任务要规划，尽量同类任务一起做，最大程度避免串行**。
- 只由主 agent 点名派发；你手里没有 Agent 工具，不派 subagent，也不从 Bash 里起 `claude` 会话。
- 轮名、产出文件路径、草稿目录、这一轮的禁读清单都由主 agent 在派发提示里给。定义里写的路径只是形态（`<轮>`），不是实际路径。
- 输入缺一样就不开工，回复只写缺什么。
- **所有派发的subagent的effort不超过high**。如果需要超过那么弹窗告知。

```

**出处 `.claude/agent-common.md:15-34`（整段抄，未转述）**

```markdown
## 规则怎么读

`.claude/agents/` 下的定义都开了 `omitClaudeMd`：项目 CLAUDE.md、它 `@` 的规则、用户级 CLAUDE.md 与主 agent 的私有记忆都不进你的上下文。

- 规则类文件（`.claude/rules/` 与 `.claude/singlefs-ai-sop/rules/` 下的 `.md`）一律整份读，不受读入大小的限制：定义「开工先读：」一行点名的是其中几个小节时，照样读全那份文件，点名的小节是这件活要照做的那几条。
- 「开工先读：」一行点名的别的文件，只读点名的那几个小节：先 `grep -n '^#' 文件` 找到小节的起止行，再按行读。点的是整份文件的，先看它的节标题，挑与这件活有关的节读。
  不整份读。
- 每个定义都照守、不再写进各自「开工先读：」一行的三处：跑命令照 `.claude/singlefs-ai-sop/rules/command-safety.md`「退不回去的操作，动手前先想一遍」「`pkill -f` / `killall` 一律禁用」两节；
  给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「文风要简单自然」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
- 报告与进度记录里的时间只写日期，不写时刻，也不写时区（门禁 12 号判，Write / Edit 由 `.claude/hooks/write-guard.sh` 当场拒）；日期用 `TZ=Asia/Tokyo date +%F` 取，直接 `date` 会差一天。
- 候选、臂、方案、判据、提问编号有了变体，起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），不在原名后面加撇号类角标（U+2032、U+2033、U+2034、U+02B9、U+02BA）；全仓由门禁 12 号判，写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」。
- 派发提示里没给、定义里也没写的项目事实（某份 kb 在哪、某条决策的原文），去仓里现查，不凭印象补。
- **找不到历史实验的数据、提示或产物，去 `git log` 里看。** 上一轮及更早的实验记录不留在工作区：
  这一轮提交之后由下一次提交删掉上一次那批，本轮的留着（`.claude/gate.d/91-archive-past-rounds.sh` 判这一条）。
  仓里因此有一批引用只写文件名、不写路径，那不是坏链接，是已经归档的东西。
  查法：`git log --all --diff-filter=D --name-only -- "*<文件名>*"` 找到删它的那次提交，
  `git show <提交>^:<路径>` 读当时的内容。**读到的是当时的数，不是今天的结论**——
  拿它支撑新结论之前先重新跑一遍（`.claude/singlefs-ai-sop/rules/kb-discipline.md`「2. 每条带出处与状态」一节里「所有旧数据都只是参考」那一条）；
  要推翻早先的结论，按 `.claude/rules/three-way-inference.md` 重走一轮，不是拿旧文件对质。

```

**出处 `.claude/agent-common.md:35-43`（整段抄，未转述）**

```markdown
## 写

- 只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。
- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；tools 里只有 Edit、没有 Write 的，新建文件照下一条「tools 里没有 Write」那样排他写；除了这种排他新建，Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向），不用 python / sed 就地改仓内文件（执行前被拒，见「执行前拒绝的写法」⑧）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你。
- tools 里没有 Write 的（只有 Read / Bash、只有 Read / Edit / Bash，或另带 WebFetch、WebSearch 这类不写文件的工具）：新建文件一律排他，`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告；之后 `>>` 追加。
- 报告分段写，每一次写进文件的内容不超过 150 行（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；表格与代码块整块放进同一段，不从中间切。
- tools 里没有 Write、Edit 的，改已有文件只用定点替换：`research/scripts/replace-once.py` 或 `research/scripts/replace-batch.py`（先 `--dry-run`），不整份重写。
- 草稿放派发提示给的草稿目录，不放会话共用的暂存目录；用不上可以空着。

```

**出处 `.claude/agent-common.md:44-82`（整段抄，未转述）**

```markdown
## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。`tooling-writer` 例外：按它定义「写范围」一节改 `.claude/agents/`、`.claude/hooks/`、`.claude/rules/` 与 `.claude/settings.json` 的 `hooks` 一节；`.claude/singlefs-ai-sop/` 与 `~/.claude/` 它同样不碰。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：checker 档包 `crates/singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而范围含它、直接执行它的测试二进制）与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置，只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑 harness 档：自己动到的 `singlefs-harness` 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 `cargo test`，按轻阶段对待；逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，checker 档包 `singlefs-checker-tier` 的测试二进制也不许。
- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读（规则类文件除外，照「规则怎么读」一节整份读）；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。上下文到 600k 就停在一个能交接的点交回（定义另写了线的照定义）：报告写做完的、做到一半的（文件与差哪一步）、没开的。
- 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
- **崩溃点测试准确率和正确率优先，其次再衡量时间成本**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由。
- 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。等自己起的后台任务一律结束本轮等完成通知，不写轮询它输出文件的循环（run_in_background 里也拒，见「执行前拒绝的写法」②）。
  等长活时不起缓存计时器，结束本轮直接等完成通知。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令与输出写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、run_in_background 里轮询本会话后台任务输出文件的循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里别的等待循环只记检出，交给主 agent 判断。跑超过 30 分钟的量，每完成一格往草稿目录的 `progress.md` 追加一行（格名、耗时、下一格预计多久），看门狗的整点询问会附上它的末几行。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
    ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
    ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`；run_in_background 里条件或循环体点名了本会话后台任务输出文件（`tasks/<id>.output`）的等待循环，有没有 timeout 都拒。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
    ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
    ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
    ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
    ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：命令位置上的 `sed -i`（目标不在 `/tmp/` 下），python 代码里 `open(<仓内路径>, 'w')` 这一类、`Path(<仓内路径>).write_text(`。仓内文件用 Edit 改，草稿目录照写。
    ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
  - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
  - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
  - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
  - 交回闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh`，挂交回工具 SubagentHandback 与 SubagentStop，只判子 agent）：临时目录里自己建的编译目录、工作树与仓副本还在，交回报告里又没逐个写全路径与为什么不删（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」）。
  - 收工闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh`，挂 Stop 与 SubagentStop）：这个会话新建或改过的门禁与钩子没写 `# gate-similar:` / `# hook-events:`、该点名的已有门禁与钩子没点全、或整段抄了已有的一份（`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」）。
  - 弹窗闸（`.claude/hooks/ask-user-claim-guard.sh`，挂 AskUserQuestion，主 agent 用）：问句或选项说明里一句话带断言词（不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为），同一句里没有出处（反引号里的路径或命令、文件:行号、`research/results/` 下的文件、`name=` 开头的产物行、「实测」「量过」「产物」「输出」旁边带数或路径），也没写「推的」「没量过」「推测」「估计」「粗估」之一。
  - 派发闸（`.claude/hooks/runner-dispatch-guard.sh`，挂 Agent / Task，主 agent 用）：派 crash-verifier、gate-triage 之外的类型，提示里没有一行「重型测试：不跑」；不读共用约束的类型没写「开工先读 `.claude/agent-common.md`」；缺定义 frontmatter `required-inputs:` 要的输入；提示要子 agent 写进它写范围之外的路径；派 `implementation-writer` 没写「要动的 crates 文件：…」、超过 8 个、或与在跑的实现员撞文件；在跑的 opus 子 agent 满 8 个、或同一族还在限额窗口里；派 `kb-scribe` 的规格过不了 `research/scripts/kb-spec-check.py`；派 `experiment-designer` 写准入判输入没变的重跑登记；派 `mutation-triage` 没给变异表；派 `experiment-runner` 没写「这一段回答的岔路：…」（只修锚点、只补落产物除外），续做没写「上一段岔路表里还差：…」或写了一行都不差；派 `kb-scribe`、`implementation-writer` 要改的文件落在还没写判决的三方轮的开工快照里。
  - 续做闸（`.claude/hooks/continuation-guard.sh`，挂 SendMessage，主 agent 用）：给已经交回过（它自己的会话记录，或主会话记录里的交回消息）、或最近一次任务通知是 failed / killed 的子 agent 发消息。
  - 交回正文闸（`.claude/hooks/handback-guard.sh`，挂交回工具 SubagentHandback，只判项目子 agent）：交回正文超过 2000 字；三方腿与核查员交回里点名的报告过不了 `research/scripts/cite-check.py`；书记员与执行员的交回或报告里有没报数、或报 0 项的绿行。

```

**出处 `.claude/agent-common.md:83-91`（整段抄，未转述）**

```markdown
## 门禁

- 扫仓的门禁阶段（本地与共享）只在提交前由 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判。交回前不跑门禁阶段；改了门禁阶段或脚本的，只跑改过的那几份自己的自证与样本（`--selftest`、`.claude/gate.d/fixtures/<阶段文件名>/`）。定义写了开工前的前提检查（`--check <格名>` 只跑一格）的照定义跑；`crash-verifier` 在提交时跑登记给它的那几道，照它的定义。
- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 第二列（agent 名，逗号分隔）是提交前整轮门禁判这一道红时先派给谁修。列出登记给你的：
  `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
- 跑了门禁阶段的，贴每个的原样末行与退出码；重型的那几道（54、55、57、59、87）照「不做」一节重型测试那一条跑：只在提交时或用户要求时、命令带前缀。
- 退出码 77 是「本次未跑」，不是通过。贴绿行时连它报的「查了多少项」一起贴；报「查了 0 项」或根本没报数的，按没判写。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
- 提交前的整轮门禁归 `gate-triage`，不归你。

```

**出处 `.claude/agent-common.md:92-102`（整段抄，未转述）**

```markdown
## 报告

- 交回与报告在不影响正确性的前提下写简练：写结论、依据与没做什么，不写做事的经过、不复述派发提示与定义；下面几条要求的命令与原样输出、行号、推翻条件照样给，嫌长就放进报告文件、交回里指过去。
- 引规则、kb、脚本、代码，写那份文件自己的行号，行号去原文件里现查，不从背景材料里数；没现查过的行号不写进报告，先写「行号待查」；行号用 `grep -n` 或 `awk 'NR==行号'` 现取，不从 `sed -n 'A,Bp'` 的输出里数偏移；贴的命令输出必须是这一次真跑出来的原样；引原文整行抄，不许摘句。
- 能用命令核的事实，贴命令与原样输出；输出不截断，嫌长先数。报告里的条数、阶段数、命中数用命令数出来，不手数。
- 结论写「什么现象会推翻它」。
- 报告末尾有「没做什么」一节：跑不动的、没验的、按定义不归你的，照实列。
- 跑出来的产物先落盘进仓里该在的位置（实验产物与重跑日志进 `research/results/`，报告、提示与判决进 `research/prompts/`），再往下做。停机条款没过、按定义不写实验页、或者主 agent 还没定要不要留的，也要把草稿产物连同「为什么没入库」写进报告——它在哪个 `/tmp` 路径、跑了什么、为什么现在不入库，主 agent 才知道它在哪、还来不来得及拷。门禁 69 号判这一条的形式：装置或变异表改了而 `research/results/` 里没有一份不比它旧的产物、kb 与这一轮新写的提示里把 `/tmp` 路径当依据引用，都红。
- 定义点名的产出文件（跑前登记、腿的 output、样本、运行记录、实验页、代码与 kb 的改动）照定义写进文件。三方论证的云端腿与核查员，报告就是派发提示给的 `research/prompts/<轮>-*-output.md`：用 Bash 分段写进去，交回内容只写文件路径、`sha256sum` 与判定一览（照 `.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」办）。其余定义的「报告」全文写进报告文件：派发提示给了报告路径的写那里，没给的写草稿目录的 `report.md`（用 Bash 时 `set -o noclobber` 排他新建，之后 `>>` 分段追加）；交回只写结论、报告路径与 `sha256sum`，不超过 2000 字（交回正文闸拒超长的），主 agent 按路径存档。交回只调一次（第二次会被拒）。子 agent 用 Write 建文件名以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 `.md` 会被工具层当场拒，Bash 写的不拦。
- 干到一半收到主 agent 的消息：当成追加的输入并进这一轮做，报告里写明在哪一步收到、改了什么；与定义或原输入冲突的，停在那一处交回。

```

**出处 `.claude/main-agent.md:2-4`（整段抄，未转述）**

```markdown

**所有任务从这里进。** 这份只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它），公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

```

**出处 `.claude/main-agent.md:5-8`（整段抄，未转述）**

```markdown
## 职责

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派；一批实现员交回后的合入与验证派一个 `implementation-writer` 做「合入后验证」（派发表那一行）；探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写（派发表「改 `crates/`：探索性的」与「一个阶段任务结束」两行），自己写的 `crates/` 改动并进同一批的代码轮三方。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活照旧临时写提示派 general-purpose。

```

**出处 `.claude/main-agent.md:9-15`（整段抄，未转述）**

```markdown
## 禁止
- **崩溃点测试准确率和正确率优先，其次再衡量时间成本**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，优先保证正确率和准确率。这一条管枚举域，不管跑的时机：什么时候跑按 `.claude/rules/verification.md`，checker 档默认只在提交时跑快档，全量由用户要求或夜间。
- **禁止自行扩大任务范围，只改自己的部分**。一个任务（派出去的一件活）一个出口，达成出口即终止任务，扩大任务必须弹窗；一件活可以关多条问题（「一轮怎么开、怎么收」第 4 条的一簇），每条各有验收标准，出口是全部达成。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**

- **禁止在subagent中跑重型测试**（例外只有 `crash-verifier` 与 `gate-triage` 各跑自己那一部分、命令带 `SINGLEFS_HEAVY_TESTS` 前缀），完整清单以 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节为准。

```

**出处 `.claude/main-agent.md:16-28`（整段抄，未转述）**

```markdown
## 一轮怎么开、怎么收

1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。把这一轮的任务列全，逐个标它依赖谁、输入齐没齐：没有依赖、输入齐的同类任务各成一批、在同一条消息里同时派——调查各派各的调查员，实验各派各的执行员（分开实验），改 `crates/` 的按第 4 条聚簇（统一实现），写回 kb 的照第 10 条一批一份规格；类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类），改 `crates/` 与改门禁不合给一个 agent（写范围不同）。同时派受派发闸 ⑦ 的 opus 并发上限约束，超出的排下一波。有依赖的只串依赖那一环；散的活不排成一串，合成大块。课题、出口与这张依赖表写进这一轮的调度记录（`records/`），第 7、8 条回头对着它判。派之前按这张表问一遍「哪几件能现在一起做」，答不出就还没规划完。
2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在定（判断与定案现在做，改动并进第 4 条最近的一批）；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
3. **重型测试**（哪几样以 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
4. **派实现员之前先聚簇，不为一条发现单派**：把这一轮已抓到、条款已定、要改 `crates/` 的改动全部列出来，按要动的 `crates/` 文件聚簇——文件有交集的并进同一个实现员一次改完（派发提示逐条列问题与各自的验收标准，一个实现员可以关多条，出口是全部达成），互不相交的在同一条消息里同时派、各自的「不碰」清单写上兄弟实现员的文件；同时派几个时一律交补丁（各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `research/scripts/apply-writer-patch.py` 打），`crates/mutations.tsv` 只追加、不算撞文件；份数不设上限。批内冒出的新发现进下一批清单，凑批再派；照第 2 条判为挡着本轮课题的，也并进下一批，不单派；与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件。下一批在上一批合入、取过代码轮快照之后就派，要改快照里文件的等那一轮判决（派发闸 ⑥）。
5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
6. **派一个 agent 之前说清它关的是哪几条已经抓到的问题**（可以不止一条，逐条列）；说不出来的，那条该进记录、不该进本轮。调查、调研类的派发点名的是要回答的问题，不是已抓到的问题。
7. **本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 8 步的收拢表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
8. **收拢要定期做**：每批交回后与每轮结束各做一次，把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，用 SendMessage 发给那个会话（`ListAgents` 列的名字）协商，送不到再弹窗让用户转达。
10. **改、写、验都按批，不按件。** 什么时候一起改：第 4 条聚出的一簇一次改；主 agent 自己写的 `crates/` 改动算进同一批。什么时候一起写：同时待写回的全部定案（几轮判决、几次用户定案）合成一批，出一份起草规格（条目少、主 agent 自己写得清的可以自己写），条目多的按 kb 文件不相交切成几份、同时派几个起草员，起草交回后再按文件不相交切给几个书记员同时写（派发提示写「并行写回」：各自不跑 21、30 号 `--write`、变更史条目「（其N）」由主 agent 派发时分好，主 agent 在全部交回后跑一次 `--write`）。什么时候一起验证：一批实现员全部交回后（半途交回、失败的先合已完成的部分，补派的归下一批）派一个 `implementation-writer` 做「合入后验证」：打全部补丁、编一次、harness 档的测试二进制逐个 `--test` 跑一遍，红了只重跑红的那几条（`research/scripts/rerun-failed-tests.py` 从日志取命令，不整份重跑）（checker 档包的用例不跑，归提交时 54 号；层 0 快档也归提交时），红的照各份报告的算法改钉值，只对合入时改过锚点或冲突的变异行 `prove-red.sh` 证红（实现员各自证过的不重证，整表归 59 号），交回；子 agent 交回前、合入后验证都不跑门禁阶段，扫仓的门禁只在提交前由 `gate-triage` 跑的 `research/scripts/gate-staged.sh` 判（`.claude/agent-common.md`「门禁」一节）；代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮。一件交回就合一件、验一件、开一轮，不许。

```

**出处 `.claude/main-agent.md:29-38`（整段抄，未转述）**

```markdown
## 派出去之后

盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的在执行前拒绝，不碰已经在跑的东西，拒哪几种列在共用约束 `.claude/agent-common.md`「不做」一节「执行前拒绝的写法」那一条（主 agent 同样被拒）。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。

改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。

子 agent 等长活时不续提示缓存；主 agent 除了看门狗报「跑满 N 小时」时的那一条例行询问，不另外定时 ping 它。临时派的 general-purpose 提示里写「开工先读 `.claude/agent-common.md`」（派发闸 ② 要求）；它干带长等待的活时，后台命令照共用约束「长活可以等」那一条写。

叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。收到「跑满 N 小时要主 agent 问一次」就给它发一条例行询问：在做什么、还差几步、在等哪个进程、预计多久，等的已经结束就接着做或交回，并写明把回答写进它草稿目录的 `progress.md`（子 agent 多半没有 SendMessage）；它结束这一轮之后读那份文件，没写就读它会话记录里最后一段正文；发完再起看门狗。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。

```

**出处 `.claude/main-agent.md:39-44`（整段抄，未转述）**

```markdown
## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）：要补的活新派一个同类 agent，派发提示指到它的报告与产物；要它会话里的进度，用 `research/scripts/agent-handover.py` 抽交接摘要给新 agent 读。还没交回、在干活或在等自己后台任务的，照旧能收整点询问与纠正。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。

```

**出处 `.claude/main-agent.md:45-52`（整段抄，未转述）**

```markdown
## 派发提示怎么写

同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数 = 此刻在跑、任务里带重活的 agent 数（三方腿编模型、主 agent 自己起的长命令各算一个，别的会话的不算）；重活数变了，给在跑的 agent 发消息改 N。

派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个带单位的上限（例 16G），照 `.claude/agents/crash-verifier.md`「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个带单位的上限（例 16G），没给用 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。

在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。

```

**出处 `.claude/main-agent.md:53-70`（整段抄，未转述）**

```markdown
## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在有前一轮判决可复核时派）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，主 agent 自己核、自己写判决（`.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」：逐条核腿报告里的原文引用、产物行与复跑命令，代码轮对着开工快照核腿引的行；不派核查员）；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一批交回到齐派「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：派 `crash-verifier` 跑 checker 档：54 号快档、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；层 0 全量只在用户要求或夜间跑，平时不跑） |
| 一批实现员交回到齐、合入之前 | `implementation-writer` 做「合入后验证」（输入给这一批的补丁目录与各份报告；照「一轮怎么开、怎么收」第 10 条：打全部补丁、编一次、harness 档测试二进制逐个 `--test` 跑一遍、红的照报告改钉值、改完只重跑红的那几条、只证合入时改过的变异行）→ 主 agent 审 diff、合入 → 上面那一行的三方 |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：`crash-verifier` 跑 checker 档（54 号快档、QEMU、herd7、crates 变异表） |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧并进「一轮怎么开、怎么收」第 4 条的下一批交 `implementation-writer`（输入给分诊报告），`research/` 那侧主 agent 先写问题单 → `experiment-designer` 写重跑登记 → `experiment-runner`（续派带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句，派发闸 ⑪） |
| 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 11 号文件头 knowledge-sync 那一格，那一格判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：54 号跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求或夜间才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；判绿按输入哈希写全绿标记），55、57、59 同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（55、57 与 54 号并行，59 号与 54 号串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（54、55、57、59 在 `gate.sh` 里只核标记与复用判定，它不直接调）；用户要求时两处都换成 `=user-request` |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-spec-drafter` 起草规格（条目少、主 agent 自己写得清的可以自己写）→ 主 agent 判 → `kb-scribe`（按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个书记员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `research/mutations/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/` |
| 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 11 号的 agent-def-adversarial-review 那一格） |
| 测试或门禁结果与预期不符 | 先派 `investigator` 复现、二分到文件与行（不先改方向和结论）→ 改法并进第 4 条下一批 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner`（登记分了几段、段与段不共用要改的装置与产物的：先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行跑，各写各的产物与报告）→ 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |

```

**出处 `.claude/agents/kb-scribe.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

照规格写，不补内容、不做判断。规格里没写的句子一个字不加；规格与原文对不上（旧串不唯一、找不到）就停下报告；规格点名的文件不在你的写范围里（例如 `CLAUDE.md`），也停下报告，不按规格硬写。
开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；`.claude/rules/format-evolution.md`「硬约束」与「决策正文只写现状，依据写成指针；决策与实验双向登记」两节；`.claude/rules/changelog-format.md`；`.claude/singlefs-ai-sop/rules/writing-discipline.md`。

```

**出处 `.claude/agents/kb-scribe.md:18-27`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
- 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照 `decisions-history.md` 对应决策节里那个日期块现取，其余逐字照给）。
- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改新池新建文件的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或一到十的汉字数字，20、75 号都认）；`.claude/kb/decisions.md` 状态列由第 3 步的 `21-decision-items-sync.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
- 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
- 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
- **规格改了某条分项的「**依据**」段时，同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动**（旧串、新串，一条分项一行）：门禁 75 号那条双向检查两侧要同时到位，而判关系是主 agent 的活、不是你的。规格里没带那几行就停下报告，列出缺哪几行。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你只放行 `.claude/kb/` 与那里）。

```

**出处 `.claude/agents/kb-scribe.md:28-37`（整段抄，未转述）**

```markdown
## 做什么

1. 开工时先记下规格点名的文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash .claude/hooks/write-guard.sh`；检出记录指到草稿，预检被拒不会叫醒主 agent 的看门狗），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
2. 决策变更史：原文写进 `.claude/kb/decisions-history.md` 对应决策的 `## D<n>（简称）` 节里，按 `.claude/rules/changelog-format.md` 组织（按日期分组、日期只出现一次；新日期块插在那一节现状句正下面，同一天有几条改动就列几个 `#### ` 子标题，摘要撞了或原本没摘要才加「（其N）」，取号照那一节该日期块里已用到的最大号取下一个）；标题下两行快查照规格。一条改动点名了几条决策，就在几节里各写一份、内容相同。写完跑 `bash .claude/gate.d/30-decision-history-entries.sh --check shape`：红了看点名的节是不是这一轮改的，是就按它的 howto 改，不是就停下交回。规格改了某条决策的现状（已定/未定项数、一句话现状）时，同一次一并跑 `bash .claude/gate.d/30-decision-history-entries.sh --write` 刷新那一节顶上的「**现状**：」行（这一步只改那一行，不碰历史条目；派发提示写「并行写回」的，这个 `--write` 由主 agent 在全部交回后跑一次）。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：新增的行里有这一轮之外的条目就停下交回，不自己收拾。
3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，把改到的实验源码逐个列给主 agent；`--dry-run` 列出要改写 `crates/**/*.rs` 的（实现的地盘，要走代码轮），停在实跑之前，把那些文件与引用逐个列给主 agent（并进实现批），不实跑；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
   3b. 规格里决策那几处与实验页那几行是一批，一起写、一起回读，不分两次。**判一个实验撑不撑一条分项不在你的活里**，规格没写的关系一个字不加。
   新立分项、改索引行这类不翻状态的规格不走第 3 步，写完正文自己跑一次 `bash .claude/gate.d/21-decision-items-sync.sh --write` 再往下。
4. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判（共用约束 `.claude/agent-common.md`「门禁」一节）；第 2、3 步里的 `--check shape` 与 `--write` 是写回流程的一步，照跑。
5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。

```

**出处 `.claude/agents/kb-scribe.md:38-44`（整段抄，未转述）**

```markdown
## 写范围

- `.claude/kb/**`。
- `relabel-item.py` 自己会改写的引用所在文件：`records/**/*.md`、`research/**/*.md`（`research/prompts/` 除外）、`research/**/*.rs`、`.claude/rules/*.md`（`crates/**/*.rs` 不改：预演列出就停下交主 agent）。这几处只许由那个脚本改，你不手改；`.claude/rules/` 的放行只到这里。
- `/tmp/claude-1000/` 下的报告文件与草稿目录。
- 用 Edit 写的部分与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致；`relabel-item.py` 走 Bash，闸管不到，照上面几条自己守。

```

**出处 `.claude/agents/kb-scribe.md:45-48`（整段抄，未转述）**

```markdown
## 产出

- 报告：规格文件路径、`replace-batch.py` 原样输出、`relabel-item.py` 改了哪些文件与「要人看」清单、第 2、3 步 `--check shape` 与 `--write` 的原样末行与退出码、改过的全部文件与前后 sha256。

```

**出处 `.claude/agents/kb-scribe.md:49-52`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没判定案对不对、没写规格外的句子、没修不是这一轮的红、没提交。

```

**出处 `.claude/agents/experiment-runner.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

照登记做，不改判据。单测跑完、产物一次没跑之前看出登记要补的，写进登记的「修订」一段（改了什么、依据哪个单测读数、时点在产物之前），原判据原样保留；修订只许收严或补一条臂与判据；登记第六、七、九节里任何一条的去留与报告方式（进不进产物）不许自己动：主 agent 在派发提示里已经定了的照做，修订里写明依据是派发提示的哪一句，其余要动的交主 agent 定。产物跑过之后不改登记，交主 agent。
开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「引产物就整行抄」及其子小节「重跑之后要回对正文，复跑绿不代表正文对」；`.claude/rules/mutation-sampling.md`；`.claude/singlefs-ai-sop/rules/code-discipline.md`；`.claude/singlefs-ai-sop/rules/kb-discipline.md`；要写实验页时另读 `.claude/rules/format-evolution.md`「决策正文只写现状，依据写成指针；决策与实验双向登记」**整节**（那张表的形状、四种关系、回看怎么写都在那里，这份定义不抄第二份）。

```

**出处 `.claude/agents/experiment-runner.md:18-25`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 跑前登记路径（`research/prompts/e<号>-preregistration.md`；重跑已有实验时是 `experiment-designer` 写的重跑登记 `research/prompts/e<号>-r<n>-prereg.md`）。登记文件头还挂着「问法待主 agent 在装置写之前删一种」的，不开工。
- 或者只修已有实验的变异表锚点（书记员翻分项状态之后 33 号红）：给表名、源文件与 33 号原样输出。这时不要跑前登记：只把锚点改到源码今天的写法，照第 3 步跑一遍整张表报三个数，其余步骤不做。
- 这一段回答岔路单的哪几行：派发提示里写一行「这一段回答的岔路：…」；续做（实验页已经有了）时再写一行「上一段岔路表里还差：…」，点名上一段交回的岔路表里还开着的行。两行由续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查，缺了派不出来。
- 实验页与索引行要不要这一次写（写的话给简称与状态措辞）。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你放行的仓外位置只有那里）。

```

**出处 `.claude/agents/experiment-runner.md:26-41`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
   1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（例 `research/target/release/e<号>-<简称>`）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；上限先取跑前登记给的，没给再照那一条取；包装退出码 250–254 的那一次输出不算产物。
2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 全写连字符 `e<号>-<英文名里的下划线换成连字符>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-checker-tier/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有四条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 11 号 crates-adversarial-review 那一格的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一格会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判。④ **入库装置先在草稿目录的副本里改，编过再整份换进主工作区**：主工作区那份先 `cp` 进草稿目录（新建的装置直接建在草稿目录里），只在草稿目录那一份上改；改完跑 `bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py <草稿副本> crates/singlefs-checker-tier/src/bin/e<号>_<英文名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进主工作区，编不过一个字节不写，照它报的错误改草稿副本再跑；主工作区里任何时候都只放编得过的版本，不用 Edit、Write 或 Bash 直接写主工作区 `crates/` 下的 `.rs`。交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
3. 入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」。`research/` 那一支写变异表 `research/mutations/e<号>_<英文名>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
   3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名，拼法 `<旧名去掉 .out>-<YYYY-MM-DD>.out`（YYYY-MM-DD 是今天；旧名去掉 .out 之后末尾已经是 `-<日期>` 或 `-<日期>-rN` 的，先去掉那一截再接今天的日期，不叠两个日期；新实验没有旧名，按 `e<号>-<简称>.out` 算），同一天再跑加 `-rN`（`<旧名去掉 .out>-<YYYY-MM-DD>-r2.out`），不写 `.rN.out`；写之前 `ls` 确认没有同名的，跑完按这个新文件名认本轮才有的完成标记。这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下（入库装置那一支编在仓根的 `target/` 下），用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
   4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
   4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（输出不截断），在报告里逐个点名其中表示「没过」的字段：取值是布尔 `false` 的；取值是 `not_run` 的；取值是大于 0 的整数、而名字表示违例、不匹配、歧义、失败的计数（例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）。取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。每个点名写：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0」。
5. 在 `research/scripts/replay.sh` 里登记复跑（`research/` 那一支写一行登记；入库装置照 E156（alloc-basis 四条岔路的代价数）、E158（择根与修复四岔路） 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 `E<号>|@driver_e<号>||<产物>|exact`；函数名先 `grep -c 'driver_e<号>' research/scripts/replay.sh` 现查没用过，同一实验的第二份产物照 E158（择根与修复四岔路） 的先例加后缀，重名的函数 bash 会静默盖掉前一个），跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。登记行改指第 4 步的新文件的同时，这份新文件要被实验页点名（逐字节一致也点一行；门禁 40 号按文件名查）；这一次不写实验页的，把「<新文件名> 还没被实验页点名」写进报告，交主 agent 派人点名。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
6. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判（共用约束 `.claude/agent-common.md`「门禁」一节）；你只跑第 5 步自己那一个实验的 `replay.sh`。整轮门禁判红派回给你修时，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。
7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
   7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增；清单里一条都没有时 ② 不适用，① ③ 照做）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
8. 跑超过 30 分钟的量：每完成一格往草稿目录的 `progress.md` 追加一行（格名、这一格耗时、下一格预计多久），主 agent 读文件不必发消息；看门狗在它超过 30 分钟没改、而你在等后台任务时提前报。

```

**出处 `.claude/agents/experiment-runner.md:42-45`（整段抄，未转述）**

```markdown
## 写范围

- 上面列的 bin 源文件与 `research/e7-index-bench/Cargo.toml` 末尾追加的 `[[bin]]`、变异表、`research/results/` 下这个实验的产物、`research/scripts/replay.sh` 里这个实验的登记行、跑前登记与重跑登记（`research/prompts/e<号>-r<n>-prereg.md`）的「修订」一段、实验页与它的索引行、`.claude/kb/experiments-history.md` 里这个实验的条目、只修锚点时点名的那张变异表、入库装置的变异行（`crates/mutations.tsv` 末尾，只追加这个实验的行，不改别人的行）、`/tmp/claude-1000/` 下的报告文件与草稿目录。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。

```

**出处 `.claude/agents/experiment-runner.md:46-50`（整段抄，未转述）**

```markdown
## 产出

- 报告：单测数（一条命令数出来）、变异三个数（抓到 / 无效 / 没红，数法照 `.claude/rules/mutation-sampling.md`「改了一个格式常量之后，要看「无效」那一栏有没有变多」那一段）与分类，另报 `mutate.sh` 收尾的内存撞顶与超时（不为 0 的整轮已判失败）、产物路径与完成标记、判决行的点名（第 4c 步）、`replay.sh` 结果、实验页路径；登记修订了什么（没有就写没有）。
- 岔路表：登记里岔路单的每一行一行，写「已够判 / 还差什么 / 登记里剩下的量能不能让它翻面」，每格指到产物里算出它的那条命令。主 agent 续派时照这张表点名还差的行；这张表缺了，报告不算交齐。

```

**出处 `.claude/agents/experiment-runner.md:51-54`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没判这个实验的结论能不能推翻或确立决策（那是推论，要走三方）；没跑门禁阶段；没提交。

```

**出处 `.claude/agents/tooling-writer.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

在主工作区改。两种活：一是门禁阶段、钩子、研究脚本、看门狗（`.claude/gate.d/`、`.claude/hooks/`、`.claude/scripts/`、`research/scripts/`）；二是照判决或用户定案改 agent 定义、共用约束与项目规则（`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/`）。
开工先读：`.claude/singlefs-ai-sop/rules/sop-first.md`「每一条拒绝都必须给出下一步」「加门禁或钩子之前，先找已有的」；`.claude/singlefs-ai-sop/rules/command-safety.md`「并行不许把失败吃掉」「子 shell 里的赋值传不回父进程」「脚本改文件之后要回读确认，警告是免费的信号」「进程边界上的三种静默失效」；`.claude/singlefs-ai-sop/rules/preflight-discipline.md` 全篇；`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」；第二种活另读 `.claude/singlefs-ai-sop/rules/rules-discipline.md`「1. 正文只写四样」「7. 正文不许用位置指代，也不许自称」与 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」。

```

**出处 `.claude/agents/tooling-writer.md:18-25`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 关的是哪几条（可以不止一条，主 agent 按要改的文件聚簇，逐条列、各有出口）：`records/2026-09-16-subagent拆分提案.md` 第四十节第 N 行、`.claude/kb/checks-owed.md` 的 C 号或判决里的改法编号，给原文路径。
- 要改的文件，全列；这一轮别的会话在改的文件（你不碰的）。
- 出口：做到哪算完。
- 每条改法要判红的输入样本与弄坏开关名（`<脚本>_BREAK=<项>` 这类）；主 agent 给不出的，写明由你起名。
- 线程上限、内存上限；报告路径与草稿目录，都在 `/tmp/claude-1000/` 下。

```

**出处 `.claude/agents/tooling-writer.md:26-36`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载；动手前跑 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`，看要改的判据有没有已有的门禁或钩子在管。
2. 每条改法先造一个该红的输入：给被改的脚本加弄坏开关或样本目录，跑它的 `--selftest` 或门禁阶段的样本，贴「弄坏开关 → 判红」的原样行；改完再跑一次贴转绿的原样行。说不出该红的输入，这一条停下交主 agent。
3. 改已存在的脚本写同目录临时文件再 `mv`，或用 `research/scripts/replace-once.py` 定点改；不在同一个 inode 上就地改。新建文件排他（Write 前先 `ls` 确认不存在）。
4. 新门禁阶段与新钩子写 `# gate-similar:` 与 `# hook-events:`；新钩子在 `.claude/settings.json` 的 `hooks` 一节注册，自证挂进 `.claude/gate.d/63-agent-write-scope.sh`；新研究脚本写 `admission:` / `run-condition:` 文件头、开头调 `preflight`，自证挂进 `.claude/gate.d/47-research-script-selftests.sh` 的 runner 表。
5. 自证里只对自己起的进程号发信号（`kill "$!"`、`proc.py stop <pid>`），不按名字、cgroup 或进程组发。
6. 第二种活：只写判决或规格给的改法，不另加条款；改完在报告里逐份列出改过的定义、共用约束与规则，交主 agent 开定义三方（门禁 11 号的 agent-def-adversarial-review 那一格）。
7. 收尾只跑这一轮改过的脚本与阶段自己的自证与样本，贴末行与退出码：改过的研究脚本与钩子各跑 `--selftest`；改了钩子或 `.claude/settings.json` 的 `hooks` 一节的，另跑 63 号与 `bash .claude/singlefs-ai-sop/scripts/hooks-registered.sh .`；改过的门禁阶段只跑它们的样本——`.claude/singlefs-ai-sop/scripts/stage-selftest.sh` 整目录跑、不认单道，在仓根下搭一棵只放那几道的阶段目录交给它：`d=<草稿目录>/stage-only; rm -rf "${d:?}"; mkdir -p "$d/.claude/gate.d"; for x in * .[!.]* .claude/*; do case "$x" in .claude|.git|.claude/gate.d) ;; *) ln -s "$PWD/$x" "$d/$x";; esac; done; for x in .claude/gate.d/*; do case "$x" in *.sh) ;; *) ln -s "$PWD/$x" "$d/$x";; esac; done; for s in <改过的阶段文件名…>; do ln -s "$PWD/.claude/gate.d/$s" "$d/.claude/gate.d/$s"; done; nice -n 19 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$d/.claude/gate.d"`；改了定义、共用约束与规则的，另跑规则纪律项目本地那一道（`RULES_LINT_DIR=.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .`）。整道的 47 号、gate-lint、shell-lint、preflight-lint、gate-overlap、doc-lint 与其余门禁阶段由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判，交回前不跑。54、55、57、59、87 号不真跑，改了它们只跑 `--selftest` 或样本。红了先看点名的文件在不在这一轮的改动里；不在的不修，照写。
8. 上下文过 600k：停在最近一个自证全绿的点，报告写做完的条、做到一半的（文件与差哪一步）、没开的，交回。

```

**出处 `.claude/agents/tooling-writer.md:37-42`（整段抄，未转述）**

```markdown
## 写范围

- `.claude/hooks/**`、`.claude/gate.d/**`（含 fixtures 与 `stage-*.tsv`）、`.claude/scripts/**`、`research/scripts/**`、`.claude/settings.json`（只改 `hooks` 一节）、`/tmp/claude-1000/` 下的报告文件与草稿目录。
- 第二种活另加 `.claude/agents/*.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/*.md`。
- 不写 `.claude/singlefs-ai-sop/**`（上游副本）、`.claude/kb/**`、`crates/**`、`research/prompts/**`、`research/results/**`。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。

```

**出处 `.claude/agents/tooling-writer.md:43-46`（整段抄，未转述）**

```markdown
## 产出

- 报告：改过的文件清单与 `git diff --stat -- <这些文件>` 原样；每条改法的判红原样行与转绿原样行；第 7 步各项的末行与退出码；新写的 `# gate-similar:` / `# hook-events:` 行；第二种活改过的定义清单；没做什么。全文写进报告文件，交回只写结论、报告路径与 `sha256sum`。

```

**出处 `.claude/agents/tooling-writer.md:47-50`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没跑重型测试（54、55、57、59、87 号本身、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。

```

**出处 `.claude/agents/three-way-materials.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

正文（问题、判据、跑前条款、「实现今天的样子」那一行）是主 agent 写的，你不改它；你负责让各条腿拿到的原文不漏、不摘句、能核。
开工先读：`.claude/rules/three-way-inference.md`「引 kb 里的条目要整行抄，不许摘句——三处都管」整节；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「喂给多方论证的背景材料，本身要先核」。

```

**出处 `.claude/agents/three-way-materials.md:18-23`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 轮名、正文路径（形态 `research/prompts/_<轮>-body.md`）。
- 要进清单的文件列表（主 agent 知道的）；你查出的另加。
- 代码轮另给 diff 的范围（例：`git diff <基准> -- crates/`），放附录二，形态照已归档的 `_m2-code-r1-diff.md`（取法见共用约束「找不到历史实验的数据」那一条）；别的会话同时在改同一批文件时，主 agent 另给实现员报告里的文件清单与 `crates/mutations.tsv` 追加的变异名，diff 按它们截。工作区改动没有提交点、给不出 `git diff` 的，主 agent 给「文件::项名」清单，项名写全名（函数、类型、`impl 类型名`、测试函数全名）；名字含糊、一个名字对得上两项的，停下要全名，不猜。

```

**出处 `.claude/agents/three-way-materials.md:24-34`（整段抄，未转述）**

```markdown
## 做什么

0. 开工先跑 `bash .claude/gate.d/11-review-and-sync-records.sh --check implementation-premise`（门禁 11 号里判三方论证材料有没有「实现今天的样子」的那一格）与阶段归属表登记给你的其余阶段（共用约束 `.claude/agent-common.md`「门禁」一节）：那一格红、而且点名的就是这一轮的正文，就不开工，回复里原样抄它的 ✗ 与 →；点名的是别的轮次的正文，照写、继续。47 号红时看点名的是哪份脚本：是你要用的 `quote-kb.py`、`kb-sections.py`、`checklist-specs.py`（代码轮还有 `quote-rust-items.py`），停下报告；是别的脚本，照写、继续。
1. 正文里提到的每个 kb 文件、以及正文每一句「已经如何」按动词全仓 grep 出来的条款所在文件，都用 `python3 research/scripts/kb-sections.py 文件…` 生成清单；清单不许再过滤。
2. 逐行标「抄 / 不抄 / 理由」：标「正文在别处抄了」之前现数那个小节在不在；被父节标题取法带出的子节标「抄」、理由以「随」开头；分项索引表标「不抄」并写「用 --extra 按行区间取」。标题里带冒号的小节，`@标题` 取法会在冒号处切错：那一行标「抄」、理由以「随」开头，再用 `--extra 文件:行区间` 取同一段。
3. 用 `python3 research/scripts/checklist-specs.py 清单 --cited 正文 --out 附录 [--extra 文件:行区间 …]` 抽附录；退出码非 0 就按它给的下一步改清单再抽，不绕过。
4. 代码轮先写 `_<轮>-diff.md`（附录二）：文件头写基准与生成日期，输入给的 diff 原样放进代码块，再附新文件全文，形态照已归档的 `_m2-code-r1-diff.md`（取法见共用约束「找不到历史实验的数据」那一条）；给的是「文件::项名」清单时用 `python3 research/scripts/quote-rust-items.py 文件::项名 …` 整段抽（每段前自带文件名与行区间，回读逐字节比对），不手挑 `awk` 行区间。它不并进背景材料，各腿按正文里写的路径去读。
5. 拼背景材料：正文 + 清单 + 附录，排他新建。顺序固定，主 agent 派发时写成别的顺序（例如把 diff 并进来）照定义拼、回复里写明。`.tsv`、`.sh` 这类非 markdown 文件过不了 `kb-sections.py`，整份用 `--extra 文件:1-末行` 带进附录，清单里写明。
   5b. 写开工快照：清单里每个 kb 文件一行进 `research/prompts/<轮>-snapshot/kb-sha256.txt`，代码轮 diff 里的每个 `crates/` 文件一行进同目录的 `crates-sha256.txt`（`sha256sum` 原样输出，路径从仓根起），排他新建；这个目录已经在的不动，回复里写明。门禁 11 号的 implementation-premise 格查新轮次有没有这个目录、清单里的 kb 文件在不在里面。
6. 标「不抄」的每一行与理由留在清单文件里，回复不列；主 agent 要过目就读清单文件。

```

**出处 `.claude/agents/three-way-materials.md:35-38`（整段抄，未转述）**

```markdown
## 写范围

- `research/prompts/_<轮>-checklist.md`、`_<轮>-appendix.md`、`_<轮>-background.md`、代码轮的 `_<轮>-diff.md`、`research/prompts/<轮>-snapshot/` 下的两份 sha256 清单。辅助脚本放草稿目录，每改一版起新文件名，不在原文件上改。除此之外不写；正文不动。

```

**出处 `.claude/agents/three-way-materials.md:39-42`（整段抄，未转述）**

```markdown
## 产出

- 回复：三份文件路径（代码轮四份）、快照目录与两份 sha256 清单各几行、清单行数（抄 / 不抄各多少）、`checklist-specs.py` 的原样末行与退出码；「不抄」行不进回复。回复不超过 2000 字（交回闸拒超长的）。

```

**出处 `.claude/agents/three-way-materials.md:43-47`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 不判正文问得对不对；机械抽取保证「抄的没错」，不保证「该抄的都抄了」，这一句照抄进回复。
- 正文里没有 kb 路径、也没有 D / E / C / I 编号时，`--cited` 一处都核不到，「已经如何」按动词 grep 那一步没有任何机械检查兜底，照写。

```

**出处 `.claude/agents/prior-art.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

别家怎么做是线索，不是证据：只交能核实、能重跑的事实，不写「所以我们该怎么做」。
开工先读：`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」；`.claude/singlefs-ai-sop/rules/kb-discipline.md`「2. 每条带出处与状态」；`.claude/singlefs-ai-sop/rules/verify-before-claiming.md`。

```

**出处 `.claude/agents/prior-art.md:18-22`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 要查的问题，以及要比的对象（哪几家、哪个版本或源码树）。
- 报告路径与草稿目录。

```

**出处 `.claude/agents/prior-art.md:23-30`（整段抄，未转述）**

```markdown
## 做什么

0. 要查本机源码树的，先跑门禁 10 号外部引用那一格：`nice -n 19 bash .claude/gate.d/10-references-and-invariants.sh --check citations`。它会报哪些承重引用指向的源码树或文献不在本机了，那几棵树上的事实写「本机核不动」。这一格只核 kb 里登记过的那批断言，不替你回答这一次的问题。
1. 本机有源码树的先查本机（贴 `grep -r` 命令、命中计数与文件:行）；没有的查官方文档或源码仓，贴 URL 与取得日期。
2. 每条事实写：出处、日期、实测还是读文档、口径；标「未在本项目验证」。
3. 每条写一处它与本工程的已知差异（负载、盘上格式、并发模型、兼容包袱其中之一）；差异写在一组事实背后的做法上，同一个做法的几个数合写一处；写不出就标「差异不明，只能当线索」。
4. 「没有任何现役实现这样做」这类反证单列，写清查了哪几家、怎么查的。

```

**出处 `.claude/agents/prior-art.md:31-34`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件、草稿目录。不改 `.claude/kb/prior-art.md`，要进 kb 由主 agent 交书记员。

```

**出处 `.claude/agents/prior-art.md:35-38`（整段抄，未转述）**

```markdown
## 产出

- 一张表：事实 / 出处与日期 / 口径 / 与本工程的差异；然后反证清单与「没做什么」。

```

**出处 `.claude/agents/prior-art.md:39-42`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没论证它适不适合本工程；没在本项目里验证。

```

**出处 `.claude/agents/crash-verifier.md:11-16`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「重型测试只在提交时跑」里最重的那几道。
开工先读：`.claude/skills/crash-test/SKILL.md`「判读纪律」；`.claude/rules/verification.md`「崩溃一致性只能靠崩溃点重放验证」「文件系统特有的反推缺口」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「阴性结果要能和「代码没跑到」分开」。

```

**出处 `.claude/agents/crash-verifier.md:17-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
- 54、55、57 号各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-device-streams.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
- 54 号跑哪一档：默认快档（`cargo test --release -p singlefs-checker-tier --lib --tests` 加逐条核全绿标记，`.claude/rules/verification.md`），派发提示什么都不写就是它；要跑全量的写明「54 号带 --full」与在哪棵 worktree 里跑（HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建）、命令前缀 `SINGLEFS_HEAVY_TESTS=user-request`；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
- 报告路径与草稿目录。

```

**出处 `.claude/agents/crash-verifier.md:25-35`（整段抄，未转述）**

```markdown
## 做什么

1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 54 号，等它结束。55、57、59 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（55、59 号查 `crates`，55 号按名字读的 `research/results/` 产物这里不查，由整轮门禁里的 87 号兜；57 号查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层；要没有输出，别处的未跟踪文件不挡；57 号读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
   1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
   1c. 54 号 `--full` 开跑前在那棵 worktree 里跑 `bash research/scripts/layer0-shard-configuration-check.sh --emit-assignments <worktree>` 与 `awk -F'\t' '$1 ~ /^crash-case:/ && $3 ~ /shard=across-machines/ { print $1 }' <worktree>/.claude/gate.d/stage-inputs.tsv`，两样原样抄进报告：前一条退 0 是双机分片开着，抄它打的 `PEER_SSH_HOST=` 与 `PEER_MEMORY_CAP=` 两行（第二台那一片的内存上限：驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 用它起那一片，本机那一片与 merge 由第 1b 步的包装管），退 1 是关着、照单机跑，抄它那一句原因；后一条列出的是开着时两台各跑一片的用例。驱动只由 54 号 `--full` 调（`--merged-log`），不单独跑它。
2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段：55、57 号与 54 号并行起（各自后台、各自内存上限，照第 1b 步），59 号等 54 号跑完再起（整表复跑与层 0 全量争 CPU），其余轻阶段一次一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认跑快档（`bash .claude/gate.d/54-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「54 号带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑全量（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记耗时与退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（新池新建文件的干跑） 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。双机分片跑的那几条另抄驱动打的 ①（两台工具链相同）、③（两台输入指纹相同）、④（两片开跑那一行，写着第二台那一片经 `run-with-memory-cap.sh <上限>` 起）、⑤（账本拷回）、⑥（三份发现日志）各一行；判绿要这几行都在、54 号判 merge 那一趟的日志绿；第二台那一片退 250–254 是内存包装自己的结局，照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条办，那一条用例这一趟的判定不算。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus Cargo.toml Cargo.lock research/scripts | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

```

**出处 `.claude/agents/crash-verifier.md:36-39`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件、草稿目录，与为 54 号 `--full` 建的临时 worktree（跑完删）。阶段自己用的临时目录、编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）与 54 号写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；你不改仓里任何文件。

```

**出处 `.claude/agents/crash-verifier.md:40-43`（整段抄，未转述）**

```markdown
## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行 / 日志路径；末尾「没做什么」。每个阶段的整份输出都写进 `<草稿目录>/<阶段>.log`（前台跑也写），59 号那一份是变异分诊员要的输入。

```

**出处 `.claude/agents/crash-verifier.md:44-48`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没修任何一处红，也不判红是不是这一轮的改动造成的（交主 agent 或 `gate-triage`）。
- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 54 号头部与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。

```

**出处 `.claude/agents/implementation-writer.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

派发提示写「交补丁」的在草稿目录的副本里改、交补丁目录（「产出」一节），不碰主工作区；没写的在主工作区改。主 agent 同时派几个实现员时一律交补丁。你做的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」的第 1 步；第 2 步三方对抗与第 3 步 checker 由主 agent 另派。
开工先读：`.claude/singlefs-ai-sop/rules/show-me-test.md`「新增的测试必须先证明它会红」；`.claude/singlefs-ai-sop/rules/code-discipline.md` 全篇；`.claude/rules/implementation-first.md`「规矩」；`.claude/rules/fs-design.md`；`.claude/rules/verification.md`「定义与名字」「harness 档里再分轻用例与耗时用例」「崩溃枚举用例住哪、怎么登记」（测试文件按测什么起名、一条用例一个场景、checker 档测试文件第一行声明模块）。

```

**出处 `.claude/agents/implementation-writer.md:18-26`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 里程碑文件与步号（或并行线编号，或 `mutation-triage` 报告里要补取样点、补断言的条目，或主 agent 聚成的一簇：逐条列问题，可以不止一条），每条各自的验收标准。一簇里某一条要停下交主 agent（第 6 步）时只停那一条，其余照做，报告分条写。
- 或者只修 `crates/mutations.tsv` 的锚点（`relabel-item.py` 改写了 `crates/` 下的源码之后 33 号红）：给 33 号原样输出与被改写的源文件，这时不要步号与验收标准，只把表里那几行的原文改到源码今天的写法，改完跑 33 号。
- 压着的条款：kb 文件路径与小节标题（不给摘要）。
- 主 agent 读过的 `crates/` 路径，以及这一轮别的会话正在改的 `crates/` 文件（你不碰的）。
- 单独一行「要动的 crates 文件：…」，逐个列全（从仓根起），份数不设上限；派发闸按这一行判与在跑的实现员撞不撞文件（`crates/mutations.tsv` 只追加、不算撞），撞了拒。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你只放行 `crates/`、`litmus/` 与那里）。

```

**出处 `.claude/agents/implementation-writer.md:27-45`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载。
   1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层。
   1c. 动到的测试二进制跑红了，修完只重跑红的那几条：`python3 research/scripts/rerun-failed-tests.py <那一趟的日志> -p <包>` 打印命令（带 `--include-ignored`，耗时用例也跑得到），经内存包装起；不整份重跑（`.claude/rules/verification.md`「harness 档里再分轻用例与耗时用例」）。
2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红（经内存包装，共用约束「不做」一节），记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
   3a. 证红一律用 `bash research/scripts/prove-red.sh --copy <副本> [--memory <上限>] <crate> <变异名…>`：先把变异行写进 `crates/mutations.tsv`（参数带 `-p <crate>` 与 `--lib` / `--test <目标>` / `--bin <名>` 之一），它逐条施加、经内存包装跑、判红、还原；不挑目标的行它整次拒，目标带 layer0 的跳过并列出。不自己写证红脚本。
4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。checker 档不跑（`.claude/rules/verification.md`）：自己新加的崩溃枚举流写进 `crates/singlefs-checker-tier/tests/`，不在交回前跑，随提交时的 54 号快档与之后的全量验；已有的一条都不跑；新测试落在 checker 档包 `singlefs-checker-tier` 里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。checker 档由提交时的崩溃验证员跑（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的 harness 档测试二进制（整个二进制，不按名字挑；checker 档包 `singlefs-checker-tier`的不跑）、第 3 步的证红、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`（照那份脚本现抄，不跑 `check.sh` 本身，它是重型）、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、checker 档的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
   4a. 改了 checker（`crates/singlefs-checker/src/`）某条不变量的判定集合的，报告单列一节「受影响的层 0 流与崩溃枚举用例」：哪几条流、哪几个用例的钉值会跟着变；层 0 快档是重型（整轮门禁的 54 号），集成时不跑，主 agent 把这一节交提交时的 `crash-verifier`。`research/scripts/apply-writer-patch.py` 见补丁动了 checker 而报告没这一节就拒绝打。
5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
   - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
   - 在报告里写出「为什么走不到」（哪条构造保证、哪几个调用点）之后，才许写 `todo!` 或 `assert!`。
     条款没写的不是分支（trait 实现、derive、访问器），不加，逐项列进报告；其余照做。
7. 新加层 0 流或崩溃点重放用例时，报告写明它比已有的流多罩了哪些崩溃状态、多跑了哪一步（重开、挂载、恢复）；与已有流的基线镜像、写表、段序列逐项相同的，不新开全量枚举，只加一条快用例钉住「相同」。
   7a. 新写的崩溃枚举用例（测试函数直接调 `enumerate_layer0` 一族做全量枚举、不是 `quick_tier` 那几个的；或自己逐个造崩溃状态：名字带 `every_crash`、循环里对录制操作取到循环变量为止的前缀去 `apply`、循环里造 `CrashImage`，判法在 `research/scripts/crash-case-check.py`）一律写在 `crates/singlefs-checker-tier/tests/` 里、标 `#[ignore]`，共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness（`.claude/rules/verification.md`「崩溃枚举用例住哪、怎么登记」），报告里给出要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行原文（`test=singlefs-checker-tier:…`），由主 agent 或 tooling-writer 登记；几步的合成流、单跑几秒的，在测试函数上面写一行注释 `// crash-case-check:not-a-crash-case <理由>`。`research/scripts/crash-case-check.py` 判这一条。
8. 上下文过 600k：停在最近一个编得过的点，报告写做完的件与没做的件，交回；不硬撑到被自动压缩。

```

**出处 `.claude/agents/implementation-writer.md:46-49`（整段抄，未转述）**

```markdown
## 写范围

- `crates/**`（输入里标了别的会话在改的文件除外）、`litmus/**`、`crates/mutations.tsv`、`/tmp/claude-1000/` 下的报告文件与草稿目录。不写 kb、不写 `research/`。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。

```

**出处 `.claude/agents/implementation-writer.md:50-55`（整段抄，未转述）**

```markdown
## 产出

- 报告：这一轮写过的文件清单（`crates/mutations.tsv` 另写追加了哪几行的变异名；你自己列；别的会话同时在改 `crates/` 时，`git diff --stat` 分不出谁改的），再附 `git diff --stat -- crates litmus` 原样；每条新测试的「改坏哪一行 → 哪条断言红」、第 4 步那几样的末尾原样输出、停下交主 agent 的设计问题。
- 派发提示写明「交补丁」（在副本里改、不碰主工作区）时：补丁目录 `<草稿目录>/patch/` 里放 `crates.patch`（`git diff -- crates litmus ':!crates/mutations.tsv'`）、`mutations-append.tsv`（要追加的变异行，六段，名字表里没有）、`mutations-replacements.tsv`（整行替换，按第一段的名字找表里恰好一行）、`mutations-delete.txt`（要删的变异名，一行一个）与 `report.md`（这份报告）；用不上的不建。格式以 `research/scripts/apply-writer-patch.py` 文件头为准，主 agent 用它打。
- 全文写进报告文件，交回只写结论、报告路径与 `sha256sum`（交回正文超过 2000 字交回闸拒）。

```

**出处 `.claude/agents/implementation-writer.md:56-59`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没走三方对抗；checker 档（54 号快档与全量、QEMU、herd7、crates 变异表）归 `crash-verifier`；没提交。

```

**出处 `.claude/agents/investigator.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只查不修：复现、定位、给推翻条件；修法与要不要修归主 agent。
开工先读：`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「所有结论要三次推导：正推 / 反推 / 校验」「一个假设要能被观测否掉，否则它不是假设」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/verify-before-claiming.md`「核了窄的那一句，说出口的却是宽的那一句」。

```

**出处 `.claude/agents/investigator.md:18-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 现象原文：产物整行、门禁的 ✗ 与 → 行、种子、跑出现象的命令。
- 要回答的问题：是真是假、机理、最小复现，哪几样。
- 只读，还是可以在草稿目录的仓副本里改（改坏、加打印、二分）。
- 线程上限、内存上限；报告路径与草稿目录，都在 `/tmp/claude-1000/` 下。

```

**出处 `.claude/agents/investigator.md:25-33`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载；编译与跑编译出来的代码照那一节经内存包装、带线程上限。
2. 先原样复现：用输入给的命令在仓副本里（`rsync -a --exclude target --exclude .git`）跑一次，贴命令与原样输出。复现不出就写复现不出，列出与原现场差在哪（提交、环境变量、种子、线程数），不往下猜。
3. 二分到文件:行：缩小输入、改坏或加打印都只在副本里做；每一步贴命令与输出，记下排除了哪几种解释、各凭哪条观测。
4. 给最小复现：一条命令加它的原样输出；再给「什么现象会推翻这个定位」，并在副本里造一次那个现象看它确实会出现。
5. 要跑的东西是重型测试（名字带 layer0 的测试二进制、54、55、57、59、87 号、全量 cargo test）的，不跑，写进报告交主 agent 去问用户。
6. 上下文过 600k：停在最近一个能交接的点，报告写查到哪一步、排除了什么、还差什么，交回。

```

**出处 `.claude/agents/investigator.md:34-37`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件与草稿目录（仓副本里随便改）；不写仓里任何文件。

```

**出处 `.claude/agents/investigator.md:38-41`（整段抄，未转述）**

```markdown
## 产出

- 报告：复现命令与原样输出、定位到的文件:行、最小复现、推翻条件与造它的那一次输出、排除掉的解释与各自的观测、没做什么。全文写进报告文件，交回只写结论、报告路径与 `sha256sum`。

```

**出处 `.claude/agents/investigator.md:42-45`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没修；没判该怎么改；没跑重型测试；副本里的改动不回主工作区。

```

**出处 `.claude/agents/kb-spec-drafter.md:12-17`（整段抄，未转述）**

```markdown

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只起草规格，不写 kb。规格交主 agent 判过之后派 `kb-scribe` 照写。
开工先读：`.claude/singlefs-ai-sop/rules/kb-discipline.md`「1. 每条事实自足」「5. 编号只能做索引，不能做称呼」「8. 正文只写现状，历史放文末的「历史版本」」；`.claude/rules/format-evolution.md`「硬约束」与「决策正文只写现状，依据写成指针；决策与实验双向登记」两节；`.claude/agents/kb-scribe.md`「输入（主 agent 必须给）」一节（规格要给全的那几样）。

```

**出处 `.claude/agents/kb-spec-drafter.md:18-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 判决路径与要写回的条目编号，一次不超过 8 条；超过的按 kb 文件不相交切成几份，主 agent 同时派几个起草员，不分几次串着派。
- 用户定案原话的出处（变更史里引）。
- 要动的 kb 文件；`.claude/kb/decisions-history.md`（决策变更史，组织形态见 `.claude/rules/changelog-format.md`）。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下。

```

**出处 `.claude/agents/kb-spec-drafter.md:25-33`（整段抄，未转述）**

```markdown
## 做什么

1. 逐条读判决里那一条的定案句与它点名的 kb 条款；每条给：目标文件、旧串（原文整行，`grep -cF` 恰好 1 次，贴命令与输出）、新串、依据（判决文件与行号，或用户定案的出处）。
2. 新串里的编号一律写成「编号（简称）」，简称照登记位逐字抄；判决内部的段名、臂名不写成与已登记编号同形的记号。
3. 动了某条分项「**依据**」段、增删了引的实验号的，同一份规格里给那个实验页 `### 影响的决策` 表的配对行改动（旧串、新串，一条分项一行）。
4. 翻分项状态的，给门禁 20、31 号要的句子（「**状态：已定。**」或「**状态：未定。**」，未定项的两句字节判决）；写变更史的，给 `### ` 标题整行与「快查·改前：」「快查·改后：」两行。
5. 规格写成两份：`<草稿目录>/spec.json`（`research/scripts/kb-spec-check.py` 文件头「规格的两种写法」的 JSON 写法）与同内容的 `spec.md`；跑 `python3 research/scripts/kb-spec-check.py <草稿目录>/spec.json`，红了改规格原文直到绿，贴末行。改不绿的那一条写进「要主 agent 判的点」，不硬改原意。
6. 判决里有两种读法、或定案句与 kb 现文互相矛盾的，不替主 agent 选，写进「要主 agent 判的点」。

```

**出处 `.claude/agents/kb-spec-drafter.md:34-37`（整段抄，未转述）**

```markdown
## 写范围

- 只写草稿目录（规格文件、辅助脚本）与报告文件；不写仓里任何文件。

```

**出处 `.claude/agents/kb-spec-drafter.md:38-41`（整段抄，未转述）**

```markdown
## 产出

- 报告：规格文件路径与 `sha256sum`、`kb-spec-check.py` 的原样末行、要主 agent 判的点（每条一句）。全文写进报告文件，交回只写结论、报告路径与 `sha256sum`。

```

**出处 `.claude/agents/kb-spec-drafter.md:42-45`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没写 kb；没跑 kb 门禁阶段（书记员写完之后跑）；规格里的判断以判决为准，没另做推论。

```

**出处 `CLAUDE.md:2-9`（整段抄，未转述）**

```markdown

**一个从零设计的 COW 文件系统，Rust 实现。**
现有 COW 文件系统是**设计输入**（它们的病历和解法），不是移植目标。
**并行agent治理与上游sop治理**
agent治理与上游sop治理，是另外一个重要的任务。因此遇到问题优先从流程、规范和门禁等方面着手解决问题，目的不是改一行代码，而是避免再发。

当前里程碑：**「覆盖写、释放、回退与复用」**（`.claude/kb/milestone/02-second-txn.md`，做到哪一步看那份文件）。上一个里程碑「新池新建文件」（`.claude/kb/milestone/01-first-txn.md`）的出口已经满足，代码在 `crates/` 下五个 crate（格式常量、核心、池级 checker、harness 档、checker 档；两档怎么分见 `.claude/rules/verification.md`）。

```

**出处 `CLAUDE.md:10-17`（整段抄，未转述）**

```markdown
## 任务从哪进

**所有任务从 [.claude/main-agent.md](.claude/main-agent.md) 进**：主 agent 的职责、一轮怎么开怎么收（出口、判阻塞、收拢、再判）、派出去之后怎么盯、交回怎么读、派发提示怎么写、什么时候派哪个 agent 的调度表，都在那一份。这份文件只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实。

@.claude/main-agent.md

改了定义要新派才生效：续做（SendMessage）沿用第一次派发时的定义；新建的定义要等几秒才派得出去。改了本文件（连同它 `@` 的规则）要新开会话才对临时派的 agent（general-purpose 这类）生效：同一会话里派的继承会话开始时那一份；`.claude/agents/` 下的定义都开了 `omitClaudeMd`，根本不读本文件，要它们知道就写进定义或 `.claude/agent-common.md`。计划、实测与现状在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它）。

```

**出处 `CLAUDE.md:18-23`（整段抄，未转述）**

```markdown
## 经验与约定写在哪

别的会话、别的贡献者、派出去的 subagent 可能撞上的坑与约定，写进项目：规则（`.claude/rules/`）、agent 定义与 `.claude/agent-common.md`、kb、`records/`。私有 memory 只放用户个人的偏好（提交时间窗、回复语言这类）：它只在本机，别的贡献者看不到，定义开了 `omitClaudeMd` 的 subagent 也读不到。往本文件加内容之前先问它属于哪个 agent，属于就写进那个定义或共用约束。

subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案.md`，不占 `.claude/kb/` 的 C / D / E 编号，kb 只管文件系统本身。定义、共用约束、三方流程与配套脚本的问题：修它们本身就是这一轮的出口时，直接改，不先问；是做别的任务时撞上的，照 `.claude/main-agent.md`「禁止」一节，记下或弹窗，不自行扩大范围。改完记进那份计划，只跑改过的脚本或门禁阶段自己的自证与样本；门禁归提交时，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh` 判，改完不跑。改了定义与共用约束的走一轮三方或由用户逐份豁免（11 号 agent-def-adversarial-review 那一格判），改了触发文件的写阶段同步记录（11 号 knowledge-sync 那一格判）。

```

**出处 `CLAUDE.md:24-42`（整段抄，未转述）**

```markdown
## 规则（始终生效）

@.claude/singlefs-ai-sop/rules/engineering-philosophy.md
@.claude/singlefs-ai-sop/rules/sop-first.md
@.claude/singlefs-ai-sop/rules/show-me-test.md
@.claude/singlefs-ai-sop/rules/machine-first.md
@.claude/singlefs-ai-sop/rules/code-discipline.md
@.claude/singlefs-ai-sop/rules/writing-discipline.md
@.claude/singlefs-ai-sop/rules/rules-discipline.md
@.claude/singlefs-ai-sop/rules/design-doc-discipline.md
@.claude/singlefs-ai-sop/rules/kb-discipline.md
@.claude/singlefs-ai-sop/rules/test-discipline.md
@.claude/singlefs-ai-sop/rules/evidence-discipline.md
@.claude/singlefs-ai-sop/rules/verify-before-claiming.md
@.claude/singlefs-ai-sop/rules/pushback-discipline.md
@.claude/singlefs-ai-sop/rules/command-safety.md
@.claude/singlefs-ai-sop/rules/session-wrapup.md
@.claude/singlefs-ai-sop/rules/preflight-discipline.md

```

**出处 `CLAUDE.md:43-51`（整段抄，未转述）**

```markdown
## 规范从哪来

| 项 | 值 |
|---|---|
| 上游仓 | `singlefs-ai-sop`，在本机兄弟目录 `../singlefs-ai-sop-zh`。同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的 |
| 项目里的副本 | `.claude/singlefs-ai-sop/`（[README](.claude/singlefs-ai-sop/README.md)），是**拷贝，不是符号链接**；与上游同步靠重新拷贝一份：`rsync -a --exclude '.git' ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/`，拷完 `diff -rq --exclude=.git ../singlefs-ai-sop-zh .claude/singlefs-ai-sop` 确认一致再刷版本戳（`cp -r` 会把上游的 `.git` 一起拷进副本，第二次同步时那些只读 object 报一屏 Permission denied） |
| 版本戳 | `.singlefs-ai-sop-version`。门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红——那是在提醒「规矩变过了，先读再跑」 |
| 怎么改 | 共享规则只能在**上游**改。从本仓的会话改上游**只往文件里填内容**（规则正文、脚本逻辑、三语译文那几行）：不跑 `bump.sh`、不动 `VERSION`、不改 MANIFEST / SOURCE-MANIFEST 与译文首行的溯源哈希、不写带版本号的 CHANGELOG 条目、不提交，这些归做发版的会话；填之前先看上游三语仓 `git status`，有人在发版就等它提交完，填完告诉发版会话填了哪些文件哪几段。发版之后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳。**不许在 `.claude/singlefs-ai-sop/` 里就地改**——下次同步就没了，而且改它们等于改所有项目。上游的改动应当罕见：经常变说明规范本身没设计好；**作业在本仓，不在上游仓** |

```

**出处 `CLAUDE.md:52-67`（整段抄，未转述）**

```markdown
## 项目本地规则

@.claude/rules/fs-design.md
@.claude/rules/format-evolution.md
@.claude/rules/three-way-inference.md
@.claude/rules/mutation-sampling.md
@.claude/rules/implementation-workflow.md
@.claude/rules/verification.md
@.claude/rules/implementation-first.md
@.claude/rules/path-moves.md
@.claude/rules/changelog-format.md

共享 SOP 只管「项目怎么和 AI 协作」；只有本工程需要的纪律（文件系统怎么设计、压在本机资源上的流程）放 `.claude/rules/`，不往上游推。

规则正文只写怎么做（`.claude/singlefs-ai-sop/rules/rules-discipline.md`）：实测、论证、定案日期不写进规则，经过要留就留在 `records/` 与 `.claude/kb/` 里已有的那一份。

```

**出处 `CLAUDE.md:68-99`（整段抄，未转述）**

```markdown
## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | **决策索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/decisions/` | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
| `.claude/kb/decisions-history.md` | 决策变更史，一条决策一节、节内是它的完整历史，组织形态见 `.claude/rules/changelog-format.md` |
| `.claude/kb/experiments.md` | **实验索引**：编号、简称、状态、指向正文的链接 |
| `.claude/kb/experiments/` | 每个实验一个文件（`NN-简称.md`），正文与口径都在这里 |
| `.claude/kb/experiments-history.md` | 全部实验的变更史，一个实验一节，组织形态见 `.claude/rules/changelog-format.md` |
| `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
| `.claude/kb/feature-bits.md` | feature bit 的记账表（位号 / 类别 / 名称 / 引入版本 / 引入 commit / 状态 / 语义一句话），形态由 D15（格式冻结政策） 已定项 10 定。**位号的唯一登记位不在这里**，在 D15（格式冻结政策） 已定项 4 那张表；门禁 93 号判两处逐位一致 |
| `.claude/kb/term-renames.md` | 全仓术语改名的登记表，一行一条（旧名 / 新名 / 匹配），门禁 90 号按它查全仓不再出现旧名；怎么改名见 `.claude/rules/path-moves.md`「改一个全仓术语」 |
| `.claude/kb/freeze-layer-membership.md` | 冻结层归属登记表：一行一个结构，写它落 D15（格式冻结政策） 已定项 7 的哪一层或哪个独立冻结组件、态别、退不退出冻结、依据哪条分项；门禁 16 号按它判三条（每层、每棵树、每个单元类都有行；写「退出」的必须是派生态；依据点名的分项存在且已定）。态别判得对不对门禁管不了 |
| `.claude/kb/tooling.md` | 本机工具与模型的事实（本地腿用哪个模型、网关怎么调、量化到几位、中文为什么会退化性复读） |
| `.claude/kb/INDEX.md` | kb 的导航表，本身不放事实；它是唯一不留「## 历史版本」节的 kb 文件（`kb-discipline.md` 第 8 条显式豁免） |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `.claude/kb/checks-owed.md` | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| `.claude/kb/layout/` | 每个里程碑写出哪些字节，一个里程碑一个文件，与 `milestone/` 同号（`NN-简称.md`）：`01-first-txn.md` 每段每字段指向一条决策分项、给宽度与取值，指不到的就是格式级空白，第八节是根槽写路径的段序列登记表；之后的里程碑只登记新写出的形态，宽度与落点仍以 `01` 为准 |
| `.claude/kb/vm-harness.md` | 怎么把实验送进虚机在真块设备上跑 |
| `.claude/kb/verification-build.md` | 三样验证手段（checker、事务层、崩溃点重放）怎么落地、被谁挡着 |
| `.claude/kb/milestone/` | 里程碑规划，一个里程碑一个文件（`NN-简称.md`）：每步设想、验收标准、写出的字节、会碰到的决策点；每步开工前回来改 |
| `research/scripts/fetch-refs.sh` | 把承重的外部文献重新固定到本机（URL + sha256 + 引用方），`pdf-text.py` 抽文本，断言在 `verify-citations.sh` |
| `research/scripts/stage-mine.py` | 几个会话共写一批文件时，只把这一轮的块放进暂存区（命中 `--match` 的进，命中 `--foreign` 的拒绝） |
| `research/scripts/check-staged.sh` | 在临时 worktree 上只拿「HEAD + 暂存区」跑 doc-lint 与快的 kb 阶段 |
| `research/scripts/replace-once.py` | 定点替换：旧串在文件里必须恰好命中一次，否则不写；几个会话共写一批文件时只许这样改，不许整份重写 |
| `research/scripts/insert-row.py` | 往公共表里插一行：按锚点定位（锚点必须恰好命中一次），写之前复核文件没被别人改过（读时记 sha256，写前再比一次，变了就拒绝）。`replace-once.py` 只解决「改一处已有的文字」，插入没有旧串可替、只能读整份写回，两个会话一前一后会互相覆盖 |
| `research/perf-by-milestone.md` | singlefs 与六家文件系统按里程碑的性能对比；数来自 E152（按里程碑对比六家文件系统的文件性能），表由 `research/scripts/e152-tables.py` 生成 |
| `records/` | 建设过程 |
| `briefs/` | 每次更新的简报，按日期一份（`YYYY-MM-DD.md`）：那一版能做什么、验到哪、还没罩到什么；旧的不回头改 |

```

**出处 `CLAUDE.md:100-106`（整段抄，未转述）**

```markdown
## 本项目的特殊性

1. **没有 oracle。** 从零设计意味着没有参照实现可比对——移植类项目那种
   「拿现成工具的输出当标准答案」的便利这里不存在。功能正确性只能靠模型对拍，
   这是最大的隐性成本，见 `.claude/kb/prior-art.md`「三、Rust 侧现有轮子」。
2. **不进 Linux 主线**（D7（是否进 Linux 主线））。前几年按单人项目做，准入判据是门禁。

```

**出处 `CLAUDE.md:107-112`（整段抄，未转述）**

```markdown
## 一句话版本

- 先定决策，再写代码——未定项还开着就写下去的实现多半要返工。
- 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」（`.claude/rules/fs-design.md`「从事务开始，不从功能开始」）。
- 门禁全绿**只构成新池新建文件在模型层的崩溃一致性证据**；里程碑「覆盖写、释放、回退与复用」步 0 那条固定脚本（覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次）在管理员回退改成挂着时的向前发布之后，层 0 用例只改到编得过、下游钉的值没有重核，重写与重跑归层 0 规模那一轮与实六，这之前它的层 0 结果不作数（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实三交回」那一行）——层 0 崩溃点重放（门禁 54 号）的负载是两条流：新池新建文件，以及固定脚本到 E；checker 判 48 条不变量（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`；第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变）、I-5.4（分配记录罩住的槽互不相交）、I-1.8（归并后版本全序）、I-7.3（环健康性） 与 I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读）、I-8.9（一次发布的记录序号连续且只有末条带标志） 、I-7.12（系统配置 F 不低于同盘根上的 F） 、I-1.11（映射 key 与单元头相符）、I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序）；I-3.1（已分配统计对得上）、I-2.1（校验和与内容匹配） 与后加的六条里除 I-3.8（实例表行唯一且低于挂载根） 之外的五条按回退候选集判，候选集的下界取 F_生效（各幸存盘最新持久有效根带的 F 与系统配置里的 F 取大）；I-3.1（已分配统计对得上） 在最新根带的 F 低于 F_生效 时报不适用）。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:2-5`（整段抄，未转述）**

```markdown

调度记录：records/2026-09-28-门禁59号提速与双机分片.md「第二轮：门禁收缩、提炼、三方与测试」第 1 步。日期 2026-09-28。
只交事实，不交「该合、该砍」的结论。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:6-11`（整段抄，未转述）**

```markdown
## 交付

- 逐格表：`/tmp/claude-1000/gate-inventory/cells.tsv`，84 行数据 + 1 行表头，10 列，制表符分隔（`awk -F'\t' '{print NF}'` 85 行都是 10）。
- 表里「本文件:N」指第 1 列那份门禁文件的第 N 行；「NN 号:N」指 `.claude/gate.d/` 下以 NN 开头的那份的第 N 行；别的写全路径加行号。所有「文件:行」都是这一次现查的（一段脚本核过 506 处引用的行号不越界，余下 2 处是抄自记录的装置源码行号 `new_pool_file_creation_on_device.rs:1475`、`publish_order_matches_litmus.rs:47`，照原记录写）。
- 各格原样日志：`/tmp/claude-1000/gate-inventory/logs/<门禁>__<格>.log`；计时：`timings.tsv`（门禁、格、退出码、毫秒）；原 47 号那一趟：`old47-run.log`、`old47-run-attempt1.log`。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:12-51`（整段抄，未转述）**

````markdown
## 一、总数

```
$ wc -l .claude/gate.d/*.sh | tail -1
  15276 total
$ ls .claude/gate.d/*.sh | wc -l
23
```

格数按每份文件的格名数组数（`CELLS=`、`CELL_NAMES=`、`CELL_ORDER=`、`GRID_NAMES=`；没有数组的算整道一格）：

```
10-references-and-invariants.sh	7
11-review-and-sync-records.sh	7
12-doc-forbidden-notations-and-old-terms.sh	3
13-code-vague-names-and-test-file-names.sh	2
14-harness-one-scenario-per-test.sh	1
15-research-build.sh	1
20-doc-decision-documents.sh	13
27-code-constants-enums-bits-match-kb.sh	3
30-decision-history-entries.sh	3
32-doc-field-and-layout-registry.sh	9
33-code-experiment-and-mutation-source-discipline.sh	3
34-doc-experiment-pages-and-products.sh	10
43-checks-owed-and-closeout.sh	5
47-code-tooling-selftests-and-registries.sh	8
54-layer0-replay.sh	1
55-qemu-device-streams.sh	1
57-lkmm.sh	1
59-crates-mutation-replay.sh	1
74-model-differential.sh	1
77-test-environment.sh	1
87-replay.sh	1
92-layout-checker-sync.sh	1
94-checker-implementation-disjoint.sh	1
总格数 84
```

其中 74 格在 14 道带格名数组的门禁里，10 道是整道一格。没写 `# gate-category:` 的 13 道（10、11、15、30、43、55、57、59、74、77、87、92、94）第 3 列按 records 那份的五类表填、标「推」（records/2026-09-28-门禁59号提速与双机分片.md:63–69）。

````

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:52-59`（整段抄，未转述）**

```markdown
## 二、怎么跑的

- 逐格顺序跑，一次一格：`nice -n 19 bash research/scripts/capped.sh 8 timeout -k 10 300 bash .claude/gate.d/<门禁> --check <格>`，在仓根上跑、不给项目根参数（默认取脚本往上两级）。整道的三份（77、92、94）不带 `--check`。脚本在 `/tmp/claude-1000/gate-inventory/run-cells.sh`，清单 `cell-list.tsv` 77 行。
- 开跑前看负载：没有 qemu、vm-bench、e152、fio；load average 2.46（32 核）；本机另有 vllm 本地模型与几个 claude 会话在跑。跑的是**工作区**（不是 `--staged` 的临时树），工作区里有几个会话的未提交改动，所以取改动范围的格（11 号几格、34 号几格、20 号 settled-same-file、30 号几格）判的是这些改动合在一起的样子。
- 没跑：54、55、57、59、87（重型）与 15、74（要编译），第 6、10 列写「不跑」。
- 两格撞了 300 秒停掉：12 号 term-renames（日志只有格名一行）、47 号 research-script-selftests。
- 跑的过程中门禁文件没变（`find .claude/gate.d -maxdepth 1 -name '*.sh' -newer cell-list.tsv` 零命中）；20 号的暂存状态开工时是 `AM`、收尾时是 `A `（有人在这期间暂存了它，文件内容没变）。别的会话在这期间改了 `.claude/rules/changelog-format.md`（「门禁管哪一半」从第 51 行挪到第 52 行），表里引它的行号按收尾时的文件改过。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:60-79`（整段抄，未转述）**

````markdown
## 三、今天的判定

按 cells.tsv 第 10 列数：绿 54、红 17、77 有 4、不跑 7、不跑完 2（合计 84）。

```
$ awk -F'\t' 'NR>1 {v=$10; sub(/[（：(].*/,"",v); print v}' cells.tsv | sort | uniq -c
      4 77
      7 不跑
      2 不跑完
     17 红
     54 绿
```

红的 17 格（第一处点名见 cells.tsv）：10 governance-refs；11 batch-scope、crates-adversarial-review、abandoned-rounds、knowledge-sync、agent-def-adversarial-review；12 prime-marks；13 vague-names；20 item-ref-status；34 results-cited、evidence-in-repo、decision-links、archive-past-rounds；43 paid-cited-tests；47 stage-owners、research-gate-lint；77 整道。

其中几处红的对象是这一轮合并本身带出来的（事实，逐条可核）：
- 10 governance-refs 列了 28 处指不到的指向（`grep -c '✗ \.claude/\|✗ CLAUDE.md'` 数得 28：旧门禁号 20 处、不存在的路径 8 处），全是旧门禁号与旧路径：例 `.claude/agent-common.md:28` 的 `.claude/gate.d/91-archive-past-rounds.sh`、`.claude/agent-common.md:99`「69 号」、`.claude/main-agent.md:65`「75 号」、`.claude/rules/path-moves.md:33`「21 号」、`.claude/rules/verification.md:81`「62 号」、`CLAUDE.md:79–81`「93 / 90 / 16 号」、`.claude/agents/kb-scribe.md:32` 的 `21-decision-items-sync.sh`（全表在 `logs/10-references-and-invariants__governance-refs.log`）。
- 47 stage-owners 点名合并出来的 10、11、12… 号在 `stage-owners.tsv` 里没登记。
- governance-refs 只扫治理文档，kb 里按旧文件名称呼门禁的它看不见：`.claude/kb/decisions.md:14`（`37-decision-summary-width.sh`）、`.claude/kb/decisions.md:83`（`22-item-ref-status.sh`）。

````

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:80-121`（整段抄，未转述）**

````markdown
## 四、47 号 research-script-selftests 的四份

这一格先跑 runner 表 45 条（`.claude/gate.d/47-code-tooling-selftests-and-registries.sh:190–217`），全过了才进覆盖段（:230–286）。逐格计时那一趟在 runner 表里撞了 300 秒，没走到覆盖段。覆盖段是一段内嵌 python，我把它原样抽出来（`/tmp/claude-1000/gate-inventory/coverage.py`，与新 47 号 :237–263 那一段同字节）在真仓根上跑，传同一条 NOT_RUN_HERE：

```
CLAIMED	53	RUN	48
MISSING	research/scripts/layer0-shard-configuration-check.sh
MISSING	research/scripts/layer0-shard-run-selftest.sh
MISSING	research/scripts/memory-peaks.tsv
MISSING	research/scripts/rerun-failed-tests.py
EXEMPT	research/scripts/vm-bench.sh	自证要连起三次虚机，挂钟太重，不进每轮门禁
```

「声称有 --selftest」的认法是文件全文里出现 `--selftest` 这串字（:241–242 `"--selftest" in open(path…).read()`），不看它是不是自己的开关。逐份现查：

| 文件 | 自己有没有 `--selftest` | 依据（`grep -n -- --selftest`） |
|---|---|---|
| research/scripts/layer0-shard-configuration-check.sh | 没有，只在注释里提别人的 | :24「research/scripts/layer0-shard-run.sh --selftest 里对应那一格判错」、:27「layer0-shard-run.sh --selftest 用」 |
| research/scripts/layer0-shard-run-selftest.sh | 没有，它是别人的自证本体 | :2「research/scripts/layer0-shard-run.sh --selftest 的本体」；`layer0-shard-run.sh --selftest` 在 runner 表里（47 号:208） |
| research/scripts/memory-peaks.tsv | 没有，是数据文件 | 内存包装写的峰值日志，第 248、249、598、743、1012、1081、2200、2207 行记的是别的命令行里的 `--selftest`；它被 .gitignore 挡着（`git check-ignore -v` → `.gitignore:14:/research/scripts/memory-peaks.tsv`），`--staged` 的临时树里没有它 |
| research/scripts/rerun-failed-tests.py | 有 | :8 用法「rerun-failed-tests.py --selftest」、:154 `if arguments == ["--selftest"]`；runner 表与 15 号都没调它 |

所以四份里三份是认法认错（文本里提到而已），一份（rerun-failed-tests.py）是真有自证、没人跑。

合并前的原 47 号在同一份仓上判不判它们：

- 覆盖段两份逐字相同：`diff <(sed -n '/^NOT_RUN_HERE=(/,/^PY_COVERAGE/p' old47.sh) <(sed -n '/^NOT_RUN_HERE=(/,/^PY_COVERAGE/p' 新 47 号)` → 「覆盖段逐字相同」。它按被判仓的 `.claude/gate.d/[0-9][0-9]-*.sh` 认「有阶段在跑」，不看自己那张 runner 表，所以在同一棵树上两份判得一样。
- 实跑：把 `git show HEAD:.claude/gate.d/47-research-script-selftests.sh` 拷进 `/tmp/claude-1000/gate-inventory/old47-tree/.claude/gate.d/`（sha256 d1b39662…121b，与 HEAD 那一版相同），`.claude/scripts` 与 `.claude/singlefs-ai-sop` 两个符号链接指回真仓，给真仓根当参数跑。第一次少了 `.claude/singlefs-ai-sop` 链接，preflight 当场拒（`old47-run-attempt1.log`，rc=1，0 秒）；补链接之后第二次原样输出（`old47-run.log`）：

```
  ✗ 这些 research 脚本声称有 --selftest，却没有任何门禁阶段在跑它：
      research/scripts/layer0-shard-configuration-check.sh
      research/scripts/layer0-shard-run-selftest.sh
      research/scripts/memory-peaks.tsv
      research/scripts/rerun-failed-tests.py
  → 怎么办：把它加进上面的 runner 表；确实不该每轮跑的，登记进 NOT_RUN_HERE 并写明为什么——没人跑的自证只在写它的那天跑过一次。
rc=1 用时 344 秒
```

  原 47 号的 runner 表 41 条（新的 45 条，多了 mutation-shard-run.sh、multi-host-run.sh、crates-mutation-rows.py、migrate-changelog-format.py 四条）在今天的工作区上全过（日志里没有一行 runner 的 ✗），然后覆盖段报同样四份，退 1。
- 另在 HEAD 那棵树上现算（`git show HEAD:` 取出 research/scripts/ 78 份与 `.claude/gate.d/[0-9][0-9]-*.sh` 79 份放进 `head-coverage-tree/`，跑同一段覆盖代码）：`CLAIMED 49 RUN 44`，MISSING 是 harness-test-timing.py、layer0-shard-configuration-check.sh、layer0-shard-run-selftest.sh、rerun-failed-tests.py。也就是说三份在合并之前的 HEAD 上就判红；memory-peaks.tsv 只在工作区出现（没进 git）；harness-test-timing.py 今天在暂存区里是删除（`git status` 为 `D `）。

````

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:122-130`（整段抄，未转述）**

```markdown
## 五、跑的先后有依赖的

1. **77 号要在所有跑测试的阶段之后，而按文件名排它不在最后。** 共享 gate.sh 按文件名排序逐个跑本地阶段（`.claude/singlefs-ai-sop/scripts/gate.sh:636` `find "$GATE_D" -maxdepth 1 -name '*.sh' … | sort`）；77 号自己的分界写的是「这一阶段排在最后」（`.claude/gate.d/77-test-environment.sh:40`），取祖先 gate.sh 的启动时刻当分界、晚于它的临时条目不判红（:38–41）。按文件名 87、92、94 排在 77 后面；87 号起 research/scripts/replay.sh，它的输出目录是 `${TMPDIR:-/tmp}/singlefs-replay-$$`（replay.sh:30），名字落在 77 号扫的 singlefs-* 里（77 号今天的日志「共查 206 项 singlefs-*」）。推的，没量过：87 留下的东西这一趟 77 看不见，下一趟门禁开跑之前留下的会被下一趟 77 判红（与 C474 同形，`.claude/kb/checks-owed.md:396`）。92、94 只读文本。
2. **共享阶段先于本地阶段**：「本地阶段判别力」stage-selftest（gate.sh:404）与「构建与单测」check.sh（gate.sh:539）都在「项目本地阶段」（gate.sh:631）之前跑；check.sh 的 `cargo test --all` 已在 debug 下跑过 54 号快档与 74 号那批用例（见 cells.tsv 第 7 列）。
3. **跨趟的依赖**：55、57、59、74、87 号先问 research/scripts/stage-must-run.sh，比 `refs/sop/staged-green` 那棵树与这次暂存树（stage-must-run.sh:16）；那条 ref 只在 research/scripts/gate-staged.sh 起的整轮全绿之后才前移（gate-staged.sh:4、:47）。所以这几道「这一次要不要跑」取决于上一次整轮门禁有没有全绿。54 号快档核的全绿标记由此前的 `--full` 写（54 号:21），同样跨趟。
4. **写回的次序**：30 号 status-sync 的 `--write` 按 decisions.md 索引表的「状态」「结论（简报）」两格重写史册节顶的现状行（30 号:57–60）；20 号 decision-items-sync 的 `--write` 重写的正是 decisions.md 索引表的「状态」列（20 号:26、:35–36）。修红时 20 号的 `--write` 要先于 30 号的 `--write`，反过来 30 号写进去的是旧状态。门禁里的判定次序（20 在 30 前）与这个一致。
5. **归档与同步记录打架**：34 号 archive-past-rounds 要求删掉上一轮的记录，11 号 knowledge-sync 要求改动范围里的同步记录还在；C450 仍欠（`.claude/kb/checks-owed.md:376`），34 号的出路里写了「不给基准…删完门禁 11 号的 knowledge-sync 格当场红」（`logs/34-doc-experiment-pages-and-products__archive-past-rounds.log`）。这是 `--apply` 与判定之间的先后，不是门禁里的次序。
6. 10 号 citations 在整道七格全跑时先在后台起、轮到它时再 wait（10 号:23–24），是道内的并行，与别的阶段无关。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:131-139`（整段抄，未转述）**

```markdown
## 六、重复与重叠（cells.tsv 第 7 列的汇总，只列事实）

第 7 列不写「无」的 29 行（`awk -F'\t' 'NR>1 && $7 !~ /^无/' cells.tsv | wc -l` → 29）。按性质分：

- **同一份判法跑两遍**：33 mutation-tables 的 crates 那一半与 59 号派活前判的是同一份 `row_problems`（59 号:8–10、:14；33 号:106–108）；47 agent-write-scope 的「注册着、自证过」与共享「工具层的闸」hooks-registered.sh ①②（hooks-registered.sh:12–14）判同一件事，整轮里 .claude/hooks/ 下这些钩子的 `--selftest` 各跑两遍；54 号快档、74 号那批用例先被共享 check.sh 的 `cargo test --all`（check.sh:97）在 debug 下跑一遍。
- **同一判据落在两处**：43 table-shape「同一个编号在两张表里各登记一次」（43 号:103）与共享 doc-lint F「一个编号只许有一处登记位」（doc-lint.sh:770–777），checks-owed.md 两张表都带 registry 标记（:13、:484）；12 prime-marks 与 43 closeout-collects-open ④ 在收口表首列的 U+2032、U+2033 上重叠（43 号:223）；20 kb-shape 第 7 段与 20 decision-items-sync 判同一个量（20 号:1765–1768 写明是故意留的两条路）。
- **同一件事分两道**：研究脚本自证 5 份在 15 号（15 号:85–139）、45 条在 47 号 runner 表。
- **同对象不同判据 / 同数据源**：10 experiment-refs、decision-refs 与 doc-lint H（≥3 次、只 kb）；13 vague-names 与 naming-lint；30 shape 与 history-ordinal；30 status-sync 与 34 index-sync（都读 experiments.md 索引）；27 feature-bits 与 92 ③（都读 D15 已定项 4 登记表）；32 segment-registry 与 55 direct 档（都比 E142 产物）；20 status-redundancy、kb-shape 第 5 段与 34 decision-links ⑦（决策首行与分项索引，format-evolution.md:27）；32 format-const-placeholders 与 10 decision-refs；47 fixture-claims ④ 与 doc-lint M（同一个 date_out_of_range）；47 research-gate-lint 与共享 gate-lint、shell-lint、script-modes（同工具、目录不重叠，只有 .claude/gate.d/ 上的终止进程扫描与 shell-lint S2 重叠）；47 rules-manifest 与 doc-lint K（同判据、对象不重叠）；77 与 gate.sh「跑完没留下临时文件」；92 与 27 format-constants（同库）。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:140-147`（整段抄，未转述）**

```markdown
## 七、依附的对象过期或已关的（第 8 列里挑出来的事实）

- 依附已关里程碑一「新池新建文件」（出口已满足，CLAUDE.md:8）的格：32 号 field-refs、field-projection、first-txn-hooks、format-const-placeholders。对象文件都在；first-txn-hooks 今天判「是」的未定项 0 条。
- 依附里程碑二收口表具体一行的格：11 abandoned-rounds（第 33 行，02-second-txn.md:367）、32 second-txn-hooks（第 31 行，:365）、32 tree-table-reserve（第 17 行，:348）、43 closeout-collects-open（第 34 行，:368）、43 row27-preconditions（第 27 行，:361，那一行今天写「这一行已有去向，不再等前置」）。几行都还在。
- 依附的欠账已还清、格还在盯的：C4（47 rules-manifest）、C11（27 feature-bits）、C12（94）、C14（92）、C31（20 stale-open-items）、C38（10 citations）、C45（20 freeze-layer-membership）、C94（32 field-table-sums）、C316、C485（32 segment-registry）、C327（33 mutation-tables）、C338（32 tree-table-reserve）、C358（43 audit-contradictions）、C382、C384（47 research-gate-lint）、C396（77）、C468（11 knowledge-sync）、C472（11 batch-scope）。行号见 cells.tsv。
- 没有任何规则或定义点名的格（只靠欠账或记录撑着）：11 batch-scope、11 abandoned-rounds（`grep -rn` 零命中，命令在 cells.tsv 第 8 列）、27 clause-enums、34 verdict-false-named、repro-command、experiment-orphans、43 paid-cited-tests、15 整道。
- 史册改成按决策分节之后：原 48 号（条目住在日期所在月的那一份）与原 49 号（快查与原文同步）判的月份文件已不存在（`.claude/kb/decisions-history/` 不在），30 号的 shape、status-sync 两格判的是新形态（.claude/rules/changelog-format.md:7–43）；20 item-ref-status 今天的 131 处红列出来的都在 decisions-history.md。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:148-154`（整段抄，未转述）**

```markdown
## 八、什么现象会推翻这份表

- 某格计时：同一条命令在负载更低时重跑，时间差一个量级以上（这一次是 nice 19、单格串行、load 约 2–3 / 32 核）。
- 「抓到过没有」写「没找到记录」的格：在 records/、.claude/kb/、research/prompts/ 里找到一条它判红而是真问题的记录（我按旧号、`.claude/gate.d/<旧文件名>`、判红原句搜，另搜了强信号词「转绿、改回、抓到、查出」，逐条读的只是命中的那一部分，见「没做什么」）。
- 「今天」那一列：工作区在变（别的会话在改），同一格重跑判定可能不同；红的多数点名的是别的会话在途的改动。
- 47 号四份的结论：rerun-failed-tests.py 之外的三份，只要在文件里出现一处真的 `--selftest` 开关处理，就不是认法认错。

```

**出处 `research/prompts/gate-shrink-r1-inventory/report.md:155-164`（整段抄，未转述）**

```markdown
## 九、没做什么

- 54、55、57、59、87 号（重型）与 15、74 号（要编译）没跑，第 6、10 列写「不跑」；它们的「判什么、读什么」取自文件头与 stage-inputs.tsv。
- 12 号 term-renames、47 号 research-script-selftests 撞 300 秒停掉，没有完整判定；47 号那一格的覆盖段另外现算了（第四节）。term-renames 没有另算。
- 「抓到过没有」一列不是穷举：我按旧号（「门禁 N 号」「N 号」、旧文件名）在 records/、.claude/kb/、research/prompts/ 里搜到 3716 行命中（`hits.txt`，`grep -vc '^=== ' hits.txt` 数），只逐条读了 records/ 与 kb 里的那一部分（`hits-kb-records.txt`）和带强信号词的一部分（`strong-hits.txt`），research/prompts/ 里的大半没逐条读；写「没找到记录」的格可能有漏的。也没搜 git 历史里已归档的文件。
- 「与谁重复」只比了共享 gate.sh 第 380–560 行那批阶段与这 23 道之间；没比钩子（除 write-guard 共用库那一处）、没比 research/scripts 里不属于门禁的检查。
- 各格的红没有修，也没判归属是这一轮的还是别的会话的（只按日志写了点名的文件）。
- 没跑 `gate.sh` 整轮与 `gate-staged.sh`，没做任何 git 写操作，没改仓里任何文件。
- 删了自己在草稿目录建的两份仓副本：`/tmp/claude-1000/gate-inventory/head-coverage-tree`（2.9M，HEAD 的 research/scripts 与门禁阶段）、`/tmp/claude-1000/gate-inventory/old47-tree`（24K，原 47 号的拷贝与两个指回真仓的符号链接）。没删：`logs/`（428K，各格原样日志）、`old47-run.log`、`old47-run-attempt1.log`、`old47.sh`、`coverage.py`、`timings.tsv`、`hits*.txt`、`gen_part*.py` 与 `assemble.py`（cells.tsv 的生成器）——都是这份报告的依据，主 agent 要核。

```

**出处 `records/2026-09-28-门禁59号提速与双机分片.md:41-78`（整段抄，未转述）**

```markdown
## 全量合并方案（Q14）

用户 2026-09-28：「先扫一次 全部的门禁 很多门禁从注释上来看 不是功能相似 就是周期 完全可以合并」「48 和 49 这种 虽然是检测不同的文件 但是 我觉得没有什么需要单独存在的必要」。扫描产物：本会话草稿目录 `gate-scan.tsv`（79 道：号、行数、归属、复用登记、样本、`gate-stage`、调的库与脚本、读的路径、`gate-similar`）。按判的对象与判法分组，每组合成一道、沿用组里最小的号：

| 组 | 合成一道 | 原来的号 | 依据（判的对象与共用） | 波次 |
|---|---|---|---|---|
| A 文本与命名 | 12 | 12、13、14（第 2–4 样）、90 | 全仓文本或 `.rs` 名字里不许出现的写法；14 号第 1 样挪进 54 号 | 辛，等 singlefs-dd 与壬 |
| B 决策文档 | 20 | 16、20、21、22、24、28、29、31、35、37、44、60、61 | 都读 `.claude/kb/decisions/` 与 `decisions.md`；20+21 共用 `gen-decision-items.py`，20+24、22+44、60+61 各共用一份库；16 号调 22 号的 `lib-item-ref-status.py` 判冻结层归属表点名的分项已定 | 壬先做八道，16、21、31、35、37 第二波并进 |
| C 决策变更史 | 30 | 30、48、49 | 都读 `decisions-history/` 与 `decisions-history.md` | 第一波 |
| D 格式与布局登记 | 27 | 27、32、38、39、42、51、52、53、76、79、82、92、93 | 字段表、字节布局表、段序列登记表、里程碑各步挂钩、格式常量、feature bit、条文集合与代码的对应；27、39、53、92 共用 `lib-format-const.py`，79、82 的 `gate-similar` 互相点名；42、76 同形只差里程碑 | 第二波 |
| E 欠账与收口 | 43 | 43、67、78、81、89 | 都读 `checks-owed.md` 与里程碑收口表；67、81 共用 `lib-owed.py`；89 只盯收口表第 27 行 | 第一波 |
| F 引用与不变量 | 10 | 10、36、70、97 | 引用指得到、不变量条数与字段落点；10 号第 3 格与 36 号重叠 | 第一波 |
| G1 实验页与产物 | 34 | 34、40、69、75、84、85、86、88、91、99 | 实验页、实验索引、`research/results/` 产物与复跑登记 | 第二波 |
| G2 实验与 crates 源码纪律 | 33 | 33、80、96 | 变异表、绝对值断言、读数纪律，都扫实验二进制与源码 | 第二波，等庚 |
| H 流程留痕 | 11 | 11、19、56、58、66、68、72 | 三方轮次、判决点名、改动走三方、批次范围与阶段同步记录，都读 `research/prompts/` 与改动范围 | 第一波 |
| I 工具层自检 | 47 | 47、50、62、63、64、73、95、98 | 脚本自证、清单与登记一致（50、62、63、98 共用 `lib-manifest.py`）、改动范围取法、脚本 lint、样本声明 | 第二波，等己 |
| 不并 | 各自 | 15、54、55、57、59、74、77、87、94 | 重型或要编译，各有自己的复用标记与内存上限；94 判包依赖方向 | — |

用户 2026-09-28 又定：「门禁部分 可以合成的要合成 而不是互相引用」。据此 16 号从格式组挪进 B 组（它调的是 22 号的库），格式的两组合成 D。合完之后跨组共用的只剩两份不带判据的库：`lib-owed.py`（读欠账表，43、27、33 三道用）与 `lib-forbidden-notations.py`（12 号与写入钩子 write-guard 共用同一条规矩的判法，钩子与阶段合不成一个）；一道门禁不再调另一道门禁的判法，`gate-similar` 里不再点名已经并进同一道的号。

用户 2026-09-28 再定：「你最好按照下面的分类来 文档类门禁 代码类门禁 harness类门禁 checker类门禁 checker-tier类门禁这样组织 每个门类下面几个门禁」。共享 `gate.sh` 只跑 `.claude/gate.d/` 顶层的 `*.sh`（62 号第①条同一层），分子目录要改上游，不走；分类用两样表达：每道文件头 `# gate-category: <文档类|代码类|harness 类|checker 类|checker-tier 类>`（62 号判写了、且是五类之一），号段按类分。54、55、57、59 号不改号（全绿标记与复用判定按文件名存，改号要重跑、引用几百处）。

| 类 | 号段 | 合成后的门禁（原来的号；括号前的字母是上表的组） |
|---|---|---|
| 文档类 | 10–39 | 文本写法 A（12、90）；决策文档 B（16、20、21、22、24、28、29、31、35、37、44、60、61）；决策变更史 C（30、48、49）；欠账与收口 E（43、67、78、81、89）；引用与不变量 F（10、36、70、97）；实验页与产物 G1（34、40、69、75、84、85、86、88、91、99）；流程留痕 H（11、19、56、58、66、68、72）；字段与布局登记 D（32、38、39、42、51、52、53、76、79） |
| 代码类 | 40–49 | 命名与测试文件（13，14 号「不按里程碑起名」）；实验与变异源码纪律 G2（33、80、96）；格式常量与代码对应（27、82、93）；工具层自检 I（47、50、62、63、64、73、95、98）；research 构建与单测（15） |
| harness 类 | 50–53 | 模型对拍（74）；harness 用例粒度（14 号「一条用例一个场景」） |
| checker-tier 类 | 54–59 不动号 | 54（并入 14 号「崩溃枚举用例住 checker 档且登记」「测试文件声明模块」）、55、57、59、77、87 |
| checker 类 | 60–69 | checker 与实现不同源（94）；布局常量与 checker 判定同步（92） |

用户 2026-09-28 又定：「门禁的民资也要按照我说的 几类来命名」（「民资」即「名字」）。文件名定为 `<号>-<类>-<内容>.sh`，号不变，类名用英文短词：`doc`（文档类）、`code`（代码类）、`harness`、`checker`、`checker-tier`；类名已经进了文件名，号段不再按类挪（挪号要多改几百处引用），上表「号段」一列作废。54 号例外：它的文件名写在 `research/scripts/admission.py`（8 处）与 `layer0-shard-run.sh`（1 处）里，这两份按内容进崩溃枚举用例的输入指纹，改名要重跑全量约 18 小时，所以与 KV 接入同一批改（那一批本来就改这两份、接完用户已定再跑一趟全量）；55、57、59、74、87 号改名只让各自按文件名存的全绿标记作废一次，这一轮改。改名时核崩溃枚举用例的判法摘要前后逐字节相同。

用户 2026-09-28 再定：「做完后 门禁取消编号 不再以编号命名 以后记录门禁按照全名记录」。合并全部做完之后的「登记收尾」改成：文件名去掉号，写成 `<类>-<内容>.sh`（样本目录跟着改）；全仓的「N 号」「门禁 N 号」与旧路径一次改成全名（规则、定义、钩子、脚本归工具实现员，kb 归书记员，`CLAUDE.md`、`.claude/skills/`、两份排除表归主 agent，records 与冻结证据里的路径照 `path-moves.md` 改）；「文本写法」那道的禁用写法登记加一种形态「用编号称呼门禁」，写入时 write-guard 拒、提交时整轮判。54 号随 KV 那一批由 singlefs-29 改成不带号的名。动手前先核共享 `gate.sh` 与 `stage-selftest.sh` 按不按「两位数开头」找本地阶段，按的话报用户，不就地改上游。

A 组因此拆开：12、90 进文档类，13 进代码类，14 按四样各归其类；D 组也拆开：格式常量与代码对应（27、82、93）进代码类，92 进 checker 类。改号与写 `gate-category` 归最后的「登记收尾」一件；各件合并时先沿用组里最小的号。

每一件合并照同一套出口：原判据各成一格，出路、查了多少项、射程句都留；合并前后在同一份仓快照上逐格违例相同；原来的样本格与弄坏开关都留；合成的阶段认 `--check <格名>` 只跑一格（three-way-materials 开工跑 58 号那一格、prior-art 开工跑 70 号那一格要用）；`stage-owners.tsv`、`check-staged.sh`、`kb-scribe-followups.tsv`、两份规则 `verification.md` 与 `path-moves.md` 不由各件改，逐处报给主 agent，最后一件「登记收尾」统一改。

```

**出处 `records/2026-09-28-门禁59号提速与双机分片.md:79-90`（整段抄，未转述）**

```markdown
## 第二轮：门禁收缩、提炼、三方与测试

用户 2026-09-28：「继续 门禁需要继续收缩 需要继续分类和精华 同时针对门禁 进行三方和测试」。第一轮合完是 23 道（文档类 10、11、12、20、30、32、34、43；代码类 13、15、27、33、47；harness 类 14、74；checker 类 92、94；checker-tier 类 54、55、57、59、77、87），约 80 格、脚本一万五千多行。「登记收尾」（去编号改全名）推到这一轮合并之后一起做，名字只改一次。

| 步 | 做什么 | 出口 | 谁 |
|---|---|---|---|
| 1 盘点 | 每一格：判什么、读什么、跑多久、与共享门禁或别的格是否重复、依附的规矩或里程碑还在不在、记录里有没有抓到过真问题 | 一张逐格的表，每行带出处 | 调查（只读） |
| 2 方案 | 按五类再合，砍掉没价值的格（「精华」） | `research/prompts/_gate-shrink-r1-body.md` | 主 agent |
| 3 三方 | 正推（sonnet）、云端攻方（opus）、本地攻方；攻「合了、砍了丢不丢覆盖」，顺带审第一轮改过的定义与共用约束（照 11 号 agent-def-adversarial-review 那一格要的） | 判决 `research/prompts/gate-shrink-r1-main-verification.md` | 主 agent 派、主 agent 判 |
| 4 实施 | 按判决合并、砍格，去编号改全名、阶段归属表、gate-category、写入闸加「用编号称呼门禁」形态、59 号读 `ENABLE_ACROSS_MACHINES`、`peer-host-lib.sh` 循环里 kill、77 号跑的次序 | 各件出口照判决 | 工具实现员、书记员、主 agent |
| 5 测试 | 全部阶段样本自检、各脚本自证、每道非重型阶段对真仓跑；重型的（层 0、QEMU、herd7、变异表、实验复跑）跑之前问用户 | 原样末行与退出码 | 主 agent 派 |

```

**出处 `research/prompts/gate-shrink-r1-inventory/cells.tsv:1-85`（整段抄，未转述）**

```markdown
门禁文件名	格名	类	判什么	读什么	跑一次要多久	与谁重复	依附的规矩或对象还在不在	抓到过没有	今天对真仓的判定
10-references-and-invariants.sh	experiment-refs	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	kb 与项目规则里引用的实验号都有定义（本文件:8；判法 :83–143，:129 `grep -rqE "^## $e " "$KB/experiments"`）	.claude/kb/**/*.md 递归（本文件:93–94）与 .claude/rules/*.md（:96），对照 .claude/kb/experiments/ 下的 `## E<n> ` 标题（:129）	0.4 秒（退 0）	部分重叠：共享「文档铁律」doc-lint.sh 的 H「编号形状的记号反复出现（≥3 次）却一处登记位都没有」（doc-lint.sh:20），只扫 kb/*.md、门槛 3 次；这一格对 1–2 次的悬空引用与 .claude/rules/ 也判（本文件:8「doc-lint 已覆盖一部分，这里补实验号」）。共享「编号与简称」number-name-sync.sh 只在编号有登记时比简称（number-name-sync.sh:96–97），不判悬空	kb-discipline.md「4. 矛盾比空白更糟」（.claude/singlefs-ai-sop/rules/kb-discipline.md:65）与「5. 编号只能做索引」（:69）；规则在	红过一次、不是缺陷：records/2026-09-24-里程碑二收尾调度.md:264 与 research/prompts/m2-kb-writeback-batch4-scribe-report.md:140 记它红在「E161 被引用但 experiments/ 下没有它」，归因是会话 singlefs-e1 的 E161 页还没写；判红而是真缺陷的记录没找到	绿（162 个不同的实验号；扫 210 份 kb、9 份规则）
10-references-and-invariants.sh	decision-refs	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	kb 与项目规则里引用的决策号都有定义（本文件:9；判法 :144–165，:151）	同 experiment-refs 的文件集合（本文件:89–100），对照 .claude/kb/decisions/ 下的 `## D<n> ` 标题（:151）	0.4 秒（退 0）	部分重叠：doc-lint.sh H（doc-lint.sh:20，≥3 次、只 kb）；number-name-sync.sh 只比简称不判悬空（number-name-sync.sh:96–97）	kb-discipline.md:65、:69；规则在	没找到记录（找到的只有改判法的 diff：research/prompts/_gate-fix-forks-r2-diff.md:110）	绿（28 个不同的决策号）
10-references-and-invariants.sh	declared-counts	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	invariants.md 里 `<!-- invariant-count -->` 下一行「现共 N 条在用」等于表里在用条数；欠账表数得出开着与已还清的条数（本文件:10；判法 :166–245）	.claude/kb/invariants.md（本文件:170–171 数表行，:186 定位标记）；.claude/kb/checks-owed.md 经 .claude/gate.d/lib-owed.py（:215–221）	0.1 秒（退 0）	无（与 invariant-count-elsewhere 共用数法 count_invariant_rows，本文件:169；判的文件不同，:26–27「逐字核过，判的不是同一件事」；doc-lint E 只判引用的 I 号有定义，doc-lint.sh:16）	evidence-discipline.md「能被一条命令数出来的数，就由一条命令来数」（.claude/singlefs-ai-sop/rules/evidence-discipline.md:137）；规则在	红过一次、是门禁自己的错：records/2026-09-16-subagent拆分提案.md:605 记它取到历史节里的旧数判红，修法在 :607；判红而是真缺陷的记录没找到	绿（正文声称 82 条在用，表里在用 82、总 88、退役 6；欠账数得出）
10-references-and-invariants.sh	governance-refs	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、`文件「小节」`在那份文件里找得到（本文件:11–12；判法在 .claude/gate.d/lib-governance-refs.py:4–15，外壳 :246–278）	CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/agents/*.md、.claude/rules/*.md、.claude/skills/*/SKILL.md（lib-governance-refs.py:34–35）；门禁号对照 lib 所在的门禁目录（lib-governance-refs.py:8、:45）	0.3 秒（退 1）	无：共享「链接指向」link-targets.py（gate.sh:409）只判 ](相对路径) 链接与「第 N 节」（link-targets.py:9–12），这一格判反引号路径、门禁号、「小节」（本文件:5 gate-similar）	.claude/rules/path-moves.md:10、:55「治理文档反引号里的路径由门禁 10 号第 4 段兜一部分」；规则在（仍按「第 4 段」称呼，合并后已是 governance-refs 格）	抓到过：records/2026-09-24-里程碑二收尾调度.md:237「10 号两处失效指向改回」；research/prompts/verification-split-sync.md:14 小节改名后两处定义与 skill 的旧小节名，「10 号从红转绿」	红：.claude/agent-common.md:28 点名的 `.claude/gate.d/91-archive-past-rounds.sh` 在仓里不存在（同一次还报 .claude/agent-common.md:99「69 号」、.claude/agents/experiment-designer.md:62「86 号」、.claude/agents/experiment-designer.md:30 `.claude/kb/decisions-history/` 等，都是这一轮合并与史册改形态后没跟的指向）
10-references-and-invariants.sh	invariant-count-elsewhere	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	invariants.md 与变更史之外的 kb 文件里写「N 条在用」「N 条不变量在用」的地方，N 等于实际在用条数（本文件:13；判法 :267–325）	.claude/kb/**/*.md 的正文（「## 历史版本」之前，排除 *-history.md 与 decisions-history/，本文件:292–297）；在用条数取 invariants.md（:281–282）	0.1 秒（退 77）	无（与 declared-counts 同一份数法、判的文件不同，本文件:26–27）	.claude/singlefs-ai-sop/rules/evidence-discipline.md:137；show-me-test.md「先问这个坑在哪一层」（本文件 :271–276 注释引）；规则在	没找到记录（.claude/kb/checks-owed.md:373 C446 只是描述 36 号判什么）	77（kb 正文里一处「N 条在用」的声明都没有）
10-references-and-invariants.sh	citations	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	kb 里承重的外部逐行引用今天还核得动：调 research/scripts/verify-citations.sh，按它的退出码判；脚本不在判红（本文件:14；:326–366）	research/scripts/verify-citations.sh（本文件:338）与它读的本机固定外部源码树（FS_REFS、KERNEL_TREE 默认值，:333–335）	16.9 秒（退 0）	无	kb-discipline.md「2. 每条带出处与状态」（:51）；C38 已还清（.claude/kb/checks-owed.md:577）；规则在	没找到记录（.claude/kb/prior-art.md:13 只写它挂在这道门禁上）	绿
10-references-and-invariants.sh	invariant-anchors	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 F 组；文件头没写 gate-category）	这次改动新写或改写的不变量行必须点名至少一条「D<n>（简称） 已定项 k」（本文件:15；判法 :367–465）	.claude/kb/invariants.md 相对基准的 diff 与暂存区 diff（本文件:382–420，:415 git diff -U0），基准经 research/scripts/changed-paths.sh 的 gate 取法（:375–377）	0.4 秒（退 0）	无	C69 仍欠着（.claude/kb/checks-owed.md:70），这一格只管新写的那一半；规则在	没找到记录	绿（这次改动 10 行都点名；存量 82 行里 34 行点不出，记在 C69）
11-review-and-sync-records.sh	batch-scope	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	还没进 HEAD 的改动里命中触发表的文件，每个都在 .claude/batch-scope 登记；登记的都是这一批的触发文件、带理由、从仓根写（本文件:7；判据 :44–52，外壳 :339）	.claude/gate.d/knowledge-sync-triggers.tsv 与 .claude/batch-scope（本文件:45–47、:354）；HEAD 起的改动路径（工作区、暂存区、未跟踪，:321 load_changed_since_head）	0.3 秒（退 1）	无（与 knowledge-sync 读同一张触发表，基准与问题不同，本文件:37–43）	没有规则或定义点名它（`grep -rn batch-scope CLAUDE.md .claude/main-agent.md .claude/agent-common.md .claude/rules/ .claude/agents/` 零命中）；依附的是 C472，已还清（.claude/kb/checks-owed.md:542）	没找到记录（research/prompts/c472-* 几份是立它那一轮的回扫）	红：149 个触发文件没登记进 .claude/batch-scope（它登记了 62 个），另 25 个登记的不是这一批的——判的是整个工作区，几个会话的未提交改动都在里面
11-review-and-sync-records.sh	verdict-names-local-samples	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	这一次改动新写的 research/prompts/<轮>-main-verification.md 按文件名逐字点名那一轮本地腿的每一份样本（含 -output-void<n>），0 字节样本不判（本文件:8；判据 :72–77，外壳 :437）	research/prompts/ 顶层新写的判决与 <轮>-local-*-output-s*/void* 样本（本文件:72–75）；改动范围经 changed-paths.sh 的 gate 取法（:309–319）	0.4 秒（退 0）	无（与 crates-adversarial-review、agent-def-adversarial-review 同一种「新写判决里按路径点名」的判法，对象不同，本文件:80–81）	.claude/rules/three-way-inference.md:120；规则在	没找到记录	绿（新写判决 4 份，3 份逐份点名 11 份样本，1 份那一轮没有本地腿样本）
11-review-and-sync-records.sh	crates-adversarial-review	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	这次改动里每个 crates/*/src/*.rs 都要在这次新写的某份 research/prompts/*-main-verification.md 里按路径出现（本文件:9；判据 :93–97，外壳 :538）	gate 取法的改动范围（本文件:309–313）里的 crates/*/src 路径（:550）与新写的判决（:558、:570）	0.8 秒（退 1）	无（agent-def-adversarial-review 与它同形、对象不同，本文件:215）	.claude/rules/implementation-workflow.md:11（三步的第 2 步）；规则在；C508 仍欠（.claude/kb/checks-owed.md:412：认「提到」不认「判过」）	红过：records/2026-09-16-subagent拆分提案.md:383 整轮门禁里 56 号红（点名的是别的会话的 crates/ 文件）；research/prompts/test-granularity-rename-evidence/head-bc57af7a-g56.log:1「40 个文件没有三方判决点名」。没核到哪一次红之后补了三方轮	红：crates/singlefs-checker-tier/src/lib.rs、crates/singlefs-checker-tier/src/verdict_store.rs 没有新判决点名（归属推的：工作区里别的会话的 KV 改动）
11-review-and-sync-records.sh	implementation-premise	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	标题日期不早于 2026-09-17 的三方正文 research/prompts/_*-body.md 里要出现 `crates/`；另判「本地腿：攻/辩 …」一行带理由、有背景材料的轮要有开工快照（本文件:10；判据 :105–114，外壳 :582）	research/prompts/_*-body.md（本文件:583、:609）、_<轮>-checklist.md 与 <轮>-snapshot/ 的 sha256 清单（:639）	0.1 秒（退 0）	无	.claude/rules/implementation-first.md:19（第 3 条）；规则在；材料员开工单跑这一格	抓到过：records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:60「第一个材料员开工前跑 58 号红：正文没有「本地腿：攻 …」那一行…主 agent 补那一行，58 号重跑无 ✗」	绿（查了 53 份）
11-review-and-sync-records.sh	abandoned-rounds	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	发过腿的三方轮次（五种文件名、轮名以 -r<N> 结尾）要么有判决（判决文件或 kb 同一小节既点名材料又有「**判决」），要么登记在 research/prompts/abandoned-rounds.tsv；登记表不许写坏（本文件:11；判据 :124–137，外壳 :657）	research/prompts/ 顶层文件名与 git log 里归档的腿文件（本文件:661、:140–142）、.claude/kb/ 各 .md 的小节（:663）、research/prompts/abandoned-rounds.tsv（:662）	0.6 秒（退 1）	无	没有规则或定义点名它（`grep -rn abandoned-rounds` 在 CLAUDE.md、main-agent.md、agent-common.md、.claude/rules/、.claude/agents/ 零命中）；依附里程碑二增补 2 收口表第 33 行（.claude/kb/milestone/02-second-txn.md:367），行在	抓到过：records/2026-09-16-subagent拆分提案.md:552「66 号第一次在真仓上跑查出第三轮撂下的 c364-r3」	红：m3-prune-gpu-r2（_m3-prune-gpu-r2-body.md）发过腿、没有判决也没登记撂下（归属推的：别的会话在途的轮）
11-review-and-sync-records.sh	knowledge-sync	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	改动范围里命中触发表的每个文件，都在改动范围里某份带 `<!-- knowledge-sync -->` 的 research/prompts/<阶段>-sync.md 里按路径出现；记录有「## 搜索」「## 命中处置」「## 原始证据」且合形（本文件:12；判据 :158–187 ①–⑤，外壳 :846）	.claude/gate.d/knowledge-sync-triggers.tsv（本文件:875）、research/prompts/*-sync.md（:1127）、gate 取法的改动路径与新增行（:309–331）	0.4 秒（退 1）	无（与 batch-scope 共用触发表，判的是不同的事，本文件:161–163）	.claude/main-agent.md:62（写 research/prompts/<阶段>-sync.md，「格式见门禁 11 号文件头 knowledge-sync 那一格」）；CLAUDE.md:22；C468 已还清（.claude/kb/checks-owed.md:543）；规则在	没找到记录（records/2026-09-16-subagent拆分提案.md:560 是立它的起因）	红：134 个触发文件没在任何同步记录里点名（改动范围里带标记的 sync 记录只有 1 份）
11-review-and-sync-records.sh	agent-def-adversarial-review	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 H 组；文件头没写 gate-category）	改动范围里的 .claude/agents/*.md、.claude/agent-common.md、.claude/main-agent.md 都要在这次新写的判决里按路径出现，或按内容哈希登记在 .claude/agent-def-review-exempt（本文件:13；判据 :214–220，外壳 :1190）	gate 取法改动范围（本文件:309）、新写的 research/prompts/*-main-verification.md（:1234）、.claude/agent-def-review-exempt（:1211）	0.8 秒（退 1）	无（与 crates-adversarial-review 同形，本文件:215）	.claude/rules/implementation-workflow.md:20；CLAUDE.md:22；规则在	红过：records/2026-09-16-subagent拆分提案.md:681「72 号立起来之后，工作区里今天改过的定义都判红，要另派一轮三方审它们」；research/prompts/defs-gatebatch-m2-g3-report.md:195 红 2 份	红：5 份没有新判决点名（.claude/agents/experiment-runner.md、kb-spec-drafter.md、prior-art.md 等，这一轮合并改过的定义）
12-doc-forbidden-notations-and-old-terms.sh	prime-marks	文档类（文件头 :5）	全仓文本不许出现撇号类角标 U+2032、U+2033、U+2034、U+02B9、U+02BA（本文件:4、:10；判据 :16–20，形态登记在 .claude/gate.d/lib-forbidden-notations.py 的 FORMS）	`git ls-files --cached --others --exclude-standard` 列出的每一份文本文件（本文件:18），排除上游副本（:23）	3.7 秒（退 1）	部分重叠：43 号 closeout-collects-open ④ 在收口表首列上也判撇号（43 号:223「全仓的 U+2032、U+2033 另由 12 号判；这一条多管的是 ASCII 单引号、字母…」）；写入钩子 .claude/hooks/write-guard.sh 读同一份库在写入时拒（本文件:29）	.claude/rules/path-moves.md:51；.claude/agent-common.md:25；规则在	抓到过：research/prompts/e116-r1-prereg.md:325、research/prompts/e159-r1-prereg.md:258 与 .claude/kb/experiments-history.md:7815 记它红、三份登记与两份源码里的角标名改成新编号	红：8 行在 4 份 research/prompts 冻结证据里（例 research/prompts/_defs-m2-closeout-r1-diff.md:746、research/prompts/m2-layer0-scale-r3-opus-output.md 4 行）
12-doc-forbidden-notations-and-old-terms.sh	clock-times	文档类（文件头 :5）	描述里的钟点、时区词与时间戳（ISO 时间戳、diff 头与 stat 的机器时间、不挨日期的时:分、光秃时区词），.sh/.py/.rs 带放行标记的行放过（本文件:11；判据 :18–26，形态在 lib-forbidden-notations.py）	与 prime-marks 同一趟扫描（本文件:21「两格在同一趟扫描里判」）；排除前缀逐项在 FORMS 的 excluded_prefixes（:23）	37.8 秒（退 0）	无门禁重复（doc-lint 不判钟点）；与 prime-marks 同一趟扫描、同一份库；write-guard.sh 写入时读钟点那一形态（本文件:29）	.claude/agent-common.md:24「报告与进度记录里的时间只写日期…（门禁 12 号判…）」；规则在	没找到记录（records/2026-09-16-subagent拆分提案.md:979 是原 17 号的样本自证）	绿（查了 4276 份文本文件）
12-doc-forbidden-notations-and-old-terms.sh	term-renames	文档类（文件头 :5）	research/scripts/sweep-term.py --check 按 .claude/kb/term-renames.md 查全仓不再出现旧名，排除表指向不存在的路径也红（本文件:12；判据 :31–35，外壳 :212）	.claude/kb/term-renames.md（本文件:219）、.claude/term-rename-exempt（:232）、research/scripts/replay.sh（:228）、全仓文件（sweep-term.py）	>5 分钟（300 秒 timeout 停掉）	无	.claude/rules/path-moves.md:26、:58；CLAUDE.md:80（仍写「门禁 90 号」）；规则在	没找到记录（records/2026-09-27-验证两档拆分.md:96 记的是改名把已归档产物名一起换掉、40 号找不到，修在 sweep-term.py；.claude/kb/checks-owed.md:370 C441 是它吃掉自己样本的坑）	不跑完（>5 分钟，300 秒 timeout 停掉；日志只有格名那一行）
13-code-vague-names-and-test-file-names.sh	vague-names	代码类（文件头 :5）	`git ls-files` 的 crates/**/*.rs 与 research/e7-index-bench/src/**/*.rs 里声明的 fn 与 struct/enum/type/trait 名，切出来的每个词都在 .claude/naming-vague-words 里就判违规；排除表只缩不涨（本文件:12；判据 :17–24，外壳 :79）	.claude/naming-vague-words（本文件:83）、.claude/naming-vague-exclude（:84）、crates/ 与 research/e7-index-bench/src/ 的 .rs（:105–109）	0.3 秒（退 1）	同对象不同判据：共享「命名纪律」naming-lint.sh 也扫我们声明的 .rs 名字（naming-lint.sh:8–9），判的是单字母与缩写（N1、N2，:11–14）；这一格判整名全是空泛词（本文件:8 gate-similar）	.claude/rules/verification.md:49；规则在	立它当天扫出存量：records/2026-09-27-验证两档拆分.md:47「已有的 41 份 research/ 文件登记进 .claude/naming-vague-exclude」；之后判红的记录没找到	红：research/e7-index-bench/src/bin/e162_verdict_store_network.rs:1994 struct CellRun（归属推的：工作区里别的会话的 E162 装置）
13-code-vague-names-and-test-file-names.sh	test-file-names	代码类（文件头 :5）	research/scripts/crash-case-check.py --only file-names：两个 tests/ 下的文件名不带里程碑序数、步号、增补号、并行线号、欠账号（本文件:13；判据 :25–29，外壳 :174）	crates/singlefs-harness/tests/*.rs、crates/singlefs-checker-tier/tests/*.rs 的文件名（本文件:26），判法在 research/scripts/crash-case-check.py（:181）	0.1 秒（退 0）	无（crash-case-check.py 的另三样由 14、54 号调，本文件:7 gate-similar）	.claude/rules/verification.md:18；规则在	没找到记录	绿（110 份测试文件）
14-harness-one-scenario-per-test.sh	one-scenario	harness 类（文件头 :5）	crash-case-check.py --only one-scenario：harness 档 tests/ 里带 #[test] 的函数，for 循环每一轮新建一个池（build_pool、build_through_*、format_pool、MemoryPool::with_devices）判红，写了 `// harness-test-granularity:one-scenario <理由>` 的不判（本文件:13；判据 :16–20）	crates/singlefs-harness/tests/*.rs（本文件:63–66），判法在 research/scripts/crash-case-check.py（:73）	0.3 秒（退 0）	无	.claude/rules/verification.md:27；规则在	没找到记录（records/2026-09-27-验证两档拆分.md:95「14 号在工作区照红」说的是当时 14 号里的耗时表那一样，已不在这一格）	绿（620 条用例）
15-research-build.sh	整道	代码类（推：records/2026-09-28-门禁59号提速与双机分片.md:66「research 构建与单测（15）」；文件头没写 gate-category）	research/ 工作区 `cargo test --release` 经内存包装跑过、至少一个测试批次；外加 5 份脚本自证：oov-check.py、stage-mine.py、check-staged.sh、relabel-item.py、claim-experiment.sh（本文件:4、:8–9；cargo :43，自证 :85–139）	research/Cargo.toml 与 research/ 工作区（本文件:25、:43–44）；research/scripts/ 下那 5 份脚本（:85、:101、:113、:125、:137）；research/scripts/run-with-memory-cap.sh（:36）	要编译，不跑	同一件事分两处：5 份研究脚本的自证在这里跑，其余 45 条在 47 号 research-script-selftests 的 runner 表（47 号:190–217），47 号覆盖段把这里「先赋变量再 "$变量" --selftest」算作有阶段在跑（47 号:232）；共享「构建与单测」check.sh 只管项目根 Cargo.toml 的工作区（check.sh:97 `cargo test --all`），research/ 是另一个工作区，不重叠	没有规则条款点名它（阶段自述 :5–13：research/ 的实验二进制背书 kb 结论）；对象还在：research/e7-index-bench/src/bin 下 153 份（33 号 absolute-assertions 今天的成功行）	红过、不是真问题：.claude/kb/checks-owed.md:425 C529 计时单测高负载下第一次红第二次绿；records/2026-09-24-里程碑二收尾调度.md:186 包装退 2 被 15 号当成测试红（判法的错，已改）。真问题的记录没找到	不跑（要编译）
20-doc-decision-documents.sh	kb-shape	文档类（文件头 :5）	1 未定项/已定项只许一种叫法；3 kb 文件不许链回自己；4 上游规则路径带 .claude/；5 决策标题连号带状态、两节不串味不重号、标题未定条数与列表相符；7 索引页状态列的分项计数与正文相符（本文件:16–17；判据 :426–436，:437 grid_kb_shape；第 7 段取 :127 起的 INDEX_VS_BODY_SOURCE）	.claude/kb/**/*.md 与 .claude/rules/*.md（本文件:448–450）、.claude/kb/decisions/*.md（:550）、.claude/kb/decisions.md（:712）	0.2 秒（退 0）	部分重叠：第 7 段与同一道的 decision-items-sync 判同一个量（索引表「状态」列的分项计数），本文件:1765–1768 写明是故意留的不经生成器的第二条路；第 5 段「标题未定条数」与 34 号 decision-links ⑦「首行…后面不带括注」同对象（.claude/rules/format-evolution.md:27「写了括注时 20 号判条数对不对、75 号判形态」）	.claude/singlefs-ai-sop/rules/kb-discipline.md:65；.claude/rules/format-evolution.md:27；规则在	没找到记录	绿
20-doc-decision-documents.sh	item-ref-status	文档类（文件头 :5）	每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与那条决策的「### 已定项」「### 未定项」索引相符（本文件:18；:719–728，判法在 .claude/gate.d/lib-item-ref-status.py 的 main）	kb、records、research、crates 全域，含 .rs 注释（本文件:725）；权威是各决策正文两张索引表（:724）	0.6 秒（退 1）	无（同一份库也被 settled-ref-says-open、freeze-layer-membership ③ 与 research/scripts/relabel-item.py 用，判的事不同，本文件:726；与 32 号 field-refs 分工见 32 号:115–118）	kb-discipline.md:65；.claude/kb/decisions.md:83 那一句还指着已删的 `.claude/gate.d/22-item-ref-status.sh`	抓到过：records/2026-09-24-里程碑二收尾调度.md:261「22 号另一处红是…用文件路径引「已定项 4、已定项 7」、归属判不出，改成 D13（验证路线） 已定项 4 的写法，22 号转绿」	红：131 处；列出的前 40 处都在 .claude/kb/decisions-history.md（例 :962「已定项 22」归属判不出）——史册改成按决策分节之后新出现的形态
20-doc-decision-documents.sh	status-redundancy	文档类（文件头 :5）	小节标题不带「—— 已定」；同一条目不同时写破折号与「状态：」；分项索引行的「状态：」与所在节一致（本文件:19；:733–751，:752 grid_status_redundancy）	.claude/kb/decisions 与 .claude/kb/experiments（本文件:753、:759）	0.1 秒（退 0）	同对象不同判据：34 号 decision-links ⑦ 也判决策首行与分项标题的写法（34 号:805–806：首行不带括注、分项标题不带日期、四块、定案格不超 100 字）	kb-discipline.md:65；规则在	没找到记录	绿（扫 263 份、核 351 条分项索引行）
20-doc-decision-documents.sh	cross-decision-status	文档类（文件头 :5）	kb 正文里紧贴「D<n>（简称）」写「未定」，而那条决策状态行不是待定就判红；跳过「」引文与历史节（本文件:20；:824–850，:851）	kb 正文（本文件:852–856），比 28 条决策的状态行	0.1 秒（退 0）	无（与 stale-open-items、settled-item-self-open、settled-ref-says-open 的分工在本文件:829–836、:947–949、:1051）	kb-discipline.md:65；C41 已还清（.claude/kb/checks-owed.md:578）；漏网写法 C61 仍欠（:62）	抓到过：.claude/kb/decisions-history.md:1721「由新阶段「说某条决策未定，而它已经定了」…抓出的四处」；.claude/kb/checks-owed.md:578「在真实语料上抓到 4 处真缺陷、零假阳性」	绿（扫 209 份 kb，比对 28 条决策）
20-doc-decision-documents.sh	settled-item-self-open	文档类（文件头 :5）	已定项区段里说它自己（自指）或同一决策另一条已定分项「仍未定 / 还没定 / 尚未定」判红（本文件:21；:939–962，:963）	.claude/kb/decisions/*.md 的已定项区段（成功行：28 条决策、6876 行）	0.1 秒（退 0）	无（分工在本文件:947–949）	kb-discipline.md「8. 正文只写现状」（:113）与第 4 条（:65）；规则在	抓到过：.claude/kb/decisions-history.md:1736「由新阶段「已定分项的正文里说自己还没定」…抓出的三处」（D16 已定项 5、D23 已定项 10、D23 已定项 8，本文件:949–951）	绿
20-doc-decision-documents.sh	settled-ref-says-open	文档类（文件头 :5）	引用写「已定项 k」、归属到的那条也已定，紧跟着却说「未定 / 没定 / 定不下 / 待定 / 空着」判红；两份变更史与 records/ 不扫（本文件:22；:1048–1058，:1059）	全仓除变更史与 records/（成功行：550 个文件、11250 处已定项引用），归属用 lib-item-ref-status.py	0.5 秒（退 0）	无（本文件:1051 写明另三格看不见这一型）	kb-discipline.md:65；规则在	没找到记录（本文件:1052–1053 记立它时手找出的六处）	绿
20-doc-decision-documents.sh	stale-open-items	文档类（文件头 :5）	还开着的未定项点名了另一条决策，而那条决策的状态行在这条未定项最近一次改动或复核之后又变过（看 git 历史）（本文件:23；:1100–1119，:1120；复核判据 :288）	.claude/kb/decisions/（本文件:1121）与 git log -G、git log -L（:1116–1119）	0.3 秒（退 0）	无（与 settled-same-file、blocking-verdict 是三把不同的尺，本文件:1896–1900）	C31 已还清（.claude/kb/checks-owed.md:576）；C36 仍欠（:41）；规则在	没找到记录	绿（查了 5 条未定项，1 条点名了定过东西的决策）
20-doc-decision-documents.sh	settled-same-file	文档类（文件头 :5）	这次改动在一份决策里新增了「已定」小节，而同文件还开着的未定项与索引页的待议节一个字没动（看 diff）（本文件:24；:1248–1275，:1276）	.claude/kb/decisions/*.md 与 .claude/kb/decisions.md 的 diff（本文件:1277–1278），基准经 changed-paths.sh（:1283）	0.6 秒（退 77）	无	C36 的前两条（本文件:1249）；C112「61 号阶段认不出「已定项 N」子标题」仍欠（.claude/kb/checks-owed.md:107）	没找到记录	77（这次改动没有新增已定小节）
20-doc-decision-documents.sh	freeze-layer-membership	文档类（文件头 :5）	冻结层归属登记表：每层、每个组件、每棵树两层、每个单元类码都有行；写「退出」的必是派生态；依据点名的分项在且已定（本文件:25；:1418–1448，:1449）	.claude/kb/freeze-layer-membership.md（本文件:1451）、D15 已定项 7（.claude/kb/decisions/15-*.md，:1456）、crates/singlefs-format/src/lib.rs 的 TREE_IDENTIFIER_*（:1452）、D18 已定项 11 登记表（:1457）	0.1 秒（退 0）	无	CLAUDE.md:81（仍写「门禁 16 号按它判三条」）；C45 已还清（.claude/kb/checks-owed.md:540）	没找到记录	绿（33 行）
20-doc-decision-documents.sh	decision-items-sync	文档类（文件头 :5）	decisions.md 的分项清单生成块与索引表「状态」列，与 .claude/scripts/gen-decision-items.py 的输出逐字相同；--write 只跑这一格（本文件:26；:1751–1771，:1772）	.claude/kb/decisions.md（本文件:1775）与各决策正文经生成器（:1778）	0.1 秒（退 0）	部分重叠：与同一道 kb-shape 第 7 段判同一个量，本文件:1765–1768 写明是故意留的两条独立路径	.claude/rules/path-moves.md:33 仍写「跑 `21-decision-items-sync.sh --write`」，那份文件已删；规则在，脚本名过期	没找到判红而是真缺陷的记录（它是投影，records/2026-09-28-里程碑二收尾接手.md:35 记的是跑 --write 同步）	绿（28 条决策）
20-doc-decision-documents.sh	blocking-verdict	文档类（文件头 :5）	每条未定项的登记行都写了「改新池新建文件的字节：是/否/无对象（日期…）」与「动不动格式」两把尺；定位条数与生成器对不上也红（本文件:27；:1886–1944，:1945）	.claude/kb/decisions/*.md（本文件:1946），未定项清单取自 gen-decision-items.py（:1948）	0.1 秒（退 0）	无（32 号 first-txn-hooks 读同一句判定、判的是判「是」的有没有挂钩，32 号:680、:697）	.claude/rules/format-evolution.md:29；C50 仍欠（.claude/kb/checks-owed.md:51，已还一半）；C224 仍欠（:199）	红过：records/2026-09-16-subagent拆分提案.md:410 副本演练里「20、31 号红是主 agent 的规格少给了状态句与「改不改新池新建文件」」；本文件:1893–1894 记立它时现查 23 条里 20 条没判过	绿（5 条未定项两把尺都判过）
20-doc-decision-documents.sh	user-verdict-owed	文档类（文件头 :5）	决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还（没有带日期的还清标记）的账按整号点名那条决策（本文件:28；:2065–2081，:2082）	.claude/kb/decisions（本文件:2087）、.claude/kb/checks-owed.md（:2084）	0.1 秒（退 77）	无（与 43 号方向相反，本文件:7 gate-similar）	.claude/singlefs-ai-sop/skills/decide/SKILL.md:39 硬要求第 6 条；规则在	抓到过：records/2026-09-05-SOP剥离轮.md:44「第一次跑在真实语料上就咬到 D16（发布语义） 那一处」	77（正文里没有一处「待用户复核」）
20-doc-decision-documents.sh	decision-summary-width	文档类（文件头 :5）	决策索引表每一行「结论」列不超过 decisions.md 那一句写的字数上限（上限从文档读）（本文件:29；:2129–2141，:2142）	.claude/kb/decisions.md（本文件:2143）	0.1 秒（退 0）	无（34 号 decision-links ⑦ 的「不超过 100 字」管的是各决策正文分项索引的定案格，34 号:806，不是这张表）	.claude/kb/decisions.md:14 那一句（仍写 `.claude/gate.d/37-decision-summary-width.sh`）；规则在	没找到记录（本文件:2134–2135 记立它时实测七行超标）	绿（28 行，上限 200 字）
27-code-constants-enums-bits-match-kb.sh	format-constants	代码类（文件头 :5）	kb 里 `<!-- format-const: 名字 = 值 stale=… -->` 标记的值与 research/、crates/ 源码同名 const 的整数字面量相等，stale= 旧串不再出现在 kb 正文与实验源码；标记读不出、同名登记两次判红（本文件:19；:100–131，:132）	.claude/kb/**/*.md 的标记（本文件:151），research/**/*.rs 与 crates/**/*.rs（:178），按 .claude/gate.d/lib-format-const.py 读	0.5 秒（退 0）	无（与 32 号 field-table-sums、92 号共用 lib-format-const.py 读标记，判的事不同，本文件:7–8 gate-similar）	.claude/rules/format-evolution.md:64；C56、C187 仍欠（.claude/kb/checks-owed.md:57、:167）；规则在	抓到过：records/2026-09-24-里程碑二收尾调度.md:54「记录头常量 307 → 311（门禁 27 号）」；.claude/kb/experiments-history.md:5844「登记值改成 105 / 103 之后不改源码就红」；research/prompts/m2-impl-rbf-1-implementer-report.md:248 改前红在 crates/singlefs-format/src/lib.rs:223 的 481	绿（51 个已登记，51 个在源码里被钉住）
27-code-constants-enums-bits-match-kb.sh	clause-enums	代码类（文件头 :5）	按 .claude/gate.d/clause-enum-pairs.tsv 逐对：枚举每个变体都登记、表里的变体存在、表里名字逐字是那条分项正文「 / 」列举的一项、正文列举的每一项都有行（本文件:20；:238–256，:257）	.claude/gate.d/clause-enum-pairs.tsv（本文件:258；今天 19 行非空，6 行登记，全是 D17 已定项 2 对 crates/singlefs-harness/src/segments.rs:StepKind）、.claude/kb/decisions/ 分项正文（:286）、crates 源码里的枚举	0.1 秒（退 0）	无	没有规则或定义点名它；依据是 records/2026-09-23-三轮三方与四类没东西会红.md:23（立它的记录）	没找到记录	绿（6 对成员、1 个枚举、1 条分项）
27-code-constants-enums-bits-match-kb.sh	feature-bits	代码类（文件头 :5）	feature-bits.md 与 D15 已定项 4 登记表的（类别，位号）逐行一致且语义包含含义；每张位图位号 0..n-1 不跳；crates 代码里 INCOMPAT/COMPAT_RO/COMPAT 常量的位号都在记账表；同一位不登记两行（本文件:21；:425–444，:445）	.claude/kb/feature-bits.md（本文件:446）、.claude/kb/decisions/15-*.md（:489）、crates/*/src/**/*.rs（:729）	0.2 秒（退 0）	部分重叠（数据源）：92 号 ③ 也读 D15 已定项 4 登记表，判 layouts.tsv 的 incompat 位号有行、名字与含义列逐字相同（92 号:10–11）	CLAUDE.md:79（仍写「门禁 93 号判两处逐位一致」）；.claude/rules/fs-design.md:84；C11 已还清（.claude/kb/checks-owed.md:547）	没找到记录	绿（记账表 2 位、登记表 2 位；60 个 .rs 里 4 处常量）
30-decision-history-entries.sh	entry-added	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 C 组；文件头没写 gate-category）	决策正文改了（增删合计超过 4 行或碰了状态行），decisions-history.md 里要有新增条目（按新增的 #### 数，没有就按 ### 日期数）（本文件:11；:17–36，:132）	.claude/kb/decisions/ 与 decisions.md 的 diff、.claude/kb/decisions-history.md 的新增行（本文件:134–135），基准经 changed-paths.sh 的 gate 取法（:27–28、:121）	0.3 秒（退 0）	无	.claude/rules/format-evolution.md:18–19「决策变更必须记进决策变更史」；.claude/rules/changelog-format.md:52；规则在	没找到判红而是真缺陷的记录；records/2026-08-29-复跑复核轮.md:199 记的是它当时「从来没有能力判红」（判法的错，本文件:32–35）	绿（决策正文改了 389 行，变更史新增 1182 条条目）
30-decision-history-entries.sh	shape	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 C 组；文件头没写 gate-category）	只判新增的：a 不许「### 日期（其…）」粘在一行；b 两份变更史的 ### 日期 块要住在自己的 `## D<n>（` / `## E<n>（` 节；c 同一节同一日期不许出现两次（本文件:12；:37–54，:193）	.claude/kb/ 下全部 .md、decisions-history.md、experiments-history.md（本文件:199–200）；新增行按 git diff -U0 的 hunk 头取（:49–50）	0.2 秒（退 0）	同对象不同判据：共享「历史条目编号」history-ordinal.sh 也只判新增的变更史标题，判的是「（其 N）」撞号（history-ordinal.sh:5、:15–18）；这一格判粘连、落节与同节同日（本文件:5 gate-similar）	.claude/rules/changelog-format.md:7–43、:52（这一轮新立的按决策分节形态）；规则在。原 48 号「条目住在它日期所在月的那一份」随月份文件撤掉，.claude/kb/decisions-history/ 今天不存在	没找到记录（原 48 号的判红记录也没找到）	绿（查了 210 份 kb、1109 个日期标题；只判基准之后新增的）
30-decision-history-entries.sh	status-sync	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 C 组；文件头没写 gate-category）	两份变更史每个 `## D<n>（` / `## E<n>（` 节顶上的「**现状**：」一行与 decisions.md / experiments.md 索引表那一行的「状态」「结论（简报）」一致；节与索引行一一对应；--write 只重写这一行（本文件:13；:55–66，:394）	.claude/kb/decisions.md、decisions-history.md、experiments.md、experiments-history.md（本文件:398–399）	0.1 秒（退 0）	部分重叠（数据源）：34 号 index-sync 也读 experiments.md 索引表，对的是实验正文标题（34 号:22）	.claude/rules/changelog-format.md:35–36（节顶上写一行现状）；规则在。原 49 号「决策变更史的快查与原文同步」随月份文件撤掉	没找到按今天形态判红的记录；records/2026-09-28-里程碑二收尾接手.md:35「30 号快查同步格判红」是改形态之前的快查格	绿（决策 28 节、实验 162 节）
32-doc-field-and-layout-registry.sh	field-refs	文档类（文件头 :5）	layout/01-first-txn.md 每行「指向」列写的「D<n>（简称） 已定项 k / 未定项 k」在那条决策「### 已定项 / ### 未定项」索引里真实存在（本文件:22；:106–123，:124）	.claude/kb/layout/01-first-txn.md（本文件:126）、.claude/kb/decisions/ 的索引（:128、:138）	1.2 秒（退 0）	无（与 20 号 item-ref-status 分工：那一格判状态、前提是分项存在，这一格判存在，本文件:115–118）	C66 仍欠（.claude/kb/checks-owed.md:67；records/2026-09-28-里程碑三第十项清单.md:292：只做了「指了而分项不存在」那一半）；对象是里程碑一的字节表，里程碑一出口已满足（CLAUDE.md:8），宽度与落点仍以 01 为准（CLAUDE.md:87）	没找到记录	绿（查了 408 处引用）
32-doc-field-and-layout-registry.sh	field-projection	文档类（文件头 :5）	被 layout/01 投影过的分项，它的 `| 字段 | 宽 |` 字段表每一行字段名要在 layout/01 引用了那条分项的节的表格行里出现（本文件:23；:196–215，:216）	.claude/kb/layout/01-first-txn.md（本文件:217）、.claude/kb/decisions/*.md 的字段表（:218、:242）	0.1 秒（退 0）	无	依附 layout/01（里程碑一的字节表，CLAUDE.md:87）；没有规则条款点名	没找到记录（本文件:201–203 记立它那天的实测：D8 已定项 8 ② 加了水位而投影表没动）	绿（3 张字段表、25 行；投影表引用 108 条分项）
32-doc-field-and-layout-registry.sh	field-table-sums	文档类（文件头 :5）	字段表表后「合计 N 字节」、表前说明句的总宽与增量、一个分项里唯一的 format-const 标记，都等于字段表之和；字节布局表里认得出的结构总宽要有 format-const 登记（本文件:24；:287–332，:333）	.claude/kb/decisions/*.md 与 .claude/kb/layout/*.md 的正文（本文件:457–458），标记经 lib-format-const.py（:643）	0.1 秒（退 0）	无（与 27 号 format-constants、92 号共用 lib-format-const.py，判的事不同，本文件:6 gate-similar）	C94 已还清（.claude/kb/checks-owed.md:535）；行内算式那一半 C305 仍欠（:262）	抓到过：records/2026-09-24-里程碑二收尾调度.md:68「39 号：布局表 12 处结构总宽没登记，仍红」	绿（表后合计 2 张、说明句 4 张、标记 2 个、结构总宽 15 处）
32-doc-field-and-layout-registry.sh	first-txn-hooks	文档类（文件头 :5）	判「是」且仍未定的未定项在字节表与里程碑各被引一次；空白表只收判「是」的；字节表每节有「**里程碑**：… 步 N」；里程碑每步「**写出的字节**：」点名存在的节（本文件:25；:677–703，:704）	.claude/kb/layout/01-first-txn.md（本文件:705）、.claude/kb/milestone/01-first-txn.md（:706）、.claude/kb/decisions/（:707）	0.1 秒（退 0）	无（与 20 号 blocking-verdict 读同一句判定，本文件:680；与 second-txn-hooks 同形不同里程碑，:17–20）	依附里程碑「新池新建文件」（milestone/01-first-txn.md），出口已满足（CLAUDE.md:8）；三份文件都在；今天判「是」的未定项 0 条	没找到记录（本文件:684–686 记立它那天的实测三处）	绿（判「是」的未定项 0 条、字节表 11 节、里程碑 8 步）
32-doc-field-and-layout-registry.sh	admission-terms	文档类（文件头 :5）	准入不等式（`<!-- gate:admission-formula -->` 下 `可用 =` 那一行）每一项在对照表（`<!-- gate:admission-terms -->`）有归属：统计量表里的「第 N 项」或写明的例外；decisions/ 里式子的每处副本与权威一行逐字相等（本文件:26；:817–842，:843）	.claude/kb/decisions/*.md 里两个锚、统计量表与式子副本（本文件:844、:849）	0.1 秒（退 0）	无	D5 已定项 4 用户定案（本文件:819–820）；规则在	没找到判红的真缺陷记录；research/prompts/e156-preregistration.md:325 拿它当 E156 岔路 2 的判别力（「门禁 51 号必须由红转绿」）	绿（8 项：统计量 4、例外 4；副本 2 处与权威一致）
32-doc-field-and-layout-registry.sh	segment-registry	文档类（文件头 :5）	layout/01「八、根槽写路径的段序列登记表」每行段序列与步骤种类多重集，和 E142 产物、钉住的用例逐字比对（本文件:27；:992–1017，转发 research/scripts/check-segment-registry.py，:1018）	.claude/kb/layout/01-first-txn.md 第八节、research/scripts/replay.sh 里 E142 那一行、research/results/ 的 E142 产物、钉住的用例（本文件:999、:1012–1013）	0.1 秒（退 0）	部分重叠（数据源）：55 号 direct 档「段序列与 E142 产物逐字相同」（55 号:13–14）比的是同一份产物，一边是虚机实跑、一边是登记表	C316 已还清（.claude/kb/checks-owed.md:562）；C485 已还清（:504）；C524 仍欠（:420）；对象在（成功行点名 r19 产物）	没找到真缺陷记录；records/2026-09-16-subagent拆分提案.md:959 记的两处是脚本自己判错	绿（4 处登记）
32-doc-field-and-layout-registry.sh	format-const-placeholders	文档类（文件头 :5）	crates/singlefs-format/src 里每个 `// placeholder:` 占位点名的 D<n> 分项号在那条决策两张索引表里、C<n> 在 checks-owed.md 有登记行（本文件:28；:1027–1040，:1041）	crates/singlefs-format/src/**/*.rs（本文件:1048）、.claude/kb/decisions/（:1059）、.claude/kb/checks-owed.md（:1072）	0.1 秒（退 0）	部分重叠（判据）：10 号 decision-refs 也判 D 号有定义，射程是 kb 与规则；这一格只看常量文件注释、还核分项号与 C 号；简称一致归 doc-lint / number-name-sync（本文件:1036）	里程碑「新池新建文件」步 0 的要求（本文件:1029–1030），里程碑一出口已满足；常量文件还在、今天 4 个占位	没找到记录	绿（4 个占位）
32-doc-field-and-layout-registry.sh	second-txn-hooks	文档类（文件头 :5）	layout/02 表每行六格；点名的步存在且那步「写出的字节」点名这一行；里程碑点名的 layout/01 节或 layout/02 行存在；钉住的用例文件与函数在；「八、」表标到的步与点名它的步两边相等（本文件:29；:1091–1111，:1112）	.claude/kb/layout/01-first-txn.md、layout/02-second-txn.md、milestone/02-second-txn.md（本文件:1113–1115），crates/*/tests/（:1206–1208）	0.1 秒（退 0）	无（与 first-txn-hooks 同形不同里程碑，本文件:17–20）	里程碑二增补 2 收口表第 31 行（.claude/kb/milestone/02-second-txn.md:365）；当前里程碑，行在	抓到过：records/2026-09-19-里程碑二遗留收拢.md:113「门禁 76 号立了，当天抓到一处（步 5 没点名 layout/02 的 defer 行，已补）」	绿（登记表 9 行、点名 34 处、用例 22 个）
32-doc-field-and-layout-registry.sh	tree-table-reserve	文档类（文件头 :5）	带 `<!-- gate:tree-table-reserve total=N -->` 的认购表：宽度全是正整数、合计不超过 total、每行出处带一个存在的 C/D 编号；带标记的表恰好一张（本文件:30；:1244–1263，:1264）	.claude/kb/decisions/08-核心索引结构.md 已定项 11 那张表（本文件:1343）、.claude/kb/checks-owed.md（:1299）	0.1 秒（退 0）	无	C338 已还清（.claude/kb/checks-owed.md:492）；里程碑二收口表第 17 行（.claude/kb/milestone/02-second-txn.md:348）	立表当天抓到正文 48 而写「共 56」（本文件:1247–1249）；之后判红的记录没找到	绿（5 条认购，合计 48、预留 76、余 28）
33-code-experiment-and-mutation-source-discipline.sh	mutation-tables	代码类（文件头 :5）	research/e7-index-bench/src/bin/ 每个 .rs 在 research/mutations/ 有同名成形表、锚点命中一次；crates/mutations.tsv 经 crates-mutation-rows.py static-check 判成形、锚点、测试名、转义、拷贝范围，另判点名 ignored 却没带 --include-ignored、筛选词筛不到、重复行；crates/ 下逃出拷贝范围的 "../ 字面量（本文件:20；:94–130，:131）	research/e7-index-bench/src/bin/*.rs（本文件:132）、research/mutations/*.tsv（:133）、crates/mutations.tsv 与它点名的源码与测试（:199）、.claude/gate.d/stage-inputs.tsv 59 号那一行（:444）、research/scripts/crates-mutation-rows.py（:343）	1.9 秒（退 0）	重复：crates 那一半的成形、锚点、测试名、转义、拷贝范围与 59 号派活之前判的是同一份 row_problems（59 号:8–10、:14；本文件:106–108），整轮门禁里同一批判据跑两遍（这里静态、59 号重型）	.claude/rules/mutation-sampling.md:65、:73；.claude/singlefs-ai-sop/rules/show-me-test.md:26；C40 仍欠（.claude/kb/checks-owed.md:44）；C327 已还清（:556）	抓到过：research/prompts/m2-rev-a3-checker-2-implementer-report.md:96「变异表第 990 行…原文命中 0 次（33 号红）」锚点腐化；records/2026-09-22-并行线二三四与捶打.md:53 立重复判据当场抓到一对（合法，判据随之收窄）	绿（153 个实验二进制、1749 条变异；crates/mutations.tsv 1418 条）
33-code-experiment-and-mutation-source-discipline.sh	absolute-assertions	代码类（文件头 :5）	每个实验二进制至少一条把量与数字字面量比死的断言（assert_eq!(…, 数) 或 assert!(… ==/<=/>=/</> 数)，按语句配平认多行）（本文件:21；:467–514，:563）	research/e7-index-bench/src/bin 下全部、别处 crates/*/src/bin 与 research/*/src/bin 下以 e<数字>_ 开头的（本文件:564、:571）	0.4 秒（退 0）	无	.claude/singlefs-ai-sop/rules/test-discipline.md:62；规则在	抓到过：.claude/kb/experiments/31-AAD缺快照维.md:8「门禁阶段 80 当场判红」；.claude/kb/experiments-history.md:2257	绿（158 个实验二进制）
33-code-experiment-and-mutation-source-discipline.sh	reading-discipline	代码类（文件头 :5）	实验源码 C59：名字带 seed 的标识符后面直接跟 `| 1` / `& !1`；C60：整型结构体字段从不出现在写入位置而初始化都是字面量 0；两张豁免表挂开着的欠账、对得上真违规、按文件只缩不涨（本文件:22；:600–648，:649）	research/e7-index-bench/src/bin/*.rs 与 crates/singlefs-checker-tier/src/bin/e*.rs（本文件:665）、.claude/gate.d/experiment-seed-fold-lag.tsv 与 experiment-constant-reading-lag.tsv（:668–669）、.claude/kb/checks-owed.md 经 lib-owed.py（:667）	13.6 秒（退 0）	无	C59、C60 仍欠（.claude/kb/checks-owed.md:60、:61）；豁免表今天 17 行放行 17 处	没找到记录	绿（158 个文件；C59 折叠写法 17 处都在豁免表）
34-doc-experiment-pages-and-products.sh	index-sync	文档类（文件头 :5）	实验正文标题的作废 / 未跑 / 部分已跑 / 够判与 experiments.md 索引行一致；索引结论列的数正文里找得到；两边一一对应（本文件:22；:159–185，:186）	.claude/kb/experiments.md（本文件:187）、.claude/kb/experiments/（:188）	0.1 秒（退 0）	部分重叠（数据源）：30 号 status-sync 也读 experiments.md 索引表，对的是 experiments-history.md 节顶上的现状行（30 号:13）	kb-discipline.md:65；规则在	没找到记录（.claude/kb/experiments-history.md:4363 是立它的记录）	绿（索引 162 行、正文 162 份）
34-doc-experiment-pages-and-products.sh	results-cited	文档类（文件头 :5）	research/results/ 下每个产物文件在 experiments.md 或实验页被点名；标着已跑的实验点了产物；这次改动新点名的产物在树里或历史里找得到（本文件:23；:271–284，:285）	research/results/（本文件:294）、.claude/kb/experiments.md 与 experiments/（:288、:292）、research/scripts/replay.sh（:301）、改动范围新增行（:395）	4.3 秒（退 1）	无（与 verdict-false-named、quoted-result-lines、repro-command 的分工在本文件:1344–1347、:1520）	.claude/agent-common.md:99（产物先落盘）；反向那一半 C43 仍欠（.claude/kb/checks-owed.md:46）	抓到过：research/prompts/e142-r15-s1-runner-report.md:124 记 40 号点名 7 份 E156、E158 产物没写回；records/2026-09-27-验证两档拆分.md:96 记 40 号点名 14 份已归档产物找不到（改名把引用换了）	红：research/results/e162-crash-verdict-block-store-mutate-2026-09-27-features.log 等 E162 产物没被点名（归属推的：别的会话在途）
34-doc-experiment-pages-and-products.sh	evidence-in-repo	文档类（文件头 :5）	改动范围里的实验装置与变异表在 research/results/ 要有不比它旧（或输入指纹对得上）的产物；kb 与这一轮新写的 prompts 里不许把 /tmp 路径当依据引用（本文件:24；:472–523，:524）	改动范围（本文件:125–157 取一次）、research/e7-index-bench/src/bin/e*.rs 与 research/mutations/e*.tsv（:577–578）、research/results/（:572）、stage-inputs.tsv 与 admission.py（:556、:661）、research/on-request-experiments.tsv（:581）、.claude/kb/ 与 research/prompts/（:573–574）	2.5 秒（退 1）	无	.claude/agent-common.md:99（仍写「门禁 69 号判这一条的形式」）；规则在	抓到过：records/2026-09-24-里程碑二收尾调度.md:231「门禁 69 号第二类清零」、:276「69 号 /tmp 引用清零」	红：research/e7-index-bench/src/bin/e101_node_tag_reserve.rs（改于 2026-09-28）在 research/results 下一份 E101 产物都没有（工作区里改过的装置）
34-doc-experiment-pages-and-products.sh	decision-links	文档类（文件头 :5）	①–⑨：影响的决策表形状、表里决策与分项存在、支撑/推翻双向登记、回看不过期、这次改动该回看的回看了、待回填清单只减不增、瘦身形态、有实验才有决策、实验必须对应决策（本文件:25；:783–820，:821）	.claude/kb/experiments/ 与 .claude/kb/decisions/（本文件:822）、.claude/decision-links-pending（:984）、research/results/e*（:1230）、新写判决 research/prompts/*-main-verification.md（:1272）、改动范围（:832）	2.5 秒（退 1）	部分重叠（对象）：⑦ 与 20 号 kb-shape 第 5 段、status-redundancy 同判决策首行与分项索引（.claude/rules/format-evolution.md:27）；⑨ 吸收了原 10 号第 4 项「结论悬空」（research/prompts/gate-fix-forks-r2-main-verification.md:29）	.claude/rules/format-evolution.md:40、:54；规则在；C520、C521 仍欠（.claude/kb/checks-owed.md:416、:417）	抓到过：records/2026-09-24-里程碑二收尾调度.md:261「75 号那一处不对称（D23 已定项 14 依据引了 E158、E158 页那一行还是「备料」）主 agent 判「支撑」…75 号转绿」；records/2026-09-19-决策瘦身与双向登记.md:159 E105、E69 两页当场红（原 10 号第 4 项，今 ⑨）	红：2 处没回看：.claude/kb/experiments/31-AAD缺快照维.md 正文改了、影响的决策表一行没回看；E159 产物 research/results/e159-fsync-wait-group-commit-2026-09-24-h311-smoke.out 变了
34-doc-experiment-pages-and-products.sh	verdict-false-named	文档类（文件头 :5）	replay.sh 登记产物里 `E7RESULT name=verdict` 行的每个 字段=false，在对应实验页正文或最新历史条目的同一行被点名（本文件:26；:1313–1351，:1352）	research/scripts/replay.sh 登记表（本文件:1353）、research/results/ 产物（:1381）、.claude/kb/experiments/（:1382）	0.1 秒（退 0）	无	没有规则条款点名；依据 records/2026-09-16-subagent拆分提案.md:944（本文件:1342）	没找到记录（records/2026-09-16-subagent拆分提案.md:944 是立它的起因）	绿（点名 15 个）
34-doc-experiment-pages-and-products.sh	repro-command	文档类（文件头 :5）	点名了 research/results/ 产物的实验页要写 `cargo run --release --bin X` 或点名 replay.sh / vm-bench.sh；写明原始输出未留存的不管（本文件:27；:1511–1525，:1526）	.claude/kb/experiments/（本文件:1527）	0.1 秒（退 0）	无	没有规则条款点名（本文件:1517–1520 写立它的原因）	抓到过：.claude/kb/experiments-history.md:622「第一次跑就红了 16 个文件，本轮逐个补齐」	绿（点了产物的 18 页）
34-doc-experiment-pages-and-products.sh	experiment-orphans	文档类（文件头 :5）	research/ 下（不含 target/）以 eNN 命名的东西与 crates/singlefs-checker-tier/src/bin/eNN_*.rs，在 kb/experiments/ 要有同号正文（本文件:28；:1571–1583，:1584）	research/ 全树文件名、crates/singlefs-checker-tier/src/bin（本文件:1590）、.claude/kb/experiments（:1585）	0.1 秒（退 0）	无（与 10 号 experiment-refs 方向相反：那一格从 kb 引用查正文，这一格从 research 文件名查正文）	没有规则条款点名（本文件:1578–1581 写立它的原因）	抓到过：.claude/kb/experiments-history.md:629「新增门禁阶段 86 抓出来的」	绿（157 个实验号）
34-doc-experiment-pages-and-products.sh	quoted-result-lines	文档类（文件头 :5）	kb 与 research 正文里去掉首尾空白后以 `E7RESULT ` 开头的行，在 replay.sh 登记的产物（或归档历史）里逐字找得到；只判这次改动新增或改写的行（本文件:29；:1626–1652，:1653）	.claude/kb/**/*.md、research/**/*.md（本文件:1678–1679）、replay.sh 登记的产物（:1710）、research/results/*.out（:1726）与归档历史	2.8 秒（退 0）	无（与 results-cited、verdict-false-named 的分工在本文件:1344–1347；与 43 号 audit-contradictions 方向相反，43 号:12 gate-similar）	.claude/singlefs-ai-sop/rules/evidence-discipline.md:128「引产物就整行抄」；规则在	立它当天扫出 E139 两行、E144 一行（本文件:1633–1636）；之后判红的记录没找到	绿（判了 98 行，对照 158 份登记产物）
34-doc-experiment-pages-and-products.sh	archive-past-rounds	文档类（文件头 :5）	research/prompts 与 research/results 里上一轮及更早的实验记录已归档（删掉），保留判决、abandoned-rounds.tsv、还被代码当输入的产物（本文件:30；:1819–1836，:1837；判法在 research/scripts/archive-past-rounds.py）	research/prompts、research/results（本文件:1838），基准 gate_diff_base gate 经 GATE_BASE 交给 archive-past-rounds.py	6.3 秒（退 1）	无	.claude/singlefs-ai-sop/rules/evidence-discipline.md:27；.claude/agent-common.md:28（仍写 `.claude/gate.d/91-archive-past-rounds.sh`）；C450 仍欠（.claude/kb/checks-owed.md:376：与 knowledge-sync 打架）	没找到记录	红：还留着上一轮及更早的实验记录 1816 份
34-doc-experiment-pages-and-products.sh	multipath-registry	文档类（文件头 :5）	写了「### 路径与结论登记」的实验页：表头逐字、五格、源码落点文件在或写「无实现：理由」、共用项不空不写「无」；滞后表两列、指向现存且还没写登记节的页、只缩不涨（本文件:31；:1854–1887，:1888）	.claude/kb/experiments/（本文件:1889）、.claude/gate.d/multipath-registry-lag.tsv（:1890）、基准（:1894）	2.3 秒（退 0）	无	.claude/rules/format-evolution.md:46–54；C48 仍欠（.claude/kb/checks-owed.md:49）	没找到记录	绿（5 份、21 条路径；滞后 9 份）
43-checks-owed-and-closeout.sh	table-shape	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 E 组；文件头没写 gate-category）	checks-owed.md 两张登记表每行格数对得上表头（欠着六列、已还清四列）；同一编号不许在两张表各登记一次；欠着那张不许留开头写「已还（非一半）/销账/作废」的行；编号行前面要有表头（本文件:20；:98–123，:124）	.claude/kb/checks-owed.md 里紧挨 `doc-lint:registry` 标记的两张表（checks-owed.md:13、:484）	0.1 秒（退 0）	部分重复：「同一个编号在两张表里各登记一次也判红」（本文件:103）与共享「文档铁律」doc-lint F「编号 $id 有多处登记位——一个编号只许有一处登记」（doc-lint.sh:770–777）判同一件事：两张表都带 `<!-- doc-lint:registry name-col=2 -->`（checks-owed.md:13、:484）。格数、已还行、表头三样 doc-lint 不判	kb-discipline.md:69；.claude/kb/checks-owed.md:478（「这两样门禁 43 号判红」）；规则在	抓到过：research/prompts/m2-kb-writeback-stable-drafter-report.md:11 仓副本演练里「C554 行里代码串的 | 把表格切成 8 格、43 号红」，规格里修掉	绿（检查了 568 行，2 张表）
43-checks-owed-and-closeout.sh	closeout-collects-open	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 E 组；文件头没写 gate-category）	带 `<!-- milestone:closeout-table -->` 的里程碑文件，全文点名、还开着的 C 号要么进收口表、要么表后「- 不收口 C<n>（简称）：理由」；收口表首列只许十进制顺序号或 ①–⑳（本文件:21；:205–231，:232）	.claude/kb/milestone/*.md（本文件:240）、.claude/kb/checks-owed.md 经 lib-owed.py（:241）	0.1 秒（退 0）	部分重叠：④ 在收口表首列上的 U+2032、U+2033 与 12 号 prime-marks 全仓那一格判同一样（本文件:223）	里程碑二增补 2 收口表第 34 行（.claude/kb/milestone/02-second-txn.md:368）；当前里程碑，行在	没找到记录（records/2026-09-16-subagent拆分提案.md:550 是立它的起因）	绿（判了 1 份：02-second-txn.md 点名 118 个 C 号，开着 66 个：表里 55、豁免 11）
43-checks-owed-and-closeout.sh	paid-cited-tests	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 E 组；文件头没写 gate-category）	「### 已还清」表里反引号括起的三段以上 snake_case 标识符，要在仓里 .rs/.py/.sh 的非注释代码里按整词找得到（本文件:22；:372–397，:398）	.claude/kb/checks-owed.md 已还清表（本文件:399）；仓里 .rs/.py/.sh，排除 .claude/gate.d/ 与 research/prompts/（:389–395）	0.8 秒（退 1）	无	没有规则条款点名（本文件:378–380 写它是那条定案唯一的记录位）	抓到过：records/2026-09-24-里程碑二收尾调度.md:248「78 号：C283（已还清表）引的用例…被 A4e 改了名，正在改成新名」	红：C313 引的 `fua_not_a_boundary_gives_134217754_states` 在仓里的 .rs / .py / .sh 里一处都找不到（本文件:384–386 记立它当天抓到的也是 C313 那一行引的单测名）
43-checks-owed-and-closeout.sh	audit-contradictions	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 E 组；文件头没写 gate-category）	最近一次总审核记录第五节每行处置列以「已改」开头并用「」引的原文在点名的 kb 文件今天正文里找得到，或点名一个开着的 C 号；.claude/audit-rows-pending 不许有行（本文件:23；:498–534，:535）	records/*-总审核.md 最近一份（本文件:538；今天是 records/2026-09-13-总审核.md）、checks-owed.md 经 lib-owed.py（:561–563）、点名的 kb 文件（:609–623）、.claude/audit-rows-pending（:536）	0.1 秒（退 0）	无（与 34 号 quoted-result-lines 方向相反，本文件:12 gate-similar）	C358 已还清（.claude/kb/checks-owed.md:536）；对象 records/2026-09-13-总审核.md 第五节 29 行	抓到过：records/2026-09-24-里程碑二收尾调度.md:68「81 号：总审核第五节 4 行处置列主 agent 已换成今天原文，转绿」、:70 81 号报错顺带查出 E123、E71、E97 与里程碑文件还原样引旧式	绿（第五节 29 行判了 29 行）
43-checks-owed-and-closeout.sh	row27-preconditions	文档类（推：records/2026-09-28-门禁59号提速与双机分片.md:65 的 E 组；文件头没写 gate-category）	收口表第 27 行那几笔「今天不可达」欠账各一条逐字探针（文件、原文、今天命中次数）：全对上退 77，任一条对不上判红；另报没做成探针的几笔（本文件:24；:731–751，:752）	crates/singlefs-core/src/mount.rs、instance_table.rs 等探针文件（本文件:757、:760）、.claude/kb/milestone/02-second-txn.md 第 27 行（:816）	0.1 秒（退 77）	无（与 33 号 mutation-tables 的锚点计数同形不同事，本文件:9 gate-similar）	里程碑二增补 2 收口表第 27 行（.claude/kb/milestone/02-second-txn.md:361）今天写「这一行已有去向，不再等前置」，余下仍不可达的由这一格盯；行在	红过、不是前置进来：records/2026-09-23-三轮三方与四类没东西会红.md:68「89 号当天就红了一次——mount.rs 重构改了变量名…重核之后确认…一个没变，按它自己给的出路重锚」	77（探针逐条对上，前置没进来）
47-code-tooling-selftests-and-registries.sh	research-script-selftests	代码类（文件头 :5）	runner 表 45 条自证都通过；research/scripts/ 里文本含 `--selftest` 的文件要么有门禁阶段在调它、要么登记进 NOT_RUN_HERE（本文件:29；:163–182，:183；runner :190–217，覆盖段 :230–286）	runner 表点名的 45 条命令（本文件:190–217）；覆盖段 glob research/scripts/* 与 .claude/gate.d/[0-9][0-9]-*.sh 的非注释代码（:241、:244）	>5 分钟（300 秒 timeout 停掉）	同一件事分两处：15 号另跑 5 份研究脚本自证（15 号:85–139）；共享「工具层的闸」hooks-registered.sh 跑的是钩子的自证（hooks-registered.sh:12–14），对象不重叠	.claude/singlefs-ai-sop/rules/sop-first.md:11「门禁脚本自己也要有测试」；规则在	抓到过：records/2026-09-27-验证两档拆分.md:41「47 号里 check-segment-registry.py --selftest 报第二条流段序列与用例对不上」；records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:60 同一处；research/prompts/gate-fix-forks-r3-opus-output.md:72 记它放过十个新变异（漏判）	不跑完（>5 分钟，300 秒 timeout 停掉；HEAD 版原 47 号在同一份仓上跑完 344 秒）；覆盖段单独现算（与原 47 号逐字相同的代码）：红，4 份「声称有 --selftest 却没阶段在跑」
47-code-tooling-selftests-and-registries.sh	rules-manifest	代码类（文件头 :5）	.claude/rules/ 下的文件集合与 CLAUDE.md 里 `@.claude/rules/` 的引用集合逐项相等，代码围栏与行内代码里的 @ 不算（本文件:30；:287–298，:299）	.claude/rules/（本文件:306）、CLAUDE.md（:307）	0.1 秒（退 0）	判据同、对象不重叠：共享 doc-lint K 判 CLAUDE.md↔上游副本 rules/，并写明项目本地 .claude/rules/ 由项目自己的阶段管（doc-lint.sh:960–970）	C4 已还清（.claude/kb/checks-owed.md:574）；规则在	没找到记录	绿（9 条，与 CLAUDE.md 的引用逐项相符）
47-code-tooling-selftests-and-registries.sh	stage-owners	代码类（文件头 :5）	.claude/gate.d 顶层每个 *.sh 在 stage-owners.tsv 恰好一行；表里阶段存在、owner 有定义、三列齐；每个阶段认第一个参数当项目根（本文件:31；:350–368，:369）	.claude/gate.d/stage-owners.tsv（本文件:374）、.claude/gate.d/*.sh（:381）、.claude/agents/（:393）	0.1 秒（退 1）	无	.claude/agent-common.md:86「阶段归属表 .claude/gate.d/stage-owners.tsv 第二列…」；规则在	抓到过：records/2026-09-16-subagent拆分提案.md:314「62 号在 crash-verifier 定义还没建时实跑判红，报的正是那四行」	红：10-references-and-invariants.sh、11-review-and-sync-records.sh、12-doc-forbidden-notations-and-old-terms.sh 等合并出来的阶段在表里没登记（表还按旧文件名，归「登记收尾」一件）
47-code-tooling-selftests-and-registries.sh	agent-write-scope	代码类（文件头 :5）	write-guard、bash-command-detector、runner-dispatch-guard、continuation-guard、session-start、heavy-test-guard、handback-guard、ask-user-claim-guard 与上游 pattern-process-guard 注册在要的 matcher 上、--selftest 通过；写范围表与定义双向一致；定义带 omitClaudeMd 与「开工先读：」、model 取值认得；两份共用模块的函数不另写一份（本文件:32；:442–478，:479）	.claude/settings.json（本文件:486）、.claude/hooks/*（:684–711）、.claude/hooks/agent-write-scope.tsv（:600）、.claude/agents/*.md（:617）	4.7 秒（退 0）	部分重复：共享「工具层的闸」hooks-registered.sh 对 .claude/hooks/ 与上游 claude-hooks/ 每个钩子判注册着、带 --selftest 的自检通过（hooks-registered.sh:12–14），这一格 ①②⑤⑥⑦⑨⑩⑫⑭⑯ 的「注册着、自证过」是同一件事，整轮门禁里这些钩子自证各跑两遍；matcher 那一半，bash-command-detector、continuation-guard、handback-guard、runner-dispatch-guard 写了 `# hook-events: PreToolUse:<工具>`，hooks-registered ③ 也判（hooks-registered.sh:15–17）；write-guard 没写 hook-events、heavy-test-guard 与 ask-user-claim-guard 只写 PreToolUse、session-start 只写 SessionStart，它们的 matcher 只有这一格判。写范围表、定义头、共用模块三样不重复	.claude/agent-common.md:37（写范围闸）、:63–81（执行前拒绝的写法）；规则在	没找到记录（records/2026-09-16-subagent拆分提案.md:332 是写钩子时撞的静默放行、立它的起因）	绿
47-code-tooling-selftests-and-registries.sh	change-range-single-source	代码类（文件头 :5）	.claude/gate.d/ 顶层 *.sh、*.py 不自己算 diff 基准；列路径的 git 调用带 -c core.quotepath=false；调 gate_changed_paths / gate_added_lines 要判退出码（本文件:33；:818–847，:848）	.claude/gate.d/*.sh 与 *.py（本文件:946）；research/scripts、.claude/scripts、.claude/hooks 下碰到的只报不判（:981）	0.2 秒（退 0）	无	C449 仍欠（.claude/kb/checks-owed.md:375）；C523 仍欠（:419，漏判与误拦）；依据三方判决 research/prompts/gate-fix-forks-r1-main-verification.md 的 T5（本文件:840）	没找到记录	绿（查了 28 份）
47-code-tooling-selftests-and-registries.sh	research-gate-lint	代码类（文件头 :5）	research/scripts/、.claude/hooks/、.claude/scripts/ 交给上游 gate-lint 与 shell-lint；前两个目录的 .sh 执行位；连同 .claude/gate.d/ 终止进程只许点名一个（经 bash-command-detector.sh --scan-scripts）（本文件:34；:995–1013，:1014）	research/scripts/、.claude/hooks/、.claude/scripts/（本文件:1028）、.claude/gate.d/ 与 .claude/process-safety-pending（:1061）	4.4 秒（退 1）	同工具、对象不重叠：共享「门禁自检」「shell 纪律」只给 .claude/gate.d/（gate.sh:381–395），这一格接的是它们够不着的目录（show-me-test.md:99 要求项目自己接）；执行位与共享「脚本执行位」（gate.sh:398–401：SOP 脚本、.claude/gate.d、.claude/scripts）不重叠；终止进程扫描在 .claude/gate.d/ 上与共享 shell-lint S2（pkill -f / killall，shell-lint.sh:22）部分重叠	.claude/singlefs-ai-sop/rules/show-me-test.md:99；C382、C384 已还清（.claude/kb/checks-owed.md:553、:551）	立它时扫出：.claude/kb/checks-owed.md:553 C382「2026-09-18 单跑整仓 gate-lint 红 90 处，84 处在这两个目录」；之后判红的真问题记录没找到	红：research/scripts/multi-host-run.sh 在暂存区里是 100644，.sh 要可执行（gate-lint 那一半绿：103 个脚本、342 条拒绝）
47-code-tooling-selftests-and-registries.sh	fixture-claims	代码类（文件头 :5）	① 阶段头部写了 fixtures/<自己> 的样本目录要在；② fixtures/ 下没有孤儿目录；③ 样本目录至少有 red 或 green、每个子目录有 expect；④ 全仓文件名与 fixtures 下 .md 正文里的日期不许落在不可能的区间（本文件:35；:1091–1140，:1141）	.claude/gate.d/*.sh 头部与 .claude/gate.d/fixtures/（本文件:1147、:1242）、全仓文件名（git ls-files）、.claude/singlefs-ai-sop/scripts/lib.sh 的 date_out_of_range（:1322）	2.6 秒（退 0）	部分重叠：④ 与共享 doc-lint M（doc-lint.sh:632–647，`### 日期` 与「实测（日期）」）用同一个 date_out_of_range、对象不同（本文件:1106–1109）；①–③ 与共享「本地阶段判别力」stage-selftest.sh 相邻：那一道跑样本判得对不对，没有样本目录的只列成未自检（stage-selftest.sh:50–52）	.claude/singlefs-ai-sop/rules/show-me-test.md:59；C510 仍欠（.claude/kb/checks-owed.md:414）；C46 仍欠（:47）	抓到过：.claude/kb/checks-owed.md:651「新立门禁 95 号…立起来当场抓到 91 号」；research/prompts/c510-date-gate-r1-verifier-output.md:69	绿（23 个阶段里 22 个声称有样本，都在）
47-code-tooling-selftests-and-registries.sh	kb-registry	代码类（文件头 :5）	.claude/kb/ 根下每份 .md 与每个子目录都在 CLAUDE.md「## 项目本地事实」表一行的第一格按路径登记；表里点到的 .claude/kb/… 路径都存在（本文件:36；:1361–1385，:1386）	.claude/kb/（本文件:1393）、CLAUDE.md（:1394）	0.1 秒（退 0）	无	CLAUDE.md「## 项目本地事实」表（CLAUDE.md:68 起）；规则在	立它当天实测漏登记 4 份（本文件:1372–1374）；之后判红的记录没找到	绿（19 项：15 份 .md、4 个子目录）
54-layer0-replay.sh	整道	checker-tier 类（文件头 :5）	快档：checker 档包不标 ignored 的用例在 release 下跑（`cargo test --release -p singlefs-checker-tier --lib --tests`，本文件:50），再逐条核 stage-inputs.tsv 的 crash-case: 用例全绿标记，不作数的报本次未跑；开跑前经 crash-case-check.py 判 placement 与 modules（:82、:129–130）；--full 由用户要求或夜间（:4、:14–19）	.claude/gate.d/stage-inputs.tsv:28（crates/ Cargo.toml Cargo.lock research/scripts/admission.py layer0-shard-run.sh layer0-shard-configuration-check.sh）与 crash-case: 行（:36 起）；git common-dir 的全绿标记（本文件:21）	重型不跑	部分重叠：共享「构建与单测」check.sh 跑 `cargo test --all`（check.sh:97），checker 档包不标 ignored 的用例在 debug 下已跑一遍，快档在 release 下再跑同一批	.claude/rules/verification.md:51；.claude/rules/implementation-workflow.md:77；规则在	抓到过：records/2026-09-27-验证两档拆分.md:41「54 号快档真跑红…三条 FAILED」，归因 C577 加屏障之后钉值没改（同一行第三格，写明是推的）	不跑（重型）
55-qemu-device-streams.sh	整道	checker-tier 类（推：records/2026-09-28-门禁59号提速与双机分片.md:68；文件头没写 gate-category）	两块 virtio 盘上六档虚机并行跑：direct 档设备侧日志逐项等于程序录制流、FLUSH 数与屏障对得上、段序列与 E142 产物逐字相同；漏屏障与走页缓存两个对照必须判红；后三档冷重开读回（本文件:4、:11–24）	.claude/gate.d/stage-inputs.tsv:29（crates/ Cargo.toml Cargo.lock research/scripts/ research/results/；qemu-system-x86_64、/dev/kvm、research/scripts/vm-kernel.sh --check）	重型不跑	部分重叠（数据源）：32 号 segment-registry 也拿 E142 产物比段序列（32 号:27），那一格比登记表、这一道比虚机实跑	.claude/rules/implementation-workflow.md:64；C6 仍欠（.claude/kb/checks-owed.md:20）；C487 仍欠（:403）；后三档依附里程碑二步 1、步 3 与增补 2 收口表第 58 行（本文件:16–19）	抓到过：records/2026-09-24-里程碑二收尾调度.md:241「55 号…重跑真虚机查出 new_pool_file_creation_on_device.rs:1475 与 publish_order_matches_litmus.rs:47 写死了 2026-09-14 那时的事务步数 23（今天 33）」	不跑（重型）
57-lkmm.sh	整道	checker-tier 类（推：records/2026-09-28-门禁59号提速与双机分片.md:68；文件头没写 gate-category）	litmus/ 下每条 Never 有对照组、绑到代码、herd7 判定与声明相符（本文件:7；逻辑在 .claude/scripts/lkmm.sh）	.claude/gate.d/stage-inputs.tsv:30（litmus/ .claude/scripts/lkmm.sh fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/；herd7 版本）与内核树 tools/memory-model（本文件:20–22）	重型不跑	无	.claude/rules/implementation-workflow.md:63；规则在	没找到记录（records/2026-09-16-subagent拆分提案.md:382、:744 两次都是绿）	不跑（重型）
59-crates-mutation-replay.sh	整道	checker-tier 类（推：records/2026-09-28-门禁59号提速与双机分片.md:68；文件头没写 gate-category）	crates/mutations.tsv 每条改坏一处、点名的测试必须红；派活前判成形、锚点、测试名、转义、拷贝范围与基线（点名测试在没改坏的副本上 ok）（本文件:6–12）	.claude/gate.d/stage-inputs.tsv:31（crates/ Cargo.toml Cargo.lock litmus/ research/scripts/run-with-memory-cap.sh research/scripts/crates-mutation-rows.py）与 crates/mutations.tsv	重型不跑	重复：派活前的静态几样与 33 号 mutation-tables 是同一份 row_problems（本文件:8–10、:14）	.claude/singlefs-ai-sop/rules/show-me-test.md:26；.claude/rules/implementation-workflow.md:75；.claude/rules/mutation-sampling.md:74、:82；用户 2026-09-28 定整个重设计（records/2026-09-28-门禁59号提速与双机分片.md:96）	抓到过：records/2026-09-16-subagent拆分提案.md:748「59 号 6 行锚点腐化，整张表一条没跑」（修锚点后 123 条全红）；records/2026-09-28-门禁59号提速与双机分片.md:11 Q1 第 1377 行改的是 litmus/、59 号在派活前判红	不跑（重型）
74-model-differential.sh	整道	harness 类（推：records/2026-09-28-门禁59号提速与双机分片.md:67「模型对拍（74）」；文件头没写 gate-category）	release 下单跑 crates/singlefs-harness 的 random_histories，SECTIONS 每一段都打出「模型对拍」一行、步数大于 0（本文件:9；SECTIONS :38；cargo :48）	.claude/gate.d/stage-inputs.tsv:32（crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh research/scripts/capped.sh），crates/singlefs-harness/tests/random_histories.rs（本文件:7）	要编译，不跑	部分重叠：共享 check.sh 的 `cargo test --all`（check.sh:97）在 debug 下已跑 random_histories 不标 ignored 的用例；这一道在 release 下再跑一遍并多判「模型对拍」行	.claude/rules/verification.md:55；依附里程碑二增补 3 第 2 件（本文件:4）；规则在	抓到过：records/2026-09-24-里程碑二收尾调度.md:227「调查员交回：门禁 74 号两条是 C554 同机理的真违例」	不跑（要编译）
77-test-environment.sh	整道	checker-tier 类（推：records/2026-09-28-门禁59号提速与双机分片.md:68；文件头没写 gate-category）	research/scripts/test-environment-check.py check 的 10 类：残留 loop/dm 设备、残留挂载、带 singlefs 的 qemu 进程、/tmp 残留、宿主盘只读、剩余空间、ext4 errors_count、内核日志块设备错、SMART（要 --smart）；判红或没查成都红（本文件:6–12）	research/scripts/test-environment-check.py（本文件:34）；宿主的 /tmp、挂载表、/proc、内核日志；分界取祖先 gate.sh 的启动时刻（:38–41）	1.6 秒（退 1）	部分重叠：共享 gate.sh「跑完没留下临时文件」判这一轮门禁自己的 TMPDIR 里剩什么（gate.sh:184；command-safety.md:143–147），这一道按名字扫整个 /tmp 与设备、挂载、盘	C396 已还清（.claude/kb/checks-owed.md:552，用户 2026-09-19 定）；.claude/singlefs-ai-sop/rules/command-safety.md:143；C474 仍欠（:396）	接上当天现查 /tmp 里 34 个镜像（本文件:18–19）；之后判红的多是门禁自己留下的（C448 .claude/kb/checks-owed.md:545、C474 :396），真问题记录没找到	红：临时目录残留，/tmp 下 188 项、实占 63.7 GiB（共查 206 项 singlefs-*，这一次跑自己产生的 0 项）
87-replay.sh	整道	checker-tier 类（推：records/2026-09-28-门禁59号提速与双机分片.md:68；文件头没写 gate-category）	research/scripts/replay.sh 登记表里判定为 exact 的行（减 SLOW 清单）重跑，与 research/results/ 的产物逐字节比；没跑的逐个列名（本文件:6–12；SLOW :44）	.claude/gate.d/stage-inputs.tsv:33（crates/ Cargo.toml Cargo.lock research/e7-index-bench/ research/scripts/ research/results/），research/scripts/replay.sh 登记表	重型不跑	无（34 号只读文本、不跑装置，34 号:6 gate-similar）	evidence-discipline.md「重跑之后要回对正文」；规则在	抓到过：本文件:18–21 记 2026-09-16 GATE_REPLAY_FULL=1 补跑查出 E58 产物早已对不上、E17 在 release 下 panic	不跑（重型）
92-layout-checker-sync.sh	整道	checker 类（推：records/2026-09-28-门禁59号提速与双机分片.md:69；文件头没写 gate-category）	layouts.tsv 每行四列齐、路径都在、incompat 位号在 D15 已定项 4 登记表且名字逐字相同；这次改动让某套布局的「常量名 → 值」集合变了而 checker 判定路径没碰就判红（本文件:6–17）	.claude/gate.d/layouts.tsv、layouts-checker-lag.tsv、.claude/kb/decisions/15-格式冻结政策.md、格式定义路径（.rs 顶格 const 与 .md 标记，经 lib-format-const.py）、改动范围 changed-paths.sh、checks-owed.md 经 lib-owed.py（本文件:29–41）	0.4 秒（退 0）	部分重叠：与 27 号 format-constants 共用 lib-format-const.py、判的事不同（27 号:8 gate-similar）；③ 与 27 号 feature-bits 同读 D15 已定项 4 登记表（本文件:10–11）	C14 已还清（.claude/kb/checks-owed.md:546）；规则在	没找到真缺陷记录；research/prompts/defs-gate92-first-registration-report.md:85「真仓上从红转绿」是修首次登记误判	绿（1 套布局、6 条路径；这次改动比了 111 个格式常量）
94-checker-implementation-disjoint.sh	整道	checker 类（推：records/2026-09-28-门禁59号提速与双机分片.md:69；文件头没写 gate-category）	① checker 与 core 的传递闭包交集减去 singlefs-format 为空；② checker 源码零处引 singlefs_core；③ singlefs-format 正文（除 #[cfg(test)]）不许有分支循环；④ harness 闭包不含 checker-tier、源码零处引它（本文件:6–22）	crates/*/Cargo.toml 与根 Cargo.toml 的 [workspace.dependencies]、crates/singlefs-checker/src、crates/singlefs-format/src、crates/singlefs-harness（本文件:24–29）	0.1 秒（退 0）	无	.claude/rules/verification.md:17、:45；D13（验证路线） 已定项 5、15；C12 已还清（.claude/kb/checks-owed.md:548）	没找到记录（本文件:34–36 是立它时「今天这三条在仓里都成立」）	绿（checker 闭包 1 个、实现闭包 2 个、内部依赖图 5 个 crate）
```

## 附：`.claude/gate.d/*.sh` 逐份清单（文件名、行数、`# gate-stage:` 行所在行号、格名表所在行区间）

门禁脚本本身不整份进这份附录（23 份，一万五千多行）；这张表只列元数据，各腿要看哪一格自己去读原文件。

取法：文件名与行数用 `wc -l`；`# gate-stage:` 行号用 `grep -n '^# gate-stage:'`；「格名表所在行区间」按文件里是否出现字面 `--check <格名>` 这个菜单来判——出现了，就从第一处 `#   <格名>  <说明>`（或「格名 ← 合并前的阶段」箭头式）那一行的上一行（表头）起，收到第一处空 `#` 行或非 `#` 行为止，逐份手工核过；没出现这个菜单的（15、54、55、57、59、74、77、87、92、94 十份）判「无格名表（单格文件）」，`gate-stage:` 那一行本身就是判据，没有内部 `--check` 子菜单。**这批行区间有一处已知的不精确**：11、30、13 等几份的续写行收尾时会带上「汇总：任一格判红……」这类紧跟在格名列表之后的说明句一起进区间（因为它们与格名行一样都是不换行的 `#` 注释、脚本按「遇到空 `#` 行才停」来切），行区间因此比纯格名列表略宽一两行；格名本身的计数（表里「+ N 格」那个数）与文件头 `# gate-stage:` 行报的「N格」逐份核对过、一致。

| 文件名 | 行数 | `# gate-stage:` 行号 | 格名表所在行区间 |
|---|---|---|---|
| 10-references-and-invariants.sh | 537 | 4 | 7-15（表头 7 行起 + 7 格） |
| 11-review-and-sync-records.sh | 1291 | 4 | 7-13（7 格，箭头式：「格名 ← 合并前的阶段」） |
| 12-doc-forbidden-notations-and-old-terms.sh | 265 | 4 | 9-15（表头 9 行起 + 3 格） |
| 13-code-vague-names-and-test-file-names.sh | 213 | 4 | 11-15（表头 11 行起 + 2 格） |
| 14-harness-one-scenario-per-test.sh | 75 | 4 | 12-14（表头 12 行起 + 1 格） |
| 15-research-build.sh | 151 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 20-doc-decision-documents.sh | 2257 | 4 | 15-29（表头 15 行起 + 13 格，三列式表头「格名（--check 用） 格 判什么」） |
| 27-code-constants-enums-bits-match-kb.sh | 816 | 4 | 18-21（表头 18 行起 + 3 格） |
| 30-decision-history-entries.sh | 567 | 4 | 10-15（表头 10 行起 + 3 格） |
| 32-doc-field-and-layout-registry.sh | 1416 | 4 | 21-30（表头 21 行起 + 9 格） |
| 33-code-experiment-and-mutation-source-discipline.sh | 975 | 4 | 19-22（表头 19 行起 + 3 格） |
| 34-doc-experiment-pages-and-products.sh | 2084 | 4 | 21-31（表头 21 行起 + 10 格） |
| 43-checks-owed-and-closeout.sh | 916 | 4 | 19-24（表头 19 行起 + 5 格） |
| 47-code-tooling-selftests-and-registries.sh | 1510 | 4 | 28-36（表头 28 行起 + 8 格） |
| 54-layer0-replay.sh | 646 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单；支持 `--full` 开关，不是格名子菜单） |
| 55-qemu-device-streams.sh | 343 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 57-lkmm.sh | 62 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 59-crates-mutation-replay.sh | 170 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 74-model-differential.sh | 117 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 77-test-environment.sh | 121 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 87-replay.sh | 77 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 92-layout-checker-sync.sh | 360 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |
| 94-checker-implementation-disjoint.sh | 307 | 4 | 无格名表（单格文件，没有 `--check <格名>` 菜单） |

合计 23 份，`wc -l .claude/gate.d/*.sh` 总行数：
  15276 total
