"""run-scope-guard.sh 的判法：起验证脚本的命令要带范围声明 SINGLEFS_RUN_SCOPE，种类与范围要与命令的形状对得上；提交带放宽检查的标识的要用户确认。

认法、要声明与不要声明的形态、拒绝说明的样子、要用户确认的写法、检出记录、罩不住的写法与弄坏开关都写在 run-scope-guard.sh 的文件头，这里不另抄。
切词、剥前缀与包装用同目录的 lib_shell_words.py；剥 flock、/usr/bin/time、gpu-quiesce-run.sh 这一类，
认门禁阶段、认 cargo 的子命令用 lib_project_code_runs.py（按文件路径导入，不另写一份）。
用法：python3 lib_run_scope.py <钩子目录> [--selftest]，由 run-scope-guard.sh 起；hook 的 JSON 从标准输入读。
代码分在四份：本文件是入口（文件头、部分模块的载入与读不到时的处置、名字的再导出、命令行入口），引导要的名字（弄坏开关变量名、自证标记、守卫脚本名、钩子目录、部分文件名表、读不到时退的码）在同目录的 lib_run_scope_bootstrap.py，判法（常量、弄坏开关表、判法、hook_main）在 lib_run_scope_core.py，自证（样本树、自证格、开关轮 selftest）在 lib_run_scope_selftest.py；本文件按 RUN_SCOPE_PART_NAMES 的次序 import 部分模块（后一份 import 前一份、没有环），把它们的名字全部再导出，调用方照旧按这份文件的路径导入、按这份的名字取；自证里换成替身的模块变量（submit_flag_asks_in_text）在 lib_run_scope_core.py 里换，那里的函数读的是它。
弄坏开关与自证格（同一张表也写在 run-scope-guard.sh 文件头，自证末了的开关轮核表里的格与 BREAK_SWITCH_CELLS 逐格相等；这里照抄一份给门禁 code-tooling 的 break-switch-headers 格读，改了一起改）。
平常跑设了 RUN_SCOPE_GUARD_BREAK 就在一切之前退 2、点名变量只在自证里认（本份的命令行入口与入口脚本开头那一道同一个判法）：
   RUN_SCOPE_GUARD_BREAK=declaration-optional  没带声明也放
       红：「task-server 提交带 --full-gate 的门禁整道：没带声明拒」「task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒」「门禁整道：没带声明拒」「崩溃点重放不指定范围跑：没带声明拒」「崩溃放量全部分项：没带声明拒」「崩溃放量双机驱动：没带声明拒」「replay.sh 全部实验：没带声明拒」「gate-staged.sh 没带声明拒」「cargo test 整工作区：没带声明拒」「bash -c 里的门禁没带声明拒」「here-string 喂给 bash 的 cargo test 整工作区：没带声明拒」「printf 经管道喂给 bash 的门禁：没带声明拒」「echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒」「capped.sh 包着的 cargo test 没带声明拒」「env 带别的变量、没带声明拒」「nice 与 timeout 包着、没带声明拒」「flock 包着的门禁没带声明拒」「gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒」「/usr/bin/time 包着的 cargo test 没带声明拒」「驱动脚本里一行缺声明拒」「外面带声明、驱动脚本里一行缺声明拒」「驱动脚本再起驱动脚本、里面缺声明拒」「直接执行的驱动脚本缺声明拒」「同一条命令里 heredoc 写出的驱动脚本缺声明拒」「入口：没带声明拒（退出码 2）」「入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法」「入口：拒绝说明附上次红的清单」「入口：没有上次红的记录时照写」「入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）」「驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」
   RUN_SCOPE_GUARD_BREAK=segments-unchecked  不核四段齐不齐、值空不空、写没写两次、种类认不认得
       红：「声明缺段：没写为什么拒」「声明值空：范围一段是空的拒」「声明写两次：种类写两次拒」「声明种类认不出拒」
   RUN_SCOPE_GUARD_BREAK=kind-unchecked  不核声明的种类与命令的形状
       红：「门禁 --item 声明成 whole：种类对不上拒」「merge-and-verify 声明成 whole：拒」「cargo test 整包声明成 target：种类对不上拒」「声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面」
   RUN_SCOPE_GUARD_BREAK=count-unchecked  不核范围一段的数（items、target）与整轮整包（whole）
       红：「门禁 --check 两格写成 1 格：范围数对不上拒」「whole 范围没写整轮或整包拒」「cargo test 两个目标写成 3 个：范围数对不上拒」「cargo test --exact 两个测试写成 1 个：范围数对不上拒」「崩溃点重放点名一项另加展开深度：写成两项拒（深度的值不是项）」「merge-and-verify 点名两项写成 1 项：范围数对不上拒」「merge-and-verify 两份补丁写成 1 份：范围数对不上拒」「范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法」「范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验」
   RUN_SCOPE_GUARD_BREAK=reason-unchecked  不核为什么一段的字数
       红：「gate.sh 为什么只写验证拒」「为什么太短拒」「为什么只写要跑验证拒」
   RUN_SCOPE_GUARD_BREAK=count-hint-unprinted  范围数或种类对不上的说明不写这条命令该写的整段「种类=…」「范围=<数> <单位>」与数法，只写点名了几格几项（弄坏的是说明，判拒不变）
       红：「范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法」「范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验」「声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面」
   RUN_SCOPE_GUARD_BREAK=variable-built-hint-dropped  命令里有变量拼的词而范围或种类对不上时，说明里不写「把 --item 写成字面」
       红：「声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面」
   RUN_SCOPE_GUARD_BREAK=subagent-submissions-allowed  子 agent 经 task-server/task submit 交的验证命令不按 lib_project_code_runs.subagent_task_server_refusal 拒（许子 agent 自己交）
       红：「子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）」「子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）」「入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回」「入口：命令开头已有声明时模板只留一份」
   RUN_SCOPE_GUARD_BREAK=scripts-unread  命令位置上执行的驱动脚本不读进去
       红：「驱动脚本里一行缺声明拒」「外面带声明、驱动脚本里一行缺声明拒」「驱动脚本再起驱动脚本、里面缺声明拒」「直接执行的驱动脚本缺声明拒」「同一条命令里 heredoc 写出的驱动脚本缺声明拒」「入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）」「驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）」「入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）」
   RUN_SCOPE_GUARD_BREAK=driver-path-variables-unexpanded  bash / sh / source 起的脚本路径写成赋过字面值的变量（`D=<目录>; bash $D/x.sh`）的不展开，照旧不读进去
       红：「驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」
   RUN_SCOPE_GUARD_BREAK=wrappers-opaque  flock、/usr/bin/time、gpu-quiesce-run.sh 不剥，里面那条不判
       红：「flock 包着的门禁没带声明拒」「gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒」「/usr/bin/time 包着的 cargo test 没带声明拒」
   RUN_SCOPE_GUARD_BREAK=allowed-unrecorded  放行的不记进检出记录
       红：「入口：放行的记进检出记录（四段与命令）」
   RUN_SCOPE_GUARD_BREAK=last-runs-unread  拒绝说明不读上次红的清单
       红：「入口：拒绝说明附上次红的清单」
   RUN_SCOPE_GUARD_BREAK=query-forms-declared  只查清单、自证、只编不跑、只列用例的写法也要声明
       红：「门禁 --list 不要声明」「门禁 --selftest 不要声明」「崩溃点重放 --list-items 不要声明」「replay.sh --admission-only 不要声明」「replay.sh --list-items 不要声明」「cargo test --no-run 不要声明」「cargo test -- --list 不要声明」「merge-and-verify --list-items 不要声明」
   RUN_SCOPE_GUARD_BREAK=replay-list-items-declared  replay.sh 的 --list-items 也要声明
       红：「replay.sh --list-items 不要声明」
   RUN_SCOPE_GUARD_BREAK=merge-as-whole  merge-and-verify 照整轮（whole）判，带不带 --item 都是
       红：「merge-and-verify 声明成 whole：拒」「merge-and-verify 两份补丁：声明成 items 放」「merge-and-verify 两份补丁另加展开深度：写成 2 份补丁放（深度的值不是补丁）」「merge-and-verify 点名两项：声明成 items 放」「task-server 提交合入后验证带 --item：放」
   RUN_SCOPE_GUARD_BREAK=merge-list-items-declared  merge-and-verify 的 --list-items 也要声明
       红：「merge-and-verify --list-items 不要声明」
   RUN_SCOPE_GUARD_BREAK=tooling-scripts-read  仓里的工具脚本（scripts/、.claude/ 下）也读进去
       红：「仓里的工具脚本不读：放行」
   RUN_SCOPE_GUARD_BREAK=cargo-targets-as-whole  cargo test 点名目标的也当整包
       红：「cargo test 点名源码在 tests/<目标>/ 下、哪个文件都没读缩域变量的目标：不走崩溃枚举，放」「cargo test 两个目标、带对的声明：放」「cargo test --lib --exact 两个测试、带对的声明：放」「子 agent cargo test 点名目标、带对的声明：放」「capped.sh 外面带声明放」「驱动脚本里每一行都带声明：放」「驱动脚本路径写成赋过字面值的变量、里面每一行都带声明：放」「cargo test 点名不走崩溃枚举的目标：放」「task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放」「cargo test 点名别的包的目标：这一条不判，放」「cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「task-server 提交 cargo test 点名 --exact 的测试函数：放」
   RUN_SCOPE_GUARD_BREAK=template-keeps-declaration  照抄模板时不摘命令开头已有的那份声明（照抄出来两份，bash 取后一份）
       红：「入口：命令开头已有声明时模板只留一份」
   RUN_SCOPE_GUARD_BREAK=failure-unrecorded  判不了的几样（hook 输入不是 JSON 对象、判的时候出异常、喂给 shell 的不是字面、读不到判法文件）只打 stderr 或什么都不说、不记检出
       红：「入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出」「入口：判的时候抛异常放行、stderr 报一句、记一条检出」「入口：here-string 喂给 bash 的是变量放行、记一条检出」「入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）」「入口：判法文件读不到时记一条检出（结局只记、会话号照输入）」「入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置」
   RUN_SCOPE_GUARD_BREAK=judge-crash-unrecorded  判法文件在而没跑完（导入就出错、被信号停掉，退 0 与 2 以外的码）时只打 stderr、不记检出（exec 判法，退几都照常放行）
       红：「入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）」「入口：判法的部分文件读不到：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable」「入口：判法的部分文件编不过：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable」「入口：判法的部分文件截空：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable」
   RUN_SCOPE_GUARD_BREAK=driver-unread-unrecorded  读不了、太大的驱动脚本静默不读、不交判不了的说明
       红：「入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）」
   切词那一份的开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread（lib_shell_words.py：here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明）下也红，不在开关轮里：
       红：「here-string 喂给 bash 的 cargo test 整工作区：没带声明拒」「printf 经管道喂给 bash 的门禁：没带声明拒」「echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒」「入口：here-string 喂给 bash 的是变量放行、记一条检出」
   RUN_SCOPE_GUARD_BREAK=honor-outside-selftest  平常跑（不是 --selftest、也不是自证起的子进程）设了开关也照认、不拒；它是关守卫项，只在单独一项时认，与别的项同写照拒
       红：「入口：平常跑设了弄坏开关退 2、点名它只在自证里认」「入口：标记文件内容对不上照拒」
   RUN_SCOPE_GUARD_BREAK=honor-outside-selftest-with-others  带标记的自证子进程里，关守卫项与别的项同写也照认（honor-outside-selftest 写在哪一项都认；只在带标记的子进程里认，平常跑同写照拒）
       红：「入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它」
   RUN_SCOPE_GUARD_BREAK=task-server-submissions-unread  task-server/task submit -- 之后那一条不判（当名单上的客户端放过去）
       红：「task-server 提交的门禁整道：没带声明拒」「task-server 提交的 cargo test 整工作区：没带声明拒」「task-server 提交带 --full-gate 的门禁整道：没带声明拒」「task-server 提交带 --manual 的门禁整道：没带声明拒」「task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒」「子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）」「子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）」「入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回」「入口：命令开头已有声明时模板只留一份」「task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）」「task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒」「task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒」「task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）」「task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）」「入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」「task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）」「task-server 提交 cargo test 点名目标、不点名测试函数：拒」「task-server 提交证红不点名变异：拒」「task-server 提交门禁点起测试的格、不带 --item：拒」「入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）」「task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）」「task-server 提交 bash -c、里面是带分片标记的入口：拒（套在 bash -c 里不分片）」「task-server 提交 bash -c 里带分片标记的入口、task_server_lib 载不进：放行、记一处判不了」「入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）」
   RUN_SCOPE_GUARD_BREAK=submit-flags-unjudged  放宽标识一律不判，带了也不弹窗
       红：「提交带 --full-gate：弹窗」「提交带 --whole-package-confirmed：弹窗」「范围声明写在前面：弹窗」「bash -c 里面：弹窗」「&& 连着的后一段：弹窗」「--cwd 在标识前面：弹窗」「两个标识都带：都点名」「python3 起客户端：弹窗」「cd 进 task-server 再起：弹窗」「cd 到算不出的目录再起：弹窗」「cd 到算不出的目录再起 ./task：弹窗」「eval 字面代码：弹窗」「赋过字面值的变量写在参数上：认出标识」「数组写在参数上：认出标识」「标识写成等号带值：弹窗」「参数是变量：弹窗认不出」「数组全部元素写在参数上：弹窗认不出」「eval 给出的参数是变量：弹窗认不出」「子命令位是变量：弹窗认不出」「参数是命令替换：弹窗认不出」「xargs 读进来的参数：弹窗认不出」「flock 包着：弹窗」「python3 -c 里字面写着：弹窗」「驱动脚本里：弹窗」「交给服务的那一条里又提交：弹窗」「脚本里的转发函数：弹窗」「命令词是赋过字面值的变量：弹窗」「eval 赋过字面值的变量：弹窗」「判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了」「入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）」「入口：参数是变量弹窗、理由写认不出带没带标识」
   RUN_SCOPE_GUARD_BREAK=submit-flags-by-substring  命令文字里提到标识就弹窗，不按切词判
       红：「grep 这个词：不弹窗」「commit message 带这个词：不弹窗」「--why 里提到这个词：不弹窗」「交给服务的那一条里提到这个词：不弹窗」「echo 这条命令的文字：不弹窗」「范围声明的值里提到这个词：不弹窗」
   RUN_SCOPE_GUARD_BREAK=submit-flag-variables-unread  `--` 之前算不出的词不当认不出，只判写出来的标识
       红：「参数是变量：弹窗认不出」「数组全部元素写在参数上：弹窗认不出」「eval 给出的参数是变量：弹窗认不出」「子命令位是变量：弹窗认不出」「参数是命令替换：弹窗认不出」「xargs 读进来的参数：弹窗认不出」「入口：参数是变量弹窗、理由写认不出带没带标识」
   RUN_SCOPE_GUARD_BREAK=submit-flag-after-separator  `--` 之后交给服务的那一条的参数也当 submit 的选项判
       红：「交给服务的那一条里提到这个词：不弹窗」「交给服务的那一条参数是变量：不弹窗」「xargs 读进来的落在交给服务的那一条里：不弹窗」
   RUN_SCOPE_GUARD_BREAK=submit-flag-first-option-only  只判 submit 后面紧跟的一个词
       红：「--cwd 在标识前面：弹窗」「两个标识都带：都点名」
   RUN_SCOPE_GUARD_BREAK=submit-flag-client-by-path-only  客户端只认解得开路径、对得上本仓那一份的，当前目录算不出的不认
       红：「cd 到算不出的目录再起：弹窗」「cd 到算不出的目录再起 ./task：弹窗」
   RUN_SCOPE_GUARD_BREAK=submit-flag-nesting-unread  包装、转发函数、程序里写着的、驱动脚本、交给服务的那一条与命令词上的变量都不往里判
       红：「xargs 读进来的参数：弹窗认不出」「flock 包着：弹窗」「python3 -c 里字面写着：弹窗」「驱动脚本里：弹窗」「交给服务的那一条里又提交：弹窗」「脚本里的转发函数：弹窗」「命令词是赋过字面值的变量：弹窗」「eval 赋过字面值的变量：弹窗」
   RUN_SCOPE_GUARD_BREAK=submit-flag-subagents-asked  子 agent 带标识的也弹窗
       红：「子 agent 带标识：不弹窗（由项目代码闸拒）」
   RUN_SCOPE_GUARD_BREAK=submit-flag-raise-propagated  判放宽标识抛了异常时不接住（范围声明跟着没判）
       红：「判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了」
   RUN_SCOPE_GUARD_BREAK=submit-flag-ask-silent  判出来了也不打 ask 的 JSON（等于放行）
       红：「入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）」「入口：参数是变量弹窗、理由写认不出带没带标识」「入口：判的时候抛异常、命令文字里提到标识：弹窗」
   RUN_SCOPE_GUARD_BREAK=commit-whole-flag-unchecked  不核提交时才许跑的六道门禁与崩溃点重放、崩溃放量两份入口（不指定范围那一形）经 submit 交时带没带放宽标识：一个标识都没带的也放
       红：「task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）」「task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒」「task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒」「task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）」「task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）」「入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）」
   RUN_SCOPE_GUARD_BREAK=crash-domain-replay-items-unjudged  崩溃点重放入口点名的项不判会不会走崩溃枚举
       红：「崩溃点重放点名会走崩溃枚举的项、没缩域：拒」「崩溃点重放点名放量分项的项、没缩域：拒」「崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒」「入口：崩溃域没缩拒（退出码 2、说明写缩域变量与全域理由的写法）」「崩溃点重放点名的项不在合表里：放行、记一处判不了」「崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）」
   RUN_SCOPE_GUARD_BREAK=crash-domain-cargo-targets-unjudged  cargo test 点名的验证包测试目标不判会不会走崩溃枚举
       红：「cargo test 点名源码在 tests/<目标>/ 下、子模块里读判器缩域变量的目标、没缩域：拒」「cargo test 点名登记过的崩溃枚举目标、没缩域：拒」「cargo test 点名源码里读缩域变量的目标、没缩域：拒」「cargo test 点名登记过的放量分项目标、没缩域：拒」「cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量」「cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-prove-red-unjudged  证红脚本点名的变异行不判会不会走崩溃枚举
       红：「证红点名会走崩溃枚举的变异行、没缩域：拒」「证红点名的变异名不在变异表里：放行、记一处判不了」「证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树」「证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量」
   RUN_SCOPE_GUARD_BREAK=crash-domain-mutation-items-unjudged  变异门禁点名的行不判会不会走崩溃枚举
       红：「变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒」「变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒」「变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量」「变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个」「变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒」「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-registry-unread  不读登记表（stage-inputs.tsv 的 crash-case 行与崩溃放量入口的 ITEMS）：登记过的测试目标也当不走崩溃枚举
       红：「cargo test 点名登记过的崩溃枚举目标、没缩域：拒」「cargo test 点名登记过的放量分项目标、没缩域：拒」「证红点名会走崩溃枚举的变异行、没缩域：拒」「变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒」「变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒」「变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量」「变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」「变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒」「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-source-scan-skipped  不看测试源码里读没读缩域变量：没登记的测试目标一律当不走崩溃枚举
       红：「cargo test 点名源码在 tests/<目标>/ 下、子模块里读判器缩域变量的目标、没缩域：拒」「cargo test 点名源码里读缩域变量的目标、没缩域：拒」「cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量」「cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）」「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-shrink-ignored  命令里写了 --crash-expand-up-to、环境里设了缩域变量的都当没缩
       红：「崩溃点重放点名会走崩溃枚举的项、带 --crash-expand-up-to：放」「崩溃点重放点名会走崩溃枚举的项、命令前带放量缩域变量：放」「task-server 提交崩溃点重放点名项、-- 之后 env 带缩域变量：放」「task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放」「证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放」「变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放」「cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」
   RUN_SCOPE_GUARD_BREAK=crash-domain-reason-ignored  范围声明的为什么一段写了全域理由也照判
       红：「崩溃点重放点名会走崩溃枚举的项、为什么里写了全域理由：放」
   RUN_SCOPE_GUARD_BREAK=crash-domain-reason-length-unchecked  全域理由不核字数，写了就认
       红：「崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-full-gate-judged  交它的那一条 submit 带 --full-gate 的也照判
       红：「task-server 提交带 --full-gate、崩溃点重放点名项没缩域：这一条不判，放」
   RUN_SCOPE_GUARD_BREAK=crash-domain-unknown-unrecorded  判不出会不会走崩溃枚举、判不出读哪一个缩域变量的目标或项，放行时不记进判不了的清单
       红：「崩溃点重放点名的项不在合表里：放行、记一处判不了」「证红点名的变异名不在变异表里：放行、记一处判不了」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树」「证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量」
   RUN_SCOPE_GUARD_BREAK=crash-domain-target-variable-fixed  判点到的目标读哪个缩域变量不分目标，读得到源码的一律当判器那个（放行与拒绝说明的出路都照它）
       红：「变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量」「证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放」「变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放」「变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个」
   RUN_SCOPE_GUARD_BREAK=crash-domain-target-variable-source-unread  判点到的目标读哪个缩域变量不看测试源码，一律当放量那个
       红：「cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量」「task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放」「cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」
   RUN_SCOPE_GUARD_BREAK=crash-domain-variable-comments-counted  判点到的目标读哪个缩域变量时测试源码注释里写的变量名也算
       红：「task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放」「cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量」
   RUN_SCOPE_GUARD_BREAK=crash-domain-shrink-variable-unchecked  设了两个缩域变量之一就放，不核是不是点到的目标读的那一个
       红：「崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）」「cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）」「变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个」
   RUN_SCOPE_GUARD_BREAK=crash-domain-extra-variable-refused  多设了点到的目标不读的那一个缩域变量也拒（读的那一个设了也拒）
       红：「cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放」
   RUN_SCOPE_GUARD_BREAK=crash-domain-variable-unknown-refused  判不出点到的目标读哪一个缩域变量的，设了缩域变量也照没缩对拒
       红：「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」
   RUN_SCOPE_GUARD_BREAK=crash-domain-named-tree-unread  命令点明的树（证红 --copy 点名的副本、变异门禁的根参数）不读：变异表、登记表与测试源码只读命令当前目录那一棵与项目根
       红：「证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒」「证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树」「证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量」
   RUN_SCOPE_GUARD_BREAK=crash-domain-tree-fallback-dropped  只读排头那一棵树（命令点明的副本，没点明的取命令当前目录那一棵），里面没有的不退回别的树
       红：「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-tree-variables-unexpanded  `$名` 写的树（证红 --copy、变异门禁的根参数、submit 的 --cwd）不展开，当成路径里的字面字符
       红：「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」「证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量」
   RUN_SCOPE_GUARD_BREAK=unnamed-submissions-unjudged  经 task-server/task submit 交的那一条不判点没点名（不收 SubmittedRun，task_server_lib.unnamed_run_problem 不调）
       红：「task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）」「task-server 提交 cargo test 点名目标、不点名测试函数：拒」「task-server 提交证红不点名变异：拒」「task-server 提交门禁点起测试的格、不带 --item：拒」「入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）」
   RUN_SCOPE_GUARD_BREAK=unnamed-full-gate-dropped  判点没点名时不把 submit 带的 --full-gate 交给判据（提交时那一趟也照判）
       红：「task-server 提交带 --full-gate 的门禁整道、submit 前面带对的声明：放」「task-server 提交带 --full-gate、证红不点名变异：放（提交时那一趟不判点名）」
   RUN_SCOPE_GUARD_BREAK=unnamed-confirmed-dropped  判点没点名时不把 submit 带的 --whole-package-confirmed 交给判据（用户确认过的整包也照判）
       红：「提交带 --whole-package-confirmed：弹窗」「task-server 提交带 --whole-package-confirmed、证红不点名变异：放（用户确认过的整包不判点名）」
   RUN_SCOPE_GUARD_BREAK=sharded-wrapped-unjudged  交给服务的那一条里，套在驱动脚本或 bash -c 里的带分片标记的入口不判（不收 WrappedCandidate，task_server_lib.sharded_entry 不调）
       红：「task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）」「task-server 提交 bash -c、里面是带分片标记的入口：拒（套在 bash -c 里不分片）」「task-server 提交 bash -c 里带分片标记的入口、task_server_lib 载不进：放行、记一处判不了」「入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）」
   RUN_SCOPE_GUARD_BREAK=crash-domain-directory-sources-unread  判点到的目标读哪些测试源码时不认目录式目标（tests/<目标>/main.rs），只读 tests/<目标>.rs
       红：「cargo test 点名源码在 tests/<目标>/ 下、子模块里读判器缩域变量的目标、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-directory-main-only  目录式目标只读 main.rs，不读 tests/<目标>/ 底下别的 .rs（子模块里写的缩域变量名读不到）
       红：「cargo test 点名源码在 tests/<目标>/ 下、子模块里读判器缩域变量的目标、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-directory-targets-always-enumerate  目录式目标的源码文本末尾多一行放量缩域变量名（等于读了它），一律当会走崩溃枚举
       红：「cargo test 点名源码在 tests/<目标>/ 下、哪个文件都没读缩域变量的目标：不走崩溃枚举，放」
   RUN_SCOPE_GUARD_BREAK=part-incomplete-passes  部分文件编得过但缺入口要的名字（被截空）时照原样抛 ImportError（模块层引到被截掉的名字时抛 NameError；当入口脚本起退 1、当模块导入抛这个原来的错，不点名「部分文件」、不退 70、不抛 PartFileUnreadable）；只在自证起的子进程里认
       红：「入口：判法的部分文件截空：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable」
   RUN_SCOPE_GUARD_BREAK=part-unreadable-passes  读不到或编不过部分文件时照原样抛读文件错与语法错（当入口脚本起退 1、当模块导入抛原来的错，不点名「部分文件」、不退 70、不抛 PartFileUnreadable）；只在自证起的子进程里认
       红：「入口：判法的部分文件读不到：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable」「入口：判法的部分文件编不过：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable」
   开关与红的格以 lib_run_scope.py 的 BREAK_SWITCH_CELLS（并入 lib_run_scope_target_sources.py 的 BREAK_SWITCH_CELLS）为准，开关轮核每个开关名都写在这一段里。
"""
import os
import sys

