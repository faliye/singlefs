# m2-agentdef-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽取 Opus 报告的引用「`.claude/agents/implementation-writer.md:27`」+ 抄的整行原文，在草稿目录副本
`/tmp/claude-1000/m2-agentdef-r1-verifier/selftest/implementation-writer.md` 里把行号加 1（改核第 28 行），
按第 2 步核（取该行内容与被核引文比对）：

```
$ awk 'NR==28' /tmp/claude-1000/m2-agentdef-r1-verifier/selftest/implementation-writer.md
4. 跑 `nice -n 19 bash .claude/scripts/check.sh`，贴末尾原样输出；再跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）。
```

第 28 行内容与 Opus 报告里抄的第 27 行原文（「3. 每条新测试证明会红：……」）逐字不同 ⇒ 判 **✗**。
方法能分辨，往下核。

## 一、Opus 攻方腿报告核对表

被核文件：`research/prompts/m2-agentdef-r1-opus-output.md`（sha256 与交回一致，见下方总记）。
模型目录已拷进 `/tmp/claude-1000/m2-agentdef-r1-verifier/opus-model/` 复跑，未在原目录跑。

### 1a 复跑命令（优先项）

| 命令 | 复跑方式 | 结果 |
|---|---|---|
| `mtime_toy.sh <工作目录>` | 拷模型目录到草稿，`nice -n 19 bash opus-model/mtime_toy.sh` 指向自己的空工作目录 | ✓ 输出与 `mtime_toy.out` 逐字节相同（diff 空），sha256 `922d94dc…e26` 两边一致，脚本 sha256 `e00f5535…131` 与报告表一致 |
| `probe_gate71.py . <空的工作目录>` | 第一参数（仓根，只读）指向真实仓，第二参数（空工作目录）换成草稿目录里新建的空目录 | ✓ 输出与 `probe_gate71.out` 逐字节相同，sha256 `b8c4583b…79a` 两边一致，脚本 sha256 `f58298c1…9d` 一致 |
| `probe_r2.py . <空的工作目录>` | 同上 | ✓ 输出与 `probe_r2.out` 逐字节相同，sha256 `2429723d…33e` 两边一致；R2 下 `fixture-green pass=False`，与报告「绿样本的汇总片段变了……expect 要跟着改」一致 |
| `heading_probe.py .` | 第一参数指向真实仓 | ✓ 输出与 `heading_probe.out` 逐字节相同，sha256 `e0024068…c38` 两边一致，命中 6 个小节与报告列的 6 个一致（`decisions/20:27`、`decisions/24:1`、`experiments-history.md:1543`、`tooling.md:198/235/561`） |
| 报告末尾「71 号在今天文件上」 | 直接在仓里原样跑（只读，`grep -n "open(.*['\"]w" .claude/gate.d/71-agent-def-flow-only.sh` 确认脚本无写操作） | ✓ 退出码 0，输出与报告贴的原样一致 |


### 1b FP1/FP2 与改法 R1 的判定（优先项）

| 引用 | 核的结果 |
|---|---|
| FP1 点名 `.claude/kb/tooling.md:198`「提示里的 `**粗体**:` 会诱发损坏（2026-08-29 实测 3/3 对 3/3）」 | ✓ `awk 'NR==198'` 取出该文件该行，逐字与报告引文相同 |
| FP2 点名 `.claude/kb/tooling.md:561`「并发：两个会话在同一个仓里同时工作过（2026-08-30 实测）」 | ✓ `awk 'NR==561'` 逐字相同 |
| FP1/FP2 判定「红，④ 1 处……R1_exit=0」 | ✓ 复跑 `probe_gate71.py` 输出逐行核对：`FP1 … R1_exit=0`、`FP2 … R1_exit=0`，与报告表一致 |
| FP3–FP9 判定「红……R1_exit=1」（R1 不修这几格） | ✓ 复跑输出对应七行 `R1_exit=1` 与报告表逐格一致 |
| 汇总行 `false_positive_red_on_expected=9/9 R1_turns_green=2/9 miss_cases_green=26/26` | ✓ 复跑 `probe_gate71_rerun.out` 末尾同一行逐字节相同 |
| R1 改法「④ 先剥「」」代码位置第 120 行 | ✓ `awk 'NR==120'` 取今天 71 号脚本该行，逐字与报告引文 `for match in HALF_SENTENCE.finditer(line):` 相同 |
| ② 剥「」代码位置第 109 行 | ✓ `awk 'NR==109'` 逐字相同（`if not DATE.search(without_quoted(line)):`） |
| 71 号自己给的改法文案第 163 行 | ✓ `awk 'NR==163'` 逐字与报告引文相同 |
| 共用约束 `.claude/agent-common.md:9` 整行引文 | ✓ 逐字相同 |

