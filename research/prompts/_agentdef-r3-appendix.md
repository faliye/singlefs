**出处 `research/prompts/agentdef-r1-main-verification.md:20-36`（整段抄，未转述）**

```markdown
## 三、`.claude/main-agent.md` 那一处：攻方 6 格打中 5 格，改法留到第二轮

云端攻方对「逐行全看」这条规则本身打中五格，主 agent 逐格现查：

| 格 | 攻方的结论 | 主 agent 现查 |
|---|---|---|
| C 判别子够不够 | 触发这次改动的那处判错（`.claude/kb/decisions/06-快照实现模型.md:42`），判别子在门禁 94 号的射程里，载体那一行一个字都看不出来 | **坐实，而且最重**：判它该不该改的那一刻，载体原文写的是「欠：C12（增量语义共用）：② 的引用计数旁表…」，要知道 94 号罩不罩得到那一格，得读 94 号的实现。**规则措辞「对着载体今天的原文核一遍」治不了它自己的触发案例** |
| F 立得合不合项目自己的判据 | 改前那句可机械核，改后这句不能 | **坐实**：改前「随机抽 20 行（五种判定各至少 2 行）」里「20 行」「各至少 2 行」是可数的；改后「逐行全看」没有任何可数的东西，撞 `.claude/singlefs-ai-sop/rules/sop-first.md` 的「边界」一节「一条规则想进来，先问它能不能变成一个会失败的检查」 |
| E 两条文本并存 | 共享规则写「在全部判定种类里抽查复判」，本地写「不抽样、不按判定种类挑」，两种读法各自合规 | **坐实**：两句并排字面互斥，而 `.claude/singlefs-ai-sop/` 是 rsync 拷贝，本地改不掉共享那句 |
| B 退化成什么 | 旧条款「五种判定各至少 2 行」对最稀那档是写死的 100% 下限，新条款没有下限 | 坐实（判定分布来自攻方的量尺，主 agent 没有独立复跑它的计数） |
| D 分不分得出「全看了」与「只看判定词」 | 无门禁判这一步 | 坐实：仓里确实没有这样一道检查 |
| A 上下文装不装得下 | 字面装得下（138 行原文 53941 字符），够重判的深度装不下（123 个位置各 ±20 行 ≈ 载体全文的 95.3%） | 不独立复跑，记攻方的数并标明出处 |

**判定：这一处不在这一轮定。** 打中的五格里，C 与 F 合起来说的是同一件事——这条规则既说不清判别子是什么，也落不成会失败的检查。攻方给了四个改法，**四个都只在它自己的量尺上量过、被攻过零轮**（`.claude/rules/three-way-inference.md`：攻方腿自己提的收严算没被攻过）。按那条规则，零轮的改法可以写回、但要标明零轮并为它自己的检查记一笔账；而这里是一条**已经写进文件、还没提交**的规则改动，比「写回一条收严」更重，值得为它开第二轮，不在这一轮草率定。

**这一轮的处置**：`.claude/main-agent.md` 那一处**留在工作区、不进这一批提交**，连同攻方的四个改法一起进第二轮。第二轮的攻击面写死在这里，免得下一轮重新摸索：改法一（全看加按判定种类各抽 ≥2 行、抽到哪几行写进同步记录）、改法四（把判别子从「载体今天的原文」改写成「这条事实的新说法与它的出处」）各修不同的格，改法二（`research/scripts/stale-candidates.py:284` 那个 `[:500]` 截断）与改法三（只改共享规则那句、归发版会话）各只修一格；**四个改法都要先回答 F 格——改完之后这条规则能不能落成一个会失败的检查**。

```

**出处 `research/prompts/agentdef-r2-main-verification.md:1-6`（整段抄，未转述）**

```markdown
# agentdef-r2 第二轮判决

日期：2026-09-21。被判的是 **`.claude/main-agent.md`** 第 43 行那句「逐行全看」，以及第一轮攻方给的四个改法。门禁 72 号要这份判决按路径点名改过的定义，点名在这里：**`.claude/main-agent.md`**。

第一轮判决 `research/prompts/agentdef-r1-main-verification.md`。这一轮三条腿：云端攻方 `research/prompts/agentdef-r2-opus-output.md`（攻四个改法本身）、云端辩方 `research/prompts/agentdef-r2-sonnet-output.md`（复核第一轮判决）、本地攻方 `research/prompts/agentdef-r2-local-attack.md` 与两份样本。背景材料 `research/prompts/_agentdef-r2-background.md`。

```

**出处 `research/prompts/agentdef-r2-main-verification.md:7-21`（整段抄，未转述）**

```markdown
## 一、四个改法：一个都不采纳

攻方腿逐个攻过，主 agent 逐条现查：

| 改法 | 攻方的结论 | 主 agent 现查 |
|---|---|---|
| 一（全看 + 各抽 ≥2 行 + 抽样表） | B 修得掉、D 只修一半、**F 不修而且更坏** | **坐实，这是最硬的一击**：攻方写了 `forge.py`，**不读一行载体、不重判一行，全部从两份已入库的判定报告里机械抄出一张完全合规的抽样表**——每档抽够 2 行、每行载体都在候选表里、每行末格非空且 ≥8 字。那张表核不了「全看」，只核得了「有没有交一张表」。本地攻方两份样本独立给出同一结论：抽样记录表只能证明被抽中的那几行被重判过，证不了表里所有行都被看过 |
| 二（把 `stale-candidates.py` 的 `[:500]` 调大或报截断行数） | 只修 C 的一个侧面，**而且第一轮那个「量过」的数符号反了** | 坐实。第一轮改法表里这一格写「省 0 字符（量过）」，第二轮实测是**多 19344 字符**：截断的 41 行今天全文 39844 字符、截断版 20500。主 agent 并排现查两份报告确认。⇒ 这正是 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算没被攻过」的一个实例——第一轮标着「量过」的那个数，第二轮一量就翻了 |
| 三（只改共享规则那句） | 修不全它声称修的格 | 坐实，而且它归发版会话，本轮定不了 |
| 四（判别子改写成「新说法与它的出处」） | **拿不到它括注里定义的那个东西** | **坐实**：改法四要求「出处是哪个提交、哪道门禁的哪一条判据」，而候选表里没有这一列——主 agent 判一行判定时拿不到。它真正起作用的那一半是「新说法」，与「出处」无关 |

**四个改法两两之间还互相抵消**（攻方第四格）：改法一与改法三互相抵消、改法二与改法四互相取消。

**判定：四个改法一个都不采纳。**

```

**出处 `research/prompts/agentdef-r2-main-verification.md:22-30`（整段抄，未转述）**

```markdown
## 二、第一轮的判决，辩方复核之后改两格

辩方腿逐格复核第一轮的五格，主 agent 现查：

- **C 格：辩方给的证据比第一轮原判更强，原判的方向不变。** 辩方现查发现，`.claude/kb/decisions/06-快照实现模型.md:42` 那段解释门禁 94 号射程的文字，与 `.claude/kb/checks-owed.md` 里 C12（增量语义共用） 那一行今天写的 crate 粒度描述，**是同一次提交 `11a551b` 一起新写出来的**。主 agent 用 `git log -S` 逐串现查，两处都只出自 `11a551b`。也就是说：**那次判错发生的时刻，两处都不存在**——不但载体那一行看不出来，当时连欠账行也看不出来。辩方提的那条「把载体放宽到它链接的欠账行」的替代读法，救不了 C 格。
- **F 格：第一轮的推理那一步够不着，改判。** 辩方指出 `.claude/singlefs-ai-sop/rules/sop-first.md` 的「## 边界」原话是「**先问**它能不能变成一个会失败的检查。变不成的**多半**是还没想清楚」——「先问」「多半」，不是一票否决；而 `.claude/singlefs-ai-sop/rules/show-me-test.md` 的「## 推论」第 1、2 条（门禁绿不等于可以不看代码；语义对不对靠人）本身就是「落不成检查却写成正式规则」的既有先例。主 agent 逐字现查两处原文，**采纳这一格的改判**：F 格的事实描述（改前可数、改后不可数）站得住，但「按这套规范落不成检查就一定不该立」这一步推不出来。⚠️ 同时要写明辩方没说全的那半句：「边界」那一节下文是「**先留在 kb 里当待议，别写成规矩**」——第一轮的处置（挂起、不当定案）与这句一致。
- **B 格：弱。** 辩方判「够不着（弱）」，理由是判定分布的数字来自攻方量尺，第一轮判决自己承认没独立复跑。采纳：B 格降为「攻方的量尺如此，未独立复核」。
- **D、E 两格：站得住。** 辩方独立现查确认仓里确实没有判「全看了 vs 只看判定词」的门禁；E 格两句原文并排互斥。

```

**出处 `research/prompts/agentdef-r2-main-verification.md:31-36`（整段抄，未转述）**

```markdown
## 三、辩方的辩护：部分站住，不足以让那条规则当场定案

辩方给了一条不改措辞、不加检查的辩法：「逐行全看」管的是**覆盖**（不许抽样漏行），判定质量由 `.claude/singlefs-ai-sop/rules/verify-before-claiming.md`「陈述外部状态之前，现查一次」兜底；那次判错的真正病灶是抽样漏选，不是载体文字不够。

**主 agent 判：这条辩护本身成立，但它指出的配套关系没有写在规则里。** 辩方自己也如实点了这个弱点——按 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 第 1 条「正文只写四样」，没写出来的配套关系不能算数。

```

**出处 `research/prompts/agentdef-r2-main-verification.md:37-48`（整段抄，未转述）**

```markdown
## 四、定案

**`.claude/main-agent.md:43` 那句「逐行全看」留着，一个字不改。** 理由三条：

1. 四个改法一个都不采纳（第一节），没有比它更好的写法摆在桌上。
2. F 格改判之后，「它落不成会失败的检查」不再构成「不该立」的理由——这套规范里本来就有落不成检查而照样写成规则的先例。
3. 它治不了自己的触发案例（C 格）这一条仍然成立，但辩方指出的病灶分析是对的：那次判错的病灶是**抽样漏选**，而「逐行全看」正好治这一条。它治不了的是另一件事——判别子在别处。

**同时立一笔新账**，记攻方这一轮报的 G 格：**判错的原始证据不落盘**。触发这一切的那一处判错（`.claude/kb/decisions/06-快照实现模型.md:42`），当时的候选表与逐行判定报告都在 `/tmp`、一个字节都没入库；改法一的抽样表按定义也罩不到它——06:42 正是抽样没抽到的那一行。

⚠️ **攻方要主 agent 现查坐实的那一条，坐实之后反过来把这一格推得更远**：攻方说「06:42 曾被判错」它没有直接证据。主 agent 现查 `git show 11a551b:research/prompts/owed-batch1to3-sync-judge-f1-f11.md`，那一行入库的判定是**正确的「要改」**，报告里没有任何「曾判错」的痕迹；「判错过」这件事今天只活在一句自述里（`research/prompts/owed-batch1to3-sync.md:51`「抽样这一次漏掉了 06:42 那一处判错」）。⇒ **第一轮 C 格与这一轮 G 格都建在那句自述上**，而写那句自述的人正是当时判错的人。这不是推翻 C 格——C 格说的「判别子在门禁 94 号的射程里、载体那一行看不出来」是对今天两处原文的现查，与有没有判错过无关；但它把 G 格从「证据不好查」推到「**连事件本身都只有当事人的一面之词**」。

```

