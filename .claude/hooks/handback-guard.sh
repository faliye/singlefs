#!/usr/bin/env bash
# PreToolUse hook（SubagentHandback）：子 agent 交回之前拦七样——自己起的后台任务还在跑（全部子 agent），交回正文太长、三方腿报告里的引文对不上、
# 书记员与执行员的绿行不报数、实现员执行员工具员改了 .rs 而报告里没把两条命名检查交主 agent、写回员规格的目标不在又没写进「找不到的目标」、执行员报告里的登记落地清单没有或指不到
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
#      哪一棵都没有的拒（拆分提案第九十节 ㉟）；
#      找不到它的会话记录（认不出开工时刻）就不判这一条，stderr 写一句；
#   ③ kb-scribe / experiment-runner：交回正文与它点名的报告文件（research/prompts/、工作根或会话临时目录根 /tmp/claude-<uid> 下的 .md；两个根怎么认只在同目录 lib_work_root.py）里，以 ✓ 起头的行没有数字、
#      或写「查了 0 项」这类零项的，拒，逐行列出——绿行要连它报的查了多少项一起贴，0 项或没有数的按没判写。
#   ⑤ implementation-writer / experiment-runner / tooling-writer（拆分提案第九十节 ⑥，共用约束「门禁」一节「命名检查交主 agent 跑」）：它自己的会话记录里
#      用 Edit / Write / MultiEdit 写了 crates/ 或 scripts/ 下的 .rs（工具结果不是错误），或 Bash 里写了这样的 .rs（输出重定向 >、>>、>|、&> 的目标，tee 的文件参数，
#      cp / mv 的目标：目标是 .rs，或目标是目录、来源是 .rs；-t 照认；目标里的变量照这一段赋过的值展开；拆分提案第九十节 ㉕；同一条里先算写、再算命名检查；
#      目标以命令替换起头、接着 /crates/ 或 /scripts/ 的按仓根展开：$(pwd) 取命令当时的目录，$(git rev-parse --show-toplevel) 与别的写法取命令当时的目录往上第一层带 .git 的，
#      找不到取项目根，拆分提案第一百二十二节 ⑪），
#      改动所在的树是主工作区（CLAUDE_PROJECT_DIR）、带 .git 的副本（git worktree）或没有 .git 的副本（rsync 拷的，那棵树按仓根算），而这一次读的报告（naming_report_texts：派发提示里行首「报告：」点名的那几份；
#      提示里没有这一行的，取交回正文点名的）里，标题含「要主 agent 跑的命令」的那几节（sections_titled，到同级或更高一级的下一个标题为止）没把那棵树的两条命名检查都写上：
#      门禁 code-source-discipline.sh 的 `--check` 点名的格含 vague-names、test-file-names、ordinal-names、data-names 四格、后面跟着根；上游 naming-lint.sh 后面跟着根
#      （NAMING_STAGE_IN_REPORT、NAMING_LINT_IN_REPORT；根照 resolved 展开 ~、$HOME、$SINGLEFS_WORK_ROOT 起头的写法，相对路径从项目根起，真实路径与那棵树相同才算）。
#      子 agent 一律不往统一任务服务交任务（records/2026-10-29-subagent拆分提案.md 第一百七十四节），命名检查由主 agent 交：只看报告里写没写，
#      不看会话记录里跑没跑过。读不了的报告按那一节没写算。改动所在的树没有 .git 的（rsync 拷的副本）照仓根判：那一节两条的根要填仓根
#      （共用约束写明 <根> 填仓根、两条交主 agent 在补丁打进主工作区后跑）。
#      拒了就逐棵树列出最后改的 .rs、缺哪一条、读的是哪几份报告，给出两条命令原样（NAMING_SUBMIT_COMMANDS，<根> 填那棵树），出路是写进那一节再交回。
#      认不出的：Bash 里 cp -r 整个目录、install、sed -i、python 写的 .rs；改完又被删掉的 git worktree 副本看不到 .git，按 rsync 副本照仓根判；
#      报告里两条写成别的样子的（--check 与格名之间夹别的词、根写成变量或命令替换）不认，照拒绝说明给的原样写。
#      弄坏开关 naming-report-section-unchecked（整份报告都找，不看节）、naming-report-root-unchecked（根不核）、naming-rsync-copy-unjudged（rsync 副本不判）。
#   ⑥ kb-scribe（拆分提案第九十节 ⑦，.claude/agents/kb-scribe.md「规格里的目标文件不存在」那一句）：规格里每一条的目标文件（kb-spec-check.py 的 load_spec /
#      parse_markdown_spec 认的两种写法，旧串、新串都在的条目才算）在主工作区（CLAUDE_PROJECT_DIR）不在，交回正文、交回点名的报告与派发提示「报告：」那一份里
#      都没有一节标题含「找不到的目标」、节里写着这个文件（从仓根起的路径或绝对路径）的，拒，逐个列出。规格从派发提示（它会话记录的第一条）里找：
#      提示点名的工作根或会话临时目录根下的 .json / .md / .txt（「报告：」那一份除外），「草稿目录：」下写回员自己起草的 spec.json / spec.md，提示正文本身（markdown 写法）；
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
#   往拒的方向偏的几处不算放行、不记：⑤ ⑥ ⑦ 读不了的报告按那一节没写算。
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
#   HANDBACK_GUARD_BREAK=limit-hint-unnamed  ① 超长的拒绝说明只写一句（「没给就写进草稿目录」，不写 result.md 与三方腿的文件名）
#       超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=no-cite  decide() 不判 ② 三方腿与核查员的引文
#       三方里攻方那一路的引文行号错（「应当返回 2，实际 0」）
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
#       「正好在上限的项目子 agent 应当返回 0，实际 2」
#       「回扫员不核引文 应当返回 0，实际 2」
#       「入口:读不到 lib_work_root.py 放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 (2, 0, None, False)」
#       「书记员绿行带数 应当返回 0，实际 2」
#       「回扫员的绿行不判 应当返回 0，实际 2」
#       「起过的后台任务都跑完了 应当返回 0，实际 2」
#       「三方腿类型表:样本仓没登记的攻方不核引文 应当返回 0，实际 2」
#       「改了 .rs、报告「要主 agent 跑的命令」一节写了两条命名检查（根填那棵树）放行 应当返回 0，实际 2」
#       「改 .rs 的 Edit 被钩子拒了不算改动放行 应当返回 0，实际 2」
#       「只改 .md 放行 应当返回 0，实际 2」
#       「rsync 副本里改了 .rs、那一节两条的根填仓根放行 应当返回 0，实际 2」
#       「git worktree 副本里改的、那一节根写成副本根放行 应当返回 0，实际 2」
#       「回扫员改 .rs 不判命名检查 应当返回 0，实际 2」
#       「Bash 里 mv 写 .rs 之后报告写了两条命名检查放行 应当返回 0，实际 2」
#       「Bash 里把 .rs 拷到草稿目录不算改动放行 应当返回 0，实际 2」
#       「Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs 之后那一节根写成副本根放行 应当返回 0，实际 2」
#       「改了 .rs 报告没写的拒绝说明给出两条命令（根填改动所在的树）、写进「要主 agent 跑的命令」一节、不教自己交 应当返回 True，实际 False」
#       「派发提示没有「报告：」一行的拒绝说明写明一份报告都没读到 应当返回 True，实际 False」
#       「写回员规格目标都在放行 应当返回 0，实际 2」
#       「写回员规格目标不在、报告「找不到的目标」一节写了它放行 应当返回 0，实际 2」
#       「写回员没有规格不判 应当返回 0，实际 2」
#       「写回员提示里列表起头的「- 草稿目录：」是转述，不在那里找起草的规格 应当返回 0，实际 2」
#       「写回员目标不在的拒绝说明列出文件与出路 应当返回 True，实际 False」
#       「执行员登记落地清单的文件:行与用例名都找得到放行 应当返回 0，实际 2」
#       「执行员登记落地清单写在交回正文里也认 应当返回 0，实际 2」
#       「执行员只补落产物的派发不判登记落地清单 应当返回 0，实际 2」
#       「执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单 应当返回 0，实际 2」
#       「回扫员没有登记落地清单不判 应当返回 0，实际 2」
#       「执行员没有登记落地清单的拒绝说明写出那一节与出路 应当返回 True，实际 False」
#       「那个后台任务结束之后再交回放行 应当返回 0，实际 2」
#   HANDBACK_GUARD_BREAK=no-green-count  green_count_problems() 不判 ③ 绿行没有数
#       书记员绿行没数（「应当返回 2，实际 0」）
#       执行员报告文件里的绿行没数（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（~ 写法）（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（$SINGLEFS_WORK_ROOT 写法）（「应当返回 2，实际 0」）
#       执行员会话临时目录根下的报告里绿行没数（「应当返回 2，实际 0」）
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
#   HANDBACK_GUARD_BREAK=no-naming-check  decide() 不判 ⑤ 改了 .rs 而报告里没把两条命名检查交主 agent
#       「改了 .rs、报告里没有「要主 agent 跑的命令」一节拒 应当返回 2，实际 0」
#       「改了 .rs、那一节只写了一条命名检查拒 应当返回 2，实际 0」
#       「改了 .rs、那一节里四格点名少一格不算拒 应当返回 2，实际 0」
#       「改了 .rs、那一节两条的根填的是别的树拒 应当返回 2，实际 0」
#       「改了 .rs、两条写在报告别的节里拒 应当返回 2，实际 0」
#       「改了 .rs、派发提示没有「报告：」一行、交回也没点名报告拒 应当返回 2，实际 0」
#       「rsync 副本里改了 .rs、报告没列两条命名检查拒 应当返回 2，实际 0」
#       「rsync 副本里改了 .rs、那一节两条的根填副本根拒 应当返回 2，实际 0」
#       「执行员改了 scripts/ 下 .rs、报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 mv 写进 crates/ 的 .rs、报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 cp 到 crates/ 下的目录、来源是 .rs，报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 > 重定向写 .rs、报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 >> 追加写 .rs（目标在变量里）、报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 tee 写 .rs、报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs、报告没写命名检查拒 应当返回 2，实际 0」
#       「Bash 里 > 重定向写到 "$(pwd)/crates/…rs"、报告没写命名检查拒 应当返回 2，实际 0」
#       「改了 .rs 报告没写的拒绝说明给出两条命令（根填改动所在的树）、写进「要主 agent 跑的命令」一节、不教自己交 应当返回 True，实际 False」
#       「派发提示没有「报告：」一行的拒绝说明写明一份报告都没读到 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=naming-report-section-unchecked  naming_report_commands() 不认节：报告里哪儿写了两条都算（推的，没跑）
#       改了 .rs、两条写在报告别的节里拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=naming-report-root-unchecked  naming_report_commands() 不核根：那一节写了两条就算，根填的是哪棵树都认
#       「改了 .rs、那一节两条的根填的是别的树拒 应当返回 2，实际 0」
#       「rsync 副本里改了 .rs、那一节两条的根填副本根拒 应当返回 2，实际 0」
#   HANDBACK_GUARD_BREAK=naming-any-file  naming_source_tree() 不看后缀，crates/ 或 scripts/ 下改什么文件都要命名检查
#       只改 .md 放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-rsync-copy-unjudged  naming_source_tree() 没有 .git 的副本（rsync 拷的）不判
#       「rsync 副本里改了 .rs、报告没列两条命名检查拒 应当返回 2，实际 0」
#       「rsync 副本里改了 .rs、那一节两条的根填副本根拒 应当返回 2，实际 0」
#   HANDBACK_GUARD_BREAK=naming-substitution-target-unexpanded  bash_rs_writes() 目标以命令替换起头的照旧不算（$(git rev-parse --show-toplevel)/crates/…rs 认不出落点）
#       Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs、报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 > 重定向写到 "$(pwd)/crates/…rs"、报告没写命名检查拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=naming-bash-writes-ignored  bash_rs_writes() 一个都不认（Bash 里 mv、cp、重定向、tee 写的 .rs 不算改动）
#       Bash 里 mv 写进 crates/ 的 .rs、报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 cp 到 crates/ 下的目录、来源是 .rs，报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 > 重定向写 .rs、报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 >> 追加写 .rs（目标在变量里）、报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 tee 写 .rs、报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs、报告没写命名检查拒（「应当返回 2，实际 0」）
#       Bash 里 > 重定向写到 "$(pwd)/crates/…rs"、报告没写命名检查拒（「应当返回 2，实际 0」）
#       「没判全:⑤ 切不开写 .rs 的 Bash 命令记一处（rs-split） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=leg-report-relative-root-only  leg_reports_named() 只写相对路径的报告一律按项目根（干活的那棵 worktree 不看，找不到也不拒）
#       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
#       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
#   HANDBACK_GUARD_BREAK=leg-report-work-root-token-kept  work_root_tokens_expanded() 不展开 $SINGLEFS_WORK_ROOT/（② 认不出这种写法）
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
#   HANDBACK_GUARD_BREAK=landing-exemption-substring  landing_exempted() 整份提示里有「只修锚点」「只补落产物」几个字就算豁免，不看行首标签与整句
#       执行员提示句中写「不是只补落产物」不算豁免、照判登记落地清单（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=landing-applies-always  landing_problems() 没写装置、没落产物的执行员也判
#       执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=landing-location-unchecked  landing_problems() 不核清单里的文件:行
#       执行员登记落地清单的文件:行超出那份文件的行数拒（「应当返回 2，实际 0」）
#       执行员登记落地清单的文件在副本与主工作区里都找不到拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=landing-case-unchecked  landing_problems() 不 grep 清单里的用例名
#       执行员登记落地清单的用例名 grep 不到拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=honor-break-outside-selftest  main() 平常跑设了弄坏开关也照认、照常判
#       平常跑设了 HANDBACK_GUARD_BREAK 退 2、点名它只在自证里认（「应当返回 (2, True)，实际 (0, False)」）
#   HANDBACK_GUARD_BREAK=leg-report-subdirectory-kept  leg_reports_named() 用这里自己的一份认法（子目录、<轮> 占位都认），不调 lib_three_way_legs.py
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
#   HANDBACK_GUARD_BREAK=leg-types-hardcoded  citation_checked_agents() 不读定义 frontmatter 的 leg: three-way，用写死的四种（三种腿加没有定义的 three-way-verifier）
#       三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同（「应当返回 ['sample-leg', 'three-way-defense', 'three-way-forward']，实际 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier']」）
#       三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种（「应当返回 (['three-way-attack', 'three-way-defense', 'three-way-forward'], True)，实际 (['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier'], True)」）
#       三方腿类型表:样本仓登记的 sample-leg 没点名报告拒（「应当返回 2，实际 0」）
#       三方腿类型表:样本仓没登记的攻方不核引文（「应当返回 0，实际 2」）
#       「没判全:读不到 lib_three_way_legs.py记一处（leg-types） 应当返回 True，实际 False」
#       「没判全:② 有一份定义读不了记一处（leg-definitions） 应当返回 True，实际 False」
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（同目录 lib_work_root.py：工作根退回会话临时目录根）  ③ 认不出工作根下的报告文件，⑥ 认不出派发提示点名的工作根下的规格与报告
#       执行员工作根下的报告里绿行没数（~ 写法）（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（$SINGLEFS_WORK_ROOT 写法）（「应当返回 2，实际 0」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行（「应当返回 (0, False)，实际 (2, True)」）
#       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (2, True)」）
#       写回员规格目标不在、报告没写找不到的目标拒（「应当返回 2，实际 0」）
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#       写回员自己起草的 spec.json 目标不在拒（「应当返回 2，实际 0」）
#       写回员目标不在的拒绝说明列出文件与出路（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=leg-types-unrecorded  citation_checked_agents() 读不到 lib_three_way_legs.py 时只打 stderr、不记进没判全的几处
#       「没判全:读不到 lib_three_way_legs.py记一处（leg-types） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=leg-definitions-unrecorded  citation_checked_agents() 有定义读不了（三方腿类型表读不全）时只打 stderr、不记进没判全的几处（静默跳过那一份）
#       「没判全:② 有一份定义读不了记一处（leg-definitions） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=citation-transcript-unrecorded  citation_since() 找不到这个子 agent 的会话记录时只打 stderr、不记进没判全的几处
#       「没判全:② 找不到会话记录、认不出开工时刻记一处（citation-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=cite-check-unrecorded  run_cite_check() cite-check 跑不起来、超时、退 0 与 1 以外的码时只打 stderr、不记进没判全的几处
#       「没判全:② cite-check 退别的码记一处（cite-check） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=green-report-unrecorded  green_count_problems() 点名的报告读不了时只打 stderr、不记进没判全的几处
#       「没判全:③ 点名的报告读不了记一处（green-report） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=background-transcript-unrecorded  background_problems() 找不到这个子 agent 自己的会话记录时只打 stderr、不记进没判全的几处
#       「没判全:④ 找不到自己的会话记录记一处（background-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=background-unrecorded  background_problems() 读不到 agent-watch.py 或它抛异常时只打 stderr、不记进没判全的几处
#       「没判全:④ 读不到 agent-watch.py记一处（background） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=rs-split-unrecorded  bash_rs_writes() 切不开一条 Bash 命令时只打 stderr、不记进没判全的几处
#       「没判全:⑤ 切不开写 .rs 的 Bash 命令记一处（rs-split） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=naming-transcript-unrecorded  naming_problems() 找不到这个子 agent 自己的会话记录时只打 stderr、不记进没判全的几处
#       「没判全:⑤ 找不到自己的会话记录记一处（naming-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=project-code-library-unrecorded  project_code_runs_library() 导入不了 lib_project_code_runs.py 时只打 stderr、不记进没判全的几处
#       「没判全:⑤ 导入不了 lib_project_code_runs.py记一处（project-code-library） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=scratch-line-unrecorded  prompt_scratch_directories() 读不到 lib_prompt_scratch_line.py 或它抛异常时只打 stderr、不记进没判全的几处
#       「没判全:⑥ 读不到 lib_prompt_scratch_line.py记一处（scratch-line） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=scribe-transcript-unrecorded  scribe_target_problems() 找不到写回员自己的会话记录时只打 stderr、不记进没判全的几处
#       「没判全:⑥ 找不到写回员自己的会话记录记一处（scribe-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=kb-spec-unrecorded  scribe_target_problems() 导入不了 kb-spec-check.py 时只打 stderr、不记进没判全的几处
#       「没判全:⑥ 导入不了 kb-spec-check.py记一处（kb-spec） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=spec-unreadable-unrecorded  scribe_spec_entries() 规格读不了时只打 stderr、不记进没判全的几处
#       「没判全:⑥ 草稿目录里的规格读不了记一处（spec-unreadable） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=landing-transcript-unrecorded  landing_problems() 找不到执行员自己的会话记录时只打 stderr、不记进没判全的几处
#       「没判全:⑦ 找不到执行员自己的会话记录记一处（landing-transcript） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=landing-split-unrecorded  landing_applies() 切不开一条 Bash 命令时只打 stderr、不记进没判全的几处
#       「没判全:⑦ 切不开那一条 Bash 命令记一处（landing-split） 应当返回 True，实际 False」
#   HANDBACK_GUARD_BREAK=input-unrecorded  main() 读不懂 hook 的 JSON（坏 JSON、顶层不是对象）时只打 stderr、不记检出
#       「走真实 stdin 入口：坏 JSON 放行并记一条检出（结局只记、会话号读不到） 应当返回 (0, 1, '只记', '读不到')，实际 …」
#   HANDBACK_GUARD_BREAK=exception-unrecorded  judge() 判的时候抛异常不接住、照原样抛出（进程退 1，放行、不记检出）
#       「入口:判的时候抛异常放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   HANDBACK_GUARD_BREAK=work-root-unrecorded  judge() 读不到 lib_work_root.py 时只打 stderr、不记检出
#       「入口:读不到 lib_work_root.py 放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere（同目录 lib_prompt_scratch_line.py：「草稿目录」在一行里哪都认）  ⑥ 把列表里转述的「- 草稿目录：」也当本提示的草稿目录
#       写回员提示里列表起头的「- 草稿目录：」是转述，不在那里找起草的规格（「应当返回 0，实际 2」）
#   SINGLEFS_WORK_ROOT_BREAK=session-temporary-mentions-unseen（同上：不挑会话临时目录根下的路径）  ③ 认不出会话临时目录根下的报告文件
#       执行员会话临时目录根下的报告里绿行没数（「应当返回 2，实际 0」）
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$HOOK_DIR/../singlefs-ai-sop/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shlex, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone

