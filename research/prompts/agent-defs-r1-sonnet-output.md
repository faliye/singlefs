# 正推腿报告（agent-defs-r1，Sonnet）

对象：观测类定义 `sweep`、`prior-art`、`mutation-triage`、`gate-triage`，设计类 `experiment-designer`；
以及 `CLAUDE.md`「什么时候派哪个 agent」与 `.claude/gate.d/stage-owners.tsv` 对全部 16 个定义。
判据格：J1（事实）、J2（规则冲突）、J5（分工）、J6（试跑读数）。

方法：对每个对象逐格判，一致 / 冲突 / 规则没说三选一；冲突的两句整行并排抄；能核的事实全部现跑，
命令与原样输出贴在对应小节；试跑报告与定义现状逐句对照，不凭印象。附录里的 16 个定义正文、
`agent-common.md`、`stage-owners.tsv`、`CLAUDE.md` 那一节，与仓里现存文件逐字比对一致（下面每节列了
具体比对命令），没有发现材料抽取之后又被改动的情形。

## 零、一条不属于判据格、但要先说明的事实差异

我的系统提示里注入的 `CLAUDE.md` 全文（工具描述里带的项目文件内容）与仓里现在的 `CLAUDE.md`
**不是同一个版本**：注入版本还有一整节「## 门禁」（列出 `gate.sh`／`check.sh`／`lkmm.sh` 与全部
`.claude/gate.d/*.sh` 阶段），而当前仓里的 `CLAUDE.md` 已经没有这一节（`records/2026-09-16-subagent
拆分提案.md` 第十五节记的正是这次删除：「CLAUDE.md 删掉门禁一节；调度表里实现那一行接上
`crash-verifier`」）。现查：

```
$ grep -n "^## " CLAUDE.md
8:## 什么时候派哪个 agent
24:## 规则（始终生效）
41:## 规范从哪来
50:## 项目本地规则
61:## 项目本地事实
88:## 本项目的特殊性
95:## 一句话版本
$ wc -l CLAUDE.md
99 CLAUDE.md
```

没有「## 门禁」这一节，与注入版本不同。**这不构成对背景材料的否定**：背景材料附录里整段抄出的
`CLAUDE.md:8-23`（附录里「出处 `CLAUDE.md:8-23`（整段抄，未转述）」那一块，见附录第 1961–1980 行）逐字
与仓里现文件相同（下面「六」核过），说明材料员用的是仓里的现文件，不是我系统提示里那份旧内容。我
在下面全部判断都以 `bash` 现读的仓内文件为准，不用系统提示里注入的版本——这是我这轮派发提示「注
意」一节「定义与共用约束在派这一轮之前刚按试跑改过，以仓里现在的文件为准」这句要求的一个实例，写
在这里是为了说明我没有拿旧版本去核对背景材料，而不是说背景材料本身有问题。
**什么现象会推翻这条**：如果背景材料附录抄出的 `CLAUDE.md` 内容与仓里现文件对不上，才说明材料员用
错了版本；现查过，没有这种情形。

## 一、`sweep`

### J1 事实：一致

逐条现查 `.claude/agents/sweep.md`（38 行）里点名的依据与命令，与仓里现状一致：

- 依据「`.claude/rules/format-evolution.md`「改一个格式常量：旧值的派生形态要逐类搜，改完登记进
  `stale=`」」——该文件里确有这个标题：
  ```
  $ grep -n "^## 改一个格式常量" .claude/rules/format-evolution.md
  36:## 改一个格式常量：旧值的派生形态要逐类搜，改完登记进 `stale=`
  ```
- 依据「`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「新立一条判据，当场拿它回扫已有的条
  目」及其两个子小节」——标题与两个子小节都存在：
  ```
  $ grep -n "^## 新立一条判据\|^### 撤回一个数或一条结论\|^### 撤回的理由失效" \
      .claude/singlefs-ai-sop/rules/evidence-discipline.md
  246:## 新立一条判据，当场拿它回扫已有的条目
  254:### 撤回一个数或一条结论，同样要当场回扫谁在引它
  277:### 撤回的理由失效，不等于被撤回的结论自动回来
  ```
- 依据「`.claude/singlefs-ai-sop/rules/verify-before-claiming.md`「核了窄的那一句，说出口的却是宽的
  那一句」」——标题存在（见文首规则原文，未重复贴）。
- 步骤 4 点名的 `.claude/gate.d/27-format-constants.sh` 存在且能跑：
  ```
  $ nice -n 19 bash .claude/gate.d/27-format-constants.sh
    ✓ 格式常量同步（19 个已登记，19 个在源码里被钉住）
  ```
  这条我自己现跑了一遍（sweep 的定义要求「不跑 gate.sh 全量，除非定义明写」，但单独跑一个门禁阶段
  不在此限，且这正是 sweep 的试跑报告自己说「没有实际执行门禁去验证」的那一句——见 J6）。

**什么现象会推翻它**：sweep.md 里任何一句依据指向的标题、脚本、行为，在仓里核不到或核出不同的
内容。这次全部核到一致。

### J2 规则冲突：没打中

sweep 的写范围「报告文件、草稿目录。不改任何被搜到的文件，不改 `stale=`」与 `agent-common.md`
「写」一节（只写派发提示与定义「写范围」给的路径）一致，没有冲突。sweep 的「只找、只分类，不改」
与它引用的 `evidence-discipline.md`「新立一条判据，当场拿它回扫已有的条目」这条规则本身要求的是
「当场拿它回扫」，字面看似乎要求「当场改」，但该条规则的动作主语是**立判据的人**（这个仓里通常是
主 agent 或 `kb-scribe`），不是 sweep；sweep 是被派去做「回扫」这个动作的执行者，检索之后把清单交
主 agent 或 `kb-scribe` 去改，这与 `agent-common.md`「不做」一节「本机常有别的会话在同一个仓里干活：
只动这一轮自己的文件」以及三方论证共同约束里「主 agent 只调度和判断」的分工原则相容。**没有发现
sweep 定义要求的做法与被它引用的规则原文互相矛盾的地方。**

**什么现象会推翻它**：能指出 sweep 引用的某条规则原文，字面就是要求「回扫的人自己当场改」，而
sweep 定义却写「不改」。核过引用的三处规则原文，没有这种情形。

### J5 分工：一致

`CLAUDE.md` 调度表第 19 行「撤回一个数、改格式常量、新立一条判据 | `sweep`」与 sweep.md 的
`description` 逐字对应：「回扫员（sweep）：撤回一个数、改一个格式常量、或新立一条判据之后，全仓找
还在引旧值或该被新判据管到的地方，分类交清单」——「撤回一个数」「改一个格式常量」「新立一条判据」
三个触发条件在两处逐字相同。`.claude/gate.d/stage-owners.tsv` 里没有任何一行把 `sweep` 登记成某个
门禁阶段的 owner（现查：`grep -c "sweep\b" .claude/gate.d/stage-owners.tsv` → `0`），这与 sweep.md
「做什么」一节完全不提「阶段归属表」步骤一致——它不欠任何门禁阶段。

**什么现象会推翻它**：`stage-owners.tsv` 里出现了 `sweep` 作为某阶段 owner，而 sweep.md 没有对应
的「跑阶段归属表登记给你的阶段」步骤，或者反过来。现查这两处都没有。

### J6 试跑读数：两处已在今天的定义里改掉，三处仍留给「要人看」（不是遗漏，是定义设计如此）

试跑报告 `research/prompts/agent-defs-r1-trial-sweep.md`「四、试跑观察」列了 5 点。逐点核对现状：

1. 「规则给的六种派生形态例子……唯一漏网的是……不含字面旧值、只是『更早一代』的残留数字……印证了
   任务输入里第 2 条『按量名／常量名／kb 用词再搜一遍』的必要性」——**现在的定义已经把这一条写进
   步骤本身**：
   ```
   sweep.md 第 24 行：
   2. 按量名、常量名、kb 用词再搜一遍，找不含旧值任何派生形态、却在算同一个量的地方（例：从更早
      一代的值派生出来的数）；这一遍的范围与第一遍相同，`records/` 也要扫。
   ```
   这句与试跑观察第 1 点要求的方向逐字对应（连括注「例：从更早一代的值派生出来的数」都用了同样的
   措辞），判**已改，不打中**。
2. 「`format-evolution.md` 没有明说『kb 正文』具体等价于门禁 27 号脚本里的哪个文件集合……建议在规
   则里显式点名一次门禁 27 号的排除清单」——**现在的定义步骤 4 已经这样写**：
   ```
   sweep.md 第 26 行：
   4. …「现扫」的范围照 `.claude/gate.d/27-format-constants.sh` 实际扫描的文件集合（读脚本取）。
   ```
   判**已改，不打中**。我另外现跑 `27-format-constants.sh` 验证了它排除 `*-history.md` 与
   `/decisions-history/`、且 `research/results/` 不在扫描范围（见 J1 小节的引用行号），与 sweep 试
   跑报告自己核对的范围逐字一致。
3. 「`checks-owed.md` 这类『追记』惯例……①处（无日期前缀）②处（有日期前缀）就是同一份文件里，仅
   因为有没有前缀就分别落进『要改』和『要人看』两类，这个判据本身不够硬，值得主 agent 或
   `kb-scribe` 定一条更明确的口径」——这一条**不是对 sweep 定义本身的意见**，是对 `checks-owed.md`
   书写惯例（属于 `kb-scribe`/主 agent 的职权）的意见；sweep 自己的「做什么」步骤 3 本来就把这类
   拿不准的情形路由进「分不清，要人看」这一类，试跑时**确实照这一类去做了**（trial 第 98–101
   行）。判**没打中 sweep 的定义**：sweep 的五分类设计本来就是让人工去接这一类边界情形，不是让
   sweep 自己发明一条更硬的判据。
4. 「`.claude/kb/experiments/79-根记录的容量.md:36` 与 `checks-owed.md` 的 C157 详细行……是同一笔
   债……这已经超出『树表条目 148→200 sweep』这个具体任务的边界，只是顺带看到，不建议我这一轮处
   理」——试跑报告自己已经标明这不在这一轮任务范围内，**不是定义缺陷**。
5. 「`crates/mutations.tsv` 目前没有任何一条变异 `TREE_TABLE_ENTRY_BYTES`……是否需要补，交主 agent
   判断」——同样是留给主 agent 的观察，不是 sweep 定义要改的东西。

**判定：J6 在 sweep 上没打中。** 试跑暴露的两条可操作的定义改进（第 1、2 点）已经被写进当前
`sweep.md` 的步骤 2 与步骤 4；其余三点本来就是「按定义路由给人看」的正常产出，不是定义漏改。

**什么现象会推翻它**：如果能找到试跑观察里某一条**要求 sweep 自己的步骤文本改变**、而现在的步骤
文本仍是试跑当时读到的旧文本，才算打中。逐条核过，没有这种情形。

## 二、`prior-art`

### J1 事实：一致

`.claude/agents/prior-art.md`（38 行）依据的三处引用现查：

```
$ grep -n "^## 别的项目怎么做\|^## 2\. 每条带出处与状态" \
    .claude/singlefs-ai-sop/rules/evidence-discipline.md .claude/singlefs-ai-sop/rules/kb-discipline.md
