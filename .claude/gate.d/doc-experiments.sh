#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 实验页与产物（十三格：索引行与正文标题、产物写回、产物与依据落在仓里、决策与实验双向登记、判决行的 false 被点名、复跑命令、实验号有正文、整行抄的产物行、上一轮记录已归档、多路径互证的登记、点名的路径找得到、跑前登记的并行与挂钟、重跑登记的比对对象表与机读登记逐行对得上且在单元表与复跑表里找得到）
# gate-fixture: yes
# gate-category: 文档类
# 不声明 gate-covers：未实现清单（.claude/gate-not-implemented.tsv）里的两项判被测代码的行为，这一道只判文本。
# gate-similar: checker-tier-research-build-and-replay.sh 它的 experiment-replay 格按 scripts/experiment-apparatus/replay.sh 登记表逐条复跑装置、逐字节比产物，要编译、归重型；这一道只读登记表、实验页与产物的文本，判产物有没有被点名、判决行的 false 有没有被解释、整行抄的产物行找不找得到，不跑装置
# gate-similar: doc-registries.sh 它的 experiment-refs 格判全 kb 里的实验号引用有定义，对象是引用；audit-contradictions 格核欠账表与里程碑收口表的引文在 kb 里找得到，方向是记录到 kb；segment-registry 格拿段序列登记表与 E142 的产物逐字比，对象是一份登记表。这一道判实验页、实验索引与产物本身，实验与决策之间的双向登记归这里的 decision-links 格，quoted-result-lines 格方向是 kb 与 research 正文到产物，multipath-registry 格还着 C48 可机检的那一半而不判欠账表本身
# gate-similar: doc-process-records.sh 也用 gate 取法的改动范围、也读 research/prompts/，但它判三方与同步留下的记录写没写、点没点名；这一道判产物与依据落没落进仓、新判决回看没回看决策、上一轮记录归没归档
# gate-similar: code-source-discipline.sh 它判每个实验二进制都有变异表、变异条目锚得住、钉绝对值的断言与读数纪律，对象是实验源码与变异表的内容；这一道的 evidence-in-repo 格只看这一轮改过的装置与变异表有没有一份不比它旧的产物
# gate-similar: code-tooling.sh 它的 change-range-single-source 格判各阶段取改动范围只经 changed-paths.sh 一份取法；这里是那份取法的使用者之一，改动范围在这一道里只取一次、八格共用
#
# 格名表（不给 --check 就按这个次序全跑）：
# gate-cell: index-sync 实验索引行与正文标题说的是不是同一件事
# gate-cell: results-cited 实验产物有没有写回
# gate-cell: evidence-in-repo 跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp
# gate-cell: decision-links 决策与实验双向登记，实验或结论变了之后回看过决策
# gate-cell: verdict-false-named 实验产物判决行里的 false 字段有没有被实验页点名
# gate-cell: repro-command 跑过的实验有没有留下复跑命令
# gate-cell: experiment-orphans 目录登记列的各处出现过的实验号在 kb 里有没有正文
# gate-cell: quoted-result-lines kb 与 research 正文里整行抄的产物行（含历史节里这次新增的行），产物里逐字找得到
# gate-cell: archive-past-rounds 上一轮及更早的实验记录已经归档进版本库，工作区里不留
# gate-cell: multipath-registry 多条路径互证的实验，路径与共用项要登记且指得到
# gate-cell: cited-paths-exist kb 全部正文点名的实验产物、变异表、实验源码路径，在树里或版本库历史里找得到
# gate-cell: preregistration-parallelism 这次改动新建或改动的 E186 起的跑前登记有「并行与挂钟」一节，独立单位、线程、串行与为什么、预计挂钟、预计在用核数五样写全，预计挂钟与态数乘单态耗时差不到一个数量级
# gate-cell: replay-comparison-units 重跑登记的比对对象逐行登记在机读表里：登记出处在、单元名在单元表出处里恰好出现一次、复跑驱动在 replay.sh 里且产物名相符；重跑登记里表头逐字的比对对象表与机读表逐行对得上
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码经 lib/stage-cells.sh；每格自己打判定行（成功行报查了多少项，红行带出路）。
#
# 样本 fixtures/doc-experiments.sh/：每格一对 <格名>-red / <格名>-green（cited-paths-exist-green 的 setup.sh 先提交再删掉一个产物，造「只在历史里」），另有 verdict-false-named-zero（登记表一行都读不到）、
# decision-links-formatting-green、decision-links-language-switch-green / -red（只改语言切换行 / 切换行与正文都改）、decision-links-switch-inserted-green / -red（只新插切换行加一个空行 / 再改正文一行）、decision-links-moved-and-switch-green / -red（登记的搬迁加切换行 / 再加正文一处真改动）、decision-links-no-experiment-green、decision-links-deleted-product-green、decision-links-translation-green、decision-links-pending-base-unreadable-red（基准那一版待回填清单的对象缺了）、quoted-result-lines-history-added-red、quoted-result-lines-history-old-green、quoted-result-lines-moved-history-green、quoted-result-lines-moved-unregistered-red、quoted-result-lines-moved-and-edited-red（历史节里只差搬迁的行）、evidence-in-repo-worktree-red / -green（链接 worktree）；每份样本根的 .gate-cells 只点名它判的那一格。
# 各样本放了什么写在那一格的注释里。
# replay-comparison-units 三份：replay-comparison-units-red（一份登记表各摆一种红：登记出处不在、单元表出处不在、单元名找不到、单元名出现两次、复跑驱动不在 replay.sh、产物名对不上、行不是六列，
# setup.sh 把基准提交里的登记表去掉两行复跑写 none: 的行，工作区照原样：none 行比基准多）、
# replay-comparison-units-green（驱动行两行、none 行一行，单元名「甲」与「乙甲」并存，登记表与基准相同）、replay-comparison-units-zero（登记表一行都没有，退 77）、
# replay-comparison-units-rekey-red（基准里四行 none 行；工作区三行的登记出处与留存产物名不变，实验号与单元名都换、只换单元名、只换实验号各一行，第四行原样：
# 三行判 none 行比基准多，原样那一行不报）。这四份的登记出处里各有一张与登记表逐行对得上的比对对象表（表头逐字），比对对象表那一条不另报红。
# 比对对象表的两份：replay-comparison-units-table-red（比对对象表 #3 登记表没有、文件头也没点名；一行登记的留存产物名在表里对不上；一行的登记出处里没有表头逐字的表；
# 文件头点名不登记的 #9 表里没有：四处逐条点名判红）、replay-comparison-units-table-green（表三行：一行对比对象、一行对旧登记那种只写前缀之后一段的重跑产物名、
# 一行在文件头点名不登记；围栏里另有一份举例的表头与一行，不算比对对象表）。两份的点名不登记行都写「欠在 C<号>（简称）」，样本里的 checks-owed.md 那个号开着。
# 点名不登记要指到开着的欠账：replay-comparison-units-exclusion-owed-red（表五行，一行登记对上；点名不登记四行：没写欠在哪、号不在欠账表里、号已还清各一行判红，
# 第四行欠在开着的 C92 不报）。
# preregistration-parallelism-wallclock-red / -wallclock-green：真仓 E186、E187、E197 的「## 十四、并行与挂钟」逐字抄进同号登记（红的各改坏一个数），判预计挂钟与态数乘单态耗时（放了什么写在那一格的注释里）。
# preregistration-parallelism-quantities-section-red / -quantities-section-green：「预计挂钟：」里的态数与第六节写的对不上的、对得上的与认不出的（放了什么写在那一格的注释里）。
# experiment-orphans-registry-red：样本根的目录登记末尾加一行 notes，notes/ 下的号与只登了索引行的号判成孤儿（登记里加一行，这一格跟着认）。
# 按编号取实验页的四份（实验页有中文原文 <号>-<名>.md 与译文 <号>-<名>.en.md / .ja.md，中文的索引与产物取中文原文，判法 lib-item-ref-status.py 的 numbered_files）：
# index-sync-translation-green / -red（索引结论列的数在中文原文里、译文里没有 / 只在英语译文里有），
# verdict-false-named-translation-green（E174 的中文原文没有「## 历史版本」、正文点了 false 字段的名，两份译文排在前面：绿；改前取到 .en.md、崩在 split_page 的拆包上）、
# verdict-false-named-no-history-red（中文原文没有历史节也没点名、英语译文点了名：红，不崩）。
# 产物只以 .out.gz 存在（超过单文件推送上限、压成 .gz 入库的）的六份样本：每份的 setup.sh 把样本里的 .out 用 gzip -n -9 压成 .out.gz 并删掉原文件，
# 三格读它们都是解压后读、判据不变：verdict-false-named-gz-green / -gz-red（解压内容里的 false 字段点了名 / 没点名，红行点名 .out.gz 与解压内容里的行号）、
# quoted-result-lines-gz-green / -gz-red（整行抄的产物行在解压内容里逐字找得到 / 有一行不在，红的汇总句数「有 1 行」）、
# repro-command-gz-green / -gz-red（页点名 .out.gz 产物并写了 cargo run / 点名了却只提到变异脚本）。
# cited-paths-exist 认 .out.gz 存放的两份样本（setup.sh 同样 gzip -n -9）：cited-paths-exist-gz-green（点名 .out，树里只有 .out.gz；另一份只在历史里有 .out.gz）、
# cited-paths-exist-gz-red（点名的 .out 树里与历史里 .out 与 .out.gz 都没有，另一份有 .out.gz 的点名了 .out，只报前一个）。
# 压成 .out.gz 之后点名与归档的四份样本（setup.sh 同样 gzip -n -9）：results-cited-gz-green（树里只有 .out.gz，一页点名 .out、一页点名 .out.gz，都算点名）、
# results-cited-gz-history-green（压缩产物只在历史里有 .out.gz，页里点名 .out 也算找得到）、results-cited-gz-red（点名的 .out 与 .out.gz 两样都不在，另一份 .out.gz 没人点名）、
# quoted-result-lines-archived-gz-green / -archived-gz-red（登记着的产物历史里只有 .out.gz，整行抄的行解压后逐字找得到 / 抄错一字，红的汇总句数「有 1 行」）。
# 弄坏开关（打开后样本判错；每条下面逐行点名判错的样本）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库 lib/stage-cells.sh：红格按绿记、整道退 0
#     archive-past-rounds-red、cited-paths-exist-gz-red、cited-paths-exist-red、decision-links-language-switch-red
#     decision-links-moved-and-switch-red、decision-links-pending-base-unreadable-red、decision-links-progress-mark-red、decision-links-red
#     decision-links-switch-inserted-red、evidence-in-repo-red、evidence-in-repo-worktree-red、experiment-orphans-red
#     experiment-orphans-registry-red、index-sync-red、index-sync-translation-red、multipath-registry-red
#     preregistration-parallelism-red、preregistration-parallelism-wallclock-red、quoted-result-lines-archived-gz-red、quoted-result-lines-gz-red
#     preregistration-parallelism-quantities-section-red
#     quoted-result-lines-history-added-red、quoted-result-lines-moved-and-edited-red、quoted-result-lines-moved-unregistered-red、quoted-result-lines-red
#     replay-comparison-units-exclusion-owed-red、replay-comparison-units-red、replay-comparison-units-rekey-red、replay-comparison-units-table-red
#     repro-command-gz-red、repro-command-red、results-cited-gz-red、results-cited-red
#     verdict-false-named-gz-red、verdict-false-named-no-history-red、verdict-false-named-red、verdict-false-named-zero
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本十三格全跑
#     archive-past-rounds-green、archive-past-rounds-red、cited-paths-exist-green、cited-paths-exist-gz-green
#     cited-paths-exist-gz-red、cited-paths-exist-red、decision-links-deleted-product-green、decision-links-formatting-green
#     decision-links-green、decision-links-language-switch-green、decision-links-language-switch-red、decision-links-moved-and-switch-green
#     decision-links-moved-and-switch-red、decision-links-no-experiment-green、decision-links-pending-base-unreadable-red、decision-links-progress-mark-green
#     decision-links-progress-mark-red、decision-links-red、decision-links-switch-inserted-green、decision-links-switch-inserted-red
#     decision-links-translation-green、evidence-in-repo-green、evidence-in-repo-red、evidence-in-repo-worktree-green
#     evidence-in-repo-worktree-red、experiment-orphans-green、experiment-orphans-red、experiment-orphans-registry-red
#     index-sync-green、index-sync-red、index-sync-translation-green、index-sync-translation-red
#     multipath-registry-green、multipath-registry-red、preregistration-parallelism-green、preregistration-parallelism-red
#     preregistration-parallelism-wallclock-green、preregistration-parallelism-wallclock-red、quoted-result-lines-archived-gz-green、quoted-result-lines-archived-gz-red
#     preregistration-parallelism-quantities-section-green、preregistration-parallelism-quantities-section-red
#     quoted-result-lines-green、quoted-result-lines-gz-green、quoted-result-lines-gz-red、quoted-result-lines-history-added-red
#     quoted-result-lines-history-old-green、quoted-result-lines-moved-and-edited-red、quoted-result-lines-moved-history-green、quoted-result-lines-moved-unregistered-red
#     quoted-result-lines-red、replay-comparison-units-exclusion-owed-red、replay-comparison-units-green、replay-comparison-units-red
#     replay-comparison-units-rekey-red、replay-comparison-units-table-green、replay-comparison-units-table-red、replay-comparison-units-zero
#     repro-command-green、repro-command-gz-green、repro-command-gz-red、repro-command-red
#     results-cited-green、results-cited-gz-green、results-cited-gz-history-green、results-cited-gz-red
#     results-cited-red、verdict-false-named-green、verdict-false-named-gz-green、verdict-false-named-gz-red
#     verdict-false-named-no-history-red、verdict-false-named-red、verdict-false-named-translation-green、verdict-false-named-zero
#   GATE_VERDICT_FALSE_SKIP_NAMED_CHECK=1  verdict-false-named 把每个 false 字段都当成已点名（变量名不以 _BREAK 收尾）
#     verdict-false-named-gz-red
#     verdict-false-named-no-history-red
#     verdict-false-named-red
#   KB_NUMBERED_FILE_BREAK=first-match  归 lib-item-ref-status.py：按编号取实验页不分语言、取排序后的第一个命中（改前的取法，取到 <号>-<名>.en.md）
#     index-sync-translation-green
#     index-sync-translation-red
#     verdict-false-named-no-history-red
#     verdict-false-named-translation-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-skips-whole-cell  evidence-in-repo 在链接 worktree 里整格退 77、判据二也不判
#     evidence-in-repo-worktree-green
#     evidence-in-repo-worktree-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=worktree-trusts-timestamps  链接 worktree 里照信检出时写的修改时刻
#     evidence-in-repo-worktree-green
#     evidence-in-repo-worktree-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=no-experiment-counted  decision-links 把依据段「无实验：」之后的 E<号>（ 也记成引用
#     decision-links-no-experiment-green
#     decision-links-progress-mark-green
#     decision-links-translation-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=progress-mark-counted  decision-links 量索引表定案格的字数时把行末「**进度：…**」也算进去
#     decision-links-progress-mark-green
#     decision-links-progress-mark-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=deleted-products-counted  decision-links 把这次改动里删掉的 research/results/e<号>… 也算「产物变了」
#     decision-links-deleted-product-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=read-translations  decision-links 的决策正文与实验页连译文一起读（改前）
#     decision-links-translation-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=language-switch-counted  decision-links 退回把实验页顶上第三行的语言切换行也算正文（改前）：只改了、只新插了这一行的页照「正文改了」要回看
#     decision-links-language-switch-green
#     decision-links-moved-and-switch-green
#     decision-links-switch-inserted-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=inserted-switch-counted  decision-links 去掉切换行时留着紧跟它的空行（改前：只认两版第三行都是切换行、只是改了）：新插切换行加一个空行的页照「正文改了」要回看
#     decision-links-switch-inserted-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=moves-not-rewritten  decision-links 比两版正文时基准那一版不照此刻生效的搬迁登记换算（改前：git show 原样读）
#     decision-links-moved-and-switch-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-added-lines-skipped  quoted-result-lines 退回历史节整节不判
#     quoted-result-lines-history-added-red
#     quoted-result-lines-history-old-green
#     quoted-result-lines-moved-and-edited-red
#     quoted-result-lines-moved-history-green
#     quoted-result-lines-moved-unregistered-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-lookup-skipped  cited-paths-exist 不查历史，树里没有的一律算缺
#     cited-paths-exist-green
#     cited-paths-exist-gz-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=existence-unchecked  cited-paths-exist 把每个点名的路径都当成在
#     cited-paths-exist-green
#     cited-paths-exist-gz-green
#     cited-paths-exist-gz-red
#     cited-paths-exist-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-lines-judged  evidence-in-repo 把两份变更史里与基准版逐字相同的历史行也判
#     evidence-in-repo-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=parallelism-section-presence-only  preregistration-parallelism 只看有没有这一节、不看五样与挂钟算式
#     preregistration-parallelism-red
#     preregistration-parallelism-quantities-section-green
#     preregistration-parallelism-quantities-section-red
#     preregistration-parallelism-wallclock-green
#     preregistration-parallelism-wallclock-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=parallelism-old-registrations-judged  preregistration-parallelism 号小于 186 的也判
#     preregistration-parallelism-green
#     preregistration-parallelism-red
#     preregistration-parallelism-wallclock-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=parallelism-wallclock-unchecked  preregistration-parallelism 不判挂钟算式（预计挂钟与态数乘单态耗时不比，也不报本次未判）
#     preregistration-parallelism-wallclock-green
#     preregistration-parallelism-wallclock-red
#     preregistration-parallelism-quantities-section-green
#     preregistration-parallelism-quantities-section-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=parallelism-unjudged-as-green  preregistration-parallelism 认不出态数或单态耗时的登记不报本次未判、悄悄当绿
#     preregistration-parallelism-wallclock-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=parallelism-quantities-section-unchecked  preregistration-parallelism 「预计挂钟：」的态数不与第六节比（不判红，也不报本次未判）
#     preregistration-parallelism-quantities-section-green
#     preregistration-parallelism-quantities-section-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-read-raw  verdict-false-named 与 quoted-result-lines 读 .out.gz 不解压、按原字节读
#     quoted-result-lines-archived-gz-green
#     quoted-result-lines-archived-gz-red
#     quoted-result-lines-gz-green
#     quoted-result-lines-gz-red
#     verdict-false-named-gz-green
#     verdict-false-named-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-citation-unmatched  results-cited 第一道要正文逐字点名 .out.gz、第三道只查原样的 .out（不查树里、HEAD 里、历史里的 .out.gz）
#     cited-paths-exist-gz-green
#     cited-paths-exist-gz-red
#     results-cited-gz-green
#     results-cited-gz-history-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=archived-gz-not-searched  quoted-result-lines 去历史里找归档产物时只认 .out 收尾的路径
#     quoted-result-lines-archived-gz-green
#     quoted-result-lines-archived-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-name-not-product  repro-command 不把 .out.gz 当产物名（只认 .out 收尾的）
#     repro-command-gz-green
#     repro-command-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=cited-path-gz-unmatched  cited-paths-exist 点名 <名>.out 只查原样的名字、不查压成 <名>.out.gz 的存放（树里、历史里）
#     cited-paths-exist-gz-green
#     cited-paths-exist-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unretained-note-ignored  cited-paths-exist 不认 kb 里某一行写明的「原件未留存 / 原始输出未留存」（那一行点名的路径别处再点名也不判）
#     cited-paths-exist-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-registration-unchecked  replay-comparison-units 不查登记出处在不在
#     replay-comparison-units-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-unit-unchecked  replay-comparison-units 不查单元表出处与单元名（含单元表出处不在）
#     replay-comparison-units-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-unit-count-loose  replay-comparison-units 的单元名出现一次以上都算找得到
#     replay-comparison-units-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-unit-substring  replay-comparison-units 的单元名不带双引号、按子串数（「甲」在「乙甲」里也数到）
#     replay-comparison-units-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-driver-unchecked  replay-comparison-units 不查驱动名在不在 replay.sh 复跑表里
#     replay-comparison-units-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-product-unchecked  replay-comparison-units 驱动行在就算过、不比第 4 列的产物名
#     replay-comparison-units-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-lag-growth-unchecked  replay-comparison-units 不拿登记表与基准那一版比（复跑写 none: 的行只缩不涨不判）
#     replay-comparison-units-red
#     replay-comparison-units-rekey-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-none-key-pair  replay-comparison-units 的只缩不涨退回按（登记出处，留存产物名）成对比（改前的键），换实验号、换单元名借同一对键照绿
#     replay-comparison-units-rekey-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-table-unchecked  replay-comparison-units 不解析重跑登记里的比对对象表、不与登记表逐行对（改前的盲区）
#     replay-comparison-units-exclusion-owed-red
#     replay-comparison-units-table-green
#     replay-comparison-units-table-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-table-suffix-unrecognized  replay-comparison-units 对重跑产物名只写前缀之后一段的（旧登记的写法）不认「留存产物名以 -<那一段> 收尾」
#     replay-comparison-units-table-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-exclusions-ignored  replay-comparison-units 不把登记表文件头「不登记的比对对象」那几行算点名（写法照判，只是不拿来放过表里的行）
#     replay-comparison-units-exclusion-owed-red
#     replay-comparison-units-table-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-exclusion-owed-unchecked  replay-comparison-units 不判点名不登记的理由指没指到开着的欠账（改前）
#     replay-comparison-units-exclusion-owed-red
#     replay-comparison-units-table-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-registry-ignored  experiment-orphans 不读目录登记，退回改前写死的四个目录（不看 kb 索引行，也不认登记里新加的一处）
#     experiment-orphans-registry-red
#   滞后表、豁免表与基准比的取法与复跑表的读法在 lib-lag-baseline.py（decision-links、multipath-registry、replay-comparison-units 三格取基准那一版，
#   results-cited、verdict-false-named、quoted-result-lines、replay-comparison-units 四格读复跑表）；它的开关在这一道会红的样本：
#   LAG_BASELINE_BREAK=unreadable-as-absent  归 lib-lag-baseline.py：git 取不到基准那一版当成「基准里没有这张表」，decision-links 把待回填清单整张算新加
#     decision-links-pending-base-unreadable-red
#   LAG_BASELINE_BREAK=moved-only-kept  归 lib-lag-baseline.py：与基准只差登记搬迁的文件与行照旧算改动（这一道开头经 changed-paths.sh 取的改动范围与新增行、quoted-result-lines 历史节的新增行都不剔）
#     quoted-result-lines-moved-history-green
#
#   bash .claude/gate.d/doc-experiments.sh [项目根]                              十三格按格名表的次序全跑
#   bash .claude/gate.d/doc-experiments.sh --list                                逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-experiments.sh --check <格名>[,<格名>…] [项目根]     只跑点名的格
#   格名写错、给了两个项目根、认不出的选项、进不去项目根都退 2。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_REPOSITORY_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
LIB_CHANGED_PATHS="$SCRIPT_REPOSITORY_ROOT/scripts/gate/changed-paths.sh"
ARCHIVE_SCRIPT="$SCRIPT_REPOSITORY_ROOT/scripts/kb/archive-past-rounds.py"
# 按编号取哪一份实验页（中文引用方取中文原文，不取排在前面的译文）：index-sync、verdict-false-named 调它的 numbered_files
LIB_ITEM_REF_STATUS="$STAGE_DIR/lib-item-ref-status.py"
# 滞后表与基准比、复跑表的行：decision-links、multipath-registry、replay-comparison-units 三格取基准那一版，results-cited、verdict-false-named、quoted-result-lines、replay-comparison-units 四格读复跑表，都调它
LIB_LAG_BASELINE="$STAGE_DIR/lib-lag-baseline.py"
# 实验号住在哪（目录登记 .claude/gate.d/experiment-number-directories.tsv 的读法）：experiment-orphans 调它，scripts/kb/claim-experiment.sh 的查号调同一份
LIB_EXPERIMENT_NUMBER_DIRECTORIES="$SCRIPT_REPOSITORY_ROOT/scripts/kb/lib_experiment_number_directories.py"
source "$STAGE_DIR/lib/stage-cells.sh"
# 要「这次改动」的八格：选中的格里有它们之一，才去取改动范围
CHANGE_RANGE_CELLS=" results-cited evidence-in-repo decision-links quoted-result-lines archive-past-rounds multipath-registry preregistration-parallelism replay-comparison-units "

stage_cell index-sync cell_index_sync "实验索引行与正文标题说的是不是同一件事" \
  "以正文标题为准改 .claude/kb/experiments.md 的索引行（作废、状态词、结论列的数跟着正文），两边一一对应"
stage_cell results-cited cell_results_cited "实验产物有没有写回" \
  "在 experiments.md 对应实验的口径段点名 research/results/ 下的产物；确实留不下的写明「原始输出未留存」与原因；点名了而树里与历史里都没有的改成现存的名字"
stage_cell evidence-in-repo cell_evidence_in_repo "跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp" \
  "跑一次把产物存进 research/results/ 并在 scripts/experiment-apparatus/replay.sh 指到它；把 /tmp 下当依据的东西拷进仓里（报告进 research/prompts/，数进 research/results/），引用改指仓内路径"
stage_cell decision-links cell_decision_links "决策与实验双向登记，实验或结论变了之后回看过决策" \
  "实验页补「### 影响的决策」表、决策分项的「**依据**」段引回实验，实验或结论变了就回看并在表里写日期与「改了 / 不受影响：理由」：.claude/rules/format-evolution.md「决策与实验双向登记」"
stage_cell verdict-false-named cell_verdict_false_named "实验产物判决行里的 false 字段有没有被实验页点名" \
  "在实验页那一次跑的历史条目里点名这个 false 字段并写明为什么是 false；replay.sh 登记表读不到行就先修表的形状"
stage_cell repro-command cell_repro_command "跑过的实验有没有留下复跑命令" \
  "在实验页正文的口径一节补一句复跑命令（格式与别处一致）；原始输出确实没留的写明「原始输出未留存」"
stage_cell experiment-orphans cell_experiment_orphans "目录登记列的各处出现过的实验号在 kb 里有没有正文" \
  "给这个实验号建正文 .claude/kb/experiments/<号>-*.md（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；实验号住在哪几处照 .claude/gate.d/experiment-number-directories.tsv，写法在它的文件头"
stage_cell quoted-result-lines cell_quoted_result_lines "kb 与 research 正文里整行抄的产物行（含历史节里这次新增的行），产物里逐字找得到" \
  "打开这个实验复跑命令登记的那份产物，把对应的 E7RESULT 行原样抄回来，别手改数；产物换了就连同引它的正文一起换"
