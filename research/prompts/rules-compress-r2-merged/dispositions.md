# 十份腿报告逐条去向（压缩 2c）

写法：「题·腿」里 攻 = opus 攻方腿，正 = sonnet 正推腿。「成立」指现核原件与脚本文件头之后这一条站得住。改在栏的 文件:行 都是 `revised/` 下的（`dot-claude/` 即仓里的 `.claude/`）。腿提的改法只当线索，句子都照原件或脚本文件头的原话补回。两条腿说同一件事的并成一行。

## 一、入口题（entry）

| 题·腿 | 编号 | 一句话 | 成立 | 改在 / 不改的理由 |
|---|---|---|---|---|
| 入口·攻 + 正 | 攻 丢-1、正 观察（M226） | 主 agent 收事实表的验收句 `stale-candidates.py --check-facts` 没有落点 | 成立（原件 `.claude/main-agent.md:121` 有这句，合稿树里只剩回扫员定义自己跑） | `dot-claude/main-agent.md:95`、`dot-claude/skills/dispatch/SKILL.md:75` |
| 入口·攻 + 正 | 攻 义-1 与 驻-1；正 三 | 技能「读一次」「第一次派发」与压缩上下文接着做互相打架 | 成立 | 照主 agent 定的写法：`dot-claude/main-agent.md:5`（每次开始验证段之前、每次派发之前读；正文不在上下文里就整份重读，压缩之后重读）；两份技能 frontmatter 的 description 同改（`dot-claude/skills/merge-verify/SKILL.md:3`、`dot-claude/skills/dispatch/SKILL.md:3`） |
| 入口·攻 | 义-2 | 「实现员不登记，主 agent 也不替它登记」主语写死后读得出「主 agent 不登任何行」 | 成立（原件主语本来就空，合稿补成主 agent 后与「收的行做成拒算补丁目录」相抵） | `dot-claude/main-agent.md:68`：实现员不往拒算表里写，主 agent 也不往哪一棵树里手写，收的行只走拒算补丁目录 |
| 入口·攻 + 正 | 攻 义-3；正 观察 | 「整批重交」的统一定义套到第 1 步红支与第 3 步退非 0，读得出两种 | 成立（原件 `.claude/main-agent.md:57`、`:59` 两处写法不同，合稿并成一句丢了区别） | `dot-claude/skills/merge-verify/SKILL.md:14`：第 1 步整批重交；已做过第 2 步的才在新快照上重交第 2 步；第 1 步自己红的只重交第 1 步 |
| 入口·攻 + 正 + 对照 | 攻 义-4；正 观察；对照 入口-4 | 「主 agent 可以自己写的只有两类」比原件多了「只有」，与共用约束「简单任务自己直接做」对不上 | 成立 | `dot-claude/main-agent.md:9`：「只有」限定在 `crates/` 与 kb 两处，别的文件的简单任务指共用约束「派发」一节；总表入口-4 同改 |
| 入口·攻 | 驻-2 | 拒算尾段（清单罩不到的三类测试要审 diff）只在技能末尾，用它的时刻在写段 | 成立 | `dot-claude/main-agent.md:68` 加一句并指实现员定义第 3c 步；技能末段 `dot-claude/skills/merge-verify/SKILL.md:58` 留着 |
| 入口·攻 | 驻-3 | 只提交、不派子 agent 的会话读不到 `needs-attention` 不是结局与服务健康回读 | 成立 | `dot-claude/main-agent.md:74` 各留一句（原话照 `dot-claude/skills/dispatch/SKILL.md:39`、`:41`） |
| 入口·攻 | 改-1 | 挪进技能的流程细则出了工具员写范围与四道只认 `main-agent.md` 的检查的射程 | 成立（现读：`.claude/hooks/agent-write-scope.tsv:33-36` 没有 skills；四处行 `doc-process-records.sh:2272`、`code-tooling.sh:4153`、`lib-lag-baseline.py:235`、`doc-text.sh:1953` 现读都只认 main-agent.md 与 agent-common.md） | 不改合稿的工具员定义写范围；`references.md` 新增「挪进技能的流程细则出了…」一节登成脚本那一批要做的事 |
| 入口·正 | 一 丢失（轻） | 冻结层归属登记表那一行三条判据换成指针，指到的文件头格名表里没有「每棵树」「每个单元类」 | 成立（`doc-decisions.sh:22` 只写「每层每个结构都有行」，细目在第 1851 行起的注释与 kb 文件的「门禁判哪三条」） | `CLAUDE-merged.md:84`：三条判据写回，指针改指 `.claude/kb/freeze-layer-membership.md`「门禁判哪三条」（现查标题在） |
| 入口·攻 + 正 + 验证·攻 + 验证·正 + 矛盾·攻 + 矛盾·正 | 攻 入口-二义「提交之外」、验证 攻 二-2、验证 正 格二 2、矛盾 攻 入口-1/合稿 3、矛盾 正 缺口 入口-1 | 提交之外跑不指定范围的，入口写「用户要求的照跑」，别处写「先弹窗」 | 成立；用户定案第 18 条已定 | 句子只住 `dot-claude/rules/verification.md:38`；`dot-claude/main-agent.md:26`、`dot-claude/agent-common.md:59`、`dot-claude/skills/crash-test/SKILL.md:12`、`dot-claude/rules/implementation-workflow.md:51`、`dot-claude/rules/session-wrapup.md:37` 指过去 |
| 入口·正 | 观察（参考件登 read-once） | 参考件登进 read-once，总表没登 | 成立（合稿说明里列了，总表漏） | 总表补合稿-4（`contradictions.md`）；`CLAUDE-merged.md:66-67` 不动（主 agent 已处置） |
| 入口·攻 | 没算打中 | 原件括号里几句脚本行为（退 3 时别的步绿、退 75 没有那两段、验证包带 `--release`、三样都没有时报本次未跑）技能没抄 | 不成立（不算丢） | 这几句在 `scripts/gate/merge-and-verify.sh` 文件头，现读：「验证包…带 --release 跑」在第 59 行，「退 1、2、75 的都不打这几条」在第 168 行，「3 判不出改到哪个测试目标…别的步照跑、没有红」在第 210 行，技能正文本来写「以脚本文件头为准」；不改 |
| 入口·攻 | 没算打中 | 「六道之间谁并行谁串行」没查完 | 不成立（找到了） | 在 `dot-claude/skills/crash-test/commit-stages.md:22`（次序）与 `:27`（broad-mutation 排在 crates-mutation-replay 之后）；`dot-claude/main-agent.md:96` 指过去，不改 |
| 入口·正 | 观察 | 「崩溃点测试准确率…」正文与跑整包的确认办法入口只留指针到共用约束 | 不成立（不算丢） | 指针写了文件与条名，条名现查在 `dot-claude/agent-common.md`；不改 |