**出处 `research/prompts/agentdef-r2-main-verification.md:49-59`（整段抄，未转述）**

```markdown
## 五、这一轮的零轮形态

按 `.claude/rules/three-way-inference.md`「三轮之后停」，第二轮之后不再开第三轮。这一轮新冒出、被攻过零轮的形态，逐条写明：

| 形态 | 谁提的 | 零轮在哪 |
|---|---|---|
| 辩方那条辩护（全看管覆盖、判定质量由 `verify-before-claiming.md` 兜底） | 云端辩方 | 它自己提出、自己指出弱点，没有第三条腿攻过它 |
| F 格改判（落不成检查不等于不该立） | 云端辩方 | 主 agent 逐字现查了两处原文并采纳，但这个改判本身没被攻过 |
| G 格（判错的原始证据不落盘） | 云端攻方 | 攻方自己报的新格，没有腿攻过它 |
| 「复核改判」表（同步记录里另加一张：载体、原判定、改判后的判定、抓到它靠的是哪一件东西；0 处判错的轮可以是 0 行表） | 云端攻方 | 攻方在报告末尾自己提的收严，与第一轮那四个改法同级、同样没被攻过。⚠️ 它比那四个改法多一样东西：抽样抽不到的行上的改判**只可能来自全看**，那是「全看」独有的产物，改法一的抽样表给不了。这一条值得下一次开工时先攻一轮，不在这一轮采纳 |

```

**出处 `research/prompts/agentdef-r2-main-verification.md:60-65`（整段抄，未转述）**

```markdown
## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| 不涉及决策：这一轮判的是 `.claude/main-agent.md` 一条调度规则与四个改法，一条决策分项都没碰 | — | 2026-09-21 |

```

**出处 `research/prompts/agentdef-r2-opus-output.md:6-49`（整段抄，未转述）**

````markdown
## 复跑

这一份报告里全部的数，一条命令重出（在仓根跑）：

```
bash /tmp/claude-1000/agentdef-r2-opus/measure.sh
```

| 文件 | sha256 | 说明 |
|---|---|---|
| `/tmp/claude-1000/agentdef-r2-opus/measure.sh` | `c395f4e4c89c627de84708f4c0a2711ebabaf468aaf253c6f8de60ff7be5f06b` | 这条腿的量尺，草稿，没入库 |
| `/tmp/claude-1000/agentdef-r2-opus/cost.py` | `f42f28dee489dd8bf30043613c3afc1e5ffcfeac90333729ab6f1768dd5276a5` | 量改法二的体量代价，`measure.sh` 调它 |
| `/tmp/claude-1000/agentdef-r2-opus/forge.py` | `0e2bd5acce39090ff166803cead9a3484faa3f8628c2057ee8bca0b5f9f311d3` | 造那张伪「复判抽样」表，`measure.sh` 调它 |
| `/tmp/claude-1000/owed-sync/facts.tsv` | `339706ff7bd2ff7e7481cf6127ff14774e6e34cda12612cb3fb6db99fd2ab3b2` | owed-batch1to3 那一轮的 21 行事实表，上一轮别的会话留在草稿目录里的，不是这一轮跑出来的 |
| `/tmp/claude-1000/owed-sync/candidates.tsv` | `12df6872a3fe2a3f29b7b9a828dad8d394d302f837feaff4d0db07b8d7a432a0` | 同一轮的 138 行候选表；与第一轮攻方报告开头记的 sha256 逐字相同，输入没变 |

`measure.sh` 另从 git 取两份已归档的逐行判定报告（`git show 364d323^:research/prompts/owed-batch1to3-sync-judge-f1-f11.md` 与 `-f12-f21.md`），落到 `$D/j1.md`、`$D/j2.md`。
这三样都在 `/tmp` 下、都没入库：量尺不是实验装置、没有实验号，`research/results/` 收的是实验产物；事实表与候选表是上一轮的中间件，那一轮自己没把它们入库（见下面 G 格）。主 agent 要留就自己拷。

`measure.sh` 这一次的原样输出：

```
== 事实表 ==
事实行数 21 出处提门禁 0 出处提提交 0 出处指派发提示 20 出处只写H编号 1
== 候选表 ==
候选行数 138 组数 21 原文列>=500 的行 41 这些行截断后合计字符 20500
F3 组行数 19
== 判定分布（两份入库报告，138 行） ==
     77 不相干
     44 事件句不改
      2 要人看
     15 要改
== 入库过的同步记录与候选表 ==
同步记录 19
候选表 6
== 改法二的体量代价 ==
截断的 41 行：截断版 20500 字符，今天全文 39844 字符，改法二之后候选表原文列 +19344 字符
== 伪造一张「复判抽样」表 ==
每档抽到: {'要改': 2, '要补': 0, '事件句不改': 2, '不相干': 2, '要人看': 2}
抽到的行数: 8
每行载体都在候选表里: True
每行末格非空且 >=8 字: True
```

````

**出处 `research/prompts/agentdef-r2-opus-output.md:50-61`（整段抄，未转述）**

```markdown
## 各格判定一览

| # | 问的是什么 | 判定 | 一句话依据 |
|---|---|---|---|
| 一 | **改法一真修得掉 B、D、F 吗** | **B 修得掉；D 只修一半；F 不修，而且更坏** | 造了一张「复判抽样」表：不读一行载体、不重判一行，全部从已入库的逐行判定报告里机械抄出来，8 行、四档各 2 行、每行载体都在候选表里、每行末格 ≥8 字——改法一能提出的每一条机械判据都过 |
| 二 | **两句并存会不会只做抽样那一半** | **会，而且有三种合规读法** | 抽样那一半留产物、全看那一半一个字节都不留；改法一之前「不抽样」还让「做了抽样」在文本上是违规的，改法一之后从「只抽样」到「真全看」的每一种行为在仓里的字节完全相同 |
| 三 | **改法四的「出处」拿不拿得到** | **拿不到它括注里定义的那个东西；而它真正起作用的那一半（新说法）本来就在手上** | 唯一一份真事实表 21 行：出处提门禁 **0** 行、提提交 **0** 行、20 行指向一份从没入库的派发提示；而 06:42 那一行的判别子（94 号是 crate 粒度）逐字就在候选表同一行的「新事实」列里 |
| 四 | **四个改法两两冲突** | **改法一与改法三互相抵消；改法二与改法四互相取消；改法三修不全它声称修的 E** | 改法三给的「项目可以收严成逐行全看」正是一张「可以不做抽查」的许可证，而改法一的 D、F 全押在抽查那张表上；E 格有三份文本，改法三只够得着一份 |
| 五 | **四个改法都没修到的格** | **G 格：判错的原始证据不落盘** | 触发这一切的那一处判错（06:42），仓里两处记载互相矛盾、原判定一个字节都没留；改法一的抽样表按定义罩不到它——06:42 正是抽样没抽到的那一行 |

⚠️ 各节末尾按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:176`「判据自己也会写错：打中之后先判是哪一种」答四句。

```

**出处 `research/prompts/agentdef-r2-opus-output.md:263-302`（整段抄，未转述）**

````markdown
## 五、四个改法都没修到的格

### G 格：抓到的判错，仓里不留原始证据——而这条规则是为那一处判错立的

触发这一切的是 06:42 那一处判错。仓里今天关于它有三处记载，**互相对不上**：

1. `research/prompts/owed-batch1to3-sync.md:51`（整行抄）：

   > | .claude/main-agent.md:43 | 再从全部判定里随机抽 20 行（五种判定各至少 2 行，不够的全抽）自己复判 | 不改：这一批不带这条改动。判定是要改——用户定案，阶段同步的判定主 agent **逐行全看**，不抽样；抽样这一次漏掉了 06:42 那一处判错。改动已经在工作区里，但不进这次提交：门禁 72 号要求改过的 agent 定义在同一次改动带来的三方判决里被点名，这条规则改动单独走一轮三方之后再提交 |

2. 同一份记录 `:36`（整行抄）：

   > | .claude/kb/decisions/06-快照实现模型.md:42 | **欠**：C12（增量语义共用）：② 的引用计数旁表与运行时共用增量语义时…… | 改了：**不删**——这一格说的是 core crate 内部两段代码同源，门禁 94 号只判 checker 与 core 的边界，罩不到；改成写明这一格今天没有编号盯着 |

   「**不删**」三个字说明子 agent 当时判的是删。

3. 入库的逐行判定报告里那一行（`git show 364d323^:research/prompts/owed-batch1to3-sync-judge-f1-f11.md` 第 42 行，整行抄的前半截）：

   > | F3 | .claude/kb/decisions/06-快照实现模型.md:42 | 要改 | 把「**欠**：C12（增量语义共用）：② 的引用计数旁表与运行时共用增量语义时，I-3.1（已分配统计对得上） 抓不到加减语义本身写错。」改成「**欠**：C12（增量语义共用） 已还清的是 checker 与实现（core）之间那一格（门禁 94 号，crate 粒度依赖闭包交集为空）；② 的引用计数旁表与运行时共用增量语义这一格——两段代码同源在 core crate 内部——仍欠，门禁 94 号只判 checker crate 与 core crate 的边界

   **入库的这一行已经是正确判定**（`git log --all --oneline --` 那份报告只有两个提交：`11a551b` 加、`364d323` 删，只有一个版本）。

⇒ **那处判错原来长什么样，仓里一个字节都没有。** 重派之后入库的是改正后的报告，「原判定是什么、是谁在哪一步抓到的」全部只存在于同步记录里一句自述。
⇒ **四个改法没有一个修这一格**：改法一记的是「抽到哪几行」和重判结论，而 06:42 **正是抽样没抽到的那一行**（`sync.md:51` 逐字：「抽样这一次漏掉了 06:42 那一处判错」）——按定义它进不了那张表；改法二改截断、改法三改共享条文、改法四改判别子，三个都不产生任何关于「判错」的记录。

这一格是**可落成会失败的检查**的，这是它比 D、F 更该被看一眼的地方：要求同步记录里另有一张「复核改判」表（载体、原判定、改判后的判定、抓到它靠的是哪一件东西），门禁 68 号能机械核（载体格式、两个判定都是五种之一、两者不相等、末格非空），而且它在**0 处判错**的那一轮可以是 0 行表——与现有 ④「表可以 0 行」同形。它同时给 D 格提供了改法一提供不了的那件东西：一条「全看」独有的产物（抽样抽不到的行上的改判，只可能来自全看）。
⚠️ 这是**我这一条腿自己提的收严，被攻过零轮**，与第一轮那四个同级，按 `.claude/rules/three-way-inference.md:122`「攻方腿自己提的收严，只在它自己的模型上量过，算「没被攻过」」处理；而且按同一份规则「第三轮之后停」那一段，它是这一轮新冒出来的零轮形态。

### H 格（次要，同样四个都没修）：候选表本身的漏召回

四个改法管的全是「已经进了候选表那 138 行怎么判」。而 `research/scripts/stale-candidates.py:35-37`（整段抄，工具作者自己写在「管不到的」一节里）：

```
管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上）：
  写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地 / 实现这类字，过得了闸，
  这件事实的旧说法一行都进不了候选——判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表。
