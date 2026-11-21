#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（Write、Edit、MultiEdit：.claude/settings.json 里注册在 matcher "Write|Edit|MultiEdit" 上）：写文件之前的十一道判定（一至五、七至十一写在下面，六是语法检查、写在 --selftest 那一段），合在一个 hook 里；拒绝的同时把它记进检出记录，交主 agent 看。
#
# 一、整份覆盖未跟踪文件（只管 Write）：仓里已存在、又没进 git 的文件，拒绝用 Write 整份覆盖。
#    为什么：几个会话同时在一个仓里干活。一个会话用 Write 新建 `research/prompts/e137-preregistration.md`，
#    另一个会话几分钟前刚写了同名的未提交文件，Write 回报「updated」就把它盖了——没进过 git，事后只能从对话记录里重放复原。
#    放行：目标不存在；目标已被 git 跟踪（盖了还能从 git 找回）；目标在仓外（scratchpad 之类）。
# 二、项目 subagent 的写范围（Write、Edit、MultiEdit）：项目定义的 subagent 写文件时，目标必须落在它登记的写范围里。
#    为什么：执行类 agent 越界写，不该等人翻 diff 才发现（records/2026-09-16-subagent拆分提案.md 第五节；走项目 settings hook）。
#    输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（`.claude/agents/<名>.md` 不在：内置类型、插件带来的、定义已删的旧实例）只许写工作根
#    （BUILTIN_AGENT_SCOPE，展开同表里的 $SINGLEFS_WORK_ROOT/** 那一行：只有工作根），仓里与别处（会话临时目录根 /tmp/claude-<uid> 在内）一律拒（只许写草稿目录；
#    records/2026-10-23-subagent拆分提案.md 第十七节 ①、第十九节 ④）；有项目定义而表里没有它的写范围，拒绝；
#    目标的落点命中它的任一模式才放行：落点规整 .. 并逐段解开符号链接，目标不存在时解到最深的已存在那一层再接上剩下的段（写范围里建一个指向范围外的链接、
#    经它写进去的照拒；与 bash-command-detector.sh ⑫ 同口径，认法在 lib_write_scope.py 的 landing_forms / landings_out_of_scope）。
#    覆盖、先编后换、语法、钟点与会话号、写者锁几道判的也是这个落点（landing_target）：盖掉的、改到的、落进哪个草稿目录的都是解开链接之后那一份。表在同目录的 agent-write-scope.tsv，读表与判路径落没落进范围的函数在同目录 lib_write_scope.py。这道闸拦不住 Bash 里的写：Bash 里往本仓写、落在表里它那几行之外的，由 bash-command-detector.sh 的 ⑫ 照同一张表、调同一份库判（只判表里有行的 agent）；定义的「没做什么」照写。
#    表里以 $SINGLEFS_WORK_ROOT/ 起头的模式由同目录的 lib_work_root.py 展开成工作根（默认 ~/.local/share/singlefs/work）那一条，
#    根后面的 / 不省（同名前缀的兄弟目录不放行）；Claude Code 的会话临时目录根 /tmp/claude-<uid>（各会话的 scratchpad 在里面）不展开，子 agent 写进去照越界拒。
#    读不到那份库时，这样的模式按拒判，拒绝信息点名那份库。
# 三、撇号类角标（Write、Edit、MultiEdit）：Write 的 content、Edit 的 new_string、MultiEdit 每一处的 new_string 里
#    出现撇号类字符之一就拒绝；只在 old_string 里出现（删掉它的改动）放行，ASCII 单引号不判。主 agent 与所有子 agent 一样判，目标在不在仓里都判。
#    为什么：门禁 doc-text 只在收尾跑整轮门禁时判，写的当下拦不住，这类名字会被写进重跑登记、再被抄进源码。
#    字符集与 doc-text 同一份：../gate.d/lib-forbidden-notations.py 的 PRIME_MARKS，不在这里另抄；读不到就拒绝（不静默放行）。
# 四、先编后换（Write、Edit、MultiEdit）：experiment-runner（名单是 COMPILE_FIRST_AGENTS）写主工作区 crates/ 下的 .rs（改已有的、新建的都算）一律拒绝，
#    出路是在草稿目录的副本里改，把 scripts/hosts/compile-then-swap.py 那一条（`task-server/task submit -- python3 scripts/hosts/compile-then-swap.py …`）写进报告「要主 agent 跑的命令」一节，
#    由主 agent 交服务编过再整份换进来（子 agent 不往统一任务服务交任务）。排在写范围那一道之前判，拒绝信息给的是这条出路。
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
# 七、会话号与批次代号（Write、Edit、MultiEdit）：ListAgents 列的会话名全名（目录名后面跟「-」与十六进制）写进仓里就拒绝，排除前缀、放行标记与仓外不判同五；
#    判据是同一份库里的 session-names 那一形态，与门禁 doc-text 的 session-names 格同一份。改过标题的会话认不出。不带括号的两位短号（「87」）不算会话号。
#    同一道另判批次代号（拒绝时另列一道「写了批次代号」）：拿来当名字用的批次与运行代号（「24c-1」「22a」「批次 24c」「第十五批乙二」「批 10 甲」），
#    判据是同一份库里的 batch-codes 那一形态（四条判据行、上下文规则、排除前缀都写在库的文档串里），库按后缀只判 .md、.tsv、.txt，不认放行标记，仓外不判；
#    出路用那一形态登记的 remedy（写成那一批做了什么事）。Bash 里往仓里写的字面内容归 bash-command-detector.sh 的 ⑰，读同一份库。
#    同一道再判任务号、会话短号与批号（拒绝时另列一道「写了任务号、会话短号或批号」）：任务服务的任务号（「任务 <号>」「同 <号>、<号>」「task=<号>」，反引号与围栏里的照判）、
#    括号里当会话名用的两位短号（挨着会话词的、括号里跟会话词的；不挨会话词的数字字母短号只在 records/ 下判）、批号（「批 <数字>」「第 <数字> 批」，自己会话的也算）、
#    拿「<做什么>那一路」的描述指别的会话或子 agent（「那一路会话」「那一路的 agent」；按出场次序的 mainagent(A)、subagent(A) 不判），
#    判据是同一份库里的 task-and-session-numbers 那一形态（四条判据行、按路径的上下文规则、围栏代码块的放过都写在库的文档串里）：只判 .md、.tsv、.txt，不认放行标记，仓外不判；
#    判决（research/prompts/*-main-verification.md）里行首的「A/B 读数：任务 <号>」一行（值里至多两个、引用块不算）、records/ 下以「长跑拆分：」起头的行照写任务号，
#    records/ 下表格里表头是「批次」的那一列照写批号。
#    这一判不只看新写的字：写前（现文件，不在是空串）、写后（Write 的 content，Edit 与 MultiEdit 施加到现文件之后的整份，edited_content）两份整文交库的 new_violation_hits
#    （library_hits_before_and_after），写后多出来的违例才拒（按判据行与命中的串整份计数相抵）：文件里已有的旧号不挡这一次写，只改了那一行别处的也不挡，同一行新加的号照拒；这一次写删了、加了或改坏一行围栏，让后面已有的批号出了围栏的，
#    改了表头让批号离开「批次」那一列的，Edit replace_all 一处在围栏里一处在外的，都照拒。拒绝说明里的行号是写后文件的行号。
#    出路用那一形态登记的 remedy（改写成交给服务的那条命令、结局行的 type= 与 exit= 那一段、日志里那一行的内容而不是日志路径、谁在改哪个文件、那一批做了什么）。
# 八、old_string 以空白收尾（Edit、MultiEdit）：目标不是 .md / .mdx（后缀不分大小写）时，Edit 的 old_string、MultiEdit 每一处的 old_string
#    以空格或制表符收尾就拒绝；主 agent 与所有子 agent 一样判，目标在不在仓里都判。.md / .mdx 目标与 Write 不判。
#    为什么：Edit 在 .md / .mdx 以外的目标上先去掉 new_string 每一行行尾的空格与制表符，钩子拿到的已是去过的 new_string，old_string 原样。
#    old_string 收尾的空白在文件里照样命中、被换掉，new_string 里对应的那一个却没了，紧跟着的字就粘上来。
#    钩子收到的输入落盘核过：.txt 目标 old_string、new_string 都以「declared = 」收尾，
#    钩子收到的 old_string 带着那个空格、new_string 没有，落盘成「declared =foo」，
#    例如把 doc-decisions.sh 的 `python3 - "$DEC"` 写成 `python3 -"$DEC"`（records/2026-10-01-门禁注释精简.md）。
#    看 old_string 而不看 new_string：new_string 收尾的空白到不了钩子，判它一次都拦不到；old_string 收尾的空白钩子看得到，粘连只在它那里发生。
#    .md / .mdx 不判：同一次核查里 .md、.mdx、.MD 目标的 new_string 行尾空白原样到了钩子、原样落盘，在那里以空白收尾的框选写得对。
#    后缀只认这两个：拿 Edit 在草稿目录里各改一份 .md、.MD、.markdown、.mdown、.mkd、.txt（new_string 头一行以两个空格收尾），
#    落盘只有 .md、.MD 留着那两个空格，.markdown、.mdown、.mkd 与 .txt 一样去掉了，所以它们照判、不并进 TRAILING_BLANK_EXEMPT_SUFFIXES。
#    不判 old_string 中间行的行尾空白：new_string 那一行去掉的空白后面是换行，不会把两个词粘起来。
#    出路：把框选范围收到不以空白结尾（多框到下一个非空白字符，或止于行尾）；确实要删行尾空白的，用 Bash 跑 scripts/kb/replace-once.py。
#
# 九、草稿目录写者锁（Write、Edit、MultiEdit，只判带 agent_id 的子 agent 调用）：目标在草稿根（工作根）底下、它所在的目录或往上的某一层有 .writer-lock，
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
# 十、往现行账本追加节标题（Write、Edit、MultiEdit）：目标是拆分提案账本登记表（.claude/gate.d/split-proposal-ledgers.tsv，门禁 doc-process-records 的三格读同一张）里
#    current 那一行点名的账本，这一次写让「## <汉字节号>、」标题净增就拒绝：Edit 与 MultiEdit 是任一处的新串比旧串多出节标题，Write 是新内容比现有文件多出节标题。
#    主 agent 与所有子 agent 一样判。改节号、改标题文字（新旧串的节标题个数相同）、改正文、封存的账本与别的文件放行；节标题的写法与门禁 ledger-section-numbers 格认节的写法一致
#    （「## 」后接汉字数字再接「、」）。登记表读不到、current 不是恰好一行的，这一道没判、照常放行，记一处没判全。
#    为什么：几个会话同时往账本末尾追加，各自读文件、各自取号，节号会撞，到提交时门禁才判出重号（records/2026-10-29-subagent拆分提案.md 第一百五十六节）。
#    出路：scripts/kb/ledger-append-section.py，现取最大节号加一，一次写入索引表末行与末尾新节，写前复核 sha256，撞了拒。Bash 里的写归 bash-command-detector.sh 的 ⑯。
# 十一、工具员就地改主工作区里已存在的脚本（Write、Edit、MultiEdit）：agent_type 在 SWAP_ONLY_AGENTS（tooling-writer）里，目标解开符号链接之后落在主工作区
#    （CLAUDE_PROJECT_DIR）的 SWAP_ONLY_DIRECTORIES（scripts/、.claude/gate.d/、.claude/scripts/、task-server/、prune-server/、gpu-verification-server/）下、
#    不在 SWAP_ONLY_EXCLUDED（.claude/gate.d/fixtures/ 与两个服务的 target/）下，是已存在的普通文件，且是 .sh、.py 或带执行位的，就拒。
#    两个服务的 Rust 源码目录 SWAP_ONLY_SOURCE_DIRECTORIES（prune-server/src/、gpu-verification-server/src/）下已存在的普通文件，
#    与两个服务的构建清单 SWAP_ONLY_SOURCE_FILES（prune-server、gpu-verification-server 各自的 Cargo.toml、Cargo.lock）里已存在的那几份，不看后缀与执行位，一律拒，
#    出路另是一句：在仓副本里改、把改动存成补丁写进草稿目录，打补丁的命令交主 agent 一次打上（源码或依赖清单改到一半，主工作区里的服务编不过，带临时判定库服务的自证都取不到二进制）。
#    Bash 里往这几处已存在的文件写（追加、cp、定点替换工具这些）归 bash-command-detector.sh 的 ⑭，判的是同一组落点；弄坏开关 WRITE_GUARD_BREAK=swap-manifests-unseen 下构建清单那一半不拦。
#    脚本的出路分两种，判是哪一种读项目根下的 .claude/hooks/selftest-temporary-service.tsv（TEMPORARY_SERVICE_TABLE，不写死名单）：
#    脚本在表里登记成 required 的，换上不加 --verify（--verify 的命令在换上工具里起、拿不到任务服务给的编译目录与临时服务，退 2、回滚），自证换上之后另交一条，
#    这样换没有自动回滚，自证验不过要再出一份退回的规格；不在表里、或登记成 own-server 的，照旧加 --verify 给被换脚本的 --selftest。
#    表读不到时两种出路都写、记一处没判全。弄坏开关 WRITE_GUARD_BREAK=swap-remedy-always-verify 下一律只写加 --verify 的那一种。
#    门禁阶段（GATE_STAGE_DIRECTORY 这一层的 .sh，不含 lib/ 与 fixtures/ 下的、也不含这一层 lib- 起头的 .py 共用库）单走一支，不读登记表（表里只登记脚本与钩子的 --selftest，不收门禁阶段）：
#    不带 --verify 换上（--verify 只起一条命令、在换上工具里起，拿不到任务服务给的编译目录与临时服务，起不了样本档）；验证是这一道的样本档，由主 agent 在换上之后另交一条
#    （STAGE_SELFTEST_SCRIPT 的 --item <阶段文件名>，.claude/hooks/selftest-registry.tsv 里这一道那一行写了要包在临时 prune-server 底下起的，包上 --with-temporary-service）；
#    这样换没有自动回滚，样本档验不过要再出一份退回的规格。一份规格里门禁阶段与别的脚本都有，整份都不带 --verify，所以说明写成门禁阶段单放一份规格、别的脚本另放一份照它们各自的出路。
#    弄坏开关 WRITE_GUARD_BREAK=gate-stage-remedy-as-script 下门禁阶段与别的脚本走同一支。
#    目录与 scripts/gate/hook-swap.py 脚本规格收的那一组相同（它的 SCRIPT_SPEC_DIRECTORIES；它不收的 target/、src/ 在这里是 SWAP_ONLY_EXCLUDED 与 SWAP_ONLY_SOURCE_DIRECTORIES），
#    两边各写一份、改的时候一起改。
#    已有门禁的样本档目录：目标落在主工作区 SWAP_ONLY_FIXTURE_DIRECTORY（.claude/gate.d/fixtures/）下第一层目录 <门禁>/ 里，且 .claude/gate.d/<门禁> 是已存在的普通文件（那一道门禁已在），
#    不看是新建还是改已有、不看后缀与执行位，一律拒：样本先于门禁新版进主工作区，门禁新版还在草稿里，别的会话这段时间跑这一道的样本档会判错。
#    出路另是一句：样本的新版写在草稿目录，照仓里的相对路径，被改那几份样本此刻的样子拷进 <草稿目录>/fixtures-base/（只新建样本的建成空目录）、改后的与新建的放进 <草稿目录>/fixtures-new/，
#    在草稿目录里 `git diff --no-index --binary fixtures-base fixtures-new > fixtures.patch` 生成补丁，打补丁的两条命令（`git apply -p2 --check`、`git apply -p2`，在仓根起）
#    写进报告「要主 agent 跑的命令」一节，排在门禁新版经 hook-swap.py 换上那一条之后、样本档之前，由主 agent 打上（hook-swap.py 的脚本规格只收已存在的文件、不收 fixtures/）。
#    还没有门禁阶段的样本目录（新门禁的样本，与新门禁一起写）不拦。认样本档目录的判法在 lib_write_scope.py 的 existing_gate_of_fixture，
#    Bash 里往样本档目录写（cp、mv、重定向、tee、git apply）这道闸看不见，由 bash-command-detector.sh ⑭ 调同一份判法拒。
#    弄坏开关 WRITE_GUARD_BREAK=gate-fixtures-in-place-allowed 下这一半不判。
#    放行：新建文件（已有门禁的样本档目录里的除外）；不带执行位的数据与文档（.tsv、.md、.json、.txt 这类，两个服务的 src/ 下的与上面那四份构建清单除外）；
#    还没有门禁阶段的样本目录里的样本；两个服务的 target/ 下的文件；
#    草稿目录与仓副本里的文件（落在主工作区之外）；别的 agent 类型与主 agent（输入里没有 agent_type 也没有 agent_id 的调用）。
#    钩子（.claude/hooks/）不在这一道的目录里：钩子照旧只经 hook-swap.py 的钩子规格换上。
#    判不了的照「放行但必记检出」：子 agent 的调用认不出 agent 类型（带 agent_id 而 agent_type 缺、空或不是字符串）、目标 stat 失败（不是「不存在」）。
#    为什么：分几次 Edit 改一份脚本，中间态对别的会话可见，被拷进快照的任务跑到半截的版本（records/2026-11-17-subagent账本自上而下清欠.md 第二百三十一节）。
#    出路：新版整份写在草稿目录，把 hook-swap.py 的脚本规格（一件活改几份的写进同一份规格，用 scripts/gate/make-swap-spec.py 生成）与那条换上命令写进报告「要主 agent 跑的命令」一节，
#    由主 agent 一次换上；两个服务的 src/ 走仓副本与补丁。
#    Bash 里用定点替换工具分几次改同一份脚本，这道闸看不见。
# gate-similar: bash-command-detector.sh 它的 ⑦ 只看 Bash 里在同一个 inode 上改已存在的脚本（出路改名换上或定点替换），看不见 Write / Edit 工具的调用，也不分 agent 类型；这一道挂在 Write / Edit / MultiEdit 上，按 agent 类型与目录判，与写闸别的几道共用落点与检出记录
# 十一道判定合在这一个 hook 里：一至五、七至十一各占文件头一段，六写在 --selftest 那一行的说明里。
# 十一道都判完再报：不成立的几道一次列全，每道带自己的出路；钟点与会话号逐处列出（行号与总数）。只有一处例外：
# 先编后换那一道拒了，写范围那一道不再报（目标是同一个，先编后换的出路已经写了范围外的交回主 agent）。
# 它们拒的是一次写，不停任务和脚本；拒绝时退出码 2、stderr 交给做这次写的模型，同时往检出记录
# （默认工作根下的 agent-hook-detections.jsonl，lib_work_root.py 的 default_detections_path；环境变量 AGENT_HOOK_DETECTIONS 可改）追加一行，
# 主 agent 的看门狗（scripts/agents/agent-watch.py watch）读到就叫醒主 agent，由主 agent 判断怎么处理。
# 判不了的照「放行但必记检出」：输入不是合法的 JSON 对象、写者锁那一道读不到库或自己出错或派发回执还没落盘、bash -n 起不了或超时、第六道 Edit 的目标在而读不了、判定自己抛异常、落点解不开（写范围那一道照拒，别的几道按字面路径判），
# 照常退 0，stderr 写一行可定位的诊断（钩子名、哪一处、输入的前 200 字节），检出记录追加一行（findings 以「写闸没判全」起头，outcome 只记；
# 被拒的那一行 outcome 被拒）；输入读不出会话号的，session_id 写「读不到」。
# 这份源码里撇号类字符一律写成转义、lib-forbidden-notations.py 里用码位拼（门禁 doc-text 也扫它们）；改带转义的那几行用 Bash 里的 python，反斜杠拿 chr(92) 拼：
# Write 的 content 里写的反斜杠 u 转义落盘成了字符本身，这道闸也会拒这样的写。
#
#   write-guard.sh             # 从 stdin 读 hook 的 JSON
#   write-guard.sh --selftest  # 走一遍各道判定的放行与拒绝；WRITE_GUARD_DISABLE_OVERWRITE=1、WRITE_GUARD_DISABLE_COMPILE_FIRST=1、
#                              # WRITE_GUARD_DISABLE_SCOPE=1、WRITE_GUARD_DISABLE_CLOCK_TIMES=1、WRITE_GUARD_DISABLE_SESSION_NAMES=1、WRITE_GUARD_DISABLE_SYNTAX=1 或
#                              # WRITE_GUARD_BREAK=trailing-blank-ignored（第八道放行以空白收尾的 old_string）时自检必须判红；
#                              # WRITE_GUARD_BREAK=stop-at-first（decide() 判到第一道不成立就交回、后面几道不判）时「九道判完再报:覆盖、越界与钟点一次列全」一格红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only（库的 all_hits 只交第一处）时「钟点:两处都报（行号与总数）」「会话号:两处都报（行号与总数）」两格红；
#                              # 批次代号（七里的另一判）：WRITE_GUARD_BREAK=batch-codes-unjudged（这一判不判）与 LIB_FORBIDDEN_NOTATIONS_BREAK=batch-codes（库里整形态不判）时
#                              #   「批次代号:」起头的拒绝几格红；库的判据行键一次关一条时点名那一条的格红：batch-code-suffixed「批次代号:数字加字母加短横序号」「批次代号:.tsv 也判」、
#                              #   batch-code-lettered「批次代号:两位数字加字母当名字用」「批次代号:配不上对的反引号不开代码段」、batch-code-labeled「批次代号:批次词后面跟一位数字加字母」「批次代号:单位字母的批号挨着批字」、
#                              #   batch-code-stem「批次代号:第几批加天干」「批次代号:批加数字加天干」；LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=batch-codes 时「批次代号上下文:」起头的几格红；
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=batch-codes 时「批次代号:变异表放行」红；LIB_FORBIDDEN_NOTATIONS_BREAK=judged-suffixes-ignored 时
#                              #   「批次代号:.py 不判」「批次代号:.sh 不判」两格红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=batch-code-labeled-three-digits（库里挨着批次词的三位数字加字母也判）时「批次代号:批次词后面的三位数字加字母放行」「批次代号:三位数字加字母挨着批字放行」两格红；
#                              # LIB_FORBIDDEN_NOTATIONS_BREAK=code-span-backtick-parity（库里反引号只数前面的奇偶、不按代码段配对）时「批次代号上下文:双反引号代码段里的放行」「批次代号:配不上对的反引号不开代码段」两格红；
#                              # 任务号、会话短号与批号（七里的第三判）：WRITE_GUARD_BREAK=task-and-session-numbers-unjudged（这一判不判）与 LIB_FORBIDDEN_NOTATIONS_BREAK=task-and-session-numbers
#                              #   （库里整形态不判）时「任务号与批号:」起头的拒绝几格与「任务号与批号:派发闸收而值里另带别的写法的号的四条在判决里照拒、两臂与单臂放行」红；库的判据行键一次关一条时点名那一条的格红：
#                              #   task-numbers「任务号与批号:括号里的任务号」「任务号与批号:同字后面并列的裸号」「任务号与批号:任务号冒号加号」「任务号与批号:.tsv 也判」「任务号与批号:Edit 写进的任务号」
#                              #     「任务号与批号:任务与号之间夹全角括号、冒号与井号」「任务号与批号:斜杠与短横连着的并列号」「任务号与批号:同字与与字后面并列的号」
#                              #     「任务号与批号:单个同字后面的号」「任务号与批号:任务号后接英文冒号」「任务号与批号:任务号后面跟条件与个别」「任务号与批号:日志文件名与运行目录名里的号」
#                              #     「任务号与批号:英文 task 的几种写法」「任务号与批号:结局行原文里的 task=」「任务号与批号:反引号里 task 子命令加号」「任务号与批号:反引号里的任务号」「任务号与批号:围栏代码块里的任务号」
#                              #     「任务号与批号:A/B 读数不在行首」「任务号与批号:A/B 读数一行在记录里」「任务号与批号:A/B 读数一行里同字并列的号」「任务号与批号:A/B 读数一行装三个号」
#                              #     「任务号与批号:引用块里的 A/B 读数」「任务号与批号:长跑拆分一行在 kb 里」「任务号与批号:旧号那一行新加第二个号照判」
#                              #     「任务号与批号:派发闸收而值里另带别的写法的号的四条在判决里照拒、两臂与单臂放行」
#                              #     「任务号与批号:任务编号与号任务」「任务号与批号:列表里的引用块里的 A/B 读数」、
#                              #   session-short-codes「任务号与批号:括号里数字加字母的会话短号」「任务号与批号:挨着会话词的纯数字短号」「任务号与批号:并列的会话短号」「任务号与批号:括号里短号后面跟会话词」
#                              #     「任务号与批号:括号里短号后面跟范围与判过」「任务号与批号:判决里括号里的会话短号」
#                              #     「任务号与批号:Edit 以收尾围栏起头、插入点前围栏开着」「任务号与批号:MultiEdit 前一处开了围栏、后一处以收尾围栏起头」
#                              #     「任务号与批号:mainagent 与 subagent 括号里的短号」「任务号与批号:agent 括号里的短号」「任务号与批号:会话后面不带括号跟短号」、
#                              #   batch-numbers「任务号与批号:正文里的批号」「任务号与批号:别的会话的批号」「任务号与批号:第几批」「任务号与批号:批号冒号起头的一行」
#                              #     「任务号与批号:批井号、区间与并列的第几批」「任务号与批号:批号后面跟副词只」「任务号与批号:批号后面跟处理、行为、项目」「任务号与批号:锚点里的第几批」
#                              #     「任务号与批号:记录里中文数字的第几批」「任务号与批号:记录里追加第几批」「任务号与批号:kb 里同一分句后面的批号」「任务号与批号:表格行里的批号在 kb 里」
#                              #     「任务号与批号:记录里表格别的列的批号」「任务号与批号:没有表头分隔行的竖线行」「任务号与批号:Edit 删开头围栏行让批号出了围栏」
#                              #     「任务号与批号:Edit 在开头围栏前补一行围栏」「任务号与批号:Edit 把开头围栏的信息串改成带反引号」「任务号与批号:MultiEdit 先写进围栏再删开头围栏行」
#                              #     「任务号与批号:Edit replace_all 一处在围栏里一处在外」「任务号与批号:记录里批后面跟中文数字」「任务号与批号:表格里转义的反斜杠加竖线」；
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=task-and-session-numbers 时「任务号与批号上下文:」起头的几格里靠上下文规则放行的十三格红
#                              #   （反引号里的会话短号与批号、反引号里拿描述指别的会话、A/B 读数一行、派发闸认的 A/B 读数写法、A/B 读数一行两臂两个号、长跑拆分一行、列表里的长跑拆分一行、表格行里的批号、
#                              #   Edit 单改表格里一行、表头在文件里、kb 里括号里的步号、kb 里中文数字的第几批、kb 里批后面跟中文数字、fsync 攒批的大小），
#                              #   与「任务号与批号:派发闸收而值里另带别的写法的号的四条在判决里照拒、两臂与单臂放行」（两臂、单臂那两条靠上下文规则放行）；
#                              #   WRITE_GUARD_BREAK=numbers-new-text-only（这一判只把新写进的每一段当写后、写前当空串，看不到写完之后整份的样子）时
#                              #     「任务号与批号:Edit 以收尾围栏起头、插入点前围栏开着」「任务号与批号:MultiEdit 前一处开了围栏、后一处以收尾围栏起头」
#                              #     「任务号与批号:Edit 删开头围栏行让批号出了围栏」「任务号与批号:Edit 在开头围栏前补一行围栏」「任务号与批号:Edit 把开头围栏的信息串改成带反引号」
#                              #     「任务号与批号:MultiEdit 先写进围栏再删开头围栏行」「任务号与批号上下文:Edit 单改表格里一行、表头在文件里放行」红；
#                              #   库里放宽或改掉一处判法的几项（LIB_FORBIDDEN_NOTATIONS_BREAK=<项>，写法在库的文件头）：
#                              #     fenced-blocks-judged「任务号与批号上下文:围栏代码块里的会话短号与批号放行」「任务号与批号上下文:Edit 以围栏起头、插入点前没有开着的围栏放行」
#                              #       「任务号与批号上下文:Edit 把批号写进围栏放行」「任务号与批号:Edit 删开头围栏行让批号出了围栏」「任务号与批号:Edit 在开头围栏前补一行围栏」
#                              #       「任务号与批号:Edit 把开头围栏的信息串改成带反引号」「任务号与批号上下文:围栏代码块里拿描述指别的会话放行」；
#                              #     fenced-blocks-all-rows「任务号与批号:围栏代码块里的任务号」；
#                              #     after-lines-judged-alone「任务号与批号:Edit 删开头围栏行让批号出了围栏」「任务号与批号:Edit 在开头围栏前补一行围栏」「任务号与批号:Edit 把开头围栏的信息串改成带反引号」
#                              #       「任务号与批号上下文:表格行里的批号放行」「任务号与批号上下文:Edit 单改表格里一行、表头在文件里放行」「任务号与批号上下文:围栏代码块里的会话短号与批号放行」
#                              #       「任务号与批号上下文:Edit 以围栏起头、插入点前没有开着的围栏放行」「任务号与批号上下文:Edit 把批号写进围栏放行」
#                              #       「任务号与批号上下文:围栏代码块里拿描述指别的会话放行」；
#                              #     violations-not-netted「任务号与批号:写前文件里已有的旧号不挡这一次写」「任务号与批号:改了旧号那一行、号没动放行」；
#                              #     violations-netted-by-line-text（按行文相抵，行里别处一改、原有的号就算新写的）「任务号与批号:改了旧号那一行、号没动放行」；
#                              #     violations-netted-by-line-number（写前哪一行号上有违例，写后同一行号上的违例全抵掉）「任务号与批号:旧号那一行新加第二个号照判」；
#                              #     task-numbers-in-code-span-kept「任务号与批号:反引号里 task 子命令加号」「任务号与批号:反引号里的任务号」
#                              #       「任务号与批号:派发闸收而值里另带别的写法的号的四条在判决里照拒、两臂与单臂放行」（那四条里有一条的号写在反引号里，开关放过它）；
#                              #     zero-round-reading-anywhere「任务号与批号:A/B 读数不在行首」「任务号与批号:A/B 读数一行在记录里」「任务号与批号:A/B 读数一行里同字并列的号」
#                              #       「任务号与批号:A/B 读数一行装三个号」「任务号与批号:引用块里的 A/B 读数」「任务号与批号:派发闸收而值里另带别的写法的号的四条在判决里照拒、两臂与单臂放行」
#                              #       「任务号与批号上下文:派发闸认的 A/B 读数写法放行」「任务号与批号:列表里的引用块里的 A/B 读数」（开关下只认「A/B 读数」中间一个空格、后面紧跟冒号）；
#                              #     zero-round-reading-count-unchecked「任务号与批号:A/B 读数一行装三个号」；zero-round-reading-quote-kept「任务号与批号:引用块里的 A/B 读数」
#                              #       「任务号与批号:列表里的引用块里的 A/B 读数」；
#                              #     zero-round-reading-list-quote-kept（引用块只认顶格的 >，列表里的「- >」不算）「任务号与批号:列表里的引用块里的 A/B 读数」；
#                              #     reserved-lines-any-file「任务号与批号:长跑拆分一行在 kb 里」「任务号与批号:表格行里的批号在 kb 里」「任务号与批号:记录里表格别的列的批号」
#                              #       「任务号与批号:没有表头分隔行的竖线行」「任务号与批号:表格里转义的反斜杠加竖线」；
#                              #     table-batch-any-column「任务号与批号:记录里表格别的列的批号」「任务号与批号:没有表头分隔行的竖线行」「任务号与批号:表格里转义的反斜杠加竖线」；
#                              #     table-escaped-pipe-parity-ignored（竖线前面挨着反斜杠的一律当转义，不数奇偶）「任务号与批号:表格里转义的反斜杠加竖线」；
#                              #     batch-size-loosened「任务号与批号:记录里追加第几批」「任务号与批号:kb 里同一分句后面的批号」；
#                              #     measure-compounds-counted「任务号与批号:批号后面跟处理、行为、项目」；
#                              #     bare-short-codes-everywhere「任务号与批号上下文:kb 里括号里的步号放行」；
#                              #     agent-parentheses-unjudged（mainagent( / subagent( / agent( 括号里的短号与别的写法不判）「任务号与批号:mainagent 与 subagent 括号里的短号」「任务号与批号:agent 括号里的短号」；
#                              #     session-word-code-unjudged（「会话」后面不带括号跟的短号不判）「任务号与批号:会话后面不带括号跟短号」；
#                              #     chinese-numeral-batch-unjudged（「批」后面跟的中文数字不判）「任务号与批号:记录里批后面跟中文数字」；
#                              #     task-number-variants-unjudged（「任务编号 <号>」「<号> 号任务」不判）「任务号与批号:任务编号与号任务」；
#                              #     session-description-actions-unjudged（「那一路」后面紧跟动作词的不判）「任务号与批号:那一路后面跟回信、认领、转达这类动作词」；
#                              #     session-descriptions-loose（拿描述指别的会话那一条退成带「一路」的与 mainagent(、subagent( 一律算）「任务号与批号:这一路放行」「任务号与批号:那一路径放行」
#                              #       「任务号与批号:一路绿放行」「任务号与批号:那一路后面不跟会话词放行」「任务号与批号:mainagent 按出场次序的叫法放行」「任务号与批号:subagent 按出场次序的叫法放行」
#                              #       「任务号与批号:那一路后面跟问、报红与说法放行」；
#                              #   库的判据行键 session-descriptions（拿描述指别的会话那一条不判）时「任务号与批号:那一路会话（调度记录原句：发给两路会话那一句）」
#                              #     「任务号与批号:那一路会话（调度记录原句：腿跑着时换上那一句）」「任务号与批号:那一路会话（调度记录原句：问过改检出钩子那一句）」
#                              #     「任务号与批号:那一路的会话与那一路的 agent」「任务号与批号:那一路后面跟回信、认领、转达这类动作词」红；
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=session-descriptions 时「任务号与批号上下文:反引号里拿描述指别的会话放行」红；
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK=judged-suffixes-ignored 时「任务号与批号:.py 不判」一格也红；
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=task-and-session-numbers 时「任务号与批号:门禁样本放行」红；
#                              # 第二道里工作根那一条模式的三样各一个弄坏开关，打开时点名的几项红：
#                              #   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（工作根退回会话临时目录根，lib_work_root.py）：「范围:工作根下的报告目录」「先编后换:experiment-runner 改草稿目录里的副本放行」
#                              #     「stdin:实现员写工作根下的报告」「stdin:七次拒绝都落进检出记录」（多出一次拒绝）「范围:内置 agent 写工作根放行」
#                              #     「范围:会话临时目录根下写拒（scratchpad 之外）」「范围:会话临时目录根下的 scratchpad 里写拒」（工作根退回会话临时目录根之后它成了工作根）
#                              #   SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted（写范围另认会话临时目录根，撤掉之前的放行）：「范围:会话临时目录根下写拒（scratchpad 之外）」「范围:会话临时目录根下的 scratchpad 里写拒」
#                              #   WRITE_GUARD_BREAK=scope-root-unanchored（展开时根后面不带 /）：「范围:工作根同名前缀的兄弟目录」
#                              # 没有项目定义的 agent 只许写工作根（第二道）：
#                              #   WRITE_GUARD_BREAK=builtin-agents-unscoped（没有项目定义的 agent 一概放行）：「范围:内置 agent 写仓里拒」「范围:内置 agent 写仓外别处拒」
#                              #     「范围:定义已删的旧实例写仓里拒」「stdin:内置 agent 写仓里拒」
#                              # 写范围表里 kb-scribe 的不变量清单两行（下一层与各层子目录）与 experiment-runner 的重跑比对对象登记一行：
#                              #   WRITE_GUARD_BREAK=scope-new-rows-dropped（自检用的表副本去掉这三行、experiment-runner 跑前登记那一行退回只认 e*-preregistration.md，退回加行与放宽之前的表）：「范围:kb-scribe 写不变量清单的 .md」「范围:kb-scribe 写不变量清单子目录里的 .md」「范围:experiment-runner 写重跑比对对象登记」「范围:experiment-runner 带轮号的跑前登记」
#                              # 写范围表里 experiment-runner 的英语、日语实验索引与历史条目四行：
#                              #   WRITE_GUARD_BREAK=scope-runner-language-rows-dropped（自检用的表副本去掉这四行）：「范围:experiment-runner 写英语实验索引」「范围:experiment-runner 写日语实验索引」「范围:experiment-runner 写英语历史条目」「范围:experiment-runner 写日语历史条目」
#                              # 紧凑时间戳（五里的 ISO 基本格式写法）有三个开关，打开时点名的几格红（格名都以「紧凑时间戳:」起头，另有「stdin:」两格）：
#                              #   WRITE_GUARD_BREAK=compact-timestamp-skipped（本钩子把命中的「ISO 紧凑时间戳」形态丢掉不报）：拒绝的几例（Write、Edit、MultiEdit、路径里、带冒号的时分、
#                              #     带连字符的日期、带小数秒与偏移）与「stdin:Write 写进紧凑时间戳拒绝（退出码 2）」「stdin:紧凑时间戳的拒绝点名形态与那一处」
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK=iso-compact-timestamp（库里那一条判据行不判）：同上那几格
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose（库里退回「8 位数字加 T 加 4 到 6 位数字」）：「8 位数字后跟 T 但不是时间戳」的放行五例
#                              #     （月份越界、钟点越界、年不是 20 开头、全是 9、前面挨着字母；钟点只有三位与小写 t 两例在这个开关下仍放行），
#                              #     以及「紧凑时间戳:8 位日期、T、带冒号的时分」「紧凑时间戳:带连字符的日期加连写钟点」两例（退回的写法认不出这两种）、
#                              #     「紧凑时间戳:Write / Edit / MultiEdit 写进仓里拒绝并点名形态与那一处」三格与「stdin:紧凑时间戳的拒绝点名形态与那一处」（退回的写法报出来的串少了末尾的 Z）
#                              #   LIB_FORBIDDEN_NOTATIONS_BREAK=compact-hyphenated-date-loose（库里带连字符的日期不核月日）：「紧凑时间戳:带连字符的日期月份越界放行」「紧凑时间戳:带连字符的日期日越界放行」两格
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
#                              #   unreadable-lock-absent（锁文件在而判不了的当没有锁）：「写者锁:锁文件不是 JSON 拒」「写者锁:锁缺 nonce 拒」「写者锁:锁缺 dispatched_at 拒」
#                              #     「写者锁:锁缺 prompt_sha256 拒」「写者锁:锁的位置是目录拒」「写者锁:判不了的锁的拒绝点名那份锁与出路」「stdin:坏锁拒绝（退出码 2）」「stdin:坏锁的拒绝点名那份锁与出路」
#                              #   missing-receipt-claims（派发回执读不到时照旧先到先得认领）：「写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全」
#                              #     「写者锁:真持锁者回执还没落盘时第一次写也放行、不认领」「写者锁:回执落盘之后持锁者写拿到锁」「写者锁:持锁者认领之后，回执一直没落盘的乙写被拒」
#                              #     「写者锁:认领之后持锁者续写放行」「stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）」
#                              #   exclusive-temp-kept（排他建锁、建认领时 JSON 写到一半抛异常留下临时文件）：「写者锁:排他建锁时锁内容写不进 JSON，抛异常、不留临时文件也不留锁」
#                              #     「写者锁:认领时认领内容写不进 JSON，抛异常、不留临时文件也不留认领」
#                              #   takeover-sweeps-new-claims（接手换上之后删掉所有 nonce 不同的认领与认领的临时文件）：
#                              #     「写者锁:两次接手交错（A 换锁、B 换锁并认领、A 清理）之后 B 的认领还在，第三个认领不成，旧认领删掉」「写者锁:接手不删别的进程正在写的认领临时文件」
#                              #   broken-claim-absent（认领文件在而判不了的当没有认领）：「写者锁:认领文件判不了的拒绝点名那份认领文件与出路」
#                              #     「写者锁:认领文件判不了的不另建一份、不改它」「stdin:坏认领文件拒绝并点名那份认领文件（退出码 2）」
#                              # 落点（第二道与 landing_target）：WRITE_GUARD_BREAK=landing-lexical-only（退回只按字面规整、不解符号链接）时
#                              #   「范围:范围内的链接指到范围外，经它写拒」「范围:范围内的链接指到范围外，经它写不存在的子目录拒」「范围:范围内的链接指到仓里范围外拒」
#                              #   「范围:范围内的悬空链接指到范围外拒」「覆盖:仓外的链接指到仓里未跟踪的文件，Write 拒」「先编后换:仓外的链接指到主工作区的入库装置拒」
#                              #   「stdin:范围内的链接指到范围外拒（退出码 2）」与「范围:范围外的链接指回范围内放行」（字面路径在范围外，只认字面就拒）红
#                              #   （「范围:范围内的链接指到范围内放行」在开关下照样绿）；
#                              # 判不了照常放行而记检出：WRITE_GUARD_BREAK=unparsable-input-silent（输入不是合法 JSON 对象时静默退 0）时
#                              #   「stdin:坏 JSON 的 stderr 有诊断」「stdin:坏 JSON 记一条检出（结局只记、会话号读不到）」「stdin:坏 JSON 里按字面找得到的会话号与类型照抄」
#                              #   「stdin:JSON 数组记一条检出」红（「stdin:坏 JSON 放行（退出码 0）」照样绿）；
#                              #   WRITE_GUARD_BREAK=unjudged-not-recorded（判不了的只打 stderr、不记检出）时
#                              #   「写者锁:读不到库时记一处没判全」「写者锁:自己出错时记一处没判全」「stdin:写者锁自己出错放行并记一条检出（结局只记）」
#                              #   「写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全」「写者锁:真持锁者回执还没落盘时第一次写也放行、不认领」
#                              #   「stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）」红；
#                              #   WRITE_GUARD_BREAK=syntax-unreadable-unrecorded（第六道 Edit 的目标在而读不了时静默放行）时「语法:改一份读不了的脚本照常放行、记一处没判全」红
#                              # 先编后换的出路（第四道）：WRITE_GUARD_BREAK=compile-first-capped-hint（出路写 capped.sh <线程上限> 直接起 compile-then-swap.py）时
#                              #   「stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起」红
#                              # 第十道（往现行账本追加节标题）：WRITE_GUARD_BREAK=ledger-append-ignored（判到该拒的也放行）时
#                              #   「账本追加:Edit 往现行账本追加节标题拒」「账本追加:MultiEdit 往现行账本追加节标题拒」「账本追加:Write 往现行账本追加节标题拒」
#                              #   「账本追加:拒绝点名新脚本、账本与出路」「stdin:往现行账本追加节标题拒（退出码 2）」「stdin:往现行账本追加节标题的拒绝点名新脚本与出路」红
# 第十一道（工具员就地改主工作区里已存在的脚本）的弄坏开关：
#   WRITE_GUARD_BREAK=script-in-place-allowed  第十一道整道不判：该拒的放行，判不了的那两种也不记没判全
#       「整份换上:工具员 Edit 主工作区已存在的 scripts/mutation/ 下的 .py 拒」「整份换上:工具员 Edit 主工作区已存在、带执行位无后缀的 gpu-verification-server/control 拒」
#       「整份换上:工具员 Write 整份重写已存在的 .claude/gate.d/ 下的 .sh 拒」「整份换上:工具员 MultiEdit 已存在的 .claude/scripts/ 下的 .py 拒」
#       「整份换上:工具员 Edit 已存在的 task-server/ 下的 .py 拒」「整份换上:工具员 Edit 已存在、带执行位无后缀的 prune-server/control 拒」
#       「整份换上:工具员 Write 已有门禁的一份样本拒」「整份换上:工具员在已有门禁的样本档目录里新建样本拒」
#       「整份换上:已有门禁的样本的拒绝给草稿目录、补丁与打补丁的出路」「整份换上:认不出 agent 类型的子 agent 写已有门禁的样本照常放行、记一处没判全」
#       「stdin:工具员 Write 已有门禁的样本档目录里的新样本拒、点名补丁出路」
#       「stdin:工具员 Edit 主工作区已存在的 scripts/mutation/ 下的 .py 拒并点名 hook-swap.py 与出路」
#       「stdin:工具员 Edit 主工作区已存在的 gpu-verification-server/control 拒并点名 hook-swap.py 与出路」
#       「整份换上:认不出 agent 类型的子 agent 调用照常放行、记一处没判全」「整份换上:目标 stat 失败照常放行、记一处没判全」
#       「整份换上:工具员 Edit 主工作区已存在的 prune-server/src/protocol.rs 拒」「整份换上:工具员 Edit 主工作区已存在的 gpu-verification-server/src/ 下的文件拒」
#       「整份换上:两个服务 src/ 的拒绝给仓副本与补丁的出路」
#       「整份换上:工具员 Edit 主工作区已存在的 prune-server/Cargo.toml 拒」「整份换上:工具员 Edit 主工作区已存在的 prune-server/Cargo.lock 拒」
#       「整份换上:工具员 Write 整份重写已存在的 gpu-verification-server/Cargo.toml 拒」「整份换上:两个服务构建清单的拒绝给仓副本与补丁的出路」
#       「整份换上:登记成 required 的脚本，拒绝说明写不加 --verify、自证换上之后另交一条、没有自动回滚」「整份换上:不在登记表里的脚本，拒绝说明照旧写加 --verify 换上」
#       「整份换上:登记成 own-server 的脚本照不在表里办，写加 --verify 换上」「整份换上:登记表读不到时两种出路都写、记一处没判全」
#       「整份换上:门禁共用库（.claude/gate.d/lib/ 下）不是门禁阶段，照不在登记表里的脚本写加 --verify 换上」
#       「整份换上:门禁阶段（.claude/gate.d/ 顶层的 .sh）单走一支：不带 --verify、样本档换上之后另交一条、没有自动回滚、阶段单放一份规格」
#       「整份换上:登记表读不到时门禁阶段照写不带 --verify、不记没判全（不读登记表）」
#       「整份换上:.claude/gate.d/ 顶层 lib- 起头的 .py 是门禁共用库、不是门禁阶段，照不在登记表里的脚本写加 --verify 换上」
#       「stdin:工具员 Edit 主工作区已存在的 .claude/gate.d/stage.sh 拒、点名不带 --verify 与样本档换上之后另交一条」
#   WRITE_GUARD_BREAK=gate-fixtures-in-place-allowed  第十一道不判已有门禁的样本档目录：工具员改已有的样本、在那个目录里新建样本照放
#       「整份换上:工具员 Write 已有门禁的一份样本拒」「整份换上:工具员在已有门禁的样本档目录里新建样本拒」
#       「整份换上:已有门禁的样本的拒绝给草稿目录、补丁与打补丁的出路」「整份换上:认不出 agent 类型的子 agent 写已有门禁的样本照常放行、记一处没判全」
#       「stdin:工具员 Write 已有门禁的样本档目录里的新样本拒、点名补丁出路」
#   WRITE_GUARD_BREAK=swap-manifests-unseen  第十一道不把两个服务的构建清单（Cargo.toml、Cargo.lock）当源码一类：该拒的放行
#       「整份换上:工具员 Edit 主工作区已存在的 prune-server/Cargo.toml 拒」「整份换上:工具员 Edit 主工作区已存在的 prune-server/Cargo.lock 拒」
#       「整份换上:工具员 Write 整份重写已存在的 gpu-verification-server/Cargo.toml 拒」「整份换上:两个服务构建清单的拒绝给仓副本与补丁的出路」
#   WRITE_GUARD_BREAK=swap-remedy-always-verify  第十一道脚本一支的出路一律写加 --verify，不读要临时服务的自证登记表
#       「整份换上:登记成 required 的脚本，拒绝说明写不加 --verify、自证换上之后另交一条、没有自动回滚」「整份换上:登记表读不到时两种出路都写、记一处没判全」
#   WRITE_GUARD_BREAK=gate-stage-remedy-as-script  门禁阶段与别的脚本走同一支出路（读登记表：不在表里写加 --verify，表读不到两种都写）
#       「整份换上:门禁阶段（.claude/gate.d/ 顶层的 .sh）单走一支：不带 --verify、样本档换上之后另交一条、没有自动回滚、阶段单放一份规格」
#       「整份换上:登记表读不到时门禁阶段照写不带 --verify、不记没判全（不读登记表）」
#       「stdin:工具员 Edit 主工作区已存在的 .claude/gate.d/stage.sh 拒、点名不带 --verify 与样本档换上之后另交一条」
# 引导（下面读部分文件的那一段）的弄坏开关：
#   WRITE_GUARD_BREAK=part-unreadable-passes  引导载不全部分文件（读不到、编不过、编得过而 import 抛错或入口要的名字缺）时不按拒判：照原样抛错、进程退 1（Claude Code 照常放行、检出记录一行不写）；只在自证起的子进程里认（引导读它，不经 python 部分文件）
#       「引导:读不到部分文件 write_guard_decisions.py 按拒判（退 2、点名那一份）」「引导:编不过部分文件 write_guard_decisions.py 按拒判（退 2、点名那一份）」「引导:截空部分文件 write_guard_decisions.py 按拒判（退 2、点名那一份）」
#       「引导:读不到部分文件 write_guard_selftest.py 按拒判（退 2、点名那一份）」「引导:编不过部分文件 write_guard_selftest.py 按拒判（退 2、点名那一份）」「引导:截空部分文件 write_guard_selftest.py 按拒判（退 2、点名那一份）」
#       「引导:读不到部分文件时对写回那一份本身的 Write 放行（退 0、记一条检出）」「引导:读不到部分文件时对那一份本身的 Edit 照拒（退 2、点名那一份）」
# python 代码分在同目录的这几份（本文件是入口：bash 这一层与下面的引导；引导先把每一份读进来、核顶层名字、语法解析过（只读语法树、不执行），再按次序 import 它们（每一份是普通 python 模块，用到别份的名字写显式 import、不留环），不单独执行）：
# write_guard_decisions.py：各道判定（覆盖、写范围、写者锁、先编后换、撇号类角标、行尾空白、钟点、会话号、批次代号、任务号、账本、整份换上、语法）与检出记录
# write_guard_selftest.py：自证 selftest、弄坏开关只在自证里认的拒绝与读 hook 输入的入口流程
# 载不全任何一份（读不到、编不过、编得过而不完整：顶层没有入口要的名字——WRITE_GUARD_PART_REQUIRED_NAMES 一份一组，import 之前用 ast 静态核、点名缺哪个——或 import 抛错）：照缺依赖（lib_write_scope.py 之类）的处置按拒判，退 2、stderr 点名是哪一份，不抛错退 1；
# 只有一次写放行：Write 写回点名的那一份本身（出路只剩这一条，放行同时记一条检出）；Edit、MultiEdit 与别的目标照拒。主函数 main 在引导核完之后由引导调用，部分文件末尾不自己退出。
# 弄坏开关的名单、读开关的代码在这几份 python 里（引导里那一条在引导里），开关表与红的格写在这个文件头。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
# 写成 `python3 - <<PY` 时程序占了标准输入，JSON 读不到，判定一律放行。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import ast, os, sys
WRITE_GUARD_PART_NAMES = ("write_guard_decisions.py", "write_guard_selftest.py",)   # 按次序 import（后一份 import 前一份、没有环）；路径按钩子自己所在的目录（argv[1]）取
WRITE_GUARD_PART_REQUIRED_NAMES = (   # 与上一行同次序、一份部分文件一组：入口要的名字（这一份第一个、最后一个绑定的名字与别份、入口引到的名字）；import 之前用 ast 核这一份顶层的 def、class 与赋值绑定了这些名字，缺一个就是不完整（截空、截短）
    ("expanded_scope_patterns", "decide", "record_detection", "selftest_base_directory"),
    ("selftest", "refuse_break_outside_selftest", "main"),
)
# 弄坏开关与关闭开关只在自证里认：自证起的子进程靠这两个值认（自证与读开关的代码在 write_guard_selftest.py）；引导在部分文件载入之前就读 WRITE_GUARD_BREAK，所以定义放在这里
SELFTEST_MARK_VARIABLE = "WRITE_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "write-guard.sh --selftest 起的子进程"