## 二、验证题（verification）

| 题·腿 | 编号 | 一句话 | 成立 | 改在 / 不改的理由 |
|---|---|---|---|---|
| 验证·攻 + 正 | 攻 一-1；正 格一 1 | 「接完之后旧库由主 agent 删掉」丢了主语 | 成立（原件 `.claude/rules/verification.md:68`） | `dot-claude/rules/verification.md:58` |
| 验证·攻 | 一-2、二-5 | 「测试自己的配置文件是那条测试的事，与多机无关」划出去的半句丢了 | 成立（原件 `:78`） | `dot-claude/rules/verification.md:67`；`qemu.env`、`rocksdb-prebuilt.env` 两个例子照原件放回 |
| 验证·攻 | 一-3 | 「任何命令都派得到两台，哪台由余量定」丢了 | 成立（原件 `:77`） | `dot-claude/rules/verification.md:65` |
| 验证·攻 + 正 | 攻 一-4、格三 2；正 格三 2 | 判器调用点标记写在哪、「写在函数体里的不算」只剩指针，指到的文件头行号还写死 | 成立；脚本文件头现读：「一处调用点算合规」那一段列了位置，「函数体里的不算」没写 | 常驻放回：`dot-claude/rules/verification.md:25`；参考件 `dot-claude/rules/verification-reference.md:154` 的 `:14-21` 改成按段落名指 |
| 验证·攻 | 一-4 另一处 | 参考件里写死的行号指针共 6 处，文件头一改就漂 | 成立（现读：`:138` 两处、`:146`、`:154`、`:195` 两处） | `dot-claude/rules/verification-reference.md:138`、`:146`、`:154`、`:195` 都改成按段落名或文件头标题指（`:195` 的两处：用法、多机配置） |
| 验证·攻 | 一-5；没算打中 | 主 agent 派发表「含」括注指到的 `verification.md` 那一节没有 feature-gated-tests 另跑一趟与按需领卡 | 成立 | `dot-claude/main-agent.md:96` 改指参考件「门禁管哪一半」「门禁 crash-point-replay 怎么跑」 |
| 验证·攻 + 正 | 攻 二-1；正 格二 1 | 参考件两个同名标题「门禁管哪一半」 | 成立（现读 `:140` 与 `:213`） | 第二个改名「工作线程那一半归哪一格」：`dot-claude/rules/verification-reference.md:213`；指它的 `:217` 写全路径 |
| 验证·攻 | 二-3；正 格二 3 | 「第一轮没打中就停」与「两次都没打中才记」谁先 | 成立；用户定案第 17 条已定 | `dot-claude/rules/three-way-inference.md:43`、`:70`（停是不开下一轮；没打中的结论拿去支撑别的结论时才再抽一次） |
| 验证·攻 | 二-3 另一处 | 「其中至少一轮派辩方腿」与前两轮收敛后停在两轮冲突 | 成立 | `dot-claude/rules/three-way-inference.md:43` 加「开了第二、三轮的」 |
| 验证·攻 + 正 | 攻 二-4；正 格二 3 | 代码轮「打中的写回代码」与「先挂起」前后对不上；第二轮收敛时第三轮开不开 | 成立；用户定案第 17 条 | `dot-claude/rules/implementation-workflow.md:10`（代码轮自己的一套）与 `dot-claude/rules/three-way-inference.md:43`（只管设计推论轮），互相指过去 |
| 验证·攻 | 三-1 | 「控制测试线程的只有变异那一路」原件常驻，合稿入口只写给 GPU 的活 | 成立（原件 `:59`） | `dot-claude/rules/verification.md:27` 放回常驻，变异那一路的写法留在参考件「测试线程」 |
| 验证·攻 | 三-2 | 技能「判决由主 agent 做，不由投票做」一节没有常驻入口，`--cwd` 与输出路径那两句只在技能 | 成立 | `dot-claude/rules/three-way-inference.md:35` 加指向 |
| 验证·攻 | 三-3 | 单元键常驻给了全称定义，两种排除只在参考件 | 成立 | `dot-claude/rules/verification.md:39` 补两种排除与「估重判前先读参考件」 |
| 验证·攻 + 正 | 攻 四-1；正 格四 | 规则正文里「先读某技能、某参考件」算不算点到；实现员定义括号说三节里有「测试文件第一行声明模块」，指到的节里没有 | 成立 | 点到整份读写成两句：`dot-claude/agent-common.md:17`；声明模块放回 `dot-claude/rules/verification.md:23`（实现员定义点名的三节之一）；`references.md` 里「括注改指」那一行改成不改 |
| 验证·攻 + 正 | 攻 五-1；正 格五 2 | `crash-point-replay` 可点的项多了一类挂特性的测试文件 | 成立；脚本文件头现读 `.claude/gate.d/crash-point-replay.sh:40` 写「项有两类」 | 文字不改（对），总表补合稿-8 |
| 验证·攻 + 正 | 攻 五-2；正 格五 1 | 「不 source 它」与脚本文件头 `:237` 对不上，合稿改对了没登 | 成立 | 文字不改，总表补合稿-8 |
| 验证·攻 + 正 | 攻 五-3；正 格五 另一处 | 经服务交的是「每一种跑法」，原件两处宽窄不一，合稿取了宽的 | 成立 | 文字不改（与 `crash-point-replay.sh:135` 一致），总表补合稿-8 |
| 验证·攻 + 正 | 攻 五-4；正 格五 3 | 登记表第三列从现状句变成许可句，射程变宽 | 成立；表头 `.claude/gate.d/verdict-library-scripts.tsv:3` 现读与合稿一致 | 文字不改，总表补合稿-8（射程变宽写明） |
| 验证·正 | 格一 2 | 三个欠账号 C273、C234、C320 没了 | 成立（指路编号；`scripts/kb/quote-kb.py` 文件头现读 C234、C320 在） | `dot-claude/skills/three-way/SKILL.md:16`、`:17` 放回，原件 C192、C244 本来就在 |
| 验证·正 | 格三（声明模块；判器标记；变异行筛选词；阶段第一个参数） | 四条判据只在参考件那张表 | 成立；用户定案已定放回常驻 | `dot-claude/rules/verification.md:23`、`:25`、`:26`、`:99`；原表里同类的还有「崩溃枚举用例登记进合表」，一并放回 `:23` |
| 验证·正 | 没算打中（各格判什么照 `verification.md` 两种跑法一节，三处） | 要再跳一次才到 | 成立（主 agent 要求全树核） | `dot-claude/skills/crash-test/SKILL.md:18`、`dot-claude/skills/crash-test/commit-stages.md:24`、`dot-claude/main-agent.md:96` 改指参考件有那句话的节；其余指到那一节的指针逐个核过（见 `result.md`），都落得到 |
| 验证·正 | 没算打中（`:53`/参考件 `:195` 的多机配置指针） | 指到的行不是多机配置，行号写死 | 成立（原件就如此，不是压缩带来的；现读 `minimal-copy.py` 文件头「副本里起命令的」那一句才是） | `dot-claude/rules/verification-reference.md:195`，判据句 `SINGLEFS_MULTI_HOST_CONFIG` 在这一侧写出 |
| 验证·正 | 没算打中（「不经家目录取」后半只在共用约束） | 起步读不读共用约束 | 成立（便宜） | `dot-claude/rules/verification.md:55` 放回 |
| 验证·正 | 没算打中（「库里的红块不判」原件 `:39`，两份规则都没抄） | 门禁文件头有（`crash-point-replay.sh:121`） | 不成立（不算丢，文件头在） | 不改 |
| 验证·正 | 删掉不算丢（「历史与依据」整句） | 来历指针 | 同意不算丢 | 不改 |
| 验证·攻 | 没算打中（commit-stages、staged-gate 指针；agent-common「门禁只有一个」长条；四道门禁参考件逐句） | 没核 | 不适用 | 全树指针脚本核过（`selfcheck-pointers.txt`）；agent-common 那一条逐句没核，写进 `result.md`「拿不准」 |

