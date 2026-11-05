# d137-ts-defs-r1 正推腿报告（Sonnet，格 F）

结论一句：21 份被判文件里，起法的主干（submit 写法、`--cwd`、范围声明与前缀写在 submit 前面、白名单外变量写成 `-- env`、`task wait` 末行的 `exit=`、直接跑名单、服务当场拒清单）与服务、钩子此刻的文字对得上，四个条名也都对得上；对不上的有 18 项（F1–F18）：判「对不上」的 11 项里最硬的是 F1（experiment-runner、tooling-writer 还写命名检查交主 agent，与共用约束和交回闸相反）、F2（`rerun-failed-tests.py` 打出的命令已带 submit，三份定义还让再插一次）、F3（`build=` 是 `target` 的上一层）、F4（共用约束让子 agent 直接跑 `proc.py`，钩子拒）、F10（主 agent 的十一处本仓脚本命令没带 submit，钩子拒）；M05 被各份定成「提交时不写线程、一律登记值」，M07 被悄悄定成「要前缀」（钩子文字写「不要」），M09 没被定。

## 零、版本与方法

### 0.1 被判文件与对照文件此刻的 sha256 前 8 位

快照 `research/prompts/d137-ts-defs-r1-snapshot/files-sha256.txt` 共 30 行（含正文本身），开工时逐份比全对上；我做事的中途三份变了：`.claude/agent-common.md`（快照 4f111ee5，此刻 8858e562）、`.claude/skills/crash-test/commit-stages.md`（快照 21973c2f，此刻 7f32fae5）、`.claude/hooks/lib_heavy_tests.py`（快照 d3e72828，此刻 cc11e40f；正文已说别的会话要换它）。agent-common 我核到变的是第 74 行（「被钩子或服务拒了…」那一句，现在多了「先分清是哪一种」与「是机制或门禁本身判错的，修那道机制或门禁」两截，与用户 2026-10-31 的话对得上，出处 `records/2026-10-29-一百三十七节那一轮调度记录.md:224`）；commit-stages 我没逐行找变在哪，只核了我引的几行（18、19、22、26、27）变前变后同一行同一句；lib_heavy_tests 我只引一张表的两行（2773、2775），是变后再取的。所有引文行都是变后再取的，最后跑了 `cite-check.py`。其余 27 行此刻与快照一致。

| 文件 | sha8 此刻 | 与快照 |
|---|---|---|
| .claude/agent-common.md | 8858e562 | 对不上（快照 4f111ee5） |
| .claude/main-agent.md | 41e4163e | 一致 |
| .claude/agents/clerk.md | 13d1c6eb | 一致 |
| .claude/agents/experiment-designer.md | 0c0062b1 | 一致 |
| .claude/agents/experiment-runner.md | 59d3f26f | 一致 |
| .claude/agents/implementation-writer.md | b0466a2f | 一致 |
| .claude/agents/investigator.md | 9361c3aa | 一致 |
| .claude/agents/kb-scribe.md | da237b44 | 一致 |
| .claude/agents/mutation-triage.md | 68715195 | 一致 |
| .claude/agents/three-way-attack.md | 9d5aa05a | 一致 |
| .claude/agents/three-way-defense.md | c6d943ec | 一致 |
| .claude/agents/three-way-forward.md | ab6d86cf | 一致 |
| .claude/agents/tooling-writer.md | 4fc13d5e | 一致 |
| .claude/rules/implementation-workflow.md | e24bbf10 | 一致 |
| .claude/rules/verification.md | 2687ec24 | 一致 |
| .claude/rules/three-way-inference.md | 3a83bad7 | 一致 |
| .claude/rules/test-script-discipline.md | 9d3fe69c | 一致 |
| .claude/skills/gate/SKILL.md | 0f75ded1 | 一致 |
| .claude/skills/gate/staged-gate.md | fc5a89f5 | 一致 |
| .claude/skills/crash-test/SKILL.md | 0a05eb51 | 一致 |
| .claude/skills/crash-test/commit-stages.md | 7f32fae5 | 对不上（快照 21973c2f） |
| .claude/hooks/heavy-test-guard.sh | cc9fd871 | 一致 |
| .claude/hooks/lib_heavy_tests.py | cc11e40f | 对不上（快照 d3e72828） |
| .claude/hooks/bash-command-detector.sh | ef9d5957 | 一致 |
| .claude/hooks/handback-guard.sh | 0c06912c | 一致 |
| .claude/hooks/runner-dispatch-guard.sh | 89584bcb | 一致 |
| task-server/README.md | 89b2e7ee | 一致 |
| task-server/task | f351d875 | 一致 |
| task-server/task_server_lib.py | 34dd46ae | 一致 |

对照之外我另读了七份（不在快照里，只当「此刻的文字」读，sha8 此刻）：`task-server/task_server.py` d799e304、`scripts/gate/rerun-failed-tests.py` 92c6fa6e、`scripts/mutation/broad-mutation.py` e2002a3b、`scripts/gate/merge-and-verify.sh` 349a50fc、`scripts/mutation/mutate.sh` 9a1a0c18、`scripts/mutation/prove-red.sh` 86691c21、`scripts/crash/crash-point-replay.sh` 7cce2cc8。

### 0.2 方法与实现今天的样子

- 方案按 `crates/` 今天的实现来谈：正文「二」写了 `crates/` 里没有相关实现（`grep -rln 'task-server\|task_server' crates/` 零命中，主 agent 的观测）；我没有另核这条，只按定义、钩子与服务今天的文字谈。
- 引文规则：行号都是我这一次 `grep -nF` / 脚本在被引文件里现取的；过长的行（几百到几千字）我只抄与对照点有关的那一整句（句首到句尾逐字），句中被「」隔开的地方用「…」省略，不改字。
- 读过的规则：`.claude/rules/three-way-inference.md` 的「各条腿必须互不重复」、`.claude/rules/implementation-first.md` 的「规矩」、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`（开头三步表、「别的项目怎么做，是线索不是证据」、「不许先有结论再建模型」）。
- 实测：把「照定义原文写出的命令」喂给 `.claude/hooks/heavy-test-guard.sh`（与服务无关，钩子自己不起命令），脚本在草稿目录 `feed.py`、`feed2.py`，经 `task-server/task submit --cwd <草稿目录> -- python3 <脚本>` 交，结局见第五节。

## 一、问题 1：起法、「直接跑」名单、「服务当场拒」清单，逐处对照

### 1.1 对得上的（定义侧与对照侧此刻的文字逐字比过）

1. **submit 写法与 `--cwd`**。定义：「范围声明与重型前缀写在 submit 前面：`SINGLEFS_RUN_SCOPE='…' [SINGLEFS_HEAVY_TESTS=commit|user-request] task-server/task submit -- <原命令>`（白名单环境变量照带；白名单外的写成 `-- env <名>=<值> <命令…>`）」（`.claude/agent-common.md:59`）。对照：「task-server/task submit [--cwd <目录>] -- <命令…>   排进队、当场打任务号」（`task-server/task:4`）、「工作目录不给取此刻的目录」（`task-server/task:5`）。对得上；定义里「不给 `--cwd` 取此刻的目录」（同 `.claude/agent-common.md:59`）与客户端缺省一致。
2. **范围声明与前缀写在 submit 前面**。对照：「task-server/task submit -- <命令…>                 # 打出任务号；环境里的 SINGLEFS_RUN_SCOPE、SINGLEFS_HEAVY_TESTS 这类白名单变量照带」（`task-server/README.md:28`）。我在 21 份里 `grep -noE 'task-server/task submit[^`]{0,40}SINGLEFS_RUN_SCOPE'` 零命中，没有一处把声明写在 submit 后面。对得上。
3. **白名单外的环境变量**。对照：「白名单外的环境变量当场拒并点名；命令真要它的写成 `task-server/task submit -- env <名>=<值> <命令…>`。」（`task-server/README.md:36`）。定义里的实例：「再交 `task-server/task submit -- env AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash -c 'bash .claude/hooks/write-guard.sh < <草稿目录>/precheck-<n>.json'`、run_in_background 起 `task-server/task wait <号>` 等结局」（`.claude/agents/kb-scribe.md:31`）；白名单内的实例：「要留测试的临时现场，在 submit 前面设 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`（服务白名单里的变量，照带）」（`.claude/main-agent.md:31`）对「"SINGLEFS_RUN_SCOPE", "SINGLEFS_HEAVY_TESTS", "SINGLEFS_WORK_ROOT", MULTI_HOST_CONFIG_VARIABLE, "SINGLEFS_KEEP_TEST_TEMPORARY_FILES",」（`task-server/task_server_lib.py:534`）。对得上（kb-scribe 那一句钩子放不放行见第五节 C14）。
4. **子 agent 后台只起 `task wait`**。定义：「⑩ 子 agent 用 run_in_background 起命令：只许起 `task-server/task wait <号>`（整条就是这一句，前面最多一个 `cd <目录> &&`，后面不接 `&&`、`;`、管道、重定向）」（`.claude/agent-common.md:85`）。对照：「# ⑩ 子 agent（输入里有 agent_type）用 run_in_background 起命令：只许起 `task-server/task wait <号>`（整条就是这一句，前面最多一个 `cd <目录> &&`，」（`.claude/hooks/bash-command-detector.sh:49`）。对得上。（我自己开工后第一次后台起 `cd … ; task-server/task wait` 就被它拒了，换成 `&&` 过：实测，非推。）
5. **命名检查的结局读法**。定义：「各起 `task-server/task wait <号>` 等到末行 `TASK_SERVER_RESULT`（`exit=0` 是过，`exit=1` 是判红，别的码不算跑完）」（`.claude/agent-common.md:104`）。对照：「#      而会话记录里（之后的工具结果、读输出文件、后台任务的完成通知、别的 user 消息）有这个号的结局行 `TASK_SERVER_RESULT task=<号> type=<类型> exit=<码>`、码是 0 或 1，」（`.claude/hooks/handback-guard.sh:63`）。对得上。
6. **`task wait` 末行本身**。对照：「末行「TASK_SERVER_RESULT task=<号> type=<错误类型> exit=<码> attempts=<次数> log=<日志路径>」（`task-server/task:8`）；定义「任务完成时它退出、末行 `TASK_SERVER_RESULT …`、退出码是任务的」（`.claude/agent-common.md:70`）。对得上。
7. **`task clean` 的触发**。对照：「    if build:   # 编译目录留给提交它的 agent：用完 task-server/task clean <号> 删」（`task-server/task:157`）与「task-server/task clean <号>                         # 删这条任务留下的编译目录（只许提交它的会话、有了结局才删）」（`task-server/README.md:33`）；定义侧各份「`task wait` 末行带 `build=` 的，用完 `task-server/task clean <号>` 删它的编译目录」的出处见 1.3。触发条件对得上；`build=` 指哪一层目录对不上，见 F3。
8. **服务当场拒：包装前缀与资源参数**。定义：「交给服务的命令不带包装前缀（`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh`、`nice`、`timeout`、`run-with-memory-cap.sh`、`host-dispatch.py`、`multi-host-run.sh`、`detached-run.sh`），也不写脚本自己的上限、线程与挑台参数（`--memory`、`--memory-cap`、`--checker-quick-memory`、`--threads`、`--peer`、`--local-only`），写了服务当场拒」（`.claude/agent-common.md:59`）。对照：「REFUSED_PREFIXES = {"nice": "优先级", "timeout": "限时", "capped.sh": "线程", "two-logs.sh": "日志", "gpu-quiesce-run.sh": "清场与挑卡"}」（`task-server/task_server_lib.py:559`）、「RESOURCE_ARGUMENTS = ("--memory", "--memory-cap", "--checker-quick-memory", "--threads", "--peer", "--local-only")」（`task-server/task_server_lib.py:560`），`run-with-memory-cap.sh`、`host-dispatch.py`、`multi-host-run.sh`、`detached-run.sh` 在同文件 `NESTED_DISPATCH_SCRIPTS`（我读过 `task_server_lib.py` 的 `nested_dispatch`（第 1034 行起到 1063 行））。对得上；定义清单比服务少一样（`SINGLEFS_THREAD_CAP`），见 F13。
9. **直接跑名单**。定义：「直接跑、不经服务的只有：读文件；改文件（Edit / Write，与定义点名的写入工具 `scripts/kb/replace-once.py`、`replace-batch.py`、`insert-row.py`、`relabel-item.py`、`scripts/agents/copy-originals.py`）；只读的 git 命令」（`.claude/agent-common.md:59`，整句更长，我比的是下面三句）。对照：「直接跑的只剩：不是本仓文件的命令（只读的 git、ls、grep、cat、sed -n、wc、sha256sum、diff、du、ps 这类与草稿里自己写的、里面不跑项目代码的脚本）、语法检查（bash -n、python3 -m py_compile）、」（`.claude/hooks/heavy-test-guard.sh:317`）；「stale-candidates.py、kb-sections.py、checklist-specs.py、cite-check.py、claim-experiment.sh、gate-overlap.py --list，主 agent 定的 relabel-item.py、copy-originals.py，」（`.claude/hooks/heavy-test-guard.sh:319`）；「加 task-server/task 的 submit / wait / status / cancel / log / clean 与 task-server/control check），与只给主 agent 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY（proc.py、remind.sh、watch.sh、agent-watch.py、dispatch-prompt.py、detached-run.sh 的 list / watch / stop、」（`.claude/hooks/heavy-test-guard.sh:320`）。逐项对：`kb-spec-check.py`、`stale-candidates.py`、`kb-sections.py`、`checklist-specs.py`、`claim-experiment.sh`、`cite-check.py`、`gate-overlap.py --list`、`work-directory-ownership.sh claim`、`cargo fmt`（钩子 `fmt` 不编直接跑，`.claude/hooks/heavy-test-guard.sh:311`）、语法检查，定义列的每一样都在钩子的放行名单里。对得上；钩子多放一样（`task-server/control check`），定义没写，不冲突。**名单之外定义却让人直接跑的**（`proc.py`、主 agent 的若干脚本）见 F4、F10。

