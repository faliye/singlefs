#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 决策文档与变更史（二十一格：决策与分项的状态、索引与登记表对正文、未定项回看、写频率、候选代号复用、依据块里的推论判决与三方腿况、变更史条目与形状、欠段引已还清的欠账）
# gate-fixture: yes
# gate-category: 文档类
# gate-similar: history-ordinal.sh 它也读变更史的条目标题、也只判这一轮新增的，但判的是「（其 N）」撞号，住在上游副本里由上游门禁管，项目本地改不了它
# gate-similar: doc-registries.sh 它从欠账表、收口表与总审核出发判表本身的形状与账的去向，判字段与布局登记表（字节表「指向」列点名的分项存不存在、字段投影与加和、认购表求和不超预留），它的 decision-refs 那一格判 kb 全文引用的 D 号有定义、不看分项号与状态；这里的 user-verdict-owed 那一格方向相反，从决策正文里的「待用户复核」出发去欠账表找一笔未还的账，item-ref-status 那一格判引用写的已定 / 未定与正文分项表相符，判的对象都是决策正文、索引页、冻结层归属登记表与变更史；owed-refs-paid 那一格也读欠账表（与它同一份 lib-owed.py 的分界），对象是决策中文正文的欠段，与 user-verdict-owed 同一批对象、同一张表，所以放在这里；它的 table-shape、closeout-collects-open、paid-cited-tests 三格对象是欠账表自己与里程碑收口表，不读决策正文
# gate-similar: doc-process-records.sh 它从三方轮的判决、材料与快照出发，判一批工作留没留过程记录（判决点名、轮次收尾）；这里 inference-needs-verdict 与 three-way-leg-record 两格方向相反，从决策分项依据块里的「是推论」「三方」字样出发去找 research/ 下存在的判决或腿输出，对象是决策正文不是过程记录
# gate-similar: code-source-discipline.sh 它的 clause-enums 格也读 .claude/gate.d/ 下一张登记表、去决策分项里找逐字的串，但比对的另一边是代码里的枚举；这里格「同一结构的写频率只有一个量级」两边都在决策正文里（句子写的频率与登记的量级），不碰代码
#
# kb 里同一件事只许一种写法：决策与分项的状态在哪一处说都一样，索引页与决策相关的登记表与正文对得上；决策正文改了要在变更史留条目，变更史的形状要对。
# 格名表（--list 打它；各格的中文名写在格函数上方的「格「…」」那一行）：
# gate-cell: kb-shape kb 形状：分项只许一种叫法、kb 文件不链回自己、规则路径带 .claude/、决策标题与分项两节、索引页分项计数与进度计数与正文相符
# gate-cell: item-ref-status 每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符
# gate-cell: status-redundancy 状态别说两遍：小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行挂在对的节
# gate-cell: cross-decision-status 「D<n>（简称）」之后写「未定」（紧贴、隔着名词短语、「」引文里旁边没有现状），而那条决策的状态行不是待定
# gate-cell: settled-item-self-open 已定项区段里说它自己、或同一条决策里另一条已定分项还没定
# gate-cell: settled-ref-says-open 引用写着「已定项 k」、归属到的那一条也已定，紧跟着却说它没定
# gate-cell: stale-open-items 未定项点名的决策，状态行在这条未定项最近一次复核之后又变过
# gate-cell: settled-same-file 这次在一份决策里新增了「已定」小节（含「已定项 N：…」分项小节），同文件开着的未定项与索引页的待议节一个字没动
# gate-cell: freeze-layer-membership 冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项
# gate-cell: decision-items-sync decisions.md 的分项清单生成块与索引表「状态」「进度」两列，与生成器的输出逐字相同
# gate-cell: blocking-verdict 每条未定项都写了「改新池新建文件的字节」与「动不动格式」两把尺的判定
# gate-cell: user-verdict-owed 决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还的账点名那条决策
# gate-cell: decision-summary-width 决策索引表每一行的「结论」列不超过 decisions.md 写的字数上限
# gate-cell: write-frequency 决策正文里落在登记结构上的频率断言，不与 write-frequency.tsv 登记的量级说反话
# gate-cell: entry-added 决策正文改了，变更史里要有新增条目
# gate-cell: shape 变更史的形状：日期块住在自己的节、同节同一天只一块、块下是 - 变更项而不是 #### 子条目、不留快查与现状行
# gate-cell: history-loss 两份变更史与基准相比，基准里每个日期块的数、编号、产物名、引名在改后都找得到（提交前整理不许丢）
# gate-cell: candidate-codes 同一份决策里，同一个候选代号（甲乙丙丁… 或单个大写字母）不许出现在两个分项的候选表里
# gate-cell: inference-needs-verdict 决策分项依据块里自陈是推论的条目（是推论、是推的、只走了本地腿、没走三方），要指到一份存在的 research/prompts/*-main-verification.md
# gate-cell: three-way-leg-record 决策分项依据块里写了「三方」的条目，要指到 research/ 下存在的判决或腿输出，或写明腿况缺失且不写阴性结论
# gate-cell: owed-refs-paid 决策中文正文「**欠**：」行里引的 C 号（紧跟在别的 C 号后面的全角括注里的不算）已在 checks-owed.md 的已还清表里，而同一行里这个号每一处到下一处引账或分号的那一截连同它前面 12 个字都没写已还清、还清、销账；决策中文正文欠段之外与里程碑页中文正文的现状句里引的已还清的账，那一句写了欠、待、没做、记在 C<编号> 这类现欠说法而没写已还、还清、销账、已关、做完这类写法的也判红；滞后表 owed-refs-paid-lag.tsv 登记的放过，滞后表写对了、与 gate 基准那一版比只缩不涨
#
# 用法：
#   bash .claude/gate.d/doc-decisions.sh [项目根]                                       二十一格都跑；项目根不给取本脚本往上两级
#   bash .claude/gate.d/doc-decisions.sh --list                                         逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-decisions.sh --check <格名>[,<格名>…] [项目根]              只跑点名的格，按格名表的次序
#   bash .claude/gate.d/doc-decisions.sh --write [--check decision-items-sync] [项目根]  重写 decisions.md 的分项清单生成块与索引表「状态」「进度」两列，写完回读判一遍
# 退出码：0 绿；1 有格红（红格逐格接它登记的出路）；77 跑到的格全本次未跑（逐格报进 $GATE_NOT_RUN_FILE）；
#   2 参数错：认不出的选项、两个项目根、进不去项目根、格名写错、--write 配了 decision-items-sync 之外的格。
# --list、--check、样本根的 .gate-cells、每格一个子 shell 与逐格汇总都在共用库 lib/stage-cells.sh（写法在它的文件头）。
#
# 样本 fixtures/doc-decisions.sh/，每份样本根的 .gate-cells 点名只跑哪几格：
#   <格名>-red（前十四格各一份，kb-shape-red … write-frequency-red）  那一格判红、汇总有「<格名>：红（退出 1）」；各份犯了什么写在那一格函数上方
#   decision-documents-green                前十四格的绿样本并在一棵树里（setup.sh 造 git 历史与工作区改动）：十三格绿，write-frequency 没有登记表退 77，整道退 0
#   write-frequency-green                   write-frequency 的绿样本（不是 git 仓），那一格绿；判绿靠的几句写在那一格函数上方
#   decision-items-sync-translation-green   decisions/ 下除中文正文外还放着英语与日语译文（<名>.en.md、<名>.ja.md，首行不是 `## D<n> 简称 —— 状态`），不是 git 仓：
#                                           decision-items-sync 绿，生成器只读中文那一份、分项清单与索引页逐字相同；.gate-cells 只点名 decision-items-sync
#   candidate-codes-red / -green            candidate-codes 的样本（不是 git 仓）：红的那份两个分项的候选表都用丙（一张表格、一张列表）；
#                                           绿的那份两个分项代号不同，另一处写「已定项 1 的丙」，历史节里复用的代号不扫
#   decision-documents-some-cells-not-run   不是 git 仓，没有分项引用、D15、「待用户复核」与写频率登记表：七格退 77 逐格报本次未跑，整道退 0
#   decision-history-red                    决策首行状态翻了、已提交没推出去、变更史没新增条目，新 kb 页里日期与序号粘在一起：entry-added、shape 都红
#   shape-cells-marker-red                  现场同 decision-history-red，.gate-cells 只点名 shape：汇总只列一格
#   decision-history-green                  决策正文改了 7 行，D1 节新增一个带一条顶格 `- ` 项的日期块：entry-added、shape 都绿
#   shape-red                               shape 的 a 到 e 五样都有新增的，基准里另有存量只计数：shape 红
#   shape-green                             新增的日期块都住对节、同节日期不重复、围栏里的旧形态不算，存量只计数：shape 绿、entry-added 本次未跑
#   entry-added-moved-only-not-run          决策正文 6 行点名的 tools/old-rig 搬到 tools/new-rig、登记在 .claude/gate.d/path-moves.tsv，这一次只换了这 6 行的路径、变更史没新增条目：
#                                           只差登记的搬迁不算决策正文改了，entry-added 无对象可判、整道退 77
#   entry-added-moved-unregistered-red      现场同上、搬迁没登记：决策正文改了 12 行、没新增变更史条目，entry-added 红
#   entry-added-moved-and-edited-red        现场同 entry-added-moved-only-not-run（搬迁登记了），另把一行正文改了：决策正文改了 14 行，entry-added 红
#   settled-same-file-item-heading-red / -green  settled-same-file 的已定项分项标题样本：本次提交在未定项一节之前新增 `#### 已定项 2：丙问题`；
#                                           红的那份未定项一字未动、索引页待议节补了复核注，红在那条未定项上；绿的那份未定项与待议节都补了复核注，绿
#   settled-same-file-two-files-green       本次提交里两份决策都改了：D982 新增 `### 已定项 2 —— 已定` 并在它的未定项与索引页待议节补了复核注，D983 只改了历史版本下一句话、它的未定项没动：
#                                           新增已定小节的只算 D982，D983 不算，绿（报「新增已定小节的决策 1 份，查了 1 条未定项、1 节待议」）
#   stale-open-items-translation-red        D95 的未定项点名 D96，D96 的中文原文新增了已定小节，D96 另有一份译文 96-样本.zz.md 排在中文原文之后：只读中文原文，D95 那一条判红
#   settled-same-file-bare-settled-heading-not-run  本次提交只新增了光秃秃的 `### 已定（2026-10-13）：丙问题`（history-kept: 样本标题里的日期是数据）（不是首行登记标题的 `—— 已定`，也不是 `已定项 N：…`）：格不认它，退 77「本次 diff 没有新增「已定」小节」
#   history-loss-red / history-loss-green   基准里一块带一个数：整理时弄丢了判红；整理成一条结论、数都在判绿
#   history-loss-moved-commit-ref-green     tools/old-rig 搬到 tools/new-rig 并登记，变更史一块里「改前」行（history-kept: 变更史条目的字段名）留着 `git show 31e47115:tools/old-rig/run.sh`、「依据」行换成新路径：
#                                           换算基准那一版时提交引用后面的路径不换，与改后逐字相同，判绿
#   history-loss-moved-unregistered-red     现场同上、搬迁没登记：「依据」那一行在改后找不到，判红
#   history-loss-moved-and-edited-red       现场同 history-loss-moved-commit-ref-green（登记了），另删了「依据」行里的 1.7：判红在 1.7
#   history-loss-moved-registered-in-base-green  tools/old-rig 搬到 tools/new-rig，登记行先进了基准那一次提交、搬迁这一次才做（lib-lag-baseline.py 的 effective_among 不认这一行），
#                                           变更史一块里的产物行照 path-moves.md「怎么做」第 3 步换成新路径：history-loss 照基准之后才搬的登记行换算基准那一版再比，判绿
#   shape-moved-only-green                  存量节外日期块（上一级标题「## 杂项」）里一行点名的 tools/old-rig 搬到 tools/new-rig 并登记，那一行只换了路径：
#                                           只因搬迁才变了的行不算新增，那一块仍是存量，shape 绿、存量「条目住在节外 1 处」
#   shape-moved-unregistered-red            现场同上、搬迁没登记：那一行算新增，红在「1 个新写的日期块没住进自己的节」
#   shape-moved-and-edited-red              现场同 shape-moved-only-green（登记了），那一行除了换路径还补了半句：红在同一处
#   history-loss-base-unreadable-red        现场同 history-loss-green，基准那一版变更史的对象从对象库里删掉（ls-tree 认得出、show 读不出）：
#                                           判红在「读不出基准 HEAD 里的 .claude/kb/decisions-history.md」（取法 lib-lag-baseline.py show；取不到不跳过）
#   cross-decision-status-noun-phrase-red   D2502 写「D2501 的条带宽度可变粒度仍然未定」而 D2501 已定（隔着名词短语）；半定的 D2509 同样判红：cross-decision-status 红
#   cross-decision-status-quoted-red        「」引文里的过时写法（隔名词短语的与紧贴的各一处），引文旁边没有现状：红
#   cross-decision-status-relaxed-green     引文带现状（同一句、下一句、引用块之后的下一段）、「未定 0 项」「余下三项未定」、说成未定、当时未定、夹了 E12 的、待定的决策说未定：绿
#   cross-decision-status-ledger-green      隔名词短语的一句被挂账表罩住：绿
#   inference-needs-verdict-red / -green    红：自陈是推论而没有判决、判决文件不存在各一条，另一条带存在的判决不判；
#                                           绿：带判决、「不是推的」、带「用户」字样的「没走三方」、被挂账表罩住、记号在「」引文里
#   three-way-leg-record-red / -green       红：点名的材料不存在、没有指针也没写腿况、腿况缺失却写了阴性结论各一条；
#                                           绿：指到存在的材料、腿况缺失、否定的「没有走三方论证」、「」里引的阴性原话、「第三方」、被挂账表罩住
#   evidence-ledger-red                     挂账表写坏（列数、格名、片段太短）与罩不住（对象不红、文件不在），三格都红
#   evidence-ledger-grew-red                setup.sh 造 git 仓：挂账表相对基准提交多了一行：three-way-leg-record 红
#   evidence-ledger-base-unreadable-red     setup.sh 造 git 仓：挂账表与基准逐字相同，基准那一版挂账表的对象从对象库里删掉（ls-tree 认得出、show 读不出）：
#                                           三格都红在「读不出基准 HEAD 里的 .claude/gate.d/decision-evidence-lag.tsv（git show 退 128…），这张表只缩不涨没比」
#                                           （取法 lib-lag-baseline.py 的 baseline_text；取不到不当成基准里没有这张表）
#   evidence-pointer-history-green          setup.sh 造 git 仓：判决与材料先提交、后整批删掉（归档出了树，git 历史里取得回来）：inference-needs-verdict、three-way-leg-record 绿
#   evidence-pointer-renamed-history-green  现场同上，判决文件在被整批删掉之前改过一次名，决策点名的是改名前的名字（名字集合不依赖 git 的改名检测）：两格绿，报「只在 git 历史里取得回来的 4 个」
#   按编号取文件的三份（决策有中文原文 <号>-<名>.md 与译文 <号>-<名>.en.md / .ja.md，中文的登记表取中文原文，判法 lib-item-ref-status.py 的 numbered_files 与 heads_for_referrer）：
#   freeze-layer-membership-translation-green  D15、D18 各有两份译文，首行写成 `## D15 (…) — Settled` 读不出、里面也没有「#### 已定项 7 / 11」：
#                                           freeze-layer-membership 绿（只取中文原文，读不出首行的译文不拖垮这一格，报得出 COUNT）
#   freeze-layer-membership-translation-red    同一套文件，登记表一行依据点名 D18 已定项 5、中文原文没有：freeze-layer-membership 红、点名「D18 第 5 条分项」
#   write-frequency-translation-green       D1 有两份读不出首行的译文，登记表的权威分项在中文原文里：write-frequency 绿（读不出首行的译文不当场退 1）
#   数一遍、按语言分组的两份（判法 lib-item-ref-status.py 的 same_language_files 与 files_by_language）：
#   kb-shape-translation-green              D1、D2 各有中文原文与英语、日语译文，译文的分项小节是各自的语言（### Settled Items 这类），首行状态词写中文的（首行写成各自语言的另见 kb-shape-translation-first-line-green）：
#                                           kb-shape 绿，第 5 段报「中文原文 2 条、译文 .en.md 2 条、译文 .ja.md 2 条」，第 7 段报没对的译文份数
#   kb-shape-translation-red                同一套文件，中文索引表 D2 写「已定 2 项 / 未定 0 项」而中文原文是 1 / 1，日语译文 02 那一份首行写成 D3：
#                                           kb-shape 红，点名中文原文的 D2 计数与「决策编号不连续或不从 D1 起（译文 .ja.md）：[1, 3]」
#   item-ref-status 认变更史节标题与孤立译文的四份（判法 lib-item-ref-status.py 的 references、is_orphan_translation、orphan_translations）：
#   item-ref-status-history-section-green   变更史 `## D931（样本甲）` / `## D932（样本乙）` 两节，节里 `### <日期>` 的条目裸写「已定项 k / 未定项 k」（D932 节里一条先提到 D931），
#                                           都归节标题那条决策、状态相符：item-ref-status 绿
#   item-ref-status-history-section-red     同样的变更史，D932 节里裸写的「已定项 1」归到节标题 D932 之后状态不符（D932 的第 1 条是未定）：
#                                           item-ref-status 红、点名 D932
#   item-ref-status-orphan-translation-red  decisions/ 下有两份没有同名中文原文的译文（一份首行读得出、编号同 D941 的中文原文，一份首行读不出），中文引用方写的
#                                           「D941（样本甲） 已定项 2」按中文原文判：红在孤立译文这一类上，另有一处真不一致
#   item-ref-status-orphan-translation-green  每份译文都有同名的中文原文：不报孤立译文，绿
#   译文首行与中文写法不同的两份（决策表、第 5 段的标题与分项只读中文原文，判法 lib-item-ref-status.py 的 decision_heads 与 originals_only）：
#   item-ref-status-translation-first-line-green  D941 的英语译文首行用单破折号、日语译文用冒号没有破折号：item-ref-status 绿
#   kb-shape-translation-first-line-green   kb-shape-translation-green 的四份译文首行写成各自的语言（`— Settled`、`—— 確定` 这类）：kb-shape 绿，第 5 段报「译文 4 份只按语言判编号连号」，译文的标题不按中文的状态词判
#   欠段引已还清的欠账那一格（owed-refs-paid，判据写在这一格函数上方的注释），十二份都只跑 owed-refs-paid；带 setup.sh 的两份建 git 仓，其余不是 git 仓：
#   owed-refs-paid-red                      欠段引一个已还清的 C 什么都没说、一个夹在不紧跟 C 号的括注里（「C10（…）：…（归 C21（…））」）、一个「已还清」写在下一个 C 号之后，
#                                           三处逐处点名判红；同一行里开着的 C10、C11 与「## 历史版本」之后引的已还清不点名
#   owed-refs-paid-green                    欠段写「无。」、同一截写了「已还清」、「还清」写在前面 12 个字里、同一截写「销账」、历史版本节里引的、英语译文里引的：判绿，
#                                           报「扫 1 份中文正文、4 行欠段、5 个 C 号；其中引已还清的 3 个…」
#   owed-refs-paid-heading-missing-red      checks-owed.md 的「已还清」标题写成「已经还了」：整格红在「里认不出「已还清」标题」
#   owed-refs-paid-abbreviation-parenthesis-green  开着的账的登记简称里提到已还清的号（「C40（C30 作废之后…）」，同一行前面另有一个号；嵌套括号「C42（归 C31（…）与 C30 的继任）」）：
#                                           紧跟在 C 号后面那一对配平的全角括注里的 4 个号不算引账，判绿
#   owed-refs-paid-same-line-mark-green     一行里先列「C45（…）、C46（…）、C47（…）」、后补「其中 C46（…） 已还清」：C46 前一截没写还清也放过，判绿
#   owed-refs-paid-same-line-unmarked-red   同一句、后补那一截不写「已还清」：C46 一行两次都没说还清，点名一处判红
#   owed-refs-paid-lag-green                setup.sh 建 git 仓：基准里滞后表登记两处，这一轮改好一处、删了它那一行（只缩）；剩下那一处照登记放过：判绿、报「滞后 1 处」
#   owed-refs-paid-lag-growth-red           setup.sh 建 git 仓：这一轮新写一处引已还清的欠段、往滞后表加了罩它的一行：滞后表比基准多 1 行，判红
#   owed-refs-paid-lag-anchor-wrong-red     滞后表那一行的锚抄错一个字：锚命中 0 行判红，罩不住的那一处也判红
#   owed-refs-paid-lag-table-red            滞后表一行写对、另十行各错一处（列数、C 号写法、理由太短、锚太短、重复、路径不在、锚命中 0 行、锚命中两行、那一行里没有这个号、这个号还开着）：
#                                           逐行点名判红；写对的那一行照罩住判红的那一处
#   owed-refs-paid-status-sentence-red      现状句把已还清的号当现欠：决策欠段之外的两行（一句「…的检查记在 C20」、一行两句「这一格待 C21…补上；…已还清的另说」，还清写在别的句里不算）
#                                           与里程碑页两行（「C22 那一格还没做」「仍欠 C20 那一条变异」）四处逐处点名判红；欠段写「无。」、围栏里与「## 历史版本」之后引的、
#                                           里程碑译文里引的不判，报「扫 2 份中文正文、5 句引账、5 个 C 号」
#   owed-refs-paid-status-sentence-green    决策欠段之外：那一句写了「已还清」、写了「已销账」、只在开着的 C10 的简称括注里提到 C21 的各一行，「只欠…记在 C21」一行由滞后表收着；
#                                           写「已还」的历史引用（「当时欠的那一格记在 C23…，2026-10-01 已还，立成 D8」，history-kept: 样本引文里的日期是数据）、只提到不当现欠的（「检查照 C20 那一格落在门禁里」）、
#                                           现欠的词只在登记简称括注里的（「C24（等待复核时欠着的样本账）」）、只带「欠账表」「等待」与不跟 C 号的「记在」的各一行；
#                                           里程碑页「C22 已还清」一行、开着的 C10 一行、围栏里的一行不判：判绿，报「10 句引账、10 个 C 号；其中引已还清的 8 个：那一句写了…之一的 4 个，没写现欠说法的 3 个；滞后 1 处」
# 弄坏开关（样本自检时设进环境）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/doc-decisions.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库：红格按绿记、整道退 0
#       blocking-verdict-red、candidate-codes-red、cross-decision-status-red、decision-history-red、decision-items-sync-red、decision-summary-width-red、
#       freeze-layer-membership-red、history-loss-red、item-ref-status-red、kb-shape-red、settled-item-self-open-red、settled-ref-says-open-red、
#       settled-same-file-item-heading-red、settled-same-file-red、shape-cells-marker-red、shape-red、stale-open-items-red、status-redundancy-red、user-verdict-owed-red、
#       cross-decision-status-noun-phrase-red、cross-decision-status-quoted-red、evidence-ledger-red、evidence-ledger-grew-red、evidence-ledger-base-unreadable-red、
#       inference-needs-verdict-red、three-way-leg-record-red、write-frequency-red、freeze-layer-membership-translation-red、kb-shape-translation-red、
#       item-ref-status-history-section-red、item-ref-status-orphan-translation-red、history-loss-base-unreadable-red（期望退出 1，实测 0）
#       entry-added-moved-unregistered-red、entry-added-moved-and-edited-red、history-loss-moved-unregistered-red、history-loss-moved-and-edited-red、shape-moved-unregistered-red、shape-moved-and-edited-red（期望退出 1，实测 0）
#       owed-refs-paid-red、owed-refs-paid-heading-missing-red、owed-refs-paid-same-line-unmarked-red、owed-refs-paid-lag-growth-red、owed-refs-paid-lag-anchor-wrong-red、
#       owed-refs-paid-lag-table-red、owed-refs-paid-status-sentence-red、stale-open-items-translation-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本把登记的格都跑
#       settled-same-file-bare-settled-heading-not-run（期望退出 77，实测不是 77）
#       blocking-verdict-red、candidate-codes-red、cross-decision-status-red、decision-items-sync-red、decision-summary-width-red、freeze-layer-membership-red、
#       history-loss-red、item-ref-status-red、kb-shape-red、settled-item-self-open-red、settled-ref-says-open-red、settled-same-file-item-heading-red、settled-same-file-red、shape-red、
#       stale-open-items-red、status-redundancy-red、user-verdict-owed-red、write-frequency-red、
#       cross-decision-status-noun-phrase-red、cross-decision-status-quoted-red、evidence-ledger-grew-red、inference-needs-verdict-red、three-way-leg-record-red、history-loss-base-unreadable-red、stale-open-items-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       evidence-ledger-red、evidence-ledger-base-unreadable-red（输出里找不到「doc-decisions.sh：3 格里 3 格红…」）；
#       candidate-codes-green、decision-documents-green、decision-history-green、history-loss-green、settled-same-file-item-heading-green、shape-green（期望退出 0，实测 1）；
#       decision-documents-some-cells-not-run（输出里找不到「doc-decisions.sh：14 格里 7 格绿…」）；decision-history-red（输出里找不到「doc-decisions.sh：2 格里 2 格红…」）；
#       shape-cells-marker-red（输出里找不到「样本只跑 .gate-cells 点名的格：shape」）；write-frequency-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）；
#       freeze-layer-membership-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       freeze-layer-membership-translation-green、write-frequency-translation-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）；
#       kb-shape-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；kb-shape-translation-green、kb-shape-translation-first-line-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）；
#       item-ref-status-history-section-red、item-ref-status-orphan-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       item-ref-status-history-section-green、item-ref-status-orphan-translation-green、item-ref-status-translation-first-line-green（期望退出 0，实测 1）
#       entry-added-moved-unregistered-red、entry-added-moved-and-edited-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；entry-added-moved-only-not-run（期望退出 77，实测 1）
#       history-loss-moved-unregistered-red、history-loss-moved-and-edited-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；history-loss-moved-commit-ref-green、history-loss-moved-registered-in-base-green（期望退出 0，实测 1）
#       shape-moved-unregistered-red、shape-moved-and-edited-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；shape-moved-only-green（期望退出 0，实测 1）
#       cross-decision-status-ledger-green、cross-decision-status-relaxed-green、decision-items-sync-translation-green、evidence-pointer-history-green、
#       evidence-pointer-renamed-history-green、inference-needs-verdict-green、three-way-leg-record-green、settled-same-file-two-files-green（期望退出 0，实测 1）
#       owed-refs-paid-red、owed-refs-paid-heading-missing-red、owed-refs-paid-same-line-unmarked-red、owed-refs-paid-lag-growth-red、owed-refs-paid-lag-anchor-wrong-red、
#       owed-refs-paid-lag-table-red（输出里找不到「样本只跑 .gate-cells 点名的格：owed-refs-paid」）；
#       owed-refs-paid-green、owed-refs-paid-abbreviation-parenthesis-green、owed-refs-paid-same-line-mark-green、owed-refs-paid-lag-green、owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#       owed-refs-paid-status-sentence-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）
#   STAGE_CELLS_BREAK=check-ignored  归共用库：点名选格不起作用（--check 与样本根的 .gate-cells 共用一段），照跑全部登记的格
#       settled-same-file-bare-settled-heading-not-run（期望退出 77，实测不是 77）
#       blocking-verdict-red、candidate-codes-red、cross-decision-status-red、decision-items-sync-red、decision-summary-width-red、freeze-layer-membership-red、
#       history-loss-red、item-ref-status-red、kb-shape-red、settled-item-self-open-red、settled-ref-says-open-red、settled-same-file-item-heading-red、settled-same-file-red、shape-red、
#       stale-open-items-red、status-redundancy-red、user-verdict-owed-red、write-frequency-red、
#       cross-decision-status-noun-phrase-red、cross-decision-status-quoted-red、evidence-ledger-grew-red、inference-needs-verdict-red、three-way-leg-record-red、history-loss-base-unreadable-red、stale-open-items-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       evidence-ledger-red、evidence-ledger-base-unreadable-red（输出里找不到「doc-decisions.sh：3 格里 3 格红…」）；
#       candidate-codes-green、decision-documents-green、decision-history-green、history-loss-green、settled-same-file-item-heading-green、shape-green（期望退出 0，实测 1）；
#       decision-documents-some-cells-not-run（输出里找不到「doc-decisions.sh：14 格里 7 格绿…」）；decision-history-red（输出里找不到「doc-decisions.sh：2 格里 2 格红…」）；
#       shape-cells-marker-red（输出里找不到「样本只跑 .gate-cells 点名的格：shape」）；write-frequency-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）；
#       freeze-layer-membership-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       freeze-layer-membership-translation-green、write-frequency-translation-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）；
#       kb-shape-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；kb-shape-translation-green、kb-shape-translation-first-line-green（输出里找不到「doc-decisions.sh：1 格里 1 格绿…」）；
#       item-ref-status-history-section-red、item-ref-status-orphan-translation-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；
#       item-ref-status-history-section-green、item-ref-status-orphan-translation-green、item-ref-status-translation-first-line-green（期望退出 0，实测 1）
#       entry-added-moved-unregistered-red、entry-added-moved-and-edited-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；entry-added-moved-only-not-run（期望退出 77，实测 1）
#       history-loss-moved-unregistered-red、history-loss-moved-and-edited-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；history-loss-moved-commit-ref-green、history-loss-moved-registered-in-base-green（期望退出 0，实测 1）
#       shape-moved-unregistered-red、shape-moved-and-edited-red（输出里找不到「doc-decisions.sh：1 格里 1 格红…」）；shape-moved-only-green（期望退出 0，实测 1）
#       cross-decision-status-ledger-green、cross-decision-status-relaxed-green、decision-items-sync-translation-green、evidence-pointer-history-green、
#       evidence-pointer-renamed-history-green、inference-needs-verdict-green、three-way-leg-record-green、settled-same-file-two-files-green（期望退出 0，实测 1）
#       owed-refs-paid-red、owed-refs-paid-heading-missing-red、owed-refs-paid-same-line-unmarked-red、owed-refs-paid-lag-growth-red、owed-refs-paid-lag-anchor-wrong-red、
#       owed-refs-paid-lag-table-red、owed-refs-paid-status-sentence-red（输出里找不到「样本只跑 .gate-cells 点名的格：owed-refs-paid」）；
#       owed-refs-paid-green、owed-refs-paid-abbreviation-parenthesis-green、owed-refs-paid-same-line-mark-green、owed-refs-paid-lag-green、owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   KB_NUMBERED_FILE_BREAK=first-match  归 lib-item-ref-status.py（设进环境，被各格起的 python 继承）：按编号取文件不分语言、取排序后的第一个命中（取到 <号>-<名>.en.md），
#                                       heads_for_referrer 不筛语言（读不出首行的译文也算这一格的对象）；same_language_files 不筛语言、files_by_language 并成一组
#                                       （kb-shape 第 7 段拿中文索引去对译文，第 5 段把三语的号并成一个列表判连号）
#       freeze-layer-membership-translation-green、write-frequency-translation-green、kb-shape-translation-green、kb-shape-translation-first-line-green、
#       item-ref-status-translation-first-line-green（期望退出 0，实测 1；后一份是 decision_heads 在这个开关下也连译文一起读）；
#       kb-shape-translation-red（输出里找不到「决策编号不连续或不从 D1 起（译文 .ja.md）：[1, 3]」）；
#       item-ref-status-orphan-translation-red（输出里找不到「decisions/ 下有 2 份孤立译文」：这个开关下不区分孤立与否，孤立译文又进决策表）
#   KB_ORPHAN_TRANSLATION_BREAK=substitute  归 lib-item-ref-status.py（设进环境，被各格起的 python 继承）：孤立译文（带语言后缀、同目录没有同名中文原文）又进决策表、
#                                       顶替同号的中文原文，也不单独报一类
#       item-ref-status-orphan-translation-red（输出里找不到「decisions/ 下有 2 份孤立译文」）
#   KB_HISTORY_SECTION_OWNER_BREAK=nearest-d  归 lib-item-ref-status.py：不认变更史的 `## D<n>（简称）` 节标题，每个 `### ` 标题都把标题上的归属清掉，
#                                       节里裸写的分项退回「往前找最近提到的 D 号」
#       item-ref-status-history-section-green（期望退出 0，实测 1）；item-ref-status-history-section-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=kb-shape  kb-shape 格不跑、按判过了记
#       decision-documents-green（输出里找不到「决策标题与未定项列表相符，且两节没有互相串味…」）；kb-shape-red（期望退出 1，实测 0）；
#       kb-shape-translation-green、kb-shape-translation-first-line-green（输出里找不到「决策标题与未定项列表相符，且两节没有互相串味（2 条决策…」）；kb-shape-translation-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=item-ref-status  item-ref-status 格不跑、按判过了记
#       decision-documents-green（输出里找不到「分项引用与正文状态一致…」）；decision-documents-some-cells-not-run（输出里找不到「  item-ref-status：本次未跑」）；
#       item-ref-status-red、item-ref-status-history-section-red、item-ref-status-orphan-translation-red（期望退出 1，实测 0）；
#       item-ref-status-history-section-green、item-ref-status-orphan-translation-green、item-ref-status-translation-first-line-green（输出里找不到「分项引用与正文状态一致…」）
#   DOC_DECISIONS_BREAK=read-translations  kb-shape 第 5 段的标题状态、分项两节、未定条数连译文一起判（编号连号照旧按语言各判各的）
#       kb-shape-translation-first-line-green（期望退出 0，实测 1）；
#       kb-shape-translation-green（输出里找不到「决策标题与未定项列表相符，且两节没有互相串味（2 条决策、核 3 行分项索引；译文 4 份只按语言判编号连号）…」：译文也进了判，成功句不再报只按语言判的译文份数）
#   KB_DECISION_HEADS_BREAK=read-translations  归 lib-item-ref-status.py（设进环境，被各格起的 python 继承）：决策表连译文一起读，译文首行读不出就报红
#       item-ref-status-translation-first-line-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=status-redundancy  status-redundancy 格不跑、按判过了记
#       decision-documents-green（输出里找不到「状态只说一遍，且分项都在对的节里…」）；status-redundancy-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=cross-decision-status  cross-decision-status 格不跑、按判过了记
#       cross-decision-status-red、cross-decision-status-noun-phrase-red、cross-decision-status-quoted-red（期望退出 1，实测 0）；
#       decision-documents-green（输出里找不到「没有把已定的决策说成未定…」）；
#       cross-decision-status-relaxed-green（输出里找不到「没有把已定的决策说成未定（扫 3 份 kb 文件，比对 3 条决策的状态行）」）；
#       cross-decision-status-ledger-green（输出里找不到「没有把已定的决策说成未定（扫 2 份 kb 文件，比对 2 条决策的状态行）」）；
#       evidence-ledger-red（输出里找不到「挂账表 .claude/gate.d/decision-evidence-lag.tsv 有 4 处不对」）；
#       evidence-ledger-base-unreadable-red（输出里找不到「  cross-decision-status：红（退出 1）」）
#   DOC_DECISIONS_BREAK=settled-item-self-open  settled-item-self-open 格不跑、按判过了记
#       decision-documents-green（输出里找不到「已定项的正文没有把已经定了的东西说成未定…」）；settled-item-self-open-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=settled-ref-says-open  settled-ref-says-open 格不跑、按判过了记
#       decision-documents-green（输出里找不到「扫了 29 个文件、44 处已定项引用，没有一处紧跟着说它没…」）；decision-documents-some-cells-not-run（输出里找不到「  settled-ref-says-open：本次未跑」）；
#       settled-ref-says-open-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=stale-open-items  stale-open-items 格不跑、按判过了记
#       decision-documents-green（输出里找不到「没有未定项被别处的更新甩在后面…」）；decision-documents-some-cells-not-run（输出里找不到「  stale-open-items：本次未跑」）；
#       stale-open-items-red、stale-open-items-translation-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=stale-open-items-translations-read  stale-open-items 两遍都连译文一起读：译文与中文原文同一个决策号，后读到的译文把那条决策的「—— 已定」小节时间盖成 0，点名它的未定项不比
#       stale-open-items-translation-red（期望退出 1，实测 77：没有一条点名定过东西的别的决策）
#   DOC_DECISIONS_BREAK=settled-same-file  settled-same-file 格不跑、按判过了记
#       decision-documents-green（输出里找不到「同文件的未定项与索引页的待议节都被回头看过…」）；decision-documents-some-cells-not-run（输出里找不到「  settled-same-file：本次未跑」）；
#       settled-same-file-item-heading-green、settled-same-file-two-files-green（输出里找不到「同文件的未定项与索引页的待议节都被回头看过…」）；
#       settled-same-file-item-heading-red、settled-same-file-red（期望退出 1，实测 0）
#       settled-same-file-bare-settled-heading-not-run（期望退出 77，实测不是 77）
#   DOC_DECISIONS_BREAK=settled-same-file-added-lines-pooled  settled-same-file 把几份改过的决策的新增行并成一堆看：有一份新增了已定小节，每一份改过的决策都算新增了
#       settled-same-file-two-files-green（期望退出 0，实测 1：D983 那条没动的未定项判红）
#   DOC_DECISIONS_BREAK=settled-item-heading-unrecognised  settled-same-file 退回只认 `—— 已定` 的小节标题正则，不认 `#### 已定项 N：…` 与 `#### 已定项 N（…）`
#       settled-same-file-item-heading-red（期望退出 1，实测 77：本次 diff 没有新增「已定」小节）；settled-same-file-item-heading-green（期望退出 0，实测 77）；decision-documents-green（输出里找不到「同文件的未定项与索引页的待议节都被回头看过…」与「14 格里 13 格绿…」）
#   DOC_DECISIONS_BREAK=freeze-layer-membership  freeze-layer-membership 格不跑、按判过了记
#       decision-documents-green（输出里找不到「冻结层归属登记表判过了…」）；decision-documents-some-cells-not-run（输出里找不到「  freeze-layer-membership：本次未跑」）；
#       freeze-layer-membership-red、freeze-layer-membership-translation-red（期望退出 1，实测 0）；freeze-layer-membership-translation-green（输出里找不到「冻结层归属登记表判过了…」）
#   DOC_DECISIONS_BREAK=decision-items-sync  decision-items-sync 格不跑、按判过了记
#       decision-documents-green（输出里找不到「决策索引表状态列与进度列与正文同步（21 条决策）」）；decision-items-sync-red（期望退出 1，实测 0）；
#       decision-items-sync-translation-green（输出里找不到「决策索引表状态列与进度列与正文同步（1 条决策）」）
#   GEN_DECISION_ITEMS_BREAK=read-translations  归生成器 .claude/scripts/gen-decision-items.py（设进环境，被 decision-items-sync 格起的生成器继承）：译文也当正文读
#       decision-items-sync-translation-green（期望退出 0，实测 1：「✗ 生成器跑不起来」下接「首行读不出」）
#   DOC_DECISIONS_BREAK=blocking-verdict  blocking-verdict 格不跑、按判过了记
#       blocking-verdict-red（期望退出 1，实测 0）；decision-documents-green（输出里找不到「13 条未定项两把尺都判过」）
#   DOC_DECISIONS_BREAK=user-verdict-owed  user-verdict-owed 格不跑、按判过了记
#       decision-documents-green（输出里找不到「待用户复核的条款都有未还的账盯着」）；decision-documents-some-cells-not-run（输出里找不到「  user-verdict-owed：本次未跑」）；
#       user-verdict-owed-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=decision-summary-width  decision-summary-width 格不跑、按判过了记
#       decision-documents-green（输出里找不到「决策索引结论列都不超 200 字（共 21 行）」）；decision-summary-width-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=write-frequency  write-frequency 格不跑、按判过了记
#       decision-documents-green、decision-documents-some-cells-not-run（输出里找不到「  write-frequency：本次未跑」）；write-frequency-green（输出里找不到「同一结构的写频率只有一个量级…」）；
#       write-frequency-red（期望退出 1，实测 0）；write-frequency-translation-green（输出里找不到「同一结构的写频率只有一个量级…」）
#   DOC_DECISIONS_BREAK=entry-added  entry-added 格不跑、按判过了记
#       decision-history-green（输出里找不到「决策正文改了 7 行，变更史新增 1 条条目」）；decision-history-red（输出里找不到「决策正文改了 2 行（碰了 2 行状态行，不按小改动放行），…」）；
#       shape-green（输出里找不到「doc-decisions.sh：2 格里 1 格绿…」）；entry-added-moved-unregistered-red、entry-added-moved-and-edited-red（期望退出 1，实测 0）；
#       entry-added-moved-only-not-run（期望退出 77，实测 0）
#   DOC_DECISIONS_BREAK=shape  shape 格不跑、按判过了记
#       decision-history-green（输出里找不到「查了 3 份 kb 文件、1 个 `### 日期` 标题…」）；decision-history-red（输出里找不到「.claude/kb/invariants.md:5  ##…」）；
#       shape-cells-marker-red、shape-red（期望退出 1，实测 0）；shape-green（输出里找不到「变更史的形状对：查了 7 份 kb 文件、11 个 `###…」）；
#       shape-moved-unregistered-red、shape-moved-and-edited-red（期望退出 1，实测 0）；shape-moved-only-green（输出里找不到「变更史的形状对：查了 1 份 kb 文件、2 个 `###…」）
#   DOC_DECISIONS_BREAK=history-loss  history-loss 格不跑、按判过了记
#       history-loss-green（输出里找不到「变更史整理没丢：查了 1 份，基准里每个日期块的数、编号、产…」）；history-loss-red、history-loss-base-unreadable-red（期望退出 1，实测 0）；
#       history-loss-moved-commit-ref-green、history-loss-moved-registered-in-base-green（输出里找不到「变更史整理没丢：查了 1 份，基准里每个日期块的数、编号、产…」）；history-loss-moved-unregistered-red、history-loss-moved-and-edited-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=history-loss-registered-moves-ignored  history-loss 不交 --moves-base（只照 lib-lag-baseline.py 生效的登记行换算，登记先进基准、搬迁后做的不换）
#       history-loss-moved-registered-in-base-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=write-frequency-levels-merged  write-frequency 不同级一律判同级
#       write-frequency-green（期望退出 0，实测 1）；write-frequency-red（输出里找不到「01-写频率样本甲.md:11「系统配置槽」写成「每次挂载」…」）
#   DOC_DECISIONS_BREAK=write-frequency-negation-asserts  write-frequency 否定形态也算断言
#       write-frequency-green（期望退出 0，实测 1）；write-frequency-red（输出里找不到「01-写频率样本甲.md:11「tail」否定了「每次发布」…」）
#   DOC_DECISIONS_BREAK=write-frequency-subject-in-sentence-only  write-frequency 叫法只从句内认
#       write-frequency-green（输出里找不到「同一结构的写频率只有一个量级…」）；write-frequency-red（输出里找不到「01-写频率样本甲.md:23「tail」（叫法在所属标题）…」）
#   DOC_DECISIONS_BREAK=write-frequency-negation-adjacent-only  write-frequency 否定词只认紧挨频率词的
#       write-frequency-green（期望退出 0，实测 1）；write-frequency-red（输出里找不到「01-写频率样本甲.md:12「系统配置槽」否定了「每次发布…」）
#   DOC_DECISIONS_BREAK=write-frequency-restriction-as-plain  write-frequency「只在 / 仅」当普通断言
#       write-frequency-green（输出里找不到「同一结构的写频率只有一个量级…」）；write-frequency-red（输出里找不到「01-写频率样本甲.md:15「系统配置槽」用「只在 / 仅…」）
#   DOC_DECISIONS_BREAK=stock-judged  shape 把存量也当新增判
#       shape-green、shape-moved-only-green（期望退出 0，实测 1）；shape-red（输出里找不到「2 处新写的标题把日期与「（其N）」粘在同一个 `### `…」）
#   DOC_DECISIONS_BREAK=fused-ignored  shape 不判 a（日期与序号粘在一起）
#       decision-history-red（输出里找不到「.claude/kb/invariants.md:5  ##…」）；shape-cells-marker-red（期望退出 1，实测 0）；
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「2 处新写的标题把日期与「（其N）」粘在同一个 `### `…」）
#   DOC_DECISIONS_BREAK=no-home-ignored  shape 不判 b（日期块没住进自己的节）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「4 个新写的日期块没住进自己的节」）；
#       shape-moved-only-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-moved-unregistered-red、shape-moved-and-edited-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=duplicate-date-ignored  shape 不判 c（同节同一个日期开几块）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「2 组同一节里同一个日期开了几个 `### ` 块」）
#   DOC_DECISIONS_BREAK=wrapper-ignored  shape 不判 d（旧形态包装）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「3 行新写进变更史的旧形态包装…」）
#   DOC_DECISIONS_BREAK=empty-ignored  shape 不判 e（日期块下没有变更项）
#       shape-green（输出里找不到「存量（基准里就有，等搬迁，只计数不判红）：日期与序号粘在一起…」）；shape-red（输出里找不到「1 个新开的 `### 日期` 块下一条 `- ` 变更项都…」）
#   DOC_DECISIONS_BREAK=candidate-codes-unchecked  candidate-codes 复用的候选代号不判红，汇总照样记绿
#       candidate-codes-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=inference-needs-verdict  inference-needs-verdict 格不跑、按判过了记
#       inference-needs-verdict-red（期望退出 1，实测 0）；inference-needs-verdict-green（输出里找不到「自陈是推论的依据条目都指到了存在的判决：扫 1 份决策、5 条依据条目，带推论记号的 2 条…」）；
#       evidence-ledger-red（输出里找不到「挂账表 .claude/gate.d/decision-evidence-lag.tsv 有 5 处不对」）；
#       evidence-pointer-history-green、evidence-pointer-renamed-history-green（输出里找不到「自陈是推论的依据条目都指到了存在的判决：扫 1 份决策、2 条依据条目，带推论记号的 1 条…」）；
#       evidence-ledger-base-unreadable-red（输出里找不到「  inference-needs-verdict：红（退出 1）」）
#   DOC_DECISIONS_BREAK=three-way-leg-record  three-way-leg-record 格不跑、按判过了记
#       three-way-leg-record-red、evidence-ledger-grew-red（期望退出 1，实测 0）；three-way-leg-record-green（输出里找不到「依据里写了「三方」的条目都有腿况记录：扫 1 份决策、6 条依据条目，写了「三方」的 4 条…」）；
#       evidence-ledger-red（输出里找不到「挂账表 .claude/gate.d/decision-evidence-lag.tsv 有 3 处不对」）；
#       evidence-pointer-history-green、evidence-pointer-renamed-history-green（输出里找不到「依据里写了「三方」的条目都有腿况记录：扫 1 份决策、2 条依据条目，写了「三方」的 1 条…」）；
#       evidence-ledger-base-unreadable-red（输出里找不到「  three-way-leg-record：红（退出 1）」）
#   DOC_DECISIONS_BREAK=cross-decision-status-tight-only  cross-decision-status 只认紧贴的写法
#       cross-decision-status-noun-phrase-red（期望退出 1，实测 0）；cross-decision-status-ledger-green（期望退出 0，实测 1：挂账行罩的那一句不红了）；
#       cross-decision-status-quoted-red（输出里找不到「.claude/kb/decisions/2504-引文样本引用方.md:3 说 D2503 未定（隔着名词短语，在「」引文里…」）；
#       cross-decision-status-relaxed-green（输出里找不到「隔着名词短语的写法滤掉的：计数写法 3 处；…」）
#   DOC_DECISIONS_BREAK=cross-decision-status-quote-skipped  「」引文里的写法一概放过
#       cross-decision-status-quoted-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=cross-decision-status-quote-current-ignored  「」引文旁边有没有现状都判红
#       cross-decision-status-relaxed-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=cross-decision-status-count-form-judged  「已定 3 项 / 未定 0 项」「余下三项未定」这类计数写法也判
#       cross-decision-status-relaxed-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=cross-decision-status-history-phrases-judged  「说成未定」「（当时未定）」也判
#       cross-decision-status-relaxed-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=cross-decision-status-other-number-ignored  决策号与「未定」之间夹了别的编号也判
#       cross-decision-status-relaxed-green、decision-documents-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=evidence-pointer-unchecked  点名了判决 / 材料就算指到，不查存在（inference-needs-verdict、three-way-leg-record 共用）
#       inference-needs-verdict-red（输出里找不到「自陈是推论的依据条目没有指到存在的判决：2 条」）；three-way-leg-record-red（输出里找不到「依据里写了「三方」而没有腿况记录：3 条」）
#   DOC_DECISIONS_BREAK=evidence-history-ignored  判决 / 材料只认树里的，不读 git 历史（两格共用）
#       evidence-pointer-history-green（期望退出 0，实测 1）；evidence-pointer-renamed-history-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=evidence-ledger-unjudged  挂账表的行不核罩没罩住、写得对不对（三格共用）
#       evidence-ledger-red（期望退出 1，实测 0）
#   DOC_DECISIONS_BREAK=evidence-ledger-growth-unjudged  挂账表相对基准涨了不判、基准那一版也不取（三格共用）
#       evidence-ledger-grew-red、evidence-ledger-base-unreadable-red（期望退出 1，实测 0）
#   LAG_BASELINE_BREAK=unreadable-as-absent  归 lib-lag-baseline.py（设进环境，被三格起的 python 与 history-loss 起的 show 继承）：git 取不到基准那一版当成「基准里没有」，不比
#       evidence-ledger-base-unreadable-red（期望退出 1，实测 0）；history-loss-base-unreadable-red（期望退出 1，实测 77）
#   LAG_BASELINE_BREAK=commit-ref-rewritten  归 lib-lag-baseline.py（设进环境，被 history-loss 起的 show 继承）：基准那一版换算时连「<提交哈希>:」后面的路径也换
#       history-loss-moved-commit-ref-green（期望退出 0，实测 1）
#   LAG_BASELINE_BREAK=moved-only-kept  归 lib-lag-baseline.py（设进环境，被 entry-added 经 changed-paths.sh 起的 moved-only-filter 继承）：与基准只差登记搬迁的决策文件照旧算改了
#       entry-added-moved-only-not-run（期望退出 77，实测 1）；shape 的新增行同样不剔（经 lib-lag-baseline.py 的 drop_moved_lines）：shape-moved-only-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=evidence-ledger-ignored  挂账表当空表读（三格共用）
#       cross-decision-status-ledger-green、inference-needs-verdict-green、three-way-leg-record-green（期望退出 0，实测 1）；evidence-ledger-red（期望退出 1，实测 0）；
#       evidence-ledger-grew-red（输出里找不到「挂账表 .claude/gate.d/decision-evidence-lag.tsv 有 1 处不对」）
#   DOC_DECISIONS_BREAK=inference-user-verdict-judged  带「用户」字样的「没走三方」也算自陈的推论
#       inference-needs-verdict-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=inference-negation-ignored  「不是推的」也算推论记号
#       inference-needs-verdict-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=three-way-leg-negative-allowed  腿况缺失的条目写了阴性结论也放过
#       three-way-leg-record-red（输出里找不到「依据里写了「三方」而没有腿况记录：3 条」）
#   DOC_DECISIONS_BREAK=three-way-leg-quotes-judged  「」引文里的阴性写法也算这一条写了阴性结论
#       three-way-leg-record-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=three-way-leg-negated-judged  「没有走三方论证」也算在说跑过三方
#       three-way-leg-record-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-table-unread  owed-refs-paid 不读已还清表（当它是空的），欠段引的账全放行，滞后表登记的号也都算「已不在已还清表里」
#       owed-refs-paid-red、owed-refs-paid-same-line-unmarked-red（期望退出 1，实测 0）；owed-refs-paid-lag-green（期望退出 0，实测 1）；
#       owed-refs-paid-green、owed-refs-paid-same-line-mark-green（输出里找不到「✓ 决策正文的欠段没有把已还清的账当开着的（扫 1 份中文正文、…」）；
#       owed-refs-paid-lag-anchor-wrong-red（输出里找不到「✗ .claude/kb/decisions/01-样本决策.md:7 C20（样本已还的甲）」）；
#       owed-refs-paid-lag-table-red（输出里找不到「滞后表 .claude/gate.d/owed-refs-paid-lag.tsv 有 10 处不对」）；
#       owed-refs-paid-status-sentence-red（期望退出 1，实测 0）；owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-abbreviation-parenthesis-counted  owed-refs-paid 紧跟在 C 号后面的全角括注里的 C 号照样算引账
#       owed-refs-paid-abbreviation-parenthesis-green、owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-same-line-other-mark-ignored  owed-refs-paid 每一处只看自己那一截，同一行里同一个号另一截写了已还清不算
#       owed-refs-paid-same-line-mark-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-lag-unread  owed-refs-paid 不读滞后表（判红的处一律红，表写没写对、涨没涨都不判）
#       owed-refs-paid-lag-green（期望退出 0，实测 1）；
#       owed-refs-paid-green、owed-refs-paid-abbreviation-parenthesis-green（输出里找不到「别的 C 号简称括注里的 C 号 0 个不算引账；滞后表 0 行（没有滞后表）」，后一份是 4 个）；
#       owed-refs-paid-lag-anchor-wrong-red、owed-refs-paid-lag-table-red（输出里找不到「滞后表 .claude/gate.d/owed-refs-paid-lag.tsv 有 …处不对」）；
#       owed-refs-paid-lag-growth-red（输出里找不到「滞后表比 gate 基准 HEAD 那一版多 1 行…」）；
#       owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-lag-growth-unjudged  owed-refs-paid 滞后表不与 gate 基准那一版比（滞后表涨了也不红）
#       owed-refs-paid-lag-growth-red（期望退出 1，实测 0）；
#       owed-refs-paid-lag-green（输出里找不到「别的 C 号简称括注里的 C 号 0 个不算引账；滞后表 1 行（与 gate 基准 HEAD 那一版比没涨…」）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-sentences-skipped  owed-refs-paid 现状句不判（退回只判欠段）
#       owed-refs-paid-status-sentence-red（期望退出 1，实测 0）；
#       owed-refs-paid-status-sentence-green（输出里找不到「现状句：扫 2 份中文正文（决策 1 份的欠段之外、里程碑页 1 份），10 句引账…」）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-marks-ignored  owed-refs-paid 现状句里写了已还、还清这些写明已还的写法也照红
#       owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-whole-line  owed-refs-paid 现状句整行算一句（同一行别的句里写了还清也算写了）
#       owed-refs-paid-status-sentence-red（输出里找不到「✗ .claude/kb/decisions/01-样本决策.md:6 C21（样本已还的乙）」）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-owed-words-ignored  owed-refs-paid 现状句不看现欠说法，引已还清的账、那一句没写已还就红
#       owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-old-marks  owed-refs-paid 现状句只认三个词（已还清、还清、销账）算写明已还，「已还」「做完」这类不认
#       owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-claims-in-abbreviation  owed-refs-paid 现状句登记简称括注里的「欠」「待」也算现欠说法
#       owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-not-claims-counted  owed-refs-paid 现状句「欠账表」「等待」这类词不先剔掉，里面的「欠」「待」照算现欠说法
#       owed-refs-paid-status-sentence-green（期望退出 0，实测 1）
#   DOC_DECISIONS_BREAK=owed-refs-paid-status-ledgered-in-unseen  owed-refs-paid 现状句「记在 C<编号>」不算现欠说法
#       owed-refs-paid-status-sentence-red（输出里找不到「✗ .claude/kb/decisions/01-样本决策.md:5 C20（样本已还的甲）」）
#
# 格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的几份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析与汇总。格的出路里写的「doc-decisions.sh 里这一格函数上方的注释」，就在下面对应的那份库里（格在哪份库按格名列在下面）：
# 库 lib/doc-decisions-status-cells.sh：kb-shape、item-ref-status、status-redundancy、cross-decision-status、settled-item-self-open、settled-ref-says-open
# 库 lib/doc-decisions-status-cells.sh：stale-open-items、settled-same-file
# 库 lib/doc-decisions-registry-and-history-cells.sh：freeze-layer-membership、decision-items-sync、blocking-verdict、user-verdict-owed
# 库 lib/doc-decisions-registry-and-history-cells.sh：decision-summary-width、write-frequency、entry-added、shape、history-loss
# 库 lib/doc-decisions-evidence-and-owed-cells.sh：candidate-codes、inference-needs-verdict、three-way-leg-record、owed-refs-paid
# 弄坏开关的读法在入口的赋值与格函数里，格函数在 lib/ 下的库里；开关表仍在上面文件头。下面每个开关变量在哪份库的代码里被读，按库列在这里：
# 库 lib/doc-decisions-status-cells.sh：DOC_DECISIONS_BREAK、KB_NUMBERED_FILE_BREAK
# 库 lib/doc-decisions-registry-and-history-cells.sh：DOC_DECISIONS_BREAK
# 库 lib/doc-decisions-evidence-and-owed-cells.sh：DOC_DECISIONS_BREAK
# 入口的代码里也读的：DOC_DECISIONS_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：GEN_DECISION_ITEMS_BREAK、KB_DECISION_HEADS_BREAK、KB_HISTORY_SECTION_OWNER_BREAK、KB_ORPHAN_TRANSLATION_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：LAG_BASELINE_BREAK
set -uo pipefail
# 路径要在 cd 之前算：$0 是相对路径时，cd 进项目根之后就指不到了
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ doc-decisions.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_DIRECTORY="$STAGE_DIR"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_RELATIVE_PATH=".claude/gate.d/$(basename "${BASH_SOURCE[0]}")"
LIB_ITEM_REF_STATUS="$GATE_DIRECTORY/lib-item-ref-status.py"
# 挂账表与基准那一版比：取基准那一版的三种结局（有表、没有表、git 取不到）经它（DECISION_EVIDENCE_SOURCE 的 base_ledger_text，路径由环境变量 LIB_LAG_BASELINE 给）
LIB_LAG_BASELINE="$GATE_DIRECTORY/lib-lag-baseline.py"
# 格 owed-refs-paid（判据在那一格函数上方的注释）：欠账表开着 / 已还清的分界只读这一份；同一截前面看几个字、认哪几个词算写明了还清；滞后表与基准那一版比经上面的 LIB_LAG_BASELINE
LIB_OWED="$GATE_DIRECTORY/lib-owed.py"
OWED_REFS_PAID_PREFIX_CHARACTERS=12
OWED_REFS_PAID_MARKS="已还清 还清 销账"
# 格 owed-refs-paid 的滞后表（被判的根下）与锚至少几个字；写法在那张表的文件头与这一格函数上方的注释
OWED_REFS_PAID_LAG_TABLE=.claude/gate.d/owed-refs-paid-lag.tsv
# 格 owed-refs-paid 的现状句另扫的里程碑页（被判的根下；以数字起头的中文正文，译文不扫）
OWED_REFS_PAID_MILESTONE_DIRECTORY=.claude/kb/milestone
# 格 owed-refs-paid 的现状句：一句里带哪个词算把账当现欠（「记在 C<编号>」另由格里的正则认）、带哪个词算写明已还、哪几个带「欠」「待」的词先剔掉不算说法
OWED_REFS_PAID_STATUS_OWED_WORDS="欠 待 没做 未做 还开着 仍开着"
OWED_REFS_PAID_STATUS_PAID_MARKS="已还 还清 销账 已关 关账 做完 已做 已完成"
OWED_REFS_PAID_STATUS_NOT_CLAIMS="欠账表 等待 对待 期待 看待"
OWED_REFS_PAID_LAG_ANCHOR_CHARACTERS=12
LIB_CHANGED_PATHS="$REPO_ROOT/scripts/gate/changed-paths.sh"
# 生成器按本脚本所在的仓取、不按项目根：样本根里没有 .claude/scripts/（坑见格 blocking-verdict）
GENERATOR="$REPO_ROOT/.claude/scripts/gen-decision-items.py"
DECISION_ITEMS_WRITE=0
BREAK_SWITCH="${DOC_DECISIONS_BREAK:-}"
# 握手变量 GATE_NOT_RUN_FILE 只留给共用库汇总时写，不导出给各格的子进程（command-safety.md「进程边界上的三种静默失效」）
export -n GATE_NOT_RUN_FILE 2>/dev/null || true
source "$STAGE_DIR/lib/stage-cells.sh"
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

