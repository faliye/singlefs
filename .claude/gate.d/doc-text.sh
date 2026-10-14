#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 文本写法：全仓不许出现登记过的禁用写法（撇号类角标、描述里的钟点与时区词、会话号）与登记过改名的术语旧名；改动碰到的记录与 kb 页日期只写在标题上；登记的三语文件（仓根 README 与 briefs/）三份齐全、结构对得上、改一份就三份同进同出、译文照翻译术语表用词
# gate-fixture: yes
# gate-category: 文档类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 的两项判被测代码的行为，这一道只判文本写法。
# gate-similar: code-tooling.sh 它的 fixture-claims 格也扫全仓文件，但判的是文件名与样本目录里编出来的日期、阶段头部声称的样本在不在，不是文本里的写法
# gate-similar: code-source-discipline.sh 它的 vague-names 与 test-file-names 两格也判写法，但对象是 .rs 里声明的函数名、类型名与测试文件名，读的是空泛词表与按里程碑起名的正则，不扫全部文本
# gate-similar: .claude/singlefs-ai-sop/scripts/i18n-sync.sh 它判上游各语言仓之间的同步（兄弟目录里的仓、清单、溯源哈希）；trilingual-* 四格判本仓里登记的英、中、日三份文件，对象与输入都不同，不并进它
#
# gate-cell: prime-marks 撇号类角标
# gate-cell: clock-times 描述里的钟点、时区词与时间戳
# gate-cell: session-names 会话号（ListAgents 列的会话名）
# gate-cell: term-renames 登记过改名的术语不再出现旧名
# gate-cell: date-once 这次改动碰到的记录与 kb 页里，日期只在标题上：正文行不重复所属标题或文件名里的日期，同一天不开第二个同级小节
# gate-cell: trilingual-files 登记的三语文件三份齐全：英语 <名>.md 的 <名>.zh.md、<名>.ja.md 都在，登记目录里单有 .zh.md 或 .ja.md 的也要三份齐全；登记表写对了
# gate-cell: trilingual-structure 三语文件每组三份的结构对得上（编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数），抓只改了一份
# gate-cell: trilingual-changed-together 这次改动碰了一组三语文件里的一份，就要三份都碰
# gate-cell: trilingual-glossary 翻译术语表写对了，三语文件的译文照术语表用词
# gate-cell: private-values 这次改动新加的行与改动的文件名里不出现本机私有配置 multi-host.env 里的值（8 个字以上的词）与本机主机名
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
# 这次改动碰到的（含未跟踪的）records/*.md 与 .claude/kb/ 下的 .md（两份变更史与 INDEX.md 不判）逐份交
# scripts/kb/doc-consolidate.py migrate 干跑，干跑改了什么就是违例；哪些日期不算，判法在那份脚本文件头。
# 基准取 scripts/gate/changed-paths.sh 的 gate 取法；不在 git 仓里整棵树都判；一份都没碰到：退 77。红行里的路径经 private-values 那一份打码（shown_path）。
# 规则：.claude/rules/changelog-format.md、kb-discipline 第 1 条。管不到的：别的日期该不该写、日期挪进标题之后小节切得对不对。
#
# ── trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary ──
# 对象是 .claude/trilingual-documents 登记的英语文件（今天是仓根 README.md 与 briefs/*.md）：一份英语文件 <名>.md 配 <名>.zh.md 与 <名>.ja.md，
# 四格都从这张登记表取对象，判法都在同目录 lib-trilingual.py（四个子命令，文件头有每一条判据与每个弄坏开关），阶段里不抄第二份。
# files：登记表自己写对了（行有理由、模式指到文件、模式不带语言后缀）；每组三份都在；登记目录里单有 .zh.md 或 .ja.md 的也要三份齐全，逐份点名缺哪份。
# structure：三份齐全的组逐项比，一项不同就点名组、项与各份的值：编号集合（D、E、C 加数字，I- 加数字与点；不用 \b，日文字符算词字符）、
#   Markdown 链接目标 ](目标) 的集合、围栏外各级标题的个数、围栏外 | 起头的表格行个数、围栏代码块个数。缺份的组不比，归 files。
# changed-together：基准取 scripts/gate/changed-paths.sh 的 gate 取法（gate_diff_base gate 与 gate_changed_paths … untracked，阶段里不再抄一遍）；
#   碰了一组里的一份，三份都要碰，删掉的也算碰；一组都没碰到、或不在 git 仓里：退 77。
# glossary：翻译术语表 .claude/kb/translation-glossary.md，表上方单独一行 <!-- trilingual-glossary --> 紧挨着表，表头
#   | 中文 | English | 日本語 | 说明 / Note / 注 |。表本身：每行四列、前三列不空、中文列不重复、说明列空着或恰好三段（用 <br> 分隔、每段不空）。
#   译文：每组三份齐全的，中文列的串在 zh 那份里原样出现，则 English 列的串要在 en 那份里出现（不分大小写），日本語列的串要在 ja 那份里原样出现。
#   术语表文件不在：判红，不是 77，出路写建它。
# 登记表不在：四格都判红（对象从它取）。
# 管不到的：译文译得对不对、有没有漏译段落与句子（只比结构与术语）；术语在句子里用得自然不自然；链接目标的写法在三份里逐字相同（不认语言后缀的映射）。
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
#
# 弄坏开关（打开后样本判错）：
#   GATE_TEXT_NOTATIONS_BREAK=date-once-unjudged      date-once 不干跑、记无对象可判：date-once-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=term-renames-unjudged   term-renames 不调 sweep-term、记无对象可判：term-renames-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-files-unjudged               trilingual-files 不判、记无对象可判：trilingual-files-red、trilingual-files-registry-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-structure-unjudged           trilingual-structure 不判、记无对象可判：trilingual-structure-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-changed-together-unjudged    trilingual-changed-together 不判、记无对象可判：trilingual-changed-together-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=trilingual-glossary-unjudged            trilingual-glossary 不判、记无对象可判：trilingual-glossary-red、-table-red、-marker-red、-missing-red 退 77
#   LIB_TRILINGUAL_BREAK=<项>   关掉 lib-trilingual.py 里的一条判据（项名与每项让哪几条样本判错，写在那份文件头「弄坏开关」一段）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-unjudged   private-values 不比对、记无对象可判：private-values-red 退 77
#   GATE_TEXT_NOTATIONS_BREAK=private-values-short-words  门槛从 8 个字降到 4 个字：private-values-green 判红（短词的那一行被点名），private-values-red 里「新加的 3 行」那条 want 找不到
#   GATE_TEXT_NOTATIONS_BREAK=private-values-whole-tree  不限新加的行、逐份文件整份比：private-values-green 判红（基准里就有的旧行被点名）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-hostname-skipped  不取本机主机名：private-values-red 里「键名：本机 hostname（值不打印）」那条 want 找不到，private-values-green 里「与本机主机名判了」那条 want 找不到
#   GATE_TEXT_NOTATIONS_BREAK=private-values-print-values  红行里把命中的值也打出来：private-values-red 里每条「（值不打印）」want 找不到（输出里是「（值：…）」）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-big-file-truncated  未跟踪文件只读前 8 MiB、超大也不判红（改前的样子）：private-values-big-untracked-red 与 private-values-too-big-red 退 0，退出码不对
#   GATE_TEXT_NOTATIONS_BREAK=private-values-names-unjudged  不比文件名：private-values-filename-only-red 退 0，private-values-filename-and-content-red 里「〔文件名含私有值〕  键名」那条 want 找不到
#   GATE_TEXT_NOTATIONS_BREAK=private-values-path-printed  红行里的路径原样打（含私有值的那一段也打）：private-values-filename-and-content-red 里「〔文件名含私有值〕」两条 want 找不到；兄弟三格的路径同样原样打，private-values-siblings-red 里三条路径 want（private-values 的「键名」行、clock-times 的红行、「二进制：」行）找不到
#   GATE_TEXT_NOTATIONS_BREAK=private-values-siblings-unmasked  prime-marks / clock-times / session-names 三格打路径与行文片段不打码（改前的样子）：private-values-siblings-red 里 clock-times 红行、
#     prime-marks 与 session-names 的「二进制：」行三条「〔文件名含私有值〕」want 找不到（输出里是从第二台 tar 过来的整段假仓路径）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-snippet-cut  三格的行文片段照命中位置前 20 后 10 个字直接截，不挪到私有值的边上（改前的样子）：
#     private-values-clock-snippet-red 里两条「…〔私有值〕 上 <时间戳> …」want 找不到（输出里是截剩的半截假值）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-date-once-term-renames-unmasked  date-once 的红行与 term-renames 转出的 sweep-term 输出原样打路径（改前的样子）：
#     private-values-date-once-path-red 里「〔文件名含私有值〕」那条 want 找不到（输出里是带假第二台主机名的文件名）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-path-by-segment  路径打码退回逐段比（改前的判法：哪一段含整个值才打码，值带 / 跨几段时一段都不含）：private-values-cross-segment-red 里两条「〔文件名含私有值〕」want 找不到
#     （红行把 /srv/… 这类假路径连同上下几级原样打出来），「键名」那两条 want 的路径部分同样对不上，private-values-siblings-red 的三条路径 want 同样找不到；private-values-filename-only-red、private-values-filename-and-content-red 不红（值在一级目录名里，逐段比也打得到）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-links-followed  不判符号链接（改前的样子：跟着链接走，坏链接整份跳过、好链接读它指到的内容）：private-values-symlink-dangling-red 与
#     private-values-symlink-mount-red 退 0，退出码不对（链接文字里的假仓路径、假第二台主机名没人比）
#   GATE_TEXT_NOTATIONS_BREAK=private-values-unreadable-raises  读文件不包异常（改前的样子：读不了就让格崩，python 回溯把带假仓路径的原路径打出来）：private-values-unreadable-red 里
#     「1 个文件读不了」「research/results/〔文件名含私有值〕/research/results/e999.out  EACCES」两条 want 找不到；兄弟三格同样崩，「prime-marks：绿」「clock-times：绿」「session-names：绿」与「读不了（EACCES」四条 want 找不到
#   LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键或形态键>   关掉判据行：红样本里点名那一行的报不出来
#   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=<判据行键或形态键>、LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=<形态键>、
#   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=<形态键>  关掉上下文规则、不认放行标记、排除前缀失效：那一格的绿样本判红
#   （键写在 lib-forbidden-notations.py）
#   STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；=marker-ignored 让每份样本几格都跑，「跑了 1 格」的 want 找不到
#
# 样本：fixtures/doc-text.sh/ 下四十二份，每份样本根放 .gate-cells 只点名它判的那一格（trilingual-* 十五份的现场都由 setup.sh 现造，日期取真实发生过的日子；private-values-siblings-red 与 private-values-unreadable-red 点名四格）：
#   date-once-red       setup.sh 建 git 仓；这一轮往记录正文加一行带文件名日期的话、往实验页加第二个同日 `### ` 小节，逐份点名判红
#   date-once-green     同一基准；加的行只带别的日期、加的小节是另一天，判绿
#   prime-marks-red     一份带「K9」加一撇的 md、一份带「G5」加两撇的 rs，逐处点名「文件:行」判红
#   prime-marks-green   只有正常写法（变体起了新名字、ASCII 单引号），外加上游副本里带角标的 md（排除前缀下），判绿
#   clock-times-red     每条判据行各有一份带那种写法的 md 或 rs，外加写在 md 里与没写理由的放行标记，逐处点名「文件:行」与判据行判红
#   clock-times-green   只有日期与近似但该放行的写法，.sh、.py、.rs 里带放行标记的各一行，外加钟点排除前缀下的门禁样本、hook 自检与 c510 提示，判绿
#   session-names-red   标签式、当主语用的、上游三语仓的、长十六进制的会话号各一行，外加 rs 里没写理由的放行标记，逐处点名判红
#   session-names-green 近似但该放行的写法（crate 名、mktemp 模板与临时目录名、上游仓目录名、带字母后缀的标识符）与 .sh 里带放行标记的一行，
#                       外加会话号排除前缀下的门禁样本，判绿
#   term-renames-red    一份改名登记表与一份还用着旧名的 rs（旧名由几段拼出来，样本目录里不出现旧名的字面），判红
#   term-renames-green  同一张登记表，rs 里只有新名，判绿
#   term-renames-line-exempt-green  setup.sh 建 git 仓；登记表带一行「正则」，记录里只有树层、外层那一行与按行豁免的一句用户原话带旧名，
#                       git 忽略的 multi-host.env 里有旧名，判绿（正则去掉断言、扫了 git 忽略的文件都红；旧名同样由几段拼出来）
#   term-renames-line-exempt-red  同一现场，按行豁免的那份记录里另有一行没豁免的旧名：只报那一行（1 份文件、1 处），判红
#                       （按行豁免当成整份豁免就判绿，正则去掉断言、扫了 git 忽略的文件报的处数就不是 1）
#   trilingual-files-red          登记 README.md 与 briefs/*.md：README 缺日语、一组缺中文、一组只有中文、一组有中文和日语而英语不在，逐份点名缺哪份判红
#                       （reverse-ignored 抓不到后两组）
#   trilingual-files-registry-red 登记表三处不对（模式指不到文件、行没写理由、模式写成了 .zh.md），逐行点名判红
#   trilingual-files-no-registry-red  没有登记表，判红（不是 77）
#   trilingual-files-green        三组每组三份齐全；登记目录之外的单份 .zh.md 与目录里另一种语言后缀的文件不归这一格管，判绿
#   trilingual-structure-red      五组各错一项（编号集合、链接目标集合、二级标题个数、表格行个数、围栏代码块个数），逐项点名各份的值判红
#   trilingual-structure-green    日文与中文里编号紧挨文字、链接带三种语言的标题、围栏里有 # 与 | 开头的行且三份个数不同、四个反引号的围栏里夹三个反引号的行，判绿
#   trilingual-changed-together-red    setup.sh 建 git 仓：工作区改、暂存、未跟踪三种来源各有一组只碰了一部分，逐组点名没碰哪几份判红
#   trilingual-changed-together-green  同一基准：三份都改（工作区、暂存）、三份都删、三份都新写，另有一组没碰，判绿
#   trilingual-changed-together-unrun  只改了与登记无关的文件，退 77
#   trilingual-glossary-red       表写对了，英语写 Crash-point、日语写リプレー、日语写関門：逐条点名哪一组、哪个中文术语、缺哪份的哪个译名，判红
#   trilingual-glossary-green     英语大小写与术语表不同、简报里中文没用的术语不核别的两份，判绿
#   trilingual-glossary-table-red 表头、列数、前三列有空、中文列重复、说明列段数各一处，逐行点名判红
#   trilingual-glossary-marker-red      表在而没有 <!-- trilingual-glossary --> 标记，判红
#   trilingual-glossary-not-table-red   标记下面隔了空行才是表，判红
#   trilingual-glossary-missing-red     没有术语表文件，判红（不是 77），出路写建它
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
#
#   bash .claude/gate.d/doc-text.sh [项目根]                                     十格都跑
#   bash .claude/gate.d/doc-text.sh --list                                       逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-text.sh --check <格名>[,<格名>…] [项目根]            只跑点名的格
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
stage_cell trilingual-files cell_trilingual_files "登记的三语文件三份齐全：英语 <名>.md 的 <名>.zh.md、<名>.ja.md 都在，登记目录里单有 .zh.md 或 .ja.md 的也要三份齐全；登记表写对了" \
  "缺哪份补哪份：<名>.md 是英语（默认），<名>.zh.md 是中文，<名>.ja.md 是日语，三份同时在；登记表 .claude/trilingual-documents 一行一条「<模式>  # 理由」，模式指英语那一份、要指到文件、理由不许省"
