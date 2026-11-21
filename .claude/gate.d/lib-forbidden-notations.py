#!/usr/bin/env python3
"""全仓禁用写法的登记：门禁 doc-text.sh（提交时全仓扫，格名就是形态的键）与 .claude/hooks/write-guard.sh（写入那一刻拒绝）共用这一份，不各抄一份。

FORMS 一项一种形态，加一种禁用写法就是往 FORMS 里加一项。每一项登记：
- key、name：形态键与名字。
- patterns：判据行 (键, 名字, 正则, 上下文规则)，一种形态可以有几行；一行文字命中其中任一条就算这一形态的一处违例，
  first_hit 报最靠前的那一处，all_hits 逐处全报（带行号；同一位置几行都中，报排在前面的那一行）。上下文规则是 None，或一个函数 (整行, 命中, 相对仓根的路径, 表头) → True 表示
  这一处按上下文不是违例、接着找这一行后面的命中（路径没给时是空串；表头见 reads_table_headers，不读表头的形态交 None）。
- excluded_prefixes：(路径前缀, 理由)，相对仓根；这些文件里这一形态是被测输入或只能在上游改，不判。
- allow_suffixes：哪几种后缀的文件认逐行的放行标记（一行里写「# <形态键>:allow <理由>」，Rust 写「//」，理由至少 4 个字）；
  空的就是这一形态不认放行标记。
- judged_suffixes：这一形态只判哪几种后缀的文件（不分大小写）；空的就是什么后缀都判。调用方没给 relative_path 的照判。
- skips_fenced_blocks（可不写，不写是不放过）：判据行键的元组，交进来的文字里围栏代码块中的行不判这几条（围栏认法见 fenced_line_flags），别的判据行照判。
- reads_table_headers（可不写，不写是不读）：为真时 first_hit、all_hits、hits_per_line 按交进来的整段文字认表格（table_headers_by_line），
  每一行所在那张表的表头交给上下文规则的第四个参数；不在表里的行交 None。
写入一侧判一次写进去的内容用 new_violation_hits（写前、写后两份整文，见任务号那一节）。
- clean：成功行「查了 N 份文本文件，」后面接的话。
- excluded_label：成功行「没扫的：」里这一形态排除掉的那几份怎么说（{count} 处填份数）。
- finding：判红汇总行「N 」后面接的话。
- detail_shows_pattern：判红明细行点不点名命中的判据行名字。
- snippet_to_match_end：明细里的片段往后取 10 个字，从命中的结尾算（True）还是从开头算（False）。
- remedy：判红的出路，一项一行；阶段在第一行前面加「→ 怎么办：」。

角标（prime-marks）：撇号类字符 U+2032、U+2033、U+2034、U+02B9、U+02BA 任一个出现就算，给变体起名见
.claude/rules/path-moves.md「变体起新名字，不用角标」；ASCII 单引号不在其内（机器分不开它与引号）。
这份源码里几个字符用码位拼，不出现字面：doc-text.sh 扫全仓，也扫这一份。只排除上游副本：门禁样本也扫，红样本的角标由 setup.sh 现写。

钟点（clock-times）：时间只写到日期，钟点、时区词、时间戳都不写。判据行认的是：
- 时:分 这一族：日期后面跟钟点（「日期 时区词 时:分」、diff 头与 stat 那种「日期 时:分:秒.毫秒 +0000」）、时区词后面跟钟点、
  钟点后面跟时区词、带 x 通配的「时:分x」、不带日期的 Z 钟点，与不挨日期时区的钟点（时:分、时:分:秒、带毫秒、区间、「月-日 时:分」）。
- ISO 时间戳：日期与钟点之间是 T 的（带不带日期、带不带时区偏移或 Z 都算），date 的原样输出也在内。
- ISO 基本格式的紧凑时间戳：日期（8 位连写，或带连字符的）、字母 T、钟点（4 到 6 位连写，可带小数秒）、可带 Z 或偏移，中间没有冒号；
  日期是 8 位连写的，钟点也可以是带冒号的「时:分」。日期与钟点各位都要合法（年 20 开头、月 01 到 12、日 01 到 31、时 00 到 23、分 00 到 59；带连字符的日期也一样核），
  前后不挨 ASCII 字母数字（下划线、连字符、斜杠、点不算，运行目录名与日志文件名里的写法照判）；没有放行写法，字段值、路径名都判。
  日期或钟点越界的、年不是 20 开头的、钟点位数不对的、小写 t 的数字加 T 写法（哈希、编号）不算。不带日期的「T 加钟点」不判（与编号分不开）。
- 时区词：ZONE_WORDS 里那两个缩写，大写、前后不挨 ASCII 字母数字与下划线（标识符里的不算）。
- 中文的「N 点 / N 时」：挨在日期或时区词后面的（中间可有空格、「次日」；「时」后面跟间、代、刻、序、延、钟、效、长、候、机、区、态的
  是时间、时区这一类别的词，不算）；不挨日期、时区的，只认后面跟「前后 / 左右 / 许 / 半 / 钟」的「N 点」与后面跟「许」的「N 时」
  （前面是「第」的是序号，不算）。
- 时段词（PERIOD_WORDS）：挨在日期或时区词后面的（中间可隔空格、全角或半角左括号、全角逗号），与后面跟「N 点 / N 时」或时:分的。
  不挨日期、时区、数字的光秃时段词不判。
- 英文十二小时制：一到两位的时（可带「:分」）后面跟 am / pm（大小写都认、字母后可带点、与数字之间可隔一个空格），
  前后不挨 ASCII 字母数字与下划线（「AMD」「amps」「ppm」这类词不算）；前面是英文月日（外部限额通知的原样）的照判。
会话号（session-names）：ListAgents 列的会话名，本仓与上游三语仓的目录名后面跟「-」与 2 到 6 位小写十六进制；
后面紧跟 . - } / 或 ASCII 字母数字的（mktemp 模板、临时目录名、更长的标识符）不算。改过标题的会话（「会话「<标题>」」）认不出：
同一个写法在仓里也用来引一句话、说一种读法，机器分不开，靠写的人。两位短号（「87」「7a」）不算会话号，也不算批次代号；
括号里当会话名用的两位短号（「(5b)」）归任务号、会话短号与批号那一形态判。

批次代号（batch-codes）：拿来当名字用的批次与运行代号，只判 .md、.tsv、.txt。判据行四条：
- batch-code-suffixed：一到两位数字、一个小写字母、短横、一到两位数字（「24c-1」「22c-2」）；字母是 e 的不算（1e-6 这类指数）。
- batch-code-lettered：两位数字加一个小写字母（「22a」「24c」「10q」），字母不在 UNIT_LETTERS 里（b d g h k m p s t w x 是单位、倍数、天数、
  分辨率与 sed 的 p 这一类，「16k」「64x」「30d」「60s」不判；这几个字母的批号只在挨着「批」「批次」「batch」时判，见下一条）。
- batch-code-labeled：挨着批次词的「数字加字母」：「批 / 批次 / batch(es)」后面跟的（「批次 24c」「批 3c」「batch 10k」）与后面跟「批」的（「第 10h 批」「10k 批」）。
- batch-code-stem：天干编号的批号，前面要有批次词：「第 <数> 批 <天干>」（「第十五批乙二」「第 56 批乙」「第 3 批丁 1」）与「批 / 批次 <数> <天干>」（「批 10 甲」「批次 4-3 己」）。
  不带批次词的「15 乙一」不判：「出口 4 甲一」「已定项 1 甲」「C554 乙」这类候选与臂的名字是同一个写法。地支（「辰」）不在内（「未」「午」与常用字分不开）。
这四条前后都不挨 ASCII 字母数字、- . / 与省略号，前面另不挨 :（标识符、路径、文件名、带连字符的目录名、哈希的截断「…162d」、「文件:行号」不算），
后面不接「.字母数字」（扩展名）与「:数字」（行号）。
一位数字加一个字母（「7a」「3d」）四条都不判（批次词后面的除外；括号里当会话名用的归任务号、会话短号与批号那一形态判）；三位以上数字加字母（「143k」「140n」）不判，挨着批次词的也不判。
上下文规则（batch_code_is_not_code）放过：反引号代码段里的（一串反引号开头、到后面第一串一样长的反引号为止，长短不限；后面配不上一样长的那一串按字面算，
不开代码段）、紧跟在「=」（可隔空白）后面的产物字段值、紧挨在 " ' , ; 后面的代码字面与参数
（sed 的「11,15p」）、算式里的（前面是 + * / × ÷ ( < > ≤ ≥ ~ ≈ ^，或后面是 + * / × ÷ ) < > ≤ ≥ ^ ;，可隔空白：「135 + 140n」「31u) / 32u」）。
不认放行标记；排除的是上游副本、门禁与钩子的样本目录、变异表 crates/mutations.tsv（源码原文列里有 WGSL 的「31u」这类字面，名字列的批号是变异行的键）。
管不到的：中文数字或全角数字写的代号；三位以上数字加字母的（挨不挨批次词都一样）；不带批次词、字母在 UNIT_LETTERS 里的（「10h」「10k」单写）；不带批次词的天干编号；改了写法的代号（「第二十四批丙一」之外的自造写法）。

任务号、会话短号与批号（task-and-session-numbers）：进仓的文字拿号或拿描述指东西，只判 .md、.tsv、.txt。判据行四条：
- task-numbers：任务服务的任务号，四位以上数字：
  「任务 / 任务号 / 任务编号」后面跟号，中间可夹空白、冒号、全角或半角左括号与 #（「（任务 <号>）」「任务号：<号>」「任务（<号>、<号>）」「任务 #<号>」「任务编号 <号>」）；
  号后面（可隔空白）跟「号任务」的（「<号> 号任务」「<号>、<号> 号任务」，号前面不挨数字、ASCII 字母与点）；
  「task / tasks」后面跟空白、#、=、冒号或左括号再跟号（「task <号>」「task#<号>」「task=<号>」「tasks (<号>, <号>)」，结局行的 task= 那一段在内）；
  「task <子命令> [选项] <号>」（子命令是 log、wait、status、clean、cancel，「task-server/task log <号>」这类）；
  任务服务的日志文件名与运行目录名「task-<8 位小写十六进制>-<号>」（「task-<十六进制>-<号>.attempt-1.log」「builds/task-<十六进制>-<号>-1」，号不限位数）；
  「同」后面跟号（单个的「同 <号>」与并列的「同 <号>、<号>」「同 <号>/<号>」「同 <号> 与 <号>」），整串后面（可隔空白）跟量词或单位的不算（「同 4096 字节」「同 2048 个」，表是 QUANTITY_AFTER）；
  「与 / 和 / 跟」后面跟两个以上并列的号、再跟「相同 / 一样」的。
  并列的号（「、」「，」「,」「/」「-」「–」「~」「与」「和」连着）连着算一处。号后面接 ASCII 字母数字、「.数字」或后面不是空白与行尾的冒号的（「文件:行号」、版本）不算，
  号后面接冒号再接空白或行尾的照判（「任务 <号>: 退 0」）；接 - 或 / 的只在后面再接四位以上数字时算（日期、路径不算）；
  「任务 / task」那几支整串后面（可隔空白）跟「个」的是个数（「任务 1180 个」），不算；「个」后面接成词的（「个别」「个人」，表是 COUNT_WORD_COMPOUNDS）照判，「条」不当量词。
- session-short-codes：括号里（半角全角都认）当会话名用的两位小写十六进制：
  挨着会话词的一律判——前面是「会话」「给」「问」「告诉」「通知」「发给」「交给」「转给」，括号后面跟「批」「已认领」「认领」「在修」「在改」「那一路」「那边」「回信」「回话」「的任务」「同意」「说」，
  或括号里短号后面跟 SESSION_WORDS_INSIDE 那几个词（「（b6 推的）」「（5b 在修）」「（5b、10 在修）」「（5b 范围）」「（10 判过）」）；
  mainagent( / subagent( 后面半角括号里不是只有大写字母的（「mainagent(5b)」「subagent(c3)」「mainagent(<字母>)」；按出场次序的 mainagent(A)、subagent(B) 不判），
  agent( / mainagent( / subagent( 后面半角全角括号里的两位小写十六进制（「agent(cc)」「agent(4a)」「mainagent（5b）」；前面挨着 ASCII 字母数字的不算）；
  「会话」后面不带括号、可隔冒号与空白跟的两位小写十六进制（「（会话 4a，<日期>）」「会话 ee 报」），短号后面接 ASCII 字母数字、「-数字」「.数字」「:数字」、
  量词或单位、「到 / 至」与区间符号的不算（「会话 10-14」「会话 42 个子 agent」「会话 13 份」「会话 30 到 0」）；
  不挨会话词、数字字母都有的（「(5b)」「（3b）」）只在 records/ 下与判决（research/prompts/*-main-verification.md）里（与没给路径时）判，kb、别的 research/prompts/ 文件与别处放过
  （那里是步号与攻击项编号，与会话短号字面分不开）；
  不挨会话词、只有数字或只有字母的（「(10)」「(ed)」）不判。后面用「、」「和」「与」连着的括号短号连着算（「(5b)、(10)」）。括号前面挨着 ASCII 字母数字的（「Some(22)」）不算。
- batch-numbers：批号，自己会话的也算：「批 / 批次 / 批号 / 批次号」后面跟号（「批 105」「批 118 量读数」「批 #2」「批次号 118」，可带「-数字」），
  「第 <数字> 批」（「第 6 批」「第 7–13 批」「第 105、106 批」，前面带 # 的锚点「#第 6 批」同样算），与中文数字的「第 <几> 批」（具名组 chinese_ordinal_batch，只在 records/ 下判），
  与「批 / 批次 / 批号」后面（可隔「为」「是」、冒号与空白）跟的中文数字（「截止批次为三」「它的批三」「批次是十二」；具名组 chinese_numeral_batch，只在 records/ 下判），
  中文数字后面接量词、单位或 CHINESE_BATCH_NOT_AFTER 那几个字的是数量与别的词（「这批两份补丁」「批三个实现员」「批一起改」「批三方审」），不算。
  放过数量：前面是「每」「分」的（「每批 64」），数字后面（可隔空白）跟量词或 % 的（「一批 10 个」「批 3 次」「批 3 处改动」）；
  量词字后面接成一个双字词的（「处理」「行为」「项目」，表是 MEASURE_WORD_COMPOUNDS）不当量词，照判。
  后面跟天干的（「第十五批乙二」「批 10 甲」）归批次代号那一形态的 batch-code-stem 判，这一条不重报。
- session-descriptions：拿「<做什么>那一路」的描述指别的会话或子 agent：「那一路」后面（可隔「的」与空白）紧跟会话词——「会话」「主会话」「子会话」、
  「主 agent」「子 agent」、「agent」「mainagent」「subagent」（大小写头字母都认，后面不接 ASCII 字母数字与下划线）——或紧跟会话做的事（SESSION_DESCRIPTION_ACTIONS：
  回信、来信、回话、回复、答复、认领、认了、认下、已认、转达、通知、群发、报、说、在改、在修、交回；「报」后面接红、绿、错的与「说法」「说明」不算）。
  命中的串从「那一路」起（「改检出钩子那一路会话」报「那一路会话」，「GPU 垃圾回收那一路报」报「那一路报」）。
  进仓的文字按出场次序叫别的会话 mainagent(A)、mainagent(B)，叫子 agent subagent(A)、subagent(B)，写它改了哪个文件、做了什么；这两种叫法不在判据里。
  「那一路」后面不跟会话词与这几个动作词的不判（「按块那一路」「变异那一路」「那一路径」「单条那一路问复用」），「这一路」「一路绿」不判。
上下文规则（都看相对仓根的路径；路径没给的照判）：
- task-numbers（task_number_is_kept）：反引号代码段与围栏代码块里照判；只放过两种照写任务号的行——
  research/prompts/ 下 *-main-verification.md 里行首的「A/B 读数：」一行（认法同 .claude/hooks/runner-dispatch-guard.sh 的 ZERO_ROUND_READING_LINE；引用块里的不算：
  去掉行首空白与列表记号（-、*、+、数字.、数字)）之后以「>」起头的，QUOTE_BLOCK_LINE），
  值里按派发闸的 ZERO_ROUND_TASK_NUMBER 认出的任务号至多 ZERO_ROUND_READING_MOST_NUMBERS 个时，放过的只是与认出的那几处起止都相同的命中（同一行里别的写法的号、
  连着并列号的、认出的超过两个的照判；zero_round_reading_numbers 交认出的那几处），
  与 records/ 下（去掉列表记号之后）以「长跑拆分：」起头的一行（.claude/rules/long-run-split.md「怎么做」第 6 步那一行）。
- session-short-codes（session_code_is_kept）：反引号代码段里的放过；不挨会话词的数字字母括号短号不在 records/ 下、也不在判决里的放过（见上）。
- batch-numbers（batch_number_is_kept）：反引号代码段里的放过；中文数字的「第几批」与「批」后面跟的中文数字不在 records/ 下的放过（kb 里「第一批字节」这类泛指次序）；
  records/ 下的表格里、表头那一格是「批次」的那一列的单元格放过（依赖表批次那一列；表头由 table_headers_by_line 从交进来的整段文字认：
  以「|」起头的连续几行、第二行是分隔行的才是表，头一行是表头；没有分隔行的「|」起头的行不是表，照判；分格的「|」前面紧挨着的反斜杠是偶数个，
  奇数个的是转义的竖线、不分格，unescaped_pipe_positions）；
  不在 records/ 下的「批 <数>」前面紧挨着 seq、攒、追加、覆写、写入（中间可隔空白）的放过（fsync 攒批的大小：「seq 批 1」「攒批 64」「追加批 10」）；「第 N 批」写法与 records/ 下不放宽。
- session-descriptions（session_description_is_kept）：反引号代码段里的放过（同 session-short-codes），不看路径。
围栏代码块（skips_fenced_blocks）：session-short-codes、batch-numbers 与 session-descriptions 不判围栏里的行，task-numbers 照判。围栏 0 到 3 个空格缩进的 ``` 或 ~~~ 起、
同字符且不短于开头的那一行收，没收的护到末尾，同 scripts/kb/lib_markdown.py 的认法；围栏与表头都按交进来的整段文字认。
写入一侧（写闸、bash-command-detector.sh 的 ⑰、scripts/kb/ 的写入工具）交 new_violation_hits 写前、写后两份整文：两份各判一遍（围栏与表头各按自己那一份认），
相抵的单位是违例本身——判据行名字加命中的串，按行抵：写后每一行先配上写前的一行（没变的行配原来那一行；改了的一块里按去掉违例之后剩下的字像不像配，
只许按次序配；整行一字不差挪了位置的配原来那一行），这一行上的违例只拿配上的那一行上的同一种去抵，按个数抵，抵不掉的报出来。文件里已有的旧号不挡这一次写，
只改了那一行别处（路径、措辞，或在它前后加了行）的照写；同一行新加的号、整份多出来的同一个号、写前没有的号照判，同一字面的旧号离开写前它所在的那一行
（挪到另一句所在的那一行、挪进新加的一行）照判；这一次写让后面已有的行出了围栏、离开了「批次」列的，写前那一处不是违例、抵不了，照判。
不认放行标记；排除的是上游副本与门禁、钩子的样本目录。
管不到的：没带「任务」「task」「同」的裸任务号与「号任务」的号；不挨会话词的纯数字或纯字母括号短号（与「（10）」这类计数分不开）；不挨「会话」的不带括号的会话短号；
「那个会话」「<做什么>的会话」这类不带「那一路」的泛称（与正常行文分不开）；「那一路」后面接的不是会话词与 SESSION_DESCRIPTION_ACTIONS 的描述（「那一路的工具员」）；
相抵按行配、不按句配：同一行里把旧号挪到另一句（「甲（任务 <号>）。乙。」改成「甲。乙（任务 <号>）。」）算只改了那一行别处，照抵；
拿描述指别的会话的，命中的串只是「那一路会话」「那一路回信」这几个字，指的是谁写在「那一路」前面的描述里：配上的那一行里把描述改成指另一个会话
（「改闭包的那一路会话」改成「管任务服务的那一路会话」），命中的串没变，照抵（判不出改写之后指的是哪个会话）；
配行看去掉违例之后剩下的字像不像，一块改动里号挪去的那一行比它自己原来的样子更像带旧号的那一行时会配错、照抵；
一块改动里写前行数乘写后行数超过 PAIRING_CELLS_LIMIT 的（Write 整份重写大文件），那一块按位置从上往下配；
两臂分两行写的读数：「A/B 读数：」那一行之外另起一行的「任务 <号>」不是读数行，照判（出路是两臂写在同一行）；
英文的 batch 加数字；数量与批号同形的（「三批 64、128」照判）；四个空格缩进的代码块（当正文判）；
records/ 下以「长跑拆分：」起头的一行装几个任务号（.claude/rules/long-run-split.md 第 6 步要这一行写项名、任务号与拆后各项重跑的号，几个号都合规，分不出哪个号与这一项无关）；
.json（records/ 下的机读台账也是，judged_suffixes 不收）。


上下文规则放过长得像钟点、其实不是的冒号对（不挨日期时区的钟点那一行）：
- 紧挨在前面的是 = [ { , > " ' + - . 之一或「, 」：产物行的「名字=值」、切片、字典与逗号列表、「0->1:10」、引号里的代码字面、
  时区偏移、小数；前面是反引号而小时只有一位的，是反引号里的键（`3:10`）；
- 紧跟在后面的是 %、「.数字」或「-字母」：百分比、「分:秒.百分之一秒」（time -v 的格式）、带编号的阶段文件名（「20:09-sample-stage.sh」这类）；
- 紧跟在后面的是 = 或 /、或紧挨在前面的是 @、; 或 #：产物行里「槽:代=ok@槽:代」「rows=槽:代/…;槽:代/…」这类键值对与分隔列表，与「操作名#序号:数」这类带序号的字段值；
- mountinfo 一行里的主:次设备号（`/proc/<进程>/mountinfo` 的「挂载号 父挂载号 主:次设备号 根路径 挂载点 …」，Rust 多行字符串里的续行加不了放行标记）：
  冒号对前面是「数字 空格 数字 空格」（挂载号与父挂载号），后面是「 /根路径 /挂载点」。只认这一个形态，缺一头的（只有一个挂载号、后面不是路径）照判；
- 同一句里（句的边界是 。！？，没有这几个就是整行；「；」不算）有取值出了钟点范围的冒号对（时大于 23、分大于 59、一边三位或分只有一位）或「文件:行号」（「.md:42」这类）的，
  这一句里的冒号对按「门禁号:行号」「决策号:行号」这类编号读；别的句子里的冒号对不受它牵连。
ISO 时间戳与「日期后面跟钟点」只放过紧跟在「=」后面的（产物行的字段值，「taken_jst=<日期>T<钟点>」这类）。
排除的是把钟点当被测输入或期望输出用的文件（hook 自检的假日志、门禁样本、以时区为被测算法的 c510 那一轮）与上游副本。
管不到的：同一句既有编号对又有真钟点的，真钟点跟着一起放过；引号、反引号里两位小时以外的代码字面按上面那几条放过。

弄坏开关（环境变量，逗号分隔，可以写几个键；每次现读，同一个进程里切换也认）：
- LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键或形态键>：那几条判据行（写形态键的，那一形态的全部判据行）不判。
  打开时 doc-text.sh 那一形态的红样本（prime-marks-red、clock-times-red、session-names-red）、write-guard.sh --selftest 里点名那一行的格必须判红，证判据真在管那一行。
  同一个变量另认两项会让编号对与紧凑时间戳判法退回旧样子的：numbered-pairs-whole-line（编号对按整行读，不按句读）与 compact-timestamp-loose（紧凑时间戳退回「8 位数字加 T 加 4 到 6 位数字」）；
  前者打开时 doc-text.sh 的 clock-times-red 里 records/bare-clock-sentences.md 那五行必须报不出来（want 找不到、行数对不上），后者打开时 clock-times-green 里 records/近似写法.md 的数字加 T 近似写法必须判红。
  紧凑时间戳那一条判据行的键是 iso-compact-timestamp，写进上面的 <判据行键> 关掉它时 clock-times-red 里 records/iso-compact.md 那六行必须报不出来。
  同一个变量另认一项 first-hit-only：all_hits 只交最靠前的那一处；
  打开时 write-guard.sh --selftest 的「钟点:两处都报（行号与总数）」「会话号:两处都报（行号与总数）」两格必须判红。
  同一个变量另认一项 mountinfo-device-numbers：mountinfo 一行里的主:次设备号不放过，打开时当钟点判；
  打开时 doc-text.sh 的 clock-times-green 里 records/colon-pairs.md 的 mountinfo 那两行必须判红（clock-times-mountinfo-red 不受影响，它本来就红）。
- LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=<判据行键或形态键>：那几条判据行的上下文规则不生效，命中一律算违例。
  打开时 doc-text.sh 的 clock-times-green 样本里的冒号对、write-guard.sh --selftest 里靠上下文放行的格必须判红，证放行靠的是上下文规则。
- LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=<形态键>：那几种形态的放行标记不认。
  打开时 doc-text.sh 的 clock-times-green、session-names-green 样本与 write-guard.sh --selftest 里带放行标记的格必须判红。
- LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=<形态键>：那几种形态的 excluded_prefixes 不生效。
  打开时 doc-text.sh 那一形态的绿样本（prime-marks-green、clock-times-green、session-names-green）里落在排除前缀下的放行格必须判红，证排除前缀只放过登记的那几处。
- 同一个变量另认一项 candidate-prefilter-rejects-all：hits_per_line（doc-text.sh 的 prime-marks、clock-times、session-names 三格逐份文件逐行扫时用它）先筛一律说没有候选；
  doc-text.sh 的 GATE_TEXT_NOTATIONS_BREAK=scan-prefilter-rejects-all 把它转进来，打开时三格的红样本（prime-marks-red、clock-times-red、clock-times-mountinfo-red、session-names-red、
  private-values-clock-snippet-red、private-values-siblings-red）必须判红，证先筛没有吞掉真违例；first_hit 不经先筛，留作 hits_per_line 逐项对拍的参照。
- 批次代号一形态（还没有 doc-text.sh 的格，判红的格在 write-guard.sh --selftest 与 bash-command-detector.sh --selftest 里）：
  LIB_FORBIDDEN_NOTATIONS_BREAK=batch-codes 关掉整形态、=batch-code-suffixed / batch-code-lettered / batch-code-labeled / batch-code-stem 关掉一条判据行，
  write-guard.sh --selftest 里点名那一条的「批次代号:」拒绝格必须判红；LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=batch-codes 时「批次代号上下文:」几格必须判红；
  LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=batch-codes 时「批次代号:变异表放行」必须判红。
  =batch-codes 时 bash-command-detector.sh --selftest 里写批次代号的四格「字面写入:」、scripts/kb/ 三份定点写入脚本自证里「写会话号：.md 目标的新串带」起头的两行也红。
  同一个变量另认一项 judged-suffixes-ignored：judged_suffixes 不生效，什么后缀都判（退回没有这一键的样子）；打开时 write-guard.sh --selftest 的
  「批次代号:.py 不判」「批次代号:.sh 不判」两格、bash-command-detector.sh --selftest 的「字面写入:写进 .py 的批次代号不判（批次代号只判 .md、.tsv、.txt）」、
  三份定点写入脚本自证里「写会话号：.py 目标的新串带代号」那一行必须判红。
- 三项放宽判法的开关（同一个变量 LIB_FORBIDDEN_NOTATIONS_BREAK，表在 LOOSENED_PATTERNS 与 batch_code_is_not_code）：
  compact-hyphenated-date-loose：紧凑时间戳带连字符的那一种写法不核月日；打开时 write-guard.sh --selftest 的「紧凑时间戳:带连字符的日期月份越界放行」
  「紧凑时间戳:带连字符的日期日越界放行」两格必须判红。
  batch-code-labeled-three-digits：挨着批次词的三位数字加字母也判；打开时「批次代号:批次词后面的三位数字加字母放行」「批次代号:三位数字加字母挨着批字放行」两格必须判红。
  code-span-backtick-parity：反引号只数前面的奇偶，不按代码段配对；打开时「批次代号上下文:双反引号代码段里的放行」「批次代号:配不上对的反引号不开代码段」两格必须判红。
- 任务号、会话短号与批号一形态（判红的格在 write-guard.sh --selftest、bash-command-detector.sh --selftest、scripts/kb/ 几份写入脚本的自证与 doc-text.sh 的
  task-and-session-numbers 格的样本里）：
  LIB_FORBIDDEN_NOTATIONS_BREAK=task-and-session-numbers 关掉整形态、=task-numbers / session-short-codes / batch-numbers / session-descriptions 关掉一条判据行，
  write-guard.sh --selftest 里点名那一条的「任务号与批号:」拒绝格、bash-command-detector.sh --selftest 里写这一形态的「字面写入:」格、
  scripts/kb/ 几份写入脚本自证里写这一形态的那几行、doc-text.sh 的 task-and-session-numbers-red 样本必须判红（各自文件头点名）；
  LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=task-and-session-numbers 时 write-guard.sh --selftest 的「任务号与批号上下文:」几格里靠上下文规则放行的必须判红；
  同一个变量 LIB_FORBIDDEN_NOTATIONS_BREAK 另认几项，各把这一形态的一处判法放宽或改掉（点名的格写在 write-guard.sh、bash-command-detector.sh 与 scripts/kb/ 几份写入脚本的文件头）：
    fenced-blocks-judged：围栏代码块里的行四条判据行都照判；打开时「任务号与批号上下文:围栏代码块里的会话短号与批号放行」
      「任务号与批号上下文:围栏代码块里拿描述指别的会话放行」必须判红。
    fenced-blocks-all-rows：围栏代码块里的行四条判据行都不判（任务号也放过）；打开时「任务号与批号:围栏代码块里的任务号」必须判红。
    task-numbers-in-code-span-kept：反引号代码段里的任务号放过；打开时「任务号与批号:反引号里的任务号」「任务号与批号:反引号里 task 子命令加号」必须判红。
    zero-round-reading-anywhere：「A/B 读数：」放行不锚行首、不看文件、整行放过，只认「A/B 读数」中间一个空格、后面紧跟冒号；打开时「任务号与批号:A/B 读数不在行首」
      「任务号与批号:A/B 读数一行在记录里」「任务号与批号:A/B 读数一行里同字并列的号」「任务号与批号上下文:派发闸认的 A/B 读数写法放行」必须判红。
    zero-round-reading-count-unchecked：「A/B 读数：」一行里认出的任务号不核个数；打开时「任务号与批号:A/B 读数一行装三个号」必须判红。
    zero-round-reading-quote-kept：引用块（「>」起头）里的「A/B 读数：」一行也放过；打开时「任务号与批号:引用块里的 A/B 读数」必须判红。
    reserved-lines-any-file：「长跑拆分：」一行与表格行里的批号（任何一列）在任何文件里都放过；打开时「任务号与批号:长跑拆分一行在 kb 里」「任务号与批号:表格行里的批号在 kb 里」必须判红。
    table-batch-any-column：records/ 下的表格行里任何一列的批号都放过（不看表头）；打开时「任务号与批号:记录里表格别的列的批号」必须判红。
    batch-size-loosened：攒批的放宽改成同一分句里前面有 seq、攒、追加、覆写、写入就放，records/ 下与「第 N 批」写法也放；打开时「任务号与批号:记录里追加第几批」
      「任务号与批号:kb 里同一分句后面的批号」必须判红。
    measure-compounds-counted：量词字后面接成词的（「处理」「行为」「项目」）也当量词；打开时「任务号与批号:批号后面跟处理、行为、项目」必须判红。
    bare-short-codes-everywhere：不挨会话词的数字字母括号短号在任何文件里都判；打开时「任务号与批号上下文:kb 里括号里的步号放行」必须判红。
    violations-not-netted：new_violation_hits 不拿写前的违例相抵，写后整份的每一处都交；打开时「任务号与批号:写前文件里已有的旧号不挡这一次写」
      与几份写入工具、检出钩子自证里同一形的那几格、scripts/kb/lib_notation_check.py 的「写会话号：行里已有任务号、同一行只改路径」
      「写会话号：整份多出一行带同一个旧号」「写会话号：只改同一行别处照放行」必须判红。
    下面六项各把 new_violation_hits 的相抵换成一种不对的算法，判红的格在 scripts/kb/lib_notation_check.py 的 session_guard_problems 里
    （replace-once.py、replace-batch.py、insert-row.py 三份自证共用，各自文件头点名）：
    violations-netted-by-line-text：按行文相抵（写后有违例的行与写前有违例的行行文相同才抵，同一行文写前有几行、写后就抵几行），行里别处一改、原有的号就算新写的；
      打开时「写会话号：行里已有任务号、同一行只改路径」「写会话号：只改同一行别处照放行」必须判红。
    violations-netted-by-line-number：写前哪一行号上有违例，写后同一行号上的违例全抵掉；打开时「写会话号：行里已有任务号、同一行新加第二个任务号」
      「写会话号：只改同一行别处照放行」必须判红。
    violations-netted-uncounted：按名字与串整份相抵但不计数、不看在哪一行，写前出现过的串写后出现几处都抵掉；打开时「写会话号：整份多出一行带同一个旧号」
      「写会话号：旧号挪到另一句照拒」「写会话号：旧号挪到另一行照拒」必须判红。
    violations-netted-across-lines：按名字与串整份计数相抵、不看在哪一行（hits_netted_across_lines），同一字面的旧号挪到别的行也抵掉；
      打开时「写会话号：旧号挪到另一句照拒」「写会话号：旧号挪到另一行照拒」必须判红。
    line-pairing-positional：一块改动里写前写后的行按位置从上往下配，不按去掉违例之后的字像不像配（paired_before_lines）；带旧号那一行前面加了一行，它就配不上原来那一行；
      打开时「写会话号：只改同一行别处照放行」必须判红。
    before-context-ignored：写前那一份逐行单判（不认围栏与表头），写后照认；打开时「写会话号：围栏里已有批号的那一行挪到围栏外」
      「写会话号：改了表头、已有的批号离开批次那一列」「写会话号：新串只写一行围栏、让后面已有的批号出了围栏」必须判红。
    after-lines-judged-alone：new_violation_hits 把写前、写后两份逐行单判（不认围栏与表头）；打开时写闸「任务号与批号:Edit 删开头围栏行让批号出了围栏」
      「任务号与批号上下文:Edit 以围栏起头、插入点前没有开着的围栏放行」与检出钩子、几份写入工具自证里同一形的那几格
      （scripts/kb/lib_notation_check.py 的「写会话号：围栏里已有批号的那一行挪到围栏外」「写会话号：改了表头、已有的批号离开批次那一列」也在内）必须判红。
    session-descriptions-loose：session-descriptions 那一条判据行换成宽的认法（SESSION_DESCRIPTION_LOOSE_PATTERN：带「一路」两个字的、mainagent( 与 subagent( 一律算），
      不要求「那一路」后面跟会话词、也不放过按出场次序的叫法；打开时写闸 --selftest 的「任务号与批号:这一路放行」「任务号与批号:那一路径放行」「任务号与批号:一路绿放行」
      「任务号与批号:那一路后面不跟会话词放行」「任务号与批号:mainagent 按出场次序的叫法放行」「任务号与批号:subagent 按出场次序的叫法放行」、
      scripts/kb/ 几份写入工具自证里「写会话号：」起头、说「按出场次序的叫法与不跟会话词的一路」的那一行、append-record.py 自证的「写之前判文本」一格
      与 doc-text.sh 的 task-and-session-numbers-descriptions-green 样本必须判红。
  LIB_FORBIDDEN_NOTATIONS_BREAK=session-descriptions 时写闸 --selftest 的「任务号与批号:那一路会话（调度记录原句：」起头的三格与「任务号与批号:那一路的会话与那一路的 agent」、
    scripts/kb/ 几份写入工具自证里说「拿描述指别的会话」的那一行、append-record.py 自证的「写之前判文本」一格、doc-text.sh 的 task-and-session-numbers-descriptions-red 样本必须判红；
  LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=session-descriptions 时写闸 --selftest 的「任务号与批号上下文:反引号里拿描述指别的会话放行」与 doc-text.sh 的
    task-and-session-numbers-descriptions-green 样本必须判红。
  同一个变量 LIB_FORBIDDEN_NOTATIONS_BREAK 另认几项，各把一种近似写法、表格与引用块的一处认法退回去（换上的判据串在 LOOSENED_PATTERNS；红的格写在 write-guard.sh 文件头）：
    agent-parentheses-unjudged：mainagent( / subagent( / agent( 括号里的短号与别的写法不判；打开时写闸 --selftest 的「任务号与批号:mainagent 与 subagent 括号里的短号」
      「任务号与批号:agent 括号里的短号」必须判红。
    session-word-code-unjudged：「会话」后面不带括号跟的短号不判；打开时「任务号与批号:会话后面不带括号跟短号」必须判红。
    chinese-numeral-batch-unjudged：「批」后面跟的中文数字不判；打开时「任务号与批号:记录里批后面跟中文数字」必须判红。
    task-number-variants-unjudged：「任务编号 <号>」「<号> 号任务」不判；打开时「任务号与批号:任务编号与号任务」必须判红。
    session-description-actions-unjudged：「那一路」后面紧跟动作词的不判；打开时「任务号与批号:那一路后面跟回信、认领、转达这类动作词」必须判红。
    table-escaped-pipe-parity-ignored：表格里竖线前面挨着反斜杠的一律当转义、不数奇偶（unescaped_pipe_positions）；打开时「任务号与批号:表格里转义的反斜杠加竖线」必须判红。
    zero-round-reading-list-quote-kept：引用块只认去掉行首空白后以「>」起头的，列表里的「- >」不算（quoted_line）；打开时「任务号与批号:列表里的引用块里的 A/B 读数」必须判红。
"""
import difflib
import os
import re

