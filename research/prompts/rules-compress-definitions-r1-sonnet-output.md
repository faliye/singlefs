# rules-compress-definitions-r1 正推腿报告（three-way-forward）

日期：2026-11-16。轮：rules-compress-definitions-r1，第一轮，前几轮判决：无。批次：`records/2026-11-16-三方豁免取消与存量补轮.md#压缩 2b`。草稿目录 `~/.local/share/singlefs/work/rules-compress-definitions-r1-sonnet`（已认领）。我没往任务服务交任何任务，没跑模型。

## 先说结论（证据在后面各节，末尾有判定一览）

- 格一 丢失：成立，1 处（边界）：共用约束里「想拿旧数据支撑新结论，先复跑」那句，合稿只剩两跳指针，触发条件也换了。另 1 处弱（报告一节丢了「在不影响正确性的前提下」）。其余原件规则都在合稿树里找到了落点：共用约束 175 行、14 份定义 407 行，见「逐条落点表」。
- 格二 二义：成立，2 处，都从原件继承、矛盾总表已登：读文件的射程（共用-2）、实现员命名检查的 `<根>`（共用-10，登的是「没改」）。
- 格三 常驻够不够：不成立。
- 格四 子 agent 读得到：不成立（实质上）。字面上有 5 个目标是主 agent 一侧的文件，指针只管主 agent 的动作，子 agent 不用打开；逐个指针见「指针核对表」。
- 格五 悄悄改了做法：不成立（我核过的范围内）。合稿新加、原件没有的「开工另读」入口 3 处，是下沉后补的入口，不算改做法，列在格五。
- 用户定案：我这一组里有载体的 1 到 16 都落了；17、18、19 合稿里没落，单列「待落」，不算打中。

## 输入核对（量的）

- 原件 15 份：`sha256sum -c research/prompts/rules-compress-definitions-r1-snapshot/judged-sha256.txt`，输出 15 行 `OK`、0 行不是 `OK`（退出码 0）。
- 合稿树里这 15 份：从 `research/prompts/rules-compress-definitions-r1-snapshot/merged-tree-sha256.txt` 取出这 15 行，在合稿树根下 `sha256sum -c`，输出 15 行 `OK`（退出码 0）。
- 14 份定义的 frontmatter（前 10 行）原件与合稿逐份相同（`sed -n 1,10p` 比 sha256，14 份都 same）。

## 做法

- 次序照派发提示：共用约束；implementation-writer、experiment-runner、experiment-designer、tooling-writer、kb-scribe、investigator；其余 8 份（clerk、mutation-triage、prior-art、sweep、three-way-attack、three-way-defense、three-way-forward、three-way-materials）。15 份都逐条过到了，没有「推不出」的整份。
- 共用约束：我手工逐条配了 175 行落点（原文头与合稿头都先在那一行里 `grep` 式核过，核不到脚本就停）。
- 14 份定义：脚本把原件每一句按 5 字串重合度配到合稿树里最像的那一句（先同名文件，同名里配不上再搜整棵树），得 407 行；其中重合度不到 55% 的 88 行我逐条人工核了，改配 37 行，其余是下沉、拆分、改写，备注里写了去向。
- 三条补查：① 原件里的每个反引号字面量（命令、路径、变量名）在合稿树里还找不找得到，找不到的 109 个，人工分到：钩子识别细节下沉（用户定-7）、并掉的文件名指针、`**` 通配记法的假阳性，没有一条是被丢的命令；② 合稿里 15 份里每个「文件」「节标题」指针，节标题是否真在那份文件里（合稿树优先，上游与别处读仓里的）；③ 合稿里每个「共用约束「条目」」是否在合稿共用约束里找得到。②③ 的结果：节标题缺 0 处（裸文件名的 9 处我另手核了：`test-script-discipline.md`、`evidence-discipline.md` 的小节都在），共用约束条目缺 0 处。
- 钩子文件头核对：合稿共用约束里说钩子拒什么的句子，我逐句对了 `bash-command-detector.sh` 的 ⑥ ⑨ ⑪ ⑯ ⑰ ⑱ 与 `write-guard.sh` 的第六道、第十道文件头，对得上；没逐句核的列在格五的「推不出」。
- 辅助脚本都在草稿目录，只读仓与合稿树，没写仓：`ptrcheck.py`、`cover.py`、`gen_defs.py`、`assemble_tables_v2.py`、`common_rows_v2.py`、`build_report_tables_v2.py`、`ptr_table.py`、`expand2.py`。

## 格一 丢失：成立（1 处边界，1 处弱）

**命中 1（边界）：旧数据支撑新结论先复跑。** 原件把「拿旧数据支撑新结论之前先重新跑一遍」直接指到上游 kb-discipline 那一条；合稿把指针改指 evidence-discipline，那里的触发条件换成「对早先的结论有疑问」，原话只在再往下一跳的上游文件里。
- 原件：「拿它支撑新结论之前先重新跑一遍」(`.claude/agent-common.md:35`)
- 合稿共用约束：「读到的是当时的数、不是结论，怎么用照」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`)
- 合稿指到的那一条：「对早先的结论有疑问，重新验证」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:61`)
- 原话所在（上游，合稿没动）：「想拿它支撑新结论，先复跑一遍，确认今天还成立」(`.claude/singlefs-ai-sop/rules/kb-discipline.md:39`)
- 量的：`grep -rn '支撑新结论\|先复跑一遍\|重新跑一遍'` 扫合稿树里除 `contradictions.md`、`merge-result.md`、`references.md` 外的全部 `.md`，0 行命中。
- 做法差：原件是「只要拿它支撑新结论就先复跑」；合稿读法是「有疑问才重新验证、不去翻旧证据」。没有疑问而直接拿 `git log` 里读到的旧数当依据，原件不许、合稿没拦。
- 什么现象会推翻它：主 agent 认定「想拿它支撑新结论，先复跑一遍」已由 evidence-discipline 那一行的「重新验证」覆盖，或共用约束补回那一句。

**弱 1：报告一节的前提。** 原件写「在不影响正确性的前提下写简练」，合稿共用约束只写「写结论、依据与没做什么」，前提没了；同一句前提还在合稿 main-agent 的派发提示那一条里。
- 原件：「交回与报告在不影响正确性的前提下写简练」(`.claude/agent-common.md:119`)
- 合稿：「写结论、依据与没做什么，不写做事的经过」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:114`)
- 同句前提在别处：「在不影响正确性的前提下写简练」(`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:82`)
- 我不把它算打中：同一段里 「要求的命令与原样输出、行号、推翻条件照样给」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:114`) 已经把不能省的项列死了。

**核过、不算丢的几处（防主 agent 重核）**
- 共用约束 :4 的「也不留解释的尾巴」：合稿共用约束删了半句，原话在 「定义里不写、也不留解释的尾巴」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:17`)，共用约束 :4 指到那一节。
- 多机那一条里「许读的只有多机配置与私有名单」「环境只从 `multi-host.env` 读、没写就是没有」「测试自己的配置文件进仓或放共享根」：落在 「许读的本机私有状态只有」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:62`) 与 「一条测试自己的配置文件」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:63`)，共用约束 「写这几类的，开工另读」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`) 点名读那一节。
- 实现员「收的做成一份只含这张表的补丁目录并进这一批」：落在 「收的补进拒算补丁目录、不收的退给实现员改测试」(`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:55`)（主 agent 一侧）。
- 实现员「代码轮三方拿走不到的理由每一处重新推」：落在 「走不到的理由二次查实」(`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:82`)。
- 回扫员「钉值类的行先定条款再派实现员」：落在 「钉值类的照」(`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:78`)。
- 各定义里删掉的「主 agent 跑完新派……（带「承接：」）读 task log 接着做」：共用约束 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) 管全部定义。
- 各定义里删掉的「上下文多大都不交回」「全文写进报告文件，交回只写结论、报告路径与 sha256sum」：共用约束 「上下文多大都不是交回的理由」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`)、「其余定义的报告全文写进派发提示给的报告路径」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`)。
- 实现员「不缩写、不写 `_ =>`、newtype」：上游 「封闭集合的枚举，match 不写」(`.claude/singlefs-ai-sop/rules/code-discipline.md:100`)、「各给一个 newtype」(`.claude/singlefs-ai-sop/rules/code-discipline.md:128`)，实现员开工先读那一份全篇。

## 格二 二义：成立（2 处，都是继承来的）

**命中 1：读文件的射程。** 共用约束里相邻三条，第三条的主语「别的文件」没写明排除第二条管的那些。
- 第二条：「点名的别的文件只读点名的小节」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:18`)
- 第三条：「别的文件：1,000 行以内的整读一次」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:19`)
- 两种读法：① 第三条只管第二条没管的文件（没被「开工先读」点名的）→ 被点名的 kb 文件 1,000 行以内也只读点名的小节；② 第三条是通则，被点名的 kb 文件 1,000 行以内整读一次。做法不同（读小节还是整份）。
- 原件的同一对：「点名的别的文件，只读点名的那几个小节」(`.claude/agent-common.md:23`) 与 「读文件：1,000 行以内的整读一次」(`.claude/agent-common.md:73`)。矛盾总表说两条射程分开了：「两条射程分开写」(`research/prompts/rules-compress-r1-merged/contradictions.md:56`)；合稿的字面没把「分开」写出来。
- 什么现象会推翻它：共用约束第三条加一个限定语（「没被『开工先读』点名的文件：」之类）。

**命中 2：实现员命名检查的 `<根>`。** 实现员定义第 1 步的括注把命名检查包进「填你改动所在的树」，共用约束写实现员一律在副本里改、交补丁，命名检查的 `<根>` 写 `<快照>`。
- 实现员定义：「命名检查与带 --item 的门禁与崩溃放量入口照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:31`) 同一行 「填你改动所在的树」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:31`)（实现员改动所在的树是草稿目录的副本）。
- 共用约束：「实现员一律在副本里改、交补丁」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:109`)、「写 <快照>」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:109`)。
- 两种读法：副本根 vs 快照；主 agent 交的命名检查对着哪棵树判，结局不同。
- 矛盾总表登的是「没改」：「写手拿不准、没改」(`research/prompts/rules-compress-r1-merged/contradictions.md:64`)。
- 什么现象会推翻它：实现员定义第 1 步把「命名检查」从括注覆盖的范围里拿出来，指到共用约束那一条。

**待落带来的二义（不算打中，见「待落」）**：调查员「查到就修」在主工作区改，实现员定义写「一律在草稿目录的副本里改……不碰主工作区」，而调查员的第 6 步要「照实现员定义第 1d、2、3、3a、3b、4 步」写——读者分不清它算不算实现员（用户定-19 要补的就是这一句）。

## 格三 常驻够不够：不成立

- 这一组里下沉的有三类：钩子怎么认写法的细节（用户定-7）、写者锁的认法、四条带 `--item` 的整条命令（从共用约束移到实现员定义第 1d 步）。三类都留了「什么时候读哪一份」：
  - 钩子细节：「逐条以它的文件头为准」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:73`)，且「拒哪类写法 + 出路」在共用约束里每类一行；
  - 写者锁：「认法在 .claude/hooks/lib_writer_lock.py 文件头」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:34`)，子 agent 要做的（被拒就停、写进报告）在共用约束里；
  - 带 `--item` 的四条：「整条照 .claude/agents/implementation-writer.md」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:60`)，实现员与调查员的定义都点名了第 1d 步。
- 不读钩子文件头就会做错的一步我没找到：合规写法在共用约束里都留了（例：⑥ 「放行 kill」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:80`)、⑧ 「出路：仓内文件用 Edit」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:82`)、⑲ 「python3 scripts/kb/append-record.py」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:92`)）；被拒时钩子的 stderr 另给出路。
- 主 agent 一侧下沉的（技能、main-agent）：子 agent 不用读，不在这一格判。
- 什么现象会推翻它：找到一步子 agent 要照做、只写在钩子文件头或写者锁文件头里、共用约束与定义里没有合规写法的。

## 格四 子 agent 读得到：不成立（实质），指针逐个核见下一节

- 实质：15 份文件里指到别的文件的 121 个目标（量的，`ptr_table.py`），分五类：规则文件 27 个（共用约束 「点到就整份读」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`)）；kb 与上游技能 15 个（定义的「开工先读」或该步正文点名）；脚本、钩子、门禁表 72 个（该步执行时要用，指针都带「文件头」「写法照」）；定义互指 2 个；主 agent 一侧 5 个。
- 字面上子 agent 不读的是第五类 5 个：`.claude/main-agent.md`、`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md`、`.claude/skills/three-way/SKILL.md`、`records/2026-10-29-subagent拆分提案.md`。每一处句子的主语都是主 agent（「由主 agent 跑」「主 agent 照……」），子 agent 的动作不依赖打开它们；原件同样指这些文件。
- 合稿新加的指针里，指到规则文件而又被子 agent 照做的，都在共用约束或定义里点了名（例：「写测试、门禁、入口脚本、实验装置的，开工另读」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`)）。
- 材料员的链：「三处都管」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:16`)整节 → 规则文件里点名读技能 「写材料、生成清单与附录之前先读」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:22`)；规则文件整份读，所以技能被点到。
- 什么现象会推翻它：找到一处子 agent 要照做的步骤，其指针指到子 agent 没被任何一层点名读的文件。

## 格五 悄悄改了做法：不成立（我核过的范围内），1 处推不出

- 做法差我逐条对过矛盾总表与用户定案的，都登记在册（或本身是措辞）：用户定-1/5/6/7/10/12，矛盾表 共用-1 到共用-10、定义-1 到定义-11（我这一组里有载体的）。没找到登记之外的做法改动。
- 注（不算打中）：合稿新加三处「开工另读」，原件只写指针、没写读取要求：「写测试、门禁、入口脚本、实验装置的，开工另读」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`)、「写这几类的，开工另读」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`)、「写门禁、改算键的代码时另读」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:16`)。它们是格三要的入口，不是丢失；矛盾总表没登。
- 注（钩子今天的样子与用户定-6 不一致，属脚本一侧、第 13、15 条的活）：钩子 ⑥ 文件头今天仍放行 「对自己用新会话起的那一个子进程的进程组发信号」(`.claude/hooks/bash-command-detector.sh:184`)，并把「在循环里逐个发」当违规、同一份判定由门禁 research-gate-lint 扫脚本 「同一份判定由门禁 code-tooling 的 research-gate-lint 格拿去扫」(`.claude/hooks/bash-command-detector.sh:192`)；合稿要求样本进程逐个按进程号停、连同子孙：「超时与结束时逐个按进程号发信号」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`)，在脚本里要写循环。这一条合稿与矛盾总表都没提钩子一侧的配套，请主 agent 登进第 15 条那一批（推的：这句靠读钩子文件头，没跑钩子）。
- 推不出的：合稿共用约束里项目代码闸（:95）、范围声明闸（:101）、交回正文闸（:103）、只管主 agent 的三道（:104）这几段话，我没逐句对钩子文件头；它们是原件对应段落的压缩，保留了合规写法与出路，但「拒什么」的字面我没一句句核。
- 什么现象会推翻它：找到一处原件与合稿做法不同、矛盾总表与用户定案里都没有的地方（我没找到）；或找到钩子文件头与合稿共用约束说法对不上的句子。

## 用户定案 1 到 19：落没落对（只核落点，不核该不该）

只列我这一组（共用约束 + 14 份定义）里有载体的；没载体的写「本组无载体」，主 agent 去别的题找。

| 条 | 定案（调度记录） | 本组落点 | 判 |
|---|---|---|---|
| 1 | 实现员一律交补丁，只派一个也这样 | 「一律在草稿目录的副本里改、交补丁目录」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:15`)；「实现员一律在副本里改、交补丁」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:109`)；写范围 「只在草稿目录的副本里改，主工作区的不写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:65`) | 落对。调查员那一支是第 19 条 |
| 2 | 三方开轮「两句管两件事，都留」 | 本组定义只指到 「多轮：一次打穿不算数，三轮里多数打穿才算」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:17`)；`grep` 本组 15 份没有「再攻两轮」「第三轮」字样 | 落对（本组无自己的开轮句） |
| 3 | 并行度缺省全取登记线程数 | 本组 15 份 `grep -n nproc` 0 命中；线程那句 「线程固定是 .claude/heavy-thread-default 登记的每台默认重活线程数」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 落对 |
| 4 | 新测试证红「两样都要」 | 「每条新测试都要证明会红，证红命令照第 1 步写进报告」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:46`)；开工先读 `show-me-test-by-review.md` 的「新增的测试必须先证明它会红」（那一节「两样都要」在规则里，不在本组） | 落对 |
| 5 | 上下文吃紧：执行员例外可交回 | 「例外只有 experiment-runner」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`)；「上下文吃紧或卡住时，停在最后一个写完整的点」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:39`) | 落对 |
| 6 | 进程组信号一律不许，逐个按进程号停 | 「样本进程逐个按进程号停」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`)；「自证、门禁判法与装置里起的进程逐个按进程号停」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:33`)；三方腿 「起样本进程照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:30`)、「起样本进程照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:31`)、「起样本进程照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:39`) | 落对。钩子与门禁一侧没配套，见格五注 |
| 7 | 钩子识别细节下沉，只留合规写法 | 「逐条以它的文件头为准」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:73`) | 落对 |
| 8 | 示例：留格式示例，删代码示例 | 本组无载体 | — |
| 9 | RefFS 依赖只许 std | 本组无载体 | — |
| 10 | `quote-rust-items.py` 进直接跑的名单 | 材料员 「python3 scripts/kb/quote-rust-items.py」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:29`)；共用约束 「名单以 .claude/hooks/lib_project_code_runs.py 的 TEXT_TOOLS_RUN_DIRECTLY 为准」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:58`) | 落对；钩子库今天还没加（量的：`grep -n quote-rust-items .claude/hooks/lib_project_code_runs.py` 0 行），材料员照这句跑会被项目代码闸拒，归第 15 条那一批 |
| 11 | 说明句两类都删 | 本组 15 份没有「这是 singlefs 的项目本地规则」类的开头句 | 落对（无载体） |
| 12 | 钩子已拒而共用约束没列的七类，每类一行 | ⑨ 「`experiment-runner` 在 Bash 里写主工作区」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:83`)；⑪ 「命令位置上的 rm -r 作用在工作根」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:85`)；⑯ 「往现行账本」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:89`)；⑰ 「往仓里的文件写字面内容」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:90`)；⑱ 「带路径撤掉工作区里没暂存的改动」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:91`)；第六道 「第六道：写进」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:97`)；第十道 「第十道：往现行账本写」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:98`)。我逐类对了钩子文件头，出路与拒的写法都对得上 | 落对 |
| 13、14、15 | 脚本一侧的存量与次序 | 本组无载体 | — |
| 16 | 实现员报告节名里的「层 0」留着 | 「受影响的层 0 流与崩溃枚举用例」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:51`) | 落对（原样留着，主 agent 处置-2） |
| 17 | 代码轮打中之后「现查坐实就改，再攻一轮，最多三轮」 | 本组无直接载体；本组两份腿定义（辩方、攻方）开工先读的 「多轮：一次打穿不算数，三轮里多数打穿才算」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:17`) 在合稿里写的是设计推论轮的「先挂起、三轮多数」 | 待落（不算打中）：「打中的写回代码，开不开下一轮照」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:10`) 与 「打中的东西先挂起、不直接写进正文」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`) 还是合稿原样 |
| 18 | 提交之外跑不指定范围的门禁一律先弹窗，用户要求的也弹一次 | 共用约束 「提交之外要跑先弹窗问用户」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:59`) 已经是这个意思 | 本组不用改；合稿 main-agent 那一条 「提交之外，用户要求的照跑」(`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`) 与它不一致，待落（不算打中） |
| 19 | 调查员「查到就修」照旧在主工作区改，规则里写明是「一律交补丁」的例外 | 调查员 「这时在主工作区改」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:23`)、「在主工作区只改」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:33`)；实现员 「只派你一个也这样，不碰主工作区」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:15`)；两处都没写「例外」 | 待落（不算打中）：调查员定义与实现员定义都得补那一句 |