.claude/singlefs-ai-sop/rules/evidence-discipline.md:69:## 别的项目怎么做，是线索不是证据
.claude/singlefs-ai-sop/rules/kb-discipline.md:52:## 2. 每条带出处与状态
```

两个标题都存在，与 prior-art.md 第 13 行逐字一致。`tools` 行 `Read, Bash, WebFetch, WebSearch`：
`records/2026-09-16-subagent拆分提案.md` 第十二节实测过 Grep、Glob 在这个 Claude Code 版本里根本不
存在（「2.1.273 没有 Grep、Glob 这两个工具」），已从全部 15 个定义里删掉；同一节还实测到一次完整的
嵌套会话初始化消息里列了 38 个工具、其中含 `WebFetch`、`WebSearch`，与 Grep/Glob「完全不存在」不同
——这两个工具在这套 Claude Code 版本里是存在的。**我自己这条腿被派发时拿到的 `tools` 是 `Read,
Bash`，与我自己的定义文件 `three-way-forward.md` 的 `tools` 行一致，这印证了『子 agent 拿到的正是
frontmatter 里列的工具』这条机制**，但没有第一手数据能证明 `WebFetch`/`WebSearch` 在**当前这个主
agent 会话**里对 `prior-art` 这个 agent_type 实际可派发到（十二节的 38 工具清单是**另一次独立的嵌
套测试会话**里观测到的，不是这次会话本身的观测）。

**复核不了**：`WebFetch`/`WebSearch` 在当前主 agent 会话里派发给 `prior-art` 时是否真的可用，需要
主 agent 实际派一次 `prior-art` 并读它初始化时报的工具清单才能验证，我这条腿没有 Agent 工具，做不
到。prior-art 自己的试跑报告（`agent-defs-r1-trial-prior-art.md`）里没有用到 `WebFetch`/
`WebSearch`（那次任务本机有源码树，走的是 `Read`/`Bash`），所以试跑也没有验证过这两个工具真的可
用。

**什么现象会推翻「一致」这个判断**：主 agent 派 `prior-art` 时收到「工具清单解析为空」或
`WebFetch`/`WebSearch` 缺席的报告。

### J2 规则冲突：没打中

prior-art 步骤 3「每条写一处它与本工程的已知差异……差异写在一组事实背后的做法上，同一个做法的几
个数合写一处」与它引用的 `evidence-discipline.md`「别的项目怎么做，是线索不是证据」一节不冲突：
该规则区分「可以引」（事实、反证、机制）与「不可以引」（论证、正证、搬运）两栏，prior-art 步骤 3
「写不出就标『差异不明，只能当线索』」与规则表格「机制：把别家的做法拆成机制，再论证这个机制在我
们的前提下成立」一致——prior-art 只交事实与差异，不交「所以该怎么做」的论证，这与它自己在
「没做什么」一节写的「没论证它适不适合本工程」一致。

**什么现象会推翻它**：prior-art 步骤里出现一句要求它写「所以本工程该这样做」的论证性结论。核过
`.claude/agents/prior-art.md` 全文，没有。

### J5 分工：一致

`CLAUDE.md` 调度表「要别家文件系统的事实 | `prior-art`」与 prior-art.md 的 `description`「调研员：
现查别家文件系统的源码与文档，只交带出处的事实，不交论证」一致。`.claude/gate.d/stage-owners.tsv`
第 50 行「`70-citations.sh	kb-scribe,prior-art	外部引用还核得动；写进 kb 的是书记员，查源码树的是
调研员」（用「第 70 行」指这一行是错的——70 号是门禁阶段编号，不是表里的行号，这里的行号是现查
`grep -n` 得到的 50）与 prior-art.md 步骤 0「先跑阶段归属表登记给你的阶段……：70 号会报哪些承重引用指向的源码树
或文献不在本机了」一致，都指向同一个阶段、同一个分工描述（查源码树归调研员）。

**什么现象会推翻它**：调度表、归属表、定义三处中有任意两处说 `70-citations.sh` 该谁跑不一致。核
过，三处一致。

### J6 试跑读数：一处仍未改（且是否需要改要看一条反证），三处已改，两处不适用

试跑报告 `research/prompts/agent-defs-r1-trial-prior-art.md`「七、试跑观察」6 点，逐点核对：