# U+2032 PRIME、U+2033 DOUBLE PRIME、U+2034 TRIPLE PRIME、U+02B9 MODIFIER LETTER PRIME、U+02BA MODIFIER LETTER DOUBLE PRIME
PRIME_MARKS = ''.join(map(chr, (0x2032, 0x2033, 0x2034, 0x02B9, 0x02BA)))
MARKS_SHOWN = ' '.join(PRIME_MARKS)

DATE = r'20\d\d-\d\d-\d\d'
CLOCK = r'(?<![\d:])\d{1,2}:(?:\d\d|\dx|xx)(?::\d\d(?:\.\d+)?)?(?![\d:]|\.\d)'
CLOCK_RANGE = rf'{CLOCK}(?:\s*[–~\-]\s*(?:次日\s*)?{CLOCK})?'
ZONE_WORDS = ('JST', 'UTC')  # clock-times:allow 判据自己的时区词表
ZONE = '(?:' + '|'.join(ZONE_WORDS) + ')'
NEXT_DAY = r'(?:次日\s*)?'
# 「N 点 / N 时」：「时」后面跟这几个字的是时间、时代、时刻、时序、时延、时钟、时效、时长、时候、时机、时区、时态，不是钟点
HOUR = r'(?<![\d:])\d{1,2}\s*(?:点|时(?![间代刻序延钟效长候机区态]))'
PERIOD_WORDS = ('上午', '下午', '凌晨', '早上', '早晨', '清晨', '中午', '午后', '傍晚', '晚上', '夜里', '深夜', '半夜')
PERIOD = '(?:' + '|'.join(PERIOD_WORDS) + ')'
# ASCII 的字母数字与下划线（Python 的 \w 连汉字也算，钟点挨着汉字照样要判）
ASCII_WORD = 'A-Za-z0-9_'
HOUR_VALUE = r'(?:[01]?\d|2[0-3])'
MINUTE_VALUE = r'[0-5]\d'
BARE_CLOCK = rf'(?<![{ASCII_WORD}:]){HOUR_VALUE}:{MINUTE_VALUE}(?::{MINUTE_VALUE}(?:\.\d+)?)?(?![{ASCII_WORD}:])'
ISO_TIMESTAMP = rf'(?:{DATE}|(?<![{ASCII_WORD}\-]))T(?:[01]\d|2[0-3]):{MINUTE_VALUE}'
# ISO 基本格式的紧凑时间戳，两种写法：带连字符的日期加连写的钟点；8 位连写的日期加连写的钟点（4 到 6 位，可带小数秒）或带冒号的「时:分」。
# 之后可带 Z 或偏移。日期与钟点的各个位都要在合法范围里（年 20 开头、月 01 到 12、日 01 到 31、时 00 到 23、分 00 到 59），前后不挨 ASCII 字母数字
# （下划线、连字符、斜杠、点不算：运行目录名与日志文件名里的写法照判）；8 位数字后面跟 T 与 4 到 6 位数字、但日期或钟点越界的编号不算。
# 带连字符的日期加带冒号的钟点不在这一条里（ISO_TIMESTAMP 判，它认「=」后面的字段值放行）。
MONTH_VALUE = r'(?:0[1-9]|1[0-2])'
DAY_VALUE = r'(?:0[1-9]|[12]\d|3[01])'
DATE_COMPACT = rf'20\d\d{MONTH_VALUE}{DAY_VALUE}'
DATE_HYPHENATED = rf'20\d\d-{MONTH_VALUE}-{DAY_VALUE}'
TIME_COMPACT = r'(?:[01]\d|2[0-3])[0-5]\d\d{0,2}(?:[.,]\d+)?'


