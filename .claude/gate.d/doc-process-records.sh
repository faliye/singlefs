#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树）里的三方留痕与同步记录，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 三方论证与阶段同步有没有留下该留的记录：批次范围、判决点名、正文前提、轮次收尾、同步记录、里程碑收口时的欠账、欠账去向的写法、调度记录里延后项的登记、现行账本的节号、封存账本没被改
# gate-fixture: yes
# gate-category: 文档类
# gate-similar: doc-experiments.sh 它的 evidence-in-repo 格也用 gate 取法的改动范围、也读 research/prompts/，但判产物与依据落没落进仓（/tmp 路径、产物新不新），不判三方与同步留下的记录；并进来会让这道阶段各格判的对象之外多一种对象（research/results/ 的产物）
# gate-similar: doc-experiments.sh 它的 archive-past-rounds 格按与 knowledge-sync 格同一个基准删上一轮的实验记录，保留判决与 abandoned-rounds.tsv 供这里读；它判工作区里还留没留旧记录，不判记录写没写、写得全不全
# gate-similar: doc-experiments.sh 它的 verdict-false-named 格名字里也有 verdict，但判的是实验产物判决行里的 false 字段有没有被实验页点名，对象是 research/results/ 与实验页，不是三方判决
# gate-similar: code-tooling.sh 它的 change-range-single-source 格判各阶段取改动范围只经 changed-paths.sh 一份取法；这里是那份取法的使用者之一，不判别的阶段
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 的两项判被测代码的行为，这一道只判记录。
#
# gate-cell: batch-scope 这一批要碰的触发文件有没有登记进范围
# gate-cell: verdict-names-local-samples 新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）
# gate-cell: crates-adversarial-review crates/ 下全部代码（各包 src 与 tests 里的 .rs、crates/mutations.tsv）的改动有没有走过三方正反对抗推理
# gate-cell: code-round-snapshot 按路径点名了产品代码 crates 文件的新判决，有没有点名开工快照、快照有没有罩住点名的每份文件
# gate-cell: implementation-premise 三方论证材料有没有「实现今天的样子」
# gate-cell: abandoned-rounds 发过腿的三方轮次，要么有判决，要么登记撂下
# gate-cell: knowledge-sync 改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录
# gate-cell: agent-def-adversarial-review 改过的治理文件（定义与共用约束、规则、技能、钩子、门禁、scripts/ 下的脚本、两个服务）有没有走过三方审核
# gate-cell: milestone-close-owed 段表写了收口的里程碑，欠账表开着的那张表里没有一行的前置点名它
# gate-cell: owed-destination-form 欠账表开着的行，「去向：」之后开头是三种落点之一（里程碑文件、「里程碑<号>」、「未排」），落点点名的里程碑文件都在
# gate-cell: deferred-items-registered records/ 下这次新增的带延后类词的行，同一行写着指得到的登记（C 号、拆分提案的节号与行首序号、文件:行或登记处表里的流程文件）
# gate-cell: ledger-section-numbers 现行账本（拆分提案账本登记表里 current 那一份）的「## <节号>、」标题不重号，每个节号在「## 索引」表里恰好一行
# gate-cell: sealed-ledgers-unchanged 拆分提案账本登记表里封存的那几份，内容与登记的 sha256 相同（封存之后没被改）
# gate-cell: code-round-checklist 代码轮正文逐格点名了检查单的八格（第九格截止日之后、不在只判八格名单里的九格），每格底下写了答案；名单指的正文都在、比基准那一版没多出行
# 格结构经 lib/stage-cells.sh；每格的成功行报它查了多少项。
#
#   bash .claude/gate.d/doc-process-records.sh [项目根]                              十四格都跑
#   bash .claude/gate.d/doc-process-records.sh --list                                逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-process-records.sh --check <格名>[,<格名>…] [项目根]     只跑点名的格（例：材料员开工只跑 implementation-premise）
#
# 改动范围只经 scripts/gate/changed-paths.sh 取（基准怎么定见它的 gate_diff_base），每种取法这一趟只取一次、各格共用：
#   gate 取法（含工作区、暂存区与未跟踪）：verdict-names-local-samples、code-round-snapshot 只用其中新增的，crates-adversarial-review、knowledge-sync、agent-def-adversarial-review 用全部；
#   head 取法（基准 HEAD）：batch-scope；
#   gate_added_lines（同步记录这一轮新增的行）：knowledge-sync；gate_added_lines（records/ 下这一轮新增的 .md 行）：deferred-items-registered。
# 被判的根不是 git 仓时，用改动范围的七格退 77；implementation-premise、abandoned-rounds、milestone-close-owed、owed-destination-form 与 ledger-section-numbers 只看树，照判。
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
#      BATCH_SCOPE_BREAK=prune-command-misses-last  命令里漏掉最后一行，照它跑完这一格仍红
#        batch-scope-stale-red（输出里找不到钉命令全文的那条 want）
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
# 对象是 crates/ 下的全部代码（CRATES_REVIEW_OBJECTS）：各包 crates/*/src/ 里的 .rs（角色登记表 .claude/gate.d/crate-roles.tsv 里角色不是「产品」的包的 src
#   同样在内）、各包 crates/*/tests/ 下的 .rs、crates/mutations.tsv。这次改动里每一份都要在这一次改动新写的某份
#   research/prompts/*-main-verification.md 里按路径点名；
#   「点名」只认判决里「## 这一轮被判过的文件」一节底下的路径，正文别处提到同一个路径不算（判据与写法见下面「点名一节」）；
#   点名了而那一轮开工快照记的哈希与此刻的内容对不上的（判决之后又改过）也不算，判据在「点名一节」⑥。
# 只改了注释的 .rs 文件不走三方：判法是规范副本的 scripts/comment-only-change.py（去掉注释与空白之后代码词元与基准逐个相同，
#   文档注释里会编成 doctest 的代码块、文档注释与 // SAFETY: 注释的增删照代码算；上游 show-me-test 用同一份），摘出去的逐个报出来；判法判不了就红；crates/mutations.tsv 不是 .rs，照判。
# 存量没补完三方的非产品代码（角色不是「产品」的包的 src，下面「非产品代码」一段；各包 tests，crates/mutations.tsv）登记在过渡挂账表 CRATES_TRANSITION_TABLE，登记的路径这一格不要求点名；
#   豁免表只缩不涨、过渡挂账表的判据与弄坏开关见下面「豁免表与过渡挂账表」。
# 没有 crates/ 下要判的代码（或都只改了注释）：退 77。管不到的：点名了判得对不对；各包 src 里的 #[cfg(test)] 模块按路径分不出来，照判。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   CRATES_ADVERSARIAL_REVIEW_BREAK=test-code-skipped  非产品代码（角色不是「产品」的包的 src，各包 tests，crates/mutations.tsv）不算对象
#       crates-adversarial-review-test-code-only、crates-adversarial-review-test-code-green（期望退出 1，实测 77；期望退出 0，实测 77）；
#       crates-adversarial-review-test-code-transition-green、crates-adversarial-review-transition-experiment-apparatus-green（期望退出 0，实测 77）；
#       crates-adversarial-review-test-code-comment-only（输出里找不到「只改了注释的 2 个不走三方」）
#   CRATES_ADVERSARIAL_REVIEW_BREAK=exempt-hash-ignored  user_exemption_reason 不核内容哈希，豁免表里有这条路径就算豁免
#       crates-adversarial-review-exempt-stale-red（期望退出 1，实测 0）
#   CRATES_ADVERSARIAL_REVIEW_BREAK=comments-count-as-code  不摘只改注释的文件，照代码改动要点名
#       crates-adversarial-review-comment-only、crates-adversarial-review-test-code-comment-only（期望退出 77，实测 1）；
#       crates-adversarial-review-comment-and-code-red（输出里找不到「crates 改动里 2 个文件没有三方判决点名」）
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
#      出路是再开一轮（两张豁免表不再加行，见下面「豁免表与过渡挂账表」）。
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
#   REVIEW_SECTION_BREAK=lists-path-never  没有一份判决点名任何路径（review_lists_path 一律说没点名）
#       crates-adversarial-review-green、crates-adversarial-review-pending-green、crates-adversarial-review-section-green、crates-adversarial-review-snapshot-green、
#       crates-adversarial-review-exempt-green、crates-adversarial-review-snapshot-stale-exempt-green、agent-def-adversarial-review-green、agent-def-adversarial-review-pending-green、
#       agent-def-adversarial-review-section-green、agent-def-adversarial-review-snapshot-green、crates-adversarial-review-test-code-green、
#       agent-def-adversarial-review-rules-green、agent-def-adversarial-review-skills-green、agent-def-adversarial-review-hooks-green、agent-def-adversarial-review-gate-green、
#       agent-def-adversarial-review-scripts-green、agent-def-adversarial-review-task-server-green、agent-def-adversarial-review-prune-server-green（期望退出 0，实测 1）；
#       crates-adversarial-review-red（输出里找不到「crates 改动里 2 个文件没有三方判决点名」）；agent-def-adversarial-review-red（输出里找不到「改过的治理文件里 3 份没有三方判决点名」）；
#       crates-adversarial-review-snapshot-stale-red（输出里找不到「crates 改动里 1 个在判决之后又改过」）；agent-def-adversarial-review-snapshot-stale-red（输出里找不到「改过的治理文件里 1 个在判决之后又改过」）；
#       crates-adversarial-review-section-edges-red、crates-adversarial-review-section-moved-red、crates-adversarial-review-section-negated-red、
#       agent-def-adversarial-review-section-moved-red、agent-def-adversarial-review-section-negated-red（照旧判红，红句换成一律没点名的那一句：样本要的那一句找不到）
#
# ── 豁免表与过渡挂账表（crates-adversarial-review 与 agent-def-adversarial-review 两格共用） ──
# 规则：.claude/rules/implementation-workflow.md「三步，缺一步就不算做完」「改 agent 定义与共用约束，走同一条三步」：三方不再豁免。两格各在开头判这几张表，没有对象可判的那一趟也判。
# ① 豁免表 .claude/crates-review-exempt（crates 那一格）与 .claude/agent-def-review-exempt（治理文件那一格）只缩不涨：
#    基准（gate 取法的基准）里有这张表时，表里不许出现基准里没有的行（按「路径、sha256」两列认，只改理由的字不算新行）；基准里还没有这张表时不比；表不存在不算红。
#    判法与点名一节的挂账表同一种（check_review_exempt_frozen，经 lib-lag-baseline.py 的 baseline_rows 取基准那一版）；红句的出路是删掉新加的行、改走一轮三方。
#    表里已有的行照旧按内容哈希生效（user_exemption_reason），补完一轮三方就删一行。
# ② 过渡挂账表：存量没补完三方的文件登记在这里，登记的路径这一格不要求点名（点没点名、快照对不对得上都不看）。
#    crates 那一格读 .claude/gate.d/review-test-code-pending.tsv，路径是角色不是「产品」的包的 src 里的 .rs（「非产品代码」一段），各包 tests 里的 .rs，或 crates/mutations.tsv；
#    治理文件那一格读 .claude/gate.d/governance-review-pending.tsv，路径是项目规则、技能、钩子、门禁、判定库服务、统一任务服务、脚本这几类（定义与共用约束已在管，不进这张表）。
#    形态同点名一节的挂账表：一行「路径<制表符>理由」，理由 8 个字以上，同一路径不登两次，路径要对得上这张表认的那几类（文件在不在不管：删掉的也是对象）；
#    只缩不涨：基准里有这张表时，表里不许出现基准里没有的路径；基准里还没有这张表时不比；表不存在不算红；不合规的逐行列出，判红（check_review_transition_table）。
#    登记的路径在成功句里报「另有 N … 按 <表> 挂账、不要求点名」并逐个列出理由；没用上的行不判红。补完一轮三方（判决点名了它）或那一份不再有改动，把那一行删掉。
#    表里先登什么由主 agent 定；表头只写注释、一行都没有时，两格照旧要求每一份都点名。
# 弄坏开关 REVIEW_SECTION_BREAK=<项>（认法与「点名一节」那一组相同）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本：
#   REVIEW_SECTION_BREAK=exempt-growth-ignored  两张豁免表不和基准比，新加的行放过
#       crates-adversarial-review-exempt-grown-red、agent-def-adversarial-review-exempt-grown-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=transition-growth-ignored  两张过渡挂账表不和基准比，新加的行放过
#       crates-adversarial-review-test-code-transition-grown-red、agent-def-adversarial-review-governance-transition-grown-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=transition-rows-unchecked  两张过渡挂账表的行不核（格式、理由、路径形态、重复）
#       crates-adversarial-review-test-code-transition-table-red、agent-def-adversarial-review-governance-transition-table-red（期望退出 1，实测 0）
#   REVIEW_SECTION_BREAK=transition-ignored  过渡挂账表不起作用：登记的路径照旧要点名
#       crates-adversarial-review-test-code-transition-green、crates-adversarial-review-transition-experiment-apparatus-green、agent-def-adversarial-review-governance-transition-green（期望退出 0，实测 1）
#
# ── 非产品代码（code-round-snapshot 与 crates-adversarial-review 两格共用） ──
# 规则：.claude/rules/implementation-workflow.md「三步，缺一步就不算做完」第 2 步：哪些包不是产品代码，照包的角色认，不按包名写死。
# TEST_CODE_SOURCES 在 cd 进被判的根之后由 load_non_product_crate_sources 现算：读被判的根里的 CRATE_ROLE_TABLE（.claude/gate.d/crate-roles.tsv，四列制表符分隔：
#   包名、角色、目录、理由；--staged 时是暂存进来的那一份），角色不是「产品」的每一行取第三列的目录，拼成 ^(<目录>|…)/src/（grep -E 与 Python re 都认的写法）；
#   表不在、或只有产品包时取 NO_PATH_MATCHES（什么路径都对不上）：没登记角色的包一律照产品算。列数不对的行不认（写得对不对归门禁 code-tooling 的 crate-roles 格判）。
#   code-round-snapshot 格拿它判判决点名的是不是产品代码；crates-adversarial-review 格拿它拼 CRATES_TRANSITION_PATHS（过渡挂账表认的路径、弄坏开关 test-code-skipped 去掉的那一类）。
#   表读不出（python 退非 0）时一格都不跑、退 1。
# 管不到的：角色登得对不对（归 crate-roles 格与人）；包目录不在 crates/<名> 这一层的，code-round-snapshot 格认点名的正则只认 crates/<包>/src/，照不到。
# 弄坏开关 NON_PRODUCT_CRATES_BREAK=<项>（只在样本根上认，同共用库 stage-cells.sh 的那一判）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本
# （括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   NON_PRODUCT_CRATES_BREAK=role-table-ignored  不读角色表，每个包都照产品算（TEST_CODE_SOURCES 取 NO_PATH_MATCHES）
#       code-round-snapshot-green（期望退出 0，实测 1）；code-round-snapshot-no-object、code-round-snapshot-experiment-apparatus-no-object（期望退出 77，实测 1）；
#       crates-adversarial-review-transition-experiment-apparatus-green（期望退出 0，实测 1）
#
# ── code-round-snapshot ──
# 规则：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」。快照形态：research/prompts/<轮>-snapshot/ 目录，
# 里面 crates-sha256.txt（sha256sum 格式：64 位十六进制哈希、两个空格、仓根相对路径）与 kb-sha256.txt；判决里写一行「开工快照」指到那个目录。
# 对象：这次改动新写的 research/prompts/*-main-verification.md（取法与 crates-adversarial-review 同一个函数 new_verdict_paths）里，
#   按路径点名了至少一份产品代码 crates/*/src/*.rs 的判决。点名的认法：正文里匹配 crates/<包>/src/<…>.rs 的字面路径，TEST_CODE_SOURCES（角色不是「产品」的包的 src，
#   「非产品代码」一段）不算；只点 kb 的推论轮、只点非产品代码的、通配写法（crates/*/src/*.rs）的判决不判，成功行与无对象那一行逐份报出。早先的判决（不在新增里的）不判。
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
#      每一行都要出处，不按日期、不按轮名免；这一格与派发闸 ⑥ 用同一份判法。
#   发过腿、没有判决、也没登记 ⇒ 红。登记表写坏（不是四列、有空列、日期不是 YYYY-MM-DD、同一轮登记两行）、出处收不了（没写、指不到、同一行有别的轮名……）、
#   登记了认不出的轮、登记了已经有判决的轮 ⇒ 也红。没有 research/prompts/、或认不出一个发过腿的轮：退 77。
# 文件名不是 ① 那五种形态之一的，不算发过腿：不判，也不报。
# 归档进版本库的腿文件由 git log 取：不在 git 仓里只看树；git log 失败判红，不退回只看树（那样归档的轮次整批消失）。
#   git log 列出的被删文件同样只认 research/prompts/ 顶层：子目录里被删的（腿在 <轮>-<模型>-model/ 里造的门禁样本、<轮>-snapshot/ 里抄的文件）
#   名字再像轮次文件也不算发过腿（弄坏开关 ABANDONED_ROUNDS_BREAK=archived-subdirectories-read 退回改前的读法：取 basename、不限顶层）。
# 管不到的：判决写得对不对、登记的理由真不真、出处那句话是不是用户为这一轮说的（靠主 agent 核）；② 的第二种只认同一小节里既点名又有 **判决，否定词只认紧跟在「**判决」后面的那三个，「**判决**：这一轮先不判」这类别的说法照算判过，
#   「**判决**：未判的那一格补上之后过」这种以否定词起头而其实判过的照算没判（改写那一句）；
#   新轮既不写正文、腿提示又用了五种之外的名字，不算发过腿而不红。
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
# 规则：.claude/rules/implementation-workflow.md「改 agent 定义与共用约束，走同一条三步」「治理文件同规矩」。改动范围里每个被管文件（删掉的也算）
# 都要在这一次改动新写的某份判决里按路径点名，形态照 crates-adversarial-review 格；「点名」同样只认判决里「## 这一轮被判过的文件」一节底下的路径（判据、挂账表与弄坏开关见上面「点名一节」）。
# 被管集合分八类（GOVERNANCE_CLASS_PATTERNS；一个路径算头一个对上的类，成功句逐类报几份）：
#   定义与共用约束  .claude/agents/ 下的 .md、.claude/agent-common.md、.claude/main-agent.md
#   项目规则        .claude/rules/ 顶层的 .md
#   技能            .claude/skills/ 下的 .md
#   钩子            .claude/hooks/ 顶层的 .sh 与 .py（钩子与库；样本目录 fixtures、登记表 .tsv 不算）
#   门禁            .claude/gate.d/ 顶层的 .sh 与 .py、.claude/gate.d/lib/ 下的 .sh 与 .py（样本目录 fixtures、登记表 .tsv 不算）
#   判定库服务      prune-server/ 与 scripts/prune-server/ 下的文件（.md、.lock、target/、__pycache__/ 不算）
#   统一任务服务    task-server/ 下的文件（.md、__pycache__/ 不算）
#   脚本            scripts/ 下的 .sh、.py、.rs（scripts/prune-server/ 先归判定库服务）
# 存量没补完三方的新扩进来的那七类登记在过渡挂账表 GOVERNANCE_TRANSITION_TABLE，登记的路径这一格不要求点名；两张豁免表只缩不涨（判据见「豁免表与过渡挂账表」），
#   豁免表 .claude/agent-def-review-exempt 的格式在 user_exemption_reason 里。
# 没有被管文件的改动：退 77。管不到的：点名了判得对不对；节里不带否定词的转述（「这一轮放过了它」）照算点名（判据在「点名一节」⑤）；
#   上面没列的文件（各目录里的 .tsv 登记表、说明、样本目录、.json）不归这一格管。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   AGENT_DEF_ADVERSARIAL_REVIEW_BREAK=definitions-only  被管集合只剩头一类（定义与共用约束），别的类的改动不算对象
#       agent-def-adversarial-review-rules-red、agent-def-adversarial-review-skills-red、agent-def-adversarial-review-hooks-red、agent-def-adversarial-review-gate-red、
#       agent-def-adversarial-review-scripts-red、agent-def-adversarial-review-task-server-red、agent-def-adversarial-review-prune-server-red（期望退出 1，实测 77）；
#       agent-def-adversarial-review-rules-green、agent-def-adversarial-review-skills-green、agent-def-adversarial-review-hooks-green、agent-def-adversarial-review-gate-green、
#       agent-def-adversarial-review-scripts-green、agent-def-adversarial-review-task-server-green、agent-def-adversarial-review-prune-server-green、
#       agent-def-adversarial-review-governance-transition-green（期望退出 0，实测 77）
#
# ── milestone-close-owed ──
# 为什么：用户 2026-10-22 指出里程碑三带着出口里还开着的欠账收了口；收口那一刻没有东西核欠账表里还有没有指着它的账。
# 判法只在 scripts/kb/milestone-close-owed-check.py（取这一道脚本所在仓里那一份，不取项目根：样本根里放不了坏的），这一格调它的 --closed：
#   ① 里程碑文件 .claude/kb/milestone/<两位号>-<名>-txn.md（译文不算）的段表（表头第一格「段」、有「状态」列）里有一行状态以「结束」或「已完结」起头，算收口；
#      没有段表的不算，照文件现状读、不看历史（细则在那份脚本文件头「判法」）。
#   ② 收口的每个里程碑：.claude/kb/checks-owed.md 开着的那张表（「已还清」整行标题之前，切法同 lib-owed.py）里，前置一格里第一个「去向：」之后那一段（去向段）
#      开头的落点点名它的行数为 0。落点认三种：里程碑文件 `.claude/kb/milestone/<两位号>-<名>-txn.md`（目录与反引号可省）、「里程碑<号>」（阿拉伯或中文数字，
#      「里程碑三二·5」拆出项号）、「未排」（不点名）；①② 后面可接作业点，几个落点用「、」连写在开头算几个，第一个不是落点的字之后都是说明，说明里的号与文件名不算
#      （用户 2026-10-28 定法，细则在那份脚本文件头「判法」的「落点」）；「去向：」之前的说明与出处、整格没写「去向：」的都不算。
#      不为 0 逐行列出编号、简称与点名处的原文，判红。
#   没有一个里程碑的段表写了收口：退 77。收口之前先手跑 python3 scripts/kb/milestone-close-owed-check.py <号>（不看收没收口，照判）。
# 管不到的：去向段开头不是落点的行这一格不点名任何里程碑，归 owed-destination-form 格判红；段表之外别处写的收口（文件头「状态：判定结束」）不认。
# 弄坏开关在那份脚本里（MILESTONE_CLOSE_OWED_BREAK，一次设一个，它的自证逐项钉住）。判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首；
# 去向写法那一格的样本也列在这里，开关是同一份脚本的）：
#   MILESTONE_CLOSE_OWED_BREAK=file-form-ignored  不认里程碑文件名那种落点
#       milestone-close-owed-red（输出里找不到「…有 3 行的前置」）、owed-destination-form-green（期望退出 0，实测 1）
#   MILESTONE_CLOSE_OWED_BREAK=item-suffix-kept  「里程碑三二·5」不拆项号，号认不出就不是落点
#       milestone-close-owed-red（输出里找不到「…有 3 行的前置」）、owed-destination-form-green（期望退出 0，实测 1）、
#       owed-destination-form-red（输出里找不到「欠账表开着的 4 行里写了「去向：」的 3 行：2 行去向段开头不是三种落点之一…」）
#   MILESTONE_CLOSE_OWED_BREAK=status-ignored  不看段表，每个里程碑都判
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-no-object（期望退出 77，实测 1）
#     这一格的红样本在这个开关下照红（多判出里程碑四、十三也满足它的 want），不算它的证据
#   MILESTONE_CLOSE_OWED_BREAK=whole-row-scanned  整行都看、任何一处都算点名（出处一格里的「里程碑三二·4」也算）
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-red（输出里找不到「…有 3 行的前置」）
#   MILESTONE_CLOSE_OWED_BREAK=paid-rows-counted  「已还清」之后的行也算欠着的
#       milestone-close-owed-green（期望退出 0，实测 1）、milestone-close-owed-red（输出里找不到「…有 3 行的前置」）、
#       owed-destination-form-green（期望退出 0，实测 1）、owed-destination-form-red（输出里找不到「欠账表开着的 4 行里写了「去向：」的 3 行…」）
#   MILESTONE_CLOSE_OWED_BREAK=prerequisite-whole-cell  前置那一格整格都看、任何一处都算点名（收窄到「去向：」之后之前的读法）
#       milestone-close-owed-green（期望退出 0，实测 1）
#   MILESTONE_CLOSE_OWED_BREAK=form-unchecked  --destination-form 不判去向段开头是不是落点
#       owed-destination-form-red（期望退出 1，实测 0）
#   paid-heading-optional、whole-segment-scanned、landing-file-unchecked 只在那份脚本的自证里有格（样本的欠账表都有「已还清」标题；
#     样本里去向段开头之后的说明都没写里程碑三；样本的落点文件都在）
#
# ── owed-destination-form ──
# 为什么：用户 2026-10-28：「其中 155 行已挪去别的里程碑，只是写法检查认不出； 这个问题要修复。写法要可以检查的出。」
#   milestone-close-owed 格只认去向段开头的落点，开头写成「挪出里程碑三，归 …」「（从里程碑三挪来）…」的行它不点名任何里程碑，得有一格逼着把开头写成落点。
# 判法与 milestone-close-owed 同一份脚本（scripts/kb/milestone-close-owed-check.py，取法同上），这一格调它的 --destination-form：
#   对象：.claude/kb/checks-owed.md 开着的那张表里写了「去向：」的行（切法同 lib-owed.py；切不成六格的取整行里第一个「去向：」之后那一段）。
#   ① 去向段开头（跳过空白）是三种落点之一：里程碑文件 `.claude/kb/milestone/<两位号>-<名>-txn.md`（目录与反引号可省，可带 .en / .ja）、
#      「里程碑<号>」（「里程碑三二·5」号是三、项二·5）、「未排」起头；不是的逐行列编号、简称与去向段开头 30 字，判红。
#   ② ①② 两种落点点名的里程碑在 .claude/kb/milestone/ 下找得到那份文件（「里程碑<号>」按号找 <两位号>-*-txn.md，文件落点按写的文件名找）；找不到的逐处列出，判红。
#   成功行报判了几行、三种形态各几行（按开头第一个落点算）、没写「去向：」的几行（只报数，不判红）。
#   树里没有欠账表、或开着的行一行写了「去向：」的都没有：退 77。认不出「已还清」标题：红。
# 没并进已有的格：doc-registries 的 table-shape 格也判欠账表的行，但只判格数与「已还」字样，切法是它自己的一份；closeout-collects-open 判里程碑收口表收没收全欠账号，
#   deferred-items-registered 判 records/ 下延后行的登记，对象都不是去向段；落点的认法要与 milestone-close-owed 格点名的认法是同一份，判法放在同一份脚本里。
# 管不到的：落点是不是这件事真该去的地方、作业点（「四·1」「「节名」八·4」）在那份里程碑文件里有没有；没写「去向：」的行该不该写。
# 弄坏开关在那份脚本里（MILESTONE_CLOSE_OWED_BREAK，判错的样本列在上面 milestone-close-owed 那一段；form-unchecked 是这一格自己的）。
# 退役判据：欠账表不再用「去向：」写落点（改成单独一列或别的登记处）时删掉这一格。
#
# ── deferred-items-registered ──
# 为什么：.claude/main-agent.md「一轮怎么开、怎么收」第 2 条要延后的当时就登记、调度记录里写「延后」的那一行同时写出登的那一行的编号或 文件:行；
#   落地审计抽样 8/54 的延后项只写进调度记录、没有去向（records/2026-09-16-subagent拆分提案.md 第九十节 ③），没有检查核它。
# 对象：这一轮 records/ 下新增的 .md 行（gate_added_lines，基准是 gate 取法的基准，未跟踪的整份算新增；只差登记搬迁的行照共用取法剔掉），
#   去掉「」（配平）与反引号里的字、再去掉只在点话题的「延后」之后还带 DEFERRAL_WORDS 里任一个词的。只在点话题的「延后」两种：
#   名词「延后项」（「延后项无去向」「### 延后项的登记」；前面紧挨「进」的「进延后项」「记进延后项」是把这一件延后，照旧算），
#   与带「N/M」统计数的同一个分句里的「延后」（「延后当时就登进欠账表的 6/54」；分句按逗号、句号、分号、冒号与表格竖线切，只管「延后」这一个词）。
#   不判的：词只在「」或反引号里（引规则、引原话）、只在点话题、在围栏代码块里、在标题行上（行首 0–3 个空格之后 1–6 个井号再接空白或行尾；
#   延后什么写在节里的行上）、在拆分提案账本登记表 LEDGER_REGISTRY（被判的根里的 .claude/gate.d/split-proposal-ledgers.tsv）登记的几份里
#   （current 那一份是旧账本、sealed 那几份是封存的旧账本，都不再往里登；它们的编号小节里已有的行照旧是登记）。
#   行号按正文逐字找，同样的行取下一处没报过的。
# 判据：同一行写着下面四种之一、且至少一处指得到，算登记了：
#   ① C 号（C<数字>）：.claude/kb/checks-owed.md 开着的那张表里有一行以「| C<数字> |」起头（开着与已还清的切法经 lib-owed.py，OWED_LIBRARY；
#      已还清表里的 C 号不算，红行写明它已还清；认不出「已还清」标题时照那份库的读法全文都当开着的）；
#   ② 拆分提案的节号与行首序号：行里写了「拆分提案」时，紧跟在「拆分提案」之后的那一个「第<节号>节」（中间只许隔路径尾巴 .md 与 :行号、收尾的反引号、
#      「（2026-10-23 份）」这类括注与空白）认它后面 16 个字以内的行首序号（①…㊿，可以几个连写）；同一行别处的「第<节号>节」与没写「拆分提案」的行一样，只认节号后面紧跟序号的写法
#      （「九十节 ㉟」「第九十节（行首序号 ②、③）」），说别的文件的「登记的第五节」不当拆分提案的节号；那一节（「## <节号>、」标题）在、那一节里有行首是那个序号的一行（「| ③ |」）。
#      先在现行账本里找，没有再照登记表 sealed 行的次序到封存的几份里找：同号的节任一份有那个序号就算指得到，几份都没有那一节（或都没有那个序号）才算指不到，原因里点名登记的每一份；
#      写了「拆分提案」而只写节号的，那一节一行带行首序号的行都没有（只有正文）才算指得到，看照这个次序头一份有这一节的；节号认汉字与阿拉伯数字；「九十节各行」这种不点序号的不算；
#   ③ 文件:行（<路径>:<行号>，可带 -<行号>）：路径从仓根起、文件在树里、行号不超过它的行数。只写文件名的找不到，不算；
#      指到这一行自己（同一份文件、行号或区间罩住这一行）的不算；records/ 下只认登记表里的几份拆分提案，指到别的记录（调度记录、审计记录）的不算——那里是写延后的地方，不是登记处；
#   ④ 登记处：表格行里，带延后类词的那一格之后的某一格（去向格）点名了登记处表 DEFERRAL_REGISTRY_TABLE（.claude/gate.d/deferral-registries.tsv）
#      登记的一份文件（从仓根起的全路径，可在反引号里，前后不接路径里的字）。登记处是「照它做就一定会做到」的提交时那一套流程，写哪几份在表里，不在判法里写死。
#      不是表格行、或只在延后类词那一格与它之前（被延后的那件事本身）点名的，不算。
#      表一份一行「<从仓根起的路径><TAB><为什么它本身就是登记处，8 个字以上>」，井号起头的是注释；不是两段、文件不在树里、理由不到 8 个字、同一份登记两次的，
#      逐行点名、那一行不当登记处认，这一格红。表不在时一份登记处都没有（点名流程文件的行照「没写」报）。
# 写了而一处都指不到的，逐处点名为什么指不到；一处都没写的，点名「没写」。都红。
# 这一轮 records/ 下新增的 .md 行里一行带延后类词的都没有（或都不判）而登记处表没写坏：退 77。欠账表或登记表里的任一份拆分提案读不到：红，点名读不到的那几份（C 号与节号判不了；现行账本读不到不退回只认封存的）。
# 登记表不在或写坏了（判据在「sealed-ledgers-unchanged」那一段的登记表一节）：红，逐行报，这一格不判。
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
#   DEFERRED_ITEMS_BREAK=registry-rows-judged  封存的拆分提案里新增的行也照判
#       deferred-items-registered-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=ledger-rows-judged  现行账本不当登记处，它里面新增的行也照判
#       deferred-items-registered-ledger-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=ledger-ignored  只认封存的：节号只在封存的几份里找，现行账本不读、读不到也不报
#       deferred-items-registered-ledger-green（期望退出 0，实测 1）、deferred-items-registered-ledger-missing-red（期望退出 1，实测 0）、deferred-items-registered-section-scope-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=ledger-missing-tolerated  现行账本读不到时当空的、退回只认封存的，不报读不到
#       deferred-items-registered-ledger-missing-red（期望退出 1，实测 0）
#   DEFERRED_ITEMS_BREAK=history-fallback-dropped  现行账本里找不到的节与序号不再到封存的几份里找
#       deferred-items-registered-ledger-green（期望退出 0，实测 1）
#   DEFERRED_ITEMS_BREAK=sealed-chain-cut  只到登记表头一份封存的里找，后面几份封存的不读
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
#   DEFERRED_ITEMS_BREAK=split-proposal-section-anywhere  行里写了「拆分提案」时整行每个「第<节号>节」都照拆分提案的节号认（判据 ② 收窄之前的认法）
#       deferred-items-registered-section-scope-green（期望退出 0，实测 1）
# 退役判据：.claude/main-agent.md 第 2 条不再要求调度记录里的延后行写登记编号时删掉这一格。
#
# ── ledger-section-numbers ──
# 为什么：几个会话同时往账本末尾追加，节号撞过（2026-10-23 那份撞了五十六、一百四十、一百四十二、一百四十三、一百四十四、一百四十八，索引还缺行）；
#   deferred-items-registered 按「第<节号>节」找节，撞号时找到哪一节由次序定。
# 对象只有现行账本一份（登记表 LEDGER_REGISTRY 里 current 那一行），只看树、不用改动范围，不在 git 仓里也照判；树里没有这份：退 77。封存的几份不判（一字不改，里面的重号照原样留着）。
# 判据（围栏代码块里的行不算）：
#   ① 有「## 索引」一节（标题一行逐字，行尾空白不碍）；
#   ② 每个「## <节号>、」标题（节号是汉字数字，后面接括注「## 一百六十七、（2026-10-23 那份里的第二个五十六）…」不碍）只出现一次；
#   ③ 每个节号在「## 索引」一节的表里恰好一行：行首「| <节号>」起头的行，第一格节号后面接的括注（「一百六十七（2026-10-23 那份里的第二个五十六）」）不碍；
#   ④ 索引表里每一行的节号都有对应的「## <节号>、」一节。
#   没有例外：撞了的号改成下一个空号，索引同步改。
# 登记表不在或写坏了：红，逐行报，这一格不判（判据在「sealed-ledgers-unchanged」那一段的登记表一节）。
# 管不到的：节号是不是按次序接着编（跳号不判）、索引表其余几列写得对不对、标题写成「## 一百七十三 、」「## 173、」这类认不出节号的（不算一节，也就不核索引）。
# 弄坏开关 LEDGER_SECTION_NUMBERS_BREAK=<项>，一次设一个（只在样本根上认，经共用库那一判）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   LEDGER_SECTION_NUMBERS_BREAK=duplicate-headings-tolerated  不判标题重号（判据 ②）
#       ledger-section-numbers-duplicate-red（期望退出 1，实测 0）
#   LEDGER_SECTION_NUMBERS_BREAK=index-unchecked  不核索引表（判据 ③ ④）
#       ledger-section-numbers-index-missing-red（期望退出 1，实测 0）
# 退役判据：这一格守账本的节号与索引（账本已不再往里登新节，现状是只防已有的节被改乱）；账本并回 kb、或登记表里不再有 current 那一行时删掉这一格。
#
# ── sealed-ledgers-unchanged ──
# 为什么：拆分提案账本换代之后，旧的几份封存当历史、不再改（登记表里 sealed 的行，现有三份：records/2026-10-29-subagent拆分提案.md、records/2026-10-23-subagent拆分提案.md、
#   records/2026-09-16-subagent拆分提案.md），登记表里 current 那一行点名的那份旧账本同样不再往里登新节；subagent 与协作工具的问题当场修、不记账（.claude/main-agent.md「一轮怎么开、怎么收」第 2 条）。没有检查拦，往封存那几份追加的照样会写进去。
# 登记表 LEDGER_REGISTRY（被判的根里的 .claude/gate.d/split-proposal-ledgers.tsv；deferred-items-registered、ledger-section-numbers 与这一格三格读同一份，
#   父进程里读一次）：一行一份，四段用制表符分隔「<角色><TAB><从仓根起的路径><TAB><封存时内容的 sha256><TAB><为什么>」，井号起头的与空行不算。
#   角色只认 current 与 sealed；路径在 records/ 下、以 .md 结尾；current 那一行 sha256 写「-」，sealed 行写 64 位小写十六进制；为什么 8 个字以上；
#   同一份不登两次；current 恰好一行。表不在、或有一行不合，三格都红，逐行报（那一行不认）。
#   sealed 行的次序就是 deferred-items-registered 在现行账本之后找节号的次序（新封存的写在前面）。
# 对象：登记表里每一行 sealed。判据：那份文件在树里，内容的 sha256 与登记的相同。不看 git、不看改动范围：封存之后无论哪一次改动碰了它，此刻的内容都对不上。
#   用 sha256 不用「与 gate 基准比」：账本换代的那一次提交本来就要动旧那份（抄节、写去向），与基准比会把这一次判红；登记的 sha256 是封存那一刻的内容，之后改一个字都对不上。
# 一行 sealed 都没有：退 77。
# 管不到的：登记表里的 sha256 本身被改成新内容的（重新封存要用户定，靠 review 看登记表的 diff）；现行账本写了什么（那是别几格的事）。
# 弄坏开关 DOC_PROCESS_RECORDS_BREAK=<项>，一次设一个（只在样本根上认，经共用库那一判）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   DOC_PROCESS_RECORDS_BREAK=sealed-hash-unchecked  封存的那几份只看在不在树里，不比 sha256
#       sealed-ledgers-unchanged-red（期望退出 1，实测 0）
#   DOC_PROCESS_RECORDS_BREAK=ledger-registry-unchecked  登记表的行不核（段数、角色、路径、sha256 写法、理由、重复、current 恰好一行），sealed 行照收、current 取头一行
#       sealed-ledgers-unchanged-registry-red（期望退出 1，实测 0）
#   DOC_PROCESS_RECORDS_BREAK=change-range-error-dropped  load_change_range 取不到时不把预取记下的 git 报错打进格的输出（改动范围那几格共用，这里钉在 batch-scope 的样本上）
#       batch-scope-git-error-red（输出里找不到「git 的报错原文：」）
#   DOC_PROCESS_RECORDS_BREAK=change-range-stderr-swallowed  load_change_range 取到时把命令的标准错误吞掉（改动范围那几格共用，这里钉在 batch-scope 的样本上）
#       batch-scope-git-warning-red（输出里找不到「diff.renameLimit」）
# 退役判据：封存的几份整份删出仓（只留在 git 历史里）时删掉这一格。
#
# ── code-round-checklist ──
# 规则：.claude/rules/implementation-workflow.md「代码轮正文的检查单」。对象：research/prompts/_<题>-code-r<数字>-body.md（轮名以 -code-r<数字> 收尾的三方论证正文，代码轮的约定写法），
#   标题（首行）里第一个 YYYY-MM-DD 在 CODE_ROUND_CHECKLIST_CUTOFF 及以后的；标题没有日期的照要查的算；更早的不查（规范落地之前写的，证据不回改）。
#   要查的正文判八格 CODE_ROUND_CHECKLIST_CELLS；只判八格的名单 CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE（登的是只判八格的正文，标题有没有日期都登）里的只判八格，
#   不在名单里的，标题日期在 CODE_ROUND_CHECKLIST_PLACEMENT_CUTOFF 及以后的与标题没有日期的另判第九格 CODE_ROUND_CHECKLIST_PLACEMENT_CELL；标题日期早于第九格截止日的只判八格，不回改。
#   名单从被判的根下现读：一行「<相对根的正文路径><TAB># 理由>」，空行与 # 起头的行不算；表不在当空名单。
# 判据，每份正文每一格两条，缺一样红：
#   ① 有一行逐字是「### 检查单：<格名>」的小节标题（行尾空白不算），格名是 CODE_ROUND_CHECKLIST_CELLS 的八个、要判第九格的另加 CODE_ROUND_CHECKLIST_PLACEMENT_CELL；
#   ② 标题底下、下一个标题之前至少有一行非空的字（答案与证据写在这里）。
# 另判名单只缩不长：名单里每一行指的正文都要在（正文归档、改名之后那一行判红，同一次改动删掉），写不成「<路径><TAB># 理由」的行同样判红；
#   名单与 gate 基准（scripts/gate/changed-paths.sh 的 gate_diff_base gate）那一版比，照 crate-roles 滞后表那一套经 lib-lag-baseline.py 的 baseline_rows 取：
#   基准里没有的路径逐行判红；基准里还没有这张表、被判的根不在 git 仓里不比，成功句下一行写明；git 取不到基准那一版判红。
# 红句逐份逐格点名，出路指到规则那一节；名单的红另起一句，点名第几行。没有任何一份代码轮正文，或都早于截止日：退 77。
# 管不到的：答案写得对不对、腿有没有对每一格回「成立 / 不成立 / 推不出」（靠主 agent 核腿的报告）；不叫 -code-r<数字> 的正文这一格不认；
#   标题里先写一个更早的日期（它攻的那份判决的日期这类），判几格照那个日期算（.claude/rules/directory-layout.md「门禁管哪一半」靠 review 那一栏）。
# 弄坏开关 DOC_PROCESS_RECORDS_BREAK=<项>，一次设一个（只在样本根上认，经共用库那一判）：
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-unjudged  这一格不判，有代码轮正文也直接报通过（名单也不核）
#       code-round-checklist-red、code-round-checklist-empty-red、code-round-checklist-placement-red、code-round-checklist-placement-undated-red、
#       code-round-checklist-placement-cutoff-day-red、code-round-checklist-eight-cell-list-stale-red、code-round-checklist-eight-cell-list-grown-red（期望退出 1，实测 0）
#       code-round-checklist-green（照样退 0，九格一份都没判，成功句里「判九格的 1 份、判八格的 3 份」出不来，是「判九格的 0 份、判八格的 4 份」）
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-answers-unchecked  只看格的标题在不在，不看标题底下有没有写字
#       code-round-checklist-empty-red（期望退出 1，实测 0）
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-placement-unchecked  第九格「放的位置」一份都不判
#       code-round-checklist-placement-red、code-round-checklist-placement-undated-red、code-round-checklist-placement-cutoff-day-red（期望退出 1，实测 0）
#       code-round-checklist-green（照样退 0，成功句里「判九格的 1 份、判八格的 3 份」出不来，是「判九格的 0 份、判八格的 4 份」）
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-placement-everywhere  标题日期早于第九格截止日的正文也判第九格
#       code-round-checklist-green（期望退出 0，实测 1）
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-eight-cell-list-ignored  不看只判八格的名单：名单里的正文照标题日期判（标题没有日期、日期在截止日及以后的判九格）
#       code-round-checklist-green（期望退出 0，实测 1）
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-eight-cell-list-stale-accepted  名单里的行指的正文不在了、写不成「<路径><TAB># 理由」也不报
#       code-round-checklist-eight-cell-list-stale-red（期望退出 1，实测 0）
#   DOC_PROCESS_RECORDS_BREAK=code-round-checklist-eight-cell-list-growth-accepted  名单不与基准那一版比，多出的行、基准取不到都不报
#       code-round-checklist-eight-cell-list-grown-red（期望退出 1，实测 0）
# 退役判据：代码轮正文不再按 -code-r<数字> 起名，或检查单挪进别的门禁时删掉这一格。
#
# ── 样本 ──
# fixtures/doc-process-records.sh/ 下每格一对，各放 .gate-cells 只点名它那一格；setup.sh 里给别的格补的东西不碍事。
#   batch-scope-red                 没登记的触发文件、不在这一批的登记、没写理由的登记、不从仓库根起的路径，四种同时判红
#   batch-scope-git-error-red       setup.sh 建 git 仓并把 git 配置里的 diff.renames 写成 git 认不得的值，取改动路径的 git diff 一律失败：
#                                   红行之前有 gate_changed_paths 写的哪一条 git、退几与 git 的报错原文（预取在父进程里，原文经 load_change_range 的 <前缀>_error 带进格的输出）
#   batch-scope-git-warning-red     setup.sh 在暂存区删两个文件、加两个内容不同的文件，git 配置里的 diff.renameLimit 写成 1，取改动路径的 git diff 成功、却往标准错误写重命名探测超上限的警告：
#                                   输出里要有那条警告（取到时的标准错误原样打出来），触发文件没登记照样判红
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
#   crates-adversarial-review-test-code-only  只改了验证包与池级 checker 两个包的 src 四个、验证包 tests/ 下一个测试与 crates/mutations.tsv，没有判决，判红逐个列出这六份（测试代码同样是对象）
#   crates-adversarial-review-test-code-green  同一份改动，判决在「## 这一轮被判过的文件」一节里点名了这六份，判绿报 6 个
#   crates-adversarial-review-test-code-comment-only  验证包的 src 一份与 tests/ 下一份都只改了注释，没有判决，退 77 逐个报出只改注释的
#   crates-adversarial-review-exempt-grown-red  基准里的豁免表有一行，工作区又加了一行（哈希对得上，改了的 lib.rs 靠它能豁免），豁免表只缩不涨，判红点名新加的第 2 行
#   crates-adversarial-review-exempt-shrunk-green  基准里的豁免表有两行，工作区删了一行，留着的一行按改后的内容哈希豁免 comment.rs，判绿报 1 个豁免
#   crates-adversarial-review-test-code-transition-green  过渡挂账表在基准里三行、工作区删了变异表那一行，改了的两份测试登记着、没有判决，判绿报 2 个挂账与登记的理由
#   crates-adversarial-review-test-code-transition-grown-red  基准里的挂账表一行、工作区又加了一行，两份测试都改了，挂账表只缩不涨，判红点名新加的第 2 行
#   crates-adversarial-review-test-code-transition-table-red  挂账表写坏五行（理由太短、路径是产品代码、变异表登记两次、只有一段），判红列四处
#   crates-adversarial-review-comment-only  两份产品代码只改了注释（行注释、行尾注释、文档注释的文字），没有判决，退 77 逐个报出只改注释的
#   crates-adversarial-review-comment-and-code-red  一份只改注释、一份改了字符串里的 //、一份改了文档注释里的 doctest，都没有判决，判红只列后两份
#   code-round-snapshot-red         新判决点名两份产品代码文件、快照清单只列了一份，判红只列缺的那一份
#   code-round-snapshot-no-snapshot-red  四份新判决各缺一样（没提快照目录、目录不在树里、目录里没有 crates-sha256.txt、清单里点名文件的哈希不是 64 位十六进制），判红逐份点名
#   code-round-snapshot-green       两份判决的快照罩住点名的文件（一份快照目录这次新加、一份已跟踪；一份点名的文件已删、不要求有行）；只点 kb、只点测试代码、通配写法与基准里就有的旧判决不判，判绿报对数
#   code-round-snapshot-no-object   新判决只点 kb 与测试代码，退 77 逐份报出不判的
#   code-round-snapshot-experiment-apparatus-no-object  新判决只点名角色表里登成实验装置的包的 src/bin 下一份，退 77 报只点名了角色不是产品的包的代码
#   crates-adversarial-review-transition-experiment-apparatus-green  过渡挂账表在基准里登了实验装置包 src/bin 下的一份，它改了、没有判决，判绿报 1 个挂账
#   implementation-premise-red      没有 crates/ 的正文、标题没日期的正文、缺开工快照，逐个点名判红
#   implementation-premise-green    判绿，报查了几份、没查哪一份
#   code-round-checklist-red        一份代码轮正文只写了六格（缺「不新建判定库」「所有任务多机可用」），判红点名缺的两格
#   code-round-checklist-empty-red  一份代码轮正文八格的标题都在，其中「测试优先多线程」底下紧接下一个标题、一个字没写，判红点名这一格
#   code-round-checklist-green      标题日期在第九格截止日之后的一份九格都点名并写了答案，标题日期在两个截止日之间的一份八格齐、没有第九格，
#                                   标题没有日期而在只判八格名单里的一份、标题日期就是第九格截止日而在名单里的一份，都八格齐；
#                                   另一份标题日期早于截止日的旧代码轮正文没有检查单，不判、成功行报没查哪一份；名单两行指的正文都在
#   code-round-checklist-placement-red          标题日期在第九格截止日之后的一份八格齐、缺「放的位置」，判红点名这一格
#   code-round-checklist-placement-undated-red  标题没有日期、不在只判八格名单里的一份八格齐、缺「放的位置」，判红点名这一格
#   code-round-checklist-placement-cutoff-day-red  标题日期就是第九格截止日、不在名单里的一份八格齐、缺「放的位置」，判红点名这一格
#   code-round-checklist-eight-cell-list-stale-red  名单第一行指的正文不在（另有一份九格齐的正文），判红点名名单那一行
#   code-round-checklist-eight-cell-list-grown-red  setup.sh 在样本根 git init、提交只有一行的名单，再加上第二行（两行指的八格正文都在，另有一份九格齐的正文）：
#                                   名单比基准那一版多出一行，判红点名第 2 行
#   abandoned-rounds-red            发了腿没判决没登记的轮、只在 kb 里被提到的轮、写坏与说反话的登记，另有 setup.sh 建的缺旧树对象的 git 仓，判红
#   abandoned-rounds-green          判决文件、kb 小节判决、登记撂下各一轮，判绿报对数（登记撂下那一行 c364-r3 的出处是 records/halt.md 第 1 行）
#   abandoned-rounds-source-red     没写出处的行（2026-10-13）、出处那一行同时写着别的轮名、出处指到 research/ 下这一轮自己的材料、「」里的原话不到 6 个字、没写出处而日期是 2026-10-11 的行（不按日期免）、
#                                   出处那一行另写着仓里认得而不以 -r<数字> 结尾的别的轮名 close-b1（research/prompts/_close-b1-body.md）、出处那一行另写着形如 <名>-r<数字> 而仓里没有任何它的材料的别的轮名 ghost-r2，
#                                   〔〕里的原话改了一个字、在出处文件里找不到而行号提示那一行（第 3 行）没有这一轮名字的 gone-r1，
#                                   〔〕里的原话在出处文件里命中两次而行号提示（第 9 行）不在其中的 twice-r1，只写行号没带〔〕原话的 plain-r1，
#                                   〔〕写的是转述、行号提示的第 10 行有这一轮的名字与「」原话而去掉改名与日期之后对不上的 misread-r1，
#                                   另有没写出处的 d16-item1-r2，十二行逐行点名判红
#   abandoned-rounds-source-green   四行带合格出处（「第 N 行」与「:N」两种写法，另一行的出处那一行写着 m3-close-b1：仓里没有它的任何材料、也不是 -r<数字> 形，不当别的轮名；
#                                   drift-r1 写的是第 2 行，〔〕原话在第 5 行：出处那份记录前面插过行，原话恰好找到一次照收，成功行后面报挪到第几行；
#                                   pinned-r1 的〔〕原话在第 6、7 行各一次、行号提示写第 7 行；reworded-r1 的〔〕是改名前的原话（样本根的 .claude/kb/term-renames.md 登记了这次改名），
#                                   dated-r1 的〔〕带着第 9 行后来删掉的日期，两行原话都找不到而去掉新旧名与日期之后落在行号提示那一行里；
#                                   三行照提示取那一行收，成功行后面报照提示取的三行）、
#                                   c196-r1 的出处是 records/halt.md 第 10 行，判绿报对数与出处查了几行
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
#   agent-def-adversarial-review-rules-red  改了一份项目规则与同目录的一份 .txt，没有判决，判红只列规则那一份
#   agent-def-adversarial-review-rules-green  同一份改动，判决在那一节里点名了规则，判绿报项目规则 1 份
#   agent-def-adversarial-review-skills-red  改了一份技能的 SKILL.md 与同目录的一份 .txt，没有判决，判红只列 SKILL.md
#   agent-def-adversarial-review-skills-green  同一份改动，判决点名了 SKILL.md，判绿报技能 1 份
#   agent-def-adversarial-review-hooks-red  改了钩子与钩子库各一份、同目录的登记表与 fixtures 里的一份脚本，没有判决，判红只列钩子与库两份
#   agent-def-adversarial-review-hooks-green  同一份改动，判决点名了钩子与库两份，判绿报钩子 2 份
#   agent-def-adversarial-review-gate-red  改了门禁与 lib/ 下的库各一份、同目录的登记表与 fixtures 里的一份脚本，没有判决，判红只列门禁与库两份
#   agent-def-adversarial-review-gate-green  同一份改动，判决点名了门禁与库两份，判绿报门禁 2 份
#   agent-def-adversarial-review-scripts-red  改了 scripts/ 下的 .sh、.py、.rs 各一份、同目录的登记表与说明，没有判决，判红只列三份脚本
#   agent-def-adversarial-review-scripts-green  同一份改动，判决点名了三份脚本，判绿报脚本 3 份
#   agent-def-adversarial-review-task-server-red  改了 task-server/ 下的入口、一份 .py、一份测试、同目录的说明与 __pycache__ 里的字节码，没有判决，判红只列前三份
#   agent-def-adversarial-review-task-server-green  同一份改动，判决点名了前三份，判绿报统一任务服务 3 份
#   agent-def-adversarial-review-prune-server-red  改了 scripts/prune-server/ 的客户端、prune-server/ 的 src 与 control，删了一份 src，另改说明与 Cargo.lock，没有判决，判红列四份（含删掉的）
#   agent-def-adversarial-review-prune-server-green  同一份改动，判决点名了四份，判绿报判定库服务 4 份
#   agent-def-adversarial-review-exempt-grown-red  基准里的豁免表有一行，工作区又加了一行（哈希对得上，改了的 demo.md 靠它能豁免），豁免表只缩不涨，判红点名新加的第 2 行
#   agent-def-adversarial-review-exempt-shrunk-green  基准里的豁免表有两行，工作区删了一行，留着的一行按改后的内容哈希豁免 demo.md，判绿报 1 份豁免
#   agent-def-adversarial-review-governance-transition-green  过渡挂账表在基准里三行（钩子、脚本、规则）、工作区删了规则那一行，改了的钩子与脚本登记着、没有判决，判绿报 2 份挂账与各类各几份
#   agent-def-adversarial-review-governance-transition-grown-red  基准里的挂账表一行、工作区又加了一行，钩子与脚本都改了，挂账表只缩不涨，判红点名新加的第 2 行
#   agent-def-adversarial-review-governance-transition-table-red  挂账表写坏五行（理由太短、路径是定义与共用约束、脚本登记两次、只有一段），判红列四处
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
#   owed-destination-form-red       开着的四行：去向段开头写「挪出里程碑三，归 …」与「（从里程碑三挪来）…」的两行逐行报，开头是「里程碑八一·1」后接括注的、没写「去向：」的不报，
#                                   已还清那张表里的一行不判，判红报「写了「去向：」的 3 行：2 行去向段开头不是三种落点之一」
#   owed-destination-form-green     开着的六行：文件落点后接作业点与「从里程碑三挪来」的括注、号带项号后接括注、「、」连写的两个落点（作业点里的节名带里程碑四）、
#                                   「未排」起头、「去向：」之前写说明的各一行，没写「去向：」的一行只报数；已还清那张表里写坏的一行不判，判绿报三种形态各几行
#   deferred-items-registered-red   往调度记录追加七行带延后类词的：一行写开着的 C10 不报，六行逐个报（没写登记、C99 欠账表里没有、「九十节 ⑤」那一节没有这一行、
#                                   写了「拆分提案」的「第九十节」有行首序号却没写哪一行、research/notes.md:99 超过行数、gone.md:3 不在树里）
#   deferred-items-registered-green 往调度记录追加五行，C10、样本里登记成 current 的那一份 records/2026-10-29-subagent拆分提案.md:8、「九十节 ③」、写了「拆分提案」的只有正文的「第九十四节」、research/notes.md:12 各指一行，判绿报「判了 5 行」；
#                                   词只在「」里的一行、围栏里的一行、往封存那份的九十节加的一行（登记处本身）与基准里的旧行不判
#   deferred-items-registered-topic-green  追加七行：两行标题、两行只把「延后项」当话题名词、一行只在报统计数（「延后项无去向 8/54」「延后当时就登进欠账表的 6/54」）不判，
#                                   一行写 C10 照判，判绿报「判了 1 行」，不判的里只在点话题的 4 行、标题 1 行
#   deferred-items-registered-topic-red    追加六行，一行写 C10 不报，五行逐个报：「进延后项」、「延后项」旁边另有「下一批」、「延后到」、井号后面没空格的「#延后」、统计数旁边的「下一批」
#   deferred-items-registered-registry-green  登记处表两行（staged-gate.md、commit-stages.md）写对，两行表格的去向格各点名一份（一份在反引号里），一行写 C10，判绿报「判了 3 行」
#   deferred-items-registered-registry-red    登记处表四行写坏逐行报（文件不在树里、理由太短、只有一段、登记两次）；追加八行，去向格点名 staged-gate.md 的一行不报，
#                                   七行逐个报：「九十节各行」、没登记的流程文件、登记处只在延后类词那一格之前、不是表格行、staged-gate.md.old、表里写坏了那一行的两份
#   deferred-items-registered-ledger-green  账本登记表：样本里登记成 current 的那一份是 2026-10-29，封存的两份是 2026-10-23（在前）与 2026-09-16。追加五行各指一处：只在样本里登记成 current 的那一份的「第一百零二节 ①」、
#                                   只在头一份封存的「第一百一十节 ①」、只在后一份封存的「第三十节 ①」、样本里登记成 current 的那一份与后一份封存都有九十七节而 ④ 只在后一份封存的「第九十七节 ④」、
#                                   写了「拆分提案」的只有正文的样本里登记成 current 的那一份「第一百零三节」，判绿报「判了 5 行」；往样本里登记成 current 的那一份新登一节带「下一批」的一行（登记处本身）不判
#   deferred-items-registered-ledger-red    追加三行，一行写 C10 不报，两行逐个报：「第一百零九节 ①」登记的两份都没有那一节、「九十节 ⑦」两份的九十节都没有这一行，原因里点名登记的两份
#   deferred-items-registered-ledger-missing-red  登记表点名的现行账本不在树里，追加一行「九十节 ②」（封存那份里指得到），照实报读不到现行账本、红
#   deferred-items-registered-pointers-red  追加五行：已还清的 C11、文件:行指到同一份调度记录的别一行（records/ 下不是登记的拆分提案）、文件:行指到这一行自己，三行逐个报；
#                                   指样本里登记成 current 的那一份 records/2026-10-29-subagent拆分提案.md:8 的一行与写开着的 C10 的一行不报
#   deferred-items-registered-section-scope-green  样本里登记成 current 的那一份有一百零六节 ①、没有第五节、第三节；追加三行：「登记的第五节」写在「拆分提案第一百零六节 ①」前面 16 个字以内、
#                                   「拆分提案第一百零六节 ①，登记第五节」、「拆分提案（2026-10-29 份）第一百零六节 ①，判决的第三节」，只核一百零六节，判绿报「判了 3 行」
#   ledger-section-numbers-green    样本里登记成 current 的那一份 records/2026-10-29-subagent拆分提案.md：标题与索引第一格都带括注的「一百六十七（2026-10-23 那份里的第二个五十六）」、
#                                   索引第一格带括注的「一百（…）」、围栏里的标题与表格行不算，判绿报节号、标题与索引行数
#   ledger-section-numbers-duplicate-red  另一个会话同时追加，「## 一百零三、」写了两处（索引一行），判红报重号的两行
#   ledger-section-numbers-index-missing-red  「## 一百零四、」没在索引表占号、索引表多一行没有对应一节的「一百零九」，判红逐处报
#   sealed-ledgers-unchanged-green  登记表一行现行、两行封存，两份封存的内容与登记的 sha256 相同，判绿报「封存的 2 份」
#   sealed-ledgers-unchanged-red    同一张登记表，2026-10-23 那份封存之后末尾又追加了一节，判红点名它、报两个 sha256，出路写「封存账本不改，新内容不往账本里登」；2026-09-16 那份对得上、不报
#   sealed-ledgers-unchanged-registry-red  登记表写坏五行逐行报（两行 current、认不得的角色、只有三段、理由不到 8 个字、同一份登了两次），封存那份的 sha256 本身对得上
# 登记行判法的弄坏开关在 .claude/hooks/lib_abandoned_rounds.py（LIB_ABANDONED_ROUNDS_BREAK，一次设一个；派发闸的自检也带着）。红在这一格的样本（括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   columns-unchecked        不核四列不空                     abandoned-rounds-red（输出里找不到「不是四列、或有空列」）
#   date-unchecked           不核日期是 YYYY-MM-DD            abandoned-rounds-red（输出里找不到「日期不是 YYYY-MM-DD」）
#   duplicate-unchecked      同一轮两行也不拒                 abandoned-rounds-red（输出里找不到「dup-r1（第 」）
#   source-optional          出处一概不要求                   abandoned-rounds-source-red（输出里找不到「这些行的出处收不了」）、abandoned-rounds-source-green（输出里找不到「登记表 8 行：…出处按派发闸 ⑥ 同一份判法查了 8 行」）、
#                                                             abandoned-rounds-source-date-parentheses-green（输出里找不到「登记表 2 行：…出处按派发闸 ⑥ 同一份判法查了 2 行」）
#   other-round-name-ok      出处那一行有别的轮名也收         abandoned-rounds-source-red（输出里找不到「除这一轮外还有别的轮名 other-r1」）
#   other-name-by-form       别的轮名的全集退回只认登记表里的名字（仓里的 _<轮>-body.md 等材料不算；-r<数字> 形照认）  abandoned-rounds-source-red（输出里找不到「named-r1：第 7 行的出处…除这一轮外还有别的轮名 close-b1」）
#   other-form-ignored       别的轮名不认 -r<数字> 形，只认仓里认得的全集（改前的判法）  abandoned-rounds-source-red（输出里找不到「formonly-r1：第 8 行的出处…除这一轮外还有别的轮名 ghost-r2」）
#   source-any-path          出处不限 records/                abandoned-rounds-source-red（输出里找不到「ownbody-r1：第 4 行没有出处」）
#   quote-short-ok           「」里一个字也算原话             abandoned-rounds-source-red（输出里找不到「至少 6 个字」）
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
#       abandoned-rounds-red、abandoned-rounds-source-red、agent-def-adversarial-review-red、agent-def-adversarial-review-section-moved-red、batch-scope-git-error-red、batch-scope-git-warning-red、batch-scope-red、code-round-snapshot-no-snapshot-red、code-round-snapshot-red、crates-adversarial-review-comment-and-code-red、crates-adversarial-review-exempt-stale-red、
#       crates-adversarial-review-pending-grown-red、crates-adversarial-review-pending-table-red、crates-adversarial-review-red、crates-adversarial-review-section-edges-red、crates-adversarial-review-section-moved-red、implementation-premise-red、
#       knowledge-sync-red、verdict-names-local-samples-missing-clean-sample、verdict-names-local-samples-red、abandoned-rounds-archived-top-level-red、milestone-close-owed-red、owed-destination-form-red、deferred-items-registered-red、deferred-items-registered-topic-red、deferred-items-registered-registry-red、
#       deferred-items-registered-ledger-red、deferred-items-registered-ledger-missing-red、batch-scope-stale-red、
#       crates-adversarial-review-section-negated-red、agent-def-adversarial-review-section-negated-red、abandoned-rounds-kb-negated-red、
#       ledger-section-numbers-duplicate-red、ledger-section-numbers-index-missing-red、deferred-items-registered-pointers-red、
#       crates-adversarial-review-snapshot-stale-red、agent-def-adversarial-review-snapshot-stale-red、sealed-ledgers-unchanged-red、sealed-ledgers-unchanged-registry-red、code-round-checklist-red、code-round-checklist-empty-red、
#       code-round-checklist-placement-red、code-round-checklist-placement-undated-red、code-round-checklist-placement-cutoff-day-red、code-round-checklist-eight-cell-list-stale-red、
#       code-round-checklist-eight-cell-list-grown-red、crates-adversarial-review-test-code-only、crates-adversarial-review-exempt-grown-red、crates-adversarial-review-test-code-transition-grown-red、crates-adversarial-review-test-code-transition-table-red、
#       agent-def-adversarial-review-exempt-grown-red、agent-def-adversarial-review-governance-transition-grown-red、agent-def-adversarial-review-governance-transition-table-red、
#       agent-def-adversarial-review-rules-red、agent-def-adversarial-review-skills-red、agent-def-adversarial-review-hooks-red、agent-def-adversarial-review-gate-red、
#       agent-def-adversarial-review-scripts-red、agent-def-adversarial-review-task-server-red、agent-def-adversarial-review-prune-server-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  不认样本根的 .gate-cells，每份样本十四格全跑
#       abandoned-rounds-green、abandoned-rounds-red、abandoned-rounds-source-date-parentheses-green、abandoned-rounds-source-green、abandoned-rounds-source-red、agent-def-adversarial-review-green、agent-def-adversarial-review-pending-green、agent-def-adversarial-review-red、agent-def-adversarial-review-section-green、agent-def-adversarial-review-section-moved-red、batch-scope-green、
#       batch-scope-git-error-red、batch-scope-git-warning-red、batch-scope-red、code-round-snapshot-green、code-round-snapshot-no-object、code-round-snapshot-experiment-apparatus-no-object、code-round-snapshot-no-snapshot-red、code-round-snapshot-red、crates-adversarial-review-comment-and-code-red、crates-adversarial-review-comment-only、crates-adversarial-review-exempt-green、crates-adversarial-review-exempt-stale-red、crates-adversarial-review-green、
#       crates-adversarial-review-pending-green、crates-adversarial-review-pending-grown-red、crates-adversarial-review-pending-table-red、crates-adversarial-review-red、crates-adversarial-review-section-edges-red、crates-adversarial-review-section-green、crates-adversarial-review-section-moved-red、crates-adversarial-review-test-code-only、implementation-premise-green、implementation-premise-red、
#       knowledge-sync-green、knowledge-sync-old-record-green、knowledge-sync-red、verdict-names-local-samples-green、verdict-names-local-samples-missing-clean-sample、
#       verdict-names-local-samples-red、abandoned-rounds-archived-subdirectory-green、abandoned-rounds-archived-top-level-red、
#       milestone-close-owed-red、milestone-close-owed-green、milestone-close-owed-no-object、owed-destination-form-red、owed-destination-form-green、
#       deferred-items-registered-red、deferred-items-registered-green、
#       deferred-items-registered-topic-green、deferred-items-registered-topic-red、deferred-items-registered-registry-green、deferred-items-registered-registry-red、
#       deferred-items-registered-ledger-green、deferred-items-registered-ledger-red、deferred-items-registered-ledger-missing-red、
#       crates-adversarial-review-section-negated-red、agent-def-adversarial-review-section-negated-red、abandoned-rounds-kb-negated-red、ledger-section-numbers-green、
#       ledger-section-numbers-duplicate-red、ledger-section-numbers-index-missing-red、deferred-items-registered-pointers-red、
#       crates-adversarial-review-snapshot-green、crates-adversarial-review-snapshot-stale-red、crates-adversarial-review-snapshot-stale-exempt-green、
#       agent-def-adversarial-review-snapshot-stale-red、agent-def-adversarial-review-snapshot-green、
#       crates-adversarial-review-test-code-green、crates-adversarial-review-test-code-comment-only、crates-adversarial-review-exempt-grown-red、crates-adversarial-review-exempt-shrunk-green、
#       crates-adversarial-review-test-code-transition-green、crates-adversarial-review-test-code-transition-grown-red、crates-adversarial-review-test-code-transition-table-red、
#       crates-adversarial-review-transition-experiment-apparatus-green、
#       agent-def-adversarial-review-exempt-grown-red、agent-def-adversarial-review-exempt-shrunk-green、agent-def-adversarial-review-governance-transition-green、
#       agent-def-adversarial-review-governance-transition-grown-red、agent-def-adversarial-review-governance-transition-table-red、
#       agent-def-adversarial-review-rules-red、agent-def-adversarial-review-rules-green、agent-def-adversarial-review-skills-red、agent-def-adversarial-review-skills-green、
#       agent-def-adversarial-review-hooks-red、agent-def-adversarial-review-hooks-green、agent-def-adversarial-review-gate-red、agent-def-adversarial-review-gate-green、
#       agent-def-adversarial-review-scripts-red、agent-def-adversarial-review-scripts-green、agent-def-adversarial-review-task-server-red、agent-def-adversarial-review-task-server-green、
#       agent-def-adversarial-review-prune-server-red、agent-def-adversarial-review-prune-server-green、
#       sealed-ledgers-unchanged-green、sealed-ledgers-unchanged-red、sealed-ledgers-unchanged-registry-red、deferred-items-registered-section-scope-green（输出里找不到「样本只跑 .gate-cells 点名的格：…」）；
#       batch-scope-stale-pruned-green、code-round-checklist-green（期望退出 0，实测 1）；batch-scope-stale-red（输出里找不到「样本只跑 .gate-cells 点名的格：batch-scope」）
#
# 格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的几份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析与汇总。格的出路里写的「doc-process-records.sh 里这一格函数上方的注释」，就在下面对应的那份库里（格在哪份库按格名列在下面）：
# 库 lib/doc-process-records-batch-and-review-cells.sh：batch-scope、verdict-names-local-samples、crates-adversarial-review、code-round-snapshot
# 库 lib/doc-process-records-batch-and-review-cells.sh：implementation-premise
# 库 lib/doc-process-records-rounds-and-ledger-cells.sh：abandoned-rounds、knowledge-sync、agent-def-adversarial-review、milestone-close-owed
# 库 lib/doc-process-records-rounds-and-ledger-cells.sh：owed-destination-form、deferred-items-registered、ledger-section-numbers
# 库 lib/doc-process-records-rounds-and-ledger-cells.sh：sealed-ledgers-unchanged、code-round-checklist
# 弄坏开关的读法在入口的赋值与格函数里，格函数在 lib/ 下的库里；开关表仍在上面文件头。下面每个开关变量在哪份库的代码里被读，按库列在这里：
# 库 lib/doc-process-records-batch-and-review-cells.sh：AGENT_DEF_ADVERSARIAL_REVIEW_BREAK、BATCH_SCOPE_BREAK、CODE_ROUND_SNAPSHOT_BREAK
# 库 lib/doc-process-records-batch-and-review-cells.sh：CRATES_ADVERSARIAL_REVIEW_BREAK、VERDICT_LOCAL_SAMPLES_BREAK
# 库 lib/doc-process-records-rounds-and-ledger-cells.sh：ABANDONED_ROUNDS_BREAK、CLAUSE_BREAK、DEFERRED_ITEMS_BREAK、DOC_PROCESS_RECORDS_BREAK
# 库 lib/doc-process-records-rounds-and-ledger-cells.sh：KNOWLEDGE_SYNC_BREAK、LEDGER_SECTION_NUMBERS_BREAK、LIB_ABANDONED_ROUNDS_BREAK
# 入口的代码里也读的：DOC_PROCESS_RECORDS_BREAK、NON_PRODUCT_CRATES_BREAK、REVIEW_SECTION_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：MILESTONE_CLOSE_OWED_BREAK
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ doc-process-records.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
# implementation-premise 格按 glob 取正文：没有匹配时展开成空，不留字面的通配串
shopt -s nullglob