# 同目录的部分模块按这一份所在的目录载入、不留在 sys.modules 的裸名字下：同一个进程里从两个目录各载入一次这一份（调用方按文件路径导入，自证拷一份目录再载入），
# 各用各目录里的部分模块；载入期间这个目录放在 sys.path 最前、裸名字摘出来（部分模块之间的 import 取到这一个目录里的），载入之后 sys.path 与裸名字放回载入之前的样子
_LIBRARY_DIRECTORY = os.path.realpath(os.path.dirname(os.path.abspath(__file__)))
_PART_MODULE_NAMES = ("lib_run_scope_bootstrap", "lib_run_scope_core", "lib_run_scope_selftest",)


def _begin_part_imports():
    """→ 载入之后要放回的现场：摘出来的同名模块。"""
    saved = {name: sys.modules.pop(name) for name in _PART_MODULE_NAMES if name in sys.modules}
    sys.path.insert(0, _LIBRARY_DIRECTORY)
    return saved


def _end_part_imports(saved):
    sys.path.remove(_LIBRARY_DIRECTORY)
    for name in _PART_MODULE_NAMES:
        sys.modules.pop(name, None)
    sys.modules.update(saved)


class PartFileUnreadable(ImportError):
    """这份模块的部分文件读不到、读不了、不是 UTF-8、语法错或含空字节，或编得过但不完整（入口要的名字缺了）：说明点名是哪一份。它是 ImportError，按文件路径导入这份模块的调用方照各自缺依赖（模块读不到）时的处置办。"""