**1（未改，但有一条反证）**：「『阶段归属表登记给你的阶段』这句指代的是哪张表，定义里没写清楚，
靠猜……建议调研员定义里直接给出那条 `awk` 命令，或者至少给出 `stage-owners.tsv` 的路径，不要只说
『共用约束「门禁」一节』」。现查 prior-art.md 步骤 0：
```
0. 要查本机源码树的，先跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一
   节）：70 号会报哪些承重引用……
```
与试跑当时读到的文本逐字相同，**没有改**。但有一条反证：prior-art.md 第 10 行「开工先读
`.claude/agent-common.md`」是每一个定义共用的第一句（我核过全部 16 个定义都有这句，见「六」），
这句要求恰恰是「先读共用约束」，而 `agent-common.md`「门禁」一节里逐字就是那条 `awk` 命令；试跑
报告自己也承认「这次因为读过 `agent-common.md` 才知道去哪查、命令怎么写」。也就是说，试跑设想的
「单独发给一个没读过 `agent-common.md` 的模型」这个场景，与全部定义「开工先读 `agent-common.md`」
这条结构性前提相矛盾——按定义的实际派发流程，不会有一个没读过 `agent-common.md` 就去执行
「做什么」步骤的 `prior-art`。如果按试跑的建议把 `awk` 命令抄进全部 7 个拥有门禁阶段的定义里
（`kb-scribe`、`experiment-runner`、`gate-triage`、`crash-verifier`、`three-way-materials`、
`implementation-writer`、`prior-art`），会违反「重复要生成，不许手抄」（`code-discipline.md`）——
7 份手抄的 `awk` 命令一旦 `stage-owners.tsv` 的路径或列结构改了，没有任何东西保证 7 处一起改。
**判定：J6 没打中**——试跑指出的现象是真的（这句话单独读确实不给出命令），但「要不要改」这一步
被「开工先读 agent-common.md」这条结构保证给否掉了；改法本身（抄命令进定义）会引入新的手抄重复
风险。

**2（不适用，未验证）**：「70 号阶段报『某条 D 编号的某句话』+『实测值 vs 预期值』的失败信息，不
是一句直接说『这棵树不在本机』的话，这次两棵树都在，没有踩到这条」——试跑自己已说明这条「没能验
证……没法用今天这轮实测去坐实或推翻」。**我也复核不了**：要验证需要真的让某条 70 号断言指向的
源码树缺失，而现查两棵树都在（`/home/fy5090/code/fs-refs/linux-6.17`、`/home/fy5090/kbuild/
linux-om`），我不会去删除或搬移承重引用的源码树来制造这个场景。

**3（已改）**：「『差异』是挂在『一组数字背后的做法』上，还是挂在『表格的每一行』上，定义没说
清楚」——现查 prior-art.md 步骤 3：
```
3. 每条写一处它与本工程的已知差异（负载、盘上格式、并发模型、兼容包袱其中之一）；差异写在一组
   事实背后的做法上，同一个做法的几个数合写一处；写不出就标「差异不明，只能当线索」。
```
「差异写在一组事实背后的做法上，同一个做法的几个数合写一处」这句与试跑建议的措辞（「一组数字背
后的做法」）几乎逐字对应。判**已改，不打中**。

**4（已改，随 3 一起）**：「『差异不明，只能当线索』这句兜底在本轮没有用上，无法判断写得够不够
用」——这句兜底现在就在步骤 3 的文本里（上面引用的最后一句），试跑观察本身也只是说「没有反面案
例」，不是说这句话有问题；**不是缺陷，是未触发**。

**5、6（不适用）**：草稿目录没用上、70 号阶段只覆盖树一——两条都是试跑报告自己标注为「不确定」
「不是阶段本身的缺陷」的观察，不涉及 prior-art.md 定义文本需要改的地方。

**判定：J6 在 prior-art 上，1 处仍是旧文本但有结构性反证撑住不改的理由，3、4 两处已经改了，2、5、
6 三处不构成「定义还没改」的打中。**

**什么现象会推翻这个判定**：如果主 agent 实际发生过「派 prior-art 却没有先读
`agent-common.md`」的情形（比如某种精简派发模式跳过了共用约束），那么第 1 点的反证就不成立，
应该改。现查 `agent-common.md` 本身与全部 16 个定义的第一句，没有找到允许跳过它的机制。

## 三、`mutation-triage`

### J1 事实：一致

`.claude/agents/mutation-triage.md`（39 行）依据两处：

```
$ grep -n "^## 变异测试证明的是断言会红" .claude/singlefs-ai-sop/rules/test-discipline.md
135:## 变异测试证明的是断言会红，不是覆盖
```
标题存在，与定义第 12 行逐字一致；`.claude/rules/mutation-sampling.md` 引用「全篇」，无需逐句核。

步骤 3 点名的 `.claude/gate.d/59-crates-mutation-replay.sh` 存在（见「零」之外我在一开始就核实过，
`ls` 命中）。输入一节「bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准
（多是连字符，源文件名是下划线）」——现查全部 100 个 `[[bin]]` 条目：

```
$ grep -c '^\[\[bin\]\]' research/e7-index-bench/Cargo.toml
100
$ grep -A1 '^\[\[bin\]\]' research/e7-index-bench/Cargo.toml | grep 'name = ' | grep -c '_'
0
```
100 个 `name` 字段没有一个含下划线（全部连字符），与「多是连字符」这句一致（其实是「全部」，定义
用词「多是」偏保守但不算错）。`mutate.sh` 自己对「bin 名对不上」的判定逻辑（见下面 J6）与定义描述
一致。

**什么现象会推翻它**：`Cargo.toml` 出现下划线命名的 `[[bin]]`，与「源文件名是下划线」的判据矛盾。
现查 100 个全部连字符，没有反例。

### J2 规则冲突：没打中

mutation-triage 写范围「不改变异表、不改源码、不改测试」看起来与它步骤 3「跑
`nice -n 19 bash research/scripts/mutate.sh …`」矛盾——`mutate.sh` 本身的工作原理就是「改一处源码
→ 编译测试 → 还原」。现查 `research/scripts/mutate.sh` 第 56、62、64、143 行：

```
$ grep -n "cp \"\$WORK_SRC\" \"\$BAK\"\|restore()\|trap cleanup EXIT\|^restore$" research/scripts/mutate.sh
56:BAK="$(mktemp)"; cp "$WORK_SRC" "$BAK"
62:restore() { cp "$BAK" "$WORK_SRC"; }
64:trap cleanup EXIT
143:restore
```
`mutate.sh` 自带备份/还原与 `trap cleanup EXIT` 兜底，净效果是源码不变（mutation-triage 自己的试跑
报告也贴出了「已还原，基线仍全绿」这一行）。**判没打中**：mutation-triage 定义里「不改源码」指的
是「不做自己的编辑」，工具本身的临时改动-还原是既有、经过设计的行为，两者不矛盾。

**什么现象会推翻它**：`mutate.sh` 某次异常退出没有触发 `trap cleanup EXIT`、导致源码真的被改动后
未还原。这次试跑与我自己读的脚本代码都显示有 `trap`，没有找到绕过它的路径。

### J5 分工：一致

`CLAUDE.md` 调度表「跑变异表、看抓到 / 无效 / 没红 | `mutation-triage`」与定义 `description`
「变异分诊：跑一张或几张变异表，报抓到 / 无效 / 没红三个数」逐字对应。`stage-owners.tsv` 里没有
`mutation-triage` 作为任何阶段的 owner（`grep -c "mutation-triage" .claude/gate.d/stage-owners.tsv`
→ `0`），与它自己「做什么」一节不提「阶段归属表」步骤一致。

值得记录、但**不构成冲突**的一处交叉：`crash-verifier` 也会跑 crates 变异表（`stage-owners.tsv`
第 46 行「`59-crates-mutation-replay.sh	crash-verifier	crates 变异表复跑`」，59 是门禁阶段编号
不是行号），而 mutation-triage
的输入一节也允许它跑 `crates/mutations.tsv` 里的条目（「或 `crates/mutations.tsv` 里的条目名」）。
这是两条不同用途的重叠授权：`crash-verifier` 是「实现改动走完三方之后」的固定流水线步骤（一次跑
整张表），`mutation-triage` 是随时可派的「诊断某张表哪几条没红／无效」的独立工具，`CLAUDE.md` 调
度表把两者列成两行不同的触发条件（「改 crates」流水线 vs 「跑变异表、看抓到/无效/没红」这个独立
条目），不是同一件事被两处不同地分配。

**什么现象会推翻它**：`stage-owners.tsv` 或调度表出现要求 `mutation-triage` 承担 `crash-verifier`
固定流水线里那一步、或反过来的说法。现查两处分工描述互不重叠（一个是"分诊"、一个是"复跑"）。