STAGE_NAME="$(basename "${BASH_SOURCE[0]}")"
STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$STAGE_DIR/lib/stage-cells.sh"
# 触发文件清单只在这里写一处，batch-scope 与 knowledge-sync 两格都从这里取
TRIGGER_TABLE=".claude/gate.d/knowledge-sync-triggers.tsv"
PREMISE_CUTOFF="2026-09-17"
SIDE_AND_SNAPSHOT_CUTOFF="2026-09-28"
# code-round-checklist 格：标题日期在它及以后的代码轮正文要有检查单；八个格名与 .claude/rules/implementation-workflow.md「代码轮正文的检查单」那张表逐字相同
CODE_ROUND_CHECKLIST_CUTOFF="2026-11-07"
CODE_ROUND_CHECKLIST_HEADING_PREFIX="### 检查单："
CODE_ROUND_CHECKLIST_CELLS=("走不到的理由二次查实" "类型与名字" "不兼容旧代码与旧格式" "不新建判定库" "所有任务多机可用" "测试优先多线程" "带 --item 的命令与项" "新崩溃流多罩了什么")
# 第九格：标题日期在它及以后的正文另判这一格（格名同那张表的第九行）；标题日期更早的只判上面八格
CODE_ROUND_CHECKLIST_PLACEMENT_CUTOFF="2026-11-19"
CODE_ROUND_CHECKLIST_PLACEMENT_CELL="放的位置"
# 只判八格的名单（相对被判的根）：登的是只判八格的代码轮正文，标题有没有日期都登、不看 CODE_ROUND_CHECKLIST_PLACEMENT_CUTOFF；名单里的正文不在了判红
CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE=".claude/gate.d/code-round-checklist-eight-cells.tsv"
# 包的角色登记表（四列制表符分隔：包名、角色、目录、理由；读被判的根里那一份）与产品角色：角色不是它的包的 src 是非产品代码（文件头「非产品代码」）
CRATE_ROLE_TABLE=".claude/gate.d/crate-roles.tsv"
CRATE_PRODUCT_ROLE="产品"
# 什么路径都对不上的正则：GNU grep -E 与 Python re 里 ^ 都只在开头对得上，接在 crates/ 后面永远落空（ugrep 这类把中间的 ^ 另当一回事的不认这一条）；没有非产品角色的包（表不在、只有产品包）时 TEST_CODE_SOURCES 取它
NO_PATH_MATCHES='^crates/^'
# code-round-snapshot 格点名里不算、crates-adversarial-review 格过渡挂账表认的非产品代码：角色不是产品的包的 src，形如 ^(<目录>|…)/src/；
# cd 进被判的根之后由 load_non_product_crate_sources 现算（文件头「非产品代码」），在那之前是空的
TEST_CODE_SOURCES=""
# crates-adversarial-review 格的对象：crates/ 下的全部代码，各包 src 里的 .rs、各包 tests 里的 .rs 与 crates/mutations.tsv（同一份规则第 2 步的射程）
CRATES_REVIEW_OBJECTS='^crates/([^/]+/(src|tests)/.+\.rs|mutations\.tsv)$'
# 对象里过渡挂账表 CRATES_TRANSITION_TABLE 认的那一类：非产品代码（TEST_CODE_SOURCES）里的 .rs，各包 tests 里的 .rs，crates/mutations.tsv；弄坏开关 test-code-skipped 把这一类从对象里去掉；
# 与 TEST_CODE_SOURCES 一起由 load_non_product_crate_sources 现算
CRATES_TRANSITION_PATHS=""