def iso_compact_timestamp(hyphenated_date):
    """紧凑时间戳的判据串；hyphenated_date 是带连字符那一种写法的日期（平常是核月日的 DATE_HYPHENATED）。"""
    return (rf'(?<![A-Za-z0-9])(?:{hyphenated_date}T{TIME_COMPACT}|{DATE_COMPACT}T(?:{TIME_COMPACT}|(?:[01]\d|2[0-3]):{MINUTE_VALUE}))'
            rf'(?:Z|[+\-](?:[01]\d|2[0-3]):?[0-5]\d)?(?![A-Za-z0-9])')


ISO_COMPACT_TIMESTAMP = iso_compact_timestamp(DATE_HYPHENATED)
# 弄坏用：不核日期与钟点的范围、前后不管，8 位数字加 T 加 4 到 6 位数字都算（LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose）
COMPACT_TIMESTAMP_LOOSE_PATTERN = re.compile(r'\d{8}T\d{4,6}')
# 弄坏用：带连字符的那一种写法不核月日（LIB_FORBIDDEN_NOTATIONS_BREAK=compact-hyphenated-date-loose）
COMPACT_HYPHENATED_DATE_LOOSE_PATTERN = re.compile(iso_compact_timestamp(DATE))
ZONE_WORD = rf'(?<![{ASCII_WORD}]){ZONE}(?![{ASCII_WORD}])'
TWELVE_HOUR = rf'(?<![{ASCII_WORD}:.])\d{{1,2}}(?::{MINUTE_VALUE})? ?[AaPp]\.?[Mm]\.?(?![{ASCII_WORD}])'
# 会话号：本仓与上游三语仓的目录名，后面跟 - 与 2 到 6 位小写十六进制；后面紧跟 . - } / 的是 mktemp 模板、路径里的临时目录名，不算
SESSION_NAME = rf'(?<![{ASCII_WORD}\-])singlefs(?:-ai-sop(?:-zh|-en|-ja)?)?-[0-9a-f]{{2,6}}(?![{ASCII_WORD}.\-}}/])'
# 批次代号：前后不挨 ASCII 字母数字、- . / 与省略号，前面另不挨 :（标识符、路径、带连字符的目录名、哈希的截断、「文件:行号」），后面不接扩展名与「:行号」
BATCH_CODE_BEFORE = rf'(?<![{ASCII_WORD}\-./:…])'
BATCH_CODE_AFTER = rf'(?![{ASCII_WORD}\-/]|\.[{ASCII_WORD}]|:\d)'
UNIT_LETTERS = 'bdghkmpstwx'   # 跟在两位数字后面多半是单位、倍数、天数、分辨率或 sed 的 p：不挨批次词时不判
CODE_LETTERS = ''.join(letter for letter in 'abcdefghijklmnopqrstuvwxyz' if letter not in UNIT_LETTERS)
HEAVENLY_STEMS = '甲乙丙丁戊己庚辛壬癸'
CHINESE_NUMERALS = '一二三四五六七八九十百零两'
BATCH_CODE_SUFFIXED = rf'{BATCH_CODE_BEFORE}\d{{1,2}}(?!e-)[a-z]-\d{{1,2}}{BATCH_CODE_AFTER}'
BATCH_CODE_LETTERED = rf'{BATCH_CODE_BEFORE}\d{{2}}[{CODE_LETTERS}]{BATCH_CODE_AFTER}'


def batch_code_labeled(most_digits):
    """挨着批次词的「数字加字母」的判据串；数字至多 most_digits 位（平常是 2：三位以上数字加字母不判，挨着批次词的也一样）。"""
    digits = rf'\d{{1,{most_digits}}}'
    return (rf'(?:(?<![{ASCII_WORD}])批次?\s*|(?<![{ASCII_WORD}])[Bb]atch(?:es)?\s+){BATCH_CODE_BEFORE}{digits}[a-z]{BATCH_CODE_AFTER}'
            rf'|{BATCH_CODE_BEFORE}{digits}[a-z]\s*批')


BATCH_CODE_LABELED = batch_code_labeled(2)
# 弄坏用：挨着批次词的三位数字加字母也判（LIB_FORBIDDEN_NOTATIONS_BREAK=batch-code-labeled-three-digits）
BATCH_CODE_LABELED_THREE_DIGITS_PATTERN = re.compile(batch_code_labeled(3))
BATCH_CODE_STEM = (rf'(?:第\s*[\d{CHINESE_NUMERALS}]+\s*批\s*|(?<![{ASCII_WORD}])批次?\s*\d{{1,3}}(?:-\d{{1,2}})?\s*)'
                   rf'[{HEAVENLY_STEMS}][{CHINESE_NUMERALS}\d]?')