stage_cell kb-shape judge_kb_shape "kb 形状：分项只许一种叫法、kb 文件不链回自己、规则路径带 .claude/、决策标题与分项两节、索引页分项计数与进度计数与正文相符" \
  "照这一格各段红行下面的出路改（分项叫法、自指链接、规则路径、决策标题与分项两节、分项的进度标记、索引页状态列与进度列）；两列的计数以正文两节为准"
stage_cell item-ref-status judge_item_ref_status "每一处「D<n>（简称） 已定项 k / 未定项 k」写的状态与正文分项表相符" \
  "照红行把引用处写的「已定项 / 未定项」改成与那条决策正文分项表一致；分项改了号的，引用处跟着改号（scripts/kb/relabel-item.py）"
stage_cell status-redundancy judge_status_redundancy "状态别说两遍：小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行挂在对的节" \
  "状态只在一处说：删掉小节标题里的「—— 已定」与条目里多出的那一种写法，挂错节的分项挪到它状态对应的那一节"
stage_cell cross-decision-status judge_cross_decision_status "「D<n>（简称）」之后写「未定」（紧贴、隔着名词短语、「」引文里旁边没有现状），而那条决策的状态行不是待定" \
  "读那条决策现在定了什么，把这句改写成它的现状；在「」里要原样留着的旧写法，在引文之后的同一句或下一句写一句现状；改好的存量要把罩它的挂账行删掉（.claude/gate.d/decision-evidence-lag.tsv，改 gate.d 归工具实现员）"