# load_non_product_crate_sources：照当前目录（被判的根）里的 CRATE_ROLE_TABLE 现算 TEST_CODE_SOURCES 与 CRATES_TRANSITION_PATHS（文件头「非产品代码」）。
# 在当前 shell 里调（不经 $( ) 子 shell），cd 进被判的根之后、跑格之前调一次。弄坏开关 NON_PRODUCT_CRATES_BREAK=role-table-ignored：不读表，每个包都照产品算。
load_non_product_crate_sources() {
  local pattern
  if [[ "${NON_PRODUCT_CRATES_BREAK:-}" == role-table-ignored ]]; then
    pattern="$NO_PATH_MATCHES"
  elif ! pattern="$(python3 - "$CRATE_ROLE_TABLE" "$CRATE_PRODUCT_ROLE" "$NO_PATH_MATCHES" <<'PY'
import os
import re
import sys

table, product_role, no_path_matches = sys.argv[1:4]
directories = set()
if os.path.isfile(table):
    with open(table, encoding="utf-8", errors="replace") as handle:
        for line in handle.read().split("\n"):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            fields = [field.strip() for field in line.split("\t")]
            # 四列（包名、角色、目录、理由）才认；列数不对的行照没登记算，写得对不对归门禁 code-tooling 的 crate-roles 格判
            if len(fields) == 4 and fields[1] and fields[2] and fields[1] != product_role:
                # 只转义 grep -E 与 Python re 两边都当特殊字符的那几个：re.escape 会把 - 也转义，grep -E 对 \- 报「stray \」
                directories.add(re.sub(r"([.^$*+?()\[\]{}|\\])", r"\\\1", fields[2].rstrip("/")))
print(f"^({'|'.join(sorted(directories))})/src/" if directories else no_path_matches)
PY
)"; then
    echo "  ✗ 读不出 $CRATE_ROLE_TABLE 里哪些包的角色不是产品（上面是 python 的报错），code-round-snapshot 与 crates-adversarial-review 两格认不出非产品代码，这一道一格都没跑"
    echo "     → 怎么办：照上面的报错修好那张表（四列制表符分隔：包名、角色、目录、理由；写法在 scripts/gate/crate-roles-check.py 文件头）或修好 python3，再跑这一道。"
    exit 1
  fi
  TEST_CODE_SOURCES="$pattern"
  CRATES_TRANSITION_PATHS="${TEST_CODE_SOURCES}"'.+\.rs$|^crates/[^/]+/tests/.+\.rs$|^crates/mutations\.tsv$'
}
# 判决里认点名的那一节（crates-adversarial-review 与 agent-def-adversarial-review 两格共用）：标题一行逐字是它，判决里别处提到路径不算点名
REVIEWED_SECTION_HEADING="## 这一轮被判过的文件"
# 那一节里写了这些否定词之一的行不算点名（文件头「点名一节」判据 ⑤），空白分隔；加词之前先量真仓判决的那一节会多红几行
REVIEWED_SECTION_NEGATIONS="没攻 没判 未攻 未判 没审 未审 不攻 不判"
# 这一节上线之前写的新判决没有那一节：挂账表一行一份判决，认整份正文里的提到；只缩不涨，格式见文件头「点名一节」
REVIEW_PENDING_TABLE=".claude/gate.d/review-section-pending.tsv"
# 两张豁免表只缩不涨、不再加行（表里已有的行照旧生效）；两张过渡挂账表登记存量没补完三方的文件，只缩不涨（文件头「豁免表与过渡挂账表」）
CRATES_EXEMPT_TABLE=".claude/crates-review-exempt"
AGENT_DEF_EXEMPT_TABLE=".claude/agent-def-review-exempt"
CRATES_TRANSITION_TABLE=".claude/gate.d/review-test-code-pending.tsv"
GOVERNANCE_TRANSITION_TABLE=".claude/gate.d/governance-review-pending.tsv"
# agent-def-adversarial-review 格的被管集合：一类一个名字、一条认路径的正则、一条排除的正则（空是不排除）；一个路径算头一个对上的类，次序就是成功句里各类的次序；
# 头一类（定义与共用约束）之外的七类是过渡挂账表认的那几类；判定库服务排在脚本前面，scripts/prune-server/ 归它
GOVERNANCE_CLASS_NAMES=("定义与共用约束" "项目规则" "技能" "钩子" "门禁" "判定库服务" "统一任务服务" "脚本")
GOVERNANCE_CLASS_PATTERNS=(
  '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$'
  '^\.claude/rules/[^/]+\.md$'
  '^\.claude/skills/.+\.md$'
  '^\.claude/hooks/[^/]+\.(sh|py)$'
  '^\.claude/gate\.d/([^/]+|lib/[^/]+)\.(sh|py)$'
  '^(scripts/prune-server|prune-server)/'
  '^task-server/'
  '^scripts/.+\.(sh|py|rs)$'
)
GOVERNANCE_CLASS_EXCLUDES=('' '' '' '' '' '(^|/)(target|__pycache__)/|\.(md|lock|pyc)$' '(^|/)__pycache__/|\.(md|pyc)$' '')