```

⇒ 工具自己指定的补法是「**主 agent 读事实表**」（21 行），而 `.claude/main-agent.md:43` 把主 agent 的人工动作全部押在「读全部判定」（138 行）上。**加码加在了工具声明自己管得住的那一段，工具声明管不住的那一段一个字都没加。** 四个改法都在这 138 行里打转。

````

**出处 `research/prompts/agentdef-r2-opus-output.md:303-312`（整段抄，未转述）**

```markdown
## 没打中的形状

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 「改法一的抽样表在门禁 68 号里立不起来」 | 读 `.claude/gate.d/68-knowledge-sync.sh` 文件头四条判据 | **没打中**：68 号判据④只管「## 命中处置」那一节「恰好一张表」，另起一节放第二张表不冲突。打中的是「那张表能核的谓词太弱」，写在第一节 |
| 「改法四的出处在别的轮次里是全的，只有这一轮空」 | 全仓只有这一份事实表在盘上（`facts.tsv`），另外几轮的事实表没入库（入库过的只有候选表 6 份） | **没打中，也没法打**：n=1，没有第二份事实表可比。这一条的结论只能读成「唯一一份真数据上是 0 命中」，不是「事实表的出处列一般都空」 |
| 「改法二调大截断之后会撞 `MAXIMUM_CANDIDATE_LINES_PER_FACT = 500`」 | `research/scripts/stale-candidates.py:75` | **没打中**：那是每条事实的候选行数上限，与第 284 行那个字符截断是两个不相干的量，同值只是巧合（第一轮已记） |
| 「06:42 那处判错在 git 里留了痕迹」 | `git log --all` 两份逐行判定报告全部版本、`owed-batch1to3*` 全部入库文件 | **打中的是反面**：报告只有一个版本、已是正确判定 ⇒ G 格。但「判错确实发生过」这件事我只有同步记录的自述与「不删」两个字作旁证，**没有直接证据**，见下面限度第 2 条 |
| 「改法一之后主 agent 做全看的时间成本可以量」 | 想拿 `research/results/` 里的计时产物 | **没做**：没有任何一轮记过复核这一步花了多久，量不出来 |

```

**出处 `research/prompts/agentdef-r2-opus-output.md:313-321`（整段抄，未转述）**

```markdown
## 这条腿自己的限度

1. **n=1。** 全部数出自同一轮阶段同步（owed-batch1to3，2026-09-21）的事实表与候选表，而它们是**上一轮别的会话留在 `/tmp` 里的中间件**，不是这一轮跑出来的；`candidates.tsv` 的 sha256 与第一轮攻方记的相同，说明没被动过，但「没被动过」不等于「代表一般情形」。
2. **「06:42 曾被判错」这件事我没有直接证据。** 入库的逐行判定报告只有一个版本、那一行已是正确判定；我拿到的旁证是 `owed-batch1to3-sync.md:51`「抽样这一次漏掉了 06:42 那一处判错」与 `:36` 的「不删」两个字。⇒ 第一轮 C 格与这一轮 G 格都建在这条自述上；主 agent 要坐实，得去问当轮会话或承认这条无法复核（那本身就是 G 格）。
3. **那张伪「复判抽样」表是按第一轮攻方替改法一设想的四条判据造的**（`agentdef-r1-opus-output.md:159`），不是按一道真门禁造的——**没有任何门禁判过这张表**，因为改法一还没落地。它证明的是「这四条谓词挡不住机械抄」，不是「未来某道门禁一定挡不住」。谁写出一道能分辨它的门禁，第一节的结论就翻。
4. **没在副本上改过代码。** 改法二那个 `[:500]` 我只按今天的载体行长算了体量差（`cost.py`），**没把脚本改掉重跑一次候选表**；「+19344 字符」是按今天工作区那 41 行的长度算的，与当时那一版可能不同。
5. **没量 token，只量字符。** 第 25.6%、95.3% 这类比例都是字符比，不是 token 比。
6. **不判第一轮的六格 A–F 本身**（那是这一轮辩方腿的活），也不判改法一的抽样下限在极端分布下成不成立（那是本地攻方的活）。

```

**出处 `research/prompts/agentdef-r2-opus-output.md:322-331`（整段抄，未转述）**

```markdown
## 什么现象会推翻本报告的结论

| 结论 | 推翻它的现象 |
|---|---|
| 一：改法一的抽样表核不动 | 写出一道门禁，能判红我那张 `forge.py` 生成的表，且对一张真做过重判的表判绿（双向判别力自证） |
| 二：能合规只做抽样那一半 | 给「逐行全看」单配一件产物（读过的载体清单、或 G 格那张改判表），使「只抽样」在仓里留下的字节与「全看」不同 |
| 三：改法四的出处拿不到 | 再取两份真事实表，出处列里点名门禁或提交的行占多数；或把「出处」的合法取值在 `stale-candidates.py` 文件头里改写成「提交 / 门禁 / 已定项」并让 `--check-facts` 核它 |
| 四：改法一与改法三互相抵消 | 改法三的措辞从「收严成」改成「项目可以在抽查之外另加逐行全看」（叠加而非替换），抵消当场消失 |
| 五：G 格没被修到 | 任何一轮同步记录里出现「原判定 → 改判后的判定」这一对，或任何一道门禁开始判它 |

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

**出处 `.claude/main-agent.md:35-49`（整段抄，未转述）**

```markdown
## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |

```

**出处 `.claude/gate.d/68-knowledge-sync.sh:1-290`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# gate-stage: 改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录
#
# 为什么：2026-09-17 一批改动做完后没人回头同步知识——records/2026-09-16-subagent拆分提案.md 与 .claude/agent-common.md
# 里两句「还没用过」在用过之后都留着，另一个会话在那句正下方追加新一批也没改它。用户定：阶段任务结束时有一个同步知识的任务点，
# 派 sweep 做阶段同步，主 agent 判完每处命中写一份同步记录。这一道判的是那份记录的形式。
#
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算
# （取法与 56-crates-adversarial-review.sh 相同，只多一个 core.quotepath=false：不加的话中文路径被 git 转成带引号的八进制，载体对不上）。
#
# 判据：
#   ① 触发文件：改动范围里匹配下面任一条的——^\.claude/agents/、^\.claude/agent-common\.md$、^\.claude/hooks/、
#      ^\.claude/settings\.json$、^\.claude/gate\.d/[^/]+\.(sh|py|tsv)$、^\.claude/rules/、^research/scripts/、
#      ^crates/[^/]+/src/、^CLAUDE\.md$。一个都没有 ⇒ 无对象可判，退 77（不记通过）。
#   ② 同步记录：改动范围里的 research/prompts/<阶段>-sync.md（prompts 顶层），且文件里有一行只写 <!-- knowledge-sync -->。
#      每个触发文件都要在某份同步记录里按路径逐字出现（等同 grep -F）；漏的逐个列出 ⇒ 红。
#   ③ 每份同步记录有「## 搜索」小节且不空 ⇒ 否则红。
#   ④ 每份同步记录有「## 命中处置」小节，小节里恰好一张表，表头 | 载体 | 原句 | 处置 |。逐行：
#      载体格是「路径:行号」，路径从仓库根起（可包一层反引号），在仓里存在或在改动范围里；
#      处置格以「改了」「补了」「不改：」之一开头，「不改：」后面要写理由；
#      处置是「改了」「补了」的，载体路径要在改动范围里（记录说改了而文件没动）。任一不合 ⇒ 红。表可以 0 行。
#
# 同步记录的格式（research/prompts/<阶段>-sync.md）：
#   <!-- knowledge-sync -->
#   # <阶段> 阶段同步
#
#   触发文件：.claude/agents/sweep.md、research/scripts/agent-watch.py（这一阶段改动范围里的触发文件，逐个按路径写全）
#
#   ## 搜索
#   回扫用的每条命令与它的命中计数，例：grep -rn "<旧说法>" .claude records research | wc -l → 3
#
#   ## 命中处置
#   | 载体 | 原句 | 处置 |
#   |---|---|---|
#   | .claude/agent-common.md:120 | <那一行的原句> | 改了：<改成了什么> |
#   | .claude/kb/pitfalls.md:40 | （新增） | 补了：<补了什么> |
#   | records/2026-09-16-subagent拆分提案.md:88 | <那一行的原句> | 不改：<理由，例：说的是那一次发生的事> |
#   两个小节标题逐字写，后面不加字；原句里的 | 写成 \|；代码围栏里的 # 行不算标题。
#
# 管不到的：记录写得对不对（原句是不是那一行、「不改」的理由站不站得住）、回扫搜没搜全、
# 只用过而没改动的东西（用过之后该改的句子，只要没碰触发范围就不触发）——这些靠阶段收尾的任务点与人。
# 「改了」「补了」只核载体文件在改动范围里，不核那一行真的动了；行号不核是否越界；路径里带制表符、换行或引号的，git 仍会转义，认不出。
# 判别力：fixtures/68-knowledge-sync.sh/red 放一个没被点名的触发文件、一个只在不带标记的文件里点名的触发文件、
# 说改了而载体没动、开头不合法、不改没理由、载体没行号、载体不存在、缺「## 搜索」、表头写错，必须判红；
# green 放三个触发文件分在两份记录里点名、三种处置各一行、中文文件名的载体、原句里的 \|、表后围栏里的竖线行、一份 0 行的表，必须判绿并报对数。
#
#   bash .claude/gate.d/68-knowledge-sync.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
base="HEAD"
if [[ -n "${GATE_BASE:-}" ]] && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
  base="$GATE_BASE"
elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1; then
  base="$(git merge-base HEAD '@{upstream}')"
