"""run-scope-guard.sh 的判法：起验证脚本的命令要带范围声明 SINGLEFS_RUN_SCOPE，种类与范围要与命令的形状对得上；提交带放宽检查的标识的要用户确认。

认法、要声明与不要声明的形态、拒绝说明的样子、要用户确认的写法、检出记录、罩不住的写法与弄坏开关都写在 run-scope-guard.sh 的文件头，这里不另抄。
切词、剥前缀与包装用同目录的 lib_shell_words.py；剥 flock、/usr/bin/time、gpu-quiesce-run.sh 这一类，
认门禁阶段、认 cargo 的子命令用 lib_project_code_runs.py（按文件路径导入，不另写一份）。
用法：python3 lib_run_scope.py <钩子目录> [--selftest]，由 run-scope-guard.sh 起；hook 的 JSON 从标准输入读。
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
       红：「驱动脚本里一行缺声明拒」「外面带声明、驱动脚本里一行缺声明拒」「驱动脚本再起驱动脚本、里面缺声明拒」「直接执行的驱动脚本缺声明拒」「同一条命令里 heredoc 写出的驱动脚本缺声明拒」「入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）」「驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」
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
       红：「cargo test 两个目标、带对的声明：放」「cargo test --lib --exact 两个测试、带对的声明：放」「子 agent cargo test 点名目标、带对的声明：放」「capped.sh 外面带声明放」「驱动脚本里每一行都带声明：放」「驱动脚本路径写成赋过字面值的变量、里面每一行都带声明：放」「cargo test 点名不走崩溃枚举的目标：放」「task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放」「cargo test 点名别的包的目标：这一条不判，放」「cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「task-server 提交 cargo test 点名 --exact 的测试函数：放」
   RUN_SCOPE_GUARD_BREAK=template-keeps-declaration  照抄模板时不摘命令开头已有的那份声明（照抄出来两份，bash 取后一份）
       红：「入口：命令开头已有声明时模板只留一份」
   RUN_SCOPE_GUARD_BREAK=failure-unrecorded  判不了的几样（hook 输入不是 JSON 对象、判的时候出异常、喂给 shell 的不是字面、读不到判法文件）只打 stderr 或什么都不说、不记检出
       红：「入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出」「入口：判的时候抛异常放行、stderr 报一句、记一条检出」「入口：here-string 喂给 bash 的是变量放行、记一条检出」「入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）」「入口：判法文件读不到时记一条检出（结局只记、会话号照输入）」「入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置」
   RUN_SCOPE_GUARD_BREAK=judge-crash-unrecorded  判法文件在而没跑完（导入就出错、被信号停掉，退 0 与 2 以外的码）时只打 stderr、不记检出（exec 判法，退几都照常放行）
       红：「入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）」
   RUN_SCOPE_GUARD_BREAK=driver-unread-unrecorded  读不了、太大的驱动脚本静默不读、不交判不了的说明
       红：「入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）」
   切词那一份的开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread（lib_shell_words.py：here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明）下也红，不在开关轮里：
       红：「here-string 喂给 bash 的 cargo test 整工作区：没带声明拒」「printf 经管道喂给 bash 的门禁：没带声明拒」「echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒」「入口：here-string 喂给 bash 的是变量放行、记一条检出」
   RUN_SCOPE_GUARD_BREAK=honor-outside-selftest  平常跑（不是 --selftest、也不是自证起的子进程）设了开关也照认、不拒；它是关守卫项，只在单独一项时认，与别的项同写照拒
       红：「入口：平常跑设了弄坏开关退 2、点名它只在自证里认」「入口：标记文件内容对不上照拒」
   RUN_SCOPE_GUARD_BREAK=honor-outside-selftest-with-others  带标记的自证子进程里，关守卫项与别的项同写也照认（honor-outside-selftest 写在哪一项都认；只在带标记的子进程里认，平常跑同写照拒）
       红：「入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它」
   RUN_SCOPE_GUARD_BREAK=task-server-submissions-unread  task-server/task submit -- 之后那一条不判（当名单上的客户端放过去）
       红：「task-server 提交的门禁整道：没带声明拒」「task-server 提交的 cargo test 整工作区：没带声明拒」「task-server 提交带 --full-gate 的门禁整道：没带声明拒」「task-server 提交带 --manual 的门禁整道：没带声明拒」「task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒」「子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）」「子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）」「入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回」「入口：命令开头已有声明时模板只留一份」「task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）」「task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒」「task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒」「task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）」「task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）」「入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」「task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒」「task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）」「task-server 提交 cargo test 点名目标、不点名测试函数：拒」「task-server 提交证红不点名变异：拒」「task-server 提交门禁点起测试的格、不带 --item：拒」「入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）」
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
       红：「cargo test 点名登记过的崩溃枚举目标、没缩域：拒」「cargo test 点名源码里读缩域变量的目标、没缩域：拒」「cargo test 点名登记过的放量分项目标、没缩域：拒」「cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量」「cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-prove-red-unjudged  证红脚本点名的变异行不判会不会走崩溃枚举
       红：「证红点名会走崩溃枚举的变异行、没缩域：拒」「证红点名的变异名不在变异表里：放行、记一处判不了」「证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树」「证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量」
   RUN_SCOPE_GUARD_BREAK=crash-domain-mutation-items-unjudged  变异门禁点名的行不判会不会走崩溃枚举
       红：「变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒」「变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒」「变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量」「变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个」「变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒」「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-registry-unread  不读登记表（stage-inputs.tsv 的 crash-case 行与崩溃放量入口的 ITEMS）：登记过的测试目标也当不走崩溃枚举
       红：「cargo test 点名登记过的崩溃枚举目标、没缩域：拒」「cargo test 点名登记过的放量分项目标、没缩域：拒」「证红点名会走崩溃枚举的变异行、没缩域：拒」「变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒」「变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒」「变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量」「变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个」「cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了」「证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒」「task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒」「task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒」「变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒」「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
   RUN_SCOPE_GUARD_BREAK=crash-domain-source-scan-skipped  不看测试源码里读没读缩域变量：没登记的测试目标一律当不走崩溃枚举
       红：「cargo test 点名源码里读缩域变量的目标、没缩域：拒」「cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量」「cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）」「变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒」
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
   开关与红的格以 lib_run_scope.py 的 BREAK_SWITCH_CELLS 为准，开关轮核每个开关名都写在这一段里。
"""
import importlib.util
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import unicodedata
from datetime import datetime, timezone
from typing import NamedTuple

sys.dont_write_bytecode = True

HOOK_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
GUARD_SCRIPT_NAME = "run-scope-guard.sh"
DECLARATION_VARIABLE = "SINGLEFS_RUN_SCOPE"
SCOPE_KINDS = ("whole", "items", "target")
SEGMENT_KEYS = ("种类", "跑什么", "范围", "为什么")
SEGMENT_START = re.compile(r"^(种类|跑什么|范围|为什么)\s*=\s*(.*)$", re.S)
REASON_MINIMUM_CHARACTERS = 12
# 不说理由的词：为什么一段里去掉它们再数字数
REASON_FILLER_WORDS = ("确认一下", "跑一下", "跑一遍", "要跑", "验证", "看看")
COMMIT_WHOLE_REASON = "提交前整轮"
WHOLE_RANGE_WORDS = ("整轮", "整包")
KIND_DESCRIPTIONS = {"whole": "整轮（whole）", "items": "点名的几项（items）", "target": "点名的测试目标（target）"}
LAST_RUNS_VARIABLE = "SINGLEFS_LAST_RUNS_DIRECTORY"
LAST_RUNS_ENTRY = "last-runs"   # 没设 SINGLEFS_LAST_RUNS_DIRECTORY 时取数据根下的这一项（同目录 lib_work_root.py 的 shared_data_path，两台读同一份）
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"
MAXIMUM_SCRIPT_DEPTH = 6
MAXIMUM_SCRIPT_BYTES = 1 << 20
SCRIPT_HEAD_BYTES = 4096
TEMPLATE_COMMAND_LIMIT = 2000
# 模板照抄整条命令时先摘掉命令开头已有的那份声明（不然照抄出来是两份，bash 取后一份，等于没改）
LEADING_DECLARATION = re.compile(r"\A\s*" + DECLARATION_VARIABLE + r"=(?:'[^']*'|\"(?:[^\"\\]|\\.)*\"|\S+)[ \t]+")
# 命令位置上的词里带这些字符的不当脚本路径去读（变量、命令替换、通配、引号残片、重定向）
UNREADABLE_PATH_CHARACTERS = set("$`*?[]{}()<>|;&'\"=,")
# 门禁阶段只打清单、自证、帮助、工具版本的参数：带了就不要声明
STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items", "--selftest", "--help", "-h", "--tool-versions"})
CRASH_REPLAY_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
CRASH_AMPLIFICATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
SELFTEST_ONLY_QUERY_ARGUMENTS = frozenset({"--selftest", "--help", "-h"})
MERGE_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
REPLAY_QUERY_ARGUMENTS = frozenset({"--selftest", "--admission-only", "--list-items", "--help", "-h"})
# 崩溃放量的分项驱动与双机驱动带一个值的选项（照两份的用法行：分项驱动 --test-filter、--label，双机驱动 --test-filter、--label、--skip）
CRASH_DRIVER_OPTIONS_WITH_VALUE = frozenset({"--test-filter", "--label", "--skip"})
CARGO_NAMED_TARGET_OPTIONS = ("--test", "--bin", "--example", "--bench")
CARGO_SINGLE_TARGET_FLAGS = ("--lib", "--doc")
CARGO_EVERY_TARGET_FLAGS = frozenset({"--tests", "--bins", "--examples", "--benches", "--all-targets"})
CARGO_WHOLE_WORKSPACE_FLAGS = frozenset({"--workspace", "--all"})
GATE_WHOLE_LOCATIONS = (".claude/singlefs-ai-sop/scripts", ".claude/scripts")
# 放宽检查的提交标识（task-server/task submit 的选项）→ 给用户看的作用；判法在 submit_flag_asks
LOOSENING_SUBMIT_FLAGS = {"--full-gate": "免两小时提醒", "--whole-package-confirmed": "整包测试放行", "promote": "把在排的任务提前到队头"}
SUBMIT_FLAG_USER_RULE = "用户定了对话不许自己用这种标识：要用必须用户在这里点同意，不同意就拒绝这一条。"
# 提交时才许不指定范围地跑的六道门禁（.claude/skills/crash-test/commit-stages.md 开头点名的，task-server/task_server_lib.py 的 COMMIT_GATE_ENTRIES 同一组）
# 与崩溃点重放、崩溃放量两份入口（同文件 UNSCOPED_RUN_ENTRIES）：不带选条参数的这一形经 task-server/task submit 交时，`--` 之前要带放宽标识（LOOSENING_SUBMIT_FLAGS）
COMMIT_GATE_STAGE_NAMES = ("crash-point-replay.sh", "qemu-device-streams.sh", "lkmm.sh", "crates-mutation-replay.sh", "formal-verification.sh", "broad-mutation.sh")
UNSCOPED_COMMIT_ENTRIES = tuple(f".claude/gate.d/{name}" for name in COMMIT_GATE_STAGE_NAMES) + ("scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh")
COMMIT_PROBLEM_PREFIX = "提交时才许跑："   # 这一类说明不是声明写得不对，拒绝说明里不附声明模板
# 指定范围的那几趟崩溃状态数压在 20 万量级（.claude/rules/verification.md「一个门禁：两种跑法」）：按登记表与测试源码判点到的目标或项会不会走崩溃枚举、读哪一个缩域变量，
#   命令里写了 --crash-expand-up-to 或环境里设了它读的那一个的放（crash_domain_problem）
CRASH_SHRINK_OPTION = "--crash-expand-up-to"
CRASH_SHRINK_VARIABLES = ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO", "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")
FULL_DOMAIN_REASON_PATTERN = re.compile(r"全域理由\s*[：:]\s*(.*)", re.S)   # 为什么一段里写明确要全域的理由
FULL_DOMAIN_REASON_MINIMUM_CHARACTERS = 12
STAGE_INPUTS_TABLE = ".claude/gate.d/stage-inputs.tsv"
REPLAY_ITEMS_TABLE = ".claude/gate.d/crash-point-replay-items.tsv"
CRASH_AMPLIFICATION_ENTRY = "scripts/crash/crash-amplification.sh"
MUTATION_TABLE = "crates/mutations.tsv"
VERIFICATION_PACKAGE = "singlefs-verification"
VERIFICATION_TESTS_DIRECTORY = "crates/singlefs-verification/tests"
CRASH_CASE_TEST_WORD = re.compile(r"(?:^|\s)test=" + re.escape(VERIFICATION_PACKAGE) + r":(\w+):\w+")   # stage-inputs.tsv 的 crash-case: 行里 test=<包>:<目标>:<用例>
AMPLIFICATION_ITEM_LINE = re.compile(r"^\s*\$'[a-z0-9][a-z0-9-]*\\t(\w+)\\t\w+'\s*$")   # crash-amplification.sh 的 ITEMS 里一行：$'分项\t测试目标\t用例'
# Rust 源码里注释、字符串与字符字面量的起头（rust_code_without_comments）：// 与 /* 起注释，r"、r#" 起原始字符串，" 起字符串，'x'、'\n'、'\u{…}' 是字符字面量（别的 ' 是生命周期）
RUST_LEXICAL_START = re.compile(r"""//|/\*|\br(#*)"|"|'(?:\\u\{[0-9a-fA-F]{1,6}\}|\\.|[^\\'\n])'""")
RUST_BLOCK_COMMENT_EDGE = re.compile(r"/\*|\*/")
RUST_STRING_LITERAL = re.compile(r'"(?:\\.|[^"\\])*"', re.S)
CRASH_SUBJECT_DESCRIPTIONS = {"replay-items": "崩溃点重放入口点名的项", "cargo-targets": "cargo test 点名的验证包测试目标",
                              "prove-red": "证红脚本点名的变异行", "mutation-items": "变异门禁点名的行"}
CRASH_SUBJECT_BREAK_SWITCHES = {"replay-items": "crash-domain-replay-items-unjudged", "cargo-targets": "crash-domain-cargo-targets-unjudged",
                                "prove-red": "crash-domain-prove-red-unjudged", "mutation-items": "crash-domain-mutation-items-unjudged"}

