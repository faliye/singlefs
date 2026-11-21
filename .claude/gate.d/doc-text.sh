#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 文本写法：全仓不许出现登记过的禁用写法（撇号类角标、描述里的钟点与时区词、会话号）；改动碰到的记录与 kb 页日期只写在标题上；登记的三语文件（仓根 README 与 briefs/）三份齐全、结构对得上、改了中文就三份同进同出（只改译文的放行）、译文照翻译术语表用词、译文里不残留超量没译的中文、译文与中文逐段比内容量不漏段不压成摘要；这次改动新加到记录、kb 页、三方判决正文与代码治理文件的行里不写账号与额度方面的运营细节和解析限额通知的正则；这次改动新加的行里不写不归本项目的目录名与根下系统目录的占用；这次改动里的文本文件（工作区与暂存区里的版本）是合法 UTF-8；登记的 kb 目录里的中文页在历史节之前不写带日期的现状句（滞后表登记的存量放过，滞后表只缩不涨）；代码与脚本的注释、文件头、文档串与 README 里不写历史（日期、出处、改动经过；特殊缘由写 history-kept 标记，存量登记在滞后表，滞后表只缩不涨）；这次改动里的 .md、.tsv、.txt（research/prompts/ 下的冻结证据除外）相对基准多出来的行不写任务号、括号里的会话短号与批号
# gate-fixture: yes
# gate-category: 文档类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 的两项判被测代码的行为，这一道只判文本写法。
# gate-similar: code-tooling.sh 它的 fixture-claims 格也扫全仓文件，但判的是文件名与样本目录里编出来的日期、阶段头部声称的样本在不在，不是文本里的写法
# gate-similar: code-source-discipline.sh 它的 vague-names 与 test-file-names 两格也判写法，但对象是 .rs 里声明的函数名、类型名与测试文件名，读的是空泛词表与按里程碑起名的正则，不扫全部文本
# gate-similar: .claude/singlefs-ai-sop/scripts/i18n-sync.sh 它判上游各语言仓之间的同步（兄弟目录里的仓、清单、溯源哈希）；trilingual-* 六格判本仓里登记的英、中、日三份文件，对象与输入都不同，不并进它
# gate-similar: .claude/singlefs-ai-sop/scripts/link-targets.py 它读到不是 UTF-8 的 markdown 时把那份判红（读不了、里面的链接没查），范围是全仓的 .md、判的是链接指不指得到；utf8-valid 一格判这次改动里每份文本文件（含暂存区里与工作区不同的那一版）的字节本身合不合法，范围与对象都不同，沿用本道已有的改动范围取法与路径打码，不另立门禁
# gate-similar: .claude/hooks/write-guard.sh 它在 Write / Edit 当场判写进的内容（lib-forbidden-notations.py 的禁用写法），不读本机目录与放行表，也不看这一次改动划出的新加行；out-of-project-names 一格也不并进同一道的 private-values 格：词源（家目录与仓的上一级目录现读、第二台私有名单）、只在像目录的上下文里判、放行表连同它要有脚本引到、私有名单不在时照判别的来源而不是整格 77，判法与出路都不同，只共用新加行的取法与路径打码（PRIVATE_VALUES_LIBRARY 的 added_lines、shown_path）
# gate-similar: .claude/singlefs-ai-sop/scripts/doc-lint.sh 它也扫 .claude/kb-directories 登记的 kb 目录，判的是位置指代、时间指代、自称、编号简称与历史节收尾，不判带日期的现状句；上游副本不许加本工程的判据，滞后表与 gate 基准比的只缩不涨也是本仓的读法（lib-lag-baseline.py），dated-status 一格不并进它，只照它读 kb 目录登记（上游 lib.sh 的 kb_directories，同一份判法）
#
# gate-cell: prime-marks 撇号类角标
# gate-cell: clock-times 描述里的钟点、时区词与时间戳
# gate-cell: session-names 会话号（ListAgents 列的会话名）
# gate-cell: date-once 这次改动碰到的记录与 kb 页里，日期只在标题上：正文行不重复所属标题或文件名里的日期，同一天不开第二个同级小节
# gate-cell: trilingual-files 登记的三语文件三份齐全：中文 <名>.md 的 <名>.en.md、<名>.ja.md 都在，登记目录里单有 .en.md 或 .ja.md 的也要三份齐全；登记表写对了
# gate-cell: trilingual-structure 三语文件每组三份的结构对得上（编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数），抓只改了一份
# gate-cell: trilingual-changed-together 这次改动碰了一组三语文件里的中文那一份，英语、日语两份都要碰；只改译文的组放行
# gate-cell: trilingual-glossary 翻译术语表写对了，三语文件的译文照术语表用词
# gate-cell: trilingual-untranslated 三语文件的英语、日语译文去掉反引号、引文、链接目标与路径之后，一行里不残留超量没译的中文（英语数汉字，日语数日语不用的简体字），与中文那一份某一行逐字相同而去段后还剩中文的行也不许有
# gate-cell: trilingual-omission 三语文件的英语、日语译文与中文逐段配对比内容量（表格行按首列编号配对，其余按节内次序）：过短（漏了一段、压成摘要）、过长（对错了段）与配不上的段（段数不同的节、编号行缺一边）都不许有
# gate-cell: private-values 这次改动新加的行与改动的文件名里不出现本机私有配置 multi-host.env 里的值（8 个字以上的词）与本机主机名
# gate-cell: account-quota 这次改动新加到 records/、.claude/kb/、三方判决正文，与钩子、脚本、门禁、规则、定义、task-server、prune-server（样本除外）的行里不写账号与额度方面的运营细节，也不写解析限额通知的正则（词表在 cell_account_quota；显卡额度、内存上限、空间配额不在内）
# gate-cell: out-of-project-names 这次改动新加的行里不写不归本项目的目录名（本机家目录、仓的上一级目录与第二台私有名单里的条目名，只在像目录的上下文里判）与根下系统目录的占用；放行表 .claude/out-of-project-paths 每一条要有 git 跟踪的脚本引到
# gate-cell: utf8-valid 这次改动（暂存区或工作区相对基准）里的文本文件是合法 UTF-8：非法的点名文件、工作区还是暂存区、第一处坏字节的偏移
# gate-cell: dated-status 登记的 kb 目录里的中文页在「## 历史版本」之前不写带日期的现状句（「现状（日期」与「日期 现状」两种）；滞后表 dated-status-lag.tsv 登记的放过，滞后表写对了、与 gate 基准那一版比只缩不涨
# gate-cell: comment-history 代码与脚本的注释、文件头、文档串与 README 里不写历史（日期、出处、改动经过）；特殊缘由在那一行写 history-kept 标记，存量登记在滞后表 comment-history-lag.tsv，滞后表写对了、与 gate 基准那一版比只缩不涨
# gate-cell: task-and-session-numbers 这次改动里的 .md、.tsv、.txt（research/prompts/ 下的冻结证据除外）相对基准多出来的行（新加的行与因围栏、表头改动而成了违例的行）不写任务号、括号里的会话短号与批号（判据是 lib-forbidden-notations.py 的 task-and-session-numbers 那一形态，基准里已有的不报）
# 格结构经 lib/stage-cells.sh。
# 预取：各格在自己的子 shell 里跑，取了也传不给别的格，所以改动范围与新加行的取法在父进程里按选中的格取一次：date-once、trilingual-changed-together、private-values、account-quota、
#   out-of-project-names、utf8-valid、task-and-session-numbers 七格共用一份改动路径（text_changed_paths），private-values、account-quota、out-of-project-names 三格另共用一份整份 git diff -U0
#   （PRIVATE_VALUES_LIBRARY 的 load_diff_segments 按文件切开、added_lines 取；预取里没有的文件退回逐份 git diff；预取取不到的各格照旧各自判红）。
#
# ── prime-marks、clock-times、session-names ──
# 格名就是同目录 lib-forbidden-notations.py 里 FORMS 的形态键。git ls-files --cached --others --exclude-standard 列出的每份文本文件
# （不在 git 仓里时走目录树）逐行交那一形态判，违例报「文件:行」。判据行、上下文放行规则、放行标记、排除前缀、成功行与出路都在那份库里；
# 加一种禁用写法就是往 FORMS 加一项、再在格名表里加一格。.claude/hooks/write-guard.sh 写入时读同一份库。
# 射程是全仓，冻结证据（research/prompts/、records/、变更史、research/results/）也在内。
# 扫法：各份文件互不依赖，分给 fork 出来的子进程各扫各的，进程数取每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads；取不到往标准错误打一行、一个进程扫），
#   结果按文件次序收回（派出多少份、收回多少份对不上判红），输出与进程数无关；每份文件逐行判用 lib-forbidden-notations.py 的 hits_per_line：
#   形态此刻要判的判据行连成一条正则先筛掉没有候选的行，有候选的行才走上下文规则与放行标记，与逐行 first_hit 逐项相同（first_hit 留作对拍的参照）。
# 没扫的：二进制文件（前 8 KiB 里有 NUL）、列出了却不是普通文件的与读不了的（权限这一类，不让异常回溯打出原路径），逐个列名。一份文本文件都没扫到：退 77。
# 打印的路径（红行、「二进制：」「不是普通文件」「读不了」三个清单）与红行里的行文片段都经 private-values 那一格的取词和打码（下面「private-values」一段的 PRIVATE_VALUES_LIBRARY，同一份共用函数）：
#   路径里含私有值的那一段打成「〔文件名含私有值〕」，片段里的值打成「〔私有值〕」；没有 multi-host.env（别人的克隆、第二台）就没有词可打，原样打。
#   片段取命中位置前 20 个字到后 10 个字，两头落在私有值中间的挪到值的边上（整值进片段再打码，snippet_bounds），不打出半截值。
# 管不到的：ASCII 单引号当角标（与代码里的引号分不开）；改过标题的会话。
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
#   碰了一组里的中文那一份，英语、日语两份都要碰，删掉的也算碰；中文那一份没碰、只碰了译文的组放行（只在中文变了时要求同改，
#   译得对不对归 structure、glossary、untranslated 三格），成功那句与红的汇总另报放行几组；一组都没碰到、或不在 git 仓里：退 77。
# glossary：翻译术语表 .claude/kb/translation-glossary.md，表上方单独一行 <!-- trilingual-glossary --> 紧挨着表，表头
#   | 中文 | English | 日本語 | 说明 / Note / 注 |。表本身：每行四列、前三列不空、中文列不重复、说明列空着或恰好三段（用 <br> 分隔、每段不空）。
#   译文：每组三份齐全的，中文列的串在 zh 那份里原样出现，则 English 列的串要在 en 那份里出现（不分大小写），日本語列的串要在 ja 那份里原样出现。
#   改名登记表 .claude/kb/term-renames.md 那一组，登记表的旧名、新名两列（新旧对照，三份照中文原串写、不译）不算用到术语，
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
# 取哪些词：每个键的值按 shell 的写法切词（再按 , ; @ 切一遍），只取 8 个字以上的长词（SLICE_MEMORY_CAP_BY_HOST、ENABLE_ACROSS_MACHINES、ENABLE_GPU 三个键与 systemctl、is-active 这类命令里的通用词不取）；
#   按台的键（BY_HOST_KEYS：CARGO_BIN_DIRECTORY_BY_HOST、BUILD_ROOT_BY_HOST，每项 <地址>=<值>）每项从第一个 = 切开，只取值那一截（再按 , ; @ 切一遍）：地址照 HOSTS 取，整项不取；
#   本机主机名取 hostname（短于 8 个字、或就是 localhost 的不取）。门槛 8 个字照攻方的量：4–7 个字的词（清场命令里的选项、显卡配置里的机器标签）在冻结副本里各命中 221–2001 份文件，
#   8 个字以上的只命中两份真泄漏（research/prompts/m3-defs-b3-r1-opus-output.md「E3」）。
#   共享根里的路径不取：NFS_SHARED_ROOTS 冒号隔开的各段（绝对路径，根目录 / 不算）是两台挂在同一绝对路径上、进仓的文字本来就写着的路径；哪个键的词（冒号隔开的每一段）
#   是某一段本身或在它下面（按整级目录比），就不当私有值（SHARED_DATA_ROOT 这类键，与 NFS_SHARED_ROOTS 自己），成功句「没判的」里报个数与键名。按值判，不按键名列豁免。
#   纯数字与容量（60G、1.5GiB）不取，IPv4 地址（四段数字，HOSTS 里的第二台）照取。
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
#   prime-marks / clock-times / session-names 三格打路径与片段、date-once 打红行路径、utf8-valid 打红行都调同一份，不各抄一份。
# 没有 multi-host.env（别人的克隆、第二台）、取不到够长的词、不在 git 仓里、这次改动没有新加的行：退 77，不判绿。
# 管不到的：换了写法的值（大小写、拆开写、换成别称）、用户原话里的别称（不在配置里）、commit message（提交之后才有，要另挂 commit-msg 钩子）、不足 8 个字的值。
#   词源只有 multi-host.env 与 hostname：~/.config/singlefs/ 下的清场脚本里的词没并进来（并入前要在真树上整仓逐行量新增命中，新增为 0 才并；上一次量出来不是 0，没并）。
# ── account-quota ──
# 记录、kb 页、三方判决正文、代码与治理文件里不写账号切换与用量限额：撞了哪种额度、换没换账号、额度什么时候重置， account-quota:allow 词表与样本说明
# 这些是子 agent 停下的原因，不是要留下的事实；子 agent 因此停下的，只写「中途停下没交回」。
# 落点在这一道而不是上游 scripts/doc-lint.sh：那一份只扫 kb 目录，不扫 records/ 与 research/prompts/；这一道本来就有判「这次改动新加的行」的格（private-values）。
# 射程：这次改动新加的行（scripts/gate/changed-paths.sh 的 gate 取法的基准到工作区；未跟踪的新文件整份都算新加；旧行不报），看两类文件：
#   ① 记录与三方判决正文：records/ 与 .claude/kb/ 下的 .md、research/prompts/ 顶层的 *-main-verification.md。
#      research/prompts/ 里别的文件（_*-appendix.md、_*-background.md、_*-body.md、*-snapshot/ 下的副本）是从仓里抄给云端腿读的，抄进来的旧话不算这一次写的，不判。
#   ② 代码与治理文件：.claude/hooks/、scripts/、.claude/gate.d/、.claude/rules/、.claude/agents/、
#      .claude/main-agent.md、.claude/agent-common.md、task-server/、prune-server/ 下的文本文件，不限后缀（含 .py、.sh、.rs、.tsv；二进制整份跳过）；
#      .claude/gate.d/fixtures/ 除外（样本要造这些话才能证红）。仓里不写按用量限额挑模型、拦派发的东西：解析限额通知的正则、按通知拦派发的放行行都不进仓。 account-quota:allow 词表与样本说明
#      代码里为了造限额通知而写的字面、词表与出路说明本身，在那一行写 account-quota:allow <理由>（注释里写，理由至少 4 个字）。
# 词表在 cell_account_quota 的 python 里，STRICT 与 BARE 两张表，唯一一处，每条写明键（红行点名它）和拦什么：
#   明说层（STRICT）：句子里写明哪个账号、哪一种限额（会话、周、每周、五小时、Opus、Sonnet、Haiku、Claude、API、账号、订阅后面接限额或额度），或是限额通知原文（含 resets 后接月份缩写、日、十二小时制钟点和括号里时区缩写的那一形态，与解析它的正则），
#     或「切换账号」「换了账号」「账号切换」「另一个账号」，或「限额窗口」「额度窗口」「会话上限」「周上限」，命中就红，不看同一行别的词。 account-quota:allow 词表与样本说明
#   裸词层（BARE）：没写是哪一种的「撞限额」「撞了额度」「限额撞顶」「额度恢复」「额度重置」「被额度打断」「额度用完」「用量限额」「用量上限」；命中处前后 24 个字内有显卡、GPU、显存、内存、线程、 account-quota:allow 词表与样本说明
#     账上、磁盘、空间、inode、文件系统、存储、quota 或「数字加 G / GiB / GB / MiB / MB」的放过（只看命中处附近，不看整行）：那是显卡额度、内存额度、空间配额、账上多少 G 这类算力与空间资源的正常用法。
#   不在词表里的：「额度」「限额」「上限」「配额」单独出现（显卡额度、GPU_CARDS_BY_PRIORITY 的额度、内存上限、空间配额）；「欠账号」（欠账编号）；「换号」「重写换号」（版本号）；
#     「账户」；「限流」单独出现（只有跟在「撞」后面才算）。
# 放行：行里写「account-quota:allow <理由>」（理由至少 4 个字）整行放过，md 里也认；没写理由的标记不放过，另报一类红行。用在用户原话里带这些词、要原样留的地方，和引用 three-way-inference.md 那条规则标题的地方。
# 误伤面（对着射程内 719 份文件、12 万行量的）：词表命中 72 行，逐行读过，没有一行是显卡、内存或空间配额的用法；资源词放过 1 行（撞额度：账上 8G，内存额度）。
#   裸词层只看命中处前后 24 个字里有没有资源词，不看整行：整行有资源词就放的话，行里别处提到线程上限、草稿目录多少 G 的模型限额会被放过。
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
# ── task-and-session-numbers ──
# 进仓的文字不拿号指东西（.claude/agent-common.md「写」一节「进仓的文字……不拿号指东西」那一条）：任务服务的任务号、括号里的会话短号、批号，
#   与拿「<做什么>那一路」的描述指别的会话（「那一路会话」「那一路的 agent」；按出场次序的 mainagent(A)、subagent(A) 不判）。
# 写闸、检出钩子 ⑰ 与 scripts/kb/ 的写入工具在写的那一刻判；这一格补它们够不着的路：cp、mv 与别的命令的输出写进仓的，到提交时在这里判。
# 判据是同目录 lib-forbidden-notations.py 的 task-and-session-numbers 那一形态（判据行、上下文规则、围栏与表头、只判 .md、.tsv、.txt、排除前缀都在库里），这里不另抄；
#   判法同写入一侧：每份改动里的文件，基准里的样子（git cat-file 取 gate 基准那一版；搬家、改名过来的按 git diff -M 认出的改名对取基准里旧路径那一份（renamed_sources），
#   新文件、基准是空树的当空串）与此刻的样子两份整文交库的 new_violation_hits，
#   写后多出来的违例行才报（新加的行，与这一次删、加、改坏一行围栏或改了表头而成了违例的旧行）；基准里已有的违例行不报（旧记录里的号不回改）。
# 改动范围取 scripts/gate/changed-paths.sh 的 gate 取法（text_changed_paths，与 account-quota 那几格共用父进程预取的那一份）；二进制（前 8 KiB 里有 NUL）与符号链接不判，报个数。
# research/prompts/ 下的文件（三方材料、判决与它们的快照副本）是冻结证据，写的是当时的原文、不回改，这一格不判、只报个数；判决 *-main-verification.md 在写的那一刻另有写闸与写入工具判。
#   records/、.claude/kb/ 与别处的 .md、.tsv、.txt 照判。
# 不并进别的格：prime-marks、clock-times、session-names 三格扫全仓，旧记录里的号会一直红；account-quota、private-values、out-of-project-names 判的是新加行里的另一批词，
#   而这一形态的围栏与表头要整份认，只看 diff 里的新加行认不出（上游 .claude/singlefs-ai-sop/scripts/code-name-lint.py 也只判新加行，判的是代号写法、不认这份库，且上游副本不收本工程的判据）。
# 输出：红行报文件、行号、判据行名字与命中的串（路径与串经 PRIVATE_VALUES_LIBRARY 打码），出路是那一形态的 remedy。
# 没有 git 仓、这次改动没有要判的 .md、.tsv、.txt（research/prompts/ 下的不算要判的）：退 77，不判绿。读不到判据库判红。
# 管不到的：那一形态文件头「管不到的」那几样；research/prompts/ 下新写进去的号；commit message（提交之后才有）；
#   git diff -M 认不出改名对的搬家（新路径还没 git add、改得太多相似度不到一半），基准取不到原样，里面已有的号照报。
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
#   GATE_TEXT_NOTATIONS_BREAK=scan-prefilter-rejects-all  prime-marks、clock-times、session-names 三格逐行扫时，先筛一律说没有候选（转成 LIB_FORBIDDEN_NOTATIONS_BREAK 的 candidate-prefilter-rejects-all 交给 lib-forbidden-notations.py）
#     clock-times-mountinfo-red
#     clock-times-red
#     prime-marks-red
#     private-values-clock-snippet-red
#     private-values-siblings-red
#     session-names-red
#   GATE_TEXT_NOTATIONS_BREAK=changed-paths-prefetch-empty  阶段在父进程里预取的改动路径一律交空表（用改动范围的七格拿到空的改动范围）
#     account-quota-code-green
#     account-quota-code-red
#     account-quota-green
#     account-quota-red
#     date-once-green
#     date-once-history-translations-green
#     date-once-history-translations-red
#     date-once-red
#     date-once-unjudgeable-red
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
#     private-values-big-untracked-red
#     private-values-cross-segment-red
#     private-values-filename-and-content-red
#     private-values-filename-only-red
#     private-values-green
#     private-values-nul-head-green
#     private-values-red
#     private-values-shared-root-address-red
#     private-values-shared-root-green
#     private-values-siblings-red
#     private-values-symlink-dangling-red
#     private-values-symlink-mount-red
#     private-values-too-big-red
#     private-values-unreadable-red
#     task-and-session-numbers-descriptions-green
#     task-and-session-numbers-descriptions-red
#     task-and-session-numbers-green
#     task-and-session-numbers-prompts-green
#     task-and-session-numbers-red
#     task-and-session-numbers-renamed-green
#     task-and-session-numbers-renamed-red
#     trilingual-changed-together-green
#     trilingual-changed-together-red
#     trilingual-changed-together-translation-only-green
#     utf8-valid-chunk-green
#     utf8-valid-chunk-red
#     utf8-valid-green
#     utf8-valid-red
#     utf8-valid-staged-red
#   GATE_TEXT_NOTATIONS_BREAK=prefetch-error-dropped  预取改动路径失败时不把预取记下的 git 报错打进格的输出（text_changed_paths）
#     trilingual-changed-together-git-error-red
#   GATE_TEXT_NOTATIONS_BREAK=diff-cache-hunks-dropped  阶段预取的整份 diff 里每份文件都说没有新加的行（读新加行的三格对预取到的文件拿到空）
#     account-quota-code-green
#     account-quota-code-red
#     account-quota-green
#     account-quota-red
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
#     private-values-green
#     private-values-nul-head-green
#     private-values-red
#     private-values-shared-root-address-red
#     private-values-shared-root-green
#   GATE_TEXT_NOTATIONS_BREAK=date-once-unjudged  date-once 不干跑、记无对象可判
#     date-once-green
#     date-once-history-translations-green
#     date-once-history-translations-red
#     date-once-red
#     date-once-thread-default-red
#     date-once-unjudgeable-red
#     private-values-date-once-path-red
#   GATE_TEXT_NOTATIONS_BREAK=date-once-exit-ignored  date-once 不收干跑的退出码：干跑出错的那一份读不到摘要、当干跑 0 处
#     date-once-unjudgeable-red
#   GATE_TEXT_NOTATIONS_BREAK=date-once-history-translations-judged  date-once 只排除中文那三份（两份变更史与 INDEX.md），英日译文照判
#     date-once-history-translations-green
#     date-once-history-translations-red
#   GATE_TEXT_NOTATIONS_BREAK=date-once-jobs-unregistered-passes  date-once 取不出每台默认重活线程数时退回一路、照常往下判（真仓里登记读得出，这条开关只在取不出时有差别，样本根里放 .sample-thread-cap-invalid 造出取不出）
#     date-once-thread-default-red
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
#     trilingual-changed-together-git-error-red
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
#     trilingual-untranslated-comment-open-in-code-red
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
#     trilingual-untranslated-comment-open-in-code-red
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
#     trilingual-omission-comment-open-in-code-red
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
#   LIB_TRILINGUAL_BREAK=link-target-ids-counted  编号集合也数链接目标 ](…) 里的编号
#     trilingual-structure-link-target-red
#   LIB_TRILINGUAL_BREAK=partial-ignored  碰了一份就当三份都碰了
#     trilingual-changed-together-red
#   LIB_TRILINGUAL_BREAK=translation-only-required  只碰了译文（中文那一份没碰）的组也要三份都碰
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
#   LIB_TRILINGUAL_BREAK=glossary-renames-columns-counted  改名登记表的旧名、新名两列也算用到术语
#     trilingual-glossary-renames-columns-red
#   LIB_TRILINGUAL_BREAK=omission-ratio-ignored  trilingual-omission 不比内容量
#     trilingual-omission-comment-open-in-code-red
#     trilingual-omission-english-red
#     trilingual-omission-japanese-red
#   LIB_TRILINGUAL_BREAK=omission-keys-ignored  trilingual-omission 的表格行不按首列编号配对，一律按节内次序
#     trilingual-omission-english-red
#     trilingual-omission-green
#   LIB_TRILINGUAL_BREAK=omission-unpaired-ignored  trilingual-omission 配不上的段（段数不同的节、编号行缺一边）不报
#     trilingual-omission-english-red
#   LIB_TRILINGUAL_BREAK=comment-open-in-code-counted  反引号段里的 <!-- 也当 HTML 注释的开头（trilingual-omission 的切节与 trilingual-untranslated 的逐行去注释都退回去）
#     trilingual-omission-comment-open-in-code-red
#     trilingual-untranslated-comment-open-in-code-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-unjudged  private-values 不比对、记无对象可判
#     private-values-big-untracked-red
#     private-values-cross-segment-red
#     private-values-filename-and-content-red
#     private-values-filename-only-red
#     private-values-green
#     private-values-no-config
#     private-values-nul-head-green
#     private-values-red
#     private-values-shared-root-address-red
#     private-values-shared-root-green
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
#     private-values-shared-root-green
#   GATE_TEXT_NOTATIONS_BREAK=private-values-hostname-skipped  不取本机主机名
#     private-values-green
#     private-values-nul-head-green
#     private-values-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-print-values  红行里把命中的值也打出来
#     private-values-big-untracked-red
#     private-values-filename-and-content-red
#     private-values-red
#     private-values-shared-root-address-red
#     private-values-symlink-dangling-red
#     private-values-symlink-mount-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-big-file-truncated  未跟踪文件只读前 8 MiB、超大也不判红
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
#   GATE_TEXT_NOTATIONS_BREAK=private-values-siblings-unmasked  prime-marks / clock-times / session-names 三格打路径与行文片段不打码
#     private-values-clock-snippet-red
#     private-values-siblings-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-snippet-cut  三格的行文片段照命中位置前 20 后 10 个字直接截，不挪到私有值的边上
#     private-values-clock-snippet-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-date-once-unmasked  date-once 的红行路径与 utf8-valid 的红行原样打路径
#     private-values-date-once-path-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-path-by-segment  路径打码逐段比（哪一段含整个值才打码，值带 / 跨几段时一段都不含）
#     private-values-cross-segment-red
#     private-values-siblings-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-links-followed  不判符号链接：跟着链接走，坏链接整份跳过、好链接读它指到的内容
#     private-values-symlink-dangling-red
#     private-values-symlink-mount-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-unreadable-raises  读文件不包异常：读不了就让格崩，python 回溯把带假仓路径的原路径打出来
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=private-values-shared-roots-judged  共享根（NFS_SHARED_ROOTS 各段）里的路径也当私有值
#     private-values-shared-root-address-red
#     private-values-shared-root-green
#   GATE_TEXT_NOTATIONS_BREAK=private-values-addresses-skipped  IPv4 地址当纯数字跳过、不取
#     private-values-shared-root-address-red
#     private-values-shared-root-green
#   GATE_TEXT_NOTATIONS_BREAK=private-values-by-host-unsplit  按台的键照别的键取整项（<地址>=<值> 整串），不从第一个 = 切出值（假第二台的编译根取不到）
#     private-values-clock-snippet-red
#     private-values-cross-segment-red
#     private-values-red
#     private-values-siblings-red
#     private-values-symlink-dangling-red
#     private-values-unreadable-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-unjudged  account-quota 不判、记无对象可判
#     account-quota-code-green
#     account-quota-code-red
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-words-skipped  词表与放行标记整个不判（没有这一格的样子）
#     account-quota-code-green
#     account-quota-code-red
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-strict-only  裸词层不判
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-resource-unexempt  裸词层不看资源词
#     account-quota-green
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-records-only  射程缩到只有 records/
#     account-quota-code-green
#     account-quota-code-red
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-code-unscoped  射程里没有代码与治理文件：只判 records/、.claude/kb/ 与三方判决正文
#     account-quota-code-green
#     account-quota-code-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-notice-form-skipped  限额通知里 resets 后接月日与钟点、括号时区的那一形态不判
#     account-quota-code-green
#     account-quota-code-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-whole-tree  不限新加的行、整份比
#     account-quota-code-green
#     account-quota-code-red
#     account-quota-green
#     account-quota-red
#   GATE_TEXT_NOTATIONS_BREAK=account-quota-marker-ignored  不认放行标记
#     account-quota-code-green
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
#   GATE_TEXT_NOTATIONS_BREAK=dated-status-lag-unread  dated-status 不读滞后表（命中的一律判红，表写没写对、涨没涨都不判）
#     dated-status-green
#     dated-status-lag-growth-red
#     dated-status-lag-table-red
#   GATE_TEXT_NOTATIONS_BREAK=dated-status-growth-unjudged  dated-status 不拿滞后表与 gate 基准那一版比（滞后表涨了也不红）
#     dated-status-green
#     dated-status-lag-growth-red
#   GATE_TEXT_NOTATIONS_BREAK=task-and-session-numbers-unjudged  task-and-session-numbers 不判、记无对象可判
#     task-and-session-numbers-descriptions-green
#     task-and-session-numbers-descriptions-red
#     task-and-session-numbers-green
#     task-and-session-numbers-prompts-green
#     task-and-session-numbers-red
#     task-and-session-numbers-renamed-green
#     task-and-session-numbers-renamed-red
#   GATE_TEXT_NOTATIONS_BREAK=task-and-session-numbers-base-ignored  task-and-session-numbers 不取基准里的样子（写前当空串），基准里已有的违例行也报
#     task-and-session-numbers-green
#     task-and-session-numbers-renamed-green
#     task-and-session-numbers-renamed-red
#   GATE_TEXT_NOTATIONS_BREAK=task-and-session-numbers-renames-ignored  task-and-session-numbers 不认搬家、改名（基准只按此刻的路径取，改名过来的基准成了空串，里面已有的号也报）
#     task-and-session-numbers-renamed-green
#     task-and-session-numbers-renamed-red
#   GATE_TEXT_NOTATIONS_BREAK=task-and-session-numbers-prompts-judged  task-and-session-numbers 连 research/prompts/ 下的文件也判（冻结证据里新加的号也报）
#     task-and-session-numbers-prompts-green
#     task-and-session-numbers-red
#   COMMENT_HISTORY_CHECK_BREAK 认 scripts/gate/comment-history-check.py 文件头的开关（comment-history 一格的判法在那份脚本里，这一格起它时环境原样带进去；下面十四条列出样本能证红的，另十七条只在那份脚本自己的自证里复跑）：
#   COMMENT_HISTORY_CHECK_BREAK=marker-ignored  comment-history 不认 history-kept 标记：标记行照样按命中算
#     comment-history-green
#     comment-history-marker-short-red
#   COMMENT_HISTORY_CHECK_BREAK=marker-reason-unchecked  标记理由不核长度，写什么都放过
#     comment-history-marker-short-red
#   COMMENT_HISTORY_CHECK_BREAK=docstrings-skipped  不取 Python 文档串
#     comment-history-python-docstring-red
#   COMMENT_HISTORY_CHECK_BREAK=path-dates-flagged  日期不核前后字符，路径与文件名里的日期也判
#     comment-history-green
#   COMMENT_HISTORY_CHECK_BREAK=prefix-compounds-flagged  不核时间词前后的字：以前缀、除以前者、所以前面、可以前置、以前面那一份这几种「以」是前一个词的一部分、或「前」开头的是后一个词的写法也判
#     comment-history-before-face-red
#     comment-history-before-nontime-green
#     comment-history-before-time-red
#     comment-history-past-red
#     comment-history-prefix-green
#   COMMENT_HISTORY_CHECK_BREAK=face-exception-widened  时间词后面紧跟「面」就放过，不看「面」后面是不是「那」「这」「的」「带」：时间词后面接面临、面向、面对这类真讲过去的句子也放过
#     comment-history-before-face-red
#     comment-history-before-time-red
#   COMMENT_HISTORY_CHECK_BREAK=strings-as-comments  字符串里的注释符也当注释
#     comment-history-strings-green
#   COMMENT_HISTORY_CHECK_BREAK=lag-growth-unjudged  comment-history 的滞后表不与 gate 基准那一版比（滞后表涨了也不红）
#     comment-history-lag-green
#     comment-history-lag-growth-red
#   COMMENT_HISTORY_CHECK_BREAK=lag-slack-allowed  命中少于登记的不判红
#     comment-history-lag-slack-red
#   COMMENT_HISTORY_CHECK_BREAK=lag-table-form-unchecked  滞后表的行不核写法，只取能取到的路径与数
#     comment-history-lag-table-red
#   COMMENT_HISTORY_CHECK_BREAK=exempt-table-form-unchecked  豁免表的行不核写法，路径在射程里就放过
#     comment-history-exempt-table-red
#   COMMENT_HISTORY_CHECK_BREAK=exempt-table-ignored  不读豁免表
#     comment-history-green
#     comment-history-exempt-table-red
#   COMMENT_HISTORY_CHECK_BREAK=shebang-unscoped  没有后缀的 shebang 脚本不在射程里
#     comment-history-shebang-red
#   COMMENT_HISTORY_CHECK_BREAK=readme-fences-judged  README 的围栏代码块里也判
#     comment-history-green
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
#   LIB_FORBIDDEN_NOTATIONS_BREAK=numbered-pairs-whole-line  编号对的上下文规则按整行读（同一行里有「文件:行号」或越界冒号对，整行的冒号对都放过）
#     clock-times-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=mountinfo-device-numbers  mountinfo 一行里的主:次设备号不放过（当钟点判）
#     clock-times-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK=task-and-session-numbers  关掉任务号、会话短号与批号这一形态（task-and-session-numbers 一格判的就是它）
#     task-and-session-numbers-descriptions-red
#     task-and-session-numbers-green
#     task-and-session-numbers-red
#     task-and-session-numbers-renamed-green
#     task-and-session-numbers-renamed-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=session-descriptions  关掉「拿描述指别的会话」那一条判据行
#     task-and-session-numbers-descriptions-red
#   LIB_FORBIDDEN_NOTATIONS_BREAK=session-descriptions-loose  「拿描述指别的会话」那一条判据行换成宽的认法：带「一路」的、mainagent( 与 subagent( 一律算
#     task-and-session-numbers-descriptions-green
#     task-and-session-numbers-descriptions-red
#   同一个变量的 first-hit-only（只交最靠前的一处）在这一道的样本上不判错，证红的是 .claude/hooks/write-guard.sh --selftest。
#   下面三个变量名不以 _BREAK 收尾，值写形态键，照同样写法列出判错的样本：
#   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=clock-times  那一形态判据行的上下文规则不生效，命中一律算违例
#     clock-times-green
#   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=session-descriptions  值也可以写判据行键：「拿描述指别的会话」那一条的上下文规则不生效，反引号里的也算违例
#     task-and-session-numbers-descriptions-green
#     task-and-session-numbers-descriptions-red
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
#     comment-history-before-face-red、comment-history-before-time-red、comment-history-exempt-table-red、comment-history-lag-count-red、comment-history-lag-growth-red、comment-history-lag-slack-red、comment-history-lag-table-red
#     comment-history-marker-short-red、comment-history-past-red、comment-history-python-docstring-red、comment-history-readme-red、comment-history-rust-doc-red
#     comment-history-rust-line-date-red、comment-history-shebang-red、comment-history-shell-red、comment-history-wgsl-red
#     account-quota-code-red、account-quota-red、clock-times-mountinfo-red、clock-times-red、date-once-history-translations-red
#     date-once-red、date-once-thread-default-red、date-once-unjudgeable-red、dated-status-lag-growth-red、dated-status-lag-table-red、dated-status-red
#     out-of-project-names-allowed-red、out-of-project-names-deep-red
#     out-of-project-names-forms-red、out-of-project-names-private-red、out-of-project-names-red、out-of-project-names-root-red
#     out-of-project-names-table-red、prime-marks-red、private-values-big-untracked-red、private-values-clock-snippet-red
#     private-values-cross-segment-red、private-values-date-once-path-red、private-values-filename-and-content-red、private-values-filename-only-red
#     private-values-red、private-values-shared-root-address-red、private-values-siblings-red、private-values-symlink-dangling-red、private-values-symlink-mount-red
#     private-values-too-big-red、private-values-unreadable-red、session-names-red、task-and-session-numbers-descriptions-red、task-and-session-numbers-red
#     task-and-session-numbers-renamed-red
#     trilingual-changed-together-git-error-red、trilingual-changed-together-red、trilingual-files-no-registry-red、trilingual-files-red
#     trilingual-files-registry-red、trilingual-glossary-longer-word-red、trilingual-glossary-longer-words-table-red、trilingual-glossary-marker-red
#     trilingual-glossary-missing-red、trilingual-glossary-not-table-red、trilingual-glossary-red、trilingual-glossary-renames-columns-red
#     trilingual-glossary-table-red、trilingual-omission-comment-open-in-code-red、trilingual-omission-english-red、trilingual-omission-japanese-red
#     trilingual-structure-link-target-red、trilingual-structure-red、trilingual-untranslated-comment-open-in-code-red、trilingual-untranslated-english-red
#     trilingual-untranslated-exempt-table-red、trilingual-untranslated-identical-red、trilingual-untranslated-japanese-red、utf8-valid-chunk-red
#     utf8-valid-red、utf8-valid-staged-red
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本把登记的格都跑
#     comment-history-before-face-red、comment-history-before-nontime-green、comment-history-before-time-red、comment-history-exempt-table-red、comment-history-green、comment-history-lag-count-red、comment-history-lag-green、comment-history-lag-growth-red
#     comment-history-lag-slack-red、comment-history-lag-table-red、comment-history-marker-short-red、comment-history-past-red、comment-history-prefix-green、comment-history-python-docstring-red、comment-history-readme-red
#     comment-history-rust-doc-red、comment-history-rust-line-date-red、comment-history-shebang-red、comment-history-shell-red、comment-history-strings-green
#     comment-history-wgsl-red
#     account-quota-code-green、account-quota-code-red、account-quota-green、account-quota-red、clock-times-green、clock-times-mountinfo-red
#     clock-times-red、date-once-green、date-once-history-translations-green、date-once-history-translations-red
#     date-once-red、date-once-thread-default-red、date-once-unjudgeable-red、dated-status-green、dated-status-lag-growth-red、dated-status-lag-table-red、dated-status-red
#     out-of-project-names-allowed-red、out-of-project-names-deep-green
#     out-of-project-names-deep-red、out-of-project-names-forms-green、out-of-project-names-forms-red、out-of-project-names-green
#     out-of-project-names-no-private-list、out-of-project-names-private-red、out-of-project-names-red、out-of-project-names-root-red
#     out-of-project-names-table-red、prime-marks-green、prime-marks-red、private-values-big-untracked-red
#     private-values-clock-snippet-red、private-values-cross-segment-red、private-values-date-once-path-red、private-values-filename-and-content-red
#     private-values-filename-only-red、private-values-green、private-values-no-config、private-values-nul-head-green
#     private-values-red、private-values-shared-root-address-red、private-values-shared-root-green、private-values-siblings-red、private-values-symlink-dangling-red、private-values-symlink-mount-red
#     private-values-too-big-red、private-values-unreadable-red、session-names-green、session-names-red
#     task-and-session-numbers-descriptions-green、task-and-session-numbers-descriptions-red、task-and-session-numbers-green、task-and-session-numbers-prompts-green、task-and-session-numbers-red
#     task-and-session-numbers-renamed-green、task-and-session-numbers-renamed-red
#     trilingual-changed-together-git-error-red、trilingual-changed-together-green、trilingual-changed-together-red、trilingual-changed-together-translation-only-green、trilingual-changed-together-unrun
#     trilingual-files-green、trilingual-files-no-registry-red、trilingual-files-red、trilingual-files-registry-red、trilingual-glossary-green
#     trilingual-glossary-longer-word-green、trilingual-glossary-longer-word-red、trilingual-glossary-longer-words-table-red、trilingual-glossary-marker-red
#     trilingual-glossary-missing-red、trilingual-glossary-not-table-red、trilingual-glossary-red、trilingual-glossary-renames-columns-red
#     trilingual-glossary-table-red、trilingual-omission-comment-open-in-code-red、trilingual-omission-english-red、trilingual-omission-green、trilingual-omission-japanese-red
#     trilingual-structure-green、trilingual-structure-link-target-red、trilingual-structure-red
#     trilingual-untranslated-comment-open-in-code-red、trilingual-untranslated-english-red、trilingual-untranslated-exempt-table-red、trilingual-untranslated-green、trilingual-untranslated-identical-green
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
#
# ── dated-status ──
# 带日期的现状快照会停在写下那一天：之后状态变了没人改它（规则 .claude/singlefs-ai-sop/rules/kb-discipline.md「带日期的现状快照也是历史，会停在那一天」）。
# 对象：登记的 kb 目录（上游 .claude/singlefs-ai-sop/scripts/lib.sh 的 kb_directories 读 .claude/kb-directories，与上游 doc-lint.sh 同一份判法；登记写坏了判红；没有登记表取 .claude/kb）
#   下（含子目录）的中文 .md：不含 .en.md、.ja.md 译文（kb 的译文另起了名的也以这两个后缀收尾，lib-trilingual.py 的 language_paths），不含 decisions-history.md、experiments-history.md、INDEX.md。
#   每份只看「## 历史版本」那一行之前，历史节里的旧样子不判。
# 判什么：一行里有「现状（YYYY-MM-DD」（现状与括号之间可夹 **，括号可以是半角）或「YYYY-MM-DD 现状」（中间至多一个空格），逐行打「路径:行号  片段」判红；
#   「2026-09-17 跑了一次」这种日期跟着事件的句子不判。  history-kept: 例句里的日期是被测的数据形态
# 滞后表 .claude/gate.d/dated-status-lag.tsv（被判的根下）：一行「<路径><TAB><行内锚><TAB># <为什么还没改>」，锚是那一行里至少 12 个字的原文，理由至少 4 个字；
#   命中的行落在某一条登记里（同一路径、那一行含锚）就不红，成功句报「滞后 N 行」。表不在就是没有登记，命中的一律红。
#   表自己判红的：列数不是三列、锚不到 12 个字、理由没写或不到 4 个字、同一条登记两次、登记的路径不在、锚在那份文件里命中的不是恰好一行；
#   登记里指的那一行已经不是带日期的现状句（改好了没删行）只打一行「!」提醒删掉，不判红。
#   只缩不涨：与 gate 基准（scripts/gate/changed-paths.sh 的 gate_diff_base gate）那一版按（路径、锚）比，基准那一版里没有的行判红（锚改了也算涨）；
#   基准那一版经 lib-lag-baseline.py 的 baseline_rows 取（登记的搬迁照换算）：基准里没有这张表（初次登记）不比，git 取不到判红，被判的根不在 git 仓里不比、成功句写明没比。
# 一份中文 kb 页都没扫到：退 77。红行路径与片段经 private-values 那一份打码（PRIVATE_VALUES_LIBRARY 的 shown_path、shown_text、snippet_bounds）。
# 不并进同一道的 date-once 格：那一格只判这次改动碰到的记录与 kb 页、判的是正文重复所属标题的日期与同日第二个小节，交 doc-consolidate.py 干跑；这一格判全部 kb 中文页此刻的样子，
#   日期与标题无关、只认「现状」挨着日期的两种形态，还有滞后表。不并进 clock-times 格：那一格判全仓的钟点、时区词与时间戳，日期本身不判。
# 弄坏开关 dated-status-lag-unread、dated-status-growth-unjudged 只在样本根上认（stage_cells_run 开头那一判拦真仓）。
# 管不到的：不带「现状」二字的带日期现状句（「截至 2026-09-17 已落地」「2026-09-17 时三条全过」这类，要靠人把日期删掉再读）；英语、日语译文里的同类句子；  history-kept: 例句里的日期是被测的数据形态
#   滞后表登记了之后那一行事实又变了（锚还在就照放过，靠写回员现查改）。
#
# ── comment-history ──
# 代码与脚本的注释、文件头与 README 写成了改动经过，没有别的门禁管（判据：.claude/agent-common.md「代码与脚本的注释、说明只写现状，不写历史」那一条）。
# 判法整份在 scripts/gate/comment-history-check.py，这一格只调它、退出码原样交回（不另写一份）：词表、射程、取哪些文本、特殊缘由标记、豁免表与滞后表的写法、成功句与弄坏开关都写在那份脚本的文件头；
#   python3 scripts/gate/comment-history-check.py --list <根> 逐行打全部命中，--lag-table <根> 按此刻命中现算滞后表。
# 射程：被判的根下 crates/、scripts/、task-server/、prune-server/、.claude/hooks/、.claude/gate.d/、.claude/scripts/ 里 git 跟踪的与未跟踪而未忽略的 .rs、.wgsl、.py、.sh、
#   首行是 sh、bash、python shebang 的无后缀脚本与 README.md（根不是 git 仓时按目录遍历同一射程）；.claude/gate.d/fixtures/ 与 target/ 不判；
#   取注释、Python 文档串与 README 全文（围栏代码块不判），字符串里的 // 与 # 不算注释。
# 特殊缘由：那一行的注释里写 history-kept 标记加至少 8 个字的理由，那一行放过；理由不够长的判红并点名。
# 豁免表 .claude/gate.d/comment-history-exempt.tsv：一行「<路径><TAB># <理由，至少 8 个字>」，整份文件不判，只收主题就是历史词表的文件，存量不往里放；
#   滞后表 .claude/gate.d/comment-history-lag.tsv：一行「<路径><TAB><命中行数><TAB># <理由，至少 4 个字>」，文件命中比登记多、比登记少、不在表里的文件有命中，都判红；
#   与 gate 基准那一版比只缩不涨（基准里没有的行、数比基准大都判红；基准里整张表还不存在的那一次只判写法、不判涨）；被判的根不是 git 仓时不比、成功句写明没比。
# 成功句报扫了多少份文件、多少行注释、命中几行，其中按标记放过几行、滞后几行；射程里一份文件都没有退 77，不判绿。
# 弄坏开关 COMMENT_HISTORY_CHECK_BREAK 只在样本根上认（stage_cells_run 开头那一判拦真仓，那份脚本自己也只在样本根与自证里认），开关表在上面「弄坏开关」那一段。
# 管不到的：字符串、测试输入里写的历史词；不在词表里的历史说法（靠 review）；注释里写在围栏之外的缩进代码块。
# 样本：fixtures/doc-text.sh/ 下一百零三份，每份样本根放 .gate-cells 只点名它判的那一格（trilingual-* 三十份的现场都由 setup.sh 现造，日期取真实发生过的日子；private-values-siblings-red 与 private-values-unreadable-red 点名四格）：
#   date-once-red       setup.sh 建 git 仓；这一轮往记录正文加一行带文件名日期的话、往实验页加第二个同日 `### ` 小节，逐份点名判红
#   date-once-green     同一基准；加的行只带别的日期、加的小节是另一天，判绿
#   date-once-history-translations-green  setup.sh 建 git 仓；两份变更史与 INDEX.md 连同英日译文九份，这一轮各加同一天在另一节的一块（INDEX 是同一天的第二个小节）与一行重复所属日期的话，
#                       另有一份实验页加了另一天的小节：只判实验页那一份，判绿
#   date-once-history-translations-red    setup.sh 建 git 仓；决策变更史的英语译文与一份实验页的英语译文各加同一天的第二块：只点名实验页译文（1 份里 1 份），判红
#                       （排除扩到全部 .en.md 时退 77，退回只排除中文那三份时变更史译文也被点名）
#   date-once-unjudgeable-red  setup.sh 建 git 仓；这一轮加一份带不合法 UTF-8 字节的记录（doc-consolidate.py 读不了它、退 1）与一份干净的记录：
#                       点名那一份「干跑退 1，这一份没判」，「2 份记录与 kb 页里 1 份干跑出错」，判红
#   date-once-thread-default-red  setup.sh 建 git 仓；这一轮加一份干净的记录；样本根放 .sample-thread-cap-invalid（date_once_jobs 在样本根里认它：按 SINGLEFS_THREAD_CAP 设成了不是正整数的值取，
#                       造出取不出每台默认重活线程数的样子）：点名「取不了每台默认重活线程数」，一份都没判，判红；弄坏开关 date-once-jobs-unregistered-passes 下退回一路、照常判，退 0
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
#   trilingual-changed-together-git-error-red  setup.sh 建 git 仓并把 git 配置里的 diff.renames 写成 git 认不得的值，取改动路径的 git diff 一律失败：
#                       红行之前有 gate_changed_paths 写的哪一条 git、退几与 git 的报错原文（预取在父进程里，原文经 text_prefetched_error 带进格的输出）
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
#   trilingual-omission-comment-open-in-code-red  中文、英语、日语三份第一段的反引号里都带一个没有闭合的 <!--；英语那份后面一段压成一句（比值 0.13），点名那一段判红
#   trilingual-untranslated-comment-open-in-code-red  英语那份一行反引号里带没有闭合的 <!--，同一行后面留着 22 个汉字的中文原句，点名那一行判红
#   trilingual-omission-english-red    英语那份「背景」一节漏了中文第二段（中文 3 段、英语 2 段）、「回看」表 D4 那一行漏了其中一个日期的那一块（比值 0.41，门槛那一档）、
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
#                       判红，点名链接与键名 BUILD_ROOT_BY_HOST，输出里不出现假仓路径
#   private-values-symlink-mount-red  同一份假配置；setup.sh 在样本里被 git 忽略的 mnt/ 下建以假第二台主机名命名的挂载点（目标存在、内容干净），放一个指到它的未跟踪符号链接：
#                       判红，点名链接与键名 HOSTS、QUIESCE_STOP_COMMAND，输出里不出现假主机名
#   private-values-unreadable-red  同一份假配置；setup.sh 造从第二台 tar 过来的产物目录（假仓路径整段成了本地路径的一部分），里面一份权限 000 的 .out，另有一份干净的新日志；
#                       点名四格：private-values 判红、列成「读不了」一类、路径打码，没有回溯；prime-marks、clock-times、session-names 绿，「读不了」清单里路径打码
#                       （以 root 跑时权限 000 也读得到，这一份判不出，不以 root 跑样本档）
#   private-values-date-once-path-red  同一份假配置；setup.sh 放一份文件名带假第二台主机名、正文一行重复标题日期的记录（不建 git 仓）：date-once 判红，
#                       红行里的路径只剩「records/〔文件名含私有值〕」，输出里不出现假主机名
#   private-values-clock-snippet-red  同一份假配置；setup.sh 建 git 仓（expect、setup.sh 与假配置被 .gitignore 挡在扫描之外），记录里两行时间戳的前 20 个字
#                       落在假仓路径、假第二台主机名中间：clock-times 判红，两行片段都从值的开头起、整值打成「〔私有值〕」，输出里不出现半截的假值
#   private-values-shared-root-green  另一份假配置：HOSTS 是两个假 IPv4 地址，NFS_SHARED_ROOTS 两段假共享根，SHARED_DATA_ROOT 是其中一段；setup.sh 建 git 仓提交一版干净的记录，
#                       这一轮往记录加四行：数据根本身、数据根下面的路径、另一段共享根各一行，一行没有私有值；判绿，报对着 2 个长词判了（两个地址）、共享根下的路径 2 个
#   private-values-shared-root-address-red  同一份假配置；这一轮往记录加三行：数据根一行（不报）、第二台的假地址一行、一行没有私有值：判红，只点名地址那一行与键名 HOSTS（「新加的 1 行」）
#   account-quota-red   setup.sh 建 git 仓，基准里一份记录（另带一行旧话「撞了限额」，不报）、一份 kb 页、一份判决正文都干净；这一轮往三份里加写了账号切换与用量限额的行， account-quota:allow 词表与样本说明
#                       另放一份未跟踪的新记录，词表里每一条各一行（切换账号、换了账号、另一个账号、switched the account、撞了周额度、撞会话限流、撞限额、额度恢复、通知原文、限额窗口、用量限额、额度用完、被额度打断）， account-quota:allow 词表与样本说明
#                       外加一行写了放行标记却没写理由的；射程之外的两份未跟踪文件（_ 开头的附录、门禁样本目录下的文件）带同样的话，不报；逐处点名「文件:行」与判据名判红，共 14 行
#   account-quota-green 同一基准；这一轮加的行只有显卡额度、GPU_CARDS_BY_PRIORITY 的额度、内存上限与内存额度、空间配额、欠账号、「换号」、「中途停下没交回」，命中处附近有资源词的裸词命中三行
#                       （撞额度：账上 8G、显卡额度重置、显存用量上限），带理由的放行标记一行；射程之外的未跟踪文件（附录、snapshot 副本、门禁样本目录下的文件）带「切换账号」「撞了周额度」，不报，判绿 account-quota:allow 词表与样本说明
#   account-quota-code-red   setup.sh 建 git 仓，基准里钩子、脚本、门禁、规则、定义、主 agent 定义、共用约束、task-server、prune-server 各一份都干净（钩子里另有一行旧的解析限额通知的正则，不报）；
#                       这一轮往钩子里加两行（解析限额通知的正则、通知原样），往另外八份里各加一行，射程之外的两份未跟踪文件（crates/ 下的源码、门禁样本目录下的文件）带同样的话；
#                       逐处点名「文件:行」与判据名判红，共 10 行、9 份文件，不点名射程之外的两份
#   account-quota-code-green 同一基准；这一轮钩子里解析限额通知的正则一行带了理由的放行标记、另一行干净，脚本里加一行显卡额度与内存上限，射程之外的两份未跟踪文件带同样的话：
#                       判绿，报查了 2 份文本文件的 3 行、放行标记 1 行
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
#   dated-status-red    样本根照真仓形态放 .claude/kb-directories（.claude/kb 与 crates/sample-checker/invariants 两行）与 .claude/history-carriers，不建 git 仓、没有滞后表：
#                       里程碑页正文「**现状（2026-09-17 落地）**：」、实验页正文「2026-09-14 现状：」、不变量页表格行里「**2026-09-23 现状**（」三行逐行点名判红；  history-kept: 引的是样本输入里的日期
#                       不判的：实验页「## 历史版本」里的同形句、事件句「2026-09-17 跑了一次」、不带日期的「现状：」、英语译文、实验变更史与 INDEX.md 里的同形句、records/ 下的同形句（不在登记目录里）  history-kept: 引的是样本输入里的日期
#   dated-status-green  同一套页与登记目录，setup.sh 建 git 仓：基准里滞后表登记那三行；这一轮把里程碑那一行改成现值、删掉滞后表里它那一行（只缩），
#                       判绿并报滞后 2 行、与基准比没涨；历史节里的同形句与事件句照旧不判
#   dated-status-lag-growth-red  setup.sh 建 git 仓：基准里里程碑页两行现状句、滞后表只登记一行；这一轮往滞后表加另一行：点名新加的那一行（滞后表比基准多 1 行）判红，两行命中都算滞后
#   dated-status-lag-table-red   不建 git 仓：一份页一行现状句，滞后表一行写对、另七行各错一处（锚在文件里找不到、锚命中两行、登记的路径不在、锚不到 12 个字、只有两列、理由不到 4 个字、
#                       与写对的那一行重复登记）：逐行点名行号判红，写对的那一行照放过
#   comment-history-green  setup.sh 造豁免表，不建 git 仓：射程里各类文件（.rs、.wgsl、.py、.sh、没有后缀的 sh 脚本、README.md）都只写现状，注释里提到带日期的路径与文件名（路径里的日期是名字不判），
#                       三行带够长标记的放过（.rs 注释、.sh 行尾注释、README 正文），README 围栏里写着词表里的词不判，豁免表里那份整份只讲这些词的 .py 不判；
#                       射程外写着词表里的词的（docs/ 下的 .rs、根下 README.md、.txt、没有 shebang 的无后缀文件）不判：判绿，报命中 3 行、按标记放过 3 行
#   comment-history-strings-green  不建 git 仓：.rs、.py、.sh 里字符串、原始字符串、字符字面量、三引号字符串里的 // 与 # 后面跟着词表里的词，shell 里的 ${#x}、$#、${x#y} 的 #，都不是注释：判绿，报命中 0 行
#   comment-history-prefix-green  不建 git 仓：.rs 注释里写「续扫起点要以前缀起头」「后者除以前者向上取整」「所以前面已发的写」「除以前一项的值」，都是「以」「除以」「所以」接前缀、前者、前面（或「除以」接「前……」），不是讲过去：判绿，报命中 0 行
#   comment-history-lag-green  setup.sh 建 git 仓：基准里两份 .rs 各有命中、滞后表登记两行；这一轮把第二份改净、删掉它那一行（只缩）：判绿，报滞后 2 行、与基准比没涨（基准 2 行，此刻 1 行）
#   comment-history-rust-line-date-red  .rs 行注释里写了带日期的一句：点名「文件:行」与那个日期，判红
#   comment-history-rust-doc-red  .rs 的 //! 与 /// 文档注释、块注释里各写了一处词表里的词：逐行点名（共 3 行），判红
#   comment-history-before-face-red  .sh 注释里四行：「所以前面已发的写」「除以前一项的值」两行（「所以」「除以」接前面、前一项，不算）、时间词后面接「面临」「面向」的两行（「面临」「面向」是词，时间词要算）：只点名后两行（共 2 行），判红（时间词后面紧跟「面」就放过时判绿）
#   comment-history-before-nontime-green  .sh 注释里六行：「两份结果不同时以前面那一份为准」「路径可以前面带 ./，归一之后再比」「键可以前置一段命名空间」「系数要乘以前一项再取整」「条件以前置条件为准」「取值以前面的那一份为准」，
#                       「以」是「可以」「乘以」的一部分、或「前面那」「前面带」「前置」「前面的」是后一个词，都不是讲过去：判绿，报命中 0 行
#   comment-history-before-time-red  .sh 注释里五行：「系数乘以前一项」「取值以前面的那一份为准」两行不是讲过去（「乘以」的一部分、「前面的」是词）、时间词要算的三行（后面接「面对」、后面接「一次」、前面是「半年」）：只点名后三行（共 3 行），判红
#                       （时间词后面紧跟「面」就放过时少判「面对」那一行，「以」「前」的构词不核时多判「乘以」「前面的」那两行）
#   comment-history-past-red  .sh 注释里一行是「键要以前缀起头」（「以」接前缀，不是讲过去，不算）、下一行是真讲过去的句子（词表里的时间词起头）：只点名真讲过去的那一行（共 1 行），判红
#   comment-history-python-docstring-red  .py 只有文档串里写了词表里的词（没有 # 注释）：点名文档串里那一行的行号，判红（不取文档串时判绿）
#   comment-history-shell-red  .sh 的注释写了谁定的那一形态（日期夹在中间）：点名命中的整段，判红
#   comment-history-shebang-red  没有后缀、首行是 python shebang 的脚本，# 注释里写了词表里的词：判红（脚本不在射程里时一份文件都没有，退 77）
#   comment-history-readme-red  README.md 正文写了按轮次点名的一句，围栏里是干净的：判红
#   comment-history-wgsl-red  .wgsl 行注释写了词表里的词：判红
#   comment-history-marker-short-red  .py 一行注释写了带日期的话、标记理由只有 3 个字：不放过，点名行号与理由，判红（标记不认时红在另一条上，标记理由不核时判绿）
#   comment-history-lag-growth-red  setup.sh 建 git 仓：基准里滞后表只登记一份 .rs；这一轮往表里加第二份：点名新加的那一行（滞后表比基准多 1 行）判红，两份的命中都算滞后
#   comment-history-lag-count-red  setup.sh 造滞后表，不建 git 仓：一份 .rs 命中 3 行而滞后表只登记 2 行，另一份命中 1 行没登记：两份各把全部命中列出（共 4 行），判红
#   comment-history-lag-slack-red  setup.sh 造滞后表，不建 git 仓：一份 .rs 命中 2 行而滞后表登记 3 行：点名改小到 2，判红
#   comment-history-lag-table-red  setup.sh 造滞后表，不建 git 仓：滞后表一行写对、另六行各错一处（重复登记、只有两列、数不是整数、理由不到 4 个字、路径不在、数是 0），逐行点名行号判红；
#                       两份因登记行写错没生效，它们的命中一并判红，写对的那一份照放过
#   comment-history-exempt-table-red  setup.sh 造豁免表，不建 git 仓：豁免表一行写对、另四行各错一处（重复登记、理由不到 8 个字、路径不在、三列），逐行点名行号判红；
#                       两份因登记行写错没生效，它们的命中一并判红，写对的那一份照放过
#
#   bash .claude/gate.d/doc-text.sh [项目根]                                     十七格都跑
#   bash .claude/gate.d/doc-text.sh --list                                       逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-text.sh --check <格名>[,<格名>…] [项目根]            只跑点名的格
#   bash .claude/gate.d/doc-text.sh --check out-of-project-names --all-lines [项目根]  回扫：全仓 git 跟踪的文本文件整份判，只报每份的命中处数（只读）
#
# 格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的几份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析与汇总。格的出路里写的「doc-text.sh 里这一格函数上方的注释」，就在下面对应的那份库里（格在哪份库按格名列在下面）：
# 库 lib/doc-text-notation-and-translation-cells.sh：prime-marks、clock-times、session-names、date-once、trilingual-files、trilingual-structure
# 库 lib/doc-text-notation-and-translation-cells.sh：trilingual-changed-together、trilingual-glossary、trilingual-untranslated、trilingual-omission
# 库 lib/doc-text-notation-and-translation-cells.sh：private-values、account-quota、task-and-session-numbers
# 库 lib/doc-text-content-and-history-cells.sh：out-of-project-names、utf8-valid、dated-status、comment-history
# 弄坏开关的读法在入口的赋值与格函数里，格函数在 lib/ 下的库里；开关表仍在上面文件头。下面每个开关变量在哪份库的代码里被读，按库列在这里：
# 库 lib/doc-text-notation-and-translation-cells.sh：LIB_FORBIDDEN_NOTATIONS_BREAK、LIB_TRILINGUAL_BREAK
# 入口的代码里也读的：GATE_TEXT_NOTATIONS_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：COMMENT_HISTORY_CHECK_BREAK
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ doc-text.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"
source "$STAGE_DIR/lib/stage-cells.sh"
LIBRARY="$STAGE_DIR/lib-forbidden-notations.py"
TRILINGUAL="$STAGE_DIR/lib-trilingual.py"
DOC_CONSOLIDATE="$REPO_ROOT/scripts/kb/doc-consolidate.py"
source "$REPO_ROOT/scripts/gate/changed-paths.sh"
text_notations_break="${GATE_TEXT_NOTATIONS_BREAK:-}"
# 阶段在父进程里预取的改动范围与整份 diff（文件末尾「预取」一段赋值；各格在自己的子 shell 里跑，取了也传不给别的格，所以只能在父进程里取、子 shell 继承）。调用方环境里的同名变量不认：这里先清空
text_prefetched_base="" text_prefetched_changed="" text_prefetched_status=1 text_prefetched_error="" text_diff_cache=""

