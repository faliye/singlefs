#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录；外部引用那一格调的 scripts/kb/verify-citations.sh 要本机固定下来的外部文献树，缺了在那一格判红，不交给 gate.sh 预判
# gate-stage: 登记表与引用（二十四格：字段与布局登记表之间的账；欠账表与收口表的形状与去向；实验号、决策号、门禁名、路径、小节与外部引用指得到；不变量条数与字段落点；kb 正文里的内联算式左右相等；未实现的不变量写了编号前置；字段频率「每 X 一份」的 X 指得到已定条款给出的大小）
# gate-fixture: yes
# gate-category: 文档类
# gate-similar: link-targets.py 它只解析 ](相对路径) 链接与「第 N 节」，不看反引号里的路径、「门禁 N 号」与「小节」名；这三类与 experiment-refs、decision-refs 两格同是「引用了不存在的东西」，并在这里
# gate-similar: doc-decisions.sh 它判决策文档本身（索引结论字数、分项引用状态、未定项写没写改不改新池新建文件的字节、用户定案改动记没记欠账）；这一道判字段表指向的分项存不存在、判「是」的分项在字节表与里程碑里挂没挂钩、认购表求和、欠账表与收口表的形状与去向，decision-refs 只判 D 号有定义
# gate-similar: checker-independence-and-sync.sh 它的 layout-checker-sync 格判 layouts.tsv 登记的布局常量变了 checker 跟没跟，对象是布局清单与 checker 源码；这一道只对 kb 各表之间、kb 与 E142 产物和钉住的用例之间的账
# gate-similar: code-source-discipline.sh 它的 reading-discipline、mutation-tables、format-constants、clause-enums、feature-bits 几格方向是 kb 到代码，拿登记值核 crates 源码；这一道的 field-table-sums 与它共用 lib-format-const.py 读标记但只判 kb 里的加和与登记，format-const-placeholders 只核代码注释里的编号指得到 kb，closeout-row27-preconditions 只数几段写死的原文命中几次，都不拿 kb 的值比代码
# gate-similar: doc-experiments.sh 它的 multipath-registry 格判实验页登记表的形状，quoted-result-lines 格判 kb 整行抄的产物行在 research/results/ 里找得到（kb 到产物）；这一道 audit-contradictions 格判总审核处置列引的原文在 kb 今天的正文里找得到（记录到 kb），另判欠账号还开着
# gate-similar: doc-text.sh 它判 kb 与全仓文本的写法（角标、钟点、旧名），不核数；这一道的 inline-sums 格核 kb 正文里写出来的「整数 = 项 + 项」算式左右两边的数相等，field-table-sums 只看表后合计，两者都不并进 doc-text
#
# 三组格：字段与布局组九格（kb 字段表、字节布局表、段序列登记表与里程碑各步对账）、欠账与收口组五格（checks-owed.md 与里程碑收口表）、
# 引用与不变量组十格（引用指得到、不变量条数与字段落点、内联算式左右相等、未实现的不变量的前置、字段频率的大小；governance-refs 的判据在 lib-governance-refs.py 文件头）。
# 实验改成已跑后引用它的决策有没有回看、已跑的实验有没有对应的决策，不在这里判，归 doc-experiments.sh 的 decision-links 格。
# 格名表（--list 打它；--check <格名>[,<格名>…] 只跑点名的格，不给就按下面的次序全跑）：
# gate-cell: field-refs 新池新建文件的字段表，每个「指向」都指到一条真实存在的分项；预留字段指向的 D9 已定项 12 第几项与所属单元类一致
# gate-cell: field-projection 决策里定的字段有没有漏投影进新池新建文件的表
# gate-cell: field-table-sums 字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记
# gate-cell: first-txn-hooks 新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）
# gate-cell: admission-terms 准入不等式的每一项都有人维护（统计量，或写明的例外）
# gate-cell: segment-registry 段序列登记表与钉住活代码的用例逐字比对（钉不到用例的行与 E142 产物比，产物不在报本次未跑）
# gate-cell: format-const-placeholders 格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账
# gate-cell: second-txn-hooks 覆盖写、释放、回退与复用的三处互相挂钩（layout/02 登记表 / 里程碑各步「写出的字节」/ layout/01 段序列登记表）
# gate-cell: tree-table-reserve 树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
# gate-cell: table-shape 欠账表两张登记表的行形状（格数对得上表头、欠着的那张里不留写着已还的行、表不被空行断开、两张表都在）
# gate-cell: closeout-collects-open 里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号
# gate-cell: paid-cited-tests 已还清的欠账行里点名的测试名，仓里还在不在
# gate-cell: audit-contradictions 最近一次总审核登记的文档级矛盾，每一行都要有去向
# gate-cell: closeout-row27-preconditions 里程碑收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
# gate-cell: experiment-refs kb 与项目规则里引用的实验号都有定义
# gate-cell: decision-refs kb 与项目规则里引用的决策号都有定义
# gate-cell: declared-counts invariants.md 自己那句「现共 N 条在用」与表里条数对得上；欠账表数得出条数
# gate-cell: governance-refs 治理文档、kb 与仓根的 README.md、README.zh.md、README.ja.md 里按全名指的门禁与格都在、不用旧编号称呼门禁，治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、`文件「小节」` 在那份文件里找得到
# gate-cell: invariant-count-elsewhere invariants.md 之外的 kb 文件里写「N 条在用」的地方，N 等于实际在用条数
# gate-cell: citations kb 里承重的外部逐行引用还核得动（调 scripts/kb/verify-citations.sh）
# gate-cell: invariant-anchors 这次改动新写或改写的不变量行，点名它判的字段住在哪条已定分项
# gate-cell: inline-sums 「整数 = 项 + 项 + …」形态的内联算式（kb 正文里，每项是可选标签加一个整数），左右相等
# gate-cell: invariant-prerequisites 不变量表里状态是「未实现」的每一行都写了编号前置（C / D 分项 / E），指得到、而且还没全满足
# gate-cell: field-frequency-sizes 决策与布局正文里「每 X 一份」这种字段频率，X 指得到一条已定条款给出的大小（字面大小要等于已定的某个粒度）
#
# 参数解析、样本根的 .gate-cells、逐格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头）。退出码：
#   0  跑到的格没有红
#   1  有格红（退出码既不是 0 也不是 77），红格逐格接它登记的出路；建不了临时目录也退 1
#   2  认不出的选项、两个项目根、格名写错、进不去项目根
#   77 跑到的格全退 77，不记通过；退 77 的格逐格往 $GATE_NOT_RUN_FILE 报本次未跑
#
# 样本（fixtures/doc-registries.sh/，每份样本根的 .gate-cells 点名只跑哪几格；每一格的样本放了什么写在那一格的注释里）：
#   <格名>-red                  字段与布局组九格各一份，判红；expect 另要汇总里「<格名>：红（退出 1）」那一行
#   field-refs-reservation-owner-red    只跑 field-refs：journal 记录头表的算法类型行写 D9 已定项 12 第 4 项（该是第 1 项），点名那一行判红；指针头表写第 4 项、根记录表写第 1 项各自指对，不判；
#                               第四张表没点名单元类，报「!」不判
#   field-refs-reservation-owner-green  只跑 field-refs：三处各指对（指针头表第 4 项、journal 记录头表与根记录表第 1 项），报「核了 3 行预留字段的指向」
#   field-registry-green        字段与布局组九格的绿样本并在一棵树里，九格各报成功行、整道退 0
#   admission-terms-fullwidth-d-green   只跑 admission-terms：式子那一行写成全角括号的 `可用（d） = …`，抓式子的那一步半角全角都认，判绿
#   tree-table-reserve-wide-number-green  只跑 tree-table-reserve：出处指四位的欠账号 C1000 与三位的决策号 D105，编号位数不设上限，判绿
#   checks-owed-red             同一份欠账表喂欠账与收口五格，五格都红、各报自己点名的行，整道退 1；总审核第五节第 9 行按 I-7.4 点名不变量类文件
#                               （样本自己的 .claude/kb-directories 登记 crates/sample/invariants/），引文只在 I-8 那一类的文件里，audit-contradictions 点名两份路径判红
#   checks-owed-green           四格判绿、closeout-row27-preconditions 退 77，整道退 0 并报那一格本次未跑；总审核第五节第 6、7 行按 I- 号指到登记目录里的类文件，
#                               类号没有类文件的（I-10.1）指那个目录的 README.md
#   references-red              引用与不变量七格各放一处该红的对象，逐格点名判红；不变量表按类拆在样本自己登记（.claude/kb-directories）的
#                               crates/sample/invariants/ 下，README.md 放条数那句、故意不带登记标记
#   references-unreadable-red   悬空符号链接的 kb 文件，experiment-refs、decision-refs 要报出 grep 退 2
#   decision-refs-not-decisions-red    只跑 decision-refs：not-decisions 标记管的代码块之外又写了一处 D0、标记点名有定义的 D1、点名的 D7 在管的行里没有、
#                               一个编号都没点名的标记，逐条点名判红（写法与判法在 decision-refs 格上方的注释）
#   decision-refs-not-decisions-green  只跑 decision-refs：D0 只在标记管的代码块里（块里夹一个空行），判绿并报标记几条、豁免几处
#   references-green            七格判绿；setup.sh 建成 git 仓（新加一行带分项引用又 git add，只许算 1 行），verify-citations.sh 是退 0 打一行 ✓ 的替身；
#                               不变量表按类拆在样本自己登记的 crates/sample/invariants/ 下（两份类文件，README.md 放带登记标记的条数那句）
#   governance-refs-red         只跑 governance-refs：kb 与仓根三份 README（README.md、README.zh.md、README.ja.md）里用旧编号称呼门禁、不存在的门禁与格名，逐条点名判红
#   governance-refs-green       只跑 governance-refs：按全名指的门禁与格都在，历史节、变更史与记录文件名里的旧编号不判，报扫了几份、全名与格名几处
#   inline-sums-red             只跑 inline-sums：纯数字（84 = 42 + 8 + … 右边是 94）、粗体左边、带标签与「字节」、列表项里带反引号标签、登记目录 crates/sample/invariants 里的类文件，各一条左右不等，逐条点名判红；
#                               另有一条写对的不许点名，报「扫 2 份、判了 6 条」
#   inline-sums-green           只跑 inline-sums：写对的 5 条判绿；含 ×、含括号、含减号、连等的 5 条不判，带 KiB、ms 与小数的 3 条不判，代码围栏里、「## 历史版本」之后、notes-history.md 里的错式不扫，
#                               报判了 5 条、跳过 5 条含乘除减括号的、3 条带小数、其他单位或认不出写法的
#   invariant-prerequisites-red 只跑 invariant-prerequisites：一份不变量类文件（样本自己登记的 crates/sample/invariants/）放没写前置、前置只有散文、已还清的 C、
#                               欠账表里没有的 C、进度已实现的 D 分项、只写决策号、已跑的 E、索引里没有的未定项各一行，逐行点名判红；前置还开着的一行不点名
#   invariant-prerequisites-green  只跑 invariant-prerequisites：前置是开着的 C、没实现的已定项、开着的未定项、没跑的 E、一满足一没满足的两个编号，
#                               已实现与退役的行不判，报「未实现 5 行，前置 6 个编号」
#   field-frequency-sizes-red   只跑 field-frequency-sizes：删掉 D4 已定项 1 的 DATA_UNIT_BYTES 登记（C72 的判别力自证）、D8 已定项 2 没写出 NODE_BYTES 的 16384、
#                               字面 128 KiB、没写理由的豁免、不起作用的豁免，逐条点名判红
#   field-frequency-sizes-registration-red  只跑 field-frequency-sizes：DATA_UNIT_BYTES 登记在未定项小节里、NODE_BYTES 在 layout 里另登记 4096、
#                               D4 已定项 7 写的是待定，逐条点名判红
#   field-frequency-sizes-green 只跑 field-frequency-sizes：逻辑 extent、extent、单元、节点、索引节点各读得出大小，字面 32 KiB、带豁免的 512 字节、否定句判绿；
#                               代码围栏里、「## 历史版本」之后、*-history.md 与译本里的错写不扫，报「8 处……豁免 1 处」
#   按编号取文件的六份（D<n>、E<n> 有中文原文 <号>-<名>.md 与译文 <号>-<名>.en.md / .ja.md，中文这一侧取中文原文，判法 lib-item-ref-status.py 的 numbered_files）：
#   field-refs-translation-green        只跑 field-refs：D9 的两份译文排在中文原文前面、分项表标题是各自的语言；布局表引已定项 10 与已定项 12 的两行算法类型，
#                               中文原文都有，判绿，报「查了 3 处引用」「核了 2 行」（改前取到 09-样本加密.en.md，三处都报「那张表里没有这一条」、定案表读不出）
#   field-refs-translation-red          只跑 field-refs：布局表引 D9 已定项 11，中文原文没有、英语译文照中文的写法列着它，判红、点名「指向 D9（09-样本加密.md）的「已定项 11」」
#   format-const-placeholders-translation-green  只跑 format-const-placeholders：占位指 D23 已定项 19，中文原文有、两份译文的分项表标题是各自的语言，判绿
#   format-const-placeholders-translation-red    只跑 format-const-placeholders：占位指 D23 已定项 20，中文原文没有、英语译文照中文的写法列着它，判红、点名「D23 里找不到 已定项 20」
#   audit-contradictions-translation-green       只跑 audit-contradictions：总审核「已改」行点名 D9，引的原句在中文原文里、两份译文里没有，判绿
#   audit-contradictions-translation-red         只跑 audit-contradictions：引的原句中文原文里没有、只在日语译文里有，判红、点名「只在 decisions/09-样本决策.ja.md 里有」
#                               （改前把三语三份并起来找，判绿）
#   数条数只数中文原文的两份（判法 lib-item-ref-status.py 的 same_language_files）：
#   invariant-counts-translation-green  只跑 declared-counts、invariant-count-elsewhere：不变量表的中文原文 3 行、退役 1 行，实验页的英语、日语译文里各抄着两行 `| I-…`；
#                               中文侧「现共 2 条在用」、中文普查页「2 条不变量在用」与英语译文里的「2 条在用」都对得上，判绿，两格的成功行报没数的译文 2 份、4 行
#                               （改前把译文的 4 行也数进去，实际在用算成 6 条，两格都红）
#   invariant-counts-translation-red    同一套表，中文侧写的是把译文表行数进去的「现共 6 条在用」、日语译文里一句「5 条在用」：两格都红，
#                               declared-counts 点名「正文声称在用 6 条，实际 2 条」，invariant-count-elsewhere 点名那两句、都与中文原文数出的 2 条比
#
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/doc-registries.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   DOC_REGISTRIES_BREAK=field-refs  field-refs 格不跑、按判过了记
#       field-refs-red（期望退出 1，实测 0）；field-refs-reservation-owner-red（期望退出 1，实测 0）；field-refs-reservation-owner-green（输出里找不到「新池新建文件的字段表指向都成立…」）；
#       field-registry-green（输出里找不到「新池新建文件的字段表指向都成立（查了 7 处引用）」）；field-refs-translation-red（期望退出 1，实测 0）；
#       field-refs-translation-green（输出里找不到「新池新建文件的字段表指向都成立（查了 3 处引用）」）
#   KB_NUMBERED_FILE_BREAK=first-match  归 lib-item-ref-status.py（设进环境，被各格起的 python 继承）：按编号取文件不分语言，取排序后的第一个命中（改前的取法，取到 <号>-<名>.en.md），
#                               求并集的连译文一起并进来；same_language_files 不筛语言（不变量在用条数把译文里的表行一并数进去）
#       field-refs-translation-green、format-const-placeholders-translation-green（期望退出 0，实测 1）；field-refs-translation-red、format-const-placeholders-translation-red、
#       audit-contradictions-translation-red（期望退出 1，实测 0）；audit-contradictions-translation-green 照旧绿（中文原文在并集里，开关下也找得到）；
#       invariant-counts-translation-green（期望退出 0，实测 1）；invariant-counts-translation-red（输出里找不到「不变量表（1 份文件，登记在 .claude/kb-directories） 正文声称在用 6 条，实际 2 条…」）
#   DOC_REGISTRIES_BREAK=reservation-owner-unchecked  field-refs 格里预留字段归属那一半不判（指向存在不存在的那一半照判）
#       field-refs-reservation-owner-red（期望退出 1，实测 0）；field-refs-reservation-owner-green（输出里找不到「核了 3 行预留字段的指向」）
#   DOC_REGISTRIES_BREAK=field-projection  field-projection 格不跑、按判过了记
#       field-projection-red（期望退出 1，实测 0）；field-registry-green（输出里找不到「被投影的分项，字段表都投影全了…」）
#   DOC_REGISTRIES_BREAK=field-table-sums  field-table-sums 格不跑、按判过了记
#       field-registry-green（输出里找不到「扫了 10 份决策、4 份字节布局表；表后合计 3 张，表前…」）；field-table-sums-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=first-txn-hooks  first-txn-hooks 格不跑、按判过了记
#       field-registry-green（输出里找不到「互相挂钩…」）；first-txn-hooks-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=admission-terms  admission-terms 格不跑、按判过了记
#       admission-terms-red（期望退出 1，实测 0）；field-registry-green（输出里找不到「准入不等式 3 项逐项有归属」）
#   DOC_REGISTRIES_BREAK=formula-fullwidth-d-unmatched  admission-terms 格抓式子那一行只认半角 (d)
#       admission-terms-fullwidth-d-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=segment-registry  segment-registry 格不跑、按判过了记
#       field-registry-green（输出里找不到「比对了 2 处登记…」）；segment-registry-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=segment-registry-not-run-as-red  segment-registry 格把脚本的退 77（只能与产物比的那几处产物不在，本次未跑）也记红
#       segment-registry-product-absent（期望退出 77，实测 1）
#   DOC_REGISTRIES_BREAK=format-const-placeholders  format-const-placeholders 格不跑、按判过了记
#       field-registry-green（输出里找不到「2 个占位」）；format-const-placeholders-red、format-const-placeholders-translation-red（期望退出 1，实测 0）；
#       format-const-placeholders-translation-green（输出里找不到「1 个占位」）
#   DOC_REGISTRIES_BREAK=second-txn-hooks  second-txn-hooks 格不跑、按判过了记
#       field-registry-green（输出里找不到「登记表 1 行、行到步 1 处、「写出的字节」点名 3 处、…」）；second-txn-hooks-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=tree-table-reserve  tree-table-reserve 格不跑、按判过了记
#       field-registry-green（输出里找不到「3 条认购」）；tree-table-reserve-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=owed-number-width-fixed  tree-table-reserve 格抓出处里的编号时位数写死（C 最多三位、D 最多两位）
#       tree-table-reserve-wide-number-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=table-shape  table-shape 格不跑、按判过了记
#       checks-owed-green（输出里找不到「检查了 16 行」）；checks-owed-red（输出里找不到「C4：6 格」）
#   DOC_REGISTRIES_BREAK=closeout-collects-open  closeout-collects-open 格不跑、按判过了记
#       checks-owed-green（输出里找不到「判了 2 份；02-sample.md：全文点名 3 个 C…」）；checks-owed-red（输出里找不到「02-sample.md 点名了这些还开着的欠账号，收口表里…」）
#   DOC_REGISTRIES_BREAK=paid-cited-tests  paid-cited-tests 格不跑、按判过了记
#       checks-owed-green（输出里找不到「查了 1 个标识符」）；checks-owed-red（输出里找不到「C902 引的 `the_sample_test_that_…」）
#   DOC_REGISTRIES_BREAK=audit-contradictions  audit-contradictions 格不跑、按判过了记
#       checks-owed-green、audit-contradictions-translation-green（输出里找不到「每一行都有去向」）；checks-owed-red（输出里找不到「有行没有去向」）；
#       audit-contradictions-translation-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=invariant-single-file  audit-contradictions 格走回改前的读法：不变量类文件与 README.md 不进 kb 正文，I- 号一律指 .claude/kb/invariants.md
#       checks-owed-green（期望退出 0，实测 1）；checks-owed-red（输出里找不到「第 9 行处置列写「已改」，引的「只在 journal 类里…」）
#   DOC_REGISTRIES_BREAK=closeout-row27-preconditions  closeout-row27-preconditions 格不跑、按判过了记
#       checks-owed-green（输出里找不到「5 条逐字探针、覆盖 4 笔」）；checks-owed-red（输出里找不到「收口表第 27 行那几笔欠账的前置动了」）
#   DOC_REGISTRIES_BREAK=experiment-refs  experiment-refs 格不跑、按判过了记
#       references-green（输出里找不到「实验号引用全部有定义：2 个不同的实验号…」）；references-red（输出里找不到「E999 被引用但 experiments/ 下没有它」）；
#       references-unreadable-red（输出里找不到「抽 E 号引用时有文件读不了（grep 退 2）」）
#   DOC_REGISTRIES_BREAK=decision-refs  decision-refs 格不跑、按判过了记
#       references-green（输出里找不到「决策号引用全部有定义：2 个不同的决策号…」）；references-red（输出里找不到「D998 被引用但 decisions/ 下没有它」）；
#       references-unreadable-red（输出里找不到「抽 D 号引用时有文件读不了（grep 退 2）」）；decision-refs-not-decisions-green（输出里找不到「决策号引用全部有定义：1 个不同的决策号…」）；
#       decision-refs-not-decisions-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=not-decisions-ignored  decision-refs 不认 not-decisions 标记：标记管的编号照引用判，标记本身也不判
#       decision-refs-not-decisions-green（期望退出 0，实测 1）；decision-refs-not-decisions-red（输出里找不到「D1 在 decisions/ 下有定义，.claude/kb/notes.md:15 …」）
#   DOC_REGISTRIES_BREAK=not-decisions-whole-file  not-decisions 标记管到整份文件，不只是它下面那一段
#       decision-refs-not-decisions-red（输出里找不到「.claude/kb/notes.md:18 的 not-decisions 标记点名的 D7 在它管的第 19 行里…」）
#   DOC_REGISTRIES_BREAK=declared-counts  declared-counts 格不跑、按判过了记
#       references-green（输出里找不到「欠检查 1 条、已还清 1 条」）；references-red（输出里找不到「不变量表（1 份文件，登记在 .claude/kb-dire…」）；
#       invariant-counts-translation-green（输出里找不到「不变量条数一致：正文声称 2 条在用，表里在用 2 条…」）；invariant-counts-translation-red（输出里找不到「不变量表（1 份文件，登记在 .claude/kb-dire…」）
#   DOC_REGISTRIES_BREAK=governance-refs  governance-refs 格不跑、按判过了记
#       governance-refs-green、references-green（输出里找不到「治理文档与 kb 里的门禁号、门禁全名与格名、路径与「小节」…」）；governance-refs-red（期望退出 1，实测 0）；
#       references-red（输出里找不到「治理文档与 kb 里有指不到的指向（上面逐条列出）」）
#   DOC_REGISTRIES_BREAK=invariant-count-elsewhere  invariant-count-elsewhere 格不跑、按判过了记
#       references-green（输出里找不到「不变量条数跨文件一致：实际在用 1 条，扫了 7 份 kb…」）；references-red（输出里找不到「写「99 条在用」，而 不变量表（1 份文件，登记在 .cl…」）；
#       invariant-counts-translation-green（输出里找不到「不变量条数跨文件一致：实际在用 2 条，扫了 5 份 kb…」）；invariant-counts-translation-red（输出里找不到「.claude/kb/survey.ja.md:3 写「5 条在用」…」）
#   DOC_REGISTRIES_BREAK=citations  citations 格不跑、按判过了记
#       references-green（输出里找不到「样本替身：3 条命中，0 条未命中」）；references-red（输出里找不到「找不到 scripts/verify-ci…」）
#   DOC_REGISTRIES_BREAK=invariant-anchors  invariant-anchors 格不跑、按判过了记
#       references-green（输出里找不到「这次改动新写或改写的 1 行不变量都点名了已定分项」）；references-red（输出里找不到「这次改动新写或改写的 1 行不变量，没有点名它判的字段住在哪…」）
#   DOC_REGISTRIES_BREAK=inline-sums  inline-sums 格不跑、按判过了记
#       inline-sums-green（输出里找不到「kb 正文里的内联算式左右相等：扫 2 份 kb 文件…」）；inline-sums-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=inline-sums-unchecked  inline-sums 格里左右不等不判（算式照解析、照数判了几条）
#       inline-sums-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=inline-sums-operators-judged  inline-sums 格把含乘除减括号、连等的算式也当求和判（只加读得出的项）
#       inline-sums-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=inline-sums-fences-scanned  inline-sums 格把代码围栏里的行也扫
#       inline-sums-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=inline-sums-history-scanned  inline-sums 格把「## 历史版本」之后也扫
#       inline-sums-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=inline-sums-history-files-scanned  inline-sums 格把 *-history.md 与 decisions-history/ 下的文件也扫
#       inline-sums-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=invariant-prerequisites  invariant-prerequisites 格不跑、按判过了记
#       invariant-prerequisites-red（期望退出 1，实测 0）；invariant-prerequisites-green（输出里找不到「未实现的不变量都写了指得到、还没全满足的编号前置…」）
#   DOC_REGISTRIES_BREAK=prerequisites-satisfied-unchecked  invariant-prerequisites 格里前置全都已满足也不判（C446 判别力自证的前一半）
#       invariant-prerequisites-red（输出里找不到「不变量表里 8 处未实现的不变量前置不成立…」，那三行 I-1.4、I-1.6、I-1.8 不报）
#   DOC_REGISTRIES_BREAK=prerequisites-existence-unchecked  invariant-prerequisites 格里前置指不到也不判（C446 判别力自证的后一半）
#       invariant-prerequisites-red（输出里找不到「不变量表里 8 处未实现的不变量前置不成立…」，那三行 I-1.5、I-1.7、I-1.9 不报）
#   DOC_REGISTRIES_BREAK=field-frequency-sizes  field-frequency-sizes 格不跑、按判过了记
#       field-frequency-sizes-red、field-frequency-sizes-registration-red（期望退出 1，实测 0）；field-frequency-sizes-green（输出里找不到「字段频率「每 X 一份」的 X 都指得到…」）
#   DOC_REGISTRIES_BREAK=field-frequency-mark-anywhere  field-frequency-sizes 格认住在哪一节的 format-const 登记（不要求住在已定项小节）
#       field-frequency-sizes-registration-red（输出里找不到「7 处字段频率的大小指不到已定条款…」，单元那两行与字面 32 KiB 那一行不报）
#   DOC_REGISTRIES_BREAK=field-frequency-literal-unchecked  field-frequency-sizes 格不判字面大小
#       field-frequency-sizes-red（输出里找不到「10 处字段频率的大小指不到已定条款…」）；field-frequency-sizes-registration-red（输出里找不到「7 处字段频率的大小…」）
#   DOC_REGISTRIES_BREAK=field-frequency-markers-ignored  field-frequency-sizes 格不认 not-field-frequency 豁免标记（标记本身也不判）
#       field-frequency-sizes-green（期望退出 0，实测 1）；field-frequency-sizes-red（输出里找不到「10 处字段频率的大小指不到已定条款…」）；
#       field-frequency-sizes-registration-red（输出里找不到「7 处字段频率的大小…」）
#   GOVERNANCE_REFS_BREAK=old-numbers-ignored  lib-governance-refs.py 的 ⑤ 整条不判（用旧编号称呼门禁）
#       governance-refs-red（输出里找不到「.claude/kb/tooling.md:3: 用旧编号称…」）
#   GOVERNANCE_REFS_BREAK=names-ignored  lib-governance-refs.py 的 ④ 整条不判（不是门禁目录里的门禁、格名表里没有）
#       governance-refs-green（输出里找不到「扫 5 份现状文字…」）；governance-refs-red（输出里找不到「.claude/agents/sample.md:7: 「门…」）
#   GOVERNANCE_REFS_BREAK=fences-scanned  lib-governance-refs.py 把代码围栏里的行也当现状文字扫
#       governance-refs-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=history-scanned  lib-governance-refs.py 把「## 历史版本」一节与变更史也扫
#       governance-refs-green（期望退出 0，实测 1）；governance-refs-red（输出里找不到「扫 5 份现状文字…」）
#   GOVERNANCE_REFS_BREAK=readme-skipped  lib-governance-refs.py 不把仓根的三份 README（README.md、README.zh.md、README.ja.md）算进现状文字
#       governance-refs-green（输出里找不到「扫 5 份现状文字…」）；governance-refs-red（输出里找不到「README.md:3: 用旧编号称呼门禁」「README.ja.md:3: 用旧编号称呼门禁」「README.zh.md:3: …」）
#   STAGE_CELLS_BREAK=red-swallowed  共用库：红格按绿记、整道退 0
#       admission-terms-red、checks-owed-red、decision-refs-not-decisions-red、field-projection-red、field-refs-red、field-refs-reservation-owner-red、field-table-sums-red、first-txn-hooks-red、
#       format-const-placeholders-red、governance-refs-red、inline-sums-red、references-red、references-unreadable-red、second-txn-hooks-red、segment-registry-red、
#       tree-table-reserve-red、invariant-prerequisites-red、field-frequency-sizes-red、field-frequency-sizes-registration-red、
#       field-refs-translation-red、format-const-placeholders-translation-red、audit-contradictions-translation-red、invariant-counts-translation-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=check-ignored  共用库：点名选格不起作用（--check 与样本根的 .gate-cells 共用一段），照跑全部登记的格
#       admission-terms-red、decision-refs-not-decisions-red、field-projection-red、field-refs-red、field-refs-reservation-owner-red、field-table-sums-red、first-txn-hooks-red、format-const-placeholders-red、
#       inline-sums-red、second-txn-hooks-red、segment-registry-red、tree-table-reserve-red（输出里找不到「doc-registries.sh：1 格里 1 格红…」）；checks-owed-green、decision-refs-not-decisions-green、field-registry-green、
#       field-refs-reservation-owner-green、governance-refs-green、references-green（期望退出 0，实测 1）；checks-owed-red（输出里找不到「doc-registries.sh：5 格里 5 格红…」）；
#       references-red（输出里找不到「doc-registries.sh：7 格里 7 格红…」）；references-unreadable-red（输出里找不到「doc-registries.sh：2 格里 2 格红…」）；
#       invariant-prerequisites-green、field-frequency-sizes-green（期望退出 0，实测 1）；invariant-prerequisites-red（输出里找不到「样本只跑 .gate-cells 点名的格：invariant-prerequisites」）；
#       field-frequency-sizes-red、field-frequency-sizes-registration-red（输出里找不到「样本只跑 .gate-cells 点名的格：field-frequency-sizes」）；
#       field-refs-translation-red、format-const-placeholders-translation-red、audit-contradictions-translation-red（输出里找不到「doc-registries.sh：1 格里 1 格红…」）；
#       field-refs-translation-green、format-const-placeholders-translation-green、audit-contradictions-translation-green（期望退出 0，实测 1）；
#       invariant-counts-translation-red（输出里找不到「doc-registries.sh：2 格里 2 格红…」）；invariant-counts-translation-green（期望退出 0，实测 1）
#   LIB_CHANGED_PATHS_BREAK=untracked  scripts/gate/changed-paths.sh 丢掉未跟踪文件：invariant-anchors 看不见未跟踪的清单
#       references-red（输出里找不到「这次改动新写或改写的 1 行不变量，没有点名它判的字段住在哪…」）
#
#   bash .claude/gate.d/doc-registries.sh [项目根]                                   二十四格都跑
#   bash .claude/gate.d/doc-registries.sh --list                                     逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-registries.sh --check <格名>[,<格名>…] [项目根]          只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