def part_written_back(hook_part_path):
    """这一次的 hook 输入是不是 Write 写回 hook_part_path：是交 hook 输入（dict），别的工具、别的目标、读不出输入交 None。目标按解开符号链接之后的落点比。"""
    import json
    try:
        hook_input = json.loads(sys.stdin.buffer.read().decode("utf-8"))
        target = os.path.join(hook_input.get("cwd") or os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd(), hook_input["tool_input"]["file_path"])
        if hook_input.get("tool_name") == "Write" and os.path.realpath(target) == os.path.realpath(hook_part_path):
            return hook_input
    except (ValueError, KeyError, TypeError, AttributeError, OSError):   # 不是 JSON 对象、没有目标、目标不是路径
        pass
    return None

def record_part_written_back(hook_input, hook_part_path):
    """部分文件载不全时放行的那一次写（写回那一份）照「放行但必记检出」往检出记录追加一行：decisions 的 record_detection 此刻用不了，借同目录 lib_work_root.py 的 record_unjudged_pass。交一句给人看的说明。"""
    sys.dont_write_bytecode = True
    try:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_work_root", os.path.join(sys.argv[1], "lib_work_root.py"))
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        return library.record_unjudged_pass("write-guard.sh", hook_input, f"Write：{hook_part_path}", [f"写闸没判全：部分文件 {hook_part_path} 载不全，这一次 Write 写回的就是它，放行"])[1]
    except Exception as error:   # 文件不在、语法错都算读不到
        return f"检出没记成：读不到同目录的 lib_work_root.py（{error!r}）"