## 三、常驻规则题（residents）

| 题·腿 | 编号 | 一句话 | 成立 | 改在 / 不改的理由 |
|---|---|---|---|---|
| 常驻·攻 + 正 | 攻 一-1；正 丢-7 | 提交技能「各片与合并各判什么、要卡的三处」指到 `verification.md` 那一节里没有 | 成立（现读 0 命中；话在参考件「门禁 crash-point-replay 怎么跑」） | `dot-claude/skills/crash-test/commit-stages.md:24`、`dot-claude/skills/crash-test/SKILL.md:18` |
| 常驻·攻 + 正 | 攻 一-2、五-1；正 丢-6 | `fs-design` 并发交错那一句指到的节没有 lkmm 与 litmus，又写「规则里还没有对应的节，记在欠账表」 | 成立（照用户定案处置：三样各指真有那句话的地方；lkmm 每份 litmus 在参考件「各道的判定单位」，对照组与绑到代码在参考件「门禁管哪一半」lkmm 那一行与 `.claude/scripts/lkmm.sh` 文件头「会失败的检查」第 3、4 条，现读） | `dot-claude/rules/fs-design.md:112`；总表处置-6 同改 |
| 常驻·攻 + 正 | 攻 二-1；正 格二 1、格五 2 | 「变异清单」被写死成 `crates/mutations.tsv` | 成立 | `dot-claude/rules/show-me-test-by-review.md:7`、`dot-claude/rules/implementation-workflow.md:9`；总表补合稿-5 |
| 常驻·攻 + 正 | 攻 二-2；正 丢-5 | 沟通纪律的射程例外指到上游不存在的节「射程与例外」 | 成立（上游标题现读无此节，话在开头段） | `dot-claude/rules/writing-discipline-by-review.md:56`：判据句留在项目这一侧、指上游只写到文件 |
| 常驻·攻 + 正 | 攻 五-2；正 格五 另列 1 | 总表常驻-11 标「只改措辞」，实际删了检出工具在调度方不响应时自己终止这一支 | 成立 | 总表标签改成「改了判据（标签更正）」；规则文字不动（删得与 `.claude/main-agent.md:76` 一致），是否放回交用户 |
| 常驻·正 | 丢-1 | 「等改动停下再跑，或者用 --staged」没有落点；另矛盾总表常驻-8 | 成立 | `dot-claude/rules/session-wrapup.md:37`：路留着、写明先弹窗，那句从原件放回（用户定案第 18 条）；总表常驻-8 改成「改回原判据（加先弹窗）」 |
| 常驻·攻 + 正 | 攻 没算打中；正 丢-2 | 「引的时候引那一行」没带 | 成立（现读原件 `.claude/rules/verify-before-claiming.md:48` 是单独一句） | `dot-claude/rules/evidence-discipline.md:55` |
| 常驻·正 | 丢-3 | 下标 `[147]` 的「即值 − 1」没了 | 成立（原件 `format-evolution.md:62`） | `dot-claude/rules/format-evolution.md:48` |
| 常驻·攻 + 正 | 攻 没算打中；正 丢-4 | 「先问它是哪个参数的函数，再写进正文」次序半句没了 | 成立（原件 `evidence-discipline.md:91` 是那一格的判据前半） | `dot-claude/rules/evidence-discipline.md:98` |
| 常驻·正 | 格二 4、攻 没算打中 | 提交技能「一节定的前缀」所指不明 | 成立（原件同形，但许修正） | `dot-claude/skills/crash-test/commit-stages.md:23`：改写成「不带 `SINGLEFS_HEAVY_TESTS` 前缀（「跑之前定下来」一节）」；`staged-gate.md` 已无此句 |
| 常驻·正 | 格四 步骤里指针 3 处 | `experiment-runner.md:43`→`show-me-test-by-review.md`、`sweep.md:34`→`path-moves.md`、`three-way-materials.md:27`→`fs-design.md` 靠「点到就整份读」 | 成立（取决于那一句的读法） | `dot-claude/agent-common.md:17` 写成两句，「只写 照 <规则> 没写节」的做那一步之前读那一份；三处都落在这一句里 |
| 常驻·正 | 格五 5：code-discipline「两处以上游为准」 | 新加判据，原件没有 | 成立 | `dot-claude/rules/code-discipline-by-review.md:4` 删；总表补合稿-6 |
| 常驻·正 | 格五 另列 2 | 提交技能删了「计划按多机配置点名的卡数写 N 片」「按上一次用时分两堆」，规则侧登了、技能侧没登 | 成立（脚本文件头 `crash-point-replay.sh:140` 现读「上界取它与卡数的大者」） | 总表验证-2 说明里补；参考件 `dot-claude/rules/verification-reference.md:211` 补全上界三种取法 |
| 常驻·正 | 格五 另列 3 | 射程多了「自证」一词 | 不成立（现已无关） | 用户定案第 6 条撤回，该节照原件恢复（「装置、判法与复跑脚本」），那个词也回去了：`dot-claude/rules/command-safety-by-review.md:6` |
| 常驻·攻 | 没算打中（总表 9 行「只改措辞」只核了常驻-11、12） | 标签可能不准 | 部分成立 | 对着原件逐行核了常驻-7、共用-10（正推腿也点了），改标签；别的 7 行没动，写进 `result.md`「拿不准」 |
| 常驻·攻 | 没算打中（常驻-6「门禁是下限」上游 0 命中未复核） | 处置依据是一次搜索 | 未复核 | 不改，写进 `result.md`「拿不准」 |
| 常驻·正 | 节标题变动 21 个 | 按旧节名引的地方要跟着改 | 部分成立 | 全仓按 `git grep` 对这 20 个没了的标题查过：除 `references.md` 已登的（`machine-first.md` 一级标题、`fs-design.md`、`test-discipline.md`、`remind` 技能）外没有活文件引它们；「装置起的样本进程，放进它自己的进程组」一节名因用户定案第 20 条原样保留，`references.md` 删了那 13 处改名行 |
| 常驻·攻 + 正 | 提醒 | 全树指到 `verification.md`「一个门禁：两种跑法」的指针要逐个核 | 成立（主 agent 要求） | 全树二十多处逐个读过：改了 5 处（`dot-claude/main-agent.md:96`、`dot-claude/rules/fs-design.md:112`、`dot-claude/skills/crash-test/SKILL.md:12`、`:18`、`dot-claude/skills/crash-test/commit-stages.md:24`），其余落得到（键的组成、哨兵、定义表、跑法表都在那一节） |