# 阶段自己的位置在 cd 之前取：用相对路径调本阶段、又另给项目根时，cd 之后 $(dirname "$0") 就解析不到了
STAGE_NAME="$(basename "${BASH_SOURCE[0]}")"
STAGE_FILE_NAME="$STAGE_NAME"
STAGE_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_DIRECTORY="$STAGE_DIRECTORY"
REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd)"
OWED_LIBRARY="$STAGE_DIRECTORY/lib-owed.py"
FORMAT_CONST_LIBRARY="$STAGE_DIRECTORY/lib-format-const.py"
# 按编号取哪一份决策 / 实验文件（中文引用方取中文原文，不取排在前面的译文）：field-refs、format-const-placeholders、audit-contradictions 调它的 numbered_files
ITEM_REF_STATUS_LIBRARY="$STAGE_DIRECTORY/lib-item-ref-status.py"
SEGMENT_REGISTRY_SCRIPT="$REPOSITORY/scripts/crash/check-segment-registry.py"
BREAK_SWITCH="${DOC_REGISTRIES_BREAK:-}"
# 握手变量只留给共用库在汇总时写，不导出给各格起的子进程（command-safety.md「进程边界上的三种静默失效」）
export -n GATE_NOT_RUN_FILE 2>/dev/null || true
source "$STAGE_DIRECTORY/lib/stage-cells.sh"

stage_cell field-refs judge_field_refs "新池新建文件的字段表，每个「指向」都指到一条真实存在的分项；预留字段指向的 D9 已定项 12 第几项与所属单元类一致" \
  "每一行的「指向」列要么写一条真实存在的分项（编号改过就跟着改），要么写「无分项」把格式级空白摆出来；算法类型这类预留字段项号不对的，改那一行的指向，或这个字段真属于另一类就改表标题"
stage_cell field-projection judge_field_projection "决策里定的字段有没有漏投影进新池新建文件的表" \
  "把缺的那一行补进 .claude/kb/layout/01-first-txn.md 对应的那一节（字段、宽度、指向哪条分项、状态）"
stage_cell field-table-sums judge_field_table_sums "字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记" \
  "加错了就改那个数，刚加了一行字段就把同一轮要一起改的合计、说明句与 format-const 标记一起改；没登记的结构总宽照红行下面的写法登记"
stage_cell first-txn-hooks judge_new_pool_file_creation_hooks "新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）" \
  "判「是」的未定项要同时进 layout/01-first-txn.md 的「还剩几处空白」表与 milestone/01-first-txn.md 某一步的「会碰到的决策点」；节标题下标步号，里程碑每步点名的节标题逐字抄"
stage_cell admission-terms judge_admission_terms "准入不等式的每一项都有人维护（统计量，或写明的例外）" \
  "给式子里多出来的那一项在对照表里补一行（统计量或写明的例外），副本与权威那一行逐字对齐"
stage_cell segment-registry judge_segment_registry "段序列登记表与钉住活代码的用例逐字比对（钉不到用例的行与 E142 产物比，产物不在报本次未跑）" \
  "照 scripts/crash/check-segment-registry.py 打出的逐行差异改 kb 的「八、」登记表（权威是钉住那条路径的用例）；出处指的用例文件与函数要在，钉不到用例的那几行要按产物判就重跑 E142"
stage_cell format-const-placeholders judge_format_const_placeholders "格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账" \
  "占位那一行写成「// placeholder: D<n>（简称） 已定项 k —— 为什么还是预想」或「// placeholder: C<n>（简称） —— …」，编号要指得到"
stage_cell second-txn-hooks judge_overwrite_release_rollback_reuse_hooks "覆盖写、释放、回退与复用的三处互相挂钩（layout/02 登记表 / 里程碑各步「写出的字节」/ layout/01 段序列登记表）" \
  "layout/02 登记表每一行的「里程碑那一步」与那一步的「写出的字节」互相点名（形态原样引），钉住的用例文件与函数要在仓里"
stage_cell tree-table-reserve judge_tree_table_reserve "树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）" \
  "认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11：合计超了先去掉一个认购者或压小宽度，别直接改 total；出处补一个还开着的欠账号或已定分项"
stage_cell table-shape judge_table_shape "欠账表两张登记表的行形状（格数对得上表头、欠着的那张里不留写着已还的行、表不被空行断开、两张表都在）" \
  "六列的欠账行挪回欠着的那张表、表中间的空行删掉；真还清的按已还清那张的四列改写再挪过去，只还了一部分的把开头改成写明还欠什么"
stage_cell closeout-collects-open judge_closeout_collects_open "里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号" \
  "漏收的开着编号在收口表里给一行，或在表后加一行「- 不收口 C<编号>（简称）：为什么不收」；已经还了就先把欠账表那一行挪进「### 已还清」；行号改成没被占用的顺序号，不加撇号或字母"
stage_cell paid-cited-tests judge_paid_cited_tests "已还清的欠账行里点名的测试名，仓里还在不在" \
  "先 grep 那个测试今天叫什么，用 scripts/kb/replace-once.py 把行里的旧名字定点换成现在的名字；它被删掉了，那笔欠账没还清，整行挪回欠着的那张表"
stage_cell audit-contradictions judge_audit_contradictions "最近一次总审核登记的文档级矛盾，每一行都要有去向" \
  "给这一行一个去向：改了就把处置列写成「已改…」、用「」引一句今天正文里的原文并点名它住在哪份文件；还没改就在 checks-owed.md 开着的那张表里立一笔账、把编号写进处置列"
stage_cell closeout-row27-preconditions judge_closeout_row27_preconditions "里程碑收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）" \
  "先回 .claude/kb/milestone/02-second-txn.md 收口表第 27 行重核这一笔今天可不可达：可达了就当场做成一条会红的用例、把这一笔与探针挪走；只是重构就把探针原文改成今天逐字存在的那一段并写明重核过，别只改期望值"
stage_cell experiment-refs judge_experiment_refs "kb 与项目规则里引用的实验号都有定义" \
  "在 experiments/ 下给它建正文（\`## E<n> <简称>\` 起头），或把引用改成真实存在的实验号；读不了的文件按 grep 的原话修"
stage_cell decision-refs judge_decision_refs "kb 与项目规则里引用的决策号都有定义" \
  "在 decisions/ 下给它建正文（\`## D<n> <简称>\` 起头），或把引用改成真实存在的决策号；命令原样输出、产物行里形状像决策号却不是的串改不得原文，在那一段正上方单起一行写 <!-- doc-registries:not-decisions D<n> -->（写法与判法在这一格上方的注释）；读不了的文件按 grep 的原话修"
stage_cell declared-counts judge_declared_counts "invariants.md 自己那句「现共 N 条在用」与表里条数对得上；欠账表数得出条数" \
  "按红行把 invariants.md 那句条数改成表里现数，登记标记丢了就补回；欠账表数不出条数就照 lib-owed.py 的读法把表恢复"
stage_cell governance-refs judge_governance_refs "治理文档、kb 与仓根的 README.md、README.zh.md、README.ja.md 里按全名指的门禁与格都在、不用旧编号称呼门禁，治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、\`文件「小节」\` 在那份文件里找得到" \
  "按红行把门禁改成按全名称呼（旧编号照红行给的今天的名字）、格名照那道门禁的 --list、路径与「小节」名改成今天存在的（判据与够不着的写法在 .claude/gate.d/lib-governance-refs.py 文件头）"
stage_cell invariant-count-elsewhere judge_invariant_count_elsewhere "invariants.md 之外的 kb 文件里写「N 条在用」的地方，N 等于实际在用条数" \
  "把那几处的 N 改成 invariants.md 表里实际在用的条数，或者改成引用 invariants.md 而不抄数"
stage_cell citations judge_citations "kb 里承重的外部逐行引用还核得动（调 scripts/kb/verify-citations.sh）" \
  "按 verify-citations.sh 自己打印的下一步处置；未命中不等于 kb 写错，也可能是源码固定点没了或版本变了；脚本不在就从 git 找回"
stage_cell invariant-anchors judge_invariant_anchors "这次改动新写或改写的不变量行，点名它判的字段住在哪条已定分项" \
  "在那一行正文里点名一条「D<n>（简称） 已定项 k」，那是这条不变量要判的字段的落点；点不出来先去补分项"
stage_cell inline-sums judge_inline_sums "「整数 = 项 + 项 + …」形态的内联算式（kb 正文里，每项是可选标签加一个整数），左右相等" \
  "按红行右边各项重新加一遍：合计写错了改合计，右边漏了或多了一项就补上或去掉那一项；那一行其实不是求和，就改写成不会被读成「整数 = 项 + 项」的句子（判据与不判的写法在 doc-registries.sh 里这一格函数上方的注释）"
stage_cell invariant-prerequisites judge_invariant_prerequisites "不变量表里状态是「未实现」的每一行都写了编号前置（C / D 分项 / E），指得到、而且还没全满足" \
  "在那一行状态格「未实现」后面写「，前置：<编号>」（C<n>、D<n>（简称） 已定项 k / 未定项 k、E<n>），没有编号就先立一笔欠账；指不到的改成今天存在的编号；前置都已满足的，开工实现它或写明现在挡着它的新编号（判据在 doc-registries.sh 里这一格函数上方的注释）"
stage_cell field-frequency-sizes judge_field_frequency_sizes "决策与布局正文里「每 X 一份」这种字段频率，X 指得到一条已定条款给出的大小（字面大小要等于已定的某个粒度）" \
  "粒度的大小读不出来，让给出大小的已定分项写出这个数、在它的「#### 已定项 k」小节登记 format-const；字面大小对不上就改成已定的粒度；说的不是本格式的字段，在那一行正上方写 <!-- doc-registries:not-field-frequency <为什么> -->（判据与粒度表在 doc-registries.sh 里这一格函数上方的注释）"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *)
      if [[ -n "$root_argument" ]]; then
        echo "  ✗ 给了两个项目根：$root_argument 与 $1"
        echo "     → 怎么办：参数只认一个项目根；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

KB=.claude/kb
work_directory="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 能不能写、满没满；这一道什么都没判。"; exit 1; }
trap 'rm -rf "${work_directory:?}"' EXIT

cell_failed=0
say() { printf '  %s\n' "$*"; }
bad() { printf '  ✗ %s\n' "$*"; cell_failed=1; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → 怎么办： %s\n' "$1"; shift; for l in "$@"; do printf '                %s\n' "$l"; done; }

# ════════════════ 字段与布局组的九格（字段与布局登记） ════════════════

# ── 格 field-refs：新池新建文件的字段表，每个「指向」都指到一条真实存在的分项
# 判据：kb/layout/01-first-txn.md 每行「指向」列里的「D<n>（简称） 已定项 / 未定项 <k>」（简称可省，也可套一层括号），
# 要在那条决策「### 已定项」/「### 未定项」的索引（第一个 #### 之前）里有那一行。
# 「无分项」合法（这张表就是要把格式级空白摆出来）；表或 decisions/ 不在、一处引用都没有，退 77。
# 另判一半（day-1 预留字段的归属）：.claude/kb/layout/*.md 里字段名以「算法类型」起头、指向写「D9（加密） 已定项 12 第 N 项」的行，
# N 要与这一行所在的表所属的单元类一致。单元类按表的节标题、再加表前那一段（空行隔开的最近一块）判：
#   · 点名了 D9 已定项 12 定案表里「自证单元（…）」括号列的某一个（根记录、journal 记录头、系统配置槽；名字现从定案表读，门禁里不抄）
#     ⇒ 自证单元自己身上的算法类型，N 要等于定案表里写着自证单元又写着算法类型的那一行的项号；
#   · 否则点名了指针、extent、节点 ⇒ 数据 / 元数据写的算法类型，N 要等于定案表里写着算法类型而没提自证单元的那一行的项号；
#   · 两样都没点名的行报出来（「!」行）、不判红；同一层里两样都点名，自证单元优先（节标题先判，不够再并上表前那一段）。
# 成功那句多报「核了 K 行预留字段的指向」（K 不含判不出的行）。定案表读不出、而又有行要判时判红，不悄悄当没有。
# 样本：field-refs-reservation-owner-red 里 journal 记录头表的算法类型行写第 4 项（该是第 1 项）判红、点名那一行，指针头表与根记录表各指对不判，
# 第四张表没点名单元类、只报不判；field-refs-reservation-owner-green 三处各指对，报「核了 3 行」。
# 弄坏开关 DOC_REGISTRIES_BREAK=reservation-owner-unchecked：这一半不判。
# 样本：field-refs-red 指向索引里没有的 D322 已定项 7、简称套了括号的 D323 已定项 3、只在论证编号列表里出现的 D322 已定项 5；
# field-registry-green 表格式与列表式两种索引、未定项与「无分项」都放行。
# 「指向」与预留字段的定案表按编号取哪一份：lib-item-ref-status.py 的 numbered_files（中文的布局表取中文原文，译文的取同一种语言的那一份）。
# 样本：field-refs-translation-green（译文排在前面、中文原文有那几条，判绿）、field-refs-translation-red（只有译文列着那一条，判红）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_field_refs() {
  local KB=.claude/kb
  local TABLE="$KB/layout/01-first-txn.md"
  [[ -f "$TABLE" ]] || { echo "  ! 没有 $TABLE，这一格无对象可判"; exit 77; }
  [[ -d "$KB/decisions" ]] || { echo "  ! 没有 $KB/decisions，这一格无对象可判"; exit 77; }

  local bad=0 checked=0 lineno dnum kind item file section

  # 编号 → 决策文件只走 lib-item-ref-status.py 的 numbered_files：这张表是中文，取中文原文 <号>-<名>.md。
  # 改前按 ls 排序取第一个命中，取到的是排在前面的译文 <号>-<名>.en.md（门禁日志里「指向 D9（09-加密.en.md）」那 382 处）。
  local resolved_files resolved_number resolved_file
  local -A decision_file_by_number=()
  if ! resolved_files="$(python3 - "$ITEM_REF_STATUS_LIBRARY" "$KB/decisions" "$TABLE" 2>&1 <<'PY'
import importlib.util, os, re, sys
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
directory, referrer = sys.argv[2], sys.argv[3]
numbers = sorted({int(hit.group(1)) for hit in (re.match(r"(\d+)-", name) for name in os.listdir(directory)) if hit})
for number in numbers:
    found = lib.numbered_files(directory, number, referrer)
    if found:
        print(number, os.path.basename(found[0]), sep="\t")
PY
  )"; then
    printf '%s\n' "$resolved_files" | sed 's/^/      /'   # gate-lint:detail
    echo "  ✗ 按编号取决策文件的那段 python 没跑成，一处指向都没判"
    echo "     → 怎么办： 按上面的 traceback 修（判法在 $ITEM_REF_STATUS_LIBRARY 的 numbered_files）；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  while IFS=$'\t' read -r resolved_number resolved_file; do
    [[ -n "$resolved_number" ]] && decision_file_by_number[$resolved_number]="$resolved_file"
  done <<<"$resolved_files"

  while IFS=$'\t' read -r lineno dnum kind item; do
    [[ -z "${dnum:-}" ]] && continue
    checked=$((checked + 1))

    file="${decision_file_by_number[$((10#$dnum))]:-}"
    if [[ -z "$file" ]]; then
      echo "  ✗ $TABLE:$lineno 指向 D$dnum，而 decisions/ 里没有这条决策"   # gate-lint:detail
      bad=$((bad + 1))
      continue
    fi

    # 只看「### 已定项」/「### 未定项」之后到下一个任意级标题之前的索引：`#### 已定项 N` 之下的论证与「## 历史版本」里另有编号列表和表格，那些不是分项
    section="$(awk -v want="$kind" '
      /^### 已定项[[:space:]]*$/ { cur="已定项"; next }
      /^### 未定项[[:space:]]*$/ { cur="未定项"; next }
      /^#+ / { cur="" }
      cur == want { print }
    ' "$KB/decisions/$file")"

    # 分项行认表格行 `| k | …` 与编号列表 `k. **…**` 两种写法
    if ! grep -qE "^(\| *${item} *\||${item}\. )" <<<"$section"; then
      echo "  ✗ $TABLE:$lineno 指向 D$dnum（$file）的「$kind $item」，而那张表里没有这一条"   # gate-lint:detail
      bad=$((bad + 1))
    fi
  done < <(
    grep -n '指向\|已定项\|未定项' "$TABLE" 2>/dev/null \
      | grep -oE '^[0-9]+:.*' \
      | while IFS= read -r line; do
          n="${line%%:*}"
          body="${line#*:}"
          # 一行里可能有多处引用，逐个抠出来：简称可以不写（「D22 已定项 99」），也可以里面再套一层括号
          # （「D14（双轨（大小文件 / 持久临时）） 已定项 9」）；编号左边不许紧挨字母数字
          grep -oE '(^|[^A-Za-z0-9])D[0-9]+(（([^（）]|（[^（）]*）)*）)? *(已定项|未定项) *[0-9]+' <<<"$body" \
            | grep -oE 'D[0-9]+.*' \
            | while IFS= read -r ref; do
                d="$(grep -oE '^D[0-9]+' <<<"$ref" | tr -d 'D')"
                k="$(grep -oE '(已定项|未定项)' <<<"$ref" | head -1)"
                i="$(grep -oE '[0-9]+$' <<<"$ref")"
                printf '%s\t%s\t%s\t%s\n' "$n" "$d" "$k" "$i"
              done
        done
  )

  # 预留字段的归属：python 的输出末行是 COUNT<tab>判了几行<tab>判不出几行<tab>红几行，其余是红行与「!」行
  local owner_report owner_rc owner_checked=0 owner_unknown=0 owner_bad=0 owner_summary_line
  if [[ "$BREAK_SWITCH" == "reservation-owner-unchecked" ]]; then
    owner_report=$'COUNT\t0\t0\t0'
    owner_rc=0
  else
    owner_report="$(python3 - "$KB" "$ITEM_REF_STATUS_LIBRARY" 2>&1 <<'PY'
import glob, importlib.util, os, re, sys

KB = sys.argv[1]
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[2])
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
OWNER_DECISION, OWNER_ITEM = 9, 12     # D9（加密） 已定项 12：day-1 预留四样
POINTER_WORDS = ("指针", "extent", "节点")   # 第 4 项的射程：数据 / 元数据写（指针头表、extent、节点）的表

# ── 先读定案表：谁是自证单元、各归第几项。读哪一份按引用方取（numbered_files：中文的布局表读中文原文，
#    译文读同一种语言的那一份）；改前取 glob 排序的第一个，读到的是 09-加密.en.md，定案表一行都读不出
owner_tables = {}


def read_owner_table(path):
    names, owner_self, owner_data = [], None, None
    if path is None:
        return names, owner_self, owner_data
    lines = open(path, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    inside = False
    for l in lines:
        if re.match(rf"#{{3,6}}\s*已定项\s*{OWNER_ITEM}\b", l):
            inside = True
            continue
        if inside and re.match(r"#{2,6}\s", l):
            break
        if inside and l.startswith("|"):
            cells = [c.strip() for c in l.strip().strip("|").split("|")]
            if len(cells) < 2 or not re.fullmatch(r"\d+", cells[0]):
                continue
            field = cells[1]
            if "自证单元" in l and "算法类型" in field:
                owner_self = int(cells[0])
                m = re.search(r"自证单元（([^）]+)）", l)
                if m:
                    names = [n.strip() for n in m.group(1).split("、") if n.strip()]
            elif "算法类型" in field:
                owner_data = int(cells[0])
    return names, owner_self, owner_data


def owner_table_for(referrer):
    found = item_ref_status.numbered_files(f"{KB}/decisions", OWNER_DECISION, referrer)
    key = found[0] if found else None
    if key not in owner_tables:
        owner_tables[key] = read_owner_table(key)
    return owner_tables[key]


REFERENCE = re.compile(rf"D{OWNER_DECISION}(?:（(?:[^（）]|（[^（）]*）)*）)? *已定项 *{OWNER_ITEM} *第 *(\d+) *项")


def names_unit(text):
    # 「journal 记录头」拆成 journal 与 记录 两个词，都在才算点名（表标题写「journal」、表前说明写「记录头字段表」是分开的）
    for name in self_proof_names:
        tokens = name.split()
        if len(tokens) > 1:
            tokens[-1] = tokens[-1].rstrip("头")
        if tokens and all(t in text for t in tokens):
            return name
    return None


def classify(heading, paragraph):
    for text in (heading, heading + "\n" + paragraph):
        unit = names_unit(text)
        if unit:
            return "self", unit
        if any(w in text for w in POINTER_WORDS):
            return "data", next(w for w in POINTER_WORDS if w in text)
    return None, None


bad, unknown, checked = [], [], 0
for path in sorted(glob.glob(f"{KB}/layout/*.md")):
    self_proof_names, owner_self, owner_data = owner_table_for(path)
    body = open(path, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    heading, current, last, in_table, context = "", [], [], False, None
    for number, l in enumerate(body, 1):
        if re.match(r"#{1,6}\s", l):
            heading, current, last, in_table = l.lstrip("# ").strip(), [], [], False
            continue
        if not l.startswith("|"):
            in_table = False
            if not l.strip():
                if current:
                    last, current = current, []
            else:
                current.append(l)
            continue
        if not in_table:
            in_table = True
            context = (heading, "\n".join(current or last))
            current = last = []
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if not cells or not re.sub(r"[*`]", "", cells[0]).startswith("算法类型"):
            continue
        for want in REFERENCE.findall("|".join(cells[1:])):
            where = f"{path}:{number}"
            kind, why = classify(*context)
            if kind is None:
                unknown.append(f"{where} 写「D{OWNER_DECISION} 已定项 {OWNER_ITEM} 第 {want} 项」，而它所在的表（节标题「{context[0]}」）"
                               "没点名自证单元、也不是指针 / extent / 节点的表，判不出该是哪一项，没判")
                continue
            checked += 1
            if kind == "self":
                should, tail = owner_self, f"表里点名了自证单元「{why}」，自证单元自己身上的算法类型归第 {owner_self} 项"
            else:
                should, tail = owner_data, f"表是「{why}」这类数据 / 元数据写的表，每个写的算法类型字段归第 {owner_data} 项"
            if should is None:
                bad.append(f"{where} 读不出 D{OWNER_DECISION} 已定项 {OWNER_ITEM} 定案表里{'自证单元' if kind == 'self' else '数据 / 元数据写'}那一行，"
                           "没法判这一行该指第几项")
            elif int(want) != should:
                bad.append(f"{where} 算法类型写「D{OWNER_DECISION} 已定项 {OWNER_ITEM} 第 {want} 项」，该是第 {should} 项（节标题「{context[0]}」）：{tail}")

for b in bad:
    print(f"  ✗ {b}")          # gate-lint:detail
for u in unknown:
    print(f"  ! {u}")
print(f"COUNT\t{checked}\t{len(unknown)}\t{len(bad)}")
sys.exit(1 if bad else 0)
PY
)"
    owner_rc=$?
  fi
  owner_summary_line="$(tail -n 1 <<<"$owner_report")"
  if [[ ( $owner_rc -ne 0 && $owner_rc -ne 1 ) || "$owner_summary_line" != COUNT$'\t'* ]]; then
    printf '%s\n' "$owner_report" | sed 's/^/  /'
    echo "  ✗ 判预留字段归属的那段 python 没跑成（退出码 $owner_rc）——这一半没判"
    echo "     → 怎么办： 按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    exit 1
  fi
  IFS=$'\t' read -r _ owner_checked owner_unknown owner_bad <<<"$owner_summary_line"
  head -n -1 <<<"$owner_report"   # gate-lint:detail
  bad=$((bad + owner_bad))

  if ((bad)); then
    echo "  ✗ 新池新建文件的字段表里有 $bad 处指向有误（指向了不存在的分项，或预留字段的项号与所属单元类对不上）"
    echo "     → 怎么办： 每一行的「指向」列要么写一条真实存在的分项（编号改过就跟着改），"
    echo "                要么写「无分项」——后者是这张表要暴露的格式级空白，不判红。"
    echo "                分项现在叫什么，看 .claude/kb/decisions/ 里那条决策的两张索引表。"
    echo "                预留字段（算法类型）项号对不上的，改那一行的指向，或者这个字段真属于另一类就改表标题；"
    echo "                哪个单元归哪一项，看 .claude/kb/decisions/ 里 D9（加密） 已定项 12 的定案表。"
    exit 1
  fi

  if ((checked == 0)); then
    echo "  ! $TABLE 里一处「D<n>（简称） 已定项 / 未定项 k」引用都没有，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 新池新建文件的字段表指向都成立（查了 $checked 处引用）"
  echo "  ✓ 核了 $owner_checked 行预留字段的指向（另有 $owner_unknown 行判不出所属单元类，没判）"
}

# ── 格 field-projection：决策里定的字段有没有漏投影进新池新建文件的表
# field-refs 判正向（投影表每个字段指得到分项），这一格判反向：分项的字段表新加了一行，投影表没跟着加。
# 射程：
#   1. 只认表头恰好是 `| 字段 | 宽 |` 或 `| 字段 | 宽度 |` 的表，散在段落里的字段定义抓不到。
#   2. 只查 layout/01-first-txn.md 里引过（`D<n>（简称） 已定项 <k>`）的分项：新池新建文件不含快照、加密、目录树，
#      对没投影的分项要求投影会假红；代价是从没进过投影表的分项，这一格一个字也不说。
#   3. 字段名按子串比，只判在不在，不判宽度（宽度归 code-source-discipline 与 C94（登记的格式常量与后来的定案对不上））。
#   一张被投影的字段表都没核到，退 77。
# 样本：field-projection-red 里 D362 已定项 7 的字段表三行，投影表只收了 magic，「块头版本」只在别的节的正文里提了一句；
# field-registry-green 里被投影的两行都在引了那条分项的那一节的表里。
cell_field_projection() {
  local FTL=.claude/kb/layout/01-first-txn.md
  [[ -f "$FTL" && -d .claude/kb/decisions ]] || { echo "  ! 找不到 $FTL 或 decisions/，这一格无对象可判"; exit 77; }

  python3 - "$FTL" <<'PY'
import re, sys, glob, os

ftl_path = sys.argv[1]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]

REFERENCE = r"D(\d+)（[^）]*）\s*已定项\s*(\d+)"
refs = {(int(a), int(b)) for a, b in re.findall(REFERENCE, ftl)}

# 字段名要落在引了那条分项的节（按 `## ` 切）的表格行里：整份文件按子串找，会让别的节的正文顶替（「树 ID 水位」在别处提过一句，那一节的表里却没有）
sections = []
for chunk in re.split(r"(?m)^(?=## )", ftl):
    chunk_refs = {(int(a), int(b)) for a, b in re.findall(REFERENCE, chunk)}
    table_rows = "\n".join(line for line in chunk.splitlines() if line.startswith("|"))
    sections.append((chunk_refs, table_rows))

def projected_table_rows(decision_item):
    return "\n".join(rows for chunk_refs, rows in sections if decision_item in chunk_refs)

checked_tables, checked_rows, bad = 0, 0, []
for p in sorted(glob.glob(".claude/kb/decisions/*.md")):
    dnum = int(os.path.basename(p).split("-")[0])
    lines = open(p, encoding="utf-8").read().split("\n## 历史版本")[0].splitlines()
    item = None      # 最近一个「已定项 N」标题的编号
    in_tbl = False
    for i, l in enumerate(lines, 1):
        m = re.match(r"#{3,6}\s*已定项\s*(\d+)", l)
        if m:
            item = int(m.group(1))
        if not l.startswith("|"):
            in_tbl = False
            continue
        c = [x.strip() for x in l.strip().strip("|").split("|")]
        if len(c) >= 2 and c[0] == "字段" and c[1] in ("宽", "宽度"):
            in_tbl = item is not None and (dnum, item) in refs
            if in_tbl:
                checked_tables += 1
            continue
        if in_tbl and len(c) >= 2 and re.fullmatch(r"\**\d+\**", c[1]):
            name = re.sub(r"[*`]", "", c[0]).strip()
            if not name:
                continue
            checked_rows += 1
            if name not in projected_table_rows((dnum, item)):
                bad.append(f"{p}:{i}  D{dnum} 已定项 {item} 的字段表有「{name}」（宽 {c[1]}），"
                           f"而 {ftl_path} 里引用了这条分项的那几节的表格行里找不到它")

if bad:
    print(f"  ✗ 分项定了的字段没投影进新池新建文件的表 {len(bad)} 处：")     # gate-lint:summary
    for b in bad:
        print("      " + b)                                              # gate-lint:detail
    print(f"  → 怎么办：把缺的那一行补进 {ftl_path} 对应的那一节（字段、宽度、指向哪条分项、状态），")
    print("    它是「新池新建文件写出哪些字节」的投影表，漏一行等于那几个字节没有任何人在看；")
    print("    若这个字段确实不属于新池新建文件（快照 / 加密 / 目录树），")
    print("    那就把它从这条分项的字段表里挪走，或把该分项从投影表的引用里去掉——两处口径必须一致。")
    sys.exit(1)

