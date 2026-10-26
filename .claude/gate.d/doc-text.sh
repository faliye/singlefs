#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 文本写法：全仓不许出现登记过的禁用写法（撇号类角标、描述里的钟点与时区词、会话号）与登记过改名的术语旧名；改动碰到的记录与 kb 页日期只写在标题上；登记的三语文件（仓根 README 与 briefs/）三份齐全、结构对得上、改了中文就三份同进同出（只改译文的放行）、译文照翻译术语表用词、译文里不残留超量没译的中文、译文与中文逐段比内容量不漏段不压成摘要；这次改动新加到记录、kb 页与三方判决正文的行里不写账号切换与用量限额；这次改动新加的行里不写不归本项目的目录名与根下系统目录的占用；这次改动里的文本文件（工作区与暂存区里的版本）是合法 UTF-8
# gate-fixture: yes
# gate-category: 文档类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 的两项判被测代码的行为，这一道只判文本写法。
# gate-similar: code-tooling.sh 它的 fixture-claims 格也扫全仓文件，但判的是文件名与样本目录里编出来的日期、阶段头部声称的样本在不在，不是文本里的写法
# gate-similar: code-source-discipline.sh 它的 vague-names 与 test-file-names 两格也判写法，但对象是 .rs 里声明的函数名、类型名与测试文件名，读的是空泛词表与按里程碑起名的正则，不扫全部文本
# gate-similar: .claude/singlefs-ai-sop/scripts/i18n-sync.sh 它判上游各语言仓之间的同步（兄弟目录里的仓、清单、溯源哈希）；trilingual-* 六格判本仓里登记的英、中、日三份文件，对象与输入都不同，不并进它
# gate-similar: .claude/singlefs-ai-sop/scripts/link-targets.py 它读到不是 UTF-8 的 markdown 时把那份判红（读不了、里面的链接没查），范围是全仓的 .md、判的是链接指不指得到；utf8-valid 一格判这次改动里每份文本文件（含暂存区里与工作区不同的那一版）的字节本身合不合法，范围与对象都不同，沿用本道已有的改动范围取法与路径打码，不另立门禁
# gate-similar: .claude/hooks/write-guard.sh 它在 Write / Edit 当场判写进的内容（lib-forbidden-notations.py 的禁用写法），不读本机目录与放行表，也不看这一次改动划出的新加行；out-of-project-names 一格也不并进同一道的 private-values 格：词源（家目录与仓的上一级目录现读、第二台私有名单）、只在像目录的上下文里判、放行表连同它要有脚本引到、私有名单不在时照判别的来源而不是整格 77，判法与出路都不同，只共用新加行的取法与路径打码（PRIVATE_VALUES_LIBRARY 的 added_lines、shown_path）
#
# gate-cell: prime-marks 撇号类角标
# gate-cell: clock-times 描述里的钟点、时区词与时间戳
# gate-cell: session-names 会话号（ListAgents 列的会话名）
# gate-cell: term-renames 登记过改名的术语不再出现旧名
# gate-cell: date-once 这次改动碰到的记录与 kb 页里，日期只在标题上：正文行不重复所属标题或文件名里的日期，同一天不开第二个同级小节
# gate-cell: trilingual-files 登记的三语文件三份齐全：中文 <名>.md 的 <名>.en.md、<名>.ja.md 都在，登记目录里单有 .en.md 或 .ja.md 的也要三份齐全；登记表写对了
# gate-cell: trilingual-structure 三语文件每组三份的结构对得上（编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数），抓只改了一份
# gate-cell: trilingual-changed-together 这次改动碰了一组三语文件里的中文那一份，英语、日语两份都要碰；只改译文的组放行
# gate-cell: trilingual-glossary 翻译术语表写对了，三语文件的译文照术语表用词
# gate-cell: trilingual-untranslated 三语文件的英语、日语译文去掉反引号、引文、链接目标与路径之后，一行里不残留超量没译的中文（英语数汉字，日语数日语不用的简体字），与中文那一份某一行逐字相同而去段后还剩中文的行也不许有
# gate-cell: trilingual-omission 三语文件的英语、日语译文与中文逐段配对比内容量（表格行按首列编号配对，其余按节内次序）：过短（漏了一段、压成摘要）、过长（对错了段）与配不上的段（段数不同的节、编号行缺一边）都不许有
# gate-cell: private-values 这次改动新加的行与改动的文件名里不出现本机私有配置 multi-host.env 里的值（8 个字以上的词）与本机主机名
# gate-cell: account-quota 这次改动新加到 records/、.claude/kb/ 与三方判决正文的行里不写账号切换、会话或周限额、额度重置这类用量限额（显卡额度、内存上限、空间配额不在内）
# gate-cell: out-of-project-names 这次改动新加的行里不写不归本项目的目录名（本机家目录、仓的上一级目录与第二台私有名单里的条目名，只在像目录的上下文里判）与根下系统目录的占用；放行表 .claude/out-of-project-paths 每一条要有 git 跟踪的脚本引到
# gate-cell: utf8-valid 这次改动（暂存区或工作区相对基准）里的文本文件是合法 UTF-8：非法的点名文件、工作区还是暂存区、第一处坏字节的偏移
# 格结构经 lib/stage-cells.sh。
#
# ── prime-marks、clock-times、session-names ──
# 格名就是同目录 lib-forbidden-notations.py 里 FORMS 的形态键。git ls-files --cached --others --exclude-standard 列出的每份文本文件
# （不在 git 仓里时走目录树）逐行交那一形态判，违例报「文件:行」。判据行、上下文放行规则、放行标记、排除前缀、成功行与出路都在那份库里；
# 加一种禁用写法就是往 FORMS 加一项、再在格名表里加一格。.claude/hooks/write-guard.sh 写入时读同一份库。
# 射程是全仓，冻结证据（research/prompts/、records/、变更史、research/results/）也在内。
# 没扫的：二进制文件（前 8 KiB 里有 NUL）、列出了却不是普通文件的与读不了的（权限这一类，不让异常回溯打出原路径），逐个列名。一份文本文件都没扫到：退 77。
# 打印的路径（红行、「二进制：」「不是普通文件」「读不了」三个清单）与红行里的行文片段都经 private-values 那一格的取词和打码（下面「private-values」一段的 PRIVATE_VALUES_LIBRARY，同一份共用函数）：
#   路径里含私有值的那一段打成「〔文件名含私有值〕」，片段里的值打成「〔私有值〕」；没有 multi-host.env（别人的克隆、第二台）就没有词可打，原样打。
#   片段取命中位置前 20 个字到后 10 个字，两头落在私有值中间的挪到值的边上（整值进片段再打码，snippet_bounds），不打出半截值。
# 管不到的：ASCII 单引号当角标（与代码里的引号分不开）；改过标题的会话。
#
# ── term-renames ──
# 判法在 scripts/kb/sweep-term.py --check：按 .claude/kb/term-renames.md 查（「匹配」认整串、词边界、正则三种），
# 在 git 工作树里查 git 跟踪的与未跟踪而未忽略的文件、git 忽略的不查，不在 git 工作树里查整棵目录树；
# 排除表 .claude/term-rename-exempt 认整份豁免与按行豁免（<路径>:「<行内锚>」只豁免那一行），写法都在那份脚本文件头。
# 排除表指向不存在的路径、行内锚命中不是恰好一行、正则写不对，也判红。没有登记表：退 77。规则：.claude/rules/path-moves.md「改一个全仓术语：正文之外还有五处会红」。
# sweep-term 的输出（逐份列的路径与处数）经 private-values 那一份打码再打：按空白切开，带 / 的一段当路径（shown_path），别的当行文（shown_text）；shown_lines_main 判。
# 管不到的：登记表里没登记的改名。
#
# ── date-once ──
# 这次改动碰到的（含未跟踪的）records/*.md 与 .claude/kb/ 下的 .md 逐份交
# scripts/kb/doc-consolidate.py migrate 干跑（DOC_CONSOLIDATE_LEDGER=off：干跑不查也不记那份脚本的「迁过的文件」台账，每次都判），干跑改了什么就是违例；哪些日期不算，判法在那份脚本文件头。
# 不判的：两份变更史（decisions-history.md、experiments-history.md）与 INDEX.md，连同它们的英语、日语译文。变更史按 .claude/rules/changelog-format.md 一条决策或实验一节、
#   节内按日期分块，同一天在不同节各有一块是这种结构本来的样子；译文照中文结构译，同样不判。译文路径照同目录 lib-trilingual.py 的 language_paths 取
#   （<名>.en.md、<名>.ja.md；.claude/kb/translation-paths.json 给译文另起了名的照它），三语六格认译文用的是同一份，不按后缀另写一份认法；装不进来判红，不退回只排除中文那三份。
# 干跑退 0 或 3 才读它那一行摘要；退别的（5 是认不出历史节：翻译术语表读不到或没有「历史版本」那一行；1 是读不了那一份这类）那一份记「没判」、点名退出码与报错，整格判红，不当干跑 0 处。
# 基准取 scripts/gate/changed-paths.sh 的 gate 取法；不在 git 仓里整棵树都判；一份都没碰到：退 77。红行里的路径经 private-values 那一份打码（shown_path）。
# 规则：.claude/rules/changelog-format.md、kb-discipline 第 1 条。管不到的：别的日期该不该写、日期挪进标题之后小节切得对不对。
#
# ── trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary、trilingual-untranslated、trilingual-omission ──
# 对象是 .claude/trilingual-documents 登记的中文文件（今天是仓根 README.md、briefs/*.md 与 .claude/kb/**/*.md）：一份中文文件 <名>.md 配 <名>.en.md 与 <名>.ja.md，
# 六格都从这张登记表取对象，判法都在同目录 lib-trilingual.py（六个子命令，文件头有每一条判据与每个弄坏开关），阶段里不抄第二份。
# files：登记表自己写对了（行有理由、模式指到文件、模式不带语言后缀）；每组三份都在；登记目录里单有 .en.md 或 .ja.md 的也要三份齐全，逐份点名缺哪份。
# structure：三份齐全的组逐项比，一项不同就点名组、项与各份的值：编号集合（D、E、C 加数字，I- 加数字与点；不用 \b，日文字符算词字符；
#   不数链接目标 ](…) 里的，链接文字照数：译文链的是译文另起的文件名，中文文件名里的 I-3.1 在日语文件名里写成 I-3-1）、
#   Markdown 链接目标 ](目标) 的集合、围栏外各级标题的个数、围栏外 | 起头的表格行个数、围栏代码块个数。缺份的组不比，归 files。
# changed-together：基准取 scripts/gate/changed-paths.sh 的 gate 取法（gate_diff_base gate 与 gate_changed_paths … untracked，阶段里不再抄一遍）；
#   碰了一组里的中文那一份，英语、日语两份都要碰，删掉的也算碰；中文那一份没碰、只碰了译文的组放行（用户 2026-10-23 定「只在中文变了时要求同改」，
#   译得对不对归 structure、glossary、untranslated 三格），成功那句与红的汇总另报放行几组；一组都没碰到、或不在 git 仓里：退 77。
# glossary：翻译术语表 .claude/kb/translation-glossary.md，表上方单独一行 <!-- trilingual-glossary --> 紧挨着表，表头
#   | 中文 | English | 日本語 | 说明 / Note / 注 |。表本身：每行四列、前三列不空、中文列不重复、说明列空着或恰好三段（用 <br> 分隔、每段不空）。
#   译文：每组三份齐全的，中文列的串在 zh 那份里原样出现，则 English 列的串要在 en 那份里出现（不分大小写），日本語列的串要在 ja 那份里原样出现。
#   改名登记表 .claude/kb/term-renames.md 那一组，登记表的旧名、新名两列（term-renames 格拿去全仓搜的字面，三份照中文原串写）不算用到术语，
#   表与读法照 scripts/kb/sweep-term.py 的 load_renames、table_cells；同一份里别处用到的照核。
#   术语表文件不在：判红，不是 77，出路写建它。
# untranslated：每组在的英语、日语译文逐行判，射程是全仓此刻的样子（同 files、structure、glossary，不按这次改动划窗：伪译文是文件的状态，checks-owed 日语那份约 793 行
#   中文原句只换了术语表的词，结构与术语两格都判不出，之后的改动也没再碰到那些行）。每行先去掉闭合的围栏代码块、HTML 注释、反引号段、链接目标 ](…)、
#   「」『』与英语直弯引号里的引文、带 ASCII 字母或数字的路径；剩下的部分英语数汉字、≥ 15 个记一处，日语数日语不用的简体中文字（GB 2312 收而 JIS X 0208 没收的
#   3432 个，常量 SIMPLIFIED_ONLY）、≥ 3 个记一处。不到门槛的行，整行与同组中文那一份某一行逐字相同（照抄没译）、剩下部分还有被数的字（英语任一汉字，
#   日语任一日语不用的简体字，常量 IDENTICAL_THRESHOLD）的，也记一处，红行写「与中文逐字相同（中文那一份:行号）」；逐字相同而剩下部分没有被数的字的放过，成功那句报放过几行。
#   红行先按文件列红行数，再报文件:行号、剩多少个与片段。整份照设计三语并列的组（术语表、改名登记）
#   登记在 .claude/gate.d/trilingual-untranslated-exempt.tsv 不判，登记写错的逐行判红。门槛与字表的依据、量过的数在 lib-trilingual.py 文件头「判法细节」。
#   一份译文都没判到：退 77。
# omission：三份齐全的组，英语、日语两份各与中文那一份配对比内容量，射程同 untranslated 是全仓此刻的样子（漏段与压成摘要是文件的状态：E58 英语漏的那一段、
#   E178 英语整页压成的摘要都是早先写进、之后的改动没再碰到的）。围栏外的 ATX 标题把三份切成节（标题个数对不上的那一份不比，归 structure）；
#   节里的段是空行隔开的几行，列表项另起一段，表格行一行一段；首列带编号（C、D、E、I）的表格行按编号（连同第几次出现）配对，其余按节内次序配对；
#   闭合的围栏代码块、HTML 注释与三语切换行不算段。每对算内容量：先去掉反引号段、链接目标与带 ASCII 的路径，中文数汉字加 ASCII 词，英语数词（加汉字），
#   日语数汉字与假名加 ASCII 词；中文那一段不到 20 的不比。比值（译文 ÷ 中文）英语低于 0.42 或高于 1.5、日语低于 0.9 或高于 3.3 记一处，
#   报译文的文件:行号、比值、中文那一份的文件:行号、两边的量与中文片段；一节里段数对不上、或编号行只在一边的，另报一类（配不上），不比这一节的比值。
#   红的先按文件列处数。门槛的量法、误报与漏报各一例在 lib-trilingual.py 文件头「判法细节」的 omission 一段。三份齐全的组一组都没有：退 77。
# 登记表不在：六格都判红（对象从它取）。
# 管不到的：译文译得对不对（omission 只比内容量：比值在门槛里而意思译错的、漏了一句而比值还在门槛里的判不出）；术语在句子里用得自然不自然；链接目标的写法在三份里逐字相同（不认语言后缀的映射）；
#   不到门槛、又不是与中文逐字相同的零星残留（英语一行 14 个以内的汉字，日语一行 2 个以内的简体字，例：没译的统计量名）；
#   与中文逐字相同、日语里只剩日语也用的汉字的照抄行（标题、表头写成「概要」这类）；日语里只用日语也用的字写成的中文句子（数不到简体字）；
#   放进引号或反引号里的大段中文（引号是给原话、名字留的，放错了判不出）。
# ── private-values ──
# 私有值：仓根被 git 忽略的 multi-host.env（第二台的主机名、路径、清场命令里的词）与本机主机名，不许写进这次改动新加的行。
# 配置在哪：SINGLEFS_MULTI_HOST_CONFIG 设了照它；没设取 git common-dir 的上一层（主工作树的根，在 --staged 的临时 worktree 里也读得到主工作树那一份）下的 multi-host.env，
#   不在 git 仓里取被判的根下的 multi-host.env（找法同 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file）。
#   被判的根是样本（不是这道阶段所在的仓、也没设 GATE_STAGED_FROM）时只认样本根下的 multi-host.env 与 .private-values-hostname（本机主机名的替身），不读环境变量，样本不随调用方的配置变。
# 取哪些词：每个键的值按 shell 的写法切词（再按 , ; @ 切一遍），只取 8 个字以上的长词（PEER_MEMORY_CAP、ENABLE_ACROSS_MACHINES、ENABLE_GPU 三个键与 systemctl、is-active 这类命令里的通用词不取）；
#   本机主机名取 hostname（短于 8 个字、或就是 localhost 的不取）。门槛 8 个字照攻方的量：4–7 个字的词（清场命令里的选项、显卡配置里的机器标签）在冻结副本里各命中 221–2001 份文件，
#   8 个字以上的只命中两份真泄漏（research/prompts/m3-defs-b3-r1-opus-output.md「E3」）。
# 判什么：这一次改动新加的行（scripts/gate/changed-paths.sh 的 gate 取法的基准到工作区；新文件整份都算新加），行里有任何一个长词（逐字、区分大小写的子串）就红。
#   不判全仓：旧行里已有的两处不是这一次加的，判全仓就一直红，那两处的去向交用户。
#   未跟踪的新文件整份逐行读，不截断；大到没法整份比（超过 1 GiB）的判红、出路是拆小或去掉重复内容。前 8 KiB 里有 NUL 的文件仍按二进制整份跳过，在「没判的」里报个数。
#   改动范围里每个存在的文件的路径也比同一批长词（二进制的也比），命中就红。
#   符号链接先判，不跟进去：比路径与链接文字（git 进仓的是链接文字本身），坏链接也比、好链接不读它指到的内容；链接文字里有长词就红。
#   读不了的文件（权限这一类）判红，列成「读不了」一类红行，路径经同一份打码，不让异常回溯把原路径打出来。
# 输出：只报文件、行号与键名（本机主机名写「本机 hostname」），不打印值；「取不到 … 相对基准」那一句里的路径同样经打码。路径里含私有值的那一段（文件名或目录名）打成「〔文件名含私有值〕」，红行里的路径、文件名那一类红行都这样打。
#   路径比对按私有值整串在整条路径里找，不逐段比：值自己带 `/`（真配置的长词里多数是路径）、从第二台把目录 tar 过来而整段仓路径成了本地路径的一部分时，值跨了几段目录，
#   逐段比哪一段都不含整串，红行会把值原样打出来。命中的那一段是从值开头所在的那一级目录到结尾所在的那一级，重叠或相连的命中并成一段，整段打成「〔文件名含私有值〕」，值不打印。
#   出路：换成「第二台」「本机」这类说法；文件名含值的，改名。
#   取词与路径打码是同一道里的共用函数（PRIVATE_VALUES_LIBRARY：load_private_targets、shown_path、shown_text、snippet_bounds、shown_lines_main；配置在哪由 private_value_sources 解），
#   prime-marks / clock-times / session-names 三格打路径与片段、date-once 打红行路径、term-renames 打 sweep-term 的输出都调同一份，不各抄一份。
# 没有 multi-host.env（别人的克隆、第二台）、取不到够长的词、不在 git 仓里、这次改动没有新加的行：退 77，不判绿。
# 管不到的：换了写法的值（大小写、拆开写、换成别称）、用户原话里的别称（不在配置里）、commit message（提交之后才有，要另挂 commit-msg 钩子）、不足 8 个字的值。
#   词源只有 multi-host.env 与 hostname：~/.config/singlefs/ 下的清场脚本里的词没并进来（并入前要在真树上整仓逐行量新增命中，新增为 0 才并；上一次量出来不是 0，没并）。
# ── account-quota ──
# 记录、kb 页与三方判决正文里不写账号切换与用量限额（用户 2026-10-18：「改一下 doc-lint 账号切换  限额 这种就不要记了」）：撞了哪种额度、换没换账号、额度什么时候重置，
# 这些是子 agent 停下的原因，不是要留下的事实；子 agent 因此停下的，只写「中途停下没交回」。
# 落点在这一道而不是上游 scripts/doc-lint.sh：那一份只扫 kb 目录，不扫 records/ 与 research/prompts/；这一道本来就有判「这次改动新加的行」的格（private-values）。
# 射程：这次改动新加的行（scripts/gate/changed-paths.sh 的 gate 取法的基准到工作区；未跟踪的新文件整份都算新加；旧行不报），只看三类文件：
#   records/ 与 .claude/kb/ 下的 .md、research/prompts/ 顶层的 *-main-verification.md（三方判决正文）。
#   research/prompts/ 里别的文件（_*-appendix.md、_*-background.md、_*-body.md、*-snapshot/ 下的副本）是从仓里抄给云端腿读的，抄进来的旧话不算这一次写的，不判。
#   .claude/rules/、.claude/agents/、钩子与脚本不在射程里（派发闸「限额窗口已判」这类放行行是闸的功能，写在定义与钩子里）。
# 词表在 cell_account_quota 的 python 里，STRICT 与 BARE 两张表，唯一一处，每条写明键（红行点名它）和拦什么：
#   明说层（STRICT）：句子里写明哪个账号、哪一种限额（会话、周、每周、五小时、Opus、Sonnet、Haiku、Claude、API、账号、订阅后面接限额或额度），或是限额通知原文，
#     或「切换账号」「换了账号」「账号切换」「另一个账号」，或「限额窗口」「额度窗口」「会话上限」「周上限」，命中就红，不看同一行别的词。
#   裸词层（BARE）：没写是哪一种的「撞限额」「撞了额度」「限额撞顶」「额度恢复」「额度重置」「被额度打断」「额度用完」「用量限额」「用量上限」；命中处前后 24 个字内有显卡、GPU、显存、内存、线程、
#     账上、磁盘、空间、inode、文件系统、存储、quota 或「数字加 G / GiB / GB / MiB / MB」的放过（只看命中处附近，不看整行）：那是显卡额度、内存额度、空间配额、账上多少 G 这类算力与空间资源的正常用法。
#   不在词表里的：「额度」「限额」「上限」「配额」单独出现（显卡额度、GPU_CARDS_BY_PRIORITY 的额度、内存上限、空间配额）；「欠账号」（欠账编号）；「换号」「重写换号」（版本号）；
#     「账户」；「限流」单独出现（只有跟在「撞」后面才算）。
# 放行：行里写「account-quota:allow <理由>」（理由至少 4 个字）整行放过，md 里也认；没写理由的标记不放过，另报一类红行。用在用户原话里带这些词、要原样留的地方，和引用 three-way-inference.md 那条规则标题的地方。
# 误伤面（2026-10-19 对着当时工作区里射程内 719 份文件、12 万行量的）：词表命中 72 行，逐行读过，没有一行是显卡、内存或空间配额的用法；资源词放过 1 行（撞额度：账上 8G，内存额度）。
#   裸词层最初是整行有资源词就放，量出来放过 3 行里有 2 行其实是模型限额（行里别处提到线程上限、草稿目录多少 G），所以改成只看命中处前后 24 个字。
#   会误伤的：整份搬家后算新加的旧页里带这些词的行、测试代码里为了造限额通知而写的字面；出路都是删掉这句，或写 account-quota:allow。
#   漏过的：换了写法的说法（「登录失效」「额度吃紧」「服务端过载」）、命中处附近恰好有资源词的裸词命中（「内存」「线程」离得近的模型限额）、射程之外的文件。
# 没有 git 仓、这次改动没有新加到射程内的行：退 77，不判绿。
# 管不到的：用户原话之外把这些话改写成别的词的；射程之外的文件；已经在仓里的旧行（那 72 行去留交用户）。
# ── out-of-project-names ──
# 进仓的文字只写本项目的东西（.claude/agent-common.md「写」一节「进仓的文字只写本项目的东西」那一条）：本机与第二台上不归本项目的目录名、路径、别的工程与它们的占用不写进这次改动新加的行。
# 射程与读法同 private-values：这次改动新加的行（scripts/gate/changed-paths.sh 的 gate 取法的基准到工作区；未跟踪的新文件整份都算新加），新加行经共用库的 added_lines 取，
#   红行路径经同一份 shown_path 打码；二进制（前 8 KiB 里有 NUL）与符号链接不判，报个数。
# 名字从哪来（门禁跑时现读，名字不进仓）：本机家目录（$HOME）下一层的条目名；仓的上一级目录里的条目名（仓取 git common-dir 的上一层，--staged 的临时 worktree 里也是主工作树；
#   设了 GATE_OUT_OF_PROJECT_PARENT 照它，工作根下的仓副本回扫时指到主工作区的上一级）；第二台的同类条目名在私有名单 ~/.config/singlefs/out-of-project-names
#   （一行一个名字，# 开头是注释；不在就打一行 !、往 $GATE_NOT_RUN_FILE 报「本次未查」，别的来源照判，不判红）。
#   排掉 singlefs 自己的：singlefs、singlefs- 开头的（上游 singlefs-ai-sop-* 在内）与仓自己的目录名。
# 名字类：只在像目录的上下文里判，正文里当普通词用的不判。
#   路径段：~/<名>、$HOME/<名>、${HOME}/<名>、家目录绝对路径加 /<名>、/home/<用户>/<名>（家目录与私有名单里的名字）；
#     仓的上一级目录加 /<名>（绝对路径与 ~/ 开头的几种写法）、../<名>（上一级目录与私有名单里的名字；名字也是仓根下一层的条目的不判，那是仓里自己的相对路径，例 ../.claude/kb）。
#   反引号：整个反引号里只有这个名字（可带一个收尾的 /；只由小写 ASCII 字母组成的名字不判，那多半是正文里的普通英文词）。带容量：名字后面紧跟一个容量（中间只许夹收尾的反引号、空白、一个左括号或冒号或「约」；容量是数加 K、M、G、T，可带 i 与 B）。
#   反引号与带容量两种不判仓根下一层也有的名字（同名的那个多半说的是仓里那一份）。
#   带容量也不判路径的深层段：名字前面紧挨着 /、/ 前面还有一段，而这个 / 不是家目录或仓的上一级目录前缀的收尾（例 ~/.local/share/singlefs/work/<轮>/bin 里的 bin）；
#     那条路径归不归本项目交路径段按它的第一段判，放行路径底下的深层段照放行表放过。~/<名>、../<名>、<上一级>/<名> 里的名字是第一段，照判。反引号里名字就是整个内容，前面不会紧挨 /。
# 容量类：根下一层的系统目录（现读 / 下一层的目录）写成 /<名>（可带收尾的 /）而后面紧跟容量的，判红；不带容量的路径（/<名>/log/… 这类）不判。
# 放行：放行表 .claude/out-of-project-paths 一行一条「<路径>  # 理由」（路径以 ~/ 或 / 开头，理由至少 8 个字）；命中的名字落在一条放行路径的命中范围里才放过，同一个名字单独出现照判。
#   ~/ 开头的那一条认 ~/、$HOME/、${HOME}/、家目录绝对路径与 /home/<任一用户>/ 几种写法，路径后面不能紧接名字字符（字母、数字与 . _ + -）；
#     另认 XDG 回退写法 ${XDG_CACHE_HOME:-$HOME/.cache}/singlefs（XDG_CONFIG_HOME 对 .config、XDG_DATA_HOME 对 .local/share、XDG_STATE_HOME 对 .local/state 同样）
#     与 python 拼接 os.path.expanduser("~/.local/share"), "singlefs"（家目录那一段的字串后面逐段跟着引号括起的各段）；末段是 singlefs 的，singlefs-<后缀> 也认。
#     只放到放行路径那一段为止：停在 XDG 基目录（.cache、.local/state 这一段）的路径与基目录带容量的写法照判。
#   singlefs 自己的路径（家目录或上一级目录下 singlefs 开头的、../singlefs…）整段放过，里面那一级上一级目录名不算命中。
#   放行表每一条要有至少一份 git 跟踪的 .sh、.py、.rs 字面引到（同样认那几种写法，预筛只要路径各段的字面都在），引不到判红；写法不对判红；表不在：没有放行，不判红。
# 输出：红行只报文件、行号、类别与来源（本机家目录、仓的上一级目录、第二台私有名单、根下一层的系统目录），不打印命中的名字（名字本身就是要藏的）；放行表的红行只报行号。
# 样本根（判法同 private-values 的 private_value_sources）只认替身，不读本机：.out-of-project-home（首行家目录路径，其余一行一个条目名）、
#   .out-of-project-parent（首行上一级目录路径，其余一行一个条目名）、.out-of-project-root（根下一层的目录名）、.out-of-project-private-names（私有名单的替身，不在即本次未查）；
#   样本的 .gitignore 把替身、setup.sh 与 expect 挡在改动与「git 跟踪的脚本」之外；样本不往 $GATE_NOT_RUN_FILE 写。
# 回扫：--check out-of-project-names --all-lines 对 git ls-files 列出的每份文本文件整份判（不限新加的行），只报每份的命中处数，不打印名字与行号；只读，有命中退 1。
# 没有 git 仓、这次改动没有新加的行而放行表也没毛病：退 77，不判绿。
# 管不到的：没有像目录的上下文的名字（正文里写某某目录占了多少，名字不在反引号里、后面也不跟容量）；换了写法的名字（大小写、拆开写、别称）；
#   本机与私有名单之外的机器；commit message（提交之后才有）。名字里带名字字符之外的字的，不切 token，逐个比。
#
# 弄坏开关（打开后样本判错；每条下面逐行点名判错的样本）：
#   GATE_TEXT_NOTATIONS_BREAK=utf8-valid-lenient  非法字节按替换字符放过（解码用 replace，不报错）
#     utf8-valid-chunk-red
#     utf8-valid-red
#     utf8-valid-staged-red
#   GATE_TEXT_NOTATIONS_BREAK=utf8-valid-index-unjudged  不判暂存区里与工作区不同的那一版
#     utf8-valid-staged-red
#   GATE_TEXT_NOTATIONS_BREAK=utf8-valid-chunk-carry-dropped  块尾被切开的多字节序列不带到下一块、当坏字节报
#     utf8-valid-chunk-green
#   GATE_TEXT_NOTATIONS_BREAK=date-once-unjudged  date-once 不干跑、记无对象可判
#     date-once-green
#     date-once-history-translations-green
#     date-once-history-translations-red
#     date-once-red
#     date-once-unjudgeable-red
#     private-values-date-once-path-red
#   GATE_TEXT_NOTATIONS_BREAK=date-once-exit-ignored  date-once 不收干跑的退出码（改前的样子）：干跑出错的那一份读不到摘要、当干跑 0 处
#     date-once-unjudgeable-red
#   GATE_TEXT_NOTATIONS_BREAK=date-once-history-translations-judged  date-once 退回只排除中文那三份（两份变更史与 INDEX.md），英日译文照判（改前的样子）
#     date-once-history-translations-green
#     date-once-history-translations-red
#   GATE_TEXT_NOTATIONS_BREAK=term-renames-unjudged  term-renames 不调 sweep-term、记无对象可判
#     term-renames-green
#     term-renames-line-exempt-green
#     term-renames-line-exempt-red
#     term-renames-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-files-unjudged  trilingual-files 不判、记无对象可判
#     trilingual-files-green
#     trilingual-files-no-registry-red
#     trilingual-files-red
#     trilingual-files-registry-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-structure-unjudged  trilingual-structure 不判、记无对象可判
#     trilingual-structure-green
#     trilingual-structure-link-target-red
#     trilingual-structure-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-changed-together-unjudged  trilingual-changed-together 不判、记无对象可判
#     trilingual-changed-together-green
#     trilingual-changed-together-red
#     trilingual-changed-together-translation-only-green
#     trilingual-changed-together-unrun
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-glossary-unjudged  trilingual-glossary 不判、记无对象可判
#     trilingual-glossary-green
#     trilingual-glossary-longer-word-green
#     trilingual-glossary-longer-word-red
#     trilingual-glossary-longer-words-table-red
#     trilingual-glossary-marker-red
#     trilingual-glossary-missing-red
#     trilingual-glossary-not-table-red
#     trilingual-glossary-red
#     trilingual-glossary-renames-columns-red
#     trilingual-glossary-table-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-unjudged  trilingual-untranslated 不判、记无对象可判
#     trilingual-untranslated-english-red
#     trilingual-untranslated-exempt-table-red
#     trilingual-untranslated-green
#     trilingual-untranslated-identical-green
#     trilingual-untranslated-identical-red
#     trilingual-untranslated-japanese-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-quotes-kept  不去掉「」『』与英语直弯引号里的引文（这一格转成 LIB_TRILINGUAL_BREAK 同名一项交给 lib-trilingual.py）
#     trilingual-untranslated-green
#     trilingual-untranslated-identical-green
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-code-kept  不去掉反引号段、围栏代码块、链接目标、路径与 HTML 注释（转法同上）
#     trilingual-untranslated-green
#     trilingual-untranslated-identical-green
#     trilingual-untranslated-identical-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-japanese-unjudged  不判日语译文（转法同上）
#     trilingual-untranslated-green
#     trilingual-untranslated-identical-green
#     trilingual-untranslated-identical-red
#     trilingual-untranslated-japanese-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-exempt-ignored  不读排除登记表（转法同上）
#     trilingual-untranslated-exempt-table-red
#     trilingual-untranslated-green
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-japanese-threshold-five  日语门槛退回 5（收到 3 之前的样子；转法同上）
#     trilingual-untranslated-green
#     trilingual-untranslated-japanese-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-untranslated-identical-ignored  不判与中文逐字相同的行（没有这一条之前的样子；转法同上）
#     trilingual-untranslated-identical-green
#     trilingual-untranslated-identical-red
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-omission-unjudged  trilingual-omission 不判、记无对象可判
#     trilingual-omission-english-red
#     trilingual-omission-green
#     trilingual-omission-japanese-red
#   LIB_TRILINGUAL_BREAK 的各项关掉 lib-trilingual.py 里的一条判据（那份文件头「弄坏开关」一段逐项写弄坏哪一处）：
#   LIB_TRILINGUAL_BREAK=registry-reason-ignored  登记表的行不要求理由
#     trilingual-files-registry-red
#   LIB_TRILINGUAL_BREAK=registry-unmatched-ignored  模式指不到任何文件不报
#     trilingual-files-registry-red
#   LIB_TRILINGUAL_BREAK=registry-suffix-ignored  模式带语言后缀不报
#     trilingual-files-registry-red
#   LIB_TRILINGUAL_BREAK=reverse-ignored  不看目录里单有 .en.md / .ja.md 的那一组
#     trilingual-files-red
#   LIB_TRILINGUAL_BREAK=ids-ignored  不比编号集合
#     trilingual-structure-link-target-red
#     trilingual-structure-red
#   LIB_TRILINGUAL_BREAK=word-boundary  编号前后的限制换成 \b
#     trilingual-structure-green
#   LIB_TRILINGUAL_BREAK=links-ignored  不比链接目标集合
#     trilingual-structure-red
#   LIB_TRILINGUAL_BREAK=link-language-kept  链接目标里的 .en.md / .ja.md 不归成 <名>.md
#     trilingual-structure-green
#     trilingual-structure-link-target-red
#   LIB_TRILINGUAL_BREAK=headings-ignored  不比标题个数
#     trilingual-structure-red
#   LIB_TRILINGUAL_BREAK=table-rows-ignored  不比表格行个数
#     trilingual-structure-red
#   LIB_TRILINGUAL_BREAK=fences-ignored  不比围栏代码块个数
#     trilingual-structure-red
#   LIB_TRILINGUAL_BREAK=fence-blind  围栏里的行也当正文数标题与表格行
#     trilingual-structure-green
#   LIB_TRILINGUAL_BREAK=link-target-ids-counted  编号集合也数链接目标 ](…) 里的编号（改前的样子）
#     trilingual-structure-link-target-red
#   LIB_TRILINGUAL_BREAK=partial-ignored  碰了一份就当三份都碰了
#     trilingual-changed-together-red
#   LIB_TRILINGUAL_BREAK=translation-only-required  只碰了译文（中文那一份没碰）的组也要三份都碰（改前的样子）
#     trilingual-changed-together-red
#     trilingual-changed-together-translation-only-green
#   LIB_TRILINGUAL_BREAK=glossary-marker-ignored  找不到标记不报
#     trilingual-glossary-marker-red
#   LIB_TRILINGUAL_BREAK=glossary-header-ignored  表头对不上不报
#     trilingual-glossary-table-red
#   LIB_TRILINGUAL_BREAK=glossary-columns-ignored  列数不对、前三列有空不报
#     trilingual-glossary-table-red
#   LIB_TRILINGUAL_BREAK=glossary-duplicate-ignored  中文列重复不报
#     trilingual-glossary-table-red
#   LIB_TRILINGUAL_BREAK=glossary-note-ignored  说明列的三段不判
#     trilingual-glossary-table-red
#   LIB_TRILINGUAL_BREAK=glossary-translation-ignored  不核译文用词
#     trilingual-glossary-longer-word-red
#     trilingual-glossary-red
#     trilingual-glossary-renames-columns-red
#   LIB_TRILINGUAL_BREAK=glossary-case-sensitive  English 列按大小写比
#     trilingual-glossary-green
#   LIB_TRILINGUAL_BREAK=glossary-zh-gate-ignored  中文术语不在 zh 里也核译名
#     trilingual-glossary-green
#     trilingual-glossary-longer-word-green
#     trilingual-glossary-longer-word-red
#     trilingual-glossary-longer-words-table-red
#     trilingual-glossary-red
#     trilingual-glossary-renames-columns-red
#     trilingual-glossary-table-red
#   LIB_TRILINGUAL_BREAK=glossary-substring  退回按子串认中文术语（不读更长词登记文件）
#     trilingual-glossary-longer-word-green
#     trilingual-glossary-longer-word-red
#     trilingual-glossary-longer-words-table-red
#   LIB_TRILINGUAL_BREAK=glossary-renames-columns-counted  改名登记表的旧名、新名两列也算用到术语（改前的样子）
#     trilingual-glossary-renames-columns-red
#   LIB_TRILINGUAL_BREAK=omission-ratio-ignored  trilingual-omission 不比内容量
#     trilingual-omission-english-red
#     trilingual-omission-japanese-red
#   LIB_TRILINGUAL_BREAK=omission-keys-ignored  trilingual-omission 的表格行不按首列编号配对，一律按节内次序
#     trilingual-omission-english-red
#     trilingual-omission-green
#   LIB_TRILINGUAL_BREAK=omission-unpaired-ignored  trilingual-omission 配不上的段（段数不同的节、编号行缺一边）不报
#     trilingual-omission-english-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-unjudged  private-values 不比对、记无对象可判
#     private-values-big-untracked-red
#     private-values-cross-segment-red
#     private-values-filename-and-content-red
#     private-values-filename-only-red
#     private-values-green
#     private-values-no-config
#     private-values-nul-head-green
#     private-values-red
#     private-values-siblings-red
#     private-values-symlink-dangling-red
#     private-values-symlink-mount-red
#     private-values-too-big-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-short-words  门槛从 8 个字降到 4 个字
#     private-values-green
#     private-values-nul-head-green
#     private-values-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-whole-tree  不限新加的行、逐份文件整份比
#     private-values-green
#     private-values-nul-head-green
#   GATE_TEXT_NOTATIONS_BREAK=private-values-hostname-skipped  不取本机主机名
#     private-values-green
#     private-values-nul-head-green
#     private-values-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-print-values  红行里把命中的值也打出来
#     private-values-big-untracked-red
#     private-values-filename-and-content-red
#     private-values-red
#     private-values-symlink-dangling-red
#     private-values-symlink-mount-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-big-file-truncated  未跟踪文件只读前 8 MiB、超大也不判红（改前的样子）
#     private-values-big-untracked-red
#     private-values-too-big-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-names-unjudged  不比文件名
#     private-values-cross-segment-red
#     private-values-filename-and-content-red
#     private-values-filename-only-red
#     private-values-siblings-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-path-printed  红行里的路径原样打（含私有值的那一段也打）
#     private-values-cross-segment-red
#     private-values-date-once-path-red
#     private-values-filename-and-content-red
#     private-values-filename-only-red
#     private-values-siblings-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-siblings-unmasked  prime-marks / clock-times / session-names 三格打路径与行文片段不打码（改前的样子）
#     private-values-clock-snippet-red
#     private-values-siblings-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-snippet-cut  三格的行文片段照命中位置前 20 后 10 个字直接截，不挪到私有值的边上（改前的样子）
#     private-values-clock-snippet-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-date-once-term-renames-unmasked  date-once 的红行与 term-renames 转出的 sweep-term 输出原样打路径（改前的样子）
#     private-values-date-once-path-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-path-by-segment  路径打码退回逐段比（改前的判法：哪一段含整个值才打码，值带 / 跨几段时一段都不含）
#     private-values-cross-segment-red
#     private-values-siblings-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-links-followed  不判符号链接（改前的样子：跟着链接走，坏链接整份跳过、好链接读它指到的内容）
#     private-values-symlink-dangling-red
#     private-values-symlink-mount-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-unreadable-raises  读文件不包异常（改前的样子：读不了就让格崩，python 回溯把带假仓路径的原路径打出来）
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-unjudged  account-quota 不判、记无对象可判
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-words-skipped  词表与放行标记整个不判（没有这一格的样子）
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-strict-only  裸词层不判
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-resource-unexempt  裸词层不看资源词
#     account-quota-green
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-records-only  射程缩到只有 records/
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-whole-tree  不限新加的行、整份比
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-marker-ignored  不认放行标记
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-reasonless-allowed  没写理由的标记也放行
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-unjudged  out-of-project-names 不判、记无对象可判
#     out-of-project-names-allowed-red
#     out-of-project-names-deep-green
#     out-of-project-names-deep-red
#     out-of-project-names-forms-green
#     out-of-project-names-forms-red
#     out-of-project-names-green
#     out-of-project-names-no-private-list
#     out-of-project-names-private-red
#     out-of-project-names-red
#     out-of-project-names-root-red
#     out-of-project-names-table-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-path-unjudged  路径段不判
#     out-of-project-names-allowed-red
#     out-of-project-names-deep-green
#     out-of-project-names-deep-red
#     out-of-project-names-forms-green
#     out-of-project-names-forms-red
#     out-of-project-names-green
#     out-of-project-names-private-red
#     out-of-project-names-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-backtick-unjudged  反引号不判
#     out-of-project-names-allowed-red
#     out-of-project-names-forms-red
#     out-of-project-names-private-red
#     out-of-project-names-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-capacity-unjudged  名字带容量不判
#     out-of-project-names-allowed-red
#     out-of-project-names-deep-red
#     out-of-project-names-forms-red
#     out-of-project-names-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-home-unread  不读本机家目录
#     out-of-project-names-allowed-red
#     out-of-project-names-deep-green
#     out-of-project-names-deep-red
#     out-of-project-names-forms-green
#     out-of-project-names-forms-red
#     out-of-project-names-green
#     out-of-project-names-no-private-list
#     out-of-project-names-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-parent-unread  不读仓的上一级目录
#     out-of-project-names-deep-green
#     out-of-project-names-deep-red
#     out-of-project-names-forms-green
#     out-of-project-names-green
#     out-of-project-names-no-private-list
#     out-of-project-names-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-private-unread  不读第二台私有名单（当它在、却是空的）
#     out-of-project-names-deep-green
#     out-of-project-names-forms-green
#     out-of-project-names-forms-red
#     out-of-project-names-green
#     out-of-project-names-private-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-allow-whole-name  按名字放行（放行表路径里出现过的名字到处都放过）
#     out-of-project-names-allowed-red
#     out-of-project-names-forms-red
#     out-of-project-names-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-allow-ignored  不读放行表
#     out-of-project-names-allowed-red
#     out-of-project-names-deep-green
#     out-of-project-names-forms-green
#     out-of-project-names-green
#     out-of-project-names-table-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-whole-file  不限新加的行、整份比
#     out-of-project-names-deep-green
#     out-of-project-names-forms-green
#     out-of-project-names-green
#     out-of-project-names-no-private-list
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-singlefs-unexcluded  不排掉 singlefs 自己的名字与路径
#     out-of-project-names-deep-green
#     out-of-project-names-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-repository-names-judged  仓根下一层也有的名字照样按反引号、带容量判
#     out-of-project-names-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-root-unjudged  容量类不判
#     out-of-project-names-root-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-root-without-capacity  根下系统目录不带容量也判
#     out-of-project-names-deep-green
#     out-of-project-names-green
#     out-of-project-names-root-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-table-format-unchecked  不核放行表的写法、取行首一段当路径
#     out-of-project-names-table-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-references-unchecked  不核放行表有没有脚本引到
#     out-of-project-names-table-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-print-names  红行把命中的名字也打出来
#     out-of-project-names-allowed-red
#     out-of-project-names-deep-red
#     out-of-project-names-forms-red
#     out-of-project-names-private-red
#     out-of-project-names-red
#     out-of-project-names-root-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-private-missing-silent  私有名单不在时不报本次未查
#     out-of-project-names-no-private-list
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-relative-repository-judged  ../ 后面接仓根下一层也有的名字照判
#     out-of-project-names-forms-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-relative-all-unjudged  ../ 一律不判
#     out-of-project-names-deep-red
#     out-of-project-names-forms-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-xdg-fallback-unrecognized  XDG 回退写法不认
#     out-of-project-names-forms-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-python-join-unrecognized  python 拼接不认
#     out-of-project-names-forms-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-singlefs-suffix-unrecognized  末段 singlefs-<后缀> 不认
#     out-of-project-names-forms-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-other-user-home-unrecognized  放行时 /home/<别的用户>/ 不当家目录认
#     out-of-project-names-forms-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-references-literal-tail  放行表引用的预筛认整段字面
#     out-of-project-names-forms-green
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-xdg-base-allowed  XDG 基目录底下一律放过
#     out-of-project-names-forms-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-lowercase-backtick-judged  只有小写字母的名字在反引号里也判
#     out-of-project-names-deep-red
#     out-of-project-names-forms-green
#     out-of-project-names-forms-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-lowercase-everywhere-unjudged  小写词带容量也不判
#     out-of-project-names-deep-red
#     out-of-project-names-forms-red
#   GATE_TEXT_NOTATIONS_BREAK=out-of-project-names-deep-segment-judged  路径深层段照按名字类带容量判
#     out-of-project-names-deep-green
#     out-of-project-names-deep-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK 认 lib-forbidden-notations.py 的形态键与判据行键（关掉那一形态或那一条判据行），另认几项把判法退回旧样子：
#   LIB_FORBIDDEN_NOTATIONS_BREAK=prime-marks  关掉撇号类角标这一形态（它只有同名的一条判据行）
#     prime-marks-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=clock-times  关掉钟点这一形态的全部判据行
#     clock-times-mountinfo-red
#     clock-times-red
#     private-values-clock-snippet-red
#     private-values-siblings-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=session-names  关掉会话号这一形态（它只有同名的一条判据行）
#     session-names-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=date-clock  关掉「日期后面跟钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=zone-clock  关掉「时区后面跟钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=clock-zone  关掉「钟点后面跟时区」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=x-wildcard  关掉「带 x 通配的钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=bare-z  关掉「不带日期的 Z 钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=date-zone-hour  关掉「日期或时区后面跟「N 点 / N 时」」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=date-zone-period  关掉「日期或时区后面跟时段词」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=period-hour  关掉「时段词后面跟钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=hour-suffix  关掉「「N 点前后」这类钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=twelve-hour  关掉「十二小时制钟点」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=bare-clock  关掉「不挨日期时区的钟点」那一条判据行
#     clock-times-mountinfo-red
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=iso-timestamp  关掉「ISO 时间戳」那一条判据行
#     clock-times-red
#     private-values-clock-snippet-red
#     private-values-siblings-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=iso-compact-timestamp  关掉「ISO 紧凑时间戳」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=zone-word  关掉「时区词」那一条判据行
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose  紧凑时间戳那一条判据行退回「8 位数字加 T 加 4 到 6 位数字」、不核日期钟点的范围与前后字符
#     clock-times-green
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=numbered-pairs-whole-line  编号对的上下文规则退回按整行读（改前的样子：同一行里有「文件:行号」或越界冒号对，整行的冒号对都放过）
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=mountinfo-device-numbers  mountinfo 一行里的主:次设备号不放过（改前的样子，当钟点判）
#     clock-times-green
#   同一个变量的 first-hit-only（只交最靠前的一处）在这一道的样本上不判错，证红的是 .claude/hooks/write-guard.sh --selftest。
#   下面三个变量名不以 _BREAK 收尾，值写形态键，照同样写法列出判错的样本：
#   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=clock-times  那一形态判据行的上下文规则不生效，命中一律算违例
#     clock-times-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=clock-times  那一形态的放行标记不认
#     clock-times-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names  那一形态的放行标记不认
#     session-names-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=prime-marks  那一形态的排除前缀不生效
#     prime-marks-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=clock-times  那一形态的排除前缀不生效
#     clock-times-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=session-names  那一形态的排除前缀不生效
#     session-names-green
#   CONTEXT 与 ALLOW 设成 prime-marks、CONTEXT 设成 session-names 在这一道的样本上不判错（那两种形态没有上下文规则，prime-marks 的样本里没有放行标记）。
#   STAGE_CELLS_BREAK=red-swallowed  归共用库 lib/stage-cells.sh：红格按绿记、整道退 0
#     account-quota-red、clock-times-mountinfo-red、clock-times-red、date-once-history-translations-red
#     date-once-red、date-once-unjudgeable-red、out-of-project-names-allowed-red、out-of-project-names-deep-red
#     out-of-project-names-forms-red、out-of-project-names-private-red、out-of-project-names-red、out-of-project-names-root-red
#     out-of-project-names-table-red、prime-marks-red、private-values-big-untracked-red、private-values-clock-snippet-red
#     private-values-cross-segment-red、private-values-date-once-path-red、private-values-filename-and-content-red、private-values-filename-only-red
#     private-values-red、private-values-siblings-red、private-values-symlink-dangling-red、private-values-symlink-mount-red
#     private-values-too-big-red、private-values-unreadable-red、session-names-red、term-renames-line-exempt-red
#     term-renames-red、trilingual-changed-together-red、trilingual-files-no-registry-red、trilingual-files-red
#     trilingual-files-registry-red、trilingual-glossary-longer-word-red、trilingual-glossary-longer-words-table-red、trilingual-glossary-marker-red
#     trilingual-glossary-missing-red、trilingual-glossary-not-table-red、trilingual-glossary-red、trilingual-glossary-renames-columns-red
#     trilingual-glossary-table-red、trilingual-omission-english-red、trilingual-omission-japanese-red
#     trilingual-structure-link-target-red、trilingual-structure-red、trilingual-untranslated-english-red
#     trilingual-untranslated-exempt-table-red、trilingual-untranslated-identical-red、trilingual-untranslated-japanese-red、utf8-valid-chunk-red
#     utf8-valid-red、utf8-valid-staged-red
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本把登记的格都跑
#     account-quota-green、account-quota-red、clock-times-green、clock-times-mountinfo-red
#     clock-times-red、date-once-green、date-once-history-translations-green、date-once-history-translations-red
#     date-once-red、date-once-unjudgeable-red、out-of-project-names-allowed-red、out-of-project-names-deep-green
#     out-of-project-names-deep-red、out-of-project-names-forms-green、out-of-project-names-forms-red、out-of-project-names-green
#     out-of-project-names-no-private-list、out-of-project-names-private-red、out-of-project-names-red、out-of-project-names-root-red
#     out-of-project-names-table-red、prime-marks-green、prime-marks-red、private-values-big-untracked-red
#     private-values-clock-snippet-red、private-values-cross-segment-red、private-values-date-once-path-red、private-values-filename-and-content-red
#     private-values-filename-only-red、private-values-green、private-values-no-config、private-values-nul-head-green
#     private-values-red、private-values-siblings-red、private-values-symlink-dangling-red、private-values-symlink-mount-red
#     private-values-too-big-red、private-values-unreadable-red、session-names-green、session-names-red
#     term-renames-green、term-renames-line-exempt-green、term-renames-line-exempt-red、term-renames-red
#     trilingual-changed-together-green、trilingual-changed-together-red、trilingual-changed-together-translation-only-green、trilingual-changed-together-unrun
#     trilingual-files-green、trilingual-files-no-registry-red、trilingual-files-red、trilingual-files-registry-red、trilingual-glossary-green
#     trilingual-glossary-longer-word-green、trilingual-glossary-longer-word-red、trilingual-glossary-longer-words-table-red、trilingual-glossary-marker-red
#     trilingual-glossary-missing-red、trilingual-glossary-not-table-red、trilingual-glossary-red、trilingual-glossary-renames-columns-red
#     trilingual-glossary-table-red、trilingual-omission-english-red、trilingual-omission-green、trilingual-omission-japanese-red
#     trilingual-structure-green、trilingual-structure-link-target-red、trilingual-structure-red
#     trilingual-untranslated-english-red、trilingual-untranslated-exempt-table-red、trilingual-untranslated-green、trilingual-untranslated-identical-green
#     trilingual-untranslated-identical-red、trilingual-untranslated-japanese-red、utf8-valid-chunk-green、utf8-valid-chunk-red、utf8-valid-green、utf8-valid-red
#     utf8-valid-staged-red
#
# ── utf8-valid ──
# 这次改动里的文本文件必须是合法 UTF-8（待优化 58：临时 shell 与 python 改文本，`paste -d'、'` 按字节切多字节分隔符把一行写坏，暂存区里一份记录因此成了非法 UTF-8，工作区那份重暂存才修好）。
# 射程：scripts/gate/changed-paths.sh 的 gate 取法的基准到工作区、HEAD 到暂存区碰到的路径加未跟踪的新文件（与 private-values 同一份取法，阶段里不再抄）。每份先判工作区里的版本；
#   暂存区相对基准有改动（git diff --cached 列出的）、而暂存区里的版本与工作区不同的（git diff-files 列出的），暂存区那一版另判一遍（git ls-files -s 取 blob，git cat-file 读），因为提交带走的是暂存区；
#   两边相同的只判一遍，暂存区没改动的（暂存区里就是基准那一版）不判。
# 文本文件：前 8 KiB 里有 NUL 的、后缀是压缩包 / 图片 / 目标文件这类二进制的不判，逐个列在「没查的」里（超过 10 份只列前 10 个并报总数）；符号链接、gitlink、删掉的路径也不判，同样报个数。
# 判法：整份按 UTF-8 严格解码（拒绝截断的多字节序列、孤立的续字节、过长编码、代理项、超过 U+10FFFF），16 MiB 一块读、块边界上被切开的序列带到下一块，错了就报第一处坏字节：
#   字节偏移（0 起算）、行号（1 起算）、从那里起最多 4 个字节的十六进制。红行：「<路径>（工作区 或 暂存区）  字节偏移 N（第 L 行），坏字节 0x..」，路径经 private-values 那一份打码（shown_lines_main 的 lines）。
# 一份文本文件都没判到（没有 git 仓、改动里没有文本文件）：退 77，不判绿。
# 弄坏开关 utf8-valid-lenient、utf8-valid-index-unjudged、utf8-valid-chunk-carry-dropped 只在样本根里认（PRIVATE_IN_SAMPLE，与 private-values 认样本同一种判法）；真仓里设了它，这一格打一行说明、照常判。
# 管不到的：合法 UTF-8 但内容是乱码（拿 Latin-1 当 UTF-8 存回去恰好合法的）；没有 NUL、又不在后缀名单里的二进制（会被当文本判，非法就红，出路是把后缀补进名单或登记成二进制）；提交说明。
# 样本：fixtures/doc-text.sh/ 下七十九份，每份样本根放 .gate-cells 只点名它判的那一格（trilingual-* 三十份的现场都由 setup.sh 现造，日期取真实发生过的日子；private-values-siblings-red 与 private-values-unreadable-red 点名四格）：
#   date-once-red       setup.sh 建 git 仓；这一轮往记录正文加一行带文件名日期的话、往实验页加第二个同日 `### ` 小节，逐份点名判红
#   date-once-green     同一基准；加的行只带别的日期、加的小节是另一天，判绿
#   date-once-history-translations-green  setup.sh 建 git 仓；两份变更史与 INDEX.md 连同英日译文九份，这一轮各加同一天在另一节的一块（INDEX 是同一天的第二个小节）与一行重复所属日期的话，
#                       另有一份实验页加了另一天的小节：只判实验页那一份，判绿
#   date-once-history-translations-red    setup.sh 建 git 仓；决策变更史的英语译文与一份实验页的英语译文各加同一天的第二块：只点名实验页译文（1 份里 1 份），判红
#                       （排除扩到全部 .en.md 时退 77，退回只排除中文那三份时变更史译文也被点名）
#   date-once-unjudgeable-red  setup.sh 建 git 仓；这一轮加一份带不合法 UTF-8 字节的记录（doc-consolidate.py 读不了它、退 1）与一份干净的记录：
#                       点名那一份「干跑退 1，这一份没判」，「2 份记录与 kb 页里 1 份干跑出错」，判红
#   prime-marks-red     一份带「K9」加一撇的 md、一份带「G5」加两撇的 rs，逐处点名「文件:行」判红
#   prime-marks-green   只有正常写法（变体起了新名字、ASCII 单引号），外加上游副本里带角标的 md（排除前缀下），判绿
#   clock-times-red     每条判据行各有一份带那种写法的 md 或 rs（ISO 基本格式的紧凑时间戳一份：运行目录名与日志文件名里的、带 Z 或偏移或小数秒的、日期带连字符的、钟点带冒号的；
#                       光秃钟点同一行里「文件:行号」或越界冒号对写在另一句的一份），外加写在 md 里与没写理由的放行标记，逐处点名「文件:行」与判据行判红
#   clock-times-mountinfo-red  setup.sh 现造一份 md，三行形态与 mountinfo 一行相近、其实是真钟点（前面两个挂载号而后面不是路径、后面是路径而前面只有一个数、挂载点那一段不是路径），
#                       mountinfo 那条放行规则不许放过，逐行点名判红
#   clock-times-green   只有日期与近似但该放行的写法（数字后面跟 T 却不是紧凑时间戳的哈希、编号与越界的数；同一句里带「文件:行号」的编号对），.sh、.py、.rs 里带放行标记的各一行，
#                       外加钟点排除前缀下的门禁样本、hook 自检与 c510 提示，判绿
#   session-names-red   setup.sh 现写：records/会话.md 五行，标签式「（会话 <号>，<日期>）」、当主语用的、上游三语仓的、四位十六进制的、行尾的会话号各一行；
#                       crates/demo/src/no_reason.rs 里会话号带了放行标记却没写理由；research/prompts/ 下三份三方材料各一行会话号，排除前缀不罩它们：
#                       同族而没登记的轮（_m4-trunc-r3-background）、轮名只是开头相同的（_m4-trunc-r10-appendix 不归 _m4-trunc-r1-）、登记了的轮的腿报告（m4-gc-r1-opus-output，不以 _ 起头）；
#                       逐处点名判红
#   session-names-green setup.sh 现写：records/近似写法.md 四行近似但该放行的写法——crate 名（core、checker-tier、format）与上游仓目录名、mktemp 模板与 /tmp 下的缓存目录名与
#                       变量里拼的临时目录名、更长的标识符（带字母后缀）与路径里的临时目录名、认不出的会话标题「交给会话「…」」；.claude/scripts/parse.sh 里带放行标记（写了理由）的一行；
#                       会话号排除前缀下的门禁样本 .claude/gate.d/fixtures/x/red/expect 一份；research/prompts/ 下登记了会话号排除前缀的七轮三方材料各一份
#                       （_m4-batch2-code-r1-appendix、_m4-c860-r1-background、_m4-gc-r1-appendix、_m4-segment-code-r1-background、_m4-snapshot-r1-appendix、
#                       _m4-trunc-r1-background、_m4-trunc-r2-appendix）；判绿
#   term-renames-red    一份改名登记表与一份还用着旧名的 rs（旧名由几段拼出来，样本目录里不出现旧名的字面），判红
#   term-renames-green  同一张登记表，rs 里只有新名，判绿
#   term-renames-line-exempt-green  setup.sh 建 git 仓；登记表带一行「正则」，记录里只有树层、外层那一行与按行豁免的一句用户原话带旧名，
#                       git 忽略的 multi-host.env 里有旧名，判绿（正则去掉断言、扫了 git 忽略的文件都红；旧名同样由几段拼出来）
#   term-renames-line-exempt-red  同一现场，按行豁免的那份记录里另有一行没豁免的旧名：只报那一行（1 份文件、1 处），判红
#                       （按行豁免当成整份豁免就判绿，正则去掉断言、扫了 git 忽略的文件报的处数就不是 1）
#   trilingual-files-red          登记 README.md 与 briefs/*.md：README 缺日语、一组缺英语、一组只有英语、一组有英语和日语而中文不在，逐份点名缺哪份判红
#                       （reverse-ignored 抓不到后两组）
#   trilingual-files-registry-red 登记表三处不对（模式指不到文件、行没写理由、模式写成了 .en.md），逐行点名判红
#   trilingual-files-no-registry-red  没有登记表，判红（不是 77）
#   trilingual-files-green        四组每组三份齐全（一组在 ** 登记的子目录里、名里带点）；登记目录之外的单份 .en.md 与目录里另一种语言后缀的文件不归这一格管，判绿
#   trilingual-structure-red      五组各错一项（编号集合、链接目标集合、二级标题个数、表格行个数、围栏代码块个数），逐项点名各份的值判红
#   trilingual-structure-green    日文与中文里编号紧挨文字、链接带三种语言的标题、围栏里有 # 与 | 开头的行且三份个数不同、四个反引号的围栏里夹三个反引号的行，判绿
#   trilingual-structure-link-target-red  kb 页的译文链 translation-paths.json 登记的译文文件名：experiments 索引与 E14 那一页的编号只差在链接目标里
#                       （中文文件名里的 I-3.1 在日语文件名里写成 I-3-1），不报；decisions 索引日语那份的链接文字丢了 D5，只点名这一组判红
#   trilingual-changed-together-red    setup.sh 建 git 仓：只改了中文（工作区）一组、新写了中文（暂存）与英语（未跟踪）而没写日语一组，逐组点名没碰哪几份判红；
#                       另有只改了日语的一组放行（汇总报放行 1 组），三份都改的一组不报
#   trilingual-changed-together-green  同一基准：三份都改（工作区、暂存）、三份都删、三份都新写，另有一组没碰，判绿
#   trilingual-changed-together-translation-only-green  同一基准：只改译文的三组（只改日语且暂存、改英语删日语、只新写英语而中文那一份没有）放行，
#                       另有三份都改的一组；判绿，报改了中文的 1 组、放行 3 组（只改译文也要三份都碰时判红）
#   trilingual-changed-together-unrun  只改了与登记无关的文件，退 77
#   trilingual-glossary-red       表写对了，英语写 Crash-point、日语写リプレー、日语写関門：逐条点名哪一组、哪个中文术语、缺哪份的哪个译名，判红
#   trilingual-glossary-green     英语大小写与术语表不同、简报里中文没用的术语不核别的两份，判绿
#   trilingual-glossary-table-red 表头、列数、前三列有空、中文列重复、说明列段数各一处，逐行点名判红
#   trilingual-glossary-marker-red      表在而没有 <!-- trilingual-glossary --> 标记，判红
#   trilingual-glossary-not-table-red   标记下面隔了空行才是表，判红
#   trilingual-glossary-missing-red     没有术语表文件，判红（不是 77），出路写建它
#   trilingual-glossary-longer-word-green   「集合表示」「副产物」里的「合表」「产物」登记在 trilingual-glossary-longer-words.tsv、译文不带对应译名，判绿
#   trilingual-glossary-longer-word-red     同一套登记，另有真用到「产物」「合表」而译文缺译名的句子（同一行里先被盖住、后一处照核），逐处点名判红
#   trilingual-glossary-longer-words-table-red  登记文件写错：列数不对、术语不在术语表、词不含术语或等于术语、理由少于 4 个字、同一对写两次，逐行点名判红
#   trilingual-glossary-renames-columns-red     改名登记表一组：「门禁」只在新名一列、「对拍」只在旧名一列，译文里没有对应译名，不报；
#                       表上方正文的「重放」日语没用リプレイ、「是什么」那一列的「崩溃点」英语没用 crash point，只点名这两处判红
#   trilingual-untranslated-english-red    英语那份一行表格行照中文原样留着（46 个汉字）、一行正好 15 个汉字（门槛那一档），中间隔一个没闭合的开围栏（不算围栏，后面照判），逐行点名判红
#   trilingual-untranslated-japanese-red   日语那份一行是中文原句只把术语表的词换成日语的伪译文（7 个日语不用的简体字）、一行正好 3 个（门槛那一档），逐行点名判红
#   trilingual-untranslated-green          英语、日语两份只在「」、"…"、“…”、『』、套着的引号、反引号、链接目标、带 ASCII 的路径、跨行 HTML 注释与列表里缩进的围栏里带中文
#                       （每一种段各一行、各自过门槛），另有英语正好 14 个、日语正好 2 个的一行；术语表一组带过门槛的中文、登记在排除表里，判绿
#   trilingual-untranslated-identical-red    英语那份照抄中文的语言切换行（剩 5 个汉字）、日语那份照抄中文的表头（剩 1 个日语不用的简体字）与标题（剩 2 个），
#                       都不到门槛，只因与中文逐字相同逐行点名判红（点名中文那一份的行号）
#   trilingual-untranslated-identical-green  英语、日语两份各有六行与中文逐字相同：只有数和拉丁字母的表格行与分隔行，中文只在反引号、「」、链接目标、HTML 注释里的四行；
#                       日语另有照抄的标题「概要」（只用日语也用的汉字）；判绿，报放过 13 行
#   trilingual-untranslated-exempt-table-red  排除登记表五处写错（页不在、只有一列、理由太短、同一份登记两次、登记了译文那一份），译文都已译好，逐行点名判红
#   trilingual-omission-english-red    英语那份「背景」一节漏了中文第二段（中文 3 段、英语 2 段）、「回看」表 D4 那一行漏了 2026-10-21 那一块（比值 0.41，门槛那一档）、
#                       「欠账」表首列编号 C12 那一行整行没有，三类各点名一处判红；日语那份译全
#   trilingual-omission-japanese-red   日语那份段数、列表项、表格行都与中文相同，两段与两个列表项压成一句大意（比值 0.19–0.31）、一个表格行比值 0.89（门槛那一档），逐段点名判红；英语那份译全
#   trilingual-omission-green          两组译全：中文切换行下面不空行、中文段落折行、列表续行、中文多一段跨空行的 HTML 注释、围栏里三份行数不同、反引号与链接目标里的长串中文、
#                       欠账表首列编号三份次序不同而三行长短差得多、一行中文不到 20 的短行比值偏低，判绿并报比了英语 6 对、日语 6 对、没比的 8 对
#   private-values-red    setup.sh 建 git 仓提交一版干净的记录；样本根放假的 multi-host.env（假第二台主机名、假路径、一个 4 个字的假短词）与 .private-values-hostname（假本机主机名），
#                       这一轮往记录加五行：假第二台主机名、假路径、假本机主机名各一行，一行只有 4 个字的假短词，一行没有私有值；按行号点名前三行与键名，判红
#   private-values-green  同一份假配置；基准里的旧行就带假第二台主机名（只判新加的行，不报），新加的行只有 4 个字的假短词、通用词与「第二台」这类说法，判绿
#   private-values-no-config  同一现场没有 multi-host.env：退 77
#   private-values-big-untracked-red  同一份假配置；setup.sh 在 research/results/ 下造一份 9 MiB 的未跟踪日志，假第二台主机名只在最后一行（8 MiB 之后）：判红，点名那一行
#   private-values-filename-only-red  同一份假配置；未跟踪文件的名字里带假第二台主机名、内容干净：判红，路径里那一段打成「〔文件名含私有值〕」，输出里不出现假主机名
#   private-values-filename-and-content-red  同一份假配置；未跟踪文件名与第 1 行都带假第二台主机名：判红，文件名那一类与行那一类都点名，输出里不出现假主机名
#   private-values-nul-head-green  同一份假配置；未跟踪的串口日志前 8 KiB 里有 NUL，后面带假本机主机名：仍按二进制跳过，判绿并报「二进制 1 份」
#   private-values-too-big-red  同一份假配置，.private-values-size-limit 把上限设成 1000 字节，未跟踪文件 3000 字节：判红，出路是拆小
#   private-values-cross-segment-red  同一份假配置；setup.sh 在 research/results/ 下造两份未跟踪的产物，路径是从第二台 tar 过来的样子：from-peer/srv/sample-private/peer-tree/…（假仓路径带 /、跨四级目录），
#                       另一份在 sample-peer-box-17/srv/sample-private/peer-tree/… 下（假主机名与假仓路径相连，两处命中并成一段）：判红，两条路径都只剩「research/results/〔文件名含私有值〕/…」，输出里不出现假路径
#   private-values-siblings-red  同一份假配置；setup.sh 在 research/results/ 下造从第二台 tar 过来的产物目录（假仓路径整段成了本地路径的一部分）：一份干净的 .out、一份带 ISO 时间戳的 .log、
#                       一份前 8 KiB 里有 NUL 的 .img；点名 private-values、clock-times、prime-marks、session-names 四格：private-values 与 clock-times 红，prime-marks 与 session-names 绿；
#                       四格输出里路径都只剩「research/results/〔文件名含私有值〕/…」（clock-times 红行、两格的「二进制：」行），输出里不出现假仓路径
#   private-values-symlink-dangling-red  同一份假配置；setup.sh 放一份干净的新日志与一个未跟踪的符号链接，链接文字是第二台的假仓路径（本机不存在，坏链接）：
#                       判红，点名链接与键名 PEER_REPOSITORY_DIRECTORY，输出里不出现假仓路径
#   private-values-symlink-mount-red  同一份假配置；setup.sh 在样本里被 git 忽略的 mnt/ 下建以假第二台主机名命名的挂载点（目标存在、内容干净），放一个指到它的未跟踪符号链接：
#                       判红，点名链接与键名 PEER_SSH_HOST、QUIESCE_STOP_COMMAND，输出里不出现假主机名
#   private-values-unreadable-red  同一份假配置；setup.sh 造从第二台 tar 过来的产物目录（假仓路径整段成了本地路径的一部分），里面一份权限 000 的 .out，另有一份干净的新日志；
#                       点名四格：private-values 判红、列成「读不了」一类、路径打码，没有回溯；prime-marks、clock-times、session-names 绿，「读不了」清单里路径打码
#                       （以 root 跑时权限 000 也读得到，这一份判不出，不以 root 跑样本档）
#   private-values-date-once-path-red  同一份假配置；setup.sh 放一份文件名带假第二台主机名、正文一行重复标题日期的记录（不建 git 仓）：date-once 判红，
#                       红行里的路径只剩「records/〔文件名含私有值〕」，输出里不出现假主机名
#   private-values-clock-snippet-red  同一份假配置；setup.sh 建 git 仓（expect、setup.sh 与假配置被 .gitignore 挡在扫描之外），记录里两行时间戳的前 20 个字
#                       落在假仓路径、假第二台主机名中间：clock-times 判红，两行片段都从值的开头起、整值打成「〔私有值〕」，输出里不出现半截的假值
#   account-quota-red   setup.sh 建 git 仓，基准里一份记录（另带一行旧话「撞了限额」，不报）、一份 kb 页、一份判决正文都干净；这一轮往三份里加写了账号切换与用量限额的行，
#                       另放一份未跟踪的新记录，词表里每一条各一行（切换账号、换了账号、另一个账号、switched the account、撞了周额度、撞会话限流、撞限额、额度恢复、通知原文、限额窗口、用量限额、额度用完、被额度打断），
#                       外加一行写了放行标记却没写理由的；射程之外的两份未跟踪文件（_ 开头的附录、.claude/rules 下的页）带同样的话，不报；逐处点名「文件:行」与判据名判红，共 14 行
#   account-quota-green 同一基准；这一轮加的行只有显卡额度、GPU_CARDS_BY_PRIORITY 的额度、内存上限与内存额度、空间配额、欠账号、「换号」、「中途停下没交回」，命中处附近有资源词的裸词命中三行
#                       （撞额度：账上 8G、显卡额度重置、显存用量上限），带理由的放行标记一行；射程之外的未跟踪文件（附录、snapshot 副本、规则页）带「切换账号」「撞了周额度」，不报，判绿
#
#   out-of-project-names-red  样本根放替身（.out-of-project-home、-parent、-root、-private-names，假家目录与假上一级目录的条目名），setup.sh 建 git 仓；这一轮往记录加七行：
#                       反引号加容量、只有反引号、不带反引号加容量、家目录路径段两行（~/、$HOME/、/home/<用户>/）、上一级目录路径段（../、~/<上一级>/），外加一行只说本项目目录与总可用量的；
#                       逐行点名类别与来源判红，共 6 行，名字不打印
#   out-of-project-names-private-red  同一套替身；私有名单里的假名字写成路径段一行、反引号一行，来源点名「第二台私有名单」判红
#   out-of-project-names-allowed-red  放行表登记一条假路径（基准里的 scripts/fetch.sh 引到它）：这条路径的三种写法放过；同名单独出现加容量、同一目录下别的路径、多一个字的路径判红，共 3 行
#   out-of-project-names-green  同一张放行表；基准里的旧行带假名字加容量（不报）；新加的行：放行路径的三种写法、singlefs 自己的路径与名字、当普通词用的假名字、
#                       仓根下一层也有的名字在反引号里与带容量、根下系统目录不带容量的路径、只说本项目目录与总可用量的一句，判绿并报放过 5 处
#   out-of-project-names-root-red  根下系统目录的替身：反引号里的 /<名> 加容量、括号里的容量、带收尾 / 的路径加容量判红（2 行）；/<名>/log/… 不带容量、本项目目录的容量不报
#   out-of-project-names-table-red  基准里的放行表一条写对、一条没有脚本引到、一条理由太短、一条没写理由、一条路径不以 ~/ 或 / 开头：逐行点名行号判红
#   out-of-project-names-no-private-list  没有私有名单的替身：别的来源照判、新加的一行干净，判绿并报第二台本次未查
#   utf8-valid-green    setup.sh 建 git 仓：这一轮改一份记录、新加一份带四字节字符的页，另有一份带 NUL 和非法字节的二进制与一份 .gz（不判，列在没查的里），判绿并报查了 2 份工作区文件、没查 2 份
#   utf8-valid-red      工作区里一份已跟踪的记录在第 3 行被拦腰切断一个三字节汉字，一份未跟踪的页第 1 行有孤立的 0xFF，另有一份合法的：逐份点名文件、偏移、行号和坏字节，判红
#   utf8-valid-staged-red  暂存区里的记录是坏的（截断的汉字），工作区那份已经修好：点名「暂存区」而不点工作区，判红
#   utf8-valid-chunk-green  setup.sh 用 python 造一份 16 MiB 加 100 字节的页，汉字跨在第一块（16 MiB）与第二块之间，整份合法：判绿（块尾被切开的序列带到下一块）
#   utf8-valid-chunk-red    一份页在第二块里有非法字节（0xc3 后面不是续字节），偏移加上前一块的长度才是 16778216、行号第 6 行：判红并点对偏移
#   out-of-project-names-forms-green  替身家目录里有 XDG 基目录与两个小写词，私有名单里有一个仓根下也有的名字；放行表四条 XDG 下的 singlefs 路径，只由脚本里的 XDG 回退与 python 拼接引到；
#                       新加的行：../ 接仓根下也有的名字（链接、include_str!、crates/../）、四种 XDG 回退、python 拼接、singlefs-<后缀>、/home/<别的用户>/、反引号里的小写词，判绿并报放过 9 处
#   out-of-project-names-forms-red  同一套替身与放行表：../ 接仓根下没有的名字、带容量的 ~/<XDG 基目录> 与 <XDG 基目录>、停在 XDG 基目录的路径、XDG 回退与 python 拼接接到别的目录、
#                       singlefs 后面多一个字、反引号里的小写词带容量、反引号里带 - 的名字，逐行点名判红，共 8 行
#   out-of-project-names-deep-green  替身家目录里有一个小写词条目名与 .local；放行表一条 ~/.local/share/singlefs（脚本引到）；新加的行：放行路径底下、singlefs 自己的路径底下、
#                       ../singlefs… 底下与根下系统目录底下的深层段是那个小写词、带容量，判绿并报放过 1 处（.local 那一段落在放行路径里）
#   out-of-project-names-deep-red  同一套替身与放行表：那个小写词单独带容量，~/<名>、/home/<用户>/<名>、~/<上一级>/<名> 带容量（第一段，照判带容量），
#                       ~/<家目录条目名>/sub/<名> 与 ../<上一级条目名>/docs/<名> 带容量（深层段不判带容量，第一段由路径段判红），逐行点名判红，共 6 行
#
#   bash .claude/gate.d/doc-text.sh [项目根]                                     十五格都跑
#   bash .claude/gate.d/doc-text.sh --list                                       逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-text.sh --check <格名>[,<格名>…] [项目根]            只跑点名的格
#   bash .claude/gate.d/doc-text.sh --check out-of-project-names --all-lines [项目根]  回扫：全仓 git 跟踪的文本文件整份判，只报每份的命中处数（只读）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
source "$STAGE_DIR/lib/stage-cells.sh"
LIBRARY="$STAGE_DIR/lib-forbidden-notations.py"
TRILINGUAL="$STAGE_DIR/lib-trilingual.py"
SWEEP_TERM="$REPO_ROOT/scripts/kb/sweep-term.py"
DOC_CONSOLIDATE="$REPO_ROOT/scripts/kb/doc-consolidate.py"
source "$REPO_ROOT/scripts/gate/changed-paths.sh"
text_notations_break="${GATE_TEXT_NOTATIONS_BREAK:-}"

