# 规则压缩第四版（最后一次修稿）结果

日期：2026-11-16。批次：规则压缩的最后一次修稿（调度记录 `records/2026-11-16-三方豁免取消与存量补轮.md` 里「压缩 2i」）。草稿目录 `~/.local/share/singlefs/work/rules-compress-fourth/`。仓里的文件一个字没改（只读仓、只写草稿目录）；没跑项目代码、没交任务、没交命令给主 agent。

交付物（都在草稿目录）：

| 文件 | 是什么 |
|---|---|
| `fourth/` | 第四版合稿树，整棵：`CLAUDE-merged.md`、`dot-claude/`（连 `CLAUDE-merged.md` 共 51 份）、`contradictions.md`、`references.md`；树里 `dot-claude/` 即仓里 `.claude/` |
| `dispositions-r3.md` | 十份报告的逐条去向，59 行 |
| `diff-from-r3.patch` | `diff -ruN r3-tree fourth` 的原样输出，692 行，回打到 `r3-tree` 的拷贝上与 `fourth/` 逐字相同（`diff -r` 无输出） |
| `install-map.tsv` | `fourth/` 里每份换到仓里的哪条路径，8 份要 `git rm` 的另列一节 |
| `r3-base/` | 第三版冻结树（`research/prompts/rules-compress-r3-merged/`）的原样拷贝 |
| `r3-tree/` | `r3-base/` 去掉三份第三轮自己的报告件（`diff-from-r2.patch`、`dispositions-r2.md`、`revise-result.md`），补丁以它为基线；这三份不属于合稿树，第四版不带 |
| `selfcheck-*.txt`、`tools/` | 自查输出与只读核对脚本 |

## ① 十份报告各多少条，与去向表的四个数

按各腿自己的编号数（打中、观察、「后改」、多出来的半句、核不动各算一条）：

| 报告 | 条数 |
|---|---|
| entry·攻方（opus） | 7 |
| entry·正推（sonnet） | 9 |
| verification·攻方 | 8 |
| verification·正推 | 12 |
| residents·攻方 | 4 |
| residents·正推 | 6 |
| definitions·攻方 | 5 |
| definitions·正推 | 12 |
| contradictions·攻方 | 12 |
| contradictions·正推 | 22 |
| 合计 | 97 |

两条腿说同一件事的并成一行，97 条并成 `dispositions-r3.md` 的 59 行。末尾四个数由 `tools/dispo_build.py` 数、`tools/dispo_check.py` 对着写出的表逐行回数，一致（`selfcheck-dispositions.txt`）：

| 共 | 成立 | 不成立 | 部分成立 |
|---|---|---|---|
| 59 | 44 | 12 | 3 |

## ② 用户定案第 27 到 30 条落在哪

行号是 `fourth/` 里的（`.claude/` 即 `dot-claude/`）。