def hook_part_unusable(hook_part_path, hook_part_error):
    """一份部分文件载不全：这一次写没法判，按拒判（退 2、点名那一份）；Write 写回这一份本身的放行（只在钩子那种起法里读 hook 输入）。"""
    if os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT and os.environ.get("WRITE_GUARD_BREAK") == "part-unreadable-passes":
        raise hook_part_error   # 弄坏开关 part-unreadable-passes：照原样抛错、进程退 1（只在自证起的子进程里认）
    problem = f"读不到或编不过部分文件 {hook_part_path}（{type(hook_part_error).__name__}：{hook_part_error}）"
    written_back = part_written_back(hook_part_path) if len(sys.argv) == 2 else None
    if written_back is not None:
        print(f"  ! write-guard.sh：{problem}；这一次是 Write 写回这一份，放行（没判）；{record_part_written_back(written_back, hook_part_path)}", file=sys.stderr)
        sys.exit(0)
    print(f"  ✗ write-guard.sh：{problem}，这一次写没法判，按拒判", file=sys.stderr)
    print("     → 怎么办：把这一份恢复成编得过、完整的版本再写（git show HEAD:.claude/hooks/<这一份的文件名> 取内容，或让主 agent 换回）；这一份恢复之前，写闸拒每一次 Write、Edit、MultiEdit，只放行 Write 写回这一份本身", file=sys.stderr)
    sys.exit(2)