stage_cell settled-item-self-open judge_settled_item_self_open "已定项区段里说它自己、或同一条决策里另一条已定分项还没定" \
  "这是定案之后没清理的推导过程：把这句改写成定案后的现状；真没定就把分项挪回未定项"
stage_cell settled-ref-says-open judge_settled_ref_says_open "引用写着「已定项 k」、归属到的那一条也已定，紧跟着却说它没定" \
  "那条分项已经定了：把句子改成它定下来之后的说法（定成了什么、这一处按它重算了没有），别只改标签"
stage_cell stale-open-items judge_stale_open_items "未定项点名的决策，状态行在这条未定项最近一次复核之后又变过" \
  "逐条复核：已被别处定掉的改写成「已定，权威记录在 XX」，仍然开着的把点名改成不构成依赖的写法"
stage_cell settled-same-file judge_settled_same_file "这次在一份决策里新增了「已定」小节（含「已定项 N：…」分项小节），同文件开着的未定项与索引页的待议节一个字没动" \
  "被顺带定掉的改写成「已定，权威记录在 XX」或收摊，仍然开着的在条目里补一行「YYYY-MM-DD 复核过，仍然开着，因为 …」；decisions.md 的待议节同样回看"
stage_cell freeze-layer-membership judge_freeze_layer_membership "冻结层归属登记表：每层每个结构都有行、写「退出」的必是派生态、依据指得到已定分项" \
  "照红行给缺的层与结构补一行（态别按 D21 已定项 10 的判据推，推不出写「判不动」）、写「退出」的行核态别、依据写成指得到的「D<n>（简称） 已定项 k」"