### 1.2 对不上的（逐处：两边原文、结论、什么现象会推翻它）

**F1　命名检查交谁（问题 1、2、3 都碰）——对不上，三处**

- 定义侧（旧说法还在）：「命名检查同样交主 agent」（`.claude/agents/experiment-runner.md:42`）；「命名检查也交主 agent」（`.claude/agents/experiment-runner.md:59`）；「命名检查同样交主 agent」（`.claude/agents/tooling-writer.md:35`）。
- 对照侧：「改了代码或实验源码（`crates/`、`scripts/` 下的 `.rs`）的子 agent（`implementation-writer`、`experiment-runner`、`tooling-writer`）交回前，在改动所在的树上自己经服务交两条命令」（`.claude/agent-common.md:104`）；钩子认的类型：「NAMING_CHECKED_AGENTS = {"implementation-writer", "experiment-runner", "tooling-writer"}」（`.claude/hooks/handback-guard.sh:485`）。
- 结论：对不上。这两份定义让子 agent 把命名检查「交主 agent」，共用约束与交回闸要它们自己交回前跑、跑不到就拒交回；照这两份做会撞交回闸。
- 推翻它的现象：交回闸对 experiment-runner、tooling-writer 不判命名检查（那一行集合里没有它们），或 `.claude/agent-common.md:104` 把这两类划出去。我读到的都不是这样。
- 对照侧的旧条名（不判，只报）：钩子注释里还写「共用约束「门禁」一节「命名检查交主 agent 跑」」（`.claude/hooks/handback-guard.sh:33`）与「交回前自查命名放行（.claude/agent-common.md 的交回前自查）」（`.claude/hooks/heavy-test-guard.sh:96`），现行条名是「命名检查交回前跑」。

**F2　`rerun-failed-tests.py` 打出的命令已经带 `task-server/task submit --`，三份定义还让人再插一次——对不上**

- 定义侧：「`scripts/gate/rerun-failed-tests.py` 打出的命令开头已带声明，`task-server/task submit --` 插在声明与 `cargo` 之间」（`.claude/agent-common.md:59`）；「照 1b 把 `task-server/task submit --` 插在声明与 `cargo` 之间再交」（`.claude/agents/implementation-writer.md:33`）；「声明在前，`task-server/task submit --` 插在声明与 `cargo` 之间」（`.claude/main-agent.md:55`）；措辞偏松的一份：「逐条经 `task-server/task submit -- <命令>` 交（开头的范围声明挪到 submit 前面）」（`.claude/rules/verification.md:26`）。
- 对照侧（脚本此刻的文字，sha8 92c6fa6e）：「打印出来的命令交给调用方原样经 Bash 起：每条都是 `SINGLEFS_RUN_SCOPE='…' task-server/task submit -- cargo test …`，经 task-server 跑」（`scripts/gate/rerun-failed-tests.py:9`）；「SUBMIT_WORDS = ["task-server/task", "submit", "--"]   # 打出的命令经提交客户端交 task-server（文件头）」（`scripts/gate/rerun-failed-tests.py:48`）。
- 实测 R1（任务 1792，`task-server/task submit --cwd <草稿目录> -- python3 scripts/gate/rerun-failed-tests.py <含一条失败用例的假日志> -p singlefs-harness`，`task log 1792 --all` 原样两行）：
  `✓ 1 条失败的用例，分在 1 个测试目标里`
  `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness --lib 里上一趟红的用例; 范围=1 个测试; 为什么=只重跑 x.log 里红的这 1 条用例，修完看它们转绿，不整份重跑' task-server/task submit -- cargo test -p singlefs-harness --lib -- --include-ignored --exact a::bad`
- 实测 E11、E12（钩子）：把这行里的 `task-server/task submit --` 照定义再插一次（`… task-server/task submit -- task-server/task submit -- cargo test -p singlefs-harness --lib`），子 agent 与主 agent 的钩子都退 0，没拦。二次提交到服务之后的结局我没实交（会起一条真 cargo），标「推的」：外层任务里跑的是客户端自己，它提交内层任务就退出，外层 `succeeded`；内层任务在服务的单元里被提交，会话号读不到（`task-server/task_server_lib.py:155` 沿进程树取最外层），推测记成「人手」，提交者收不到它的结局。
- 另一半实测 C11：子 agent 直接跑这个脚本本身（`python3 scripts/gate/rerun-failed-tests.py <日志> -p <包>`，`.claude/agents/implementation-writer.md:33` 与 `.claude/rules/verification.md:26` 教的第一步）被钩子拒，退 2，拒绝说明点名「本仓的 scripts/gate/rerun-failed-tests.py」；C12 经 submit 交它退 0。这两份定义没说「这个打印脚本也要经 submit 交、再读 `task log`」。
- 结论：对不上。脚本打出来的就是「声明 + submit + cargo」，按定义再插一次是多余的（且钩子拦不住）；定义还漏了一步（脚本自己也要经 submit）。
- 推翻它的现象：任务 1792 的日志里打出的命令不含 `task-server/task submit --`（实际含，见上）。

**F3　`build=` 指的是 `target` 的上一层，定义当成了 `target` 本身——对不上**

- 定义侧：「（编它的那一次 `task wait` 末行 `build=` 目录下的 `release/<bin 名>`）」（`.claude/agents/experiment-runner.md:29`）；「生成产物用的二进制取编它的那一次任务 `task wait` 末行 `build=` 目录下那一份」（`.claude/agents/experiment-runner.md:38`）。
- 对照侧：「build_directory = os.path.join(self.configuration.build_directory, label)」（`task-server/task_server.py:1671`）；「environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")」（`task-server/task_server.py:1673`）；末行的 `build=` 取的是这个 `build_directory`：「        line += f" build={build}"」（`task-server/task:158`）；文件头：「本机这一次尝试设 CARGO_TARGET_DIR=<TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>/target；跑完留着，wait 末行带 build=<编译目录>，」（`task-server/task_server_lib.py:122`）。
- 结论：对不上。`build=` 是 `<TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>`，二进制在 `<build>/target/release/<bin 名>`；照定义拼 `<build>/release/<bin 名>` 会找不到文件。
- 推翻它的现象：某次真实 `task wait` 末行的 `build=` 以 `/target` 结尾。我没有真起编译（不碰重活），这条是读代码得的，标「推的」。

**F4　共用约束让子 agent 直接跑 `proc.py`，钩子只放行主 agent——对不上**