# 私有值的取词与路径、行文片段的打码：private-values 一格与 prime-marks / clock-times / session-names 三格打印路径时共用（同一份，不各抄一份）。
# python 代码放进变量，各格的 python 用 exec 载入；取词的规则写在 private-values 一格的注释里（文件头「private-values」一段）。
PRIVATE_VALUES_LIBRARY="$(cat <<'PY_PRIVATE_VALUES'
import re, shlex, socket

FILE_NAME_MASK = "〔文件名含私有值〕"          # 红行里路径的哪一段含私有值，那一段打成这个，值不打印
VALUE_MASK = "〔私有值〕"                     # 行文片段里的私有值打成这个
SKIPPED_KEYS = {"PEER_MEMORY_CAP", "ENABLE_ACROSS_MACHINES", "ENABLE_GPU"}
COMMON_WORDS = {"ssh", "sudo", "systemctl", "--user", "bash", "stop", "start", "is-active", "--quiet", "-o", "BatchMode=yes", "BatchMode",
                "true", "false", "&&", "||", "env", "sh", "-c", "exec", "test", "status", "is-enabled", "!", "nvidia-smi", "grep", "-q"}
HOSTNAME_KEY = "本机 hostname"


def split_words(text, depth=0):
    """值按 shell 的写法切词；整个命令被一对引号括成一个词的（QUIESCE_STOP_COMMAND='ssh … systemctl …'），里面带空白的词再切一层。"""
    try:
        words = shlex.split(text, comments=(depth == 0))
    except ValueError:
        words = text.split()
    pieces = []
    for word in words:
        if depth < 3 and re.search(r"\s", word):
            pieces.extend(split_words(word, depth + 1))
        else:
            pieces.append(word)
    return pieces