记录里这三条的原话：「现查坐实就改，再攻一轮」(`records/2026-11-16-三方豁免取消与存量补轮.md:169`)；「一律先弹窗」(`records/2026-11-16-三方豁免取消与存量补轮.md:170`)；「照旧在主工作区改」(`records/2026-11-16-三方豁免取消与存量补轮.md:171`)。

## 指针核对表（格四）

范围：合稿共用约束与 14 份定义里每一处写成 `路径` 的指针（含「开工先读」行里点名的规则小节）。「谁指」写 文件名:行，文件名是合稿树里 `dot-claude/` 下的（共用约束写 agent-common），同一目标合并成一行。类别 R 的 27 个、K 的 15 个、H 的 72 个、D 的 2 个都在被点名读之列；E 的 5 个字面上不被点名。

每个「节标题」指针另核过：我用 `ptrcheck.py` 把这 15 份里「路径 + 「节标题」」写法的每一处，对着合稿树（`.claude/` 下的路径优先取合稿树，没有再取仓里，上游规则与 kb 取仓里）里该文件的标题逐个比，9 处不是整行标题相等：7 处写成裸文件名（`test-script-discipline.md`、`evidence-discipline.md` 不带目录，原件同样这样写，我手核了：小节标题都在合稿树里）；1 处是调查员指到实现员定义的「开工先读：」那一行（指的是那一行，不是标题）；1 处是共用约束 :24 的「旧证据是参考，不是依据」，它是 evidence-discipline 里一条的开头，不是标题，所在小节「原样保存的证据不许事后改」的标题在。其余指针的节标题都精确命中。

