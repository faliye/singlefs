# rules-compress-entry-r3 攻方腿报告

日期：2026-11-16。轮名 rules-compress-entry-r3，第三轮（最后一轮）；前几轮判决 `research/prompts/rules-compress-entry-r1-main-verification.md`、`research/prompts/rules-compress-entry-r2-main-verification.md`。

没写模型，没有模型目录与 `rerun.sh`；结论都来自读文字与只读命令（`grep -nF`、`sed -n`、`sha256sum -c`），没交任务、没建仓副本与编译目录。草稿目录 `~/.local/share/singlefs/work/rules-compress-entry-r3-opus` 已认领。

开工核过（量的，现跑）：`sha256sum -c research/prompts/rules-compress-entry-r3-snapshot/judged-sha256.txt` 两份原件都 OK；在合稿树目录里对 `merged-tree-sha256.txt` 跑 `sha256sum -c`，`grep -v ': OK$'` 无输出。

被判的四份：第三版合稿树 `research/prompts/rules-compress-r3-merged/` 里的 `CLAUDE-merged.md`、`dot-claude/main-agent.md`、`dot-claude/skills/merge-verify/SKILL.md`、`dot-claude/skills/dispatch/SKILL.md`。第三版相对第二版在这四份上的改动取自 `diff-from-r2.patch` 第 1-24、508-579、1133-1165、1192-1218 行（`grep -n '^diff '` 定位，量的），共 19 处改动（`CLAUDE-merged.md` 3、入口 9、派发技能 4、合入后验证技能 3）。前两轮的攻方报告、两轮判决、`dispositions-r2.md`、`revise-result.md` 读过；前两轮已打中并已修的不再报。

## 各格判定一览

| 格 | 判定 | 打中几条 | 哪几条（轻重） |
|---|---|---|---|
| 一 第二轮修补对不对 | 打中 | 1 | 一-1 用户定案第 24 条在入口第 3 条写成「钩子弹过的不另问」，按事后才知道的事判、与另两处按标记判不一致，字面读得出先问一次再被钩子弹一次（换上之后可以再改） |
| 二 修补引出的新问题 | 打中 | 3 | 二-1 写段第 6 条只补了调查员一支，且写成「命令跑完」不是「跑绿」；主 agent 自己的探索性改动同形没补（换上之后可以再改）；二-2 参考件读的时机改成三项封闭清单，常驻的 `fs-design.md` 与提交技能两份也点它而不在清单里（换上之后可以再改）；二-3 派发技能重读的时机在入口与技能 frontmatter 两处、修补只改了入口一处（换上之后可以再改） |
| 三 能不能换上 | 没找到第三处没登的做法不同 | 0 | 范围见「没打中的形状」 |

另记两条观察（不算打中，理由在各自一节）。

一句结论：这一组可以换上；四条打中都是措辞级的二义或残留，照第三版做最坏是多弹一次窗、别的会话的门禁在自己的前置核对上停一次、少读一次参考件或技能，没有一条会做出与原件不同的判定或合入。四条都建议换上之后并进下一批治理文件改。

## 格一 第二轮修补对不对

逐处对过的修补（19 处，`diff-from-r2.patch` 里落在这一组的全部）：放回的话逐字对原件、指针逐个现查目标在、引到脚本或钩子文件头的逐段读了今天的原文。只有下面一处不对。

### 一-1 用户定案第 24 条在入口第 3 条落成「钩子弹过的不另问」，判的时刻不对，读得出要问两次

- 用户定案：`records/2026-11-16-三方豁免取消与存量补轮.md:212`：「由钩子弹的那一次就算确认，主 agent 不另问第二次」（同一行接着写不带两个标记、钩子不弹的，主 agent 自己弹窗问一次）。原件 `.claude/main-agent.md:26`：「提交之外任务确实要跑，先弹窗问用户，同意了才跑」。
- 第三版三处：
  - `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`：「弹的那一次就是确认，主 agent 不另问」，主语是「命令带 `--full-gate` 或 `--whole-package-confirmed`」的——按命令上带不带标记分。
  - `research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:22`：「的由钩子弹，主 agent 不另问」，前面是「带 `--whole-package-confirmed`」——同样按标记分。
  - `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:26`：「确认一次（钩子弹过的不另问）」——按钩子弹没弹过分。
