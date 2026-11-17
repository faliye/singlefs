# rules-compress-residents-r1 正推腿报告

日期：2026-11-16。轮名 rules-compress-residents-r1，第一轮，立场：正推（逐条核落点）。背景材料：`research/prompts/_rules-compress-residents-r1-body.md`（正文就是全部材料）。

## 怎么做的

- 被判的三十五份原件每一份都整份读过，接收它的合稿文件也整份读过，逐条对。上游规则读了被指到的八份：`communication-discipline.md`、`code-discipline.md`、`show-me-test.md` 整份，`sop-first.md`、`kb-discipline.md`（第 30 行起）、`writing-discipline.md`、`design-doc-discipline.md`、`pushback-discipline.md`（头 11 行）。脚本文件头读了 `.claude/gate.d/lib-lag-baseline.py` 头 140 行；别的脚本文件头只 `grep` 了被指到的节名在不在。
- 开工快照两份清单都核过（量的，这一次跑的）：在仓根跑 `sha256sum -c research/prompts/rules-compress-residents-r1-snapshot/judged-sha256.txt`，35 行里不是 OK 的 0 行；在合稿树根跑 `sha256sum -c ../rules-compress-residents-r1-snapshot/merged-tree-sha256.txt`，54 行里不是 OK 的 0 行。
- 逐条落点表是半机械的：草稿目录里的 `landing.py` 给原件每一行在合稿树与上游规则里按字二元组重合挑一个候选落点，`overrides.txt` 是我逐条读完之后手工改派的 140 条（盖原件 163 行；改指、标「已登的改动」、标「删」、指到上游或脚本文件头），`gen.py` 把两样合成表，表里每格的原话是从被引的那一行里取的连续一段。表里 1065 行（量的，`gen.py` 的输出行 `rows 1065`）：自动落点 906、手工改派的落点 84、已登的改动 33、删 24、脚本文件头 11、上游 7（七行都是手工指的）。
- 表里不列的原件行：空行、节标题、表头与表的分隔行、围栏里的行（`fs-design.md` 两段 Rust 示意照用户定第 8 条删，`changelog-format.md` 的格式示例照留）、技能文件的 frontmatter。节标题改没改单列在「节标题的变动」。
- 自动落点靠得住到什么程度：候选落在接收文件之外的、重合分低于 0.45 的 15 行逐行看过；合稿那一侧公共片段短于 7 个字的 38 行逐行看过，改了 3 行；其余自动落点没有逐行再核第二遍，按我逐条读的印象没有见到错行，这一句是推的。
- 这一份没写模型，没有要复跑的数。辅助脚本只读仓、只写草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r1-sonnet/`，sha256（量的）：
  - `landing.py` dede2767cb16472bbe5546d56208c934507d96018019bb9c551f83789cd3ef09
  - `gen.py` f7781d6cd1426bc91c45982b6d7412cbfc80c836f287dfec3f89815762ed4181
  - `pointers.py` d4bd9344b0d0937967662c4be5a2a94d94073728a28ec98e4e060645a12d1b41
  - `overrides.txt` 5a551ae45cc87ef0c3068c4506b8731d2882be0e03fd9cb802e375eda7e91566

表里「类」一列的六种：

| 类 | 意思 |
|---|---|
| 落点 | 这一条在合稿树里的那一行 |
| 上游 | 合稿树里只留指针，话在上游规则的那一行（指到的那一节我核过确有这句） |
| 脚本文件头 | 合稿树里只留指针，话在脚本文件头的那一行 |
| 已登的改动 | 合稿改了这一条的做法或指针，矛盾总表或用户定案里登着，备注写编号 |
| 删 | 没有落点，删的是开头声明、引导句、说明句或文件内指针，不带判据 |

路径写法：合稿树的文件写全路径 `research/prompts/rules-compress-r1-merged/dot-claude/…`，上游写 `.claude/singlefs-ai-sop/rules/…`，原件写 `.claude/…`。

## 一、逐条落点表

次序照派发提示：八份被并掉的小文件；`evidence-discipline`、`session-wrapup`、`command-safety-by-review`、`format-evolution`、`fs-design`、`show-me-test-by-review`、`writing-discipline-by-review`；两份提交技能；其余规则与技能。

### `.claude/rules/verify-before-claiming.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/verify-before-claiming.md:4`：「外面现在是什么样」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:24`：「外面现在是什么样」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:5`：「记忆、kb、上一轮对话、TO」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:24`：「记忆、kb、上一轮对话、TODO 表」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:7`：「指的是：不在这次对话里、而且」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:24`：「可能在我不知道的时候被改过的一切」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:11`：「门禁现在过不过」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:28`：「引最近一次 gate.sh --staged 的」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:12`：「某个不变量实现了没」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:29`：「读 crates/singlefs-checke」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:13`：「某个决策定了没」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:30`：「kb/decisions.md」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:14`：「工具链 / 环境齐不齐」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:31`：「跑 .claude/singlefs-ai-so」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:15`：「别家实现怎么做的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:32`：「现查它的文档或源码，」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:16`：「被测对象或它的产物现在什么状」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:33`：「被测对象或它的产物现在什么状态」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:18`：「不用现查的：这次对话里刚读过」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:24`：「不用现查的：这次对话里刚读过的文件、纯代码逻辑推」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:22`：「查了状态列、确认某条决策是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:41`：「知道它定的是什么。」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:23`：「拿一条决策去做任何推导（建模」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:41`：「拿一条决策去做任何推导（建模、写检查、推翻别的结」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:24`：「贴不出来，就别拿它推导。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:41`：「就别拿它推导。」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:28`：「判据：说出口的那句话，它的主」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:45`：「判据：说出口的那句话，」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:30`：「这条路不通」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:45`：「这条路不通」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:31`：「这个文件里没写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:45`：「这个文件里没写」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:33`：「做法：把要说的那句宽的话当成」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:47`：「话当成待证命题，问」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:34`：「要让它为真，需要排除掉哪些情」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:47`：「要让它为真，需要排除掉哪些情况」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:35`：「排不完就把话缩回你真排完的那」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:47`：「排不完就把话缩回」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:37`：「一个对象有没有被条款罩着」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:48`：「之前，拿它的名字全仓搜一遍，」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:38`：「看它有没有被归进某个已有的类」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:48`：「看它有没有被归进某个已有的类。」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:40`：「截断过的输出也是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:49`：「head -12 看到的是前 12 行」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:41`：「只有这几处」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:49`：「wc -l），数对上了再看内容。」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:45`：「交出去、还没跑完的检查（测试」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:53`：「交出去、还没跑完的检查（测试、门禁、复跑、自证）」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:46`：「结局出来、读过判定的那一行，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:53`：「结局出来、读过判定的那一行，再改成结论」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:47`：「几件检查写在同一句里的，没出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:54`：「几件检查写在同一句里的，没出结局的那一件单独拆出」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:48`：「被取消、撞了资源上限或被打断」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:55`：「被取消、撞了资源上限或被打断的那一次运行，日志里」 | 落点 | 「引的时候引那一行」半句只在第 53 行有 |
| `.claude/rules/verify-before-claiming.md:52`：「要改 kb 或 TODO 里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:35`：「改 kb 或 TODO 里任何一行状态，先跑那一」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:53`：「我是靠哪条命令知道的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:36`：「我是靠哪条命令知道的」 | 落点 |  |
| `.claude/rules/verify-before-claiming.md:54`：「别人提出的事实性更正同样要核」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:37`：「不照单全收，也不急着反驳，核完把结果说清」 | 落点 |  |

### `.claude/rules/artifact-line-pointer.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/artifact-line-pointer.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条）；错指针删，登在矛盾总表常驻-2 |
| `.claude/rules/artifact-line-pointer.md:7`：「产物归档出工作区之后，正文里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:143`：「- 产物归档出工作区之后，正文里那一行写成一行指」 | 落点 |  |
| `.claude/rules/artifact-line-pointer.md:8`：「指到的那一行要以登记的产物行」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:144`：「以登记的产物行前缀起头（形态 <前缀> name」 | 落点 |  |
| `.claude/rules/artifact-line-pointer.md:9`：「前缀登记在项目根的 .cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:144`：「.claude/product-line-pre」 | 落点 |  |
| `.claude/rules/artifact-line-pointer.md:13`：「按指针 git show 取」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:145`：「按指针 git show 取那一行，核它存在、且」 | 落点 |  |

### `.claude/rules/exhaustive-interleaving.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/exhaustive-interleaving.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条）；错指针删，登在常驻-3 |
| `.claude/rules/exhaustive-interleaving.md:7`：「这个屏障够不够」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:105`：「这类判定拆成两半：」 | 落点 |  |
| `.claude/rules/exhaustive-interleaving.md:11`：「在给定的形式模型下穷举所有允」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:109`：「在给定的形式模型下穷举所有允许的交错，给出判定」 | 落点 |  |
| `.claude/rules/exhaustive-interleaving.md:12`：「把代码里的同步模式翻成形式模」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:110`：「把代码里的同步模式翻成形式模型、枚举该问哪些场景」 | 落点 |  |
| `.claude/rules/exhaustive-interleaving.md:14`：「口径：精确的是工具，不是模型」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:112`：「写下的和代码今天做的是不是一回事」 | 落点 |  |
| `.claude/rules/exhaustive-interleaving.md:16`：「用哪个工具、怎么证明它的判定」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:112`：「.claude/rules/verificati」 | 已登的改动 | 主 agent 处置-6：后两样记欠账 |
| `.claude/rules/exhaustive-interleaving.md:20`：「穷尽性能被机器检查」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:113`：「宁可写穷举的显式分支」 | 已登的改动 | 常驻-4：指针写全名 |
| `.claude/rules/exhaustive-interleaving.md:26`：「用来穷举交错的工具，判定和真」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:114`：「，那个工具降为参考，不再当门禁」 | 落点 |  |

### `.claude/rules/service-stop-readback.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/service-stop-readback.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条）；错指针删，登在常驻-1 |
| `.claude/rules/service-stop-readback.md:7`：「systemctl stop」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:47`：「读 systemctl is-active 服务」 | 落点 |  |
| `.claude/rules/service-stop-readback.md:8`：「再读一遍它的 cgroup.」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:48`：「它的 cgroup.procs（目录是 /sys」 | 落点 |  |
| `.claude/rules/service-stop-readback.md:9`：「不只看 stop 的退出码，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:47`：「它只在 active 时返回 0，activat」 | 落点 |  |
| `.claude/rules/service-stop-readback.md:10`：「停之前判服务在不在，同样看输」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:49`：「同样看输出：active、activating、」 | 落点 |  |

### `.claude/rules/communication-discipline-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/communication-discipline-by-review.md:4`：「管的是：agent 对用户、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:56`：「在 .claude/singlefs-ai-so」 | 落点 |  |
| `.claude/rules/communication-discipline-by-review.md:8`：「先说结论或结果，再说依据。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:60`：「- 先说结论或结果，再说依据」 | 落点 |  |
| `.claude/rules/communication-discipline-by-review.md:9`：「不为照顾对方的情绪软化判断。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:61`：「错了就说错在哪，对不上就说对不上哪一条（.cla」 | 落点 |  |
| `.claude/rules/communication-discipline-by-review.md:10`：「不复述对方刚说过的话。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:62`：「- 不复述对方刚说过的话。」 | 落点 |  |
| `.claude/rules/communication-discipline-by-review.md:11`：「不确定就写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:63`：「- 不确定就写」 | 落点 |  |
| `.claude/rules/communication-discipline-by-review.md:12`：「问用户拍板之前，问句里说范围」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:64`：「核了窄的那一句，说出口的却是宽的那一句」 | 落点 |  |
| `.claude/rules/communication-discipline-by-review.md:16`：「写给第三方读的产出（邮件、公」 | `.claude/singlefs-ai-sop/rules/communication-discipline.md:5`：「写给第三方读的产出（邮件、公告、对外文档）按它的」 | 上游 | 合稿第 56 行的指针写的节名「射程与例外」上游没有，话在上游开头一段 |
| `.claude/rules/communication-discipline-by-review.md:17`：「精确不让位于简短：术语、数字」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:65`：「- 精确不让位于简短：术语、数字、口径、命令、下」 | 落点 |  |

### `.claude/rules/engineering-philosophy.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/engineering-philosophy.md:4`：「实现上 AI 友好，审核上人」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:4`：「实现面与审核面是两条轴，各自优化」 | 落点 | 原话并进合稿标题（第 2 行）与第 4 行 |
| `.claude/rules/engineering-philosophy.md:6`：「两者是分开的两条轴，各自优化」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:4`：「两条轴，各自优化，不在中间折中。」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:12`：「模型写、模型改、机器验」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:10`：「模型写、模型改、机器验」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:13`：「优化什么」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:11`：「可机检、可穷举、可强制进入、可回退」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:14`：「具体形态」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:12`：「显式穷举分支胜过聪明的通用路径；类型让非法状态无」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:18`：「对模型友好的方向是更显式，不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:17`：「对模型友好的方向是更显式，不是更晦涩。」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:20`：「放宽的只有一样：为了让人脑装」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:17`：「一个机制包打天下」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:21`：「这类设计上的统一性要求。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:17`：「统一性要求」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:23`：「命名、函数职责单一、路径数有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:19`：「命名、函数职责单一、路径数有限、约定一致」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:27`：「语义写进名字，不省」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:23`：「语义写进名字，不省」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:28`：「函数职责单一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:24`：「一个函数做一件能单独验证的事」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:29`：「路径数有限」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:25`：「说得出全覆盖这段代码的控制流要多少个用例，而且有」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:30`：「约定一致」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:26`：「约定保持一致」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:32`：「判据是 .claude/ru」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:60`：「让代码更容易被机械验证」 | 落点 | 区间第 60 至 63 行 |
| `.claude/rules/engineering-philosophy.md:33`：「落到代码上怎么写，细则在 .」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:28`：「在 .claude/singlefs-ai-so」 | 落点 | 名字三条在上游 code-discipline.md 第 22 至 24 行 |
| `.claude/rules/engineering-philosophy.md:34`：「层数不限而路径数要有限；语义」 | `.claude/singlefs-ai-sop/rules/code-discipline.md:78`：「进名字，」 | 上游 | 层数不限、路径数有限在合稿 code-discipline-by-review.md 第 16 行 |
| `.claude/rules/engineering-philosophy.md:38`：「人的注意力不花在读代码上，读」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:32`：「人的注意力不花在读代码上，读代码交给模型」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:40`：「这个测试测的是不是对的东西」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:34`：「1. 这个测试测的是不是对的东西」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:41`：「这条不变量本身对不对」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:35`：「2. 这条不变量本身对不对」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:42`：「这个决策的依据成不成立」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:36`：「3. 这个决策的依据成不成立」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:43`：「要不要接受这个取舍」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:37`：「4. 要不要接受这个取舍」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:45`：「门禁的输出、kb 的结论、决」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:39`：「门禁的输出、kb 的结论、决策记录、失败信息都」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:46`：「代码写得难读可以接受，这四样」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:39`：「难读可以接受，这四样」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:50`：「Gate proves ev」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:43`：「> Gate proves evidence r」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:52`：「机器证明证据够不够，人判断语」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:45`：「机器证明证据够不够，人判断语义对不对」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:58`：「.claude/rules/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:13`：「、.claude/singlefs-ai-sop」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:59`：「.claude/single」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:13`：「、.claude/singlefs-ai-sop」 | 落点 |  |
| `.claude/rules/engineering-philosophy.md:60`：「.claude/rules/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:45`：「.claude/rules/show-me-te」 | 落点 |  |

### `.claude/rules/design-doc-discipline-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/design-doc-discipline-by-review.md:6`：「判据与尺子都在 .claud」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:101`：「在 .claude/rules/writing-」 | 落点 |  |
| `.claude/rules/design-doc-discipline-by-review.md:10`：「设计文档要回答两件事：为什么」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:102`：「设计文档要回答」 | 落点 |  |
| `.claude/rules/design-doc-discipline-by-review.md:11`：「为这两件事写的铺垫、例子、摊」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:102`：「为这两件事写的铺垫、例子、摊开讲的取舍」 | 落点 |  |
| `.claude/rules/design-doc-discipline-by-review.md:13`：「判据：删掉这一段，读的人还会」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:102`：「判据：删掉这一段，读的人还」 | 落点 |  |
| `.claude/rules/design-doc-discipline-by-review.md:14`：「都还行就可以删；不行就留。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:102`：「不行就留。」 | 落点 |  |

