#!/usr/bin/env bash
# PreToolUse hook（SubagentHandback）：子 agent 交回之前拦七样——自己起的后台任务还在跑（全部子 agent），交回正文太长、三方腿报告里的引文对不上、
# 书记员与执行员的绿行不报数、实现员执行员工具员改了 .rs 没跑命名检查、写回员规格的目标不在又没写进「找不到的目标」、执行员报告里的登记落地清单没有或指不到
# （这六样只判项目子 agent）。
# hook-events: PreToolUse:SubagentHandback
# gate-similar: handback-scratch-check.sh 同挂 PreToolUse[SubagentHandback]，但它判的是临时目录里自己建的编译目录与仓副本删没删（上游副本，本仓不许就地改），判据与这里四样没有一条相同（④ 判的是自己起的后台任务还在不在跑，不是临时目录），并进去要改上游
# gate-similar: continuation-guard.sh 也按 agent 号去主会话目录下找子 agent 的会话记录，但它挂 SendMessage、判收件人交回过没有；这里只取子 agent 第一条记录的时刻当「开工时刻」，找法两行，不值得抽共用
# admission: always Claude Code 在子 agent 每次调交回工具之前调它，判的是这一次交回的正文与它点名的报告
# run-condition: command python3
#
# 判法。任一条成立就拒（退出 2，stderr 写原因与出路）：
#   ④ 全部子 agent（输入里有 agent_id、不是 main；general-purpose 也判，主 agent 不判）：它自己起的后台任务（它会话记录
#      <主会话记录去掉 .jsonl>/subagents/agent-<agent_id>.jsonl 里 run_in_background 的工具结果、或前台跑满超时被挪进后台的，带输出文件路径的那些）
#      此刻还有进程开着输出文件，就是还在跑：拒，按任务列出最上层的进程号、已跑多久、命令与整棵子树从最底层往上的停止次序，
#      出路是结束本轮等完成通知再交回，或按进程号逐个停掉再交回。认法与看门狗「交回之后后台还在跑」同一套，从 scripts/agents/agent-watch.py 导入
#      （background_start_in_result、own_background_tasks_still_running），不另抄一份。命令里自己用 `&`、`nohup` 放出去又改了输出去向的进程认不出（与看门狗同一处射程）。
#      找不到它的会话记录、导入不了 agent-watch.py 就不判这一条，stderr 写一句。
# 下面几条只判项目子 agent（`.claude/agents/<agent_type>.md` 在的）；主 agent、general-purpose 这类不判：
#   ① 交回正文超过 HANDBACK_CHARACTER_LIMIT 个字符（按 python 的 len 数，默认 4000；原 2000 时 8 天里拒了 67 次，每次都在最大的上下文上重写整份交回）：全文写进报告文件，交回只写结论、报告路径与 sha256；
#      拒绝说明写明文件名：三方腿写派发提示给的 research/prompts/<轮>-*-output.md，别的类型写派发提示给的报告路径，没给就写草稿目录的 result.md；
#   ② 三方腿（类型表只登记在定义 frontmatter 的 leg: three-way 一行，按项目根经同目录 lib_three_way_legs.py 读，citation_checked_agents；
#      派发闸、骨架与账表读同一处，拆分提案第九十节 ⑯；读不到那份库这一条不判，stderr 写一句）：交回里点名的 research/prompts/<名>-output.md
#      （认法只在 lib_three_way_legs.py 的 leg_report_mentions_in_trees，与派发闸 ⑰ 的 leg_report_mentions 是同一处：子目录下的、没填的 <轮> 占位都不认，判决 landing-audit-r4 K2-5；
#      只认解到本仓某一棵工作树（项目根，与 git worktree list 列出的登记过的各棵）下 research/prompts/ 那一层的，不在任何一棵里的仓外副本不认，
#      判决 landing-audit-r5「交回闸 ② 仓外报告」，拆分提案第九十节 ㉜；取不到 worktree 列表时只认项目根，拒绝说明里写明）按所在那一棵分组交给
#      scripts/agents/cite-check.py（--root 那一棵，--unchanged-since 取这个子 agent 会话记录第一条的时刻），有对不上的拒；交回里一份报告都没点名也拒；
#      $SINGLEFS_WORK_ROOT/、${SINGLEFS_WORK_ROOT}/ 起头的先照同目录 lib_work_root.py 展开成工作根再认（拆分提案第九十节 ㉞）；只写相对路径（前面没有树根，或 ./）的，
#      这个子 agent 在别的登记过的工作树里干过活（会话记录里工具调用的参数与 cwd 点名到那一棵）、那一棵里有这份的按那一棵核，没有而项目根下有的按项目根，
#      哪一棵都没有的拒（拆分提案第九十节 ㉟；改前按项目根、文件不在时 cite-check 退 2 放行）；
#      找不到它的会话记录（认不出开工时刻）就不判这一条，stderr 写一句；
#   ③ kb-scribe / experiment-runner：交回正文与它点名的报告文件（research/prompts/、工作根或过渡期旧根下的 .md；两个根怎么认只在同目录 lib_work_root.py）里，以 ✓ 起头的行没有数字、
#      或写「查了 0 项」这类零项的，拒，逐行列出——绿行要连它报的查了多少项一起贴，0 项或没有数的按没判写。
#   ⑤ implementation-writer / experiment-runner / tooling-writer（拆分提案第九十节 ⑥，共用约束「门禁」一节「命名检查交回前跑」）：它自己的会话记录里
#      用 Edit / Write / MultiEdit 改成了 crates/ 或 scripts/ 下的 .rs（工具结果不是错误），或 Bash 里写了这样的 .rs（输出重定向 >、>>、>|、&> 的目标，tee 的文件参数，
#      cp / mv 的目标：目标是 .rs，或目标是目录、来源是 .rs；-t 照认；目标里的变量照这一段赋过的值展开；拆分提案第九十节 ㉕；同一条里先算写、再算命名检查；
#      目标以命令替换起头、接着 /crates/ 或 /scripts/ 的按仓根展开：$(pwd) 取命令当时的目录，$(git rev-parse --show-toplevel) 与别的写法取命令当时的目录往上第一层带 .git 的，
#      找不到取项目根，拆分提案第一百二十二节 ⑪），
#      改动所在的树是主工作区（CLAUDE_PROJECT_DIR）或带 .git 的副本（git worktree），
#      而在那棵树最后一次改 .rs 之后，没有真跑过的那两条命令：门禁 code-source-discipline.sh 的 --check 点名 vague-names、test-file-names、ordinal-names、
#      data-names 四格（不带 --check 的整道也算），与上游 naming-lint.sh；两条的根都要解到那棵树（根里的 ~、$HOME、$PWD、$(pwd) 按命令当时的目录展开，
#      $(git rev-parse --show-toplevel) 按命令当时的目录往上找到的第一层带 .git 的目录认；不给根时按它们自己的默认：前者取脚本往上两级，后者从当前目录往上找 .git）。
#      「真跑过」：Bash 调用有工具结果，不是错误、或错误是「Exit code N」；被钩子拒的、超时的、没有结果的不算。只看命令，不看报告里写了什么。
#      「跑完」（判决 landing-audit-r4 K3-1，sequential_naming_text）：两条要在顺序执行的位置上——不在 || 右侧、不在 if / while / until / for / select / case
#      与函数体里、同一段里前面只有 cd、赋值、env、export 与另一条命名检查用 && 连着、不放后台、前面没有 exit / return / exec；再按命名检查自己的退出码收
#      （判决 landing-audit-r5 K2-2a、K2-2b，sequential_naming_text）：只收自己的退出码认得出、是 0（通过）或 1（判红）的；3、77 是没跑，124、137 是被 timeout 杀掉，
#      141 是接 | head 被 SIGPIPE 杀掉，2 是用法错，都不收。自己的退出码从它所在那一段（&& / || 连成的一段）的退出码认：顶层最后一段取这一条 Bash 的退出码
#      （退非 0 时整条只许有赋值、env、cd、export 与命名检查，set -e 这类中途停下的写法认不出）；后面紧跟「; echo <标签>$?」（换行也算）的那一段取输出里那一行打出的数
#      （同一种标签打了几次按次序配，输出里那种行的行数对不上的不认）；「; echo <标签>${PIPESTATUS[k]}」打出的是紧挨着的那条管道第 k 段自己的退出码（| tail 之后也认，
#      第 k 段要是一条简单命令；前面 && 连着 cd 或命名检查的只认 0）；紧跟一段的「名字=$?」（或 =${PIPESTATUS[k]}）把那一段存下，之后不在条件里的
#      「echo <标签>$名字」打出的数认成存下的那一段的（中间别的命令改写过这个名字的不认；拆分提案第九十节 ㉞ ㉟）。输出太长、记录里只有预览（<persisted-output>）的，
#      照预览里写的路径读那份存下来的整份输出再找那一行，读不到照预览认（拆分提案第九十节 ㉞）。它是那一段的最后一个元素时，那一段的退出码就是它的（前面 && 连着 cd 或命名检查的只认 0）；
#      后面接 && 的，那一段退 0 才认它退了 0；后面接 ||（|| true 这类把退出码吃掉）、在管道里不是最后一段（| head、| tail、| grep，退出码是后一段的）、前面有 ! 的认不出，不收。
#      包着 timeout 的照这一套收（没被杀掉时 timeout 的退出码就是命名检查自己的）。bash -c 那一段：外面那一条认出的退出码交进去，照同一套往里收。
#      命令认法：同目录 lib_shell_words.py 切命令位置（前缀、bash、capped.sh、run-with-memory-cap.sh 剥掉），lib_heavy_tests.py 的 command_under_launcher
#      剥 scripts/hosts/two-logs.sh 这类包装、checked_cells 读 --check。改动所在的树没有 .git 的（rsync 拷的副本，共用约束写明两条命令交主 agent 在补丁打进主工作区后跑）不判。
#      拒了就逐棵树列出最后改的 .rs、缺哪一条，给出两条命令原样（<根> 填那棵树）；缺的那一条分两种写（拆分提案第九十节 ㉞）：改完之后会话记录里一次都没起过的写「没跑过」，
#      起过而没认出跑完的写「跑了，但认不出它自己的退出码」与最后一次为什么（接了管道、接了 ||、没紧跟 echo rc=$?、自己退了 124 这类、persisted-output 读不到……），
#      出路照抄共用约束教的那一种落盘写法（NAMING_FILED_OUTPUT_FORM）。认不出的：Bash 里 cp -r 整个目录、install、sed -i、python 写的 .rs，改完又被删掉的副本（看不到 .git，按 rsync 副本不判）；
#      「跑完」只往 bash -c 里收一层，套在 capped.sh、two-logs.sh 这类包装里的 bash -c 那一段不再分位置；set -e 之下前面的命令失败、后面的不跑而整条退 1 的，只在整条里有别的命令时认得出。
#   ⑥ kb-scribe（拆分提案第九十节 ⑦，.claude/agents/kb-scribe.md「规格里的目标文件不存在」那一句）：规格里每一条的目标文件（kb-spec-check.py 的 load_spec /
#      parse_markdown_spec 认的两种写法，旧串、新串都在的条目才算）在主工作区（CLAUDE_PROJECT_DIR）不在，交回正文、交回点名的报告与派发提示「报告：」那一份里
#      都没有一节标题含「找不到的目标」、节里写着这个文件（从仓根起的路径或绝对路径）的，拒，逐个列出。规格从派发提示（它会话记录的第一条）里找：
#      提示点名的工作根或旧根下的 .json / .md / .txt（「报告：」那一份除外），「草稿目录：」下写回员自己起草的 spec.json / spec.md，提示正文本身（markdown 写法）；
#      「草稿目录：」一行只认行首的（认法只在同目录 lib_prompt_scratch_line.py，与派发闸 ⑯ ⑰、完工清理闸共用，列表里的是转述，判决 landing-audit-r4 K2-4）。
#      一份规格都认不出的不判（写回员停在起草之前交回）；导入不了 kb-spec-check.py、读不了会话记录的不判，stderr 写一句。
#   ⑦ experiment-runner（拆分提案第九十节 ①，.claude/agents/experiment-runner.md 第 3c 步、「产出」一节）：这一段写了装置或落了产物的（会话记录里 Edit / Write / MultiEdit
#      或 Bash 的重定向、tee 写了装置源文件——入库装置 src/bin/ 下、实验装置目录下，或 e<三位号>_ 起头的 .rs / .py / .sh，replay.sh 不算；或往 research/results/ 下写了东西），
#      交回正文、交回点名的报告与派发提示「报告：」那一份里要有一节标题含「登记落地清单」、节里的表至少一行；每行里的「文件:行」（.rs / .py / .sh / .toml / .tsv / .json /
#      .md / .txt 后面紧跟 :行号）要找得到那份文件、它有这一行（绝对路径照写；相对路径从项目根、派发提示「草稿目录：」点名的目录、读到的报告所在的目录起找，
#      那几处与会话记录里改过的文件里路径尾部相同的也算），每行最后一格里的用例名（小写、至少三段下划线连着的词；被 … 截短的不算）要在那几处与主工作区 crates/、scripts/ 的
#      .rs / .py / .sh 里 grep 得到。缺这一节、一行都没有、文件:行指不到、用例名 grep 不到，拒，逐条列出。派发提示写着「只修锚点」或「只补落产物」的不判（派发闸 ⑪ 放行的两种）：
#      只认行首标签与整句（一行去掉行首的 - * > _ 与粗体之后以那两种之一起头，后面是行尾或冒号、逗号、句号、分号、括号；句中的「不是只补落产物」不认，拆分提案第一百二十二节 ⑪）。
#      认不出的：只写「:行号」不写文件的（行号接在函数名后面、文件写在节首的那种）不核；只把装置挪进主工作区（cp / mv）、只写实验页的不判。
# 这一道自己出错（读不了报告、cite-check 跑不起来、JSON 解析不了）不拒：放行，错误写进 stderr。判不了而放行的几处（JSON 读不懂、判的时候抛异常、
#   读不到 lib_work_root.py、② ④ ⑤ ⑥ ⑦ 找不到自己的会话记录、读不到共用模块或 cite-check、agent-watch.py、kb-spec-check.py、切不开命令、报告或规格读不了，
#   UNJUDGED_SITES 逐处列名）照「放行但必记检出」：放行时检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path）
#   追加一行、结局「只记」，findings 以「交回正文闸没判全、照常放行」起头（record_unjudged）；拒的那一次照拒，不另记。
#   往拒的方向偏的几处不算放行、不记：⑤ 切不开的命令按命名检查没跑算、⑥ ⑦ 读不了的报告按那一节没写算、存成 persisted-output 读不到的只按预览认。
# 弄坏开关只在自证里认（.claude/gate.d/break-switch-outside-selftest-pending.tsv 文件头的改法）：平常跑（不是 --selftest，也不是自证起的子进程）设了 HANDBACK_GUARD_BREAK，
#   在读 hook 的 JSON 之前就退 2，stderr 点名它只在自证里认；自证起的子进程由环境里的 HANDBACK_GUARD_SELFTEST_MARK 认（指的那份文件在、内容是 SELFTEST_MARK_CONTENT，读完就摘掉）。
#
#   handback-guard.sh             # 从 stdin 读 hook 的 JSON
#   handback-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝
# 弄坏开关（只给自证判红用；自证里经 stdin 起的那一次入口照样带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里 cases 的标签，括号里是那条「✗ 自检：<标签>」失败行在标签之后的开头）：
#   HANDBACK_GUARD_BREAK=no-limit  decide() 不判 ① 交回正文的长度
#       超过上限的项目子 agent（「应当返回 2，实际 0」）
#       走真实 stdin 入口：超长交回（「应当返回 2，实际 0」）
#       超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=limit-hint-unnamed  ① 超长的拒绝说明退回改前那一句（只写「没给就写进草稿目录」，不写 result.md 与三方腿的文件名）
#       超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=no-cite  decide() 不判 ② 三方腿与核查员的引文
#       攻方腿引文行号错（「应当返回 2，实际 0」）
#       核查员没点名报告（「应当返回 2，实际 0」）
#       三方腿报告路径:只点名子目录下的 -output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:只点名没填的 <轮> 占位不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:只点名仓外同名的 …/research/prompts/r1-opus-output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:登记过的 worktree 里引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (0, False)」）
#       三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明（「应当返回 True，实际 False」）
#       三方腿类型表:样本仓登记的 sample-leg 没点名报告拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=cite-any-agent  ② 引文这一条对每个项目子 agent 都判，不只三方腿与核查员：交回里没点名 -output.md 的一律拒
#       正好在上限的项目子 agent（「应当返回 0，实际 2」）
#       回扫员不核引文（「应当返回 0，实际 2」）
#       「入口:读不到 lib_work_root.py 放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 (2, 0, None, False)」
#       书记员绿行带数（「应当返回 0，实际 2」）
#       回扫员的绿行不判（「应当返回 0，实际 2」）
#       起过的后台任务都跑完了（「应当返回 0，实际 2」）
#       三方腿类型表:样本仓没登记的攻方不核引文（「应当返回 0，实际 2」）
#       改了 .rs 之后两条命名检查都跑了放行（「应当返回 0，实际 2」）
#       只改 .md 放行（「应当返回 0，实际 2」）
#       rsync 副本里改的 .rs 交主 agent 放行（「应当返回 0，实际 2」）
#       git worktree 副本里改的、命名检查根写成副本根放行（「应当返回 0，实际 2」）
#       经 two-logs.sh 包着跑命名检查放行（「应当返回 0，实际 2」）
#       cd 之后根写成 $(pwd)、&& 连着两条跑命名检查放行（「应当返回 0，实际 2」）
#       回扫员改 .rs 不判命名检查（「应当返回 0，实际 2」）
#       timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行（「应当返回 0，实际 2」）
#       两条在同一条里各自 ; echo rc=$?（rc=0、rc=1 按次序配）放行（「应当返回 0，实际 2」）
#       Bash 里 mv 写 .rs 之后两条命名检查都跑了放行（「应当返回 0，实际 2」）
#       Bash 里把 .rs 拷到草稿目录不算改动放行（「应当返回 0，实际 2」）
#       根写成 $(git rev-parse --show-toplevel)、cd 进 worktree 副本里跑放行（「应当返回 0，实际 2」）
#       两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行（「应当返回 0，实际 2」）
#       输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行（「应当返回 0，实际 2」）
#       echo rc=$? 追加进落盘的那份文件、同一条里 tail 读出来放行（「应当返回 0，实际 2」）
#       退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行（「应当返回 0，实际 2」）
#       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
#       没跑的拒绝说明写没跑过、不写认不出退出码（「应当返回 True，实际 False」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
#       改了 .rs 没跑的拒绝说明给出两条命令（根填改动所在的树）（「应当返回 True，实际 False」）
#       写回员规格目标都在放行（「应当返回 0，实际 2」）
#       写回员规格目标不在、报告「找不到的目标」一节写了它放行（「应当返回 0，实际 2」）
#       写回员没有规格不判（「应当返回 0，实际 2」）
#       写回员提示里列表起头的「- 草稿目录：」是转述，不在那里找起草的规格（「应当返回 0，实际 2」）
#       写回员目标不在的拒绝说明列出文件与出路（「应当返回 True，实际 False」）
#       执行员登记落地清单的文件:行与用例名都找得到放行（「应当返回 0，实际 2」）
#       执行员登记落地清单写在交回正文里也认（「应当返回 0，实际 2」）
#       执行员只补落产物的派发不判登记落地清单（「应当返回 0，实际 2」）
#       执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单（「应当返回 0，实际 2」）
#       回扫员没有登记落地清单不判（「应当返回 0，实际 2」）
#       执行员没有登记落地清单的拒绝说明写出那一节与出路（「应当返回 True，实际 False」）
#       那个后台任务结束之后再交回放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=no-green-count  green_count_problems() 不判 ③ 绿行没有数
#       书记员绿行没数（「应当返回 2，实际 0」）
#       执行员报告文件里的绿行没数（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（~ 写法）（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（$SINGLEFS_WORK_ROOT 写法）（「应当返回 2，实际 0」）
#       执行员过渡期旧根下的报告里绿行没数（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=zero-count-ok  green_count_problems() 不判 ③ 绿行报 0 项
#       书记员绿行 0 项（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=judge-general-purpose  is_project_agent() 把带 agent_type 的一律当项目子 agent（general-purpose 也判 ①②③）
#       不是项目定义的 agent 不判（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=no-background  background_problems() 一律交空表，不判 ④ 自己起的后台任务还在跑
#       交回时自己起的后台任务还在跑（项目子 agent）（「应当返回 2，实际 0」）
#       交回时自己起的后台任务还在跑（通用 agent 也判）（「应当返回 2，实际 0」）
#       后台还在跑的拒绝说明列出进程号、命令与两条出路（等完成通知、按进程号停）（「应当返回 True，实际 False」）
#       「没判全:④ 找不到自己的会话记录记一处（background-transcript） 应当返回 True，实际 False」
#       「没判全:④ 读不到 agent-watch.py记一处（background） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=no-naming-check  decide() 不判 ⑤ 改了 .rs 没跑命名检查
#       改了 .rs 没跑命名检查拒（「应当返回 2，实际 0」）
#       执行员改了 scripts/ 下 .rs 没跑命名检查拒（「应当返回 2，实际 0」）
#       命名检查跑在最后一次改 .rs 之前拒（「应当返回 2，实际 0」）
#       只跑了一条命名检查拒（「应当返回 2，实际 0」）
#       命名检查被钩子拒了不算跑过（「应当返回 2，实际 0」）
#       四格点名少一格不算跑过（「应当返回 2，实际 0」）
#       false && 起的两条没执行，不算跑过（「应当返回 2，实际 0」）
#       true || 起的两条没执行，不算跑过（「应当返回 2，实际 0」）
#       if false; then 两条; fi 不算跑过（「应当返回 2，实际 0」）
#       两条写进函数定义不调用，不算跑过（「应当返回 2，实际 0」）
#       timeout 0.1 包着、被杀退 124，不算跑过（「应当返回 2，实际 0」）
#       exit 0 之后的两条没执行，不算跑过（「应当返回 2，实际 0」）
#       两条用 ; 连着、后一条判红退 1（前一条的退出码看不到），拒（「应当返回 2，实际 0」）
#       两条用 && 连着、退 1（后一条可能被跳过）拒（「应当返回 2，实际 0」）
#       两条接 | head -2、被 SIGPIPE 杀掉（退 141）而整条退 0，不算跑过（「应当返回 2，实际 0」）
#       两条接 | tail -n 20、整条退的是 tail 的 0，不算跑过（「应当返回 2，实际 0」）
#       两条接 || true、退出码被吃掉，不算跑过（「应当返回 2，实际 0」）
#       ; echo rc=$? 打出 rc=2（用法错），不算跑过（「应当返回 2，实际 0」）
#       timeout 230 包着、; echo rc=$? 打出 rc=124（被 timeout 杀掉），不算跑过（「应当返回 2，实际 0」）
#       timeout -s KILL 包着、被杀退 137，不算跑过（「应当返回 2，实际 0」）
#       两条用 ; 连着、整条退 0（前一条的退出码看不到），拒（「应当返回 2，实际 0」）
#       Bash 里 mv 写进 crates/ 的 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 cp 到 crates/ 下的目录、来源是 .rs，没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 > 重定向写 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 >> 追加写 .rs（目标在变量里）、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 tee 写 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       根写成 $(git rev-parse --show-toplevel)、cd 到别的树里跑的不算在改动所在的那一棵上拒（「应当返回 2，实际 0」）
#       两条接 | tail、echo ${PIPESTATUS[1]} 打的是 tail 的，不算跑过（「应当返回 2，实际 0」）
#       rc=$? 存下之后又被改写再 echo，不算跑过（「应当返回 2，实际 0」）
#       存成 persisted-output 而存下来的那一份不在，认不出退出码拒（「应当返回 2，实际 0」）
#       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
#       没跑的拒绝说明写没跑过、不写认不出退出码（「应当返回 True，实际 False」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
#       改了 .rs 没跑的拒绝说明给出两条命令（根填改动所在的树）（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-root-unmatched  naming_check_runs() 认出的根一律当成别的树（跑在哪都不算跑在改动所在的树上）
#       改了 .rs 之后两条命名检查都跑了放行（「应当返回 0，实际 2」）
#       git worktree 副本里改的、命名检查根写成副本根放行（「应当返回 0，实际 2」）
#       经 two-logs.sh 包着跑命名检查放行（「应当返回 0，实际 2」）
#       cd 之后根写成 $(pwd)、&& 连着两条跑命名检查放行（「应当返回 0，实际 2」）
#       timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行（「应当返回 0，实际 2」）
#       两条在同一条里各自 ; echo rc=$?（rc=0、rc=1 按次序配）放行（「应当返回 0，实际 2」）
#       Bash 里 mv 写 .rs 之后两条命名检查都跑了放行（「应当返回 0，实际 2」）
#       根写成 $(git rev-parse --show-toplevel)、cd 进 worktree 副本里跑放行（「应当返回 0，实际 2」）
#       两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行（「应当返回 0，实际 2」）
#       输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行（「应当返回 0，实际 2」）
#       echo rc=$? 追加进落盘的那份文件、同一条里 tail 读出来放行（「应当返回 0，实际 2」）
#       退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行（「应当返回 0，实际 2」）
#       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-any-file  naming_source_tree() 不看后缀，crates/ 或 scripts/ 下改什么文件都要命名检查
#       只改 .md 放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-copy-judged  naming_source_tree() 没有 .git 的副本（rsync 拷的）也判
#       rsync 副本里改的 .rs 交主 agent 放行（「应当返回 0，实际 2」）
#       执行员登记落地清单的文件:行与用例名都找得到放行（「应当返回 0，实际 2」）
#       执行员登记落地清单写在交回正文里也认（「应当返回 0，实际 2」）
#       执行员只补落产物的派发不判登记落地清单（「应当返回 0，实际 2」）
#       执行员没有登记落地清单的拒绝说明写出那一节与出路（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-refused-counts  tool_calls() 把被钩子拒的、没有结果的 Bash 也当成跑过
#       命名检查被钩子拒了不算跑过（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=naming-position-unchecked  completed_naming_text() 不看两条在不在顺序执行的位置上（退 0 或 1 的整条都收）
#       false && 起的两条没执行，不算跑过（「应当返回 2，实际 0」）
#       true || 起的两条没执行，不算跑过（「应当返回 2，实际 0」）
#       if false; then 两条; fi 不算跑过（「应当返回 2，实际 0」）
#       两条写进函数定义不调用，不算跑过（「应当返回 2，实际 0」）
#       exit 0 之后的两条没执行，不算跑过（「应当返回 2，实际 0」）
#       两条用 ; 连着、后一条判红退 1（前一条的退出码看不到），拒（「应当返回 2，实际 0」）
#       两条用 && 连着、退 1（后一条可能被跳过）拒（「应当返回 2，实际 0」）
#       两条接 | head -2、被 SIGPIPE 杀掉（退 141）而整条退 0，不算跑过（「应当返回 2，实际 0」）
#       两条接 | tail -n 20、整条退的是 tail 的 0，不算跑过（「应当返回 2，实际 0」）
#       两条接 || true、退出码被吃掉，不算跑过（「应当返回 2，实际 0」）
#       ; echo rc=$? 打出 rc=2（用法错），不算跑过（「应当返回 2，实际 0」）
#       timeout 230 包着、; echo rc=$? 打出 rc=124（被 timeout 杀掉），不算跑过（「应当返回 2，实际 0」）
#       两条用 ; 连着、整条退 0（前一条的退出码看不到），拒（「应当返回 2，实际 0」）
#       两条接 | tail、echo ${PIPESTATUS[1]} 打的是 tail 的，不算跑过（「应当返回 2，实际 0」）
#       rc=$? 存下之后又被改写再 echo，不算跑过（「应当返回 2，实际 0」）
#       存成 persisted-output 而存下来的那一份不在，认不出退出码拒（「应当返回 2，实际 0」）
#       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-exit-code-unchecked  completed_naming_text() 不看退出码（这一条 Bash 的退出码与 echo 打出的那一行读到的一律当 0）
#       timeout 0.1 包着、被杀退 124，不算跑过（「应当返回 2，实际 0」）
#       两条用 && 连着、退 1（后一条可能被跳过）拒（「应当返回 2，实际 0」）
#       ; echo rc=$? 打出 rc=2（用法错），不算跑过（「应当返回 2，实际 0」）
#       timeout 230 包着、; echo rc=$? 打出 rc=124（被 timeout 杀掉），不算跑过（「应当返回 2，实际 0」）
#       timeout -s KILL 包着、被杀退 137，不算跑过（「应当返回 2，实际 0」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-tool-exit-trusted  sequential_naming_text() 退回拿这一条 Bash 的退出码当每一段的退出码（不读 echo 打出的那一行，管道里、|| 左侧的也收）
#       两条用 ; 连着、后一条判红退 1（前一条的退出码看不到），拒（「应当返回 2，实际 0」）
#       两条接 | head -2、被 SIGPIPE 杀掉（退 141）而整条退 0，不算跑过（「应当返回 2，实际 0」）
#       两条接 | tail -n 20、整条退的是 tail 的 0，不算跑过（「应当返回 2，实际 0」）
#       两条接 || true、退出码被吃掉，不算跑过（「应当返回 2，实际 0」）
#       ; echo rc=$? 打出 rc=2（用法错），不算跑过（「应当返回 2，实际 0」）
#       timeout 230 包着、; echo rc=$? 打出 rc=124（被 timeout 杀掉），不算跑过（「应当返回 2，实际 0」）
#       两条用 ; 连着、整条退 0（前一条的退出码看不到），拒（「应当返回 2，实际 0」）
#       两条接 | tail、echo ${PIPESTATUS[1]} 打的是 tail 的，不算跑过（「应当返回 2，实际 0」）
#       rc=$? 存下之后又被改写再 echo，不算跑过（「应当返回 2，实际 0」）
#       存成 persisted-output 而存下来的那一份不在，认不出退出码拒（「应当返回 2，实际 0」）
#       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-echo-unread  sequential_naming_text() 不读「; echo <标签>$?」打出的那一行（每一段的退出码只从这一条 Bash 的退出码取）
#       timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行（「应当返回 0，实际 2」）
#       两条在同一条里各自 ; echo rc=$?（rc=0、rc=1 按次序配）放行（「应当返回 0，实际 2」）
#       两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行（「应当返回 0，实际 2」）
#       输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行（「应当返回 0，实际 2」）
#       echo rc=$? 追加进落盘的那份文件、同一条里 tail 读出来放行（「应当返回 0，实际 2」）
#       退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行（「应当返回 0，实际 2」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-timeout-refused  sequential_naming_text() 退回第十批的判法：包着 timeout、又不是整条最后一条的一律不收（判决 landing-audit-r5 K2-2a 打中的误拒）
#       timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行（「应当返回 0，实际 2」）
#       退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行（「应当返回 0，实际 2」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-substitution-target-unexpanded  bash_rs_writes() 目标以命令替换起头的照旧不算（改前：$(git rev-parse --show-toplevel)/crates/…rs 认不出落点）
#       Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 > 重定向写到 "$(pwd)/crates/…rs"、没跑命名检查拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=naming-bash-writes-ignored  bash_rs_writes() 一个都不认（改前：Bash 里 mv、cp、重定向、tee 写的 .rs 不算改动）
#       Bash 里 mv 写进 crates/ 的 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 cp 到 crates/ 下的目录、来源是 .rs，没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 > 重定向写 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 >> 追加写 .rs（目标在变量里）、没跑命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 tee 写 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
#       「没判全:⑤ 切不开写 .rs 的 Bash 命令记一处（rs-split） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=naming-git-toplevel-unknown  naming_check_occurrences() 不认根写成 $(git rev-parse --show-toplevel) 的（改前：根认不出，不算跑过）
#       根写成 $(git rev-parse --show-toplevel)、cd 进 worktree 副本里跑放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-pipestatus-unread  status_echo_pattern() 不认 ${PIPESTATUS[k]}（当任意字）
#       两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-saved-status-unread  status_echo_readings() 不认「名字=$?」存下、之后 echo $名字 打出的数
#       输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-persisted-unread  tool_calls() 不读 persisted-output 存下来的那一份（只按预览认，改前的读法）
#       退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-refusal-reasons-merged  naming_problems() 不记跑了却没认出的原因（拒绝说明退回一律写没跑过）
#       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
#       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
#       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=leg-report-relative-root-only  leg_reports_named() 只写相对路径的报告一律按项目根（改前：干活的那棵 worktree 不看，找不到也不拒）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
#   HANDBACK_GUARD_BREAK=leg-report-work-root-token-kept  work_root_tokens_expanded() 不展开 $SINGLEFS_WORK_ROOT/（改前：② 认不出这种写法）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行（「应当返回 (0, False)，实际 (2, True)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (2, True)」）
#   HANDBACK_GUARD_BREAK=no-landing-checklist  decide() 不判 ⑦ 执行员的登记落地清单
#       执行员报告里没有登记落地清单一节拒（「应当返回 2，实际 0」）
#       执行员登记落地清单的文件:行超出那份文件的行数拒（「应当返回 2，实际 0」）
#       执行员登记落地清单的文件在副本与主工作区里都找不到拒（「应当返回 2，实际 0」）
#       执行员登记落地清单的用例名 grep 不到拒（「应当返回 2，实际 0」）
#       执行员登记落地清单一节里一行都没有拒（「应当返回 2，实际 0」）
#       执行员落了产物、没有登记落地清单一节拒（「应当返回 2，实际 0」）
#       执行员没有登记落地清单的拒绝说明写出那一节与出路（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=landing-exemption-ignored  landing_problems() 只修锚点、只补落产物的派发也判
#       执行员只补落产物的派发不判登记落地清单（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=landing-exemption-substring  landing_exempted() 退回改前：整份提示里有「只修锚点」「只补落产物」几个字就算豁免，不看行首标签与整句
#       执行员提示句中写「不是只补落产物」不算豁免、照判登记落地清单（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=landing-applies-always  landing_problems() 没写装置、没落产物的执行员也判
#       执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=landing-location-unchecked  landing_problems() 不核清单里的文件:行
#       执行员登记落地清单的文件:行超出那份文件的行数拒（「应当返回 2，实际 0」）
#       执行员登记落地清单的文件在副本与主工作区里都找不到拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=landing-case-unchecked  landing_problems() 不 grep 清单里的用例名
#       执行员登记落地清单的用例名 grep 不到拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=honor-break-outside-selftest  main() 平常跑设了弄坏开关也照认、照常判（改前的样子）
#       平常跑设了 HANDBACK_GUARD_BREAK 退 2、点名它只在自证里认（「应当返回 (2, True)，实际 (0, False)」）
#   HANDBACK_GUARD_BREAK=leg-report-subdirectory-kept  leg_reports_named() 退回改前这里自己的一份认法（子目录、<轮> 占位都认），不调 lib_three_way_legs.py
#       三方腿报告路径:只点名子目录下的 -output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:只点名没填的 <轮> 占位不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:只点名仓外同名的 …/research/prompts/r1-opus-output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:登记过的 worktree 里引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (0, False)」）
#       三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明（「应当返回 True，实际 False」）
#       三方腿报告路径:交回闸调的是 lib_three_way_legs.py 的 leg_report_mentions_in_trees（「应当返回 ['sentinel-from-library']，实际 ['research/prompts/r1-opus-output.md']」）
#   HANDBACK_GUARD_BREAK=leg-report-outside-root-kept  leg_reports_named() 不看点名的报告解不解到项目根下的 research/prompts/（直接用 lib_three_way_legs.py 的 LEG_REPORT_MENTION，仓外同名的也认成仓里那一份）
#       三方腿报告路径:只点名仓外同名的 …/research/prompts/r1-opus-output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:登记过的 worktree 里引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒（「应当返回 True，实际 False」）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (0, False)」）
#       三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明（「应当返回 True，实际 False」）
#       三方腿报告路径:交回闸调的是 lib_three_way_legs.py 的 leg_report_mentions_in_trees（「应当返回 ['sentinel-from-library']，实际 ['research/prompts/r1-opus-output.md']」）
#   HANDBACK_GUARD_BREAK=leg-report-worktrees-unread  leg_reports_named() 不读 git worktree list，只认项目根下的（第十一批任务 35 那一版，把登记过的 worktree 里的报告误拒）
#       三方腿报告路径:登记过的 worktree 里引文对上放行（「应当返回 0，实际 2」）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对上放行（「应当返回 0，实际 2」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行（「应当返回 (0, False)，实际 (2, True)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (2, True)」）
#   HANDBACK_GUARD_BREAK=leg-report-git-directory-trusted  leg_reports_named() 另认前面那一截是个带 .git 的目录的（不看它登没登记成本仓的 worktree）
#       三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=leg-report-citation-root-main  citation_problems() 核 worktree 里的报告时 --root 仍给项目根（不按报告所在那一棵）
#       三方腿报告路径:登记过的 worktree 里引文对上放行（「应当返回 0，实际 2」）
#       三方腿报告路径:登记过的 worktree 里引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对上放行（「应当返回 0，实际 2」）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行（「应当返回 (0, False)，实际 (2, False)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (0, False)」）
#   HANDBACK_GUARD_BREAK=leg-report-worktree-error-unnamed  citation_problems() 没点名报告的拒绝说明里不写取不到 worktree 列表
#       三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=no-scribe-targets  decide() 不判 ⑥ 写回员规格的目标不在
#       写回员规格目标不在、报告没写找不到的目标拒（「应当返回 2，实际 0」）
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#       写回员规格写在提示正文里、目标不在拒（「应当返回 2，实际 0」）
#       写回员自己起草的 spec.json 目标不在拒（「应当返回 2，实际 0」）
#       写回员目标不在的拒绝说明列出文件与出路（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=scribe-section-anywhere  missing_targets_listed() 报告里哪一节提到都算写进了「找不到的目标」
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=scribe-section-unread  missing_targets_listed() 不读「找不到的目标」一节，目标不在一律拒
#       写回员规格目标不在、报告「找不到的目标」一节写了它放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=leg-types-hardcoded  citation_checked_agents() 不读定义 frontmatter 的 leg: three-way，退回改前写死的四种（三种腿加没有定义的 three-way-verifier）
#       三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同（「应当返回 ['sample-leg', 'three-way-defense', 'three-way-forward']，实际 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier']」）
#       三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种（「应当返回 (['three-way-attack', 'three-way-defense', 'three-way-forward'], True)，实际 (['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier'], True)」）
#       三方腿类型表:样本仓登记的 sample-leg 没点名报告拒（「应当返回 2，实际 0」）
#       三方腿类型表:样本仓没登记的攻方不核引文（「应当返回 0，实际 2」）
#       「没判全:读不到 lib_three_way_legs.py记一处（leg-types） 应当返回 True，实际 False」
#       「没判全:② 有一份定义读不了记一处（leg-definitions） 应当返回 True，实际 False」
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（同目录 lib_work_root.py：工作根退回旧根）  ③ 认不出工作根下的报告文件，⑥ 认不出派发提示点名的工作根下的规格与报告
#       执行员工作根下的报告里绿行没数（~ 写法）（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（$SINGLEFS_WORK_ROOT 写法）（「应当返回 2，实际 0」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行（「应当返回 (0, False)，实际 (2, True)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (2, True)」）
#       写回员规格目标不在、报告没写找不到的目标拒（「应当返回 2，实际 0」）
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#       写回员自己起草的 spec.json 目标不在拒（「应当返回 2，实际 0」）
#       写回员目标不在的拒绝说明列出文件与出路（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=leg-types-unrecorded  citation_checked_agents() 读不到 lib_three_way_legs.py 时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:读不到 lib_three_way_legs.py记一处（leg-types） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=leg-definitions-unrecorded  citation_checked_agents() 有定义读不了（三方腿类型表读不全）时只打 stderr、不记进没判全的几处（改前的样子：静默跳过那一份）
#       「没判全:② 有一份定义读不了记一处（leg-definitions） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=citation-transcript-unrecorded  citation_since() 找不到这个子 agent 的会话记录时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:② 找不到会话记录、认不出开工时刻记一处（citation-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=cite-check-unrecorded  run_cite_check() cite-check 跑不起来、超时、退 0 与 1 以外的码时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:② cite-check 退别的码记一处（cite-check） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=green-report-unrecorded  green_count_problems() 点名的报告读不了时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:③ 点名的报告读不了记一处（green-report） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=background-transcript-unrecorded  background_problems() 找不到这个子 agent 自己的会话记录时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:④ 找不到自己的会话记录记一处（background-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=background-unrecorded  background_problems() 读不到 agent-watch.py 或它抛异常时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:④ 读不到 agent-watch.py记一处（background） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=rs-split-unrecorded  bash_rs_writes() 切不开一条 Bash 命令时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑤ 切不开写 .rs 的 Bash 命令记一处（rs-split） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=naming-transcript-unrecorded  naming_problems() 找不到这个子 agent 自己的会话记录时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑤ 找不到自己的会话记录记一处（naming-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=heavy-library-unrecorded  heavy_tests_library() 导入不了 lib_heavy_tests.py 时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑤ 导入不了 lib_heavy_tests.py记一处（heavy-library） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=scratch-line-unrecorded  prompt_scratch_directories() 读不到 lib_prompt_scratch_line.py 或它抛异常时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑥ 读不到 lib_prompt_scratch_line.py记一处（scratch-line） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=scribe-transcript-unrecorded  scribe_target_problems() 找不到写回员自己的会话记录时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑥ 找不到写回员自己的会话记录记一处（scribe-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=kb-spec-unrecorded  scribe_target_problems() 导入不了 kb-spec-check.py 时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑥ 导入不了 kb-spec-check.py记一处（kb-spec） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=spec-unreadable-unrecorded  scribe_spec_entries() 规格读不了时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑥ 草稿目录里的规格读不了记一处（spec-unreadable） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=landing-transcript-unrecorded  landing_problems() 找不到执行员自己的会话记录时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑦ 找不到执行员自己的会话记录记一处（landing-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=landing-split-unrecorded  landing_applies() 切不开一条 Bash 命令时只打 stderr、不记进没判全的几处（改前的样子）
#       「没判全:⑦ 切不开那一条 Bash 命令记一处（landing-split） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=input-unrecorded  main() 读不懂 hook 的 JSON（坏 JSON、顶层不是对象）时只打 stderr、不记检出（改前的样子）
#       「走真实 stdin 入口：坏 JSON 放行并记一条检出（结局只记、会话号读不到） 应当返回 (0, 1, '只记', '读不到')，实际 …」
#   HANDBACK_GUARD_BREAK=exception-unrecorded  judge() 判的时候抛异常不接住、照原样抛出（改前的样子：进程退 1，放行、不记检出）
#       「入口:判的时候抛异常放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   HANDBACK_GUARD_BREAK=work-root-unrecorded  judge() 读不到 lib_work_root.py 时只打 stderr、不记检出（改前的样子）
#       「入口:读不到 lib_work_root.py 放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere（同目录 lib_prompt_scratch_line.py：「草稿目录」在一行里哪都认）  ⑥ 把列表里转述的「- 草稿目录：」也当本提示的草稿目录
#       写回员提示里列表起头的「- 草稿目录：」是转述，不在那里找起草的规格（「应当返回 0，实际 2」）
#   SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown（同上：不认过渡期旧根）  ③ 认不出旧根下的报告文件
#       执行员过渡期旧根下的报告里绿行没数（「应当返回 2，实际 0」）
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$HOOK_DIR/../singlefs-ai-sop/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shlex, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone

