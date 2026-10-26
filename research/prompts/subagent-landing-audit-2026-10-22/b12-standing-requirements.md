# 第十二批任务 46：派发提示的常设要求清单

日期：2026-10-23。批次：`records/2026-10-22-子agent结果落地审计.md#12`。只读，没改仓里任何文件。
对应 `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑤（`:1476`）。

## 一、结论

- 戊类 5 次（E171 #19、#20、#21、#31、#33）漏的要求只有三条：长活按多机分片续跑（含不写死台数与额度）、判定复用、长活的草稿目录与跑法脚本放开机不清的目录。逐次对应见第二节。
- 下表列 31 条要求（A 通用 7、B 长活 11、C 实验 4、D 实现 3、E 改门禁 2、F 其他类型 4），其中派发闸或骨架在核的 19 条（A4 只有骨架 `--check` 核，不挂钩子），没人核的 12 条。
- 戊类漏的三条里：多机分片续跑那一条，派发闸 ⑭ 今天核「三行在不在」（2026-10-06 的五十九节之后才建，E171 #19–#21 发生时没有），核不到的是「整份提示里写死台数线程」；判定复用今天没人核；持久目录今天没人核，派发闸还照放 `/tmp/claude-<uid>`（实测见附录）。
- 没人核的 12 条里，能做成「骨架按类型打那一行、派发闸核那一行在」的 8 条（B6、B7、A2、B8、B9、B10、D3、E2；B8 限定在执行员续做），只能核一半的 3 条（B4、B2b、A5：形式能核、内容不能），不能的 1 条（C4）。第四节乙组逐条写。
- 戊类要先做的是 B6（判定复用，骨架多打一行「复用：」）、B7（草稿目录不落 `/tmp`，不用加行）、B4（整份提示不写死线程、核、台）。这是排序建议，不是这一件的出口，没动手。
- 推翻条件：第三节「谁在核」是读代码加探针得的。如果派的那一刻派发闸还有这几份文件之外的判（例如 settings 里另挂的 hook），表里「没人核」的行要改；我查了 `runner-dispatch-guard.sh`、`lib_long_run.py`、`lib_work_root.py`、`dispatch-prompt.py`，没读 `.claude/settings.json` 里挂的别的 hook（见「没做什么」）。

## 二、戊类 5 次各漏哪一条要求

依据 `research/prompts/subagent-landing-audit-2026-10-22/blind-rework.md`（下称盲判）第 61–63、73、75 行与第七节第 172 行。要求编号见第三节的表。

| 次 | 盲判行 | 盲判写的缺口（原文） | 漏的要求 |
|---|---|---|---|
| E171 #19 | 61 | 「主 agent 给 #14、#18 的草稿目录都在 /tmp，推约 34–38 小时的长跑把跑法脚本与判法放在开机就清空的目录」 | B7 长活放持久目录 |
| E171 #20 | 62 | 「主 agent 派第一段跑法（#14、#18）与重建（#19）时，没把已有的「用上现有的硬件加速」和用户常设的「重活两台都用」写进提示；按单机 10 线程起的那一趟十几分钟后就被停掉重写」 | B2 分片、续跑、多机三行（多机、分片两样） |
| E171 #21 | 63 | 「主 agent 写 #20 的提示时写死两台各 31 线程、合计 80G，没照用户常设要求按多机配置自适应、带续跑；用户当场纠正」 | B4 不写死台数与额度；B3 多机自适应；B2 续跑 |
| E171 #31 | 73 | 「修订 8、10 的设计（#22、#29）没照 test-script-discipline「判定可复用：按输入存，按前缀剪」给单元算键；用户问起时主 agent 先答不建议做，用户改判后才补修订 11 与实现」 | B6 判定复用 |
| E171 #33 | 75 | 「同 #31」（承 #31，实现修订 11 的复用层） | B6 判定复用 |

- 用户原话：「长时间重型任务 。分片 断点重启 这都不用我说。 自适应多机跑 /home/fy5090/code/singlefs/multi-host.env」（`records/2026-10-01-里程碑三形式化接入.md:1281`）；「凡是超过2两小时的脚本都要考虑多机 分片 跑 不然慢的不行」（`records/2026-09-16-subagent拆分提案.md:1160`）；「这个E171  形式化还是要做剪枝 复用的」（盲判:73 引的主会话 2026-10-08）；「把工作目录搬出 /tmp」（拆分提案:1140）。
- 盲判第七节第 172 行自己写了推翻条件：`verification.md` 引「用上现有的硬件加速」「判定可复用」两节在 2026-10-03 的提交 0da17aa14，两份实验定义要到 2026-10-10 的 0df2bd4aa 才引。我没有重核这条。
- 需要分清的一点：#20 和 #21 是两个不同的缺口。#20 是 #14/#18/#19 的提示没要多机分片；#21 是 #20 的提示要了多机却写死了数、没要续跑。今天 ⑭ 只拦得住前一种（三行缺）；后一种里「续跑」缺行也拦得住，「写死」只在「多机：」那一行里拦（探针 p2 见附录：整份提示正文写「本机 31 线程、第二台 31 线程，内存两台合计 80G」照放）。

## 三、常设要求表（一条要求一行）

列：编号｜要求原文（整句抄，长句抄到句号）｜出处｜适用｜今天谁在核｜戊类漏。
「谁在核」引用的行号与命中取自附录的 grep（`greps1.txt`、`greps2.txt`）和探针（`probes.txt`）。简写：派发闸＝`.claude/hooks/runner-dispatch-guard.sh`（下称闸），骨架＝`scripts/agents/dispatch-prompt.py`，库＝`.claude/hooks/lib_long_run.py`。

### A 通用（所有项目 agent 类型）

| 编号 | 要求原文 | 出处 | 适用 | 今天谁在核 | 戊类漏 |
|---|---|---|---|---|---|
| A1 | 派任何类型，提示里要有一行「重型测试：不跑」（重型测试只由主 agent 跑） | 闸:8（来源 `.claude/main-agent.md:15`「禁止在subagent中跑重型测试」） | 所有类型 | 闸 ①核；骨架 `dispatch-prompt.py:330` 打这一行 | 无 |
| A2 | 开工时同时写死这一轮的派发上限与截止批次，依赖表加「批次」列，每份派发提示写一行「批次：<调度记录>#<批号>」；批内冒出的发现只进下一轮的清单，不当场派。 | `.claude/main-agent.md:20` | 所有类型 | 骨架 `dispatch-prompt.py:331` 打占位；闸只在 ⑬ 核占位没填完，不核这一行在不在、指的文件在不在。探针 p5：删掉整行照放；p2：写 `records/x.md#1`（不存在的文件）照放。**没人核** | 无 |
| A3 | 轮名、产出文件路径、草稿目录、这一轮的禁读清单都由主 agent 在派发提示里给。（紧接一句：输入缺一样就不开工，回复只写缺什么。） | `.claude/agent-common.md:11`、`:12` | 14 份定义都有 `required-inputs:`（材料员的是 `-body.md`） | 闸 ③（`guard:852` 读各定义 frontmatter 的 `required-inputs:`，组内同义词出现一个就算）；骨架按同一张表打（`dispatch-prompt.py:4`、`:19`） | 无 |
| A4 | 带「草稿目录：」一行的类型，骨架在它后面多一行「认领：bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '<用途一句>'」 | `dispatch-prompt.py:25` | 带「草稿目录：」一行的类型 | 骨架 `--check` 核（探针 p6：去掉认领行，`--check` 退 3 并点名；直调闸自己的 `--check-prompt`、与钩子同一份判法，退 0 放行）。派发闸不核；`--check` 要主 agent 自己想起来跑。**只有骨架核** | 无 |
| A5 | 派一个 agent 之前说清它关的是哪几条已经抓到的问题（可以不止一条，逐条列）；说不出来的，那条该进记录、不该进本轮。 | `.claude/main-agent.md:25`（该行第 6 条） | 所有类型（调查、调研类写要回答的问题） | 骨架 `dispatch-prompt.py:363` 打节标题「关的是哪几条」与占位；闸只核占位填完（⑬）。节在不在、是不是空话，没人核。**没人核** | 无 |
| A6 | 要读的文件与要跑的命令（有的话）；要改仓内文件的，逐个列出路径，并且只许是「写范围」里的那几处。 | `.claude/agents/clerk.md:21`（其余类型各自的写范围节） | 所有项目类型 | 闸 ④（`guard:24` 报告行与「写进/写到/更新」后的路径按 `agent-write-scope.tsv` 判）；写闸 `write-guard.sh` 事后再拒 | 无 |
| A7 | 不读共用约束的类型（Explore、写了放行行的通用类型，或没有定义文件的类型）：提示里要有「开工先读」与 `.claude/agent-common.md`。（闸:21）内置通用类型（general-purpose、claude，或不给 subagent_type）一律拒：一次性的活派项目定义 clerk（起步不带 CLAUDE.md 与规则），只读搜索派 Explore；确要派内置类型，提示里写一行「通用已判：<为什么 clerk 做不了>」放行。（闸:18） | `runner-dispatch-guard.sh` 文件头第 21、18 行（来源 `.claude/main-agent.md:7`） | 内置与无定义的类型 | 闸 ②（`guard:799`、`:2177`）与 ⑫（`guard:2165`） | 无 |