## 四、定义题（definitions）

| 题·腿 | 编号 | 一句话 | 成立 | 改在 / 不改的理由 |
|---|---|---|---|---|
| 定义·攻 | 攻-1；正 格五 注 | 样本进程逐个按进程号停，正是钩子 ⑥ 与门禁 research-gate-lint 判红的写法 | 成立；随后用户定案第 6、13 条撤回、第 20 条维持钩子 ⑥ | 规则、共用约束照仓里原件的做法恢复（`dot-claude/rules/command-safety-by-review.md:4`、`dot-claude/agent-common.md:67`，⑥ 的放行行补一句 `:80`），工具员第 5 步写清只对自己用新会话起的那一个子进程的组发（`dot-claude/agents/tooling-writer.md:33`）；写法以钩子文件头第 166 到 196 行现读为准；「逐个按进程号停」「不新写停样本进程的代码」「按会话号在 /proc 里认」都不再出现；`references.md` 里为第 6、13 条登的两节删掉；不必再跑那条扫样本的命令 |
| 定义·攻 + 正 | 攻-2；正 格二（读文件射程）| 「点到就整份读」谁点到算数读得出两种 | 成立 | `dot-claude/agent-common.md:17`：定义「开工先读：」点名的与共用约束点名的规则开工整份读；步骤里写「照 <规则>「<节>」」的做那一步之前读那一节；只写「照 <规则>」的读那一份（用户定的两句） |
| 定义·攻 | 攻-3 | 三条三方腿的样本进程停法删成指针，再指的规则腿没被点名 | 成立 | `dot-claude/agents/three-way-forward.md`、`three-way-defense.md`、`three-way-attack.md`（第 30、31、39 行）写成「照共用约束「样本进程放进它自己的进程组」与 `.claude/rules/command-safety-by-review.md`「装置起的样本进程，放进它自己的进程组」」，规则那一节写在步骤里，照 `agent-common.md:17` 做那一步之前读 |
| 定义·攻 + 正 | 攻-4；正 格一 命中 1 | 「拿旧数支撑新结论之前先重新跑一遍」触发条件被换 | 成立 | `dot-claude/agent-common.md:24`：半句放回，指上游 kb-discipline 只写到文件 |
| 定义·攻 | 攻-5 | 完工清理闸：写成「交接」的副本子 agent 交回时不列，这半句丢了 | 成立（`.claude/hooks/completion-cleanup-guard.sh:100` 现读） | `dot-claude/agent-common.md:100` |
| 定义·攻 | 攻-6（⑬出路） | ⑬ 的出路「要另起一个 agent 的活写进报告交回主 agent」丢了 | 成立（原件 `agent-common.md:95`） | `dot-claude/agent-common.md:87` |
| 定义·攻 | 攻-6（派发闸「…已判」） | 「通用已判」「准入已判」「写者锁已判」三种行与一次性的活派 clerk 找不到 | 成立 | `dot-claude/agent-common.md:104` 各补一句；「自证在哪跑」主 agent 那一半（不许点工作根）没补：拒绝说明会给写法，写进 `result.md`「拿不准」 |
| 定义·攻 | 攻-7 | 实现员 1d「照说明给的写」的「说明」失了所指 | 成立 | `dot-claude/agents/implementation-writer.md:42`：入口退 79 时打的说明给的值 |
| 定义·攻 | 攻-7 | 共用约束「`scripts/kb/` 下的定点写入脚本」没列是哪几份 | 成立（原件逐个列名） | `dot-claude/agent-common.md:58` 列回五份与 `ledger-append-section.py` |
| 定义·攻 | 弱：带 `--item` 的四条整条删成指针 | 今天没有哪类会因此做错 | 不成立（攻方自己判弱） | 不改 |
| 定义·正 | 格一 弱 1 | 报告一节「在不影响正确性的前提下写简练」前提没了 | 成立（便宜） | `dot-claude/agent-common.md:114` |
| 定义·正 | 格二 命中 2 | 实现员定义命名检查的 `<根>`「你改动所在的树」与共用约束 `<快照>` 对不上 | 成立 | `dot-claude/agent-common.md:109`、`dot-claude/agents/implementation-writer.md:31`：命名检查的 `<根>` 写 `<快照>`，由主 agent 在验证段第 2 步填；总表共用-10 标签更正 |
| 定义·正 | 格五 注（新加三处「开工另读」） | 总表没登 | 成立 | 总表补合稿-11 |
| 定义·正 | 格五 推不出（项目代码闸、范围声明闸、交回正文闸、只管主 agent 的三道） | 没逐句核钩子文件头 | 部分核 | 现读核了交回正文闸文件头 ① 到 ⑤（与 `dot-claude/agent-common.md:103` 一致）、范围声明闸文件头的提交时才许不指定范围一段（与 `:59` 一致）；项目代码闸、写闸其余各道、弹窗闸、定时闸没逐句核，写进 `result.md`「拿不准」 |
| 定义·正 | 用户定案 10 | `quote-rust-items.py` 钩子库还没加，材料员照定义跑会被拒 | 成立（脚本一侧的活） | 不改文字；总表用户定-10 补充已有；`references.md` 第四节已登 |
| 定义·正 | 用户定案 17、18、19「待落」 | 合稿没落 | 成立 | 17：`dot-claude/rules/implementation-workflow.md:10`、`dot-claude/rules/three-way-inference.md:43`、`:70`；18：见入口题；19：`dot-claude/main-agent.md:47`、`dot-claude/agent-common.md:109`、`dot-claude/agents/investigator.md:23`、`dot-claude/agents/implementation-writer.md:15`、`dot-claude/skills/dispatch/SKILL.md:14` |
| 定义·正 | 找不到落点 3 到 5（消费方说明、写范围闸括注、「不抄第二份」） | 说明句 | 不成立（不带判据） | 不改 |
| 定义·正 | 指针核对表 E 类 5 个（子 agent 不读主 agent 一侧文件） | 字面上不被点名读 | 不成立（只管主 agent 的动作） | 不改 |