- 钩子今天的原文：`.claude/hooks/run-scope-guard.sh:9` 挂在 `PreToolUse:Bash`，即主 agent 执行那一条 submit 的那一刻才判；`.claude/hooks/run-scope-guard.sh:34`：「钩子不放行也不拒」，接着打 ask 的 JSON 要用户在界面上确认；`.claude/hooks/run-scope-guard.sh:41`：「判法文件读不到或导入就出错时这条命令照常执行、不弹窗」。
- 为什么算打中（格一「定案落偏」）：主 agent 决定「要不要自己先弹窗」是在交命令之前，那一刻钩子还没弹过；「钩子弹过的」在这个时刻永远不成立。字面读法甲：入口第 3 条照原件那样先自己问一次（钩子还没弹），交上去钩子再弹一次——两次，正是第 24 条说的「不另问第二次」要去掉的那一次。读法乙：把「弹过的」读成「会弹的」，与 `verification.md:38` 一致。入口是常驻、`verification.md:38` 要顺着指针去读，读者先碰到的是入口那一句。
- 同时满足格二「同一条判据又住了两处」：修稿一方写「句子只住 verification.md 一处」（`research/prompts/rules-compress-r3-merged/dispositions-r2.md:36`），实际入口与合入后验证技能各写了一遍判法，一处按标记、一处按事后的事件。
- 照第三版做会做成什么：用户要求跑一次 `crash-point-replay` 不指定范围（命令必带 `--full-gate`，否则 `.claude/hooks/run-scope-guard.sh:45` 一段执行前拒），主 agent 照入口先弹窗问一次，再交命令、钩子再弹一次。多一次弹窗，判定与跑法不变。
- 四句：不涉及臂；主 agent 在要不要先问的那一刻看得到命令带没带标记，看不到钩子弹没弹（还没交）；满足格一「定案落偏」的字面（落成了按事件判、事件在判的时刻不可见），兼格二「住了两处」；没有跑前改法。
- 轻重：换上之后可以再改（后果是多一次弹窗）。
- 我提的改法（只在我的阅读上核过、被攻过零轮）：入口第 3 条括号改成「（命令带 `--full-gate` 或 `--whole-package-confirmed` 的由钩子弹，不另问）」，与合入后验证技能第 22 行同一种写法；或括号整个删掉，只留指针。

格一其余各处（没打中）：

| 改动 | 第三版 文件:行 | 核了什么 | 结果 |
|---|---|---|---|
| 上游仓那一格放回 | `CLAUDE-merged.md:40` | 对原件 `CLAUDE.md:41` 逐字 | 同 |
| 版本戳那一格放回「规矩变过了，先读再跑」 | `CLAUDE-merged.md:42` | 对原件 `CLAUDE.md:43` | 同（句号换分号，不带做法） |
| 参考件读的时机加派发表 | `CLAUDE-merged.md:66` | 派发表第 96 行确实点参考件两节，两节标题在参考件第 14、140 行 | 指得到；清单封闭见二-2 |
| 技能重读的时机加两件 | `main-agent.md:5`、`:74`、`:78` | 两节入口都写了做这一步之前；压缩那一句改成「这几件事」 | 罩到；技能 frontmatter 没跟见二-3 |
| 「只有两类」去掉（定案 25） | `main-agent.md:9` | 对原件 `.claude/main-agent.md:7` 那半句逐字 | 同；第二版多出的半句删了 |
| 整包确认的弹窗半句放回 | `main-agent.md:20` | 对原件 `.claude/main-agent.md:19` 括号里那半句 | 逐字同，与定案 24 不相抵 |
| 写段第 6 条补调查员一支 | `main-agent.md:51` | 定案第 19 条只定「照旧在主工作区改」 | 补了；措辞见二-1 |
| 拒算第 1 条删重复半句、并入三类 | `main-agent.md:68` | 对 `scripts/gate/refused-test-registry.py` 文件头「覆盖：」那一行 | 一致；节开头第 66 行一处留着 |
| 派出去之后、交回怎么读两节入口 | `main-agent.md:74`、`:78` | 同上第 5 行 | 罩到 |
| 派发表「要推论」行分两处指 | `main-agent.md:90` | 定案 23；`implementation-workflow.md` 第 25 行有「代码轮与治理文件轮怎么收」一节，第 27 行写同一套收 | 一致 |
| 技能第 4 条调查员例外拆句、改成指过去 | `dispatch/SKILL.md:14` | 目标写段第 2 条在入口第 47 行 | 指得到，括号里的自相矛盾读法没了 |
| 坏卡与服务健康一段改指入口 | `dispatch/SKILL.md:39` | 入口第 74 行有 `control check` 回读、退 0 才算健康 | 一处住 |
| needs-attention 一段改指入口 | `dispatch/SKILL.md:41` | 入口第 74 行有；`task-server/task_server_lib.py` 第 2071 行：退 230、以 `TASK_SERVER_ATTENTION` 起头 | 一处住 |
| 事实表验收改指派发表 | `dispatch/SKILL.md:75` | 入口第 95 行「交回的事实表过了 `--check-facts` 才往下」 | 一处住 |
| 整批重交里点名是哪一种退 2 | `merge-verify/SKILL.md:14` | `scripts/gate/merge-and-verify.sh` 第 208-209 行退 2 的几种、第 181 行只有 `--fingerprint-of` 那一种出路是整批重交 | 一致 |
| 退 3 一支的确认改照「提交之外」 | `merge-verify/SKILL.md:22` | 钩子文件头第 33-35 行 | 一致（见一-1 的另两处） |
| 拒算末段删 | `merge-verify/SKILL.md` 原第 58 行 | 内容并进入口第 68 行 | 没丢 |