### J6 试跑读数：一处已改（且是最关键那一处），一处遗留但影响已被前一处的修复缩小

试跑报告 `research/prompts/agent-defs-r1-trial-mutation-triage.md`「试跑观察」3 点：

**1（已改）**：「派发提示写 bin 名 `e67_device_subset`，但 `Cargo.toml` 声明的是
`e67-device-subset`（连字符）……分诊定义没有写『bin 名对不上时怎么办』，只能靠脚本自己报的下一步
命令去猜」。现查 mutation-triage.md「输入」一节现在的文本：

```
- 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目
  名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源
  文件名是下划线）；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
```

这句直接给出了：① bin 名的权威来源（`Cargo.toml` 的 `[[bin]]`）；② 命名差异的一般规律（连字符 vs
下划线）；③ 撞上退出码 6 时的处置（照 `mutate.sh` 报的改法改，写进报告）——**逐字对应试跑遇到的
真实场景**（`mutate.sh` 退出码正是 6，给出的改法正是「用 `e67-device-subset` 重跑」）。判**已改，
打中的现象已经消失**。

**2（部分遗留，但影响已缩小）**：「`mutate.sh` 退出码有好几种，分诊定义第 4 步只写了一种判据：
『收尾没有「已还原，基线仍全绿」就是中途退出』；退出码 6（bin 名不对）发生在『基线：全绿』这行都
还没打印之前，比『中途退出』更早——脚本连基线都没跑」。现查 mutation-triage.md 步骤 4 原文：

```
4. 确认整张表跑完：收尾没有「已还原，基线仍全绿」就是中途退出，这一次的数不算，照实报。
```

这句字面确实**没有变**，仍然只给了一种判据形态，没有把「退出码 6（输入错误，基线都没跑）」与
「跑到一半才失败」区分开。但因为第 1 点已经在「输入」一节把 bin-名校正提前到**跑 `mutate.sh` 之
前**处理，实际操作序列变成：先按 `Cargo.toml` 校正 bin 名 → 再跑 `mutate.sh` → 这时才走到步骤 4
判「有没有跑完」。也就是说，试跑当时撞到退出码 6 是因为**先跑了错的命令**，而现在「输入」一节已
经要求先核对 bin 名，正常情况下不会再走到「退出码 6 发生在基线之前」这条路径；步骤 4 本身没有把
这个边界情形写全，是一处字面上仍然存在、但被上游步骤挡住了大部分触发概率的残留缺口。**判 J6 部分
打中**：步骤 4 的文本本身没有回应这条观察，但触发它的前置条件已经被第 1 点的修复削弱。

**3（不适用）**：「派发提示的例外条款只提到『允许照常编译』，没提到 bin 名要不要按 Cargo.toml
校正」——这是**派发提示**（主 agent 每次现写的内容）层面的问题，不是持久化定义文本的问题；现在
既然定义本身（不是派发提示）已经把 bin 名权威来源写死，未来派发时即使派发提示不提，agent 也会照
持久定义去做，这条观察已经被结构性地绕过。

**判定：mutation-triage 的 J6，最要紧的第 1 点已经改掉，第 2 点是字面残留但影响已缩小，不构成一
个会阻碍工作的缺口。**

**什么现象会推翻它**：如果未来一次真实任务里，`mutate.sh` 在**基线还没打印**之前就退出（不是
bin 名问题，而是别的原因，比如源文件路径不存在），mutation-triage 按现在步骤 4 的字面判据会把它
误判成「中途退出」而不是「一开始就没跑起来」——这种情形我没有构造出来验证，是这条判定的弱点，
留给主 agent 或 `crash-verifier` 复核。

## 四、`gate-triage`

### J1 事实：一致

`.claude/agents/gate-triage.md`（38 行）依据两处：

```
$ grep -n "^## " .claude/singlefs-ai-sop/skills/gate/SKILL.md
6:# 准入门禁
10:## 跑
20:## 阶段与判读
41:## 未实现的阶段
52:## 常见假失败
63:## 门禁自己也要能失败
```
「阶段与判读」「常见假失败」两节存在，与定义描述「阶段含义、常见假失败」意思对应（措辞是概括，不
是逐字引用，`.claude/agents/*.md` 的依据行本来就是概括指路，不受「引 kb 条目要整行抄」那条纪律约
束——那条纪律管的是三方论证材料引用 kb 决策/实验/规则条款，不管 agent 定义里指向 skill 文件的一
句话）。

```
$ grep -n "^## 4\. 同一个仓里有没有别的会话在飞" .claude/singlefs-ai-sop/rules/session-wrapup.md
47:## 4. 同一个仓里有没有别的会话在飞？
```
标题存在，与定义第 13 行逐字一致。

「写范围」一节声称的门禁自身 git 写行为，我现查了共享 `gate.sh` 源码：

```
$ grep -n "worktree add --detach\|update-ref refs/singlefs/gate-ok" \
    .claude/singlefs-ai-sop/scripts/gate.sh
58:  if ! staged_add_err="$(git -C "$src_root" worktree add --detach "$staged_tree" HEAD 2>&1)"; then
436:  git -C "$ROOT" update-ref refs/singlefs/gate-ok "$START_HEAD" 2>/dev/null || true
```
`--staged` 模式确实用 `git worktree add`；`update-ref refs/singlefs/gate-ok` 确实存在，且贴在
「已实现的门禁阶段全部通过」那句 `ok` 之前（第 436 行），与定义描述「共享 `gate.sh` 全绿时
`update-ref …`」一致。

**什么现象会推翻它**：`gate.sh` 源码里这两处 git 写操作被移除或改成别的语义。现查一致，没有。

### J2 规则冲突：没打中（且是一处「共用约束显式开例外」的正例）

`agent-common.md`「不做」一节写「不做 git 写操作（`add`、`commit`、`stash`、`checkout`、
`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见
它的定义」。`gate-triage.md`「写范围」一节写「门禁自己的 git 写（共享 `gate.sh` 全绿时
`update-ref refs/singlefs/gate-ok`、`--staged` 时的临时 worktree）是门禁本身的行为，允许；你自己
不做任何 git 写」。**两句并排**：

| 位置 | 原句 |
|---|---|
| `agent-common.md`「不做」 | 「不做 git 写操作（……`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。」 |
| `gate-triage.md`「写范围」 | 「门禁自己的 git 写（……`--staged` 时的临时 worktree）是门禁本身的行为，允许；你自己不做任何 git 写。」 |

两句不是矛盾，是共用约束**显式声明的一处例外**在 gate-triage 自己的定义里被精确复述：约束禁止的
是「gate-triage 自己发起的 git 写」，允许的是「它调用的 `gate.sh` 内部产生的 git 写」，
gate-triage.md 用「你自己不做任何 git 写」这句把这条界线重申了一遍，界线画在同一处。**判没打中。**

**什么现象会推翻它**：gate-triage.md 的「做什么」步骤里出现一处要求它**自己**（不通过
`gate.sh`）执行 `git worktree`/`update-ref`/`add`/`commit` 之类命令。核过全文没有。

### J5 分工：一致

`CLAUDE.md` 调度表「暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存
→ `gate-triage`」与 gate-triage.md「输入」一节「这一轮的改动已经按 `research/scripts/
stage-mine.py` 暂存了没有（没暂存就不派你……）」一致——都把「先用 `stage-mine.py` 暂存」放在派
`gate-triage` 之前。`stage-owners.tsv` 里 `gate-triage` 是 7 个阶段的 owner
（`41-pipefail-grepq.sh`、`45-script-modes.sh`、`46-write-hook.sh`、`50-rules-manifest.sh`、
`56-crates-adversarial-review.sh`、`62-stage-owners.sh`、`89-stage-selftest.sh`，另加
`63-agent-write-scope.sh`，共 8 个而不是「7 个」——见下面「六」节的精确计数），gate-triage.md 自
己的「做什么」一节没有专门去 `awk` 取这份清单再逐个跑（它跑的是 `gate.sh --staged`，即整道门禁一
次性覆盖全部阶段，包括表里登记给它的那几个），这与 `stage-owners.tsv` 表头注释「提交前的整轮门禁
（`gate-triage`）照旧跑全部阶段」一致，不是遗漏。

