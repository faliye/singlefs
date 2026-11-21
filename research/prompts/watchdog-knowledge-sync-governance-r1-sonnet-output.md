# 看门狗知识同步提醒治理文件轮第一轮：正推腿报告

轮名 watchdog-knowledge-sync-governance-r1，正推腿（第 1 到 4 问）。方案按 `crates/` 与这些治理文件今天的实现来谈；这一轮没有改 `crates/`，被判六份文件里也没有读 `crates/` 的行（量的：`grep -n "crates/" scripts/agents/agent-watch.py` 命中 105、513、2863 三行，120 到 139 行与 1463 到 1734 行各数得 0 处）。

## 读到的版本

被判六份读主工作区此刻那一版（量的，`sha256sum`，HEAD 是 `4319aa2b0`；`git diff HEAD --stat` 六份共 728 行加、23 行减）：

| 文件 | sha256 |
|---|---|
| scripts/agents/agent-watch.py | 1c46829bc927f97c4cb1a2dd0fc488bfbdd2a1c7d192325f9173f237f898719a |
| scripts/agents/watch.sh | 87a4dcbfa0998e39ec89b2820ab75c85481c8cf5a33470a96bc8734a65ee1d6d |
| scripts/agents/watch.conf | f5c737d3a36c5cc68e41bde8eb19b2e8e6411d2aa6f01e4491f335cf40ea243c |
| .claude/skills/dispatch/SKILL.md | 417aa5393bf0c0183bacc51c96369d30859591a0a3e183c8ea80285d7b8400a6 |
| .claude/hooks/selftest-registry.tsv | b8e4d774264dddf1e118c299ef11f63c8025602e66996ecc027bf1c2e8334168 |
| .claude/gate.d/break-switch-replay-files.tsv | 4b818439bbbaab9c73f48d8677f99d8a7c1cd68fd35968303691f1b762ccda61 |

## 模型与复跑

我写了一份探针（不是独立模型：它直接调被测模块里的函数，把几个边界各造一遍 git 操作后打出原样结果）。没有跑过：我不交任务、也不直接起项目代码（共用约束「子 agent 不交任务」一条），下面「要主 agent 跑的命令」一节给整条命令。

| 文件（相对仓根） | sha256 |
|---|---|
| research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-model/probe/section_probe.py | 4661bd5ab9d4f81bcc76cb079e6c9bbc45a2f4625a526d66869d31c184fa3ae6 |
| research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-model/tools/rerun.sh | b7981c8f3cf15edb359c998dd65c7e12a1ff5f42623ae1e0247c67d485363ea9 |
| research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |

复跑命令（整条原样、在哪个目录起、看哪几行，见末尾「要主 agent 跑的命令」一节；骨架是 `bash <模型目录>/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录，工作根下、以 -run 收尾> <模型目录>`）。