# 任务号、会话短号与批号（task-and-session-numbers）的三条判据行；判法写在文件头那一节
# 号后面：不挨 ASCII 字母数字与「.数字」，冒号只在后面接空白或行尾时算号的收尾（「文件:行号」不算）；- 与 / 只在后面再接四位以上数字时算并列的号（日期「2026-11-18」、路径照放）
TASK_NUMBER_AFTER = rf'(?![{ASCII_WORD}]|:(?![ \t]|$)|[\-/](?!\d{{4,}}(?![{ASCII_WORD}]))|\.\d)'
TASK_DIGITS = rf'\d{{4,}}{TASK_NUMBER_AFTER}'
TASK_NUMBER_JOINER = r'\s*(?:[、，,/\-–~～]|与|和)\s*'
TASK_JOINED = rf'{TASK_DIGITS}(?:{TASK_NUMBER_JOINER}{TASK_DIGITS})*+'   # 并列的号连着算一处；占有量词：收尾的判法看整串之后，不退回少吃几个号去凑
TASK_SUBCOMMANDS = 'log|wait|status|clean|cancel'   # task-server/task 后面跟任务号的几个子命令
# 「任务 / task」那几支整串后面跟「个」是个数；「个」后面接成这几个词的不算量词
COUNT_WORD_COMPOUNDS = ('个别', '个人', '个体', '个性', '个案')
COUNT_AFTER = rf'(?!\s*个(?!{"|".join(word[1:] for word in COUNT_WORD_COMPOUNDS)}))'
# 量词字后面接成一个双字词的不当量词（「批 105 处理的是」「同 1234 条件」）；表里只收常作双字词头一个字、跟在号后面又不是数量的
MEASURE_WORD_COMPOUNDS = ('处理', '处于', '处在', '处置', '行为', '行动', '行使', '行进', '项目', '位于', '位置', '道理', '次序', '次要',
                          '组织', '组成', '组合', '页面', '字段', '格式', '片段', '轮询', '轮换', '周期', '种类', '条件', '条目', '条款',
                          '个别', '个人', '个体', '个性', '个案', '份额')
BATCH_NUMBER_MEASURE_WORDS = '个条份项次件台组块页字道格趟片轮天周位行种处遍%'
TIME_AND_SIZE_UNITS = '字节|毫秒|微秒|纳秒|秒|分钟|小时|年|月|日|[KMGT]i?B(?![A-Za-z])|B(?![A-Za-z])|ms(?![A-Za-z])|us(?![A-Za-z])|ns(?![A-Za-z])|s(?![A-Za-z])'


def quantity_after(compounds_counted, with_units):
    """号后面（可隔空白）跟量词（with_units 为真时另认 TIME_AND_SIZE_UNITS 那几个单位）的判据串：量词字（BATCH_NUMBER_MEASURE_WORDS）后面接成
    MEASURE_WORD_COMPOUNDS 里的词的不算量词，compounds_counted 为真时接成词的也算（弄坏开关 measure-compounds-counted 换上的写法）。"""
    not_compound = '' if compounds_counted else rf'(?!{"|".join(MEASURE_WORD_COMPOUNDS)})'
    units = f'|{TIME_AND_SIZE_UNITS}' if with_units else ''
    return rf'\s*(?:{not_compound}[{BATCH_NUMBER_MEASURE_WORDS}]{units})'


QUANTITY_AFTER = quantity_after(False, True)   # 「同 <号>」后面跟量词或单位的是数量
# 任务服务的日志文件名与运行目录名：task-<存储标签，8 位小写十六进制>-<任务号>（「task-<十六进制>-<号>.attempt-1.log」「builds/task-<十六进制>-<号>-1」）
TASK_LOG_NAME = rf'(?<![{ASCII_WORD}])task-[0-9a-f]{{8}}-\d+'
def task_number(with_variants):
    """任务号那一条判据行的判据串；with_variants 为真时另认「任务编号 <号>」与「<号> 号任务」（平常是真，弄坏开关 task-number-variants-unjudged 换上假的那一版）。"""
    task_word = '任务(?:号|编号)?' if with_variants else '任务(?:号)?'
    numbered_task = rf'|(?<![\d{ASCII_WORD}.]){TASK_JOINED}\s*号任务' if with_variants else ''
    return (rf'(?<![{ASCII_WORD}])(?:{task_word}\s*[:：]?\s*[（(]?\s*#?\s*'
            rf'|[Tt]asks?(?:\s*[=:#（(]\s*#?|\s+#?)\s*'
            rf'|task\s+(?:{TASK_SUBCOMMANDS})(?:\s+--?[A-Za-z][\w\-]*)*\s+#?){TASK_JOINED}{COUNT_AFTER}'
            rf'|(?<![相不共如协合])同\s*{TASK_JOINED}(?!{QUANTITY_AFTER})'
            rf'|(?<![{ASCII_WORD}])(?:与|和|跟)\s*(?={TASK_DIGITS}(?:{TASK_NUMBER_JOINER}{TASK_DIGITS})+\s*(?:相同|一样)){TASK_JOINED}'
            rf'|{TASK_LOG_NAME}{numbered_task}')


TASK_NUMBER = task_number(True)
# 弄坏用：不认「任务编号 <号>」与「<号> 号任务」（LIB_FORBIDDEN_NOTATIONS_BREAK=task-number-variants-unjudged）
TASK_NUMBER_WITHOUT_VARIANTS_PATTERN = re.compile(task_number(False))
SHORT_CODE_IN_PARENTHESES = r'[（(]\s*[0-9a-f]{2}\s*[)）]'
SESSION_WORDS_BEFORE = '会话|给|问|告诉|通知|发给|交给|转给'
SESSION_WORDS_AFTER = '批|已认领|认领|在修|在改|那一路|那边|回信|回话|的任务|同意|说'
SESSION_WORDS_INSIDE = '在修|在改|在跑|在做|接了|已接|接手|推的|量的|转述|范围|判过|已认领|认领|那一路|回信|回话'   # 括号里短号后面跟的会话词（不收「说」：「（12 说明）」这类）
# 「会话」后面不带括号跟的短号：短号后面接 ASCII 字母数字、「-数字」「.数字」「:数字」（日期、编号、行号）、量词或单位（「会话 42 个子 agent」「会话 13 份」）、
# 「到 / 至」与区间符号（「会话 30 到 0」）的不算
SESSION_WORD_CODE_AFTER = rf'(?![{ASCII_WORD}]|[\-./:]\d|{QUANTITY_AFTER}|\s*[到至~～–])'


def session_short_code(with_agent_parentheses, with_session_word_code):
    """括号里的会话短号那一条判据行的判据串。挨着会话词的几种排在前面：同一个位置上先认它们，不挨会话词的数字字母短号（具名组 bare_short_code，只在 records/ 下判）排最后。
    with_agent_parentheses 为真时另认：mainagent( / subagent( 括号里不是只有大写字母的（「mainagent(5b)」「subagent(c3)」），与 agent( / mainagent( / subagent(
    半角全角括号里的两位小写十六进制（「agent(cc)」「mainagent（5b）」）；with_session_word_code 为真时另认「会话」后面不带括号跟的两位短号（「（会话 4a，<日期>）」，
    后面不接 SESSION_WORD_CODE_AFTER 那几样）。两样平常都是真，弄坏开关 agent-parentheses-unjudged、session-word-code-unjudged 各换上一样为假的那一版。"""
    agent_parentheses = (rf'|(?<![{ASCII_WORD}])(?:[Mm]ain|[Ss]ub)[Aa]gent\s*\((?![A-Z]+\s*\))[^()\n]{{1,16}}\)'
                         rf'|(?<![{ASCII_WORD}])(?:[Mm]ain|[Ss]ub)?[Aa]gent\s*{SHORT_CODE_IN_PARENTHESES}') if with_agent_parentheses else ''
    session_word_code = rf'|(?<![{ASCII_WORD}])会话\s*[:：]?\s*[0-9a-f]{{2}}{SESSION_WORD_CODE_AFTER}' if with_session_word_code else ''
    return (rf'(?:(?:{SESSION_WORDS_BEFORE})\s*[「“"]?\s*{SHORT_CODE_IN_PARENTHESES}'
            rf'|(?<![{ASCII_WORD}]){SHORT_CODE_IN_PARENTHESES}(?=\s*(?:{SESSION_WORDS_AFTER}))'
            rf'|(?<![{ASCII_WORD}])[（(]\s*[0-9a-f]{{2}}(?:\s*[、和与,，]\s*[0-9a-f]{{2}})*\s*(?:{SESSION_WORDS_INSIDE})'
            rf'{agent_parentheses}{session_word_code}'
            rf'|(?P<bare_short_code>(?<![{ASCII_WORD}])[（(]\s*(?=[0-9a-f]{{0,1}}\d)(?=[0-9a-f]{{0,1}}[a-f])[0-9a-f]{{2}}\s*[)）]))'
            rf'(?:\s*[、和与]\s*{SHORT_CODE_IN_PARENTHESES})*')


SESSION_SHORT_CODE = session_short_code(True, True)
# 弄坏用：不认 mainagent( / subagent( / agent( 括号里的短号与别的写法（LIB_FORBIDDEN_NOTATIONS_BREAK=agent-parentheses-unjudged）
SESSION_SHORT_CODE_WITHOUT_AGENT_PARENTHESES_PATTERN = re.compile(session_short_code(False, True))
# 弄坏用：不认「会话」后面不带括号跟的短号（LIB_FORBIDDEN_NOTATIONS_BREAK=session-word-code-unjudged）
SESSION_SHORT_CODE_WITHOUT_SESSION_WORD_CODE_PATTERN = re.compile(session_short_code(True, False))
# 拿「<做什么>那一路」的描述指别的会话或子 agent：「那一路」后面（可隔「的」与空白）紧跟会话词，或紧跟会话做的事（回信、报、认领、转达、在改、说这类动作词）；
# 按出场次序的 mainagent(A)、subagent(A) 不在内。动作词里「报」后面接红、绿、错的（说一条跑法报了什么），「说」后面接法、明的（说法、说明）不算
SESSION_DESCRIPTION_WORDS = rf'[主子]?会话|[主子]\s*[Aa]gent(?![{ASCII_WORD}])|(?:[Mm]ain|[Ss]ub)?[Aa]gent(?![{ASCII_WORD}])'
SESSION_DESCRIPTION_ACTIONS = '回信|来信|回话|回复|答复|认领|认了|认下|已认|转达|通知|群发|报(?![红绿错])|说(?![法明])|在改|在修|交回'


def session_description(with_actions):
    """拿描述指别的会话那一条判据行的判据串；with_actions 为真时「那一路」后面紧跟 SESSION_DESCRIPTION_ACTIONS 的也算（平常是真，弄坏开关 session-description-actions-unjudged 换上假的那一版）。"""
    actions = f'|{SESSION_DESCRIPTION_ACTIONS}' if with_actions else ''
    return rf'那一路(?:的)?\s*(?:{SESSION_DESCRIPTION_WORDS}{actions})'


SESSION_DESCRIPTION = session_description(True)
# 弄坏用：「那一路」后面紧跟动作词的不算（LIB_FORBIDDEN_NOTATIONS_BREAK=session-description-actions-unjudged）
SESSION_DESCRIPTION_WITHOUT_ACTIONS_PATTERN = re.compile(session_description(False))
# 弄坏用：带「一路」的、mainagent( 与 subagent( 一律算（LIB_FORBIDDEN_NOTATIONS_BREAK=session-descriptions-loose）
SESSION_DESCRIPTION_LOOSE_PATTERN = re.compile(r'一路|(?:[Mm]ain|[Ss]ub)[Aa]gent\(')
# 「批 / 批次 / 批号」后面（可隔「为」「是」、冒号与空白）跟的中文数字（具名组 chinese_numeral_batch，只在 records/ 下判）：中文数字后面接量词、单位或这几个字的是数量与别的词
# （「这批两份补丁」「批三个实现员」「批一起改」「批三方审」「两批七问」「这一批一直」），与后面接「·」的小节号（「本批七·6」），不算
CHINESE_BATCH_NOT_AFTER = '张棵腿方起并路名本段节样些点篇套门场根批问直包槽几·'


def batch_number(compounds_counted, with_chinese_numerals=True):
    """批号那一条判据行的判据串；compounds_counted 见 quantity_after（平常是假：量词字后面接成词的不当量词）；
    with_chinese_numerals 为真时另认「批 / 批次 / 批号」后面跟的中文数字（平常是真，弄坏开关 chinese-numeral-batch-unjudged 换上假的那一版）。"""
    chinese_numeral_batch = (rf'|(?P<chinese_numeral_batch>(?<![每分{ASCII_WORD}])批(?:次号|次|号)?\s*(?:为|是)?\s*[:：]?\s*[{CHINESE_NUMERALS}]+)'
                             rf'(?![{CHINESE_NUMERALS}]|{quantity_after(compounds_counted, True)}|\s*[{CHINESE_BATCH_NOT_AFTER}{HEAVENLY_STEMS}])'
                             ) if with_chinese_numerals else ''
    return (rf'(?<![每分{ASCII_WORD}])批(?:次号|次|号)?\s*[:：]?\s*#?\s*\d+(?:-\d+)?'
            rf'(?![\d{ASCII_WORD}]|\.\d|{quantity_after(compounds_counted, False)}|\s*[{HEAVENLY_STEMS}])'
            rf'|(?<![{ASCII_WORD}])第\s*\d+(?:\s*[–\-~～、，,和与至到]\s*\d+)*\s*批(?!\s*[{HEAVENLY_STEMS}])'
            rf'|(?P<chinese_ordinal_batch>(?<![{ASCII_WORD}])第[{CHINESE_NUMERALS}]+批)(?!\s*[{HEAVENLY_STEMS}])'
            rf'{chinese_numeral_batch}')


BATCH_NUMBER = batch_number(False)
# 弄坏用：量词字后面接成词的也当量词（LIB_FORBIDDEN_NOTATIONS_BREAK=measure-compounds-counted）
BATCH_NUMBER_COMPOUNDS_COUNTED_PATTERN = re.compile(batch_number(True))
# 弄坏用：不认「批」后面跟的中文数字（LIB_FORBIDDEN_NOTATIONS_BREAK=chinese-numeral-batch-unjudged）
BATCH_NUMBER_WITHOUT_CHINESE_NUMERALS_PATTERN = re.compile(batch_number(False, False))
# 照写任务号的两种行：判决里行首的「A/B 读数：」一行（LINE_START、ZERO_ROUND_READING_LINE、ZERO_ROUND_TASK_NUMBER 三条与 .claude/hooks/runner-dispatch-guard.sh 里同名的三条逐字相同），
# 调度记录里以「长跑拆分：」起头的一行
LINE_START = r"^[ \t>*_\-]*"
ZERO_ROUND_READING_LINE = re.compile(LINE_START + r"A/B[ \t]*读数[* \t]*[：:][* \t]*(?P<value>.*?)[ \t]*$", re.M)
ZERO_ROUND_TASK_NUMBER = re.compile(r"(?:任务(?:号)?|task)[ \t#：:]*\d+", re.I)
ZERO_ROUND_READING_MOST_NUMBERS = 2   # A/B 两臂：「A/B 读数：」一行的值里认出的任务号多于这个数，一个都不放
ZERO_ROUND_VERDICT_PATH = re.compile(r'research/prompts/[^/]+-main-verification\.md')
LONG_RUN_SPLIT_LINE = re.compile(r'^\s*(?:[-*+]\s+|\d+[.)、]\s*)?长跑拆分\s*[:：]')
RECORDS_PREFIX = 'records/'   # 照写批号的表格列、「长跑拆分：」一行、中文数字的第几批与不挨会话词的括号短号只在这下面放过或判
BATCH_COLUMN_HEADER = '批次'   # records/ 下表格里照写批号的那一列的表头（依赖表批次那一列）
BATCH_SIZE_WORDS = re.compile(r'(?:seq|攒|追加|覆写|写入)\s*$')   # fsync 攒批的大小：「批 <数>」前面紧挨着这几个词
# 弄坏开关 batch-size-loosened 换上的认法：同一分句里紧挨着「批」或紧挨着这一处，分句边界是 CLAUSE_BREAKS
BATCH_SIZE_WORDS_IN_CLAUSE = re.compile(r'(?:seq|攒|追加|覆写|写入)\s*(?:批|$)')
CLAUSE_BREAKS = '。！？；;，,（）()'
# 弄坏开关 zero-round-reading-anywhere 换上的认法：一行里任何位置有「A/B 读数」加冒号就整行放过，不看文件
ZERO_ROUND_READING_ANYWHERE = re.compile(r'A/B 读数\s*[:：]')
FORMULA_BEFORE = re.compile(r'[+*/×÷(<>≤≥~≈^]\s*$')
FORMULA_AFTER = re.compile(r'\s*[+*/×÷)<>≤≥^;]')