**什么现象会推翻它**：`stage-owners.tsv` 表头声称 gate-triage 该单独取自己的阶段来跑，而不是整轮
跑。现查表头原文就是「照旧跑全部阶段」，与 gate-triage.md 的行为一致。

（核实用的 `awk` 命令按 `agent-common.md`「门禁」一节给的原句跑：8 个阶段，与我上面数的一致，
「7 个」是我口误写的初稿数字，这里已经用实测的 8 改正——写在这里是提醒读者：这份报告里所有计数都
要求自己能用命令重新数出来，下面「六」的判定一览也用这条命令统一核过一遍。）

### J6 试跑读数：没有可判的对象

背景材料正文第二节明写：「`gate-triage` | 试跑与这一轮同时在跑，腿不读 | 真仓整轮门禁」——这一轮
的材料员没有给 gate-triage 单独生成 `agent-defs-r1-trial-gate-triage.md` 这样的试跑报告，而是说明
它的「试跑」就是这一整轮三方论证本身依赖的真仓门禁运行（由主 agent 或另一个并发在跑的会话执行），
并且明确要求各条腿不去读它（大概率是因为它此刻仍在跑、或者读了会与这一轮的证据链条混在一起）。
现查确认没有这样一份文件：

```
$ ls research/prompts/ | grep -i "trial-gate"
（无输出）
```

**判定：J6 在 gate-triage 上没有可判对象，不判。** 我没有违反「腿不读」这条限制去寻找或猜测这份
文件的内容。

**什么现象会推翻它**：如果之后出现了 `research/prompts/agent-defs-r1-trial-gate-triage.md` 这样的
文件、且其中指出的问题在今天的 `gate-triage.md` 里仍未修正，那时才有 J6 可判的对象。

## 五、`experiment-designer`

### J1 事实：一致

`.claude/agents/experiment-designer.md`（38 行）依据五处标题，逐一现查：

```
$ grep -n "^## 实验开跑之前，答案不许已经存在\|^## 实验的失败条款不许让结论不可证伪\|^## 阳性对照必须对\|^## 只让多条臂互相比\|^## 端点不是轨迹" \
    .claude/singlefs-ai-sop/rules/test-discipline.md
35:## 实验开跑之前，答案不许已经存在
51:## 实验的失败条款不许让结论不可证伪
93:## 阳性对照必须对**每一条**被测的臂都跑
104:## 只让多条臂互相比，测不出「所有臂一起错」
123:## 端点不是轨迹：被条款当谓词输入的量，实验要报轨迹
$ grep -n "^## 不许先有结论再建模型" .claude/singlefs-ai-sop/rules/evidence-discipline.md
108:## 不许先有结论再建模型
$ grep -n "^## 第六类：跑前写死的判决只在一个几何" .claude/rules/mutation-sampling.md
65:## 第六类：跑前写死的判决只在一个几何 / 旋钮取样点上量过
```

7 处标题全部命中且逐字与 experiment-designer.md 第 13 行引用一致。`research/scripts/
claim-experiment.sh --next` 与 `.claude/gate.d/86-experiment-orphans.sh` 都存在（我在开头就核实
过存在性），试跑报告里贴出的 86 号失败信息（「有 research 文件没有 kb 正文」）与
experiment-designer.md「产出」一节「登记落盘之后……门禁 86 号会判这个实验号『有 research 文件没
有 kb 正文』而红」逐字对应。

**什么现象会推翻它**：任一处标题在规则文件里核不到，或 86 号阶段的失败信息措辞与定义描述不同。
核过全部一致。

### J2 规则冲突：一处值得记录的张力，判「规则没说清楚」而非「打中」

experiment-designer 步骤 2「不读 `.claude/kb/experiments/` 下已有实验的结果与结论小节……全仓搜条
款与用词时加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts`」与它引用
`.claude/rules/mutation-sampling.md`「第五类：两个口径算同一个量，而它们在基线那一臂上恰好同值」
之间有一处张力（这条第五类不在 experiment-designer.md 的「依据」引用清单里，但试跑那次的执行者
自己判断它适用）。第五类原文最后一句判据是：

```
⇒ 判据：把一个数写进产物之前，问一句『这个量仓里是不是已经有人算过；如果算过，他用的分母是不是
我这个』。两个口径都在 ⇒ 不许自己挑一个……只在基线那一臂上对过账 ⇒ 等于没对。
```

这句判据字面要求「去查仓里是不是已经有人算过同一个量」，而 experiment-designer 步骤 2 把
`experiments/` 整个目录排除在搜索范围之外——**排除掉整个目录，连『别人是不是算过』这个事实本身
都查不到，而不只是查不到『算出的答案是什么』**。两句并排：

| 位置 | 原句 |
|---|---|
| `mutation-sampling.md` 第五类判据 | 「把一个数写进产物之前，问一句『这个量仓里是不是已经有人算过；如果算过，他用的分母是不是我这个』。」 |
| `experiment-designer.md` 步骤 2 | 「不读 `.claude/kb/experiments/` 下已有实验的结果与结论小节，不读 `research/results/`；全仓搜条款与用词时加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts`，免得命中行把结论带进来。」 |

**判定**：这不是一处两条规则字面矛盾（`mutation-sampling.md` 的第五类不是 experiment-designer.md
自己引用的依据，所以谈不上「定义引的依据不支持它这一步」），而是**依据清单本身没有覆盖到这条相
关规则**，导致按 `test-discipline.md`「实验开跑之前，答案不许已经存在」执行到底（连目录名都排除）
时，会顺带堵死 `mutation-sampling.md` 第五类要求的「查有没有人算过同一个量」这个动作。experiment-
designer 的试跑报告已经把这条张力自己指出来了（见下面 J6 第 3 点），当前定义文本对这条张力**仍
未给出出路**——步骤 2 现在的写法（整目录排除）比试跑时的做法（`grep -v '^./experiments/'` 这种坏
掉的过滤）更严格、更安全（避免读到结论），但没有恢复「查有没有人算过」这个动作本身。

**什么现象会推翻它**：如果能证明 experiment-designer 不需要做第五类那种「查有没有人算过同一个量」
的检查（比如这项检查被明确划给了 `experiment-runner` 或主 agent），那么这条张力就不成立。现查
`experiment-runner.md` 第 13 行「依据：……`.claude/rules/mutation-sampling.md`；……」——`experiment-
runner` 笼统引用了 `mutation-sampling.md` 整篇（不像 experiment-designer 那样只挑了「第六类」这
一节），且它步骤 3「没红的逐条按 `mutation-sampling.md` 分类」（第 25 行）明确提到用这份规则去分
类变异测试的结果，但那是对变异表跑完之后的「没红/无效」条目分类，与第五类「写一个数进产物之前，
查有没有人算过同一个量」针对的是**设计阶段**、发生在有任何变异表之前，两者时间点不同。也就是
说，`experiment-runner` 引用 `mutation-sampling.md` 覆盖的是它自己步骤里用得到的那部分（分类没红
的变异），而第五类那条「设计前查重」的检查，两份定义都没有显式点名谁该做。这是**规则没说清楚**，
不是某一句字面矛盾，我按
「规则没说」记录，不算 J2 打中。

### J5 分工：一致

`CLAUDE.md` 调度表「要建计数实验 | `experiment-designer` 写跑前登记 → `experiment-runner`」与
experiment-designer.md 的「产出」一节「登记落盘之后……由主 agent 尽快接派 `experiment-runner`」、
以及 `experiment-runner.md`「输入」一节「跑前登记路径（`research/prompts/e<号>-preregistration.
md`）」三处对同一条链路（先设计后执行）的描述一致。`stage-owners.tsv` 里没有 `experiment-designer`
作为任何阶段的 owner（`grep -c "experiment-designer" .claude/gate.d/stage-owners.tsv` → `0`），
experiment-designer 试跑报告自己核实过这一点（「登记给 experiment-designer 的阶段是零个」），与
它「做什么」一节不提门禁步骤一致。