### B 长活与重活（带重活的十种：实现员、调查员、工具员、执行员、变异分诊员、攻方腿、clerk、正推腿、辩方腿，加设计员）

| 编号 | 要求原文 | 出处 | 适用 | 今天谁在核 | 戊类漏 |
|---|---|---|---|---|---|
| B1 | 预计挂钟：一行「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」；超过的另有「分片：」「续跑：」「多机：」三行，分不了的那几样由一行「分不了：」点名；这三个标签各只写一行，同一标签写第二行拒，闭合了的围栏代码块（照 CommonMark 认）里的行不算。 | `.claude/agents/clerk.md:23`（同句在 `experiment-designer.md:25`、`experiment-runner.md:23`、`implementation-writer.md:26`、`investigator.md:24`、`mutation-triage.md:24`、`tooling-writer.md:24`、`three-way-attack.md:24`、`three-way-defense.md:25`、`three-way-forward.md:25`） | 上述十种 | 闸 ⑭（类型表 `guard:1888` `LONG_RUN_AGENTS`，判法 `lib_long_run.py`）；骨架 `dispatch-prompt.py:187` `LONG_RUN_TYPES` 同十种，打「预计挂钟：」一行 | 无（但 E171 #14/#18 的挂钟推 34–38 小时，当时没有这一判） |
| B2 | 长活默认分片、能断点续跑、按配置里的机器自适应：派发提示里不写死台数与额度（几台、哪几张卡、每台多少内存；用户给了逐台参数的，照「多机已判：」那一行的写法写）；预计超过 2 小时的照派发闸 ⑭ 写「分片：」「续跑：」「多机：」三行，（整句很长，抄到这一处；后半句是 B3，「不写死」那一半是 B4） | `.claude/main-agent.md:82` | 上述十种，预计挂钟超过 2 小时的活 | 闸 ⑭ 核三行在不在、各只一行，缺的要「分不了：」点名（探针 p4：去掉「续跑：」闸退 3）；骨架同十种打这三行占位。三行里只核有字，不核写了什么（探针 p3：「分片：好」「续跑：好」照放，见 B2b） | #20（三行都没要，单机 10 线程起跑）；#21（续跑没要） |
| B2b | 「分片：<分片单元>」「续跑：<同一条命令重起只补没做完的，怎么认做完的>」 | 骨架占位 `dispatch-prompt.py:198`、`:199`，闸拒绝说明 `guard:1895`；`test-script-discipline.md:72`「一块判完才落盘，写入是原子的；被打断后重跑同一条命令就是续跑，只补没判的。」 | 同 B2 | 没人核内容。探针 p3 照放；脚本文件头的 `# long-run-shards:` `# long-run-resume:` 由 `detached-run.sh` 与 `long-run-check` 核写法，不核做没做。**没人核** | 无（盲判没点这一层） |
| B3 | 「多机：」行与脚本文件头 `# long-run-hosts:` 只认一句规范写法，都逐字等于骨架那一句 | `.claude/main-agent.md:82`（该句很长，抄到第一个括号前）；规范句 `lib_long_run.py:102` | 同 B2 | 闸 ⑭（`canonical_hosts_problem`，`lib_long_run.py:144`）；骨架占位引同一句 | #21（没按多机配置自适应） |
| B4 | 派发提示里不写死台数与额度（几台、哪几张卡、每台多少内存；用户给了逐台参数的，照「多机已判：」那一行的写法写）（B2 同一句的第一个分句） | `.claude/main-agent.md:82` | 上述十种，所有预计挂钟的活（不只超过 2 小时） | 闸只在「多机：」这一行里查写死（`lib_long_run.py:180` 用 `HARD_CODED_QUOTA.finditer(hosts)`，`hosts` 只是那一行的值）。整份提示正文写「本机 31 线程、第二台 31 线程，内存两台合计 80G」照放（探针 p2）。**没人核** | #21（#20 的提示写死 31 线程、合计 80G） |
| B5 | 线程上限：一行「线程上限：N」，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），这一行只写这一个十进制整数，后面不接任何字（括注也拒）；要加说明另起一行，照普通正文写。 | `.claude/agents/clerk.md:24`（同句在九份定义的「输入」节：clerk、experiment-runner、implementation-writer、investigator、mutation-triage、tooling-writer、three-way-attack、three-way-defense、three-way-forward） | 九种（`guard:1247` `THREAD_LIMIT_REQUIRED_AGENTS`） | 闸 ⑮（`guard:1389` `thread_limit_verdict`，`:1267` 判形态）；骨架默认填登记值（`dispatch-prompt.py:184` `HEAVY_WORK_TYPES`） | 无 |
| B6 | 每一份判定按它依赖的全部输入算键存下：被测代码、判法代码、参数、工具链。键相同的直接复用，不重算。 | `.claude/rules/test-script-discipline.md:64`（同节 `:65`、`:75`；两份实验定义只在 `experiment-designer.md:35`、`experiment-runner.md:34` 把它当判据引） | 预计挂钟超过 2 小时的活；重跑已有实验的登记（设计员）与它的执行 | 派发提示这一层没人核：闸只在 ⑭ 的拒绝说明与文件头引这一节（`guard:70`、`:1891`，`lib_long_run.py:2`、`:426`），判的是「续跑：」有字，不是键复用。没有「复用」标签。登记层 `## 十四、并行与挂钟` 的五个标签（`experiment-designer.md:57`）也不含复用。仓里只有登记在 `verdict-library-scripts.tsv` 的脚本才有 `reuse-selftests` 格（`verification.md:60`），那是事后核脚本，不核派发。**没人核** | #31、#33 |
| B7 | 报告路径与草稿目录，都在 `~/.local/share/singlefs/work/` 下。 | `.claude/agents/clerk.md:22`（同句在 `experiment-runner.md:26`、`implementation-writer.md:28`、`investigator.md:26`、`tooling-writer.md:26`、`kb-scribe.md:26`）；起因 `records/2026-09-16-subagent拆分提案.md:1140`「把工作目录搬出 /tmp」 | 所有写草稿的类型，对长活尤其要紧（跑法脚本、判法、冻结树放在里面） | 没人核。闸 ④、⑯、⑰ 认「工作根与过渡期旧根」两处（`lib_work_root.py:25` `LEGACY_ROOT_IN_TRANSITION = True`；闸自证 `guard:2521` 还专门放一条 `/tmp/claude-1000/e/` 的草稿目录期望退 0）；探针 p2：草稿目录写 `/tmp/claude-1000/e171-r1-segment1/` 照放。`/tmp` 与工作根同在一个 ext4 根分区，`/usr/lib/tmpfiles.d/tmp.conf:11` 是 `D /tmp 1777 root root 30d`（开机清空，见附录 P）。仓里只有崩溃放量的判定库自己拒开机清空的目录（`verification.md:37`「落在开机会清空的目录里被拒」）。拆分提案五十六节（`:1128`）写过「派发闸查「草稿目录」与「报告」那两行指不指向 `/tmp`」，用户当时选的是搬根、留过渡期，闸这一判没做。**没人核** | #19 |
| B8 | 主 agent 跑完新派一个同类 agent 接着做，派发提示指到前一份报告与这次的输出路径。 | `.claude/agent-common.md:69`（`.claude/main-agent.md:76` 同义：要补的活新派一个同类 agent，派发提示指到它的报告与产物） | 接着前一个 agent 做的所有类型（长活交回后的续派） | 只有执行员的续做被核：闸 ⑪（`guard:1995`）核「上一段岔路表里还差：…」，不核有没有指到前一份报告的路径。别的类型没人核。**没人核** | 无（盲判的七类里没有这一类） |
| B9 | 内存上限；报告路径与草稿目录，都在 `~/.local/share/singlefs/work/` 下。 | `.claude/agents/investigator.md:26`、`.claude/agents/tooling-writer.md:26`；回退值在 `.claude/agent-common.md:60`「上限取派发提示给的（定义另写了来源的照定义），都没给就取 `scripts/experiment-apparatus/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。」 | 调查员、工具员（执行员的上限按登记给，`experiment-runner.md:30`、`:32`） | 闸里「内存上限」零命中，骨架也不打（`greps1.txt`）。**没人核** | 无（盲判庚类 E188 #2 是内存峰值估错，不是这一行缺） |
| B10 | 会跑自证的类型（SELFTEST_RUNNER_TYPES，按各定义「输入」一节判）的骨架另带一行「自证在哪跑：<填 …>」：自证登记 .claude/hooks/selftest-registry.tsv 按本仓路径认，副本、草稿里的拷贝跑 --selftest 被重型闸拒，派的时候就写明在哪跑。 | `scripts/agents/dispatch-prompt.py:130`、`:131` | 工具员、实现员、调查员、执行员（`dispatch-prompt.py:218`） | 骨架打这一行；闸里「自证在哪跑」零命中，`dispatch-prompt.py:24` 自己写「派发闸不单判，没填时按占位拒」。填了什么没人核。**没人核** | 无 |

### C 实验（设计员、执行员）

| 编号 | 要求原文 | 出处 | 适用 | 今天谁在核 | 戊类漏 |
|---|---|---|---|---|---|
| C1 | 不是为岔路建的实验（独立核实现、核一条推论），主 agent 给一份同格式的问题单，每行一个要回答的判断。没有岔路单或问题单不开工。 | `.claude/agents/experiment-designer.md:20` | 实验设计员 | 闸 ③（required-inputs `岔路单\|问题单\|重跑`，`被测条款`）；骨架打 `岔路单：`、`被测条款：` 占位 | 无 |
| C2 | 这一段回答岔路单的哪几行：派发提示里写一行「这一段回答的岔路：…」；续做（实验页已经有了）时再写一行「上一段岔路表里还差：…」，点名上一段交回的岔路表里还开着的行。 | `.claude/agents/experiment-runner.md:22` | 实验执行员 | 闸 ⑪（`guard:1980`–`:1997`，续做按实验页在不在判，写「无」的拒）；骨架打第一行（`dispatch-prompt.py:19`） | 无 |
| C3 | `scripts/gate/admission.py experiment <根> E<号>` 退 77（这一实验的单元键与上次产物头记的相同）就拒 | 闸:64（设计员定义 `experiment-designer.md:22` 管重跑登记） | 设计员写重跑登记 | 闸 ⑨（`guard:1122`），放行行「准入已判：<理由>」 | 无 |
| C4 | 登记分了几段、段与段不共用要改的装置与产物的：先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行跑，各写各的产物与报告 | `.claude/main-agent.md:105` | 要建计数实验或重跑已有实验的主 agent | 没人核（派发次序，不在任何一份提示里）。**没人核** | 无 |

### D 实现（实现员）

| 编号 | 要求原文 | 出处 | 适用 | 今天谁在核 | 戊类漏 |
|---|---|---|---|---|---|
| D1 | 压着的条款：kb 文件路径与小节标题（不给摘要）。 | `.claude/agents/implementation-writer.md:23` | 实现员 | 闸 ③（required-inputs `条款`，按字面子串）；骨架打 `条款：` | 无 |
| D2 | 单独一行「要动的 crates 文件：…」，逐个列全（从仓根起），份数不设上限；派发闸按这一行判与在跑的实现员撞不撞文件（`crates/mutations.tsv` 只追加、不算撞），撞了拒。 | `.claude/agents/implementation-writer.md:25` | 实现员 | 闸 ⑤（`guard:1035`、`:2376`）；骨架打这一行 | 无 |
| D3 | 主 agent 读过的 `crates/` 路径，以及这一轮别的会话正在改的 `crates/` 文件（你不碰的）。 | `.claude/agents/implementation-writer.md:24`；工具员同样 `.claude/agents/tooling-writer.md:21`「要改的文件，全列；这一轮别的会话在改的文件（你不碰的）。」 | 实现员、工具员 | 闸只用 ⑤ 把「要动的 crates 文件」与在跑的实现员互相比（实现员对实现员）；「你不碰的」这一行没人核，骨架也不打。**没人核** | 无 |

### E 改门禁、钩子、脚本（工具员）

| 编号 | 要求原文 | 出处 | 适用 | 今天谁在核 | 戊类漏 |
|---|---|---|---|---|---|
| E1 | 出口：做到哪算完。（连同「要改的文件」） | `.claude/agents/tooling-writer.md:22`、`:21` | 工具员 | 闸 ③（required-inputs `出口`、`要改的文件`）；骨架打 `出口：`、`要改的文件：` | 无 |
| E2 | 每条改法要判红的输入样本与弄坏开关名（`<脚本>_BREAK=<项>` 这类）；主 agent 给不出的，写明由你起名。 | `.claude/agents/tooling-writer.md:23` | 工具员 | 闸零命中，骨架不打（`greps1.txt` 里「弄坏开关」只在骨架的自证说明里出现）。**没人核** | 无 |

### F 其他类型（不属长活、实验、实现、门禁，列出来为完整）

| 编号 | 要求原文 | 出处 | 适用 | 今天谁在核 | 戊类漏 |
|---|---|---|---|---|---|
| F1 | 判决路径与要写回的条目编号（一次不超过 8 条；超过的按 kb 文件不相交切成几份、同时派几个写回员），或者主 agent 写好的逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。 | `.claude/agents/kb-scribe.md:20` | 写回员 | 闸 ③（`判决\|规格`）、⑧（`guard:1077`，规格交 `kb-spec-check.py`）；骨架打 `规格检查已判` 占位 | 无 |
| F2 | 不接没有表的广谱变异活动（cargo-mutants 这类几千条的扫描）：那是实验，走 experiment-designer 登记；派发闸在提示里没有变异表路径时拒派。 | `.claude/agents/mutation-triage.md:19` | 变异分诊员 | 闸 ⑩（`guard:66`）、③；骨架打 `scripts/mutation/tables/：` 占位 | 无 |
| F3 | 这一轮的禁读清单。 | `.claude/agents/three-way-attack.md:26`（辩方、正推腿同） | 三方腿 | 闸 ③（`禁读清单`）与 ⑰（本腿报告路径只认 `research/prompts/<轮>-*-output.md`）；骨架打 | 无 |
| F4 | 要改一轮还没判决的代码轮快照里的文件，等那一轮判决（派发闸 ⑥） | `.claude/main-agent.md:23`（第 4 条） | 写回员、实现员 | 闸 ⑥（`guard:36`，快照清单读 `lib_snapshot_conflicts.py`） | 无 |

## 四、两组清单

### 甲、已经有派发闸或骨架核的（19 条）

「核的是什么」只写形式（行在不在、写法对不对）；内容的真假另见乙组 B2b、A5。

| 编号 | 谁核 | 核到哪一层 |
|---|---|---|
| A1 重型测试：不跑 | 闸 ①；骨架打 | 行在、正文没有许可子 agent 跑重型的句子 |
| A3 草稿目录、报告等 required-inputs | 闸 ③；骨架按定义打 | 每组同义词出现一个就算 |
| A4 认领行 | 骨架 `--check`（不挂钩子） | 行在、目录与「草稿目录：」一致、用途非空；主 agent 不跑 `--check` 就没人核（探针 p6） |
| A6 写范围 | 闸 ④；写闸事后再拒 | 报告行与「写进」后的路径 |
| A7 开工先读、通用已判 | 闸 ②、⑫ | 字面 |
| B1 预计挂钟 | 闸 ⑭；骨架打（十种类型表逐个相同） | 行在、值是「2 小时内」或「超过 2 小时」 |
| B2 分片、续跑、多机三行 | 闸 ⑭；骨架打 | 超过 2 小时时三行在、各只一行、缺的由「分不了：」点名；值只核有字 |
| B3 多机规范句 | 闸 ⑭（`canonical_hosts_problem`） | 这一行逐字等于规范句，行内不写死额度 |
| B5 线程上限 | 闸 ⑮；骨架默认填登记值 | 只有一个整数、等于登记值，或另写「线程上限已判：」 |
| C1 岔路单、被测条款 | 闸 ③ | 字面 |
| C2 这一段回答的岔路、还差 | 闸 ⑪ | 非空；续做按实验页在不在判 |
| C3 重跑准入 | 闸 ⑨ | `admission.py` 退 77 就拒 |
| D1 条款 | 闸 ③ | 字面子串 |
| D2 要动的 crates 文件 | 闸 ⑤；骨架打 | 行在、与在跑的实现员不撞 |
| E1 出口、要改的文件 | 闸 ③ | 字面 |
| F1 规格 | 闸 ③、⑧ | `kb-spec-check.py` 过 |
| F2 变异表 | 闸 ⑩、③ | 路径在 |
| F3 禁读清单、报告形态 | 闸 ③、⑰ | 字面与形态 |
| F4 快照冲突 | 闸 ⑥ | 要改的文件不在开着的三方轮快照里 |

### 乙、没人核的（12 条）

每条写：骨架今天打不打；能不能做成「骨架按类型打那一行、派发闸核那一行在」；要判它在不在，提示里该出现的字样。判定档：能／只能核一半（形式能核、内容不能）／不能。

- **B6 判定复用（戊类 #31、#33）——能。**
  - 骨架今天不打。可做：骨架在预计挂钟超过 2 小时的十种类型里，「分片：」「续跑：」「多机：」之后多打一行「复用：<填 判定键由哪些输入算、判定库放哪、键相同复用什么；没有判定这一层的写「不复用」并写为什么>」；闸 ⑭ 的标签表（`lib_long_run.py:95` `PROMPT_LABELS`）加「复用」，同样只许一行，写「不复用」的理由至少 8 字（照 `UNSHARDABLE_MINIMUM_CHARACTERS`，`lib_long_run.py:111`）。
  - 字样：行首「复用：」。E171 修订 8、10 的设计与第一段重建都是预计挂钟几十小时的活，会被这一行问到。
  - 局限：键罩没罩全输入，`test-script-discipline.md:106` 写明「靠 review」，闸只能核这一行在不在。
- **B7 长活放持久目录（戊类 #19）——能。**
  - 骨架已打「草稿目录：」「报告：」，不用加行。可做：闸在预计挂钟超过 2 小时的提示里（想严一点就所有带草稿目录的类型）核这两行的路径落在工作根，不落在过渡期旧根 `/tmp/claude-<uid>` 下（`lib_work_root.py` 里 `work_root()` 与 `legacy_root()` 已分开）。
  - 字样：「草稿目录：」一行以 `~/.local/share/singlefs/work/` 起头（或它的展开写法、`$SINGLEFS_WORK_ROOT/`）；出现 `/tmp/` 的拒。
  - 更彻底的做法是 `LEGACY_ROOT_IN_TRANSITION` 改 False，但那一步的条件是在跑的子 agent 与别的会话的子 agent 都交回（`lib_work_root.py:5`）并改一批自证，不是这一件的事。
  - 局限：正文里另写的跑法脚本路径（「跑法脚本放 /tmp/…」）闸认不出是不是写位置，只核这两行。
- **B4 不写死台数与额度（戊类 #21）——只能核一半。**
  - 这是禁止型，没有「一行」可打；骨架今天不为它打东西，也不用打。可做：闸把 `HARD_CODED_QUOTA` 里的线程、核、台这几种（不含内存）扩到整份提示，去掉围栏代码块、「线程上限：」一行、「多机已判：」一行与它后面的逐台参数行。
  - 字样：正文里出现「<数> 线程」「<数> 核」「<数> 台」「两台各…」。
  - 不能核的：内存数。投调查员、工具员的「内存上限」本来就要写数（`investigator.md:26`、`tooling-writer.md:26`），一刀切会误拒；写死的内存（「合计 80G」）只能靠主 agent 自己区分。
- **B2b 分片、续跑两行的内容——只能核一半。**
  - 骨架已打占位。可做：值至少 8 字（照「分不了」的写法），拦「好」之类的空话。
  - 不能做：分片真能分、续跑真能续。脚本头声明与脚本做的事一致与否，`main-agent.md:82` 与 `agent-common.md:70` 都写明文件头的读法与脚本做的事是否一致「由写的人保证」，钩子只判写法。只做长度容易变成凑字数，建议和 B6 一起做时再带。
  - 字样：「分片：」「续跑：」行值字数。
- **A2 批次——能。**
  - 骨架已打（`dispatch-prompt.py:331`）。可做：闸 ⑬ 之外加一判，要有「批次：<records 下的 .md>#<数>」形态，文件存在，文件里能找到那一批（「#12」对「第十二批」之类的写法要先定）。
  - 字样：行首「批次：」，值 `records/….md#<数>`。探针 p5、p2 说明今天删行与写不存在的文件都放行。
