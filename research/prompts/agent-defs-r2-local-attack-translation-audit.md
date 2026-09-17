# 核对表：agent-defs-r2 本地攻方提示的逐句翻译核对

对照 `research/prompts/agent-defs-r2-local-attack.md`。四列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。
「首稿缺的」记的是第一次写出来之后、逐句核对原文时发现的问题（缺限定词、归属错误、事实错误、编号不一致），不是空话；没问题的行写「无」。

## 五个问题（原文出自主 agent 派发提示「## 输入」第 4 项，不是仓库文件，按问题编号引用）

| 英文项（提示文件行号） | 原文（派发提示第 4 项，问题号） | 首稿缺的 | 定稿 |
|---|---|---|---|
| Question one（第 31 行）：构造一个坏写范围表项或坏定义措辞，在仓副本里试跑判可用、在真仓上越界写或被拒到做不下去 | 问题 1：「写出一个具体的坏定义或坏写范围表，它在这种试跑里照样被判可用，而在真仓上会越界写或被闸拒到做不下去」 | 无：「越界写」与「被闸拒到做不下去」两个分句都保留，「具体」译成 be concrete 并要求点名确切的坏模式/坏措辞 | 同首稿 |
| Question two（第 33 行）：构造一个定义缺陷，一次试跑有很大机会碰不到，说明第二次试跑要什么样 | 问题 2：「写出一个定义缺陷，一次试跑有很大机会碰不到它，并说明要什么样的第二次试跑才碰得到」 | 无 | 同首稿 |
| Question three（第 35 行）：构造一个因规格同一人写而不会触发的缺陷，举例（缺输入就停的路从没走过），指出现有试跑里哪一次正是这样 | 问题 3：「写出一个定义缺陷，正因为规格是同一个人写的而不会被触发（例：规格恰好给全了定义要的输入，于是「缺输入就停」那条路从没走过），并指出现有试跑里哪一次就是这样」 | 首稿把「现有试跑里哪一次就是这样」指向的背景（原 Fact eight/nine）本身写错了（见下表「背景事实」的修正记录）；问题句本身翻译时没有遗漏限定词 | 问题句文字不变；改的是它所指向的背景事实（facts eight–eleven），把回指从「fact eight and fact nine」改成「facts eight, nine, ten, and eleven」 |
| Question four（第 37 行）：构造一种判决偏差会让错改法被采纳，D1 可当例子但要给出它之外的可达情形 | 问题 4：「写出一种判决方式上的偏差，会让一条错的改法被采纳；第一轮 D1（……）可以当例子，但要给出它之外的另一个可达情形」 | 首稿曾只写「different kind of bias」，核对时发现原文「但要给出它之外的另一个可达情形」里的「可达」（reachable）必须保留，否则模型可能给一个纯假设、不可达的情形 | 补上 reachable：「construct one additional, concrete, reachable scenario, different from the scenario in fact six」 |
| Question five（第 39 行）：列出定义里有、试跑一次都没走到的路径，例子给三种，每条写漏测会漏掉什么错 | 问题 5：「列出定义里有、而这些试跑一次都没走到的路径（例：干到一半收到主 agent 的消息、交回被拒第二次、与别的会话同时改同一个文件），每条写它不走到会漏掉什么错」 | 「与别的会话同时改同一个文件」里的「别的会话」被改写成「two different roles, or two different running instances of the same role」——这是为了自足（提示里没有定义「会话」这个词）而做的改写，不是漏译；已在此列注明 | 保留改写，本表记录改写理由；语义没有收窄（提示里三个有写权限的定义本来就只可能被别的角色实例并发写同一文件） |

## 背景事实（Fact 一至十一）

