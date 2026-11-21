#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 实验页与产物（十六格：索引行与正文标题、产物写回、产物与依据落在仓里、决策与实验双向登记、判决行的 false 被点名、复跑命令、实验号有正文、整行抄的产物行、上一轮记录已归档、多路径互证的登记、点名的路径找得到、跑前登记的并行与挂钟、跑前登记里走崩溃判段的装置带 gpu 特性、判段入口表与源码同步、重跑登记的比对对象表与机读登记逐行对得上且在单元表与复跑表里找得到、钉了装置 sha256 的起跑脚本的钉值或装置源码变了而登记第十二节没写着新钉值的修订、改写了已有的行、钉回已作废的值或登记被改名走了）
# gate-fixture: yes
# gate-category: 文档类
# 不声明 gate-covers：未实现清单（.claude/gate-not-implemented.tsv）里的两项判被测代码的行为，这一道只判文本。
# gate-similar: research-build-and-replay.sh 它的 experiment-replay 格按 scripts/experiment-apparatus/replay.sh 登记表逐条复跑装置、逐字节比产物，要编译；这一道只读登记表、实验页与产物的文本，判产物有没有被点名、判决行的 false 有没有被解释、整行抄的产物行找不找得到，不跑装置
# gate-similar: doc-registries.sh 它的 experiment-refs 格判全 kb 里的实验号引用有定义，对象是引用；audit-contradictions 格核欠账表与里程碑收口表的引文在 kb 里找得到，方向是记录到 kb；segment-registry 格拿段序列登记表与 E142 的产物逐字比，对象是一份登记表。这一道判实验页、实验索引与产物本身，实验与决策之间的双向登记归这里的 decision-links 格，quoted-result-lines 格方向是 kb 与 research 正文到产物，multipath-registry 格还着 C48 可机检的那一半而不判欠账表本身
# gate-similar: doc-process-records.sh 也用 gate 取法的改动范围、也读 research/prompts/，但它判三方与同步留下的记录写没写、点没点名；这一道判产物与依据落没落进仓、新判决回看没回看决策、上一轮记录归没归档
# gate-similar: code-source-discipline.sh 它判每个实验二进制都有变异表、变异条目锚得住、钉绝对值的断言与读数纪律，对象是实验源码与变异表的内容；这一道的 evidence-in-repo 格只看这一轮改过的装置与变异表有没有一份不比它旧的产物
# gate-similar: code-tooling.sh 它的 change-range-single-source 格判各阶段取改动范围只经 changed-paths.sh 一份取法；这里是那份取法的使用者之一，改动范围在这一道里只取一次、十格共用
# gate-similar: doc-registries.sh 它的 governance-refs 格（判法 lib-governance-refs.py 的 ② 扩到 kb）只判 kb 中文现状文字（历史版本节、变更史、英日页与代码围栏除外）反引号里以仓根目录起头的全部路径，不在就红、只 research/ 下进过 git 的放行；这里的 cited-paths-exist 格判 kb 全文（含历史版本节与英日页，反引号内外都认）里实验产物、变异表、实验源码三类路径，树里或历史里有就放。判的文字段与判法都不同，只在中文现状文字里这三类上重叠（research/ 那一类两边都是历史里有就放，另两类那边更严），不并
# gate-similar: doc-experiments.sh 自己的 repro-command 格已有「产物逐字节复跑不了」登记表 .claude/gate.d/repro-command-unreproducible.tsv，按实验页登记、只放行「没写复跑命令」；quoted-result-lines 格读的 .claude/gate.d/quoted-product-unreplayable.tsv 按产物文件名登记、放行的是「整行抄的行去哪份产物里找」。键（页 / 产物）、列（页与原因 / 产物、怎么跑出的、为什么进不了复跑表）与放行的判据都不同，并成一张表一行要同时答两件事，不并
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
# gate-cell: preregistration-judge-features 这次改动新建或改动的跑前登记里命令位置上（围栏里的行、行首是命令的行、表格里整格是命令的格；行文里内嵌的不判）起装置的 cargo run / cargo test，反斜杠续行先并成一条（shell 注释不续行），点名的 --bin（cargo test 另认 --test）目标走崩溃判段（源文件调了 crash_amplification、crash_state_enumeration 的判段入口，别名导入照认）的特性要带 gpu
# gate-cell: judge-entries-in-sync 判段入口表 JUDGE_ENTRIES 与 crash_amplification.rs、crash_state_enumeration.rs 里认出的公开判段入口逐个相同
# gate-cell: replay-comparison-units 重跑登记的比对对象逐行登记在机读表里：登记出处在、单元名在单元表出处里恰好出现一次、复跑驱动在 replay.sh 里且产物名相符；重跑登记里表头逐字的比对对象表与机读表逐行对得上
# gate-cell: pinned-apparatus-revision 起跑脚本钉了装置 sha256（registered_apparatus_sha256=）的，钉值或装置源码在这次改动里变了，就核它的跑前登记第十二节：装置源码变了的要新加一行写着新钉值的修订，只改钉值的要写着新钉值；基准那一版第十二节的行都原样在，新钉值写在基准钉值之后（钉回已作废的值不算），登记被改名走的判红
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
# pinned-apparatus-revision-red / -green / -unresolved-red：钉了装置 sha256 的起跑脚本（放了什么写在那一格的注释里）：red 的装置源码都变了，一份登记完全没动、一份只改了别的节，两份判红；
#   green 的两份都在第十二节新加了一行、另有一份登记已归档不判、一份装置没变不判；unresolved-red 的两份起跑脚本认不出装置源码、认不出跑前登记，判红。
#   钉值与登记的十八份样本（每份放了什么写在那一格的注释里）：pinned-apparatus-revision-inplace-rewrite-red、-internal-whitespace-red、-duplicated-line-red、-repin-only-red / -repin-only-green、
#   -pin-trailing-comment-red、-registry-mentioned-after-another-red、-registry-unlabeled-red、-added-line-without-pin-red、-pin-in-other-section-red、-archived-history-red / -archived-history-green、-pin-prefix-green、
#   -older-line-rewritten-red、-archived-revert-to-superseded-red、-intree-repin-superseded-red、-revert-with-new-line-green、-registry-renamed-red。
# preregistration-parallelism-wallclock-red / -wallclock-green：真仓 E186、E187、E197 的「## 十四、并行与挂钟」逐字抄进同号登记（红的各改坏一个数），判预计挂钟与态数乘单态耗时（放了什么写在那一格的注释里）。
# preregistration-parallelism-quantities-section-red / -quantities-section-green：「预计挂钟：」里的态数与第六节写的对不上的、对得上的与认不出的（放了什么写在那一格的注释里）。
# preregistration-judge-features-red / -green：跑前登记里起走崩溃判段的装置二进制没带 / 带了 gpu 特性（放了什么写在那一格的注释里）；
# preregistration-judge-features-prose-green：没带 gpu 的几处都不在命令位置（行文里内嵌、标签之后、表格里夹着字、围栏里的输出行），不判。
# preregistration-judge-features-continued-red / -continued-green：围栏里的命令反斜杠续行，--bin 拆到下一行没带 gpu / --features gpu 拆到下一行；
# preregistration-judge-features-alias-red：别名导入之后调判段入口没带 gpu 判红、只取崩溃点不调判段入口的不报（放了什么写在那一格的注释里）。
# preregistration-judge-features-comment-swallows-red / -comment-hides-red / -comment-control-red：围栏里以反斜杠收尾的 shell 注释行不续行，下一行没带 gpu 的命令照判红
#   （第三份注释不以反斜杠收尾，是对照）；preregistration-judge-features-trailing-comment-red：命令行尾注释里写的 --features gpu 不算带了；
# preregistration-judge-features-continued-outside-fence-red：围栏外行首是命令的行以反斜杠续行，--bin 拆到下一行没带 gpu，并成一条判红；
# preregistration-judge-features-spellings-green：-Fgpu 与 --features "lookup-work-counters gpu" 都算带了 gpu；preregistration-judge-features-test-target-red：cargo test 只点名 --test、测试目标调判段入口没带 gpu 判红。
# preregistration-judge-features-list-item-trailing-comment-red / -table-cell-trailing-comment-red：列表项里行首是命令的行 / 表格整格的命令，行尾 shell 注释里写的 --features gpu 截掉、不算带了，判红。
# judge-entries-in-sync-green / -red / -stale-red：判段入口表与两份判段源文件逐个相同 / 源码里多一个公开入口 / 表里的一个名字在源码里不再公开（放了什么写在那一格的注释里）。
# judge-entries-in-sync-array-parameter-new-entry-red / -array-parameter-on-the-path-green / -cycle-new-entry-after-red：签名里带 [u8; 32] 的新公开入口判红 / 判段路径上的私有函数签名里带数组照连边、判绿 /
#   新公开入口与表里的入口互相调、写在它后面照判红（放了什么写在那一格的注释里）。
# experiment-orphans-registry-red：样本根的目录登记末尾加一行 notes，notes/ 下的号与只登了索引行的号判成孤儿（登记里加一行，这一格跟着认）。
# experiment-orphans-model-directory-green / -sibling-red：三方腿的模型目录（research/prompts/ 直接底下名字以 -model 结尾的目录）里的假 e990 样本不算用过的号，只有 kb 页的 E99 一个号，判绿 /
#   模型目录外同名形态的照算（research/prompts/ 底下的、research/results/ 底下的 -model 目录里的、research/prompts/ 更深一层的 -model 目录里的），红行点名这三处，模型目录里的假样本同在样本里。
# experiment-orphans-page-directory-red / -page-binary-red / -empty-page-red：E42 有产物，kb/experiments/ 里同号的只有目录、只有 .bin、只有只含换行的 .md，都不算正文，判 E42 没有正文。
# experiment-orphans-blank-beside-body-red：同 -empty-page-red，kb/experiments/ 里另有一页有正文的 99（E99 有产物）：有正文的页不替只含换行的 42 作保，判 E42 没有正文。
# quoted-result-lines-normalized-green：产物三行，一行前后带空格、一行的回车隔在 E7RESULT 与空格之间（行尾也有回车）、一行原样，页里整行抄的三行都找得到（setup.sh 造产物）。
# 按编号取实验页的四份（实验页有中文原文 <号>-<名>.md 与译文 <号>-<名>.en.md / .ja.md，中文的索引与产物取中文原文，判法 lib-item-ref-status.py 的 numbered_files）：
# index-sync-translation-green / -red（索引结论列的数在中文原文里、译文里没有 / 只在英语译文里有），
# verdict-false-named-translation-green（E174 的中文原文没有「## 历史版本」、正文点了 false 字段的名，两份译文排在前面：绿）、
# verdict-false-named-no-history-red（中文原文没有历史节也没点名、英语译文点了名：红，不崩）。
# 产物只以 .out.gz 存在（超过单文件推送上限、压成 .gz 入库的）的六份样本：每份的 setup.sh 把样本里的 .out 用 gzip -n -9 压成 .out.gz 并删掉原文件，
# 三格读它们都是解压后读、判据不变：verdict-false-named-gz-green / -gz-red（解压内容里的 false 字段点了名 / 没点名，红行点名 .out.gz 与解压内容里的行号）、
# quoted-result-lines-gz-green / -gz-red（整行抄的产物行在解压内容里逐字找得到 / 有一行不在，红的汇总句数「有 1 行」）、
# repro-command-gz-green / -gz-red（页点名 .out.gz 产物并写了 cargo run / 点名了却只提到变异脚本）。
# 纯 Python 装置的复跑行与产物逐字节复跑不了的登记表五份（脚本与装置目录由每份的 setup.sh 造出来，样本里不放 .py / .sh）：
# repro-command-apparatus-green / -apparatus-red（复跑行点名的脚本都在 / 点名的脚本或装置目录不在）、
# repro-command-unreproducible-green（没写复跑命令的页登记了、放行）/ -unreproducible-page-missing-red（登记行指到不存在的实验页）/ -unreproducible-reason-empty-red（登记行没写原因）。
# 整行抄的行去进不了 replay.sh 复跑表的登记产物里找的五份样本（replay.sh 骨架只登记 E99 的一份产物，另有分片跑出的产物不在 replay.sh 里；setup.sh 与 quoted-result-lines-red 同一写法，实验页停在未跟踪状态）：quoted-result-lines-unreplayable-green / -mistyped-red / -unlisted-red / -product-missing-red / -reason-empty-red（各放了什么写在那一格的注释里）。
# cited-paths-exist 认 .out.gz 存放的两份样本（setup.sh 同样 gzip -n -9）：cited-paths-exist-gz-green（点名 .out，树里只有 .out.gz；另一份只在历史里有 .out.gz）、
# cited-paths-exist-gz-red（点名的 .out 树里与历史里 .out 与 .out.gz 都没有，另一份有 .out.gz 的点名了 .out，只报前一个）。
# 压成 .out.gz 之后点名与归档的四份样本（setup.sh 同样 gzip -n -9）：results-cited-gz-green（树里只有 .out.gz，一页点名 .out、一页点名 .out.gz，都算点名）、
# results-cited-gz-history-green（压缩产物只在历史里有 .out.gz，页里点名 .out 也算找得到）、results-cited-gz-red（点名的 .out 与 .out.gz 两样都不在，另一份 .out.gz 没人点名）、
# quoted-result-lines-archived-gz-green / -archived-gz-red（登记着的产物历史里只有 .out.gz，整行抄的行解压后逐字找得到 / 抄错一字，红的汇总句数「有 1 行」）。
# 弄坏开关（打开后样本判错；每条下面逐行点名判错的样本）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库 lib/stage-cells.sh：红格按绿记、整道退 0
#     archive-past-rounds-red、cited-paths-exist-gz-red、cited-paths-exist-red、decision-links-language-switch-red
#     decision-links-moved-and-switch-red、decision-links-pending-base-unreadable-red、decision-links-progress-mark-red、decision-links-red
#     decision-links-switch-inserted-red、evidence-in-repo-mixed-replay-red、evidence-in-repo-red、evidence-in-repo-worktree-red、experiment-orphans-red
#     experiment-orphans-registry-red、experiment-orphans-model-directory-sibling-red、experiment-orphans-page-directory-red、experiment-orphans-page-binary-red
#     experiment-orphans-empty-page-red、experiment-orphans-blank-beside-body-red、index-sync-red、index-sync-translation-red、multipath-registry-red
#     pinned-apparatus-revision-red、pinned-apparatus-revision-unresolved-red
#     pinned-apparatus-revision-inplace-rewrite-red、pinned-apparatus-revision-internal-whitespace-red、pinned-apparatus-revision-duplicated-line-red
#     pinned-apparatus-revision-repin-only-red、pinned-apparatus-revision-pin-trailing-comment-red、pinned-apparatus-revision-registry-mentioned-after-another-red
#     pinned-apparatus-revision-registry-unlabeled-red、pinned-apparatus-revision-added-line-without-pin-red、pinned-apparatus-revision-pin-in-other-section-red
#     pinned-apparatus-revision-archived-history-red
#     pinned-apparatus-revision-older-line-rewritten-red、pinned-apparatus-revision-archived-revert-to-superseded-red
#     pinned-apparatus-revision-intree-repin-superseded-red、pinned-apparatus-revision-registry-renamed-red
#     evidence-in-repo-exact-only-rerun-red、evidence-in-repo-unrelated-file-red、evidence-in-repo-timing-rows-red、evidence-in-repo-crates-bin-red
#     evidence-in-repo-suffix-timing-row-red、evidence-in-repo-timing-gz-red
#     preregistration-parallelism-git-error-red、preregistration-parallelism-red、preregistration-parallelism-wallclock-red、quoted-result-lines-archived-gz-red、quoted-result-lines-gz-red
#     preregistration-parallelism-quantities-section-red、preregistration-judge-features-red
#     preregistration-judge-features-continued-red、preregistration-judge-features-alias-red
#     preregistration-judge-features-comment-swallows-red、preregistration-judge-features-comment-hides-red、preregistration-judge-features-comment-control-red
#     preregistration-judge-features-trailing-comment-red、preregistration-judge-features-continued-outside-fence-red、preregistration-judge-features-test-target-red
#     judge-entries-in-sync-red、judge-entries-in-sync-stale-red、judge-entries-in-sync-array-parameter-new-entry-red、judge-entries-in-sync-cycle-new-entry-after-red
#     preregistration-judge-features-list-item-trailing-comment-red、preregistration-judge-features-table-cell-trailing-comment-red
#     quoted-result-lines-history-added-red、quoted-result-lines-moved-and-edited-red、quoted-result-lines-moved-unregistered-red、quoted-result-lines-red
#     quoted-result-lines-unreplayable-mistyped-red、quoted-result-lines-unreplayable-product-missing-red
#     quoted-result-lines-unreplayable-reason-empty-red、quoted-result-lines-unreplayable-unlisted-red
#     replay-comparison-units-exclusion-owed-red、replay-comparison-units-red、replay-comparison-units-rekey-red、replay-comparison-units-table-red
#     repro-command-apparatus-red、repro-command-gz-red、repro-command-red
#     repro-command-unreproducible-page-missing-red、repro-command-unreproducible-reason-empty-red
#     results-cited-gz-red、results-cited-red
#     verdict-false-named-gz-red、verdict-false-named-no-history-red、verdict-false-named-red、verdict-false-named-zero
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本十六格全跑
#     archive-past-rounds-green、archive-past-rounds-red、cited-paths-exist-green、cited-paths-exist-gz-green
#     cited-paths-exist-gz-red、cited-paths-exist-red、decision-links-deleted-product-green、decision-links-formatting-green
#     decision-links-green、decision-links-language-switch-green、decision-links-language-switch-red、decision-links-moved-and-switch-green
#     decision-links-moved-and-switch-red、decision-links-no-experiment-green、decision-links-pending-base-unreadable-red、decision-links-progress-mark-green
#     decision-links-progress-mark-red、decision-links-red、decision-links-switch-inserted-green、decision-links-switch-inserted-red
#     decision-links-translation-green、evidence-in-repo-green、evidence-in-repo-mixed-replay-red、evidence-in-repo-red、evidence-in-repo-worktree-green
#     evidence-in-repo-worktree-red、experiment-orphans-green、experiment-orphans-red、experiment-orphans-registry-red
#     experiment-orphans-model-directory-green、experiment-orphans-model-directory-sibling-red
#     experiment-orphans-page-directory-red、experiment-orphans-page-binary-red、experiment-orphans-empty-page-red、experiment-orphans-blank-beside-body-red
#     index-sync-green、index-sync-red、index-sync-translation-green、index-sync-translation-red
#     multipath-registry-green、multipath-registry-red、pinned-apparatus-revision-green、pinned-apparatus-revision-red、pinned-apparatus-revision-unresolved-red
#     pinned-apparatus-revision-inplace-rewrite-red、pinned-apparatus-revision-internal-whitespace-red、pinned-apparatus-revision-duplicated-line-red
#     pinned-apparatus-revision-repin-only-red、pinned-apparatus-revision-repin-only-green、pinned-apparatus-revision-pin-trailing-comment-red
#     pinned-apparatus-revision-registry-mentioned-after-another-red、pinned-apparatus-revision-registry-unlabeled-red、pinned-apparatus-revision-added-line-without-pin-red
#     pinned-apparatus-revision-pin-in-other-section-red、pinned-apparatus-revision-archived-history-red、pinned-apparatus-revision-archived-history-green
#     pinned-apparatus-revision-pin-prefix-green
#     pinned-apparatus-revision-older-line-rewritten-red、pinned-apparatus-revision-archived-revert-to-superseded-red
#     pinned-apparatus-revision-intree-repin-superseded-red、pinned-apparatus-revision-revert-with-new-line-green、pinned-apparatus-revision-registry-renamed-red
#     evidence-in-repo-exact-only-rerun-red、evidence-in-repo-unrelated-file-red、evidence-in-repo-timing-rows-red、evidence-in-repo-timing-rows-green
#     evidence-in-repo-crates-bin-red、evidence-in-repo-crates-bin-green
#     evidence-in-repo-suffix-timing-row-red、evidence-in-repo-timing-gz-green、evidence-in-repo-timing-gz-red
#     preregistration-parallelism-git-error-red、preregistration-parallelism-green、preregistration-parallelism-red
#     preregistration-parallelism-wallclock-green、preregistration-parallelism-wallclock-red、quoted-result-lines-archived-gz-green、quoted-result-lines-archived-gz-red
#     preregistration-parallelism-quantities-section-green、preregistration-parallelism-quantities-section-red
#     preregistration-judge-features-green、preregistration-judge-features-red、preregistration-judge-features-prose-green
#     preregistration-judge-features-continued-green、preregistration-judge-features-continued-red、preregistration-judge-features-alias-red
#     preregistration-judge-features-comment-swallows-red、preregistration-judge-features-comment-hides-red、preregistration-judge-features-comment-control-red
#     preregistration-judge-features-trailing-comment-red、preregistration-judge-features-continued-outside-fence-red、preregistration-judge-features-spellings-green
#     preregistration-judge-features-test-target-red、judge-entries-in-sync-green、judge-entries-in-sync-red、judge-entries-in-sync-stale-red
#     judge-entries-in-sync-array-parameter-new-entry-red、judge-entries-in-sync-array-parameter-on-the-path-green、judge-entries-in-sync-cycle-new-entry-after-red
#     preregistration-judge-features-list-item-trailing-comment-red、preregistration-judge-features-table-cell-trailing-comment-red
#     quoted-result-lines-green、quoted-result-lines-normalized-green、quoted-result-lines-gz-green、quoted-result-lines-gz-red、quoted-result-lines-history-added-red
#     quoted-result-lines-history-old-green、quoted-result-lines-moved-and-edited-red、quoted-result-lines-moved-history-green、quoted-result-lines-moved-unregistered-red
#     quoted-result-lines-unreplayable-green、quoted-result-lines-unreplayable-mistyped-red、quoted-result-lines-unreplayable-product-missing-red
#     quoted-result-lines-unreplayable-reason-empty-red、quoted-result-lines-unreplayable-unlisted-red
#     quoted-result-lines-red、replay-comparison-units-exclusion-owed-red、replay-comparison-units-green、replay-comparison-units-red
#     replay-comparison-units-rekey-red、replay-comparison-units-table-green、replay-comparison-units-table-red、replay-comparison-units-zero
#     repro-command-apparatus-green、repro-command-apparatus-red、repro-command-green、repro-command-gz-green、repro-command-gz-red、repro-command-red
#     repro-command-unreproducible-green、repro-command-unreproducible-page-missing-red、repro-command-unreproducible-reason-empty-red
#     results-cited-green、results-cited-gz-green、results-cited-gz-history-green、results-cited-gz-red
#     results-cited-red、verdict-false-named-green、verdict-false-named-gz-green、verdict-false-named-gz-red
#     verdict-false-named-no-history-red、verdict-false-named-red、verdict-false-named-translation-green、verdict-false-named-zero
#   KB_NUMBERED_FILE_BREAK=first-match  归 lib-item-ref-status.py：按编号取实验页不分语言、取排序后的第一个命中（取到 <号>-<名>.en.md）
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
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=progress-mark-counted  decision-links 量索引表里写结论那一格的字数时把行末「**进度：…**」也算进去
#     decision-links-progress-mark-green
#     decision-links-progress-mark-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=deleted-products-counted  decision-links 把这次改动里删掉的 research/results/e<号>… 也算「产物变了」
#     decision-links-deleted-product-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=read-translations  decision-links 的决策正文与实验页连译文一起读
#     decision-links-translation-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=language-switch-counted  decision-links 把实验页顶上第三行的语言切换行也算正文：只改了、只新插了这一行的页照「正文改了」要回看
#     decision-links-language-switch-green
#     decision-links-moved-and-switch-green
#     decision-links-switch-inserted-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=inserted-switch-counted  decision-links 去掉切换行时留着紧跟它的空行（只认两版第三行都是切换行、只是改了）：新插切换行加一个空行的页照「正文改了」要回看
#     decision-links-switch-inserted-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=moves-not-rewritten  decision-links 比两版正文时基准那一版不照此刻生效的搬迁登记换算（git show 原样读）
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
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-features-unchecked  preregistration-judge-features 不看特性，走判段的命令一律当带了 gpu
#     preregistration-judge-features-alias-red
#     preregistration-judge-features-comment-control-red
#     preregistration-judge-features-comment-hides-red
#     preregistration-judge-features-comment-swallows-red
#     preregistration-judge-features-continued-outside-fence-red
#     preregistration-judge-features-continued-red
#     preregistration-judge-features-list-item-trailing-comment-red
#     preregistration-judge-features-red
#     preregistration-judge-features-table-cell-trailing-comment-red
#     preregistration-judge-features-test-target-red
#     preregistration-judge-features-trailing-comment-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-binaries-all-counted  preregistration-judge-features 不读源文件，点名 --bin 的命令一律当走判段
#     preregistration-judge-features-alias-red
#     preregistration-judge-features-green
#     preregistration-judge-features-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-commands-anywhere  preregistration-judge-features 不认命令位置，行文里内嵌的 cargo run / cargo test 也判
#     preregistration-judge-features-prose-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=continuation-unjoined  preregistration-judge-features 不并反斜杠续行，逐行认命令（围栏里与围栏外都不并；拆到下一行的 --bin 漏判、--features 误红）
#     preregistration-judge-features-continued-green
#     preregistration-judge-features-continued-outside-fence-red
#     preregistration-judge-features-continued-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-by-module-marker  preregistration-judge-features 按源文件里有没有 crash_amplification:: 或 crash_state_enumeration:: 认走判段（别名导入漏判、只取崩溃点的误判）
#     preregistration-judge-features-alias-red
#     preregistration-judge-features-continued-red
#     preregistration-judge-features-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=shell-comments-ignored  preregistration-judge-features 不认 shell 注释：以反斜杠收尾的注释行照续行、续行接到整行注释照并、命令行尾的注释照当命令的一部分
#     preregistration-judge-features-comment-hides-red
#     preregistration-judge-features-comment-swallows-red
#     preregistration-judge-features-list-item-trailing-comment-red
#     preregistration-judge-features-table-cell-trailing-comment-red
#     preregistration-judge-features-trailing-comment-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=features-spelling-narrow  preregistration-judge-features 认特性时命令按空白切词、不认引号，-F 只认与值分开写的（-Fgpu、--features "lookup-work-counters gpu" 误红）
#     preregistration-judge-features-spellings-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=test-targets-unjudged  preregistration-judge-features 只认 --bin，cargo test 只点名 --test 的不判（只报数）
#     preregistration-judge-features-test-target-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=trailing-comments-kept-outside-fence  preregistration-judge-features 只截围栏里命令行尾的 shell 注释（列表项里行首是命令的行、表格整格的命令里，注释里写的 --features gpu 算带了）
#     preregistration-judge-features-list-item-trailing-comment-red
#     preregistration-judge-features-table-cell-trailing-comment-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-entries-new-unreported  judge-entries-in-sync 不报源码里认出而 JUDGE_ENTRIES 里没有的入口
#     judge-entries-in-sync-array-parameter-new-entry-red
#     judge-entries-in-sync-cycle-new-entry-after-red
#     judge-entries-in-sync-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-entries-stale-unreported  judge-entries-in-sync 不报 JUDGE_ENTRIES 里有而源码里认不出的名字
#     judge-entries-in-sync-stale-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-entries-header-after-any-semicolon  judge-entries-in-sync 切函数单元时表头从上一个分号或右花括号算起，方括号里的也算（签名里带 [u8; N] 的函数不进单元）
#     judge-entries-in-sync-array-parameter-new-entry-red
#     judge-entries-in-sync-array-parameter-on-the-path-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=judge-entries-cycle-memoized  judge-entries-in-sync 递归走图、记住每个单元的结果，环上截断时算出的「走不到」也记住（结局随函数在文件里的先后翻）
#     judge-entries-in-sync-cycle-new-entry-after-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-read-raw  verdict-false-named 与 quoted-result-lines 读 .out.gz 不解压、按原字节读
#     quoted-result-lines-archived-gz-green
#     quoted-result-lines-archived-gz-red
#     quoted-result-lines-gz-green
#     quoted-result-lines-gz-red
#     verdict-false-named-gz-green
#     verdict-false-named-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=named-check-skipped  verdict-false-named 把每个 false 字段都当成已点名
#     verdict-false-named-gz-red
#     verdict-false-named-no-history-red
#     verdict-false-named-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=on-request-unreported  evidence-in-repo 的成功句之后不报这一趟放过了哪几个按要求才跑的实验
#     evidence-in-repo-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=change-range-error-dropped  取改动范围失败的格不把父进程里记下的 git 报错（CHANGE_RANGE_ERROR_FILE）打进自己的输出
#     preregistration-parallelism-git-error-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=replay-exact-ignored  evidence-in-repo 不认复跑表里的 exact 行：装置比产物新一律照修改时刻判红
#     evidence-in-repo-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=replay-exact-by-experiment  evidence-in-repo 按实验号认复跑表的 exact 行：同一个实验另有 timing 行、装置改了而产物旧，也按复跑表放过
#     evidence-in-repo-exact-only-rerun-red
#     evidence-in-repo-mixed-replay-red
#     evidence-in-repo-suffix-timing-row-red
#     evidence-in-repo-unrelated-file-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=timing-rows-ignored  evidence-in-repo 有 timing 行的实验也比这个实验号在盘最新的那一份产物（exact 行另存的新产物、不相干的 e<号> 文件都算数，红句照没有 timing 行的写「最新的」）
#     evidence-in-repo-exact-only-rerun-red
#     evidence-in-repo-mixed-replay-red
#     evidence-in-repo-red
#     evidence-in-repo-suffix-timing-row-red
#     evidence-in-repo-timing-gz-red
#     evidence-in-repo-timing-rows-red
#     evidence-in-repo-unrelated-file-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=timing-newest-row-only  evidence-in-repo 有几条 timing 行的实验只比点名的产物里最新的那一份，不在盘的与更旧的几份不算
#     evidence-in-repo-timing-rows-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=replay-key-suffix-unattributed  evidence-in-repo 复跑表行键去掉 E 整个当实验号（E926R2 这种带重跑后缀的行归不到 E926）
#     evidence-in-repo-suffix-timing-row-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=timing-gz-unrecognized  evidence-in-repo 找 timing 行点名的产物只认原名，只存着 <名>.gz 的当不在盘
#     evidence-in-repo-timing-gz-green
#     evidence-in-repo-timing-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=crates-binaries-unrecognized  evidence-in-repo 不认 crates/*/src/bin/ 下以 e<号>_ 起头的装置（只认 scripts/experiment-apparatus/）
#     evidence-in-repo-crates-bin-green
#     evidence-in-repo-crates-bin-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-citation-unmatched  results-cited 第一道要正文逐字点名 .out.gz、第三道只查原样的 .out（不查树里、HEAD 里、历史里的 .out.gz）
#     cited-paths-exist-gz-green
#     cited-paths-exist-gz-red
#     results-cited-gz-green
#     results-cited-gz-history-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=archived-gz-not-searched  quoted-result-lines 去历史里找归档产物时只认 .out 收尾的路径
#     quoted-result-lines-archived-gz-green
#     quoted-result-lines-archived-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=archived-blob-batch-unread  quoted-result-lines 从历史里批量取归档产物时（git cat-file --batch 那一路）取到的字节读成空：取得到的份数不变、里面一行都没有
#     quoted-result-lines-archived-gz-green
#     quoted-result-lines-archived-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=result-lines-raw-prefix  quoted-result-lines 收产物行时不去首尾空白就筛 E7RESULT 前缀（缩进的产物行收不进来）
#     quoted-result-lines-normalized-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=result-lines-prefilter-ignores-carriage-return  quoted-result-lines 产物里有回车也先筛含「E7RESULT 加一个空格」的行（回车隔在 E7RESULT 与空格之间的行收不进来）
#     quoted-result-lines-normalized-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=repro-command-batch-unattributed  repro-command 批量 grep 的命中不分页：有一页写明了原始输出未留存（或写了复跑入口），就当每一页都写了
#     repro-command-green
#     repro-command-unreproducible-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=tmp-prefilter-skips-all  evidence-in-repo 判据二的整份预筛一律说文本里没有 /tmp/，所有 .md 都不扫
#     evidence-in-repo-green
#     evidence-in-repo-red
#     evidence-in-repo-worktree-green
#     evidence-in-repo-worktree-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unreplayable-registry-ignored  quoted-result-lines 不读 .claude/gate.d/quoted-product-unreplayable.tsv，登记了的产物照旧只认 replay.sh
#     quoted-result-lines-unreplayable-green
#     quoted-result-lines-unreplayable-mistyped-red
#     quoted-result-lines-unreplayable-product-missing-red
#     quoted-result-lines-unreplayable-reason-empty-red
#     quoted-result-lines-unreplayable-unlisted-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unreplayable-product-existence-unchecked  quoted-result-lines 不查登记表里的产物在不在（树里与历史里都找不到也当在）
#     quoted-result-lines-unreplayable-product-missing-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unreplayable-reason-unchecked  quoted-result-lines 不查登记行的怎么跑出的、为什么进不了复跑表两列写没写
#     quoted-result-lines-unreplayable-reason-empty-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=gz-name-not-product  repro-command 不把 .out.gz 当产物名（只认 .out 收尾的）
#     repro-command-gz-green
#     repro-command-gz-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=apparatus-lines-unrecognized  repro-command 不认起 scripts/experiment-apparatus/ 下 python3 或 bash 脚本的复跑行（只认 cargo run 与 replay.sh、vm-bench.sh）
#     repro-command-apparatus-green
#     repro-command-apparatus-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=apparatus-script-existence-unchecked  repro-command 不查复跑行点名的脚本与装置目录在不在，点名了就算存在
#     repro-command-apparatus-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unreproducible-registry-ignored  repro-command 不读 .claude/gate.d/repro-command-unreproducible.tsv，登记了的页照判
#     repro-command-unreproducible-green
#     repro-command-unreproducible-page-missing-red
#     repro-command-unreproducible-reason-empty-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unreproducible-page-existence-unchecked  repro-command 不查登记行指到的实验页在不在
#     repro-command-unreproducible-page-missing-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=unreproducible-reason-unchecked  repro-command 不查登记行有没有写原因
#     repro-command-unreproducible-reason-empty-red
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
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-none-key-pair  replay-comparison-units 的只缩不涨按（登记出处，留存产物名）成对比，换实验号、换单元名借同一对键照绿
#     replay-comparison-units-rekey-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-table-unchecked  replay-comparison-units 不解析重跑登记里的比对对象表、不与登记表逐行对
#     replay-comparison-units-exclusion-owed-red
#     replay-comparison-units-table-green
#     replay-comparison-units-table-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-table-suffix-unrecognized  replay-comparison-units 对重跑产物名只写前缀之后一段的（旧登记的写法）不认「留存产物名以 -<那一段> 收尾」
#     replay-comparison-units-table-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-exclusions-ignored  replay-comparison-units 不把登记表文件头「不登记的比对对象」那几行算点名（写法照判，只是不拿来放过表里的行）
#     replay-comparison-units-exclusion-owed-red
#     replay-comparison-units-table-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=comparison-exclusion-owed-unchecked  replay-comparison-units 不判点名不登记的理由指没指到开着的欠账
#     replay-comparison-units-exclusion-owed-red
#     replay-comparison-units-table-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-registry-ignored  experiment-orphans 不读目录登记，用写死的四个目录（不看 kb 索引行，也不认登记里新加的一处）
#     experiment-orphans-registry-red
#     experiment-orphans-blank-beside-body-red
#     experiment-orphans-empty-page-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-page-shape-ignored  experiment-orphans 的「有正文」只看 kb/experiments/ 底下条目名的数字前缀（同号的目录、.bin、空 .md 都算有正文）
#     experiment-orphans-blank-beside-body-red
#     experiment-orphans-empty-page-red
#     experiment-orphans-page-binary-red
#     experiment-orphans-page-directory-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-batch-any-body  experiment-orphans 批量找有正文的页时只要有一页有非空白字符，就当每一页都有正文
#     experiment-orphans-blank-beside-body-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-revision-unchecked  pinned-apparatus-revision 钉值或装置源码变了也不核登记第十二节（新加的行、基准那一版的行、钉值写在哪都不看）
#     pinned-apparatus-revision-added-line-without-pin-red
#     pinned-apparatus-revision-archived-history-red
#     pinned-apparatus-revision-archived-revert-to-superseded-red
#     pinned-apparatus-revision-duplicated-line-red
#     pinned-apparatus-revision-inplace-rewrite-red
#     pinned-apparatus-revision-internal-whitespace-red
#     pinned-apparatus-revision-intree-repin-superseded-red
#     pinned-apparatus-revision-older-line-rewritten-red
#     pinned-apparatus-revision-pin-in-other-section-red
#     pinned-apparatus-revision-pin-trailing-comment-red
#     pinned-apparatus-revision-red
#     pinned-apparatus-revision-registry-mentioned-after-another-red
#     pinned-apparatus-revision-repin-only-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-base-ignored  pinned-apparatus-revision 不取基准那一版的登记，第十二节里每一行都当新加的、基准那一版的行一行都不比
#     pinned-apparatus-revision-added-line-without-pin-red
#     pinned-apparatus-revision-duplicated-line-red
#     pinned-apparatus-revision-inplace-rewrite-red
#     pinned-apparatus-revision-internal-whitespace-red
#     pinned-apparatus-revision-older-line-rewritten-red
#     pinned-apparatus-revision-pin-in-other-section-red
#     pinned-apparatus-revision-pin-trailing-comment-red
#     pinned-apparatus-revision-red
#     pinned-apparatus-revision-registry-mentioned-after-another-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-whole-registry-counted  pinned-apparatus-revision 新加的行不限第十二节，登记里别的节新加的行也算
#     pinned-apparatus-revision-pin-in-other-section-red
#     pinned-apparatus-revision-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-bin-flag-unresolved  pinned-apparatus-revision 认装置源码只认 apparatus_source= 一行，不认 cargo 命令的 --bin
#     pinned-apparatus-revision-green
#     pinned-apparatus-revision-red
#     pinned-apparatus-revision-unresolved-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-archived-judged  pinned-apparatus-revision 登记已归档（不在树里）的也当判红
#     pinned-apparatus-revision-green
#     pinned-apparatus-revision-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-unresolved-ignored  pinned-apparatus-revision 认不出装置源码或登记的起跑脚本悄悄跳过，不判红
#     pinned-apparatus-revision-registry-unlabeled-red
#     pinned-apparatus-revision-unresolved-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-change-untriggered  pinned-apparatus-revision 钉值变了不触发，只有装置源码变了才核登记
#     pinned-apparatus-revision-intree-repin-superseded-red
#     pinned-apparatus-revision-repin-only-green
#     pinned-apparatus-revision-repin-only-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-any-added-line  pinned-apparatus-revision 第十二节新加的行里有没有出现新钉值不看，有一行新加的就算修订
#     pinned-apparatus-revision-added-line-without-pin-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-copied-line-counted  pinned-apparatus-revision 第十二节新加的行与基准那一版第十二节里已有的行逐字相同（去首尾空白）也算写着钉值的修订（把已有的一行再抄一遍放行）
#     pinned-apparatus-revision-duplicated-line-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-base-lines-rewritable  pinned-apparatus-revision 基准那一版第十二节里的行被改掉或删掉也放行
#     pinned-apparatus-revision-inplace-rewrite-red
#     pinned-apparatus-revision-internal-whitespace-red
#     pinned-apparatus-revision-older-line-rewritten-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-base-lines-pin-bearing-only  pinned-apparatus-revision 基准那一版第十二节里只有写着基准钉值的行要原样在，别的行原地改写放行
#     pinned-apparatus-revision-older-line-rewritten-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-last-value-unchecked  pinned-apparatus-revision 不比新钉值与基准钉值在第十二节里谁写得更后（钉回已被后续修订作废的值放行，在树里与已归档的都放）
#     pinned-apparatus-revision-archived-revert-to-superseded-red
#     pinned-apparatus-revision-intree-repin-superseded-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-last-value-first-occurrence  pinned-apparatus-revision 比新钉值与基准钉值谁写得更后时各取第一次出现的位置（新加一行写回旧值的合规回退判红）
#     pinned-apparatus-revision-revert-with-new-line-green
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-rename-unrecognized  pinned-apparatus-revision 登记不在树里时不看它是不是被 git mv 改名走的，拿改名之前那一版当已归档的判
#     pinned-apparatus-revision-registry-renamed-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-trailing-comment-unrecognized  pinned-apparatus-revision 钉值行带行尾注释的起跑脚本不进对象
#     pinned-apparatus-revision-pin-trailing-comment-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-registry-first-mention  pinned-apparatus-revision 登记取注释里头一处点名的 prereg 的 .md，不看是不是写成「跑前登记 <路径>」
#     pinned-apparatus-revision-registry-mentioned-after-another-red
#     pinned-apparatus-revision-registry-unlabeled-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-archived-history-unread  pinned-apparatus-revision 登记不在树里就当已归档不判，不去基准与 git 历史里取它被删之前的那一版
#     pinned-apparatus-revision-archived-history-green
#     pinned-apparatus-revision-archived-history-red
#     pinned-apparatus-revision-archived-revert-to-superseded-red
#   EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=pin-prefix-whole-value  pinned-apparatus-revision 登记里要出现整串 64 位的钉值，只写前 16 位的不算
#     pinned-apparatus-revision-pin-prefix-green
#   滞后表、豁免表与基准比的取法与复跑表的读法在 lib-lag-baseline.py（decision-links、multipath-registry、replay-comparison-units 三格取基准那一版，
#   results-cited、verdict-false-named、quoted-result-lines、replay-comparison-units 四格读复跑表）；它的开关在这一道会红的样本：
#   LAG_BASELINE_BREAK=unreadable-as-absent  归 lib-lag-baseline.py：git 取不到基准那一版当成「基准里没有这张表」，decision-links 把待回填清单整张算新加
#     decision-links-pending-base-unreadable-red
#   LAG_BASELINE_BREAK=moved-only-kept  归 lib-lag-baseline.py：与基准只差登记搬迁的文件与行照旧算改动（这一道开头经 changed-paths.sh 取的改动范围与新增行、quoted-result-lines 历史节的新增行都不剔）
#     quoted-result-lines-moved-history-green
#
#   bash .claude/gate.d/doc-experiments.sh [项目根]                              十六格按格名表的次序全跑
#   bash .claude/gate.d/doc-experiments.sh --list                                逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-experiments.sh --check <格名>[,<格名>…] [项目根]     只跑点名的格
#   格名写错、给了两个项目根、认不出的选项、进不去项目根都退 2。
#
# 格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的几份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析与汇总。格的出路里写的「doc-experiments.sh 里这一格函数上方的注释」，就在下面对应的那份库里（格在哪份库按格名列在下面）：
# 库 lib/doc-experiments-index-and-evidence-cells.sh：index-sync、results-cited、evidence-in-repo、decision-links
# 库 lib/doc-experiments-verdict-and-citation-cells.sh：verdict-false-named、repro-command、experiment-orphans、quoted-result-lines、archive-past-rounds
# 库 lib/doc-experiments-verdict-and-citation-cells.sh：multipath-registry、cited-paths-exist
# 库 lib/doc-experiments-preregistration-and-replay-cells.sh：preregistration-parallelism、preregistration-judge-features、judge-entries-in-sync
# 库 lib/doc-experiments-preregistration-and-replay-cells.sh：replay-comparison-units、pinned-apparatus-revision
# 弄坏开关的读法在入口的赋值与格函数里，格函数在 lib/ 下的库里；开关表仍在上面文件头。下面每个开关变量在哪份库的代码里被读，按库列在这里：
# 库 lib/doc-experiments-index-and-evidence-cells.sh：EXPERIMENT_PAGES_AND_PRODUCTS_BREAK
# 库 lib/doc-experiments-verdict-and-citation-cells.sh：EXPERIMENT_PAGES_AND_PRODUCTS_BREAK
# 库 lib/doc-experiments-preregistration-and-replay-cells.sh：EXPERIMENT_PAGES_AND_PRODUCTS_BREAK
# 入口的代码里也读的：EXPERIMENT_PAGES_AND_PRODUCTS_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：KB_NUMBERED_FILE_BREAK、LAG_BASELINE_BREAK
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ doc-experiments.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi

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
# 要「这次改动」的十格：选中的格里有它们之一，才去取改动范围
CHANGE_RANGE_CELLS=" results-cited evidence-in-repo decision-links quoted-result-lines archive-past-rounds multipath-registry preregistration-parallelism preregistration-judge-features replay-comparison-units pinned-apparatus-revision "

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
  "在实验页正文的口径一节补一句复跑命令（格式与别处一致；纯 Python 或 bash 的装置写起它的 task-server/task submit 整条，点名的脚本要在仓里）；原始输出确实没留的写明「原始输出未留存」；产物逐字节复跑不了的在 .claude/gate.d/repro-command-unreproducible.tsv 登一行（页文件名、原因）"