- 27（改完再攻的那一轮攻什么）：`.claude/rules/implementation-workflow.md:31`「改完再攻的那一轮，攻上一轮的修补，加前一轮在正文或判决里写明只抽查过、没看过的地方；已攻过而没打中的角度不重复，派发提示里明令写上」；「那一轮只攻上一轮的修补」删掉。原件三方规则不分轮的种类、管三类轮的「不许重复已攻过的角度」与「攻击腿撤回自己先前给的方案更有价值」同在这一行放回（三面清单只属设计推论轮，不挪）。
- 28（代码轮与治理文件轮的辩方腿）：同一份 `:32`「开了第二轮的，在第二轮派一条辩方腿复核第一轮的判决」，指到三方规则；设计推论轮的辩方腿时点放回原件写法 `.claude/rules/three-way-inference.md:45`「其中至少一轮派一条辩方腿去复核已有的判决」，指回实现流程那一节；两处互相指过去。
- 29（设计推论轮的停法）：`.claude/rules/three-way-inference.md:45`「第一轮有打中的或结论不收敛的，才开第二、三轮……同一个结论攻满三轮、按多数判……第二轮没打中也照开第三轮」。
- 「停」的判据只住一处：`.claude/rules/three-way-inference.md:72`「这是「停」的唯一判据：管第一轮，也管代码轮与治理文件轮的每一轮；设计推论轮只在第一轮这样停」；三方规则「开不开下一轮」（`:45`）与实现流程（`.claude/rules/implementation-workflow.md:29`）只指过去，原先三处写三种的状态没了。设计推论轮的定义只留 `.claude/rules/three-way-inference.md:43` 一份，实现流程 `:27` 改指过去。
- 30（Python 循环里每一轮停自己用新会话起的组）：循环分两种写——按进程号逐个发（shell 的 `kill`、`os.kill`）不许，Python 里每一轮 `os.killpg` 只停自己用新会话起的那一个子进程的组可以，出处写钩子代码 `SESSION_CHILD_CALLS` 与 `OWN_GROUP_TARGET` 两段注释：`.claude/rules/command-safety-by-review.md:9`、`.claude/agent-common.md:68` 与 `:81`（⑥）、`.claude/agents/tooling-writer.md:33`；`setsid` 统一成钩子认的 `setsid [--wait] <命令> &` 加紧跟的 `名字=$!`（run_in_background 里跑的脚本另带 `--wait`，写在共用约束 `:68` 括号里）。
- 主 agent 处置的两句留着、不记在用户定案名下：「现查推翻了打中的那一条不改、不为它开一轮」`.claude/rules/implementation-workflow.md:33`，「第三轮又打中的改了收」`:34`（另写：修补标「零轮」、要写进产品代码的先做 A/B，原件三方规则「写进判决的交用户表、标「零轮」」的写法）；总表依据栏记「主 agent 处置-13」「主 agent 处置-14」。

## ③ 不采纳的每一条与理由（12 条不成立、3 条只部分采纳，共 15 行；理由取自读到的原文）

- 去向表第 9 行（入口·攻 观察-2；不成立）：修稿报告里一句话与树不合（`gate.sh --staged` 的 `gate-staged.sh` 带不带 `--full-gate`）。不改：那份修稿报告不进第四版（第四版的报告是 `result.md`），树里 `staged-gate.md` 写的是带 `--full-gate`，对
- 去向表第 11 行（入口·正推 H2；不成立）：合入后验证技能里「第 2 步还没做过，只重交第 1 步」原件没有这半句。不改：是「退 2 整批重交」的自然推论、不改变做法，第二版起就有，腿自己判不挡换上
- 去向表第 16 行（验证·攻 二-4；部分成立）：设计推论轮与治理文件轮的定义交叠：为推翻一条决策而改规则的一轮两边都落得进（推的）。加一句拆题：.claude/rules/three-way-inference.md；腿自己标「推的」、没有实际正文佐证，登为主 agent 处置-15
- 去向表第 17 行（验证·攻 三-1；部分成立）：「三轮各换一组攻击面，不许重复已攻过的角度」与「攻击腿撤回方案更有价值」原件管所有三方轮，第三版只留在设计推论轮的段里。两句放回代码轮与治理文件轮：.claude/rules/implementation-workflow.md；三面清单（过度外推、修补本身、「已定」项）只属设计推论轮，不挪
- 去向表第 19 行（验证·正推 E；不成立）：「也不为它开一轮」定案没明说。不改：从用户定案第 17 条与三方规则「现查推翻而不采」推得出、做法与「没新打中就收」相同，用户指示留着；总表记主 agent 处置-13
- 去向表第 20 行（验证·正推 F；不成立）：三方规则「一条腿只抽一次样」括号「（分工只分给其中一条的格、只有一条读了的文件）」是修稿给的举例。不改：举例不冲突、不改变做法
- 去向表第 23 行（验证·正推 H；不成立）：原「停是不开下一轮，不是已经能拿这个没打中去支撑别的结论」被用户定案第 26 条替掉。不改：用户定案 26 已登，不是丢失
- 去向表第 31 行（常驻·正推 六 #9、4.1；不成立）：`code-discipline-by-review.md` 第 4 行「只写上游没写的」原件没有、第一遍压缩带进。不改：只是范围说明、不规定做法，腿自己判不算打中
- 去向表第 32 行（常驻·正推 4.4、六 #2；不成立）：「它停任务，不属于「检出和处置分开」那一条说的检出」是改后多出的半句。不改：只消歧、不改变做法（与用户定案第 21 条同向）；指针部分已在 stalled 那一行改
- 去向表第 40 行（定义·正推 R8；不成立）：调查员补测试要不要写 `refused-test-registry.py list`，两版都没说。不改：两版相同的空白，补了就是给调查员新加一条义务（原件没有），属新条款；那一支不新写崩溃枚举用例、拒算的核对留给实现员路径
- 去向表第 42 行（定义·正推 H5；不成立）：定时闸一项多出「子 agent 要等什么写进报告交主 agent」。不改：由名单推出、总表定义-13 已登，不改变做法
- 去向表第 43 行（定义·正推 指针清单；不成立）：共用约束第 67、80 行指到的规则三方腿起步读不到。不改：共用约束「规则怎么读」许整读一次没被点名的文件，节标题存在且唯一
- 去向表第 56 行（矛盾·攻 核不动 2；部分成立）：证红命令模板在副本里起副本那份脚本，会不会被 `prove-red.sh` 当主工作区拒。静态读 `scripts/mutation/prove-red.sh`：`MAIN_ROOT` 取脚本所在目录往上两层、`copy == main_root` 退 2，模板有此风险；没跑、不是第三版引入（原件第 3a 步同写法），模板不动，登进 `references.md` 四「要核的」交主 agent
- 去向表第 57 行（矛盾·正推 核不动；不成立）：验证-13、验证-14 的门禁真实行为、用户定-20 的钩子行为、用户定-24 的服务端收不收，要起门禁或钩子才判得了。不改：只读、没有要改的句子；那几句都照文件头今天的原文写，要跑的不在这一件的范围
- 去向表第 58 行（矛盾·正推 入口-16；不成立）：入口-16 说明里「五条判据各收成入口或技能一处」只核了拒算那一条。不改：没有找到不一致，只是没逐条核；落点（needs-attention、control check、`--check-facts`、调查员例外）本版逐处读过都在

