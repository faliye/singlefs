# d137-gov-r3：一百三十七节那一轮的治理文件改动——第三轮（最终轮，2026-10-31）

## 一、题面

前两轮判决：`research/prompts/d137-gov-r1-main-verification.md`、`research/prompts/d137-gov-r2-main-verification.md`。第二轮判决「三、怎么办」列的修补都已交回，各件的报告与只含自己改动的 diff 在 `~/.local/share/singlefs/work/` 下：`d137-r2fix-hooks/`、`d137-r2fix-dispatch/`、`d137-r2fix-watch/`、`d137-r2fix-registry-2/`、`d137-r2fix-defs/`，各目录的 `result.md` 与 `own.diff`。另有这一轮用户新定、新写的三件：`d137-queue-order/`（内存包装排队不限时、先来后到带保守回填）、`d137-dispatch-route/`（子 agent 排不上不绕开、只读仓树的经 `scripts/hosts/host-dispatch.py`；主 agent 重活两台按资源均衡）、`d137-hook-launcher/`（重型测试闸剥开 `host-dispatch.py` 与 `multi-host-run.sh … --` 两种起法照里面判）。复跑分类（`d137-replay-classify/`）判出的误拒与漏拒，又修了两件：`d137-shellwords-misrefuse/`（共用切词与检出钩子：E1、E6、E7、E8、E9、只定义不调用的 POSIX 函数、`builtin coproc`）、`d137-guard-misrefuse/`（重型测试闸：E2、E3、E5、E4、漏拒 L1、`$CLAUDE_CODE_EXECPATH` 两道口径与两处说明）。

这是第三轮，也是最终轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第三轮分工：前两轮「已定」的形态撑不撑得住、未定项清单完不完整、报出的数能不能核。第三轮之后不再开第四轮。