# 上下文规则用的：紧挨在冒号对前面、说明它是代码或产物字段的字符
CODE_CHARACTERS_BEFORE = ('=', '[', '{', ',', '>', '"', "'", '+', '-', '.')
COLON_PAIR = re.compile(rf'(?<![{ASCII_WORD}:.])(\d{{1,3}}):(\d{{1,3}})(?![{ASCII_WORD}:])')
FILE_LINE_REFERENCE = re.compile(r'\.[A-Za-z]\w*:\d')


# 编号对按「句」读：句子的边界是 。！？（没有这几个就是整行；「；」不算，表格单元格与清单里常用它隔开并列的编号对）；这一句里有「文件:行号」或取值出了钟点范围的冒号对，这一句里的冒号对按编号读，
# 别的句子里的不受它牵连（日志文件名、行号引用与真钟点常写在同一行的不同句里）
SENTENCE_BREAKS = '。！？'


def numbered_pairs_in(text):
    """这段文字里的冒号对按编号读：有取值出了钟点范围的冒号对，或有「文件:行号」。"""
    if FILE_LINE_REFERENCE.search(text):
        return True
    return any(int(hour) > 23 or int(minute) > 59 or len(hour) > 2 or len(minute) != 2 for hour, minute in COLON_PAIR.findall(text))


def sentence_around(line, match):
    """这一行里命中所在的那一句（弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=numbered-pairs-whole-line 时是整行）。"""
    if 'numbered-pairs-whole-line' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return line
    start = max(line.rfind(mark, 0, match.start()) for mark in SENTENCE_BREAKS) + 1
    ends = [position for position in (line.find(mark, match.end()) for mark in SENTENCE_BREAKS) if position >= 0]
    return line[start:min(ends) if ends else len(line)]


# mountinfo 一行的开头：「挂载号 父挂载号 主:次设备号 根路径 挂载点 …」；设备号前面是两个十进制数各带一个空格，后面是「 /根路径 /挂载点」
MOUNTINFO_IDS_BEFORE = re.compile(r'(?<![\w:.])\d+ \d+ $')
MOUNTINFO_PATHS_AFTER = re.compile(r' /\S* /')


def is_mountinfo_device_number(line, match):
    """冒号对是 mountinfo 一行里的主:次设备号。弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=mountinfo-device-numbers 时一律不是。"""
    if 'mountinfo-device-numbers' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return False
    return (MOUNTINFO_IDS_BEFORE.search(line[:match.start()]) is not None
            and MOUNTINFO_PATHS_AFTER.match(line[match.end():]) is not None)


def bare_clock_is_not_clock(line, match, relative_path='', table_header=None):
    """不挨日期时区的钟点那一行的上下文规则：True 是这一处按上下文不是钟点。"""
    before, after = line[:match.start()], line[match.end():]
    if before.endswith(CODE_CHARACTERS_BEFORE) or before.endswith(', '):
        return True
    if before.endswith('`') and len(match.group(0).split(':')[0]) == 1:
        return True
    if after.startswith('%') or re.match(r'\.\d|-[A-Za-z]', after):
        return True
    if after.startswith(('=', '/')) or before.endswith(('@', ';', '#')):
        return True
    if is_mountinfo_device_number(line, match):
        return True
    return numbered_pairs_in(sentence_around(line, match))


def field_value(line, match, relative_path='', table_header=None):
    """紧跟在「=」后面的是产物行的字段值（名字=值）。"""
    return line[:match.start()].endswith('=')


def inside_code_span(line, position):
    """这一行的 position 落在代码段里：一串反引号开头、到后面第一串一样长的反引号为止（长短不限，``a`b`` 这种里面夹短串的照算一段）；
    后面找不到一样长的那一串的，这一串按字面算，不开代码段（同 CommonMark 的行内代码）。"""
    index = 0
    while index < len(line):
        if line[index] != '`':
            index += 1
            continue
        opening_end = index
        while opening_end < len(line) and line[opening_end] == '`':
            opening_end += 1
        width = opening_end - index
        closing = re.compile(rf'(?<!`)`{{{width}}}(?!`)').search(line, opening_end)
        if closing is None:
            index = opening_end
            continue
        if opening_end <= position < closing.start():
            return True
        index = closing.end()
    return False


def batch_code_is_not_code(line, match, relative_path='', table_header=None):
    """批次代号四条判据行的上下文规则：True 是这一处按上下文不是批次代号——反引号代码段里的（inside_code_span）、「=」（可隔空白）后面的字段值、
    紧挨在 " ' , ; 后面的代码字面与参数、算式里的。弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=code-span-backtick-parity 时只数前面反引号的奇偶。"""
    before = line[:match.start()]
    if 'code-span-backtick-parity' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        if before.count('`') % 2 == 1:
            return True
    elif inside_code_span(line, match.start()):
        return True
    if re.search(r'=\s*$', before) or before.endswith(('"', "'", ',', ';')):
        return True
    return FORMULA_BEFORE.search(before) is not None or FORMULA_AFTER.match(line[match.end():]) is not None


def under_records(relative_path):
    """这份文件在 records/ 下；路径没给（空串）的按在 records/ 下算（照判）。"""
    return not relative_path or relative_path.startswith(RECORDS_PREFIX)


def session_code_is_kept(line, match, relative_path='', table_header=None):
    """session-short-codes 的上下文规则：反引号代码段里的，与不挨会话词的数字字母括号短号（具名组 bare_short_code）既不在 records/ 下、也不在判决里的，不算。
    弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=bare-short-codes-everywhere 时后一种在任何文件里都算。"""
    if inside_code_span(line, match.start()):
        return True
    if match.group('bare_short_code') is None or 'bare-short-codes-everywhere' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return False
    return not (under_records(relative_path) or ZERO_ROUND_VERDICT_PATH.fullmatch(relative_path) is not None)


def session_description_is_kept(line, match, relative_path='', table_header=None):
    """session-descriptions 的上下文规则：反引号代码段里的不算（同 session_code_is_kept 的头一条），不看路径。"""
    return inside_code_span(line, match.start())


QUOTE_BLOCK_LINE = re.compile(r'[ \t]*(?:(?:[-*+]|\d+[.)])[ \t]+)*>')   # 引用块：去掉行首空白与列表记号（-、*、+、数字.、数字)）之后以「>」起头


def quoted_line(line):
    """这一行是引用块里的一行（QUOTE_BLOCK_LINE；列表项里的引用块「- > …」同样算）。
    弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=zero-round-reading-list-quote-kept 时只认去掉行首空白后以「>」起头的。"""
    if 'zero-round-reading-list-quote-kept' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return line.lstrip().startswith('>')
    return QUOTE_BLOCK_LINE.match(line) is not None


def zero_round_reading_numbers(line):
    """这一行是派发闸认的「A/B 读数：」一行（ZERO_ROUND_READING_LINE；引用块里的一行不算，认法见 quoted_line）时，值里按派发闸的 ZERO_ROUND_TASK_NUMBER
    认出的每一处任务号在这一行里的 (起, 止)；不是这样的一行交 None。弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=zero-round-reading-quote-kept 时引用块也算。"""
    reading = ZERO_ROUND_READING_LINE.match(line)
    if reading is None:
        return None
    if quoted_line(line) and 'zero-round-reading-quote-kept' not in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return None
    value_start = reading.start('value')
    return [(value_start + found.start(), value_start + found.end()) for found in ZERO_ROUND_TASK_NUMBER.finditer(reading.group('value'))]


def task_number_is_kept(line, match, relative_path='', table_header=None):
    """task-numbers 的上下文规则：照写任务号的两种行里的不算——判决（research/prompts/*-main-verification.md）里「A/B 读数：」一行（zero_round_reading_numbers）
    认出的任务号不多于 ZERO_ROUND_READING_MOST_NUMBERS 个时，起止与认出的某一处相同的那一处；records/ 下（去掉列表记号之后）以「长跑拆分：」起头的一行。反引号代码段里的照算。
    弄坏开关（LIB_FORBIDDEN_NOTATIONS_BREAK）：task-numbers-in-code-span-kept 时反引号代码段里的不算；zero-round-reading-anywhere 时一行里任何位置有「A/B 读数：」、
    任何文件都整行不算；zero-round-reading-count-unchecked 时认出的不核个数；reserved-lines-any-file 时「长跑拆分：」一行在任何文件里都不算。"""
    broken = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK')
    if 'task-numbers-in-code-span-kept' in broken and inside_code_span(line, match.start()):
        return True
    if LONG_RUN_SPLIT_LINE.match(line) and ('reserved-lines-any-file' in broken or (relative_path and relative_path.startswith(RECORDS_PREFIX))):
        return True
    if 'zero-round-reading-anywhere' in broken:
        return ZERO_ROUND_READING_ANYWHERE.search(line) is not None
    if ZERO_ROUND_VERDICT_PATH.fullmatch(relative_path or '') is None:
        return False
    recognized = zero_round_reading_numbers(line)
    if not recognized or (len(recognized) > ZERO_ROUND_READING_MOST_NUMBERS and 'zero-round-reading-count-unchecked' not in broken):
        return False
    return (match.start(), match.end()) in recognized


TABLE_SEPARATOR_ROW = re.compile(r'\s*\|?\s*:?-+:?\s*(?:\|\s*:?-+:?\s*)*\|?\s*')


def unescaped_pipe_positions(line):
    """这一行里分格的「|」的下标：前面紧挨着的反斜杠是偶数个（零个也算）的才分格，奇数个的是转义的竖线（「\\\\|」是转义的反斜杠加分格的竖线）。
    弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=table-escaped-pipe-parity-ignored 时前面挨着反斜杠的一律当转义。"""
    parity_ignored = 'table-escaped-pipe-parity-ignored' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK')
    positions = []
    for index, character in enumerate(line):
        if character != '|':
            continue
        backslashes = len(line[:index]) - len(line[:index].rstrip('\\'))
        if backslashes == 0 or (not parity_ignored and backslashes % 2 == 0):
            positions.append(index)
    return positions


def table_cells(line):
    """以「|」起头的一行切成单元格（不带两头的空格与粗体记号 *、_）：按分格的「|」（unescaped_pipe_positions）切，去掉头一个「|」前面与最后一个「|」后面的空段。"""
    stripped = line.strip()
    bounds = [-1] + unescaped_pipe_positions(stripped) + [len(stripped)]
    pieces = [stripped[start + 1:end] for start, end in zip(bounds, bounds[1:])]
    if pieces and pieces[0] == '':
        pieces = pieces[1:]
    if pieces and pieces[-1].strip() == '':
        pieces = pieces[:-1]
    return [piece.strip().strip('*_').strip() for piece in pieces]


def table_headers_by_line(lines):
    """每一行所在那张表的表头（table_cells 切好的单元格），不在表里的行（表头那一行与分隔行也算不在）交 None。
    以「|」起头（去掉行首空白）的连续几行、第二行是分隔行（TABLE_SEPARATOR_ROW）的才是表，头一行是表头；没有分隔行的那几行不是表。"""
    headers = [None] * len(lines)
    index = 0
    while index < len(lines):
        if not lines[index].lstrip().startswith('|'):
            index += 1
            continue
        end = index
        while end < len(lines) and lines[end].lstrip().startswith('|'):
            end += 1
        if end - index >= 2 and TABLE_SEPARATOR_ROW.fullmatch(lines[index + 1]):
            header = table_cells(lines[index])
            for row in range(index + 2, end):
                headers[row] = header
        index = end
    return headers


def column_header(line, position, table_header):
    """表格行 line 里 position 落在的那一格的表头；不在表里、或那一格超出表头的交 None。"""
    if table_header is None:
        return None
    column = len(unescaped_pipe_positions(line[:position])) - (1 if line.lstrip().startswith('|') else 0)
    return table_header[column] if 0 <= column < len(table_header) else None


def batch_number_is_kept(line, match, relative_path='', table_header=None):
    """batch-numbers 的上下文规则：反引号代码段里的；不在 records/ 下的中文数字「第几批」与「批」后面跟的中文数字（具名组 chinese_ordinal_batch、chinese_numeral_batch）；
    records/ 下表格里表头是 BATCH_COLUMN_HEADER 的那一列的单元格（table_header 是这一行所在那张表的表头）；
    不在 records/ 下、「批 <数>」前面紧挨着 seq、攒、追加、覆写、写入的（fsync 攒批的大小，BATCH_SIZE_WORDS）。
    弄坏开关（LIB_FORBIDDEN_NOTATIONS_BREAK）：reserved-lines-any-file 时以「|」起头的行在任何文件里、任何一列都不算；table-batch-any-column 时 records/ 下
    以「|」起头的行任何一列都不算；batch-size-loosened 时同一分句里前面有那几个词就不算（records/ 下与「第 N 批」写法也一样，BATCH_SIZE_WORDS_IN_CLAUSE）。"""
    broken = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK')
    if inside_code_span(line, match.start()):
        return True
    chinese_numerals = match.group('chinese_ordinal_batch') is not None or match.groupdict().get('chinese_numeral_batch') is not None
    if chinese_numerals and not under_records(relative_path):
        return True
    if line.lstrip().startswith('|'):
        if 'reserved-lines-any-file' in broken:
            return True
        if relative_path and relative_path.startswith(RECORDS_PREFIX) and (
                'table-batch-any-column' in broken or column_header(line, match.start(), table_header) == BATCH_COLUMN_HEADER):
            return True
    before = line[:match.start()]
    if 'batch-size-loosened' in broken:
        clause = before[max(before.rfind(mark) for mark in CLAUSE_BREAKS) + 1:]
        return BATCH_SIZE_WORDS_IN_CLAUSE.search(clause) is not None
    return not under_records(relative_path) and match.group(0).startswith('批') and BATCH_SIZE_WORDS.search(before) is not None


FENCE_LINE = re.compile(r' {0,3}(`{3,}|~{3,})(.*)')


def fenced_line_flags(lines):
    """每一行在不在围栏代码块里（开、收两行算在里面，没收的护到末尾）：0 到 3 个空格缩进的 ``` 或 ~~~ 开，反引号围栏的信息串里不许有反引号；
    同字符、不短于开头、后面只有空白的那一行收。认法同 scripts/kb/lib_markdown.py 的 fenced_flags（这份库会被单独抄进自证的假仓，不从别处 import）。"""
    flags, opened = [], None
    for line in lines:
        inside, opened = fence_step(line, opened)
        flags.append(inside)
    return flags


def fence_step(line, opened):
    """读一行：(这一行在不在围栏里, 读完之后开着的围栏或 None)。opened 是读这一行之前开着的围栏（(字符, 长度)），None 是没开着。"""
    fence = FENCE_LINE.fullmatch(line)
    if fence and opened is None:
        marker, tail = fence.groups()
        if not (marker[0] == '`' and '`' in tail):
            return True, (marker[0], len(marker))
        return False, None
    if fence and fence.group(1)[0] == opened[0] and len(fence.group(1)) >= opened[1] and not fence.group(2).strip():
        return True, None
    return opened is not None, opened


