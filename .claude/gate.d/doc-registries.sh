#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录；外部引用那一格调的 scripts/kb/verify-citations.sh 要共享数据根下固定下来的外部文献树（fs-refs、kernel-tree，两台同一个绝对路径），缺了在那一格判红，不交给 gate.sh 预判
# gate-stage: 登记表与引用（三十一格：字段与布局登记表之间的账；单元头字段表与 D18 逐项比；欠账表与收口表的形状与去向；实验号、决策号、门禁名、路径、小节与外部引用指得到；不变量条数与字段落点；kb 正文里的内联算式左右相等；未实现的不变量写了编号前置；字段频率「每 X 一份」的 X 指得到已定条款给出的大小；字段行的宽度写在被指的分项里；删除写出的字节三处挂钩；挂载透明字段表与 kb 对账；kb 新增的「不可达」类断言带用例名或写推的；CLAUDE.md 与仓根三份 README 里登记的能数的数与命令现数相等）
# gate-fixture: yes
# gate-category: 文档类
# gate-similar: link-targets.py 它只解析 ](相对路径) 链接与「第 N 节」，不看反引号里的路径、「门禁 N 号」与「小节」名；这三类与 experiment-refs、decision-refs 两格同是「引用了不存在的东西」，并在这里
# gate-similar: doc-decisions.sh 它判决策文档本身（索引结论字数、分项引用状态、未定项写没写改不改新池新建文件的字节、待用户复核的决策记没记欠账）；这一道判字段表指向的分项存不存在、判「是」的分项在字节表与里程碑里挂没挂钩、认购表求和、欠账表与收口表的形状与去向，decision-refs 只判 D 号有定义
# gate-similar: checker-independence-and-sync.sh 它的 layout-checker-sync 格判 layouts.tsv 登记的布局常量变了 checker 跟没跟，对象是布局清单与 checker 源码；这一道只对 kb 各表之间、kb 与 E142 产物和钉住的用例之间的账
# gate-similar: code-source-discipline.sh 它的 reading-discipline、mutation-tables、format-constants、clause-enums、feature-bits 几格方向是 kb 到代码，拿登记值核 crates 源码；这一道的 field-table-sums 与它共用 lib-format-const.py 读标记但只判 kb 里的加和与登记，format-const-placeholders 只核代码注释里的编号指得到 kb，closeout-row27-preconditions 只数几段写死的原文命中几次，都不拿 kb 的值比代码
# gate-similar: doc-experiments.sh 它的 multipath-registry 格判实验页登记表的形状，quoted-result-lines 格判 kb 整行抄的产物行在 research/results/ 里找得到（kb 到产物）；这一道 audit-contradictions 格判总审核处置列引的原文在 kb 今天的正文里找得到（记录到 kb），另判欠账号还开着
# gate-similar: doc-text.sh 它判 kb 与全仓文本的写法（角标、钟点、会话号），不核数；这一道的 inline-sums 格核 kb 正文里写出来的「整数 = 项 + 项」算式左右两边的数相等，field-table-sums 只看表后合计，两者都不并进 doc-text；counted-claims 格核 CLAUDE.md 与仓根三份 README 里登记的数与登记的命令此刻数出来的相等，同样是核数、不判写法，doc-text 判那三份 README 的结构对齐与同进同出，不看里面的数
#
# 三组格：字段与布局组十三格（kb 字段表、字节布局表、段序列登记表与里程碑各步对账，单元头字段表与 D18 逐项比，连同 harness 挂载透明字段表与 kb 的对账）、欠账与收口组五格（checks-owed.md 与里程碑收口表）、
# 引用与不变量组十三格（搬迁留下的旧路径、引用指得到、不变量条数与字段落点、内联算式左右相等、未实现的不变量的前置、字段频率的大小、kb 新增的「不可达」类断言钉没钉用例、CLAUDE.md 与仓根 README 里登记的能数的数；governance-refs 的判据在 lib-governance-refs.py 文件头）。
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
# gate-cell: third-txn-hooks 删除写出的字节的三处互相挂钩（layout/03 登记表 / 里程碑登记它的那一步 / layout/01 的节与段序列登记表）
# gate-cell: tree-table-reserve 树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）
# gate-cell: field-widths 新池新建文件字段表里写了宽度的行，被指的已定分项正文里写出了这个宽度
# gate-cell: unit-header-fields 单元头字段表（数据单元与树表单元两张）与 D18 逐项比：五元组对已定项 3，偏移与名字对已定项 7 的共同前缀加已定项 18 那一类的偏移表
# gate-cell: mount-transparency-fields 挂载透明字段表的 kb 一侧对账（D13 逐字段表的行名、根记录与系统配置的名与区间）
# gate-cell: table-shape 欠账表两张登记表的行形状（格数对得上表头、欠着的那张里不留写着已还的行、表不被空行断开、两张表都在）
# gate-cell: closeout-collects-open 里程碑收口表收全了文件里点名、还开着的欠账号，行号只许是顺序号
# gate-cell: paid-cited-tests 已还清的欠账行里点名的测试名，仓里还在不在
# gate-cell: audit-contradictions 最近一次总审核登记的文档级矛盾，每一行都要有去向
# gate-cell: closeout-row27-preconditions 里程碑收口表第 27 行那几笔欠账的前置有没有进来（进来了就得补会红的用例）
# gate-cell: experiment-refs kb 与项目规则里引用的实验号都有定义
# gate-cell: decision-refs kb 与项目规则里引用的决策号都有定义
# gate-cell: declared-counts invariants.md 自己那句「现共 N 条在用」与表里条数对得上；欠账表数得出条数
# gate-cell: governance-refs 治理文档、kb 与仓根的 README.md、README.en.md、README.ja.md 里按全名指的门禁与格都在、不用旧编号称呼门禁，治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、`文件「小节」` 在那份文件里找得到；kb 中文页反引号里以仓根目录起头的路径存在、`文件「小节」` 找得到（research/ 下进过 git 的放行，存量登在只缩不涨的 dead-path-lag.tsv 与 dead-section-lag.tsv，照规矩该留旧路径或旧小节的登在可增长的 dead-path-kept.tsv，git 历史命令 -- 之后的与说搬迁的那一行不判；保留表与这两种跳过治理文档的路径同样认）
# gate-cell: invariant-count-elsewhere invariants.md 之外的 kb 文件里写「N 条在用」的地方，N 等于实际在用条数
# gate-cell: citations kb 里承重的外部逐行引用还核得动（调 scripts/kb/verify-citations.sh）
# gate-cell: invariant-anchors 这次改动新写或改写的不变量行，点名它判的字段住在哪条已定分项
# gate-cell: inline-sums 「整数 = 项 + 项 + …」形态的内联算式（kb 正文里，每项是可选标签加一个整数），左右相等
# gate-cell: invariant-prerequisites 不变量表里状态是「未实现」的每一行都写了编号前置（C / D 分项 / E），指得到、而且还没全满足
# gate-cell: field-frequency-sizes 决策与布局正文里「每 X 一份」这种字段频率，X 指得到一条已定条款给出的大小（字面大小要等于已定的某个粒度）
# gate-cell: unreachable-claims kb 里这次改动新增的「不可达 / 走不到 / 触发不了」断言句，同一句带一条仓里在的用例名或写「推的」「没量过」（存量登在只缩不涨的清单里）
# gate-cell: counted-claims CLAUDE.md 与仓根三份 README 这类文档里登记的能数的数（条数、项数），与登记表 counted-claims.tsv 里的命令此刻数出来的相等
#
# 参数解析、样本根的 .gate-cells、逐格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头）。退出码：
#   0  跑到的格没有红
#   1  有格红（退出码既不是 0 也不是 77），红格逐格接它登记的出路；建不了临时目录也退 1
#   2  认不出的选项、两个项目根、格名写错、进不去项目根
#   77 跑到的格全退 77，不记通过；退 77 的格逐格往 $GATE_NOT_RUN_FILE 报本次未跑
#
# 样本（fixtures/doc-registries.sh/，每份样本根的 .gate-cells 点名只跑哪几格；每一格的样本放了什么写在那一格的注释里）：
#   <格名>-red                  字段与布局组最早那九格各一份，判红；expect 另要汇总里「<格名>：红（退出 1）」那一行（后加的四格各有自己的 -red，见下）
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
#   governance-refs-red         只跑 governance-refs：kb 与仓根三份 README（README.md、README.en.md、README.ja.md）里用旧编号称呼门禁、不存在的门禁与格名，逐条点名判红（README.en.md 里反引号点名了格名表里没有的格）
#   governance-refs-green       只跑 governance-refs：按全名指的门禁与格都在，历史节、变更史与记录文件名里的旧编号不判，报扫了几份、全名与格名几处；
#                               kb 页反引号里的 .claude/gate.d/doc-text.sh 与 records/ 下那份记录在样本根下各放了一份，kb 路径判了 2 处、带占位的跳过 1 处
#   governance-refs-old-gate-path-red  只跑 governance-refs：治理文档反引号里写去编号之前的门禁路径 `.claude/gate.d/47-…`，② 交给 ⑤、⑤ 点名 47 号判红（退 1），
#                               不崩成 traceback
#   governance-refs-kb-red      只跑 governance-refs（setup.sh 建 git 仓，样本自己登记 .claude/kb-directories 带一个不变量清单目录）：kb 中文页里改了名的路径红行写出
#                               「改成 <新路径>（改名：改名链 …」、删了没后继的写出「删于 <提交>（标题）…`<提交>^:<路径>`」、research/ 下从没进过 git 的与不变量清单里从没进过 git 的
#                               写「从没进过 git」，四处逐条点名
#   governance-refs-kb-green    只跑 governance-refs（setup.sh 建 git 仓）：research/ 下进过 git 而已删的放行、相对简写跳过 3 处、译文与历史版本节里的不判、
#                               滞后表（基准里就有那一行）登记了的不红，报「判了 3 处（research/ 进过 git 放行 1 处、滞后 1 处）」与基准比没涨
#   governance-refs-kb-gz-green  只跑 governance-refs（setup.sh 建 git 仓）：kb 中文页点名 research/results/ 下的 <名>.out，树里只有 <名>.out.gz 的判过、指得到，
#                               只在历史里有 <名>.out.gz 的照 research/ 进过 git 放行，报「判了 2 处（research/ 进过 git 放行 1 处…）」
#   governance-refs-kb-lag-red  只跑 governance-refs（setup.sh 建 git 仓）：滞后表登记的路径此刻在了、载体已经不引了的两行判「这一行该删」，比基准多的一行判涨，
#                               只有两列、理由不到 4 个字、重复登记的三行各判写得不对，六处逐条点名
#   governance-refs-kb-kept-green  只跑 governance-refs（setup.sh 建 git 仓）：保留表 dead-path-kept.tsv 登记了的不红、`git log … -- <删了的路径>` 里 -- 之后的跳过、
#                               同一行写着后继文件名的搬迁行（`<旧路径>` → `<新文件名>`）跳过，报「判了 2 处（…保留 1 处）」与两类跳过各 1 处
#   governance-refs-kb-kept-red    只跑 governance-refs（setup.sh 建 git 仓）：保留表里滞后表也登了的、理由不到 8 个字的（kb 那一处跟着红）、路径此刻在了的、载体不再引的，
#                               五处逐条点名；登对了的一行不红；红行的出路写「登进 .claude/gate.d/dead-path-kept.tsv」
#   governance-refs-kb-skips-red   只跑 governance-refs（setup.sh 建 git 仓）：git log 命令里 -- 之前的路径照判、同一行没写后继文件名的改名照判、后继与旧路径同名
#                               （fixtures/table.tsv 搬成 table.tsv）的不因旧路径自己算数照判，三处逐条点名；-- 之后的与写着后继文件名的搬迁行各跳过 1 处
#   governance-refs-governance-skips-green  只跑 governance-refs（setup.sh 建 git 仓）：治理文档里登在保留表的不在路径、`git log … -- <删了的路径>` 里 -- 之后的、
#                               同一行写着后继文件名的搬迁行都不红，报治理文档那一行「git 历史命令里 -- 之后 1 处、说搬迁的行 1 处、保留 1 处」
#   governance-refs-kb-sections-red    只跑 governance-refs（setup.sh 建 git 仓）：kb 中文页里两处 `文件「小节」` 找不到，红行各接出路（最像的那一句 / 那份文件里没有），
#                               找得到的一处不红，历史版本节与英语译文里找不到的不判，报「kb 小节：…判了 3 处」
#   governance-refs-kb-sections-green  只跑 governance-refs（setup.sh 建 git 仓）：嵌套引号写成『』的、编号带（简称）的、脚本输出句带占位字母的三处照认，
#                               登在小节滞后表（基准里就有那一行）与保留表小节行的各一处不红，文件找不到的计 1 处，报「判了 5 处（滞后 1 处、保留 1 处）」
#   governance-refs-kb-sections-lag-red  只跑 governance-refs（setup.sh 建 git 仓）：小节滞后表里小节找得到了的、载体不再引的两行判该删，比基准多的一行判涨，
#                               只有两列、理由不到 4 个字、重复登记的三行各判写得不对；保留表小节行找得到了的判该删、与小节滞后表同一对的判两张表只登一张，八处逐条点名
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
#   field-frequency-sizes-unit-registration-removed-red  只跑 field-frequency-sizes：field-frequency-sizes-green 只删 D4 已定项 1 的 DATA_UNIT_BYTES 登记这一处
#                               （C72 的判别力自证照字面：同一份输入由绿转红），逻辑 extent、单元、extent 与字面 32 KiB 五处逐条点名判红
#   field-frequency-sizes-green 只跑 field-frequency-sizes：逻辑 extent、extent、单元、节点、索引节点各读得出大小，字面 32 KiB、带豁免的 512 字节、否定句判绿；
#                               代码围栏里、「## 历史版本」之后、*-history.md 与译本里的错写不扫，报「8 处……豁免 1 处」
#   unreachable-claims-red      只跑 unreachable-claims（setup.sh 建 git 小仓）：基准之后 kb 新增两句断言，一句「不可达」什么都没带、一句「走不到」带的 ghost_case_rejects
#                               在 crates/ 下找不到 #[test] fn 定义，两句逐句点名判红；基准里早就有的一句「触发不了」不是新增，不判
#   unreachable-claims-test-name-green  只跑 unreachable-claims：新增的「走不到」句、表格行里「不可达」那一格各带一条 crates/demo/tests/ 下 #[test] 定义的用例名，判绿；
#                               另一行与存量清单登记的一行逐字相同（挪了位置），按清单放过；.en.md 译文里新增的同类句不判
#   unreachable-claims-inferred-green  只跑 unreachable-claims：新增的「触发不了」句写「推的，没量过」、「不可达」句只写「推的」，判绿
#   unreachable-claims-stock-grown-red  只跑 unreachable-claims：基准里存量清单一行，工作区多加一行（kb 那一句本身是基准里就有的），只缩不涨判红
#   field-refs-blank-red        只跑 field-refs：指向写「无分项」与空着的各一行没登记豁免、根槽那一节一行指未定项、根槽那一节一行「无分项」登了豁免也不认、
#                               豁免表里一条一行都不命中，五处逐条点名判红（C66 的判别力自证：「无分项」由绿转红）；「## 历史版本」之后那张表里的「无分项」不判
#   third-txn-hooks-red         只跑 third-txn-hooks：一行写「第 50 批」（history-kept: 样本里的批次写法是被判的数据）、一行指里程碑没有的 三·9、一行少一格、引 layout/01 不存在的节、引段序列登记表不存在的行、
#                               出处的函数改了名、出处的文件不在，七处逐条点名判红
#   third-txn-hooks-green       只跑 third-txn-hooks：三处互相点名，报「字节那一步 三·6；形态 3 行、行到作业点 6 处、引 layout/01 2 处、段序列出处的文件与函数 4 个」
#   field-widths-red            只跑 field-widths：「设备 ID」宽 1 指一条只说「带」的已定项、「密文校验和」宽 4 指一条只说「32 位」的已定项，各点名判红（C66 的判别力自证）；
#                               宽度写成「14 × 2」与只指未定项的两行不判
#   field-widths-green          只跑 field-widths：同一张表，被指的已定项写出了 1 与 4，报「判了 3 行；…不判的 2 行」
#   unit-header-fields-green    只跑 unit-header-fields：样本 D18 的已定项 3、7、18 与 layout/01 里带标记的数据单元（码 1，有「段」列、夹着两行不是单元头的）与树表单元（码 2，key-width=8）两张表对得上，
#                               报「比了 2 张表、33 个字段的偏移与名字、五元组 5 项」
#   unit-header-fields-five-tuple-removed-red  只跑 unit-header-fields：unit-header-fields-green 只把 D18 已定项 3 的五元组与宽度那一句一起删掉「锚点偏移」（C68 ② 的判别力自证：
#                               同一份输入由绿转红），点名表里「五元组 · 锚点偏移」那一行
#   unit-header-fields-offsets-red  只跑 unit-header-fields：树表单元表少了「出生序号」一行（后面的偏移整体前移 4、合计 127）、数据单元表「五元组 · 对象 ID」的指向没点名 D18 已定项 3，逐条点名判红
#   unit-header-fields-missing-red  只跑 unit-header-fields：没有树表单元那张、一个标记下面紧跟的是一段字、数据单元表一行宽度写「随算法」，逐条点名判红
#   field-refs-unit-header-red  只跑 field-refs：根槽那一节之外一张带 unit-header-fields 标记的表，一行指未定项、一行「无分项」登了豁免，两行都点名判红（单元头字段表照根槽那一节办）；
#                               同一节里不带标记的表指未定项、「无分项」登了豁免的那两行不判
#   mount-transparency-fields-red    只跑 mount-transparency-fields：D13 表删了 harness 引的「卸载记号」、多一行谁都没引的、D22 根记录表 fsid 改名成同宽的「池号」，
#                               三处逐条点名判红（C736 的判别力自证：D13 删一行、kb 字段表改名同宽）
#   mount-transparency-fields-green  只跑 mount-transparency-fields：两边对得上（harness 名字带全角括号注、粗体的 kb 名字都照认，「、」「 / 」拆开的几行合一段），
#                               「POSIX 内容与存在性」登记成不按字节比，报「D13 逐字段表 5 行（harness 引 4 行、不按字节比登记 1 行），根记录 4 段、系统配置 3 段…」
#   field-registry-green 另放 .claude/gate.d/field-pointer-blanks.tsv，把「样本字段丁」那一行「无分项」登成豁免，报「指向列 7 行都写了分项（登记豁免的格式级空白 1 行」
#   按编号取文件的六份（D<n>、E<n> 有中文原文 <号>-<名>.md 与译文 <号>-<名>.en.md / .ja.md，中文这一侧取中文原文，判法 lib-item-ref-status.py 的 numbered_files）：
#   field-refs-translation-green        只跑 field-refs：D9 的两份译文排在中文原文前面、分项表标题是各自的语言；布局表引已定项 10 与已定项 12 的两行算法类型，
#                               中文原文都有，判绿，报「查了 3 处引用」「核了 2 行」
#   field-refs-translation-red          只跑 field-refs：布局表引 D9 已定项 11，中文原文没有、英语译文照中文的写法列着它，判红、点名「指向 D9（09-样本加密.md）的「已定项 11」」
#   format-const-placeholders-translation-green  只跑 format-const-placeholders：占位指 D23 已定项 19，中文原文有、两份译文的分项表标题是各自的语言，判绿
#   format-const-placeholders-translation-red    只跑 format-const-placeholders：占位指 D23 已定项 20，中文原文没有、英语译文照中文的写法列着它，判红、点名「D23 里找不到 已定项 20」
#   audit-contradictions-translation-green       只跑 audit-contradictions：总审核「已改」行点名 D9，引的原句在中文原文里、两份译文里没有，判绿
#   audit-contradictions-translation-red         只跑 audit-contradictions：引的原句中文原文里没有、只在日语译文里有，判红、点名「只在 decisions/09-样本决策.ja.md 里有」
#   数条数只数中文原文的两份（判法 lib-item-ref-status.py 的 same_language_files）：
#   invariant-counts-translation-green  只跑 declared-counts、invariant-count-elsewhere：不变量表的中文原文 3 行、退役 1 行，实验页的英语、日语译文里各抄着两行 `| I-…`；
#                               中文侧「现共 2 条在用」、中文普查页「2 条不变量在用」与英语译文里的「2 条在用」都对得上，判绿，两格的成功行报没数的译文 2 份、4 行
#   invariant-counts-translation-red    同一套表，中文侧写的是把译文表行数进去的「现共 6 条在用」、日语译文里一句「5 条在用」：两格都红，
#                               declared-counts 点名「正文声称在用 6 条，实际 2 条」，invariant-count-elsewhere 点名那两句、都与中文原文数出的 2 条比
#   kb-directories-unreadable-red  只跑 invariant-count-elsewhere、inline-sums：setup.sh 往 .claude/kb-directories 写一个不是 UTF-8 的字节，读登记位的那段 python
#                               （scripts/kb/quote-kb.py 的 kb_candidate_roots）起不来；两格都判红、说登记位读不出
#   读登记位的另四格各一份，都只跑点名的那一格，setup.sh 写同一个坏字节（判法 kb_roots_or_red，六格共用）：
#   audit-contradictions-kb-directories-unreadable-red  只跑 audit-contradictions（records/ 下放一份总审核记录，这一格才有对象）：判红、说登记位读不出
#   declared-counts-kb-directories-unreadable-red  只跑 declared-counts：判红、说登记位读不出，欠账表那一半照数（报「欠检查 1 条、已还清 1 条」）
#   invariant-anchors-kb-directories-unreadable-red  只跑 invariant-anchors（setup.sh 另建一个 git 小仓）：判红、说登记位读不出
#   invariant-prerequisites-kb-directories-unreadable-red  只跑 invariant-prerequisites：判红、说登记位读不出
#   能数的数那一格（counted-claims，判据与各份样本放了什么写在这一格函数上方的注释），四份都只跑 counted-claims，都不是 git 仓：
#   counted-claims-red          登记表九行：数对不上、命中两次、命中 0 次、命令退 3、命令打的不是整数、只有三列、文件不在、正则没有捕获组，八处逐行点名判红；写对的一行不点名
#   counted-claims-green        README.md、README.en.md、NOTES.md 三份四行（两行共用一条命令），判绿，报「核了 4 行、3 份文件、起了 3 条命令」
#   counted-claims-registry-empty   登记表只有注释与空行，退 77、报本次未跑
#   counted-claims-registry-absent  没有登记表，退 77、报本次未跑
#
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/doc-registries.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   DOC_REGISTRIES_BREAK=field-refs  field-refs 格不跑、按判过了记
#       field-refs-red（期望退出 1，实测 0）；field-refs-reservation-owner-red（期望退出 1，实测 0）；field-refs-reservation-owner-green（输出里找不到「新池新建文件的字段表指向都成立…」）；
#       field-registry-green（输出里找不到「新池新建文件的字段表指向都成立（查了 7 处引用）」）；field-refs-translation-red（期望退出 1，实测 0）；
#       field-refs-translation-green（输出里找不到「新池新建文件的字段表指向都成立（查了 3 处引用）」）；field-refs-blank-red（期望退出 1，实测 0）；field-refs-unit-header-red（期望退出 1，实测 0）
#   KB_NUMBERED_FILE_BREAK=first-match  归 lib-item-ref-status.py（设进环境，被各格起的 python 继承）：按编号取文件不分语言，取排序后的第一个命中（取到 <号>-<名>.en.md），
#                               求并集的连译文一起并进来；same_language_files 不筛语言（不变量在用条数把译文里的表行一并数进去）
#       field-refs-translation-green、format-const-placeholders-translation-green（期望退出 0，实测 1）；field-refs-translation-red、format-const-placeholders-translation-red、
#       audit-contradictions-translation-red（期望退出 1，实测 0）；audit-contradictions 那一格的译文绿样本照旧绿（中文原文在并集里，开关下也找得到）；
#       invariant-counts-translation-green（期望退出 0，实测 1）；invariant-counts-translation-red（输出里找不到「不变量表（1 份文件，登记在 .claude/kb-directories） 正文声称在用 6 条，实际 2 条…」）
#   DOC_REGISTRIES_BREAK=reservation-owner-unchecked  field-refs 格里预留字段归属那一半不判（指向存在不存在的那一半照判）
#       field-refs-reservation-owner-red（期望退出 1，实测 0）；field-refs-reservation-owner-green（输出里找不到「核了 3 行预留字段的指向」）；
#       field-refs-translation-green（输出里找不到「核了 2 行预留字段的指向（另有 0 行判不出所属单元类，没判）」）
#   DOC_REGISTRIES_BREAK=field-projection  field-projection 格不跑、按判过了记
#       field-projection-red（期望退出 1，实测 0）；field-registry-green（输出里找不到「被投影的分项，字段表都投影全了…」）
#   DOC_REGISTRIES_BREAK=field-table-sums  field-table-sums 格不跑、按判过了记
#       field-registry-green、field-registry-translation-green（输出里找不到「扫了 10 份决策、4 份字节布局表；表后合计 3 张，表前…」）；field-table-sums-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=first-txn-hooks  first-txn-hooks 格不跑、按判过了记
#       field-registry-green（输出里找不到「互相挂钩…」）；first-txn-hooks-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=admission-terms  admission-terms 格不跑、按判过了记
#       admission-terms-fullwidth-d-green（输出里找不到「准入不等式 3 项逐项有归属」）；admission-terms-red（期望退出 1，实测 0）；field-registry-green、field-registry-translation-green（输出里找不到「准入不等式 3 项逐项有归属」）
#   DOC_REGISTRIES_BREAK=formula-fullwidth-d-unmatched  admission-terms 格抓式子那一行只认半角 (d)
#       admission-terms-fullwidth-d-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=segment-registry  segment-registry 格不跑、按判过了记
#       field-registry-green（输出里找不到「比对了 2 处登记…」）；segment-registry-red、segment-registry-paths-red（期望退出 1，实测 0）；
#       segment-registry-product-absent（期望退出 77，实测 0）
#   DOC_REGISTRIES_BREAK=segment-registry-not-run-as-red  segment-registry 格把脚本的退 77（只能与产物比的那几处产物不在，本次未跑）也记红
#       segment-registry-product-absent（期望退出 77，实测 1）
#   DOC_REGISTRIES_BREAK=format-const-placeholders  format-const-placeholders 格不跑、按判过了记
#       field-registry-green（输出里找不到「2 个占位」）；format-const-placeholders-red、format-const-placeholders-translation-red（期望退出 1，实测 0）；
#       format-const-placeholders-translation-green（输出里找不到「1 个占位」）
#   DOC_REGISTRIES_BREAK=second-txn-hooks  second-txn-hooks 格不跑、按判过了记
#       field-registry-green（输出里找不到「登记表 1 行、行到步 1 处、「写出的字节」点名 3 处、…」）；second-txn-hooks-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=tree-table-reserve  tree-table-reserve 格不跑、按判过了记
#       field-registry-green、field-registry-translation-green（输出里找不到「3 条认购」）；tree-table-reserve-red（期望退出 1，实测 0）；tree-table-reserve-wide-number-green（输出里找不到「3 条认购」）
#   DOC_REGISTRIES_BREAK=owed-number-width-fixed  tree-table-reserve 格抓出处里的编号时位数写死（C 最多三位、D 最多两位）
#       tree-table-reserve-wide-number-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=table-shape  table-shape 格不跑、按判过了记
#       checks-owed-green（输出里找不到「检查了 16 行」）；checks-owed-red（输出里找不到「C4：6 格」）
#   DOC_REGISTRIES_BREAK=closeout-collects-open  closeout-collects-open 格不跑、按判过了记
#       checks-owed-green（输出里找不到「判了 2 份；02-sample.md：全文点名 3 个 C…」）；checks-owed-red（输出里找不到「02-sample.md 点名了这些还开着的欠账号，收口表里…」）
#   DOC_REGISTRIES_BREAK=paid-cited-tests  paid-cited-tests 格不跑、按判过了记
#       checks-owed-green（输出里找不到「查了 1 个标识符」）；checks-owed-red（输出里找不到「C902 引的 `the_sample_test_that_…」）
#   DOC_REGISTRIES_BREAK=paid-cited-batch-names-dropped  paid-cited-tests 格一次交给 grep 的名单少最后一个名字（那个名字当成仓里找不到）
#       checks-owed-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=citations-exit-ignored  citations 格读到复核脚本的退出码也不判，退非 0 当过了
#       citations-fake-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=defined-numbers-empty  experiment-refs、decision-refs 取到的「已定义的编号」集合是空的，被引用的编号一个都找不到定义
#       references-green、decision-refs-not-decisions-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=audit-contradictions  audit-contradictions 格不跑、按判过了记
#       checks-owed-green、audit-contradictions-translation-green（输出里找不到「每一行都有去向」）；checks-owed-red（输出里找不到「有行没有去向」）；
#       audit-contradictions-translation-red、audit-contradictions-unnamed-translation-red、audit-contradictions-kb-directories-unreadable-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=read-translations  field-table-sums、admission-terms、tree-table-reserve 连译文一起读，audit-contradictions 没点名文件的行连译文一起找
#       field-registry-translation-green（期望退出 0，实测 1）；audit-contradictions-unnamed-translation-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=invariant-single-file  audit-contradictions 格不变量类文件与 README.md 不进 kb 正文，I- 号一律指 .claude/kb/invariants.md
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
#       invariant-counts-translation-green（输出里找不到「不变量条数一致：正文声称 2 条在用，表里在用 2 条…」）；invariant-counts-translation-red（输出里找不到「不变量表（1 份文件，登记在 .claude/kb-dire…」）；
#       declared-counts-kb-directories-unreadable-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=governance-refs  governance-refs 格不跑、按判过了记
#       governance-refs-green、references-green（输出里找不到「治理文档与 kb 里的门禁号、门禁全名与格名、路径与「小节」…」）；governance-refs-red（期望退出 1，实测 0）；
#       references-red（输出里找不到「治理文档与 kb 里有指不到的指向（上面逐条列出）」）；
#       governance-refs-old-gate-path-red、governance-refs-kb-red、governance-refs-kb-lag-red、governance-refs-kb-kept-red、governance-refs-kb-skips-red（期望退出 1，实测 0）；
#       governance-refs-kb-sections-red、governance-refs-kb-sections-lag-red（期望退出 1，实测 0）；
#       governance-refs-kb-green、governance-refs-kb-kept-green、governance-refs-kb-gz-green（输出里找不到「  kb 路径：扫 1 份 kb 中文页…」）；
#       governance-refs-governance-skips-green（输出里找不到「  扫 1 份治理文档…」）；governance-refs-kb-sections-green（输出里找不到「  kb 小节：扫 1 份 kb 中文页…」）
#   DOC_REGISTRIES_BREAK=invariant-count-elsewhere  invariant-count-elsewhere 格不跑、按判过了记
#       references-green（输出里找不到「不变量条数跨文件一致：实际在用 1 条，扫了 7 份 kb…」）；references-red（输出里找不到「写「99 条在用」，而 不变量表（1 份文件，登记在 .cl…」）；
#       kb-directories-unreadable-red（输出里找不到「invariant-count-elsewhere 没判：kb 目录登记位读不出」）；
#       invariant-counts-translation-green（输出里找不到「不变量条数跨文件一致：实际在用 2 条，扫了 5 份 kb…」）；invariant-counts-translation-red（输出里找不到「.claude/kb/survey.ja.md:3 写「5 条在用」…」）
#   DOC_REGISTRIES_BREAK=citations  citations 格不跑、按判过了记
#       references-green（输出里找不到「样本替身：3 条命中，0 条未命中」）；references-red（输出里找不到「找不到 scripts/verify-ci…」）
#   DOC_REGISTRIES_BREAK=invariant-anchors  invariant-anchors 格不跑、按判过了记
#       references-green（输出里找不到「这次改动新写或改写的 1 行不变量都点名了已定分项」）；references-red（输出里找不到「这次改动新写或改写的 1 行不变量，没有点名它判的字段住在哪…」）；
#       invariant-anchors-kb-directories-unreadable-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=inline-sums  inline-sums 格不跑、按判过了记
#       inline-sums-green（输出里找不到「kb 正文里的内联算式左右相等：扫 2 份 kb 文件…」）；inline-sums-red（期望退出 1，实测 0）；
#       kb-directories-unreadable-red（输出里找不到「inline-sums 没判：kb 目录登记位读不出」）
#   DOC_REGISTRIES_BREAK=kb-roots-failure-ignored  读登记位的六格（kb_roots_or_red）都不看读登记位那段 python 的退出码（读不出就当候选目录是空的）
#       kb-directories-unreadable-red（期望退出 1，实测 77）；invariant-anchors-kb-directories-unreadable-red、invariant-prerequisites-kb-directories-unreadable-red（期望退出 1，实测 77）；
#       audit-contradictions-kb-directories-unreadable-red（输出里找不到「audit-contradictions 没判：kb 目录登记位读不出」）；
#       declared-counts-kb-directories-unreadable-red（输出里找不到「declared-counts 没判：kb 目录登记位读不出」）
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
#       invariant-prerequisites-red、invariant-prerequisites-kb-directories-unreadable-red（期望退出 1，实测 0）；invariant-prerequisites-green（输出里找不到「未实现的不变量都写了指得到、还没全满足的编号前置…」）
#   DOC_REGISTRIES_BREAK=prerequisites-satisfied-unchecked  invariant-prerequisites 格里前置全都已满足也不判（C446 判别力自证的前一半）
#       invariant-prerequisites-red（输出里找不到「不变量表里 8 处未实现的不变量前置不成立…」，那三行 I-1.4、I-1.6、I-1.8 不报）
#   DOC_REGISTRIES_BREAK=prerequisites-existence-unchecked  invariant-prerequisites 格里前置指不到也不判（C446 判别力自证的后一半）
#       invariant-prerequisites-red（输出里找不到「不变量表里 8 处未实现的不变量前置不成立…」，那三行 I-1.5、I-1.7、I-1.9 不报）
#   DOC_REGISTRIES_BREAK=field-frequency-sizes  field-frequency-sizes 格不跑、按判过了记
#       field-frequency-sizes-red、field-frequency-sizes-registration-red、field-frequency-sizes-unit-registration-removed-red（期望退出 1，实测 0）；
#       field-frequency-sizes-green（输出里找不到「字段频率「每 X 一份」的 X 都指得到…」）
#   DOC_REGISTRIES_BREAK=field-frequency-mark-anywhere  field-frequency-sizes 格认住在哪一节的 format-const 登记（不要求住在已定项小节）
#       field-frequency-sizes-registration-red（输出里找不到「7 处字段频率的大小指不到已定条款…」，单元那两行与字面 32 KiB 那一行不报）
#   DOC_REGISTRIES_BREAK=field-frequency-literal-unchecked  field-frequency-sizes 格不判字面大小
#       field-frequency-sizes-red（输出里找不到「10 处字段频率的大小指不到已定条款…」）；field-frequency-sizes-registration-red（输出里找不到「7 处字段频率的大小…」）；
#       field-frequency-sizes-unit-registration-removed-red（输出里找不到「5 处字段频率的大小指不到已定条款…」）
#   DOC_REGISTRIES_BREAK=field-frequency-markers-ignored  field-frequency-sizes 格不认 not-field-frequency 豁免标记（标记本身也不判）
#       field-frequency-sizes-green（期望退出 0，实测 1）；field-frequency-sizes-red（输出里找不到「10 处字段频率的大小指不到已定条款…」）；
#       field-frequency-sizes-registration-red（输出里找不到「7 处字段频率的大小…」）；field-frequency-sizes-unit-registration-removed-red（输出里找不到「5 处字段频率的大小…」）
#   DOC_REGISTRIES_BREAK=pointer-blanks-unchecked  field-refs 格里指向列空着、根槽那一节指未定项那一半不判（C66 收紧的那一半）
#       field-refs-blank-red（期望退出 1，实测 0）；field-refs-unit-header-red（期望退出 1，实测 0）；field-registry-green（输出里找不到「指向列 7 行都写了分项（登记豁免的格式级空白 1 行」）
#   DOC_REGISTRIES_BREAK=third-txn-hooks  third-txn-hooks 格不跑、按判过了记
#       third-txn-hooks-red（期望退出 1，实测 0）；third-txn-hooks-green（输出里找不到「删除写出的字节三处互相挂钩（字节那一步 三·6；…」）
#   DOC_REGISTRIES_BREAK=third-txn-bytes-step-unchecked  third-txn-hooks 格里「一、」表的行不点名字节那一步也放过
#       third-txn-hooks-red（输出里找不到「删除写出的字节的三处挂钩对不上 7 处」）
#   DOC_REGISTRIES_BREAK=field-widths  field-widths 格不跑、按判过了记
#       field-widths-red（期望退出 1，实测 0）；field-widths-green（输出里找不到「新池新建文件字段表写了宽度的行，宽度都写在被指的已定分项里…」）
#   DOC_REGISTRIES_BREAK=field-widths-numbering-counted  field-widths 格把索引行的编号格、小节标题与编号写法里的数也算成写出了宽度
#       field-widths-red（输出里找不到「「设备 ID（8）」宽 1，指向的 D419 已定项 1 正文里没写出 1」）
#   DOC_REGISTRIES_BREAK=unit-header-fields  unit-header-fields 格不跑、按判过了记
#       unit-header-fields-five-tuple-removed-red、unit-header-fields-offsets-red、unit-header-fields-missing-red（期望退出 1，实测 0）；
#       unit-header-fields-green（输出里找不到「单元头字段表与 D18 对得上（比了 2 张表、33 个字段的偏移与名字、五元组 5 项）」）
#   DOC_REGISTRIES_BREAK=five-tuple-unchecked  unit-header-fields 格里五元组那一半（①）不判
#       unit-header-fields-five-tuple-removed-red（期望退出 1，实测 0）；unit-header-fields-offsets-red（输出里找不到「「五元组 · 对象 ID」的指向没点名 D18 已定项 3」）；
#       unit-header-fields-green（输出里找不到「…、五元组 5 项）」）
#   DOC_REGISTRIES_BREAK=header-offsets-unchecked  unit-header-fields 格里偏移与名字那一半（②）不判
#       unit-header-fields-offsets-red（输出里找不到「从偏移 88 起的是「载荷 CRC（偏移 92）」，D18 定的是「出生序号」」）；unit-header-fields-green（输出里找不到「…、33 个字段的偏移与名字…」）
#   DOC_REGISTRIES_BREAK=unit-header-pointer-loose  field-refs 格不把带 unit-header-fields 标记的表照根槽那一节办（指未定项、登了豁免的「无分项」都放过）
#       field-refs-unit-header-red（期望退出 1，实测 0）
#   DOC_REGISTRIES_BREAK=mount-transparency-fields  mount-transparency-fields 格不跑、按判过了记
#       mount-transparency-fields-red（期望退出 1，实测 0）；mount-transparency-fields-green（输出里找不到「挂载透明字段表与 kb 对得上：…」）
#   DOC_REGISTRIES_BREAK=mount-transparency-names-unchecked  mount-transparency-fields 格的 ② 只比区间、不比名字（改名同宽放过）
#       mount-transparency-fields-red（输出里找不到「harness 根记录字段表「fsid」4..20 与 …名字对不上…」）
#   DOC_REGISTRIES_BREAK=unreachable-claims  unreachable-claims 格不跑、按判过了记
#       unreachable-claims-red、unreachable-claims-stock-grown-red（期望退出 1，实测 0）；
#       unreachable-claims-test-name-green（输出里找不到「✓ kb 这次改动新增的 3 行里 2 句「不可达 / 走不到 / 触发不了」断言都带了仓里在的用例名或写了推的…」）；
#       unreachable-claims-inferred-green（输出里找不到「✓ kb 这次改动新增的 2 行里 2 句「不可达 / 走不到 / 触发不了」断言都带了仓里在的用例名或写了推的…」）
#   DOC_REGISTRIES_BREAK=unreachable-claims-names-unresolved  unreachable-claims 格里像用例名的标识符（小写、带下划线）就算用例名，不核 crates/、scripts/ 下有没有 #[test] fn 定义
#       unreachable-claims-red（输出里找不到「kb 这次改动新增的 2 句「不可达 / 走不到 / 触发不了」断言既没带仓里在的用例名…」）
#   DOC_REGISTRIES_BREAK=unreachable-claims-names-ignored  unreachable-claims 格不认用例名（只认「推的」「没量过」）
#       unreachable-claims-test-name-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=unreachable-claims-inferred-ignored  unreachable-claims 格不认「推的」「没量过」（只认用例名）
#       unreachable-claims-inferred-green（期望退出 0，实测 1）
#   DOC_REGISTRIES_BREAK=unreachable-claims-stock-growth-unchecked  unreachable-claims 格不拿存量清单和基准那一版比：清单涨了不红
#       unreachable-claims-stock-grown-red（期望退出 1，实测 77：kb 这次一句断言都没新增）；
#       unreachable-claims-test-name-green（输出里找不到「存量清单 .claude/gate.d/unreachable-claims-stock.tsv 1 行（与基准 HEAD 那一版比，没涨；…」）
#   DOC_REGISTRIES_BREAK=counted-claims  counted-claims 格不跑、按判过了记
#       counted-claims-red（期望退出 1，实测 0）；counted-claims-green（输出里找不到「✓ 登记的能数的数都与现数相等（核了 4 行、3 份文件、起了 3 条命令…」）；
#       counted-claims-registry-empty、counted-claims-registry-absent（期望退出 77，实测 0）
#   DOC_REGISTRIES_BREAK=counted-claims-mismatch-unchecked  counted-claims 格里两个数不等不判
#       counted-claims-red（输出里找不到「NOTES.md:3 正文写 7，现数 9；出路：正文改成 9，或登记表的命令改对」）
#   DOC_REGISTRIES_BREAK=counted-claims-match-count-unchecked  counted-claims 格里正则命中几次不判：取第一处，一处都没有就跳过那一行
#       counted-claims-red（输出里找不到「的定位正则在 NOTES.md 里命中 2 次（第 4 行、第 8 行），要恰好一次…」）
#   DOC_REGISTRIES_BREAK=counted-claims-command-exit-ignored  counted-claims 格里数数命令的退出码不看，只照输出判
#       counted-claims-red（输出里找不到「NOTES.md:5 数的命令退 3（样本：故意退 3）…」）
#   GOVERNANCE_REFS_BREAK=old-numbers-ignored  lib-governance-refs.py 的 ⑤ 整条不判（用旧编号称呼门禁）
#       governance-refs-red（输出里找不到「.claude/kb/tooling.md:3: 用旧编号称…」）；governance-refs-old-gate-path-red（期望退出 1，实测 0）
#   GOVERNANCE_REFS_BREAK=names-ignored  lib-governance-refs.py 的 ④ 整条不判（不是门禁目录里的门禁、格名表里没有）
#       governance-refs-green（输出里找不到「扫 5 份现状文字…」）；governance-refs-red（输出里找不到「.claude/agents/sample.md:7: 「门…」）；
#       governance-refs-kb-green（输出里找不到「  共查 5 项（…」，定义里那一处全名与格名不数了）；governance-refs-kb-kept-green（输出里找不到「  共查 4 项（…」，同上）；
#       governance-refs-governance-skips-green（输出里找不到「  共查 4 项（…」，同上）；governance-refs-kb-sections-green（输出里找不到「  共查 12 项（…」，同上）
#   GOVERNANCE_REFS_BREAK=fences-scanned  lib-governance-refs.py 把代码围栏里的行也当现状文字扫
#       governance-refs-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=history-scanned  lib-governance-refs.py 把「## 历史版本」一节与变更史也扫
#       governance-refs-green（期望退出 0，实测 1）；governance-refs-red（输出里找不到「扫 5 份现状文字…」）；
#       governance-refs-kb-green（期望退出 0，实测 1：历史版本节里那一处路径也判了）；
#       governance-refs-kb-sections-red（输出里找不到「  kb 小节：扫 1 份 kb 中文页，判了 3 处…」：历史版本节里那一处小节也判了）
#   GOVERNANCE_REFS_BREAK=readme-skipped  lib-governance-refs.py 不把仓根的三份 README（README.md、README.en.md、README.ja.md）算进现状文字
#       governance-refs-green（输出里找不到「扫 5 份现状文字…」）；governance-refs-red（输出里找不到「README.md:3: 用旧编号称呼门禁」「README.ja.md:3: 用旧编号称呼门禁」「README.en.md:3: …」）
#   GOVERNANCE_REFS_BREAK=readme-old-name  lib-governance-refs.py 的 README_PATTERNS 退回 README.zh.md（仓根没有这一份，README.en.md 扫不到）
#       governance-refs-green（输出里找不到「扫 5 份现状文字…」，少一份）；governance-refs-red（输出里找不到「README.en.md:3: …」）
#   GOVERNANCE_REFS_BREAK=count-line-missing  lib-governance-refs.py 不打末行「共查 T 项（…）」：governance-refs 格退 0 却取不到查了多少项，判红
#       governance-refs-green、governance-refs-kb-green、governance-refs-kb-kept-green、references-green（期望退出 0，实测 1）；
#       governance-refs-governance-skips-green、governance-refs-kb-sections-green、governance-refs-kb-gz-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=count-zero  lib-governance-refs.py 把末行的 T 打成 0：governance-refs 格判红（扫到 0 项不是通过）
#       governance-refs-green、governance-refs-kb-green、governance-refs-kb-kept-green、references-green（期望退出 0，实测 1）；
#       governance-refs-governance-skips-green、governance-refs-kb-sections-green、governance-refs-kb-gz-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=kb-unscanned  lib-governance-refs.py 的 ② 与 ③ 都不扫 kb（kb_carriers 一份都不交）
#       governance-refs-kb-red、governance-refs-kb-skips-red、governance-refs-kb-sections-red（期望退出 1，实测 0）；governance-refs-kb-green（期望退出 0，实测 1：滞后表那一行成了「不引了」）；
#       governance-refs-kb-kept-green（期望退出 0，实测 1：保留表那一行成了「不引了」）；governance-refs-kb-gz-green（期望退出 0，实测 1：kb 一处都不判，共查 0 项）；
#       governance-refs-kb-sections-green（期望退出 0，实测 1：小节滞后表与保留表那两行成了该删）；governance-refs-kb-sections-lag-red（输出里找不到「  kb 小节：扫 1 份 kb 中文页，判了 3 处…」）；
#       governance-refs-green（输出里找不到「  kb 路径：扫 1 份 kb 中文页…」）；governance-refs-kb-lag-red（输出里找不到「  kb 路径：扫 1 份 kb 中文页，判了 3 处…」）；
#       governance-refs-kb-kept-red（输出里找不到「.claude/kb/tooling.md:5: `scripts/kb/short-reason.py` 里的…」）
#   GOVERNANCE_REFS_BREAK=research-not-passed  lib-governance-refs.py 把 research/ 下进过 git 的也判红（不放行归档）
#       governance-refs-kb-green、governance-refs-kb-gz-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=gz-storage-ignored  lib-governance-refs.py 的 kb 路径只查点名的原样（research/results/ 下的 <名>.out 不认 <名>.out.gz 存放）
#       governance-refs-kb-gz-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=successor-dropped  lib-governance-refs.py 的红行不带后继结论
#       governance-refs-kb-red（输出里找不到「.claude/kb/tooling.md:3: `scripts/kb/old-name.py` 里的 …改成 scripts/kb/new-name.py（改名：改名链」）；
#       governance-refs-kb-kept-red（输出里找不到「….py 在仓里不存在：从没进过 git」）；governance-refs-kb-skips-red（输出里找不到「…改成 scripts/kb/moved-here.py（改名：改名链」）
#   GOVERNANCE_REFS_BREAK=lag-unread  lib-governance-refs.py 不读滞后表 dead-path-lag.tsv（登记了的照判红，表写没写对、该不该删、涨没涨、与保留表重没重都不判）
#       governance-refs-kb-green（期望退出 0，实测 1）；governance-refs-kb-lag-red（输出里找不到「.claude/gate.d/dead-path-lag.tsv:6: 有 2 列…」）；
#       governance-refs-green、governance-refs-kb-kept-green（输出里找不到「…滞后表 0 行（没有滞后表）」）；
#       governance-refs-kb-kept-red（输出里找不到「.claude/gate.d/dead-path-kept.tsv:3: 与滞后表…第 2 行登记的是同一对…」）
#   GOVERNANCE_REFS_BREAK=lag-growth-unjudged  lib-governance-refs.py 的滞后表与小节滞后表都不与 gate 基准那一版比（只缩不涨不判）
#       governance-refs-kb-lag-red（输出里找不到「.claude/gate.d/dead-path-lag.tsv:5: 比 gate 基准 HEAD 那一版多这一行…」）；
#       governance-refs-kb-green（输出里找不到「…与 gate 基准 HEAD 那一版比没涨（基准 1 行，此刻 1 行）」）；
#       governance-refs-kb-sections-lag-red（输出里找不到「.claude/gate.d/dead-section-lag.tsv:5: 比 gate 基准 HEAD 那一版多这一行…」）；
#       governance-refs-kb-sections-green（输出里找不到「…小节滞后表 1 行（与 gate 基准 HEAD 那一版比没涨（基准 1 行，此刻 1 行））」）
#   GOVERNANCE_REFS_BREAK=kept-unread  lib-governance-refs.py 不读保留表 dead-path-kept.tsv（登记了的照判红，表的几种红都不判）
#       governance-refs-kb-kept-green、governance-refs-governance-skips-green、governance-refs-kb-sections-green（期望退出 0，实测 1）；
#       governance-refs-kb-kept-red（输出里找不到「.claude/gate.d/dead-path-kept.tsv:3: 与滞后表…」）；
#       governance-refs-kb-sections-lag-red（输出里找不到「.claude/gate.d/dead-path-kept.tsv:2: …已经没有找不到的…「样例规则」」）；
#       governance-refs-green、governance-refs-kb-green（输出里找不到「…保留表 0 行（没有保留表）」）
#   GOVERNANCE_REFS_BREAK=kept-reds-unjudged  lib-governance-refs.py 的保留表三种红（登记的路径在了、载体不再引它、理由不到 8 个字；小节行的该删）不判
#       governance-refs-kb-kept-red（输出里找不到「.claude/gate.d/dead-path-kept.tsv:4: 第三列要以 # 起头…」「…:5: …此刻在了：建好了或回来了，删这一行」「…:6: …已经不引…」）；
#       governance-refs-kb-sections-lag-red（输出里找不到「.claude/gate.d/dead-path-kept.tsv:2: …已经没有找不到的…「样例规则」」）
#   GOVERNANCE_REFS_BREAK=git-history-judged  lib-governance-refs.py 的 git log / show / blame / diff 命令里 -- 之后的路径照判（不跳，治理文档与 kb 两半）
#       governance-refs-kb-kept-green、governance-refs-governance-skips-green（期望退出 0，实测 1）；governance-refs-kb-skips-red（输出里找不到「  kb 路径：…git 历史命令里 -- 之后 1 处…」）
#   GOVERNANCE_REFS_BREAK=move-line-judged  lib-governance-refs.py 的说搬迁的那一行照判（同一行写着后继文件名也不跳，治理文档与 kb 两半）
#       governance-refs-kb-kept-green、governance-refs-governance-skips-green（期望退出 0，实测 1）；governance-refs-kb-skips-red（输出里找不到「  kb 路径：…说搬迁的行 1 处」）
#   GOVERNANCE_REFS_BREAK=kb-sections-unscanned  lib-governance-refs.py 的 ③ 不扫 kb（kb 中文页里的 `文件「小节」` 一处都不判）
#       governance-refs-kb-sections-red（期望退出 1，实测 0）；governance-refs-kb-sections-green（期望退出 0，实测 1：小节滞后表与保留表那两行成了该删）；
#       governance-refs-green（输出里找不到「  kb 小节：扫 1 份 kb 中文页，判了 1 处…」）；governance-refs-kb-sections-lag-red（输出里找不到「  kb 小节：扫 1 份 kb 中文页，判了 3 处…」）
#   GOVERNANCE_REFS_BREAK=nearest-dropped  lib-governance-refs.py 的 ③ 红行不接出路（最像的标题或句子、那份文件里没有）
#       governance-refs-kb-sections-red（输出里找不到「….md「跑批脚本不许吞掉单轮失败」…最像的是那份文件第 5 行的句子…」「…那份文件里没有，改写那一句或删去」）
#   GOVERNANCE_REFS_BREAK=nested-quotes-kept  lib-governance-refs.py 的 ③ 不把『』当装饰去掉
#       governance-refs-kb-sections-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=short-names-kept  lib-governance-refs.py 的 ③ 不去掉编号后面的（简称）
#       governance-refs-kb-sections-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=placeholders-literal  lib-governance-refs.py 的 ③ 把脚本输出句里的占位字母照字面比
#       governance-refs-kb-sections-green（期望退出 0，实测 1）
#   GOVERNANCE_REFS_BREAK=hyphenless-names-skipped  lib-governance-refs.py 的门禁全名与「X 的 Y 格」只认带连字符的名字（并包之后不带连字符的 lkmm 不数、不判）
#       governance-refs-green（输出里找不到「扫 5 份现状文字…」）；governance-refs-red（输出里找不到「README.md:4: 「lkmm 的 no-such-cell 格」…」）
#   GOVERNANCE_REFS_BREAK=section-lag-unread  lib-governance-refs.py 不读小节滞后表 dead-section-lag.tsv（登记了的照判红，表写没写对、该不该删、涨没涨都不判）
#       governance-refs-kb-sections-green（期望退出 0，实测 1）；governance-refs-kb-sections-lag-red（输出里找不到「.claude/gate.d/dead-section-lag.tsv:3: …」）；
#       governance-refs-green、governance-refs-kb-sections-red（输出里找不到「…小节滞后表 0 行（没有小节滞后表）」）
#   STAGE_CELLS_BREAK=red-swallowed  共用库：红格按绿记、整道退 0
#       admission-terms-red、checks-owed-red、decision-refs-not-decisions-red、field-projection-red、field-refs-red、field-refs-reservation-owner-red、field-table-sums-red、first-txn-hooks-red、
#       format-const-placeholders-red、governance-refs-red、inline-sums-red、references-red、references-unreadable-red、second-txn-hooks-red、segment-registry-red、segment-registry-paths-red、
#       tree-table-reserve-red、invariant-prerequisites-red、field-frequency-sizes-red、field-frequency-sizes-registration-red、
#       field-refs-translation-red、format-const-placeholders-translation-red、audit-contradictions-translation-red、invariant-counts-translation-red、
#       audit-contradictions-unnamed-translation-red、field-refs-blank-red、third-txn-hooks-red、field-widths-red、mount-transparency-fields-red、
#       field-frequency-sizes-unit-registration-removed-red、kb-directories-unreadable-red、audit-contradictions-kb-directories-unreadable-red、declared-counts-kb-directories-unreadable-red、
#       invariant-anchors-kb-directories-unreadable-red、invariant-prerequisites-kb-directories-unreadable-red、unreachable-claims-red、unreachable-claims-stock-grown-red、counted-claims-red、
#       governance-refs-old-gate-path-red、governance-refs-kb-red、governance-refs-kb-lag-red、governance-refs-kb-kept-red、governance-refs-kb-skips-red、
#       governance-refs-kb-sections-red、governance-refs-kb-sections-lag-red、field-refs-unit-header-red、unit-header-fields-five-tuple-removed-red、unit-header-fields-missing-red、
#       unit-header-fields-offsets-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=check-ignored  共用库：点名选格不起作用（--check 与样本根的 .gate-cells 共用一段），照跑全部登记的格
#       admission-terms-fullwidth-d-green、checks-owed-green、decision-refs-not-decisions-green、field-frequency-sizes-green、field-refs-reservation-owner-green、field-registry-green、
#       field-registry-translation-green、governance-refs-green、inline-sums-green、invariant-prerequisites-green、references-green、tree-table-reserve-wide-number-green、
#       field-refs-translation-green、format-const-placeholders-translation-green、audit-contradictions-translation-green、invariant-counts-translation-green、
#       third-txn-hooks-green、field-widths-green、mount-transparency-fields-green、unreachable-claims-test-name-green、unreachable-claims-inferred-green、
#       governance-refs-kb-green、governance-refs-kb-kept-green、governance-refs-governance-skips-green、governance-refs-kb-sections-green、governance-refs-kb-gz-green、unit-header-fields-green（期望退出 0，实测 1）；
#       segment-registry-product-absent（期望退出 77，实测 1）；
#       admission-terms-red、field-projection-red、field-refs-red、field-refs-reservation-owner-red、field-table-sums-red、first-txn-hooks-red、format-const-placeholders-red、
#       inline-sums-red、second-txn-hooks-red、segment-registry-red、segment-registry-paths-red、tree-table-reserve-red、field-refs-translation-red、
#       format-const-placeholders-translation-red、audit-contradictions-translation-red、audit-contradictions-unnamed-translation-red、field-refs-blank-red（输出里找不到「doc-registries.sh：1 格里 1 格红…」）；
#       decision-refs-not-decisions-red（输出里找不到「样本只跑 .gate-cells 点名的格：decision-refs」）；invariant-prerequisites-red（输出里找不到「样本只跑 .gate-cells 点名的格：invariant-prerequisites」）；
#       field-frequency-sizes-red、field-frequency-sizes-registration-red、field-frequency-sizes-unit-registration-removed-red（输出里找不到「样本只跑 .gate-cells 点名的格：field-frequency-sizes」）；
#       third-txn-hooks-red、field-widths-red、mount-transparency-fields-red、unreachable-claims-red、unreachable-claims-stock-grown-red（输出里找不到「样本只跑 .gate-cells 点名的格：<那一格>」）；
#       checks-owed-red（输出里找不到「doc-registries.sh：5 格里 5 格红…」）；references-red（输出里找不到「doc-registries.sh：7 格里 7 格红…」）；
#       references-unreadable-red（输出里找不到「doc-registries.sh：2 格里 2 格红…」）；invariant-counts-translation-red、kb-directories-unreadable-red（输出里找不到「doc-registries.sh：2 格里 2 格红…」）；
#       audit-contradictions-kb-directories-unreadable-red、declared-counts-kb-directories-unreadable-red、invariant-anchors-kb-directories-unreadable-red、
#       invariant-prerequisites-kb-directories-unreadable-red（输出里找不到「doc-registries.sh：1 格里 1 格红…」）；
#       counted-claims-green（期望退出 0，实测 1）；counted-claims-registry-empty、counted-claims-registry-absent（期望退出 77，实测 1）；counted-claims-red（输出里找不到「样本只跑 .gate-cells 点名的格：counted-claims」）；
#       field-refs-unit-header-red（输出里找不到「样本只跑 .gate-cells 点名的格：field-refs」）；unit-header-fields-five-tuple-removed-red、unit-header-fields-missing-red、
#       unit-header-fields-offsets-red（输出里找不到「样本只跑 .gate-cells 点名的格：unit-header-fields」）；
#       以 governance-refs 起头的八份红样本照判得对（它们的 expect 不写汇总行，照跑全部格时 governance-refs 那几行照样在；
#       名字不在这一条里逐个写，开关复跑按名字认点名）
#   LIB_CHANGED_PATHS_BREAK=untracked  scripts/gate/changed-paths.sh 丢掉未跟踪文件：invariant-anchors 看不见未跟踪的清单
#       （references 那一份红样本在这个开关下照判得对，这一条在样本上没有判错的，证红欠着）
#
#   bash .claude/gate.d/doc-registries.sh [项目根]                                   三十一格都跑
#   bash .claude/gate.d/doc-registries.sh --list                                     逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-registries.sh --check <格名>[,<格名>…] [项目根]          只跑点名的格
#
# 格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的几份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析与汇总。格的出路里写的「doc-registries.sh 里这一格函数上方的注释」，就在下面对应的那份库里（格在哪份库按格名列在下面）：
# 库 lib/doc-registries-field-and-layout-cells.sh：field-refs、field-projection、field-table-sums、first-txn-hooks、admission-terms、segment-registry
# 库 lib/doc-registries-field-and-layout-cells.sh：format-const-placeholders、second-txn-hooks、third-txn-hooks、tree-table-reserve、field-widths
# 库 lib/doc-registries-unit-header-and-debt-cells.sh：unit-header-fields、mount-transparency-fields、table-shape、closeout-collects-open、paid-cited-tests
# 库 lib/doc-registries-unit-header-and-debt-cells.sh：audit-contradictions、closeout-row27-preconditions
# 库 lib/doc-registries-reference-and-invariant-cells.sh：experiment-refs、decision-refs、declared-counts、governance-refs、invariant-count-elsewhere
# 库 lib/doc-registries-reference-and-invariant-cells.sh：citations、invariant-anchors、inline-sums、invariant-prerequisites、field-frequency-sizes
# 库 lib/doc-registries-reference-and-invariant-cells.sh：unreachable-claims、counted-claims
# 弄坏开关的读法在入口的赋值与格函数里，格函数在 lib/ 下的库里；开关表仍在上面文件头。下面每个开关变量在哪份库的代码里被读，按库列在这里：
# 库 lib/doc-registries-field-and-layout-cells.sh：DOC_REGISTRIES_BREAK
# 库 lib/doc-registries-reference-and-invariant-cells.sh：DOC_REGISTRIES_BREAK
# 入口的代码里也读的：DOC_REGISTRIES_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：GOVERNANCE_REFS_BREAK、KB_NUMBERED_FILE_BREAK、LIB_CHANGED_PATHS_BREAK
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ doc-registries.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi

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
# unreachable-claims 格（判据在这一格函数上方的注释）：认的断言词、认的「推的」类标记（都空白分隔；加词之前先量真仓 kb 会多红几行），存量清单（只缩不涨）
UNREACHABLE_CLAIM_WORDS="不可达 走不到 触发不了"
UNREACHABLE_INFERRED_MARKERS="推的 没量过"
UNREACHABLE_STOCK_TABLE=".claude/gate.d/unreachable-claims-stock.tsv"
# 存量清单与基准那一版比（只缩不涨）：取基准那一版、切表行的读法只在这一份（与 doc-process-records、code-source-discipline 几处同一份）
LAG_BASELINE_LIBRARY="$STAGE_DIRECTORY/lib-lag-baseline.py"
# counted-claims 格（判据在这一格函数上方的注释）：登记表（被判的根里那一份）与每条数数命令的限时
COUNTED_CLAIMS_REGISTRY=".claude/gate.d/counted-claims.tsv"
COUNTED_CLAIMS_COMMAND_SECONDS=120
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
stage_cell third-txn-hooks judge_crash_verification_rebuild_hooks "删除写出的字节的三处互相挂钩（layout/03 登记表 / 里程碑登记它的那一步 / layout/01 的节与段序列登记表）" \
  "layout/03 开头写明它是里程碑哪一步的登记表、那一步点名 \`layout/03-third-txn.md\`；「一、」表每一行的「里程碑那一步」点名那一步；引 layout/01 的节名与段序列登记表行名逐字抄；「四、」出处的文件与函数要在仓里（判据在这一格函数上方的注释）"
