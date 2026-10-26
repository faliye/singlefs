#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树）里的三方留痕与同步记录，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 三方论证与阶段同步有没有留下该留的记录：批次范围、判决点名、正文前提、轮次收尾、同步记录、里程碑收口时的欠账、调度记录里延后项的登记、欠账那份的节号
# gate-fixture: yes
# gate-category: 文档类
# gate-similar: doc-experiments.sh 它的 evidence-in-repo 格也用 gate 取法的改动范围、也读 research/prompts/，但判产物与依据落没落进仓（/tmp 路径、产物新不新），不判三方与同步留下的记录；并进来会让八格之外多一种对象（research/results/ 的产物）
# gate-similar: doc-experiments.sh 它的 archive-past-rounds 格按与 knowledge-sync 格同一个基准删上一轮的实验记录，保留判决与 abandoned-rounds.tsv 供这里读；它判工作区里还留没留旧记录，不判记录写没写、写得全不全
# gate-similar: doc-experiments.sh 它的 verdict-false-named 格名字里也有 verdict，但判的是实验产物判决行里的 false 字段有没有被实验页点名，对象是 research/results/ 与实验页，不是三方判决
# gate-similar: code-tooling.sh 它的 change-range-single-source 格判各阶段取改动范围只经 changed-paths.sh 一份取法；这里是那份取法的使用者之一，不判别的阶段
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 的两项判被测代码的行为，这一道只判记录。
#
# gate-cell: batch-scope 这一批要碰的触发文件有没有登记进范围
# gate-cell: verdict-names-local-samples 新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）
# gate-cell: crates-adversarial-review crates 里产品代码的实现改动有没有走过三方正反对抗推理
# gate-cell: code-round-snapshot 按路径点名了产品代码 crates 文件的新判决，有没有点名开工快照、快照有没有罩住点名的每份文件
# gate-cell: implementation-premise 三方论证材料有没有「实现今天的样子」
# gate-cell: abandoned-rounds 发过腿的三方轮次，要么有判决，要么登记撂下
# gate-cell: knowledge-sync 改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录
# gate-cell: agent-def-adversarial-review 改过的 agent 定义与共用约束有没有走过三方审核
# gate-cell: milestone-close-owed 段表写了收口的里程碑，欠账表开着的那张表里没有一行的前置点名它
# gate-cell: deferred-items-registered records/ 下这次新增的带延后类词的行，同一行写着指得到的登记（C 号、拆分提案的节号与行首序号、文件:行或登记处表里的流程文件）
# gate-cell: ledger-section-numbers 欠账那份拆分提案的「## <节号>、」标题不重号，每个节号在「## 索引」表里恰好一行
# 格结构经 lib/stage-cells.sh；每格的成功行报它查了多少项。
#
#   bash .claude/gate.d/doc-process-records.sh [项目根]                              十一格都跑
#   bash .claude/gate.d/doc-process-records.sh --list                                逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-process-records.sh --check <格名>[,<格名>…] [项目根]     只跑点名的格（例：材料员开工只跑 implementation-premise）
#
# 改动范围只经 scripts/gate/changed-paths.sh 取（基准怎么定见它的 gate_diff_base），每种取法这一趟只取一次、各格共用：
#   gate 取法（含工作区、暂存区与未跟踪）：verdict-names-local-samples、code-round-snapshot 只用其中新增的，crates-adversarial-review、knowledge-sync、agent-def-adversarial-review 用全部；
#   head 取法（基准 HEAD）：batch-scope；
#   gate_added_lines（同步记录这一轮新增的行）：knowledge-sync；gate_added_lines（records/ 下这一轮新增的 .md 行）：deferred-items-registered。
# 被判的根不是 git 仓时，用改动范围的七格退 77；implementation-premise、abandoned-rounds、milestone-close-owed 与 ledger-section-numbers 只看树，照判。
# 「新写的判决」各格同一个认法：相对基准新增（git diff --diff-filter=A）、暂存区新增、未跟踪；被改过的旧判决（路径改写、补一句）不算。
#
# ── batch-scope ──
# 为什么：knowledge-sync 格的阶段同步按触发文件算范围，每多碰一个触发文件候选表就多几十到几百行；
# 这一格在暂存之后当场红，逼着先判多碰的那个文件挡不挡着本轮课题。
# 基准是 HEAD（工作区、暂存区与未跟踪），不是 GATE_BASE：这一格问「这一次提交要带哪些」，只能是还没进 HEAD 的；
# knowledge-sync 问「这一轮要回扫哪些」，一轮可以跨几个提交，所以取 GATE_BASE。这一格取 GATE_BASE 会把上一个提交的触发文件
# 也算进来（gate-ok 不存在时基准退回 HEAD~1），每次提交之后必然误红。
# 判据：
#   ① 触发文件：改动范围里命中 TRIGGER_TABLE 里任一条正则的路径；表读不到或一条正则都没有 ⇒ 红。
#   ② 每个触发文件都要在 .claude/batch-scope 里有一行；少一个 ⇒ 红。
#   ③ .claude/batch-scope 里的每个路径都要是这一批的触发文件；多出来的 ⇒ 红（上一批留下的登记会让下一批白放行）。
#   ④ 登记行要写理由：路径之后 # 起，# 后面非空；空理由 ⇒ 红。
#   ⑤ 登记的路径要从仓库根起写（不以 / ./ ../ 开头、不含 /../）；写歪了 ⇒ 红。
#   ⑥ 判③红时列的每一行格式是「.claude/batch-scope:行号：路径」，路径只是冒号后面那一段；同一个路径登记了几行就列几行。
#      出路里打一条能直接粘贴的删行命令：按整行内容匹配（grep -vxF，别的行不动）、写到 .claude/batch-scope.pruned 再换上、删完回读确认那几行已不在。
#      弄坏开关 BATCH_SCOPE_BREAK=prune-command-misses-last：命令里漏掉最后一行，batch-scope-stale-red 里钉命令全文的那条 want 找不到。
#   改动范围一个文件都没有：退 77。
# .claude/batch-scope 一行一条 <从仓库根起的路径><制表符或空格>#<为什么这一批要碰它>；空行与整行以 # 开头的行是注释。一批提交完把它清空（只留注释）。
# 管不到的：理由写得对不对、这一批该不该碰这个文件、登记是不是开工时写的。
# 几个会话共写一个仓时，不带 --staged 跑会把别的会话没提交的改动也算进来；判这一批的范围跑 bash .claude/scripts/gate.sh --staged。
#
# ── verdict-names-local-samples ──
# 规则：.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」。判据：
#   ① 对象是这一次改动新写的 research/prompts/<轮>-main-verification.md（只认 research/prompts/ 顶层）。
#   ② 轮名取判决文件名去掉 -main-verification.md。那一轮本地腿的样本：research/prompts/ 顶层里文件名以 <轮>-local- 起头、
#      含 -output-s 或 -output-void、以 .md 结尾的普通文件（含 <轮>-local-attack-part1-output-s1.md 这类拆题的）。
#   ③ 每一份样本的文件名都要在那份判决里逐字出现（带不带目录都算）；漏的逐个列出，判红。
#   0 字节的样本（ask-local.sh 退 5、退 6 时留下的空 s<n>）不判，成功行逐个报跳过的；没有本地腿样本的判决不判，照样计数、逐个报出。
#   没有新写的判决：退 77。
# 管不到的：点名了读没读、标得对不对、「不稳定」格里的线索有没有逐条写去向；文件名出现在「没读」那一句里也算点名
# （crates-adversarial-review、agent-def-adversarial-review 两格照「点名一节」判据 ⑤ 不认写了否定词的行，这一格没跟）；轮名互为前缀的（x-r1 与 x-r1-local-y-r1）会把后者的样本算给前者。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/doc-process-records.sh/ 下的目录名，括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   VERDICT_LOCAL_SAMPLES_BREAK=skip-void  不认作废副本
#       verdict-names-local-samples-green（输出里找不到「✓ 新写的判决 2 份：1 份逐份点名了那一轮本地腿的 3…」）；verdict-names-local-samples-red（期望退出 1，实测 0）
#   VERDICT_LOCAL_SAMPLES_BREAK=require-empty  连 0 字节的样本也要点名
#       verdict-names-local-samples-green（期望退出 0，实测 1）
#
# ── crates-adversarial-review ──
# 规则：.claude/rules/implementation-workflow.md「三步，缺一步就不算做完」的第 2 步（三方正反对抗推理，.claude/rules/three-way-inference.md）。
# 这次改动里每个产品代码的 crates/*/src/*.rs 文件，都要在这一次改动新写的某份 research/prompts/*-main-verification.md 里按路径点名；
#   「点名」只认判决里「## 这一轮被判过的文件」一节底下的路径，正文别处提到同一个路径不算（判据与写法见下面「点名一节」）；
#   点名了而那一轮开工快照记的哈希与此刻的内容对不上的（判决之后又改过）也不算，判据在「点名一节」⑥。
# 测试代码不判：TEST_CODE_SOURCES 那四个包（harness 档 crates/singlefs-harness/、checker 档 crates/singlefs-checker-tier/、池级 checker crates/singlefs-checker/、函数单元图 crates/singlefs-unit-graph/）的 src 从对象里去掉，
#   成功行与无对象那一行逐个报出去掉的；crates/*/tests/ 与 crates/mutations.tsv 不是 crates/*/src/*.rs，本来就不在对象里。
# 只改了注释的产品代码文件不走三方：判法是规范副本的 scripts/comment-only-change.py（去掉注释与空白之后代码词元与基准逐个相同，
#   文档注释里会编成 doctest 的代码块、文档注释与 // SAFETY: 注释的增删照代码算；上游 show-me-test 用同一份），摘出去的逐个报出来；判法判不了就红。
# 没有产品代码的 crates/*/src 改动（或都只改了注释）：退 77。管不到的：点名了判得对不对；产品代码包 src 里的 #[cfg(test)] 模块按路径分不出来，照判。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   CRATES_ADVERSARIAL_REVIEW_BREAK=test-code-counted  测试代码那四个包的 src 也算对象、要点名
#       crates-adversarial-review-test-code-only（期望退出 77，实测 1）
#   CRATES_ADVERSARIAL_REVIEW_BREAK=exempt-hash-ignored  user_exemption_reason 不核内容哈希，豁免表里有这条路径就算豁免
#       crates-adversarial-review-exempt-stale-red（期望退出 1，实测 0）
#   CRATES_ADVERSARIAL_REVIEW_BREAK=comments-count-as-code  不摘只改注释的文件，照产品代码改动要点名
#       crates-adversarial-review-comment-only（期望退出 77，实测 1）；crates-adversarial-review-comment-and-code-red（输出里找不到「crates 改动里 2 个文件没有三方判决点名」）
#
# ── 点名一节（crates-adversarial-review 与 agent-def-adversarial-review 两格共用） ──
# 为什么：旧判据是整份判决里的字面子串，判决里任何一次提到这个路径都算点名，连「这份文件里没有那一句」这种提到也算；闸判的是「出现过」，不是「被判过」。
# 判决里写一节，两格只在这一节里找路径：
#   ## 这一轮被判过的文件
#
#   - crates/<包>/src/<文件>.rs：<这一轮判了它的什么>
#   - .claude/agents/<名>.md：<这一轮判了它的什么>
# 判据：
#   ① 标题是二级标题，一行逐字是「## 这一轮被判过的文件」（行尾空白不碍；标题后面不加字、不用三级标题）。代码围栏（行首的 ``` 或 ~~~）里的行不当标题认。
#   ② 节到下一个一级或二级标题为止，三级以下的小标题留在节里；节里怎么列不限（列表项、表格行、围栏里的行都算）；一份判决里写几节都认。
#   ③ 路径在节里字面出现（等同 grep -F）才算点名；正文别处、别的小节里出现不算。红格对每个缺的路径写明哪份新判决在正文别处提到过它。
#   ④ 挂账表 .claude/gate.d/review-section-pending.tsv：这一节立起来之前写的新判决没有那一节，一份一行「research/prompts/<轮>-main-verification.md<制表符>理由」，
#      登记的判决认整份正文里的字面提到（改前的判据）。只登记立起来当时真仓上这一次新写、没有那一节、点名全靠正文提到的判决，不预留。
#      只缩不涨：基准（gate 取法的基准）里有这张表时，表里不许出现基准里没有的路径；基准里还没有这张表时不比。
#      每行：路径是 research/prompts/ 顶层的 *-main-verification.md 且文件在仓里、理由 8 个字以上、同一份不登两次；不合规的逐行列出，两格都判红（没有对象可判的那一趟也判）。
#      表里的判决补上了那一节，或不再是这一次新写的，把那一行删掉；表文件不存在不算红；没用上的行不判红，只是不出现在成功行报的「按挂账表认整份正文」里。
#   ⑤ 节里写了否定词（REVIEWED_SECTION_NEGATIONS：没攻、没判、未攻、未判、没审、未审、不攻、不判）的行整行不算点名：「- crates/x.rs：这一轮没攻它」只在这种行里出现的路径照缺的报，
#      红格的提示写明它只在写了否定词的行上出现。按行判，不按分句：同一行里先列判过的、再写「没攻 …」的，那一行上的路径都不算，判过的另起一行列。挂账表登记的判决认整份正文，这一条不管。
#   ⑥ 判决之后又改过不算（用户 2026-10-26 定「比快照与暂存」）：点名它的判决那一轮开工快照 research/prompts/<轮>-snapshot/ 下任一份 *sha256*.txt 清单里
#      有这份文件的行（sha256sum 格式，二进制模式的 * 与开头的 ./ 不碍）时，拿记的哈希比此刻被判的树里的内容（--staged 时那棵树就是暂存进来的那一份）：
#      某份点名它的判决那一轮有一行对得上才算有去向；有行而都对不上的照红，红行写哪份判决、哪份清单记的哈希与此刻的哈希，
#      出路是再开一轮，或照两格各自的豁免表（.claude/crates-review-exempt、.claude/agent-def-review-exempt）按此刻的内容哈希登记豁免（豁免表照判）。
#      那一轮没有快照目录、清单里没有它的行的，没有哈希可比：照算有去向，成功行之后逐个列「没比」（crates 那一格的开工快照立没立住归 code-round-snapshot 格判；
#      定义轮今天不要求快照）；文件此刻已删的没有内容可比，照算有去向。
# 管不到的：节里的路径是不是真判过（不带否定词的转述「这一轮放过了它」「留给下一轮」照算点名）；快照里的哈希是不是派腿那一刻记的（事后照改后的内容补一份同形的清单也过）；
#   围栏的开合只认行首的 ``` 与 ~~~、不核长度与种类是否配对；code-round-snapshot 格判「点名了哪些产品代码文件」仍读整份正文里的路径（它问快照罩没罩住，不问判没判过）。
# 弄坏开关 REVIEW_SECTION_BREAK=<项>，只在样本里认（被判的根不是这道阶段所在的仓、也不是 --staged 的临时树；平常跑时环境里留着它不起作用）。
# 逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   REVIEW_SECTION_BREAK=whole-verdict-substring  退回整份判决的字面子串匹配（改前的判据），挂账表也不起作用
#       crates-adversarial-review-section-moved-red、crates-adversarial-review-section-edges-red、agent-def-adversarial-review-section-moved-red、
#       crates-adversarial-review-section-negated-red、agent-def-adversarial-review-section-negated-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=negated-lines-counted  节里写了否定词的行也算点名（判据 ⑤ 之前的判法）
#       crates-adversarial-review-section-negated-red、agent-def-adversarial-review-section-negated-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=pending-rows-unchecked  挂账表的行不核（格式、理由、路径形态、文件在不在、重复）
#       crates-adversarial-review-pending-table-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=pending-growth-ignored  挂账表不和基准比，新加的行放过
#       crates-adversarial-review-pending-grown-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=snapshot-hash-unchecked  点名了就算有去向，不拿那一轮快照的哈希比此刻的内容（判据 ⑥ 之前的判法），成功行之后的快照比对那一行也不打
#       crates-adversarial-review-snapshot-stale-red、agent-def-adversarial-review-snapshot-stale-red（期望退出 1，实测 0）；
#       crates-adversarial-review-snapshot-green、agent-def-adversarial-review-snapshot-green（输出里找不到「判决之后改没改过（拿那一轮开工快照清单里的 sha256 比此刻的内容）：1 个对得上，1 个没比」）；
#       crates-adversarial-review-snapshot-stale-exempt-green（输出里找不到「✓ crates 改动 2 个文件都有去向（新写的判决文件 1 份，另有 1 个按 .claude/crates-review-exempt 豁免，基准 HEAD）」）
#
# ── code-round-snapshot ──
# 规则：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」。快照形态：research/prompts/<轮>-snapshot/ 目录，
# 里面 crates-sha256.txt（sha256sum 格式：64 位十六进制哈希、两个空格、仓根相对路径）与 kb-sha256.txt；判决里写一行「开工快照」指到那个目录。
# 对象：这次改动新写的 research/prompts/*-main-verification.md（取法与 crates-adversarial-review 同一个函数 new_verdict_paths）里，
#   按路径点名了至少一份产品代码 crates/*/src/*.rs 的判决。点名的认法：正文里匹配 crates/<包>/src/<…>.rs 的字面路径，TEST_CODE_SOURCES 那四个包的 src 不算；
#   只点 kb 的推论轮、只点测试代码的、通配写法（crates/*/src/*.rs）的判决不判，成功行与无对象那一行逐份报出。早先的判决（不在新增里的）不判。
# 判据，每份判决三条，缺一样就红，理由点名判决与缺的东西：
#   ① 正文出现 research/prompts/<轮>-snapshot/，轮名取判决文件名去掉 -main-verification.md；
#   ② 那个目录在树里（这次新加的或已跟踪的都行），里面有清单 crates-sha256.txt（冻结副本那一套记的 frozen-copy-sha256.txt 也认，任一 *sha256.txt）；
#   ③ 判决点名的每份产品代码文件，在那几份清单的某一份里有一行（两个空格前是 64 位十六进制哈希、后面是那份文件的仓根相对路径）。
#      点名的文件树里已经不存在的（这一轮删掉的），没有内容可算哈希，不要求有行，成功行逐个报出。
# 快照里的哈希与此刻的内容（被判的树，--staged 时是暂存进来的那一份）对不对得上不在这一格判：crates-adversarial-review 格拿它比（「点名一节」判据 ⑥，对不上判红：判决之后又改过）。
# 管不到的：哈希与派腿那一刻的文件内容对不对得上（门禁只看这一次的树，看不到派腿时刻；对得上靠主 agent 写判决前 sha256sum -c，
#   对不上的倒推，写法在 .claude/rules/implementation-workflow.md 同一小节）；快照是不是派腿之前记的（事后补一份同形的也过）；
#   快照里列没列判决没点名、但这一轮改了的文件（那一半由 crates-adversarial-review 管点名，这里只管点名的被罩住）；路径出现在「没攻它」那句里也算点名。
# 没有新写的判决，或新写的判决都没点名产品代码文件：退 77。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   CODE_ROUND_SNAPSHOT_BREAK=snapshot-coverage-skipped  不判 ③，清单里有没有判决点名的文件都不看
#       code-round-snapshot-red（期望退出 1，实测 0）；
#       code-round-snapshot-green（输出里找不到「✓ 新写的判决 5 份：3 份点名了产品代码 crates 文件（共点名 5 处）…」：③ 不跑，罩住的处数与不在树里的那份都不报）；
#       code-round-snapshot-no-snapshot-red（输出里找不到「新写的判决里 4 份点名了产品代码 crates 文件，开工快照没立住（共 4 处）」：哈希写坏的那份不再报）
#
# ── implementation-premise ──
# 规则：.claude/rules/implementation-first.md 第 3 条。判据只看形式：
#   ① 标题日期在 PREMISE_CUTOFF 及以后的正文（research/prompts/_*-body.md）里必须出现 crates/。
#      日期取首行里的第一个 YYYY-MM-DD；标题里没有日期的照要查的算。成功行报查了几份，逐个列出因为日期早而没查的。
#   标题日期在 SIDE_AND_SNAPSHOT_CUTOFF 及以后的正文另判两条：
#   ② 退役：本地腿不再派（.claude/rules/three-way-inference.md「各条腿必须互不重复」），正文不再判这一行；
#   ③ 有背景材料（_<轮>-background.md）的轮要有开工快照目录 <轮>-snapshot/，小节清单（_<轮>-checklist.md）里每个「### 小节清单：`<文件>`」
#      点名的 .claude/kb/ 文件在快照目录里某一份 sha256 清单里有一行。
#   正文全都早于 PREMISE_CUTOFF：退 77。
# 两个截止日之前的证据不回改（evidence-discipline「原样保存的证据不许事后改」）。管不到的：读没读对、改法是不是真按实现写的。
#
# ── abandoned-rounds ──
# 判据（只看 research/prompts/ 顶层文件，不进子目录）：
#   ① 认轮次：文件名是 _<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md 之一，
#      就算这一轮发过腿；轮名必须以 -r<N> 结尾。
#   ② 认判决，两种任一：research/prompts/<轮>-main-verification.md；或 .claude/kb/ 下某个 .md 的同一小节（一个标题到下一个标题之间）
#      既点名这一轮的材料（research/prompts/<轮>- 或 research/prompts/_<轮>-），又有「**判决」（判决文件出现之前的几轮这样写）；
#      「**判决」后面去掉紧跟的 **、冒号与空白之后以否定词（没判、未判、待判）起头的（「**判决**：没判」「**判决：待判**」）不算，同一小节里另有一处不带否定词的照算。
#   ③ 认撂下：research/prompts/abandoned-rounds.tsv 一行一轮，四列用制表符分隔：轮名、日期、为什么撂下、去向。
#   ④ 登记行一行收不收，判法只在 .claude/hooks/lib_abandoned_rounds.py 一份，派发闸 ⑥ 调同一份（不收 ⟺ 红）：四列不空、日期 YYYY-MM-DD、同一轮只一行、
#      出处「records/<文件>.md 第 N 行〔<原话>〕」（或 records/<文件>.md:N〔<原话>〕；〔〕里照抄出处那一行里的一段原话，原话所在的那一行含这一轮的名字；拿它在那份文件里找：
#      恰好命中一次的取命中的那一行，行号只当提示：对不上而原话找得到照收、成功行后面逐个报挪到第几行；
#      命中几处而行号提示正落在其中一处的、原话找不到而行号提示那一行有这一轮的名字、〔〕与那一行去掉 .claude/kb/term-renames.md 登记的新旧名与日期
#      （再去掉空白与逗号、剩下的空括号「（）」「()」）之后〔〕逐字落在那一行里的（只经过术语改名、删日期），取提示那一行，成功行后面逐个报照提示取的是哪几行；
#      没带原话、找不到而提示那一行没有这一轮的名字、找不到而提示那一行去掉新旧名与日期之后对不上（〔〕写的是转述、改了别的字）、命中几处而提示不在其中都红；
#      取到的那一行里有这一轮的名字、除这一轮外没有别的轮名、有一句「」括起来的 6 个字以上的用户原话）。
#      「别的轮名」= 仓里认得的轮名全集（research/prompts/ 下 _<轮>-body.md、<轮>-snapshot/、<轮>-main-verification.md 的轮名，加登记表第一列的轮名）
#      ∪ 形如 <名>-r<数字> 的写法（不看仓里有没有它的材料）。
#      不要求出处的只有库里写死的旧行名单（LEGACY_SOURCELESS_ROUNDS：c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1，出处这条规矩立起来之前登记、没有出处可补；只缩不涨，不按日期判）；
#      这一格与派发闸 ⑥ 用同一张名单，名单外的行一律要出处。
#   发过腿、没有判决、也没登记 ⇒ 红。登记表写坏（不是四列、有空列、日期不是 YYYY-MM-DD、同一轮登记两行）、出处收不了（没写、指不到、同一行有别的轮名……）、
#   登记了认不出的轮、登记了已经有判决的轮 ⇒ 也红。没有 research/prompts/、或认不出一个发过腿的轮：退 77。
# 不判的，成功行逐个列出前缀：腿提示是旧形态的轮（<题>-local.md、<题>-forward-sonnet.md / -reverse-opus.md）与轮名不以 -r<N> 结尾的；
#   它们都早于判决文件，判决写进 kb、形态不一，按 ② 认会把任意一句提到材料的话读成判决。
# 归档进版本库的腿文件由 git log 取：不在 git 仓里只看树；git log 失败判红，不退回只看树（那样归档的轮次整批消失）。
#   git log 列出的被删文件同样只认 research/prompts/ 顶层：子目录里被删的（腿在 <轮>-<模型>-model/ 里造的门禁样本、<轮>-snapshot/ 里抄的文件）
#   名字再像轮次文件也不算发过腿（弄坏开关 ABANDONED_ROUNDS_BREAK=archived-subdirectories-read 退回改前的读法：取 basename、不限顶层）。
# 管不到的：判决写得对不对、登记的理由真不真、出处那句话是不是用户为这一轮说的（靠主 agent 核）；② 的第二种只认同一小节里既点名又有 **判决，否定词只认紧跟在「**判决」后面的那三个，「**判决**：这一轮先不判」这类别的说法照算判过，
#   「**判决**：未判的那一格补上之后过」这种以否定词起头而其实判过的照算没判（改写那一句）；
#   新轮既不写正文、腿提示又用了五种之外的名字，落进不判的那一列而不红。
#
# ── knowledge-sync ──
# 判阶段同步记录的形式（阶段任务结束时主 agent 判完每处命中写的那份）。判据：
#   ① 触发文件：改动范围里命中 TRIGGER_TABLE 里任一条正则的路径；表读不到或一条正则都没有 ⇒ 红；一个触发文件都没有：退 77。
#   ② 同步记录：改动范围里的 research/prompts/<阶段>-sync.md（prompts 顶层），且文件里有一行只写 <!-- knowledge-sync -->。
#      每个触发文件都要在某份同步记录里按路径逐字出现（等同 grep -F）；漏的逐个列出 ⇒ 红。
#   ③ 每份同步记录有「## 搜索」小节且不空 ⇒ 否则红。
#   ④ 每份同步记录有「## 命中处置」小节，小节里恰好一张表，表头 | 载体 | 原句 | 处置 |。逐行：
#      载体格是「路径:行号」，路径从仓库根起（可包一层反引号），在仓里存在或在改动范围里；
#      处置格以「改了」「补了」「不改：」之一开头，「不改：」后面要写理由；
#      处置是「改了」「补了」的，载体路径要在改动范围里。任一不合 ⇒ 红。表可以 0 行。
#      只判这一轮写的处置行：新写的记录整份，加上旧记录这一轮新加的行（gate_added_lines 取的新增行逐字认）。
#   ⑤ 还没进 HEAD 的同步记录有「## 原始证据」小节，小节里恰好一张表，表头 | 材料 | 路径 | 行数 |，至少一行。
#      逐行：材料格非空；路径格是仓库根起的路径、在仓里现存；行数格是十进制整数，与那份文件的 wc -l 逐字相等。任一不合 ⇒ 红。
#      已经提交的记录是冻结证据、改不得，它点名的材料也归档了，不判。
#
# 同步记录的格式（research/prompts/<阶段>-sync.md）：
#   <!-- knowledge-sync -->
#   # <阶段> 阶段同步
#
#   触发文件：.claude/agents/sweep.md、scripts/agents/agent-watch.py（这一阶段改动范围里的触发文件，逐个按路径写全）
#
#   ## 搜索
#   回扫用的每条命令与它的命中计数，例：grep -rn "<旧说法>" .claude records research | wc -l → 3
#
#   ## 命中处置
#   | 载体 | 原句 | 处置 |
#   |---|---|---|
#   | .claude/agent-common.md:120 | <那一行的原句> | 改了：<改成了什么> |
#   | .claude/kb/pitfalls.md:40 | （新增） | 补了：<补了什么> |
#   | records/2026-09-16-subagent拆分提案.md:88 | <那一行的原句> | 不改：<理由，例：说的是那一次发生的事> |
#
#   ## 原始证据
#   | 材料 | 路径 | 行数 |
#   |---|---|---|
#   | 事实表 | research/prompts/<阶段>-facts.tsv | 137 |
#   | 候选表 | research/prompts/<阶段>-candidates.tsv | 138 |
#   | 逐行判定报告（第一段 f1-f11） | research/prompts/<阶段>-judge-f1-f11.md | 420 |
#   三个小节标题逐字写，后面不加字；原句里的 | 写成 \|；代码围栏里的 # 行不算标题。
#
# 管不到的：记录写得对不对（原句是不是那一行、「不改」的理由站不站得住）、回扫搜没搜全、只用过而没改动的东西；
# 「改了」「补了」只核载体文件在改动范围里，不核那一行真的动了；行号不核是否越界；路径里带制表符、换行或引号的认不出；
# ⑤ 只核点名的材料在不在、行数对不对得上，内容一个字都判不了。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   KNOWLEDGE_SYNC_BREAK=old-rows-rejudged  旧记录里不是这一轮新加的「改了」「补了」行也按这一轮写的判，载体不在改动范围里就报
#       knowledge-sync-old-record-green（期望退出 0，实测 1）
#     这一格原有的红、绿两份样本在这个开关下照过，不算它的证据：红的那份旧记录多报一行 old-sync.md:14 也满足它的 want（expect 只列必须出现的句子、没有不许出现的），
#     绿的那份记录整份是这一轮新写的，没有旧行可错判。
#
# ── agent-def-adversarial-review ──
# 规则：.claude/rules/implementation-workflow.md「改 agent 定义与共用约束，走同一条三步」。改动范围里每个被管文件
# （.claude/agents/、.claude/agent-common.md、.claude/main-agent.md；删掉的也算）都要在这一次改动新写的某份判决里按路径点名，
# 形态照 crates-adversarial-review 格；「点名」同样只认判决里「## 这一轮被判过的文件」一节底下的路径（判据、挂账表与弄坏开关见上面「点名一节」）。
# 没有被管文件的改动：退 77。豁免表的格式在 check_agent_def_adversarial_review 里。
# 管不到的：点名了判得对不对；节里不带否定词的转述（「这一轮放过了它」）照算点名（判据在「点名一节」⑤）。
#
# ── milestone-close-owed ──
# 为什么：用户 2026-10-22 指出里程碑三带着出口里还开着的欠账收了口；收口那一刻没有东西核欠账表里还有没有指着它的账。
# 判法只在 scripts/kb/milestone-close-owed-check.py（取这一道脚本所在仓里那一份，不取项目根：样本根里放不了坏的），这一格调它的 --closed：
#   ① 里程碑文件 .claude/kb/milestone/<两位号>-<名>-txn.md（译文不算）的段表（表头第一格「段」、有「状态」列）里有一行状态以「结束」或「已完结」起头，算收口；
#      没有段表的不算，照文件现状读、不看历史（细则在那份脚本文件头「判法」）。
#   ② 收口的每个里程碑：.claude/kb/checks-owed.md 开着的那张表（「已还清」整行标题之前，切法同 lib-owed.py）里，前置一格里第一个「去向：」之后那一段
#      （「去向：」之前的说明与出处、整格没写「去向：」的都不算）点名它的行数为 0；认「里程碑<号>」（阿拉伯或中文数字，「里程碑三二·5」拆出项号）与「<两位号>-*-txn.md」两种写法，带日期的文件名里的不算。
#      不为 0 逐行列出编号、简称与点名处的原文，判红。
#   没有一个里程碑的段表写了收口：退 77。收口之前先手跑 python3 scripts/kb/milestone-close-owed-check.py <号>（不看收没收口，照判）。
# 管不到的：前置里点名它的那一句是去向还是旁及（「挪出里程碑三」也算点名，要改写成不点名它）；段表之外别处写的收口（文件头「状态：判定结束」）不认。
# 弄坏开关在那份脚本里（MILESTONE_CLOSE_OWED_BREAK，一次设一个，它的自证逐项钉住）。红在这一格的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   MILESTONE_CLOSE_OWED_BREAK=file-form-ignored  不认里程碑文件名那种写法
#       milestone-close-owed-red（输出里找不到「…有 3 行的前置」）
#   MILESTONE_CLOSE_OWED_BREAK=item-suffix-kept  「里程碑三二·5」不拆项号
#       milestone-close-owed-red（输出里找不到「…有 3 行的前置」）
#   MILESTONE_CLOSE_OWED_BREAK=status-ignored  不看段表，每个里程碑都判
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-no-object（期望退出 77，实测 1）
#     这一格的红样本在这个开关下照红（多判出里程碑四、十三也满足它的 want），不算它的证据
#   MILESTONE_CLOSE_OWED_BREAK=filename-dates-counted  带日期的文件名里的「里程碑三」也算点名
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-red（输出里找不到「…有 3 行的前置」）
#   MILESTONE_CLOSE_OWED_BREAK=whole-row-scanned  整行都看，不只看前置那一格（出处一格里的「里程碑三二·4」也算）
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-red（输出里找不到「…有 3 行的前置」）
#   MILESTONE_CLOSE_OWED_BREAK=paid-rows-counted  「已还清」之后的行也算欠着的
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-red（输出里找不到「…有 3 行的前置」）
#   MILESTONE_CLOSE_OWED_BREAK=prerequisite-whole-cell  前置那一格整格都看，不只看「去向：」之后（收窄之前的读法）
#       milestone-close-owed-green（期望退出 0，实测 1）
#   paid-heading-optional 只在那份脚本的自证里有格（样本的欠账表都有「已还清」标题）
#
# ── deferred-items-registered ──
# 为什么：.claude/main-agent.md「一轮怎么开、怎么收」第 2 条要延后的当时就登记、调度记录里写「延后」的那一行同时写出登的那一行的编号或 文件:行；
#   落地审计抽样 8/54 的延后项只写进调度记录、没有去向（records/2026-09-16-subagent拆分提案.md 第九十节 ③），没有检查核它。
# 对象：这一轮 records/ 下新增的 .md 行（gate_added_lines，基准是 gate 取法的基准，未跟踪的整份算新增；只差登记搬迁的行照共用取法剔掉），
#   去掉「」（配平）与反引号里的字、再去掉只在点话题的「延后」之后还带 DEFERRAL_WORDS 里任一个词的。只在点话题的「延后」两种：
#   名词「延后项」（「延后项无去向」「### 延后项的登记」；前面紧挨「进」的「进延后项」「记进延后项」是把这一件延后，照旧算），
#   与带「N/M」统计数的同一个分句里的「延后」（「延后当时就登进欠账表的 6/54」；分句按逗号、句号、分号、冒号与表格竖线切，只管「延后」这一个词）。
#   不判的：词只在「」或反引号里（引规则、引原话）、只在点话题、在围栏代码块里、在标题行上（行首 0–3 个空格之后 1–6 个井号再接空白或行尾；
#   延后什么写在节里的行上）、在两份拆分提案里（欠账那份 records/2026-10-23-subagent拆分提案.md 是第 2 条点名的登记处本身，历史那份 records/2026-09-16-subagent拆分提案.md
#   是它拆出来之前的登记处，两份的编号小节里的行就是登记）。
#   行号按正文逐字找，同样的行取下一处没报过的。
# 判据：同一行写着下面四种之一、且至少一处指得到，算登记了：
#   ① C 号（C<数字>）：.claude/kb/checks-owed.md 开着的那张表里有一行以「| C<数字> |」起头（开着与已还清的切法经 lib-owed.py，OWED_LIBRARY；
#      已还清表里的 C 号不算，红行写明它已还清；认不出「已还清」标题时照那份库的读法全文都当开着的）；
#   ② 拆分提案的节号与行首序号：行里写了「拆分提案」时认「第<节号>节」后 16 个字以内的行首序号（①…㊿，可以几个连写），没写时只认节号后面紧跟序号的写法
#      （「九十节 ㉟」「第九十节（行首序号 ②、③）」）；那一节（「## <节号>、」标题）在、那一节里有行首是那个序号的一行（「| ③ |」）。
#      先在欠账那份里找，没有再到历史那份里找：两份里同号的节任一份有那个序号就算指得到，两份都没有那一节（或都没有那个序号）才算指不到，原因里点名两份；
#      写了「拆分提案」而只写节号的，那一节一行带行首序号的行都没有（只有正文）才算指得到，欠账那份有这一节就看它、没有看历史那份；节号认汉字与阿拉伯数字；「九十节各行」这种不点序号的不算；
#   ③ 文件:行（<路径>:<行号>，可带 -<行号>）：路径从仓根起、文件在树里、行号不超过它的行数。只写文件名的找不到，不算；
#      指到这一行自己（同一份文件、行号或区间罩住这一行）的不算；records/ 下只认两份拆分提案，指到别的记录（调度记录、审计记录）的不算——那里是写延后的地方，不是登记处；
#   ④ 登记处：表格行里，带延后类词的那一格之后的某一格（去向格）点名了登记处表 DEFERRAL_REGISTRY_TABLE（.claude/gate.d/deferral-registries.tsv）
#      登记的一份文件（从仓根起的全路径，可在反引号里，前后不接路径里的字）。登记处是「照它做就一定会做到」的提交时那一套流程，写哪几份在表里，不在判法里写死。
#      不是表格行、或只在延后类词那一格与它之前（被延后的那件事本身）点名的，不算。
#      表一份一行「<从仓根起的路径><TAB><为什么它本身就是登记处，8 个字以上>」，井号起头的是注释；不是两段、文件不在树里、理由不到 8 个字、同一份登记两次的，
#      逐行点名、那一行不当登记处认，这一格红。表不在时一份登记处都没有（点名流程文件的行照「没写」报）。
# 写了而一处都指不到的，逐处点名为什么指不到；一处都没写的，点名「没写」。都红。
# 这一轮 records/ 下新增的 .md 行里一行带延后类词的都没有（或都不判）而登记处表没写坏：退 77。欠账表或两份拆分提案里任一份读不到：红，点名读不到的那几份（C 号与节号判不了；欠账那份读不到不退回只认历史那份）。
# 管不到的：登的那一行是不是真在讲这件事（编号指得到就算；登记处只看去向格点没点名，不看那一格是不是真在说「照它跑」）；只写文件名、写成「第 N 行」的出处；
#   别的说法的延后（「推迟」「以后再」「进欠账」「要等」，不在词表里）；「延后项」与统计数旁边另写了真延后、却只用这两种说法的（「进延后项」除外）；
#   「留给」当「交给这一轮已排定的下一步、保留给某人当场用」讲、「下一批」当「这一次就派出去的那一批」讲、把延后的提前做的，照样判（文字分不出，改写那一行）；
#   决策正文与里程碑收口表只认 文件:行 那一种写法。
# 弄坏开关 DEFERRED_ITEMS_BREAK=<项>，一次设一个（只在样本根上认，经共用库那一判）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   DEFERRED_ITEMS_BREAK=pointers-unresolved-accepted  写了 C 号、节号或 文件:行就算登记了，不核指不指得到
#       deferred-items-registered-red（输出里找不到「…:6：带「下一批」，写的登记指不到——C 号「C99」指不到」）、deferred-items-registered-ledger-red、
#       deferred-items-registered-pointers-red（期望退出 1，实测 0）
#   DEFERRED_ITEMS_BREAK=quotes-counted  「」与反引号里的词也算这一行在说延后
#       deferred-items-registered-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=registry-rows-judged  历史那份拆分提案里新增的行也照判
#       deferred-items-registered-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=ledger-rows-judged  欠账那份拆分提案不当登记处，它里面新增的行也照判
#       deferred-items-registered-ledger-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=ledger-ignored  只认历史那份：节号只在历史那份里找，欠账那份不读、读不到也不报
#       deferred-items-registered-ledger-green（期望退出 0，实测 1）、deferred-items-registered-ledger-missing-red（期望退出 1，实测 0）
#   DEFERRED_ITEMS_BREAK=ledger-missing-tolerated  欠账那份读不到时当空的、退回只认历史那份，不报读不到
#       deferred-items-registered-ledger-missing-red（期望退出 1，实测 0）
#   DEFERRED_ITEMS_BREAK=history-fallback-dropped  欠账那份里找不到的节与序号不再到历史那份里找
#       deferred-items-registered-ledger-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=fenced-lines-judged  围栏代码块里的行也照判
#       deferred-items-registered-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=headings-judged  标题行也照判（关掉标题那一条收窄，回到改前）
#       deferred-items-registered-topic-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=deferral-noun-counted  名词「延后项」也算在说延后（关掉名词那一条收窄，回到改前）
#       deferred-items-registered-topic-green（期望退出 0，实测 1）、deferred-items-registered-topic-red（输出里找不到「…:5：带「下一批」，同一行没写」）
#   DEFERRED_ITEMS_BREAK=statistics-counted  统计数分句里的「延后」也算在说延后（关掉统计数那一条收窄，回到改前）
#       deferred-items-registered-topic-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=into-noun-skipped  「进延后项」也当话题名词去掉
#       deferred-items-registered-topic-red（输出里找不到「…:4：带「延后」，同一行没写」）
#   DEFERRED_ITEMS_BREAK=noun-line-skipped  带名词「延后项」的整行都不判，旁边另写的延后类词一并放过
#       deferred-items-registered-topic-red（输出里找不到「…:5：带「下一批」，同一行没写」）
#   DEFERRED_ITEMS_BREAK=registry-table-ignored  不读登记处表，去向格点名流程文件的不算登记（关掉登记处那一条收窄，回到改前）
#       deferred-items-registered-registry-green（期望退出 0，实测 1）、deferred-items-registered-registry-red（输出里找不到「…里 7 行没登记去向（判了 8 行）」）
#   DEFERRED_ITEMS_BREAK=registry-cell-unchecked  登记处在行里哪儿都算（不看是不是表格行、是不是在去向格）
#       deferred-items-registered-registry-red（输出里找不到「…:6：带「下轮开」，同一行没写」）
#   DEFERRED_ITEMS_BREAK=registry-rows-unchecked  登记处表的行不核（段数、文件在不在、理由长短、重复），第一段照收
#       deferred-items-registered-registry-red（输出里找不到「….tsv:3：.claude/skills/gone.md 不在树里」）
#   DEFERRED_ITEMS_BREAK=paid-c-numbers-accepted  已还清表里的 C 号也算登记（判据 ① 收窄之前的认法）
#       deferred-items-registered-pointers-red（输出里找不到「…:4：带「延后」，写的登记指不到——C 号「C11」指不到」）
#   DEFERRED_ITEMS_BREAK=records-pointers-accepted  文件:行指到 records/ 下别的记录也算登记（判据 ③ 收窄之前的认法）
#       deferred-items-registered-pointers-red（输出里找不到「…:5：带「下一批」，写的登记指不到——文件:行「records/2026-10-22-样本调度.md:3」指不到」）
#   DEFERRED_ITEMS_BREAK=self-pointer-accepted  文件:行指到这一行自己不另判（落到 records/ 那一条照红，原因不再是「指的是这一行自己」）
#       deferred-items-registered-pointers-red（输出里找不到「…:6：…「records/2026-10-22-样本调度.md:6」指不到：指的是这一行自己」）
# 退役判据：.claude/main-agent.md 第 2 条不再要求调度记录里的延后行写登记编号时删掉这一格。
#
# ── ledger-section-numbers ──
# 为什么：几个会话同时往欠账那份 records/2026-10-23-subagent拆分提案.md 追加，节号撞过三次（那份第一百一十节）；deferred-items-registered 按「第<节号>节」找节，
#   撞号时找到哪一节由次序定。对象只有这一份（LEDGER_PROPOSAL），只看树、不用改动范围，不在 git 仓里也照判；树里没有这份：退 77。
# 判据（围栏代码块里的行不算）：
#   ① 有「## 索引」一节（标题一行逐字，行尾空白不碍）；
#   ② 每个「## <节号>、」标题（节号是汉字数字）只出现一次；
#   ③ 每个节号在「## 索引」一节的表里恰好一行：行首「| <节号>」起头的行，第一格节号后面接的括注（「一百（历史那份里的第二个九十七）」）不碍；
#   ④ 索引表里每一行的节号都有对应的「## <节号>、」一节。
#   例外：历史上已经重号、照原样留着的节登记在 LEDGER_DUPLICATE_SECTIONS（<节号>:<份数>，今天只有两个五十六），标题与索引行都要正好那个份数；
#   登记的节号在这份里一节都没有、或份数对不上，照红（例外过时要删）。例外表只缩不涨：新撞的号改号，不往表里加。
# 管不到的：节号是不是按次序接着编（跳号不判）、索引表其余几列写得对不对、历史那份 records/2026-09-16-subagent拆分提案.md（一字不改，里面的重号不判）。
# 弄坏开关 LEDGER_SECTION_NUMBERS_BREAK=<项>，一次设一个（只在样本根上认，经共用库那一判）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   LEDGER_SECTION_NUMBERS_BREAK=duplicate-headings-tolerated  不判标题重号（判据 ②）
#       ledger-section-numbers-duplicate-red（期望退出 1，实测 0）
#   LEDGER_SECTION_NUMBERS_BREAK=index-unchecked  不核索引表（判据 ③ ④）
#       ledger-section-numbers-index-missing-red（期望退出 1，实测 0）
# 退役判据：欠账那份不再往末尾追加新节（并回 kb 或换成别的登记处）时删掉这一格。
#
# ── 样本 ──
# fixtures/doc-process-records.sh/ 下每格一对，各放 .gate-cells 只点名它那一格；setup.sh 里给别的格补的东西不碍事。
#   batch-scope-red                 没登记的触发文件、不在这一批的登记、没写理由的登记、不从仓库根起的路径，四种同时判红
#   batch-scope-green               三个触发文件（工作区改、暂存新增、未跟踪）都登记了，另四个非触发文件不用登记，判绿报对数
#   batch-scope-stale-red           两个触发文件登记对了，只剩三行悬空的（一个路径登记了两行）：红行逐行列「文件:行号：路径」，出路里钉删这三行的那条命令全文
#   batch-scope-stale-pruned-green  同一份登记，setup.sh 里照那条命令跑一遍（命令与 stale-red 的 expect 逐字相同），悬空的三行没了、别的行还在，这一格转绿
#   verdict-names-local-samples-red   两份新写的判决（一份未跟踪、一份暂存新增）各漏一份作废副本，判红逐个列出
#   verdict-names-local-samples-missing-clean-sample  漏一份干净样本，判红
#   verdict-names-local-samples-green 点名齐；0 字节 s2、别一轮与子目录里的样本、没有本地腿样本的判决、只补了一句的旧判决都不算漏，判绿报对数
#   crates-adversarial-review-red   一个没人点名、一个只被改过的旧判决点名、一个被中文文件名的新判决（在「## 这一轮被判过的文件」一节里）点名，判红只列前两个
#   crates-adversarial-review-green 两份新判决各在「## 这一轮被判过的文件」一节里点名一个，判绿报对数
#   crates-adversarial-review-exempt-green  一个被新判决（在那一节里）点名、一个按 .claude/crates-review-exempt 的内容哈希豁免，判绿报豁免的那一个
#   crates-adversarial-review-section-green  三份在那一节里的三种列法（三级小标题底下的列表项、表格行、节里的代码围栏）；标题行尾带空白、判决开头围栏里有同名标题行、节后正文再提一次都不碍事，判绿报对数
#   crates-adversarial-review-section-moved-red  同一份判决：一份列在那一节里，一份只在节前的正文里提到，一份只在节后的正文里提到，判红只列后两份并提示判决在正文别处提到过
#   crates-adversarial-review-section-edges-red  形不对的一节（三级标题、代码围栏里的同名标题、标题带后缀、没有那一节只在正文提到）底下的四份都不算点名，另一份判决的合格一节只救它自己那一份，判红列四份
#   crates-adversarial-review-section-negated-red  同一份判决的那一节里三份都出现：一份写判了什么，一份写「这一轮没攻它」（列表项）、一份写「未判」（表格行），
#                                   判红只列后两份并提示它们只在写了否定词的行上出现（正常点名的绿样本是 crates-adversarial-review-section-green）
#   crates-adversarial-review-pending-green  新判决没有那一节、挂账表登记了它，基准里的表有两行、工作区删了一行，按挂账表认整份正文，判绿并逐个报按挂账表认的判决
#   crates-adversarial-review-pending-table-red  挂账表里判决不存在、理由太短、路径不是判决文件、同一份判决登记两次，判红列四处
#   crates-adversarial-review-pending-grown-red  基准里已有挂账表，工作区新加一行，只缩不涨，判红
#   crates-adversarial-review-exempt-stale-red  豁免表登记的是改前那一版的哈希，不算豁免，判红列出它
#   crates-adversarial-review-snapshot-green  一份在那一节里点名、那一轮开工快照 crates-sha256.txt 记的哈希与此刻对得上；另一份点名它的判决那一轮没有快照，判绿、报 1 个对得上并列出没比的那一份
#   crates-adversarial-review-snapshot-stale-red  同一轮点名两份，快照记的是返回 2 那一版，判决之后 lib.rs 又改成返回 3：判红只列 lib.rs（写清单与两边的哈希），kept.rs 对得上不列
#   crates-adversarial-review-snapshot-stale-exempt-green  同 snapshot-stale-red 的现场，.claude/crates-review-exempt 按改后那一版的哈希登记了 lib.rs，判绿报 1 个豁免
#   crates-adversarial-review-test-code-only  只改了 harness 档、checker 档与池级 checker 三个包的 src、另有 crates/*/tests/ 与 crates/mutations.tsv，没有判决，退 77 逐个报出不判的
#   crates-adversarial-review-comment-only  两份产品代码只改了注释（行注释、行尾注释、文档注释的文字），没有判决，退 77 逐个报出只改注释的
#   crates-adversarial-review-comment-and-code-red  一份只改注释、一份改了字符串里的 //、一份改了文档注释里的 doctest，都没有判决，判红只列后两份
#   code-round-snapshot-red         新判决点名两份产品代码文件、快照清单只列了一份，判红只列缺的那一份
#   code-round-snapshot-no-snapshot-red  四份新判决各缺一样（没提快照目录、目录不在树里、目录里没有 crates-sha256.txt、清单里点名文件的哈希不是 64 位十六进制），判红逐份点名
#   code-round-snapshot-green       两份判决的快照罩住点名的文件（一份快照目录这次新加、一份已跟踪；一份点名的文件已删、不要求有行）；只点 kb、只点测试代码、通配写法与基准里就有的旧判决不判，判绿报对数
#   code-round-snapshot-no-object   新判决只点 kb 与测试代码，退 77 逐份报出不判的
#   implementation-premise-red      没有 crates/ 的正文、标题没日期的正文、缺开工快照，逐个点名判红
#   implementation-premise-green    判绿，报查了几份、没查哪一份
#   abandoned-rounds-red            发了腿没判决没登记的轮、只在 kb 里被提到的轮、写坏与说反话的登记，另有 setup.sh 建的缺旧树对象的 git 仓，判红
#   abandoned-rounds-green          判决文件、kb 小节判决、登记撂下各一轮与三个旧形态前缀，判绿报对数（登记撂下那一行是库里旧行名单里的 c364-r3、没有出处，不要求）
#   abandoned-rounds-source-red     没写出处的行（2026-10-13）、出处那一行同时写着别的轮名、出处指到 research/ 下这一轮自己的材料、「」里的原话不到 6 个字、没写出处而日期是 2026-10-11 的行（不在旧行名单里，不按日期免）、
#                                   出处那一行另写着仓里认得而不以 -r<数字> 结尾的别的轮名 close-b1（research/prompts/_close-b1-body.md）、出处那一行另写着形如 <名>-r<数字> 而仓里没有任何它的材料的别的轮名 ghost-r2，
#                                   〔〕里的原话改了一个字、在出处文件里找不到而行号提示那一行（第 3 行）没有这一轮名字的 gone-r1，
#                                   〔〕里的原话在出处文件里命中两次而行号提示（第 9 行）不在其中的 twice-r1，只写行号没带〔〕原话的 plain-r1，
#                                   〔〕写的是转述、行号提示的第 10 行有这一轮的名字与「」原话而去掉改名与日期之后对不上的 misread-r1，
#                                   十一行逐行点名判红；另有一行名单里的 d16-item1-r2 没出处，不点名
#   abandoned-rounds-source-green   四行带合格出处（「第 N 行」与「:N」两种写法，另一行的出处那一行写着 m3-close-b1：仓里没有它的任何材料、也不是 -r<数字> 形，不当别的轮名；
#                                   drift-r1 写的是第 2 行，〔〕原话在第 5 行：出处那份记录前面插过行，原话恰好找到一次照收，成功行后面报挪到第几行；
#                                   pinned-r1 的〔〕原话在第 6、7 行各一次、行号提示写第 7 行；reworded-r1 的〔〕是改名前的原话（样本根的 .claude/kb/term-renames.md 登记了这次改名），
#                                   dated-r1 的〔〕带着第 9 行后来删掉的日期，两行原话都找不到而去掉新旧名与日期之后落在行号提示那一行里；
#                                   三行照提示取那一行收，成功行后面报照提示取的三行）、
#                                   一行旧行名单里的 c196-r1 没出处，判绿报对数与出处查了几行
#   abandoned-rounds-source-date-parentheses-green  两行的〔〕带着登记时写在括号里的日期：bracketed-date-r1 那一行的全角括号连日期被 date-once 的出路（doc-consolidate.py migrate）删了，
#                                   halfwidth-date-r1 那一行的半角括号连日期是手删的；两行原话都找不到，去掉日期、空白与逗号、剩下的空括号之后落在行号提示那一行里，
#                                   照提示取那一行收，判绿报对数与照提示取的两行
#   abandoned-rounds-kb-negated-red  三轮只有 kb 小节里的判决：neg-r1 写「**判决**：没判」、wait-r2 写「**判决：待判**」，判红点名这两轮；
#                                   ok-r3 写「**判决**：样本候选过（原先未判的那一格…）」（否定词不紧跟在后面）照算判过、不点名（kb 小节判过的绿样本是 abandoned-rounds-green）
#   abandoned-rounds-archived-subdirectory-green  setup.sh 建 git 仓：research/prompts/<轮>-<模型>-model/gate-samples/… 子目录里三份名字像轮次文件的样本
#                                   （_old-topic-r1-body.md、old-topic-r1-opus.md、_r-victim-r1-body.md）先提交、再归档删掉；只认顶层，它们不算发过腿，
#                                   顶层 live-r1 有判决，判绿报「查了 1 轮」（改前取 basename 不限顶层，old-topic-r1 与 r-victim-r1 判红）
#   abandoned-rounds-archived-top-level-red  setup.sh 建 git 仓：顶层发过腿的 dropped-r1（_dropped-r1-body.md、dropped-r1-opus.md）与子目录里一份 _old-topic-r1-body.md
#                                   一起归档删掉，dropped-r1 没判决没登记：判红点名 dropped-r1（归档的顶层轮次照判，不因为只认顶层就整批放过）
#   knowledge-sync-red              没点名的触发文件、只在不带标记的文件里点名的、说改了而载体没动、处置开头不合法、不改没理由、载体没行号、
#                                   载体不存在、缺「## 搜索」、表头写错，外加「## 原始证据」整节缺、表头错、零行、材料不在、行数不对，判红
#   knowledge-sync-green            两份记录点名三个触发文件、三种处置各一行、中文载体、原句里的 \|、围栏里的竖线行、0 行的表、
#                                   对得上的原始证据表（含反引号包着的路径、不以换行结尾的材料），判绿报对数
#   knowledge-sync-old-record-green 基准里已有的旧记录这一轮只追加了一行「不改：」，旧记录里早先「改了」「补了」两行点名的载体（更早一轮改的）不在这次改动范围里，不判；
#                                   这一轮新建的触发文件由这一轮新写的记录点名，判绿报对数
#   agent-def-adversarial-review-red   共用约束没人点名、主 agent 入口只被旧判决点名、删掉的定义没有去向，判红逐个列出
#   agent-def-adversarial-review-green 四份被新判决（在「## 这一轮被判过的文件」一节里）点名、一份删除按 deleted 豁免、顺带改的 README 不算，判绿报对数
#   agent-def-adversarial-review-section-green  三份在那一节里的三种列法（列表项、表格行、三级小标题底下的列表项），判绿报对数
#   agent-def-adversarial-review-section-moved-red  同一份判决：一份列在那一节里，共用约束只在节前的正文里提到，主 agent 入口只在节后的正文里提到，判红只列后两份并提示判决在正文别处提到过
#   agent-def-adversarial-review-pending-green  新判决没有那一节、挂账表登记了它，按挂账表认整份正文，判绿并逐个报按挂账表认的判决（两格共用同一张表）
#   agent-def-adversarial-review-snapshot-stale-red  定义轮快照 sha256sums.txt（路径带 ./）记的是 demo.md 改后第一版，判决之后又加一行：判红只列 demo.md；共用约束记在 judged-sha256.txt、对得上不列
#   agent-def-adversarial-review-snapshot-green  demo.md 在快照 judged-sha256.txt 里有一行（sha256sum -b 的 * 写法）、对得上；主 agent 入口快照里没有行，判绿、报 1 个对得上并列出没比的那一份
#   agent-def-adversarial-review-section-negated-red  同一份判决的那一节里三份都出现：定义那一行写判了什么，共用约束那一行写「没判」、主 agent 入口那一行写「没攻它」，
#                                   判红只列后两份并提示它们只在写了否定词的行上出现
#   milestone-close-owed-red        里程碑三的段表写了结束，欠账表开着的六行里三行的前置点名它（中文号带项号、文件名、阿拉伯号各一行），判红只列这三行；
#                                   点名里程碑十三、「去向：」之后带日期的文件名、只在出处一格、已还清那张表里的各一行不算
#   milestone-close-owed-green      里程碑三的段表写了已完结，开着的五行去向都是别的里程碑（十三、四、十四带项号、文件名 04-*），「去向：」之后带日期的文件名、
#                                   前置格里「去向：」之前与出处一格里的「里程碑三」不算，判绿报对数
#   milestone-close-owed-no-object  两份里程碑文件的段表都没写收口，开着的账点名里程碑三也不判，退 77
#   deferred-items-registered-red   往调度记录追加七行带延后类词的：一行写开着的 C10 不报，六行逐个报（没写登记、C99 欠账表里没有、「九十节 ⑤」那一节没有这一行、
#                                   写了「拆分提案」的「第九十节」有行首序号却没写哪一行、research/notes.md:99 超过行数、gone.md:3 不在树里）
#   deferred-items-registered-green 往调度记录追加五行，C10、欠账那份的 records/2026-10-23-subagent拆分提案.md:8、「九十节 ③」、写了「拆分提案」的只有正文的「第九十四节」、research/notes.md:12 各指一行，判绿报「判了 5 行」；
#                                   词只在「」里的一行、围栏里的一行、往拆分提案九十节加的一行（登记处本身）与基准里的旧行不判
#   deferred-items-registered-topic-green  追加七行：两行标题、两行只把「延后项」当话题名词、一行只在报统计数（「延后项无去向 8/54」「延后当时就登进欠账表的 6/54」）不判，
#                                   一行写 C10 照判，判绿报「判了 1 行」，不判的里只在点话题的 4 行、标题 1 行
#   deferred-items-registered-topic-red    追加六行，一行写 C10 不报，五行逐个报：「进延后项」、「延后项」旁边另有「下一批」、「延后到」、井号后面没空格的「#延后」、统计数旁边的「下一批」
#   deferred-items-registered-registry-green  登记处表两行（staged-gate.md、commit-stages.md）写对，两行表格的去向格各点名一份（一份在反引号里），一行写 C10，判绿报「判了 3 行」
#   deferred-items-registered-registry-red    登记处表四行写坏逐行报（文件不在树里、理由太短、只有一段、登记两次）；追加八行，去向格点名 staged-gate.md 的一行不报，
#                                   七行逐个报：「九十节各行」、没登记的流程文件、登记处只在延后类词那一格之前、不是表格行、staged-gate.md.old、表里写坏了那一行的两份
#   deferred-items-registered-ledger-green  追加四行各指一处：只在欠账那份的「第一百零二节 ①」、只在历史那份的「第三十节 ①」、两份都有九十七节而 ④ 只在历史那份的
#                                   「第九十七节 ④」、写了「拆分提案」的只有正文的欠账那份「第一百零三节」，判绿报「判了 4 行」；往欠账那份新登一节带「下一批」的一行（登记处本身）不判
#   deferred-items-registered-ledger-red    追加三行，一行写 C10 不报，两行逐个报：「第一百零九节 ①」两份都没有那一节、「九十节 ⑦」两份的九十节都没有这一行，原因里点名两份
#   deferred-items-registered-ledger-missing-red  没有欠账那份，追加一行「九十节 ②」（历史那份里指得到），照实报读不到欠账那份、红
#   deferred-items-registered-pointers-red  追加五行：已还清的 C11、文件:行指到同一份调度记录的别一行（records/ 下不是拆分提案）、文件:行指到这一行自己，三行逐个报；
#                                   指欠账那份 records/2026-10-23-subagent拆分提案.md:8 的一行与写开着的 C10 的一行不报
#   ledger-section-numbers-green    欠账那份：两个五十六（例外表登记的 2 处，索引两行）、索引第一格带括注的「一百（…）」、围栏里的标题与表格行不算，判绿报节号、标题与索引行数
#   ledger-section-numbers-duplicate-red  另一个会话同时追加，「## 一百零三、」写了两处（索引一行），判红报重号的两行
#   ledger-section-numbers-index-missing-red  「## 一百零四、」没在索引表占号、索引表多一行没有对应一节的「一百零九」，判红逐处报
# 登记行判法的弄坏开关在 .claude/hooks/lib_abandoned_rounds.py（LIB_ABANDONED_ROUNDS_BREAK，一次设一个；派发闸的自检也带着）。红在这一格的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   columns-unchecked        不核四列不空                     abandoned-rounds-red（输出里找不到「不是四列、或有空列」）
#   date-unchecked           不核日期是 YYYY-MM-DD            abandoned-rounds-red（输出里找不到「日期不是 YYYY-MM-DD」）
#   duplicate-unchecked      同一轮两行也不拒                 abandoned-rounds-red（输出里找不到「dup-r1（第 」）
#   source-optional          出处一概不要求                   abandoned-rounds-source-red（输出里找不到「这些行的出处收不了」）、abandoned-rounds-source-green（输出里找不到「登记表 8 行：…出处按派发闸 ⑥ 同一份判法查了 7 行」）、
#                                                             abandoned-rounds-source-date-parentheses-green（输出里找不到「登记表 2 行：…出处按派发闸 ⑥ 同一份判法查了 2 行」）
#   other-round-name-ok      出处那一行有别的轮名也收         abandoned-rounds-source-red（输出里找不到「除这一轮外还有别的轮名 other-r1」）
#   other-name-by-form       别的轮名的全集退回只认登记表里的名字（仓里的 _<轮>-body.md 等材料不算；-r<数字> 形照认）  abandoned-rounds-source-red（输出里找不到「named-r1：第 7 行的出处…除这一轮外还有别的轮名 close-b1」）
#   other-form-ignored       别的轮名不认 -r<数字> 形，只认仓里认得的全集（改前的判法）  abandoned-rounds-source-red（输出里找不到「formonly-r1：第 8 行的出处…除这一轮外还有别的轮名 ghost-r2」）
#   legacy-by-date           免出处退回按日期（早于 2026-10-12 的行不要求）  abandoned-rounds-source-red（输出里找不到「datedold-r1：第 6 行没有出处」）
#   source-any-path          出处不限 records/                abandoned-rounds-source-red（输出里找不到「ownbody-r1：第 4 行没有出处」）
#   quote-short-ok           「」里一个字也算原话             abandoned-rounds-source-red（输出里找不到「至少 6 个字」）
#   source-required-always   旧行名单里的行也要求出处         abandoned-rounds-green、abandoned-rounds-source-green（期望退出 0，实测 1）
#   line-unread              不找〔〕原话、不取那一行，整份文件里找   abandoned-rounds-source-red（输出里找不到「除这一轮外还有别的轮名 other-r1」「至少 6 个字」）、
#                                                             abandoned-rounds-source-green（输出里找不到「出处行号对不上而〔〕原话恰好找到一次、照收的 1 行」）、
#                                                             abandoned-rounds-source-date-parentheses-green（输出里找不到「出处〔〕原话命中几处或找不到、照行号提示取那一行照收的 2 行」）
#   line-number-read         退回按行号读，〔〕原话不要求也不找   abandoned-rounds-source-green（期望退出 0，实测 1：drift-r1 的第 2 行里没有这一轮的名字）、
#                                                             abandoned-rounds-source-red（输出里找不到「gone-r1：第 9 行的出处 records/halt.md 里找不到」）、
#                                                             abandoned-rounds-source-date-parentheses-green（输出里找不到「出处〔〕原话命中几处或找不到、照行号提示取那一行照收的 2 行」）
#   line-hint-ignored        不看行号提示（命中几处、找不到一律红）  abandoned-rounds-source-green（期望退出 0，实测 1：pinned-r1 命中 2 次、reworded-r1 与 dated-r1 找不到）、
#                                                             abandoned-rounds-source-red（输出里找不到「行号提示的第 10 行有这一轮的名字，但去掉改名表」：不看提示时 misread-r1 的说明退回「也没有这一轮的名字」）、
#                                                             abandoned-rounds-source-date-parentheses-green（期望退出 0，实测 1：bracketed-date-r1 与 halfwidth-date-r1 找不到）
#   hint-line-unmatched      原话找不到时不比〔〕与提示那一行（改前的判法）  abandoned-rounds-source-red（输出里找不到「misread-r1：第 12 行的出处 records/halt.md 里找不到」）
#   empty-parentheses-kept   比〔〕与提示那一行时剩下的空括号「（）」「()」留着不去（改前的判法）  abandoned-rounds-source-date-parentheses-green（期望退出 0，实测 1：bracketed-date-r1 与 halfwidth-date-r1
#                                                             去掉日期之后〔〕里剩一对空括号，落不进提示那一行）
#   source-escapes、name-substring、round-name-unchecked 这三项只在派发闸的自检里有格（这一道的样本没有对应的写法）
# 读归档历史的弄坏开关（这一格自己的，设进环境）：
#   ABANDONED_ROUNDS_BREAK=archived-subdirectories-read  git log 列出的被删文件不限顶层、一律取 basename（改前的读法）
#       abandoned-rounds-archived-subdirectory-green（期望退出 0，实测 1：old-topic-r1、r-victim-r1 被当成没判决没登记的轮）
#     只有顶层归档的那份红样本在这个开关下照红（多报一行 old-topic-r1 也满足它的 want：expect 只列必须出现的句子），不算它的证据
#   ABANDONED_ROUNDS_BREAK=negated-kb-verdict-counted  kb 小节里「**判决」后面接没判、未判、待判的也算判过（改前的认法）
#       abandoned-rounds-kb-negated-red（期望退出 1，实测 0）
# 共用库的弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   STAGE_CELLS_BREAK=red-swallowed  红格按绿记、整道退 0
#       abandoned-rounds-red、abandoned-rounds-source-red、agent-def-adversarial-review-red、agent-def-adversarial-review-section-moved-red、batch-scope-red、code-round-snapshot-no-snapshot-red、code-round-snapshot-red、crates-adversarial-review-comment-and-code-red、crates-adversarial-review-exempt-stale-red、
#       crates-adversarial-review-pending-grown-red、crates-adversarial-review-pending-table-red、crates-adversarial-review-red、crates-adversarial-review-section-edges-red、crates-adversarial-review-section-moved-red、implementation-premise-red、
#       knowledge-sync-red、verdict-names-local-samples-missing-clean-sample、verdict-names-local-samples-red、abandoned-rounds-archived-top-level-red、milestone-close-owed-red、deferred-items-registered-red、deferred-items-registered-topic-red、deferred-items-registered-registry-red、
#       deferred-items-registered-ledger-red、deferred-items-registered-ledger-missing-red、batch-scope-stale-red、
#       crates-adversarial-review-section-negated-red、agent-def-adversarial-review-section-negated-red、abandoned-rounds-kb-negated-red、
#       ledger-section-numbers-duplicate-red、ledger-section-numbers-index-missing-red、deferred-items-registered-pointers-red、
#       crates-adversarial-review-snapshot-stale-red、agent-def-adversarial-review-snapshot-stale-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  不认样本根的 .gate-cells，每份样本十一格全跑
#       abandoned-rounds-green、abandoned-rounds-red、abandoned-rounds-source-date-parentheses-green、abandoned-rounds-source-green、abandoned-rounds-source-red、agent-def-adversarial-review-green、agent-def-adversarial-review-pending-green、agent-def-adversarial-review-red、agent-def-adversarial-review-section-green、agent-def-adversarial-review-section-moved-red、batch-scope-green、
#       batch-scope-red、code-round-snapshot-green、code-round-snapshot-no-object、code-round-snapshot-no-snapshot-red、code-round-snapshot-red、crates-adversarial-review-comment-and-code-red、crates-adversarial-review-comment-only、crates-adversarial-review-exempt-green、crates-adversarial-review-exempt-stale-red、crates-adversarial-review-green、
#       crates-adversarial-review-pending-green、crates-adversarial-review-pending-grown-red、crates-adversarial-review-pending-table-red、crates-adversarial-review-red、crates-adversarial-review-section-edges-red、crates-adversarial-review-section-green、crates-adversarial-review-section-moved-red、crates-adversarial-review-test-code-only、implementation-premise-green、implementation-premise-red、
#       knowledge-sync-green、knowledge-sync-old-record-green、knowledge-sync-red、verdict-names-local-samples-green、verdict-names-local-samples-missing-clean-sample、
#       verdict-names-local-samples-red、abandoned-rounds-archived-subdirectory-green、abandoned-rounds-archived-top-level-red、
#       milestone-close-owed-red、milestone-close-owed-green、milestone-close-owed-no-object、deferred-items-registered-red、deferred-items-registered-green、
#       deferred-items-registered-topic-green、deferred-items-registered-topic-red、deferred-items-registered-registry-green、deferred-items-registered-registry-red、
#       deferred-items-registered-ledger-green、deferred-items-registered-ledger-red、deferred-items-registered-ledger-missing-red、
#       crates-adversarial-review-section-negated-red、agent-def-adversarial-review-section-negated-red、abandoned-rounds-kb-negated-red、ledger-section-numbers-green、
#       ledger-section-numbers-duplicate-red、ledger-section-numbers-index-missing-red、deferred-items-registered-pointers-red、
#       crates-adversarial-review-snapshot-green、crates-adversarial-review-snapshot-stale-red、crates-adversarial-review-snapshot-stale-exempt-green、
#       agent-def-adversarial-review-snapshot-stale-red、agent-def-adversarial-review-snapshot-green（输出里找不到「样本只跑 .gate-cells 点名的格：…」）；
#       batch-scope-stale-pruned-green（期望退出 0，实测 1）；batch-scope-stale-red（输出里找不到「样本只跑 .gate-cells 点名的格：batch-scope」）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# implementation-premise 格按 glob 取正文：没有匹配时展开成空，不留字面的通配串
shopt -s nullglob