**什么现象会推翻它**：`stage-owners.tsv` 出现 `experiment-designer` 作为某阶段 owner、而定义里没
有对应步骤，或者调度表与两处定义对「先设计后执行」的顺序描述不一致。现查三处一致。

### J6 试跑读数：13 点里，1 点已改，1 点部分改，其余 11 点仍是今天定义的字面缺口

试跑报告 `research/prompts/agent-defs-r1-trial-experiment-designer.md`「试跑观察」列了 13 点，是
我这一轮读到的最长一份试跑观察，逐点核对当前 `experiment-designer.md` 全文（38 行），结果与
sweep/prior-art/mutation-triage 明显不同：**这份定义在试跑之后几乎没有针对试跑观察改动**。

| # | 试跑观察要点（整行摘） | 现查当前定义文本 | 判定 |
|---|---|---|---|
| 1 | 「先占号（步骤 1）、后读条款判歧义（步骤 2），歧义在读完条款才发现，此时号已占、门禁 86 号已红……建议改顺序或写明空登记怎么处理」 | 步骤仍是「1. `claim-experiment.sh --next` 取号……2. 读被测条款……」，顺序未变，也没有加「交回时留下的空登记怎么处理」这句 | **仍是缺口** |
| 2 | 「『有两种读法先交回』没给『两种都报、零成本』这条出路，建议写明这种情形可以都登记」 | 「输入」一节仍是「有两种读法的先交回主 agent 定问法」，没有「零成本歧义可以都登记」这句 | **仍是缺口** |
| 3 | 「第 2 步『不读实验结果』与 mutation-sampling 第五类『查分母』直接冲突，全仓 grep 排除写法给错」 | 步骤 2 已经加上了 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts`，修好了「排除写法给错」这一半；但没有恢复「查有没有人算过同一个量」这个动作本身（见上面 J2） | **部分已改**（排除语法修好，功能性张力见 J2） |
| 4 | 「『答案不许已经存在』在纯算术题上做不到，需要一节『跑之前已经存在的数』」 | 步骤 3 的登记项清单里没有这一节；只在描述里说「读过的每个文件列进登记；还是读到了已有的数……照实列进登记，不删」——这句**部分**覆盖了「不删除已知道的数」，但没有像试跑那样单独设一节 | **部分已改**（有一句兜底，但没有独立小节） |
| 5 | 「臂、阳性对照、门槛……这张清单大半没有对象（纯算术、没有判决、没有臂），建议写『纯算术/没有判决的问题这几格怎么写』」 | 步骤 3 仍然是完整清单（「每条臂的定义……阳性对照……判据与门槛……钉绝对值的断言……几何敏感性……失败条款……作废条款」），没有给出「不适用时怎么写」的分支 | **仍是缺口** |
| 6 | 「纯算术里最现成的锚点是被测条款自己的数，这与『条款可能错』的失败条款打架，建议定义写明这个分法（条款内 vs 条款外的锚点）」 | 未提及 | **仍是缺口** |
| 7 | 「`implementation-first.md` 第 4 条『两边都查，不许默认哪边对』——定义没提『停机条款』这第三种状态」 | 「依据」引用了 `implementation-first.md`（整篇），但「做什么」步骤 3 的清单里没有「停机条款」这一项 | **仍是缺口** |
| 8 | 「grep 命中行算不算『读过』没说清楚」 | 步骤 2「读过的每个文件列进登记」没有说明 grep 命中行是否算「读过」 | **仍是缺口** |
| 9 | 「登记没有模板，节名、编号都是试跑者自己起的」 | 定义仍未提供登记模板或固定节名 | **仍是缺口** |
| 10 | 「`agent-common`『报告』要求写文件，本次会话系统说明说『不要写报告 .md』——按派发提示与 `agent-common` 做了，建议派发提示说明以哪条为准」 | 这是**派发提示**（主 agent 现写）层面的问题，不是持久定义文本问题 | **不适用于定义本身** |
| 11 | 「`quote-kb.py` 单行要写 `A-A`；`replace-batch.py --help` 报 traceback；工具只有 Read/Bash」 | 前两条是脚本本身的小毛病（不在这三个 agent 定义的射程内，是 `research/scripts/` 维护范围）；第三条「不用动」是试跑报告自己的结论 | **不适用**（脚本 bug，非定义冲突；已现查 `quote-kb.py --help` 确认单行确需 `A-A` 格式） |
| 12 | 「变异表草拟没有人核是不是等价变异，建议定义写明『设计员要逐条说它在哪个取样点上改变输出』」 | 步骤 3 清单没有这句要求 | **仍是缺口** |
| 13 | 「定义要求读 crates/，很有用，保留」 | 步骤 2「读 `crates/` 里对应的实现」仍在，且是正面评价 | 不适用（无需改） |

**判定：J6 在 experiment-designer 上，13 点观察里只有第 3、4 两点得到部分回应，其余 9 点（1、2、
5、6、7、8、9、12，以及仍不完整的 3、4 剩余部分）在今天的定义文本里逐字仍是试跑当时的样子。** 与
sweep（2 点全改）、prior-art（3 点已改、1 点有反证）、mutation-triage（1 点全改、1 点部分改）相
比，experiment-designer 的试跑观察是 5 个对象里篇幅最长、最具体，但被吸收进定义的比例最低。这本
身是一条值得报给主 agent 的分工/优先级观察：与其说是某一点「打中」了 experiment-designer 定义的
错误，不如说是**这份定义还没有走完与其它 4 份定义相同的「试跑 → 改定义」这一步**。

**什么现象会推翻它**：如果主 agent 能说明这 9 点里的某几点是**故意**不改（比如判断它们优先级低、
或者判断按现有文本已经够用），那么它们就不算「还没改」而是「已判断不用改」，需要在定义文件里或
别处留一句依据；现查定义文件本身和 `records/2026-09-16-subagent拆分提案.md` 都没有这样的说明。

## 六、`CLAUDE.md`「什么时候派哪个 agent」与 `.claude/gate.d/stage-owners.tsv` 对全部 16 个定义

### J1 事实：一致

背景材料引用的 `CLAUDE.md:8-23` 与仓里现文件逐字比对：

```
$ sed -n '8,23p' CLAUDE.md
```
输出与背景材料附录第 1961–1980 行「出处 `CLAUDE.md:8-23`」引用块逐字相同（已在开头「零」核对，
不重复贴）。`stage-owners.tsv` 全文与背景材料「出处 `.claude/gate.d/stage-owners.tsv:1-57`」引用
块逐字相同（已在读取阶段核对，两者字节相同）。`bash .claude/gate.d/62-stage-owners.sh` 现跑：

```
$ nice -n 19 bash .claude/gate.d/62-stage-owners.sh
  ✓ 阶段归属表与门禁目录一致（54 个阶段，归 7 个 agent）