- 定义侧：「要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），」（`.claude/agent-common.md:63`）；「放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`」（`.claude/agent-common.md:82`）；「或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`」（`.claude/agent-common.md:77`）。这三条写在给所有子 agent 读的共用约束里。
- 对照侧：「加 task-server/task 的 submit / wait / status / cancel / log / clean 与 task-server/control check），与只给主 agent 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY（proc.py、remind.sh、watch.sh、agent-watch.py、dispatch-prompt.py、detached-run.sh 的 list / watch / stop、」（`.claude/hooks/heavy-test-guard.sh:320`）；表里的一行：「    ".claude/singlefs-ai-sop/scripts/proc.py": "any",」（`.claude/hooks/lib_heavy_tests.py:2775`），上面一行注释：「# 只给主 agent 直接跑的会话工具：起在服务的单元里就丢了会话身份（停进程按会话认、定时提醒与看门狗按会话号认）或停不了服务自己（control 的装与起停）」（`.claude/hooks/lib_heavy_tests.py:2773`）。
- 实测（钩子，第五节 C01、C02、C03）：子 agent（implementation-writer）直接 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop 12345` 退 2、`python3 .claude/singlefs-ai-sop/scripts/proc.py wait 12345 --timeout 5` 退 2，拒绝说明「不经 task-server 跑项目代码被拒（implementation-writer）：本仓的 .claude/singlefs-ai-sop/scripts/proc.py」；同一条主 agent 退 0（C02）。
- 结论：对不上。共用约束对所有子 agent 教「逐个 proc.py stop」「proc.py wait」，钩子只放行主 agent；子 agent 照做被拒，钩子给的出路（`task-server/task submit -- python3 …/proc.py stop <pid>`）又走不通：对照侧自己写了 proc.py 起在服务单元里就「丢了会话身份」（`.claude/hooks/lib_heavy_tests.py:2773`），而且单元里的进程不是子 agent 会话的子孙，停不了子 agent 自己起的那条链。这是「出路无解」，不是写法问题，属 `.claude/agent-common.md:74` 说的机制问题，由主 agent 定（放进 TEXT_TOOLS 还是定义里改成「子 agent 不停，交回主 agent」）。波及的三条腿：`.claude/agents/three-way-defense.md:31`、`.claude/agents/three-way-forward.md:30`、`.claude/agents/three-way-attack.md:39`（都写着「停进程照旧一次点名一个」，指到「执行前拒绝的写法」⑥ 里放行的那条 `proc.py stop`）。
- 推翻它的现象：C01 钩子退 0（实际退 2）。

**F5　同一个 `type=command-failed`，三份读法不一——对不上**

- 定义侧一：「结局照 `task-server/task wait` 末行 `TASK_SERVER_RESULT` 的 `type=` 判」「不是 `succeeded` 的那一次输出不算产物。」（`.claude/agents/experiment-runner.md:29`，两句同一行）。
- 定义侧二：「服务判出的结局（`task wait` 末行 `type=` 不是 `succeeded`、`command-failed`、`not-run` 的）那一道这一趟没判出结果」（`.claude/skills/crash-test/commit-stages.md:27`）；`.claude/skills/gate/staged-gate.md:17` 同形：「服务判出的结局（`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=` 是 `memory-cap-at-maximum`、`never-fits`、`card-fault`、`launcher-lost`、`unit-start-failed`、`peer-run-failed` 这类」。
- 对照侧：命令自己退非 0 在服务里就是 `command-failed`（`task-server/task_server_lib.py:43`：「  command-failed          照命令  命令自己退非 0：表里没有的退出码一律落在这里，不吞掉（本机的 255 也在这里）」），变异驱动「有没红」恰恰退 1：「# 退出码：0 每条都判完、没有判失败的；1 有没红或抓名字的规则有盲区；」（`scripts/mutation/mutate.sh:129`），而 experiment-runner 同一份定义要读这类输出：「整份输出用 `task-server/task log <号> --all` 读）；跑完把没红的逐条按 `mutation-sampling.md` 分类。」（`.claude/agents/experiment-runner.md:35`）。
- 结论：对不上。按 `experiment-runner.md:29` 字面，`mutate.sh` 发现没红退 1 → `type=command-failed` → 「输出不算产物」，与第 35 步要分类没红矛盾；门禁两份 skill 则把 `command-failed` 当有结果。要么执行员那一句写成「`type=` 是服务判出的那几种（`commit-stages.md:27` 的口径）才不算产物」，要么说明变异、实验的非 0 退出另当。
- 推翻它的现象：`mutate.sh` 有没红时 wait 末行不是 `command-failed`（服务另有归类）。我读到的服务表里没有别的归类。

**F7　卡住怎么判：定义写得比服务严——对不上**

- 定义侧：「任务卡住由服务判（日志 10 分钟不涨就停掉报错、经 `task wait` 叫醒），调用方不另判」（`.claude/agent-common.md:73`）。
- 对照侧：「起来之后先宽限 10 分钟不判；之后日志满 10 分钟没涨、这段时间 CPU 也几乎没涨（平均不到一个核的 1%）的服务停掉、记 `stalled`」（`task-server/README.md:23`）；「  stalled                 233    卡住：起来后先宽限 STALL_GRACE_SECONDS（10 分钟）不判，之后日志距上一次增长满 STALL_SECONDS（10 分钟）、这段时间 CPU 也几乎没涨，或已跑超过记录用时的 5 倍而日志满 10 分钟没涨（不看 CPU），服务停了它（两台都判）；不重试」（`task-server/task_server_lib.py:61`）。
- 结论：对不上（定义漏了宽限、漏了「CPU 也没涨」这一条件）。后果：读定义的 agent 会以为一个 10 分钟不打日志的长计算会被停，转而给命令加无意义的输出，或把「日志 10 分钟没涨」当成必死的信号提前去 cancel。
- 推翻它的现象：服务代码里「日志满 10 分钟没涨」单独就停（不看 CPU、不宽限）。README 与文件头都说不是。

**F8　「服务对子 agent 交的重型当场拒」——拒的不是服务，是钩子——对不上（归属）**

- 定义侧：「服务对子 agent 交的重型当场拒」（`.claude/agent-common.md:59`）。
- 对照侧：「服务认不出提交的是不是子 agent」（`.claude/hooks/heavy-test-guard.sh:325`，同一行后面接「这一判只在钩子里」）；服务拿会话号的办法：「身份：服务经 unix socket 的 SO_PEERCRED 拿到调用进程号与 uid（uid 不是服务自己的拒），沿进程树往上读各祖先的 /proc/<进程号>/environ 里的 CLAUDE_CODE_SESSION_ID，」（`task-server/task_server_lib.py:155`）；我在 `task-server/task_server.py`、`task-server/task` 里 `grep -n 'agent_type\|subagent\|sub_agent'` 零命中，`task_server_lib.py` 只有第 123 行一句注释命中。
- 结论：对不上。拒重型的是 PreToolUse 钩子（只看 Bash 工具的命令文本与它读进来的脚本正文），服务不分主 agent 与子 agent。后果（推的，没造例子）：重型命令走钩子看不见的路（钩子读不到正文的脚本、已在服务单元里的二次提交）到了服务，服务照排照跑；定义那句「服务当场拒」给了一道并不存在的保险。
- 推翻它的现象：服务里有按调用者类型拒重型的分支。上面的 grep 是我能给的反证范围（这三个文件）。

**F9　被钩子拒了怎么办：钩子给的出路与共用约束第 74 行相反（认不出起什么的写法这一类）——对不上**

- 定义侧：「主 agent 与子 agent 先分清是哪一种：命令要写得精确，是写法的问题就照出路把命令或脚本改成合规的写法再交（去掉不必要的参数、包装、格式与异常的内容），不换写法绕过去」（`.claude/agent-common.md:74`）；用户原话：「如果被拒绝了 agent 和 子agent 需要自己修改自己的脚本  修改为合适的不违规的」（`records/2026-10-29-一百三十七节那一轮调度记录.md:222`）。
- 对照侧：子 agent 经 submit 交的命令被判成「认不出起什么的写法」时，钩子的出路是：「→ 怎么办：这一条不提交，把命令原样写进交回报告」（`.claude/hooks/heavy-test-guard.sh:2229`），触发条件：「服务认不出提交的是不是子 agent，所以由这道闸照 lib_heavy_tests.classify 判 `--` 之后那条：重型、提交时才跑的检查与认不出起什么的写法照拒，」（`.claude/hooks/heavy-test-guard.sh:2227`）。
- 实测（本次会话，非推）：我第一次提交喂钩子脚本时命令里写了 `python3 $D/feed.py`（脚本路径放在变量里），被拒，拒绝说明是「python 起的脚本路径 `$D/feed.py` 由变量、命令替换、位置参数或进程替换拼出，执行前展开不了，认不出起的是什么（认不出起什么的写法）」，末尾的出路是上面那句「这一条不提交，把命令原样写进交回报告」；把路径改成字面的就过了（任务 1784 起了）。
- 结论：对不上。对这一类（纯写法问题）钩子的出路是交回，定义与用户原话是改写法重交；钩子文字会把子 agent 引向不必要的交回。重型那一类钩子的出路与定义一致。
- 推翻它的现象：把 `SUBMITTED_SUBAGENT_POLICY` 的出路按拒绝的种类分开（认不出的写法 → 改字面再交）；我读到的此刻文字没有分开。

**F11　`task log <号>` 默认只给最后 200 行，定义里多数份只写 `task log <号>`——部分对得上**

- 定义侧：「读 `task-server/task log <号>` 接着做」（`.claude/agent-common.md:59`）；「最后一次尝试的日志看 `task-server/task log <号>`」（`.claude/agent-common.md:70`）；同一份里的另一条：「输出不截断，嫌长先数。报告里的条数、阶段数、命中数用命令数出来，不手数」（`.claude/agent-common.md:111`）。
- 对照侧：「task-server/task log <号>                           # 最后一次尝试的日志（最后 200 行；--all 整份；--attempt <第几次> 取那一次；任务或那一次不在退 4）」（`task-server/README.md:32`）；「LOG_TAIL_LINES = 200」（`task-server/task:47`）。
- 写了 `--all` 的：`.claude/agents/mutation-triage.md:28`、`.claude/agents/experiment-runner.md:35`、`.claude/skills/gate/staged-gate.md:15`、`.claude/skills/crash-test/commit-stages.md:19`。只写 `task log <号>` 的：`.claude/agents/implementation-writer.md:31`、`investigator.md:28`、`.claude/agents/kb-scribe.md:32`、`three-way-forward.md:30`、`three-way-defense.md:31`、`three-way-attack.md:38`、`tooling-writer.md:29`、`clerk.md:27`、`experiment-runner.md:28`。
- 结论：取汇总行、取末行用 `task log <号>` 够（`.claude/agent-common.md:104` 要的「汇总行」在尾部）；要贴「原样输出、不截断」的（如喂钩子、rerun.sh 的整份输出，超过 200 行）只写 `task log <号>` 会截。定义没说什么时候加 `--all`。判：部分对得上，缺一句。
- 推翻它的现象：服务的 `task log` 默认整份（`task-server/task:377` 的 `lines[-LOG_TAIL_LINES:]` 不在）。

**F12　GPU 手跑「先交 `--no-run`」——服务对 `--no-run` 没有例外——对不上（推的，不碰 GPU 没实测）**

- 定义侧：「手跑的 GPU 测试分两条交：先交 `--no-run` 只编的那一条，等到结局再交只跑测试的那一条」（`.claude/rules/verification.md:63`）。
- 对照侧：服务认「要卡」只看 cargo 的特性：「def cargo_uses_gpu(words):」（`task-server/task_server.py:538`）到「    return any(feature == "gpu" or feature.endswith("/gpu") for value in values for feature in re.split(r"[\s,]+", value) if feature)」（`task-server/task_server.py:552`）之间没有对 `--no-run` 的判断（`grep -n 'no-run' task-server/task_server.py task-server/task_server_lib.py task-server/task` 零命中）。
- 结论：对不上的是隐含的前提：只编的那一条带 `gpu` 特性，服务照样给它要卡、清场、占卡；两条分开交只是多占一次卡，不省卡。定义没说它省什么。
- 推翻它的现象：`task status` 对一条带 `--features gpu --no-run` 的任务报「不要卡」。（要交带 gpu 特性的 cargo，按本轮不碰 GPU 的约束我没交。）

**F13　定义里「服务当场拒」的清单比服务少一样：`SINGLEFS_THREAD_CAP`——对不上（轻）**

- 定义侧：见 1.1 第 8 条，`.claude/agent-common.md:59` 的清单里没有 `SINGLEFS_THREAD_CAP`。
- 对照侧：「命令里不套包装、不写资源参数：`run-with-memory-cap.sh <上限>`、`host-dispatch.py … --`、`multi-host-run.sh … --`、`detached-run.sh start`、`nice`、`timeout`、`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh` 与脚本自己的 `--memory`、`--memory-cap`、`--checker-quick-memory`、`--threads`、`--peer`、`--local-only`、`SINGLEFS_THREAD_CAP`：这几样由服务做，写了当场拒并点名去掉哪一样。」（`task-server/README.md:36`）；代码：「            if word.startswith("SINGLEFS_THREAD_CAP="):」（`task-server/task_server_lib.py:1047`）。
- 结论：对不上（轻）。共用约束清单没列它，又说「白名单外的写成 `-- env <名>=<值> <命令…>`」，会让人以为 `-- env SINGLEFS_THREAD_CAP=… cmd` 是白名单外的合规写法；服务连这种写法也拒（`task-server/task_server_lib.py:1047` 扫的是命令词前面的 `名=值` 词，`env` 后面的赋值词在命令词之前）。这是 M05 的落点，见第四节。
- 实测 R2（本次会话，客户端当场拒，没排进队）：`task-server/task submit --cwd <草稿目录> -- env SINGLEFS_THREAD_CAP=2 true` 退 3，stderr 原样：`  ✗ 命令自己套了一层：SINGLEFS_THREAD_CAP=2：线程一律是登记的每台默认重活线程数，命令里不写它（去掉这个赋值）`、`  → 怎么办：去掉那一层再提交：task-server/task submit -- <里面那条命令…>；挑机器、定上限、分卡都由服务做`。所以 `-- env SINGLEFS_THREAD_CAP=… cmd` 这条「白名单外的合规写法」被拒，上面的推断成立。
- 推翻它的现象：上面这条退 0（实际退 3）。

**F14　实现员的重型默认不写进交回（M01「重型交回」的字面不一致）——不一致，内容自洽**

- 基准：「子 agent 不交，写进报告」（`.claude/agent-common.md:59`，同一行后面接「要主 agent 跑的命令」一节交回）。
- 实现员：「checker 档的测试不交、也不写进」（`.claude/agents/implementation-writer.md:39`，后面接「要主 agent 跑的命令」）；同一行：「checker 档的快档与全量、轻型三条不带 `--quick` 的重型跑法都不交、也不写进」；派发提示另点名要跑的才写：「重型的写进「要主 agent 跑的命令」一节、标「重型」」（同一行末）。
- 结论：字面上和 `.claude/agent-common.md:59` 不一致（重型默认不交回），内容上自洽（留给提交时主 agent 统一跑，`.claude/agents/implementation-writer.md:62` 也这么写）。是缺陷还是例外由主 agent 判。
- 推翻它的现象：主 agent 的判决把「重型交回」定成对所有类型一律写进报告。

**F15　实验执行员把「耗时类的量」的产物命令也交回主 agent，服务没有「独占」档——不一致，且判不了**

- 定义侧：「登记第六节里有耗时类的量时，产物命令不自己交，写进「要主 agent 跑的命令」一节，写明跑之前连别的 `cargo`、`gate.sh` 也要等。」（`.claude/agents/experiment-runner.md:28`）。
- 对照侧：服务是「同一个队、按到达次序、不另设延迟」（`task-server/README.md:5`，同一句：「singlefs 的统一任务服务：一切跑项目代码的任务（轻、重、CPU、GPU）都经它起，同一个队、按到达次序、不另设延迟」）；我在 `task-server/README.md` 与 `task-server/task` 文件头里没有找到「独占」「别的任务都等」的提交参数。
- 结论：和 M01「非重型自己交」的字面不一致（这一类非重型却交回）；「跑之前连别的 `cargo`、`gate.sh` 也要等」靠主 agent 自己排，服务不替它保证。判不了它对不对：缺一个「独占」的取舍，是 M01 之外的另一件事。
- 推翻它的现象：服务有独占提交参数，或主 agent 在别处定了「耗时类的量由执行员自己交」。

**F17　`--cwd` 给仓根时命令可能被派去第二台，带本机绝对路径的草稿脚本到那边找不到路径——定义没覆盖，判不了**

- 定义侧：「内存上限、线程、哪台机器、哪几张卡、GPU 清场、排队次序、分片、卡住、撞上限放大重排都由服务定，调用方不写、不判」（`.claude/agent-common.md:59`）；「（不给 `--cwd` 取此刻的目录）」同一行。
- 对照侧：「  第二台另要命令派得过去：工作目录是仓根、点到 task-server/ 的不派、multi-host-run.sh --verdict-library-route 判不带判定库、多机配置的双机开关开着；」（`task-server/task_server_lib.py:110`）。
- 实测（本次会话）：任务 1783 我写的是 `task-server/task submit --cwd <仓根> -- python3 <草稿目录里的脚本>`，服务把它派到了第二台，那边脚本里写死的本机仓路径不存在：日志里「FileNotFoundError: [Errno 2] No such file or directory: '〔本机仓根〕'」（原输出里是本机仓根的绝对路径，这里换成了〔本机仓根〕），`task wait` 末行 `type=command-failed exit=1`。改成 `--cwd <草稿目录>` 重交（任务 1784、1785、1792、1793）就只在本机排队。
- 结论：定义没有说「`--cwd` 给仓根且命令不点 `task-server/` 时可能被派去第二台」，也没说「命令里带本机绝对路径要给 `--cwd` 非仓根的目录」；`.claude/agents/three-way-forward.md` 这一类定义里的 `submit [--cwd <目录>]` 写成可选，照字面不给 `--cwd` 取此刻目录，取决于 agent 当时在哪。对不对判不了：服务的挑台是服务定的，定义「不判」是 M01/M03 的口径；缺的是一句提示。
- 推翻它的现象：服务把「命令里点到的仓外文件」随树送去之外，还把脚本正文里写死的本机绝对路径换成那边的路径。README 只说换命令参数里的路径：「参数里点到的仓外文件与目录（合计至多 2G、此刻在的）由服务随树送到第二台、换成那边的相对路径」（`task-server/README.md:21`，该行很长，这是其中一整句）；脚本正文它不动。所以这不是服务的毛病，是定义缺一句提示。

**F18　子 agent 与主 agent 同一个会话号，`task status` 无号时列出别人的任务——定义没覆盖，判不了**

- 对照侧：「身份：服务经 unix socket 的 SO_PEERCRED 拿到调用进程号与 uid（uid 不是服务自己的拒），沿进程树往上读各祖先的 /proc/<进程号>/environ 里的 CLAUDE_CODE_SESSION_ID，」（`task-server/task_server_lib.py:155`，下一行接「取最外层」）；「task-server/task clean <号>                         # 删这条任务留下的编译目录（只许提交它的会话、有了结局才删）」（`task-server/README.md:33`）。
- 实测（本次会话）：我无号 `task-server/task status` 列出了 1781、1782、1787 等不是我交的任务（会话号与我的相同，命令是主 agent 与另一条腿的）。
- 定义侧：「排队位次与「在等：…」看 `task-server/task status [<号>]`」（`.claude/agent-common.md:70`）。定义没写「子 agent 只看自己交的号、不 cancel / clean 别人的」。
- 结论：判不了对错，是缺一句。后果：子 agent 的 `task clean`、`task cancel` 对别人的号也会成功（同一会话）。
- 推翻它的现象：服务按进程树里更内层的标识区分子 agent。我读到的取最外层。

### 1.3 各份用的起法句型（逐份核过一句：submit 写法、`--cwd`、范围声明位置、`task wait`、`task clean`）

| 文件:行 | submit 写法 | `--cwd` | 范围声明 / 前缀位置 | `task wait` | `task clean` | 结论 |
|---|---|---|---|---|---|---|
| `.claude/agents/clerk.md:27` | `task-server/task submit -- <命令>` | 无 | 未写 | 后台起 | `build=` 的用完 clean | 对得上 |
| `.claude/agents/experiment-runner.md:28-29` | `submit [--cwd <目录>] -- <命令>`；`--cwd scripts/experiment-apparatus` | 相对仓根 | 写在 submit 前面 | 后台起 | `build=` 的用完 clean | 对得上（`build=` 路径见 F3；`type=` 读法见 F5） |
| `.claude/agents/implementation-writer.md:31-34` | `submit [--cwd <根>] -- <命令>` | `<根>` 填改动所在的树 | 写在 submit 前面（1b、1d） | 后台起 | 同上 | 对得上（rerun 一句见 F2） |
| `.claude/agents/investigator.md:28` | `submit [--cwd <副本>] -- <命令>` | 副本 | 写在 submit 前面 | 后台起 | 同上 | 对得上 |
| `.claude/agents/kb-scribe.md:31-32,35` | `submit -- env AGENT_HOOK_DETECTIONS=… bash -c '…'`；`SINGLEFS_RUN_SCOPE=… submit -- bash …doc-decisions.sh --write …` | 无 | 写在 submit 前面 | 后台起 | 无编译，没提 clean | 对得上（31 的钩子放不放行见第五节 C14） |
| `.claude/agents/mutation-triage.md:28,30` | `submit [--cwd <目录>] -- <命令>`；`--cwd scripts/experiment-apparatus` | 相对仓根 | 未写 | 后台起 | 同上 | 对得上 |
| `.claude/agents/three-way-attack.md:38,51` | `submit [--cwd <副本>] -- <命令>` | 副本 | 写在 submit 前面 | 后台起 | 同上 | 对得上（35、51 的「直接执行」见 F16） |
| `.claude/agents/three-way-defense.md:31` | `submit -- <命令>` | 无 | 未写 | 后台起 | 同上 | 对得上 |
| `.claude/agents/three-way-forward.md:30` | `submit -- <命令>` | 无 | 未写 | 后台起 | 同上 | 对得上 |
| `.claude/agents/tooling-writer.md:29` | `submit -- <命令>` | 无 | 未写 | 后台起 | 同上 | 对得上（35 的命名检查一句见 F1） |
| `.claude/agents/experiment-designer.md` | 不起项目命令 | — | — | — | — | 无可比 |
| `.claude/main-agent.md:7,17,31,52,55,98` | `submit -- bash …`、声明在前 | 无（`--cwd <worktree>` 在 commit-stages.md:26） | 声明与前缀写在 submit 前面 | 后台起 | 31：跑完 clean | 对得上（43、30、100 的未带 submit 见 F10） |
| `.claude/rules/implementation-workflow.md:43,46,47,49` | `submit --cwd <自己的副本> -- <命令>`、`submit --cwd <集成副本> -- …` | 副本 | 前缀在 submit 前面 | 后台起 | 未提 | 对得上 |
| `.claude/rules/verification.md:26,27,55,59,63` | `submit -- <命令>` | 无 | 声明挪到 submit 前面（60） | 未提 | 未提 | 26 一句见 F2，63 一句见 F12，其余对得上 |
| `.claude/rules/three-way-inference.md:106,128` | `submit -- <命令>`、`submit --cwd <副本> -- <命令>` | 副本 | 未写 | 后台起 | 128：clean 删 | 对得上 |
| `.claude/rules/test-script-discipline.md:89` | 不写起法 | — | — | — | — | 无可比（只说「由统一任务服务分，调用方不写、不挑」，与 `task-server/README.md:5` 一致） |
| `.claude/skills/gate/SKILL.md:12` | `SINGLEFS_HEAVY_TESTS=commit` 在前、`submit -- <命令>` | 无 | 前缀在 submit 前面 | 后台起 | 未提 | 对得上（前缀要不要见 F6） |
| `.claude/skills/gate/staged-gate.md:15-16` | `submit -- <命令>` | 无 | 声明与前缀在 submit 前面 | 后台起 | 15：跑完 clean | 对得上（17 的 `env.sh` 见 F10，`type=` 读法见 F5） |
| `.claude/skills/crash-test/SKILL.md:14-15` | 「`task-server/task submit` 交」 | 无 | 前缀写在命令里 | 未提 | 未提 | 对得上 |
| `.claude/skills/crash-test/commit-stages.md:17-19,26` | `submit -- <命令>`、`submit --cwd <worktree> -- …` | worktree | 声明与前缀在 submit 前面 | 后台起 | 17：跑完 clean | 对得上（18、22、26 的未带 submit 见 F10） |
| `.claude/agent-common.md:59-60,70,104` | 基准 | 基准 | 基准 | 基准 | 基准 | 见 1.1、F7、F8、F11 |

（这一张表是我逐份读那几行得到的归纳，不是引文；每一格的原句在对应行里，必要时 `grep -nF 'task-server/task' <文件>` 就能复核。）

## 二、问题 2：M01 落进各份是不是同一个意思

基准（用户口径 M01「非重型自己交，重型交回」）落在共用约束两处：「子 agent 自己交非重型的：轻型三条、登记为 light 的自证、样本档与弄坏开关、harness 测试、clippy / build、证红」（`.claude/agent-common.md:59`）与「子 agent 不交，写进报告」（同一行，后面接「要主 agent 跑的命令」一节交回）；交回内容的三样写在 `.claude/agent-common.md:60`：「在哪个目录起（主工作区写仓根，副本写副本的绝对路径，即 submit 的 `--cwd`」。

逐份（谁交非重型 / 谁交重型 / 交回写什么）：

| 文件:行 | 非重型谁交 | 重型谁交 | 交回写什么 | 与基准 |
|---|---|---|---|---|
| `.claude/main-agent.md:16-17` | 子 agent 自己交，后台起 `task wait` | 主 agent 照原样经服务交；子 agent 写进报告 | 报告「要主 agent 跑的命令」一节 | 一致；清单 6 项与基准逐项同（我逐项比过） |
| `.claude/main-agent.md:44,68,76` | 同上（实现员自己交动到的二进制、clippy、build、证红、轻型三条） | 同上 | 同上 | 一致 |
| `.claude/agents/clerk.md:27` | 自己交 | 写进报告交回 | 要主 agent 跑的命令一节 | 一致 |
| `.claude/agents/experiment-runner.md:28` | 自己交 | 写进报告交回 | 同上 | 一致，但有两处偏：F15（耗时类的量也交回）、F5 |
| `.claude/agents/implementation-writer.md:31,39,62` | 自己交（含轻型三条，1d） | 「照第 4 步办：不交」 | 默认不写进（只写派发提示点名的） | 与基准字面不一致，见 F14 |
| `.claude/agents/investigator.md:28,32` | 自己交 | 写进报告，标「重型」，「交主 agent 去问用户」 | 同上 | 一致（多一句「去问用户」是主 agent 的事，不冲突） |
| `.claude/agents/kb-scribe.md:31,32,35` | 自己交门禁生成一格（`--write`）与喂写范围闸 | 无 | 末行 | 一致；kb-scribe 交的 `--write` 不在基准那 6 项里，钩子另放行：「带 --write（生成）的不算。」（`.claude/hooks/heavy-test-guard.sh:87`） |
| `.claude/agents/mutation-triage.md:28` | 自己交 | 写进报告交回 | 同上 | 一致 |
| `.claude/agents/three-way-attack.md:38,43,59` | 自己交 | 写进报告 | 同上，副本上的数标「推的，待主 agent 跑」 | 一致 |
| `.claude/agents/three-way-defense.md:31,44` | 自己交 | 只写进报告那一节 | 同上 | 一致 |
| `.claude/agents/three-way-forward.md:30,44` | 自己交 | 写进报告，那一格写「待主 agent 跑」 | 同上 | 一致 |
| `.claude/agents/tooling-writer.md:29,35,50` | 自己交（自证、样本档、弄坏开关） | 重型的与钩子换上写进报告 | 同上 | 一致；35 的命名检查一句见 F1 |
| `.claude/agents/experiment-designer.md` | 不起命令 | — | — | 无可比 |
| `.claude/rules/implementation-workflow.md:43,47,49` | 子 agent 自己交 harness 与轻型三条 | 「重型写进交回报告由主 agent 交」 | 同上 | 一致（43 的清单是：harness 测试二进制、clippy / build、证红、轻型三条，比基准少「登记为 light 的自证、样本档与弄坏开关」，46 另写了仓副本里改过正文的自证归主 agent；不冲突） |
| `.claude/rules/implementation-workflow.md:46` | 副本里改过正文的脚本的自证：子 agent 不交 | 主 agent 在集成副本里交，命令带 `SINGLEFS_HEAVY_TESTS=commit` | 把命令写进交回报告 | 与钩子一致（`.claude/hooks/heavy-test-guard.sh:104-107` 同一规则）；前缀要不要见 F6 |
| `.claude/rules/verification.md:27,55,59` | 主 agent 与子 agent 都交（慢档、轻型三条） | 不带 `--quick` 的照重型拒 | — | 一致 |
| `.claude/rules/three-way-inference.md:106,128` | 腿自己交 | 「腿非重型的自己交、重型交主 agent」 | 要主 agent 跑的命令一节 | 一致 |
| `.claude/skills/gate/SKILL.md:12` | 登记为 light 的自证：「改了谁就经 `task-server/task submit` 交谁」（没说谁交） | 主 agent 提交时 | — | 没说谁交，不冲突 |
| `.claude/skills/crash-test/SKILL.md:14-15` | 实现员交自己动到的 `--test` 目标 | 主 agent 交 checker 档 | — | 一致 |
| `.claude/skills/gate/staged-gate.md`、`crash-test/commit-stages.md` | — | 主 agent 交 | — | 一致（主 agent 的活） |

### 2.1 M01 上不一致的几处（汇总）

1. F8：共用约束说「服务对子 agent 交的重型当场拒」，拒的其实是钩子。
2. F14：实现员的重型默认不写进交回（和基准的字面不一致）。
3. F15：耗时类的量的产物命令，执行员也交回（非重型却交回）。
4. F9：被拒之后钩子的出路（交回）与共用约束第 74 行（改写法重交）在「认不出的写法」这一类上相反。
5. F1：experiment-runner、tooling-writer 还写着命名检查「交主 agent」，与共用约束第 104 行相反。

**F10　定义让主 agent 直接跑的本仓脚本，钩子一概拒（没带 submit，主 agent 也拒）——对不上，十一处（另 `stage-mine.py` 一处也喂了，同形的两处没喂）**

- 钩子的规则：「主 agent 带 SINGLEFS_HEAVY_TESTS 前缀也照拒；读进来的脚本正文里的照判（脚本路径与行号写进拒绝说明）。拒绝说明的出路是 `task-server/task submit -- <原命令>`，」（`.claude/hooks/heavy-test-guard.sh:314`）；放行名单里的文本工具只有那一串：「stale-candidates.py、kb-sections.py、checklist-specs.py、cite-check.py、claim-experiment.sh、gate-overlap.py --list，主 agent 定的 relabel-item.py、copy-originals.py，」（`.claude/hooks/heavy-test-guard.sh:319`）加 `.claude/hooks/heavy-test-guard.sh:320` 的主 agent 会话工具。
- 逐处（定义原文 → 把这一句字面喂给钩子的结果，第五节 D、E 两批；钩子版本 cc9fd871 + lib_heavy_tests.py cc11e40f）：
  1. 「主 agent 在全部交回后跑一次 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync`」（`.claude/main-agent.md:30`，第 100 行同一条）→ D01 退 2；同一条加上 `task-server/task submit --` D02 退 0。**对不上**。
  2. 「再 `python3 scripts/gate/apply-writer-patch.py <补丁目录>` 逐份打进工作树」（`.claude/main-agent.md:43`）→ D04 退 2。**对不上**。
  3. 「主 agent 写 `research/prompts/_<轮>-body.md` → `python3 scripts/agents/three-way-materials.py <轮>`」（`.claude/main-agent.md:92`）→ E07（`--help`）退 2。**对不上**。
  4. 「重跑之前主 agent 用 `python3 scripts/gate/admission.py experiment <仓根> E<号>` 判准入」（`.claude/agents/experiment-designer.md:35`）→ D12 退 2。**对不上**。
  5. 「先跑 `python3 scripts/agents/snapshot-tree.py <轮> --out <草稿目录>/<轮>-tree`」（`.claude/rules/three-way-inference.md:126`）→ E02 退 2。**对不上**。
  6. 「`python3 scripts/agents/agent-handover.py --agent <它的 id> --out <交接摘要>`」（`.claude/rules/three-way-inference.md:163`）→ E03 退 2。**对不上**。
  7. 「别用手抄：`scripts/kb/quote-kb.py` 按」…（`.claude/rules/three-way-inference.md:51`，第 53、70 行同名脚本的 `--selftest`、`--checklist` 用法）→ E01 退 2。**对不上**。
  8. 「工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证」（`.claude/skills/gate/staged-gate.md:17`）→ D05 退 2（拒绝说明还列出它在第 12、5 行里 source 的两层脚本）。**对不上**。
  9. 「先只读列一次 `SINGLEFS_HEAVY_TESTS=<commit|user-request> bash scripts/crash/crash-point-replay.sh --list-items --backlog --milestone <mN>`」（`.claude/skills/crash-test/commit-stages.md:26`）→ D03 退 2（`--list-items` 单独才放行，带 `--backlog --milestone` 照重型拒）。**对不上**。
  10. 「起单元之前在会话 shell 里跑 `SINGLEFS_HEAVY_TESTS=commit python3 scripts/mutation/broad-mutation.py --unit-environment`」（`.claude/skills/crash-test/commit-stages.md:22`）→ D06 退 2。**对不上**；而且这一句的意图「在会话 shell 里」（读的是会话 shell 里的构建环境变量）经服务交就变成读服务单元的环境，这一步在服务下还要不要、读谁的环境，我判不了。
  11. 「开跑前在那棵 worktree 里跑 `bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments <worktree>`」（`.claude/skills/crash-test/commit-stages.md:18`）→ D07 退 2。**对不上**。