HANDBACK_CHARACTER_LIMIT = int(os.environ.get("HANDBACK_CHARACTER_LIMIT", "4000"))
CITATION_CHECKED_AGENTS_BEFORE_REGISTRY = {"three-way-forward", "three-way-attack", "three-way-defense", "three-way-verifier"}   # 改前写死的那一份：只有弄坏开关 leg-types-hardcoded 下还在用
GREEN_COUNT_CHECKED_AGENTS = {"kb-scribe", "experiment-runner"}
# ② 三方腿报告路径的认法只在同目录 lib_three_way_legs.py 的 leg_report_mentions_in_trees（与派发闸 ⑰ 共用，判决 landing-audit-r4 K2-5：占位与子目录下的不认）；
# 改前这里自己的一份连子目录、占位都认，只有弄坏开关 leg-report-subdirectory-kept 下还在用
LEG_REPORT_MENTION_BEFORE_LIBRARY = re.compile(r"research/prompts/[^\s`'\"，。；：、（）()「」]+-output\.md")
try:   # 工作根与过渡期旧根的唯一定义：同目录的 lib_work_root.py（读不到时那两处下的报告认不出、不判，stderr 写一句）
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    SCRATCH_MENTION_PREFIX, work_roots_error = work_roots.mention_prefix_pattern(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, SCRATCH_MENTION_PREFIX, work_roots_error = None, "(?!)", error
try:   # 三方腿类型表的读法（登记处是定义 frontmatter 的 leg: three-way 一行；与派发闸、骨架、账表共用）：同目录的 lib_three_way_legs.py（读不到时 ② 不判，stderr 写一句）
    sys.path.insert(0, sys.argv[1]); import lib_three_way_legs as three_way_legs
    three_way_legs_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    three_way_legs, three_way_legs_error = None, error
try:   # 「本提示的草稿目录」那一行的认法（与派发闸 ⑯ ⑰、完工清理闸共用一份，只认行首「草稿目录：」）：同目录的 lib_prompt_scratch_line.py（读不到时 ⑥ 不看草稿目录里起草的规格，stderr 写一句）
    sys.path.insert(0, sys.argv[1]); import lib_prompt_scratch_line as prompt_scratch_line
    prompt_scratch_line_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    prompt_scratch_line, prompt_scratch_line_error = None, error
REPORT_MENTION = re.compile(r"(?:research/prompts/|" + SCRATCH_MENTION_PREFIX + r")[^\s`'\"，。；：、（）()「」]+\.md")
GREEN_LINE = re.compile(r"^\s*(?:[-*>|]\s*)*✓")
# 编号不是数：阶段号、实验 / 决策 / 欠账 / 不变量号、阶段文件名、时刻
NOT_A_COUNT = re.compile(r"\d+\s*号|\b\d{2}-[\w.-]+|\b[EDC]\d+|\bI-[\d.]+|\d{1,2}:\d{2}")
ZERO_COUNT = re.compile(r"(?:查了|检查了?|扫了|判了|核了|共|跑了)\s*0\s*(?:项|个|条|份|处|种|行)")
BROKEN = os.environ.get("HANDBACK_GUARD_BREAK", "")
# ⑤ 交回前要跑命名检查的类型、要跑的四格与两条命令（共用约束 .claude/agent-common.md「门禁」一节「命名检查交回前跑」，<根> 填改动所在的树）
NAMING_CHECKED_AGENTS = {"implementation-writer", "experiment-runner", "tooling-writer"}
NAMING_CHECK_CELLS = frozenset({"vague-names", "test-file-names", "ordinal-names", "data-names"})
NAMING_SOURCE_DIRECTORIES = ("crates", "scripts")
NAMING_STAGE_TAIL = "/.claude/gate.d/code-source-discipline.sh"
NAMING_LINT_TAIL = "/.claude/singlefs-ai-sop/scripts/naming-lint.sh"
NAMING_STAGE_COMMAND = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; "
                        "范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh "
                        "--check vague-names,test-file-names,ordinal-names,data-names {root}")
NAMING_LINT_COMMAND = "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh {root}"
# ⑤ 只认真跑完的那两条（判决 landing-audit-r4 K3-1，写法在 sequential_naming_text）：跑完的退出码 0 通过、1 判红
# （.claude/singlefs-ai-sop/scripts/naming-lint.sh 文件头「退出码」，门禁判红退 1；3、77 是没跑，124、137 这类是被截断）
NAMING_COMPLETED_EXIT_CODES = (0, 1)
NAMING_TRUNCATING_PREFIXES = frozenset({"timeout"})   # 只有弄坏开关 naming-timeout-refused 下还看它（第十批的判法）
# ⑤ 「; echo <标签>$?」打出的那一行：$? 那一处认成退出码，别的展开（$D、${x}、$(…)、`…`）当任意字（判决 landing-audit-r5 K2-2a）；
# ${PIPESTATUS[k]} 认成紧挨着的那条管道第 k 段的退出码，紧跟一段的「名字=$?」（或 =${PIPESTATUS[k]}）存下的、之后 echo 打出的 $名字 认成那一段的（拆分提案第九十节 ㉞ ㉟）
NAMING_STATUS_FORMS = (r"\$\?", r"\$\{\?\}")
NAMING_PIPE_STATUS_FORM = r"\$\{PIPESTATUS\[(?P<pipe>\d+)\]\}"
NAMING_SAVED_STATUS = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(?:\$\?|\$\{\?\}|\$\{PIPESTATUS\[(\d+)\]\})$")
NAMING_OTHER_EXPANSION = re.compile(r"\$\{[^}]*\}|\$\([^)]*\)|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9#@*!$-]|`[^`]*`")
NAMING_ECHO_COMMANDS = frozenset({"echo", "printf"})
# ⑤ 命名检查的根写成 $(git rev-parse --show-toplevel)（或反引号写法）：按命令当时的目录往上找 .git 的那一层认（拆分提案第九十节 ㉕）
NAMING_GIT_TOPLEVEL_ROOT = re.compile(r"^(?:\$\(\s*git\s+rev-parse\s+--show-toplevel\s*\)|`\s*git\s+rev-parse\s+--show-toplevel\s*`)$")
# ⑤ Bash 里写 .rs 也算改动（拆分提案第九十节 ㉕）：输出重定向（>、>>、>|、&>）的目标、tee 的文件参数、cp / mv 的目标
NAMING_COPYING_COMMANDS = frozenset({"cp", "mv"})
# ⑤ 写 .rs 的目标以命令替换起头、接着 /crates/ 或 /scripts/ 的（拆分提案第一百二十二节 ⑪：$(git rev-parse --show-toplevel)/crates/…rs 原来认不出落点、不计）
NAMING_SUBSTITUTION_TARGET = re.compile(r"^[\"']?(?P<substitution>\$\([^()]*\)|`[^`]*`)[\"']?(?P<rest>/(?:crates|scripts)/\S*\.rs)[\"']?$")
NAMING_COPY_OPTIONS_WITH_VALUE = frozenset({"-S", "--suffix"})
# ⑤ 输出太长、被存成 persisted-output 的工具结果：照里面写的路径读那份存下来的整份输出（拆分提案第九十节 ㉞）
PERSISTED_OUTPUT = re.compile(r"<persisted-output>\s*Output too large \([^)\n]*\)\. Full output saved to: (\S+)")
PERSISTED_OUTPUT_READ_LIMIT = 64 * 1024 * 1024
# ⑤ 交回闸认得出退出码的那一种落盘写法（共用约束「不做」一节读文件那一条、「门禁」一节「命名检查交回前跑」照抄同一种），拒绝说明里照抄
NAMING_FILED_OUTPUT_FORM = "<命令> > <草稿目录>/<名>.txt 2>&1; echo rc=$?; tail -n 20 <草稿目录>/<名>.txt"
# ⑦ 执行员交回的报告里要有的「登记落地清单」一节（.claude/agents/experiment-runner.md 第 3c 步、「产出」一节；拆分提案第九十节 ①）
LANDING_CHECKED_AGENTS = {"experiment-runner"}
LANDING_SECTION = "登记落地清单"
LANDING_EXEMPT_MARKS = ("只修锚点", "只补落产物")   # 派发闸 ⑪ 放行的两种：不跑产物，没有第 3c 步那张清单
# ⑦ 豁免只认行首标签与整句（拆分提案第一百二十二节 ⑪）：一行（去掉行首的 - * > _ 与粗体）以那两种之一起头，后面是行尾或冒号、逗号、句号、分号、括号；
# 句中的「这一段不是只补落产物」「只补落产物不算」都不认
LANDING_EXEMPT_LINE = re.compile(r"^[ \t>*_\-]*(?:只修锚点|只补落产物)[* \t]*(?:$|[：:，,。；;（(])", re.M)
LANDING_LOCATION = re.compile(r"(?<![\w./~$<>{}-])((?:[\w.~-]+/)*[\w.-]+\.(?:rs|py|sh|toml|tsv|json|md|txt))[:：](\d+)(?!\d)")
LANDING_CASE_NAME = re.compile(r"(?<![\w/.$-])([a-z][a-z0-9]*(?:_[a-z0-9]+){2,})(?![\w…]|\.\.\.|\.[A-Za-z])")
LANDING_INDEX_SKIPPED = frozenset({"target", ".git", "__pycache__", "node_modules"})
LANDING_SOURCE_SUFFIXES = (".rs", ".py", ".sh")
LANDING_INDEX_LIMIT = 200000
LANDING_TREE_DIRECTORIES = ("crates", "scripts")
# ⑦ 只判写了装置或落了产物的执行员（第 3c 步写在「跑产物之前」）：装置源文件（入库装置 src/bin/ 下、实验装置目录下，或 e<三位号>_ 起头的 .rs / .py / .sh；
# replay.sh 是复跑入口的登记，不算），产物落在 research/results/ 下
LANDING_APPARATUS_SOURCE = re.compile(r"(?:^|/)(?:crates/singlefs-checker-tier/src/bin|scripts/experiment-apparatus|scripts/experiments)/(?:[^/\s]+/)*[^/\s]+\.(?:rs|py|sh)$"
                                      r"|(?:^|/)e\d{3}_[A-Za-z0-9_]*\.(?:rs|py|sh)$")
