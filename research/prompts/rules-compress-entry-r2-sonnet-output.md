# rules-compress-entry-r2 辩方腿报告

日期：2026-11-16。轮名 rules-compress-entry-r2，第二轮，立场辩方。复核对象：第一轮判决 `research/prompts/rules-compress-entry-r1-main-verification.md`，它依据的两份腿报告 `research/prompts/rules-compress-entry-r1-opus-output.md`、`research/prompts/rules-compress-entry-r1-sonnet-output.md`。

没写模型，没有 rerun.sh 与模型目录。草稿目录 `~/.local/share/singlefs/work/rules-compress-entry-r2-sonnet` 里只有两份 cite-check 输出（`cite-r1-legs.txt`、`cite-r1-legs-2.txt`）与三份从文件截出来读的片段（`patch-main.txt`、`patch-skills.txt`、`claude-diff.txt`）。

## 先核的：被判的东西没变（sha256sum 现跑，量的）

- 原件：`CLAUDE.md` 87510a525cc97c082a47c119c95c5f7c76e2a59faa24cbc7a6a8bc42ddb048a7，`.claude/main-agent.md` c310eb2f472bca9172c9e5bb5d0be21f0da5c3a752191ffb90014e41b5293bc9，与 `research/prompts/rules-compress-entry-r2-snapshot/judged-sha256.txt` 逐份相同。
- 第一版合稿四份（`CLAUDE-merged.md` ca5c5a6f…、`dot-claude/main-agent.md` ff025997…、`dispatch/SKILL.md` ca378a40…、`merge-verify/SKILL.md` 253db006…）与 `research/prompts/rules-compress-entry-r1-snapshot/merged-tree-sha256.txt` 逐份相同。
- 第二版合稿四份（`CLAUDE-merged.md` abc9333f…、`dot-claude/main-agent.md` 765f3edb…、`dispatch/SKILL.md` 09b62836…、`merge-verify/SKILL.md` bc59789d…）与 `research/prompts/rules-compress-entry-r2-snapshot/merged-tree-sha256.txt` 逐份相同。

下文行号：原件是仓里 `.claude/main-agent.md`、`CLAUDE.md`；第一版、第二版是 `research/prompts/rules-compress-r1-merged/`、`research/prompts/rules-compress-r2-merged/` 下的 `dot-claude/` 与 `CLAUDE-merged.md`。

## 一览

| 判决条 | 辩方判定 | 一句话 |
|---|---|---|
| 1 丢了事实表验收句 | 站得住 | 原件有、第一版 0 命中、没有别处替它守 |
| 2 技能「读一次」遇压缩上下文 | 站得住（程度中等） | 第一版两句并存是真的；但第一版逐步的「先读」已罩住大半，同形状在 CLAUDE.md 的读一次清单上原样还在 |
| 3 「只有两类」 | 部分站得住 | 「只有」是新字，但原件「其余按下面那张表派」是同一个封闭世界；第二版多走了半步，加了新判据 |
| 4 「整批重交」统一定义 | 站得住（弱） | 原件两处写法确实不同；两种读法在第 1 步红支做法几乎一样，在第 3 步退非 0 才有差 |
| 5 「用户要求的照跑」只改一头 | 站得住 | 原件两句本来不齐；第二版五处已指到同一句；残留的「（或用户要求时）」不矛盾 |
| 6 冻结层归属三条判据换成指针 | 站得住（轻） | 指到的格名表那一格有三条的概括，只缺「每棵树」「每个单元类」两个枚举 |
| 7 细则出了工具员写范围与四道检查的射程 | 站得住，但不是新缺口的全部 | 射程外是真的；技能目录原本就在这几处之外，新的是落进去的细则变多；登记处够不够见下 |

其余三块的结论：第二版修补，第 1、2、4、5、6 条够得着且没改错，第 3 条够得着字面但多走半步（新增半句是新判据），第 7 条只登记、没改文字；第二版新写的句子里另有两处读得出两种（技能第 4 条括号、入口第 9 行新半句）。不采纳的入口题三行：20、21 行理由成立，22 行就崩溃点那一句成立、整包确认那一条的常驻缩减没被它答到。核查方式：主 agent 没自己跑 cite-check、现查清单没盖第 4、6、7 条，我补跑了 cite-check（721 处引文无红）并补读了这三条。