HANDBACK_CHARACTER_LIMIT = int(os.environ.get("HANDBACK_CHARACTER_LIMIT", "4000"))
CITATION_CHECKED_AGENTS_BEFORE_REGISTRY = {"three-way-forward", "three-way-attack", "three-way-defense", "three-way-verifier"}   # 写死的那一份：只有弄坏开关 leg-types-hardcoded 下还在用
GREEN_COUNT_CHECKED_AGENTS = {"kb-scribe", "experiment-runner"}
# ② 三方腿报告路径的认法只在同目录 lib_three_way_legs.py 的 leg_report_mentions_in_trees（与派发闸 ⑰ 共用，判决 landing-audit-r4 K2-5：占位与子目录下的不认）；
# 这里自己的一份连子目录、占位都认，只有弄坏开关 leg-report-subdirectory-kept 下还在用
LEG_REPORT_MENTION_BEFORE_LIBRARY = re.compile(r"research/prompts/[^\s`'\"，。；：、（）()「」]+-output\.md")
try:   # 工作根与会话临时目录根的唯一定义：同目录的 lib_work_root.py（读不到时那两处下的报告认不出、不判，stderr 写一句）
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
# ⑤ 改了 .rs 要在报告里交主 agent 跑命名检查的类型、四格与两条命令（共用约束 .claude/agent-common.md「门禁」一节「命名检查交主 agent 跑」，<根> 填改动所在的树）
NAMING_CHECKED_AGENTS = {"implementation-writer", "experiment-runner", "tooling-writer"}
NAMING_CHECK_CELLS = frozenset({"vague-names", "test-file-names", "ordinal-names", "data-names"})
NAMING_SOURCE_DIRECTORIES = ("crates", "scripts")
NAMING_SUBMIT_COMMANDS = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; "
                          "范围=4 格; 为什么=交回前核这一件改动里的命名' task-server/task submit -- bash .claude/gate.d/code-source-discipline.sh "
                          "--check vague-names,test-file-names,ordinal-names,data-names {root}",
                          "task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/naming-lint.sh {root}")