stage_cell batch-scope check_batch_scope "这一批要碰的触发文件有没有登记进范围" \
  "没登记的触发文件先问它挡不挡着本轮课题，挡着的在 .claude/batch-scope 加一行「<从仓库根起的路径>  # 为什么这一批要碰它」，不挡着的挪到下一批；多出来的登记（红行格式「文件:行号：路径」，路径是冒号后面那段）照红行下面那条命令整行删掉"
stage_cell verdict-names-local-samples check_verdict_names_local_samples "新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）" \
  "逐份读那一轮本地腿交回的样本（干净的、通读判带损坏的、-output-void<n>.md），在新写的判决里按路径列出：.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」"
stage_cell crates-adversarial-review check_crates_adversarial_review "crates/ 下全部代码（各包 src 与 tests 里的 .rs、crates/mutations.tsv）的改动有没有走过三方正反对抗推理" \
  "提交之前走一轮三方（.claude/rules/three-way-inference.md），判决写进 research/prompts/<题>-r<n>-main-verification.md，判决里写一节「## 这一轮被判过的文件」（标题逐字、二级标题），节里逐个按路径列出改过的 crates 代码文件（各包 src 与 tests 里的 .rs、crates/mutations.tsv，测试代码同样在内，射程见 .claude/rules/implementation-workflow.md「三步，缺一步就不算做完」）；只认这一节，正文别处提到路径不算；点名写进新的判决，不往旧判决里补；判决之后又改过的（那一轮开工快照的 sha256 与此刻的内容对不上）再开一轮；豁免表 .claude/crates-review-exempt 不再加行；存量测试代码没补完的登记在 .claude/gate.d/review-test-code-pending.tsv（只缩不涨）"