### `.claude/rules/pushback-discipline-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/pushback-discipline-by-review.md:4`：「用户提的方案，和别处来的方案」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:106`：「和别处来的方案过同一道闸」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:6`：「.claude/rules/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:106`：「.claude/rules/show-me-te」 | 已登的改动 | 按需-M8：节名改成「不按来源分类，只看证据」 |
| `.claude/rules/pushback-discipline-by-review.md:8`：「警告记录的格式在 .clau」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:106`：「/rules/pushback-discipli」 | 已登的改动 | 按需-M9：改指上游 pushback-discipline.md |
| `.claude/rules/pushback-discipline-by-review.md:12`：「提议和实测数据、或者已经核实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:110`：「的事实对不上时，能多早说就多早说，三样都」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:16`：「对不上的是哪一条：哪个实验、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:114`：「对不上的是哪一条：哪个实验、哪个数、哪条决策，带」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:17`：「按提议做会出什么问题，什么现」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:115`：「按提议做会出什么问题，什么现象说明它出现了」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:18`：「有没有能同时满足两边的第三条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:116`：「有没有能同时满足两边的第三条路，没有就说没有」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:20`：「对不上不等于对方错。 报的是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:118`：「报的是哪个观测和它对不上，不是」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:21`：「对方给了理由就去核（.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:118`：「对方给了理由就去核（.claude/rules/」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:25`：「做过的工作不是留着它的理由。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:122`：「做到一半发现做不到、做完了才发现前提不成立、」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:26`：「当初调查得不够就动了手：说出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:122`：「说出来，该撤就整个撤回。」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:28`：「已经做了这么多」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:122`：「是报给对方的一条信息，不写进技术依据。」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:30`：「这一条只管已经做出来的工作。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:124`：「git checkout、rm -rf 这些」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:31`：「以及磁盘格式、协议、公开 A」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:124`：「随手做（.claude/singlefs-ai-」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:35`：「我核过，对不上」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:128`：「我记得好像对不上」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:36`：「（.claude/rules」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:128`：「：说不出靠哪条命令知道的，就写成推测）」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:38`：「核不动的就说核不动，不把没查」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:128`：「说核不动，不把没查过的疑虑说成事实。」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:42`：「反对过一次、对方明确重申，那」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:132`：「就是对方的决定：照做，不再劝，也不」 | 落点 |  |
| `.claude/rules/pushback-discipline-by-review.md:44`：「新证据是例外。 后来跑出来的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:132`：「再报一次（.claude/rules/evide」 | 落点 |  |

### `.claude/rules/evidence-discipline.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/evidence-discipline.md:4`：「任何结论说出口之前，这三步都」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:4`：「结论说出口之前」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:5`：「少一步就写成假设，不许写成结」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:4`：「少一步就写成假设，不」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:9`：「从机制、代码或数据推到结论」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:8`：「哪条命令、哪几行代码、哪个数据文件」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:10`：「假设这个结论是错的，那我本该」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:9`：「说得出一个具体的、这一轮做得到的观测，以及它确实」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:11`：「换一条独立的路子，把同一个判」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:10`：「两条路子不共用同一段代码、同一次采样、同一个工具」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:15`：「往被测对象里塞一个已知的、本」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:14`：「本该被校验那条路子抓到的」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:16`：「抓不到，之前所有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:14`：「抓不到，之前所有」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:18`：「独立计数只对上了总数，印证不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:16`：「拿一个计数去印证一个拆法之前，先」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:19`：「拿一个计数去印证一个拆法之前」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:16`：「、写明拆法来自哪里，并把另一个原始计数也对一遍。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:23`：「找几个独立来源交叉验证一个推」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:20`：「发给各方的」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:24`：「某某实现是这样的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:20`：「某某实现是这样的」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:28`：「发给模型的提示、跑出来的原始」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:59`：「发给模型的提示、跑出来的原始输出、当时的日志，在」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:29`：「改提示只能连同重跑一起改。要」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:59`：「改提示只能连同重跑一起改」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:31`：「冻结只在这一轮内有效，出了轮」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:60`：「证据（提示、腿的报告、原始输出、留存产物）」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:32`：「这一轮提交之后，下一次提交把」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:60`：「这一轮提交之后，下一次提交把上一次留下的那批」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:33`：「删之前查一遍被检查读的登记表」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:60`：「删之前查一遍被检查读的登记表（门禁登记表、钉表、」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:35`：「旧证据是参考，不是依据。 对」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:61`：「（.claude/singlefs-ai-sop」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:36`：「要推翻早先的结论，按项目自己」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:61`：「重走一轮，不拿旧文件对质。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:38`：「正文引产物的检查按」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:62`：「正文引产物的检查按」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:39`：「早先抄下的行当历史参考，不再」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:62`：「早先抄下的行当历史参考」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:41`：「门禁也得绕开这批目录。 .c」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:63`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:42`：「一行一条，每条都要写明为什么」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:63`：「指向不存在的目录、或」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:46`：「任何进入结论的速率、容量、比」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:67`：「结论的速率、容量、比例，先问」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:47`：「量不到的（比如企业盘的 fs」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:67`：「按 10⁵/秒是 89 年，那一档本机测不到，仍」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:49`：「转述给别的会话或别的 age」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:68`：「，并写明是哪个量（哪个文件、哪份产物、源文件还是」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:50`：「收到的一方把它当线索：现查过」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:68`：「收到的一方把它当线索：现查过才记进记录，没标的按」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:54`：「别的项目怎么实现，不能直接当」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:72`：「外部实现只用在两处：提出假设、指」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:55`：「外部实现只能用在两个地方：提」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:72`：「：提出假设、指出该测哪条路径。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:56`：「不许进正推、反推、校验这三步」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:72`：「进正推、反推、校验」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:57`：「想拿它当证据，先在这个项目里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:72`：「想拿它当证据，先在这个项目里跑出能观测的数据，」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:59`：「分清哪一半可以引：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:74`：「不可以引」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:63`：「XLOG_CONTINUE_」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:76`：「XLOG_CONTINUE_TRANS 这个常量」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:64`：「反证：某个方案在现役实现里从」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:77`：「方案在现役实现里出现过，不等于它适合这个项目」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:65`：「机制：把别家的做法拆成机制，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:78`：「搬运：把结论连同它没写出来的前提一起搬过来」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:67`：「没有任何现役实现走某条路」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:80`：「没有任何现役实现走某条路」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:69`：「引用别家实现的时候，必须同时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:81`：「同时写下它和这个项目的一处已知差异」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:71`：「引用时要标来源，并注明」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:32`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:75`：「写下假设之后，先问一句」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:85`：「答不上来就别拿去建实验。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:77`：「排除一个假设，要么拿出能推翻」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:85`：「排除一个假设，要么拿出能推翻它的观测，要么明说」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:78`：「我想不出别的解释」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:85`：「我想不出别的解释」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:82`：「写下结论之前先自问这六条，答」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:89`：「，答不上来就是还没做完：」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:86`：「这个模型是不是先于我的倾向就」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:93`：「这个模型是不是先于我的倾向就有了」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:87`：「我有没有给另一边也认真建一条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:94`：「说得出对面那条臂最好该是什么样，以及」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:88`：「结果反过来我接不接受」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:95`：「要是量出相反的数，我会怎么改结论」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:89`：「我是不是只在一个点上取了值」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:96`：「一个尺寸、一个参数、一个负载算出来的倍数，不能叫」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:90`：「我给的那个区间，同一份产物里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:97`：「说得出这个区间覆盖了产物里的全部取样点」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:91`：「这个数是不是某个没扫过的参数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:98`：「这个数是不是某个没扫过的参数的函数」 | 落点 | 「先问它是哪个参数的函数，再写进正文」半句没有落点 |
| `.claude/rules/evidence-discipline.md:93`：「推翻一个旧结论时，要比确立时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:100`：「推翻一个旧结论时，要比确立时更严，不是更松。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:95`：「稻草人对照臂：给对面那条路挑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:101`：「稻草人对照臂：给对面那条路挑一个没人会真用的实现」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:96`：「判据：对面那条臂的样子，支持」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:101`：「判据：对面那条臂的样子，支持那条路的人认不认。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:98`：「引用某条臂的数，要连它的定义」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:41`：「连它的定义一起带上」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:99`：「（.claude/rules」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:41`：「确认一条决策」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:103`：「跑前写死的不只是判据、阈值和」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:105`：「判据、阈值和作废条款，还有每条臂到底是什么。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:104`：「失败条款打中一条写得含糊的臂」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:105`：「失败条款打中一条写得含糊的臂时，不许不记一次输就」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:106`：「合法的出路只有三步，缺一步就」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:105`：「三步，缺一步就算新实验」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:110`：「记一次输」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:109`：「按最弱读法把失败条款的判决原样写下」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:111`：「只许收严」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:110`：「新形态要被原来那套判据判得更严或相同」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:112`：「写明收严在哪」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:111`：「说得出新形态比旧形态多要求了什么」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:114`：「臂的定义里同时写了」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:113`：「，先证明前一句推得出后一句」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:115`：「推不出来就删掉」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:113`：「把做法改成推得出它的那一种。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:117`：「跑产物之前修订臂或判据，是合」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:114`：「单元测试跑完、产物一次没跑时补一条更强的臂或一条」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:118`：「在跑前登记里写明改了什么、依」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:114`：「哪个单元测试读数、时点在产物之前」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:124`：「打中不分辨臂」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:120`：「这种条款时，同时写一格让所有臂一起出局时怎么记」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:125`：「判别子观测不到」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:121`：「判据要被判的系统区分两种情形，而」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:126`：「打中归错了判据」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:122`：「，而发生的事按字面属于门槛不同的另一条判据」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:127`：「改法碰不到打中的格」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:123`：「修的是哪一格；打中之后先在打中的格上逐个核改法还」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:129`：「打中之后先问四句：它分不分辨」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:125`：「打中之后先问四句：它分不分辨臂」 | 落点 |  |

`.claude/rules/evidence-discipline.md` 续（从原件第 131 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/evidence-discipline.md:133`：「引产物整行抄，带上文件名；概」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:129`：「引产物整行抄，带上文件名；概括只说自己算过的那一」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:137`：「产物重跑一次，正文里引它的每」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:133`：「复跑全绿不构成」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:139`：「同理管可以被一条命令数出来的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:134`：「<个数> 个单元测试」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:140`：「能被一条命令数出来的数，就由」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:134`：「能被一条命令数出来的」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:142`：「正文与产物对不上还有一种形态」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:135`：「限定词没了」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:143`：「判据：一条结论说」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:135`：「只有某条臂才买得到 / 某条臂独有 / 只有某条」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:144`：「去产物里看那条臂之外的每一条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:135`：「每一条臂在每一个参数格上的值；」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:145`：「正文那张表里没有那个参数的列」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:135`：「正文那张表里没有那个参数的列，」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:149`：「报扫描出来的极值（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:139`：「报扫描出来的极值（」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:150`：「是端点，就写成」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:139`：「≤ 端点（扫描下沿，边界没量到）」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:154`：「立完判据先问：它对已经在册的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:149`：「它对已经在册的条目判什么」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:158`：「对象换成被撤回或被改写的数值」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:153`：「按批做：新立的判据、撤回的数各记一行进项目的欠账」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:159`：「判据：撤回之后问一句」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:154`：「，答得出一份清单才算撤完」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:160`：「扫的范围是全仓，不是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:154`：「扫的范围是全仓」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:162`：「扫出来的清单不能一把全替换：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:155`：「扫出来的清单不」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:163`：「判据：把这句话里的旧值换成新」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:155`：「：把这句话里的旧值换成新值，」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:165`：「回扫要按旧说法搜，不能只按新」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:156`：「按旧说法搜，不」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:166`：「回扫之前先写出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:156`：「（没有 <这件事>、<这件事> 还没、只有 <别」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:167`：「这一阶段的 diff 里删掉」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:156`：「这一阶段的 diff 里删掉或改写的现状句，也逐」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:171`：「撤回的理由后来失效了，被撤回」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:160`：「回来，得重新论证。」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:173`：「发现撤回的理由失效时，按顺序」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:160`：「发现撤回的理由失效时」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:177`：「今天顶在这一格上的依据是哪一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:164`：「今天顶在这一格上的依据是哪一条」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:178`：「那条依据自己站不站得住」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:165`：「那条依据自己站不站得住」 | 落点 |  |
| `.claude/rules/evidence-discipline.md:179`：「要让被撤回的那条复活，重新论」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:166`：「要让被撤回的那条复活，重新论证做了没有」 | 落点 |  |

### `.claude/rules/session-wrapup.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/session-wrapup.md:6`：「一行三件事：现在在哪个里程碑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:8`：「一行三件事：现在在哪个里程碑 / 门禁过不过 /」 | 落点 |  |
| `.claude/rules/session-wrapup.md:7`：「门禁那一格引最近一次提交前门」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:8`：「门禁那一格」 | 落点 | 改成指 evidence-discipline.md 第 28 行那一格 |
| `.claude/rules/session-wrapup.md:8`：「里程碑与证据两格现查（读记录」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:8`：「里程碑与证据两格现查（读记录与产物），不从上一轮」 | 落点 |  |
| `.claude/rules/session-wrapup.md:9`：「用户用 halt 终止的那一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:4`：「用户用 halt 终止的那一轮」 | 落点 |  |
| `.claude/rules/session-wrapup.md:11`：「还要说清这一轮给这个数加了多」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:10`：「出了个零结论、推翻了一条决策」 | 落点 |  |
| `.claude/rules/session-wrapup.md:12`：「或者只是弄明白了某个证据现在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:10`：「只是弄明白了某个证据现在还拿不到，就写 0」 | 落点 |  |
| `.claude/rules/session-wrapup.md:16`：「会不会被抄第二遍」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:14`：「会不会被抄第二遍」 | 落点 |  |
| `.claude/rules/session-wrapup.md:18`：「踩过的坑要做成拒绝执行的检查」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:15`：「照 .claude/singlefs-ai-so」 | 落点 |  |
| `.claude/rules/session-wrapup.md:20`：「反过来也看一眼：有没有已经没」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:16`：「已经没人用的，」 | 落点 |  |
| `.claude/rules/session-wrapup.md:24`：「看看有没有哪一步是靠记性撑住」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:20`：「、同样的样板」 | 落点 |  |
| `.claude/rules/session-wrapup.md:25`：「有就当场改那个文件，别留到」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:20`：「就当场改那个文件，」 | 落点 |  |
| `.claude/rules/session-wrapup.md:27`：「改哪里，看这条管的是什么：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:20`：「看这条管」 | 落点 |  |
| `.claude/rules/session-wrapup.md:29`：「管协作（证据、文档、决策落点」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:22`：「改 singlefs-ai-sop，并抬 VER」 | 落点 |  |
| `.claude/rules/session-wrapup.md:30`：「（改哪些路径要抬版本，见 .」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:22`：「，见 .claude/singlefs-ai-s」 | 落点 |  |
| `.claude/rules/session-wrapup.md:31`：「管文件系统怎么设计 → 改项」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:23`：「kb/、.claude/rules/ 或项目 C」 | 落点 |  |
| `.claude/rules/session-wrapup.md:33`：「拿不准就放项目本地。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:24`：「别的项目会不会也需要」 | 落点 |  |
| `.claude/rules/session-wrapup.md:35`：「不写进会话私有的记忆（例如」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:26`：「Claude Code 的 memory」 | 落点 |  |
| `.claude/rules/session-wrapup.md:36`：「私有记忆只放用户个人的偏好（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:26`：「私有记忆只放用户个人的偏好（什么时候提交、用什么」 | 落点 |  |
| `.claude/rules/session-wrapup.md:40`：「这一轮如果推翻或者定下了任何」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:30`：「kb/decisions.md，带上依据。」 | 落点 |  |
| `.claude/rules/session-wrapup.md:42`：「掂量完给个说法，也算一次决策」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:30`：「这条现在不该定」 | 落点 |  |
| `.claude/rules/session-wrapup.md:43`：「如果这实际上推翻了 kb/d」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:30`：「kb/decisions.md 里已」 | 落点 |  |
| `.claude/rules/session-wrapup.md:44`：「同样要当场记进去，带上依据。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:30`：「，带上依据。」 | 落点 |  |
| `.claude/rules/session-wrapup.md:48`：「几个会话同时在一个仓里干活时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:34`：「几个会话同时在一个仓里干活时，收尾前逐条对一遍：」 | 落点 |  |
| `.claude/rules/session-wrapup.md:50`：「工作区里的都是我改的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:36`：「提交前把改动分成」 | 落点 |  |
| `.claude/rules/session-wrapup.md:51`：「只提交这一轮的。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:36`：「只提交这一轮的。」 | 落点 |  |
| `.claude/rules/session-wrapup.md:52`：「门禁红了就是我改坏了」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「判红之前先看失败涉及的文件在不在这一轮的改动里」 | 落点 |  |
| `.claude/rules/session-wrapup.md:53`：「不在就如实说」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「照 .claude/rules/command-」 | 落点 |  |
| `.claude/rules/session-wrapup.md:54`：「分不清的时候跑 bash .」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「在临时 worktree 上只拿 HEAD」 | 已登的改动 | 常驻-8：不再为分清谁的红另跑门禁 |
| `.claude/rules/session-wrapup.md:55`：「不带 --staged 跑时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「gate.sh 开跑与收尾各记一次工作区指纹，对」 | 落点 | 「等改动停下再跑，或者用 --staged」半句没有落点（随常驻-8） |
| `.claude/rules/session-wrapup.md:56`：「公共编号先到先得。 历史条目」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:38`：「实验号这类大家共用的编号，取号之前先看已有的最大」 | 落点 |  |
| `.claude/rules/session-wrapup.md:57`：「改公共文件只做定点替换，别整」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:39`：「改公共文件只做定点替换，」 | 落点 |  |
| `.claude/rules/session-wrapup.md:58`：「新建一个文件不会盖掉谁」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:39`：「改公共文件只做定点替换，不整篇重」 | 落点 |  |
| `.claude/rules/session-wrapup.md:59`：「新建文件用排他方式（set」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:39`：「新建文件用排他方式（set -o noclobb」 | 落点 | 区间第 38 至 39 行 |
| `.claude/rules/session-wrapup.md:60`：「能在工具层拦（写入前的钩子拒」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:39`：「能在工具层拦（写入前的钩子拒绝覆盖未跟踪的文件）」 | 落点 |  |
| `.claude/rules/session-wrapup.md:61`：「只提交这些路径」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:40`：「不等于 git commit -- <路径>」 | 落点 |  |
| `.claude/rules/session-wrapup.md:62`：「暂存前先确认 git dif」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:40`：「暂存前先确认 git diff --cached」 | 落点 |  |
| `.claude/rules/session-wrapup.md:63`：「暂存区里只有我的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:41`：「清索引（git reset、git restor」 | 落点 |  |
| `.claude/rules/session-wrapup.md:65`：「历史条目撞号由门禁阶段」 | `.claude/singlefs-ai-sop/rules/kb-discipline.md:87`：「scripts/history-ordinal.」 | 上游 | 常驻-19：合稿 session-wrapup.md 第 38 行留指针 |
| `.claude/rules/session-wrapup.md:66`：「项目用了它不认的编号形态，才」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:38`：「不认的编号形态，才在项目侧补一道会红的检查」 | 落点 |  |

### `.claude/rules/command-safety-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/command-safety-by-review.md:6`：「装置、判法与复跑脚本起样本进」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:6`：「与复跑脚本起样本进程（真 shell、被测程序、」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:8`：「每条样本放进它自己的进程组：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:10`：「start_new_session=True」 | 已登的改动 | 用户定第 6 条：不放进程组，逐个按进程号停 |
| `.claude/rules/command-safety-by-review.md:9`：「超时与结束时对整组发信号，先」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:9`：「不只停直接起的那一个」 | 已登的改动 | 用户定第 6 条：不对进程组发信号 |
| `.claude/rules/command-safety-by-review.md:10`：「跑完按进程组号在 /proc」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:10`：「停掉，并写进这条样本的失败信息。」 | 已登的改动 | 用户定第 6 条 |
| `.claude/rules/command-safety-by-review.md:11`：「只停直接子进程会留下孙进程：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:9`：「的一方永远等不到 EOF」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:13`：「这一条还没做成检查：起的是不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:12`：「，靠 review。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:17`：「长活可以等，但要有人看着：」 | 无 | 删 | 引导句删 |
| `.claude/rules/command-safety-by-review.md:19`：「起了就要复检。 后台起的任务」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:16`：「最后一次动作是多久以前、是不是停在等待循环里、」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:20`：「同一条命令是不是反复跑而输出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:16`：「同一条命令是不是反复跑而输出一模一样、它写的文件」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:21`：「检出和处置分开。 检出卡住与」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:17`：「检出卡住与空转的工具（hook、看门狗脚本）只记」 | 已登的改动 | 常驻-11：检出工具只报，不再许检出工具终止 |
| `.claude/rules/command-safety-by-review.md:22`：「结不结束、怎么处置，由调度的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:17`：「可以终止它：进程用 .claude/single」 | 已登的改动 | 常驻-11：检出工具只报，不再许检出工具终止 |
| `.claude/rules/command-safety-by-review.md:23`：「拒绝某一种危险写法的钩子不在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:17`：「拒绝某一种危险写法的钩子不在」 | 落点 | 指 session-wrapup 的括注删 |
| `.claude/rules/command-safety-by-review.md:24`：「提醒里的每一种处置都给能照跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:18`：「的命令，不只写」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:25`：「出路里有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:18`：「的，同时给一条把」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:26`：「停掉子 agent 之后，当」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:19`：「- 停掉子 agent 之后，当场定它的改动与它」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:27`：「定下的写进记录并告诉接手的一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:19`：「写进记录并告诉接手的一方，不留没有主的半截改动。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:28`：「后台任务与脚本任务的名字照与」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:20`：「- 后台任务与脚本任务的名字照与使用者对话的语言」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:29`：「等日志里的一行字之前，先确认」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:21`：「- 等日志里的一行字之前，先确认那一行真会写进那」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:30`：「在 Claude Code」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:16`：「，不在前台干等」 | 已登的改动 | 常驻-12 |
| `.claude/rules/command-safety-by-review.md:34`：「管的是干活中遇到的、自己这次」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:25`：「干活中遇到的、自己这次任务解决不了或不归自己这次」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:36`：「动手修、绕开或停下干等之前，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:25`：「动手修、绕开或停下干等之前，按顺序做：」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:38`：「先查谁在管。 列出正在跑的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:27`：「ListAgents），按任务名、它在改的文件与」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:39`：「有在管的，发消息给它（Cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:28`：「SendMessage），不自己去改它管的那一块」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:40`：「自己修不动、或缺别人手上的信」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:29`：「3. 自己修不动、或缺别人手上的信息与权限，主动」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:41`：「一个在管的都没有：归自己这次」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:30`：「4. 一个在管的都没有：归自己这次任务的，自己处」 | 已登的改动 | 常驻-14：补主 agent 那一支 |
| `.claude/rules/command-safety-by-review.md:42`：「发出去之后接着做手上不依赖它」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:31`：「对方一直不响应，报给用户或调度的一方，不擅自接手」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:44`：「门禁红在不是自己改的文件上怎」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:28`：「照 .claude/singlefs-ai-so」 | 已登的改动 | 常驻-16：指 session-wrapup 的那一半删，转述那一半并进第 2 步 |
| `.claude/rules/command-safety-by-review.md:46`：「这一条还没做成检查：查没查谁」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:33`：「，靠 review。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:50`：「判据：这一批检测项互不依赖，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:37`：「判据：这一批检测项互不依赖，而每一项都要起子进程」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:51`：「那就并行跑，别逐个等。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:37`：「就并行跑」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:53`：「快下来之后再看一眼剩下的时间」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:37`：「超时逻辑被破坏之后 wait 不返回」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:55`：「这三种不并行：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:38`：「不并行的三种：后一项要读前一项的」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:57`：「后一项要读前一项的产物；」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:38`：「后一项要读前一项的产物；」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:58`：「共用一份可写状态（同一个临时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:38`：「共用一份可写状态（同一个临时目录、同一个 tar」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:59`：「每项本身就快，一项只跑几毫秒」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:38`：「项只跑几毫秒。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:61`：「并行度（进程数，以及进程内的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:39`：「进程内的线程数）从参数或环境变量取，」 | 已登的改动 | 用户定第 3 条：缺省取登记的线程数 |
| `.claude/rules/command-safety-by-review.md:65`：「不写 命令 2>/dev/n」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:43`：「不写 命令 2>/dev/null; echo」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:66`：「凡是改状态的命令，都要回头读」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:43`：「确认：判的是读回来的内容，不是那条命令的退出码。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:70`：「做法：被测程序在最后一行报出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:53`：「被测程序在最后一行报出它总共发了多少条，抓取方比」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:71`：「这道闸自己也要先证明会红：喂」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:53`：「先证明会红：喂一个声称的条数比实际发出的多一条的」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:75`：「一个脚本既产出结果、又负责判」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:57`：「脚本既产出结果、又负责判它能不能用时，」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:77`：「做法：结果先写临时文件，闸过」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:57`：「结果先写临时文件，闸过了再输出；判红时 stdo」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:78`：「这道闸同样要证明会红：把脚本」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:57`：「，自证必须判红。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:82`：「用脚本做字符串替换来改代码或」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:61`：「用脚本做字符串替换来改代码或文档」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:84`：「替换必须断言命中：替换数为」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:61`：「替换数为 0 就报错退出」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:85`：「改完回读：grep 一下新内」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:61`：「跑一遍看行为变没变」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:87`：「编译器和 linter 的警」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:61`：「编译器和 linter 的警告不许略过。」 | 落点 |  |