- 同形、这一轮没喂的（推的，按同一条规则）：「主 agent 先用 `python3 scripts/kb/batch-target.py` 只拿这一批的路径与块做一个提交对象」（`.claude/main-agent.md:97`）、`.claude/main-agent.md:28` 的 `python3 scripts/agents/agent-cost.py usage --since <开工日期>`、`.claude/main-agent.md:98` 与 `.claude/skills/gate/staged-gate.md:7` 的 `scripts/gate/stage-mine.py`（E04 已喂：退 2）。
- 对得上的主 agent 直接跑：`bash scripts/agents/watch.sh`（D08 退 0）、`python3 scripts/agents/cite-check.py`（E05 退 0）、`python3 scripts/kb/kb-sections.py`（E06 退 0）、`python3 .claude/singlefs-ai-sop/scripts/proc.py stop`（C02 退 0）。
- 结论：这十一处加 `stage-mine.py`（E04 退 2）定义写的是直接跑，钩子拒，钩子的拒绝说明会给出能抄的 submit 那一条，所以不会卡死；但定义是错的，主 agent 与新派来的腿照字面做都先撞一次拒。出路在 `.claude/agent-common.md:74` 的第一分支（写法问题改写法）：每处补 `task-server/task submit --` 并后台 `task wait`；或者把这几个脚本放进 TEXT_TOOLS（机制问题，由主 agent 定）。
- 推翻它的现象：把钩子换成别的版本后这几条退 0；或这几个脚本被加进 `TEXT_TOOLS_RUN_DIRECTLY`。喂钩子的脚本与原样输出在草稿目录，复跑命令在「要主 agent 跑的命令」。

