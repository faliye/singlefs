# m2-agentdef-r1 正推腿报告（判 A3、A4）

分工：本腿只判 A3（入口完整）、A4（规则与门禁对得上、三道门禁覆盖不漏不重）。不判 A1、A2（Opus 攻方）。

## A3：入口完整

### A3-a 逐句核对：改前 `CLAUDE.md`「什么时候派哪个 agent」→ 今天 `main-agent.md` / `CLAUDE.md`

源文件：`research/prompts/m2-agentdef-r1-claude-md-at-head.md`（主 agent 给定的冻结件，第 8-27 行是「什么时候派哪个 agent」整节）。今天的对照文件：`.claude/main-agent.md`、`CLAUDE.md`（均已现查 `sha256sum -c research/prompts/m2-agentdef-r1-start-snapshot.sha256` 全部 `OK`，含 `main-agent.md`、`CLAUDE.md`，与我读到的内容一致）。

逐句对照表（源句按冻结件行号；今天的行号已用 `grep -nF` 现查）：

| 冻结件原句（摘要，完整原文见冻结件对应行） | 今天在哪 | 判定 |
|---|---|---|
| L10「主 agent 只调度和判断……其余按下表派。」 | `.claude/main-agent.md:12`（`grep -nF` 命中；「下表」改写成「下面那张表」，同义） | 一致 |
| L10「定义在 `.claude/agents/`……共用约束在 `.claude/agent-common.md`。」 | `main-agent.md:12` 逐字命中 | 一致 |
| L10「一次性的活照旧临时写提示派 general-purpose。」 | `main-agent.md:12` 逐字命中 | 一致 |
| L10「改了定义要新派才生效……新建的定义要等几秒才派得出去。」 | `CLAUDE.md:12` 逐字命中 | 一致 |
| L10「改了本文件（连同它 `@` 的规则）要新开会话才对派出去的 agent 生效……同一会话里派的继承会话开始时那一份。」 | `CLAUDE.md:12` 逐字命中；这句仍留在 `CLAUDE.md` 里，「本文件」的指代对象没变 | 一致 |
| L10「计划、实测与现状在 `records/2026-09-16-subagent拆分提案.md`。」 | `CLAUDE.md:12` 逐字命中 | 一致 |
| L14「派发提示与发给别的会话的消息，在不影响正确性的前提下写简练……不删内容。」 | `main-agent.md:36` 逐字命中（只删了「（用户 2026-09-17 定）」这个日期括注） | 一致 |
| L16-26 表格 9 行（要推论/改 crates 两种/跑变异表/暂存/撤回/定案/建实验/别家事实） | `main-agent.md:40-51`：9 行逐字都在，另加一行「一个阶段任务结束……」（sweep 阶段同步），是纯新增 | 一致 |


**冲突：L12 段落丢了一句可执行的调度指令。**

冻结件 `research/prompts/m2-agentdef-r1-claude-md-at-head.md:12` 逐字（`grep -nF` 命中）：
「派出去之后要盯住，**不能一直干等**，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本（2026-09-17 用户定）。……」

今天 `.claude/main-agent.md:24`（`grep -nF` 命中）：
「盯住，不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。……子 agent 全部**交回或被停**也退出……」

对比：
- 「不能一直干等」这半句整句不见了（不是摘句，是整句删除）；`grep -n "不能一直干等" .claude/main-agent.md CLAUDE.md .claude/agent-common.md` 三份文件全部 0 命中（命令已跑，退出码 1）。A3 判据只认 `main-agent.md` 或 `CLAUDE.md` 两处，两处都没有。
- 「子 agent 全部结束也退出」→「子 agent 全部**交回或被停**也退出」：这是措辞收紧（区分「交回」与「被停」两种结束方式），不是丢内容，判一致。
- 「（2026-09-17 用户定）」「经过见 `records/2026-09-17-已分配口径三方与两个实验.md` 第六节」两处被删：这是纯粹的日期/经过标注，删除后不改变任何可执行步骤，符合 `.claude/rules/implementation-workflow.md:18`（`grep -nF` 命中）「定义里不写、也不为它留解释的尾巴」的新规则，判一致；这条清理动作本身改没改指令是 A1 的判区，这里不越界判。