# 私有值的取词与路径、行文片段的打码：private-values 一格与 prime-marks / clock-times / session-names 三格打印路径时共用（同一份，不各抄一份）。
# python 代码放进变量，各格的 python 用 exec 载入；取词的规则写在 private-values 一格的注释里（文件头「private-values」一段）。
PRIVATE_VALUES_LIBRARY="$(cat <<'PY_PRIVATE_VALUES'
import io, re, shlex, socket

FILE_NAME_MASK = "〔文件名含私有值〕"          # 红行里路径的哪一段含私有值，那一段打成这个，值不打印
VALUE_MASK = "〔私有值〕"                     # 行文片段里的私有值打成这个
SKIPPED_KEYS = {"SLICE_MEMORY_CAP_BY_HOST", "ENABLE_ACROSS_MACHINES", "ENABLE_GPU"}
BY_HOST_KEYS = {"CARGO_BIN_DIRECTORY_BY_HOST", "BUILD_ROOT_BY_HOST"}   # 每项 <地址>=<值>：只取值那一截（文件头「private-values」取哪些词）
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


SHARED_ROOTS_KEY = "NFS_SHARED_ROOTS"           # 两台同一绝对路径的共享根，冒号隔开；落在它某一段下的路径不是私有值
IPV4_ADDRESS = re.compile(r"[0-9]{1,3}(?:\.[0-9]{1,3}){3}")