核不动的三类（要起门禁、钩子或服务才判得了）也在上面：验证-13、验证-14 的门禁真实行为，用户定-20 的钩子行为，用户定-24 的服务端收不收；没有要改的句子，登在 `references.md` 四「要核的」的只有证红命令模板那一条。

## ④ 要用户定的

没有现在就必须用户一句话的。有两处是这一版新写的、腿推的或主 agent 处置的句子，进第三轮判决的交用户表（用户定案第 23 条：第三轮又打中的改了收、写进判决的交用户表、不开第四轮），换上不等它们：

1. 三方规则「多轮」一节新加的一句「一题只属一类：同一件事既要推翻或确立决策、又要照它改治理文件的，拆成两题，各按各的类收」（`.claude/rules/three-way-inference.md:43`，总表主 agent 处置-15）：攻方二-4 推的，没有实际正文佐证；不要这一句就删它，设计推论轮与治理文件轮的两种收法仍各写各的。
2. 实现流程「第三轮又打中的」那一条加的「修补标「零轮」、要写进产品代码的先做 A/B」（`.claude/rules/implementation-workflow.md:34`，总表主 agent 处置-14 与验证-22）：照原件三方规则「标零轮、写进交用户表」与「腿提的、判决里标零轮的改法写进产品代码之前先做 A/B」两句并起来的读法；用户若要第三轮的修补直接写进代码、不做 A/B，删这半句。

另有一处不是问用户的、是要主 agent 判的：证红命令模板 `--cwd <副本>` 加副本里那份 `prove-red.sh` 会不会被它当主工作区拒（静态读 `scripts/mutation/prove-red.sh` 是会退 2，没跑，原件第 3a 步同写法）；改不改实现员第 3a 步与调查员第 6 步两处模板，等探针结果，登在 `references.md` 四「要核的」。

## ⑤ 字数（`tools/wordcount4.py`，字符数，`selfcheck-wordcount.txt`）

常驻：`CLAUDE.md`、`.claude/main-agent.md` 与 `CLAUDE.md` 里 `@` 点名的规则；按需读：其余（共用约束、14 份定义、技能、不 `@` 的规则、参考件）。

| | 常驻字数 | 常驻份数 | 按需读字数 | 按需读份数 |
|---|---|---|---|---|
| 仓里原件 | 123271 | 21 | 180250 | 34 |
| 第三版 | 73874 | 16 | 191988 | 35 |
| 第四版 | 75078 | 16 | 193716 | 35 |