if checked_tables == 0:
    print(f"  ! 本次无对象可判：投影表引用了 {len(refs)} 条分项，其中没有一条的正文里有「| 字段 | 宽 |」字段表")
    sys.exit(77)
print(f"  ✓ 被投影的分项，字段表都投影全了（{checked_tables} 张字段表、{checked_rows} 行；"
      f"投影表引用了 {len(refs)} 条分项）")
PY
}

# ── 格 field-table-sums：字段表加出来的数（表后合计、表前说明句、format-const 标记）与字节布局表里结构总宽的登记
# 落的是 C94（登记的格式常量与后来的定案对不上）里能机械判的那几样。射程：
#   1. 只认表头里「字段」列紧跟「宽」或「宽度」列的表（前面可以有「段」这类列）；表后合计在表后 3 行内，写成「合计 N 字节」或「合计 **N** 字节」。
#   2. 宽度格按整数或只含 ×、+、− 的整数算式读；有一行读不出（「未定」「随实现定」「—」），整张表跳过并计入跳过数：加不出确定的和时判红只会是假红。
#   3. 只判总宽与各行之和、标记与分项之和、布局里的总宽有没有登记；各行宽度本身对不对不判。
#   4. 一条「已定项 N」里恰好一个 format-const 标记时，标记值要等于该分项下全部字段表之和；有多个标记时无从对应，跳过并报出。
#   5. 扫 .claude/kb/decisions/*.md 与 .claude/kb/layout/*.md 的正文（「## 历史版本」之前）。
#   6. 标记按 lib-format-const.py 读；以 `<!-- format-const` 开头而读不出的、同一份文件里同一个名字登记了不止一次的，判红。
#   7. 表前说明句只认表头前最多 3 行非空行里以 `**` 起头的标题行。增量式「+M 字节 ⇒ 共 N 字节」判 M = 这张表之和、
#      N = 这一小节（上一个标题起）到这张表为止全部字段表之和；绝对式「合计 / 共 N 字节」「N 字节初值」「（N 字节，」「定长 N」判 N = 这张表之和。
#   8. 字节布局表（.claude/kb/layout/*.md）的结构总宽只认三种写法：① 非表格行的「合计 N 字节」；② 以 ⇒ 起头的行里紧挨求和等式的加粗整数
#      （等式那一侧去掉括注后要有 +，只有 × 的「6 × 55 = **330**」是条数乘宽、不算）；③ 带「状态」列的表里状态以「已定」开头的行、宽度格里加粗的整数。
#      登记 = 同一行（① 还可以是它那张表的说明句）里有值等于 N 的 `<!-- format-const: 名字 = N -->`，或有「`format-const: 名字`」引用、
#      而那个名字在 kb 正文里登记的值等于 N；缺即判红。认不出的写法这一格一个字也不说，成功那句报认出了几处。
#   四样一处都没认出，退 77；表前表后都没写认得出的总宽的表，成功行下面逐个列表头行。
# 样本：field-table-sums-red 放合计写错的决策表与布局表、与表和对不上的标记、多写了键的标记、同一份文件登记两次的名字、
# 登记 78 而正文两笔增量共 13 字节（367-样例）、三句写错的表前说明句（369-样例）、布局里没登记的三种结构总宽、引用值不对与没登记的名字、
# `| 段 | 字段 | 宽 |` 带乘式的表合计写错（02-样例），判红；field-registry-green 放同一个增量样本而登记值改成 91（368-样例）、
# 三种写法各一处登记好的总宽、未定行里的加粗宽度与只有 × 的等式（不算），判绿。
cell_field_table_sums() {
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 decisions/，这一格无对象可判"; exit 77; }

  python3 - "$FORMAT_CONST_LIBRARY" <<'PY'
import re, glob, importlib.util, os, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)

SUM = re.compile(r"合计\s*\**\s*(\d+)\s*\**\s*字节")
HEAD = re.compile(r"^#{2,6}\s")
ITEM = re.compile(r"^#{3,6}\s*已定项\s*(\d+)")
# 表前说明句（射程第 7 条）：增量式先认，认上了就不再按绝对式判同一句
CAPTION_INCREMENT = re.compile(r"\+\s*(\d+)\s*字节\s*⇒\s*共\s*\**\s*(\d+)\s*\**\s*字节")
CAPTION_ABSOLUTE_FORMS = (
    re.compile(r"(?:合计|共)\s*\**\s*(\d+)\s*\**\s*字节"),
    re.compile(r"(\d+)\s*字节初值"),
    re.compile(r"（\s*(\d+)\s*字节[，；）]"),
    re.compile(r"定长\s*(\d+)"),
)
WIDTH_EXPRESSION = re.compile(r"\d+(?:\s*[×+−-]\s*\d+)*")
# 射程第 8 条：字节布局表里认得出的结构总宽
BOLD_WIDTH = re.compile(r"\*\*([^*\d]{0,4})(\d+)\*\*")
BOLD_PURE_WIDTH = re.compile(r"\*\*(\d+)\*\*")
CLAUSE_BOUNDARY = re.compile(r"[；;，,。：:⇒]")
PARENTHETICAL = re.compile(r"（[^（）]*）|\([^()]*\)")
REFERENCE = re.compile(r"format-const:\s*`?([A-Z][A-Z0-9_]*)")


def table_cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def is_separator_row(cells):
    return set("".join(cells)) <= set("-: ")


def width_value(cell):
    """宽度格按整数或只含 ×、+、− 的整数算式读；读不出来返回 None。"""
    text = re.sub(r"[*`]", "", cell).strip()
    if not WIDTH_EXPRESSION.fullmatch(text):
        return None
    expression_value = 0
    for sign, term in re.findall(r"([+−-]?)\s*(\d+(?:\s*×\s*\d+)*)", text):
        product = 1
        for factor in re.split(r"\s*×\s*", term):
            product *= int(factor)
        expression_value += -product if sign in ("−", "-") else product
    return expression_value


def field_table_width_column(cells):
    """表头里「字段」紧跟着「宽 / 宽度」时，返回宽度列的下标。"""
    for index in range(len(cells) - 1):
        if cells[index] == "字段" and cells[index + 1] in ("宽", "宽度"):
            return index + 1
    return None


def caption_lines(lines, header_index):
    """表头之前的说明句：往上最多 3 行非空行，碰到表格行或标题就停。"""
    collected = []
    index = header_index - 1
    while index >= 0 and len(collected) < 3:
        line = lines[index]
        if line.startswith("|") or HEAD.match(line):
            break
        if line.strip():
            collected.append(index)
        index -= 1
    return sorted(collected)


def without_parentheticals(text):
    """去掉括注（可以嵌套）：「头部 50 + 位置条目 14 × 2（`format-const: LOC_ENTRY`）+ 写序 10」只剩算式。"""
    previous = None
    while previous != text:
        previous, text = text, PARENTHETICAL.sub("", text)
    return text


def strip_bold_and_code(text):
    return re.sub(r"[*`]", "", text)


def note_bold_width_in_settled_row(header, cells, line_index, layout_widths):
    """带「状态」列的表里，状态以「已定」开头的行，宽度格里加粗的整数算一处结构总宽。"""
    if is_separator_row(cells) or "状态" not in header:
        return
    width_names = [name for name in ("宽", "宽度") if name in header]
    if not width_names:
        return
    status_index, width_index = header.index("状态"), header.index(width_names[0])
    status = strip_bold_and_code(cells[status_index]) if status_index < len(cells) else ""
    if not status.startswith("已定") or width_index >= len(cells):
        return
    for bold in BOLD_PURE_WIDTH.finditer(cells[width_index]):
        layout_widths.append((line_index + 1, int(bold.group(1)), [line_index], "已定行的加粗宽度"))


# 登记值：kb 正文里全部读得出来的标记（与 code-source-discipline 同一个取法：不含变更史）
kb_body_paths = [path for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True))
                 if not path.endswith("-history.md") and "/decisions-history/" not in path]
registered_value_by_name = {}
for path in kb_body_paths:
    body_text = open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
    for mark in format_const.parse_marks(body_text).marks:
        registered_value_by_name.setdefault(mark.name, mark.value)


def registrations_in(scope_text):
    """一段文字里的登记：标记（带值）与 `format-const: 名字` 引用（值取它在 kb 里登记的那个）。"""
    found = [(mark.name, mark.value) for mark in format_const.parse_marks(scope_text).marks]
    for name in REFERENCE.findall(format_const.strip_marks(scope_text)):
        found.append((name, registered_value_by_name.get(name)))
    return found


checked, skipped, tables_without_total, bad = 0, [], [], []
caption_checked = 0
mark_checked, mark_skipped = 0, []
marker_problems = []
layout_widths_checked, layout_unregistered = 0, []
decision_paths = sorted(glob.glob(".claude/kb/decisions/*.md"))
layout_paths = sorted(glob.glob(".claude/kb/layout/*.md"))

for kb_path in decision_paths + layout_paths:
    body = open(kb_path, encoding="utf-8").read().split("\n## 历史版本")[0]
    lines = body.split("\n")
    is_layout = kb_path in layout_paths
    parsed_marks = format_const.parse_marks(body)
    marks_by_line = {}
    for mark in parsed_marks.marks:
        marks_by_line.setdefault(mark.line_number, []).append((mark.name, mark.value))
    for unparsable in parsed_marks.unparsable:
        marker_problems.append(f"{kb_path}:{unparsable.line_number}  标记按文法读不出来：「{unparsable.excerpt}」")
    for duplicate in parsed_marks.duplicates:
        marker_problems.append(f"{kb_path}  {duplicate.name} 在这一份里登记了 {len(duplicate.line_numbers)} 次"
                               f"（第 {'、'.join(str(line_number) for line_number in duplicate.line_numbers)} 行）")
    # 分项区间：从「已定项 N」标题到下一个任意标题
    item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0
    # 小节区间：从任意标题到下一个任意标题，给「+M 字节 ⇒ 共 N 字节」的累计用
    section_sum, section_ok = 0, True
    # 字节布局表里最近一张字段表：「合计 N 字节」那句的登记可以写在它的说明句里
    last_table_end, last_table_caption = -10, []
    layout_widths = []   # (行号, 值, 登记范围的行下标, 怎么认出来的)

    def close_item():
        global item, item_sum, item_marks, item_ok, item_head, mark_checked
        # 分项里有标记而没比成的，一律进跳过清单点名：宽度不是数、或这一节没有字段表，都不许静默放过
        if item is not None and item_marks:
            if len(item_marks) != 1:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 里有 "
                                    f"{len(item_marks)} 个 format-const 标记，对不上哪张表")
            elif not item_ok:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 的 format-const 标记 {item_marks[0][0]} 没核："
                                    f"这一节有宽度不是数的字段表，加不出确定的和")
            elif not item_sum:
                mark_skipped.append(f"{os.path.basename(kb_path)} 已定项 {item} 的 format-const 标记 {item_marks[0][0]} 没核："
                                    f"这一节没有加得出和的字段表")
            else:
                name, value_registered = item_marks[0]
                mark_checked += 1
                if value_registered != item_sum:
                    bad.append(f"{kb_path}:{item_head}  已定项 {item} 登记 {name} = {value_registered}，"
                               f"而这一节的字段表加起来是 {item_sum}")
        item, item_sum, item_marks, item_ok, item_head = None, 0, [], True, 0

    header_of_current_table = None
    line_index = 0
    while line_index < len(lines):
        line = lines[line_index]
        item_match = ITEM.match(line)
        if item_match:
            close_item()
            item, item_head = int(item_match.group(1)), line_index + 1
        elif HEAD.match(line) and item is not None:
            close_item()
        if HEAD.match(line):
            section_sum, section_ok = 0, True
        if item is not None:
            item_marks.extend(marks_by_line.get(line_index + 1, []))
        if not line.startswith("|"):
            header_of_current_table = None
            if is_layout:
                for sum_match in SUM.finditer(line):
                    scope = [line_index] + (last_table_caption if line_index - last_table_end <= 3 else [])
                    layout_widths.append((line_index + 1, int(sum_match.group(1)), scope, "合计"))
                if line.lstrip().startswith("⇒"):
                    for bold in BOLD_WIDTH.finditer(line):
                        after = CLAUSE_BOUNDARY.split(without_parentheticals(line[bold.end():]), 1)[0]
                        before = CLAUSE_BOUNDARY.split(without_parentheticals(line[:bold.start()]))[-1]
                        defines_by_sum = (after.lstrip().startswith("=") and "+" in after) or \
                                         (before.rstrip().endswith("=") and "+" in before)
                        if defines_by_sum:
                            layout_widths.append((line_index + 1, int(bold.group(2)), [line_index], "⇒ 等式"))
            line_index += 1
            continue
        cells = table_cells(line)
        if header_of_current_table is None:
            header_of_current_table = cells
        elif is_layout:
            note_bold_width_in_settled_row(header_of_current_table, cells, line_index, layout_widths)
        width_index = field_table_width_column(cells)
        if width_index is None:
            line_index += 1
            continue
        header_line_number = line_index + 1
        caption = caption_lines(lines, line_index)
        line_index += 2
        table_sum, unreadable_width, summed_rows = 0, None, 0
        while line_index < len(lines) and lines[line_index].startswith("|"):
            row_cells = table_cells(lines[line_index])
            if is_layout:
                note_bold_width_in_settled_row(cells, row_cells, line_index, layout_widths)
            cell = row_cells[width_index] if width_index < len(row_cells) else ""
            value = width_value(cell)
            if value is not None:
                table_sum += value; summed_rows += 1
            elif cell and not is_separator_row([cell]):
                unreadable_width = unreadable_width or (f"{os.path.basename(kb_path)}:{line_index+1} 「{strip_bold_and_code(row_cells[width_index - 1])[:40]}」"
                                  f"宽度是「{strip_bold_and_code(cell)[:40]}」")
            line_index += 1
        header_of_current_table = None
        last_table_end, last_table_caption = line_index - 1, caption
        if item is not None:
            if unreadable_width:
                item_ok = False
            else:
                item_sum += table_sum
        if unreadable_width:
            section_ok = False
        else:
            section_sum += table_sum
        caption_text = format_const.strip_marks(" ".join(lines[index] for index in caption
                                                         if lines[index].lstrip().startswith("**")))
        caption_judged = False
        increment_match = CAPTION_INCREMENT.search(caption_text)
        absolute_match = None
        if not increment_match:
            for form in CAPTION_ABSOLUTE_FORMS:
                absolute_match = form.search(caption_text)
                if absolute_match:
                    break
        if (increment_match or absolute_match) and unreadable_width:
            skipped.append(unreadable_width)
            caption_judged = True
        elif increment_match:
            caption_judged = True
            caption_checked += 1
            increment, cumulative = int(increment_match.group(1)), int(increment_match.group(2))
            if increment != table_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「+{increment} 字节」，而这张字段表 {summed_rows} 行加起来是 {table_sum}")
            if not section_ok:
                skipped.append(f"{os.path.basename(kb_path)}:{header_line_number} 表前写「共 {cumulative} 字节」，"
                               f"而同一节前面有宽度不是数的字段表，累计加不出来")
            elif cumulative != section_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「共 {cumulative} 字节」，而这一节到这张表为止的字段表加起来是 {section_sum}")
        elif absolute_match:
            caption_judged = True
            caption_checked += 1
            if int(absolute_match.group(1)) != table_sum:
                bad.append(f"{kb_path}:{header_line_number}  表前写「{absolute_match.group(0)}」，而这张字段表 {summed_rows} 行加起来是 {table_sum}")
        sum_match_after, sum_line_number = None, 0
        for following_index in range(line_index, min(line_index + 3, len(lines))):
            sum_match_after = SUM.search(lines[following_index])
            if sum_match_after:
                sum_line_number = following_index + 1
                break
        if not sum_match_after:
            if not caption_judged:
                tables_without_total.append(f"{os.path.basename(kb_path)}:{header_line_number}")
            continue
        if unreadable_width:
            if unreadable_width not in skipped:
                skipped.append(unreadable_width)
            continue
        checked += 1
        if int(sum_match_after.group(1)) != table_sum:
            bad.append(f"{kb_path}:{sum_line_number}  写「合计 {sum_match_after.group(1)} 字节」，而它上面那张字段表 {summed_rows} 行加起来是 {table_sum}"
                       f"（表头在第 {header_line_number} 行）")
    close_item()

    for line_number, value, scope, form in layout_widths:
        layout_widths_checked += 1
        found = registrations_in("\n".join(lines[index] for index in scope))
        if any(registered == value for _name, registered in found):
            continue
        shown = "、".join(f"{name} = {registered if registered is not None else '（kb 里没登记）'}"
                         for name, registered in found) or "一个都没有"
        layout_unregistered.append(f"{kb_path}:{line_number}  {form}写出宽度 {value}，这一行的登记：{shown}")

if bad:
    print(f"  ✗ 字段表加起来的数与写下来的对不上 {len(bad)} 处：")        # gate-lint:summary
    for entry in bad:
        print("      " + entry)                                              # gate-lint:detail
    print("  → 怎么办：加错了就改那个数；若是刚加了一行字段，那么**同一轮要一起改的还有**——")
    print("    该决策索引表那一行里的合计、这一节的 format-const 标记、表前说明句里的「+M 字节 ⇒ 共 N 字节」、")
    print("    [layout/01-first-txn.md] 对应那一节，以及全仓引过这个数的地方")
    print("    （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`：撤回一个数要当场回扫谁在引它）。")
if marker_problems:
    print(f"  ✗ format-const 标记读不出来或在同一份文件里重复登记 {len(marker_problems)} 处：")  # gate-lint:summary
    for problem in marker_problems:
        print("      " + problem)                                        # gate-lint:detail
    print("  → 怎么办：读不出来的照 <!-- format-const: 名字 = 整数 stale=旧串|旧串 --> 改写，stale= 之外不许有别的键；")
    print("    它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。")
    print("    重复的只留定这个值的那一处，别处要提它写成不带 <!-- 的文字（例如「`format-const: 名字`」）。")
    print("    标记的文法只有一份，在 .claude/gate.d/lib-format-const.py，code-source-discipline、checker-independence-and-sync 按同一份读。")
if layout_unregistered:
    print(f"  ✗ 字节布局表里写出的结构总宽没有登记 {len(layout_unregistered)} 处：")  # gate-lint:summary
    for entry in layout_unregistered:
        print("      " + entry)                                          # gate-lint:detail
    print("  → 怎么办：在同一行（「合计 N 字节」那句也可以写在它那张表的说明句里）登记这个宽度：")
    print("    这个量还没有登记位，就在定它的那一处加 <!-- format-const: 名字 = N -->；")
    print("    已经在某份决策里登记过，就在这一行写「`format-const: 名字`」引用它，门禁按登记值比。")
    print("    名字与 crates/singlefs-format 里的 const 同名时，code-source-discipline 会拿登记值去核那个 const，要写成整数字面量。")
    print("    这个数其实不是一个结构的总宽，就别用加粗或「合计 N 字节」写它（C94（登记的格式常量与后来的定案对不上））。")
if bad or marker_problems or layout_unregistered:
    sys.exit(1)

# 四样一样都没核到：什么都没比过，不记通过（射程第 8 条：认出 0 处不是「都登记了」）
if checked + caption_checked + mark_checked + layout_widths_checked == 0:
    print(f"  ! 本次无对象可判：扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表，"
          f"表后合计、表前说明句、format-const 标记、结构总宽一处都没认出来")
    sys.exit(77)

message = (f"  ✓ 字段表加出来的数都对得上（扫了 {len(decision_paths)} 份决策、{len(layout_paths)} 份字节布局表；"
           f"表后合计 {checked} 张，表前说明句 {caption_checked} 张，format-const 标记 {mark_checked} 个；"
           f"字节布局表里的结构总宽 {layout_widths_checked} 处都有登记）")
if skipped:
    message += f"，跳过 {len(skipped)} 处（有非数字宽度）"
if tables_without_total:
    message += f"，另有 {len(tables_without_total)} 张表前表后都没写认得出的总宽、这一格没验它们（表头行逐个列在下面）"
print(message)
for entry in skipped + mark_skipped:
    print(f"     ! 跳过：{entry}")
for entry in tables_without_total:
    print(f"     没验：{entry} 那张字段表，表前表后都没写认得出的总宽")
PY
}

# ── 格 first-txn-hooks：新池新建文件的三份文件互相挂钩（字节表 / 里程碑 / 决策索引）
# 三份说的是同一件事：decisions/*.md 的每条未定项判过「改新池新建文件的字节：是 / 否」；layout/01-first-txn.md 列出每个字节，
# 判「是」的未定项在它的「还剩几处空白」表里各占一行；milestone/01-first-txn.md 把写这些字节的活拆成步，每步写「写出的字节」在哪几节、「会碰到的决策点」。
# 判据（每条都报绝对数，扫到 0 也报）：
#   1. 判「是」且仍未定的每一条 `D<n> 未定项 <k>`，字节表与里程碑各至少引一次。
#   2. 字节表「还剩几处空白」表里引到的未定项，必须都在第 1 条那个集合里（定了就要从表里拿走）。
#   3. 字节表每一节（历史版本与空白表除外）标题下紧跟一行 `**里程碑**：… 步 N`，N 是里程碑里存在的 `## 步 N`。
#   4. 里程碑每个 `## 步 N` 有一行 `**写出的字节**：`，要么写「无」，要么点名字节表里存在的节标题（「…」引起来的那段）：
#      要与某一节的标题逐字相同，只许省掉标题末尾那一段全角括注；只是标题的一截不算点名。被字节表某节标成归属的步，不许写「无」。
# 射程：只认「改新池新建文件的字节：**是**（日期…」这一种写法（括号是判据的一部分），写在未定项小节之外的散文抓不到；
#   引用形态只认 `D<n>（简称） 未定项 <k>`，不判引用处说的内容对不对。判「是」的未定项、字节表的节、里程碑的步三样一个都没有，退 77。
# 样本：first-txn-hooks-red 里判「是」的 D401 未定项 1 里程碑没引、空白表引着判「否」的未定项 2、一节没标步号、
# 被标成归属的步 1 写「无」、步 2 只点了标题的一截；field-registry-green 里三面都对得上。
cell_new_pool_file_creation_hooks() {
  local FTL=.claude/kb/layout/01-first-txn.md
  local MS=.claude/kb/milestone/01-first-txn.md
  local DEC=.claude/kb/decisions
  [[ -f "$FTL" && -f "$MS" && -d "$DEC" ]] || { echo "  ! 找不到 $FTL / $MS / $DEC，这一格无对象可判"; exit 77; }

  python3 - "$FTL" "$MS" "$DEC" <<'PY'
import re, sys, glob, os

ftl_path, ms_path, dec_dir = sys.argv[1:4]
ftl = open(ftl_path, encoding="utf-8").read().split("\n## 历史版本")[0]
ms = open(ms_path, encoding="utf-8").read().split("\n## 历史版本")[0]
REF = re.compile(r"D(\d+)（[^）]*）\s*未定项\s*(\d+)")
# 只认带日期括号的规范形态；「……判过「改新池新建文件的字节：是」」这种转述引用不算
YES = re.compile(r"改新池新建文件的字节：\**是\**（")

# 1. 判「是」且仍未定的分项集合
blocking = set()
for p in sorted(glob.glob(os.path.join(dec_dir, "*.md"))):
    m = re.match(r"(\d+)-", os.path.basename(p))
    if not m:
        continue
    dnum = int(m.group(1))
    body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
    sec = re.search(r"^### 未定项.*?(?=^### |\Z)", body, re.M | re.S)
    if not sec:
        continue
    entries = re.split(r"^(?=\| *\d+ *\||\s*\d+\. )", sec.group(0), flags=re.M)
    for e in entries:
        m2 = re.match(r"\| *(\d+) *\||\s*(\d+)\. ", e)
        if not m2:
            continue
        k = int(m2.group(1) or m2.group(2))
        if YES.search(e):
            blocking.add((dnum, k))

in_ftl = {(int(a), int(b)) for a, b in REF.findall(ftl)}
in_ms = {(int(a), int(b)) for a, b in REF.findall(ms)}
bad = []
for d, k in sorted(blocking):
    if (d, k) not in in_ftl:
        bad.append(f"D{d} 未定项 {k} 判了「改新池新建文件的字节：是」，而 {ftl_path} 一处都没引它")
    if (d, k) not in in_ms:
        bad.append(f"D{d} 未定项 {k} 判了「改新池新建文件的字节：是」，而 {ms_path} 一处都没引它")

# 2. 空白表只许装第 1 条那个集合里的分项
gap = re.search(r"^## 还剩几处空白.*?(?=^## |\Z)", ftl, re.M | re.S)
gap_refs = set()
if gap:
    for l in gap.group(0).splitlines():
        if l.startswith("|"):
            gap_refs |= {(int(a), int(b)) for a, b in REF.findall(l)}
for d, k in sorted(gap_refs - blocking):
    bad.append(f"{ftl_path} 的「还剩几处空白」表引着 D{d} 未定项 {k}，而它今天不在「判是且未定」的集合里（定了，或判的不是「是」）")

# 3. 字节表每节标里程碑步号
steps = {int(x) for x in re.findall(r"^## 步 (\d+)", ms, re.M)}
ftl_lines = ftl.splitlines()
sections, tagged_steps = 0, set()
for i, l in enumerate(ftl_lines):
    if not re.match(r"^#{2,3} ", l) or l.startswith("## 还剩几处空白"):
        continue
    sections += 1
    nxt = next((x for x in ftl_lines[i + 1:] if x.strip()), "")
    m = re.match(r"\*\*里程碑\*\*：.*步 (\d+)", nxt)
    if not m:
        bad.append(f"{ftl_path}:{i + 1} 「{l.strip()}」标题下面没有一行「**里程碑**：… 步 N」")
        continue
    ns = {int(x) for x in re.findall(r"步 (\d+)", nxt)}
    for n in ns:
        if n not in steps:
            bad.append(f"{ftl_path}:{i + 2} 标着里程碑步 {n}，而 {ms_path} 里没有「## 步 {n}」")
    tagged_steps |= ns

# 4. 里程碑每步一行「写出的字节」
headings = [re.sub(r"^#{2,3} ", "", l).strip() for l in ftl_lines if re.match(r"^#{2,3} ", l)]
ms_lines = ms.splitlines()
step_lines = [(i, int(m.group(1))) for i, l in enumerate(ms_lines) if (m := re.match(r"^## 步 (\d+)", l))]
for idx, (i, n) in enumerate(step_lines):
    end = step_lines[idx + 1][0] if idx + 1 < len(step_lines) else len(ms_lines)
    block = ms_lines[i:end]
    row = next((x for x in block if x.startswith("**写出的字节**：")), None)
    if row is None:
        bad.append(f"{ms_path}:{i + 1} 「## 步 {n}」里没有「**写出的字节**：」这一行")
        continue
    if re.match(r"\*\*写出的字节\*\*：无", row):
        if n in tagged_steps:
            bad.append(f"{ms_path}:{i + 1} 步 {n} 写「写出的字节：无」，而 {ftl_path} 有节标着归它")
        continue
    quoted = re.findall(r"「([^」]+)」", row)
    if "layout/01-first-txn.md" not in row or not quoted:
        bad.append(f"{ms_path}:{i + 1} 步 {n} 的「写出的字节」既不是「无」，也没点名 layout/01-first-txn.md 里的节")
        continue
    for q in quoted:
        # 逐字等于某一节的标题，或只省掉了标题末尾的全角括注；标题的一截（「根」之于「七、发布（根记录与根槽）」）不算
        if not any(h == q or (h.startswith(q) and h[len(q):].startswith("（") and h.endswith("）")) for h in headings):
            bad.append(f"{ms_path}:{i + 1} 步 {n} 点名「{q}」，而 {ftl_path} 没有标题是它的节（只许省掉标题末尾的全角括注）")

if bad:
    print(f"  ✗ 新池新建文件的三份文件对不上 {len(bad)} 处：")                    # gate-lint:summary
    for b in bad:
        print("      " + b)                                                     # gate-lint:detail
    print(f"  → 怎么办：判「是」的未定项要同时进 {ftl_path} 的「还剩几处空白」表和 {ms_path} 某一步的「会碰到的决策点」；")
    print("    定了案的就从空白表里拿走；字节表每节标题下写「**里程碑**：… 步 N」，里程碑每步写「**写出的字节**：无」或点名字节表的节标题。")
    sys.exit(1)

if not blocking and not sections and not steps and not gap_refs:
    print(f"  ! 本次无对象可判：{dec_dir} 里没有判「是」的未定项，{ftl_path} 没有节，{ms_path} 没有步，三样一样都没比")
    sys.exit(77)
print(f"  ✓ 新池新建文件的三份文件互相挂钩（判「是」的未定项 {len(blocking)} 条、字节表 {sections} 节、里程碑 {len(steps)} 步）")
PY
}