stage_cell decision-items-sync judge_decision_items_sync "decisions.md 的分项清单生成块与索引表「状态」「进度」两列，与生成器的输出逐字相同" \
  "权威记录是各决策正文的「### 已定项」/「### 未定项」两节；改完正文跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync 重写分项清单与索引表状态列、进度列"
stage_cell blocking-verdict judge_blocking_verdict "每条未定项都写了「改新池新建文件的字节」与「动不动格式」两把尺的判定" \
  "在每条缺判定的未定项的登记行或列表条目里补「改新池新建文件的字节：是 / 否 / 无对象（日期，依据：…）」与「动不动格式」那一句"
stage_cell user-verdict-owed judge_user_verdict_owed "决策正文里标着「待用户复核」的，checks-owed.md 里要有一笔未还的账点名那条决策" \
  "在 .claude/kb/checks-owed.md 立一笔未还的账点名那条决策的哪一项、原定案哪天由谁拍、复核要回答什么；已经复核完就把「待用户复核」改成结论"
stage_cell decision-summary-width judge_decision_summary_width "决策索引表每一行的「结论」列不超过 decisions.md 写的字数上限" \
  "结论列只写那条决策定了什么，为什么定、还欠什么搬进 decisions/ 下的正文"
stage_cell write-frequency judge_write_frequency "决策正文里落在登记结构上的频率断言，不与 write-frequency.tsv 登记的量级说反话" \
  "按登记行点名的权威分项改这句的频率；权威分项自己改了先改 .claude/gate.d/write-frequency.tsv 的登记；这句不是在说这个结构多久写一次，加进 .claude/gate.d/write-frequency-exclude.tsv 并写理由"