| 类 | 目标 | 谁指（文件:行） | 子 agent 怎么被点名 | 判 |
|---|---|---|---|---|
| D | `.claude/agent-common.md` | clerk:13; experiment-designer:13; experiment-runner:13; implementation-writer:13; investigator:13; kb-scribe:13; mutation-triage:13; prior-art:13; sweep:13; three-way-attack:14; three-way-defense:14; three-way-forward:14; three-way-materials:13; tooling-writer:13,15,40 | 是（开工先读：clerk,experiment-designer,experiment-runner,implementation-writer,investigator,kb-scribe,mutation-triage,prior-art,sweep,three-way-attack,three-way-defense,three-way-forward,three-way-materials,tooling-writer） | 读得到 |
| D | `.claude/agents/implementation-writer.md` | agent-common:60,109; investigator:16,32,33 | 是（开工先读：investigator） | 读得到 |
| E | `.claude/main-agent.md` | agent-common:51,64,109; tooling-writer:15,40 | 无（不被点名读；主 agent 一侧的动作） | 字面不读，实质不影响 |
| E | `.claude/skills/crash-test/commit-stages.md` | agent-common:59,108 | 无（不被点名读；主 agent 一侧的动作） | 字面不读，实质不影响 |
| E | `.claude/skills/gate/staged-gate.md` | agent-common:59,108 | 无（不被点名读；主 agent 一侧的动作） | 字面不读，实质不影响 |
| E | `.claude/skills/three-way/SKILL.md` | agent-common:104 | 无（不被点名读；主 agent 一侧的动作） | 字面不读，实质不影响 |
| E | `records/2026-10-29-subagent拆分提案.md` | agent-common:4; tooling-writer:20 | 无（不被点名读；主 agent 一侧的动作） | 字面不读，实质不影响 |
| H | `.claude/gate.d/code-source-discipline.sh` | sweep:35 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/code-tooling.sh` | tooling-writer:32,32 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/comment-history-lag.tsv` | agent-common:41 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/crash-point-replay-items.tsv` | implementation-writer:59,65 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/doc-experiments.sh` | experiment-designer:38 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/path-moves.tsv` | tooling-writer:16 | 是（开工先读：tooling-writer） | 读得到 |
| H | `.claude/gate.d/refused-tests.tsv` | implementation-writer:49,65 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/replay-comparison-units.tsv` | experiment-designer:29; experiment-runner:48,56 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/split-proposal-ledgers.tsv` | agent-common:89 | 是（正文点名） | 读得到 |
| H | `.claude/gate.d/stage-inputs.tsv` | agent-common:61,61; implementation-writer:59,65 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/agent-write-scope.tsv` | agent-common:30; clerk:31; experiment-runner:56; implementation-writer:65; investigator:37; kb-scribe:45; tooling-writer:41 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/ask-user-claim-guard.sh` | agent-common:104 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/bash-command-detector.sh` | agent-common:74 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/completion-cleanup-guard.sh` | agent-common:36,100 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/continuation-guard.sh` | agent-common:104 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/handback-guard.sh` | agent-common:57,103 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/lib_project_code_runs.py` | agent-common:58,95 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/lib_writer_lock.py` | agent-common:34 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/project-code-guard.sh` | agent-common:64,95 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/run-scope-guard.sh` | agent-common:56,61,101; experiment-designer:38; investigator:32 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/runner-dispatch-guard.sh` | agent-common:104; experiment-runner:22 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/selftest-registry.tsv` | agent-common:108 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/selftest-temporary-service.tsv` | agent-common:56 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/timer-only-remind-guard.sh` | agent-common:102 | 是（正文点名） | 读得到 |
| H | `.claude/hooks/write-guard.sh` | agent-common:96 | 是（正文点名） | 读得到 |
| H | `.claude/singlefs-ai-sop/scripts/check.sh` | implementation-writer:50 | 是（正文点名） | 读得到 |
| H | `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh` | agent-common:99 | 是（正文点名） | 读得到 |
| H | `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh` | agent-common:94 | 是（正文点名） | 读得到 |
| H | `.claude/singlefs-ai-sop/scripts/comment-only-change.py` | agent-common:41 | 是（正文点名） | 读得到 |
| H | `crates-mutation-replay.sh` | implementation-writer:44 | 是（正文点名） | 读得到 |
| H | `crates/mutations.tsv` | experiment-runner:34,41,56; implementation-writer:22,26,46,47…; investigator:33,37; mutation-triage:20,21; three-way-materials:22 | 是（正文点名） | 读得到 |
| H | `crates/singlefs-verification/src/crash.rs` | three-way-attack:35 | 是（正文点名） | 读得到 |
| H | `crates/singlefs-verification/src/crash_judge_dispatch.rs` | implementation-writer:69 | 是（正文点名） | 读得到 |
| H | `crates/singlefs-verification/src/crash_point_replay_provenance.rs` | implementation-writer:60 | 是（正文点名） | 读得到 |
| H | `crates/singlefs-verification/src/crash_state_enumeration.rs` | implementation-writer:47; three-way-attack:34 | 是（正文点名） | 读得到 |
| H | `crates/singlefs-verification/src/judging_units_entries.rs` | implementation-writer:60 | 是（正文点名） | 读得到 |
| H | `crates/singlefs-verification/src/memory_pool.rs` | three-way-attack:32 | 是（正文点名） | 读得到 |
| H | `research/prompts/abandoned-rounds.tsv` | agent-common:104 | 是（正文点名） | 读得到 |
| H | `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh` | three-way-attack:47; three-way-defense:32; three-way-forward:32 | 是（正文点名） | 读得到 |
| H | `scripts/agents/agent-watch.py` | agent-common:72 | 是（正文点名） | 读得到 |
| H | `scripts/agents/cite-check.py` | agent-common:103,115 | 是（正文点名） | 读得到 |
| H | `scripts/agents/copy-originals.py` | agent-common:58 | 是（正文点名） | 读得到 |
| H | `scripts/agents/dispatch-prompt.py` | agent-common:104 | 是（正文点名） | 读得到 |
| H | `scripts/agents/three-way-rerun-template.sh` | three-way-attack:46; three-way-defense:37; three-way-forward:36 | 是（正文点名） | 读得到 |
| H | `scripts/crash/crash-amplification.sh` | agent-common:61,64; implementation-writer:60,61,65 | 是（正文点名） | 读得到 |
| H | `scripts/crash/crash-case-check.py` | implementation-writer:58,58,61 | 是（正文点名） | 读得到 |
| H | `scripts/crash/crash-point-replay.sh` | implementation-writer:39,59 | 是（正文点名） | 读得到 |
| H | `scripts/experiment-apparatus/Cargo.toml` | experiment-runner:33,41,56; mutation-triage:20 | 是（正文点名） | 读得到 |
| H | `scripts/experiment-apparatus/perf-by-milestone.md` | experiment-designer:31 | 是（正文点名） | 读得到 |
| H | `scripts/experiment-apparatus/replay.sh` | experiment-runner:44,48,56 | 是（正文点名） | 读得到 |
| H | `scripts/gate/apply-writer-patch.py` | implementation-writer:51,54,56,70 | 是（正文点名） | 读得到 |
| H | `scripts/gate/gate-staged.sh` | agent-common:108 | 是（正文点名） | 读得到 |
| H | `scripts/gate/hook-swap.py` | tooling-writer:31 | 是（正文点名） | 读得到 |
| H | `scripts/gate/merge-and-verify.sh` | agent-common:64 | 是（正文点名） | 读得到 |
| H | `scripts/gate/rerun-failed-tests.py` | agent-common:56 | 是（正文点名） | 读得到 |
| H | `scripts/gate/selftest-process-guard.py` | agent-common:108 | 是（正文点名） | 读得到 |
| H | `scripts/hosts/compile-then-swap.py` | agent-common:83,96; experiment-designer:38 | 是（正文点名） | 读得到 |
| H | `scripts/hosts/lib_repository_copy.py` | agent-common:35 | 是（正文点名） | 读得到 |
| H | `scripts/hosts/minimal-copy.py` | agent-common:58 | 是（正文点名） | 读得到 |
| H | `scripts/hosts/work-directory-ownership.sh` | agent-common:85 | 是（正文点名） | 读得到 |
| H | `scripts/kb/insert-row.py` | agent-common:90 | 是（正文点名） | 读得到 |
| H | `scripts/kb/kb-spec-check.py` | kb-scribe:15,30 | 是（正文点名） | 读得到 |
| H | `scripts/kb/ledger-append-section.py` | agent-common:92 | 是（正文点名） | 读得到 |
| H | `scripts/kb/quote-kb.py` | experiment-designer:51 | 是（正文点名） | 读得到 |
| H | `scripts/kb/relabel-item.py` | agent-common:49 | 是（正文点名） | 读得到 |
| H | `scripts/kb/replace-batch.py` | agent-common:32; kb-scribe:31 | 是（正文点名） | 读得到 |
| H | `scripts/kb/replace-once.py` | agent-common:32,32,81,90; tooling-writer:31 | 是（正文点名） | 读得到 |
| H | `scripts/kb/stale-candidates.py` | sweep:64 | 是（正文点名） | 读得到 |
| H | `scripts/mutation/broad-mutation.py` | tooling-writer:49 | 是（正文点名） | 读得到 |
| H | `scripts/mutation/mutate.sh` | experiment-runner:60 | 是（正文点名） | 读得到 |
| H | `scripts/mutation/prove-red.sh` | agent-common:62 | 是（正文点名） | 读得到 |
| H | `task-server/task_server_lib.py` | agent-common:70; experiment-runner:29 | 是（正文点名） | 读得到 |
| K | `.claude/kb/checks-owed.md` | tooling-writer:20 | 是（正文点名） | 读得到 |
| K | `.claude/kb/decisions-history.md` | agent-common:41; experiment-designer:31; kb-scribe:22,31,32 | 是（正文点名） | 读得到 |
| K | `.claude/kb/decisions.md` | kb-scribe:22,31 | 是（正文点名） | 读得到 |
| K | `.claude/kb/experiments-history.en.md` | experiment-runner:56 | 是（正文点名） | 读得到 |
| K | `.claude/kb/experiments-history.ja.md` | experiment-runner:56 | 是（正文点名） | 读得到 |
| K | `.claude/kb/experiments-history.md` | agent-common:41; experiment-runner:56 | 是（正文点名） | 读得到 |
| K | `.claude/kb/experiments.en.md` | experiment-runner:50,56 | 是（正文点名） | 读得到 |
| K | `.claude/kb/experiments.ja.md` | experiment-runner:50,56 | 是（正文点名） | 读得到 |
| K | `.claude/kb/experiments.md` | experiment-runner:50,56 | 是（正文点名） | 读得到 |
| K | `.claude/kb/prior-art.md` | prior-art:32 | 是（正文点名） | 读得到 |
| K | `.claude/kb/term-renames.md` | agent-common:37 | 是（正文点名） | 读得到 |
| K | `.claude/kb/translation-glossary.md` | experiment-runner:50 | 是（正文点名） | 读得到 |
| K | `.claude/kb/verification-build.md` | implementation-writer:17 | 是（正文点名） | 读得到 |
| K | `.claude/kb/vm-harness.md` | experiment-designer:37; experiment-runner:48,48 | 是（正文点名） | 读得到 |
| K | `.claude/singlefs-ai-sop/skills/decide/SKILL.md` | kb-scribe:16 | 是（开工先读：kb-scribe） | 读得到 |
| R | `.claude/rules/changelog-format.md` | experiment-runner:16; kb-scribe:16,21,30,32 | 是（开工先读：experiment-runner,kb-scribe） | 读得到 |
| R | `.claude/rules/command-safety-by-review.md` | agent-common:67; experiment-runner:16; tooling-writer:16 | 是（开工先读：experiment-runner,tooling-writer） | 读得到 |
| R | `.claude/rules/directory-layout.md` | experiment-runner:16; tooling-writer:16 | 是（开工先读：experiment-runner,tooling-writer） | 读得到 |
| R | `.claude/rules/evidence-discipline.md` | agent-common:20,24,117,118; experiment-designer:16; experiment-runner:16; investigator:16; prior-art:16; sweep:16; three-way-attack:17; three-way-defense:17; three-way-forward:17; three-way-materials:16 | 是（开工先读：experiment-designer,experiment-runner,investigator,prior-art,sweep,three-way-attack,three-way-defense,three-way-forward,three-way-materials） | 读得到 |
| R | `.claude/rules/format-evolution.md` | experiment-runner:16; implementation-writer:16; kb-scribe:16,24; sweep:16 | 是（开工先读：experiment-runner,implementation-writer,kb-scribe,sweep） | 读得到 |
| R | `.claude/rules/fs-design.md` | implementation-writer:16; three-way-materials:27 | 是（开工先读：implementation-writer） | 读得到 |
| R | `.claude/rules/implementation-first.md` | experiment-designer:16; implementation-writer:16; three-way-forward:17 | 是（开工先读：experiment-designer,implementation-writer,three-way-forward） | 读得到 |
| R | `.claude/rules/implementation-workflow.md` | agent-common:4,52; experiment-runner:16,36; implementation-writer:15,16; tooling-writer:16,16 | 是（开工先读：experiment-runner,implementation-writer,tooling-writer） | 读得到 |
| R | `.claude/rules/machine-first.md` | clerk:16; implementation-writer:16; three-way-attack:17; tooling-writer:16 | 是（开工先读：clerk,implementation-writer,three-way-attack,tooling-writer） | 读得到 |
| R | `.claude/rules/mutation-sampling.md` | experiment-designer:16; experiment-runner:16,60; implementation-writer:16; mutation-triage:15,32 | 是（开工先读：experiment-designer,experiment-runner,implementation-writer,mutation-triage） | 读得到 |
| R | `.claude/rules/path-moves.md` | agent-common:22; kb-scribe:16; sweep:34; tooling-writer:16 | 是（开工先读：kb-scribe,tooling-writer） | 读得到 |
| R | `.claude/rules/show-me-test-by-review.md` | experiment-runner:43; implementation-writer:16; tooling-writer:16 | 是（开工先读：implementation-writer,tooling-writer） | 读得到 |
| R | `.claude/rules/test-discipline.md` | experiment-designer:16; experiment-runner:16; implementation-writer:16; investigator:16; mutation-triage:15 | 是（开工先读：experiment-designer,experiment-runner,implementation-writer,investigator,mutation-triage） | 读得到 |
| R | `.claude/rules/test-script-discipline.md` | experiment-designer:16; experiment-runner:16,40; implementation-writer:16; tooling-writer:16 | 是（开工先读：experiment-designer,experiment-runner,implementation-writer,tooling-writer） | 读得到 |
| R | `.claude/rules/three-way-inference.md` | agent-common:33,121; experiment-designer:16,20; three-way-attack:17,30; three-way-defense:17; three-way-forward:17; three-way-materials:16 | 是（开工先读：experiment-designer,three-way-attack,three-way-defense,three-way-forward,three-way-materials） | 读得到 |
| R | `.claude/rules/timer-only-remind.md` | implementation-writer:16; investigator:16; tooling-writer:16 | 是（开工先读：implementation-writer,investigator,tooling-writer） | 读得到 |
| R | `.claude/rules/verification-reference.md` | tooling-writer:16 | 是（开工先读：tooling-writer） | 读得到 |
| R | `.claude/rules/verification.md` | agent-common:28,29,59; experiment-runner:16; implementation-writer:16; investigator:16; tooling-writer:16 | 是（开工先读：experiment-runner,implementation-writer,investigator,tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/code-discipline.md` | experiment-runner:16; implementation-writer:16; tooling-writer:16 | 是（开工先读：experiment-runner,implementation-writer,tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/command-safety.md` | agent-common:20; experiment-runner:16; implementation-writer:16; investigator:16; three-way-attack:17; tooling-writer:16 | 是（开工先读：experiment-runner,implementation-writer,investigator,three-way-attack,tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/design-doc-discipline.md` | clerk:16 | 是（开工先读：clerk） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/kb-discipline.md` | agent-common:42; experiment-runner:16; kb-scribe:16; prior-art:16 | 是（开工先读：experiment-runner,kb-scribe,prior-art） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/preflight-discipline.md` | experiment-runner:16; tooling-writer:16 | 是（开工先读：experiment-runner,tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/rules-discipline.md` | tooling-writer:16 | 是（开工先读：tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/show-me-test.md` | tooling-writer:16 | 是（开工先读：tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/sop-first.md` | tooling-writer:16 | 是（开工先读：tooling-writer） | 读得到 |
| R | `.claude/singlefs-ai-sop/rules/writing-discipline.md` | agent-common:20; kb-scribe:16 | 是（开工先读：kb-scribe） | 读得到 |

## 逐条落点表

列：编号 | 原件 文件:行 与原话头 | 合稿树 文件:行 与落点原话头 | 备注。原话头与落点头都是从那一行里截的（脚本断言过它在那一行里）；「落点」可以在别的文件里（挪走的规则）。共用约束的 175 行是我手工逐条配的；14 份定义的 407 行是脚本按重合度配的、低于 55% 的 88 行我逐条人工核过（备注「人工指定」是我改配的）。备注里「用户定-N」「共用-N」「定义-N」指调度记录的用户定案与 `contradictions.md` 里的条。

### 共用约束 `.claude/agent-common.md`（第 1 到 88 条，共 175 条）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| C001 | 「下每个定义开工前先读这一份；与定义冲突时」(`.claude/agent-common.md:3`) | 「下每个定义开工前先读这一份；与定义冲突时」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:3`) | 一致或同义改写 |
| C002 | 「共用约束与各份定义只写怎么做：步骤、约束」(`.claude/agent-common.md:4`) | 「共用约束与各份定义只写怎么做：步骤、约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:4`) | 一致或同义改写 |
| C003 | 「也不留解释的尾巴」(`.claude/agent-common.md:4`) | 「要用时写成读取指令」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:4`) | 「不留解释的尾巴」半句删，其余在 |
| C004 | 「阶段判这一条（规则：.claude/ru」(`.claude/agent-common.md:4`) | 「阶段判这一句（.claude/rules」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:4`) | 一致或同义改写 |
| C005 | 「简单任务不派发，自己直接改」(`.claude/agent-common.md:8`) | 「自己直接做，不派」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:11`) | 改写进主 agent 派发那一条（:11） |
| C006 | 「派发任务要规划，尽量同类任务一起做」(`.claude/agent-common.md:9`) | 「派之前先规划，同类任务一起派」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:11`) | 一致或同义改写 |
| C007 | 「只由主 agent 点名派发；你手里没有」(`.claude/agent-common.md:10`) | 「只由主 agent 派发。子 agent」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:8`) | 一致或同义改写 |
| C008 | 「轮名、产出文件路径、草稿目录、这一轮的禁」(`.claude/agent-common.md:11`) | 「轮名、产出路径、草稿目录、禁读清单由派发」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:9`) | 一致或同义改写 |
| C009 | 「输入缺一样就不开工，回复只写缺什么」(`.claude/agent-common.md:12`) | 「输入缺一样不开工，回复只写缺什么」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:9`) | 一致或同义改写 |
| C010 | 「的，同一条里连那条命令的原文与它的输出一」(`.claude/agent-common.md:13`) | 「的，同一条里给那条命令的原文与输出」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:10`) | 一致或同义改写 |
| C011 | 「开工先核那条命令真跑成了」(`.claude/agent-common.md:13`) | 「开工先核那条命令真跑成了」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:10`) | 一致或同义改写 |
| C012 | 「严格遵守文档中model和effort的」(`.claude/agent-common.md:14`) | 「严格照定义设的 model 与 effo」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:11`) | 一致或同义改写 |
| C013 | 「所有派发的subagent的effort」(`.claude/agent-common.md:15`) | 「effort 不超过 high，要超过弹」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:11`) | 一致或同义改写 |
| C014 | 「任务连续三次派发失败，主agent自己做」(`.claude/agent-common.md:16`) | 「同一件任务连续三次派发失败，主 agen」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:11`) | 加了「同一件」 |
| C015 | 「下的定义都开了 omitClaudeMd」(`.claude/agent-common.md:20`) | 「下的定义都开了 omitClaudeMd」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:15`) | 一致或同义改写 |
| C016 | 「一律整份读，不受读入大小的限制」(`.claude/agent-common.md:22`) | 「点到就整份读，不受读入大小的限制」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`) | 一致或同义改写 |
| C017 | 「点名的小节是这件活要照做的那几条」(`.claude/agent-common.md:22`) | 「点名的小节是这件活要照做的那几条」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`) | 一致或同义改写 |
| C018 | 「先 grep -n '^#' 文件 找到」(`.claude/agent-common.md:23`) | 「先 grep -n '^#' 文件 找起」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:18`) | 一致或同义改写 |
| C019 | 「点的是整份文件的，先看它的节标题，挑与这」(`.claude/agent-common.md:23`) | 「点的是整份的，先看节标题，只读与这件活有」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:18`) | 一致或同义改写 |
| C020 | 「跑命令照 .claude/singlef」(`.claude/agent-common.md:25`) | 「跑命令照 .claude/singlef」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:20`) | 一致或同义改写 |
| C021 | 「给人看的文字照 .claude/sing」(`.claude/agent-common.md:26`) | 「给人看的文字照 .claude/sing」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:20`) | 一致或同义改写 |
| C022 | 「说外部状态之前现查（.claude/ru」(`.claude/agent-common.md:26`) | 「说外部状态之前现查（.claude/ru」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:20`) | 指针换了文件（verify-before-claiming 并进 evidence-discipline） |
| C023 | 「报告与进度记录里的时间只写日期」(`.claude/agent-common.md:27`) | 「报告与进度记录里的时间只写日期」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:21`) | 一致或同义改写 |
| C024 | 「日期用 TZ=Asia/Tokyo da」(`.claude/agent-common.md:27`) | 「用 TZ=Asia/Tokyo date」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:21`) | 一致或同义改写 |
| C025 | 「候选、臂、方案、判据、提问编号有了变体」(`.claude/agent-common.md:28`) | 「候选、臂、方案、判据、提问编号有了变体」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:22`) | 一致或同义改写 |
| C026 | 「派发提示里没给、定义里也没写的项目事实」(`.claude/agent-common.md:29`) | 「派发提示与定义都没给的项目事实」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:23`) | 一致或同义改写 |
| C027 | 「找不到历史实验的数据、提示或产物，去」(`.claude/agent-common.md:30`) | 「找不到历史实验的数据、提示或产物，去」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`) | 一致或同义改写 |
| C028 | 「git log --all --diff」(`.claude/agent-common.md:33`) | 「git log --all --diff」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`) | 一致或同义改写 |
| C029 | 「读到的是当时的数，不是结论」(`.claude/agent-common.md:34`) | 「读到的是当时的数、不是结论」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`) | 一致或同义改写 |
| C030 | 「拿它支撑新结论之前先重新跑一遍」(`.claude/agent-common.md:35`) | 「旧证据是参考，不是依据」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:24`) | 只剩两跳指针：共用约束 :24 指到 evidence-discipline :61，:61 再指到上游 kb-discipline :39（该句原话在那里），见格一注 |
| C031 | 「要推翻早先的结论，按 .claude/r」(`.claude/agent-common.md:36`) | 「要推翻早先的结论，照 .claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:61`) | 落在 evidence-discipline.md |
| C032 | 「不新建判定库（核心策略，没有例外）」(`.claude/agent-common.md:40`) | 「不新建判定库（核心策略，没有例外）」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`) | 一致或同义改写 |
| C033 | 「判定一律经 prune-server 的」(`.claude/agent-common.md:40`) | 「判定一律经 prune-server 客」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`) | 一致或同义改写 |
| C034 | 「并回即删的中转库与对临时服务起的自证库不」(`.claude/agent-common.md:40`) | 「并回即删的中转库与对临时服务起的自证库不」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`) | 一致或同义改写 |
| C035 | 「改动里出现新的判定存放处，主 agent」(`.claude/agent-common.md:40`) | 「改动里出现新的判定存放处，主 agent」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`) | 一致或同义改写 |
| C036 | 「所有任务都要多机可用，不许绑定某一台」(`.claude/agent-common.md:41`) | 「所有任务都要多机可用，不许绑定某一台」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`) | 一致或同义改写 |
| C037 | 「派到某一台就红的是缺陷，当场修，不往统一」(`.claude/agent-common.md:41`) | 「派到某一台就红的是缺陷，当场修，不往任务」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`) | 一致或同义改写 |
| C038 | 「不读会改变判定结局的本机独有状态」(`.claude/agent-common.md:41`) | 「不读会改变判定结局的本机独有状态」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`) | 一致或同义改写 |
| C039 | 「下的私有名单与密钥）、不写死主机名与地址」(`.claude/agent-common.md:41`) | 「许读的本机私有状态只有 ~/.confi」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:62`) | 许读那一括号、multi-host.env、没写就是没有 三处落在 verification.md「所有任务都要多机可用」 |
| C040 | 「一条测试自己的配置文件进仓或放共享根」(`.claude/agent-common.md:41`) | 「一条测试自己的配置文件（qemu.env」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:63`) | 落在 verification.md |
| C041 | 「不经 ~ 与 $HOME 取」(`.claude/agent-common.md:41`) | 「不经 ~ 与 $HOME 取」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`) | 一致或同义改写 |
| C042 | 「只写派发提示与定义」(`.claude/agent-common.md:43`) | 「只写派发提示与定义」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:30`) | 一致或同义改写 |
| C043 | 「被拒之后不换 Bash 或别的办法写过去」(`.claude/agent-common.md:43`) | 「被拒之后不换 Bash 或别的办法写过去」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:30`) | 一致或同义改写 |
| C044 | 「没有项目定义的 agent（内置类型、定」(`.claude/agent-common.md:43`) | 「没有项目定义的 agent（内置类型、定」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:30`) | 一致或同义改写 |
| C045 | 「不在任何子 agent 的写范围里」(`.claude/agent-common.md:43`) | 「不在任何子 agent 的写范围里」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:30`) | 一致或同义改写 |
| C046 | 「备份原文件用 python3 scrip」(`.claude/agent-common.md:43`) | 「备份原文件用 python3 scrip」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:31`) | 一致或同义改写 |
| C047 | 「按文件写多份输出写成 for 名字 in」(`.claude/agent-common.md:43`) | 「按文件写多份输出写成 for 名字 in」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:31`) | 一致或同义改写 |
| C048 | 「命令词由变量、命令替换拼出的」(`.claude/agent-common.md:43`) | 「命令词不由变量、命令替换拼」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:31`) | 一致或同义改写 |
| C049 | 「钩子判不到的写法」(`.claude/agent-common.md:43`) | 「钩子判不到的写法」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:31`) | 一致或同义改写 |
| C050 | 「手写的改动一律用 Edit（旧串必须在文」(`.claude/agent-common.md:44`) | 「手写改动：tools 里有 Edit 的」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:32`) | 一致或同义改写 |
| C051 | 「新建文件用 Write（先 ls 确认不」(`.claude/agent-common.md:44`) | 「新建文件，有 Write 的用 Writ」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:32`) | 一致或同义改写 |
| C052 | 「set -o noclobber 后 c」(`.claude/agent-common.md:45`) | 「set -o noclobber 后 c」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:32`) | 一致或同义改写 |
| C053 | 「报告分段写，每一次写进文件的内容不超过」(`.claude/agent-common.md:46`) | 「报告分段写，每一次写进文件的内容不超过」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:33`) | 一致或同义改写 |
| C054 | 「改已有文件只用定点替换」(`.claude/agent-common.md:47`) | 「改已有文件只用 scripts/kb/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:32`) | 一致或同义改写 |
| C055 | 「草稿放派发提示给的草稿目录」(`.claude/agent-common.md:48`) | 「草稿与报告写派发提示给的草稿目录」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:30`) | 一致或同义改写 |
| C056 | 「开工先认锁：派发提示里」(`.claude/agent-common.md:49`) | 「开工先认锁（tools 里有 Write」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:34`) | 一致或同义改写 |
| C057 | 「你往这个目录的第一次 Write / E」(`.claude/agent-common.md:49`) | 「你往里的第一次 Write / Edit」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:34`) | 认法细节（派发回执对锁、先到先得）下沉到 lib_writer_lock.py 文件头（用户定-7） |
| C058 | 「子目录另派给别的子 agent 之后，父」(`.claude/agent-common.md:49`) | 「子目录另派给别的子 agent 之后，父」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:34`) | 一致或同义改写 |
| C059 | 「建仓副本或编译目录之前，先 du -sh」(`.claude/agent-common.md:50`) | 「建仓副本或编译目录之前，先 du -sh」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:35`) | 一致或同义改写 |
| C060 | 「python3 scripts/host」(`.claude/agent-common.md:50`) | 「python3 scripts/host」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:35`) | 一致或同义改写 |
| C061 | 「要门禁表的加 --with-gate-t」(`.claude/agent-common.md:50`) | 「要门禁表的加 --with-gate-t」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:35`) | 一致或同义改写 |
| C062 | 「不自己 rsync、cp 拷 crate」(`.claude/agent-common.md:50`) | 「不自己 rsync、cp 拷 crate」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:35`) | 一致或同义改写 |
| C063 | 「副本、编译目录与运行目录里的中转库」(`.claude/agent-common.md:51`) | 「副本、编译目录与运行目录里的中转库」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:36`) | 一致或同义改写 |
| C064 | 「删之前 du -sh 记大小写进报告」(`.claude/agent-common.md:51`) | 「删之前 du -sh 记大小写进报告」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:36`) | 一致或同义改写 |
| C065 | 「这一行只在删不掉时用，删得掉的照删、不拿」(`.claude/agent-common.md:51`) | 「删得掉的照删，不拿这一行代替」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:36`) | 一致或同义改写 |
| C066 | 「要交给主 agent 接着用的副本」(`.claude/agent-common.md:51`) | 「要交给主 agent 接着用的副本」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:36`) | 一致或同义改写 |
| C067 | 「名字、注释、文档、输出文字都不用序数或位」(`.claude/agent-common.md:52`) | 「名字、注释、文档、输出文字都不用序数或位」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:37`) | 一致或同义改写 |
| C068 | 「在一份文件里新加函数、类、方法或 she」(`.claude/agent-common.md:53`) | 「在一份文件里新加函数、类、方法或 she」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:38`) | 一致或同义改写 |
| C069 | 「进仓的文字不写会话名与批次代号」(`.claude/agent-common.md:54`) | 「进仓的文字不写会话全名」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:39`) | 一致或同义改写 |
| C070 | 「里程碑 17 收尾之前不兼容任何旧东西」(`.claude/agent-common.md:55`) | 「里程碑 17 收尾之前不兼容任何旧东西」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:40`) | 一致或同义改写 |
| C071 | 「代码与脚本的注释、说明只写现状，不写历史」(`.claude/agent-common.md:56`) | 「代码与脚本的注释、说明只写现状，不写历史」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:41`) | 一致或同义改写 |
| C072 | 「那一行的注释里写 history-kep」(`.claude/agent-common.md:56`) | 「那一行的注释写 history-kept」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:41`) | 一致或同义改写 |
| C073 | 「写进仓里的注释与 .md（kb 之外的也」(`.claude/agent-common.md:57`) | 「写进仓里的注释与 .md 引 D、E 编」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:42`) | 一致或同义改写 |
| C074 | 「私有主机信息不进仓」(`.claude/agent-common.md:58`) | 「私有主机信息不进仓」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:43`) | 一致或同义改写 |
| C075 | 「进仓的文字只写本项目的东西」(`.claude/agent-common.md:59`) | 「进仓的文字只写本项目的东西」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:44`) | 一致或同义改写 |
| C076 | 「不做 git 写操作」(`.claude/agent-common.md:63`) | 「不做 git 写操作」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:48`) | 一致或同义改写 |
| C077 | 「不建、不改 .claude/agents」(`.claude/agent-common.md:64`) | 「不建、不改 .claude/agents」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:49`) | 一致或同义改写 |
| C078 | 「tooling-writer 例外：按它」(`.claude/agent-common.md:64`) | 「例外照各自定义」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:49`) | 加了 kb-scribe 的例外（共用-1） |
| C079 | 「不读派发提示给的禁读清单里的文件」(`.claude/agent-common.md:65`) | 「不读派发提示给的禁读清单里的文件」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:50`) | 一致或同义改写 |
| C080 | 「编译、测试、门禁、实验、自证这类跑项目代」(`.claude/agent-common.md:66`) | 「跑项目代码的命令（编译、测试、门禁、实验」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:55`) | 一致或同义改写 |
| C081 | 「子 agent 不往服务交任务」(`.claude/agent-common.md:66`) | 「子 agent 不起 task-serv」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:55`) | 一致或同义改写 |
| C082 | 「测试、带 --item 的门禁与崩溃放量」(`.claude/agent-common.md:66`) | 「测试、带 --item 的门禁与崩溃放量」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:55`) | 一致或同义改写 |
| C083 | 「子 agent 只读 task-serv」(`.claude/agent-common.md:66`) | 「子 agent 只读 task-serv」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:55`) | 一致或同义改写 |
| C084 | 「范围声明写在 submit 前面：SIN」(`.claude/agent-common.md:66`) | 「范围声明写在 submit 前面（SIN」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 一致或同义改写 |
| C085 | 「交给服务的命令不带包装前缀（capped」(`.claude/agent-common.md:66`) | 「不带包装前缀（capped.sh」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 一致或同义改写 |
| C086 | 「线程固定是登记的每台默认重活线程数」(`.claude/agent-common.md:66`) | 「线程固定是 .claude/heavy-」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 一致或同义改写 |
| C087 | 「scripts/gate/rerun-f」(`.claude/agent-common.md:66`) | 「scripts/gate/rerun-f」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 一致或同义改写 |
| C088 | 「登在 .claude/hooks/sel」(`.claude/agent-common.md:66`) | 「.claude/hooks/selfte」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 整条命令写出，与表文件头 required 一条对得上（量的） |
### 共用约束 `.claude/agent-common.md`（第 89 到 175 条，共 175 条）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| C089 | 「提交时当场拒（task submit 退」(`.claude/agent-common.md:66`) | 「task submit 当场退 3」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`) | 一致或同义改写 |
| C090 | 「那一节每一行最多一条交任务的命令」(`.claude/agent-common.md:66`) | 「每一行最多一条交任务的命令，不用分号」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 一致或同义改写 |
| C091 | 「子 agent 直接跑、不经服务的只有：」(`.claude/agent-common.md:66`) | 「子 agent 直接跑、不经服务的只有：」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:58`) | 一致或同义改写 |
| C092 | 「名单是 .claude/hooks/li」(`.claude/agent-common.md:66`) | 「名单以 .claude/hooks/li」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:58`) | 逐个文本工具名在共用约束里删了，各定义点名自己用的 |
| C093 | 「报告另写做完的、没做的、等这些命令的结果」(`.claude/agent-common.md:67`) | 「报告另写做完的、没做的、等哪几条结果才能」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 一致或同义改写 |
| C094 | 「新派一个同类 agent，派发提示写一行」(`.claude/agent-common.md:67`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 一致或同义改写 |
| C095 | 「不指定范围的在提交时由主 agent 跑」(`.claude/agent-common.md:68`) | 「只在提交时由主 agent 跑」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:59`) | 一致或同义改写 |
| C096 | 「提交之外要跑先弹窗问用户」(`.claude/agent-common.md:68`) | 「提交之外要跑先弹窗问用户」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:59`) | 一致或同义改写 |
| C097 | 「带 --item 的四条照 .claud」(`.claude/agent-common.md:68`) | 「带 --item 的四条（登记的崩溃枚举」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:60`) | 四条命令原样落在 implementation-writer.md 第 1d 步 |
| C098 | 「目标里没登记成崩溃枚举用例、也不是放量分」(`.claude/agent-common.md:68`) | 「没登记成崩溃枚举用例、也不是放量分项的测」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:60`) | 一致或同义改写 |
| C099 | 「范围声明闸要两样之一、都没有的执行前拒：」(`.claude/agent-common.md:68`) | 「缩域写成 task-server/tas」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:62`) | 一致或同义改写 |
| C100 | 「缩域只当写段的快速反馈」(`.claude/agent-common.md:68`) | 「缩域只当写段的快速反馈」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:62`) | 一致或同义改写 |
| C101 | 「确要全域的，范围声明的为什么一段末尾写」(`.claude/agent-common.md:68`) | 「全域：范围声明的为什么一段末尾写」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:63`) | 一致或同义改写 |
| C102 | 「不编不跑、退 79」(`.claude/agent-common.md:68`) | 「不编不跑、退 79」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:64`) | 一致或同义改写 |
| C103 | 「clippy / build 逐条的判法」(`.claude/agent-common.md:68`) | 「clippy / build 逐条的判法」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:64`) | 一致或同义改写 |
| C104 | 「被 .claude/hooks/proj」(`.claude/agent-common.md:68`) | 「被项目代码闸拒了不换写法绕过去」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:64`) | 一致或同义改写 |
| C105 | 「日常开发的测试不跑整个包」(`.claude/agent-common.md:69`) | 「日常开发的测试不跑整个包」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`) | 一致或同义改写 |
| C106 | 「三遍写成一份文件」(`.claude/agent-common.md:69`) | 「写成一份文件（工作根」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`) | 一致或同义改写 |
| C107 | 「--whole-package-conf」(`.claude/agent-common.md:69`) | 「--whole-package-conf」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`) | 一致或同义改写 |
| C108 | 「累计跑满两小时服务不停、推提醒给提交它的」(`.claude/agent-common.md:69`) | 「累计满两小时服务不停、推提醒给提交它的会」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`) | 一致或同义改写 |
| C109 | 「记结局 over-four-hours」(`.claude/agent-common.md:69`) | 「记结局 over-four-hours」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:65`) | 一致或同义改写 |
| C110 | 「草稿脚本里起自证、门禁阶段、入口脚本或」(`.claude/agent-common.md:70`) | 「草稿脚本里起自证、门禁阶段、入口脚本或」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 一致或同义改写 |
| C111 | 「写进脚本文件、放进 python3 的」(`.claude/agent-common.md:70`) | 「写进脚本、python 的 subpro」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:55`) | 一致或同义改写 |
| C112 | 「交主 agent 跑的验证链里不夹不指定」(`.claude/agent-common.md:70`) | 「交主 agent 跑的验证链里不夹不指定」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 一致或同义改写 |
| C113 | 「要停经服务交的任务，主 agent 用」(`.claude/agent-common.md:71`) | 「要停经服务交的任务，主 agent 用」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:66`) | 一致或同义改写 |
| C114 | 「子 agent 不 cancel、不 c」(`.claude/agent-common.md:71`) | 「子 agent 不 cancel、不 c」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:66`) | 一致或同义改写 |
| C115 | 「装置与判法里起样本进程（真 shell、」(`.claude/agent-common.md:72`) | 「装置、判法、自证与复跑脚本里起样本进程（」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`) | 用户定-6：整组信号改为逐个按进程号停 |
| C116 | 「命令行里停进程照旧一次点名一个」(`.claude/agent-common.md:72`) | 「命令行里停进程照旧一次点名一个」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`) | 一致或同义改写 |
| C117 | 「读文件：1,000 行以内的整读一次」(`.claude/agent-common.md:73`) | 「别的文件：1,000 行以内的整读一次」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:19`) | 与上一条射程分开（矛盾表共用-2），见格二 |
| C118 | 「同一份文件读过一次、之后没改过，就不再重」(`.claude/agent-common.md:73`) | 「读过、之后没改过的不重读」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:19`) | 一致或同义改写 |
| C119 | 「几条只读命令（ls、grep、wc、sh」(`.claude/agent-common.md:73`) | 「几条只读命令（ls、grep、wc、sh」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 一致或同义改写 |
| C120 | 「长命令的输出落进草稿目录的文件，只读汇总」(`.claude/agent-common.md:73`) | 「长命令的输出落进草稿目录的文件、只读汇总」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 一致或同义改写 |
| C121 | 「上下文多大都不是交回的理由：活没做完就接」(`.claude/agent-common.md:73`) | 「上下文多大都不是交回的理由，活没做完就接」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 加 experiment-runner 例外（用户定-5） |
| C122 | 「崩溃点测试准确率和正确率优先」(`.claude/agent-common.md:74`) | 「崩溃点测试准确率和正确率优先」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:54`) | 一致或同义改写 |
| C123 | 「禁止自行扩大任务范围，只改自己的部分」(`.claude/agent-common.md:75`) | 「禁止自行扩大任务范围，只改自己的部分」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:51`) | 一致或同义改写 |
| C124 | 「如果验证为红给其他任务发消息」(`.claude/agent-common.md:75`) | 「不属于自己任务的验证（门禁检测这类）红了」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:51`) | 共用-3：子 agent 不发消息（矛盾表登记） |
| C125 | 「遇到同时需要修改的内容协商处理」(`.claude/agent-common.md:76`) | 「要和别的会话协商的，子 agent 写进」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:52`) | 共用-4：同上 |
| C126 | 「按内容哈希登记的豁免表」(`.claude/agent-common.md:76`) | 「按内容哈希登记的豁免表」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:52`) | 一致或同义改写 |
| C127 | 「表里现有的行补完一轮三方删一行，只缩不涨」(`.claude/agent-common.md:76`) | 「豁免表 .claude/crates-r」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:10`) | 落在 implementation-workflow.md 第 2 步那一格；共用约束只留「不再加行」+指针 |
| C128 | 「一份内容不在工作区」(`.claude/agent-common.md:77`) | 「一份内容不在工作区」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:53`) | 一致或同义改写 |
| C129 | 「子 agent 前台起的命令（只读命令、」(`.claude/agent-common.md:78`) | 「前台命令的 timeout 不超过 24」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:69`) | 一致或同义改写 |
| C130 | 「每完成一格往草稿目录的 progress」(`.claude/agent-common.md:81`) | 「每完成一格往草稿目录的 progress」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:69`) | 一致或同义改写 |
| C131 | 「子 agent 读 task-serve」(`.claude/agent-common.md:81`) | 「子 agent 读 task-serve」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:69`) | 一致或同义改写 |
| C132 | 「task wait 退 78（连不上服务」(`.claude/agent-common.md:81`) | 「task wait 退 78（连不上服务」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:70`) | 一致或同义改写 |
| C133 | 「等日志里的一行字之前，先确认那一行真会写」(`.claude/agent-common.md:81`) | 「等日志里的一行字之前，先确认那一行真会写」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:72`) | 一致或同义改写 |
| C134 | 「主 agent 的看门狗（scripts」(`.claude/agent-common.md:81`) | 「看门狗（scripts/agents/a」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:72`) | 一致或同义改写 |
| C135 | 「项目 settings 里注册的这几道」(`.claude/agent-common.md:82`) | 「项目 settings 注册的钩子在执行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:73`) | 一致或同义改写 |
| C136 | 「被钩子或服务拒了（task-server」(`.claude/agent-common.md:82`) | 「被钩子或服务（task-server/t」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:73`) | 一致或同义改写 |
| C137 | 「① 起看门狗的错误写法」(`.claude/agent-common.md:84`) | 「① 起看门狗（只有主 agent 起）」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:75`) | 一致或同义改写 |
| C138 | 「② 前台没有超时的等待循环」(`.claude/agent-common.md:85`) | 「② 前台 until / while 里」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:76`) | 一致或同义改写 |
| C139 | 「③ 把活放出追踪：disown、copr」(`.claude/agent-common.md:86`) | 「③ 把活放出追踪：disown、copr」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:77`) | 一致或同义改写 |
| C140 | 「④ run_in_background」(`.claude/agent-common.md:87`) | 「④ run_in_background」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:78`) | 一致或同义改写 |
| C141 | 「⑤ 整份覆盖 research/resu」(`.claude/agent-common.md:89`) | 「⑤ 整份覆盖 research/resu」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:79`) | 一致或同义改写 |
| C142 | 「⑥ 终止进程不是点名一个自己起的进程号或」(`.claude/agent-common.md:90`) | 「⑥ 终止进程不是点名一个自己起的进程号或」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:80`) | 一致或同义改写 |
| C143 | 「⑧ 有 Edit 工具的项目子 agen」(`.claude/agent-common.md:91`) | 「⑧ 有 Edit 工具的项目子 agen」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:82`) | 一致或同义改写 |
| C144 | 「⑦ 在同一个 inode 上改已经存在的」(`.claude/agent-common.md:92`) | 「⑦ 在同一个 inode 上改已存在的脚」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:81`) | 一致或同义改写 |
| C145 | 「⑩ 子 agent 用 run_in_b」(`.claude/agent-common.md:93`) | 「⑩ 子 agent 用 run_in_b」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:84`) | 一致或同义改写 |
| C146 | 「⑫ 子 agent 在 Bash 里写出」(`.claude/agent-common.md:94`) | 「⑫ 子 agent 在 Bash 里写本」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:86`) | 一致或同义改写 |
| C147 | 「⑬ 项目子 agent 在 Bash 里」(`.claude/agent-common.md:95`) | 「⑬ 项目子 agent 在 Bash 里」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:87`) | 一致或同义改写 |
| C148 | 「⑮ 暂存区非空时不带具体路径清整个索引」(`.claude/agent-common.md:96`) | 「⑮ 暂存区非空时不带具体路径清整个索引」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:88`) | 一致或同义改写 |
| C149 | 「⑲ 往 records/ 下的 .md」(`.claude/agent-common.md:97`) | 「⑲ 往 records/ 下的 .md」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:92`) | 一致或同义改写 |
| C150 | 「⑳ 手拷仓副本」(`.claude/agent-common.md:98`) | 「⑳ 手拷仓副本」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:93`) | 一致或同义改写 |
| C151 | 「上游的 .claude/singlefs」(`.claude/agent-common.md:99`) | 「上游 .claude/singlefs-」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:94`) | 一致或同义改写 |
| C152 | 「项目代码闸（.claude/hooks/」(`.claude/agent-common.md:100`) | 「项目代码闸（.claude/hooks/」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:95`) | 一致或同义改写 |
| C153 | 「认不出起什么的写法照拒」(`.claude/agent-common.md:100`) | 「认不出起什么的写法（命令词」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:95`) | 识别细节下沉到钩子文件头，留合规写法（用户定-7） |
| C154 | 「写闸（.claude/hooks/wri」(`.claude/agent-common.md:101`) | 「写闸（.claude/hooks/wri」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:96`) | 一致或同义改写 |
| C155 | 「交回闸（上游的 .claude/sing」(`.claude/agent-common.md:102`) | 「交回闸（上游 .claude/singl」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:99`) | 一致或同义改写 |
| C156 | 「完工清理闸（.claude/hooks/」(`.claude/agent-common.md:103`) | 「完工清理闸（.claude/hooks/」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:100`) | 一致或同义改写 |
| C157 | 「弹窗闸（.claude/hooks/as」(`.claude/agent-common.md:104`) | 「弹窗闸（.claude/hooks/as」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:104`) | 一致或同义改写 |
| C158 | 「派发闸（.claude/hooks/ru」(`.claude/agent-common.md:105`) | 「派发闸（.claude/hooks/ru」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:104`) | 一致或同义改写 |
| C159 | 「范围声明闸（.claude/hooks/」(`.claude/agent-common.md:106`) | 「范围声明闸（.claude/hooks/」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:101`) | 一致或同义改写 |
| C160 | 「定时闸（.claude/hooks/ti」(`.claude/agent-common.md:107`) | 「定时闸（.claude/hooks/ti」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:102`) | 一致或同义改写 |
| C161 | 「续做闸（.claude/hooks/co」(`.claude/agent-common.md:108`) | 「续做闸（.claude/hooks/co」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:104`) | 一致或同义改写 |
| C162 | 「交回正文闸（.claude/hooks/」(`.claude/agent-common.md:109`) | 「交回正文闸（.claude/hooks/」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:103`) | 一致或同义改写 |
| C163 | 「扫仓的门禁阶段（本地与共享）与提交前的整」(`.claude/agent-common.md:113`) | 「扫仓的门禁阶段（本地与共享）与提交前的整」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:108`) | 一致或同义改写 |
| C164 | 「命名检查交主 agent 跑：改了代码或」(`.claude/agent-common.md:114`) | 「命名检查交主 agent 跑：改了」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:109`) | 一致或同义改写 |
| C165 | 「退出码 77 是」(`.claude/agent-common.md:115`) | 「退出码 77 是」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:110`) | 一致或同义改写 |
| C166 | 「交回与报告在不影响正确性的前提下写简练」(`.claude/agent-common.md:119`) | 「写结论、依据与没做什么，不写做事的经过」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:114`) | 丢了「在不影响正确性的前提下」前提（见格一注） |
| C167 | 「引规则与 kb 的条款优先写」(`.claude/agent-common.md:120`) | 「引规则与 kb 的条款优先写」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:115`) | 一致或同义改写 |
| C168 | 「能用命令核的事实，贴命令与原样输出」(`.claude/agent-common.md:121`) | 「能用命令核的事实，贴命令与原样输出」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:116`) | 一致或同义改写 |
| C169 | 「报告里转述的数每个都标」(`.claude/agent-common.md:122`) | 「转述的数各标」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:117`) | 一致或同义改写 |
| C170 | 「把话缩到真核过的那个范围」(`.claude/agent-common.md:123`) | 「把话缩到真核过的那个范围」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:118`) | 一致或同义改写 |
| C171 | 「跑不动的、没验的、按定义不归你的，照实列」(`.claude/agent-common.md:125`) | 「跑不动的、没验的、按定义不归你的，照实列」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:119`) | 一致或同义改写 |
| C172 | 「跑出来的产物先落盘进仓里该在的位置」(`.claude/agent-common.md:126`) | 「跑出来的产物先落盘进仓里该在的位置」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:120`) | 一致或同义改写 |
| C173 | 「定义点名的产出文件（跑前登记、腿的 ou」(`.claude/agent-common.md:127`) | 「定义点名的产出文件（跑前登记、腿的 ou」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 一致或同义改写 |
| C174 | 「交回只调一次（第二次会被拒）」(`.claude/agent-common.md:127`) | 「交回只调一次（第二次被拒）」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 一致或同义改写 |
| C175 | 「干到一半收到主 agent 的消息」(`.claude/agent-common.md:128`) | 「干到一半收到主 agent 的消息」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:122`) | 一致或同义改写 |