## 格二 修补引出的新问题

### 二-1 写段第 6 条只补了调查员一支，「命令跑完」不等于「跑绿」，主 agent 自己的探索性改动同形没补

- 第三版：`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51`：「先回在改，命令跑完再回「已收尾」」（前半句是调查员「查到就修」在主工作区改了、验证命令还没跑完的）。同一条前半句仍是原件的条件：`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:51`：「主工作区里没有打了而没验过的补丁」。原件 `.claude/main-agent.md:51` 只有这一半。
- 新句子读得出两种（格二「新写的句子读不读得出两种意思」）：
  - 读法甲：「跑完」是有了结局，绿红不论，跑完就回「已收尾」。
  - 读法乙：「跑完」指跑过了（绿），红了还在改。
  调查员的验证命令（`research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33`：「这几条命令的 `--cwd` 与命令里的 `<根>` 写主工作区仓根」，证红、clippy、build、测试与带 `--item` 的几条）红了，改动照旧留在主工作区；甲读法下回「已收尾」时主工作区里是一处红着的改动。原件的条件「没有打了而没验过的」也没说红的算不算验过，但原件没写「跑完再回」这一句，甲读法是这一句带进来的。
- 同形没补的一支：主 agent 自己写的探索性 `crates/` 改动同样直接在主工作区上（`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:9`、派发表 `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:93`：「主 agent 自己写、直接和用户对话」，合入后验证技能 `research/prompts/rules-compress-r3-merged/dot-claude/skills/merge-verify/SKILL.md:26`：「只用于改动确已在主工作区上的（主 agent 自己写的探索性改动这类）」）。它不是「补丁」，第 6 条前半句的条件对它照样成立；修补把例外列成只有调查员一支，读者更容易读成「别的主工作区改动不算」。原件同样没写这一支，所以这半条是修补没顺手补全，不是修补改坏的。
- 照第三版做会做成什么：写段里主 agent 在主工作区改着探索性代码（或调查员的验证命令跑红了），别的会话来问要不要起提交前的六道门禁，主 agent 回「已收尾」。那个会话照提交技能开跑前先核工作区等于暂存区（`research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:15`：「开跑前核工作区等于暂存区」），`git diff --quiet` 不过就停：`research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:18`：「两样有一样不过就不跑那一道，停下」，再回头找改的会话协商。后果是对方白起一次、多一轮协商，不会判错（这一段是推的，没跑）。
- 四句：不涉及臂；主 agent 回话那一刻看得到自己的探索性改动与调查员命令的结局；满足格二「新写的句子读得出两种意思」；没有跑前改法。
- 轻重：换上之后可以再改（对方门禁的前置核对兜得住）。
- 我提的改法（被攻过零轮）：第 6 条改成「主工作区里有没验过、或验红了的改动（调查员『查到就修』那一支、主 agent 自己的探索性改动）的，先回在改，验绿了再回『已收尾』」。

### 二-2 参考件读的时机改成三项封闭清单，常驻的 `fs-design.md` 与提交阶段两份技能也点它，不在清单里