fi
changed="$( { git -c core.quotepath=false diff --name-only "$base" -- ; git -c core.quotepath=false diff --name-only --cached -- ; git -c core.quotepath=false ls-files --others --exclude-standard -- ; } | sort -u )"
# 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
python3 - "$base" <(printf '%s\n' "$changed") <<'PY'
import os, re, subprocess, sys

base = sys.argv[1]
with open(sys.argv[2], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})
changed_set = set(changed_files)

trigger_patterns = [re.compile(pattern) for pattern in (
    r"^\.claude/agents/",
    r"^\.claude/agent-common\.md$",
    r"^\.claude/hooks/",
    r"^\.claude/settings\.json$",
    r"^\.claude/gate\.d/[^/]+\.(sh|py|tsv)$",
    r"^\.claude/rules/",
    r"^research/scripts/",
    r"^crates/[^/]+/src/",
    r"^CLAUDE\.md$",
)]
trigger_files = [path for path in changed_files if any(pattern.search(path) for pattern in trigger_patterns)]
outside_trigger_count = len(changed_files) - len(trigger_files)
if not trigger_files:
    print(f"  ! 这次改动没碰规则、agent、hook、门禁、脚本与实现，本阶段无对象可判（改动范围 {len(changed_files)} 个文件，基准 {base}）")
    sys.exit(77)