stage_cell entry-added judge_entry_added "决策正文改了，变更史里要有新增条目" \
  "在 .claude/kb/decisions-history.md 那条决策的 \`## D<n>（简称）\` 节里加条目（.claude/rules/changelog-format.md）；纯排版改动拆成单独一个提交"
stage_cell shape judge_shape "变更史的形状：日期块住在自己的节、同节同一天只一块、块下是 - 变更项而不是 #### 子条目、不留快查与现状行" \
  "日期写在 \`### \`，块下每条改动一个顶格 \`- \` 项，条目搬进自己的 \`## D<n>（\` / \`## E<n>（\` 节，同节同一天并成一个日期块；\`#### \`、「（其N）」、快查两行与现状行删掉（.claude/rules/changelog-format.md）"
stage_cell history-loss judge_history_loss "两份变更史与基准相比，基准里每个日期块的数、编号、产物名、引名在改后都找得到（提交前整理不许丢）" \
  "红行列出的数或编号逐个补回那一块、改成指针或 git 引用；确认是有意删的过程数才可以不补，那就在同一块里写一行「去向」说明"
stage_cell candidate-codes judge_candidate_codes "同一份决策里，同一个候选代号（甲乙丙丁… 或单个大写字母）不许出现在两个分项的候选表里" \
  "红行点名的代号，在后写的那个分项里换一个这份决策里没用过的代号；引用别的分项的候选时写「已定项 7 的丙」带上分项，带了分项前缀的写法不算复用"
