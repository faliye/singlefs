#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓里三语文件与术语表的样子，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本，除了 python3 之外没有环境要求
# gate-similar: .claude/singlefs-ai-sop/scripts/i18n-sync.sh 它判上游各语言仓之间的同步（兄弟目录里的仓、清单与溯源哈希）；这一份判本仓里登记的英、中、日三份文件，对象与输入都不同，不并进它
"""三语文件的判法：门禁 doc-text 的 trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary、trilingual-untranslated、trilingual-omission 六格调它，不单独成阶段。

三份：一份中文文件（默认语言）<名>.md 配 <名>.en.md（英语）与 <名>.ja.md（日语）。
登记表 .claude/trilingual-documents 一行一条「<模式>  # 理由」，模式指中文那一份（相对仓根；* 与 ? 不跨目录，** 跨目录），理由不许省。
名里可以带点；以 .<两个小写字母>.md 结尾的（.en.md、.ja.md 是译文，.fr.md 这类是别的语言）不是中文那一份，只有 .en.md 与 .ja.md 进组。

用法：
    lib-trilingual.py files <根>              登记的文件三份齐全；登记表本身写对了
    lib-trilingual.py structure <根>          每组三份的编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数逐项相同
    lib-trilingual.py changed-together <根>   标准输入给这次改动碰到的路径（一行一条）；碰了一组里的中文那一份，英语、日语两份都要碰；只碰了译文的组放行
    lib-trilingual.py glossary <根>           术语表写对了，译文照术语表用词
    lib-trilingual.py untranslated <根>       英语、日语译文里，去掉该留原样的段之后不残留超量没译的中文；与中文那一份某一行逐字相同、去段之后还剩中文的行也不许有
    lib-trilingual.py omission <根>           英语、日语译文与中文逐段配对比内容量：不过短（漏了一段、压成摘要）、不过长（对错了段），各节段数与首列编号的表格行配得上

退出码：0 都对；1 有不对的（逐条列出）；2 用法错；77 无对象可判（不是通过）；78 准入不满足（preflight）。

判法细节：
  files：登记表缺、读不出，或行没写理由、模式指不到任何文件、模式带语言后缀或不以 .md 结尾，判红；
        每组三份缺哪份点名哪份；反过来，登记模式所在目录里出现了 <名>.en.md 或 <名>.ja.md、而 <名>.md 符合某条登记模式，
        这一组同样要三份齐全（中文那份不在也判红）。
  structure：只比三份都在的组（缺份的归 files）。编号用 (?<![A-Za-z0-9_])(?:[DEC][0-9]+|I-[0-9]+(?:\\.[0-9]+)*)(?![0-9])，
        不用 \\b（日文字符算词字符，「ファイルE152」里 \\b 不成立）；编号不数 Markdown 链接目标 ](…) 括号里那一段，链接文字照数
        （译文链的是译文另起的文件名，.claude/kb/translation-paths.json 登记的：中文文件名里的 I-3.1 在日语文件名里写成 I-3-1、英语文件名里写成小写，
        数进去三份的编号集合就对不上；目标本身归下面的链接目标集合比）；链接目标取 ](目标) 里第一个空白之前的那一段，
        以 .en.md 或 .ja.md 结尾的目标先归成 <名>.md 再比（译文里链同一种语言的那一份，与中文链中文那一份算同一个目标；
        三语切换行 [中文](<名>.md) · [English](<名>.en.md) · [日本語](<名>.ja.md) 每份不链自己，归名之后三份都是 <名>.md）；
        标题取围栏外 ATX 写法（行首至多三个空格、一到六个 #、后接空白或行尾）的行，按级别分别数；表格行取围栏外去掉行首空白后以 | 起头的行；
        围栏取 ``` 或 ~~~ 起头的开围栏，数开围栏的个数。
  changed-together：路径是 <名>.md、<名>.en.md、<名>.ja.md 之一，而 <名>.md 符合某条登记模式，就算碰到了那一组（不看文件还在不在，删掉的也算碰）。
        碰到的组里，中文那一份 <名>.md 碰了的，英语、日语两份都要碰，缺哪份点名哪份；中文那一份没碰、只碰了英语或日语（只改译文）的组放行，不报，
        成功那句与红的汇总另报放行了几组。用户 2026-10-23 在弹窗里定「只在中文变了时要求同改」：译文跟着中文走，中文没变时单改译文（补译、修译名）是常事，
        译得对不对交给 structure、glossary、untranslated 三格；改前的判法（碰了任何一份就要三份都碰）当天主 agent 相对 HEAD 现跑，碰到的 254 组里 216 组判红，绝大多数是只改译文。
  glossary：术语表 .claude/kb/translation-glossary.md，单独一行 <!-- trilingual-glossary --> 紧挨着表，表头
        | 中文 | English | 日本語 | 说明 / Note / 注 |，分隔行，再是数据行（到第一个不以 | 起头的行为止）。
        表：每行四列，中文、English、日本語三列不空；中文列不重复；说明列可以空，不空就要恰好三段、用 <br> 分隔、每段不空。
        译文：每组三份齐全的，中文列的串在 zh 那份里原样出现，则 English 列的串要在 en 那份里出现（不分大小写），日本語列的串要在 ja 那份里原样出现。
        「在 zh 里出现」按词边界认：中文没有空格，术语前后紧挨别的字很常见（「实验产物」「三条合表」里的术语都算用到），所以不看前后一个字是不是汉字，
        而是看这一处是不是落在登记过的「更长的词」里。登记文件 .claude/gate.d/trilingual-glossary-longer-words.tsv（相对被判的仓根），
        一行一条「<术语><制表符><包含这个术语、但不是这个术语的更长的词><制表符><理由>」，# 起头的行与空行跳过：
        术语在 zh 里的某一处，如果正好落在那个更长的词的某一次出现里（位置对得上：词里术语所在的那几个字就是这一处），这一处不算用到；
        一份 zh 里至少有一处没落在任何登记的更长词里，才算用到术语、才核译名。例：「集合表示」里的「合表」、「副产物」里的「产物」不算，「实验产物」里的「产物」算。
        登记文件不在就没有排除，等于按子串认。登记文件自己要写对：每行三列、更长的词要真包含术语且不等于术语、术语要是术语表中文列里有的、理由至少 4 个字、同一对不许写两次；写错的逐行点名判红。
        英语一侧不加词边界：English 列的串后面带词形变化（replay 对 replayed、artifact 对 artifacts）是正常译法，加了右边界，已经对的译文会判红；
        日本語一侧按原样子串认。
        改名登记表那一组（.claude/kb/term-renames.md，路径取 scripts/kb/sweep-term.py 的 RENAMES_PATH）：登记表数据行的旧名、新名两列不算用到术语，
        这两列是门禁 doc-text 的 term-renames 格拿去全仓搜的字面，规定三份里都照中文原串写、不译；同一份里别处（表上方的正文、「是什么」那一列）用到的照核。
        哪几行是数据行、怎么切格照 sweep-term.py 的 load_renames 与 table_cells（标记 <!-- term-renames:table --> 之后以 | 起头的行，表头与分隔行不算），
        不另写一份；load_renames 读不出这张表时一行不换、整份照核（表的毛病归 term-renames 格报）。成功那句报换了几行。
  untranslated：对象同另外几格从登记表取组；每组在的英语、日语译文逐行判（缺份的组在的那几份照判，缺的归 files）。
        射程是全仓此刻的样子，同 files、structure、glossary，不按这次改动划窗：伪译文是文件的状态，哪一次改动碰没碰它都是错的，
        这一次漏网的 checks-owed 日语那份正是早先写进、之后的改动没再碰到的行。
        不判的组登记在 .claude/gate.d/trilingual-untranslated-exempt.tsv（相对被判的仓根）：一行「<中文那一份的路径><制表符><理由>」，# 起头的行与空行跳过，
        只登记整份照设计就三语并列的（术语表、改名登记）。登记文件不在就没有排除；写错的行（不是两列、路径不是登记表指到的在的中文那一份、
        理由不到 4 个字、同一份登记两次）逐行点名判红。
        每行先去掉该照中文原样留的段：闭合了的围栏代码块整块（行首可带空白与 >，列表项、引用块里照录旧文件原文的块也认；没有闭合行的开围栏不算，后面照判）、
        HTML 注释（可跨行）、反引号段、Markdown 链接目标 ](…)、「」『』与英语直弯引号 "…"、“…” 里的引文（套着的由里往外反复去净）、
        带 ASCII 字母或数字的路径（含 / 的一段，或以 .md、.rs、.py、.sh、.tsv、.json、.txt、.toml、.log、.out 收尾的文件名；全是汉字的「读/写」不当路径）。
        剩下的部分：英语数汉字（CJK 统一汉字、扩展 A、扩展 B 起各区与兼容汉字），≥ ENGLISH_HAN_THRESHOLD 个记一处；
        日语数日语不用的简体中文字（SIMPLIFIED_ONLY），≥ JAPANESE_SIMPLIFIED_THRESHOLD 个记一处。
        与中文逐字相同：不到上面门槛的行，整行（不去段、不去首尾空白）与同组中文那一份的某一行逐字相同、本身不是空白行，而剩下的部分英语还有汉字、日语还有日语不用的简体字
        （≥ IDENTICAL_THRESHOLD 个，英语日语同一个数），也记一处：照抄中文没译，数得到一个就是没译的字。中文那一份不在的组不比这一条（缺份归 files）。
        剩下部分没有被数的字的逐字相同行（只剩数、拉丁字母、标点，或中文都在反引号、引文、链接目标、路径、HTML 注释里，日语里只剩日语也用的汉字）放过，成功那句报放过几行。
        红行先按文件列红行数，再逐行报「文件:行号  语言，剩 N 个…（门槛 M）：片段」，片段从剩下部分第一个被数的字起取 SNIPPET_CHARACTERS 个字；
        与中文逐字相同的报「文件:行号  语言，与中文逐字相同（中文那一份:行号），剩 N 个…：片段」，汇总另报其中几行是这一类。
        字表 SIMPLIFIED_ONLY：GB 2312（简体中文的国标字符集）收的 6763 个汉字里，JIS X 0208（日语的国标字符集，即 Shift_JIS 编得出的汉字）没收的 3432 个。
          这些字日语正文用不到（「这、们、说、时、对、实、现、发、进、单、确、响」这类），中文里却常见：中文那几份照同样去段之后 1850525 个汉字里 472127 个在表里（0.255）。
          先取 GB 2312 再减，而不是只取「JIS X 0208 没收的字」：JIS X 0213 才收的日语字（剝、頰 这类）不在 GB 2312 里，不会被当成简体字。
          重新生成（打出来的 3432 个字与常量按码位逐字相同）：
            python3 -c 'print("".join(chr(c) for c in range(0x4E00, 0xA000) if chr(c).encode("gb2312", "replace") != b"?" and chr(c).encode("shift_jis", "replace") == b"?"))'
        门槛怎么定的（2026-10-23 在主工作区量，排除登记的两组不算；英语 254 份、日语 254 份）：
          英语：剩下部分有汉字的行 426 行；正好 5 个的 259 行几乎都是语言切换行（[中文] 与 [日本語] 共 5 个汉字，254 行带「中文」）。
            门槛 6、10、12、14、15、16、20、30 依次红 30、7、4、4、3、3、3、2 行。照设计带中文的行最多 14 个（E169 页第 13 行列的候选名甲乙丙丁），取 15 正在它上面。
            此刻红的 3 行：checks-owed 英语 C1 那一行整格没译（53 个）；决策变更史英语第 14465 行引用块里的用户原话照中文留着（50 个，要么译、要么放进「」）；
            E183 页英语第 22 行变异名前缀照中文写在反引号外（20 个，算误报一例：名字本身要留中文，出路是放进反引号）。
            漏报一例：D5 页英语第 216 行统计量名「待删除但意图未完成占用」没译，剩 11 个，不到门槛。
          日语（2026-10-23 门槛从 5 收到 3 时在主工作区 kb 的同一份快照上重量，日语 254 份）：剩下部分有这类字的行 726 行；
            门槛 1、2、3、4、5、6、8、10 依次红 726、564、492、445、422、400、366、327 行（84、33、17、13、11、9、6、6 份）。
            取 3 是用户在弹窗里定的：从 5 收到 3 多红 3、4 两档共 70 行、6 份文件，逐行读过，都是没译的中文（C 号、D 号的简称照中文留着，
            「结论登记」「复跑第三段」这类小节名，「桩」「份」混进日语句子），没有一处是日语本来就用的字。
            中文一行 10–14 个汉字的 68% 到 3 个这类字（到 5 个的 22%），15–19 个的 85%，20 个以上的 92% 以上：没译的中文短句到 3 个就抓得到。
            误报一例：E166 页日语第 89 行，臂名 P设、O读 照中文写在反引号外（3 个；名字要留中文，出路是放进反引号）。
            漏报一例：D5 页日语第 216 行统计量名「待删除但意图未完成占用」没译，只数到 2 个（删、图）；1、2 两档共 234 行。
          与中文逐字相同（IDENTICAL_THRESHOLD 取 1；2026-10-23 主 agent 在主工作区量，排除登记的两组不算，量法同这一条：整行与中文那一份某一行逐字相同、不是空白行、
            不在闭合的围栏里，照上面的去段之后英语剩至少一个汉字、日语剩至少一个日语不用的简体字）：全仓 40 行、12 份文件，逐行读过都是真问题，没有误报：
            E165 页日语的标题与表头、D5 页英语与日语的表格行、7 份英语文件与 README 三份照抄中文的语言切换行（英语那份切换行剩「中文」「日本語」5 个汉字，
            日语表头剩 1 到 2 个简体字，都在上面两个门槛之下，所以单靠门槛判不出）。这 40 行当天改掉之后，同一量法在主工作区读到 0 行。
            取 1 的理由：整行与中文逐字相同说明译者没碰过这一行，去段之后剩下的中文都是没译的；日语只数日语不用的简体字，标题、表头只用日语也用的汉字
            （与中文写法相同的「概要」这类）的照抄行判不出，算漏报，不为它放宽到全部汉字（日语正文里本来就是汉字，逐字相同的短行会误报）。
  omission：对象同另外几格从登记表取组，只比三份齐全的（缺份的归 files）；射程是全仓此刻的样子，同 files、structure、glossary、untranslated，不按这次改动划窗：
        漏了的段、压成的摘要是文件的状态，E58 英语漏的 2026-10-21 那一段、E178 英语整页压过的摘要都是早先写进、之后的改动没再碰到的。
        切法（omission_units）：围栏外的 ATX 标题（同 structure 的 HEADING）把一份切成节，第一个标题之前算一节；三份的节数不同，这一份不比，汇总报个数，归 structure。
        节里的段：空行隔开的连续几行算一段，列表项（行首可带空白与 >，- * + 或「数字.」「数字)」起头）另起一段，表格行一行一段（分隔行不算）；
        闭合的围栏代码块整块不算（认法同 untranslated 的 fenced_line_indexes），HTML 注释先去掉（换成同样多的换行，行号不变），三语切换行不算一段、前后断开
        （E37 页中文的切换行下面没空行、紧接正文，不断开就比英语日语少一段）。
        配对：首列里有编号（同 structure 的编号写法，不数链接目标里的）的表格行按（编号，这个编号在这一份里第几次出现）配对，不看次序；其余的段按节内次序配对。
        一节里三份段数不同的，这一节不比比值，报一处「这一节段数与中文不同」（点名译文那一节的标题行、中文那一节的标题行、两边段数与整节内容量比）；
        编号只在一边的表格行报一处（中文有、译文没有的点名中文那一行）。两类合称配不上。
        内容量（omission_measure）：先去掉反引号段、链接目标 ](…) 与带 ASCII 字母或数字的路径（同 untranslated 的 INLINE_CODE、LINK_TARGET_SEGMENT、PATH_TOKEN；三份照原样留，不是译出来的量），
        中文数汉字个数加 ASCII 词数（一串字母数字算一个，数也这样数），英语数 ASCII 词数加汉字个数（照原样留的中文原话照数），日语数汉字与假名个数加 ASCII 词数。
        中文那一段不到 OMISSION_MINIMUM_MEASURE（20）的不比（短段比值抖得厉害：「| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |」译全了也只有 0.43）。
        比值是译文量 ÷ 中文量，低于 OMISSION_RATIO_LIMITS 的前一个数记过短，高于后一个数记过长。红行先按文件列处数，再逐处报「译文:行号  语言，内容量比 R（门槛，过短或过长）：
        中文那一份:行号 量 Z、这一份 T：中文片段」，片段取中文那一段开头 OMISSION_SNIPPET_CHARACTERS 个字。
        门槛怎么定的（2026-10-24 在主工作区量，256 组，中文一段不到 20 的不算；英语 21555 对、日语 21473 对）：
          比值的分布：英语中位数 0.694，千分位 0.212，1% 分位 0.356，2.5% 分位 0.414，5% 分位 0.467，99% 分位 1.0，千分之九百九十九分位 1.163；
            日语中位数 1.648，千分位 0.307，1% 分位 0.755，2.5% 分位 0.903，5% 分位 1.016，99% 分位 2.224，千分之九百九十九分位 2.625。
          过短：英语门槛 0.30、0.35、0.42、0.45、0.50 依次红 78、199、593、882、1432 对；日语 0.70、0.80、0.85、0.90、0.95、1.00 依次红 156、262、367、519、755、953 对。
            低的那一头没有断档，取英语 0.42（中位数的六成，2.4% 分位）是要抓得到 E58 那种漏法：E58 英语的 HEAD 那一版（补齐之前）回看表第 260–266 行
            每行漏了「2026-10-21 不受影响：只加了三语切换行」那一块，比值 0.41–0.45，取 0.42 抓到其中 5 行（0.41–0.42）；日语取同一分位上的 0.90。
            门槛附近抽读：英语 0.36–0.45 一档回看表之外的行抽 12 行，8 行漏了某一块或压成一句大意（E156、E183 的段落，里程碑三欠账表的列），
            4 行译全了、只是简称后面的括注没重复写（E167、验证装置页）或写得紧（E166、E182）；日语 0.80–1.00 一档抽 8 行，4 行是回看表漏了 2026-10-21 那一块，
            1 行漏了半句（prior-art），1 行看不准，2 行译全了（比值 0.89、0.98）。回看表漏 2026-10-21、2026-10-22 那几块的行在英语 0.35–0.42 一档 394 行里占 220 行。
            误报一例：README 英语第 105 行，中文那一行是英语口号后面带一段中文注解，英语那一行只有口号（译全了），比值 0.37。
            漏报一例：上面 E58 英语 HEAD 那一版第 264、265 行（漏的同一块，比值 0.45），不到门槛判不出。
          过长：英语门槛 1.3、1.5、2.0 依次红 9、4、2 对，日语 2.6、3.0、3.3、4.0 依次红 22、6、6、5 对；取英语 1.5、日语 3.3（中位数的两倍上下），
            红的 10 对里 9 对是 E139 两份译文与决策变更史英语表格行、段落对错了位（中文删改过、译文没跟），
            误报一例：决策变更史英语第 14519 行，引用块里照留中文原话又跟了一段英语译文，比值 1.77。
          校准：E178 英语整页重译之前的那一版（压成摘要的那一页，草稿目录里留着的副本；那一版多出的一个 ### 先改掉，照 structure 修好之后的样子）
            判出过短 36 处、配不上 1 处；E139 中文第 213 行英语那一行只剩一句（比值 0.13），判过短。

弄坏开关 LIB_TRILINGUAL_BREAK=<项>（只给门禁 doc-text 的样本证红用，一次给一项）。每项打开后 doc-text 哪几份样本判错，逐项写在门禁 doc-text 文件头的开关表（mutate.sh 写法），这里只写弄坏哪一处：
  registry-reason-ignored     登记表的行不要求理由
  registry-unmatched-ignored  模式指不到任何文件不报
  registry-suffix-ignored     模式带语言后缀不报
  reverse-ignored             不看目录里单有 .en.md / .ja.md 的那一组
  ids-ignored                 不比编号集合
  word-boundary               编号前后的限制换成 \\b
  links-ignored               不比链接目标集合
  link-language-kept          链接目标里的 .en.md / .ja.md 不归成 <名>.md
  headings-ignored            不比标题个数
  table-rows-ignored          不比表格行个数
  fences-ignored              不比围栏代码块个数
  fence-blind                 围栏里的行也当正文数标题与表格行
  link-target-ids-counted     编号集合也数链接目标 ](…) 里的编号（改前的样子）
  partial-ignored             碰了一份就当三份都碰了
  translation-only-required   只碰了译文（中文那一份没碰）的组也要三份都碰（改前的样子）
  glossary-marker-ignored     找不到标记不报
  glossary-header-ignored     表头对不上不报
  glossary-columns-ignored    列数不对、前三列有空不报
  glossary-duplicate-ignored  中文列重复不报
  glossary-note-ignored       说明列的三段不判
  glossary-translation-ignored  不核译文用词
  glossary-case-sensitive     English 列按大小写比
  glossary-zh-gate-ignored    中文术语不在 zh 里也核译名
  glossary-substring          退回按子串认中文术语（不读更长词登记文件）
  glossary-renames-columns-counted  改名登记表的旧名、新名两列也算用到术语（改前的样子）
  omission-ratio-ignored      omission 不比内容量（配对照做，只报配不上的）
  omission-keys-ignored       omission 的表格行不按首列编号配对，一律按节内次序
  omission-unpaired-ignored   omission 配不上的段（段数不同的节、编号行缺一边）不报
  下面六项由门禁 doc-text 的同名开关 GATE_TEXT_NOTATIONS_BREAK=<项> 转进来（那一道文件头的开关表逐条点名样本），直接设 LIB_TRILINGUAL_BREAK 也认：
  trilingual-untranslated-quotes-kept      不去掉「」『』与英语直弯引号里的引文
  trilingual-untranslated-code-kept        不去掉反引号段、围栏代码块、链接目标、路径与 HTML 注释
  trilingual-untranslated-japanese-unjudged  不判日语译文
  trilingual-untranslated-exempt-ignored   不读排除登记表
  trilingual-untranslated-japanese-threshold-five  日语门槛退回 5（收到 3 之前）
  trilingual-untranslated-identical-ignored  不判与中文逐字相同的行（没有这一条之前的样子）
"""
import contextlib
import glob
import importlib.util
import io
import os
import json
from functools import lru_cache
import posixpath
import re
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_OF_GATES = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_OF_GATES, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