# ⑤ 报告里交主 agent 跑的那一节（标题含这几个字）与节里认的两条：门禁 code-source-discipline.sh 的 --check 点名的格含那四格、跟着根；上游 naming-lint.sh 跟着根
NAMING_COMMANDS_SECTION = "要主 agent 跑的命令"
NAMING_STAGE_IN_REPORT = re.compile(r"code-source-discipline\.sh\s+--check(?:\s+|=)(?P<cells>[A-Za-z0-9_,-]+)\s+(?P<root>[^\s`'\"|;&()<>，。；、]+)")
NAMING_LINT_IN_REPORT = re.compile(r"naming-lint\.sh\s+(?P<root>[^\s`'\"|;&()<>，。；、]+)")
NAMING_ANY_ROOT = "<哪棵树都认>"   # 只有弄坏开关 naming-report-root-unchecked 下记它（naming_report_commands）
# ⑤ Bash 里写 .rs 也算改动（拆分提案第九十节 ㉕）：输出重定向（>、>>、>|、&>）的目标、tee 的文件参数、cp / mv 的目标
NAMING_COPYING_COMMANDS = frozenset({"cp", "mv"})
# ⑤ 写 .rs 的目标以命令替换起头、接着 /crates/ 或 /scripts/ 的（拆分提案第一百二十二节 ⑪：$(git rev-parse --show-toplevel)/crates/…rs 原来认不出落点、不计）
NAMING_SUBSTITUTION_TARGET = re.compile(r"^[\"']?(?P<substitution>\$\([^()]*\)|`[^`]*`)[\"']?(?P<rest>/(?:crates|scripts)/\S*\.rs)[\"']?$")
NAMING_COPY_OPTIONS_WITH_VALUE = frozenset({"-S", "--suffix"})
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
LANDING_APPARATUS_SOURCE = re.compile(r"(?:^|/)(?:crates/singlefs-verification/src/bin|scripts/experiment-apparatus|scripts/experiments)/(?:[^/\s]+/)*[^/\s]+\.(?:rs|py|sh)$"
                                      r"|(?:^|/)e\d{3}_[A-Za-z0-9_]*\.(?:rs|py|sh)$")