def shared_roots(entries):
    """NFS_SHARED_ROOTS 的各段（绝对路径、去掉结尾的 /；空段与根目录 / 不算，免得把所有绝对路径都放过）；没写这个键就是空表。"""
    roots = []
    for key, value in entries:
        if key != SHARED_ROOTS_KEY:
            continue
        for word in split_words(value):
            for segment in word.split(":"):
                segment = segment.rstrip("/")
                if segment.startswith("/") and segment not in roots:
                    roots.append(segment)
    return roots


def under_shared_root(word, roots):
    """词是共享根里的路径：冒号隔开的每一段（只有一段的就是整词）都是某个共享根本身或在它下面（按整级目录比，/srv/data-root-other 不算在 /srv/data-root 下）。"""
    segments = [segment.rstrip("/") for segment in word.split(":")]
    return bool(roots) and all(any(segment == root or segment.startswith(root + "/") for root in roots) for segment in segments)


def load_private_targets(configuration_path, hostname_path, broken):
    """取私有词：(词表 [(键名, 词)], 不足门槛没取的词数 {键名: 个数}, 配置里取到的词数, 本机主机名那一句说明, 门槛字数, 共享根下没取的词数 {键名: 个数})；读不了配置抛 OSError。"""
    minimum_length = 4 if broken == "private-values-short-words" else 8   # 攻方量过：4–7 个字的词各命中 221–2001 份文件，8 个字以上的只命中两份真泄漏
    targets = []              # (键名, 词)
    short_words = {}          # 键名 → 不足门槛没取的词数
    shared_words = {}         # 键名 → 落在共享根下、不当私有值的词数
    configuration_text = open(configuration_path, encoding="utf-8", errors="replace").read()
    entries = []              # (键名, 值)：先读全，NFS_SHARED_ROOTS 写在哪一行都认
    for line in configuration_text.split("\n"):
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        if key.startswith("export "):
            key = key[len("export "):].strip()
        entries.append((key, value.strip()))
    roots = [] if broken == "private-values-shared-roots-judged" else shared_roots(entries)
    for key, value in entries:
        if key in SKIPPED_KEYS:
            continue
        for word in split_words(value):
            # 显卡先后与额度（<地址>:0:24G,<地址>:1:8G 这种）整串不当私有值，只取逗号、分号、冒号、@ 切开的各段（都短，实际取不到）；
            # 按台的键每项只取第一个 = 之后的值那一截与它切开的各段（弄坏开关 private-values-by-host-unsplit 下照别的键取整项）；别的键整词与切开的各段都取
            if key.startswith("GPU_CARDS"):
                pieces = set(re.split(r"[,;:@]", word))
            elif key in BY_HOST_KEYS and broken != "private-values-by-host-unsplit":
                host_value = word.split("=", 1)[1] if "=" in word else ""
                pieces = {host_value, *re.split(r"[,;@]", host_value)}
            else:
                pieces = {word, *re.split(r"[,;@]", word)}
            for piece in pieces:
                piece = piece.strip()
                # 纯数字与容量（60G、1.5GiB）不取；IPv4 地址（HOSTS 里的第二台）也是数字与点，照取，不当容量跳过
                numeric = piece.isdigit() or re.fullmatch(r"[0-9.]+[GMK]?i?B?", piece)
                if numeric and IPV4_ADDRESS.fullmatch(piece) and broken != "private-values-addresses-skipped":
                    numeric = None
                if not piece or piece in COMMON_WORDS or piece.startswith("-") or numeric:
                    continue
                if under_shared_root(piece, roots):
                    shared_words[key] = shared_words.get(key, 0) + 1
                elif len(piece) < minimum_length:
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
    return targets, short_words, configuration_word_count, hostname_note, minimum_length, shared_words