stage_cell archive-past-rounds cell_archive_past_rounds "上一轮及更早的实验记录已经归档进版本库，工作区里不留" \
  "照这一格打出来的基准跑 python3 scripts/kb/archive-past-rounds.py --apply 删掉上一轮的记录（不带基准会把这一次提交要带的也删掉）"
stage_cell multipath-registry cell_multipath_registry "多条路径互证的实验，路径与共用项要登记且指得到" \
  "在实验页写「### 路径与结论登记」一节（形态见 .claude/rules/format-evolution.md），源码落点写现存路径、共用项写实；multipath-registry-lag.tsv 只缩不涨，补齐的删行"
stage_cell cited-paths-exist cell_cited_paths_exist "kb 全部正文点名的实验产物、变异表、实验源码路径，在树里或版本库历史里找得到" \
  "补回那份文件；或把那一行改成写明「原件未留存、从哪次提交取」；或改成现存的路径（移过位置的用 git log --all --diff-filter=A --name-only -- '*<文件名>' 找新路径）"
stage_cell preregistration-parallelism cell_preregistration_parallelism "这次改动新建或改动的 E186 起的跑前登记有「并行与挂钟」一节，独立单位、线程、串行与为什么、预计挂钟、预计在用核数五样写全，预计挂钟与态数乘单态耗时差不到一个数量级" \
  "照 .claude/agents/experiment-designer.md「登记的固定节名」的「## 十四、并行与挂钟」一行在登记里补这一节，五个标签行各一行（「独立单位：」「线程：」「串行：」「预计挂钟：」「预计在用核数：」，冒号后至少 8 个字，两行预计要带数、推的写依据）；预计挂钟与态数乘单态耗时差一个数量级以上的，照设计员定义第 3a 步现算后改挂钟或改错了的数，并行跑的写成式子「<态数> × <单态耗时> ÷ <并行数> ≈ <挂钟>」"
stage_cell replay-comparison-units cell_replay_comparison_units "重跑登记的比对对象逐行登记在机读表里：登记出处在、单元名在单元表出处里恰好出现一次、复跑驱动在 replay.sh 里且产物名相符；重跑登记里表头逐字的比对对象表与机读表逐行对得上" \
  "照 .claude/gate.d/replay-comparison-units.tsv 文件头的列义补齐或改正那一行（单元名照单元表里的字符串字面量逐字抄，复跑驱动与产物名照 scripts/experiment-apparatus/replay.sh 复跑表那一行抄）；确实没有复跑行的写 none:理由（复跑写 none: 的行与基准那一版比只缩不涨，新登记的比对对象要在 replay.sh 补驱动行）；重跑登记比对对象表里的每一行要有一行登记对上，单元表里没有单元的在那份文件头写一行「# 不登记的比对对象：<登记出处> #<号>：<理由>」"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2
      ;;
    *)
      if [[ -n "$root_argument" ]]; then
        echo "  ✗ 给了两个项目根：$root_argument 与 $1"
        echo "     → 怎么办：参数只认一个项目根与 --check <格名>[,<格名>…]；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"
      shift
      ;;
  esac
done
ROOT="${root_argument:-$SCRIPT_REPOSITORY_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

# ── 改动范围：经共用库 scripts/gate/changed-paths.sh 只取一次（基准 gate_diff_base gate），八格共用。
# 各格在自己的子 shell 里跑，所以在父进程里取、落进文件，各格只读。按 --check 选中的格判要不要取；样本根的 .gate-cells 在这之后才生效，那时按全部格取。
# 取新增行失败时：results-cited 与 quoted-result-lines 退回全量判，evidence-in-repo 判红。
# 没有上游、也没设 GATE_BASE 时基准是 HEAD，窗口只含工作区与暂存区：已提交没推的那几次不在里面。
changed_paths_library_loaded=0
in_git_work_tree=0
head_born=0
shared_base=""
changed_paths_ok=0
added_paths_ok=0
added_lines_ok=0
CHANGE_RANGE_DIRECTORY="$(mktemp -d)"
trap 'rm -rf "${CHANGE_RANGE_DIRECTORY:?}"' EXIT
SHARED_CHANGED_PATHS="$CHANGE_RANGE_DIRECTORY/changed-paths"
SHARED_ADDED_PATHS="$CHANGE_RANGE_DIRECTORY/added-paths"
SHARED_ADDED_LINES="$CHANGE_RANGE_DIRECTORY/added-lines"
: > "$SHARED_CHANGED_PATHS"; : > "$SHARED_ADDED_PATHS"; : > "$SHARED_ADDED_LINES"
change_range_needed=0
for cell in "${STAGE_CELLS_SELECTED[@]}"; do
  [[ "$CHANGE_RANGE_CELLS" == *" $cell "* ]] && change_range_needed=1
done
if ((change_range_needed)); then
  # shellcheck source=../../scripts/gate/changed-paths.sh
  if source "$LIB_CHANGED_PATHS"; then changed_paths_library_loaded=1; fi
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    in_git_work_tree=1
    if git rev-parse --verify -q HEAD >/dev/null 2>&1; then head_born=1; fi
    if ((changed_paths_library_loaded)); then
      shared_base="$(gate_diff_base gate)"
      if gate_changed_paths "$shared_base" untracked > "$SHARED_CHANGED_PATHS"; then changed_paths_ok=1; fi
      if gate_changed_paths "$shared_base" untracked A > "$SHARED_ADDED_PATHS"; then added_paths_ok=1; fi
      # research/ 只取 .md：未跟踪的产物可能很大，整份读一遍白费
      if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$SHARED_ADDED_LINES"; then added_lines_ok=1; fi
    fi
  fi
fi

# ── 格 index-sync：实验索引行与正文标题说的是不是同一件事
# 索引行与正文标题 `## E<n> 简称 —— 状态` 各写各的，就会一处改对、一处停在旧值，只检索到索引那一行的人拿到的是作废结论。
# 查四样：
#   1. 正文标题带「结论作废 / 结论整体作废」的，索引行里必须也出现「作废」
#   2. 正文标题的状态词是「未跑」「部分已跑」或「已跑（够判）」的，索引状态列必须跟着说
#   3. 索引结论列里的数，正文里要找得到
#   4. 两边一一对应：正文有而索引无、索引有而正文无都红
# 管不到：旧值以历史叙述的形态留在正文里时（「从 31.6M 抬到 31.8M」），第 3 项照样判它找得到。
# 样本：index-sync-red 里一行正文宣告作废而索引没提、一行索引结论列的数正文里找不到、一行正文「已跑（够判）」而索引没跟、
# 一行按子串会被「100」冒充找到的「10」；index-sync-green 里补零链接、结论整体作废两边都写了、够判两边都写了。
# 每个实验号的正文取哪一份：lib-item-ref-status.py 的 numbered_files，索引是中文，取中文原文（只有译文的号取那份译文，缺中文原文归 doc-text 的 trilingual-files）。
# 样本：index-sync-translation-green / -red（那个数在中文原文里 / 只在英语译文里）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_index_sync() {
  local IDX=.claude/kb/experiments.md
  local EXP=.claude/kb/experiments
  [[ -f "$IDX" && -d "$EXP" ]] || { echo "  ! 找不到 $IDX 或 $EXP，这一格跳过"; exit 77; }

  python3 - "$IDX" "$EXP" "$LIB_ITEM_REF_STATUS" <<'PY'
import importlib.util, re, sys, glob, os

idx_path, exp_dir = sys.argv[1], sys.argv[2]
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[3])
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)

def unspace(s):
    for ch in (" ", " ", " "):   # 窄不换行空格、窄空格、不换行空格
        s = s.replace(ch, " ")
    return re.sub(r"(?<=\d)[ 　](?=\d\d\d)", "", s)

bodies = {}
# 每个实验号只取一份正文：索引是中文，取中文原文（numbered_files）。改前按 glob 不排序的次序逐份覆盖，
# 留下哪一份（中文还是哪份译文）看目录项的次序。只有译文、没有中文原文的号照旧算「有正文」（取那份译文），缺中文原文归 doc-text 的 trilingual-files
all_pages = sorted(glob.glob(os.path.join(exp_dir, "*.md")))
for n in sorted({int(os.path.basename(p).split("-")[0]) for p in all_pages}):
    found = (item_ref_status.numbered_files(exp_dir, n, idx_path)
             or [p for p in all_pages if re.match(r"0*%d-" % n, os.path.basename(p))])
    p = found[0]
    text = open(p, encoding="utf-8").read()
    bodies[n] = (os.path.basename(p), text.splitlines()[0], unspace(text))

rows = []
for line in open(idx_path, encoding="utf-8"):
    if not line.startswith("| E"):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    m = re.match(r"E(\d+)", cells[0])
    if m and len(cells) >= 3:
        rows.append((int(m.group(1)), cells[1], cells[2], line))

def status_seg(title):
    seg = title.split("——", 1)[1] if "——" in title else title
    return seg.split("：", 1)[0]

bad = []
for n in sorted(set(bodies) - {r[0] for r in rows}):
    bad.append(f"E{n} 有正文（{bodies[n][0]}）却不在 experiments.md 的索引表里")
for n, st, concl, raw in rows:
    if n not in bodies:
        bad.append(f"E{n} 索引有行，{exp_dir}/ 下没有正文文件")
        continue
    fname, title, text = bodies[n]
    # 第 1 项只看状态列与结论列：简称里碰巧带「作废」二字不算索引说了这件事
    if re.search(r"结论(整体)?作废|结论已作废", title) and "作废" not in (st + concl):
        bad.append(f"E{n} 正文标题写着结论作废，索引行里没有「作废」二字（{fname}）")
    seg = status_seg(title)
    if seg.strip().startswith("未跑") and "未跑" not in st:
        bad.append(f"E{n} 正文状态是「未跑」，索引状态列写「{st}」")
    if "部分已跑" in seg and "部分" not in st:
        bad.append(f"E{n} 正文状态是「部分已跑」，索引状态列写「{st}」")
    if "够判" in seg and "够判" not in st:
        bad.append(f"E{n} 正文状态是「已跑（够判）」，索引状态列写「{st}」")
    c = re.sub(r"git show [0-9a-f]+:\S*", "", unspace(concl))
    for mm in re.finditer(r"\d[\d.]*", c):
        v = mm.group(0).rstrip(".")
        prev = c[max(0, mm.start() - 3):mm.start()]
        if re.search(r"[EDCIK\-]$", prev) or re.match(r"^20\d\d$", v) or len(v) <= 1:
            continue
        # 左边不许紧挨数字或小数点，整数右边不许紧挨数字：按子串比时「10 ms」会在「100 ms」里算找到。
        # 带小数的许正文多几位小数（索引写「10.6」、正文写「10.63」是索引按位截短，不算找不到）
        right_boundary = "" if "." in v else r"(?!\d)"
        if not re.search(r"(?<![\d.])" + re.escape(v) + right_boundary, text):
            bad.append(f"E{n} 索引结论列的「{v}」在 {fname} 正文里找不到")

if bad:
    print(f"  ✗ 实验索引与正文对不上 {len(bad)} 处：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                # gate-lint:detail
    print("  → 怎么办：以**正文标题**为准改索引行（编号的登记位是正文那行 `## E<n> 简称 —— 状态`，")
    print("    索引表只是导航）；若该改的是正文，就先改正文再回来改索引，并在")
    print("    .claude/kb/experiments-history.md 里按「改前 / 改后 / 依据」记一条。")
    print("    正文有而索引无的那一类：把它登记进 experiments.md 的索引表（编号、状态、一句话结论、正文链接）。")
    sys.exit(1)

if not rows and not bodies:
    print(f"  ! 本次无对象可判：{idx_path} 里一行 `| E<n>` 索引都没有，{exp_dir}/ 下也没有正文")
    sys.exit(77)
print(f"  ✓ 实验索引行与正文标题一致（索引 {len(rows)} 行、正文 {len(bodies)} 份）")
PY
}