stage_cell code-round-snapshot check_code_round_snapshot "按路径点名了产品代码 crates 文件的新判决，有没有点名开工快照、快照有没有罩住点名的每份文件" \
  "派腿之前 sha256sum <判决要点名的 crates 文件…> > research/prompts/<轮>-snapshot/crates-sha256.txt（kb 文件另记 kb-sha256.txt），判决里写一行开工快照指到那个目录（.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」）；派腿时刻已过的，主 agent 补记快照并在判决里写明是事后补的"
stage_cell implementation-premise check_implementation_premise "三方论证材料有没有「实现今天的样子」" \
  "正文前提表里加一行「实现今天的样子」写读过的 crates/ 路径与事实；补开工快照 research/prompts/<轮>-snapshot/（.claude/agents/three-way-materials.md 第 5b 步）"
stage_cell abandoned-rounds check_abandoned_rounds "发过腿的三方轮次，要么有判决，要么登记撂下" \
  "补判决 research/prompts/<轮>-main-verification.md，或在 research/prompts/abandoned-rounds.tsv 加一行（轮名、日期、为什么撂下、去向，制表符分隔）；登记表写坏的照上面逐行改"
stage_cell knowledge-sync check_knowledge_sync "改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录" \
  "写或补 research/prompts/<阶段>-sync.md（格式见这道门禁文件头 knowledge-sync 那一段）：触发文件逐个按路径点名，「## 搜索」「## 命中处置」「## 原始证据」三节照格式写全"