**F16　「直接执行编出来的二进制」——字面被钩子拒，换一种读法对得上——判不了**

- 定义侧：「调枚举函数的原型只交 `cargo build` 与直接执行编出来的二进制（不 `cargo run`、不 `cargo test`）」（`.claude/agents/three-way-attack.md:35`）；「调枚举函数的原型照第 3b 步只 `cargo build` 再直接执行编出来的二进制」（`.claude/agents/three-way-attack.md:51`）。
- 对照侧：C09（三方攻方腿直接执行 `<副本>/target/debug/proto`）退 2：「✗ 不经 task-server 跑项目代码被拒（three-way-attack）：直接执行 cargo 编出来的二进制」；C10（`task-server/task submit -- <那个二进制>`）退 0。钩子文件头里还留着改前的话：「许可的用法：攻方原型 `cargo build` 之后经内存包装直接执行编出来的二进制」（`.claude/hooks/heavy-test-guard.sh:37`）。
- 结论：字面「直接执行」对不上；若读作「交（经服务）编出来的二进制」则对得上。同一句里 `cargo build` 前面写的是「交」，后面「直接执行」没有动词「交」，容易读成不经服务。判不了作者意图，建议改成「经 `task-server/task submit` 交编出来的二进制」。（第 35 行说原型不 `cargo run`，第 51 行的 `cargo run` 是副本里别的运行，两处说的不是同一件事，我不算它们冲突。）
- 推翻它的现象：作者意图本就是 submit；这时只剩措辞问题。