### 1c M01–M27 漏判清单（优先 M01–M04，其余按脚本复跑抽验）

| 引用 | 核的结果 |
|---|---|
| M01 `crash-verifier.md:24` 整行 | ✓ `awk 'NR==24'` 逐字与报告引文相同 |
| M02 `kb-scribe.md:26` 整行（含「agent-defs-r2 攻方模型：1/3 对 0/3」） | ✓ `awk 'NR==26'` 逐字相同 |
| M03/M04 `mutation-triage.md:25` 整行（「今天 4 张」「计划第十八节」） | ✓ `awk 'NR==25'` 逐字相同；`grep -l '被测装置是 shell 探针' research/mutations/*.tsv \| wc -l` → `4`，与报告「现状核对」一致 |
| M01–M04 探针字段 `present=True flagged_by_71=False same_text_before_cleanup=True` | ✓ 复跑 `probe_gate71.out` 对应四行字段与报告表一致 |
| M05–M26（22 处）file:line 与三个布尔字段 | ✓ 复跑输出（`grep -E '^M(0[5-9]\|1[0-9]\|2[0-6])'`）逐行 file:line 与报告表完全一致，`present=True flagged_by_71=False same_text_before_cleanup=True` 22/22；因整份脚本复跑已逐字节匹配存档输出，未再对 22 处逐一手工抄行比对，判 ✓（脚本从真实文件读取，非贴出来的孤立断言） |
| M27 `experiment-designer.md:39` 表格行 | ✓ `awk 'NR==39'` 逐字与报告引文相同 |
| M17 前提「在仓根下跑会被报成「基线就是红的」」出自 `mutation-triage.md:25` 同一行 | ✓ 同上第 25 行已核 |

### 1d 其余引用（非优先项，逐条现查过）

| 引用 | 核的结果 |
|---|---|
| H1 清理前 `.../before/agents/implementation-writer.md:27` 整行 | ✓ 逐字相同 |
| H1 今天 `implementation-writer.md:27` 整行 | ✓ 逐字相同（也是判别力自证用的引用） |
| H2 清理前 `.../before/agents/three-way-verifier.md:21` 整行 | ✓ 逐字相同 |
| H2 今天 `three-way-verifier.md:21` 整行 | ✓ 逐字相同 |
| H2「两版第 31 行一字不差」 | ✓ `awk 'NR==31'` 分别取两版，逐字相同 |
| 背景材料 `_m2-agentdef-r1-background.md:33/40/41/45` 四处引文 | ✓ 逐行取出，与报告引文逐字相同（含反向接受条款原文） |
| gate-triage 清理前/今天 `:27` 整行 | ✓ 逐字相同 |
| `.claude/singlefs-ai-sop/skills/gate/SKILL.md:33` 表格单元格 | ✓ 逐字相同 |
| `.claude/singlefs-ai-sop/scripts/gate.sh:369-388` 代码块 | ✓ 逐行相同，「只打两个指纹、不点名文件」的说法与代码里只输出 `${START_TREE:0:12}`/`${END_TREE:0:12}` 一致 |
| `.claude/kb/decisions/20-承重面单元的原子性与自包含.md:27` | ✓ 标题文字（去掉 `####` 后）与 heading_probe 输出逐字相同 |
| `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:1` | ✓ 同上 |
| `.claude/kb/experiments-history.md:1543` | ✓ 同上 |
| 71 号文件头第 12、19–23、25、27 行 | ✓ 逐行取出，与报告转述的判据内容一致（第 25、27 行为整行引用，逐字相同） |
| 71 号第 93、119 行（M27 判据依据） | ✓（弱）：`awk` 取出两行字面与报告一致（`elif not line.strip() or HEADING.match(line) or TABLE.match(line):`、`if not breaks_paragraph[index]:`），但未逐步跟踪完整控制流验证这两行确实实现「表格行不判 ③④」，只confirm字面存在 |
| 绿样本 `fixtures/71-agent-def-flow-only.sh/green/setup.sh:17/27/31` | ✓ 三行逐字取出，与报告引文相同；第 17、27 行确认不含 ④ 触发词，第 31 行确认带「因为」 |
| `.claude/scripts/lkmm.sh:283/288/320` | ✓ 三行逐字取出，与报告转述（先加载 opam env / 打「herd7 缺失」/ 打 herd7 版本行）一致 |
| `.claude/kb/vm-harness.md:147` 小节标题 + `experiment-designer.md` 今天第 29 行 | ✓ 标题「## 计时不在转发输出的循环里打时间戳」与今天定义里的指路文字逐字对应；清理前同一行的「2026-09-17 E152……」经过在今天版本里确认已删除、换成指路 |