# ── 格 admission-terms：准入不等式的每一项都有人维护（统计量，或写明的例外）
# 判据在 D5（快照 / 空间记账机制） 已定项 4：式子里出现的每一项都是被维护的统计量，做不到就写明例外是谁。
#   1. 式子的权威正文前面有一行 `<!-- gate:admission-formula -->`，全仓只许一处；取它下面第一个代码块里 `可用 =`、`可用(d) =` 或 `可用（d） =`（半角、全角括号都认）那一行，
#      按 `−` 拆成项（去掉 `Σ设备(` 与括号）。
#   2. 对照表前面有一行 `<!-- gate:admission-terms -->`，全仓只许一处；取它下面第一张表的两列。
#   3. 两边的项逐一相等：式子多一项或对照表多一项，都判红。
#   4. 对照表每一行的右列要么是「第 N 项（名字）」，N 在同一文件那张 `| # | 统计量 |` 表里、名字逐字相等、且不是已撤回的编号；
#      要么以 `**例外**：` 开头并写出理由（不少于 8 个字）。
#   5. `decisions/` 下凡是写着 `可用 = Σ设备` 的地方（索引行、挪走后留的指针），去掉空白与反引号之后必须与权威那一行逐字相等：
#      副本在反引号里的，取到收尾那个反引号；在表格行里、没有反引号的，取到那一格的竖线；都不是的取到行尾（末尾的句号、分号不算）。
#   两个锚不是各恰好一处（含一处都没有）判红、不退 77：锚没了就是式子与对照表脱了钩。decisions/ 不在，退 77。
# 射程：只判每一项有归属、归属指得到，不判归属对不对（该不该是统计量、例外的理由成不成立，靠人）；
#   不判统计量表里多出来的项（判定不止准入一处，机器列不全）。
# 样本：admission-terms-red 的式子多了「挂载期承诺量」而对照表没有、副本在权威那一行后面多写一项；field-registry-green 三项各有归属、副本逐字一致；
#   admission-terms-fullwidth-d-green 的式子写成 `可用（d） = …`（全角括号），判绿。
cell_admission_terms() {
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions/，这一格无对象可判"; exit 77; }

  BREAK_SWITCH="$BREAK_SWITCH" python3 - <<'PY'
import glob, os, re, sys

files = sorted(glob.glob('.claude/kb/decisions/*.md'))
texts = {path: open(path, encoding='utf-8').read() for path in files}

def anchors(marker):
    hits = []
    for path, text in texts.items():
        lines = text.split('\n')
        for number, line in enumerate(lines):
            if line.strip() == marker:
                hits.append((path, number, lines))
    return hits

problems = []

formula_hits = anchors('<!-- gate:admission-formula -->')
terms_hits = anchors('<!-- gate:admission-terms -->')
if len(formula_hits) != 1 or len(terms_hits) != 1:
    print(f'  ✗ 准入式子的锚 {len(formula_hits)} 处、对照表的锚 {len(terms_hits)} 处，各要恰好 1 处')
    print('     → 在准入不等式权威正文的代码块上一行写 `<!-- gate:admission-formula -->`，')
    print('       在逐项对照表上一行写 `<!-- gate:admission-terms -->`；两个锚全仓各一处，别处只留指针。')
    sys.exit(1)

formula_path, formula_line, formula_lines = formula_hits[0]
formula_text = None
inside_block = False
# 弄坏开关 DOC_REGISTRIES_BREAK=formula-fullwidth-d-unmatched：抓式子那一行只认半角 (d)（改前的写法）
formula_line_pattern = r'^可用(\(d\))?\s*=' if os.environ.get('BREAK_SWITCH') == 'formula-fullwidth-d-unmatched' else r'^可用(\(d\)|（d）)?\s*='
for line in formula_lines[formula_line + 1:]:
    if line.startswith('```'):
        if inside_block:
            break
        inside_block = True
        continue
    if inside_block and re.match(formula_line_pattern, line.strip()):
        formula_text = line.strip()
        break
if formula_text is None:
    print(f'  ✗ {formula_path}:{formula_line + 1} 的锚下面没有代码块，或代码块里没有 `可用 =` 那一行')
    print('     → 锚要紧挨着式子的代码块，式子写成一行：可用 = Σ设备( … ) − … 。')
    sys.exit(1)

right_hand_side = formula_text.split('=', 1)[1]
right_hand_side = right_hand_side.replace('(d)', '').replace('（d）', '')
right_hand_side = right_hand_side.replace('Σ设备(', ' ').replace('Σ设备（', ' ').replace('(', ' ').replace(')', ' ').replace('（', ' ').replace('）', ' ')
formula_terms = [re.sub(r'\s*÷\s*副本数$', '', term.strip()) for term in right_hand_side.split('−') if term.strip()]

terms_path, terms_line, terms_lines = terms_hits[0]
mapping = {}
table_started = False
for line in terms_lines[terms_line + 1:]:
    stripped = line.strip()
    if not stripped:
        if table_started:
            break
        continue
    if not stripped.startswith('|'):
        if table_started:
            break
        continue
    table_started = True
    cells = [cell.strip() for cell in stripped.strip('|').split('|')]
    if len(cells) < 2 or set(cells[0]) <= set('-: ') or cells[0] == '式子里的项':
        continue
    mapping[cells[0]] = cells[1]

statistics = {}
statistics_text = texts[terms_path].split('\n')
in_statistics_table = False
for line in statistics_text:
    if re.match(r'^\| # \| 统计量 \|', line):
        in_statistics_table = True
        continue
    if in_statistics_table:
        if not line.startswith('|'):
            in_statistics_table = False
            continue
        cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
        if cells and cells[0].isdigit():
            statistics[int(cells[0])] = cells[1].replace('**', '')

if not statistics:
    problems.append(f'{terms_path}: 找不到表头是 `| # | 统计量 |` 的统计量表，对照表的「第 N 项」没处可指')

formula_set, mapping_set = set(formula_terms), set(mapping)
for term in formula_terms:
    if term not in mapping_set:
        problems.append(f'式子里有「{term}」，对照表（{terms_path}）里没有这一行')
for term in mapping:
    if term not in formula_set:
        problems.append(f'对照表（{terms_path}）里有「{term}」，式子（{formula_path}）里没有这一项')

exceptions = []
for term, owner in mapping.items():
    if owner.startswith('**例外**：'):
        reason = owner[len('**例外**：'):].strip()
        if len(reason) < 8:
            problems.append(f'「{term}」写了例外，但理由不到 8 个字：{owner}')
        exceptions.append(term)
        continue
    match = re.match(r'^第 (\d+) 项（(.+)）$', owner)
    if not match:
        problems.append(f'「{term}」的归属写成了「{owner}」，要么「第 N 项（名字）」、要么「**例外**：理由」')
        continue
    number, name = int(match.group(1)), match.group(2)
    if number not in statistics:
        problems.append(f'「{term}」指到第 {number} 项，统计量表里没有这一行')
    elif '已撤回' in statistics[number]:
        problems.append(f'「{term}」指到第 {number} 项，而那一项已撤回')
    elif statistics[number] != name:
        problems.append(f'「{term}」写的是第 {number} 项（{name}），统计量表里第 {number} 项叫「{statistics[number]}」')

canonical = re.sub(r'[\s`]', '', formula_text)
copies = 0
for path, text in texts.items():
    for number, line in enumerate(text.split('\n'), 1):
        position = line.find('可用(d) =')
        if position < 0:
            position = line.find('可用 = Σ设备')
        if position < 0:
            position = line.find('可用 =Σ设备')
        if position < 0:
            continue
        copies += 1
        fragment = line[position:]
        if '`' in line[:position]:
            fragment = fragment.split('`')[0]
        elif line.lstrip().startswith('|'):
            fragment = fragment.split('|')[0]
        fragment = re.sub(r'[\s`]', '', fragment).rstrip('。；;，,')
        # 逐字相等，不是只比开头：副本在权威那一行后面多写一项（「… − 多一项」）也要红
        if fragment != canonical:
            problems.append(f'{path}:{number} 的式子副本与权威那一行（{formula_path}）对不上')

if problems:
    print(f'  ✗ 准入不等式与统计量清单的逐项对照有 {len(problems)} 处不对')
    for problem in problems:
        print(f'     {problem}')  # gate-lint:detail
    print('     → 式子里每一项都要在对照表里有一行：被维护的统计量写「第 N 项（名字）」，做不到的写「**例外**：理由」')
    print('       （2026-09-13 用户定案，D5（快照 / 空间记账机制） 已定项 4）；式子改了，先改对照表，再改各处副本。')
    sys.exit(1)

print(f'  ✓ 准入不等式 {len(formula_terms)} 项逐项有归属：统计量 {len(formula_terms) - len(exceptions)} 项，写明的例外 {len(exceptions)} 项（{"、".join(exceptions)}）；式子副本 {copies} 处与权威一致')
PY
}

# ── 格 segment-registry：段序列登记表与钉住活代码的用例逐字比对（钉不到用例的行与 E142 产物比，产物不在报本次未跑）
# 落的是 C316（提交步骤的登记位有四处且互不相同） 的第①半：layout/01-first-txn.md「八、」登记表的段序列是人抄的。
# 转发给 scripts/crash/check-segment-registry.py，判法不写第二份（在它的文档字符串里）：出处指到用例、或 path 登记了钉住它的 crates 用例的，与用例比；
#   钉不到用例的那几行才与 E142 产物比，产物不在它退 77、这一格跟着退 77（research/ 整个可删，.claude/rules/directory-layout.md）；
#   它退 1（对不上）与 2（结构认不出、文件不在）都记红。
# 它的 --selftest 由 code-tooling 跑，这里不重复跑。
# 脚本按本阶段自己的位置取（$SEGMENT_REGISTRY_SCRIPT），不按 cwd 取，`--root` 指被判的仓：样本会把 cwd 换成样本目录，
#   按 cwd 取到的是样本里的拷贝，仓里的脚本退化了样本照样判对。脚本不在退 1 不退 77：它随仓走，不在就是被删了或挪了。
# 样本：segment-registry-red 只放脚本读的三样合成输入（layout 表的「八、」一节、replay.sh 里 E142 那一行、产物里几行 name=segments；
#   合成的 path 名 a、b… 不在脚本的 PATH_PINNING_TESTS 里，走的是与产物比那一支），
#   十行各埋一种错，另把整条流的状态数、覆盖写固定脚本流的写数与数组与用例里的状态数各错一处，脚本每条逐行比对的分支都有一处会红，逐条点名；
#   field-registry-green 一致，判绿。「声明的 path 集合剩不下恰好一条」走退 2 那一支由 segment-registry-paths-red 判（脚本退 2、这一格退 1）；
#   segment-registry-product-absent 只能与产物比、replay.sh 指的产物不在：脚本退 77，这一格与整道退 77（本次未跑）；
#   与钉住的 crates 用例比那一支与产物行的解析由脚本自己的 --selftest 测（输入是真仓的布局表与用例、脚本旁边的样本，不读 research/）。
cell_segment_registry() {
  if [[ ! -f "$SEGMENT_REGISTRY_SCRIPT" ]]; then
    echo "  ✗ 找不到 $SEGMENT_REGISTRY_SCRIPT"
    echo "     → 怎么办：它随仓走（scripts/ 下），不在就是被删了或挪了：从 git 找回来，挪了就改这个阶段里 SEGMENT_REGISTRY_SCRIPT 那一行的路径。"
    exit 1
  fi
  local segment_registry_status=0
  python3 "$SEGMENT_REGISTRY_SCRIPT" --root "$ROOT" || segment_registry_status=$?
  # 77：只能与产物比的那几处产物不在，脚本已经逐处报了本次未跑；别的非 0 都记红
  if ((segment_registry_status == 0)) || { ((segment_registry_status == 77)) && [[ "$BREAK_SWITCH" != segment-registry-not-run-as-red ]]; }; then
    exit "$segment_registry_status"
  fi
  exit 1
}

# ── 格 format-const-placeholders：格式常量文件里的占位，每个都指得到一条真实存在的分项或欠账
# 占位的写法（crates/singlefs-format/src/*.rs，常量前一行；`/// placeholder:`、`//! placeholder:` 一样认）：
#   // placeholder: D22（单元原子性怎么合成） 已定项 2 —— 为什么还是预想
#   // placeholder: C323（镜像大小全仓没有条款） —— 为什么还是预想
# 判据：编号带简称；D<n> 的分项号在 .claude/kb/decisions/<n>-*.md 的「### 已定项 / ### 未定项」两张表里能找到那一行，
# C<n> 在 .claude/kb/checks-owed.md 里有登记行；简称与登记位一致归 doc-lint，这里只核编号与分项号。
# 没有常量目录、或一个占位都没有，退 77。
# 样本：format-const-placeholders-red 里一个占位指已定项 99、一个把只在已定项表里的第 2 行写成未定项、一个没写编号、
# 一个文档注释写法的指到不存在的 D99；field-registry-green 里一个 D 占位、一个 C 占位都指得到。
# D<n> 取哪一份决策文件：lib-item-ref-status.py 的 numbered_files，代码是中文这一侧，取中文原文。
# 样本：format-const-placeholders-translation-green / -red（中文原文有 / 只有译文有那一条）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_format_const_placeholders() {
  [[ -d crates/singlefs-format/src ]] || { echo "  ! 没有 crates/singlefs-format/src，这一格无对象可判（步 0 之前没有常量文件）"; exit 77; }

  python3 - "$ITEM_REF_STATUS_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys
spec = importlib.util.spec_from_file_location('item_ref_status', sys.argv[1])
item_ref_status = importlib.util.module_from_spec(spec); spec.loader.exec_module(item_ref_status)
bad = []
placeholders = []
for path in sorted(glob.glob('crates/singlefs-format/src/**/*.rs', recursive=True)):
    for number, line in enumerate(open(path, encoding='utf-8'), 1):
        m = re.match(r'\s*//[/!]?\s*placeholder:\s*(.*)$', line)
        if not m:
            continue
        text = m.group(1).strip()
        placeholders.append((path, number, text))
        d = re.match(r'D(\d+)（[^）]+）\s*(已定项|未定项)\s*(\d+)', text)
        c = re.match(r'C(\d+)（[^）]+）', text)
        if d:
            n, kind, k = d.group(1), d.group(2), d.group(3)
            # 代码是中文这一侧，取中文原文（numbered_files）；改前取 glob 不排序的第一个，取到哪份译文看目录项的次序
            files = item_ref_status.numbered_files('.claude/kb/decisions', n, path)
            if not files:
                bad.append((path, number, f'D{n} 没有决策文件')); continue
            body = open(files[0], encoding='utf-8').read()
            # 索引行 `| k | **…` 或正文标题 `#### 已定项 k（`，两种形态任一命中即可。
            # 索引行只在同名那一节（`### 已定项` / `### 未定项`）第一个 #### 之前找：不分节时 `未定项 2` 会被已定项表的第 2 行放行
            section = re.search(rf'^### {kind}[ \t]*$(.*?)(?=^#{{2,4}} |\Z)', body, re.M | re.S)
            index_table = section.group(1) if section else ''
            hit = re.search(rf'^\|\s*{k}\s*\|', index_table, re.M) or re.search(rf'^#{{3,5}}\s*{kind}\s*{k}[（:：]', body, re.M)
            if not hit:
                bad.append((path, number, f'D{n} 里找不到 {kind} {k}'))
        elif c:
            n = c.group(1)
            owed = open('.claude/kb/checks-owed.md', encoding='utf-8').read()
            if not re.search(rf'^\|\s*C{n}\s*\|', owed, re.M):
                bad.append((path, number, f'checks-owed.md 里找不到 C{n} 的登记行'))
        else:
            bad.append((path, number, '占位没写成「D<n>（简称） 已定项/未定项 k」或「C<n>（简称）」'))
if bad:
    print(f'  ✗ 格式常量文件里有 {len(bad)} 个占位指不到分项或欠账：')
    for path, number, why in bad:
        print(f'     {path}:{number}  {why}')  # gate-lint:detail
    print('     → 怎么办：占位那一行写成「// placeholder: D<n>（简称） 已定项 k —— 为什么还是预想」或「// placeholder: C<n>（简称） —— …」，')
    print('       编号要在 .claude/kb/decisions/ 的索引表或 .claude/kb/checks-owed.md 里真的有那一行；分项定了就把占位行删掉。')
    sys.exit(1)
if not placeholders:
    print('  ! 本次无对象可判：crates/singlefs-format/src 下一个 `// placeholder:` 占位都没有，没有编号可核')
    sys.exit(77)
print(f'  ✓ 格式常量文件里的占位都指得到分项或欠账（{len(placeholders)} 个占位：' + '；'.join(t.split(' —— ')[0] for _, _, t in placeholders) + '）')
PY
}

# ── 格 second-txn-hooks：覆盖写、释放、回退与复用的三处互相挂钩
# 里程碑「覆盖写、释放、回退与复用」的字节登记在三处：`layout/02-second-txn.md` 那张表登记第一次出现的盘上形态，
# 每行指里程碑的一步与钉住它的用例；里程碑每步的「**写出的字节**：」点名 layout/01 的节与 layout/02 的行；
# `layout/01-first-txn.md`「八、根槽写路径的段序列登记表」的行标「里程碑「覆盖写、释放、回退与复用」步 N」。
# 新池新建文件那三份归 first-txn-hooks。
# 判据（每条都报绝对数）：
#   0. layout/02 表每一行都要六格（形态 / 第一次出现在哪次发布 / 写成什么 / 决策分项 / 里程碑那一步 / 钉住它的用例），少一格判红，不静默丢掉。
#   1. layout/02 表每一行「里程碑那一步」点名的步 N 在里程碑里存在，且步 N 的「写出的字节」行用「」点名了这一行的形态
#      （前缀即可，但这个前缀只许对得上一行：同时是几行形态的前缀，就指不出点的是哪一行，不算点名，判红）。
#   2. 里程碑里每一行「写出的字节」用「」点名的东西，要么是 layout/01 的节标题（子串即可），要么是 layout/02 表某一行形态的前缀
#      （同样只许对得上一行）；点名 layout/02 某一行的步 N，那一行的「里程碑那一步」里要有步 N。
#   3. layout/02 表「钉住它的用例」里反引号的 `*.rs` 在 `crates/*/tests/` 下存在，反引号的函数名在同一格点名的文件里有 `fn 名字`。
#   4. 步 N 的「写出的字节」点名了 layout/01 的「八、」那一节，当且仅当「八、」那张表有一行写「里程碑「覆盖写、释放、回退与复用」步 N」。
# 射程：只认「」引起来的点名与反引号里的用例名；点名的内容与字节对不对，这里不判（字节由用例与崩溃点重放钉）。三份文件缺一份，退 77。
# 样本：second-txn-hooks-red 里一行指的步没点名它、一步点名了不存在的节、用例文件不在、用例函数改了名、
# 「八、」表标着的步没点名那一节、点名「八、」的步表里没有、一行只有四格、点名写成两行的共同前缀；field-registry-green 三处互相点名。
cell_overwrite_release_rollback_reuse_hooks() {
  local L1=.claude/kb/layout/01-first-txn.md
  local L2=.claude/kb/layout/02-second-txn.md
  local MS=.claude/kb/milestone/02-second-txn.md
  [[ -f "$L1" && -f "$L2" && -f "$MS" ]] || { echo "  ! 找不到 $L1 / $L2 / $MS，这一格无对象可判"; exit 77; }

  python3 - "$L1" "$L2" "$MS" <<'PY'
import glob, re, sys

l1_path, l2_path, ms_path = sys.argv[1:4]
body = lambda path: open(path, encoding="utf-8").read().split("\n## 历史版本")[0]
l1, l2, ms = body(l1_path), body(l2_path), body(ms_path)
headings = [re.sub(r"^#{2,3} ", "", line).strip() for line in l1.splitlines() if re.match(r"^#{2,3} ", line)]
split_cells = lambda line: [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
plain = lambda text: text.replace("`", "")

rows, in_table = [], False
for line in l2.splitlines():
    if line.startswith("| 形态 |"):
        in_table = True
        continue
    if in_table and line.startswith("|---"):
        continue
    if in_table and line.startswith("|"):
        rows.append(split_cells(line))
        continue
    in_table = False

bad = []
if not rows:
    bad.append(f"{l2_path} 里找不到表头是「| 形态 |」的登记表，或表里一行都没有")
short_rows = [row for row in rows if len(row) < 6]
for row in short_rows:
    bad.append(f"{l2_path}「{row[0][:30]}」这一行只有 {len(row)} 格，登记表每行要 6 格（形态 / 第一次出现在哪次发布 / 写成什么 / 决策分项 / 里程碑那一步 / 钉住它的用例）")
rows = [row for row in rows if len(row) >= 6]

ms_lines = ms.splitlines()
step_starts = [(index, int(match.group(1))) for index, line in enumerate(ms_lines) if (match := re.match(r"^## 步 (\d+)", line))]
step_bytes_line = {}
for position, (start, number) in enumerate(step_starts):
    end = step_starts[position + 1][0] if position + 1 < len(step_starts) else len(ms_lines)
    step_bytes_line[number] = next((line for line in ms_lines[start:end] if line.startswith("**写出的字节**：")), "")

def rows_named_by(quote):
    """「」里的串点名 layout/02 的哪一行：它是那一行形态的前缀，而且只对得上这一行。同时是几行的前缀就指不出是哪一行，不算点名。"""
    matched = [row for row in rows if plain(row[0]).startswith(plain(quote))]
    return matched if len(matched) == 1 else []

def ambiguous_prefix(quote):
    matched = [row for row in rows if plain(row[0]).startswith(plain(quote))]
    return matched if len(matched) > 1 else []

# 1. 登记表一行 → 里程碑那一步
row_links = 0
for row in rows:
    for number in [int(x) for x in re.findall(r"步 (\d+)", row[4])]:
        row_links += 1
        if number not in step_bytes_line:
            bad.append(f"{l2_path}「{row[0][:30]}」指里程碑步 {number}，而 {ms_path} 里没有「## 步 {number}」")
            continue
        quotes = re.findall(r"「([^」]+)」", step_bytes_line[number])
        if not any(row in rows_named_by(quote) for quote in quotes):
            bad.append(f"{l2_path}「{row[0][:30]}」指里程碑步 {number}，而步 {number} 的「写出的字节」没点名这一行")

# 2. 里程碑每行「写出的字节」的点名都落得到
quotes_checked = 0
for index, line in enumerate(ms_lines):
    if not line.startswith("**写出的字节**："):
        continue
    step_number = next((number for start, number in reversed(step_starts) if start < index), None)
    for quote in re.findall(r"「([^」]+)」", line):
        quotes_checked += 1
        named_rows = rows_named_by(quote)
        shared_prefix = ambiguous_prefix(quote)
        if shared_prefix:
            bad.append(f"{ms_path}:{index + 1} 点名「{quote[:40]}」，而它是 {l2_path} 里 {len(shared_prefix)} 行形态的共同前缀"
                       f"（{'、'.join(row[0][:20] for row in shared_prefix)}），指不出点的是哪一行，不算点名")
            continue
        if not named_rows and not any(quote in heading for heading in headings):
            bad.append(f"{ms_path}:{index + 1} 点名「{quote[:40]}」，而 {l1_path} 没有这一节、{l2_path} 表里也没有这一行")
            continue
        for row in named_rows:
            if step_number is not None and str(step_number) not in re.findall(r"步 (\d+)", row[4]):
                bad.append(f"{ms_path}:{index + 1} 步 {step_number} 点名 {l2_path}「{row[0][:30]}」，而那一行的「里程碑那一步」里没有步 {step_number}")

# 3. 钉住它的用例在仓里
tests_checked = 0
for row in rows:
    tokens = re.findall(r"`([^`]+)`", row[5])
    files = [token for token in tokens if token.endswith(".rs")]
    functions = [token for token in tokens if re.fullmatch(r"[a-z_][a-z0-9_]*", token)]
    paths = []
    for file_name in files:
        tests_checked += 1
        found = glob.glob(f"crates/*/tests/{file_name}")
        if not found:
            bad.append(f"{l2_path}「{row[0][:30]}」点名的用例文件 {file_name} 在 crates/*/tests/ 下不存在")
        paths += found
    sources = [open(path, encoding="utf-8").read() for path in paths]
    for function in functions:
        tests_checked += 1
        if not any(re.search(rf"\bfn {re.escape(function)}\b", source) for source in sources):
            bad.append(f"{l2_path}「{row[0][:30]}」点名的用例 {function} 在同一格点名的文件里找不到 fn")

# 4. 段序列登记表的行与里程碑各步对得上
registry = re.search(r"^## 八、.*?(?=^## |\Z)", l1, re.M | re.S)
registry_steps = set()
if registry:
    for line in registry.group(0).splitlines():
        if line.startswith("|"):
            registry_steps |= {int(x) for x in re.findall(r"里程碑「覆盖写、释放、回退与复用」\s*步 (\d+)", line)}
else:
    bad.append(f"{l1_path} 里没有「## 八、」那一节（段序列登记表）")
steps_naming_registry = {number for number, line in step_bytes_line.items() if re.search(r"「八、[^」]*段序列登记表", line)}
for number in sorted(registry_steps - steps_naming_registry):
    bad.append(f"{l1_path}「八、」有一行标着里程碑「覆盖写、释放、回退与复用」步 {number}，而步 {number} 的「写出的字节」没点名「八、」那一节")
for number in sorted(steps_naming_registry - registry_steps):
    bad.append(f"{ms_path} 步 {number} 的「写出的字节」点名了「八、」那一节，而那张表里没有一行标「里程碑「覆盖写、释放、回退与复用」步 {number}」")

if bad:
    print(f"  ✗ 覆盖写、释放、回退与复用的三处挂钩对不上 {len(bad)} 处：")    # gate-lint:summary
    for item in bad:
        print("      " + item)                                # gate-lint:detail
    print(f"  → 怎么办：{l2_path} 每一行的「里程碑那一步」与那一步的「**写出的字节**：」互相点名（「形态」原样引）；")
    print(f"    点名的节要在 {l1_path} 里存在，用例文件与函数名要在 crates/*/tests/ 里存在；")
    print(f"    某一步写了「八、」段序列那一节，那张表就要有一行标「里程碑「覆盖写、释放、回退与复用」步 N」，反过来也一样。")
    sys.exit(1)

print(f"  ✓ 覆盖写、释放、回退与复用的三处互相挂钩（登记表 {len(rows)} 行、行到步 {row_links} 处、「写出的字节」点名 {quotes_checked} 处、用例 {tests_checked} 个、段序列登记表标到的步 {len(registry_steps)} 个）")
PY
}

# ── 格 tree-table-reserve：树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
# 补的是 C338（树表条目预留没有余量）要的算和检查：合计靠人算会漂。
# 表的形态（.claude/kb/decisions/08-核心索引结构.md 已定项 11）：
#   <!-- gate:tree-table-reserve total=76 -->
#   | 认购者 | 住哪条条目 | 宽度 | 出处 |
#   |---|---|---|---|
#   | … | 头条目 | 8 | C271（…） |
# 判据三条：① 宽度列全是正整数；② 合计不超过标记里的 total；③ 每行「出处」要带一个编号
#（C<n> 在 .claude/kb/checks-owed.md 有登记行，D<n> 在 .claude/kb/decisions/<n>-*.md 存在；编号的位数不设上限，后面紧跟全角括号里的简称）。
# 带标记的表不是恰好一张（含一张都没有）判红；认购行不足四格（少了宽度或出处）判红，不静默丢掉；标记下面一条认购都没有，退 77。
# 内嵌 python 的退出码要取，而且必须报出 CHECKED 那一行：它崩了（例如读不到 checks-owed.md）时不许走绿。
# 样本：tree-table-reserve-red 里认购合计 88 超过预留 76、一行只有三格；field-registry-green 三条认购合计 32、余 44；
#   tree-table-reserve-wide-number-green 的出处指到四位的欠账号 C1000 与三位的决策号 D105（样本里都有登记行或正文），判绿。
cell_tree_table_reserve() {
  local report python_rc
  report="$(BREAK_SWITCH="$BREAK_SWITCH" python3 - <<'PY'
import glob, os, re, sys

marker = re.compile(r"<!--\s*gate:tree-table-reserve\s+total=(\d+)\s*-->")
files = sorted(glob.glob(".claude/kb/decisions/*.md"))
found = []
for path in files:
    lines = open(path, encoding="utf-8").read().split("\n")
    for index, line in enumerate(lines):
        matched = marker.search(line)
        if matched:
            found.append((path, index, int(matched.group(1)), lines))

if len(found) != 1:
    print("BAD", f"带 gate:tree-table-reserve 标记的表应当恰好有一张，实际 {len(found)} 张", sep="\t")
    print("CHECKED", 0, 0, 0, sep="\t")
    sys.exit()

path, index, total, lines = found[0]
rows = []
for line in lines[index + 1:]:
    if not line.startswith("|"):
        if rows:
            break
        continue
    cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
    if cells[0] in ("认购者",) or cells[0].startswith("---"):
        continue
    if len(cells) < 4:
        print("BAD", f"「{cells[0]}」这一行只有 {len(cells)} 格，认购表每行要四格（认购者 / 住哪条条目 / 宽度 / 出处），它的宽度没进合计", sep="\t")
        continue
    rows.append(cells)

owed = open(".claude/kb/checks-owed.md", encoding="utf-8").read()
subtotal = 0
for cells in rows:
    name, width_text, source = cells[0], cells[2], cells[3]
    if not re.fullmatch(r"\d+", width_text):
        print("BAD", f"「{name}」的宽度列不是正整数：{width_text}", sep="\t")
        continue
    subtotal += int(width_text)
    # 弄坏开关 DOC_REGISTRIES_BREAK=owed-number-width-fixed：编号位数写死（C 三位、D 两位，改前的写法）
    number_pattern = r"(C\d{1,3}|D\d{1,2})（" if os.environ.get("BREAK_SWITCH") == "owed-number-width-fixed" else r"(C\d+|D\d+)（"
    numbers = re.findall(number_pattern, source)
    if not numbers:
        print("BAD", f"「{name}」的出处里没有带简称的编号", sep="\t")
        continue
    for number in numbers:
        if number.startswith("C"):
            if f"| {number} |" not in owed:
                print("BAD", f"「{name}」指的 {number} 在 checks-owed.md 里没有登记行", sep="\t")
        elif not (glob.glob(f".claude/kb/decisions/{int(number[1:])}-*.md")
                  or glob.glob(f".claude/kb/decisions/{int(number[1:]):02d}-*.md")):
            print("BAD", f"「{name}」指的 {number} 在 decisions/ 下没有正文", sep="\t")

if subtotal > total:
    print("BAD", f"认购合计 {subtotal} 超过预留 {total}", sep="\t")
print("CHECKED", len(rows), subtotal, total, sep="\t")
PY
)"
  python_rc=$?
  if ((python_rc != 0)) || ! grep -q '^CHECKED' <<<"$report"; then
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "  ✗ 认购表没算完：内嵌 python 退出码 $python_rc，$(grep -q '^CHECKED' <<<"$report" && echo '报了' || echo '没报出') CHECKED 那一行"
    echo "    → 怎么办：单独跑一遍这一格（--check tree-table-reserve）看 python 的报错（多半是 .claude/kb/checks-owed.md 或决策文件读不到）；"
    echo "      没算完就是什么都没查，成功句里的数会是空的，而它看着与判过了一模一样。"
    exit 1
  fi

  local -a bad
  local rows subtotal total why
  mapfile -t bad < <(grep '^BAD' <<<"$report")
  read -r _ rows subtotal total < <(grep '^CHECKED' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 树表条目预留的认购表不合规："
    while IFS=$'\t' read -r _ why; do
      printf '      %s\n' "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11，标记那一行写着 total=<预留字节数>。"
    echo "      合计超了就先去掉一个认购者（或把那一段的宽度压小），别直接改 total——预留多少由那条已定项定；"
    echo "      出处指不到编号的，补一个还开着的欠账号或一条已定分项，编号要带简称。"
    exit 1
  fi
  if [[ "$rows" == 0 ]]; then
    echo "  ⊘ 本次无对象可判：gate:tree-table-reserve 标记下面一条认购都没有（预留 ${total}）"
    exit 77
  fi
  echo "  ✓ 树表条目预留的认购表对得上（${rows} 条认购，合计 ${subtotal} 字节、预留 ${total}、余 $((total - subtotal))；每条的出处都指得到一个编号）"
}

# ════════════════ 欠账与收口组的五格（欠账表与里程碑收口表） ════════════════

# ── 格 table-shape：欠账表两张登记表的行形状（已还清那张不许混进六列的欠账行，欠着那张不许留写着已还的行）
# 判据：`.claude/kb/checks-owed.md` 有两张登记表——欠着的那张六列（编号 / 简称 / 要拦什么 / 怎么拦 / 前置 / 出处），
# 「### 已还清」那张四列（编号 / 简称 / 怎么还的 / 还清日期）。一行的格数与它所在那张表的表头对不上，判红；
# 欠着的那张里，一行有哪一格（第 3 列起）开头写着「已还」而后面不是「一半 / 这一半 / 大半」、或开头写着「销账」「作废」
# （前面带 ⚠️ 与日期都算），也判红：还清了就挪走，只还了一部分就把开头写成还欠什么（「已还一半」「决策已定、检查仍欠」「前置已还」）。
# 编号行前面没有表头（被空行从上一张表断开）判红。
# 只看紧挨着 `doc-lint:registry` 标记的表，历史版本里别的表不管；带编号行而没判的表，成功行逐张列出（现算）。
# 「同一个编号在两张表里各登记一次」不在这一格判：两张表都是 kb 的登记位，归上游 doc-lint 的 F（一个编号只许有一处登记位）逐个编号判。
# 样本：checks-owed-red 的欠账表里欠着那张有一行写「已还（日期）」、一行「日期 作废」、一行「日期 销账」，一行被空行断开、前面没有表头，
# 已还清那张混进一行六格的；checks-owed-green 里欠着那张的「已还一半」「已还这一半」「前置已还」、已还清那张里写「销账」的都放行。
cell_table_shape() {
  local F="$ROOT/.claude/kb/checks-owed.md"
  if [[ ! -f "$F" ]]; then
    echo "  ✗ 找不到 $F"
    echo "  → 怎么办：欠账表挪了位置就同步改这个阶段里的路径。"
    exit 1
  fi
  python3 - "$F" <<'PY'
import re, sys
lines = open(sys.argv[1], encoding='utf-8').read().split('\n')
tables, cur, section, marked = [], None, '（开头）', False
headless, in_history = [], False
for i, l in enumerate(lines, 1):
    s = l.strip()
    if s.startswith('<!-- doc-lint:registry'):
        marked = True; cur = None; continue
    if l.startswith('#'):
        section = l.lstrip('#').strip(); cur = None; marked = False
        in_history = in_history or section.startswith('历史版本')
        continue
    if not s.startswith('|'):
        cur = None
        if s:
            marked = False
        continue
    cells = re.sub(r'\\\|', '', s).strip('|').split('|')
    if cur is None:
        # 一张表的第一行就是编号行，说明它前面没有表头：多半是被一个空行从上一张表断开的。
        # Markdown 在空行处结束表格，断开之后的行渲染不成表，也不在 registry 标记管的那张表里。
        if re.match(r'\|\s*C\d+\s*\|', s) and not in_history:
            headless.append(f'第 {i} 行 {s.split("|")[1].strip()}：前面没有表头（上一行是空行或别的正文）')
        cur = {'sec': section, 'width': len(cells), 'head': i, 'rows': [], 'reg': marked}
        tables.append(cur); marked = False; continue
    if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', s):
        continue
    m = re.match(r'\|\s*(C\d+)\s*\|', s)
    if m:
        cur['rows'].append((i, m.group(1), cells))
reg = [t for t in tables if t['reg'] and t['rows']]
if len(reg) < 2:
    print(f'  ✗ 只找到 {len(reg)} 张带编号行的登记表，欠着的与已还清的两张应当都在')
    print('  → 怎么办：表头或它上面的 doc-lint:registry 标记被改坏了；按文件开头的约定把两张表恢复出来。')
    sys.exit(1)
paid_claim = re.compile(r'(?:20\d\d-\d\d-\d\d\s*)?(已还(?!一半|这一半|大半)|销账|作废)')
bad, claims, checked = [], [], 0
for t in reg:
    for ln, num, cells in t['rows']:
        checked += 1
        if len(cells) != t['width']:
            bad.append(f"第 {ln} 行 {num}：{len(cells)} 格，而它所在那张表（「{t['sec']}」下、表头在第 {t['head']} 行）是 {t['width']} 格")
        if t['sec'] == '已还清':
            continue
        for column_index, cell in enumerate(cells[2:], start=3):
            lead = re.sub(r'^[\s*⚠️]+', '', cell)
            if paid_claim.match(lead):
                claims.append(f"第 {ln} 行 {num}：第 {column_index} 列开头写着「{lead.replace('*', '')[:16]}…」")
if bad or claims or headless:
    print(f'  ✗ 欠账表有 {len(bad)} 行的格数与所在表的表头对不上、{len(claims)} 处还在欠着的表里却写着已还、{len(headless)} 处被空行断开：')  # gate-lint:summary
    for b in bad:
        print('      ' + b)  # gate-lint:detail
    for claim in claims:
        print('      ' + claim)  # gate-lint:detail
    for fragment in headless:
        print('      ' + fragment)  # gate-lint:detail
    print('  → 怎么办：六列的欠账行挪回欠着的那张表，接在它最后一行后面，不接在文件末尾；表中间的空行删掉，一张表从表头到最后一行不许断；')
    print('            真还清了的，按「已还清」那张表的四列改写（编号 / 简称 / 怎么还的 / 还清日期）再挪过去，欠着的那张里删掉；')
    print('            只还了一部分的，把那一格的开头改成写明还欠什么的说法（「已还一半」「决策已定、检查仍欠」「前置已还」）。')
    sys.exit(1)
print(f'  ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 {checked} 行，{len(reg)} 张表）')
# 没判的：带 C 编号行、却不紧挨着 registry 标记的表（历史版本里的、别处举例的），逐张列出，清单与被扫集合出自同一趟解析
unjudged_tables = [t for t in tables if not t['reg'] and t['rows']]
print(f"    没判 {len(unjudged_tables)} 张带编号行的表（没有紧挨着的 doc-lint:registry 标记）"
      + ('：' + '；'.join(f"「{t['sec']}」下表头在第 {t['head']} 行、{len(t['rows'])} 行" for t in unjudged_tables) if unjudged_tables else ''))
PY
}

# ── 格 closeout-collects-open：里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号
# 判据：
#   ① 收口表用一行标记指明：`<!-- milestone:closeout-table -->`，紧挨着写在表上方（中间只许空行），一份里程碑文件至多一处。
#   ② 还开着的欠账号：checks-owed.md 里「### 已还清」标题之前、表格首列是 C<编号> 的行（按 `lib-owed.py` 读）。
#   ③ 带标记的文件，全文（含历史版本节）出现的每个 C<编号>，若还开着，要么在收口表的某一行里出现，
#      要么在表后（表结束到下一个标题之前）有一行显式豁免，形态：`- 不收口 C<编号>（简称）：为什么不收`。
#   任一个开着的编号两处都没有，红。豁免行写坏（没有编号、冒号后面是空的）、同一个编号既豁免又进了表、
#   标记后面不是表、一份文件两处标记，也红。
#   ④ 收口表每一行的首列（表头 `#` 与分隔行除外）要么是纯十进制数字，要么是圆圈号 `①`–`⑳`；
#      带撇号（U+2032、U+2033、ASCII 单引号）或紧跟字母的，红，指名那一行（为什么不许后缀见 path-moves.md「变体起新名字，不用角标」）。
#      全仓的 U+2032、U+2033 另由 doc-text 判；这一条多管的是 ASCII 单引号、字母与其余一切非顺序号的写法，只在收口表首列。
# 没有标记的里程碑文件不判，成功行逐个列出（现算）；一份带标记的文件都没有，退 77。
# 管不到的：表里那一行写的去向对不对、豁免的理由站不站得住（靠人）；编号只按字面 C<数字> 认，
# 写成「C 374」或只写简称的引用看不见；已还清的编号、checks-owed.md 里没有的编号一律不管。
# 样本：checks-owed-red 放一个漏收的开着编号、一个既豁免又进表的编号、一行空理由的豁免、
# 一份标记后面没有表的文件、一行字母后缀与一行 ASCII 单引号后缀的行号，判红；checks-owed-green 放表里一个、豁免一个、已还清一个、
# 一份不带标记的文件，收口表里一个圆圈号、一个一位数与一个两位数的行号，判绿并报对数
# （checks-owed-green 里 closeout-row27-preconditions 那一格的 02-second-txn.md 也带标记，这一格一并判它）。
cell_closeout_collects_open() {
  python3 - "$OWED_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys

owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[1])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)