def shown_path(path, targets, broken):
    """红行里的路径：私有值按整串在整条路径里找（值自己带 `/` 跨了几段目录的也找得到，不逐段比），命中的那一段（从值开头所在的那一级到结尾所在的那一级，
    重叠的并成一段）打成 FILE_NAME_MASK，值不打印。没有词表（没有 multi-host.env）原样交回。"""
    if broken == "private-values-path-printed":      # 弄坏：路径原样打
        return path
    if broken == "private-values-path-by-segment":      # 弄坏：逐段比，值带 `/` 跨了几段时哪一段都不含整串，整条路径连值原样打出
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
    """date-once 的红行路径、utf8-valid 的红行经这里打码：arguments = [配置, 主机名, 弄坏开关, path 或 lines]，
    stdin 逐行打码打到 stdout（path：整行是一个路径；lines：shown_line）。没有配置、读不了配置时原样打。"""
    configuration_path, hostname_path, broken, mode = arguments
    targets = []
    if configuration_path and broken != "private-values-date-once-unmasked":
        try:
            targets = load_private_targets(configuration_path, hostname_path, broken)[0]
        except OSError:
            targets = []
    for line in stream_in.read().splitlines():
        stream_out.write((shown_path(line, targets, broken) if mode == "path" else shown_line(line, targets, broken)) + "\n")