`.claude/rules/command-safety-by-review.md` 续（从原件第 89 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/command-safety-by-review.md:93`：「set -e 下写 里层;」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:67`：「在 if 里取：if 里层; then exit」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:94`：「export 变量名="$(」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:68`：「export 变量名="$(命令)" / loc」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:95`：「握手用的环境变量漏给子进程」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:69`：「读完就 unset；起子进程时 env -u 清」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:97`：「这个值跨了几层进程」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:71`：「问它在下一层还在不在、该不该在。」 | 落点 |  |
| `.claude/rules/command-safety-by-review.md:101`：「mkfs、dd、dmsetu」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:75`：「mkfs、dd、dmsetup remove」 | 落点 |  |

### `.claude/rules/format-evolution.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/format-evolution.md:4`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/format-evolution.md:5`：「共享规则在 .claude/」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/format-evolution.md:7`：「在第一个外部用户出现之前，磁」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:4`：「第一个外部用户出现之前，磁盘格式是软的」 | 落点 |  |
| `.claude/rules/format-evolution.md:9`：「但概念模型要慢（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:4`：「概念模型要慢（」 | 落点 |  |
| `.claude/rules/format-evolution.md:10`：「真正要慎重的是 kb/dec」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:4`：「kb/decisions/ 下的决策正文，不是」 | 落点 |  |
| `.claude/rules/format-evolution.md:14`：「改格式必须同步更新 crat」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:8`：「同步更新 crates/singlefs-che」 | 落点 |  |
| `.claude/rules/format-evolution.md:15`：「决策变更必须记进决策变更史（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:9`：「进 .claude/kb/decisions-h」 | 落点 |  |
| `.claude/rules/format-evolution.md:16`：「组织形态见 .claude/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:9`：「.claude/rules/changelog-」 | 落点 |  |
| `.claude/rules/format-evolution.md:17`：「一旦有外部用户，本文作废，改」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:10`：「改为严格兼容」 | 落点 |  |
| `.claude/rules/format-evolution.md:18`：「并把推翻依据写进决策变更史。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:10`：「并把推翻依据写进决策变更史。」 | 落点 |  |
| `.claude/rules/format-evolution.md:22`：「决策正文长什么样（.clau」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:14`：「（.claude/kb/decisions/NN」 | 落点 |  |
| `.claude/rules/format-evolution.md:24`：「首行 ## D<n> 简称」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:16`：「、门禁 doc-experiments 的 de」 | 落点 |  |
| `.claude/rules/format-evolution.md:25`：「开头一段写射程：管什么、不管」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:17`：「- 开头一段写射程：管什么、不管什么。」 | 落点 |  |
| `.claude/rules/format-evolution.md:26`：「### 已定项 / ###」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:19`：「改新池新建文件的字节：否（YYYY-MM-DD，」 | 落点 |  |
| `.claude/rules/format-evolution.md:27`：「每个分项一节 #### 已定」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:20`：「- 每个分项一节 #### 已定项 N：名字，标」 | 落点 |  |
| `.claude/rules/format-evolution.md:28`：「定案： 现行规则，不带日期；」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:20`：「定案： 现行规则，不带日期；」 | 落点 |  |
| `.claude/rules/format-evolution.md:29`：「射程： 管到哪、不管什么、已」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:20`：「射程： 管到哪、不管什么、已知边角，各一两句」 | 落点 |  |
| `.claude/rules/format-evolution.md:30`：「依据： 只写指针，每条一行—」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:20`：「依据： 只写指针，每条一行——E<n>（简称）」 | 落点 |  |
| `.claude/rules/format-evolution.md:31`：「欠： C<n>（简称），没有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:20`：「欠： C<n>（简称），没有写」 | 落点 |  |
| `.claude/rules/format-evolution.md:32`：「移出正文的东西：定案经过、改」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:21`：「这类只在写下那天成立的句子」 | 落点 |  |
| `.claude/rules/format-evolution.md:33`：「挪的时候不许丢内容：旧正文里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:22`：「按现查结果改成现值；两条已定条款互相矛盾的」 | 落点 |  |
| `.claude/rules/format-evolution.md:35`：「实验页的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:24`：「：每个实验页有一节 ### 影响的决策，」 | 落点 |  |
| `.claude/rules/format-evolution.md:37`：「决策分项写 D<n>（简称）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:26`：「实验正文（历史版本与这一节之外）提到的每条决策都」 | 落点 |  |
| `.claude/rules/format-evolution.md:38`：「实验必须对应决策：表里至少一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:27`：「至少一行关系是支撑、推翻或备料；结论作废或退役的」 | 落点 |  |
| `.claude/rules/format-evolution.md:39`：「关系是 支撑 / 推翻 /」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:28`：「支撑 / 推翻 / 备料 / 不影响」 | 落点 |  |
| `.claude/rules/format-evolution.md:40`：「回看写 YYYY-MM-DD」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:29`：「- 回看写 YYYY-MM-DD 改了 或 YY」 | 落点 |  |
| `.claude/rules/format-evolution.md:41`：「新写的三方判决（resear」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:30`：「- 新写的三方判决（research/promp」 | 落点 |  |
| `.claude/rules/format-evolution.md:43`：「实验页的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:32`：「读了哪些共用项」 | 落点 |  |
| `.claude/rules/format-evolution.md:45`：「路径写它在正文里的名字，逐字」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:34`：「- 路径写它在正文里的名字，逐字一致」 | 落点 |  |
| `.claude/rules/format-evolution.md:46`：「源码落点写 路径:行号 函数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:35`：「是一条要么补实现、要么别说成」 | 落点 |  |
| `.claude/rules/format-evolution.md:47`：「共用项那一列写这条路径读了哪」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:36`：「它错了，这几条会不会一起错」 | 落点 |  |
| `.claude/rules/format-evolution.md:48`：「跨路径的比对断言本身也要钉在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:37`：「钉在源码里（assert_eq!」 | 落点 |  |
| `.claude/rules/format-evolution.md:49`：「结论写成全称的（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:38`：「对所有 X，A 与 B 相同 / 不同」 | 落点 |  |
| `.claude/rules/format-evolution.md:51`：「门禁管哪一半：门禁 doc-」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:40`：「门禁管哪一半：门禁 doc-experiment」 | 落点 |  |
| `.claude/rules/format-evolution.md:55`：「什么时候起这块字节不能再改」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:44`：「的条款时，同时写清谁在那一刻会被叫醒。」 | 落点 |  |
| `.claude/rules/format-evolution.md:56`：「叫不醒任何人的，要么把时刻挪」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:44`：「把时刻挪回提交侧，」 | 落点 |  |
| `.claude/rules/format-evolution.md:57`：「（例如写路径第一次写该结构时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:44`：「写路径第一次写该结构时自检 spec 在不在、指」 | 落点 |  |
| `.claude/rules/format-evolution.md:61`：「门禁 code-source」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:48`：「门禁 code-source-disciplin」 | 落点 |  |
| `.claude/rules/format-evolution.md:62`：「消息串（expect("14」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:48`：「消息串（expect("148")）、按预留宽写」 | 落点 | 括注「即值 − 1」没有落点 |
| `.claude/rules/format-evolution.md:63`：「商与平方（⌊16253 /」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:48`：「商与平方（⌊16253 / 148⌋ = 109」 | 落点 |  |
| `.claude/rules/format-evolution.md:65`：「改一个格式常量时：」 | 无 | 删 | 引导句删 |
| `.claude/rules/format-evolution.md:67`：「动手前按」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:50`：「字面量、消息串、预留宽的读写、下标、倍数、商与平」 | 落点 |  |
| `.claude/rules/format-evolution.md:68`：「改完把只可能指旧值的那几个串」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:51`：「（门禁脚本、research/results/」 | 落点 |  |
| `.claude/rules/format-evolution.md:69`：「裸数字（148）不许登记。登」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:51`：「前现扫一遍，kb 正文与源码里命中 0 次才登记」 | 落点 |  |
| `.claude/rules/format-evolution.md:70`：「标记写在表格单元格里的，挪到」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:52`：「3. 标记写在表格单元格里的，挪到表后单独一行：」 | 落点 |  |

`.claude/rules/format-evolution.md` 续（从原件第 72 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/format-evolution.md:74`：「先做写路径（分配策略、事务边」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:56`：「先做写路径（分配策略、事务边界、COW 顺序），」 | 落点 |  |

### `.claude/rules/fs-design.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/fs-design.md:4`：「具体的设计决策记在 .cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:4`：「在 .claude/kb/decisions.m」 | 落点 |  |
| `.claude/rules/fs-design.md:6`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/fs-design.md:7`：「共享规则在 .claude/」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/fs-design.md:11`：「任何新功能都是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:8`：「一种新的事务」 | 落点 |  |
| `.claude/rules/fs-design.md:12`：「没有事务层的功能实现一律不收」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:8`：「没有事务层的功能实现一律不收。」 | 落点 |  |
| `.claude/rules/fs-design.md:14`：「第一个可运行目标不是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:8`：「能正确提交一个事务」 | 落点 |  |
| `.claude/rules/fs-design.md:15`：「哪怕它只能分配块、写入、提交」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:8`：「只能分配块、写入、提交」 | 落点 |  |
| `.claude/rules/fs-design.md:19`：「谁消费这个数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:12`：「用什么机制算出来」 | 落点 |  |
| `.claude/rules/fs-design.md:23`：「运行时决策路径（分配、ENO」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:16`：「运行时决策路径（分配、ENOSPC 准入、def」 | 落点 |  |
| `.claude/rules/fs-design.md:24`：「checker / 审计」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:17`：「checker / 审计」 | 落点 |  |
| `.claude/rules/fs-design.md:25`：「后台、可续做、非决策路径（销」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:18`：「后台、可续做、非决策路径（销毁快照、整理、scr」 | 落点 |  |
| `.claude/rules/fs-design.md:27`：「空间统计与配额落在第一格：必」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:20`：「空间统计与配额落在第一格：」 | 落点 |  |
| `.claude/rules/fs-design.md:29`：「一个量落不落在第一格，按这一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:21`：「一个量落不落在第一格，按」 | 落点 |  |
| `.claude/rules/fs-design.md:30`：「快照用量」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:21`：「per-snapshot 配额」 | 落点 |  |
| `.claude/rules/fs-design.md:31`：「见 .claude/kb/d」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:21`：「见 .claude/kb/decisions/0」 | 落点 |  |
| `.claude/rules/fs-design.md:33`：「被禁止的不是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:22`：「被禁止的」 | 落点 |  |
| `.claude/rules/fs-design.md:34`：「在需要答案的那一刻还没有答案」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:22`：「需要答案的那一刻还没有答案」 | 落点 |  |
| `.claude/rules/fs-design.md:35`：「运行时与 checker 必」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:22`：「运行时与 checker 必须用不同的算法。」 | 落点 |  |
| `.claude/rules/fs-design.md:37`：「别拿第一格的禁令去否第三格的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:23`：「别拿第一格的禁令去否第三格的设计。」 | 落点 |  |
| `.claude/rules/fs-design.md:41`：「任何一个块单独捡起来，都要能」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:27`：「任何一个块单独捡起来，都要能回答」 | 落点 |  |
| `.claude/rules/fs-design.md:45`：「要在盘上开一块新空间时，先问」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:31`：「它是不是单元」 | 落点 |  |
| `.claude/rules/fs-design.md:46`：「不是 ⇒ 校验和 / MAC」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:31`：「校验和 / MAC、扫描重建认领、checker」 | 落点 |  |
| `.claude/rules/fs-design.md:47`：「要么逐条补齐各自的替代品，要」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:31`：「替代品，要么把它做成单元。」 | 落点 |  |
| `.claude/rules/fs-design.md:48`：「说的是格式形态，D21（权威」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:31`：「D21（权威态与派生态的分界） 已定项 17 第」 | 落点 |  |
| `.claude/rules/fs-design.md:49`：「核心层的对象」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:31`：「说的是可达性身份：」 | 落点 |  |
| `.claude/rules/fs-design.md:50`：「一块空间可以既是单元、又不从」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:31`：「一块空间可以既是单元、又不从任何根独立可达（生命」 | 落点 |  |
| `.claude/rules/fs-design.md:54`：「自描述数据宁可写大一点。 遇」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:35`：「这个字段能不能省掉 /」 | 落点 |  |
| `.claude/rules/fs-design.md:55`：「默认答案是不省——除非有实测」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:35`：「除非有实测证明省下来的那部分是瓶颈。」 | 落点 |  |
| `.claude/rules/fs-design.md:57`：「不构成任何判据。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:36`：「不构成任何判据」 | 落点 |  |
| `.claude/rules/fs-design.md:58`：「我们比 X 省 / 比 X」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:36`：「我们比 X 省 / 比 X 费」 | 落点 |  |
| `.claude/rules/fs-design.md:59`：「只许用来驳」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:36`：「只许用来驳」 | 落点 |  |
| `.claude/rules/fs-design.md:61`：「这条不是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:37`：「已经论证过有用的字段该不该为省空间而砍，」 | 落点 |  |
| `.claude/rules/fs-design.md:62`：「要不要加这个字段」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:37`：「各自给理由」 | 落点 |  |
| `.claude/rules/fs-design.md:64`：「它也不推翻」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:37`：「格式上吝啬分支」 | 落点 |  |
| `.claude/rules/fs-design.md:68`：「可选的是 AAD 那道运算（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:41`：「可选的是 AAD 那道运算（加密关着就不算），不」 | 落点 |  |
| `.claude/rules/fs-design.md:70`：「判据表怎么写：依据来自 D9」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:42`：「加密不进第一个可运行版本（D9（加密） 已定项」 | 落点 |  |
| `.claude/rules/fs-design.md:71`：「不要提议」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:41`：「不加密的卷省掉加密预留位」 | 落点 |  |
| `.claude/rules/fs-design.md:75`：「incompat（不认识不许」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:46`：「分出去的位登记在 D15（格式冻结政策） 已定项」 | 落点 |  |
| `.claude/rules/fs-design.md:79`：「逻辑地址、物理地址、设备内偏」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:50`：「逻辑地址、物理地址、设备内偏移、代号、inode」 | 落点 |  |
| `.claude/rules/fs-design.md:83`：「不许为某个路径（如 fsyn」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:54`：「为某个路径（如 fsync）开第二套事务机制。」 | 落点 |  |
| `.claude/rules/fs-design.md:85`：「它约束的是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:54`：「字节完全相同的落盘代码」 | 落点 |  |
| `.claude/rules/fs-design.md:86`：「一个共享的事务抽象状态机 +」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:54`：「一个共享的事务抽象状态机」 | 落点 |  |
| `.claude/rules/fs-design.md:87`：「这一步具体怎么落盘」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:54`：「这一步具体怎么落盘」 | 落点 |  |
| `.claude/rules/fs-design.md:88`：「绕开状态机的第二条发布路径不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:54`：「绕开状态机的第二条发布路径不允许」 | 落点 |  |
| `.claude/rules/fs-design.md:90`：「提交协议归本工程独占，不是可」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:55`：「提交协议归本工程独占」 | 落点 |  |
| `.claude/rules/fs-design.md:91`：「一条第三方管道要引入新的结构」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:55`：「第三方管道要引入新的结构性提交步骤，」 | 落点 |  |
| `.claude/rules/fs-design.md:92`：「不许在自己的黑盒里发明。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:55`：「在自己的黑盒里发明。」 | 落点 |  |