milestone_dir = ".claude/kb/milestone"
owed_path = ".claude/kb/checks-owed.md"
marker = "<!-- milestone:closeout-table -->"
owed_number = re.compile(r"(?<![A-Za-z0-9_])(C[0-9]+)(?![0-9])")
exemption_start = re.compile(r"^- 不收口 ")
exemption_form = re.compile(r"^- 不收口 (?P<number>C[0-9]+)(?:（.*?）)?：(?P<reason>.*\S.*)$")
circled_row_numbers = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳"

milestone_files = [p for p in sorted(glob.glob(os.path.join(milestone_dir, "*.md")))
                   if not p.endswith((".en.md", ".ja.md"))]
if not milestone_files:
    print(f"  ! {milestone_dir} 下没有里程碑文件，本阶段无对象可判")
    sys.exit(77)

marked_files = []
unmarked_files = []
for path in milestone_files:
    lines = open(path, encoding="utf-8").read().split("\n")
    if any(line.strip() == marker for line in lines):
        marked_files.append((path, lines))
    else:
        unmarked_files.append(os.path.basename(path))

if not marked_files:
    print(f"  ! {milestone_dir} 下 {len(milestone_files)} 份里程碑文件没有一份带收口表标记 {marker}，本阶段无对象可判；没判：{'、'.join(unmarked_files)}")
    print(f"    要让本阶段判一份里程碑：在它的收口表上方单独写一行 {marker}，表后逐行写豁免「- 不收口 C<编号>（简称）：为什么不收」。")
    sys.exit(77)

if not os.path.isfile(owed_path):
    print(f"  ✗ 没有 {owed_path}，分不出哪些欠账号还开着")
    print(f"     → 怎么办：确认欠账表还在 {owed_path}；搬过家就照 .claude/rules/path-moves.md 改，并改本阶段的路径。")
    sys.exit(1)

owed_table = owed_library.read_owed_table(owed_path)
open_owed_names = owed_table.open_names
found_paid_heading = owed_table.paid_heading_found
if not found_paid_heading or not open_owed_names:
    print(f"  ✗ {owed_path} 里认不出还开着的欠账：找到「已还清」标题 {found_paid_heading}，标题之前首列是 C<编号> 的表格行 {len(open_owed_names)} 行")
    print(f"     → 怎么办：本阶段按「### 已还清 之前、表格首列是 C<编号>」认开着的账；欠账表改了形状，就同步改本阶段的解析，别让它对着一张认不出的表判绿。")
    sys.exit(1)

failed = False
per_file_counts = []
for path, lines in marked_files:
    name = os.path.basename(path)
    marker_line_indexes = [index for index, line in enumerate(lines) if line.strip() == marker]
    if len(marker_line_indexes) > 1:
        failed = True
        print(f"  ✗ {path} 有 {len(marker_line_indexes)} 处收口表标记（第 {', '.join(str(index + 1) for index in marker_line_indexes)} 行）")
        print(f"     → 怎么办：一份里程碑只留一张收口表、一处标记；另一张表要收的项并进收口表。")
        continue
    index = marker_line_indexes[0] + 1
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index >= len(lines) or not lines[index].lstrip().startswith("|"):
        failed = True
        print(f"  ✗ {path} 第 {marker_line_indexes[0] + 1} 行的收口表标记后面第一处非空内容不是表格")
        print(f"     → 怎么办：把 {marker} 挪到收口表表头的正上方（中间只许空行）；还没有表就先删掉标记，本阶段对这份文件报未判。")
        continue
    table_start = index
    while index < len(lines) and lines[index].lstrip().startswith("|"):
        index += 1
    table_end = index
    table_text = "\n".join(lines[table_start:table_end])
    numbers_in_table = set(owed_number.findall(table_text))

    # ④ 行号：表头 `#` 与分隔行之外，每一行的首列只许是顺序号或圆圈号。
    suffixed_row_numbers = []
    judged_row_count = 0
    for row_index in range(table_start, table_end):
        first_cell = lines[row_index].split("|")[1].strip()
        if not first_cell or first_cell.startswith("-") or first_cell == "#":
            continue
        judged_row_count += 1
        if first_cell.isdecimal() or (len(first_cell) == 1 and first_cell in circled_row_numbers):
            continue
        suffixed_row_numbers.append(f"第 {row_index + 1} 行：行号写成「{first_cell}」")

    exempted_numbers = {}
    malformed_exemptions = []
    while index < len(lines) and not lines[index].startswith("#"):
        line = lines[index]
        if exemption_start.match(line):
            match = exemption_form.match(line)
            if match:
                exempted_numbers[match.group("number")] = index + 1
            else:
                malformed_exemptions.append(f"第 {index + 1} 行：{line}")
        index += 1

    first_line_by_number = {}
    for line_number, line in enumerate(lines, 1):
        for number in owed_number.findall(line):
            first_line_by_number.setdefault(number, line_number)
    open_numbers = sorted((number for number in first_line_by_number if number in open_owed_names), key=lambda number: int(number[1:]))
    missing_numbers = [number for number in open_numbers if number not in numbers_in_table and number not in exempted_numbers]
    contradicted_numbers = sorted((number for number in exempted_numbers if number in numbers_in_table), key=lambda number: int(number[1:]))

    if malformed_exemptions:
        failed = True
        print(f"  ✗ {path} 收口表后面这些豁免行写坏了（没有编号，或冒号后面没写为什么不收）：")  # gate-lint:summary
        for entry in malformed_exemptions:
            print(f"     {entry}")  # gate-lint:detail
        print("     → 怎么办：一行豁免一个编号，形态「- 不收口 C<编号>（简称）：为什么不收」，冒号后面写它归哪个里程碑、为什么不在这里还。")
    if contradicted_numbers:
        failed = True
        print(f"  ✗ {path} 这些编号豁免了，又写进了收口表——两处说反话：")  # gate-lint:summary
        for number in contradicted_numbers:
            print(f"     {number}（{open_owed_names.get(number, '已还清或欠账表里没有')}）：豁免在第 {exempted_numbers[number]} 行")  # gate-lint:detail
        print("     → 怎么办：这个里程碑要还它，就删掉豁免行；不还，就把它从收口表里拿掉，豁免行写清归哪。")
    if missing_numbers:
        failed = True
        print(f"  ✗ {path} 点名了这些还开着的欠账号，收口表里没有、表后也没有豁免：")  # gate-lint:summary
        for number in missing_numbers:
            print(f"     {number}（{open_owed_names[number]}）：第 {first_line_by_number[number]} 行起出现")  # gate-lint:detail
        print("     → 怎么办：在收口表里给它一行（或并进已有的一行，写清性质与去向），")
        print("               或在表后加一行「- 不收口 C<编号>（简称）：为什么不收」；它其实已经还了，就先把 checks-owed.md 那一行挪进「### 已还清」。")
    if suffixed_row_numbers:
        failed = True
        print(f"  ✗ {path} 收口表的行号带了后缀（撇号或字母），行号只许是顺序号或圆圈号：")  # gate-lint:summary
        for entry in suffixed_row_numbers:
            print(f"     {entry}")  # gate-lint:detail
        print("     → 怎么办：改成一个没被占用的顺序号（接着表里最大的那个往下取），不要在旧号上加撇号或字母——后缀会让读的人以为这一行从属于那个基号。")
        print("               ⚠️ 改号要连全仓引用一起改（「第 N 行」「跟第 N 行」这类），照 .claude/rules/path-moves.md「怎么做」那六步办。")
    per_file_counts.append(f"{name}：全文点名 {len(first_line_by_number)} 个 C 编号，其中还开着 {len(open_numbers)} 个：表里 {len([number for number in open_numbers if number in numbers_in_table])} 个、豁免 {len([number for number in open_numbers if number in exempted_numbers and number not in numbers_in_table])} 个；收口表 {judged_row_count} 行的行号都是顺序号或圆圈号")

if failed:
    sys.exit(1)
print(f"  ✓ 里程碑收口表收全了文件里点名、还开着的欠账号（判了 {len(marked_files)} 份；{'；'.join(per_file_counts)}）")
print(f"    没判 {len(unmarked_files)} 份（没有收口表标记）：{'、'.join(unmarked_files) if unmarked_files else '无'}")
PY
}

# ── 格 paid-cited-tests：已还清的欠账行里点名的测试名，仓里还在不在
# 判据：`.claude/kb/checks-owed.md` 的「### 已还清」那张表里，反引号括起来的 snake_case 标识符（至少三段）
# 必须在仓里的 `.rs` / `.py` / `.sh` 的代码里按整词找得到；找不到就红。按整词（加了后缀的不算还在）、只认注释之外
# （`.rs` 的 `//` 之后、`.py` / `.sh` 的 `#` 之后不算：旧名只剩在一句注释里，那个测试其实已经没了）。
# 已还清表一行都没有、或行里一个这样的标识符都没有，退 77。
# 已还清的行里点名的单测常是那条定案唯一的记录位，它被改名或删掉时没有别的东西会说话。
# 欠着的那张表不查：那里点名一个还不存在的测试名是它应有的样子。
# 至少三段：放宽到两段会把 `log_append`、`write_cache` 这类词组也扫进来。
# 注释里不写被拦的那个标识符的整串字面量，搜索也排掉 `.claude/gate.d/`：否则 grep 扫到这个阶段自己，把该红的那一条判成「还在」。
# 不扫 `research/prompts/`：原样发给模型的材料是冻结证据，里面的旧名字不跟着改（evidence-discipline.md「原样保存的证据不许事后改」）。
# 样本：checks-owed-red 的已还清表里一行引的单测改了名、一行引的旧名只剩在注释里、一行引的名字只剩加了后缀的，三个都要点名；
# 欠着那张里一行引一个不存在的名字，不许点名。checks-owed-green 里已还清表引的单测在、另有不带标识符的行进「没查的」。
cell_paid_cited_tests() {
  local OWED=".claude/kb/checks-owed.md"
  if [[ ! -f "$OWED" ]]; then
    echo "  ! 没有 $OWED，本阶段跳过"
    exit 77
  fi

  python3 - "$OWED" <<'PY'
import pathlib, re, subprocess, sys

owed = pathlib.Path(sys.argv[1])
lines = owed.read_text(encoding="utf-8").splitlines()

settled_heading = None
for index, line in enumerate(lines):
    if line.strip().startswith("### 已还清"):
        settled_heading = index
        break
if settled_heading is None:
    print("  ✗ 找不到「### 已还清」那一节，判不了已还清的行引的测试名还在不在")
    print("     → 怎么办：确认 .claude/kb/checks-owed.md 里那一节的标题没被改写；改了标题就把这一格的取法一起改。")
    raise SystemExit(1)

identifier = re.compile(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+){2,})`")
row_number = re.compile(r"^\|\s*(C\d+)\s*\|")

rows = []
for line in lines[settled_heading + 1:]:
    stripped = line.strip()
    if stripped.startswith("## ") or (stripped.startswith("### ") and not stripped.startswith("### 已还清")):
        break
    # 只认首格是 `C<数字>` 的登记行：表头与分隔行数进去，成功句报的行数会比真实行数多，而那个数没有东西钉着。
    # 一行掉了编号算表的形状坏了，归 table-shape 判。
    if stripped.startswith("|") and stripped.count("|") >= 4 and row_number.match(stripped):
        rows.append(stripped)

cited = []
skipped = []
for row in rows:
    number = row_number.match(row).group(1)
    names = list(dict.fromkeys(identifier.findall(row)))  # 同一行里引两次算一个
    if names:
        for name in names:
            cited.append((number, name))
    else:
        skipped.append(number)

if not rows:
    print("  ⊘ 本次无对象可判：「### 已还清」那张表里一行首格是 C<编号> 的登记行都没有")
    raise SystemExit(77)
if not cited:
    print(f"  ⊘ 本次无对象可判：已还清的 {len(rows)} 行里没有一个至少三段的 snake_case 标识符："
          + "、".join(skipped))
    raise SystemExit(77)

def code_part(line, suffix):
    """去掉注释之后的那一截：.rs 砍掉 `//` 之后，.py / .sh 砍掉行首或空白之后的 `#` 之后。"""
    if suffix == ".rs":
        return line.split("//", 1)[0]
    return re.split(r"(?:^|\s)#", line, maxsplit=1)[0]

def is_in_the_repository(name):
    # grep 只用来缩小候选（按字面、按整词）；注释里的命中在下面逐行剔掉
    found = subprocess.run(
        ["grep", "-rnwF", "--include=*.rs", "--include=*.py", "--include=*.sh",
         "--exclude-dir=prompts", "--exclude-dir=target", "--exclude-dir=.git",
         "--exclude-dir=gate.d", "--", name, "."],
        capture_output=True, text=True)
    if found.returncode not in (0, 1):
        print(f"  ✗ grep 找 `{name}` 时出错（退 {found.returncode}）：{found.stderr.strip()[:120]}")
        print("     → 怎么办：按 grep 的报错修好（读不了的目录、权限），再跑；找的时候出错不等于没找到，也不等于找到了。")
        raise SystemExit(1)
    word = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])")
    for hit in found.stdout.splitlines():
        path, _, rest = hit.partition(":")
        _, _, text = rest.partition(":")
        if word.search(code_part(text, pathlib.Path(path).suffix)):
            return True
    return False

missing = [(number, name) for number, name in cited if not is_in_the_repository(name)]

if missing:
    for number, name in missing:
        print(f"  ✗ {number} 引的 `{name}` 在仓里的 .rs / .py / .sh 里一处都找不到")  # gate-lint:detail
    print(f"  ✗ 已还清的行里有 {len(missing)} 个点名的标识符已经不在仓里了")  # gate-lint:summary
    print("     → 怎么办：那条欠账是靠这个测试还清的。先 grep 它今天叫什么（多半是改了名），")
    print("               用 scripts/kb/replace-once.py 把行里的旧名字定点换成现在的名字；")
    print("               它要是被删掉了，那这条欠账并没有还清，把整行挪回欠着的那张表。")
    raise SystemExit(1)

print(f"  ✓ 已还清的行引的测试名都还在（{len(rows)} 行，查了 {len(cited)} 个标识符："
      + "、".join(f"{number} {name}" for number, name in cited) + "）")
print(f"     没查的 {len(skipped)} 行（行里没有至少三段的 snake_case 标识符）："
      + "、".join(skipped))
PY
}

# ── 格 audit-contradictions：最近一次总审核登记的文档级矛盾，每一行都要有去向
# `records/` 里的总审核记录第五节登记的文档级矛盾在 kb 里没有落点，腐化了没有别的检查会红
# （C358（总审核第五节的文档级矛盾没有回扫闸））。
# 判据：第五节表里每一行的处置列，要么以「已改」开头，要么点名一个 checks-owed.md 开着那张表里的欠账号
# （只点名已还清的号不算去向：那笔账还了，这一行的矛盾却还在）。
#   「已改」要用「」引至少一句原文，在这一行点名的 kb 文件今天的正文里找得到（归一与点名的规则在下面 QUOTE 旁的注释）。
#   引文去掉编号（D / E / C / I- 号连同简称括注）与 .md 文件名之后至少剩两个字：只引一个编号或一个字，在点名的文件里总找得到。
#   只判引文找不找得到，不判引文说的是不是那一行的矛盾（靠人）。
# 第五节标题认「## 五」后面跟标点、空白或行尾的写法；最近一次总审核里认不出第五节标题，判红；表里一行矛盾都没有，退 77。
# 挂账表 .claude/audit-rows-pending 已关闭：再出现任何一行非注释内容判红，新的矛盾行一律当场给去向。
# 内嵌 python 的退出码要取，而且必须报出 COUNT 那一行，缺一样判红：不取退出码时，python 崩了成功句照印、只是数变空白。
# 样本：checks-owed-red 放一行「未改」且不给欠账号、一行只点名已还清的号、一行挂进已关闭的挂账表、一行「已改」却不引原文、
# 两行「已改」引的原文在点名的文件里找不到而只在别的文件里有（一行按 D 编号点名、一行按文件名点名）、一行引的原文哪里都没有、
# 一行只引了一个编号「D9」，第五节标题写成「## 五 文档级矛盾」（不带顿号），判红；checks-owed-green 放按 D 编号与按文件名点名、
# 原文都在点名的文件里各一行，一行没点名文件而原文在 kb 别处，一行带「「旧说法」0 命中」而 kb 里没有「旧说法」，
# 欠账表开着的那张里有一行正文提到「### 已还清」这几个字（按子串切会把它后面开着的账算成还清），判绿。
# D / E 编号点名的是哪一份：lib-item-ref-status.py 的 numbered_files，总审核记录是中文，取中文原文，不把三语三份并起来找。
# 样本：audit-contradictions-translation-green / -red（原句在中文原文里 / 只在日语译文里）；弄坏开关 KB_NUMBERED_FILE_BREAK=first-match。
cell_audit_contradictions() {
  local PENDING=".claude/audit-rows-pending"
  local latest report invariant_files
  latest="$(ls -1 records/*-总审核.md 2>/dev/null | sort | tail -1)"
  if [[ -z "$latest" ]]; then
    echo "  ⊘ 本次未跑：records/ 下没有总审核记录，今天无对象可判"
    exit 77
  fi

  # 不变量表住在哪几份文件：与 declared-counts 那三格同一份 invariant_table_files（登记的 kb 目录里含不变量表行的 .md）
  invariant_files="$(invariant_table_files)"
  if ! report="$(LATEST="$latest" PENDING="$PENDING" OWED_LIBRARY="$OWED_LIBRARY" ITEM_REF_STATUS_LIBRARY="$ITEM_REF_STATUS_LIBRARY" INVARIANT_TABLE_FILES="$invariant_files" BREAK_SWITCH="$BREAK_SWITCH" python3 - 2>&1 <<'PY'
import glob, importlib.util, os, re

latest = os.environ["LATEST"]
pending_path = os.environ["PENDING"]
owed_library_spec = importlib.util.spec_from_file_location("owed", os.environ["OWED_LIBRARY"])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
item_ref_status_spec = importlib.util.spec_from_file_location("item_ref_status", os.environ["ITEM_REF_STATUS_LIBRARY"])
item_ref_status = importlib.util.module_from_spec(item_ref_status_spec)
item_ref_status_spec.loader.exec_module(item_ref_status)

# 挂账表已关闭：任何一行非注释内容都是新挂的账。
if os.path.exists(pending_path):
    for number, line in enumerate(open(pending_path, encoding="utf-8"), 1):
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            print("BAD", f"{pending_path}:{number}",
                  "挂账表 2026-09-23 已清空并关闭，不许再加行；这一行的矛盾要当场给去向", sep="\t")

# 只认开着那张表：「已还清」整行标题之后的号是还清了的，点名它们不算去向。切法用共用读法 lib-owed.py：
# 按子串「### 已还清」切，开着的那张表里有一行正文提到这几个字，它后面开着的账就全被算成还清。
owed_table = owed_library.read_owed_table(".claude/kb/checks-owed.md")
if not owed_table.file_found:
    print("BAD", ".claude/kb/checks-owed.md", "欠账表不在，分不出哪些欠账号还开着", sep="\t")
elif not owed_table.paid_heading_found:
    print("BAD", ".claude/kb/checks-owed.md", "认不出「已还清」那一行整行标题，分不出哪些欠账号还开着（不对着一张认不出的表判）", sep="\t")
open_owed = set(owed_table.open_names) if owed_table.paid_heading_found else set()
repaid_owed = set(owed_table.paid_names)

lines = open(latest, encoding="utf-8").read().split("\n")
in_section = False
section_heading = None
rows = []
for number, line in enumerate(lines, 1):
    if section_heading is None and re.match(r"^##\s*五(?:[、.．:：\s]|$)", line):
        in_section = True
        section_heading = (number, line.strip())
        continue
    if in_section and re.match(r"^##\s", line):
        break
    if in_section and line.startswith("| "):
        rows.append((number, line))

rows = [(number, line) for number, line in rows
        if not line.startswith("|---") and not re.match(r"^\|\s*#\s*\|", line)]
if section_heading is None:
    print("BAD", latest, "认不出第五节的标题（要写成「## 五」后面跟标点、空白或行尾），一行矛盾都没判", sep="\t")

# 「已改」要指得到今天的正文：处置列用「」引的原文，按空白与 ** ` 归一之后，要在今天的 kb 正文
# （「## 历史版本」之前，不含变更史）里找得到，而且要在这一行点名的文件里找得到（D / E 编号、C 号 → checks-owed.md、
# I- 号 → 含这一类不变量表行的那几份文件，类号找不到文件的指它们所在目录的 README.md、直接写出的 .md 文件名）；
# 一个 kb 文件都没点名的行才在 kb 全部正文里找。不变量表的文件由格经 INVARIANT_TABLE_FILES 传进来（invariant_table_files，
# 按 .claude/kb-directories 登记的目录找），连同它们所在目录的 README.md 一起算进 kb 正文。
# 不退到全部正文里找：旧条款的原句常被实验页原样引着，改掉之后照样搜得到，会把已经不在的那句当成还在。
# 引号后面紧跟「0 命中」「不在」「没有了」的，说的是那句话已经删掉，不当成指向正文的引文。
QUOTE = re.compile(r"「([^」]+)」")
NEGATED_AFTER = re.compile(r"^\s*(?:字样|都|也|已经)?\s*(?:0\s*命中|不在|没有了)")

def normalized(text):
    return re.sub(r"\s+", " ", re.sub(r"[*`]", "", text)).strip()

REFERENCE = re.compile(r"(?<![A-Za-z0-9_])(?:[DEC]\d+|I-\d+(?:\.\d+)*)(?:（[^）]*）)?|[\w\-]+\.md")

def anchor_text(quote):
    """引文里去掉编号（连同简称括注）与 .md 文件名、标点与空白之后剩下的字：少于两个就指不出正文里的一句话。"""
    return re.sub(r"[\s\W_]+", "", REFERENCE.sub("", normalized(quote)))

kb_body_by_path = {}
for path in sorted(glob.glob(".claude/kb/**/*.md", recursive=True)):
    if path.endswith("-history.md") or "/decisions-history/" in path:
        continue
    kb_body_by_path[path] = normalized(open(path, encoding="utf-8").read().split("\n## 历史版本")[0])

# 弄坏开关 invariant-single-file：走回改前的读法，不变量文件不进 kb 正文、I- 号一律指 .claude/kb/invariants.md
single_file_reading = os.environ.get("BREAK_SWITCH") == "invariant-single-file"
invariant_files_by_class, invariant_indexes = {}, []
for path in [] if single_file_reading else [line for line in os.environ.get("INVARIANT_TABLE_FILES", "").split("\n") if line]:
    text = open(path, encoding="utf-8").read()
    kb_body_by_path.setdefault(path, normalized(text.split("\n## 历史版本")[0]))
    for number in set(re.findall(r"^\| I-(\d+)\.\d+ ", text, re.M)):
        invariant_files_by_class.setdefault(int(number), set()).add(path)
    index = os.path.join(os.path.dirname(path), "README.md")
    if os.path.isfile(index) and index not in invariant_indexes:
        invariant_indexes.append(index)
for index in invariant_indexes:
    kb_body_by_path.setdefault(index, normalized(open(index, encoding="utf-8").read().split("\n## 历史版本")[0]))

def shown(path):
    """kb 目录下的写成相对 .claude/kb 的路径，别处的（按类拆开的不变量文件）写相对仓根的路径。"""
    return os.path.relpath(path, ".claude/kb") if path.startswith(".claude/kb/") else path

def kb_paths_named_by(text):
    # 编号指的是总审核记录（中文）那一侧的中文原文（numbered_files）；改前把三语三份并进来，中文正文里没有的原句在日语译文里搜得到也算找到
    named = set()
    for number in re.findall(r"(?<![A-Za-z0-9_])D(\d+)(?!\d)", text):
        named |= set(item_ref_status.numbered_files(".claude/kb/decisions", number, latest))
    for number in re.findall(r"(?<![A-Za-z0-9_])E(\d+)(?!\d)", text):
        named |= set(item_ref_status.numbered_files(".claude/kb/experiments", number, latest))
    if re.search(r"(?<![A-Za-z0-9_])C\d+(?!\d)", text):
        named.add(".claude/kb/checks-owed.md")
    for number in re.findall(r"(?<![A-Za-z0-9_])I-(\d+)", text):
        if single_file_reading:
            named.add(".claude/kb/invariants.md")
        else:
            named |= invariant_files_by_class.get(int(number)) or set(invariant_indexes)
    for file_name in re.findall(r"([\w\-]+\.md)\b", text):
        named |= {path for path in kb_body_by_path if os.path.basename(path) == file_name}
    return {path for path in named if path in kb_body_by_path}

checked = 0
fixed_rows, quote_count, unnamed_rows = 0, 0, 0
for number, line in rows:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    if len(cells) < 4:
        print("BAD", f"{latest}:{number}", "这一行不足四列，读不出处置列", sep="\t")
        continue
    checked += 1
    disposition = cells[3]
    if disposition.startswith("已改"):
        fixed_rows += 1
        affirmed_quotes = [quote.group(1) for quote in QUOTE.finditer(disposition)
                           if not NEGATED_AFTER.match(disposition[quote.end():])]
        if not affirmed_quotes:
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，却没有用「」引一句今天正文里的原文（只写「0 命中」「不在了」的那种不算）",
                  sep="\t")
            continue
        named_paths = kb_paths_named_by(" ".join(cells[1:4]))
        if not named_paths:
            unnamed_rows += 1
        searched_paths = named_paths or set(kb_body_by_path)
        for quote in affirmed_quotes:
            quote_count += 1
            wanted = normalized(quote)
            if len(anchor_text(quote)) < 2:
                print("BAD", f"{latest}:{number}",
                      f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」只是编号或文件名（去掉之后不足两个字），"
                      "在点名的那份文件里总找得到，指不出今天正文里的一句话", sep="\t")
                continue
            if any(wanted in kb_body_by_path[path] for path in searched_paths):
                continue
            elsewhere = sorted(shown(path) for path, text in kb_body_by_path.items()
                               if path not in searched_paths and wanted in text)
            where_else = f"；只在 {'、'.join(elsewhere[:3])} 里有，那一行没点名它" if elsewhere else ""
            named_shown = "、".join(shown(path) for path in sorted(named_paths)) or "kb 全部正文"
            print("BAD", f"{latest}:{number}",
                  f"第 {cells[0]} 行处置列写「已改」，引的「{quote[:40]}」在 {named_shown} 的今天正文里找不到{where_else}",
                  sep="\t")
        continue
    cited = set(re.findall(r"\bC\d+\b", disposition))
    if cited & open_owed:
        continue
    if cited & repaid_owed:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列只点名了已还清的 {'、'.join(sorted(cited & repaid_owed))}——账还了，这一行的矛盾还在",
              sep="\t")
        continue
    if cited:
        print("BAD", f"{latest}:{number}",
              f"第 {cells[0]} 行处置列点名 {'、'.join(sorted(cited))}，而 checks-owed.md 里没有这个号",
              sep="\t")
        continue
    print("BAD", f"{latest}:{number}",
          f"第 {cells[0]} 行处置列写「{disposition[:40]}」，既不是「已改」、也没点名一个还开着的欠账号",
          sep="\t")