### 定义 `.claude/agents/implementation-writer.md`（64 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| iw-01 | 「派发提示写」(`.claude/agents/implementation-writer.md:15`) | 「一律在草稿目录的副本里改、交」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:15`) | 改写 62%；用户定-1：一律副本、交补丁 |
| iw-02 | 「主 agent 同时派几个实」(`.claude/agents/implementation-writer.md:15`) | 「一律在草稿目录的副本里改、交补丁目录」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:15`) | 人工指定；用户定-1：不再分「同时派几个」；用户定-1：一律副本、交补丁 |
| iw-03 | 「派发提示给了段树或快照路径的」(`.claude/agents/implementation-writer.md:15`) | 「副本照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:15`) | 改写 20%（低分行逐条人工核过）；用户定-1：一律副本、交补丁 |
| iw-04 | 「开工先读：.claude/r」(`.claude/agents/implementation-writer.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:16`) | 改写 41%（低分行逐条人工核过） |
| iw-05 | 「改崩溃点重放（崩溃枚举用例、」(`.claude/agents/implementation-writer.md:17`) | 「改崩溃点重放（崩溃枚举用例、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:17`) | 原话基本在 79% |
| iw-06 | 「里程碑文件与步号（或并行线编」(`.claude/agents/implementation-writer.md:21`) | 「里程碑文件与步号（或并行线编」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:21`) | 原话基本在 100% |
| iw-07 | 「或者只修 crates/mu」(`.claude/agents/implementation-writer.md:22`) | 「或者只修 crates/mu」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:22`) | 原话基本在 100% |
| iw-08 | 「压着的条款：kb 文件路径与」(`.claude/agents/implementation-writer.md:23`) | 「压着的条款：kb 文件路径与」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:23`) | 原话基本在 100% |
| iw-09 | 「主 agent 读过的 cr」(`.claude/agents/implementation-writer.md:24`) | 「主 agent 读过的 cr」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:24`) | 原话基本在 100% |
| iw-10 | 「这一轮别的会话正在改的 cr」(`.claude/agents/implementation-writer.md:25`) | 「这一轮别的会话正在改的 cr」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:25`) | 原话基本在 100% |
| iw-11 | 「单独一行」(`.claude/agents/implementation-writer.md:26`) | 「单独一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:26`) | 原话基本在 100% |
| iw-12 | 「报告路径与草稿目录，都在 ~」(`.claude/agents/implementation-writer.md:27`) | 「报告路径与草稿目录，都在 ~」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:27`) | 改写 56% |
| iw-13 | 「编译、测试、证红、命名检查与」(`.claude/agents/implementation-writer.md:31`) | 「编译、测试、证红、命名检查与」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:31`) | 改写 39%（低分行逐条人工核过） |
| iw-14 | 「主 agent 交、等到结局」(`.claude/agents/implementation-writer.md:31`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| iw-15 | 「不指定范围的照第 4 步办：」(`.claude/agents/implementation-writer.md:31`) | 「不指定范围的照第 4 步办：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:31`) | 原话基本在 100% |
| iw-16 | 「第 3 步的证红、第 4 步」(`.claude/agents/implementation-writer.md:32`) | 「第 3 步的证红、第 4 步」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:32`) | 改写 40%（低分行逐条人工核过） |
| iw-17 | 「承接时动到的测试二进制跑红了」(`.claude/agents/implementation-writer.md:33`) | 「承接时动到的测试二进制跑红了」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:33`) | 原话基本在 90% |
| iw-18 | 「不整份重跑（.claude/」(`.claude/agents/implementation-writer.md:33`) | 「.claude/rules/」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:16`) | 原话基本在 77% |
| iw-19 | 「登记的崩溃枚举用例、崩溃放量」(`.claude/agents/implementation-writer.md:34`) | 「范围一段的项数是 1 格加点」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:36`) | 改写 52%（低分行逐条人工核过） |
| iw-20 | 「范围一段的项数是 1 格加点」(`.claude/agents/implementation-writer.md:34`) | 「变异：SINGLEFS_RU」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:38`) | 改写 69% |
| iw-21 | 「命令原样可抄，<根> 填你改」(`.claude/agents/implementation-writer.md:34`) | 「项名拿不准的，把各自的 --」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:42`) | 改写 44%（低分行逐条人工核过） |
| iw-22 | 「改到整趟才跑的那几类的测试目」(`.claude/agents/implementation-writer.md:34`) | 「改到整趟才跑的那几类的测试目」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:39`) | 原话基本在 94% |
| iw-23 | 「改到挂特性的测试文件的（te」(`.claude/agents/implementation-writer.md:34`) | 「文件名照 bash .cla」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:40`) | 原话基本在 85% |
| iw-24 | 「崩溃放量那一条点名的分项没缩」(`.claude/agents/implementation-writer.md:34`) | 「崩溃放量那一条点名的分项没缩」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:42`) | 改写 60% |
| iw-25 | 「新写的变异行，证红的命令报告」(`.claude/agents/implementation-writer.md:34`) | 「新写的变异行，证红命令两样各」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:44`) | 原话基本在 78% |
| iw-26 | 「变异门禁这两条点到的行，第五」(`.claude/agents/implementation-writer.md:34`) | 「点到会走崩溃枚举的目标（ca」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:61`) | 原话基本在 88% |
| iw-27 | 「会不会走以范围声明闸的判定为」(`.claude/agents/implementation-writer.md:34`) | 「范围声明闸要缩域与全域理由之一」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:44`) | 人工指定；判法移到共用约束 :61-63 |
| iw-28 | 「缩域证红只走第 3a 步 s」(`.claude/agents/implementation-writer.md:34`) | 「新写的变异行，证红命令两样各」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:44`) | 改写 64% |
| iw-29 | 「<缩域变量>（第 3a 步用」(`.claude/agents/implementation-writer.md:34`) | 「（第 3a 步用）挑哪个、会不会走对着哪」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:44`) | 人工指定；挑法移到共用约束 :62 |
| iw-30 | 「写段的命令对着自己的副本判（」(`.claude/agents/implementation-writer.md:34`) | 「闸从命令的当前目录往上找带 」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:61`) | 改写 40%（低分行逐条人工核过） |
| iw-31 | 「缩域只当写段的快速反馈，判出」(`.claude/agents/implementation-writer.md:34`) | 「缩域证出的红只当写段的快速反」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:47`) | 改写 31%（低分行逐条人工核过） |
| iw-32 | 「读条款与代码，写实现与测试。」(`.claude/agents/implementation-writer.md:35`) | 「读条款与代码，写实现与测试。」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:45`) | 改写 43%（低分行逐条人工核过） |
| iw-33 | 「每条新测试都要证明会红，证红」(`.claude/agents/implementation-writer.md:36`) | 「每条新测试都要证明会红，证红」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:46`) | 原话基本在 85% |
| iw-34 | 「证红命令一律写成 task-」(`.claude/agents/implementation-writer.md:37`) | 「证红命令一律写成 task-」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:47`) | 原话基本在 91% |
| iw-35 | 「新写崩溃点重放、形式化验证或」(`.claude/agents/implementation-writer.md:38`) | 「新写崩溃点重放、形式化验证或」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:48`) | 改写 57% |
| iw-36 | 「新写的测试默认要算得出单元键」(`.claude/agents/implementation-writer.md:39`) | 「新写的测试默认要算得出单元键」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:49`) | 改写 46%（低分行逐条人工核过） |
| iw-37 | 「变异证红按测试算、不按行算：」(`.claude/agents/implementation-writer.md:40`) | 「变异证红按测试算、不按行算：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 原话基本在 100% |
| iw-38 | 「登记的崩溃枚举用例与崩溃放量」(`.claude/agents/implementation-writer.md:40`) | 「登记的崩溃枚举用例与崩溃放量分项不写成」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 人工指定；「（.claude/rules/verification.md「一个门禁：两种跑法」）」括注删 |
| iw-39 | 「不指定范围的由提交时的主 a」(`.claude/agents/implementation-writer.md:40`) | 「门禁阶段（第 1d 步那几条」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 人工指定；「main-agent.md 那一步」指针删，主 agent 侧动作 |
| iw-40 | 「验证命令里 clippy、b」(`.claude/agents/implementation-writer.md:40`) | 「验证命令只到这几样，各写一条」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 人工指定；「一行一条、不用分号、不写 echo $? >」在共用约束 :57 |
| iw-41 | 「第 1d 步的各条与第 3 」(`.claude/agents/implementation-writer.md:40`) | 「验证命令只到这几样，各写一条」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 改写 20%（低分行逐条人工核过） |
| iw-42 | 「派发提示另点名要跑的，照点名」(`.claude/agents/implementation-writer.md:40`) | 「派发提示另点名要跑的，照点名写进那一节」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 人工指定 |
| iw-43 | 「补丁改了包的 Cargo.t」(`.claude/agents/implementation-writer.md:40`) | 「补丁改了包的 Cargo.t」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:50`) | 改写 60% |
| iw-44 | 「改了 checker（cra」(`.claude/agents/implementation-writer.md:41`) | 「改了 checker（cra」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:51`) | 原话基本在 100% |
| iw-45 | 「主 agent 点崩溃枚举用」(`.claude/agents/implementation-writer.md:41`) | 「补丁动了 checker 而报告没这一节」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:51`) | 人工指定；说明句「主 agent 点…对着这一节看」删（消费方说明） |
| iw-46 | 「scripts/gate/a」(`.claude/agents/implementation-writer.md:41`) | 「补丁动了 checker 而」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:51`) | 原话基本在 81% |
| iw-47 | 「要改的文件在输入给的」(`.claude/agents/implementation-writer.md:42`) | 「要改的文件在输入给的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:52`) | 原话基本在 100% |
| iw-48 | 「改动里碰到条款没写、要做设计」(`.claude/agents/implementation-writer.md:43`) | 「改动里碰到条款没写、要做设计」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:53`) | 原话基本在 100% |
| iw-49 | 「走得到（包括要坏盘、崩溃、瞬」(`.claude/agents/implementation-writer.md:44`) | 「走得到（包括要坏盘、崩溃、瞬」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:54`) | 原话基本在 100% |
| iw-50 | 「在报告里写出」(`.claude/agents/implementation-writer.md:45`) | 「在报告里写出」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:55`) | 原话基本在 100% |
| iw-51 | 「条款没写的不是分支（trai」(`.claude/agents/implementation-writer.md:46`) | 「条款没写的不是分支（trai」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:55`) | 原话基本在 100% |
| iw-52 | 「报告单列一节，标题恰好是」(`.claude/agents/implementation-writer.md:47`) | 「报告单列一节，标题恰好是」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:56`) | 原话基本在 97% |
| iw-53 | 「代码轮三方拿这一节的每一处照」(`.claude/agents/implementation-writer.md:47`) | 「走不到的理由二次查实」(`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:82`) | 人工指定；落到技能（主 agent 侧） |
| iw-54 | 「新加崩溃点重放流或崩溃点重放」(`.claude/agents/implementation-writer.md:48`) | 「新加崩溃点重放流或崩溃点重放」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:57`) | 原话基本在 100% |
| iw-55 | 「新写的崩溃枚举用例（测试函数」(`.claude/agents/implementation-writer.md:49`) | 「新写的崩溃枚举用例（测试函数」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:58`) | 原话基本在 98% |
| iw-56 | 「每条都要的三处：.claud」(`.claude/agents/implementation-writer.md:49`) | 「每条都要的三处：.claud」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:59`) | 原话基本在 100% |
| iw-57 | 「有条件的三处：合表第三列点名」(`.claude/agents/implementation-writer.md:49`) | 「有条件的三处：合表第三列点名」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:60`) | 原话基本在 100% |
| iw-58 | 「crates/ 下的变异行、」(`.claude/agents/implementation-writer.md:49`) | 「crates/ 下的变异行、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:61`) | 原话基本在 89% |
| iw-59 | 「上下文多大都不交回，活没做完」(`.claude/agents/implementation-writer.md:50`) | 「上下文多大都不是交回的理由，」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 改写 53%（低分行逐条人工核过） |
| iw-60 | 「crates/（输入里标了别」(`.claude/agents/implementation-writer.md:54`) | 「crates/（输入里标了别」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:65`) | 原话基本在 99% |
| iw-61 | 「报告：这一轮写过的文件清单（」(`.claude/agents/implementation-writer.md:58`) | 「报告：这一轮写过的文件清单（」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:69`) | 原话基本在 100% |
| iw-62 | 「派发提示写明」(`.claude/agents/implementation-writer.md:59`) | 「补丁目录 <草稿目录>/pa」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:70`) | 原话基本在 76% |
| iw-63 | 「全文写进报告文件，交回只写结」(`.claude/agents/implementation-writer.md:60`) | 「交回只写结论、报告路径与 s」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 改写 45%（低分行逐条人工核过） |
| iw-64 | 「没往统一任务服务交任务（命令」(`.claude/agents/implementation-writer.md:64`) | 「没往统一任务服务交任务（命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:74`) | 原话基本在 100% |