# ── 格 results-cited：实验产物有没有写回
# 三道，判完再一起退出；三道各报各的对象数，都没有对象时退 77：
#   ① research/results/ 下（含子目录）每个普通文件（不只 *.out）要在实验索引或正文里被点名，顶层按文件名、子目录按相对路径；
#      不算产物的几类照名字跳过、成功行列名。
#   ② 标着已跑的实验要点名产物，或写明「原始输出未留存」。
#   ③ 这次改动新增或改写的点名，树里或版本库历史里要找得到。
# 样本：results-cited-red 里一份没被点名的产物、一份名字像逐轮中间件而登记了却没点名的产物、一个标着已跑却没点产物的实验、
# 一页未跟踪的新实验页点名了树里与历史里都没有的产物；results-cited-green 里点名齐、一份没登记的逐轮中间件列进没判的清单。
#
# 产物压成 <名>.out.gz 入库（超过单文件推送上限的）之后：第一道，树里只有 <名>.out.gz 时正文点名 <名>.out 或 <名>.out.gz 都算点名了它；
# 第三道，点名的 <名>.out（点 .out.gz 的同样取到 <名>.out）在树里、HEAD 里、历史里只以 <名>.out.gz 存在也算找得到（product_stored_forms 列两种存放）。
# 样本：results-cited-gz-green（产物只以 .out.gz 存在，一页点名 .out、一页点名 .out.gz，一份只在历史里有 .out.gz）、results-cited-gz-red（点名的 .out 树里与历史里 .out 与 .out.gz 都没有，另一份 .out.gz 没人点名）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-citation-unmatched：第一道要正文逐字点名 .out.gz、第三道只查原样的名字，两份绿样本判错。
product_stored_forms() {
  printf '%s\n' "$1"
  [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == gz-citation-unmatched ]] || printf '%s\n' "$1.gz"
}
cell_results_cited() {
  # 索引与各实验正文都要扫：产物可能被任一实验正文点名。
  EXP_DIR=.claude/kb/experiments
  EXP_ALL="$(mktemp)"; UNCITED="$(mktemp)"; NAMED="$(mktemp)"
  # 中间文件全交给 trap 删：判红那一支直接 exit，逐个手删会漏。
  trap 'rm -f "$EXP_ALL" "$UNCITED" "$NAMED"' EXIT
  cat .claude/kb/experiments.md "$EXP_DIR"/*.md > "$EXP_ALL" 2>/dev/null
  EXP="$EXP_ALL"
  RES=research/results
  [[ -s "$EXP" ]] || { echo "  ! 找不到实验正文，这一格跳过"; exit 77; }
  [[ -d "$RES" ]] || { echo "  ! 没有 $RES 目录，这一格无对象可判"; exit 77; }

  # 三道都判完再退出，一次把问题说全；判的工具自己没跑成（awk 出错）才当场退出。
  failed=0
  missing=()
  # replay.sh 登记的入库产物（复跑行经 lib-lag-baseline.py 的 replay-rows 取，与 verdict-false-named、quoted-result-lines、replay-comparison-units 三格同一份读法，
  # 行形状与 checker-tier-research-build-and-replay.sh 的 experiment-replay 格相同；这里取第 4 列的文件名）：名字像逐轮中间件（*.r<一位数>.out）的，登记了照样要被点名；
  # 读不了 replay.sh 时这种名字的产物全会被当成中间件跳过，所以有这种产物就判红。
  REPLAY_TABLE=scripts/experiment-apparatus/replay.sh
  replay_table_readable=1
  REGISTERED=""
  if [[ -r "$REPLAY_TABLE" && -f "$REPLAY_TABLE" ]]; then
    REGISTERED="$(python3 "$LIB_LAG_BASELINE" replay-rows "$REPLAY_TABLE" | cut -f4 | sed 's#.*/##')" || replay_table_readable=0
  else
    replay_table_readable=0
  fi
  judged_products=0
  not_products=()
  round_named_skipped=()
  while IFS= read -r f; do
    relative="${f#"$RES"/}"
    b="$(basename "$f")"
    # 三类不算产物：本地腿的问答、逐轮复跑的中间件（replay.sh 登记了的除外）、确认目录
    case "$relative" in
      *local*|*.round*|confirm*) not_products+=("$relative"); continue ;;
      *.r[0-9].out)
        case "$(printf "\n%s\n" "$REGISTERED")" in
          *"$(printf "\n%s\n" "$b")"*) : ;;
          *) not_products+=("$relative"); round_named_skipped+=("$relative"); continue ;;
        esac ;;
    esac
    judged_products=$((judged_products + 1))
    # 压成 <名>.out.gz 入库的产物：正文点名 <名>.out 或 <名>.out.gz 都算点名了它（点名的名字取产物压缩之前的那个，grep -F 找子串，所以点名 .out.gz 的同样命中）
    cited_name="$relative"
    if [[ "$relative" == *.out.gz && "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != gz-citation-unmatched ]]; then cited_name="${relative%.gz}"; fi
    grep -qF -- "$cited_name" "$EXP" || missing+=("$relative")
  done < <(find "$RES" -type f | sort)

  if ((${#missing[@]})); then
    echo "  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核："
    for m in "${missing[@]}"; do echo "     $RES/$m"; done
    echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
    echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
    failed=1
  elif ((judged_products == 0)); then
    echo "  ! 第一道本次无对象可判：$RES 下没有要点名的产物（不算产物的 ${#not_products[@]} 份除外）"
  else
    echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（查了 ${judged_products} 份）"
  fi
  if ((replay_table_readable == 0 && ${#round_named_skipped[@]})); then
    echo "  ✗ 读不到 $REPLAY_TABLE：它登记的入库产物认不出来，${#round_named_skipped[@]} 份名字像逐轮中间件的产物全被当成中间件跳过了"
    echo "     → 怎么办：replay.sh 搬了家或改了名，就把这里的 REPLAY_TABLE 一起改（.claude/rules/path-moves.md）；读权限丢了就补上。"
    failed=1
  fi
  if ((${#not_products[@]})); then
    echo "     第一道没判的 ${#not_products[@]} 份（本地腿的问答、没登记的逐轮中间件、确认目录）："
    printf '       %s\n' "${not_products[@]}"
  fi

  # ── 第二道：已跑的实验要么点名产物，要么写明产物没留 ──
  awk '
    /^## E[0-9]+ /{
      if (cur != "" && done && !cited && !excused) print cur
      cur=$2; done=($0 ~ /已跑|已测/); cited=0; excused=0
      if (done) ran_count++
      next
    }
    # 目录名后面要跟着一个文件名才算点名：只写「产物放在 research/results/ 下」的那一节什么都没点
    /research\/results\/[A-Za-z0-9_.-]/{ cited=1 }
    # 归档之后引用只剩文件名，所以裸文件名也算点名；它点的那份由第三道去树里与 git 历史里找。
    # 前面要有词边界：少了它，`-stage1.out` 里的 "stag|e1.out" 会被咬成一个产物名
    match($0, /(^|[^a-zA-Z0-9_-])e[0-9]+[a-zA-Z0-9._-]*\.out/) {
      tok=substr($0, RSTART, RLENGTH); sub(/^[^a-zA-Z0-9_-]/, "", tok)
      named[tok]=cur; cited=1
    }
    /原始输出未留存|输出未留存/{ excused=1 }
    END{ if (cur != "" && done && !cited && !excused) print cur
         for (n in named) print "NAMED" "\t" n "\t" named[n] > "/dev/stderr"
         print "RAN_COUNT" "\t" ran_count + 0 > "/dev/stderr" }
  ' "$EXP" > "$UNCITED" 2>"$NAMED" || {
    echo "  ✗ 判「已跑实验有没有点名产物」的 awk 没跑成（退出码 $?），这一格没判："
    sed 's/^/     /' "$NAMED"   # gate-lint:detail
    echo "     → 怎么办：看上面 awk 的报错修这一段；它没跑完时未点名的实验一个都不会被列出来，不许当成通过。"
    exit 1
  }
  if [[ -s "$UNCITED" ]]; then
    echo "  ✗ 这些实验标着已跑，却既没点名原始输出、也没说明产物为什么没留："
    sed 's/^/     /' "$UNCITED"
    echo "     → 怎么办：存一份产物并在口径段点名；确实留不下（要虚机/真设备）"
    echo "               就写明「原始输出未留存」以及为什么，别让读的人以为能翻到。"
    failed=1
  fi
  rm -f "$UNCITED"
  ran_count="$(sed -n 's/^RAN_COUNT\t//p' "$NAMED")"

  # ── 第三道：点名的产物，树里没有就必须在版本库历史里找得到 ──
  # 只判这次改动新增或改写的点名行（射程与 quoted-result-lines 格同一条）：早先的点名是历史；别的会话还没提交产物的实验页也不由这一次判。
  # 只看 .claude/kb/experiments.md 与 .claude/kb/experiments/ 下的新增行，未跟踪的实验页整份算新增；不在 git 仓里或取不到新增行就判全部。
  # 少了这一道，第二道就退化成「正文里写个像文件名的串」：从没存在过的文件名与归档进历史的产物在正文里长得一样。
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  BASE="" touched="" scope_note="全部的行（不在 git 仓里，判全部）"
  if ((in_git_work_tree)); then
    BASE="$shared_base"
    if ((added_lines_ok)); then
      touched="$(awk -F'\t' '$1 == ".claude/kb/experiments.md" || $1 == ".claude/kb/experiments" || index($1, ".claude/kb/experiments/") == 1' "$SHARED_ADDED_LINES" \
                 | cut -f2- | grep -oE "e[0-9]+[a-zA-Z0-9._-]*\.out" | sort -u)"
      scope_note="这次改动（基准 $BASE）新增或改写的行"
    else
      BASE="" scope_note="全部的行（取不到基准 $shared_base 起的新增行，判全部）"
    fi
  fi
  missing=0; checked=0; skipped=0
  declare -A skipped_names_by_owner=()
  while read -r _ name owner; do   # 字段是 NAMED、文件名、实验号，都不含空格，默认分隔够用
    [[ -n "$name" ]] || continue
    # 不用管道接 grep -q（pipefail 下退出码会被后一段盖掉）：$touched 是换行分隔的名字，两端各补一个换行再用 case 比整段。
    if [[ -n "$BASE" ]]; then
      case "$(printf "\n%s\n" "$touched")" in
        *"$(printf "\n%s\n" "$name")"*) : ;;
        *) skipped=$((skipped+1))
           skipped_names_by_owner["${owner:-（不在任何实验节里）}"]+=" $name"
           continue ;;
      esac
    fi
    checked=$((checked+1))
    # 树里、HEAD 里、历史里各查一遍，原样的 <名> 与压成 .gz 入库的 <名>.gz 两种存放都算在
    found_stored=0
    while IFS= read -r stored; do
      [[ -f "$RES/$stored" ]] && { found_stored=1; break; }
      # 正在归档的：HEAD 里还在、工作区里删了（这一批照归档规则删上一轮的产物，删除还没提交，下面的 git log 还找不到它）
      git -c core.quotepath=false cat-file -e "HEAD:$RES/$stored" 2>/dev/null && { found_stored=1; break; }
      if [[ -n "$(git -c core.quotepath=false log --all --diff-filter=D --format=%h --name-only -- "*$stored" 2>/dev/null | head -1)" ]]; then
        found_stored=1; break
      fi
    done < <(product_stored_forms "$name")
    ((found_stored)) && continue
    # 实验页里有一行同时写着这个产物名与「原始输出未留存 / 原件未留存」：如实交代过没留下，不算指空
    naming_lines="$(grep -rhF -- "$name" .claude/kb/experiments.md .claude/kb/experiments/ 2>/dev/null || true)"
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" != unretained-note-ignored ]] && grep -qE "原始输出未留存|原件未留存" <<<"$naming_lines"; then
      continue
    fi
    echo "     $owner 点名 $name —— 树里没有，git 历史里也没有"   # gate-lint:detail
    missing=$((missing+1))
  done < <(grep '^NAMED' "$NAMED" | sort -u)
  rm -f "$NAMED"
  if [[ $missing -gt 0 ]]; then
    echo "  ✗ $missing 份被点名的产物既不在 $RES 下、也不在版本库历史里"   # gate-lint:summary
    echo "     → 怎么办：产物归档了就该能从历史取回（git log --all --diff-filter=D --name-only 找删它的提交，"
    echo "               再 git show <提交>^:<路径> 读回来）；取不回来说明它从没存在过，"
    echo "               把那一节改成「原始输出未留存」并写明为什么，别让读的人以为翻得到。"
    failed=1
  elif ((checked == 0)); then
    echo "  ! 第三道本次无对象可判：$scope_note里没有点名产物的"
  else
    echo "  ✓ $scope_note点名的 $checked 份产物在树里或版本库历史里都找得到"
  fi
  if ((skipped)); then
    echo "     第三道没判的 $skipped 份：不在这次改动新增或改写的行里的点名，按实验列："
    while IFS= read -r skipped_owner; do
      echo "       $skipped_owner：${skipped_names_by_owner[$skipped_owner]# }"
    done < <(printf '%s\n' "${!skipped_names_by_owner[@]}" | sort -V)
  fi
  ((failed)) && exit 1
  if ((judged_products == 0 && ${ran_count:-0} == 0 && checked == 0)); then
    echo "  ! 本次无对象可判：没有要点名的产物、没有标着已跑的实验、这次改动里也没有点名产物的行"
    exit 77
  fi
  if ((${ran_count:-0} == 0)); then
    echo "  ! 第二道本次无对象可判：没有标着已跑的实验"
  else
    echo "  ✓ 已跑的 $((ran_count)) 个实验都点了名或写明了产物未留存"
  fi
}

# ── 格 evidence-in-repo：跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp
#
# 判据一「实验跑了没留存」：改动范围里还在盘上的 scripts/experiment-apparatus/e<号>_*.rs 或 scripts/mutation/tables/e<号>_*.tsv，
#   research/results/ 下（含子目录）要有一份这个实验号的产物（文件名 e<号> 后面跟非数字或到头），且 mtime 不早于源码。
#   登记了准入的实验（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）先按产物头记的实验键判（admission.py 的 stored_product_row_key_status：
#   实验键 = 复跑表那一行的行键去掉期望产物，算法在 admission.py 文件头「复跑表每一行的行键」）：对得上就绿、对不上就红，mtime 不作数；
#   一份带实验键的产物都没有（或今天的实验键拒算）才按 mtime 判。mtime 会被 git checkout / stash pop 刷新，实验键不会。
#   只改了 // 注释的装置源码不判、成功行列名：按 path-moves.md 改注释里的路径碰不到产物。
#   按要求才跑的实验（scripts/experiment-apparatus/on-request-experiments.tsv 登记的）不要求新产物：装置比产物新时，实验页这一轮新加的行里要写
#   「没有正式跑」或「未正式跑」；页里早先那句说的是上一次改装置，不算。
# 判据二「证据指向仓外」：.claude/kb/ 下全部 .md 与 research/prompts/ 下这一轮新写（新增或未跟踪）的 .md 里，
#   依据词（citation_form）后面接 /tmp/ 路径就红。
#   两半射程不同：kb 正文只写现状，旧文件照样该改；research/prompts/ 是原样保存的证据（evidence-discipline.md），只判还没冻的新写那一批。
#
# 链接 worktree 里（git-dir 落在 .git/worktrees/ 下；gate-staged.sh 与 --staged 的临时 worktree 就是）mtime 是检出时写的：
# 判据一里只能比 mtime 的那几份（产物在、实验键没判出对不上）不判，逐份列成本次未跑并报进 $GATE_NOT_RUN_FILE；别的照判，
# 所以提交路径上判据二照样判得到。全是要比 mtime 的、判据二又没有 .md 时退 77。
#
# 管不到：产物对不对得上源码（归 checker-tier-research-build-and-replay.sh 的逐字节复跑，它不看变异表的重跑日志）；
#   replay.sh 的登记行指没指到最新那份；句子是说做法还是引依据只按形状认（「草稿放 /tmp/…」不算，「报告原件在 /tmp/…，已转存」照样红）；
#   已提交的 prompts 文件后来加进的 /tmp 依据。
#
# 样本：evidence-in-repo-red 放一个产物比源码旧的装置、一个一份产物都没有的变异表、kb 里一处「依据：/tmp/…」、
# 一份新写的 prompts 里「报告 `/tmp/…`」、一个登记了准入而产物头记的实验键对不上今天的装置（E910，按 mtime 会判绿；样本里放了一份只有它一个 [[bin]] 的
# scripts/experiment-apparatus/Cargo.toml，实验键算得出），必须判红；
# evidence-in-repo-green 放产物比源码新的装置、不在改动范围里的旧装置、kb 里三种说做法的 /tmp 写法、改过但不是新写的 prompts 里的「依据：/tmp/…」、
# 登记了准入而产物头没记实验键、产物比源码新的装置（E909，按 mtime 判绿），必须判绿并报对数。产物头记的实验键与今天的相同就判绿（mtime 不作数）
# 那一路样本算不出（实验键要准入模块现算，setup.sh 不认得它在哪），由 python3 scripts/gate/admission.py --selftest 的「给 doc-experiments 的四种状态」那几格判。
# 链接 worktree 的一对（setup.sh 在样本根里建主仓 .origin-repository/，把它的一个链接 worktree 搬到样本根）：
# evidence-in-repo-worktree-red 放 kb 与新写的 prompts 里各一处 /tmp 依据、一个产物比源码旧的装置（E920），必须判红、点名两处、把 E920 列成本次未跑；
# evidence-in-repo-worktree-green 放一个产物比源码旧的装置（E921）与只说做法的 /tmp 写法，必须判绿、把 E921 列成本次未跑。
cell_evidence_in_repo() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，这一格跳过"; exit 77; }
  local timestamps_from_checkout=0
  if [[ "$(git rev-parse --git-dir 2>/dev/null)" == */worktrees/* ]]; then
    timestamps_from_checkout=1
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == worktree-skips-whole-cell ]]; then
      echo "  ! 弄坏开关 worktree-skips-whole-cell：$ROOT 是链接 worktree，整格退 77，判据二也不判"
      exit 77
    fi
    if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == worktree-trusts-timestamps ]]; then
      echo "  ! 弄坏开关 worktree-trusts-timestamps：$ROOT 是链接 worktree，照信检出时写的修改时刻"
      timestamps_from_checkout=0
    fi
  fi
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"
    exit 1
  fi
  base="$shared_base"
  if ((! changed_paths_ok || ! added_paths_ok)); then
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    exit 1
  fi
  # 按要求才跑的实验要读实验页这一轮新加的行
  if ((! added_lines_ok)); then
    echo "  ✗ 取不到实验页这一轮新加了哪些行（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到时按要求才跑的实验判不了，不是通过。"
    exit 1
  fi
  # 几份清单经文件与进程替换当文件传：当成命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - "$base" "$SHARED_CHANGED_PATHS" "$SHARED_ADDED_PATHS" "$(dirname "$LIB_CHANGED_PATHS")" \
    <(awk -F'\t' '$1 == ".claude/kb/experiments" || index($1, ".claude/kb/experiments/") == 1' "$SHARED_ADDED_LINES") "$timestamps_from_checkout" <<'PY'
import os, re, subprocess, sys
from datetime import datetime
from zoneinfo import ZoneInfo

base = sys.argv[1]
# argv[6] 为 1：在链接 worktree 里，修改时刻是检出时写的
timestamps_from_checkout = sys.argv[6] == "1"
# 实验键的算法只有 scripts/gate/admission.py 一份（门禁与实验共用），不在这里另算。
sys.path.insert(0, sys.argv[4])
try:
    import admission
except Exception as error:
    print(f"  ✗ 读不到准入模块 {sys.argv[4]}/admission.py：{error}")
    print("     → 怎么办：恢复 scripts/gate/admission.py（判「产物跟不跟得上输入」的实验键只有那一份算法），再跑这一格。")
    sys.exit(1)

def read_list(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return sorted({line for line in handle.read().split("\n") if line})

changed_files = read_list(sys.argv[2])
added_files = set(read_list(sys.argv[3]))

results_dir = "research/results"
kb_dir = ".claude/kb"
prompts_dir = "research/prompts"

source_forms = (
    (re.compile(r"^scripts/experiment-apparatus/e([0-9]+)_[^/]*\.rs$"), "装置源码"),
    (re.compile(r"^scripts/mutation/tables/e([0-9]+)_[^/]*\.tsv$"), "变异表"),
)
product_form = re.compile(r"^e([0-9]+)(?![0-9])")
on_request_list = "scripts/experiment-apparatus/on-request-experiments.tsv"
on_request_numbers = set()
if os.path.isfile(on_request_list):
    with open(on_request_list, encoding="utf-8") as handle:
        for line in handle:
            cells = line.split("#", 1)[0].split("\t")
            if cells[0].strip():
                on_request_numbers.add(cells[0].strip().lstrip("Ee"))
not_run_note = re.compile(r"没有正式跑|未正式跑")
# 实验页这一轮新加的行（gate_added_lines 的「路径<TAB>行文」）
added_experiment_page_lines = []
with open(sys.argv[5], encoding="utf-8", errors="replace") as handle:
    for added in handle.read().split("\n"):
        if "\t" in added:
            added_experiment_page_lines.append(tuple(added.split("\t", 1)))

def experiment_page_says_not_run(number):
    """按要求才跑的实验，这一轮有没有在实验页新写一句「装置改过、之后没有正式跑」。
    只认这一轮新加的行：页里早先那句说的是上一次改装置（门禁说明里提到这个词的那种句子也在其中），不给这一次作证。"""
    page_prefix = os.path.join(kb_dir, "experiments", f"{number}-")
    return any(page_path.startswith(page_prefix) and not_run_note.search(page_line)
               for page_path, page_line in added_experiment_page_lines)

# ── 判据一：改动范围里的实验装置与变异表，要有一份不比它旧的产物 ───────────
changed_sources = []
for path in changed_files:
    for form, kind in source_forms:
        match = form.match(path)
        if match and os.path.isfile(path):
            changed_sources.append((path, match.group(1), kind))
            break

newest_product_by_number = {}
if os.path.isdir(results_dir):
    for directory, _subdirectories, file_names in os.walk(results_dir):
        for file_name in file_names:
            match = product_form.match(file_name)
            if not match:
                continue
            product_path = os.path.join(directory, file_name)
            try:
                modified_at = os.path.getmtime(product_path)
            except OSError:
                continue
            number = match.group(1)
            previous = newest_product_by_number.get(number)
            if previous is None or modified_at > previous[1]:
                newest_product_by_number[number] = (product_path, modified_at)

TOKYO = ZoneInfo("Asia/Tokyo")

def moment(seconds):
    """修改时刻按东京的日期报（只报日子，不报钟点）：比新旧用的是秒，这里只是给人看的那一截。"""
    return datetime.fromtimestamp(seconds, TOKYO).strftime("%Y-%m-%d")

def changes_only_comments(path):
    """装置源码与基准比，增删的每一行都是 // 注释或空行：这种改动（多半是按 path-moves.md 改注释里的路径）碰不到产物。"""
    if path in added_files:
        return False
    diff = subprocess.run(["git", "-c", "core.quotepath=false", "diff", "-U0", base, "--", path],
                          capture_output=True, text=True, errors="replace")
    if diff.returncode != 0:
        return False
    changed_lines = [line[1:] for line in diff.stdout.split("\n")
                     if line[:1] in "+-" and not line.startswith(("+++", "---"))]
    return bool(changed_lines) and all(line.strip() == "" or line.lstrip().startswith("//") for line in changed_lines)

unstored_runs = []
comment_only_sources = []
on_request_not_run = []
on_request_missing_note = []
fingerprint_matched = []
fingerprint_absent = []
fingerprint_errors = []
timestamp_unjudged = []
for path, number, kind in changed_sources:
    if kind == "装置源码" and changes_only_comments(path):
        comment_only_sources.append(path)
        continue
    source_modified_at = os.path.getmtime(path)
    newest = newest_product_by_number.get(number)
    try:
        fingerprint_status, fingerprint_detail = admission.stored_product_row_key_status(".", f"E{number}")
    except (admission.InputManifestError, admission.RegistrationError) as error:
        fingerprint_errors.append(f"{path}（{kind}）：E{number} 登记了准入，却算不出实验键：{error}")
        continue
    if fingerprint_status == "matched":
        fingerprint_matched.append(f"E{number}（{path}；{fingerprint_detail}）")
        continue
    if fingerprint_status == "unfingerprinted":
        fingerprint_absent.append(f"E{number}")
    fingerprint_stale = fingerprint_status == "stale"
    if timestamps_from_checkout and not fingerprint_stale and newest is not None:
        # 产物在、实验键没判出对不上，剩下只能比修改时刻，而修改时刻是检出时写的：这一份本次未跑
        timestamp_unjudged.append(f"{path}（{kind}，E{number}）")
        continue
    if number in on_request_numbers and (fingerprint_stale or newest is None or newest[1] < source_modified_at):
        if experiment_page_says_not_run(number):
            on_request_not_run.append(f"E{number}（{path}）")
        else:
            on_request_missing_note.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：E{number} 按要求才跑，实验页没写明装置改过之后没有正式跑"
                                           "（只认这一轮新加进实验页的那一句）")
        continue
    if fingerprint_stale:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：E{number} 今天的实验键与产物头记的对不上，{fingerprint_detail}")
        continue
    if newest is None:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：{results_dir} 下一份 E{number} 的产物都没有")
    elif newest[1] < source_modified_at:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：最新的 E{number} 产物 {newest[0]} 还是 {moment(newest[1])} 那份")

# ── 判据二：kb 与这一轮新写的提示里，不许把 /tmp 路径当依据引用 ────────────
citation_form = re.compile(
    r"(依据|证据|出处|产物|报告|原件|结论|存档|留档|留存|记录|落点|详见|参见|见)"
    r"[ \t]{0,2}(?:是|在|为|：|:|=|→)?[ \t]{0,2}[`「]?"
    r"(/tmp/[^\s`\"'()（）「」，。；：、|]*)"
)
tmp_form = re.compile(r"/tmp/[^\s`\"'()（）「」，。；：、|]*")

scanned_files = []
skipped_prompt_files = 0
if os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if file_name.endswith(".md"):
                scanned_files.append(os.path.join(directory, file_name))
if os.path.isdir(prompts_dir):
    for directory, _subdirectories, file_names in os.walk(prompts_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            if path in added_files:
                scanned_files.append(path)
            else:
                skipped_prompt_files += 1

# 两份变更史里与基准版逐字相同的行是早先写下的历史条目（丢失核对要它们原样留着），按「这次改动」判，不判它们
HISTORY_CARRIERS = {os.path.join(kb_dir, "decisions-history.md"), os.path.join(kb_dir, "experiments-history.md")}
outside_citations = []
tmp_mentions = 0
history_lines_kept = 0
for path in sorted(set(scanned_files)):
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        continue
    lines_at_base = set()
    if path in HISTORY_CARRIERS and base and os.environ.get("EXPERIMENT_PAGES_AND_PRODUCTS_BREAK", "") != "history-lines-judged":
        shown = subprocess.run(["git", "-c", "core.quotepath=false", "show", f"{base}:{os.path.relpath(path)}"], capture_output=True, text=True)
        if shown.returncode == 0:
            lines_at_base = set(shown.stdout.split("\n"))
    for line_number, line in enumerate(lines, 1):
        tmp_mentions += len(tmp_form.findall(line))
        if line in lines_at_base and citation_form.search(line):
            history_lines_kept += 1
            continue
        for match in citation_form.finditer(line):
            excerpt = line[max(0, match.start() - 24):match.end()].strip()
            outside_citations.append(f"{path}:{line_number}：…{excerpt}")

if not changed_sources and not scanned_files:
    print(f"  ! 这次改动没碰实验装置与变异表（基准 {base}），{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，这一格无对象可判")
    sys.exit(77)
if changed_sources and len(timestamp_unjudged) == len(changed_sources) and not scanned_files:
    print(f"  ! 链接 worktree 里这一轮改过的 {len(changed_sources)} 份实验装置与变异表都要比修改时刻（检出时写的，判不了），"
          f"{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，这一格无对象可判：{'、'.join(timestamp_unjudged)}")
    sys.exit(77)

failed = False
if unstored_runs:
    failed = True
    print("  ✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物——这一跑多半只留在 /tmp 的草稿里：")  # gate-lint:summary
    for entry in unstored_runs:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：跑一次把产物存进 {results_dir}/，并在 scripts/experiment-apparatus/replay.sh 把这个实验的登记行指到它。")
    print("               这一跑已经跑过、产物还在 /tmp 的草稿目录里，就现在拷进来（会话一重启那个目录就没了）；")
    print("               只改了 // 注释的装置这一格不判；改了字符串里的路径，照 .claude/rules/path-moves.md 第 4 条连同留存产物一起改，复跑仍逐字相同。")
    print("               登记了准入的实验（.claude/gate.d/stage-inputs.tsv 有 E<号> 那一行）按产物头记的实验键判：实验键对不上就是产物跟不上今天的输入，")
    print("               重跑一次（装置入口放行并把新的实验键打在产物头），修改时刻新旧不作数。")
if fingerprint_errors:
    failed = True
    print("  ✗ 这些实验登记了准入，却算不出实验键，判不了产物跟不跟得上：")  # gate-lint:summary
    for entry in fingerprint_errors:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：单跑 python3 scripts/gate/admission.py experiment . E<号> 看它报什么；多半是 .claude/gate.d/stage-inputs.tsv 那一行的路径写错，")
    print("               或 cargo -V / rustc -V 跑不出来（bash .claude/scripts/env.sh 报缺什么）。")
if on_request_missing_note:
    failed = True
    print("  ✗ 这些按要求才跑的实验装置改了、没有新产物，实验页也没写明之后没有正式跑：")  # gate-lint:summary
    for entry in on_request_missing_note:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：不用跑（scripts/experiment-apparatus/on-request-experiments.tsv 里的实验只在用户说跑时才跑）；")
    print("               这一轮在实验页新写一句「装置某日改过（改了什么），之后没有正式跑」，要数的时候等用户说跑；页里早先那句说的是上一次改装置，不算。")
if outside_citations:
    failed = True
    print("  ✗ 这些地方把 /tmp 下的东西当依据引用了——那是会话自己的草稿目录，一重启就没了，三个月后没人核得动：")  # gate-lint:summary
    for entry in outside_citations:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：把 /tmp 下那份东西拷进仓里该在的位置（报告、提示与判决进 {prompts_dir}/，跑出来的数进 {results_dir}/），")
    print("               引用改成指那一份：在引用它的文件里把 /tmp 路径逐处换成仓内路径。")
    print("               原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。")
if timestamp_unjudged:
    print(f"    本次未跑 {len(timestamp_unjudged)} 份要比修改时刻的（链接 worktree 里全部文件的时间戳都是检出那一刻写的，分不出装置与产物谁新；"
          f"判据二与判据一里不看时间戳的照判了）：{'、'.join(timestamp_unjudged)}")
    print("    不带 --staged 在工作区里跑 bash .claude/gate.d/doc-experiments.sh --check evidence-in-repo 才判得了这几份。")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh 的 evidence-in-repo 格有 {len(timestamp_unjudged)} 份装置与变异表本次未跑"
                         f"（链接 worktree 里修改时刻是检出时写的）：{'、'.join(timestamp_unjudged)}\n")
if failed:
    sys.exit(1)

print(f"  ✓ 产物与依据都落在仓里（判了 {len(changed_sources) - len(timestamp_unjudged)} 份这一轮改过的实验装置与变异表、{len(set(scanned_files))} 份 .md，扫到 {tmp_mentions} 处 /tmp 路径、没有一处是引依据；基准 {base}）")
if not changed_sources:
    print("    没判：这次改动没碰 scripts/experiment-apparatus/ 与 scripts/mutation/tables/，判据一这一半没有对象")
print(f"    没判 {skipped_prompt_files} 份 {prompts_dir} 下这一轮没新写的 .md（登记在 .claude/doc-lint-exclude 的原样保存证据，改不得）")
if comment_only_sources:
    print(f"    没判 {len(comment_only_sources)} 份只改了 // 注释的装置源码（碰不到产物，不要求重跑）：{'、'.join(comment_only_sources)}")
if on_request_not_run:
    print(f"    没要求重跑 {len(on_request_not_run)} 份按要求才跑的实验（装置比产物新，实验页写明了没有正式跑）：{'、'.join(on_request_not_run)}")
if fingerprint_matched:
    print(f"    按实验键判 {len(fingerprint_matched)} 份（登记了准入，产物头记的实验键与今天的相同，修改时刻不作数）：{'、'.join(fingerprint_matched)}")
if fingerprint_absent:
    print(f"    按修改时刻判 {len(fingerprint_absent)} 份登记了准入、research/results/ 里却还没有一份带实验键产物的（或今天的实验键拒算）：{'、'.join(fingerprint_absent)}")
PY
}

# ── 格 decision-links：决策与实验双向登记，实验或结论变了之后回看过决策
#
# 规则在 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。判据：
#   ① 实验页（待回填清单之外的）有「### 影响的决策」一节，表头 | 决策分项 | 关系 | 回看 |；
#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。
#   ② 表里每一行：决策真实存在，写了分项的那条分项也存在；关系是 支撑 / 推翻 / 备料 / 不影响 之一；
#      回看格以日期开头，后跟「改了」或「不受影响：理由」。
#   ③ 双向：关系是支撑或推翻的那条分项，它的「**依据**」段要引回这个实验；反过来，
#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。
#   ④ 时效（全量）：实验最近一次变动的日期——页内历史节带日期的 ### 标题、experiments-history.md 里标题
#      （标题没点名实验时看条目正文）点名它的条目——晚于表里某一行的回看日期，判红。
#   ⑤ 按这次改动（这一道开头取的那一份）：
#      实验页正文（历史版本与影响的决策两节之外）改了、或 research/results/e<号>… 的产物新增或改写了（删掉的不算：归档删上一轮的产物不是结论变了），
#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md
#      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
#      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，
#      决策正文（历史版本之前）引了它的每条决策（`E<号>（`，粗体、反引号、空格、半角括号的写法也认）都要回看：
#      这次改动在那份决策里新增或改写的行点到了它，或者表里那条决策的一行改过。搬了家的页按实验号找基准那一版的标题。
#   ⑥ 待回填清单 .claude/decision-links-pending：一行一个 E<号> 或 D<号>，# 后写理由；指到的要存在；
#      只减不增——比基准那一版多出来的行判红；基准里还没有这张清单时整张都算新加，git 取不到基准那一版判红
#      （取法 lib-lag-baseline.py 的 baseline_rows，红行句式是它的 BaselineProblem.sentence；样本 decision-links-pending-base-unreadable-red）。
#   认不出标题的实验页（首个二级标题不是「## E<号> 」）与决策文件（不是「## D<号> 」）判红，不静默跳过；
#   ⑤ 取某一页的新增行时 git diff 失败，判红，不当成那一页没改。
#   ⑦ 瘦身形态（待回填清单之外的决策）：首行 `## D<号> 简称 —— 状态` 后面不带括注；分项标题不带日期；
#      每个已定项有「**定案**：」「**射程**：」「**依据**：」「**欠**：」四块；索引表的定案格不带日期、不超过 100 字
#      （行末「**状态：…**」「**进度：…**」两个规范标记与未定项登记行那两句判定不算字数、不判日期）。
#   ⑧ 有实验才有决策：已定项的依据段至少引一个实验，或写「无实验：理由」（理由至少八个字）；
#      成功行报出写「无实验」的已定项有几个。「无实验：」到行尾是理由，理由里点名的实验（`E<号>（`）不算这条分项引的依据，
#      不进③的双向检查：不要求那个实验页回指这条分项；「无实验：」之前、或依据段别的行里引的实验照算。
#   ⑨ 实验必须对应决策：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
#      标题里写着「结论……作废」或「结论……退役」的实验不判（它们留着是为了记下那条路不通）。
#
# 管不到的：回看写的理由对不对、关系判得对不对（支撑写成不影响也过得了③以外的判据），靠人与抽查；
# 实验正文只用编号不用「D<号>（」形态提到的决策，①看不见。
#
# 只删与所属标题相同的行内日期、只把同一天的第二个小节并进第一个的改动，不算「正文改了」（判法借 scripts/kb/doc-consolidate.py，
# 两版整理后的正文按行排序逐字相同就是），样本 decision-links-formatting-green 判绿、成功行报「不算正文改了 1 份」。
# 只改了顶上第三行语言切换行的改动也不算「正文改了」：切换行是三语导航（写法照 .claude/skills/trilingual/SKILL.md「改一组的做法」第 3 条的三种，
# 链接目标是 <名>.md / .en.md / .ja.md），不是实验的结论、数与产物。两版各自去掉这一行之后，历史版本之前、影响的决策一节之外的正文逐字相同就是；
# 第三行认不出这三种形态的照旧算正文。样本 decision-links-language-switch-green（只改切换行的链接目标）判绿、成功行报「只改语言切换行的改动 1 份」；
# decision-links-language-switch-red（切换行与正文一行都改了）照旧判红在「没回看」。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=language-switch-counted 退回把切换行算正文，decision-links-language-switch-green 判红在「没回看」。
# 旧版第三行没有切换行、新版在第三行新插了切换行（加紧跟的一个空行）的，与改了一样不算正文：去掉切换行时连同紧跟它的那一个空行一起去掉，
# 旧版没有切换行时什么都不去，两版这样处理之后逐字相同才算，计进同一个「只改语言切换行的改动」。样本 decision-links-switch-inserted-green
# （旧版没有切换行，新版插进切换行加一个空行）判绿、成功行报「只改语言切换行的改动 1 份」；decision-links-switch-inserted-red（插进切换行，正文一行也改了）
# 照旧判红在「没回看」。弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=inserted-switch-counted 退回只去掉切换行、留着紧跟的空行，
# decision-links-switch-inserted-green 判红在「没回看」。
# 两版正文（切换行、日期整理这两种判法）比之前，基准那一版先照此刻生效的搬迁登记换算（baseline_page_text，调 lib-lag-baseline.py 的 baseline_text：
# 只按路径前缀换、提交引用后面的不换、页自己搬了家按搬迁之前的路径取；边界规则归那份库，这里不另写）：页里一行因登记的搬迁换了路径、
# 同时顶上切换行也改了的，只差这两样，不算正文改了（只差搬迁的整份由 changed-paths.sh 剔出改动范围，到不了这里）。
# 基准里没有这一页、或 git 取不到 / 登记表写坏了：前者按正文改了判，后者另报「取不到基准」一条。
# 样本 decision-links-moved-and-switch-green（一行登记的搬迁，含 `cd scripts/experiments &&` 这种后面不跟斜杠的写法，加切换行）判绿、成功行报「只改语言切换行的改动 1 份」；
# decision-links-moved-and-switch-red（同样两样再加正文一处真改动）照旧判红在「没回看」。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=moves-not-rewritten 去掉这一步换算，decision-links-moved-and-switch-green 判红在「没回看」。
# 样本：decision-links-red 在一个临时仓里把上面每一条各犯一次（want 逐条点名）；decision-links-green 放瘦身的与没瘦身的决策、
# 待回填清单、结论作废的实验、这次改动改了正文与产物又回看了表、新判决带「## 回看决策」、搬了家的实验页，必须判绿并报对数。
# decision-links-no-experiment-green 放一条依据写「无实验：」、理由里点名一个实验（只有备料关系、没有回指这条分项）的已定项，必须判绿；
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=no-experiment-counted 退回把理由里的实验也记成引用，这份样本判红（不对称）。
# decision-links-progress-mark-green 的索引行去掉「**状态：已定。**」「**进度：已实现。**」之后 95 字、带上是 104 字，必须判绿；
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=progress-mark-counted 下把进度标记算进字数，这份样本判红。
# decision-links-progress-mark-red 的索引行去掉两个标记之后 101 字，照样判红在那一行（现在 101 字）。
# decision-links-deleted-product-green 的这次改动只删了一份已提交的产物（research/results/e1-sample.out），实验页与决策表一行没动，必须判绿、
# 报「这次改动触发回看 0 处」；弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=deleted-products-counted 下删除照旧算产物变了，判红在「没回看」。
# 决策正文与实验页只读中文原文（lib-item-ref-status.py 的 originals_only）：译文的首行、分项小节、依据段与「影响的决策」表头是各自的语言。
# decision-links-translation-green 是 decision-links-no-experiment-green 加上 D1、E5 的英语与日语译文（首行写成各自的语言，日语那两份没有 `## D1 ` / `## E5 ` 的空格），
# 必须判绿、报数同那一份；弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=read-translations 照改前连译文一起读，判红在「认不出标题」「没有表」。
# decision-links-pending-base-unreadable-red 的现场与 decision-links-green 相同，setup.sh 末尾删掉基准那一版待回填清单的对象：判红在「读不出基准 HEAD 里的
# .claude/decision-links-pending（git show 退 128」；弄坏开关 LAG_BASELINE_BREAK=unreadable-as-absent 下当成基准里没有这张清单，红在「新加了」，那一条 want 找不到。
cell_decision_links() {
  if [[ ! -d .claude/kb/experiments || ! -d .claude/kb/decisions ]]; then
    echo "  ✗ 找不到 .claude/kb/experiments 或 .claude/kb/decisions"
    echo "  → 怎么办：kb 挪了位置就同步改这一格里的路径。"
    exit 1
  fi
  base="HEAD"
  in_git=0
  changed_list=/dev/null
  if ((in_git_work_tree && head_born)); then
    in_git=1
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
    if ((! changed_paths_ok)); then
      echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
      echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
      exit 1
    fi
    changed_list="$SHARED_CHANGED_PATHS"
  fi
  # 只删同日日期、只并同日小节的改动不算「正文改了」：判法借 scripts/kb/doc-consolidate.py（门禁 doc-text 的 date-once 格用同一份）；
  # 只改了顶上语言切换行的也不算，判法在下面的 language_switch_only_change
  DOC_CONSOLIDATE="$(dirname "$LIB_CHANGED_PATHS")/../kb/doc-consolidate.py"
  export DOC_CONSOLIDATE
  LIB_ITEM_REF_STATUS="$LIB_ITEM_REF_STATUS" LIB_LAG_BASELINE="$LIB_LAG_BASELINE" python3 - "$ROOT" "$base" "$in_git" "$changed_list" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

root, base, in_git = sys.argv[1], sys.argv[2], sys.argv[3] == '1'
item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', os.environ['LIB_ITEM_REF_STATUS'])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
# 待回填清单与基准那一版比（⑥）：取基准那一版的三种结局经 lib-lag-baseline.py 的 baseline_rows，不在这里另取
lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', os.environ['LIB_LAG_BASELINE'])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
# 决策正文与实验页只读中文原文（lib-item-ref-status.py 的 originals_only）：译文的首行、分项小节、依据段与「影响的决策」表头都是各自的语言，
# 读进来就报「认不出标题」「没有表」，读得出的还靠文件名排序让中文那份最后覆盖。弄坏开关 read-translations 照改前连译文一起读
read_translations = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'read-translations'
changed = {line for line in open(sys.argv[4], encoding='utf-8').read().split('\n') if line}
kb = os.path.join(root, '.claude', 'kb')
RELATIONS = ('支撑', '推翻', '备料', '不影响')
BASIS_RELATIONS = ('支撑', '推翻')
if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'no-experiment-counted':
    print('  ! 弄坏开关 no-experiment-counted：依据段里「无实验：」之后的 E<号>（ 也记成引用')
E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')
D_REF = re.compile(r'(?<![A-Za-z0-9])D(\d+)（')
# 「决策正文引了它」按 doc-lint 认引用的写法认：粗体、反引号、编号与括号之间有空格、半角括号都算
MENTION_REF = re.compile(r'(?<![A-Za-z0-9._-])[*`]*E(\d+)[*`]*\s*[（(]')
TABLE_HEAD = re.compile(r'^\|\s*决策分项\s*\|\s*关系\s*\|\s*回看\s*\|\s*$')
REVIEW = re.compile(r'^(20\d\d-\d\d-\d\d)\s*(改了|不受影响：\s*\S.{3,})')
DATE_HEAD = re.compile(r'^### (20\d\d-\d\d-\d\d)')


def read(path):
    return open(path, encoding='utf-8').read()


def history_start(lines):
    for index, line in enumerate(lines):
        if re.match(r'^## 历史版本\s*$', line):
            return index
    return len(lines)


def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], cwd=root, capture_output=True, text=True)


def split_cells(line):
    return [cell.strip() for cell in re.sub(r'\\\|', '\x00', line.strip()).strip('|').split('|')]


def parse_decision_ref(cell):
    """「D28（挂载期承诺量） 已定项 2」→ (28, ('已定项', 2))；只写决策 → (28, None)；认不出 → None。"""
    head = re.match(r'^\**D(\d+)（', cell)
    if not head:
        return None
    item = re.search(r'(已定项|未定项)\s*(\d+)\s*\**$', cell)
    return int(head.group(1)), ((item.group(1), int(item.group(2))) if item else None)


def section_range(lines, heading_pattern, stop_level):
    """标题行号与它那一节的结束行号（下一个级别 ≤ stop_level 的标题之前），找不到返回 None。"""
    for index, line in enumerate(lines):
        if re.match(heading_pattern, line):
            for end in range(index + 1, len(lines)):
                level = re.match(r'^(#{1,6}) ', lines[end])
                if level and len(level.group(1)) <= stop_level:
                    return index, end
            return index, len(lines)
    return None


def table_rows(lines, start, end):
    """一节里那张影响的决策表的行：[(行号, 单元格)]；没有表头返回 None。"""
    rows, seen_head = [], False
    for index in range(start + 1, end):
        line = lines[index]
        if TABLE_HEAD.match(line.strip()):
            seen_head = True
            continue
        if seen_head and re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', line.strip()):
            continue
        if seen_head and line.strip().startswith('|'):
            rows.append((index, split_cells(line)))
        elif seen_head:
            break
    return rows if seen_head else None


problems = []   # (类别, 说明)


def bad(kind, text):
    if (kind, text) not in problems:   # 同一份文件被几处调到时（git 失败那一类）只报一次
        problems.append((kind, text))


# ── 决策：分项与各分项的依据段 ──
decisions = {}
for path in item_ref_status.originals_only(sorted(glob.glob(os.path.join(kb, 'decisions', '*.md'))), read_translations):
    lines = read(path).split('\n')
    body = lines[:history_start(lines)]
    head = next((re.match(r'^## D(\d+) ', line) for line in body if re.match(r'^## D(\d+) ', line)), None)
    if not head:
        # 认不出标题的决策文件不许静默跳过：跳过就不计数、不列名，这份决策的分项与依据一条都没判
        bad('认不出标题', f'{os.path.relpath(path, root)}：「## 历史版本」之前没有一行写成「## D<号> 简称 —— 状态」，这份决策一条都没判')
        continue
    number = int(head.group(1))
    items, basis, shapes, index_cells = set(), {}, [], []
    for index, line in enumerate(body):
        item = re.match(r'^(#{3,4}) (已定项|未定项) (\d+)', line)
        if not item:
            continue
        key = (item.group(2), int(item.group(3)))
        items.add(key)
        cited, in_basis, labels, basis_text = set(), False, set(), ''
        for inner in body[index + 1:]:
            level = re.match(r'^(#{1,6}) ', inner)
            if level and len(level.group(1)) <= len(item.group(1)):
                break
            label = re.match(r'^\*\*(定案|射程|依据|欠)\*\*：', inner)
            if label:
                labels.add(label.group(1))
            if inner.startswith('**依据**'):
                in_basis = True
            elif re.match(r'^\*\*[^*]{1,12}\*\*：', inner) or level:
                in_basis = False
            if in_basis:
                # 「无实验：」到行尾是不引实验的理由，理由里点名的实验不是这条分项引的依据，不进 cited（C535）
                scanned = inner
                cut = inner.find('无实验：')
                if cut >= 0 and os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') != 'no-experiment-counted':
                    scanned = inner[:cut]
                cited |= {int(e) for e in E_REF.findall(scanned)}
                basis_text += inner + '\n'
        basis[key] = cited
        shapes.append((key, line, labels, cited, basis_text))
    for section in ('已定项', '未定项'):
        found = section_range(body, r'^### %s\s*$' % section, 3)
        if found:
            for line in body[found[0] + 1:found[1]]:
                if line.startswith('#### '):
                    break
                row = re.match(r'^\|\s*(\d+)\s*\|', line)
                if row:
                    items.add((section, int(row.group(1))))
                    index_cells.append((section, int(row.group(1)), split_cells(line)[-1]))
                # 没瘦身的决策有的用编号列表列分项（`9. **名字**…`），doc-decisions.sh 数分项时也认这种写法
                listed = re.match(r'^(\d+)\.\s', line)
                if listed:
                    items.add((section, int(listed.group(1))))
    title = next(line for line in body if re.match(r'^## D(\d+) ', line))
    decisions[number] = {'path': os.path.relpath(path, root), 'items': items, 'basis': basis,
                         'shapes': shapes, 'index_cells': index_cells, 'title': title,
                         'mentions': {int(e) for e in MENTION_REF.findall('\n'.join(body))}}

# ── 待回填清单 ──
pending_path = os.path.join(root, '.claude', 'decision-links-pending')
pending_rel = '.claude/decision-links-pending'
pending = set()
pending_lines = read(pending_path).split('\n') if os.path.exists(pending_path) else []
for number, line in enumerate(pending_lines, 1):
    if not line.strip() or line.lstrip().startswith('#'):
        continue
    entry = re.match(r'^(E|D)(\d+)\s+#\s*(\S.{3,})$', line.strip())
    if not entry:
        bad('待回填', f'{pending_rel}:{number}：写成「E<号>  # 理由」或「D<号>  # 理由」，理由至少四个字——「{line.strip()[:30]}」')
        continue
    pending.add((entry.group(1), int(entry.group(2))))
if in_git and os.path.exists(pending_path):
    # 「基准里还没有这张清单」与「git 取不到基准」要分开：前者整张都算新加，后者判不了只减不增，两种都不许静默放过（lib-lag-baseline.py 的三种结局）
    previous_rows, baseline_problem = lag_baseline.baseline_rows(base, pending_rel, cwd=root)
    before, base_note = None, ''
    if baseline_problem is not None:
        bad('待回填', f'{baseline_problem.sentence()}，「只减不增」这一条没判')
    elif previous_rows is None:
        before, base_note = set(), f'（基准 {base} 里还没有这张清单，这次改动新建了它，整张都算新加）'
    else:
        before = set()
        for _number, fields in previous_rows:
            entry = re.match(r'^(E|D)(\d+)\s', '\t'.join(fields).strip())
            if entry:
                before.add((entry.group(1), int(entry.group(2))))
    if before is not None:
        for kind, number in sorted(pending - before):
            bad('待回填', f'{pending_rel} 新加了 {kind}{number}：清单只减不增，新写的实验页与决策当场登记，不进清单{base_note}')

# ── 瘦身形态与「有实验才有决策」（待回填清单之外的决策） ──
DATE = re.compile(r'20\d\d-\d\d-\d\d')
slim_items = no_experiment_items = 0
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    where = info['path']
    # 半定 / 待定的决策，doc-decisions.sh 要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+\s*[项条]未定）)?\s*$', info['title']):
        bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
    for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
        if DATE.search(heading):
            bad('瘦身形态', f'{where}：D{decision} {kind} {item_number} 的标题带日期，日期与来历进变更史')
        if kind != '已定项':
            continue
        slim_items += 1
        for label in ('定案', '射程', '依据', '欠'):
            if label not in labels:
                bad('瘦身形态', f'{where}：D{decision} 已定项 {item_number} 缺「**{label}**：」一块')
        if '依据' in labels and not cited:
            if re.search(r'无实验：\s*\S.{7,}', basis_text):
                no_experiment_items += 1
            else:
                bad('没有实验', f'{where}：D{decision} 已定项 {item_number} 的依据一个实验都没引——有实验才有决策；实在没有可量的，写「无实验：理由」')
    for kind, item_number, cell in info['index_cells']:
        # 定案格里这几样规范标记不算字数、不判日期：「**状态：…**」是 doc-decisions.sh 要的检索标记，「**进度：…**」是行末三选一的进度标记
        # （.claude/rules/format-evolution.md「决策正文只写现状」）；「改新池新建文件的字节：…」「动不动格式：…」是 doc-decisions.sh 的
        # blocking-verdict 格要写在登记行里、必须带日期的两句。弄坏开关 progress-mark-counted 下「**进度：…**」照算字数
        without_marks = cell
        for mark in ('改新池新建文件的字节', '动不动格式'):
            without_marks = re.sub(mark + r'：[^（(]*[（(][^）)]*[）)]', '', without_marks)
        counted_marks = r'\*\*状态：[^*]*\*\*' if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'progress-mark-counted' else r'\*\*(状态|进度)：[^*]*\*\*'
        plain = re.sub(r'\*\*|`', '', re.sub(counted_marks, '', without_marks)).strip()
        if DATE.search(plain) or len(plain) > 100:
            bad('瘦身形态', f'{where}：D{decision} {kind}索引表第 {item_number} 行的定案格要一句话、不带日期、不超过 100 字（现在 {len(plain)} 字）')

# ── 实验最近一次变动的日期：页内历史节 + experiments-history.md ──
central_dates = {}
central_path = os.path.join(kb, 'experiments-history.md')
if os.path.exists(central_path):
    lines = read(central_path).split('\n')
    start = history_start(lines)
    current_date, heading_refs, body_refs = None, set(), set()

    def flush():
        refs = heading_refs or body_refs
        for experiment in refs:
            if current_date and current_date > central_dates.get(experiment, ''):
                central_dates[experiment] = current_date

    for line in lines[start:]:
        dated = DATE_HEAD.match(line)
        if dated or line.startswith('### ') or line.startswith('## '):
            flush()
            current_date = dated.group(1) if dated else None
            heading_refs = {int(e) for e in E_REF.findall(line)} if dated else set()
            body_refs = set()
            continue
        body_refs |= {int(e) for e in E_REF.findall(line)}
    flush()

# ── 实验页 ──
experiments, rows_total = {}, 0
for path in item_ref_status.originals_only(sorted(glob.glob(os.path.join(kb, 'experiments', '*.md'))), read_translations):
    rel = os.path.relpath(path, root)
    lines = read(path).split('\n')
    head_line = next((line for line in lines if re.match(r'^## E(\d+) ', line)), None)
    if not head_line:
        # 认不出标题的实验页不许静默跳过：跳过就不计数、不列名，它的影响的决策表一行都没判
        bad('认不出标题', f'{rel}：没有一行写成「## E<号> 简称 —— 状态」（编号后面要跟空格），这一页一条都没判')
        continue
    number = int(re.match(r'^## E(\d+) ', head_line).group(1))
    # 只认「结论……作废 / 退役」：标题里别处出现这两个词（「上界 A 作废」「容器退役记录」）的实验结论照样成立，照判
    voided = re.search(r'结论[^，。：]{0,3}(作废|退役)', head_line) is not None
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    rows = table_rows(lines, section[0], section[1]) if section else None
    outside = [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]
    mentioned = {int(d) for d in D_REF.findall('\n'.join(outside))}
    local_dates = [m.group(1) for m in (DATE_HEAD.match(line) for line in lines[hist:]) if m]
    latest = max(local_dates + ([central_dates[number]] if number in central_dates else []), default='')
    is_pending = ('E', number) in pending
    experiments[number] = {'rel': rel, 'lines': lines, 'hist': hist, 'section': section, 'rows': rows or []}
    if rows is None and not is_pending:
        bad('没有表', f'{rel}：E{number} 没有「### 影响的决策」表（表头 | 决策分项 | 关系 | 回看 |）')
        continue
    covered = set()
    for index, cells in rows or []:
        rows_total += 1
        where = f'{rel}:{index + 1}'
        if len(cells) != 3:
            bad('行形状', f'{where}：影响的决策表一行要三格，这一行 {len(cells)} 格')
            continue
        ref = parse_decision_ref(cells[0])
        if not ref:
            bad('行形状', f'{where}：第一格要写「D<号>（简称）」或「D<号>（简称） 已定项 k」——「{cells[0][:30]}」')
            continue
        decision, item = ref
        covered.add(decision)
        if decision not in decisions:
            bad('指不到', f'{where}：D{decision} 在 decisions/ 下没有正文')
            continue
        if item and item not in decisions[decision]['items']:
            bad('指不到', f'{where}：D{decision} 没有{item[0]} {item[1]}')
        relation = cells[1].strip('* ')
        if relation not in RELATIONS:
            bad('行形状', f'{where}：关系写「{relation[:12]}」，只认 支撑 / 推翻 / 备料 / 不影响')
            continue
        if relation in BASIS_RELATIONS and item is None and decisions[decision]['items']:
            bad('行形状', f'{where}：关系是{relation}就要写到分项（D{decision} 有分项）')
        review = REVIEW.match(cells[2].strip('* '))
        if not review:
            bad('行形状', f'{where}：回看格写「YYYY-MM-DD 改了」或「YYYY-MM-DD 不受影响：理由」——「{cells[2][:30]}」')
            continue
        if latest and review.group(1) < latest:
            bad('回看过期', f'{where}：E{number} 最近一次变动在 {latest}，对 D{decision} 的回看停在 {review.group(1)}')
        if relation in BASIS_RELATIONS and item and ('D', decision) not in pending \
                and number not in decisions[decision]['basis'].get(item, set()):
            bad('不对称', f'{where}：E{number} 说它{relation} D{decision} {item[0]} {item[1]}，而那条分项的「**依据**」段没引 E{number}')
    if not is_pending and not voided and rows is not None \
            and not any(len(cells) == 3 and cells[1].strip('* ') in ('支撑', '推翻', '备料') for _, cells in rows):
        bad('没对应决策', f'{rel}：E{number} 的影响的决策表里没有一行是支撑 / 推翻 / 备料——实验必须对应决策（结论作废或退役的实验在标题里写明，就不判这一条）')
    if not is_pending:
        for decision in sorted(mentioned - covered):
            bad('没登记', f'{rel}：E{number} 正文提到 D{decision}，影响的决策表里没有它那一行')

# ── 决策这一侧：依据引的实验，实验页里要有支撑或推翻那一行 ──
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    for item, cited in sorted(info['basis'].items()):
        for experiment in sorted(cited):
            page = experiments.get(experiment)
            if not page:
                bad('指不到', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，kb 里没有这个实验页")
                continue
            ok = any(parse_decision_ref(cells[0]) == (decision, item) and cells[1].strip('* ') in BASIS_RELATIONS
                     for _, cells in page['rows'] if len(cells) == 3)
            if not ok:
                bad('不对称', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，{page['rel']} 的影响的决策表里没有「支撑 / 推翻」这一行")
for kind, number in sorted(pending):
    exists = number in (experiments if kind == 'E' else decisions)
    if not exists:
        bad('待回填', f'{pending_rel} 里的 {kind}{number} 在 kb 里没有正文')

# ── 按这次改动 ──
triggered = 0
formatting_only = 0
language_switch_only = 0
became_ran_checked = 0


def added_lines(rel):
    """这次改动里 rel 新加的行：[(新文件里的行号, 内容)]；未跟踪的文件整份都算。"""
    if git('ls-files', '--error-unmatch', rel).returncode != 0:
        return [(index, line) for index, line in enumerate(read(os.path.join(root, rel)).split('\n'))]
    shown = git('diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', rel)
    if shown.returncode != 0:
        # git 失败时取不到新增行：当成「一行没改」就会把 ⑤ 整条放过去
        bad('取不到改动', f'{rel}：git diff 相对基准 {base} 失败（退 {shown.returncode}：{shown.stderr.strip()[:80]}），这一页新增了哪些行判不了，⑤ 对它没判')
        return []
    diff = shown.stdout
    result, new_line, in_header = [], 0, True
    for line in diff.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', line)
        if hunk:
            in_header = False
            new_line = int(hunk.group(1)) - 1
            count = int(hunk.group(2)) if hunk.group(2) is not None else 1
            if count == 0:
                result.append((new_line, None))   # 纯删除：记它在新文件里的位置
            continue
        # 「+++ 」只在第一个 @@ 之前是文件头：正文里以「++ 」开头的一行新增在 diff 里也长成「+++ 」
        if line.startswith('+') and not in_header:
            result.append((new_line, line[1:]))
            new_line += 1
    return result


def ran_status(title_line):
    """标题的状态段（「——」之后、括注之前）恰好是「已跑」：部分已跑、已跑但作废都不算。"""
    status = re.match(r'^## E\d+ .*?——\s*(.*)$', title_line or '')
    return bool(status) and re.sub(r'（.*$', '', status.group(1)).strip() == '已跑'


def base_title(rel, number=None):
    """基准那一版的标题行；基准里没有这一页返回 None。"""
    shown = git('show', f'{base}:{rel}')
    # 基准里这个路径没有页（这一批搬了家）：按实验号在基准那棵树里找它原来那页的标题，搬家不算「改成已跑」
    if shown.returncode != 0 and number is not None:
        found = git('grep', '-h', '-m1', '-E', f'^## E{number} ', base, '--', '.claude/kb/experiments/')
        hit = next((l[l.find('## E'):] for l in found.stdout.split('\n') if '## E' in l), None)
        return hit if found.returncode == 0 else None
    if shown.returncode != 0:
        return None
    return next((line for line in shown.stdout.split('\n') if re.match(r'^## E\d+ ', line)), None)


def baseline_page_text(rel):
    """基准那一版实验页的全文，照此刻生效的搬迁登记把旧前缀换成新前缀（lib-lag-baseline.py 的 baseline_text：只按路径前缀换，
    提交引用后面的不换，页自己搬了家就按搬迁之前的路径取）；基准里没有这一页交 None，git 取不到或登记表写坏了报一条并交 None（按正文改了判）。
    弄坏开关 moves-not-rewritten 退回不换算（改前：git show 原样读）。"""
    if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'moves-not-rewritten':
        shown = git('show', f'{base}:{rel}')
        return shown.stdout if shown.returncode == 0 else None
    text, problem = lag_baseline.baseline_text(base, rel, cwd=root)
    if problem is not None:
        bad('取不到基准', f'{problem.sentence()}，{rel} 是不是只差语言切换行或日期整理判不了，按正文改了判')
    return text


_consolidate = None


def consolidate_module():
    """scripts/kb/doc-consolidate.py：读不到就当没有这一层判法（改动照样算正文改了）。"""
    global _consolidate
    if _consolidate is None:
        path = os.environ.get('DOC_CONSOLIDATE', '')
        if not os.path.isfile(path):
            _consolidate = False
        else:
            spec = importlib.util.spec_from_file_location('doc_consolidate', path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            _consolidate = module
    return _consolidate


def normalized_body_lines(text):
    """整理之后的正文行（去空白与列表、标题、表格的符号）按行排序：两版一样就是只删了同日日期、只并了同日小节。"""
    module = consolidate_module()
    body, _hist = module.split_history(text.split('\n'))
    migrated, _report = module.migrate_document('\n'.join(body), '', drop_history=False)
    return sorted(stripped for stripped in (re.sub(r'[\s#*|>\-]', '', line) for line in migrated.split('\n')) if stripped)


def formatting_only_change(rel):
    """这次对实验页的改动是不是只删了与所属标题相同的日期、只把同一天的第二个小节并进第一个。"""
    if not consolidate_module():
        return False
    before = baseline_page_text(rel)
    if before is None:
        return False
    try:
        return normalized_body_lines(before) == normalized_body_lines(read(os.path.join(root, rel)))
    except Exception:  # noqa: BLE001 —— 判法自己出错时按「正文改了」判，不放过
        return False


# 顶上第三行的语言切换行：.claude/skills/trilingual/SKILL.md「改一组的做法」第 3 条的三种写法（中文、英语、日语），链接目标照它们的后缀认
_SWITCH_TARGET = r'\]\([^()\s]+%s\)'
LANGUAGE_SWITCH = re.compile(
    r'\*\*中文\*\* · \[English' + _SWITCH_TARGET % r'\.en\.md' + r' · \[日本語' + _SWITCH_TARGET % r'\.ja\.md'
    + r'|\[中文' + _SWITCH_TARGET % r'\.md' + r' · \*\*English\*\* · \[日本語' + _SWITCH_TARGET % r'\.ja\.md'
    + r'|\[中文' + _SWITCH_TARGET % r'\.md' + r' · \[English' + _SWITCH_TARGET % r'\.en\.md' + r' · \*\*日本語\*\*')


def body_without_language_switch(lines):
    """历史版本之前、影响的决策一节之外的正文行；顶上第三行是认得出的语言切换行时先去掉它，连同紧跟它的那一个空行
    （旧版没有切换行、新版在第三行插进切换行加一个空行的，两版这样去掉之后逐字相同）；没有切换行的什么都不去，认不出的照旧算正文。
    弄坏开关 language-switch-counted 退回不去掉（改前：切换行算正文）；inserted-switch-counted 退回只去掉切换行、留着紧跟的空行
    （改前：只认两版第三行都是切换行、只是改了）。"""
    breaking = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')
    if breaking != 'language-switch-counted' and len(lines) > 2 and LANGUAGE_SWITCH.fullmatch(lines[2].rstrip()):
        trailing_blank = breaking != 'inserted-switch-counted' and len(lines) > 3 and not lines[3].strip()
        lines = lines[:2] + lines[4 if trailing_blank else 3:]
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    return [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]


def language_switch_only_change(rel):
    """这次对实验页的改动在正文里是不是只改了、或只新插了语言切换行（两版各自去掉它与紧跟的空行之后正文逐字相同）；基准里取不到这一页的按正文改了判。"""
    before = baseline_page_text(rel)
    if before is None:
        return False
    return body_without_language_switch(before.split('\n')) == body_without_language_switch(read(os.path.join(root, rel)).split('\n'))


def check_changed_rows(rel, row_lines):
    for text in row_lines:
        cells = split_cells(text)
        if len(cells) != 3:
            continue
        ref = parse_decision_ref(cells[0])
        review = REVIEW.match(cells[2].strip('* '))
        if ref and review and review.group(2) == '改了' and ref[0] in decisions \
                and decisions[ref[0]]['path'] not in changed:
            bad('改了没改', f"{rel}：这次写了「改了」D{ref[0]}，而 {decisions[ref[0]]['path']} 不在这次改动里")


if in_git:
    # 「产物变了」只认这次改动里新增或改写的产物：删掉的（树上已经不在）不算——归档按规则删上一轮的产物，
    # 不是实验的结论变了，不要求回看决策表（research/ 整个可删，.claude/rules/directory-layout.md）。
    # 判法：改动范围里的路径在被判的树上还在才算；改名的旧路径不在、新路径在，照新增算。
    count_deleted_products = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'deleted-products-counted'
    result_hits = {}
    for rel in sorted(changed):
        match = re.match(r'^research/results/e(\d+)[^0-9]', rel)
        if match and (count_deleted_products or os.path.lexists(os.path.join(root, rel))):
            result_hits.setdefault(int(match.group(1)), rel)
    for number, page in sorted(experiments.items()):
        rel = page['rel']
        body_changed = False
        added = added_lines(rel) if rel in changed and os.path.exists(os.path.join(root, rel)) else []
        section = page['section']
        for line_number, _ in added:
            inside_table = section and section[0] <= line_number < section[1]
            if line_number < page['hist'] and not inside_table:
                body_changed = True
        if body_changed and language_switch_only_change(rel):
            body_changed = False
            language_switch_only += 1
        if body_changed and formatting_only_change(rel):
            body_changed = False
            formatting_only += 1
        table_touched = [text for line_number, text in added
                         if text is not None and section and section[0] <= line_number < section[1]
                         and text.strip().startswith('|') and not TABLE_HEAD.match(text.strip())
                         and not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', text.strip())]
        check_changed_rows(rel, table_touched)
        reasons = []
        if body_changed:
            reasons.append('正文改了')
        if number in result_hits:
            reasons.append(f'产物 {result_hits[number]} 变了')
        if reasons:
            triggered += 1
            if not table_touched:
                bad('没回看', f"{rel}：这次改动里 E{number} {'、'.join(reasons)}，影响的决策表一行都没回看")
        # ⑤ 的第二半：这一批把实验改成已跑，正文里引了它的每条决策都要回看——决策文件在这一批里，
        # 或者影响表里那条决策的一行在这一批里改过。只看「表里任一行被改过」放不过「决策在欠账块里等它、表里却没它」那一种。
        title_line = next((line for line in page['lines'] if re.match(r'^## E\d+ ', line)), None)
        if body_changed and ran_status(title_line) and not ran_status(base_title(rel, number)):
            reviewed = {ref[0] for ref in (parse_decision_ref(split_cells(text)[0]) for text in table_touched) if ref}
            for decision, info in sorted(decisions.items()):
                if number not in info['mentions'] or ('D', decision) in pending:
                    continue
                became_ran_checked += 1
                # 决策文件在这一批里还不够：要这一批在那份决策里新增或改写的行点到了这个实验号（改一处别的不算回看）
                touched_mention = info['path'] in changed and any(
                    text and re.search(r'(?<![A-Za-z0-9])E%d(?![0-9])' % number, text) for _, text in added_lines(info['path']))
                if not touched_mention and decision not in reviewed:
                    bad('没回看引用它的决策', f"{rel}：这次改动把 E{number} 改成已跑，{info['path']} 的正文引了它，"
                        f"而这次改动在那份决策里没有一行点到它、影响的决策表里 D{decision} 那一行也没回看")
    for rel in sorted(changed):
        if not re.match(r'^research/prompts/[^/]+-main-verification\.md$', rel) or not os.path.exists(os.path.join(root, rel)):
            continue
        if git('cat-file', '-e', f'{base}:{rel}').returncode == 0:
            continue
        triggered += 1
        lines = read(os.path.join(root, rel)).split('\n')
        section = section_range(lines, r'^## 回看决策\s*$', 2)
        if not section:
            bad('没回看', f'{rel}：新写的三方判决没有「## 回看决策」一节')
            continue
        rows = table_rows(lines, section[0], section[1])
        waived = any(re.match(r'^不涉及决策：\s*\S.{3,}', line.strip()) for line in lines[section[0] + 1:section[1]])
        if not rows and not waived:
            bad('没回看', f'{rel}：「## 回看决策」里既没有表行，也没有一行「不涉及决策：理由」')
            continue
        for index, cells in rows or []:
            ref = parse_decision_ref(cells[0]) if len(cells) == 3 else None
            if not ref or not REVIEW.match(cells[2].strip('* ')):
                bad('行形状', f'{rel}:{index + 1}：回看决策表一行写成 | D<号>（简称） [已定项 k] | 关系 | YYYY-MM-DD 改了 / 不受影响：理由 |')
        check_changed_rows(rel, [lines[index] for index, _ in rows or []])

pending_experiments = sum(1 for kind, _ in pending if kind == 'E')
pending_decisions = sum(1 for kind, _ in pending if kind == 'D')
summary = (f'实验页 {len(experiments)} 个（待回填 {pending_experiments}）、决策 {len(decisions)} 条（待回填 {pending_decisions}）、'
           f'已瘦身决策的已定项 {slim_items} 个（其中写「无实验」的 {no_experiment_items} 个）、'
           f'表行 {rows_total} 行、这次改动触发回看 {triggered} 处、改成已跑而引了它的决策 {became_ran_checked} 条；'
           f'只删同日日期、只并同日小节的改动 {formatting_only} 份不算正文改了；只改语言切换行的改动 {language_switch_only} 份不算正文改了')
if problems:
    kinds = {}
    for kind, _ in problems:
        kinds[kind] = kinds.get(kind, 0) + 1
    print(f"  ✗ 决策与实验的登记有 {len(problems)} 处问题（{'、'.join(f'{k} {v}' for k, v in kinds.items())}）；{summary}：")  # gate-lint:summary
    for kind, text in problems:
        print(f'      [{kind}] {text}')  # gate-lint:detail
    print('  → 怎么办：格式与判据见 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。')
    print('            实验出了新结论、换了产物，就逐行回看它影响的决策，回看格写当天日期与「改了」或「不受影响：理由」；')
    print('            支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。')
    sys.exit(1)
print(f'  ✓ 决策与实验双向登记对得上、回看不过期（查了实验页 {len(experiments)} 个、决策 {len(decisions)} 条；{summary}）')
PY
}

# ── 格 verdict-false-named：实验产物判决行里的 false 字段有没有被实验页点名
#
# 判据：scripts/experiment-apparatus/replay.sh 登记的入库产物（在 research/results/ 下的）里，以 `E7RESULT name=verdict ` 开头的判决行，
# 每个 `字段=false` 都要在对应实验号的实验页（.claude/kb/experiments/<号>-*.md）里被点名，否则判红。不开豁免。
# 射程只认这个字面形式：源码里有 name=verdict_candidate / _base / _unit 这几种变体，没落进入库产物，出现了这一格看不见。
#
# 点名：同一物理行里字段名紧跟它自己的 =false（中间可夹反引号与空白；同一行别的字段的 false 不算），且这一行在最新那条历史条目
# （「## 历史版本」之后第一个 ### 那一节）或正文（「## 历史版本」之前）里。只认字段名出现在正文不够：正文可能是旧一轮留下的。
# 正文里那一行自己标着「第 N 次跑」、而标的次数里没有最新历史条目标题里的那一次，是上一次跑留下的，不算点名。
# 登记的产物一份都不在 research/results/ 下、或一个 false 字段都没有，退 77。
#
# 样本：verdict-false-named-red 里三个没点名的 false（一个在正文里只写字段名、一个点名的那一行标着上一次跑、一个根本没提）；
# verdict-false-named-green 里一个在最新历史条目点名、一个实验页文件名补了零；verdict-false-named-zero 的登记表一行都读不到。
# 实验页取哪一份：lib-item-ref-status.py 的 numbered_files，产物不带语言，取中文原文。没有「## 历史版本」的页照判正文（最新历史条目当空的）。
# 样本：verdict-false-named-translation-green（译文排在前面、中文原文没有历史节而正文点了名，判绿）、verdict-false-named-no-history-red
# （中文原文没有历史节也没点名、英语译文点了名，判红）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_verdict_false_named() {
  REPLAY=scripts/experiment-apparatus/replay.sh
  [[ -f "$REPLAY" ]] || { echo "  ! 找不到 $REPLAY，这一格无对象可判"; exit 77; }

  python3 - "$REPLAY" "$LIB_ITEM_REF_STATUS" "$LIB_LAG_BASELINE" <<'PY'
import glob
import importlib.util
import os
import re
import sys

replay_path = sys.argv[1]
item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)
lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', sys.argv[3])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)

# 复跑行（实验号、产物）：lib-lag-baseline.py 的 replay_rows，与 results-cited、quoted-result-lines、replay-comparison-units 三格同一份读法
rows = [(row.experiment, row.product) for row in (lag_baseline.replay_rows(replay_path) or [])]

if not rows:
    print(f"  ✗ 从 {replay_path} 一行都没读到——登记表的格式变了？")
    print("     → 怎么办：这一格靠 '编号|二进制|参数|产物|exact/timing' 这个形状取行（取法在 .claude/gate.d/lib-lag-baseline.py 的 replay_rows，")
    print("               与门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格同一个形状），改表格式要同时改这两处。")
    sys.exit(1)

RESULTS_DIR = 'research/results'
EXP_DIR = '.claude/kb/experiments'
VERDICT_PREFIX = 'E7RESULT name=verdict '
skip_named_check = os.environ.get('GATE_VERDICT_FALSE_SKIP_NAMED_CHECK') == '1'

missing_files = []      # 登记了却在 research/results/ 下找不到文件（多半是已归档，见 checker-tier-research-build-and-replay.sh 复跑里「产物已归档」那一档）
checked_files = 0
verdict_lines = 0
false_fields = []       # (exp, product, lineno, field)

for exp, product in rows:
    path = os.path.join(RESULTS_DIR, product)
    if not os.path.isfile(path):
        missing_files.append((exp, product))
        continue
    checked_files += 1
    # 压成 .gz 入库的产物（超过单文件推送上限的）解压后读，判据不变；弄坏开关 gz-read-raw：不解压，按原字节读
    with (__import__('gzip').open(path, 'rt', encoding='utf-8', errors='replace')
          if path.endswith('.gz') and os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK') != 'gz-read-raw'
          else open(path, encoding='utf-8', errors='replace')) as handle:
        for lineno, raw in enumerate(handle, 1):
            stripped = raw.strip()
            if not stripped.startswith(VERDICT_PREFIX):
                continue
            verdict_lines += 1
            rest = stripped[len(VERDICT_PREFIX):]
            for token in rest.split():
                if '=' not in token:
                    continue
                field, value = token.split('=', 1)
                if value == 'false':
                    false_fields.append((exp, product, lineno, field))


def experiment_page(exp, product):
    # 实验页文件名有的补零（08-、09-），有的不补：两种都认（numbered_files）。产物不带语言，取中文原文；
    # 改前取排序后的第一个，取到的是 <号>-<名>.en.md，那份没有「## 历史版本」，split_page 当场崩
    matches = item_ref_status.numbered_files(EXP_DIR, int(exp[1:]), os.path.join(RESULTS_DIR, product))
    return matches[0] if matches else None


def split_page(text):
    """(正文, 最新那条历史条目, 它的标题行)：没有「## 历史版本」、或历史节里一个 ### 都没有的，后两样是空串。"""
    marker = '\n## 历史版本'
    idx = text.find(marker)
    body = text if idx < 0 else text[:idx]
    if idx < 0:
        # 改前这一支只交回两样，调用方按三样拆包：没有历史节的实验页（中文原文少了这一节、或取到了译文）让整格崩在 ValueError 上
        return body, '', ''
    tail = text[idx + len(marker):]
    head_match = re.search(r'^### .*$', tail, re.M)
    if not head_match:
        return body, '', ''
    start = head_match.end()
    next_match = re.search(r'^#{2,3} ', tail[start:], re.M)
    end = start + next_match.start() if next_match else len(tail)
    return body, tail[start:end], head_match.group(0)


RUN_TAG = re.compile(r'第([一二三四五六七八九十百零〇两0-9]+)次跑')


def is_named(field, body, latest, latest_heading):
    """返回 (点没点名, 正文里那一行若是上一次跑留下的就交出它)。字段名要紧跟着它自己的 =false，同一行别的字段的 false 不算。"""
    if skip_named_check:
        return True, None
    pattern = re.compile(r'(?<![A-Za-z0-9_])' + re.escape(field) + r'`?\s*[=:：]\s*`?false(?![A-Za-z0-9_])')
    if any(pattern.search(line) for line in latest.split('\n')):
        return True, None
    latest_tags = set(RUN_TAG.findall(latest_heading))
    stale = None
    for line in body.split('\n'):
        if not pattern.search(line):
            continue
        tags = set(RUN_TAG.findall(line))
        if tags and latest_tags and not (tags & latest_tags):
            stale = stale or line.strip()
            continue
        return True, None
    return False, stale


unnamed = []   # (exp, product, lineno, field, reason)
named_count = 0
page_cache = {}
for exp, product, lineno, field in false_fields:
    if exp not in page_cache:
        page_cache[exp] = experiment_page(exp, product)
    page = page_cache[exp]
    if page is None:
        unnamed.append((exp, product, lineno, field,
                         f'找不到实验页 {EXP_DIR}/{exp[1:]}-*.md'))
        continue
    with open(page, encoding='utf-8') as handle:
        text = handle.read()
    body, latest, latest_heading = split_page(text)
    named, stale = is_named(field, body, latest, latest_heading)
    if named:
        named_count += 1
    elif stale:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 正文里点到它的那一行标的是别的某次跑（「{stale[:40]}」），'
                         f'最新历史条目是「{latest_heading.lstrip("# ").strip()[:30]}」：那是上一次跑留下的，不算点名这一次的 false'))
    else:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 里既不在最新历史条目、也不在正文里被点名'))

print(f'  · 登记了 {len(rows)} 行产物；{checked_files} 份文件在 {RESULTS_DIR} 下找到、'
      f'{len(missing_files)} 份登记了却找不到（多半已归档）')
if missing_files:
    print(f'     没查到文件的登记行（{len(missing_files)} 个）：')
    for exp, product in missing_files:
        print(f'       {exp}  {RESULTS_DIR}/{product}')  # gate-lint:detail
print(f'  · 查到的文件里共 {verdict_lines} 行判决（name=verdict）；{len(false_fields)} 个字段是 false，'
      f'其中 {named_count} 个已点名、{len(unnamed)} 个没点名')

if unnamed:
    print('  ✗ 这些判决字段是 false，对应实验页没有点名：')
    for exp, product, lineno, field, reason in unnamed:
        print(f'     {exp}  {RESULTS_DIR}/{product}:{lineno}  {field}=false  ({reason})')   # gate-lint:detail
    print('     → 怎么办：在实验页那一次跑的历史条目里写明这个字段为什么是 false——是判据如实判出')
    print('               的结论，还是装置或期望值的问题；后者照问题单重跑，把结论一起写回历史条目。')
    sys.exit(1)

if checked_files == 0:
    print(f'  ⊘ 本次无对象可判：登记的 {len(rows)} 行产物一份都不在 {RESULTS_DIR} 下（上面逐个列了）')
    sys.exit(77)
if not false_fields:
    print(f'  ⊘ 本次无对象可判：查到的 {checked_files} 份产物里 {verdict_lines} 行判决，没有一个字段是 false')
    sys.exit(77)
print(f'  ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 {named_count} 个）')
PY
}

# ── 格 repro-command：跑过的实验有没有留下复跑命令
#
# 判据：实验页点名了 research/results/ 里的产物，就要写出怎么把它跑出来：一条 `cargo run --release --bin X`，
# 或者点名复跑入口 scripts/experiment-apparatus/replay.sh / scripts/devices/vm-bench.sh；提到 mutate.sh 这类别的脚本不算，它们跑不出那份产物。
# 写明「原始输出未留存」的页不判、成功行列名。读不了的页判红（grep 出错与「没点产物」不是一回事）；一页 .md 都没有、或没有一页点产物，退 77。
# 样本：repro-command-red 里一页点了产物却只提到变异脚本；repro-command-green 里一页写了 cargo run、一页写明原始输出未留存。
cell_repro_command() {
  EXP_DIR=.claude/kb/experiments
  [[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，这一格跳过"; exit 77; }
  PRODUCT_RE='research/results/[A-Za-z0-9._-]+\.out(\.gz)?'
  # 弄坏开关 gz-name-not-product：.out.gz 不算产物名，只认以 .out 收尾的名字（点名 .gz 产物的页不再算点了产物）
  if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == gz-name-not-product ]]; then PRODUCT_RE='research/results/[A-Za-z0-9._-]+\.out([^.A-Za-z0-9_-]|$)'; fi
  REPRO_RE='cargo run --release --bin |scripts/(experiment-apparatus/replay|devices/vm-bench)\.sh'

  bad=(); unreadable=(); exempt=(); pointing=0
  for f in "$EXP_DIR"/*.md; do
    # 一页 .md 都没有时通配原样留下，grep 找不到它会退 2、被当成「读不了的页」：那是无对象可判，不是读不了
    [[ -e "$f" || -L "$f" ]] || continue
    grep -qE "$PRODUCT_RE" "$f"; rc=$?
    if ((rc == 2)); then unreadable+=("$(basename "$f")"); continue; fi
    ((rc == 0)) || continue
    pointing=$((pointing + 1))
    if grep -qF '原始输出未留存' "$f"; then exempt+=("$(basename "$f")"); continue; fi
    grep -qE "$REPRO_RE" "$f" && continue
    bad+=("$(basename "$f")")
  done

  if ((${#unreadable[@]})); then
    echo "  ✗ 这些实验页读不了，点没点产物、写没写复跑命令都没判："
    printf '      %s\n' "${unreadable[@]}"
    echo "  → 怎么办：按 grep 的报错修好（权限、编码），再跑；读不了的页不能当成没点产物放过去。"
    exit 1
  fi

  if ((${#bad[@]})); then
    echo "  ✗ 这些实验点了产物却没写复跑命令，读的人只能靠反推："
    printf '      %s\n' "${bad[@]}"
    echo "  → 怎么办：在正文的口径一节补一句，格式与别处一致——"
    echo "    代码 \`scripts/experiment-apparatus/eNN_xxx.rs\`（\`cargo run --release --bin eNN-xxx\`），"
    echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"
    echo "    多次运行拼起来的产物，要把那个循环也写出来（种子、参数各扫了哪些值）。"
    echo "    要真设备 / 虚机因而没留产物的，正文写明「原始输出未留存」，这一格就不管它。"
    echo "    复跑入口只认 cargo run --release --bin 与 scripts/experiment-apparatus/replay.sh、scripts/devices/vm-bench.sh；提到 mutate.sh 这类别的脚本不算。"
    exit 1
  fi
  if ((pointing == 0)); then
    echo "  ⊘ 本次无对象可判：$EXP_DIR 下没有一页点到 research/results/ 里的产物"
    exit 77
  fi
  echo "  ✓ 点了产物的实验都写了复跑命令（点了产物的 ${pointing} 页里判了 $((pointing - ${#exempt[@]})) 页）"
  echo "    写明「原始输出未留存」不判的 ${#exempt[@]} 页：${exempt[*]:-（没有）}"
}

# ── 格 experiment-orphans：目录登记列的各处出现过的实验号在 kb 里有没有正文
#
# 判据：目录登记 .claude/gate.d/experiment-number-directories.tsv 列的每一处（research/、scripts/、入库装置 crates/singlefs-checker-tier/src/bin/、
# kb 实验页与索引行；名字形态写在登记文件头）出现过的实验号，kb/experiments/ 里都要有对应编号的正文：跑过的东西没入库，
# 会以「我们量过」的形式活在对话里，谁也复核不了。results-cited 格按名字排除的（*local* 这类）也在这里判到。
# 读法只有一份（scripts/kb/lib_experiment_number_directories.py），scripts/kb/claim-experiment.sh 的查号读同一份登记：实验号住进新地方，登记里加一行，两处跟着认。
# 射程逐处报：登记了而不在的列进成功行；登记不在、写坏，或哪个目录读不了，判红——射程静默缩小与判过了在输出里长得一样。
# 样本：experiment-orphans-red 里 scripts/experiment-apparatus/ 与 crates/singlefs-checker-tier/src/bin/ 下各一个 kb 里没有正文的实验号；
# experiment-orphans-green 里的实验号都有正文；experiment-orphans-registry-red 的登记末尾加了一行 notes，notes/ 下的号与只登了索引行的号都判成孤儿。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-registry-ignored：不读登记、退回改前写死的四个目录，experiment-orphans-registry-red 判错。
cell_experiment_orphans() {
  EXP_DIR=.claude/kb/experiments
  REGISTRY=.claude/gate.d/experiment-number-directories.tsv
  [[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，这一格跳过"; exit 77; }

  have=$(ls "$EXP_DIR" | grep -oE '^[0-9]+' | sed 's/^0*//' | sort -un)
  listing_file="$(mktemp)"; errors_file="$(mktemp)"
  trap 'rm -f "$listing_file" "$errors_file"' EXIT
  registry_source="$REGISTRY"; registry_text=""; registry_label="目录登记 $REGISTRY"
  if [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == orphans-registry-ignored ]]; then
    registry_source="-"; registry_label="改前写死的四个目录"
    registry_text=$'research\te-number-names\t# 改前写死的那一组\nscripts/experiment-apparatus\te-number-names\t# 改前写死的那一组\nscripts/mutation/tables\te-number-names\t# 改前写死的那一组\ncrates/singlefs-checker-tier/src/bin\te-number-names\t# 改前写死的那一组'
  elif [[ ! -f "$REGISTRY" ]]; then
    echo "  ✗ 目录登记 $REGISTRY 不在：不知道实验号住在哪几处，这一格什么都没扫"
    echo "  → 怎么办：从 git 找回这份登记（git log --diff-filter=D --name-only -- $REGISTRY）；写法在它自己的文件头。"
    exit 1
  fi
  if ! python3 "$LIB_EXPERIMENT_NUMBER_DIRECTORIES" numbers "$registry_source" . <<<"$registry_text" >"$listing_file" 2>"$errors_file"; then
    echo "  ✗ 照 $registry_label 扫实验号没扫全："
    sed 's/^/      /' "$errors_file"
    echo "  → 怎么办：按上面的报错改登记（写法在 $REGISTRY 文件头）或修好读不了的目录，再跑；扫不全的目录会让孤儿实验号静默漏掉。"
    exit 1
  fi
  mapfile -t scope < <(awk -F'\t' '$1 == "entry" { print $2 }' "$listing_file")
  mapfile -t absent < <(awk -F'\t' '$1 == "absent" { print $2 }' "$listing_file")
  want=$(awk -F'\t' '$1 == "number" { print $2 }' "$listing_file" | sort -un)

  orphan=()
  for n in $want; do grep -qx "$n" <<<"$have" || orphan+=("$n"); done

  if ((${#orphan[@]})); then
    echo "  ✗ $registry_label 列的各处有这些实验号的东西，kb/experiments/ 里却没有正文："
    for n in "${orphan[@]}"; do
      printf '      %-6s' "E$n"
      awk -F'\t' -v number="$n" '$1 == "number" && $2 == number { print $3 }' "$listing_file" | head -5 | tr '\n' ' '
      echo
    done
    echo "  → 怎么办：给它建正文（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；"
    echo "    若那轮的结论不打算入库，就把 research 下那些文件删掉——留着等于留一份没人能复核的证据。"
    exit 1
  fi
  if [[ -z "$want" ]]; then
    echo "  ⊘ 本次无对象可判：$registry_label 列的 ${scope[*]} 下没有一处出现过实验号（不在的：${absent[*]:-（没有）}）"
    exit 77
  fi
  echo "  ✓ $registry_label 列的各处出现过的实验号在 kb 里都有正文（$(wc -w <<<"$want") 个）"
  echo "    扫了 ${#scope[@]} 处：${scope[*]}（target/ 不扫）；不在的 ${#absent[@]} 处：${absent[*]:-（没有）}"
}

# ── 格 quoted-result-lines：kb 与 research 正文里整行抄的产物行，产物里逐字找得到
#
# 判据：kb 正文（*-history.md 与 decisions-history/ 不算）里去掉首尾空白后以 `E7RESULT ` 开头的行，
# 必须在 research/results/*.out 的某一行里逐字出现；「## 历史版本」之后的行只判这次改动新增的（射程 3）。
# 找不到 = 抄的时候改了数，或者产物重跑之后正文没跟。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）：
#   1. 只认整行。句中反引号里的片段、按字段转述成一句话的数，一概不判。
#   2. 不按实验号配对：正文允许引别的实验的产物（E152 引 E142 的 write_list）。代价是两个实验恰好吐出同一行时，
#      引错了实验认不出来。**这次改动新增或改写的行只对照 `scripts/experiment-apparatus/replay.sh` 登记着的产物**
#      （树里还留着的旧一轮产物不算：登记的已经是新一轮，正文却新抄了旧一轮的行，就是「重跑之后正文没跟」）；
#      登记的产物已归档出了树的，去版本库历史里取它最后那一版。退回全量判时照旧对照全部产物加全部归档。
#   3. 历史节里的旧行不判：旧产物被删掉之后，历史节里留着当时抄的那一行是对的。历史节里**这次改动新增的行**照判（C438），
#      新增与否按行号认（该页相对基准的 git diff -U0，未跟踪的页整份算新增），不按文字认；退回全量判时历史节整节不判（分不出新旧）。
#      历史节里新增的行对照的产物与正文新增的行一样，只是登记着的那些；git diff 取不到时判红。
#   4. 只判这次改动新增或改写的行：正文里早先抄下的行是历史，那时的产物按归档规则删掉了，再判就是判一个不存在的对照；
#      要推翻早先的结论按 `.claude/rules/three-way-inference.md` 重跑。不在 git 仓里、或取不到新增行时退回全量判（宁可多判，不可漏判）。
# 样本：quoted-result-lines-red 里一页未跟踪的新实验页整行抄了两行在登记着的产物里找不到的产物行（一行只在没登记的旧一轮产物里有）；
# quoted-result-lines-green 里正文抄的那一行在产物里逐字找得到、历史节里的旧行不判。
# quoted-result-lines-history-added-red 是已跟踪的页：基准里历史节已有一行错引文，这次改动在历史节里又新增一行错引文，必须红并只点名新增的那一行；
# quoted-result-lines-history-old-green 同一页这次改动只在正文新抄一行对的，历史节里早先的错行不判（成功行的「没判的」里列着它）。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-added-lines-skipped 退回整节排除，history-added-red 判绿，样本判错。
# 压成 .out.gz 入库的产物（树里的、归档删出树之后只在历史里的）同一处解压再比（product_text）；quoted-result-lines-archived-gz-green / -red 是历史里只有 .out.gz 的一对，
# 开关 archived-gz-not-searched（历史里只认 .out 收尾的路径）与 gz-read-raw（不解压）各让它们判错。
# 登记了搬迁（.claude/gate.d/path-moves.tsv）时，历史节里只因搬迁才变了的行不算新增（new_line_numbers 末尾经 lib-lag-baseline.py 的 drop_moved_lines 剔，
#   与 changed-paths.sh 的 gate_added_lines 给正文剔的同一种；未跟踪的页整份也逐行过一遍）：quoted-result-lines-moved-history-green 历史节里那一行只换了路径，不判、判绿；
#   quoted-result-lines-moved-unregistered-red（没登记）、quoted-result-lines-moved-and-edited-red（登记了又多写一段）照判、判红在那一行。
#   弄坏开关 LAG_BASELINE_BREAK=moved-only-kept 下 quoted-result-lines-moved-history-green 判红。
cell_quoted_result_lines() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 .claude/kb，这一格无对象可判"; exit 77; }

  # 改动范围用这一道开头取的那一份（新增行里未跟踪的文件整份算新增）。
  if ((! changed_paths_library_loaded)); then
    echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
    echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  SCOPE_BASE=""
  if ((in_git_work_tree && added_lines_ok)); then
    SCOPE_BASE="$shared_base"
  fi
  ADDED_LINES="$SHARED_ADDED_LINES"
  export SCOPE_BASE ADDED_LINES

  python3 - "$LIB_LAG_BASELINE" "$LIB_ITEM_REF_STATUS" <<'PY'
import glob, importlib.util, os, subprocess, sys

lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', sys.argv[1])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)

item_ref_status_spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)

BREAK = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')
read_translations = BREAK == 'read-translations'

# 射程含 research/ 下的正文（scripts/experiment-apparatus/perf-by-milestone.md 也整行抄产物）；冻结证据目录不判：
# research/prompts/ 按 evidence-discipline 不许事后改，research/results/ 是被比对的一方。
import re
kb_raw = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)
                 + glob.glob('research/**/*.md', recursive=True)
            if not f.endswith('-history.md') and '/decisions-history/' not in f
            and not f.startswith('research/prompts/') and not f.startswith('research/results/'))
kb = item_ref_status.originals_only(kb_raw, read_translations)
quoted = []
history_quoted = []   # 「## 历史版本」之后整行抄的产物行：只有这次改动新增的那几行判（C438）
for path in kb:
    text = open(path, encoding='utf-8').read()
    in_history = False
    for number, line in enumerate(text.split('\n'), 1):
        if number > 1 and (line.startswith('## 历史版本') or line.startswith('## Historical Versions') or line.startswith('## 履歴バージョン')):
            in_history = True
        stripped = line.strip()
        if stripped.startswith('E7RESULT '):
            (history_quoted if in_history else quoted).append((path, number, stripped))
# 弄坏开关 history-added-lines-skipped：退回整节排除，历史节里新增的行也不判
if os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '') == 'history-added-lines-skipped':
    print('  ! 弄坏开关 history-added-lines-skipped：「## 历史版本」整节不判，历史节里这次新增的行也跳过')
    history_quoted = []


def new_line_numbers(path, base):
    """path 里这次改动（相对 base）新增的行的行号集合；未跟踪的文件整份算新增，返回 None。git 取不到就红，不当成一行没改。
    登记了搬迁时只因搬迁才变了的行不算（without_moved_lines）；未跟踪的整份也逐行过一遍，交行号集合。"""
    if subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '--error-unmatch', '--', path], capture_output=True).returncode != 0:
        moves, problems = lag_baseline.path_moves()
        if not moves and not problems:
            return None
        with open(path, encoding='utf-8', errors='surrogateescape') as handle:
            line_count = len(handle.read().split('\n'))
        return without_moved_lines(path, base, set(range(1, line_count + 1)))
    shown = subprocess.run(['git', '-c', 'core.quotepath=false', 'diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', path],
                           capture_output=True, text=True)
    if shown.returncode != 0:
        print(f'  ✗ {path}：git diff 相对基准 {base} 失败（退 {shown.returncode}：{shown.stderr.strip()[:80]}），这一页历史节里新增了哪些行判不了，不是没改')
        print('     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑。')
        sys.exit(1)
    numbers = set()
    for diff_line in shown.stdout.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', diff_line)
        if hunk:
            numbers |= set(range(int(hunk.group(1)), int(hunk.group(1)) + (int(hunk.group(2)) if hunk.group(2) is not None else 1)))
    return without_moved_lines(path, base, numbers)


def without_moved_lines(path, base, numbers):
    """只因登记的搬迁才变了的行不算新增（lib-lag-baseline.py 的 drop_moved_lines，与 changed-paths.sh 的 gate_added_lines 剔得一样）。
    搬迁登记表没有内容行时原样交回；登记表写坏了照没有登记时交回、打一行提示；git 取不到判红。"""
    moves, problems = lag_baseline.path_moves()
    if not numbers or (not moves and not problems):
        return numbers
    lines = open(path, encoding='utf-8', errors='surrogateescape').read().split('\n')
    entries = [(path, lines[number - 1], number) for number in sorted(numbers) if 0 < number <= len(lines)]
    kept, problem = lag_baseline.drop_moved_lines(base, entries)
    if isinstance(problem, lag_baseline.PathMovesProblem):
        print(f'  ! {problem.sentence()}；{path} 历史节的新增行没剔只差搬迁的')
        return numbers
    if problem is not None:
        print(f'  ✗ {path}：{problem.sentence()}，这一页历史节里哪些行只因搬迁才变了判不了')
        print('     → 怎么办：按括号里 git 的报错修好仓库状态（基准要存在、对象库没缺）再跑。')
        sys.exit(1)
    return {number for _path, _text, number in kept} | {number for number in numbers if not 0 < number <= len(lines)}


# 只判这次改动新增或改写的行；SCOPE_BASE 为空（不在 git 仓里或取不到新增行）就退回全量判（全量里历史节整节不判）。
scope = '全量'
all_quoted = list(quoted)
if os.environ.get('SCOPE_BASE', '').strip():
    added = set()
    with open(os.environ['ADDED_LINES'], encoding='utf-8', errors='replace') as added_handle:
        for record in added_handle.read().split('\n'):
            added_path, separator, added_text = record.partition('\t')
            if separator and added_text.strip().startswith('E7RESULT '):
                added.add((added_path, added_text.strip()))
    quoted = [(path, number, stripped) for path, number, stripped in quoted
              if (path, stripped) in added]
    # 历史节：按行号认这次新增的行（同一句话早先留在历史节里、正文又新抄一遍，不能按文字冒充新增）
    for history_path in sorted({path for path, _, _ in history_quoted}):
        new_numbers = new_line_numbers(history_path, os.environ['SCOPE_BASE'].strip())
        for item in (entry for entry in history_quoted if entry[0] == history_path):
            all_quoted.append(item)
            if new_numbers is None or item[1] in new_numbers:
                quoted.append(item)
    scope = '这次改动新增或改写的'

if not quoted:
    print('  ! %s kb 正文与历史节里没有整行抄的 E7RESULT 行，这一格无对象可判' % scope)
    sys.exit(77)

# 登记的产物：replay.sh 登记表每一行的第 4 列（lib-lag-baseline.py 的 replay_rows，与 results-cited、verdict-false-named、replay-comparison-units 三格同一份读法，
# 形状与 experiment-replay 格相同）
REPLAY = 'scripts/experiment-apparatus/replay.sh'
registered = None
registered_experiments = set()
if os.path.isfile(REPLAY):
    replay_rows = lag_baseline.replay_rows(REPLAY) or []
    registered = {row.product for row in replay_rows}
    registered_experiments = {row.experiment.upper() for row in replay_rows}
if scope != '全量' and not registered:
    print(f'  ✗ 读不到 {REPLAY} 的登记表（文件不在，或一行 编号|二进制|参数|产物|exact/timing 都没读到），分不出哪些产物是这一轮登记的')
    print('     → 怎么办：登记表是新抄的产物行唯一的对照；它挪了地方或改了格式，就同步改这一格（与 verdict-false-named 格、门禁 checker-tier-research-build-and-replay.sh 的 experiment-replay 格同一份形状）。')
    sys.exit(1)

BREAK = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')


def product_text(name, raw_bytes):
    """产物字节转成文本：压成 .out.gz 入库的（超过单文件推送上限的）先解压，判据不变；树里的与历史里取出来的共用这一处。
    弄坏开关 gz-read-raw：不解压，按原字节读。解压不开的（不是 gzip）当场红，不当成一行都没有。"""
    if name.endswith('.gz') and BREAK != 'gz-read-raw':
        try:
            raw_bytes = __import__('gzip').decompress(raw_bytes)
        except (OSError, EOFError) as problem:
            print(f'  ✗ {name} 以 .gz 结尾却解压不开（{problem}），里面有哪些行判不了')
            print('     → 怎么办：产物压缩要用 gzip -n -9 <文件>（别的压缩格式不认）；文件坏了就从历史取回或重跑。')
            sys.exit(1)
    return raw_bytes.decode('utf-8', errors='ignore')


products = sorted(glob.glob('research/results/*.out') + glob.glob('research/results/*.out.gz'))
product_lines = set()
registered_lines = set()
line_sources = {}
for product in products:
    base_prod = os.path.basename(product)
    is_registered = registered is not None and base_prod in registered
    m_exp = re.match(r'^e(\d+)', base_prod)
    if m_exp and ('E' + m_exp.group(1)) not in registered_experiments:
        is_registered = True
    with open(product, 'rb') as handle:
        product_content = product_text(product, handle.read())
    for line in product_content.split('\n'):
        text = line.strip().replace('\r', '')
        product_lines.add(text)
        line_sources.setdefault(text, set()).add(base_prod)
        if is_registered:
            registered_lines.add(text)

# 树里找不到就去版本库历史里找：产物按归档规则删进了 git，历史引文当时逐字是对的；少了这一段，这道检查会退化成「抓归档过的产物」。
# 引文与归档那一刻的产物字节不符照样红。
archived_count = 0
# 归档的产物可以是 .out，也可以是压成 .out.gz 入库的（取出来经 product_text 解压，与树里的同一处）；弄坏开关 archived-gz-not-searched：只认 .out 收尾的路径，历史里只有 .out.gz 的算找不到
ARCHIVED_SUFFIXES = ('.out',) if BREAK == 'archived-gz-not-searched' else ('.out', '.out.gz')


def blob_lines(entry, revision_path):
    """git show 取出的归档产物字节，按文件名解压（若压过）后逐行去首尾空白；取不出返回 None。"""
    blob = subprocess.run(['git', 'show', revision_path], capture_output=True)
    if blob.returncode != 0:
        return None
    return {one.strip().replace('\r', '') for one in product_text(entry, blob.stdout).split('\n')}


def archived_product_lines(only_names=None):
    """归档进版本库的产物的行；only_names 给了就只取这些文件名的（登记着、却已出了树的那几份）。"""
    global archived_count
    lines = set()
    log = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only',
                          '--', 'research/results'], capture_output=True, text=True).stdout
    commit = None
    for entry in log.split('\n'):
        entry = entry.strip()
        if not entry:
            continue
        if len(entry) == 40 and all(character in '0123456789abcdef' for character in entry):
            commit = entry
            continue
        if commit and entry.endswith(ARCHIVED_SUFFIXES) and (only_names is None or os.path.basename(entry) in only_names):
            archived = blob_lines(entry, '%s^:%s' % (commit, entry))
            if archived is not None:
                archived_count += 1
                lines |= archived
    # 正在归档的：HEAD 里还在、工作区里已经删了的产物（这一批照归档规则删上一轮的产物，删除还没提交，
    # git log --diff-filter=D 还找不到它）。它们的字节就在 HEAD 里，与已提交的归档是同一件事。
    in_head = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-tree', '-r', '--name-only', 'HEAD', '--', 'research/results'],
                             capture_output=True, text=True)
    for entry in in_head.stdout.split('\n') if in_head.returncode == 0 else []:
        entry = entry.strip()
        if entry.endswith(ARCHIVED_SUFFIXES) and not os.path.exists(entry) and (only_names is None or os.path.basename(entry) in only_names):
            archived = blob_lines(entry, 'HEAD:%s' % entry)
            if archived is not None:
                archived_count += 1
                lines |= archived
    return lines