STAGE_NAME="$(basename "${BASH_SOURCE[0]}")"
STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$STAGE_DIR/lib/stage-cells.sh"
# 触发文件清单只在这里写一处，batch-scope 与 knowledge-sync 两格都从这里取
TRIGGER_TABLE=".claude/gate.d/knowledge-sync-triggers.tsv"
PREMISE_CUTOFF="2026-09-17"
SIDE_AND_SNAPSHOT_CUTOFF="2026-09-28"
# crates-adversarial-review 格不判的测试代码：harness 档包、checker 档包与池级 checker 的 src（.claude/rules/implementation-workflow.md「三步，缺一步就不算做完」）
TEST_CODE_SOURCES='^crates/(singlefs-harness|singlefs-checker-tier|singlefs-checker|singlefs-unit-graph)/src/'
# 判决里认点名的那一节（crates-adversarial-review 与 agent-def-adversarial-review 两格共用）：标题一行逐字是它，判决里别处提到路径不算点名
REVIEWED_SECTION_HEADING="## 这一轮被判过的文件"
# 那一节里写了这些否定词之一的行不算点名（文件头「点名一节」判据 ⑤），空白分隔；加词之前先量真仓判决的那一节会多红几行
REVIEWED_SECTION_NEGATIONS="没攻 没判 未攻 未判 没审 未审 不攻 不判"
# 这一节上线之前写的新判决没有那一节：挂账表一行一份判决，认整份正文里的提到；只缩不涨，格式见文件头「点名一节」
REVIEW_PENDING_TABLE=".claude/gate.d/review-section-pending.tsv"