### 定义 `.claude/agents/experiment-runner.md`（45 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| er-01 | 「单测跑完、产物一次没跑之前看」(`.claude/agents/experiment-runner.md:15`) | 「单测跑完、产物一次没跑之前看」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:15`) | 原话基本在 85% |
| er-02 | 「开工先读：.claude/r」(`.claude/agents/experiment-runner.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:16`) | 改写 19%（低分行逐条人工核过） |
| er-03 | 「跑前登记路径（researc」(`.claude/agents/experiment-runner.md:20`) | 「跑前登记路径（researc」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:20`) | 原话基本在 100% |
| er-04 | 「或者只修已有实验的变异表锚点」(`.claude/agents/experiment-runner.md:21`) | 「或者只修已有实验的变异表锚点」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:21`) | 原话基本在 91% |
| er-05 | 「这一段回答岔路单的哪几行：派」(`.claude/agents/experiment-runner.md:22`) | 「这一段回答岔路单的哪几行：派」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:22`) | 原话基本在 100% |
| er-06 | 「实验页与索引行要不要这一次写」(`.claude/agents/experiment-runner.md:23`) | 「实验页与索引行要不要这一次写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:23`) | 原话基本在 100% |
| er-07 | 「报告路径与草稿目录，都在 ~」(`.claude/agents/experiment-runner.md:24`) | 「报告路径与草稿目录，都在 ~」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:24`) | 改写 66% |
| er-08 | 「编译、单测、装置、变异与产物」(`.claude/agents/experiment-runner.md:28`) | 「编译、单测、装置、变异与产物照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:28`) | 人工指定；拼法细节下沉到共用约束「命令怎么写」 |
| er-09 | 「主 agent 跑完新派执行」(`.claude/agents/experiment-runner.md:28`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| er-10 | 「登记第六节里有耗时类的量时，」(`.claude/agents/experiment-runner.md:28`) | 「登记第六节里有耗时类的量时，」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:28`) | 原话基本在 100% |
| er-11 | 「scripts/experi」(`.claude/agents/experiment-runner.md:29`) | 「scripts/experi」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:29`) | 改写 46%（低分行逐条人工核过） |
| er-12 | 「单测、装置与变异各写一条：一」(`.claude/agents/experiment-runner.md:29`) | 「每一行最多一条交任务的命令，」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 改写 34%（低分行逐条人工核过） |
| er-13 | 「入库装置那一支（crates」(`.claude/agents/experiment-runner.md:30`) | 「入库装置那一支（crates」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:30`) | 改写 69% |
| er-14 | 「跑前登记第五节推的那一段超过」(`.claude/agents/experiment-runner.md:31`) | 「跑前登记第五节推的那一段超过」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:31`) | 原话基本在 100% |
| er-15 | 「写 scripts/expe」(`.claude/agents/experiment-runner.md:32`) | 「源文件、变异表、bin 名与」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:32`) | 改写 54%（低分行逐条人工核过） |
| er-16 | 「scripts/experi」(`.claude/agents/experiment-runner.md:32`) | 「research/ 那一支：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:33`) | 改写 63% |
| er-17 | 「中文简称只用在 kb 页），」(`.claude/agents/experiment-runner.md:32`) | 「⚠️ 跑前登记把装置写死成」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:34`) | 改写 51%（低分行逐条人工核过） |
| er-18 | 「另有四条只对入库装置生效：①」(`.claude/agents/experiment-runner.md:32`) | 「不许从 crates/ 里引」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:35`) | 原话基本在 74%；「四条只对入库装置生效」原样在 |
| er-19 | 「入库装置在验证包 crate」(`.claude/agents/experiment-runner.md:32`) | 「入库装置在验证包 crate」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:36`) | 原话基本在 87%；定义-1：「不进代码轮判决的点名清单」→「进代码轮三方」（矛盾表登记） |
| er-20 | 「入库装置」(`.claude/agents/experiment-runner.md:32`) | 「入库装置」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:37`) | 原话基本在 95% |
| er-21 | 「入库装置先在草稿目录的副本里」(`.claude/agents/experiment-runner.md:32`) | 「先在草稿目录的副本里改，编过」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:38`) | 改写 38%（低分行逐条人工核过） |
| er-22 | 「只追加，不改别人的）：纯算术」(`.claude/agents/experiment-runner.md:32`) | 「装置：纯算术的计数模型、单测」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:32`) | 改写 59% |
| er-23 | 「装置源码、变异表（补丁表）与」(`.claude/agents/experiment-runner.md:33`) | 「报告分段写，每一次写进文件的」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:33`) | 改写 49%（低分行逐条人工核过） |
| er-24 | 「第一段排他新建（Write 」(`.claude/agents/experiment-runner.md:33`) | 「装置源码、变异表（补丁表）与」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:39`) | 改写 24%（低分行逐条人工核过）；用户定-5：上下文吃紧可交回；共用约束 :68 写例外 |
| er-25 | 「表格与代码块整块放进同一段，」(`.claude/agents/experiment-runner.md:33`) | 「报告分段写，每一次写进文件的」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:33`) | 改写 56% |
| er-26 | 「不在读完源码之后一次写整份。」(`.claude/agents/experiment-runner.md:33`) | 「装置源码、变异表（补丁表）与」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:39`) | 原话基本在 100%；用户定-5：上下文吃紧可交回；共用约束 :68 写例外 |
| er-27 | 「装置照跑前登记」(`.claude/agents/experiment-runner.md:34`) | 「装置照跑前登记」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:40`) | 原话基本在 100% |
| er-28 | 「入库装置那一支不写 rese」(`.claude/agents/experiment-runner.md:35`) | 「变异：入库装置那一支不写 r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:41`) | 改写 67% |
| er-29 | 「登记里的停机条款（第十一节里」(`.claude/agents/experiment-runner.md:36`) | 「登记里的停机条款（第十一节里」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:42`) | 原话基本在 71% |
| er-30 | 「跑产物之前，把跑前登记里的每」(`.claude/agents/experiment-runner.md:37`) | 「跑产物之前，把跑前登记里的每」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:43`) | 原话基本在 86% |
| er-31 | 「产物命令把输出写进 rese」(`.claude/agents/experiment-runner.md:38`) | 「产物命令把输出写进 rese」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:44`) | 改写 57% |
| er-32 | 「产物出来之后先自查齐不齐：按」(`.claude/agents/experiment-runner.md:39`) | 「产物出来之后先自查齐不齐：按」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:45`) | 原话基本在 85% |
| er-33 | 「交回之前逐行读判决行：gre」(`.claude/agents/experiment-runner.md:40`) | 「交回之前逐行读判决行：gre」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:46`) | 原话基本在 91% |
| er-34 | 「跑了走崩溃判段的装置（按判段」(`.claude/agents/experiment-runner.md:40`) | 「跑了走崩溃判段的装置的（按判」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:47`) | 改写 38%（低分行逐条人工核过） |
| er-35 | 「在 scripts/expe」(`.claude/agents/experiment-runner.md:41`) | 「在 scripts/expe」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:48`) | 原话基本在 83% |
| er-36 | 「门禁阶段交回前不跑（改了 .」(`.claude/agents/experiment-runner.md:42`) | 「扫仓的门禁阶段（本地与共享）」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:108`) | 改写 35%（低分行逐条人工核过） |
| er-37 | 「整轮门禁判红派回给你修时，产」(`.claude/agents/experiment-runner.md:42`) | 「整轮门禁判红派回给你修时，产」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:49`) | 原话基本在 100% |
| er-38 | 「主 agent 要写实验页时」(`.claude/agents/experiment-runner.md:43`) | 「主 agent 要写实验页时」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:50`) | 原话基本在 100% |
| er-39 | 「实验页的 ### 影响的决策」(`.claude/agents/experiment-runner.md:44`) | 「实验页的 ### 影响的决策」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:51`) | 改写 37%（低分行逐条人工核过） |
| er-40 | 「产物命令预计跑超过 30 分」(`.claude/agents/experiment-runner.md:45`) | 「产物命令预计跑超过 30 分」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:52`) | 原话基本在 100% |
| er-41 | 「上面列的 bin 源文件与 」(`.claude/agents/experiment-runner.md:49`) | 「上面列的 bin 源文件与 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:56`) | 原话基本在 100% |
| er-42 | 「报告：交回的命令清单（」(`.claude/agents/experiment-runner.md:53`) | 「报告：交回的命令清单（」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:60`) | 原话基本在 100% |
| er-43 | 「登记落地清单」(`.claude/agents/experiment-runner.md:54`) | 「登记落地清单」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:61`) | 原话基本在 100% |
| er-44 | 「岔路表：登记里岔路单的每一行」(`.claude/agents/experiment-runner.md:55`) | 「岔路表：登记里岔路单的每一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:62`) | 原话基本在 100% |
| er-45 | 「没判这个实验的结论能不能推翻」(`.claude/agents/experiment-runner.md:59`) | 「没判这个实验的结论能不能推翻」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:66`) | 原话基本在 100% |

### 定义 `.claude/agents/experiment-designer.md`（39 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| ed-01 | 「判据在跑之前写死，设计时不看」(`.claude/agents/experiment-designer.md:15`) | 「判据在跑之前写死，设计时不看」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:15`) | 原话基本在 100% |
| ed-02 | 「开工先读：.claude/r」(`.claude/agents/experiment-designer.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:16`) | 原话基本在 100% |
| ed-03 | 「岔路单（research/p」(`.claude/agents/experiment-designer.md:20`) | 「岔路单（research/p」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:20`) | 原话基本在 100% |
| ed-04 | 「要回答的问题（一种读法；」(`.claude/agents/experiment-designer.md:21`) | 「要回答的问题（一种读法；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:21`) | 原话基本在 100% |
| ed-05 | 「重跑已有实验的：实验号与第几」(`.claude/agents/experiment-designer.md:22`) | 「重跑已有实验的：实验号与第几」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:22`) | 原话基本在 100% |
| ed-06 | 「被测条款：kb 文件路径与小」(`.claude/agents/experiment-designer.md:23`) | 「被测条款：kb 文件路径与小」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:23`) | 原话基本在 100% |
| ed-07 | 「实验简称（或由你按问题起一个」(`.claude/agents/experiment-designer.md:24`) | 「实验简称（或由你按问题起一个」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:24`) | 原话基本在 100% |
| ed-08 | 「先读被测条款与它的定义逐字（」(`.claude/agents/experiment-designer.md:29`) | 「先读被测条款与它的定义逐字（」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:29`) | 原话基本在 100% |
| ed-09 | 「bash scripts/k」(`.claude/agents/experiment-designer.md:30`) | 「bash scripts/k」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:30`) | 原话基本在 84% |
| ed-10 | 「不读 .claude/kb/」(`.claude/agents/experiment-designer.md:30`) | 「不读 .claude/kb/」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:31`) | 原话基本在 93% |
| ed-11 | 「登记写明：问题，连同岔路单逐」(`.claude/agents/experiment-designer.md:31`) | 「第六节每个量写明它对应岔路单」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:33`) | 改写 67% |
| ed-12 | 「每条臂的定义（」(`.claude/agents/experiment-designer.md:31`) | 「每条臂的定义（」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:34`) | 原话基本在 100% |
| ed-13 | 「钉绝对值的锚点分两类写：出自」(`.claude/agents/experiment-designer.md:31`) | 「钉绝对值的断言，锚点分两类写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:35`) | 原话基本在 93% |
| ed-14 | 「装置写在哪要在登记里写死一句」(`.claude/agents/experiment-designer.md:31`) | 「装置写在哪要在登记里写死一句」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:36`) | 改写 68% |
| ed-15 | 「计时类的量写明在哪个进程里、」(`.claude/agents/experiment-designer.md:31`) | 「计时类的量写明在哪个进程里、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:37`) | 原话基本在 88% |
| ed-16 | 「装置走到崩溃判段的（按判段入」(`.claude/agents/experiment-designer.md:31`) | 「装置走到崩溃判段的（按判段入」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:38`) | 原话基本在 77% |
| ed-17 | 「登记写死之前核可跑性与判别力」(`.claude/agents/experiment-designer.md:32`) | 「登记写死之前核可跑性与判别力」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:39`) | 原话基本在 77% |
| ed-18 | 「估一下实现量：历史族 × 几」(`.claude/agents/experiment-designer.md:33`) | 「估一下实现量：历史族 × 几」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:40`) | 原话基本在 100% |
| ed-19 | 「## 十四、并行与挂钟」(`.claude/agents/experiment-designer.md:34`) | 「## 十四、并行与挂钟」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:41`) | 原话基本在 100% |
| ed-20 | 「差分重跑登记：问题单只说」(`.claude/agents/experiment-designer.md:35`) | 「差分重跑登记：问题单只说」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:42`) | 原话基本在 95% |
| ed-21 | 「按这个次序写，节名照抄；」(`.claude/agents/experiment-designer.md:39`) | 「按这个次序写，节名照抄；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:46`) | 原话基本在 100% |
| ed-22 | 「## 一、问题」(`.claude/agents/experiment-designer.md:43`) | 「## 一、问题」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:50`) | 原话基本在 100% |
| ed-23 | 「## 二、被测条款与它引的定」(`.claude/agents/experiment-designer.md:44`) | 「## 二、被测条款与它引的定」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:51`) | 原话基本在 100% |
| ed-24 | 「## 三、实现今天的样子」(`.claude/agents/experiment-designer.md:45`) | 「## 三、实现今天的样子」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:52`) | 原话基本在 100% |
| ed-25 | 「## 四、跑之前已经存在的数」(`.claude/agents/experiment-designer.md:46`) | 「## 四、跑之前已经存在的数」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:53`) | 原话基本在 100% |
| ed-26 | 「## 五、臂、阳性对照、真实」(`.claude/agents/experiment-designer.md:47`) | 「## 五、臂、阳性对照、真实」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:54`) | 原话基本在 100% |
| ed-27 | 「## 六、报哪些量与各自的判」(`.claude/agents/experiment-designer.md:48`) | 「## 六、报哪些量与各自的判」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:55`) | 原话基本在 100% |
| ed-28 | 「## 七、钉绝对值的断言」(`.claude/agents/experiment-designer.md:49`) | 「## 七、钉绝对值的断言」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:56`) | 原话基本在 100% |
| ed-29 | 「## 八、轨迹与几何敏感性」(`.claude/agents/experiment-designer.md:50`) | 「## 八、轨迹与几何敏感性」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:57`) | 原话基本在 100% |
| ed-30 | 「## 九、变异」(`.claude/agents/experiment-designer.md:51`) | 「## 九、变异」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:58`) | 原话基本在 100% |
| ed-31 | 「## 十、失败条款」(`.claude/agents/experiment-designer.md:52`) | 「## 十、失败条款」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:59`) | 原话基本在 100% |
| ed-32 | 「## 十一、作废条款与停机条」(`.claude/agents/experiment-designer.md:53`) | 「## 十一、作废条款与停机条」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:60`) | 原话基本在 100% |
| ed-33 | 「## 十二、修订」(`.claude/agents/experiment-designer.md:54`) | 「## 十二、修订」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:61`) | 原话基本在 100% |
| ed-34 | 「## 十三、读过的文件与跑过」(`.claude/agents/experiment-designer.md:55`) | 「## 十三、读过的文件与跑过」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:62`) | 原话基本在 100% |
| ed-35 | 「## 十四、并行与挂钟」(`.claude/agents/experiment-designer.md:56`) | 「## 十四、并行与挂钟」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:63`) | 原话基本在 100% |
| ed-36 | 「只写 research/pr」(`.claude/agents/experiment-designer.md:60`) | 「只写 research/pr」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:67`) | 原话基本在 100% |
| ed-37 | 「登记里待主 agent 认的」(`.claude/agents/experiment-designer.md:64`) | 「登记里待主 agent 认的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:71`) | 原话基本在 100% |
| ed-38 | 「登记落盘之后、实验页建起来之」(`.claude/agents/experiment-designer.md:65`) | 「登记落盘之后、实验页建起来之」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:72`) | 原话基本在 100% |
| ed-39 | 「没写代码、没跑、没往统一任务」(`.claude/agents/experiment-designer.md:69`) | 「没写代码、没跑、没往统一任务」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-designer.md:76`) | 原话基本在 100% |