def load_private_targets(configuration_path, hostname_path, broken):
    """取私有词：(词表 [(键名, 词)], 不足门槛没取的词数 {键名: 个数}, 配置里取到的词数, 本机主机名那一句说明, 门槛字数)；读不了配置抛 OSError。"""
    minimum_length = 4 if broken == "private-values-short-words" else 8   # 攻方量过：4–7 个字的词各命中 221–2001 份文件，8 个字以上的只命中两份真泄漏
    targets = []              # (键名, 词)
    short_words = {}          # 键名 → 不足门槛没取的词数
    configuration_text = open(configuration_path, encoding="utf-8", errors="replace").read()
    for line in configuration_text.split("\n"):
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        if key.startswith("export "):
            key = key[len("export "):].strip()
        if key in SKIPPED_KEYS:
            continue
        for word in split_words(value.strip()):
            # 显卡先后与额度（local:0:24G,peer:1:8G 这种）整串不当私有值，只取逗号、分号、冒号、@ 切开的各段（都短，实际取不到）；别的键整词与切开的各段都取
            for piece in (set(re.split(r"[,;:@]", word)) if key.startswith("GPU_CARDS") else {word, *re.split(r"[,;@]", word)}):
                piece = piece.strip()
                if not piece or piece in COMMON_WORDS or piece.startswith("-") or piece.isdigit() or re.fullmatch(r"[0-9.]+[GMK]?i?B?", piece):
                    continue
                if len(piece) < minimum_length:
                    short_words[key] = short_words.get(key, 0) + 1
                elif (key, piece) not in targets:
                    targets.append((key, piece))
    configuration_word_count = len(targets)
    hostname_note = "本机主机名没判（不取）"
    if broken != "private-values-hostname-skipped":
        if hostname_path:
            local_names = [open(hostname_path, encoding="utf-8").read().strip()]
        else:
            local_names = [socket.gethostname()]
        local_names = [name for name in local_names + [name.split(".")[0] for name in local_names] if name]
        judged_names = [name for name in dict.fromkeys(local_names) if len(name) >= minimum_length and name != "localhost"]
        for name in judged_names:
            targets.append((HOSTNAME_KEY, name))
        hostname_note = "本机主机名判了" if judged_names else f"本机主机名没判（短于 {minimum_length} 个字或是 localhost）"
    return targets, short_words, configuration_word_count, hostname_note, minimum_length