stage_cell agent-def-adversarial-review check_agent_def_adversarial_review "改过的治理文件（定义与共用约束、规则、技能、钩子、门禁、scripts/ 下的脚本、两个服务）有没有走过三方审核" \
  "走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，判决里写一节「## 这一轮被判过的文件」（标题逐字、二级标题），节里按路径列出改过的每一份治理文件；只认这一节，正文别处提到路径不算；点名写进新的判决，不往旧判决里补；判决之后又改过的（那一轮开工快照的 sha256 与此刻的内容对不上）再开一轮；豁免表 .claude/agent-def-review-exempt 不再加行；存量没补完的登记在 .claude/gate.d/governance-review-pending.tsv（只缩不涨）"
stage_cell milestone-close-owed check_milestone_close_owed "段表写了收口的里程碑，欠账表开着的那张表里没有一行的前置点名它" \
  "收口之前把点名它的那几行逐行了结：做完的挪进 .claude/kb/checks-owed.md 的「已还清」，没做完的把去向改到还开着的里程碑（「去向：」之后开头的落点不再写这个里程碑的号或文件名，挪出来的经过写在落点之后）；改不了的就别收口，把段表那一格的「结束 / 已完结」撤回；改完跑 python3 scripts/kb/milestone-close-owed-check.py <号> 到绿"