- **A5 关的是哪几条——只能核一半。**
  - 骨架已打节标题与占位。可做：节在、正文非空且不是占位。
  - 不能做：是不是「已经抓到的问题」、每条有没有验收标准——语义。
  - 字样：节标题「## 关的是哪几条」，下面至少一个列表项或编号行。
- **B8 续派指到前一份报告——能（限定）。**
  - 骨架不打。可做：执行员续做已有闸 ⑪ 判「是不是续做」，在那里要求「承接：<上一份报告路径>」且路径在工作根下存在；别的类型没有「是不是续派」的标记，要么所有类型都打「承接：<路径 或 无（首派）>」，首派写「无」就能蒙过去。
  - 字样：行首「承接：」。
- **B9 内存上限——能。**
  - 骨架不打。可做：给调查员、工具员（也可加执行员）打「内存上限：<填 带单位，例 16G>」，闸核在、带单位、不是 `auto`（`experiment-runner.md:32` 写「写明数值，不写 `auto`」）。
  - 因为 `agent-common.md:60` 有回退值，缺这一行不致返工，优先级低。
  - 字样：行首「内存上限：」加数字与单位。
- **B10 自证在哪跑——能。**
  - 骨架已打（四种类型，`dispatch-prompt.py:218`）。可做：闸核值是「不跑」，或本仓主工作区路径；值指到工作根、`/tmp`、副本的拒（重型闸会拒副本里的 `--selftest`，`dispatch-prompt.py:131`）。
  - 字样：行首「自证在哪跑：」。