stage_cell batch-scope check_batch_scope "这一批要碰的触发文件有没有登记进范围" \
  "没登记的触发文件先问它挡不挡着本轮课题，挡着的在 .claude/batch-scope 加一行「<从仓库根起的路径>  # 为什么这一批要碰它」，不挡着的挪到下一批；多出来的登记（红行格式「文件:行号：路径」，路径是冒号后面那段）照红行下面那条命令整行删掉"
stage_cell verdict-names-local-samples check_verdict_names_local_samples "新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）" \
  "逐份读那一轮本地腿交回的样本（干净的、通读判带损坏的、-output-void<n>.md），在新写的判决里按路径列出：.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」"
stage_cell crates-adversarial-review check_crates_adversarial_review "crates 里产品代码的实现改动有没有走过三方正反对抗推理" \
  "提交之前走一轮三方（.claude/rules/three-way-inference.md），判决写进 research/prompts/<题>-r<n>-main-verification.md，判决里写一节「## 这一轮被判过的文件」（标题逐字、二级标题），节里逐个按路径列出改过的 crates 产品代码文件（测试代码不点名，射程见 .claude/rules/implementation-workflow.md「三步，缺一步就不算做完」）；只认这一节，正文别处提到路径不算；点名写进新的判决，不往旧判决里补；判决之后又改过的（那一轮开工快照的 sha256 与此刻的内容对不上）再开一轮，或照 .claude/crates-review-exempt 按内容哈希登记豁免"