stage_cell owed-destination-form check_owed_destination_form "欠账表开着的行，「去向：」之后开头是三种落点之一（里程碑文件、「里程碑<号>」、「未排」），落点点名的里程碑文件都在" \
  "把去向段开头改成三种落点之一，说明挪到落点之后：① .claude/kb/milestone/<两位号>-<名>-txn.md（反引号可有可无）后接作业点（「四·1」「「节名」八·4」）；② 「里程碑<号>」后接作业点（「里程碑三二·5」）；③ 「未排」起头；几个落点用「、」连写在开头；落点的文件找不到的核号与文件名；改完跑 python3 scripts/kb/milestone-close-owed-check.py --destination-form 到绿"
stage_cell deferred-items-registered check_deferred_items_registered "records/ 下这次新增的带延后类词的行，同一行写着指得到的登记（C 号、拆分提案的节号与行首序号、文件:行或登记处表里的流程文件）" \
  "照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行：先在 .claude/kb/checks-owed.md、里程碑收口表或决策正文登一行（拆分提案账本不再往里登，里面已有的节照旧认），再在调度记录那一行写开着的 C<号>、「第<节号>节 <行首序号>」或从仓根起的 <路径>:<行号>（records/ 下只认登记表里的拆分提案，不指这一行自己）；subagent 与协作工具的事不延后、不登账，当场修（第 4 条）；去向是提交时那一套流程的，表格行的去向格点名 .claude/gate.d/deferral-registries.tsv 登记的那份文件；指不到的照红行写的原因改"
