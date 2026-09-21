<!-- knowledge-sync -->
# C224 动不动格式判定 阶段同步

按 `.claude/kb/checks-owed.md` 欠着那张表的顺序往下找前置已备齐的一批，做成的事：还掉 C224（未定分项没有「动不动格式」的判定） 的写判定那一半——给全仓 5 条未定项各判一次并写进登记行，判定那道检查不另立阶段、并进门禁 31 号当第二把尺；另把 C48（多条校验路径共用同一个前提）、C49（全称断言只在抽样点验过）、C50（阻塞标记没人维护） 三条这一轮评估下来为什么不做写进各自的欠账行。

触发文件：.claude/main-agent.md、.claude/gate.d/31-blocking-verdict.sh、.claude/gate.d/75-decision-experiment-links.sh、.claude/agents/kb-scribe.md、.claude/gate.d/stage-owners.tsv、.claude/gate.d/68-knowledge-sync.sh、research/scripts/archive-past-rounds.py

## 搜索

按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「回扫要按旧说法搜，不能只按新名字搜」，先写出这一阶段做成之前仓里会怎么说它，再拿这些说法全仓搜。

- `grep -rn '31-blocking-verdict\|31 号'`（排掉 SOP 副本与 31 号自己）→ 16 行命中：现状句 6 行，其余住在 `.claude/kb/decisions-history/`、`records/` 里说的是那一天什么样，不改。
- `grep -rn '动不动格式'`（排掉 SOP 副本与判别力样本）→ 21 行命中：现状句 6 行（D15（格式冻结政策） 正文 4 行、5 条未定项的登记行），其余在历史文件里。
- `grep -rn '99-format-impact-verdict'` → 合并之前立过的那道阶段，删掉之后全仓 0 命中（`.claude/kb/checks-owed.md` 里两处指着它的话当场改成指 31 号）。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/rules/format-evolution.md:28 | 未定项的登记行里还要带「改第一个事务的字节：否（YYYY-MM-DD，依据：…）」（31 号要它写在登记行里…） | 改了：31 号现在是两把尺，规则只说了第一把。改成两句判定都要，并写明两把尺量的不是一个集合、不许拿一句的结论填另一句 |
| .claude/main-agent.md:43 | 再从全部判定里随机抽 20 行（五种判定各至少 2 行，不够的全抽）自己复判 | 改了：用户定案，阶段同步的判定主 agent 逐行全看、不抽样。⚠️ 这一处**走过三方而没有定死**：云端攻方六格打中五格，最重的两格是「规则措辞治不了它自己的触发案例」（那次判错的判别子在门禁 94 号的射程里，载体原文一个字看不出来）与「改前那句可机械核、改后这句不能」，判决 `research/prompts/agentdef-r1-main-verification.md` 第三节把它连同攻方的四个改法一起留到第二轮，攻击面已在判决里写死 |
| .claude/agents/kb-scribe.md:20 | 翻成未定的判一句改不改第一个事务的字节（31 号） | 改了：两句都要，缺一句就红。按门禁 72 号走过一轮三方（`research/prompts/agentdef-r1-main-verification.md`），三条腿都没打中它本身；本地攻方报的三种「会让两把尺混成一把」的写法经主 agent 造样本仓实测，门禁 31 号三条全判红，记它没打中 |
| .claude/kb/decisions/15-格式冻结政策.md:118 | 第 7 项扫全仓未定分项里那句「动不动格式」的判定，形态与门禁阶段…同构，换一个正则、换一个触发时机 | 改了：那是另立一道阶段的设想，实况是并进 31 号当第二把尺。改成拆两半——写没写那一半常跑，判红那一半仍在冻结请求发起时，前置是 C225（组件的冻结时刻绕开门禁触发） |
| .claude/gate.d/75-decision-experiment-links.sh:240 | without_marks = re.sub(r'改第一个事务的字节：…', '', cell) | 改了：它只剥掉第一把尺那句，而两句都必须带日期。不改的话新写的 5 句判定会让 75 号把 5 行定案格判成「带日期」——这一处是回扫抓到的，不在预想里 |
| research/scripts/archive-past-rounds.py:41 | dirty = set(… git diff --name-only "HEAD" …) | 改了：门禁 91 号写死 HEAD 当「这一轮」的起点，而门禁 68 号取 GATE_BASE（跑时是 HEAD~1）⇒ 每次提交之后上一轮的同步记录必然同时被 91 号判「该删」、被 68 号判「要留」。改成同一个基准（新的 base_of 取 GATE_BASE，没给才退 HEAD），自证加一格：造出「上一轮的记录已进仓、它点名的文件相对 HEAD~1 还在范围里」那个场景，写死 HEAD 必须红 |
| .claude/gate.d/68-knowledge-sync.sh:211 | 处置写「改了」「补了」而载体不在改动范围里 | 改了：那条判据假设同步记录与改动范围同期，而跨轮留存的旧记录点名的载体是更早一轮改的，相对今天的基准当然不在范围里——那些行说的都是真话，改写它们才是假的。加一道前置：只对这一轮**新写**的记录（相对基准是新增）判处置行，被追加过说明的旧记录不判 |
| .claude/gate.d/stage-owners.tsv:14 | 31-blocking-verdict.sh	kb-scribe	未定项判没判过挡不挡第一个事务 | 改了：31 号现在判两把尺，那一行只说了第一把；同一次把并进来之前短暂立过的那道阶段从登记表里摘掉 |
| .claude/kb/decisions.md:46 | D15（格式冻结政策） 已定 11 项 / 未定 0 项 | 不改：这一轮一条分项都没翻状态，只在 D2、D22、D26 的未定项登记行里各加一句判定 |
| records/2026-09-09-D15两格定案.md:48 | C224（未定分项没有「动不动格式」的判定，18 条一条都没写过） | 不改：它断言的是 2026-09-09 那天仓里什么样，改了反而成假话（`.claude/rules/path-moves.md` 第 5 条同形）；`.claude/kb/decisions-history/2026-09.md` 里的同类旧说法同理 |