stage_cell code-round-snapshot check_code_round_snapshot "按路径点名了产品代码 crates 文件的新判决，有没有点名开工快照、快照有没有罩住点名的每份文件" \
  "派腿之前 sha256sum <判决要点名的 crates 文件…> > research/prompts/<轮>-snapshot/crates-sha256.txt（kb 文件另记 kb-sha256.txt），判决里写一行开工快照指到那个目录（.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」）；派腿时刻已过的，主 agent 补记快照并在判决里写明是事后补的"
stage_cell implementation-premise check_implementation_premise "三方论证材料有没有「实现今天的样子」" \
  "正文前提表里加一行「实现今天的样子」写读过的 crates/ 路径与事实；补开工快照 research/prompts/<轮>-snapshot/（.claude/agents/three-way-materials.md 第 5b 步）"
stage_cell abandoned-rounds check_abandoned_rounds "发过腿的三方轮次，要么有判决，要么登记撂下" \
  "补判决 research/prompts/<轮>-main-verification.md，或在 research/prompts/abandoned-rounds.tsv 加一行（轮名、日期、为什么撂下、去向，制表符分隔）；登记表写坏的照上面逐行改"
stage_cell knowledge-sync check_knowledge_sync "改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录" \
  "写或补 research/prompts/<阶段>-sync.md（格式见这道门禁文件头 knowledge-sync 那一段）：触发文件逐个按路径点名，「## 搜索」「## 命中处置」「## 原始证据」三节照格式写全"
stage_cell agent-def-adversarial-review check_agent_def_adversarial_review "改过的 agent 定义与共用约束有没有走过三方审核" \
  "走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，判决里写一节「## 这一轮被判过的文件」（标题逐字、二级标题），节里按路径列出改过的每一份；只认这一节，正文别处提到路径不算；用户逐次豁免的登记进 .claude/agent-def-review-exempt；判决之后又改过的（那一轮开工快照的 sha256 与此刻的内容对不上）再开一轮，或照 .claude/agent-def-review-exempt 按内容哈希登记豁免"
stage_cell milestone-close-owed check_milestone_close_owed "段表写了收口的里程碑，欠账表开着的那张表里没有一行的前置点名它" \
  "收口之前把点名它的那几行逐行了结：做完的挪进 .claude/kb/checks-owed.md 的「已还清」，没做完的把去向改到还开着的里程碑（前置里不再写这个里程碑的号或文件名）；改不了的就别收口，把段表那一格的「结束 / 已完结」撤回；改完跑 python3 scripts/kb/milestone-close-owed-check.py <号> 到绿"
stage_cell deferred-items-registered check_deferred_items_registered "records/ 下这次新增的带延后类词的行，同一行写着指得到的登记（C 号、拆分提案的节号与行首序号、文件:行或登记处表里的流程文件）" \
  "照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行：先在 .claude/kb/checks-owed.md、里程碑收口表、决策正文或 records/2026-10-23-subagent拆分提案.md 的编号小节登一行（records/2026-09-16-subagent拆分提案.md 里已有的节照旧认），再在调度记录那一行写开着的 C<号>、「第<节号>节 <行首序号>」或从仓根起的 <路径>:<行号>（records/ 下只认两份拆分提案，不指这一行自己）；去向是提交时那一套流程的，表格行的去向格点名 .claude/gate.d/deferral-registries.tsv 登记的那份文件；指不到的照红行写的原因改"
stage_cell ledger-section-numbers check_ledger_section_numbers "欠账那份拆分提案的「## <节号>、」标题不重号，每个节号在「## 索引」表里恰好一行" \
  "新登一节先在 records/2026-10-23-subagent拆分提案.md 的「## 索引」表插一行占号（scripts/kb/insert-row.py），取下一个没用过的节号再写「## <节号>、」标题；撞了号的改成下一个空号、索引表里同步改；索引里多出的、少了的照红行补删；历史上已经重号的节只登记在这一格的 LEDGER_DUPLICATE_SECTIONS，不往里加"
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
        echo "  ✗ 多给了一个参数「$1」（项目根已经是 $root_argument）"
        echo "     → 怎么办：用法是 bash .claude/gate.d/$STAGE_NAME [项目根] [--check <格名>[,<格名>…]]"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
inside_git=0
git rev-parse --is-inside-work-tree >/dev/null 2>&1 && inside_git=1
# 弄坏开关 REVIEW_SECTION_BREAK 只在样本里认（与样本根的 .gate-cells 同一个口径）：被判的根是这道阶段所在的仓或 --staged 的临时树时不认，平常跑环境里留着它也不起作用
review_break=""
case "${REVIEW_SECTION_BREAK:-}" in
  whole-verdict-substring|pending-rows-unchecked|pending-growth-ignored|negated-lines-counted|snapshot-hash-unchecked)
    if [[ -z "${GATE_STAGED_FROM:-}" && "$(pwd -P)" != "$STAGE_CELLS_STAGE_REPOSITORY" ]]; then review_break="$REVIEW_SECTION_BREAK"; fi ;;
esac
LIB_CHANGED_PATHS="$(cd "$STAGE_DIR/../.." && pwd)/scripts/gate/changed-paths.sh"
# 哪些文件只改了注释：判法只在上游一份（show-me-test 与这道门禁共用），从规范副本调
COMMENT_ONLY_JUDGE="$(cd "$STAGE_DIR/.." && pwd)/singlefs-ai-sop/scripts/comment-only-change.py"
# 叫停轮登记行怎么判：判法只在这一份，派发闸 ⑥ 调同一份（取自这一道脚本旁边 ../hooks/，不取项目根：样本根里放不了坏的）
ABANDONED_ROUNDS_LIBRARY="$(cd "$STAGE_DIR/../hooks" && pwd)/lib_abandoned_rounds.py"
# 里程碑收口时的欠账怎么判：判法只在这一份（文件头「milestone-close-owed」），取自这一道脚本所在的仓，不取项目根：样本根里放不了坏的
MILESTONE_CLOSE_CHECK="$(cd "$STAGE_DIR/../.." && pwd)/scripts/kb/milestone-close-owed-check.py"
# deferred-items-registered 格认的延后类词（.claude/main-agent.md「一轮怎么开、怎么收」第 2 条说的「延后」那一类），空白分隔；加词照这一格文件头「管不到的」先量真仓 records/ 会多红几行
DEFERRAL_WORDS="延后 下轮开 下一轮开 下一批 留给 下一个题目"
# deferred-items-registered 格认的登记处表（文件头「deferred-items-registered」判据 ④）：提交时那一套流程的文件，一份一行带理由，不在判法里写死
DEFERRAL_REGISTRY_TABLE=".claude/gate.d/deferral-registries.tsv"
# ledger-section-numbers 格判的那份（文件头「ledger-section-numbers」）与它历史上已经重号、照原样留着的节：<节号>:<标题与索引行各该有几处>，空白分隔；只缩不涨
LEDGER_PROPOSAL="records/2026-10-23-subagent拆分提案.md"
# deferred-items-registered 格认 C 号只认开着的：欠账表开着与已还清的切法只在这一份（取这一道脚本所在仓里那一份，不取项目根：样本根里放不了坏的）
OWED_LIBRARY="$STAGE_DIR/lib-owed.py"
LEDGER_DUPLICATE_SECTIONS="五十六:2"
# 挂账表与基准那一版比（只缩不涨）：取基准那一版与切表行的读法只在这一份（与 doc-experiments、code-source-discipline、code-tooling 几处同一份）
LAG_BASELINE_LIBRARY="$STAGE_DIR/lib-lag-baseline.py"
# shellcheck source=../../scripts/gate/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里另抄一份。"; exit 1; }

# ── 共用的改动范围：每种取法这一趟只取一次，结果留在这几个变量里（在当前 shell 里调，不经 $( ) 子 shell） ──
gate_base=""
head_base=""
changed_since_gate=""; changed_since_gate_loaded=0; changed_since_gate_status=0
added_since_gate=""; added_since_gate_loaded=0; added_since_gate_status=0
changed_since_head=""; changed_since_head_loaded=0; changed_since_head_status=0
added_record_lines=""; added_record_lines_loaded=0; added_record_lines_status=0
added_records_directory_lines=""; added_records_directory_lines_loaded=0; added_records_directory_lines_status=0
if ((inside_git)); then
  gate_base="$(gate_diff_base gate)"
  head_base="$(gate_diff_base head)"
fi

load_changed_since_gate() {
  if ((changed_since_gate_loaded)); then return "$changed_since_gate_status"; fi
  changed_since_gate_loaded=1
  if changed_since_gate="$(gate_changed_paths "$gate_base" untracked)"; then changed_since_gate_status=0; else changed_since_gate_status=1; fi
  return "$changed_since_gate_status"
}
load_added_since_gate() {
  if ((added_since_gate_loaded)); then return "$added_since_gate_status"; fi
  added_since_gate_loaded=1
  if added_since_gate="$(gate_changed_paths "$gate_base" untracked A)"; then added_since_gate_status=0; else added_since_gate_status=1; fi
  return "$added_since_gate_status"
}
load_changed_since_head() {
  if ((changed_since_head_loaded)); then return "$changed_since_head_status"; fi
  changed_since_head_loaded=1
  if changed_since_head="$(gate_changed_paths "$head_base" untracked)"; then changed_since_head_status=0; else changed_since_head_status=1; fi
  return "$changed_since_head_status"
}
load_added_record_lines() {
  if ((added_record_lines_loaded)); then return "$added_record_lines_status"; fi
  added_record_lines_loaded=1
  if added_record_lines="$(gate_added_lines "$gate_base" 'research/prompts/*-sync.md')"; then added_record_lines_status=0; else added_record_lines_status=1; fi
  return "$added_record_lines_status"
}
# records/ 下这一轮新增的 .md 行（deferred-items-registered 用）
load_added_records_directory_lines() {
  if ((added_records_directory_lines_loaded)); then return "$added_records_directory_lines_status"; fi
  added_records_directory_lines_loaded=1
  if added_records_directory_lines="$(gate_added_lines "$gate_base" 'records/*.md')"; then added_records_directory_lines_status=0; else added_records_directory_lines_status=1; fi
  return "$added_records_directory_lines_status"
}
not_git_repository() {
  echo "  ! $ROOT 不是 git 仓，这一格无对象可判"
  return 77
}

# ── batch-scope ──────────────────────────────────────────────────────────────
check_batch_scope() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_head || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 HEAD）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  # 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - <(printf '%s\n' "$changed_since_head") "$TRIGGER_TABLE" <<'PY'
import os, re, sys

with open(sys.argv[1], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})

TRIGGER_TABLE = sys.argv[2]
SCOPE_FILE = ".claude/batch-scope"

if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，knowledge-sync 格与这一格都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if body and not body.startswith("#"):
        patterns.append(re.compile(body))
if not patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)

if not changed_files:
    print("  ! 相对 HEAD 一个改动都没有，这一格无对象可判")
    sys.exit(77)

triggers = [path for path in changed_files if any(pattern.search(path) for pattern in patterns)]

registered = {}
registered_lines = []   # (行号, 路径, 整行原文)：同一个路径登记了几行就有几条，悬空的要一行不漏地列、删
no_reason, bad_path = [], []
if os.path.isfile(SCOPE_FILE):
    for number, raw in enumerate(open(SCOPE_FILE, encoding="utf-8").read().split("\n"), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        path, _, reason = line.partition("#")
        path = path.strip()
        if not path:
            continue
        registered[path] = number
        registered_lines.append((number, path, raw))
        if not reason.strip():
            no_reason.append(f"{SCOPE_FILE}:{number}：{path} 后面没写为什么这一批要碰它")
        if path.startswith(("/", "./", "../")) or "/../" in path:
            bad_path.append(f"{SCOPE_FILE}:{number}：{path} 不是从仓库根起写的")

missing = [path for path in triggers if path not in registered]
stale = [path for path in registered if path not in triggers]

failed = False
if missing:
    failed = True
    print(f"  ✗ {len(missing)} 个触发文件没登记进这一批的范围（{SCOPE_FILE} 现在登记了 {len(registered)} 个）：")
    for path in sorted(missing):
        print(f"      {path}")                                  # gate-lint:detail
    print(f"     → 怎么办：每多碰一个触发文件就多一轮阶段同步（knowledge-sync 格按触发文件算范围），所以先问一句"
          "「它挡不挡着这一批的课题」——")
    print(f"               不挡：从暂存区撤出去（git restore --staged <路径>，工作区的改动留着），"
          f"把这件事记进 .claude/kb/checks-owed.md 往后延；")
    print(f"               挡着：在 {SCOPE_FILE} 里补一行「<路径><制表符>#<为什么>」，并认下它带来的那一轮阶段同步。")
def shell_quote(text):
    """bash 能直接粘贴的引用：平常的字符原样，含空白、引号、反斜杠或控制字符的写成 $'…'。"""
    if re.fullmatch(r"[A-Za-z0-9_./:=@%+,\-]+", text):
        return text
    if "'" not in text and text.isprintable():
        return "'" + text + "'"
    escapes = {"\\": "\\\\", "'": "\\'", "\t": "\\t", "\n": "\\n", "\r": "\\r"}
    return "$'" + "".join(escapes.get(ch, ch if ch.isprintable() else "\\x%02x" % ord(ch)) for ch in text) + "'"


def prune_command(lines):
    """删掉这几行（按整行内容匹配）并回读确认它们不在了的一条命令，在仓库根跑。
    弄坏开关 BATCH_SCOPE_BREAK=prune-command-misses-last：命令里漏掉最后一行，照它跑完这一格仍红。"""
    patterns = [raw for _, _, raw in lines]
    if os.environ.get("BATCH_SCOPE_BREAK", "") == "prune-command-misses-last" and len(patterns) > 1:
        patterns = patterns[:-1]
    selected = " ".join("-e " + shell_quote(text) for text in patterns)
    pruned_file = f"{SCOPE_FILE}.pruned"
    return (f"grep -vxF {selected} {SCOPE_FILE} > {pruned_file}; [ $? -le 1 ] && mv {pruned_file} {SCOPE_FILE} "
            f"&& ! grep -qxF {selected} {SCOPE_FILE} && echo '已删 {len(lines)} 行，回读确认它们已不在 {SCOPE_FILE} 里' "
            f"|| {{ rm -f {pruned_file}; echo '没删成，请手工核对 {SCOPE_FILE}'; }}")


if stale:
    failed = True
    stale_set = set(stale)
    stale_lines = [entry for entry in registered_lines if entry[1] in stale_set]
    print(f"  ✗ {len(stale)} 个登记的路径不是这一批的触发文件（登记项不起作用，会让下一批白放行）：")
    for number, path, _ in stale_lines:
        print(f"      {SCOPE_FILE}:{number}：{path}")  # gate-lint:detail
    print(f"     → 怎么办：上一批提交完没清表就会这样，把这几行删掉；"
          f"路径写错了就照 git status 的写法改（从仓库根起，中文路径不要带引号）。")
    print(f"               上面每行的格式是「文件:行号：路径」：冒号后面那一段才是路径，前面的 {SCOPE_FILE}:行号 是登记行在哪，不是路径的一部分。")
    print(f"               要整行删掉上面列的这 {len(stale_lines)} 行：在仓库根粘贴下面这一条（按整行内容匹配，别的行不动，删完回读确认它们已不在）：")
    print(f"               {prune_command(stale_lines)}")
if no_reason:
    failed = True
    print(f"  ✗ {len(no_reason)} 行登记没写理由：")
    for entry in no_reason:
        print(f"      {entry}")                                  # gate-lint:detail
    print(f"     → 怎么办：路径后面写 #，# 后面写为什么这一批非碰它不可，理由不许省"
          f"（规矩同 .claude/abbreviations 与 .claude/naming-lint-exclude）。")
if bad_path:
    failed = True
    print(f"  ✗ {len(bad_path)} 行登记的路径不是从仓库根起写的：")
    for entry in bad_path:
        print(f"      {entry}")                                  # gate-lint:detail
    print("     → 怎么办：照 git status 的写法写（例 .claude/gate.d/doc-process-records.sh），别写 ./ 开头或相对别处的路径。")
if failed:
    sys.exit(1)

print(f"  ✓ 这一批的 {len(triggers)} 个触发文件都登记进了 {SCOPE_FILE}，登记的 {len(registered)} 个都在这一批里"
      f"（相对 HEAD 共 {len(changed_files)} 个改动路径，另外 {len(changed_files) - len(triggers)} 个不是触发文件、不用登记；"
      f"触发文件的判据是 {TRIGGER_TABLE} 的 {len(patterns)} 条正则）")
PY
}