marker_line = "<!-- knowledge-sync -->"
record_name = re.compile(r"^research/prompts/[^/]+-sync\.md$")
records = []
fresh_records = set()
unmarked_candidates = []
for path in changed_files:
    if not record_name.search(path) or not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    if any(line.strip() == marker_line for line in text.split("\n")):
        records.append((path, text))
        # 这一轮**新写**的那几份：处置行的「改了 / 补了」只对它们判。
        # ⚠️ 跨轮留存的旧记录点名的载体是更早一轮改的，相对今天的基准当然不在范围里，
        # 而那些行说的都是真话——改写它们才是假的（C450 实测 2026-09-21）。
        # 判据用同一个基准：相对基准是新增（A）的才算这一轮写的；只是被改过（M，例如
        # 按 C450 追加一句「为什么这一轮还留着」）的不算。
        # 判据是「基准那一版里有没有这个文件」，不是 `git diff --name-status`：
        # 后者看不见**未跟踪**的新文件，而这一轮刚写出来还没 add 的记录正是那一种
        # （门禁自己的红样本就这么造的，用 diff 判会把它当成旧记录放行）。
        exists_at_base = subprocess.run(
            ["git", "cat-file", "-e", f"{base}:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_base:
            fresh_records.add(path)
    else:
        unmarked_candidates.append(path)

fence_open = re.compile(r"^ {0,3}(`{3,}|~{3,})")
heading = re.compile(r"^#{1,2}\s")


def second_level_sections(text):
    """二级标题整行 → 小节体 [(行号, 行, 是否在代码围栏里)]；同名标题只认第一次，代码围栏里的 # 行不算标题。"""
    sections = {}
    current_body = None
    open_fence = None
    for line_number, line in enumerate(text.split("\n"), 1):
        if open_fence is None:
            fence_match = fence_open.match(line)
            if fence_match:
                open_fence = fence_match.group(1)
                if current_body is not None:
                    current_body.append((line_number, line, True))
                continue
            if heading.match(line):
                title = line.rstrip()
                if title.startswith("## ") and title not in sections:
                    sections[title] = []
                    current_body = sections[title]
                else:
                    current_body = None
                continue
            if current_body is not None:
                current_body.append((line_number, line, False))
        else:
            if re.match(r"^ {0,3}" + re.escape(open_fence[0]) + "{" + str(len(open_fence)) + r",}\s*$", line):
                open_fence = None
            if current_body is not None:
                current_body.append((line_number, line, True))
    return sections


def table_cells(row):
    inner = row.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|") and not inner.endswith("\\|"):
        inner = inner[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", inner)]


carrier_form = re.compile(r"^(?P<path>\S(?:.*\S)?):(?P<line>[1-9][0-9]*)$")
separator_cell = re.compile(r"^:?-{3,}:?$")

missing_triggers = [path for path in trigger_files if not any(path in text for _record_path, text in records)]
search_problems = []
table_problems = []
carrier_problems = []
disposition_start_problems = []
empty_reason_problems = []
out_of_range_problems = []
disposition_counts = {"改了": 0, "补了": 0, "不改": 0}

for record_path, text in records:
    sections = second_level_sections(text)
    if "## 搜索" not in sections:
        search_problems.append(f"{record_path}：没有「## 搜索」小节")
    elif not any(line.strip() for _number, line, _fenced in sections["## 搜索"]):
        search_problems.append(f"{record_path}：「## 搜索」小节是空的")
    if "## 命中处置" not in sections:
        table_problems.append(f"{record_path}：没有「## 命中处置」小节")
        continue
    table_runs = []
    previous_was_table_line = False
    for line_number, line, fenced in sections["## 命中处置"]:
        is_table_line = (not fenced) and line.lstrip().startswith("|")
        if is_table_line and not previous_was_table_line:
            table_runs.append([])
        if is_table_line:
            table_runs[-1].append((line_number, line))
        previous_was_table_line = is_table_line
    if not table_runs:
        table_problems.append(f"{record_path}：「## 命中处置」小节里没有表")
        continue
    if len(table_runs) > 1:
        table_problems.append(f"{record_path}：「## 命中处置」小节里有 {len(table_runs)} 张表，只许一张（第二张从第 {table_runs[1][0][0]} 行起）")
        continue
    table_rows = table_runs[0]
    header_number, header_line = table_rows[0]
    if table_cells(header_line) != ["载体", "原句", "处置"]:
        table_problems.append(f"{record_path}:{header_number}：「## 命中处置」的表头不是 | 载体 | 原句 | 处置 |，实际是 {header_line.strip()}")
        continue
    if len(table_rows) < 2 or len(table_cells(table_rows[1][1])) != 3 or not all(separator_cell.match(cell) for cell in table_cells(table_rows[1][1])):
        table_problems.append(f"{record_path}:{header_number + 1}：表头下面一行不是三格的分隔行 |---|---|---|")
        continue
    for row_number, row_line in table_rows[2:]:
        cells = table_cells(row_line)
        location = f"{record_path}:{row_number}"
        if len(cells) != 3:
            table_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
            continue
        carrier_text, _original_sentence, disposition = cells
        if len(carrier_text) >= 2 and carrier_text.startswith("`") and carrier_text.endswith("`"):
            carrier_text = carrier_text[1:-1].strip()
        carrier_match = carrier_form.match(carrier_text)
        carrier_path = None
        if not carrier_match:
            carrier_problems.append(f"{location}：载体格不是「路径:行号」：{carrier_text}")
        else:
            candidate_path = carrier_match.group("path")
            if candidate_path.startswith(("/", "./", "../")) or "/../" in candidate_path:
                carrier_problems.append(f"{location}：载体路径不是从仓库根起写的：{candidate_path}")
            elif not os.path.exists(candidate_path) and candidate_path not in changed_set:
                carrier_problems.append(f"{location}：载体路径 {candidate_path} 在仓里不存在，也不在改动范围里")
            else:
                carrier_path = candidate_path
        if disposition.startswith("改了") or disposition.startswith("补了"):
            disposition_counts[disposition[:2]] += 1
            if (record_path in fresh_records
                    and carrier_path is not None and carrier_path not in changed_set):
                out_of_range_problems.append(f"{location}：处置写「{disposition[:2]}」，载体 {carrier_path} 不在改动范围里")
        elif disposition.startswith("不改："):
            disposition_counts["不改"] += 1
            if not disposition[len("不改："):].strip():
                empty_reason_problems.append(f"{location}：「不改：」后面没写理由")
        else:
            disposition_start_problems.append(f"{location}：处置开头不合法：{disposition or '（空）'}")

sync_step = ("阶段同步怎么做：阶段任务结束时派 sweep 做阶段同步（sweep 定义里的第四种活），主 agent 判完每处命中写 "
             "research/prompts/<阶段>-sync.md，格式见 .claude/gate.d/68-knowledge-sync.sh 文件头。")
failed = False
if missing_triggers:
    failed = True
    print(f"  ✗ {len(missing_triggers)} 个触发文件没在任何同步记录里点名（改动范围里带 <!-- knowledge-sync --> 的 research/prompts/*-sync.md 共 {len(records)} 份，一份都没按路径提到它）：")
    for path in missing_triggers:
        print(f"      {path}")
    if unmarked_candidates:
        print("     这几份文件名像同步记录，但没有只写 <!-- knowledge-sync --> 的那一行，不算：")
        for path in unmarked_candidates:
            print(f"      {path}")
    print("     → 怎么办：这一阶段还没做知识同步就先做；做过的，把上面每个文件按路径写全补进同步记录的「触发文件」一行。")
if search_problems:
    failed = True
    print(f"  ✗ {len(search_problems)} 份同步记录缺「## 搜索」或它是空的：")
    for entry in search_problems:
        print(f"      {entry}")
    print("     → 怎么办：加「## 搜索」小节（标题逐字写、后面不加字），写回扫用的每条命令与它的命中计数。")
if table_problems:
    failed = True
    print(f"  ✗ {len(table_problems)} 处「## 命中处置」的表写坏了：")
    for entry in table_problems:
        print(f"      {entry}")
    print("     → 怎么办：「## 命中处置」下面恰好一张表，表头逐字是 | 载体 | 原句 | 处置 |，第二行 |---|---|---|；每行三格，原句里的 | 写成 \\|；没有命中也留表头（0 行）。")
if carrier_problems:
    failed = True
    print(f"  ✗ {len(carrier_problems)} 行载体格不是仓库根起的「路径:行号」或指不到文件：")
    for entry in carrier_problems:
        print(f"      {entry}")
    print("     → 怎么办：载体格写仓库根起的 路径:行号（例 .claude/agent-common.md:120），路径要在仓里现存；别写成相对 research/prompts/ 的路径。")
if disposition_start_problems:
    failed = True
    print(f"  ✗ {len(disposition_start_problems)} 行处置开头不合法（只认「改了」「补了」「不改：」）：")
    for entry in disposition_start_problems:
        print(f"      {entry}")
    print("     → 怎么办：每处命中先判完再写：改了写「改了：…」，新加的写「补了：…」，不动写「不改：理由」；别写「待定」「看过」。")
if empty_reason_problems:
    failed = True
    print(f"  ✗ {len(empty_reason_problems)} 行「不改：」没写理由：")
    for entry in empty_reason_problems:
        print(f"      {entry}")
    print("     → 怎么办：「不改：」后面写不改的理由，例：冻结证据 / 说的是那一次发生的事 / 同一个词、不同的事。")
if out_of_range_problems:
    failed = True
    print(f"  ✗ {len(out_of_range_problems)} 行处置写「改了」「补了」而载体不在改动范围里（记录说改了，文件没动）：")
    for entry in out_of_range_problems:
        print(f"      {entry}")
    print("     → 怎么办：去把那一处真的改了；判下来不该改的，处置写成「不改：理由」；也对一遍载体路径是不是写成了别的文件。")
if failed:
    print(f"     → {sync_step}")
    sys.exit(1)
hit_rows = sum(disposition_counts.values())
print(f"  ✓ 触发文件 {len(trigger_files)} 个都在同步记录里点名（同步记录 {len(records)} 份；命中 {hit_rows} 行："
      f"改了 {disposition_counts['改了']} / 补了 {disposition_counts['补了']} / 不改 {disposition_counts['不改']}；基准 {base}）；"
      f"改动范围里另有 {outside_trigger_count} 个文件不在触发范围")
PY
```

**出处 `research/scripts/stale-candidates.py:1-670`（整段抄，未转述）**

```markdown
#!/usr/bin/env python3
"""阶段同步的候选表：这一阶段哪些事实变了，仓里还有哪些现状句在说旧的。回扫员照表逐行判，不自己挑关键词、不整组放行。

用法（按次序）：
    stale-candidates.py --changes --base 提交 [--target 提交] --out 变更清单.md
        列出这一阶段在 kb 各文件「## 历史版本」节里新加的每一条变更记录（编号 H1、H2……），以及每个被改过的现状载体
        里删改前后的差异片段——写事实表的原料
    stale-candidates.py --check-facts 事实表.tsv --base 提交 [--target 提交]
        核事实表罩没罩全：每条 H 编号都要出现在某一行的「出处」列里；每行的检索词要是合法的正则，
        而且在基准那一版的现状句里至少命中一行（旧说法当时就在那里）；新立事项反过来，检索词在基准那一版要零命中
    stale-candidates.py --facts 事实表.tsv --base 提交 [--target 提交] --out 候选.tsv
        按事实表的检索词在全部现状载体里搜，每处命中一行候选
    stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]
        核报告有没有判到候选表里（分到的组里）的每一行
    stale-candidates.py --benchmark     拿仓里一段真实历史与一份盲写的事实表当阳性对照
    stale-candidates.py --selftest

事实表（制表符分隔，首行是表头）：
    编号	旧事实	新事实	检索词	出处
    F1	层 0 的负载只有第一个事务	层 0 两条流：……	层 0|崩溃点重放|录制流	H3;H17;提交 cae5092
  检索词是 Python 正则，写这件事实涉及的概念名词（「层 0」「录制流」「多次挂载」），新旧两种说法都会提到的那种；
  不写状态词，不照抄新说法的原句（「按整条流切段」只搜得到改好了的句子）；宁宽勿窄。
  几个词要同时出现就用 && 连（「E142&&改动计数」）；一行事实的检索词在结束那一版命中不许超过 150 行，超了就是太宽，用 && 收窄。
  出处写这条事实来自哪几条变更记录（H 编号，--changes 给的）或哪个提交。
  旧事实写「只改措辞」的行只用来罩住变更记录，不出候选。
  旧事实以「新立：」开头的行是这一阶段之前仓里一个字都没提过的东西（新立的欠账、新门禁阶段），也不出候选；
  实现落地、实验跑完、欠账还清不是新立，是事实变了：旧事实写仓里原来怎么说它没有 / 还欠，检索词写那件事的概念名词。

报告（--check-report 判的就是这个）：
    ## 逐行判定
    | 组 | 载体 | 判定 | 改后的句子或理由 |
  候选表里每一行都要有一行；判定以「要改」「要补」「事件句不改」「不相干」「要人看」之一开头；
  最后一格不能空：要改的写改后的句子，其余写理由（至少 8 个字）。几份报告合起来判。

管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上）：
  写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地 / 实现这类字，过得了闸，
  这件事实的旧说法一行都进不了候选——判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表。
  逐行判的理由引了原话、判定也是五种之一，照样可以是没读懂就判的；靠主 agent 在全部判定里抽查复判。

退出码：0 通过；2 参数或 git 出错；3 报告漏判或判定格式不对；4 阳性对照漏了已知的腐烂；5 自证失败；6 事实表没罩全。
"""
import argparse
import difflib
import hashlib
import os
import re
import subprocess
import sys
import tempfile

CARRIER_PATTERNS = [
    r'^CLAUDE\.md$',
    r'^README\.md$',
    r'^\.claude/agent-common\.md$',
    r'^\.claude/main-agent\.md$',
    r'^\.claude/agents/[^/]+\.md$',
    r'^\.claude/rules/[^/]+\.md$',
    r'^\.claude/skills/[^/]+/SKILL\.md$',
    r'^\.claude/handover/[^/]+/README\.md$',
    r'^\.claude/kb/.+\.md$',
    r'^records/[^/]+\.md$',
]
CHANGE_LOG_ONLY_PATTERNS = [
    r'^\.claude/kb/decisions-history',
    r'^\.claude/kb/experiments-history\.md$',
]
HISTORY_SECTION_TITLE = '## 历史版本'
HISTORY_ENTRY_HEADING = re.compile(r'^### .+$', re.M)
LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')
MINIMUM_REASON_LENGTH = 8
FACT_TABLE_COLUMNS = ['编号', '旧事实', '新事实', '检索词', '出处']
NEWLY_ADDED_OLD_FACT_PREFIX = '新立：'
NOT_NEWLY_ADDED_WORDS = re.compile(r'落地|实现|已有|跑完|还清|有了')
REWORDING_ONLY_OLD_FACT_PREFIX = '只改措辞'
MAXIMUM_CANDIDATE_LINES_PER_FACT = 500
REWORDING_HEADING_WORDS = re.compile(r'措辞|指代|写法|改名|搬|路径|一个字(不|没)改|错字|拼写|简称|格式对齐')
MINIMUM_QUOTED_CHARACTERS = 4
TERM_CONJUNCTION = '&&'


def compile_search_term(term):
    """「A&&B」→ 每一段各编一个正则，一行里全都命中才算命中。"""
    return [re.compile(part) for part in term.split(TERM_CONJUNCTION)]


def search_term_matches(compiled_parts, line):
    return all(part.search(line) for part in compiled_parts)

# 阳性对照：2026-09-18 知识腐烂回扫里主 agent 逐条现查坐实的 25 处腐烂（research/prompts/knowledge-rot-2026-09-18-sync.md），
# 行号是 BENCHMARK_TARGET 那一版的。只存「载体:行号」的 sha256 前 16 位：回扫员读得到这个文件，明文会被照抄成答案。
# 事实表 BENCHMARK_FACTS 由一个没读过这 25 处的 agent 照这一段历史的提交正文与变更记录盲写（第四版，
# research/prompts/sweep-acceptance-2026-09-18-v2-facts.md）；2026-09-19 三方第一轮之后按新闸把 9 行多余的 && 机械地退回第一个词
# （research/prompts/sweep-rel-r1-main-verification.md），罩住 25 处。对照守两样：罩住的不少于这个数、候选行数与冻结时相同。
# 答案在仓里另有明文（那份同步记录），对照只防工具被改窄，不当盲测用。
BENCHMARK_BASE = 'b1c8cef~1'
BENCHMARK_TARGET = '00c9d4f'
BENCHMARK_FACTS = 'research/scripts/fixtures/stale-candidates-benchmark-facts.tsv'
BENCHMARK_MINIMUM_COVERED = 25
BENCHMARK_EXPECTED_CANDIDATE_ROWS = 4190
BENCHMARK_EXPECTED_UNIQUE_LINES = 2821
BENCHMARK_KNOWN_STALE_LOCATION_HASHES = [
    '88600dee84e43cb1', '603dc36527671cbc', 'a93c0915fab7efa8', '948054ed6df42b0b', '7bb23ca4ed5f8432',
    '6287ca8bd47723bf', '201247a903f206ae', '16403eb11c52d13c', '82e4f62b752e4383', '84b95c231a2692a3',
    '767dc5b0a2eed64f', '43cec00ac6c7e52f', '7ec30e7996379b13', 'ef8cdf810547c225', '799d9c319db094c0',
    '8eb3cae28cc99670', 'bbed8920622afcc2', '4af43929ce1d4f2b', '9919777fff2f4391', '7c5fdaebc4e022d3',
    '873c19307057473f', '8dfe08b83d4f054f', '86b893c315e5795e', 'a3ae13e499fa522c', 'b4e110d8f010be83',
]


def normalize(text):
    """去掉空白、强调与表格记号，比原话时不受排版影响。"""
    return re.sub(r'[\s*`>#|]+', '', text)


def location_hash(location):
    return hashlib.sha256(location.encode('utf-8')).hexdigest()[:16]


def matches_any(path, patterns):
    return any(re.search(pattern, path) for pattern in patterns)


def is_current_state_carrier(path):
    return matches_any(path, CARRIER_PATTERNS) and not matches_any(path, CHANGE_LOG_ONLY_PATTERNS)


def run_git(arguments, repository_root):
    completed = subprocess.run(['git', '-C', repository_root, '-c', 'core.quotepath=false', *arguments],
                               capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(f'git {" ".join(arguments)} 失败：{completed.stderr.strip()}')
    return completed.stdout


def read_version(repository_root, revision, path):
    """revision 为 None 读工作区；文件不在返回空串。"""
    if revision is None:
        full_path = os.path.join(repository_root, path)
        if not os.path.isfile(full_path):
            return ''
        with open(full_path, encoding='utf-8', errors='replace') as handle:
            return handle.read()
    completed = subprocess.run(['git', '-C', repository_root, 'show', f'{revision}:{path}'],
                               capture_output=True, text=True)
    return completed.stdout if completed.returncode == 0 else ''


def all_paths(repository_root, revision):
    if revision is None:
        paths = (run_git(['ls-files'], repository_root).split('\n')
                 + run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n'))
    else:
        paths = run_git(['ls-tree', '-r', '--name-only', revision], repository_root).split('\n')
    return sorted({path for path in paths if path})


def changed_paths(repository_root, base, target):
    paths = run_git(['diff', '--name-only', base] + ([target] if target else []), repository_root).split('\n')
    if target is None:
        paths += run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n')
    return sorted({path for path in paths if path})


def current_state_lines(text):
    """历史版本节之前的行，(行号, 原文)。"""
    lines = []
    for line_number, line in enumerate(text.split('\n'), start=1):
        if line.strip() == HISTORY_SECTION_TITLE:
            break
        lines.append((line_number, line))
    return lines


def history_headings(text, path):
    """变更记录的标题：普通 kb 文件取「## 历史版本」之后的三级标题；变更史文件整份都是变更记录。"""
    if matches_any(path, CHANGE_LOG_ONLY_PATTERNS):
        section = text
    else:
        start = text.find('\n' + HISTORY_SECTION_TITLE)
        if start < 0:
            return []
        section = text[start:]
    return HISTORY_ENTRY_HEADING.findall(section)


def renamed_from(repository_root, base, target):
    """{新路径: 旧路径}：搬了目录、改了名的文件，历史标题要拿旧路径那一版去比。"""
    output = run_git(['diff', '--name-status', '-M', base] + ([target] if target else []), repository_root)
    renames = {}
    for line in output.split('\n'):
        columns = line.split('\t')
        if len(columns) == 3 and columns[0].startswith('R'):
            renames[columns[2]] = columns[1]
    return renames


def new_change_entries(repository_root, base, target):
    """[(H 编号, 文件, 标题)]，按文件与出现次序编号，同一 base / target 下稳定。"""
    entries = []
    renames = renamed_from(repository_root, base, target)
    for path in changed_paths(repository_root, base, target):
        if not path.startswith('.claude/kb/') or not path.endswith('.md'):
            continue
        base_path = renames.get(path, path)
        before = set(history_headings(read_version(repository_root, base, base_path), base_path))
        for heading in history_headings(read_version(repository_root, target, path), path):
            if heading not in before:
                entries.append((path, heading))
    return [(f'H{index}', path, heading) for index, (path, heading) in enumerate(entries, start=1)]


def changed_segments(base_text, target_text):
    """删改前后的差异片段 [(旧片段, 新片段)]：每一行被删的现状句配上同一文件新加的最像的一行，按字符比出改掉的那几段。"""
    base_lines = [line for _, line in current_state_lines(base_text)]
    target_lines = [line for _, line in current_state_lines(target_text)]
    target_set, base_set = set(target_lines), set(base_lines)
    added = [line for line in target_lines if line not in base_set and line.strip()]
    segments = []
    for removed in (line for line in base_lines if line not in target_set and line.strip()):
        replacement = max(added, default='', key=lambda candidate: difflib.SequenceMatcher(
            None, removed, candidate, autojunk=False).quick_ratio())
        matcher = difflib.SequenceMatcher(None, removed, replacement, autojunk=False)
        for operation, old_start, old_end, new_start, new_end in matcher.get_opcodes():
            if operation in ('replace', 'delete') and len(removed[old_start:old_end].strip()) >= 2:
                segments.append((removed[max(0, old_start - 12):old_end + 12].strip(),
                                 replacement[max(0, new_start - 12):new_end + 12].strip()))
    return segments


def write_changes(repository_root, base, target, output):
    output.write(f'# 变更清单 {base}..{target or "工作区"}\n\n## 变更记录（事实表的「出处」列要罩住每一个 H 编号）\n\n')
    output.write('| 编号 | 文件 | 标题 |\n|---|---|---|\n')
    for entry_id, path, heading in new_change_entries(repository_root, base, target):
        output.write(f'| {entry_id} | {path} | {heading[4:].replace("|", "｜")} |\n')
    output.write('\n## 现状载体里改掉的片段\n\n')
    for path in changed_paths(repository_root, base, target):
        if not is_current_state_carrier(path):
            continue
        segments = changed_segments(read_version(repository_root, base, path), read_version(repository_root, target, path))
        if not segments:
            continue
        output.write(f'### {path}\n\n')
        for old_segment, new_segment in segments:
            output.write(f'- 旧：{old_segment[:200]}\n  新：{new_segment[:200]}\n')
        output.write('\n')


def read_fact_table(path):
    with open(path, encoding='utf-8') as handle:
        rows = [line.rstrip('\n').split('\t') for line in handle if line.strip()]
    if not rows or rows[0][:len(FACT_TABLE_COLUMNS)] != FACT_TABLE_COLUMNS:
        raise ValueError(f'{path} 的表头要是：{" / ".join(FACT_TABLE_COLUMNS)}（制表符分隔）')
    facts = []
    for row in rows[1:]:
        if len(row) < len(FACT_TABLE_COLUMNS):
            raise ValueError(f'{path} 这一行少列：{row[0] if row else "（空）"}')
        facts.append(dict(zip(FACT_TABLE_COLUMNS, row)))
    return facts


def current_state_corpus(repository_root, target):
    return {path: current_state_lines(read_version(repository_root, target, path))
            for path in all_paths(repository_root, target) if is_current_state_carrier(path)}


def build_candidates(repository_root, target, facts):
    """[(组号, 旧事实, 新事实, 载体:行号, 原文)]。"""
    corpus = current_state_corpus(repository_root, target)
    candidates = []
    for fact in facts:
        if fact['旧事实'].startswith((REWORDING_ONLY_OLD_FACT_PREFIX, NEWLY_ADDED_OLD_FACT_PREFIX)):
            continue
        compiled_parts = compile_search_term(fact['检索词'])
        for path, lines in corpus.items():
            for line_number, line in lines:
                if search_term_matches(compiled_parts, line):
                    candidates.append((fact['编号'], fact['旧事实'], fact['新事实'], f'{path}:{line_number}', line.strip()))
    return candidates


def write_candidates(candidates, output):
    output.write('组\t旧事实\t新事实\t载体\t原文\n')
    for group, old_fact, new_fact, location, line in candidates:
        output.write(f'{group}\t{old_fact}\t{new_fact}\t{location}\t{line.replace(chr(9), " ")[:500]}\n')


def check_facts(repository_root, base, target, fact_path):
    try:
        facts = read_fact_table(fact_path)
    except ValueError as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：照本脚本文件头「事实表」那一段的格式写，五列用制表符分开')
        return 6
    entries = new_change_entries(repository_root, base, target)
    cited = set()
    for fact in facts:
        cited.update(re.findall(r'H\d+', fact['出处']))
    uncovered = [(entry_id, path, heading) for entry_id, path, heading in entries if entry_id not in cited]
    base_corpus = current_state_corpus(repository_root, base)
    target_corpus = current_state_corpus(repository_root, target)
    heading_by_id = {entry_id: heading for entry_id, _, heading in entries}
    broken, empty, too_broad, not_new, fake_rewording, needless_conjunction, gone = [], [], [], [], [], [], []
    for fact in facts:
        try:
            compiled_parts = compile_search_term(fact['检索词'])
        except re.error as error:
            broken.append(f'{fact["编号"]}（{error}）')
            continue
        if fact['旧事实'].startswith(REWORDING_ONLY_OLD_FACT_PREFIX):
            cited_headings = [heading_by_id[entry_id] for entry_id in re.findall(r'H\d+', fact['出处']) if entry_id in heading_by_id]
            if any(not REWORDING_HEADING_WORDS.search(heading) for heading in cited_headings):
                fake_rewording.append(fact['编号'])
            continue
        if fact['旧事实'].startswith(NEWLY_ADDED_OLD_FACT_PREFIX):
            base_hits = any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines)
            if base_hits or NOT_NEWLY_ADDED_WORDS.search(fact['新事实']):
                not_new.append(fact['编号'])
            continue
        target_hits = sum(1 for lines in target_corpus.values() for _, line in lines
                          if search_term_matches(compiled_parts, line))
        if target_hits > MAXIMUM_CANDIDATE_LINES_PER_FACT:
            too_broad.append(f'{fact["编号"]}（{target_hits} 行）')
        if target_hits == 0:
            gone.append(fact['编号'])
        if len(compiled_parts) > 1:
            first_part_hits = sum(1 for lines in target_corpus.values() for _, line in lines if compiled_parts[0].search(line))
            if first_part_hits <= MAXIMUM_CANDIDATE_LINES_PER_FACT:
                needless_conjunction.append(f'{fact["编号"]}（第一个词只命中 {first_part_hits} 行）')
        if not any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines):
            empty.append(fact['编号'])
    if uncovered or broken or empty or too_broad or not_new or fake_rewording or needless_conjunction or gone:
        for entry_id, path, heading in uncovered[:40]:
            print(f'  ✗ {entry_id} 没有任何一行事实罩着：{path} {heading[4:80]}')  # gate-lint:detail
        if broken:
            print(f'  ✗ 检索词不是合法的正则：{"、".join(broken)}')  # gate-lint:detail
        if empty:
            print(f'  ✗ 检索词在基准那一版的现状句里一处都没命中（多半照抄了新说法）：{"、".join(empty)}')  # gate-lint:detail
        if not_new:
            print(f'  ✗ 标成「新立：」却不是新东西（检索词在基准那一版有命中，或新事实写着落地 / 实现 / 已有 / 跑完 / 还清）：'  # gate-lint:detail
                  f'{"、".join(not_new)}')
        if fake_rewording:
            print(f'  ✗ 标成「只改措辞」的行引的变更记录，标题里看不出只改了措辞：{"、".join(fake_rewording)}')  # gate-lint:detail
        if needless_conjunction:
            print(f'  ✗ 用了 && 而第一个词自己不超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行，不许再收窄：{"、".join(needless_conjunction)}')  # gate-lint:detail
        if gone:
            print(f'  ✗ 检索词在结束那一版一处都没命中（照抄了被这一阶段改掉的旧句）：{"、".join(gone)}')  # gate-lint:detail
        if too_broad:
            print(f'  ✗ 检索词太宽，结束那一版命中超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行：{"、".join(too_broad)}')  # gate-lint:detail
        print(f'  ✗ 事实表没罩全：{len(uncovered)} 条变更记录没罩、{len(broken)} 行正则坏了、'  # gate-lint:summary
              f'{len(empty)} 行检索词在基准零命中、{len(gone)} 行在结束零命中、{len(too_broad)} 行检索词太宽、'
              f'{len(needless_conjunction)} 行多余的 &&、{len(not_new)} 行冒充新立、{len(fake_rewording)} 行冒充只改措辞')
        print('     → 怎么办：读 --changes 给的变更清单，每条 H 编号写进它说的那件事实的「出处」列（只改措辞的也写一行，旧新事实写「只改措辞」）；'
              '检索词写新旧说法都会提到的概念名词，基准与结束两版都要命中；太宽（超过上限）才用 && 并上第二个概念名词收窄；'
              '事实真变了的不许写成只改措辞或新立')
        return 6
    print(f'  ✓ 事实表罩全了：{len(entries)} 条变更记录都有出处，{len(facts)} 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）')
    return 0


def parse_group_ranges(text):
    """「F1-F12,F15」→ 组号集合；None 表示全部。"""
    if not text:
        return None
    selected = set()
    for part in text.split(','):
        match = re.fullmatch(r'([A-Z])(\d+)(?:-[A-Z]?(\d+))?', part.strip())
        if not match:
            if not re.fullmatch(r'[A-Z]\w*', part.strip()):
                raise ValueError(f'组号区间写不认得：{part}')
            selected.add(part.strip())
            continue
        low, high = int(match.group(2)), int(match.group(3) or match.group(2))
        if high < low or re.fullmatch(r'[A-Z]\d+-([A-Z])\d+', part.strip()) and part.strip().split('-')[1][0] != match.group(1):
            raise ValueError(f'组号区间写不认得（倒写，或两端字母不同）：{part}')
        selected.update(f'{match.group(1)}{number}' for number in range(low, high + 1))
    return selected


def table_rows_under(title, text):
    """某个二级标题下表格的数据行（去掉表头与分隔行），每行拆成单元格。"""
    rows, inside = [], False
    for line in text.split('\n'):
        if line.startswith('## '):
            inside = line.strip() == title
            continue
        if inside and line.startswith('|') and not re.match(r'^\|\s*-', line):
            rows.append([cell.strip().strip('`') for cell in line.strip().strip('|').split('|')])
    return [row for row in rows if row and row[0] != '组']


def read_candidate_rows(table_path):
    """[((组号, 载体:行号), 原文)]。"""
    rows = []
    with open(table_path, encoding='utf-8') as handle:
        next(handle)
        for row in handle:
            columns = row.rstrip('\n').split('\t')
            rows.append(((columns[0], columns[3]), columns[4] if len(columns) > 4 else ''))
    return rows


def quotes_the_line(reason, line):
    """理由里有没有这一行的原话（去掉空白与强调记号之后，至少 MINIMUM_QUOTED_CHARACTERS 个连续字符）。"""
    normalized_line, normalized_reason = normalize(line), normalize(reason)
    if len(normalized_line) < MINIMUM_QUOTED_CHARACTERS:
        return True
    return any(normalized_line[start:start + MINIMUM_QUOTED_CHARACTERS] in normalized_reason
               for start in range(len(normalized_line) - MINIMUM_QUOTED_CHARACTERS + 1))


def check_reports(table_path, report_paths, selected_groups=None):
    candidate_rows = read_candidate_rows(table_path)
    if selected_groups is not None:
        unknown = sorted(selected_groups - {key[0] for key, _ in candidate_rows})
        if unknown:
            print(f'  ✗ --groups 里有候选表没有的组：{"、".join(unknown[:20])}')
            print('     → 怎么办：组号照候选表第一列写，区间两端同一个字母、小号在前')
            return 2
        candidate_rows = [row for row in candidate_rows if row[0][0] in selected_groups]
    keys = [key for key, _ in candidate_rows]
    text_by_key = dict(candidate_rows)
    verdicts = {}
    for report_path in report_paths:
        with open(report_path, encoding='utf-8') as handle:
            for cells in table_rows_under('## 逐行判定', handle.read()):
                if len(cells) >= 3:
                    verdicts[(cells[0], cells[1])] = (cells[2], cells[3] if len(cells) > 3 else '')
    missing = [key for key in keys if key not in verdicts]
    malformed = []
    for key in keys:
        if key not in verdicts:
            continue
        verdict, reason = verdicts[key]
        if (not verdict.startswith(LINE_VERDICTS) or len(reason) < MINIMUM_REASON_LENGTH
                or not quotes_the_line(reason, text_by_key.get(key, ''))):
            malformed.append(key)
    if missing or malformed:
        if missing:
            print(f'  ✗ {len(missing)} 行候选没有逐行判定，例：'  # gate-lint:detail
                  + '；'.join(f'{group} {location}' for group, location in missing[:10]))
        if malformed:
            print(f'  ✗ {len(malformed)} 行的判定不以五种之一开头、最后一格不到 {MINIMUM_REASON_LENGTH} 个字、'  # gate-lint:detail
                  f'或最后一格没引这一行至少 {MINIMUM_QUOTED_CHARACTERS} 个字的原话，例：'
                  + '；'.join(f'{group} {location}' for group, location in malformed[:10]))
        print(f'  ✗ 报告没判全：共 {len(keys)} 行候选')  # gate-lint:summary
        print('     → 怎么办：在「## 逐行判定」表里给每一行候选一行 | 组 | 载体 | 判定 | 改后的句子或理由 |，'
              '判定以 要改 / 要补 / 事件句不改 / 不相干 / 要人看 开头，最后一格写改后的句子或理由，理由里引这一行的原话')
        return 3
    print(f'  ✓ 报告判全了：{len(keys)} 行候选都有逐行判定')
    return 0


def benchmark(repository_root):
    facts_path = os.path.join(repository_root, BENCHMARK_FACTS)
    if not os.path.isfile(facts_path):
        print(f'  ✗ 缺阳性对照的事实表 {BENCHMARK_FACTS}')
        print('     → 怎么办：它随仓走；被删了就从 git 历史里取回，不要照着答案重写')
        return 4
    facts = read_fact_table(facts_path)
    candidates = build_candidates(repository_root, BENCHMARK_TARGET, facts)
    covered = {location_hash(location) for _, _, _, location, _ in candidates}
    covered_known = sum(1 for known in BENCHMARK_KNOWN_STALE_LOCATION_HASHES if known in covered)
    unique_lines = len({location for _, _, _, location, _ in candidates})
    if (len(candidates), unique_lines) != (BENCHMARK_EXPECTED_CANDIDATE_ROWS, BENCHMARK_EXPECTED_UNIQUE_LINES):
        print(f'  ✗ 阳性对照的候选数变了：{len(candidates)} 行、{unique_lines} 个不同的行，冻结时是 '
              f'{BENCHMARK_EXPECTED_CANDIDATE_ROWS} 行、{BENCHMARK_EXPECTED_UNIQUE_LINES} 个不同的行')
        print('     → 怎么办：同一份事实表、同一段历史，候选只该随候选规则变；是有意改了规则就重数并改这两个常量，写明为什么变')
        return 4
    if covered_known < BENCHMARK_MINIMUM_COVERED:
        print(f'  ✗ 阳性对照只罩住 {covered_known} 处已知的腐烂，低于 {BENCHMARK_MINIMUM_COVERED}（{BENCHMARK_BASE}..{BENCHMARK_TARGET}，只存哈希）')
        print('     → 怎么办：候选规则或盲写的事实表被改窄了；看 build_candidates 与 current_state_corpus 的改动，'
              '或那份事实表的检索词是不是被收窄过')
        return 4
    print(f'  ✓ 阳性对照：已知的 {len(BENCHMARK_KNOWN_STALE_LOCATION_HASHES)} 处腐烂里 {covered_known} 处在候选里（下限 {BENCHMARK_MINIMUM_COVERED}）'
          f'（盲写事实表 {len(facts)} 行，候选 {len(candidates)} 行、{unique_lines} 个不同的行）')
    return 0


def selftest():
    with tempfile.TemporaryDirectory() as directory:
        def git(*arguments):
            subprocess.run(['git', '-C', directory, *arguments], check=True, capture_output=True)

        def write(relative_path, content):
            full_path = os.path.join(directory, relative_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, 'w', encoding='utf-8') as handle:
                handle.write(content)

        def quietly(function, *arguments):
            standard_output = sys.stdout
            with open(os.devnull, 'w') as silent:
                sys.stdout = silent
                try:
                    return function(*arguments)
                finally:
                    sys.stdout = standard_output

        git('init', '-q')
        git('config', 'user.email', 'selftest@example.invalid')
        git('config', 'user.name', 'selftest')
        write('CLAUDE.md', '# 项目\n\n层 0 的负载还只有第一个事务。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('.claude/kb/table-before-move.md', '# 字节表\n\n' + '字段一行。\n' * 20 + '\n## 历史版本\n\n### 2026-08-30：搬家之前就有的条目\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'base')
        os.makedirs(os.path.join(directory, '.claude/kb/layout'), exist_ok=True)
        os.rename(os.path.join(directory, '.claude/kb/table-before-move.md'),
                  os.path.join(directory, '.claude/kb/layout/01-first-txn.md'))
        write('CLAUDE.md', '# 项目\n\n层 0 的负载是两条流。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-02：层 0 扩成两条流\n\n### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('records/2026-01-01-log.md', '层 0 只有一次挂载的时候写的记录。\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'stage')
        write('records/2026-01-02-draft.md', '层 0 只有一次挂载（还没进 git 的草稿）。\n')
        failures = []
        entries = new_change_entries(directory, 'HEAD~1', 'HEAD')
        if [heading for _, _, heading in entries] != ['### 2026-09-02：层 0 扩成两条流']:
            failures.append(f'新增变更记录没认对（改名的文件要按旧路径比）：{entries}')
        segments = changed_segments(read_version(directory, 'HEAD~1', 'CLAUDE.md'), read_version(directory, 'HEAD', 'CLAUDE.md'))
        if not any('只有第一个事务' in old for old, _ in segments):
            failures.append('差异片段里没有被改掉的「只有第一个事务」')
        header = '\t'.join(FACT_TABLE_COLUMNS) + '\n'
        good_row = 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0\tH1\n'
        red_tables = {
            'facts-uncovered.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0\t提交 abc\n', '漏了 H1'),
            'facts-empty.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t不存在的词\tH1\n', '检索词零命中'),
            'facts-new-wording.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t两条流\tH1\n', '检索词照抄新说法（基准零命中）'),
            'facts-gone.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t负载还只有\tH1\n', '检索词照抄被改掉的旧句（结束零命中）'),
            'facts-fake-new.tsv': (header + good_row + 'F2\t新立：层 0\t层 0 两条流\t层 0\tH1\n', '冒充新立（基准有命中）'),
            'facts-fake-new-words.tsv': (header + good_row + 'F2\t新立：两条流\t层 0 两条流落地\t两条流\tH1\n', '冒充新立（新事实写着落地）'),
            'facts-fake-rewording.tsv': (header + good_row + 'F2\t只改措辞\t只改措辞\t层 0\tH1\n', '冒充只改措辞（变更标题看不出改措辞）'),
            'facts-needless-and.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0&&一次挂载\tH1\n', '第一个词不宽却用了 &&'),
        }
        write('facts-good.tsv', header + good_row)
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 0:
            failures.append('写对了的事实表被判红')
        for name, (content, what) in red_tables.items():
            write(name, content)
            if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, name)) != 6:
                failures.append(f'{what}的事实表没被判红')
        saved_limit = globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT']
        globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT'] = 0
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 6:
            failures.append('命中行数超上限的事实表没被判红')
        globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT'] = saved_limit
        write('facts-build.tsv', header + good_row + 'F2\t只改措辞\t只改措辞\t层 0\tH1\n'
              'F3\t层 0 只有第一个事务\t层 0 两条流\t一次挂载\tH1\n')
        candidates = build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-build.tsv')))
        table_path = os.path.join(directory, 'candidates.tsv')
        with open(table_path, 'w', encoding='utf-8') as handle:
            write_candidates(candidates, handle)
        written = [key for key, _ in read_candidate_rows(table_path)]
        if ('F1', '.claude/kb/checks-owed.md:1') not in written:
            failures.append('还说「层 0 只有一次挂载」的现状句没进候选')
        if ('F1', 'records/2026-01-01-log.md:1') not in written:
            failures.append('records/ 下的现状句没写进候选表')
        if [location for group, location in written].count('.claude/kb/checks-owed.md:1') != 2:
            failures.append('同一行命中两件事实，候选表里没有各占一行')
        if any(group == 'F2' for group, _ in written):
            failures.append('只改措辞的行出了候选')
        if any(location.startswith('.claude/kb/checks-owed.md:') and location != '.claude/kb/checks-owed.md:1'
               for _, location in written):
            failures.append('历史版本节里的句子进了候选')
        command_output = os.path.join(directory, 'candidates-from-command.tsv')
        subprocess.run([sys.executable, os.path.abspath(__file__), '--facts', os.path.join(directory, 'facts-build.tsv'),
                        '--base', 'HEAD~1', '--target', 'HEAD', '--out', command_output],
                       cwd=directory, capture_output=True, text=True)
        if not os.path.isfile(command_output) or read_candidate_rows(command_output) != read_candidate_rows(table_path):
            failures.append('--facts 命令写出的候选表与 build_candidates 的结果不一致')
        worktree_locations = [row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts-good.tsv')))]
        if 'records/2026-01-02-draft.md:1' not in worktree_locations:
            failures.append('结束在工作区时，没进 git 的新文件没进候选')
        if parse_group_ranges('F1-F2') != {'F1', 'F2'}:
            failures.append('组号区间没把两端都算进去')
        for bad_range in ('F2-F1', 'F1-G2'):
            try:
                parse_group_ranges(bad_range)
                failures.append(f'组号区间 {bad_range} 没被拒绝')
            except ValueError:
                pass
        rows = read_candidate_rows(table_path)
        def report(verdict, reason_of):
            return ('## 逐行判定\n| 组 | 载体 | 判定 | 改后的句子或理由 |\n|---|---|---|---|\n'
                    + ''.join(f'| {group} | {location} | {verdict} | {reason_of(line)} |\n' for (group, location), line in rows))
        write('complete.md', report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成层 0 的负载是两条流'))
        write('missing.md', report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成两条流').rstrip('\n').rsplit('\n', 1)[0] + '\n')
        write('unknown-verdict.md', report('同意', lambda line: '原话「' + line.replace('|', ' ') + '」照旧'))
        write('short-reason.md', report('不相干', lambda line: normalize(line)[:7]))
        write('no-quote.md', report('不相干', lambda line: '这一行说的是别的事情，与这件事实无关，不用改'))
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')]) != 0:
            failures.append('判全了的报告被判漏')
        for name, what in (('missing.md', '少判一行'), ('unknown-verdict.md', '判定不是五种之一'),
                           ('short-reason.md', '理由只有 7 个字'), ('no-quote.md', '理由没引这一行的原话')):
            if quietly(check_reports, table_path, [os.path.join(directory, name)]) != 3:
                failures.append(f'{what}的报告没被判红')
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')], {'F9'}) != 2:
            failures.append('--groups 里写了候选表没有的组，没被拒绝')
    if failures:
        for failure in failures:
            print(f'  ✗ 自证失败：{failure}')  # gate-lint:detail
        print('  ✗ stale-candidates 自证没过')  # gate-lint:summary
        print('     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、'
              'build_candidates、check_reports 哪一支被改了')
        return 5
    print('  ✓ stale-candidates 自证通过：变更记录（含改名）与差异片段认对；事实表八种写法各判红、写对的判绿、超上限判红；'
          '候选收 records/ 与没进 git 的文件、同一行两件事实各占一行、只改措辞不出候选、历史版本节不进；'
          '组号区间含两端、倒写与跨字母拒绝、写了不存在的组拒绝；报告漏判、判定不认得、理由太短、理由没引原话都判红，--facts 命令写出的表与函数结果一致（27 格）')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--base')
    parser.add_argument('--target')
    parser.add_argument('--out')
    parser.add_argument('--changes', action='store_true')
    parser.add_argument('--check-facts', metavar='事实表')
    parser.add_argument('--facts', metavar='事实表')
    parser.add_argument('--check-report', nargs='+', metavar='文件')
    parser.add_argument('--groups')
    parser.add_argument('--benchmark', action='store_true')
    parser.add_argument('--selftest', action='store_true')
    arguments = parser.parse_args()
    if arguments.selftest:
        return selftest()
    try:
        repository_root = run_git(['rev-parse', '--show-toplevel'], os.getcwd()).strip()
        if arguments.benchmark:
            return benchmark(repository_root)
        if arguments.check_report:
            if len(arguments.check_report) < 2:
                print('  ✗ --check-report 要候选表与至少一份报告')
                print('     → 怎么办：stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]')
                return 2
            return check_reports(arguments.check_report[0], arguments.check_report[1:], parse_group_ranges(arguments.groups))
        if not arguments.base:
            print('  ✗ 缺 --base')
            print('     → 怎么办：给阶段开始之前的提交，例：--base b1c8cef~1；用法见本脚本文件头')
            return 2
        if arguments.check_facts:
            return check_facts(repository_root, arguments.base, arguments.target, arguments.check_facts)
        if not arguments.out:
            print('  ✗ 缺 --out')
            print('     → 怎么办：--changes 与 --facts 都要 --out 指一个新文件（排他新建）')
            return 2
        if not arguments.changes and not arguments.facts:
            print('  ✗ 缺 --changes 或 --facts')
            print('     → 怎么办：先 --changes 出变更清单、写事实表、--check-facts 过了，再 --facts 出候选表')
            return 2
        with open(arguments.out, 'x', encoding='utf-8') as output:
            if arguments.changes:
                write_changes(repository_root, arguments.base, arguments.target, output)
                entry_total = len(new_change_entries(repository_root, arguments.base, arguments.target))
                print(f'  ✓ 变更清单 {arguments.out}：{entry_total} 条变更记录')
                return 0
            candidates = build_candidates(repository_root, arguments.target, read_fact_table(arguments.facts))
            write_candidates(candidates, output)
        print(f'  ✓ 候选表 {arguments.out}：{len(candidates)} 行（{len({row[3] for row in candidates})} 个不同的行）')
        return 0
    except (RuntimeError, ValueError) as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：确认 --base / --target 是这个仓里存在的提交、事实表格式照文件头')
        return 2


if __name__ == '__main__':
    sys.exit(main())
```

**出处 `.claude/kb/checks-owed.md:415-415`（整段抄，未转述）**

```markdown
| C468 | 判错的原始证据不落盘 | **阶段同步判错一行之后，能拿来复盘的东西一个字节都没有**：触发「逐行全看」那条规则的那一处判错（`.claude/kb/decisions/06-快照实现模型.md:42`），当时的候选表与两份逐行判定报告都在 `/tmp` 下、没入库，仓里今天那两处解释射程的文字是**事后补写的**——`git log -S` 现查，`decisions/06-快照实现模型.md:42` 那段与 `.claude/kb/checks-owed.md` 里 C12（增量语义共用） 那一行的 crate 粒度描述，都只出自提交 `11a551b`，在判错发生的那一刻两处都不存在。⇒ 想复盘「当时看得出来吗」，没有任何原始材料可查，只能靠事后重建，而重建的人正是当时判错的人 | 阶段同步的候选表与逐行判定报告跟着同步记录一起入库（今天它们落在 `/tmp/claude-1000/owed-sync/` 下，`research/prompts/` 里只有同步记录本身）；门禁形态：同步记录点名的候选表与判定报告要在仓里现存、行数与记录里报的数对得上，指不到即判红。判别力自证：把候选表从仓里挪走，必须由绿转红。⚠️ 与 C465（门禁自己的并发让门禁里的性能断言变红） 不同族——那一条是门禁互相干扰，这一条是证据没留 | 无前置（候选表与判定报告今天就在跑，只是没入库） | 2026-09-21 `research/prompts/agentdef-r2-main-verification.md` 第四节，云端攻方报的 G 格，主 agent 用 `git log -S` 逐串现查坐实 |
```

**出处 `research/prompts/agentdef-r2-opus-output.md:286-300`（整段抄，未转述）**

````markdown
⇒ **四个改法没有一个修这一格**：改法一记的是「抽到哪几行」和重判结论，而 06:42 **正是抽样没抽到的那一行**（`sync.md:51` 逐字：「抽样这一次漏掉了 06:42 那一处判错」）——按定义它进不了那张表；改法二改截断、改法三改共享条文、改法四改判别子，三个都不产生任何关于「判错」的记录。

这一格是**可落成会失败的检查**的，这是它比 D、F 更该被看一眼的地方：要求同步记录里另有一张「复核改判」表（载体、原判定、改判后的判定、抓到它靠的是哪一件东西），门禁 68 号能机械核（载体格式、两个判定都是五种之一、两者不相等、末格非空），而且它在**0 处判错**的那一轮可以是 0 行表——与现有 ④「表可以 0 行」同形。它同时给 D 格提供了改法一提供不了的那件东西：一条「全看」独有的产物（抽样抽不到的行上的改判，只可能来自全看）。
⚠️ 这是**我这一条腿自己提的收严，被攻过零轮**，与第一轮那四个同级，按 `.claude/rules/three-way-inference.md:122`「攻方腿自己提的收严，只在它自己的模型上量过，算「没被攻过」」处理；而且按同一份规则「第三轮之后停」那一段，它是这一轮新冒出来的零轮形态。

### H 格（次要，同样四个都没修）：候选表本身的漏召回

四个改法管的全是「已经进了候选表那 138 行怎么判」。而 `research/scripts/stale-candidates.py:35-37`（整段抄，工具作者自己写在「管不到的」一节里）：

```
管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上）：
  写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地 / 实现这类字，过得了闸，
  这件事实的旧说法一行都进不了候选——判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表。
```

````

**出处 `research/scripts/stale-candidates.py:33-40`（整段抄，未转述）**

```markdown
  最后一格不能空：要改的写改后的句子，其余写理由（至少 8 个字）。几份报告合起来判。

管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上）：
  写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地 / 实现这类字，过得了闸，
  这件事实的旧说法一行都进不了候选——判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表。
  逐行判的理由引了原话、判定也是五种之一，照样可以是没读懂就判的；靠主 agent 在全部判定里抽查复判。

退出码：0 通过；2 参数或 git 出错；3 报告漏判或判定格式不对；4 阳性对照漏了已知的腐烂；5 自证失败；6 事实表没罩全。
```