if scope == '全量':
    compared_lines = product_lines
else:
    compared_lines = registered_lines
missing = [(path, number, stripped) for path, number, stripped in quoted if stripped not in compared_lines]
if missing:
    if scope == '全量':
        compared_lines |= archived_product_lines()
    else:
        on_disk = {os.path.basename(product) for product in products}
        compared_lines |= archived_product_lines(only_names=registered - on_disk)
    missing = [item for item in missing if item[2] not in compared_lines]
if missing:
    compared_what = (f'research/results/ 的 {len(products)} 份产物' if scope == '全量'
                     else f'{REPLAY} 登记着的 {len(registered)} 份产物')
    print(f'  ✗ {scope} kb 正文与历史节里有 {len(missing)} 行整行抄的产物行，在 {compared_what}里一份都找不到：')
    for path, number, stripped in missing:
        print(f'     {path}:{number}  {stripped[:160]}')  # gate-lint:detail
        unregistered = sorted(line_sources.get(stripped, ()))
        if unregistered:
            print(f'       只在没在 replay.sh 登记的产物里有（{"、".join(unregistered[:3])}）：那是旧一轮的，登记的已经换了')  # gate-lint:detail
    print('     → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；')
    print('               正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。')
    sys.exit(1)
compared_count = len(products) if scope == '全量' else len(registered)
print(f'  ✓ {scope} kb 正文与历史节里整行抄的产物行都在产物里逐字找得到（判了 {len(quoted)} 行，对照 {compared_count} 份'
      + ('产物）' if scope == '全量' else '登记着的产物）'))