- 第三版：`research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:66`：「与 `.claude/main-agent.md` 派发表点名它的那一步之前读点名的那一节」（前面是 `verification.md`、`implementation-workflow.md`，三项用「、」「与」连成一张列举）。
- 第三版里另外点参考件某一节、要主 agent 照着做的：
  - `research/prompts/rules-compress-r3-merged/dot-claude/rules/fs-design.md:112`：「每份 litmus 一条判定，`.claude/rules/verification-reference.md`「各道的判定单位」」；`fs-design.md` 是 `@` 常驻的（`research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:47`）。
  - `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:24`：「要卡的三处怎么按需领卡，照 `.claude/rules/verification-reference.md`「门禁 crash-point-replay 怎么跑」」——提交时主 agent 照这一份跑六道。
  - `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/SKILL.md:18`：「各格判什么在 `.claude/rules/verification-reference.md`「门禁管哪一半」」。
- 来历：第二轮攻方把这一处记成观察、不算打中（`research/prompts/rules-compress-entry-r2-opus-output.md:99`：「只是又多了一处」），理由是「照 X 某节」本身就叫人去读、而第二版那一句只列两份。第三版照去向表（`research/prompts/rules-compress-r3-merged/dispositions-r2.md:35`：「时机一句补上主 agent 派发表」）只补了派发表一处，清单从两项变成三项、仍是列举、仍漏常驻的 `fs-design.md` 与两份技能。这不是重报第二轮的观察：第二轮问的是「派发表没列」，这里问的是修补之后这张清单成了「列了三处、别处不算」的封闭写法。
- 两种读法（格二）：甲，清单是全部时机，别处点到参考件不在这句的「到…之前读」之内（照指针自己的「照 X」去读与不读两可）；乙，清单只是举例，任何一处点名都在那一步之前读。
- 照第三版做会做成什么：提交时照 `commit-stages.md:24` 跑 `crash-point-replay`，那一句叫照参考件「门禁 crash-point-replay 怎么跑」看各片各判什么、领卡被拒怎么办；甲读法下主 agent 不在清单的时机里，不读，读不出分片与按需领卡的细则，只靠门禁文件头（推的，没跑）。
- 四句：不涉及臂；主 agent 那一刻看得到 `commit-stages.md:24` 的指针，看不到它算不算在 `CLAUDE-merged.md:66` 的时机里；满足格二「新写的句子读得出两种意思」；没有跑前改法。
- 轻重：换上之后可以再改。
- 我提的改法（被攻过零轮）：那一句改成「到任何一处点名它某一节的那一步之前读点名的那一节」，不列文件。

### 二-3 派发技能重读的时机住在入口与技能 frontmatter 两处，修补只改了入口

- 第三版入口：`research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:5`：「每次派发之前、被叫醒后处置之前、读交回之前读」派发技能。
- 第三版技能自己的描述：`research/prompts/rules-compress-r3-merged/dot-claude/skills/dispatch/SKILL.md:3`：「主 agent 每次派发子 agent 之前读它」——只一件。
- 修稿一方有意不动：`research/prompts/rules-compress-r3-merged/dispositions-r2.md:30`：「技能 frontmatter 按「与第二版相同」不动」；`research/prompts/rules-compress-r3-merged/revise-result.md:70`：「24 份带 frontmatter 的文件与第二版逐字相同」。
- 为什么算打中（格二「同一条判据是不是又住了两处」）：「什么时候读派发技能」是一条判据，第三版住入口与 frontmatter 两处、两处不同。第二轮打中一-2 修的正是这条判据，修补只改了入口一处。技能的 description 是技能被列给模型、决定什么时候调起它时看的那一句；照它读，叫醒与读交回两个时刻不在其中。
- 照第三版做会做成什么：多数情形入口常驻、照入口读，做法不变；只在主 agent 靠技能列表而不是入口决定读不读时少读一次（推的，没量技能列表在调度里起多大作用）。
- 四句：不涉及臂；两处主 agent 都看得到；满足格二「住了两处」；没有跑前改法。
- 轻重：换上之后可以再改。
- 我提的改法（被攻过零轮）：派发技能 frontmatter 那一句改成与入口第 5 行同样三件；或改成「何时读照 `.claude/main-agent.md` 开头那一段」。

## 格三 能不能换上