BREAK_VARIABLE_NAME = "RUN_SCOPE_GUARD_BREAK"
# 弄坏开关 → 打开后红的自证格，写全（文件头「弄坏开关」一段逐项照这张表写；开关轮核红的格与这里逐格相等，并核每个开关名都写在 run-scope-guard.sh 文件头里）
BREAK_SWITCH_CELLS = {
    "declaration-optional": ["task-server 提交带 --full-gate 的门禁整道：没带声明拒", "task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒", "门禁整道：没带声明拒", "崩溃点重放不指定范围跑：没带声明拒", "崩溃放量全部分项：没带声明拒", "崩溃放量双机驱动：没带声明拒", "replay.sh 全部实验：没带声明拒", "gate-staged.sh 没带声明拒", "cargo test 整工作区：没带声明拒", "bash -c 里的门禁没带声明拒", "here-string 喂给 bash 的 cargo test 整工作区：没带声明拒", "printf 经管道喂给 bash 的门禁：没带声明拒", "echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒", "capped.sh 包着的 cargo test 没带声明拒", "env 带别的变量、没带声明拒", "nice 与 timeout 包着、没带声明拒", "flock 包着的门禁没带声明拒", "gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", "/usr/bin/time 包着的 cargo test 没带声明拒", "驱动脚本里一行缺声明拒", "外面带声明、驱动脚本里一行缺声明拒", "驱动脚本再起驱动脚本、里面缺声明拒", "直接执行的驱动脚本缺声明拒", "同一条命令里 heredoc 写出的驱动脚本缺声明拒", "入口：没带声明拒（退出码 2）", "入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法", "入口：拒绝说明附上次红的清单", "入口：没有上次红的记录时照写", "入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）", "驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒"],
    "segments-unchecked": ["声明缺段：没写为什么拒", "声明值空：范围一段是空的拒", "声明写两次：种类写两次拒", "声明种类认不出拒"],
    "kind-unchecked": ["门禁 --item 声明成 whole：种类对不上拒", "merge-and-verify 声明成 whole：拒", "cargo test 整包声明成 target：种类对不上拒", "声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面"],
    "count-unchecked": ["门禁 --check 两格写成 1 格：范围数对不上拒", "whole 范围没写整轮或整包拒", "cargo test 两个目标写成 3 个：范围数对不上拒", "cargo test --exact 两个测试写成 1 个：范围数对不上拒", "崩溃点重放点名一项另加展开深度：写成两项拒（深度的值不是项）", "merge-and-verify 点名两项写成 1 项：范围数对不上拒",
                        "merge-and-verify 两份补丁写成 1 份：范围数对不上拒", "范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法", "范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验"],
    "reason-unchecked": ["gate.sh 为什么只写验证拒", "为什么太短拒", "为什么只写要跑验证拒"],
    "subagent-submissions-allowed": ["子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）", "子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）",
                                     "入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回", "入口：命令开头已有声明时模板只留一份"],
    "scripts-unread": ["驱动脚本里一行缺声明拒", "外面带声明、驱动脚本里一行缺声明拒", "驱动脚本再起驱动脚本、里面缺声明拒", "直接执行的驱动脚本缺声明拒", "同一条命令里 heredoc 写出的驱动脚本缺声明拒",
                       "入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）", "驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒"],
    "wrappers-opaque": ["flock 包着的门禁没带声明拒", "gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", "/usr/bin/time 包着的 cargo test 没带声明拒"],
    "allowed-unrecorded": ["入口：放行的记进检出记录（四段与命令）"],
    "last-runs-unread": ["入口：拒绝说明附上次红的清单"],
    "query-forms-declared": ["门禁 --list 不要声明", "门禁 --selftest 不要声明", "崩溃点重放 --list-items 不要声明", "replay.sh --admission-only 不要声明", "replay.sh --list-items 不要声明", "cargo test --no-run 不要声明", "cargo test -- --list 不要声明", "merge-and-verify --list-items 不要声明"],
    "replay-list-items-declared": ["replay.sh --list-items 不要声明"],
    "merge-as-whole": ["merge-and-verify 声明成 whole：拒", "merge-and-verify 两份补丁：声明成 items 放", "merge-and-verify 两份补丁另加展开深度：写成 2 份补丁放（深度的值不是补丁）",
                       "merge-and-verify 点名两项：声明成 items 放", "task-server 提交合入后验证带 --item：放"],
    "merge-list-items-declared": ["merge-and-verify --list-items 不要声明"],
    "tooling-scripts-read": ["仓里的工具脚本不读：放行"],
    "cargo-targets-as-whole": ["cargo test 两个目标、带对的声明：放", "cargo test --lib --exact 两个测试、带对的声明：放", "子 agent cargo test 点名目标、带对的声明：放", "capped.sh 外面带声明放", "驱动脚本里每一行都带声明：放", "驱动脚本路径写成赋过字面值的变量、里面每一行都带声明：放",
                              "cargo test 点名不走崩溃枚举的目标：放", "task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", "cargo test 点名别的包的目标：这一条不判，放",
                              "cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了",
                              "task-server 提交 cargo test 点名 --exact 的测试函数：放"],
    "template-keeps-declaration": ["入口：命令开头已有声明时模板只留一份"],
    "failure-unrecorded": ["入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", "入口：判的时候抛异常放行、stderr 报一句、记一条检出", "入口：here-string 喂给 bash 的是变量放行、记一条检出",
                           "入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）",
                           "入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", "入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置"],
    "judge-crash-unrecorded": ["入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）"],
    "driver-unread-unrecorded": ["入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）"],
    "honor-outside-selftest": ["入口：平常跑设了弄坏开关退 2、点名它只在自证里认", "入口：标记文件内容对不上照拒"],
    "honor-outside-selftest-with-others": ["入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它"],
    "task-server-submissions-unread": ["task-server 提交的门禁整道：没带声明拒", "task-server 提交的 cargo test 整工作区：没带声明拒",
                                       "task-server 提交带 --full-gate 的门禁整道：没带声明拒", "task-server 提交带 --manual 的门禁整道：没带声明拒",
                                       "task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒",
                                       "子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）", "子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）",
                                       "入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回", "入口：命令开头已有声明时模板只留一份",
                                       "task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒",
                                       "task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒", "task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）",
                                       "入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒",
                                       "task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）", "task-server 提交 cargo test 点名目标、不点名测试函数：拒", "task-server 提交证红不点名变异：拒",
                                       "task-server 提交门禁点起测试的格、不带 --item：拒", "入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）"],
    "submit-flags-unjudged": ["提交带 --full-gate：弹窗", "提交带 --whole-package-confirmed：弹窗", "范围声明写在前面：弹窗", "bash -c 里面：弹窗", "&& 连着的后一段：弹窗", "--cwd 在标识前面：弹窗", "两个标识都带：都点名",
                              "python3 起客户端：弹窗", "cd 进 task-server 再起：弹窗", "cd 到算不出的目录再起：弹窗", "cd 到算不出的目录再起 ./task：弹窗", "eval 字面代码：弹窗", "赋过字面值的变量写在参数上：认出标识", "数组写在参数上：认出标识", "标识写成等号带值：弹窗",
                              "参数是变量：弹窗认不出", "数组全部元素写在参数上：弹窗认不出", "eval 给出的参数是变量：弹窗认不出", "子命令位是变量：弹窗认不出", "参数是命令替换：弹窗认不出", "xargs 读进来的参数：弹窗认不出", "flock 包着：弹窗", "python3 -c 里字面写着：弹窗",
                              "驱动脚本里：弹窗", "交给服务的那一条里又提交：弹窗", "脚本里的转发函数：弹窗", "命令词是赋过字面值的变量：弹窗", "eval 赋过字面值的变量：弹窗", "判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了",
                              "入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）", "入口：参数是变量弹窗、理由写认不出带没带标识"],
    "submit-flags-by-substring": ["grep 这个词：不弹窗", "commit message 带这个词：不弹窗", "--why 里提到这个词：不弹窗", "交给服务的那一条里提到这个词：不弹窗", "echo 这条命令的文字：不弹窗", "范围声明的值里提到这个词：不弹窗"],
    "submit-flag-variables-unread": ["参数是变量：弹窗认不出", "数组全部元素写在参数上：弹窗认不出", "eval 给出的参数是变量：弹窗认不出", "子命令位是变量：弹窗认不出", "参数是命令替换：弹窗认不出", "xargs 读进来的参数：弹窗认不出", "入口：参数是变量弹窗、理由写认不出带没带标识"],
    "submit-flag-after-separator": ["交给服务的那一条里提到这个词：不弹窗", "交给服务的那一条参数是变量：不弹窗", "xargs 读进来的落在交给服务的那一条里：不弹窗"],
    "submit-flag-first-option-only": ["--cwd 在标识前面：弹窗", "两个标识都带：都点名"],
    "submit-flag-client-by-path-only": ["cd 到算不出的目录再起：弹窗", "cd 到算不出的目录再起 ./task：弹窗"],
    "submit-flag-nesting-unread": ["xargs 读进来的参数：弹窗认不出", "flock 包着：弹窗", "python3 -c 里字面写着：弹窗", "驱动脚本里：弹窗", "交给服务的那一条里又提交：弹窗", "脚本里的转发函数：弹窗", "命令词是赋过字面值的变量：弹窗", "eval 赋过字面值的变量：弹窗"],
    "submit-flag-subagents-asked": ["子 agent 带标识：不弹窗（由项目代码闸拒）"],
    "submit-flag-raise-propagated": ["判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了"],
    "submit-flag-ask-silent": ["入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）", "入口：参数是变量弹窗、理由写认不出带没带标识", "入口：判的时候抛异常、命令文字里提到标识：弹窗"],
    "commit-whole-flag-unchecked": ["task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒",
                                    "task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒", "task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）",
                                    "入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）"],
    "crash-domain-replay-items-unjudged": ["崩溃点重放点名会走崩溃枚举的项、没缩域：拒", "崩溃点重放点名放量分项的项、没缩域：拒", "崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒", "入口：崩溃域没缩拒（退出码 2、说明写缩域变量与全域理由的写法）", "崩溃点重放点名的项不在合表里：放行、记一处判不了",
                                           "崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）"],
    "crash-domain-cargo-targets-unjudged": ["cargo test 点名登记过的崩溃枚举目标、没缩域：拒", "cargo test 点名源码里读缩域变量的目标、没缩域：拒", "cargo test 点名登记过的放量分项目标、没缩域：拒", "cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量",
                                            "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒"],
    "crash-domain-prove-red-unjudged": ["证红点名会走崩溃枚举的变异行、没缩域：拒", "证红点名的变异名不在变异表里：放行、记一处判不了", "证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "crash-domain-mutation-items-unjudged": ["变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒", "变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒", "变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个", "变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", "变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-registry-unread": ["cargo test 点名登记过的崩溃枚举目标、没缩域：拒", "cargo test 点名登记过的放量分项目标、没缩域：拒", "证红点名会走崩溃枚举的变异行、没缩域：拒", "变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒", "变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒",
                                     "变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", "证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", "变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", "变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-source-scan-skipped": ["cargo test 点名源码里读缩域变量的目标、没缩域：拒", "cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量", "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）",
                                        "变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-shrink-ignored": ["崩溃点重放点名会走崩溃枚举的项、带 --crash-expand-up-to：放", "崩溃点重放点名会走崩溃枚举的项、命令前带放量缩域变量：放", "task-server 提交崩溃点重放点名项、-- 之后 env 带缩域变量：放",
                                    "task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", "证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放", "变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放",
                                    "cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了"],
    "crash-domain-reason-ignored": ["崩溃点重放点名会走崩溃枚举的项、为什么里写了全域理由：放"],
    "crash-domain-reason-length-unchecked": ["崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒"],
    "crash-domain-full-gate-judged": ["task-server 提交带 --full-gate、崩溃点重放点名项没缩域：这一条不判，放"],
    "crash-domain-unknown-unrecorded": ["崩溃点重放点名的项不在合表里：放行、记一处判不了", "证红点名的变异名不在变异表里：放行、记一处判不了", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", "证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "crash-domain-target-variable-fixed": ["变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", "证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放", "变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个"],
    "crash-domain-target-variable-source-unread": ["cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量", "task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放",
                                                   "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了"],
    "crash-domain-variable-comments-counted": ["task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", "cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量"],
    "crash-domain-shrink-variable-unchecked": ["崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）", "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个"],
    "crash-domain-extra-variable-refused": ["cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放"],
    "crash-domain-variable-unknown-refused": ["cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了"],
    "count-hint-unprinted": ["范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法", "范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验", "声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面"],
    "variable-built-hint-dropped": ["声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面"],
    "driver-path-variables-unexpanded": ["驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒"],
    "crash-domain-named-tree-unread": ["证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", "证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "crash-domain-tree-fallback-dropped": ["变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-tree-variables-unexpanded": ["task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "unnamed-submissions-unjudged": ["task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）", "task-server 提交 cargo test 点名目标、不点名测试函数：拒", "task-server 提交证红不点名变异：拒",
                                     "task-server 提交门禁点起测试的格、不带 --item：拒", "入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）"],
    "unnamed-full-gate-dropped": ["task-server 提交带 --full-gate 的门禁整道、submit 前面带对的声明：放", "task-server 提交带 --full-gate、证红不点名变异：放（提交时那一趟不判点名）"],
    "unnamed-confirmed-dropped": ["提交带 --whole-package-confirmed：弹窗", "task-server 提交带 --whole-package-confirmed、证红不点名变异：放（用户确认过的整包不判点名）"],
}
# 弄坏开关只在自证里认：自证起的入口靠这个变量指到自证建的标记文件、内容逐字是 SELFTEST_MARK_CONTENT 才认开关（同一句写在 run-scope-guard.sh 那一段守卫里）
SELFTEST_MARK_VARIABLE = "RUN_SCOPE_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "run-scope-guard.sh --selftest 起的子进程"

project_code_runs = None
shell_words = None


def guard_switch_on(name):
    """弄坏开关 RUN_SCOPE_GUARD_BREAK 里有没有这一项；有不认识的项抛 ValueError（免得拼错的开关被当成没开）。"""
    switches = [item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item]
    unknown = [item for item in switches if item not in BREAK_SWITCH_CELLS]
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE_NAME} 里有认不出的项 {'、'.join(unknown)}，只认 {'、'.join(BREAK_SWITCH_CELLS)}")
    return name in switches


def load_judging_libraries():
    """按文件路径导入这份库同目录的 lib_project_code_runs.py（它再导入同目录的 lib_shell_words.py）。读不到或导入出错照抛，由调用方放行并报一句。"""
    global project_code_runs, shell_words
    if project_code_runs is None:
        spec = importlib.util.spec_from_file_location("lib_project_code_runs", os.path.join(HOOK_DIRECTORY, "lib_project_code_runs.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        project_code_runs, shell_words = module, module.shell_words
    return project_code_runs


class RunShape(NamedTuple):
    kind: str            # whole / items / target
    what: str            # 这条命令跑的是什么（给 agent 看的一句）
    count: int | None    # items 与 target 的 N
    unit: str            # N 的单位：格、项、个实验、个测试目标、个测试
    range_hint: str      # 模板里范围一段现算好的写法
    script_key: str | None   # 上次红的清单按它找（脚本相对仓根的路径）；cargo test 没有
    family: str          # stage、crash、replay、merge、gate-whole、cargo


class FoundRun(NamedTuple):
    frames: tuple        # 读到它经过的驱动脚本：((脚本路径, 行号或 None), …)；命令文本里直接写的是 ()
    words: list          # 剥掉前缀与包装之后的那一条
    shape: RunShape
    declaration: str | None   # 这一条看得到的 SINGLEFS_RUN_SCOPE；没有是 None
    client_refusal: str | None = None   # 经 task-server/task submit 交给服务的那一条：lib_project_code_runs.subagent_task_server_refusal 对交它的那一条客户端命令的说明（子 agent 照它拒）
    submit: tuple | None = None   # 经 task-server/task submit 交的：交它的那一条在 `--` 之前带的放宽标识（可以是空元组）；不经 submit 起的是 None


class CrashSubject(NamedTuple):
    frames: tuple         # 读到它经过的驱动脚本，同 FoundRun.frames
    kind: str             # replay-items、cargo-targets、prove-red、mutation-items（CRASH_SUBJECT_DESCRIPTIONS 的键）
    words: tuple          # 剥掉前缀与包装之后的那一条
    directory: str | None
    environment: dict     # 这一条看得到的环境赋值（命令前的赋值、env、export），缩域变量与范围声明都从这里取
    submit: tuple | None  # 同 FoundRun.submit
    variables: dict | None = None   # 这一条之前赋过的 shell 变量（名字 → 已展开的值，None 是算不出）：命令点名的那棵树写成 `$名` 时照它展开（point_tree_words）


class ScanState:
    """一次判定里各层共用：同一条命令里前面用 heredoc 写出的文件（实际路径 → 正文），各层判不了的、喂给 shell 的输入（每处一句）。"""
    def __init__(self):
        self.written = {}
        self.unjudged = []
        self.flag_scripts_read = set()   # 判放宽标识时读过的驱动脚本（实际路径）：一次判定里每份只读一遍
        self.submit = None   # 正往里判的那一条 task-server/task submit 带的放宽标识；不在 submit 里是 None
        self.submit_frames = None   # 正往里判的那一条 submit 所在的那一层（frames）：同一层的（含剥了包装的）才是交给服务的那一条，读进去的驱动脚本里的不算
        self.crash_subjects = []   # 命令位置上每一条崩溃域闸要判的指定范围的命令（CrashSubject）
        self.submitted_runs = []   # 经 task-server/task submit 交给服务的那一条里命令位置上的每一条（SubmittedRun）：点没点名照 unnamed_submission_problem 判


class SubmittedRun(NamedTuple):
    frames: tuple         # 同 FoundRun.frames
    words: tuple          # 剥掉前缀与包装之后的那一条
    directory: str | None
    submit: tuple         # 交它的那一条 submit 带的放宽标识（同 FoundRun.submit）


def without_force(arguments):
    """各入口的 preflight 先摘掉每一个 --force，判的是摘完的参数。"""
    return [argument for argument in arguments if argument != "--force"]


def named_option_values(arguments, option):
    """`--item X` 与 `--item=X` 点名的值，去重、按出现次序。"""
    values, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == option and position + 1 < len(arguments):
            values.append(arguments[position + 1])
            position += 2
            continue
        if argument.startswith(option + "="):
            values.append(argument[len(option) + 1:])
        position += 1
    return list(dict.fromkeys(value for value in values if value))


def query_form(arguments, query_arguments):
    """只查清单、自证、帮助这类不要声明的跑法。弄坏开关 query-forms-declared 下一律当真跑。"""
    return not guard_switch_on("query-forms-declared") and any(argument in query_arguments for argument in arguments)


def counted_text(count, unit):
    return f"{count} {unit}"


def stage_run_shape(stage, arguments):
    rest = without_force(arguments)
    if query_form(rest, STAGE_QUERY_ARGUMENTS):
        return None
    key = f".claude/gate.d/{stage}"
    cells = sorted(project_code_runs.checked_cells(rest) or set())
    items = named_option_values(rest, "--item")
    if cells or items:
        parts = ([f"格 {'、'.join(cells)}"] if cells else []) + ([f"项 {'、'.join(items)}"] if items else [])
        unit = "格" if cells and not items else "项"
        count = len(cells) + len(items)
        return RunShape("items", f"门禁 {stage} 点名的{'，'.join(parts)}", count, unit, counted_text(count, unit), key, "stage")
    return RunShape("whole", f"门禁 {stage} 的全部格（不带 --check / --item）", None, "", "整轮", key, "stage")


def crash_run_shape(name, arguments):
    rest = without_force(arguments)
    key = f"scripts/crash/{name}"
    if name == "crash-point-replay.sh":
        if query_form(rest, CRASH_REPLAY_QUERY_ARGUMENTS):
            return None
        items = named_option_values(rest, "--item")
        if items:
            return RunShape("items", f"崩溃点重放入口点名的项 {'、'.join(items)}", len(items), "项", counted_text(len(items), "项"), key, "crash")
        return RunShape("whole", "崩溃点重放入口不指定范围跑全部项", None, "", "整轮", key, "crash")
    if name == "crash-amplification.sh":
        if query_form(rest, CRASH_AMPLIFICATION_QUERY_ARGUMENTS):
            return None
        items = named_option_values(rest, "--item")
        if items:
            return RunShape("items", f"崩溃放量点名的分项 {'、'.join(items)}", len(items), "项", counted_text(len(items), "项"), key, "crash")
        return RunShape("whole", "崩溃放量的全部分项（不带 --item）", None, "", "整轮", key, "crash")
    if query_form(rest, SELFTEST_ONLY_QUERY_ARGUMENTS):
        return None
    target, filtered, position = None, False, 0
    while position < len(rest):
        argument = rest[position]
        if argument in CRASH_DRIVER_OPTIONS_WITH_VALUE:
            filtered = filtered or argument == "--test-filter"
            position += 2
            continue
        if not argument.startswith("-") and target is None:
            target = argument
        position += 1
    driver = "分项驱动" if name == "crash-amplification-item.sh" else "双机驱动"
    unit = "个测试" if filtered else "个测试目标"
    return RunShape("target", f"崩溃放量{driver}起测试目标 {target or '（没给）'}{'（带 --test-filter）' if filtered else ''}", 1, unit,
                    counted_text(1, unit), key, "crash")


def replay_run_shape(arguments):
    rest = without_force(arguments)
    query_arguments = REPLAY_QUERY_ARGUMENTS - {"--list-items"} if guard_switch_on("replay-list-items-declared") else REPLAY_QUERY_ARGUMENTS
    if query_form(rest, query_arguments):
        return None
    numbers = list(dict.fromkeys(argument for argument in rest if not argument.startswith("-")))
    key = "scripts/experiment-apparatus/replay.sh"
    if numbers:
        return RunShape("items", f"实验复跑点名的实验 {'、'.join(numbers)}", len(numbers), "个实验", counted_text(len(numbers), "个实验"), key, "replay")
    return RunShape("whole", "实验复跑的全部实验（不带实验号）", None, "", "整轮", key, "replay")


MERGE_OPTIONS_WITH_VALUE = frozenset({"--item", "--crash-expand-up-to"})   # merge-and-verify 带一个值的选项（照它的用法行）：值不是补丁目录


def merge_patch_directories(arguments):
    """merge-and-verify 命令上给的补丁目录：不以 - 起头的词，减掉 --item、--crash-expand-up-to 后面那个值（--item=X 的写法不占下一个词），去重、按出现次序。"""
    found, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in MERGE_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if not argument.startswith("-"):
            found.append(argument)
        position += 1
    return list(dict.fromkeys(found))


def merge_run_shape(arguments):
    """合入后验证只跑补丁改到的测试目标，种类恒是 items（不带 --item 也不是 whole）：带 --item（给几次取并集，--item=X 也认）范围数是点名的项数，
    不带时是给的补丁目录个数（merge_patch_directories）；带 --list-items 只列这一趟可点的项，是问询、不要声明。
    弄坏开关 merge-as-whole 下照整轮（whole）判，merge-list-items-declared 下 --list-items 也要声明。"""
    rest = without_force(arguments)
    key = "scripts/gate/merge-and-verify.sh"
    query_arguments = MERGE_QUERY_ARGUMENTS - {"--list-items"} if guard_switch_on("merge-list-items-declared") else MERGE_QUERY_ARGUMENTS
    if query_form(rest, query_arguments):
        return None
    if guard_switch_on("merge-as-whole"):
        return RunShape("whole", "合入后验证（弄坏开关 merge-as-whole：照整轮判）", None, "", "整轮", key, "merge")
    items = named_option_values(rest, "--item")
    if items:
        return RunShape("items", f"合入后验证点名的项 {'、'.join(items)}（测试那一步只跑点名的测试目标；打补丁、fmt、clippy、build 照旧，验证包那一步只交点名的验证包目标里的）",
                        len(items), "项", counted_text(len(items), "项"), key, "merge")
    directories = merge_patch_directories(rest)
    return RunShape("items", f"合入后验证 {len(directories)} 份补丁（{'、'.join(directories) or '没给补丁目录'}；只跑补丁改到的与变异行点名的测试目标，判不出的那几种测试不跑、退 3）",
                    len(directories), "份补丁", counted_text(len(directories), "份补丁"), key, "merge")


def cargo_run_shape(arguments, directory):
    command = project_code_runs.cargo_subcommand(arguments, directory)
    if command is None or command.subcommand not in ("test", "t"):
        return None
    rest = command.rest
    separator = rest.index("--") if "--" in rest else len(rest)
    before, after = rest[:separator], rest[separator + 1:]
    if "--no-run" in before and not guard_switch_on("query-forms-declared"):
        return None
    if project_code_runs.libtest_lists_only(after) and not guard_switch_on("query-forms-declared"):
        return None
    packages, targets, filters, every_target, whole_workspace = [], [], [], False, False
    position = 0
    while position < len(before):
        word = before[position]
        following = before[position + 1] if position + 1 < len(before) else ""
        if word in ("-p", "--package"):
            packages.append(following)
            position += 2
            continue
        if word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
        elif word.startswith("-p") and not word.startswith("--") and len(word) > 2:
            packages.append(word[2:])
        elif word in CARGO_NAMED_TARGET_OPTIONS:
            targets.append(f"{word} {following}")
            position += 2
            continue
        elif word.startswith(tuple(option + "=" for option in CARGO_NAMED_TARGET_OPTIONS)):
            targets.append(word.replace("=", " ", 1))
        elif word in CARGO_SINGLE_TARGET_FLAGS:
            targets.append(word)
        elif word in CARGO_EVERY_TARGET_FLAGS:
            every_target = True
        elif word in CARGO_WHOLE_WORKSPACE_FLAGS:
            whole_workspace = True
        elif word in project_code_runs.CARGO_TEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        elif not word.startswith("-"):
            filters.append(word)
        position += 1
    position = 0
    while position < len(after):
        word = after[position]
        if word in project_code_runs.LIBTEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if not word.startswith("-"):
            filters.append(word)
        position += 1
    packages = list(dict.fromkeys(package for package in packages if package))
    targets = list(dict.fromkeys(targets))
    filters = list(dict.fromkeys(filters))
    package_text = f"-p {'、'.join(packages)} " if packages else ""
    if whole_workspace or every_target or not targets or guard_switch_on("cargo-targets-as-whole"):
        if whole_workspace or not packages:
            return RunShape("whole", f"cargo test {package_text}不挑目标（整工作区，或当前目录那个包的全部测试目标）", None, "", "整轮", None, "cargo")
        return RunShape("whole", f"cargo test 整包 {'、'.join(packages)}（不挑目标，全部测试二进制）", None, "", "整包", None, "cargo")
    if "--exact" in after and filters:
        return RunShape("target", f"cargo test {package_text}{'、'.join(targets)} 里按全名点名的测试 {'、'.join(filters)}", len(filters), "个测试",
                        counted_text(len(filters), "个测试"), None, "cargo")
    return RunShape("target", f"cargo test {package_text}的测试目标 {'、'.join(targets)}", len(targets), "个测试目标",
                    counted_text(len(targets), "个测试目标"), None, "cargo")


def run_shape_of(words, directory):
    """剥完包装的一条命令是不是要声明的验证脚本：是就交 RunShape，不是（或是只查清单的写法）交 None。"""
    if not words or project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    word, arguments = words[0], words[1:]
    name = os.path.basename(word)
    stage = project_code_runs.gate_stage_name(word, directory)
    if stage is not None:
        return stage_run_shape(stage, arguments)
    if name in ("crash-point-replay.sh", "crash-amplification.sh", "crash-amplification-item.sh", "crash-amplification-two-hosts.sh") \
            and project_code_runs.located_in(word, directory, "scripts/crash"):
        return crash_run_shape(name, arguments)
    if name == "replay.sh" and project_code_runs.located_in(word, directory, "scripts/experiment-apparatus"):
        return replay_run_shape(arguments)
    if name == "merge-and-verify.sh" and project_code_runs.located_in(word, directory, "scripts/gate"):
        return merge_run_shape(arguments)
    if name == "gate-staged.sh" and project_code_runs.located_in(word, directory, "scripts/gate"):
        if query_form(without_force(arguments), SELFTEST_ONLY_QUERY_ARGUMENTS):
            return None
        return RunShape("whole", "提交前的整轮门禁（gate-staged.sh 起整轮 gate.sh --staged）", None, "", "整轮", "scripts/gate/gate-staged.sh", "gate-whole")
    if name == "gate.sh" and any(project_code_runs.located_in(word, directory, location) for location in GATE_WHOLE_LOCATIONS):
        return RunShape("whole", "整轮门禁 gate.sh", None, "", "整轮", ".claude/singlefs-ai-sop/scripts/gate.sh", "gate-whole")
    if name == "cargo":
        return cargo_run_shape(arguments, directory)
    return None


class ScopeDeclaration(NamedTuple):
    segments: dict
    problems: list


def parse_scope_declaration(text):
    """SINGLEFS_RUN_SCOPE 的值按 ; 切成四段；值里自带的 ; 接到上一段后面。"""
    segments, problems, current = {}, [], None
    for raw in text.split(";"):
        part = raw.strip()
        found = SEGMENT_START.match(part)
        if found:
            key, value = found.group(1), found.group(2).strip()
            if key in segments:
                problems.append(f"「{key}」一段写了两次")
            segments[key] = value
            current = key
        elif current is not None:
            if part:
                segments[current] = f"{segments[current]}; {part}"
        elif part:
            problems.append(f"认不出「{part[:40]}」：每段写成 键=值，键只认 种类、跑什么、范围、为什么")
    for key in SEGMENT_KEYS:
        if key not in segments:
            problems.append(f"缺「{key}」一段")
        elif not segments[key]:
            problems.append(f"「{key}」一段是空的")
    kind = segments.get("种类")
    if kind and kind not in SCOPE_KINDS:
        problems.append(f"种类「{kind}」认不出，只认 whole、items、target")
    return ScopeDeclaration(segments, problems)


def reason_character_count(reason):
    """为什么一段的字数：去掉不说理由的词，再只数字母与数字（汉字算字母），空白与标点不算。"""
    for filler in REASON_FILLER_WORDS:
        reason = reason.replace(filler, "")
    return sum(1 for character in reason if unicodedata.category(character)[0] in ("L", "N"))


def commit_gate_problem(shape, submit):
    """提交时才许不指定范围地跑的那几道门禁与入口（UNSCOPED_COMMIT_ENTRIES）：经 task-server/task submit 交、`--` 之前一个放宽标识（LOOSENING_SUBMIT_FLAGS）都没带的，
    交一句说明（出路写在里面）；别的交 None。不经 submit 起的（直接跑）不在这里判，由 project-code-guard.sh 拒。弄坏开关 commit-whole-flag-unchecked 下不判。"""
    if submit is None or shape.kind != "whole" or shape.script_key not in UNSCOPED_COMMIT_ENTRIES or guard_switch_on("commit-whole-flag-unchecked"):
        return None
    if shape.script_key == CRASH_AMPLIFICATION_ENTRY:
        if "--full-gate" in submit:
            return None
        return (f"{COMMIT_PROBLEM_PREFIX}{shape.what}，这一条经 submit 交却没带 --full-gate：崩溃放量全部分项每次提交必跑、与六道同列，"
                "照 .claude/skills/crash-test/commit-stages.md 带 --full-gate（整包确认标识 --whole-package-confirmed 顶替不了它）；平时改成点名项（--item <项>）")
    if any(flag in submit for flag in LOOSENING_SUBMIT_FLAGS):
        return None
    if shape.script_key.startswith(".claude/gate.d/"):
        return (f"{COMMIT_PROBLEM_PREFIX}{shape.what}，这一条经 submit 交却没带 --full-gate："
                "提交时照 .claude/skills/crash-test/commit-stages.md 带 --full-gate（会弹窗要用户确认）；平时改成点名项或格（--item <项> / --check <格>）")
    return (f"{COMMIT_PROBLEM_PREFIX}{shape.what}，这一条经 submit 交却既没带 --full-gate、也没带 --whole-package-confirmed："
            "提交时的崩溃点重放走门禁 .claude/gate.d/crash-point-replay.sh，照 .claude/skills/crash-test/commit-stages.md 带 --full-gate（会弹窗要用户确认）；"
            "平时改成点名项（--item <项>）")


def count_basis(shape):
    """闸数范围一段那个数的办法，按命令的形状写一句（范围对不上的说明里用）。"""
    if shape.family == "stage":
        return "--check 点名的格（逗号隔的逐个算）与 --item 点名的项各自去重后相加；有 --item 单位写项，只有 --check 单位写格"
    if shape.family == "crash":
        if shape.kind == "target":
            return "起一个测试目标算 1 个（带 --test-filter 的单位写个测试）"
        return "--item 点名的项去重后数（--crash-expand-up-to 后面的深度值不算项）"
    if shape.family == "replay":
        return "命令里不以 - 起头的词当实验号、去重后数，单位是个实验（范围里别的单位，如「行」「次」，不当实验数）"
    if shape.family == "merge":
        if shape.unit == "项":
            return "--item 点名的项去重后数"
        return "不带 --item 时数补丁目录份数（不以 - 起头的词，--item、--crash-expand-up-to 后面的值不算）"
    if shape.family == "cargo":
        if shape.unit == "个测试":
            return "`--` 之后带 --exact 又给了筛选词时数筛选词个数"
        return "--test / --bin / --example / --bench 点名的目标与 --lib、--doc 各算一个"
    return ""


def variable_built_words(words):
    """命令的参数里写成变量、命令替换的词（含 `--item=$名`）：闸只数写成字面的 --item / --check，这些词里拼出来的数不到。"""
    return [word for word in words[1:] if "$" in word or "`" in word]


def variable_built_hint(words):
    """范围或种类对不上的说明末尾加的一句：命令里有变量拼的词时点名它们、出路是把 --item 写成字面。弄坏开关 variable-built-hint-dropped 下不写。"""
    built = [] if guard_switch_on("variable-built-hint-dropped") else variable_built_words(words or [])
    if not built:
        return ""
    shown = "、".join(word if len(word) <= 40 else word[:40] + "…" for word in built[:3])
    return f"；命令里的 {shown} 是变量或命令替换，闸只数写成字面的 --item / --check，数不到里面拼的：把 --item 写成字面（--check 的格名同样写成字面）"


def declaration_to_write(shape):
    """说明里写这条命令该写的整段声明（种类与范围，数与单位都是闸自己数的那一个）。弄坏开关 count-hint-unprinted 下不写。"""
    if guard_switch_on("count-hint-unprinted"):
        return ""
    return f"；这条命令该写「种类={shape.kind}」「范围={shape.range_hint}」"


def scope_problems(shape, declaration_text, agent_type, client_refusal=None, submit=None, words=None):
    """这一条要声明的命令哪里不合：交回一串说明，空的是放行。client_refusal 是交它的那一条客户端命令照 lib_project_code_runs.subagent_task_server_refusal 判出的说明
    （经 task-server/task submit 交的才有）：子 agent 交的照它拒，种类不论（子 agent 一律不往统一任务服务交任务，
    records/2026-10-29-subagent拆分提案.md 第一百七十四节；判定与 project-code-guard.sh 的 ② 是同一处，这里不另写）。子 agent 直接起的验证命令这里只判声明，
    直接跑项目代码由 project-code-guard.sh 的 ① 拒。弄坏开关 subagent-submissions-allowed 下子 agent 交的不拒（许子 agent 自己交）。
    submit 是交它的那一条 task-server/task submit 带的放宽标识（FoundRun.submit）：主 agent 交的不指定范围的提交时门禁与入口没带放宽标识的照 commit_gate_problem 拒。
    种类或范围数对不上的说明写这条命令该写的整段「种类=…」「范围=<数> <单位>」（declaration_to_write，数与单位是闸数出来的那一个）、数法（count_basis）；
    words 是剥完包装的命令词，里面有变量拼的词时另写「把 --item 写成字面」（variable_built_hint）。"""
    problems = []
    if agent_type and client_refusal and not guard_switch_on("subagent-submissions-allowed"):
        problems.append(f"子 agent 不往统一任务服务交任务：这一条经 {client_refusal} 交的是{shape.what}。{project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT}")
    commit_problem = None if agent_type else commit_gate_problem(shape, submit)
    if commit_problem:
        problems.append(commit_problem)
    if declaration_text is None:
        if not guard_switch_on("declaration-optional"):
            problems.append("没带范围声明")
        return problems
    declaration = parse_scope_declaration(declaration_text)
    if not guard_switch_on("segments-unchecked"):
        problems += declaration.problems
    segments = declaration.segments
    declared_kind = segments.get("种类")
    if declared_kind in SCOPE_KINDS and declared_kind != shape.kind:
        if not guard_switch_on("kind-unchecked"):
            problems.append(f"声明的是{KIND_DESCRIPTIONS[declared_kind]}，命令跑的是{KIND_DESCRIPTIONS[shape.kind]}：{shape.what}"
                            f"{declaration_to_write(shape)}{variable_built_hint(words)}")
    range_text = segments.get("范围", "")
    if range_text and declared_kind == shape.kind and not guard_switch_on("count-unchecked"):
        if shape.kind in ("items", "target"):
            numbers = re.findall(r"\d+", range_text)
            if not numbers or int(numbers[0]) != shape.count:
                if guard_switch_on("count-hint-unprinted"):
                    problems.append(f"范围写的是「{range_text}」，命令点名的是 {shape.count} {shape.unit}：范围一段开头写这个数（阿拉伯数字）{variable_built_hint(words)}")
                else:
                    problems.append(f"范围写的是「{range_text}」，闸数出这条命令点名的是 {shape.count} {shape.unit}（{shape.what}；数法：{count_basis(shape)}）"
                                    f"{declaration_to_write(shape)}，范围一段开头是阿拉伯数字{variable_built_hint(words)}")
        elif shape.kind == "whole" and not any(word in range_text for word in WHOLE_RANGE_WORDS):
            problems.append(f"范围写的是「{range_text}」，命令跑的是整轮：whole 的范围写「整轮」或「整包」")
    reason = segments.get("为什么", "")
    if reason and not guard_switch_on("reason-unchecked") and not (shape.family == "gate-whole" and COMMIT_WHOLE_REASON in reason):
        counted = reason_character_count(reason)
        if counted < REASON_MINIMUM_CHARACTERS:
            problems.append(f"为什么一段只有 {counted} 个字（去掉空白、标点与「要跑」「验证」这类不说理由的词之后）：至少 {REASON_MINIMUM_CHARACTERS} 个字，"
                            "写为什么这么跑、这个范围为什么合适")
    return problems


def submit_loosening_flags(words):
    """task-server/task submit 在 `--` 之前带的放宽标识（LOOSENING_SUBMIT_FLAGS 的键），去重、按出现次序。words 是 task_server_submission 认过的客户端命令：`--` 之前的选项都是写出来的。"""
    separator = words.index("--") if "--" in words else len(words)
    return tuple(dict.fromkeys(word for word in words[2:separator] if word in LOOSENING_SUBMIT_FLAGS))


def verification_test_targets(test_arguments):
    """cargo test 的参数（test 之后的）里 --test 点名的验证包测试目标，去重、按出现次序。点名的包里没有验证包、带 --no-run、只列用例的交空表。"""
    separator = test_arguments.index("--") if "--" in test_arguments else len(test_arguments)
    before, after = test_arguments[:separator], test_arguments[separator + 1:]
    if "--no-run" in before or project_code_runs.libtest_lists_only(after):
        return []
    packages, targets, position = [], [], 0
    while position < len(before):
        word = before[position]
        following = before[position + 1] if position + 1 < len(before) else ""
        if word in ("-p", "--package"):
            packages.append(following)
            position += 2
            continue
        if word == "--test":
            targets.append(following)
            position += 2
            continue
        if word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
        elif word.startswith("--test="):
            targets.append(word.split("=", 1)[1])
        elif word.startswith("-p") and not word.startswith("--") and len(word) > 2:
            packages.append(word[2:])
        elif word in project_code_runs.CARGO_TEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        position += 1
    if packages and VERIFICATION_PACKAGE not in packages:
        return []
    return list(dict.fromkeys(target for target in targets if target))


def crash_subject_kind(words, directory):
    """这一条是不是崩溃域闸要判的四种指定范围的形状：崩溃点重放入口点名项（replay-items）、cargo test 点名验证包测试目标（cargo-targets）、
    证红脚本 scripts/mutation/prove-red.sh（prove-red）、变异门禁 crates-mutation-replay.sh 点名行（mutation-items）；是交种类，不是交 None。
    只查清单、自证、帮助的写法不算。"""
    if not words or project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    word, arguments = words[0], without_force(list(words[1:]))
    name = os.path.basename(word)
    if project_code_runs.gate_stage_name(word, directory) == "crates-mutation-replay.sh":
        listing = any(argument in STAGE_QUERY_ARGUMENTS for argument in arguments)
        return "mutation-items" if named_option_values(arguments, "--item") and not listing else None
    if name == "crash-point-replay.sh" and project_code_runs.located_in(word, directory, "scripts/crash"):
        listing = any(argument in CRASH_REPLAY_QUERY_ARGUMENTS for argument in arguments)
        return "replay-items" if named_option_values(arguments, "--item") and not listing else None
    if name == "prove-red.sh" and project_code_runs.located_in(word, directory, "scripts/mutation"):
        return "prove-red" if len(arguments) >= 4 and arguments[0] == "--copy" else None
    if name == "cargo":
        command = project_code_runs.cargo_subcommand(arguments, directory)
        if command is not None and command.subcommand in ("test", "t") and verification_test_targets(list(command.rest)):
            return "cargo-targets"
    return None


def crash_shrink_option_written(subject):
    """命令里写了 --crash-expand-up-to（崩溃点重放入口认它）。"""
    return any(word == CRASH_SHRINK_OPTION or word.startswith(CRASH_SHRINK_OPTION + "=") for word in subject.words)


def crash_shrink_variables_written(subject):
    """这一条看得到的环境里设了（值非空）的缩域变量，照 CRASH_SHRINK_VARIABLES 的次序。"""
    return [name for name in CRASH_SHRINK_VARIABLES if subject.environment.get(name)]


def full_domain_reason_written(declaration_text):
    """范围声明的为什么一段里写了「全域理由：<理由>」，理由去掉不说理由的词之后至少 FULL_DOMAIN_REASON_MINIMUM_CHARACTERS 个字。弄坏开关 crash-domain-reason-length-unchecked 下写了就算。"""
    if not declaration_text:
        return False
    reason = parse_scope_declaration(declaration_text).segments.get("为什么", "")
    found = FULL_DOMAIN_REASON_PATTERN.search(reason)
    if found is None:
        return False
    return guard_switch_on("crash-domain-reason-length-unchecked") or reason_character_count(found.group(1)) >= FULL_DOMAIN_REASON_MINIMUM_CHARACTERS


def read_text_lines(path):
    """文件按换行切成的行；读不了（不在、不是 UTF-8、没权限）交 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read().split("\n")
    except (OSError, UnicodeDecodeError):
        return None


def rust_code_without_comments(text):
    """Rust 源码去掉 // 与 /* */（可嵌套）注释之后的文本，字符串、原始字符串（r"…"、r#"…"#）与字符字面量照留：
    判点到的目标读哪个缩域变量只认代码里写的变量名，注释里提到的不算。没收尾的字符串与注释照到文末算。"""
    kept, position, length = [], 0, len(text)
    while True:
        found = RUST_LEXICAL_START.search(text, position)
        if found is None:
            kept.append(text[position:])
            return "".join(kept)
        kept.append(text[position:found.start()])
        token = found.group(0)
        if token == "//":
            end = text.find("\n", found.end())
            position = length if end < 0 else end
        elif token == "/*":
            depth, position = 1, found.end()
            while depth:
                edge = RUST_BLOCK_COMMENT_EDGE.search(text, position)
                if edge is None:
                    position = length
                    break
                depth, position = depth + (1 if edge.group(0) == "/*" else -1), edge.end()
        elif token.startswith("r"):
            closing = '"' + found.group(1)
            end = text.find(closing, found.end())
            position = length if end < 0 else end + len(closing)
            kept.append(text[found.start():position])
        elif token == '"':
            string = RUST_STRING_LITERAL.match(text, found.start())
            position = string.end() if string else length
            kept.append(text[found.start():position])
        else:
            kept.append(token)
            position = found.end()


def registry_root(directory, project_root):
    """登记表所在的树的根：从命令的当前目录往上找带 STAGE_INPUTS_TABLE 的那一层，找不到取 project_root。"""
    current = os.path.abspath(directory) if directory else None
    while current:
        if os.path.isfile(os.path.join(current, STAGE_INPUTS_TABLE)):
            return current
        parent = os.path.dirname(current)
        if parent == current:
            break
        current = parent
    return project_root


def registered_crash_targets(root):
    """登记表里走崩溃枚举的验证包测试目标：stage-inputs.tsv 的 crash-case: 行点名的，加崩溃放量入口 ITEMS 里分项点名的；交 (目标集合, 读不了登记表的说明或 None)。
    弄坏开关 crash-domain-registry-unread 下交空集合。"""
    if guard_switch_on("crash-domain-registry-unread"):
        return set(), None
    lines = read_text_lines(os.path.join(root, STAGE_INPUTS_TABLE))
    if lines is None:
        return set(), f"读不了 {os.path.join(root, STAGE_INPUTS_TABLE)}"
    targets = set()
    for line in lines:
        if line.startswith("crash-case:"):
            targets.update(CRASH_CASE_TEST_WORD.findall(line))
    amplification, inside = read_text_lines(os.path.join(root, CRASH_AMPLIFICATION_ENTRY)) or [], False
    for line in amplification:
        if line.startswith("ITEMS=("):
            inside = True
        elif inside and line.startswith(")"):
            break
        elif inside:
            found = AMPLIFICATION_ITEM_LINE.match(line)
            if found:
                targets.add(found.group(1))
    return targets, None


def crash_target_enumerates(root, target, registered):
    """验证包测试目标会不会走崩溃枚举：登记过的是；没登记的看测试源码里有没有读缩域变量（读它的用例能缩域，就是会枚举全域）。
    没登记又读不到源码的交 None（判不出）。弄坏开关 crash-domain-source-scan-skipped 下没登记的一律交 False。"""
    if target in registered:
        return True
    if guard_switch_on("crash-domain-source-scan-skipped"):
        return False
    lines = read_text_lines(os.path.join(root, VERIFICATION_TESTS_DIRECTORY, target + ".rs")) if re.fullmatch(r"\w+", target) else None
    if lines is None:
        return None
    text = "\n".join(lines)
    return any(name in text for name in CRASH_SHRINK_VARIABLES)


def target_shrink_variables(root, target, registered):
    """会走崩溃枚举的验证包测试目标读哪几个缩域变量（照 CRASH_SHRINK_VARIABLES 的次序；放行照它判，拒绝说明照它点名）：
    登记过的（崩溃枚举用例与放量分项经放量引擎）读放量那个；测试源码的代码里（rust_code_without_comments：注释里提到的不算）写了哪个变量名就读哪个，两样都占的两个都交。
    判不出的交 None：读不到测试源码（目标是 tests/<目标>/main.rs 这类、名字不是一个词），或没登记而代码里两个变量名都没写。
    弄坏开关 crash-domain-target-variable-source-unread 下不读源码、一律交放量那个；crash-domain-target-variable-fixed 下读得到源码的一律交判器那个；
    crash-domain-variable-comments-counted 下注释里写的变量名也算。"""
    amplification, judge = CRASH_SHRINK_VARIABLES
    if guard_switch_on("crash-domain-target-variable-source-unread"):
        return [amplification]
    lines = read_text_lines(os.path.join(root, VERIFICATION_TESTS_DIRECTORY, target + ".rs")) if re.fullmatch(r"\w+", target) else None
    if lines is None:
        return None
    if guard_switch_on("crash-domain-target-variable-fixed"):
        return [judge]
    text = "\n".join(lines)
    code = text if guard_switch_on("crash-domain-variable-comments-counted") else rust_code_without_comments(text)
    found = [name for name in CRASH_SHRINK_VARIABLES if name in code or (name == amplification and target in registered)]
    return found or None


class CrashDomains(NamedTuple):
    """崩溃域闸对一条命令点到的目标或项判出的四样（crash_subject_domains 交）。"""
    enumerating: list    # 会走崩溃枚举的（说明）
    unknown: list        # 判不出会不会走的（说明）
    variables: list      # 会走的那几个读的缩域变量的并（照 CRASH_SHRINK_VARIABLES 的次序）
    undetermined: list   # 会走而判不出读哪一个缩域变量的（说明）


def consulted_trees(roots):
    """崩溃域闸这一趟读的树：roots 排在前面的优先（crash_domain_trees）；弄坏开关 crash-domain-tree-fallback-dropped 下只读排头那一棵，里面没有的不退回别的树。"""
    return roots[:1] if guard_switch_on("crash-domain-tree-fallback-dropped") else list(roots)


def target_domains(roots, targets, whence):
    """点名的验证包测试目标逐个判，交 CrashDomains。每个目标取 roots 里第一棵判得出来的树的结论（登记过，或测试源码读得到），这一棵里判不出的才退回下一棵；
    登记表读不了的树只信「会走」、不信「不走」（排在最后一棵的照信）。whence 写这些目标是谁点的，进判不出的说明。"""
    trees = consulted_trees(roots)
    registrations = {root: registered_crash_targets(root) for root in trees}
    enumerating, variables, undetermined, unknown, notes = [], set(), [], [], []
    for target in targets:
        decided = None
        for position, root in enumerate(trees):
            registered, problem = registrations[root]
            verdict = crash_target_enumerates(root, target, registered)
            if verdict is None or (problem and not verdict and position < len(trees) - 1):
                continue
            decided = (root, registered, verdict)
            break
        for root in ([decided[0]] if decided else trees):   # 登记表读不了的说明只记定下这个目标的那一棵（没定下的记读过的每一棵），每一棵一句
            problem = registrations[root][1]
            if problem and f"{whence}：{problem}，登记过的崩溃枚举目标没核" not in notes:
                notes.append(f"{whence}：{problem}，登记过的崩溃枚举目标没核")
        if decided is None:
            unknown.append(f"{whence}：测试目标 {target} 既没登记、也读不到 {VERIFICATION_TESTS_DIRECTORY}/{target}.rs，没判会不会走崩溃枚举"
                           f"{'（读过 ' + '、'.join(trees) + '）' if len(trees) > 1 else ''}")
            continue
        root, registered, verdict = decided
        if verdict:
            enumerating.append(f"测试目标 {target}")
            names = target_shrink_variables(root, target, registered)
            if names is None:
                undetermined.append(f"{whence}：测试目标 {target} 会走崩溃枚举，判不出它读哪一个缩域变量"
                                    f"（读不到 {VERIFICATION_TESTS_DIRECTORY}/{target}.rs，或没登记而代码里两个变量名都没写）")
            else:
                variables.update(names)
    return CrashDomains(enumerating, notes + unknown, [name for name in CRASH_SHRINK_VARIABLES if name in variables], undetermined)


def replay_item_domains(items, roots):
    """崩溃点重放入口点名的项逐个判：合表（REPLAY_ITEMS_TABLE）里这一项点了 crash-case 键或放量分项的会走崩溃枚举；几棵树里都有的取排在前面那一棵的。
    交 CrashDomains：入口照放量那个变量缩（它另认 CRASH_SHRINK_OPTION）。"""
    tables = [lines for lines in (read_text_lines(os.path.join(root, REPLAY_ITEMS_TABLE)) for root in consulted_trees(roots)) if lines is not None]
    if not tables:
        return CrashDomains([], [f"崩溃点重放入口点名的项 {'、'.join(items)}：读不了 {REPLAY_ITEMS_TABLE}，没判会不会走崩溃枚举"], [], [])
    enumerates_by_item = {}
    for lines in reversed(tables):   # 排在前面的树后写，盖住后面的
        for line in lines:
            if not line.strip() or line.startswith("#"):
                continue
            fields = line.split("\t#", 1)[0].split("\t")
            if len(fields) == 3:
                enumerates_by_item[fields[0].strip()] = any(field.strip() not in ("", "-") for field in fields[1:])
    enumerating = [f"项 {item}" for item in items if enumerates_by_item.get(item)]
    unknown = [f"崩溃点重放入口点名的项 {item} 不在 {REPLAY_ITEMS_TABLE} 里，没判会不会走崩溃枚举" for item in items if item not in enumerates_by_item]
    return CrashDomains(enumerating, unknown, [CRASH_SHRINK_VARIABLES[0]], [])


def mutation_row_targets(table_lines, item):
    """变异表里行号或变异名等于 item 的行（同名的几行都算）第五段点名的验证包测试目标；表里一行也没对上交 None。"""
    matched, targets = False, []
    for number, line in enumerate(table_lines, 1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 5 or item not in (str(number), fields[0]):
            continue
        matched = True
        try:
            targets += verification_test_targets(shlex.split(fields[4]))
        except ValueError:
            continue
    return targets if matched else None


def mutation_item_domains(items, roots, whence):
    """变异行逐个判：点名的变异名或行号在 roots 里每棵树的变异表（MUTATION_TABLE）里按次序找，取第一张对得上的表里那一行（读不了的表跳过），
    行点名的测试目标再按 target_domains 判（同一份 roots）。交 CrashDomains。"""
    trees = consulted_trees(roots)
    tables = [lines for lines in (read_text_lines(os.path.join(root, MUTATION_TABLE)) for root in trees) if lines is not None]
    if not tables:
        return CrashDomains([], [f"{whence}：读不了变异表 {MUTATION_TABLE}（读过 {'、'.join(trees)}），没判会不会走崩溃枚举"], [], [])
    targets, unknown = [], []
    for item in items:
        found = next((row_targets for row_targets in (mutation_row_targets(lines, item) for lines in tables) if row_targets is not None), None)
        if found is None:
            unknown.append(f"{whence}点名的 {item} 不在变异表 {MUTATION_TABLE} 里（读过 {'、'.join(trees)}），没判会不会走崩溃枚举")
        else:
            targets += found
    domains = target_domains(roots, list(dict.fromkeys(targets)), f"{whence}点名的行")
    return domains._replace(unknown=unknown + domains.unknown)


def stage_positional_words(arguments):
    """门禁阶段的参数里不是选项、也不是 --item / --check 的值的词（项目根）。"""
    found, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("--item", "--check"):
            position += 2
            continue
        if not argument.startswith("-"):
            found.append(argument)
        position += 1
    return found


def known_tree_word(word, variables):
    """点名一棵树的词（submit 的 --cwd 值、证红的 --copy 值、变异门禁的根参数）：写成 `$名`、`${名}` 的，用这段命令文本里前面赋过的值展开
    （lib_shell_words.py 的 expand_known_variables，$HOME 也认）；不带变量的原样交回，展开不出（没赋过的变量、命令替换、带空白）交 None。
    弄坏开关 crash-domain-tree-variables-unexpanded 下一律原样交回（`$名` 当成路径里的字面字符）。"""
    if ("$" not in word and "`" not in word) or guard_switch_on("crash-domain-tree-variables-unexpanded"):
        return word
    value = shell_words.expand_known_variables(word, variables or {})
    return value if value and not re.search(r"\s", value) else None


def submit_words_with_known_directory(words, variables):
    """task-server/task submit 在 `--` 之前的 --cwd 写成赋过字面值的变量的（`S=<副本>; task-server/task submit --cwd $S -- …`），把值换成展开之后的（known_tree_word），
    别的词不动；展开不出的照原样。"""
    separator = words.index("--") if "--" in words else len(words)
    option = project_code_runs.TASK_SERVER_SUBMIT_DIRECTORY_OPTION
    changed = list(words)
    for position in range(2, separator):
        if changed[position] == option and position + 1 < separator:
            changed[position + 1] = known_tree_word(changed[position + 1], variables) or changed[position + 1]
        elif changed[position].startswith(option + "="):
            changed[position] = option + "=" + (known_tree_word(changed[position][len(option) + 1:], variables) or changed[position][len(option) + 1:])
    return changed


def crash_domain_trees(subject, project_root):
    """崩溃域闸读的树（变异表、登记表、测试源码都按这个次序，前面的树里有的以它为准，没有的才退回后面的）：命令点明的树（证红 --copy 点名的副本、变异门禁的根参数，
    写成赋过字面值的变量的先展开）、命令当前目录那棵树（registry_root：submit 的 --cwd 指到副本时就是那个副本）、项目根（主工作区）。
    交 ([树的根…], [展开不出的说明…])：同一棵（realpath 相同）只留最前面那一份，盘上不是目录的不读。弄坏开关 crash-domain-named-tree-unread 下命令点明的树不读。"""
    arguments, base = without_force(list(subject.words[1:])), subject.directory or project_root
    named = arguments[1:2] if subject.kind == "prove-red" else list(reversed(stage_positional_words(arguments))) if subject.kind == "mutation-items" else []
    candidates, notes = [], []
    for word in ([] if guard_switch_on("crash-domain-named-tree-unread") else named):
        value = known_tree_word(word, subject.variables)
        if value is None:
            notes.append(f"{CRASH_SUBJECT_DESCRIPTIONS[subject.kind]}：命令点名的树 {word} 是展开不出值的变量，没读它指的那棵树，读的是命令当前目录那一棵与项目根")
        else:
            candidates.append(os.path.join(base, os.path.expanduser(value)))
    candidates += [registry_root(subject.directory, project_root), project_root]
    roots, seen = [], set()
    for candidate in candidates:
        real = os.path.realpath(candidate)
        if os.path.isdir(real) and real not in seen:
            seen.add(real)
            roots.append(candidate)
    return roots, notes


def crash_subject_domains(subject, roots, notes=()):
    """这一条点到的目标或项判出的 CrashDomains；roots 与 notes 是 crash_domain_trees 交的读的树与展开不出的说明（说明并进判不出的）。
    弄坏开关 CRASH_SUBJECT_BREAK_SWITCHES 里这一种的那一项下不判（四样都空）。"""
    if guard_switch_on(CRASH_SUBJECT_BREAK_SWITCHES[subject.kind]):
        return CrashDomains([], [], [], [])
    arguments = without_force(list(subject.words[1:]))
    if subject.kind == "replay-items":
        domains = replay_item_domains(named_option_values(arguments, "--item"), roots)
    elif subject.kind == "cargo-targets":
        command = project_code_runs.cargo_subcommand(arguments, subject.directory)
        domains = target_domains(roots, verification_test_targets(list(command.rest)) if command is not None else [], "cargo test")
    elif subject.kind == "prove-red":
        domains = mutation_item_domains(list(dict.fromkeys(arguments[3:])), roots, "证红")
    else:
        domains = mutation_item_domains(named_option_values(arguments, "--item"), roots, "变异门禁")
    return domains._replace(unknown=list(notes) + domains.unknown)


def crash_domain_problem(subject, project_root, state):
    """指定范围的命令点到会走崩溃枚举的目标或项而没缩对域：交一句说明（出路写在里面，缩域变量照点到的目标点名）；放行交 None。
    放行的有：交它的那一条 submit 带 --full-gate（不指定范围的提交时那一趟另判）、命令里写了 --crash-expand-up-to、范围声明的为什么一段写了全域理由、
    这一条看得到的环境里设了点到的目标读的每一个缩域变量（target_shrink_variables；多设一个不拒）、
    点到的目标里有判不出读哪一个的而环境里设了两个缩域变量之一（照有一个就放，判不出的那一处记进 state.unjudged）。
    点到的目标或项判不出会不会枚举的，放行并把说明记进 state.unjudged。
    弄坏开关：crash-domain-full-gate-judged 下带 --full-gate 的也判；crash-domain-shrink-ignored 下写了 --crash-expand-up-to、设了缩域变量的都当没缩；
    crash-domain-reason-ignored 下写了全域理由的也判；crash-domain-unknown-unrecorded 下判不出的（会不会走、读哪一个）都不记；
    crash-domain-variable-unknown-refused 下判不出读哪一个的照没缩对拒；crash-domain-shrink-variable-unchecked 下设了两个缩域变量之一就放、不核是不是读的那一个；
    crash-domain-extra-variable-refused 下多设了点到的目标不读的那一个也拒。
    变异表、登记表与测试源码读的树见 crash_domain_trees（命令点明的副本在前、没有的退回命令当前目录那棵与项目根，读不到的说明记进 state.unjudged）；
    弄坏开关 crash-domain-named-tree-unread 下命令点明的树不读，crash-domain-tree-fallback-dropped 下只读排头那一棵，crash-domain-tree-variables-unexpanded 下 `$名` 写的树不展开。"""
    if subject.submit is not None and "--full-gate" in subject.submit and not guard_switch_on("crash-domain-full-gate-judged"):
        return None
    shrink_ignored = guard_switch_on("crash-domain-shrink-ignored")
    if crash_shrink_option_written(subject) and not shrink_ignored:
        return None
    if full_domain_reason_written(subject.environment.get(DECLARATION_VARIABLE)) and not guard_switch_on("crash-domain-reason-ignored"):
        return None
    roots, tree_notes = crash_domain_trees(subject, project_root)
    domains = crash_subject_domains(subject, roots, tree_notes)
    unrecorded = guard_switch_on("crash-domain-unknown-unrecorded")
    if domains.unknown and not unrecorded:
        state.unjudged += domains.unknown
    if not domains.enumerating:
        return None
    written = [] if shrink_ignored else crash_shrink_variables_written(subject)
    needed = domains.variables
    if written and domains.undetermined and not guard_switch_on("crash-domain-variable-unknown-refused"):
        if not unrecorded:
            state.unjudged += [f"{entry}；命令里设了 {'、'.join(written)}，照有一个缩域变量就放行" for entry in domains.undetermined]
        return None
    if written and guard_switch_on("crash-domain-shrink-variable-unchecked"):
        return None
    missing = [name for name in needed if name not in written]
    extra = [name for name in written if name not in needed] if guard_switch_on("crash-domain-extra-variable-refused") else []
    if written and not missing and not extra and not domains.undetermined:
        return None
    reads = f"点到的目标读的是 {'、'.join(needed)}" if needed else "判不出点到的目标读哪一个缩域变量"
    named = needed or list(CRASH_SHRINK_VARIABLES)
    way_out = (f"出路：带点到的目标读的缩域变量（经服务交的写成 `-- env {' '.join(name + '=<n>' for name in named)} <命令…>`"
               f"{'，崩溃点重放入口也可以加 ' + CRASH_SHRINK_OPTION + ' <n>' if subject.kind == 'replay-items' else ''}），"
               f"或在 {DECLARATION_VARIABLE} 的为什么一段写「全域理由：<至少 {FULL_DOMAIN_REASON_MINIMUM_CHARACTERS} 个字，说明为什么这一趟要判全部崩溃状态>」"
               "（证红脚本不要求声明，要写理由就带一份四段声明）")
    head = f"{CRASH_SUBJECT_DESCRIPTIONS[subject.kind]}会走崩溃枚举（{'、'.join(domains.enumerating)}）"
    limit = "指定范围的那几趟崩溃状态数压在 20 万量级（.claude/rules/verification.md「一个门禁：两种跑法」）。"
    if not written:
        return (f"崩溃域没缩：{head}，命令与环境里都没带缩域（{CRASH_SHRINK_OPTION}、{'、'.join(CRASH_SHRINK_VARIABLES)}），"
                f"范围声明的为什么一段也没写全域理由；{reads}。{limit}{way_out}")
    wrong = [f"少了 {'、'.join(missing)}（它读的那一个没设，域没缩）"] if missing else []
    wrong += [f"多了 {'、'.join(extra)}（它不读）"] if extra else []
    wrong += domains.undetermined
    return f"崩溃域没缩对：{head}，{reads}，命令里设的是 {'、'.join(written)}：{'；'.join(wrong)}。{limit}{way_out}"


def unnamed_submission_problem(run, state):
    """经 task-server/task submit 交给服务的一条（SubmittedRun）点没点名：判据只调 task-server/task_server_lib.py 的 unnamed_run_problem
    （服务提交时拒 ⑦ 同一个函数；经 lib_project_code_runs.task_server_library 载入，不另写一份）。→ 那个函数交的 {problem, next_step}，点了名交 None。
    不指定范围的提交时门禁与两份崩溃入口（UNSCOPED_COMMIT_ENTRIES 的 whole 那一形）归 commit_gate_problem 判，这里不重复判。
    task_server_lib.py 载不进、判的时候抛了：放行，那一处记进 state.unjudged。
    弄坏开关 unnamed-full-gate-dropped 下不把 --full-gate 交给判据，unnamed-confirmed-dropped 下不把 --whole-package-confirmed 交给判据。"""
    shape = run_shape_of(list(run.words), run.directory)
    if shape is not None and shape.kind == "whole" and shape.script_key in UNSCOPED_COMMIT_ENTRIES:
        return None
    library, problem = project_code_runs.task_server_library()
    if library is None:
        state.unjudged.append(f"点没点名判不了（{problem}）：{shlex.join(run.words)[:200]}")
        return None
    full_gate = "--full-gate" in run.submit and not guard_switch_on("unnamed-full-gate-dropped")
    confirmed = "--whole-package-confirmed" in run.submit and not guard_switch_on("unnamed-confirmed-dropped")
    try:
        return library.unnamed_run_problem(list(run.words), run.directory, full_gate, confirmed)
    except Exception as error:   # 判据出事：放行，记一处判不了
        state.unjudged.append(f"点没点名判不了（task_server_lib.unnamed_run_problem 抛了 {error!r}）：{shlex.join(run.words)[:200]}")
        return None


def repository_tooling_script(real_path):
    """仓里的工具脚本：落在某一层带 .claude/hooks/ 的目录下的 scripts/ 或 .claude/ 里（主工作区与仓副本都算）。"""
    current = os.path.dirname(real_path)
    while True:
        parent = os.path.dirname(current)
        if parent == current:
            return False
        if os.path.basename(current) in ("scripts", ".claude") and os.path.isdir(os.path.join(parent, ".claude", "hooks")):
            return True
        current = parent


def remember_heredoc_writes(command, state):
    """`cat > 文件 <<EOF`、`cat <<EOF > 文件`、`tee [-a] 文件 <<EOF`：把 heredoc 正文记成那个文件此刻的内容（追加的接在后面）。"""
    if not command.standard_input_heredocs or not command.words:
        return
    body = command.standard_input_heredocs[-1].text
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    targets = []
    if name == "cat" and not arguments:
        targets = [(target, operator in (">>", "&>>")) for operator, target in command.output_redirections if operator in (">", ">|", ">>", "&>", "&>>")]
    elif name == "tee":
        appending = any(argument in ("-a", "--append") for argument in arguments)
        targets = [(argument, appending) for argument in arguments if not argument.startswith("-")]
    for target, appending in targets:
        if UNREADABLE_PATH_CHARACTERS & set(target):
            continue
        resolved = shell_words.resolve_path(command.directory, os.path.expanduser(target))
        if resolved is None:
            continue
        real = os.path.realpath(resolved)
        earlier = ""
        if appending:
            earlier = state.written.get(real)
            if earlier is None:
                try:
                    with open(real, encoding="utf-8") as handle:
                        earlier = handle.read()
                except (OSError, UnicodeDecodeError):
                    earlier = ""
        state.written[real] = earlier + body


def interpreter_of_head(head):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。"""
    if not head.startswith(b"#!"):
        return None
    first = head[2:].split(b"\n", 1)[0].decode("utf-8", "replace").split()
    if not first:
        return ""
    program = os.path.basename(first[0])
    if program == "env":
        rest = [word for word in first[1:] if not word.startswith("-") and "=" not in word]
        return os.path.basename(rest[0]) if rest else ""
    return program


def script_path_word(command, variables):
    """bash / sh / source 起的脚本路径写成 `$名`、`${名}` 的（`S=<目录>; bash $S/x.sh`），用这段命令文本里前面赋过的值展开（variables 是外层交进来的，这一条自己的 command.variables 盖在上面；
    lib_shell_words.py 的 expand_known_variables，$HOME 也认）：展开得出、不带空白的换上，展开不出（没赋过的变量、for 的循环变量、命令替换）与别的写法照原样交回。
    弄坏开关 driver-path-variables-unexpanded 下一律照原样。"""
    word = command.words[0]
    if "$" not in word or "`" in word or command.launcher not in ("shell", "source") or guard_switch_on("driver-path-variables-unexpanded"):
        return word
    known = dict(variables or {})
    known.update(command.variables or {})
    value = shell_words.expand_known_variables(word, known)
    return value if value and not re.search(r"\s", value) else word


def driver_script_text(command, state, variables=None):
    """命令位置上执行的是不是要读进去的驱动脚本：是就交 (实际路径, 正文)，不是或读不到交 None。脚本路径写成赋过字面值的变量的先展开（script_path_word）。"""
    word = script_path_word(command, variables)
    if command.launcher == "python" or word.startswith("-") or UNREADABLE_PATH_CHARACTERS & set(word):
        return None
    expanded = os.path.expanduser(word)
    if command.launcher is None and "/" not in expanded:
        return None
    resolved = shell_words.resolve_path(command.directory, expanded)
    if resolved is None:
        return None
    real = os.path.realpath(resolved)
    if real in state.written:
        data = state.written[real].encode("utf-8")
    else:
        if not os.path.isfile(real):
            return None
        if repository_tooling_script(real) and not guard_switch_on("tooling-scripts-read"):
            return None
        try:
            with open(real, "rb") as handle:
                data = handle.read(MAXIMUM_SCRIPT_BYTES + 1)
        except OSError as error:   # 在而读不了：里面要声明的命令判不了，记一处（弄坏开关 driver-unread-unrecorded 下静默跳过）
            if not guard_switch_on("driver-unread-unrecorded"):
                state.unjudged.append(f"命令位置上执行的脚本 {real} 读不了（{type(error).__name__}: {error.strerror or error}），里面要声明的命令没判")
            return None
    if b"\0" in data[:SCRIPT_HEAD_BYTES]:
        return None
    if len(data) > MAXIMUM_SCRIPT_BYTES:   # 太大：是 shell 脚本的照判不了记一处（同上）
        interpreter = interpreter_of_head(data[:SCRIPT_HEAD_BYTES])
        shell_script = (command.launcher is not None or interpreter in shell_words.SHELL_NAMES
                        or (interpreter is None and (real.endswith(".sh") or resolved.endswith(".sh"))))
        if shell_script and not guard_switch_on("driver-unread-unrecorded"):
            state.unjudged.append(f"命令位置上执行的脚本 {real} 比 {MAXIMUM_SCRIPT_BYTES} 字节大，没读进去，里面要声明的命令没判")
        return None
    if command.launcher is None:
        interpreter = interpreter_of_head(data[:SCRIPT_HEAD_BYTES])
        if interpreter is None and not real.endswith(".sh") and not resolved.endswith(".sh"):
            return None
        if interpreter is not None and interpreter not in shell_words.SHELL_NAMES:
            return None
    if b"\0" in data:
        return None
    try:
        return real, data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def script_command_line_numbers(script_text):
    """脚本里每条命令（剥完前缀与包装的词序列）出现在哪几行：逐个逻辑行（反斜杠续行接上）单独切，喂给非 shell 命令的 heredoc 正文换成空行、行号不变。"""
    numbered = {}
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    index = 0
    while index < len(lines):
        start_line, joined = index + 1, lines[index]
        while joined.endswith("\\") and index + 1 < len(lines):
            index += 1
            joined = joined + "\n" + lines[index]
        index += 1
        try:
            line_commands = shell_words.commands_at_command_position(joined).commands
        except Exception:
            continue
        for line_command in line_commands:
            numbered.setdefault(tuple(line_command.words), []).append(start_line)
    return numbered


def scope_runs_in_text(text, directory, environment=None, frames=(), script=None, state=None, variables=None):
    """text 里命令位置上每一条要声明的命令；包装剥掉往里判，执行的驱动脚本读进去递归判。
    variables 是外层这段命令文本里前面赋过的 shell 变量（名字 → 已展开的值）：交给服务的那一条与包装里面那一条是重新切词的另一段文本，里面看不到外层赋的值，
    由这里带进去（驱动脚本里起的不带：脚本是另一个 shell 进程，看不到外层没 export 的变量）。"""
    if state is None:
        state = ScanState()
    found, seen, numbered = [], {}, None
    scan = shell_words.commands_at_command_position(text, directory, environment)
    # 喂给从标准输入读命令的 shell、判不了的输入（lib_shell_words.py 的 shell_standard_input）：照常判字面的那一部分，这一处记进检出
    state.unjudged += [f"脚本 {script} 里{entry}" if script else entry for entry in scan.unjudged_shell_inputs]
    for command in scan.commands:
        key = tuple(command.words)
        occurrence = seen.get(key, 0)
        seen[key] = occurrence + 1
        if script is None:
            here = frames
        else:
            if numbered is None:
                numbered = script_command_line_numbers(text)
            line_numbers = numbered.get(key, [])
            here = frames + ((script, line_numbers[occurrence] if occurrence < len(line_numbers) else None),)
        remember_heredoc_writes(command, state)
        known = dict(variables or {})
        known.update(command.variables or {})
        # 经 task-server/task submit 交给服务的那一条（lib_project_code_runs.task_server_submission）：照一条命令接着判，声明认 submit 前面写的
        # （客户端把 SINGLEFS_RUN_SCOPE 当白名单变量带进任务）与 `--` 之后 env 写的（run-scope-guard.sh 文件头「经 task-server 提交」）；--cwd 写成赋过字面值的变量的先展开
        submitted = (None if guard_switch_on("task-server-submissions-unread")
                     else project_code_runs.task_server_submission(submit_words_with_known_directory(command.words, known), command.directory, command.launcher))
        if submitted is not None:
            submitted_words, submitted_directory = submitted
            client_refusal = project_code_runs.subagent_task_server_refusal(command.words, command.directory, command.launcher)
            outer_submit, state.submit = state.submit, submit_loosening_flags(list(command.words))
            outer_frames, state.submit_frames = state.submit_frames, here
            try:
                found += [run._replace(client_refusal=client_refusal)
                          for run in scope_runs_in_text(shlex.join(submitted_words), submitted_directory or command.directory, command.environment, here, None, state, known)]
            finally:
                state.submit, state.submit_frames = outer_submit, outer_frames
            continue
        launched = None
        if not guard_switch_on("wrappers-opaque"):
            launched = (project_code_runs.command_under_launcher(command.words, command.directory)
                        or project_code_runs.command_under_gpu_quiesce(command.words, command.directory))
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += scope_runs_in_text(shlex.join(inner_words), inner_start, command.environment, here, None, state, known)
            continue
        # 交给服务的那一条（与它剥了包装的那一层）里命令位置上的每一条：点没点名事后照 unnamed_submission_problem 判（服务提交时拒 ⑦ 同一份判据）；
        #   读进去的驱动脚本里的不算（服务只判提交的这一条）。弄坏开关 unnamed-submissions-unjudged 下不收
        if state.submit is not None and here == state.submit_frames and not guard_switch_on("unnamed-submissions-unjudged"):
            state.submitted_runs.append(SubmittedRun(here, tuple(command.words), command.directory, state.submit))
        crash_kind = crash_subject_kind(command.words, command.directory)
        if crash_kind is not None:
            state.crash_subjects.append(CrashSubject(here, crash_kind, tuple(command.words), command.directory, dict(command.environment), state.submit, known))
        shape = run_shape_of(command.words, command.directory)
        if shape is not None:
            found.append(FoundRun(here, list(command.words), shape, command.environment.get(DECLARATION_VARIABLE), submit=state.submit))
            continue
        if guard_switch_on("scripts-unread") or len(here) >= MAXIMUM_SCRIPT_DEPTH:
            continue
        driver = driver_script_text(command, state, known)
        if driver is None:
            continue
        real, body = driver
        if real in {os.path.realpath(path) for path, _line in here if path}:
            continue
        inner_environment = {name: value for name, value in command.environment.items() if name != DECLARATION_VARIABLE}
        found += scope_runs_in_text(body, command.directory, inner_environment, here, real, state)
    return found


class SubmitFlagAsk(NamedTuple):
    flags: tuple          # 这一条 submit 带的放宽标识
    unread_words: tuple   # `--` 之前算不出的词（变量、命令替换、xargs 这类读进来的）：认不出带没带标识
    problem: str = ""     # 判法没判成时的一句：flags 是从命令文字里按提到算的


def submit_client_command(command):
    """这一条的命令词是不是提交客户端：路径尾部是 task-server/task 的（当前目录算不出、前面是变量的同样算）、当前目录算不出而写成 ./task 这样带路径的，
    或解得开路径、对得上本仓那一份的（lib_project_code_runs.task_server_client_subcommand：cd 进 task-server 再起的、链过去的）。
    认错了只多弹一次窗，认漏了等于放行，所以路径尾部相同就算。弄坏开关 submit-flag-client-by-path-only 下只认解得开路径的。"""
    word, client = command.words[0], project_code_runs.TASK_SERVER_CLIENT
    if command.launcher not in ("startup", "program") and not guard_switch_on("submit-flag-client-by-path-only"):
        normalized = os.path.normpath(word)
        if normalized == client or normalized.endswith("/" + client):
            return True
        if command.directory is None and "/" in word and os.path.basename(normalized) == os.path.basename(client):
            return True
    return project_code_runs.task_server_client_subcommand(command.words, command.directory, command.launcher) is not None


def submit_option_pieces(word, variables):
    """submit 选项位上的一个词经外层 shell 展开之后的几个词；算不出的（没赋过的变量、命令替换、xargs 这类读进来的占位词）交 None。记成 `(元素 …)` 的数组去掉括号。"""
    if not shell_words.has_expansion([word]):
        return [word]
    value = shell_words.expand_known_variables(word, variables)
    return None if value is None else value.strip("()").split()


def submit_flag_ask(command):
    """这一条是 task-server/task submit、`--` 之前带放宽标识或有算不出的词：交 SubmitFlagAsk；别的交 None。`--` 之后是交给服务的那一条的参数，客户端不当选项，这里也不判。
    弄坏开关 submit-flag-after-separator 下 `--` 之后的也判，submit-flag-first-option-only 下只判紧跟 submit 的一个词，submit-flag-variables-unread 下算不出的词不算。"""
    variables = command.variables or {}
    if len(command.words) < 2 or not submit_client_command(command):
        return None
    head = submit_option_pieces(command.words[1], variables)
    if head is None:   # 子命令位算不出：可能就是 submit
        return None if guard_switch_on("submit-flag-variables-unread") else SubmitFlagAsk((), (command.words[1],))
    arguments = head + list(command.words[2:])
    if arguments[:1] == ["promote"]:   # 任务提前到队头：用户定「这个功能必须要弹窗让我确认才能用」
        return SubmitFlagAsk(("promote",), ())
    if arguments[:1] != ["submit"]:
        return None
    flags, unread = [], []
    for word in arguments[1:]:
        if word == "--" and not guard_switch_on("submit-flag-after-separator"):
            break
        pieces = submit_option_pieces(word, variables)
        if pieces is None:
            unread.append(word)
        else:
            flags += [flag for piece in pieces for flag in LOOSENING_SUBMIT_FLAGS if piece == flag or piece.startswith(flag + "=")]
        if guard_switch_on("submit-flag-first-option-only"):
            break
    if guard_switch_on("submit-flag-variables-unread"):
        unread = []
    return SubmitFlagAsk(tuple(flags), tuple(unread)) if flags or unread else None


def texts_launched_by(command, function_launchers, state):
    """这一条里面还会起的几段命令文本：[(文本, 当前目录)]。交给服务的那一条、flock 这类包装里的那一条、脚本里转发函数的参数、程序里字面写着的、命令词上赋过字面值的变量展开之后的、
    命令位置上执行的驱动脚本的正文（一次判定里每份只读一遍）。"""
    words, directory, nested = command.words, command.directory, []
    if submit_client_command(command) and "--" in words:
        nested.append((shlex.join(words[words.index("--") + 1:]), directory))
    launched = (project_code_runs.command_under_launcher(words, directory) or project_code_runs.command_under_gpu_quiesce(words, directory)
                or project_code_runs.command_under_dispatch_launcher(words, directory))
    if launched is not None:
        inner_words, inner_directory = launched
        nested.append((shlex.join(inner_words), shell_words.resolve_path(directory, inner_directory) if inner_directory else directory))
    if command.launcher is None and os.path.basename(words[0]) in function_launchers:
        nested += [(shlex.join(words[start:]), directory) for start in range(1, len(words))]
    nested += [(text, directory) for text in shell_words.embedded_launches(command).texts]
    if shell_words.has_expansion(words[:1]):
        value = shell_words.expand_known_variables(words[0], command.variables or {})
        if value is not None:
            nested.append((" ".join([value] + [shlex.quote(word) for word in words[1:]]), directory))
    driver = driver_script_text(command, state)
    if driver is not None and driver[0] not in state.flag_scripts_read:
        state.flag_scripts_read.add(driver[0])
        nested.append((driver[1], directory))
    return nested


def submit_flag_asks_in_text(text, directory, environment, state, depth=0):
    """text 里命令位置上每一条带放宽标识的提交（SubmitFlagAsk）；里面还会起的接着判（texts_launched_by，弄坏开关 submit-flag-nesting-unread 下不往里判）。"""
    asks = []
    function_launchers = shell_words.launcher_functions(text)
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        remember_heredoc_writes(command, state)
        ask = submit_flag_ask(command)
        if ask is not None:
            asks.append(ask)
        if depth >= MAXIMUM_SCRIPT_DEPTH or guard_switch_on("submit-flag-nesting-unread"):
            continue
        for nested_text, nested_directory in texts_launched_by(command, function_launchers, state):
            asks += submit_flag_asks_in_text(nested_text, nested_directory, command.environment, state, depth + 1)
    return asks


def flag_asks_by_mention(command_text, problem):
    """按命令文字里提到标识算（判法没判成时才这样算：宁可多弹一次窗）：提到了交一条 SubmitFlagAsk，没提到交空表。"""
    flags = tuple(flag for flag in LOOSENING_SUBMIT_FLAGS if flag in command_text)
    return [SubmitFlagAsk(flags, (), problem)] if flags else []


def submit_flag_asks(command_text, directory, agent_type, state):
    """这条命令里要用户确认的提交。子 agent 不判（它们起 submit 由 project-code-guard.sh 拒）。这一段抛了不牵连范围声明：记一处判不了，命令文字里提到标识的照弹窗。
    弄坏开关 submit-flag-subagents-asked 下子 agent 也判，submit-flags-unjudged 下一律不判，submit-flags-by-substring 下命令文字里提到就算，submit-flag-raise-propagated 下抛了不接住。"""
    try:
        if (agent_type and not guard_switch_on("submit-flag-subagents-asked")) or guard_switch_on("submit-flags-unjudged"):
            return []
        asks = submit_flag_asks_in_text(command_text, directory, None, ScanState())
        return asks + flag_asks_by_mention(command_text, "") if guard_switch_on("submit-flags-by-substring") else asks
    except Exception as error:
        if guard_switch_on("submit-flag-raise-propagated"):
            raise
        problem = f"放宽标识没判成（{error!r}），按命令文字里提到的算"
        state.unjudged.append(problem)
        return flag_asks_by_mention(command_text, problem)


def submit_flag_reason(asks, command_text):
    """给用户看的理由：带哪个标识、标识的作用、算不出的词、命令全文、对话不许自己用。"""
    flags = [flag for flag in LOOSENING_SUBMIT_FLAGS if any(flag in ask.flags for ask in asks)]
    unread = [word for ask in asks for word in ask.unread_words]
    lines = []
    if flags:
        lines.append("这条命令带放宽检查的标识 " + "、".join(f"{flag}（{LOOSENING_SUBMIT_FLAGS[flag]}）" for flag in flags) + "。")
    if unread:
        lines.append(f"参数里有变量，认不出带没带标识（{'、'.join(unread)}）。")
    lines += [ask.problem for ask in asks if ask.problem]
    return "\n".join(lines + [SUBMIT_FLAG_USER_RULE, f"命令全文：{command_text}"])


def ask_user_about_submit_flags(asks, command_text):
    """有要确认的就往标准输出打 permissionDecision 为 ask 的 JSON（退出码照 0）。弄坏开关 submit-flag-ask-silent 下不打；判的时候抛异常那一支也走这里，
    而认不出的项正是让 guard_switch_on 抛异常的那一种，所以直接读变量。"""
    if not asks or "submit-flag-ask-silent" in os.environ.get(BREAK_VARIABLE_NAME, "").split(","):
        return
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "ask",
                                             "permissionDecisionReason": submit_flag_reason(asks, command_text)}}, ensure_ascii=False))


def last_runs_path(script_key):
    """上次红的清单：SINGLEFS_LAST_RUNS_DIRECTORY 设了放它底下，没设是数据根下的 last-runs；同目录 lib_work_root.py 读不到、取不到时抛它的异常。"""
    directory = os.environ.get(LAST_RUNS_VARIABLE)
    if not directory:
        if HOOK_DIRECTORY not in sys.path:
            sys.path.insert(0, HOOK_DIRECTORY)
        import lib_work_root
        directory = lib_work_root.shared_data_path(LAST_RUNS_ENTRY)
    return os.path.join(directory, script_key.replace("/", "__") + ".tsv")


def last_red_lines(shape):
    """上次这一道红的清单（给拒绝说明用的几行）。"""
    if shape.script_key is None:
        return ["上次红的：cargo test 没有清单文件，从上次的日志取：python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包>"]
    try:
        path = last_runs_path(shape.script_key)
    except (ImportError, OSError, ValueError) as error:
        return [f"上次红的：取不到上次红的清单在哪（同目录 lib_work_root.py 读不到或取不到数据根：{error!r}）"]
    if guard_switch_on("last-runs-unread"):
        return [f"上次红的：没有上次红的记录（{path}）"]
    try:
        with open(path, encoding="utf-8") as handle:
            names = [line.strip() for line in handle if line.strip() and not line.strip().startswith("#")]
    except (OSError, UnicodeDecodeError):
        return [f"上次红的：没有上次红的记录（{path} 不在或读不了；各道跑完写这份文件归下一批）"]
    if not names:
        return [f"上次红的：没有上次红的记录（{path} 是空的）"]
    return [f"上次红的（{path}，{len(names)} 项）：{'、'.join(names)}"]


def template_line(found, command_text):
    shape = found.shape
    declaration = f"{DECLARATION_VARIABLE}='种类={shape.kind}; 跑什么={shape.what}; 范围={shape.range_hint}; 为什么=<为什么这么跑、这个范围为什么合适>'"
    if found.frames:
        return f"  在 {frames_description(found.frames[-1:])} 那一行的命令前面加（已有的那份换掉）：{declaration}"
    bare = command_text if guard_switch_on("template-keeps-declaration") else LEADING_DECLARATION.sub("", command_text, count=1)
    if DECLARATION_VARIABLE in bare:
        return f"  把命令里已有的那份 {DECLARATION_VARIABLE} 换成：{declaration}"
    if len(bare) > TEMPLATE_COMMAND_LIMIT:
        return f"  这条命令 {len(bare)} 字，不整条贴；在起它的那一条前面加：{declaration}"
    return f"  {declaration} {bare}"


NARROWING_LINES = (
    "→ 先想范围合不合适：红了只跑红的，改了只点名改到的——",
    "  门禁：bash .claude/gate.d/<阶段>.sh --check <格>[,<格>…]（格名用 --list 列）；有逐项的门禁写 --item <项>（--list-items 列）",
    "  崩溃用例：bash .claude/gate.d/crash-point-replay.sh --check crash-case-replay --crash-cases-only --item crash-case:<用例名> <根>（--list-items 列）；"
    "崩溃放量 bash scripts/crash/crash-amplification.sh --item <分项>（--list-items 列）",
    "  cargo test：cargo test -p <包> --test <目标> -- <测试名> --exact；上次红的用 python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包> 打出只重跑它们的命令",
    "  merge-and-verify：挑测试目标在脚本里做（只跑补丁改到的与变异行点名的测试目标），种类写 items，不带 --item 范围写补丁目录的份数，为什么写明这一批补丁改到哪些目标；"
    "红了只重跑红的写 --skip-apply --item <包>:<目标根文件> <同一批补丁目录>（--list-items 列），范围写项数",
)
DECLARATION_RULE_LINES = (
    f"→ 声明写在命令开头：{DECLARATION_VARIABLE}='种类=…; 跑什么=…; 范围=…; 为什么=…'，四段用 ; 隔、各段 键=值；"
    "种类 whole（不带 --item / --check 的整轮、cargo test 整包或整工作区、整轮门禁）、items（带 --item / --check、merge-and-verify（带 --item 范围写项数，不带写补丁目录的份数）、replay.sh 带实验号）、"
    "target（cargo test 带 --test / --bin / --lib 目标、崩溃放量的分项与双机驱动）；"
    "范围：items 写「N 格 / N 项」、target 写「N 个测试目标」（带 --exact 筛选词的写「N 个测试」），N 是阿拉伯数字、等于点名的个数，whole 写「整轮」或「整包」；"
    f"为什么至少 {REASON_MINIMUM_CHARACTERS} 个字，只写「要跑」「验证」不算；整轮门禁的为什么写「提交前整轮」就放。驱动脚本里每一条验证命令那一行各自带声明。",
)


def frames_description(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)


def refusal_text(refused, command_text, crash_refused=(), unnamed_refused=()):
    """被拒的每一条逐条写：命令、跑的是什么、哪里不合；声明写得不对的（说明不以 COMMIT_PROBLEM_PREFIX 起头的）另附上次红的清单、声明模板与只跑红的写法，
    只是提交时才许跑的、崩溃域没缩的、没点名的不附（说明里已写出路）。crash_refused 是崩溃域闸拒的：[(CrashSubject, 说明)]；
    unnamed_refused 是经 submit 交而没点名的：[(SubmittedRun, {problem, next_step})]，逐条写那一句与改成点名的样子。"""
    lines = [f"✗ 范围声明不合，这条命令没跑（{GUARD_SCRIPT_NAME}）："]
    declaration_refused = []
    for number, (found, problems) in enumerate(refused, start=1):
        where = f"脚本 {frames_description(found.frames)} 里的 " if found.frames else ""
        words = shlex.join(found.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     跑的是：{found.shape.what}；范围：{found.shape.range_hint}")
        lines += [f"     ✗ {problem}" for problem in problems]
        if any(not problem.startswith(COMMIT_PROBLEM_PREFIX) for problem in problems):
            declaration_refused.append(found)
            lines += [f"     {line}" for line in last_red_lines(found.shape)]
    for number, (subject, problem) in enumerate(crash_refused, start=len(refused) + 1):
        where = f"脚本 {frames_description(subject.frames)} 里的 " if subject.frames else ""
        words = shlex.join(subject.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     ✗ {problem}")
    for number, (run, found) in enumerate(unnamed_refused, start=len(refused) + len(crash_refused) + 1):
        where = f"脚本 {frames_description(run.frames)} 里的 " if run.frames else ""
        words = shlex.join(run.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     ✗ {found['problem']}")
        lines.append(f"     → 怎么办：{found['next_step']}")
    if declaration_refused:
        lines.append("→ 照抄声明模板，把为什么填上（写为什么这么跑、这个范围为什么合适）：")
        lines += [template_line(found, command_text) for found in declaration_refused]
        lines += list(NARROWING_LINES) + list(DECLARATION_RULE_LINES)
    return "\n".join(lines)


class ScopeDecision(NamedTuple):
    code: int            # 0 放行，2 拒绝
    message: str | None  # 拒绝时写给 stderr 的说明
    allowed: list        # 放行的每一条（FoundRun）
    refused: list        # 被拒的每一条：(FoundRun, [说明])
    unjudged: list = []  # 判不了的、喂给 shell 的输入（每处一句）：照常判别的，记进检出
    flag_asks: list = []  # 要用户确认的提交（SubmitFlagAsk）；拒绝时是空的，拒了就不弹窗
    crash_refused: list = []  # 崩溃域闸拒的每一条：(CrashSubject, 说明)
    unnamed_refused: list = []  # 经 submit 交而没点名的每一条：(SubmittedRun, {problem, next_step})


def decide_scope(hook_input, project_root):
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = ScanState()
    runs = scope_runs_in_text(command_text, directory, state=state)
    refused, allowed = [], []
    for found in runs:
        problems = scope_problems(found.shape, found.declaration, agent_type, found.client_refusal, found.submit, found.words)
        (refused if problems else allowed).append((found, problems) if problems else found)
    crash_refused = [(subject, problem) for subject, problem in ((subject, crash_domain_problem(subject, project_root, state)) for subject in state.crash_subjects) if problem]
    # 经 submit 交的那一条点没点名（子 agent 交的另被拒，不判）
    unnamed_refused = [] if agent_type else [(run, found) for run, found in ((run, unnamed_submission_problem(run, state)) for run in state.submitted_runs) if found]
    flag_asks = submit_flag_asks(command_text, directory, agent_type, state)
    if refused or crash_refused or unnamed_refused:
        return ScopeDecision(2, refusal_text(refused, command_text, crash_refused, unnamed_refused), allowed, refused, state.unjudged, [], crash_refused, unnamed_refused)
    return ScopeDecision(0, None, allowed, [], state.unjudged, flag_asks)


def detections_path_now():
    configured = os.environ.get(DETECTIONS_VARIABLE)
    if configured:
        return configured
    try:
        sys.path.insert(0, HOOK_DIRECTORY)
        import lib_work_root
        return lib_work_root.default_detections_path()
    except Exception:
        return None


def append_detection(path, entry):
    if not path:
        return
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass


def record_scope_decision(hook_input, decision, path):
    """放行的每一条记一行（会话号写在 scope_session_id，看门狗不当检出叫醒）；被拒的整条命令记一行（session_id、outcome 被拒，看门狗只记不叫醒）。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = (tool_input.get("command") or "")[:2000]
    common = {"time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME, "agent_id": hook_input.get("agent_id"),
              "agent_type": hook_input.get("agent_type") or "主 agent", "transcript_path": hook_input.get("transcript_path"), "command": command_text}
    if not guard_switch_on("allowed-unrecorded"):
        for found in decision.allowed:
            declaration = parse_scope_declaration(found.declaration or "")
            entry = dict(common, scope_session_id=hook_input.get("session_id"), outcome="放行", run_scope={
                "where": frames_description(found.frames) if found.frames else "命令文本", "words": shlex.join(found.words)[:500],
                "kind_of_command": found.shape.kind, "what": found.shape.what, "declaration": found.declaration, "segments": declaration.segments})
            append_detection(path, entry)
    findings = [f"范围声明被拒：{found.shape.what}：{'；'.join(problems)}" for found, problems in decision.refused]
    findings += [f"范围声明被拒：{CRASH_SUBJECT_DESCRIPTIONS[subject.kind]}：{problem}" for subject, problem in decision.crash_refused]
    findings += [f"范围声明被拒：{found['problem']}" for _run, found in decision.unnamed_refused]
    if decision.unjudged and not failure_recording_dropped():   # 判不了的、喂给 shell 的输入与崩溃域闸判不出的目标或项：跟着这一条命令记（放行的写「只记」）
        findings += [f"范围声明闸判不了：{entry}" for entry in decision.unjudged]
    if findings:
        append_detection(path, dict(common, session_id=hook_input.get("session_id"), findings=findings,
                                    outcome="被拒" if decision.refused or decision.crash_refused or decision.unnamed_refused else "只记"))


HOOK_INPUT_HEAD_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节


def failure_recording_dropped():
    """弄坏开关 failure-unrecorded（判不了的只打 stderr、不记检出）设没设。不核别的项认不认得：判的时候抛异常那一支也读它，
    而认不出的项正是让 guard_switch_on 抛异常的那一种（自证拿它当注入）。"""
    return "failure-unrecorded" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")


def note_unjudged(warning, hook_input):
    """判不了、照常执行的一条：stderr 打一行，检出记录记一行（结局「只记」；hook 输入读不出来时会话号与 agent 写「读不到」）。"""
    print(f"  ! {warning}", file=sys.stderr)
    if failure_recording_dropped():
        return
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) and isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    known = hook_input if isinstance(hook_input, dict) else {}
    append_detection(detections_path_now(), {
        "time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME,
        "session_id": known.get("session_id", "读不到"), "agent_id": known.get("agent_id", "读不到"),
        "agent_type": (known.get("agent_type") or "主 agent") if known else "读不到", "transcript_path": known.get("transcript_path", "读不到"),
        "command": (command if isinstance(command, str) else repr(command))[:2000], "findings": [warning], "outcome": "只记"})


def hook_main(hook_directory):
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw)
        unreadable = None if isinstance(hook_input, dict) else f"解出来是 {type(hook_input).__name__}，不是对象"
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        hook_input, unreadable = None, f"解析出错：{error!r}"
    if unreadable:   # 照常执行，打一行可定位的诊断（钩子名、开头的字节）、记一条检出
        note_unjudged(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象（{unreadable}），这条命令没判、照常执行；"
                      f"开头 {len(raw[:HOOK_INPUT_HEAD_BYTES])} 字节（共 {len(raw)} 字节）：{raw[:HOOK_INPUT_HEAD_BYTES]!r}", None)
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    # 粗筛：这几样一个都没有的命令起不了验证脚本（cargo、shell 与 .sh、路径、source / .），也带不了放宽标识
    if not any(marker in command_text for marker in ("cargo", "sh", "/", "source", ". ", *LOOSENING_SUBMIT_FLAGS)):
        return 0
    try:
        load_judging_libraries()
        project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_directory))
        decision = decide_scope(hook_input, project_root)
        record_scope_decision(hook_input, decision, detections_path_now())
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记一条检出
        note_unjudged(f"{GUARD_SCRIPT_NAME} 没判成（{error!r}），这条命令照常执行", hook_input)
        if not hook_input.get("agent_type"):   # 放宽标识同样没判：命令文字里提到的照弹窗
            ask_user_about_submit_flags(flag_asks_by_mention(command_text, f"{GUARD_SCRIPT_NAME} 没判成（{error!r}），按命令文字里提到的算。"), command_text)
        return 0
    if decision.code:
        print(decision.message, file=sys.stderr)
    ask_user_about_submit_flags(decision.flag_asks, command_text)
    return decision.code


# ── 自证 ──────────────────────────────────────────────

def scope_text(kind, what, range_text, reason="改了这一道的判法，只跑改到的这几格看它转绿"):
    return f"{DECLARATION_VARIABLE}='种类={kind}; 跑什么={what}; 范围={range_text}; 为什么={reason}'"


def build_selftest_tree(work):
    """临时目录里的驱动脚本与一份仓的样子（带 .claude/hooks/ 的目录下的 scripts/ 是仓里的工具脚本）。"""
    def write(relative, content, executable=False):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(content)
        if executable:
            os.chmod(path, 0o755)
        return path
    declared_stage = scope_text("whole", "门禁 doc-text 全部格", "整轮", "提交之前补跑这一道，看全仓文字的格是不是都绿")
    write("drivers/missing.sh", "#!/usr/bin/env bash\necho 开跑\nbash .claude/gate.d/doc-text.sh .\n"
                                f"{scope_text('target', 'cargo test 一个目标', '1 个测试目标')} cargo test -p singlefs-verification --test model_contract\n")
    write("drivers/declared.sh", f"#!/usr/bin/env bash\n{declared_stage} bash .claude/gate.d/doc-text.sh .\n"
                                 f"{scope_text('target', 'cargo test 一个目标', '1 个测试目标')} cargo test -p singlefs-verification --test model_contract\n")
    write("drivers/direct.sh", "#!/usr/bin/env bash\ncargo test --workspace\n", executable=True)
    write("drivers/outer.sh", "#!/usr/bin/env bash\nbash drivers/missing.sh\n")
    write("drivers/flagged.sh", "#!/usr/bin/env bash\ntask-server/task submit --full-gate -- ls\n")
    write("repo/.claude/hooks/placeholder", "仓的样子：这一层带 .claude/hooks/\n")
    write("repo/scripts/tool.sh", "#!/usr/bin/env bash\ncargo test --workspace\n")
    # 崩溃域闸读的登记表与测试源码：登记过的崩溃枚举目标、登记过而源码在 tests/<目标>/main.rs（读不到 tests/<目标>.rs）的崩溃枚举目标、登记过的放量分项目标、
    #   代码里读判器缩域变量而注释里提到放量那个的目标、不走崩溃枚举的目标
    write(STAGE_INPUTS_TABLE, "# 样本登记表\n"
          f"crash-case:sample-stream\t-\ttest={VERIFICATION_PACKAGE}:crash_enumeration_sample_stream:sample_stream_enumerates count-line=SAMPLE\n"
          f"crash-case:sample-directory\t-\ttest={VERIFICATION_PACKAGE}:crash_enumeration_sample_directory:sample_directory_enumerates count-line=SAMPLE\n")
    write(CRASH_AMPLIFICATION_ENTRY, "#!/usr/bin/env bash\nITEMS=(\n  $'sample-amplification\\tcrash_amplification_sample\\tsample_amplification_runs'\n)\n")
    write(REPLAY_ITEMS_TABLE, "# 样本合表：项名、crash-case 键、放量分项\n"
          "sample-stream-item\tcrash-case:sample-stream\t-\t# 走崩溃点重放那一套\n"
          "sample-amplified-item\t-\tsample-amplification\t# 走放量那一套\n"
          "sample-plain-item\t-\t-\t# 两套都不走\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_enumeration_sample_stream.rs", "// 样本：登记过的崩溃枚举目标\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_amplification_sample.rs", "// 样本：登记过的放量分项目标\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_enumeration_sample_directory/main.rs", "// 样本：登记过、源码不在 tests/<目标>.rs\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_judge_sample_every_state.rs",
          f"// 样本：没登记、代码里读判器缩域变量；与 {CRASH_SHRINK_VARIABLES[0]} 同一判法（注释里提到的不算读它）\n"
          f"const VARIABLE: &str = \"{CRASH_SHRINK_VARIABLES[1]}\";\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/plain_sample.rs", "// 样本：不走崩溃枚举\n")
    write(MUTATION_TABLE, "# 样本变异表\n"
          f"sample-red-crash\tcrates/sample.rs\told\tnew\t-p {VERIFICATION_PACKAGE} --test crash_enumeration_sample_stream -- sample_stream_enumerates --exact\tsample_stream_enumerates\n"
          f"sample-red-plain\tcrates/sample.rs\told-plain\tnew-plain\t-p {VERIFICATION_PACKAGE} --test plain_sample -- plain_check --exact\tplain_check\n")
    # 命令点明的副本 copy-tree/（证红 --copy、变异门禁的根参数、submit 的 --cwd 指到它）：副本里新登记了崩溃枚举目标 crash_enumeration_copy_only 与点名它的变异行，
    #   这个目标的登记、源码与变异行主工作区这一棵（work 本身）里都没有；副本里没有主工作区的变异行 sample-red-crash
    write(f"copy-tree/{STAGE_INPUTS_TABLE}", "# 样本副本登记表\n"
          f"crash-case:copy-only\t-\ttest={VERIFICATION_PACKAGE}:crash_enumeration_copy_only:copy_only_enumerates count-line=SAMPLE\n")
    write(f"copy-tree/{VERIFICATION_TESTS_DIRECTORY}/crash_enumeration_copy_only.rs", "// 样本：副本里新登记的崩溃枚举目标\n")
    write(f"copy-tree/{MUTATION_TABLE}", "# 样本副本变异表\n"
          f"copy-only-crash\tcrates/sample.rs\told\tnew\t-p {VERIFICATION_PACKAGE} --test crash_enumeration_copy_only -- copy_only_enumerates --exact\tcopy_only_enumerates\n")
    # 提交客户端：与本仓那一份逐字节相同（lib_project_code_runs.task_server_submission 只认这样的）
    client = os.path.join(os.path.dirname(os.path.dirname(HOOK_DIRECTORY)), "task-server", "task")
    if os.path.isfile(client):
        os.makedirs(os.path.join(work, "task-server"), exist_ok=True)
        shutil.copyfile(client, os.path.join(work, "task-server", "task"))


def selftest_cells(work):
    """(说明, agent_type（None 是主 agent）, 命令, 该退 0 / 2, 当前目录)。"""
    sub = "implementation-writer"
    whole_stage = scope_text("whole", "门禁 doc-text 全部格", "整轮")
    one_cell = scope_text("items", "门禁 doc-text 一格", "1 格")
    two_patches = scope_text("items", "合入后验证两份补丁", "2 份补丁", "这一批两份补丁只改到 core 的库与验证包的一个目标，脚本只挑这几个目标")
    two_cells = "bash .claude/gate.d/code-tooling.sh --check agent-write-scope,research-script-selftests ."
    replay_declaration = scope_text("items", "崩溃点重放点名一项", "1 项")
    target_declaration = scope_text("target", "验证包一个测试目标", "1 个测试目标")
    mutation_declaration = scope_text("items", "变异门禁点名一行", "1 项")
    amplification_variable, judge_variable = CRASH_SHRINK_VARIABLES
    replay_stream = "bash scripts/crash/crash-point-replay.sh --item sample-stream-item"
    judge_target = f"cargo test -p {VERIFICATION_PACKAGE} --test crash_judge_sample_every_state"
    cells = [
        # 门禁阶段
        ("门禁整道：没带声明拒", None, "bash .claude/gate.d/doc-text.sh .", 2),
        ("门禁整道：带对的声明放", None, f"{whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("门禁 --check 两格写成 2 格：放", None, f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} {two_cells}", 0),
        ("门禁 --check 两格写成 1 格：范围数对不上拒", None, f"{scope_text('items', '门禁 code-tooling 两格', '1 格')} {two_cells}", 2),
        ("门禁 --item 声明成 items：放", None, f"{scope_text('items', '形式化点名 proof', '1 项')} bash .claude/gate.d/formal-verification.sh --item proof .", 0),
        ("门禁 --item 声明成 whole：种类对不上拒", None, f"{scope_text('whole', '形式化', '整轮')} bash .claude/gate.d/formal-verification.sh --item proof .", 2),
        ("门禁 --item 一项声明成 items：放", sub,
         f"{scope_text('items', 'crates 变异点名一项', '1 项')} bash .claude/gate.d/crates-mutation-replay.sh --item row-a .", 0),
        ("门禁 --list 不要声明", None, "bash .claude/gate.d/code-tooling.sh --list", 0),
        ("门禁 --selftest 不要声明", sub, "bash .claude/gate.d/code-tooling.sh --selftest", 0),
        ("whole 范围没写整轮或整包拒", None, f"{scope_text('whole', '门禁 doc-text', '全部')} bash .claude/gate.d/doc-text.sh .", 2),
        # 崩溃入口
        ("崩溃点重放不指定范围跑：没带声明拒", None, "bash scripts/crash/crash-point-replay.sh", 2),
        ("崩溃点重放两项：带对的声明放", None, f"{scope_text('items', '崩溃点重放两项', '2 项')} bash scripts/crash/crash-point-replay.sh --item a --item b", 0),
        ("崩溃点重放 --list-items 不要声明", sub, "bash scripts/crash/crash-point-replay.sh --list-items", 0),
        ("子 agent 起崩溃点重放点名一项、带对的声明：放", sub,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item fixed-script", 0),
        ("崩溃放量一个分项：带对的声明放", None, f"{scope_text('items', '崩溃放量一个分项', '1 项')} bash scripts/crash/crash-amplification.sh --item tree-split-streams", 0),
        # 指定范围的参数也能缩崩溃域：带 --item 时另加展开深度参数（--crash-expand-up-to <N>）照 items 声明放，深度的值不当项名
        ("崩溃点重放点名一项另加展开深度：带对的声明放", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item fixed-script --crash-expand-up-to 3", 0),
        ("崩溃放量点名一个分项另加展开深度：带对的声明放", None,
         f"{scope_text('items', '崩溃放量一个分项', '1 项')} bash scripts/crash/crash-amplification.sh --crash-expand-up-to 3 --item tree-split-streams", 0),
        ("崩溃点重放点名一项另加展开深度：写成两项拒（深度的值不是项）", None,
         f"{scope_text('items', '崩溃点重放点名两项', '2 项')} bash scripts/crash/crash-point-replay.sh --item fixed-script --crash-expand-up-to 3", 2),
        ("崩溃放量全部分项：没带声明拒", None, "bash scripts/crash/crash-amplification.sh", 2),
        ("崩溃放量分项驱动：带对的声明放", None,
         f"{scope_text('target', '分项驱动一个目标', '1 个测试目标')} bash scripts/crash/crash-amplification-item.sh crash_amplification_fixed_script --threads 8", 0),
        ("崩溃放量双机驱动：没带声明拒", None, "bash scripts/crash/crash-amplification-two-hosts.sh crash_amplification_fixed_script", 2),
        # 实验复跑、合入后验证、整轮门禁
        ("replay.sh 两个实验：带对的声明放", None, f"{scope_text('items', '复跑 E142 与 E165', '2 个实验')} bash scripts/experiment-apparatus/replay.sh E142 E165", 0),
        ("replay.sh 全部实验：没带声明拒", None, "bash scripts/experiment-apparatus/replay.sh", 2),
        ("replay.sh --admission-only 不要声明", None, "bash scripts/experiment-apparatus/replay.sh --admission-only E142", 0),
        ("replay.sh --list-items 不要声明", None, "bash scripts/experiment-apparatus/replay.sh --list-items", 0),
        # 合入后验证恒是 items：不带 --item 范围数是补丁目录的份数（--crash-expand-up-to 的值不算），声明成 whole 拒（弄坏开关 kind-unchecked、merge-as-whole 下那一格红）
        ("merge-and-verify 声明成 whole：拒", None,
         f"{scope_text('whole', '合入后验证', '整轮', '这一批三份补丁碰了 harness 与 core 两个包，照整轮声明')} bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("merge-and-verify 两份补丁：声明成 items 放", None,
         f"{scope_text('items', '合入后验证两份补丁', '2 份补丁', '这一批两份补丁只改到 core 的库与验证包的一个目标，脚本只挑这几个目标')} bash scripts/gate/merge-and-verify.sh p1 p2", 0),
        ("merge-and-verify 两份补丁写成 1 份：范围数对不上拒", None,
         f"{scope_text('items', '合入后验证两份补丁', '1 份补丁', '这一批两份补丁只改到 core 的库与验证包的一个目标，脚本只挑这几个目标')} bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("merge-and-verify 两份补丁另加展开深度：写成 2 份补丁放（深度的值不是补丁）", None,
         f"{scope_text('items', '合入后验证两份补丁', '2 份补丁', '这一批两份补丁只改到 core 的库与验证包的一个目标，缩崩溃域看一眼')} "
         "bash scripts/gate/merge-and-verify.sh --skip-apply --crash-expand-up-to 6 p1 p2", 0),
        # 带 --item（也认 --item=X）是 items，范围数是点名的项数；带 --list-items 是问询，不要声明（弄坏开关 merge-as-whole、merge-list-items-declared 下这两格红）
        ("merge-and-verify 点名两项：声明成 items 放", None,
         f"{scope_text('items', '合入后验证点名两项', '2 项', '上次红了这两个测试目标，只重跑它们看改后是不是转绿')} "
         "bash scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs --item=alpha:tests/beta.rs", 0),
        ("merge-and-verify 点名两项写成 1 项：范围数对不上拒", None,
         f"{scope_text('items', '合入后验证点名两项', '1 项', '上次红了这两个测试目标，只重跑它们看改后是不是转绿')} "
         "bash scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs --item=alpha:tests/beta.rs", 2),
        ("merge-and-verify --list-items 不要声明", sub, "bash scripts/gate/merge-and-verify.sh --list-items --with-keys p1", 0),
        ("gate-staged.sh 为什么写提交前整轮：放", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=提交前门禁; 范围=整轮; 为什么=提交前整轮' bash scripts/gate/gate-staged.sh", 0),
        ("gate-staged.sh 没带声明拒", None, "bash scripts/gate/gate-staged.sh", 2),
        ("gate.sh 为什么写提交前整轮：放", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=整轮门禁; 范围=整轮; 为什么=提交前整轮' bash .claude/singlefs-ai-sop/scripts/gate.sh --staged", 0),
        ("gate.sh 为什么只写验证拒", None, f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=整轮门禁; 范围=整轮; 为什么=验证' bash .claude/singlefs-ai-sop/scripts/gate.sh --staged", 2),
        # cargo test
        ("cargo test 整工作区：没带声明拒", None, "cargo test --workspace", 2),
        ("cargo test 整工作区：带对的声明放", None, f"{scope_text('whole', 'cargo test 整工作区', '整轮', '提交之前整工作区跑一遍，确认全部测试二进制都绿')} cargo test --workspace", 0),
        ("cargo test 两个目标、带对的声明：放", None,
         f"{scope_text('target', 'harness 两个目标', '2 个测试目标')} cargo test -p singlefs-verification --test model_contract --test scenarios", 0),
        ("cargo test 两个目标写成 3 个：范围数对不上拒", None,
         f"{scope_text('target', 'harness 两个目标', '3 个测试目标')} cargo test -p singlefs-verification --test model_contract --test scenarios", 2),
        ("cargo test --lib --exact 两个测试、带对的声明：放", None,
         f"{scope_text('target', 'unit-graph 库单测两条', '2 个测试')} cargo test -p singlefs-unit-graph --lib -- keys::first keys::second --exact", 0),
        ("cargo test --exact 两个测试写成 1 个：范围数对不上拒", None,
         f"{scope_text('target', 'unit-graph 库单测两条', '1 个测试')} cargo test -p singlefs-unit-graph --lib -- keys::first keys::second --exact", 2),
        ("cargo test 整包声明成 target：种类对不上拒", None, f"{scope_text('target', 'harness', '1 个测试目标')} cargo test -p singlefs-verification", 2),
        ("子 agent cargo test 点名目标、带对的声明：放", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --test model_contract", 0),
        ("cargo test --no-run 不要声明", sub, "cargo test -p singlefs-verification --no-run", 0),
        ("cargo test -- --list 不要声明", sub, "cargo test -p singlefs-verification --test model_contract -- --list", 0),
        ("cargo build 不要声明", sub, "cargo build --offline --all-targets", 0),
        # 包装
        ("bash -c 里的门禁没带声明拒", None, "bash -c 'bash .claude/gate.d/doc-text.sh .'", 2),
        ("bash -c 外面带声明放", None, f"{whole_stage} bash -c 'bash .claude/gate.d/doc-text.sh .'", 0),
        # here-string 与管道喂给从标准输入读命令的 shell 的字面文本当命令判（lib_shell_words.py 的 shell_standard_input；LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下这三格红）
        ("here-string 喂给 bash 的 cargo test 整工作区：没带声明拒", None, "bash <<< 'cargo test --workspace'", 2),
        ("printf 经管道喂给 bash 的门禁：没带声明拒", None, "printf 'bash .claude/gate.d/doc-text.sh .\\n' | bash", 2),
        ("echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒", None, "echo 'cargo test --workspace' | sh", 2),
        ("here-string 外面带声明放", None, f"{whole_stage} bash <<< 'bash .claude/gate.d/doc-text.sh .'", 0),
        ("capped.sh 包着的 cargo test 没带声明拒", sub, "bash scripts/hosts/capped.sh 31 cargo test -p singlefs-verification --test model_contract", 2),
        ("capped.sh 外面带声明放", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} bash scripts/hosts/capped.sh 31 cargo test -p singlefs-verification --test model_contract", 0),
        ("env 带别的变量、没带声明拒", None, "env FOO=1 bash .claude/gate.d/doc-text.sh .", 2),
        ("env 的参数里带声明放", None, f"env {whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("nice 与 timeout 包着、没带声明拒", None, "nice -n 19 timeout 600 bash .claude/gate.d/doc-text.sh .", 2),
        ("nice 与 timeout 外面带声明放", None, f"{whole_stage} nice -n 19 timeout 600 bash .claude/gate.d/doc-text.sh .", 0),
        ("同一段命令前面 export 声明放", None, f"export {whole_stage}; bash .claude/gate.d/doc-text.sh .", 0),
        ("flock 包着的门禁没带声明拒", None, "flock /tmp/run-scope-sample.lock bash .claude/gate.d/doc-text.sh .", 2),
        ("flock 外面带声明放", None, f"{whole_stage} flock /tmp/run-scope-sample.lock bash .claude/gate.d/doc-text.sh .", 0),
        ("gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", None, "bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/crash/crash-point-replay.sh --item fixed-script", 2),
        ("gpu-quiesce-run.sh 外面带声明放", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/crash/crash-point-replay.sh --item fixed-script", 0),
        ("/usr/bin/time 包着的 cargo test 没带声明拒", None, "/usr/bin/time -v cargo test --workspace", 2),
        # 驱动脚本
        ("驱动脚本里一行缺声明拒", None, "bash drivers/missing.sh", 2),
        ("驱动脚本里每一行都带声明：放", None, "bash drivers/declared.sh", 0),
        ("外面带声明、驱动脚本里一行缺声明拒", None, f"{whole_stage} bash drivers/missing.sh", 2),
        ("驱动脚本再起驱动脚本、里面缺声明拒", None, "bash drivers/outer.sh", 2),
        ("直接执行的驱动脚本缺声明拒", None, "./drivers/direct.sh", 2),
        ("同一条命令里 heredoc 写出的驱动脚本缺声明拒", None, "cat > drivers/written.sh <<'EOF'\ncargo test --workspace\nEOF\nbash drivers/written.sh", 2),
        ("同一条命令里 heredoc 写出的驱动脚本带声明：放", None,
         f"cat > drivers/written-ok.sh <<'EOF'\n{whole_stage} bash .claude/gate.d/doc-text.sh .\nEOF\nbash drivers/written-ok.sh", 0),
        ("仓里的工具脚本不读：放行", None, "bash repo/scripts/tool.sh", 0),
        # 声明的写法
        ("声明缺段：没写为什么拒", None, f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text; 范围=整轮' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明值空：范围一段是空的拒", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text; 范围=; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明写两次：种类写两次拒", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 种类=whole; 跑什么=门禁 doc-text; 范围=整轮; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明种类认不出拒", None,
         f"{DECLARATION_VARIABLE}='种类=all; 跑什么=门禁 doc-text; 范围=整轮; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么太短拒", None, f"{scope_text('whole', '门禁 doc-text', '整轮', '改了文档')} bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么只写要跑验证拒", None, f"{scope_text('whole', '门禁 doc-text', '整轮', '要跑验证要跑验证要跑验证要跑验证')} bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么里自带分号接到上一段：放", None,
         f"{scope_text('whole', '门禁 doc-text', '整轮', '提交之前补跑; 看全仓文字的格是不是都绿')} bash .claude/gate.d/doc-text.sh .", 0),
        # 子 agent 与别的命令
        # 子 agent 直接起的验证命令这里只判声明（直接跑项目代码由 project-code-guard.sh 的 ① 拒）；经 submit 交的照 lib_project_code_runs.subagent_task_server_refusal 拒
        ("子 agent 直接起整道门禁、带对的声明：放（直接跑由 ① 拒）", sub, f"{whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("子 agent 起门禁两格、带对的声明：放", sub, f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} {two_cells}", 0),
        ("不是验证脚本的命令：放", sub, "ls -la && bash .claude/hooks/run-scope-guard.sh --selftest", 0),
        # 经 task-server/task submit 交给服务的那一条照判，声明写在 submit 前面或 -- 之后 env 里（样本树里放一份与本仓逐字节相同的客户端；开关 task-server-submissions-unread 下「拒」的三格红）
        ("task-server 提交的门禁整道：没带声明拒", None, "task-server/task submit -- bash .claude/gate.d/doc-text.sh .", 2),
        ("task-server 提交的门禁点名一格、submit 前面带对的声明：放", None, f"{one_cell} task-server/task submit -- bash .claude/gate.d/doc-text.sh --check prime-marks .", 0),
        ("task-server 提交的门禁点名一格、-- 之后 env 里带对的声明：放", None, f"task-server/task submit -- env {one_cell} bash .claude/gate.d/doc-text.sh --check prime-marks .", 0),
        ("task-server 提交的 cargo test 整工作区：没带声明拒", None, "task-server/task submit -- cargo test --workspace", 2),
        ("task-server 提交带 --full-gate 的门禁整道：没带声明拒", None, "task-server/task submit --full-gate -- bash .claude/gate.d/doc-text.sh .", 2),
        ("task-server 提交带 --manual 的门禁整道：没带声明拒", None, "task-server/task submit --manual -- bash .claude/gate.d/doc-text.sh .", 2),
        ("task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒", None,
         "task-server/task submit --whole-package-confirmed /tmp/run-scope-sample-confirmed -- cargo test --workspace", 2),
        ("task-server 提交带 --full-gate 的门禁整道、submit 前面带对的声明：放", None, f"{whole_stage} task-server/task submit --full-gate -- bash .claude/gate.d/doc-text.sh .", 0),
        ("子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）", sub,
         f"{scope_text('whole', '崩溃点重放', '整轮')} task-server/task submit -- bash scripts/crash/crash-point-replay.sh", 2),
        ("子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} task-server/task submit -- cargo test -p singlefs-verification --test model_contract", 2),
        ("task-server/task wait 不要声明", sub, "task-server/task wait 7", 0),
        # 提交时才许不指定范围地跑的六道门禁与两份入口：经 submit 交而 `--` 之前一个放宽标识都没带的拒，带 --full-gate 或 --whole-package-confirmed 的放（弹窗另判）；
        # 点名格的、直接起的不判这一条（六道逐道各一格见 selftest_judge_cells）
        ("task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）", None,
         f"{whole_stage} task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh", 2),
        ("task-server 提交不指定范围的崩溃点重放门禁、带 --full-gate：放", None,
         f"{whole_stage} task-server/task submit --full-gate -- bash .claude/gate.d/crash-point-replay.sh", 0),
        ("task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒", None, f"{whole_stage} task-server/task submit -- bash scripts/crash/crash-point-replay.sh", 2),
        ("task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）", None,
         f"{whole_stage} task-server/task submit -- bash scripts/crash/crash-amplification.sh", 2),
        ("task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）", None,
         f"{whole_stage} task-server/task submit --whole-package-confirmed /tmp/run-scope-sample-confirmed -- bash scripts/crash/crash-amplification.sh", 2),
        ("task-server 提交不指定范围的崩溃放量入口、带 --full-gate：放", None,
         f"{whole_stage} task-server/task submit --full-gate -- bash scripts/crash/crash-amplification.sh", 0),
        ("task-server 提交点名格的提交时门禁：放（不是不指定范围）", None,
         f"{scope_text('items', '门禁 lkmm 一格', '1 格')} task-server/task submit -- bash .claude/gate.d/lkmm.sh --check sample-cell", 0),
        ("直接起的不指定范围的提交时门禁：这一条不判（直接跑由 project-code-guard.sh 拒）", None, f"{whole_stage} bash .claude/gate.d/lkmm.sh", 0),
        # 指定范围的崩溃域：按登记表与测试源码判点到的目标或项会不会走崩溃枚举、读哪一个缩域变量；设了读的那一个（多设一个也行）的放，
        #   只设了它不读的那一个的拒（样本树里的登记见 build_selftest_tree）
        ("崩溃点重放点名会走崩溃枚举的项、没缩域：拒", None, f"{replay_declaration} {replay_stream}", 2),
        ("崩溃点重放点名放量分项的项、没缩域：拒", None, f"{replay_declaration} bash scripts/crash/crash-point-replay.sh --item sample-amplified-item", 2),
        ("崩溃点重放点名不走崩溃枚举的项：放", None, f"{replay_declaration} bash scripts/crash/crash-point-replay.sh --item sample-plain-item", 0),
        ("崩溃点重放点名会走崩溃枚举的项、带 --crash-expand-up-to：放", None, f"{replay_declaration} {replay_stream} {CRASH_SHRINK_OPTION} 3", 0),
        ("崩溃点重放点名会走崩溃枚举的项、命令前带放量缩域变量：放", None, f"{replay_declaration} {amplification_variable}=4 {replay_stream}", 0),
        ("崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）", None, f"{replay_declaration} {judge_variable}=4 {replay_stream}", 2),
        ("task-server 提交崩溃点重放点名项、-- 之后 env 带缩域变量：放", None,
         f"{replay_declaration} task-server/task submit -- env {CRASH_SHRINK_VARIABLES[0]}=4 {replay_stream}", 0),
        ("崩溃点重放点名会走崩溃枚举的项、为什么里写了全域理由：放", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项', '全域理由：这一项刚登记，库里还没有判定，要全域判一遍才有基线')} {replay_stream}", 0),
        ("崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项', '这一项刚登记看一遍是不是转绿，全域理由：略')} {replay_stream}", 2),
        ("task-server 提交带 --full-gate、崩溃点重放点名项没缩域：这一条不判，放", None,
         f"{replay_declaration} task-server/task submit --full-gate -- {replay_stream}", 0),
        ("cargo test 点名登记过的崩溃枚举目标、没缩域：拒", None,
         f"{target_declaration} cargo test -p {VERIFICATION_PACKAGE} --test crash_enumeration_sample_stream", 2),
        ("cargo test 点名源码里读缩域变量的目标、没缩域：拒", None, f"{target_declaration} {judge_target}", 2),
        ("cargo test 点名不走崩溃枚举的目标：放", None, f"{target_declaration} cargo test -p {VERIFICATION_PACKAGE} --test plain_sample", 0),
        ("task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", None,
         f"{target_declaration} task-server/task submit -- env {judge_variable}=4 {judge_target} -- sample_judge_case --exact", 0),
        ("cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", None,
         f"{target_declaration} {amplification_variable}=4 {judge_target}", 2),
        ("cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放", None,
         f"{target_declaration} {amplification_variable}=4 {judge_variable}=4 {judge_target}", 0),
        ("cargo test 点名登记过的放量分项目标、没缩域：拒", None,
         f"{target_declaration} cargo test -p {VERIFICATION_PACKAGE} --test crash_amplification_sample", 2),
        ("cargo test 点名别的包的目标：这一条不判，放", None,
         f"{target_declaration} cargo test -p singlefs-core --test crash_enumeration_sample_stream", 0),
        ("证红点名会走崩溃枚举的变异行、没缩域：拒", None, f"bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-crash", 2),
        ("证红点名不走崩溃枚举的变异行：放", None, f"bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-plain", 0),
        ("证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放", None,
         f"{amplification_variable}=4 bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-crash", 0),
        ("变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒", None,
         f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .", 2),
        ("变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒", None, f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item 2 .", 2),
        ("变异门禁点名不走崩溃枚举的行：放", None, f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-plain .", 0),
        ("变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放", None,
         f"{mutation_declaration} {amplification_variable}=4 bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .", 0),
        # 命令点明的副本（样本树里的 copy-tree/，见 build_selftest_tree）：变异表、登记表与测试源码先读那棵树的，那棵树里没有的才退回当前目录那棵与项目根；
        #   `$名` 写的树（--copy、submit 的 --cwd）用赋过的字面值展开，包在 submit 里的也认
        ("证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", None,
         f"bash scripts/mutation/prove-red.sh --copy copy-tree {VERIFICATION_PACKAGE} copy-only-crash", 2),
        ("task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", None,
         f"C=copy-tree; task-server/task submit -- bash scripts/mutation/prove-red.sh --copy $C {VERIFICATION_PACKAGE} copy-only-crash", 2),
        ("task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", None,
         f"S=copy-tree; {target_declaration} task-server/task submit --cwd $S -- cargo test -p {VERIFICATION_PACKAGE} --test crash_enumeration_copy_only -- copy_only_enumerates --exact", 2),
        ("变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", None,
         f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item copy-only-crash copy-tree", 2),
        ("变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒", None,
         f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash copy-tree", 2),
        # 驱动脚本的路径写成赋过字面值的变量（`D=<目录>; bash $D/x.sh`）：展开之后读进去判（script_path_word），包在 submit 里的同样
        ("驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", None, f"D={work}/drivers; bash $D/missing.sh", 2),
        ("驱动脚本路径写成赋过字面值的变量、里面每一行都带声明：放", None, f"D={work}/drivers; bash $D/declared.sh", 0),
        ("task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", None, f"D={work}/drivers; task-server/task submit -- bash $D/missing.sh", 2),
        # 经 submit 交的启动测试与验证要点名（判据是 task_server_lib.py 的 unnamed_run_problem，服务提交时拒 ⑦ 同一个函数）：声明都写对，拒的只因为没点名
        ("task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）", None, f"{two_patches} task-server/task submit -- bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("task-server 提交合入后验证带 --item：放", None,
         f"{scope_text('items', '合入后验证点名一项', '1 项', '上次红了这一个测试目标，只重跑它看改后是不是转绿')} "
         "task-server/task submit -- bash scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs p1", 0),
        ("task-server 提交 cargo test 点名目标、不点名测试函数：拒", None, f"{target_declaration} task-server/task submit -- cargo test -p {VERIFICATION_PACKAGE} --test plain_sample", 2),
        ("task-server 提交 cargo test 点名 --exact 的测试函数：放", None,
         f"{scope_text('target', '验证包一条测试函数', '1 个测试')} task-server/task submit -- cargo test -p {VERIFICATION_PACKAGE} --test plain_sample -- plain_check --exact", 0),
        ("task-server 提交证红不点名变异：拒", None, f"task-server/task submit -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE}", 2),
        ("task-server 提交证红点名变异：放", None, f"task-server/task submit -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-plain", 0),
        ("task-server 提交门禁点起测试的格、不带 --item：拒", None,
         f"{scope_text('items', '门禁 crash-point-replay 一格', '1 格')} task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check feature-gated-tests .", 2),
        ("task-server 提交门禁只点不起测试的格：放", None,
         f"{scope_text('items', '门禁 crash-point-replay 一格', '1 格')} task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check crash-knobs .", 0),
        ("task-server 提交带 --full-gate、证红不点名变异：放（提交时那一趟不判点名）", None,
         f"task-server/task submit --full-gate -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE}", 0),
        ("task-server 提交带 --whole-package-confirmed、证红不点名变异：放（用户确认过的整包不判点名）", None,
         f"task-server/task submit --whole-package-confirmed /tmp/run-scope-sample-confirmed -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE}", 0),
    ]
    return cells


def submit_flag_cells():
    """放宽检查的提交标识：(说明, agent_type（None 是主 agent）, 命令, 该认出的)。该认出的是 submit_flag_summary 的写法：点名的标识，认不出的写「变量」，不该弹窗的是空串。"""
    full, whole = "--full-gate", "--whole-package-confirmed"
    submit = "task-server/task submit"
    return [
        ("提交带 --full-gate：弹窗", None, f"{submit} {full} -- ls", full),
        ("提交带 --whole-package-confirmed：弹窗", None, f"{scope_text('whole', 'cargo test 整包', '整包')} {submit} {whole} /tmp/run-scope-sample-confirmed -- cargo test -p singlefs-verification", whole),
        ("范围声明写在前面：弹窗", None, f"{scope_text('whole', '提交前的整轮门禁', '整轮')} {submit} {full} -- bash scripts/gate/gate-staged.sh", full),
        ("bash -c 里面：弹窗", None, f"bash -c '{submit} {full} -- ls'", full),
        ("&& 连着的后一段：弹窗", None, f"ls && {submit} {full} -- ls", full),
        ("--cwd 在标识前面：弹窗", None, f"{submit} --cwd /tmp/run-scope-sample {full} -- ls", full),
        ("两个标识都带：都点名", None, f"{submit} {full} {whole} /tmp/run-scope-sample-confirmed -- ls", f"{full} {whole}"),
        ("python3 起客户端：弹窗", None, f"python3 {submit} {full} -- ls", full),
        ("cd 进 task-server 再起：弹窗", None, f"cd task-server && ./task submit {full} -- ls", full),
        ("cd 到算不出的目录再起：弹窗", None, f'cd "$SAMPLE_UNKNOWN" && {submit} {full} -- ls', full),
        ("cd 到算不出的目录再起 ./task：弹窗", None, f'cd "$SAMPLE_UNKNOWN" && ./task submit {full} -- ls', full),
        ("eval 字面代码：弹窗", None, f"eval '{submit} {full} -- ls'", full),
        ("赋过字面值的变量写在参数上：认出标识", None, f"sample_flag={full}; {submit} $sample_flag -- ls", full),
        ("数组写在参数上：认出标识", None, f"sample_options=({full}); {submit} $sample_options -- ls", full),
        ("标识写成等号带值：弹窗", None, f"{submit} {whole}=/tmp/run-scope-sample-confirmed -- ls", whole),
        ("参数是变量：弹窗认不出", None, f"{submit} $sample_options -- ls", "变量"),
        ("数组全部元素写在参数上：弹窗认不出", None, f'sample_options=(--manual); {submit} "${{sample_options[@]}}" -- ls', "变量"),
        ("eval 给出的参数是变量：弹窗认不出", None, f'eval "{submit} $sample_options -- ls"', "变量"),
        ("子命令位是变量：弹窗认不出", None, "task-server/task $sample_subcommand --cwd /tmp/run-scope-sample -- ls", "变量"),
        ("参数是命令替换：弹窗认不出", None, f"{submit} $(cat sample-options) -- ls", "变量"),
        ("xargs 读进来的参数：弹窗认不出", None, f"printf x | xargs {submit}", "变量"),
        ("flock 包着：弹窗", None, f"flock /tmp/run-scope-sample.lock {submit} {full} -- ls", full),
        ("python3 -c 里字面写着：弹窗", None, f"""python3 -c "import subprocess; subprocess.run(['task-server/task', 'submit', '{full}', '--', 'ls'])\"""", full),
        ("驱动脚本里：弹窗", None, "bash drivers/flagged.sh", full),
        ("交给服务的那一条里又提交：弹窗", None, f"{submit} --manual -- bash -c '{submit} {full} -- ls'", full),
        ("脚本里的转发函数：弹窗", None, f'sample_forward() {{ "$@"; }}; sample_forward {submit} {full} -- ls', full),
        ("命令词是赋过字面值的变量：弹窗", None, f"sample_command='{submit} {full} -- ls'; $sample_command", full),
        ("eval 赋过字面值的变量：弹窗", None, f"""sample_command='{submit} {full} -- ls'; eval "$sample_command\"""", full),
        ("不带标识的提交：不弹窗", None, f"{submit} -- ls", ""),
        ("--manual 提交：不弹窗", None, f"{submit} --manual -- ls", ""),
        ("grep 这个词：不弹窗", None, f"grep -- {full} sample-file", ""),
        ("commit message 带这个词：不弹窗", None, f"git commit -m '提到 {full} 这个词'", ""),
        ("--why 里提到这个词：不弹窗", None, f"task-server/task confirm 7 --why '为什么提到 {full} 与 {whole}'", ""),
        ("交给服务的那一条里提到这个词：不弹窗", None, f"{submit} -- grep -- {full} sample-file", ""),
        ("交给服务的那一条参数是变量：不弹窗", None, f'{submit} -- ls "$sample_path"', ""),
        ("xargs 读进来的落在交给服务的那一条里：不弹窗", None, f"printf x | xargs {submit} -- ls", ""),
        ("echo 这条命令的文字：不弹窗", None, f"echo '{submit} {full} -- ls'", ""),
        ("范围声明的值里提到这个词：不弹窗", None, f"SINGLEFS_RUN_SCOPE='提到 {full}' {submit} -- ls", ""),
        ("子 agent 带标识：不弹窗（由项目代码闸拒）", "implementation-writer", f"{submit} {full} -- ls", ""),
    ]


def submit_flag_summary(asks):
    """要确认的提交写成一个串给自证格比：点名的标识（照 LOOSENING_SUBMIT_FLAGS 的次序）接认不出的「变量」，空格连，没有要确认的是空串。"""
    named = [flag for flag in LOOSENING_SUBMIT_FLAGS if any(flag in ask.flags for ask in asks)]
    return " ".join(named + (["变量"] if any(ask.unread_words for ask in asks) else []))


def selftest_judge_cells(work):
    """在内存里逐格判；交回 [(说明, 期望, 实际)]。"""
    global submit_flag_asks_in_text   # 判放宽标识抛异常那一格换掉它、判完放回（照名字写，不按名字现取 globals()）
    results = []
    for label, agent_type, command, want in selftest_cells(work):
        hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        try:
            got = decide_scope(hook_input, work).code
        except Exception as error:
            got = f"抛了 {error!r}"
        results.append((label, want, got))
    for label, agent_type, command, want in submit_flag_cells():
        hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        try:
            got = submit_flag_summary(decide_scope(hook_input, work).flag_asks)
        except Exception as error:
            got = f"抛了 {error!r}"
        results.append((label, want, got))
    # 判放宽标识这一段抛了不牵连范围声明：把判法换成一抛就停的，这条命令的 decide_scope 照常交回
    def raising_walker(*_arguments, **_options):
        raise RuntimeError("自证注入的异常")
    real_walker = submit_flag_asks_in_text
    submit_flag_asks_in_text = raising_walker
    try:
        kept = decide_scope({"tool_name": "Bash", "cwd": work, "tool_input": {"command": "task-server/task submit --full-gate -- ls"}}, work)
        got = (kept.code, submit_flag_summary(kept.flag_asks), bool(kept.unjudged))
    except Exception as error:
        got = f"抛了 {error!r}"
    finally:
        submit_flag_asks_in_text = real_walker
    results.append(("判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了", (0, "--full-gate", True), got))
    # 带一个值的选项照两份驱动的用法行（CRASH_DRIVER_OPTIONS_WITH_VALUE）：--skip 的值不当测试目标
    skipped = run_shape_of(["scripts/crash/crash-amplification-two-hosts.sh", "--skip", "crash_amplification_slow_case", "crash_amplification_fixed_script"], work)
    results.append(("崩溃放量双机驱动：--skip 的值不当测试目标", "崩溃放量双机驱动起测试目标 crash_amplification_fixed_script", skipped.what if skipped else None))

    def decided(command):
        try:
            return decide_scope({"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}, work)
        except Exception as error:
            return f"抛了 {error!r}"
    # 提交时才许不指定范围地跑的六道门禁逐道各判一遍（登记在 COMMIT_GATE_STAGE_NAMES）
    whole_declaration = scope_text("whole", "提交时的某一道门禁全部格", "整轮")
    six_codes = [getattr(decided(f"{whole_declaration} task-server/task submit -- bash .claude/gate.d/{name}"), "code", "抛了异常") for name in COMMIT_GATE_STAGE_NAMES]
    results.append(("task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒", [2] * len(COMMIT_GATE_STAGE_NAMES), six_codes))
    # 崩溃域闸点到的项或变异名不在登记里：放行，判不出的那一处记进判不了的清单（进检出记录的「范围声明闸判不了」）
    for label, command, name in (
            ("崩溃点重放点名的项不在合表里：放行、记一处判不了", f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item sample-missing-item", "sample-missing-item"),
            ("证红点名的变异名不在变异表里：放行、记一处判不了", f"bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-missing-row", "sample-missing-row")):
        decision = decided(command)
        results.append((label, (0, True), (decision.code, any(name in entry for entry in decision.unjudged)) if not isinstance(decision, str) else decision))
    # 拒绝说明的出路点对缩域变量（target_shrink_variables）：登记过的崩溃枚举目标的变异行点放量那个，源码里只写了判器那个变量名的目标点判器那个。
    #   (退出码, 出路里有没有「放量那个=<n>」, 有没有「判器那个=<n>」)
    def way_out_names(decision):
        if isinstance(decision, str):
            return decision
        message = decision.message or ""
        return (decision.code, f"{CRASH_SHRINK_VARIABLES[0]}=<n>" in message, f"{CRASH_SHRINK_VARIABLES[1]}=<n>" in message)
    mutation_way_out = decided(f"{scope_text('items', '变异门禁点名一行', '1 项')} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .")
    results.append(("变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", (2, True, False), way_out_names(mutation_way_out)))
    cargo_way_out = decided(f"{scope_text('target', '验证包一个测试目标', '1 个测试目标')} cargo test -p {VERIFICATION_PACKAGE} --test crash_judge_sample_every_state")
    results.append(("cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量", (2, False, True), way_out_names(cargo_way_out)))
    # 设的缩域变量不是点到的目标读的那一个：拒，说明点名该设的那一个（出路里只有「放量那个=<n>」）
    wrong_variable = decided(f"{scope_text('items', '变异门禁点名一行', '1 项')} {CRASH_SHRINK_VARIABLES[1]}=4 bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .")
    results.append(("变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个", (2, True, False), way_out_names(wrong_variable)))
    # 判不出点到的目标读哪一个缩域变量（登记过、源码在 tests/<目标>/main.rs）：设了两个之一照放行，那一处记进判不了的清单
    undetermined = decided(f"{scope_text('target', '验证包一个测试目标', '1 个测试目标')} {CRASH_SHRINK_VARIABLES[1]}=4 "
                           f"cargo test -p {VERIFICATION_PACKAGE} --test crash_enumeration_sample_directory")
    results.append(("cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", (0, True),
                     (undetermined.code, any("crash_enumeration_sample_directory" in entry and "读哪一个缩域变量" in entry for entry in undetermined.unjudged))
                     if not isinstance(undetermined, str) else undetermined))
    # 命令点明的副本里与项目根都找不到的变异名：放行，记的判不了里写读过哪几棵树；--copy 写成展开不出值的变量：记的判不了点名那个变量
    def unjudged_mentions(decision, *texts):
        if isinstance(decision, str):
            return decision
        return (decision.code, any(all(text in entry for text in texts) for entry in decision.unjudged))
    results.append(("证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", (0, True),
                    unjudged_mentions(decided(f"bash scripts/mutation/prove-red.sh --copy copy-tree {VERIFICATION_PACKAGE} sample-missing-row"), "sample-missing-row", "copy-tree")))
    results.append(("证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量", (0, True),
                    unjudged_mentions(decided(f"bash scripts/mutation/prove-red.sh --copy $UNSET_TREE {VERIFICATION_PACKAGE} sample-missing-row"), "$UNSET_TREE", "展开不出")))
    # 范围对不上的说明写这条命令该写的整段「范围=<数> <单位>」（数与单位都是闸自己数的那一个）与数法；命令里有变量拼的词时另写「把 --item 写成字面」。
    #   断言找带引号的「范围=…」：拒绝文本末尾的声明模板行也打不带引号的 范围=…，那一处不经 declaration_to_write，不能当说明里写了这一句的凭据
    def refusal_mentions(decision, *texts):
        if isinstance(decision, str):
            return decision
        return (decision.code, all(text in (decision.message or "") for text in texts))
    results.append(("范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法", (2, True),
                    refusal_mentions(decided(f"{scope_text('items', '门禁 code-tooling 一格加两项', '2 项')} "
                                             "bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item first-item --item second-item ."), "「范围=3 项」", "各自去重后相加")))
    results.append(("范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验", (2, True),
                    refusal_mentions(decided(f"{scope_text('items', '复跑 E207', '2 行')} bash scripts/experiment-apparatus/replay.sh E207"), "「范围=1 个实验」")))
    results.append(("声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面", (2, True),
                    refusal_mentions(decided(f"items='--item a --item b'; {scope_text('items', '门禁 code-tooling 两项', '2 项')} bash .claude/gate.d/code-tooling.sh $items ."),
                                     "「范围=整轮」", "把 --item 写成字面", "$items")))
    return results


def selftest_entry_cells(work, guard_script):
    """另起入口（run-scope-guard.sh）喂 hook 的 JSON：退出码、stderr 的说明、检出记录与上次红的清单。交回 [(说明, 期望, 实际)]。"""
    results = []
    detections = os.path.join(work, "detections.jsonl")
    last_runs = os.path.join(work, "last-runs")
    os.makedirs(last_runs, exist_ok=True)
    with open(os.path.join(last_runs, ".claude__gate.d__doc-text.sh.tsv"), "w", encoding="utf-8") as handle:
        handle.write("# 上次这一道红的\nclock-times\nprime-marks\n")
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, SINGLEFS_LAST_RUNS_DIRECTORY=last_runs, CLAUDE_PROJECT_DIR=work)

    def feed(command, agent_type=None, session="s-run-scope"):
        hook_input = {"tool_name": "Bash", "cwd": work, "session_id": session, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        return subprocess.run(["bash", guard_script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=120)

    refused = feed("bash .claude/gate.d/doc-text.sh .")
    results.append(("入口：没带声明拒（退出码 2）", 2, refused.returncode))
    template = f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text.sh 的全部格（不带 --check / --item）; 范围=整轮; 为什么=<"
    results.append(("入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法", True,
                    all(text in refused.stderr for text in ("跑的是：门禁 doc-text.sh 的全部格", template, "--check <格>", "--exact", "rerun-failed-tests.py",
                                                             "挑测试目标在脚本里做"))))
    results.append(("入口：拒绝说明附上次红的清单", True, "clock-times、prime-marks" in refused.stderr))
    no_record = feed("bash .claude/gate.d/doc-experiments.sh .")
    results.append(("入口：没有上次红的记录时照写", True, "没有上次红的记录" in no_record.stderr))
    sub_submitted = feed(f"{scope_text('whole', '门禁 doc-text 全部格', '整轮')} task-server/task submit -- bash .claude/gate.d/doc-text.sh .", "implementation-writer")
    results.append(("入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回", (2, True),
                    (sub_submitted.returncode, "写进报告「要主 agent 跑的命令」一节交回" in sub_submitted.stderr)))
    template_lines = [line for line in sub_submitted.stderr.splitlines() if line.startswith(f"  {DECLARATION_VARIABLE}=")]
    results.append(("入口：命令开头已有声明时模板只留一份", [1], [line.count(f"{DECLARATION_VARIABLE}=") for line in template_lines]))
    if os.path.exists(detections):
        os.remove(detections)
    allowed_command = f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} bash .claude/gate.d/code-tooling.sh --check agent-write-scope,research-script-selftests ."
    allowed = feed(allowed_command)
    entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            entries = [json.loads(line) for line in handle if line.strip()]
    recorded = [entry for entry in entries if entry.get("outcome") == "放行"]
    good_record = (len(recorded) == 1 and "session_id" not in recorded[0] and recorded[0].get("scope_session_id") == "s-run-scope"
                   and recorded[0]["run_scope"]["segments"].get("范围") == "2 格" and recorded[0]["run_scope"]["kind_of_command"] == "items"
                   and recorded[0]["command"] == allowed_command)
    results.append(("入口：放行的记进检出记录（四段与命令）", (0, True), (allowed.returncode, good_record)))
    feed("bash .claude/gate.d/doc-text.sh .")
    refused_entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            refused_entries = [json.loads(line) for line in handle if line.strip() and json.loads(line).get("outcome") == "被拒"]
    results.append(("入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）", True,
                    len(refused_entries) == 1 and refused_entries[0].get("session_id") == "s-run-scope"
                    and refused_entries[0]["findings"][0].startswith("范围声明被拒")))
    plain = feed("ls -la")
    results.append(("入口：别的命令放行、不写 stderr", (0, ""), (plain.returncode, plain.stderr)))
    # 放宽检查的提交标识：标准输出整份是一个 JSON 对象（Claude Code 才认得），退 0、permissionDecision 是 ask，理由给用户看
    def asked_reason(run):
        try:
            output = json.loads(run.stdout)["hookSpecificOutput"]
        except (ValueError, KeyError, TypeError):
            return None
        return output.get("permissionDecisionReason") if output.get("permissionDecision") == "ask" and output.get("hookEventName") == "PreToolUse" else None
    flagged_command = f"{scope_text('whole', '提交前的整轮门禁', '整轮')} task-server/task submit --full-gate -- bash scripts/gate/gate-staged.sh"
    flagged = feed(flagged_command)
    flagged_reason = asked_reason(flagged) or ""
    results.append(("入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）", (0, True),
                    (flagged.returncode, all(text in flagged_reason for text in ("--full-gate", LOOSENING_SUBMIT_FLAGS["--full-gate"], f"命令全文：{flagged_command}", SUBMIT_FLAG_USER_RULE)))))
    unread = feed("task-server/task submit $sample_options -- ls")
    results.append(("入口：参数是变量弹窗、理由写认不出带没带标识", (0, True), (unread.returncode, "参数里有变量，认不出带没带标识" in (asked_reason(unread) or ""))))
    unflagged = feed("task-server/task submit -- ls")
    results.append(("入口：不带标识的提交不弹窗（标准输出是空的）", (0, ""), (unflagged.returncode, unflagged.stdout)))
    commit_unflagged = feed(f"{scope_text('whole', '门禁 lkmm 全部格', '整轮')} task-server/task submit -- bash .claude/gate.d/lkmm.sh")
    results.append(("入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）", (2, True, True),
                    (commit_unflagged.returncode, all(text in commit_unflagged.stderr for text in ("commit-stages.md", "--full-gate")),
                     "照抄声明模板" not in commit_unflagged.stderr)))
    # 经 submit 交的 cargo test 只点目标：拒，说明写没点名与改成点名的样子（`-- <测试全名> --exact`），声明写对了不附声明模板；被拒的记一行
    unnamed_command = f"{scope_text('target', '验证包一个测试目标', '1 个测试目标')} task-server/task submit -- cargo test -p {VERIFICATION_PACKAGE} --test plain_sample"
    unnamed = feed(unnamed_command, session="s-unnamed")
    unnamed_entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            unnamed_entries = [entry for entry in (json.loads(line) for line in handle if line.strip())
                               if entry.get("session_id") == "s-unnamed" and entry.get("outcome") == "被拒"]
    results.append(("入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）", (2, True, True, 1),
                    (unnamed.returncode, all(text in unnamed.stderr for text in ("启动测试与验证没点名", "-- <测试全名> --exact", "→ 怎么办：")),
                     "照抄声明模板" not in unnamed.stderr, len(unnamed_entries))))
    domain_unshrunk = feed(f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item sample-stream-item")
    results.append(("入口：崩溃域没缩拒（退出码 2、说明写缩域变量与全域理由的写法）", (2, True),
                    (domain_unshrunk.returncode, all(text in domain_unshrunk.stderr for text in (CRASH_SHRINK_OPTION, CRASH_SHRINK_VARIABLES[1], "全域理由：")))))
    # 判不了的照常放行、记一条检出（结局「只记」）：hook 输入不是 JSON 对象（stderr 打带钩子名与开头字节的诊断，会话号写读不到）、
    # 判的时候抛异常（RUN_SCOPE_GUARD_BREAK 里加一个认不出的项，guard_switch_on 抛 ValueError）、here-string 喂给 bash 的是变量。
    # RUN_SCOPE_GUARD_BREAK=failure-unrecorded 下这三格红，LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下第三格红
    def recorded_entries():
        if not os.path.exists(detections):
            return []
        with open(detections, encoding="utf-8") as handle:
            return [json.loads(line) for line in handle if line.strip()]
    before = len(recorded_entries())
    unreadable = [subprocess.run(["bash", guard_script], input=raw, capture_output=True, env=environment, timeout=120)
                  for raw in (b'{"tool_name": "Bash", "tool_input": ', b'["Bash"]')]
    added = recorded_entries()[before:]
    results.append(("入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", ([0, 0], True, 2, ["读不到", "读不到"]),
                    ([run.returncode for run in unreadable],
                     all(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象" in run.stderr.decode("utf-8", "replace") for run in unreadable)
                     and "b'[\"Bash\"]'" in unreadable[1].stderr.decode("utf-8", "replace"),
                     len(added), [entry.get("session_id") for entry in added])))
    before = len(recorded_entries())
    # 关守卫项 honor-outside-selftest 与别的项同写会被入口的守卫先拒（第一百二十二节 ②），注入认不出的项时摘掉它
    injected = ",".join([item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item and item != "honor-outside-selftest"] + ["not-a-switch"])
    raising = subprocess.run(["bash", guard_script], input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise",
                             "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}), capture_output=True, text=True,
                             env=dict(environment, **{BREAK_VARIABLE_NAME: injected}), timeout=120)
    added = recorded_entries()[before:]
    results.append(("入口：判的时候抛异常放行、stderr 报一句、记一条检出", (0, True, 1, "s-raise", "只记"),
                    (raising.returncode, "没判成" in raising.stderr, len(added), added[0].get("session_id") if added else None,
                     added[0].get("outcome") if added else None)))
    raising_flagged = subprocess.run(["bash", guard_script], input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise-flagged",
                                     "tool_input": {"command": flagged_command}}), capture_output=True, text=True,
                                     env=dict(environment, **{BREAK_VARIABLE_NAME: injected}), timeout=120)
    results.append(("入口：判的时候抛异常、命令文字里提到标识：弹窗", (0, True), (raising_flagged.returncode, "--full-gate" in (asked_reason(raising_flagged) or ""))))
    before = len(recorded_entries())
    fed_variable = feed('bash <<< "$CMD"', session="s-fed")
    added = [entry for entry in recorded_entries()[before:] if entry.get("session_id") == "s-fed"]
    results.append(("入口：here-string 喂给 bash 的是变量放行、记一条检出", (0, 1, True, "只记"),
                    (fed_variable.returncode, len(added), bool(added) and any("here-string `$CMD`" in finding for finding in added[0].get("findings", [])),
                     added[0].get("outcome") if added else None)))
    # 判法文件读不到：照常放行、报一句、记一条检出（钩子挂在每一条 Bash 上，坏了不许把所有命令都拒掉）
    # 只拷入口与 lib_work_root.py（入口记检出时取默认位置用）；它 source 的 .claude/scripts/preflight.sh 与那一份找的规范副本 .claude/singlefs-ai-sop/ 链接到钩子所在的仓。
    #   入口自己记：AGENT_HOOK_DETECTIONS 设了记那里，没设记进 lib_work_root.py 的默认位置（这里把 SINGLEFS_WORK_ROOT 指进临时目录）。failure-unrecorded 下记检出的两格红
    lonely = os.path.join(work, "lonely", ".claude", "hooks")
    os.makedirs(lonely, exist_ok=True)
    shutil.copy(guard_script, os.path.join(lonely, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(guard_script), "lib_work_root.py"), os.path.join(lonely, "lib_work_root.py"))   # 入口记检出时取默认位置
    claude_directory = os.path.dirname(os.path.dirname(os.path.abspath(guard_script)))
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "lonely", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    orphan_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-orphan", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}})
    before = len(recorded_entries())
    orphan = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_entries()[before:]
    results.append(("入口：判法文件读不到时放行并报一句", (0, True), (orphan.returncode, "照常执行" in orphan.stderr)))
    results.append(("入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", (1, "s-orphan", "只记", GUARD_SCRIPT_NAME, True),
                    (len(added), added[0].get("session_id") if added else None, added[0].get("outcome") if added else None,
                     added[0].get("hook") if added else None, bool(added) and "读不到判法" in " ".join(added[0].get("findings") or []))))
    default_detections = os.path.join(work, "orphan-work-root", "agent-hook-detections.jsonl")

    def default_entries():
        try:
            with open(default_detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []
    before = len(default_entries())
    without_variable = {key: value for key, value in environment.items() if key not in (DETECTIONS_VARIABLE, "SINGLEFS_WORK_ROOT_BREAK")}
    orphan_default = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, timeout=60,
                                    env=dict(without_variable, SINGLEFS_WORK_ROOT=os.path.dirname(default_detections)))
    added = default_entries()[before:]
    results.append(("入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置", (0, 1, "s-orphan", "只记"),
                    (orphan_default.returncode, len(added), added[0].get("session_id") if added else None, added[0].get("outcome") if added else None)))
    # 判法文件在而没跑完（这里放一份导入就抛异常的 lib_run_scope.py）：入口照常放行、stderr 报一句、自己记一条检出（会话号照输入）；
    #   judge-crash-unrecorded 下这一格红（只打 stderr、不记）
    crashing = os.path.join(work, "crashing", ".claude", "hooks")
    os.makedirs(crashing, exist_ok=True)
    shutil.copy(guard_script, os.path.join(crashing, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(guard_script), "lib_work_root.py"), os.path.join(crashing, "lib_work_root.py"))
    with open(os.path.join(crashing, "lib_run_scope.py"), "w", encoding="utf-8") as handle:
        handle.write("raise RuntimeError('自证造的导入错')\n")
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "crashing", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    before = len(recorded_entries())
    crashed = subprocess.run(["bash", os.path.join(crashing, GUARD_SCRIPT_NAME)],
                             input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-crash", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}),
                             capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_entries()[before:]
    results.append(("入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）", (0, True, 1, "s-crash", "只记", True),
                    (crashed.returncode, "没跑完" in crashed.stderr, len(added), added[0].get("session_id") if added else None,
                     added[0].get("outcome") if added else None, bool(added) and "没跑完" in " ".join(added[0].get("findings") or []))))
    # 驱动脚本在而读不了（chmod 成 000）：照常放行、这一条命令记一行检出（「范围声明闸判不了」起头）；driver-unread-unrecorded 下这一格红
    locked_driver = os.path.join(work, "locked-driver.sh")
    with open(locked_driver, "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\nbash .claude/gate.d/doc-text.sh .\n")
    os.chmod(locked_driver, 0)
    before = len(recorded_entries())
    try:
        locked_run = feed("bash ./locked-driver.sh", session="s-locked")
    finally:
        os.chmod(locked_driver, 0o600)
    added = [entry for entry in recorded_entries()[before:] if entry.get("session_id") == "s-locked"]
    results.append(("入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）", (0, 1, "只记", True),
                    (locked_run.returncode, len(added), added[0].get("outcome") if added else None,
                     bool(added) and any(str(finding).startswith("范围声明闸判不了") and "读不了" in str(finding) for finding in added[0].get("findings") or []))))
    # 弄坏开关只在自证里认：不带标记、或标记文件内容对不上的平常跑设了开关，入口在读 hook 的 JSON 之前退 2、stderr 点名它只在自证里认（探法同门禁
    #   code-tooling 的 break-switch-outside-selftest 格）；带标记的自证子进程照认开关。honor-outside-selftest 设着时探的开关里也带上它，前两格红。检出写另一份，不搅前面几格的数
    honored = "honor-outside-selftest" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    probe_item = "honor-outside-selftest" if honored else "declaration-optional"
    unmarked = {key: value for key, value in environment.items() if key != SELFTEST_MARK_VARIABLE}
    unjudged = os.path.join(work, "outside-selftest-detections.jsonl")
    plain = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                           env=dict(unmarked, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：平常跑设了弄坏开关退 2、点名它只在自证里认", (2, True),
                    (plain.returncode, any(BREAK_VARIABLE_NAME in line and "自证" in line for line in plain.stderr.splitlines()))))
    forged_mark = os.path.join(work, "forged-selftest-mark")
    with open(forged_mark, "w", encoding="utf-8") as handle:
        handle.write("别处写的标记\n")
    plain_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "ls -la"}})
    forged = subprocess.run(["bash", guard_script], input=plain_input, capture_output=True, text=True, timeout=120,
                            env=dict(unmarked, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: unjudged, SELFTEST_MARK_VARIABLE: forged_mark}))
    results.append(("入口：标记文件内容对不上照拒", 2, forged.returncode))
    marked_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}})
    marked = subprocess.run(["bash", guard_script], input=marked_input, capture_output=True, text=True, timeout=120,
                            env=dict(environment, **{BREAK_VARIABLE_NAME: "declaration-optional", DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：自证起的子进程（带标记）照认弄坏开关", 0, marked.returncode))
    # 关守卫项 honor-outside-selftest 只在单独一项时认（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）：与别的项同写，平常跑与带标记的子进程都退 2、点名它；
    #   honor-outside-selftest-with-others 设着时带标记那一格的开关里也带上它（同写照认），那一格红
    combined_item = "honor-outside-selftest,declaration-optional"
    combined_plain = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                    env=dict(unmarked, **{BREAK_VARIABLE_NAME: combined_item, DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：关守卫项与别的项同写：平常跑退 2、点名它只在单独一项时认", (2, True),
                    (combined_plain.returncode, "honor-outside-selftest" in combined_plain.stderr and "唯一一项" in combined_plain.stderr)))
    lenient = "honor-outside-selftest-with-others" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    combined_marked = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                     env=dict(environment, **{BREAK_VARIABLE_NAME: combined_item + (",honor-outside-selftest-with-others" if lenient else ""),
                                                              DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它", (2, True),
                    (combined_marked.returncode, "honor-outside-selftest" in combined_marked.stderr and "唯一一项" in combined_marked.stderr)))
    return results


def failed_labels(results):
    return [label for label, want, got in results if want != got]


def selftest(hook_directory):
    load_judging_libraries()
    guard_script = os.path.join(hook_directory, GUARD_SCRIPT_NAME)
    work = tempfile.mkdtemp(prefix="run-scope-guard-")
    # 外面设了弄坏开关：照它判一遍、点名的格应当红，不跑开关轮（开关轮自己逐个设）
    outside_break = os.environ.get(BREAK_VARIABLE_NAME) or None
    try:
        build_selftest_tree(work)
        # 自证起的入口带这个标记才认弄坏开关（入口的守卫；不带标记的平常跑设了开关退 2）
        selftest_mark = os.path.join(work, "selftest-mark")
        with open(selftest_mark, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_MARK_CONTENT + "\n")
        os.environ[SELFTEST_MARK_VARIABLE] = selftest_mark
        results = selftest_judge_cells(work) + selftest_entry_cells(work, guard_script)
        failures = failed_labels(results)
        for label, want, got in results:
            if want != got:
                print(f"  ✗ 自检：{label}：应当是 {want!r}，实际 {got!r}")
                print(f"  → 怎么办：照 {label} 那一格的判法改 lib_run_scope.py，再跑 bash .claude/hooks/run-scope-guard.sh --selftest")
        if outside_break:
            print(f"  （{BREAK_VARIABLE_NAME}={outside_break} 下判了 {len(results)} 格，红 {len(failures)} 格；开关轮不跑）")
            return 1 if failures else 0
        labels = [label for label, _want, _got in results]
        try:
            with open(guard_script, encoding="utf-8") as handle:
                header = handle.read()
        except OSError:
            header = ""
        switch_problems = []
        for switch, cells in BREAK_SWITCH_CELLS.items():
            missing = [cell for cell in cells if cell not in labels]
            if missing:
                switch_problems.append(f"开关 {switch} 点名的格不在自证里：{'、'.join(missing)}")
                continue
            if not re.search(r"^#\s+" + re.escape(f"{BREAK_VARIABLE_NAME}={switch}") + r"\s", header, re.M):
                switch_problems.append(f"开关 {switch} 没写进 {GUARD_SCRIPT_NAME} 的文件头（一行「{BREAK_VARIABLE_NAME}={switch}  <弄坏哪一处>」，下一行起写红的格）")
            os.environ[BREAK_VARIABLE_NAME] = switch
            try:
                broken = selftest_judge_cells(work)
                if any(cell.startswith("入口：") for cell in cells):   # 另起入口的几格慢，只在开关点名了它们时再跑
                    broken += selftest_entry_cells(work, guard_script)
            finally:
                os.environ.pop(BREAK_VARIABLE_NAME, None)
            reds = failed_labels(broken)
            still_green = [cell for cell in cells if cell not in reds]
            if still_green:
                switch_problems.append(f"开关 {switch} 打开后这几格没红：{'、'.join(still_green)}")
            unexpected = [cell for cell in reds if cell not in cells]
            if unexpected:
                switch_problems.append(f"开关 {switch} 打开后另有这几格红、表里没写：{'、'.join(unexpected)}（写进 BREAK_SWITCH_CELLS 与文件头）")
        for problem in switch_problems:
            print(f"  ✗ 自检：{problem}")
        if failures or switch_problems:
            print(f"  → 怎么办：照上面点名的格改 {os.path.join(hook_directory, 'lib_run_scope.py')} 的判法（或 {GUARD_SCRIPT_NAME} 的文件头），再单独跑 "
                  f"bash .claude/hooks/{GUARD_SCRIPT_NAME} --selftest 看它转绿")
            return 1
        memory_count = len([label for label in labels if not label.startswith("入口：")])
        print(f"  ✓ {GUARD_SCRIPT_NAME} 自检通过：{len(results)} 格（内存里判 {memory_count} 格、另起入口 {len(results) - memory_count} 格）；"
              f"开关轮 {len(BREAK_SWITCH_CELLS)} 个开关逐个打开，红的格与表里点名的逐格相等（共 {sum(len(cells) for cells in BREAK_SWITCH_CELLS.values())} 格次）")
        return 0
    finally:
        if outside_break:
            os.environ[BREAK_VARIABLE_NAME] = outside_break
        os.environ.pop(SELFTEST_MARK_VARIABLE, None)
        shutil.rmtree(work, ignore_errors=True)


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认（与入口脚本开头那一道同一个判法，免得直接起这份库绕过它）：参数不是单独一个 --selftest、自证标记文件对不上而 RUN_SCOPE_GUARD_BREAK 非空，
    交回拒绝的几行（点名变量、说它只在自证里认）；关守卫项 honor-outside-selftest 单独一项时放行，与别的项同写照拒（带标记又同写着 honor-outside-selftest-with-others 的除外）；放行交回空表。"""
    value = os.environ.get(BREAK_VARIABLE_NAME, "")
    if not value or arguments == ["--selftest"]:
        return []
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    marked = False
    if mark and os.path.isfile(mark):
        with open(mark, encoding="utf-8", errors="replace") as handle:
            marked = handle.read().rstrip("\n") == SELFTEST_MARK_CONTENT
    items = [item for item in value.split(",") if item]
    guard_off = "honor-outside-selftest" in items
    others = [item for item in items if item != "honor-outside-selftest"]
    lenient = "honor-outside-selftest-with-others" in others
    if guard_off and others and not (marked and lenient):
        return [f"  ✗ {BREAK_VARIABLE_NAME}={value}：关守卫项 honor-outside-selftest 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；弄坏开关只在自证里认",
                f"  → 怎么办：去掉 {BREAK_VARIABLE_NAME} 再跑；要证红就跑 {BREAK_VARIABLE_NAME}=<项> bash .claude/hooks/run-scope-guard.sh --selftest"]
    if not marked and not (guard_off and not others):
        return [f"  ✗ {BREAK_VARIABLE_NAME}={value} 是弄坏开关，只在自证（run-scope-guard.sh --selftest 与它起的子进程）里认；平常跑设了就拒，这一趟什么都没做",
                f"  → 怎么办：去掉 {BREAK_VARIABLE_NAME} 再跑；要证红就跑 {BREAK_VARIABLE_NAME}=<项> bash .claude/hooks/run-scope-guard.sh --selftest"]
    return []


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