**这条冲突够不够格「一句调度指令两处都没有」（A3 反向接受条款）**：够格。「不能一直干等」是一条独立的行为指令（禁止阻塞式等待），不是「盯住」「不强制结束」的同义重复——它管的是主 agent 自己会不会卡在原地等子 agent，而后面那句「立刻用 `run_in_background` 起看门狗」是**满足这条禁令的具体做法**，两者不是同一句话的两种写法。指令句被删而做法句留着，等于「为什么不能傻等」这条禁令本身消失了，只剩下「应该怎么做」的操作步骤。

**什么现象会推翻它**：若日后在 `main-agent.md` 或 `CLAUDE.md` 里找到「不能一直干等」或语义等价的禁令句（例如「不许阻塞式等待」「不能同步等它跑完」），这条冲突就不成立。目前两处都是 0 命中（现查命令见上）。

**风险评估（不越界判决，只报现象）**：由于看门狗机制（`run_in_background` 起 `agent-watch.py`）仍然完整保留在 `main-agent.md:24`，实际操作步骤没有变化；丢失的只是这条步骤「为什么必须这样做」的禁令性说明。按 `.claude/rules/implementation-workflow.md:18` 的新规则，这类「为什么」本就该搬进 kb 或 records，不留在定义里——但这条不是「为什么」，是一条独立的行为约束（做什么/不做什么），不应该被当成解释性尾巴一起删掉。

### A3-b 主会话开工时会不会读到 `main-agent.md`

现查命令与输出：

```
$ grep -c '^@' CLAUDE.md
21
$ grep -n 'main-agent' CLAUDE.md
10:**所有任务从 [.claude/main-agent.md](.claude/main-agent.md) 进**：主 agent 的职责、一轮怎么开怎么收（出口、判阻塞、收拢、再判）、派出去之后怎么盯、交回怎么读、派发提示怎么写、什么时候派哪个 agent 的调度表，都在那一份。这份文件只放公共上下文：项目是什么、当前里程碑、规则、项目本地事实。
```

事实：`CLAUDE.md` 用 `@` 引了 21 份规则（会被 Claude Code 自动注入上下文），`main-agent.md` 不在其中，只用普通 Markdown 链接指了一句。这与背景材料「现查事实」表第一行给的观测一致（现查复核过，一致）。

**判定：一致，但依据的是这个项目一贯的做法，不是技术强制。** 理由：
1. `CLAUDE.md` 本身会被 Claude Code 自动加载进主会话上下文（这是本会话的共同前提，`main-agent.md` 的 frontmatter 也写着「会话里的主 agent 从这一份进（`CLAUDE.md`「任务从哪进」）」，`grep -nF` 命中 `.claude/main-agent.md:3`）。
2. `CLAUDE.md` 被自动加载的正文里，第一个实质小节（「## 任务从哪进」）就是那句加粗的「所有任务从……进」，指令语气而不是旁白语气。
3. 这与全项目现有的做法同构：16 份 subagent 定义没有一份靠强制注入来读规则，全部靠「开工先读：」这一行文字指令（`.claude/gate.d/63-agent-write-scope.sh:11` 检查⑦，`grep -nF` 命中：「正文有一行以「开工先读：」开头（不继承 `CLAUDE.md` 之后，要读的规则全靠这一行点名）」）。用同一套「写指令、靠读」的做法去要求主会话读 `main-agent.md`，与项目现行纪律一致，不是新出现的薄弱环节。

**复核不了的部分**：我没有能力另开一个全新的顶层会话去实测「一个真正从零开始的主会话，会不会先读 `main-agent.md` 再动手派 agent」——这需要主 agent 自己或另一条腿去做真实的会话冷启动观测，我只能核实静态文本，不能核实运行时行为。

**什么现象会推翻它**：若某次真实的新会话开工记录（transcript）显示主 agent 读了 `CLAUDE.md` 之后没有去读 `main-agent.md` 就直接派了 agent、且因此漏用了某条只写在 `main-agent.md` 里的规则（比如「每次派发之后立刻起看门狗」），就证明「写一句指令」不足以保证「读到」，这条判定要改判冲突。

## A4：规则与门禁对得上；三道门禁覆盖不漏不重

### A4-a `implementation-workflow.md` 两节新规则 vs 门禁 71、72、63 号

`.claude/rules/implementation-workflow.md` 全文只有 45 行（`wc -l` 已核）。两节新规则里点名门禁的位置，`grep -n "71\|72\|63 号\|gate.d/63" .claude/rules/implementation-workflow.md` 现查只有两行命中：

```
18:**定义是工作流，不是说明**（用户 2026-09-18 定）：……门禁 71 号判这一条。
20:**改它们与改 `crates/` 同规矩**：……门禁 72 号判形式（形态照 56 号）。
```