### 定义 `.claude/agents/tooling-writer.md`（28 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| tl-01 | 「两种活：一是门禁阶段、钩子、」(`.claude/agents/tooling-writer.md:15`) | 「两种活：一是门禁阶段、钩子、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:15`) | 原话基本在 100% |
| tl-02 | 「开工先读：.claude/s」(`.claude/agents/tooling-writer.md:16`) | 「开工先读：.claude/s」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:16`) | 改写 55% |
| tl-03 | 「关的是哪几条（可以不止一条，」(`.claude/agents/tooling-writer.md:20`) | 「关的是哪几条（可以不止一条，」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:20`) | 原话基本在 100% |
| tl-04 | 「要改的文件，全列（写成行首」(`.claude/agents/tooling-writer.md:21`) | 「要改的文件，全列（写成行首」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:21`) | 原话基本在 100% |
| tl-05 | 「这一轮别的会话在改的文件（你」(`.claude/agents/tooling-writer.md:22`) | 「这一轮别的会话在改的文件（你」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:22`) | 原话基本在 100% |
| tl-06 | 「每条改法要判红的输入样本与弄」(`.claude/agents/tooling-writer.md:24`) | 「每条改法要判红的输入样本与弄」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:24`) | 原话基本在 100% |
| tl-07 | 「报告路径与草稿目录，都在 ~」(`.claude/agents/tooling-writer.md:25`) | 「报告路径与草稿目录，都在 ~」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:25`) | 原话基本在 100% |
| tl-08 | 「自证、样本档、弄坏开关、命名」(`.claude/agents/tooling-writer.md:29`) | 「自证、样本档、弄坏开关、命名」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:29`) | 改写 49%（低分行逐条人工核过） |
| tl-09 | 「主 agent 跑完新派工具」(`.claude/agents/tooling-writer.md:29`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| tl-10 | 「门禁阶段照第 7 步不跑。」(`.claude/agents/tooling-writer.md:29`) | 「门禁阶段照第 7 步不跑。」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:29`) | 原话基本在 100% |
| tl-11 | 「每条改法先造一个该红的输入：」(`.claude/agents/tooling-writer.md:30`) | 「每条改法先造一个该红的输入：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:30`) | 原话基本在 100% |
| tl-12 | 「改已存在的脚本写同目录临时文」(`.claude/agents/tooling-writer.md:31`) | 「改已存在的脚本照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:31`) | 原话基本在 89% |
| tl-13 | 「不在同一个 inode 上就」(`.claude/agents/tooling-writer.md:31`) | 「在同一个 inode 上改已」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:81`) | 改写 55%（低分行逐条人工核过） |
| tl-14 | 「改 .claude/hook」(`.claude/agents/tooling-writer.md:31`) | 「改 .claude/hook」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:31`) | 原话基本在 99% |
| tl-15 | 「新建文件排他（Write 前」(`.claude/agents/tooling-writer.md:31`) | 「新建文件，有 Write 的」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:32`) | 改写 32%（低分行逐条人工核过） |
| tl-16 | 「新门禁阶段与新钩子写 # g」(`.claude/agents/tooling-writer.md:32`) | 「新门禁阶段与新钩子写 # g」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:32`) | 原话基本在 100% |
| tl-17 | 「自证里只对自己起的进程号发信」(`.claude/agents/tooling-writer.md:33`) | 「自证、门禁判法与装置里起的进程逐个按进程」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:33`) | 人工指定；用户定-6 |
| tl-18 | 「第二种活：只写判决或规格给的」(`.claude/agents/tooling-writer.md:34`) | 「第二种活：只写判决或规格给的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:34`) | 原话基本在 100% |
| tl-19 | 「交回前不跑门禁阶段与各类 l」(`.claude/agents/tooling-writer.md:35`) | 「交回前不跑门禁阶段与各类 l」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:35`) | 原话基本在 100% |
| tl-20 | 「改了 scripts/ 下 」(`.claude/agents/tooling-writer.md:35`) | 「命名检查交主 agent 跑」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:109`) | 改写 47%（低分行逐条人工核过） |
| tl-21 | 「改了的脚本与门禁，--sel」(`.claude/agents/tooling-writer.md:35`) | 「改了的脚本与门禁，--sel」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:35`) | 改写 70% |
| tl-22 | 「上下文多大都不交回，活没做完」(`.claude/agents/tooling-writer.md:36`) | 「上下文多大都不是交回的理由，」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 改写 53%（低分行逐条人工核过） |
| tl-23 | 「.claude/hooks/」(`.claude/agents/tooling-writer.md:40`) | 「.claude/hooks/」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:39`) | 原话基本在 100% |
| tl-24 | 「第二种活另加 .claude」(`.claude/agents/tooling-writer.md:41`) | 「第二种活另加 .claude」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:40`) | 原话基本在 100% |
| tl-25 | 「不写 .claude/sin」(`.claude/agents/tooling-writer.md:42`) | 「不写 .claude/sin」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:41`) | 原话基本在 100% |
| tl-26 | 「报告：改过的文件清单与 gi」(`.claude/agents/tooling-writer.md:46`) | 「报告：改过的文件清单与 gi」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:45`) | 原话基本在 100% |
| tl-27 | 「全文写进报告文件，交回只写结」(`.claude/agents/tooling-writer.md:46`) | 「交回只写结论、报告路径与 s」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 改写 63% |
| tl-28 | 「没跑门禁与测试（crash-」(`.claude/agents/tooling-writer.md:50`) | 「没跑门禁与测试（crash-」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:49`) | 原话基本在 100% |

### 定义 `.claude/agents/kb-scribe.md`（30 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| kb-01 | 「两段活在一个 agent 里」(`.claude/agents/kb-scribe.md:15`) | 「两段活在一个 agent 里」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:15`) | 原话基本在 98% |
| kb-02 | 「开工先读：.claude/s」(`.claude/agents/kb-scribe.md:16`) | 「开工先读：.claude/s」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:16`) | 原话基本在 100% |
| kb-03 | 「判决路径与要写回的条目编号（」(`.claude/agents/kb-scribe.md:20`) | 「判决路径与要写回的条目编号（」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:20`) | 原话基本在 100% |
| kb-04 | 「要不要记决策变更史：条目的每」(`.claude/agents/kb-scribe.md:21`) | 「要不要记决策变更史：条目的每」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:21`) | 改写 58%；细则落 changelog-format.md :8-26（量的） |
| kb-05 | 「分项翻状态的：决策号与分项号」(`.claude/agents/kb-scribe.md:22`) | 「分项翻状态的：决策号与分项号」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:22`) | 原话基本在 100% |
| kb-06 | 「欠账表要加或还的行：也写成逐」(`.claude/agents/kb-scribe.md:23`) | 「欠账表要加或还的行：也写成逐」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:23`) | 原话基本在 100% |
| kb-07 | 「写决策正文（新立分项、或改一」(`.claude/agents/kb-scribe.md:24`) | 「写决策正文（新立分项、或改一」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:24`) | 改写 57% |
| kb-08 | 「规格改了某条分项的」(`.claude/agents/kb-scribe.md:25`) | 「规格改了某条分项的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:25`) | 原话基本在 100% |
| kb-09 | 「报告路径与草稿目录，都在 ~」(`.claude/agents/kb-scribe.md:26`) | 「报告路径与草稿目录，都在」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:26`) | 人工指定；括注「写范围闸对你放行…」删，写范围在「写范围」一节 |
| kb-10 | 「起草规格（给了判决时）：逐条」(`.claude/agents/kb-scribe.md:30`) | 「起草规格（给了判决时）：逐条」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:30`) | 原话基本在 100% |
| kb-11 | 「新串里的编号一律写成」(`.claude/agents/kb-scribe.md:30`) | 「写进仓里的注释与 .md 引」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:42`) | 改写 36%（低分行逐条人工核过） |
| kb-12 | 「动了某条分项」(`.claude/agents/kb-scribe.md:30`) | 「动了某条分项」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:30`) | 原话基本在 100% |
| kb-13 | 「开工时先记下规格点名的文件、」(`.claude/agents/kb-scribe.md:31`) | 「开工时先记下规格点名的文件、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:31`) | 原话基本在 92% |
| kb-14 | 「决策变更史：条目写进 .cl」(`.claude/agents/kb-scribe.md:32`) | 「决策变更史：条目写进 .cl」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:32`) | 改写 61%；细则落 changelog-format.md :8-26（量的） |
| kb-15 | 「一条改动点名几条决策的，只在」(`.claude/agents/kb-scribe.md:32`) | 「一条改动点名几条决策：只在标」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:26`) | 原话基本在 79% |
| kb-16 | 「规格改了某条决策的状态或分项」(`.claude/agents/kb-scribe.md:32`) | 「规格改了某条决策的状态或分项」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:32`) | 原话基本在 89%；细则落 changelog-format.md :8-26（量的） |
| kb-17 | 「分项翻状态：python3 」(`.claude/agents/kb-scribe.md:33`) | 「分项翻状态：python3 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:33`) | 原话基本在 100% |
| kb-18 | 「然后照第 2 步把 SING」(`.claude/agents/kb-scribe.md:33`) | 「规格改了某条决策的状态或分项」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:32`) | 原话基本在 96% |
| kb-19 | 「规格里决策那几处与实验页那几」(`.claude/agents/kb-scribe.md:34`) | 「规格里决策那几处与实验页那几」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:34`) | 原话基本在 100% |
| kb-20 | 「新立分项、改索引行这类不翻状」(`.claude/agents/kb-scribe.md:35`) | 「规格改了某条决策的状态或分项」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:32`) | 原话基本在 84% |
| kb-21 | 「门禁阶段交回前不跑，由提交前」(`.claude/agents/kb-scribe.md:36`) | 「扫仓的门禁阶段（本地与共享）」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:108`) | 改写 50%（低分行逐条人工核过） |
| kb-22 | 「第 2、3 步里的 --wr」(`.claude/agents/kb-scribe.md:36`) | 「第 2、3 步里的 --wr」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:36`) | 原话基本在 100% |
| kb-23 | 「收尾再记一次每个被改文件的 」(`.claude/agents/kb-scribe.md:37`) | 「收尾再记一次每个被改文件的 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:37`) | 原话基本在 100% |
| kb-24 | 「.claude/kb/。」(`.claude/agents/kb-scribe.md:41`) | 「.claude/kb/。」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:41`) | 原话基本在 100% |
| kb-25 | 「crates/singlef」(`.claude/agents/kb-scribe.md:42`) | 「crates/singlef」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:42`) | 原话基本在 100% |
| kb-26 | 「relabel-item.p」(`.claude/agents/kb-scribe.md:43`) | 「relabel-item.p」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:43`) | 原话基本在 100% |
| kb-27 | 「~/.local/share」(`.claude/agents/kb-scribe.md:44`) | 「~/.local/share」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:44`) | 原话基本在 100% |
| kb-28 | 「用 Edit 写的部分与 .」(`.claude/agents/kb-scribe.md:45`) | 「用 Edit 写的部分与 .」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:45`) | 原话基本在 100% |
| kb-29 | 「报告：规格文件路径与 sha」(`.claude/agents/kb-scribe.md:49`) | 「报告：规格文件路径与 sha」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:49`) | 原话基本在 100% |
| kb-30 | 「没判定案对不对（判决里的判断」(`.claude/agents/kb-scribe.md:53`) | 「没判定案对不对（判决里的判断」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/kb-scribe.md:53`) | 原话基本在 100% |

### 定义 `.claude/agents/investigator.md`（22 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| inv-01 | 「先查：复现、定位、给推翻条件」(`.claude/agents/investigator.md:15`) | 「先查：复现、定位、给推翻条件」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:15`) | 原话基本在 83%；定义-11：第 7 步改第 6 步 |
| inv-02 | 「开工先读：.claude/r」(`.claude/agents/investigator.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:16`) | 原话基本在 99% |
| inv-03 | 「.claude/rules/」(`.claude/agents/investigator.md:16`) | 「核了窄的那一句，说出口的却是宽的那一句」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:16`) | 人工指定；指针换文件（verify-before-claiming 并进 evidence-discipline） |
| inv-04 | 「.claude/rules/」(`.claude/agents/investigator.md:16`) | 「核了窄的那一句，说出口的却是宽的那一句」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:16`) | 人工指定；指针换文件（verify-before-claiming 并进 evidence-discipline） |
| inv-05 | 「现象原文：产物整行、门禁的 」(`.claude/agents/investigator.md:20`) | 「现象原文：产物整行、门禁的 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:20`) | 原话基本在 100% |
| inv-06 | 「要回答的问题：是真是假、机理」(`.claude/agents/investigator.md:21`) | 「要回答的问题：是真是假、机理」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:21`) | 原话基本在 100% |
| inv-07 | 「只读，还是可以在草稿目录的仓」(`.claude/agents/investigator.md:22`) | 「只读，还是可以在草稿目录的仓」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:22`) | 原话基本在 100% |
| inv-08 | 「查到就修」(`.claude/agents/investigator.md:23`) | 「查到就修」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:23`) | 原话基本在 100%；用户定-19 待落：规则里还没写「这是一律交补丁的例外」 |
| inv-09 | 「报告路径与草稿目录，都在 ~」(`.claude/agents/investigator.md:24`) | 「报告路径与草稿目录，都在 ~」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:24`) | 原话基本在 100% |
| inv-10 | 「编译、测试、复现与探针照共用」(`.claude/agents/investigator.md:28`) | 「编译、测试、复现与探针照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:28`) | 人工指定 |
| inv-11 | 「主 agent 跑完新派调查」(`.claude/agents/investigator.md:28`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| inv-12 | 「先原样复现：在草稿目录建仓副」(`.claude/agents/investigator.md:29`) | 「先原样复现：照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:29`) | 人工指定；建副本命令写法在共用约束「建仓副本」 |
| inv-13 | 「输入命令是整包、整工作区或整」(`.claude/agents/investigator.md:29`) | 「输入命令是整包、整工作区或整」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:29`) | 原话基本在 100% |
| inv-14 | 「二分到文件:行：缩小输入、改」(`.claude/agents/investigator.md:30`) | 「二分到文件:行：缩小输入、改」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:30`) | 原话基本在 90% |
| inv-15 | 「给最小复现：一条命令加它的原」(`.claude/agents/investigator.md:31`) | 「给最小复现：一条命令加它的原」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:31`) | 原话基本在 75% |
| inv-16 | 「要跑的是不指定范围的（范围声」(`.claude/agents/investigator.md:32`) | 「要跑的是不指定范围的（范围声」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:32`) | 原话基本在 77% |
| inv-17 | 「上下文多大都不交回，活没做完」(`.claude/agents/investigator.md:33`) | 「上下文多大都不是交回的理由，」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 改写 53%（低分行逐条人工核过） |
| inv-18 | 「派发提示写了」(`.claude/agents/investigator.md:34`) | 「派发提示写了」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:33`) | 改写 63%；定义-11：第 7 步改第 6 步 |
| inv-19 | 「报告文件与草稿目录（仓副本里」(`.claude/agents/investigator.md:38`) | 「报告文件与草稿目录（仓副本里」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:37`) | 原话基本在 100% |
| inv-20 | 「报告：复现命令与原样输出（承」(`.claude/agents/investigator.md:42`) | 「报告：复现命令与原样输出（承」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:41`) | 原话基本在 100% |
| inv-21 | 「全文写进报告文件，交回只写结」(`.claude/agents/investigator.md:42`) | 「交回只写结论、报告路径与 s」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 改写 63% |
| inv-22 | 「查到就修」(`.claude/agents/investigator.md:46`) | 「查到就修」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/investigator.md:45`) | 原话基本在 100% |

### 定义 `.claude/agents/clerk.md`（15 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| cl-01 | 「只做派发提示写明的那一件活，」(`.claude/agents/clerk.md:15`) | 「只做派发提示写明的那一件活，」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:15`) | 原话基本在 100% |
| cl-02 | 「开工先读：派发提示点名的文件」(`.claude/agents/clerk.md:16`) | 「开工先读：派发提示点名的文件」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:16`) | 原话基本在 100% |
| cl-03 | 「规则读三样，别的不读：共用约」(`.claude/agents/clerk.md:16`) | 「每个定义都照守、不再各自点名」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:20`) | 改写 37%（低分行逐条人工核过） |
| cl-04 | 「改 records/ 下的文」(`.claude/agents/clerk.md:16`) | 「改 records/ 下的文」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:16`) | 原话基本在 76% |
| cl-05 | 「活的描述与出口：做成什么算完」(`.claude/agents/clerk.md:20`) | 「活的描述与出口：做成什么算完」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:20`) | 原话基本在 100% |
| cl-06 | 「要读的文件、要跑的只读命令与」(`.claude/agents/clerk.md:21`) | 「要读的文件、要跑的只读命令与」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:21`) | 原话基本在 100% |
| cl-07 | 「报告路径与草稿目录，都在 ~」(`.claude/agents/clerk.md:22`) | 「报告路径与草稿目录，都在 ~」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:22`) | 原话基本在 100% |
| cl-08 | 「要现查的事实贴命令与原样输出」(`.claude/agents/clerk.md:26`) | 「能用命令核的事实，贴命令与原」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:116`) | 改写 29%（低分行逐条人工核过） |
| cl-09 | 「活里要起的编译、测试与别的脚」(`.claude/agents/clerk.md:27`) | 「活里要起的编译、测试与别的脚本」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:27`) | 人工指定；「读 task log 接着做」半句在共用约束 |
| cl-10 | 「主 agent 跑完新派事务」(`.claude/agents/clerk.md:27`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| cl-11 | 「上下文多大都不交回，活没做完」(`.claude/agents/clerk.md:28`) | 「上下文多大都不是交回的理由，」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:68`) | 改写 53%（低分行逐条人工核过） |
| cl-12 | 「records/、resea」(`.claude/agents/clerk.md:32`) | 「records/、resea」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:31`) | 原话基本在 99% |
| cl-13 | 「报告：做成了什么、依据（命令」(`.claude/agents/clerk.md:36`) | 「报告：做成了什么、依据（命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:35`) | 原话基本在 100% |
| cl-14 | 「全文写进报告文件，交回只写结」(`.claude/agents/clerk.md:36`) | 「交回只写结论、报告路径与 s」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 改写 63% |
| cl-15 | 「没往统一任务服务交任务（命令」(`.claude/agents/clerk.md:40`) | 「没往统一任务服务交任务（命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/clerk.md:39`) | 原话基本在 100% |