judged = {(path, number) for path, number, _ in quoted}
skipped = {}
for path, number, _ in all_quoted:
    if (path, number) not in judged:
        skipped.setdefault(path, []).append(number)
skipped_count = sum(len(numbers) for numbers in skipped.values())
print(f'     没判的 {skipped_count} 行（早先抄下的，历史参考，产物按「每次提交删上一次的实验记录」删掉了；要推翻它们按三方重跑）'
      + (''.join(f'\n       {path}:{",".join(str(n) for n in numbers)}' for path, numbers in sorted(skipped.items())) if skipped else '：（没有）'))
PY
}

# ── 格 archive-past-rounds：上一轮及更早的实验记录已经归档进版本库，工作区里不留
#
# 规则：.claude/singlefs-ai-sop/rules/evidence-discipline.md「原样保存的证据不许事后改」之下那三段；判法与保留哪几类在 scripts/kb/archive-past-rounds.py。
# 基准是这一道开头取的 gate_diff_base gate（与 doc-process-records.sh 的 knowledge-sync 格同一个），经 GATE_BASE 交给脚本，出路里打出的基准就是这个提交。
# 判别力：fixtures/doc-experiments.sh/archive-past-rounds-red 是一个留着上一轮产物的小仓，必须判红；
# archive-past-rounds-green 只有本轮的，必须判绿。
cell_archive_past_rounds() {
  [[ -d "$ROOT/research/prompts" || -d "$ROOT/research/results" ]] || { echo "  ! 没有 research/prompts 与 research/results，这一格无对象可判"; exit 77; }
  # 不在 git 仓里分不出哪些是上一轮提交进来的（脚本的 git ls-files 会带着调用栈退出）：这一格无对象可判，不记通过
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出哪些记录是上一轮提交进来的，这一格无对象可判"; exit 77; }
  if ((changed_paths_library_loaded)); then
    if GATE_BASE="$shared_base" python3 "$ARCHIVE_SCRIPT" --check "$ROOT"; then
      exit 0
    fi
  elif python3 "$ARCHIVE_SCRIPT" --check "$ROOT"; then
    exit 0
  fi
  echo "  ✗ 上一轮及更早的实验记录没收干净（上面 scripts/kb/archive-past-rounds.py 逐类列了红的是哪几类：工作区里还留着的上一轮记录、登记表点名而已经不在的路径、这一趟要删而被平铺登记表点名的产物）"
  echo "     → 怎么办：还留着的上一轮记录，照上面那一行打出来的基准跑 --apply 删掉（不带基准它会按 HEAD 算、把这一次提交要带的记录也删掉），"
  echo "               它同时把别处指向这些文件的引用改成只留文件名、不留路径；删完跑共享门禁的「链接指向」阶段确认链接都还到得了。"
  echo "               登记表指空、平铺登记表点名的那几行，同一次提交里删掉或改掉那几行登记（上面每一类自己的出路写了怎么改）。"
  exit 1
}