没找到矛盾总表与用户定案之外、照第三版做与照原件做不同的第三处（一-1、二-1 两条在格一、格二里已算）。下面是查过的句子，范围见「没打中的形状」。

做法：原件 `.claude/main-agent.md` 第 7-72 行与第 102-129 行里带「不」「只」「一律」「除外」「先」「才」的分句逐个摘出（第 76-100 行第二轮攻方逐句对过，`CLAUDE.md` 第二轮攻方逐行对过，这两块只看了第三版改过的三行），对到第三版入口、两份技能，指针指出去的再对到目标那一份。抽出来要特别核的几条与落点：

| 原件的分句 | 第三版落点 | 判 |
|---|---|---|
| `.claude/main-agent.md:116` 腿「一轮一条；第一轮派正推，辩方只在已有一轮判决可复核时派」「不派本地模型的腿」 | 入口第 90 行指 `three-way-inference.md`「各条腿必须互不重复」；那一份第 9 行：「第一轮派 `three-way-forward`，已有一轮判决可复核时才派 `three-way-defense`」 | 在，同义 |
| `.claude/main-agent.md:116`「第三轮为最终轮，论证前两轮的结论和判定过程」 | `three-way-inference.md` 第 45 行（设计推论轮）；代码轮与治理文件轮由定案 23 另立 `implementation-workflow.md` 第 25-32 行 | 登在定案 23 名下 |
| `.claude/main-agent.md:122` 六道「谁并行谁串行」、broad-mutation 排在 crates-mutation-replay 之后（第一轮攻方记「没查完」） | `commit-stages.md` 第 22 行：「crates-mutation-replay 等 `crash-point-replay` 跑完再起」，第 27 行 2b 等 crates-mutation-replay 跑完再起 | 在 |
| `.claude/main-agent.md:124` 主 agent 自己写回时「自己写规格、跑 `scripts/kb/replace-batch.py`」 | 派发技能第 82 行：「主 agent 自己写的，写规格、跑 `scripts/kb/replace-batch.py`」 | 在（触发时机的讨论见观察-3） |
| `.claude/main-agent.md:124` 规格动了分项依据段要带实验页 `### 影响的决策` 配对行 | 派发技能第 84 行指写回员定义「输入」；主 agent 自己写时常驻的 `format-evolution.md` 第 28 行：「分项依据里引的每个实验，实验页里都有这一行」 | 两头都罩到 |
| `.claude/main-agent.md:94`「`cancel`、`clean` 只对自己交的号做（服务只认会话）」 | 派发技能第 41 行；服务一侧有自证格 `cancel-own-session-only`（`task-server/task_server_lib.py` 第 471 行） | 在，且服务兜 |
| `.claude/main-agent.md:33`「只对合入时改过锚点或冲突的变异行 prove-red.sh 证红」 | 合入后验证技能第 47 行 | 在 |
| `.claude/main-agent.md:37`「只经 `python3 scripts/kb/append-record.py」 | 入口第 37 行 | 逐字 |
| `.claude/main-agent.md:24`「答不出就还没规划完」 | 派发技能第 12 行 | 在 |
| `.claude/main-agent.md:78`「派预计超过一小时的活时，告诉用户…」 | 派发技能第 24 行 | 在 |

## 观察（不算打中）

### 观察-1 定案 25 的「三处原话各留」，共用约束那一处不是原话

- 定案：`records/2026-11-16-三方豁免取消与存量补轮.md:213`：「「三处原话各留，不合并」」。修稿一方的落点 `research/prompts/rules-compress-r3-merged/revise-result.md:30`：「`dot-claude/agent-common.md:11`（「自己直接做，不派」）」。
- 原件 `.claude/agent-common.md:8`：「**简单任务不派发，自己直接改**」；第三版 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:11`：「自己直接做，不派」。不是逐字原话。
- 不算打中：「改」与「做」在原件同一句的例子里就含「查询一下、计划一下」，「做」不比原意宽；入口与 `CLAUDE-merged.md` 两处是逐字原话。共用约束归「定义」那一题，这里只记。

### 观察-2 修稿报告里一句与树不合（不在树里）

- `research/prompts/rules-compress-r3-merged/revise-result.md:49`：「`gate.sh --staged` 的 `gate-staged.sh` 不带 `--full-gate` 交」；树里 `research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:17` 写的是带 `--full-gate` 交，`research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:20`：「只有它们与 `scripts/gate/gate-staged.sh` 认这个标识」，钩子文件头第 34 行凡带标识的都弹。树对、报告那一句错；不在这一组，判这一份时只当线索。

