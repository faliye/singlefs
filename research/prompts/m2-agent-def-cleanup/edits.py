# 定义清理：每条是（文件、旧串、新串、经过在哪）。旧串必须恰好命中一次（replace-once.py 语义）。
P = "records/2026-09-16-subagent拆分提案.md"
X = "records/2026-09-17-已分配口径三方与两个实验.md"
A = ".claude/agents/"
EDITS = [
# ---------- 「依据：」一行改成读取指令 ----------
(A+"crash-verifier.md", "\n依据：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`", "\n开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`", "（标签改名，无经过）"),
(A+"experiment-designer.md", "\n依据：`.claude/singlefs-ai-sop/rules/test-discipline.md`「实验开跑之前", "\n开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md`「实验开跑之前", "（标签改名，无经过）"),
(A+"experiment-runner.md", "\n依据：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇", "\n开工先读：`.claude/singlefs-ai-sop/rules/test-discipline.md` 全篇", "（标签改名，无经过）"),
(A+"gate-triage.md", "\n依据：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`", "\n开工先读：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`", "（标签改名，无经过）"),
(A+"implementation-writer.md", "\n依据：`.claude/singlefs-ai-sop/rules/show-me-test.md`", "\n开工先读：`.claude/singlefs-ai-sop/rules/show-me-test.md`", "（标签改名，无经过）"),
(A+"kb-scribe.md", "\n依据：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`", "\n开工先读：`.claude/singlefs-ai-sop/skills/decide/SKILL.md`", "（标签改名，无经过）"),
(A+"mutation-triage.md", "\n依据：`.claude/rules/mutation-sampling.md` 全篇", "\n开工先读：`.claude/rules/mutation-sampling.md` 全篇", "（标签改名，无经过）"),
(A+"prior-art.md", "\n依据：`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做", "\n开工先读：`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做", "（标签改名，无经过）"),
(A+"sweep.md", "\n依据：`.claude/rules/format-evolution.md`", "\n开工先读：`.claude/rules/format-evolution.md`", "（标签改名，无经过）"),
(A+"three-way-attack.md", "\n依据：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」「判决由主 agent 做", "\n开工先读：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」「判决由主 agent 做", "（标签改名，无经过；行内日期是规则小节名的一部分）"),
(A+"three-way-defense.md", "\n依据：`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令）」（辩方腿推翻命中靠的是去查同类先例，不是重新论证利弊）。", "\n开工先读：`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算（2026-09-06 用户明令）」。", "括注的理由在被读的那一节正文里（「辩方腿也买到了东西」一句）；做法已在第 2 步"),
(A+"three-way-forward.md", "\n依据：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」；", "\n开工先读：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」；", "（标签改名，无经过）"),
(A+"three-way-local-attack.md", "\n依据：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」「一条腿只抽一次样", "\n开工先读：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」「一条腿只抽一次样", "（标签改名，无经过）"),
(A+"three-way-local-defense.md", "\n依据：同 `.claude/agents/three-way-local-attack.md` 的「依据」一行。", "\n开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。", "（标签改名，无经过）"),
(A+"three-way-materials.md", "\n依据：`.claude/rules/three-way-inference.md`「引 kb 里的条目要整行抄", "\n开工先读：`.claude/rules/three-way-inference.md`「引 kb 里的条目要整行抄", "（标签改名，无经过）"),
(A+"three-way-verifier.md", "\n依据：`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」；", "\n开工先读：`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」；", "（标签改名，无经过）"),
# ---------- 共用约束 ----------
(".claude/agent-common.md", "计划与来历在 `records/2026-09-16-subagent拆分提案.md`。这些定义 2026-09-17 写成；试跑、对抗与真活上的使用到哪一步，只记在那份计划里，不在这里写第二份。",
 "这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。门禁 71 号判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。",
 P+" 第二十一节第五批（开头不写进度，只指到计划）"),
(".claude/agent-common.md", "\n2026-09-17 两次试点（同一件活派给继承与不继承的两个定义）产出逐项一致；起步上下文从约 11 万 token 降到约 1 万（`records/2026-09-16-subagent拆分提案.md` 第二十一节第三批）。\n", "\n", P+" 第二十一节第三批"),
(".claude/agent-common.md", "- 定义「依据」一行点名的规则，只读点名的那几个小节：", "- 定义「开工先读：」一行点名的规则，只读点名的那几个小节：", "（跟着「依据：」改名，无经过）"),
(".claude/agent-common.md", "  不整份读：读进来的规则每次调用都跟着上下文走，第二次试点整份读，平均上下文只比继承时少约 21%。", "  不整份读。", P+" 第二十一节第三批（第二次试点整份读，平均上下文 11.2 万对 14.1 万）"),
(".claude/agent-common.md", "- 每个定义都照守、不再写进各自「依据」的三处：", "- 每个定义都照守、不再写进各自「开工先读：」一行的三处：", "（跟着「依据：」改名，无经过）"),
(".claude/agent-common.md", "那条完成通知把你叫醒、续上缓存（子 agent 的提示缓存是 5 分钟档，超过 5 分钟不调用模型就整份重写，2026-09-17 实测一段执行员 6 次、每次 36–59 万 token）。", "那条完成通知把你叫醒。", X+" 第五节「缓存过期后整份重写」一行；"+P+" 第二十一节第六批"),
(".claude/agent-common.md", "不起计时器更省：一次叫醒约 2–3 次调用、折 4 万基础输入，一次整份重写约 14 万（2026-09-18 A/B 实测）。", "不起计时器。", P+" 第二十一节第六批「单次成本」"),
(".claude/agent-common.md", "结不结束、怎么处置由主 agent 定（2026-09-17 用户定：子 agent 与脚本不强制结束）。", "结不结束、怎么处置由主 agent 定。", P+" 第二十一节第三批 hook 一行；"+X+" 第七节第 4 行"),
(".claude/agent-common.md", "先确认那一行真会写进那个文件（2026-09-17 一个执行员把 `echo \"exit=$?\"` 打到标准输出、却在日志里等 `exit=`，空转到被主 agent 停掉）；", "先确认那一行真会写进那个文件；", P+" 第二十一节第三批第一行"),
(".claude/agent-common.md", "- 交回与报告在不影响正确性的前提下写简练（用户 2026-09-17 定）：", "- 交回与报告在不影响正确性的前提下写简练：", P+" 第二十七节（新写）"),
(".claude/agent-common.md", "不从 `sed -n 'A,Bp'` 的输出里数偏移（2026-09-17 一轮里三条腿四处差 1，方向全一样）；", "不从 `sed -n 'A,Bp'` 的输出里数偏移；", P+" 第二十七节（新写）"),
(".claude/agent-common.md", "（整份报告塞进一次交回会撞单次输出上限，`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）", "（照 `.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」办）", P+" 第十九节「报告放进交回内容」一行"),
(".claude/agent-common.md", "主 agent 按路径存档（2026-09-17 两个实现员一个写了文件、一个按「放进交回」没写，主 agent 存不了档）。", "主 agent 按路径存档。", P+" 第二十一节第四批第二行"),
# ---------- main-agent ----------
(".claude/main-agent.md", "agent 的结论句一律当线索：同一个 agent 两次交回、数不一样是实际发生过的（2026-09-18 一轮里四次），以仓里的产物为准", "agent 的结论句一律当线索：以仓里的产物为准", P+" 第二十六节三"),
# ---------- gate-triage ----------
(A+"gate-triage.md", "跑全量工作区时「工作区跑的过程中没变」那一条在别的会话同时改仓时多半红（2026-09-17 试跑整轮 2 小时 13 分，工作区从 128 项变到 224 项）：照抄开跑、收尾两个指纹", "别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹", P+" 第十七节 gate-triage 一行（2 小时 13 分）；128 → 224 项在第二十七节（新写）"),
# ---------- kb-scribe ----------
(A+"kb-scribe.md", "数出来贴原样，不手数（2026-09-17 两次试跑都把 31、32 写成 30）。", "数出来贴原样，不手数。", P+" 第十七节、第十八节 kb-scribe 两行"),
(A+"kb-scribe.md", "贴末行：doc-lint 是共享 SOP 的阶段，不在阶段归属表里，登记给你的阶段一条都不管它，而照规格写进去的句子最常撞的就是它（裸编号、正文里写历史措辞、没登记的标签）；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子（2026-09-17 实测：照规格写回之后阶段全绿交回，doc-lint 在写进去的段落上红 7 处）。", "贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。", P+" 第二十七节（新写）"),
# ---------- experiment-designer ----------
(A+"experiment-designer.md", "当分段挂钟（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」；2026-09-17 E152 第二个事务的挂钟因此混进串口打印积压，10 轮 9 轮小于子进程自己计的数）。", "当分段挂钟（照 `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」一节办）。", ".claude/kb/vm-harness.md 历史版本；"+P+" 历史版本「计时的坑写进定义」"),
(A+"experiment-designer.md", "后面的段不跑。（2026-09-17 E153 登记九族历史 × 12 格 × 5 臂 × 崩溃支线，一次做不完就分段一路补，派了七次，后几段的数不改变任何岔路。）", "后面的段不跑。", P+" 第二十一节第二批；"+X+" 第四节（执行员派了七次）、第六节"),
# ---------- experiment-runner ----------
(A+"experiment-runner.md", "（两个路径都相对 `research/`；在仓根下跑时仓根那份 `Cargo.toml` 也有 `[workspace]`，脚本的目录检查照样放过，接着编不出 bin，报成「基线就是红的」，2026-09-17 试跑实测。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；bin 名写错时同样报成「基线就是红的」）", "（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）", P+" 第十八节 experiment-runner 一行与「顺带发现」；第十七节 mutation-triage 一行"),
(A+"experiment-runner.md", "状态写「部分已跑」，不写「已跑」（2026-09-17 E153：停机条款 S1 没跑、九族历史只跑了三族，状态却写成「已跑」，主 agent 事后改）。", "状态写「部分已跑」，不写「已跑」。", P+" 第二十一节第二批第一行"),
(A+"experiment-runner.md", "对某条候选不利的那一半同样要写（2026-09-17 E153 第三段：报了 G12 在写失败族上零红，没报它在回退族上 36 格全红；两次回退那两族 24 行没有臂字段也没发现，是主 agent 读产物时数出来的）。", "对某条候选不利的那一半同样要写。", P+" 第二十一节第二批第三行；「36 格」在第二十七节（新写）"),
(A+"experiment-runner.md", "确认它们真是 0（2026-09-17 E153 第五段：报告写「8 族 `sampled=0`、H6/H7 因被抛弃根很快被轮转覆写而为 0」，产物里 H6、H7 每行都有采样，为 0 的是另外 6 族；实验页同一句也漏了 H6、H7，而同一句里的「360 行」按它自己写的四族只有 240 行）。", "确认它们真是 0。", P+" 第二十一节第二批第五行"),
(A+"experiment-runner.md", "与 87 号（复跑全部已入库实验）与这个实验大小无关、合起来十几分钟，2026-09-17 起不归你，", "与 87 号（复跑全部已入库实验）不归你，", P+" 第二十一节第三批「15、87 号」一行"),
# ---------- crash-verifier ----------
(A+"crash-verifier.md", "的原样输出作旁证（`env.sh` 不查这三样）。这三条命令不单独下判断：本机 herd7 不在 PATH 里，`.claude/scripts/lkmm.sh` 找不到时自己加载 `opam env` 再找（2026-09-17 试跑：`command -v herd7` 退出码 1，57 号照样通过）。", "的原样输出作旁证。这三条命令不单独下判断。", P+" 第十七节 crash-verifier 一行（env.sh 不查三样）、第十八节 crash-verifier 一行（herd7 退出码 1、lkmm.sh 自己加载 opam env）"),
(A+"crash-verifier.md", "6. 别的会话常在同时改 `crates/`：每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里（分两次调用，中间几秒的改动会落进空窗，2026-09-17 试跑撞上过）：", "6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：", P+" 第十七节 crash-verifier 一行（14 → 18 个文件）、第十八节 crash-verifier 一行（指纹空窗）"),
# ---------- three-way-forward / defense ----------
(A+"three-way-forward.md", "不把自己的读法写成条款原文（2026-09-17 实测：一句「有效只按实例表判」被写成出自 D23（journal 的角色与格式），全文没有这句；另一份报告把背景材料代码块里的相对行号当成主仓行号）。", "不把自己的读法写成条款原文。", P+" 第二十七节（新写）"),
(A+"three-way-defense.md", "不把自己的读法写成条款原文（2026-09-17 实测：一句「有效只按实例表判」被写成出自 D23（journal 的角色与格式），全文没有这句；另一份报告把背景材料代码块里的相对行号当成主仓行号）。", "不把自己的读法写成条款原文。", P+" 第二十七节（新写）"),
# ---------- local-attack / local-defense ----------
(A+"three-way-local-attack.md", "1. 把攻击面写成英文提示：自足（本地模型读不到附录）、不用任何", "1. 把攻击面写成英文提示：自足、不用任何", ".claude/rules/three-way-inference.md「给本地腿的提示一律用英文」（本地腿读不到中文附录）"),
(A+"three-way-local-attack.md", "不许只答 yes / no（2026-09-17 第一轮只答 yes / no 的两份样本没有一格可核）。", "不许只答 yes / no。", P+" 第二十一节「两个新角色」表本地攻辩一行"),
(A+"three-way-local-attack.md", "一律标「模型自给、未核」（2026-09-17 核查员抽查两份样本里模型自给的行号，两处全错）。", "一律标「模型自给、未核」。", P+" 第二十七节（新写）"),
(A+"three-way-local-attack.md", "写明为什么加（2026-09-17 一句多加的括注让一格作废）。", "写明为什么加。", P+" 第二十一节「两个新角色」表核查员一行"),
(A+"three-way-local-attack.md", "看它列出的生词：缺头的粘连词（例：`achievesceives`）闸抓不到（计划第十一节；2026-09-17 起闸认得自己尾部复读 `reachableachable`、`anewew` 与派生前缀 `observationalomputational`，别的形态照样可能漏），见到就把那份记成「带损坏」，不当干净样本；", "看它列出的生词：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；", P+" 第十一节第一行、第二十一节「这一次直接改了的」损坏闸一行"),
(A+"three-way-local-attack.md", "另外通读一遍样本：缺词、断句这类闸抓不到的损坏（", "另外通读一遍样本：缺词、断句这类损坏（", "（删的是说明，没有经过）"),
(A+"three-way-local-attack.md", "几份样本方向一不一致（2026-09-17 一份运行记录把方向相反的两份写成一致）。", "几份样本方向一不一致。", P+" 第十九节末段（核查员查出本地辩方运行记录）"),
(A+"three-way-local-defense.md", "- 实测（2026-09-16 第一轮）：辩方提示在本地模型上五次坏三次，其中两次闸判绿；样本更容易不够两份，不够就照实报。", "- 样本不够两份就照实报。", P+" 第二十七节（新写）"),
# ---------- three-way-verifier ----------
(A+"three-way-verifier.md", "代码轮必给）。腿交回之后主 agent 往往已经改了主树，行号对主树核会报一批假 ✗——2026-09-17 步 6 checker 那一轮，一格「24 条」判 ✗、还建议查材料，实为快照晚于主 agent 的改正；之后两轮给了快照，没有再出假 ✗。没给快照的代码轮", "代码轮必给）。没给快照的代码轮", P+" 第二十七节（新写）；.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」"),
(A+"three-way-verifier.md", "也有没有多加原文没有的限定词或括注（2026-09-17 alloc-basis-r3 本地攻方提示多加一句括注，核对表写成逐字照抄，据此出的反例作废）。", "也有没有多加原文没有的限定词或括注。", P+" 第二十一节「两个新角色」表核查员一行；.claude/rules/three-way-inference.md「给本地腿的提示一律用英文」"),
(A+"three-way-verifier.md", "报告只交在回复里不算交（2026-09-17 alloc-basis-r3 一份核查员报告这样交回，主 agent 事后转存）。", "报告只交在回复里不算交。", P+" 第二十一节「两个新角色」表核查员一行、「这一次直接改了的」核查员一行"),
# ---------- three-way-materials ----------
(A+"three-way-materials.md", "停下要全名，不猜（2026-09-17 实测：「名字含 unreadable 的用例」对得上两条）。", "停下要全名，不猜。", P+" 第二十七节（新写）"),
(A+"three-way-materials.md", "不手挑 `awk` 行区间（2026-09-17 手挑行区间每轮 15–19 分钟）。", "不手挑 `awk` 行区间。", P+" 第二十七节（新写）"),
(A+"three-way-materials.md", "照定义拼、回复里写明（2026-09-17 几轮代码轮的背景材料顺序各不相同，是主 agent 口头改的）。", "照定义拼、回复里写明。", P+" 第二十七节（新写）"),
# ---------- three-way-attack ----------
(A+"three-way-attack.md", "只固定前缀与故障（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节里 2026-09-16 那条）。", "只固定前缀与故障（照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」那一条办）。", ".claude/rules/three-way-inference.md 同一条（C143 第三轮的实测写在规则里）"),
(A+"three-way-attack.md", "或「推的」（按代码推、没实现没跑）——2026-09-17 实测：一张三个改法各修哪几格的表全是推的、没标，下一轮正推腿在副本上把其中一个改法实现两版，它说修掉的那一格仍然红。", "或「推的」（按代码推、没实现没跑）。", P+" 第二十七节（新写）"),
# ---------- implementation-writer ----------
(A+"implementation-writer.md", "在主工作区改（用户 2026-09-16 定）。", "在主工作区改。", P+" 第九节「implementation-writer 在哪改」一行"),
(A+"implementation-writer.md", "（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录：`rsync -a` 保留源码的旧修改时间，共用 target 时 cargo 会直接跑上一份副本编出的二进制，2026-09-17 实测源码与原件相同的副本报 8 过 2 红）", "（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）", P+" 第十八节 implementation-writer 一行与「这一轮新测到的事实」第三行"),
(A+"implementation-writer.md", "已经红的测试记成基线红集（多半是别的会话在制的改动），", "已经红的测试记成基线红集，", P+" 第十九节（实现员基线红集）"),
(A+"implementation-writer.md", "写之前重算、对不上就不写（2026-09-17 实测：判定与取号各读一遍超级块，两次瞬时读错让判定放行、取号写进新号之后才 panic）。", "写之前重算、对不上就不写。", P+" 第二十七节（新写）"),
(A+"implementation-writer.md", "\n   - 2026-09-17 实测：三处 `todo!` 落在可达路径上，其中一处在取号写进超级块之后才 panic，而改之前那一格是干净地报错返回；主 agent 让续做两次才改成写之前拒绝。\n", "\n", P+" 第二十七节（新写）"),
(A+"implementation-writer.md", "只加一条快用例钉住「相同」（2026-09-17 实测：一条新流与第一个事务那条逐项相同，全量等于把第一条流再跑一遍，是三方攻方腿量出来的）。", "只加一条快用例钉住「相同」。", P+" 第二十七节（新写）"),
]