LANDING_NOT_APPARATUS = frozenset({"replay.sh"})
LANDING_PRODUCT_DIRECTORY = "research/results/"
FILE_EDITING_TOOLS = {"Edit", "Write", "MultiEdit"}
# ⑥ 写回员：规格目标不在时报告里要有的那一节（.claude/agents/kb-scribe.md），写回员自己起草的规格放草稿目录的这两个名字
SCRIBE_TARGET_CHECKED_AGENTS = {"kb-scribe"}
MISSING_TARGETS_SECTION = "找不到的目标"
SCRIBE_DRAFTED_SPECIFICATIONS = ("spec.json", "spec.md")
MARKDOWN_HEADING = re.compile(r"^ {0,3}(#{1,6})\s+(.*)$")
PROMPT_LABEL_LINE_START = r"^[ \t>*_\-]*"
SPECIFICATION_SUFFIXES = (".json", ".md", ".txt")
# ① 拒绝说明没写报告文件名的写法：只有弄坏开关 limit-hint-unnamed 下还在用
LIMIT_HINT_BEFORE = "→ 怎么办：全文写进报告文件（派发提示给的报告路径；没给就写进草稿目录），交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。"
# 弄坏开关只在自证里认（.claude/gate.d/break-switch-outside-selftest-pending.tsv 文件头的改法）：--selftest 与它起的子进程（带这份标记）认，平常跑设了就拒
SELFTEST_MARK_VARIABLE = "HANDBACK_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "handback-guard.sh --selftest 起的子进程"
# 判不了而放行的几处（文件头「这一道自己出错」一段）：各处 note_unjudged 交进来，放行时入口照「放行但必记检出」记一条检出
UNJUDGED = []
UNJUDGED_PREFIX = "交回正文闸没判全、照常放行"
UNJUDGED_SITES = ("input", "exception", "work-root", "leg-types", "leg-definitions", "citation-transcript", "cite-check", "green-report", "background-transcript", "background",
                  "rs-split", "naming-transcript", "project-code-library", "scratch-line", "scribe-transcript", "kb-spec", "spec-unreadable", "landing-transcript", "landing-split")