### 观察-3 入口「派出去之后」括号把「这一步」点成两件，细则里的「自己交的任务怎么看与停」不在两件里

- `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:74`：「做这一步（起看门狗、被叫醒后处置）之前先读」，括号前的细则清单有「自己交的任务怎么看与停」。主 agent 自己起意去停一条任务时不在两件之内、也不在第 5 行的三件之内。
- 不算打中：那一段的判据（只对自己的号 `cancel`、`clean`）服务自己判（见格三表），`needs-attention` 与 `control check` 两条留在入口常驻；读不到技能也做不错。

## 没打中的形状

这些都只读了一遍，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，一次抽样不撑「没打中」的结论；照用户定案第 26 条，要拿它当依据得看正推腿是不是也覆盖到。

- 格一：`diff-from-r2.patch` 落在这一组的 19 处改动逐处读了改前、改后；放回的五处（上游仓、版本戳、整包弹窗半句、「只有两类」回到原件、拒算三类）逐字对原件；引到文件头的四处（`merge-and-verify.sh` 退出码一节、`refused-test-registry.py`「覆盖：」、`run-scope-guard.sh`「放宽检查的提交标识」、`task_server_lib.py` needs-attention 退 230）逐段读了今天的原文；定案 23、24、25 在这一组的落点逐处对了调度记录原话。只打中一-1。
- 格二：第三版新加或改写的指针逐个现查目标在且唯一：`implementation-workflow.md`「代码轮与治理文件轮怎么收」（第 25 行，一个）、`verification.md`「提交之外」（第 38 行，粗体条目一个）、入口「写与验分两段」写段第 2 条（第 47 行）、入口「派出去之后」（第 72 行起，两条判据都在第 74 行）、入口派发表「一批阶段任务结束」（第 95 行）、参考件两节（第 14、140 行，各一个）。同一条判据住两处：第二轮点名的五处（needs-attention、control check、`--check-facts`、拒算三类、调查员例外）在第三版各只剩一处判据、另一处只是指针；新冒出的是一-1（定案 24 的三处写法）与二-3（技能重读时机两处）。新写的句子：入口第 5、9、20、26、51、68、74、78、90 行与两份技能改过的五行逐句读了两种读法，只有二-1、二-2 与一-1 读得出两种。
- 格三：见格三那一节的做法与表；原件第 102-110 行「派发提示怎么写」四段对到入口第 82 行与派发技能第 56-64 行，没丢。
- 没再攻的角度（前两轮已打中并已修、这一轮核过修对了的）：读一次与压缩上下文、拒算谁登记、整批重交的三种场合、「只有两类」、`--check-facts`、拒算尾段、needs-attention 常驻、改技能出了工具员写范围（登在引用改法清单，归脚本那一批）、版本戳、`-en` 先行词、调查员例外的括号。

## 这条腿自己的限度

- 全部结论来自读文字，没有一条经过执行。标「推的」的三处（二-1 对方门禁前置核对会停、二-2 读不读参考件、二-3 技能列表的作用）没量。
- 格三对原件 `.claude/main-agent.md` 第 76-100 行与 `CLAUDE.md` 没有重做逐句对照，依的是第二轮攻方的结果；这两块只看了第三版改过的行。
- 指针出去的目标文件（`verification.md`、`implementation-workflow.md`、`three-way-inference.md`、`commit-stages.md`、`format-evolution.md`、写回员与调查员定义）只读了被指到的那几行，那几份自己的压缩对不对归别的题。
- `CLAUDE-merged.md` 的 `@` 常驻清单与读一次清单的增删归常驻那一题，这里没判。
- 一次样本；「没打中」的几格照规则不撑结论。

## 要主 agent 跑的命令

无。这条腿没写模型、没有要经服务跑的东西；引文核对 `scripts/agents/cite-check.py` 是直接跑的文本工具，交回前已跑，末行贴在交回里。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务；没有副本上的数。
- 没建仓副本与编译目录；草稿目录里只有 `cite-check.py` 的输出，没有要删的副本。
- 禁读清单里的文件（正推腿的报告、模型与草稿目录，别的四题第三轮各腿的产出与草稿目录）没读。