stage_cell experiment-orphans cell_experiment_orphans "目录登记列的各处出现过的实验号在 kb 里有没有正文" \
  "给这个实验号建正文 .claude/kb/experiments/<号>-*.md（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；实验号住在哪几处照 .claude/gate.d/experiment-number-directories.tsv，写法在它的文件头"
stage_cell quoted-result-lines cell_quoted_result_lines "kb 与 research 正文里整行抄的产物行（含历史节里这次新增的行），产物里逐字找得到" \
  "打开这个实验复跑命令登记的那份产物，把对应的 E7RESULT 行原样抄回来，别手改数；产物换了就连同引它的正文一起换；这一轮分片跑出、进不了 replay.sh 复跑表的产物，在 .claude/gate.d/quoted-product-unreplayable.tsv 登一行（产物名、怎么跑出的、为什么进不了复跑表）"
stage_cell archive-past-rounds cell_archive_past_rounds "上一轮及更早的实验记录已经归档进版本库，工作区里不留" \
  "照这一格打出来的基准跑 python3 scripts/kb/archive-past-rounds.py --apply 删掉上一轮的记录（不带基准会把这一次提交要带的也删掉）"
stage_cell multipath-registry cell_multipath_registry "多条路径互证的实验，路径与共用项要登记且指得到" \
  "在实验页写「### 路径与结论登记」一节（形态见 .claude/rules/format-evolution.md），源码落点写现存路径、共用项写实；multipath-registry-lag.tsv 只缩不涨，补齐的删行"