ADDED_HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")


DIFF_HEADER_PREFIX = "diff --git a/"


DIFF_HEADER_PREFIX_LINE = "diff --git "      # 整份 diff 里一份文件那一段的头行都以它起头（认不认得出路径另由 diff_header_path 判）


def diff_header_path(header):
    """整份 diff 里一个文件那一段的头行 `diff --git a/<路径> b/<路径>` 的路径（--no-renames，两边同一路径，所以按长度对半切再核一遍）；
    路径带引号（含制表符、引号、反斜杠）或两半对不上的交 None。"""
    if not header.startswith(DIFF_HEADER_PREFIX):
        return None
    rest = header[len(DIFF_HEADER_PREFIX):]
    path = rest[:(len(rest) - 3) // 2]
    return path if rest == f"{path} b/{path}" else None


class DiffSegments:
    """整份预取 diff 按文件切开之后的表：只记每份文件那一段在预取文件里的字节区间，`get(路径)` 时才读那一段、解码、切行。
    整份 diff 可以有几百万行；一次解码成一个字符串再切成逐行的列表，内存是 diff 字节数的好几倍，而一格只用到改动范围里的那几份。"""

    def __init__(self, handle, spans, hunks_dropped):
        self.handle, self.spans, self.hunks_dropped = handle, spans, hunks_dropped

    def __bool__(self):
        return bool(self.spans)

    def __len__(self):
        return len(self.spans)

    def __iter__(self):
        return iter(self.spans)

    def __contains__(self, path):
        return path in self.spans

    def get(self, path, default=None):
        span = self.spans.get(path)
        if span is None:
            return default
        if self.hunks_dropped:
            return []
        self.handle.seek(span[0])
        return self.handle.read(span[1] - span[0]).decode("utf-8", "replace").split("\n")


def load_diff_segments(cache_path, base, broken):
    """阶段在父进程里预取的整份 `git diff -U0 --no-renames <基准>` 按文件切开：{路径: 这份文件那一段的行}（段的行与 `git diff -U0 <基准> -- <路径>` 的输出逐行相同）。
    预取文件的第一行是预取时的基准；cache_path 空、读不了、基准与这一格取的不同，交空表，added_lines 对每份文件退回逐份 git diff。头行认不出路径的那一段不收。
    交回的是 DiffSegments（按字节区间记，`get` 时才读）：逐行扫一遍预取文件只认头行，不把整份留在内存里；同一路径出现两段的取后一段。
    弄坏开关 diff-cache-hunks-dropped：收下的每份文件都说没有新加的行。"""
    if not cache_path:
        return {}
    try:
        handle = open(cache_path, "rb")
        recorded_base = handle.readline().decode("utf-8", "replace").rstrip("\n")
        if recorded_base != base:
            handle.close()
            return {}
        spans, current, current_start, offset = {}, None, 0, handle.tell()
        header_prefix = DIFF_HEADER_PREFIX_LINE.encode("utf-8")
        for raw in handle:
            if raw.startswith(header_prefix):
                if current is not None:
                    spans[current] = (current_start, offset - 1)        # 段尾不含下一段头行之前的那个换行：切行之后与整份切行时这一段的行逐行相同
                current = diff_header_path((raw[:-1] if raw.endswith(b"\n") else raw).decode("utf-8", "replace"))
                current_start = offset
            offset += len(raw)
        if current is not None:
            spans[current] = (current_start, offset)
    except OSError:
        return {}
    return DiffSegments(handle, spans, broken == "diff-cache-hunks-dropped")


def added_lines(path, whole_file, base, git, fail, shown_path, segments=None, whole_file_limit=None):
    """这份文件这一次新加的 (行号, 行文)：已跟踪的取相对基准 git diff -U0 的新加行（segments 里有这份文件的那一段就用它，没有就逐份起 git diff），
    未跟踪的（新文件整份都算新加）整份逐行读（whole_file_limit 给了字节数就只读前这么多）。
    private-values、account-quota、out-of-project-names 三格共用；git、fail、shown_path 由调用的格给（各格自己的 git 起法、红行写法与路径打码）。"""
    if not whole_file:
        diff_lines = segments.get(path) if segments else None
        if diff_lines is None:
            difference = git("diff", "-U0", "--no-color", "--no-ext-diff", "--no-renames", base, "--", path)
            if difference.returncode != 0:
                fail(f"取不到 {shown_path(path)} 相对基准 {base[:12]} 的改动（git diff 退 {difference.returncode}）", "看基准提交在不在、git 是不是可用；改动范围的基准只经 scripts/gate/changed-paths.sh 的 gate_diff_base 取。")
            diff_lines = difference.stdout.decode("utf-8", "replace").split("\n")
        number, in_hunk = 0, False
        for line in diff_lines:
            found = ADDED_HUNK.match(line)
            if found:
                number, in_hunk = int(found.group(1)), True
            elif in_hunk and line.startswith("+"):
                yield number, line[1:]
                number += 1
        return
    with open(path, "rb") as handle:
        source = io.BytesIO(handle.read(whole_file_limit)) if whole_file_limit else handle
        for number, raw in enumerate(source, 1):
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
  "会话号不写进仓：标签式的「（会话 <号>，<日期>）」只留日期，当主语用的别的会话写成 mainagent(A)、mainagent(B)，子 agent 写成 subagent(A)、subagent(B)（同一份文字里按出场次序编字母）；代码真要用的那一行写「# session-names:allow <理由>」"
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
stage_cell account-quota cell_account_quota "这次改动新加到 records/、.claude/kb/、三方判决正文，与钩子、脚本、门禁、规则、定义、task-server、prune-server（样本除外）的行里不写账号与额度方面的运营细节，也不写解析限额通知的正则（词表在 cell_account_quota；显卡额度、内存上限、空间配额不在内）" \
  "删掉这句；子 agent 因此停下的，只写「中途停下没交回」，不写额度与账号方面的细节；代码里不写按限额通知挑模型、拦派发的东西（解析通知的正则、按通知拦派发的放行行）；用户原话里带这些词、要原样留的，那一行末尾写「account-quota:allow <理由>」（理由至少 4 个字，代码里写在注释里）"
stage_cell out-of-project-names cell_out_of_project_names "这次改动新加的行里不写不归本项目的目录名（本机家目录、仓的上一级目录与第二台私有名单里的条目名，只在像目录的上下文里判）与根下系统目录的占用；放行表 .claude/out-of-project-paths 每一条要有 git 跟踪的脚本引到" \
  "改成不指名的说法（例「根分区其余不归 singlefs」），要说空间只说本项目自己的目录与总的可用量；singlefs 自己读的仓外路径登记进放行表 .claude/out-of-project-paths（一行「<路径>  # 理由」，理由至少 8 个字，要有 git 跟踪的 .sh、.py、.rs 字面引到它），只放行那条路径本身"
stage_cell utf8-valid cell_utf8_valid "这次改动（暂存区或工作区相对基准）里的文本文件是合法 UTF-8：非法的点名文件、工作区还是暂存区、第一处坏字节的偏移" \
  "照红行的文件、版本（工作区或暂存区）、字节偏移和行号，把那一处按字符定点改回来（Edit 或 scripts/kb/replace-once.py；shell 的 paste -d、cut -c、head -c 这类按字节切的命令会把多字节字符切坏，改文本别用）；暂存区里坏而工作区已修好的，git add 重新暂存；确实是二进制的，补进 cell_utf8_valid 的后缀名单"
stage_cell dated-status cell_dated_status "登记的 kb 目录里的中文页在「## 历史版本」之前不写带日期的现状句（「现状（日期」与「日期 现状」两种）；滞后表 dated-status-lag.tsv 登记的放过，滞后表写对了、与 gate 基准那一版比只缩不涨" \
  "删掉日期、把事实现查后改成现值；那一天的样子要留就进 ## 历史版本（登记在 .claude/history-carriers 的文件）；存量还没改完的照 .claude/gate.d/dated-status-lag.tsv 的写法登记（只缩不涨，新写的句子不许往里加），改好一行就删它那一行"
stage_cell comment-history cell_comment_history "代码与脚本的注释、文件头、文档串与 README 里不写历史（日期、出处、改动经过）；特殊缘由在那一行写 history-kept 标记，存量登记在滞后表 comment-history-lag.tsv，滞后表写对了、与 gate 基准那一版比只缩不涨" \
  "改成只写现状：删掉日期、出处与改动经过，要指依据写现行登记位的编号与名字；历史进 commit message、.claude/kb/decisions-history.md、experiments-history.md 或 records/（.claude/agent-common.md「代码与脚本的注释、说明只写现状，不写历史」）；特殊缘由在那一行注释里写 history-kept: <理由，至少 8 个字>；存量改不完的照 .claude/gate.d/comment-history-lag.tsv 的写法登记（只缩不涨，新写的不许往里加）；逐行看命中：python3 scripts/gate/comment-history-check.py --list <根>"
stage_cell task-and-session-numbers cell_task_and_session_numbers "这次改动里的 .md、.tsv、.txt（research/prompts/ 下的冻结证据除外）相对基准多出来的行（新加的行与因围栏、表头改动而成了违例的行）不写任务号、括号里的会话短号与批号（判据是 lib-forbidden-notations.py 的 task-and-session-numbers 那一形态，基准里已有的不报）" \
  "照红行改：任务号换成交给服务的那条命令、结局行的 type= 与 exit= 那一段或日志里那一行的内容（不抄日志路径），会话短号换成谁在改哪个文件，批号换成那一批做了什么；照写号的只有判决里行首的「A/B 读数：任务 <号>」、records/ 下以「长跑拆分：」起头的一行与 records/ 下表头是「批次」的那一列（出路全文在 lib-forbidden-notations.py 那一形态的 remedy）"
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
# 根写成相对路径时换成绝对路径：各格里还要 cd "$ROOT"，此刻已在根里，相对路径再 cd 一次就进不去（任务 3316：--check private-values <样本的相对路径>）
ROOT="$PWD"

# 按格组 source 下面几份库；它们读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取
source "$STAGE_DIR/lib/doc-text-notation-and-translation-cells.sh"
source "$STAGE_DIR/lib/doc-text-content-and-history-cells.sh"

# ── 预取：改动范围与整份 diff 在父进程里按选中的格取一次 ──
# 用改动范围的七格（date-once、trilingual-changed-together、private-values、account-quota、out-of-project-names、utf8-valid、task-and-session-numbers）共用同一份改动路径；
# 读新加行的三格（private-values、account-quota、out-of-project-names）另共用一份整份 git diff -U0（预取文件第一行是基准，其余是 diff；各格切出自己那份文件的一段，切不出的文件退回逐份 git diff）。
# 取不到的不在这里判红：状态留在 text_prefetched_status 与空的 text_diff_cache 里，用它的格照样拿到失败或退回逐份取，各自判红、各报各的出路。
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  text_prefetch_changed=0 text_prefetch_diff=0
  for text_prefetch_cell in "${STAGE_CELLS_SELECTED[@]}"; do
    case "$text_prefetch_cell" in
      date-once|trilingual-changed-together|private-values|account-quota|out-of-project-names|utf8-valid|task-and-session-numbers) text_prefetch_changed=1 ;;
    esac
    case "$text_prefetch_cell" in
      private-values|account-quota|out-of-project-names) text_prefetch_diff=1 ;;
    esac
  done
  if ((text_prefetch_changed)); then
    text_prefetched_base="$(gate_diff_base gate)"
    # 父进程里取的：gate_changed_paths 写到标准错误的报错（哪一条 git、退几、git 的原文）各格的子 shell 里没有，接进 text_prefetched_error，格里取不到时由 text_changed_paths 打出来
    text_prefetch_error_file="$(mktemp "${TMPDIR:-/tmp}/text-prefetch-error.XXXXXX")" || text_prefetch_error_file=""
    if text_prefetched_changed="$(gate_changed_paths "$text_prefetched_base" untracked 2>"${text_prefetch_error_file:-/dev/stderr}")"; then
      text_prefetched_status=0
    else
      text_prefetched_status=1
      [[ -z "$text_prefetch_error_file" ]] || text_prefetched_error="$(cat -- "$text_prefetch_error_file")"
    fi
    [[ -z "$text_prefetch_error_file" ]] || rm -f -- "$text_prefetch_error_file"
  fi
  if ((text_prefetch_diff && text_prefetched_status == 0 && ! out_of_project_all_lines)) && [[ -n "$text_prefetched_base" ]]; then
    if text_diff_cache="$(mktemp "${TMPDIR:-/tmp}/text-diff-cache.XXXXXX")"; then
      trap 'rm -f "$text_diff_cache"' EXIT
      if ! { printf '%s\n' "$text_prefetched_base"; git -c core.quotepath=false diff -U0 --no-color --no-ext-diff --no-renames --src-prefix=a/ --dst-prefix=b/ "$text_prefetched_base"; } > "$text_diff_cache"; then
        rm -f "$text_diff_cache"; text_diff_cache=""
      fi
    else
      text_diff_cache=""
    fi
  fi
fi

# 这一道的格互不依赖、都只读被判的树：并行跑（.claude/gate.d/lib/stage-cells.sh 文件头「格并行」）
# 这一道的格不并行（不设 STAGE_CELLS_JOBS 的缺省）：读新增行的那几格各自把整份 diff 装进内存，17 格同时起时内存按格数叠加。
stage_cells_run "$ROOT"