REGISTRY = ".claude/trilingual-documents"
GLOSSARY = ".claude/kb/translation-glossary.md"
GLOSSARY_MARKER = "<!-- trilingual-glossary -->"
LONGER_WORDS = ".claude/gate.d/trilingual-glossary-longer-words.tsv"
# 门禁 doc-text 的 term-renames 格读改名登记表的那一份脚本；glossary 判改名登记那一组时照它读表（读法不另写一份）
TERM_RENAMES_SCRIPT = os.path.join(REPOSITORY_OF_GATES, "scripts", "kb", "sweep-term.py")
GLOSSARY_HEADER = ["中文", "English", "日本語", "说明 / Note / 注"]
GLOSSARY_COLUMN_NAMES = ["中文", "English", "日本語"]
LANGUAGES = (("zh", ""), ("en", ".en"), ("ja", ".ja"))
MINIMUM_REASON_CHARACTERS = 4
DETAIL_LIMIT = 200
BROKEN = os.environ.get("LIB_TRILINGUAL_BREAK", "")

IDENTIFIER_BODY = r"(?:[DEC][0-9]+|I-[0-9]+(?:\.[0-9]+)*)(?![0-9])"
IDENTIFIER = re.compile(r"(?<![A-Za-z0-9_])" + IDENTIFIER_BODY)
IDENTIFIER_WORD_BOUNDARY = re.compile(r"\b" + IDENTIFIER_BODY)
LINK_TARGET = re.compile(r"\]\(([^)]*)\)")
FENCE_OPEN = re.compile(r"^ {0,3}(`{3,}|~{3,})")
FENCE_CLOSE = re.compile(r"^ {0,3}(`{3,}|~{3,})[ \t]*$")
HEADING = re.compile(r"^ {0,3}(#{1,6})(?:[ \t]|$)")
LANGUAGE_FILE = re.compile(r"^(?P<name>.+)\.(?P<language>en|ja)\.md$")
LINK_LANGUAGE_SUFFIX = re.compile(r"\.(?:en|ja)\.md(?=$|[#?])")
LANGUAGE_SUFFIXED = re.compile(r"\.[a-z]{2}\.md$")
CHANGED_FILE = re.compile(r"^(?P<directory>(?:.*/)?)(?P<name>[^/]+?)(?P<suffix>\.en|\.ja)?\.md$")
TABLE_SEPARATOR_CELL = re.compile(r"^:?-+:?$")
UNESCAPED_PIPE = re.compile(r"(?<!\\)\|")
GLOB_CHARACTERS = re.compile(r"[*?]")