LANDING_NOT_APPARATUS = frozenset({"replay.sh"})
LANDING_PRODUCT_DIRECTORY = "research/results/"
NAMING_TERMINATING_COMMANDS = frozenset({"exit", "return", "logout"})
NAMING_CHAIN_COMMANDS = frozenset({"cd", "pushd", "popd", "export"})
NAMING_ALLOWED_PREDECESSORS = frozenset({"trivial", "cd", "naming"})
NAMING_COMPOUND_CLOSERS = {"if": "fi", "case": "esac", "while": "done", "until": "done", "for": "done", "select": "done"}
NAMING_ARRAY_ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*\+?=$")
FILE_EDITING_TOOLS = {"Edit", "Write", "MultiEdit"}
# ⑥ 写回员：规格目标不在时报告里要有的那一节（.claude/agents/kb-scribe.md），写回员自己起草的规格放草稿目录的这两个名字
SCRIBE_TARGET_CHECKED_AGENTS = {"kb-scribe"}
MISSING_TARGETS_SECTION = "找不到的目标"
SCRIBE_DRAFTED_SPECIFICATIONS = ("spec.json", "spec.md")
MARKDOWN_HEADING = re.compile(r"^ {0,3}(#{1,6})\s+(.*)$")
PROMPT_LABEL_LINE_START = r"^[ \t>*_\-]*"
SPECIFICATION_SUFFIXES = (".json", ".md", ".txt")
# ① 拒绝说明改前的写法（没写报告文件名，辩方 landing-audit-r2 正推 8）：只有弄坏开关 limit-hint-unnamed 下还在用
LIMIT_HINT_BEFORE = "→ 怎么办：全文写进报告文件（派发提示给的报告路径；没给就写进草稿目录），交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。"
# 弄坏开关只在自证里认（.claude/gate.d/break-switch-outside-selftest-pending.tsv 文件头的改法）：--selftest 与它起的子进程（带这份标记）认，平常跑设了就拒
SELFTEST_MARK_VARIABLE = "HANDBACK_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "handback-guard.sh --selftest 起的子进程"
# 判不了而放行的几处（文件头「这一道自己出错」一段）：各处 note_unjudged 交进来，放行时入口照「放行但必记检出」记一条检出
UNJUDGED = []
UNJUDGED_PREFIX = "交回正文闸没判全、照常放行"
UNJUDGED_SITES = ("input", "exception", "work-root", "leg-types", "leg-definitions", "citation-transcript", "cite-check", "green-report", "background-transcript", "background",
                  "rs-split", "naming-transcript", "heavy-library", "scratch-line", "scribe-transcript", "kb-spec", "spec-unreadable", "landing-transcript", "landing-split")


def note_unjudged(site, message):
    """判不了而放行的一处：stderr 照原样写 message，记进 UNJUDGED 等入口记检出。弄坏开关 HANDBACK_GUARD_BREAK=<site>-unrecorded 下只打 stderr（改前的样子）。"""
    print(message, file=sys.stderr)
    if BROKEN != f"{site}-unrecorded":
        UNJUDGED.append((site, message.strip().removeprefix("! handback-guard：").removeprefix("! handback-guard ")))


def record_unjudged(hook_input):
    """放行而 UNJUDGED 里有几处：检出记录追加一行（结局「只记」，findings 以 UNJUDGED_PREFIX 起头）。检出记录取 AGENT_HOOK_DETECTIONS，
    没设取同目录 lib_work_root.py 的 default_detections_path。交一句给人看的说明：记进了哪里，或为什么没记成。"""
    path = os.environ.get("AGENT_HOOK_DETECTIONS")
    if not path and work_roots is not None:
        try:
            path = work_roots.default_detections_path()
        except Exception:   # 工作根的弄坏开关写错这类：不知道往哪写，下面照打一句
            path = None
    if not path:
        return "! 检出没记成：取不到检出记录的位置（AGENT_HOOK_DETECTIONS 没设，同目录 lib_work_root.py 也读不到）"
    known = hook_input if isinstance(hook_input, dict) else {}
    session = known.get("session_id")
    entry = {"time": datetime.now(timezone.utc).isoformat(), "hook": "handback-guard.sh",
             "session_id": session if isinstance(session, str) and session else "读不到", "agent_id": known.get("agent_id"),
             "agent_type": known.get("agent_type") or "主 agent", "transcript_path": known.get("transcript_path"),
             "command": f"SubagentHandback（{known.get('agent_type') or '?'}）" if known else "SubagentHandback（hook 输入读不懂）",
             "findings": [f"{UNJUDGED_PREFIX}：{text}" for _, text in UNJUDGED], "outcome": "只记"}
    try:
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        return f"! 检出没记成：写不了 {path}（{error}）"
    return f"  照「放行但必记检出」，检出记进 {path}"


def is_project_agent(agent_type, project_root):
    if BROKEN == "judge-general-purpose" and agent_type:
        return True
    return bool(agent_type) and os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md"))


def citation_checked_agents(project_root):
    """② 要核引文的三方腿类型：项目根下定义 frontmatter 写了 leg: three-way 的（lib_three_way_legs.py 的 three_way_leg_types）。
    读不到那份库交空集、stderr 写一句；弄坏开关 leg-types-hardcoded 下退回改前写死的四种。"""
    if BROKEN == "leg-types-hardcoded":
        return frozenset(CITATION_CHECKED_AGENTS_BEFORE_REGISTRY)
    if three_way_legs is None:
        note_unjudged("leg-types", f"! handback-guard 读不到共用模块 lib_three_way_legs.py（{three_way_legs_error!r}），三方腿引文这一条没判（放行）")
        return frozenset()
    unreadable = three_way_legs.unreadable_definitions(project_root)
    if unreadable:
        note_unjudged("leg-definitions", f"! handback-guard：定义 {'、'.join(unreadable)} 读不了，三方腿类型表没读全，那几种的引文这一条没判（放行）")
    return three_way_legs.three_way_leg_types(project_root)


def started_at(hook_input):
    """这个子 agent 会话记录第一条的时刻（epoch 秒）；找不到返回 None。"""
    transcript_path = hook_input.get("transcript_path") or ""
    agent_id = hook_input.get("agent_id") or ""
    if not transcript_path.endswith(".jsonl") or not agent_id:
        return None
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl")
    try:
        with open(own, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                stamp = json.loads(line).get("timestamp")
                if stamp:
                    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp()
    except (OSError, ValueError):
        return None
    return None


def resolved(path, project_root):
    path = work_roots.expand_written_path(path) if work_roots else path   # ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 写法
    return path if os.path.isabs(path) else os.path.join(project_root, path)


def work_root_tokens_expanded(text):
    """文字里的 $SINGLEFS_WORK_ROOT/、${SINGLEFS_WORK_ROOT}/ 换成展开后的工作根（同目录 lib_work_root.py 的 expand_written_path 那一种解法）；读不到那份库原样交回。
    弄坏开关 leg-report-work-root-token-kept 下不换（改前：② 认不出这种写法）。"""
    if work_roots is None or BROKEN == "leg-report-work-root-token-kept":
        return text
    variable = re.escape(work_roots.WORK_ROOT_VARIABLE)
    try:
        root = work_roots.work_root()
    except ValueError:   # 工作根的弄坏开关写错：照原样
        return text
    return re.sub(r"\$" + variable + r"(?=/)|\$\{" + variable + r"\}(?=/)", lambda _: root, text)


def agent_working_trees(hook_input, project_root):
    """这个子 agent 在哪几棵本仓登记过的别的工作树里干过活（项目根不算）：它会话记录里工具调用的参数与 cwd 点名到那一棵（那一棵的根或它底下的路径），
    最后一次点名的排前面；读不到会话记录、读不到 lib_three_way_legs.py 交空表。"""
    records = own_transcript_records(hook_input or {}) if hook_input else None
    if not records or three_way_legs is None:
        return []
    trees = three_way_legs.registered_trees(project_root)[0][1:]
    last = {}
    for number, record in enumerate(records):
        texts = [str(record.get("cwd") or "")]
        message = record.get("message")
        for block in (message.get("content") if isinstance(message, dict) and isinstance(message.get("content"), list) else []):
            if isinstance(block, dict) and block.get("type") == "tool_use" and isinstance(block.get("input"), dict):
                texts += [str(value) for value in block["input"].values() if isinstance(value, str)]
        for tree in trees:
            if any(re.search(re.escape(tree.rstrip("/")) + r"(?![^/\s'\"`;&|)])", text) for text in texts):
                last[tree] = number
    return sorted(last, key=lambda tree: -last[tree])


def leg_reports_named(message, project_root, hook_input=None):
    """② 交回里点名的、解到本仓某一棵工作树 research/prompts/ 的三方腿报告（lib_three_way_legs.py 的 leg_report_mentions_in_trees）
    → [(那一棵, research/prompts/…-output.md)]，去重排序；读不到那份库交空表。$SINGLEFS_WORK_ROOT/ 起头的照 lib_work_root.py 展开再认（work_root_tokens_expanded）。
    只写相对路径（前面没有树根，或 ./）的：这个子 agent 在别的登记过的工作树里干过活、那一棵里有这份的，按那一棵（agent_working_trees，最后干活的那一棵先）；
    都没有而项目根下有的按项目根；哪一棵都没有的，那一棵记 None（citation_problems 按找不到拒）。
    弄坏开关 leg-report-subdirectory-kept 下退回改前这里自己的一份，leg-report-outside-root-kept 下仓外同名的也认成项目根那一份，
    leg-report-worktrees-unread 下只认项目根，leg-report-git-directory-trusted 下另认带 .git 的目录，leg-report-relative-root-only 下相对路径一律按项目根（找不到也不拒）。"""
    root = os.path.normpath(project_root)
    if BROKEN == "leg-report-subdirectory-kept":
        return sorted({(root, path) for path in LEG_REPORT_MENTION_BEFORE_LIBRARY.findall(message)})
    if three_way_legs is None:
        return []
    text = work_root_tokens_expanded(message)
    if BROKEN == "leg-report-outside-root-kept":
        return sorted({(root, match.group(0)) for match in three_way_legs.LEG_REPORT_MENTION.finditer(text)})
    trees = [root] if BROKEN == "leg-report-worktrees-unread" else None
    working = None
    named = set()
    for match, tree in three_way_legs.leg_report_mentions_in_trees(text, project_root, trees):
        path = match.group(0)
        window = text[max(0, match.start() - three_way_legs.LEG_REPORT_PREFIX_WINDOW):match.start()]
        if BROKEN != "leg-report-relative-root-only" and three_way_legs.LEG_REPORT_PREFIX.search(window).group(0) in ("", "./"):
            if working is None:
                working = agent_working_trees(hook_input, project_root) if BROKEN != "leg-report-worktrees-unread" else []
            found = [candidate for candidate in working + [root] if os.path.isfile(os.path.join(candidate, path))]
            tree = found[0] if found else None
        named.add((tree, path))
    if BROKEN == "leg-report-git-directory-trusted":
        for match in three_way_legs.LEG_REPORT_MENTION.finditer(text):
            prefix = three_way_legs.LEG_REPORT_PREFIX.search(text[:match.start()]).group(0).rstrip("/")
            if os.path.isabs(prefix) and os.path.exists(os.path.join(prefix, ".git")):
                named.add((os.path.normpath(prefix), match.group(0)))
    return sorted(named, key=lambda pair: (pair[0] or "", pair[1]))


def worktree_list_note(project_root):
    """取不到本仓 git worktree 列表时拒绝说明里加的那一句（取到了是空串）。"""
    if three_way_legs is None or BROKEN == "leg-report-worktree-error-unnamed":
        return ""
    error = three_way_legs.registered_trees(project_root)[1]
    return f"；这一次取不到本仓的 git worktree 列表（{error}），只认了项目根下的，登记过的 worktree 里的报告也没认出来" if error else ""


def citation_problems(message, hook_input, project_root, cite_check):
    reports = leg_reports_named(message, project_root, hook_input)
    if not reports:
        return ["交回里没点名报告文件（本仓某一棵工作树下的 research/prompts/<轮>-…-output.md，工作树是项目根与 git worktree list 列出的各棵；"
                "不在任何一棵里的仓外副本不算：报告写进项目根或登记过的 worktree 的 research/prompts/，交回点名那一份" + worktree_list_note(project_root) + "）"]
    missing = [path for tree, path in reports if tree is None]
    if missing:
        searched = "、".join(agent_working_trees(hook_input, project_root) + [os.path.normpath(project_root)])
        return [f"交回里只写相对路径的报告 {path} 在你干过活的工作树与项目根（{searched}）下都找不到：报告还没写进去，或写在了别处——"
                "写进去再交回，或交回里写那一份的绝对路径" for path in missing]
    since = citation_since(hook_input)
    if since is None:
        return []
    problems, root = [], os.path.normpath(project_root)
    for tree in dict.fromkeys(tree for tree, _ in reports):   # 按报告所在那一棵当根核引文
        citation_root = root if BROKEN == "leg-report-citation-root-main" else tree
        command = [sys.executable, cite_check, *[os.path.join(tree, path) for named_tree, path in reports if named_tree == tree],
                   "--root", citation_root, "--unchanged-since", str(since)]
        result = run_cite_check(command, tree)
        if result is not None:
            label = "" if tree == root else f"（{tree}）"
            problems += [label + line.strip() for line in result.stdout.splitlines() if line.strip().startswith("✗")]
    return problems


def citation_since(hook_input):
    """② 核引文的起点（这个子 agent 开工的时刻）；找不到它的会话记录交 None，记一处没判全。"""
    since = started_at(hook_input)
    if since is None:
        note_unjudged("citation-transcript", "! handback-guard：找不到这个子 agent 的会话记录，认不出开工时刻，引文这一条没判（放行）")
    return since


def run_cite_check(command, tree):
    """起一次 cite-check：退 1（有对不上的引文）交那一趟的结果；退 0 交 None；跑不起来、超时、退别的码交 None 并记一处没判全。"""
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=110)
    except (OSError, subprocess.TimeoutExpired) as error:
        note_unjudged("cite-check", f"! handback-guard：cite-check 跑不起来，{tree} 下的引文这一条没判（放行）：{error}")
        return None
    if result.returncode == 1:
        return result
    if result.returncode != 0:
        note_unjudged("cite-check", f"! handback-guard：cite-check 退出码 {result.returncode}，{tree} 下的引文这一条没判（放行）：{result.stdout[-300:]}{result.stderr[-300:]}")
    return None


def green_count_problems(message, project_root):
    texts = [("交回正文", message)]
    for path in sorted(set(REPORT_MENTION.findall(message))):
        absolute = resolved(path, project_root)
        if os.path.isfile(absolute):
            try:
                texts.append((path, open(absolute, encoding="utf-8", errors="replace").read()))
            except OSError as error:
                note_unjudged("green-report", f"! handback-guard：读不了 {absolute}（{error}），这一份的绿行没判")
    problems = []
    for name, text in texts:
        for number, line in enumerate(text.splitlines(), 1):
            if not GREEN_LINE.match(line):
                continue
            if BROKEN != "no-green-count" and not re.search(r"\d", NOT_A_COUNT.sub("", line)):
                problems.append(f"{name}:{number} 绿行没有数：{line.strip()[:120]}")
            elif BROKEN != "zero-count-ok" and ZERO_COUNT.search(line):
                problems.append(f"{name}:{number} 绿行报的是 0 项：{line.strip()[:120]}")
    return problems


def background_problems(hook_input, agent_watch_path):
    """④ 这个子 agent 自己起的后台任务此刻还在跑的：交 own_background_tasks_still_running 的结果；主 agent、判不了的交空表。"""
    agent_id = hook_input.get("agent_id") or ""
    transcript_path = hook_input.get("transcript_path") or ""
    if BROKEN == "no-background" or agent_id in ("", "main"):
        return []
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl") if transcript_path.endswith(".jsonl") else ""
    if not os.path.isfile(own):
        note_unjudged("background-transcript", f"! handback-guard：找不到这个子 agent 自己的会话记录（{own or '输入里没有主会话记录'}），后台任务这一条没判（放行）")
        return []
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("agent_watch", agent_watch_path)
        agent_watch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(agent_watch)
        return agent_watch.own_background_tasks_still_running(own)
    except Exception as error:   # 文件不在、导入失败、读不了会话记录：这一条没判，放行
        note_unjudged("background", f"! handback-guard：后台任务这一条没判（放行）：{type(error).__name__}：{error}")
        return []


def background_refusal(running):
    lines = []
    for task_id, output, tops in running:
        for pid, seconds, command, stop_order in tops:
            lines.append(f"  任务 {task_id}（输出 {output}）：进程 {pid} 已跑 {int(seconds // 60)} 分 {int(seconds % 60)} 秒：{command[:160]}；"
                         f"停的次序（从最底层往上）：{' '.join(map(str, stop_order[:24]))}{' …' if len(stop_order) > 24 else ''}")
    return (f"✗ 交回时你自己起的后台任务还在跑（{len(running)} 个任务）：\n" + "\n".join(lines[:20]) +
            "\n→ 怎么办：要它的结果就结束本轮、不再调工具，等它的完成通知到了、把结果写进报告再交回；不要了就按上面的次序逐个跑 "
            "`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>`，停完 `ps -o pid,ppid,args --ppid <进程号>` 再列一遍确认一个不剩，"
            "在报告「没做什么」里写明停了哪条、为什么，再交回（.claude/agent-common.md「不做」一节停自己起的一条链那一条）。")