stage_cell inference-needs-verdict judge_inference_needs_verdict "决策分项依据块里自陈是推论的条目（是推论、是推的、只走了本地腿、没走三方），要指到一份存在的 research/prompts/*-main-verification.md" \
  "在红行点名的那一条里写上判决文件（在树里，或在 git 历史里取得回来）；推论还没走三方的先走一轮三方，别先登挂账表 .claude/gate.d/decision-evidence-lag.tsv；改好的存量要把罩它的那一行挂账行删掉（改 gate.d 归工具实现员）"
stage_cell three-way-leg-record judge_three_way_leg_record "决策分项依据块里写了「三方」的条目，要指到 research/ 下存在的判决或腿输出，或写明腿况缺失且不写阴性结论" \
  "在红行点名的那一条里点名那一轮的判决或腿输出；取不回来的写明腿况缺失（没有腿况记录），并去掉「都没找到」「构造不出」这类阴性写法；改好的存量要把罩它的那一行挂账行删掉（.claude/gate.d/decision-evidence-lag.tsv，改 gate.d 归工具实现员）"
stage_cell owed-refs-paid judge_owed_refs_paid "决策中文正文「**欠**：」行里引的 C 号（紧跟在别的 C 号后面的全角括注里的不算）已在 checks-owed.md 的已还清表里，而同一行里这个号每一处到下一处引账或分号的那一截连同它前面 12 个字都没写已还清、还清、销账；决策中文正文欠段之外与里程碑页中文正文的现状句里引的已还清的账，那一句写了欠、待、没做、记在 C<编号> 这类现欠说法而没写已还、还清、销账、已关、做完这类写法的也判红；滞后表 owed-refs-paid-lag.tsv 登记的放过，滞后表写对了、与 gate 基准那一版比只缩不涨" \
  "把那个 C<编号>（简称）连同只说它的那一截从 **欠**： 行里去掉（去掉后这一行没有别的 C 号就写 **欠**：无。）；现状句写的是历史就在那一句写明已还，写的是现状就去掉把它当现欠的说法；那笔账其实没还清就把 checks-owed.md 那一行挪回欠着的那张表；要留着这一处引用就在同一截写明已还清；那份决策这一轮不许改的（在一轮还没判决的三方的开工快照里），照 .claude/gate.d/owed-refs-paid-lag.tsv 文件头的写法登记一行（只缩不涨，改 gate.d 归工具实现员），改好就删它那一行（判据在 doc-decisions.sh 里这一格函数上方的注释）"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