# ── 格 multipath-registry：多条路径互证的实验，路径与共用项要登记且指得到
#
# 判据与形态在 `.claude/rules/format-evolution.md`「实验页的「路径与结论登记」」，这一格判四条：
#   ① 登记表的形状：`### 路径与结论登记` 之下恰好一张表，表头逐字
#      `| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |`，每行五格；表头下面要有分隔行，
#      数据行按「不是分隔行」认，不按位置跳过前两行（少了分隔行时第一条数据行照判）；
#   ② 源码落点指得到：写成 `路径:行号` 或 `路径:行号 函数名` 的，那个文件要在仓里现存；只活在正文里的写「无实现：理由」，理由不许空；
#   ③ 共用项那一格不许空，也不许只写「无」（去掉括号、标点与「共用项」这类字眼之后只剩「无」「没有」「N/A」「none」这一类的都算）；
#   ④ 还没写登记节的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，这张表每一行：
#      两列（页面路径、普查判定的依据）用制表符分隔、都不许空；路径形如
#      `.claude/kb/experiments/<编号>-….md` 且文件在仓里现存；那一页确实还没有登记节——补上了就判过期，删行；
#      **只缩不涨**：与基准提交比，基准里这张表没有的实验编号不许出现。按编号比、不按整条路径比：
#      页面改名、登记行抄错了题目改正，都不算新加了一份。基准里还没有滞后表时不比；git 取不到基准那一版判红（取法 lib-lag-baseline.py 的 baseline_rows）。
#   没有一页写了登记节、滞后表也一行都没有，退 77。
#
# 射程只到已经写了这一节的实验页。「该写而没写」要先认出实验在做多条路径互证，按关键词扫正文认不出来，
# 那一半欠在 C48（多条校验路径共用同一个前提）。
#
# 判别力：fixtures/doc-experiments.sh/{multipath-registry-red,multipath-registry-green}/ 各一份实验页，
# red 那份四行里一行共用项写「无」、一行写「N/A」、一行源码落点指不到文件，另一份页的表缺分隔行、第一条数据行共用项写「无」，
# 另带一张滞后表：一行指向不存在的页、
# 一行指向已经写了登记节的页、一行少了依据那一列、一行的实验编号基准里没有，必须判红；
# green 的滞后表只有一行、指向一份还没写登记节的页、与基准相同，必须判绿。
cell_multipath_registry() {
  EXPERIMENTS=.claude/kb/experiments
  LAG=.claude/gate.d/multipath-registry-lag.tsv
  [[ -d "$EXPERIMENTS" ]] || { echo "  ! 没有 $EXPERIMENTS，这一格无对象可判"; exit 77; }
  base=""
  if ((in_git_work_tree)); then
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
  fi

  python3 - "$EXPERIMENTS" "$LAG" "$base" "$LIB_LAG_BASELINE" <<'PY'
import importlib.util, os, re, sys, glob

experiments, lag_path, base = sys.argv[1], sys.argv[2], sys.argv[3]
lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', sys.argv[4])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
HEADING = '### 路径与结论登记'
WANTED = '| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |'
LAG_PAGE = re.compile(r'\.claude/kb/experiments/(?P<number>[0-9]+)-[^/]+\.md')

def has_registration(text):
    return HEADING in text

SAYS_NOTHING = ('', '无', '没有', '没', 'na', 'none', 'nil', '不适用')

def says_nothing(shared):
    """共用项那一格去掉括号、标点、空白与「共用项 / 共用 / 共享」这类字眼之后，只剩「无」这一类：等于没写。"""
    core = re.sub(r'[\s（）()\[\]【】「」`*。．.，,、;；:：/\\\-—–_]+', '', shared).lower()
    core = re.sub(r'共用项|共用|共享', '', core)
    return core in SAYS_NOTHING

problems, checked_pages, checked_rows = [], 0, 0
for path in sorted(glob.glob(os.path.join(experiments, '*.md'))):
    text = open(path, encoding='utf-8').read()
    if not has_registration(text):
        continue
    checked_pages += 1
    section = text.split(HEADING, 1)[1]
    cut = re.search(r'^#{2,4}\s', section, re.M)
    if cut:
        section = section[:cut.start()]
    rows = [line for line in section.split('\n') if line.startswith('|')]
    if not rows:
        problems.append(f"{path}：「{HEADING}」之下一张表都没有")
        continue
    if rows[0].strip() != WANTED:
        problems.append(f"{path}：表头不是 {WANTED}，实际是 {rows[0].strip()[:60]}")
        continue
    if len(rows) < 2 or not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', rows[1].strip()):
        problems.append(f"{path}：表头下面没有分隔行（|---|---|…|），表格渲染不出来；下面的数据行照判")
    for row in rows[1:]:
        if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', row.strip()):
            continue
        cells = [cell.strip() for cell in row.strip().strip('|').split('|')]
        if len(cells) != 5:
            problems.append(f"{path}：这一行不是五格：{row.strip()[:60]}")
            continue
        checked_rows += 1
        number, name, _how, where, shared = cells
        for hit in re.findall(r'`([^`:]+?):(\d+)(?:\s[^`]*)?`', where):
            if not os.path.exists(hit[0]):
                problems.append(f"{path}：第 {number} 行「{name}」的源码落点指不到文件：{hit[0]}")
        if not re.search(r'`[^`:]+?:\d+(?:\s[^`]*)?`', where) and not where.startswith('无实现：'):
            problems.append(f"{path}：第 {number} 行「{name}」的源码落点既不是 路径:行号，也不是「无实现：理由」：{where[:40]}")
        if where.startswith('无实现：') and not where[len('无实现：'):].strip():
            problems.append(f"{path}：第 {number} 行「{name}」写了「无实现：」而理由是空的")
        if says_nothing(shared):
            problems.append(f"{path}：第 {number} 行「{name}」的共用项那一格是空的或只写了「{shared}」")

# ④ 滞后表：形状、路径、过期、只缩不涨
lag_problems, lagging, lag_pages_by_number = [], [], {}
if os.path.isfile(lag_path):
    for line_number, fields in lag_baseline.content_rows(open(lag_path, encoding='utf-8').read()):
        where = f"{os.path.basename(lag_path)} 第 {line_number} 行"
        if len(fields) != 2 or not all(field.strip() for field in fields):
            lag_problems.append(f"{where}：不是两列（页面路径、普查判定的依据）或有一列是空的：{chr(9).join(fields)[:80]}")
            continue
        page = fields[0].strip()
        page_shape = LAG_PAGE.fullmatch(page)
        if not page_shape:
            lag_problems.append(f"{where}：路径不是 .claude/kb/experiments/<编号>-….md 的形状：{page}")
            continue
        lag_pages_by_number[int(page_shape.group('number'))] = page
        if not os.path.isfile(page):
            lag_problems.append(f"{where}：指向的实验页不存在：{page}")
            continue
        if has_registration(open(page, encoding='utf-8').read()):
            lag_problems.append(f"{where}：{page} 已经写了「{HEADING}」，这一行过期了")
            continue
        lagging.append(page)

baseline_numbers = None
if base:
    # 「基准里还没有这张表」与「git 取不到基准那一版」分开（lib-lag-baseline.py 的 baseline_rows）：后者判红，不当成没有基线静默不比
    baseline, baseline_problem = lag_baseline.baseline_rows(base, lag_path)
    if baseline_problem is not None:
        lag_problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
    elif baseline is not None:
        baseline_numbers = set()
        for _line_number, fields in baseline:
            page_shape = LAG_PAGE.fullmatch(fields[0].strip())
            if page_shape:
                baseline_numbers.add(int(page_shape.group('number')))
grown = sorted(set(lag_pages_by_number) - baseline_numbers) if baseline_numbers is not None else []

if problems:
    print(f"  ✗ {len(problems)} 处登记不合形态：")                       # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：形态见 .claude/rules/format-evolution.md「实验页的「路径与结论登记」」。")
    print("               表头逐字 | # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |，每行五格；")
    print("               源码落点写 `路径:行号 函数名`，只活在正文里的写「无实现：理由」；")
    print("               共用项写这条路径读了哪些常量、哪些规则、哪份规范，一个都不许省——")
    print("               多条路径逐格相等不构成证据，当它们共用同一个错误前提时。")
if lag_problems:
    print(f"  ✗ 滞后表 {lag_path} 里 {len(lag_problems)} 行对不上仓里的实验页：")  # gate-lint:summary
    for entry in lag_problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：路径抄错了就照 .claude/kb/experiments/ 里现存的文件名改；那一页已经补了登记节，就删掉这一行——")
    print("               留着的滞后行会让人以为那一页还欠着，而指向不存在的页等于那一格没人管。")
if grown:
    print(f"  ✗ 滞后表涨了：这些实验页的编号，基准 {base} 的滞后表里没有：")  # gate-lint:summary
    for number in grown:
        print(f"     {lag_pages_by_number[number]}")                     # gate-lint:detail
    print("     → 怎么办：这张表只缩不涨。新写的实验页要做多条路径互证，就当场写「### 路径与结论登记」那一节，")
    print("               不许加一行滞后把它按下去——加得了一行，这一格对新写的实验页就什么都拦不住了。")
if problems or lag_problems or grown:
    sys.exit(1)

if checked_pages == 0 and not lagging and not lag_pages_by_number:
    print(f"  ⊘ 本次无对象可判：没有一页实验写了「{HEADING}」，{os.path.basename(lag_path)} 里也一行都没有")
    sys.exit(77)
compared = f"与基准 {base} 比没涨" if baseline_numbers is not None else "基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比"
print(f"  ✓ 登记了路径与共用项的实验页判过了（{checked_pages} 份、{checked_rows} 条路径）；"
      f"还没写登记节的 {len(lagging)} 份在 {os.path.basename(lag_path)} 里、每一份都现存且确实还没写，{compared}；逐份补齐后删行")
PY
}