def report_problems(summary, details, advice, limit=DETAIL_LIMIT):
    """红：汇总一行、明细逐行（超过上限 limit 的折叠）、出路。"""
    print(f"  ✗ {summary}")   # gate-lint:summary
    for detail in details[:limit]:
        print(f"      {detail}")   # gate-lint:detail
    if len(details) > limit:
        print(f"      （另有 {len(details) - limit} 条没列出）")   # gate-lint:detail
    print(f"     → 怎么办：{advice[0]}")
    for line in advice[1:]:
        print(f"               {line}")
    return 1


def read_text(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return handle.read()


def pattern_regex(pattern):
    """登记模式 → 正则：* 与 ? 不跨目录，** 跨目录，其余按字面。"""
    pieces, index = [], 0
    while index < len(pattern):
        if pattern.startswith("**/", index):
            pieces.append("(?:.*/)?")
            index += 3
        elif pattern.startswith("**", index):
            pieces.append(".*")
            index += 2
        elif pattern[index] == "*":
            pieces.append("[^/]*")
            index += 1
        elif pattern[index] == "?":
            pieces.append("[^/]")
            index += 1
        else:
            pieces.append(re.escape(pattern[index]))
            index += 1
    return re.compile("^" + "".join(pieces) + "$")


def load_registry():
    """→ (条目列表, 登记表自己的毛病列表)；登记表不在返回 (None, [])。条目：行号、模式、理由、正则、匹配到的英语文件（排序）。"""
    if not os.path.isfile(REGISTRY):
        return None, []
    entries, problems = [], []
    for line_number, raw in enumerate(read_text(REGISTRY).split("\n"), 1):
        text = raw.strip()
        if not text or text.startswith("#"):
            continue
        pattern, separator, reason = text.partition("#")
        pattern, reason = pattern.strip(), reason.strip()
        where = f"{REGISTRY}:{line_number}"
        if BROKEN != "registry-reason-ignored" and (not separator or len(reason) < MINIMUM_REASON_CHARACTERS):
            problems.append(f"{where}：模式 {pattern} 没写理由（一行是「<模式>  # 理由」，理由至少 {MINIMUM_REASON_CHARACTERS} 个字）")
        pattern = posixpath.normpath(pattern) if pattern else pattern
        base = posixpath.basename(pattern)
        if not pattern or pattern.startswith("/") or pattern.split("/")[0] == "..":
            problems.append(f"{where}：模式 {pattern or '（空）'} 要写成相对仓根的路径")
            continue
        if not base.endswith(".md") or (LANGUAGE_SUFFIXED.search(base) and BROKEN != "registry-suffix-ignored"):
            problems.append(f"{where}：模式 {pattern} 带语言后缀或不以 .md 结尾，登记的是中文那一份 <名>.md")
            continue
        matches = sorted(path for path in glob.glob(pattern, root_dir=".", recursive=True)
                         if os.path.isfile(path) and not LANGUAGE_SUFFIXED.search(posixpath.basename(path)))
        if not matches and BROKEN != "registry-unmatched-ignored":
            problems.append(f"{where}：模式 {pattern} 指不到任何文件")
        entries.append({"line": line_number, "pattern": pattern, "regex": pattern_regex(pattern), "matches": matches})
    return entries, problems


def registered(entries, chinese_path):
    return any(entry["regex"].match(chinese_path) for entry in entries)


@lru_cache(maxsize=1)
def translation_paths():
    """Localized KB filenames retain their original three-language group identity."""
    path = ".claude/kb/translation-paths.json"
    if not os.path.isfile(path):
        return {}
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def original_translation_path(path):
    reverse = {value: key for key, value in translation_paths().items()}
    if path in reverse:
        return reverse[path]
    # Markdown links may be relative to a sibling page.
    names = {posixpath.basename(value): posixpath.basename(key) for key, value in translation_paths().items()}
    directory, name = posixpath.split(path)
    return posixpath.join(directory, names.get(name, name))


def language_paths(directory, name):
    return {language: translation_paths().get(posixpath.join(directory, name + suffix + ".md"), posixpath.join(directory, name + suffix + ".md")) for language, suffix in LANGUAGES}


def build_groups(entries):
    """登记模式指到的中文文件各成一组；模式所在目录里单有 .en.md 或 .ja.md 的也成一组。→ 按组名排序的列表。"""
    keys = set()
    directories = set()
    for entry in entries:
        for path in entry["matches"]:
            directory, base = posixpath.split(path)
            keys.add((directory, base[:-3]))
            directories.add(directory)
        if not GLOB_CHARACTERS.search(posixpath.dirname(entry["pattern"])):
            directories.add(posixpath.dirname(entry["pattern"]))
    if BROKEN != "reverse-ignored":
        for directory in sorted(directories):
            try:
                names = os.listdir(directory or ".")
            except OSError:
                continue
            for file_name in names:
                found = LANGUAGE_FILE.match(posixpath.basename(original_translation_path(posixpath.join(directory, file_name))))
                if found and registered(entries, posixpath.join(directory, found.group("name") + ".md")):
                    keys.add((directory, found.group("name")))
    groups = []
    for directory, name in sorted(keys):
        paths = language_paths(directory, name)
        groups.append({"key": posixpath.join(directory, name), "paths": paths,
                       "missing": [language for language, _ in LANGUAGES if not os.path.isfile(paths[language])]})
    return groups


def require_registry():
    entries, problems = load_registry()
    if entries is None:
        report_problems(f"没有三语文件登记表 {REGISTRY}",
                        [],
                        [f"建 {REGISTRY}：一行一条「<模式>  # 理由」，模式指中文那一份（例 README.md、briefs/*.md），理由不许省；"
                         "六格都从它取对象。"])
        return None, None
    return entries, problems


def judge_files():
    entries, registry_problems = require_registry()
    if entries is None:
        return 1
    groups = build_groups(entries)
    details = list(registry_problems)
    incomplete = 0
    for group in groups:
        if group["missing"]:
            incomplete += 1
            names = "、".join(f"{group['paths'][language]}（{language}）" for language in group["missing"])
            details.append(f"{group['key']}：缺 {names}")
    if details:
        advice = []
        if incomplete:
            advice.append("缺哪份补哪份：<名>.md 是中文（默认），<名>.en.md 是英语，<名>.ja.md 是日语，三份同时在、同时改；")
        if registry_problems:
            advice.append(f"登记表 {REGISTRY} 一行一条「<模式>  # 理由」，模式指中文那一份（相对仓根）、要指到文件、理由不许省；"
                          "不再登记的整行删掉。")
        return report_problems(f"三语文件有 {len(details)} 处不对（登记表 {len(registry_problems)} 处，三份没配齐的 {incomplete} 组，共 {len(groups)} 组）",
                               details, advice)
    if not groups:
        print("  ! 登记表里没有一条模式指到文件，这一格无对象可判")
        return 77
    print(f"  ✓ 查了登记表 {len(entries)} 条，指到 {len(groups)} 组，每组三份（{len(groups) * 3} 份文件）都在；"
          "登记目录里单有 .en.md 或 .ja.md 的也归在这些组里")
    return 0


def analyze(text):
    """→ 编号集合（不数链接目标 ](…) 里的）、链接目标集合（.en.md / .ja.md 归成 .md）、各级标题个数、表格行个数、围栏个数。"""
    identifier = IDENTIFIER_WORD_BOUNDARY if BROKEN == "word-boundary" else IDENTIFIER
    identifier_text = text if BROKEN == "link-target-ids-counted" else LINK_TARGET.sub("]()", text)
    identifiers = set(identifier.findall(identifier_text))
    links = set()
    for target in LINK_TARGET.findall(text):
        words = target.split()
        if words:
            link = words[0].strip("<>")
            if BROKEN != "link-language-kept":
                link = LINK_LANGUAGE_SUFFIX.sub(".md", original_translation_path(link))
            links.add(link)
    headings = {level: 0 for level in range(1, 7)}
    table_rows = fences = 0
    open_fence = None
    for line in text.splitlines():
        if open_fence is not None:
            closing = FENCE_CLOSE.match(line)
            if closing and closing.group(1)[0] == open_fence[0] and len(closing.group(1)) >= open_fence[1]:
                open_fence = None
            if BROKEN != "fence-blind":
                continue
        else:
            opening = FENCE_OPEN.match(line)
            if opening:
                fences += 1
                open_fence = (opening.group(1)[0], len(opening.group(1)))
                continue
        heading = HEADING.match(line)
        if heading:
            headings[len(heading.group(1))] += 1
        if line.lstrip().startswith("|"):
            table_rows += 1
    return {"ids": identifiers, "links": links, "headings": headings, "table_rows": table_rows, "fences": fences}


def set_difference_note(name, sets):
    union = set().union(*sets.values())
    lacks = []
    for language, _ in LANGUAGES:
        missing = sorted(union - sets[language])
        if missing:
            shown = "、".join(missing[:8]) + (f" 等 {len(missing)} 个" if len(missing) > 8 else "")
            lacks.append(f"{language} 缺 {shown}")
    sizes = "、".join(f"{language} {len(sets[language])} 个" for language, _ in LANGUAGES)
    return f"{name}不同：{sizes}；{'；'.join(lacks)}"


def compare_structure(infos):
    """三份的分析结果逐项比，→ 不同项的说明列表。"""
    notes = []
    languages = [language for language, _ in LANGUAGES]
    if BROKEN != "ids-ignored" and len({frozenset(infos[language]["ids"]) for language in languages}) > 1:
        notes.append(set_difference_note("编号集合", {language: infos[language]["ids"] for language in languages}))
    if BROKEN != "links-ignored" and len({frozenset(infos[language]["links"]) for language in languages}) > 1:
        notes.append(set_difference_note("链接目标集合", {language: infos[language]["links"] for language in languages}))
    if BROKEN != "headings-ignored":
        for level in range(1, 7):
            counts = {language: infos[language]["headings"][level] for language in languages}
            if len(set(counts.values())) > 1:
                notes.append(f"{'#' * level} 标题个数不同：" + "，".join(f"{language} {counts[language]}" for language in languages))
    for key, label, switch in (("table_rows", "表格行个数", "table-rows-ignored"), ("fences", "围栏代码块个数", "fences-ignored")):
        if BROKEN == switch:
            continue
        counts = {language: infos[language][key] for language in languages}
        if len(set(counts.values())) > 1:
            notes.append(f"{label}不同：" + "，".join(f"{language} {counts[language]}" for language in languages))
    return notes


def complete_groups(entries):
    groups = build_groups(entries)
    return [group for group in groups if not group["missing"]], [group for group in groups if group["missing"]]


def judge_structure():
    entries, _ = require_registry()
    if entries is None:
        return 1
    judged, skipped = complete_groups(entries)
    details = []
    for group in judged:
        infos = {language: analyze(read_text(group["paths"][language])) for language, _ in LANGUAGES}
        for note in compare_structure(infos):
            details.append(f"{group['key']}：{note}")
    if details:
        different = len({detail.split("：", 1)[0] for detail in details})
        return report_problems(f"三份的结构对不上：{different} 组里 {len(details)} 项不同（查了 {len(judged)} 组）",
                               details,
                               ["三份说的是同一件事，编号、链接目标、各级标题个数、表格行个数、围栏代码块个数都要一样：照点名的组和项，"
                                "把落后的那几份改到与改了的那份一致（改动是另一份多出来的内容就补译，是误删就恢复）；",
                                "围栏代码块里的 # 开头行与 | 开头行不算标题与表格行。"])
    if not judged:
        print(f"  ! 没有三份齐全的组可比（缺份的 {len(skipped)} 组归 trilingual-files），这一格无对象可判")
        return 77
    print(f"  ✓ 查了 {len(judged)} 组三份的结构：编号集合、链接目标集合、各级标题个数、表格行个数、围栏代码块个数都对得上；"
          f"缺份没比的 {len(skipped)} 组归 trilingual-files")
    return 0


def judge_changed_together(changed_paths):
    entries, _ = require_registry()
    if entries is None:
        return 1
    touched = {}
    for path in changed_paths:
        path = original_translation_path(path)
        found = CHANGED_FILE.match(posixpath.normpath(path)) if path else None
        if not found or (found.group("suffix") is None and LANGUAGE_SUFFIXED.search(path)):
            continue
        chinese = found.group("directory") + found.group("name") + ".md"
        if not registered(entries, chinese):
            continue
        language = {None: "zh", ".en": "en", ".ja": "ja"}[found.group("suffix")]
        touched.setdefault(found.group("directory") + found.group("name"), set()).add(language)
    if not touched:
        print("  ! 这次改动没碰到登记的三语文件，这一格无对象可判")
        return 77
    details = []
    translation_only = 0
    for key in sorted(touched):
        if "zh" not in touched[key] and BROKEN != "translation-only-required":
            translation_only += 1
            continue
        have = [language for language, _ in LANGUAGES if language in touched[key]]
        lacking = [language for language, _ in LANGUAGES if language not in touched[key]]
        if lacking and BROKEN != "partial-ignored":
            directory, name = posixpath.split(key)
            paths = language_paths(directory, name)
            details.append(f"{key}：碰了 {'、'.join(paths[language] for language in have)}，没碰 {'、'.join(paths[language] for language in lacking)}")
    chinese_touched = len(touched) - translation_only
    if details:
        return report_problems(f"这次改动碰了 {len(touched)} 组三语文件，改了中文的 {chinese_touched} 组里 {len(details)} 组没碰全三份；"
                               f"只改译文的 {translation_only} 组放行",
                               details,
                               ["改了中文那一份就要三份一起改：把没碰的译文照中文的改动同步译过去，再重跑；",
                                "只改译文、中文那一份没动的组不要求同改（译得对不对归 trilingual-structure、trilingual-glossary、trilingual-untranslated 三格）；",
                                "这一组其实不该改动的，把误碰的那一份改回去（git diff 看碰了什么）。"])
    print(f"  ✓ 这次改动碰到的 {len(touched)} 组三语文件：改了中文的 {chinese_touched} 组每组三份都碰了；"
          f"只改译文的 {translation_only} 组放行（译得对不对归 trilingual-structure、trilingual-glossary、trilingual-untranslated 三格）")
    return 0


def split_table_row(line):
    text = line.strip()
    if text.startswith("|"):
        text = text[1:]
    if text.endswith("|") and not text.endswith("\\|"):
        text = text[:-1]
    return [cell.strip().replace("\\|", "|") for cell in UNESCAPED_PIPE.split(text)]


def parse_glossary(text):
    """→ (可用于核译文的行 [(行号, 中文, English, 日本語)], 毛病 [(行号或 None, 说明)])。"""
    lines = text.split("\n")
    problems, terms = [], []
    markers = [index for index, line in enumerate(lines) if line.strip() == GLOSSARY_MARKER]
    if not markers:
        if BROKEN != "glossary-marker-ignored":
            problems.append((None, f"找不到单独一行的 {GLOSSARY_MARKER} 标记"))
        return terms, problems
    if len(markers) > 1:
        problems.append((markers[1] + 1, f"标记 {GLOSSARY_MARKER} 出现了 {len(markers)} 次，只许一处"))
    marker = markers[0]
    if marker + 1 >= len(lines) or not lines[marker + 1].lstrip().startswith("|"):
        problems.append((marker + 1, "标记下面不是表：表头要紧挨着标记那一行"))
        return terms, problems
    header = split_table_row(lines[marker + 1])
    if header != GLOSSARY_HEADER and BROKEN != "glossary-header-ignored":
        problems.append((marker + 2, f"表头不对：要 | {' | '.join(GLOSSARY_HEADER)} |，实际 | {' | '.join(header)} |"))
    separator_line = lines[marker + 2] if marker + 2 < len(lines) else ""
    separator = split_table_row(separator_line) if separator_line.lstrip().startswith("|") else []
    if not separator or not all(TABLE_SEPARATOR_CELL.match(cell) for cell in separator):
        problems.append((marker + 3, "表头下面不是分隔行（| --- | --- | --- | --- |），标记下面不是表"))
        return terms, problems
    first_seen = {}
    row_count = 0
    index = marker + 3
    while index < len(lines) and lines[index].lstrip().startswith("|"):
        number = index + 1
        cells = split_table_row(lines[index])
        index += 1
        row_count += 1
        if BROKEN != "glossary-columns-ignored":
            if len(cells) != 4:
                problems.append((number, f"表每行要四列，这一行有 {len(cells)} 列"))
                continue
            for name, cell in zip(GLOSSARY_COLUMN_NAMES, cells[:3]):
                if not cell:
                    problems.append((number, f"{name}列是空的"))
        elif len(cells) != 4:
            continue
        if cells[0]:
            if cells[0] in first_seen and BROKEN != "glossary-duplicate-ignored":
                problems.append((number, f"中文列「{cells[0]}」与第 {first_seen[cells[0]]} 行重复"))
            first_seen.setdefault(cells[0], number)
        note = cells[3]
        if note and BROKEN != "glossary-note-ignored":
            segments = note.split("<br>")
            if len(segments) != 3 or any(not segment.strip() for segment in segments):
                problems.append((number, f"说明列不空就要恰好三段、用 <br> 分隔、每段不空，这一行有 {len(segments)} 段"
                                         f"（空段 {sum(1 for segment in segments if not segment.strip())} 段）"))
        if cells[0] and cells[1] and cells[2]:
            terms.append((number, cells[0], cells[1], cells[2]))
    if row_count == 0:
        problems.append((marker + 3, "表里一行数据都没有"))
    return terms, problems


def load_longer_words(terms):
    """→ ({术语: [更长的词, ...]}, 毛病 [(行号, 说明)], 登记条数)。登记文件不在，或开了 glossary-substring：没有排除。"""
    if BROKEN == "glossary-substring" or not os.path.isfile(LONGER_WORDS):
        return {}, [], 0
    known = {chinese for _, chinese, _, _ in terms}
    longer, problems, seen, count = {}, [], set(), 0
    for number, raw in enumerate(read_text(LONGER_WORDS).split("\n"), 1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        count += 1
        cells = [cell.strip() for cell in raw.split("\t")]
        if len(cells) != 3 or not all(cells):
            problems.append((number, "每行要三列、用制表符分隔、都不空：术语、包含它的更长的词、理由"))
            continue
        term, word, reason = cells
        if term not in known:
            problems.append((number, f"术语「{term}」不是术语表中文列里的术语"))
        elif term not in word or term == word:
            problems.append((number, f"「{word}」要是包含术语「{term}」、又比它长的词"))
        elif len(reason) < MINIMUM_REASON_CHARACTERS:
            problems.append((number, f"理由至少 {MINIMUM_REASON_CHARACTERS} 个字"))
        elif (term, word) in seen:
            problems.append((number, f"「{term}」与「{word}」这一对写了两次"))
        else:
            seen.add((term, word))
            longer.setdefault(term, []).append(word)
    return longer, problems, count


def term_used(text, term, longer_words):
    """term 在 text 里至少有一处不落在登记的更长词（longer_words）的某一次出现里 → True。落在里面指位置对得上：词里术语所在的那几个字就是这一处。"""
    covered = set()
    for word in longer_words:
        offsets = [at for at in range(len(word) - len(term) + 1) if word.startswith(term, at)]
        start = word_at = 0
        while (word_at := text.find(word, start)) >= 0:
            covered.update(word_at + offset for offset in offsets)
            start = word_at + 1
    start = 0
    while (found := text.find(term, start)) >= 0:
        if found not in covered:
            return True
        start = found + 1
    return False


@lru_cache(maxsize=1)
def sweep_term():
    """把 scripts/kb/sweep-term.py 载成模块：改名登记表在哪、标记、怎么切格、哪几行是一对旧名新名，都照它（门禁 doc-text 的 term-renames 格调的就是它）。"""
    directory = os.path.dirname(TERM_RENAMES_SCRIPT)
    if directory not in sys.path:
        sys.path.insert(0, directory)
    spec = importlib.util.spec_from_file_location("sweep_term", TERM_RENAMES_SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def renames_columns_blanked(text):
    """改名登记表中文那一份：登记表数据行的旧名、新名两列换成空的，别的列与表外的正文原样 → (换过的文本, 换了几行)。
    这两列是 term-renames 格拿去全仓搜的字面，规定不译不改，在 zh 里出现不算用到术语。
    哪几行是数据行照 sweep-term.py 的 load_renames（标记之后以 | 起头、表头与分隔行不算），切格照它的 table_cells；
    load_renames 读不出这张表（它打出错再退出）时一行不换，整份照核，表的毛病归 term-renames 格报。"""
    module = sweep_term()
    if module.TABLE_MARK not in text:
        return text, 0
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            pairs = {(old, new) for old, new, _ in module.load_renames(".")}
    except SystemExit:
        return text, 0
    mark = text.index(module.TABLE_MARK)
    rows, blanked = text[mark:].split("\n"), 0
    for index, row in enumerate(rows):
        if not row.startswith("|"):
            continue
        cells = module.table_cells(row)
        if len(cells) >= 2 and (cells[0], cells[1]) in pairs:
            rows[index] = "| | | " + " | ".join(cells[2:]) + " |"
            blanked += 1
    return text[:mark] + "\n".join(rows), blanked


def judge_glossary():
    entries, _ = require_registry()
    if entries is None:
        return 1
    if not os.path.isfile(GLOSSARY):
        return report_problems(f"没有翻译术语表 {GLOSSARY}", [],
                               [f"建 {GLOSSARY}：表上方单独一行 {GLOSSARY_MARKER}，紧挨着表，表头 | {' | '.join(GLOSSARY_HEADER)} |；"
                                "译文照它用词，这一格才有对象可判。"])
    terms, problems = parse_glossary(read_text(GLOSSARY))
    details = [f"{GLOSSARY}:{number}：{message}" if number else f"{GLOSSARY}：{message}" for number, message in problems]
    longer_words, longer_problems, longer_count = load_longer_words(terms)
    details += [f"{LONGER_WORDS}:{number}：{message}" for number, message in longer_problems]
    judged, skipped = complete_groups(entries)
    hits = inside_longer_words = renames_rows_blanked = 0
    for group in judged:
        texts = {language: read_text(group["paths"][language]) for language, _ in LANGUAGES}
        english_folded = texts["en"] if BROKEN == "glossary-case-sensitive" else texts["en"].casefold()
        chinese_text = texts["zh"]
        if group["paths"]["zh"] == sweep_term().RENAMES_PATH and BROKEN != "glossary-renames-columns-counted":
            chinese_text, renames_rows_blanked = renames_columns_blanked(chinese_text)
        for number, chinese, english, japanese in terms:
            if BROKEN != "glossary-zh-gate-ignored":
                if chinese not in chinese_text:
                    continue
                if not term_used(chinese_text, chinese, longer_words.get(chinese, ())):
                    inside_longer_words += 1
                    continue
            hits += 1
            if BROKEN == "glossary-translation-ignored":
                continue
            wanted_english = english if BROKEN == "glossary-case-sensitive" else english.casefold()
            if wanted_english not in english_folded:
                details.append(f"{group['key']}：中文「{chinese}」在 zh 里出现，en 里找不到「{english}」（术语表第 {number} 行）")
            if japanese not in texts["ja"]:
                details.append(f"{group['key']}：中文「{chinese}」在 zh 里出现，ja 里找不到「{japanese}」（术语表第 {number} 行）")
    if details:
        return report_problems(f"翻译术语表与译文有 {len(details)} 处不对（表自己 {len(problems)} 处；查了 {len(judged)} 组、{len(terms)} 条术语）",
                               details,
                               [f"表的毛病照点名的行改 {GLOSSARY}（每行四列，前三列不空，中文列不重复，说明列空着或恰好三段用 <br> 分隔）；",
                                "译文的毛病改译文：zh 里用了某个术语，en 和 ja 里就要用术语表给的译名，三份都改；",
                                "术语表里的译名本身要改的，改表，再把三份里用到它的地方一起改；",
                                f"zh 里那一处其实是一个更长的词的一部分（例：「集合表示」里的「合表」），不是用到这个术语：把那个更长的词登记进 {LONGER_WORDS}"
                                "（一行「<术语><制表符><更长的词><制表符><理由>」），不要改译文去凑；登记文件自己的毛病照点名的行改。"])
    if not judged:
        print(f"  ! 没有三份齐全的组可核译文（缺份的 {len(skipped)} 组归 trilingual-files）；术语表本身 {len(terms)} 条写对了，这一格无对象可判")
        return 77
    print(f"  ✓ 术语表 {len(terms)} 条写对了；{len(judged)} 组三份里中文术语在 zh 出现 {hits} 次，对应的英文、日文译名都在 en、ja 里；"
          f"缺份没核的 {len(skipped)} 组归 trilingual-files；"
          f"另有 {inside_longer_words} 项（某组里的某条术语）只出现在登记的更长词里（{LONGER_WORDS} 登记 {longer_count} 条），没核译名；"
          f"改名登记表 {sweep_term().RENAMES_PATH} 的旧名、新名两列（{renames_rows_blanked} 行）不算用到术语")
    return 0


UNTRANSLATED_EXEMPT = ".claude/gate.d/trilingual-untranslated-exempt.tsv"
ENGLISH_HAN_THRESHOLD = 15            # 英语译文一行剩下这么多个汉字就红；取值的量法在文件头「判法细节」的 untranslated 一段
JAPANESE_SIMPLIFIED_THRESHOLD = 3     # 日语译文一行剩下这么多个日语不用的简体字就红；同上
JAPANESE_THRESHOLD_BEFORE = 5         # 收到 3 之前的日语门槛，只给弄坏开关 trilingual-untranslated-japanese-threshold-five 退回用
IDENTICAL_THRESHOLD = 1               # 与中文那一份某一行逐字相同的译文行，剩下这么多个被数的字就红（英语日语同一个数）；同上
SNIPPET_CHARACTERS = 40               # 红行里的片段从剩下部分第一个被数的字起取这么多个字
# 日语不用的简体中文字：GB 2312 收的汉字里 JIS X 0208 没收的那 3432 个，按码位排。怎么来的、怎么重新生成，见文件头「判法细节」的 untranslated 一段。
SIMPLIFIED_ONLY = frozenset(
    "丌专业丛东丝丢两严丧丨丫丬丰临为丽举乇么义乌乐乒乓乔乜习乡书乩买亍亏亓亚产亩亲亵亻亿仃仅仉仑仓仡仨仪仫们仳仵份仿众优伙伛"
    "伞伟传伢伤伥伦伧伪伫伲佘佟你佣佤佥佧佬佴佾侄侉侔侗侣侥侦侧侨侩侪侬俅俏俜俞俦俨俩俪俭俱倌倘倜倮债值倾偌偎偷偻偾偿傈傣傥傧"
    "储傩傺傻僦僬僳儆儇儋兑兕兖兰关兴兹养兽冁冈军农冯冻冼净凇减凑凤凫凯凳击凼凿刁刂刍划刖刘则刚创删刨别刭刽刿剀剁剂剐剑剜剡剧"
    "劁劂劐劓劝办务劢动劲劳势勋勐勖勰匀匦匮匾华协单卖卟卡卢卣卤卧卫卺厅历厉压厌厍厕厝厢厣县叁发变叠另叵叹叻叼叽吆吒吓吕吖吗吞"
    "吡吣吧吨启吱吲吴吵呃呋呐呒呓呔呕呖呗员呙呛呜呢呤呦呲呸咂咔咕咖咙咚咛咝咣咦咧咩咪咭咱咴咻咿哆哌响哎哏哐哑哒哓哔哕哗哙哚哜"
    "哝哞哟哧哪哳哼哿唁唉唑唛唠唢唣唤唧唪唬唰唷唼唿啁啃啉啊啐啕啡啤啥啦啧啪啬啭啮啵啶啷啸喁喂喈喏喑喔喱喳喵喷喹喻喽喾嗉嗌嗍嗑"
    "嗒嗓嗖嗝嗡嗥嗦嗨嗪嗫嗬嗯嗲嗳嗵嘀嘁嘈嘌嘎嘏嘞嘟嘣嘤嘧嘬嘭嘹嘻嘿噍噔噗噘噙噜噢噩噱噶噻噼嚅嚎嚓嚣嚯嚷囊囔囝囟囡团囤囫园囱围"
    "囵图圆圊圣圩圪圬圮圯圳圹场圾坌坍块坚坛坜坝坞坟坠坨坫坭坯坳坶坷坻坼垃垄垅垆垌垒垛垡垦垧垩垫垭垮垲垴垸埂埏埕埘埙埚埝埤埭埯"
    "埸埽堇堍堑堞堠塄塌塍塥塬墀墁墉墒墙墚增墩墼壳壶处备复夔够夤夯头夹夺夼奁奂奋奖奶她妆妇妈妒妗妞妤妩妪妫妮妯姊姒姗姘姝姣姬姹"
    "娄娅娆娇娈娌娓娣娱娲娴婊婕婧婴婵婶婷婺媪媲媳媵媸嫒嫔嫘嫜嫠嫫嫱嬗嬴嬷孓孙孢孪孬孽宁宄宓实宠审宪宫宽宾对寻导尔尕尘尜尝尥尧"
    "尬尴层屉屙屣屦屺屿岁岂岈岍岖岗岘岙岚岛岜岢岣岭岵岽岿峁峄峋峒峤峥峦崂崃崆崞崤崦崧崭崮崴崽崾嵇嵊嵘嵛嵝嵫嵴嶙巅巢巩巯币帅师"
    "帏帐帔帕帘帜带帧帮帱帻帼幂幛幞庀庆庋庐庑库应庙庞废庥庳庹庾廑廒廪开异弈张弪弹强彀彐归录彘彤彻徉徕徜徵德忄忆忉忏忐忑忒忡忧"
    "忪忭忮忾怀态怂怃怄怅怆怊怍怔怵总怼怿恝恧恳恶恸恹恺恻恼恽恿悝您悫悬悭悯悱悻惊惋惕惝惦惩惫惬惭惮惯愠愣愤愦愫慑憋憝憨憷懂懑"
    "懒懔懵戆戋戏戕戗战戢戤戥戬户戽戾扃扌扑扒扔扦执扩扪扫扬扭扯扰扳抚抟抠抡抢护报抨抿拄拎拖拚拟拢拣拥拦拧拨择拴拼拽挎挖挚挛挝"
    "挞挠挡挢挣挤挥挪挲挹捂捃捅捆捋捎捞损捡换捣捭捱掂掇掊掐掭掮掰掳掷掸掺掼揍揎揞揠揪揭揲揸揽揿搀搁搂搅搋搌搐搔搛搞搠搡搪搽搿"
    "摁摄摅摆摇摈摊摒摔摞摭摹撂撄撅撇撑撖撙撬撵撷撸撺擀擎擐擗擞擤攉攒攥攮敉敌敛敫斋斓斩旎旖旮旯旰时旷昀昕昙昝昱显晌晓晔晕晖晗"
    "晚晡晷晾暂暌暧暨曛朊朐术朵杀杂权杈杌杨杩极构枘枞枣枥枧枨枪枫枭枰枵柃柒柙柜柠查柰柽栀栅标栈栉栊栋栌栎栏树栝栳样栾桄桅桉桊"
    "桌桕桠桡桢桤桥桦桨桩桫梆检棂棰棱棵棼椐椟椠椤椭楂楗楣楦楱榀榄榆榇榈榉榍榘榨榫榭榷槔槛槟槠樘樨樯樱樾橐橛橥橱橹橼檑檩檫欢欤"
    "歆步歧歼殁殂殇殒殓殚殛殡毁毂每毕毖毗毙毡毪毵毹毽氅氆氇氍氐氕氖氘氙氚氟氡氢氦氧氨氩氪氮氯氰氲氵氽汆汇汉汊汔汛汜污汤汩汴汶"
    "汹沅沆沉沏沔沟沣沤沥沦沧沩沪沭沲泐泔泖泞泠泫泮泵泶泷泸泺泻泼泽泾洁洄洇洎洚洧洮洱洹洼浃浆浇浈浊测浍济浏浑浒浓浔浞浠浯浼涂"
    "涉涑涔涝涞涟涠涡涣涤润涧涨涩涪涫涮涿淄淖淝淠淼渌渍渎渐渑渔渖渗渲渴湄湓湔溃溅溆溉溧溱溴溻滁滇滏滗滚滟滠满滢滤滥滦滨滩滹漤"
    "漩漪漭漯漳漶潆潇潋潍潞潢潲澈澉澌澍澜澧澶濉濑濒濞瀣瀵瀹灏灞灬灭灵灶灾灿炀炅炔炕炖炜炝炫炱炷炻炼炽烀烁烂烃烊烘烛烤烦烧烨烩"
    "烫烬热烯烷焊焐焓焕焖焘焯焰焱煅煊煜煞煨煲煳煸煺熘熠熳熵爝爱爷爸爹牍牖牦牮牯牵牺牿犄犊犋犍犏犟犭犰犴犷犸狁狈狍狞狨狮狯狰狱"
    "狲狳狴狺狻猁猃猎猓猕猞猡猢猬猱猸猹獍獐獒獠獬獭獯獾玎玑玛玟玢玫玮环现玷玺珉珏珐珑珙珧珩珲琊琏琐琚琛琦琨琪琬琮琰琼瑗瑭瑷璀"
    "璁璇璎璐璜璨璩璺瓒瓞瓤瓯瓴瓿甏甙甩甭甯电甾畀畅畈畎畲畹疃疒疖疗疙疟疠疡疤疬疮疯疰疴痄痈痉痖痤痦痧痨痪痫痱痹瘀瘃瘅瘊瘌瘐瘕"
    "瘗瘘瘙瘛瘥瘦瘩瘪瘫瘭瘳瘵瘸瘼瘾瘿癀癃癍癔癞癣癫癯皂皑皤皱皲盅盎盏盐监盔盘盯盱盹盼眍眙眚眢眨眭眯眵眶睁睃睐睑睢睬睽瞀瞄瞅瞌"
    "瞍瞒瞟瞢瞧瞩瞪瞵矫矬矶矸矽矾矿砀码砉砍砑砖砗砘砚砜砝砟砣砩砬砭砰砷砸砹砻砼砾础硇硌硎硐硒硕硖硗硪硭确硷碉碘碛碜碟碡碥碰碱"
    "碲碳碴碹磉磕磙磲磷磺礅礓礞礤礴礻祆祛祜祧祯祸禚离秃秆种秫秭积秸秽稂稃稆稞稣稳稹穑穷穸窀窆窍窑窜窝窠窥窦窨窬窭窳竖竞竽笃笔"
    "笕笤笪笫笮笱笸笺笼笾筇筘筚筛筠筢筲筷筹筻签简箅箐箢箦箧箨箩箫箬篑篓篙篚篡篪篮篱篼篾簋簌簏簖簦籁籴类籼籽粑粜粝粞粪粼糁糇糈"
    "糌糍糕糗糙糨紧絷綦繇纟纠纡红纣纤纥约级纨纩纪纫纬纭纯纰纱纲纳纵纶纷纸纹纺纽纾线绀绁绂练组绅细织终绉绊绋绌绍绎经绐绑绒结绔"
    "绕绗绘给绚绛络绝绞统绠绡绢绣绥绦继绨绩绪绫续绮绯绰绱绲绳维绵绶绷绸绺绻综绽绾绿缀缁缂缃缄缅缆缇缈缉缋缌缍缎缏缑缒缓缔缕编"
    "缗缘缙缚缛缜缝缟缠缡缢缣缤缥缦缧缨缩缪缫缬缭缮缯缰缱缲缳缴缵罂罄罗罚罡罢罱罴罾羁羟羡羧羰羼羿翎翘翟翥翮翱耍耔耖耠耢耥耦耧"
    "耩耪耱耵耷耸聂聃聋职聍联聩聪聱肀肃肜肟肠肤肫肮肷肼肽肾肿胀胁胂胍胗胜胧胨胩胪胫胬胭胰胲胳胶胺脍脎脏脐脑脒脓脔脖脘脞脬脲脶"
    "脸腈腊腌腙腚腠腧腩腭腻腼腽腾膑膘膛膦膪膻臁臃臊臌臬舀舄舆舔舡舢舣舨舭舯舰舱舴舻舾艄艉艋艏艰艳艴艹艺艽艿节芄芈芊芎芏芑芗芘"
    "芜芡芤芨芩芪芮芰芴芷芾苁苄苇苈苊苋苌苍苎苏苕苘苠苤苯苷茁茇茈茌茏茑茔茕茚茛茧茬茭茳茺茼荃荆荇荑荔荚荛荜荞荟荠荡荣荤荥荦荧"
    "荨荩荪荫荬荭荮药荸荽莆莒莘莛莜莩莰莲莳莴莶获莸莹莺莼菀菇菏菔菝菡菥菪菸菹萁萆萏萑萘萜萝萤营萦萧萨葑葙葚葜葳葶葸蒇蒈蒉蒌蒎"
    "蒗蒯蒴蒺蒽蓓蓝蓟蓠蓣蓥蓦蓰蔌蔫蔷蔸蔹蔺蔻蔼蕖蕙蕞蕤蕲蕴蕹蕺蕻薅薏薰薷藓藿蘅蘑蘧蘩蘸蘼虏虑虢虬虮虺虼虽虾虿蚀蚁蚂蚍蚜蚝蚧蚨"
    "蚬蚱蚴蚵蚺蛀蛊蛏蛐蛑蛘蛰蛱蛲蛳蛴蜇蜓蜕蜗蜞蜡蜢蜣蜮蜱蜾蝇蝈蝤蝥蝰蝻蝼蝽蝾螃螅螈螋螓螗螨螬螭螵蟊蟑蟓蟛蟥蟪蟮蠃蠊蠓蠛蠲蠼衅"
    "衔衤补衩衬衮袄袅袜袭袼裆裉裎裒裢裣裤裥裰裱褙褚褛褡褰褴襻见观规觅视觇览觉觊觋觌觎觏觐觑觖觞觥觫觯觳訇訾詹誊讠计订讣认讥讦"
    "讧讨让讪讫训议讯记讲讳讴讵讶讷许讹论讼讽设访诀证诂诃评诅识诈诉诊诋诌词诎诏译诒诓诔试诖诗诘诙诚诛诜话诞诟诠诡询诣诤该详诧"
    "诨诩诫诬语诮误诰诱诲诳说诵诶请诸诹诺读诼诽课诿谀谁谂调谄谅谆谇谈谊谋谌谍谎谏谐谑谒谓谔谕谖谗谘谙谚谛谜谝谟谠谡谢谣谤谥谦"
    "谧谨谩谪谫谬谭谮谯谰谱谲谳谴谵谶豇豉豳贝贞负贡财责贤败账货质贩贪贫贬购贮贯贰贱贲贳贴贵贶贷贸费贺贻贼贽贾贿赀赁赂赃资赅赆"
    "赇赈赉赊赋赌赍赎赏赐赓赔赕赖赘赙赚赛赜赝赞赠赡赢赣赵赶趄趋趑趔趟趱趴趵趸趼趿跃跄跆跎跑跗跞跤跬跷跸跹跺跻跽踅踌踔踢踣踩踬"
    "踮踯踱踹踺踽蹀蹁蹋蹑蹒蹦蹩蹬蹭蹯蹰蹿躏躐躜躞躲躺軎车轧轨轩轫转轭轮软轰轱轲轳轴轵轶轷轸轹轺轻轼载轾轿辁辂较辄辅辆辇辈辉辊"
    "辋辍辎辏辐辑输辔辕辖辗辘辙辚辩辫辶边辽达迁过迈运迓迕还这进远违连迟迤迨迮迳适逄选逊递逛逦逭逯逻遄遗遛遢遴邈邋邓邕邗邙邛邝"
    "邡邢邬邮邰邳邴邶邹邺邻邾郄郅郇郏郐郑郓郗郜郝郦郧郫郯郴郸郾鄄鄞鄢鄣鄯鄱鄹酃酆酏酐酗酚酝酞酡酤酮酯酰酱酴酶酹酽酾酿醅醌醑醚"
    "醛醣醭醮释鉴銎銮鋈錾鍪鎏鏊鐾鑫钅钆钇针钉钊钋钌钍钎钏钐钒钓钔钕钗钙钚钛钜钝钞钟钠钡钢钣钤钥钦钧钨钩钪钫钬钭钮钯钰钱钲钳钴"
    "钵钶钷钸钹钺钻钼钽钾钿铀铁铂铃铄铅铆铈铉铊铋铌铍铎铐铑铒铕铖铗铘铙铛铜铝铞铟铠铡铢铣铤铥铧铨铩铪铫铬铭铮铯铰铱铲铳铴铵银"
    "铷铸铹铺铼铽链铿销锁锂锃锄锅锆锇锈锉锊锋锌锍锎锏锐锑锒锓锔锕锖锗锘错锚锛锝锞锟锡锢锣锤锥锦锨锩锪锫锬锭键锯锰锱锲锴锵锶锷"
    "锸锹锺锻锼锾锿镀镁镂镄镅镆镇镉镊镌镍镎镏镐镑镒镓镔镖镗镘镙镛镜镝镞镟镡镢镣镤镥镦镧镨镩镪镫镬镭镯镰镱镲镳镶长门闩闪闫闭问"
    "闯闰闱闲闳间闵闶闷闸闹闺闻闼闽闾阀阁阂阃阄阅阆阈阉阊阋阌阍阎阏阐阑阒阔阕阖阗阙阚阝队阢阱阳阴阵阶阼阽际陆陇陈陉陔陕陡陧陨"
    "险陴隐隳隽难雏雒雠雩雯雳雾霁霉霭靓靛靥靳靶鞑鞒鞔鞯鞲韦韧韩韪韫韬页顶顷顸项顺须顼顽顾顿颀颁颂颃预颅领颇颈颉颊颌颍颏颐频颓"
    "颔颖颗题颚颛颜额颞颟颠颡颢颤颥颦颧风飑飒飓飕飘飙飚飞飧飨餍饔饣饥饧饨饩饪饫饬饭饮饯饰饱饲饴饵饶饷饺饼饽饿馀馁馄馅馆馇馈馊"
    "馋馍馏馐馑馒馓馔馕马驭驮驯驰驱驳驴驵驶驷驸驹驺驻驼驽驾驿骀骁骂骄骅骆骇骈骊骋验骏骐骑骒骓骖骗骘骚骛骜骝骞骟骠骡骢骣骤骥骧"
    "骱骶骷骺髁髂髅髋髌髡髹鬃鬈鬏鬓魇魈魉鱼鱿鲁鲂鲅鲆鲇鲈鲋鲍鲎鲐鲑鲒鲔鲕鲚鲛鲜鲞鲟鲠鲡鲢鲣鲤鲥鲦鲧鲨鲩鲫鲭鲮鲰鲱鲲鲳鲴鲵鲶鲷"
    "鲸鲺鲻鲼鲽鳃鳄鳅鳆鳇鳊鳋鳌鳍鳎鳏鳐鳓鳔鳕鳖鳗鳘鳙鳜鳝鳞鳟鳢鸟鸠鸡鸢鸣鸥鸦鸨鸩鸪鸫鸬鸭鸯鸱鸲鸳鸵鸶鸷鸸鸹鸺鸽鸾鸿鹁鹂鹃鹄鹅"
    "鹆鹇鹈鹉鹊鹋鹌鹎鹏鹑鹕鹗鹘鹚鹛鹜鹞鹣鹤鹦鹧鹨鹩鹪鹫鹬鹭鹰鹱鹳鹾麂麇麴麽黉黑黟黢黧黩黪黾鼋鼍鼐鼗鼙鼢鼯鼷鼹鼽齄齐齑齿龀龃龄"
    "龅龆龇龈龉龊龋龌龙龚龛龟"
)
HAN_CHARACTER = re.compile(r"[㐀-䶿一-鿿豈-﫿\U00020000-\U0002fa1f]")
INLINE_CODE = re.compile(r"(`+)(.+?)(?<!`)\1(?!`)")
QUOTED_SEGMENTS = tuple(re.compile(pattern) for pattern in (r"「[^「」]*」", r"『[^『』]*』", r'"[^"]*"', r"“[^“”]*”"))
LINK_TARGET_SEGMENT = re.compile(r"\]\([^)]*\)")
PATH_TOKEN = re.compile(r"[\w.\-<>~*]*(?:/[\w.\-<>~*]+)+/?|[\w\-]+\.(?:md|rs|py|sh|tsv|json|txt|toml|log|out)\b")
ASCII_ALPHANUMERIC = re.compile(r"[A-Za-z0-9]")
ANY_FENCE_OPEN = re.compile(r"^[ \t>]*(`{3,}|~{3,})")
ANY_FENCE_CLOSE = re.compile(r"^[ \t>]*(`{3,}|~{3,})[ \t]*$")
WHITESPACE_RUN = re.compile(r"\s+")


def fenced_line_indexes(rows):
    """闭合了的围栏代码块（开、闭两行连同中间）所在的行下标。行首可带空白与 >：列表项、引用块里缩进的围栏也认（照录旧文件原文的块常这样写）；
    没有闭合行的开围栏不算围栏，后面的行照判，免得一处漏写的闭合把整份后半截藏起来。"""
    inside, index = set(), 0
    while index < len(rows):
        opening = ANY_FENCE_OPEN.match(rows[index])
        if opening:
            mark = opening.group(1)
            for close in range(index + 1, len(rows)):
                closing = ANY_FENCE_CLOSE.match(rows[close])
                if closing and closing.group(1)[0] == mark[0] and len(closing.group(1)) >= len(mark):
                    inside.update(range(index, close + 1))
                    index = close
                    break
        index += 1
    return inside


def untranslated_remainder(line, in_comment):
    """一行去掉该照中文原样留的段之后剩下的部分 → (剩下的正文, 行尾是否还在 HTML 注释里)。
    去掉的：HTML 注释（可跨行）、反引号段、Markdown 链接目标 ](…)、「」『』与英语直弯引号里的引文（由里往外反复去，套着的引号整段去净）、
    带 ASCII 字母或数字的路径（含 / 的一段，或以 .md、.rs 这类后缀收尾的文件名；全是汉字的「读/写」不当路径）。"""
    code_kept = BROKEN == "trilingual-untranslated-code-kept"
    text = line
    if not code_kept:
        visible, rest = [], line
        while rest:
            if in_comment:
                end = rest.find("-->")
                if end < 0:
                    break
                rest, in_comment = rest[end + 3:], False
            else:
                start = rest.find("<!--")
                if start < 0:
                    visible.append(rest)
                    break
                visible.append(rest[:start])
                rest, in_comment = rest[start + 4:], True
        text = INLINE_CODE.sub(" ", "".join(visible))
        text = LINK_TARGET_SEGMENT.sub("]", text)
    if BROKEN != "trilingual-untranslated-quotes-kept":
        previous = None
        while previous != text:
            previous = text
            for pattern in QUOTED_SEGMENTS:
                text = pattern.sub(" ", text)
    if not code_kept:
        text = PATH_TOKEN.sub(lambda found: " " if ASCII_ALPHANUMERIC.search(found.group(0)) else found.group(0), text)
    return text, in_comment


def load_untranslated_exempt(entries):
    """→ (不判的中文路径集合, 毛病 [(行号, 说明)], 登记条数)。登记文件不在，或开了 trilingual-untranslated-exempt-ignored：没有排除。"""
    if BROKEN == "trilingual-untranslated-exempt-ignored" or not os.path.isfile(UNTRANSLATED_EXEMPT):
        return set(), [], 0
    exempt, problems, count = set(), [], 0
    for number, raw in enumerate(read_text(UNTRANSLATED_EXEMPT).split("\n"), 1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        count += 1
        cells = [cell.strip() for cell in raw.split("\t")]
        if len(cells) != 2 or not all(cells):
            problems.append((number, "每行要两列、用制表符分隔、都不空：中文那一份的路径、理由"))
            continue
        path, reason = posixpath.normpath(cells[0]), cells[1]
        if LANGUAGE_SUFFIXED.search(posixpath.basename(path)) or not os.path.isfile(path) or not registered(entries, path):
            problems.append((number, f"{cells[0]} 不是登记表 {REGISTRY} 指到的、在的中文那一份 <名>.md"))
        elif len(reason) < MINIMUM_REASON_CHARACTERS:
            problems.append((number, f"理由至少 {MINIMUM_REASON_CHARACTERS} 个字"))
        elif path in exempt:
            problems.append((number, f"{path} 登记了两次"))
        else:
            exempt.add(path)
    return exempt, problems, count


def counted_characters(language, text):
    """英语数汉字，日语数日语不用的简体字 → 被数到的字在 text 里的下标列表。"""
    if language == "en":
        return [found.start() for found in HAN_CHARACTER.finditer(text)]
    return [index for index, character in enumerate(text) if character in SIMPLIFIED_ONLY]


def judge_untranslated():
    entries, _ = require_registry()
    if entries is None:
        return 1
    exempt, exempt_problems, exempt_count = load_untranslated_exempt(entries)
    details = [f"{UNTRANSLATED_EXEMPT}:{number}：{message}" for number, message in exempt_problems]
    languages = ("en",) if BROKEN == "trilingual-untranslated-japanese-unjudged" else ("en", "ja")
    japanese_threshold = JAPANESE_THRESHOLD_BEFORE if BROKEN == "trilingual-untranslated-japanese-threshold-five" else JAPANESE_SIMPLIFIED_THRESHOLD
    thresholds = {"en": (ENGLISH_HAN_THRESHOLD, "英语", "个汉字"), "ja": (japanese_threshold, "日语", "个日语不用的简体字")}
    judged, red_lines, per_file, findings = {"en": 0, "ja": 0}, {"en": 0, "ja": 0}, {}, []
    exempt_groups = identical_red = identical_passed = 0
    for group in build_groups(entries):
        if group["paths"]["zh"] in exempt:
            exempt_groups += 1
            continue
        # 中文那一份每一行 → 第一次出现的行号；与中文逐字相同那一条拿它比（中文那一份不在的组不比）
        chinese_lines = {}
        if BROKEN != "trilingual-untranslated-identical-ignored" and os.path.isfile(group["paths"]["zh"]):
            for number, chinese_row in enumerate(read_text(group["paths"]["zh"]).split("\n"), 1):
                chinese_lines.setdefault(chinese_row, number)
        for language in languages:
            path = group["paths"][language]
            if not os.path.isfile(path):
                continue
            judged[language] += 1
            threshold, language_name, unit = thresholds[language]
            rows = read_text(path).split("\n")
            fenced = set() if BROKEN == "trilingual-untranslated-code-kept" else fenced_line_indexes(rows)
            in_comment = False
            for index, row in enumerate(rows):
                if index in fenced:
                    continue
                remainder, in_comment = untranslated_remainder(row, in_comment)
                positions = counted_characters(language, remainder)
                identical_at = chinese_lines.get(row) if row.strip() else None
                if len(positions) >= threshold:
                    note = f"剩 {len(positions)} {unit}（门槛 {threshold}）"
                elif identical_at is not None and len(positions) >= IDENTICAL_THRESHOLD:
                    note = f"与中文逐字相同（{group['paths']['zh']}:{identical_at}），剩 {len(positions)} {unit}"
                    identical_red += 1
                else:
                    if identical_at is not None:
                        identical_passed += 1
                    continue
                snippet = WHITESPACE_RUN.sub(" ", remainder[positions[0]:]).strip()[:SNIPPET_CHARACTERS]
                findings.append(f"{path}:{index + 1}  {language_name}，{note}：{snippet}")
                red_lines[language] += 1
                per_file[path] = per_file.get(path, 0) + 1
    if details or findings:
        listing = [f"按文件：{path}  {count} 行" for path, count in sorted(per_file.items(), key=lambda item: (-item[1], item[0]))]
        return report_problems(
            f"三语译文里残留没译的中文超量：{len(findings)} 行（英语 {red_lines['en']} 行、日语 {red_lines['ja']} 行，在 {len(per_file)} 份译文里）；"
            f"排除登记表 {len(exempt_problems)} 处不对；查了英语 {judged['en']} 份、日语 {judged['ja']} 份译文，排除登记的 {exempt_groups} 组没判；"
            f"红行里 {identical_red} 行是与中文逐字相同、不到门槛而还剩中文的",
            details + listing + findings,
            ["照中文那一份同一行整行译，反引号、路径、链接目标逐字照中文（只把术语表里的词换成译名、句子还是中文的，同样整行重译）；",
             "与中文逐字相同的行是照抄没译：照中文那一份点名的那一行整行译（语言切换行加粗本语言、链另外两种，写法照 .claude/skills/trilingual/SKILL.md）；",
             "要原样留的中文（用户原话、变异名、文件名）放进「」、『』或反引号，路径与链接目标不译；",
             f"整份按设计三语并列的（翻译术语表、改名登记这类），把中文那一份登记进 {UNTRANSLATED_EXEMPT}"
             f"（一行「<中文那一份路径><制表符><理由>」，理由至少 {MINIMUM_REASON_CHARACTERS} 个字）；登记表自己的毛病照点名的行改。"])
    if not judged["en"] and not judged["ja"]:
        print(f"  ! 登记的三语文件里一份英语、日语译文都没有（排除登记的 {exempt_groups} 组不判），这一格无对象可判")
        return 77
    print(f"  ✓ 查了英语 {judged['en']} 份、日语 {judged['ja']} 份译文（排除登记 {exempt_count} 条、{exempt_groups} 组没判）："
          f"去掉反引号段、围栏代码块、引文、链接目标、路径与 HTML 注释之后，英语没有一行剩 {ENGLISH_HAN_THRESHOLD} 个以上汉字，"
          f"日语没有一行剩 {japanese_threshold} 个以上日语不用的简体字；"
          f"与中文逐字相同的行没有一行还剩中文（英语汉字、日语日语不用的简体字），这样的行放过 {identical_passed} 行")
    return 0


OMISSION_MINIMUM_MEASURE = 20         # 中文那一段内容量（汉字个数加 ASCII 词数）不到这么多，不比比值（短段的比值抖得厉害）；取值的量法在文件头「判法细节」的 omission 一段
OMISSION_RATIO_LIMITS = {"en": (0.42, 1.5), "ja": (0.9, 3.3)}   # 译文内容量 ÷ 中文内容量：低于前一个算过短（漏段、压成摘要），高于后一个算过长（对错了段）；同上
OMISSION_SNIPPET_CHARACTERS = 30      # 红行里的片段取中文那一段开头这么多个字
OMISSION_LANGUAGE_NAMES = {"en": "英语", "ja": "日语"}
KANA_CHARACTER = re.compile(r"[぀-ヿㇰ-ㇿｦ-ﾟ]")
ASCII_WORD = re.compile(r"[A-Za-z0-9]+(?:['’.\-][A-Za-z0-9]+)*")
LIST_ITEM_START = re.compile(r"^[ \t>]*(?:[-*+]|[0-9]+[.)])[ \t]+")
HTML_COMMENT = re.compile(r"<!--.*?-->", re.S)


def is_language_switch_line(line):
    """三语切换行（中文 · English · 日本語 三个链接或加粗，写法照 .claude/skills/trilingual/SKILL.md）：不算一段，前后断开。"""
    return "中文" in line and "日本語" in line and ("[English](" in line or "**English**" in line)


def omission_measure(language, text):
    """一段的内容量：先去掉反引号段、链接目标 ](…)、带 ASCII 字母或数字的路径（三份里照中文原样留的，不是译出来的量），
    中文数汉字个数加 ASCII 词数（一串字母数字算一个，数字也是这样数），英语数 ASCII 词数加汉字个数，日语数汉字与假名个数加 ASCII 词数。"""
    text = INLINE_CODE.sub(" ", text)
    text = LINK_TARGET_SEGMENT.sub("]", text)
    text = PATH_TOKEN.sub(lambda found: " " if ASCII_ALPHANUMERIC.search(found.group(0)) else found.group(0), text)
    words = len(ASCII_WORD.findall(text))
    han = len(HAN_CHARACTER.findall(text))
    if language == "ja":
        return han + len(KANA_CHARACTER.findall(text)) + words
    return han + words


def table_row_key(row):
    """表格行首列里第一个编号（C、D、E 加数字，I- 加数字与点；不数链接目标里的）→ 编号，首列没有编号 → None。"""
    cells = split_table_row(row)
    found = IDENTIFIER.search(LINK_TARGET.sub("]()", cells[0])) if cells else None
    return found.group(0) if found else None


def omission_units(text):
    """一份文件切成节与段 → (节列表, 按编号配对的表格行, 各节标题所在行号)。
    节：围栏外的 ATX 标题（判法同 structure 的 HEADING）把文件切开，第一个标题之前算一节（标题行号记 1）。
    段：节里的一段是空行隔开的连续几行，列表项（行首可带空白与 >）另起一段；表格行一行一段（分隔行不算）；
    闭合了的围栏代码块整块不算（判法同 untranslated 的 fenced_line_indexes，块前后断开）；HTML 注释先去掉（换成同样多的换行，行号不变）；
    三语切换行不算一段、前后断开。节内的段 [行号, 文本] 按出现次序放；首列带编号的表格行另放进 {(编号, 这个编号第几次出现): (行号, 文本)}。"""
    rows = HTML_COMMENT.sub(lambda found: "\n" * found.group(0).count("\n"), text).split("\n")
    fenced = fenced_line_indexes(rows)
    sections, headings, keyed, seen = [[]], [1], {}, {}
    current = None
    for index, row in enumerate(rows):
        if index in fenced or not row.strip() or is_language_switch_line(row):
            current = None
            continue
        if HEADING.match(row):
            sections.append([])
            headings.append(index + 1)
            current = None
            continue
        stripped = row.strip()
        if stripped.startswith("|"):
            current = None
            cells = split_table_row(stripped)
            if all(TABLE_SEPARATOR_CELL.match(cell) for cell in cells):
                continue
            key = None if BROKEN == "omission-keys-ignored" else table_row_key(stripped)
            if key:
                occurrence = seen.get(key, 0)
                seen[key] = occurrence + 1
                keyed[(key, occurrence)] = (index + 1, stripped)
            else:
                sections[-1].append([index + 1, stripped])
            continue
        if current is not None and not LIST_ITEM_START.match(row):
            current[1] += "\n" + row
        else:
            current = [index + 1, row]
            sections[-1].append(current)
    return sections, keyed, headings


def omission_snippet(text):
    return WHITESPACE_RUN.sub(" ", text).strip()[:OMISSION_SNIPPET_CHARACTERS]


def judge_omission():
    entries, _ = require_registry()
    if entries is None:
        return 1
    judged, skipped = complete_groups(entries)
    short, long, unpaired, per_file = [], [], [], {}
    compared = {"en": 0, "ja": 0}
    below_minimum = heading_mismatched = 0

    def red(bucket, path, line):
        bucket.append(line)
        per_file[path] = per_file.get(path, 0) + 1

    for group in judged:
        chinese_path = group["paths"]["zh"]
        chinese_sections, chinese_keyed, chinese_headings = omission_units(read_text(chinese_path))
        for language in ("en", "ja"):
            path = group["paths"][language]
            name = OMISSION_LANGUAGE_NAMES[language]
            low, high = OMISSION_RATIO_LIMITS[language]
            sections, keyed, headings = omission_units(read_text(path))
            if len(sections) != len(chinese_sections):
                heading_mismatched += 1
                continue
            pairs = []
            for chinese_units, units, chinese_heading, heading in zip(chinese_sections, sections, chinese_headings, headings):
                if len(chinese_units) == len(units):
                    pairs.extend(zip(chinese_units, units))
                elif BROKEN != "omission-unpaired-ignored":
                    chinese_total = sum(omission_measure("zh", unit[1]) for unit in chinese_units)
                    total = sum(omission_measure(language, unit[1]) for unit in units)
                    ratio = f"{total / chinese_total:.2f}" if chinese_total else "—"
                    red(unpaired, path, f"{path}:{heading}  {name}，这一节段数与中文不同：中文那一份 {chinese_path}:{chinese_heading} 起这一节 {len(chinese_units)} 段，"
                                        f"这一份 {len(units)} 段（整节内容量比 {ratio}）")
            for key in sorted(set(chinese_keyed) | set(keyed), key=lambda item: (item[0], item[1])):
                if key in chinese_keyed and key in keyed:
                    pairs.append((chinese_keyed[key], keyed[key]))
                elif BROKEN != "omission-unpaired-ignored":
                    ordinal = f"第 {key[1] + 1} 个"
                    if key in chinese_keyed:
                        red(unpaired, path, f"{chinese_path}:{chinese_keyed[key][0]}  {name}那一份 {path} 没有首列编号 {key[0]} 的表格行（{ordinal}）："
                                            f"{omission_snippet(chinese_keyed[key][1])}")
                    else:
                        red(unpaired, path, f"{path}:{keyed[key][0]}  {name}，首列编号 {key[0]} 的表格行（{ordinal}）中文那一份 {chinese_path} 没有")
            for (chinese_line, chinese_text), (line, text) in sorted(pairs, key=lambda pair: pair[1][0]):
                chinese_measure = omission_measure("zh", chinese_text)
                if chinese_measure < OMISSION_MINIMUM_MEASURE:
                    below_minimum += 1
                    continue
                compared[language] += 1
                if BROKEN == "omission-ratio-ignored":
                    continue
                measure = omission_measure(language, text)
                ratio = measure / chinese_measure
                if ratio < low or ratio > high:
                    kind, limit, bucket = ("过短", f"不到 {low}", short) if ratio < low else ("过长", f"超过 {high}", long)
                    red(bucket, path, f"{path}:{line}  {name}，内容量比 {ratio:.2f}（{limit}，{kind}）：中文那一份 {chinese_path}:{chinese_line} 量 {chinese_measure}、"
                                      f"这一份 {measure}：{omission_snippet(chinese_text)}")
    if short or long or unpaired:
        listing = [f"按文件：{path}  {count} 处" for path, count in sorted(per_file.items(), key=lambda item: (-item[1], item[0]))]
        return report_problems(
            f"三语译文与中文的内容量对不上：过短 {len(short)} 处、过长 {len(long)} 处、配不上 {len(unpaired)} 处（段数不同的节与编号行），在 {len(per_file)} 份译文里；"
            f"查了 {len(judged)} 组，比了英语 {compared['en']} 对、日语 {compared['ja']} 对；标题对不上没比的 {heading_mismatched} 份归 trilingual-structure",
            listing + short + long + unpaired,
            ["过短：照中文那一份点名的那一段整段译，漏了的那一段补上，压成摘要的照中文逐句译回去（不加不减；比值是译文内容量 ÷ 中文内容量，"
             "中文数汉字与 ASCII 词，英语数词，日语数汉字与假名）；",
             "过长：这一段多半与中文那一段对错了（中文删了或改了而译文没跟），照中文那一段改；",
             "段数不同的节：照中文那一节逐段对齐（漏了一段补译，多出来的删掉，两段并成一段的照中文拆开；中文自己少了空行、把两段连在一起的，在中文那一份补空行）；",
             "编号行配不上：补上缺的那一行的译文，或删掉中文那一份没有的那一行。"],
            limit=len(listing) + DETAIL_LIMIT)
    if not judged:
        print(f"  ! 没有三份齐全的组可比（缺份的 {len(skipped)} 组归 trilingual-files），这一格无对象可判")
        return 77
    print(f"  ✓ 查了 {len(judged)} 组三份：比了英语 {compared['en']} 对、日语 {compared['ja']} 对段的内容量，"
          f"英语在 {OMISSION_RATIO_LIMITS['en'][0]}–{OMISSION_RATIO_LIMITS['en'][1]}、日语在 {OMISSION_RATIO_LIMITS['ja'][0]}–{OMISSION_RATIO_LIMITS['ja'][1]} 之间；"
          f"各节段数与首列编号的表格行都配得上；中文一段不到 {OMISSION_MINIMUM_MEASURE} 没比的 {below_minimum} 对，"
          f"标题对不上没比的 {heading_mismatched} 份（归 trilingual-structure）；缺份没比的 {len(skipped)} 组归 trilingual-files")
    return 0


COMMANDS = ("files", "structure", "changed-together", "glossary", "untranslated", "omission")


def main(arguments):
    if len(arguments) != 2 or arguments[0] not in COMMANDS:
        print(f"  ✗ 用法：lib-trilingual.py {{{'|'.join(COMMANDS)}}} <项目根>", file=sys.stderr)   # gate-lint:detail
        print("     → 怎么办：给一个命令和项目根；changed-together 另从标准输入读这次改动碰到的路径。", file=sys.stderr)
        return 2
    command, root = arguments
    try:
        os.chdir(root)
    except OSError as error:
        print(f"  ✗ 进不去项目根 {root}：{error}")   # gate-lint:summary
        print("     → 怎么办：第二个参数给项目根（仓的顶层目录）。")
        return 2
    if command == "files":
        return judge_files()
    if command == "structure":
        return judge_structure()
    if command == "glossary":
        return judge_glossary()
    if command == "untranslated":
        return judge_untranslated()
    if command == "omission":
        return judge_omission()
    return judge_changed_together([line.strip() for line in sys.stdin.read().split("\n") if line.strip()])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