stage_cell tree-table-reserve judge_tree_table_reserve "树表条目预留字节的认购合计（认购不许超过预留，每条要指得到一个编号）" \
  "认购表在 .claude/kb/decisions/08-核心索引结构.md 已定项 11：合计超了先去掉一个认购者或压小宽度，别直接改 total；出处补一个还开着的欠账号或已定分项"
stage_cell field-widths judge_field_widths "新池新建文件字段表里写了宽度的行，被指的已定分项正文里写出了这个宽度" \
  "让被指的已定分项写出这个宽度（定过了就补那一句，没定过就开未定项、把这一行的指向改成它）；指错了分项就改指向（判据在这一格函数上方的注释）"
stage_cell unit-header-fields judge_unit_header_fields "单元头字段表（数据单元与树表单元两张）与 D18 逐项比：五元组对已定项 3，偏移与名字对已定项 7 的共同前缀加已定项 18 那一类的偏移表" \
  "表写错了改 .claude/kb/layout/01-first-txn.md 那张表（漏的字段补一行、宽度与偏移照 D18 已定项 18），D18 改了定案就照新定案改表；缺表的照红行写的标记在表前一行补上（判据在这一格函数上方的注释）"
stage_cell mount-transparency-fields judge_mount_transparency_fields "挂载透明字段表的 kb 一侧对账（D13 逐字段表的行名、根记录与系统配置的名与区间）" \
  "D13 逐字段表改了行名或 kb 字段表改了名、宽：crates/singlefs-verification/src/mount_transparency.rs 的 MOUNT_TRANSPARENCY_TABLE_* 行名常量与三张 *_FIELD_BYTE_RANGES 跟着改（交实现员）；不按字节比的 D13 行登记进 .claude/gate.d/mount-transparency-d13-rows-without-bytes.tsv（判据在这一格函数上方的注释）"
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
stage_cell governance-refs judge_governance_refs "治理文档、kb 与仓根的 README.md、README.en.md、README.ja.md 里按全名指的门禁与格都在、不用旧编号称呼门禁，治理文档里「门禁 N 号」有对应阶段、反引号里的仓内路径存在、\`文件「小节」\` 在那份文件里找得到；kb 中文页反引号里以仓根目录起头的路径存在、\`文件「小节」\` 找得到（research/ 下进过 git 的放行，存量登在只缩不涨的 dead-path-lag.tsv 与 dead-section-lag.tsv，照规矩该留旧路径或旧小节的登在可增长的 dead-path-kept.tsv，git 历史命令 -- 之后的与说搬迁的那一行不判；保留表与这两种跳过治理文档的路径同样认）" \
  "按红行把门禁改成按全名称呼（旧编号照红行给的今天的名字）、格名照那道门禁的 --list、路径改成今天存在的、「小节」照红行给的最像的标题或句子改（判据与够不着的写法在 .claude/gate.d/lib-governance-refs.py 文件头）"
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
stage_cell unreachable-claims judge_unreachable_claims "kb 里这次改动新增的「不可达 / 走不到 / 触发不了」断言句，同一句带一条仓里在的用例名或写「推的」「没量过」（存量登在只缩不涨的清单里）" \
  "在同一句里写上构造这条路径、断言它被拒的那条用例的名字（crates/ 或 scripts/ 下 #[test] fn <名字> 那一条），没有用例就写「推的，没量过」；存量清单 .claude/gate.d/unreachable-claims-stock.tsv 只缩不涨，新写的句子不往里加（判据在 doc-registries.sh 里这一格函数上方的注释）"