### 定义 `.claude/agents/mutation-triage.md`（21 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| mt-01 | 「开工先读：.claude/r」(`.claude/agents/mutation-triage.md:15`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:15`) | 原话基本在 100% |
| mt-02 | 「不接没有表的广谱变异活动（c」(`.claude/agents/mutation-triage.md:19`) | 「不接没有表的广谱变异活动（c」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:19`) | 原话基本在 100% |
| mt-03 | 「变异表：scripts/mu」(`.claude/agents/mutation-triage.md:20`) | 「变异表：scripts/mu」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:20`) | 原话基本在 100% |
| mt-04 | 「给的是 crates/mut」(`.claude/agents/mutation-triage.md:21`) | 「给的是 crates/mut」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:21`) | 原话基本在 88% |
| mt-05 | 「改动前的三个数（有就给）。」(`.claude/agents/mutation-triage.md:22`) | 「改动前的三个数（有就给）。」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:22`) | 原话基本在 100% |
| mt-06 | 「分 crates/mutat」(`.claude/agents/mutation-triage.md:23`) | 「给的是 crates/mut」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:21`) | 改写 56%；定义-6：两项并成一项 |
| mt-07 | 「主 agent 给了输出路径」(`.claude/agents/mutation-triage.md:28`) | 「主 agent 给了输出路径」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:27`) | 改写 45%（低分行逐条人工核过） |
| mt-08 | 「先逐条做子串计数（grep 」(`.claude/agents/mutation-triage.md:29`) | 「先逐条做子串计数（grep 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:28`) | 原话基本在 100% |
| mt-09 | 「research/ 那侧的表」(`.claude/agents/mutation-triage.md:30`) | 「research/ 那侧的表」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:29`) | 原话基本在 86% |
| mt-10 | 「crates 那张表同样不跑」(`.claude/agents/mutation-triage.md:30`) | 「crates 那张表不写命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:29`) | 人工指定；指针改指共用约束（定义-7） |
| mt-11 | 「主 agent 给你那一次 」(`.claude/agents/mutation-triage.md:30`) | 「crates 那张表不写命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:29`) | 改写 52%（低分行逐条人工核过） |
| mt-12 | 「确认整张表跑完：mutate」(`.claude/agents/mutation-triage.md:31`) | 「确认整张表跑完：mutate」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:30`) | 改写 53%（低分行逐条人工核过） |
| mt-13 | 「报抓到 / 无效 / 没红三」(`.claude/agents/mutation-triage.md:32`) | 「mutate.sh 没有三数」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:58`) | 原话基本在 80% |
| mt-14 | 「crates-mutatio」(`.claude/agents/mutation-triage.md:32`) | 「crates-mutatio」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:33`) | 改写 55% |
| mt-15 | 「无效、没红都让整道判红；」(`.claude/agents/mutation-triage.md:32`) | 「无效、没红都让整道判红；」(`research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:69`) | 改写 59% |
| mt-16 | 「基线不绿不在」(`.claude/agents/mutation-triage.md:32`) | 「基线不绿不在」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:33`) | 改写 43%（低分行逐条人工核过） |
| mt-17 | 「与改动前的数比，」(`.claude/agents/mutation-triage.md:32`) | 「与改动前的数比，」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:34`) | 原话基本在 100% |
| mt-18 | 「没红与无效的逐条按 muta」(`.claude/agents/mutation-triage.md:33`) | 「没红与无效的逐条按 muta」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:35`) | 原话基本在 100% |
| mt-19 | 「不改变异表、不改源码、不改测」(`.claude/agents/mutation-triage.md:37`) | 「不改变异表、不改源码、不改测」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:39`) | 原话基本在 100% |
| mt-20 | 「三个数与其余几栏（mutat」(`.claude/agents/mutation-triage.md:41`) | 「crates-mutatio」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:43`) | 改写 55% |
| mt-21 | 「没往统一任务服务交任务（cr」(`.claude/agents/mutation-triage.md:45`) | 「没往统一任务服务交任务（cr」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/mutation-triage.md:47`) | 原话基本在 100% |

### 定义 `.claude/agents/prior-art.md`（12 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| pa-01 | 「别家怎么做是线索，不是证据：」(`.claude/agents/prior-art.md:15`) | 「别家怎么做是线索，不是证据：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:15`) | 原话基本在 100% |
| pa-02 | 「开工先读：.claude/r」(`.claude/agents/prior-art.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:16`) | 原话基本在 98% |
| pa-03 | 「.claude/rules/」(`.claude/agents/prior-art.md:16`) | 「陈述外部状态之前，现查一次」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:16`) | 人工指定；指针换文件，并列出三个子小节 |
| pa-04 | 「要查的问题，以及要比的对象（」(`.claude/agents/prior-art.md:20`) | 「要查的问题，以及要比的对象（」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:20`) | 原话基本在 100% |
| pa-05 | 「本机有源码树的先查本机（贴 」(`.claude/agents/prior-art.md:25`) | 「本机有源码树的先查本机（贴 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:25`) | 原话基本在 100% |
| pa-06 | 「每条事实写：出处、日期、实测」(`.claude/agents/prior-art.md:26`) | 「每条事实写出处、日期、实测还」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:26`) | 改写 68% |
| pa-07 | 「每条写一处它与本工程的已知差」(`.claude/agents/prior-art.md:27`) | 「每条写一处它与本工程的已知差」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:27`) | 原话基本在 92% |
| pa-08 | 「没有任何现役实现这样做」(`.claude/agents/prior-art.md:28`) | 「没有任何现役实现这样做」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:28`) | 原话基本在 100% |
| pa-09 | 「不改 .claude/kb/」(`.claude/agents/prior-art.md:32`) | 「不改 .claude/kb/」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:32`) | 原话基本在 89% |
| pa-10 | 「一张表：事实 / 出处与日期」(`.claude/agents/prior-art.md:36`) | 「一张表：事实 / 出处与日期」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:36`) | 原话基本在 100% |
| pa-11 | 「然后反证清单与」(`.claude/agents/prior-art.md:36`) | 「跑不动的、没验的、按定义不归你的，照实列」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:119`) | 人工指定；「与没做什么」半句删，由共用约束「报告」一节要求 |
| pa-12 | 「没论证它适不适合本工程；」(`.claude/agents/prior-art.md:40`) | 「没论证它适不适合本工程；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/prior-art.md:40`) | 原话基本在 100% |

### 定义 `.claude/agents/sweep.md`（42 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| sw-01 | 「清单交主 agent，要改的」(`.claude/agents/sweep.md:15`) | 「清单交主 agent，要改的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:15`) | 原话基本在 100% |
| sw-02 | 「开工先读：.claude/r」(`.claude/agents/sweep.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:16`) | 原话基本在 99% |
| sw-03 | 「.claude/rules/」(`.claude/agents/sweep.md:16`) | 「核了窄的那一句，说出口的却是宽的那一句」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:16`) | 人工指定；指针换文件 |
| sw-04 | 「改了一个数或格式常量：旧值、」(`.claude/agents/sweep.md:21`) | 「改了一个数或格式常量：旧值、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:21`) | 原话基本在 100% |
| sw-05 | 「撤回一条结论：结论原文所在的」(`.claude/agents/sweep.md:22`) | 「撤回一条结论：结论原文所在的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:22`) | 原话基本在 100% |
| sw-06 | 「新立一条判据：判据原文所在的」(`.claude/agents/sweep.md:23`) | 「新立一条判据：判据原文所在的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:23`) | 原话基本在 100% |
| sw-07 | 「阶段同步（一个阶段任务结束、」(`.claude/agents/sweep.md:24`) | 「阶段同步（一个阶段任务结束、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:24`) | 原话基本在 100% |
| sw-08 | 「逐行盘点一张表的现状：表的路」(`.claude/agents/sweep.md:25`) | 「逐行盘点一张表的现状：表的路」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:25`) | 原话基本在 100% |
| sw-09 | 「第 1–4 步是改数的做法；」(`.claude/agents/sweep.md:30`) | 「第 1–4 步是改数的做法；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:30`) | 原话基本在 97%；定义-8：补「逐行盘点走第 12 步」 |
| sw-10 | 「按 format-evolu」(`.claude/agents/sweep.md:32`) | 「按 format-evolu」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:32`) | 原话基本在 100% |
| sw-11 | 「按量名、常量名、kb 用词再」(`.claude/agents/sweep.md:33`) | 「按量名、常量名、kb 用词再」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:33`) | 改写 67% |
| sw-12 | 「在历史版本节或 resear」(`.claude/agents/sweep.md:34`) | 「在历史版本节或 resear」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:34`) | 原话基本在 94% |
| sw-13 | 「给出可登记进 format-」(`.claude/agents/sweep.md:35`) | 「给出可登记进 format-」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:35`) | 原话基本在 91% |
| sw-14 | 「撤回结论的：按结论里的关键数」(`.claude/agents/sweep.md:36`) | 「撤回结论的：按结论里的关键数」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:36`) | 原话基本在 100% |
| sw-15 | 「新立判据的：把它管的那一类对」(`.claude/agents/sweep.md:37`) | 「新立判据的：把它管的那一类对」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:37`) | 原话基本在 93% |
| sw-16 | 「阶段同步分两段，派发提示写明」(`.claude/agents/sweep.md:38`) | 「阶段同步分两段，派发提示写明」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:38`) | 原话基本在 89% |
| sw-17 | 「一行写一件事实，同一件事实的」(`.claude/agents/sweep.md:38`) | 「一行一件事实，同一件事实的几」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:39`) | 原话基本在 88% |
| sw-18 | 「检索词写新旧说法都会提到的概」(`.claude/agents/sweep.md:38`) | 「检索词写新旧说法都会提到的概」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:40`) | 原话基本在 82% |
| sw-19 | 「实现落地、实验跑完、欠账还清」(`.claude/agents/sweep.md:38`) | 「只有这一阶段之前一个字都没提」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:41`) | 改写 50%（低分行逐条人工核过） |
| sw-20 | 「在结束那一版命中不超过 50」(`.claude/agents/sweep.md:38`) | 「在结束那一版命中不超过 50」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:40`) | 原话基本在 98% |
| sw-21 | 「事实真变了的（数、状态、口径」(`.claude/agents/sweep.md:38`) | 「只改措辞的行引的变更记录标题」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:42`) | 改写 60% |
| sw-22 | 「出处写它来自哪几条 H 编号」(`.claude/agents/sweep.md:38`) | 「出处写它来自哪几条 H 编号」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:39`) | 原话基本在 100% |
| sw-23 | 「只改措辞、没改事实的变更记录」(`.claude/agents/sweep.md:38`) | 「只改措辞、没改事实的变更记录」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:42`) | 原话基本在 100% |
| sw-24 | 「写事实表时每个检索词先报它在」(`.claude/agents/sweep.md:39`) | 「每个检索词先报它在结束那一版」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:43`) | 原话基本在 76% |
| sw-25 | 「写完跑 stale-cand」(`.claude/agents/sweep.md:40`) | 「写完跑 stale-cand」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:44`) | 原话基本在 100% |
| sw-26 | 「逐行判：只判候选表里分给你的」(`.claude/agents/sweep.md:41`) | 「逐行判：只判候选表里分给你的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:45`) | 原话基本在 100% |
| sw-27 | 「判据：这一行里每个描述现状的」(`.claude/agents/sweep.md:41`) | 「判据：这一行里每个描述现状的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:46`) | 原话基本在 100% |
| sw-28 | 「分句带日期的，把日期删掉再读」(`.claude/agents/sweep.md:41`) | 「分句带日期的，把日期删掉再读」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:47`) | 原话基本在 100% |
| sw-29 | 「判定：要改（给改后的句子）；」(`.claude/agents/sweep.md:41`) | 「判定：要改（给改后的句子）；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:48`) | 原话基本在 100% |
| sw-30 | 「读完分到的组，再反向问一遍：」(`.claude/agents/sweep.md:42`) | 「读完分到的组，再反向问一遍：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:49`) | 原话基本在 100% |
| sw-31 | 「交回前跑 python3 s」(`.claude/agents/sweep.md:43`) | 「交回前跑 python3 s」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:50`) | 原话基本在 100% |
| sw-32 | 「逐行盘点一张表：表的每一行都」(`.claude/agents/sweep.md:44`) | 「逐行盘点一张表：表的每一行都」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:51`) | 原话基本在 100% |
| sw-33 | 「不改任何被搜到的文件，不改 」(`.claude/agents/sweep.md:48`) | 「不改任何被搜到的文件，不改 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:55`) | 原话基本在 100% |
| sw-34 | 「所有种类的报告都单列一节」(`.claude/agents/sweep.md:52`) | 「所有种类的报告都单列一节」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:59`) | 改写 57% |
| sw-35 | 「主 agent 把」(`.claude/agents/sweep.md:52`) | 「钉值类的照 .claude/」(`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:78`) | 改写 67% |
| sw-36 | 「改数的：每类搜索的命令与计数」(`.claude/agents/sweep.md:53`) | 「改数的：每类搜索的命令与计数」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:60`) | 原话基本在 100% |
| sw-37 | 「撤回结论与新立判据的：搜索或」(`.claude/agents/sweep.md:54`) | 「撤回结论与新立判据的：搜索或」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:61`) | 原话基本在 100% |
| sw-38 | 「阶段同步写事实表的：事实表、」(`.claude/agents/sweep.md:55`) | 「阶段同步写事实表的：事实表、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:62`) | 原话基本在 100% |
| sw-39 | 「逐行盘点的：第 12 步那张」(`.claude/agents/sweep.md:56`) | 「逐行盘点的：第 12 步那张」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:63`) | 原话基本在 100% |
| sw-40 | 「阶段同步逐行判的：」(`.claude/agents/sweep.md:57`) | 「阶段同步逐行判的：」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:64`) | 原话基本在 100% |
| sw-41 | 「按式子推不出的派生（手算后硬」(`.claude/agents/sweep.md:61`) | 「按式子推不出的派生（手算后硬」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:68`) | 原话基本在 100% |
| sw-42 | 「阶段同步的候选表只罩事实表写」(`.claude/agents/sweep.md:62`) | 「阶段同步的候选表只罩事实表写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/sweep.md:69`) | 原话基本在 100% |

### 定义 `.claude/agents/three-way-attack.md`（36 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| at-01 | 「假设主 agent 的倾向是」(`.claude/agents/three-way-attack.md:16`) | 「假设主 agent 的倾向是」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:16`) | 原话基本在 100% |
| at-02 | 「开工先读：.claude/r」(`.claude/agents/three-way-attack.md:17`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:17`) | 改写 51%（低分行逐条人工核过） |
| at-03 | 「轮名、背景材料路径、分给这条」(`.claude/agents/three-way-attack.md:21`) | 「轮名、背景材料路径、分给这条」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:21`) | 原话基本在 100% |
| at-04 | 「前几轮的判决路径（第一轮写」(`.claude/agents/three-way-attack.md:22`) | 「前几轮的判决路径（第一轮写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:22`) | 原话基本在 100% |
| at-05 | 「报告路径（形态 resear」(`.claude/agents/three-way-attack.md:23`) | 「报告路径（形态 resear」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:23`) | 原话基本在 100% |
| at-06 | 「只攻分到的攻击面，不重复前几」(`.claude/agents/three-way-attack.md:28`) | 「只攻分到的攻击面，不重复前几」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:28`) | 原话基本在 100% |
| at-07 | 「每个打中给出具体的历史或输入」(`.claude/agents/three-way-attack.md:29`) | 「每个打中给出具体的历史或输入」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:29`) | 原话基本在 100% |
| at-08 | 「由用户决定的动作（回退之后先」(`.claude/agents/three-way-attack.md:30`) | 「由用户决定的动作（回退之后先」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:30`) | 原话基本在 70% |
| at-09 | 「造历史、写模型时这样取样（模」(`.claude/agents/three-way-attack.md:31`) | 「造历史、写模型时这样取样（编」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:31`) | 改写 47%（低分行逐条人工核过） |
| at-10 | 「盘用内存稀疏盘（crates」(`.claude/agents/three-way-attack.md:32`) | 「盘用内存稀疏盘（crates」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:32`) | 原话基本在 100% |
| at-11 | 「模型先跑一小段、打出按它估的」(`.claude/agents/three-way-attack.md:33`) | 「模型先跑一小段、打出按它估的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:33`) | 原话基本在 75% |
| at-12 | 「崩溃状态不在缩的范围里（共用」(`.claude/agents/three-way-attack.md:34`) | 「崩溃状态不缩（共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:34`) | 改写 60% |
| at-13 | 「在自己的原型里这样调枚举函数」(`.claude/agents/three-way-attack.md:35`) | 「调枚举函数的原型只写 car」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:35`) | 改写 21%（低分行逐条人工核过） |
| at-14 | 「随机跑批分小批：每批的段数按」(`.claude/agents/three-way-attack.md:36`) | 「随机跑批分小批：每批的段数按」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:36`) | 原话基本在 100% |
| at-15 | 「编译与跑的命令照共用约束」(`.claude/agents/three-way-attack.md:38`) | 「编译与跑（cargo build 同样）」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:38`) | 人工指定 |
| at-16 | 「范围声明写在 submit 」(`.claude/agents/three-way-attack.md:38`) | 「命令怎么写：范围声明写在 submit 」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`) | 人工指定 |
| at-17 | 「主 agent 跑完新派同立」(`.claude/agents/three-way-attack.md:38`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| at-18 | 「装置与判法里起样本进程（真 」(`.claude/agents/three-way-attack.md:39`) | 「起样本进程照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:39`) | 人工指定；用户定-6：整组信号改为逐个按进程号停 |
| at-19 | 「命令行里停进程照旧一次点名一」(`.claude/agents/three-way-attack.md:39`) | 「命令行里停进程照旧一次点名一」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`) | 原话基本在 76% |
| at-20 | 「自己 Bash 里直接跑的一」(`.claude/agents/three-way-attack.md:40`) | 「自己 Bash 里直接跑的一」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:40`) | 改写 22%（低分行逐条人工核过） |
| at-21 | 「最后单独一个不带参数的 wa」(`.claude/agents/three-way-attack.md:40`) | 「要并行就每件写成 { <命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:78`) | 改写 52%（低分行逐条人工核过） |
| at-22 | 「对得上再按起的次序逐个读（共」(`.claude/agents/three-way-attack.md:40`) | 「④ 的写法收退出码」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:40`) | 人工指定 |
| at-23 | 「不用 wait "$pid"」(`.claude/agents/three-way-attack.md:40`) | 「不用 wait "$pid"」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:78`) | 改写 69% |
| at-24 | 「不改正在跑的脚本，要改的写同」(`.claude/agents/three-way-attack.md:41`) | 「不改正在跑的脚本，要改的照共」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:41`) | 改写 63% |
| at-25 | 「打中之后先答四句：分不分辨臂」(`.claude/agents/three-way-attack.md:42`) | 「打中之后先答四句：分不分辨臂」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:42`) | 原话基本在 100% |
| at-26 | 「自己提的改法写明」(`.claude/agents/three-way-attack.md:43`) | 「自己提的改法写明」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:43`) | 原话基本在 100% |
| at-27 | 「引文同正推腿：写进报告之前在」(`.claude/agents/three-way-attack.md:44`) | 「每一句引文（kb 条款、代码」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:44`) | 改写 39%（低分行逐条人工核过） |
| at-28 | 「没打中的写试过哪些形状、取样」(`.claude/agents/three-way-attack.md:45`) | 「没打中的写试过哪些形状、取样」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:45`) | 原话基本在 100% |
| at-29 | 「写模型的，报告开头给复跑命令」(`.claude/agents/three-way-attack.md:46`) | 「写模型的，报告开头给复跑命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:46`) | 原话基本在 93% |
| at-30 | 「写了模型的，<模型目录>/t」(`.claude/agents/three-way-attack.md:47`) | 「写了模型的，<模型目录>/t」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:47`) | 改写 45%（低分行逐条人工核过） |
| at-31 | 「报告文件、模型目录、草稿目录」(`.claude/agents/three-way-attack.md:51`) | 「报告文件、模型目录、草稿目录」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:51`) | 原话基本在 91% |
| at-32 | 「要改代码试的，把仓拷到草稿目」(`.claude/agents/three-way-attack.md:51`) | 「要改代码试的，照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:51`) | 人工指定；建副本命令写法在共用约束「建仓副本」 |
| at-33 | 「副本与自己的模型的编译、跑照」(`.claude/agents/three-way-attack.md:51`) | 「点名测试目标写 SINGLE」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:60`) | 原话基本在 73% |
| at-34 | 「调枚举函数的原型照第 3b 」(`.claude/agents/three-way-attack.md:51`) | 「调枚举函数的原型照第 3b 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:51`) | 原话基本在 100% |
| at-35 | 「报告开头一张」(`.claude/agents/three-way-attack.md:55`) | 「报告开头一张」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:55`) | 原话基本在 100% |
| at-36 | 「不替主 agent 采纳；」(`.claude/agents/three-way-attack.md:59`) | 「不替主 agent 采纳；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:59`) | 原话基本在 100% |

