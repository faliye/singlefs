# d137-ts-defs-r1：统一任务服务那两批定义改动——第一轮（2026-10-31）

## 一、题面

一百三十七节那一轮的三方（`d137-gov-r1`、`-r2`、`-r3`）已判完，不开第四轮。第三轮开工快照（`research/prompts/d137-gov-r3-snapshot/files-sha256.txt`）之后，用户定下统一任务服务（`task-server/`），定义侧又改了两批，三轮里一轮都没攻过，另立这一题：

- 件 17（`~/.local/share/singlefs/work/d137-no-subagent-scripts/`）：子 agent 不起任何脚本；其中与件 18 冲突的句子已被件 18 改掉。
- 件 18：定义、共用约束、规则、skill 改成经 `task-server/task submit` 交、后台 `task-server/task wait <号>` 等，删内存包装、线程、上限、挑台、分片、长活声明、看门狗管任务进程的告警。口径规格 `~/.local/share/singlefs/work/d137-taskserver-spec/spec.md`；五件报告 `d137-ts-common/`、`d137-ts-agents/`、`d137-ts-rules/`、`d137-ts-followup/`（各目录 `result.md`，前四件附 diff）；skill 四份与后来三处（`.claude/main-agent.md` 写与验分两段的轻型句、`.claude/agent-common.md` 被拒后自己改、`.claude/agents/kb-scribe.md` 喂写范围闸、`.claude/agents/implementation-writer.md` 一处条名、`.claude/skills/crash-test/commit-stages.md` 分片抄行）由主 agent 改，改法逐条记在调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md`。

用户定的口径（逐字见 `records/2026-10-30-编译绕过内存总上限.md` 第 205 行与调度记录）：
- 「说白了，就是把以前分散到各个任务中的脚本启动和判定 统一收起来。 统一管理。」
- M01「非重型自己交，重型交回」；M03「全删，整条交服务」；M06「服务一处全拒」。
- 写与验分两段与 M01 冲突那几句：「子agent 可以自己写。但是必须要经过 task-server」。
- 「如果被拒绝了 agent 和 子agent 需要自己修改自己的脚本  修改为合适的不违规的」。
- 这一题：「你可以攻 但是这些是已经成立的方式， 你的攻击点主要是找到漏洞」——做法本身（经服务交、服务定资源）不重议，攻的是写出来的定义有没有漏洞。
- 没定、定义里写成不表态的：M05（线程能不能覆盖）、M07（重型还要不要 `SINGLEFS_HEAVY_TESTS` 前缀）、M09（GPU 一件一张卡还是全部卡）。

被判的文件（快照 `research/prompts/d137-ts-defs-r1-snapshot/files-sha256.txt`）：
- 定义与共用约束：`.claude/agent-common.md`、`.claude/main-agent.md`，`.claude/agents/` 下 clerk、experiment-designer、experiment-runner、implementation-writer、investigator、kb-scribe、mutation-triage、three-way-attack、three-way-defense、three-way-forward、tooling-writer 共 11 份。
- 规则：`.claude/rules/implementation-workflow.md`、`verification.md`、`three-way-inference.md`、`test-script-discipline.md`。
- skill：`.claude/skills/gate/SKILL.md`、`gate/staged-gate.md`、`crash-test/SKILL.md`、`crash-test/commit-stages.md`。
- 对照（不判，只读来核「写的与此刻判的、收的对不对得上」）：钩子 `.claude/hooks/heavy-test-guard.sh`、`lib_heavy_tests.py`、`bash-command-detector.sh`、`handback-guard.sh`、`runner-dispatch-guard.sh`；服务 `task-server/README.md`、`task-server/task`、`task-server/task_server_lib.py` 文件头（「错误类型」「分片」「挑机器」）。

## 二、实现今天的样子（主 agent 的观测，2026-10-31）

- `crates/` 里没有相关实现：`grep -rln 'task-server\|task_server' crates/` 零命中。方案按定义、钩子与服务今天的文字来谈。
- 这一批相对 HEAD：`git diff HEAD --stat -- .claude/agent-common.md .claude/main-agent.md .claude/agents/ .claude/rules/ .claude/skills/` 末行「23 files changed, 204 insertions(+), 227 deletions(-)」（含件 17 与这一轮早先件 12 的改动；只判件 17、件 18 与后来几处，早先的已在 `d137-gov-r1`–`r3` 判过）。
- 对照文件此刻的 sha256 前 8 位：`heavy-test-guard.sh` cc9fd871、`lib_heavy_tests.py` d3e72828（别的会话要改它 prove-red 那一处，换上要等它那边放行）、`bash-command-detector.sh` ef9d5957、`handback-guard.sh` 0c06912c、`runner-dispatch-guard.sh` 89584bcb、`task-server/task` f351d875、`task_server_lib.py` 34dd46ae、`README.md` 89b2e7ee。
- 经服务跑过的两条检查：rules-lint 只红 `CLAUDE.md:110` 两处与 `.claude/main-agent.md:24`「用户原话」，都是 HEAD 里就在的行；doc-lint 在第二台的红全是第二台暂存区与本机不同造成的假红，这一批文件零命中。
- 主 agent 这一会话自己撞到的一次：命令词放在变量里（`R='python3 …'; $R …`）被重型测试闸拒「认不出起的是什么」，照出路改成写全命令；只读命令里夹一条本仓脚本（`scripts/agents/three-way-materials.py --help`）整条被拒，拆开后只读的直接跑。

## 三、格

### F　定义写的与钩子、服务此刻做的对不对得上（正推腿）
逐份核：每一处起法（submit 的写法、`--cwd`、范围声明与前缀写在哪、白名单外的环境变量、`task wait` 末行怎么读、`task clean`）、每一处「直接跑」的名单、每一处「服务当场拒」的清单，与对照文件此刻的文字逐字比；M01 落进各份是不是同一个意思（谁交非重型、谁交重型、交回写什么）；条名改了之后各份引的名字是不是都对得上；M05、M07、M09 有没有被哪一份悄悄定了。

### A　找漏洞（攻方腿）
做法本身不攻。攻写出来的定义：照着定义做会撞拒、会漏判、会互相矛盾、会让重型被子 agent 交出去、会让任务结局被读错、会让编译目录留下、会让被拒之后绕开而不是改成合规写法的地方；定义没覆盖到、照字面做会出错的边角（例：子 agent 在副本里交、`--cwd` 指草稿目录、喂钩子、分片任务的结局、服务不通时）。每一处给出照定义原文做的那一步与它撞上的钩子或服务行号。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿 | F | 从定义原文推到「照做会发生什么」，与钩子、服务的文字逐处对 |
| 攻方腿 | A | 找漏洞；不重复正推腿的逐份对照，不攻做法本身，不造绕过钩子的写法 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、钩子、服务写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 不跑重型测试、不起 cargo、不碰 GPU；喂钩子、自证与探针这类非重型的经 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 交、run_in_background 起 `task-server/task wait <号>` 等结局，命令里不套包装、不写上限与线程；被拒了照出路改成合规写法再交；重型的写进报告「要主 agent 跑的命令」。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