stage_cell cited-paths-exist cell_cited_paths_exist "kb 全部正文点名的实验产物、变异表、实验源码路径，在树里或版本库历史里找得到" \
  "补回那份文件；或把那一行改成写明「原件未留存、从哪次提交取」；或改成现存的路径（移过位置的用 git log --all --diff-filter=A --name-only -- '*<文件名>' 找新路径）"
stage_cell preregistration-parallelism cell_preregistration_parallelism "这次改动新建或改动的 E186 起的跑前登记有「并行与挂钟」一节，独立单位、线程、串行与为什么、预计挂钟、预计在用核数五样写全，预计挂钟与态数乘单态耗时差不到一个数量级" \
  "照 .claude/agents/experiment-designer.md「登记的固定节名」的「## 十四、并行与挂钟」一行在登记里补这一节，五个标签行各一行（「独立单位：」「线程：」「串行：」「预计挂钟：」「预计在用核数：」，冒号后至少 8 个字，两行预计要带数、推的写依据）；预计挂钟与态数乘单态耗时差一个数量级以上的，照设计员定义第 3a 步现算后改挂钟或改错了的数，并行跑的写成式子「<态数> × <单态耗时> ÷ <并行数> ≈ <挂钟>」"
stage_cell preregistration-judge-features cell_preregistration_judge_features "这次改动新建或改动的跑前登记里命令位置上（围栏里的行、行首是命令的行、表格里整格是命令的格；行文里内嵌的不判）起装置的 cargo run / cargo test，反斜杠续行先并成一条（shell 注释不续行），点名的 --bin（cargo test 另认 --test）目标走崩溃判段（源文件调了 crash_amplification、crash_state_enumeration 的判段入口，别名导入照认）的特性要带 gpu" \
  "照 .claude/agents/experiment-designer.md 写起法：走到崩溃判段的装置二进制，登记里的 cargo run / cargo test 带 --features gpu（经 task-server/task submit 交，清场由服务做）；认不出的命令写成 -p <包名> --bin <名>（cargo test 也可 --test <名>）"