- **D3 你不碰的文件——能（浅）。**
  - 骨架不打。可做：给实现员、工具员打「不碰：<填 别的会话在改的文件；没有写「无」>」，闸核在。
  - 内容是主 agent 现查的事实，闸只核在不在；实现员互撞已有 ⑤ 拦，补上的只是别的会话与工具员那一半。价值低。
  - 字样：行首「不碰：」。
- **E2 弄坏开关与红样本——能（浅）。**
  - 骨架不打。可做：给工具员打「弄坏开关与红样本：<填 每条改法一行；给不出的写「由工具员起名」>」，闸核在。
  - 字样：行首「弄坏开关与红样本：」。
- **C4 实验两段派法——不能。** 它是派发次序（先装置、再按段并行），不在任何一份提示里；要判得读登记、看段间共用不共用，不是「一行在不在」。

## 五、没做什么

- 没改仓里任何文件，没提交，没跑重型测试，没编译。探针只调了 `dispatch-prompt.py --check` 与闸自己的 `--check-prompt`，提示文件都在草稿目录 `probe/` 里。
- 没读 `.claude/settings.json`，所以派发那一刻是否还有别的 hook 挂在 Agent 工具上我没核；`runner-dispatch-guard.sh` 里「派的那一刻另判」的 ⑦（并发与限额窗口）、⑯（草稿目录写者锁）没测（探针只走 `--check-prompt`，这两判在 `--check-prompt` 下不真判）。
- 没去会话记录里核 E171 #14、#18、#19、#20、#21、#31、#33 的提示原文，戊类各次漏哪一条取自盲判的「缺的是哪一步」与引文；盲判第七节自己写了推翻条件（第二节末）。
- 没核 `.claude/rules/implementation-workflow.md`、`.claude/skills/` 下有没有另一批会进派发提示的要求（派发提示的要求来源按任务点名的五处读：`main-agent.md`「派发提示怎么写」一节、`agent-common.md`、`test-script-discipline.md`、`verification.md`、各定义「输入」一节；`main-agent.md` 别处顺带引到的 A2、A5、C4、F4 也列了，没穷尽）。
- 没核 `agent-common.md:13`、`:14`（model、effort 不超过 high）：它是 Agent 调用的参数，不是提示里的一行，闸能不能读到 effort 我没查，所以没列进表。
- 没判「该不该做成检查」，只列了能不能做；做哪几条、先后由主 agent 与用户定。
- 报告分段写入，附录那一段（约 170 行）因为要改掉被 `cut -c` 切坏的中文，是整段重写进去的，超过了「每次不超过 150 行」；内容就是附录里贴的命令输出。
- 草稿目录里的 `probe/`（四份探针提示与输出，几 KB）留着给主 agent 复核；没有仓副本、编译目录、库拷贝。