```

54 个阶段（`.claude/gate.d/[0-9][0-9]-*.sh` 文件数）与表里登记的行数一致，7 个 agent 作为 owner
出现（`kb-scribe`、`experiment-runner`、`gate-triage`、`crash-verifier`、`three-way-materials`、
`implementation-writer`、`prior-art`）——这 7 个都在 `CLAUDE.md` 调度表里出现过，`CLAUDE.md` 调度
表里提到的全部 16 个 agent 名字，在 `.claude/agents/` 下都能找到同名 `.md`：

```
$ sed -n '8,23p' CLAUDE.md | grep -oE '`[a-z][a-z0-9-]*`' | tr -d '`' | sort -u
crash-verifier
experiment-designer
experiment-runner
gate-triage
implementation-writer
kb-scribe
mutation-triage
prior-art
sweep
three-way-attack
three-way-defense
three-way-forward
three-way-local-attack
three-way-local-defense
three-way-materials
three-way-verifier
$ ls .claude/agents/*.md | xargs -n1 basename | sed 's/\.md$//' | sort
crash-verifier
experiment-designer
experiment-runner
gate-triage
implementation-writer
kb-scribe
mutation-triage
prior-art
sweep
three-way-attack
three-way-defense
three-way-forward
three-way-local-attack
three-way-local-defense
three-way-materials
three-way-verifier
```
两份清单逐行相同：调度表这一节（第 8–23 行）反引号里出现的 16 个名字，与 `.claude/agents/` 目录
下的 16 个定义文件名（去掉 `.md`）逐一对上，一个不多一个不少。

**什么现象会推翻「一致」这个判断**：`CLAUDE.md` 调度表、`stage-owners.tsv`、`.claude/agents/`
目录三者中出现了名字对不上的情况（表里提到的 agent 在目录里没有定义，或反过来）。62 号门禁阶段
的判据正是这个，现跑绿。

### J5 分工：一致，附三条「规则没说」的观察

逐条把 `CLAUDE.md` 调度表的 9 行与 `stage-owners.tsv` 的 54 行、以及 16 个定义各自的
`description`/「输入」/「做什么」/「门禁」表述交叉核对（我这一轮读过全部 16 个定义的正文——5 个
是我自己的判据对象，另外 11 个是背景材料整段抄进来的附录，按「引 kb 里的条目要整行抄」的同一份
材料读的，不是转述），结论：**没有发现调度表、归属表、定义三处对『该谁跑、谁先谁后、谁写哪个文
件』说法互相矛盾的情形**。

三条值得记录的观察（不是冲突，是「规则没说清楚」或「设计留白」）：

1. **`CLAUDE.md` 调度表没有给 `crash-verifier` 单开一行**，它只出现在两条「改 `crates/`」流水线
   的末尾。`stage-owners.tsv` 里 `crash-verifier` 拥有 4 个阶段（`54-layer0-replay.sh`、
   `55-qemu-first-transaction.sh`、`57-lkmm.sh`、`59-crates-mutation-replay.sh`）。如果主 agent
   想在**不经过 `implementation-writer`** 的情况下单独重跑这四道重阶段（比如只是想确认某次改动
   后层 0 还是绿的，没有新代码要写），调度表没有一行直接指向这个用法。这不是矛盾，是「规则没
   说」——`crash-verifier.md` 自己的 `description` 写着「只在主 agent 点名派发、并给出这次改动的
   范围时用；不要自动派发」，允许主 agent 随时点名派它，调度表只是没有专门写一行覆盖这个用法。
2. **`stage-owners.tsv` 里 4 个阶段有两个 owner**（`27-format-constants.sh`→
   `kb-scribe,experiment-runner`；`52-segment-registry.sh`→`kb-scribe,experiment-runner`；
   `53-format-const-placeholders.sh`→`implementation-writer,kb-scribe`；`70-citations.sh`→
   `kb-scribe,prior-art`），`CLAUDE.md` 调度表没有说明「两个 owner 谁先跑」。62 号门禁阶段的判
   据只要求「第二列每个 agent 名都有定义」，不判「多 owner 时的先后顺序」。这是一处**规则没说**：
   如果 `kb-scribe` 与 `experiment-runner` 同一轮各自完工，谁该先跑 `27-format-constants.sh`，
   目前没有文字规定，`agent-common.md`「门禁」一节写的是「哪个门禁阶段该由谁在干完自己的活之后
   先跑」——多个 owner 时这句本身允许「谁先干完自己的活谁先跑」这种隐含读法，但没有明说。
3. `stage-owners.tsv` 第 50 行（`70-citations.sh` 那一行）「查源码树的是调研员」与
   `prior-art.md` 步骤 0 一致（已在「二」核对过），但 `CLAUDE.md` 调度表没有把 `70-citations.sh`
   与 `prior-art` 的调用关系写进「要别家文件
   系统的事实 | `prior-art`」这一行——这不是冲突（那一行本来就是简表，不是逐门禁阶段的映射），
   只是提醒：想从 `CLAUDE.md` 单独一张表看出「prior-art 什么时候该跑 70 号」，看不出来，要跳到
   `stage-owners.tsv` 或 `prior-art.md` 才看得到。

**什么现象会推翻「一致」这个判断**：能找到调度表某一行与 `stage-owners.tsv` 或某个定义的「谁先
跑」「谁写哪个文件」字面矛盾的一句话。逐条核过 54 行 + 9 行 + 16 份定义，没有找到这种矛盾，只有
上面三条「规则没说清楚」的留白。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| sweep / J1 | 一致 | 依据引用的 3 个规则标题、`27-format-constants.sh` 都核实存在且行为相符 |
| sweep / J2 | 没打中 | 「只找不改」与被引用规则的动作主语不冲突 |
| sweep / J5 | 一致 | 调度表触发条件与 `description` 逐字对应；`stage-owners.tsv` 里没有它，与它不提门禁步骤一致 |
| sweep / J6 | 没打中 | 试跑观察 5 点里 2 点可操作的已写进步骤 2、4，其余 3 点本就路由给人看 |
| prior-art / J1 | 一致（1 处复核不了） | 依据标题都在；`WebFetch`/`WebSearch` 是否真给到这个 agent_type 需要主 agent 实测 |
| prior-art / J2 | 没打中 | 步骤 3 与被引用规则的「可以引/不可以引」两栏不矛盾 |
| prior-art / J5 | 一致 | 调度表、`stage-owners.tsv` 第 50 行、定义步骤 0 三处对 70 号阶段归属一致 |
| prior-art / J6 | 大部分没打中 | 6 点观察：1 点仍是旧文本但有「开工先读 agent-common.md」这条结构性反证；3、4 两点已改；2、5、6 三点不适用或不构成打中 |
| mutation-triage / J1 | 一致 | 依据标题、`Cargo.toml` 全部 100 个 bin 名连字符的断言、`mutate.sh` 备份还原机制都核实 |
| mutation-triage / J2 | 没打中 | 「不改源码」与 `mutate.sh` 自带备份/还原/trap 机制不矛盾 |
| mutation-triage / J5 | 一致 | 调度表、`stage-owners.tsv`（它无阶段）、`crash-verifier` 的交叉授权都是分工不重叠的正常设计 |
| mutation-triage / J6 | 部分打中 | 3 点观察：第 1 点（最关键）已写进「输入」一节；第 2 点字面仍缺但触发条件已被第 1 点削弱；第 3 点不适用 |
| gate-triage / J1 | 一致 | 依据两个标题、`gate.sh` 的 worktree 与 `update-ref` 行为都核实 |
| gate-triage / J2 | 没打中 | 「写范围」与 `agent-common.md`「不做」一节的例外条款逐字对应，是共用约束显式开例外的正例 |
| gate-triage / J5 | 一致 | 「先暂存后派」的顺序在调度表与定义「输入」一节一致；8 个阶段的归属与「照旧跑全部阶段」的设计一致 |
| gate-triage / J6 | 不适用 | 没有独立试跑报告，材料明写「腿不读」，本轮无可判对象 |
| experiment-designer / J1 | 一致 | 7 处依据标题、`claim-experiment.sh`、86 号阶段失败信息都核实 |
| experiment-designer / J2 | 规则没说清楚（非打中） | 步骤 2 的目录排除与 `mutation-sampling.md` 第五类「查有没有人算过」之间有功能性张力，但第五类不在它自己引用的依据清单里，是依据清单没覆盖，不是引用的依据与步骤互相矛盾 |
| experiment-designer / J5 | 一致 | 「设计 → 执行」链路在调度表、两份定义、`stage-owners.tsv`（它无阶段）四处一致 |
| experiment-designer / J6 | 大部分打中 | 13 点观察里 9 点（1、2、5、6、7、8、9、12 及 3、4 的剩余部分）今天的定义文本仍是试跑当时的样子，只有 3、4 两点部分回应；这是 5 个对象里试跑吸收率明显最低的一个 |
| CLAUDE.md/stage-owners.tsv 对 16 个定义 / J1 | 一致 | 62 号门禁绿；调度表与目录里的 agent 名字集合相同（含我自己先漏抓 `three-way-verifier` 又更正的一处自证） |
| CLAUDE.md/stage-owners.tsv 对 16 个定义 / J5 | 一致，附 3 条留白 | 没找到「该谁跑、谁先谁后、谁写哪个文件」的字面矛盾；`crash-verifier` 未单独成行、多 owner 阶段的先后顺序、70 号与 prior-art 的映射没写进调度表这三处是设计留白，不是冲突 |

## 没做什么

- **不判 J3、J4**：这两格分给云端攻方腿（`three-way-attack`），我没有去构造「照定义做会卡住」或
  「照定义做会出错的可达操作序列」这类情形，只在 J2 里顺带记录了一处「规则没说清楚」的功能性张
  力（experiment-designer 步骤 2 与 mutation-sampling 第五类），那是判据格 J2 本身要求的「定义点
  名的依据原文并不支持它那一步」的边界情况，不是我主动去找 J3/J4 那种可达序列。
- **不判别的 11 个定义**（`three-way-forward`〔我自己〕、`three-way-attack`、`three-way-defense`、
  `three-way-local-attack`、`three-way-local-defense`、`three-way-materials`、`three-way-
  verifier`、`implementation-writer`、`kb-scribe`、`experiment-runner`、`crash-verifier`）的
  J1/J2/J6：这些不在我的对象名单里，只在「六」核对 `CLAUDE.md`/`stage-owners.tsv` 与全部 16 个定
  义的一致性时，读过它们的正文（背景材料附录整段抄的），但没有对它们各自的 J1/J2/J6 单独下判
  断。
- **不替主 agent 采纳或出判决**：判定一览只是我这条腿的报告，改不改定义、按 K1/K2/K3 哪一条条款
  处理，由主 agent 决定。
- **`WebFetch`/`WebSearch` 对 `prior-art` 是否真的可派发**：复核不了，需要主 agent 实际派一次
  `prior-art` 才能验证，见「二」J1 小节。
- **mutation-triage 步骤 4 在「退出码 6 之外的其它提前失败」场景下会不会误判**：没有构造这类场景
  去验证，见「三」J6 小节末尾。
- **experiment-designer 定义为什么没有像另外 3 个对象一样吸收试跑观察**：我只报告了这个现象（对
  比表），没有去问为什么——这需要问设计/维护这份定义的人（可能是优先级排序，也可能是遗漏），
  不是我这条腿能查出来的。
- **没有跑 `gate.sh` 全量**：只单独跑了 `27-format-constants.sh` 与 `62-stage-owners.sh` 两个门
  禁阶段（都不在 `stage-owners.tsv` 登记给 `three-way-forward` 的阶段里，因为 `three-way-
  forward` 本身不拥有任何阶段——这与 `three-way-attack`/`three-way-defense`/`three-way-verifier`
  等三方论证角色一致，见「六」）；跑这两个阶段是为了核实 J1 事实，不是走门禁流程。

## 试跑观察（我这个定义 `three-way-forward` 本身哪些步骤不清楚、做不下去）

这一轮同时也是 `three-way-forward` 定义自己的试跑，按派发提示的要求另记一节。

1. **「四、这一轮的路径与禁读」里的禁读清单只精确到文件名模式，没有说清『我自己的产出目录』要不
   要在写之前先确认它不在别的腿的目录集合里。** 我的模型目录形态是
   `research/prompts/agent-defs-r1-sonnet-model/`，草稿目录是
   `/tmp/claude-1000/agent-defs-r1-sonnet/`——这两个路径本身没有和禁读清单冲突，但定义正文（
   `three-way-forward.md`）「写范围」一节只写「报告文件、模型目录、草稿目录」，没有像
   `three-way-attack.md`「写范围」那样加一句「除此之外不写」的显式收尾（其实 `three-way-
   forward.md` 也有「除此之外不写」，是我一开始读的时候没有立刻意识到这句话同时管着「不越权写
   模型目录之外的任何路径」——这次没有撞到实际冲突，只是定义文本本身在「模型目录」这件事上比较
   简略：它没有说「模型目录」在这一轮里我实际上没有用到（这次判定不需要建模型，只需要读、跑命
   令、写报告），是不是可以完全不建这个目录。我的处理是没有创建 `research/prompts/agent-defs-r1-
   sonnet-model/`（用不上就空着，这点 `agent-common.md`「草稿放派发提示给的草稿目录……用不上可以
   空着」只提到草稿目录，没有提到模型目录用不上时怎么办，我按同样的逻辑处理，但定义本身没有明
   说）。
2. **「做什么」步骤 3「能用命令核的事实复跑并贴原样输出」这句话，实测下来最容易在草稿阶段被
   违反成「先凭印象写出『像』命令输出的文本，回头再核」，而定义本身没有一步要求「写完先自己核一
   遍才交」。** 我这次的草稿第一版里，`.claude/rules/format-evolution.md`、`evidence-discipline.
   md`、`test-discipline.md`、`mutation-sampling.md`、`session-wrapup.md`、`.claude/gate.d/
   stage-owners.tsv`、`.claude/singlefs-ai-sop/scripts/gate.sh` 等多处引用的**行号是我凭读过的印
   象编的**，不是真跑 `grep -n` 得到的；`stage-owners.tsv` 里两行的「第 N 行」还把**门禁阶段编号**
   （70、59）误当成了**表里的行号**用。写完之后我把草稿里每一条 `$ ` 开头的命令重新跑了一遍、把
   每一个「第 N 行」重新 `sed -n`/`grep -n` 核了一遍，改正了十几处（本文里能看到的「70 是门禁阶段
   编号，不是行号」这类自我更正，就是这一遍核对的产物）。**这条纪律本身写在 `agent-common.md`
   「报告」一节（「能用命令核的事实，贴命令与原样输出」）与派发提示里，不是 `three-way-forward.md`
   缺了这句话**——缺的是「写完交回之前，把自己写的每一条命令输出重新跑一遍」这一步骤性的要求；
   现在这一步完全靠个人自觉。**建议**：`agent-common.md`「报告」一节加一句「交回前把自己写的每
   一条命令输出、每一个行号重新核一遍，编造的命令输出比没有证据更危险，因为它看起来是证据」。
3. **判据格之间的边界在「规则没说」与「J2 打中」之间不总是清楚。** 我在 experiment-designer 那节
   遇到一处（步骤 2 的目录排除 vs `mutation-sampling.md` 第五类）：这条张力算不算「J2 冲突」，取
   决于 `mutation-sampling.md` 第五类是不是 experiment-designer.md **自己**引用的依据——如果算
   J2 打中，判据是「定义点名的『依据』原文并不支持它那一步」，而这条规则根本没被定义点名，所以
   严格按字面判据我应该记「不适用」而不是「规则没说」。`three-way-forward.md`「做什么」步骤 2
   给的三个格是「一致 / 冲突 / 规则没说」，没有第四个格给「一条相关但没被定义引用的规则」这种情
   形。我最终选择把它记在 J2 小节里、标成「规则没说清楚（非打中）」，是我自己在三个格之外临时开
   的一种表述，**这不是定义要求的标准格式**，交主 agent 判断这样处理是否合适。
4. **背景材料体量很大（2435 行），「附录不够时读 kb 原文与 crates/ 源码」这句在『判断附录够不
   够』上没有更具体的判据。** 这次背景材料的附录已经把 16 个定义、`agent-common.md`、调度表、
   `stage-owners.tsv`、两份 hook 文件、两个门禁脚本全部整段抄出来了，我读完之后仍然去读了仓里的
   现文件（而不是只用附录）——这条提示实际来自**这一轮的派发提示**（不是背景材料本身）：派发提
   示「注意」一节写着「定义与共用约束在派这一轮之前刚按试跑改过，以仓里现在的文件为准（附录是材
   料员抽取那一刻的副本，对不上时读仓里的现文件并写明）」，这句提示已经足够指导我该怎么做，只是
   它出现在派发提示里而不是 `three-way-forward.md` 自己的定义里；`three-way-forward.md` 本身如果
   要独立于这一轮的派发提示使用（比如以后某一轮派发提示没写这句），「附录不够时读 kb 原文」这句
   还需要补一句「即使附录看着齐
   全，涉及『现状是什么』的判断也要用仓里现文件复核，不能只信附录」，否则容易漏掉像本轮「CLAUDE.
   md 系统提示注入版本已过时」这种只有对照现文件才能发现的情形。