stage_cell counted-claims judge_counted_claims "CLAUDE.md 与仓根三份 README 这类文档里登记的能数的数（条数、项数），与登记表 counted-claims.tsv 里的命令此刻数出来的相等" \
  "数对不上的把正文改成红行给的现数，或把 .claude/gate.d/counted-claims.tsv 那一行的命令改对；正则命中不是一次的改窄或改对，命令跑不通的先在仓根手跑修好（判据在 doc-registries.sh 里这一格函数上方的注释）"
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

# 按格组 source 下面几份库；它们读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取
source "$STAGE_DIRECTORY/lib/doc-registries-field-and-layout-cells.sh"
source "$STAGE_DIRECTORY/lib/doc-registries-unit-header-and-debt-cells.sh"
source "$STAGE_DIRECTORY/lib/doc-registries-reference-and-invariant-cells.sh"

# ── 提前起 citations：选中了这一格，就在各格开始之前把外部引用复核起到后台（格在子 shell 里，起不了、也等不到父进程的后台作业；起法与等法在 start_citations 与 cell_citations 上面的说明）──
if [[ " ${STAGE_CELLS_SELECTED[*]} " == *" citations "* ]]; then
  start_citations
fi

# 这一道的格互不依赖、都只读被判的树：并行跑（.claude/gate.d/lib/stage-cells.sh 文件头「格并行」）
STAGE_CELLS_JOBS="${STAGE_CELLS_JOBS:-$(stage_cells_default_jobs)}"
stage_cells_run "$ROOT"