stage_cell judge-entries-in-sync cell_judge_entries_in_sync "判段入口表 JUDGE_ENTRIES 与 crash_amplification.rs、crash_state_enumeration.rs 里认出的公开判段入口逐个相同" \
  "源码里新加的公开判段入口把名字加进 .claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh 的 JUDGE_ENTRIES（方法写类型名）；表里认不出的，改名的换成新名字、删了或不再走判段的从表里删；认法在这一格函数上方的注释"
stage_cell replay-comparison-units cell_replay_comparison_units "重跑登记的比对对象逐行登记在机读表里：登记出处在、单元名在单元表出处里恰好出现一次、复跑驱动在 replay.sh 里且产物名相符；重跑登记里表头逐字的比对对象表与机读表逐行对得上" \
  "照 .claude/gate.d/replay-comparison-units.tsv 文件头的列义补齐或改正那一行（单元名照单元表里的字符串字面量逐字抄，复跑驱动与产物名照 scripts/experiment-apparatus/replay.sh 复跑表那一行抄）；确实没有复跑行的写 none:理由（复跑写 none: 的行与基准那一版比只缩不涨，新登记的比对对象要在 replay.sh 补驱动行）；重跑登记比对对象表里的每一行要有一行登记对上，单元表里没有单元的在那份文件头写一行「# 不登记的比对对象：<登记出处> #<号>：<理由>」"