**F6　（M07）见第四节 4.2。**

## 三、问题 3：条名改了之后，各份引的名字对不对得上现存的条

现存的四条（`grep -nF` 在 `.claude/agent-common.md`，sha8 8858e562）：「**子 agent 不交重型**：编译、测试、门禁、实验、自证这类跑项目代码的命令，谁起都交统一任务服务」（`.claude/agent-common.md:59`，在 `## 不做` 一节里）；「长活交回主 agent：重型交回主 agent」（`.claude/agent-common.md:70`）；「**命名检查交回前跑**：改了代码或实验源码」（`.claude/agent-common.md:104`，在 `## 门禁` 一节里）；「执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前」（`.claude/agent-common.md:74`）。

| 条名 | 引它的位置（`grep -noE` 取的） | 对得上？ |
|---|---|---|
| 子 agent 不交重型 | `.claude/agents/` 下 clerk.md:27、experiment-runner.md:28、implementation-writer.md:31、investigator.md:28、.claude/agents/kb-scribe.md:32、mutation-triage.md:28、three-way-attack.md:38、three-way-defense.md:31、three-way-forward.md:30、tooling-writer.md:29（都写「共用约束「不做」一节「子 agent 不交重型」」）；`.claude/main-agent.md:16,17,44,68,76`；`.claude/rules/implementation-workflow.md:43,46`、`three-way-inference.md:106`、`verification.md:27,59`；`.claude/skills/gate/staged-gate.md:3`、`.claude/skills/crash-test/commit-stages.md:3` | 对得上（章节在「不做」，句首加粗） |
| 长活交回主 agent | 只有 `.claude/main-agent.md:70`（「主 agent 自己交的任务（长的同样）照共用约束「长活交回主 agent」那一条」） | 对得上 |
| 命名检查交回前跑 | `.claude/agents/implementation-writer.md:39`（「共用约束「门禁」一节「命名检查交回前跑」」）；共用约束自己 `.claude/agent-common.md:103` 指到它 | 对得上；experiment-runner.md:42,59 与 tooling-writer.md:35 没引这个名，写的是旧说法，见 F1 |
| 执行前拒绝的写法 | `experiment-runner.md:38`（⑤）、`three-way-attack.md:40,41`（④、⑦）、`three-way-defense.md:31`、`three-way-forward.md:30`（⑥）、`.claude/main-agent.md:60` | 对得上；它们点的 ④⑤⑥⑦ 在现行共用约束里都在：④ 在 `:79`，⑤ 在 `:81`，⑥ 在 `:82`，⑦ 在 `:84` |

别的名字也一并核了（共用约束的小节名、条名），都在：「规则怎么读」一节（`## 规则怎么读`，第 17 行）、「重型测试就是 checker 档那一批」（`.claude/agent-common.md:61`）、「几个会话共写一份文件」（`.claude/agent-common.md:68` 句中，被 `.claude/skills/crash-test/commit-stages.md:26` 当条名引，它不是标题也不是加粗，只是一句里的片段，能找到）、「崩溃点测试准确率和正确率优先，其次再衡量时间成本」（`.claude/agent-common.md:66`）、「找不到历史实验的数据」（`.claude/agent-common.md:29`）、「里程碑 17 收尾之前不兼容任何旧东西」（`.claude/agent-common.md:49`）、「进仓的文字只写本项目的东西」（`.claude/agent-common.md:52`）。**没有一处引到已经不存在的条名**（我在 21 份里搜了 `长活声明`、`命名检查交主 agent 跑`、`交回前自查`，零命中）。

对照侧（不判，只报）：钩子注释里还有旧名两处，已在 F1 写出：`.claude/hooks/handback-guard.sh:33`「命名检查交主 agent 跑」、`.claude/hooks/heavy-test-guard.sh:96`「交回前自查」。

**结论：条名都对得上；对不上的是 F1 那三句的内容（不是条名）。**

## 四、问题 4：M05、M07、M09 有没有被哪一份悄悄定了

派发时的口径是「M05、M07、M09 没定，写成不表态的形态」（`records/2026-10-29-一百三十七节那一轮调度记录.md:205`）。

### 4.1 M05（线程能不能覆盖）——被定了，且定的是「提交时不写、一律登记值」，与服务一致

- 定义侧：「服务分到哪台都取这一个数，机器忙时也固定这个数、不看忙闲；调用方提交时不写线程」（`.claude/rules/implementation-workflow.md:64`）；「内存上限、线程、哪台机器、哪几张卡、GPU 清场、排队次序、分片、卡住、撞上限放大重排都由服务定，调用方不写、不判」（`.claude/agent-common.md:59`）；「派发提示里不写线程、内存上限、台数与卡、分片与续跑：这些由服务定，子 agent 交的命令里同样不写。」（`.claude/main-agent.md:80`）；实验侧：「线程数取登记值」（`.claude/agents/experiment-runner.md:34`），「台数、挑哪台与内存上限不写进登记（统一任务服务定）。」（`.claude/agents/experiment-designer.md:34`）；还有四份定义写「命令里不套包装、不写内存上限与线程」（`.claude/agents/implementation-writer.md:32`、`investigator.md:28`、`three-way-attack.md:38`、`experiment-runner.md:29`）。
- 服务侧：「每次尝试的命令外面套 `scripts/hosts/capped.sh <每台默认重活线程数>`，线程一律是这个数，提交时不收」（`task-server/README.md:23`）；连 `-- env SINGLEFS_THREAD_CAP=…` 也拒：「            if word.startswith("SINGLEFS_THREAD_CAP="):」（`task-server/task_server_lib.py:1047`）。另一个会话的记录里 M05 已写成：「M05：线程一律登记值，`SINGLEFS_THREAD_CAP` 出白名单。」（`records/2026-10-30-编译绕过内存总上限.md:259`）。
- 结论：这些定义没有不表态，写的就是「调用方不能覆盖」，和服务现行做法（也是同一个会话里定的）一致。如果 M05 在用户那里还开着，需要改的是上面这些定义与服务两边；定义侧另漏了 `SINGLEFS_THREAD_CAP` 的点名（F13）。
- 推翻它的现象：用户把 M05 定成「可以覆盖」，则以上十来处与服务要一起改。

### 4.2 M07（重型要不要 `SINGLEFS_HEAVY_TESTS` 前缀）——F6：被悄悄定成了「要」，只有共用约束第 59 行按不表态写，同份第 61 行又写成「带」；钩子文字写「不要」