## 附录：命令与原样输出

命令里的 `cc N` 是草稿目录里的 `cc.py`，按字符取每行前 N 个字（GNU `cut -c` 按字节，会切坏中文）；完整文件在草稿目录（`greps1.txt`、`greps2.txt`、`greps3.txt`、`probes.txt`）。

### G1 派发闸与骨架里的命中（greps1.txt）

```
$ grep -n "重型测试：不跑" .claude/hooks/runner-dispatch-guard.sh | head -2 | cc 150
8:#   ① 重型测试一行：派任何类型，提示里要有一行「重型测试：不跑」（重型测试只由主 agent 跑）（行首可带 - * > 与粗体）。
16:#      同一子句后面写了不跑 / 归提交 / 重型的、后面接 clippy / build / 编译的不算；「」『』“”里的转述、「重型测试：不跑」那一截、提到主 agent 的子句不判。
$ grep -n "重型测试：不跑" scripts/agents/dispatch-prompt.py | head -2 | cc 150
182:HEAVY_TEST_OWNERS = set()   # 子 agent 一个都不跑重型测试，骨架一律带「重型测试：不跑」
330:        lines.append("重型测试：不跑")
$ grep -n "批次" .claude/hooks/runner-dispatch-guard.sh | cc 120
3193:                 "批次：x#1。回复用中文。草稿目录 `~/.local/share/singlefs/work/w/`（自己建，排他），报告 `~/.local/share/singlefs/work/w/re
$ grep -n "批次" scripts/agents/dispatch-prompt.py | head -2 | cc 120
331:    lines.append(f"批次：{batch or '<填 调度记录#批号>'}")
476:        wanted = ["重型测试：不跑", "批次：<填 调度记录#批号>", "草稿目录：~/.local/share/singlefs/work/x/", "报告：~/.local/share/singlefs/w
$ grep -n "LONG_RUN_AGENTS =\|THREAD_LIMIT_REQUIRED_AGENTS =" .claude/hooks/runner-dispatch-guard.sh | cc 260
1247:THREAD_LIMIT_REQUIRED_AGENTS = {"implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack",
1888:LONG_RUN_AGENTS = {"experiment-designer"} | THREAD_LIMIT_REQUIRED_AGENTS   # 设计员，加上 ⑮ 那张任务带重活的类型表（用户 2026-10-12 定的判定射程）
$ grep -n "LONG_RUN_TYPES\|HEAVY_WORK_TYPES" scripts/agents/dispatch-prompt.py | head -4 | cc 200
32:骨架的两张类型表 HEAVY_WORK_TYPES、LONG_RUN_TYPES 与派发闸的 THREAD_LIMIT_REQUIRED_AGENTS、LONG_RUN_AGENTS 逐个相同，自检解析派发闸那两行逐个比。
184:HEAVY_WORK_TYPES = {"implementation-writer", "experiment-runner", "three-way-attack", "investigator", "tooling-writer", "clerk", "mutation-triage",
187:LONG_RUN_TYPES = ("experiment-designer", "implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack",
189:LONG_RUN_TYPES_BEFORE_WIDENING = ("experiment-designer", "experiment-runner")
$ grep -n "HARD_CODED_QUOTA" .claude/hooks/lib_long_run.py .claude/hooks/runner-dispatch-guard.sh | cc 200
.claude/hooks/lib_long_run.py:25:    两种都不许写死额度（HARD_CODED_QUOTA：数字后面跟线程、核、台、K/M/G/T，或 --threads N、-j N、--memory-cap N、--shard k/n、THREADS=N 这类）。
.claude/hooks/lib_long_run.py:114:HARD_CODED_QUOTA = re.compile(
.claude/hooks/lib_long_run.py:180:        quotas = [match.group(0).strip() for match in HARD_CODED_QUOTA.finditer(hosts)]
$ sed -n 176,184p .claude/hooks/lib_long_run.py | cc 200
        else:
            unnamed = [read for read in HOST_READS if read not in hosts]
            if unnamed:
                problems.append(f"{spelling['hosts']} 一行没点名 {'、'.join(unnamed)}：机器清单只从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存从 host-resources.py --json 现算")
        quotas = [match.group(0).strip() for match in HARD_CODED_QUOTA.finditer(hosts)]
        if quotas:
            problems.append(f"{spelling['hosts']} 一行写死了额度：{'、'.join(quotas)}（台数、线程、内存要按多机配置与现查算，不写常量）")
    missing = [key for key in REQUIRED_KEYS if key not in written]
    if missing:
$ grep -n "LEGACY_ROOT_IN_TRANSITION = " .claude/hooks/lib_work_root.py
25:LEGACY_ROOT_IN_TRANSITION = True
$ grep -n "复用\|持久\|开机" .claude/hooks/runner-dispatch-guard.sh .claude/hooks/lib_long_run.py | cc 170
.claude/hooks/lib_long_run.py:2:判据：.claude/rules/test-script-discipline.md「用上现有的硬件加速」与「判定可复用：按输入存，按前缀剪」。
.claude/hooks/lib_long_run.py:426:        print(f"     → 怎么办：在 {script} 文件头的注释块里补齐（判据 .claude/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）："
.claude/hooks/runner-dispatch-guard.sh:70:#      .claude/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）：
.claude/hooks/runner-dispatch-guard.sh:1891:                 "（.claude/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）。\n")
.claude/hooks/runner-dispatch-guard.sh:3193:                 "批次：x#1。回复用中文。草稿目录 `~/.local/share/singlefs/work/w/`（自己建，排他），报告 `~/.local/share/singlefs/work/w/report.md`，交回
$ grep -c "内存上限" .claude/hooks/runner-dispatch-guard.sh
0
$ grep -c "自证在哪跑" .claude/hooks/runner-dispatch-guard.sh
0
$ grep -n "自证在哪跑" scripts/agents/dispatch-prompt.py | head -3 | cc 140
24:「自证在哪跑」一行派发闸不单判，没填时按占位拒。
105:  DISPATCH_PROMPT_BREAK=selftest-location-dropped  skeleton() 不打「自证在哪跑」一行
106:      会跑自证的类型骨架带这一行（「✗ 自检：会跑自证的类型骨架应当带一行「自证在哪跑：<填 …>」」）、
$ grep -n "SELFTEST_RUNNER_TYPES =" scripts/agents/dispatch-prompt.py | cc 200
218:SELFTEST_RUNNER_TYPES = ("tooling-writer", "implementation-writer", "investigator", "experiment-runner")
```

### G2 各条判法的命中（greps2.txt）

```
$ grep -n 'required-inputs' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
852:    groups = [group.strip() for group in (fields.get("required-inputs") or "").split(",") if group.strip()]
1962:    """③ 缺定义 required-inputs 的拒绝说明：每组按字面子串找的是哪几个字，再给能照抄进提示的几行（与 dispatch-prompt.py 骨架同一个写法）。"""
1965:        return (f"✗ 派 {subagent_type} 的提示缺它定义要的输入：{'、'.join(missing)}（{definition} 的 required-inputs）。\n"
$ grep -n '要动的 crates 文件' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1035:        return ("✗ 派 implementation-writer 的提示里没有「要动的 crates 文件：…」一行。\n"
2376:            if subagent_type == "implementation-writer" and footer_text and "要动的 crates 文件" not in text:
2378:                text += f"要动的 crates 文件：crates/selftest/src/case{writer_case_counter[0]}.rs\n"
$ grep -n '这一段回答的岔路' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1980:    """⑪ 执行员：这一段回答的岔路；续做时上一段岔路表里还差的行。交回拒绝说明，放行交 None。"""
1983:    answered = labelled_value(prompt, "这一段回答的岔路")
1985:        return ("✗ 派 experiment-runner 的提示里没写「这一段回答的岔路：…」。\n"
$ grep -n '上一段岔路表里还差' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1980:    """⑪ 执行员：这一段回答的岔路；续做时上一段岔路表里还差的行。交回拒绝说明，放行交 None。"""
1995:    remaining = labelled_value(prompt, "上一段岔路表里还差")
1997:        return (f"✗ E{number_match.group(1)} 已经有实验页，这是续做，派发提示里却没写「上一段岔路表里还差：…」。\n"
$ grep -n '规格检查已判' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1077:    if valve_reason(prompt, "规格检查已判") is not None or break_switch_is("spec-check-skipped"):
1110:                    "\n→ 怎么办：改规格原文再派；判断不了的交用户；要写回员自己起草就给它判决与条目、不给规格。确要照这份派，提示里写一行「规格检查已判：<理由>」。")
1113:                    "→ 怎么办：照 scripts/kb/kb-spec-check.py 文件头「规格的两种写法」写，或给判决让写回员自己起草；确要照原样派，提示里写一行「规格检查已判：<理由>」。")
$ grep -n '通用已判' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
2165:    if (subagent_type in GENERAL_PURPOSE_TYPES and valve_reason(prompt, "通用已判") is None
2169:                   "只读的搜索派 Explore；确要派内置类型，提示里单起一行「通用已判：<为什么 clerk 做不了>」。"))
2496:            case_in(defs_work, "共用约束:通用 agent 没写开工先读", "general-purpose", heavy_line + "通用已判：要 WebSearch 查外部文档\n查一个文件。", 2, footer=False),
$ grep -n '开工先读' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
799:COMMON_CONSTRAINTS_READ = re.compile(r"开工先读[^\n]*\.claude/agent-common\.md")
2177:        refusals.append(("开工先读共用约束", f"✗ {subagent_type} 不是项目定义的 agent，不读共用约束，提示里却没写「开工先读 `.claude/agent-common.md`」。\n"
2179:                   "查真假与机理派 investigator）；确要派它，提示里写一行「开工先读 `.claude/agent-common.md`」，长活照那份的「长活交回主 agent」写。"))
$ grep -n '写者锁已判' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1829:    valve = valve_reason(prompt, "写者锁已判") is not None and not break_switch_is("writer-lock-valve-ignored")
1860:                     "与写闸（.claude/hooks/write-guard.sh 第九道）同一口径按「锁在而判不了」办：不当没锁，也不接手，「写者锁已判」行在这里不接手判不了的锁。\n"
1869:                     "→ 怎么办：等持锁的交回再派，或给这一件换一个草稿目录；持锁的确实已经失效（客户端重启后留下的旧进程、已经停掉的），在提示里单起一行「写者锁已判：<理由>」接手，"
$ grep -n '线程上限已判' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1248:                                "clerk", "three-way-forward", "three-way-defense"}   # 后三种：clerk 的定义许编译、跑编译出来的代码（clerk.md）；three-way-forward、three-way-defense 的定义只跑模
1391:    不等于登记值、认不出数的拒；THREAD_LIMIT_REQUIRED_AGENTS 里的类型没写这一行也拒；除非另写「线程上限已判：<理由>」。代码块（照 CommonMark，without_code_blocks）里的行不判（里面的已判行也不算）。交回拒绝说明，放行交 None。"""
1397:            return code_block_ambiguity_refusal(subagent_type, ambiguous, "「线程上限：」「线程上限已判：」")
$ grep -n '准入已判' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170
1122:    if match is None or break_switch_is("admission-ignored") or valve_reason(prompt, "准入已判") is not None:
1133:                "→ 怎么办：沿用上次产物与结论；要为别的原因重跑（换臂、换判据），在提示里写一行「准入已判：<哪一样变了>」。")
2619:            case("准入:写了放行行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。\n准入已判：换了一条臂\n", 0),
$ grep -n '被测条款' .claude/hooks/runner-dispatch-guard.sh | grep -v '^[0-9]*:#' | head -3 | cc 170

--- skeleton
$ grep -n '要动的 crates 文件' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
19:定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、
211:for long_run_type in LONG_RUN_TYPES:   # 执行员、设计员上面已经带了；别的几种在这里补（要动的 crates 文件之类的类型专属行排在前面）
$ grep -n '这一段回答的岔路' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
19:定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、
$ grep -n '上一段岔路表里还差' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
$ grep -n '规格检查已判' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
19:定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、
$ grep -n '禁读清单' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
39:本腿的报告只认报告行点名的 research/prompts/<名>-output.md，或报告行里没有时整份提示里不在引号里、不在禁读清单里、名字以本腿轮名起头的那一处；
40:一处都没有、报告行点名的文件落在草稿目录下或是裸文件名又不是这一形态（例如草稿目录下的 result.md）、禁读清单点到本腿自己的报告、本腿报告路径的名字里带 report 都拒，
41:交回闸只认这种形态。骨架照填：「报告：」一行写本腿的，「禁读清单：」一行只写别的腿的报告与模型目录。
$ grep -n '岔路单' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
$ grep -n '被测条款' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
$ grep -n '关的是哪几条' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
$ grep -n '出口' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
16:为什么：2026-09-20 起 8 天里派发闸拒了 187 次，前几种原因全是缺一行（岔路行、要改的文件、出口、还差）；主 agent 派完再给子 agent
$ grep -n 'required-inputs' scripts/agents/dispatch-prompt.py | grep -v '^[0-9]*:[#「 ]' | head -3 | cc 170
4:"""派发提示的骨架：按定义的 required-inputs 与共用约束要的固定行打出来，主 agent 只填内容；核一份提示还剩不剩占位。
19:定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、
38:轮与模型两处主 agent 填（不填 --check 照占位拒）；定义的 required-inputs 没要「报告」的也打这一行，要了的只打一行。派发闸 ⑰ 对这几种另判形态（判法只在派发闸文件头 ⑰ 条）：
```

### G3 补查（greps3.txt）

```
$ grep -n '弄坏开关' .claude/hooks/runner-dispatch-guard.sh scripts/agents/dispatch-prompt.py | grep -v ':[0-9]*:#' | head -3 | cc 120
scripts/agents/dispatch-prompt.py:9:    dispatch-prompt.py --selftest                                                   
scripts/agents/dispatch-prompt.py:46:弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自
scripts/agents/dispatch-prompt.py:163:THREE_WAY_REPORT_TYPES_BEFORE_REGISTRY = ("three-way-forward", "three-way-attack",
$ grep -c '不碰的\|你不碰' .claude/hooks/runner-dispatch-guard.sh scripts/agents/dispatch-prompt.py
scripts/agents/dispatch-prompt.py:0
.claude/hooks/runner-dispatch-guard.sh:0
$ grep -n '前一份报告\|上一份报告' .claude/hooks/runner-dispatch-guard.sh scripts/agents/dispatch-prompt.py | head -3 | cc 120
$ grep -n '/tmp' .claude/hooks/runner-dispatch-guard.sh | grep -v ':[0-9]*:#' | cc 140
27:#      工作根（默认 ~/.local/share/singlefs/work/）认展开的绝对路径与 ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 几种写法，过渡期旧根认 /tmp/claude-<uid>/，
2252:    "**抓到的实例**：实现员把「库单测 → 动到的测试二进制 → 新层 0 七条流全量 → 变异」写进 `/tmp/claude-1000/impl-m2-treesplit/run-chain.sh`，再用后台 Bash 起它",
2521:                      "草稿目录 /tmp/claude-1000/e/\n报告写进 /tmp/claude-1000/e/result.md\n这一段回答的岔路：岔路 1\n", 0),
$ grep -n '旧根\|legacy_root\|scratch_roots\|lib_work_root' .claude/hooks/runner-dispatch-guard.sh | grep -v ':[0-9]*:#' | head -5 | cc 140
25:#      与「写进 / 写到 / 更新」后面紧跟的路径（工作根与过渡期旧根下的、research/、.claude/、crates/、litmus/、records/ 起头）逐个按它那几行模式判，不在里面就拒；
27:#      工作根（默认 ~/.local/share/singlefs/work/）认展开的绝对路径与 ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 几种写法，过渡期旧根认 /tmp/claude-<uid>/，
29:#      工作根怎么认只在同目录的 lib_work_root.py；旧根在跑的子 agent 与别的会话的子 agent 都交回之后由主 agent 撤（撤法在那份库的文件头）。
39:#      （AGENT_HOOK_DETECTIONS，没设取 lib_work_root.py 的 default_detections_path）追加一行、结局只记，看门狗据此报给主 agent（lib_snapshot_conflicts.py 的 unreada
62:#   ⑧ 书记员：提示里点名的工作根或过渡期旧根下的规格文件逐份交给 scripts/kb/kb-spec-check.py，红了拒；一份认得出的规格都没有也拒
```

### P 探针与环境（probes.txt）：p2 是一份预计超过 2 小时的 clerk 提示，草稿目录写 /tmp、正文写死线程与内存、没提复用；p3 把分片、续跑两行改成「好」；p4 删续跑行；p5 删批次行；p6 删认领行

```
$ python3 scripts/agents/dispatch-prompt.py --check probe/p2.txt --type clerk --root <仓根>
rc=0
  ✓ 派发闸照这份提示放行（clerk，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）

$ python3 scripts/agents/dispatch-prompt.py --check probe/p3.txt --type clerk --root <仓根>
rc=0
  ✓ 派发闸照这份提示放行（clerk，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）

$ python3 scripts/agents/dispatch-prompt.py --check probe/p4.txt --type clerk --root <仓根>
rc=3
✗ 派 clerk 的提示写了预计挂钟超过 2 小时，长活声明不齐：
  缺 续跑（续跑：），也没有 分不了： 一行写明缺哪一段
→ 规矩：预计挂钟超过 2 小时的活默认分片、断点续跑、按多机配置自适应；多机配置里每台线程取登记值 .claude/heavy-thread-default，不现算（.claude/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）。
→ 怎么办：超过 2 小时的，同一份提示里写三行：「分片：<分片单元>」「续跑：<同一条命令重起只补没做完的，怎么认做完的>」「多机：台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量」（多机这一行只认这一句，逐字，可带句末句号；后面不接任何说明，要说明另起一行写，别的说法一律不认；用户给了逐台不同的台数或线程的，另加一行「多机已判：<理由>」，逐台参数再另起一行照普通正文写）；分不了的写「分不了：<点名缺的分片 / 续跑 / 多机与为什么，至少 8 个字>」。骨架 python3 scripts/agents/dispatch-prompt.py <类型> 带这几行。
→ 派发闸照这份提示会拒 1 处（预计挂钟与长活声明），照上面逐处改完再核；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）

$ python3 scripts/agents/dispatch-prompt.py --check probe/p5.txt --type clerk --root <仓根>
rc=0
  ✓ 派发闸照这份提示放行（clerk，查了 20 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）

--- p2 与 p3、p4、p5 的差别（diff p2 pN）
diff p2 p3:
10,11c10,11
< 分片：按整表行切
< 续跑：重起只补没做完的
---
> 分片：好
> 续跑：好
diff p2 p4:
11d10
< 续跑：重起只补没做完的
diff p2 p5:
4d3
< 批次：records/x.md#1

--- p6（p2 去掉认领行）
$ python3 scripts/agents/dispatch-prompt.py --check probe/p6.txt --type clerk --root <仓根>
rc=3
  ✓ 派发闸照这份提示放行（clerk，查了 20 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）
  ✗ 认领行：有「草稿目录：/tmp/claude-1000/e171-r1-segment1/」却没有「认领：bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '<用途一句>'」一行
  → 怎么办：「草稿目录：」那一行之后写一行「认领：bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '<用途一句>'」，目录与上一行一致（骨架本来就带这一行）
$ bash .claude/hooks/runner-dispatch-guard.sh --check-prompt probe/p6.txt --type clerk --root <仓根>
rc=0
  ✓ 派发闸照这份提示放行（clerk，查了 20 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）

$ findmnt -T ~/.local/share/singlefs/work -o TARGET,FSTYPE
TARGET FSTYPE
/      ext4
$ findmnt -T /tmp -o TARGET,FSTYPE
TARGET FSTYPE
/      ext4
$ grep -n "^D" /usr/lib/tmpfiles.d/tmp.conf
11:D /tmp 1777 root root 30d
```