def fenced_lines(form_key, lines):
    """这一形态在这几行里哪几行落在围栏代码块里（行下标的集合）；skips_fenced_blocks 没登记判据行的形态交空集合。
    弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=fenced-blocks-judged 时交空集合。"""
    if not FORMS_BY_KEY[form_key].get('skips_fenced_blocks') or 'fenced-blocks-judged' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return set()
    return {index for index, inside in enumerate(fenced_line_flags(lines)) if inside}


def line_contexts(form_key, lines):
    """整段文字的逐行上下文：(围栏里的行下标集合, 每一行的表头列表)；不读表头的形态（reads_table_headers 没写）表头一律 None。"""
    headers = table_headers_by_line(lines) if FORMS_BY_KEY[form_key].get('reads_table_headers') else [None] * len(lines)
    return fenced_lines(form_key, lines), headers


def patterns_for_line(form_key, patterns, fenced):
    """这一行要判的判据行：围栏代码块里的行去掉 skips_fenced_blocks 点名的那几条（弄坏开关 fenced-blocks-all-rows 时一条不判）。"""
    if not fenced:
        return patterns
    if 'fenced-blocks-all-rows' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return []
    skipped_keys = FORMS_BY_KEY[form_key].get('skips_fenced_blocks') or ()
    return [pattern for pattern in patterns if pattern[1] not in skipped_keys]


UPSTREAM_COPY = '.claude/singlefs-ai-sop/'
C510_REASON = 'c510 那一轮论证的就是按哪个时区取日期，时区词与钟点是被测算法本身'