## 逐条复核

### 1 丢了主 agent 一侧「交回的事实表过了 --check-facts」的验收句：站得住

- 原件：「交回的事实表过了 scripts/kb/stale-candidates.py --check-facts」（`.claude/main-agent.md:121`），句子在派发表「一批阶段任务结束」那一行，是主 agent 往下切候选表之前的条件。
- 第一版：入口那一行只写「`sweep` 写事实表 → 候选表按组切段，一段派一个」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:95`）；技能同名条目第一项没有它：「事实表那一趟，派发提示的输入给改动范围与做成的事」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/dispatch/SKILL.md:75`）。两份 `grep -c check-facts` 都是 0（量的）。
- 替第一版辩：回扫员定义自己写了「写完跑 stale-candidates.py --check-facts 事实表 --base 基准」（`.claude/agents/sweep.md:40`），交回物里带输出末行，所以主 agent 看得到「过了」。辩不住的理由有三：
  1. 那是回扫员的步骤，原件这一句是主 agent 收货的条件，不是同一件事；同一形状的第二段 `--check-report` 技能里留了、第一段没留，不对称。
  2. 入口自己写「agent 的结论句一律当线索」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:78`），原件同句「agent 的结论句一律当线索」（`.claude/main-agent.md:98`）：回扫员说它过了不能当主 agent 的验收。
  3. 没有机制替它守：`grep -rn check-facts` 钩子（`.claude/hooks/*.sh`、`*.py`）与 `scripts/agents/*.py` 都没有命中，命中只在 `scripts/kb/stale-candidates.py` 自己与回扫员定义。
- 推翻它的现象：交回闸或续派闸对回扫员交回物逐项核 `--check-facts` 的输出末行。没找到。

### 2 技能「整份读一次」「第一次派发之前」遇压缩上下文：站得住（程度中等）

- 第一版两句并存：「到那一步之前整份读一次」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:5`）与「会话过 40 万上下文就压缩上下文接着做」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:35`）。原件整份常驻（`CLAUDE.md` 用 `@` 引 `.claude/main-agent.md`），压缩之后还在；第一版把细则挪进按需读的技能，压缩之后技能正文不保证还在。这一点两条腿结论相反（攻方说不够，正推说够），判决取不够。
- 替第一版辩（正推腿的理由）：逐步的入口句「做这一步之前先读」第一版已有（`grep -nF 做这一步之前先读` 在第一版 `dot-claude/main-agent.md` 命中 24、27、31、34、38、57、58、59、61、68、69 行，第二版同样那些行），压缩之后走到那一步照样会再读一次。这条理由成立一半：验证段各步、拒算各条、「一轮怎么开、怎么收」五条（第 1、4、8、11、15 条）都有；「派出去之后」「交回怎么读」「派发提示怎么写」三节只写「先读」、不写哪一步之前，靠第 5 行那一句兜底。所以第 5 行「读一次」与逐步「先读」相抵时，读法甲（一个会话读一次）会漏掉后三节，不是整个流程。判决要求的修补站得住，严重度偏中。
- 同一形状没处理的地方（对辩方有用，对判决是补充）：`CLAUDE.md` 的读一次清单压缩之后同样会掉：原件「开会话先整份读一次、之后按名字回查」（`./CLAUDE.md:34`）、「主 agent 碰到对应的活之前先整份读一次」（`./CLAUDE.md:68`）、「开会话先整份读一次、之后按名字回查」（`./CLAUDE.md:70`）；第二版「开会话先整份读一次、之后按名字回查」（`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:33`），另一处「开会话先整份读一次、之后按名字回查」（`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:64`）。那是压缩前就有的做法，不是压缩带来的，第二版没动。
- 推翻它的现象：Read 过的技能正文在压缩之后保留在上下文里（运行时行为，这一轮没量，推的）。

### 3 「主 agent 可以自己写的只有两类」：部分站得住

- 原件同一行有两半：「其余按下面那张表派」与「可以自己写（派发表」（都在 `.claude/main-agent.md:7`）。原件没有「只有」，但前半已经把主 agent 自己留着的几样之外全交给派发表，是封闭的写法。
- 第一版：「主 agent 可以自己写的只有两类」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:9`）。
- 替第一版辩：乙读法（主 agent 一个别的文件也不自己改）在原件里已经成立；它与共用约束「简单任务不派发，自己直接改」（`.claude/agent-common.md:8`）的对不上也是原件就有的。「只有」没有造出新做法，只是把封闭写法写显眼了；正推腿把它记成观察、不算打中，理由是照派发表逐行做不会做错。所以判决写「多了『只有』」字面对、说它改了做法站不稳。
- 判决要去掉它仍有站得住的一面：新字让乙读法更顺，且第一版总表里 grep -cF 只有两类 得 0（量的），第二版总表才登了「入口-4」（`research/prompts/rules-compress-r2-merged/contradictions.md:14`）。
- 第二版写「别的文件的简单任务」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:9`），接着指到共用约束「派发」一节、自己直接做。那一节存在：「主 agent 派发：简单任务」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:11`）。够得着乙读法，没把对的改错；但这半句不是放回原话，而是把原件封闭写法与共用约束之间原有的冲突按共用约束一边定了：派发表那一行「改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:99`）没有大小条件，门禁与钩子的一行小改算不算「别的文件的简单任务」仍读得出两种。用户定案 1 到 22 里没有这一条。交主 agent 判：留作新判据并登进总表，或只去掉「只有」。
- 推翻我这条辩护的现象：有一份压缩前的流程记录写明主 agent 不得自己改治理文件的小处（我没找到）。

### 4 「整批重交」统一定义套到第 1 步红支与第 3 步退非 0：站得住（弱）

- 原件两处写法不同：第 1 步退 1 那一支写「照它的出路整批重交」（`.claude/main-agent.md:57`），第 3 步核退非 0 那一支写「第 2 步在新留下的快照上重验，再回到这一步」（`.claude/main-agent.md:59`）。
- 第一版统一成「第 1 步连同修补的补丁目录整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:14`），第 1 步红支沿用「照它的出路整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:23`），第 3 步写「退非 0 不合，整批重交」（`research/prompts/rules-compress-r1-merged/dot-claude/skills/merge-verify/SKILL.md:37`）。
- 替第一版辩：第 1 步红支，第 2 步本来就在第 1 步绿之后才做，读乙（连第 2 步那几条也要重交）与读甲（只重交第 1 步）落到的动作一样，没有差。第 3 步，「连同修补的补丁目录」没有修补时是空的，正推腿同样把它记成「观察，不算打中」（`research/prompts/rules-compress-entry-r1-sonnet-output.md:432`）。所以「读得出两种」成立，「做法因此不同」只在第 3 步退非 0 时有可能（把「要有修补才重交」读进去，会为一次主工作区被别人改动去派实现员），概率低。
- 第二版分开写：「第 1 步把这一批的补丁目录（有修补的连同修补的补丁目录，没有的原样）整批重交」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:14`），并写「第 1 步自己红了、退 2 的，第 2 步还没做过，只重交第 1 步」（同一行）。与原件 :57、:59 的字面一致，没改错。残留：「第 1 步自己红了」读成退 1 时，退 1 支写的是只重跑红的整条命令（同文件 :23）；但定义只管写了「整批重交」四个字的地方，退 1 支那一句本身没写，不冲突。
- 推翻它的现象：第 3 步退非 0 时主 agent 没有修补也照原样重交（做法只有一种）。

### 5 「用户要求的照跑」只改了入口一头：站得住

- 原件两句本来不齐：「不指定范围的只在提交时或用户要求时由主 agent 经服务交」（`.claude/main-agent.md:18`）与「提交之外任务确实要跑，先弹窗问用户，同意了才跑」（`.claude/main-agent.md:26`）。第一版入口写「提交之外，用户要求的照跑」（`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`），别处还是先弹窗。用户定案第 18 条已定，不辩该不该。
- 第二版：入口写「提交之外要跑不指定范围的」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:26`），后接指针指到规则那一条；句子只住一处：「提交之外要跑（包括用户要求的那一趟），一律先弹窗问用户，同意了才跑」（`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:38`）。我 grep 了第二版全树：指到它的是共用约束、崩溃技能、实现流程规则、收尾规则四处加入口，没有留着「用户要求的照跑」。
- 残留的「用户要求时」不算矛盾：「只在提交时（或用户要求时）跑这六道」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/crash-test/commit-stages.md:21`）说的是触发时机，弹窗由 `--full-gate` 那一次兜底；规则那一句已写「提交时那一趟的确认就是带 --full-gate 的那一次弹窗」（`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:38`），原件整轮门禁技能同样写「这一条会弹窗要用户确认」（`.claude/skills/gate/staged-gate.md:16`）。
- 推翻它的现象：第二版里有一句让用户开口要跑的那一趟直接交、不弹窗（没找到）。

### 6 冻结层归属登记表三条判据换成指针，指针缺两项：站得住（轻）

- 原件：「每层、每棵树、每个单元类都有行」（`./CLAUDE.md:90`）。第一版：「写在 .claude/gate.d/doc-decisions.sh 文件头格名表那一格」（`research/prompts/rules-compress-r1-merged/CLAUDE-merged.md:84`），括号里的三条没有了。
- 指到的格名表那一格写的是「冻结层归属登记表：每层每个结构都有行」（`.claude/gate.d/doc-decisions.sh:22`），后面还有退出必须派生态、依据指得到已定分项两条的概括。函数上方注释里写「每棵树在 key 编码层与索引节点内部布局层各至少一行」（`.claude/gate.d/doc-decisions.sh:1851`），不在文件头格名表。
- 替第一版辩：指针点名的地方有三条的概括，「每层每个结构」把树与单元类的意思含进去了；判决说「缺两项」只对「每棵树」「每个单元类」两个枚举成立，不是整条判据丢了。正推腿本来也记「轻」，并说这是项目事实表里描述一道门禁的句子、不是主 agent 照做的步骤。照原话放回成本为零，判决站得住。
- 第二版放回：括号里三条逐字照原件：「每层、每棵树、每个单元类都有行」（`research/prompts/rules-compress-r2-merged/CLAUDE-merged.md:84`），另加指针「三份清单怎么现读见」（同一行）。指到的小节在：「每个层与每个结构在表里都有行，漏一个判红」（`.claude/kb/freeze-layer-membership.md:115`，标题是「门禁判哪三条」，同文件 109 行）。没改错；原件「格按它判三条」变成「格判三条」，少了「按它」，不影响判据。
- 推翻它的现象：文件头格名表那一格逐字写着「每棵树」「每个单元类」（`grep -n 每棵树 .claude/gate.d/doc-decisions.sh` 命中从 1851 行起，文件头没有）。

### 7 细则出了工具员写范围与四道只认 main-agent.md 的检查的射程：站得住，但要分清哪些是新缺口

- 写范围：工具员那几行是 33 到 45 行，没有技能目录（`grep -c skills .claude/hooks/agent-write-scope.tsv` 得 0，量的）；第二种活那几行是入口、共用约束、定义与规则，不含技能：「第二种活」（`.claude/hooks/agent-write-scope.tsv:43`）。
- 四道检查只认入口与共用约束：「agent-common\.md|main-agent\.md」（`.claude/gate.d/doc-process-records.sh:2272`）、「judged_files = definitions + [path for path in (".claude/agent-common.md", ".claude/main-agent.md")」（`.claude/gate.d/code-tooling.sh:4153`）、「GOVERNED_FILES = (".claude/agent-common.md", ".claude/main-agent.md")」（`.claude/gate.d/lib-lag-baseline.py:235`）、「CODE_FILES = (".claude/main-agent.md", ".claude/agent-common.md")」（`.claude/gate.d/doc-text.sh:1953`）。四处行号与原文现读都在。
- 替第一版辩：（a）技能目录压缩前就在这几处之外：原件 `CLAUDE.md` 自己写「其余几类还没有格判」（`./CLAUDE.md:22`），提交时六道门禁的流程压缩前就住在技能里：「只在提交时（或用户要求时）跑这六道」（`.claude/skills/crash-test/commit-stages.md:17`）。新的只是落进射程外的细则变多。（b）四道检查并不都该把技能收进来：doc-text 那一道是 account-quota 词表检查，它另有一行「CODE_DIRECTORIES = (".claude/hooks/", "scripts/", ".claude/gate.d/", ".claude/rules/", ".claude/agents/", "task-server/", "prune-server/")」（`.claude/gate.d/doc-text.sh:1952`）把钩子、脚本、门禁、规则、定义与两个服务列进来、没有技能目录，lib-lag-baseline 管搬迁分类，code-tooling 那一道判定义不抄规则；各有射程理由，哪几道该收技能是脚本那一批要判的，不是文字压缩要判的。（c）调度记录出口 1 已把「规则、技能、钩子、门禁、脚本与两个服务收进被管集合」列为堵口那一批的活：「agent-def-adversarial-review 格把规则、技能、钩子、门禁、脚本与两个服务收进被管集合」（`records/2026-11-16-三方豁免取消与存量补轮.md:6`），doc-process-records 那一道（2272 行）本来就要改；剩下写范围表与另三道。
- 判决站得住的部分：压缩前判决打中之后派工具员改入口文件改得了、改完没走三方那一格会红；压缩后细则在技能里，工具员写闸拒（按写范围表推，没跑）、自己改又没有哪一格红。这是实实在在多出来的联动改动。
- 判决的处置够不够：判决把它「登进引用改法清单，归脚本那一批」。第二版在引用改法清单里加了一节，其中写「在这一批换上之前，改这三份技能的活派不出去」（`research/prompts/rules-compress-r2-merged/references.md:171`）。两处要主 agent 看：
  1. 登记位：入口自己写延后项要登进「里程碑收口表、checks-owed、决策正文」：「不挡就延后，并在判的同一步登一行」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:25`），引用改法清单是这一轮的产物，不在其列；这一件只在清单里、调度记录依赖表与欠账表里都没有（`grep -n 写范围 records/2026-11-16-三方豁免取消与存量补轮.md` 只命中一行，说的是实现员写范围表，不是工具员）。
  2. 清单里写的「:33 到 :36」偏窄：「的几行（:33 到 :36）」（`research/prompts/rules-compress-r2-merged/references.md:168`），实际是 33 到 45 行；不影响结论，换上时按 45 行核。
- 推翻它的现象：写闸表另有一行放行工具员写技能目录（grep 得 0，没有）。

## 修补够不够得着（第二版 vs 第一版，逐条）

| 判决条 | 第二版改在 | 够得着 | 有没有把对的改错 / 新问题 |
|---|---|---|---|
| 1 事实表验收句 | 入口第 95 行、派发技能第 75 行 | 够得着：原话「交回的事实表过了 scripts/kb/stale-candidates.py --check-facts」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:95`），逐字放回；技能第 75 行同样有这句 | 没改错；入口写「才往下」、技能写「才切候选表」，同一个意思写了两遍，不矛盾；原件那一句「过了」由谁跑仍没写（回扫员自己的输出末行还是主 agent 重跑），与原件一样模糊 |
| 2 读一次与压缩 | 入口第 5 行，两份技能 frontmatter 第 3 行 | 够得着：「每次开始『写与验分两段』验证段之前读」「技能正文不在此刻的上下文里就整份重读」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:5`） | 没改错；两句并排，读成「每次都读」与「不在才读」结果一样（多读一遍无害）。同一形状在 CLAUDE-merged 第 33、64 行的读一次清单上没动（见「辩方顺带看到的」） |
| 3 只有两类 | 入口第 9 行 | 够得着字面，多走半步 | 新增半句是新判据，见第 3 条复核 |
| 4 整批重交 | 验证技能第 14 行 | 够得着 | 没改错，见第 4 条复核 |
| 5 用户要求 | 入口第 26 行、规则 verification 第 38 行及另四处指针 | 够得着 | 没改错；残留「（或用户要求时）」不矛盾 |
| 6 冻结层判据 | CLAUDE-merged 第 84 行 | 够得着：三条逐字放回 | 没改错 |
| 7 射程 | 引用改法清单新增一节，文字不动 | 只登记，不修；够不够取决于登记位 | 清单行号偏窄；登记位不在入口规定的几处 |

### 判决「其余打中的」三条与用户定案 19 在第二版里的落点（辩方顺手核）

判决只点名七条，其余腿的打中进了修稿去向表。同一组文件上还有四处第二版动了，一并核：

| 条 | 原件 | 第二版 | 判 |
|---|---|---|---|
| 「主 agent 也不替它登记」主语（攻方义-2） | 「实现员不登记，也不替它登记」（`.claude/main-agent.md:68`），同一节开头已写「不另往哪一棵树里手写这一行」（`.claude/main-agent.md:66`） | 「实现员不往拒算表里写，主 agent 也不往哪一棵树里手写，收的行只走拒算补丁目录」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:68`） | 够得着，与原件开头那句同一个意思，没改错 |
| 拒算清单罩不到的三类（攻方驻-2） | 原件最后一段写「挂特性的测试文件、产品 GPU 包的测试包」（`.claude/main-agent.md:72`） | 「这三类审 diff 时照实现员定义第 3c 步看有没有读活树」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:68`），第 3c 步在：「3c. 新写的测试默认要算得出单元键」（`research/prompts/rules-compress-r2-merged/dot-claude/agents/implementation-writer.md:49`） | 够得着；放进写段那一步，用的时刻与落点对上 |
| 只交任务、不派子 agent 的会话读不到 needs-attention（攻方驻-3） | 原件「派出去之后」末段写「时任务还在、不是结局」（`.claude/main-agent.md:94`） | 「`task wait` 以 `needs-attention` 退出时任务还在、不是结局」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:74`），回读 `task-server/control check` 也在同一行 | 够得着，原话放回 |
| 用户定案 19：调查员查到就修是例外 | 原件无 | 入口写「那一支是例外」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:47`）；技能写「那一支除外，在主工作区改；各在副本里改」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/dispatch/SKILL.md:14`） | 入口那句清楚；技能那句把例外塞进了「各在副本里改」前面的同一个括号，「各」指实现员还是也含调查员，读得出两种，建议拆成两句（新问题，轻） |

## 不采纳的几行（入口题）复核

`research/prompts/rules-compress-r2-merged/dispositions.md` 入口题里写「不成立」的三行（20、21、22 行），没有写「部分」的。

### 第 20 行：攻方说原件括号里几句脚本行为技能没抄，修稿判不算丢：理由站得住

- 原件那几句：退 3 时别的步绿、验证包带 --release、三样都没有时报本次未跑，与「汇总行之后没有第 2 步与合进主工作区那两段」（`.claude/main-agent.md:57`）。
- 脚本文件头今天有：「验证包 singlefs-verification 的目标带 --release 跑」（`scripts/gate/merge-and-verify.sh:59`）、「判不出改到哪个测试目标」一条写「测试那一步与验证包那一步没跑，别的步照跑、没有红」（`scripts/gate/merge-and-verify.sh:210`）、「75 按条复用那一层连不上 prune-server」一条写「别的没有红」（`scripts/gate/merge-and-verify.sh:212`）、「带 --skip-apply 的那一趟（补丁已在主工作区上）与退 1、2、75 的都不打这几条」（`scripts/gate/merge-and-verify.sh:168`）、「三样都没有就不起那一道」（`scripts/gate/merge-and-verify.sh:227`）。
- 技能写了以文件头为准：「挑哪些目标、验证包那一步交什么、快照与先验后合，照 scripts/gate/merge-and-verify.sh 文件头」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:18`）；退 75 那一支保留了做法：「不算绿、不进第 2 步，等服务回来照原命令重交」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/merge-verify/SKILL.md:25`）。
- 我的保留：技能指向的是「挑目标、验证包那一步、快照」三块，没点名文件头的退出码表；「退 3 时别的步没红」在退出码表里，技能没写指针。但退 3 的做法（弹窗、整包跑、接第 2 步）不依赖这句，退出码 3 本身就说明「别的没红、有红退 1」，没有哪一步做法因此不同。理由成立，不该采。
- 推翻它的现象：有一步做法取决于「退 3 时别的步绿」「退 75 没有后两段」而文件头里没有这句。没找到。

### 第 21 行：攻方说「六道之间谁并行谁串行」没查完，修稿判找到了：理由站得住

- 原件入口派发表那一行写「qemu-device-streams、lkmm、formal-verification 与 crash-point-replay 并行」（`.claude/main-agent.md:122`）。
- 第二版落在提交技能的一行：「次序：qemu-device-streams」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/crash-test/commit-stages.md:22`），写同时交与「crates-mutation-replay 等 crash-point-replay 跑完再起」，第 2b 步写「2b. broad-mutation 等 crates-mutation-replay 跑完再起」（`research/prompts/rules-compress-r2-merged/dot-claude/skills/crash-test/commit-stages.md:27`）；入口指过去：「次序照那一份，各道判什么照」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:96`）。
- 攻方腿没找到的原因是搜「串行」：落点写的是「同时交」「等…跑完再起」。我另核了一件事：仓里原件提交技能里没有这一行（`grep -n 次序： .claude/skills/crash-test/commit-stages.md` 命中 0），所以这一行是压缩时新写进技能的，不是原来就在；它与原件入口那一句的意思一致，没有改。
- 推翻它的现象：入口派发表原来写的并行与串行关系，与技能这一行有出入。没有。

### 第 22 行：正推说「崩溃点测试准确率…」与跑整包的确认办法入口只留指针，修稿判不算丢：就崩溃点那一句理由站得住，整包那一条要补一句

- 原件入口写「要加崩溃点、要多枚举一档，就加」（`.claude/main-agent.md:10`），共用约束同一句「要加崩溃点、要多枚举一档，就加」（`.claude/agent-common.md:74`）。第二版入口留标题与「这一条管枚举域，不管跑的时机」，把那一句指到共用约束；共用约束里在：「要加崩溃点、要多枚举一档，就加」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:54`）。入口标题「准确率和正确率优先」已把方向说完，理由成立。
- 但同一行的第二半（跑整包的确认办法）不是同一件事：原件入口写「带它的提交弹窗要用户确认，对话不自己决定」（`.claude/main-agent.md:19`），第二版入口只留「三遍确认另写进调度记录」，弹窗那半句现在只在共用约束里：「带它的提交只由主 agent 起，弹窗要用户确认，对话不自己决定」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:65`）。入口有指针，但共用约束不在主 agent 的常驻里；第二版共用约束写的「做那一步之前读那一节，读全」（`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:17`）管的是定义，不管主 agent。主 agent 交整包之前会不会去读那一条，靠的是指针。这一点修稿的理由没答，建议交攻方腿在格三里判：原件入口是内联的，第二版是指针。
- 推翻我这条保留的现象：有规则明写主 agent 碰到「照 <共用约束>「<节>」」要先读那一节（我没找到）。

## 核查方式的缺口

判决写：「这一轮没有另跑 snapshot-tree.py」「任务服务排着 91 条、小任务起不来」「改成对腿点名的行逐处用 grep 与 sed 现读」（都在 `research/prompts/rules-compress-entry-r1-main-verification.md:7`）。照三方规则，核查要先倒推开工快照再跑 cite-check 带 `--root`，「核查与判决由主 agent 做，不派子 agent 去核子 agent」（`.claude/rules/three-way-inference.md:126`）。缺哪几处：

1. 倒推快照这一步对这一题其实不缺：原件两份今天的 sha256 与 `judged-sha256.txt` 相同，第一版合稿四份与 `merged-tree-sha256.txt` 相同（上文「先核的」，量的），树在仓里冻结，没有「开工那一刻的树」要倒推。真缺的是另一半：主 agent 没有自己跑 `cite-check.py`。它是文本工具，子 agent 直接跑的名单里就有它，不走任务服务，排着 91 条与它无关；判决只写了「两条腿交回时各自跑过」。我补跑了（直接跑，量的）：`python3 scripts/agents/cite-check.py research/prompts/rules-compress-entry-r1-opus-output.md research/prompts/rules-compress-entry-r1-sonnet-output.md --root research/prompts/rules-compress-r1-merged --root . --background research/prompts/_rules-compress-entry-r1-body.md`，末行「核了 721 处引文、29 处裸行号，对上 721 处，没判 29 处」，0 处对不上；没判的 29 处里 28 处是裸行号（没有引文可比，其中 14 处在正推腿那张「找不到的句子」表里），1 处是攻方腿报告第 33 行引的短名 dispatch/SKILL.md 第 77 行（在第一版树里是 `dot-claude/skills/dispatch/SKILL.md`，我另读过那一行，在）。输出在草稿目录 `cite-r1-legs-2.txt`；只给默认根跑一次的 `cite-r1-legs.txt` 同样 721 处对上，没判 29 处里 15 处是树里的相对路径认不出。所以「引文逐字在不在」这一层没有红；「引文是否撑得起腿的结论」仍只靠下一条。
2. 现查坐实的清单没盖满判决：记录里「主 agent 现查坐实的（对着合稿树与原件逐条 grep、读行，量的）：」（`records/2026-11-16-三方豁免取消与存量补轮.md:177`）一段对入口题只列了验收句 0 命中（`records/2026-11-16-三方豁免取消与存量补轮.md:178` 「合稿入口与派发技能 0 命中」）与二义几处；判决第 4、6、7 条（整批重交、冻结层指针、射程）不在那一段里，支撑只有腿报告与修稿去向表（修稿的工具员写的）。去向表是修稿一方自己的说法，判决里只当线索。这三条我补读了原件、第一版、第二版与脚本文件头（见上面各条），都成立，但是辩方读的，不能代替主 agent 的现查。
3. 腿报告里的两个总数没人复核：正推腿写「核过并确认有落点：337 条」（`research/prompts/rules-compress-entry-r1-sonnet-output.md:508`），粒度是它自己切的，并写明「没有保证每个逗号分句都单列一行」（`research/prompts/rules-compress-entry-r1-sonnet-output.md:509`）；攻方腿写「CLAUDE.md 只对了 diff 出的 117 行差异」（`research/prompts/rules-compress-entry-r1-opus-output.md:120`）。判决没有写这两个范围的缺口，也没有点名「格四：子 agent 读得到」两条腿各自只抽了一次样。我这一轮也没有复核这 337 条（我的分工是复核判决，不是重做落点表）。
4. 不采纳各行的复核：判决写主 agent「读了其中不采纳的 14 行」，没有记哪几行对着脚本文件头读过；入口题的三行（20、21、22 行）我都现读了文件头与技能行（见上），理由成立（22 行保留一句）。

## 辩方顺带看到的（不在判决里，也不是对判决的反驳）

- 同一形状没处理：`CLAUDE.md` 读一次清单（见第 2 条）；原件与第二版都有，压缩前就有，不是压缩带来的。
- 第二版新写的句子里有两处读得出两种：技能第 4 条括号里的「各在副本里改」（见上表最后一行）、入口第 9 行新增的半句（见第 3 条）。
- 抽读 `CLAUDE.md` 差异时看到的对得上的两处，供攻方腿省一遍：原件末半句「其余几类还没有格判、由主 agent 写判决时逐份点名」（`./CLAUDE.md:22`）在第二版落在实现流程规则：「这几类门禁还没判，由主 agent 写判决时逐份点名」（`research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:21`）；「不登豁免」落在入口：「不等用户点名、不登豁免」（`research/prompts/rules-compress-r2-merged/dot-claude/main-agent.md:27`）。
- 同类先例：第 2 条的读一次与压缩上下文、第 3 条的入口封闭写法与共用约束冲突、第 7 条的射程，都在仓里原件找到先例（已贴出处）；第 1、4、5、6 条没有找到别的项目或仓里的同类先例，不拿它们顶替论证。

## 要主 agent 跑的命令

无。没有写模型；cite-check 是直接跑的文本工具。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数（这一题没有模型数）没有要复跑的。
- 没读禁读清单里的文件（攻方腿 `rules-compress-entry-r2-opus-output.md` 与它的模型目录、草稿目录，别的四题第二轮各腿产出）。
- 没复核正推腿的 337 条落点表，没核格四，没核 `CLAUDE.md` 差异里除上面两处以外的各行（归攻方腿的格三）。
- 没建仓副本、编译目录；草稿目录里只有五个小文本文件（两份 cite-check 输出与三份截出来读的片段），没有进程，也没有要删的东西。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/rules-compress-entry-r2-sonnet-output.md` 末行（交回前跑的）：  ✓ 核了 93 处引文、0 处裸行号，对上 93 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