def report_part_unusable(part_file_name, part_error):
    """一份部分文件载不全：当模块导入抛 PartFileUnreadable，当入口脚本起退 PART_UNREADABLE_STATUS，说明都点名是哪一份，不让导入时抛读文件错与语法错；
    读不到、编不过（OSError、SyntaxError、ValueError、模块读不到）与编得过而不完整（入口要的名字缺了：ImportError、NameError）分开处置，弄坏开关各管一类。
    HOOK_DIRECTORY、GUARD_SCRIPT_NAME、BREAK_VARIABLE_NAME、RUN_SCOPE_PART_NAMES、PART_UNREADABLE_STATUS、selftest_marked 是下面先 import 的引导要的名字，调用的时候已经在了。"""
    part_path = os.path.join(HOOK_DIRECTORY, part_file_name)
    incomplete = isinstance(part_error, (ImportError, NameError)) and not isinstance(part_error, ModuleNotFoundError)
    if selftest_marked() and ("part-incomplete-passes" if incomplete else "part-unreadable-passes") in os.environ.get(BREAK_VARIABLE_NAME, "").split(","):
        raise part_error   # 弄坏开关 part-unreadable-passes、part-incomplete-passes：照原样抛原来的错（只在自证起的子进程里认）
    if incomplete:
        part_problem = (f"{GUARD_SCRIPT_NAME} 的判法 lib_run_scope.py 的部分文件编得过但不完整，入口要的名字缺了，没有哪一份部分文件定义它（{type(part_error).__name__}：{part_error}）："
                        + "；".join(f"部分文件 {os.path.join(HOOK_DIRECTORY, name)}" for name in RUN_SCOPE_PART_NAMES))
    else:
        part_problem = f"{GUARD_SCRIPT_NAME} 的判法 lib_run_scope.py 读不到或编不过部分文件 {part_path}（{type(part_error).__name__}：{part_error}）"
    if __name__ == "__main__":
        print(f"  ✗ {part_problem}，这条命令的范围声明没判", file=sys.stderr)
        sys.exit(PART_UNREADABLE_STATUS)
    raise PartFileUnreadable(part_problem) from part_error