### 定义 `.claude/agents/three-way-defense.md`（17 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| df-01 | 「逐条复核已有判决够不够得着、」(`.claude/agents/three-way-defense.md:16`) | 「逐条复核已有判决够不够得着、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:16`) | 原话基本在 100% |
| df-02 | 「开工先读：.claude/r」(`.claude/agents/three-way-defense.md:17`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:17`) | 原话基本在 100% |
| df-03 | 「轮名、背景材料路径、被复核的」(`.claude/agents/three-way-defense.md:21`) | 「轮名、背景材料路径、被复核的」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:21`) | 原话基本在 100% |
| df-04 | 「要辩护的一方与要特别回答的问」(`.claude/agents/three-way-defense.md:22`) | 「要辩护的一方与要特别回答的问」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:22`) | 原话基本在 100% |
| df-05 | 「报告路径（形态 resear」(`.claude/agents/three-way-defense.md:23`) | 「报告路径（形态 resear」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:23`) | 原话基本在 100% |
| df-06 | 「对判决里每一条打中：它引的原」(`.claude/agents/three-way-defense.md:28`) | 「对判决里每一条打中：它引的原」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:28`) | 改写 56% |
| df-07 | 「找得到同类先例的，贴出处；」(`.claude/agents/three-way-defense.md:29`) | 「找得到同类先例的，贴出处；」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:29`) | 原话基本在 100% |
| df-08 | 「每条结论写」(`.claude/agents/three-way-defense.md:30`) | 「每条结论写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:30`) | 原话基本在 100% |
| df-09 | 「写模型的，报告开头给复跑命令」(`.claude/agents/three-way-defense.md:31`) | 「写模型的，报告开头给复跑命令」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:31`) | 改写 45%（低分行逐条人工核过） |
| df-10 | 「主 agent 跑出之前读数」(`.claude/agents/three-way-defense.md:31`) | 「跑出之前读数写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:31`) | 人工指定；「承接」半句在共用约束 |
| df-11 | 「装置与判法里起样本进程（真 」(`.claude/agents/three-way-defense.md:31`) | 「起样本进程照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:31`) | 人工指定；用户定-6 |
| df-12 | 「命令行里停进程照旧一次点名一」(`.claude/agents/three-way-defense.md:31`) | 「命令行里停进程照旧一次点名一」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`) | 原话基本在 76% |
| df-13 | 「写了模型的，写进报告的复跑命」(`.claude/agents/three-way-defense.md:32`) | 「写了模型的，写进报告的复跑命」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:32`) | 改写 45%（低分行逐条人工核过） |
| df-14 | 「交回前跑 python3 s」(`.claude/agents/three-way-defense.md:33`) | 「交回前跑 python3 s」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:33`) | 原话基本在 71% |
| df-15 | 「报告文件、模型目录、草稿目录」(`.claude/agents/three-way-defense.md:37`) | 「报告文件、模型目录、草稿目录」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:37`) | 原话基本在 91% |
| df-16 | 「一张表：判决条目 / 辩方判」(`.claude/agents/three-way-defense.md:41`) | 「一张表：判决条目 / 辩方判」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:41`) | 原话基本在 100% |
| df-17 | 「不替主 agent 撤销或维」(`.claude/agents/three-way-defense.md:45`) | 「不替主 agent 撤销或维」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-defense.md:45`) | 原话基本在 100% |

### 定义 `.claude/agents/three-way-forward.md`（18 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| fw-01 | 「从已定条款与 crates/」(`.claude/agents/three-way-forward.md:16`) | 「从已定条款与 crates/」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:16`) | 原话基本在 100% |
| fw-02 | 「开工先读：.claude/r」(`.claude/agents/three-way-forward.md:17`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:17`) | 原话基本在 100% |
| fw-03 | 「轮名、背景材料路径（正文 +」(`.claude/agents/three-way-forward.md:21`) | 「轮名、背景材料路径（正文 +」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:21`) | 原话基本在 100% |
| fw-04 | 「分给这条腿的判据格与问题编号」(`.claude/agents/three-way-forward.md:22`) | 「分给这条腿的判据格与问题编号」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:22`) | 原话基本在 100% |
| fw-05 | 「报告路径（形态 resear」(`.claude/agents/three-way-forward.md:23`) | 「报告路径（形态 resear」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:23`) | 原话基本在 100% |
| fw-06 | 「这一轮的禁读清单：别的腿这一」(`.claude/agents/three-way-forward.md:24`) | 「这一轮的禁读清单：别的腿这一」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:24`) | 原话基本在 100% |
| fw-07 | 「附录不够时读 kb 原文与 」(`.claude/agents/three-way-forward.md:28`) | 「附录不够时读 kb 原文与 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:28`) | 原话基本在 100% |
| fw-08 | 「每一格各报各的判定（一致 /」(`.claude/agents/three-way-forward.md:29`) | 「每一格各报各的判定（一致 /」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:29`) | 改写 56% |
| fw-09 | 「能用只读命令（grep、读文」(`.claude/agents/three-way-forward.md:30`) | 「能用只读命令（grep、读文」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:30`) | 改写 50%（低分行逐条人工核过） |
| fw-10 | 「主 agent 跑完新派同立」(`.claude/agents/three-way-forward.md:30`) | 「新派同类 agent 时，派发提示写一行」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:57`) | 人工指定；承接写法在共用约束 |
| fw-11 | 「只能由主 agent 实测、」(`.claude/agents/three-way-forward.md:30`) | 「命令也写不出、只能由主 ag」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:30`) | 改写 67% |
| fw-12 | 「装置与判法里起样本进程（真 」(`.claude/agents/three-way-forward.md:30`) | 「起样本进程照共用约束」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:30`) | 人工指定；用户定-6 |
| fw-13 | 「命令行里停进程照旧一次点名一」(`.claude/agents/three-way-forward.md:30`) | 「命令行里停进程照旧一次点名一」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:67`) | 原话基本在 76% |
| fw-14 | 「每条结论写」(`.claude/agents/three-way-forward.md:31`) | 「每条结论写」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:31`) | 原话基本在 100% |
| fw-15 | 「写了模型的，<模型目录>/t」(`.claude/agents/three-way-forward.md:32`) | 「写了模型的，<模型目录>/t」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:32`) | 改写 45%（低分行逐条人工核过） |
| fw-16 | 「报告文件、模型目录、草稿目录」(`.claude/agents/three-way-forward.md:36`) | 「报告文件、模型目录、草稿目录」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:36`) | 原话基本在 91% |
| fw-17 | 「报告按问题编号分节，末尾一张」(`.claude/agents/three-way-forward.md:40`) | 「报告按问题编号分节，末尾一张」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:40`) | 原话基本在 100% |
| fw-18 | 「不判别的腿的格，不替主 ag」(`.claude/agents/three-way-forward.md:44`) | 「不判别的腿的格，不替主 ag」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:44`) | 原话基本在 100% |

### 定义 `.claude/agents/three-way-materials.md`（18 行）

| 编号 | 原件 | 合稿树 | 备注 |
|---|---|---|---|
| mat-01 | 「正文（问题、判据、跑前条款、」(`.claude/agents/three-way-materials.md:15`) | 「正文（问题、判据、跑前条款、」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:15`) | 原话基本在 100% |
| mat-02 | 「开工先读：.claude/r」(`.claude/agents/three-way-materials.md:16`) | 「开工先读：.claude/r」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:16`) | 原话基本在 98% |
| mat-03 | 「.claude/rules/」(`.claude/agents/three-way-materials.md:16`) | 「它定了没，和它到底怎么定义的，是两个问题」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:16`) | 人工指定；指针换文件 |
| mat-04 | 「轮名、正文路径（形态 res」(`.claude/agents/three-way-materials.md:20`) | 「轮名、正文路径（形态 res」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:20`) | 原话基本在 100% |
| mat-05 | 「要进清单的文件列表（主 ag」(`.claude/agents/three-way-materials.md:21`) | 「要进清单的文件列表（主 ag」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:21`) | 原话基本在 100% |
| mat-06 | 「代码轮另给 diff 的范围」(`.claude/agents/three-way-materials.md:22`) | 「代码轮另给 diff 的范围」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:22`) | 原话基本在 84% |
| mat-07 | 「正文里提到的每个 kb 文件」(`.claude/agents/three-way-materials.md:26`) | 「正文里提到的每个 kb 文件」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:26`) | 原话基本在 100% |
| mat-08 | 「抄 / 不抄 / 理由」(`.claude/agents/three-way-materials.md:27`) | 「抄 / 不抄 / 理由」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:27`) | 原话基本在 100% |
| mat-09 | 「用 python3 scri」(`.claude/agents/three-way-materials.md:28`) | 「用 python3 scri」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:28`) | 原话基本在 100% |
| mat-10 | 「代码轮先写 _<轮>-dif」(`.claude/agents/three-way-materials.md:29`) | 「代码轮先写 _<轮>-dif」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:29`) | 改写 50%（低分行逐条人工核过） |
| mat-11 | 「拼背景材料：正文 + 清单 」(`.claude/agents/three-way-materials.md:30`) | 「拼背景材料：正文 + 清单 」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:30`) | 原话基本在 100% |
| mat-12 | 「写开工快照：清单里每个 kb」(`.claude/agents/three-way-materials.md:31`) | 「写开工快照：清单里每个 kb」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:31`) | 原话基本在 100% |
| mat-13 | 「的每一行与理由留在清单文件里」(`.claude/agents/three-way-materials.md:32`) | 「的每一行与理由留在清单文件里」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:32`) | 原话基本在 100% |
| mat-14 | 「research/promp」(`.claude/agents/three-way-materials.md:36`) | 「research/promp」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:36`) | 原话基本在 100% |
| mat-15 | 「回复：三份文件路径（代码轮四」(`.claude/agents/three-way-materials.md:40`) | 「回复：三份文件路径（代码轮四」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:40`) | 原话基本在 100% |
| mat-16 | 「回复不超过 4000 字（交」(`.claude/agents/three-way-materials.md:40`) | 「交回只写结论、报告路径与 sha256s」(`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:121`) | 人工指定；「回复」与「交回」同一件事（推的） |
| mat-17 | 「机械抽取保证」(`.claude/agents/three-way-materials.md:44`) | 「机械抽取保证」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:44`) | 原话基本在 100% |
| mat-18 | 「正文里没有 kb 路径、也没」(`.claude/agents/three-way-materials.md:45`) | 「正文里没有 kb 路径、也没」(`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:45`) | 原话基本在 100% |

## 找不到落点的（单列）

| 条 | 原件 | 我找过哪几份 | 结果 |
|---|---|---|---|
| 1 | 「拿它支撑新结论之前先重新跑一遍」(`.claude/agent-common.md:35`) | 合稿 agent-common.md 全文、rules/evidence-discipline.md、rules/three-way-inference.md、rules/implementation-workflow.md、skills/three-way/SKILL.md、skills/dispatch/SKILL.md、main-agent.md；`grep -rn '支撑新结论\|先复跑一遍\|重新跑一遍'` 扫整棵合稿树（除三份总表）0 行 | 没有落点原句，只有两跳指针（格一命中 1） |
| 2 | 「交回与报告在不影响正确性的前提下写简练」(`.claude/agent-common.md:119`) 的前提 | 合稿 agent-common.md「报告」一节、main-agent.md、rules/writing-discipline-by-review.md；`grep -rn '不影响正确性'` 扫合稿树 | 只在 「在不影响正确性的前提下写简练」(`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:82`)（派发提示），报告一节没有（格一弱 1） |
| 3 | 「主 agent 点崩溃枚举用例与崩溃放量分项」(`.claude/agents/implementation-writer.md:41`)（说明谁来看这一节，不是判据） | 合稿 implementation-writer.md、agent-common.md、skills/merge-verify、skills/dispatch、main-agent.md | 没有；它是消费方说明，不影响做法，不算丢 |
| 4 | 括注类：「写范围闸对你只放行」(`.claude/agents/implementation-writer.md:27`)、「写范围闸对你放行的仓外位置只有那里」(`.claude/agents/experiment-runner.md:24`)、「写范围闸对你放行」(`.claude/agents/kb-scribe.md:26`) | 各自合稿定义的「写范围」一节（写范围本身在） | 括注删了，写范围一节在，不算丢 |
| 5 | 「那张表的形状、四种关系、回看怎么写都在那里，这份定义不抄第二份」(`.claude/agents/experiment-runner.md:16`) | 合稿 experiment-runner.md 开工先读行；rules/format-evolution.md 那一节 | 说明句删了，读取指令在，不算丢 |

另：共用约束里钩子「怎么认」的细节（子 agent 不需要照做的识别条款，如 `cd` 之后相对路径怎么算、`env -S` 怎么拆、写者锁怎么对回执）合稿删了、指到文件头，这是用户定-7 的做法，不在这里逐条列。

## 逐条过到的文件与没过到的

逐条过到（15 份，行数是原件 `wc -l`，量的）：

| 文件 | 原件行数 | 落点表行数 |
|---|---|---|
| `.claude/agent-common.md` | 129 | 175 |
| `.claude/agents/implementation-writer.md` | 64 | 见各节标题 |
| `.claude/agents/experiment-runner.md` | 59 | 同上 |
| `.claude/agents/experiment-designer.md` | 69 | 同上 |
| `.claude/agents/tooling-writer.md` | 50 | 同上 |
| `.claude/agents/kb-scribe.md` | 53 | 同上 |
| `.claude/agents/investigator.md` | 46 | 同上 |
| `.claude/agents/clerk.md`、`mutation-triage.md`、`prior-art.md`、`sweep.md` | 40、45、40、62 | 同上 |
| `.claude/agents/three-way-attack.md`、`three-way-defense.md`、`three-way-forward.md`、`three-way-materials.md` | 59、45、44、45 | 同上 |

没逐条过到的：这 15 份里没有。我没做的是：合稿树里别的 36 份文件（规则、技能、main-agent、CLAUDE-merged）没逐条过，只在找落点、核指针时读到相关行；钩子文件头只核了 ⑥ ⑨ ⑪ ⑯ ⑰ ⑱ 与写闸第六、十道。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一 丢失 | 成立（1 处边界 + 1 处弱） | 「想拿旧数据支撑新结论先复跑」只剩两跳指针、触发条件被换；报告一节丢了「不影响正确性」前提；其余 175 + 407 行都有落点 |
| 二 二义 | 成立（2 处，继承自原件） | 读文件射程（共用约束第二、三条）；实现员命名检查 `<根>` 副本 vs 快照 |
| 三 常驻够不够 | 不成立 | 下沉的三类都留了入口与合规写法 |
| 四 子 agent 读得到 | 不成立（实质） | 121 个指针目标，只有 5 个主 agent 一侧文件字面不读，不影响子 agent 的动作 |
| 五 悄悄改了做法 | 不成立（核过的范围） | 做法差都在总表或用户定案里；钩子 ⑥ 与用户定-6 的配套、项目代码闸等 4 段话的逐句核对没做（推不出） |
| 用户定案 1 到 16 | 落对 | 本组无载体的写「无载体」 |
| 用户定案 17、18、19 | 待落 | 17 在 implementation-workflow、three-way-inference；18 在 main-agent；19 在调查员与实现员定义 |

## 要主 agent 跑的命令

无。我没起任何要经任务服务的命令；报告里的数都是只读命令（`sha256sum -c`、`grep`、`wc`）和草稿目录里只读脚本数出来的（量的），「钩子 ⑥ 与脚本循环」那一条是读文件头推的（推的）。

## 没做什么

- 不替主 agent 采纳、不出判决；不判别的腿（攻方）的格：二义的「找得出两种读法」我只写了摆在面上的 2 处，没替攻方去攻。
- 没往统一任务服务交任务，没起模型，判决要引的数都是只读量的，不需要复跑。
- 没读禁读清单里的文件与目录；没碰别的几题（entry、verification、residents、contradictions）各条腿的产出。
- 没跑钩子与门禁，没核 `crates/`；合稿树里别的文件只读了找落点要的那几行。
- 钩子描述句：共用约束里项目代码闸（`agent-common.md` 项目代码闸那一段）、范围声明闸、交回正文闸、只管主 agent 的三道，我没逐句对文件头（格五推不出）。

## 辅助脚本（草稿目录里，只读仓与合稿树）

脚本与 sha256 见草稿目录 `helper-sha256.txt`；用法：`python3 -I <脚本>`。它们不是模型，没有复跑命令要交主 agent。