stage_cell ledger-section-numbers check_ledger_section_numbers "现行账本（拆分提案账本登记表里 current 那一份）的「## <节号>、」标题不重号，每个节号在「## 索引」表里恰好一行" \
  "新登一节用 python3 scripts/kb/ledger-append-section.py --title '<标题（含日期）>' --body-file <正文文件>：它读 .claude/gate.d/split-proposal-ledgers.tsv 里 current 那一份，现取最大节号加一，「## 索引」表末行与末尾的「## <节号>、」一节一次写入，两个会话同时追加时后到的退 4、重跑取下一个号；不手写新节的标题；撞了号的改成下一个空号、索引表里同步改；索引里多出的、少了的照红行补删"
stage_cell sealed-ledgers-unchanged check_sealed_ledgers_unchanged "拆分提案账本登记表里封存的那几份，内容与登记的 sha256 相同（封存之后没被改）" \
  "封存账本不改，新内容不往账本里登：把红行点名的那份恢复成封存时那一版（git log -- <路径> 找到内容的 sha256 与登记表那一行相同的提交，git show <提交>:<路径> > <路径>）；要记的新内容，subagent 与协作工具的问题当场修（.claude/main-agent.md「一轮怎么开、怎么收」第 4 条），别的延后项登 .claude/kb/checks-owed.md（第 2 条）；登记表写坏的照红行逐行改"
stage_cell code-round-checklist check_code_round_checklist "代码轮正文逐格点名了检查单的八格（第九格截止日之后、不在只判八格名单里的九格），每格底下写了答案；名单指的正文都在、比基准那一版没多出行" \
  "照 .claude/rules/implementation-workflow.md「代码轮正文的检查单」，在红句点名的正文里补上缺的格：每格一个「### 检查单：<格名>」小节，底下写答案与证据（文件:行），再单跑 bash .claude/gate.d/doc-process-records.sh --check code-round-checklist"
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
load_non_product_crate_sources
inside_git=0
git rev-parse --is-inside-work-tree >/dev/null 2>&1 && inside_git=1
# 弄坏开关 REVIEW_SECTION_BREAK 只在样本里认（与样本根的 .gate-cells 同一个口径）：被判的根是这道阶段所在的仓或 --staged 的临时树时不认，平常跑环境里留着它也不起作用
review_break=""
case "${REVIEW_SECTION_BREAK:-}" in
  whole-verdict-substring|pending-rows-unchecked|pending-growth-ignored|negated-lines-counted|snapshot-hash-unchecked|lists-path-never|exempt-growth-ignored|transition-growth-ignored|transition-rows-unchecked|transition-ignored)
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
# 拆分提案账本登记表（现行一份、封存几份；文件头「sealed-ledgers-unchanged」）：deferred-items-registered、ledger-section-numbers、sealed-ledgers-unchanged 三格都读被判的根里的这一份，换账本只改它
LEDGER_REGISTRY=".claude/gate.d/split-proposal-ledgers.tsv"
# deferred-items-registered 格认 C 号只认开着的：欠账表开着与已还清的切法只在这一份（取这一道脚本所在仓里那一份，不取项目根：样本根里放不了坏的）
OWED_LIBRARY="$STAGE_DIR/lib-owed.py"
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

# load_change_range <变量名前缀> <取法命令…>：命令的标准输出存进 <前缀>，状态存进 <前缀>_status，标准错误存进 <前缀>_error；取过的不再取。
# 取法命令是 gate_changed_paths 或 gate_added_lines：它们失败时把哪一条 git、退几、git 的报错原文写到标准错误。下面「预取」一段在父进程里取，
# 各格的子 shell 里没有那段标准错误，所以接进 <前缀>_error；取不到时（预取过的与现取的一样）每次调用都把它原样打进调用者自己的输出，退 1。
# 取到时命令写到标准错误的东西（例：重命名探测超上限的警告）在取的那一次原样打出来，与直接起命令时一样。
# 弄坏开关 DOC_PROCESS_RECORDS_BREAK=change-range-stderr-swallowed：取到时不打命令的标准错误。
load_change_range() {
  local prefix="$1" loaded_name="${1}_loaded" status_name="${1}_status" error_name="${1}_error" error_file="" value status=0
  shift
  if [[ "${!loaded_name}" != 1 ]]; then
    printf -v "$loaded_name" '%s' 1
    error_file="$(mktemp "${TMPDIR:-/tmp}/change-range-error.XXXXXX")" || error_file=""
    if [[ -n "$error_file" ]]; then
      if value="$("$@" 2> "$error_file")"; then status=0; else status=1; fi
      printf -v "$error_name" '%s' "$(cat -- "$error_file")"
      rm -f -- "$error_file"
      if ((status == 0)) && [[ -n "${!error_name}" && "${DOC_PROCESS_RECORDS_BREAK:-}" != change-range-stderr-swallowed ]]; then
        printf '%s\n' "${!error_name}" >&2
      fi
    else
      if value="$("$@")"; then status=0; else status=1; fi
    fi
    printf -v "$prefix" '%s' "$value"
    printf -v "$status_name" '%s' "$status"
  fi
  if ((${!status_name} != 0)); then
    # 弄坏开关 DOC_PROCESS_RECORDS_BREAK=change-range-error-dropped：不打预取时记下的报错
    [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == change-range-error-dropped ]] || printf '%s\n' "${!error_name:-  ! 取改动范围的命令没有留下报错}" >&2
    return 1
  fi
}
load_changed_since_gate() { load_change_range changed_since_gate gate_changed_paths "$gate_base" untracked || return 1; }
load_added_since_gate() { load_change_range added_since_gate gate_changed_paths "$gate_base" untracked A || return 1; }
load_changed_since_head() { load_change_range changed_since_head gate_changed_paths "$head_base" untracked || return 1; }
load_added_record_lines() { load_change_range added_record_lines gate_added_lines "$gate_base" 'research/prompts/*-sync.md' || return 1; }
# records/ 下这一轮新增的 .md 行（deferred-items-registered 用）
load_added_records_directory_lines() { load_change_range added_records_directory_lines gate_added_lines "$gate_base" 'records/*.md' || return 1; }
not_git_repository() {
  echo "  ! $ROOT 不是 git 仓，这一格无对象可判"
  return 77
}

# ── 拆分提案账本登记表：父进程里读一次（各格在自己的子 shell 里跑，子 shell 里读的传不回父进程），用它的三格各自报 ledger_registry_problems ──
# 写法与判据在文件头「sealed-ledgers-unchanged」的登记表一节；弄坏开关 DOC_PROCESS_RECORDS_BREAK=ledger-registry-unchecked 下不核行，sealed 照收、current 取头一行。
# 读出来一行一条：「current<TAB>路径」「sealed<TAB>路径<TAB>sha256」「problem<TAB>哪一行、哪里不合」。
current_ledger=""; sealed_ledgers=(); sealed_ledger_digests=(); ledger_registry_problems=()
if ledger_registry_rows="$(python3 - "$LEDGER_REGISTRY" "${DOC_PROCESS_RECORDS_BREAK:-}" <<'PY'
import re, sys

path, broken = sys.argv[1], sys.argv[2]
unchecked = broken == "ledger-registry-unchecked"
DIGEST = re.compile(r"[0-9a-f]{64}")
MINIMUM_REASON_CHARACTERS = 8
try:
    with open(path, encoding="utf-8") as handle:
        rows = handle.read().split("\n")
except FileNotFoundError:
    print(f"problem\t{path} 不在：现行账本与封存的几份都登记在被判的根里的这一份")
    sys.exit(0)
except (OSError, UnicodeDecodeError) as error:
    print(f"problem\t读不了 {path}（{error}）")
    sys.exit(0)
current, seen = [], set()
for number, row in enumerate(rows, 1):
    if not row.strip() or row.startswith("#"):
        continue
    fields = [field.strip() for field in row.split("\t")]
    where = f"{path}:{number}"
    if unchecked:
        if fields[0] == "current" and len(fields) > 1:
            current.append((where, fields[1]))
        elif fields[0] == "sealed" and len(fields) > 2:
            print(f"sealed\t{fields[1]}\t{fields[2]}")
        continue
    if len(fields) != 4:
        print(f"problem\t{where}：不是「角色<TAB>路径<TAB>sha256<TAB>为什么」四段（读出 {len(fields)} 段）")
        continue
    role, ledger, digest, reason = fields
    if role not in ("current", "sealed"):
        print(f"problem\t{where}：角色「{role}」认不得（只认 current 与 sealed）")
    elif not (ledger.startswith("records/") and ledger.endswith(".md")):
        print(f"problem\t{where}：{ledger} 不是 records/ 下的 .md（写从仓根起的路径）")
    elif role == "current" and digest != "-":
        print(f"problem\t{where}：current 那一行是还没封存的那一行，sha256 写「-」，读出「{digest}」")
    elif role == "sealed" and not DIGEST.fullmatch(digest):
        print(f"problem\t{where}：sealed 行的 sha256 要写 64 位小写十六进制（sha256sum < {ledger} 的头一段），读出「{digest}」")
    elif len(reason) < MINIMUM_REASON_CHARACTERS:
        print(f"problem\t{where}：{ledger} 的为什么不到 {MINIMUM_REASON_CHARACTERS} 个字")
    elif ledger in seen:
        print(f"problem\t{where}：{ledger} 登了两次")
    else:
        seen.add(ledger)
        if role == "current":
            current.append((where, ledger))
        else:
            print(f"sealed\t{ledger}\t{digest}")
if current and (unchecked or len(current) == 1):
    print(f"current\t{current[0][1]}")
elif not current:
    print("problem\t没有 current 那一行：现行账本恰好登一份")
else:
    print(f"problem\tcurrent 登了 {len(current)} 行（{'、'.join(where for where, _ledger in current)}）：现行账本恰好登一份")
PY
)"; then
  while IFS= read -r ledger_registry_row; do
    case "$ledger_registry_row" in
      current$'\t'*) current_ledger="${ledger_registry_row#current$'\t'}" ;;
      sealed$'\t'*)
        ledger_registry_row="${ledger_registry_row#sealed$'\t'}"
        sealed_ledgers+=("${ledger_registry_row%%$'\t'*}")
        sealed_ledger_digests+=("${ledger_registry_row#*$'\t'}") ;;
      problem$'\t'*) ledger_registry_problems+=("${ledger_registry_row#problem$'\t'}") ;;
    esac
  done <<<"$ledger_registry_rows"
else
  ledger_registry_problems+=("读 $LEDGER_REGISTRY 的 python 退了非 0（报错在上面），现行账本与封存的几份一份都没认出来")
fi

# 登记表有不合的行：逐行报、退 1；没有：退 0、什么都不打。用它的三格开头先调。
report_ledger_registry_problems() {
  ((${#ledger_registry_problems[@]})) || return 0
  local problem
  for problem in "${ledger_registry_problems[@]}"; do
    echo "  ✗ $problem"   # gate-lint:detail
  done
  echo "  ✗ 拆分提案账本登记表 $LEDGER_REGISTRY 有 ${#ledger_registry_problems[@]} 处不合，现行账本与封存的几份认不准，这一格没判"
  echo "     → 怎么办：照这道门禁文件头「sealed-ledgers-unchanged」的登记表一节逐行改：「current 或 sealed<TAB>records/<名>.md<TAB>sha256（current 写 -）<TAB>为什么（8 个字以上）」，current 恰好一行、同一份不登两次"
  return 1
}

# 下面两个函数（点名一节的取法与去否定句）留在入口：scripts/agents/verdict-skeleton.py 按名字从这一份文件里抽它们的正文，连同上面 REVIEWED_SECTION_HEADING、REVIEWED_SECTION_NEGATIONS 两行，自证骨架里的点名一节
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

# 按格组 source 下面几份库；它们读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取
source "$STAGE_DIR/lib/doc-process-records-batch-and-review-cells.sh"
source "$STAGE_DIR/lib/doc-process-records-rounds-and-ledger-cells.sh"

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

# 这一道的格互不依赖、都只读被判的树：并行跑（.claude/gate.d/lib/stage-cells.sh 文件头「格并行」）
STAGE_CELLS_JOBS="${STAGE_CELLS_JOBS:-$(stage_cells_default_jobs)}"
stage_cells_run "$ROOT"