# ── 格 cited-paths-exist：kb 点名的实验产物、变异表、实验源码，路径要在树里或版本库历史里
#
# 这是 C43 反向那一半的存量判：results-cited 格第三道只判「这次改动新增或改写的点名行」、只判 research/results/ 下的产物，
# 存量的点名行与变异表、实验源码两类它都不看。分工：results-cited 第三道按这次改动、认裸文件名、带登记在 replay.sh 的那一道判法；
# 这一格不看改动范围，每次判 kb 全部正文，只判存在性。
#
# 扫哪些文件：.claude/kb 与 crates/singlefs-checker/invariants 下全部 .md（含历史节、含子目录），
# 再加 .claude/kb-directories 登记的别的目录；目录不在就跳过，一份 .md 都扫不到退 77。
# 认哪三类路径（整路径形态，反引号、括号、链接里都认；`../../research/results/…` 这种相对链接按仓内路径算）：
#   research/results/<名>.out（名里可带子目录）、scripts/mutation/tables/<名>.tsv、scripts/experiment-apparatus/<名>.rs。
# 光写文件名不带目录的不认（那是 results-cited 第二道的事）；带通配符、占位符（`e17-*.out`、`<名>.out`）或写成区间（`c113-r1..r6-local.out`）的不是一个具体路径，不认。
# 每个路径去重后判一次：工作区里在、或 git ls-files 里在 → 树里；不在 → git log --all --diff-filter=A 找得到加入过 → 只在历史里；都没有 → 红。
# 不在 git 仓里没有历史可查，树里没有的一律判红。
#
# 判别力：fixtures/doc-experiments.sh/cited-paths-exist-red 一页点名一个树里与历史里都没有的产物 e9-missing.out、
# 一张不存在的变异表、一个只在写法上带区间的名字（不该被认）和一个存在的实验源码，必须判红且只点名那两处；
# cited-paths-exist-green 点名三类各一个在树里的、另一个产物先提交后删掉（只在历史里），必须判绿并报「只在历史里 1 个」。
# 产物压成 <名>.out.gz 入库之后，点名 <名>.out 而树里、历史里只有 <名>.out.gz 的也算找得到：存放名由 results-cited 格的 product_stored_forms 列
# （export -f 递给 python，不抄第二份）；变异表与实验源码只认原样。cited-paths-exist-gz-green 绿、cited-paths-exist-gz-red 红；
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=cited-path-gz-unmatched：只查原样的名字，gz-green 判错。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-lookup-skipped：历史里找得到也算缺，green 判红；
# existence-unchecked：一律当在，red 判绿。
cell_cited_paths_exist() {
  export -f product_stored_forms   # python 里查存放名要调它（results-cited 格同一份）
  python3 - "$ROOT" <<'PY'
import os, re, subprocess, sys

root = sys.argv[1]
break_switch = os.environ.get('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', '')
os.chdir(root)

NAME = r'[A-Za-z0-9_+-]+(?:\.[A-Za-z0-9_+-]+)*'
KINDS = [
    ('research/results 下的产物', r'research/results/(?:' + NAME + r'/)*' + NAME + r'\.out'),
    ('变异表', r'scripts/mutation/tables/(?:' + NAME + r'/)*' + NAME + r'\.tsv'),
    ('实验源码', r'scripts/experiment-apparatus/(?:' + NAME + r'/)*' + NAME + r'\.rs'),
]
cited_path = re.compile(r'(?<![A-Za-z0-9_-])(' + '|'.join(f'(?P<k{i}>{pattern})' for i, (_label, pattern) in enumerate(KINDS))
                        + r')(?![A-Za-z0-9_+-]|\.[A-Za-z0-9])')

directories = ['.claude/kb', 'crates/singlefs-checker/invariants']
if os.path.isfile('.claude/kb-directories'):
    with open('.claude/kb-directories', encoding='utf-8') as registry:
        for line in registry:
            entry = line.split('#', 1)[0].strip()
            if entry and entry not in directories:
                directories.append(entry)

pages = []
for directory in directories:
    for walked, _names, files in os.walk(directory):
        pages.extend(os.path.join(walked, name) for name in files if name.endswith('.md'))
pages = sorted(set(pages))
if not pages:
    print(f"  ! 没有可扫的 .md（{', '.join(directories)} 下一份都没有），这一格无对象可判")
    sys.exit(77)

places = {}      # 路径 -> [(页, 行号)]
kind_of = {}
# 哪一行写明了原件没留存（出路里给的那种写法），那一行点名的路径就是如实交代过的：kb 里别处再点名它，也不算指空
unretained_declared = set()
if break_switch != 'unretained-note-ignored':
    for page in pages:
        with open(page, encoding='utf-8', errors='replace') as handle:
            for line in handle:
                if '原件未留存' in line or '原始输出未留存' in line:
                    unretained_declared.update(found.group(1) for found in cited_path.finditer(line))
unretained_noted = 0
for page in pages:
    with open(page, encoding='utf-8', errors='replace') as handle:
        for number, line in enumerate(handle, 1):
            for found in cited_path.finditer(line):
                path = found.group(1)
                if path in unretained_declared:
                    unretained_noted += 1
                    continue
                places.setdefault(path, []).append((page, number))
                kind_of[path] = next(i for i in range(len(KINDS)) if found.group(f'k{i}'))

in_git = subprocess.run(['git', 'rev-parse', '--is-inside-work-tree'], capture_output=True, text=True).returncode == 0
tracked = set()
if in_git:
    listed = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '-z'], capture_output=True)
    if listed.returncode != 0:
        print(f"  ✗ git ls-files 退 {listed.returncode}（{listed.stderr.decode('utf-8', 'replace').strip()[:80]}），树里有没有没判成")   # gate-lint:summary
        print("     → 怎么办：先让 git 在这个仓里能正常跑（索引锁、仓损坏），再重跑这一格；不许把没判成当成都在。")   # gate-lint:detail
        sys.exit(1)
    tracked = set(listed.stdout.decode('utf-8', 'replace').split('\0'))

def stored_forms(path):
    """点名的路径在树里、历史里可能的存放名：.out 产物压成 .out.gz 入库之后只剩后者。
    判法是 results-cited 格的 product_stored_forms（同一份，bash 函数经 export -f 递过来），这里不抄第二份；其余两类路径只认原样。"""
    if kind_of[path] != 0 or break_switch == 'cited-path-gz-unmatched':
        return [path]
    listed_forms = subprocess.run(['bash', '-c', 'product_stored_forms "$1"', '_', path], capture_output=True, text=True)
    forms = [form for form in listed_forms.stdout.split('\n') if form]
    if listed_forms.returncode != 0 or path not in forms:
        print(f"  ✗ product_stored_forms 查 {path} 的存放名退 {listed_forms.returncode}（{listed_forms.stderr.strip()[:80]}），这一个没判成")   # gate-lint:summary
        print("     → 怎么办：它是 doc-experiments.sh 里 results-cited 格用的同一个函数，看这一份脚本有没有被改坏；不许把没判成当成在。")   # gate-lint:detail
        sys.exit(1)
    return forms


in_tree, only_history, absent = [], [], []
for path in sorted(places):
    forms = stored_forms(path)
    if break_switch != 'existence-unchecked' and not any(os.path.isfile(form) or form in tracked for form in forms):
        found_in_history = False
        if in_git and break_switch != 'history-lookup-skipped':
            for form in forms:
                logged = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=A', '-n1', '--format=%H', '--', form],
                                        capture_output=True, text=True)
                if logged.returncode != 0:
                    print(f"  ✗ git log 查 {form} 的历史退 {logged.returncode}（{logged.stderr.strip()[:80]}），这一个没判成")   # gate-lint:summary
                    print("     → 怎么办：先让 git 在这个仓里能正常跑，再重跑这一格；不许把没判成当成历史里有。")   # gate-lint:detail
                    sys.exit(1)
                if logged.stdout.strip():
                    found_in_history = True
                    break
        (only_history if found_in_history else absent).append(path)
    else:
        in_tree.append(path)

if absent:
    reference_count = sum(len(places[path]) for path in absent)
    print(f"  ✗ kb 点名的 {len(absent)} 个路径（{reference_count} 处引用）树里没有、版本库历史里也没有：")   # gate-lint:summary
    for path in absent:
        shown = '、'.join(f'{page}:{number}' for page, number in places[path][:3])
        more = f" 等 {len(places[path])} 处" if len(places[path]) > 3 else ''
        print(f"     {shown}{more} 点名 {path}（{KINDS[kind_of[path]][0]}）")   # gate-lint:detail
    print("     → 怎么办：补回那份文件；或把那一行改成写明「原件未留存、从哪次提交取」；或改成现存的路径")
    print("               （移过位置的在 git log --all --diff-filter=A --name-only -- '*<文件名>' 里找新路径）。")
    print("               只写文件名不写目录的不在这一格的射程里，改成不带目录的写法不算修好——取不回来的就如实写原件未留存。")
    sys.exit(1)

by_kind = [sum(1 for path in places if kind_of[path] == i) for i in range(len(KINDS))]
reference_total = sum(len(spots) for spots in places.values())
if not places:
    print(f"  ! 本次无对象可判：扫了 {len(pages)} 份 .md，一个整路径形态的产物、变异表、实验源码路径都没有")
    sys.exit(77)
kinds_text = '、'.join(f'{KINDS[i][0]} {by_kind[i]}' for i in range(len(KINDS)))
print(f"  ✓ {len(pages)} 份 .md 点名的 {len(places)} 个路径（{reference_total} 处引用；{kinds_text}）都找得到：树里 {len(in_tree)} 个、只在历史里 {len(only_history)} 个")
if only_history:
    print("     只在历史里的（树里没有，git log --all --diff-filter=A 找得到它加入过）：")
    for path in only_history:
        print(f"       {path}")
if unretained_noted:
    print(f"     写明了原件未留存的 {len(unretained_declared)} 个路径，点名 {unretained_noted} 处不判")
PY
}

# ── 格 preregistration-parallelism：E186 起的跑前登记有「并行与挂钟」一节，五样写全，预计挂钟与态数乘单态耗时对得上
# 判据（.claude/agents/experiment-designer.md「登记的固定节名」的「## 十四、并行与挂钟」一行与第 3a 步「现算态数或输入规模与单态耗时……相乘得挂钟」）：这次改动新建或改动的
#   research/prompts/e<号>-preregistration.md，号不小于 PREREGISTRATION_PARALLELISM_FIRST_NUMBER（186）的，要有一节标题是「## <汉字序号>、并行与挂钟」，
#   节里（到下一个「## 」标题为止，围栏代码块里的行不算）五个标签行各至少一行、冒号后至少 8 个字：
#     独立单位：哪些单位互相独立            线程：分几条线程、取哪个登记值
#     串行：哪几步因计时或共享状态必须串行与为什么（没有就写「无」并写为什么都能并行）
#     预计挂钟：各段预计挂钟（推的写依据）  预计在用核数：各段预计在用几个核（推的写依据）
#   标签行可带列表记号（- 或 *）与加粗（**…**），冒号全角半角都认；「预计挂钟」「预计在用核数」两行还要带一个数字。
# 挂钟算式（判「预计挂钟：」那一行自己写的态数乘单态耗时，读法在下面的 wallclock_claims）：那一行按「；」「;」「。」切成段，每段认两种写法之一：
#   式子：「<数>[量词] × … × <数><时间单位> [÷ <数>] ≈|=|约 <挂钟>」，乘数里恰好一个带时间单位（单态耗时），其余是态数；算出的是乘积除以写的除数。
#   每：段里没有式子时，「每<量词>」后 40 个字以内的「<数><时间单位>」是单态耗时，它前面最近的、量词与「每」后那串字开头相同的「<数><量词>」是态数
#     （「<数><量词> × <数><量词> … [= <数><量词>]」算一个态数，有等号取等号右边）；一段里几对的乘积相加，与这一段最后一对之后最后写的「<数><时间单位>」比。
#   时间单位认 纳秒 微秒 毫秒 秒 分钟 小时 天 与 ns µs us ms s min h；数可带千分位逗号、小数、e 指数、万或亿；「10–25 分钟」这种写法当一个区间。
#   算出的与写的差一个数量级以上（写的上限乘 10 还不到算出的下限，或写的下限超过算出的上限乘 10）判红，点名那一段与怎么算的。
#   一段都认不出的登记（写不出态数或单态耗时，或写了而没有挂钟可比）报「本次未判」并点名，往 $GATE_NOT_RUN_FILE 报一行，不算绿。
# 态数与第六节（「## 六、」那一节，围栏里的不算）对（quantities_section_judgement）：上面认出的每一条用到的态数（式子里不带时间单位的乘数、「每」配上的那个「<数><量词>」），
#   在第六节里按同一份读法（segment_counts）找「<数><量词>」，前面紧挨「第」的是序号、不算：有同值而量词互为开头的算对上；没有同值、有量词逐字相同的算对不上，判红，
#   点名两边的原文与第六节的行号；第六节里没有同值也没有同量词的、态数没写量词的（「960 × 10000 × …」）算认不出，报「态数与第六节的比对本次未判」并点名，
#   往 $GATE_NOT_RUN_FILE 报一行，不算绿，也不判红。
# 起判：号不小于 186 的才判，E185 及以前的登记不回补这一节（不设排除表：号以下的一律不判，号以上的没有例外）。
#   重跑登记（e<号>-r<n>-prereg.md）不判：差分重跑照设计员定义只写三样、其余「同上一次登记」，这一节跟着原登记走。
#   这次改动里一份要判的都没有：退 77（无对象可判），成功句与无对象句逐个列出号小于 186 不判的那几份。
# 管不到：写的数对不对、装置是不是照它实现、串行的理由成不成立——靠三方与运行时的核数告警（scripts/hosts/core_usage.py）；
#   单态耗时只读「预计挂钟：」那一行，写在第六节或别的行、这一行里没重写的不读；态数只与第六节比，第九节（变异条数）这类别的节里写的不比；
#   第六节里同一个量词写着几个数的，有一个同值就算对上；「每」认的量词按开头相同配对，配错了靠 review；
#   没写除数的并行（「按 31 线程并行约 5 分钟」）按串行乘积比，一个数量级的余量盖得住的照绿。
# 样本：preregistration-parallelism-red（setup.sh 提交基准之后新建 e990 缺整节、e991 有节缺「串行：」一行、e992「预计在用核数：」没有数字，
#   改 e100（号小于 186、没有这一节）不判）；preregistration-parallelism-green（新建 e993 五样写全，改 e100 不判，报「查了 1 份」，e993 的挂钟写成散文、报本次未判）；
#   preregistration-parallelism-wallclock-red（真仓 E186、E187、E197 的「## 十四、并行与挂钟」逐字抄进同号登记，各改坏一个数：E186 变异 17 条改成 1700 条、
#   E187 式子里单态 0.5 毫秒改成 50 毫秒、E197 的 65 次改成 6500 次，三条逐个点名判红）；
#   preregistration-parallelism-wallclock-green（同三节原样：E186、E197 走「每」、E187 走式子，判了 3 条；另一份 e994 只写散文，报本次未判 1 份并点名）；
#   preregistration-parallelism-quantities-section-red（e995 第六节写「变异 17 条」，「预计挂钟：」写「变异 24 条、每条约 1 分钟…」，挂钟算式对得上而态数对不上，点名判红）；
#   preregistration-parallelism-quantities-section-green（e996 第六节写「960 段」与「10000 次发布」，「预计挂钟：」的式子写「960 段 × 10000 次 × 0.5 毫秒 ÷ 31 ≈ 155 秒」，比了 2 个、判绿；
#   另一份 e997 的第六节没写态数，报「态数与第六节的比对本次未判 1 份」并点名，不算红）。
# 弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 parallelism-section-presence-only、parallelism-old-registrations-judged、
#   parallelism-wallclock-unchecked、parallelism-unjudged-as-green、parallelism-quantities-section-unchecked 五项。
# 退役判据：设计员定义不再要求这一节时删掉这一格（连续几周零红不算：它只判新写的登记，零红说明新登记都写全了，不说明对象没了）。
PREREGISTRATION_PARALLELISM_FIRST_NUMBER=186
cell_preregistration_parallelism() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出哪些登记是这次改动新建或改动的，这一格无对象可判"; exit 77; }
  if ((! changed_paths_library_loaded || ! changed_paths_ok)); then
    echo "  ✗ 取不到这次改动的路径（scripts/gate/changed-paths.sh 读不到，或它的 gate_changed_paths 失败）"
    echo "     → 怎么办：看上面 changed-paths.sh 打的错误，修好再单跑这一格（bash .claude/gate.d/doc-experiments.sh --check preregistration-parallelism）"
    exit 1
  fi
  python3 - "$SHARED_CHANGED_PATHS" "$PREREGISTRATION_PARALLELISM_FIRST_NUMBER" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" <<'PY'
import os, re, sys

changed_file, first_number, broken = sys.argv[1], int(sys.argv[2]), sys.argv[3].split(",")
NAME = re.compile(r"research/prompts/e([0-9]+)-preregistration\.md")
HEADING = re.compile(r"^##\s+[一二三四五六七八九十百]+、并行与挂钟\s*$")
LABELS = ("独立单位", "线程", "串行", "预计挂钟", "预计在用核数")
LABEL_LINE = re.compile(r"^\s*(?:[-*]\s+)?(?:\*\*)?(独立单位|线程|串行|预计挂钟|预计在用核数)(?:\*\*)?\s*[：:]\s*(?:\*\*)?\s*(.*)$")
NEEDS_NUMBER = ("预计挂钟", "预计在用核数")

# ── 挂钟算式的读法（判据写在这一格函数上方的注释）
SECONDS = {"纳秒": 1e-9, "ns": 1e-9, "微秒": 1e-6, "µs": 1e-6, "μs": 1e-6, "us": 1e-6, "毫秒": 1e-3, "ms": 1e-3,
           "秒": 1.0, "s": 1.0, "分钟": 60.0, "min": 60.0, "小时": 3600.0, "h": 3600.0, "天": 86400.0}
NUMBER = r"[0-9]+(?:,[0-9]{3})*(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?\s*(?:万|亿)?"
TIME_UNIT = r"(?:纳秒|微秒|毫秒|分钟|小时|秒|天|ns|µs|μs|us|ms|min|s|h)(?![A-Za-z])"
SPAN = r"\s*[–~\-－到至]\s*"
TIME = re.compile(rf"(?P<low>{NUMBER})(?:{SPAN}(?P<high>{NUMBER}))?\s*(?P<unit>{TIME_UNIT})")
TIMES = r"[×✕*]"
WORD = r"[一-鿿]+"
COUNT = re.compile(rf"(?P<number>{NUMBER})\s*(?P<word>{WORD})")
FACTOR = rf"{NUMBER}\s*(?:{WORD})?"
CHAIN = re.compile(rf"(?P<chain>{FACTOR}(?:\s*{TIMES}\s*{FACTOR})+)(?:\s*=\s*(?P<total>{NUMBER})\s*(?P<total_word>{WORD})?)?")
FORMULA_FACTOR = rf"{NUMBER}\s*(?:{TIME_UNIT}|{WORD})?"
FORMULA = re.compile(rf"(?P<factors>(?:{FORMULA_FACTOR}\s*{TIMES}\s*)+{FORMULA_FACTOR})(?:\s*[÷/]\s*(?P<divisor>{NUMBER}))?"
                     rf"\s*(?:≈|=|约等于|约)\s*(?P<result>{NUMBER}(?:{SPAN}{NUMBER})?\s*{TIME_UNIT})")
EACH = re.compile(rf"每(?P<word>{WORD})")
EACH_REACH = 40


def number_value(text):
    text = text.replace(",", "").strip()
    scale = 1.0
    if text.endswith("万"):
        scale, text = 1e4, text[:-1].strip()
    elif text.endswith("亿"):
        scale, text = 1e8, text[:-1].strip()
    return float(text) * scale


def time_span(found):
    unit = SECONDS[found.group("unit")]
    low = number_value(found.group("low")) * unit
    return low, (number_value(found.group("high")) * unit if found.group("high") else low)


def common_prefix(left, right):
    size = 0
    while size < min(len(left), len(right)) and left[size] == right[size]:
        size += 1
    return size


def formula_claims(segment):
    claims = []
    for found in FORMULA.finditer(segment):
        factors = [factor.strip() for factor in re.split(TIMES, found.group("factors"))]
        timed = [TIME.fullmatch(factor) for factor in factors]
        if sum(1 for item in timed if item) != 1:
            continue
        low = high = 1.0
        counts = []   # 这一条用到的态数：(值, 量词, 原文)，与第六节比（quantities_section_counts）
        for factor, item in zip(factors, timed):
            factor_low, factor_high = time_span(item) if item else (number_value(re.match(NUMBER, factor).group(0)),) * 2
            low, high = low * factor_low, high * factor_high
            if not item:
                counted = COUNT.match(factor)
                counts.append((factor_low, counted.group("word") if counted else "", factor))
        divisor = number_value(found.group("divisor")) if found.group("divisor") else 1.0
        claims.append((low / divisor, high / divisor, *time_span(TIME.search(found.group("result"))), f"式子「{found.group(0).strip()}」", counts))
    return claims


def segment_counts(segment):
    """→ [(位置, 态数, 量词, 原文)]：乘出来的一串算一个，单个的「<数><量词>」（不是时间）各算一个。"""
    counts, covered = [], []
    for found in CHAIN.finditer(segment):
        if TIME.search(found.group(0)):
            continue
        if found.group("total"):
            value, word = number_value(found.group("total")), found.group("total_word") or ""
        else:
            value, word = 1.0, ""
            for factor in re.split(TIMES, found.group("chain")):
                item = COUNT.match(factor.strip())
                value *= number_value(item.group("number") if item else factor)
                word = item.group("word") if item else ""
        counts.append((found.start(), value, word, found.group(0).strip()))
        covered.append((found.start(), found.end()))
    for found in COUNT.finditer(segment):
        if any(start <= found.start() < end for start, end in covered) or TIME.match(segment, found.start()):
            continue
        counts.append((found.start(), number_value(found.group("number")), found.group("word"), found.group(0).strip()))
    return counts


def each_claim(segment):
    counts, pairs, last_end = segment_counts(segment), [], 0
    for each in EACH.finditer(segment):
        unit_time = TIME.search(segment, each.end(), each.end() + EACH_REACH)
        candidates = [count for count in counts if count[0] < each.start() and common_prefix(count[2], each.group("word"))]
        if not unit_time or not candidates:
            continue
        _position, value, word, text = max(candidates, key=lambda count: count[0])
        low, high = time_span(unit_time)
        pairs.append((value * low, value * high, f"「{text}」乘「{segment[each.start():unit_time.end()].strip()}」", (value, word, text)))
        last_end = max(last_end, unit_time.end())
    stated = list(TIME.finditer(segment, last_end)) if pairs else []
    if not stated:
        return []
    return [(sum(pair[0] for pair in pairs), sum(pair[1] for pair in pairs), *time_span(stated[-1]),
             "、".join(pair[2] for pair in pairs) + f"，与写的「{stated[-1].group(0).strip()}」比", [pair[3] for pair in pairs])]


def wallclock_claims(line):
    """「预计挂钟：」那一行里认得出的每一条 → [(算出的秒数下限, 上限, 写的秒数下限, 上限, 怎么算的)]。"""
    claims = []
    for segment in re.split(r"[；;。]", line):
        claims.extend(formula_claims(segment) or each_claim(segment))
    return claims


QUANTITIES_SECTION = re.compile(r"^##\s+六、")


def quantities_section_counts(lines):
    """登记第六节（「## 六、」标题到下一个「## 」标题，围栏代码块里的行不算）里写的态数 → [(行号, 值, 量词, 原文)]；
    读法与「预计挂钟：」那一行同一份（segment_counts）；前面紧挨「第」的是序号（「第 3 条」），不算态数。没有第六节交空表。"""
    start = next((index for index, line in enumerate(lines) if QUANTITIES_SECTION.match(line)), None)
    if start is None:
        return []
    found, in_fence = [], False
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if line.startswith("## "):
            break
        if line.lstrip().startswith(("```", "~~~")):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for position, value, word, text in segment_counts(line):
            if word and not line[:position].rstrip().endswith("第"):
                found.append((index + 1, value, word, text))
    return found


def quantities_section_judgement(claims, quantities):
    """「预计挂钟：」那一行用到的每个态数与第六节比 → (对不上的说明, 认不出的原文, 比上了几个)。
    第六节里有同值、量词互为开头的算对上；没有同值而有量词逐字相同的算对不上；都没有（或态数没写量词）算认不出。"""
    mismatched, unknown, compared = [], [], 0
    for claim in claims:
        for value, word, written in claim[5]:
            related = [count for count in quantities if word and (count[2].startswith(word) or word.startswith(count[2]))]
            if any(abs(count[1] - value) <= 1e-9 * max(1.0, value) for count in related):
                compared += 1
                continue
            same_word = [count for count in related if count[2] == word]
            if same_word:
                compared += 1
                mismatched.append(f"「预计挂钟：」里的态数「{written}」与第六节写的"
                                  + "、".join(f"「{count[3]}」（第 {count[0]} 行）" for count in same_word) + "对不上")
            else:
                unknown.append(written)
    return mismatched, unknown, compared


def seconds_text(value):
    for unit, size in (("小时", 3600.0), ("分钟", 60.0), ("秒", 1.0)):
        if value >= size:
            return f"{value / size:.3g} {unit}"
    return f"{value:.3g} 秒"


with open(changed_file, encoding="utf-8") as handle:
    changed = sorted({line.strip() for line in handle if line.strip()})
judged, old, problems, mismatched, unjudged, claim_count, claimed = [], [], [], [], [], 0, []
states_mismatched, states_unjudged, states_compared = [], [], 0
for path in changed:
    found_name = NAME.fullmatch(path)
    if not found_name or not os.path.isfile(path):
        continue
    if int(found_name.group(1)) < first_number and "parallelism-old-registrations-judged" not in broken:
        old.append(path)
        continue
    judged.append(path)
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    start = next((index for index, line in enumerate(lines) if HEADING.match(line)), None)
    if start is None:
        problems.append(f"{path}：没有「## 十四、并行与挂钟」这一节")
        continue
    if "parallelism-section-presence-only" in broken:
        continue
    end = next((index for index in range(start + 1, len(lines)) if lines[index].startswith("## ")), len(lines))
    labelled, in_fence = {}, False
    for index in range(start + 1, end):
        if lines[index].lstrip().startswith(("```", "~~~")):
            in_fence = not in_fence
            continue
        found_label = None if in_fence else LABEL_LINE.match(lines[index])
        if found_label and found_label.group(1) not in labelled:
            labelled[found_label.group(1)] = (index + 1, found_label.group(2).strip().strip("*").strip())
    for label in LABELS:
        if label not in labelled:
            problems.append(f"{path}:{start + 1}：「并行与挂钟」一节里没有「{label}：」一行")
            continue
        number, text = labelled[label]
        if len(text) < 8:
            problems.append(f"{path}:{number}：「{label}：」后面不到 8 个字（「{text}」）")
        elif label in NEEDS_NUMBER and not re.search(r"[0-9]", text):
            problems.append(f"{path}:{number}：「{label}：」里没有数（各段写估值，推的写依据）")
    if "预计挂钟" not in labelled or "parallelism-wallclock-unchecked" in broken:
        continue
    number, text = labelled["预计挂钟"]
    claims = wallclock_claims(text)
    if not claims:
        if "parallelism-unjudged-as-green" not in broken:
            unjudged.append(f"{path}:{number}")
        continue
    claimed.append(path)
    for computed_low, computed_high, stated_low, stated_high, how, _counts in claims:
        claim_count += 1
        if stated_high * 10 < computed_low or stated_low > computed_high * 10:
            mismatched.append(f"{path}:{number}：预计挂钟与态数乘单态耗时差一个数量级以上：{how}；算出约 {seconds_text(computed_low)}"
                              f"{'' if computed_high == computed_low else '–' + seconds_text(computed_high)}，"
                              f"写的约 {seconds_text(stated_low)}{'' if stated_high == stated_low else '–' + seconds_text(stated_high)}")
    # 态数与第六节对：对不上判红，认不出的报本次未判（弄坏开关 parallelism-quantities-section-unchecked 下不比）
    if "parallelism-quantities-section-unchecked" not in broken:
        quantities_mismatched, quantities_unknown, quantities_compared = quantities_section_judgement(claims, quantities_section_counts(lines))
        states_compared += quantities_compared
        states_mismatched.extend(f"{path}:{number}：{problem}" for problem in quantities_mismatched)
        if quantities_unknown:
            states_unjudged.append(f"{path}:{number}（{'、'.join(quantities_unknown)}）")