## 五、矛盾题（contradictions）

| 题·腿 | 编号 | 一句话 | 成立 | 改在 / 不改的理由 |
|---|---|---|---|---|
| 矛盾·正 | 验证-9 | 合稿「缩过域的那一趟不判」丢了「字段缺了照判红」 | 成立（`scripts/gate/admission.py` 文件头 `exhaustive=<前缀>` 那一条现读） | `dot-claude/rules/verification-reference.md:146`；总表验证-9 |
| 矛盾·攻 | 入口-4 | 「只有两类」改了判据 | 成立 | 见入口题 |
| 矛盾·攻 + 正 | 常驻-8 | 落成禁令，删了一条做法 | 成立 | 见常驻题；总表常驻-8 |
| 矛盾·攻 | 合稿-1 | 代码轮「再攻一轮」被换成通则 | 成立；用户定案 17 | 见验证题；总表合稿-1 |
| 矛盾·攻 + 正 | 常驻-11 标签 | 标「只改措辞」实际删了一种许可 | 成立 | 总表标签更正（不动规则文字） |
| 矛盾·正 | 常驻-7、共用-10 标签 | 「只改措辞」实际动了做法 | 成立 | 总表标签更正 |
| 矛盾·攻 + 正 | 总表位置写偏：入口-11、按需-M7、按需-M11、常驻-19、共用-7 | 行号 | 成立 | 总表逐条改（M11 在合稿技能里是第 18 行，攻方说 16、正推说 18，现读为 18） |
| 矛盾·攻 | 共用-8、定义-8、定义-9、按需-M1、按需-M9 | 依据只写文件名、没带目录 | 成立 | 总表补全路径 |
| 矛盾·攻 + 正 | 验证-2 | 参考件上界半句窄 | 成立 | `dot-claude/rules/verification-reference.md:211` |
| 矛盾·正 | 常驻-15 | 共用约束 `:121` 括号指回三方规则同名节，那一节不写交回内容 | 成立 | `dot-claude/agent-common.md:121` 把括号挪到「分段落盘」后；总表常驻-15 |
| 矛盾·正 | 按需-M4 | 上游 `show-me-test.md` 整份从清单里掉了 | 成立 | `dot-claude/rules/test-script-discipline.md:4`；总表按需-M4 |
| 矛盾·正 | 合稿-3 | `implementation-workflow.md` 引上游的路径写成裸文件名 | 成立 | `dot-claude/rules/implementation-workflow.md:23` |
| 矛盾·正 + 攻 | 用户定-9、处置-6 欠账号 | 「记在欠账表」此刻不成立 | 部分成立 | 处置-6 那一句已删「记在欠账表」；用户定-9 的「收严记在欠账」保留，欠账号由主 agent 登了再补，总表补充已写 |
| 矛盾·正 | 用户定-10 | 钩子库没加，两句此刻对不上 | 成立（脚本一侧） | 不改文字 |
| 矛盾·攻 | 用户定-1 余项、-5 余项（执行员「卡住」只写了吃紧）、-6 余项 | 调查员那一支；执行员例外只写吃紧；会话号一句 | 部分成立 | 调查员：用户定案 19；会话号那句随用户定案第 20 条删掉；执行员「卡住」：`dot-claude/agent-common.md:68` 的例外照用户定案「保留吃紧就交回」只写吃紧，不改，写进 `result.md`「拿不准」 |
| 矛盾·攻 | 总表之外：合稿内并存说法 | 提交之外 | 成立 | 同入口题 |
| 矛盾·正 | 验证-5（测试自己名字的线程变量写成 `-- env` 交服务拒不拒没核） | 不影响判定 | 不适用 | 不改，写进 `result.md`「拿不准」 |
| 矛盾·攻 + 正 | 其余 85 条里改得对的 | 82 或 84 条 | 同意 | 不改 |