def own_transcript_records(hook_input):
    """这个子 agent 自己的会话记录逐条（dict）；输入里没有主会话记录或 agent_id、文件读不了交 None。"""
    transcript_path = hook_input.get("transcript_path") or ""
    agent_id = hook_input.get("agent_id") or ""
    if not transcript_path.endswith(".jsonl") or not agent_id:
        return None
    records = []
    try:
        with open(os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl"), encoding="utf-8", errors="replace") as handle:
            for line in handle:
                try:
                    record = json.loads(line)
                except ValueError:
                    continue
                if isinstance(record, dict):
                    records.append(record)
    except OSError:
        return None
    return records


def tool_calls(records):
    """会话记录里的工具调用，按次序：[(工具名, 参数, 当前目录, 跑过没有, 退出码, 结果的文字)]。跑过：有工具结果，不是错误，或错误是「Exit code N」（跑了、退非 0）；
    被钩子拒的、超时的、没有结果的不算。退出码：不是错误的是 0，「Exit code N」的是 N，别的是 None。结果的文字给 ⑤ 读「; echo <标签>$?」打出的那一行；
    输出太长、记录里只留了预览（<persisted-output> … Full output saved to: <路径>）的，结果的文字换成那份存下来的整份输出（读不到照留预览）。
    弄坏开关 naming-refused-counts 下一律算跑过（认不出的退出码当 0），naming-persisted-unread 下不读存下来的那一份。"""
    results, calls = {}, []
    for record in records:
        message = record.get("message")
        content = message.get("content") if isinstance(message, dict) else None
        for block in content if isinstance(content, list) else []:
            if not isinstance(block, dict):
                continue
            if record.get("type") == "assistant" and block.get("type") == "tool_use":
                arguments = block.get("input") if isinstance(block.get("input"), dict) else {}
                calls.append((block.get("id"), block.get("name"), arguments, record.get("cwd")))
            elif record.get("type") == "user" and block.get("type") == "tool_result":
                results[block.get("tool_use_id")] = block

    def outcome(result):
        """→ (跑过没有, 退出码, 结果的文字)。"""
        exit_code, text = None, ""
        if result is not None:
            text = result.get("content")
            if isinstance(text, list):
                text = "".join(str(part.get("text") or "") for part in text if isinstance(part, dict))
            text = text if isinstance(text, str) else ""
        if result is not None and result.get("is_error") is not True:
            exit_code = 0
        elif result is not None:
            found = re.match(r"\s*Exit code (\d+)", text)
            exit_code = int(found.group(1)) if found else None
        persisted = PERSISTED_OUTPUT.search(text[:600]) if BROKEN != "naming-persisted-unread" else None
        if persisted:
            text = text[:persisted.start()] + persisted_output(persisted.group(1), text[persisted.start():])
        if BROKEN == "naming-refused-counts":
            return True, 0 if exit_code is None else exit_code, text
        return exit_code is not None, exit_code, text
    return [(name, arguments, cwd, *outcome(results.get(call_id))) for call_id, name, arguments, cwd in calls]


def persisted_output(path, preview):
    """存成 persisted-output 的那一份整份输出；读不到（不在、太大、读错）交回预览，stderr 写一句。"""
    try:
        if os.path.getsize(path) > PERSISTED_OUTPUT_READ_LIMIT:
            raise OSError(f"大于 {PERSISTED_OUTPUT_READ_LIMIT} 字节")
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError as error:
        print(f"! handback-guard：工具结果存成了 persisted-output，读不了存下来的那一份 {path}（{error}），只按预览认", file=sys.stderr)
        return preview


def bash_rs_writes(command, directory, heavy, project_root=None):
    """⑤ 一条 Bash 命令里写到 .rs 的目标（认得出落点的，绝对路径）：输出重定向（>、>>、>|、&> 这类）的目标、tee 的文件参数、
    cp / mv 的目标（目标是 .rs，或目标是目录而来源是 .rs 时是 目录/来源的文件名；-t / --target-directory 照认）。目标里的变量照这一段命令里赋过的值展开，
    展开不了的不算。目标以命令替换起头、接着 /crates/ 或 /scripts/ 的（$(git rev-parse --show-toplevel)/crates/…rs、`pwd`/scripts/…rs 这类，拆分提案第一百二十二节 ⑪）
    按仓根展开后判（substituted_root：$(pwd) 这一种取命令当时的目录，别的取命令当时的目录往上第一层带 .git 的，找不到取 project_root）。
    弄坏开关 naming-bash-writes-ignored 下一个都不认，naming-substitution-target-unexpanded 下命令替换起头的目标照旧不算。"""
    if BROKEN == "naming-bash-writes-ignored":
        return []
    shell = heavy.shell_words
    try:
        scan = shell.commands_at_command_position(command, directory)
    except Exception as error:   # 切不开：这一条里写的 .rs 认不出
        note_unjudged("rs-split", f"! handback-guard：切不开一条 Bash 命令，它写的 .rs 没认：{type(error).__name__}：{error}")
        return []
    written = []
    for item in scan.commands:
        variables = dict(item.variables or {})
        if item.directory:
            variables["PWD"] = item.directory

        def landing(word):
            substituted = NAMING_SUBSTITUTION_TARGET.match(word) if BROKEN != "naming-substitution-target-unexpanded" else None
            if substituted:
                root = substituted_root(substituted.group("substitution"), item.directory, project_root)
                return os.path.normpath(root + substituted.group("rest")) if root else None
            expanded = shell.expand_known_variables(os.path.expanduser(word), variables)
            return shell.resolve_path(item.directory, expanded) if expanded is not None and "$" not in expanded else None
        targets = [target for _, target in item.output_redirections]
        words = list(item.words)
        name = os.path.basename(words[0]) if words else ""
        if name == "tee":
            targets += [word for word in words[1:] if not word.startswith("-")]
        elif name in NAMING_COPYING_COMMANDS:
            positional, directory_option, position, options_done = [], None, 1, False
            while position < len(words):
                word = words[position]
                if not options_done and word == "--":
                    options_done = True
                elif not options_done and word in ("-t", "--target-directory"):
                    directory_option = words[position + 1] if position + 1 < len(words) else None
                    position += 1
                elif not options_done and word.startswith("--target-directory="):
                    directory_option = word.split("=", 1)[1]
                elif not options_done and word in NAMING_COPY_OPTIONS_WITH_VALUE:
                    position += 1
                elif options_done or not word.startswith("-"):
                    positional.append(word)
                position += 1
            if directory_option is not None:
                targets += [os.path.join(directory_option, os.path.basename(source)) for source in positional if source.endswith(".rs")]
            elif len(positional) >= 2:
                destination, sources = positional[-1], positional[:-1]
                targets += ([destination] if destination.endswith(".rs") else
                            [os.path.join(destination, os.path.basename(source)) for source in sources if source.endswith(".rs")])
        for target in targets:
            if target.endswith(".rs"):
                absolute = landing(target)
                if absolute:
                    written.append(absolute)
    return written


def substituted_root(substitution, directory, project_root):
    """⑤ 目标起头的那一段命令替换按哪一棵树的根展开：$(pwd) / `pwd`（可带 -P）取命令当时的目录；$(git rev-parse --show-toplevel) 与别的写法取命令当时的目录
    往上第一层带 .git 的（git_toplevel），找不到取 project_root。"""
    inner = substitution[2:-1] if substitution.startswith("$(") else substitution[1:-1]
    if re.fullmatch(r"\s*pwd(?:\s+-[LP])?\s*", inner) and directory:
        return os.path.normpath(directory)
    return git_toplevel(directory) or (os.path.normpath(project_root) if project_root else None)


def git_toplevel(directory):
    """directory 往上第一层带 .git（目录或 worktree 的 .git 文件）的目录；没有交 None。"""
    candidate = os.path.normpath(directory) if directory else None
    while candidate:
        if os.path.exists(os.path.join(candidate, ".git")):
            return candidate
        parent = os.path.dirname(candidate)
        candidate = parent if parent != candidate else None
    return None


def naming_source_tree(path, project_root):
    """改的文件是某棵树里 crates/ 或 scripts/ 下的 .rs 时交 (那棵树的根, 判不判)；不是交 None。
    主工作区（project_root）与带 .git 的副本（git worktree）判；没有 .git 的副本（rsync 拷的，命令交主 agent 跑）不判。
    弄坏开关 naming-any-file 下不看后缀，naming-copy-judged 下没有 .git 的副本也判。"""
    if not isinstance(path, str) or not path or (not path.endswith(".rs") and BROKEN != "naming-any-file"):
        return None
    absolute = os.path.normpath(path if os.path.isabs(path) else os.path.join(project_root, path))
    parts = absolute.split(os.sep)
    trees = [os.sep.join(parts[:index]) or os.sep for index in range(1, len(parts) - 1) if parts[index] in NAMING_SOURCE_DIRECTORIES]
    if not trees:
        return None
    for project in dict.fromkeys((os.path.normpath(project_root), os.path.realpath(project_root))):
        if project in trees:
            return project, True
    for tree in reversed(trees):
        if os.path.exists(os.path.join(tree, ".git")):
            return tree, True
    return trees[-1], BROKEN == "naming-copy-judged"


def naming_check_runs(command, directory, heavy):
    """一条 Bash 命令里起的命名检查：{(哪一条, 根的真实路径)}；认法在 naming_check_occurrences，根认不出的不算。
    弄坏开关 naming-root-unmatched 下认出的根一律接一段，当成别的树。"""
    return {(kind, root + ("/别的树" if BROKEN == "naming-root-unmatched" else "")) for kind, root, _ in naming_check_occurrences(command, directory, heavy) if root}


def naming_check_occurrences(command, directory, heavy):
    """一条 Bash 命令里起的命名检查：[(哪一条, 根的真实路径或 None, 起它那一路路过的前缀名)]。哪一条是 "stage"（门禁 code-source-discipline.sh 的 --check
    点全四格，或不带 --check 的整道）或 "lint"（上游 naming-lint.sh）。命令位置照 lib_shell_words 切，two-logs.sh 这类包装照 lib_heavy_tests 的 command_under_launcher 剥；
    前缀名（timeout、nice……）取 lib_shell_words 切那一路每一层的 prefix_names。"""
    found, pending = [], [(command, directory, 0, frozenset())]
    while pending:
        text, current, depth, prefixes = pending.pop()
        try:
            scan = heavy.shell_words.commands_at_command_position(text, current)
        except Exception as error:   # 切不开的命令（引号不配对这类）：这一条不算跑过
            print(f"! handback-guard：切不开一条 Bash 命令，命名检查按没在这一条里跑算：{type(error).__name__}：{error}", file=sys.stderr)
            continue
        prefixes = prefixes | frozenset(scan.prefix_names)
        for item in scan.commands:
            words = list(item.words)
            if not words:
                continue
            unwrapped = heavy.command_under_launcher(words, item.directory) if depth < 4 else None
            if unwrapped:
                inner_directory = heavy.shell_words.resolve_path(item.directory, unwrapped[1]) if unwrapped[1] else item.directory
                pending.append((shlex.join(unwrapped[0]), inner_directory, depth + 1, prefixes))
                continue
            script = os.path.normpath(heavy.shell_words.resolve_path(item.directory, os.path.expanduser(words[0])) or words[0])
            if script.endswith(NAMING_STAGE_TAIL):
                kind, arguments = "stage", words[1:]
                cells = heavy.checked_cells(arguments)
                if "--list" in arguments or (cells is not None and not NAMING_CHECK_CELLS <= cells):
                    continue
                default = script[: -len(NAMING_STAGE_TAIL)] if os.path.isabs(script) else None
            elif script.endswith(NAMING_LINT_TAIL):
                kind, arguments, default, candidate = "lint", words[1:], None, item.directory
                while candidate and candidate != os.sep:   # 不给根时它从当前目录往上找（lib.sh 的 project_root）
                    if any(os.path.exists(os.path.join(candidate, mark)) for mark in (".git", ".singlefs-ai-sop-version")):
                        default = candidate
                        break
                    candidate = os.path.dirname(candidate)
            else:
                continue
            positional, position = [], 0
            while position < len(arguments):
                if arguments[position] == "--check":
                    position += 2
                    continue
                if not arguments[position].startswith("-"):
                    positional.append(arguments[position])
                position += 1
            root = default
            if positional:
                variables = dict(item.variables or {})
                if item.directory:
                    variables["PWD"] = item.directory
                written = item.directory if positional[0] in ("$(pwd)", "$(pwd -P)", "`pwd`") and item.directory else positional[0]
                if NAMING_GIT_TOPLEVEL_ROOT.match(positional[0]) and BROKEN != "naming-git-toplevel-unknown":
                    written = git_toplevel(item.directory) or positional[0]
                expanded = heavy.shell_words.expand_known_variables(os.path.expanduser(written), variables)
                root = heavy.shell_words.resolve_path(item.directory, expanded) if expanded is not None else None
            found.append((kind, os.path.realpath(root) if root else None, prefixes))
    return found


def naming_command_class(words, heavy):
    """一条简单命令（顶层切出来的词）的类别与它包没包会截断的前缀 → (类别, 有没有 timeout)。类别：trivial（只有赋值、env 这类前缀，没有命令）、
    cd（cd、pushd、popd、export）、terminator（exit、return、logout、exec 后面接命令：之后的命令都不跑）、naming（起的是两条命名检查之一）、other。"""
    skipped = heavy.shell_words.skip_prefixes(words)
    if skipped.command_position is None:
        return "trivial", False
    name = os.path.basename(words[skipped.command_position])
    occurrences = naming_check_occurrences(shlex.join(words), "/", heavy)
    timeout = any(NAMING_TRUNCATING_PREFIXES & prefixes for _, _, prefixes in occurrences) or bool(NAMING_TRUNCATING_PREFIXES & skipped.prefix_names)
    if name in NAMING_TERMINATING_COMMANDS or "exec" in skipped.prefix_names:
        return "terminator", timeout
    if name in NAMING_CHAIN_COMMANDS:
        return "cd", timeout
    return ("naming" if occurrences else "other"), timeout


def naming_sequence(command, heavy):
    """按顶层切一段命令文本 → (简单命令表, 顶层最后一段的号)。每条一个 dict：words、blocked（在 if / while / until / for / select / case、函数体里，
    或所在的 { } / ( ) 本身不在顺序位置上）、in_group（在 { } / ( ) 里）、list_id（第几段 && / || 连成的一段；; 换行 & 断开）、after_or（同一段里前面有 ||）、
    predecessors（同一段里它前面那几个元素的类别；管道整个算一个元素，多于一条的算 other）、in_pipeline、negated（前面有 !）、backgrounded（那一段以 & 收尾）、
    terminator_before（前面有 exit / return / logout / exec 接命令，在哪一层都算）、class、timeout；
    frame（所在那一层 { } / ( ) / 复合命令的号，顶层是 0）、piped_into（后面接 |，在管道里不是最后一段）、next_operator（它那个元素后面接的 && / || / ; / 换行 / &，没有接的是 None）、
    element_id（它所在的那个元素，即那条管道，的号）、element_position（它在那条管道里是第几段，从 0 起；整个 { } / ( ) / 复合命令占一段）。"""
    shell = heavy.shell_words
    tokens = [token for token in shell.shell_tokens(shell.strip_data_heredocs(command)) if token[0] is not None]
    commands, frames, list_counter, frame_counter = [], [{"kind": "top", "blocked": False, "list": None, "id": 0}], [0], [0]
    element_counter = [0]
    state = {"words": [], "negated": False, "pipe": False, "pipeline": [], "element": [], "function": False, "skip_name": False, "terminated": False}

    def new_list():
        list_counter[0] += 1
        return {"id": list_counter[0], "after_or": False, "predecessors": [], "members": []}
    frames[0]["list"] = new_list()
    top_list_ids = []

    def current():
        return frames[-1]["list"]

    def blocked_now():
        return any(frame["blocked"] for frame in frames)

    def position_ok():
        listing = current()
        return not listing["after_or"] and all(kind in NAMING_ALLOWED_PREDECESSORS for kind in listing["predecessors"])

    def finish_command(piped):
        words, listing = state["words"], current()
        if words:
            kind, timeout = naming_command_class(words, heavy)
            commands.append({"words": words, "blocked": blocked_now(), "in_group": len(frames) > 1, "list_id": listing["id"],
                             "after_or": listing["after_or"], "predecessors": list(listing["predecessors"]), "in_pipeline": state["pipe"] or piped,
                             "negated": state["negated"], "backgrounded": False, "terminator_before": state["terminated"], "class": kind, "timeout": timeout,
                             "frame": frames[-1]["id"], "piped_into": piped, "next_operator": None,
                             "element_id": None, "element_position": len(state["pipeline"])})
            listing["members"].append(len(commands) - 1)
            state["element"].append(len(commands) - 1)
            if len(frames) == 1:
                top_list_ids.append(listing["id"])
            state["pipeline"].append(kind)
            if kind == "terminator":
                state["terminated"] = True
        state["words"], state["negated"], state["pipe"] = [], False, piped

    def finish_element(operator=None):
        if state["pipeline"]:
            current()["predecessors"].append(state["pipeline"][0] if len(state["pipeline"]) == 1 else "other")
        element_counter[0] += 1
        for member in state["element"]:
            commands[member]["next_operator"] = operator
            commands[member]["element_id"] = element_counter[0]
        state["pipeline"], state["element"] = [], []

    def open_frame(kind, blocked):
        frame_counter[0] += 1
        frames.append({"kind": kind, "blocked": blocked, "list": new_list(), "outer_pipeline": state["pipeline"], "outer_element": state["element"], "id": frame_counter[0]})
        state["pipeline"], state["element"] = [], []

    def close_frame():
        frame = frames.pop()
        state["pipeline"] = frame["outer_pipeline"] + ["other"]   # 整个复合命令或 { } / ( ) 当一个可能失败的元素
        state["element"] = frame["outer_element"]
        if len(frames) == 1:
            top_list_ids.append(current()["id"])

    index = 0
    while index < len(tokens):
        is_operator, text = tokens[index]
        index += 1
        if not is_operator:
            if state["skip_name"]:
                state["skip_name"] = False
                continue
            if not state["words"]:
                if text in NAMING_COMPOUND_CLOSERS:
                    open_frame(text, True)
                    continue
                if text in ("then", "else", "elif", "do", "time"):
                    continue
                if text in ("fi", "done", "esac"):
                    while len(frames) > 1 and NAMING_COMPOUND_CLOSERS.get(frames[-1]["kind"]) != text:
                        close_frame()
                    if len(frames) > 1:
                        close_frame()
                    continue
                if text == "{":
                    open_frame("group", state["function"] or blocked_now() or not position_ok())
                    state["function"] = False
                    continue
                if text == "}":
                    if len(frames) > 1 and frames[-1]["kind"] == "group":
                        finish_command(False)
                        finish_element()
                        close_frame()
                    continue
                if text == "!":
                    state["negated"] = True
                    continue
                if text == "function":
                    state["function"], state["skip_name"] = True, True
                    continue
            state["words"].append(text)
            continue
        if text == "(":
            if state["words"] and NAMING_ARRAY_ASSIGNMENT.match(state["words"][-1]):
                while index < len(tokens) and tokens[index] != (True, ")"):
                    index += 1
                index += 1
                continue
            if len(state["words"]) == 1 and index < len(tokens) and tokens[index] == (True, ")"):
                state["words"], state["function"] = [], True   # f() …：函数定义，后面那一段是函数体
                index += 1
                continue
            if not state["words"]:
                open_frame("subshell", state["function"] or blocked_now() or not position_ok())
                state["function"] = False
                continue
        if text == ")":
            if len(frames) > 1 and frames[-1]["kind"] == "subshell":
                finish_command(False)
                finish_element()
                close_frame()
            continue
        if text in ("|", "|&"):
            finish_command(True)
            continue
        if text in ("&&", "||"):
            finish_command(False)
            finish_element(text)
            if text == "||":
                current()["after_or"] = True
            continue
        if text in (";", "\n", "&", ";;"):
            finish_command(False)
            finish_element(text)
            if text == "&":
                for member in current()["members"]:
                    commands[member]["backgrounded"] = True
            frames[-1]["list"] = new_list()
            continue
        if index < len(tokens) and tokens[index][0] is False:   # 重定向：连同它的目标词跳过
            index += 1
    finish_command(False)
    finish_element()
    return commands, (top_list_ids[-1] if top_list_ids else None)


def status_echo_pattern(words, heavy, saved_names=()):
    """⑤ 一条简单命令是不是「echo <标签>$?」（打出前一段的退出码）→ (认它打出的那一行的正则（退出码是第一组）, 打的是哪一个)；不是、或认不准（echo -n 不换行、
    认得的那几种不止一处）交 None。认得的：$? 与 ${?}（("status",)）、${PIPESTATUS[k]}（("pipe", k)）、saved_names 里存过退出码的 $名字 与 ${名字}（("saved", 名字)）。
    标签里别的展开（$D、${x}、$(…)、`…`）当任意字，前后的空白照收。弄坏开关 naming-pipestatus-unread 下 ${PIPESTATUS[k]} 当任意字，naming-saved-status-unread 下 $名字 当任意字。"""
    skipped = heavy.shell_words.skip_prefixes(words)
    position = skipped.command_position
    if position is None or os.path.basename(words[position]) != "echo":
        return None
    arguments = list(words[position + 1:])
    while arguments and re.fullmatch(r"-[neE]+", arguments[0]):
        if "n" in arguments[0]:
            return None
        arguments = arguments[1:]
    forms = list(NAMING_STATUS_FORMS)
    if BROKEN != "naming-pipestatus-unread":
        forms.append(NAMING_PIPE_STATUS_FORM)
    names = sorted(saved_names) if BROKEN != "naming-saved-status-unread" else []
    if names:
        alternatives = "|".join(re.escape(name) for name in names)
        forms += [r"\$\{(?P<braced>" + alternatives + r")\}", r"\$(?P<bare>" + alternatives + r")(?![A-Za-z0-9_])"]
    text = " ".join(arguments)
    found = list(re.finditer("|".join(forms), text))
    if len(found) != 1:
        return None
    match = found[0]
    groups = match.groupdict()
    source = (("pipe", int(groups["pipe"])) if groups.get("pipe") is not None else
              ("saved", groups.get("braced") or groups.get("bare")) if (groups.get("braced") or groups.get("bare")) else ("status",))
    def literal(piece):
        return ".*?".join(re.escape(part) for part in NAMING_OTHER_EXPANSION.split(piece))
    return re.compile(r"^[ \t]*" + literal(text[:match.start()]) + r"(\d+)" + literal(text[match.end():]) + r"[ \t]*$", re.M), source


def saved_status_assignment(item):
    """「名字=$?」（或 =${?}、=${PIPESTATUS[k]}）单独成一条的 → (名字, k 或 None)；不是交 None。"""
    words = item["words"]
    found = NAMING_SAVED_STATUS.match(words[0]) if len(words) == 1 and item["class"] == "trivial" else None
    return (found.group(1), int(found.group(2)) if found.group(2) is not None else None) if found else None


def status_target(commands, before_index, pipe_index):
    """echo（或存退出码的那一条）紧挨着的前一条是 commands[before_index]：它打的 $? 是那一段的（("list", 段号)），${PIPESTATUS[k]} 是那条管道第 k 段的
    （("cmd", 那一条的下标)；第 k 段不是一条简单命令、或不在同一层的交 None）。"""
    before = commands[before_index]
    if pipe_index is None:
        return ("list", before["list_id"])
    members = [index for index, other in enumerate(commands) if other["element_id"] == before["element_id"] and other["frame"] == before["frame"]
               and other["element_position"] == pipe_index]
    return ("cmd", members[0]) if len(members) == 1 else None


def status_echo_readings(commands, output, heavy):
    """⑤ 后面紧跟「; echo <标签>$?」（换行也算）的那几段 → {("list", 段号) 或 ("cmd", 下标): 输出里那一行打出的数}。echo 要是下一段的头一条、与前一条在同一层；
    「; echo <标签>${PIPESTATUS[k]}」打出的数记在紧挨着的那条管道的第 k 段上；紧跟一段的「名字=$?」（或 =${PIPESTATUS[k]}）把那一段（那一段）存下，
    之后同一层、不在条件里的「echo <标签>$名字」打出的数记在存下的那一段上（中间别的命令改写过这个名字的不认）。
    同一种标签打了几次按次序配，输出里那种行的行数对不上的不认。弄坏开关 naming-echo-unread 下一段都不认，naming-exit-code-unchecked 下读到的一律当 0。"""
    if BROKEN == "naming-echo-unread":
        return {}
    by_pattern, saved = {}, {}
    for index in range(1, len(commands)):
        item, before = commands[index], commands[index - 1]
        adjacent = not (item["blocked"] or item["frame"] != before["frame"] or item["list_id"] == before["list_id"]
                        or before["next_operator"] not in (";", "\n") or before["piped_into"])
        assignment = saved_status_assignment(item) if BROKEN != "naming-saved-status-unread" else None
        if assignment is not None:
            saved[assignment[0]] = status_target(commands, index - 1, assignment[1]) if adjacent else None
            continue
        if os.path.basename(item["words"][0]) not in NAMING_ECHO_COMMANDS:
            for name in [name for name in saved if any(word == name or word.startswith((name + "=", name + "+=")) for word in item["words"])]:
                saved[name] = None
        found = status_echo_pattern(item["words"], heavy, [name for name, target in saved.items() if target is not None])
        if found is None:
            continue
        pattern, source = found
        if source[0] == "saved":
            target = saved.get(source[1]) if not item["blocked"] else None
        elif not adjacent:
            continue
        else:
            target = status_target(commands, index - 1, source[1] if source[0] == "pipe" else None)
        if target is not None:
            by_pattern.setdefault(pattern.pattern, (pattern, []))[1].append(target)
    readings = {}
    for pattern, targets in by_pattern.values():
        printed = pattern.findall(output or "")
        if len(printed) == len(targets):
            readings.update({target: 0 if BROKEN == "naming-exit-code-unchecked" else int(code) for target, code in zip(targets, printed)})
    return readings


def sequential_naming_text(command, exit_code, heavy, carries=True, depth=0, output="", reasons=None):
    """⑤ 只留真跑完了的那几条（判决 landing-audit-r4 K3-1、landing-audit-r5 K2-2a、K2-2b）：顺序执行位置上的命令（不在 || 右侧、不在 if / while / until / for / select / case
    与函数体里、同一段里前面只有 cd、赋值、env、export 与另一条命名检查用 && 连着、不放后台、前面没有 exit / return / exec），再按命名检查自己的退出码收，只收 0 与 1：
      每一段（&& / || 连成的一段）的退出码：顶层最后一段取这一条 Bash 的退出码（退非 0 时整条只许有赋值、env、cd、export 与命名检查：set -e 之下前面一条失败、
      后面不跑也是这个退出码）；后面紧跟「; echo <标签>$?」的取输出里那一行打出的数（status_echo_readings）；别的段认不出。
      一条命名检查自己的退出码：它是那一段的最后一个元素时就是那一段的（前面 && 连着 cd 或命名检查的，那一段退 0 才认，退别的认不出是不是它退的）；
      后面接 && 的，那一段退 0 才认它退 0；后面接 ||、在管道里不是最后一段（命令里有 pipefail 时最后一段也算）、前面有 ! 的认不出。
      包着 timeout 的照这一套（没杀掉时 timeout 退的就是命名检查自己的；杀掉是 124、137，不收）；3、77、2、141 这类一样不收。
    收下的写回成一段命令文本（每条一行，shlex.join），连同前面不在条件里的赋值与 cd（目录照它们算），交 naming_check_runs 认；
    bash -c 那一段：外面那一条认出的退出码交进去（认不出交 None），照同一套往里收。
    「; echo <标签>${PIPESTATUS[k]}」认出的是管道第 k 段自己的退出码，「名字=$?」存下、之后 echo 打出的认成存下的那一段的（status_echo_readings）。
    reasons 给了一个列表时，每条认出是命名检查、却没收下的，追加 (前面收下的赋值与 cd, 这一条的词, 为什么没收) 给拒绝说明用（naming_unrecognized_reason）。
    弄坏开关 naming-tool-exit-trusted 下每一段的退出码都取这一条 Bash 的、管道里与 || 左侧的也收；naming-timeout-refused 下包着 timeout、又不是整条最后一条的不收。"""
    commands, last_top_list = naming_sequence(command, heavy)
    last_index = len(commands) - 1
    trusted = BROKEN == "naming-tool-exit-trusted"
    whole_is_naming = all(other["class"] in ("trivial", "cd", "naming") for other in commands)
    statuses = status_echo_readings(commands, output, heavy) if not trusted else {}
    if carries and exit_code is not None and last_top_list is not None and (exit_code == 0 or whole_is_naming):
        statuses[("list", last_top_list)] = exit_code
    pipefail = any("pipefail" in word for item in commands for word in item["words"])
    kept = []

    def unrecognized(item, reason):
        if reasons is not None and item["class"] == "naming":
            reasons.append((list(kept), list(item["words"]), reason))
    for position, item in enumerate(commands):
        if item["class"] in ("trivial", "cd"):
            if not item["blocked"]:
                kept.append(item["words"])
            continue
        if item["blocked"] or item["after_or"] or item["backgrounded"] or item["terminator_before"]:
            unrecognized(item, "position")
            continue
        if any(kind not in NAMING_ALLOWED_PREDECESSORS for kind in item["predecessors"]):
            unrecognized(item, "position")
            continue
        status_carried = carries and position == last_index and not item["in_group"] and not item["in_pipeline"] and not item["negated"]
        if BROKEN == "naming-timeout-refused" and item["timeout"] and not status_carried:
            unrecognized(item, "unread")
            continue
        fallible = any(kind in ("cd", "naming") for kind in item["predecessors"])
        segment = (exit_code if carries else None) if trusted else statuses.get(("list", item["list_id"]))
        reading = None if trusted else statuses.get(("cmd", position))
        why = None
        if reading is not None and not item["negated"]:
            own = reading if not fallible or reading == 0 else None
            why = None if own is not None else "and-nonzero"
        elif segment is None or item["negated"] or (not trusted and (item["piped_into"] or item["next_operator"] == "||" or (pipefail and item["in_pipeline"]))):
            own = None
            why = ("negated" if item["negated"] else "piped" if not trusted and (item["piped_into"] or (pipefail and item["in_pipeline"])) else
                   "or" if not trusted and item["next_operator"] == "||" else "unread")
        elif item["next_operator"] == "&&" or fallible:
            own = 0 if segment == 0 else None
            why = None if own is not None else "and-nonzero"
        else:
            own = segment
        words = list(item["words"])
        skipped = heavy.shell_words.skip_prefixes(words)
        code = None
        if depth < 4 and skipped.command_position is not None and os.path.basename(words[skipped.command_position]) in heavy.shell_words.SHELL_NAMES:
            code = heavy.shell_words.shell_invocation(words[skipped.command_position + 1:]).code_string
        if code is not None:
            code_index = next(index for index in range(len(words) - 1, skipped.command_position, -1) if words[index] == code)
            words[code_index] = sequential_naming_text(code, own, heavy, own is not None, depth + 1, output, reasons)
        elif own not in NAMING_COMPLETED_EXIT_CODES:
            unrecognized(item, why or f"exit-{own}")
            continue
        kept.append(words)
    return "\n".join(shlex.join(words) for words in kept)


def completed_naming_text(command, exit_code, heavy, output="", reasons=None):
    """⑤ 一条 Bash 命令里真跑完的那几条（sequential_naming_text；output 是这一条的结果文字，读「; echo <标签>$?」打出的那一行；reasons 收没收下的那几条与原因）。
    弄坏开关 naming-position-unchecked 下不看位置（退出码 0 或 1 的整条都收），naming-exit-code-unchecked 下退出码一律当 0。切不开的交空串（这一条不算跑过），stderr 写一句。"""
    if BROKEN == "naming-position-unchecked":
        return command if exit_code in NAMING_COMPLETED_EXIT_CODES else ""
    try:
        return sequential_naming_text(command, 0 if BROKEN == "naming-exit-code-unchecked" else exit_code, heavy, output=output, reasons=reasons)
    except Exception as error:   # 切词库这一版对不上、意外的写法：这一条不算跑过
        print(f"! handback-guard：认不出一条 Bash 命令里哪几条真跑完了，按没跑算：{type(error).__name__}：{error}", file=sys.stderr)
        return ""


NAMING_UNRECOGNIZED_REASONS = {
    "position": "不在顺序执行的位置上（在 if / while / for / case 或函数体里、在 || 右侧、放进了后台、前面有 exit / return / exec，或同一段里前面用 && 连着别的命令）",
    "piped": "接了管道（| tail、| head、| grep 这类，或设了 pipefail 的管道里），整条的退出码是管道的，不是它自己的",
    "or": "后面接了 ||（|| true 这类把退出码吃掉）",
    "negated": "前面有 !",
    "and-nonzero": "前面或后面用 && 连着别的命令、那一段退了非 0，认不出是不是它退的",
    "unread": "后面没有紧跟 ; echo rc=$?，它自己的退出码看不到（后面用 ; 接了别的命令，或整条退非 0 时还夹着别的命令）",
    "unread-persisted": "输出存成了 persisted-output，存下来的那一份读不到，echo rc=$? 打的那一行看不到",
}
NAMING_EXIT_MEANINGS = {124: "被 timeout 杀掉", 137: "被 timeout -s KILL 杀掉", 141: "被 SIGPIPE 杀掉（接了 | head 这类）", 2: "用法错", 3: "本次没跑", 77: "本次未跑"}


def naming_unrecognized_reason(reason, output):
    """没收下的原因码 → 一句话。exit-N 是认出了它自己的退出码、却不是跑完的 0 或 1。"""
    if reason.startswith("exit-"):
        code = reason[len("exit-"):]
        meaning = NAMING_EXIT_MEANINGS.get(int(code)) if code.isdigit() else None
        return f"它自己退了 {code}（{meaning or '不是跑完的 0 或 1'}），不算跑完"
    if reason == "unread" and "<persisted-output>" in (output or "")[:600]:
        reason = "unread-persisted"
    return NAMING_UNRECOGNIZED_REASONS.get(reason, reason)


def naming_problems(hook_input, project_root, heavy):
    """⑤：改了 .rs（Edit / Write / MultiEdit，或 Bash 里 mv、cp、重定向、tee 写的，bash_rs_writes）、改完之后没在那棵树上把两条命名检查都跑过的
    → [(树, 最后改的文件, [缺的那几条], {那一条: 跑了却没收下的原因})]：原因只记改完之后在那棵树上起过、却认不出跑完的那几条（最后一次的原因），
    一次都没起过的不在里面（拒绝说明分开写「没跑过」与「跑了但认不出退出码」）。读不了会话记录交 []（stderr 写一句）。
    弄坏开关 naming-refusal-reasons-merged 下原因一律不记（拒绝说明退回改前那一句「没跑过」）。"""
    records = own_transcript_records(hook_input)
    if records is None:
        note_unjudged("naming-transcript", "! handback-guard：找不到这个子 agent 自己的会话记录，命名检查这一条没判（放行）")
        return []
    last_edit, runs_after, attempts = {}, {}, {}

    def edited(path):
        tree = naming_source_tree(path, project_root)
        if tree is not None and tree[1]:
            last_edit[tree[0]] = path
            runs_after[tree[0]], attempts[tree[0]] = set(), {}
    for name, arguments, cwd, ran, exit_code, output in tool_calls(records):
        if name in FILE_EDITING_TOOLS and ran:
            edited(arguments.get("file_path"))
        elif name == "Bash" and ran:
            command = str(arguments.get("command") or "")
            for path in bash_rs_writes(command, cwd, heavy, project_root):
                edited(path)
            if not runs_after:
                continue
            reasons = []
            completed = completed_naming_text(command, exit_code, heavy, output, reasons)
            for kind, root in (naming_check_runs(completed, cwd, heavy) if completed else ()):
                for tree, kinds in runs_after.items():
                    if os.path.realpath(tree) == root:
                        kinds.add(kind)
            if BROKEN == "naming-refusal-reasons-merged":
                continue
            for prefix, words, reason in reasons:
                for kind, root in naming_check_runs("\n".join(shlex.join(part) for part in prefix + [words]), cwd, heavy):
                    for tree in runs_after:
                        if os.path.realpath(tree) == root:
                            attempts[tree][kind] = naming_unrecognized_reason(reason, output)
            for kind, root in naming_check_runs(command, cwd, heavy):   # 起过、却没切出原因的（套在包装里的 bash -c 这类）
                for tree in runs_after:
                    if os.path.realpath(tree) == root and kind not in runs_after[tree]:
                        attempts[tree].setdefault(kind, "认不出它自己的退出码")
    return [(tree, last_edit[tree], [kind for kind in ("stage", "lint") if kind not in runs_after[tree]],
             {kind: reason for kind, reason in attempts[tree].items() if kind not in runs_after[tree]})
            for tree in last_edit if len(runs_after[tree]) < 2]


def naming_refusal(agent_type, problems):
    def kind_name(kind):
        return "code-source-discipline.sh 那四格" if kind == "stage" else "naming-lint.sh"
    lines = []
    for tree, edited, missing, attempted in problems:
        unrun = [kind for kind in missing if kind not in attempted]
        parts = ([f"会话记录里改完之后没跑过：{'、'.join(kind_name(kind) for kind in unrun)}"] if unrun else []) + [
            f"{kind_name(kind)} 跑了，但认不出它自己的退出码：{attempted[kind]}" for kind in missing if kind in attempted]
        lines.append(f"  {tree}：最后改的是 {edited}；" + "；".join(parts))
    commands = [f"    {command.format(root=tree)}" for tree, _, _, _ in problems for command in (NAMING_STAGE_COMMAND, NAMING_LINT_COMMAND)]
    return (f"✗ {agent_type} 改了 crates/ 或 scripts/ 下的 .rs（Edit / Write，或 Bash 里 mv、cp、重定向写的），改完之后两条命名检查没都认出跑完"
            "（只认会话记录里真跑完的命令、认得出它自己的退出码是 0 或 1，报告里写跑了不算）：\n" +
            "\n".join(lines[:10]) +
            "\n→ 怎么办：在改动所在的树上跑下面两条（<根> 已填好），各自单独一条，或输出落进草稿目录的文件时写成 "
            f"`{NAMING_FILED_OUTPUT_FORM}`（echo rc=$? 紧跟在命令后面；接管道、|| true、中间夹别的命令的，退出码认不出），"
            "命中在自己改动的文件里的修掉、在别的文件里的照写，两条的命令与末尾原样输出贴进报告，再交回"
            "（.claude/agent-common.md「门禁」一节「命名检查交回前跑」）：\n" + "\n".join(commands[:20]) +
            "\n  在 rsync 拷的副本（没有 .git）里改的这里不判，照那一条把两条命令写进报告「要主 agent 跑的命令」。")


def first_prompt(records):
    """派发提示：会话记录里第一条 user 记录的文字（字符串，或 text 块连起来）。"""
    for record in records:
        if record.get("type") != "user" or not isinstance(record.get("message"), dict):
            continue
        content = record["message"].get("content")
        if isinstance(content, str):
            return content
        if isinstance(content, list):
            return "\n".join(str(block.get("text") or "") for block in content if isinstance(block, dict) and block.get("type") == "text")
        return ""
    return ""


def prompt_label_paths(prompt, label):
    """提示里「<label>：<路径>」那几行的路径（去掉反引号与句末标点，取第一个词）。"""
    paths = []
    for match in re.finditer(PROMPT_LABEL_LINE_START + re.escape(label) + r"[*\s]*[：:][*\s]*(\S+)", prompt, re.M):
        value = match.group(1).strip("`'\"").rstrip("。，；：、.,;:")
        if value:
            paths.append(value)
    return paths


def prompt_scratch_directories(prompt):
    """⑥ 本提示的草稿目录：行首「草稿目录：」一行点名的、工作根或过渡期旧根底下的目录（同目录 lib_prompt_scratch_line.py 的 scratch_directories，
    与派发闸 ⑯ ⑰、完工清理闸共用一份认法）；读不到那份库、它抛异常交空表，stderr 写一句。"""
    if prompt_scratch_line is None:
        note_unjudged("scratch-line", f"! handback-guard 读不到共用模块 lib_prompt_scratch_line.py（{prompt_scratch_line_error!r}），⑥ 没看草稿目录里起草的规格")
        return []
    try:
        return prompt_scratch_line.scratch_directories(prompt)
    except Exception as error:   # 弄坏开关写错、读不到 lib_work_root.py
        note_unjudged("scratch-line", f"! handback-guard 认本提示的草稿目录出错（{type(error).__name__}：{error}），⑥ 没看草稿目录里起草的规格")
        return []


def scribe_spec_entries(prompt, project_root, kb_spec):
    """写回员这一次的规格条目：[(规格出处, 条目)]，只收旧串、新串都在的。出处：提示点名的工作根或旧根下的 .json / .md / .txt（「报告：」那一份除外）、
    「草稿目录：」下的 spec.json / spec.md、提示正文本身。"""
    reports = {os.path.realpath(resolved(path, project_root)) for path in prompt_label_paths(prompt, "报告")}
    candidates = []
    for mention in re.findall(SCRATCH_MENTION_PREFIX + r"[^\s`'\"，。；：、（）()「」]+", prompt):
        path = resolved(mention.rstrip(".:"), project_root)
        if path.endswith(SPECIFICATION_SUFFIXES) and os.path.isfile(path) and os.path.realpath(path) not in reports:
            candidates.append(path)
    for draft in prompt_scratch_directories(prompt):
        candidates += [path for path in (os.path.join(draft, name) for name in SCRIBE_DRAFTED_SPECIFICATIONS) if os.path.isfile(path)]
    loaded = []
    for path in dict.fromkeys(candidates):
        try:
            loaded += [(path, entry) for entry in kb_spec.load_spec(path)]
        except Exception as error:   # 读不了、写法意外：这一份不算规格
            note_unjudged("spec-unreadable", f"! handback-guard：读不了规格 {path}（{type(error).__name__}：{error}），这一份没判")
    loaded += [("派发提示正文", entry) for entry in kb_spec.parse_markdown_spec(prompt)]
    return [(source, entry) for source, entry in loaded
            if str(entry.get("file") or "").strip() and entry.get("old") is not None and entry.get("new") is not None]


def missing_targets_sections(text):
    """文字里标题含「找不到的目标」的那几节的正文（到同级或更高一级的下一个标题为止；读法在 sections_titled）。"""
    return sections_titled(text, MISSING_TARGETS_SECTION)


def missing_targets_listed(texts, forms):
    """forms（规格里的写法、从仓根起的路径、绝对路径）之一写在了哪一份文字的「找不到的目标」一节里。
    弄坏开关 scribe-section-anywhere 下整份文字哪里提到都算，scribe-section-unread 下一节都不读。"""
    for _, text in texts:
        section = text if BROKEN == "scribe-section-anywhere" else "" if BROKEN == "scribe-section-unread" else missing_targets_sections(text)
        for form in forms:
            if form and re.search(r"(?<![A-Za-z0-9_\-./])" + re.escape(form) + r"(?![A-Za-z0-9_\-/]|\.[A-Za-z0-9_])", section):
                return True
    return False


def scribe_target_problems(hook_input, message, project_root, scripts_directory):
    """⑥：规格里在主工作区不在、又没写进「找不到的目标」一节的目标 → [「路径（规格 出处）」]。认不出规格、读不了会话记录、导入不了 kb-spec-check.py 交 []。"""
    records = own_transcript_records(hook_input)
    if records is None:
        note_unjudged("scribe-transcript", "! handback-guard：找不到这个写回员自己的会话记录，规格目标这一条没判（放行）")
        return []
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("kb_spec_check", os.path.join(scripts_directory, "kb", "kb-spec-check.py"))
        kb_spec = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(kb_spec)
    except Exception as error:   # 文件不在、导入失败
        note_unjudged("kb-spec", f"! handback-guard：导入不了 kb-spec-check.py，规格目标这一条没判（放行）：{type(error).__name__}：{error}")
        return []
    prompt = first_prompt(records)
    entries = scribe_spec_entries(prompt, project_root, kb_spec)
    if not entries:
        return []
    texts = [("交回正文", message)]
    for path in dict.fromkeys(REPORT_MENTION.findall(message) + prompt_label_paths(prompt, "报告")):
        absolute = resolved(path, project_root)
        if os.path.isfile(absolute):
            try:
                texts.append((path, open(absolute, encoding="utf-8", errors="replace").read()))
            except OSError as error:
                print(f"! handback-guard：读不了报告 {absolute}（{error}），这一份的「找不到的目标」没看", file=sys.stderr)
    problems, root = [], os.path.normpath(project_root)
    for source, entry in entries:
        written = str(entry["file"]).strip()
        absolute = os.path.normpath(written if os.path.isabs(written) else os.path.join(root, written))
        if os.path.exists(absolute):
            continue
        relative = os.path.relpath(absolute, root) if absolute.startswith(root + os.sep) else absolute
        if not missing_targets_listed(texts, (written, relative, absolute)):
            problems.append(f"{relative}（规格 {source}）")
    return list(dict.fromkeys(problems))


def sections_titled(text, title):
    """文字里标题含 title 的那几节的正文（到同级或更高一级的下一个标题为止；下一个标题又含 title 的接着收）。"""
    kept, level = [], None
    for line in text.splitlines():
        heading = MARKDOWN_HEADING.match(line)
        if heading:
            if level is not None and len(heading.group(1)) <= level:
                level = None
            if level is None and title in heading.group(2):
                level = len(heading.group(1))
                continue
        if level is not None:
            kept.append(line)
    return "\n".join(kept)


def table_data_rows(text):
    """markdown 表里的数据行（去掉表头那一行与 |---| 分隔行）→ [[格, …]]；格按没转义的 | 切，去掉首尾空白。"""
    rows, lines = [], text.splitlines()
    for number, line in enumerate(lines):
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", stripped.strip("|"))]
        if all(re.fullmatch(r":?-{2,}:?", cell) for cell in cells if cell) and any(cells):
            continue
        following = lines[number + 1].strip() if number + 1 < len(lines) else ""
        if following.startswith("|") and re.fullmatch(r"\|?(?:\s*:?-{2,}:?\s*\|)+\s*:?-{0,}:?\s*\|?", following):
            continue   # 表头
        rows.append(cells)
    return rows


def landing_file_index(directories, edited_files):
    """⑦ 找文件用的索引：给的几个目录底下的文件（target、.git 这类不进，至多 LANDING_INDEX_LIMIT 份）加上会话记录里改过的文件 → 去重的绝对路径表。"""
    files = []
    for directory in directories:
        for current, subdirectories, names in os.walk(directory):
            subdirectories[:] = [name for name in subdirectories if name not in LANDING_INDEX_SKIPPED]
            files += [os.path.join(current, name) for name in names]
            if len(files) > LANDING_INDEX_LIMIT:
                break
    return list(dict.fromkeys(files + [path for path in edited_files if os.path.isfile(path)]))


def landing_applies(records, heavy):
    """⑦ 这个执行员这一段写了装置或落了产物没有：Edit / Write / MultiEdit 改了装置源文件（LANDING_APPARATUS_SOURCE），Bash 里输出重定向或 tee 写了装置源文件，
    或者往 research/results/ 下写了东西（Edit / Write，Bash 的输出重定向、tee、cp / mv 的目标、two-logs.sh 的名字）。只拷、只挪装置（cp / mv）的不算写了装置。"""
    def apparatus(path):
        return bool(path) and LANDING_APPARATUS_SOURCE.search(path) is not None and os.path.basename(path) not in LANDING_NOT_APPARATUS
    def product(path):
        return bool(path) and ("/" + LANDING_PRODUCT_DIRECTORY) in ("/" + path.lstrip("./"))
    for name, arguments, cwd, ran, _, _ in tool_calls(records):
        if not ran:
            continue
        if name in FILE_EDITING_TOOLS:
            written = str(arguments.get("file_path") or "")
            if apparatus(written) or product(written):
                return True
        elif name == "Bash" and heavy is not None:
            try:
                scan = heavy.shell_words.commands_at_command_position(str(arguments.get("command") or ""), cwd)
            except Exception as error:   # 切不开：这一条不算，记一处没判全
                note_unjudged("landing-split", f"! handback-guard：切不开一条 Bash 命令，它写没写装置、落没落产物没认：{type(error).__name__}：{error}")
                continue
            for item in scan.commands:
                words = list(item.words)
                command_name = os.path.basename(words[0]) if words else ""
                targets = [target for _, target in item.output_redirections] + ([word for word in words[1:] if not word.startswith("-")] if command_name == "tee" else [])
                if any(apparatus(target) or product(target) for target in targets):
                    return True
                if (command_name in NAMING_COPYING_COMMANDS or "two-logs" in " ".join(words[:3])) and any(product(word) for word in words[1:]):
                    return True
    return False


def landing_exempted(prompt):
    """⑦ 派发提示是不是只修锚点、只补落产物的那两种：行首标签与整句才认（LANDING_EXEMPT_LINE）。弄坏开关 landing-exemption-substring 下退回改前：整份提示里有那几个字就算。"""
    if BROKEN == "landing-exemption-substring":
        return any(mark in prompt for mark in LANDING_EXEMPT_MARKS)
    return LANDING_EXEMPT_LINE.search(prompt) is not None


def landing_problems(hook_input, message, project_root, heavy=None):
    """⑦ 执行员（.claude/agents/experiment-runner.md 第 3c 步、「产出」一节）：交回正文、交回点名的报告与派发提示「报告：」那一份里，要有一节标题含「登记落地清单」、
    节里的表至少一行；每行里写的「文件:行」（.rs / .py / .sh / .toml / .tsv / .json / .md / .txt，后面紧跟 :行号）要在它的副本或主工作区里找得到、那份文件有这一行
    （找的地方：绝对路径照写；相对路径从项目根、派发提示「草稿目录：」点名的目录、读到的报告所在的目录与会话记录里改过的文件那几棵树起找，路径尾部相同的也算），
    每行最后一格（会红的用例）里的用例名（小写、至少三段下划线连着的词；被 … 截短的不算）要在那几处的 .rs / .py / .sh 与主工作区 crates/、scripts/ 里 grep 得到。
    派发提示写着「只修锚点」或「只补落产物」的不判（派发闸 ⑪ 放行的两种，不跑产物）。读不了会话记录不判，stderr 写一句。→ [说明]。
    这一段没写装置、也没落产物的（只写实验页、只把装置挪进主工作区这类；landing_applies）不判。
    弄坏开关 landing-exemption-ignored 下那两种也判，landing-applies-always 下没写装置、没落产物的也判，landing-location-unchecked 下不核文件:行，landing-case-unchecked 下不 grep 用例名。"""
    records = own_transcript_records(hook_input)
    if records is None:
        note_unjudged("landing-transcript", "! handback-guard：找不到这个执行员自己的会话记录，登记落地清单这一条没判（放行）")
        return []
    prompt = first_prompt(records)
    if BROKEN != "landing-exemption-ignored" and landing_exempted(prompt):
        return []
    if BROKEN != "landing-applies-always" and not landing_applies(records, heavy):
        return []
    texts, report_directories = [("交回正文", message)], []
    for path in dict.fromkeys(REPORT_MENTION.findall(work_root_tokens_expanded(message)) + prompt_label_paths(prompt, "报告")):
        absolute = resolved(path, project_root)
        if os.path.isfile(absolute):
            try:
                texts.append((path, open(absolute, encoding="utf-8", errors="replace").read()))
                report_directories.append(os.path.dirname(os.path.abspath(absolute)))
            except OSError as error:
                print(f"! handback-guard：读不了报告 {absolute}（{error}），这一份的登记落地清单没看", file=sys.stderr)
    sections = [(name, sections_titled(text, LANDING_SECTION)) for name, text in texts
                if any(heading and LANDING_SECTION in heading.group(2) for heading in map(MARKDOWN_HEADING.match, text.splitlines()))]
    if not sections:
        return [f"交回正文、交回点名的报告与派发提示「报告：」那一份（{'、'.join(name for name, _ in texts[1:]) or '一份都没读到'}）里都没有一节标题含「{LANDING_SECTION}」"]
    rows = [(name, cells) for name, section in sections for cells in table_data_rows(section)]
    if not rows:
        return [f"「{LANDING_SECTION}」一节（{'、'.join(name for name, _ in sections)}）里一行清单都没有（表里一行写一条登记条款：条款、函数文件:行、会红的用例）"]
    edited_files = []
    for name, arguments, _, ran, _, _ in tool_calls(records):
        if name in FILE_EDITING_TOOLS and ran and isinstance(arguments.get("file_path"), str):
            edited_files.append(os.path.normpath(arguments["file_path"]))
    roots = list(dict.fromkeys([os.path.normpath(project_root)] + prompt_scratch_directories(prompt) + report_directories))
    index = landing_file_index(roots[1:], edited_files)
    index += landing_file_index([os.path.join(project_root, name) for name in LANDING_TREE_DIRECTORIES], [])
    by_name = {}
    for path in index:
        by_name.setdefault(os.path.basename(path), []).append(path)
    line_counts = {}

    def lines_of(path):
        if path not in line_counts:
            try:
                with open(path, "rb") as handle:
                    data = handle.read()
                line_counts[path] = data.count(b"\n") + (0 if data.endswith(b"\n") or not data else 1)
            except OSError:
                line_counts[path] = 0
        return line_counts[path]

    def candidates(written):
        path = work_roots.expand_written_path(written) if work_roots else os.path.expanduser(written)
        if os.path.isabs(path):
            return [path]
        relative = os.path.normpath(path)
        found = [os.path.join(root, relative) for root in roots]
        return found + [candidate for candidate in by_name.get(os.path.basename(relative), []) if candidate.endswith(os.sep + relative) or "/" not in relative]
    problems, names = [], {}
    for source, cells in rows:
        row = " | ".join(cells)
        if BROKEN != "landing-location-unchecked":
            for written, line in LANDING_LOCATION.findall(row):
                if not any(lines_of(candidate) >= int(line) for candidate in candidates(written) if os.path.isfile(candidate)):
                    present = [candidate for candidate in candidates(written) if os.path.isfile(candidate)]
                    problems.append(f"{source}：「{row[:80]}」里的 {written}:{line} " +
                                    (f"指不到：找到的 {present[0]} 只有 {lines_of(present[0])} 行" if present else "在副本、草稿目录与主工作区里都找不到这份文件"))
        if BROKEN != "landing-case-unchecked":
            for name in LANDING_CASE_NAME.findall(cells[-1] if cells else ""):
                names.setdefault(name, (source, row))
    if names:
        remaining = set(names)
        for path in index:
            if not remaining:
                break
            if not path.endswith(LANDING_SOURCE_SUFFIXES):
                continue
            try:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    content = handle.read()
            except OSError:
                continue
            for name in [name for name in remaining if name in content]:
                if re.search(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", content):
                    remaining.discard(name)
        for name in sorted(remaining, key=lambda name: list(names).index(name)):
            source, row = names[name]
            problems.append(f"{source}：「{row[:80]}」里的用例名 {name} 在副本、草稿目录与主工作区的 .rs / .py / .sh 里都 grep 不到")
    return problems


def landing_refusal(problems):
    return ("✗ 执行员的报告里「登记落地清单」一节没有，或它的行指不到（.claude/agents/experiment-runner.md 第 3c 步、「产出」一节）：\n" +
            "\n".join(f"  {line}" for line in problems[:20]) +
            "\n→ 怎么办：把第 3c 步那张清单（草稿目录的 landing-checklist.md）原样放进报告「登记落地清单」一节，一条登记条款一行：条款、函数文件:行（行号用 grep -n 现取）、"
            "会红的用例名（贴 grep -n 确认它在的命令与原样输出）；文件:行写副本里或主工作区里真在的那一份，副本删了就写补丁里那一份的路径；写不出来的那一行停下照第 3c 步交回，"
            "报告写差哪几条。只修锚点、只补落产物的派发不判。")


def decide(hook_input, project_root, cite_check):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    running = background_problems(hook_input, os.path.join(os.path.dirname(cite_check), "agent-watch.py"))
    if running:
        return 2, background_refusal(running)
    agent_type = hook_input.get("agent_type")
    if not is_project_agent(agent_type, project_root):
        return 0, None
    message = str((hook_input.get("tool_input") or {}).get("message") or "")
    if BROKEN != "no-limit" and len(message) > HANDBACK_CHARACTER_LIMIT:
        hint = (LIMIT_HINT_BEFORE if BROKEN == "limit-hint-unnamed" else
                "→ 怎么办：全文写进报告文件：三方腿写派发提示给的 research/prompts/<轮>-*-output.md；别的类型写派发提示给的报告路径，"
                "没给报告路径就写进草稿目录的 result.md（文件名不带 report）；交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。")
        return 2, f"✗ {agent_type} 的交回正文 {len(message)} 字，超过 {HANDBACK_CHARACTER_LIMIT} 字。\n" + hint
    if (BROKEN == "cite-any-agent" or agent_type in citation_checked_agents(project_root)) and BROKEN != "no-cite":
        problems = citation_problems(message, hook_input, project_root, cite_check)
        if problems and problems[0].startswith("交回里只写相对路径的报告"):
            return 2, ("✗ 三方腿交回点名的报告找不到：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：报告写进你干活的那棵工作树（或项目根）的 research/prompts/，交回写那一份的路径（别的 worktree 里的写绝对路径）；"
                       "写好之后跑 python3 scripts/agents/cite-check.py <报告> 看它转绿，再交回。")
        if problems:
            return 2, ("✗ 报告里的引文对不上（scripts/agents/cite-check.py）：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：按每处的提示改报告里的行号或引文——行号去被引文件里现查（grep -nF），不从背景材料里数；改完再跑一次 "
                       "python3 scripts/agents/cite-check.py <报告> 看它转绿，再交回。")
    if agent_type in GREEN_COUNT_CHECKED_AGENTS:
        problems = green_count_problems(message, project_root)
        if problems:
            return 2, ("✗ 绿行没报查了多少项，或报的是 0 项：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：贴绿行时连它报的「查了多少项」一起贴；那一道报 0 项或根本没报数的，按没判写（「本次未判：…」），不写成 ✓（.claude/agent-common.md「门禁」一节）。")
    if agent_type in NAMING_CHECKED_AGENTS and BROKEN != "no-naming-check":
        heavy = heavy_tests_library()
        problems = naming_problems(hook_input, project_root, heavy) if heavy is not None else []
        if problems:
            return 2, naming_refusal(agent_type, problems)
    if agent_type in LANDING_CHECKED_AGENTS and BROKEN != "no-landing-checklist":
        problems = landing_problems(hook_input, message, project_root, heavy_tests_library())
        if problems:
            return 2, landing_refusal(problems)
    if agent_type in SCRIBE_TARGET_CHECKED_AGENTS and BROKEN != "no-scribe-targets":
        problems = scribe_target_problems(hook_input, message, project_root, os.path.dirname(os.path.dirname(cite_check)))
        if problems:
            return 2, ("✗ 写回员规格点名的目标文件在主工作区不在，交回与报告里「找不到的目标」一节也没有它：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：照 .claude/agents/kb-scribe.md 那一句办：整份不写，报告里加一节「## 找不到的目标」，逐条写规格里的文件（从仓根起的路径）与旧串，"
                       "以及 `git log --diff-filter=DR --name-only -- <文件名>` 和 `grep -rnF <旧串的一段>` 查到的新位置（查不到写查不到），再交回；"
                       "目标其实在的，核规格里的路径是不是写错了（从仓根起写）。")
    return 0, None


HEAVY_TESTS_LIBRARY = []

def heavy_tests_library():
    """同目录的 lib_heavy_tests.py（它带着 lib_shell_words）；导入不了交 None，stderr 写一句（命名检查这一条不判）。"""
    if not HEAVY_TESTS_LIBRARY:
        try:
            import importlib.util
            spec = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(sys.argv[1], "lib_heavy_tests.py"))
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            HEAVY_TESTS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败
            note_unjudged("heavy-library", f"! handback-guard：导入不了 lib_heavy_tests.py，命名检查这一条没判（放行）：{type(error).__name__}：{error}")
            HEAVY_TESTS_LIBRARY.append(None)
    return HEAVY_TESTS_LIBRARY[0]


def selftest(hook_dir):
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与过渡期旧根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    if three_way_legs is None:
        print(f"  ✗ 自检读不到三方腿类型表的读法 {os.path.join(hook_dir, 'lib_three_way_legs.py')}：{three_way_legs_error!r}")
        print("    → 怎么办：恢复那份库（三方腿类型表只登记在定义 frontmatter 的 leg: three-way，读法只在那份库里），再跑 --selftest")
        return 1
    if prompt_scratch_line is None:
        print(f"  ✗ 自检读不到草稿目录一行的认法 {os.path.join(hook_dir, 'lib_prompt_scratch_line.py')}：{prompt_scratch_line_error!r}")
        print("    → 怎么办：恢复那份库（「草稿目录：」一行怎么认只在那里，派发闸、完工清理闸与这里共用），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="handback-guard-selftest-")
    sleeper = None   # ④ 的样本：开着一个后台任务输出文件在睡的进程，自检自己起、自己停
    scratch_reports = []   # ③ 放在工作根与旧根下的报告目录，自检的 TMPDIR 之外
    try:
        cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "agents", "cite-check.py")
        for agent in ("three-way-attack", "three-way-verifier", "kb-scribe", "experiment-runner", "sweep", "implementation-writer", "tooling-writer"):
            os.makedirs(os.path.join(work, ".claude", "agents"), exist_ok=True)   # 三方腿类型表登记在 frontmatter：样本仓里攻方与核查员登记成三方腿
            open(os.path.join(work, ".claude", "agents", f"{agent}.md"), "w").write(
                f"---\nname: {agent}\nleg: three-way\n---\n# {agent}\n" if agent.startswith("three-way-") else f"# {agent}\n")
        os.makedirs(os.path.join(work, "kb"))
        open(os.path.join(work, "kb", "rule.md"), "w", encoding="utf-8").write("# 规则\n第一条：先查再写。\n")
        os.makedirs(os.path.join(work, "research", "prompts"))
        open(os.path.join(work, "research", "prompts", "r1-opus-output.md"), "w", encoding="utf-8").write("「第一条：先查再写」（`kb/rule.md:2`）\n")
        open(os.path.join(work, "research", "prompts", "r2-opus-output.md"), "w", encoding="utf-8").write("「第一条：先查再写」（`kb/rule.md:1`）\n")
        session = os.path.join(work, "session")
        os.makedirs(os.path.join(session, "subagents"))
        # 开工时刻取样本文件都建好之后：被引文件不算「开工之后改过」
        started = datetime.fromtimestamp(os.path.getmtime(os.path.join(work, "research", "prompts", "r2-opus-output.md")) + 5, timezone.utc)
        open(os.path.join(session, "subagents", "agent-a1.jsonl"), "w").write(json.dumps({"timestamp": started.isoformat()}) + "\n")
        # ④ a2 起的后台任务 blive 的输出文件有一个在睡的进程开着（交回时还在跑）；a3 起的 bdone 输出文件在、没有进程开着（跑完了）
        live_output, done_output = os.path.join(work, "tasks", "blive.output"), os.path.join(work, "tasks", "bdone.output")
        os.makedirs(os.path.dirname(live_output))
        open(done_output, "w").close()

        def background_started_record(task_id, output):
            return json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [
                {"type": "tool_result", "tool_use_id": f"t-{task_id}",
                 "content": f"Command running in background with ID: {task_id}. Output is being written to: {output}. You will be notified when it completes."}]}})
        for agent_id, task_id, output in (("a2", "blive", live_output), ("a3", "bdone", done_output)):
            open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w").write(
                json.dumps({"timestamp": started.isoformat()}) + "\n" + background_started_record(task_id, output) + "\n")
        with open(live_output, "w") as output_handle:
            sleeper = subprocess.Popen(["sleep", "120"], stdout=output_handle, stderr=subprocess.STDOUT)
        scribe_report = os.path.join(work, "research", "prompts", "scribe-result.md")
        open(scribe_report, "w", encoding="utf-8").write("✓ 文档铁律检查通过（检查 486，跳过 0）\n✓ 通过\n")
        # ③ 的报告放工作根（不看弄坏开关）与过渡期旧根下各一份，交回里工作根那一份写成 ~ 起头；自己建、finally 里删
        home = os.path.expanduser("~")
        for scratch_root in (work_roots.configured_work_root(), work_roots.legacy_root()):
            os.makedirs(scratch_root, exist_ok=True)
            scratch_reports.append(tempfile.mkdtemp(prefix="handback-guard-selftest-", dir=scratch_root))
            open(os.path.join(scratch_reports[-1], "result.md"), "w", encoding="utf-8").write("✓ 通过\n")
        new_root_report = os.path.join(scratch_reports[0], "result.md")
        new_root_report = "~" + new_root_report[len(home):] if new_root_report.startswith(home + "/") else new_root_report

        # ⑦ 执行员交回要有「登记落地清单」一节：③ ⑤ 那几格的执行员交回里带一份没有文件:行与用例名可核的清单，免得 ⑦ 盖住它们的开关轮
        landing_sample = "\n\n## 登记落地清单\n\n| 登记条款 | 落在哪个函数（文件:行） | 会红的用例 |\n|---|---|---|\n| 第十一节停机「产物上核」 | 产物上核 | 无用例 |\n"

        def case(label, agent_type, message, want, agent_id="a1"):
            hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": message}, "agent_type": agent_type,
                          "agent_id": agent_id, "transcript_path": session + ".jsonl"}
            return (label, want, decide(hook_input, work, cite_check)[0])
        cases = [
            case("超过上限的项目子 agent", "sweep", "字" * (HANDBACK_CHARACTER_LIMIT + 1), 2),
            case("正好在上限的项目子 agent", "sweep", "字" * HANDBACK_CHARACTER_LIMIT, 0),
            case("不是项目定义的 agent 不判", "general-purpose", "字" * (HANDBACK_CHARACTER_LIMIT + 1), 0),
            case("主 agent 不判", None, "字" * (HANDBACK_CHARACTER_LIMIT + 1), 0),
            case("攻方腿引文对上", "three-way-attack", "报告 research/prompts/r1-opus-output.md sha256 x", 0),
            case("攻方腿引文行号错", "three-way-attack", "报告 research/prompts/r2-opus-output.md sha256 x", 2),
            case("核查员没点名报告", "three-way-verifier", "核完了", 2),
            case("回扫员不核引文", "sweep", "报告 research/prompts/r2-opus-output.md", 0),
            case("书记员绿行带数", "kb-scribe", "✓ doc-decisions 检查通过（查了 12 项）", 0),
            case("书记员绿行没数", "kb-scribe", "✓ doc-registries 转绿", 2),
            case("书记员绿行 0 项", "kb-scribe", "✓ doc-registries 通过（查了 0 项）", 2),
            case("执行员报告文件里的绿行没数", "experiment-runner", f"报告 {os.path.relpath(scribe_report, work)}" + landing_sample, 2),
            case("执行员工作根下的报告里绿行没数（~ 写法）", "experiment-runner", f"报告 {new_root_report}" + landing_sample, 2),
            case("执行员工作根下的报告里绿行没数（$SINGLEFS_WORK_ROOT 写法）", "experiment-runner",
                 f"报告 ${work_roots.WORK_ROOT_VARIABLE}/{os.path.relpath(os.path.join(scratch_reports[0], 'result.md'), work_roots.configured_work_root())}" + landing_sample, 2),
            case("执行员过渡期旧根下的报告里绿行没数", "experiment-runner", f"报告 {os.path.join(scratch_reports[1], 'result.md')}" + landing_sample, 2),
            case("回扫员的绿行不判", "sweep", "✓ 通过", 0),
            case("交回时自己起的后台任务还在跑（项目子 agent）", "sweep", "报告 /tmp/x", 2, "a2"),
            case("交回时自己起的后台任务还在跑（通用 agent 也判）", "general-purpose", "交回", 2, "a2"),
            case("起过的后台任务都跑完了", "sweep", "交回", 0, "a3"),
            case("主 agent 不判后台任务", None, "交回", 0, None),
        ]
        # 三方腿类型表（拆分提案第九十节 ⑯）：② 读到的类型集合与登记处（定义 frontmatter 的 leg: three-way 一行）逐个相同。样本仓里的登记故意与改前写死的四种不同：
        # 攻方没登记、辩方的值带行尾注释、另登记一种不以 three-way 起头的 sample-leg、核查员写 leg: other、材料员没登记、正文里写的不算；本仓按 frontmatter 现扫，至少要有一种。
        # 再经 decide 判一遍：样本仓里 sample-leg 没点名报告拒、没登记的攻方不核引文（开关 leg-types-hardcoded 下样本仓那三格红）
        # 三方腿报告路径一处认（判决 landing-audit-r4 K2-5）：子目录下的、没填的 <轮> 占位都不认，交回里只点名这种的按没点名报告拒；
        # 认法调的是 lib_three_way_legs.py 的 leg_report_mentions_in_trees（换成替身，leg_reports_named 交回的就是替身认的）。开关 leg-report-subdirectory-kept 下这三格红
        def unnamed_refusal(message):
            refusal = decide({"tool_name": "SubagentHandback", "tool_input": {"message": message}, "agent_type": "three-way-attack", "agent_id": "a1",
                              "transcript_path": session + ".jsonl"}, work, cite_check)
            return refusal[0] == 2 and "交回里没点名报告文件" in (refusal[1] or "")
        cases += [
            ("三方腿报告路径:只点名子目录下的 -output.md 不认，按没点名报告拒", True, unnamed_refusal("报告 research/prompts/r1/r1-opus-output.md sha256 x")),
            ("三方腿报告路径:只点名没填的 <轮> 占位不认，按没点名报告拒", True, unnamed_refusal("报告 research/prompts/<轮>-opus-output.md")),
            # 只认解到项目根下 research/prompts/ 的（判决 landing-audit-r5「交回闸 ② 仓外报告」）：仓外副本里同名的那一份不当成仓里这一份（开关 leg-report-outside-root-kept 下这一格红）
            ("三方腿报告路径:只点名仓外同名的 …/research/prompts/r1-opus-output.md 不认，按没点名报告拒", True,
             unnamed_refusal(f"报告 {os.path.join(work, 'elsewhere-copy')}/research/prompts/r1-opus-output.md sha256 x")),
            case("三方腿报告路径:项目根起头的绝对路径照认、引文对上放行", "three-way-attack", f"报告 {work}/research/prompts/r1-opus-output.md sha256 x", 0),
            case("三方腿报告路径:./ 起头的照认、引文对上放行", "three-way-attack", "报告 ./research/prompts/r1-opus-output.md sha256 x", 0),
        ]
        # 本仓登记过的 git worktree 里的报告照认、按那一棵当根核引文；没登记的仓外副本（带 .git 也一样）不认；取不到 worktree 列表时只认项目根、拒绝说明写明
        # （拆分提案第九十节 ㉜ 第十一批：m5-dev 这棵登记过的 worktree 里派的三方腿把报告写在那棵树里，只认项目根时被误拒）。
        # 样本仓 trees-sample 的 kb/rule.md 第 1 行是标题、第 2 行是那句，登记过的那一棵与没登记的副本里只有那一句一行：按项目根核，两份报告的对错正好倒过来
        trees_root, registered_tree, unregistered_copy = (os.path.join(work, name) for name in ("trees-sample", "trees-registered", "trees-unregistered"))
        git_environment = {key: value for key, value in os.environ.items() if key not in three_way_legs.GIT_LOCATION_VARIABLES}

        def sample_git(directory, *arguments):
            subprocess.run(["git", "-c", "user.name=selftest", "-c", "user.email=selftest@invalid", "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null",
                            "-C", directory, *arguments], env=git_environment, check=True, capture_output=True, timeout=60)
        for directory in (trees_root, unregistered_copy):
            os.makedirs(directory)
            sample_git(directory, "init", "-q")
        sample_git(trees_root, "commit", "-q", "--allow-empty", "-m", "selftest")
        sample_git(trees_root, "worktree", "add", "-q", "--detach", registered_tree)
        list_unreadable_root = os.path.join(work, "trees-list-unreadable")   # .git 指到不在的目录：git worktree list 退非 0
        for tree, rule in ((trees_root, "# 规则\n第一条：先查再写。\n"), (registered_tree, "第一条：先查再写。\n"), (unregistered_copy, "第一条：先查再写。\n"),
                           (list_unreadable_root, "第一条：先查再写。\n")):
            os.makedirs(os.path.join(tree, "kb"), exist_ok=True)
            open(os.path.join(tree, "kb", "rule.md"), "w", encoding="utf-8").write(rule)
            os.makedirs(os.path.join(tree, ".claude", "agents"), exist_ok=True)
            open(os.path.join(tree, ".claude", "agents", "three-way-attack.md"), "w", encoding="utf-8").write("---\nname: three-way-attack\nleg: three-way\n---\n")
            if tree != trees_root:
                os.makedirs(os.path.join(tree, "research", "prompts"), exist_ok=True)
                for report_name, cited_line in (("wt-opus-output.md", 1), ("wt-wrong-opus-output.md", 2)):
                    open(os.path.join(tree, "research", "prompts", report_name), "w", encoding="utf-8").write(f"「第一条：先查再写」（`kb/rule.md:{cited_line}`）\n")
        open(os.path.join(list_unreadable_root, ".git"), "w").write(f"gitdir: {os.path.join(work, 'no-such-git-directory')}\n")
        trees_started = datetime.fromtimestamp(max(os.path.getmtime(os.path.join(tree, "kb", "rule.md")) for tree in (trees_root, registered_tree, unregistered_copy)) + 5, timezone.utc)
        open(os.path.join(session, "subagents", "agent-trees.jsonl"), "w").write(json.dumps({"timestamp": trees_started.isoformat()}) + "\n")

        def trees_decide(root, message, agent_id="trees"):
            return decide({"tool_name": "SubagentHandback", "tool_input": {"message": message}, "agent_type": "three-way-attack", "agent_id": agent_id,
                           "transcript_path": session + ".jsonl"}, root, cite_check)
        # trees-relative 这个子 agent 在登记过的那一棵里干过活（cd 进去跑过命令），交回只写相对路径
        open(os.path.join(session, "subagents", "agent-trees-relative.jsonl"), "w").write(
            json.dumps({"timestamp": trees_started.isoformat()}) + "\n" +
            json.dumps({"type": "assistant", "timestamp": trees_started.isoformat(), "cwd": trees_root, "message": {"role": "assistant", "content": [
                {"type": "tool_use", "id": "call-cd", "name": "Bash", "input": {"command": f"cd {registered_tree} && ls research/prompts"}}]}}) + "\n")

        def work_root_token_case(report_name):
            """工作根临时设成 work（登记过的那一棵就在工作根底下），交回写 $SINGLEFS_WORK_ROOT/trees-registered/…：→ (退出码, 是不是按没点名报告拒)。"""
            saved_root = os.environ.get(work_roots.WORK_ROOT_VARIABLE)
            os.environ[work_roots.WORK_ROOT_VARIABLE] = work
            try:
                refusal = trees_decide(trees_root, f"报告 ${work_roots.WORK_ROOT_VARIABLE}/trees-registered/research/prompts/{report_name} sha256 x")
            finally:
                if saved_root is None:
                    os.environ.pop(work_roots.WORK_ROOT_VARIABLE, None)
                else:
                    os.environ[work_roots.WORK_ROOT_VARIABLE] = saved_root
            return refusal[0], "交回里没点名报告文件" in (refusal[1] or "")
        copy_refusal = trees_decide(trees_root, f"报告 {unregistered_copy}/research/prompts/wt-opus-output.md sha256 x")
        unreadable_refusal = trees_decide(list_unreadable_root, f"报告 {work}/research/prompts/r1-opus-output.md sha256 x")
        cases += [
            ("三方腿报告路径:登记过的 worktree 里引文对上放行", 0, trees_decide(trees_root, f"报告 {registered_tree}/research/prompts/wt-opus-output.md sha256 x")[0]),
            ("三方腿报告路径:登记过的 worktree 里引文对不上拒", 2, trees_decide(trees_root, f"报告：{registered_tree}/research/prompts/wt-wrong-opus-output.md")[0]),
            ("三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒", True,
             copy_refusal[0] == 2 and "交回里没点名报告文件" in (copy_refusal[1] or "")),
            ("三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对上放行", 0,
             trees_decide(trees_root, "报告 research/prompts/wt-opus-output.md sha256 x", "trees-relative")[0]),
            ("三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒", 2,
             trees_decide(trees_root, "报告 research/prompts/wt-wrong-opus-output.md sha256 x", "trees-relative")[0]),
            ("三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到", (2, True),
             (lambda refusal: (refusal[0], "都找不到" in (refusal[1] or "")))(trees_decide(trees_root, "报告 research/prompts/nowhere-opus-output.md", "trees-relative"))),
            ("三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行", (0, False), work_root_token_case("wt-opus-output.md")),
            ("三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）", (2, False), work_root_token_case("wt-wrong-opus-output.md")),
            ("三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明", True,
             unreadable_refusal[0] == 2 and "交回里没点名报告文件" in (unreadable_refusal[1] or "") and "取不到本仓的 git worktree 列表" in (unreadable_refusal[1] or "")),
        ]
        library_mentions = three_way_legs.leg_report_mentions_in_trees
        three_way_legs.leg_report_mentions_in_trees = lambda text, project_root, trees=None: [(match, project_root) for match in re.finditer(r"sentinel-from-library", text)]
        try:
            named_by_library = [path for _, path in leg_reports_named("报告 sentinel-from-library、research/prompts/r1-opus-output.md", work)]
        finally:
            three_way_legs.leg_report_mentions_in_trees = library_mentions
        cases.append(("三方腿报告路径:交回闸调的是 lib_three_way_legs.py 的 leg_report_mentions_in_trees", ["sentinel-from-library"], named_by_library))
        legs_root = os.path.join(work, "legs-sample")
        os.makedirs(os.path.join(legs_root, ".claude", "agents"))
        for leg_name, leg_line in (("three-way-forward", "leg: three-way\n"), ("three-way-attack", ""), ("three-way-defense", "leg: three-way  # 注释\n"),
                                   ("three-way-verifier", "leg: other\n"), ("three-way-materials", ""), ("sample-leg", "leg: three-way\n")):
            open(os.path.join(legs_root, ".claude", "agents", f"{leg_name}.md"), "w", encoding="utf-8").write(
                f"---\nname: {leg_name}\n{leg_line}---\n正文里写的\nleg: three-way\n不算\n")
        frontmatter_scanned_legs = three_way_legs.three_way_leg_types_scanned   # 「frontmatter 里写着什么」：同一份库里不拆键值的另一种读法
        repository_root = os.path.dirname(os.path.dirname(hook_dir))
        repository_legs = frontmatter_scanned_legs(repository_root)

        def legs_case(label, agent_type, want):
            return (label, want, decide({"tool_name": "SubagentHandback", "tool_input": {"message": "核完了"}, "agent_type": agent_type}, legs_root, cite_check)[0])
        cases += [
            ("三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同", frontmatter_scanned_legs(legs_root), sorted(citation_checked_agents(legs_root))),
            ("三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种", (repository_legs, True), (sorted(citation_checked_agents(repository_root)), bool(repository_legs))),
            legs_case("三方腿类型表:样本仓登记的 sample-leg 没点名报告拒", "sample-leg", 2),
            legs_case("三方腿类型表:样本仓没登记的攻方不核引文", "three-way-attack", 0),
        ]

        # ⑤ 命名检查：样本树就是 work（项目根）；git worktree 副本带 .git 文件，rsync 副本不带
        def transcript(agent_id, records):
            with open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w", encoding="utf-8") as handle:
                handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": "# 派发"}}, ensure_ascii=False) + "\n")
                for number, (tool, arguments, outcome, cwd) in enumerate(records):
                    handle.write(json.dumps({"type": "assistant", "timestamp": started.isoformat(), "cwd": cwd, "message": {"role": "assistant", "content": [
                        {"type": "tool_use", "id": f"call-{number}", "name": tool, "input": arguments}]}}, ensure_ascii=False) + "\n")
                    result = {"type": "tool_result", "tool_use_id": f"call-{number}", "content": outcome[1]}
                    if outcome[0]:
                        result["is_error"] = True
                    handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [result]}}, ensure_ascii=False) + "\n")
        worktree, copy = os.path.join(work, "worktree"), os.path.join(work, "rsync-copy")
        os.makedirs(worktree)
        open(os.path.join(worktree, ".git"), "w").write("gitdir: /nonexistent\n")
        done, exited_red, refused = (False, "ok"), (True, "Exit code 1\n  ✗ 命中 1 处"), (True, "PreToolUse:Bash hook error: ✗ 拒")
        def edited(path):
            return ("Edit", {"file_path": path, "old_string": "a", "new_string": "b"}, done, work)
        def naming_stage(root, cells="vague-names,test-file-names,ordinal-names,data-names"):
            return ("Bash", {"command": NAMING_STAGE_COMMAND.replace("vague-names,test-file-names,ordinal-names,data-names", cells).format(root=root)}, done, work)
        def naming_lint(root, outcome=exited_red):
            return ("Bash", {"command": NAMING_LINT_COMMAND.format(root=root)}, outcome, work)
        core_source = os.path.join(work, "crates", "core", "src", "lib.rs")
        transcript("naming-unrun", [edited(core_source)])
        transcript("naming-run", [edited(core_source), naming_stage(work), naming_lint(".")])
        transcript("naming-markdown", [edited(os.path.join(work, "crates", "core", "README.md"))])
        transcript("naming-rsync-copy", [edited(os.path.join(copy, "crates", "core", "src", "lib.rs"))])
        transcript("naming-scripts", [edited(os.path.join(work, "scripts", "experiment-apparatus", "e1_probe.rs"))])
        transcript("naming-before-edit", [edited(core_source), naming_stage(work), naming_lint(work), edited(core_source)])
        transcript("naming-one-command", [edited(core_source), naming_stage(work)])
        transcript("naming-refused", [edited(core_source), naming_stage(work), naming_lint(work, refused)])
        transcript("naming-three-cells", [edited(core_source), naming_stage(work, "vague-names,test-file-names,ordinal-names"), naming_lint(work)])
        transcript("naming-worktree", [edited(os.path.join(worktree, "crates", "core", "src", "lib.rs")), naming_stage(worktree), naming_lint(worktree)])
        transcript("naming-two-logs", [edited(core_source), ("Bash", {"command": "bash scripts/hosts/two-logs.sh naming -- " + NAMING_STAGE_COMMAND.format(root=work)}, done, work),
                                       ("Bash", {"command": f"cd {work} && bash scripts/hosts/two-logs.sh lint -- " + NAMING_LINT_COMMAND.format(root="$PWD")}, done, "/")])
        transcript("naming-pwd", [edited(core_source), ("Bash", {"command": f"cd {work} && " + NAMING_STAGE_COMMAND.format(root='"$(pwd)"') + " && "
                                                                 + NAMING_LINT_COMMAND.format(root='"$(pwd)"')}, done, "/")])
        # 只认真跑完的（判决 landing-audit-r4 K3-1，攻方探针 naming_probe.py 的五种）：不在顺序位置上的、被 timeout 截断的都不算跑过
        stage_command, lint_command = NAMING_STAGE_COMMAND.format(root=work), NAMING_LINT_COMMAND.format(root=work)
        killed = (True, "Exit code 124")
        transcript("naming-false-and", [edited(core_source), ("Bash", {"command": "false && " + stage_command}, exited_red, work),
                                        ("Bash", {"command": "false && " + lint_command}, exited_red, work)])
        transcript("naming-true-or", [edited(core_source), ("Bash", {"command": "true || " + stage_command}, done, work), ("Bash", {"command": "true || " + lint_command}, done, work)])
        transcript("naming-if-false", [edited(core_source), ("Bash", {"command": f"if false; then {stage_command}; {lint_command}; fi"}, done, work)])
        transcript("naming-function-unused", [edited(core_source), ("Bash", {"command": f"check_names() {{ {stage_command}; {lint_command}; }}"}, done, work)])
        transcript("naming-timeout-killed", [edited(core_source), ("Bash", {"command": "timeout 0.1 " + stage_command}, killed, work),
                                             ("Bash", {"command": "timeout 0.1 " + lint_command}, killed, work)])
        transcript("naming-after-exit", [edited(core_source), ("Bash", {"command": f"exit 0; {stage_command}; {lint_command}"}, done, work)])
        transcript("naming-semicolon-red", [edited(core_source), ("Bash", {"command": f"{stage_command}; {lint_command}"}, exited_red, work)])
        transcript("naming-and-red", [edited(core_source), ("Bash", {"command": f"{stage_command} && {lint_command}"}, exited_red, work)])
        # 只认命名检查自己的退出码（判决 landing-audit-r5 K2-2a、K2-2b）：timeout 包着、后面 ; echo rc=$? 的按打出的数收；| head、| tail、|| true、; 连着看不到退出码的不收
        timed_stage = NAMING_STAGE_COMMAND.replace("bash .claude/gate.d", "timeout 230 nice -n 19 bash .claude/gate.d").format(root=work)
        timed_lint = "timeout 230 nice -n 19 " + lint_command
        transcript("naming-timeout-echo", [edited(core_source),
                                           ("Bash", {"command": f"D={work}; {timed_stage} > $D/stage.txt 2>&1; echo rc=$?; tail -n 12 $D/stage.txt"},
                                            (False, "rc=1\n  ✗ 数据形态的名字不合命名纪律：3 处违规"), work),
                                           ("Bash", {"command": f'D={work}; {timed_lint} > $D/lint.txt 2>&1; echo "rc=$?"; tail -n 5 $D/lint.txt'},
                                            (False, "rc=0\n  ✓ 查了 1 个 .rs 文件"), work)])
        transcript("naming-both-echo", [edited(core_source), ("Bash", {"command": f'D={work}; {stage_command} > $D/a.txt 2>&1; echo "rc=$?"; {lint_command} > $D/b.txt 2>&1; '
                                                                       'echo "rc=$?"; tail -n 3 $D/b.txt'}, (False, "rc=0\nrc=1\n  \u2717 3 处名字不合命名纪律"), work)])
        transcript("naming-head-sigpipe", [edited(core_source), ("Bash", {"command": f"{stage_command} 2>&1 | head -2"}, (False, "── vague-names"), work),
                                           ("Bash", {"command": f"{lint_command} 2>&1 | head -2"}, (False, "\n══ 命名纪律 ══"), work)])
        transcript("naming-tail-pipe", [edited(core_source), ("Bash", {"command": f"{stage_command} 2>&1 | tail -n 20"}, (False, "  ✓ 通过"), work),
                                        ("Bash", {"command": f"{lint_command} 2>&1 | tail -n 20"}, (False, "  ✓ 通过"), work)])
        transcript("naming-or-true", [edited(core_source), ("Bash", {"command": f"{stage_command} || true"}, (False, "! 本次无对象可判"), work),
                                      ("Bash", {"command": f"{lint_command} || true"}, (False, "没有 .rs 可查"), work)])
        transcript("naming-echo-usage", [edited(core_source), ("Bash", {"command": f"D={work}; {stage_command} > $D/a.txt 2>&1; echo rc=$?"}, (False, "rc=2"), work),
                                         ("Bash", {"command": f"D={work}; {lint_command} > $D/b.txt 2>&1; echo rc=$?"}, (False, "rc=2"), work)])
        transcript("naming-echo-killed", [edited(core_source), ("Bash", {"command": f"D={work}; {timed_stage} > $D/a.txt 2>&1; echo rc=$?"}, (False, "rc=124"), work),
                                          ("Bash", {"command": f"D={work}; {timed_lint} > $D/b.txt 2>&1; echo rc=$?"}, (False, "rc=124"), work)])
        transcript("naming-kill-137", [edited(core_source), ("Bash", {"command": "timeout -s KILL 0.1 " + stage_command}, (True, "Exit code 137"), work),
                                       ("Bash", {"command": "timeout -s KILL 0.1 " + lint_command}, (True, "Exit code 137"), work)])
        transcript("naming-semicolon-zero", [edited(core_source), ("Bash", {"command": f"{stage_command}; {lint_command}"}, done, work)])
        # Bash 里写 .rs 也算改动（拆分提案第九十节 ㉕）：mv、cp 到目录、> 重定向、>>、tee；写到草稿目录的不算
        def bash_call(command, outcome=done, cwd=work):
            return ("Bash", {"command": command}, outcome, cwd)
        transcript("naming-bash-mv", [bash_call(f"mv {work}/draft-lib.rs crates/core/src/lib.rs")])
        transcript("naming-bash-cp-directory", [bash_call("cp patch/files/lib.rs crates/core/src/")])
        transcript("naming-bash-redirect", [bash_call("cat > crates/core/src/lib.rs <<'EOF'\nfn probe() {}\nEOF")])
        transcript("naming-bash-append", [bash_call(f"D={work}/crates/core/src; echo '// 补一行' >> $D/lib.rs")])
        transcript("naming-bash-tee", [bash_call("printf 'fn probe() {}\\n' | tee crates/core/src/lib.rs")])
        transcript("naming-bash-mv-then-run", [bash_call(f"mv {work}/draft-lib.rs crates/core/src/lib.rs"), naming_stage(work), naming_lint(work)])
        transcript("naming-bash-scratch-copy", [bash_call(f"cp crates/core/src/lib.rs {work}/originals/lib.rs")])
        # 目标以命令替换起头的（拆分提案第一百二十二节 ⑪）：$(git rev-parse --show-toplevel) 按命令当时的目录往上找 .git 那一层展开，"$(pwd)" 按命令当时的目录展开
        transcript("naming-bash-substitution", [bash_call("cp draft-lib.rs $(git rev-parse --show-toplevel)/crates/core/src/lib.rs", done, worktree)])
        transcript("naming-bash-pwd-substitution", [bash_call("cat > \"$(pwd)/crates/core/src/lib.rs\" <<'EOF'\nfn probe() {}\nEOF")])
        transcript("naming-bash-substitution-then-run", [bash_call("cp draft-lib.rs $(git rev-parse --show-toplevel)/crates/core/src/lib.rs", done, worktree),
                                                         naming_stage(worktree), naming_lint(worktree)])
        # 根写成 $(git rev-parse --show-toplevel)：worktree 副本带 .git 文件，cd 进去跑；cd 到别的树里跑的不算在改动所在的那一棵上
        toplevel_stage = NAMING_STAGE_COMMAND.format(root='"$(git rev-parse --show-toplevel)"')
        toplevel_lint = NAMING_LINT_COMMAND.format(root='"$(git rev-parse --show-toplevel)"')
        transcript("naming-git-toplevel", [edited(os.path.join(worktree, "crates", "core", "src", "lib.rs")),
                                           bash_call(f"cd {worktree} && {toplevel_stage}", done, "/"), bash_call(f"cd {worktree} && {toplevel_lint}", done, "/")])
        transcript("naming-git-toplevel-elsewhere", [edited(os.path.join(worktree, "crates", "core", "src", "lib.rs")),
                                                     bash_call(f"cd {trees_root} && {toplevel_stage}", done, "/"), bash_call(f"cd {trees_root} && {toplevel_lint}", done, "/")])
        # ${PIPESTATUS[0]} 打出的是管道第一段（命名检查自己）的退出码；${PIPESTATUS[1]} 是 tail 的，不算
        transcript("naming-pipestatus", [edited(core_source), bash_call(f'{stage_command} 2>&1 | tail -n 20; echo "rc=${{PIPESTATUS[0]}}"', (False, "  ✓ 通过\nrc=0")),
                                         bash_call(f'{lint_command} 2>&1 | tail -n 5; echo "rc=${{PIPESTATUS[0]}}"', (False, "  \u2717 3 处\nrc=1"))])
        transcript("naming-pipestatus-tail", [edited(core_source), bash_call(f'{stage_command} 2>&1 | tail -n 20; echo "rc=${{PIPESTATUS[1]}}"', (False, "  ✓ 通过\nrc=0")),
                                              bash_call(f'{lint_command} 2>&1 | tail -n 5; echo "rc=${{PIPESTATUS[1]}}"', (False, "  ✓ 通过\nrc=0"))])
        # 「名字=$?」存下、读完汇总行之后再 echo；存下之后改写过的不认
        transcript("naming-saved-status", [edited(core_source),
                                           bash_call(f'D={work}; {stage_command} > $D/a.txt 2>&1; rc=$?; tail -n 3 $D/a.txt; echo "rc=$rc"', (False, "  ✓ 通过\nrc=0")),
                                           bash_call(f'D={work}; {lint_command} > $D/b.txt 2>&1; status=$?; grep -c \u2717 $D/b.txt; echo "naming-lint ${{status}}"', (False, "3\nnaming-lint 1"))])
        transcript("naming-saved-status-rebound", [edited(core_source),
                                                   bash_call(f'D={work}; {stage_command} > $D/a.txt 2>&1; rc=$?; rc=0; echo "rc=$rc"', (False, "rc=0")),
                                                   bash_call(f'D={work}; {lint_command} > $D/b.txt 2>&1; rc=$?; rc=0; echo "rc=$rc"', (False, "rc=0"))])
        # echo rc=$? 追加进落盘的那份文件、同一条里 tail 读出来
        transcript("naming-echo-into-file", [edited(core_source),
                                             bash_call(f'D={work}; {stage_command} > $D/a.txt 2>&1; echo "rc=$?" >> $D/a.txt; tail -n 2 $D/a.txt', (False, "  ✓ 通过\nrc=0")),
                                             bash_call(f'D={work}; {lint_command} > $D/b.txt 2>&1; echo "rc=$?" >> $D/b.txt; tail -n 2 $D/b.txt', (False, "  ✗ 3 处\nrc=1"))])
        # 退 0、输出太长存成 persisted-output：rc= 那一行在存下来的整份里、不在预览里
        persisted_directory = os.path.join(work, "tool-results")
        os.makedirs(persisted_directory)
        def persisted(name, body):
            saved = os.path.join(persisted_directory, name)
            if body is not None:
                open(saved, "w", encoding="utf-8").write(body)
            return (False, f"<persisted-output>\nOutput too large (52.1KB). Full output saved to: {saved}\n\nPreview (first 2KB):\n── vague-names\n  ✓ 通过\n...\n</persisted-output>")
        long_output = "── vague-names\n" + "  · 一行明细\n" * 4000
        transcript("naming-persisted", [edited(core_source),
                                        bash_call(f"D={work}; timeout 230 {stage_command} > $D/a.txt 2>&1; echo rc=$?; cat $D/a.txt", persisted("stage.txt", "rc=1\n" + long_output)),
                                        bash_call(f"D={work}; timeout 230 {lint_command} > $D/b.txt 2>&1; echo rc=$?; cat $D/b.txt", persisted("lint.txt", "rc=0\n" + long_output))])
        transcript("naming-persisted-gone", [edited(core_source),
                                             bash_call(f"D={work}; timeout 230 {stage_command} > $D/a.txt 2>&1; echo rc=$?; cat $D/a.txt", persisted("gone-stage.txt", None)),
                                             bash_call(f"D={work}; timeout 230 {lint_command} > $D/b.txt 2>&1; echo rc=$?; cat $D/b.txt", persisted("gone-lint.txt", None))])

        def naming_refusal_of(agent_id):
            return decide({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "implementation-writer", "agent_id": agent_id,
                           "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        tail_refusal, unrun_refusal, killed_refusal, gone_refusal = (naming_refusal_of(name) for name in
                                                                     ("naming-tail-pipe", "naming-unrun", "naming-echo-killed", "naming-persisted-gone"))
        naming_refusal_text = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "命名检查两条都跑了"}, "agent_type": "implementation-writer",
                                      "agent_id": "naming-unrun", "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases += [
            case("改了 .rs 没跑命名检查拒", "implementation-writer", "命名检查两条都跑了、都绿", 2, "naming-unrun"),
            case("改了 .rs 之后两条命名检查都跑了放行", "implementation-writer", "交回", 0, "naming-run"),
            case("只改 .md 放行", "implementation-writer", "交回", 0, "naming-markdown"),
            case("rsync 副本里改的 .rs 交主 agent 放行", "implementation-writer", "交回", 0, "naming-rsync-copy"),
            case("执行员改了 scripts/ 下 .rs 没跑命名检查拒", "experiment-runner", "交回" + landing_sample, 2, "naming-scripts"),
            case("命名检查跑在最后一次改 .rs 之前拒", "tooling-writer", "交回", 2, "naming-before-edit"),
            case("只跑了一条命名检查拒", "implementation-writer", "交回", 2, "naming-one-command"),
            case("命名检查被钩子拒了不算跑过", "implementation-writer", "交回", 2, "naming-refused"),
            case("四格点名少一格不算跑过", "implementation-writer", "交回", 2, "naming-three-cells"),
            case("git worktree 副本里改的、命名检查根写成副本根放行", "implementation-writer", "交回", 0, "naming-worktree"),
            case("经 two-logs.sh 包着跑命名检查放行", "implementation-writer", "交回", 0, "naming-two-logs"),
            case("cd 之后根写成 $(pwd)、&& 连着两条跑命名检查放行", "implementation-writer", "交回", 0, "naming-pwd"),
            case("回扫员改 .rs 不判命名检查", "sweep", "交回", 0, "naming-unrun"),
            case("false && 起的两条没执行，不算跑过", "implementation-writer", "交回", 2, "naming-false-and"),
            case("true || 起的两条没执行，不算跑过", "implementation-writer", "交回", 2, "naming-true-or"),
            case("if false; then 两条; fi 不算跑过", "implementation-writer", "交回", 2, "naming-if-false"),
            case("两条写进函数定义不调用，不算跑过", "implementation-writer", "交回", 2, "naming-function-unused"),
            case("timeout 0.1 包着、被杀退 124，不算跑过", "implementation-writer", "交回", 2, "naming-timeout-killed"),
            case("exit 0 之后的两条没执行，不算跑过", "implementation-writer", "交回", 2, "naming-after-exit"),
            case("两条用 ; 连着、后一条判红退 1（前一条的退出码看不到），拒", "implementation-writer", "交回", 2, "naming-semicolon-red"),
            case("两条用 && 连着、退 1（后一条可能被跳过）拒", "implementation-writer", "交回", 2, "naming-and-red"),
            case("timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行", "tooling-writer", "交回", 0, "naming-timeout-echo"),
            case("两条在同一条里各自 ; echo rc=$?（rc=0、rc=1 按次序配）放行", "implementation-writer", "交回", 0, "naming-both-echo"),
            case("两条接 | head -2、被 SIGPIPE 杀掉（退 141）而整条退 0，不算跑过", "implementation-writer", "交回", 2, "naming-head-sigpipe"),
            case("两条接 | tail -n 20、整条退的是 tail 的 0，不算跑过", "implementation-writer", "交回", 2, "naming-tail-pipe"),
            case("两条接 || true、退出码被吃掉，不算跑过", "implementation-writer", "交回", 2, "naming-or-true"),
            case("; echo rc=$? 打出 rc=2（用法错），不算跑过", "implementation-writer", "交回", 2, "naming-echo-usage"),
            case("timeout 230 包着、; echo rc=$? 打出 rc=124（被 timeout 杀掉），不算跑过", "tooling-writer", "交回", 2, "naming-echo-killed"),
            case("timeout -s KILL 包着、被杀退 137，不算跑过", "implementation-writer", "交回", 2, "naming-kill-137"),
            case("两条用 ; 连着、整条退 0（前一条的退出码看不到），拒", "implementation-writer", "交回", 2, "naming-semicolon-zero"),
            case("Bash 里 mv 写进 crates/ 的 .rs、没跑命名检查拒", "implementation-writer", "交回", 2, "naming-bash-mv"),
            case("Bash 里 cp 到 crates/ 下的目录、来源是 .rs，没跑命名检查拒", "tooling-writer", "交回", 2, "naming-bash-cp-directory"),
            case("Bash 里 > 重定向写 .rs、没跑命名检查拒", "implementation-writer", "交回", 2, "naming-bash-redirect"),
            case("Bash 里 >> 追加写 .rs（目标在变量里）、没跑命名检查拒", "implementation-writer", "交回", 2, "naming-bash-append"),
            case("Bash 里 tee 写 .rs、没跑命名检查拒", "implementation-writer", "交回", 2, "naming-bash-tee"),
            case("Bash 里 mv 写 .rs 之后两条命名检查都跑了放行", "implementation-writer", "交回", 0, "naming-bash-mv-then-run"),
            case("Bash 里把 .rs 拷到草稿目录不算改动放行", "implementation-writer", "交回", 0, "naming-bash-scratch-copy"),
            case("Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs、没跑命名检查拒", "implementation-writer", "交回", 2, "naming-bash-substitution"),
            case("Bash 里 > 重定向写到 \"$(pwd)/crates/…rs\"、没跑命名检查拒", "implementation-writer", "交回", 2, "naming-bash-pwd-substitution"),
            case("Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs 之后两条命名检查都跑了放行", "implementation-writer", "交回", 0, "naming-bash-substitution-then-run"),
            case("根写成 $(git rev-parse --show-toplevel)、cd 进 worktree 副本里跑放行", "implementation-writer", "交回", 0, "naming-git-toplevel"),
            case("根写成 $(git rev-parse --show-toplevel)、cd 到别的树里跑的不算在改动所在的那一棵上拒", "implementation-writer", "交回", 2, "naming-git-toplevel-elsewhere"),
            case("两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行", "implementation-writer", "交回", 0, "naming-pipestatus"),
            case("两条接 | tail、echo ${PIPESTATUS[1]} 打的是 tail 的，不算跑过", "implementation-writer", "交回", 2, "naming-pipestatus-tail"),
            case("输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行", "implementation-writer", "交回", 0, "naming-saved-status"),
            case("rc=$? 存下之后又被改写再 echo，不算跑过", "implementation-writer", "交回", 2, "naming-saved-status-rebound"),
            case("echo rc=$? 追加进落盘的那份文件、同一条里 tail 读出来放行", "implementation-writer", "交回", 0, "naming-echo-into-file"),
            case("退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行", "implementation-writer", "交回", 0, "naming-persisted"),
            case("存成 persisted-output 而存下来的那一份不在，认不出退出码拒", "implementation-writer", "交回", 2, "naming-persisted-gone"),
            ("跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法", True,
             all(part in tail_refusal for part in ("跑了，但认不出它自己的退出码", "接了管道", NAMING_FILED_OUTPUT_FORM, "→ 怎么办"))),
            ("没跑的拒绝说明写没跑过、不写认不出退出码", True, "会话记录里改完之后没跑过" in unrun_refusal and "认不出它自己的退出码" not in unrun_refusal),
            ("echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉", True, "跑了，但认不出它自己的退出码：它自己退了 124（被 timeout 杀掉）" in killed_refusal),
            ("存成 persisted-output 读不到的拒绝说明写 persisted-output", True, "跑了，但认不出它自己的退出码：输出存成了 persisted-output" in gone_refusal),
            ("改了 .rs 没跑的拒绝说明给出两条命令（根填改动所在的树）", True,
             all(part in naming_refusal_text for part in (f"ordinal-names,data-names {work}", f"naming-lint.sh {work}", "→ 怎么办"))),
        ]

        # ⑥ 写回员规格的目标：规格与报告放工作根下（派发提示认工作根下的规格文件），目标一份在 work 里、一份不在
        scribe_root = tempfile.mkdtemp(prefix="handback-guard-selftest-", dir=work_roots.configured_work_root())
        scratch_reports.append(scribe_root)
        os.makedirs(os.path.join(work, ".claude", "kb"), exist_ok=True)
        open(os.path.join(work, ".claude", "kb", "present.md"), "w", encoding="utf-8").write("旧的一行\n")
        def scribe_file(name, text):
            path = os.path.join(scribe_root, name)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write(text)
            return path
        present_spec = scribe_file("present-spec.json", json.dumps([{"file": ".claude/kb/present.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"}], ensure_ascii=False))
        missing_spec = scribe_file("missing-spec.json", json.dumps([{"file": ".claude/kb/present.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"},
                                                                    {"file": ".claude/kb/missing.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"}], ensure_ascii=False))
        drafted = scribe_file("drafted/spec.json", json.dumps([{"file": ".claude/kb/missing.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"}], ensure_ascii=False))
        plain_report = scribe_file("plain-result.md", "# 写回员报告\n\n## 改过的文件\n\n0 份。\n")
        listed_report = scribe_file("listed-result.md", "# 写回员报告\n\n## 找不到的目标\n\n- `.claude/kb/missing.md`，旧串「旧的一行」；git log 与 grep 都查不到\n\n## 改过的文件\n\n0 份。\n")
        elsewhere_report = scribe_file("elsewhere-result.md", "# 写回员报告\n\n## 找不到的目标\n\n0 条。\n\n## 改过的文件\n\n没改 `.claude/kb/missing.md`。\n")
        inline_spec = "文件：.claude/kb/missing.md\n旧串：\n```\n旧的一行\n```\n新串：\n```\n新的一行\n```\n依据：判决\n"
        for agent_id, prompt in (("scribe-present", f"规格：{present_spec}\n报告：{plain_report}\n"),
                                 ("scribe-missing", f"规格：{missing_spec}\n报告：{plain_report}\n"),
                                 ("scribe-inline", f"照下面的规格写回。\n\n{inline_spec}\n报告：{plain_report}\n"),
                                 ("scribe-drafted", f"判决 research/prompts/x-main-verification.md\n草稿目录：{os.path.dirname(drafted)}\n报告：{plain_report}\n"),
                                 ("scribe-verdict-only", f"判决 research/prompts/x-main-verification.md\n草稿目录：{os.path.join(scribe_root, 'empty-draft')}\n"),
                                 ("scribe-listed-draft", f"判决 research/prompts/x-main-verification.md\n草稿目录：{os.path.join(scribe_root, 'empty-draft')}\n"
                                                         f"前一段的材料：\n- 草稿目录：{os.path.dirname(drafted)}\n")):
            with open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w", encoding="utf-8") as handle:
                handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": prompt}}, ensure_ascii=False) + "\n")
        scribe_refusal_text = decide({"tool_name": "SubagentHandback", "tool_input": {"message": f"报告 {plain_report}"}, "agent_type": "kb-scribe",
                                      "agent_id": "scribe-missing", "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases += [
            case("写回员规格目标都在放行", "kb-scribe", f"报告 {plain_report}", 0, "scribe-present"),
            case("写回员规格目标不在、报告没写找不到的目标拒", "kb-scribe", f"报告 {plain_report}", 2, "scribe-missing"),
            case("写回员规格目标不在、报告「找不到的目标」一节写了它放行", "kb-scribe", f"报告 {listed_report}", 0, "scribe-missing"),
            case("写回员规格目标不在、只在报告别的节提到拒", "kb-scribe", f"报告 {elsewhere_report}", 2, "scribe-missing"),
            case("写回员规格写在提示正文里、目标不在拒", "kb-scribe", f"报告 {plain_report}", 2, "scribe-inline"),
            case("写回员自己起草的 spec.json 目标不在拒", "kb-scribe", f"报告 {plain_report}", 2, "scribe-drafted"),
            case("写回员没有规格不判", "kb-scribe", "停在起草之前，要主 agent 判", 0, "scribe-verdict-only"),
            # 草稿目录只认行首「草稿目录：」一行（与派发闸、完工清理闸共用 lib_prompt_scratch_line.py，判决 landing-audit-r4 K2-4）：列表里的是转述，不在那里找起草的规格
            case("写回员提示里列表起头的「- 草稿目录：」是转述，不在那里找起草的规格", "kb-scribe", "停在起草之前，要主 agent 判", 0, "scribe-listed-draft"),
            ("写回员目标不在的拒绝说明列出文件与出路", True,
             all(part in scribe_refusal_text for part in (".claude/kb/missing.md", "找不到的目标", "git log --diff-filter=DR", "→ 怎么办"))),
        ]
        # ⑦ 执行员的登记落地清单：副本、报告与派发提示都放工作根下（派发提示「草稿目录：」认工作根下的），一份装置在副本里
        landing_root = tempfile.mkdtemp(prefix="handback-guard-selftest-", dir=work_roots.configured_work_root())
        scratch_reports.append(landing_root)
        probe_source = os.path.join(landing_root, "copy", "scripts", "experiment-apparatus", "e999_landing_probe.rs")
        os.makedirs(os.path.dirname(probe_source))
        open(probe_source, "w", encoding="utf-8").write("// 装置\nfn probe_reading_of_the_segment() -> u32 {\n    7\n}\n#[test]\n"
                                                         "fn probe_reading_turns_red_without_the_function() {\n    assert_eq!(probe_reading_of_the_segment(), 7);\n}\n")
        def landing_table(location, case_name):
            return ("## 登记落地清单\n\n| 登记条款 | 落在哪个函数（文件:行） | 没落地时会红的用例 |\n|---|---|---|\n"
                    f"| 第六节 N1「段的读数」 | `{location}` `probe_reading_of_the_segment` | `{case_name}`（待主 agent 证红） |\n"
                    "| 第十一节停机「跑得完」 | 产物上核 | 无用例 |\n")
        good_location, good_case = "scripts/experiment-apparatus/e999_landing_probe.rs:2", "probe_reading_turns_red_without_the_function"
        landing_edit_records = "".join(json.dumps(record, ensure_ascii=False) + "\n" for record in (
            {"type": "assistant", "timestamp": started.isoformat(), "cwd": work, "message": {"role": "assistant", "content": [
                {"type": "tool_use", "id": "landing-edit", "name": "Edit", "input": {"file_path": probe_source, "old_string": "7", "new_string": "7"}}]}},
            {"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "landing-edit", "content": "ok"}]}}))
        landing_reports = {
            "good": "# 执行员报告\n\n## 结论\n\n跑完了。\n\n" + landing_table(good_location, good_case),
            "absent": "# 执行员报告\n\n## 结论\n\n跑完了。\n",
            "past-end": "# 执行员报告\n\n" + landing_table("scripts/experiment-apparatus/e999_landing_probe.rs:99", good_case),
            "file-missing": "# 执行员报告\n\n" + landing_table("scripts/experiment-apparatus/e998_not_written.rs:2", good_case),
            "case-missing": "# 执行员报告\n\n" + landing_table(good_location, "probe_reading_case_that_is_not_written"),
            "empty": "# 执行员报告\n\n## 登记落地清单\n\n见草稿目录。\n\n## 没做什么\n\n无。\n",
        }
        for name, body in landing_reports.items():
            open(os.path.join(landing_root, f"{name}-result.md"), "w", encoding="utf-8").write(body)
            with open(os.path.join(session, "subagents", f"agent-landing-{name}.jsonl"), "w", encoding="utf-8") as handle:
                handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content":
                             f"这一段回答的岔路：甲\n草稿目录：{landing_root}\n报告：{os.path.join(landing_root, name + '-result.md')}\n"}}, ensure_ascii=False) + "\n")
                handle.write(landing_edit_records)
        with open(os.path.join(session, "subagents", "agent-landing-backfill.jsonl"), "w", encoding="utf-8") as handle:
            handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content":
                         f"只补落产物：E999 那份。\n草稿目录：{landing_root}\n报告：{os.path.join(landing_root, 'absent-result.md')}\n"}}, ensure_ascii=False) + "\n")
            handle.write(landing_edit_records)
        with open(os.path.join(session, "subagents", "agent-landing-negated.jsonl"), "w", encoding="utf-8") as handle:   # 句中写「不是只补落产物」：不是行首标签，不算豁免
            handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content":
                         f"这一段回答的岔路：甲；这一段不是只补落产物，照登记写了装置。\n草稿目录：{landing_root}\n报告：{os.path.join(landing_root, 'absent-result.md')}\n"}}, ensure_ascii=False) + "\n")
            handle.write(landing_edit_records)
        with open(os.path.join(session, "subagents", "agent-landing-page-only.jsonl"), "w", encoding="utf-8") as handle:   # 只写实验页：没改装置、没落产物
            handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content":
                         f"这一段回答的岔路：甲\n草稿目录：{landing_root}\n报告：{os.path.join(landing_root, 'absent-result.md')}\n"}}, ensure_ascii=False) + "\n")
            handle.write(landing_edit_records.replace(probe_source, os.path.join(work, ".claude", "kb", "experiments", "999-页.md")))
        with open(os.path.join(session, "subagents", "agent-landing-product.jsonl"), "w", encoding="utf-8") as handle:   # 落了产物（Bash 重定向进 research/results/）、没改装置
            handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content":
                         f"这一段回答的岔路：甲\n草稿目录：{landing_root}\n报告：{os.path.join(landing_root, 'absent-result.md')}\n"}}, ensure_ascii=False) + "\n")
            for record in ({"type": "assistant", "timestamp": started.isoformat(), "cwd": work, "message": {"role": "assistant", "content": [
                               {"type": "tool_use", "id": "landing-run", "name": "Bash", "input": {"command": "./target/release/e999_probe > research/results/e999-probe-2026-10-23.out"}}]}},
                           {"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "landing-run", "content": ""}]}}):
                handle.write(json.dumps(record, ensure_ascii=False) + "\n")
        landing_refusal_text = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "experiment-runner", "agent_id": "landing-absent",
                                       "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases += [
            case("执行员登记落地清单的文件:行与用例名都找得到放行", "experiment-runner", "交回", 0, "landing-good"),
            case("执行员报告里没有登记落地清单一节拒", "experiment-runner", "交回", 2, "landing-absent"),
            case("执行员登记落地清单的文件:行超出那份文件的行数拒", "experiment-runner", "交回", 2, "landing-past-end"),
            case("执行员登记落地清单的文件在副本与主工作区里都找不到拒", "experiment-runner", "交回", 2, "landing-file-missing"),
            case("执行员登记落地清单的用例名 grep 不到拒", "experiment-runner", "交回", 2, "landing-case-missing"),
            case("执行员登记落地清单一节里一行都没有拒", "experiment-runner", "交回", 2, "landing-empty"),
            case("执行员登记落地清单写在交回正文里也认", "experiment-runner", "交回\n\n" + landing_table(good_location, good_case), 0, "landing-absent"),
            case("执行员只补落产物的派发不判登记落地清单", "experiment-runner", "交回", 0, "landing-backfill"),
            case("执行员提示句中写「不是只补落产物」不算豁免、照判登记落地清单", "experiment-runner", "交回", 2, "landing-negated"),
            case("执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单", "experiment-runner", "交回", 0, "landing-page-only"),
            case("执行员落了产物、没有登记落地清单一节拒", "experiment-runner", "交回", 2, "landing-product"),
            case("回扫员没有登记落地清单不判", "sweep", "交回", 0, "landing-absent"),
            ("执行员没有登记落地清单的拒绝说明写出那一节与出路", True,
             all(part in landing_refusal_text for part in ("登记落地清单", "experiment-runner.md 第 3c 步", "→ 怎么办"))),
        ]
        library_directories = prompt_scratch_line.scratch_directories
        prompt_scratch_line.scratch_directories = lambda prompt, include_roots=False: ["/sentinel-from-library"]
        try:
            directories_by_library = prompt_scratch_directories("提示里一行草稿目录都没有")
        finally:
            prompt_scratch_line.scratch_directories = library_directories
        cases.append(("草稿目录一行:⑥ 调的是 lib_prompt_scratch_line.py 的 scratch_directories", ["/sentinel-from-library"], directories_by_library))
        live_refusal = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "sweep", "agent_id": "a2",
                               "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases.append(("后台还在跑的拒绝说明列出进程号、命令与两条出路（等完成通知、按进程号停）", True,
                      all(part in live_refusal for part in (f"进程 {sleeper.pid}", "sleep 120", "blive", "完成通知", "proc.py stop"))))
        sleeper.kill()
        sleeper.wait()
        cases.append(case("那个后台任务结束之后再交回放行", "sweep", "交回", 0, "a2"))
        stdin_input = json.dumps({"tool_name": "SubagentHandback", "tool_input": {"message": "字" * (HANDBACK_CHARACTER_LIMIT + 1)}, "agent_type": "sweep"})
        mark = os.path.join(work, "selftest-mark")   # 自证起的子进程带这份标记，照样认调用方设的弄坏开关
        open(mark, "w", encoding="utf-8").write(SELFTEST_MARK_CONTENT + "\n")
        entry = subprocess.run(["bash", os.path.join(hook_dir, "handback-guard.sh")], input=stdin_input, capture_output=True, text=True,
                               env={**os.environ, "CLAUDE_PROJECT_DIR": work, SELFTEST_MARK_VARIABLE: mark}, timeout=60)
        cases.append(("走真实 stdin 入口：超长交回", 2, entry.returncode))
        # 平常跑（不带标记、不是 --selftest）设了弄坏开关：退 2、点名它只在自证里认（开关 honor-break-outside-selftest 下照认、照常判）
        outside = subprocess.run(["bash", os.path.join(hook_dir, "handback-guard.sh")],
                                 input=json.dumps({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "sweep"}),
                                 capture_output=True, text=True, timeout=60,
                                 env={**{key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE}, "CLAUDE_PROJECT_DIR": work,
                                      "HANDBACK_GUARD_BREAK": BROKEN if BROKEN == "honor-break-outside-selftest" else "no-limit"})
        cases.append(("平常跑设了 HANDBACK_GUARD_BREAK 退 2、点名它只在自证里认", (2, True),
                      (outside.returncode, any("HANDBACK_GUARD_BREAK" in line and "自证" in line for line in outside.stderr.splitlines()))))
        limit_refusal = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "字" * (HANDBACK_CHARACTER_LIMIT + 1)}, "agent_type": "sweep"},
                               work, cite_check)[1] or ""
        cases.append(("超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）", True,
                      all(part in limit_refusal for part in ("草稿目录的 result.md", "research/prompts/<轮>-*-output.md", "→ 怎么办"))))
        # 判不了而放行的几处（文件头「这一道自己出错」一段）：每一处造一个判不了的输入，在进程里调那一段，核它记进了 UNJUDGED；
        #   各处的弄坏开关 <处>-unrecorded 下那一处不记，对应的一格红
        import types
        def raising(*arguments, **keywords):
            raise RuntimeError("自证造的异常")
        unsplittable = types.SimpleNamespace(shell_words=types.SimpleNamespace(commands_at_command_position=raising))
        unjudged_session = os.path.join(work, "unjudged-session")
        os.makedirs(os.path.join(unjudged_session, "subagents"))
        open(os.path.join(unjudged_session, "subagents", "agent-u1.jsonl"), "w").close()
        own_input = {"agent_id": "u1", "transcript_path": unjudged_session + ".jsonl"}
        unreadable_report = os.path.join(work, "research", "prompts", "unreadable-report.md")
        open(unreadable_report, "w").close()
        os.chmod(unreadable_report, 0)
        spec_draft = os.path.join(work, "unjudged-draft")
        os.makedirs(spec_draft)
        open(os.path.join(spec_draft, "spec.json"), "w").close()
        failing_spec = types.SimpleNamespace(load_spec=raising, parse_markdown_spec=lambda prompt: [])
        draft_reader = types.SimpleNamespace(scratch_directories=lambda prompt, include_roots=False: [spec_draft])
        landing_records = [{"type": "assistant", "cwd": work, "message": {"role": "assistant", "content": [
                               {"type": "tool_use", "id": "unjudged-run", "name": "Bash", "input": {"command": "./probe > research/results/x.out"}}]}},
                           {"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "unjudged-run", "content": ""}]}}]
        empty_hooks = os.path.join(work, "empty-hooks")
        os.makedirs(empty_hooks)
        locked_definitions_root = os.path.join(work, "locked-definitions")
        os.makedirs(os.path.join(locked_definitions_root, ".claude", "agents"))
        locked_definition = os.path.join(locked_definitions_root, ".claude", "agents", "three-way-locked.md")
        open(locked_definition, "w").write("---\nleg: three-way\n---\n")
        os.chmod(locked_definition, 0)

        def heavy_library_from_empty_directory():
            saved_library, saved_directory = list(HEAVY_TESTS_LIBRARY), sys.argv[1]
            HEAVY_TESTS_LIBRARY.clear()
            sys.argv[1] = empty_hooks
            try:
                return heavy_tests_library()
            finally:
                HEAVY_TESTS_LIBRARY[:] = saved_library
                sys.argv[1] = saved_directory

        def with_globals(replaced, thunk):
            saved = {name: globals()[name] for name in replaced}
            globals().update(replaced)
            try:
                return thunk()
            finally:
                globals().update(saved)

        unjudged_probes = [
            ("leg-types", "读不到 lib_three_way_legs.py", lambda: with_globals({"three_way_legs": None}, lambda: citation_checked_agents(work))),
            ("leg-definitions", "② 有一份定义读不了", lambda: citation_checked_agents(locked_definitions_root)),
            ("citation-transcript", "② 找不到会话记录、认不出开工时刻", lambda: citation_since({})),
            ("cite-check", "② cite-check 退别的码", lambda: run_cite_check([sys.executable, os.path.join(work, "absent-cite-check.py")], work)),
            ("green-report", "③ 点名的报告读不了", lambda: green_count_problems("见 research/prompts/unreadable-report.md", work)),
            ("background-transcript", "④ 找不到自己的会话记录", lambda: background_problems({"agent_id": "u9", "transcript_path": unjudged_session + ".jsonl"}, cite_check)),
            ("background", "④ 读不到 agent-watch.py", lambda: background_problems(own_input, os.path.join(work, "absent-agent-watch.py"))),
            ("rs-split", "⑤ 切不开写 .rs 的 Bash 命令", lambda: bash_rs_writes("cp a.rs b.rs", work, unsplittable, work)),
            ("naming-transcript", "⑤ 找不到自己的会话记录", lambda: naming_problems({}, work, None)),
            ("heavy-library", "⑤ 导入不了 lib_heavy_tests.py", heavy_library_from_empty_directory),
            ("scratch-line", "⑥ 读不到 lib_prompt_scratch_line.py", lambda: with_globals({"prompt_scratch_line": None}, lambda: prompt_scratch_directories("提示"))),
            ("scribe-transcript", "⑥ 找不到写回员自己的会话记录", lambda: scribe_target_problems({}, "交回", work, os.path.dirname(cite_check))),
            ("kb-spec", "⑥ 导入不了 kb-spec-check.py", lambda: scribe_target_problems(own_input, "交回", work, os.path.join(work, "absent-scripts"))),
            ("spec-unreadable", "⑥ 草稿目录里的规格读不了", lambda: with_globals({"prompt_scratch_line": draft_reader}, lambda: scribe_spec_entries("提示", work, failing_spec))),
            ("landing-transcript", "⑦ 找不到执行员自己的会话记录", lambda: landing_problems({}, "交回", work, None)),
            ("landing-split", "⑦ 切不开那一条 Bash 命令", lambda: landing_applies(landing_records, unsplittable)),
        ]
        saved_stderr = sys.stderr
        for site, description, probe in unjudged_probes:
            UNJUDGED.clear()
            sys.stderr = open(os.devnull, "w")
            try:
                probe()
            finally:
                sys.stderr.close()
                sys.stderr = saved_stderr
            cases.append((f"没判全:{description}记一处（{site}）", True, any(noted == site for noted, _ in UNJUDGED)))
        UNJUDGED.clear()
        os.chmod(unreadable_report, 0o600)
        os.chmod(locked_definition, 0o600)
        # 入口那三处：放行时检出记录真的多一行（结局只记、findings 以 UNJUDGED_PREFIX 起头）；在进程里照入口调 judge，或走真实 stdin 入口
        unjudged_detections = os.path.join(work, "unjudged-detections.jsonl")

        def unjudged_lines():
            try:
                with open(unjudged_detections, encoding="utf-8") as handle:
                    return [json.loads(line) for line in handle if line.strip()]
            except OSError:
                return []

        def judged_in_process(hook_input, replaced):
            """→ (退出码或「抛出 …」, 检出多了几行, 最后一行的结局, findings 都以 UNJUDGED_PREFIX 起头)。"""
            before = len(unjudged_lines())
            saved_detections, saved_stderr = os.environ.get("AGENT_HOOK_DETECTIONS"), sys.stderr
            os.environ["AGENT_HOOK_DETECTIONS"], sys.stderr = unjudged_detections, open(os.devnull, "w")
            UNJUDGED.clear()
            try:
                code = with_globals(replaced, lambda: judge(hook_input, hook_dir))
            except Exception as error:   # 弄坏开关 exception-unrecorded：照原样抛出
                code = f"抛出 {type(error).__name__}"
            finally:
                sys.stderr.close()
                sys.stderr = saved_stderr
                UNJUDGED.clear()
                if saved_detections is None:
                    os.environ.pop("AGENT_HOOK_DETECTIONS", None)
                else:
                    os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
            added = unjudged_lines()[before:]
            return (code, len(added), added[-1].get("outcome") if added else None,
                    bool(added) and all(str(finding).startswith(UNJUDGED_PREFIX) for finding in added[-1].get("findings") or [None]))
        plain_input = {"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "sweep", "session_id": "s-unjudged"}
        cases.append(("入口:判的时候抛异常放行并记一条检出（结局只记）", (0, 1, "只记", True), judged_in_process(plain_input, {"decide": raising})))
        cases.append(("入口:读不到 lib_work_root.py 放行并记一条检出（结局只记）", (0, 1, "只记", True), judged_in_process(plain_input, {"work_roots": None})))
        before = len(unjudged_lines())
        broken_entry = subprocess.run(["bash", os.path.join(hook_dir, "handback-guard.sh")], input='{"tool_name": "SubagentHandback"', capture_output=True, text=True,
                                      timeout=60, env={**os.environ, "CLAUDE_PROJECT_DIR": work, SELFTEST_MARK_VARIABLE: mark, "AGENT_HOOK_DETECTIONS": unjudged_detections})
        added = unjudged_lines()[before:]
        cases.append(("走真实 stdin 入口：坏 JSON 放行并记一条检出（结局只记、会话号读不到）", (0, 1, "只记", "读不到"),
                      (broken_entry.returncode, len(added), added[-1].get("outcome") if added else None, added[-1].get("session_id") if added else None)))
        failures = [item for item in cases if item[1] != item[2]]
        for label, want, got in failures:
            print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
        if failures:
            print("    → 看 decide()、citation_problems()、leg_reports_named()、green_count_problems()、naming_problems()、sequential_naming_text()、naming_check_runs()、"
                  "bash_rs_writes()、status_echo_readings()、tool_calls()、naming_refusal()、landing_problems()、scribe_target_problems()、prompt_scratch_directories()、main() 的判法；"
                  "HANDBACK_GUARD_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ handback-guard 自检通过（查了 {len(cases)} 种）：子 agent 交回时自己起的后台任务还在跑拒（通用 agent 也判、主 agent 不判、说明里列出进程号与两条出路），"
              "跑完了或停掉之后放行；项目子 agent 超长交回拒（拒绝说明写明 result.md 与三方腿的 -output.md）、正好在上限放行、非项目 agent 与主 agent 不判；"
              "三方腿（类型读定义 frontmatter 的 leg: three-way，样本仓与本仓读到的都与 frontmatter 逐个相同）引文对不上或没点名报告拒、对上放行；书记员与执行员绿行没数或报 0 项拒（交回正文与点名的报告文件都查，报告在 research/prompts/、工作根（含 ~ 写法）与过渡期旧根下都认）；"
              "实现员、执行员、工具员改了 .rs 没在改完之后跑两条命名检查拒（跑在改之前、只跑一条、被钩子拒、少一格都拒，false &&、true ||、if、函数体、exit 之后、timeout 杀掉退 124 或 137、"
              "echo 打出 124 或 2、| head、| tail、|| true、; 连着前一条看不到退出码、&& 连着退 1 都不算跑完，拒绝说明给出两条命令），跑了、只改 .md、rsync 副本、worktree 副本根对上、two-logs.sh 包着、"
              "根写成 $(pwd) 或 $(git rev-parse --show-toplevel)、timeout 包着后面 ; echo rc=$? 打出 0 或 1、| tail 之后 echo ${PIPESTATUS[0]}、rc=$? 存下再 echo、"
              "输出存成 persisted-output 而 rc= 在存下的那一份里放行；Bash 里 mv、cp、>、>>、tee 写的 .rs 也算改动；拒绝说明分开写没跑过与跑了但认不出退出码；"
              "三方腿只点名子目录下、<轮> 占位与仓外同名的报告按没点名拒，项目根起头、./ 起头、$SINGLEFS_WORK_ROOT/ 起头的照认，相对路径按干活的那棵 worktree 核、哪一棵都没有拒；"
              "写回员规格目标不在而「找不到的目标」一节没写拒（规格文件、提示正文、草稿目录里起草的都认），目标都在、那一节写了、没有规格放行；"
              "执行员写了装置或落了产物而登记落地清单没有、一行都没有、文件:行指不到、用例名 grep 不到拒，只补落产物、只写实验页不判；"
              "走真实 stdin 入口拒得出，平常跑设了弄坏开关退 2 并点名只在自证里认；"
              "写 .rs 的目标以 $(git rev-parse --show-toplevel)、\"$(pwd)\" 起头的按仓根展开、照判命名检查；只修锚点、只补落产物的豁免只认行首标签与整句，句中「不是只补落产物」照判登记落地清单；"
              f"判不了而放行的 {len(UNJUDGED_SITES)} 处各记进没判全的几处，放行时检出记录多一行（结局只记）")
        return 0
    finally:
        if sleeper is not None and sleeper.poll() is None:
            sleeper.kill()
            sleeper.wait()
        shutil.rmtree(work, ignore_errors=True)
        for directory in scratch_reports:
            shutil.rmtree(directory, ignore_errors=True)


def selftest_child():
    """这一次是不是 --selftest 起的子进程：环境里 HANDBACK_GUARD_SELFTEST_MARK 指的那份文件在、内容是 SELFTEST_MARK_CONTENT。读完就从环境里摘掉，不往下传。"""
    mark = os.environ.pop(SELFTEST_MARK_VARIABLE, "")
    try:
        return bool(mark) and open(mark, encoding="utf-8").read().strip() == SELFTEST_MARK_CONTENT
    except OSError:
        return False


def main():
    hook_dir = sys.argv[1]
    selftest_run = len(sys.argv) > 2 and sys.argv[2] == "--selftest"
    if BROKEN and not selftest_run and not selftest_child() and BROKEN != "honor-break-outside-selftest":
        print(f"  ✗ handback-guard：HANDBACK_GUARD_BREAK={BROKEN} 设着，这个弄坏开关只在自证里认（--selftest 与它起的子进程），平常跑不认：这一次没判，退 2", file=sys.stderr)
        print("    → 怎么办：unset HANDBACK_GUARD_BREAK 再来；要看一条开关弄坏了什么，在 bash .claude/hooks/handback-guard.sh --selftest 前面设它", file=sys.stderr)
        return 2
    if selftest_run:
        return selftest(hook_dir)
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw.decode("utf-8"))
        problem = None if isinstance(hook_input, dict) else f"顶层是 {type(hook_input).__name__}，不是 JSON 对象"
    except ValueError as error:
        hook_input, problem = None, f"{error}；前 200 字节：{raw[:200].decode('utf-8', 'replace')!r}"
    if problem:
        note_unjudged("input", f"! handback-guard 读不懂 hook 的 JSON，这一次放行：{problem}")
        if UNJUDGED:
            print(record_unjudged(None), file=sys.stderr)
        return 0
    return judge(hook_input, hook_dir)


def judge(hook_input, hook_dir):
    """读得懂的 hook 输入 → 退出码。放行而有判不了的几处（UNJUDGED）照「放行但必记检出」记一条检出；判的时候抛异常照常放行、同样记
    （弄坏开关 exception-unrecorded 下照原样抛出，改前的样子）。"""
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "agents", "cite-check.py")
    if work_roots is None:
        note_unjudged("work-root", f"! handback-guard 读不到 {os.path.join(hook_dir, 'lib_work_root.py')}（{work_roots_error!r}）：工作根与旧根下的报告这一次没判")
    try:
        code, message = decide(hook_input, project_root, cite_check)
    except Exception as error:
        if BROKEN == "exception-unrecorded":
            raise
        note_unjudged("exception", f"! handback-guard：判的时候抛异常（{type(error).__name__}: {error}），这一次放行")
        code, message = 0, None
    if message:
        print(message, file=sys.stderr)
    if code == 0 and UNJUDGED:
        print(record_unjudged(hook_input), file=sys.stderr)
    return code

sys.exit(main())
PY