def note_unjudged(site, message):
    """判不了而放行的一处：stderr 照原样写 message，记进 UNJUDGED 等入口记检出。弄坏开关 HANDBACK_GUARD_BREAK=<site>-unrecorded 下只打 stderr。"""
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
    读不到那份库交空集、stderr 写一句；弄坏开关 leg-types-hardcoded 下用写死的四种。"""
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
    弄坏开关 leg-report-work-root-token-kept 下不换（② 认不出这种写法）。"""
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
    弄坏开关 leg-report-subdirectory-kept 下用这里自己的一份，leg-report-outside-root-kept 下仓外同名的也认成项目根那一份，
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
    """会话记录里的工具调用，按次序：[(工具名, 参数, 当前目录, 跑过没有)]。跑过：有工具结果，不是错误，或错误是「Exit code N」（跑了、退非 0）；
    被钩子拒的、超时的、没有结果的不算。"""
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

    def ran(result):
        if result is None:
            return False
        if result.get("is_error") is not True:
            return True
        text = result.get("content")
        if isinstance(text, list):
            text = "".join(str(part.get("text") or "") for part in text if isinstance(part, dict))
        return isinstance(text, str) and re.match(r"\s*Exit code \d+", text) is not None
    return [(name, arguments, cwd, ran(results.get(call_id))) for call_id, name, arguments, cwd in calls]


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
    主工作区（project_root）与带 .git 的副本（git worktree）判那棵树；没有 .git 的副本（rsync 拷的）照仓根判：交 (project_root, 判)，
    报告里两条的根要填仓根（共用约束「命名检查交主 agent 跑」：rsync 副本里改的 <根> 填仓根，主 agent 在补丁打进主工作区之后交）。
    弄坏开关 naming-any-file 下不看后缀，naming-rsync-copy-unjudged 下没有 .git 的副本不判。"""
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
    if BROKEN == "naming-rsync-copy-unjudged":
        return trees[-1], False
    return os.path.normpath(project_root), True


def naming_report_texts(records, message, project_root):
    """⑤ 读哪几份报告：派发提示里行首「报告：」点名的那几份；提示里没有这一行的，取交回正文点名的（REPORT_MENTION）。
    → ([(绝对路径, 正文；读不了是 None，按那一节没写算)], 认报告的说法)。"""
    named = prompt_label_paths(first_prompt(records), "报告")
    source = "派发提示「报告：」那一行"
    if not named:
        named = REPORT_MENTION.findall(work_root_tokens_expanded(message))
        source = "派发提示没有「报告：」一行，取交回正文点名的报告"
    texts = []
    for path in dict.fromkeys(named):
        absolute = resolved(path, project_root)
        try:
            with open(absolute, encoding="utf-8", errors="replace") as handle:
                texts.append((absolute, handle.read()))
        except OSError as error:
            print(f"! handback-guard：读不了报告 {absolute}（{error}），这一份的「{NAMING_COMMANDS_SECTION}」一节按没写算", file=sys.stderr)
            texts.append((absolute, None))
    return texts, source


def naming_report_commands(text, project_root):
    """⑤ 一份报告里「要主 agent 跑的命令」那几节写着的命名检查：{(哪一条, 根的真实路径)}。哪一条是 "stage"（门禁 code-source-discipline.sh 的 --check
    点名的格含 NAMING_CHECK_CELLS 那四格）或 "lint"（上游 naming-lint.sh）；根照 resolved 展开（~、$HOME、$SINGLEFS_WORK_ROOT 起头的照展开，相对路径从项目根起）。
    节的认法同 ⑥ ⑦（sections_titled）。弄坏开关 naming-report-section-unchecked 下整份报告都找（不看节），naming-report-root-unchecked 下根一律记成 NAMING_ANY_ROOT。"""
    body = text if BROKEN == "naming-report-section-unchecked" else sections_titled(text, NAMING_COMMANDS_SECTION)

    def root_of(word):
        return NAMING_ANY_ROOT if BROKEN == "naming-report-root-unchecked" else os.path.realpath(resolved(word.rstrip("/") or "/", project_root))
    found = {("stage", root_of(match.group("root"))) for match in NAMING_STAGE_IN_REPORT.finditer(body)
             if NAMING_CHECK_CELLS <= set(match.group("cells").split(","))}
    return found | {("lint", root_of(match.group("root"))) for match in NAMING_LINT_IN_REPORT.finditer(body)}


def naming_problems(hook_input, message, project_root, heavy):
    """⑤：改了 .rs（Edit / Write / MultiEdit，或 Bash 里 mv、cp、重定向、tee 写的，bash_rs_writes）的那几棵树里，报告（naming_report_texts）的
    「要主 agent 跑的命令」一节没把那棵树的两条命名检查都写上的 → ([(树, 最后改的文件, [缺的那几条])], [(读的报告, 读得了没有)], 认报告的说法)。
    只看会话记录里的改动与报告里写的命令，不看会话记录里跑没跑过。读不了会话记录交 ([], [], "")（记一处没判全）。"""
    records = own_transcript_records(hook_input)
    if records is None:
        note_unjudged("naming-transcript", "! handback-guard：找不到这个子 agent 自己的会话记录，命名检查这一条没判（放行）")
        return [], [], ""
    last_edit = {}
    for name, arguments, cwd, ran in tool_calls(records):
        if not ran:
            continue
        if name in FILE_EDITING_TOOLS:
            paths = [arguments.get("file_path")]
        elif name == "Bash":
            paths = bash_rs_writes(str(arguments.get("command") or ""), cwd, heavy, project_root)
        else:
            continue
        for path in paths:
            tree = naming_source_tree(path, project_root)
            if tree is not None and tree[1]:
                last_edit[tree[0]] = path
    if not last_edit:
        return [], [], ""
    texts, source = naming_report_texts(records, message, project_root)
    listed = set()
    for _path, text in texts:
        if text is not None:
            listed |= naming_report_commands(text, project_root)
    problems = []
    for tree, path in last_edit.items():
        root = os.path.realpath(tree)
        missing = [kind for kind in ("stage", "lint") if (kind, root) not in listed and (kind, NAMING_ANY_ROOT) not in listed]
        if missing:
            problems.append((tree, path, missing))
    return problems, [(path, text is not None) for path, text in texts], source


def naming_refusal(agent_type, problems, reports, source):
    def kind_name(kind):
        return "code-source-discipline.sh 那四格" if kind == "stage" else "naming-lint.sh"
    lines = [f"  {tree}：最后改的是 {edited}；缺 {'、'.join(kind_name(kind) for kind in missing)}（<根> 要填 {tree}）" for tree, edited, missing in problems]
    read = "；".join(f"{path}{'' if readable else '（读不了，按那一节没写算）'}" for path, readable in reports) or "一份都没有"
    commands = [f"    {command.format(root=tree)}" for tree, _, _ in problems for command in NAMING_SUBMIT_COMMANDS]
    return (f"✗ {agent_type} 改了 crates/ 或 scripts/ 下的 .rs（Edit / Write，或 Bash 里 mv、cp、重定向、tee 写的），报告里「{NAMING_COMMANDS_SECTION}」一节"
            "没把那棵树的两条命名检查写全（子 agent 不往统一任务服务交任务，命名检查由主 agent 交；共用约束 .claude/agent-common.md「门禁」一节「命名检查交主 agent 跑」）：\n"
            + "\n".join(lines[:10]) + f"\n  读的报告（{source}）：{read}"
            + f"\n→ 怎么办：在派发提示「报告：」点名的那份报告里加一节「## {NAMING_COMMANDS_SECTION}」，把下面两条原样写进去（<根> 已填好改动所在的树；"
            "在哪个目录起写仓根；跑完看末行 TASK_SERVER_RESULT，exit=0 是过、exit=1 是判红），子 agent 自己不交，写好再交回"
            "（.claude/hooks/handback-guard.sh 文件头 ⑤）：\n" + "\n".join(commands[:20]) +
            f"\n  在 rsync 拷的副本（没有 .git）里改的照仓根判：<根> 填仓根，主 agent 在补丁打进主工作区之后交。")


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
    """⑥ 本提示的草稿目录：行首「草稿目录：」一行点名的、工作根底下的目录（同目录 lib_prompt_scratch_line.py 的 scratch_directories，
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
    """写回员这一次的规格条目：[(规格出处, 条目)]，只收旧串、新串都在的。出处：提示点名的工作根或会话临时目录根下的 .json / .md / .txt（「报告：」那一份除外）、
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
    for name, arguments, cwd, ran in tool_calls(records):
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
    """⑦ 派发提示是不是只修锚点、只补落产物的那两种：行首标签与整句才认（LANDING_EXEMPT_LINE）。弄坏开关 landing-exemption-substring 下整份提示里有那几个字就算。"""
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
    for name, arguments, _, ran in tool_calls(records):
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
        heavy = project_code_runs_library()
        problems, reports, source = naming_problems(hook_input, message, project_root, heavy) if heavy is not None else ([], [], "")
        if problems:
            return 2, naming_refusal(agent_type, problems, reports, source)
    if agent_type in LANDING_CHECKED_AGENTS and BROKEN != "no-landing-checklist":
        problems = landing_problems(hook_input, message, project_root, project_code_runs_library())
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


PROJECT_CODE_RUNS_LIBRARY = []

def project_code_runs_library():
    """同目录的 lib_project_code_runs.py（它带着 lib_shell_words）；导入不了交 None，stderr 写一句（命名检查这一条不判）。"""
    if not PROJECT_CODE_RUNS_LIBRARY:
        try:
            import importlib.util
            spec = importlib.util.spec_from_file_location("lib_project_code_runs", os.path.join(sys.argv[1], "lib_project_code_runs.py"))
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            PROJECT_CODE_RUNS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败
            note_unjudged("project-code-library", f"! handback-guard：导入不了 lib_project_code_runs.py，命名检查这一条没判（放行）：{type(error).__name__}：{error}")
            PROJECT_CODE_RUNS_LIBRARY.append(None)
    return PROJECT_CODE_RUNS_LIBRARY[0]


def selftest(hook_dir):
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与会话临时目录根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
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
    scratch_reports = []   # ③ 放在工作根与会话临时目录根下的报告目录，自检的 TMPDIR 之外
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
        # ③ 的报告放工作根（不看弄坏开关）与会话临时目录根下各一份，交回里工作根那一份写成 ~ 起头；自己建、finally 里删
        home = os.path.expanduser("~")
        for scratch_root in (work_roots.configured_work_root(), work_roots.claude_session_temporary_root()):
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
            case("执行员会话临时目录根下的报告里绿行没数", "experiment-runner", f"报告 {os.path.join(scratch_reports[1], 'result.md')}" + landing_sample, 2),
            case("回扫员的绿行不判", "sweep", "✓ 通过", 0),
            case("交回时自己起的后台任务还在跑（项目子 agent）", "sweep", "报告 /tmp/x", 2, "a2"),
            case("交回时自己起的后台任务还在跑（通用 agent 也判）", "general-purpose", "交回", 2, "a2"),
            case("起过的后台任务都跑完了", "sweep", "交回", 0, "a3"),
            case("主 agent 不判后台任务", None, "交回", 0, None),
        ]
        # 三方腿类型表（拆分提案第九十节 ⑯）：② 读到的类型集合与登记处（定义 frontmatter 的 leg: three-way 一行）逐个相同。样本仓里的登记故意与写死的四种不同：
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

        # ⑤ 命名检查：样本树就是 work（项目根）；git worktree 副本带 .git 文件，rsync 副本不带。判的是派发提示「报告：」那一份里「要主 agent 跑的命令」一节写没写那两条
        def transcript(agent_id, records, prompt="# 派发"):
            with open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w", encoding="utf-8") as handle:
                handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": prompt}}, ensure_ascii=False) + "\n")
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
        done, refused = (False, "ok"), (True, "PreToolUse:Edit hook error: ✗ 拒")
        def edited(path, outcome=done):
            return ("Edit", {"file_path": path, "old_string": "a", "new_string": "b"}, outcome, work)
        def bash_call(command, outcome=done, cwd=work):
            return ("Bash", {"command": command}, outcome, cwd)
        naming_reports = os.path.join(work, "naming-reports")
        os.makedirs(naming_reports)
        def naming_report(name, body):
            """报告写进样本根下，交回派发提示（行首「报告：」点名它）。"""
            path = os.path.join(naming_reports, f"{name}.md")
            open(path, "w", encoding="utf-8").write(body)
            return f"# 派发\n\n报告：{path}\n"
        def naming_commands(root, stage_cells="vague-names,test-file-names,ordinal-names,data-names", lint=True):
            stage, lint_command = (command.format(root=root) for command in NAMING_SUBMIT_COMMANDS)
            stage = stage.replace("vague-names,test-file-names,ordinal-names,data-names", stage_cells)
            return f"1. `{stage}`\n   在哪个目录起：仓根\n" + (f"2. `{lint_command}`\n   在哪个目录起：仓根\n" if lint else "")
        listed_section = f"# 报告\n\n## 改了什么\n\n见下。\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(work) + "\n## 没做什么\n\n无。\n"
        core_source = os.path.join(work, "crates", "core", "src", "lib.rs")
        no_section_prompt = naming_report("no-section", "# 报告\n\n## 改了什么\n\n改了 crates/core/src/lib.rs。\n\n## 没做什么\n\n无。\n")
        listed_prompt = naming_report("listed", listed_section)
        transcript("naming-unlisted", [edited(core_source)], no_section_prompt)
        transcript("naming-listed", [edited(core_source)], listed_prompt)
        transcript("naming-one-listed", [edited(core_source)],
                   naming_report("one-listed", f"# 报告\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(work, lint=False)))
        transcript("naming-three-cells-listed", [edited(core_source)],
                   naming_report("three-cells", f"# 报告\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(work, "vague-names,test-file-names,ordinal-names")))
        transcript("naming-other-root", [edited(core_source)],
                   naming_report("other-root", f"# 报告\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(worktree)))
        transcript("naming-other-section", [edited(core_source)],
                   naming_report("other-section", "# 报告\n\n## 我跑过的\n\n" + naming_commands(work) + f"\n## {NAMING_COMMANDS_SECTION}\n\n无。\n"))
        transcript("naming-no-report", [edited(core_source)])
        transcript("naming-refused-edit", [edited(core_source, refused)], no_section_prompt)
        transcript("naming-markdown", [edited(os.path.join(work, "crates", "core", "README.md"))], no_section_prompt)
        # rsync 副本（没有 .git）里改的照仓根判：两条的根要填仓根（work），填副本根的不算
        transcript("naming-rsync-copy", [edited(os.path.join(copy, "crates", "core", "src", "lib.rs"))], no_section_prompt)
        transcript("naming-rsync-copy-listed", [edited(os.path.join(copy, "crates", "core", "src", "lib.rs"))], listed_prompt)
        transcript("naming-rsync-copy-root", [edited(os.path.join(copy, "crates", "core", "src", "lib.rs"))],
                   naming_report("rsync-copy-root", f"# 报告\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(copy)))
        transcript("naming-scripts", [edited(os.path.join(work, "scripts", "experiment-apparatus", "e1_probe.rs"))])
        transcript("naming-worktree", [edited(os.path.join(worktree, "crates", "core", "src", "lib.rs"))],
                   naming_report("worktree", f"# 报告\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(worktree)))
        # Bash 里写 .rs 也算改动（拆分提案第九十节 ㉕）：mv、cp 到目录、> 重定向、>>、tee；写到草稿目录的不算
        transcript("naming-bash-mv", [bash_call(f"mv {work}/draft-lib.rs crates/core/src/lib.rs")], no_section_prompt)
        transcript("naming-bash-cp-directory", [bash_call("cp patch/files/lib.rs crates/core/src/")], no_section_prompt)
        transcript("naming-bash-redirect", [bash_call("cat > crates/core/src/lib.rs <<'EOF'\nfn probe() {}\nEOF")], no_section_prompt)
        transcript("naming-bash-append", [bash_call(f"D={work}/crates/core/src; echo '// 补一行' >> $D/lib.rs")], no_section_prompt)
        transcript("naming-bash-tee", [bash_call("printf 'fn probe() {}\\n' | tee crates/core/src/lib.rs")], no_section_prompt)
        transcript("naming-bash-mv-listed", [bash_call(f"mv {work}/draft-lib.rs crates/core/src/lib.rs")], listed_prompt)
        transcript("naming-bash-scratch-copy", [bash_call(f"cp crates/core/src/lib.rs {work}/originals/lib.rs")], no_section_prompt)
        # 目标以命令替换起头的（拆分提案第一百二十二节 ⑪）：$(git rev-parse --show-toplevel) 按命令当时的目录往上找 .git 那一层展开，"$(pwd)" 按命令当时的目录展开
        transcript("naming-bash-substitution", [bash_call("cp draft-lib.rs $(git rev-parse --show-toplevel)/crates/core/src/lib.rs", done, worktree)], no_section_prompt)
        transcript("naming-bash-pwd-substitution", [bash_call("cat > \"$(pwd)/crates/core/src/lib.rs\" <<'EOF'\nfn probe() {}\nEOF")], no_section_prompt)
        transcript("naming-bash-substitution-listed", [bash_call("cp draft-lib.rs $(git rev-parse --show-toplevel)/crates/core/src/lib.rs", done, worktree)],
                   naming_report("substitution-listed", f"# 报告\n\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(worktree)))

        def naming_refusal_of(agent_id):
            return decide({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "implementation-writer", "agent_id": agent_id,
                           "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        unlisted_refusal, no_report_refusal = naming_refusal_of("naming-unlisted"), naming_refusal_of("naming-no-report")
        cases += [
            case("改了 .rs、报告里没有「要主 agent 跑的命令」一节拒", "implementation-writer", "交回", 2, "naming-unlisted"),
            case("改了 .rs、报告「要主 agent 跑的命令」一节写了两条命名检查（根填那棵树）放行", "implementation-writer", "交回", 0, "naming-listed"),
            case("改了 .rs、那一节只写了一条命名检查拒", "implementation-writer", "交回", 2, "naming-one-listed"),
            case("改了 .rs、那一节里四格点名少一格不算拒", "implementation-writer", "交回", 2, "naming-three-cells-listed"),
            case("改了 .rs、那一节两条的根填的是别的树拒", "tooling-writer", "交回", 2, "naming-other-root"),
            case("改了 .rs、两条写在报告别的节里拒", "implementation-writer", "交回", 2, "naming-other-section"),
            case("改了 .rs、派发提示没有「报告：」一行、交回也没点名报告拒", "implementation-writer", "交回", 2, "naming-no-report"),
            case("改 .rs 的 Edit 被钩子拒了不算改动放行", "implementation-writer", "交回", 0, "naming-refused-edit"),
            case("只改 .md 放行", "implementation-writer", "交回", 0, "naming-markdown"),
            case("rsync 副本里改了 .rs、报告没列两条命名检查拒", "implementation-writer", "交回", 2, "naming-rsync-copy"),
            case("rsync 副本里改了 .rs、那一节两条的根填仓根放行", "implementation-writer", "交回", 0, "naming-rsync-copy-listed"),
            case("rsync 副本里改了 .rs、那一节两条的根填副本根拒", "implementation-writer", "交回", 2, "naming-rsync-copy-root"),
            case("执行员改了 scripts/ 下 .rs、报告没写命名检查拒", "experiment-runner", "交回" + landing_sample, 2, "naming-scripts"),
            case("git worktree 副本里改的、那一节根写成副本根放行", "implementation-writer", "交回", 0, "naming-worktree"),
            case("回扫员改 .rs 不判命名检查", "sweep", "交回", 0, "naming-unlisted"),
            case("Bash 里 mv 写进 crates/ 的 .rs、报告没写命名检查拒", "implementation-writer", "交回", 2, "naming-bash-mv"),
            case("Bash 里 cp 到 crates/ 下的目录、来源是 .rs，报告没写命名检查拒", "tooling-writer", "交回", 2, "naming-bash-cp-directory"),
            case("Bash 里 > 重定向写 .rs、报告没写命名检查拒", "implementation-writer", "交回", 2, "naming-bash-redirect"),
            case("Bash 里 >> 追加写 .rs（目标在变量里）、报告没写命名检查拒", "implementation-writer", "交回", 2, "naming-bash-append"),
            case("Bash 里 tee 写 .rs、报告没写命名检查拒", "implementation-writer", "交回", 2, "naming-bash-tee"),
            case("Bash 里 mv 写 .rs 之后报告写了两条命名检查放行", "implementation-writer", "交回", 0, "naming-bash-mv-listed"),
            case("Bash 里把 .rs 拷到草稿目录不算改动放行", "implementation-writer", "交回", 0, "naming-bash-scratch-copy"),
            case("Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs、报告没写命名检查拒", "implementation-writer", "交回", 2, "naming-bash-substitution"),
            case("Bash 里 > 重定向写到 \"$(pwd)/crates/…rs\"、报告没写命名检查拒", "implementation-writer", "交回", 2, "naming-bash-pwd-substitution"),
            case("Bash 里 cp 写到 $(git rev-parse --show-toplevel)/crates/…rs 之后那一节根写成副本根放行", "implementation-writer", "交回", 0, "naming-bash-substitution-listed"),
            ("改了 .rs 报告没写的拒绝说明给出两条命令（根填改动所在的树）、写进「要主 agent 跑的命令」一节、不教自己交", True,
             all(part in unlisted_refusal for part in (f"ordinal-names,data-names {work}", f"naming-lint.sh {work}", f"## {NAMING_COMMANDS_SECTION}", "→ 怎么办",
                                                        os.path.join(naming_reports, "no-section.md")))
             and "task-server/task wait" not in unlisted_refusal),
            ("派发提示没有「报告：」一行的拒绝说明写明一份报告都没读到", True, "一份都没有" in no_report_refusal and "派发提示没有「报告：」一行" in no_report_refusal),
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
        # 装置改在 rsync 副本（landing_root/copy，没有 .git）里：⑤ 照仓根判命名检查，每份报告都带那一节、根填仓根（work），这里只判 ⑦
        landing_naming_section = f"\n## {NAMING_COMMANDS_SECTION}\n\n" + naming_commands(work)
        for name, body in landing_reports.items():
            open(os.path.join(landing_root, f"{name}-result.md"), "w", encoding="utf-8").write(body + landing_naming_section)
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
            saved_library, saved_directory = list(PROJECT_CODE_RUNS_LIBRARY), sys.argv[1]
            PROJECT_CODE_RUNS_LIBRARY.clear()
            sys.argv[1] = empty_hooks
            try:
                return project_code_runs_library()
            finally:
                PROJECT_CODE_RUNS_LIBRARY[:] = saved_library
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
            ("naming-transcript", "⑤ 找不到自己的会话记录", lambda: naming_problems({}, "交回", work, None)),
            ("project-code-library", "⑤ 导入不了 lib_project_code_runs.py", heavy_library_from_empty_directory),
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
            print("    → 看 decide()、citation_problems()、leg_reports_named()、green_count_problems()、naming_problems()、naming_report_commands()、"
                  "bash_rs_writes()、tool_calls()、naming_refusal()、landing_problems()、scribe_target_problems()、prompt_scratch_directories()、main() 的判法；"
                  "HANDBACK_GUARD_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ handback-guard 自检通过（查了 {len(cases)} 种）：子 agent 交回时自己起的后台任务还在跑拒（通用 agent 也判、主 agent 不判、说明里列出进程号与两条出路），"
              "跑完了或停掉之后放行；项目子 agent 超长交回拒（拒绝说明写明 result.md 与三方腿的 -output.md）、正好在上限放行、非项目 agent 与主 agent 不判；"
              "三方腿（类型读定义 frontmatter 的 leg: three-way，样本仓与本仓读到的都与 frontmatter 逐个相同）引文对不上或没点名报告拒、对上放行；书记员与执行员绿行没数或报 0 项拒（交回正文与点名的报告文件都查，报告在 research/prompts/、工作根（含 ~ 写法）与会话临时目录根下都认）；"
              "实现员、执行员、工具员改了 .rs 而派发提示「报告：」那一份的「要主 agent 跑的命令」一节没写全两条命名检查拒（没有那一节、只写一条、少一格、根填别的树、写在别的节、"
              "没有报告都拒，拒绝说明给出两条填好根的命令、不教自己交），写全了、只改 .md、Edit 被拒、worktree 副本根对上放行，rsync 副本里改的照仓根判（根填仓根放行、没列或填副本根拒）；Bash 里 mv、cp、>、>>、tee 写的 .rs 也算改动；"
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
    （弄坏开关 exception-unrecorded 下照原样抛出）。"""
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "agents", "cite-check.py")
    if work_roots is None:
        note_unjudged("work-root", f"! handback-guard 读不到 {os.path.join(hook_dir, 'lib_work_root.py')}（{work_roots_error!r}）：工作根与会话临时目录根下的报告这一次没判")
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