def shown_path(path, targets, broken):
    """红行里的路径：私有值按整串在整条路径里找（值自己带 `/` 跨了几段目录的也找得到，不逐段比），命中的那一段（从值开头所在的那一级到结尾所在的那一级，
    重叠的并成一段）打成 FILE_NAME_MASK，值不打印。没有词表（没有 multi-host.env）原样交回。"""
    if broken == "private-values-path-printed":      # 弄坏：路径原样打
        return path
    if broken == "private-values-path-by-segment":      # 弄坏：改前的逐段比，值带 `/` 跨了几段时哪一段都不含整串，整条路径连值原样打出
        return "/".join(FILE_NAME_MASK if any(word in part for _key, word in targets) else part for part in path.split("/"))
    spans = []
    for _key, word in targets:
        found = path.find(word)
        while found >= 0:
            spans.append((path.rfind("/", 0, found) + 1, len(path) if path.find("/", found + len(word)) < 0 else path.find("/", found + len(word))))
            found = path.find(word, found + 1)
    merged = []
    for start, end in sorted(spans):
        if merged and start <= merged[-1][1]:
            merged[-1] = (merged[-1][0], max(merged[-1][1], end))
        else:
            merged.append((start, end))
    shown, position = [], 0
    for start, end in merged:
        shown.append(path[position:start] + FILE_NAME_MASK)
        position = end
    return "".join(shown) + path[position:]


def shown_text(text, targets):
    """行文片段里的私有值打成 VALUE_MASK（长的先换，免得短词把长词切碎）。"""
    for _key, word in sorted(targets, key=lambda pair: -len(pair[1])):
        text = text.replace(word, VALUE_MASK)
    return text


def snippet_bounds(line, start, end, targets):
    """行文片段 line[start:end] 的两头落在私有值中间时挪到值的边上（整值进片段，再由 shown_text 打码），挪到不再落在任何值中间为止；没有词表原样交回。"""
    moved = True
    while moved:
        moved = False
        for _key, word in targets:
            found = line.find(word)
            while found >= 0:
                if found < start < found + len(word):
                    start, moved = found, True
                if found < end < found + len(word):
                    end, moved = found + len(word), True
                found = line.find(word, found + 1)
    return start, end


def shown_line(line, targets, broken):
    """一行输出按空白切开：带 / 的一段当路径（shown_path），别的当行文（shown_text）。"""
    return re.sub(r"\S+", lambda found: shown_path(found.group(0), targets, broken) if "/" in found.group(0) else shown_text(found.group(0), targets), line)


def shown_lines_main(arguments, stream_in, stream_out):
    """date-once 的红行路径、term-renames 转出的 sweep-term 输出经这里打码：arguments = [配置, 主机名, 弄坏开关, path 或 lines]，
    stdin 逐行打码打到 stdout（path：整行是一个路径；lines：shown_line）。没有配置、读不了配置时原样打。"""
    configuration_path, hostname_path, broken, mode = arguments
    targets = []
    if configuration_path and broken != "private-values-date-once-term-renames-unmasked":
        try:
            targets = load_private_targets(configuration_path, hostname_path, broken)[0]
        except OSError:
            targets = []
    for line in stream_in.read().splitlines():
        stream_out.write((shown_path(line, targets, broken) if mode == "path" else shown_line(line, targets, broken)) + "\n")


ADDED_HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")


def added_lines(path, whole_file, base, git, fail, shown_path):
    """这份文件这一次新加的 (行号, 行文)：已跟踪的取相对基准 git diff -U0 的新加行，未跟踪的（新文件整份都算新加）整份逐行读。
    out-of-project-names 一格用它；git、fail、shown_path 由调用的格给（各格自己的 git 起法、红行写法与路径打码）。"""
    if not whole_file:
        difference = git("diff", "-U0", "--no-color", "--no-ext-diff", "--no-renames", base, "--", path)
        if difference.returncode != 0:
            fail(f"取不到 {shown_path(path)} 相对基准 {base[:12]} 的改动（git diff 退 {difference.returncode}）", "看基准提交在不在、git 是不是可用；改动范围的基准只经 scripts/gate/changed-paths.sh 的 gate_diff_base 取。")
        number, in_hunk = 0, False
        for line in difference.stdout.decode("utf-8", "replace").split("\n"):
            found = ADDED_HUNK.match(line)
            if found:
                number, in_hunk = int(found.group(1)), True
            elif in_hunk and line.startswith("+"):
                yield number, line[1:]
                number += 1
        return
    with open(path, "rb") as handle:
        for number, raw in enumerate(handle, 1):
            yield number, raw.decode("utf-8", "replace").rstrip("\n")
PY_PRIVATE_VALUES
)"