- 写成「要」的（逐字）：「重型测试只由主 agent 带 `SINGLEFS_HEAVY_TESTS` 前缀跑」（`.claude/main-agent.md:16`）；「重型测试只由主 agent 在提交时或用户要求时经服务交、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）」（`.claude/agent-common.md:61`）；「两条命令开头都带 `SINGLEFS_HEAVY_TESTS=user-request`」（`.claude/main-agent.md:52`）；「主 agent 经 `task-server/task submit --cwd <集成副本> -- …` 交，命令带 `SINGLEFS_HEAVY_TESTS=commit`，不弹窗」（`.claude/rules/implementation-workflow.md:46`）；「主 agent 经 `task-server/task submit` 交，命令带 `SINGLEFS_HEAVY_TESTS=commit`（提交时）或 `=user-request`（用户要求）」（`.claude/skills/crash-test/SKILL.md:15`）；「那个前缀与四段范围声明」（`.claude/skills/crash-test/commit-stages.md:19`，句首「六道的命令都带「跑之前定下来」一节定的」）；「带 `SINGLEFS_HEAVY_TESTS=commit` 把 `bash scripts/gate/gate-staged.sh` 经 `task-server/task submit -- <命令>` 交、」（`.claude/skills/gate/SKILL.md:12`）。
- 按不表态写的：方括号可选「`SINGLEFS_RUN_SCOPE='…' [SINGLEFS_HEAVY_TESTS=commit|user-request] task-server/task submit -- <原命令>`」（`.claude/agent-common.md:59`）。同一份第 61 行就写成「带」，所以共用约束内部就不一致。
- 钩子写「不要」：「#      经 task-server/task submit 交给服务的那一条：不判内存包装、不要 SINGLEFS_HEAVY_TESTS 前缀（服务替它排队、定额度、挑机器、分卡；套了包装的由服务当场拒，钩子不另判），」（`.claude/hooks/heavy-test-guard.sh:322`）、「#      主 agent 交什么都放（什么时候跑重型照旧是规矩：提交时或用户要求时，钩子不再凭前缀判）。」（`.claude/hooks/heavy-test-guard.sh:323`）。实测 C07：主 agent 不带前缀经 submit 交 checker 档测试，钩子退 0；D14：子 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 交 harness 测试，钩子也退 0（白名单带得进去，不拒）。
- 脚本自己要：「前缀：起 cargo 的与改库目录的（跑、--full、--refresh-list、--import-results）都要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，不带退 2；」（`scripts/mutation/broad-mutation.py:233`）；「起它的命令要带 SINGLEFS_HEAVY_TESTS=user-request（提交时 =commit）：没带就不起、记一项红，出路写带前缀重跑的写法。」（`scripts/gate/merge-and-verify.sh:102`）。子 agent 与主 agent 的区分也靠它：`scripts/mutation/prove-red.sh` 里「没设 SINGLEFS_HEAVY_TESTS（子 agent 起的）时照样跑重型的行」是它的弄坏开关说明（`scripts/mutation/prove-red.sh:20`）。
- 结论：M07 在这 21 份里事实上被定成「要」（七份用「带」，一份用方括号），与钩子文字「不要」对不上，与几个脚本自己的判定一致。该由主 agent 定：定「要」，则 `.claude/hooks/heavy-test-guard.sh:322-323` 与 2187 行（「经 task-server 提交的不要这个前缀」）要改；定「不要」，则上面七处与两个脚本的前缀检查要改。
- 推翻它的现象：用户把 M07 定成「要」且接受钩子文字（钩子只是不判，不是禁止带）——那七处就是对的，只剩 agent-common 第 59 行的方括号要去掉。

### 4.3 M09（GPU 一件一张卡还是全部卡）——没被定；措辞偏复数，服务今天是一件一张

- 定义侧各处都只写「由服务分」：「内存上限、线程、哪台机器、哪几张卡、GPU 清场、排队次序、分片、卡住、撞上限放大重排都由服务定，调用方不写、不判」（`.claude/agent-common.md:59`）；「哪几张卡由服务分；派活（哪件上哪张卡）与判法拆开」（`.claude/rules/verification.md:68`）；「用哪几台机器、哪几张卡、各给多少额度由统一任务服务（`task-server/`）分，调用方不写、不挑。」（`.claude/rules/test-script-discipline.md:89`）；`.claude/rules/verification.md:66` 的「单个 GPU 测试二进制只用分到的卡（哪几张卡由服务分）」、`.claude/main-agent.md:98` 的「哪几张卡由服务分」、`.claude/skills/crash-test/commit-stages.md:19` 的「清场与分卡由服务做」。
- 服务今天：「       挑优先级最高、此刻没有服务起的任务占着、所在那台又放得下的一张，经环境 GPU_CARDS_BY_PRIORITY=<台>:<序号>:<额度> 交给命令；一张都没点名的记 run-condition-unmet。」（`task-server/task_server.py:56`）。
- 结论：没有哪一份把 M09 悄悄定成「一件一张」或「全部卡」。但「哪几张卡」「分到的卡」这两种写法读起来是复数，服务今天挑的是一张；M09 定「一件一张」后这几处该改成「哪张卡」，定「全部卡」则服务要改。
- 推翻它的现象：某份定义明写了「一件用一张」或「用全部卡」。我在 21 份里 `grep` 了 `一张卡|全部卡|所有卡|每张卡|两张卡|一件一|独占`，只有上面这些「几张」「分卡」的命中。

## 五、实测记录（把定义原文照字面喂给钩子；脚本与原样输出）

喂的是此刻的 `.claude/hooks/heavy-test-guard.sh`（cc9fd871）、`.claude/hooks/lib_heavy_tests.py`（cc11e40f）、`.claude/hooks/lib_shell_words.py`（f3a60c51）；脚本打出来的前三行就是这三份的 sha8（`task log` 原样）。钩子自己不起命令，只判。agent_type 的写法照派发闸的输入（子 agent 带 `agent_type`，主 agent 不带）。每条的退出码（rc）是钩子的退出码：2 是拒，0 是放。