用户在这一轮定的口径同前两轮正文 `research/prompts/_d137-gov-r1-body.md`、`_d137-gov-r2-body.md`「一、题面」，另加（逐字见调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md`）：
- 钩子的漏拒里那几处要构造绕过写法的（第二轮判决 Y014、Y029、Z021 与变量拼命令词的两处）「登账留你亲自处理」：这一轮不攻、不复现、不造新的绕法样本，只攻不涉及写法的格。
- 「退 2 一律照拒」。
- 内存包装：「排队中的脚本永远不超时 一直等待除非被取消」；排队「不限时 + 先来后到带保守回填」；子 agent 排不上「取消绕开，排不上就经 host-dispatch.py 去第二台」；两台「我定的是均分。 那边有资源取那边 不是必须要去那边。两台机器环境基本一致 因此均衡的跑比较好」。

被判的文件（快照 `research/prompts/d137-gov-r3-snapshot/files-sha256.txt`）：
- 钩子：`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/bash-command-detector.sh`、`.claude/hooks/lib_shell_words.py`、`.claude/hooks/runner-dispatch-guard.sh`（前两份上别的会话另有没提交的「起编译不经内存包装」一道，不归这一轮）。
- 定义与共用约束：`.claude/agent-common.md`、`.claude/main-agent.md`。
- 研究脚本：`scripts/agents/agent-watch.py`、`scripts/hosts/core_usage.py`、`scripts/hosts/detached-run.sh`、`scripts/hosts/run-with-memory-cap.sh`、`scripts/mutation/crates-mutation-rows.py`、`scripts/agents/dispatch-prompt.py`、`scripts/gate/verus-proof-check.py`、`.claude/gate.d/checker-tier-formal-verification.sh`。
- 登记表：`.claude/gate.d/break-switch-replay-files.tsv`。

## 二、实现今天的样子（主 agent 的观测，2026-10-31）

- `crates/` 里没有相关实现：`grep -rln 'host-dispatch\|declaration_excuses_low\|queue_entries\|claude_session_temporary_root' crates/` 零命中。方案按钩子、定义与脚本今天的代码来谈。
- 主 agent 用当时的钩子（件 13 换上之前、件 15 与件 16 之前；与那时主工作区五份钩子 `cmp` 逐份相同）重跑了早先四条腿的装置与真实命令语料（量的）：第二轮腿甲 rc=0；第三轮腿甲 rc=0、第 8 条少了 2 条误拒；第三轮腿乙全量 2710 条里 294 条与登记不同；第二轮腿乙 30 条与登记不同、与上一趟逐条相同；语料 30,595 条新拒 326 条、随机样本 5000 条里 16 条。输出在 `~/.local/share/singlefs/work/d137-hooks/main-replays-4/` 与 `~/.local/share/singlefs/work/d137-r3-prep/real-work/out/`；逐条分类在 `~/.local/share/singlefs/work/d137-replay-classify/result.md`（没有「腿登记该拒、钩子放」的回退；误拒九类与零星两类、漏拒一条，都已由件 15、件 16 修）。复跑装置的三处读法问题（legB 桩把外层内存包装算进样本、compare 只打前 30、rerun.sh 的 diff 只看前 12 行）主 agent 已修。用最终钩子再跑一遍要等统一任务启动器（用户通知新的脚本任务先不起），读数到了补进判决。
- 登记表里各份钩子的开关条数照各件在主工作区逐个跑过的读数（各件报告）：heavy-test-guard 79 条（别的会话新加的三条没复跑）、lib_heavy_tests 74 条（别的会话两条没复跑）、bash-command-detector 118 条、core_usage 22 条。`lib_shell_words.py` 没有自己的 `--selftest`，主 agent 从登记表删了它那一行。
- 主 agent 跑 `rules-lint.sh` 对两份定义：红 1 处在 `.claude/main-agent.md` 第 23 行「用户原话」，那一行与 HEAD 逐字相同。
- 别的会话没提交的改动在被判文件里：只判这一轮的改动（各件 `own.diff`）。

## 三、格

### R1　论证前两轮的结论与判定过程（辩方腿）
前两轮判决每一条「打中，采」「采为线索」「撑住」：判得够不够得着、修补之后还站不站得住、是不是同样打中所有替代判法；前两轮的判定过程本身（主 agent 现喂多少条、哪几条只采为线索没现喂、复跑抽样的范围）漏没漏。替被判出局或被改掉的一方辩护：修补里哪一处过度，把正当命令拒掉了（引 `d137-replay-classify` 的分类核）。不攻钩子漏拒那一半的写法。

### R2　已定撑不撑得住、未定清单完不完整、报出的数能不能核（攻方腿）
- 已定撑不撑得住：前两轮站住、修补里落定的形态——看门狗的声明判法与单元归属（A1、W1、W2、U1）、派发闸退 2 一律拒与 `sha256=refused` 放行记检出（H12、D1、D2）、串行声明的预计秒数（M6 与 verus-proof-check 那一处）、`.claude/main-agent.md` 三种处置一行只归一种（C1）、开关复跑表的登记（M3、M4）。每一处在并发、负载、时钟或输入边角下判错的地方。
- 未定清单完不完整：调度记录里写「登进收尾那一节」「留用户亲自处理」「下一轮」的条目，与前两轮判决「采为线索」而没现喂的条目，有没有漏的、有没有被修补悄悄改了性质的。
- 报出的数能不能核：各件报告与调度记录里的数（开关条数 79 / 74 / 118 / 22 与它们「逐个跑过」的范围、自证项数 161 / 187 / 119 与重型闸「查了 2502 种」、复跑差 294 / 38 与分类总表、语料 30,595 / 326 / 16、排队 592 秒），量的范围撑不撑得住那句话，能不能用一条命令数出来。
- 这一轮新写的五件（零轮）：排队判法（`run-with-memory-cap.sh` 的队列登记、保守回填、死条认法、报行）、共用约束「内存包装排不上」一条与主 agent 定义的均衡句与「禁止」一节新加的一条、重型测试闸剥开两种起法、共用切词与检出钩子的误拒修补、重型测试闸的误拒与漏拒修补。按用户口径核它们做的是不是用户定的那件事、有没有自相矛盾、有没有把别的规矩改坏；误拒修补那两件只核「放开的写法里有没有真起重型的」用分类报告里已有的同形写法，不造新的绕法。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿 | R1 | 论证前两轮的结论与判定过程；替被修掉的一方辩护，找过度修补 |
| 攻方腿 | R2 | 攻已定的形态、未定清单的完整性、报出的数；攻这一轮新写的三件；不重复辩方腿，不攻钩子漏拒的写法 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写被判文件在主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 喂钩子在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里拷一份钩子喂，cwd 放草稿目录，不在主工作区里改任何文件；喂法与原样输出写进报告。不造新的绕过写法、不复现第二轮判决 Y014、Y029、Z021 与变量拼命令词那两处。
- 起真 shell 的判法在脚本里用 `setsid --wait` 起每条样本，超时与结束在脚本里杀整组；交回前 `ps` 核自己起的进程一个不剩。
- 不跑重型测试、不起 cargo、不碰 GPU；自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，排不上就在队里等（不设等待上限），不改成 `ulimit -v`、`nice` 直跑。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
