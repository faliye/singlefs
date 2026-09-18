| # | 位置（清理前行号） | 在用户那 36 行里 | 删掉或换掉的原文 | 换成 | 经过在哪 |
|---|---|---|---|---|---|
| 1 | `crash-verifier.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 2 | `experiment-designer.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 3 | `experiment-runner.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 4 | `gate-triage.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 5 | `implementation-writer.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 6 | `kb-scribe.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 7 | `mutation-triage.md:13` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 8 | `prior-art.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 9 | `sweep.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 10 | `three-way-attack.md:14` | 是 | 依据 | 开工先读 | （标签改名，无经过；行内日期是规则小节名的一部分） |
| 11 | `three-way-defense.md:14` | 是 | 依据：`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令）」（辩方腿推翻命中靠的是去查同类先例，不是重新论证利弊） | 开工先读：`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令）」 | 括注的理由在被读的那一节正文里（「辩方腿也买到了东西」一句）；做法已在第 2 步 |
| 12 | `three-way-forward.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 13 | `three-way-local-attack.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 14 | `three-way-local-defense.md:12` | 否 | 依据：同 `.claude/agents/three-way-local-attack.md` 的「依据」一行 | 开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节 | （标签改名，无经过） |
| 15 | `three-way-materials.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 16 | `three-way-verifier.md:14` | 否 | 依据 | 开工先读 | （标签改名，无经过） |
| 17 | `agent-common.md:4` | 否 | 计划与来历在 `records/2026-09-16-subagent拆分提案.md`。这些定义 2026-09-17 写成；试跑、对抗与真活上的使用到哪一步，只记在那份计划里，不在这里写第二份 | 这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。门禁 71 号判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」） | records/2026-09-16-subagent拆分提案.md 第二十一节第五批（开头不写进度，只指到计划） |
| 18 | `agent-common.md:15` | 否 | 2026-09-17 两次试点（同一件活派给继承与不继承的两个定义）产出逐项一致；起步上下文从约 11 万 token 降到约 1 万（`records/2026-09-16-subagent拆分提案.md` 第二十一节第三批）。 | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第三批 |
| 19 | `agent-common.md:17` | 否 | 依据 | 开工先读： | （跟着「依据：」改名，无经过） |
| 20 | `agent-common.md:18` | 否 | ：读进来的规则每次调用都跟着上下文走，第二次试点整份读，平均上下文只比继承时少约 21% | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第三批（第二次试点整份读，平均上下文 11.2 万对 14.1 万） |
| 21 | `agent-common.md:19` | 否 | 依据」 | 开工先读：」一行 | （跟着「依据：」改名，无经过） |
| 22 | `agent-common.md:42` | 是 | 、续上缓存（子 agent 的提示缓存是 5 分钟档，超过 5 分钟不调用模型就整份重写，2026-09-17 实测一段执行员 6 次、每次 36–59 万 token） | （删掉） | records/2026-09-17-已分配口径三方与两个实验.md 第五节「缓存过期后整份重写」一行；records/2026-09-16-subagent拆分提案.md 第二十一节第六批 |
| 23 | `agent-common.md:42` | 是 | 更省：一次叫醒约 2–3 次调用、折 4 万基础输入，一次整份重写约 14 万（2026-09-18 A/B 实测） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第六批「单次成本」 |
| 24 | `agent-common.md:43` | 是 | （2026-09-17 用户定：子 agent 与脚本不强制结束） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第三批 hook 一行；records/2026-09-17-已分配口径三方与两个实验.md 第七节第 4 行 |
| 25 | `agent-common.md:43` | 是 | （2026-09-17 一个执行员把 `echo "exit=$?"` 打到标准输出、却在日志里等 `exit=`，空转到被主 agent 停掉） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第三批第一行 |
| 26 | `agent-common.md:55` | 是 | （用户 2026-09-17 定） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 27 | `agent-common.md:56` | 是 | （2026-09-17 一轮里三条腿四处差 1，方向全一样） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 28 | `agent-common.md:61` | 是 | 整份报告塞进一次交回会撞单次输出上限，`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」 | 照 `.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」办 | records/2026-09-16-subagent拆分提案.md 第十九节「报告放进交回内容」一行 |
| 29 | `agent-common.md:61` | 是 | （2026-09-17 两个实现员一个写了文件、一个按「放进交回」没写，主 agent 存不了档） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第四批第二行 |
| 30 | `main-agent.md:27` | 是 | 同一个 agent 两次交回、数不一样是实际发生过的（2026-09-18 一轮里四次）， | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十六节三 |
| 31 | `gate-triage.md:27` | 是 | 跑全量工作区时「工作区跑的过程中没变」那一条在别的会话同时改仓时多半红（2026-09-17 试跑整轮 2 小时 13 分，工作区从 128 项变到 224 项） | 别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了 | records/2026-09-16-subagent拆分提案.md 第十七节 gate-triage 一行（2 小时 13 分）；128 → 224 项在第二十八节（新写） |
| 32 | `kb-scribe.md:29` | 是 | （2026-09-17 两次试跑都把 31、32 写成 30） | （删掉） | records/2026-09-16-subagent拆分提案.md 第十七节、第十八节 kb-scribe 两行 |
| 33 | `kb-scribe.md:29` | 是 | ：doc-lint 是共享 SOP 的阶段，不在阶段归属表里，登记给你的阶段一条都不管它，而照规格写进去的句子最常撞的就是它（裸编号、正文里写历史措辞、没登记的标签）；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子（2026-09-17 实测：照规格写回之后阶段全绿交回，doc-lint 在写进去的段落上红 7 处） | ；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子 | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 34 | `experiment-designer.md:29` | 否 | `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」；2026-09-17 E152 第二个事务的挂钟因此混进串口打印积压，10 轮 9 轮小于子进程自己计的数 | 照 `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」一节办 | .claude/kb/vm-harness.md 历史版本；records/2026-09-16-subagent拆分提案.md 历史版本「计时的坑写进定义」 |
| 35 | `experiment-designer.md:30` | 是 | （2026-09-17 E153 登记九族历史 × 12 格 × 5 臂 × 崩溃支线，一次做不完就分段一路补，派了七次，后几段的数不改变任何岔路。） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第二批；records/2026-09-17-已分配口径三方与两个实验.md 第四节（执行员派了七次）、第六节 |
| 36 | `experiment-runner.md:28` | 是 | 在仓根下跑时仓根那份 `Cargo.toml` 也有 `[workspace]`，脚本的目录检查照样放过，接着编不出 bin，报成「基线就是红的」，2026-09-17 试跑实测。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；bin 名写错时同样报成「基线就是红的」 | bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错 | records/2026-09-16-subagent拆分提案.md 第十八节 experiment-runner 一行与「顺带发现」；第十七节 mutation-triage 一行 |
| 37 | `experiment-runner.md:29` | 是 | （2026-09-17 E153：停机条款 S1 没跑、九族历史只跑了三族，状态却写成「已跑」，主 agent 事后改） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第二批第一行 |
| 38 | `experiment-runner.md:31` | 是 | （2026-09-17 E153 第三段：报了 G12 在写失败族上零红，没报它在回退族上 36 格全红；两次回退那两族 24 行没有臂字段也没发现，是主 agent 读产物时数出来的） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第二批第三行；「36 格」在第二十八节（新写） |
| 39 | `experiment-runner.md:31` | 是 | （2026-09-17 E153 第五段：报告写「8 族 `sampled=0`、H6/H7 因被抛弃根很快被轮转覆写而为 0」，产物里 H6、H7 每行都有采样，为 0 的是另外 6 族；实验页同一句也漏了 H6、H7，而同一句里的「360 行」按它自己写的四族只有 240 行） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第二批第五行 |
| 40 | `experiment-runner.md:33` | 是 | 与这个实验大小无关、合起来十几分钟，2026-09-17 起 | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节第三批「15、87 号」一行 |
| 41 | `crash-verifier.md:27` | 是 | （`env.sh` 不查这三样）。这三条命令不单独下判断：本机 herd7 不在 PATH 里，`.claude/scripts/lkmm.sh` 找不到时自己加载 `opam env` 再找（2026-09-17 试跑：`command -v herd7` 退出码 1，57 号照样通过） | 。这三条命令不单独下判断 | records/2026-09-16-subagent拆分提案.md 第十七节 crash-verifier 一行（env.sh 不查三样）、第十八节 crash-verifier 一行（herd7 退出码 1、lkmm.sh 自己加载 opam env） |
| 42 | `crash-verifier.md:28` | 是 | 别的会话常在同时改 `crates/`：每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里（分两次调用，中间几秒的改动会落进空窗，2026-09-17 试跑撞上过） | 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里 | records/2026-09-16-subagent拆分提案.md 第十七节 crash-verifier 一行（14 → 18 个文件）、第十八节 crash-verifier 一行（指纹空窗）；空窗的理由在第二十八节（新写） |
| 43 | `three-way-forward.md:26` | 是 | （2026-09-17 实测：一句「有效只按实例表判」被写成出自 D23（journal 的角色与格式），全文没有这句；另一份报告把背景材料代码块里的相对行号当成主仓行号） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 44 | `three-way-defense.md:25` | 是 | （2026-09-17 实测：一句「有效只按实例表判」被写成出自 D23（journal 的角色与格式），全文没有这句；另一份报告把背景材料代码块里的相对行号当成主仓行号） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 45 | `three-way-local-attack.md:25` | 是 | （本地模型读不到附录） | （删掉） | .claude/rules/three-way-inference.md「给本地腿的提示一律用英文」（本地腿读不到中文附录） |
| 46 | `three-way-local-attack.md:25` | 是 | （2026-09-17 第一轮只答 yes / no 的两份样本没有一格可核） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节「两个新角色」表本地攻辩一行 |
| 47 | `three-way-local-attack.md:25` | 是 | （2026-09-17 核查员抽查两份样本里模型自给的行号，两处全错） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 48 | `three-way-local-attack.md:26` | 是 | （2026-09-17 一句多加的括注让一格作废） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节「两个新角色」表核查员一行 |
| 49 | `three-way-local-attack.md:29` | 是 | 缺头的粘连词（例：`achievesceives`）闸抓不到（计划第十一节；2026-09-17 起闸认得自己尾部复读 `reachableachable`、`anewew` 与派生前缀 `observationalomputational`，别的形态照样可能漏），见到 | 见到缺头的粘连词（例：`achievesceives`） | records/2026-09-16-subagent拆分提案.md 第十一节第一行、第二十一节「这一次直接改了的」损坏闸一行 |
| 50 | `three-way-local-attack.md:29` | 是 | 闸抓不到的 | （删掉） | （删的是说明，没有经过） |
| 51 | `three-way-local-attack.md:38` | 是 | （2026-09-17 一份运行记录把方向相反的两份写成一致） | （删掉） | records/2026-09-16-subagent拆分提案.md 第十九节末段（核查员查出本地辩方运行记录） |
| 52 | `three-way-local-defense.md:24` | 是 | 实测（2026-09-16 第一轮）：辩方提示在本地模型上五次坏三次，其中两次闸判绿；样本更容易不够两份，不够 | 样本不够两份 | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 53 | `three-way-verifier.md:21` | 否 | 腿交回之后主 agent 往往已经改了主树，行号对主树核会报一批假 ✗——2026-09-17 步 6 checker 那一轮，一格「24 条」判 ✗、还建议查材料，实为快照晚于主 agent 的改正；之后两轮给了快照，没有再出假 ✗。 | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写）；.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」 |
| 54 | `three-way-verifier.md:30` | 是 | （2026-09-17 alloc-basis-r3 本地攻方提示多加一句括注，核对表写成逐字照抄，据此出的反例作废） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节「两个新角色」表核查员一行；.claude/rules/three-way-inference.md「给本地腿的提示一律用英文」 |
| 55 | `three-way-verifier.md:36` | 是 | （2026-09-17 alloc-basis-r3 一份核查员报告这样交回，主 agent 事后转存） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十一节「两个新角色」表核查员一行、「这一次直接改了的」核查员一行 |
| 56 | `three-way-materials.md:20` | 是 | （2026-09-17 实测：「名字含 unreadable 的用例」对得上两条） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 57 | `three-way-materials.md:28` | 是 | （2026-09-17 手挑行区间每轮 15–19 分钟） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 58 | `three-way-materials.md:29` | 是 | （2026-09-17 几轮代码轮的背景材料顺序各不相同，是主 agent 口头改的） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 59 | `three-way-attack.md:27` | 是 | `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节里 2026-09-16 那条 | 照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」那一条办 | .claude/rules/three-way-inference.md 同一条（C143 第三轮的实测写在规则里） |
| 60 | `three-way-attack.md:29` | 是 | ——2026-09-17 实测：一张三个改法各修哪几格的表全是推的、没标，下一轮正推腿在副本上把其中一个改法实现两版，它说修掉的那一格仍然红 | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 61 | `implementation-writer.md:13` | 是 | （用户 2026-09-16 定） | （删掉） | records/2026-09-16-subagent拆分提案.md 第九节「implementation-writer 在哪改」一行 |
| 62 | `implementation-writer.md:27` | 是 | ：`rsync -a` 保留源码的旧修改时间，共用 target 时 cargo 会直接跑上一份副本编出的二进制，2026-09-17 实测源码与原件相同的副本报 8 过 2 红 | （删掉） | records/2026-09-16-subagent拆分提案.md 第十八节 implementation-writer 一行与「这一轮新测到的事实」第三行 |
| 63 | `implementation-writer.md:27` | 是 | （多半是别的会话在制的改动） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写）；基线红集这一概念在第十九节 |
| 64 | `implementation-writer.md:31` | 是 | （2026-09-17 实测：判定与取号各读一遍超级块，两次瞬时读错让判定放行、取号写进新号之后才 panic） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 65 | `implementation-writer.md:33` | 是 | - 2026-09-17 实测：三处 `todo!` 落在可达路径上，其中一处在取号写进超级块之后才 panic，而改之前那一格是干净地报错返回；主 agent 让续做两次才改成写之前拒绝。 | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |
| 66 | `implementation-writer.md:35` | 是 | （2026-09-17 实测：一条新流与第一个事务那条逐项相同，全量等于把第一条流再跑一遍，是三方攻方腿量出来的） | （删掉） | records/2026-09-16-subagent拆分提案.md 第二十八节（新写） |

用户命令清理前 36 行，被这 66 处覆盖的：36 行（全集）