`.claude/rules/fs-design.md` 续（从原件第 94 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/fs-design.md:96`：「X 同 Y」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:59`：「X = 一次 Y」 | 落点 |  |
| `.claude/rules/fs-design.md:98`：「写借用式条款时，把被借规则的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:59`：「在 X 上各写一句它取什么」 | 落点 |  |
| `.claude/rules/fs-design.md:102`：「不硬造万能方案，显式分支：不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:67`：「不硬造万能方案，显式分支：不同规模、不同介质、不」 | 落点 |  |
| `.claude/rules/fs-design.md:106`：「只在操作期间不会变的输入上分」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:71`：「1. 只在操作期间不会变的输入上分支。 设备类型」 | 落点 |  |
| `.claude/rules/fs-design.md:107`：「当前填充率」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:71`：「会在操作中途越过阈值，」 | 落点 |  |
| `.claude/rules/fs-design.md:109`：「行为上放开分支，格式上吝啬分」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:72`：「2. 行为上放开分支，格式上吝啬分支。」 | 落点 |  |
| `.claude/rules/fs-design.md:110`：「第一个外部用户出现之前格式是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:72`：「第一个外部用户出现之前格式是软的（.claude」 | 落点 |  |
| `.claude/rules/fs-design.md:112`：「门禁的范围必须可判定——挂钟」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:73`：「3. 门禁的范围必须可判定——挂钟本身不是约束。」 | 落点 |  |
| `.claude/rules/fs-design.md:113`：「单次改动只碰一条分支时，只重」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:73`：「语料总量再大也不构成成本。」 | 落点 |  |
| `.claude/rules/fs-design.md:114`：「这次改动作废了哪几条判定」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:73`：「这次改动作废了哪几条判定」 | 落点 |  |
| `.claude/rules/fs-design.md:115`：「我只动了一条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:73`：「我只动了一条」 | 落点 |  |
| `.claude/rules/fs-design.md:116`：「落点：[checks-owe」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:73`：「C19（介质判定渗进事务层）」 | 落点 |  |
| `.claude/rules/fs-design.md:120`：「切换判据必须显式、可计算、写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:77`：「1. 切换判据必须显式、可计算、写下来」 | 落点 |  |
| `.claude/rules/fs-design.md:121`：「，也不许是埋在代码里的魔数。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:77`：「代码里的魔数。」 | 落点 |  |
| `.claude/rules/fs-design.md:122`：「每条分支必须能被测试强制进入」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:78`：「2. 每条分支必须能被测试强制进入」 | 落点 |  |
| `.claude/rules/fs-design.md:123`：「留一个只供测试使用的开关，能」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:78`：「无视判据直接走指定分支」 | 落点 |  |
| `.claude/rules/fs-design.md:124`：「进不去就等于没测——靠碰运气」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:78`：「进不去就等于没测」 | 落点 |  |
| `.claude/rules/fs-design.md:125`：「每条分支单独验，包括崩溃点重」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:79`：「3. 每条分支单独验，包括崩溃点重放」 | 落点 |  |
| `.claude/rules/fs-design.md:126`：「另一条分支测过了」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:79`：「另一条分支测过了」 | 落点 |  |
| `.claude/rules/fs-design.md:127`：「分支必须可观测：运行时能看出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:80`：「4. 分支必须可观测：运行时」 | 落点 |  |
| `.claude/rules/fs-design.md:128`：「分支变量不许在操作中途改变。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:81`：「5. 分支变量不许在操作中途改变」 | 落点 |  |
| `.claude/rules/fs-design.md:130`：「判据写不出来时，先只做一条，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:83`：「判据写不出来时，先只做一条，而且先做慢而正确的那」 | 落点 |  |
| `.claude/rules/fs-design.md:134`：「.claude/rules/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:87`：「.claude/rules/machine-fi」 | 落点 |  |
| `.claude/rules/fs-design.md:135`：「用类型让非法状态无法表示——」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:87`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/fs-design.md:136`：「格式层也照用，判据见」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:87`：「格式层也照用」 | 落点 |  |
| `.claude/rules/fs-design.md:167`：「一条格式分支值不值得，看它能」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「一条格式分支值不值得，看它能不能把」 | 落点 |  |
| `.claude/rules/fs-design.md:169`：「值得」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「一条格式分支值不值得，看它能不能」 | 已登的改动 | 常驻-17：表头当成了数据，写成一句 |
| `.claude/rules/fs-design.md:171`：「不值得」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「一条格式分支值不值得，看它能不能」 | 已登的改动 | 常驻-17：表头当成了数据，写成一句 |
| `.claude/rules/fs-design.md:173`：「格式上吝啬分支」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「是买不到失败模式改善的那些分支」 | 落点 |  |
| `.claude/rules/fs-design.md:175`：「判据里不许出现」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「判据里不许出现」 | 落点 |  |
| `.claude/rules/fs-design.md:176`：「拿性能去换永久的格式债，要单」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:95`：「单独立决策、单独给数字」 | 落点 |  |
| `.claude/rules/fs-design.md:182`：「D12（目标介质） 的介质分」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:99`：「D12（目标介质） 的介质分层布局」 | 落点 |  |
| `.claude/rules/fs-design.md:183`：「D9（加密） 预留的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:100`：「D9（加密） 预留的」 | 落点 |  |
| `.claude/rules/fs-design.md:184`：「持久 / 临时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:101`：「D14（双轨（大小文件 / 持久临时）））」 | 落点 |  |
| `.claude/rules/fs-design.md:188`：「按访问模式选索引结构是允许的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:63`：「推荐的（反向索引写多读少、目录项点查、exten」 | 落点 |  |
| `.claude/rules/fs-design.md:189`：「唯一不许多样化的是事务层。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:63`：「唯一不许多样化的是事务层。」 | 落点 |  |

### `.claude/rules/show-me-test-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/show-me-test-by-review.md:6`：「都得能失败。 做法是：先把被」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:6`：「先把被测代码改坏，看着测试变红，再改回来」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:7`：「证明不了会红的测试，等于没写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:6`：「证明不了会红的测试」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:9`：「脚本只看得出有没有测试，查不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「commit message」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:10`：「改坏了哪一行，看到哪个断言失」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「改坏了哪一行，看到哪个断言失败」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:12`：「有变异 harness 的仓」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「做成存进仓里的变异」 | 已登的改动 | 用户定第 4 条：两样都要 |
| `.claude/rules/show-me-test-by-review.md:13`：「交给门禁反复复跑。commi」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「commit message 里」 | 已登的改动 | 用户定第 4 条：两样都要 |
| `.claude/rules/show-me-test-by-review.md:15`：「设计论证同样照办：对抗轮打中」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:8`：「设计论证同样照办：对抗轮打中的每一条，先在模型里」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:19`：「立的时机照 .claude/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:12`：「立的时机照 .claude/singlefs-a」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:21`：「踩到新坑，第一反应该是」 | `.claude/singlefs-ai-sop/rules/sop-first.md:10`：「这条怎么做成门禁里会红的一项」 | 上游 | 合稿 show-me-test-by-review.md 第 12 行留指针 |
| `.claude/rules/show-me-test-by-review.md:22`：「做成一个当场拒绝执行的检查，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:12`：「当场拒绝执行的检查，」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:26`：「判据不是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:21`：「判据：换一套装置从头把同一件事再」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:30`：「一段代码的行为里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:18`：「一段代码的行为里」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:31`：「建模口径上（同一个量该怎么算」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:19`：「立在装置之间：两套装置算同一个量，就要有一条检查」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:33`：「快速判据：换一套装置、从头把」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:21`：「从头把同一件事再做一遍，还会不会再踩一次」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:35`：「跨装置的检查要钉住」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:22`：「：两个量就登记两个名字，各钉各的值」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:36`：「（.claude/singl」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:22`：「（.claude/singlefs-ai-sop」 | 已登的改动 | 常驻-5：指针写到节与行 |
| `.claude/rules/show-me-test-by-review.md:40`：「一个规范、一次操作、一次改动」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:26`：「一个规范、一次操作、一次改动只铺了一部分文件时，」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:41`：「一行一条、# 后面写为什么不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:26`：「一行一条、# 后」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:43`：「判据不是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:27`：「改了这句话就成假的，或者改了某道闸就失效」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:44`：「别家项目的术语与原文引文、某」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:27`：「别家项目的术语与原文引文、某道检查自己的输入、工」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:45`：「这几类改了会出错，登记；其余」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:27`：「其余一律改。」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:47`：「排除表指向不存在的路径判红。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:28`：「排除表指向不存在的路径判红」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:49`：「成功那句里报的数，本身也要有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:29`：「gate-lint 不管这一条，靠写阶段的人。」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:53`：「Gate proves ev」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:33`：「门禁证明的是证据要求被满足，不是」 | 落点 | 英文原句在合稿 machine-first.md 第 43 行 |
| `.claude/rules/show-me-test-by-review.md:55`：「门禁证明的是证据要求被满足，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:33`：「门禁证明的是证据要求被满足，不是代码语义正确。」 | 落点 | 英文原句在合稿 machine-first.md 第 43 行 |
| `.claude/rules/show-me-test-by-review.md:61`：「测试测的是实现，不是契约」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:39`：「门禁只数有没有测试，不判测得对不对」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:62`：「声明本身写错了」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:40`：「声明与实测一致就判」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:63`：「不变量本身就是错的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:41`：「检查会老老实实判一条错规矩，而且全绿」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:64`：「覆盖的不是出问题的那条路径」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:42`：「覆盖的不是出问题的那条路径」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:65`：「正文引的数与它自己的产物对不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:43`：「复跑的区间断言只钉住结论落在哪一档，」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:66`：「两套装置对同一个量算出不同的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:44`：「两套装置对同一个量算出不同的数」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:67`：「设计本身就错了」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:45`：「设计本身就错了」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:71`：「门禁绿不等于可以不看代码。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:49`：「1. 门禁绿不等于可以不看代码」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:73`：「语义对不对，写在 kb/de」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:50`：「kb/decisions.md 和 crates」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:74`：「门禁只保证这两样写下来了、实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:50`：「写下来了、实现了、」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:75`：「它们本身写得对不对，靠人。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:50`：「它们本身写得对不对，靠人。」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:77`：「门禁是下限，不是上限（.cl」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:51`：「3. 门禁是下限，不是上限」 | 已登的改动 | 常驻-6：判据留，指针删 |
| `.claude/rules/show-me-test-by-review.md:79`：「没实现的阶段要一直列出来。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:52`：「gate.sh 每次都打印这份清单」 | 落点 | 换下来的条件指上游 show-me-test.md 第 25 行 |
| `.claude/rules/show-me-test-by-review.md:80`：「项目阶段声明覆盖了某一项（#」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:52`：「项目阶段声明覆盖了某一项（# gate-cove」 | 落点 | 换下来的条件指上游 show-me-test.md 第 25 行 |
| `.claude/rules/show-me-test-by-review.md:82`：「交回的清单能被机器核全，不等」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:53`：「5. 交回的清单能被机器核全，不等于判对了」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:83`：「子 agent 逐项判的活（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:53`：「每一项都有判定、判定是认得的那几种、理由里引了那」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:84`：「它拦得住」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:53`：「在全部判定种类里抽查复判，抽中判错的那一段重做、」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:88`：「这个项目不按来源分类，也不给」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:57`：「不按来源分类，也不给哪一类单开规矩」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:89`：「只分两种：带证据的提交，和不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:57`：「带证据的提交」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:93`：「跑批脚本不许把单轮的失败吞掉」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:61`：「跑批脚本不许把单轮的失败吞掉（」 | 落点 |  |
| `.claude/rules/show-me-test-by-review.md:94`：「这份输出能不能证明是这一轮产」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:61`：「：跑之前删掉旧输出，跑完认一个本轮才会出现的完成」 | 落点 |  |

### `.claude/rules/writing-discipline-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/writing-discipline-by-review.md:6`：「三类文档，三套写法：」 | 无 | 删 | 引导句删 |
| `.claude/rules/writing-discipline-by-review.md:10`：「设计文档」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:8`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:11`：「工程 kb」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:9`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:12`：「能变成会失败的检查」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:10`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:14`：「动笔前先问一句：这份东西是给」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:12`：「：这份东西是给人从头读的，还是会被模型单条取出来」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:16`：「三类只有一条是通用的：正文只」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:12`：「：正文只写现状」 | 已登的改动 | 常驻-7：历史放哪照各自的规则 |
| `.claude/rules/writing-discipline-by-review.md:20`：「尺子：几行到几十行的改动，c」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:16`：「尺子：几行到几十行的改动，commit 正文最多」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:21`：「写不下就先查是不是在写不该写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:16`：「先查是不是在写不该写的东西。」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:23`：「删到合适长度是写代码的人的事」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:16`：「删到合适长度是写代码的人的事」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:27`：「判据：删掉这句话，读的人少知」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:20`：「，读的人少知道了什么？」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:28`：「读着更顺」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:20`：「显得更周全」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:30`：「这三种直接删：把说过的话换个」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:22`：「换个说法再说一遍、给没人质疑的结论加」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:32`：「但短不能靠丢内容换来。 证据」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:26`：「证据、口径、下一步（.claude/single」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:33`：「该留什么，见」 | 无 | 删 | 文件内指针删 |
| `.claude/rules/writing-discipline-by-review.md:37`：「拿不准时留着。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:26`：「拿不准时留着。」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:39`：「这三种是删过头：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:26`：「拿不准时留着。证据、口径、下一步」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:43`：「实测 3.26×」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:30`：「口径：什么硬件、什么负载、块多大」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:44`：「下一步：.claude/si」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:26`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:45`：「按某某决策办」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:32`：「依据：为什么这么定」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:47`：「正反两条判据都要过：」 | 无 | 删 | 引导句删，两条在第 20、21 行 |
| `.claude/rules/writing-discipline-by-review.md:49`：「正：删掉这句，读的人少知道什」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:20`：「- 正：删掉这句，读的人少知道」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:50`：「反：留着这句，读的人多知道什」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:21`：「- 反：留着这句，读的人多知道什么？说得上来」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:54`：「给 review 用的实测数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:36`：「reviewer 拿过去自己就能验的」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:58`：「数字：命中/未命中计数、前后」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:40`：「数字：命中/未命中计数、前后对比、耗时」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:59`：「复现命令和它的原样输出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:41`：「我是怎么想到跑这条命令的」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:60`：「复跑要用的口径：版本、配置、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:42`：「复跑要用的口径：版本、配置、硬件」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:62`：「只贴能定胜负的那几个数，完整」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:36`：「只贴能定胜负的那几个数，完整记录放」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:66`：「主观评价。 换成能核对的事实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:46`：「换成能核对的事实。」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:67`：「和 kb 重复的内容。 结论」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:47`：「结论放正文，验证方法放 kb。」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:68`：「旁证。 读者还得自己推一步才」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:48`：「读者还得自己推一步才能用的证据，换成直接就是结论」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:69`：「完整的推导过程。 我怎么一步」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:49`：「4. 完整的推导过程。」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:70`：「代码注释里的背景交代。 注释」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:50`：「注释说明这一行是什么，不承担论证。」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:71`：「为过去辩护。 commit」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:51`：「commit 只写改了什么，讨论留在上一轮的记录」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:72`：「贴源码。 日志和 dump」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:52`：「日志和 dump」 | 落点 |  |
| `.claude/rules/writing-discipline-by-review.md:73`：「例外是不贴读者就没法自己验的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:52`：「不贴读者就没法自己验的」 | 落点 |  |

### `.claude/skills/crash-test/commit-stages.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/crash-test/commit-stages.md:3`：「子 agent 不交这几道（」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:3`：「六道门禁（文件是 .claude/gate.d/」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:4`：「开工先读：.claude/s」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:4`：「开工先读：.claude/skills/cras」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:8`：「这次改动的范围：git di」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:8`：「- 这次改动的范围：git diff --sta」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:9`：「这一次是提交时跑还是用户要求」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:9`：「- 这一次是提交时跑还是用户要求时跑（只在这两种」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:10`：「崩溃放量全量：每次提交都跑全」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:10`：「- 崩溃放量全量：每次提交都跑全部分项（不带 -」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:11`：「草稿目录。」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:11`：「- 草稿目录。」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:15`：「另有别的 gate.sh 正」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:15`：「1. 另有别的 gate.sh 正在跑 cras」 | 落点 | 一步拆成五行；「键相同的复用、键对不上的现判」在合稿 verification.md 第 35 行 |
| `.claude/skills/crash-test/commit-stages.md:16`：「六道与崩溃放量不指定范围的那」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:20`：「。结局读 task wait 的退出码与末行 T」 | 落点 | 包装名单、分片与领卡的声明改成指共用约束与脚本文件头 |
| `.claude/skills/crash-test/commit-stages.md:17`：「只在提交时（或用户要求时）跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:21`：「那一行点名的相同，不从阶段归属表取；不跑别的门禁」 | 落点 | 一步拆成六行；各片与合并判什么、三处怎么领卡改成指针 |
| `.claude/skills/crash-test/commit-stages.md:18`：「broad-mutation」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:27`：「2b. broad-mutation 等 cra」 | 落点 | 哪些条补 GPU 那一趟、任一趟红算抓到改成指驱动文件头 |
| `.claude/skills/crash-test/commit-stages.md:19`：「门禁起的驱动判的是主工作区（」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:28`：「① 门禁起的驱动判的是主工作区（--source」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:20`：「SINGLEFS_RUN_S」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:29`：「② SINGLEFS_RUN_SCOPE='种类」 | 落点 | 计划怎么分堆、各片怎么跑改成指驱动文件头 |
| `.claude/skills/crash-test/commit-stages.md:21`：「退非 0 时原样抄这一道与驱」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:31`：「分片里没成的那一步照第 1b 步抄末行的 sha」 | 落点 | 一步拆成十行 |
| `.claude/skills/crash-test/commit-stages.md:22`：「门禁 crash-point」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:40`：「，要两张、三张卡的对拍目标在格里不起），逐个点名」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:23`：「每个阶段原样抄它的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:41`：「。服务判出的结局（task wait 末行 ty」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:24`：「计数行原样抄：崩溃状态数、恢」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:42`：「merge：每条用例起一趟 0/1 的核账，核账」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:25`：「缺 herd7、缺 KVM、」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:43`：「，以阶段自己的报错为准，另贴 command -」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:26`：「每个阶段开跑与结束时各记一次」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:44`：「6. 每个阶段开跑与结束时各记一次指纹，开跑那次」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:30`：「这一趟 ✓ N ✗ M 本次」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:48`：「的 N、M、K 数第 3、4 步抄下来的判定行，」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:32`：「单位是一条判定：阶段汇总里的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:50`：「那一行）、门禁 crash-point-repl」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:33`：「✓：判定是绿的。门禁 cra」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:51`：「crash-point-replay 逐条用例行」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:34`：「✗：判定是红的。一条判定的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:52`：「✗ <阶段>：N 格里 M 格红」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:35`：「本次未跑：退出码 77 的格」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:53`：「- 本次未跑：退出码 77 的格、项与阶段，报」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:36`：「监控行里 ✗ 的逐条点名，本」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:54`：「K 后面逐条点名是哪一条、属于 77、作废、没跑」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:37`：「三栏相加要等于这一趟数到的判」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:55`：「等于这一趟数到的判定条数；对不上就回头重数，对上」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:41`：「草稿目录。阶段自己用的临时目」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:59`：「- 草稿目录。阶段自己用的临时目录、编译产物（c」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:45`：「一张表：阶段 / 退出码（0」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:63`：「两条。每个阶段的整份输出都写进 <草稿目录>/<」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:49`：「红了不在这里修：照 .cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:67`：「- 红了不在这里修：照 .claude/skil」 | 落点 |  |
| `.claude/skills/crash-test/commit-stages.md:50`：「全绿只说明这几道的证据要求满」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:68`：「- 全绿只说明这几道的证据要求满足；崩溃点重放覆」 | 落点 |  |

### `.claude/skills/gate/staged-gate.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/gate/staged-gate.md:3`：「子 agent 不交整轮门禁」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:3`：「）。跑之前先读：.claude/singlefs」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:7`：「这一轮的改动已经按 scri」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:7`：「- 这一轮的改动已经按 scripts/gate」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:8`：「这一次是提交时跑还是用户要求」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:8`：「- 这一次是提交时跑还是用户要求时跑（只在这两种」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:9`：「这一轮暂存区 diff 的范」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:9`：「- 这一轮暂存区 diff 的范围：git di」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:10`：「草稿目录。」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:10`：「- 草稿目录。」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:14`：「另有别的 gate.sh 在」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:14`：「1. 另有别的 gate.sh 在跑就等它结束。」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:15`：「自己单跑的编译出来的代码（c」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:15`：「1b. 自己单跑的编译出来的代码（cargo t」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:16`：「只在提交时（或用户要求时）跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:17`：「SINGLEFS_RUN_SCOPE='种类=w」 | 落点 | 一步拆成六行 |
| `.claude/skills/gate/staged-gate.md:17`：「每个红阶段：抄门禁给的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:23`：「看它点名的文件与行在不在暂存区 diff 的块里」 | 落点 | 一步拆成八行 |
| `.claude/skills/gate/staged-gate.md:18`：「原样抄未实现清单与」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:30`：「。gate.sh 不打印单个阶段的耗时，取不到的」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:22`：「草稿目录。门禁自己的 git」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:34`：「gate-staged.sh 整轮全绿时 upd」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:26`：「一张表：阶段 / 绿·红·7」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:38`：「- 一张表：阶段 / 绿·红·77 / 归属（这」 | 落点 |  |
| `.claude/skills/gate/staged-gate.md:30`：「红了不在这里修，修照 .cl」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/staged-gate.md:42`：「修照 .claude/main-agent.md」 | 落点 |  |

### `.claude/rules/changelog-format.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/changelog-format.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明与说明句删（用户定第 11 条） |
| `.claude/rules/changelog-format.md:4`：「哪几类文件有历史、历史住在哪」 | 无 | 删 | 开头声明与说明句删（用户定第 11 条） |
| `.claude/rules/changelog-format.md:8`：「历史只记两类：决策（.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:5`：「决策（.claude/kb/decisions/」 | 落点 |  |
| `.claude/rules/changelog-format.md:9`：「.claude/kb/dec」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:5`：「.claude/kb/decisions-his」 | 落点 |  |
| `.claude/rules/changelog-format.md:11`：「其余文件（records/、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:6`：「其余文件（records/、.claude/kb」 | 落点 |  |
| `.claude/rules/changelog-format.md:12`：「改动直接改正文，经过靠 gi」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:6`：「改动直接改正文，经过靠 git。哪些 kb 文件」 | 落点 |  |
| `.claude/rules/changelog-format.md:13`：「上游 doc-lint 规则」 | `.claude/singlefs-ai-sop/rules/kb-discipline.md:84`：「## 历史版本」 | 上游 | 合稿 changelog-format.md 第 6 行留指针 |
| `.claude/rules/changelog-format.md:28`：「条目：每条决策一个 ## D」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:21`：「- 条目：每条决策一个 ## D<n>（简称）」 | 落点 |  |
| `.claude/rules/changelog-format.md:29`：「日期：节里按 ### YYY」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:22`：「- 日期：节里按 ### YYYY-MM-DD」 | 落点 |  |
| `.claude/rules/changelog-format.md:30`：「变更：块下每条改动一个顶格的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:23`：「- 变更：块下每条改动一个顶格的 - 项，写结论」 | 落点 |  |
| `.claude/rules/changelog-format.md:31`：「日期只在标题上：项里不写与所」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:24`：「- 日期只在标题上：项里不写与所属 ### 相同」 | 落点 |  |
| `.claude/rules/changelog-format.md:32`：「改前只写引用：git sho」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:25`：「- 改前只写引用：git show <提交>:<」 | 落点 |  |
| `.claude/rules/changelog-format.md:33`：「一条改动点名几条决策：只在标」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:26`：「- 一条改动点名几条决策：只在标题里点名的第一条」 | 落点 |  |
| `.claude/rules/changelog-format.md:35`：「同一天可以先追加几个 - 项」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:28`：「同一天可以先追加几个 - 项；提交前主 agen」 | 落点 |  |
| `.claude/rules/changelog-format.md:36`：「整理只许做四种：与标题同日的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:28`：「整理只许做四种：与标题同日的日期删掉、包装（快查」 | 落点 |  |
| `.claude/rules/changelog-format.md:37`：「数、编号、限定词、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:28`：「引出的理由、引名一个」 | 落点 |  |
| `.claude/rules/changelog-format.md:38`：「python3 script」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:28`：「python3 scripts/kb/histo」 | 落点 |  |
| `.claude/rules/changelog-format.md:42`：「门禁 doc-decisio」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:32`：「：entry-added（决策正文改了，deci」 | 落点 |  |
| `.claude/rules/changelog-format.md:43`：「shape（日期块住在自己的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:32`：「shape（日期块住在自己的节、同节同一天只一块」 | 落点 |  |
| `.claude/rules/changelog-format.md:44`：「数与编号有没有丢由 hist」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:32`：「（两份变更史与基准相比，基准里每个日期块的数、编」 | 落点 |  |
| `.claude/rules/changelog-format.md:46`：「它管不了的：结论写得对不对、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:32`：「结论写得对不对、一条改动该点名的决策或实验」 | 落点 |  |

### `.claude/rules/code-discipline-by-review.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/code-discipline-by-review.md:6`：「能恢复的失败（I/O 错、盘」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:8`：「- 能恢复的失败（I/O 错、盘上数据损坏、资源」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:7`：「不按底层原因逐个展开：比如上」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:8`：「不按底层原因逐个展开：」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:8`：「底层几十种系统错误挂在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:8`：「里当 source 带着」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:9`：「判据是调用方有没有因此失去该」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:9`：「库代码的公开边界不用 anyhow、Box<dy」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:10`：「也不把每一种底层失败都在每一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:9`：「测试和命令行工具可以用」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:11`：「不变量被破坏是 bug，用断」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:10`：「- 不变量被破坏是 bug，用断言，不带着坏状态」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:12`：「不写 unwrap()，写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:11`：「- 不写 unwrap()，写 expect，消」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:13`：「expect("journa」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:11`：「expect("journal 头在 open(」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:17`：「函数的长度按」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:15`：「能单独验证的」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:18`：「两件事挤在二十行里要拆开。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:15`：「两件事挤在二十行里要拆」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:20`：「嵌套层数不设限，限的是路径数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:16`：「加路径（break、continue、?」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:21`：「条件嵌套按指数增加（12 层」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:16`：「嵌套按指数」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:22`：「层数随意，只要两条都满足：路」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:16`：「：路径数说得出、而且有限；」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:23`：「每一层的迭代对象有名字、有类」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:16`：「每一层的迭代对象有名字、有类型，不」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:25`：「路径数只量控制流，不是验证难」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:17`：「路径数超了就拆；没超不等于好验。循环」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:26`：「迭代次数的上界、跨轮携带的状」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:17`：「它的不变量（写成断言）、每一个提前出口。」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:27`：「只有一条路径」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:17`：「只有一条路径」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:29`：「参数不按个数限。 每个参数有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:18`：「几个参数本身构成一个带不变量的概念时」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:30`：「才收成一个类型，不为了让签名」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:18`：「才收成一个类型，不为」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:31`：「结果放进返回类型，不用 &m」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:19`：「- 结果放进返回类型，不用 &mut 输出参数」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:32`：「副作用挤到边上，核心写成纯函」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:20`：「- 副作用挤到边上，核心写成纯函数，带 I/O」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:33`：「提前返回随便用。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:21`：「- 提前返回随便用。」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:37`：「每一条都照 .claude/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:25`：「照 .claude/rules/machine-」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:43`：「代码读的次数比写的多，要为读」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:29`：「代码读的次数比写的多，要为读的人优化」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:44`：「别写聪明的代码」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:31`：「别写聪明的代码」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:45`：「KISS，保持简单」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:30`：「简单指控制流路径少、要验的状态少、每条路径都写明」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:46`：「YAGNI，用不到的别写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:31`：「用不到的代码不写」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:47`：「最小惊讶原则」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:31`：「名字和类型带够信息，让模型猜对行为」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:48`：「约定优于配置」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:32`：「不靠隐式的默认行为」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:49`：「关注点分离、高内聚低耦合」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:31`：「边界清楚、契约明确，每一块能单独验证」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:50`：「文件、模块不要太长」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:33`：「模块按契约切，不按行数切」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:56`：「名字要短，离声明越近越短」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「离声明越近越短」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:57`：「循环计数叫 i，临时变量叫」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「临时变量叫 tmp」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:58`：「泛型参数用 T、U，生命周期」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「泛型参数用 T、U，生命周期用 'a」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:59`：「大家都认得的缩写可以用」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「大家都认得的缩写可以用」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:60`：「名字要念得出来」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:38`：「要的是不歧义、搜得到」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:61`：「名字要搜得到」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「名字要搜得到」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:62`：「匈牙利命名法、m_ 前缀、接」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「含义不同就是不同的 newtype」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:63`：「不用 Manager、Hel」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:39`：「不用 Manager、Helper、Data、I」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:64`：「模块名别在类型名里重复（bl」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「模块名别在类型名里重复」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:65`：「取值方法不加 get_ 前缀」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:40`：「Rust API 指南 C-GETTER」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:66`：「转换方法按 as_、to_、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:40`：「as_ 借用到借用、约定几乎无开销；to_ 约定」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:67`：「布尔名以 is_、has_」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「布尔名以 is_、has_ 开头」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:68`：「类型用名词、方法用动词」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:41`：「类型用名词、方法用动词」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:69`：「Rust 的大小写约定（sn」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:41`：「Rust 的大小写约定」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:70`：「同名遮蔽（let text」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「由 clippy 拦」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:71`：「测试函数叫 test_<被测」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:37`：「测试函数叫 test_<被测函数>」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:77`：「函数要短：一两屏，最好不到」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「函数要短：一两屏，最好不到 20 行」 | 已登的改动 | 按需-M1：处置「留下」改「改写」 |
| `.claude/rules/code-discipline-by-review.md:78`：「只做一件事」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「能单独验证的」 | 落点 | 写法在第 15 行 |
| `.claude/rules/code-discipline-by-review.md:79`：「每个函数只有一个抽象层次」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:46`：「要紧的是契约清楚、路径数有限」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:80`：「从上往下读（下沉规则）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:47`：「不要求函数按调用顺序」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:81`：「参数最多三个」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「参数最多三个」 | 落点 | 写法在第 18 行 |
| `.claude/rules/code-discipline-by-review.md:82`：「不用标志参数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:48`：「不用标志参数」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:83`：「不用输出参数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「不用输出参数」 | 落点 | 写法在第 19 行 |
| `.claude/rules/code-discipline-by-review.md:84`：「函数没有副作用；函数式核心、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「函数没有副作用；函数式核心、命令式外壳」 | 落点 | 写法在第 20 行 |
| `.claude/rules/code-discipline-by-review.md:85`：「查询与修改分开」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「查询与修改分开」 | 落点 | 写法在第 20 行 |
| `.claude/rules/code-discipline-by-review.md:86`：「单一出口」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「单一出口」 | 落点 | 写法在第 21 行 |
| `.claude/rules/code-discipline-by-review.md:87`：「嵌套不超过三层」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:45`：「嵌套不超过三层」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:88`：「圈复杂度不超过某个数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:49`：「穷举 match 的臂数不算」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:94`：「用值对象代替基本类型」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「用值对象代替基本类型」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:95`：「让非法状态写不出来」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「让非法状态写不出来」 | 落点 | 写法在上游「类型：让非法状态写不出来」 |
| `.claude/rules/code-discipline-by-review.md:96`：「解析，而不是校验」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「解析，而不是校验」 | 落点 | 写法在上游「类型：让非法状态写不出来」 |
| `.claude/rules/code-discipline-by-review.md:97`：「字段私有，配上 getter」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「字段私有，配上 getter / setter」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:98`：「默认不可变」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:54`：「默认不可变」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:99`：「不用全局可变状态；不用单例」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:54`：「不用全局可变状态；不用单例」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:100`：「能 derive Defau」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「能 derive Default 就 deriv」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:101`：「结构体更新语法 ..Defa」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「..Default::default()」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:102`：「字段多就上 builder」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「字段多就上 builder」 | 落点 | 写法在上游 code-discipline.md 第 136、137 行 |
| `.claude/rules/code-discipline-by-review.md:103`：「数值转换用 as」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「由 clippy 拦」 | 落点 | 写法在上游 code-discipline.md 第 136、137 行 |
| `.claude/rules/code-discipline-by-review.md:104`：「别用 typedef 给结构」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:55`：「别用 typedef 给结构体换个名字」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:105`：「魔法数字要起名」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:53`：「魔法数字要起名」 | 落点 | 写法在上游 code-discipline.md 第 151 行 |
| `.claude/rules/code-discipline-by-review.md:111`：「用多态替换条件分支」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:59`：「用多态替换条件分支」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:112`：「开闭原则」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:59`：「加一种情形，让编译器逼着把每一处旧代码过一遍」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:113`：「单一职责」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:60`：「一块只有一个能单独验证的契约」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:114`：「里氏替换」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:61`：「.claude/rules/test-disci」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:115`：「接口隔离」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:60`：「trait 往小里切」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:116`：「依赖倒置：依赖抽象，不依赖具」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:59`：「至少两个真实现才抽 trait，对拍用的模型算一」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:117`：「迪米特法则：只和直接的朋友说」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:62`：「模块内部随便链；跨模块不许穿过别人的内部」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:118`：「设计模式：工厂、策略、访问者」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:63`：「策略、访问者在封闭集合上写成 match」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:119`：「DRY，避免重复」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:64`：「重复要生成，不许手抄」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:120`：「三次重复之后再抽象」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:64`：「重复先交给生成器，不抽成吃掉穷尽性检查的通用路径」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:121`：「default: / _ =」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:59`：「default: / _ => 兜底」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:127`：「快速失败」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:68`：「快速失败」 | 落点 | 写法在第 10 行 |
| `.claude/rules/code-discipline-by-review.md:128`：「库代码不 panic，一律返」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:68`：「库代码不 panic，一律返回 Result」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:129`：「用 anyhow、Box<d」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:68`：「anyhow、Box<dyn Error>」 | 落点 | 写法在第 9、11 行 |
| `.claude/rules/code-discipline-by-review.md:130`：「确定不会失败时用 unwra」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:68`：「确定不会失败时用 unwrap()」 | 落点 | 写法在第 9、11 行 |
| `.claude/rules/code-discipline-by-review.md:131`：「防御式编程：处处检查参数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:69`：「边界上查一次，换成带证明的类型，往里不重复查」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:132`：「错误码还是异常」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:68`：「错误码还是异常」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:138`：「多写注释」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:73`：「多写注释」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:139`：「注释是失败，好代码不需要注释」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:73`：「注释是失败，好代码不需要注释」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:140`：「每个公开项都写文档注释」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:73`：「每个公开项都写文档注释」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:141`：「unsafe 块写 // S」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:73`：「// SAFETY:」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:142`：「注释掉的代码留着以后用」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:73`：「注释掉的代码留着以后用」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:143`：「在注释里记改动历史和作者」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:74`：「在注释里记改动历史和作者」 | 已登的改动 | 按需-M2：历史落点改指共用约束 |
| `.claude/rules/code-discipline-by-review.md:144`：「TODO 注释」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:73`：「TODO 注释」 | 落点 | 写法在上游 code-discipline.md 第 156 行 |
| `.claude/rules/code-discipline-by-review.md:150`：「每行不超过 80 列」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:78`：「rustfmt 的默认值说了算；名字长了就换行，」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:151`：「8 格缩进，逼人少嵌套」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:80`：「8 格缩进，逼人少嵌套」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:152`：「竖向留白、相关的挨着放」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:79`：「竖向留白、相关的挨着放」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:158`：「一个测试只写一个断言」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:84`：「一个测试一个场景；断言可以多条，每条的消息写清期」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:159`：「覆盖率达到某个比例」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:85`：「拿路径数和变异清单说话（.claude/rule」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:160`：「依赖一律 mock」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:86`：「优先用对拍的模型和真实实现」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:161`：「测试金字塔：单元测试最多」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:87`：「.claude/singlefs-ai-sop/」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:162`：「测试代码也要 DRY」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:88`：「每个测试自己读得出场景；搭建可以抽成辅助函数，断」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:163`：「不稳定的测试多跑几次就过」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:89`：「.claude/rules/test-disci」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:164`：「先写测试」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:90`：「先看着它红（.claude/rules/show」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:170`：「童子军规则：路过就顺手整理」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:94`：「顺手的整理单独提交，不混进功能提交」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:171`：「不容忍破窗」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:95`：「编译器和 clippy 的警告按错误处理」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:172`：「避免过早优化」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:96`：「避免过早优化」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:174`：「设计与流程层面的准则（设计要」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:25`：「在 .claude/rules/machine-」 | 落点 |  |

`.claude/rules/code-discipline-by-review.md` 续（从原件第 176 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/code-discipline-by-review.md:178`：「改一个行为、删一个功能、换一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:100`：「- 改一个行为、删一个功能、换一种做法时，依赖它」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:179`：「动手前按旧名字、旧路径、旧取」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:101`：「- 动手前按旧名字、旧路径、旧取值全仓搜一遍，再」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:180`：「改完按同一份清单再搜一遍，旧」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:102`：「- 改完按同一份清单再搜一遍，旧的只剩说这次改动」 | 落点 |  |
| `.claude/rules/code-discipline-by-review.md:182`：「这一条还没做成检查：本地配置」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:104`：「靠改的人现查。」 | 落点 |  |

### `.claude/rules/directory-layout.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/directory-layout.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/directory-layout.md:7`：「工程反复要跑的脚本、它们读的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:5`：「- 工程反复要跑的脚本、它们读的输入表、自证与样」 | 落点 |  |
| `.claude/rules/directory-layout.md:8`：「层级：scripts/ 下最」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:6`：「scripts/ 下最多三层，最深是 scrip」 | 落点 |  |
| `.claude/rules/directory-layout.md:9`：「scripts/ 顶层只放」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:7`：「- scripts/ 顶层只放 README.m」 | 落点 |  |
| `.claude/rules/directory-layout.md:10`：「不放结果：产物、日志、实测表」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:8`：「- 不放结果：产物、日志、实测表不进 scrip」 | 落点 |  |
| `.claude/rules/directory-layout.md:11`：「两台主机跑同一份脚本：资源分」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:9`：「- 两台主机跑同一份脚本：资源分发、编译、跑测试」 | 落点 |  |
| `.claude/rules/directory-layout.md:15`：「只有 prompts/（三方」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:13`：「- 只有 prompts/（三方论证与冻结证据）」 | 落点 |  |
| `.claude/rules/directory-layout.md:16`：「里面每一个文件都可以删。要用」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:14`：「- 里面每一个文件都可以删。要用某份旧证据或旧产」 | 落点 |  |
| `.claude/rules/directory-layout.md:17`：「反复要用的脚本、装置与输入不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:15`：「- 反复要用的脚本、装置与输入不放 resear」 | 落点 |  |
| `.claude/rules/directory-layout.md:21`：「不变量清单、登记表、钉表这类」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:19`：「- 不变量清单、登记表、钉表这类只被某一组测试读」 | 落点 |  |
| `.claude/rules/directory-layout.md:22`：「只被门禁自己读的配置（格名、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:20`：「- 只被门禁自己读的配置（格名、阶段归属、排除表」 | 落点 |  |
| `.claude/rules/directory-layout.md:26`：「门禁 code-toolin」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:24`：「门禁 code-tooling 的 direct」 | 落点 |  |
| `.claude/rules/directory-layout.md:28`：「它管不了的：脚本放进的类对不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/directory-layout.md:24`：「脚本放进的类对不对、说明文件写得准不准、有没有为」 | 落点 |  |

### `.claude/rules/implementation-first.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/implementation-first.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/implementation-first.md:4`：「共享规则在 .claude/」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/implementation-first.md:6`：「除了独立验证这类确有需要的场」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:3`：「都建在 crates/ 的实现上（四个验证用的包」 | 落点 |  |
| `.claude/rules/implementation-first.md:10`：「谈方案之前先读实现：候选臂、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:7`：「1. 谈方案之前先读实现：候选臂、改法、实验要建」 | 落点 |  |
| `.claude/rules/implementation-first.md:11`：「改法写成对实现的改动：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:8`：「并附 grep 命令与零命中，候选写成以后加在哪」 | 落点 |  |
| `.claude/rules/implementation-first.md:12`：「三方论证的背景材料必须有一行」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:9`：「列出读过的 crates/ 路径与看到的事实，标」 | 落点 |  |
| `.claude/rules/implementation-first.md:13`：「scripts/experi」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:10`：「4. scripts/experiment-ap」 | 落点 |  |
| `.claude/rules/implementation-first.md:14`：「例外：独立验证本身（chec」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:11`：「5. 例外：独立验证本身（checker 与核心」 | 落点 |  |
| `.claude/rules/implementation-first.md:18`：「门禁 doc-process」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:15`：「--check implementation-p」 | 落点 |  |
| `.claude/rules/implementation-first.md:19`：「它管不到的：读没读对、改法是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-first.md:15`：「读没读对、改法是不是真按实现写的、实验跑前登记里」 | 落点 |  |

### `.claude/rules/long-run-split.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/long-run-split.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/long-run-split.md:7`：「经统一任务服务交的任务里的每」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:5`：「经统一任务服务交的任务里的每一项：一条测试函数、」 | 落点 |  |
| `.claude/rules/long-run-split.md:8`：「提交时不指定范围那一趟同样按」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:5`：「提交时不指定范围那一趟同样按其中每一项、每一片判」 | 落点 |  |
| `.claude/rules/long-run-split.md:12`：「一项一次运行的用时超过 72」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:9`：「就要拆。用时以服务记的为准：撞了内存上限、照大一」 | 落点 |  |
| `.claude/rules/long-run-split.md:13`：「撞没撞四小时上限、结局是绿是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:9`：「撞没撞四小时上限、结局是绿是红、填没填过 --w」 | 落点 |  |
| `.claude/rules/long-run-split.md:17`：「让它跑完、出结局，结局照常用」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:13`：「1. 让它跑完、出结局，结局照常用；不为拆而取消」 | 落点 |  |
| `.claude/rules/long-run-split.md:18`：「出结局的同一轮里拆：把这一项」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:14`：「每一项能单独点名、各自预计不超过 7200 秒」 | 落点 |  |
| `.claude/rules/long-run-split.md:19`：「按参数循环的用例拆成一个带参」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:14`：「按参数循环的用例拆成一个带参数的函数加每个取值一」 | 落点 |  |
| `.claude/rules/long-run-split.md:20`：「崩溃枚举用例与崩溃放量分项按」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:14`：「崩溃枚举用例与崩溃放量分项按流、按崩溃点区间拆成」 | 落点 |  |
| `.claude/rules/long-run-split.md:21`：「变异行点名的测试收到罩着被改」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:14`：「变异行点名的测试收到罩着被改函数的那几条测试函数」 | 落点 |  |
| `.claude/rules/long-run-split.md:22`：「一片超了的，改分片计划多分几」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:14`：「一片超了的，改分片计划多分几片。」 | 落点 |  |
| `.claude/rules/long-run-split.md:23`：「拆不开的（单个场景、单条义务」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:15`：「优化前后判定逐项相同（.claude/rules」 | 落点 |  |
| `.claude/rules/long-run-split.md:24`：「缩域（--crash-exp」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:16`：「4. 缩域（--crash-expand-up-」 | 落点 |  |
| `.claude/rules/long-run-split.md:25`：「拆与优化的改动照 .clau」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:17`：「走；那一项的代码归别的会话管的，照 .claud」 | 落点 |  |
| `.claude/rules/long-run-split.md:26`：「调度记录里写一行：项名、任务」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:18`：「6. 调度记录里写一行：项名、任务号、用时、怎么」 | 落点 |  |
| `.claude/rules/long-run-split.md:30`：「只有用户点名的项豁免。豁免登」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:22`：「登记在 .claude/long-run-spl」 | 落点 |  |
| `.claude/rules/long-run-split.md:34`：「统一任务服务判用时：任务（分」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:26`：「按整条任务判（task-server/task_」 | 落点 |  |
| `.claude/rules/long-run-split.md:35`：「门禁 code-toolin」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:27`：「行有 #、理由至少 8 个字、理由里有存在的 r」 | 落点 |  |
| `.claude/rules/long-run-split.md:36`：「还靠主 agent：收到提醒」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/long-run-split.md:28`：「后按日志里各项的起止判是哪一项超了 7200 秒」 | 落点 |  |

### `.claude/rules/machine-first.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/machine-first.md:4`：「这个项目不默认照搬软件工程里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:4`：「这个项目不默认照搬软件工程里的通行准则」 | 落点 |  |
| `.claude/rules/machine-first.md:8`：「最佳实践」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:49`：「：每一条的理由重新查一遍，不因为」 | 落点 |  |
| `.claude/rules/machine-first.md:12`：「整套做法靠一条前提撑着。要反」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:67`：「。要反驳这套做法，靶子是这条前提，不是」 | 落点 |  |
| `.claude/rules/machine-first.md:16`：「200K / 500K /」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:67`：「200K / 500K / 1M 三档」 | 落点 |  |
| `.claude/rules/machine-first.md:17`：「别家的数字不写进来。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:67`：「别家的数字不写进来。」 | 落点 |  |
| `.claude/rules/machine-first.md:19`：「口径：这说的是宣称的窗口上限」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:67`：「整个窗口里准确率都一样」 | 落点 |  |
| `.claude/rules/machine-first.md:20`：「长上下文中段记不牢从多大规模」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:67`：「长上下文中段记不牢从多大规模开始碍事，这个项目没」 | 落点 |  |
| `.claude/rules/machine-first.md:22`：「它撑起的：放宽」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:69`：「DRY 靠人同步」 | 落点 |  |
| `.claude/rules/machine-first.md:23`：「它撑不起的：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:69`：「推理范围要有界」 | 落点 |  |
| `.claude/rules/machine-first.md:27`：「这张表写出什么观测会推翻这条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:69`：「观测到哪一行的现象，当场收回同一行的放宽」 | 落点 |  |
| `.claude/rules/machine-first.md:31`：「在这个项目的实际规模上，模型」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:73`：「在这个项目的实际规模上，模型漏掉了一条已经在上下」 | 落点 |  |
| `.claude/rules/machine-first.md:32`：「生成出来的重复代码和生成器对」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:74`：「生成出来的重复代码和生成器对不上」 | 落点 |  |
| `.claude/rules/machine-first.md:36`：「问它当初要解决什么问题。 说」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:51`：「1. 问它当初要解决什么问题。 说不出来就直接放」 | 落点 |  |
| `.claude/rules/machine-first.md:37`：「问那个问题今天还在不在。 当」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:52`：「2. 问那个问题今天还在不在。 当初解决」 | 落点 |  |
| `.claude/rules/machine-first.md:38`：「review 忙不过来」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:52`：「review 忙不过来」 | 落点 |  |
| `.claude/rules/machine-first.md:39`：「问它有没有第二个理由：跟人无」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:53`：「留下的（与谁读代码无关）」 | 落点 |  |
| `.claude/rules/machine-first.md:40`：「有就留下。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:53`：「有就留下。」 | 落点 |  |
| `.claude/rules/machine-first.md:41`：「改过的理由要能落成门禁。 落」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:54`：「落不成的降为建议，不写成规矩。」 | 落点 |  |
| `.claude/rules/machine-first.md:45`：「这条规则是让代码更容易被机械」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:60`：「> 这条规则是让代码更容易被机械验证，还是只让它」 | 落点 |  |
| `.claude/rules/machine-first.md:47`：「只是让人看着舒服 → 可以放」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:62`：「- 只是让人看着舒服」 | 落点 |  |
| `.claude/rules/machine-first.md:48`：「让验证更容易 → 留下。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:63`：「- 让验证更容易」 | 落点 |  |
| `.claude/rules/machine-first.md:52`：「代码怎么写（名字、分支、类型」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:56`：「逐条审过的」 | 已登的改动 | 按需-M6：改指 code-discipline-by-review.md |
| `.claude/rules/machine-first.md:53`：「连同每一条的写法，在 .cl」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:56`：「/rules/code-discipline」 | 已登的改动 | 按需-M6：改指 code-discipline-by-review.md |
| `.claude/rules/machine-first.md:54`：「设计与流程层面的准则在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:56`：「放宽的（人脑带宽的产物）」 | 落点 |  |
| `.claude/rules/machine-first.md:60`：「设计要统一、概念要少」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:80`：「设计要统一、概念要少」 | 落点 |  |
| `.claude/rules/machine-first.md:61`：「PR 要小」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:81`：「PR 要小」 | 落点 |  |
| `.claude/rules/machine-first.md:63`：「放宽的是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:83`：「为了让人装得下而做的压缩」 | 落点 |  |
| `.claude/rules/machine-first.md:64`：「见 .claude/rule」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:83`：「AI 友好 ≠ 不可读」 | 落点 |  |
| `.claude/rules/machine-first.md:68`：「不变量」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:87`：「- 不变量」 | 落点 |  |
| `.claude/rules/machine-first.md:69`：「对外格式与协议兼容」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:88`：「- 对外格式与协议兼容」 | 落点 |  |
| `.claude/rules/machine-first.md:70`：「测试可复现」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:89`：「- 测试可复现」 | 落点 |  |
| `.claude/rules/machine-first.md:71`：「故障域隔离」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:90`：「- 故障域隔离」 | 落点 |  |
| `.claude/rules/machine-first.md:72`：「可二分、可回退」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:91`：「- 可二分、可回退」 | 落点 |  |
| `.claude/rules/machine-first.md:73`：「模块边界要有明确契约」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:92`：「- 模块边界要有明确契约」 | 落点 |  |
| `.claude/rules/machine-first.md:74`：「推理范围要有界」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:93`：「- 推理范围要有界」 | 落点 |  |
| `.claude/rules/machine-first.md:78`：「.claude/single」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/machine-first.md:95`：「.claude/singlefs-ai-sop/」 | 已登的改动 | 按需-M7：改指 design-doc、kb、rules 三份上游规则 |

### `.claude/rules/mutation-sampling.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/mutation-sampling.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/mutation-sampling.md:4`：「共享规则在 .claude/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:3`：「test-discipline.md」 | 已登的改动 | 按需-M5：总纲指针改指项目的 test-discipline.md |
| `.claude/rules/mutation-sampling.md:6`：「test-disciplin」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:3`：「test-discipline.md」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:7`：「判定为等价的把等价性写成一条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:3`：「判定为等价的把等价性写成一条测试留档）」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:9`：「这里的编号共八类：第一至三类」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:5`：「那张表的三行），第四至八类」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:15`：「断言压根没覆盖那段代码」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:11`：「断言压根没覆盖那段代码」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:16`：「在所有输入上与原式同值」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:12`：「把等价性写成一条测试留档，再换一个真会改行为的变」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:17`：「在我选的取样点上恰好同值，换」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:13`：「在我选的取样点上恰好同值，换一个取样点就不同」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:21`：「取样点不敏感的变异不记成等价」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:17`：「取样点不敏感的变异不记成等价变异留档：补一个敏感」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:25`：「前三类说的都是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:21`：「取样点选窄了，单测全绿、变异也全」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:26`：「取样点选窄了，单测全绿、变异」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:21`：「取样点选窄了，单测全绿、变异也全抓，」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:28`：「判据与第三类同，只是问句换一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:21`：「问句换一个方向：单测取的那几个点，」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:29`：「覆不覆盖实验自己会跑到的全部」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:21`：「覆不覆盖实验自己会跑到的全部取值？」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:30`：「.claude/rules/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:21`：「.claude/rules/test-disci」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:31`：「断言之间的盲区；这一条管的是」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:21`：「取样点与实际取值域之间的盲区。」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:35`：「这一类连变异都没有，它出现在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:25`：「在两份算术之间：仓里对同一个物理量有两个口径，」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:36`：「写新算术的人挑了其中一个，而」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:25`：「对答案的那一臂上恰好」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:38`：「判据：把一个数写进产物之前，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:27`：「判据：把一个数写进产物之前，问一句」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:39`：「两个口径都在 ⇒ 不许自己挑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:28`：「不报这个数，把这一格留给已有的欠账。」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:40`：「只在基线那一臂上对过账 ⇒」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:29`：「- 只在基线那一臂上对过账」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:42`：「它不需要变异也会发生，scr」 | 无 | 删 | 说明句删 |
| `.claude/rules/mutation-sampling.md:46`：「对象不是变异，是判决本身：一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:33`：「是判决本身：一个实验跑前写死的判据（阈值、翻不翻」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:47`：「而那个几何 / 旋钮不是任何」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:33`：「而那个几何 / 旋钮不是任何已定条款给的。」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:49`：「判据：每个带跑前判决的计数实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:35`：「：至少在一个方向相反的几何 / 旋钮取样点上重跑」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:50`：「翻面就记」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:35`：「之前，把那条政策依赖的每个旋钮扫一遍。」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:51`：「判别力自证：把判据阈值挪到两」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:35`：「判别力自证：把判据阈值挪到两点之间，检查必须由绿」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:55`：「前六类都预设变异真的被施加了」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:39`：「变异表里一条的」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:56`：「在对应源码里命中 0 次或多」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:39`：「在对应源码里命中 0 次或多次（crates 那」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:57`：「mutate.sh 在派活之」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:40`：「派活之前逐条核锚点，锚点失效的那一条打一行 ⚓、」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:58`：「门禁 crates-muta」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:41`：「- 门禁 crates-mutation-rep」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:60`：「判据：读 mutate.sh」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:43`：「判据：读 mutate.sh 的输出之前，先确认」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:61`：「收尾没有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:43`：「那一行）就是中途退出，那一轮的」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:62`：「锚点失效那一栏不为 0 的，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:43`：「：把原文改到今天源码里逐字存在、只出现一次的那一」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:63`：「门禁形态：不跑变异，只对每张」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:45`：「不跑变异，只对每张表逐条做子串计数，命中不是 1」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:64`：「（门禁 code-sourc」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:45`：「门禁 code-source-disciplin」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:68`：「替换之后编不过的变异记」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:3`：「.claude/rules/test-disci」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:69`：「一条无效变异等于它本要证明的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:49`：「一条无效变异等于它本要证明的那个行为零变异覆盖：」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:70`：「要换一条真会改行为的补上，实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:49`：「换一条真会改行为的补上，实验页不按表里的条数写」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:72`：「门禁形态：反斜杠转义这一种成」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:51`：「反斜杠转义这一种成因由门禁 code-sourc」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:73`：「出现别的反斜杠转义就判红。跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:52`：「门禁 crates-mutation-repla」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:74`：「分成两栏、各带各的出路：无效」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:52`：「两栏、各带各的出路：无效先按」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:75`：「读 crates-mutat」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:52`：「crates-mutation-replay」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:79`：「改完格式常量，重跑受影响的每」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:58`：「mutate.sh 没有三数汇总行，按每条结果行」 | 落点 | 一行拆成第 56 至 69 行；谁跑、何时跑改成指 verification.md |
| `.claude/rules/mutation-sampling.md:80`：「无效那一栏变多，等于有断言被」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:56`：「无效那一栏变多，等于有断言被关掉了。」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:84`：「一条变异没被抓时，问一句：换」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:73`：「> 一条变异没被抓时，问一句：换一个取样点，它还」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:86`：「答不上来 ⇒ 去找一个会让它」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:75`：「找得到就是取样点不敏感，补那一格。」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:87`：「确认在所有输入上同值 ⇒ 才」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:76`：「test-discipline.md」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:89`：「怎么找那个取样点：变异改的若」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:78`：「出现在除法或取整里的常量，就去解」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:90`：「让两个式子跨过整数边界」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:78`：「让两个式子跨过整数边界」 | 落点 |  |
| `.claude/rules/mutation-sampling.md:92`：「这条只在有变异 harnes」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:80`：「判据落在 scripts/mutation/mu」 | 落点 | 「只在有变异 harness 的仓里生效」半句删 |
| `.claude/rules/mutation-sampling.md:93`：「一个测试都没红」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/mutation-sampling.md:80`：「一个测试都没红」 | 落点 |  |

### `.claude/rules/path-moves.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/path-moves.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/path-moves.md:5`：「.claude/rules/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:3`：「.claude/rules/evidence-d」 | 落点 |  |
| `.claude/rules/path-moves.md:6`：「文件与目录的路径不在冻结范围」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:3`：「research/prompts/、resear」 | 落点 |  |
| `.claude/rules/path-moves.md:10`：「门禁兜的只是链接与文档里的路」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:55`：「门禁 doc-registries 的 gove」 | 落点 | 区间第 55 至 57 行 |
| `.claude/rules/path-moves.md:12`：「搬之前先把旧路径的全部写法搜」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:7`：「…)）、拆成几段的写法（"<上一层>", "<旧」 | 落点 |  |
| `.claude/rules/path-moves.md:13`：「git mv 搬。在 .cl」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:8`：「.claude/gate.d/path-move」 | 落点 |  |
| `.claude/rules/path-moves.md:14`：「整目录搬，搬前搬后各跑一次」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:9`：「- 整目录搬，搬前搬后各跑一次 python3」 | 落点 |  |
| `.claude/rules/path-moves.md:15`：「登记行只在搬迁提交之前对门禁」 | `.claude/gate.d/lib-lag-baseline.py:56`：「不在 gate 基准那一版的登记表里，」 | 脚本文件头 | 合稿 path-moves.md 第 13 行留指针，细则在脚本文件头 |
| `.claude/rules/path-moves.md:16`：「门禁照此刻生效的登记换算四样」 | `.claude/gate.d/lib-lag-baseline.py:60`：「scripts/agents/snapshot-」 | 脚本文件头 | 合稿 path-moves.md 第 13 行留指针，细则在脚本文件头 |
| `.claude/rules/path-moves.md:17`：「改动范围：与基准只差仓根起的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:12`：「.claude/agents/、.claude/」 | 落点 | 类的名单留在规则；前缀写法细则在脚本文件头（第 67 至 84 行） |
| `.claude/rules/path-moves.md:18`：「新增行：同一处被删的旧行照登」 | `.claude/gate.d/lib-lag-baseline.py:85`：「加条数不等的块一行都不剔；整份」 | 脚本文件头 | 合稿第 13 行留指针 |
| `.claude/rules/path-moves.md:19`：「不许串：此刻生效的几行之间，」 | `.claude/gate.d/lib-lag-baseline.py:91`：「行之间，任一行的旧前缀或新前缀不许等于、包含或落」 | 脚本文件头 | 合稿第 10、13 行 |
| `.claude/rules/path-moves.md:20`：「登记行要有搬迁实据：基准里旧」 | `.claude/gate.d/lib-lag-baseline.py:93`：「新前缀在基准里没有文件；旧前缀此刻不在磁盘上」 | 脚本文件头 | 合稿第 13 行留指针 |
| `.claude/rules/path-moves.md:21`：「新行必须生效：不在基准那一版」 | `.claude/gate.d/lib-lag-baseline.py:64`：「或新前缀下有基准里本来就在的文件」 | 脚本文件头 | 合稿第 8 行「在同一次改动里」、第 13 行指针 |
| `.claude/rules/path-moves.md:22`：「把命令里的目录换成另一个已有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:11`：「- 把命令里的目录换成另一个已有的目录、换了实现」 | 落点 |  |
| `.claude/rules/path-moves.md:23`：「按第 1 步那份清单逐处改成」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:14`：「清单逐处改成新路径。Markdown 链接」 | 落点 |  |
| `.claude/rules/path-moves.md:24`：「改完按同一份清单再搜一遍，除」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:15`：「：Cargo.toml 的 path = 按 c」 | 落点 |  |
| `.claude/rules/path-moves.md:25`：「说搬迁这件事本身的那一行保留」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:16`：「人判，判过该留旧路径的（将来落点、句子本身说它不」 | 落点 |  |
| `.claude/rules/path-moves.md:26`：「判决或核对表里记过的 sha」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:17`：「因此对不上：只差路径的不算证据被改，在那一轮的记」 | 落点 |  |
| `.claude/rules/path-moves.md:27`：「门禁、钩子与脚本的自检样本和」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:18`：「7. 门禁、钩子与脚本的自检样本和用例跟着搬：被」 | 落点 |  |
| `.claude/rules/path-moves.md:31`：「同一类随里程碑一份一份长出来」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:22`：「同一类随里程碑一份一份长出来的 kb 文件（字节」 | 落点 |  |
| `.claude/rules/path-moves.md:35`：「路径搬迁之外的另一类是术语改」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:26`：「一次做完：用 scripts/kb/sweep-」 | 落点 |  |
| `.claude/rules/path-moves.md:39`：「编号的简称」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:30`：「kb 里由 doc-lint、kb 之外的 D」 | 落点 |  |
| `.claude/rules/path-moves.md:40`：「简称的宽度上限 48」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:31`：「简称的宽度上限 48」 | 落点 |  |
| `.claude/rules/path-moves.md:41`：「文件名与指向它的链接」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:32`：「共享 gate.sh 的」 | 落点 |  |
| `.claude/rules/path-moves.md:42`：「决策分项清单」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:33`：「跑 SINGLEFS_RUN_SCOPE='种类」 | 落点 |  |
| `.claude/rules/path-moves.md:43`：「实验页的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:34`：「门禁 doc-experiments 的 dec」 | 落点 |  |
| `.claude/rules/path-moves.md:45`：「留存产物分两类，别一把梭：装」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:36`：「登记进 .claude/term-rename-」 | 落点 |  |
| `.claude/rules/path-moves.md:47`：「历史类文件同样换名（.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:38`：「这件事本身的）。留旧名的登记进 .claude/」 | 落点 |  |
| `.claude/rules/path-moves.md:49`：「英文标识符与中文术语可以分批」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:40`：「英文标识符与中文术语可以分批改：只改中文时门禁」 | 落点 |  |
| `.claude/rules/path-moves.md:53`：「候选、臂、方案、命题、提问编」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:44`：「候选、臂、方案、命题、提问编号有了变体，给变体起」 | 落点 |  |
| `.claude/rules/path-moves.md:55`：「先定这个名字属于哪一族、那一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:46`：「1. 先定这个名字属于哪一族、那一族在哪几份文件」 | 落点 |  |
| `.claude/rules/path-moves.md:56`：「在那一族里取下一个没用过的号」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:47`：「2. 在那一族里取下一个没用过的号；取号会造成误」 | 落点 |  |
| `.claude/rules/path-moves.md:57`：「撞号先查：新名字在那一族的全」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:48`：「新名字在那一族的全部文件里零命中。」 | 落点 |  |
| `.claude/rules/path-moves.md:58`：「变体与原名的关系写在新名字的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:49`：「4. 变体与原名的关系写在新名字的定义那一句里，」 | 落点 |  |
| `.claude/rules/path-moves.md:60`：「门禁 doc-text 的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:51`：「门禁 doc-text 的 prime-mark」 | 落点 |  |
| `.claude/rules/path-moves.md:64`：「搬迁这一半：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:55`：「后继，research/ 下进过 git 的放行」 | 落点 |  |
| `.claude/rules/path-moves.md:65`：「搬迁登记表 .claude/」 | `.claude/gate.d/lib-lag-baseline.py:94`：「被判的根不在 git 仓里、」 | 脚本文件头 | 合稿第 56 行留指针 |
| `.claude/rules/path-moves.md:67`：「门禁关不住的（改动范围里这几」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:13`：「门禁关不住的」 | 落点 |  |
| `.claude/rules/path-moves.md:68`：「gate 基准不总是 HEA」 | `.claude/gate.d/lib-lag-baseline.py:62`：「此时删掉一份搬过去的文件、在旧前缀下重用旧名字、」 | 脚本文件头 | 合稿第 13 行留指针 |
| `.claude/rules/path-moves.md:69`：「钉值一侧按全部登记行逐次倒换」 | `.claude/gate.d/lib-lag-baseline.py:97`：「钉住的文件改指那一份早就在的实现，倒换倒得回钉值」 | 脚本文件头 | 合稿第 13 行留指针 |
| `.claude/rules/path-moves.md:70`：「${变量:-旧前缀/…} 这」 | `.claude/gate.d/lib-lag-baseline.py:99`：「照算改动」 | 脚本文件头 | 合稿第 13 行留指针 |
| `.claude/rules/path-moves.md:71`：「新前缀下基准里已有占位文件的」 | `.claude/gate.d/lib-lag-baseline.py:100`：「换了实现的不登记：落点逐份在、被」 | 脚本文件头 | 合稿第 13 行留指针 |
| `.claude/rules/path-moves.md:73`：「术语改名那一半没有门禁复查：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:58`：「登记在 [.claude/kb/term-ren」 | 落点 |  |

### `.claude/rules/test-discipline.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/test-discipline.md:6`：「跑至少 5 轮。要说」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:6`：「跑至少 5 轮。」 | 落点 |  |
| `.claude/rules/test-discipline.md:7`：「两边都不满足，就如实说」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:6`：「，不下结论。」 | 落点 |  |
| `.claude/rules/test-discipline.md:9`：「前提是观测本身有变化：真实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:8`：「前提是观测本身有变化：真实 I/O、并发、时序、」 | 落点 |  |
| `.claude/rules/test-discipline.md:10`：「一样都不占的是确定性模型，只」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:8`：「跑多少轮说明的都只是没有隐藏状态，不是统计上稳定」 | 落点 |  |
| `.claude/rules/test-discipline.md:11`：「证据强度靠变异测试和钉死绝对」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:8`：「证据强度靠变异测试和钉死绝对值的断言，不靠轮数。」 | 落点 |  |
| `.claude/rules/test-discipline.md:15`：「指标读出来是空的，要重试，而」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:12`：「重试，而且这一轮作废，」 | 落点 |  |
| `.claude/rules/test-discipline.md:19`：「不许预设结论，设计阶段也不许」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:16`：「参考任何已有结论：这个项目上一轮的结论、」 | 落点 |  |
| `.claude/rules/test-discipline.md:20`：「别处的定论、自己的直觉，一概」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:16`：「别处的定论、自己的直觉」 | 落点 |  |
| `.claude/rules/test-discipline.md:22`：「该读的是定义，不是答案。 被」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:17`：「被测对象的口径、参数、语义」 | 落点 |  |
| `.claude/rules/test-discipline.md:23`：「（见 .claude/rul」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:17`：「它定了没，和它到底怎么定义的，是两个问题」 | 落点 |  |
| `.claude/rules/test-discipline.md:24`：「上一轮测出来是某个值」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:17`：「上一轮测出来是某个值」 | 落点 |  |
| `.claude/rules/test-discipline.md:26`：「做法：判据、阈值、作废条款都」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:18`：「判据、阈值、作废条款都在跑之前写死，写完再去看已」 | 落点 |  |
| `.claude/rules/test-discipline.md:27`：「两边对不上就如实并列，不许回」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:18`：「两边对不上就如实并列，不」 | 落点 |  |
| `.claude/rules/test-discipline.md:29`：「阳性对照的答案，不能放在被测」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:19`：「一方读得到的地方。」 | 落点 |  |
| `.claude/rules/test-discipline.md:30`：「被测的是一个会读仓的 age」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:19`：「会读仓的 agent 时，答案不写进它要跑的脚本」 | 落点 |  |
| `.claude/rules/test-discipline.md:31`：「只存哈希也不够：候选空间小、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:19`：「又不加盐的短哈希算明文。」 | 落点 |  |
| `.claude/rules/test-discipline.md:32`：「当盲测用的答案不进被测方能读」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:19`：「盲测用的答案不进被测方能读的仓，或者换一段它没见」 | 落点 |  |
| `.claude/rules/test-discipline.md:36`：「没测出效应」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:23`：「一律写成实现有 bug。」 | 落点 |  |
| `.claude/rules/test-discipline.md:40`：「阳性对照：效应在文献里确凿存」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:27`：「阳性对照：效应在文献里确凿存在的那个基线」 | 落点 |  |
| `.claude/rules/test-discipline.md:41`：「真实基线：这个项目实际打算实」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:28`：「真实基线：这个项目实际打算实现的那个方案」 | 落点 |  |
| `.claude/rules/test-discipline.md:43`：「每写一条失败条款，紧接着写一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:30`：「每写一条失败条款，紧接着写一句」 | 落点 |  |
| `.claude/rules/test-discipline.md:44`：「写不出那句观测，这条条款要么」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:30`：「写不出那句观测」 | 落点 |  |
| `.claude/rules/test-discipline.md:48`：「不把两个量合成一个 true」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:34`：「- 不把两个量合成一个 true / false」 | 落点 |  |
| `.claude/rules/test-discipline.md:49`：「门槛不能是被测臂的定义：写门」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:35`：「- 门槛不能是被测臂的定义：写门槛之前把被测臂的」 | 落点 |  |
| `.claude/rules/test-discipline.md:53`：「对照那段循环必须遍历所有臂，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:39`：「加臂时连对照那一段一起改。」 | 落点 |  |
| `.claude/rules/test-discipline.md:57`：「每写一条互比断言，旁边就要有」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:43`：「要有一条把绝对值钉死的断言」 | 落点 |  |
| `.claude/rules/test-discipline.md:58`：「三条臂的开销相等」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:43`：「三条臂的开销相等」 | 落点 |  |
| `.claude/rules/test-discipline.md:60`：「做法：写完一条互比断言就问自」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:43`：「条臂一起错了，谁会发现」 | 落点 |  |
| `.claude/rules/test-discipline.md:61`：「答不上来就补一条钉绝对值的。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:43`：「答不上来就补一条钉绝对值的。」 | 落点 |  |
| `.claude/rules/test-discipline.md:65`：「一个量被某条条款当停机 /」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:47`：「一个量被某条条款当停机 / 起跑 / 准入谓词的」 | 落点 |  |
| `.claude/rules/test-discipline.md:66`：「吃光初始供给之后的峰值、为正」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:47`：「吃光初始供给之后的峰值、为正的轮数、期末值；」 | 落点 |  |
| `.claude/rules/test-discipline.md:67`：「只报期末值的，正文里不许写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:47`：「这类说轨迹的词。」 | 落点 |  |
| `.claude/rules/test-discipline.md:71`：「跑前登记只写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:51`：「写这个量为哪个决定而量、它取什么值会让那个决定翻」 | 落点 |  |
| `.claude/rules/test-discipline.md:72`：「写不出这三样的量不登记。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:51`：「写不出这三样的量不登记。」 | 落点 |  |
| `.claude/rules/test-discipline.md:74`：「选择与结论分开写。 交选择的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:53`：「每个选择的候选、什么观测会让选择翻面、量到哪算够」 | 落点 |  |
| `.claude/rules/test-discipline.md:75`：「它不带倾向、不带数，可以原样」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:53`：「它不带倾向、不带数，可以原样交给设计实验的一方。」 | 落点 |  |
| `.claude/rules/test-discipline.md:76`：「登记里每个量对应清单的一行，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:54`：「- 登记里每个量对应清单的一行，对不上任何一行的」 | 落点 |  |
| `.claude/rules/test-discipline.md:77`：「每段交回附一张表，逐行写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:55`：「已够判 / 还差什么 / 剩下的量能不能让它翻面」 | 落点 |  |
| `.claude/rules/test-discipline.md:78`：「够判之后没跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:56`：「是合法结局，不是欠账，实验状态里」 | 落点 |  |
| `.claude/rules/test-discipline.md:82`：「变异只证明」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:60`：「凡是写了断言的代码都被变异测过」 | 落点 |  |
| `.claude/rules/test-discipline.md:84`：「，只能写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:60`：「<条数> 条变异全部被抓」 | 落点 |  |
| `.claude/rules/test-discipline.md:85`：「这条结论的算术是从哪来的，那」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:60`：「，那部分有断言盯着吗？」 | 落点 |  |
| `.claude/rules/test-discipline.md:87`：「两个坑会让变异悄悄失效：」 | 无 | 删 | 引导句删，两条都在 |
| `.claude/rules/test-discipline.md:89`：「常量断言写加法，别写减法：写」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:62`：「写减法：写 assert_eq!(total_s」 | 落点 |  |
| `.claude/rules/test-discipline.md:90`：「等价变异不算盲区，但要分开记」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:63`：「2. 等价变异不算盲区，」 | 落点 |  |
| `.claude/rules/test-discipline.md:91`：「把等价性写成一条测试留档，再」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:63`：「把等价性写成一条测试留档，再换一个真会改行为的变」 | 落点 |  |

`.claude/rules/test-discipline.md` 续（从原件第 93 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/test-discipline.md:95`：「下断言之前先确认这个检查分得」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:69`：「下断言之前先确认这个检查分得出差别：验」 | 落点 |  |
| `.claude/rules/test-discipline.md:96`：「同时确认扫描真的读到了块。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:69`：「同时确认扫描真的读到了块。」 | 落点 |  |
| `.claude/rules/test-discipline.md:98`：「写完一个检查就问：被测对象真」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:69`：「写完一个检查就问：被测对象真坏了，它会不会红？答」 | 落点 |  |
| `.claude/rules/test-discipline.md:100`：「比较前后两个值的核对（改前与」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:70`：「比较前后两个值的核对（改前与改后、两次运行、两份」 | 落点 |  |
| `.claude/rules/test-discipline.md:104`：「自证分了格、能点名单跑的，每」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:8`：「自证分了格、能点名单跑的，每一格」 | 落点 | 挪到 test-script-discipline.md；test-discipline.md 第 71 行留指针 |
| `.claude/rules/test-discipline.md:105`：「失败句里不写失败行起头的那种」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:41`：「- 失败句里不写失败行起头的那种形态（例」 | 落点 | 挪到 test-script-discipline.md |
| `.claude/rules/test-discipline.md:109`：「测试里的资源、配置与规模照被」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:75`：「- 测试里的资源、配置与规模照被测对象真会遇到的」 | 落点 |  |
| `.claude/rules/test-discipline.md:110`：「用例要证明的行为必须是真环境」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:76`：「是真环境里会发生的；真环境里达不到的条件（凑不齐」 | 落点 |  |
| `.claude/rules/test-discipline.md:111`：「要验的条件在测试里造不出来，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:77`：「，不拿一个造得出、」 | 落点 |  |
| `.claude/rules/test-discipline.md:113`：「这一条还没做成检查：取值是不」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:79`：「，靠 review。」 | 落点 |  |
| `.claude/rules/test-discipline.md:117`：「检查一份资源（显存、内存、句」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:83`：「检查一份资源（显存、内存、句柄、磁盘配额）够不够」 | 落点 |  |
| `.claude/rules/test-discipline.md:119`：「这一条还没做成检查，靠 re」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:85`：「，靠 review。」 | 落点 |  |
| `.claude/rules/test-discipline.md:123`：「这条路径是对的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:89`：「这条路径是对的」 | 落点 |  |
| `.claude/rules/test-discipline.md:124`：「得拿出读数证明那条路径真的跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:89`：「拿出读数证明那条路径真的跑过。」 | 落点 |  |
| `.claude/rules/test-discipline.md:126`：「扫一批格、数某一支出了几次的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-discipline.md:91`：「扫一批格、数某一支出了几次的探针，每个计数同时报」 | 落点 |  |

### `.claude/rules/test-script-discipline.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/test-script-discipline.md:4`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/test-script-discipline.md:6`：「管项目自己的验证代码：单元与」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:4`：「管项目自己的验证代码：单元与集成测试、穷举与长时」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:7`：「它们同样受 .claude/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:4`：「、.claude/rules/test-disc」 | 已登的改动 | 按需-M4：指针改指 show-me-test-by-review.md |
| `.claude/rules/test-script-discipline.md:11`：「判据：点名单跑一条用例，与它」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:8`：「判据：点名单跑一条用例，与它在不指定范围那一趟里」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:13`：「一条用例一个场景。按参数循环」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:10`：「- 一条用例一个场景。按参数循环、每一轮重建被测」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:14`：「不依赖跑序，不读别的用例留下」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:11`：「、自己删（.claude/singlefs-ai」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:15`：「随机输入带种子。红了打印种子」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:12`：「红了打印种子，凭种子能单独复现那一条。」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:16`：「用例名写场景和预期（.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:13`：「- 用例名写场景和预期（.claude/sing」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:17`：「红了只重跑红的那几条：入口脚」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:14`：「（.claude/rules/command-s」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:23`：「不指定范围：全部项，照」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:20`：「判定可复用：按输入存，按前缀剪」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:24`：「--list-items」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:21`：「逐行列出可点名的项，不编译、不跑」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:25`：「--item <项>」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:22`：「指定范围：只跑点名的项，给几次取并集」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:26`：「项名写错、--item 后面」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:23`：「项名写错、--item 后面缺项名」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:28`：「不指定范围与指定范围是同一套」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:25`：「只跑一部分状态或一部分种子的（抽样、缩域）不是不」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:29`：「什么时候跑哪一种：不指定范围」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:26`：「对拍（加速路径与参照路径、多线程与单线程、优化前」 | 落点 | 谁跑、何时跑改成指 verification.md 第 32 至 33 行 |
| `.claude/rules/test-script-discipline.md:30`：「门禁阶段不是入口脚本：gat」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:27`：「- 门禁阶段不是入口脚本：gate.sh 不带参」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:31`：「不用测试框架自带的标记（#[」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:28`：「- 不用测试框架自带的标记（#[ignore]」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:32`：「规模变量（种子数、展开深度这」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:29`：「写出它的取值；缩过规模的那一趟不当门禁通过、不当」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:33`：「整个脚本都离不开的工具、设备」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:30`：「写成 run-condition（.claude」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:34`：「每项结束打一行：项名、退出码」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:31`：「，不并进总计，成功那句点名是哪几项；两种都不写」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:35`：「退出码 0 跑到的全绿（有项」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:32`：「也是 0）、1 有红、2 用法错、77 一项都没」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:39`：「判据：测试脚本与它用的判法，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:36`：「.claude/rules/test-disci」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:41`：「写成能被测的形态：判法写成纯」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:38`：「- 写成能被测的形态：判法写成纯函数，项的驱动、」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:42`：「自证用 --selftest」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:39`：「每一种判定（绿、红、本次未跑、用法错）至少喂一个」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:43`：「每一处会判红的分支，配一个弄」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:40`：「变异进入库的变异清单（.claude/rules」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:44`：「自证接进门禁，改了测试脚本或」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:42`：「一律用假驱动，不起编译与重活；改了谁，主 age」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:45`：「自证同样守」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:8`：「判据：点名单跑一条用例，与它在不」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:49`：「被测代码的每个测试目标（库、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:46`：「- 被测代码的每个测试目标（库、集成测试、可执行」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:50`：「每一种判法的小输入用例（同判」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:47`：「、剪枝自证、多线程与单线程的对拍、优化前后的对拍」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:51`：「挂编译开关（特性、cfg 这」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:48`：「- 挂编译开关（特性、cfg 这类）的测试，不指」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:55`：「每一份判定按它依赖的全部输入」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:52`：「- 每一份判定按它依赖的全部输入算键存下：被测代」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:56`：「复用只看键，不设时限：输入没」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:52`：「，不设时限」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:57`：「判定库不是这一趟的输出：输出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:53`：「- 判定库不是这一趟的输出：输出照 .claud」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:58`：「状态空间按共享前缀组织：几条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:54`：「- 状态空间按共享前缀组织：几条流、几个项共有的」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:59`：「剪掉的只能是两种：键相同、已」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:55`：「- 剪掉的只能是两种：键相同、已经判过的；与已判」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:60`：「按登记整批不判的一类不算剪枝」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:55`：「按登记整批不判的一类不算剪枝：登记表一行一类、写」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:61`：「剪枝要自证：在小规模输入上关」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:56`：「- 剪枝要自证：在小规模输入上关掉剪枝与复用跑一」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:62`：「复用也要自证，凡声称按判定入」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:57`：「同一份输入跑两趟，第二趟现判 0、复用数等于第一」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:63`：「一块判完才落盘，写入是原子的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:58`：「- 一块判完才落盘，写入是原子的；被打断后重跑同」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:64`：「成功那句报」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:59`：「现判 <个数>、复用 <个数>」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:65`：「整个脚本的输入没变就不重跑，」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:60`：「- 整个脚本的输入没变就不重跑，写成 admis」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:69`：「穷举、长时间随机、真设备这类」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:64`：「- 穷举、长时间随机、真设备这类用例用优化构建跑」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:70`：「一趟只编译一次：这一趟的几个」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:65`：「- 一趟只编译一次：这一趟的几个项共用这一趟的一」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:71`：「几份仓副本不共用一个编译目录」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:66`：「- 几份仓副本不共用一个编译目录，每份各给一个；」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:72`：「热路径上不做与判定无关的事：」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:67`：「- 热路径上不做与判定无关的事：格式化、逐条日志」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:73`：「进程内互相独立的单位按区间切」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:68`：「取序号最小的；多线程与单线程的对拍逐字相同，能用」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:74`：「自证同样并行：.claude」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:69`：「）；整份不能并行的，在文件头写一行 # self」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:75`：「单项跑得超过项目定的进度间隔」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:70`：「，每过一个间隔至少打一行进度：每片报区间与已判的」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:76`：「优化不许换掉判据：优化前后在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:71`：「- 优化不许换掉判据：优化前后在同一组输入上判定」 | 落点 |  |

`.claude/rules/test-script-discipline.md` 续（从原件第 78 行的节起）：

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/test-script-discipline.md:80`：「哪几类计算走加速硬件（GPU」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:75`：「用哪几台机器、哪几张卡、各给多少额度由统一任务服」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:81`：「配置开了的硬件或机器是整个脚」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:76`：「- 配置开了的硬件或机器是整个脚本离不开的，写成」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:82`：「加速路径留一份 CPU 参照」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:77`：「- 加速路径留一份 CPU 参照实现，两边在同一」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:83`：「加速路径同样先证明会红：往加」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:78`：「- 加速路径同样先证明会红：往加速那一侧塞一个已」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:87`：「门禁每出一个红，把触发它的输」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:82`：「- 门禁每出一个红，把触发它的输入钉成能用指定范」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:88`：「钉之前能缩就把输入缩到仍然红」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:83`：「原样钉。钉完核一次：在出红的那份代码上，钉住的用」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:89`：「项目有一张钉表，每钉一条登记」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:84`：「每钉一条登记一行：出红的那一行整行抄下、带产物文」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:90`：「门禁产物里的每一个红，在钉表」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:84`：「在钉表里都找得到对应的那一行，才算关掉。」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:94`：「还没有一条做成了共享检查。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:88`：「项目可以在登记的门禁目录（默认 .claude/」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:95`：「拿副本编的脚本核没核改过的包」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:88`：「拿副本编的脚本核没核改过的包重编了、入口脚本的参」 | 落点 |  |
| `.claude/rules/test-script-discipline.md:97`：「靠 review 的：自证覆」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/test-script-discipline.md:90`：「靠 review 的：自证覆没覆盖全每一种判定、」 | 落点 |  |

### `.claude/rules/timer-only-remind.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/rules/timer-only-remind.md:3`：「这是 singlefs 的项」 | 无 | 删 | 开头声明删（用户定第 11 条） |
| `.claude/rules/timer-only-remind.md:5`：「本工程设定时（到点提醒、周期」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:3`：「设定时（到点提醒、周期检查、过一阵再回来看一眼）」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:9`：「起：bash scripts」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:7`：「- 起：bash scripts/agents/」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:10`：「命令写法、到点之后做什么、关」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:8`：「- 命令写法、到点之后做什么、关掉与列出：.cl」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:11`：「子 agent 起不了后台任」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:9`：「- 子 agent 起不了后台任务：要定时的活写」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:15`：「工具 CronCreate、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:13`：「- 工具 CronCreate、Schedule」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:16`：「Bash 与 Monitor」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:14`：「- Bash 与 Monitor 里只为定时而写」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:17`：「crontab、at、bat」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:15`：「、at、batch、带定时属性的 systemd」 | 已登的改动 | 按需-M16：补 crontab -l 例外 |
| `.claude/rules/timer-only-remind.md:19`：「.claude/hooks/」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:17`：「.claude/hooks/timer-only」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:23`：「等一个进程结束（until」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:21`：「等一个进程结束（until ! kill -0」 | 已登的改动 | 常驻-9：限定为定时闸的拒绝范围 |
| `.claude/rules/timer-only-remind.md:27`：「门禁 code-toolin」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:25`：「门禁 code-tooling 的 agent-」 | 落点 |  |
| `.claude/rules/timer-only-remind.md:29`：「它管不了的：提醒内容有没有点」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:25`：「提醒内容有没有点名该跑的检查、变量里拼出来的命令」 | 落点 |  |

### `.claude/skills/crash-test/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/crash-test/SKILL.md:8`：「规则在 .claude/ru」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:8`：「规则在 .claude/rules/verifi」 | 已登的改动 | 按需-M10：两条路径写全 |
| `.claude/skills/crash-test/SKILL.md:14`：「crates/singlef」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:12`：「与门禁 qemu-device-streams、」 | 落点 | 表改成一段 |
| `.claude/skills/crash-test/SKILL.md:20`：「崩溃点重放」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:18`：「.claude/gate.d/crash-poi」 | 已登的改动 | 按需-M11：各格判什么改成指针；话在合稿 verification-reference.md 第 19、146 行 |
| `.claude/skills/crash-test/SKILL.md:21`：「模型对拍」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:19`：「.claude/gate.d/model-dif」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:22`：「QEMU 真设备」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:20`：「.claude/gate.d/qemu-devi」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:23`：「LKMM 内存序」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:21`：「.claude/gate.d/lkmm.sh，逻」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:24`：「Verus 形式化验证」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:22`：「.claude/gate.d/formal-ve」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:26`：「阶段头部写 # gate-c」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:24`：「阶段头部写 # gate-covers: <键>」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:27`：「这个阶段这一轮跑了且通过，g」 | `.claude/singlefs-ai-sop/rules/show-me-test.md:25`：「这一轮跑了且通过，gate.sh」 | 上游 | 合稿 SKILL.md 第 25 行留指针 |
| `.claude/skills/crash-test/SKILL.md:28`：「覆盖只说明」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:25`：「；覆盖到多大范围，看那个阶段自己的说明与登记表第」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:29`：「这一轮无对象可判的阶段退出码」 | `.claude/singlefs-ai-sop/rules/show-me-test.md:32`：「，不记通过，也不算覆盖」 | 上游 | 合稿 SKILL.md 第 25 行留指针 |
| `.claude/skills/crash-test/SKILL.md:33`：「写在 crates/sing」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:29`：「登记进 .claude/gate.d/stage」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:37`：「没复现问题」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:33`：「看门禁 crash-point-replay 的」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:38`：「checker 不报错，也可」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:33`：「先看 crates/singlefs-check」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:39`：「判定结果读不到，这一轮就作废」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:34`：「- 判定结果读不到」 | 落点 |  |
| `.claude/skills/crash-test/SKILL.md:40`：「谁在什么时候跑、带什么前缀，」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:35`：「- 谁在什么时候跑、带什么前缀，照 .claud」 | 落点 |  |

### `.claude/skills/decide/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/decide/SKILL.md:6`：「正文在共享层，读它：.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/decide/SKILL.md:6`：「正文在共享层，读它：.claude/single」 | 落点 |  |
| `.claude/skills/decide/SKILL.md:8`：「不要把正文抄到这里。」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/decide/SKILL.md:6`：「把正文抄到这里。」 | 落点 |  |
| `.claude/skills/decide/SKILL.md:12`：「共享正文的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/decide/SKILL.md:10`：「一条决策一个文件（.claude/kb/deci」 | 落点 |  |

### `.claude/skills/gate/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/gate/SKILL.md:6`：「正文在共享层，读它：.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/SKILL.md:6`：「正文在共享层，读它：.claude/single」 | 落点 |  |
| `.claude/skills/gate/SKILL.md:8`：「不要把正文抄到这里。」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/SKILL.md:6`：「把正文抄到这里。」 | 落点 |  |
| `.claude/skills/gate/SKILL.md:12`：「gate.sh 与 chec」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/gate/SKILL.md:10`：「由主 agent 照 .claude/skill」 | 已登的改动 | 按需-M13、M14 |

### `.claude/skills/halt/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/halt/SKILL.md:6`：「正文在共享层，读它：.cla」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/halt/SKILL.md:6`：「正文在共享层，读它：.claude/single」 | 落点 |  |
| `.claude/skills/halt/SKILL.md:8`：「不要把正文抄到这里。 正文只」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/halt/SKILL.md:6`：「把正文抄到这里。」 | 落点 | 理由句删 |

### `.claude/skills/queue-health/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/queue-health/SKILL.md:8`：「主 agent 在 Bash」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:8`：「主 agent 在 Bash 里直接跑，不经 t」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:12`：「全部在跑的任务」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:12`：「python3 scripts/hosts/qu」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:13`：「点名的任务」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:13`：「python3 scripts/hosts/qu」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:14`：「一段时间里的尖峰」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:14`：「python3 scripts/hosts/qu」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:16`：「合计的口径：逐进程读 /pr」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:16`：「写时复制页与共享内存按映射它的进程数分摊，整棵树」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:17`：「每条任务每台一行：号、第几次」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:17`：「- 每条任务每台一行：号、第几次、哪台、合计、进」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:18`：「退出码：0 都对得上；1 有」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:18`：「- 退出码：0 都对得上；1 有要看的结论；2」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:19`：「起头，逐条判据在」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:19`：「起头，逐条判据在」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:25`：「此刻找不到进程」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:26`：「不拿别的数顶。核号；task-server/ta」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:26`：「现算与 cgroup 对不上」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:27`：「：有，进程合计偏大是共享页各算一份，以 cgro」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:27`：「占用贴近上限」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:28`：「不用手动处置，撞了由服务放大重排；连着撞的核代码」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:28`：「现算合计超过在跑上限之和」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:29`：「的进程（共享页各算一份），再对着 slice a」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:29`：「现算合计占在跑上限之和 N%」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:30`：「只是比例，不用处置，也不据此判上限记得对不对；每」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:30`：「读不到 smaps_roll」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:31`：「只提醒：这几个进程按 RssAnon + Rss」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:31`：「占用都在上限以内、现算与 c」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:32`：「占用都在上限以内、现算与 cgroup 对得上」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:32`：「第二台那一行写着原因，修好再」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:33`：「第二台那一行写着原因，修好再跑；读不到的那一台不」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:33`：「此刻没有在跑任务的进程」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:34`：「此刻没有在跑任务的进程」 | 落点 |  |
| `.claude/skills/queue-health/SKILL.md:35`：「一次现算漏尖峰，要看尖峰用」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/queue-health/SKILL.md:20`：「看尖峰用 --watch。排队原因看 task-」 | 落点 |  |

### `.claude/skills/remind/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/remind/SKILL.md:6`：「本工程设定时只有一个办法：b」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:6`：「.claude/hooks/timer-only」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:10`：「要每隔一段时间跑一次检查或报」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:10`：「每隔一段时间跑一次检查或报一次进度（例：每半小时」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:11`：「要过一阵再回来看一眼某件事（」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:11`：「过一阵再回来看一眼某件事（例：二十分钟后看长命令」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:12`：「等一个进程结束、盯日志里的一」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:21`：「until ! kill -0 <pid>; d」 | 落点 | 技能第 6 行留指针 |
| `.claude/skills/remind/SKILL.md:16`：「全部用 Bash 的 run」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:15`：「全部用 Bash 的 run_in_backgr」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:20`：「循环提醒」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:19`：「bash scripts/agents/remi」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:21`：「只提醒一次」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:20`：「bash scripts/agents/remi」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:22`：「关掉一条」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:21`：「bash scripts/agents/remi」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:23`：「列出这个会话登记着的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:22`：「bash scripts/agents/remi」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:25`：「--every 是正整数分钟」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:24`：「点名到点要跑的命令或要看的东西；--name 缺」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:26`：「同名的已经有一个在等，再 s」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:25`：「不另起；要换内容或周期，先 stop 再 sta」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:27`：「子 agent 起不了后台任」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:26`：「- 子 agent 起不了后台任务：要定时的活写」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:31`：「完成通知带来一行 REMIN」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:30`：「1. 完成通知带来一行 REMIND …」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:32`：「把提醒内容里点名的检查跑完，」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:31`：「2. 把提醒内容里点名的检查跑完，有问题就处理，」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:33`：「循环提醒：结束这一轮时停止钩」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:32`：「3. 循环提醒：结束这一轮时停止钩子 .clau」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:34`：「--once 的提醒到点即删」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:33`：「4. --once 的提醒到点即删登记，不重起。」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:38`：「这几样在执行前被拒，主 ag」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/remind/SKILL.md:6`：「拒，主 agent 与子 agent 一样」 | 落点 |  |
| `.claude/skills/remind/SKILL.md:40`：「工具 CronCreate、」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:13`：「- 工具 CronCreate、Schedule」 | 落点 | 技能第 6 行留指针 |
| `.claude/skills/remind/SKILL.md:41`：「Bash 与 Monitor」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:14`：「; echo …、while true; do」 | 落点 | 技能第 6 行留指针 |
| `.claude/skills/remind/SKILL.md:42`：「crontab（-l 除外）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:15`：「- crontab（-l 除外）、at、batc」 | 落点 | 技能第 6 行留指针 |

### `.claude/skills/trilingual/SKILL.md`

| 原件 | 落点 | 类 | 备注 |
|---|---|---|---|
| `.claude/skills/trilingual/SKILL.md:6`：「三语文件一组三份：<名>.m」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:6`：「登记在 .claude/trilingual-d」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:10`：「改、新建、删掉一份登记过的中」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:10`：「- 改、新建、删掉一份登记过的中文文件（kb 的」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:11`：「写回员、回扫员、实现员交回的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:11`：「- 写回员、回扫员、实现员交回的改动碰了 .cl」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:12`：「门禁 doc-text 的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:12`：「- 门禁 doc-text 的 trilingu」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:16`：「先改中文那份。英语、日语两份」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:16`：「没动、只改译文（补译、修译名）的，单改那一份」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:17`：「结构三份逐项相同：编号集合（」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:17`：「2. 结构三份逐项相同：编号集合（D、E、C、I」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:18`：「顶上第三行是语言切换行，三份」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:18`：「：中文 中文 · [English](<名>.e」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:19`：「用词照 .claude/kb」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:19`：「4. 用词照 .claude/kb/transl」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:20`：「链接目标与中文那份指同一个文」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:20`：「5. 链接目标与中文那份指同一个文件：指别的三语」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:21`：「几个会话共写一份文件时，三份」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:21`：「6. 几个会话共写一份文件时，三份都用 scri」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:25`：「写回员、回扫员这类 agen」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:25`：「交回的 kb 改动只有中文。主 agent 暂存」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:27`：「git diff --nam」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:27`：「1. git diff --name-only」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:28`：「条目少的主 agent 自己」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:28`：「；clerk 不改 kb，交 scripts/k」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:29`：「译文改完再暂存，三份一起进暂」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:29`：「3. 译文改完再暂存，三份一起进暂存区。」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:33`：「扫仓的门禁只在提交时跑；暂存」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:33`：「单看三语六格，照 .claude/skills/」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:41`：「trilingual-fil」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:41`：「缺哪份补哪份；登记表的行照它的出路改」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:42`：「trilingual-str」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:42`：「按它列的那一项（编号、链接、标题、表格行、围栏）」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:43`：「trilingual-cha」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:43`：「它点名的那一组（改了中文那份、没碰全两份译文的）」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:44`：「trilingual-glo」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:44`：「它点名的那一组，把译文里的词换成术语表里的译名；」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:45`：「trilingual-unt」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:45`：「或反引号；整组不该判的（术语表这类）照它的出路登」 | 落点 |  |
| `.claude/skills/trilingual/SKILL.md:46`：「trilingual-omi」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/trilingual/SKILL.md:46`：「的照中文那一节逐段对齐；编号行配不上的补上缺的那」 | 落点 |  |

## 二、节标题的变动

原件的节标题里，有 21 个在合稿树任何一份文件里都不再是标题（量的：草稿目录外一段只读的 python 拿 35 份原件的标题逐个在合稿树的标题集合里找，末行 `missing_headings 21`）。按节名引它们的地方要跟着改，归 `research/prompts/rules-compress-r1-merged/references.md` 那一批；这里只列名，不判引用处：

| 原件 | 没了的节标题 | 内容去了哪 |
|---|---|---|
| `.claude/rules/artifact-line-pointer.md` 第 11 行 | 判法 | 合稿 evidence-discipline.md 第 145 行，成了一条 |
| `.claude/rules/command-safety-by-review.md` 第 4 行 | 装置起的样本进程，放进它自己的进程组 | 改名「装置起的样本进程，逐个按进程号停」（用户定第 6 条） |
| `.claude/rules/communication-discipline-by-review.md` 第 14 行 | 射程与例外 | 一条并进合稿 writing-discipline-by-review.md 第 65 行，一条成了指针（见第三节） |
| `.claude/rules/design-doc-discipline-by-review.md` 第 2、4、8 行 | 设计文档铁律（靠 review 的那一半）；2. 长度要配得上改动的重量；3. 该说服人的地方就好好说服 | 合稿 machine-first.md「设计文档：长度相称，该说服的就说服」，条号没了 |
| `.claude/rules/engineering-philosophy.md` 第 2、54 行 | 工程理念；这两面各写在哪 | 合稿 machine-first.md 的标题与第 13 行 |
| `.claude/rules/exhaustive-interleaving.md` 第 5、18、22 行 | 判定怎么分工；它撑起什么；什么时候收回 | 合稿 fs-design.md「并发交错由工具穷举判定」一节里的三条 |
| `.claude/rules/fs-design.md` 第 178 行 | 一个已经成立的实例，和一个不成立的 | 表并进「格式分支的判据」一节 |
| `.claude/rules/machine-first.md` 第 2、6、14、25、50、76 行 | 机器优先：先分清「可读」与「可验证」；态度：先怀疑，别照单全收；前提：上下文窗口已到 2×10⁵ ~ 10⁶ token 量级；前提失效时，收回哪几条；审完的结果放在哪；一条反向的提醒：文档纪律不在放弃之列 | 各并进相邻的节 |
| `.claude/rules/test-discipline.md` 第 102 行 | 自证的每一格单跑与整份跑判定相同，失败句不冒充别的格 | 挪到合稿 test-script-discipline.md 第 8、11、41 行 |
| `.claude/rules/verify-before-claiming.md` 第 50 行 | 落实 | 三条并进合稿 evidence-discipline.md 第 35 至 37 行 |
| `.claude/skills/remind/SKILL.md` 第 36 行 | 只许这一个 | 改成指 timer-only-remind.md「不用」 |

## 三、找不到落点的条目

下面每一条都是「半句」或「指针指到的节里没有那句话」，没有见到整条判据、阈值、命令或退出码丢掉的。每条写找过哪几份。

| 号 | 原件那一句 | 合稿里最近的那一句 | 找过哪几份 | 丢的是什么 |
|---|---|---|---|---|
| 丢-1 | `.claude/rules/session-wrapup.md:55`：「等改动停下再跑，或者用 --staged。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/session-wrapup.md:37`：「不为分清是谁的红另起门禁」 | 合稿树全部 `.md`（`grep -rn '等改动停下'` 0 命中） | 不带 `--staged` 跑整轮时先等改动停下这一句做法；矛盾总表常驻-8 登的是前一句「分不清的时候跑」，这半句没单独登 |
| 丢-2 | `.claude/rules/verify-before-claiming.md:48`：「结局以调度它的那一方给出的最终判定为准，引的时候引那一行。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:55`：「结局以调度它的那一方给出的最终判定为准。」 | 合稿 evidence-discipline.md 整份 | 被取消或被打断的那一次要引最终判定那一行；合稿只在同一节前一条（第 53 行）对出了结局的检查写了引那一行 |
| 丢-3 | `.claude/rules/format-evolution.md:62`：「下标（[147]，即值 − 1）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/format-evolution.md:48`：「下标（[147]）」 | 合稿 format-evolution.md、agents/sweep.md | 下标这一类派生形态怎么算（值减一）的括注 |
| 丢-4 | `.claude/rules/evidence-discipline.md:91`：「先问它是哪个参数的函数，再写进正文」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/evidence-discipline.md:98`：「说得出这条臂的代价与收益依赖装置里哪几个旋钮、每个都扫过」 | 合稿 evidence-discipline.md 整份（`grep -n '哪个参数的函数'` 0 命中） | 写进正文之前先问的次序；问的内容在那一格的问句里还在 |
| 丢-5 | `.claude/rules/communication-discipline-by-review.md:16`：「写给第三方读的产出（邮件、公告、对外文档）按它的读者写，不在射程里。」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:56`：「射程的例外照它的」 | 上游 `communication-discipline.md` 的全部标题（`grep -n '^#'`：没有叫「射程与例外」的节） | 指针写的节名上游没有；那句话在上游，见下一行 |
| 丢-6 | `.claude/rules/exhaustive-interleaving.md:16`：「用哪个工具、怎么证明它的判定分得出差别、怎么把写下的模型绑到代码上，写在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:112`：「工具是门禁 lkmm（每份 litmus 一条判定」 | 合稿 verification.md 整份（`grep -n 'lkmm\|litmus'` 0 命中）、verification-reference.md | 指针指的 verification.md「一个门禁：两种跑法」里没有 lkmm 与 litmus；话在参考件，见下面 |
| 丢-7 | `.claude/skills/crash-test/SKILL.md:20`：「按条复用两趟」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:18`：「各格判什么在 .claude/rules/verification.md」 | 合稿 verification.md「一个门禁：两种跑法」一节（第 26 至 46 行，没有各格判什么）、verification-reference.md | 指针指的节里没有各格判什么；话在参考件，见下面 |

丢-5 至丢-7 的话各在这里（指针差一跳或节名不对，话没丢）：

- 丢-5：`.claude/singlefs-ai-sop/rules/communication-discipline.md:5`：「写给第三方读的产出（邮件、公告、对外文档）按它的读者写，不在射程里。」在上游开头一段，不在哪个带名字的节里。
- 丢-6：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:42`：「lkmm 每份 litmus」。
- 丢-7：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:19`：「crash-case-replay 格起 cargo test --release -p singlefs-verification --lib --tests」；合稿 verification.md 那一节末尾另有一句叫人先读参考件：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:44`：「核它的判定单位、缩域与复用自证之前，先读 .claude/rules/verification-reference.md」，所以照指针走两跳到得了。

「删」一类的 24 行（表里标着）我逐行看过，删的是开头声明（用户定第 11 条）、引导句、说明句与文件内指针，没有判据；提交技能里服务与驱动内部怎么做的那几句改成了指脚本文件头，被指到的节名都在（`grep` 核的：`.claude/gate.d/crash-point-replay.sh` 的「只有一条路」「分片」「显卡与清场」，`scripts/crash/crash-amplification.sh` 的「按需领卡」，`scripts/mutation/broad-mutation.py` 的「分片」「判定库」「GPU 那一趟」「按需领卡」「退出码」，`.claude/gate.d/broad-mutation.sh` 的「判什么」，`task-server/task_server_lib.py` 的「分片」「GPU」），节里的字我只对了被合稿删掉的那几句的关键词，没有整节读。

## 四、五格的结论与证据

每格的「成立」指这一格的打中条件成立（找到至少一处），「不成立」指我过完全部 35 份没找到一处。「不成立」是一份样本的没打中，采不采由主 agent 照多样本的规矩定。

### 格一 丢失：成立（7 处，都是半句或指针差一跳）

证据就是第三节的丢-1 至丢-7。分两种：

- 话真的没有落点的 4 处（丢-1 至丢-4）。其中丢-1 是一句做法，另三处是半句限定或括注。
- 落点只剩指针、指到的节里没有那句话的 3 处（丢-5 至丢-7）。话在上游开头一段或参考件里，指针的节名或文件差一跳。

没打中的范围：三十五份原件里的阈值（7200 秒、5 轮、100 字、48 宽、8 个字这类）、命令行、退出码表、次序（提交技能的步号、`path-moves` 七步、`long-run-split` 六步）我逐条对过，都在落点表里有行，没有见到丢的。

什么现象会推翻：在合稿树或被指到的上游节、脚本文件头里找到丢-1 至丢-4 的那半句；或者在原件里指出一条带判据的行，落点表把它标成了「删」或指错了行。

### 格二 二义：成立（1 处新生的，3 处指针指不准，1 处原件就有）

1. 新生的二义，两种读法做法不同。合稿把「变异清单」写成了一个具体路径：`research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「做成存进仓里的变异行（crates/mutations.tsv）交给门禁反复复跑」；原件是泛称：`.claude/rules/show-me-test-by-review.md:12`：「做成存进仓里的变异清单，」。
   - 读法甲：每一条新测试的证红行都进 `crates/mutations.tsv`。
   - 读法乙：只有 `crates/` 的测试进这张表，别处的测试进各自的变异表或弄坏开关。
   - 两种读法在实验装置上分叉：执行员定义把 `research/` 那一支的变异表放在另一处，`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:41`：「那一支写变异表」，而同一份定义又叫它照这一节证红，`research/prompts/rules-compress-r1-merged/dot-claude/agents/experiment-runner.md:43`：「用例要靠改坏该函数一处证红的，照 .claude/rules/show-me-test-by-review.md」。仓里 `scripts/mutation/tables/` 下此刻有 208 张 `.tsv`（量的，`ls scripts/mutation/tables/*.tsv | wc -l`）。
   - 用户定第 4 条的原话没点路径：`records/2026-11-16-三方豁免取消与存量补轮.md:121`：「变异行之外 commit message 也写改坏了哪一行、哪条断言红」。
2. 指针指不准的 3 处，就是丢-5、丢-6、丢-7：节名在目标文件里不存在，或那一节里没有被引的话。读的人照指针去找，找不到时有两种做法（当成没有这条例外或这条判法；或去别处翻），所以也记在这一格。
3. 另有两处节名写了简称，目标唯一，不算打中，列出备查：合稿 `code-discipline-by-review.md` 第 48、69 行写「上游「类型」」，上游的节名是「类型：让非法状态写不出来」。
4. 原件就有、合稿照留的一处：`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:23`：「一节定的前缀与四段范围声明」，而它指的那一节只说不带哪个前缀：`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:9`：「交给服务的命令不带 SINGLEFS_HEAVY_TESTS 前缀」。原件同形：`.claude/skills/crash-test/commit-stages.md:17`：「一节定的那个前缀与四段范围声明」。「带前缀」读成带范围声明还是带一个没写出的环境变量前缀，两种读法做法不同；这不是压缩生出来的，照问题原文「有没有读得出两种意思的句子」列在这里。`staged-gate.md` 原件第 16 行的「（前缀照带）」合稿删了，同一个存量问题在那一份里因此没了。

什么现象会推翻第 1 条：合稿树里另有一句把 `show-me-test-by-review.md` 这一条的射程限定在 `crates/` 的测试；或者执行员、工具员的定义里写明它们证红不照这一条的路径。我找过合稿 `show-me-test-by-review.md` 整份、`test-script-discipline.md` 第 40 行（那里还是泛称「变异进入库的变异清单」）、`implementation-workflow.md` 第 9 行（那里说的是 `crates/` 下的改动），没有找到这样的限定句。

### 格三 常驻够不够：不成立

这一组里从常驻文件挪出去的细则，常驻一侧都留着入口，我没有找到「不读就会做错而没叫人先读」的一步：

- 定时：`research/prompts/rules-compress-r1-merged/dot-claude/rules/timer-only-remind.md:8`：「命令写法、到点之后做什么、关掉与列出：.claude/skills/remind/SKILL.md」（与原件同）。
- 提交时的门禁：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「提交时由主 agent 经任务服务交」，同一格点了两份提交技能的路径。
- 搬迁登记的细则：`research/prompts/rules-compress-r1-merged/dot-claude/rules/path-moves.md:13`：「登记行什么时候生效、门禁照它换算哪几样」，指到脚本文件头「搬迁登记」；那一节我读过（`.claude/gate.d/lib-lag-baseline.py` 第 53 至 102 行），生效条件、不许串、搬迁实据、新行必须生效、关不住的几样都在。它少两类「按路径管的类」（`.claude/gate.d/` 顶层、上游规则目录），合稿规则第 12 行把八类写全了，矛盾总表按需-M3 登着。
- 被并掉的八份里原来常驻的五份（`verify-before-claiming`、`artifact-line-pointer`、`exhaustive-interleaving`、`service-stop-readback`、`communication-discipline-by-review`）并进的四份（`evidence-discipline`、`fs-design`、`command-safety-by-review`、`writing-discipline-by-review`）在合稿 `CLAUDE-merged.md` 里都还是 `@` 常驻（第 47、55、58、59 行）；原来开会话读一次的三份并进的 `machine-first.md` 照旧是开会话读一次（第 65 行的 read-once 登记）。
- 改成指上游的几条（历史条目编号、`history-carriers` 的判法、踩坑先想门禁、覆盖与本次未跑怎么记）指到的上游文件 `kb-discipline.md`、`sop-first.md`、`show-me-test.md` 都是 `@` 常驻。

什么现象会推翻：指出一步做法，它的细则只在技能、参考件或脚本文件头里，而常驻文件里没有一句指过去，照常驻文件做会做错。丢-6、丢-7 是指针差一跳，不是没有入口，我没把它们算进这一格。

### 格四 子 agent 读得到：不成立（只核了指到这一组文件的指针）

- 合稿树里不再有指到八份旧文件名的地方（量的）：`grep -rnE 'verify-before-claiming|artifact-line-pointer|exhaustive-interleaving|service-stop-readback|communication-discipline-by-review|engineering-philosophy|design-doc-discipline-by-review|pushback-discipline-by-review' dot-claude CLAUDE-merged.md` 在合稿树根跑，0 行输出。
- 共用约束「每个定义都照守」的那一处改指到了并入后的节，节在：`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:20`：「说外部状态之前现查（.claude/rules/evidence-discipline.md」，合稿 evidence-discipline.md 第 22 行是这个节标题。
- 十四份定义与共用约束里指到这一组文件的指针（`grep -oE` 数出 67 处，量的）我逐处看过：带节名的，节名在合稿目标文件里都是标题或加粗的条名（`pointers.py` 对全树 364 处「路径加节名」的指针核标题，指到这一组文件而不是标题的只有 3 处，逐处看过都是条名或标题的前半截：共用约束第 24 行的「旧证据是参考，不是依据」是合稿 evidence-discipline.md 第 61 行那一条；`implementation-workflow.md` 第 40 行的「门禁的范围必须可判定」是合稿 fs-design.md 第 73 行那一条；`CLAUDE-merged.md` 第 83 行的「改一个全仓术语」是 path-moves.md 节标题的前半截）。
- 不在「开工先读：」一行里、写在定义步骤里的指针有 10 行指到这一组（分诊员第 32 行，写回员第 21、24、30、32 行，回扫员第 34 行，执行员第 40、43、60 行，材料员第 27 行；量的，`grep -nE` 去掉带「开工先读」的行），所指文件除执行员第 43 行的 `show-me-test-by-review.md`、回扫员第 34 行的 `path-moves.md`、材料员第 27 行的 `fs-design.md` 外都在各自「开工先读：」里。这三处靠共用约束那一句兜：`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`：「点到就整份读」。这一句读成「定义与共用约束里任何一处点到就读」时三处都读得到；读成「只管读的时候整份读」时三处没人叫读。这一句归定义那一题判，我只在这里记下它撑着这三处。

什么现象会推翻：合稿的某份定义或共用约束里有一处指到这一组文件的节，而那个节名在合稿目标文件里既不是标题也不是条名。

### 格五 悄悄改了做法：成立（2 处没登的；另列 3 处供判，不计入）

没登的 2 处：

1. 新加了一句谁为准。`research/prompts/rules-compress-r1-merged/dot-claude/rules/code-discipline-by-review.md:4`：「两处说同一件事以上游为准」。原件 `.claude/rules/code-discipline-by-review.md` 整份没有这句，仓里别的项目规则也没有（`grep -n '同射程\|以上游为准' .claude/rules/*.md` 0 行输出）。矛盾总表、合稿结果与调度记录里都没登（量的：`grep -c '以上游为准'` 对 `contradictions.md`、`merge-result.md`、`records/2026-11-16-三方豁免取消与存量补轮.md` 各打 0）。它此刻改不改某一条的结果我没找到实例（合稿这一份指到上游 `code-discipline.md` 的那些条我逐条对过，没见到两处说法不同的；没被指到的条没对），但它是一条新判据：以后两处写岔了，照它是上游赢。
2. 证红的变异行钉到了一张表。就是格二第 1 条：原件「变异清单」是泛称，合稿写成 `crates/mutations.tsv`。用户定第 4 条登的是「两样都要」，没登路径；矛盾总表里没有这一处（`grep -c 'mutations.tsv' contradictions.md` 打 0）。照合稿的字面，不在 `crates/` 的测试也要往这张表加行。

另列 3 处，登了或登在别处，不计入这一格，交矛盾那一题或主 agent 判：

- 登了、但「改没改做法」一栏标成只改措辞：矛盾总表常驻-11，`research/prompts/rules-compress-r1-merged/contradictions.md:46`：「检出工具只报，终止归报告的人或 agent」。原件许检出的工具自己终止：`.claude/rules/command-safety-by-review.md:22`：「报告的一方（人、agent 或检出的工具）可以终止它」；合稿不许：`research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:17`：「报告的人或 agent 可以终止它」。这是改了谁能终止，不只是措辞。
- 同一句在规则里登了、在提交技能里没登：原件 `.claude/skills/crash-test/commit-stages.md:17`：「计划按多机配置点名的卡数写 N 片」与 `.claude/skills/crash-test/commit-stages.md:20`：「计划把这一趟要现判的条按上一次的用时分成两堆」，合稿技能里这两句都删成了指脚本文件头的指针。矛盾总表验证-2、验证-3 登的是规则里的同一句（`research/prompts/rules-compress-r1-merged/contradictions.md:25`：「规则不写片数，照门禁文件头」），技能这两处没列。脚本文件头今天的说法与原件那句不同：`.claude/gate.d/crash-point-replay.sh:140`：「上界取它与卡数的大者」。主 agent 照技能做的步骤没变（分片由服务做），所以我没把它算成改了做法。
- 射程多了一个词：合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:6`：「装置、判法、自证与复跑脚本起样本进程」，原件 `.claude/rules/command-safety-by-review.md:6`：「装置、判法与复跑脚本起样本进程」。这一节整节是用户定第 6 条改写的，定案原话是「脚本与装置里也不对进程组发信号」，自证脚本在它的射程里，我按已登算。

用户定案里落在这一组的几条，落点我核过，对得上：

| 定案 | 合稿落点 | 核的是什么 |
|---|---|---|
| 第 3 条 并行度缺省 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:39`：「没给时一律取登记的每台默认重活线程数」 | 同一行还写着不按 `nproc` 取 |
| 第 4 条 两样都要 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「变异行之外，commit message 里也写明怎么确认它会红」 | 两样都在；路径那一处见上 |
| 第 6 条 进程组信号 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:8`：「信号只按进程号逐个发，不对进程组发」 | 节名、三条做法都改了；合稿另留「起在它自己的会话里、按会话号认」一句，调度记录里主 agent 已定留着 |
| 第 8 条 示例 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/changelog-format.md:15`：「新立，待定：三个候选全落在第三格」 | 格式示例留着；合稿 `fs-design.md` 里围栏行数是 0（量的，`grep -c` 三个反引号打 0） |
| 第 11 条 开头声明 | 无 | 合稿这一组十八份规则里带「这是 singlefs 的项目本地规则」的文件数是 0（量的，`grep -l … \| wc -l`） |

矛盾总表里指到这一组的条目（常驻-1 至常驻-14、常驻-16 至常驻-19，按需-M1 至 M17，主 agent 处置-1、4、5、6）我在逐条读的时候都对到了合稿的那一行，落点表里标着「已登的改动」的 33 行就是它们；改成的那一句对不对归矛盾那一题，这里没判。

什么现象会推翻：矛盾总表、用户定案或调度记录里找到一行登着「以上游为准」或 `crates/mutations.tsv` 这一处的改动；或者原件里本来就有同义的一句。

## 五、逐条过到的文件与没过到的

逐条过到的 35 份（原件与它的接收文件都整份读过）：

- 八份被并掉的：`verify-before-claiming.md`、`artifact-line-pointer.md`、`exhaustive-interleaving.md`、`service-stop-readback.md`、`communication-discipline-by-review.md`、`engineering-philosophy.md`、`design-doc-discipline-by-review.md`、`pushback-discipline-by-review.md`。
- 七份规则：`evidence-discipline.md`、`session-wrapup.md`、`command-safety-by-review.md`、`format-evolution.md`、`fs-design.md`、`show-me-test-by-review.md`、`writing-discipline-by-review.md`。
- 两份提交技能：`crash-test/commit-stages.md`、`gate/staged-gate.md`。
- 其余规则十一份：`changelog-format.md`、`code-discipline-by-review.md`、`directory-layout.md`、`implementation-first.md`、`long-run-split.md`、`machine-first.md`、`mutation-sampling.md`、`path-moves.md`、`test-discipline.md`、`test-script-discipline.md`、`timer-only-remind.md`。
- 其余技能七份：`crash-test/SKILL.md`、`decide/SKILL.md`、`gate/SKILL.md`、`halt/SKILL.md`、`queue-health/SKILL.md`、`remind/SKILL.md`、`trilingual/SKILL.md`。

没逐条过到的：没有。过得浅的两处照实写：`trilingual/SKILL.md` 与 `queue-health/SKILL.md` 是拿去掉加粗之后的 `diff` 与并排通读核的，两份都只有措辞改动；提交技能里改成指脚本文件头的那几句，被指到的节我只核了节名在不在与被删那几句的关键词，没有整节读（`lib-lag-baseline.py` 的「搬迁登记」一节除外，那一节整节读了）。

## 要主 agent 跑的命令

没有。这一份没写模型，也没有要起项目代码的核对；用到的都是只读命令、草稿目录里的文本脚本与 `scripts/agents/cite-check.py`。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一 丢失 | 成立 | 7 处，都小：4 处半句没有落点（丢-1 是一句做法），3 处指针指到的节里没有那句话、话在上游开头或参考件 |
| 二 二义 | 成立 | 1 处新生：证红的变异行写成 `crates/mutations.tsv`，不在 `crates/` 的测试两种读法；3 处指针指不准；1 处原件就有的「前缀」 |
| 三 常驻够不够 | 不成立 | 挪出去的细则常驻一侧都有入口；被并掉的常驻文件并进的还是常驻文件 |
| 四 子 agent 读得到 | 不成立 | 指到这一组的 67 处指针节名都在；3 处步骤里的指针靠共用约束「点到就整份读」一句兜，那一句归定义那一题 |
| 五 悄悄改了做法 | 成立 | 2 处没登：`code-discipline-by-review.md` 新加「以上游为准」；变异行钉到一张表。另 3 处登了或登在别处，交矛盾那一题 |

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务，没有要复跑的数。
- 没读禁读清单里的文件；写手的对照表（`~/.local/share/singlefs/work/rules-compress-*/result.md`）一份都没读，`merge-result.md` 只 `grep` 过节标题与几行命中，没当证据引。
- 矛盾总表里每条「改成的那一句」对不对没判，只核了它们在这一组里的落点在不在。
- 全仓按旧文件名、旧节名引这一组的地方（钩子、门禁、脚本、kb）没核，那是 `references.md` 的事；节标题的变动只列了名。
- 上游规则 `rules-discipline.md`、`preflight-discipline.md`、`command-safety.md` 没整份读，只核了被指到的节名在不在。
- 自动落点 906 行没有逐行再核第二遍；落点表里某一行指错了行而我没看出来，这一种错排除不了。
- 没有草稿目录之外的副本、编译目录要删；草稿目录里只有文本脚本与它们的输出。