stage_cell pinned-apparatus-revision cell_pinned_apparatus_revision "起跑脚本钉了装置 sha256（registered_apparatus_sha256=）的，钉值或装置源码在这次改动里变了，就核它的跑前登记第十二节：装置源码变了的要新加一行写着新钉值的修订，只改钉值的要写着新钉值；基准那一版第十二节的行都原样在，新钉值写在基准钉值之后（钉回已作废的值不算），登记被改名走的判红" \
  "补登记第十二节一行修订（新 sha256、改了什么）：在红行点名的登记里「## 十二、…」一节末尾新写一行，写改完的装置 sha256（sha256sum <装置源码>）与改了什么，再把起跑脚本的 registered_apparatus_sha256 钉成同一个值（只改钉值的也要先写这一行，回退到旧值的同样新写一行，已有的行不改写）；登记被 git mv 改名的，把起跑脚本注释里「跑前登记 <路径>」改成改名之后的路径；认不出装置源码的起跑脚本加一行 apparatus_source=<装置源码路径>，认不出登记的在文件头注释里写成「跑前登记 research/prompts/<登记文件名>」"
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

# ── 改动范围：经共用库 scripts/gate/changed-paths.sh 只取一次（基准 gate_diff_base gate），十格共用。
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
CHANGE_RANGE_ERROR_FILE="$CHANGE_RANGE_DIRECTORY/change-range-error"
: > "$SHARED_CHANGED_PATHS"; : > "$SHARED_ADDED_PATHS"; : > "$SHARED_ADDED_LINES"; : > "$CHANGE_RANGE_ERROR_FILE"
# 取改动范围时 gate_changed_paths 与 git 写到标准错误的报错（哪一条 git、退几、git 的原文）：下面在父进程里取，各格的子 shell 里没有那段标准错误，
# 所以接进 CHANGE_RANGE_ERROR_FILE，取不到的失败分支调这里原样打出来。
print_change_range_error() {
  [[ "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" == change-range-error-dropped ]] && return 0   # 弄坏开关：不打预取记下的报错
  if [[ -s "$CHANGE_RANGE_ERROR_FILE" ]]; then
    cat -- "$CHANGE_RANGE_ERROR_FILE"
  else
    echo "  ! 取改动范围时没有留下 git 的报错（共用库没读进来，或没取就退了）"
  fi
}
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
      if gate_changed_paths "$shared_base" untracked > "$SHARED_CHANGED_PATHS" 2>> "$CHANGE_RANGE_ERROR_FILE"; then changed_paths_ok=1; fi
      if gate_changed_paths "$shared_base" untracked A > "$SHARED_ADDED_PATHS" 2>> "$CHANGE_RANGE_ERROR_FILE"; then added_paths_ok=1; fi
      # research/ 只取 .md：未跟踪的产物可能很大，整份读一遍白费
      if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$SHARED_ADDED_LINES" 2>> "$CHANGE_RANGE_ERROR_FILE"; then added_lines_ok=1; fi
    fi
  fi
fi

# 按格组 source 下面几份库；它们读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取
source "$STAGE_DIR/lib/doc-experiments-index-and-evidence-cells.sh"
source "$STAGE_DIR/lib/doc-experiments-verdict-and-citation-cells.sh"
source "$STAGE_DIR/lib/doc-experiments-preregistration-and-replay-cells.sh"

# 这一道的格互不依赖、都只读被判的树：并行跑（.claude/gate.d/lib/stage-cells.sh 文件头「格并行」）
STAGE_CELLS_JOBS="${STAGE_CELLS_JOBS:-$(stage_cells_default_jobs)}"
stage_cells_run "$ROOT"