stage_cell trilingual-structure cell_trilingual_structure "三语文件每组三份的结构对得上（编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数），抓只改了一份" \
  "照点名的组和项，把落后的那几份改到与改了的那份一致（另一份多出来的内容就补译，误删就恢复）；围栏代码块里的 # 开头行与 | 开头行不算标题与表格行"
stage_cell trilingual-changed-together cell_trilingual_changed_together "这次改动碰了一组三语文件里的一份，就要三份都碰" \
  "改一份就要三份一起改：没碰的那几份照改了的那份同步译过去；这一组其实不该改的，把误碰的那份改回去"
stage_cell trilingual-glossary cell_trilingual_glossary "翻译术语表写对了，三语文件的译文照术语表用词" \
  "表的毛病照点名的行改 .claude/kb/translation-glossary.md（每行四列、前三列不空、中文列不重复、说明列空着或恰好三段用 <br> 分隔）；译文的毛病改译文，zh 里用了某术语，en 和 ja 里就用术语表给的译名；术语表文件不在就建它"
stage_cell private-values cell_private_values "这次改动新加的行与改动的文件名里不出现本机私有配置 multi-host.env 里的值（8 个字以上的词）与本机主机名" \
  "把命中的词换成「第二台」「本机」这类说法（红行列的是文件、行号与键名，值不打印，路径里含值的那一段打成「〔文件名含私有值〕」；文件名含值的改名；要引整句的，整句里的值也换掉；超大的未跟踪文件拆小）；值不要进仓、不要进日志原样输出，用到它的命令照 multi-host.env 的键名写"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
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
  echo "               换完跑 cargo test --workspace 与 bash scripts/experiments/replay.sh，同一份源码要仍然吐出逐字节相同的产物——"
  echo "               那是重新生成，不是改证据；**跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，"
  echo "               连同引它的正文整段留旧名（evidence-discipline「原样保存的证据不许事后改」）。"
  echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件里换了就成假话的那一句、引文块、对照表自己），"
  echo "               登记进 .claude/term-rename-exempt 并写明为什么；历史类文件里只有一句该留的写 <路径>:「<那一行里的一段原文>」只豁免那一行，"
  echo "               整份都该留的逐文件登记，不整个目录豁免；指向不存在路径、行内锚命中不是恰好一行的那一条改掉或删掉。"
  exit 1
}