def hook_part_bound_names(hook_part_tree):
    """部分文件顶层（不进 if、try、函数体）绑定的名字：def、class、赋值的目标。只读语法树，不执行这一份。"""
    bound = set()
    for statement in hook_part_tree.body:
        if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            bound.add(statement.name)
        elif isinstance(statement, ast.Assign):
            for target in statement.targets:
                bound.update(node.id for node in ast.walk(target) if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store))
        elif isinstance(statement, ast.AnnAssign) and isinstance(statement.target, ast.Name) and statement.value is not None:
            bound.add(statement.target.id)
    return bound

sys.dont_write_bytecode = True   # 部分文件按同目录的模块 import：不往 __pycache__ 里写（自证拷一份钩子目录再写坏、截空其中一份，缓存的字节码不许盖住）
sys.path.insert(0, sys.argv[1])   # 部分文件之间的 import 与下面的 import 都从钩子自己所在的目录找
for hook_part_name, hook_part_required in zip(WRITE_GUARD_PART_NAMES, WRITE_GUARD_PART_REQUIRED_NAMES, strict=True):   # 每一份先读进来、核顶层绑定了入口要的名字、语法解析过（只读语法树、不执行），全核完全过了才 import：读不到、语法错或缺名字的，按缺依赖的处置（点名是哪一份、缺哪个名字），不让 python 在 main 之前抛错退 1 而被照常放行
    hook_part_path = os.path.join(sys.argv[1], hook_part_name)
    try:
        with open(hook_part_path, encoding="utf-8") as hook_part_handle:
            hook_part_tree = ast.parse(hook_part_handle.read(), hook_part_path)
        hook_part_bound = hook_part_bound_names(hook_part_tree)
        hook_part_missing = [name for name in hook_part_required if name not in hook_part_bound]
        if hook_part_missing:
            raise LookupError(f"顶层没有绑定 {'、'.join(hook_part_missing)}，这一份不完整")
    except (OSError, SyntaxError, ValueError, LookupError) as hook_part_error:   # 文件不在、读不了、不是 UTF-8、语法错、含空字节、缺名字
        hook_part_unusable(hook_part_path, hook_part_error)
try:   # 每一份单独 import（后一份 import 前一份、没有环）：名字齐了、运行到一半抛错（引到被截掉的名字、模块层语句出错）点名出错的那一份
    import write_guard_decisions
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "write_guard_decisions.py"), hook_part_error)
try:
    from write_guard_selftest import main
except Exception as hook_part_error:
    hook_part_unusable(os.path.join(sys.argv[1], "write_guard_selftest.py"), hook_part_error)
sys.exit(main())   # 主函数由引导在核完之后调，不靠部分文件末行自己退出
PY