# ── verdict-names-local-samples ─────────────────────────────────────────────
check_verdict_names_local_samples() {
  ((inside_git)) || { not_git_repository; return; }
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  local -a new_verdicts=()
  mapfile -t new_verdicts < <(grep -E '^research/prompts/[^/]+-main-verification\.md$' <<<"$added_since_gate" || true)
  if ((${#new_verdicts[@]} == 0)); then
    echo "  ! 这次改动没有新写的判决 research/prompts/<轮>-main-verification.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  python3 - "$gate_base" "${new_verdicts[@]}" <<'PY'
import os
import sys

base = sys.argv[1]
verdict_paths = [path for path in sys.argv[2:] if path]
prompts_directory = "research/prompts"
suffix = "-main-verification.md"
breakage = os.environ.get("VERDICT_LOCAL_SAMPLES_BREAK", "")

top_level_names = sorted(os.listdir(prompts_directory)) if os.path.isdir(prompts_directory) else []


def samples_of(round_name):
    """那一轮本地腿的样本文件名（research/prompts/ 顶层的普通文件）。"""
    prefix = round_name + "-local-"
    found = []
    for name in top_level_names:
        if not (name.startswith(prefix) and name.endswith(".md")):
            continue
        is_void = "-output-void" in name
        if not ("-output-s" in name or is_void):
            continue
        if is_void and breakage == "skip-void":
            continue
        if not os.path.isfile(os.path.join(prompts_directory, name)):
            continue
        found.append(name)
    return found


named_total = 0
with_samples = 0
without_samples = []
skipped_empty = []
missing_by_verdict = []
for verdict_path in verdict_paths:
    if not os.path.isfile(verdict_path):
        continue
    round_name = os.path.basename(verdict_path)[: -len(suffix)]
    samples = samples_of(round_name)
    if not samples:
        without_samples.append(verdict_path)
        continue
    with open(verdict_path, encoding="utf-8", errors="replace") as handle:
        verdict_text = handle.read()
    judged = 0
    missing = []
    for name in samples:
        sample_path = os.path.join(prompts_directory, name)
        if os.path.getsize(sample_path) == 0 and breakage != "require-empty":
            skipped_empty.append(sample_path)
            continue
        judged += 1
        if name in verdict_text:
            named_total += 1
        else:
            missing.append(name)
    if judged:
        with_samples += 1
    else:
        without_samples.append(verdict_path)
    if missing:
        missing_by_verdict.append((verdict_path, missing))

verdict_count = len(verdict_paths)
if missing_by_verdict:
    missing_count = sum(len(missing) for _path, missing in missing_by_verdict)
    print(f"  ✗ 新写的判决里 {len(missing_by_verdict)} 份没按路径点名那一轮本地腿的全部样本（共漏 {missing_count} 份）：")  # gate-lint:summary
    for verdict_path, missing in missing_by_verdict:
        print(f"      {verdict_path} 漏 {len(missing)} 份：")  # gate-lint:detail
        for name in missing:
            print(f"        {name}")  # gate-lint:detail
    print("     → 怎么办：逐份读这一轮本地腿交回的样本（干净的、通读判带损坏的、损坏闸留下的 -output-void<n>.md），在这份判决里按路径列出、")
    print("               各标干净或参考，写它说了什么、去向如何（.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」一节")
    print("               「本地腿交回的每一份样本都读，判决里逐份按路径列出」那一条）；不改样本、不删样本。")
    sys.exit(1)

print(f"  ✓ 新写的判决 {verdict_count} 份：{with_samples} 份逐份点名了那一轮本地腿的 {named_total} 份样本，"
      f"{len(without_samples)} 份那一轮没有本地腿样本；0 字节样本跳过 {len(skipped_empty)} 份（基准 {base}）")
if without_samples:
    print("    没有本地腿样本的：" + "、".join(without_samples))
if skipped_empty:
    print(f"    0 字节样本跳过 {len(skipped_empty)} 份：" + "、".join(skipped_empty))
PY
}

# ── crates-adversarial-review ───────────────────────────────────────────────
# user_exemption_reason <豁免表> <路径>：豁免表一行三段（路径、豁免时那一版的 sha256、为什么），打印这一路径、这一份内容那一行的理由，没有就不打印。
# 按内容哈希认：那份文件再改一个字就对不上。删掉的文件没有内容可算哈希，第二段写 deleted，只在文件确实不在时对得上。
# 只有用户明说「这次不走三方」时才加一行，理由里写明是哪一次、他说了什么。两格共用：.claude/agent-def-review-exempt 与 .claude/crates-review-exempt。
user_exemption_reason() {
  local table="$1" path="$2" now
  [[ -f "$table" ]] || return 0
  if [[ -f "$path" ]]; then now="$(sha256sum -- "$path" | cut -d' ' -f1)"; else now="deleted"; fi
  if [[ "${CRATES_ADVERSARIAL_REVIEW_BREAK:-}" == exempt-hash-ignored ]]; then
    awk -F'\t' -v path="$path" '$1 !~ /^#/ && NF >= 3 && $1 == path { print $3; exit }' "$table"; return 0
  fi
  awk -F'\t' -v path="$path" -v hash="$now" \
    '$1 !~ /^#/ && NF >= 3 && $1 == path && $2 == hash { print $3; exit }' "$table"
}

# new_verdict_paths：这次改动新写的判决（一行一个），要先调过 load_added_since_gate。crates-adversarial-review、agent-def-adversarial-review 与 code-round-snapshot 三格共用这一份取法。
new_verdict_paths() {
  grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added_since_gate" || true
}

# ── 判决里「这一轮被判过的文件」一节：crates-adversarial-review 与 agent-def-adversarial-review 两格只在这一节里找路径 ──
# 判据在文件头「点名一节」。下面这组函数都在当前 shell 里调（缓存是全局数组，经 $( ) 子 shell 传不回来）。
declare -A review_scope_cache=()
declare -A review_pending_rows=()
review_pending_loaded=0
review_scope_result=""

# review_section_text <判决>：打印判决里那一节的正文。标题一行逐字是 REVIEWED_SECTION_HEADING（行尾空白不算）；
# 一节到下一个一级或二级标题为止（三级以下的小标题留在节里）；代码围栏里的行不当标题认，围栏里的路径算在节里。
review_section_text() {
  awk -v heading="$REVIEWED_SECTION_HEADING" '
    { line = $0; sub(/[ \t\r]+$/, "", line) }
    line ~ /^ ? ? ?(```|~~~)/ { fence = !fence; if (in_section) print line; next }
    !fence && line == heading { in_section = 1; next }
    !fence && line ~ /^##?[ \t]/ { in_section = 0; next }
    in_section { print line }
  ' "$1"
}

# review_drop_negated：标准输入是那一节的正文，打印其中没写 REVIEWED_SECTION_NEGATIONS 里任一个词的行（写了的整行不算点名）
review_drop_negated() {
  awk -v words="$REVIEWED_SECTION_NEGATIONS" '
    BEGIN { count = split(words, negations, " ") }
    { for (i = 1; i <= count; i++) if (index($0, negations[i])) next; print }
  '
}

# load_review_pending：读挂账表第一列（判决路径）进 review_pending_rows，一趟只读一次。表自己合不合规是 check_review_pending_table 的事。
load_review_pending() {
  ((review_pending_loaded)) && return 0
  review_pending_loaded=1
  [[ -f "$REVIEW_PENDING_TABLE" ]] || return 0
  local path reason
  while IFS=$'\t' read -r path reason _ || [[ -n "$path" ]]; do
    path="${path//[$' \r']/}"
    [[ -z "$path" || "$path" == '#'* ]] && continue
    review_pending_rows[$path]="$reason"
  done < "$REVIEW_PENDING_TABLE"
  return 0
}

# review_scope_text <判决>：把「在这份判决里按路径点名」认的那段文字放进 review_scope_result。
# 默认是那一节的正文；挂账表里登记的判决、以及弄坏开关打开时，认整份正文（改前的整份子串匹配）。
review_scope_text() {
  local verdict="$1"
  if [[ -n "${review_scope_cache[$verdict]+set}" ]]; then review_scope_result="${review_scope_cache[$verdict]}"; return 0; fi
  load_review_pending
  if [[ "$review_break" == whole-verdict-substring || -n "${review_pending_rows[$verdict]+set}" ]]; then
    review_scope_result="$(cat -- "$verdict")"
  else
    review_scope_result="$(review_section_text "$verdict")"
    # 节里写了否定词的行（「没攻它」「没判」）不算点名；弄坏开关 negated-lines-counted 退回改前：节里出现就算
    [[ "$review_break" == negated-lines-counted ]] || review_scope_result="$(review_drop_negated <<<"$review_scope_result")"
  fi
  review_scope_cache[$verdict]="$review_scope_result"
}

# review_lists_path <判决> <路径>：这份判决点名了这个路径吗（字面子串，范围见 review_scope_text）
review_lists_path() {
  review_scope_text "$1"
  grep -qF -- "$2" <<<"$review_scope_result"
}

# review_mention_note <路径> <判决（一行一个）>：没被点名的路径，若某份新判决在正文别处提到过它，打印一句提示（给红格的出路用），没有就不打印
review_mention_note() {
  local path="$1" verdicts="$2" verdict section
  local -a mentioned=() negated=()
  while IFS= read -r verdict; do
    [[ -z "$verdict" || ! -f "$verdict" ]] && continue
    section="$(review_section_text "$verdict")"
    if grep -qF -- "$path" <<<"$section"; then negated+=("${verdict##*/}")
    elif grep -qF -- "$path" "$verdict"; then mentioned+=("${verdict##*/}"); fi
  done <<<"$verdicts"
  if ((${#negated[@]})); then
    printf '（%s 在「%s」一节里只在写了否定词（%s 之一）的行上提到它，不算点名）' "${negated[*]}" "$REVIEWED_SECTION_HEADING" "${REVIEWED_SECTION_NEGATIONS// /、}"
    return 0
  fi
  ((${#mentioned[@]})) || return 0
  printf '（%s 只在正文别处提到了它，没列在「%s」一节里）' "${mentioned[*]}" "$REVIEWED_SECTION_HEADING"
}

# ── 判决之后又改过没有：拿点名它的那一轮开工快照清单里的 sha256 比此刻的内容（文件头「点名一节」判据 ⑥） ──
review_snapshot_state=""
review_snapshot_detail=""
review_snapshot_matched=0
review_outcome=""
review_outcome_note=""

# review_snapshot_compare <判决> <路径>：结果放进 review_snapshot_state——
#   match（那一轮快照目录 research/prompts/<轮>-snapshot/ 里某份 *sha256*.txt 清单记的这份文件的哈希等于此刻被判的树里的内容；--staged 时那棵树就是暂存进来的那一份）、
#   mismatch（清单里有它的行、没有一行对得上；review_snapshot_detail 写哪份清单记的什么、此刻是什么）、
#   deleted（文件此刻不在，没有内容可比）、none（没有快照目录、或清单里没有它的行）。
# 清单行是 sha256sum 的格式：64 位十六进制、两个空格（二进制模式是一个空格接 *）、路径（开头的 ./ 不碍）。
review_snapshot_compare() {
  local verdict="$1" path="$2" round directory manifest now hash recorded=""
  review_snapshot_state=none; review_snapshot_detail=""
  [[ -f "$path" ]] || { review_snapshot_state=deleted; return 0; }
  round="${verdict##*/}"; round="${round%-main-verification.md}"
  directory="research/prompts/${round}-snapshot"
  [[ -d "$directory" ]] || return 0
  now="$(sha256sum -- "$path" | cut -d' ' -f1)"
  for manifest in "$directory"/*sha256*.txt; do
    [[ -f "$manifest" ]] || continue
    while IFS= read -r hash; do
      if [[ "$hash" == "$now" ]]; then review_snapshot_state=match; review_snapshot_detail="$manifest"; return 0; fi
      recorded+="${recorded:+、}$manifest 记的 ${hash:0:12}…"
    done < <(awk -v path="$path" '
      { sub(/\r$/, "") }
      length($1) == 64 && $1 ~ /^[0-9a-f]+$/ && (substr($0, 65, 2) == "  " || substr($0, 65, 2) == " *") {
        listed = substr($0, 67); sub(/^\.\//, "", listed)
        if (listed == path) print $1
      }' "$manifest")
  done
  if [[ -n "$recorded" ]]; then review_snapshot_state=mismatch; review_snapshot_detail="$recorded，此刻是 ${now:0:12}…"; fi
  return 0
}

# review_judge_named <路径> <判决（一行一个）>：这份文件在这次新写的判决里的去向，放进 review_outcome（两格共用；豁免表由调用方接着查）：
#   named（某份判决在那一节里点名了它，而且那一轮快照清单记的哈希与此刻的内容对得上，或文件已删）、
#   unhashed（点名了，可点名它的判决那一轮都没有快照、或清单里没有它的行，没比）、
#   stale（点名了，快照清单里有它的行而都对不上：判决之后又改过；review_outcome_note 写哪份判决、哪份清单记的什么、此刻是什么）、
#   missing（没有一份判决在那一节里点名它）。
# 弄坏开关 REVIEW_SECTION_BREAK=snapshot-hash-unchecked 退回改前：点名了就算 named，不比哈希。
review_judge_named() {
  local path="$1" verdicts="$2" verdict unhashed=0 notes=""
  review_outcome=missing; review_outcome_note=""
  while IFS= read -r verdict; do
    [[ -z "$verdict" || ! -f "$verdict" ]] && continue
    review_lists_path "$verdict" "$path" || continue
    if [[ "$review_break" == snapshot-hash-unchecked ]]; then review_outcome=named; return 0; fi
    review_snapshot_compare "$verdict" "$path"
    case "$review_snapshot_state" in
      match) review_outcome=named; review_snapshot_matched=$((review_snapshot_matched + 1)); return 0 ;;
      deleted) review_outcome=named; return 0 ;;
      mismatch) notes+="${notes:+；}${verdict##*/} 那一轮的 $review_snapshot_detail" ;;
      *) unhashed=1 ;;
    esac
  done <<<"$verdicts"
  if [[ -n "$notes" ]]; then review_outcome=stale; review_outcome_note="$notes"
  elif ((unhashed)); then review_outcome=unhashed; fi
  return 0
}

# review_report_stale <这一格的对象叫什么> <豁免表> <条目…>：判决之后又改过的那几份，打红行与出路（两格共用）
review_report_stale() {
  local what="$1" table="$2"; shift 2
  echo "  ✗ ${what}里 $# 个在判决之后又改过（点名它的新判决那一轮开工快照 research/prompts/<轮>-snapshot/ 的 sha256 清单记的哈希，与此刻被判的树里——--staged 时是暂存进来的那一份——的内容对不上）："   # gate-lint:summary
  printf '      %s\n' "$@"   # gate-lint:detail
  echo "     → 怎么办：判决之后又改过：再开一轮（派腿之前重记快照，新判决在「$REVIEWED_SECTION_HEADING」一节里点名它），"
  echo "               或照 $table 按内容哈希登记豁免（一行：路径、此刻的 sha256、用户说了什么；用户明说这一版不再走三方才登）。"
}

# review_report_snapshot_coverage <没比的路径说明> <没比的路径…>：成功行之后报快照比对了几份、没比的逐个列出（两格共用；弄坏开关 snapshot-hash-unchecked 下不报）
review_report_snapshot_coverage() {
  local why="$1"; shift
  [[ "$review_break" == snapshot-hash-unchecked ]] && return 0
  echo "    判决之后改没改过（拿那一轮开工快照清单里的 sha256 比此刻的内容）：${review_snapshot_matched} 个对得上，$# 个没比（$why）"
  (($#)) && printf '      没比：%s\n' "$@"
  return 0
}

# report_review_pending_in_use <判决（一行一个）>：这一趟新写的判决里，有哪几份是按挂账表认整份正文的，逐个报出，没有就不报
report_review_pending_in_use() {
  local verdict listed=0
  load_review_pending
  while IFS= read -r verdict; do
    [[ -n "$verdict" && -n "${review_pending_rows[$verdict]+set}" ]] || continue
    if ((listed == 0)); then echo "    按挂账表 $REVIEW_PENDING_TABLE 认整份正文里提到的新写判决（没有「$REVIEWED_SECTION_HEADING」一节；只缩不涨，补了那一节的删掉这一行）："; fi
    listed=$((listed + 1))
    printf '      %s：%s\n' "$verdict" "${review_pending_rows[$verdict]}"
  done <<<"$1"
  return 0
}

# check_review_pending_table：挂账表自己合不合规，有问题打印并返回 1。两格各在开头调一次，没有对象可判的那一趟也判，表烂了不靠有改动才发现。
# 表不存在不算红（少一张表只会让判据更严）。判据：每行「判决路径<制表符>理由」两段非空、理由 8 个字以上、路径是 research/prompts/ 顶层的 *-main-verification.md 且文件在、
# 同一路径不登两次；只缩不涨：基准（gate 取法的基准）里有这张表时，表里不许出现基准里没有的路径。
check_review_pending_table() {
  [[ -f "$REVIEW_PENDING_TABLE" ]] || return 0
  python3 - "$REVIEW_PENDING_TABLE" "$gate_base" "$inside_git" "$REVIEWED_SECTION_HEADING" "$review_break" "$LAG_BASELINE_LIBRARY" <<'PY'
import importlib.util, re, sys

table, base, inside_git, heading, breakage = sys.argv[1], sys.argv[2], sys.argv[3] == "1", sys.argv[4], sys.argv[5]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[6])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
VERDICT_PATH = re.compile(r"research/prompts/[^/]+-main-verification\.md")
MINIMUM_REASON = 8

problems, rows = [], {}
with open(table, encoding="utf-8") as handle:
    for number, fields in lag_baseline.content_rows(handle.read()):
        path = fields[0].strip()
        reason = fields[1].strip() if len(fields) > 1 else ""
        if breakage == "pending-rows-unchecked":
            rows[path] = number
            continue
        if len(fields) < 2 or not path or len(reason) < MINIMUM_REASON:
            problems.append(f"第 {number} 行不是「判决路径<制表符>理由」两段，或理由不到 {MINIMUM_REASON} 个字：{fields[0].strip()}")
            continue
        if not VERDICT_PATH.fullmatch(path):
            problems.append(f"第 {number} 行的路径不是 research/prompts/ 顶层的 *-main-verification.md：{path}")
            continue
        if path in rows:
            problems.append(f"第 {number} 行重复登记了 {path}（第 {rows[path]} 行已经有）")
            continue
        rows[path] = number
        try:
            open(path, "rb").close()
        except OSError:
            problems.append(f"第 {number} 行指向的判决不存在：{path}")

baseline = None
if inside_git and base and breakage != "pending-growth-ignored":
    # 「基准里还没有这张表」与「git 取不到基准那一版」分开（lib-lag-baseline.py 的 baseline_rows）：后者判红，不当成没有基线静默不比
    baseline_rows, baseline_problem = lag_baseline.baseline_rows(base, table)
    if baseline_problem is not None:
        problems.append(f"{baseline_problem.sentence()}，只缩不涨没比")
    elif baseline_rows is not None:
        baseline = {fields[0].strip() for _number, fields in baseline_rows}
if baseline is not None:
    for path, number in rows.items():
        if path not in baseline:
            problems.append(f"第 {number} 行 {path} 是基准 {base} 之后新加的：挂账表只缩不涨")

if problems:
    print(f"  ✗ 点名一节的挂账表 {table} 有 {len(problems)} 处不合规：")   # gate-lint:summary
    for problem in problems:
        print(f"      {problem}")   # gate-lint:detail
    print(f"     → 怎么办：表里一行一份判决「research/prompts/<轮>-main-verification.md<制表符>理由（8 个字以上）」，指到的判决要在仓里；")
    print(f"               只缩不涨：新写的判决一律在「{heading}」一节里按路径列出文件，不往表里加行；补了那一节或判决已不再是这一轮新写的，把那一行删掉。")
    sys.exit(1)
PY
}

check_crates_adversarial_review() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  check_review_pending_table || return 1
  local sources verdicts source verdict_count all_sources test_sources test_count why
  local -a missing=() exempted=() stale=() unhashed=()
  local count=0
  local exempt_table=".claude/crates-review-exempt"
  all_sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed_since_gate" || true)"
  # 测试代码（TEST_CODE_SOURCES）改完直接测、不走三方：从对象里去掉，另列出来
  if [[ "${CRATES_ADVERSARIAL_REVIEW_BREAK:-}" == test-code-counted ]]; then
    sources="$all_sources"; test_sources=""
  else
    sources="$(grep -vE "$TEST_CODE_SOURCES" <<<"$all_sources" || true)"
    test_sources="$(grep -E "$TEST_CODE_SOURCES" <<<"$all_sources" || true)"
  fi
  test_count="$(grep -c . <<<"$test_sources" || true)"
  # 只改了注释的产品代码不走三方：去掉注释与空白之后代码词元与基准逐个相同（判法文件头写了哪些注释照代码算）
  local comment_only="" comment_only_count=0
  local -a source_list=()
  if [[ -n "$sources" && "${CRATES_ADVERSARIAL_REVIEW_BREAK:-}" != comments-count-as-code ]]; then
    if [[ ! -f "$COMMENT_ONLY_JUDGE" ]]; then
      echo "  ✗ 找不到判「只改了注释」的判法 $COMMENT_ONLY_JUDGE"
      echo "     → 怎么办：照 CLAUDE.md「规范从哪来」那一行同步规范副本（判法在上游 scripts/comment-only-change.py），不在这道门禁里另写一份。"
      return 1
    fi
    mapfile -t source_list <<<"$sources"
    if ! comment_only="$(python3 "$COMMENT_ONLY_JUDGE" "$ROOT" "$gate_base" "${source_list[@]}")"; then
      echo "  ✗ 判不出哪些产品代码文件只改了注释（$COMMENT_ONLY_JUDGE 判不了，原话在上面；基准 $gate_base）"
      echo "     → 怎么办：核基准在不在、改动清单里的路径对不对，修好再跑；判不了的不当只改注释。"
      return 1
    fi
    if [[ -n "$comment_only" ]]; then
      sources="$(grep -vxF -f <(printf '%s\n' "$comment_only") <<<"$sources" || true)"
      comment_only_count="$(grep -c . <<<"$comment_only" || true)"
    fi
  fi
  if [[ -z "$sources" ]]; then
    local comment_only_note=""
    ((comment_only_count)) && comment_only_note="，只改注释的产品代码 ${comment_only_count} 个不走三方"
    echo "  ! 这次改动没碰产品代码的 crates/*/src（测试代码 ${test_count} 个不归这一格${comment_only_note}），这一格无对象可判（基准 $gate_base）"
    report_comment_only_sources "$comment_only" "$comment_only_count"
    report_test_code_sources "$test_sources" "$test_count"
    return 77
  fi
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  verdicts="$(new_verdict_paths)"
  while IFS= read -r source; do
    [[ -z "$source" ]] && continue
    count=$((count + 1))
    # 点名了、快照对得上（或没快照可比）的有去向；又改过的与没点名的先查豁免表（按此刻的内容哈希）
    review_judge_named "$source" "$verdicts"
    case "$review_outcome" in
      named) continue ;;
      unhashed) unhashed+=("$source"); continue ;;
    esac
    why="$(user_exemption_reason "$exempt_table" "$source")"
    if [[ -n "$why" ]]; then exempted+=("$source：$why"); continue; fi
    if [[ "$review_outcome" == stale ]]; then stale+=("$source（$review_outcome_note）"); continue; fi
    missing+=("$source$(review_mention_note "$source" "$verdicts")")
  done <<<"$sources"
  if ((${#stale[@]})); then
    review_report_stale "crates 改动" "$exempt_table" "${stale[@]}"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ crates 改动里 ${#missing[@]} 个文件没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没在「$REVIEWED_SECTION_HEADING」一节里按路径列出它；正文别处提到不算，节里写了没攻、没判这类否定词的行也不算；被改过的旧判决不算）："   # gate-lint:summary
    printf '      %s\n' "${missing[@]}"   # gate-lint:detail
    echo "     → 怎么办：写代码之后、提交之前走一轮三方（.claude/rules/three-way-inference.md）：材料带上 diff，正推 / 反推 / 找反例三条腿各攻一遍，"
    echo "               主 agent 的判决写进 research/prompts/<题>-r<n>-main-verification.md，在判决里写一节，标题一行逐字是「$REVIEWED_SECTION_HEADING」（二级标题），"
    echo "               节里一行一个，逐个按路径列出这一轮判过的 crates 文件（节到下一个一、二级标题为止；打中的写回代码再改一轮）："
    echo "                 $REVIEWED_SECTION_HEADING"
    echo "                 - crates/<包>/src/<文件>.rs：<这一轮判了它的什么>"
    echo "               点名写进新的判决，不往旧判决里补；用户明说这次不走三方的，按内容哈希登记进 $exempt_table（一行：路径、sha256、他说了什么）。"
    report_comment_only_sources "$comment_only" "$comment_only_count"
    return 1
  fi
  if ((${#stale[@]})); then
    report_comment_only_sources "$comment_only" "$comment_only_count"
    return 1
  fi
  verdict_count="$(grep -c . <<<"$verdicts" || true)"
  if ((${#exempted[@]})); then
    echo "  ✓ crates 改动 ${count} 个文件都有去向（新写的判决文件 ${verdict_count} 份，另有 ${#exempted[@]} 个按 $exempt_table 豁免，基准 $gate_base）"
    echo "    豁免的 ${#exempted[@]} 个（按内容哈希认，那份文件再改一个字就不再豁免）："
    printf '      %s\n' "${exempted[@]}"
  else
    echo "  ✓ crates 改动 ${count} 个文件都有三方判决点名（新写的判决文件 ${verdict_count} 份，基准 $gate_base）"
  fi
  review_report_snapshot_coverage "那一轮没有开工快照、或快照清单里没有它的行；快照立没立住归 code-round-snapshot 格判" ${unhashed[@]+"${unhashed[@]}"}
  report_review_pending_in_use "$verdicts"
  report_comment_only_sources "$comment_only" "$comment_only_count"
  report_test_code_sources "$test_sources" "$test_count"
}

# report_comment_only_sources <只改注释的路径（一行一个）> <个数>：逐个报出只改了注释、不走三方的产品代码，0 个不报
report_comment_only_sources() {
  local listed="$1" listed_count="$2"
  ((listed_count)) || return 0
  echo "    只改了注释、不走三方的产品代码 ${listed_count} 个（判法 $COMMENT_ONLY_JUDGE）："
  printf '%s\n' "$listed" | sed 's/^/      /'
}

# report_test_code_sources <去掉的路径（一行一个）> <个数>：逐个报出这一格不判的测试代码，0 个不报
report_test_code_sources() {
  local listed="$1" listed_count="$2" path
  ((listed_count)) || return 0
  echo "    不判的测试代码 ${listed_count} 个（harness 档包、checker 档包与池级 checker 三个包的 src，改完直接测、不走三方：.claude/rules/implementation-workflow.md「三步，缺一步就不算做完」）："
  while IFS= read -r path; do
    [[ -n "$path" ]] && printf '      %s\n' "$path"
  done <<<"$listed"
  return 0
}

# ── code-round-snapshot ─────────────────────────────────────────────────────
# 对象与判据在文件头「code-round-snapshot」那一节。判决的取法是 new_verdict_paths，与 crates-adversarial-review 同一份。
check_code_round_snapshot() {
  ((inside_git)) || { not_git_repository; return; }
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  local -a new_verdicts=()
  mapfile -t new_verdicts < <(new_verdict_paths)
  if ((${#new_verdicts[@]} == 0)); then
    echo "  ! 这次改动没有新写的判决 research/prompts/<轮>-main-verification.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  python3 - "$gate_base" "$TEST_CODE_SOURCES" "${CODE_ROUND_SNAPSHOT_BREAK:-}" "${new_verdicts[@]}" <<'PY'
import glob
import os
import re
import sys

base, test_code_pattern, breakage = sys.argv[1:4]
verdict_paths = [path for path in sys.argv[4:] if path]
suffix = "-main-verification.md"
# 判决正文里按路径点名的产品代码文件：crates/<包>/src/<…>.rs 的字面路径（ASCII 字符，遇到中文标点、反引号、冒号、井号就断）；crates/*/src/*.rs 这种通配写法认不出，不算点名
named_source = re.compile(r"crates/[A-Za-z0-9_.-]+/src/[A-Za-z0-9_./-]+\.rs(?![A-Za-z0-9_])")
test_code = re.compile(test_code_pattern)
hash_line = re.compile(r"([0-9a-f]{64})  (\S.*)")

judged = []          # (判决路径, 点名的产品代码文件)
not_judged = []      # (判决路径, 为什么不判)
for verdict_path in verdict_paths:
    if not os.path.isfile(verdict_path):
        continue
    with open(verdict_path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    named = sorted(set(named_source.findall(text)))
    # 树里没有那个包目录的路径（定义轮里举例用的 crates/demo/src/… 这类）不是真文件，不算点名
    real = [path for path in named if os.path.isdir(path.split("/src/", 1)[0])]
    product = [path for path in real if not test_code.search(path)]
    if product:
        judged.append((verdict_path, product, text))
    elif real:
        not_judged.append((verdict_path, f"只点名了测试代码 {len(real)} 份"))
    elif named:
        not_judged.append((verdict_path, f"点名的 {len(named)} 处路径所在的包不在树里（举例用的路径）"))
    else:
        not_judged.append((verdict_path, "没按路径点名 crates/*/src/*.rs"))

if not judged:
    print(f"  ! 新写的判决 {len(verdict_paths)} 份都没按路径点名产品代码的 crates/*/src/*.rs，这一格无对象可判（基准 {base}）")
    for verdict_path, why in not_judged:
        print(f"      {verdict_path}：{why}")
    sys.exit(77)

problems_by_verdict = []
covered_total = 0
absent_notes = []
for verdict_path, product, text in judged:
    round_name = os.path.basename(verdict_path)[: -len(suffix)]
    directory = f"research/prompts/{round_name}-snapshot"
    # 清单：快照目录下任一 *sha256.txt（按包记的 crates-sha256.txt，或冻结副本那一套的 frozen-copy-sha256.txt），点名的文件在哪一份里有行都算；kb-sha256.txt 记的是 kb 文件，不算 crates 清单
    manifests = sorted(path for path in glob.glob(f"{directory}/*sha256.txt") if os.path.basename(path) != "kb-sha256.txt") if os.path.isdir(directory) else []
    problems = []
    if directory + "/" not in text:
        problems.append(f"正文没写开工快照目录 {directory}/")
    if not os.path.isdir(directory):
        problems.append(f"目录 {directory}/ 不在树里（这次改动新加的、已跟踪的都算在树里）")
    elif not manifests:
        problems.append(f"目录里没有 {directory}/crates-sha256.txt（也没有别的 *sha256.txt 清单）")
    elif breakage != "snapshot-coverage-skipped":
        well_formed, malformed = set(), {}
        for manifest in manifests:
            with open(manifest, encoding="utf-8", errors="replace") as handle:
                for number, raw in enumerate(handle.read().split("\n"), 1):
                    line = raw.rstrip("\r")
                    if not line.strip():
                        continue
                    found = hash_line.fullmatch(line)
                    if found:
                        well_formed.add(found.group(2))
                        continue
                    pieces = line.split(None, 1)
                    if len(pieces) == 2:
                        malformed.setdefault(pieces[1].lstrip("*").strip(), (manifest, number))
        shown = "、".join(manifests)
        for path in product:
            if not os.path.exists(path):
                absent_notes.append(f"{verdict_path} 点名的 {path}")
                continue
            if path in well_formed:
                covered_total += 1
            elif path in malformed:
                bad_manifest, bad_number = malformed[path]
                problems.append(f"{bad_manifest} 第 {bad_number} 行点名了 {path}，但哈希不是 64 位十六进制、或哈希与路径之间不是两个空格")
            else:
                problems.append(f"{shown} 没有 {path} 的一行")
    if problems:
        problems_by_verdict.append((verdict_path, problems))

if problems_by_verdict:
    problem_count = sum(len(problems) for _path, problems in problems_by_verdict)
    print(f"  ✗ 新写的判决里 {len(problems_by_verdict)} 份点名了产品代码 crates 文件，开工快照没立住（共 {problem_count} 处）：")  # gate-lint:summary
    for verdict_path, problems in problems_by_verdict:
        print(f"      {verdict_path}：")  # gate-lint:detail
        for problem in problems:
            print(f"        {problem}")  # gate-lint:detail
    print("     → 怎么办：派腿之前 sha256sum <判决点名的 crates 文件…> > research/prompts/<轮>-snapshot/crates-sha256.txt（kb 文件另记 kb-sha256.txt），")
    print("               判决里写一行「开工快照」指到那个目录；腿已经派过的，现在补记并在判决里写明是事后补的、哈希对应哪一版")
    print("               （.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」一节）。")
    sys.exit(1)

named_total = sum(len(product) for _path, product, _text in judged)
print(f"  ✓ 新写的判决 {len(verdict_paths)} 份：{len(judged)} 份点名了产品代码 crates 文件（共点名 {named_total} 处），"
      f"开工快照目录与清单（crates-sha256.txt 或 frozen-copy-sha256.txt）都在、点名的文件逐份有哈希行（{covered_total} 处）；{len(not_judged)} 份没点名产品代码文件不判（基准 {base}）")
if not_judged:
    print("    不判的：")
    for verdict_path, why in not_judged:
        print(f"      {verdict_path}：{why}")
if absent_notes:
    print(f"    点名的文件树里已经不存在、不要求快照有行的 {len(absent_notes)} 处：")
    for note in absent_notes:
        print(f"      {note}")
print("    哈希与派腿那一刻的文件内容对不对得上，这一格判不了（靠主 agent 写判决前 sha256sum -c 核）；"
      "与此刻的内容对不对得上由 crates-adversarial-review 格比（对不上判红：判决之后又改过）")
PY
}

# ── implementation-premise ──────────────────────────────────────────────────
check_implementation_premise() {
  local prompts_directory="research/prompts"
  local body heading date date_label round listed
  local -a bodies=("$prompts_directory"/_*-body.md)
  local checked=0 side_checked=0 failed=0
  local -a missing=() before_cutoff=() undated=() missing_snapshot=()
  for body in "${bodies[@]}"; do
    heading="$(head -n 1 "$body")"
    date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"
    # 标题里没有日期：证不出它早于 PREMISE_CUTOFF，照要查的算，不静默跳过
    if [[ -z "$date" ]]; then
      undated+=("$body")
      date_label="标题没有日期，按要查的算"
    elif [[ "$date" < "$PREMISE_CUTOFF" ]]; then
      before_cutoff+=("$body（$date）")
      continue
    else
      date_label="$date"
    fi
    checked=$((checked+1))
    grep -qF 'crates/' "$body" || missing+=("$body（$date_label）")
    if [[ -n "$date" && ! "$date" < "$SIDE_AND_SNAPSHOT_CUTOFF" ]]; then
      side_checked=$((side_checked + 1))
      round="$(basename "$body")"; round="${round#_}"; round="${round%-body.md}"
      if [[ -f "$prompts_directory/_$round-background.md" ]]; then
        if [[ ! -d "$prompts_directory/$round-snapshot" ]]; then
          missing_snapshot+=("$round：没有 research/prompts/$round-snapshot/")
        else
          while IFS= read -r listed; do
            [[ "$listed" == .claude/kb/* ]] || continue
            grep -qsF "$listed" "$prompts_directory/$round-snapshot"/* || missing_snapshot+=("$round：$listed 不在快照清单里")
          done < <(sed -n 's/^### 小节清单：`\([^`]*\)`.*/\1/p' "$prompts_directory/_$round-checklist.md" 2>/dev/null)
        fi
      fi
    fi
  done
  if ((checked == 0)); then
    echo "  ! 标题日期在 $PREMISE_CUTOFF 及以后的三方论证正文 0 份，这一格无对象可判"
    return 77
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些三方论证正文没有「实现今天的样子」——全文一处 crates/ 都没有："   # gate-lint:summary
    printf '     %s\n' "${missing[@]}"
    echo "     → 怎么办：读 crates/ 里与这一问相关的代码路径，在正文前提表里加一行「实现今天的样子」，写文件名与看到的事实并标成观测；"
    echo "               实现里没有相关代码时，写 grep 命令与零命中（.claude/rules/implementation-first.md 第 2、3 条）。"
    failed=1
  fi
  if ((${#missing_snapshot[@]})); then
    echo "  ✗ 这些轮次的开工快照缺了（有背景材料就要有 <轮>-snapshot/，清单点名的 kb 文件都要在快照清单里）："   # gate-lint:summary
    printf '     %s\n' "${missing_snapshot[@]}"   # gate-lint:detail
    echo "     → 怎么办：照 .claude/agents/three-way-materials.md 第 5b 步补 research/prompts/<轮>-snapshot/kb-sha256.txt（sha256sum 原样输出，路径从仓根起）；腿已经开跑的，快照里写明是补写的与补写的日期"
    failed=1
  fi
  ((failed)) && return 1
  echo "  ✓ 三方论证正文都写了实现今天的样子（检查了 $checked 份：标题日期 ≥ $PREMISE_CUTOFF 的 $((checked - ${#undated[@]})) 份、标题没有日期的 ${#undated[@]} 份）"
  if ((${#undated[@]})); then
    echo "    标题没有日期、照要查的算的 ${#undated[@]} 份：${undated[*]}"
  fi
  echo "    标题日期 ≥ $SIDE_AND_SNAPSHOT_CUTOFF、另核了开工快照的 $side_checked 份"
  if ((${#before_cutoff[@]})); then
    echo "    没查 ${#before_cutoff[@]} 份（标题日期早于 $PREMISE_CUTOFF，规范落地之前写的）：${before_cutoff[*]}"
  else
    echo "    没查 0 份"
  fi
}

# ── abandoned-rounds ────────────────────────────────────────────────────────
check_abandoned_rounds() {
  python3 - "$ABANDONED_ROUNDS_LIBRARY" <<'PY'
import importlib.util, os, re, subprocess, sys

prompts_dir = "research/prompts"
registry_path = "research/prompts/abandoned-rounds.tsv"
kb_dir = ".claude/kb"

if not os.path.isdir(prompts_dir):
    print(f"  ! 没有 {prompts_dir}，这一格无对象可判")
    sys.exit(77)

round_name = r"(?P<round>[a-z0-9][a-z0-9-]*-r[0-9]+)"
dispatch_forms = (
    re.compile(r"^_" + round_name + r"-body\.md$"),
    re.compile(r"^" + round_name + r"-opus\.md$"),
    re.compile(r"^" + round_name + r"-sonnet\.md$"),
    re.compile(r"^" + round_name + r"-local-attack\.md$"),
    re.compile(r"^" + round_name + r"-local-defense\.md$"),
)
legacy_body_form = re.compile(r"^_(?P<prefix>.+)-body\.md$")
legacy_leg_form = re.compile(r"^(?P<prefix>[^_].*?)(?:(?:-(?:forward|reverse|attack|defense|admission|ledger))?-(?:opus|sonnet)|-local|-local-attack|-local-defense)\.md$")

# 树上现有的，加上归档进版本库的那些：只看树的话，一轮的腿文件一被归档，撂下登记表里那一行就报「认不出它发过腿」。
archived_names = set()
archive_log_error = None                     # 在 git 仓里而 git log 失败：归档的轮次读不到，收尾判红
inside_repository = subprocess.run(["git", "rev-parse", "--is-inside-work-tree"],
                                   capture_output=True, text=True).returncode == 0
if inside_repository:
    log = subprocess.run(["git", "-c", "core.quotepath=false", "log", "--all", "--diff-filter=D", "--format=", "--name-only",
                          "--", prompts_dir], capture_output=True, text=True)
    if log.returncode != 0:
        archive_log_error = f"git log 退出码 {log.returncode}：{(log.stderr.strip().splitlines() or ['（git 没说原因）'])[-1]}"
    else:
        # 与树上一样只认 research/prompts/ 顶层：子目录里被删的（腿在自己模型目录里造的门禁样本
        # research/prompts/<轮>-<模型>-model/gate-samples/…/_old-topic-r1-body.md、快照里抄的 kb）不是这一轮发出去的腿，
        # 取 basename 会把它们当成轮次文件。弄坏开关 ABANDONED_ROUNDS_BREAK=archived-subdirectories-read：子目录里的也取 basename（改前的读法）。
        read_subdirectories = os.environ.get("ABANDONED_ROUNDS_BREAK", "") == "archived-subdirectories-read"
        for entry in log.stdout.split("\n"):
            entry = entry.strip()
            if not entry.startswith(prompts_dir + "/"):
                continue
            if "/" in entry[len(prompts_dir) + 1:] and not read_subdirectories:
                continue
            archived_names.add(os.path.basename(entry))
# 不在 git 仓里（样本目录）时只看树：那里没有归档可读，判据不变宽
prompt_files = sorted(set(name for name in os.listdir(prompts_dir)
                          if os.path.isfile(os.path.join(prompts_dir, name))) | archived_names)
dispatch_files_by_round = {}
unrecognized_files = []
for name in prompt_files:
    matched_round = None
    for form in dispatch_forms:
        match = form.match(name)
        if match:
            matched_round = match.group("round")
            break
    if matched_round is not None:
        dispatch_files_by_round.setdefault(matched_round, []).append(name)
    elif legacy_body_form.match(name) or legacy_leg_form.match(name):
        unrecognized_files.append(name)

legacy_prefixes = set()
for name in unrecognized_files:
    match = legacy_body_form.match(name) or legacy_leg_form.match(name)
    prefix = match.group("prefix")
    if prefix not in dispatch_files_by_round:
        legacy_prefixes.add(prefix)

if not dispatch_files_by_round:
    print(f"  ! {prompts_dir} 下没有一个文件名认得出发过腿的轮次（_<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md），这一格无对象可判")
    sys.exit(77)

# kb 小节：一个标题行到下一个标题行之间。只读还没有判决文件的轮要用到的那几条。
rounds_without_verdict_file = [name for name in dispatch_files_by_round if not os.path.isfile(os.path.join(prompts_dir, f"{name}-main-verification.md"))]
kb_sections = []
if rounds_without_verdict_file and os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            section_lines = []
            section_heading = ""
            for line in open(path, encoding="utf-8"):
                if re.match(r"^#{1,6} ", line):
                    if section_lines:
                        kb_sections.append((path, section_heading, "".join(section_lines)))
                    section_lines = [line]
                    section_heading = line.strip()
                else:
                    section_lines.append(line)
            if section_lines:
                kb_sections.append((path, section_heading, "".join(section_lines)))

# 「**判决」后面（去掉紧跟的 **、冒号与空白）接着否定词的不算判过：「**判决**：没判」「**判决：待判**」是在说还没判（文件头 abandoned-rounds 判据 ②）。
# 弄坏开关 ABANDONED_ROUNDS_BREAK=negated-kb-verdict-counted 退回改前：小节里有「**判决」就算
KB_VERDICT_MARK = re.compile(r"\*\*判决")
KB_VERDICT_NEGATIONS = ("没判", "未判", "待判")
negated_kb_verdict_counted = os.environ.get("ABANDONED_ROUNDS_BREAK", "") == "negated-kb-verdict-counted"


def kb_section_has_verdict(text):
    for mark in KB_VERDICT_MARK.finditer(text):
        following = re.sub(r"^[\s*：:]+", "", text[mark.end():mark.end() + 24])
        if negated_kb_verdict_counted or not following.startswith(KB_VERDICT_NEGATIONS):
            return True
    return False


def kb_verdict_location(name):
    citation = re.compile(r"research/prompts/_?" + re.escape(name) + r"-")
    for path, heading, text in kb_sections:
        if citation.search(text) and kb_section_has_verdict(text):
            return f"{path}「{heading}」"
    return None

verdict_file_rounds = []
kb_verdict_rounds = []
rounds_without_any_verdict = []
for name in sorted(dispatch_files_by_round):
    if name not in rounds_without_verdict_file:
        verdict_file_rounds.append(name)
    elif kb_verdict_location(name):
        kb_verdict_rounds.append(name)
    else:
        rounds_without_any_verdict.append(name)

registered_line_numbers = {}
registered_reasons = {}                      # 轮名 → 第三列「为什么撂下」，判「腿一条没派」那一格要读它
malformed_registry_rows = []
source_problem_rows = []
# 登记行一行收不收只在 lib_abandoned_rounds.py 一份（形状、同一轮重复、出处三样），派发闸 ⑥ 调同一份：不收就是红
library_path = sys.argv[1]
try:
    sys.dont_write_bytecode = True
    library_spec = importlib.util.spec_from_file_location("lib_abandoned_rounds", library_path)
    abandoned_library = importlib.util.module_from_spec(library_spec)
    library_spec.loader.exec_module(abandoned_library)
    library_modes = abandoned_library.breaks()
except Exception as error:   # 文件不在、语法错、弄坏开关写错都算读不到
    print(f"  ✗ 读不到登记行的共用判法 {library_path}（{error!r}）")
    print("     → 怎么办：登记行怎么判只有那一份（派发闸 ⑥ 也调它），恢复它，别在这一格另抄一份；LIB_ABANDONED_ROUNDS_BREAK 只写它认的值。")
    sys.exit(1)
registry_rows, registry_error = abandoned_library.read_rows(os.getcwd())
if registry_error is not None:
    print(f"  ✗ 读不了 {registry_path}（{registry_error}）：登记撂下的轮这一次没判")
    print("     → 怎么办：恢复这张表（文件、编码、权限），读不到不是「没有登记」。")
    sys.exit(1)
registry_judgement = abandoned_library.judge_table(os.getcwd(), registry_rows, library_modes)
for row in registry_rows:
    if row.number in registry_judgement.shape:
        malformed_registry_rows.append(registry_judgement.shape[row.number])
        continue
    registered_line_numbers.setdefault(row.name, []).append(row.number)
    if len(row.fields) >= 3:
        registered_reasons[row.name] = row.fields[2]
    if row.number in registry_judgement.sources:
        source_problem_rows.append(f"{row.name}：{registry_judgement.sources[row.number]}")

duplicated_registrations = [f"{name}（第 {', '.join(map(str, numbers))} 行）" for name, numbers in registry_judgement.duplicates.items()]
# 理由里明写没派腿的，不要求认出腿文件：这种轮本来就没有腿文件，正文一被归档就永远认不出，登记表那一行比文件名更权威。
no_leg_form = re.compile(r"腿一条没派|没有派腿|一条腿都没派")
registered_unknown_rounds = [name for name in sorted(registered_line_numbers)
                             if name not in dispatch_files_by_round
                             and not no_leg_form.search(registered_reasons.get(name, ""))]
registered_rounds_with_verdict = [name for name in sorted(registered_line_numbers) if name in verdict_file_rounds or name in kb_verdict_rounds]
unjudged_unregistered_rounds = [name for name in rounds_without_any_verdict if name not in registered_line_numbers]
abandoned_rounds = [name for name in rounds_without_any_verdict if name in registered_line_numbers]

failed = False
if archive_log_error:
    failed = True
    print(f"  ✗ 归档进版本库的腿文件读不到（{archive_log_error}）：只按树判，已归档、既没判决也没登记的轮次这一轮没判")
    print("     → 怎么办：按 git 的报错修好仓库（git fsck 看缺了哪些对象，从远端或备份找回）再跑；读不到归档不是「没有归档」。")
if unjudged_unregistered_rounds:
    failed = True
    print("  ✗ 这些三方轮次发过腿，却没有判决、也不在撂下登记表里：")  # gate-lint:summary
    for name in unjudged_unregistered_rounds:
        print(f"     {name}（{'、'.join(dispatch_files_by_round[name])}）")  # gate-lint:detail
    print(f"     → 怎么办：补判决（写 {prompts_dir}/<轮>-main-verification.md），或在 {registry_path} 加一行：轮名、日期、为什么撂下、去向（四列，制表符分隔）。")
    print("               腿还在跑的轮次，等腿交齐、判决写完再跑门禁；只在 kb 里提到这一轮而没有「**判决」的小节不算判过，「**判决**：」后面写没判、未判、待判的也不算。")
if malformed_registry_rows:
    failed = True
    print(f"  ✗ {registry_path} 这些行写坏了：")  # gate-lint:summary
    for entry in malformed_registry_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行四列用制表符分隔：轮名、日期（YYYY-MM-DD）、为什么撂下、去向；四列都要有内容，# 开头的行是注释。")
if duplicated_registrations:
    failed = True
    print("  ✗ 这些轮次在撂下登记表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated_registrations:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，理由与去向写在同一行里。")
if source_problem_rows:
    failed = True
    print(f"  ✗ {registry_path} 这些行的出处收不了（派发闸 ⑥ 读同一张表、判同一份，不收就是红；只有库里旧行名单 {'、'.join(abandoned_library.LEGACY_SOURCELESS_ROUNDS)} 里的行不要出处，别的行都要）：")  # gate-lint:summary
    for entry in source_problem_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：在那一行里写出处「records/<文件>.md 第 N 行〔<那一行里含这一轮名字的一段原话，逐字照抄>〕」（或 records/<文件>.md:N〔…〕）：〔〕里的原话在 records/ 下那份现存文件里要恰好找到一次"
          "（行号只当提示；找不到就照现在的原文逐字重抄，不写转述——只经过术语改名、删日期才找不到的，行号写到那一行照收；命中多次就把行号写成含这一轮名字的那一处，或多抄几个字），"
          "取到的那一行里要有这一轮的名字、除这一轮外没有别的轮名、并有一句「」括起来的用户原话（去掉空白后至少 6 个字）；")
    print("               这一轮自己的材料（research/prompts/<轮>*）、规则与定义文件、仓外路径、同一行写着别的轮叫停原话的行都不认，另找只写这一轮的那一行。")
if registered_unknown_rounds:
    failed = True
    print(f"  ✗ 撂下登记表登记了这些轮，{prompts_dir} 里认不出它们发过腿（没有 _<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md）：")  # gate-lint:summary
    for name in registered_unknown_rounds:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：轮名多半写错了，改成目录里文件名的前缀；那一轮的文件搬过家就照 .claude/rules/path-moves.md 改；确实没有这一轮就删掉这一行。")
if registered_rounds_with_verdict:
    failed = True
    print("  ✗ 这些轮次登记了撂下，却已经有判决——两处说反话：")  # gate-lint:summary
    for name in registered_rounds_with_verdict:
        where = f"{prompts_dir}/{name}-main-verification.md" if name in verdict_file_rounds else kb_verdict_location(name)
        print(f"     {name}：判决在 {where}")  # gate-lint:detail
    print("     → 怎么办：判决是这一轮的现状，就删掉登记表里那一行；判决那一处其实没判，就把它改成不再写「**判决」或删掉那份判决文件，再留登记。")
if failed:
    sys.exit(1)

checked = len(dispatch_files_by_round)
print(f"  ✓ 发过腿的三方轮次都有判决或撂下登记（查了 {checked} 轮：判决文件 {len(verdict_file_rounds)} 轮、kb 小节里的判决 {len(kb_verdict_rounds)} 轮、登记撂下 {len(abandoned_rounds)} 轮）")
sourced_rows = sum(1 for row in registry_rows if abandoned_library.source_required(row, library_modes))
print(f"    登记表 {len(registry_rows)} 行：形状与同一轮重复都查了，出处按派发闸 ⑥ 同一份判法查了 {sourced_rows} 行，{len(registry_rows) - sourced_rows} 行在旧行名单里、不要求出处")
moved_rows = [f"{row.name}：出处 {registry_judgement.moved[row.number][0]} 写的第 {registry_judgement.moved[row.number][1]} 行，原话现在在第 {registry_judgement.moved[row.number][2]} 行"
              for row in registry_rows if row.number in registry_judgement.moved]
if moved_rows:
    print(f"    出处行号对不上而〔〕原话恰好找到一次、照收的 {len(moved_rows)} 行（行号只当提示，顺手把行号改成现在的）：{'；'.join(moved_rows)}")
hinted_rows = [f"{row.name}：出处 {registry_judgement.hinted[row.number][0]} 第 {registry_judgement.hinted[row.number][1]} 行：{registry_judgement.hinted[row.number][2]}"
               for row in registry_rows if row.number in registry_judgement.hinted]
if hinted_rows:
    print(f"    出处〔〕原话命中几处或找不到、照行号提示取那一行照收的 {len(hinted_rows)} 行（取到的那一行是不是用户为这一轮说的，靠主 agent 核）：{'；'.join(hinted_rows)}")
if legacy_prefixes:
    print(f"    没判 {len(legacy_prefixes)} 个前缀（腿提示是旧形态：<题>-local.md、<题>-forward-sonnet.md 这类，或轮名不以 -r<N> 结尾）：{'、'.join(sorted(legacy_prefixes))}")
else:
    print("    没判 0 个前缀")
PY
}

# ── knowledge-sync ──────────────────────────────────────────────────────────
check_knowledge_sync() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  # 同步记录这一轮新增的行（未跟踪的整份算新增）：旧记录里这一轮新加的「改了 / 补了」行也要判载体在不在改动范围里
  load_added_record_lines || {
    echo "  ✗ 取不到同步记录这一轮新增了哪些行（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到新增行时旧记录里新加的处置行没法判，不是通过。"
    return 1
  }
  # 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - "$gate_base" <(printf '%s\n' "$changed_since_gate") <(printf '%s\n' "$added_record_lines") "$TRIGGER_TABLE" "$STAGE_NAME" <<'PY'
import os, re, subprocess, sys

base = sys.argv[1]
with open(sys.argv[2], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})
changed_set = set(changed_files)
# 同步记录这一轮新增的行：{记录路径: {整行}}（gate_added_lines 的「路径<TAB>行文」）
added_lines_by_record = {}
with open(sys.argv[3], encoding="utf-8", errors="replace") as handle:
    for added in handle.read().split("\n"):
        if "\t" in added:
            added_path, added_text = added.split("\t", 1)
            added_lines_by_record.setdefault(added_path, set()).add(added_text)

# 触发文件的清单只读外层传进来的 TRIGGER_TABLE（与 batch-scope 格同一份），不在这里再存一份：两格算的范围要一致。
TRIGGER_TABLE = sys.argv[4]
STAGE_NAME = sys.argv[5]
if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，这一格与 batch-scope 格都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
trigger_patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if not body or body.startswith("#"):
        continue
    trigger_patterns.append(re.compile(body))
if not trigger_patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)
trigger_files = [path for path in changed_files if any(pattern.search(path) for pattern in trigger_patterns)]
outside_trigger_count = len(changed_files) - len(trigger_files)
if not trigger_files:
    print(f"  ! 这次改动没碰规则、agent、hook、门禁、脚本与实现，这一格无对象可判（改动范围 {len(changed_files)} 个文件，基准 {base}）")
    sys.exit(77)

marker_line = "<!-- knowledge-sync -->"
record_name = re.compile(r"^research/prompts/[^/]+-sync\.md$")
records = []
fresh_records = set()
uncommitted_records = set()
unmarked_candidates = []
for path in changed_files:
    if not record_name.search(path) or not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    if any(line.strip() == marker_line for line in text.split("\n")):
        records.append((path, text))
        # 这一轮新写的记录：基准那一版里没有这个文件。只是被改过的旧记录不算，它早先的行点名的是更早一轮改的载体。
        # 不用 git diff --name-status：它看不见未跟踪的新文件，而这一轮刚写出来还没 add 的记录正是那一种。
        exists_at_base = subprocess.run(
            ["git", "cat-file", "-e", f"{base}:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_base:
            fresh_records.add(path)
        # ⑤ 的基准是 HEAD，不是 GATE_BASE：它问这一次提交要带上的记录带没带原始材料；
        # 拿 GATE_BASE 会连上一个提交里写好、已经冻结的记录一起判（gate-ok 不存在时基准退回 HEAD~1）。
        exists_at_head = subprocess.run(
            ["git", "cat-file", "-e", f"HEAD:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_head:
            uncommitted_records.add(path)
    else:
        unmarked_candidates.append(path)

fence_open = re.compile(r"^ {0,3}(`{3,}|~{3,})")
heading = re.compile(r"^#{1,2}\s")


def second_level_sections(text):
    """二级标题整行 → 小节体 [(行号, 行, 是否在代码围栏里)]；同名标题只认第一次，代码围栏里的 # 行不算标题。"""
    sections = {}
    current_body = None
    open_fence = None
    for line_number, line in enumerate(text.split("\n"), 1):
        if open_fence is None:
            fence_match = fence_open.match(line)
            if fence_match:
                open_fence = fence_match.group(1)
                if current_body is not None:
                    current_body.append((line_number, line, True))
                continue
            if heading.match(line):
                title = line.rstrip()
                if title.startswith("## ") and title not in sections:
                    sections[title] = []
                    current_body = sections[title]
                else:
                    current_body = None
                continue
            if current_body is not None:
                current_body.append((line_number, line, False))
        else:
            if re.match(r"^ {0,3}" + re.escape(open_fence[0]) + "{" + str(len(open_fence)) + r",}\s*$", line):
                open_fence = None
            if current_body is not None:
                current_body.append((line_number, line, True))
    return sections


def table_cells(row):
    inner = row.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|") and not inner.endswith("\\|"):
        inner = inner[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", inner)]


carrier_form = re.compile(r"^(?P<path>\S(?:.*\S)?):(?P<line>[1-9][0-9]*)$")
separator_cell = re.compile(r"^:?-{3,}:?$")

missing_triggers = [path for path in trigger_files if not any(path in text for _record_path, text in records)]
search_problems = []
table_problems = []
carrier_problems = []
disposition_start_problems = []
empty_reason_problems = []
out_of_range_problems = []
disposition_counts = {"改了": 0, "补了": 0, "不改": 0}
evidence_problems = []
evidence_rows_total = 0
evidence_record_count = 0


def tables_in(section_body):
    """小节体里的几张表：连着的表格行算一张，代码围栏里的行不算。返回 [[(行号, 整行), …], …]。"""
    runs = []
    previous_was_table_line = False
    for line_number, line, fenced in section_body:
        is_table_line = (not fenced) and line.lstrip().startswith("|")
        if is_table_line and not previous_was_table_line:
            runs.append([])
        if is_table_line:
            runs[-1].append((line_number, line))
        previous_was_table_line = is_table_line
    return runs


def newline_count(path):
    """`wc -l` 报的数：数换行符，不补末尾。不以换行结尾的文件两者差 1，取一个口径，别让写记录的人猜。"""
    with open(path, "rb") as handle:
        return handle.read().count(b"\n")


for record_path, text in records:
    sections = second_level_sections(text)
    if "## 搜索" not in sections:
        search_problems.append(f"{record_path}：没有「## 搜索」小节")
    elif not any(line.strip() for _number, line, _fenced in sections["## 搜索"]):
        search_problems.append(f"{record_path}：「## 搜索」小节是空的")
    # ⑤ 原始证据只对还没进 HEAD 的记录判：已提交的记录改不得，它点名的材料也归档了。
    if record_path in uncommitted_records:
        evidence_record_count += 1
        if "## 原始证据" not in sections:
            evidence_problems.append(f"{record_path}：没有「## 原始证据」小节")
        else:
            evidence_tables = tables_in(sections["## 原始证据"])
            if not evidence_tables:
                evidence_problems.append(f"{record_path}：「## 原始证据」小节里没有表")
            elif len(evidence_tables) > 1:
                evidence_problems.append(f"{record_path}：「## 原始证据」小节里有 {len(evidence_tables)} 张表，只许一张（第二张从第 {evidence_tables[1][0][0]} 行起）")
            else:
                evidence_rows = evidence_tables[0]
                evidence_header_number, evidence_header_line = evidence_rows[0]
                if table_cells(evidence_header_line) != ["材料", "路径", "行数"]:
                    evidence_problems.append(f"{record_path}:{evidence_header_number}：「## 原始证据」的表头不是 | 材料 | 路径 | 行数 |，实际是 {evidence_header_line.strip()}")
                elif (len(evidence_rows) < 2 or len(table_cells(evidence_rows[1][1])) != 3
                      or not all(separator_cell.match(cell) for cell in table_cells(evidence_rows[1][1]))):
                    evidence_problems.append(f"{record_path}:{evidence_header_number + 1}：「## 原始证据」表头下面一行不是三格的分隔行 |---|---|---|")
                elif len(evidence_rows) < 3:
                    evidence_problems.append(f"{record_path}:{evidence_header_number}：「## 原始证据」的表一行都没有——判错一行之后拿不出任何原始材料，正是这一条要拦的")
                else:
                    for row_number, row_line in evidence_rows[2:]:
                        cells = table_cells(row_line)
                        location = f"{record_path}:{row_number}"
                        evidence_rows_total += 1
                        if len(cells) != 3:
                            evidence_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
                            continue
                        material, evidence_path, declared_lines = cells
                        if len(evidence_path) >= 2 and evidence_path.startswith("`") and evidence_path.endswith("`"):
                            evidence_path = evidence_path[1:-1].strip()
                        if not material:
                            evidence_problems.append(f"{location}：材料格是空的，要写这份材料是什么（事实表 / 候选表 / 逐行判定报告）")
                        if evidence_path.startswith(("/", "./", "../")) or "/../" in evidence_path:
                            evidence_problems.append(f"{location}：路径不是从仓库根起写的：{evidence_path}")
                        elif not os.path.isfile(evidence_path):
                            evidence_problems.append(f"{location}：{evidence_path} 不在仓里——原始材料没入库，判错之后无从复盘")
                        elif not re.fullmatch(r"[0-9]+", declared_lines):
                            evidence_problems.append(f"{location}：行数格不是十进制整数：{declared_lines or '（空）'}")
                        else:
                            actual_lines = newline_count(evidence_path)
                            if actual_lines != int(declared_lines):
                                evidence_problems.append(f"{location}：行数写 {declared_lines}，{evidence_path} 实际 {actual_lines}（wc -l）")
    if "## 命中处置" not in sections:
        table_problems.append(f"{record_path}：没有「## 命中处置」小节")
        continue
    table_runs = tables_in(sections["## 命中处置"])
    if not table_runs:
        table_problems.append(f"{record_path}：「## 命中处置」小节里没有表")
        continue
    if len(table_runs) > 1:
        table_problems.append(f"{record_path}：「## 命中处置」小节里有 {len(table_runs)} 张表，只许一张（第二张从第 {table_runs[1][0][0]} 行起）")
        continue
    table_rows = table_runs[0]
    header_number, header_line = table_rows[0]
    if table_cells(header_line) != ["载体", "原句", "处置"]:
        table_problems.append(f"{record_path}:{header_number}：「## 命中处置」的表头不是 | 载体 | 原句 | 处置 |，实际是 {header_line.strip()}")
        continue
    if len(table_rows) < 2 or len(table_cells(table_rows[1][1])) != 3 or not all(separator_cell.match(cell) for cell in table_cells(table_rows[1][1])):
        table_problems.append(f"{record_path}:{header_number + 1}：表头下面一行不是三格的分隔行 |---|---|---|")
        continue
    for row_number, row_line in table_rows[2:]:
        cells = table_cells(row_line)
        location = f"{record_path}:{row_number}"
        if len(cells) != 3:
            table_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
            continue
        carrier_text, _original_sentence, disposition = cells
        if len(carrier_text) >= 2 and carrier_text.startswith("`") and carrier_text.endswith("`"):
            carrier_text = carrier_text[1:-1].strip()
        carrier_match = carrier_form.match(carrier_text)
        carrier_path = None
        if not carrier_match:
            carrier_problems.append(f"{location}：载体格不是「路径:行号」：{carrier_text}")
        else:
            candidate_path = carrier_match.group("path")
            if candidate_path.startswith(("/", "./", "../")) or "/../" in candidate_path:
                carrier_problems.append(f"{location}：载体路径不是从仓库根起写的：{candidate_path}")
            elif not os.path.exists(candidate_path) and candidate_path not in changed_set:
                carrier_problems.append(f"{location}：载体路径 {candidate_path} 在仓里不存在，也不在改动范围里")
            else:
                carrier_path = candidate_path
        if disposition.startswith("改了") or disposition.startswith("补了"):
            disposition_counts[disposition[:2]] += 1
            # 这一轮写的处置行才判：新写的记录整份都是；旧记录只判这一轮新加的那几行（早先的行点名的是更早一轮改的载体）
            written_this_round = (os.environ.get("KNOWLEDGE_SYNC_BREAK", "") == "old-rows-rejudged"
                                  or record_path in fresh_records or row_line in added_lines_by_record.get(record_path, set()))
            if (written_this_round
                    and carrier_path is not None and carrier_path not in changed_set):
                out_of_range_problems.append(f"{location}：处置写「{disposition[:2]}」，载体 {carrier_path} 不在改动范围里")
        elif disposition.startswith("不改："):
            disposition_counts["不改"] += 1
            if not disposition[len("不改："):].strip():
                empty_reason_problems.append(f"{location}：「不改：」后面没写理由")
        else:
            disposition_start_problems.append(f"{location}：处置开头不合法：{disposition or '（空）'}")

sync_step = ("阶段同步怎么做：阶段任务结束时派 sweep 做阶段同步（sweep 定义里的第四种活），主 agent 判完每处命中写 "
             f"research/prompts/<阶段>-sync.md，格式见 .claude/gate.d/{STAGE_NAME} 文件头「knowledge-sync」那一段。")
failed = False
if missing_triggers:
    failed = True
    print(f"  ✗ {len(missing_triggers)} 个触发文件没在任何同步记录里点名（改动范围里带 <!-- knowledge-sync --> 的 research/prompts/*-sync.md 共 {len(records)} 份，一份都没按路径提到它）：")
    for path in missing_triggers:
        print(f"      {path}")
    if unmarked_candidates:
        print("     这几份文件名像同步记录，但没有只写 <!-- knowledge-sync --> 的那一行，不算：")
        for path in unmarked_candidates:
            print(f"      {path}")
    print("     → 怎么办：这一阶段还没做知识同步就先做；做过的，把上面每个文件按路径写全补进同步记录的「触发文件」一行。")
if search_problems:
    failed = True
    print(f"  ✗ {len(search_problems)} 份同步记录缺「## 搜索」或它是空的：")
    for entry in search_problems:
        print(f"      {entry}")
    print("     → 怎么办：加「## 搜索」小节（标题逐字写、后面不加字），写回扫用的每条命令与它的命中计数。")
if table_problems:
    failed = True
    print(f"  ✗ {len(table_problems)} 处「## 命中处置」的表写坏了：")
    for entry in table_problems:
        print(f"      {entry}")
    print("     → 怎么办：「## 命中处置」下面恰好一张表，表头逐字是 | 载体 | 原句 | 处置 |，第二行 |---|---|---|；每行三格，原句里的 | 写成 \\|；没有命中也留表头（0 行）。")
if carrier_problems:
    failed = True
    print(f"  ✗ {len(carrier_problems)} 行载体格不是仓库根起的「路径:行号」或指不到文件：")
    for entry in carrier_problems:
        print(f"      {entry}")
    print("     → 怎么办：载体格写仓库根起的 路径:行号（例 .claude/agent-common.md:120），路径要在仓里现存；别写成相对 research/prompts/ 的路径。")
if disposition_start_problems:
    failed = True
    print(f"  ✗ {len(disposition_start_problems)} 行处置开头不合法（只认「改了」「补了」「不改：」）：")
    for entry in disposition_start_problems:
        print(f"      {entry}")
    print("     → 怎么办：每处命中先判完再写：改了写「改了：…」，新加的写「补了：…」，不动写「不改：理由」；别写「待定」「看过」。")
if empty_reason_problems:
    failed = True
    print(f"  ✗ {len(empty_reason_problems)} 行「不改：」没写理由：")
    for entry in empty_reason_problems:
        print(f"      {entry}")
    print("     → 怎么办：「不改：」后面写不改的理由，例：冻结证据 / 说的是那一次发生的事 / 同一个词、不同的事。")
if evidence_problems:
    failed = True
    print(f"  ✗ {len(evidence_problems)} 处「## 原始证据」写坏了，或点名的材料不在仓里、行数对不上：")
    for entry in evidence_problems:
        print(f"      {entry}")
    print("     → 怎么办：这一轮新写的同步记录要有「## 原始证据」小节，小节里恰好一张表，表头逐字 | 材料 | 路径 | 行数 |，至少一行；")
    print("               把这一阶段的事实表、候选表与每份逐行判定报告从 /tmp 挪进 research/prompts/ 跟着记录一起提交，行数写 wc -l 报的数。")
if out_of_range_problems:
    failed = True
    print(f"  ✗ {len(out_of_range_problems)} 行处置写「改了」「补了」而载体不在改动范围里（记录说改了，文件没动）：")
    for entry in out_of_range_problems:
        print(f"      {entry}")
    print("     → 怎么办：去把那一处真的改了；判下来不该改的，处置写成「不改：理由」；也对一遍载体路径是不是写成了别的文件。")
if failed:
    print(f"     → {sync_step}")
    sys.exit(1)
hit_rows = sum(disposition_counts.values())
print(f"  ✓ 触发文件 {len(trigger_files)} 个都在同步记录里点名（同步记录 {len(records)} 份；命中 {hit_rows} 行："
      f"改了 {disposition_counts['改了']} / 补了 {disposition_counts['补了']} / 不改 {disposition_counts['不改']}；基准 {base}）；"
      f"这一次提交新带的同步记录 {evidence_record_count} 份，点名的 {evidence_rows_total} 份原始材料都在仓里、行数对得上；"
      f"改动范围里另有 {outside_trigger_count} 个文件不在触发范围")
PY
}

# ── agent-def-adversarial-review ────────────────────────────────────────────
check_agent_def_adversarial_review() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  check_review_pending_table || return 1
  local definitions verdicts definition why verdict_count
  local -a exempted=() missing=() stale=() unhashed=()
  local count=0
  definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed_since_gate" || true)"
  if [[ -z "$definitions" ]]; then
    echo "  ! 这次改动没碰 .claude/agents/、.claude/agent-common.md、.claude/main-agent.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  verdicts="$(new_verdict_paths)"
  # 豁免表的格式与认法在 user_exemption_reason
  local exempt_table=".claude/agent-def-review-exempt"
  while IFS= read -r definition; do
    [[ -z "$definition" ]] && continue
    count=$((count + 1))
    # 判法与 crates-adversarial-review 同一份：review_judge_named 判点名与快照，又改过的与没点名的先查豁免表
    review_judge_named "$definition" "$verdicts"
    case "$review_outcome" in
      named) continue ;;
      unhashed) unhashed+=("$definition"); continue ;;
    esac
    why="$(user_exemption_reason "$exempt_table" "$definition")"
    if [[ -n "$why" ]]; then exempted+=("$definition：$why"); continue; fi
    if [[ "$review_outcome" == stale ]]; then stale+=("$definition（$review_outcome_note）"); continue; fi
    missing+=("$definition$(review_mention_note "$definition" "$verdicts")")
  done <<<"$definitions"
  if ((${#stale[@]})); then
    review_report_stale "改过的定义与共用约束" "$exempt_table" "${stale[@]}"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 改过的定义与共用约束里 ${#missing[@]} 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没在「$REVIEWED_SECTION_HEADING」一节里按路径列出它；正文别处提到不算，节里写了没攻、没判这类否定词的行也不算；被改过的旧判决不算）："   # gate-lint:summary
    printf '      %s\n' "${missing[@]}"   # gate-lint:detail
    echo "     → 怎么办：走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，"
    echo "               在判决里写一节，标题一行逐字是「$REVIEWED_SECTION_HEADING」（二级标题），节里一行一个，逐个按路径列出改过的每一份定义"
    echo "               （节到下一个一、二级标题为止；打中的写回定义再改一轮）："
    echo "                 $REVIEWED_SECTION_HEADING"
    echo "                 - .claude/agents/<名>.md：<这一轮判了它的什么>"
    echo "               点名写进新的判决，不往旧判决里补。"
    return 1
  fi
  ((${#stale[@]})) && return 1
  verdict_count="$(grep -c . <<<"$verdicts" || true)"
  echo "  ✓ 改过的定义与共用约束 ${count} 份都有去向（新写的判决文件 ${verdict_count} 份，另有 ${#exempted[@]} 份按 $exempt_table 豁免，基准 $gate_base）"
  if ((${#exempted[@]})); then
    echo "    豁免的 ${#exempted[@]} 份（按内容哈希认，那份文件再改一个字就不再豁免）："
    printf '      %s\n' "${exempted[@]}"
  fi
  review_report_snapshot_coverage "那一轮没有开工快照、或快照清单里没有它的行" ${unhashed[@]+"${unhashed[@]}"}
  report_review_pending_in_use "$verdicts"
}

# ── milestone-close-owed ────────────────────────────────────────────────────
# 判据在文件头「milestone-close-owed」；判法与弄坏开关都在 $MILESTONE_CLOSE_CHECK，这里只调它的 --closed、照它的退出码（0 绿、1 红、77 无对象、2 用法错按红）
check_milestone_close_owed() {
  if [[ ! -f "$MILESTONE_CLOSE_CHECK" ]]; then
    echo "  ✗ 判法 $MILESTONE_CLOSE_CHECK 不在，判不了收口的里程碑还有没有点名它的开着的账"
    echo "     → 怎么办：那份脚本随仓走，从 git 找回再跑；判法只在那一份，别在这一格里另抄"
    return 1
  fi
  python3 "$MILESTONE_CLOSE_CHECK" --closed "$ROOT"
}

# ── deferred-items-registered ───────────────────────────────────────────────
# 判据在文件头「deferred-items-registered」；词表是上面的 DEFERRAL_WORDS，弄坏开关 DEFERRED_ITEMS_BREAK 写在文件头那一段
check_deferred_items_registered() {
  ((inside_git)) || { not_git_repository; return; }
  load_added_records_directory_lines || {
    echo "  ✗ 取不到 records/ 下这一轮新增了哪些行（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到新增行时延后项登没登记没法判，不是通过。"
    return 1
  }
  # 新增行经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - <(printf '%s\n' "$added_records_directory_lines") "$DEFERRAL_WORDS" "${DEFERRED_ITEMS_BREAK:-}" "$DEFERRAL_REGISTRY_TABLE" "$OWED_LIBRARY" <<'PY'
import importlib.util, os, re, sys

added_file, words_text, broken, REGISTRY_TABLE, OWED_LIBRARY = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]
sys.dont_write_bytecode = True
DEFERRAL_WORDS = [word for word in words_text.split() if word]
CHECKS_OWED = ".claude/kb/checks-owed.md"
# 两份拆分提案：欠账那份（开着的节与新欠账）先找，没有再找历史那份（一字不改，做完的节照旧指得到）；两份都是登记处
SPLIT_PROPOSAL_LEDGER = "records/2026-10-23-subagent拆分提案.md"
SPLIT_PROPOSAL = "records/2026-09-16-subagent拆分提案.md"
BOTH_PROPOSALS = f"{SPLIT_PROPOSAL_LEDGER} 与 {SPLIT_PROPOSAL}"
# 「延后项」是在说「延后的那几件」这个话题（「延后项无去向 8/54」），不是在延后什么；「进延后项」「记进延后项」是把这一件延后，照旧算
DEFERRAL_NOUN = re.compile(r"延后项" if "into-noun-skipped" in broken else r"(?<!进)延后项")
HEADING = re.compile(r"^ {0,3}#{1,6}(?:[ \t]|$)")
# 同一个分句里带「N/M」的「延后」是在报统计数（「延后当时就登进欠账表的 6/54」），也不是在延后什么；分句按逗号、句号、分号、冒号与表格竖线切
CLAUSE_BREAK = re.compile(r"([，。；;：:|])")
RATIO = re.compile(r"[0-9]+\s*/\s*[0-9]+")


def without_topic_mentions(text):
    """去掉只在点话题的「延后」：名词「延后项」，与和统计数同在一个分句里的「延后」。"""
    if "statistics-counted" not in broken:
        text = "".join(piece.replace("延后", "") if RATIO.search(piece) else piece for piece in CLAUSE_BREAK.split(text))
    return text if "deferral-noun-counted" in broken else DEFERRAL_NOUN.sub("", text)


CIRCLED = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳㉑㉒㉓㉔㉕㉖㉗㉘㉙㉚㉛㉜㉝㉞㉟㊱㊲㊳㊴㊵㊶㊷㊸㊹㊺㊻㊼㊽㊾㊿"
CHINESE_DIGITS = {"零": 0, "〇": 0, "一": 1, "二": 2, "两": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
C_NUMBER = re.compile(r"(?<![A-Za-z0-9_])C([0-9]+)(?![0-9])")
SECTION_ITEMS = re.compile(rf"第\s*(?P<section>[零〇一二两三四五六七八九十百]+|[0-9]+)\s*节(?:[^{CIRCLED}\n。；]{{0,16}}?(?P<items>[{CIRCLED}](?:\s*[、，,和与及]?\s*[{CIRCLED}])*))?")
# 行里没写「拆分提案」时只认节号后面紧跟行首序号的写法（「九十节 ㉟」「第九十节（行首序号 ②、③）」），别的文档的「第三节 … ①」不当成它
SECTION_ITEMS_BARE = re.compile(rf"(?<![零〇一二两三四五六七八九十百0-9])第?\s*(?P<section>[零〇一二两三四五六七八九十百]+|[0-9]+)\s*节\s*(?:[（(]\s*)?(?:行首序号\s*)?(?P<items>[{CIRCLED}](?:\s*[、，,和与及]?\s*[{CIRCLED}])*)")
FILE_LINE = re.compile(r"(?P<path>(?:\.{0,2}/)?[\w.\-一-鿿]+(?:/[\w.\-一-鿿]+)*\.[A-Za-z0-9]+):(?P<line>[0-9]+)(?:[-–](?P<end>[0-9]+))?")
FENCE = re.compile(r"^\s*(```|~~~)")


def chinese_number(text):
    if text.isdigit():
        return int(text)
    total, current = 0, 0
    for character in text:
        if character == "百":
            total, current = total + (current or 1) * 100, 0
        elif character == "十":
            total, current = total + (current or 1) * 10, 0
        else:
            current = CHINESE_DIGITS[character]
    return total + current


def outside_quotes(text):
    """去掉「…」（配平）与 `…` 里的字：引原话、引规则里的词不算这一行在说延后。"""
    kept, depth, in_code = [], 0, False
    for character in text:
        if character == "`" and depth == 0:
            in_code = not in_code
            continue
        if in_code:
            continue
        if character == "「":
            depth += 1
            continue
        if character == "」" and depth:
            depth -= 1
            continue
        if depth == 0:
            kept.append(character)
    return "".join(kept)


def read_lines(path):
    try:
        return open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        return None


owed_lines = read_lines(CHECKS_OWED)
registered_c, paid_c = set(), set()
if owed_lines is not None:
    # 开着与已还清的切法只在 lib-owed.py 一份（「已还清」整行标题之前开着、之后到「## 历史版本」已还清）；认不出那一行标题时全文的行都当开着的（那份库的 open_names）
    owed_specification = importlib.util.spec_from_file_location("lib_owed", OWED_LIBRARY)
    owed_library = importlib.util.module_from_spec(owed_specification)
    owed_specification.loader.exec_module(owed_library)
    owed_table = owed_library.read_owed_table(CHECKS_OWED)
    registered_c = {int(name[1:]) for name in owed_table.open_names}
    paid_c = {int(name[1:]) for name in owed_table.paid_names}
    if "paid-c-numbers-accepted" in broken:
        registered_c |= paid_c
def section_items(lines):
    """→ {节号: {行首序号}}：「## <节号>、」标题起一节，节里行首是「| ③ |」的行；同一个节号出现两次的并成一节。"""
    items = {}
    section = None
    for row in lines or ():
        heading = re.match(r"^##\s+([零〇一二两三四五六七八九十百]+)、", row)
        if heading:
            section = chinese_number(heading.group(1))
            items.setdefault(section, set())
            continue
        if row.startswith("## "):
            section = None
            continue
        item = re.match(rf"^\|\s*([{CIRCLED}])\s*\|", row)
        if item and section is not None:
            items[section].add(item.group(1))
    return items


proposal_lines = read_lines(SPLIT_PROPOSAL)
proposal_items = section_items(proposal_lines)
# 欠账那份读不到就照实报（不退回只认历史那份）；ledger-ignored 不读它，ledger-missing-tolerated 读不到当空的、不报
ledger_lines = [] if "ledger-ignored" in broken else read_lines(SPLIT_PROPOSAL_LEDGER)
if ledger_lines is None and "ledger-missing-tolerated" in broken:
    ledger_lines = []
ledger_items = section_items(ledger_lines)
# 登记处表：一份一行「<从仓根起的路径><TAB><为什么它本身就是登记处>」；路径要在树里、理由 8 个字以上，写坏的那一行不认、逐行报
registries, registry_problems = [], []
registry_rows = read_lines(REGISTRY_TABLE) if os.path.isfile(REGISTRY_TABLE) else []
for index, row in enumerate(registry_rows or [], start=1):
    if not row.strip() or row.startswith("#"):
        continue
    fields = row.split("\t")
    if "registry-rows-unchecked" in broken:
        registries.append(fields[0].strip())
        continue
    if len(fields) != 2:
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：不是「路径<TAB>理由」两段（读出 {len(fields)} 段）")
    elif not os.path.isfile(fields[0]):
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：{fields[0]} 不在树里（写从仓根起的路径）")
    elif len(fields[1].strip()) < 8:
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：{fields[0]} 的理由不到 8 个字（写为什么这份文件本身就是登记处）")
    elif fields[0] in registries:
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：{fields[0]} 登记了两次")
    else:
        registries.append(fields[0])


def registry_named(text):
    """表格行里，带延后类词的那一格之后的某一格点名的登记处（去向格）；前后不接路径里的字，`staged-gate.md.old` 不算 staged-gate.md。
    不是表格行、或登记处只在延后类词那一格与它之前（被延后的那件事本身）的，不算。"""
    if "registry-table-ignored" in broken:
        return []
    cells = text.strip().strip("|").split("|") if text.lstrip().startswith("|") else [text]
    if "registry-cell-unchecked" in broken:
        later = text
    elif len(cells) < 2:
        return []
    else:
        word_cell = next((index for index, cell in enumerate(cells)
                          if any(word in without_topic_mentions(outside_quotes(cell)) for word in DEFERRAL_WORDS)), None)
        if word_cell is None:
            return []
        later = "|".join(cells[word_cell + 1:])
    return [path for path in registries
            if re.search(rf"(?<![\w./\-一-鿿]){re.escape(path)}(?![\w./\-一-鿿])", later)]


def pointers(text, own_path, own_number):
    """→ [(写法, 指得到吗, 原文, 指不到的原因)]；own_path、own_number 是这一行自己（行号找不到时 own_number 是 None）。"""
    found = []
    for match in C_NUMBER.finditer(text):
        number = int(match.group(1))
        ok = owed_lines is not None and number in registered_c
        if ok:
            why = ""
        elif number in paid_c:
            why = f"C{number} 在 {CHECKS_OWED} 的「已还清」表里，已还清的不算登记（延后的事另登一行开着的）"
        else:
            why = f"{CHECKS_OWED} 里没有「| C{number} |」这一行"
        found.append(("C 号", ok, match.group(0), why))
    for match in (SECTION_ITEMS if "拆分提案" in text else SECTION_ITEMS_BARE).finditer(text):
        section = chinese_number(match.group("section"))
        # 先找欠账那份，没有再找历史那份：序号在两份同号的节里任一份有就算指得到（历史那份里重号的节在欠账那份里改了号）
        sources = (ledger_items,) if "history-fallback-dropped" in broken else (ledger_items, proposal_items)
        present = [items_by_section[section] for items_by_section in sources if section in items_by_section]
        items = re.findall(f"[{CIRCLED}]", match.group("items") or "")
        if not present:
            found.append(("拆分提案节号", False, f"第{match.group('section')}节", f"{BOTH_PROPOSALS} 里都没有第 {section} 节"))
        elif not items:
            rows = present[0]   # 只写节号的：欠账那份有这一节就看它，没有看历史那份
            ok = not rows
            found.append(("拆分提案节号", ok, f"第{match.group('section')}节", "" if ok else f"那一节有行首序号（{len(rows)} 行），要写是哪一行"))
        for item in items if present else ():
            ok = any(item in rows for rows in present)
            found.append(("拆分提案节号与序号", ok, f"第{match.group('section')}节 {item}", "" if ok else f"{BOTH_PROPOSALS} 的第 {section} 节里都没有行首是「{item}」的一行"))
    for match in FILE_LINE.finditer(text):
        path, number = match.group("path"), int(match.group("line"))
        last = int(match.group("end")) if match.group("end") else number
        target = os.path.normpath(path)
        # 指到这一行自己的不算登记（弄坏开关 self-pointer-accepted 下不判这一样）
        if (target == os.path.normpath(own_path) and own_number is not None and number <= own_number <= max(number, last)
                and "self-pointer-accepted" not in broken):
            found.append(("文件:行", False, match.group(0), "指的是这一行自己"))
            continue
        # records/ 下只认两份拆分提案：调度记录与别的记录是写延后的地方，不是登记处（弄坏开关 records-pointers-accepted 下不判这一样）
        if (target.startswith("records/") and target not in (SPLIT_PROPOSAL_LEDGER, SPLIT_PROPOSAL)
                and "records-pointers-accepted" not in broken):
            found.append(("文件:行", False, match.group(0), f"records/ 下只认两份拆分提案（{BOTH_PROPOSALS}），别的记录不是登记处"))
            continue
        lines = read_lines(path) if os.path.isfile(path) else None
        ok = lines is not None and 1 <= number <= len(lines) - (1 if lines and lines[-1] == "" else 0)
        why = "" if ok else (f"{path} 不在树里（只写文件名的要写从仓根起的路径）" if lines is None else f"{path} 只有 {len(lines) - (1 if lines[-1] == '' else 0)} 行")
        found.append(("文件:行", ok, match.group(0), why))
    for path in registry_named(text):
        found.append(("登记处", True, path, ""))
    return found


with open(added_file, encoding="utf-8", errors="replace") as handle:
    added = [line.rstrip("\n").split("\t", 1) for line in handle if "\t" in line]
file_cache, used, judged, problems, scanned = {}, {}, [], [], 0
skipped_quoted = skipped_fenced = skipped_registry = skipped_noun = skipped_heading = 0
for path, text in added:
    if not path.endswith(".md"):
        continue
    scanned += 1
    visible = text if "quotes-counted" in broken else outside_quotes(text)
    if "noun-line-skipped" in broken and DEFERRAL_NOUN.search(visible):
        skipped_noun += 1
        continue
    verbs = without_topic_mentions(visible)
    words = [word for word in DEFERRAL_WORDS if word in verbs]
    if not words:
        if any(word in visible for word in DEFERRAL_WORDS):
            skipped_noun += 1
        elif any(word in text for word in DEFERRAL_WORDS):
            skipped_quoted += 1
        continue
    if ((path == SPLIT_PROPOSAL and "registry-rows-judged" not in broken)
            or (path == SPLIT_PROPOSAL_LEDGER and "ledger-rows-judged" not in broken)):
        skipped_registry += 1   # 两份拆分提案本身是登记处（编号小节），它们里面的行不判
        continue
    lines = file_cache.setdefault(path, read_lines(path) or [])
    start = used.get((path, text), 0)
    number = next((index + 1 for index in range(start, len(lines)) if lines[index] == text), None)
    used[(path, text)] = number if number else start
    if number is not None:
        in_fence = False
        for index in range(number - 1):
            if FENCE.match(lines[index]):
                in_fence = not in_fence
        if in_fence and "fenced-lines-judged" not in broken:
            skipped_fenced += 1
            continue
    if HEADING.match(text) and "headings-judged" not in broken:
        skipped_heading += 1   # 标题是在点话题（「### 延后项的登记」），延后什么写在节里的行上
        continue
    where = f"{path}:{number}" if number else f"{path}（行号找不到）"
    judged.append(where)
    found = pointers(text, path, number)
    if any(ok or "pointers-unresolved-accepted" in broken for _kind, ok, _text, _why in found):
        continue
    if found:
        detail = "；".join(f"{kind}「{written}」指不到：{why}" for kind, _ok, written, why in found)
        problems.append(f"{where}：带「{'」「'.join(words)}」，写的登记指不到——{detail}")
    else:
        problems.append(f"{where}：带「{'」「'.join(words)}」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处")
if owed_lines is None or proposal_lines is None or ledger_lines is None:
    missing = [path for path, lines in ((CHECKS_OWED, owed_lines), (SPLIT_PROPOSAL_LEDGER, ledger_lines), (SPLIT_PROPOSAL, proposal_lines)) if lines is None]
    print(f"  ✗ 读不到 {'、'.join(missing)}：C 号与拆分提案的节号指不指得到判不了")
    print("     → 怎么办：从 git 找回那份文件（git log --diff-filter=D --name-only -- <路径>），修好再单跑这一格（bash .claude/gate.d/doc-process-records.sh --check deferred-items-registered）")
    sys.exit(1)
if registry_problems:
    for problem in registry_problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 登记处表 {REGISTRY_TABLE} 里 {len(registry_problems)} 行写坏了，这几行不当登记处认")
    print(f"     → 怎么办：照 {REGISTRY_TABLE} 文件头写成「从仓根起的路径<TAB>为什么它本身就是登记处（8 个字以上）」，一份一行、文件要在树里；不再是登记处的整行删掉")
skipped_counts = (skipped_quoted, skipped_fenced, skipped_registry, skipped_noun, skipped_heading)
skipped_text = (f"；不判的：词只在「」或反引号里的 {skipped_quoted} 行、围栏里的 {skipped_fenced} 行、拆分提案（登记处本身）里的 {skipped_registry} 行、"
                f"「延后」只在点话题的（名词「延后项」、统计数）{skipped_noun} 行、标题 {skipped_heading} 行"
                if any(skipped_counts) else "")
if not judged:
    print(f"  ! 这次改动里 records/ 下新增的行（{scanned} 行）没有带延后类词（{'、'.join(DEFERRAL_WORDS)}）的，这一格无对象可判{skipped_text}")
    sys.exit(1 if registry_problems else 77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ records/ 下这次新增的带延后类词的行里 {len(problems)} 行没登记去向（判了 {len(judged)} 行）")
    print("     → 怎么办：照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行"
          f"（.claude/kb/checks-owed.md 开着的 C 号、{SPLIT_PROPOSAL_LEDGER} 的「第<节号>节 <行首序号>」（新欠账登这份；{SPLIT_PROPOSAL} 里已有的节照旧认）、"
          "或从仓根起的 <路径>:<行号>（records/ 下只认两份拆分提案，不指这一行自己）），先在那里登一行再写延后；"
          f"去向是提交时那一套流程的，同一行点名 {REGISTRY_TABLE} 里登记的那份文件（写全路径）")
    sys.exit(1)
if registry_problems:
    sys.exit(1)
print(f"  ✓ records/ 下这次新增的带延后类词的行：判了 {len(judged)} 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或登记处；records/ 新增的 .md 行共 {scanned} 行）{skipped_text}")
PY
}

# ── ledger-section-numbers ──────────────────────────────────────────────────
# 判据在文件头「ledger-section-numbers」；只看树，不用改动范围
check_ledger_section_numbers() {
  python3 - "$LEDGER_PROPOSAL" "$LEDGER_DUPLICATE_SECTIONS" "${LEDGER_SECTION_NUMBERS_BREAK:-}" <<'PY'
import collections, re, sys

path, duplicates_text, broken = sys.argv[1], sys.argv[2], sys.argv[3]
allowed = {}   # 历史上已经重号的节：节号 → 标题与索引行各该有几处
for entry in duplicates_text.split():
    section, _, count = entry.partition(":")
    allowed[section] = int(count)
try:
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
except FileNotFoundError:
    print(f"  ! 树里没有 {path}，这一格无对象可判")
    sys.exit(77)
except (OSError, UnicodeDecodeError) as error:
    print(f"  ✗ 读不了 {path}（{error}）：节号重不重、索引全不全这一次没判")
    print("     → 怎么办：修好那份文件的编码或权限再单跑这一格（bash .claude/gate.d/doc-process-records.sh --check ledger-section-numbers）；读不了不是没有重号。")
    sys.exit(1)

NUMERALS = "零〇一二两三四五六七八九十百千"
HEADING = re.compile(rf"^##\s+([{NUMERALS}]+)、")
INDEX_ROW = re.compile(rf"^\|\s*([{NUMERALS}]+)")   # 第一格以节号起头（「一百（历史那份里的第二个九十七）」只取「一百」）；表头「节号」与分隔行不是
FENCE = re.compile(r"^ {0,3}(```|~~~)")
headings = collections.defaultdict(list)     # 节号 → 标题所在的行号
index_rows = collections.defaultdict(list)   # 节号 → 索引表里的行号
index_seen = in_index = in_fence = False
for number, line in enumerate(lines, 1):
    if FENCE.match(line):
        in_fence = not in_fence
        continue
    if in_fence:
        continue
    if line.startswith("## "):
        in_index = line.rstrip() == "## 索引"
        index_seen = index_seen or in_index
        found = HEADING.match(line)
        if found:
            headings[found.group(1)].append(number)
        continue
    if in_index:
        found = INDEX_ROW.match(line)
        if found:
            index_rows[found.group(1)].append(number)


def at(numbers):
    return "第 " + "、".join(map(str, numbers)) + " 行"


problems = []
if not index_seen:
    problems.append("没有「## 索引」一节（文件开头那张一节一行的表）")
for section, numbers in headings.items():
    want = allowed.get(section, 1)
    if len(numbers) != want and "duplicate-headings-tolerated" not in broken:
        if section in allowed:
            problems.append(f"「## {section}、」标题 {len(numbers)} 处（{at(numbers)}），例外表 LEDGER_DUPLICATE_SECTIONS 登记的是 {want} 处")
        else:
            problems.append(f"「## {section}、」标题重号 {len(numbers)} 处（{at(numbers)}）")
    if index_seen and "index-unchecked" not in broken:
        rows = index_rows.get(section, [])
        if not rows:
            problems.append(f"节号 {section}（标题在{at(numbers)}）在索引表里没有一行")
        elif len(rows) != want:
            problems.append(f"节号 {section} 在索引表里有 {len(rows)} 行（{at(rows)}），该是 {want} 行")
if "index-unchecked" not in broken:
    for section, rows in index_rows.items():
        if section not in headings:
            problems.append(f"索引表{at(rows)}的节号 {section} 没有对应的「## {section}、」一节")
for section, want in allowed.items():
    if section not in headings:
        problems.append(f"例外表 LEDGER_DUPLICATE_SECTIONS 登记的 {section} 在这份里一节都没有（例外过时，删掉那一项）")

if problems:
    for problem in problems:
        print(f"  ✗ {path}：{problem}")   # gate-lint:detail
    print(f"  ✗ {path} 的节号与索引有 {len(problems)} 处对不上（{len(headings)} 个节号）")
    print("     → 怎么办：新登一节先在「## 索引」表插一行占号（scripts/kb/insert-row.py），取下一个没用过的节号再写标题；撞了号的改成下一个空号、索引表同步改；"
          "索引里少的补一行、多的删掉；历史上已经重号的节只登记在 LEDGER_DUPLICATE_SECTIONS（只缩不涨）")
    sys.exit(1)
heading_count = sum(len(numbers) for numbers in headings.values())
row_count = sum(len(rows) for rows in index_rows.values())
kept = "、".join(f"{section} {want} 处" for section, want in sorted(allowed.items()))
print(f"  ✓ {path}：{len(headings)} 个节号（标题 {heading_count} 处、索引表 {row_count} 行）不重号、各在索引表里一行；照例外表认的历史重号：{kept or '无'}")
PY
}

# ── 改动范围在父进程里按选中的格预取：各格在自己的子 shell 里跑，子 shell 里取的传不回父进程，也传不到别的格 ──
# 取不到的不在这里判红：状态留在 *_status 里，要用它的格调 load_* 时照样拿到失败，各自判红、各报各的出路。
if ((inside_git)); then
  for cell in "${STAGE_CELLS_SELECTED[@]}"; do
    case "$cell" in
      batch-scope) load_changed_since_head ;;
      verdict-names-local-samples) load_added_since_gate ;;
      crates-adversarial-review|agent-def-adversarial-review) load_changed_since_gate; load_added_since_gate ;;
      knowledge-sync) load_changed_since_gate; load_added_record_lines ;;
      deferred-items-registered) load_added_records_directory_lines ;;
    esac
  done
fi

stage_cells_run "$ROOT"