| 英文项（提示文件行号） | 原文（文件:行） | 首稿缺的 / 首稿错的 | 定稿 |
|---|---|---|---|
| Fact one（第 7 行）：16 个定义，12 个只试跑一次，4 个试跑两次，没有定义试跑超过两次 | `records/2026-09-16-subagent拆分提案.md:403`「改动最多的四个定义各拿一件一次性小活再试跑一次」；总数 16 见 `_agent-defs-r2-body.md:12`；「没试跑超过两次」见 `records/...md:430`「这些改法被攻过零轮，第二轮对抗还没派」 | 「十二」是我自己用 16－4 算出来的，原文没有一句直接写「十二」；不是误译，是推导，已在此列注明 | 保留，因为算术是从原文两个数直接推出、可核 |
| Fact two（第 9 行）：写范围闸只拦 Write/Edit，Bash 里的写它看不见 | `.claude/agent-common.md:14`「写范围闸……只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守」；`.claude/hooks/agent-write-scope.sh:11`「拦不住 Bash 里的写」 | 无 | 同首稿 |
| Fact three（第 11 行）：三个定义登记了写范围，各自都有 `/tmp/claude-1000/**` 这条通配 | `.claude/hooks/agent-write-scope.tsv:4,6,7,8,18` | 无 | 同首稿 |
| Fact four（第 13 行）：kb-scribe 三次 Edit 测试，副本里两次放行（含副本的 CLAUDE.md）、真仓一次拒 | `_agent-defs-r2-body.md:30`「对 kb-scribe 各喂一条 Edit 输入，目标分别是副本的 .claude/kb/decisions/22-x.md、副本的 CLAUDE.md、真仓的 CLAUDE.md：退出码 0、0、2」 | 无 | 同首稿 |
| Fact five（第 15 行）：试跑的活与读数评判都由同一个主 agent 做，没有独立第三方 | `_agent-defs-r2-body.md:93`「试跑的活与规格由写定义的主 agent 出、由主 agent 自己核读数」 | 「没有独立第三方」是我自己加的总结句，原文没有逐字这么写，是从「主 agent 出规格 + 主 agent 自己核」直接推出的合理推论，已在此列注明 | 保留 |
| Fact six（第 17 行）：D1「在仓根下跑 mutate.sh」被采纳、第二次试跑才暴露是错的 | `agent-defs-r1-main-verification.md:37`；`records/...md:408`「第一轮 D1 写进定义的『在仓根下跑 mutate.sh』是错的……第一轮 mutation-triage 试跑本来就是在 research/ 下跑的，主 agent 抄的时候没核」 | 无重大遗漏；「仓根的 Cargo.toml 也有 [workspace]，目录检查照样放过」这句抽象成了「the top level directory also happens to satisfy that same check for an unrelated reason」，为了不用先定义 Cargo.toml/workspace 这两个术语，属于为自足性做的抽象，不是丢限定词 | 保留抽象写法 |
| Fact seven（第 19 行）：判决由同一个主 agent 做，不是投票 | `.claude/rules/three-way-inference.md:187-189`「判决由主 agent 做，不由投票做……不一致 → 先核实，再讨论。核实指去查能分辨它们的那个事实，不是再加一个模型来投票。分歧点本身要写进结论里」 | 首稿丢了「先核实，再讨论」的「再讨论」与「分歧点本身要写进结论里」两个分句。核对后判断：这两个分句是文档撰写纪律（怎么记录分歧），与本题要问的「判决机制本身的偏差」不是同一件事，不影响 Question four 要问的内容，故未补入正文，但在此列如实记录被舍弃的两个分句，供主 agent 核查 | 保留不补；如实记录 |
| Fact eight（第 21 行）：implementation-writer 两次试跑（故意留空隙被发现停下；文件被标成别人在改而避开） | `records/...md:336,343`（第一次，「条款 4 故意没写」「停了两处，写成 todo!」）；`records/...md:407`（第二次，「lib.rs 列成『别的会话正在改』」「lib.rs 没动」） | **首稿曾把这两条试跑之外，凭空多写一条「experiment-runner 某次试跑：复现已有实验、全部检查通过」，把它错误地当成一条独立的、干净通过的试跑，与 D1 那次（同样是 experiment-runner 复跑 E67）混在一起，造成读者以为有两次不同的复现实验试跑。经核对，experiment-runner 只有两次试跑：第一次是新建 E153（不是复现已有实验），第二次是复跑 E67（就是 D1 暴露的那一次），没有第三次「复现已有实验、全部通过」的独立试跑。这是一处事实错误，不是遗漏限定词** | 删除了那条凭空捏造的子例，改成如实的两条 implementation-writer 试跑；D1 那次不在这里重复，只在 Fact six 里出现一次 |
| Fact nine（第 23 行）：mutation-triage 复跑 E67 变异表，主 agent 给错 bin 名（下划线），角色靠脚本自己的报错信息改对，结果与已知答案一致 | `records/...md:381`「mutation-triage 试跑：主 agent 派发时 bin 名给错（下划线），它靠 mutate.sh 的核对改对；抓到 3、无效 1、没红 0，与已知答案一致」 | 首稿没有这一条：这是核对阶段新补的，用来替换上面被删掉的那条错误子例，确保 Fact 里确实有一条「主 agent 规格有错但角色没有走『缺输入就停』那条路，而是用别的方式自己纠正」的真实先例 | 新增为独立一条 Fact |
| Fact ten（第 25 行）：kb-scribe 同一次试跑里两个现象——CLAUDE.md 那句被拒、状态句与字节句缺失导致门禁红 | `records/...md:410`「kb-scribe：副本里把 D22 第 7 项翻回未定，规格里夹一条写 CLAUDE.md 的……出界那条停住了；20、31 号红是主 agent 的规格少给了状态句与『改不改第一个事务的字节』」 | **首稿把这两个现象拆成了「一次试跑」与「另一次、不同时间的试跑」，写成 In a different trial run of that same knowledge base writing role, at a different time——这是事实错误：两个现象出自同一次试跑，不是两次。另外首稿加了一句未经原文证实的「specifically in order to test whether the role would correctly recognize...」，把主 agent 加那条 CLAUDE.md 分句的意图说成『特意为了测试』，原文只写『规格里夹一条』，没有说明是不是故意——这是替原文加了一个原文没给的限定词（意图），需要去掉或改成不确定的说法** | 改成同一次试跑的两个方面（用「Separately, in that very same trial」衔接）；意图那句改成中性描述（不再断言「specifically in order to test」），并加一句依据 Fact four 指出「这条边界这个网关本来就不会拦」，把「角色自己认出越界」与「网关本身没测到」区分清楚 |
| Fact eleven（第 27 行，原 Fact nine）：gate-triage 试跑，主 agent 给了文件清单，角色用它正确分归属 | `records/...md:383`「gate-triage……空暂存区时按主 agent 给的文件清单判归属」 | 无 | 同首稿，仅编号从 Fact nine 改成 Fact eleven |

## 结构性修正（不是某一句翻译，是发现之后删除/改动的整段）

| 问题 | 发现方式 | 处理 |
|---|---|---|
| 首稿多写了一条「Fact ten」，内容是本项目另一份文档（`.claude/rules/three-way-inference.md` 描述的「同一人写摘要、同一人复核摘要，两次都没查出漏掉的分句」那类事故），但这一轮派给本地攻方的五个问题一条都没有用到它——它是我误把 r1 提示里 Question six 用到的背景材料带了过来。核对每条 Fact 有没有被某个 Question 引用时发现它悬空 | 全文 grep「fact ten」确认没有任何 Question 指向它 | 删除整条 Fact，用 `replace-once.py` 定点删掉，前后空行收拢为一行；同时把开头「I will give you ten/nine numbered facts」改成与实际条数一致的「eleven」 |
| Fact 编号与「Now answer」之间一度出现双空行（删除 Fact ten 之后的残留） | 逐行核对文件 | 用 `replace-once.py` 把三个换行收成两个 |

## 未发现问题、判定「无」的项

其余没有在上表单独列出的英文句子（Fact two、three、four、five、six、seven、eleven 与五个问题里未特别标注的部分）逐句与原文对照后未发现遗漏限定词或事实错误。