第四版比第三版常驻多 1204 字（入口：提交之外按标记分、写段第 6 条、fmt 与 clippy 与 build、参考件读的时机；规则：验证规则「提交之外」、实现流程「代码轮与治理文件轮怎么收」与「只改文档和脚本除外」、三方规则「一条腿只抽一次样」「回复只写短句」与设计推论轮的拆题、证红那一句、`command-safety-by-review.md` 的循环两种与指针）、按需读多 1728 字（共用约束的简单任务一条、批次代号、闸项几处与进程停法；调查员的命令落点、实现员的建副本与证红、工具员第 5 步；技能 dispatch 与 staged-gate；参考件两处）。比仓里原件常驻少 48193 字，按需读多 13466 字。

## ⑥ 自查输出在哪（都在草稿目录，`tools/finalize.sh` 一次重出并跑完）

| 自查 | 输出 | 结果 |
|---|---|---|
| 总表回查（只认写的那一行逐字含片段；编号重复也算） | `selfcheck-contradictions.txt`，命令 `tools/contra_check.py fourth/contradictions.md fourth` | `rows=130 users=30 dispositions=15 refs_ok=513 problems=0` |
| 指针（`文件`「节」）指得到且唯一，指上游的对着 `.claude/singlefs-ai-sop/rules/` 今天的标题 | `selfcheck-pointers.txt`，`tools/pointer_check.py` | 标题 353、前缀 21、加粗条 34、正文 3；找不到 0、重复 0 |
| 钟点、会话号、批次代号、撇号角标（用仓里 `lib-forbidden-notations.py` 的 `all_hits`，`fourth/` 全树含总表与改法清单） | `selfcheck-notation.txt`，`tools/notation_scan.py` | 0 处 |
| 位置指代与自称（只算新增） | `selfcheck-position.txt`，`tools/pos_scan2.py` | 新增 0 处；全树 31 处命中都是子串（「上下文」里的「下文」、「脚本文件」里的「本文件」、「历史版本节」里的「本节」）与第三版已有的句子 |
| frontmatter 与第三版相同 | `selfcheck-frontmatter.txt`，`tools/frontmatter_check.py` | 24 份里只 `dot-claude/skills/dispatch/SKILL.md` 一份不同（description 写成同一组触发时刻） |
| 第三轮十份报告点名的原件原话在 `fourth/` 里按字面找得到落点 | `selfcheck-quotes.txt`，`tools/quote_check.py` | 44 处引文，原件里逐字在 41，其中在 `fourth/` 里逐字在 23；其余 18 处是第四版有意改写或挪走的句子（用户定案第 18、20、24、30 条与去向表各行），每一处对得上去向表的行；原件侧对不上的 3 处是腿转述 |
| 四个数 | `selfcheck-dispositions.txt` | 59 行 / 44 / 12 / 3，与表逐行回数一致 |
| 补丁回打 | 命令：`patch -s -p1 -d <r3-tree 的拷贝> < diff-from-r3.patch` 再 `diff -r <拷贝> fourth` | 无输出（逐字相同），`sha256sum diff-from-r3.patch` 是 `3f6fe850f58e92e6cc218c2dc8188f3cfb3ab407974a9d64e56316a985b395ec` |

## 没做什么

- 没改仓里任何文件，没 `git` 写操作；`install-map.tsv` 与 `references.md` 是交给主 agent 换上那一步用的。
- 没跑门禁、钩子与自证，没往任务服务交任务，没交命令给主 agent；证红模板那一条要探针，登在 `references.md`。
- 没替用户定、没走定义三方（换上之后由主 agent 开）。
- 原件第 3 步以外：`.claude/hooks/agent-write-scope.tsv` 没有给 `tooling-writer` 写 `.claude/skills/` 的一行，这是第三版就登在 `references.md` 四的脚本那一批的活，这一版没动。
- 草稿目录里没有仓副本、编译目录与库拷贝；`r3-base/`、`r3-tree/`、`fourth/` 是纯文本树（草稿目录合计 3.7 MB，`du -sh` 量的）。