print("COUNT", latest, checked, len(rows), fixed_rows, quote_count, unnamed_rows,
      len(open_owed), len(repaid_owed), section_heading[0] if section_heading else 0,
      section_heading[1] if section_heading else "认不出", sep="\t")
PY
)"; then
    echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一行都没判"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/doc-registries.sh 里 audit-contradictions 那一格的 python；"
    echo "      这一步没跑完就是什么都没查，不是通过。"
    exit 1
  fi

  if ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check audit-contradictions）看 python 的报错；没有 COUNT 就是一行都没判，"
    echo "      而汇总行看着与判过了一模一样。"
    exit 1
  fi

  local -a bad
  local where checked total fixed quotes unnamed open_count repaid_count heading_line heading_text why
  mapfile -t bad < <(grep -E '^BAD' <<<"$report")
  IFS=$'\t' read -r _ where checked total fixed quotes unnamed open_count repaid_count heading_line heading_text < <(grep '^COUNT' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 最近一次总审核第五节有行没有去向（第五节标题：${heading_text}）："
    while IFS=$'\t' read -r _ where why; do
      printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：给这一行一个去向——改了就把处置列写成「已改…」，并用「」引一句今天正文里的原文、"
    echo "      点名它住在哪份文件（D / E 编号、C 号、I- 号或文件名）；引的那句今天找不到，就是那处又被改过，"
    echo "      回去现查这一行的矛盾还在不在：不在了就换成今天的原文，又回来了就当它没改。"
    echo "      还没改就在 .claude/kb/checks-owed.md 开着的那张表里立一笔账、把那个编号写进处置列。"
    echo "      只点名已还清的号不算；挂账表 .claude/audit-rows-pending 已关闭，不许往里加行。"
    exit 1
  fi

  if [[ "$total" == 0 ]]; then
    echo "  ⊘ 本次无对象可判：$where 第五节（第 ${heading_line} 行的标题）下面的表里一行矛盾都没有"
    exit 77
  fi
  echo "  ✓ 最近一次总审核第五节每一行都有去向（$where：第五节 ${total} 行，判了 ${checked} 行；写「已改」的 ${fixed} 行引了 ${quotes} 句原文，都在那一行点名的文件的今天正文里找得到；其中 ${unnamed} 行没点名 kb 文件，按 kb 全部正文找）"
  echo "    欠账表按共用读法 lib-owed.py 切：开着 ${open_count} 笔、已还清 ${repaid_count} 笔；第五节标题在第 ${heading_line} 行"
  exit 0
}

# ── 格 closeout-row27-preconditions：收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
# .claude/kb/milestone/02-second-txn.md 收口表第 27 行点名了一批「今天不可达的欠账」：走不到那一格，就写不出会红的用例。
# 这一格判的是它们各自的前置动没动：每一笔登记一条逐字探针（文件、原文、今天的命中次数）。
#   全部对上：前置一个都没进来，退 77。任一条对不上：那一笔的前置进来了，或者它压着的那段代码改了形态，红，回去重核那一笔。
#   重构那几处也会红：要做的同样是重核，不是把探针的期望值改成新数（那等于把这一笔的描述悄悄换掉）。
# UNPROBED 表是手写的：第 27 行只写「alloc-basis 第三轮转来的四条」、没有逐条列，现算不出来。
# 能现算的那一半每轮现算、只报不判：第 27 行第二格按「、」「；」切成几笔，探针与 UNPROBED 两边都没有的逐个列名；
#   「alloc-basis 第三轮转来的 N 条」拿 N 与这里名字以「alloc-basis 第三轮」起头的笔数比。对不上是这张表手写得不全，要人回去补。
# 样本：checks-owed-red 的 instance_table.rs 里多了一处 `rows.retain(`（C333 那一笔的前置进来了），判红并点名那一笔；
# checks-owed-green 的探针逐条对上今天的值，这一格退 77，并报第 27 行现算的笔数与 alloc-basis 那一笔的条数对照。
cell_closeout_row27_preconditions() {
  local PROBES UNPROBED report row27_report probe_count item_count unprobed_count name why line
  local -a bad
  # 四段：笔名、文件、原文（逐字）、今天的命中次数。同一笔可以有几条探针。
  PROBES="$(cat <<'TSV'
豁免集与隔离集按记录起点做 key	crates/singlefs-core/src/mount.rs	(slot.0..end_slot).map(move |covered_slot| (device.0, covered_slot))	1
扣住位在空发布循环之后无条件放开	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();	2
alloc-basis 第三轮 ①：挂载内回收	crates/singlefs-core/src/mount.rs	reclaim_released_up_to(	3
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.retain(	0
C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.remove(	0
TSV
)"

  # 做不成逐字探针的那几笔，逐个列名——它们同样开着，只是这一格够不着。
  UNPROBED="$(cat <<'TSV'
alloc-basis 第三轮 ②：根槽写失败推进一格重发	这条路径今天不存在，没有一处逐字文本代表「它进来了」
alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像	扣住位只住内存、盘上没有落点，没有可扫的对象
TSV
)"

  if ! report="$(PROBES="$PROBES" python3 - 2>&1 <<'PY'
import os

probes = []
for line in os.environ["PROBES"].split("\n"):
    if not line.strip():
        continue
    name, path, original, expected = line.split("\t")
    probes.append((name, path, original, int(expected)))

for name, path, original, expected in probes:
    try:
        source = open(path, encoding="utf-8").read()
    except FileNotFoundError:
        print("BAD", name, f"{path} 不在了（探针指着的文件没了）", sep="\t")
        continue
    hits = source.count(original)
    if hits != expected:
        print("BAD", name, f"{path} 里「{original.strip()}」命中 {hits} 次，登记的今天的值是 {expected} 次", sep="\t")
print("CHECKED", len(probes), len({name for name, _, _, _ in probes}), sep="\t")
PY
)"; then
    echo "  ✗ 探针没跑完：内嵌 python 自己出错了，一条都没核"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/doc-registries.sh 里 closeout-row27-preconditions 那一格的 python；"
    echo "      没跑完不是「本次未跑」，是这一格自己坏了。"
    exit 1
  fi

  if ! grep -q '^CHECKED' <<<"$report"; then
    echo "  ✗ 探针没跑完：内嵌 python 没报出 CHECKED 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check closeout-row27-preconditions）看 python 的报错；没有 CHECKED 就是一条都没核，"
    echo "      而「本次未跑」那句看着与核过了一模一样。"
    exit 1
  fi

  mapfile -t bad < <(grep '^BAD' <<<"$report")
  read -r _ probe_count item_count < <(grep '^CHECKED' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 收口表第 27 行那几笔欠账的前置动了（或者它压着的那段代码改了形态）："
    while IFS=$'\t' read -r _ name why; do
      printf '      %s：%s\n' "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：先回 .claude/kb/milestone/02-second-txn.md 收口表第 27 行重核这一笔今天可不可达。"
    echo "      可达了就当场做成一条会红的用例（改坏一行、看着它红），把这一笔从第 27 行挪出去，探针那一行一起删；"
    echo "      只是重构、这一笔还不可达，就把探针的原文改成今天逐字存在的那一段，命中次数照实写，并在报告里写明重核过。"
    echo "      ⚠️ 别只把期望值改成新数：那等于把这一笔的描述换掉，而外面看不出换过。"
    exit 1
  fi

  unprobed_count="$(grep -c . <<<"$UNPROBED")"
  row27_report="$(PROBES="$PROBES" UNPROBED="$UNPROBED" python3 - 2>&1 <<'PY'
import os, re
path = ".claude/kb/milestone/02-second-txn.md"
names = [line.split("\t")[0] for key in ("PROBES", "UNPROBED") for line in os.environ[key].split("\n") if line.strip()]
if not os.path.isfile(path):
    print(f"读不到 {path}，第 27 行现算对照没做")
    raise SystemExit(0)
row = next((line for line in open(path, encoding="utf-8") if re.match(r"^\|\s*27\s*\|", line)), None)
if row is None:
    print(f"{path} 里找不到收口表第 27 行，现算对照没做")
    raise SystemExit(0)
cell = row.strip().strip("|").split("|")[1].strip()
cell = re.sub(r"^[^：]*：", "", cell, count=1)
items, depth, current = [], 0, ""
for character in cell:   # 只在括号外按「、」「；」切：括注里的「步 3、步 4」不是两笔
    depth += character in "（("
    depth -= character in "）)" and depth > 0
    if character in "、；" and depth == 0:
        items.append(current.strip()); current = ""
    else:
        current += character
items = [item for item in items + [current.strip()] if item]
declared_alloc_basis = None
uncovered = []
for item in items:
    counted = re.match(r"^alloc-basis 第三轮转来的([一二三四五六七八九十0-9]+)条$", item)
    if counted:
        digits = "一二三四五六七八九十"
        text = counted.group(1)
        declared_alloc_basis = int(text) if text.isdigit() else digits.index(text) + 1 if len(text) == 1 else None
        continue
    if not any(item.startswith(name.split("：")[0]) or name.startswith(item) for name in names):
        uncovered.append(item)
listed_alloc_basis = len({name.split("：")[0] for name in names if name.startswith("alloc-basis 第三轮")})
print(f"收口表第 27 行现算 {len(items)} 笔；探针与没做成探针的清单两边都没有的 {len(uncovered)} 笔：{'、'.join(uncovered) or '（没有）'}")
if declared_alloc_basis is not None:
    print(f"「alloc-basis 第三轮转来的」那一笔第 27 行写 {declared_alloc_basis} 条，这里探针与清单合计 {listed_alloc_basis} 条"
          + ("" if declared_alloc_basis == listed_alloc_basis else "，对不上：清单是手写的，回去按那一轮判决补齐"))
PY
)"
  echo "  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（${probe_count} 条逐字探针、覆盖 ${item_count} 笔，逐条对上今天的值）"
  echo "    没做成探针的 ${unprobed_count} 笔："
  while IFS=$'\t' read -r name why; do
    [[ -z "$name" ]] && continue
    printf '      %s：%s\n' "$name" "$why"
  done <<<"$UNPROBED"
  while IFS= read -r line; do
    [[ -n "$line" ]] && printf '    %s\n' "$line"
  done <<<"$row27_report"
  exit 77
}

# ════════════════ 引用与不变量组的七格（引用与不变量） ════════════════