write_mode=0; root_argument=""
while (($#)); do
  case "$1" in
    --write) write_mode=1; shift ;;
    -*)
      bad "认不出的选项 $1"
      howto "只认 --list、--check <格名>[,<格名>…]、--write 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *)
      if [[ -n "$root_argument" ]]; then
        bad "给了两个项目根：$root_argument 与 $1"
        howto "参数只认一个项目根；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$REPO_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

# 按格组 source 下面几份库；它们读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取
source "$STAGE_DIR/lib/doc-decisions-status-cells.sh"
source "$STAGE_DIR/lib/doc-decisions-registry-and-history-cells.sh"
source "$STAGE_DIR/lib/doc-decisions-evidence-and-owed-cells.sh"

# ════ --write：在子 shell 里重写 decision-items-sync 的两块投影，写完回读判一遍 ════
if ((write_mode)); then
  write_cells=(decision-items-sync)
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    for write_cell in "${STAGE_CELLS_SELECTED[@]}"; do
      if [[ "$write_cell" != decision-items-sync ]]; then
        bad "--write 只重写 decision-items-sync（decisions.md 的分项清单与索引表状态列）管的投影，不能与 --check $write_cell 一起用"
        howto "去掉 --check，或写成 --check decision-items-sync：SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync"
        exit 2
      fi
    done
  fi
  echo "── decision-items-sync（--write）"
  if ( DECISION_ITEMS_WRITE=1; grid_decision_items_sync ); then write_exit=0; else write_exit=$?; fi
  if ((write_exit == 77)); then
    echo "  ! --write 写的 ${#write_cells[@]} 块都本次无对象可判（${write_cells[*]}），不记通过"
    exit 77
  fi
  if ((write_exit != 0)); then
    bad "--write 写了 ${#write_cells[@]} 块，写完回读仍红：${write_cells[*]}（退出 ${write_exit}）"
    howto "照上面那一块自己的红行改（节或索引行缺了 --write 补不了，先核编号），再跑 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync"
    exit 1
  fi
  ok "--write 写完 ${#write_cells[@]} 块、回读都对：${write_cells[*]}"
  exit 0
fi


# 这一道的格互不依赖、都只读被判的树：并行跑（.claude/gate.d/lib/stage-cells.sh 文件头「格并行」）
STAGE_CELLS_JOBS="${STAGE_CELLS_JOBS:-$(stage_cells_default_jobs)}"
stage_cells_run "$ROOT"