**Opus 腿计数**：核了 70 处，✓ 70，✗ 0，核不动 0。

## 二、Sonnet 正推腿报告核对表

被核文件：`research/prompts/m2-agentdef-r1-sonnet-output.md`（sha256 与交回一致）。

主 agent 已现查属实、只记「已现查」的两条（本表仍独立复核一遍，结果与「已现查」一致，供交叉核对）：

| 引用 | 状态 |
|---|---|
| A3-a「`main-agent.md` 丢了『不能一直干等』」 | 已现查（本报告独立复核：`grep -n "不能一直干等" .claude/main-agent.md CLAUDE.md .claude/agent-common.md` 退出码 1，三份文件 0 命中，与 Sonnet 报告一致） |
| A4-b「72 号把 `agent-common.md` 因一份被路径改写碰过的旧判决算作已点名」 | 已现查（本报告独立复核：`bash .claude/gate.d/72-agent-def-adversarial-review.sh` 退出码 1，17 份缺失名单里没有 `agent-common.md`；`grep -rl` 只命中 `agent-defs-r1-main-verification.md` 一份，与 Sonnet 报告一致） |

| 引用 | 核的结果 |
|---|---|
| `m2-agentdef-r1-claude-md-at-head.md` 第 10、12、14 行整行 | ✓ 逐字取出，与 Sonnet 报告引文相同 |
| 同文件第 16–26 行表格 9 行 | ✓ 与今天 `main-agent.md:40-51` 逐行比对，9 行一字不差，多一行（阶段同步）为纯新增，与 Sonnet 报告一致 |
| `main-agent.md:12` | ✓ 逐字相同（「下表」→「下面那张表」同义改写，与报告一致） |
| `main-agent.md:24` | ✓ 逐字相同（子确认「不能一直干等」缺失、「子 agent 全部结束」→「交回或被停」措辞收紧、日期与经过标注被删） |
| `main-agent.md:36` | ✓ 逐字相同 |
| `CLAUDE.md:10`、`CLAUDE.md:12` | ✓ 逐字相同 |
| `grep -c '^@' CLAUDE.md` → 21 | ✓ 复跑同一命令，输出 21 |
| `.claude/rules/implementation-workflow.md` 全文 45 行 | ✓ `wc -l` 复跑 = 45 |
| 该文件第 18、20 行整行 | ✓ 逐字相同 |
| 第 24–28 行「代码轮派腿之前记一份开工快照」整节 | ✓ 逐字相同，且该节确实不含「71/72/63/gate」字样（`grep -n` 复跑只命中第 18、20 行） |