# ── experiment-refs、decision-refs 共用：扫的文件与编号引用的抽法 ─────────────────
# 必须递归扫：决策与实验拆在子目录里，引用大半住在那里，$KB/*.md 单层通配会静默漏掉。
# 扫的文件：kb 下全部 .md（递归）加项目规则。数组装、不靠 $(find …) 的分词，文件名里有空格也不散。
scanned_files_collected=0
collect_scanned_files() {
  ((scanned_files_collected)) && return 0
  scanned_files_collected=1
  kb_md_files=()
  while IFS= read -r -d '' found_file; do kb_md_files+=("$found_file"); done \
    < <(find "$KB" -name '*.md' -print0 2>/dev/null)
  shopt -s nullglob
  rule_md_files=(.claude/rules/*.md)
  shopt -u nullglob
  scanned_md_files=("${kb_md_files[@]}" "${rule_md_files[@]}")
  scanned_note="扫 ${#kb_md_files[@]} 份 kb 文件、${#rule_md_files[@]} 份规则"
}
# 引用抽取用 grep 一次读全部文件；某一份读不了（悬空的符号链接、没有读权限）时 grep 退 2，
# 那一份里的引用就从判据里静默消失，其余照判照绿。所以 grep 的错误不丢进 /dev/null：退 2 判红并列出它的原话。
grep_error_file="$work_directory/grep-errors"
# 用法：extract_numbered_references <字母> <数组名>；抽出全部不同的「字母+1–3 位数字」引用，读文件出错判红
extract_numbered_references() {
  local letter="$1" raw_matches grep_exit_code=0
  local -n extracted_references="$2"
  extracted_references=()
  [[ ${#scanned_md_files[@]} -gt 0 ]] || return 0
  raw_matches="$(grep -ohE "(^|[^A-Za-z0-9/-])${letter}[0-9]{1,3}([^A-Za-z0-9-]|\$)" "${scanned_md_files[@]}" 2>"$grep_error_file")" \
    || grep_exit_code=$?
  if [[ "$grep_exit_code" -ge 2 ]]; then
    bad "抽 ${letter} 号引用时有文件读不了（grep 退 $grep_exit_code），那几份里的引用没进判据："
    sed 's/^/      /' "$grep_error_file"
    howto "按上面 grep 的原话修：悬空的符号链接删掉或指回真文件，没有读权限的补上读权限；" \
          "读不了的文件里的引用一条都没判，不能当成都有定义。"
  fi
  mapfile -t extracted_references < <(printf '%s\n' "$raw_matches" | grep -oE "${letter}[0-9]{1,3}" | sort -u)
}

cell_experiment_refs() {
  cell_failed=0
  collect_scanned_files
  local missing=0 e
  local experiment_refs=()
  # 不能用 \bE[0-9]+\b：它会把 URL 里的 E19253-01 当成实验号
  extract_numbered_references E experiment_refs
  for e in "${experiment_refs[@]}"; do
    grep -rqE "^## $e " "$KB/experiments" 2>/dev/null || { bad "$e 被引用但 experiments/ 下没有它"
      howto "要么在 experiments/ 下给它建正文（\`## $e <简称>\` 起头），" \
            "要么把引用它的那处改成真实存在的实验号——编号引用悬空，检索到的人会自己补一个。"
      missing=1; }
  done
  if [[ ${#experiment_refs[@]} -eq 0 ]]; then
    bad "一个实验号引用都没扫到（$scanned_note）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的（第一个参数是仓库根）、$KB 目录在；" \
          "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
  elif [[ $missing -eq 0 ]]; then
    ok "实验号引用全部有定义：${#experiment_refs[@]} 个不同的实验号（$scanned_note）"
  fi
  return "$cell_failed"
}

# ── decision-refs 的豁免标记：<!-- doc-registries:not-decisions D<n> … -->（与 doc-lint 的 <!-- doc-lint:not-numbers … --> 同形）
# 用处：命令原样输出、产物行里形状像决策号、其实不是的串（例：E165 页一段输出里的域名 D0），改不得原文，就在那一段上方单起一行写标记。
# 判法：标记要独占一行（前后只许空白）；它管的是标记之后的那几行——到下一个空行为止；标记下面紧接着是代码块的，管到那个代码块的收尾围栏
#   （代码块里的空行不算断开）。标记点名的编号，在它管的那几行里（连同标记那一行自己）出现的处数不算引用；一个编号全仓每一处都落在
#   某条标记管的行里，才从「被引用的决策号」里摘掉，管的行外面还有一处就照判。
# 判红：标记一个编号都没点名、点名的不是 D 加 1–3 位数字、点名的编号在 decisions/ 下有定义（等于一行注释把检查对它关掉，与 doc-lint 的 not-numbers 同一条）、
#   点名的编号在它管的那几行里一处都没有（不起作用的标记）。
# 射程：扫的文件与 experiment-refs、decision-refs 同一批（collect_scanned_files）；处数的取法与 extract_numbered_references 的 grep 同一条式子、逐行不重叠地取。
# 样本：decision-refs-not-decisions-red 一份 kb 页里标记管的代码块有 D0、块后空行之外的正文又写了一处 D0（照判），另有点名已定义的 D1、
#   点名的 D7 在管的行里没有、一个编号都没点名的三条标记；decision-refs-not-decisions-green 同一段 D0 只在标记管的代码块里，判绿并报豁免了几处。
apply_not_decisions_markers() {
  local -n kept_references="$1"
  local marker_report kind field rest
  not_decisions_note="not-decisions 标记 0 条"
  [[ "$BREAK_SWITCH" == "not-decisions-ignored" ]] && return 0
  [[ ${#scanned_md_files[@]} -gt 0 ]] || return 0
  printf '%s\0' "${scanned_md_files[@]}" > "$work_directory/decision-refs-scanned-files"
  if ! marker_report="$(python3 - "$work_directory/decision-refs-scanned-files" "$KB/decisions" "$BREAK_SWITCH" 2>&1 <<'PY'
import os, re, sys

files_list, decisions_directory, broken = sys.argv[1], sys.argv[2], sys.argv[3]
# 与 extract_numbered_references 的 grep -E 同一条式子、同样逐行、不重叠地取，数出来的处数与 grep 对得上
REFERENCE = re.compile(r"(?:^|[^A-Za-z0-9/-])(D[0-9]{1,3})(?:[^A-Za-z0-9-]|$)")
MARKER = re.compile(r"^\s*<!--\s*doc-registries:not-decisions(?:\s(.*?))?\s*-->\s*$")
FENCE = re.compile(r"^\s*(?:```|~~~)")
NUMBER = re.compile(r"D[0-9]{1,3}")

defined = set()
for directory, _subdirectories, names in os.walk(decisions_directory):
    for name in names:
        if name.endswith(".md"):
            for line in open(os.path.join(directory, name), encoding="utf-8", errors="replace"):
                heading = re.match(r"## (D[0-9]{1,3}) ", line)
                if heading:
                    defined.add(heading.group(1))

total, exempted = {}, {}
exempted_outside_marker_lines = 0
marker_count = 0
problems = []
paths = [path for path in open(files_list, "rb").read().decode("utf-8", "replace").split("\0") if path]
for path in paths:
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        continue  # 读不了的那一份 grep 那一步已经判红
    occurrences = [[match.group(1) for match in REFERENCE.finditer(line)] for line in lines]
    for found in occurrences:
        for token in found:
            total[token] = total.get(token, 0) + 1
    for index, line in enumerate(lines):
        marker = MARKER.match(line)
        if not marker:
            continue
        marker_count += 1
        place = f"{path}:{index + 1}"
        tokens = (marker.group(1) or "").split()
        if not tokens:
            problems.append(f"{place} 的 not-decisions 标记一个编号都没点名")
            continue
        if broken == "not-decisions-whole-file":
            scope = list(range(len(lines)))
        else:
            scope, position, in_code = [index], index + 1, False
            while position < len(lines):
                text = lines[position]
                if not in_code and not text.strip():
                    break
                scope.append(position)
                if FENCE.match(text):
                    if in_code:
                        break
                    in_code = True
                position += 1
        body = [position for position in scope if position != index]
        if not body:
            span = "（标记下面紧挨着的是空行，它一行都不管）"
        elif len(body) == 1:
            span = f"第 {body[0] + 1} 行"
        else:
            span = f"第 {body[0] + 1}–{body[-1] + 1} 行"
        for token in tokens:
            if not NUMBER.fullmatch(token):
                problems.append(f"{place} 的 not-decisions 标记里「{token}」不是决策号的写法（D 加 1–3 位数字）")
                continue
            if token in defined:
                problems.append(f"{token} 在 decisions/ 下有定义，{place} 却用 not-decisions 标记把它摘出去")
                continue
            inside = sum(occurrences[position].count(token) for position in body)
            if inside == 0:
                problems.append(f"{place} 的 not-decisions 标记点名的 {token} 在它管的{span}里一处都没有")
                continue
            exempted[token] = exempted.get(token, 0) + inside + occurrences[index].count(token)
            exempted_outside_marker_lines += inside

for problem in problems:
    print("PROBLEM", problem, sep="\t")
for token in sorted(exempted):
    if exempted[token] >= total.get(token, 0):
        print("EXEMPT", token, sep="\t")
print("COUNT", exempted_outside_marker_lines, marker_count, sep="\t")
PY
)"; then
    bad "not-decisions 标记没判完：内嵌 python 自己出错了，标记一条都没认"
    printf '%s\n' "$marker_report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    howto "按上面的报错修 .claude/gate.d/doc-registries.sh 里 apply_not_decisions_markers 的 python；没判完不是通过，标记管的编号这一轮照引用判。"
    return 0
  fi
  local -A exempt_numbers=()
  local -a marker_problems=()
  while IFS=$'\t' read -r kind field rest; do
    case "$kind" in
      PROBLEM) marker_problems+=("$field") ;;
      EXEMPT) exempt_numbers[$field]=1 ;;
      COUNT) not_decisions_note="not-decisions 标记 ${rest} 条，豁免 ${field} 处" ;;
    esac
  done <<<"$marker_report"
  if ((${#marker_problems[@]})); then
    for field in "${marker_problems[@]}"; do bad "$field"; done   # gate-lint:detail
    howto "标记写坏的照上面那一句改：点名 D 加 1–3 位数字、只点名它管的那几行里真有的串；有定义的决策号不许摘出去，引用处写成「D<n>（简称）」；" \
          "不起作用的标记删掉，或挪到那段输出正上方（标记与它要管的那几行之间不留空行）。写法与判法在 decision-refs 格上方的注释。"
  fi
  local reference kept=()
  for reference in "${kept_references[@]}"; do
    [[ -n "${exempt_numbers[$reference]+set}" ]] || kept+=("$reference")
  done
  kept_references=(${kept[@]+"${kept[@]}"})
}

cell_decision_refs() {
  cell_failed=0
  collect_scanned_files
  local missing=0 d
  local decision_refs=()
  extract_numbered_references D decision_refs
  apply_not_decisions_markers decision_refs
  for d in "${decision_refs[@]}"; do
    grep -rqE "^## $d " "$KB/decisions" 2>/dev/null || { bad "$d 被引用但 decisions/ 下没有它"
      howto "要么在 decisions/ 下给它建正文（\`## $d <简称>\` 起头），" \
            "要么把引用它的那处改成真实存在的决策号。"
      missing=1; }
  done
  if [[ ${#decision_refs[@]} -eq 0 ]]; then
    bad "一个决策号引用都没扫到（$scanned_note）——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的、$KB 目录在；" \
          "扫到 0 项不是通过（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。"
  elif [[ $missing -eq 0 ]]; then
    ok "决策号引用全部有定义：${#decision_refs[@]} 个不同的决策号（$scanned_note）；$not_decisions_note"
  fi
  return "$cell_failed"
}

# ── declared-counts、invariant-count-elsewhere、invariant-anchors 共用：不变量表住在哪些文件里 ─────────────
# 表行形如 `| I-1.1 | 简称 | 陈述 | … |`；表里第 2 列是简称（singlefs-ai-sop/rules/kb-discipline.md 第 5 条），陈述在第 3 列。
# 不变量表搬出 .claude/kb/invariants.md 单文件（按类拆成 crates/<包>/invariants/ 这样的多文件）时，
# 登记项目根的 .claude/kb-directories（kb-discipline.md）；这三格不认死 invariants.md 这个文件名，
# 按「候选目录里，哪些 .md 文件有不变量表行」找文件。登记是**替换**不是追加（与 doc-lint.sh、history-ordinal.sh
# 读 kb_directories 的口径一致）：登记了就只认登记的那几个目录，仍要 .claude/kb 的话自己把它也列进登记表；没登记才默认 .claude/kb。
kb_candidate_roots() {  # → 每行一个候选目录，相对项目根，去重（CWD 已经是项目根，见 ROOT 的 cd）
  local reg=.claude/kb-directories raw path registered=()
  if [[ -f "$reg" ]]; then
    while IFS= read -r raw || [[ -n "$raw" ]]; do
      [[ -z "${raw// }" || "$raw" == \#* ]] && continue
      path="${raw%%#*}"; path="${path%"${path##*[![:space:]]}"}"; path="${path#"${path%%[![:space:]]*}"}"
      [[ -n "$path" && -d "$path" ]] && registered+=("${path%/}")
    done < "$reg"
  fi
  if ((${#registered[@]})); then
    printf '%s\n' "${registered[@]}" | sort -u
  else
    printf '%s\n' "$KB"
  fi
}

kb_candidate_md_files() {  # → 每行一个路径：候选目录下全部 .md 文件，按路径排序去重
  local root files=()
  while IFS= read -r root; do
    [[ -d "$root" ]] || continue
    while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$root" -name '*.md' -type f -print0 2>/dev/null)
  done < <(kb_candidate_roots)
  ((${#files[@]})) || return 0
  printf '%s\n' "${files[@]}" | sort -u
}

invariant_table_files() {  # → 每行一个路径：候选目录下含不变量表行的 .md 文件，按路径排序去重
  local files=()
  while IFS= read -r f; do files+=("$f"); done < <(kb_candidate_md_files)
  ((${#files[@]})) || return 0
  grep -lE '^\| I-[0-9]+\.[0-9]+ ' "${files[@]}" 2>/dev/null | sort -u
}

# 设 inv_actual（总行）、inv_retired（退役）、inv_live（在用）、inv_files（数组：含不变量表行的文件，三语都在）、
# inv_counted_files（数组：数条数的那几份，只有中文原文）、inv_translation_files / inv_translation_rows（没数的译文份数与表行数，成功行报出来）。
# 条数只数中文原文：译文（<名>.en.md、<名>.ja.md，常见的是实验页、决策页的译文里抄着几行 `| I-…`）是同一份内容的另一种语言，
# 数进去就把一条算成两三条。取法走 lib-item-ref-status.py 的 same_language_files（与按编号取文件同一份判法，不在这里另认后缀）；
# 弄坏开关 KB_NUMBERED_FILE_BREAK=first-match 下不分语言、译文的表行一并数（改前的数法）。inv_files 照旧三语都在：
# invariant-anchors 判这次改动写的每一行（译文里的也判），invariant-count-elsewhere 不扫带表行的文件，都用它。
# 判法库读不到、python 没跑成：退 1，两格判红，不退回不分语言去数。
count_invariant_rows() {
  inv_actual=0; inv_retired=0; inv_files=(); inv_counted_files=(); inv_translation_files=0; inv_translation_rows=0
  local f counted_listing rows
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    inv_files+=("$f")
  done < <(invariant_table_files)
  ((${#inv_files[@]})) || { inv_live=0; return 0; }
  if ! counted_listing="$(python3 - "$ITEM_REF_STATUS_LIBRARY" "${inv_files[@]}" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
for path in lib.same_language_files(sys.argv[2:]):
    print(path)
PY
  )"; then
    inv_live=0
    return 1
  fi
  while IFS= read -r f; do
    [[ -n "$f" ]] && inv_counted_files+=("$f")
  done <<<"$counted_listing"
  for f in "${inv_files[@]}"; do
    rows=$(grep -cE '^\| I-[0-9]+\.[0-9]+ ' "$f")
    if grep -qxF -- "$f" <<<"$counted_listing"; then
      inv_actual=$(( inv_actual + rows ))
      inv_retired=$(( inv_retired + $(grep -cE '^\| I-[0-9]+\.[0-9]+ \| [^|]* \| \*\*此编号不再使用' "$f") ))
    else
      inv_translation_files=$(( inv_translation_files + 1 )); inv_translation_rows=$(( inv_translation_rows + rows ))
    fi
  done
  inv_live=$(( inv_actual - inv_retired ))
}

# count_invariant_rows 没跑成时两格同一句红行
report_invariant_count_failure() {
  bad "按语言挑数条数那几份文件的 python 没跑成（判法 $ITEM_REF_STATUS_LIBRARY 的 same_language_files）——不变量条数没数，这一格没判"
  howto "按上面的 traceback 修（判法库在不在、能不能 import）；没数成就是没判，不许退回不分语言去数（译文的表行会被多数一遍）。"
}

# 没数的译文：成功行与红行在括号里报出来（一份都没有时不报，消息与改前逐字相同）
inv_translation_note() {
  ((inv_translation_files)) && printf '；只数中文原文，译文 %s 份里的 %s 行表行不数' "$inv_translation_files" "$inv_translation_rows"
  return 0
}

# 候选文件只有一份且叫 invariants.md 时，消息里仍称呼它的文件名（历史惯例）；拆成多份时改叫「不变量表」。份数只数中文原文（数条数的那几份）。
inv_table_label() {
  if [[ ${#inv_counted_files[@]} -eq 1 && "$(basename "${inv_counted_files[0]}")" == invariants.md ]]; then
    printf '%s' "invariants.md"
  else
    printf '%s' "不变量表（${#inv_counted_files[@]} 份文件，登记在 .claude/kb-directories）"
  fi
}

cell_declared_counts() {
  cell_failed=0
  if ! count_invariant_rows; then
    report_invariant_count_failure
  elif [[ ${#inv_files[@]} -gt 0 && ${#inv_counted_files[@]} -eq 0 ]]; then
    bad "候选 kb 目录里带不变量表行的 ${#inv_files[@]} 份文件全是译文，没有一份中文原文：在用条数没有可数的（条数只数中文原文）"
    howto "不变量表的中文原文 <名>.md 搬了家就按 .claude/rules/path-moves.md 改登记（.claude/kb-directories）；译文先有、中文原文缺的，先补中文原文。"
  elif [[ ${#inv_files[@]} -eq 0 ]]; then
    bad "候选 kb 目录里一个带不变量表行的文件都没扫到（表行形如 \`| I-1.1 | 简称 | 陈述 | … |\`）"
    howto "确认门禁是在仓库根上跑的；文件搬了家的话，登记项目根的 .claude/kb-directories（kb-discipline.md），" \
          "或按 .claude/rules/path-moves.md 把这里的路径一起改；写法真改了的话，改这里的 count_invariant_rows（另两格用同一份）。"
  else
    local inv_label marker_hits=() f marker_line_number claim_line inv_claim marker_file
    inv_label="$(inv_table_label)"
    # 当前条数的权威登记位是 <!-- invariant-count --> 下一行那句：历史版本里也有「现共 N 条在用」，那是当时的数、不跟着改，
    # 按文件序取第一处会取到历史里的那一句。标记丢了、不止一处、或下一行读不出那句，都判红，不退回去取第一处。
    # 标记可以住在候选目录里**任一** .md 文件（拆分成多文件时多半落在它们共同的 README.md），不限于带表行的那几份。
    while IFS= read -r f; do
      grep -qF -- '<!-- invariant-count -->' "$f" && marker_hits+=("$f")
    done < <(kb_candidate_md_files)
    if [[ ${#marker_hits[@]} -eq 0 ]]; then
      bad "$inv_label 里没有 <!-- invariant-count --> 标记——当前条数的登记位丢了，读不出正文声称几条（表里在用 $inv_live 条）"
      howto "在「现共 N 条在用」那句的上一行补回 <!-- invariant-count -->；" \
            "不许让检查退回去取文件里第一处「N 条在用」：历史版本里那几句记的是当时的数。"
    elif [[ ${#marker_hits[@]} -gt 1 ]]; then
      bad "$inv_label 里有 ${#marker_hits[@]} 处 <!-- invariant-count --> 标记，登记位该只有一处：${marker_hits[*]}"
      howto "只留一处权威登记位（拆分成多文件时该在它们共同的 README.md 里），其余那句删掉或改成不带标记的历史陈述。"
    else
      marker_file="${marker_hits[0]}"
      marker_line_number=$(grep -n -m1 -F '<!-- invariant-count -->' "$marker_file" | cut -d: -f1)
      claim_line=$(sed -n "$((marker_line_number + 1))p" "$marker_file")
      inv_claim=$(grep -oE '现共 [0-9]+ 条在用' <<<"$claim_line" | grep -oE '[0-9]+' | head -1)
      if [[ -z "$inv_claim" ]]; then
        bad "$marker_file 第 $((marker_line_number + 1)) 行（<!-- invariant-count --> 的下一行）读不出「现共 N 条在用」：${claim_line:0:60}"
        howto "标记的下一行就写那句「现共 $inv_live 条在用（…）」，中间不空行；" \
              "读不出声明就是没核过，不许当成一致。"
      elif [[ "$inv_claim" != "$inv_live" ]]; then
        bad "$inv_label 正文声称在用 $inv_claim 条，实际 $inv_live 条（总行 $inv_actual，退役 $inv_retired$(inv_translation_note)）"
        howto "把正文那句「现共 N 条在用」改成 $inv_live，或者补回漏掉的那几条——" \
              "两个数对不上时，读的人不知道该信哪一个。"
      else
        ok "不变量条数一致：正文声称 $inv_claim 条在用，表里在用 $inv_live 条（总行 $inv_actual，退役 $inv_retired$(inv_translation_note)）"
      fi
    fi
  fi
  # 三段各数各的：欠着的那张表、「### 已还清」那张表、「## 历史版本」里的条目。
  # 分界线是「### 已还清」，不是「## 历史版本」：已还清那张表住在历史版本之前，按历史版本切会把还清的全算进欠账里。
  # 数不出来（文件不在、awk 出错、两张表一行都没数到）判红：两个空白或两个 0 印在成功行里，看着就像数过了。
  if [[ ! -f "$KB/checks-owed.md" ]]; then
    bad "找不到 $KB/checks-owed.md，欠账条数取不到"
    howto "确认门禁是在仓库根上跑的；文件真搬了家的话，按 .claude/rules/path-moves.md 把这里的路径一起改。"
  else
    # 开着与已还清的切法用共用读法 lib-owed.py，不在这里再抄一份 awk
    local chk_counts="" chk_actual chk_done chk_heading
    if chk_counts=$(python3 - "$GATE_DIRECTORY/lib-owed.py" "$KB/checks-owed.md" <<'PY_OWED'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_owed", sys.argv[1])
lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)
table = lib_owed.read_owed_table(sys.argv[2])
print(len(table.open_names), len(table.paid_names), int(table.paid_heading_found))
PY_OWED
    ); then :; else chk_counts=""; fi
    read -r chk_actual chk_done chk_heading <<<"$chk_counts"
    if [[ -z "${chk_actual:-}" || -z "${chk_done:-}" ]]; then
      bad "共用读法 lib-owed.py 没数出 checks-owed.md 的条数（输出「$chk_counts」）"
      howto "单独跑一遍上面那段 python 看它报什么错；数不出来就是没核过，不许当成数过了。"
    elif [[ $((chk_actual + chk_done)) -eq 0 ]]; then
      bad "checks-owed.md 里一行 \`| C<n> \` 都没数到（欠着 0、已还清 0）——欠账表的写法变了，或这份文件是空的"
      howto "欠账行形如 \`| C12 | 简称 | … |\`；写法真改了的话，改 .claude/gate.d/lib-owed.py 的正则，doc-registries、checker-independence-and-sync、code-source-discipline 与这里一起跟上。"
    elif [[ "${chk_heading:-0}" != 1 ]]; then
      bad "checks-owed.md 里认不出「已还清」标题——开着的与已还清的分不开，$chk_actual 条全被算成欠着"
      howto "已还清那张表上面要有一行「### 已还清」；标题改过名的话，改 .claude/gate.d/lib-owed.py 认标题的那条正则。"
    else
      ok "欠检查 $chk_actual 条、已还清 $chk_done 条（checks-owed.md）"
    fi
  fi
  return "$cell_failed"
}

cell_governance_refs() {
  cell_failed=0
  local governance_rc=0
  python3 "$GATE_DIRECTORY/lib-governance-refs.py" || governance_rc=$?
  if [[ $governance_rc -eq 0 ]]; then
    ok "治理文档与 kb 里的门禁号、门禁全名与格名、路径与「小节」都指得到，没有用旧编号称呼门禁"
  elif [[ $governance_rc -eq 3 ]]; then
    bad "一份治理文档都没扫到——这一段没有对象可判"
    howto "确认门禁是在仓库根上跑的；治理文档搬了家的话，改 lib-governance-refs.py 的 CARRIER_PATTERNS。"
  elif [[ $governance_rc -ne 1 ]]; then
    bad "lib-governance-refs.py 没跑成（退出码 $governance_rc）——这一段没判"
    howto "单独跑 python3 $GATE_DIRECTORY/lib-governance-refs.py 看它报什么错；没跑成就是没判，不许当成判过了。"
  else
    bad "治理文档与 kb 里有指不到的指向（上面逐条列出）"
    howto "旧编号与全名：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼，旧编号照红行给的今天的名字改，格名用那道门禁的 --list 看；" \
          "门禁号：阶段被删或收归上游的，改成共享 gate.sh 里那一道的名字（「链接指向」这类）或现存的号；" \
          "路径：搬了家的改成新路径，已删的改指现存的做法，已归档的写裸文件名并指到取法（.claude/agent-common.md「找不到历史实验的数据」那一条）；" \
          "小节：按那份文件今天的标题改。改 agent 定义与共用约束照走门禁 doc-process-records 的 agent-def-adversarial-review 格那一条。"
  fi
  return "$cell_failed"
}

# ── 格 invariant-count-elsewhere：同一个量被别的文件抄过去之后，declared-counts 只看 invariants.md 自己，就没有东西在对了。
# 查的是：`invariants.md` 之外的 kb 文件，正文里写「N 条在用」或「N 条不变量在用」的地方，N 必须等于实际在用条数。
# 一处这样的声明都没有，退 77。抓不到的：
#   1. 不带「在用」二字的写法（「66 条不变量」）：多半在复述某一轮普查当时的口径，照现值判会误判。要它被管住，就把话写成「N 条在用」。
#   2. `## 历史版本` 之后的内容整段跳过（旧数留在那里是 `kb-discipline.md` 第 8 条要求的）。
#   3. `*-history.md` 与 `decisions-history/` 下整份跳过：那些文件通篇是历史。
cell_invariant_count_elsewhere() {
  cell_failed=0
  if ! count_invariant_rows; then
    report_invariant_count_failure
    return 1
  fi
  if [[ ${#inv_files[@]} -eq 0 ]]; then
    echo "  ! 候选 kb 目录里一个带不变量表行的文件都没扫到，这一格本次无对象可判"
    return 77
  fi
  if [[ ${#inv_counted_files[@]} -eq 0 ]]; then
    bad "候选 kb 目录里带不变量表行的 ${#inv_files[@]} 份文件全是译文，没有一份中文原文：实际在用条数没有可数的（条数只数中文原文）"
    howto "不变量表的中文原文 <名>.md 搬了家就按 .claude/rules/path-moves.md 改登记（.claude/kb-directories）；译文先有、中文原文缺的，先补中文原文。"
    return 1
  fi
  local inv_label
  inv_label="$(inv_table_label)"
  printf '%s\n' "${inv_files[@]}" > "$work_directory/invariant-count-elsewhere.inv_files"
  kb_candidate_roots > "$work_directory/invariant-count-elsewhere.roots"
  local elsewhere_rc=0
  # 实际在用条数只数中文原文（count_invariant_rows）；译文里写的「N 条在用」照扫，与中文原文数出的同一个数比
  python3 - "$inv_live" "$inv_label" "$work_directory/invariant-count-elsewhere.inv_files" "$work_directory/invariant-count-elsewhere.roots" "$(inv_translation_note)" <<'PY' || elsewhere_rc=$?
import re, sys, os, glob

live = int(sys.argv[1])
label = sys.argv[2]
inv_paths = set(os.path.abspath(l.strip()) for l in open(sys.argv[3], encoding="utf-8") if l.strip())
roots = [l.strip() for l in open(sys.argv[4], encoding="utf-8") if l.strip()]
translation_note = sys.argv[5].lstrip("；")

scanned, hits, bad = 0, [], []
for root in roots:
    for p in sorted(glob.glob(os.path.join(root, "**", "*.md"), recursive=True)):
        if os.path.abspath(p) in inv_paths or os.path.basename(p).endswith("-history.md") or "/decisions-history/" in p:
            continue
        scanned += 1
        body = open(p, encoding="utf-8").read().split("\n## 历史版本")[0]
        # 「条」与「在用」之间许夹「不变量」三个字（「现有 26 条不变量在用」）；别的名词不认，免得把别的量读成不变量条数
        for m in re.finditer(r"(\d+)\s*条(?:不变量)?\s*在用", body):
            n = int(m.group(1))
            hits.append((p, n))
            if n != live:
                line = body[:m.start()].count("\n") + 1
                bad.append(f"{p}:{line} 写「{m.group(0)}」，而 {label} 实际在用 {live} 条")

if bad:
    print(f"  ✗ 不变量条数跨文件对不上 {len(bad)} 处：")      # gate-lint:summary
    for b in bad:
        print("      " + b)                                   # gate-lint:detail
    print(f"  → 怎么办：以 {label} 为准，把那句里的数改成 {live}；")
    print("    若那句说的其实是某一轮普查当时的口径（不是现值），就把「在用」二字去掉并写明是哪一轮，")
    print("    否则下一个检索到它的人会拿一个旧数当现状。")
    sys.exit(1)

if not hits:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份 kb 文件（{label} 与变更史之外），没有一处写「N 条在用」的声明")
    sys.exit(77)
print(f"  ✓ 不变量条数跨文件一致：实际在用 {live} 条，扫了 {scanned} 份 kb 文件、命中 {len(hits)} 处声明"
      + (f"（实际条数{translation_note}；译文里写的「N 条在用」照扫）" if translation_note else ""))
PY
  if [[ $elsewhere_rc -ne 0 && $elsewhere_rc -ne 1 && $elsewhere_rc -ne 77 ]]; then
    bad "数「N 条在用」的那段 python 没跑成（退出码 $elsewhere_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$elsewhere_rc"
}

# ── 格 citations：还 checks-owed.md C38（外部文献不可复核）的源码那一半。
# 要拦的是证据蒸发（路径变了、版本升了、树被删了），不是 kb 写错；红了怎么处置见 verify-citations.sh 自己打印的下一步。
# 源码树不在也判红，不当跳过：跳过正是让上一批文献无声蒸发的那个行为。
# 真 verify-citations.sh 的判别力不在这里测：它读两个本机绝对路径下的源码树（FS_REFS、KERNEL_TREE 的默认值），装不进密封的样本目录；
# 它自己的 --selftest 合成两棵假树测，由 code-tooling 跑。
CITATIONS_SCRIPT=scripts/kb/verify-citations.sh
citations_pid=""
start_citations() {
  [[ -f "$CITATIONS_SCRIPT" ]] || return 0
  bash "$CITATIONS_SCRIPT" > "$work_directory/citations.out" 2>&1 &
  citations_pid=$!
}
cell_citations() {
  cell_failed=0
  if [[ ! -f "$CITATIONS_SCRIPT" ]]; then
    echo "  ✗ 找不到 $CITATIONS_SCRIPT"
    echo "     → 怎么办：这一格靠它逐条复核引文。文件被挪走就改这里的路径，"
    echo "               还没写就先写它——缺了它，引文一条都没被验过。"
    return 1
  fi
  start_citations
  local citations_rc=0
  wait "$citations_pid" || citations_rc=$?
  cat "$work_directory/citations.out"
  if [[ $citations_rc -ne 0 ]]; then
    bad "外部引用复核没过：$CITATIONS_SCRIPT 退 $citations_rc"
    howto "按上面 verify-citations.sh 自己打印的下一步处置；未命中不等于 kb 写错，也可能是源码固定点没了或版本变了。"
  fi
  return "$cell_failed"
}

# ── 格 invariant-anchors：这次改动在不变量表（invariant_table_files 找到的那几份，今天是 `crates/singlefs-checker/invariants/0*.md`）里新增或改写的每一行不变量（`| I-x.y | …`），
# 正文里必须至少点名一条 `D<n>（简称） 已定项 k`，那是这条不变量要判的字段的落点；点不出来判红。正文写着「此编号不再使用」的不判。
# 判不了的不变量比没有这条更糟：checker 会照跑、照报绿（C69（已定不变量没有字段可判））。
# 射程只到这次改动的行：存量里点不出分项的行一次全判会挡住每一次提交；存量那笔账仍欠着（C69（已定不变量没有字段可判）），
# 所以成功那句每轮都把存量的条数现算着报出来。不判点名的那条分项里到底有没有这个字段、宽度对不对。
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算
# （基准用共用库 scripts/gate/changed-paths.sh 的 gate 取法）。两份 diff 任一份 git 失败判红，不当成「一行没改」。
# 这次改动没有新写或改写任何不变量行，退 77（存量条数照样报出来）。
cell_invariant_anchors() {
  cell_failed=0
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $PWD 不是 git 仓，这一格本次无对象可判"; return 77; }
  count_invariant_rows
  if [[ ${#inv_files[@]} -eq 0 ]]; then
    echo "  ! 候选 kb 目录里一个带不变量表行的文件都没扫到，这一格本次无对象可判"
    return 77
  fi
  local LIB_CHANGED_PATHS base added added_path f is_new anchors_rc=0
  LIB_CHANGED_PATHS="$(cd "$GATE_DIRECTORY/../.." && pwd)/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; return 1; }
  base="$(gate_diff_base gate)"
  added="$(gate_changed_paths "$base" untracked A)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  {
    for f in "${inv_files[@]}"; do
      is_new=0
      while IFS= read -r added_path; do
        [[ "$added_path" == "$f" ]] && { is_new=1; break; }
      done <<<"$added"
      printf '%s\t%s\n' "$f" "$is_new"
    done
  } > "$work_directory/invariant-anchors.files"
  python3 - "$base" "$work_directory/invariant-anchors.files" <<'PY' || anchors_rc=$?
import re, subprocess, sys

base = sys.argv[1]
entries = []
for raw_line in open(sys.argv[2], encoding="utf-8"):
    raw_line = raw_line.rstrip("\n")
    if not raw_line:
        continue
    path, flag = raw_line.rsplit("\t", 1)
    entries.append((path, flag == "1"))
ROW = re.compile(r"^\|\s*(I-[\d.]+)\s*\|")
ITEM = re.compile(r"D\d+（[^）]+）\s*已定项\s*\d+")
RETIRED = "此编号不再使用"

def anchored(line):
    return ITEM.search(line) is not None

# 这次改动新增或改写的行：diff 的 `+` 侧。基准那一版没有这个文件时（新建，含未跟踪），整份都算新写。
# 两份 diff 会重叠（改动已暂存、工作区没再动时两份都有同一行），同一行只算一次，别让报出来的行数翻倍。
# 不变量表拆成几份文件时，逐份各自取「+」侧，再并到同一份 touched 里。
touched = []
for path, file_is_new in entries:
    if file_is_new:
        candidate_lines = open(path, encoding="utf-8").read().split("\n")
    else:
        git_diff = ["git", "-c", "core.quotepath=false", "diff", "-U0"]
        diffs = []
        for arguments in ([base, "--", path], ["--cached", "--", path]):
            shown = subprocess.run([*git_diff, *arguments], capture_output=True, text=True)
            if shown.returncode != 0:
                # git 失败时 + 行是空的：当成「这次没改不变量」就会走到无对象可判，把新写的行整批放过去
                print(f"  ✗ 取不到 {path} 这次改动的 diff（git diff {' '.join(arguments[:-2]) or '基准'} 退 {shown.returncode}：{shown.stderr.strip()[:120]}）")
                print("     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库与索引没坏）再跑；取不到 diff 时这一格什么都没比，不是通过。")
                sys.exit(1)
            diffs.append(shown.stdout)
        diff, staged = diffs
        candidate_lines = [line[1:] for chunk in (diff, staged) for line in chunk.split("\n")
                           if line.startswith("+") and not line.startswith("+++")]
    for body in candidate_lines:
        if ROW.match(body) and RETIRED not in body and body not in touched:
            touched.append(body)

missing = [body for body in touched if not anchored(body)]

# 存量：全部候选文件里点不出分项的行，现算，每轮报出来
total, unanchored_all = 0, 0
for path, _ in entries:
    for line in open(path, encoding="utf-8"):
        if ROW.match(line) and RETIRED not in line:
            total += 1
            if not anchored(line):
                unanchored_all += 1

if missing:
    print(f"  ✗ 这次改动新写或改写的 {len(missing)} 行不变量，没有点名它判的字段住在哪条已定分项（基准 {base}）：")  # gate-lint:summary
    for body in missing:
        identifier = ROW.match(body).group(1)
        print(f"      {identifier}：{body.strip()[:120]}")  # gate-lint:detail
    print("     → 怎么办：在那一行正文里写清这条不变量判的字段住哪，形态是「D<编号>（简称） 已定项 k」。")
    print("               写不出来，说明这条不变量今天判不了任何字节——那正是 C69（已定不变量没有字段可判） 记着的那一类：")
    print("               checker 会老老实实跑它、老老实实报绿。先补字段（在那条决策里定出落点与宽度），或者把这条不变量降级。")
    sys.exit(1)

if not touched:
    print(f"  ⊘ 本次无对象可判：这次改动没有新写或改写不变量行（基准 {base}）；存量 {total} 行里 {unanchored_all} 行还点不出已定分项，那笔账记在 C69（已定不变量没有字段可判）")
    sys.exit(77)
print(f"  ✓ 这次改动新写或改写的 {len(touched)} 行不变量都点名了已定分项（基准 {base}）；"
      f"存量 {total} 行里 {unanchored_all} 行还点不出，那笔账记在 C69（已定不变量没有字段可判）")
PY
  if [[ $anchors_rc -ne 0 && $anchors_rc -ne 1 && $anchors_rc -ne 77 ]]; then
    bad "判不变量字段落点的那段 python 没跑成（退出码 $anchors_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$anchors_rc"
}

# ── 格 inline-sums：还 checks-owed.md C305（kb 里的内联算式没人加）。field-table-sums 只看「合计 N 字节」那种表后合计，行内「84 = 42 + 8 + …」一格都盖不到。
# 判据：扫 kb 候选目录（.claude/kb-directories 登记的目录，没登记就是 .claude/kb）下全部 .md 的正文，每一行里的算式
#   左边：一个连续数字的整数（可带粗体 **，可带「字节」「B」），紧挨它左边不许是字母数字、小数点或 + × ÷ / − – - = ^ 这类运算符；
#   右边：= 之后由 + 连接的两项以上，每项是「可选标签（中文、英文单词、反引号里的名字）加一个整数」，整数后可带「字节」「B」；
#   算完与左边不等判红，打印文件:行号、算式、左右两边的数与那一行。
# 不判、只数进成功句的：整条算式里出现 × ÷ * / − – - ^ ( （ 或后面又接 =（连等），报「跳过 K 条含乘除减括号的」；
#   整数后带小数、千分位、KiB 这类别的单位或认不出的写法，另报一个数。
# 不扫：「## 历史版本」之后、代码围栏里的行、*-history.md 与 decisions-history/ 下的文件（与 invariant-count-elsewhere 同一个口径）。一行里写了几处算式各判各的。
# 一条算式都判不到（或一份 kb 文件都扫不到）退 77，不当通过。
# 样本：inline-sums-red 放左右不等的几种写法（纯数字、带标签、带「字节」、粗体左边、登记目录里的类文件），逐条点名判红；
# inline-sums-green 同样的式子改对，另放含 ×、含括号、带 KiB、连等、代码围栏里、历史节里、*-history.md 里的错式，一条都不判，报判了几条、跳过几条。
# 弄坏开关 DOC_REGISTRIES_BREAK=inline-sums（整格不跑）与 inline-sums-unchecked（左右不等也不判）、inline-sums-operators-judged（含乘除括号的也只加能读出的项）、
# inline-sums-fences-scanned、inline-sums-history-scanned、inline-sums-history-files-scanned：写法与对应的样本在文件头弄坏开关一节。
cell_inline_sums() {
  cell_failed=0
  kb_candidate_roots > "$work_directory/inline-sums.roots"
  local sums_rc=0
  python3 - "$work_directory/inline-sums.roots" "$BREAK_SWITCH" <<'PY' || sums_rc=$?
import glob, os, re, sys

roots = [l.strip() for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
BREAK = sys.argv[2]
FENCE = re.compile(r"^\s*(```|~~~)")
HISTORY = re.compile(r"^##\s*历史版本")
# 数字紧挨的左边：编号、小数、版本号、变量名里的数不是算式的左边
LEFT = re.compile(r"(?<![A-Za-z0-9_.])(\*\*)?(\d+)(\*\*)?[ ]*(?:字节|B(?![A-Za-z0-9_]))?[ ]*=(?![=>])")
LIST_MARK = re.compile(r"\s*(?:[-*+]|\d+[.)])")
OPERATORS_BEFORE = "+×÷/−–-=^<>~≈≠≤≥"
OPERATORS_AFTER = "×÷*/−–-^(（=<>≈"
OPERATORS_OPENING = "(（−–-"
LABEL_WORD = r"(?:[\u4e00-\u9fffA-Za-z_]+|`[^`\n]*`)"
LABEL = rf"{LABEL_WORD}(?: +{LABEL_WORD})*"
TERM = re.compile(rf" *(?:(?P<label>{LABEL})(?P<gap> *))?\*{{0,2}}(?P<int>\d+)\*{{0,2}}(?:[ ]*(?:字节|B(?![A-Za-z0-9_])))?")
PLUS = re.compile(r" *\+(?!\+)")
CLAUSE_END = "，。；、;|`"

scanned = judged = skipped_operators = skipped_unrecognized = 0
bad = []

def kb_files():
    seen = set()
    for root in roots:
        for path in sorted(glob.glob(os.path.join(root, "**", "*.md"), recursive=True)):
            history_file = os.path.basename(path).endswith("-history.md") or "/decisions-history/" in path
            if path in seen or (history_file and BREAK != "inline-sums-history-files-scanned"):
                continue
            seen.add(path)
            yield path

def current_lines(path):
    in_fence = False
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
        if FENCE.match(line):
            in_fence = not in_fence if BREAK != "inline-sums-fences-scanned" else False
            continue
        if in_fence:
            continue
        if HISTORY.match(line) and BREAK != "inline-sums-history-scanned":
            return
        yield number, line

def classify(line, start):
    """解析 start（等号之后）起的右边：→ (类, 各项, 右边结束位置)。类：sum、ops、unrecognized、none（不是这个形态）。"""
    terms, index, plus_seen = [], start, False
    while True:
        term = TERM.match(line, index)
        glued = term and term.group("label") and not term.group("gap") and term.group("label")[-1] != "`" and term.group("label")[-1].isascii()
        if not term or glued:
            rest = line[index:].lstrip(" ")
            if not terms:
                return ("ops" if rest[:1] and rest[0] in OPERATORS_OPENING and "+" in chunk_of(rest) else "none"), terms, index
            return "unrecognized", terms, index
        value, end = int(term.group("int")), term.end()
        tail = line[end:]
        follow = tail.lstrip(" ")[:1]
        terms.append(value)
        plus = PLUS.match(line, end)
        if plus:
            index, plus_seen = plus.end(), True
            continue
        has_plus = plus_seen or "+" in chunk_of(tail)
        if follow and follow in OPERATORS_AFTER:
            return ("ops" if has_plus else "none"), terms, end
        if re.match(r"\.\d|,\d{3}(?!\d)|[%^~_]|[A-Za-z]", tail.lstrip(" ")):
            return ("unrecognized" if has_plus else "none"), terms, end
        return ("sum" if len(terms) >= 2 else "none"), terms, end

def chunk_of(text):
    for position, character in enumerate(text):
        if character in CLAUSE_END:
            return text[:position]
    return text

for path in kb_files():
    scanned += 1
    for number, line in current_lines(path):
        for left in LEFT.finditer(line):
            before = line[:left.start()].rstrip(" ")
            if before and not LIST_MARK.fullmatch(before) and before[-1] in OPERATORS_BEFORE:
                continue
            if line[left.start() - 1:left.start()] == "," and left.start() >= 2 and line[left.start() - 2].isdigit():
                continue
            kind, terms, end = classify(line, left.end())
            if kind == "ops" and BREAK == "inline-sums-operators-judged":
                kind = "sum"
            if kind == "ops":
                skipped_operators += 1
            elif kind == "unrecognized":
                skipped_unrecognized += 1
            elif kind == "sum":
                judged += 1
                want, got = int(left.group(2)), sum(terms)
                if want != got and BREAK != "inline-sums-unchecked":
                    bad.append((path, number, line[left.start():end].strip(), want, got, line))

if bad:
    print(f"  ✗ kb 正文里 {len(bad)} 条内联算式左右不等（扫 {scanned} 份、判了 {judged} 条）：")  # gate-lint:summary
    shown = set()
    for path, number, expression, want, got, line in bad:
        print(f"      {path}:{number} 「{expression}」左边是 {want}，右边是 {got}（右边比左边{'多' if got > want else '少'} {abs(got - want)}）")  # gate-lint:detail
        if (path, number) not in shown:
            shown.add((path, number))
            print(f"          原行：{line}")  # gate-lint:detail
    print("  → 怎么办：按右边各项重新加一遍，是合计写错了就改合计，是右边漏了或多了一项就补上或去掉那一项；")
    print("    这一行其实不是在求和（右边的数不是加数）时，改写成不会被读成「整数 = 项 + 项」的句子，不要凑数。")
    sys.exit(1)
if not scanned or not judged:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份 kb 文件，一条「整数 = 项 + 项 + …」的算式都没有")
    sys.exit(77)
print(f"  ✓ kb 正文里的内联算式左右相等：扫 {scanned} 份 kb 文件（「## 历史版本」之后、代码围栏、变更史除外），判了 {judged} 条；"
      f"跳过 {skipped_operators} 条含乘除减括号的、{skipped_unrecognized} 条带小数、其他单位或认不出写法的")
PY
  if [[ $sums_rc -ne 0 && $sums_rc -ne 1 && $sums_rc -ne 77 ]]; then
    bad "判内联算式的那段 python 没跑成（退出码 $sums_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$sums_rc"
}

# ── 格 invariant-prerequisites：还 checks-owed.md C446（未实现的不变量没有前置登记，也没有到期）。
# declared-counts 与 invariant-count-elsewhere 只核条数，没有一格问「未实现那几条被什么挡着、挡它的东西还在不在」。
# 判据：不变量表（invariant_table_files 找到的那几份，与 declared-counts 同一份读法）里每一行 `| I-x.y | … |`，最后一格（状态）去掉 ** 之后以「未实现」起头的，
#   状态格里要有「前置：」，它后面（到第一个 。；; 或 ⚠ 为止）至少写一个编号，每个编号都指得到、而且不能全都已经满足：
#     C<n>              checks-owed.md 里有这一行（按共用读法 lib-owed.py）；在「### 已还清」那张表里算已满足
#     D<n> 已定项 k     decisions/ 下 `## D<n> ` 那份文件「### 已定项」索引里有第 k 行；那一行写「进度：已实现」或「进度：无对象」算已满足
#     D<n> 未定项 k     「### 未定项」索引里有第 k 行；还开着就是没满足
#     E<n>              experiments/ 下有 `## E<n> ` 标题；标题「——」后的状态以「已跑」起头算已满足（「部分已跑」不算）
#   编号后面可以带（简称），简称里的字不当编号读。只写 D<n> 不写分项的判红（要写到分项，才判得出满足没有）。
# 判红：没写前置；前置里一个编号都没有（只有散文）；编号指不到；只写决策号；写的编号全都已经满足（可以开工而没人开工）。
# 看不见：里程碑步号（「三·5」这类）不认，写了也不算编号；前置里编号之外的散文不判；状态不以「未实现」起头、而正文里另有「未实现」字样的行不判（「已实现」「部分」照旧不管）。
# 一份不变量表文件都扫不到、或未实现 0 行，退 77。
# 样本：invariant-prerequisites-red 一份不变量类文件放没写前置、只有散文、已还清的 C、不存在的 C、进度已实现的 D 分项、只写决策号、已跑的 E、不存在的未定项各一行，逐行点名判红，
#   另有一行前置还开着的不点名；invariant-prerequisites-green 前置开着的 C、未实现的已定项、开着的未定项、没跑的 E、一满足一没满足的两个编号各一行，判绿并报几行几个编号。
# 弄坏开关 DOC_REGISTRIES_BREAK=invariant-prerequisites（整格不跑）、prerequisites-satisfied-unchecked（前置全都满足了也不判）、
#   prerequisites-existence-unchecked（编号指不到也不判）：写法与对应的样本在文件头弄坏开关一节。
cell_invariant_prerequisites() {
  cell_failed=0
  invariant_table_files > "$work_directory/invariant-prerequisites.files"
  local prerequisites_rc=0
  python3 - "$work_directory/invariant-prerequisites.files" "$OWED_LIBRARY" "$KB" "$BREAK_SWITCH" <<'PY' || prerequisites_rc=$?
import glob, importlib.util, os, re, sys

invariant_list, owed_library, KB, BREAK = sys.argv[1:5]
spec = importlib.util.spec_from_file_location("lib_owed", owed_library)
lib_owed = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_owed)

files = [l.strip() for l in open(invariant_list, encoding="utf-8") if l.strip()]
ROW = re.compile(r"^\|\s*(I-\d+\.\d+)\s*\|")
PREREQUISITE = re.compile(r"前置\s*[：:]")
SEGMENT_END = re.compile(r"[。；;]|⚠")
PAREN = r"(?:（(?:[^（）]|（[^（）]*）)*）|\((?:[^()]|\([^()]*\))*\))?"
REFERENCE = re.compile(
    rf"(?<![A-Za-z0-9_])(?:D(?P<item_decision>\d+){PAREN}\s*(?P<kind>已定项|未定项)\s*(?P<item>\d+)(?![0-9])"
    rf"|D(?P<bare_decision>\d+)(?![0-9]){PAREN}"
    rf"|C(?P<owed>\d+)(?![0-9]){PAREN}"
    rf"|E(?P<experiment>\d+)(?![0-9]){PAREN})")
TRANSLATION = (".en.md", ".ja.md")


def current_text(path):
    return open(path, encoding="utf-8", errors="replace").read().split("\n## 历史版本")[0]


def numbered_headings(directory, letter):
    """编号 → 那份文件的路径与标题行（译本不算）。"""
    found = {}
    for path in sorted(glob.glob(os.path.join(KB, directory, "**", "*.md"), recursive=True)):
        if path.endswith(TRANSLATION):
            continue
        for line in current_text(path).splitlines():
            heading = re.match(rf"^##\s+{letter}(\d+)\s(.*)$", line)
            if heading:
                found.setdefault(int(heading.group(1)), (path, heading.group(2)))
    return found


def decision_items(path):
    """「### 已定项」「### 未定项」索引（到下一个标题为止）里的行：项号 → 整行。"""
    items = {"已定项": {}, "未定项": {}}
    current = None
    for line in current_text(path).splitlines():
        section = re.match(r"^###\s*(已定项|未定项)\s*$", line)
        if section:
            current = items[section.group(1)]
            continue
        if re.match(r"^#+\s", line):
            current = None
            continue
        if current is not None:
            row = re.match(r"^\|\s*(\d+)\s*\|", line) or re.match(r"^(\d+)\.\s", line)
            if row:
                current.setdefault(int(row.group(1)), line)
    return items


owed = lib_owed.read_owed_table(os.path.join(KB, "checks-owed.md"))
decisions = numbered_headings("decisions", "D")
experiments = numbered_headings("experiments", "E")
items_cache = {}


def resolve(reference):
    """→ (写法, 指得到没有, 满足没有, 不成立时的原因)。"""
    if reference.group("owed"):
        name = f"C{reference.group('owed')}"
        if not owed.file_found or not owed.paid_heading_found:
            return name, False, False, f"判不了：{KB}/checks-owed.md 读不出开着与已还清两张表"
        if name in owed.paid_names:
            return name, True, True, ""
        if name in owed.open_names:
            return name, True, False, ""
        return name, False, False, "指不到：checks-owed.md 两张表里都没有这一行"
    if reference.group("experiment"):
        number = int(reference.group("experiment"))
        name = f"E{number}"
        if number not in experiments:
            return name, False, False, "指不到：experiments/ 下没有 `## " + name + " ` 标题"
        status = experiments[number][1].split("——", 1)[1].strip() if "——" in experiments[number][1] else ""
        return name, True, status.startswith("已跑"), ""
    if reference.group("bare_decision"):
        name = f"D{reference.group('bare_decision')}"
        return name, False, False, "只写了决策号，要写到分项（D<n> 已定项 k / 未定项 k），才判得出满足没有"
    number, kind, item = int(reference.group("item_decision")), reference.group("kind"), int(reference.group("item"))
    name = f"D{number} {kind} {item}"
    if number not in decisions:
        return name, False, False, f"指不到：decisions/ 下没有 `## D{number} ` 标题"
    path = decisions[number][0]
    if path not in items_cache:
        items_cache[path] = decision_items(path)
    row = items_cache[path][kind].get(item)
    if row is None:
        return name, False, False, f"指不到：{path} 的「### {kind}」索引里没有第 {item} 行"
    if kind == "未定项":
        return name, True, False, ""
    return name, True, bool(re.search(r"进度[：:]\s*\**\s*(已实现|无对象)", row)), ""


unimplemented = references_total = 0
kinds = {"C": 0, "D": 0, "E": 0}
bad = []   # (路径, 行号, 编号, 原因)
for path in files:
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
        row = ROW.match(line)
        if not row:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        status = cells[-1].replace("**", "").strip()
        if not status.startswith("未实现"):
            continue
        unimplemented += 1
        invariant = row.group(1)
        marker = PREREQUISITE.search(status)
        if not marker:
            bad.append((path, number, invariant, "状态写「未实现」，没写「前置：」——它被什么挡着没有登记"))
            continue
        segment = status[marker.end():]
        end = SEGMENT_END.search(segment)
        segment = (segment[:end.start()] if end else segment).strip()
        references = list(REFERENCE.finditer(segment))
        if not references:
            bad.append((path, number, invariant, f"前置「{segment[:60]}」里一个编号都没有——散文判不出满足没有"))
            continue
        problems, satisfied = [], []
        for reference in references:
            references_total += 1
            name, exists, done, why = resolve(reference)
            kinds[name[0]] += 1
            if not exists and BREAK != "prerequisites-existence-unchecked":
                problems.append(f"前置 {name} {why}")
            satisfied.append(done)
        if not problems and all(satisfied) and BREAK != "prerequisites-satisfied-unchecked":
            names = "、".join(resolve(r)[0] for r in references)
            problems.append(f"前置 {names} 都已经满足，状态仍写「未实现」——这一条可以开工了")
        for problem in problems:
            bad.append((path, number, invariant, problem))

if bad:
    print(f"  ✗ 不变量表里 {len(bad)} 处未实现的不变量前置不成立（未实现 {unimplemented} 行，扫 {len(files)} 份）：")  # gate-lint:summary
    for path, number, invariant, problem in bad:
        print(f"      {path}:{number} {invariant}：{problem}")  # gate-lint:detail
    print("  → 怎么办：在那一行状态格「未实现」后面写「，前置：<编号>」，编号写 C<n>（欠账）、D<n>（简称） 已定项 k / 未定项 k（决策分项）或 E<n>（实验），")
    print("    挡着它的事还没有编号就先去立一笔欠账；指不到的改成今天存在的编号；前置都已满足的，开工实现它、或写明现在挡着它的新编号。")
    sys.exit(1)
if not files or not unimplemented:
    print(f"  ! 本次无对象可判：扫了 {len(files)} 份不变量表文件，状态写「未实现」的 0 行")
    sys.exit(77)
print(f"  ✓ 未实现的不变量都写了指得到、还没全满足的编号前置：扫 {len(files)} 份不变量表文件，未实现 {unimplemented} 行，"
      f"前置 {references_total} 个编号（C {kinds['C']}、D 分项 {kinds['D']}、E {kinds['E']}）")
PY
  if [[ $prerequisites_rc -ne 0 && $prerequisites_rc -ne 1 && $prerequisites_rc -ne 77 ]]; then
    bad "判未实现不变量前置的那段 python 没跑成（退出码 $prerequisites_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$prerequisites_rc"
}

# ── 格 field-frequency-sizes：还 checks-owed.md C72（extent 和单元什么关系没人答）的门禁比对那一半：「每逻辑 extent 一份」与「覆盖 32 KiB」
#   说的是不是同一个粒度，要能从已定条款里读出来。
# 判据：扫 $KB/decisions/ 与 $KB/layout/ 下的 .md 正文（译本 *.en.md / *.ja.md、*-history.md、decisions-history/、代码围栏里、「## 历史版本」之后都不扫；
#   去掉 ** 再认），找「每[个|一个] X [恒|各|只|都|存|带|持|有|写] 一份」：
#   X 是下面粒度表里的写法 ⇒ 要读得出它的大小：
#     ① 它读成的格式常量在 decisions/ 下有 format-const 标记（lib-format-const.py 的文法），而且至少一处住在某条决策的「#### 已定项 k」小节里、
#        那条分项索引行写着「状态：已定」；
#     ② 那个常量在 decisions/ 与 layout/ 里登记的值只有一个；
#     ③ 粒度表点名的那条条款（下表末两列）在那份决策「### 已定项」索引里、写着「状态：已定」，它的索引行加「#### 已定项 k」小节里写出了这个大小
#        （字节数，或整除时的「N KiB」「N MiB」）。
#   X 是字面大小（「每 512 字节一份」「每 32 KiB 一份」）⇒ 换成字节数，要等于粒度表里某个读得出大小的粒度。
#   粒度表（正文写法 → 格式常量，给出它等于这个大小的已定条款）：
#     逻辑 extent、extent → DATA_UNIT_BYTES，D4 已定项 7（一个逻辑 extent 占盘恒为一个单元）
#     数据单元、单元      → DATA_UNIT_BYTES，D4 已定项 5（单元恒 32768 字节、含头）
#     索引节点、节点      → NODE_BYTES，D8 已定项 2（节点大小 16 KiB）
# 豁免：不是这个格式的字段频率（否决掉的候选设计、别家系统的做法）在那一行正上方单起一行写 <!-- doc-registries:not-field-frequency <为什么> -->，
#   它只管下一个非空行；理由去掉空白不到 4 个字、管的那一行里一处「每 X 一份」都没有，都判红。
# 看不见：粒度表外的 X（「每物理位置一份」「每头一份」）不判；「一份」之外的量词（一个、一条）不认；否定句（「不是每个单元一份」）照判 X 的大小；
#   「单元」一律读成数据单元（码 2 索引节点也叫单元，16 KiB，写的人要说节点）；粒度表点名的条款号在这里写死，那条分项改了号要跟着改这张表。
# 一处「每 X 一份」都没扫到退 77。
# 样本：field-frequency-sizes-red 删掉 D4 已定项 1 的 DATA_UNIT_BYTES 登记（C72 的判别力自证）、D8 已定项 2 没写出 16 KiB、字面 128 KiB、
#   不起作用的豁免与没写理由的豁免，逐条点名判红；field-frequency-sizes-registration-red DATA_UNIT_BYTES 登记在未定项小节里、NODE_BYTES 在 layout 里另登记一个值、
#   D4 已定项 7 不是已定，逐条点名判红；field-frequency-sizes-green 各粒度都读得出、字面 32 KiB、带豁免的 512 字节、否定句、代码围栏、历史节与译本里的错写不判，报判了几处。
# 弄坏开关 DOC_REGISTRIES_BREAK=field-frequency-sizes（整格不跑）、field-frequency-mark-anywhere（登记住在哪一节都认）、
#   field-frequency-literal-unchecked（字面大小不判）、field-frequency-markers-ignored（豁免标记不认）：写法与对应的样本在文件头弄坏开关一节。
cell_field_frequency_sizes() {
  cell_failed=0
  local frequency_rc=0
  python3 - "$FORMAT_CONST_LIBRARY" "$KB" "$BREAK_SWITCH" <<'PY' || frequency_rc=$?
import glob, importlib.util, os, re, sys

format_const_library, KB, BREAK = sys.argv[1:4]
spec = importlib.util.spec_from_file_location("lib_format_const", format_const_library)
lib_format_const = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib_format_const)

# 粒度表：正文写法 → (格式常量, 给出大小的决策号, 已定项号)。长写法排在它的短写法前面，正则按这个次序试。
GRANULES = (
    ("逻辑 extent", "DATA_UNIT_BYTES", 4, 7),
    ("extent", "DATA_UNIT_BYTES", 4, 7),
    ("数据单元", "DATA_UNIT_BYTES", 4, 5),
    ("单元", "DATA_UNIT_BYTES", 4, 5),
    ("索引节点", "NODE_BYTES", 8, 2),
    ("节点", "NODE_BYTES", 8, 2),
)
UNIT_BYTES = {"KiB": 1024, "MiB": 1024 * 1024, "字节": 1, "B": 1}
WORDS = "|".join(re.escape(word).replace(r"\ ", r"\s*") for word, *_ in GRANULES)
FREQUENCY = re.compile(
    rf"每(?:个|一个)?\s*(?:(?P<word>{WORDS})|(?P<number>\d+)\s*(?P<unit>KiB|MiB|字节|B(?![A-Za-z])))"
    r"\s*(?:恒|各|只|都|存|带|持|有|写)?\s*一份")
MARKER = re.compile(r"^\s*<!--\s*doc-registries:not-field-frequency\b(?P<reason>.*?)-->\s*$")
FENCE = re.compile(r"^\s*(```|~~~)")
HISTORY = re.compile(r"^##\s*历史版本")
TRANSLATION = (".en.md", ".ja.md")


def scanned_files(directory):
    for path in sorted(glob.glob(os.path.join(KB, directory, "**", "*.md"), recursive=True)):
        if path.endswith(TRANSLATION) or path.endswith("-history.md") or "/decisions-history/" in path:
            continue
        yield path


def current_lines(path):
    """（行号, 行）：代码围栏里与「## 历史版本」之后的不给。"""
    in_fence = False
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace").read().split("\n"), 1):
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if HISTORY.match(line):
            return
        yield number, line


def granule_word(text):
    squeezed = re.sub(r"\s+", " ", text)
    for word, *_ in GRANULES:
        if squeezed == word:
            return word
    return None


decision_files = list(scanned_files("decisions"))
layout_files = list(scanned_files("layout"))

# ── 决策正文：每份的决策号、已定项索引行、「#### 已定项 k」小节正文、format-const 标记落在哪一节
decision_by_number, settled_rows, settled_sections, marks = {}, {}, {}, []
for path in decision_files:
    lines = list(current_lines(path))
    number = None
    for _, line in lines:
        heading = re.match(r"^##\s+D(\d+)\s", line)
        if heading:
            number = int(heading.group(1))
            decision_by_number.setdefault(number, path)
            break
    in_index, section = False, None
    for line_number, line in lines:
        if re.match(r"^###\s*已定项\s*$", line):
            in_index, section = True, None
            continue
        sub = re.match(r"^####\s*(已定项|未定项)\s*(\d+)", line)
        if re.match(r"^#+\s", line):
            in_index = False
            section = (sub.group(1), int(sub.group(2))) if sub else None
            continue
        if in_index and number is not None:
            row = re.match(r"^\|\s*(\d+)\s*\|", line)
            if row:
                settled_rows.setdefault((number, int(row.group(1))), line)
        if section and section[0] == "已定项" and number is not None:
            settled_sections.setdefault((number, section[1]), []).append(line)
        for mark in lib_format_const.parse_marks(line).marks:
            marks.append((mark.name, mark.value, path, line_number, number, section))
for path in layout_files:
    for line_number, line in current_lines(path):
        for mark in lib_format_const.parse_marks(line).marks:
            marks.append((mark.name, mark.value, path, line_number, None, None))


def row_settled(row):
    return bool(row) and bool(re.search(r"状态[：:]\s*\**\s*已定", row))


def size_spellings(value):
    spellings = [rf"(?<![\d.]){value}(?!\d)"]
    for unit, factor in (("KiB", 1024), ("MiB", 1024 * 1024)):
        if value % factor == 0:
            spellings.append(rf"(?<![\d.]){value // factor}\s*{unit}")
    return re.compile("|".join(spellings))


granule_sizes, granule_problems = {}, {}
for word, constant, decision, item in GRANULES:
    problems = []
    registered = [m for m in marks if m[0] == constant]
    in_decisions = [m for m in registered if m[4] is not None]
    settled_marks = [m for m in in_decisions
                     if BREAK == "field-frequency-mark-anywhere"
                     or (m[5] and m[5][0] == "已定项" and row_settled(settled_rows.get((m[4], m[5][1]))))]
    if not in_decisions:
        problems.append(f"decisions/ 下没有 {constant} 的 format-const 登记")
    elif not settled_marks:
        places = "、".join(f"{m[2]}:{m[3]}" for m in in_decisions)
        problems.append(f"{constant} 登记在 {places}，不在任何写着「状态：已定」的已定项小节里")
    values = sorted({m[1] for m in registered})
    if len(values) > 1:
        places = "、".join(f"{m[2]}:{m[3]}（{m[1]}）" for m in registered)
        problems.append(f"{constant} 登记了不止一个值：{places}")
    clause = f"D{decision} 已定项 {item}"
    row = settled_rows.get((decision, item))
    if decision not in decision_by_number:
        problems.append(f"给出它大小的 {clause}：decisions/ 下没有 `## D{decision} ` 标题")
    elif row is None:
        problems.append(f"给出它大小的 {clause}：{decision_by_number[decision]} 的「### 已定项」索引里没有第 {item} 行")
    elif not row_settled(row):
        problems.append(f"给出它大小的 {clause} 没写「状态：已定」")
    elif settled_marks and len(values) == 1:
        # 标记自己写着这个数，不算条款写出了它：先去掉标记再找
        clause_text = lib_format_const.strip_marks("\n".join([row] + settled_sections.get((decision, item), [])))
        if not size_spellings(values[0]).search(clause_text):
            problems.append(f"给出它大小的 {clause} 里没写出 {constant} 登记的 {values[0]}（字节数，或 KiB / MiB 写法）")
    if problems:
        granule_problems[word] = problems
    else:
        granule_sizes[word] = (constant, values[0], clause)

resolved_sizes = sorted({size for _, size, _ in granule_sizes.values()})
hits, exempted, by_kind = 0, 0, {}
bad = []      # (路径, 行号, 原文片段, 原因)
for path in decision_files + layout_files:
    lines = list(current_lines(path))
    governed = {}
    for position, (line_number, line) in enumerate(lines):
        marker = MARKER.match(line)
        if not marker:
            continue
        if BREAK == "field-frequency-markers-ignored":
            continue
        following = next(((n, l) for n, l in lines[position + 1:] if l.strip()), None)
        reason = re.sub(r"\s+", "", marker.group("reason"))
        if len(reason) < 4:
            bad.append((path, line_number, line.strip()[:80], "not-field-frequency 标记没写理由（去掉空白至少 4 个字）"))
        if following is None or not FREQUENCY.search(following[1].replace("**", "")):
            bad.append((path, line_number, line.strip()[:80], "not-field-frequency 标记管的下一个非空行里一处「每 X 一份」都没有，是不起作用的标记"))
        else:
            governed[following[0]] = True
    for line_number, line in lines:
        text = line.replace("**", "")
        for hit in FREQUENCY.finditer(text):
            if MARKER.match(line):
                continue
            hits += 1
            phrase = hit.group(0)
            if governed.get(line_number):
                exempted += 1
                continue
            if hit.group("word"):
                word = granule_word(hit.group("word"))
                by_kind[word] = by_kind.get(word, 0) + 1
                if word in granule_problems:
                    bad.append((path, line_number, phrase, f"「{word}」的大小读不出来：" + "；".join(granule_problems[word])))
                continue
            by_kind["字面大小"] = by_kind.get("字面大小", 0) + 1
            size = int(hit.group("number")) * UNIT_BYTES[hit.group("unit")]
            if size not in resolved_sizes and BREAK != "field-frequency-literal-unchecked":
                known = f"读得出大小的粒度：{'、'.join(str(s) for s in resolved_sizes)} 字节" if resolved_sizes else "没有一个粒度读得出大小"
                bad.append((path, line_number, phrase, f"{size} 字节不是哪条已定条款给出大小的粒度（{known}）"))

scanned = len(decision_files) + len(layout_files)
if bad:
    print(f"  ✗ {len(bad)} 处字段频率的大小指不到已定条款（扫 {scanned} 份、{hits} 处「每 X 一份」）：")  # gate-lint:summary
    for path, line_number, phrase, problem in sorted(bad, key=lambda entry: (entry[0], entry[1])):
        print(f"      {path}:{line_number} 「{phrase}」：{problem}")  # gate-lint:detail
    print("  → 怎么办：粒度的大小读不出来，先让给出大小的那条已定分项写出这个数，并在它的「#### 已定项 k」小节里登记 <!-- format-const: 名字 = 值 -->；")
    print("    字面大小对不上，改成已定的粒度（单元、节点）或把字段频率写对；这句说的不是本格式的字段（否决掉的候选、别家系统），")
    print("    在那一行正上方单起一行写 <!-- doc-registries:not-field-frequency <为什么> -->（判据在 doc-registries.sh 里这一格函数上方的注释）。")
    sys.exit(1)
if not hits:
    print(f"  ! 本次无对象可判：扫了 {scanned} 份决策与布局正文，一处「每 X 一份」都没有")
    sys.exit(77)
kinds = "、".join(f"{word} {count}" for word, count in by_kind.items())
clauses_by_constant = {}
for constant, size, clause in granule_sizes.values():
    entry = clauses_by_constant.setdefault((constant, size), [])
    if clause not in entry:
        entry.append(clause)
sizes = "，".join(f"{constant} = {size}（{'、'.join(clauses)}）" for (constant, size), clauses in clauses_by_constant.items())
print(f"  ✓ 字段频率「每 X 一份」的 X 都指得到已定条款给出的大小：扫 {scanned} 份，{hits} 处（{kinds}；豁免 {exempted} 处）；读得出大小的粒度：{sizes}")
PY
  if [[ $frequency_rc -ne 0 && $frequency_rc -ne 1 && $frequency_rc -ne 77 ]]; then
    bad "判字段频率大小的那段 python 没跑成（退出码 $frequency_rc）——这一格没判"
    howto "按上面的 traceback 修；没跑成就是没判，不许当成判过了。"
    return 1
  fi
  return "$frequency_rc"
}

