#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Write、Edit、MultiEdit：.claude/settings.json 里注册在 matcher "Write|Edit|MultiEdit" 上）：写文件之前的九道判定（一至五、七至九写在下面，六是语法检查、写在 --selftest 那一段），合在一个 hook 里；拒绝的同时把它记进检出记录，交主 agent 看。
#
# 一、整份覆盖未跟踪文件（只管 Write）：仓里已存在、又没进 git 的文件，拒绝用 Write 整份覆盖。
#    为什么：几个会话同时在一个仓里干活。2026-09-11 一个会话用 Write 新建 `research/prompts/e137-preregistration.md`，
#    另一个会话几分钟前刚写了同名的未提交文件，Write 回报「updated」就把它盖了——没进过 git，事后只能从对话记录里重放复原。
#    放行：目标不存在；目标已被 git 跟踪（盖了还能从 git 找回）；目标在仓外（scratchpad 之类）。
# 二、项目 subagent 的写范围（Write、Edit、MultiEdit）：项目定义的 subagent 写文件时，目标必须落在它登记的写范围里。
#    为什么：执行类 agent 越界写，不该等人翻 diff 才发现（records/2026-09-16-subagent拆分提案.md 第五节，用户 2026-09-17 定走项目 settings hook）。
#    输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（`.claude/agents/<名>.md` 不在：内置类型、插件带来的、定义已删的旧实例）只许写工作根
#    （BUILTIN_AGENT_SCOPE，展开同表里的 $SINGLEFS_WORK_ROOT/** 那一行：工作根与过渡期旧根），仓里与别处一律拒（用户 2026-10-26 定「只许写草稿目录」；
#    records/2026-10-23-subagent拆分提案.md 第十七节 ①、第十九节 ④）；有项目定义而表里没有它的写范围，拒绝；
#    目标的落点命中它的任一模式才放行：落点规整 .. 并逐段解开符号链接，目标不存在时解到最深的已存在那一层再接上剩下的段（写范围里建一个指向范围外的链接、
#    经它写进去的照拒；与 bash-command-detector.sh ⑫ 同口径，认法在 lib_write_scope.py 的 landing_forms / landings_out_of_scope）。
#    覆盖、先编后换、语法、钟点与会话号、写者锁几道判的也是这个落点（landing_target）：盖掉的、改到的、落进哪个草稿目录的都是解开链接之后那一份。表在同目录的 agent-write-scope.tsv，读表与判路径落没落进范围的函数在同目录 lib_write_scope.py。这道闸拦不住 Bash 里的写：Bash 里往本仓写、落在表里它那几行之外的，由 bash-command-detector.sh 的 ⑫ 照同一张表、调同一份库判（只判表里有行的 agent）；定义的「没做什么」照写。
#    表里以 $SINGLEFS_WORK_ROOT/ 起头的模式由同目录的 lib_work_root.py 展开成工作根（默认 ~/.local/share/singlefs/work）与过渡期旧根 /tmp/claude-<uid> 各一条，
#    根后面的 / 不省（同名前缀的兄弟目录不放行）；旧根那一条在跑的子 agent 与别的会话的子 agent 都交回之后由主 agent 撤（撤法在那份库的文件头）。
#    读不到那份库时，这样的模式按拒判，拒绝信息点名那份库。
# 三、撇号类角标（Write、Edit、MultiEdit）：Write 的 content、Edit 的 new_string、MultiEdit 每一处的 new_string 里
#    出现撇号类字符之一就拒绝；只在 old_string 里出现（删掉它的改动）放行，ASCII 单引号不判。主 agent 与所有子 agent 一样判，目标在不在仓里都判。
#    为什么：门禁 doc-text 只在收尾跑整轮门禁时判，2026-09-24 实验设计员照样把这类名字写进三份重跑登记、执行员又抄进源码。
#    字符集与 doc-text 同一份：../gate.d/lib-forbidden-notations.py 的 PRIME_MARKS，不在这里另抄；读不到就拒绝（不静默放行）。
# 四、先编后换（Write、Edit、MultiEdit）：experiment-runner（名单是 COMPILE_FIRST_AGENTS）写主工作区 crates/ 下的 .rs（改已有的、新建的都算）一律拒绝，
#    出路是在草稿目录的副本里改、经 scripts/hosts/compile-then-swap.py 编过再整份换进来。排在写范围那一道之前判，拒绝信息给的是这条出路。
#    implementation-writer、主 agent、别的 agent 不判；草稿目录里的副本与 crates/ 下不是 .rs 的（crates/mutations.tsv）不判。
#    为什么：执行员直接在主工作区改入库装置，改到一半编不过，别的会话带 --all-targets 的编译一起卡住
#    （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 51 行）。Bash 里的同一类写法归 bash-command-detector.sh 的 ⑨。
# 五、描述里的钟点、时区词与时间戳（Write、Edit、MultiEdit）：目标在仓里、不在 lib-forbidden-notations.py 里钟点那一形态（clock-times）的排除前缀下，
#    写进去的内容命中那一形态的判据行之一就拒绝；仓外（草稿目录、报告目录）不判。主 agent 与所有子 agent 一样判。
#    判据与门禁 doc-text 同一份：../gate.d/lib-forbidden-notations.py，不在这里另抄；读不到就拒绝。Bash 里的写（cp、重定向）归 doc-text 在收尾时判。
#    形态有几族：时:分（日期后、时区词旁、x 通配、Z 后缀、不挨日期时区的时:分与区间）、ISO 时间戳、ISO 基本格式的紧凑时间戳（日期连写或带连字符、T、连写的钟点，
#    运行目录名与日志文件名里的写法、路径里的也判；与门禁 doc-text 的 clock-times 格同一条判据行 iso-compact-timestamp，这里没有另抄）、光秃的时区词，
#    与中文的「N 点 / N 时」和时段词（挨着日期或时区词的、时段词后面跟钟点的、不挨日期时区却带「前后 / 左右 / 许 / 半 / 钟」的）；
#    长得像钟点的冒号对靠库里的上下文规则放过，.sh、.py、.rs 里带「clock-times:allow <理由>」的那一行放过（目标路径交给库判后缀）；
#    逐个形态的判据与放行写在那份库的文档串里。
# 七、会话号（Write、Edit、MultiEdit）：ListAgents 列的会话名（目录名后面跟「-」与十六进制）写进仓里就拒绝，排除前缀、放行标记与仓外不判同五；
#    判据是同一份库里的 session-names 那一形态，与门禁 doc-text 的 session-names 格同一份。改过标题的会话认不出。
# 八、old_string 以空白收尾（Edit、MultiEdit）：目标不是 .md / .mdx（后缀不分大小写）时，Edit 的 old_string、MultiEdit 每一处的 old_string
#    以空格或制表符收尾就拒绝；主 agent 与所有子 agent 一样判，目标在不在仓里都判。.md / .mdx 目标与 Write 不判。
#    为什么：Edit 在 .md / .mdx 以外的目标上先去掉 new_string 每一行行尾的空格与制表符，钩子拿到的已是去过的 new_string，old_string 原样。
#    old_string 收尾的空白在文件里照样命中、被换掉，new_string 里对应的那一个却没了，紧跟着的字就粘上来。
#    2026-10-01 把钩子收到的输入落盘核过：.txt 目标 old_string、new_string 都以「declared = 」收尾，
#    钩子收到的 old_string 带着那个空格、new_string 没有，落盘成「declared =foo」；门禁注释精简那一批因此把
#    doc-decisions.sh 的 `python3 - "$DEC"` 改成 `python3 -"$DEC"`（records/2026-10-01-门禁注释精简.md「第 3 批」件一）。
#    看 old_string 而不看 new_string：new_string 收尾的空白到不了钩子，判它一次都拦不到；old_string 收尾的空白钩子看得到，粘连只在它那里发生。
#    .md / .mdx 不判：同一次核查里 .md、.mdx、.MD 目标的 new_string 行尾空白原样到了钩子、原样落盘，在那里以空白收尾的框选写得对。
#    后缀只认这两个：2026-10-03 拿 Edit 在草稿目录里各改一份 .md、.MD、.markdown、.mdown、.mkd、.txt（new_string 头一行以两个空格收尾），
#    落盘只有 .md、.MD 留着那两个空格，.markdown、.mdown、.mkd 与 .txt 一样去掉了，所以它们照判、不并进 TRAILING_BLANK_EXEMPT_SUFFIXES。
#    不判 old_string 中间行的行尾空白：new_string 那一行去掉的空白后面是换行，不会把两个词粘起来。
#    出路：把框选范围收到不以空白结尾（多框到下一个非空白字符，或止于行尾）；确实要删行尾空白的，用 Bash 跑 scripts/kb/replace-once.py。
#
# 九、草稿目录写者锁（Write、Edit、MultiEdit，只判带 agent_id 的子 agent 调用）：目标在草稿根（工作根或过渡期旧根）底下、它所在的目录或往上的某一层有 .writer-lock，
#    而这个子 agent 不是持锁的那一个就拒绝。主 agent（输入里没有 agent_id）、没有锁的目录、草稿根之外的目标不判；锁文件在而判不了（读不了、不是 JSON 对象、
#    缺认持锁者要用的字段，lib_writer_lock.py 的 lock_state）的，子 agent 的写一律拒，拒绝说明点名那份锁与出路（主 agent 核过之后删掉那把锁再派，派发闸重建）；
#    认领文件在而判不了（.writer-claim-<nonce> 读不了、不是 JSON 对象、缺 agent_id，claim_state）的同一口径：子 agent 的写拒，点名那份认领文件与出路（主 agent 核过之后删掉它）；
#    判法读不到库、自己出错的照常放行，stderr 写一句并记一条检出（结局只记）。
#    锁由派发闸（runner-dispatch-guard.sh 的 ⑯）在派发那一刻排他建在提示里「草稿目录：」点名的目录里；持锁者是第一个被认领的子 agent：派发回执（主会话记录）里的提示 sha256
#    对得上锁、回执时刻不早于锁的派发时刻的才认领；回执还没落盘（或记录读不了）的这一次写照「放行但必记检出」放行、不认领（stderr 一行、检出一条，写明「回执还没落盘，没认领」），
#    回执落盘之后照旧核回执再认领，先写的非持锁者拿不到锁；认领之后只有它（含 SendMessage 续做）能写，后来的与旧进程的写被拒。
#    嵌套的草稿目录（目标往上经过几处有锁的目录）：只认最近一层的锁——父目录的持锁者 A 写子目录而子目录另派给 E 持锁时，A 写 E 的子目录被拒；E 写父目录，也被父目录的锁拦
#    （lib_writer_lock.py 文件头「嵌套的草稿目录」）。
#    回执对不上而最近一层还没有认领时，拒绝说明去派发会话的记录里找提示对得上这把锁的回执，点名持锁者的 agent_id；找不到写锁登记的提示 sha256 前 12 位、派发时刻与会话。
#    为什么：客户端重启后旧进程又活了几分钟，旧子 agent 与重派的子 agent 写同一个草稿目录，交替写同一份副本、互删副本（.claude/kb/milestone/03-third-txn.md 待优化表「草稿目录写者锁」一条）。
#    判法、锁与认领文件的格式、弄坏开关都在同目录 lib_writer_lock.py；读不到那份库时这一道不判，stderr 写一句并记一条检出（结局只记）。拦不住 Bash 里的写（这道闸只挂 Write / Edit / MultiEdit）。
# 九道判定合在这一个 hook 里：一至五、七至九各占文件头一段，六写在 --selftest 那一行的说明里。
# 九道都判完再报：不成立的几道一次列全，每道带自己的出路；钟点与会话号逐处列出（行号与总数）。只有一处例外：
# 先编后换那一道拒了，写范围那一道不再报（目标是同一个，先编后换的出路已经写了范围外的交回主 agent）。
# 它们拒的是一次写，不停任务和脚本；拒绝时退出码 2、stderr 交给做这次写的模型，同时往检出记录
# （默认工作根下的 agent-hook-detections.jsonl，lib_work_root.py 的 default_detections_path；环境变量 AGENT_HOOK_DETECTIONS 可改）追加一行，
# 主 agent 的看门狗（scripts/agents/agent-watch.py watch）读到就叫醒主 agent，由主 agent 判断怎么处理。
# 判不了的照「放行但必记检出」：输入不是合法的 JSON 对象、写者锁那一道读不到库或自己出错或派发回执还没落盘、bash -n 起不了或超时、第六道 Edit 的目标在而读不了、判定自己抛异常、落点解不开（写范围那一道照拒，别的几道按字面路径判），
# 照常退 0，stderr 写一行可定位的诊断（钩子名、哪一处、输入的前 200 字节），检出记录追加一行（findings 以「写闸没判全」起头，outcome 只记；
# 被拒的那一行 outcome 被拒）；输入读不出会话号的，session_id 写「读不到」。
# 这份源码里撇号类字符一律写成转义、lib-forbidden-notations.py 里用码位拼（门禁 doc-text 也扫它们）；改带转义的那几行用 Bash 里的 python，反斜杠拿 chr(92) 拼：
# 2026-09-24 写这一道时实测，Write 的 content 里写的反斜杠 u 转义落盘成了字符本身，这道闸也会拒这样的写。
#
#   write-guard.sh             # 从 stdin 读 hook 的 JSON
#   write-guard.sh --selftest  # 走一遍各道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1、WRITE_GUARD_DISABLE_COMPILE_FIRST=1、
#                              # WRITE_GUARD_DISABLE_SCOPE=1、WRITE_GUARD_DISABLE_CLOCK_TIMES=1、WRITE_GUARD_DISABLE_SESSION_NAMES=1、WRITE_GUARD_DISABLE_SYNTAX=1 或
#                              # WRITE_GUARD_BREAK=trailing-blank-ignored（第八道放行以空白收尾的 old_string）时自检必须判红；
#                              # WRITE_GUARD_BREAK=stop-at-first（decide() 判到第一道不成立就交回、后面几道不判）时「九道判完再报:覆盖、越界与钟点一次列全」一格红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only（库的 all_hits 只交第一处）时「钟点:两处都报（行号与总数）」「会话号:两处都报（行号与总数）」两格红；
#                              # 第二道里工作根那一条模式的三样各一个弄坏开关，打开时点名的几项红：
#                              #   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（工作根退回旧根，lib_work_root.py）：「范围:工作根下的报告目录」「先编后换:experiment-runner 改草稿目录里的副本放行」
#                              #     「stdin:实现员写工作根下的报告」「stdin:七次拒绝都落进检出记录」（多出一次拒绝）「范围:内置 agent 写工作根放行」
#                              #   SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown（不认旧根）：「范围:过渡期旧根下的报告目录」
#                              #   WRITE_GUARD_BREAK=scope-root-unanchored（展开时根后面不带 /）：「范围:工作根同名前缀的兄弟目录」
#                              # 没有项目定义的 agent 只许写工作根（第二道）：
#                              #   WRITE_GUARD_BREAK=builtin-agents-unscoped（没有项目定义的 agent 一概放行，改前的样子）：「范围:内置 agent 写仓里拒」「范围:内置 agent 写仓外别处拒」
#                              #     「范围:定义已删的旧实例写仓里拒」「stdin:内置 agent 写仓里拒」
#                              # 写范围表里 kb-scribe 的不变量清单两行（下一层与各层子目录）与 experiment-runner 的重跑比对对象登记一行：
#                              #   WRITE_GUARD_BREAK=scope-new-rows-dropped（自检用的表副本去掉这三行，退回加行之前的表）：「范围:kb-scribe 写不变量清单的 .md」「范围:kb-scribe 写不变量清单子目录里的 .md」「范围:experiment-runner 写重跑比对对象登记」
#                              # 紧凑时间戳（五里的 ISO 基本格式写法）有三个开关，打开时点名的几格红（格名都以「紧凑时间戳:」起头，另有「stdin:」两格）：
#                              #   WRITE_GUARD_BREAK=compact-timestamp-skipped（本钩子把命中的「ISO 紧凑时间戳」形态丢掉不报）：拒绝的几例（Write、Edit、MultiEdit、路径里、带冒号的时分、
#                              #     带连字符的日期、带小数秒与偏移）与「stdin:Write 写进紧凑时间戳拒绝（退出码 2）」「stdin:紧凑时间戳的拒绝点名形态与那一处」
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK=iso-compact-timestamp（库里那一条判据行不判）：同上那几格
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose（库里退回「8 位数字加 T 加 4 到 6 位数字」）：「8 位数字后跟 T 但不是时间戳」的放行五例
#                              #     （月份越界、钟点越界、年不是 20 开头、全是 9、前面挨着字母；钟点只有三位与小写 t 两例在这个开关下仍放行），
#                              #     以及「紧凑时间戳:8 位日期、T、带冒号的时分」「紧凑时间戳:带连字符的日期加连写钟点」两例（退回的写法认不出这两种）、
#                              #     「紧凑时间戳:Write / Edit / MultiEdit 写进仓里拒绝并点名形态与那一处」三格与「stdin:紧凑时间戳的拒绝点名形态与那一处」（退回的写法报出来的串少了末尾的 Z）
#                              # 第六道：写进 .claude/hooks/、.claude/gate.d/、.claude/scripts/、scripts/ 下的 .sh 先过 bash -n（写临时文件、起 bash）、.py 先过 ast 解析（本进程里 ast.parse，不写临时文件、不起子进程），
#                              # 语法错的拒（带语法错误的钩子一上线，每一次 Bash 调用都收到它的报错）；
#                              # WRITE_GUARD_BREAK=python-syntax-subprocess（.py 也退回另起 python3 判）时「语法:写坏的 .py 在本进程里判，不起 python 子进程」一格红；
#                              # 撇号、钟点、会话号三道读的共用判据库在进程里只装一次（load_notations_library，装过的直接取）：
#                              # WRITE_GUARD_BREAK=library-reloaded-each-time（每回重装）时「库:一次写入判撇号、钟点、会话号三道，共用判据库只装一次」一格红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=<判据行键>（钟点的 date-zone-hour、date-zone-period、period-hour、hour-suffix、twelve-hour、
#                              # bare-clock、iso-timestamp、iso-compact-timestamp、zone-word，键写在 ../gate.d/lib-forbidden-notations.py）关掉一条判据行时，点名那一行的用例必须判红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=clock-times 时靠上下文放行的用例、LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=clock-times 时
#                              # 带放行标记的用例必须判红；
#                              # 第九道（写者锁）：WRITE_GUARD_DISABLE_WRITER_LOCK=1 时「写者锁:提示对不上的子 agent 不能认领」「写者锁:同一份提示但派发早于这把锁的旧进程不能认领」
#                              # 「写者锁:第二个写者被拒」「写者锁:第二个写者写子目录也被拒」「写者锁:拒绝说明点名持锁者与出路」「写者锁:认领之后同一份提示的另一次派发写被拒」
#                              # 「写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id」「写者锁:回执对不上又找不到持锁者的回执时，拒绝写锁登记的提示 sha256 前几位」
#                              # 「写者锁:父目录锁的持有者写子目录另派给别人持锁的部分被拒」「写者锁:子目录锁的持有者写父目录仍被父目录锁拦」「写者锁:不持有任何一层锁的第三者写嵌套的子目录被拦」
#                              # 「写者锁:父目录认领之后，认领者写别人持锁的子目录仍被拒」「stdin:第二个写者被拒（退出码 2）」「stdin:第二个写者的拒绝点名持锁者、锁的目录与出路」
#                              # 「stdin:回执对不上的拒绝（退出码 2）」「stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路」「stdin:父目录锁的持有者写别人持锁的子目录被拒（退出码 2）」红；
#                              # LIB_WRITER_LOCK_BREAK=<项>（一次设一个，lib_writer_lock.py 的开关）打开时点名的几格红：
#                              #   claim-not-exclusive（认领不排他，后写的抢走持锁者）：「写者锁:认领之后同一份提示的另一次派发写被拒」「写者锁:不持有任何一层锁的第三者写嵌套的子目录被拦」
#                              #   holder-ignored（不比 agent_id）：「写者锁:提示对不上的子 agent 不能认领」「写者锁:同一份提示但派发早于这把锁的旧进程不能认领」「写者锁:第二个写者被拒」
#                              #     「写者锁:第二个写者写子目录也被拒」「写者锁:拒绝说明点名持锁者与出路」「写者锁:认领之后同一份提示的另一次派发写被拒」「写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id」
#                              #     「写者锁:回执对不上又找不到持锁者的回执时，拒绝写锁登记的提示 sha256 前几位」「写者锁:父目录锁的持有者写子目录另派给别人持锁的部分被拒」
#                              #     「写者锁:子目录锁的持有者写父目录仍被父目录锁拦」「写者锁:不持有任何一层锁的第三者写嵌套的子目录被拦」「写者锁:父目录认领之后，认领者写别人持锁的子目录仍被拒」
#                              #     「stdin:第二个写者被拒（退出码 2）」「stdin:第二个写者的拒绝点名持锁者、锁的目录与出路」「stdin:回执对不上的拒绝（退出码 2）」
#                              #     「stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路」「stdin:父目录锁的持有者写别人持锁的子目录被拒（退出码 2）」
#                              #   launch-ignored（认领前不看派发回执）：「写者锁:提示对不上的子 agent 不能认领」「写者锁:持锁者第一次写放行」「写者锁:持锁者续写放行」「写者锁:持锁者写子目录放行」
#                              #     「写者锁:第二个写者被拒」「写者锁:第二个写者写子目录也被拒」「写者锁:拒绝说明点名持锁者与出路」「写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id」
#                              #     「写者锁:回执对不上又找不到持锁者的回执时，拒绝写锁登记的提示 sha256 前几位」「写者锁:父目录锁的持有者写子目录另派给别人持锁的部分被拒」
#                              #     「写者锁:子目录锁的持有者写自己的子目录放行」「写者锁:子目录锁的持有者写父目录仍被父目录锁拦」「写者锁:父目录锁的持有者写父目录放行（认领父目录）」
#                              #     「写者锁:父目录认领之后，认领者写别人持锁的子目录仍被拒」「写者锁:父目录认领之后，子目录锁的持有者写自己的子目录仍放行」
#                              #     「stdin:第二个写者被拒（退出码 2）」「stdin:持锁者放行」「stdin:第二个写者的拒绝点名持锁者、锁的目录与出路」「stdin:回执对不上的拒绝（退出码 2）」
#                              #     「stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路」「stdin:父目录锁的持有者写别人持锁的子目录被拒（退出码 2）」
#                              #   stale-launch-accepted（认领前不比回执时刻）：「写者锁:同一份提示但派发早于这把锁的旧进程不能认领」「写者锁:持锁者第一次写放行」「写者锁:持锁者续写放行」
#                              #     「写者锁:持锁者写子目录放行」「写者锁:拒绝说明点名持锁者与出路」「写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id」
#                              #     「stdin:持锁者放行」「stdin:第二个写者的拒绝点名持锁者、锁的目录与出路」「stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路」
#                              #   no-walk-up（只看目标所在那一层目录）：「写者锁:第二个写者写子目录也被拒」
#                              #   foreign-holder-unnamed（回执对不上的拒绝不去记录里找持锁者）：「写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id」
#                              #     「stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路」
#                              #   replace-temp-kept（接手换锁失败不删自己建的临时文件）：「写者锁:换锁时锁内容写不进 JSON，抛异常且不留临时文件」「写者锁:换锁时锁的位置被目录占着，抛异常且不留临时文件」
#                              #   unreadable-lock-absent（锁文件在而判不了的当没有锁，改前的样子）：「写者锁:锁文件不是 JSON 拒」「写者锁:锁缺 nonce 拒」「写者锁:锁缺 dispatched_at 拒」
#                              #     「写者锁:锁缺 prompt_sha256 拒」「写者锁:锁的位置是目录拒」「写者锁:判不了的锁的拒绝点名那份锁与出路」「stdin:坏锁拒绝（退出码 2）」「stdin:坏锁的拒绝点名那份锁与出路」
#                              #   missing-receipt-claims（派发回执读不到时照旧先到先得认领，改前的样子）：「写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全」
#                              #     「写者锁:真持锁者回执还没落盘时第一次写也放行、不认领」「写者锁:回执落盘之后持锁者写拿到锁」「写者锁:持锁者认领之后，回执一直没落盘的乙写被拒」
#                              #     「写者锁:认领之后持锁者续写放行」「stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）」
#                              #   exclusive-temp-kept（排他建锁、建认领时 JSON 写到一半抛异常留下临时文件，改前的样子）：「写者锁:排他建锁时锁内容写不进 JSON，抛异常、不留临时文件也不留锁」
#                              #     「写者锁:认领时认领内容写不进 JSON，抛异常、不留临时文件也不留认领」
#                              #   takeover-sweeps-new-claims（接手换上之后删掉所有 nonce 不同的认领与认领的临时文件，改前的样子）：
#                              #     「写者锁:两次接手交错（A 换锁、B 换锁并认领、A 清理）之后 B 的认领还在，第三个认领不成，旧认领删掉」「写者锁:接手不删别的进程正在写的认领临时文件」
#                              #   broken-claim-absent（认领文件在而判不了的当没有认领，改前的样子）：「写者锁:认领文件判不了的拒绝点名那份认领文件与出路」
#                              #     「写者锁:认领文件判不了的不另建一份、不改它」「stdin:坏认领文件拒绝并点名那份认领文件（退出码 2）」
#                              # 落点（第二道与 landing_target）：WRITE_GUARD_BREAK=landing-lexical-only（退回只按字面规整、不解符号链接）时
#                              #   「范围:范围内的链接指到范围外，经它写拒」「范围:范围内的链接指到范围外，经它写不存在的子目录拒」「范围:范围内的链接指到仓里范围外拒」
#                              #   「范围:范围内的悬空链接指到范围外拒」「覆盖:仓外的链接指到仓里未跟踪的文件，Write 拒」「先编后换:仓外的链接指到主工作区的入库装置拒」
#                              #   「stdin:范围内的链接指到范围外拒（退出码 2）」与「范围:范围外的链接指回范围内放行」（字面路径在范围外，只认字面就拒）红
#                              #   （「范围:范围内的链接指到范围内放行」在开关下照样绿）；
#                              # 判不了照常放行而记检出：WRITE_GUARD_BREAK=unparsable-input-silent（输入不是合法 JSON 对象时静默退 0，改前的样子）时
#                              #   「stdin:坏 JSON 的 stderr 有诊断」「stdin:坏 JSON 记一条检出（结局只记、会话号读不到）」「stdin:坏 JSON 里按字面找得到的会话号与类型照抄」
#                              #   「stdin:JSON 数组记一条检出」红（「stdin:坏 JSON 放行（退出码 0）」照样绿）；
#                              #   WRITE_GUARD_BREAK=unjudged-not-recorded（判不了的只打 stderr、不记检出，改前的样子）时
#                              #   「写者锁:读不到库时记一处没判全」「写者锁:自己出错时记一处没判全」「stdin:写者锁自己出错放行并记一条检出（结局只记）」
#                              #   「写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全」「写者锁:真持锁者回执还没落盘时第一次写也放行、不认领」
#                              #   「stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）」红；
#                              #   WRITE_GUARD_BREAK=syntax-unreadable-unrecorded（第六道 Edit 的目标在而读不了时静默放行，改前的样子）时「语法:改一份读不了的脚本照常放行、记一处没判全」红
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
# 写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行——2026-09-17 写范围闸实测撞到的。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timezone