# private_value_sources：私有配置在哪，设 PRIVATE_CONFIGURATION_FILE / PRIVATE_HOSTNAME_FILE / PRIVATE_SIZE_LIMIT_FILE（没有的是空串）与 PRIVATE_IN_SAMPLE（1 是样本根）。
# 被判的根是样本（不是这道阶段所在的仓、也不在 gate.sh --staged 的临时树里；与共用库认 .gate-cells 同一种判法）时只认样本根下的文件，不读环境变量，样本不随调用方的配置变。
# 调用前已 cd 到被判的根。
private_value_sources() {
  local common_directory
  PRIVATE_CONFIGURATION_FILE="" PRIVATE_HOSTNAME_FILE="" PRIVATE_SIZE_LIMIT_FILE="" PRIVATE_IN_SAMPLE=0
  if [[ "$(pwd -P)" != "$(cd "$REPO_ROOT" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then PRIVATE_IN_SAMPLE=1; fi
  if (( PRIVATE_IN_SAMPLE )); then
    PRIVATE_CONFIGURATION_FILE="$ROOT/multi-host.env"
    [[ -f "$ROOT/.private-values-hostname" ]] && PRIVATE_HOSTNAME_FILE="$ROOT/.private-values-hostname"
    [[ -f "$ROOT/.private-values-size-limit" ]] && PRIVATE_SIZE_LIMIT_FILE="$ROOT/.private-values-size-limit"
  elif [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
    PRIVATE_CONFIGURATION_FILE="$SINGLEFS_MULTI_HOST_CONFIG"
  elif common_directory="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    PRIVATE_CONFIGURATION_FILE="$(dirname "$common_directory")/multi-host.env"
  else
    PRIVATE_CONFIGURATION_FILE="$ROOT/multi-host.env"
  fi
}

stage_cell prime-marks cell_prime_marks "撇号类角标" \
  "给变体起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），撞号先查，全仓一次改完：.claude/rules/path-moves.md「变体起新名字，不用角标」"
stage_cell clock-times cell_clock_times "描述里的钟点、时区词与时间戳" \
  "时间只写到日期，钟点、时区词、时间戳连同「前后」一起删；代码真要用这些字面的那一行写「# clock-times:allow <理由>」，整份是被测输入的登记进 lib-forbidden-notations.py 的 excluded_prefixes"
stage_cell session-names cell_session_names "会话号（ListAgents 列的会话名）" \
  "会话号不写进仓：标签式的「（会话 <号>，<日期>）」只留日期，当主语用的写成「别的会话」这类不带号的说法；代码真要用的那一行写「# session-names:allow <理由>」"
stage_cell term-renames cell_term_renames "登记过改名的术语不再出现旧名" \
  "python3 scripts/kb/sweep-term.py --apply 一次换完（跑不出来的产物一个字节都不许动，连同引它的正文留旧名）；确实该留旧名的登记进 .claude/term-rename-exempt 并写明为什么"
stage_cell date-once cell_date_once "这次改动碰到的记录与 kb 页里，日期只在标题上：正文行不重复所属标题或文件名里的日期，同一天不开第二个同级小节" \
  "python3 scripts/kb/doc-consolidate.py migrate <那一份> --out <临时文件> 看它删了哪些、并了哪些，照着改；红行列的是每份删了几个日期、并了几个小节"
stage_cell trilingual-files cell_trilingual_files "登记的三语文件三份齐全：中文 <名>.md 的 <名>.en.md、<名>.ja.md 都在，登记目录里单有 .en.md 或 .ja.md 的也要三份齐全；登记表写对了" \
  "缺哪份补哪份：<名>.md 是中文（默认），<名>.en.md 是英语，<名>.ja.md 是日语，三份同时在（怎么改照 .claude/skills/trilingual/SKILL.md）；登记表 .claude/trilingual-documents 一行一条「<模式>  # 理由」，模式指中文那一份、要指到文件、理由不许省"
stage_cell trilingual-structure cell_trilingual_structure "三语文件每组三份的结构对得上（编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数），抓只改了一份" \
  "照点名的组和项，把落后的那几份改到与改了的那份一致（另一份多出来的内容就补译，误删就恢复）；围栏代码块里的 # 开头行与 | 开头行不算标题与表格行"
stage_cell trilingual-changed-together cell_trilingual_changed_together "这次改动碰了一组三语文件里的中文那一份，英语、日语两份都要碰；只改译文的组放行" \
  "改了中文那一份就要三份一起改：没碰的译文照中文的改动同步译过去；只改译文的组不要求同改；这一组其实不该改的，把误碰的那份改回去"
stage_cell trilingual-glossary cell_trilingual_glossary "翻译术语表写对了，三语文件的译文照术语表用词" \
  "表的毛病照点名的行改 .claude/kb/translation-glossary.md（每行四列、前三列不空、中文列不重复、说明列空着或恰好三段用 <br> 分隔）；译文的毛病改译文，zh 里用了某术语，en 和 ja 里就用术语表给的译名；术语表文件不在就建它"
stage_cell trilingual-untranslated cell_trilingual_untranslated "三语文件的英语、日语译文去掉反引号、引文、链接目标与路径之后，一行里不残留超量没译的中文（英语数汉字，日语数日语不用的简体字），与中文那一份某一行逐字相同而去段后还剩中文的行也不许有" \
  "照中文那一份同一行整行译，反引号、路径、链接目标逐字照中文（与中文逐字相同的照抄行照它点名的中文那一行译）；要原样留的中文（用户原话、变异名、文件名）放进「」『』或反引号；整份照设计三语并列的组把中文那一份登记进 .claude/gate.d/trilingual-untranslated-exempt.tsv（一行「<路径><制表符><理由>」）"
stage_cell trilingual-omission cell_trilingual_omission "三语文件的英语、日语译文与中文逐段配对比内容量（表格行按首列编号配对，其余按节内次序）：过短（漏了一段、压成摘要）、过长（对错了段）与配不上的段（段数不同的节、编号行缺一边）都不许有" \
  "照点名的那一段把译文对齐中文：过短的补上漏的那一段、把压成摘要的照中文逐句译回去，过长的多半对错了段、照中文那一段改；段数不同的节照中文逐段对齐，编号行补上缺的那一行或删掉多出来的（判法与门槛在 lib-trilingual.py 文件头「判法细节」的 omission 一段）"
stage_cell private-values cell_private_values "这次改动新加的行与改动的文件名里不出现本机私有配置 multi-host.env 里的值（8 个字以上的词）与本机主机名" \
  "把命中的词换成「第二台」「本机」这类说法（红行列的是文件、行号与键名，值不打印，路径里含值的那一段打成「〔文件名含私有值〕」；文件名含值的改名；要引整句的，整句里的值也换掉；超大的未跟踪文件拆小）；值不要进仓、不要进日志原样输出，用到它的命令照 multi-host.env 的键名写"
stage_cell account-quota cell_account_quota "这次改动新加到 records/、.claude/kb/ 与三方判决正文的行里不写账号切换、会话或周限额、额度重置这类用量限额（显卡额度、内存上限、空间配额不在内）" \
  "删掉这句；子 agent 因此停下的，只写「中途停下没交回」，不写撞了哪种额度、换没换账号、什么时候重置；用户原话里带这些词、要原样留的，那一行末尾写「account-quota:allow <理由>」（理由至少 4 个字）"
stage_cell out-of-project-names cell_out_of_project_names "这次改动新加的行里不写不归本项目的目录名（本机家目录、仓的上一级目录与第二台私有名单里的条目名，只在像目录的上下文里判）与根下系统目录的占用；放行表 .claude/out-of-project-paths 每一条要有 git 跟踪的脚本引到" \
  "改成不指名的说法（例「根分区其余不归 singlefs」），要说空间只说本项目自己的目录与总的可用量；singlefs 自己读的仓外路径登记进放行表 .claude/out-of-project-paths（一行「<路径>  # 理由」，理由至少 8 个字，要有 git 跟踪的 .sh、.py、.rs 字面引到它），只放行那条路径本身"
stage_cell utf8-valid cell_utf8_valid "这次改动（暂存区或工作区相对基准）里的文本文件是合法 UTF-8：非法的点名文件、工作区还是暂存区、第一处坏字节的偏移" \
  "照红行的文件、版本（工作区或暂存区）、字节偏移和行号，把那一处按字符定点改回来（Edit 或 scripts/kb/replace-once.py；shell 的 paste -d、cut -c、head -c 这类按字节切的命令会把多字节字符切坏，改文本别用）；暂存区里坏而工作区已修好的，git add 重新暂存；确实是二进制的，补进 cell_utf8_valid 的后缀名单"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument="" out_of_project_all_lines=0
while (($#)); do
  case "$1" in
    --all-lines) out_of_project_all_lines=1; shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--all-lines（只配 --check out-of-project-names）与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
if (( out_of_project_all_lines )) && [[ "${STAGE_CELLS_SELECTED[*]}" != out-of-project-names ]]; then
  echo "  ✗ --all-lines 只给 out-of-project-names 一格用（这一趟选中的格：${STAGE_CELLS_SELECTED[*]}）"
  echo "     → 怎么办：写成 bash .claude/gate.d/doc-text.sh --check out-of-project-names --all-lines [项目根]；别的格不认它。"
  exit 2
fi
ROOT="${root_argument:-$REPO_ROOT}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级。"; exit 2; }

# scan_form <形态键>：按 lib-forbidden-notations.py 里那一形态扫全仓文本，退出码 0 没有违例、1 有违例（或读不到库）、77 一份文本文件都没扫到
scan_form() {
  private_value_sources
  python3 - "$LIBRARY" "$1" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" <<'PY'
import errno, importlib.util, os, subprocess, sys, types

library_path, form_key, private_values_source, configuration_path, hostname_path, broken = sys.argv[1:7]
try:
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', library_path)
    notations = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(notations)
except OSError as error:
    print(f'  ✗ 读不到禁用写法的登记 {library_path}：{error}')
    print('     → 怎么办：形态只在那一份里登记（.claude/hooks/write-guard.sh 读同一份），从 git 恢复它，别在阶段里再抄一份。')
    sys.exit(1)
if form_key not in notations.FORMS_BY_KEY:
    print(f'  ✗ 格名 {form_key} 在 lib-forbidden-notations.py 的 FORMS 里没有对应的形态')
    print('     → 怎么办：格名表与 FORMS 的键要逐个相同；在 FORMS 里加了或改了形态，就把阶段文件头的格名表与 stage_cell 登记一起改。')
    sys.exit(1)

listing = subprocess.run(['git', '-c', 'core.quotepath=false', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'],
                         capture_output=True)
if listing.returncode == 0:
    candidate_paths = sorted({name for name in listing.stdout.decode('utf-8', 'replace').split('\0') if name})
else:
    candidate_paths = sorted(os.path.relpath(os.path.join(directory, name), '.')
                             for directory, subdirectories, names in os.walk('.')
                             if not subdirectories.__setitem__(slice(None), [entry for entry in subdirectories if entry != '.git'])
                             for name in names)

# 红行与「没扫的」清单里打的路径都经 private-values 那一份打码（路径里含私有值的那一段打成〔文件名含私有值〕，片段里的值打成〔私有值〕）；
# 没有 multi-host.env（别人的克隆、第二台）或读不了就没有词可打，原样打
private_values = types.ModuleType("lib_private_values")
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if broken != "private-values-siblings-unmasked" and configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


form = notations.FORMS_BY_KEY[form_key]
findings = []
scanned_paths = []
excluded_count = 0
binary_files, irregular_files, unreadable_files = [], [], []   # unreadable_files：(路径, 错误)，读不了的（权限这一类）不让异常回溯把原路径打出来，路径经打码列进「没扫的」
for relative_path in candidate_paths:
    if notations.is_excluded(form_key, relative_path):
        excluded_count += 1
        continue
    if not os.path.isfile(relative_path):
        irregular_files.append(relative_path)
        continue
    try:
        with open(relative_path, 'rb') as handle:
            content = handle.read()
    except OSError as error:
        if broken == 'private-values-unreadable-raises':   # 弄坏：改前的样子，读不了就让格崩
            raise
        unreadable_files.append((relative_path, error))
        continue
    if b'\0' in content[:8192]:
        binary_files.append(relative_path)
        continue
    scanned_paths.append(relative_path)
    for line_number, line in enumerate(content.decode('utf-8', 'replace').split('\n'), 1):
        found = notations.first_hit(form_key, line, relative_path)
        if found:
            position, pattern_name, matched = found
            anchor = position + len(matched) if form['snippet_to_match_end'] else position
            start, end = max(0, position - 20), anchor + 10
            if broken != 'private-values-snippet-cut':   # 两头落在私有值中间的挪到值的边上，整值进片段再打码
                start, end = private_values.snippet_bounds(line, start, end, masking_targets)
            findings.append((relative_path, line_number, pattern_name, line[start:end].strip()))

if findings:
    finding_files = sorted({path for path, _, _, _ in findings})
    print(f'  ✗ {len(findings)} {form["finding"]}，在 {len(finding_files)} 份文件里：')   # gate-lint:summary
    for path, line_number, pattern_name, snippet in findings[:200]:
        shown_pattern = f'  {pattern_name}' if form['detail_shows_pattern'] else ''
        print(f'      {shown_path(path)}:{line_number}{shown_pattern}  …{private_values.shown_text(snippet, masking_targets)}…')   # gate-lint:detail
    if len(findings) > 200:
        print(f'      （另有 {len(findings) - 200} 行没列出，每份文件的行数见下）')   # gate-lint:detail
        for path in finding_files:
            print(f'      {shown_path(path)}：{sum(1 for hit_path, _, _, _ in findings if hit_path == path)} 行')   # gate-lint:detail
    print('     → 怎么办：' + form['remedy'][0])
    for remedy_line in form['remedy'][1:]:
        print('               ' + remedy_line)
    sys.exit(1)
if not scanned_paths:
    print(f'  ! {form["name"]}这一形态本次无对象可判：没扫到一份文本文件（列出 {len(candidate_paths)} 个路径，'
          f'二进制 {len(binary_files)}、不是普通文件 {len(irregular_files)}、读不了 {len(unreadable_files)}、排除前缀下 {excluded_count}）')
    sys.exit(77)
print(f'  ✓ 查了 {len(scanned_paths)} 份文本文件，{form["clean"]}')
print('     没扫的：' + form['excluded_label'].format(count=excluded_count))
print(f'     二进制 {len(binary_files)} 份、列出了却不是普通文件 {len(irregular_files)} 份、读不了 {len(unreadable_files)} 份，这一形态没扫，逐个列在下面')
for path in binary_files:
    print(f'       二进制：{shown_path(path)}')
for path in irregular_files:
    print(f'       不是普通文件（工作区里删掉还没暂存的）：{shown_path(path)}')
for path, error in unreadable_files:
    print(f'       读不了（{errno.errorcode.get(error.errno, error.errno)}，补上读权限再跑才扫得到）：{shown_path(path)}')
PY
}

cell_prime_marks() {
  scan_form prime-marks
}

cell_clock_times() {
  scan_form clock-times
}

cell_session_names() {
  scan_form session-names
}

cell_term_renames() {
  if [[ "$text_notations_break" == term-renames-unjudged ]]; then
    echo "  ! 弄坏开关 term-renames-unjudged：这一格不调 sweep-term，记本次无对象可判"
    exit 77
  fi
  if [[ ! -f "$ROOT/.claude/kb/term-renames.md" ]]; then
    echo "  ! 没有 .claude/kb/term-renames.md，这一格本次无对象可判"
    exit 77
  fi
  local sweep_output sweep_status
  private_value_sources
  # sweep-term 逐份打的路径经 private-values 那一份打码再打（shown_lines_main 的 lines：带 / 的一段当路径，别的当行文）
  if sweep_output="$(python3 "$SWEEP_TERM" --check "$ROOT" 2>&1)"; then sweep_status=0; else sweep_status=$?; fi
  if [[ -n "$sweep_output" ]]; then
    printf '%s\n' "$sweep_output" | python3 -c 'import sys, types; library = types.ModuleType("lib_private_values"); exec(compile(sys.argv[1], "private-values-library", "exec"), library.__dict__); library.shown_lines_main(sys.argv[2:], sys.stdin, sys.stdout)' \
      "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" lines
  fi
  if ((sweep_status == 0)); then
    exit 0
  fi
  echo "  ✗ 登记过的术语改名，仓里还有地方用着旧名（上面逐份列出份数与处数），或排除表指向不存在的路径"
  echo "     → 怎么办：python3 scripts/kb/sweep-term.py --apply 一次换完。"
  echo "               产物分两类，别一把梭：**跑得出来的**（装置源码在树里、replay.sh 判 exact）跟着源码一起换，"
  echo "               换完跑 cargo test --workspace 与 bash scripts/experiment-apparatus/replay.sh，同一份源码要仍然吐出逐字节相同的产物——"
  echo "               那是重新生成，不是改证据；**跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，"
  echo "               连同引它的正文整段留旧名（evidence-discipline「原样保存的证据不许事后改」）。"
  echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件里换了就成假话的那一句、引文块、对照表自己），"
  echo "               登记进 .claude/term-rename-exempt 并写明为什么；历史类文件里只有一句该留的写 <路径>:「<那一行里的一段原文>」只豁免那一行，"
  echo "               整份都该留的逐文件登记，不整个目录豁免；指向不存在路径、行内锚命中不是恰好一行的那一条改掉或删掉。"
  exit 1
}

# date-once 红行里的路径经 private-values 那一份打码（shown_lines_main 的 path）
date_once_shown_path() {
  printf '%s\n' "$1" | python3 -c 'import sys, types; library = types.ModuleType("lib_private_values"); exec(compile(sys.argv[1], "private-values-library", "exec"), library.__dict__); library.shown_lines_main(sys.argv[2:], sys.stdin, sys.stdout)' \
    "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" path
}

cell_date_once() {
  local base changed path count=0 red=0 unjudged=0 rc reason scratch summary dates merges shown excluded
  if [[ "$text_notations_break" == date-once-unjudged ]]; then
    echo "  ! 弄坏开关 date-once-unjudged：这一格不干跑，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    base="$(gate_diff_base gate)"
    changed="$(gate_changed_paths "$base" untracked)" || { echo "  ✗ 取不到这次改动碰了哪些路径（基准 ${base:-空}）"; echo "     → 怎么办：在 git 仓里跑；基准与路径的取法只在 scripts/gate/changed-paths.sh 里有一份，它报了什么先照它的输出处理。"; exit 1; }
  else
    changed="$(find records .claude/kb -name '*.md' 2>/dev/null)"
  fi
  # 不判的：两份变更史与 INDEX.md 连同它们的英语、日语译文。译文路径照 lib-trilingual.py 的 language_paths 取（<名>.en.md、<名>.ja.md，
  # .claude/kb/translation-paths.json 给译文另起了名的照它），不另抄一份认法；弄坏开关 date-once-history-translations-judged 只排除中文那三份。
  if ! excluded="$(python3 -B - "$TRILINGUAL" "$text_notations_break" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_trilingual", sys.argv[1])
trilingual = importlib.util.module_from_spec(spec)
spec.loader.exec_module(trilingual)
for name in ("decisions-history", "experiments-history", "INDEX"):
    paths = trilingual.language_paths(".claude/kb", name)
    for language, _ in trilingual.LANGUAGES:
        if language == "zh" or sys.argv[2] != "date-once-history-translations-judged":
            print(paths[language])
PY
)"; then
    echo "  ✗ 取不到不判的那几份（两份变更史与 INDEX.md 连同英日译文）的路径：同目录 lib-trilingual.py 装不进来或 language_paths 出错（原因在上面几行）"
    echo "     → 怎么办：照上面的报错修 lib-trilingual.py 或 .claude/kb/translation-paths.json，再跑这一格；不退回只排除中文那三份。"
    exit 1
  fi
  scratch="$(mktemp "${TMPDIR:-/tmp}/date-once.XXXXXX")"
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    case "$path" in
      records/*.md) ;;
      .claude/kb/*.md) ;;
      *) continue ;;
    esac
    [[ $'\n'"$excluded"$'\n' == *$'\n'"$path"$'\n'* ]] && continue
    [[ -f "$path" ]] || continue
    count=$((count + 1))
    # 干跑的退出码要收：0 与 3（丢失核对找不到东西）才有那一行摘要；别的（5 认不出历史节、1 读不了那一份这类）是没判，不当干跑 0 处
    rc=0
    DOC_CONSOLIDATE_LEDGER=off python3 "$DOC_CONSOLIDATE" migrate "$path" --out "$scratch" --force > "$scratch.out" 2>&1 || rc=$?
    summary="$(head -1 "$scratch.out")"
    if ((rc != 0 && rc != 3)) && [[ "$text_notations_break" != date-once-exit-ignored ]]; then
      unjudged=$((unjudged + 1))
      shown="$(date_once_shown_path "$path")"
      reason="$(grep -m1 -E '✗|Error' "$scratch.out" || true)"
      echo "  ✗ ${shown:-〔路径打码没打出来〕}：doc-consolidate.py 干跑退 $rc，这一份没判（${reason:0:200}）"   # gate-lint:detail
      continue
    fi
    dates="$(sed -n 's/.*同日日期删 \([0-9]*\).*/\1/p' <<<"$summary")"
    merges="$(sed -n 's/.*同日小节合并 \([0-9]*\).*/\1/p' <<<"$summary")"
    if [[ "${dates:-0}" != 0 || "${merges:-0}" != 0 ]]; then
      red=$((red + 1))
      shown="$(date_once_shown_path "$path")"
      echo "  ✗ ${shown:-〔路径打码没打出来〕}：正文里重复所属日期 ${dates:-0} 处、同一天的第二个小节 ${merges:-0} 个"   # gate-lint:detail
    fi
  done <<<"$changed"
  rm -f "$scratch" "$scratch.out"
  if ((unjudged)); then
    echo "  ✗ 这次改动碰到的 $count 份记录与 kb 页里 $unjudged 份干跑出错、没判（scripts/kb/doc-consolidate.py 退的不是 0 也不是 3）"   # gate-lint:summary
    echo "     → 怎么办：照上面每一份括号里的报错修：退 5 是认不出历史节（翻译术语表读不到，或没有中文列是「历史版本」的那一行，补那一行）；"
    echo "               别的退出码照 doc-consolidate.py 的报错修那一份文件或那份脚本，再跑这一格。出错的那几份不算判过。"
    ((red)) || exit 1
  fi
  if ((red)); then
    echo "  ✗ 这次改动碰到的 $count 份记录与 kb 页里 $red 份日期没有只写在标题上"   # gate-lint:summary
    echo "     → 怎么办：python3 scripts/kb/doc-consolidate.py migrate <那一份> --out <临时文件>，看它删了哪些日期、并了哪些小节，照着改回仓里；"
    echo "               规则：.claude/rules/changelog-format.md「只有决策与实验有历史」与 kb-discipline 第 1 条。"
    exit 1
  fi
  if ((count == 0)); then
    echo "  ! 这次改动没碰到 records/ 与 .claude/kb/ 下的 .md，这一格无对象可判"
    exit 77
  fi
  echo "  ✓ 日期只在标题上：这次改动碰到的 $count 份记录与 kb 页干跑一份都没改"
}

# 三语六格的判法都在 lib-trilingual.py；弄坏开关 trilingual-<格名>-unjudged 让这一格不判、记无对象可判（退 77）
trilingual_unjudged() {
  if [[ "$text_notations_break" == "trilingual-$1-unjudged" ]]; then
    echo "  ! 弄坏开关 trilingual-$1-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
}

cell_trilingual_files() {
  trilingual_unjudged files
  python3 "$TRILINGUAL" files "$ROOT"
  exit $?
}

cell_trilingual_structure() {
  trilingual_unjudged structure
  python3 "$TRILINGUAL" structure "$ROOT"
  exit $?
}

cell_trilingual_changed_together() {
  local base changed
  trilingual_unjudged changed-together
  cd "$ROOT" || exit 1
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 不在 git 仓里，拿不到这次改动碰了哪些路径，这一格无对象可判"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  if ! changed="$(gate_changed_paths "$base" untracked)"; then
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 ${base:-空}）"   # gate-lint:summary
    echo "     → 怎么办：在 git 仓里跑；基准与路径的取法只在 scripts/gate/changed-paths.sh 里有一份，它报了什么先照它的输出处理。"
    exit 1
  fi
  python3 "$TRILINGUAL" changed-together "$ROOT" <<<"$changed"
  exit $?
}

cell_trilingual_glossary() {
  trilingual_unjudged glossary
  python3 "$TRILINGUAL" glossary "$ROOT"
  exit $?
}

# 判法在 lib-trilingual.py 的 untranslated；这一道的开关 trilingual-untranslated-<项>（-unjudged 之外）转成 LIB_TRILINGUAL_BREAK 同名一项交给它，
# 开关只在样本根上认（stage_cells_run 开头那一判已经拦过真仓）。
cell_trilingual_untranslated() {
  local library_break="${LIB_TRILINGUAL_BREAK:-}"
  trilingual_unjudged untranslated
  case "$text_notations_break" in
    trilingual-untranslated-quotes-kept|trilingual-untranslated-code-kept|trilingual-untranslated-japanese-unjudged|trilingual-untranslated-exempt-ignored|trilingual-untranslated-japanese-threshold-five|trilingual-untranslated-identical-ignored)
      library_break="$text_notations_break" ;;
  esac
  LIB_TRILINGUAL_BREAK="$library_break" python3 "$TRILINGUAL" untranslated "$ROOT"
  exit $?
}

cell_trilingual_omission() {
  trilingual_unjudged omission
  python3 "$TRILINGUAL" omission "$ROOT"
  exit $?
}

cell_private_values() {
  local base changed configuration_file changed_file hostname_file size_limit_file python_rc=0
  if [[ "$text_notations_break" == private-values-unjudged ]]; then
    echo "  ! 弄坏开关 private-values-unjudged：这一格不比对，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  configuration_file="$PRIVATE_CONFIGURATION_FILE" hostname_file="$PRIVATE_HOSTNAME_FILE" size_limit_file="$PRIVATE_SIZE_LIMIT_FILE"
  if [[ ! -f "$configuration_file" ]]; then
    echo "  ! 这台机器没有 multi-host.env（别人的克隆、第二台，或还没配多机），没有私有值可比，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动新加的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(gate_changed_paths "$base" untracked)" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 $base）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/private-values.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python3 - "$base" "$configuration_file" "$hostname_file" "$changed_file" "$text_notations_break" "$size_limit_file" "$PRIVATE_VALUES_LIBRARY" <<'PY' || python_rc=$?
import errno, io, os, re, subprocess, sys, types

base, configuration_path, hostname_path, changed_path, broken, size_limit_path, private_values_source = sys.argv[1:8]
private_values = types.ModuleType("lib_private_values")      # 取词与路径打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY；prime-marks / clock-times / session-names 三格打路径时用同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
HOSTNAME_KEY = private_values.HOSTNAME_KEY
FILE_NAME_MASK = private_values.FILE_NAME_MASK
HEAD_BYTES = 8192                           # 前 8 KiB 里有 NUL 的文件按二进制整份跳过
TRUNCATED_BYTES = 8 * 1024 * 1024           # 只有弄坏开关 private-values-big-file-truncated 才用：改前的样子，未跟踪文件只读前 8 MiB
HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


UNTRACKED_SIZE_LIMIT = 1024 * 1024 * 1024      # 未跟踪文件整份逐行比，超过这个大小判红、不截断；样本根里的 .private-values-size-limit（字节数）顶替它，样本不必造出一个 GiB 级的文件
if size_limit_path:
    try:
        UNTRACKED_SIZE_LIMIT = int(open(size_limit_path, encoding="utf-8").read().strip())
    except (OSError, ValueError) as error:
        fail(f"读不了样本的大小上限 {size_limit_path}：{error}", "样本根里的 .private-values-size-limit 只写一个字节数。")


try:
    targets, short_words, configuration_word_count, hostname_note, MINIMUM_LENGTH = private_values.load_private_targets(configuration_path, hostname_path, broken)
except OSError as error:
    fail(f"读不了 {configuration_path}：{error}", "看它的权限；这一格要读它里面每个键的值（只比对，不打印）。")
if not targets:
    print(f"  ! multi-host.env 里没有 {MINIMUM_LENGTH} 个字以上的词，本机主机名也不取，没有私有值可比，这一格本次无对象可判（这不是通过）")
    sys.exit(77)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


untracked_listing = git("ls-files", "-z", "--others", "--exclude-standard")
if untracked_listing.returncode != 0:
    fail("列不出未跟踪的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
untracked = {name for name in untracked_listing.stdout.decode("utf-8", "replace").split("\0") if name}
configuration_real = os.path.realpath(configuration_path)
changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]

def shown_path(path):
    """红行里的路径（私有值按整串在整条路径里找、命中的那一段打成 FILE_NAME_MASK，值不打印）：判法在共用函数 private_values.shown_path。"""
    return private_values.shown_path(path, targets, broken)


def added_lines(path, tracked_diff):
    """这份文件新加的 (行号, 行文)：已跟踪的取 git diff -U0 的新加行；未跟踪的整份逐行读（不整份进内存）。"""
    if tracked_diff:
        difference = git("diff", "-U0", "--no-color", "--no-ext-diff", "--no-renames", base, "--", path)
        if difference.returncode != 0:
            fail(f"取不到 {shown_path(path)} 相对基准 {base[:12]} 的改动（git diff 退 {difference.returncode}）", "看基准提交在不在、git 是不是可用；改动范围的基准只经 scripts/gate/changed-paths.sh 的 gate_diff_base 取。")
        number, in_hunk = 0, False
        for line in difference.stdout.decode("utf-8", "replace").split("\n"):
            found = HUNK.match(line)
            if found:
                number, in_hunk = int(found.group(1)), True
            elif in_hunk and line.startswith("+"):
                yield number, line[1:]
                number += 1
        return
    with open(path, "rb") as handle:
        source = io.BytesIO(handle.read(TRUNCATED_BYTES)) if broken == "private-values-big-file-truncated" else handle
        for number, raw in enumerate(source, 1):
            yield number, raw.decode("utf-8", "replace").rstrip("\n")


findings = []             # (路径, 行号, 键名列表, 词)
name_findings = []        # (路径, 键名列表)：文件名本身含私有值
too_big = []              # (路径, 字节数)：未跟踪、超过 UNTRACKED_SIZE_LIMIT，没法整份比
link_findings = []        # (路径, 键名列表, 词)：符号链接的链接文字含私有值（git 进仓的是链接文字本身）
unreadable = []           # (路径, 错误)：读不了的文件（权限这一类），没法比对；路径经打码，不让异常回溯把原路径打出来
files_judged, lines_judged, links_judged, binary_files = 0, 0, 0, []
for path in changed_paths:
    if os.path.islink(path) and broken != "private-values-links-followed":
        # 符号链接比路径与链接文字，不跟进去：坏链接也比，好链接不读它指到的内容
        name_keys = sorted({key for key, word in targets if word in path}) if broken != "private-values-names-unjudged" else []
        if name_keys:
            name_findings.append((path, name_keys))
        try:
            link_text = os.readlink(path)
        except OSError as error:
            unreadable.append((path, error))
            continue
        links_judged += 1
        keys = sorted({key for key, word in targets if word in link_text})
        if keys:
            link_findings.append((path, keys, [word for key, word in targets if word in link_text]))
        continue
    if not os.path.isfile(path) or os.path.realpath(path) == configuration_real:
        continue
    name_keys = sorted({key for key, word in targets if word in path}) if broken != "private-values-names-unjudged" else []
    if name_keys:
        name_findings.append((path, name_keys))
    try:
        with open(path, "rb") as handle:
            head = handle.read(HEAD_BYTES)
        if b"\0" in head:
            binary_files.append(path)
            continue
        whole_file = path in untracked or broken == "private-values-whole-tree"
        size = os.path.getsize(path)
        if whole_file and size > UNTRACKED_SIZE_LIMIT and broken != "private-values-big-file-truncated":
            too_big.append((path, size))
            continue
        files_judged += 1
        for number, text in added_lines(path, tracked_diff=not whole_file):
            lines_judged += 1
            keys = sorted({key for key, word in targets if word in text})
            if keys:
                findings.append((path, number, keys, [word for key, word in targets if word in text]))
    except OSError as error:
        if broken == "private-values-unreadable-raises":   # 弄坏：改前的样子，读不了就让格崩，回溯里带原路径
            raise
        unreadable.append((path, error))

if findings:
    print(f"  ✗ 这次改动新加的 {len(findings)} 行里出现了本机私有值（multi-host.env 里 {MINIMUM_LENGTH} 个字以上的词，或本机主机名）；只列文件、行号与键名，不打印值：")   # gate-lint:summary
    for path, number, keys, words in findings:
        shown = f"（值：{'、'.join(words)}）" if broken == "private-values-print-values" else "（值不打印）"
        print(f"      {shown_path(path)}:{number}  键名：{'、'.join(keys)}{shown}")   # gate-lint:detail
    print("     → 怎么办：把命中的词换成「第二台」「本机」这类说法（要引整句的，整句里的值也换掉）；值只留在 multi-host.env，别写进记录、日志摘录、提示和代码。")
    print("               红行没打印值，对着行号和键名去 multi-host.env 找；路径里哪一段含私有值，那一段打成" + FILE_NAME_MASK + "；commit message 这一格看不到，提交说明里同样别写。")
if name_findings:
    print(f"  ✗ 这次改动里 {len(name_findings)} 个文件的名字含本机私有值（路径里含值的那一段打成{FILE_NAME_MASK}，值不打印）：")   # gate-lint:summary
    for path, keys in name_findings:
        print(f"      {shown_path(path)}  键名：{'、'.join(keys)}（值不打印）")   # gate-lint:detail
    print("     → 怎么办：文件名里的值换成「第二台」「本机」这类说法再改名（用 git status 看这次新加、改动的文件名，对着目录和键名找）；产物的文件名由脚本拼出来的，改脚本别拿私有值拼名字。")
if too_big:
    print(f"  ✗ 这次改动里 {len(too_big)} 个未跟踪的文件超过 {UNTRACKED_SIZE_LIMIT} 字节，这一格没法整份比对（截断会漏掉后面的行，不截断）：")   # gate-lint:summary
    for path, size in too_big:
        print(f"      {shown_path(path)}  {size} 字节")   # gate-lint:detail
    print(f"     → 怎么办：把它拆成每份不超过 {UNTRACKED_SIZE_LIMIT} 字节的几份，或去掉进度行这类重复内容后重新落盘；值不要进仓、不要进日志原样输出。")
if link_findings:
    print(f"  ✗ 这次改动里 {len(link_findings)} 个符号链接的链接文字含本机私有值（git 进仓的是链接文字本身，不是它指到的内容；值不打印）：")   # gate-lint:summary
    for path, keys, words in link_findings:
        shown = f"（值：{'、'.join(words)}）" if broken == "private-values-print-values" else "（值不打印）"
        print(f"      {shown_path(path)}  链接目标  键名：{'、'.join(keys)}{shown}")   # gate-lint:detail
    print("     → 怎么办：别把指到第二台路径、挂载点的链接放进仓里：要留的产物拷一份进来（内容里的值照样换成「第二台」「本机」这类说法），只是图找起来方便的链接删掉。")
if unreadable:
    print(f"  ✗ 这次改动里 {len(unreadable)} 个文件读不了（权限这一类），这一格没法比对（路径里含私有值的那一段打成{FILE_NAME_MASK}，值不打印）：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；从第二台 tar 回来的权限位不对的，解包时加 --no-same-permissions；确实不该留的删掉。")
if findings or name_findings or too_big or link_findings or unreadable:
    sys.exit(1)
if lines_judged == 0 and links_judged == 0:
    print(f"  ! 这次改动没有新加的行（改动范围 {len(changed_paths)} 个路径，二进制 {len(binary_files)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
short_total = sum(short_words.values())
short_keys = "、".join(sorted(short_words)) or "无"
print(f"  ✓ 查了这次改动里 {files_judged} 份文本文件新加的 {lines_judged} 行：对着 multi-host.env 的 {configuration_word_count} 个长词（{MINIMUM_LENGTH} 个字以上）与{hostname_note}，没有一处命中")
if links_judged:
    print(f"     符号链接 {links_judged} 个：比了路径与链接文字（不跟进去），没有一处命中")
print(f"     没判的：不足 {MINIMUM_LENGTH} 个字的词 {short_total} 个（键 {short_keys}）、二进制 {len(binary_files)} 份")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}

cell_account_quota() {
  local base changed changed_file python_rc=0
  if [[ "$text_notations_break" == account-quota-unjudged ]]; then
    echo "  ! 弄坏开关 account-quota-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动新加的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(gate_changed_paths "$base" untracked)" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 ${base:-空}）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/account-quota.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python3 - "$base" "$changed_file" "$text_notations_break" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" <<'PY' || python_rc=$?
import errno, os, re, subprocess, sys, types

base, changed_path, broken, private_values_source, configuration_path, hostname_path = sys.argv[1:7]
private_values = types.ModuleType("lib_private_values")      # 路径与片段打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY，与别的格同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []
HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")
HEAD_BYTES = 8192

# ── 词表（唯一一处）：逐条写明拦什么 ──
# 两层。
#   明说层（STRICT）：句子里写明了是哪个账号、哪一种限额（会话、周、Opus、用量、账号……），或是限额通知的原文，命中就红，不看同一行里还有什么词。
#   裸词层（BARE）：「撞限额」「额度恢复」这类没写是哪一种额度的说法；命中处前后 24 个字内有算力资源的词（RESOURCE_WORDS：显卡、显存、内存、线程、账上、磁盘、空间、quota……）的放过，
#     那是显卡额度、内存额度、空间配额这类正常用法。只有「额度」「限额」两个词单独出现、没有上面这些动词与限定词的，不在词表里，不判。
# 键是红行里点名的判据名；改词表只改这一处。
SCOPE = r"(?:会话|周|每周|五小时|5\s?小时|Opus|Sonnet|Haiku|Claude|API|账号|帐号|订阅)"
ACCOUNT = r"(?<!欠)[账帐]号"       # 「欠账号」「欠账号 C820」是欠账编号，不是账号
STRICT = [
    ("账号切换·动词", "拦「切换账号」「换了账号」「换成别的账号」这类动作：切换、切到、换到、换成、轮换、更换、换（了）后六个字以内跟着账号。不拦「欠账号」",
     r"(?:切换|切到|换到|换成|轮换|更换|换)(?:了)?[^，。；！？\n「」]{0,6}?" + ACCOUNT),
    ("账号切换·名词", "拦「账号切换」「账号额度」「另一个账号」「备用账号」「别的账号」这类名词写法",
     ACCOUNT + r"(?:切换|轮换|额度|限额|用量|被限)|(?:另一个|别的|备用|第二个|不同的?)" + ACCOUNT),
    ("账号切换·英文", "拦 switched the account、account switch",
     r"(?i:switch(?:ed|ing)?\s+(?:the\s+)?account|account\s+switch)"),
    ("限额·写明哪种", "拦「会话限额」「周限额」「每周额度」「Opus 周额度」「账号额度」「订阅限额」：会话、周、每周、五小时、Opus、Sonnet、Haiku、Claude、API、账号、订阅后面直接接限额或额度（中间可隔空白与「用量」）",
     SCOPE + r"\s*(?:用量)?\s*(?:限额|额度)"),
    ("限额·窗口", "拦「限额窗口」「额度窗口」（含派发闸的放行行「限额窗口已判」被抄进记录）",
     r"(?:限额|额度)窗口"),
    ("限额·会话周上限", "拦「会话上限」「周上限」",
     r"(?:会话|周)上限"),
    ("限额·撞会话周限流", "拦「撞会话限流」「撞了周限流」：撞或碰、会话或周、限流",
     r"(?:撞|碰)(?:到|上|了|过)*\s*" + SCOPE + r"\s*限流"),
    ("限额·通知原文", "拦限额通知的原文：You've hit your … limit、usage limit、session limit、weekly limit",
     r"(?i:you(?:'|’)ve hit your|usage limit|session limit|weekly limit)"),
]
BARE = [
    ("限额·撞", "拦「撞限额」「撞了额度」「碰到限流」「限额撞顶」「额度见底」：撞或碰（后面可接到、上、了、过）加用量、使用（可无）加限额、额度、限流；限额或额度后面接撞顶、到顶、见底、耗尽、用尽、耗光",
     r"(?:撞|碰)(?:到|上|了|过)*(?:用量|使用)?(?:限额|额度|限流)|(?:限额|额度)(?:撞顶|到顶|见底|耗尽|用尽|耗光)"),
    ("限额·重置恢复", "拦「额度重置」「限额恢复」「额度刷新」「恢复了额度」「重置限额」：限额或额度后面接重置、刷新、恢复、解除、复位，或前面接重置、刷新、恢复（可带了）",
     r"(?:限额|额度)(?:重置|刷新|恢复|解除|复位)|(?:重置|刷新|恢复)(?:了)?(?:限额|额度)"),
    ("限额·打断", "拦「被额度打断」「限额中断」「限额续做」「限额暂停」：被（用量）额度或限额后面接打断、卡住、停，或额度、限额后面接打断、暂停、中断、续做",
     r"被(?:用量)?(?:额度|限额)(?:打断|卡住|停)|(?:额度|限额)(?:打断|暂停|中断|续做)"),
    ("限额·用完", "拦「额度用完」「限额用光」「用完了额度」「耗尽限额」",
     r"(?:额度|限额)(?:用完|用光)|(?:用完|用光|耗尽)了?(?:限额|额度)"),
    ("用量限额·额度", "拦「用量限额」「用量额度」「使用限额」「使用额度」「用量上限」",
     r"(?:用量|使用)(?:限额|额度)|用量上限"),
]
# 命中处前后 RESOURCE_WINDOW 个字内有这些词，裸词层的命中放过（显卡额度、内存额度、空间配额、账上多少 G 这类算力与空间资源的正常用法）；
# 只看命中处附近、不看整行：一行里别处提到线程上限、草稿目录多少 G，不能把同一行里的模型限额放过去（对着仓里量过，整行放过的三行里有两行其实是模型限额）
RESOURCE_WINDOW = 24
RESOURCE_WORDS = re.compile(r"显卡|GPU|显存|VRAM|内存|memory|CPU|线程|账上|磁盘|空间|inode|文件系统|存储|quota|\d\s?(?:G|GiB|GB|MiB|MB)\b")
# 放行标记：行里写 account-quota:allow <理由>（理由至少 4 个字），整行放过；没写理由的不放过，另报。文件头「account-quota」一段写了什么时候用。
ALLOW_MARKER = re.compile(r"account-quota:allow(?P<reason>.*)")
COMPILED_STRICT = [(key, what, re.compile(pattern)) for key, what, pattern in STRICT]
COMPILED_BARE = [(key, what, re.compile(pattern)) for key, what, pattern in BARE]
if broken == "account-quota-strict-only":                    # 弄坏：裸词层不判
    COMPILED_BARE = []
if broken == "account-quota-resource-unexempt":              # 弄坏：裸词层不看资源词，显卡额度、内存额度这类也红
    RESOURCE_WORDS = re.compile(r"(?!)")


def in_scope(path):
    """射程：records/ 与 .claude/kb/ 下的 .md，research/prompts/ 顶层的 *-main-verification.md（三方判决正文）。
    prompts 里别的文件（_*-appendix.md、_*-background.md、_*-body.md、*-snapshot/ 下的副本）是从仓里抄来给云端腿读的，抄进来的旧话不算这一次写的，不判。"""
    if broken == "account-quota-records-only":                # 弄坏：只判 records/
        return path.startswith("records/") and path.endswith(".md")
    if not path.endswith(".md"):
        return False
    if path.startswith("records/") or path.startswith(".claude/kb/"):
        return True
    return path.startswith("research/prompts/") and "/" not in path[len("research/prompts/"):] and path.endswith("-main-verification.md")


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


untracked_listing = git("ls-files", "-z", "--others", "--exclude-standard")
if untracked_listing.returncode != 0:
    fail("列不出未跟踪的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
untracked = {name for name in untracked_listing.stdout.decode("utf-8", "replace").split("\0") if name}
changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]


def added_lines(path, whole_file):
    """这份文件新加的 (行号, 行文)：已跟踪的取 git diff -U0 的新加行；未跟踪的整份逐行读（新文件整份都算新加）。"""
    if not whole_file:
        difference = git("diff", "-U0", "--no-color", "--no-ext-diff", "--no-renames", base, "--", path)
        if difference.returncode != 0:
            fail(f"取不到 {shown_path(path)} 相对基准 {base[:12]} 的改动（git diff 退 {difference.returncode}）", "看基准提交在不在、git 是不是可用；改动范围的基准只经 scripts/gate/changed-paths.sh 的 gate_diff_base 取。")
        number, in_hunk = 0, False
        for line in difference.stdout.decode("utf-8", "replace").split("\n"):
            found = HUNK.match(line)
            if found:
                number, in_hunk = int(found.group(1)), True
            elif in_hunk and line.startswith("+"):
                yield number, line[1:]
                number += 1
        return
    with open(path, "rb") as handle:
        for number, raw in enumerate(handle, 1):
            yield number, raw.decode("utf-8", "replace").rstrip("\n")


def judge(line):
    """一行的判法：返回 (放行状况, 命中)。放行状况 allowed（带理由的标记）/ reasonless（标记没写理由）/ None；命中 (判据名, 命中位置, 命中文字) 或 None；
    资源词放过的裸词命中记成 exempt。"""
    if broken == "account-quota-words-skipped":                # 弄坏：整格不判（词表与放行标记都不看）
        return None, None
    marker = ALLOW_MARKER.search(line) if broken != "account-quota-marker-ignored" else None
    marker_state = None
    if marker:
        reason = re.sub(r"-->|^[\s:：，,]+|\s+$", "", marker.group("reason"))
        marker_state = "allowed" if (len(reason) >= 4 or broken == "account-quota-reasonless-allowed") else "reasonless"
    for key, _what, pattern in COMPILED_STRICT:
        found = pattern.search(line)
        if found:
            return marker_state, ("hit", key, found.start(), found.group(0))
    for key, _what, pattern in COMPILED_BARE:
        found = pattern.search(line)
        if found:
            if RESOURCE_WORDS.search(line[max(0, found.start() - RESOURCE_WINDOW): found.end() + RESOURCE_WINDOW]):
                return marker_state, ("exempt", key, found.start(), found.group(0))
            return marker_state, ("hit", key, found.start(), found.group(0))
    return marker_state, None


findings, reasonless, unreadable = [], [], []      # (路径, 行号, 判据名, 片段) / (路径, 行号, 片段) / (路径, 错误)
files_judged, lines_judged, allowed_lines, exempt_lines, binary_files = 0, 0, 0, 0, []
for path in changed_paths:
    if not in_scope(path) or os.path.islink(path) or not os.path.isfile(path):
        continue
    try:
        with open(path, "rb") as handle:
            head = handle.read(HEAD_BYTES)
        if b"\0" in head:
            binary_files.append(path)
            continue
        files_judged += 1
        whole_file = path in untracked or broken == "account-quota-whole-tree"
        for number, text in added_lines(path, whole_file):
            lines_judged += 1
            marker_state, hit = judge(text)
            if marker_state == "reasonless":
                reasonless.append((path, number, text.strip()[:60]))
                continue
            if hit is None:
                continue
            kind, key, position, matched = hit
            if kind == "exempt":
                exempt_lines += 1
                continue
            if marker_state == "allowed":
                allowed_lines += 1
                continue
            start, end = max(0, position - 15), position + len(matched) + 15
            start, end = private_values.snippet_bounds(text, start, end, masking_targets)
            findings.append((path, number, key, text[start:end].strip()))
    except OSError as error:
        unreadable.append((path, error))

if findings:
    print(f"  ✗ 这次改动新加的 {len(findings)} 行写了账号切换或用量限额，在 {len({path for path, _, _, _ in findings})} 份文件里（只判这次新加的行；旧行不报）：")   # gate-lint:summary
    for path, number, key, snippet in findings[:200]:
        print(f"      {shown_path(path)}:{number}  {key}  …{private_values.shown_text(snippet, masking_targets)}…")   # gate-lint:detail
    if len(findings) > 200:
        print(f"      （另有 {len(findings) - 200} 行没列出）")   # gate-lint:detail
    print("     → 怎么办：删掉这句。子 agent 因此停下的，只写「中途停下没交回」，不写是撞了哪种额度、换没换账号、什么时候重置；")
    print("               用户原话里带这些词、要原样留的，那一行末尾写「account-quota:allow <理由>」（理由至少 4 个字）；显卡额度、内存上限、空间配额这类不是这一条，命中处前后 24 个字内有显卡、内存、线程这类资源词的裸词命中本来就放过。")
if reasonless:
    print(f"  ✗ 这次改动新加的 {len(reasonless)} 行写了 account-quota:allow 却没写理由（理由至少 4 个字，写不出理由的就是该删的）：")   # gate-lint:summary
    for path, number, snippet in reasonless:
        print(f"      {shown_path(path)}:{number}  …{private_values.shown_text(snippet, masking_targets)}…")   # gate-lint:detail
    print("     → 怎么办：在 account-quota:allow 后面写为什么这一行要留（例如「用户原话」）；写不出来就删掉这一行里的限额写法。")
if unreadable:
    print(f"  ✗ 这次改动里 {len(unreadable)} 个文件读不了（权限这一类），这一格没法判：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；确实不该留的删掉。")
if findings or reasonless or unreadable:
    sys.exit(1)
if lines_judged == 0:
    print(f"  ! 这次改动没有新加到 records/、.claude/kb/ 与 research/prompts/*-main-verification.md 的行（改动范围 {len(changed_paths)} 个路径，二进制 {len(binary_files)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
print(f"  ✓ 查了这次改动里 {files_judged} 份记录、kb 页与判决正文新加的 {lines_judged} 行：没有账号切换、用量限额的写法")
print(f"     放过的：带理由的放行标记 {allowed_lines} 行、命中处附近有显卡内存这类资源词的裸词命中 {exempt_lines} 行；没判的：二进制 {len(binary_files)} 份")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}

cell_out_of_project_names() {
  local base="" changed changed_file common_directory main_root python_rc=0
  if [[ "$text_notations_break" == out-of-project-names-unjudged ]]; then
    echo "  ! 弄坏开关 out-of-project-names-unjudged：这一格不判，记本次无对象可判"
    exit 77
  fi
  cd "$ROOT" || exit 1
  private_value_sources
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动新加的行」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  # 仓自己与它的上一级目录：取 git common-dir 的上一层（主工作树的根，--staged 的临时 worktree 里也是主工作树），同 private_value_sources
  main_root="$ROOT"
  if common_directory="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    main_root="$(dirname "$common_directory")"
  fi
  changed_file="$(mktemp "${TMPDIR:-/tmp}/out-of-project-names.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  if (( ! out_of_project_all_lines )); then
    base="$(gate_diff_base gate)"
    changed="$(gate_changed_paths "$base" untracked)" || { rm -f "$changed_file"; echo "  ✗ 取不到这一轮碰了哪些文件（基准 ${base:-空}）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
    printf '%s\n' "$changed" > "$changed_file"
  fi
  python3 - "$base" "$changed_file" "$text_notations_break" "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" \
    "$PRIVATE_IN_SAMPLE" "$out_of_project_all_lines" "${GATE_NOT_RUN_FILE:-}" "${GATE_OUT_OF_PROJECT_PARENT:-}" "$main_root" <<'PY' || python_rc=$?
import errno, os, re, subprocess, sys, types

(base, changed_path, broken, private_values_source, configuration_path, hostname_path,
 in_sample, all_lines, not_run_file, parent_override, main_root) = sys.argv[1:12]
in_sample, all_lines = in_sample == "1", all_lines == "1"
private_values = types.ModuleType("lib_private_values")      # 读改动的新加行与路径打码的共用函数（doc-text.sh 的 PRIVATE_VALUES_LIBRARY，与别的格同一份）
exec(compile(private_values_source, "private-values-library", "exec"), private_values.__dict__)
masking_targets = []
if configuration_path:
    try:
        masking_targets = private_values.load_private_targets(configuration_path, hostname_path, broken)[0]
    except OSError:
        masking_targets = []
HEAD_BYTES = 8192
NAME_CHARACTERS = "A-Za-z0-9._+-"
NAME_CHARACTER = re.compile(f"[{NAME_CHARACTERS}]")
TABLE_PATH = ".claude/out-of-project-paths"
PRIVATE_LIST = os.path.expanduser("~/.config/singlefs/out-of-project-names")
MINIMUM_REASON_CHARACTERS = 8
SOURCE_LABELS = {"home": "本机家目录", "parent": "仓的上一级目录", "private": "第二台私有名单", "root": "根下一层的系统目录"}
CONTEXT_LABELS = {"path": "名字类·路径段", "backtick": "名字类·反引号", "capacity": "名字类·带容量", "root": "容量类·根下系统目录带容量"}
CONTEXT_ORDER = ["path", "backtick", "capacity", "root"]
# 容量：数加单位（419G、16 GiB、3.5T、（82G） 里的 82G）；单位后面不能再接字母数字
CAPACITY = re.compile(r"(?<![A-Za-z0-9.])\d+(?:\.\d+)?[ \t]?(?:[KMGT]i?B|[KMGT])(?![A-Za-z0-9])")
# 名字与容量之间许夹的：收尾的反引号、空白、一个左括号或冒号或「约」
CAPACITY_GAP = re.compile(r"`?[ \t]*(?:[（(：:]|约)?[ \t]*$")
BACKTICK = re.compile(r"`([^`\n]+)`")
# 只由小写 ASCII 字母组成的名字（多半是普通英文词）不在「反引号里只有这个名字」的上下文里判，路径段与带容量照判
LOWERCASE_WORD = re.compile(r"[a-z]+")


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


def shown_path(path):
    return private_values.shown_path(path, masking_targets, broken)


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def entry_names(directory):
    try:
        return sorted(os.listdir(directory))
    except OSError as error:
        fail(f"列不出 {directory} 下一层的条目（{errno.errorcode.get(error.errno, error.errno)}）", "看这个目录在不在、有没有读权限；这一格要现读它下一层的条目名来比。")


def sample_listing(name):
    """样本根里的替身：第一行是那个目录的路径（.out-of-project-root 没有这一行），其余一行一个条目名；文件不在交回 None。"""
    try:
        lines = [line.strip() for line in open(name, encoding="utf-8").read().split("\n")]
    except FileNotFoundError:
        return None
    return [line for line in lines if line and not line.startswith("#")]


# ── 名字从哪来 ──
if in_sample:
    home_listing = sample_listing(".out-of-project-home") or ["/nonexistent-sample-home"]
    parent_listing = sample_listing(".out-of-project-parent") or ["/nonexistent-sample-parent"]
    home, home_names = home_listing[0], home_listing[1:]
    parent, parent_names = parent_listing[0], parent_listing[1:]
    root_names = sample_listing(".out-of-project-root") or []
    private_names = sample_listing(".out-of-project-private-names")
    repository_name = os.path.basename(os.getcwd())
else:
    home = os.path.expanduser("~")
    parent = parent_override or os.path.dirname(main_root)
    home_names, parent_names = entry_names(home), entry_names(parent)
    root_names = [name for name in entry_names("/") if os.path.isdir(os.path.join("/", name))]
    try:
        private_names = [line.strip() for line in open(PRIVATE_LIST, encoding="utf-8").read().split("\n")
                         if line.strip() and not line.strip().startswith("#")]
    except FileNotFoundError:
        private_names = None
    repository_name = os.path.basename(main_root)
if broken == "out-of-project-names-home-unread":            # 弄坏：不读本机家目录
    home_names = []
if broken == "out-of-project-names-parent-unread":          # 弄坏：不读仓的上一级目录
    parent_names = []
if broken == "out-of-project-names-private-unread":         # 弄坏：不读第二台私有名单（当它在、却是空的）
    private_names = [] if private_names is not None else None


def excluded(name):
    """singlefs 自己的：singlefs、singlefs- 开头的（上游 singlefs-ai-sop-* 也在内）与仓自己。"""
    if broken == "out-of-project-names-singlefs-unexcluded":   # 弄坏：不排掉 singlefs 自己的
        return name in {".", ".."}
    return name in {".", "..", "singlefs", repository_name} or name.startswith("singlefs-")


sources_of = {}             # 名字 → {来源}
for source, names in (("home", home_names), ("parent", parent_names), ("private", private_names or [])):
    for name in names:
        if not excluded(name):
            sources_of.setdefault(name, set()).add(source)
home_side = sorted((name for name, sources in sources_of.items() if sources & {"home", "private"}), key=len, reverse=True)
parent_side = sorted((name for name, sources in sources_of.items() if sources & {"parent", "private"}), key=len, reverse=True)
all_names = sorted(sources_of, key=len, reverse=True)
repository_top_names = set()
if broken != "out-of-project-names-repository-names-judged":   # 弄坏：仓里自己也有的顶层名照样按反引号、带容量判
    repository_top_names = set(os.listdir("."))
# ../<名> 里名字也是仓根下一层的条目的（../.claude/kb、include_str!("../../../.claude/…")、crates/../.claude），是仓里自己的相对路径，不判
relative_repository_names = set()
if broken != "out-of-project-names-relative-repository-judged":   # 弄坏：../ 后面接仓根下一层也有的名字照判
    relative_repository_names = set(os.listdir("."))
loose_names = sorted((name for name in all_names if name not in repository_top_names), key=len, reverse=True)
root_side = sorted(set(root_names), key=len, reverse=True)

# ── 像目录的上下文：路径前缀 ──
BOUNDARY = f"(?<![{NAME_CHARACTERS}])"
home_alternatives = [BOUNDARY + "~", r"\$HOME", r"\$\{HOME\}", BOUNDARY + re.escape(home), BOUNDARY + r"/home/[^/\s`'\"]+"]
HOME_PREFIX = "(?:" + "|".join(home_alternatives) + ")/"
# 不带边界的家目录写法：扫路径前缀（边界在 prefix_boundary 里逐处补判）与放行表的 XDG 回退写法（${XDG_CACHE_HOME:-$HOME/.cache}/singlefs 里 :- 紧接着家目录）共用
home_bare = "(?:~|\\$HOME|\\$\\{HOME\\}|" + re.escape(home) + "|/home/[^/\\s`'\"]+)/"
parent_alternatives = [BOUNDARY + re.escape(parent)]
if parent.startswith(home.rstrip("/") + "/"):
    parent_alternatives.append(HOME_PREFIX + re.escape(parent[len(home.rstrip("/")) + 1:]))
PARENT_PREFIX = "(?:" + "|".join(parent_alternatives) + ")/|(?<![A-Za-z0-9_+-])\\.\\./"
# singlefs 自己的路径（仓自己、上游 singlefs-ai-sop-*、家目录下 singlefs- 开头的）整段放过：~/<上一级>/singlefs 里那个上一级目录名不算命中
own_names = "singlefs[" + NAME_CHARACTERS + "]*" + ("" if repository_name.startswith("singlefs") else "|" + re.escape(repository_name))
OWN_SPAN_RE = re.compile(f"(?:{HOME_PREFIX}|{PARENT_PREFIX})(?:{own_names})(?![{NAME_CHARACTERS}])")


NAME_END = f"(?![{NAME_CHARACTERS}])"
# XDG 基目录与它的变量：放行表里 ~/<基目录>/… 那一条另认 ${<变量>:-<家目录>/<基目录>}/… 的回退写法
XDG_BASES = (("XDG_CACHE_HOME", ".cache"), ("XDG_CONFIG_HOME", ".config"), ("XDG_DATA_HOME", ".local/share"), ("XDG_STATE_HOME", ".local/state"))


def allow_pattern(path):
    """放行表里一条路径的命中范围，后面不能紧接名字字符。~/ 开头的这几种写法都当成同一条 ~/… 路径认：
    家目录写成 ~/、$HOME/、${HOME}/、家目录的绝对路径或 /home/<任一用户>/；XDG 回退 ${XDG_CACHE_HOME:-$HOME/.cache}/singlefs（另三个基目录同样）；
    python 拼接 os.path.expanduser("~/.local/share"), "singlefs"（家目录那一段的字串后面逐段跟着引号括起的各段）；
    末段是 singlefs 的，singlefs-<后缀>（~/.cache/singlefs-golden）也算 singlefs 自己的。XDG 基目录本身、停在半中间的路径不放。"""
    if not path.startswith("~/"):
        return re.compile(BOUNDARY + re.escape(path) + NAME_END)
    home_prefix, home_loose = HOME_PREFIX, home_bare
    if broken == "out-of-project-names-other-user-home-unrecognized":   # 弄坏：/home/<别的用户>/ 不当家目录认
        home_prefix = "(?:" + "|".join(home_alternatives[:-1]) + ")/"
        home_loose = "(?:~|\\$HOME|\\$\\{HOME\\}|" + re.escape(home) + ")/"
    segments = path[2:].split("/")
    escaped = [re.escape(segment) for segment in segments]
    if segments[-1] == "singlefs" and broken != "out-of-project-names-singlefs-suffix-unrecognized":   # 弄坏：singlefs-<后缀> 不认
        escaped[-1] += f"(?:-[{NAME_CHARACTERS}]*)?"
    forms = [home_prefix + "/".join(escaped) + NAME_END]
    for variable, base in XDG_BASES:
        depth = base.count("/") + 1
        if segments[:depth] != base.split("/") or len(segments) == depth:
            continue
        if broken != "out-of-project-names-xdg-fallback-unrecognized":   # 弄坏：XDG 回退写法不认
            forms.append(r"\$\{" + variable + ":-" + home_loose + "/".join(escaped[:depth]) + r"\}/" + "/".join(escaped[depth:]) + NAME_END)
        if broken == "out-of-project-names-xdg-base-allowed":   # 弄坏：XDG 基目录底下一律放过
            forms.append(home_prefix + "/".join(escaped[:depth]) + NAME_END)
    if broken != "out-of-project-names-python-join-unrecognized":   # 弄坏：python 拼接不认
        for split in range(1, len(segments)):
            forms.append(home_prefix + "/".join(escaped[:split]) + "[\"']\\)?"
                         + "".join("\\s*,\\s*[\"']" + segment + "[\"']" for segment in escaped[split:]))
    return re.compile("|".join(f"(?:{form})" for form in forms))


# ── 放行表 ──
allow_entries, table_problems = [], []      # (行号, 路径, 编译好的范围) / (行号, 哪里不对)
table_present = os.path.isfile(TABLE_PATH)
if table_present and broken != "out-of-project-names-allow-ignored":
    for number, raw in enumerate(open(TABLE_PATH, encoding="utf-8", errors="replace").read().split("\n"), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        found = re.fullmatch(r"(\S+)\s+#\s*(.*?)", line)
        if broken == "out-of-project-names-table-format-unchecked":   # 弄坏：不核写法，取行首一段当路径
            allow_entries.append((number, line.split()[0], allow_pattern(line.split()[0])))
            continue
        if not found:
            table_problems.append((number, "不是「<路径>  # 理由」的写法（路径后面要有 # 与理由）"))
        elif not (found.group(1).startswith("~/") or found.group(1).startswith("/")):
            table_problems.append((number, "路径要以 ~/ 或 / 开头（写 singlefs 读的那条仓外路径本身）"))
        elif len(found.group(2)) < MINIMUM_REASON_CHARACTERS:
            table_problems.append((number, f"理由不到 {MINIMUM_REASON_CHARACTERS} 个字"))
        else:
            allow_entries.append((number, found.group(1), allow_pattern(found.group(1))))

unreferenced = []           # (行号)：放行表里这一条没有一份 git 跟踪的脚本字面引到
if allow_entries and broken != "out-of-project-names-references-unchecked":
    listing = git("ls-files", "-z", "--", "*.sh", "*.py", "*.rs")
    if listing.returncode != 0:
        fail("列不出 git 跟踪的脚本", "看 git 是不是可用；放行表每一条都要有一份 git 跟踪的 .sh、.py 或 .rs 字面引到。")
    waiting = {number: pattern for number, _path, pattern in allow_entries}
    # 预筛的字面：路径（~/ 之后那一段）的每一段都要在；XDG 回退与 python 拼接把路径拆开写，不认整段字面
    tails = {number: [segment for segment in (path[2:] if path.startswith("~/") else path).split("/") if segment] for number, path, _pattern in allow_entries}
    if broken == "out-of-project-names-references-literal-tail":   # 弄坏：预筛认整段字面，拆开写的引用认不到
        tails = {number: [path[2:] if path.startswith("~/") else path] for number, path, _pattern in allow_entries}
    for script in listing.stdout.decode("utf-8", "replace").split("\0"):
        if not waiting:
            break
        if not script or script == TABLE_PATH or os.path.islink(script) or not os.path.isfile(script):
            continue
        try:
            text = open(script, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        # 先认路径各段的字面在不在，在了才用正则认几种写法：整仓几千份脚本逐份试正则太慢
        for number in [number for number, pattern in waiting.items() if all(segment in text for segment in tails[number]) and pattern.search(text)]:
            del waiting[number]
    unreferenced = sorted(waiting)


NAME_SET = frozenset("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._+-")
TOKEN_AT = re.compile(f"[{NAME_CHARACTERS}]+")
TOKEN_LIMIT = 256           # 往回认名字最多看这么多个字，再长的不当名字
# 扫路径前缀用不带后顾断言的写法（整份大文件里逐处试后顾断言太慢），边界在 prefix_boundary 里逐处补判
HOME_SCAN_RE = re.compile(home_bare)
parent_bare = [re.escape(parent)]
if parent.startswith(home.rstrip("/") + "/"):
    parent_bare.append(home_bare + re.escape(parent[len(home.rstrip("/")) + 1:]))
PARENT_SCAN_RE = re.compile("(?:" + "|".join(parent_bare) + ")/|\\.\\./")


def prefix_boundary(text, start):
    """前缀前面那个字：~、家目录、/home/ 与上一级目录前面不能紧接名字字符，../ 前面不能紧接名字字符里除 . 之外的，$HOME 不看。"""
    if start == 0 or text[start] == "$":
        return True
    before = text[start - 1]
    if text.startswith("../", start):
        return before == "." or before not in NAME_SET
    return before not in NAME_SET


def odd(names):
    """名字里有名字字符之外的字（空白、非 ASCII 这类）的，切 token 认不出来，逐个比。"""
    return [name for name in names if not TOKEN_AT.fullmatch(name)]


def name_at(text, position, name_set, odd_names):
    """position 起头、后面不紧接名字字符的那个名字；没有交回 None。"""
    found = TOKEN_AT.match(text, position)
    if found and found.group(0) in name_set:
        return found.group(0)
    for name in odd_names:
        end = position + len(name)
        if text.startswith(name, position) and (end == len(text) or text[end] not in NAME_SET):
            return name
    return None


def name_ending(text, end, name_set, odd_names):
    """在 end 收尾、前面不紧接名字字符的那个名字：(起点, 名字)；没有交回 None。"""
    start, limit = end, max(0, end - TOKEN_LIMIT)
    while start > limit and text[start - 1] in NAME_SET:
        start -= 1
    if start < end and (start == 0 or text[start - 1] not in NAME_SET) and text[start:end] in name_set:
        return start, text[start:end]
    for name in odd_names:
        start = end - len(name)
        if start >= 0 and text.endswith(name, 0, end) and (start == 0 or text[start - 1] not in NAME_SET):
            return start, name
    return None


ROOT_LEADING = frozenset(NAME_SET | set("/~$}"))
home_set, parent_set, loose_set, root_set = set(home_side), set(parent_side), set(loose_names), set(root_side)
home_odd, parent_odd, loose_odd, root_odd = odd(home_side), odd(parent_side), odd(loose_names), odd(root_side)
allow_names = {segment for _number, path, _pattern in allow_entries for segment in path.split("/")}
# 家目录或仓的上一级目录前缀收尾在这一处（~/、$HOME/、/home/<用户>/、<上一级>/、../）：它后面那个名字是第一段，不是深层段
FIRST_SEGMENT_PREFIX_RE = re.compile("(?:" + home_bare + "|(?:" + "|".join(parent_bare) + ")/|\\.\\./)\\Z")
FIRST_SEGMENT_WINDOW = len(home) + len(parent) + TOKEN_LIMIT


def deep_segment(text, start):
    """start 起头的名字是不是一条路径的深层段：前面紧挨着 /，/ 前面还有一段（名字字符或 / ~ $ }），
    而这个 / 不是家目录或仓的上一级目录前缀的收尾（~/<名>、$HOME/<名>、/home/<用户>/<名>、<上一级>/<名>、../<名> 里的名字是第一段，不算深层）。"""
    if start < 2 or text[start - 1] != "/" or text[start - 2] not in ROOT_LEADING:
        return False
    position = max(0, start - FIRST_SEGMENT_WINDOW)
    while True:
        found = FIRST_SEGMENT_PREFIX_RE.search(text, position, start)
        if not found:
            return True
        if prefix_boundary(text, found.start()):
            return False
        position = found.start() + 1


def line_allowed_spans(line):
    """一行里的放行范围（行内偏移）：singlefs 自己的路径与放行表每一条的命中范围。"""
    spans = []
    if broken != "out-of-project-names-singlefs-unexcluded":
        spans.extend(found.span() for found in OWN_SPAN_RE.finditer(line))
    for _number, path, pattern in allow_entries:
        spans.extend(found.span() for found in pattern.finditer(line))
    return spans


def judge(text):
    """一段文字（一行，或 --all-lines 时整份文件）里的命中：[(起点, 终点, 上下文, 名字)]，落在放行范围里的去掉；另交回放过了几处。
    放行范围只在有命中的那一行里现算（放行表的写法都在一行之内）。"""
    hits = []
    if broken != "out-of-project-names-path-unjudged":
        for scan_re, name_set, odd_names in ((HOME_SCAN_RE, home_set, home_odd), (PARENT_SCAN_RE, parent_set, parent_odd)):
            for found in scan_re.finditer(text):
                if not prefix_boundary(text, found.start()):
                    continue
                name = name_at(text, found.end(), name_set, odd_names)
                if name and found.group(0) == "../" and name in relative_repository_names:
                    continue
                if name and found.group(0) == "../" and broken == "out-of-project-names-relative-all-unjudged":   # 弄坏：../ 一律不判
                    continue
                if name:
                    hits.append((found.end(), found.end() + len(name), "path", name))
    if broken != "out-of-project-names-backtick-unjudged":
        for found in BACKTICK.finditer(text):
            content = found.group(1).strip()
            content = content[:-1] if content.endswith("/") else content
            if LOWERCASE_WORD.fullmatch(content) and broken != "out-of-project-names-lowercase-backtick-judged":   # 弄坏：只有小写字母的名字在反引号里也判
                continue
            if content in sources_of and content not in repository_top_names:
                hits.append((found.start(1), found.end(1), "backtick", content))
    for found in CAPACITY.finditer(text):
        gap = CAPACITY_GAP.search(text, max(0, found.start() - 12), found.start())
        end = gap.start() if gap else found.start()
        if broken != "out-of-project-names-capacity-unjudged":
            named = name_ending(text, end, loose_set, loose_odd)
            if named and broken == "out-of-project-names-lowercase-everywhere-unjudged" and LOWERCASE_WORD.fullmatch(named[1]):   # 弄坏：小写词带容量也不判
                named = None
            # 路径的深层段（~/.local/share/singlefs/work/<轮>/bin 里的 bin）不按名字类判，那条路径归不归本项目交路径段按第一段判
            if named and broken != "out-of-project-names-deep-segment-judged" and deep_segment(text, named[0]):   # 弄坏：路径深层段照按名字类判
                named = None
            if named:
                hits.append((named[0], end, "capacity", named[1]))
        if broken != "out-of-project-names-root-unjudged":
            root_end = end - 1 if end > 0 and text[end - 1] == "/" else end
            named = name_ending(text, root_end, root_set, root_odd)
            if named and named[0] > 0 and text[named[0] - 1] == "/" and (named[0] == 1 or text[named[0] - 2] not in ROOT_LEADING):
                hits.append((named[0] - 1, root_end, "root", named[1]))
    if broken == "out-of-project-names-root-without-capacity":   # 弄坏：根下系统目录不带容量也判
        for name in root_side:
            for found in re.finditer(r"(?<![" + NAME_CHARACTERS + r"/~$}])/" + re.escape(name) + f"(?![{NAME_CHARACTERS}])", text):
                hits.append((found.start(), found.end(), "root", name))
    kept, allowed, spans_by_line = [], 0, {}
    for start, end, context, name in sorted(set(hits)):
        if context != "root":
            if broken == "out-of-project-names-allow-whole-name":   # 弄坏：按名字放行（放行表路径里出现过的名字到处都放过）
                if name in allow_names:
                    allowed += 1
                    continue
            else:
                line_start = text.rfind("\n", 0, start) + 1
                if line_start not in spans_by_line:
                    line_end = text.find("\n", start)
                    spans_by_line[line_start] = line_allowed_spans(text[line_start: len(text) if line_end < 0 else line_end])
                if any(span_start <= start - line_start and end - line_start <= span_end for span_start, span_end in spans_by_line[line_start]):
                    allowed += 1
                    continue
        kept.append((start, end, context, name))
    return kept, allowed


def labels(hits):
    """一行命中的类别与来源：「名字类·路径段（本机家目录）」这样，不带名字本身。"""
    by_context = {}
    for _start, _end, context, name in hits:
        by_context.setdefault(context, set()).update(sources_of.get(name, {"root"}) if context != "root" else {"root"})
    order = ["home", "parent", "private", "root"]
    return "、".join(f"{CONTEXT_LABELS[context]}（来源：{'、'.join(SOURCE_LABELS[source] for source in order if source in by_context[context])}）"
                    for context in CONTEXT_ORDER if context in by_context)


def shown_names(hits):
    if broken == "out-of-project-names-print-names":            # 弄坏：红行把命中的名字也打出来
        return f"（名字：{'、'.join(sorted({name for _s, _e, _c, name in hits}))}）"
    return "（名字不打印）"


untracked_listing = git("ls-files", "-z", "--others", "--exclude-standard")
if untracked_listing.returncode != 0:
    fail("列不出未跟踪的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
untracked = {name for name in untracked_listing.stdout.decode("utf-8", "replace").split("\0") if name}
if all_lines:
    tracked_listing = git("ls-files", "-z")
    if tracked_listing.returncode != 0:
        fail("列不出 git 跟踪的文件", "看 git 是不是可用；--all-lines 逐行判的是 git ls-files 列出的每一份。")
    judged_paths = [name for name in tracked_listing.stdout.decode("utf-8", "replace").split("\0") if name]
else:
    judged_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]

findings, per_file, unreadable = [], {}, []     # (路径, 行号, 命中) / 路径 → 处数 / (路径, 错误)
files_judged, lines_judged, allowed_total, binary_files, links_skipped = 0, 0, 0, [], 0
for path in judged_paths:
    if os.path.islink(path):
        links_skipped += 1
        continue
    if not os.path.isfile(path):
        continue
    try:
        with open(path, "rb") as handle:
            head = handle.read(HEAD_BYTES)
        if b"\0" in head:
            binary_files.append(path)
            continue
        files_judged += 1
        if all_lines:
            text = open(path, "rb").read().decode("utf-8", "replace")
            lines_judged += text.count("\n") + (0 if text.endswith("\n") or not text else 1)
            hits, allowed = judge(text)
            allowed_total += allowed
            if hits:
                per_file[path] = len({(start, end) for start, end, _context, _name in hits})
            continue
        whole_file = path in untracked or broken == "out-of-project-names-whole-file"
        for number, line in private_values.added_lines(path, whole_file, base, git, fail, shown_path):
            lines_judged += 1
            hits, allowed = judge(line)
            allowed_total += allowed
            if hits:
                findings.append((path, number, hits))
    except OSError as error:
        unreadable.append((path, error))

private_note = None
if private_names is None:
    where = "样本根里的 .out-of-project-private-names" if in_sample else PRIVATE_LIST
    private_note = f"第二台私有名单（{where}）不在，第二台的目录名本次未查（不判红）"
    if broken != "out-of-project-names-private-missing-silent":
        print(f"  ! {private_note}；建它：一行一个名字，# 开头是注释，放第二台家目录与仓的上一级目录里的条目名")
        if not_run_file and not in_sample:
            with open(not_run_file, "a", encoding="utf-8") as handle:
                handle.write(f"doc-text.sh 的 out-of-project-names 格：{private_note}\n")

if findings:
    print(f"  ✗ 这次改动新加的 {len(findings)} 行写了不归本项目的目录名或根下系统目录的占用，在 {len({path for path, _, _ in findings})} 份文件里；只列文件、行号、类别与来源，不打印名字：")   # gate-lint:summary
    for path, number, hits in findings[:200]:
        print(f"      {shown_path(path)}:{number}  {labels(hits)}{shown_names(hits)}")   # gate-lint:detail
    if len(findings) > 200:
        print(f"      （另有 {len(findings) - 200} 行没列出）")   # gate-lint:detail
    print("     → 怎么办：改成不指名的说法（例「根分区其余不归 singlefs」）；要说空间只说本项目自己的目录与总的可用量；")
    print(f"               singlefs 自己读的仓外路径登记进放行表 {TABLE_PATH}（一行「<路径>  # 理由」，要有 git 跟踪的脚本引到它），只放行那条路径本身。")
if per_file:
    print(f"  ✗ 全仓 git 跟踪的 {files_judged} 份文本文件里 {len(per_file)} 份有不归本项目的目录名或根下系统目录的占用，共 {sum(per_file.values())} 处；只列每份的处数，不打印名字与行号：")   # gate-lint:summary
    for path in sorted(per_file):
        print(f"      {shown_path(path)}  {per_file[path]} 处")   # gate-lint:detail
    print(f"     → 怎么办：逐份改成不指名的说法（例「根分区其余不归 singlefs」），或把 singlefs 自己读的那条仓外路径登记进放行表 {TABLE_PATH}；")
    print("               已入库的冻结证据（research/prompts/ 这类）照批次定的不改；这一格平时只判这次改动新加的行，回扫后留着不改的旧行不会让它红。")
if table_problems:
    print(f"  ✗ 放行表 {TABLE_PATH} 有 {len(table_problems)} 行写得不对（只列行号，不打印路径）：")   # gate-lint:summary
    for number, what in table_problems:
        print(f"      {TABLE_PATH}:{number}  {what}")   # gate-lint:detail
    print(f"     → 怎么办：一行一条「<路径>  # 理由」，路径以 ~/ 或 / 开头，理由至少 {MINIMUM_REASON_CHARACTERS} 个字（写哪份脚本读它、读来做什么）。")
if unreferenced:
    print(f"  ✗ 放行表 {TABLE_PATH} 里 {len(unreferenced)} 条没有一份 git 跟踪的脚本（.sh、.py、.rs）字面引到（只列行号，不打印路径）：")   # gate-lint:summary
    for number in unreferenced:
        print(f"      {TABLE_PATH}:{number}  没有脚本引到这条路径（认 ~/、$HOME/、${{HOME}}/、家目录绝对路径、/home/<任一用户>/、XDG 回退与 python 拼接几种写法）")   # gate-lint:detail
    print("     → 怎么办：放行表只登记 singlefs 自己的脚本真读的仓外路径；没有脚本用它就删掉这一行，改成不指名的说法。")
if unreadable:
    print(f"  ✗ {len(unreadable)} 个文件读不了（权限这一类），这一格没法判：")   # gate-lint:summary
    for path, error in unreadable:
        print(f"      {shown_path(path)}  {errno.errorcode.get(error.errno, error.errno)}（{error.strerror}）")   # gate-lint:detail
    print("     → 怎么办：给自己补上读权限（chmod u+r）再跑；确实不该留的删掉。")
if findings or per_file or table_problems or unreferenced or unreadable:
    sys.exit(1)
if lines_judged == 0:
    print(f"  ! 这次改动没有新加的行（判了 {len(judged_paths)} 个路径，二进制 {len(binary_files)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
private_count = "本次未查" if private_names is None else f"{len(private_names)} 个条目名"
scope = f"全仓 git 跟踪的 {files_judged} 份文本文件的 {lines_judged} 行" if all_lines else f"这次改动里 {files_judged} 份文本文件新加的 {lines_judged} 行"
print(f"  ✓ 查了{scope}：对着本机家目录 {len(home_names)} 个条目名、仓的上一级目录 {len(parent_names)} 个条目名、第二台私有名单 {private_count}、根下一层 {len(root_side)} 个系统目录，没有一处不归本项目的目录名与占用")
print(f"     放过的：落在放行表路径或 singlefs 自己的路径里的 {allowed_total} 处；放行表 {len(allow_entries)} 条{'都有 git 跟踪的脚本引到' if allow_entries else '（表不在或是空的）'}；没判的：二进制 {len(binary_files)} 份、符号链接 {links_skipped} 个")
PY
  rm -f "$changed_file"
  exit "$python_rc"
}

cell_utf8_valid() {
  local base changed changed_file python_output python_rc=0 utf8_break=""
  cd "$ROOT" || exit 1
  private_value_sources
  if [[ "$text_notations_break" == utf8-valid-* ]]; then
    if ((PRIVATE_IN_SAMPLE)); then
      utf8_break="$text_notations_break"
    else
      echo "  ! 弄坏开关 $text_notations_break 只在样本根里认，这一趟被判的根不是样本，没理它，照常判"
    fi
  fi
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "  ! 被判的根不在 git 仓里，划不出「这一次改动」，这一格本次无对象可判（这不是通过）"
    exit 77
  fi
  base="$(gate_diff_base gate)"
  changed="$(gate_changed_paths "$base" untracked)" || { echo "  ✗ 取不到这一轮碰了哪些文件（基准 $base）"; echo "     → 怎么办：看 git 是不是可用、基准提交在不在；改动范围只经 scripts/gate/changed-paths.sh 取。"; exit 1; }
  changed_file="$(mktemp "${TMPDIR:-/tmp}/utf8-valid.XXXXXX")" || { echo "  ✗ 建不了放改动文件清单的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
  printf '%s\n' "$changed" > "$changed_file"
  python_output="$(python3 - "$changed_file" "$utf8_break" "$base" <<'PY'
import io, os, subprocess, sys

changed_path, broken, base = sys.argv[1:4]
HEAD_BYTES = 8192                            # 前 8 KiB 里有 NUL 的文件按二进制整份跳过
CHUNK_BYTES = 16 * 1024 * 1024               # 一块读多少；块边界上被切开的多字节序列带到下一块
LISTED_SKIPPED = 10                          # 「没查的」逐个列名的上限，其余只报个数
BINARY_SUFFIXES = (".gz", ".xz", ".zst", ".bz2", ".zip", ".tar", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".bin", ".img", ".qcow2",
                   ".so", ".a", ".o", ".rlib", ".rmeta", ".wasm", ".db", ".parquet")


def git(*arguments):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True)


def fail(what, howto):
    print(f"  ✗ {what}")
    print(f"     → 怎么办：{howto}")
    sys.exit(1)


def locate_invalid_utf8(handle):
    """handle 里第一处不合法的字节：None（整份合法）或 (字节偏移从 0 起算, 行号从 1 起算, 从那里起最多 4 个字节的十六进制)。
    弄坏开关 utf8-valid-lenient：用 replace 解码，永远不报错。"""
    offset = lines_before = 0
    carry = b""
    errors = "replace" if broken == "utf8-valid-lenient" else "strict"
    while True:
        block = handle.read(CHUNK_BYTES)
        final = not block
        data = carry + block
        carry = b""
        try:
            data.decode("utf-8", errors)
        except UnicodeDecodeError as error:
            if not final and error.reason == "unexpected end of data" and error.end == len(data) and broken != "utf8-valid-chunk-carry-dropped":
                carry, data = data[error.start:], data[:error.start]          # 块尾被切开的多字节序列：带到下一块再判
            else:
                bad = " ".join("0x%02X" % byte for byte in data[error.start:error.start + 4])
                return offset + error.start, lines_before + data.count(b"\n", 0, error.start) + 1, bad
        if final:
            return None
        lines_before += data.count(b"\n")
        offset += len(data)


changed_paths = [name for name in open(changed_path, encoding="utf-8").read().split("\n") if name]
index_entries = {}                           # 路径 → (模式位, blob 哈希)，只收普通文件
listing = git("ls-files", "-z", "--stage")
if listing.returncode != 0:
    fail("列不出暂存区的文件", "看 git 是不是可用；改动范围只经 scripts/gate/changed-paths.sh 取。")
for record in listing.stdout.split(b"\0"):
    meta, _, name = record.partition(b"\t")
    fields = meta.split()
    if name and len(fields) == 3 and fields[0] in (b"100644", b"100755"):
        index_entries[os.fsdecode(name)] = (fields[0], fields[1].decode())
drift = git("diff-files", "-z", "--name-only")
if drift.returncode != 0:
    fail("列不出工作区与暂存区不同的文件", "看 git 是不是可用。")
differs_from_index = {os.fsdecode(name) for name in drift.stdout.split(b"\0") if name}
staged = git("diff", "--cached", "--name-only", "-z", base)          # 暂存区相对基准有改动的路径：提交带走的是暂存区里的这一版
if staged.returncode != 0:
    fail(f"列不出暂存区相对基准 {base[:12]} 有改动的文件", "看基准提交在不在、git 是不是可用；改动范围的基准只经 scripts/gate/changed-paths.sh 的 gate_diff_base 取。")
staged_changed = {os.fsdecode(name) for name in staged.stdout.split(b"\0") if name}

findings = []                                # (路径, 哪一版, 偏移, 行号, 坏字节)
judged_working, judged_index = 0, 0
skipped = []                                 # (路径, 为什么)
for path in changed_paths:
    if os.path.islink(path):
        skipped.append((path, "符号链接"))
        continue
    if os.path.isfile(path):
        if path.lower().endswith(BINARY_SUFFIXES):
            skipped.append((path, "二进制后缀"))
        else:
            try:
                with open(path, "rb") as handle:
                    head = handle.read(HEAD_BYTES)
                    if b"\0" in head:
                        skipped.append((path, "二进制（前 8 KiB 里有 NUL）"))
                    else:
                        handle.seek(0)
                        judged_working += 1
                        found = locate_invalid_utf8(handle)
                        if found:
                            findings.append((path, "工作区") + found)
            except OSError as error:
                skipped.append((path, f"读不了（{error.strerror}）"))
    elif path not in index_entries:
        skipped.append((path, "路径已删"))
    if path in index_entries and path in staged_changed and path in differs_from_index and broken != "utf8-valid-index-unjudged" and not path.lower().endswith(BINARY_SUFFIXES):
        blob = git("cat-file", "blob", index_entries[path][1])
        if blob.returncode != 0:
            fail(f"读不出暂存区里 {path} 的内容（blob {index_entries[path][1][:12]}）", "看 git 是不是可用、暂存区有没有损坏（git fsck）。")
        if b"\0" in blob.stdout[:HEAD_BYTES]:
            skipped.append((path + "（暂存区）", "二进制（前 8 KiB 里有 NUL）"))
            continue
        judged_index += 1
        found = locate_invalid_utf8(io.BytesIO(blob.stdout))
        if found:
            findings.append((path, "暂存区") + found)

if findings:
    print(f"  ✗ 这次改动里 {len(findings)} 份文本文件不是合法 UTF-8（字节偏移从 0 起算、行号从 1 起算，坏字节是从偏移处起最多 4 个字节）：")   # gate-lint:summary
    for path, side, offset, number, bad in findings:
        print(f"      {path}（{side}）  字节偏移 {offset}（第 {number} 行），坏字节 {bad}")   # gate-lint:detail
    print("     → 怎么办：对着行号和偏移看那一处是不是多字节字符被拦腰切断了（shell 的 paste -d、cut -c、head -c 这类按字节切的命令切多字节字符会这样），"
          "用 Edit 或 replace-once.py 按字符定点改回来；暂存区里坏而工作区已修好的，git add 重新暂存那一份；")
    print("               确实是二进制的，补进本格 BINARY_SUFFIXES（.claude/gate.d/doc-text.sh 里 cell_utf8_valid 的后缀名单），不要放过文本。")
    sys.exit(1)
if judged_working + judged_index == 0:
    print(f"  ! 这次改动里没有文本文件可判（改动范围 {len(changed_paths)} 个路径，没查 {len(skipped)} 份），这一格本次无对象可判（这不是通过）")
    sys.exit(77)
print(f"  ✓ 查了这次改动里 {judged_working} 份工作区文本文件、{judged_index} 份暂存区里与工作区不同的版本：整份按 UTF-8 严格解码，没有一处非法字节")
listed = "；".join(f"{path}（{why}）" for path, why in skipped[:LISTED_SKIPPED])
extra = f"；…另有 {len(skipped) - LISTED_SKIPPED} 份" if len(skipped) > LISTED_SKIPPED else ""
print(f"     没查的 {len(skipped)} 份：{listed or '无'}{extra}")
PY
)" || python_rc=$?
  rm -f "$changed_file"
  if [[ -n "$python_output" ]]; then
    # 红行与「没查的」里的路径经 private-values 那一份打码（shown_lines_main 的 lines：带 / 的一段当路径，别的当行文）
    printf '%s\n' "$python_output" | python3 -c 'import sys, types; library = types.ModuleType("lib_private_values"); exec(compile(sys.argv[1], "private-values-library", "exec"), library.__dict__); library.shown_lines_main(sys.argv[2:], sys.stdin, sys.stdout)' \
      "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" lines
  fi
  exit "$python_rc"
}

stage_cells_run "$ROOT"