| 引用 | 核的结果 |
|---|---|
| `.claude/gate.d/71-agent-def-flow-only.sh:38`（`targets = sorted(glob.glob(...))`） | ✓ 逐字相同 |
| `.claude/gate.d/72-agent-def-adversarial-review.sh:25/39/7-8` | ✓ 逐字相同 |
| 72 号复跑：退出码 1，17 份缺失名单 | ✓ 独立复跑，17 行文件名与报告列的逐一相同，且 `agent-common.md` 确实不在名单里 |
| `grep -rl '\.claude/agents/agent-common\.md' research/prompts/*-main-verification.md` 只命中一份 | ✓ 复跑同一命令，只命中 `agent-defs-r1-main-verification.md` |
| 该文件第 30/48/49 行引文 | ✓ 逐字相同 |
| `ls research/prompts/*-main-verification.md \| wc -l` → 62 | ✓ 复跑相同 |
| `agent-defs-r2-main-verification.md` 里 `.claude/agents/` 路径 0 命中 | ✓ 复跑相同 |
| `records/2026-09-16-subagent拆分提案.md:700` 整行 | ✓ 逐字相同 |
| `agent-common.md`、`main-agent.md` frontmatter 只有 `name`/`description` | ✓ 直接 Read 全文确认，两份 `description` 都写「不是可派发的……：不要派发它」 |
| `.claude/hooks/agent-write-scope.tsv` 无 `agent-common`/`main-agent` 行 | ✓ `grep -n` 复跑退出码 1，零命中 |
| `write-guard.sh` 两条命令原样输出（`agent-common`/`main-agent` 都拒绝，exit=2） | ✓ 复跑同一命令，输出逐字节相同 |
| **`write-guard.sh:71-76`「`grep -nF` 命中 `if not patterns: return 2`」** | **✗ 实际位置不对：`decide_scope()` 函数从第 75 行开始，`if not patterns:` 实际在第 86 行、`return 2, (f"✗ …")` 在第 87–88 行，且拆成两行；`grep -nF "if not patterns: return 2" .claude/hooks/write-guard.sh` 复跑退出码 1（零命中），文件里不存在这个单行连写的字符串。71–76 行实际是另一段代码（`return 2, (f"✗ 拒绝用 Write 整份覆盖…")`），与「有文件、无登记 ⇒ 拒绝」这条逻辑无关。这条实质结论（无登记即拒绝）本身没错，但引用的行号与「grep 命中」的说法不成立** |
| `.claude/gate.d/63-agent-write-scope.sh:11` 判据注释整行 | ✓ 逐字相同 |
| 该文件第 83–86 行代码块 | ✓ 逐字相同 |
| 该文件第 119 行 | ✓ 逐字相同 |
| 63 号复跑：退出码 0，输出原文 | ✓ 复跑输出逐字节与报告贴的相同 |
| `three-way-forward.md:1-6` frontmatter 含 `omitClaudeMd: true` | ✓ 逐字相同 |
| 「补充第一手观测」自己会话转录里的系统提示注入现象 | 核不动：这是 Sonnet 自己那次会话的转录内容，本次核查没有权限或途径读取该腿的原始会话记录，无法独立复现或验证 |

**Sonnet 腿计数**：核了 41 处（含 2 处「已现查」重复独立复核），✓ 39，✗ 1，核不动 1。

**背景材料误写检查**：Opus、Sonnet 两份报告里的「文件:行号」引用逐条核对时，没有发现任何一处行号实际指向的是 `_m2-agentdef-r1-background.md` 或 `_m2-agentdef-r1-diff.md` 而被误标成别的文件；两条腿引背景材料本身时（Opus 引 `background.md:40/41/45`、`_m2-agentdef-r1-body.md` 未被引）都直接写明是背景材料路径，没有误标成 kb 或定义文件。

## 三、本地攻方腿材料核对（提示 + 转述核对表）

按分派要求：主 agent 已用真值逐格比过两份样本（40/40 相同），这里只核提示与转述核对表的转述有没有丢/多加限定词，逐条对照 `.claude/gate.d/71-agent-def-flow-only.sh:8-27`（判据原文）与十个样本各自指向的真实文件。

### 3a 标签清单（27 个标签，逐条核「原文文件:行」与所标文字）