# ════ 登记的格函数：弄坏开关点名这一格就不跑、按判过了记，否则调这一格的判法 ════
# 字段与布局组九格的 stdin 接到 /dev/null，与它们并进来之前在后台起时一致。
break_skips() {
  if [[ "$BREAK_SWITCH" == "$1" ]]; then
    echo "  ! 弄坏开关 DOC_REGISTRIES_BREAK=$1：这一格没判"
    exit 0
  fi
}
judge_field_refs()                   { break_skips field-refs;                   cell_field_refs </dev/null; }
judge_field_projection()             { break_skips field-projection;             cell_field_projection </dev/null; }
judge_field_table_sums()             { break_skips field-table-sums;             cell_field_table_sums </dev/null; }
judge_new_pool_file_creation_hooks()              { break_skips first-txn-hooks;              cell_new_pool_file_creation_hooks </dev/null; }
judge_admission_terms()              { break_skips admission-terms;              cell_admission_terms </dev/null; }
judge_segment_registry()             { break_skips segment-registry;             cell_segment_registry </dev/null; }
judge_format_const_placeholders()    { break_skips format-const-placeholders;    cell_format_const_placeholders </dev/null; }
judge_overwrite_release_rollback_reuse_hooks()             { break_skips second-txn-hooks;             cell_overwrite_release_rollback_reuse_hooks </dev/null; }
judge_tree_table_reserve()           { break_skips tree-table-reserve;           cell_tree_table_reserve </dev/null; }
judge_table_shape()                  { break_skips table-shape;                  cell_table_shape; }
judge_closeout_collects_open()       { break_skips closeout-collects-open;       cell_closeout_collects_open; }
judge_paid_cited_tests()             { break_skips paid-cited-tests;             cell_paid_cited_tests; }
judge_audit_contradictions()         { break_skips audit-contradictions;         cell_audit_contradictions; }
judge_closeout_row27_preconditions() { break_skips closeout-row27-preconditions; cell_closeout_row27_preconditions; }
judge_experiment_refs()              { break_skips experiment-refs;              cell_experiment_refs; }
judge_decision_refs()                { break_skips decision-refs;                cell_decision_refs; }
judge_declared_counts()              { break_skips declared-counts;              cell_declared_counts; }
judge_governance_refs()              { break_skips governance-refs;              cell_governance_refs; }
judge_invariant_count_elsewhere()    { break_skips invariant-count-elsewhere;    cell_invariant_count_elsewhere; }
judge_citations()                    { break_skips citations;                    cell_citations; }
judge_invariant_anchors()            { break_skips invariant-anchors;            cell_invariant_anchors; }
judge_inline_sums()                  { break_skips inline-sums;                  cell_inline_sums; }
judge_invariant_prerequisites()      { break_skips invariant-prerequisites;      cell_invariant_prerequisites; }
judge_field_frequency_sizes()        { break_skips field-frequency-sizes;        cell_field_frequency_sizes; }

stage_cells_run "$ROOT"