复跑（非重型，主 agent 也可以自己交；草稿目录 `~/.local/share/singlefs/work/d137-ts-defs-r1-forward/`）：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/d137-ts-defs-r1-forward -- python3 ~/.local/share/singlefs/work/d137-ts-defs-r1-forward/feed.py`（C01–C27，任务 1784，sha256 `feed.py` 0b797c8e35e333665c392d7cde07f308278b7cbbddc54d2fb21a79ffcca88bfc）
- 同上换 `feed2.py`（D01–D16，任务 1785，sha256 92c638200c54cf6a9bf7ab501e0c96e7e086b6e4d0f8fac57a29b05d920f9837）与 `feed3.py`（E01–E12，任务 1793，sha256 8d2e74c41c0c89d100fbc3646366329909ed80153843a4550cb2b8b07b1d2dae）。
- 打印 `rerun-failed-tests.py` 输出（任务 1792）：`task-server/task submit --cwd ~/.local/share/singlefs/work/d137-ts-defs-r1-forward -- python3 <仓根>/scripts/gate/rerun-failed-tests.py ~/.local/share/singlefs/work/d137-ts-defs-r1-forward/x.log -p singlefs-harness`（`x.log` sha256 6d4b9c306c6c709105bfa2860639c290e8851e8425273b43b3cca75fccca3069）。
- 看 `TASK_SERVER_RESULT` 末行 `type=succeeded exit=0`，再 `task-server/task log <号> --all`，每条 `==== <编号> … rc = <码>`。输出落盘副本：草稿目录的 `feed1-out.txt`（a4ee363d…）、`feed2-out.txt`（053f7870…）、`feed3-out.txt`（92fd5010…）、`rerun-out.txt`（cb75c8d7…）。这些是检查日志，不是实验产物，所以没入库：关键行已整行抄进上面各 F 项，原样输出留在上面这个仓外路径与服务的任务日志里。
- 我没有写模型，没有模型目录与 `rerun.sh`。（任务 1783 是第一次提交，被派到第二台失败，见 F17，不算数。）

| 编号 | 谁 | 命令（缩写） | rc | 对应 |
|---|---|---|---|---|
| C01 | 子 agent | 直接 `proc.py stop 12345` | 2 | F4 |
| C02 | 主 agent | 直接 `proc.py stop 12345` | 0 | F4 |
| C03 | 子 agent | 直接 `proc.py wait 12345 --timeout 5` | 2 | F4 |
| C04 | 子 agent | `cargo fmt --all -- --check` | 0 | 1.1 第 9 条 |
| C05 | 子 agent | 带声明交 harness 测试（`--cwd` 副本） | 0 | 1.1 |
| C06 | 子 agent | 带 `SINGLEFS_HEAVY_TESTS=commit` 交 checker 档测试 | 2 | F8（拒的是钩子） |
| C07 | 主 agent | 不带前缀交 checker 档测试 | 0 | F6 |
| C08 | 主 agent | 带前缀直接 `cargo test -p singlefs-checker-tier` | 2 | 钩子 :314 |
| C09 | 三方攻方 | 直接执行编出来的二进制 | 2 | F16 |
| C10 | 三方攻方 | 交编出来的二进制 | 0 | F16 |
| C11 | 子 agent | 直接 `rerun-failed-tests.py` | 2 | F2 |
| C12 | 子 agent | 交 `rerun-failed-tests.py` | 0 | F2 |
| C13 | 子 agent | 交 `prove-red.sh --copy …` | 0 | 1.3 |
| C14 | kb-scribe | 交 `env AGENT_HOOK_DETECTIONS=… bash -c 'bash …write-guard.sh < …'` | 0 | 1.1 第 3 条 |
| C15 | kb-scribe | 不经 submit 直接喂 | 2 | 1.1 第 3 条 |
| C16、C17 | 子 agent | `task log 5 --all`、`task clean 5` | 0、0 | 1.1 第 9 条 |
| C18 | 子 agent | `cite-check.py` | 0 | 1.1 第 9 条 |
| C19、C20 | 子 agent | 命名检查两条（带声明交 / 交） | 0、0 | 1.1 第 5 条 |
| C22 | 子 agent | 直接 `python3 task-server/control check` | 0 | 1.1 第 9 条（钩子多放） |
| C24 | 工具员 | 交 `gate-staged.sh` | 2 | 钩子分类（整轮门禁，子 agent 不交） |
| C25 | 工具员 | 直接 `gate-overlap.py --list` | 0 | 1.1 第 9 条 |
| C26 | 执行员 | 交 `mutate.sh`，`--cwd scripts/experiment-apparatus` | 0 | 1.3 |
| C27 | 子 agent | 交 `-p singlefs-checker-tier --test …` | 2 | F8 |
| D01 | 主 agent | 直接 `doc-decisions.sh --write --check decision-items-sync` | 2 | F10-1 |
| D02 | 主 agent | 同上经 submit | 0 | F10-1 |
| D03 | 主 agent | 直接 `crash-point-replay.sh --list-items --backlog --milestone m4` | 2 | F10-9 |
| D04 | 主 agent | 直接 `apply-writer-patch.py` | 2 | F10-2 |
| D05 | 主 agent | 直接 `bash .claude/scripts/env.sh` | 2 | F10-8 |
| D06 | 主 agent | 直接 `broad-mutation.py --unit-environment` | 2 | F10-10 |
| D07 | 主 agent | 直接 `…shard-configuration-check.sh --emit-assignments` | 2 | F10-11 |
| D08 | 主 agent | `watch.sh` | 0 | F10 对得上的 |
| D09、D10 | 子 agent | 直接 / 交 `crates-mutation-rows.py static-check` | 2、0 | 1.3 |
| D11 | 设计员 | 直接 `claim-experiment.sh --next` | 0 | 1.1 第 9 条 |
| D12 | 主 agent | 直接 `admission.py experiment` | 2 | F10-4 |
| D13 | kb-scribe | 直接 `kb-spec-check.py` | 0 | 1.1 第 9 条 |
| D14 | 子 agent | 带 `SINGLEFS_HEAVY_TESTS=user-request` 交 harness 测试 | 0 | F6 |
| D15 | 子 agent | 交一个没登记的 `--selftest` | 2 | 钩子分类（符合 `.claude/agent-common.md:103`，不算问题） |
| D16 | 子 agent | 交 `test-targets-parallel.py` | 0 | 1.3 |
| E01 | 主 agent | 直接 `quote-kb.py` | 2 | F10-7 |
| E02 | 主 agent | 直接 `snapshot-tree.py` | 2 | F10-5 |
| E03 | 主 agent | 直接 `agent-handover.py` | 2 | F10-6 |
| E04 | 主 agent | 直接 `stage-mine.py` | 2 | F10 |
| E05、E06 | 主 agent | 直接 `cite-check.py`、`kb-sections.py` | 0、0 | F10 对得上的 |
| E07 | 主 agent | 直接 `three-way-materials.py --help` | 2 | F10-3 |
| E08 | 工具员 | 交 `env LIB_HEAVY_TESTS_BREAK=… python3 …/lib_heavy_tests.py --selftest` | 0 | 1.1 第 3 条（弄坏开关的写法） |
| E09、E10 | 工具员、调查员 | `py_compile`、`rsync` 副本 | 0、0 | 1.1 第 9 条 |
| E11、E12 | 子 agent、主 agent | `task-server/task submit -- task-server/task submit -- cargo …`（二次 submit） | 0、0 | F2 |

（C21 我喂的 `rerun.sh` 路径不存在，钩子照常放行、记一条检出，不算数；缺 C23 是 `watch.sh --selftest` 交了放行，不影响结论。）

## 六、要主 agent 跑的命令

没有重型要跑。判决要引的数（上面第五节的 rc、`rerun-failed-tests.py` 的输出）由主 agent 复跑，三条都不重型，经服务交：

1. `task-server/task submit --cwd ~/.local/share/singlefs/work/d137-ts-defs-r1-forward -- python3 ~/.local/share/singlefs/work/d137-ts-defs-r1-forward/feed.py`；起点目录：草稿目录；看 `TASK_SERVER_RESULT` 末行 `type=succeeded exit=0`，再 `task log <号> --all` 里 `==== C01 … rc = 2`、`==== C02 … rc = 0`、`==== C09 … rc = 2`、`==== C11 … rc = 2` 这几行。
2. 同上换 `feed2.py`：看 `==== D01 … rc = 2`、`==== D03 … rc = 2`、`==== D04 … rc = 2`、`==== D12 … rc = 2`。
3. 同上换 `feed3.py`：看 `==== E01 … rc = 2`、`==== E02 … rc = 2`、`==== E03 … rc = 2`、`==== E07 … rc = 2`、`==== E11 … rc = 0`。
4. `task-server/task submit --cwd ~/.local/share/singlefs/work/d137-ts-defs-r1-forward -- python3 <仓根>/scripts/gate/rerun-failed-tests.py ~/.local/share/singlefs/work/d137-ts-defs-r1-forward/x.log -p singlefs-harness`：`task log <号> --all` 第二行是否含 `task-server/task submit -- cargo test`。
5. （F13，一行，客户端当场拒，不排队）`task-server/task submit --cwd ~/.local/share/singlefs/work/d137-ts-defs-r1-forward -- env SINGLEFS_THREAD_CAP=2 true`：退 3，stderr 含「线程一律是登记的每台默认重活线程数」。

注意：这三份脚本喂的是此刻磁盘上的钩子（读 `lib_heavy_tests.py` 等），别的会话换钩子之后 rc 可能变；脚本打出的前三行 sha8 就是用来对这个的。要用草稿目录当 `--cwd`（不要给仓根，见 F17），脚本路径写字面的（子 agent 提交时变量写的路径被钩子拒，见 F9；主 agent 不判）。

没交、写给主 agent 的（不碰 GPU、不起真编译）：F12 要看 `task-server/task status <号>` 对一条带 `--features gpu --no-run` 的 cargo 报「要卡」还是「不要卡」；F3 要看一次真编译的 `task wait` 末行 `build=` 是不是以 `/target` 结尾；F2 的二次 submit 要真交一条看结局。三条都会起真任务，不重型但会占卡 / 起 cargo，没交。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 submit 写法与 `--cwd` | 对得上 | `.claude/agent-common.md:59` 与 `task-server/task:4`、`:5` 逐字一致；11 份定义用同一句型 |
| 1 范围声明与前缀写在 submit 前面 | 对得上 | 21 份里零处写在后面；`task-server/README.md:28` 同 |
| 1 白名单外的环境变量 | 对得上 | 写成 `-- env <名>=<值>`，C14、E08 钩子放行；`SINGLEFS_THREAD_CAP` 除外见 F13 |
| 1 `task wait` 末行（命名检查的 `exit=0/1`） | 对得上 | `.claude/agent-common.md:104` 与 `.claude/hooks/handback-guard.sh:63` 一致 |
| 1 `task wait` 末行的 `type=` 读法 | 对不上 | F5：experiment-runner「非 succeeded 输出不算产物」与 mutate.sh 有没红退 1、与两份 skill 把 command-failed 当有结果矛盾 |
| 1 `build=` 指哪层 | 对不上 | F3：`build=` 是 `target` 的上一层，experiment-runner 当成 `target` 本身 |
| 1 卡住怎么判 | 对不上 | F7：`.claude/agent-common.md:73` 漏了宽限与「CPU 也没涨」 |
| 1 `task log` 默认只给最后 200 行 | 部分对得上 | F11：汇总行够；要整份时多数定义没写 `--all` |
| 1 `task clean` 触发 | 对得上 | 「`build=` 的用完 clean」与 `task-server/task:157` 一致 |
| 1 `rerun-failed-tests.py` 打出的命令 | 对不上 | F2：脚本打出的已带 `task-server/task submit --`，三份定义还让再插一次；脚本自己子 agent 直接跑被拒 |
| 1 直接跑名单（共用约束 59 行） | 对得上 | 定义列的每一样都在钩子 `.claude/hooks/heavy-test-guard.sh:317-320` 的放行名单里 |
| 1 直接跑：`proc.py`（共用约束 63、77、82 行） | 对不上 | F4：子 agent 直接跑被拒（C01、C03），主 agent 放（C02），出路无解 |
| 1 直接跑：主 agent 的若干本仓脚本 | 对不上 | F10：十一处（外加 stage-mine.py）定义写直接跑、钩子拒，钩子给了能抄的 submit |
| 1 直接执行编出来的二进制（攻方腿） | 判不了 | F16：字面被拒（C09），换读法（交二进制，C10）对得上 |
| 1 服务当场拒清单 | 对得上（轻微缺） | F13：定义漏 `SINGLEFS_THREAD_CAP`；`-- env SINGLEFS_THREAD_CAP=…` 被客户端拒（R2） |
| 1 「服务对子 agent 交的重型当场拒」 | 对不上 | F8：拒的是钩子，服务不分主 agent / 子 agent |
| 1 被拒之后怎么办 | 对不上 | F9：钩子对「认不出的写法」给的出路是交回，共用约束 74 行与用户原话是改写法重交 |
| 1 GPU 手跑先交 `--no-run` | 对不上（推的） | F12：服务对 `--no-run` 没有例外，只编的那条照样要卡 |
| 1 `--cwd` 给仓根可能被派去第二台 | 判不了 | F17：定义没覆盖；本次会话实测一次失败 |
| 1 子 agent 与主 agent 同一个会话号 | 判不了 | F18：定义没覆盖，`task status` / `cancel` / `clean` 能碰别人的号 |
| 2 M01 落进各份 | 大体一致，五处不一致 | F1（命名检查）、F8、F9、F14（实现员重型默认不交回）、F15（耗时类的量也交回） |
| 3 四个条名 | 对得上 | 四条都在共用约束里，引的小节名与加粗句首都对；没有引到已删的条名 |
| 3 条名之外的旧说法 | 对不上 | F1：experiment-runner 42、59 与 tooling-writer 35 还写「命名检查交主 agent」 |
| 4 M05 | 被定了 | 「提交时不写线程、一律登记值」落在 implementation-workflow 64、共用约束 59、主 agent 80 等，与服务一致 |
| 4 M07 | 被悄悄定了 | F6：七份写「带前缀」，共用约束 59 行方括号、61 行又写「带」，钩子文字写「不要」，脚本自己要 |
| 4 M09 | 没被定 | 只有「哪几张卡由服务分」，措辞偏复数，服务今天一件一张（`task-server/task_server.py:56`） |

## 八、没做什么

- 攻方腿的格（找漏洞）不判，不替主 agent 采纳或出判决；没读禁读清单里的任何文件（攻方腿的报告与模型目录）。`task status` 无号时列出了别的腿提交的任务，我没读它们的命令之外的任何东西。
- 重型没跑，没起 cargo，没碰 GPU；判决要引的数没复跑（命令在第六节，三条都非重型，经服务交）。
- 没实交、因而标「推的」的：F2 二次 submit 到服务之后的结局、F3 `build=` 的真实值、F12 带 gpu 特性的 `--no-run` 任务是否报「要卡」、F8 里「重型走钩子看不见的路到服务照排照跑」。
- 没比对：`.claude/hooks/runner-dispatch-guard.sh`（`grep -n 'task-server\|task submit'` 零命中，对照不上，所以派发闸那一大段 `.claude/agent-common.md:95` 我没逐句比）；`.claude/hooks/bash-command-detector.sh` 只比了 ⑩ 一条；`.claude/agent-common.md` 的「执行前拒绝的写法」其余各条（①–⑮ 里除 ⑩ 外）没逐条对钩子；`.claude/rules/test-script-discipline.md` 只核了 43、89 两处与任务服务有关的句子。
- 没比对 `run-scope-guard.sh`（不在对照清单）：定义里「每一段各带自己的范围声明，不用一份 export」（`.claude/agents/implementation-writer.md:39`，前面是「验证命令里 clippy、build 与动到的测试二进制合成一条交」）经 submit 之后范围声明怎么认，我判不了。
- 没核主 agent 在中途改的三份（agent-common、commit-stages、lib_heavy_tests）里除我引的行以外有没有别的变化。
- 中途发现的一个服务现象不属本格，只报：没有记录的命令先记 4 GiB 外加 2 GiB 余量，几个草稿小脚本（本次的喂钩子脚本）因此在队里等了很久才排上（`task status` 的「在等：…」原样）。