**第一节（改 agent 定义与共用约束，走同一条三步）↔ 71 号：一致。** 规则要求「`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做」，71 号的目标集是 `targets = sorted(glob.glob(".claude/agents/*.md"))`（`.claude/gate.d/71-agent-def-flow-only.sh:38`，`grep -nF` 命中），也就是 `.claude/agents/` 下全部 `.md`，三份对象（含共用约束与主 agent 入口）都在集合里，范围对得上。现跑 `bash .claude/gate.d/71-agent-def-flow-only.sh`：`✓ 定义只写怎么做（18 份文件 824 行；4 行带日期……）`，退出码 0。

**第二节（改它们与改 `crates/` 同规矩）↔ 72 号：一致，但当前红（原因见 A4-b）。** 规则要求判决「按路径点名改过的每一份定义」，72 号的目标集是 `grep -E '^\.claude/agents/[^/]+\.md$'`（`.claude/gate.d/72-agent-def-adversarial-review.sh:25`，`grep -nF` 命中），同样覆盖 `.claude/agents/` 直属的全部 `.md`。现跑 `bash .claude/gate.d/72-agent-def-adversarial-review.sh`：退出码 1，报 17 份缺引用点名（详见 A4-b）——这是「这一轮的判决还没写」造成的红，不是规则与门禁判法本身不一致（本轮的判决文件 `research/prompts/m2-agentdef-r1-main-verification.md` 写出并点名 18 条路径之后，这一项理应转绿；但 A4-b 会指出即便转绿，其中一份仍然可能是假阳性满足）。

**第二节（代码轮派腿之前记一份开工快照）：不点名任何门禁，无门禁可比对，判「规则没说」。** `.claude/rules/implementation-workflow.md:24-28`（`grep -nF` 命中标题行）全文没有出现「71」「72」「63」或「gate」字样。这条新规则目前只靠人工遵守；本轮自己确实照做了——现查 `sha256sum -c research/prompts/m2-agentdef-r1-start-snapshot.sha256`，24 个文件全部 `OK`（含 3 道门禁脚本、`CLAUDE.md`、`implementation-workflow.md` 本身、18 份 `.claude/agents/*.md`、`disposition.md`），证明这一轮腿跑的时候这些文件确实没被主 agent 改过，做法与规则文字一致。**什么现象会推翻它**：若这些文件里任何一个的哈希对不上快照，就说明规则被违反了；目前全部吻合。

**门禁 63 号：两节新规则都没提到它，判「规则没说」——但 63 号自己的判法与一条新观测冲突，写在 A4-c。**

### A4-b 搬迁之后三道门禁对 `.claude/agents/` 下每个文件判一次，不漏不重

`.claude/agents/` 下现有 18 个 `.md` 文件（`ls .claude/agents/*.md | wc -l` → 18；`ls -la` 已现查，无 `INDEX.md`、无子目录）：`agent-common.md`、`main-agent.md`、另外 16 份 subagent 定义。

覆盖矩阵（现查三道门禁源码 + 实跑结果）：

| 文件 | 71 号（内容风格） | 72 号（三方引用） | 63 号（写范围/派发安全） |
|---|---|---|---|
| 16 份 subagent 定义 | 覆盖，当前绿 | 覆盖（今天全部改过），当前红：缺三方判决点名 | 覆盖，当前绿（16 份都有 `omitClaudeMd`、「开工先读：」，见下） |
| `agent-common.md` | 覆盖，当前绿 | 覆盖，**当前假绿**（见下） | **被显式跳过，零检查** |
| `main-agent.md` | 覆盖，当前绿 | 覆盖，当前红：缺三方判决点名 | **被显式跳过，零检查** |

**72 号现跑结果**（`bash .claude/gate.d/72-agent-def-adversarial-review.sh`，退出码 1）：报 17 份「没有三方判决点名」，唯独 `.claude/agent-common.md` **不在缺失名单里**。查为什么：

```
$ grep -rl '\.claude/agents/agent-common\.md' research/prompts/*-main-verification.md
research/prompts/agent-defs-r1-main-verification.md
$ grep -n '\.claude/agents/agent-common\.md' research/prompts/agent-defs-r1-main-verification.md
30:| A1 执行类照共用约束用 Bash 写……| `.claude/agent-common.md`「写」：……
48:| 编造的命令输出、行号差 1 | 核查员 | **采纳（共用约束）** |……| `.claude/agent-common.md`「报告」：……
49:| 报告不写文件……| `.claude/agent-common.md`「报告」：……
```

`research/prompts/agent-defs-r1-main-verification.md` 是**更早一轮、与今天这轮无关的判决**（今天的对象改动、清理与迁移都发生在 `records/2026-09-16-subagent拆分提案.md` 第二十八、二十九节，日期都是 2026-09-18；那份旧判决审的是当时的共用约束内容，不可能审到今天的 frontmatter、清理结果）。72 号的判法是「`grep -qF` 命中即算点名」（`.claude/gate.d/72-agent-def-adversarial-review.sh:39`，`grep -nF` 命中该行），不区分判决是不是这一轮产出、是不是这一轮改动之后写的。全仓另外 61 份 `*-main-verification.md`（`ls research/prompts/*-main-verification.md | wc -l` → 62，减去这一份）里，`grep -oE '\.claude/agents/[A-Za-z0-9_.-]+\.md' research/prompts/agent-defs-r2-main-verification.md` 零命中，其余也没有再命中 `.claude/agents/` 路径的（`grep -l '\.claude/agents/' research/prompts/*-main-verification.md` 只列出这一份）。

**判定：漏。** `agent-common.md` 今天改动很大（新增 frontmatter、清理经过），但 72 号因为一份**审的是别的内容、时间也更早**的旧判决里恰好出现同一条路径字符串，就判定它「已点名」。这与 72 号自己文件头写明的盲区（`.claude/gate.d/72-agent-def-adversarial-review.sh:7-8`，`grep -nF` 命中：「与 56 号同一个盲区——路径出现在「没攻它」那句里也算点名」）是同一族问题，但更严重：不要求判决与改动**同一轮**，甚至不要求判决**晚于**改动。**什么现象会推翻它**：若 72 号的判法改成「判决文件必须在这次改动的 `git diff` 基准之后新增或修改」（比如按 mtime 或按 `git log --since` 过滤候选判决文件），这份旧判决就不会再被算作点名，这条「漏」就被堵上；目前源码里没有这样的过滤，退出码 1 的现跑结果印证了另外 17 份文件确实（正确地）被判缺失，只有这一份被误判为满足。

**63 号现跑结果**：`bash .claude/gate.d/63-agent-write-scope.sh` → `✓ ……（3 个有 Write 或 Edit 的定义、16 条路径模式）`，退出码 0，对 16 份真定义判绿。但 `agent-common.md`、`main-agent.md` 两个文件在 63 号的检查循环里被整段跳过（`.claude/gate.d/63-agent-write-scope.sh:83-86`，`grep -nF` 命中 `not_definitions = {"agent-common.md", "main-agent.md"}` 与 `if os.path.basename(path) in not_definitions: continue`）——不只是跳过 frontmatter 检查，`tools_line` 解析与 `writers.append(...)` 也一起跳过，这两个文件不会进入「有 Write/Edit 就必须登记写范围」的判定。**判定：漏（按设计，但设计前提被下面的新观测推翻）。**

**综上，「不漏不重」的判定：71 号不漏不重（18/18，逐个判一次）；72 号有一处假阳性造成的漏判（agent-common.md）、其余 17 份是真实、待补的漏（本轮判决写出后应清零）；63 号对 2/18 文件按设计零检查，是否构成「漏」取决于 A4-c 的新观测。三道门禁之间没有发现「重」（同一份文件被两道门禁按同一标准判两次）——三道门禁各查不同的维度（内容风格 / 三方引用 / 写范围与派发安全），职责不重叠。

### A4-c 主 agent 给的新观测：frontmatter 只有 `name`/`description` 之后可被派发，冲了 63 号的前提

**主 agent 派我之前给的观测**：`agent-common.md`、`main-agent.md` 加了只有 `name`、`description` 的 frontmatter 之后，Claude Code 把它们列进了可派发的 agent 类型，工具是全部工具。现查 `records/2026-09-16-subagent拆分提案.md:700`（`grep -n` 命中）：

「**预想（交用户）**：上游 doc-lint 要求 `.claude/agents/` 下每个 `.md` 都有 frontmatter 的 `name` 与 `description`（只豁免 `INDEX.md`），这两份照这条各加了一段最小 frontmatter，`description` 写明不是可派发的 agent；代价是它们会出现在可派发 agent 的清单里（加完之后主 agent 现查：Claude Code 把 `agent-common`、`main-agent` 列成可派发的 agent 类型，工具是全部工具）。另两条路：上游 doc-lint 加豁免（要发版会话），或退回原位置。」

这与主 agent 给我的新观测逐字对应，不是我这轮才发现的，是主 agent 自己已经现查并写进 records 的一条已知风险。我现查确认三点：

**1. frontmatter 现状**：`.claude/agent-common.md:1-4`、`.claude/main-agent.md:1-4` 都只有 `name`、`description` 两个键（已直接 Read 过两份文件全文），都没有 `tools:` 字段，两份 `description` 都写着「不是可派发的……：不要派发它」。

**2. 写入侧安全，靠的是 `write-guard.sh` 的默认拒绝，不是 63 号**：现场跑（两条命令原样输出）：

```
$ echo '{"tool_name":"Write","agent_type":"agent-common","tool_input":{"file_path":"crates/singlefs-core/src/lib.rs","content":"x"}}' | bash .claude/hooks/write-guard.sh; echo "exit=$?"
✗ agent-common 在写范围表里没有登记，按拒绝处理：crates/singlefs-core/src/lib.rs
→ 怎么办：在 .claude/hooks/agent-write-scope.tsv 按它定义的「写范围」一节补上路径模式；补之前这件活交回主 agent。
exit=2
$ echo '{"tool_name":"Write","agent_type":"main-agent","tool_input":{"file_path":"crates/singlefs-core/src/lib.rs","content":"x"}}' | bash .claude/hooks/write-guard.sh; echo "exit=$?"
✗ main-agent 在写范围表里没有登记，按拒绝处理：crates/singlefs-core/src/lib.rs
exit=2
```
`.claude/hooks/agent-write-scope.tsv` 里没有 `agent-common`、`main-agent` 两行（`grep -n "^agent-common\|^main-agent" .claude/hooks/agent-write-scope.tsv` 退出码 1，零命中）；`decide_scope()` 的逻辑是「有文件、无登记 ⇒ 拒绝」（`.claude/hooks/write-guard.sh:71-76`，`grep -nF` 命中 `if not patterns: return 2`），不是「无登记 ⇒ 放行」。**所以若这两个名字真被当成 `agent_type` 派发，写文件会被拒，不会越权写。**

**3. 但 63 号自己放弃了对这两个文件的实质检查，前提是「它们不是定义、不会被派发」——这个前提现在不成立。** 63 号第 ⑦ 条的判据注释（`.claude/gate.d/63-agent-write-scope.sh:11`，`grep -nF` 命中）：「`.claude/agents/` 里每个定义（共用约束 `agent-common.md` 与主 agent 入口 `main-agent.md` 不是定义，不判）的 frontmatter 有 `omitClaudeMd: true`，正文有一行以「开工先读：」开头（不继承 `CLAUDE.md` 之后，要读的规则全靠这一行点名）」。这两个文件**没有** `omitClaudeMd: true`，也**没有**「开工先读：」这一行（已直接 Read 全文确认）。如果它们真的从不会被当成可派发对象，缺这两样不要紧；但新观测说它们**已经出现在可派发清单里、工具是全部工具**，一旦被（哪怕是误）派发，按 63 号自己在同一行写的道理，它们会「不继承 `CLAUDE.md` 之后，要读的规则没有地方点名」——而且它们连「不继承 `CLAUDE.md`」这半句本身都不满足，会整份带上项目 `CLAUDE.md` 与它 `@` 的规则，正是 63 号原本要拦的那类代价（`.claude/gate.d/63-agent-write-scope.sh:119`，`grep -nF` 命中：「每次派发都白带约 10 万 token 的 `CLAUDE.md` 与规则」）。

**判定：冲突。** 63 号「这两份不是定义，不判」的判法，建立在「它们不会被派发」这个假设上；这个假设已经被主 agent 自己的现查推翻。63 号的代码没有跟着改。

**补充第一手观测（口径更窄，供交叉核对，不作为独立证据链）**：我自己这次是以 `three-way-forward` 身份被派发的，其定义 `.claude/agents/three-way-forward.md:1-6` 确认写了 `omitClaudeMd: true`。但在这一轮任务开始时，我在没有主动 `Read` 的情况下，收到了一段以「Contents of `/home/fy5090/code/singlefs/.claude/singlefs-ai-sop/CLAUDE.md`」开头的系统提示，随后又收到 13 段分别以「Contents of .../rules/《文件名》」开头的系统提示（`engineering-philosophy.md`、`sop-first.md`、`show-me-test.md`、`machine-first.md`、`code-discipline.md`、`writing-discipline.md`、`design-doc-discipline.md`、`kb-discipline.md`、`test-discipline.md`、`verify-before-claiming.md`、`pushback-discipline.md`、`command-safety.md`、`session-wrapup.md`——这是 `.claude/singlefs-ai-sop/CLAUDE.md`「## 规则（始终生效）」一节 `@` 引的 14 份规则里的 13 份，缺的第 14 份 `evidence-discipline.md` 是因为我这一轮自己按「开工先读」指令主动 `Read` 过，大概率被去重）。同一次会话里，项目根的 `/home/fy5090/code/singlefs/CLAUDE.md`（「项目 `CLAUDE.md`」本尊）以及它 `@` 的项目本地规则（`fs-design.md`、`format-evolution.md` 等）**没有**被自动注入——我是这一轮自己主动 `Read` 到的。这与「`omitClaudeMd: true` 只挡住了根 `CLAUDE.md` 本尊，挡不住仓里另一份嵌套的 `.claude/singlefs-ai-sop/CLAUDE.md`」这个现象吻合。**这一条我只在自己这一次会话里观测到一次，是我自己的转录（transcript），主 agent 可以直接核对这次派发的完整记录来复核；我不能另起一次派发去交叉验证，所以不升级成独立结论，只作为对 A4-c 冲突判定的旁证。什么现象会推翻它**：若主 agent 核对另一条同样 `omitClaudeMd: true` 的腿（例如同一轮的本地攻方腿或 Opus 攻方腿）的转录，发现它没有出现同样的自动注入，说明这是我这次会话独有的现象，不是通用行为，这条旁证要撤回。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A3-a 逐句核对 | 冲突（一处） | `main-agent.md:24` 丢了「不能一直干等」这句独立禁令，两处（`main-agent.md`、`CLAUDE.md`）都 0 命中；其余全部句子与整张表都一致或纯新增 |
| A3-b 主会话读不读得到 `main-agent.md` | 一致（有条件） | `CLAUDE.md` 未 `@` 引 `main-agent.md`，但自动加载的正文第一节就是指令式指路，与全项目「开工先读」惯例同构；运行时是否真读到，复核不了 |
| A4-a 两节新规则 vs 71/72/63 | 一致（71/72）+ 规则没说（第二节、63 号） | 第一节点名 71、72 且范围对得上；第二节与 63 号都没被这两节新规则提及，用快照哈希核实了本轮确实照第二节做了 |
| A4-b 三道门禁覆盖 `.claude/agents/` 18 个文件 | 71 不漏不重；72 有 1 处假阳性 + 17 处待补真漏；63 对 2/18 按设计零检查 | 71 号 18/18 判过且当前绿；72 号当前红 17 份，`agent-common.md` 因命中一份无关旧判决被误判为已点名；63 号显式跳过 `agent-common.md`、`main-agent.md` |
| A4-c 新观测：frontmatter 导致可派发 | 冲突 | 63 号「这两份不是定义、不判」的前提被主 agent 自己现查的事实推翻（已列入可派发清单、工具全开）；写入侧靠 `write-guard.sh` 的默认拒绝暂时兜底，但 `omitClaudeMd`/「开工先读」两项防线仍然缺失 |

## 没做什么

- 不判 A1（清理改没改指令）、A2（71 号漏判/误判）：按分工归 Opus 攻方腿。
- 没有另开一个全新顶层会话去实测「主会话开工时真的会不会先读 `main-agent.md`」；A3-b 的判定停在静态文本层面，运行时行为复核不了。
- 没有交叉核对本轮另外两条腿（Opus 攻方、本地攻方）的转录来验证 A4-c 里「嵌套 `.claude/singlefs-ai-sop/CLAUDE.md` 绕过 `omitClaudeMd`」这条现象是不是通用的——按禁读清单，我不读它们这一轮的产出，这条观测只以我自己这次会话的转录为准，已在正文里注明推翻条件。
- 没有去验证「用户级 CLAUDE.md」与「主 agent 私有记忆」是否也存在同样的嵌套穿透问题（agent-common.md「规则怎么读」一节提到的另外两类），本轮没有观测到它们被注入，但也没有专门设计动作去验证它们的边界。
- 没有修改任何规则、门禁或定义文件——按写范围只写这份报告。
- 没有判断 A4-c 这条冲突该怎么改（比如给这两个文件也补 `omitClaudeMd`、`开工先读`，或是否要走「预想（交用户）」里列的另外两条路：上游 doc-lint 加豁免、或退回原位置）：这是判决与用户决定的事，不是正推腿的职责。