## 这一轮顺带抓到、记成账的

门禁自己的并发让门禁里的性能断言变红：59 号（crates 变异表复跑）开 8 个工作进程吃满核，而同一道门禁里「research 构建与单测」跑的实验单测带着按挂钟判的断言。实测两次全量门禁各红一个、且是不同的两个——`research/e7-index-bench/src/bin/e6_multicore.rs:215`「4 线程做同样总量应当更快」（那一刻 load average 35.7）与 `research/e7-index-bench/src/bin/e44_jsn_width.rs:269`「不 fdatasync 该至少快一倍」，各自单独复跑 5 轮全过。全仓这类断言就这 2 处，两次各中一个。记成 C465（门禁自己的并发让门禁里的性能断言变红），这一轮不修——两条断言与 59 号都不是这一批碰过的东西。

## 另外做成的

- 变更史补 `2026-09-21（其三）`：五条判定各带依据，以及第 7 项可执行形态的改前 / 改后。门禁 30 号由红转绿，49 号 `--write` 重新生成，471 条条目、0 条待补快查。
- 判别力：31 号两把尺各自分得出——red 那份第 3 条两把尺都没写、第 4 条只缺「动不动格式」，两条各被对应那把尺点名；green 两条都写齐判绿。阶段自检 116 个样本判得都对。
- 不另立阶段的依据：99 号与 31 号的对象（同一批未定项）、权威清单（`.claude/scripts/gen-decision-items.py`）、登记行定位、归属（`kb-scribe`）全都相同，正文只差一条正则，手抄一份会分叉（`.claude/singlefs-ai-sop/rules/code-discipline.md`「重复要生成，不许手抄」）。门禁阶段数因此仍是 66。