_saved_modules = _begin_part_imports()
# 引导要的名字这一份读不到时没法点名是哪一份，直接抛原来的错（这一份没载进来：调用方整份丢掉这次导入）
from lib_run_scope_bootstrap import (
    BREAK_VARIABLE_NAME, GUARD_SCRIPT_NAME, HOOK_DIRECTORY, PART_UNREADABLE_STATUS, RUN_SCOPE_BOOTSTRAP_FILE_NAME, RUN_SCOPE_PART_NAMES,
    SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE, selftest_marked,
)
try:
    try:
        from lib_run_scope_core import (
            AMPLIFICATION_ITEM_LINE, BREAK_SWITCH_CELLS, CARGO_EVERY_TARGET_FLAGS, CARGO_NAMED_TARGET_OPTIONS, CARGO_SINGLE_TARGET_FLAGS,
            CARGO_WHOLE_WORKSPACE_FLAGS, COMMIT_GATE_STAGE_NAMES, COMMIT_PROBLEM_PREFIX, COMMIT_WHOLE_REASON, CRASH_AMPLIFICATION_ENTRY,
            CRASH_AMPLIFICATION_QUERY_ARGUMENTS, CRASH_CASE_TEST_WORD, CRASH_DRIVER_OPTIONS_WITH_VALUE, CRASH_REPLAY_QUERY_ARGUMENTS,
            CRASH_SHRINK_OPTION, CRASH_SHRINK_VARIABLES, CRASH_SUBJECT_BREAK_SWITCHES, CRASH_SUBJECT_DESCRIPTIONS, CrashDomains, CrashSubject,
            DECLARATION_RULE_LINES, DECLARATION_VARIABLE, DETECTIONS_VARIABLE, FULL_DOMAIN_REASON_MINIMUM_CHARACTERS, FULL_DOMAIN_REASON_PATTERN,
            FoundRun, GATE_WHOLE_LOCATIONS, HOOK_INPUT_HEAD_BYTES, KIND_DESCRIPTIONS, LAST_RUNS_ENTRY, LAST_RUNS_VARIABLE, LAUNCHER_PROGRAMS,
            LEADING_DECLARATION, LOOSENING_SUBMIT_FLAGS, MAXIMUM_SCRIPT_BYTES, MAXIMUM_SCRIPT_DEPTH, MERGE_OPTIONS_WITH_VALUE, MERGE_QUERY_ARGUMENTS,
            MUTATION_TABLE, NARROWING_LINES, REASON_FILLER_WORDS, REASON_MINIMUM_CHARACTERS, REPLAY_ITEMS_TABLE, REPLAY_QUERY_ARGUMENTS,
            RUST_BLOCK_COMMENT_EDGE, RUST_LEXICAL_START, RUST_STRING_LITERAL, RunShape, SCOPE_KINDS, SCRIPT_HEAD_BYTES, SEGMENT_KEYS, SEGMENT_START,
            SELFTEST_ONLY_QUERY_ARGUMENTS, STAGE_INPUTS_TABLE, STAGE_QUERY_ARGUMENTS, SUBMIT_FLAG_USER_RULE, ScanState, ScopeDecision,
            ScopeDeclaration, SubmitFlagAsk, SubmittedRun, TEMPLATE_COMMAND_LIMIT, UNREADABLE_PATH_CHARACTERS, UNSCOPED_COMMIT_ENTRIES,
            VERIFICATION_PACKAGE, VERIFICATION_TESTS_DIRECTORY, WHOLE_RANGE_WORDS, WrappedCandidate, append_detection, ask_user_about_submit_flags,
            candidate_command_words, cargo_run_shape, commit_gate_problem, consulted_trees, count_basis, counted_text, crash_domain_problem,
            crash_domain_trees, crash_run_shape, crash_shrink_option_written, crash_shrink_variables_written, crash_subject_domains,
            crash_subject_kind, crash_target_enumerates, decide_scope, declaration_to_write, detections_path_now, direct_submission_command,
            driver_script_text, failure_recording_dropped, flag_asks_by_mention, frames_description, full_domain_reason_written, guard_switch_on,
            hook_main, interpreter_of_head, known_tree_word, last_red_lines, last_runs_path, load_judging_libraries, merge_patch_directories,
            merge_run_shape, mutation_item_domains, mutation_row_targets, named_option_values, note_unjudged, parse_scope_declaration, query_form,
            read_text_lines, reason_character_count, record_scope_decision, refusal_text, refuse_break_outside_selftest, registered_crash_targets,
            registry_root, remember_heredoc_writes, replay_item_domains, replay_run_shape, repository_tooling_script, run_shape_of,
            rust_code_without_comments, scope_problems, scope_runs_in_text, script_command_line_numbers, script_path_word, sharded_wrapped_entry,
            sharded_wrapped_lines, stage_positional_words, stage_run_shape, submit_client_command, submit_flag_ask, submit_flag_asks,
            submit_flag_reason, submit_loosening_flags, submit_option_pieces, submit_words_with_known_directory, target_domains,
            target_shrink_variables, template_line, texts_launched_by, unnamed_submission_problem, variable_built_hint, variable_built_words,
            verification_test_targets, without_force,
        )
    except (ImportError, NameError, OSError, SyntaxError, ValueError) as part_error:
        report_part_unusable("lib_run_scope_core.py", part_error)
    try:
        from lib_run_scope_selftest import (
            build_selftest_tree, failed_labels, scope_text, selftest, selftest_cells, selftest_entry_cells, selftest_judge_cells, submit_flag_cells,
            submit_flag_summary,
        )
    except (ImportError, NameError, OSError, SyntaxError, ValueError) as part_error:
        report_part_unusable("lib_run_scope_selftest.py", part_error)
finally:
    _end_part_imports(_saved_modules)


def main():
    hook_directory = sys.argv[1] if len(sys.argv) > 1 else HOOK_DIRECTORY
    refusal = refuse_break_outside_selftest(sys.argv[2:])
    if refusal:
        for line in refusal:
            print(line, file=sys.stderr)
        return 2
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_directory)
    return hook_main(hook_directory)


if __name__ == "__main__":
    sys.exit(main())