cell_date_once() {
  local base changed path count=0 red=0 scratch summary dates merges shown
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
  scratch="$(mktemp "${TMPDIR:-/tmp}/date-once.XXXXXX")"
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    case "$path" in
      records/*.md) ;;
      .claude/kb/*.md) ;;
      *) continue ;;
    esac
    case "$path" in
      .claude/kb/decisions-history.md|.claude/kb/experiments-history.md|.claude/kb/INDEX.md) continue ;;
    esac
    [[ -f "$path" ]] || continue
    count=$((count + 1))
    summary="$(python3 "$DOC_CONSOLIDATE" migrate "$path" --out "$scratch" --force 2>&1 | head -1)"
    dates="$(sed -n 's/.*同日日期删 \([0-9]*\).*/\1/p' <<<"$summary")"
    merges="$(sed -n 's/.*同日小节合并 \([0-9]*\).*/\1/p' <<<"$summary")"
    if [[ "${dates:-0}" != 0 || "${merges:-0}" != 0 ]]; then
      red=$((red + 1))
      # 红行里的路径经 private-values 那一份打码（shown_lines_main 的 path）
      shown="$(printf '%s\n' "$path" | python3 -c 'import sys, types; library = types.ModuleType("lib_private_values"); exec(compile(sys.argv[1], "private-values-library", "exec"), library.__dict__); library.shown_lines_main(sys.argv[2:], sys.stdin, sys.stdout)' \
        "$PRIVATE_VALUES_LIBRARY" "$PRIVATE_CONFIGURATION_FILE" "$PRIVATE_HOSTNAME_FILE" "$text_notations_break" path)"
      echo "  ✗ ${shown:-〔路径打码没打出来〕}：正文里重复所属日期 ${dates:-0} 处、同一天的第二个小节 ${merges:-0} 个"   # gate-lint:detail
    fi
  done <<<"$changed"
  rm -f "$scratch"
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

# 三语四格的判法都在 lib-trilingual.py；弄坏开关 trilingual-<格名>-unjudged 让这一格不判、记无对象可判（退 77）
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

stage_cells_run "$ROOT"