`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来、只改了 `rerun_steps`（它自带的 `--selftest` 测的是模板原来的步骤，改后不再适用，不拿它当自证）。语法检查：三份 `bash -n`、`py_compile` 都过（量的，已删 `py_compile` 留下的缓存目录）。探针每格在临时小仓里造几条 git 操作，十格共约八十条 git，只读本仓的脚本；耗时与内存都是推的（秒级、几十 MiB，没量过）。

## 先说打中的（不成立或推不出的，细节在各问里）

1. 范围表第八项 `.claude/singlefs-ai-sop/rules/` 在本仓里永远点不到名：本仓 `.gitignore` 忽略这个目录、`git ls-files` 在它底下零个文件，`git status --short` 与 `git diff --name-only` 都看不见它的改动；文件头、技能那一段与自证都写成它会被点名（第 1 问 H5、第 3 问技能段）。不成立。
2. 提醒句与「→」写的「份」不是文件数：`git status --short` 把整个未跟踪目录写成一行，代码按路径键数，一个目录算一份；改名行拆成两头各取在范围里的；「已不在」那一类也算一份（第 1 问 H3）。推不出。
3. 技能那一段只列三种来源标签，漏了第四种「起来时未提交的改动已不在 git status 里」；它写的「照末尾「→」那一行」在带子 agent 的定时回报里指到重起看门狗那一行，不是这一节的「→ 同步」（第 3 问）。不全、含糊。
4. `break-switch-replay-files.tsv` 里 agent-watch.py 那一行写「另六条的点名改过之后还没再复跑」，与任务 20085 的结局行（100 条对上 100 条）对不上，已过时（第 3 问）。
5. 用户要的「最新的 git status」：这一节只给三个计数与「相对起来时的增量」，没有此刻 `git status --short` 的路径清单；清单只在「没查成」时以命令形式给（第 4 问）。部分落到。
6. 多做的：第八项范围、内容摘要读盘、起来时基线加增量比、`--repository-report-lines` 与 `--repository-root` 两个选项与三个环境变量（第 4 问）。记录里用户原话没有这些。
7. 十七条开关：每条写的「弄坏哪一处」都是代码里那一处，点名的格都在 `selftest_repository_report` 里，我静态推出的红格与文件头点名的逐条相同（17 条对 17 条，推的）；任务 20085 的结局行也是 100 对 100（量的，读了任务日志）。没打中，但有三处自证覆盖的缺口（第 2 问末）。

## 第 1 问：文件头「知识同步提醒」每一句，对函数

判定写「成立」「不成立」「推不出」；行号都是文件头那一段在 `scripts/agents/agent-watch.py` 里的行（120 到 139），代码行同一份文件。没起脚本，都是读代码（推的），要机器核的在探针里、见「要主 agent 跑的命令」。

| 句 | 判定 | 依据 |
|---|---|---|
| H1 退 4、两路、位置：定时回报之后、出路句之前 | 成立 | 文件头 「那一行之后、重起看门狗的出路句之前打一节」（`scripts/agents/agent-watch.py:120`）；`--processes-only` 路 「print_repository_section(repository_baseline, arguments)」（`scripts/agents/agent-watch.py:2205`）在「定时回报」那行之后、出路句之前，「return 4」（`scripts/agents/agent-watch.py:2208`）；`--agents` 路同理（2705、2706、2707、2708 行）。出路句由 `print_active_list` 打，没有还在跑的子 agent 时它什么都不打（「if not active_ids and not unstarted_ids:」（`scripts/agents/agent-watch.py:2437`）），那时「之前」落空 |
| H2 同一个函数 repository_status_section 出 | 成立 | 「同一个函数 repository_status_section 出，次序：」（`scripts/agents/agent-watch.py:121`）；两路都经 「for line in repository_status_section(baseline, arguments):」（`scripts/agents/agent-watch.py:1733`） |
| H3 提醒句起头；变过的写几份、一份没有的写没变 | 句子成立；「份」的单位推不出 | 「这一节以一句写给主对话的提醒句起头」（`scripts/agents/agent-watch.py:122`）；提醒句 「知识同步提醒：到点了，该同步项目最新变动与状态了。」（`scripts/agents/agent-watch.py:1680`）；「知识类文件变过 {len(knowledge)} 份」（`scripts/agents/agent-watch.py:1696`）、「知识类文件没变，没有要重读的。」（`scripts/agents/agent-watch.py:1698`）；次序 「return [reminder] + named + lines + [arrow]」（`scripts/agents/agent-watch.py:1729`）。`len(knowledge)` 是 `knowledge_files_changed` 返回的行数，行是路径键：未跟踪整目录一行、改名取在范围里的那头、「已不在」也一行，所以「几份」不等于变过几个文件（探针「untracked-directory」「rename-across-scope」打出来看） |
| H4 逐份点名、各标来源；再「仓的现状」；末尾「→」 | 成立 | 「随后逐份点名变过的知识类文件、各标来源」（`scripts/agents/agent-watch.py:123`）；「named = rows_with_remainder(」（`scripts/agents/agent-watch.py:1699`）；「lines = [f"仓的现状（{root}）："]」（`scripts/agents/agent-watch.py:1701`）；分支与 HEAD、新提交、计数、别的路径依次 append（1703 到 1719 行） |
| H5 范围八项（表 KNOWLEDGE_SCOPE） | 表成立；效果不成立 | 「知识类文件的范围写在表 KNOWLEDGE_SCOPE 里」（`scripts/agents/agent-watch.py:124`）；表 「KNOWLEDGE_SCOPE = ("CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md", ".claude/rules/", ".claude/skills/", ".claude/agents/",」（`scripts/agents/agent-watch.py:535`）、「".claude/kb/", ".claude/singlefs-ai-sop/rules/")」（`scripts/agents/agent-watch.py:536`）八项逐字对得上。但第八项在本仓看不见：下面「第八项的实测」 |
| H6 点名先后（表 KNOWLEDGE_PRIORITY） | 成立 | 「点名的先后写在表 KNOWLEDGE_PRIORITY 里」（`scripts/agents/agent-watch.py:125`）；表 「KNOWLEDGE_PRIORITY = (".claude/kb/milestone/", ".claude/kb/decisions.md", ".claude/kb/checks-owed.md", "CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md")」（`scripts/agents/agent-watch.py:538`）；不在表里的 「return (len(KNOWLEDGE_PRIORITY), path)」（`scripts/agents/agent-watch.py:1515`）排后、同档按路径 |
| H7 变过取两处并集，不分提交没提交 | 成立 | 「变过的判法取两处的并集，不分提交没提交」（`scripts/agents/agent-watch.py:126`）；提交一半 「changed_paths = run_git(root, ["diff", "--name-only", "--no-renames", start_snapshot.head_hash, "HEAD"]).splitlines()」（`scripts/agents/agent-watch.py:1598`）；工作区一半 「if scoped_parts and (start.status_by_path.get(key) != code or digest_changed):」（`scripts/agents/agent-watch.py:1618`）、不在了 「dropped.update(part for part in status_paths_of(key) if in_knowledge_scope(part))」（`scripts/agents/agent-watch.py:1624`）。「新出现、状态变了」都落在 `!= code` 一条里（起来时没有该键，`get` 得 None） |
| H8 摘要：范围里的路径起来时记；文件 sha256、整目录取底下每个文件、不存在写「不在」；范围外只比状态码 | 成立，两处边角推不出 | 「文件取内容的 sha256，git status 写成一行的整个未跟踪目录取底下每个文件的相对路径与摘要，不存在写」（`scripts/agents/agent-watch.py:129`）；只记范围里的 「for key in status_by_path if digest_wanted(key)}」（`scripts/agents/agent-watch.py:1567`）。边角一：符号链接取的是链接目标字符串，不是内容 sha256 「return "链接 " + os.readlink(path)」（`scripts/agents/agent-watch.py:1527`）。边角二：整目录走 「for directory, subdirectories, file_names in os.walk(full_path):」（`scripts/agents/agent-watch.py:1545`），`os.walk` 不跟随链接，目录里指向目录的符号链接既不在 `file_names` 里也不被下探，链接改指别处摘要不变（探针「symlink-in-untracked-directory」打出 `digest_equal`） |
| H9 来源标签四种 | 成立 | 「来源标签：…新提交动到…工作区改了没提交…两样都有…起来时未提交的改动已不在 git status 里」（`scripts/agents/agent-watch.py:130`）；四个常量逐字相同：「COMMITTED_KNOWLEDGE_TAG = "新提交动到"」（`scripts/agents/agent-watch.py:539`）、「UNCOMMITTED_KNOWLEDGE_TAG = "工作区改了没提交"」（`scripts/agents/agent-watch.py:540`）、「BOTH_KNOWLEDGE_TAG = "两样都有"」（`scripts/agents/agent-watch.py:541`）、「DROPPED_KNOWLEDGE_TAG = "起来时未提交的改动已不在 git status 里"」（`scripts/agents/agent-watch.py:542`）；贴标签的分支 1627 到 1636 行：提交与工作区都有标「两样都有」，只在 `dropped` 里才标第四种，所以「新提交又没动到它」成立 |
| H10 「仓的现状」：新提交逐条、没有写没有；计数 | 成立；「几个路径」同 H3 的单位问题 | 「里：新提交逐条」（`scripts/agents/agent-watch.py:131`）；「commit_rows = run_git(root, ["log", "--format=%h %s", f"{start_snapshot.head_hash}..HEAD"]).splitlines()」（`scripts/agents/agent-watch.py:1597`）；「没有新提交{moved}」（`scripts/agents/agent-watch.py:1709`）；计数从两位状态码现数：「staged = 0 if BROKEN_DETECTION == "repositorystagedunread" else sum(1 for code in codes if code[0] not in " ?!")」（`scripts/agents/agent-watch.py:1492`）、「untracked = sum(1 for code in codes if code == "??")」（`scripts/agents/agent-watch.py:1494`）；一个路径两边都有（`MM`）两边各算一次，成立。解析 「return {row[3:]: row[:2] for row in text.split("\n") if len(row) >= 4}」（`scripts/agents/agent-watch.py:1485`）按路径键去重，一个未跟踪整目录就是一个「路径」 |
| H11 别的路径只列范围外的；没变写「没变」；末尾「→」 | 成立 | 「变了的别的路径只列范围外的」（`scripts/agents/agent-watch.py:132`）；「return not any(in_knowledge_scope(part) for part in status_paths_of(path))」（`scripts/agents/agent-watch.py:1648`）；「看门狗起来之后变了的别的路径：没变」（`scripts/agents/agent-watch.py:1719`）；出路 「→ 同步：先重读上面点名的」（`scripts/agents/agent-watch.py:1724`）、「→ 同步：知识类文件没变，不用重读；」（`scripts/agents/agent-watch.py:1726`） |
| H12 `--repository-report-lines`：默认 20、每张清单最多列几条、余下报条数、写 0 不打也不记基线 | 成立；「watch.conf 的同名键」只在注释里 | 「是每张清单（点名的知识类文件、新提交、变了的别的路径）最多列几条，余下的报条数」（`scripts/agents/agent-watch.py:133`）；「REPOSITORY_REPORT_LINES_DEFAULT = 20」（`scripts/agents/agent-watch.py:532`）；「return list(rows[:shown]) + [f"    …余 {len(rows) - shown} {unit}没列：{full_command}"]」（`scripts/agents/agent-watch.py:1667`），三张清单都经它；写 0：「if arguments.repository_report_lines <= 0 and BROKEN_DETECTION != "repositorylineszero":」（`scripts/agents/agent-watch.py:1580`）不记基线、「if baseline is None or BROKEN_DETECTION == "repositorysectionoff":」（`scripts/agents/agent-watch.py:1676`）返回空。`watch.conf` 里这个键只有注释行（「repository-report-lines 20」（`scripts/agents/watch.conf:27`）），没有键值行，键值行只有 5 个，最后是 「root-free-gib-alert=128」（`scripts/agents/watch.conf:34`）；默认 20 来自脚本，不来自配置 |
| H13 只在定时回报里打；告警退 3、全部交回退 0、report 不打；退出码不因这一节改变 | 打印位置成立；「退出码不因这一节改变」在一处推不出 | 「只在定时回报里打：告警退 3、全部交回退 0、report 那几路不打，退出码不因这一节改变。」（`scripts/agents/agent-watch.py:134`）；基线只在 watch 起：「take_repository_baseline(arguments) if arguments.mode == "watch" else None」（`scripts/agents/agent-watch.py:2159`、`scripts/agents/agent-watch.py:2658`）；无弄坏开关的打印点只有 2205、2706 两处（`grep -nF "print_repository_section(repository_baseline, arguments)"` 命中 2191、2205、2693、2700、2706，其中 2191、2693、2700 在 `repositoryonalert`、`repositoryonfinish` 的 `if` 里）。推不出那处：基线只接 `RepositoryUnreadable` 与 `SessionDirectoryError`（「except RepositoryUnreadable as error:」（`scripts/agents/agent-watch.py:1588`）），而每条 git 的时限 「timeout = float(os.environ.get(GIT_TIMEOUT_VARIABLE) or GIT_TIMEOUT_SECONDS_DEFAULT)」（`scripts/agents/agent-watch.py:1467`）读环境变量，值不是数字时 `float` 抛 `ValueError`，起来时就没人接，看门狗起不来；该变量文件头写「只给 --selftest 用」，所以只在有人误设时才会碰到 |
| H14 读哪个仓：`--repository-root`，缺省 TASK_SERVER_SOURCE_REPOSITORY，没有取脚本所在的仓；与 lib_session_directory.py 同一个来源；两台同一个值 | 取法成立；「两台同一个值」是推的 | 「读哪个仓：--repository-root；缺省取统一任务服务给的来源仓根 TASK_SERVER_SOURCE_REPOSITORY」（`scripts/agents/agent-watch.py:135`）；「return arguments.repository_root or session_directories.source_repository() or REPOSITORY_ROOT」（`scripts/agents/agent-watch.py:1574`）；`REPOSITORY_ROOT` 是脚本往上三层 「REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))」（`scripts/agents/agent-watch.py:549`）；会话目录同取 `source_repository`：「return os.path.join(projects_root(environment), mangle(repository or source_repository(environment) or own_repository))」（`scripts/agents/lib_session_directory.py:125`）。「两台同一个值」：代码不管，靠共享根两台同一绝对路径（「两台挂在同一绝对路径」（`.claude/agent-common.md:30`）），推的 |
| H15 查不成：不是仓、git 起不来、超 30 秒；提醒句照写并写没查成；一行「没查成：…」与命令；退 4；标准错误第一行进原因；只读；三个 GIT_ 变量不继承 | 成立，一处措辞偏差 | 「git 查不成（不是仓、git 起不来、一条命令超过 30 秒）」（`scripts/agents/agent-watch.py:136`）；三种都抛 `RepositoryUnreadable`：「raise RepositoryUnreadable(f"{shown} 超过 {timeout:g} 秒没有返回，被停掉了")」（`scripts/agents/agent-watch.py:1474`）、「raise RepositoryUnreadable(f"git 起不来（{error}）")」（`scripts/agents/agent-watch.py:1476`）、非 0 退出码一条（`scripts/agents/agent-watch.py:1479`，取标准错误第一个非空行）；「GIT_TIMEOUT_SECONDS_DEFAULT = 30」（`scripts/agents/agent-watch.py:531`）；输出 「仓的现状：没查成：」（`scripts/agents/agent-watch.py:1691`）与 「→ 自己看：git -C {root} status --short；git -C {root} log -5 --format='%h %s'」（`scripts/agents/agent-watch.py:1692`）；「GIT_OPTIONAL_LOCKS=0，不刷新索引；GIT_DIR、GIT_WORK_TREE、GIT_INDEX_FILE 不继承」（`scripts/agents/agent-watch.py:137`）对应 「environment["GIT_OPTIONAL_LOCKS"] = "0"」（`scripts/agents/agent-watch.py:1469`）与 「GIT_ENVIRONMENT_DROPPED = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE")」（`scripts/agents/agent-watch.py:533`）。偏差：文件头写「随后一行「没查成：<…>」」，实际那一行以「仓的现状：」起头、「没查成：」在行中间。起来时就没读到的写「看门狗起来时就没读到：…」 |
| H16 三个环境变量只给 --selftest 用 | 成立（约定，不是强制） | 「环境变量只给 --selftest 用：AGENT_WATCH_REPOSITORY_ROOT」（`scripts/agents/agent-watch.py:138`）；三个变量照读：「finished = subprocess.run([os.environ.get(GIT_EXECUTABLE_VARIABLE) or "git",」（`scripts/agents/agent-watch.py:1471`）、「--repository-root」缺省取 `AGENT_WATCH_REPOSITORY_ROOT`（`scripts/agents/agent-watch.py:4553`）；没有像弄坏开关那样挡「非自证不认」，与文件头里已有的 `AGENT_WATCH_ROOT_FREE_BYTES` 同一做法 |
| H17 「repository 起头十一条、knowledge 起头六条」 | 成立 | 「弄坏开关见下面开关表里 repository 起头的十一条与 knowledge 起头的六条。」（`scripts/agents/agent-watch.py:139`）；数：见下面命令输出 |

命令输出（量的）：

```
$ grep -c '^  AGENT_WATCH_BREAK=repository' scripts/agents/agent-watch.py
11
$ grep -c '^  AGENT_WATCH_BREAK=knowledge' scripts/agents/agent-watch.py
6
```

### 第八项的实测（量的，本仓此刻）

```
$ git check-ignore -v .claude/singlefs-ai-sop/rules/writing-discipline.md
.gitignore:8:/.claude/singlefs-ai-sop/	.claude/singlefs-ai-sop/rules/writing-discipline.md
$ git ls-files .claude/singlefs-ai-sop/ | wc -l
0
$ git ls-files | grep -c '^\.claude/singlefs-ai-sop'
0
```

`git status --short` 不列被忽略的文件，`git diff --name-only <旧 HEAD> HEAD` 只列进过提交的文件，这个目录两样都不是；所以 `.claude/singlefs-ai-sop/rules/` 底下的改动（含上游同步换上的新规矩）这一节永远不点名。自证用的临时小仓里这个目录是被跟踪的（`SELFTEST_KNOWLEDGE_FILE_SAMPLES` 里有 `.claude/singlefs-ai-sop/rules/sample-sop-rule.md`，「SELFTEST_KNOWLEDGE_FILE_SAMPLES = {」（`scripts/agents/agent-watch.py:3661`）），所以自证绿而真仓里点不到。仓里别处有叫 `singlefs-ai-sop` 的路径（门禁样本树，在 `.claude/gate.d/fixtures/` 底下），不是这个目录。

### 第 1 问：什么现象会推翻上面的判定

| 判定 | 什么现象会推翻 |
|---|---|
| H5 第八项在本仓点不到 | 上面三条命令里任何一条变样（`git ls-files` 在该目录下有文件，或 `.gitignore` 不再忽略它）；或探针「ignored-scope」打出的 `knowledge=` 不是空表 |
| H3、H10 「份」「几个路径」不是文件数 | 探针「untracked-directory」打出的 `count=` 等于 3（目录底下三个文件各一份） |
| H8 边角二：链接改指摘要不变 | 探针「symlink-in-untracked-directory」打出 `digest_equal=False` |
| H13 `ValueError` | 设 `AGENT_WATCH_GIT_TIMEOUT_SECONDS=abc` 起 `watch --agents <号> --session-dir <目录> --max-minutes 0.03` 仍正常退 4（我没造这个输入，读代码推的） |
| H14 两台同一个值 | 在第二台的任务里读到的 `TASK_SERVER_SOURCE_REPOSITORY` 与本机不是同一个绝对路径 |
| 其余「成立」 | 任一句对应的代码行被改掉；或自证 `repository_status_section` 相关格出红 |

另有三处探针也一并打出，不是对文件头某一句的判定，是边界现象，交主 agent 与攻方腿对读：

- 起来之后 HEAD 退回去（`git reset --hard HEAD~1`）：`new_commits_since` 的 「if BROKEN_DETECTION == "repositorycommitsblind" or start_snapshot.head_hash == now_snapshot.head_hash:」（`scripts/agents/agent-watch.py:1595`）只比哈希是否相同，不同就取两个 HEAD 的净差当「新提交动到的路径」，而「新提交」一栏写的是「没有新提交（HEAD 从 x 移到了 y，不是往前走）」；读下来文件会标「新提交动到」，与同一节里的「没有新提交」并存（探针「head-moved-backward」）。文件头没有写这种情形，不判不成立。
- 路径带空格或中文：`parse_status_short` 取每行第 4 个字符起当路径、不去引号（1485 行）；`run_git` 带 `core.quotepath=off`（1471 行）管中文，空格是否被 git 加引号由 git 定，探针「space-and-cjk-path」打出模块实际看到的键。读到带引号的键时，`.claude/kb/` 前缀判不中。
- 只 `git add`、内容没动：状态码变了就算「工作区改了没提交」，没有内容比对（「if scoped_parts and (start.status_by_path.get(key) != code or digest_changed):」（`scripts/agents/agent-watch.py:1618`））；只碰 mtime：状态码与摘要都不变，不点名（探针「stage-only」「mtime-only」）。

## 第 2 问：开关表十七条

做法（推的，没起自证）：对每条开关，读代码里它改的那一处，再把 `selftest_repository_report` 的每个 `check(...)` 对着这一处改后会怎样逐个过一遍，得出红格，与文件头点名的比。「点名的格在不在」用两张名单对：文件头 262 到 273 行与 326 到 353 行里出现的「知识同步提醒（…）」格名共 18 个，`check("…"` 在 3736 到 3963 行里的不同格名也是 18 个，两边求差为空（量的，见下）。

| 开关 | 代码里弄坏的那一处 | 文件头点名的格 | 判定 |
|---|---|---|---|
| repositorychangesblind | 「paths_changed_since() 一概当没变」（`scripts/agents/agent-watch.py:326`）对 「if BROKEN_DETECTION == "repositorychangesblind":」（`scripts/agents/agent-watch.py:1644`） | 有新提交与改动、清单超过上限、--processes-only 有新提交与改动 | 对。红在三只看门狗各自的「新出现…」那一条断言，推出的红格相同 |
| repositorycommitsblind | 「new_commits_since() 一概当没有新提交」（`scripts/agents/agent-watch.py:328`）对 「BROKEN_DETECTION == "repositorycommitsblind"」（`scripts/agents/agent-watch.py:1595`） | 有新提交与改动、新提交动了知识类文件、点名次序、工作区改了没提交、清单超过上限、--processes-only 有新提交与改动 | 对。工作区改了没提交红在「两样都有」那一条（提交那一半没了，标签退成「工作区改了没提交」） |
| repositoryfailuresilent | 「查不成时也不写「没查成」，这一节是空的」（`scripts/agents/agent-watch.py:331`）对 「if BROKEN_DETECTION == "repositoryfailuresilent":」（`scripts/agents/agent-watch.py:1688`） | 仓根不是仓、git 超时、git 起不来 | 对 |
| repositoryheadomitted | 「不写「分支 …，HEAD …」那一行」（`scripts/agents/agent-watch.py:333`）对 「if BROKEN_DETECTION != "repositoryheadomitted":」（`scripts/agents/agent-watch.py:1702`） | 有新提交与改动 | 对。只有这一格读分支与 HEAD 那一行 |
| repositorylimitignored | 「visible_count() 不看 --repository-report-lines」（`scripts/agents/agent-watch.py:335`）对 「return total if BROKEN_DETECTION == "repositorylimitignored" else min(total, limit)」（`scripts/agents/agent-watch.py:1659`） | 清单超过上限 | 对 |
| repositorylineszero | 「在 --repository-report-lines 写 0 时照记起来时的仓」（`scripts/agents/agent-watch.py:337`）对 1580 行那一条 | 写 0 不打 | 对 |
| repositoryonalert | 「run() 与 run_processes_only() 告警退 3 之前也打这一节」（`scripts/agents/agent-watch.py:339`）对 「if BROKEN_DETECTION == "repositoryonalert":」（`scripts/agents/agent-watch.py:2190`）与（`scripts/agents/agent-watch.py:2692`） | 告警退 3 不打 | 对。一格里两路各一次断言，同一个格名 |
| repositoryonfinish | 「run() 被看的子 agent 全部交回退 0 之前也打这一节」（`scripts/agents/agent-watch.py:341`）对 「if BROKEN_DETECTION == "repositoryonfinish":」（`scripts/agents/agent-watch.py:2699`） | 全部交回与 report 不打 | 对。红在 finished 那一条，report 那一条不红（见下面缺口） |
| repositoryruleblind | 「in_knowledge_scope() 一份知识类文件都不认」（`scripts/agents/agent-watch.py:343`）对 「return BROKEN_DETECTION != "repositoryruleblind" and any(matches_scope_entry(path, entry) for entry in KNOWLEDGE_SCOPE)」（`scripts/agents/agent-watch.py:1505`） | 新提交动了知识类文件、点名次序、工作区改了没提交、起来时已改内容又变、清单超过上限 | 对。范围外不点名不红：范围外那个文件状态码没变，本来就不在「别的路径」里 |
| repositorysectionoff | 「repository_status_section() 一概是空」（`scripts/agents/agent-watch.py:346`）对 1676 行那一条 | 十四格：有新提交与改动、新提交动了知识类文件、点名次序、什么都没变、知识类文件没变、提醒句排第一、工作区改了没提交、起来时已改内容又变、范围外不点名、清单超过上限、--processes-only 有新提交与改动、仓根不是仓、git 超时、git 起不来 | 对。写 0、告警退 3、全部交回与 report、状态码计数四格本来就没有这一节，不红，没被点名，一致 |
| repositorystagedunread | 「status_counts() 不数状态码的第一位，已暂存恒为 0」（`scripts/agents/agent-watch.py:352`）对 1492 行那一条 | 状态码计数、有新提交与改动、什么都没变 | 对 |
| knowledgedigestblind | 「不比内容摘要」（`scripts/agents/agent-watch.py:262`）对 「digest_changed = (BROKEN_DETECTION != "knowledgedigestblind"」（`scripts/agents/agent-watch.py:1616`） | 起来时已改内容又变 | 对。「工作区改了没提交」三条不红：起来时干净的、撤回的、提交又改的，靠状态码就够 |
| knowledgedigestoutofscope | 「范围外的路径也记内容摘要」（`scripts/agents/agent-watch.py:264`）对 「return BROKEN_DETECTION == "knowledgedigestoutofscope" or any(in_knowledge_scope(part) for part in status_paths_of(status_key))」（`scripts/agents/agent-watch.py:1556`） | 范围外不点名 | 对 |
| knowledgeorderflat | 「knowledge_sort_key() 一律只按路径排」（`scripts/agents/agent-watch.py:266`）对 「if BROKEN_DETECTION == "knowledgeorderflat":」（`scripts/agents/agent-watch.py:1510`） | 点名次序 | 对 |
| knowledgereminderlast | 「把提醒句放在这一节最后，不排第一行」（`scripts/agents/agent-watch.py:268`）对 「if BROKEN_DETECTION == "knowledgereminderlast":」（`scripts/agents/agent-watch.py:1727`） | 提醒句排第一 | 对。四只看门狗各一次断言，同一个格名 |
| knowledgetagsource | 「来源标签恒写「新提交动到」」（`scripts/agents/agent-watch.py:270`）对 「if BROKEN_DETECTION == "knowledgetagsource":」（`scripts/agents/agent-watch.py:1627`） | 工作区改了没提交、起来时已改内容又变 | 对 |
| knowledgeworktreeblind | 「工作区那一半不看」（`scripts/agents/agent-watch.py:272`）对 「if BROKEN_DETECTION != "knowledgeworktreeblind":」（`scripts/agents/agent-watch.py:1612`） | 工作区改了没提交、起来时已改内容又变 | 对 |

17 条逐条「代码那一处对、点名的格在、我推出的红格与点名的相同」，没打中。

格名对账（量的）：

```
$ comm -13 cells-in-code.txt cells-in-header.txt    # 文件头点名、代码里没有的格
（空）
$ wc -l cells-in-code.txt cells-in-header.txt
 18 cells-in-code.txt
 18 cells-in-header.txt
```

主 agent 读到的复跑结局我也读了日志：`task-server/task log 20085` 末几行是 「复跑了 100 条开关（1 份），自证红且点名的格与实际红的格逐格相同 100 条」，汇总行 「break-switch-replay：绿」。它与我的静态推演是两条互不共用代码的路子，结果一致，只能说明这十七条与另外八十三条一起对上，不能把我的静态推演当作复跑（我没起自证）。

### 第 2 问：自证覆盖的缺口（读代码，不是对十七条的判定；一并交攻方腿的第 10 问）

- 「全部交回与 report 不打」那一格的第二条断言（report 那一趟，「check("全部交回与 report 不打", results["report"][0] == 0」（`scripts/agents/agent-watch.py:3961`））没有任何一条开关能让它红：report 不记基线（「take_repository_baseline(arguments) if arguments.mode == "watch" else None」（`scripts/agents/agent-watch.py:2159`）），也没有一条开关把这一节接到 report 上。它只是正面断言。
- `repositoryonalert` 只咬 `run()` 的第一个退 3 处（2692 行）；`run()` 里读到检出就整查一次的那个退 3 处（「return 3」（`scripts/agents/agent-watch.py:2732`））没有这一节、也没有开关、也没有格。
- `--processes-only` 的「起来时的进程都退出了、退 0」那一路（「print(f"看门狗起的时候就在跑的 {len(watched)} 个进程都退出了。")」（`scripts/agents/agent-watch.py:2197`））同样没有开关、没有格；`repositoryonfinish` 只改 `run()`。
- `knowledgetagsource` 与 `knowledgeworktreeblind` 点名的格集合相同（两格），仅凭格名分不出是哪条坏了；分辨靠失败行里「实际点名 …」那一段（标签全变 或 文件没了）。

### 第 2 问：什么现象会推翻

任一条开关设上起 `--selftest`，实际红的 `✗ 自检：知识同步提醒（…）` 行的格名集合与上表「文件头点名的格」不同。任务 20085 的结局行是同一判据的机器版；我的推演若与它出入，以它为准、回头查我哪一格读错。

## 第 3 问：技能段、`watch.conf` 两行、两张登记表两行，对代码

### 技能段（`.claude/skills/dispatch/SKILL.md` 第 39 行，一整行）

| 句 | 判定 | 依据 |
|---|---|---|
| 「定时回报（看门狗退 4）带一节」 | 成立 | 「定时回报（看门狗退 4）带一节」（`.claude/skills/dispatch/SKILL.md:39`）；两路 「return 4」（`scripts/agents/agent-watch.py:2208`） |
| 「以一句提醒起头（到点该同步项目最新变动与状态）」 | 成立 | 「到点该同步项目最新变动与状态」（`.claude/skills/dispatch/SKILL.md:39`）；「知识同步提醒：到点了，该同步项目最新变动与状态了。」（`scripts/agents/agent-watch.py:1680`） |
| 范围八项逐一列出 | 表成立；第八项在本仓点不到 | 技能段列的八项与表一致：「`.claude/singlefs-ai-sop/rules/` 底下的」（`.claude/skills/dispatch/SKILL.md:39`），对 「".claude/kb/", ".claude/singlefs-ai-sop/rules/")」（`scripts/agents/agent-watch.py:536`）；效果同第 1 问 H5 |
| 「不分提交没提交，各标「新提交动到」「工作区改了没提交」「两样都有」」 | 不全 | 来源标签四种，这一句只列前三种（`.claude/skills/dispatch/SKILL.md:39`）；第四种 「DROPPED_KNOWLEDGE_TAG = "起来时未提交的改动已不在 git status 里"」（`scripts/agents/agent-watch.py:542`）会在输出里出现、技能没说是什么意思 |
| 「里程碑、决策索引、欠账表排最前」 | 成立 | 「里程碑、决策索引、欠账表排最前」（`.claude/skills/dispatch/SKILL.md:39`）；表第 538 行前三项 |
| 「再是分支与 HEAD、看门狗起来之后的新提交、`git status --short` 的计数、别的变了的路径」 | 成立 | 次序同 1703 到 1719 行 |
| 「照末尾「→」那一行核手上正在改或要改的文件有没有被别的会话改过、提交过」 | 指向含糊 | 这一节自己的末行是 「→ 同步：先重读上面点名的」（`scripts/agents/agent-watch.py:1724`）；整份输出的最后一个「→」行是后面的重起看门狗那行（`print_active_list` 里 「接着盯就用 run_in_background 再起一个看门狗」（`scripts/agents/agent-watch.py:2440`），`--processes-only` 路是 2206 行那条）。带子 agent 的定时回报里「末尾「→」那一行」按字面读到的是重起那行，不是「→ 同步」 |
| 「最后起下一只看门狗」 | 在没有子 agent 还在跑时推不出 | 出路句只在还有子 agent 时才打（「if not active_ids and not unstarted_ids:」（`scripts/agents/agent-watch.py:2437`）），代码不要求总是再起；这一句是写给主 agent 的做法，无条件写出（交第 11 问） |
| 「提醒句写知识类文件没变的，不用重读」 | 成立 | 「→ 同步：知识类文件没变，不用重读；」（`scripts/agents/agent-watch.py:1726`） |
| 「写着「没查成」的，照它给的命令自己看仓」 | 成立 | 「→ 自己看：git -C {root} status --short；git -C {root} log -5 --format='%h %s'」（`scripts/agents/agent-watch.py:1692`） |
| 「条数上限是 `watch.conf` 的 `repository-report-lines`」 | 键名成立；当前值在配置里读不到 | 键在 `watch.sh` 的认键表里：「root-free-gib-alert repository-report-lines」（`scripts/agents/watch.sh:76`）；`watch.conf` 只有注释行 27，没有键值行；当前值是脚本默认 20（532 行） |

### `watch.conf` 两行注释（27、28 行）

| 句 | 判定 | 依据 |
|---|---|---|
| 「repository-report-lines 20」、「定时回报（--max-minutes 到点、没有告警）里」 | 成立 | 「repository-report-lines 20」（`scripts/agents/watch.conf:27`）；默认 「REPOSITORY_REPORT_LINES_DEFAULT = 20」（`scripts/agents/agent-watch.py:532`）；退 4 的两处 2208、2708 |
| 「点名的知识类文件、新提交、起来之后变了的别的路径这几张清单各最多列这么多条，余下的报条数」 | 成立 | 「点名的知识类文件、新提交、起来之后变了的别的路径这几张清单各最多列这么多条，余下的报条数」（`scripts/agents/watch.conf:27`）；1667 行的 `rows_with_remainder` |
| 「写 0 不打这一节（告警、全部交回、--report 那几路本来就不打）」 | 成立 | 「写 0 不打这一节」（`scripts/agents/watch.conf:28`）；1580、1676 行；`--report` 是 watch.sh 的选项：「--report) mode="report" ;;」（`scripts/agents/watch.sh:280`） |
| 「读哪个仓：缺省是统一任务服务给的来源仓根，没有就是本脚本所在的仓」 | 成立 | 「读哪个仓：缺省是统一任务服务给的来源仓根，没有就是本脚本所在的仓」（`scripts/agents/watch.conf:28`）；「return arguments.repository_root or session_directories.source_repository() or REPOSITORY_ROOT」（`scripts/agents/agent-watch.py:1574`） |

### `watch.sh` 两处（76、216 行，范围里点名）

「printf 'max-minutes=0\nroot-free-gib-alert=128\nrepository-report-lines=0\n'」（`scripts/agents/watch.sh:216`）：写 0 时 `take_repository_baseline` 返回 None（1580 行），这几趟真起的看门狗不跑 git、不读真仓，与行尾的说明一致，成立。76 行见上表。

### 两张登记表各一行

| 句 | 判定 | 依据 |
|---|---|---|
| `selftest-registry.tsv`（24 行）：自证「在临时目录里造小仓、起 git（init、symbolic-ref、add、commit、log、branch、status、diff，读用户与系统的配置一律关掉）与一个只睡的 sh 脚本顶替 git（造超时，仓根、git 与时限都换成环境变量给的值，不读本仓），并读写小仓里的文件取内容摘要」 | 成立 | 「在临时目录里造小仓、起 git（init、symbolic-ref、add、commit、log、branch、status、diff，读用户与系统的配置一律关掉）与一个只睡的 sh 脚本顶替 git（造超时，仓根、git 与时限都换成环境变量给的值，不读本仓），并读写小仓里的文件取内容摘要」（`.claude/hooks/selftest-registry.tsv:24`）。八个 git 子命令与代码的并集逐个相同（量的，下面输出）；「读用户与系统的配置一律关掉」对 「SELFTEST_GIT_ENVIRONMENT = {」（`scripts/agents/agent-watch.py:3658`）；顶替的脚本 「handle.write("#!/bin/sh\nexec sleep 30\n")」（`scripts/agents/agent-watch.py:3813`）；不读本仓对 「os.environ[REPOSITORY_ROOT_VARIABLE] = build_selftest_repository」（`scripts/agents/agent-watch.py:3999`） |
| `break-switch-replay-files.tsv`（73 行）：现数一百条、四族各几条 | 成立 | 「现数一百条」（`.claude/gate.d/break-switch-replay-files.tsv:73`）；「九十六条，另有 SINGLEFS_WORK_ROOT_BREAK 两条、SUBAGENT_TRANSCRIPT_BREAK 一条、SESSION_DIRECTORY_BREAK 一条，这一格读的是全部四族」（`.claude/gate.d/break-switch-replay-files.tsv:73`）；数：96 加 2 加 1 加 1 等于 100，输出见下 |
| 同一行：「任务 19936 复跑这一百条：九十四条点名的格与实际红的格逐格相同，另六条的点名改过之后还没再复跑」 | 不成立（已过时） | 这句抄的是 「任务 19936 结局退 1：100 条开关里 94 条点名的格与实际红的格相同，6 条不符」（`records/2026-11-17-subagent账本自上而下清欠.md:832`）那一趟；之后任务 20085 复跑了全部 100 条、100 条对上（量的，`task-server/task log 20085` 末几行），行里「另六条还没再复跑」与它对不上 |

量的输出：

```
$ grep -c '^  AGENT_WATCH_BREAK=' scripts/agents/agent-watch.py
96
$ grep -nE '^  (SINGLEFS_WORK_ROOT_BREAK|SUBAGENT_TRANSCRIPT_BREAK|SESSION_DIRECTORY_BREAK)=' scripts/agents/agent-watch.py | cut -c1-60
412:  SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  工作
415:  SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted
418:  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered
421:  SESSION_DIRECTORY_BREAK=home-from-environment  设
（自证用 git 子命令：selftest_git 的 add、commit、init、log、symbolic-ref；run_git 的 branch、diff、log、status；并集八个，与登记行一致）
```

### 第 3 问小结：四个点

| 点 | 判定 |
|---|---|
| 范围八项 | 表、技能段、文件头三处写法一致；第八项在本仓永远点不到（第 1 问 H5） |
| 标签字面 | 文件头与代码常量逐字相同；技能段只列三种，漏第四种 |
| 默认值 20 | 文件头 133 行、`watch.conf` 27 行、代码 532 行与 4551 行（`--repository-report-lines` 的 `default`）一致；配置文件里没有键值行 |
| 开关条数与四族 | 文件头 11 加 6 等于 17，登记行 100 条（96、2、1、1）都对；登记行里「另六条还没再复跑」已过时 |

### 第 3 问：什么现象会推翻

- 技能段「不全」「含糊」：技能段补上第四种标签、把「末尾「→」」改成「这一节末尾「→ 同步：」」，或真实输出里 「→ 同步：」 就是最后一个「→」行（没有还在跑的子 agent 时是）。
- 登记行「过时」：任务 20085 的结局行不是 100 对 100，或该行写进了 20085 的读数。
- `selftest-registry.tsv` 那行：自证起了一个行里没写的东西（`ps` 一类、`cargo`），或 git 子命令并集变了。

## 第 4 问：用户要的三件各落在哪，有没有多做

用户原话在记录第二百六十二节：「也就是说主对话 一个小时同步一次项目的最新改动信息 不要做久了忘完了」（`records/2026-11-17-subagent账本自上而下清欠.md:782`）；弹窗问同步什么，答「最新的git statis」（`records/2026-11-17-subagent账本自上而下清欠.md:782`）；问没起看门狗的对话要不要罩到，答「只加在看门狗里」（`records/2026-11-17-subagent账本自上而下清欠.md:782`）；补充「这主要是一个提醒 为了解决知识不同步的问题」（`records/2026-11-17-subagent账本自上而下清欠.md:792`）。

| 用户要的 | 落在哪 | 判定 |
|---|---|---|
| 每小时提醒主对话同步 | 周期是定时回报已有的 `--max-minutes` 默认 60：「parser.add_argument("--max-minutes", type=float, default=60」（`scripts/agents/agent-watch.py:4525`），`watch.conf` 没有覆盖它（键值行只有 5 个，`scripts/agents/watch.conf:30` 到 34 行）；到点打这一节：2205、2706 行；提醒句 1680 行、出路 1724、1726 行 | 部分落到：只在「没有告警、看门狗还活着」的那一趟到点才打（告警退 3、全部交回退 0 不打），不是无条件每小时；这一点的轻重交攻方腿的第 8 问 |
| 内容是最新的 git status | 到点现读：「now = repository_snapshot(root)」那一步在 1685 行，里面 `git status --short`（1566 行）；打出三个计数（1710 行）与相对起来时的增量（1712 到 1719 行），知识类文件按路径点名（1699 行） | 部分落到：没有此刻 `git status --short` 的路径清单；清单只在「没查成」时以命令形式给（1692 行）。用户那句「最新的 git status」读成「状态的信息」就够，读成「那张清单」就不够，原话读不出是哪种（推不出） |
| 只加在看门狗里 | 运行时代码都在 `scripts/agents/agent-watch.py`（基线 2159、2658，打印 2205、2706，函数 1459 到 1734），`watch.sh`、`watch.conf` 是它的配置通道；没派过子 agent 的对话没有看门狗、不会收到 | 成立。技能一段与两张登记表是说明与登记，不加运行时行为；六份的 `git diff HEAD --stat` 只有这六份（量的，728 行加、23 行减） |

多做的（用户没要而加了行为的），逐条标记录里有没有出处：

| 多出来的 | 位置 | 记录里的出处 |
|---|---|---|
| 范围表第八项 `.claude/singlefs-ai-sop/rules/`（且本仓点不到） | 「".claude/kb/", ".claude/singlefs-ai-sop/rules/")」（`scripts/agents/agent-watch.py:536`） | 记录里主 agent 写的范围只有七项，止于 `.claude/kb/`：「`.claude/skills/`、`.claude/agents/`、`.claude/kb/`）变过的不分提交没提交逐份点名，里程碑文件、决策索引、欠账表排最前」（`records/2026-11-17-subagent账本自上而下清欠.md:792`）；第八项的出处在我读到的材料里没有（推不出） |
| 内容摘要：对范围内路径读盘算 sha256 | 「for key in status_by_path if digest_wanted(key)}」（`scripts/agents/agent-watch.py:1567`）；1537 到 1551 行 | 用户要的是 git status，读文件内容是超出 git 的行为；记录里没有，是为了补「状态码没变而内容又改」的漏报 |
| 起来时基线与「相对起来时」的增量 | 「take_repository_baseline(arguments) if arguments.mode == "watch" else None」（`scripts/agents/agent-watch.py:2159`） | 记录里主 agent 的改法写了「看门狗起来之后新进的提交」「看门狗起来之后变了的路径」（`records/2026-11-17-subagent账本自上而下清欠.md:784`），随后用户「换上并验」（799 行）；算有出处 |
| 新选项 `--repository-report-lines`、`--repository-root` 与三个环境变量、`watch.conf` 的键 | 「parser.add_argument("--repository-root", default=os.environ.get(REPOSITORY_ROOT_VARIABLE),」（`scripts/agents/agent-watch.py:4553`）等 | 记录只写了「配自证格与弄坏开关」，没写这几个配置面；上限防刷屏、仓根给自证用，是实现需要，不是用户要的行为 |
| 写 0 关闭这一节 | 1580、1676 行 | 记录里没有；与 `watch.conf` 各键「写 0 不查」同一风格 |
| 提醒句、「→ 同步」出路 | 1680、1724 行 | 用户补充「这主要是一个提醒」（792 行）支撑 |

### 第 4 问：什么现象会推翻

- 「部分落到」两条：用户补一句「清单要有」「告警那一轮也要」，则改判成没落到；用户回「计数和增量就够」，则改判成落到。
- 「多做」：能在派发提示或用户原话里找到第八项范围、内容摘要的出处，则对应一行改成有出处。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 1 问 H1 位置、退 4、两路 | 成立 | 2205、2706 打在「定时回报」行之后、出路句之前；没有子 agent 时出路句不打 |
| 第 1 问 H2 同一个函数 | 成立 | 两路都经 `repository_status_section` |
| 第 1 问 H3 提醒句与几份 | 句子成立，「份」推不出 | 「份」按状态路径键数：未跟踪整目录一份、「已不在」一份，不是文件数 |
| 第 1 问 H4 次序 | 成立 | 提醒句、点名、仓的现状、「→」，1729 行 |
| 第 1 问 H5 范围八项 | 表成立，第八项不成立 | `.claude/singlefs-ai-sop/` 被 `.gitignore` 忽略、零个跟踪文件，永远点不到 |
| 第 1 问 H6 点名先后 | 成立 | 538 行表加 1515 行 |
| 第 1 问 H7 取并集 | 成立 | 提交一半 1598 行，工作区一半 1618、1624 行 |
| 第 1 问 H8 内容摘要 | 成立，两处边角推不出 | 链接取链接目标；未跟踪目录里指向目录的链接不下探 |
| 第 1 问 H9 四种标签 | 成立 | 539 到 542 行逐字相同 |
| 第 1 问 H10 仓的现状与计数 | 成立，单位同 H3 | 计数现数、两边各算一次对；一个整目录算一个路径 |
| 第 1 问 H11 别的路径与「→」 | 成立 | 1648、1719、1724、1726 行 |
| 第 1 问 H12 上限与写 0 | 成立 | 默认 20；「watch.conf 的同名键」只有注释行，没有键值行 |
| 第 1 问 H13 只在定时回报里打、退出码不变 | 打印位置成立，退出码不变有一处推不出 | `GIT_TIMEOUT_VARIABLE` 非数字时 `float` 抛 `ValueError`，起来时没人接 |
| 第 1 问 H14 读哪个仓 | 取法成立，两台同值是推的 | 三级取法 1574 行；同值靠共享根同一路径 |
| 第 1 问 H15 查不成 | 成立，措辞偏差 | 那一行以「仓的现状：」起头，不是单独一行「没查成：」 |
| 第 1 问 H16 环境变量只给自证用 | 成立（约定） | 代码照读，不挡非自证，与已有的 `AGENT_WATCH_ROOT_FREE_BYTES` 同做法 |
| 第 1 问 H17 开关 11 加 6 | 成立 | grep -c 得 11 与 6 |
| 第 2 问 十七条开关 | 17 条全对 | 弄坏的那一处对、点名的 18 个格都在、我推出的红格与点名相同 |
| 第 2 问 自证缺口 | 4 处（读代码） | report 那条断言无开关咬；第二个退 3 处与 `--processes-only` 退 0 处无开关无格；两条开关点名集合相同 |
| 第 3 问 技能段 | 11 句里 6 句成立、1 句表成立第八项点不到、1 句不全、1 句含糊、1 句推不出、1 句键名成立值读不到 | 漏第四种标签；「末尾「→」」指向重起那行；「起下一只」无条件 |
| 第 3 问 `watch.conf` 两行 | 4 句全成立 | 默认值来自脚本，配置里没有键值行 |
| 第 3 问 `watch.sh` 76、216 | 成立 | 认键表加了键；自证配置写 0，不读真仓 |
| 第 3 问 `selftest-registry.tsv` 一行 | 成立 | 八个 git 子命令并集与代码一致 |
| 第 3 问 `break-switch-replay-files.tsv` 一行 | 条数成立，复跑叙述不成立 | 100 条、96/2/1/1 对；「另六条还没再复跑」被任务 20085 超过 |
| 第 4 问 三件 | 部分落到、部分落到、成立 | 周期只在没有告警的定时回报里到；没有此刻 status 清单；只在看门狗里 |
| 第 4 问 多做 | 6 项 | 第八项范围（出处不明）、内容摘要读盘、起来时增量、新选项与变量、写 0 关闭、提醒句与出路 |

## 要主 agent 跑的命令

一条，探针（不带范围声明：它不起编译、门禁、测试，只在工作根下建十个临时小仓、跑 git 与读 `agent-watch.py` 里的函数；每趟先由包装删掉运行目录再起）。

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r1-snapshot /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-sonnet-run /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-model
```

- 在哪个目录起：submit 的 `--cwd` 是我的草稿目录（上面已写）；探针读的是主工作区此刻的 `scripts/agents/agent-watch.py`，小仓建在运行目录 `…-sonnet-run` 下。
- 跑完看哪几行：`TASK_SERVER_RESULT` 行 `type=` 与 `exit=0`；`task-server/task log <号>` 里有 `name=probe_done cases=10` 一行，并有十组 `name=<格名>` 的行。运行目录与小仓跑完由主 agent 删（`du -sh` 后 `rm -rf` 工作根下那一个以 `-run` 收尾的目录）；`rerun.sh` 的第二个参数 `--frozen` 只要求是个目录，探针不读它。
- 探针的内存与时间：没起过，推的：十格共约八十条 git、每条毫秒级，整趟秒级，内存几十 MiB。

我推的输出（没跑过，写在这里就是为了让输出能推翻它；格名后面是关键字段）：

| 格 | 推的结果 | 推翻它的输出 |
|---|---|---|
| untracked-directory | `knowledge=[('.claude/kb/newdir/', '工作区改了没提交')] count=1 counts=(0, 0, 1)` | count 等于 3 |
| ignored-scope | `raw_status=''`，`knowledge=[] count=0` | 点到 `.claude/singlefs-ai-sop/` 底下的文件 |
| space-and-cjk-path | git 给带空格的路径加双引号，该键判不进范围，只点名中文名那一份，`count=1` | count 等于 2（git 没加引号） |
| rename-across-scope | 两个改名各取在范围里的那头，`count=2`，`others=[]` | `others` 非空 |
| head-moved-backward | `commits=[]`、`commit_paths` 含 `.claude/kb/second.md`、标签「新提交动到」 | knowledge 为空 |
| symlink-in-untracked-directory | `digest_equal=True`、`knowledge=[]` | `digest_equal=False` |
| stage-only | 点名 `.claude/kb/seed.md`「工作区改了没提交」，计数 `(1, 0, 0)` | knowledge 为空 |
| mtime-only | `knowledge=[]` | 点名了 seed.md |
| deleted-by-commit | `commit_paths=['.claude/kb/gone.md']`、标签「新提交动到」 | knowledge 为空 |
| commit-then-edit-again | 点名 seed.md、标签「两样都有」 | 标签不是「两样都有」 |

## 没做什么

- 不判别的腿的格：第 5 到 11 问全部没做，没替攻方腿判；第 2 问末与第 1 问末的几处边界现象只标「交攻方腿」。
- 没替主 agent 采纳或出判决；没起 `selftest`、任务服务、门禁与任何一条开关（都经「要主 agent 跑的命令」）；没往统一任务服务交任务，上面那条探针没有复跑过、判决要引的数都是推的。
- 任务 20085 的日志只读了末几行，没对着每条开关逐一看。
- 第八项范围的出处：只查了记录第二百六十二节与工具员的报告（`~/.local/share/singlefs/work/` 下），没有读到用户对「八项」的原话；说「出处不明」只限于我读到的材料。
- 没读攻方腿这一轮的产出与草稿；为核用户原话，对 `_watchdog-knowledge-sync-governance-r1-body.md` 只 grep 过含「用户」的几行，没有通读。