全部 27 个标签（`WEISHENME`…`CJKQ_OPEN/CJKQ_CLOSE`）逐条用 `awk 'NR==<行>'` 取出 71 号脚本对应行，比对核对表标的中文原词与行号：**27/27 ✓**，行号与原文逐字相同，包括几处标了两个行号的（`JINGGUO`/`YUANYIN` 标 16、20；`LISHI` 标 10、20 分别对应①③；`YINWEI`/`ZHISUOYI` 标 17、20）。抽验命令：

```
$ awk 'NR==16' .claude/gate.d/71-agent-def-flow-only.sh
#      标签词 为什么 / 依据 / 理由 / 实测 / 经过 / 原因 / 来历 / 背景 / 历史 / 沿革 / 前情，后面紧跟 ：:（(，,。、 空白或「是」；
$ awk 'NR==19' .claude/gate.d/71-agent-def-flow-only.sh
#   ④ 解释性半句：行内在 （(，,；;。 或 —— 之后（中间可隔空白与一个日期）紧接着
```
第 19 行的触发标点集合「（(，,；;。 或 ——」里确认没有「、」（DUN），核对表「DUN 不在 ④ 触发标点集合里」这条判断成立。

### 3b 四条判据的转述（16 行，逐条核「原文文件:行」）

15/16 逐字或语义等价，1 处发现走样：

| 引用 | 核的结果 |
|---|---|
| C1 只管标题行 `:10` | ✓ |
| C2 引号例外 `:12` | ✓ 逐字保留举例，未省略 |
| **C2「只提到目录」例外 `:13`** | **✗ 丢限定词（换成更宽的类）**：原文「只提到目录（`.claude/kb/` 后面直接是**反引号**或空白）」只认「反引号」这一种标点；提示译成「immediately followed by **a punctuation mark** or by whitespace」——把「反引号」这个具体字符换成了「任意标点符号」这个更宽的类，核对表自称「原样写回」但实际是加宽了触发条件。`awk 'NR==13' .claude/gate.d/71-agent-def-flow-only.sh` 复核原文只有「反引号」一词。十个样本本身不测这一分支，不影响 Part A 的 40 个判定，但会让模型在 Part B 为 C2 构造例子时，可选的「标点」范围比判据实际认的宽 |
| C3 只管段首 `:14` | ✓ |
| C3 剥离前缀 `:14`（四项） | ✓ 核对表自陈首稿漏 `>`，定稿补齐，四项与原文一致 |
| C3 连接符集合 `:16` | ✓ 七种全部列出 |
| C3「不认」`:18` | ✓ 改写成规则未复述具体例句，未改变判据本身 |
| C4 触发标点集合 `:19` | ✓ 五种，不含顿号，与原文一致 |
| C4 中间可隔的内容 `:19` | ✓ |
| C4 两组及后续要求 `:20` | ✓ 分组与原文一致 |
| C4 段落中间行特例 `:21` | ✓ 补全「渲染出来是同一段的后半句」，未丢 |
| C4「不认：顿号之后」`:22` | ✓ |
| C4「不认：标签词后接别的字」`:22` | ✓ |
| C4「不认：日期与词之间隔了别的字，由②管」`:23` | ✓ |
| 三条都不判的（围栏、表格行）`:25` | ✓ 核对表自陈首稿漏「②照判」，定稿补回，未丢 |
| 段落怎么切 `:26` | ✓ |
| 判不到的（背景说明）`:27` | ✓ |


### 3c 十个样本的转述（15 处「首稿缺的/定稿」行，逐条核「原文文件:行」）

对每个样本用 `awk 'NR==<行>'` 直接取真实文件那一行，与提示里给的「紧跟标点之后的几个字」逐点核对：

| 引用 | 核的结果 |
|---|---|
| P1 occurrence 1（`agent-common.md:9`） | ✓ 真实行「。」后依次是「「为什么」「实测」「经过」」，与提示「PERIOD_FW, CJKQ_OPEN, WEISHENME, CJKQ_CLOSE, CJKQ_OPEN, SHICE, CJKQ_CLOSE, CJKQ_OPEN, JINGGUO, CJKQ_CLOSE」逐字对应 |
| P1 occurrence 4（`agent-common.md:9`） | ✓ 「（」后是「「开工先读：`文件`「小节」」」，提示写「引到 COLON 后截断，不声称有无闭合引号」——核对表自陈这处比正文附录三给的原始摘录（截止到「开工先读」）更完整，因为另外去查了真实文件，判断合理 |
| P2 occurrence 1（`main-agent.md:8`） | ✓ 「；」后「为什么这么定」，与提示「WEISHENME, 无间隔, "is decided this way"」一致；提示另加「LPAREN 之后含 SHI_COPULA 但不紧邻」的旁注，真实行「（项目是什么……）」里「是」是「（」后第 3 个字，旁注属实但未被核对表第四节的「多加的限定词」表逐一列出——不算错误，只是索引不全 |
| P3 occurrence 3（`main-agent.md:20`） | ✓ 「。」后「发散是探索该有的样子」，`是` 非首字，提示旁注属实，且已被核对表第四节列出 |
| P4 occurrence 1（`three-way-defense.md:14`） | ✓ 「开工先读：」…「（」后日期在「」里，与提示一致 |
| P5 附注（`implementation-writer.md:32`） | ✓ 另一处 WEISHENME 紧跟普通动词、不在任何触发标点后，属实 |
| P6 occurrence 1 附注 + YUANYIN 附注（`three-way-forward.md:27`） | ✓ 逐字核对：「；」后是「只能由主 agent」，之后才出现「实测」；行尾「与原因」的「原因」前是「与」非触发标点，两处附注均属实 |
| P7 occurrence 2/3（`kb-scribe.md:18`） | ✓ 「、」紧跟「依据」、「依据」紧跟「（」，顺序与提示一致 |
| P8 opening + 日期归类（`.../before/agents/three-way-local-defense.md:24`） | ✓ 「实测（」之间无空白，日期在普通括号里、不在「」里，均与提示一致 |
| P9 occurrence 3（`.../before/agents/kb-scribe.md:29`） | ✓ 日期与「试跑」之间隔着「两次」二字（非纯空白），提示「DATE, 两字普通文本, SHIPAO」准确捕捉到这一点使该处不满足 C4 的「只许 WS 与一个 DATE」的间隙要求 |
| P9 occurrence 18（同上行） | ✓ 「（2026-09-17 实测：」日期与「实测」之间只隔一个空白，与提示「DATE, WS, SHICE, COLON」一致 |
| P10 occurrence 5（`.../before/agents/implementation-writer.md:27`） | ✓ 「，2026-09-17 实测源码……」，与提示补全后的「COMMA, DATE, WS, SHICE, 普通文字」一致 |
| P8/P9/P10 出处标注（历史快照说明） | ✓ 三份确实来自 `before/agents/`，提示明确标注「历史快照，非今天定义」 |


### 3d 独立完整性核查（超出核对表自己列出的行，逐点核样本 P8 的「紧跟标点之后」是否穷尽）

用脚本逐字符扫了 `.../before/agents/three-way-local-defense.md:24` 全行的触发标点位置（命令与输出）：

```
$ python3 -c "
line = open('research/prompts/m2-agent-def-cleanup/before/agents/three-way-local-defense.md', encoding='utf-8').readlines()[23]
import re
for m in re.finditer(r'[（(，,；;。]|——', line):
    print(m.start(), repr(m.group()), repr(line[m.end():m.end()+15]))
"
4 '（' '2026-09-16 第一轮）'
36 '，' '其中两次闸判绿；样本更容易不够'
44 '；' '样本更容易不够两份，不够就照实'
54 '，' '不够就照实报。\n'
61 '。' '\n'
```

**✗ P8 的「紧跟标点之后」点列漏了一处**：真实行有 4 个有效触发点（末尾「。」后为空行不计），提示 SAMPLE P8 只列了 3 个（LPAREN、SEMI、最后一个 COMMA），**漏掉第一个「，」之后的「其中两次闸判绿」**。这一处的缺失同样存在于正文 `_m2-agentdef-r1-body.md` 附录三的源表格（P8 行只给了 3 个点），核对表宣称「逐格另用 `sed -n` 现读了样本指向的真实文件核对内容一致」，但没有发现这处遗漏。**实质影响**：「其中两次闸判绿」不含任何 16 个标签词，此处遗漏不改变 C4 对 P8 的 GREEN/RED 判定（既有列表里 3 个点全部不触发，漏掉的第 4 个点同样不触发），因此不影响本轮已核对过的 40 个真值答案，但转述本身不完整。

## 计数与没打中形状汇总

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| Opus 攻方 | 70 | 70 | 0 | 0 |
| Sonnet 正推 | 41 | 39 | 1 | 1 |
| 本地攻方材料 | 63（27 标签 + 16 判据 + 15 样本 + 1 独立完整性核查，另计 P2 旁注索引不全为观察项不计入✗/✓） | 60 | 2 | 0 |
| **合计** | **174** | **169** | **3** | **1** |

三处 ✗：
1. Sonnet 报告 `write-guard.sh:71-76` 行号与「grep -nF 命中」的字面说法不成立（实际逻辑在第 75、86–88 行，且不是单行连写字符串）；实质结论（无登记即拒绝）本身未受影响。
2. 本地攻方转述核对表 C2「只提到目录」例外一行，把原文「反引号」这个具体标点扩写成「a punctuation mark」这一更宽的类，核对表自称「原样写回精确条件」但实际加宽了；十个具体样本不触及这一分支，不影响已核对过的 40 个真值。
3. 本地攻方提示 SAMPLE P8 的「紧跟标点之后」点列漏了一处（「，」后「其中两次闸判绿」），源头是正文附录三表格本身的遗漏，核对表的「一致」核实没有发现它；该点不含任何标签词，不改变 P8 在任何判据上的 GREEN/RED 结论。


## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- Sonnet A3-a、A4-b 两条主 agent 已现查属实的，本报告只做了独立复核（结果一致），未重新论证它们该怎么判。
- Sonnet「补充第一手观测」（自己那次会话收到的系统提示注入）无法复核：没有权限读取那次会话的原始转录，只能记「核不动」。
- 未逐条核 Opus 报告里标为「推的」「没在实现员身上试」「被攻过零轮」的那些身段（H1 第 1、2 步的推断、H2 整格、R1/R2/R4 改法的效果预测）——这些是它自己标注的推论/未验证内容，不属于「引用、产物、复跑」的核查范围。
- 未核 Opus 报告「没打中的形状」一节里除已列的几处以外的其余逐处读取（如 `oov-check.py` 实跑那次、三份 frontmatter 撞名试验），因这些是该腿自陈的探索过程而非可独立复核的引用/产物，超出优先项范围。
- 本地攻方材料的第四节「多出来的限定词」表，发现有一处（P2 occurrence 1 的 SHI_COPULA 旁注）内容属实但未被该表收录，记为观察项，不计入 ✓/✗ 计数——这不是转述错误，只是索引不全。
- 未对本地攻方两份样本（`-output-s1.md`、`-output-s2.md`）自身的 44 条编号答复做内容正确性判断：按分派，这一步已由主 agent 拿真值逐格比过（40/40 相同），只复核了退出前置条件（词数、`oov-check.py`、`corruption-check.py` 三个命令的复跑结果与运行记录逐字节一致）。
- 未跑门禁全量、未做任何 git 写操作；仓里除本报告文件与 `/tmp/claude-1000/m2-agentdef-r1-verifier/` 草稿目录外未写任何文件。
- 草稿目录里另有：`opus-model/` 复跑用的模型拷贝与 5 份 rerun 输出（`mtime_toy_rerun.out`、`probe_gate71_rerun.out`、`probe_r2_rerun.out`、`heading_probe_rerun.out`）、`selftest/` 判别力自证用的副本、`mtime_toy_work/`、`emptywork/`、`r2work/` 三个探针空工作目录；均为一次性验证产物，不承重、不需要入库。