old_text = f"；号小于 {first_number} 不判的 {len(old)} 份：{'、'.join(old)}" if old else ""
wallclock_text = f"；挂钟算式判了 {claim_count} 条（{len(claimed)} 份）"
unjudged_text = f"；挂钟算式本次未判 {len(unjudged)} 份（写不出态数或单态耗时）：{'、'.join(unjudged)}" if unjudged else ""
if not judged:
    print(f"  ! 这次改动里没有新建或改动 E{first_number} 起的跑前登记（research/prompts/e<号>-preregistration.md），这一格无对象可判{old_text}")
    sys.exit(77)
if unjudged:
    print(f"  ! 挂钟算式本次未判 {len(unjudged)} 份：「预计挂钟：」那一行写不出态数或单态耗时（或写了而没有挂钟可比）：{'、'.join(unjudged)}")
    print("     要这一格判它：在那一行写式子「<态数> × <单态耗时> [÷ <并行数>] ≈ <挂钟>」，或同一段里写「<数><量词>」与「每<量词>…<数><时间单位>」再写这一段的挂钟")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh preregistration-parallelism：挂钟算式本次未判 {len(unjudged)} 份（{'、'.join(unjudged)}）\n")
if states_unjudged:
    print(f"  ! 态数与第六节的比对本次未判 {len(states_unjudged)} 份：「预计挂钟：」里的态数在第六节里找不到同值或同量词的数（或态数没写量词）：{'；'.join(states_unjudged)}")
    print("     要这一格判它：第六节那个量的行里写出同一个态数与量词（「<数><量词>」，例：变异 17 条），「预计挂钟：」里的态数照它写")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh preregistration-parallelism：态数与第六节的比对本次未判 {len(states_unjudged)} 份（{'；'.join(states_unjudged)}）\n")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(problems)} 处没写全「并行与挂钟」（查了 {len(judged)} 份）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md「登记的固定节名」的「## 十四、并行与挂钟」一行补这一节：哪些单位互相独立（独立单位：）、"
          "分几条线程（线程：）、哪几步因计时或共享状态必须串行与为什么（串行：）、各段预计挂钟（预计挂钟：）与预计在用核数（预计在用核数：），推的写依据")
if mismatched:
    for problem in mismatched:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(mismatched)} 处预计挂钟与态数乘单态耗时差一个数量级以上（判了 {claim_count} 条、{len(claimed)} 份）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md 第 3a 步现算态数与单态耗时（量过的写命令与读数，估的写式子和依据），改「预计挂钟：」里写的挂钟，"
          "或改错了的态数、单态耗时；并行跑的写成式子「<态数> × <单态耗时> ÷ <并行数> ≈ <挂钟>」，读法在 doc-experiments.sh 这一格函数上方的注释")
if states_mismatched:
    for problem in states_mismatched:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(states_mismatched)} 处「预计挂钟：」的态数与第六节写的对不上（比了 {states_compared} 个态数）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md 第 3a 步，态数以第六节那个量的定义为准现算，「预计挂钟：」里的态数与挂钟照它改；"
          "第六节写错了的改第六节，两处写的是不同的量的，把「预计挂钟：」里的量词改成与第六节分得开的说法")
if problems or mismatched or states_mismatched:
    sys.exit(1)
states_text = f"；态数与第六节比了 {states_compared} 个" + (f"，本次未判 {len(states_unjudged)} 份：{'；'.join(states_unjudged)}" if states_unjudged else "")
print(f"  ✓ 跑前登记的「并行与挂钟」：查了 {len(judged)} 份（{'、'.join(judged)}），五样都写全了{old_text}{wallclock_text}{unjudged_text}{states_text}")
PY
}

# ── 格 replay-comparison-units：重跑登记的比对对象，在单元表与 replay.sh 里都找得到，与重跑登记里的比对对象表逐行对得上
#
# 还着 .claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 3 行改法的后半句（另立一条检查：每份重跑登记的比对对象表里的每条正式跑，
# 在单元表和 replay.sh 里都找得到）。比对对象表在各份重跑登记里是散文表、写法各不相同，所以先立固定形态再立格（用户 2026-10-22 定）：
# 机读登记 .claude/gate.d/replay-comparison-units.tsv，一行一个比对对象，六列（实验号、登记出处、单元名、单元表出处、留存产物名、复跑），列义写在那份文件头。
# 登记行由执行员在实现单元、登记复跑的同一次改动里加（阶段归属表这一格那一行）。
# 重跑登记里的比对对象表有固定形态（.claude/agents/experiment-designer.md 第 1 步）：表头一行逐字是
#   | # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
#   下一行是分隔行，之后每一行一个正式跑：# 列是正整数、一份登记里不重号，比对对象与重跑产物名两格各是一个文件名（可带一对反引号），七格；围栏代码块里的不算。
# 判四样，前三样每行各判：
#   ① 登记出处（仓内路径）的文件存在；
#   ② 单元名作为带双引号的字符串字面量在单元表出处那份文件里恰好出现一次（0 次是找不到，两次以上是指不到唯一一个单元；「甲」不会在「乙甲」里被数到）；
#   ③ 复跑列写驱动名的，scripts/experiment-apparatus/replay.sh 复跑表里有实验号相同、第 2 列是 @驱动名的行，且其中一行的第 4 列（期望产物）等于留存产物名。
#      复跑表的行经 lib-lag-baseline.py 的 replay_rows 取（与 results-cited、verdict-false-named、quoted-result-lines 三格同一份读法）。
#      复跑列写 none:理由 的不判③，改判只缩不涨（见下）。
# 只缩不涨：复跑写 none: 的行按（实验号，登记出处，单元名，留存产物名）与基准提交那一版登记表里复跑写 none: 的行比，基准里没有的判红；
#   四样都进键：只按（登记出处，留存产物名）成对比时，把一行已有 none 行的实验号或单元名换掉、借它的登记就照绿（样本 replay-comparison-units-rekey-red）。
#   重复行的判法（同一登记出处、同一留存产物名不许两行）不随这个键变。
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句写明；git 取不到基准那一版判红（取法 lib-lag-baseline.py 的 baseline_rows）。
#   补了复跑行（复跑列改写驱动名）的那一行自然不再是 none 行，不用另删什么。
#   ④ 比对对象表与登记表逐行对：判的登记是登记表点名的登记出处，加 research/prompts/ 顶层 e<号> 起头的 .md 里有固定形态表头的那几份。
#      登记表点名的登记出处里没有固定形态的表，判红。一行登记「对上」表里一行：留存产物名等于那一行的比对对象或重跑产物名；
#      重跑产物名不是 e<号>- 起头的全名（旧登记只写前缀之后那一段，如 E165 r5 的「重跑产物名都在前缀 e165-replay-bounded-enumeration-<日期>-r5- 之后」），
#      留存产物名以「-<那一段>」收尾也算对上——旧登记就这样认，不改冻结的登记。
#      表里每一行要有一行登记对上，或在登记表文件头点名不登记（一行一条，行首写「# 不登记的比对对象：<登记出处> #<号>：<理由>」，理由至少 4 个字）；
#      两样都没有、两样都有，判红。登记表里每一行要在它登记出处的表里对上一行，对不上判红。文件头点名的那一行在表里没有、写法不对、同一行点名两次，判红。
#      点名不登记的每一行，理由里要写「欠在 C<号>（简称）」，号要在 .claude/kb/checks-owed.md 开着的那张表里（读法 lib-owed.py）：没写、号不在、号已还清、
#      欠账表不在或认不出「已还清」标题，都判红，点名那一行（单元表里补不补单元归那条欠账，点名不登记就是登记在那条欠账上；这一判替掉了「点名行与基准比、只缩不涨」）。
#      成功句报比对对象表几份几行、对上几行、点名不登记几行。
# 登记表一行都没有（只有注释）而也没有一份登记里有固定形态的表，退 77，不记通过。
# 管不到：表头不是逐字那一行的比对对象表（登记表也没点名那份登记时，这一格看不见它）、sha256 与行数两列对不对、点名不登记的理由除欠账号之外成不成立
#   （欠账号指到的那条账是不是真在说这一行、简称与登记的对不对得上，靠 review）、留存产物名指到的文件在不在、单元表里没有登记的单元、replay.sh 里没有登记的驱动行。
# 样本：replay-comparison-units-red / -green / -zero / -rekey-red / -table-red / -table-green / -exclusion-owed-red，弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 comparison-* 各项。
# 退役判据：单元表与复跑表合成一份机读登记、这张登记表不再有人写时删掉这一格。
cell_replay_comparison_units() {
  UNITS=.claude/gate.d/replay-comparison-units.tsv
  REPLAY=scripts/experiment-apparatus/replay.sh
  [[ -f "$UNITS" ]] || { echo "  ! 没有 $UNITS，这一格无对象可判"; exit 77; }
  base=""
  if ((in_git_work_tree)); then
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
  fi

  python3 - "$UNITS" "$REPLAY" "$base" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" "$LIB_LAG_BASELINE" "$STAGE_DIR/lib-owed.py" <<'PY'
import glob, importlib.util, os, re, sys

units_path, replay_path, base, broken_text, library, owed_library_path = sys.argv[1:7]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", library)
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
broken = set(broken_text.split(","))
units_name = os.path.basename(units_path)
NONE_PREFIX = "none:"
COLUMNS = 6

def none_identity(experiment, registration, unit, product):
    """复跑写 none: 的一行在「只缩不涨」里的键：（实验号，登记出处，单元名，留存产物名）。
    弄坏开关 comparison-none-key-pair 退回改前的（登记出处，留存产物名）：换实验号、换单元名借同一对键照绿。"""
    if "comparison-none-key-pair" in broken:
        return (registration, product)
    return (experiment, registration, unit, product)

def none_key(fields):
    """登记表一行复跑写 none: 的，交它的键（none_identity）；别的交 None。"""
    if len(fields) != COLUMNS or not fields[5].strip().startswith(NONE_PREFIX):
        return None
    return none_identity(*(fields[index].strip() for index in (0, 1, 2, 4)))

sources = {}
def source_text(path):
    if path not in sources:
        sources[path] = open(path, encoding="utf-8", errors="replace").read() if os.path.isfile(path) else None
    return sources[path]

# 重跑登记里比对对象表的固定形态（.claude/agents/experiment-designer.md 第 1 步）
FIXED_HEADER = "| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |"
TABLE_COLUMNS = 7
SEPARATOR_ROW = re.compile(r"\s*\|(\s*:?-{3,}:?\s*\|)+\s*")
FENCE_OPEN = re.compile(r" {0,3}(`{3,}|~{3,})")
FULL_PRODUCT_NAME = re.compile(r"e[0-9]+-")
EXCLUSION_START = re.compile(r"#\s*不登记的比对对象：")
EXCLUSION = re.compile(r"#\s*不登记的比对对象：(?P<registration>\S+)\s+#(?P<row>[0-9]+)：(?P<reason>.+)")
MINIMUM_REASON_CHARACTERS = 4
# 点名不登记的每一行指到的欠账：理由里写「欠在 C<号>（简称）」，号要在欠账表开着的那张表里（读法 lib-owed.py，与读欠账表的各格同一份）
OWED_PATH = ".claude/kb/checks-owed.md"
OWED_REFERENCE = re.compile(r"欠在\s*(?P<number>C[0-9]+)（(?P<name>[^（）]+)）")
owed_spec = importlib.util.spec_from_file_location("lib_owed", owed_library_path)
owed_library = importlib.util.module_from_spec(owed_spec)
owed_spec.loader.exec_module(owed_library)

def exclusion_owed_problem(reason, owed):
    """点名不登记的一行的理由指没指到一条开着的欠账：指到了交 None，没指到交一句。弄坏开关 comparison-exclusion-owed-unchecked 不判。"""
    if "comparison-exclusion-owed-unchecked" in broken:
        return None
    found = OWED_REFERENCE.search(reason)
    if not found:
        return "「不登记的比对对象」没写欠在哪一条账（理由里要写「欠在 C<号>（简称）」）"
    number = found.group("number")
    if not owed.file_found:
        return f"写的 {number} 查不了：{OWED_PATH} 不在"
    if not owed.paid_heading_found:
        return f"写的 {number} 查不了：{OWED_PATH} 里认不出「已还清」那一行整行标题，分不出哪些欠账还开着"
    if number in owed.paid_names:
        return f"写的 {number}（{owed.paid_names[number]}）在 {OWED_PATH} 里已还清：还清了就该补上单元、改登记成正式行，不再点名不登记"
    if number not in owed.open_names:
        return f"写的 {number} 在 {OWED_PATH} 里没有登记"
    return None

def markdown_cells(line):
    """表格的一行切成格：反引号里的 | 与 \\| 不切。"""
    body = line.strip()
    body = body[1:] if body.startswith("|") else body
    body = body[:-1] if body.endswith("|") and not body.endswith("\\|") else body
    cells, current, in_code, index = [], [], False, 0
    while index < len(body):
        character = body[index]
        if character == "\\" and body[index + 1:index + 2] == "|":
            current.append("|")
            index += 2
            continue
        if character == "`":
            in_code = not in_code
        if character == "|" and not in_code:
            cells.append("".join(current).strip())
            current = []
        else:
            current.append(character)
        index += 1
    cells.append("".join(current).strip())
    return cells

def file_name_cell(text):
    """格里的一个文件名（可带一对反引号）；不是一个不带空白的名字交 None。"""
    name = text.strip()
    if len(name) >= 2 and name.startswith("`") and name.endswith("`"):
        name = name[1:-1].strip()
    return name if name and not re.search(r"[\s`|]", name) else None

def comparison_table(path):
    """→ (rows, problems)：rows 是 None（这份登记里没有表头逐字的比对对象表）或 [{row, object, rerun, line}]；围栏代码块里的不算。"""
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError as error:
        return None, [f"{path}：读不了（{error.strerror}）"]
    rows, problems, fence, numbered, index = None, [], None, {}, 0
    while index < len(lines):
        line = lines[index].rstrip("\r")
        index += 1
        opener = FENCE_OPEN.match(line)
        if fence is not None:
            if opener and opener.group(1)[0] == fence[0] and len(opener.group(1)) >= len(fence) and not line.strip()[len(opener.group(1)):].strip():
                fence = None
            continue
        if opener:
            fence = opener.group(1)
            continue
        if line.strip() != FIXED_HEADER:
            continue
        rows = [] if rows is None else rows
        if index >= len(lines) or not SEPARATOR_ROW.fullmatch(lines[index].rstrip("\r")):
            problems.append(f"{path} 第 {index} 行是比对对象表头，下面一行不是分隔行（|---|…）")
            continue
        index += 1
        while index < len(lines) and lines[index].lstrip().startswith("|"):
            number, text = index + 1, lines[index].rstrip("\r")
            index += 1
            cells = markdown_cells(text)
            if len(cells) != TABLE_COLUMNS:
                problems.append(f"{path} 第 {number} 行：比对对象表这一行不是 {TABLE_COLUMNS} 格：{text[:90]}")
                continue
            row, comparison_object, rerun_product = cells[0], file_name_cell(cells[1]), file_name_cell(cells[5])
            if not re.fullmatch(r"[1-9][0-9]*", row):
                problems.append(f"{path} 第 {number} 行：比对对象表的 # 列要写正整数，一行一个正式跑：{row[:30]}")
                continue
            if comparison_object is None or rerun_product is None:
                problems.append(f"{path} 第 {number} 行：比对对象与重跑产物名两格各要是一个文件名（可带一对反引号）：{text[:90]}")
                continue
            if row in numbered:
                problems.append(f"{path} 第 {number} 行：比对对象表 #{row} 与第 {numbered[row]} 行重号")
                continue
            numbered[row] = number
            rows.append({"row": row, "object": comparison_object, "rerun": rerun_product, "line": number})
    return rows, problems

def covers(product, row):
    """登记表一行的留存产物名对不对得上比对对象表的这一行。"""
    if product in (row["object"], row["rerun"]):
        return True
    if FULL_PRODUCT_NAME.match(row["rerun"]) or "comparison-table-suffix-unrecognized" in broken:
        return False
    return product.endswith("-" + row["rerun"])

problems, registered, seen = [], [], {}
units_text = open(units_path, encoding="utf-8").read()
for number, fields in lag_baseline.content_rows(units_text):
    where = f"{units_name} 第 {number} 行"
    if len(fields) != COLUMNS or not all(field.strip() for field in fields):
        problems.append(f"{where}：不是六列（实验号、登记出处、单元名、单元表出处、留存产物名、复跑）或有一列是空的：{'<制表符>'.join(fields)[:90]}")
        continue
    experiment, registration, unit, unit_source, product, rerun = (field.strip() for field in fields)
    if not re.fullmatch(r"E[0-9]+", experiment):
        problems.append(f"{where}：实验号不是 E<数字> 的形状：{experiment}")
        continue
    if rerun.startswith(NONE_PREFIX):
        if len(rerun[len(NONE_PREFIX):].strip()) < 4:
            problems.append(f"{where}：复跑写了 none: 而理由不到 4 个字：{rerun}")
            continue
    elif not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", rerun):
        problems.append(f"{where}：复跑不是 replay.sh 里的驱动名（不带 @），也不是 none:理由：{rerun}")
        continue
    key = (registration, product)
    if key in seen:
        problems.append(f"{where}：和第 {seen[key]} 行是同一条（登记出处与留存产物名都相同）：{product}")
        continue
    seen[key] = number
    registered.append((number, experiment, registration, unit, unit_source, product, rerun))

replay = lag_baseline.replay_rows(replay_path)
driver_rows = 0
none_rows = []
for number, experiment, registration, unit, unit_source, product, rerun in registered:
    where = f"{units_name} 第 {number} 行（{experiment}「{unit}」）"
    if "comparison-registration-unchecked" not in broken and not os.path.isfile(registration):
        problems.append(f"{where}：登记出处不在：{registration}")
    if "comparison-unit-unchecked" not in broken:
        text = source_text(unit_source)
        if text is None:
            problems.append(f"{where}：单元表出处不在：{unit_source}")
        else:
            count = text.count(unit) if "comparison-unit-substring" in broken else text.count(f'"{unit}"')
            if count == 0:
                problems.append(f"{where}：单元名「{unit}」在 {unit_source} 里一次都没找到（要作为带双引号的字符串字面量出现）")
            elif count > 1 and "comparison-unit-count-loose" not in broken:
                problems.append(f"{where}：单元名「{unit}」在 {unit_source} 里出现 {count} 次，不是恰好一次，指不到唯一一个单元")
    if rerun.startswith(NONE_PREFIX):
        none_rows.append((number, experiment, registration, unit, product))
        continue
    driver_rows += 1
    if replay is None:
        problems.append(f"{where}：读不了 {replay_path}（不在，或没有读权限），驱动行没法核")
        continue
    mine = [row for row in replay if row.experiment == experiment and row.driver == "@" + rerun]
    if not mine:
        if "comparison-driver-unchecked" not in broken:
            problems.append(f"{where}：{replay_path} 复跑表里没有 {experiment} 的驱动行 @{rerun}")
    elif "comparison-product-unchecked" not in broken and product not in {row.product for row in mine}:
        problems.append(f"{where}：@{rerun} 那一行的产物名是 {'、'.join(sorted({row.product for row in mine}))}，不是留存产物名 {product}")

# ④ 比对对象表与登记表逐行对
table_count = table_rows = covered_rows = excluded_rows = 0
table_checked = "comparison-table-unchecked" not in broken
if table_checked:
    exclusions = {}
    owed_table = owed_library.read_owed_table(OWED_PATH)
    for number, line in enumerate(units_text.split("\n"), 1):
        stripped = line.strip()
        if not EXCLUSION_START.match(stripped):
            continue
        found = EXCLUSION.fullmatch(stripped)
        if not found or len(found.group("reason").strip()) < MINIMUM_REASON_CHARACTERS:
            problems.append(f"{units_name} 第 {number} 行：「不登记的比对对象」要写成「# 不登记的比对对象：<登记出处> #<号>：<理由>」，理由至少 {MINIMUM_REASON_CHARACTERS} 个字：{stripped[:90]}")
            continue
        owed_problem = exclusion_owed_problem(found.group("reason"), owed_table)
        if owed_problem:
            problems.append(f"{units_name} 第 {number} 行：{owed_problem}：{stripped[:90]}")
        key = (found.group("registration"), found.group("row"))
        if key in exclusions:
            problems.append(f"{units_name} 第 {number} 行：{key[0]} #{key[1]} 在第 {exclusions[key]} 行已经点名不登记")
            continue
        exclusions[key] = number
    by_registration = {}
    for item in registered:
        by_registration.setdefault(item[2], []).append(item)
    candidates = set(by_registration) | {path for path, _row in exclusions} | set(glob.glob("research/prompts/e[0-9]*.md"))
    tables = {}
    for path in sorted(path for path in candidates if os.path.isfile(path)):
        rows, table_problems = comparison_table(path)
        problems.extend(table_problems)
        if rows is not None:
            tables[path] = rows
    for path, items in sorted(by_registration.items()):
        if os.path.isfile(path) and path not in tables:
            for number, experiment, registration, unit, unit_source, product, rerun in items:
                problems.append(f"{units_name} 第 {number} 行（{experiment}「{unit}」）：登记出处 {path} 里没有表头逐字是「{FIXED_HEADER}」的比对对象表")
    for path, rows in sorted(tables.items()):
        table_count += 1
        table_rows += len(rows)
        items = by_registration.get(path, [])
        for number, experiment, registration, unit, unit_source, product, rerun in items:
            if not any(covers(product, row) for row in rows):
                problems.append(f"{units_name} 第 {number} 行（{experiment}「{unit}」）：留存产物名 {product} 在 {path} 的比对对象表里一行都对不上"
                                f"（要等于某一行的比对对象或重跑产物名）")
        for row in rows:
            covered = any(covers(item[5], row) for item in items)
            excluded = (path, row["row"]) in exclusions and "comparison-exclusions-ignored" not in broken
            described = f"{path} 比对对象表 #{row['row']}（比对对象 {row['object']}，重跑产物名 {row['rerun']}）"
            if covered and excluded:
                problems.append(f"{described}：登记表里有一行对上，文件头第 {exclusions[(path, row['row'])]} 行又点名不登记")
            elif covered:
                covered_rows += 1
            elif excluded:
                excluded_rows += 1
            else:
                problems.append(f"{described}在 {units_name} 里没有一行对得上，文件头也没点名不登记")
    for (path, row_number), number in sorted(exclusions.items()):
        if row_number not in {row["row"] for row in tables.get(path, [])}:
            problems.append(f"{units_name} 文件头「不登记的比对对象」点名的 {path} #{row_number} 在那份登记的比对对象表里没有（第 {number} 行）")

# 只缩不涨：复跑写 none: 的行与基准那一版登记表里复跑写 none: 的行比
baseline_none_keys, baseline_problem = None, None
if base:
    baseline, baseline_problem = lag_baseline.baseline_rows(base, units_path)
    if baseline is not None:
        baseline_none_keys = {key for key in (none_key(fields) for _number, fields in baseline) if key is not None}
grown = ([row for row in none_rows if none_identity(*row[1:]) not in baseline_none_keys]
         if baseline_none_keys is not None and "comparison-lag-growth-unchecked" not in broken else [])

if problems:
    print(f"  ✗ {units_path} 里 {len(problems)} 处对不上：")                     # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                                 # gate-lint:detail
    print(f"     → 怎么办：列义在 {units_path} 文件头。登记出处写仓内现存路径；单元名照单元表里的字符串字面量逐字抄，要在单元表出处里恰好出现一次；")
    print(f"               复跑写 scripts/experiment-apparatus/replay.sh 复跑表里的驱动名（不带 @），留存产物名照那一行第 4 列抄；确实没有复跑行的写 none:理由（只缩不涨）。")
    print(f"               重跑登记的比对对象表照固定形态写（表头逐字「{FIXED_HEADER}」，一行一个正式跑，.claude/agents/experiment-designer.md 第 1 步）；")
    print(f"               表里每一行要有一行登记对上（留存产物名等于比对对象或重跑产物名），单元表里没有单元的在 {units_name} 文件头写「# 不登记的比对对象：<登记出处> #<号>：<理由>」，")
    print(f"               理由里写「欠在 C<号>（简称）」：没有欠账号的，先在 `{OWED_PATH}` 登记欠账号，同一行写上；号已还清的，补上单元、改登记成正式行。")
if baseline_problem is not None:
    print(f"  ✗ {baseline_problem.sentence()}，复跑写 none: 的行只缩不涨没比")
    print("     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑；取不到基准那一版不是「基准里还没有这张表」。")
if grown:
    print(f"  ✗ none 行比基准多：{units_name} 里这几行复跑写 none:，基准 {base} 那一版里没有这样的行：")  # gate-lint:summary
    for number, experiment, registration, unit, product in grown:
        print(f"     第 {number} 行 {experiment}「{unit}」{product}（{registration}）")  # gate-lint:detail
    print("     → 怎么办：复跑写 none: 的行只缩不涨，按（实验号，登记出处，单元名，留存产物名）与基准比：换了实验号或单元名就是新的比对对象。")
    print("               新登记的比对对象要在 replay.sh 复跑表里加一行驱动行、复跑列写驱动名，不许写 none: 把它按下去，也不许改一行已有 none 行的实验号或单元名借它的登记。")
if problems or baseline_problem is not None or grown:
    sys.exit(1)

if not registered and not table_count:
    print(f"  ⊘ 本次无对象可判：{units_path} 里一行比对对象都没有")
    sys.exit(77)
compared = f"与基准 {base} 比没涨" if baseline_none_keys is not None else "基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比"
experiments = sorted({row[1] for row in registered})
tables_compared = (f"；比对对象表 {table_count} 份 {table_rows} 行：登记表对上 {covered_rows} 行、文件头点名不登记 {excluded_rows} 行"
                   + ("" if "comparison-exclusion-owed-unchecked" in broken else "（点名行都欠在开着的欠账上）")
                   if table_checked else "")
print(f"  ✓ 比对对象 {len(registered)} 行（{'、'.join(experiments)}）：登记出处都在，单元名在单元表出处里都恰好出现一次；"
      f"复跑写驱动名的 {driver_rows} 行在 replay.sh 里都有且产物名相符；复跑写 none 的 {len(none_rows)} 行只缩不涨，{compared}{tables_compared}")
PY
}

stage_cells_run "$ROOT"