try:   # 工作根与过渡期旧根的唯一定义：同目录的 lib_work_root.py
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    DEFAULT_DETECTIONS, work_roots_error = work_roots.default_detections_path(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS, work_roots_error = None, None, error

try:   # 草稿目录写者锁的判法与锁的格式：同目录的 lib_writer_lock.py（读不到时第九道不判，stderr 写一句并记检出）
    sys.path.insert(0, sys.argv[1]); import lib_writer_lock as writer_lock
    writer_lock_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    writer_lock, writer_lock_error = None, error

# 写范围表的读法与判法（读表、展开工作根、glob 转正则、路径落没落进范围）在同目录的 lib_write_scope.py，bash-command-detector.sh 判 Bash 里的写用同一份；
# 读不到它时子 agent 的写按拒判（decide_scope），拒绝信息点名那份库
try:
    sys.path.insert(0, sys.argv[1]); import lib_write_scope as write_scope
    write_scope_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    write_scope, write_scope_error = None, error

def expanded_scope_patterns(patterns):
    """写范围表里的模式：$SINGLEFS_WORK_ROOT/ 起头的展开成工作根与过渡期旧根各一条（弄坏开关 WRITE_GUARD_BREAK=scope-root-unanchored 时根后面不带 /）。"""
    return write_scope.expanded_scope_patterns(patterns, work_roots, os.environ.get("WRITE_GUARD_BREAK") != "scope-root-unanchored")

def glob_to_regex(pattern):
    return write_scope.glob_to_regex(pattern)

def load_scopes(table_path):
    return write_scope.load_scopes(table_path)

def absolute_target(hook_input, project_root):
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if not file_path:
        return ""
    return os.path.normpath(file_path if os.path.isabs(file_path) else os.path.join(project_root, file_path))

UNJUDGED = []   # 这一次调用里判不了、照常放行的几处：入口逐条已打过 stderr，收尾记一条检出（结局只记）

def note_unjudged(text):
    """判不了、照常放行的一处：stderr 写一行，记进 UNJUDGED 等入口记检出。弄坏开关 WRITE_GUARD_BREAK=unjudged-not-recorded 下只打 stderr（改前的样子）。"""
    print(f"! write-guard.sh：{text}", file=sys.stderr)
    if os.environ.get("WRITE_GUARD_BREAK") != "unjudged-not-recorded":
        UNJUDGED.append(text)

def landing_lexical_only():
    return os.environ.get("WRITE_GUARD_BREAK") == "landing-lexical-only"

def landing_target(hook_input, project_root):
    """这一次写最终落在哪：absolute_target 规整 .. 之后逐段解开符号链接（目标不存在时解到最深的已存在那一层，lib_write_scope.landing_forms）。
    取的是 landing_forms 里「先规整 .. 再解开」那一种（工具先规整路径再打开）；解不开时交字面那一份并记一处没判全；弄坏开关 WRITE_GUARD_BREAK=landing-lexical-only 下交字面那一份（改前的样子）。"""
    absolute = absolute_target(hook_input, project_root)
    if not absolute or landing_lexical_only():
        return absolute
    try:
        return os.path.realpath(absolute)
    except (OSError, ValueError) as error:
        note_unjudged(f"{absolute} 的符号链接解不开（{type(error).__name__}: {error}），覆盖、先编后换、语法、钟点与写者锁几道按字面路径判")
        return absolute

def landing_root(project_root):
    """和 landing_target 配对的仓根：也解开符号链接（仓根自己在链接底下时，落点照样算得出相对仓根的路径）。"""
    return os.path.normpath(project_root) if landing_lexical_only() else os.path.realpath(project_root)

def decide_overwrite(hook_input, project_root):
    """一、Write 整份覆盖未跟踪的已有文件。"""
    if hook_input.get("tool_name") != "Write" or os.environ.get("WRITE_GUARD_DISABLE_OVERWRITE") == "1":
        return 0, None
    target = landing_target(hook_input, project_root)
    if not target or not os.path.exists(target):
        return 0, None
    top = subprocess.run(["git", "-C", os.path.dirname(target), "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    if top.returncode != 0:
        return 0, None
    tracked = subprocess.run(["git", "-C", top.stdout.strip(), "ls-files", "--error-unmatch", "--", target], capture_output=True, text=True)
    if tracked.returncode == 0:
        return 0, None
    return 2, (f"✗ 拒绝用 Write 整份覆盖 {target}：它已存在而且没进 git，可能是别的会话还没提交的文件，盖掉就找不回来。\n"
               "→ 新建文件：换一个没被占的名字，用排他方式写（set -o noclobber; cat > 路径 <<'EOF'，或 python open(路径, 'x')）。\n"
               "→ 确认是这一轮自己的文件：先 Read 看一眼，再用 Edit 改；整份重写就先 git add 它。")

# 没有项目定义的 agent（内置类型、插件带来的、定义已删的旧实例）的写范围：只有工作根（文件头第二道；bash-command-detector.sh 的 ⑫ 用同一句模式）
BUILTIN_AGENT_SCOPE = ("$SINGLEFS_WORK_ROOT/**",)

def decide_builtin_scope(agent_type, absolute, project_root):
    """二之一：没有项目定义的 agent 只许写工作根（BUILTIN_AGENT_SCOPE 展开成工作根与过渡期旧根），仓里与别处一律拒；落点照 lib_write_scope.landings_out_of_scope 解开符号链接。"""
    if work_roots is None:
        return 2, (f"✗ 读不到工作根的定义 .claude/hooks/lib_work_root.py（{work_roots_error!r}），没有项目定义的 {agent_type} 只许写工作根，这一次按拒判：{absolute}\n"
                   "→ 怎么办：先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它；这件活交回主 agent。")
    try:
        outside = write_scope.landings_out_of_scope(expanded_scope_patterns(list(BUILTIN_AGENT_SCOPE)), absolute, project_root, landing_lexical_only())
    except (OSError, ValueError) as error:
        return 2, (f"✗ {absolute} 的符号链接解不开（{type(error).__name__}: {error}），算不出这次写落在哪，没有项目定义的 {agent_type} 的写按拒判\n"
                   "→ 怎么办：直接写链接指向的那个路径（ls -l 看它指到哪）；落在工作根之外的写进报告交回主 agent。")
    if not outside:
        return 0, None
    landed = (f"（解开符号链接之后落在 {'、'.join(outside)}）" if outside != [absolute] else "")
    return 2, (f"✗ {agent_type} 没有项目定义（.claude/agents/{agent_type}.md 不在：内置类型、插件带来的或定义已删的旧实例），只许写工作根，不含 {absolute}{landed}\n"
               f"→ 它能写的：{work_roots.work_root()}/ 下（过渡期另认旧根 {work_roots.legacy_root()}/）；仓里与别处一律不写（用户 2026-10-26 定「只许写草稿目录」）。\n"
               "→ 怎么办：草稿、报告写进派发提示给的草稿目录；要改仓里的文件，写进报告交回主 agent，由主 agent 改或派有写范围的项目 agent（一次性的活派 clerk）。")

def decide_scope(hook_input, project_root, table_path):
    """二、项目 subagent 的写范围；没有项目定义的 agent 只许写工作根（decide_builtin_scope）。"""
    agent_type = hook_input.get("agent_type")
    if not agent_type:
        return 0, None
    defined = os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md"))
    if not defined and os.environ.get("WRITE_GUARD_BREAK") == "builtin-agents-unscoped":   # 弄坏开关：没有项目定义的一概放行（改前的样子）
        return 0, None
    if os.environ.get("WRITE_GUARD_DISABLE_SCOPE") == "1":
        return 0, None
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    if write_scope is None:
        return 2, (f"✗ 读不到写范围的判法 .claude/hooks/lib_write_scope.py（{write_scope_error!r}），{agent_type} 的写按拒判：{file_path}\n"
                   "→ 怎么办：先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它（Edit / Write 这一刻都被这道闸拦着）；这件活交回主 agent。")
    if not defined:
        return decide_builtin_scope(agent_type, absolute_target(hook_input, project_root), project_root)
    patterns = load_scopes(table_path).get(agent_type, [])
    if not patterns:
        return 2, (f"✗ {agent_type} 在写范围表里没有登记，按拒绝处理：{file_path}\n"
                   f"→ 怎么办：在 .claude/hooks/agent-write-scope.tsv 按它定义的「写范围」一节补上路径模式；补之前这件活交回主 agent。")
    absolute = absolute_target(hook_input, project_root)
    try:   # 落点：规整 .. 并逐段解开符号链接，每一种认法都要在范围里（lib_write_scope.py 的 landings_out_of_scope）
        outside = write_scope.landings_out_of_scope(expanded_scope_patterns(patterns), absolute, project_root, landing_lexical_only())
    except (OSError, ValueError) as error:
        return 2, (f"✗ {absolute} 的符号链接解不开（{type(error).__name__}: {error}），算不出这次写落在哪，{agent_type} 的写按拒判\n"
                   "→ 怎么办：直接写链接指向的那个路径（ls -l 看它指到哪）；落在写范围之外的写进报告交回主 agent。")
    if not outside:
        return 0, None
    landed = (f"（解开符号链接之后落在 {'、'.join(outside)}）" if outside != [absolute] else "")
    unexpanded = (f"\n→ 读不到工作根的定义 .claude/hooks/lib_work_root.py（{work_roots_error!r}），表里 $SINGLEFS_WORK_ROOT/ 起头的模式按拒判："
                  "先 git status 看那份库是被删了还是改坏了，用 Bash 恢复它") if work_roots is None and any(p.startswith("$") for p in patterns) else ""
    return 2, (f"✗ {agent_type} 的写范围不含 {absolute}{landed}\n"
               f"→ 它的写范围：{'、'.join(patterns)}（.claude/hooks/agent-write-scope.tsv；$SINGLEFS_WORK_ROOT 是工作根 ~/.local/share/singlefs/work，过渡期另认旧根 /tmp/claude-<uid>）。\n"
               f"→ 怎么办：范围外的改动不自己做，写进报告交回主 agent，由主 agent 改或派该写的 agent。{unexpanded}")

def decide_writer_lock(hook_input, project_root):
    """九、子 agent 写草稿根底下的文件，那个目录（或往上的某一层）有写者锁、而它不是持锁的那一个。判法与锁的格式在 lib_writer_lock.py。"""
    agent_id = hook_input.get("agent_id")
    if not agent_id or os.environ.get("WRITE_GUARD_DISABLE_WRITER_LOCK") == "1":
        return 0, None
    if writer_lock is None or work_roots is None:
        note_unjudged(f"写者锁这一道读不到 lib_writer_lock.py 或 lib_work_root.py（{(writer_lock_error or work_roots_error)!r}），这一次没判、照常放行")
        return 0, None
    target = landing_target(hook_input, project_root)
    if not target:
        return 0, None
    transcript_path = hook_input.get("transcript_path") or ""
    try:
        scratch = writer_lock.find_lock_directory(target, work_roots.scratch_roots(), any_entry=True)   # 嵌套的草稿目录只认最近一层的锁（lib_writer_lock.py 文件头「嵌套的草稿目录」）
        if not scratch:
            return 0, None
        state, lock = writer_lock.lock_state(scratch)
        if state == "absent":
            return 0, None
        if state == "unreadable":
            lock_file = writer_lock.lock_path(scratch)
            return 2, (f"✗ {target} 在草稿目录 {scratch} 里，这个目录的写者锁 {lock_file} 在、却判不了谁持锁（{lock}）：锁判不了的目录，子 agent 一律不写。\n"
                       "→ 怎么办：停下对这个目录的写，已经写的与没写完的在报告里交回主 agent，交回里点名这把锁的路径。\n"
                       f"→ 主 agent：核过这个目录现在归谁（.owner、在跑的子 agent）之后，删掉这把锁（rm {lock_file}；锁的位置是目录的 rm -r）再重派，"
                       "派发闸重建（提示里「草稿目录：」点名它）；锁判不了时派发闸不认「写者锁已判」接手，不删就派会被它拒。")
        verdict, holder = writer_lock.writer_verdict(scratch, lock, agent_id, transcript_path)
        if verdict == "holder":
            return 0, None
        if verdict == "unclaimed":   # 回执还没落盘：照「放行但必记检出」放行、不认领（lib_writer_lock.py 文件头「谁是持锁者」2）
            note_unjudged(f"写者锁这一道：{agent_id} 写 {target}，派发回执还没落盘，没认领 {scratch} 的写者锁"
                          f"（主会话记录 {transcript_path or '（输入里没有）'} 里还找不到它的回执，或记录读不了），这一次照常放行；回执落盘之后照旧核回执再认领")
            return 0, None
        if verdict == "broken-claim":
            claim_file = writer_lock.claim_path(scratch, lock)
            return 2, (f"✗ {target} 在草稿目录 {scratch} 里，这个目录的写者锁的认领文件 {claim_file} 在、却判不了谁持锁（{holder}）：认领判不了的目录，子 agent 一律不写。\n"
                       "→ 怎么办：停下对这个目录的写，已经写的与没写完的在报告里交回主 agent，交回里点名这份认领文件的路径。\n"
                       f"→ 主 agent：核过这个目录现在归谁（.owner、在跑的子 agent）之后，删掉这份认领文件（rm {claim_file}；是目录的 rm -r），"
                       "持锁者下一次写照派发回执重新认领；归属核不清的，换一个草稿目录重派。")
        unknown = None if holder else writer_lock.describe_unknown_holder(lock, agent_id, transcript_path)
    except Exception as error:   # 挂在每一次写上的闸，坏了不能把所有写都挡住
        note_unjudged(f"写者锁这一道自己出错，这一次放行：{type(error).__name__}: {error}")
        return 0, None
    owner = f"子 agent {holder}" if holder else f"{unknown}（你的派发回执不是这把锁登记的那一份提示，或早于这把锁）"
    return 2, (f"✗ {target} 在草稿目录 {scratch} 里，这个目录的写者锁（派发 {lock.get('subagent_type') or '?'}，派发于 {str(lock.get('dispatched_date'))[:10]}）由{owner}持有，"
               f"你（{agent_id}）不是：同一个草稿目录同一时刻只许持锁的那一个子 agent 写。\n"
               "→ 怎么办：停下对这个目录的写，已经写的与没写完的在报告里交回主 agent；主 agent 给你换一个没人持锁的草稿目录重派，"
               "或在派发提示里单起一行「写者锁已判：<理由>」接手这个目录（接手之后原持锁者写这个目录会被拒）。\n"
               "→ 规矩：锁的格式与谁算持锁者见 .claude/hooks/lib_writer_lock.py 文件头；Bash 里的写这道闸看不见，同样不许写进别人持锁的目录。")

# 四、先编后换：这几个项目 subagent 不直接写主工作区 crates/ 下的 .rs，出路是 scripts/hosts/compile-then-swap.py。
# Bash 里的同一类写法由 bash-command-detector.sh 的 ⑨ 判，agent 名单与路径判据两边各写一份、改的时候一起改。
COMPILE_FIRST_AGENTS = ("experiment-runner",)
COMPILE_THEN_SWAP_SCRIPT = "scripts/hosts/compile-then-swap.py"

def decide_compile_first(hook_input, project_root):
    """四、登记在 COMPILE_FIRST_AGENTS 里的 agent 写主工作区 crates/ 下的 .rs。"""
    agent_type = hook_input.get("agent_type")
    if agent_type not in COMPILE_FIRST_AGENTS or os.environ.get("WRITE_GUARD_DISABLE_COMPILE_FIRST") == "1":
        return 0, None
    absolute = landing_target(hook_input, project_root)
    root = landing_root(project_root)
    if not absolute or not absolute.endswith(".rs") or not absolute.startswith(os.path.join(root, "crates") + os.sep):
        return 0, None
    relative = os.path.relpath(absolute, root)
    return 2, (f"✗ {agent_type} 不直接写主工作区的 {relative}：入库装置先在草稿目录的副本里改，编过再整份换进主工作区，"
               "主工作区里任何时候都只放编得过的版本。\n"
               f"→ 怎么办：cp {relative} <草稿目录>/ 拷一份（新建的装置直接在草稿目录里建），在草稿目录那一份上用 Edit / Write 改；"
               f"改完跑 bash scripts/hosts/capped.sh <线程上限> python3 {COMPILE_THEN_SWAP_SCRIPT} <草稿目录>/{os.path.basename(relative)} "
               f"{relative} --scratch <草稿目录>，它编过才整份换进来，编不过一个字节不写。\n"
               "→ 目标不是 crates/<crate>/src/bin/ 下的装置（crate 的库、测试）的，不归你改，写进报告交回主 agent。")

def notations_library_path(hook_dir):
    return os.path.join(os.path.dirname(hook_dir), "gate.d", "lib-forbidden-notations.py")

NOTATIONS_LIBRARIES = {}      # 库路径 → 装好的模块：一个进程里同一份库只装一次（一次写入判撇号、钟点、会话号三道，都从这里取）
NOTATIONS_LIBRARY_LOADS = [0]  # 真装了几次，自证数它

def load_notations_library(library_path):
    """门禁 doc-text 与本钩子共用的判据库 lib-forbidden-notations.py：装过的直接交回，没装过的装一次并记下；读不到、语法错抛异常（失败不记，下一回再试）。
    弄坏开关 WRITE_GUARD_BREAK=library-reloaded-each-time 每回重装。"""
    library = NOTATIONS_LIBRARIES.get(library_path)
    if library is not None and os.environ.get("WRITE_GUARD_BREAK") != "library-reloaded-each-time":
        return library
    spec = importlib.util.spec_from_file_location("lib_forbidden_notations", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    NOTATIONS_LIBRARY_LOADS[0] += 1
    NOTATIONS_LIBRARIES[library_path] = library
    return library

def load_prime_marks(library_path):
    """撇号类字符集，与门禁 doc-text 同一份；读不到或读出来是空的就抛异常，由调用方按拒绝处理。"""
    library = load_notations_library(library_path)
    prime_marks = library.PRIME_MARKS
    if not isinstance(prime_marks, str) or not prime_marks:
        raise ValueError(f"PRIME_MARKS 是 {prime_marks!r}，不是非空字符串")
    return prime_marks

def written_texts(hook_input):
    """这一次写要落进文件的文字：Write 的 content、Edit 的 new_string、MultiEdit 每一处的 new_string。old_string 不算。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    if tool_name == "Write":
        return [str(tool_input.get("content") or "")]
    if tool_name == "Edit":
        return [str(tool_input.get("new_string") or "")]
    if tool_name == "MultiEdit":
        return [str((edit or {}).get("new_string") or "") for edit in tool_input.get("edits") or []]
    return []

def decide_prime_marks(hook_input, library_path):
    """三、写进去的内容里有撇号类角标。"""
    texts = [text for text in written_texts(hook_input) if text]
    if not texts:
        return 0, None
    try:
        prime_marks = load_prime_marks(library_path)
    except Exception as error:
        return 2, (f"✗ 读不到撇号类字符集 {library_path}（{type(error).__name__}：{error}），这次写没法判，按拒绝处理。\n"
                   "→ 怎么办：那一份是门禁 doc-text 与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
                   "用 Bash 恢复它（不在 hook 里另抄一份）；恢复之前 Write / Edit 都会被拒。")
    occurrences = sum(text.count(mark) for text in texts for mark in prime_marks)
    if not occurrences:
        return 0, None
    offending_text = next(text for text in texts if any(mark in text for mark in prime_marks))
    position, mark = min((offending_text.find(mark), mark) for mark in prime_marks if mark in offending_text)
    line_start = offending_text.rfind("\n", 0, position) + 1
    line_end = offending_text.find("\n", position)
    line = offending_text[line_start:line_end if line_end != -1 else len(offending_text)]
    column = position - line_start
    snippet = line[max(0, column - 20):column + 10].strip()
    file_path = (hook_input.get("tool_input") or {}).get("file_path") or ""
    return 2, (f"✗ 写进去的内容里有撇号类角标 {mark}（U+{ord(mark):04X}）在「…{snippet}…」"
               f"（{file_path}，这次写的内容里共 {occurrences} 处）\n"
               "→ 怎么办：给这个变体起一个新名字——在它那一族里取下一个没用过的号，或起一个短的描述性名字；"
               "做法见 .claude/rules/path-moves.md「变体起新名字，不用角标」\n"
               "→ 引用户原话或原文时也不例外：把那个字符改写成文字（写成「B2（加撇号）」这类）")

TRAILING_BLANKS = (" ", "\t")
TRAILING_BLANK_EXEMPT_SUFFIXES = (".md", ".mdx")

def decide_trailing_blank(hook_input):
    """八、目标不是 .md / .mdx 时，Edit 的 old_string、MultiEdit 每一处的 old_string 以空格或制表符收尾。"""
    if os.environ.get("WRITE_GUARD_BREAK") == "trailing-blank-ignored":
        return 0, None
    tool_name = hook_input.get("tool_name")
    if tool_name not in ("Edit", "MultiEdit"):
        return 0, None
    tool_input = hook_input.get("tool_input") or {}
    if str(tool_input.get("file_path") or "").lower().endswith(TRAILING_BLANK_EXEMPT_SUFFIXES):
        return 0, None
    edits = (tool_input.get("edits") or []) if tool_name == "MultiEdit" else [tool_input]
    texts = [str((edit or {}).get("old_string") or "") for edit in edits]
    offending = [index for index, text in enumerate(texts) if text.endswith(TRAILING_BLANKS)]
    if not offending:
        return 0, None
    text = texts[offending[0]]
    last_line = text[text.rfind("\n") + 1:]
    blank = last_line[len(last_line.rstrip(" \t")):]
    shown = last_line.rstrip(" \t")[-30:] + blank.replace(" ", "␠").replace("\t", "⇥")
    where = f"第 {offending[0] + 1} 处（共 {len(offending)} 处以空白收尾）的 " if tool_name == "MultiEdit" else ""
    file_path = tool_input.get("file_path") or ""
    return 2, (f"✗ {tool_name} {where}old_string 以空白收尾：「…{shown}」（{file_path}；␠ 是空格，⇥ 是制表符）\n"
               "  .md / .mdx 以外的目标，Edit 先去掉 new_string 每一行行尾的空格与制表符再施加：old_string 收尾的空白照样被换掉，"
               "new_string 里对应的那一个却没了，文件里紧跟着的字就粘上来（`python3 - \"$DEC\"` 落成 `python3 -\"$DEC\"`）。\n"
               "→ 怎么办：把 old_string 与 new_string 的框选范围收到不以空白结尾：两边一起往后多框到下一个非空白字符，或止于行尾（框到换行之前）。\n"
               "→ 确实要删行尾空白的：用 Bash 跑 python3 scripts/kb/replace-once.py <文件> <旧串> <新串>（旧串恰好命中一次才写）。")

LIBRARY_UNREADABLE_PREFIX = "✗ 读不到"
LISTED_HITS_LIMIT = 20

def written_text_places(hook_input, project_root):
    """written_texts 的每一段，连同「它的第 n 行在哪」的说法：[(文字, 行号 → 说法)]。Write 的行号就是文件的行号；
    Edit 的 old_string 在现文件里找得到时折算成施加之后文件里的行号，找不到写 new_string 的第几行；MultiEdit 写第几处的第几行。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    texts = written_texts(hook_input)
    if tool_name == "Edit":
        base, old_string = None, str(tool_input.get("old_string") or "")
        try:
            current = open(absolute_target(hook_input, project_root), encoding="utf-8", errors="replace").read()
            index = current.find(old_string) if old_string and not tool_input.get("replace_all") else -1
            base = current[:index].count("\n") if index >= 0 else None
        except (OSError, ValueError):
            pass
        return [(texts[0], (lambda number: f"第 {base + number} 行") if base is not None else (lambda number: f"new_string 第 {number} 行"))]
    if tool_name == "MultiEdit":
        return [(text, lambda number, place=place: f"第 {place} 处 new_string 第 {number} 行") for place, text in enumerate(texts, 1)]
    return [(text, lambda number: f"第 {number} 行") for text in texts]

def library_hits_in_written_texts(hook_input, project_root, library_path, form_key, form_name):
    """写进仓里的内容在共用库那一形态下的每一处：(拒绝信息, None, []) 读不到库；(None, None, []) 没命中或不判；
    (None, 相对路径, [(在哪, 判据行名字, 命中的串)]) 命中（库的 all_hits，按出现的次序）。"""
    places = [(text, where) for text, where in written_text_places(hook_input, project_root) if text]
    absolute = landing_target(hook_input, project_root)
    root = landing_root(project_root)
    if not places or not absolute or os.path.commonpath([absolute, root]) != root:
        return None, None, []
    try:
        library = load_notations_library(library_path)
        relative = os.path.relpath(absolute, root)
        if library.is_excluded(form_key, relative):
            return None, None, []
        found = [(where(number), name, matched) for text, where in places for number, _, name, matched in library.all_hits(form_key, text, relative)]
    except Exception as error:
        return (f"{LIBRARY_UNREADABLE_PREFIX}{form_name}判据 {library_path}（{type(error).__name__}：{error}），这次写没法判，按拒绝处理。\n"
                "→ 怎么办：那一份是门禁 doc-text 与这道 hook 共用的唯一定义，先 git status 看它是被删了还是被改坏了，"
                "用 Bash 恢复它（不在 hook 里另抄一份）；恢复之前往仓里的 Write / Edit 都会被拒。"), None, []
    return None, relative, found

def listed_hits(found, with_shape):
    """逐处一行：「  第 n 行（判据行名字）：「命中的串」」；超过 LISTED_HITS_LIMIT 处的只报另有几处。"""
    shown = [f"  {where}（{shape}）：「{matched}」" if with_shape else f"  {where}：「{matched}」" for where, shape, matched in found[:LISTED_HITS_LIMIT]]
    if len(found) > LISTED_HITS_LIMIT:
        shown.append(f"  （另有 {len(found) - LISTED_HITS_LIMIT} 处没列，改完上面这些再写一次会列出来）")
    return "\n".join(shown) + "\n"

COMPACT_TIMESTAMP_SHAPE = "ISO 紧凑时间戳"   # 库里 iso-compact-timestamp 那一条判据行的名字；只给弄坏开关 compact-timestamp-skipped 认形态用，判法在库里

def decide_clock_times(hook_input, project_root, library_path):
    """五、写进仓里的内容带描述性的钟点。"""
    if os.environ.get("WRITE_GUARD_DISABLE_CLOCK_TIMES") == "1":
        return 0, None
    unreadable, relative, found = library_hits_in_written_texts(hook_input, project_root, library_path, "clock-times", "钟点")
    if unreadable:
        return 2, unreadable
    if os.environ.get("WRITE_GUARD_BREAK") == "compact-timestamp-skipped":   # 弄坏开关：把紧凑时间戳那一形态的命中丢掉
        found = [hit for hit in found if hit[1] != COMPACT_TIMESTAMP_SHAPE]
    if not found:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容在描述里写了钟点，共 {len(found)} 处：\n" + listed_hits(found, True) +
               "→ 怎么办：时间只写到日期，把钟点（时:分、「N 点 / N 时」、上午下午这类时段词）连同挨着它的时区词、「前后」一起删掉；"
               "光秃的时区词、ISO 时间戳（含运行目录名、日志文件名里连写的紧凑写法，路径里的也判：目录名里的时间戳段删掉）与 date 的原样输出也删，只写日期。\n"
               "→ .sh、.py、.rs 里代码真要用这些字面（TZ 环境变量、解析外部文本的正则、造时间戳的自检输入），在那一行写「# clock-times:allow <理由>」（Rust 写「//」）；"
               "命中的其实不是钟点（产物字段、切片、编号对），改 .claude/gate.d/lib-forbidden-notations.py 的上下文规则\n"
               "→ 钟点确实是被测输入的整份文件（hook 自检、门禁样本），登记进 .claude/gate.d/lib-forbidden-notations.py 里钟点那一形态的 excluded_prefixes 并写明理由")

def decide_session_names(hook_input, project_root, library_path):
    """七、写进仓里的内容带会话号（ListAgents 列的会话名）。"""
    if os.environ.get("WRITE_GUARD_DISABLE_SESSION_NAMES") == "1":
        return 0, None
    unreadable, relative, found = library_hits_in_written_texts(hook_input, project_root, library_path, "session-names", "会话号")
    if unreadable:
        return 2, unreadable
    if not found:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容写了会话号，共 {len(found)} 处：\n" + listed_hits(found, False) +
               "→ 怎么办：会话号不写进仓：标签式的「（会话 <号>，<日期>）」只留日期，当主语用的写成「别的会话」「SOP 会话」这类不带号的说法，事情与日期照留；"
               "改过标题的会话（「会话「<标题>」」）同样不写标题。\n"
               "→ .sh、.py、.rs 里代码真要用这些字面（解析 ListAgents 输出的正则、造会话号的自检输入），在那一行写「# session-names:allow <理由>」（Rust 写「//」）")

SYNTAX_CHECKED_DIRECTORIES = (".claude/hooks", ".claude/gate.d", ".claude/scripts", "scripts")
PYTHON_SYNTAX_CHECK = "import ast, sys; ast.parse(open(sys.argv[1], encoding='utf-8').read(), sys.argv[1])"   # 只给弄坏开关 python-syntax-subprocess 用

def python_syntax_error(content, relative):
    """.py 的内容在本进程里 ast 解析（不写临时文件、不起子进程）：过得去交 None，过不去交错误说明（文件名写相对仓根的路径）。"""
    import ast, traceback
    try:
        ast.parse(content, filename=relative)
    except (SyntaxError, ValueError, RecursionError) as error:
        return "".join(traceback.format_exception_only(type(error), error)).strip()[-600:]
    return None

def syntax_error_by_checker(checker, content, suffix, relative):
    """内容写进临时文件、起 checker（bash -n）判：过得去交 None，过不去交错误说明；起不了、超时按放行交 None。"""
    handle = tempfile.NamedTemporaryFile("w", suffix=suffix, delete=False, encoding="utf-8")
    try:
        handle.write(content)
        handle.close()
        result = subprocess.run(checker + [handle.name], capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired) as error:
        note_unjudged(f"{' '.join(checker[:2])} 起不了或超时（{type(error).__name__}: {error}），{relative} 的语法这一次没判、照常放行")
        return None
    finally:
        handle.close()
        os.unlink(handle.name)
    if result.returncode == 0:
        return None
    return (result.stderr or result.stdout).strip().replace(handle.name, relative)[-600:]

def edited_content(hook_input, target):
    """Write 给的整份内容，或 Edit / MultiEdit 施加到现文件之后的内容；施加不了（文件不在、旧串找不到）返回 None，那一步 Edit 自己会拒。"""
    tool_name = hook_input.get("tool_name")
    tool_input = hook_input.get("tool_input") or {}
    if tool_name == "Write":
        return tool_input.get("content")
    if not os.path.isfile(target):
        return None
    try:
        text = open(target, encoding="utf-8", errors="replace").read()
    except OSError as error:   # 在而读不了：改完之后的内容算不出，语法这一道判不了，记一处没判全（弄坏开关 syntax-unreadable-unrecorded 下静默放行，改前的样子）
        if os.environ.get("WRITE_GUARD_BREAK") != "syntax-unreadable-unrecorded":
            note_unjudged(f"{target} 读不了（{type(error).__name__}: {error.strerror or error}），改完之后的内容算不出，语法这一次没判、照常放行")
        return None
    edits = tool_input.get("edits") if tool_name == "MultiEdit" else [tool_input]
    for edit in edits or []:
        old_string = edit.get("old_string") or ""
        if not old_string or old_string not in text:
            return None
        new_string = edit.get("new_string") or ""
        text = text.replace(old_string, new_string) if edit.get("replace_all") else text.replace(old_string, new_string, 1)
    return text

def decide_shell_syntax(hook_input, project_root):
    """六、钩子、门禁、项目脚本与研究脚本目录下的 .sh 与 .py：写进去的内容先过语法，过不了就拒。"""
    if os.environ.get("WRITE_GUARD_DISABLE_SYNTAX") == "1" or hook_input.get("tool_name") not in ("Write", "Edit", "MultiEdit"):
        return 0, None
    target = landing_target(hook_input, project_root)
    if not target:
        return 0, None
    relative = os.path.relpath(target, landing_root(project_root))
    if relative.startswith("..") or not any(relative.startswith(directory + "/") for directory in SYNTAX_CHECKED_DIRECTORIES):
        return 0, None
    if target.endswith(".sh"):
        checker, checker_name = ["bash", "-n"], "bash -n"
    elif target.endswith(".py"):
        checker, checker_name = None, "python3 的 ast 解析"
    else:
        return 0, None
    content = edited_content(hook_input, target)
    if content is None:
        return 0, None
    if checker is None and os.environ.get("WRITE_GUARD_BREAK") == "python-syntax-subprocess":   # 弄坏开关：.py 也退回另起 python3 的旧做法
        checker = [sys.executable, "-c", PYTHON_SYNTAX_CHECK]
    detail = python_syntax_error(content, relative) if checker is None else syntax_error_by_checker(checker, content, os.path.splitext(target)[1], relative)
    if detail is None:
        return 0, None
    return 2, (f"✗ 写进 {relative} 的内容过不了 {checker_name}：\n  {detail}\n"
               f"→ 怎么办：先在草稿目录里改到 `{checker_name}` 过，再写进来；钩子、门禁与脚本带着语法错误上线，之后每一次调用都会收到它的报错。")

def decide(hook_input, project_root, table_path, prime_marks_library, clock_times_library=None):
    """返回 (0, None, None) 放行；(2, 说明, 哪几道（「、」连）) 拒绝。九道都判完，不成立的按下面的次序一次列全；
    先编后换拒了不再判写范围，钟点那一道读不到判据时会话号那一道不再报同一句。弄坏开关 WRITE_GUARD_BREAK=stop-at-first 判到第一道不成立就交回。"""
    if clock_times_library is None:
        clock_times_library = prime_marks_library
    judges = (("整份覆盖未跟踪文件", lambda: decide_overwrite(hook_input, project_root)),
              ("绕过先编后换", lambda: decide_compile_first(hook_input, project_root)),
              ("old_string 以空白收尾", lambda: decide_trailing_blank(hook_input)),
              ("写进语法错误", lambda: decide_shell_syntax(hook_input, project_root)),
              ("越出写范围", lambda: decide_scope(hook_input, project_root, table_path)),
              ("写进撇号类角标", lambda: decide_prime_marks(hook_input, prime_marks_library)),
              ("描述里写了钟点", lambda: decide_clock_times(hook_input, project_root, clock_times_library)),
              ("写了会话号", lambda: decide_session_names(hook_input, project_root, clock_times_library)),
              ("写别的子 agent 持锁的草稿目录", lambda: decide_writer_lock(hook_input, project_root)))
    refused = []   # [(哪一道, 说明)]
    for label, judge in judges:
        already = {name for name, _ in refused}
        if label == "越出写范围" and "绕过先编后换" in already:
            continue
        if (label == "写了会话号" and "描述里写了钟点" in already
                and dict(refused)["描述里写了钟点"].startswith(LIBRARY_UNREADABLE_PREFIX)):
            continue
        code, message = judge()
        if code:
            refused.append((label, message))
            if os.environ.get("WRITE_GUARD_BREAK") == "stop-at-first":
                break
    if not refused:
        return 0, None, None
    labels = "、".join(label for label, _ in refused)
    if len(refused) == 1:
        return 2, refused[0][1], labels
    blocks = [f"── 第 {number} 道：{label} ──\n{message}" for number, (label, message) in enumerate(refused, 1)]
    return 2, (f"✗ 这一次写有 {len(refused)} 道判定不成立（{labels}）：九道都判完了，下面一次列全，逐道照各自的「→」改完再写。\n"
               + "\n".join(blocks)), labels

def record_detection(hook_input, finding, detections_path, outcome="被拒"):
    """检出记录追加一行：outcome「被拒」时 findings 是「写被拒：<哪几道>」，「只记」（判不了、照常放行）时 finding 是一串说明、各以「写闸没判全：」起头。
    写不进（没有路径、OSError）时 stderr 写一句。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": f"{hook_input.get('tool_name')} {tool_input.get('file_path', '')}"[:500],
        "findings": [f"写被拒：{finding}"] if outcome == "被拒" else [f"写闸没判全：{text}" for text in finding],
        "outcome": outcome,
    }
    if not detections_path:
        print("! write-guard.sh：读不到工作根的定义、又没设 AGENT_HOOK_DETECTIONS，这一条检出没地方记", file=sys.stderr)
        return
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        print(f"! write-guard.sh：检出记录 {detections_path} 写不进（{error}）", file=sys.stderr)

INPUT_PREFIX_BYTES = 200
INPUT_FIELD = r'"{}"\s*:\s*"((?:[^"\\]|\\.)*)"'

def unparsable_input_entry(raw, problem):
    """输入不是合法的 JSON 对象：从原文里按字面找得到的会话号、agent 号与类型、记录路径照抄，找不到的会话号与类型写「读不到」。"""
    text = raw.decode("utf-8", "replace")
    def field(name):
        found = re.search(INPUT_FIELD.format(name), text)
        return found.group(1) if found else None
    prefix = raw[:INPUT_PREFIX_BYTES].decode("utf-8", "replace")
    return ({"session_id": field("session_id") or "读不到", "agent_id": field("agent_id"), "agent_type": field("agent_type") or "读不到",
             "transcript_path": field("transcript_path"), "tool_name": field("tool_name"), "tool_input": {"file_path": field("file_path") or ""}},
            f"输入不是合法的 JSON 对象（{problem}），这一次没判、照常放行；读到 {len(raw)} 字节，前 {INPUT_PREFIX_BYTES} 字节：{prefix!r}")

def selftest_base_directory(table_path):
    """自检的现场放在哪。不能落进写范围表里任何一条仓外绝对路径模式（落进去，「越界」那几例会被那条模式放行），
    也不能落在某个 git 仓里（「仓外文件」那一例要真在仓外）。TMPDIR 指到哪都不影响判定：依次试几处，取第一处两条都满足的。"""
    absolute_patterns = [glob_to_regex(pattern) for patterns in load_scopes(table_path).values()
                         for pattern in expanded_scope_patterns(patterns) if pattern.startswith("/")]
    for candidate in (tempfile.gettempdir(), "/tmp", "/var/tmp", "/dev/shm"):
        if not os.path.isdir(candidate) or not os.access(candidate, os.W_OK):
            continue
        probe = os.path.join(candidate, "write-guard-selftest-probe", "crates", "a.rs")
        if any(pattern.match(os.path.normpath(probe)) for pattern in absolute_patterns):
            continue
        if subprocess.run(["git", "-C", candidate, "rev-parse", "--show-toplevel"], capture_output=True, text=True).returncode == 0:
            continue
        return candidate
    return None

def selftest(hook_dir):
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与过渡期旧根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    # 工作根下的样本路径不看弄坏开关（configured_work_root）；旧根照库里的写法
    new_root, old_root = work_roots.configured_work_root(), work_roots.legacy_root()
    base = selftest_base_directory(os.path.join(hook_dir, "agent-write-scope.tsv"))
    if base is None:
        print("  ✗ 自检找不到一处能放现场的临时目录：TMPDIR、/tmp、/var/tmp、/dev/shm 要么落进写范围表的仓外模式、要么在 git 仓里、要么写不进去")
        print("    → 怎么办：给一个不在 git 仓里、也不在 agent-write-scope.tsv 任何一条绝对路径模式底下的可写目录当 TMPDIR 再跑")
        return 1
    work = tempfile.mkdtemp(dir=base)
    outside_directory = work + "-outside"   # 符号链接那几格仓外的一头：work 旁边、同样不在 git 仓与写范围的绝对模式里
    cases = []
    library = notations_library_path(hook_dir)
    try:
        subprocess.run(["git", "init", "-q", work], check=True)
        subprocess.run(["git", "-C", work, "config", "user.email", "t@t"], check=True)
        subprocess.run(["git", "-C", work, "config", "user.name", "t"], check=True)
        os.makedirs(os.path.join(work, ".claude", "agents"))
        os.makedirs(os.path.join(work, ".claude", "kb"))
        for name in ("kb-scribe", "implementation-writer", "experiment-runner", "unscoped-writer"):
            open(os.path.join(work, ".claude", "agents", f"{name}.md"), "w").write("---\n")
        open(os.path.join(work, "tracked.md"), "w").write("a\n")
        subprocess.run(["git", "-C", work, "add", "tracked.md"], check=True)
        subprocess.run(["git", "-C", work, "commit", "-qm", "t"], check=True)
        open(os.path.join(work, "untracked.md"), "w").write("b\n")
        open(os.path.join(work, ".claude", "kb", "untracked-kb.md"), "w").write("c\n")
        outside = tempfile.NamedTemporaryFile(delete=False, dir=base)
        outside.close()
        table = os.path.join(work, "scope.tsv")
        shutil.copyfile(os.path.join(hook_dir, "agent-write-scope.tsv"), table)
        if os.environ.get("WRITE_GUARD_BREAK") == "scope-new-rows-dropped":   # 弄坏开关：表副本退回没有这两行的样子
            with open(table, encoding="utf-8") as table_source:
                kept_rows = [row for row in table_source if not row.startswith(("kb-scribe\tcrates/singlefs-checker/invariants/", "experiment-runner\t.claude/gate.d/replay-comparison-units.tsv\t"))]
            with open(table, "w", encoding="utf-8") as table_target:
                table_target.writelines(kept_rows)
        def case(label, tool_name, agent, path, want):
            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[0])
        def finding_case(label, tool_name, agent, path, want):
            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[2])
        os.makedirs(os.path.join(work, "scripts"))
        open(os.path.join(work, "scripts", "y.sh"), "w").write("#!/usr/bin/env bash\necho ok\n")
        # 落点：符号链接（弄坏开关 WRITE_GUARD_BREAK=landing-lexical-only 下解不开，「链接指到范围外」那几格红）；仓外的那一头放在 work 旁边的 outside_directory
        os.makedirs(outside_directory)
        os.makedirs(os.path.join(work, "crates", "singlefs-core"))
        os.makedirs(os.path.join(work, "crates", "singlefs-checker-tier", "src", "bin"))
        open(os.path.join(work, "crates", "singlefs-checker-tier", "src", "bin", "e161_x.rs"), "w").write("fn main() {}\n")
        os.symlink(outside_directory, os.path.join(work, "crates", "escape"))                              # 范围内的链接指到仓外
        os.symlink(os.path.join(outside_directory, "not-yet"), os.path.join(work, "crates", "dangling"))   # 范围内的悬空链接指到仓外
        os.symlink(os.path.join(work, ".claude", "kb"), os.path.join(work, "crates", "kb-link"))           # 范围内的链接指到仓里范围外
        os.symlink(os.path.join(work, "crates", "singlefs-core"), os.path.join(work, "crates", "inner"))   # 范围内的链接指到范围内
        os.symlink(os.path.join(work, "crates"), os.path.join(outside_directory, "back"))                  # 仓外的链接指回范围内
        os.symlink(os.path.join(work, "untracked.md"), os.path.join(outside_directory, "untracked-link"))   # 仓外的链接指到仓里未跟踪的文件
        os.symlink(os.path.join(work, "crates", "singlefs-checker-tier", "src", "bin", "e161_x.rs"), os.path.join(outside_directory, "e161-link.rs"))
        def content_case(label, tool_name, path, tool_input, want):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=path)}
            return (label, want, decide(hook_input, work, table, library)[0])
        def prime_case(label, tool_name, agent, tool_input, want, library_path=library):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/crates/a.rs")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library_path)[0])
        cases += [
            # 六、语法：钩子、门禁、项目脚本、研究脚本目录下的 .sh / .py
            content_case("语法:hooks 下写进语法错误的 .sh", "Write", f"{work}/.claude/hooks/x.sh", {"content": "if [ ; then\n  echo\n"}, 2),
            content_case("语法:hooks 下语法对的 .sh", "Write", f"{work}/.claude/hooks/x.sh", {"content": "#!/usr/bin/env bash\necho ok\n"}, 0),
            content_case("语法:Edit 把研究脚本改出语法错误", "Edit", f"{work}/scripts/y.sh", {"old_string": "echo ok", "new_string": "if [ ; then"}, 2),
            content_case("语法:Edit 研究脚本合法改动", "Edit", f"{work}/scripts/y.sh", {"old_string": "echo ok", "new_string": "echo fine"}, 0),
            content_case("语法:门禁目录下语法错误的 .py", "Write", f"{work}/.claude/gate.d/z.py", {"content": "def (:\n"}, 2),
            content_case("语法:门禁目录下语法对的 .py", "Write", f"{work}/.claude/gate.d/z.py", {"content": "print(1)\n"}, 0),
            content_case("语法:别处的 .sh 不判", "Write", f"{work}/crates/run.sh", {"content": "if [ ; then\n"}, 0),
            # 一、整份覆盖未跟踪文件
            case("覆盖:未跟踪的已有文件用 Write", "Write", None, f"{work}/untracked.md", 2),
            case("覆盖:未跟踪的已有文件用 Edit", "Edit", None, f"{work}/untracked.md", 0),
            case("覆盖:已跟踪文件", "Write", None, f"{work}/tracked.md", 0),
            case("覆盖:不存在的文件", "Write", None, f"{work}/missing.md", 0),
            case("覆盖:仓外文件", "Write", None, outside.name, 0),
            # 二、写范围
            case("范围:主 agent", "Edit", None, f"{work}/crates/a.rs", 0),
            # 没有项目定义的 agent 只许写工作根（弄坏开关 builtin-agents-unscoped 下「拒」的三格红）
            case("范围:内置 agent 写仓里拒", "Edit", "general-purpose", f"{work}/crates/a.rs", 2),
            case("范围:内置 agent 写仓外别处拒", "Write", "general-purpose", outside.name, 2),
            case("范围:定义已删的旧实例写仓里拒", "Write", "gate-triage", f"{work}/records/x.md", 2),
            case("范围:内置 agent 写工作根放行", "Write", "general-purpose", f"{new_root}/builtin-x/result.md", 0),
            case("范围:kb-scribe 范围内", "Edit", "kb-scribe", f"{work}/.claude/kb/decisions/01-x.md", 0),
            case("范围:kb-scribe 越界 crates", "Edit", "kb-scribe", f"{work}/crates/a.rs", 2),
            case("范围:kb-scribe 越界 records", "Edit", "kb-scribe", f"{work}/records/x.md", 2),
            case("范围:kb-scribe 写不变量清单的 .md", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/cp-01.md", 0),
            case("范围:kb-scribe 不变量清单目录里的 .rs 越界", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/x.rs", 2),
            case("范围:kb-scribe 写不变量清单子目录里的 .md", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/sub/x.md", 0),
            case("范围:kb-scribe 不变量清单子目录里的 .rs 越界", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/sub/x.rs", 2),
            case("范围:kb-scribe checker 源码越界", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/src/lib.rs", 2),
            case("范围:implementation-writer 范围内", "Edit", "implementation-writer", f"{work}/crates/singlefs-core/src/a.rs", 0),
            case("范围:.. 绕路", "Edit", "implementation-writer", f"{work}/crates/../.claude/kb/x.md", 2),
            case("范围:implementation-writer 越界 research", "Edit", "implementation-writer", f"{work}/research/x.md", 2),
            case("范围:工作根下的报告目录", "Write", "implementation-writer", f"{new_root}/agents/implementation-writer/result.md", 0),
            case("范围:过渡期旧根下的报告目录", "Write", "implementation-writer", f"{old_root}/agents/implementation-writer/result.md", 0),
            case("范围:工作根同名前缀的兄弟目录", "Write", "implementation-writer", f"{new_root}-other/result.md", 2),
            case("范围:仓外别处", "Edit", "implementation-writer", "/etc/passwd", 2),
            case("范围:范围内的链接指到范围外，经它写拒", "Edit", "implementation-writer", f"{work}/crates/escape/hosts", 2),
            case("范围:范围内的链接指到范围外，经它写不存在的子目录拒", "Write", "implementation-writer", f"{work}/crates/escape/no-such/x.rs", 2),
            case("范围:范围内的悬空链接指到范围外拒", "Write", "implementation-writer", f"{work}/crates/dangling", 2),
            case("范围:范围内的链接指到仓里范围外拒", "Write", "implementation-writer", f"{work}/crates/kb-link/x.md", 2),
            case("范围:范围内的链接指到范围内放行", "Edit", "implementation-writer", f"{work}/crates/inner/a.rs", 0),
            case("范围:范围外的链接指回范围内放行", "Edit", "implementation-writer", f"{outside_directory}/back/a.rs", 0),
            case("覆盖:仓外的链接指到仓里未跟踪的文件，Write 拒", "Write", None, f"{outside_directory}/untracked-link", 2),
            case("范围:experiment-runner 登记", "Edit", "experiment-runner", f"{work}/research/prompts/e12-preregistration.md", 0),
            case("范围:experiment-runner 三方正文", "Edit", "experiment-runner", f"{work}/research/prompts/_e12-body.md", 2),
            case("范围:experiment-runner replay.sh", "Edit", "experiment-runner", f"{work}/scripts/experiment-apparatus/replay.sh", 0),
            case("范围:experiment-runner mutate.sh", "Edit", "experiment-runner", f"{work}/scripts/mutation/mutate.sh", 2),
            case("范围:experiment-runner 写重跑比对对象登记", "Edit", "experiment-runner", f"{work}/.claude/gate.d/replay-comparison-units.tsv", 0),
            case("范围:experiment-runner 写别的门禁文件越界", "Edit", "experiment-runner", f"{work}/.claude/gate.d/doc-experiments.sh", 2),
            case("范围:未登记的项目 agent", "Edit", "unscoped-writer", f"{work}/anything.md", 2),
            case("两道都中时先报覆盖", "Write", "kb-scribe", f"{work}/untracked.md", 2),
            # 四、实验执行员对主工作区 crates/ 下 .rs 的写：拒的那一道要是「绕过先编后换」，不是写范围那一道（出路不同）
            finding_case("先编后换:experiment-runner Edit 主工作区的入库装置", "Edit", "experiment-runner",
                         f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", "绕过先编后换"),
            finding_case("先编后换:experiment-runner Write 新建入库装置", "Write", "experiment-runner",
                         f"{work}/crates/singlefs-checker-tier/src/bin/e999_new.rs", "绕过先编后换"),
            finding_case("先编后换:experiment-runner 改 crates 里 bin 以外的 .rs", "Edit", "experiment-runner",
                         f"{work}/crates/singlefs-core/src/lib.rs", "绕过先编后换"),
            finding_case("先编后换:.. 绕路进 crates 也算", "Edit", "experiment-runner",
                         f"{work}/research/../crates/singlefs-checker-tier/src/bin/e161_x.rs", "绕过先编后换"),
            finding_case("先编后换:仓外的链接指到主工作区的入库装置拒", "Edit", "experiment-runner", f"{outside_directory}/e161-link.rs", "绕过先编后换"),
            finding_case("先编后换:implementation-writer 同一处放行", "Edit", "implementation-writer",
                         f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", None),
            finding_case("先编后换:主 agent 同一处放行", "Edit", None, f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", None),
            finding_case("先编后换:experiment-runner 改草稿目录里的副本放行", "Edit", "experiment-runner",
                         f"{new_root}/runner-x/e161_x.rs", None),
            finding_case("先编后换:experiment-runner 追加 crates/mutations.tsv 放行", "Edit", "experiment-runner",
                         f"{work}/crates/mutations.tsv", None),
            # 三、撇号类角标：五个字符各写死一例（共用字符集少了哪一个，这里就红）
            prime_case("角标:Write 内容含 B2 加一撇（U+2032）", "Write", None, {"content": "取 B2\u2032 的读法\n"}, 2),
            prime_case("角标:Edit new_string 含 P1 加两撇（U+2033）", "Edit", None, {"old_string": "P1", "new_string": "P1\u2033"}, 2),
            prime_case("角标:Write 内容含三撇（U+2034）", "Write", None, {"content": "臂 A\u2034\n"}, 2),
            prime_case("角标:Edit new_string 含 U+02B9", "Edit", None, {"old_string": "丙", "new_string": "丙\u02b9"}, 2),
            prime_case("角标:Write 内容含 U+02BA", "Write", None, {"content": "候选 K9\u02ba\n"}, 2),
            prime_case("角标:MultiEdit 第二处 new_string 含", "MultiEdit", None,
                       {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": "K9\u2032"}]}, 2),
            prime_case("角标:子 agent 在写范围内也判", "Edit", "implementation-writer", {"old_string": "x", "new_string": "G5\u2032"}, 2),
            prime_case("角标:子 agent 同一处不带角标放行（上一例的 2 出自角标这一道）", "Edit", "implementation-writer", {"old_string": "x", "new_string": "G6"}, 0),
            prime_case("角标:ASCII 单引号放行", "Write", None, {"content": "S1' 与 it's\n"}, 0),
            prime_case("角标:只在 old_string 里有（删掉它的改动）放行", "Edit", None, {"old_string": "B2\u2032", "new_string": "B3"}, 0),
            prime_case("角标:MultiEdit 只在 old_string 里有放行", "MultiEdit", None,
                       {"edits": [{"old_string": "B2\u2032", "new_string": "B3"}]}, 0),
            prime_case("角标:读不到共用字符集就拒绝", "Write", None, {"content": "普通内容\n"}, 2,
                       library_path=os.path.join(work, "no-such-lib-forbidden-notations.py")),
        ]
        # 六、语法（续）：.py 在本进程里 ast 解析、不起子进程，.sh 起一个 bash -n；WRITE_GUARD_BREAK=python-syntax-subprocess 下「.py 在本进程里判」一格红
        real_run, started = subprocess.run, []
        def counting_run(command, *run_arguments, **run_keywords):
            started.append(command)
            return real_run(command, *run_arguments, **run_keywords)
        def checker_starts_of(path, content):
            del started[:]
            decided = decide({"tool_name": "Write", "tool_input": {"file_path": path, "content": content}}, work, table, library)
            return [command for command in started if command[0] in ("bash", sys.executable)], decided[1] or ""
        subprocess.run = counting_run
        try:
            python_starts, python_message = checker_starts_of(f"{work}/.claude/gate.d/z.py", "def (:\n")
            shell_starts, _ = checker_starts_of(f"{work}/.claude/hooks/x.sh", "if [ ; then\n  echo\n")
        finally:
            subprocess.run = real_run
        cases += [
            ("语法:写坏的 .py 在本进程里判，不起 python 子进程", 0, len(python_starts)),
            ("语法:写坏的 .py 的拒绝说明点名文件与 SyntaxError", True, ".claude/gate.d/z.py" in python_message and "SyntaxError" in python_message and "python3 的 ast 解析" in python_message),
            ("语法:写坏的 .sh 仍起一个 bash -n", ["bash"], [command[0] for command in shell_starts]),
        ]
        # 共用判据库一次写入只装一次：撇号、钟点、会话号三道都读它；WRITE_GUARD_BREAK=library-reloaded-each-time 下这一格红（装三回）
        NOTATIONS_LIBRARIES.clear()
        NOTATIONS_LIBRARY_LOADS[0] = 0
        decide({"tool_name": "Write", "tool_input": {"file_path": f"{work}/records/library-once.md", "content": "普通内容\n"}}, work, table, library, library)
        cases.append(("库:一次写入判撇号、钟点、会话号三道，共用判据库只装一次", 1, NOTATIONS_LIBRARY_LOADS[0]))
        # 八、old_string 以空白收尾：判的是拒绝那一道的名字；WRITE_GUARD_BREAK=trailing-blank-ignored 时拒绝的几例就红。
        # 不是 .md / .mdx 的目标，钩子收到的 new_string 已去过行尾空白，例子里照那样写
        trailing_finding = "old_string 以空白收尾"
        def trailing_case(label, tool_name, tool_input, want, agent=None, target=None):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=target or f"{work}/records/x.sh")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[2])
        cases += [
            trailing_case("行尾空白:.sh 目标 old_string 以空格收尾", "Edit", {"old_string": "x = 1\ndeclared = ", "new_string": "x = 2\ndeclared ="}, trailing_finding),
            trailing_case("行尾空白:.sh 目标 old_string 以制表符收尾", "Edit", {"old_string": "key\t", "new_string": "key"}, trailing_finding),
            trailing_case("行尾空白:MultiEdit 第二处 old_string 以空格收尾", "MultiEdit",
                          {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "python3 - ", "new_string": "python3 -"}]}, trailing_finding),
            trailing_case("行尾空白:.txt 目标也判", "Edit", {"old_string": "declared = ", "new_string": "declared ="}, trailing_finding, target=f"{work}/records/x.txt"),
            trailing_case("行尾空白:.markdown 目标也判（Edit 只在 .md / .mdx 上留行尾空白）", "Edit", {"old_string": "declared = ", "new_string": "declared ="},
                          trailing_finding, target=f"{work}/records/x.markdown"),
            trailing_case("行尾空白:子 agent 写仓外文件也判", "Edit", {"old_string": "a ", "new_string": "b"}, trailing_finding,
                          agent="implementation-writer", target=f"{new_root}/agents/implementation-writer/notes.txt"),
            trailing_case("行尾空白:.md 目标 old_string 以空格收尾放行", "Edit", {"old_string": "x = 1\ndeclared = ", "new_string": "x = 2\ndeclared = "}, None,
                          target=f"{work}/records/x.md"),
            trailing_case("行尾空白:.mdx 目标放行", "Edit", {"old_string": "a ", "new_string": "b "}, None, target=f"{work}/records/x.mdx"),
            trailing_case("行尾空白:大写 .MD 目标放行", "Edit", {"old_string": "a\t", "new_string": "b\t"}, None, target=f"{work}/records/X.MD"),
            trailing_case("行尾空白:.sh 目标 old_string 不以空白收尾放行", "Edit", {"old_string": "x = 1\ndeclared = foo", "new_string": "x = 2\ndeclared = foo"}, None),
            trailing_case("行尾空白:.sh 目标只在中间行有行尾空白放行", "Edit", {"old_string": "x = 1  \ny", "new_string": "x = 3\ny"}, None),
            trailing_case("行尾空白:old_string 以换行收尾、换行前有空白放行", "Edit", {"old_string": "a \n", "new_string": "b\n"}, None),
            trailing_case("行尾空白:只有 new_string 以空白收尾放行（旧判据已撤）", "Edit", {"old_string": "a", "new_string": "b "}, None),
            trailing_case("行尾空白:Write 不判", "Write", {"content": "x = 1 "}, None, target=f"{work}/records/new.sh"),
        ]
        # 五、钟点：样本串用 colon 拼、时区词用 jst / utc 两个变量拼，这份源码里不出现字面钟点与时区词（门禁 doc-text 也扫它）
        colon = ":"
        jst, utc = "J" + "ST", "U" + "TC"
        clock_library = notations_library_path(hook_dir)
        def clock_case(label, target, tool_input, want, agent=None, library_path=clock_library):
            hook_input = {"tool_name": "Edit" if "new_string" in tool_input else "Write",
                          "tool_input": dict(tool_input, file_path=target if os.path.isabs(target) else f"{work}/{target}")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library, library_path)[2])
        cases += [
            clock_case("钟点:日期 时区词 时分x", "records/x.md", {"content": f"用户 2026-09-26 {jst} 15{colon}3x 定\n"}, "描述里写了钟点"),
            clock_case("钟点:日期 时分 时区词 前后", "records/x.md", {"content": f"实十九（2026-09-24 16{colon}50 {utc} 前后落地）\n"}, "描述里写了钟点"),
            clock_case("钟点:时区词 时分，不带日期", ".claude/kb/x.md", {"old_string": "a", "new_string": f"同日 {utc} 14{colon}00 前后：\n"}, "描述里写了钟点"),
            clock_case("钟点:时分 时区词，不带日期", "crates/a.rs", {"content": f"// 跑于 03{colon}44 {jst}\n"}, "描述里写了钟点"),
            clock_case("钟点:带 x 通配的裸时分", "records/x.md", {"content": f"主 agent 22{colon}5x 跑前修订\n"}, "描述里写了钟点"),
            clock_case("钟点:子 agent 也判", "research/prompts/e12-preregistration.md", {"content": f"交回 {jst} 22{colon}0x\n"}, "描述里写了钟点", agent="experiment-runner"),
            clock_case("钟点:只写日期放行", "records/x.md", {"content": "用户 2026-09-26 定\n"}, None),
            clock_case("钟点:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": f"2026-09-27 22{colon}41 {jst} 编过\n"}, None),
            clock_case("钟点:排除的门禁样本放行", ".claude/gate.d/fixtures/x/red/expect", {"content": f"改于 2026-09-05 00{colon}00\n"}, None),
            clock_case("钟点:排除的 hook 自检放行", ".claude/hooks/session-start.sh", {"content": f"2026-09-25 07{colon}58{colon}02 {utc}\n"}, None),
            clock_case("钟点:排除的 c510 提示放行", "research/prompts/_c510-date-gate-r1-body.md", {"content": f"本机时钟是 {utc}\n"}, None),
            clock_case("钟点:只在 old_string 里有放行", "records/x.md", {"old_string": f"{jst} 15{colon}3x", "new_string": ""}, None),
            clock_case("钟点:读不到判据就拒绝", "records/x.md", {"content": "普通内容\n"}, "描述里写了钟点",
                       library_path=os.path.join(work, "no-such-lib-forbidden-notations.py")),
        ]
        # 五、钟点里的各个判据行：样本串用 glued 把数字与「点」、日期与时段词拆开拼，这份源码里不出现字面写法；
        # 判的是拒绝信息点名的判据行名字，LIB_FORBIDDEN_NOTATIONS_BREAK 关掉哪一条判据行，点名它的那几例就红；
        # 放行的几组分别靠上下文规则、放行标记，LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT / _ALLOW 点名钟点那一形态时它们就红
        def glued(*parts):
            return "".join(parts)
        def clock_shape_case(label, content, want_shape, target="records/x.md"):
            code, message, finding = decide({"tool_name": "Write", "tool_input": {"file_path": f"{work}/{target}", "content": content + "\n"}},
                                            work, table, library, clock_library)
            shape = re.search(r"描述里写了钟点，共 \d+ 处：\n  [^（\n]*（(.+?)）：「", message) if finding == "描述里写了钟点" else None
            return (label, want_shape, shape.group(1) if shape else finding)
        hour_shape, period_shape, period_hour_shape, suffix_shape = "日期或时区后面跟「N 点 / N 时」", "日期或时区后面跟时段词", "时段词后面跟钟点", "「N 点前后」这类钟点"
        date_shape, bare_shape, iso_shape, zone_shape = "日期后面跟钟点", "不挨日期时区的钟点", "ISO 时间戳", "时区词"
        twelve_hour_shape = "十二小时制钟点"
        cases += [
            clock_shape_case("钟点:限额通知原样的英文月日十二小时制", glued("通知写 resets Oct 4 4", f"pm {utc}"), twelve_hour_shape),
            clock_shape_case("钟点:十二小时制隔空格大写", glued("11 ", "AM 截止"), twelve_hour_shape),
            clock_shape_case("钟点:十二小时制带点", glued("交回于 3", "p.m."), twelve_hour_shape),
            clock_shape_case("钟点:以 am / pm 起头的英文词放行", glued("2 AMD 显卡、3 ", "amps、5 ppm、pm2 进程、v1.2", "am"), None),
        ]
        # 七、会话号：样本串用 glued 把目录名与十六进制拆开拼，这份源码里不出现会话号；
        # WRITE_GUARD_DISABLE_SESSION_NAMES=1 或 LIB_FORBIDDEN_NOTATIONS_BREAK=session-names 时拒绝的几例就红，
        # LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names 时带放行标记的那一例就红
        session_shape = "写了会话号"
        cases += [
            clock_shape_case("会话号:标签式（会话 号，日期）", glued("### 第 4 批开工（会话 singlefs-", "21，2026-10-01）"), session_shape),
            clock_shape_case("会话号:当主语用的", glued("已发消息，singlefs-", "e1 回话"), session_shape),
            clock_shape_case("会话号:上游三语仓的", glued("SOP 会话 singlefs-ai-sop-zh-", "86 同步来"), session_shape),
            clock_shape_case("会话号:四位十六进制", glued("会话 singlefs-", "e152 在跑"), session_shape),
            clock_shape_case("会话号:crate 名、上游仓目录名、临时目录名放行",
                             glued("改 singlefs-core 与 singlefs-ai-sop-zh；mktemp -d /tmp/singlefs-", "e152.XXXXXX；/tmp/singlefs-", "e152-packages"), None),
            clock_shape_case("会话号放行标记:sh 里的自检输入放行",
                             glued("echo singlefs-", "6a  # session-names:allow 解析 ListAgents 输出的自检输入"), None, target=".claude/scripts/parse.sh"),
            clock_shape_case("钟点:日期 时区词 N 点前后", glued(f"用户 2026-09-28 {jst} 08 ", "点前后要开"), hour_shape),
            clock_shape_case("钟点:时区词 日期 N 点后（时区词那一处先中）", glued(f"本机 {utc} 2026-09-26 23 ", "点后"), zone_shape),
            clock_shape_case("钟点:日期 时区词 N 时许", glued(f"2026-09-16 {utc} 17 ", "时许的快照"), hour_shape),
            clock_shape_case("钟点:不带年份的日期 时区词 N 点前后", glued(f"09-24 {jst} 19 ", "点前后问了四件"), hour_shape),
            clock_shape_case("钟点:日期 时区词 N 点后", glued(f"2026-09-25 {jst} 01 ", "点后弹窗"), hour_shape),
            clock_shape_case("钟点:日期 时区词 时段词", glued(f"交回于 2026-09-27 {jst} ", "上午"), period_shape),
            clock_shape_case("钟点:括号里日期后跟时段词", glued("降为待证（2026-09-13 ", "上午）"), period_shape),
            clock_shape_case("钟点:日期时段词再括号时区词", glued("2026-09-12 ", f"凌晨（{jst}）"), period_shape),
            clock_shape_case("钟点:括号里时区词日期时段词（时区词那一处先中）", glued(f"（{jst} 2026-09-27 ", "早上）"), zone_shape),
            clock_shape_case("钟点:日期时段词前后", glued("满载到 2026-09-28 ", "中午前后"), period_shape),
            clock_shape_case("钟点:标题里日期后括号时段词", glued("### 2026-09-27（", "下午）：E162"), period_shape),
            clock_shape_case("钟点:日期时段词再时分（日期那一处先中）", glued("2026-09-27 ", f"上午 08{colon}32 现跑"), period_shape),
            clock_shape_case("钟点:时段词后跟时分", f"上午 08{colon}32 现跑", period_hour_shape),
            clock_shape_case("钟点:时段词后跟 N 点", glued("晚上 ", "8 点"), period_hour_shape),
            clock_shape_case("钟点:光秃的 N 点前后", glued("19 ", "点前后问了"), suffix_shape),
            clock_shape_case("钟点:裸时分", f"用户 23{colon}10 定", bare_shape),
            clock_shape_case("钟点:裸时分秒", f"交回 14{colon}00{colon}10、续做", bare_shape),
            clock_shape_case("钟点:裸时分秒带毫秒", f"07{colon}58{colon}02.123 起跑", bare_shape),
            clock_shape_case("钟点:时分区间", f"14{colon}00–15{colon}30 之间跑完", bare_shape),
            clock_shape_case("钟点:月-日 时分", f"核查员缺快照（09-24 23{colon}44 那一次）", bare_shape),
            clock_shape_case("钟点:反引号里两位小时的时分", f"跑于 `09{colon}55` 的那一轮", bare_shape),
            clock_shape_case("钟点:日期后面跟带秒带偏移的 diff 头", f"--- a/x.rs\t2026-09-24 16{colon}27{colon}05.398761624 +0000", date_shape),
            clock_shape_case("钟点:stat 的修改时间", f"Modify: 2026-09-24 18{colon}48{colon}58.254848952 +0000", date_shape),
            clock_shape_case("钟点:ISO 时间戳", f"派腿 2026-09-27T09{colon}55{colon}07+00{colon}00", iso_shape),
            clock_shape_case("钟点:不带日期的 T 钟点", f"交回于 …T23{colon}15Z", iso_shape),
            clock_shape_case("钟点:光秃的时区词", f"本机时钟是 {utc}、直接 date 会差一天", zone_shape),
            clock_shape_case("钟点:日期后括号时区词", f"跑于 2026-09-27（{jst}），本机", zone_shape),
            clock_shape_case("钟点:两个时区的名字", f"{jst} 与 {utc} 两个时区", zone_shape),
            clock_shape_case("钟点:时区词 时间", f"{utc} 时间", zone_shape),
            clock_shape_case("钟点:放行标记写在 md 里不认", f"用户 23{colon}10 定  # clock-times:allow 写在 md 里不认", bare_shape),
            clock_shape_case("钟点:放行标记没写理由不认", f"// 跑于 23{colon}10  // clock-times:allow", bare_shape, target="crates/a.rs"),
            clock_shape_case("钟点:第 N 点前后是序号放行", "第 3 点前后两句", None),
            clock_shape_case("钟点:N 点意见放行", "3 点意见", None),
            clock_shape_case("钟点:个数后的点放行", "1000 个崩溃点", None),
            clock_shape_case("钟点:夜间放行", "全量由用户要求或夜间跑", None),
            clock_shape_case("钟点:只写日期放行（Write 整份）", "用户 2026-09-28 定", None),
            clock_shape_case("钟点:不挨日期时区的光秃时段词放行", "同一天下午续跑", None),
            clock_shape_case("钟点:标识符里的时区词放行", f"`{jst}_OFFSET`、`started_{utc}`", None),
        ]
        context_cases = [
            clock_shape_case("钟点上下文:产物行的名字=值放行", f"remount_chosen=2{colon}12 device_chosen=1{colon}16 values=1{colon}16.000,3{colon}12.000", None),
            clock_shape_case("钟点上下文:差分与计数的冒号对放行", f"adjacent_differences=0->1{colon}10,1->2{colon}8 steps_by_changed_leaf_count=1{colon}108,2{colon}36", None),
            clock_shape_case("钟点上下文:字典放行", f"分布 {{0{colon}14, 1{colon}14, 2{colon}16}}", None),
            clock_shape_case("钟点上下文:Python 切片放行", f"r[8{colon}12]=i.to_bytes(4); fields[11{colon}15]; arguments[5{colon}13]", None),
            clock_shape_case("钟点上下文:代码里的时区偏移放行", f'stamp.replace("Z", "+00{colon}00")', None),
            clock_shape_case("钟点上下文:双引号里的代码字面放行", f'            Some("2{colon}11"),', None),
            clock_shape_case("钟点上下文:单引号里的代码字面放行", f"                      printf '2{colon}11' ;;", None),
            clock_shape_case("钟点上下文:百分比放行", f"B Lost Writes **13{colon}54%**", None),
            clock_shape_case("钟点上下文:time -v 的格式说明放行", f"Elapsed (wall clock) time (h:mm:ss or m:ss): 0{colon}03.21", None),
            clock_shape_case("钟点上下文:门禁号行号表格放行", f"| 共用脚本不在 | 21{colon}38 | 31{colon}68 |", None),
            clock_shape_case("钟点上下文:门禁号行号列表放行", f"| kb 目录不在 | —— | 21{colon}39、24{colon}27、27{colon}39 |", None),
            clock_shape_case("钟点上下文:门禁文件名放行", f"门禁 20{colon}09-sample-stage.sh 那一行", None),
            clock_shape_case("钟点上下文:同一行带文件行号的编号对放行", f"触发的那一处（decisions/06-快照实现模型.md{colon}42），06{colon}42 正是没抽到的那一行", None),
            clock_shape_case("钟点上下文:反引号里一位小时的键放行", f"撞键 `3{colon}10`：冷重开是 `2{colon}11`", None),
            clock_shape_case("钟点上下文:产物字段里的 ISO 时间戳放行", f"taken_jst=2026-09-27T01{colon}20{colon}12+09{colon}00", None),
        ]
        allow_cases = [
            clock_shape_case("钟点放行标记:sh 里的 TZ 环境变量放行", f"TZ={utc}-14 date +%F  # clock-times:allow 环境变量写法，符号与东京时间相反", None, target=".claude/scripts/tz.sh"),
            clock_shape_case("钟点放行标记:py 里解析外部通知的正则放行", f'RESETS = re.compile(r"resets (\\d+)(am|pm) \\({utc}\\)")  # clock-times:allow 解析外部通知文本', None,
                             target=".claude/hooks/parse.py"),
            clock_shape_case("钟点放行标记:rs 里造时间戳的自检输入放行", f'let stamp = "2026-09-27T09{colon}55{colon}07Z"; // clock-times:allow 造时间戳格式的自检输入', None,
                             target="crates/a.rs"),
        ]
        cases += context_cases + allow_cases
        # 五、ISO 基本格式的紧凑时间戳（运行目录名、日志文件名里连写的那种）：样本串用 glued 把日期与「T 加钟点」拆开拼，这份源码里不出现字面写法；
        # 拒绝的几例判的是拒绝信息点名的形态；WRITE_GUARD_BREAK=compact-timestamp-skipped 或 LIB_FORBIDDEN_NOTATIONS_BREAK=iso-compact-timestamp 时它们红，
        # 放行的「8 位数字后跟 T 但不是时间戳」几例在 LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose 时红
        compact_shape = "ISO 紧凑时间戳"
        compact_sample = glued("运行目录 20261014", "T153000Z")
        compact_cases = [
            clock_shape_case("紧凑时间戳:8 位日期、T、6 位钟点、Z", compact_sample, compact_shape),
            clock_shape_case("紧凑时间戳:8 位日期、T、4 位钟点", glued("日志 20261014", "T1530"), compact_shape),
            clock_shape_case("紧凑时间戳:8 位日期、T、带冒号的时分", glued("跑于 20261014", "T15", colon, "30"), compact_shape),
            clock_shape_case("紧凑时间戳:带连字符的日期加连写钟点", glued("run 2026-10-14", "T153000"), compact_shape),
            clock_shape_case("紧凑时间戳:带小数秒与偏移", glued("20261014", "T153000.5+0900"), compact_shape),
            clock_shape_case("紧凑时间戳:路径里的运行目录名也判", glued("见 logs/run-20261014", "T153000Z/out.log"), compact_shape),
        ]
        compact_lookalike_cases = [
            clock_shape_case("紧凑时间戳:月份越界放行", glued("编号 20261314", "T1530"), None),
            clock_shape_case("紧凑时间戳:钟点越界放行", glued("编号 20261014", "T2560"), None),
            clock_shape_case("紧凑时间戳:年不是 20 开头放行", glued("编号 12345678", "T0000"), None),
            clock_shape_case("紧凑时间戳:钟点只有三位放行", glued("编号 20261014", "T123"), None),
            clock_shape_case("紧凑时间戳:8 位数字全是 9 放行", glued("编号 99999999", "T123456"), None),
            clock_shape_case("紧凑时间戳:前面挨着字母放行", glued("哈希 ab20261014", "T1530"), None),
            clock_shape_case("紧凑时间戳:小写 t 放行", glued("哈希 20261014", "t1530"), None),
        ]
        compact_pass_cases = [
            clock_shape_case("紧凑时间戳:只写日期放行", "运行目录 20261014", None),
            clock_shape_case("紧凑时间戳:.py 里带理由的放行标记放行", glued('stamp = "20261014', 'T153000Z"  # clock-times:allow 造紧凑时间戳的自检输入'), None,
                             target=".claude/hooks/parse.py"),
            clock_case("紧凑时间戳:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": compact_sample + "\n"}, None),
        ]
        cases += compact_cases + compact_lookalike_cases + compact_pass_cases
        # 钟点、会话号逐处全报（行号与总数；LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only 下前两格红）；Edit 的行号折算成文件里的行号；
        # 九道判完再报：覆盖、越界与钟点三道一次列全（WRITE_GUARD_BREAK=stop-at-first 下末格红）
        def refusal_of(tool_name, target, tool_input, agent=None):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/{target}")}
            if agent:
                hook_input["agent_type"] = agent
            code, message, finding = decide(hook_input, work, table, library, clock_library)
            return message or "", finding
        two_clocks, _ = refusal_of("Write", "records/x.md", {"content": f"用户 23{colon}10 定\n中间一行\n交回 14{colon}00{colon}10、续做\n"})
        cases.append(("钟点:两处都报（行号与总数）", True, all(fragment in two_clocks for fragment in ("共 2 处", "  第 1 行（", "  第 3 行（"))))
        two_sessions, _ = refusal_of("Write", "records/x.md", {"content": glued("会话 singlefs-", "e152 在跑\n已发消息，singlefs-", "e1 回话\n")})
        cases.append(("会话号:两处都报（行号与总数）", True, all(fragment in two_sessions for fragment in ("共 2 处", "  第 1 行：", "  第 2 行："))))
        edit_lines, _ = refusal_of("Edit", "tracked.md", {"old_string": "a", "new_string": f"x\n用户 23{colon}10 定"})
        cases.append(("钟点:Edit 的行号折算成文件里的行号", True, "  第 2 行（" in edit_lines))
        # 紧凑时间戳：Write、Edit、MultiEdit 三个工具各写一次，拒绝点名形态与命中的那一串（开关见上面紧凑时间戳那一段）
        compact_stamp = compact_sample[len("运行目录 "):]
        for tool_name, tool_input in (("Write", {"content": compact_sample + "\n"}),
                                      ("Edit", {"old_string": "a", "new_string": compact_sample}),
                                      ("MultiEdit", {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": compact_sample}]})):
            compact_message, compact_finding = refusal_of(tool_name, "records/x.md", tool_input)
            cases.append((f"紧凑时间戳:{tool_name} 写进仓里拒绝并点名形态与那一处", True,
                          compact_finding == "描述里写了钟点" and f"（{compact_shape}）：「{compact_stamp}」" in compact_message))
        _, three_labels = refusal_of("Write", "untracked.md", {"content": f"用户 23{colon}10 定\n"}, agent="kb-scribe")
        cases.append(("九道判完再报:覆盖、越界与钟点一次列全", "整份覆盖未跟踪文件、越出写范围、描述里写了钟点", three_labels))
        # 走真实入口：把脚本当子进程、从标准输入喂 JSON，防「判定函数对、入口读不到输入」；拒绝要落进检出记录
        script = os.path.join(hook_dir, "write-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def run_entry(hook_input):
            return subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment)
        def stdin_case(label, hook_input, want):
            return (label, want, run_entry(hook_input).returncode)
        cases += [
            stdin_case("stdin:主 agent 写新文件", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/crates/a.rs", "content": "x"}}, 0),
            stdin_case("stdin:覆盖未跟踪文件", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/untracked.md", "content": "x"}}, 2),
            stdin_case("stdin:kb-scribe 越界", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/crates/a.rs"}}, 2),
            stdin_case("stdin:kb-scribe 范围内", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/.claude/kb/x.md"}}, 0),
            stdin_case("stdin:kb-scribe 用 MultiEdit 越界", {"tool_name": "MultiEdit", "agent_type": "kb-scribe",
                                                            "tool_input": {"file_path": f"{work}/crates/a.rs", "edits": [{"old_string": "a", "new_string": "b"}]}}, 2),
            stdin_case("stdin:kb-scribe 用 MultiEdit 范围内", {"tool_name": "MultiEdit", "agent_type": "kb-scribe",
                                                              "tool_input": {"file_path": f"{work}/.claude/kb/x.md", "edits": [{"old_string": "a", "new_string": "b"}]}}, 0),
            stdin_case("stdin:大内容越界", {"tool_name": "Write", "agent_type": "implementation-writer", "tool_input": {"file_path": f"{work}/research/big.md", "content": "y" * 300000}}, 2),
            stdin_case("stdin:实现员写工作根下的报告", {"tool_name": "Write", "agent_type": "implementation-writer",
                                                       "tool_input": {"file_path": f"{new_root}/x/result.md", "content": "x"}}, 0),
        ]
        runner_entry = run_entry({"tool_name": "Edit", "agent_type": "experiment-runner",
                                  "tool_input": {"file_path": f"{work}/crates/singlefs-checker-tier/src/bin/e161_x.rs", "old_string": "a", "new_string": "b"}})
        cases.append(("stdin:experiment-runner 改主工作区的入库装置拒绝", 2, runner_entry.returncode))
        cases.append(("stdin:先编后换的拒绝点名那条脚本与草稿目录", True,
                      all(fragment in runner_entry.stderr for fragment in ("scripts/hosts/compile-then-swap.py", "草稿目录", "→ 怎么办"))))
        prime_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/crates/a.rs", "old_string": "B2", "new_string": "取 B2\u2032 的读法"}})
        cases.append(("stdin:Edit 写进角标", 2, prime_entry.returncode))
        cases.append(("stdin:角标的拒绝带字符、前后几个字与出路", True,
                      all(fragment in prime_entry.stderr for fragment in ("撇号类角标 \u2032", "取 B2\u2032 的读法", "→ 怎么办", "path-moves.md"))))
        trailing_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/x.sh", "old_string": "python3 - ", "new_string": "python3 -"}})
        cases.append(("stdin:.sh 目标 Edit old_string 以空白收尾", 2, trailing_entry.returncode))
        cases.append(("stdin:行尾空白的拒绝带收尾那几个字、两条出路", True,
                      all(fragment in trailing_entry.stderr for fragment in ("「…python3 -␠」", "→ 怎么办", "多框到下一个非空白字符", "replace-once.py"))))
        cases.append(("stdin:.md 目标 Edit old_string 以空白收尾放行", 0,
                      run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/x.md", "old_string": "python3 - ", "new_string": "python3 - "}}).returncode))
        recorded = sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        cases.append(("stdin:七次拒绝都落进检出记录", 7, recorded))
        compact_entry = run_entry({"tool_name": "Write", "tool_input": {"file_path": f"{work}/records/compact.md", "content": compact_sample + "\n"}})
        cases.append(("stdin:Write 写进紧凑时间戳拒绝（退出码 2）", 2, compact_entry.returncode))
        cases.append(("stdin:紧凑时间戳的拒绝点名形态与那一处", True,
                      all(fragment in compact_entry.stderr for fragment in (f"（{compact_shape}）：「{compact_stamp}」", "→ 怎么办", "紧凑写法"))))
        # 没有项目定义的 agent 写仓里：入口拒，拒绝说明点名只许写工作根与交回主 agent 的出路（弄坏开关 builtin-agents-unscoped 下红）
        builtin_entry = run_entry({"tool_name": "Write", "agent_type": "general-purpose", "tool_input": {"file_path": f"{work}/crates/builtin.rs", "content": "x"}})
        cases.append(("stdin:内置 agent 写仓里拒", (2, True),
                      (builtin_entry.returncode, all(fragment in builtin_entry.stderr for fragment in ("只许写工作根", "交回主 agent", "→ 怎么办")))))
        link_entry = run_entry({"tool_name": "Edit", "agent_type": "implementation-writer",
                                "tool_input": {"file_path": f"{work}/crates/escape/hosts", "old_string": "a", "new_string": "b"}})
        cases.append(("stdin:范围内的链接指到范围外拒（退出码 2）", 2, link_entry.returncode))
        # 判不了照常放行而记检出：坏 JSON、JSON 数组各喂一次，检出记录单用一份（开关 unparsable-input-silent 下诊断与检出那几格红）
        unjudged_detections = os.path.join(work, "unjudged-detections.jsonl")
        def run_raw(raw_text, extra_environment=None):
            return subprocess.run(["bash", script], input=raw_text, capture_output=True, text=True,
                                  env=dict(environment, AGENT_HOOK_DETECTIONS=unjudged_detections, **(extra_environment or {})))
        def unjudged_records():
            return [json.loads(line) for line in open(unjudged_detections, encoding="utf-8")] if os.path.exists(unjudged_detections) else []
        broken_entry = run_raw('{"tool_name": "Write", "tool_input": {"file_path": "x.md"')
        broken_records = unjudged_records()
        cases.append(("stdin:坏 JSON 放行（退出码 0）", 0, broken_entry.returncode))
        cases.append(("stdin:坏 JSON 的 stderr 有诊断", True,
                      all(fragment in broken_entry.stderr for fragment in ("write-guard.sh", "不是合法的 JSON 对象", '{"tool_name": "Write"'))))
        cases.append(("stdin:坏 JSON 记一条检出（结局只记、会话号读不到）", (1, "只记", "读不到", True),
                      (len(broken_records), broken_records[-1].get("outcome") if broken_records else None, broken_records[-1].get("session_id") if broken_records else None,
                       bool(broken_records) and all(str(finding).startswith("写闸没判全：") for finding in broken_records[-1].get("findings") or [None]))))
        run_raw('{"session_id": "selftest-session", "agent_type": "implementation-writer", "tool_name": "Edit"')
        cases.append(("stdin:坏 JSON 里按字面找得到的会话号与类型照抄", ("selftest-session", "implementation-writer"),
                      tuple(unjudged_records()[-1].get(key) for key in ("session_id", "agent_type")) if len(unjudged_records()) == 2 else None))
        array_entry = run_raw("[1, 2]")
        cases.append(("stdin:JSON 数组记一条检出", (0, 3), (array_entry.returncode, len(unjudged_records()))))
        # ── 九、草稿目录写者锁：SINGLEFS_WORK_ROOT 指到自检自己的工作根，锁与主会话记录都在里面 ──
        # 开关见文件头 --selftest 一段；下面「写者锁:」「stdin:第二个写者」「stdin:持锁者」开头的格各自点名在开关下红
        lock_root = tempfile.mkdtemp(dir=base)
        work_root_before = os.environ.get("SINGLEFS_WORK_ROOT")
        os.environ["SINGLEFS_WORK_ROOT"] = lock_root
        try:
            if writer_lock is None:
                cases.append(("写者锁:库读得到", True, False))
            else:
                prompt_one, prompt_other, prompt_three = "写者锁自检的派发提示甲", "写者锁自检的派发提示乙", "写者锁自检的派发提示丙"
                prompt_four = "写者锁自检的派发提示丁"   # 没有任何回执对得上它
                lock_transcript = os.path.join(lock_root, "main-session.jsonl")
                def make_lock(name, prompt_text):
                    scratch = os.path.join(lock_root, name)
                    writer_lock.create_lock(scratch, writer_lock.new_lock("self-test", prompt_text, "implementation-writer", lock_transcript))
                    return scratch
                def receipt(agent_id, prompt_text, seconds_ago=0):
                    stamped = datetime.fromtimestamp(time.time() - seconds_ago, timezone.utc).isoformat().replace("+00:00", "Z")
                    return json.dumps({"timestamp": stamped, "toolUseResult": {"status": "async_launched", "agentId": agent_id, "prompt": prompt_text}}, ensure_ascii=False)
                for name in ("w1", "w2", "w4", "w5", "w6", "w7/sub", "w9"):
                    os.makedirs(os.path.join(lock_root, name), exist_ok=True)
                make_lock("w1", prompt_one)
                make_lock("w4", prompt_three)
                make_lock("w6", prompt_one)       # 回执对不上的拒绝点名持锁者：持锁者（提示甲）还没写过
                make_lock("w7", prompt_one)       # 嵌套：父目录锁是持锁者甲的
                make_lock("w7/sub", prompt_other)  # 嵌套：子目录锁是对手乙的
                make_lock("w9", prompt_four)      # 没有回执对得上这把锁
                open(os.path.join(lock_root, "w5", writer_lock.LOCK_NAME), "w", encoding="utf-8").write("{不是 JSON")
                for name, dropped in (("w13", "nonce"), ("w14", "dispatched_at"), ("w15", "prompt_sha256")):   # 锁文件在、字段缺一项（开关 unreadable-lock-absent 下这几格红）
                    os.makedirs(os.path.join(lock_root, name))
                    partial = writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    partial.pop(dropped)
                    with open(os.path.join(lock_root, name, writer_lock.LOCK_NAME), "w", encoding="utf-8") as partial_file:
                        json.dump(partial, partial_file)
                os.makedirs(os.path.join(lock_root, "w16", writer_lock.LOCK_NAME))   # 锁的位置是个目录
                for name, broken_claim in (("w22", "{不是 JSON"), ("w23", json.dumps({"claimed_at": 1}))):   # 认领文件在而判不了（开关 broken-claim-absent 下点名那份认领文件的几格红）
                    broken_scratch = make_lock(name, prompt_one)
                    with open(writer_lock.claim_path(broken_scratch, writer_lock.lock_state(broken_scratch)[1]), "w", encoding="utf-8") as claim_file:
                        claim_file.write(broken_claim)
                holder_id, rival_id, zombie_id, first_id, second_id = ("a0000000000000aa1", "a0000000000000bb2", "a0000000000000cc3", "a0000000000000dd4", "a0000000000000ee5")
                with open(lock_transcript, "w", encoding="utf-8") as transcript_file:   # 回执：持锁者（提示甲、锁之后）、对手（提示乙）、旧进程（提示甲、一小时前）；w4 的两个写者没有回执
                    transcript_file.write("\n".join([receipt(rival_id, prompt_other), receipt(zombie_id, prompt_one, seconds_ago=3600), receipt(holder_id, prompt_one)]) + "\n")
                def lock_hook_input(agent_id, relative):
                    return {"tool_name": "Write", "agent_id": agent_id, "agent_type": "implementation-writer", "transcript_path": lock_transcript,
                            "tool_input": {"file_path": os.path.join(lock_root, relative), "content": "x"}}
                def lock_case(label, agent_id, relative, want):
                    return (label, want, decide(lock_hook_input(agent_id, relative), work, table, library)[0])
                def refusal_names_holder():   # 持锁者认领之后才有「持锁者是谁」可写：在 cases 列表里按次序求值
                    message = decide(lock_hook_input(rival_id, "w1/d.md"), work, table, library)[1] or ""
                    return all(fragment in message for fragment in (holder_id, rival_id, "→ 怎么办", "写者锁已判："))
                def bad_lock_refusal_names_lock():
                    message = decide(lock_hook_input(rival_id, "w5/z.md"), work, table, library)[1] or ""
                    return all(fragment in message for fragment in (os.path.join(lock_root, "w5", writer_lock.LOCK_NAME), "判不了谁持锁", "→ 怎么办", "→ 主 agent", "rm "))
                def foreign_message(agent_id, relative):   # 持锁者还没认领的目录里被拒的说明（在 cases 列表里按次序求值，w6、w9 没人认领过）
                    return decide(lock_hook_input(agent_id, relative), work, table, library)[1] or ""
                def foreign_names_expected_holder():
                    rival_message, zombie_message = foreign_message(rival_id, "w6/a.md"), foreign_message(zombie_id, "w6/b.md")
                    return (f"子 agent {holder_id}" in rival_message and "还没有认领文件" in rival_message and rival_id in rival_message and "→ 怎么办" in rival_message
                            and f"子 agent {holder_id}" in zombie_message and zombie_message.count(zombie_id) == 1)
                def foreign_unfound_names_registration():
                    message = foreign_message(rival_id, "w9/a.md")
                    return "主会话记录里没找到它的回执" in message and writer_lock.prompt_digest(prompt_four)[:writer_lock.PROMPT_DIGEST_SHOWN] in message and "→ 怎么办" in message
                def broken_claim_refusal_names_claim():
                    shown = []
                    for name in ("w22", "w23"):
                        broken_scratch = os.path.join(lock_root, name)
                        message = decide(lock_hook_input(holder_id, f"{name}/m.md"), work, table, library)[1] or ""
                        claim_file = writer_lock.claim_path(broken_scratch, writer_lock.lock_state(broken_scratch)[1])
                        shown.append(all(fragment in message for fragment in (claim_file, "判不了谁持锁", "→ 怎么办", "→ 主 agent", "rm ")))
                    return all(shown)
                def claim_owner(name):   # 这个目录的锁此刻的认领：认领者的 agent_id，没有认领交 "absent"，判不了交 "unreadable"
                    scratch = os.path.join(lock_root, name)
                    state, claim = writer_lock.claim_state(scratch, writer_lock.lock_state(scratch)[1])
                    return claim["agent_id"] if state == "ok" else state
                def append_receipt(agent_id, prompt_text):
                    with open(lock_transcript, "a", encoding="utf-8") as transcript_file:
                        transcript_file.write(receipt(agent_id, prompt_text) + "\n")
                def receipt_pending_cases():
                    """w4 的锁登记提示丙：乙（first_id）一直没有回执，甲（second_id）的回执晚落盘，same_prompt_id 是同一份提示的另一次派发。
                    回执读不到的写放行、不认领、记一处没判全（开关 missing-receipt-claims 下退回先到先得，前四格红）；按次序求值。"""
                    import contextlib, io
                    def pending_case(label, agent_id, relative):
                        del UNJUDGED[:]
                        with contextlib.redirect_stderr(io.StringIO()):
                            code = decide(lock_hook_input(agent_id, relative), work, table, library)[0]
                        notes = list(UNJUDGED)
                        del UNJUDGED[:]
                        return (label, (0, "absent", True), (code, claim_owner("w4"), any("回执还没落盘，没认领" in note for note in notes)))
                    same_prompt_id = "a0000000000000ff6"
                    found = [pending_case("写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全", first_id, "w4/x.md"),
                             pending_case("写者锁:真持锁者回执还没落盘时第一次写也放行、不认领", second_id, "w4/y.md")]
                    append_receipt(second_id, prompt_three)
                    found.append(("写者锁:回执落盘之后持锁者写拿到锁", (0, second_id), (decide(lock_hook_input(second_id, "w4/z.md"), work, table, library)[0], claim_owner("w4"))))
                    found.append(lock_case("写者锁:持锁者认领之后，回执一直没落盘的乙写被拒", first_id, "w4/u.md", 2))
                    append_receipt(same_prompt_id, prompt_three)
                    found.append(lock_case("写者锁:认领之后同一份提示的另一次派发写被拒", same_prompt_id, "w4/v.md", 2))
                    found.append(lock_case("写者锁:认领之后持锁者续写放行", second_id, "w4/w.md", 0))
                    return found
                cases += [
                    lock_case("写者锁:提示对不上的子 agent 不能认领", rival_id, "w1/a.md", 2),
                    lock_case("写者锁:同一份提示但派发早于这把锁的旧进程不能认领", zombie_id, "w1/a.md", 2),
                    lock_case("写者锁:持锁者第一次写放行", holder_id, "w1/a.md", 0),
                    lock_case("写者锁:持锁者续写放行", holder_id, "w1/b.md", 0),
                    lock_case("写者锁:持锁者写子目录放行", holder_id, "w1/sub/c.md", 0),
                    lock_case("写者锁:第二个写者被拒", rival_id, "w1/c.md", 2),
                    lock_case("写者锁:第二个写者写子目录也被拒", rival_id, "w1/sub/deeper/e.md", 2),
                    ("写者锁:拒绝说明点名持锁者与出路", True, refusal_names_holder()),
                    *receipt_pending_cases(),
                    lock_case("写者锁:主 agent 不判", None, "w1/f.md", 0),
                    lock_case("写者锁:没有锁的目录不判", rival_id, "w2/x.md", 0),
                    lock_case("写者锁:锁文件不是 JSON 拒", rival_id, "w5/x.md", 2),
                    lock_case("写者锁:锁缺 nonce 拒", holder_id, "w13/x.md", 2),
                    lock_case("写者锁:锁缺 dispatched_at 拒", holder_id, "w14/x.md", 2),
                    lock_case("写者锁:锁缺 prompt_sha256 拒", holder_id, "w15/x.md", 2),
                    lock_case("写者锁:锁的位置是目录拒", holder_id, "w16/x.md", 2),
                    lock_case("写者锁:主 agent 写坏锁的目录不判", None, "w5/y.md", 0),
                    ("写者锁:判不了的锁的拒绝点名那份锁与出路", True, bad_lock_refusal_names_lock()),
                    lock_case("写者锁:认领文件不是 JSON，真持锁者写也拒", holder_id, "w22/x.md", 2),
                    lock_case("写者锁:认领文件缺 agent_id，真持锁者写也拒", holder_id, "w23/x.md", 2),
                    ("写者锁:认领文件判不了的拒绝点名那份认领文件与出路", True, broken_claim_refusal_names_claim()),
                    ("写者锁:认领文件判不了的不另建一份、不改它", ("unreadable", "unreadable"), (claim_owner("w22"), claim_owner("w23"))),
                    lock_case("写者锁:草稿根自己下面直接写不判", rival_id, "result.md", 0),
                    ("写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id", True, foreign_names_expected_holder()),
                    ("写者锁:回执对不上又找不到持锁者的回执时，拒绝写锁登记的提示 sha256 前几位", True, foreign_unfound_names_registration()),
                    # 嵌套的草稿目录：w7 的锁是甲的、w7/sub 的锁是乙的，只认最近一层（writer_lock 文件头「嵌套的草稿目录」；开关 holder-ignored、launch-ignored 下第一格红）
                    lock_case("写者锁:父目录锁的持有者写子目录另派给别人持锁的部分被拒", holder_id, "w7/sub/x.md", 2),
                    lock_case("写者锁:子目录锁的持有者写自己的子目录放行", rival_id, "w7/sub/z.md", 0),
                    lock_case("写者锁:子目录锁的持有者写父目录仍被父目录锁拦", rival_id, "w7/y.md", 2),
                    lock_case("写者锁:不持有任何一层锁的第三者写嵌套的子目录被拦", first_id, "w7/sub/q.md", 2),
                    lock_case("写者锁:父目录锁的持有者写父目录放行（认领父目录）", holder_id, "w7/h.md", 0),
                    lock_case("写者锁:父目录认领之后，认领者写别人持锁的子目录仍被拒", holder_id, "w7/sub/x2.md", 2),
                    lock_case("写者锁:父目录认领之后，子目录锁的持有者写自己的子目录仍放行", rival_id, "w7/sub/z2.md", 0),
                    ("写者锁:草稿根之外不判", 0, decide({"tool_name": "Write", "agent_id": rival_id, "agent_type": "implementation-writer", "transcript_path": lock_transcript,
                                                   "tool_input": {"file_path": f"{work}/crates/b.rs", "content": "x"}}, work, table, library)[0]),
                ]
                # 接手换锁（派发闸用 replace_lock）失败不留临时文件：锁内容写不进 JSON（json.dump 写到一半抛 TypeError）、锁的位置是个非空目录（os.replace 抛 OSError）
                # 两种失败各一格，另有一格成功的对照；LIB_WRITER_LOCK_BREAK=replace-temp-kept 下前两格红
                def replace_lock_outcome(name, obstacle):
                    scratch = os.path.join(lock_root, name)
                    os.makedirs(scratch)
                    if obstacle == "directory-in-the-way":
                        os.makedirs(os.path.join(scratch, writer_lock.LOCK_NAME, "keep"))
                    new = writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    if obstacle == "unserializable":
                        new["unserializable"] = object()
                    try:
                        writer_lock.replace_lock(scratch, new)
                        raised = False
                    except (TypeError, OSError):
                        raised = True
                    return raised, sorted(item for item in os.listdir(scratch) if item.endswith(".tmp"))
                cases += [
                    ("写者锁:换锁时锁内容写不进 JSON，抛异常且不留临时文件", (True, []), replace_lock_outcome("w10", "unserializable")),
                    ("写者锁:换锁时锁的位置被目录占着，抛异常且不留临时文件", (True, []), replace_lock_outcome("w11", "directory-in-the-way")),
                    ("写者锁:换锁成功不抛异常也不留临时文件", (False, []), replace_lock_outcome("w12", "none")),
                ]
                # 排他建锁（派发闸用 create_lock）、建认领（try_claim）共用 write_exclusively：内容写不进 JSON（json.dump 写到一半抛 TypeError）时抛异常、
                # 不留临时文件，锁与认领也不落下；LIB_WRITER_LOCK_BREAK=exclusive-temp-kept 下这两格红
                def exclusive_write_outcome(name, which):
                    scratch = os.path.join(lock_root, name)
                    os.makedirs(scratch)
                    new = writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    try:
                        if which == "lock":
                            new["unserializable"] = object()
                            writer_lock.create_lock(scratch, new)
                        else:
                            writer_lock.create_lock(scratch, new)
                            writer_lock.try_claim(scratch, new, object())
                        raised = False
                    except TypeError:
                        raised = True
                    landed = writer_lock.lock_path(scratch) if which == "lock" else writer_lock.claim_path(scratch, new)
                    return raised, sorted(item for item in os.listdir(scratch) if item.endswith(writer_lock.TEMPORARY_SUFFIX)), os.path.lexists(landed)
                cases += [
                    ("写者锁:排他建锁时锁内容写不进 JSON，抛异常、不留临时文件也不留锁", (True, [], False), exclusive_write_outcome("w19", "lock")),
                    ("写者锁:认领时认领内容写不进 JSON，抛异常、不留临时文件也不留认领", (True, [], False), exclusive_write_outcome("w20", "claim")),
                ]
                # 两次接手交错：A 换上锁之后、清理旧认领之前，B 换上自己的锁、B 的子 agent 认领（replace_lock 的 after_replace 造这一刻），A 再清理。
                # 改后 A 只删换锁之前列过的认领：B 的认领还在、第三个子 agent 认领不成，别的进程正在写的认领临时文件也还在；
                # LIB_WRITER_LOCK_BREAK=takeover-sweeps-new-claims（改前：换上之后删掉所有 nonce 不同的认领）下这两格红：
                #     「✗ 自检：写者锁:两次接手交错（A 换锁、B 换锁并认领、A 清理）之后 B 的认领还在，第三个认领不成，旧认领删掉」
                #     「✗ 自检：写者锁:接手不删别的进程正在写的认领临时文件」

                def interleaved_takeover_outcome(name):
                    scratch = os.path.join(lock_root, name)
                    old = writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    writer_lock.create_lock(scratch, old)
                    writer_lock.try_claim(scratch, old, holder_id)   # 换锁之前就在的旧认领
                    in_flight = os.path.join(scratch, f"{writer_lock.CLAIM_PREFIX}0123456789abcdef.4242.abcd{writer_lock.TEMPORARY_SUFFIX}")
                    open(in_flight, "w", encoding="utf-8").close()   # 别的进程正在写、还没链上的认领临时文件
                    lock_a = writer_lock.new_lock("self-test", prompt_other, "implementation-writer", lock_transcript)
                    lock_b = writer_lock.new_lock("self-test", prompt_three, "implementation-writer", lock_transcript)
                    def b_takes_over_and_claims():
                        writer_lock.replace_lock(scratch, lock_b)
                        writer_lock.try_claim(scratch, lock_b, rival_id)
                    writer_lock.replace_lock(scratch, lock_a, after_replace=b_takes_over_and_claims)
                    state, current = writer_lock.lock_state(scratch)
                    survived = claim_owner(name)
                    third = writer_lock.try_claim(scratch, lock_b, zombie_id)
                    return ((state == "ok" and current["nonce"] == lock_b["nonce"], survived, third, os.path.exists(writer_lock.claim_path(scratch, old))),
                            os.path.exists(in_flight))
                takeover_claims, in_flight_kept = interleaved_takeover_outcome("w21")
                cases += [
                    ("写者锁:两次接手交错（A 换锁、B 换锁并认领、A 清理）之后 B 的认领还在，第三个认领不成，旧认领删掉", (True, rival_id, rival_id, False), takeover_claims),
                    ("写者锁:接手不删别的进程正在写的认领临时文件", True, in_flight_kept),
                ]
                # 判不了照常放行而记一处没判全：读不到库、判法自己出错（LIB_WRITER_LOCK_BREAK 设成认不出的值，find_lock_directory 抛 ValueError）；
                # 开关 WRITE_GUARD_BREAK=unjudged-not-recorded 下这两格与下面 stdin 那一格红
                def unjudged_after(prepare, restore, relative):
                    import contextlib, io
                    del UNJUDGED[:]
                    prepare()
                    try:
                        with contextlib.redirect_stderr(io.StringIO()):
                            code = decide(lock_hook_input(rival_id, relative), work, table, library)[0]
                    finally:
                        restore()
                    notes = list(UNJUDGED)
                    del UNJUDGED[:]
                    return code, notes
                saved_library, saved_lock_break = writer_lock, os.environ.get("LIB_WRITER_LOCK_BREAK")
                def restore_lock_break():
                    if saved_lock_break is None:
                        os.environ.pop("LIB_WRITER_LOCK_BREAK", None)
                    else:
                        os.environ["LIB_WRITER_LOCK_BREAK"] = saved_lock_break
                code, notes = unjudged_after(lambda: globals().__setitem__("writer_lock", None), lambda: globals().__setitem__("writer_lock", saved_library), "w1/u.md")
                cases.append(("写者锁:读不到库时记一处没判全", (0, True), (code, any("读不到 lib_writer_lock.py" in note for note in notes))))
                code, notes = unjudged_after(lambda: os.environ.__setitem__("LIB_WRITER_LOCK_BREAK", "selftest-unknown-mode"), restore_lock_break, "w2/u.md")
                cases.append(("写者锁:自己出错时记一处没判全", (0, True), (code, any("写者锁这一道自己出错" in note for note in notes))))
                lock_environment = dict(environment, SINGLEFS_WORK_ROOT=lock_root)
                def run_lock_entry(agent_id, relative):
                    return subprocess.run(["bash", script], input=json.dumps(lock_hook_input(agent_id, relative)), capture_output=True, text=True, env=lock_environment)
                second_entry = run_lock_entry(rival_id, "w1/g.md")
                cases.append(("stdin:第二个写者被拒（退出码 2）", 2, second_entry.returncode))
                cases.append(("stdin:第二个写者的拒绝点名持锁者、锁的目录与出路", True,
                              all(fragment in second_entry.stderr for fragment in (holder_id, os.path.join(lock_root, "w1"), "→ 怎么办"))))
                cases.append(("stdin:持锁者放行", 0, run_lock_entry(holder_id, "w1/h.md").returncode))
                foreign_entry = run_lock_entry(rival_id, "w6/h.md")
                cases.append(("stdin:回执对不上的拒绝（退出码 2）", 2, foreign_entry.returncode))
                cases.append(("stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路", True,
                              all(fragment in foreign_entry.stderr for fragment in (holder_id, os.path.join(lock_root, "w6"), "→ 怎么办"))))
                cases.append(("stdin:父目录锁的持有者写别人持锁的子目录被拒（退出码 2）", 2, run_lock_entry(holder_id, "w7/sub/h2.md").returncode))
                bad_lock_entry = run_lock_entry(rival_id, "w5/h.md")
                cases.append(("stdin:坏锁拒绝（退出码 2）", 2, bad_lock_entry.returncode))
                cases.append(("stdin:坏锁的拒绝点名那份锁与出路", True,
                              all(fragment in bad_lock_entry.stderr for fragment in (os.path.join(lock_root, "w5", writer_lock.LOCK_NAME), "→ 怎么办", "→ 主 agent"))))
                broken_claim_entry = run_lock_entry(holder_id, "w22/h.md")
                broken_claim_file = writer_lock.claim_path(os.path.join(lock_root, "w22"), writer_lock.lock_state(os.path.join(lock_root, "w22"))[1])
                cases.append(("stdin:坏认领文件拒绝并点名那份认领文件（退出码 2）", (2, True),
                              (broken_claim_entry.returncode, all(fragment in broken_claim_entry.stderr for fragment in (broken_claim_file, "→ 怎么办", "→ 主 agent")))))
                # 回执还没落盘：经钩子入口写，放行、不认领、stderr 一行、检出一条（结局只记）；开关 missing-receipt-claims 下红
                make_lock("w18", prompt_three)
                pending_detections = os.path.join(lock_root, "pending-detections.jsonl")
                pending_entry = subprocess.run(["bash", script], input=json.dumps(lock_hook_input("a0000000000000ab9", "w18/h.md")), capture_output=True, text=True,
                                               env=dict(lock_environment, AGENT_HOOK_DETECTIONS=pending_detections))
                pending_records = [json.loads(line) for line in open(pending_detections, encoding="utf-8")] if os.path.exists(pending_detections) else []
                cases.append(("stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）", (0, "absent", True, 1, "只记", True),
                              (pending_entry.returncode, claim_owner("w18"), "回执还没落盘，没认领" in pending_entry.stderr, len(pending_records),
                               pending_records[-1].get("outcome") if pending_records else None,
                               bool(pending_records) and any(str(finding).startswith("写闸没判全：写者锁这一道") and "回执还没落盘，没认领" in str(finding)
                                                             for finding in pending_records[-1].get("findings") or []))))
                lock_error_detections = os.path.join(lock_root, "lock-error-detections.jsonl")
                lock_error_entry = subprocess.run(["bash", script], input=json.dumps(lock_hook_input(rival_id, "w2/h.md")), capture_output=True, text=True,
                                                  env=dict(lock_environment, LIB_WRITER_LOCK_BREAK="selftest-unknown-mode", AGENT_HOOK_DETECTIONS=lock_error_detections))
                lock_error_records = [json.loads(line) for line in open(lock_error_detections, encoding="utf-8")] if os.path.exists(lock_error_detections) else []
                cases.append(("stdin:写者锁自己出错放行并记一条检出（结局只记）", (0, 1, "只记", True),
                              (lock_error_entry.returncode, len(lock_error_records), lock_error_records[-1].get("outcome") if lock_error_records else None,
                               bool(lock_error_records) and any(str(finding).startswith("写闸没判全：写者锁这一道自己出错") for finding in lock_error_records[-1].get("findings") or []))))
        finally:
            if work_root_before is None:
                os.environ.pop("SINGLEFS_WORK_ROOT", None)
            else:
                os.environ["SINGLEFS_WORK_ROOT"] = work_root_before
            shutil.rmtree(lock_root, ignore_errors=True)
        os.unlink(outside.name)
    finally:
        shutil.rmtree(work)
        shutil.rmtree(outside_directory, ignore_errors=True)
    # 第六道判不了照常放行而记一处没判全：Edit 一份在而读不了（chmod 成 000）的脚本，改完之后的内容算不出；开关 WRITE_GUARD_BREAK=syntax-unreadable-unrecorded 下这一格红
    import contextlib, io
    locked_root = tempfile.mkdtemp(prefix="write-guard-locked-")
    try:
        os.makedirs(os.path.join(locked_root, "scripts"))
        locked_script = os.path.join(locked_root, "scripts", "locked.sh")
        with open(locked_script, "w", encoding="utf-8") as handle:
            handle.write("echo a\n")
        os.chmod(locked_script, 0)
        del UNJUDGED[:]
        with contextlib.redirect_stderr(io.StringIO()):
            code = decide_shell_syntax({"tool_name": "Edit", "tool_input": {"file_path": locked_script, "old_string": "echo a", "new_string": "echo b"}}, locked_root)[0]
        notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("语法:改一份读不了的脚本照常放行、记一处没判全", (0, True), (code, any("读不了" in note and "语法这一次没判" in note for note in notes))))
    finally:
        for directory, _, names in os.walk(locked_root):
            for name in names:
                os.chmod(os.path.join(directory, name), 0o600)
        shutil.rmtree(locked_root, ignore_errors=True)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide_overwrite() / decide_compile_first() / decide_trailing_blank() / decide_shell_syntax() / decide_scope() / decide_prime_marks() / decide_clock_times() / decide_session_names() / decide_writer_lock() 与入口；"
              "WRITE_GUARD_DISABLE_WRITER_LOCK / LIB_WRITER_LOCK_BREAK / WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_COMPILE_FIRST / WRITE_GUARD_DISABLE_SYNTAX / WRITE_GUARD_DISABLE_SCOPE / WRITE_GUARD_DISABLE_CLOCK_TIMES / WRITE_GUARD_DISABLE_SESSION_NAMES、"
              "WRITE_GUARD_BREAK=trailing-blank-ignored / scope-root-unanchored / builtin-agents-unscoped / python-syntax-subprocess / library-reloaded-each-time / landing-lexical-only / unparsable-input-silent / unjudged-not-recorded 或 SINGLEFS_WORK_ROOT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种情形）：钩子、门禁、项目脚本、研究脚本目录下写进语法错误的 .sh（Write 与 Edit）与 .py 拒绝、语法对的与别处的放行；"
          "未跟踪的已有文件整份覆盖拒绝，已跟踪 / 不存在 / 仓外 / Edit 放行；主 agent 放行、没有项目定义的 agent（内置类型、定义已删的旧实例）只许写工作根（仓里与仓外别处拒）、范围内放行、"
          "工作根与过渡期旧根下的报告目录放行、工作根同名前缀的兄弟目录拒绝，范围外（Edit、Write、MultiEdit）与 .. 绕路与未登记的项目 agent 拒绝，按解开符号链接之后的落点判（范围内的链接指到范围外、经它写不存在的子目录、悬空链接、指到仓里范围外的拒，指到范围内与范围外指回范围内的放行；覆盖与先编后换经链接同样认得出）；experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，"
          "implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
          "只在 old_string 里有、ASCII 单引号放行，共用字符集读不到拒绝；写进仓里的内容带钟点（日期后、时区词旁、x 通配、日期或时区词后的「N 点 / N 时」与时段词、时段词后的钟点、光秃的「N 点前后」、"
          "不挨日期时区的时分与区间、ISO 与 diff 头 stat 的机器时间、ISO 紧凑时间戳（日期连写或带连字符加连写的钟点，含路径里的运行目录名，Write、Edit、MultiEdit 各一例）、光秃的时区词、英文十二小时制，"
          f"md 里与没写理由的放行标记不认，主 agent 与子 agent 一样）拒绝并点名形态，8 位数字后跟 T 但不是时间戳的 {len(compact_lookalike_cases)} 例（月份与钟点越界、年不是 20 开头、钟点只有三位、前面挨着字母、小写 t）与只写日期、仓外、.py 里带理由的放行标记放行，"
          f"只写日期、序号与个数后的「点」、不挨日期时区的时段词、标识符里的时区词、以 am / pm 起头的英文词、{len(context_cases)} 类长得像钟点的冒号对、"
          f".sh / .py / .rs 里带理由的放行标记 {len(allow_cases)} 例、仓外、排除的样本、hook 自检与 c510 提示、只在 old_string 里有放行，判据读不到拒绝；写进仓里的会话号（标签式、当主语、上游三语仓、长十六进制）拒绝，crate 名、上游仓目录名、临时目录名与 .sh 里带理由的放行标记放行；"
          "目标不是 .md / .mdx 时 Edit 与 MultiEdit 的 old_string 以空格或制表符收尾（.sh、.txt、.markdown，仓里仓外、主 agent 与子 agent 一样）拒绝并给出收窄框选与 replace-once.py 两条出路，"
          ".md / .mdx / .MD 目标、不以空白收尾、只在中间行或换行前有行尾空白、只有 new_string 以空白收尾与 Write 放行；"
          "钟点与会话号逐处列出（行号与总数，Edit 折算成文件里的行号），几道都不成立的一次列全；拒绝都记进检出记录；输入不是合法的 JSON 对象照常放行、stderr 写诊断并记一条只记的检出（会话号读不到写「读不到」，按字面找得到的照抄）；"
          "草稿目录写者锁：提示对不上与派发早于锁的旧进程不能认领，持锁者写与续写（含子目录）放行，第二个写者（含子目录）拒绝并点名持锁者与出路，回执对不上的拒绝点名记录里找到的持锁者（找不到写锁登记的提示 sha256 前几位），"
          "嵌套的子目录另派给别人持锁时，父目录锁的持有者写那个子目录、子目录锁的持有者写父目录与不持有任何一层的第三者都按最近一层的锁拦，"
          "派发回执还没落盘的写（先写的非持锁者、真持锁者自己的第一次写）放行、不认领并记一处没判全（经入口时 stderr 一行、检出一条），回执落盘之后持锁者写才认领，认领之后没回执的与同一份提示的另一次派发被拒，"
          "锁文件在而判不了（不是 JSON、缺 nonce / dispatched_at / prompt_sha256、锁的位置是目录）的拒绝并点名那份锁与出路，认领文件在而判不了（不是 JSON、缺 agent_id）的真持锁者写也拒、点名那份认领文件与出路，"
          "主 agent、没有锁的目录、草稿根自己下面与草稿根之外不判，读不到库与判法自己出错照常放行并记一处没判全；接手换锁时锁内容写不进 JSON、锁的位置被目录占着都不留临时文件，"
          "排他建锁与建认领时内容写不进 JSON 抛异常、不留临时文件也不落下，两次接手交错之后后一次接手的认领还在、第三个认领不成、别的进程正在写的认领临时文件不删；"
          ".py 的语法在本进程里判、不起子进程，共用判据库一次写入只装一次")
    return 0

hook_dir = sys.argv[1]
if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
    sys.exit(selftest(hook_dir))
detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
raw = sys.stdin.buffer.read()
try:
    hook_input = json.loads(raw.decode("utf-8"))
    if not isinstance(hook_input, dict):
        raise ValueError(f"顶层是 {type(hook_input).__name__}，不是对象")
except ValueError as error:   # 判不了：照常放行，打诊断并记一条检出（弄坏开关 unparsable-input-silent 下静默放行，改前的样子）
    if os.environ.get("WRITE_GUARD_BREAK") != "unparsable-input-silent":
        identity, text = unparsable_input_entry(raw, f"{type(error).__name__}: {error}")
        note_unjudged(text)
        record_detection(identity, [text], detections_path, "只记")
    sys.exit(0)
project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
try:
    code, message, finding = decide(hook_input, project_root, os.path.join(hook_dir, "agent-write-scope.tsv"), notations_library_path(hook_dir))
except Exception as error:   # 判定自己抛异常：照常放行，打诊断并记检出
    note_unjudged(f"判定自己抛异常，这一次放行：{type(error).__name__}: {error}")
    code, message, finding = 0, None, None
if code:
    print(message, file=sys.stderr)
    record_detection(hook_input, finding, detections_path)
if UNJUDGED:
    record_detection(hook_input, UNJUDGED, detections_path, "只记")
sys.exit(code)
PY