FORMS = (
    {
        'key': 'prime-marks',
        'name': '角标',
        'patterns': (
            ('prime-marks', '撇号类角标', re.compile('[' + re.escape(PRIME_MARKS) + ']'), None),
        ),
        'excluded_prefixes': (
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
            ('research/prompts/m3-defs-b6-r2-opus-model/out/', '三方腿探针跑 doc-text 的原样输出，引了这一格成功句里举的角标例子；冻结证据不改'),
        ),
        'allow_suffixes': (),
        'judged_suffixes': (),
        'clean': f'没有角标写法（{MARKS_SHOWN}）',
        'excluded_label': '排除前缀下 {count} 份（上游副本 ' + UPSTREAM_COPY + '，与腿的探针输出 research/prompts/m3-defs-b6-r2-opus-model/out/）',
        'finding': f'行用了角标写法（{MARKS_SHOWN}）',
        'detail_shows_pattern': False,
        'snippet_to_match_end': False,
        'remedy': (
            '给这个变体起一个新名字——在它那一族里取下一个没用过的号，或起一个短的描述性名字，全仓一次改完；',
            '撞号先查（新名字在它那一族的全部文件里零命中）。做法见 .claude/rules/path-moves.md「变体起新名字，不用角标」。',
        ),
    },
    {
        'key': 'clock-times',
        'name': '钟点',
        'patterns': (
            ('date-clock', '日期后面跟钟点', re.compile(rf'{DATE} ?(?:{ZONE} ?)?{CLOCK_RANGE}'), field_value),
            ('zone-clock', '时区后面跟钟点', re.compile(rf'{ZONE} ?{NEXT_DAY}{CLOCK_RANGE}'), None),
            ('clock-zone', '钟点后面跟时区', re.compile(rf'{CLOCK_RANGE} ?{ZONE}'), None),
            ('x-wildcard', '带 x 通配的钟点', re.compile(r'(?<![\w:])(?:\d{1,2}|\dx):(?:\dx|xx)'), None),
            ('bare-z', '不带日期的 Z 钟点', re.compile(r'(?<![\w:`])\d{1,2}:\d\d(?::\d\d)?Z(?!\w)'), None),
            ('date-zone-hour', '日期或时区后面跟「N 点 / N 时」', re.compile(rf'(?:{DATE}|{ZONE})\s*(?:{ZONE}\s*)?{NEXT_DAY}{HOUR}'), None),
            ('date-zone-period', '日期或时区后面跟时段词', re.compile(rf'(?:{DATE}(?:\s*{ZONE})?|{ZONE})[\s（(，]*{NEXT_DAY}{PERIOD}'), None),
            ('period-hour', '时段词后面跟钟点', re.compile(rf'{PERIOD}\s*(?:{HOUR}|{CLOCK})'), None),
            ('hour-suffix', '「N 点前后」这类钟点', re.compile(r'(?<![第\d])(?<!第 )\d{1,2}\s*(?:点(?:前后|左右|许|半|钟)|时许)'), None),
            ('twelve-hour', '十二小时制钟点', re.compile(TWELVE_HOUR), None),
            ('bare-clock', '不挨日期时区的钟点', re.compile(BARE_CLOCK), bare_clock_is_not_clock),
            ('iso-timestamp', 'ISO 时间戳', re.compile(ISO_TIMESTAMP), field_value),
            ('iso-compact-timestamp', 'ISO 紧凑时间戳', re.compile(ISO_COMPACT_TIMESTAMP), None),
            ('zone-word', '时区词', re.compile(ZONE_WORD), None),
        ),
        'excluded_prefixes': (
            ('.claude/session-checks/kernel-memory-kills', '自检用的假 journalctl 行与它打出来的期望输出，钟点是被测输入'),
            ('research/prompts/_gate-shrink-r1-diff.md', '冻结证据：那一轮改动前后的 diff，旧文里的时区词是被改掉的原文'),
            ('research/prompts/gate-shrink-r2-sync-evidence/changes-scoped.md', '冻结证据：逐条抄录改前旧文（「旧：」行），钟点与时区词是被改掉的原文'),
            ('research/prompts/abandoned-floor-r1-opus-model/', '冻结证据：腿的模型目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/c245-r3-opus-model/', '冻结证据：腿的模型目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/closeout-recheck-2026-09-28/', '冻结证据：收口复核留的 diff，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/k3-9-barrier-r1-opus-model/patches/', '冻结证据：腿的补丁目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/unreadable-at-mount-r1-opus-model/runs/', '冻结证据：腿的运行日志，内存包装打出的 ISO 时间戳是原样输出'),
            ('research/prompts/unreadable-at-mount-r2-opus-model/', '冻结证据：腿的模型目录，diff 头带 diff 工具写的机器时间'),
            ('.claude/gate.d/fixtures/', '门禁样本：造现场的 touch -d、GIT_COMMITTER_DATE 与门禁打出来的期望输出'),
            ('research/prompts/m3-defs-b6-r2-opus-model/', '冻结证据：腿的模型目录，喂样本的 setup.sh 与它的输出里的 ISO 时间戳是被测输入'),
            ('research/prompts/m3-defs-b9-r1-opus-model/', '冻结证据：腿的模型目录，喂样本的 setup.sh、样本与输出里的 ISO 时间戳是被测输入'),
            ('research/prompts/m3-defs-b9-r2-opus-model/', '冻结证据：腿的模型目录，探针脚本里的时间戳是被测输入'),
            ('research/prompts/m3-defs-b9-r1-opus-output.md', '冻结证据：腿的报告，抄了它模型输出里的时间戳'),
            ('research/prompts/m3-admission-r1-main-verification.md', '冻结证据：那一轮的判决，记了冻结副本被别的会话通删与改动的钟点'),
            ('research/prompts/m3-close-noncode-candidates.tsv', '冻结证据：回扫候选表整行抄录记录里的旧文，旧文带钟点'),
            ('research/prompts/m3-close-noncode-sync.md', '冻结证据：阶段同步记录整行抄录记录里的旧文，旧文带钟点'),
            ('research/prompts/agentdef-r2-main-verification.md', '冻结证据：那一轮的判决，里面「决策号:行号」写法的冒号对（决策 06 的第 42 行）不是钟点，同一句里没有「文件:行号」可认'),
            ('research/results/e142-mutate-r25-s1-2026-10-13.log', '实验产物：变异驱动的原样输出带时间戳，复跑要逐字相同'),
            ('research/results/e71-accounting-keys-single-generation-mutations-2026-10-20.log', '实验产物：变异驱动的原样输出，内存包装报的上一次峰值带量的时间戳'),
            ('.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/', '钩子样本：派发闸自证读的真派发提示原样，提示里抄的钟点与时间戳是被测输入'),
            ('research/prompts/_rot-defs-r2-background.md', '冻结证据：那一轮的材料原样贴了 git log 输出，Date 行带提交的机器时间'),
            ('research/prompts/_rot-defs-r2-body.md', '冻结证据：那一轮的材料原样贴了 git log 输出，Date 行带提交的机器时间'),
            ('research/prompts/apparatus-rename-r1-opus-model/attack_samples.py', '冻结证据：腿的探针，造样本提交的 GIT_AUTHOR_DATE 是被测输入'),
            ('research/prompts/batch-2026-10-21-facts.tsv', '冻结证据：那一批的事实表，举了被误判成钟点的设备号写法，不是钟点'),
            ('research/prompts/batch-2026-10-21-judge-6.md', '冻结证据：那一批的判决，「文件:行号」写法的冒号对不是钟点，同一句里没有带后缀的文件名可认'),
            ('research/prompts/landing-audit-r1-opus-model/tools/outcome_check.py', '冻结证据：腿的工具，按会话记录的时区切天，时区词是代码字面'),
            ('research/prompts/landing-audit-r1-snapshot/judged-vs-index.diff', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里带时区词的代码'),
            ('research/prompts/landing-audit-r2-opus-model/', '冻结证据：腿的模型目录，会话记录的原样导出与探针里的时间戳、时区词是被测输入'),
            ('research/prompts/landing-audit-r2-opus-output.md', '冻结证据：腿的报告，论证的就是按哪个时区切天'),
            ('research/prompts/landing-audit-r2-snapshot/judged-vs-index.diff', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里带时区词的代码'),
            ('research/prompts/landing-audit-r3-opus-model/', '冻结证据：腿的模型目录，会话记录的原样导出、真派发提示与探针里的时间戳、时区词是被测输入'),
            ('research/prompts/landing-audit-r3-opus-output.md', '冻结证据：腿的报告，论证的就是按哪个时区切天，抄了窗口边界的时间戳'),
            ('research/prompts/landing-audit-r3-snapshot/judged-vs-index.diff', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里带时区词的代码'),
            ('research/prompts/landing-audit-r4-opus-model/probes/naming_probe.py', '冻结证据：腿的探针，造会话记录的时间戳是被测输入'),
            ('research/prompts/landing-audit-r4-snapshot/judged-vs-head.diff', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里的时区词与运行目录名'),
            ('research/prompts/apparatus-rename-r2-snapshot/judged-vs-head.diff', '冻结证据：那一轮开工快照的 diff，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/apparatus-rename-r2-opus-model/attack_samples.py', '冻结证据：腿的探针，造样本提交的 GIT_AUTHOR_DATE 是被测输入'),
            ('research/prompts/apparatus-rename-r2-sonnet-model/tools/probe_h3.py', '冻结证据：腿的探针，样本行里的钟点是被测输入'),
            ('research/prompts/landing-audit-r5-opus-model/', '冻结证据：腿的模型目录，被判钩子的原件、会话记录导出与探针里的时间戳、时区词是被测输入'),
            ('research/prompts/landing-audit-r5-snapshot/judged-vs-head.diff', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里的时区词与运行目录名'),
            ('research/prompts/landing-audit-r6-opus-model/', '冻结证据：腿的模型目录，被判钩子的原件、探针与它的输出里的时间戳、时区词是被测输入'),
            ('research/prompts/landing-audit-r6-opus-output.md', '冻结证据：腿的报告，抄了探针输出里按时区切日期的汇总行'),
            ('research/prompts/landing-audit-r6-snapshot/judged-vs-head.diff', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里的时区词、运行目录名与真派发提示'),
            ('research/prompts/m3-close-c506-r2-opus-model/patches/', '冻结证据：腿的补丁目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m3-close-c656-r1-opus-model/patches/', '冻结证据：腿的补丁目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m3-close-c656-r2-sonnet-model/patches/', '冻结证据：腿的补丁目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m3-close-reuse-closure-r1-opus-model/', '冻结证据：腿的模型目录，造现场的时间戳与内存包装打出的起跑时间戳是被测输入与原样输出'),
            ('research/prompts/m3-close-reuse-closure-r2-opus-model/', '冻结证据：腿的模型目录，造现场的时间戳与内存包装打出的起跑时间戳是被测输入与原样输出'),
            ('research/prompts/m4-c860-r1-opus-s2-model/patches/', '冻结证据：腿的补丁目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m4-c860-r1-sonnet-model/patch/', '冻结证据：腿的补丁目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m4-i315-r1-sonnet-model/probes/instrumentation.patch', '冻结证据：腿的探针补丁，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m4-publish-lookup-code-r1-snapshot/reversal/b6-c860/', '冻结证据：那一轮回滚核对留的 diff 与说明，diff 头与说明里是拷贝时的机器时间'),
            ('research/prompts/m4-publish-lookup-code-r2-opus-model/out/runs-logs/', '冻结证据：腿的运行日志，内存包装报的上一次峰值带量的时间戳，是原样输出'),
            ('research/prompts/m4-publish-lookup-followup-code-r1-opus-model/tools/probe.patch', '冻结证据：腿的探针补丁，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/rules-compress-r2-merged/diff-from-r1.patch', '冻结证据：规则压缩相邻两棵合稿树的 diff，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/rules-compress-r3-merged/diff-from-r2.patch', '冻结证据：规则压缩相邻两棵合稿树的 diff，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/rules-compress-r4-merged/diff-from-r3.patch', '冻结证据：规则压缩相邻两棵合稿树的 diff，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/subagent-landing-audit-2026-10-22/', '冻结证据：那一次落地审计的逐组记录，抄了会话记录里的钟点、运行目录名与按时区切天的原文'),
            ('research/prompts/apparatus-rename-r3-opus-model/attack_samples.py', '冻结证据：腿的探针，造样本提交的 GIT_AUTHOR_DATE 是被测输入'),
            ('research/prompts/landing-audit-r7-snapshot/', '冻结证据：那一轮开工快照的 diff，抄了被判脚本里的时区词、运行目录名与真派发提示'),
            ('research/prompts/landing-audit-r7-sonnet-output.md', '冻结证据：腿的报告，抄了报告里的时间戳与钟点'),
            ('research/prompts/m3-close-gov-b17-r2-opus-output.md', '冻结证据：腿的报告，抄了 stat 机器时间'),
            ('research/prompts/m3-close-gov-b17-r3-opus-model/', '冻结证据：腿的模型目录，diff 头带 diff 工具写的机器时间'),
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
            ('research/prompts/_c510-date-gate-r1-appendix.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-attack-output.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-background.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-body.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-checklist.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-forward-output.md', C510_REASON),
            ('research/prompts/_c510-date-gate-r1-local-attack-output.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-output-s1.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-output-s2.md', C510_REASON),
            ('research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-translation-audit.md', C510_REASON),
            ('research/prompts/c510-date-gate-r1-verifier-output.md', C510_REASON),
            ('research/prompts/crash-slice-false-resume-candidates.tsv', '冻结证据：回扫候选表，抄了旧记录带时间戳的原文'),
            ('research/prompts/d165-valves-r1-sonnet-output.md', '冻结证据：腿的模型报告，时间戳是被测输入'),
            ('research/prompts/m3-close-b19-code-r3-sonnet-model/', '冻结证据：腿的模型目录，diff 头带 diff 工具写的机器时间'),
            ('research/prompts/m3-close-b19-design-r2-sonnet-output.md', '冻结证据：腿的模型报告，时间戳是被测输入'),
        ),
        'allow_suffixes': ('.sh', '.py', '.rs'),
        'judged_suffixes': (),
        'clean': '描述里没有钟点、时区词与时间戳',
        'excluded_label': '钟点的排除前缀下 {count} 份（钟点是被测输入，或上游副本）',
        'finding': '行在描述里写了钟点、时区词或时间戳',
        'detail_shows_pattern': True,
        'snippet_to_match_end': True,
        'remedy': (
            '时间只写到日期，把钟点连同挨着它的时区词、「前后」一起删掉（例：「用户 <日期> <时区词> <时:分> 定」写成「用户 <日期> 定」）；',
            '「N 点 / N 时」与时段词同样删（例：「<日期> <时区词> <N> 点前后要开」「交回于 <日期> <上午 / 下午>」「<时段词> <N> 点」'
            '都写成「<日期> 要开」「交回于 <日期>」这样只到日期）；',
            '光秃的时区词（lib-forbidden-notations.py 的 ZONE_WORDS）也删；ISO 时间戳（含运行目录名、日志文件名里连写的紧凑写法，路径里的也判：目录名里的时间戳段删掉，只写日期或「<运行目录>」）、'
            'diff 头与 stat 的机器时间、date 的原样输出不贴进仓，只写日期；',
            '命中的冒号对其实不是钟点（产物字段、切片、编号对），改那份库里「不挨日期时区的钟点」的上下文规则，并在 doc-text.sh 的 clock-times-green 样本补一行；',
            '.sh、.py、.rs 里代码真要用这些字面（TZ 环境变量、解析外部文本的正则、造时间戳的自检输入），在那一行写'
            '「# clock-times:allow <理由>」（Rust 写「//」），理由不许省；',
            '钟点确实是被测输入的整份文件，登记进 .claude/gate.d/lib-forbidden-notations.py 里钟点那一形态的 excluded_prefixes 并写明理由。',
        ),
    },
    {
        'key': 'session-names',
        'name': '会话号',
        'patterns': (
            ('session-names', '会话号', re.compile(SESSION_NAME), None),
        ),
        'excluded_prefixes': (
            ('.claude/gate.d/fixtures/', '门禁样本：会话号是被测输入，由 setup.sh 拆开拼出'),
            ('research/prompts/m3-admission-r1-main-verification.md', '冻结证据：那一轮的判决，点名了改动冻结副本的会话'),
            ('research/prompts/m3-close-noncode-candidates.tsv', '冻结证据：回扫候选表整行抄录记录里的旧文，旧文带会话号'),
            ('research/prompts/m3-close-noncode-sync.md', '冻结证据：阶段同步记录整行抄录记录里的旧文，旧文带会话号'),
            ('research/prompts/m3-close-noncode-judge-f14-f27.md', '冻结证据：那一轮逐行判决抄录记录里的旧文，旧文带会话号'),
            ('research/prompts/m3-defs-b3-r3-sonnet-output.md', '冻结证据：腿的报告，引了派发提示里的会话号'),
            ('research/prompts/m3-defs-b3-r3-sonnet-model/', '冻结证据：腿的模型目录，探针脚本里的会话号是被测输入'),
            ('research/prompts/_m3-close-c409-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-c656-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-c656-r2-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-c656-r3-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-c693-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-c693-r2-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-c693-r3-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-rootring-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-rootring-r2-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-rootring-r3-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-sysconf-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-sysconf-r2-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-close-sysconf-r3-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/landing-audit-r2-snapshot/', '冻结证据：那一轮开工快照对基准的 diff，抄了被判记录当时的原文，原文带会话号'),
            ('research/prompts/landing-audit-r3-snapshot/', '冻结证据：那一轮开工快照对基准的 diff，抄了被判记录当时的原文，原文带会话号'),
            ('research/prompts/landing-audit-r4-snapshot/', '冻结证据：那一轮开工快照对基准的 diff，抄了被判记录当时的原文，原文带会话号'),
            ('research/prompts/landing-audit-r5-snapshot/', '冻结证据：那一轮开工快照对基准的 diff，抄了被判记录当时的原文，原文带会话号'),
            ('research/prompts/landing-audit-r6-snapshot/', '冻结证据：那一轮开工快照对基准的 diff，抄了被判记录当时的原文，原文带会话号'),
            ('research/prompts/landing-audit-r6-sonnet-model/inputs/dispatch-record.md', '冻结证据：腿的模型目录里拷的调度记录当时那一版，带会话号'),
            ('research/prompts/subagent-landing-audit-2026-10-22/no-effect.md', '冻结证据：那一批交回报告的归档，引了别的会话的会话号'),
            ('research/prompts/subagent-landing-audit-2026-10-22/b9-watch-cost.md', '冻结证据：那一批交回报告的归档，引了别的会话的会话号'),
            ('research/prompts/_m3-multi-file-unlink-r1-appendix.md', '冻结证据：三方材料整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m3-multi-file-unlink-r1-background.md', '冻结证据：三方材料整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_rot-defs-r2-appendix.md', '冻结证据：三方材料整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_rot-defs-r2-background.md', '冻结证据：三方材料整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_rot-defs-r3-appendix.md', '冻结证据：三方材料整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_rot-defs-r3-background.md', '冻结证据：三方材料整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-batch2-code-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-c860-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-gc-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-segment-code-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-snapshot-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-trunc-r1-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            ('research/prompts/_m4-trunc-r2-', '冻结证据：三方材料（附录与背景）整段抄了 checks-owed.md 当时那一格，那一格带会话号'),
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
        ),
        'allow_suffixes': ('.sh', '.py', '.rs'),
        'judged_suffixes': (),
        'clean': '没有会话号',
        'excluded_label': '会话号的排除前缀下 {count} 份（门禁样本或上游副本）',
        'finding': '行写了会话号',
        'detail_shows_pattern': False,
        'snippet_to_match_end': True,
        'remedy': (
            '会话号（ListAgents 列的「目录名-十六进制」全名）与后面的两位短号都不写进仓：'
            '别的会话写成 mainagent(A)、mainagent(B)，子 agent 写成 subagent(A)、subagent(B)，同一份文字里按出场次序编字母，写它改了哪个文件、做了什么；'
            '不写「<做什么>那一路的会话」这类描述；标签式的「（会话 <号>，<日期>）」只留日期，事情与日期照留；',
            '改过标题的会话（「会话「<标题>」」）门禁认不出，照同样的写法改；',
            '.sh、.py、.rs 里代码真要用这些字面（解析 ListAgents 输出的正则、造会话号的自检输入），在那一行写「# session-names:allow <理由>」，理由不许省。',
        ),
    },
    {
        'key': 'batch-codes',
        'name': '批次代号',
        'patterns': (
            ('batch-code-suffixed', '数字加字母加短横序号的代号', re.compile(BATCH_CODE_SUFFIXED), batch_code_is_not_code),
            ('batch-code-lettered', '两位数字加字母的代号', re.compile(BATCH_CODE_LETTERED), batch_code_is_not_code),
            ('batch-code-labeled', '挨着批次词的数字加字母', re.compile(BATCH_CODE_LABELED), batch_code_is_not_code),
            ('batch-code-stem', '天干编号的批号', re.compile(BATCH_CODE_STEM), batch_code_is_not_code),
        ),
        'excluded_prefixes': (
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
            ('.claude/gate.d/fixtures/', '门禁样本：批次代号是被测输入'),
            ('.claude/hooks/fixtures/', '钩子样本：派发闸自证读的真派发提示原样，提示里的批号是被测输入'),
            ('crates/mutations.tsv', '变异表：源码原文那几列里有 WGSL 的「31u」这类字面，名字列里的批号是变异行的键，改名照变异表的改名办'),
        ),
        'allow_suffixes': (),
        'judged_suffixes': ('.md', '.tsv', '.txt'),
        'clean': '描述里没有批次代号',
        'excluded_label': '批次代号的排除前缀下 {count} 份（上游副本、样本目录与变异表）',
        'finding': '行写了批次代号',
        'detail_shows_pattern': True,
        'snippet_to_match_end': True,
        'remedy': (
            '写成那一批做了什么事（例「24c-1 交回」写成「配置判法那一份补丁交回」、「批 10 甲」写成「GPU 判器那一批」），不用代号；',
            '命中的其实不是代号（单位、算式、产物字段、代码字面），改 .claude/gate.d/lib-forbidden-notations.py 里批次代号的上下文规则或 UNIT_LETTERS；'
            '代码字面放进反引号；整份是被测输入的，登记进那一形态的 excluded_prefixes 并写明理由。',
        ),
    },
    {
        'key': 'task-and-session-numbers',
        'name': '任务号、会话短号与批号',
        'patterns': (
            ('task-numbers', '任务服务的任务号', re.compile(TASK_NUMBER), task_number_is_kept),
            ('session-short-codes', '括号里的会话短号', re.compile(SESSION_SHORT_CODE), session_code_is_kept),
            ('batch-numbers', '批号', re.compile(BATCH_NUMBER), batch_number_is_kept),
            ('session-descriptions', '拿描述指别的会话', re.compile(SESSION_DESCRIPTION), session_description_is_kept),
        ),
        'excluded_prefixes': (
            (UPSTREAM_COPY, '上游副本，只能在上游改'),
            ('.claude/gate.d/fixtures/', '门禁样本：任务号、会话短号与批号是被测输入'),
            ('.claude/hooks/fixtures/', '钩子样本：派发闸自证读的真派发提示原样，提示里的批号是被测输入'),
        ),
        'allow_suffixes': (),
        'judged_suffixes': ('.md', '.tsv', '.txt'),
        'skips_fenced_blocks': ('session-short-codes', 'batch-numbers', 'session-descriptions'),
        'reads_table_headers': True,
        'clean': '描述里没有任务号、会话短号与批号',
        'excluded_label': '任务号、会话短号与批号的排除前缀下 {count} 份（上游副本与样本目录）',
        'finding': '行写了任务号、会话短号或批号',
        'detail_shows_pattern': True,
        'snippet_to_match_end': True,
        'remedy': (
            '任务号、会话短号与批号都不写进仓，写那件事本身：任务号（「任务编号 <号>」「<号> 号任务」同样算）换成交给服务的那条命令（不带号）、结局行的 type= 与 exit= 那一段（不抄 task= 与 log=）'
            '或日志里那一行的内容（只抄那一行的字，不抄日志路径 task-<十六进制>-<号>）；'
            '会话短号（「会话 <短号>」、mainagent(<短号>)、agent(<短号>) 同样算）与「<做什么>那一路的会话」「那一路的 agent」「<做什么>那一路回信 / 认领 / 转达」这类描述换成按出场次序的叫法：'
            '别的会话写 mainagent(A)、mainagent(B)，子 agent 写 subagent(A)、subagent(B)（括号里只放大写字母），'
            '同一份文字里各按出场次序编字母，写清它改了哪个文件、做了什么；批号（自己会话的也一样，records/ 下中文数字的「第几批」「批三」「批次为三」同样算）换成那一批做了什么、改了哪几个文件；',
            '照写的只有三处：research/prompts/ 下判决（*-main-verification.md）里行首「A/B 读数：任务 <号>」那一行（值里至多两个任务号，两个写在同一行；引用块里的不算，列表里的「- >」也是引用块）、'
            'records/ 下调度记录里以「长跑拆分：」起头的那一行（这两处照写任务号），与 records/ 下表格里表头是「批次」的那一列（依赖表批次那一列照写批号，别的列、没有表头分隔行的「|」行照判）；',
            '反引号与围栏代码块只放过批号与会话短号形状的代码字面，任务号在里面照判（task-server/task 加子命令加号的、日志路径里的也算）；'
            '命中的其实不是任务号（计数、状态数）的换说法：「同」换成「也是」，或数后面带上量词与单位；'
            '命中的其实不是批号或会话短号（步号、参数这类代码字面）的放进反引号，或改 .claude/gate.d/lib-forbidden-notations.py 里这一形态的上下文规则；'
            '整份是被测输入的，登记进它的 excluded_prefixes 并写明理由。',
        ),
    },
)

FORMS_BY_KEY = {form['key']: form for form in FORMS}
# 弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK 里这几项把一条判据行换成宽松写法：开关项 → (判据行键, 换上的正则)。
# 同一条判据行点了几项的，取这张表里排在后面的那一项。
LOOSENED_PATTERNS = {
    'compact-timestamp-loose': ('iso-compact-timestamp', COMPACT_TIMESTAMP_LOOSE_PATTERN),
    'compact-hyphenated-date-loose': ('iso-compact-timestamp', COMPACT_HYPHENATED_DATE_LOOSE_PATTERN),
    'batch-code-labeled-three-digits': ('batch-code-labeled', BATCH_CODE_LABELED_THREE_DIGITS_PATTERN),
    'measure-compounds-counted': ('batch-numbers', BATCH_NUMBER_COMPOUNDS_COUNTED_PATTERN),
    'session-descriptions-loose': ('session-descriptions', SESSION_DESCRIPTION_LOOSE_PATTERN),
    'session-description-actions-unjudged': ('session-descriptions', SESSION_DESCRIPTION_WITHOUT_ACTIONS_PATTERN),
    'agent-parentheses-unjudged': ('session-short-codes', SESSION_SHORT_CODE_WITHOUT_AGENT_PARENTHESES_PATTERN),
    'session-word-code-unjudged': ('session-short-codes', SESSION_SHORT_CODE_WITHOUT_SESSION_WORD_CODE_PATTERN),
    'chinese-numeral-batch-unjudged': ('batch-numbers', BATCH_NUMBER_WITHOUT_CHINESE_NUMERALS_PATTERN),
    'task-number-variants-unjudged': ('task-numbers', TASK_NUMBER_WITHOUT_VARIANTS_PATTERN),
}


def environment_keys(variable):
    return {key.strip() for key in os.environ.get(variable, '').split(',') if key.strip()}


def is_excluded(form_key, relative_path):
    """这份文件（相对仓根）在不在这一形态的排除前缀下；弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS 点名这一形态时一律不排除。"""
    if form_key in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS'):
        return False
    return any(relative_path.startswith(prefix) for prefix, _ in FORMS_BY_KEY[form_key]['excluded_prefixes'])


def allowed_by_marker(form_key, line, relative_path):
    """这一行带这一形态的放行标记（后缀在 allow_suffixes 里、标记后面写了至少 4 个字的理由）；LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW 点名这一形态时一律不认。"""
    form = FORMS_BY_KEY[form_key]
    if not form['allow_suffixes'] or not relative_path.endswith(form['allow_suffixes']):
        return False
    if form_key in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW'):
        return False
    return re.search(rf'(?:#|//)\s*{re.escape(form_key)}:allow[ \t]+\S.{{3,}}', line) is not None


def judged_patterns(form_key):
    """这一形态此刻要判的判据行：[(序号, 键, 名字, 正则, 上下文规则或 None)]；整形态被弄坏开关关掉时交 None。"""
    skipped = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK')
    if form_key in skipped:
        return None
    context_off = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT')
    replaced = {pattern_key: pattern for switch, (pattern_key, pattern) in LOOSENED_PATTERNS.items() if switch in skipped}
    return [(index, key, name, replaced.get(key, pattern),
             None if key in context_off or form_key in context_off else not_violation)
            for index, (key, name, pattern, not_violation) in enumerate(FORMS_BY_KEY[form_key]['patterns']) if key not in skipped]


def outside_judged_suffixes(form_key, relative_path):
    """这份文件的后缀不在这一形态的 judged_suffixes 里（不分大小写）：不判。judged_suffixes 空着、没给 relative_path 的照判；
    弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=judged-suffixes-ignored 时一律照判。"""
    suffixes = FORMS_BY_KEY[form_key]['judged_suffixes']
    if not suffixes or not relative_path or 'judged-suffixes-ignored' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return False
    return not relative_path.lower().endswith(suffixes)


def line_hits(form_key, line, relative_path, patterns, table_header=None):
    """一行里这一形态的违例，按位置排、互不重叠：[(列, 判据行名字, 命中的串)]。同一位置几行都中，取排在前面的那一行。
    后缀不在 judged_suffixes 里的文件、带放行标记的行交空表。table_header 是这一行所在那张表的表头（line_contexts 交的），交给上下文规则。"""
    if outside_judged_suffixes(form_key, relative_path) or allowed_by_marker(form_key, line, relative_path):
        return []
    candidates = [(candidate.start(), index, candidate.end(), name, candidate.group(0))
                  for index, key, name, pattern, not_violation in patterns for candidate in pattern.finditer(line)
                  if not_violation is None or not not_violation(line, candidate, relative_path or '', table_header)]
    hits, covered_until = [], -1
    for start, _, end, name, matched in sorted(candidates):
        if start >= covered_until:
            hits.append((start, name, matched))
            covered_until = max(end, start + 1)
    return hits


def first_hit(form_key, text, relative_path=''):
    """这一形态在 text 里最靠前的一处：(位置, 判据行名字, 命中的串)；没有返回 None。
    relative_path 是 text 所在文件相对仓根的路径，用来认放行标记与按路径的上下文规则；不给就不认放行标记。围栏与表头按 text 整段认（line_contexts）。"""
    patterns = judged_patterns(form_key)
    if patterns is None:
        return None
    offset = 0
    lines = text.split('\n')
    fenced, headers = line_contexts(form_key, lines)
    for index, line in enumerate(lines):
        hits = line_hits(form_key, line, relative_path, patterns_for_line(form_key, patterns, index in fenced), headers[index])
        if hits:
            position, name, matched = hits[0]
            return offset + position, name, matched
        offset += len(line) + 1
    return None


_CANDIDATE_SEARCHES = {}


def candidate_search(patterns):
    """一形态此刻要判的判据行合成一个先筛：返回 f(行) → 这一行里有没有哪一条判据行的候选（上下文规则与放行标记之前）；没有候选的行 line_hits 一定交空表。
    判据行的正则各包进 (?:…) 用 | 连成一条：一条正则在某个位置上能中，当且仅当连成的那一条在这个位置上能中，所以 search 交 None 当且仅当每一条各自在这一行里都没有命中。
    正则带着 re.UNICODE 之外的标志（连起来标志会串）的，退回逐条 search。弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=candidate-prefilter-rejects-all：
    先筛一律说没有候选，有违例的行也不再交给 line_hits。"""
    if 'candidate-prefilter-rejects-all' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK'):
        return lambda line: None
    regexes = tuple(pattern for _, _, _, pattern, _ in patterns)
    if regexes not in _CANDIDATE_SEARCHES:
        if any(regex.flags != re.UNICODE for regex in regexes):
            searches = tuple(regex.search for regex in regexes)
            _CANDIDATE_SEARCHES[regexes] = lambda line: next((found for found in (search(line) for search in searches) if found), None)
        else:
            _CANDIDATE_SEARCHES[regexes] = re.compile('|'.join(f'(?:{regex.pattern})' for regex in regexes)).search
    return _CANDIDATE_SEARCHES[regexes]


def hits_per_line(form_key, text, relative_path=''):
    """text 按 \\n 切成行，每一行里这一形态最靠前的一处：[(行号（从 1 起）, 行文, 位置, 判据行名字, 命中的串)]，没有违例的行不列。
    与对每一行各调一次 first_hit 逐项相同（first_hit 留着当参照）；差别只在先用 candidate_search 筛掉没有候选的行，形态此刻要判的判据行与后缀判断只取一次。
    skips_fenced_blocks、reads_table_headers 的形态按整段文字认围栏与表头（line_contexts），与对整段调 first_hit 一致（逐行单调 first_hit 看不见围栏与表头）。"""
    patterns = judged_patterns(form_key)
    if patterns is None or outside_judged_suffixes(form_key, relative_path):
        return []
    has_candidate = candidate_search(patterns)
    found = []
    lines = text.split('\n')
    fenced, headers = line_contexts(form_key, lines)
    for number, line in enumerate(lines, 1):
        if has_candidate(line) is None:
            continue
        hits = line_hits(form_key, line, relative_path, patterns_for_line(form_key, patterns, number - 1 in fenced), headers[number - 1])
        if hits:
            position, name, matched = hits[0]
            found.append((number, line, position, name, matched))
    return found


def every_hit(form_key, text, relative_path='', with_context=True):
    """这一形态在 text 里的每一处：[(行号（从 1 起）, 列, 判据行名字, 命中的串)]，按出现的次序。with_context 为假时逐行单判（不认围栏与表头）。"""
    patterns = judged_patterns(form_key)
    if patterns is None:
        return []
    lines = text.split('\n')
    fenced, headers = line_contexts(form_key, lines) if with_context else (set(), [None] * len(lines))
    return [(number, column, name, matched) for number, line in enumerate(lines, 1)
            for column, name, matched in line_hits(form_key, line, relative_path, patterns_for_line(form_key, patterns, number - 1 in fenced), headers[number - 1])]


def all_hits(form_key, text, relative_path=''):
    """这一形态在 text 里的每一处：[(行号（从 1 起）, 列, 判据行名字, 命中的串)]，按出现的次序；没有交空表。围栏与表头按 text 整段认。
    弄坏开关 LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only 时只交最靠前的那一处。"""
    hits = every_hit(form_key, text, relative_path)
    return hits[:1] if 'first-hit-only' in environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK') else hits


def hits_netted_by_line_text(before_hits, after_hits, before_lines, after_lines):
    """弄坏开关 violations-netted-by-line-text 换上的相抵：写后有违例的行与写前有违例的行按行文相抵（同一行文写前有几行、写后就抵几行），
    抵不掉的那几行里的每一处都交；行里别处一改，那一行原有的号就算新写进去的。"""
    remaining = {}
    for number in sorted({number for number, _, _, _ in before_hits}):
        remaining[before_lines[number - 1]] = remaining.get(before_lines[number - 1], 0) + 1
    new_lines = set()
    for number in sorted({number for number, _, _, _ in after_hits}):
        line = after_lines[number - 1]
        if remaining.get(line, 0) > 0:
            remaining[line] -= 1
        else:
            new_lines.add(number)
    return [hit for hit in after_hits if hit[0] in new_lines]


def hits_netted_across_lines(before_hits, after_hits, before_lines, after_lines, uncounted):
    """弄坏开关 violations-netted-across-lines 与 violations-netted-uncounted 换上的相抵：(判据行名字, 命中的串) 按整份文件计数，写后某一种比写前多出几处才交几处，
    在哪一行不论（同一字面的旧号挪到别的行也抵掉）；uncounted 为真时不计数，写前出现过的串写后出现几处都抵掉。
    多出来的那几处先拿行文与写前某一违例行相同的行上的去抵，再按写后的次序从前往后抵，抵不掉的交出来。"""
    remaining = {}
    for _, _, name, matched in before_hits:
        remaining[(name, matched)] = remaining.get((name, matched), 0) + 1
    if uncounted:
        remaining = dict.fromkeys(remaining, len(after_hits))
    kept_texts = {before_lines[number - 1] for number, _, _, _ in before_hits}
    netting_order = sorted(range(len(after_hits)), key=lambda index: (after_lines[after_hits[index][0] - 1] not in kept_texts, index))
    new_indices = set()
    for index in netting_order:
        key = after_hits[index][2:]
        if remaining.get(key, 0) > 0:
            remaining[key] -= 1
        else:
            new_indices.add(index)
    return [hit for index, hit in enumerate(after_hits) if index in new_indices]


# 一块改动里写前行数乘写后行数超过它的，那一块按位置从上往下配行：按相似度配要算这么多对行的相似度，写闸每一次 Write、Edit 都要等它
PAIRING_CELLS_LIMIT = 40000
# 配上一对行的底分：一块改动里去掉违例之后剩下的字一点不像的两行，配上也比不配高（同一行改得面目全非、号没动的照抵），同样像的配法里配上的对数多的高
PAIRING_BONUS = 0.01


def hits_by_line(hits):
    """[(行号（从 1 起）, 列, 判据行名字, 命中的串)] → {行下标（从 0 起）: [(列, 判据行名字, 命中的串)]}。"""
    grouped = {}
    for number, column, name, matched in hits:
        grouped.setdefault(number - 1, []).append((column, name, matched))
    return grouped


def pairing_text(line, hits_in_line):
    """配行用的字：这一行去掉这一形态的违例（命中的串）之后剩下的字母、数字与汉字；标点、括号与空白不算，
    号挪走之后原处留下的空括号与新处跟着号来的括号不让两行显得像。"""
    pieces, position = [], 0
    for column, _, matched in sorted(hits_in_line):
        if column >= position:
            pieces.append(line[position:column])
            position = column + len(matched)
    pieces.append(line[position:])
    return ''.join(character for character in ''.join(pieces) if character.isalnum())


def aligned_rows(before_texts, after_texts):
    """一块改动里写前、写后的行按配行用的字配对：只许按次序配（不交叉），配上的每一对得 PAIRING_BONUS 加两行的相似度（difflib 的 ratio），
    取总分最高的配法，同分时先配、再跳过写前那一行、最后跳过写后那一行。→ [(写前下标, 写后下标)]，下标从块头算起。"""
    rows, columns = len(before_texts), len(after_texts)
    score = [[0.0] * (columns + 1) for _ in range(rows + 1)]
    move = [[''] * (columns + 1) for _ in range(rows + 1)]
    matcher = difflib.SequenceMatcher(None, autojunk=False)
    for column in range(1, columns + 1):
        matcher.set_seq2(after_texts[column - 1])
        for row in range(1, rows + 1):
            matcher.set_seq1(before_texts[row - 1])
            score[row][column], move[row][column] = max(
                (score[row - 1][column - 1] + PAIRING_BONUS + matcher.ratio(), 'pair'),
                (score[row - 1][column], 'skip-before'),
                (score[row][column - 1], 'skip-after'), key=lambda candidate: candidate[0])
    pairs = []
    row, column = rows, columns
    while row > 0 and column > 0:
        if move[row][column] == 'pair':
            pairs.append((row - 1, column - 1))
            row, column = row - 1, column - 1
        elif move[row][column] == 'skip-before':
            row -= 1
        else:
            column -= 1
    return pairs[::-1]


def paired_before_lines(before_lines, after_lines, before_groups, after_groups, positional):
    """写后带违例的每一行配上写前的哪一行：{写后行下标: 写前行下标}，配不上的不列。before_groups、after_groups 是 hits_by_line 交的两份。
    两份按行比（difflib.SequenceMatcher，不丢垃圾行）：没变的行配原来那一行；改了的一块里写前写后都有违例行的，按 aligned_rows 配
    （positional 为真、或写前行数乘写后行数超过 PAIRING_CELLS_LIMIT 时按位置从上往下配），别的块配不配结局都一样、不配；
    剩下没配上的写后违例行，行文与一行没配上的写前违例行一字不差的配那一行（整行原样挪了位置，号没离开它的行）。
    弄坏开关 line-pairing-positional 时 positional 为真。"""
    pairs, used = {}, set()
    matcher = difflib.SequenceMatcher(None, before_lines, after_lines, autojunk=False)
    for tag, before_start, before_end, after_start, after_end in matcher.get_opcodes():
        if tag == 'equal':
            block = list(zip(range(before_start, before_end), range(after_start, after_end)))
        elif tag == 'replace' and any(index in after_groups for index in range(after_start, after_end)) \
                and any(index in before_groups for index in range(before_start, before_end)):
            if positional or (before_end - before_start) * (after_end - after_start) > PAIRING_CELLS_LIMIT:
                block = list(zip(range(before_start, before_end), range(after_start, after_end)))
            else:
                block = [(before_start + row, after_start + column) for row, column in aligned_rows(
                    [pairing_text(before_lines[index], before_groups.get(index, ())) for index in range(before_start, before_end)],
                    [pairing_text(after_lines[index], after_groups.get(index, ())) for index in range(after_start, after_end)])]
        else:
            continue
        for before_index, after_index in block:
            used.add(before_index)
            if after_index in after_groups:
                pairs[after_index] = before_index
    unpaired_before = {}
    for index in sorted(before_groups):
        if index not in used:
            unpaired_before.setdefault(before_lines[index], []).append(index)
    for index in sorted(after_groups):
        if index not in pairs and unpaired_before.get(after_lines[index]):
            pairs[index] = unpaired_before[after_lines[index]].pop(0)
    return pairs


def hits_netted_by_paired_lines(before_hits, after_hits, before_lines, after_lines, positional):
    """平常的相抵：写后每一行上的违例只拿它配上的写前那一行（paired_before_lines）上的同一种（判据行名字加命中的串）去抵，按个数抵，
    一行里同一种多出来的报后面那几处；配不上写前任何一行的写后行，上面的违例全交。"""
    before_groups, after_groups = hits_by_line(before_hits), hits_by_line(after_hits)
    remaining = {}
    for after_index, before_index in paired_before_lines(before_lines, after_lines, before_groups, after_groups, positional).items():
        for _, name, matched in before_groups.get(before_index, ()):
            remaining[(after_index, name, matched)] = remaining.get((after_index, name, matched), 0) + 1
    found = []
    for hit in after_hits:
        key = (hit[0] - 1, hit[2], hit[3])
        if remaining.get(key, 0) > 0:
            remaining[key] -= 1
        else:
            found.append(hit)
    return found


def new_violation_hits(form_key, before_text, after_text, relative_path=''):
    """一次写进去的违例：写前、写后两份整文各判一遍（围栏与表头各按自己那一份整文认），[(写后的行号（从 1 起）, 列, 判据行名字, 命中的串)]。
    相抵的单位是违例本身（判据行名字, 命中的串），按行抵（hits_netted_by_paired_lines）：写后每一行配上写前的一行，只拿配上的那一行上的同一种去抵，抵不掉的交出来；
    文件里已有的旧号不挡这一次写，只改了那一行别处的照写；同一行新加的号、整份多出来的同一个号、同一字面的旧号离开写前它所在的那一行照报；
    这一次写让后面已有的行出了围栏、离开了表头是「批次」的那一列，写前那一处不是违例、抵不了，照报。
    弄坏开关（LIB_FORBIDDEN_NOTATIONS_BREAK，各自弄坏了什么写在文件头）：violations-not-netted、after-lines-judged-alone、before-context-ignored、
    violations-netted-by-line-text、violations-netted-by-line-number、violations-netted-uncounted、violations-netted-across-lines、line-pairing-positional。"""
    broken = environment_keys('LIB_FORBIDDEN_NOTATIONS_BREAK')
    with_context = 'after-lines-judged-alone' not in broken
    after_hits = every_hit(form_key, after_text, relative_path, with_context)
    if not after_hits or 'violations-not-netted' in broken:
        return after_hits
    before_hits = every_hit(form_key, before_text, relative_path, with_context and 'before-context-ignored' not in broken)
    if not before_hits:
        return after_hits
    before_lines, after_lines = before_text.split('\n'), after_text.split('\n')
    if 'violations-netted-by-line-text' in broken:
        return hits_netted_by_line_text(before_hits, after_hits, before_lines, after_lines)
    if 'violations-netted-by-line-number' in broken:
        numbers_with_hits = {number for number, _, _, _ in before_hits}
        return [hit for hit in after_hits if hit[0] not in numbers_with_hits]
    if 'violations-netted-across-lines' in broken or 'violations-netted-uncounted' in broken:
        return hits_netted_across_lines(before_hits, after_hits, before_lines, after_lines, 'violations-netted-uncounted' in broken)
    return hits_netted_by_paired_lines(before_hits, after_hits, before_lines, after_lines, 'line-pairing-positional' in broken)
