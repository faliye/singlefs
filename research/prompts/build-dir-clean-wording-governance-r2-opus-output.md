# build-dir-clean-wording-governance-r2 攻方腿（Opus）报告

正文 `research/prompts/_build-dir-clean-wording-governance-r2-body.md`；被判的 30 份逐个对过开工快照 `research/prompts/build-dir-clean-wording-governance-r2-snapshot/judged-sha256.txt`，`sha256sum -c` 没有一行不是 OK（量的，这一次在主工作区跑）。改前原样对着正文点名的各草稿目录 `base/` 与 `git diff HEAD` 看。禁读清单里的那一份没读。

## 各格判定一览

| 问 | 结论 | 一句话 |
|---|---|---|
| 1 完工清理闸新判的任务现场 | 推不出（打中） | 闸按尝试表的 `session` 列取尝试，服务写每一次尝试时这一列恒是空串；真库上「任务现场」一个都列不出，五格自证的样本库把这一列填成会话，看不见这一处 |
| 2 两份定义与复跑模板 | 推不出（打中） | 第 9 步新写的 `--cwd <运行目录>` 交法照字面填（定义自己给的模型目录形态是相对路径 `research/prompts/<轮>-opus-model/`）时，命令位置上的外层脚本按运行目录解，找不到、退 127，会编声明也读不到；别的几样成立 |
| 3 注释历史判法 | 推不出（打中） | 来历写在「」外、「」里只放角色名的句子（样本见第 3 问）新判法放过、旧判法判红；四条开关红在点名的格；少报的行在这几个射程目录里只有检出钩子与派发闸（推的，grep 近似） |
| 4 写全的树与改过的出路、说明 | 推不出（打中） | 改法 6 写进 `task-server/task`、`attempt_runner.py`、GPU 控制命令与 `prune-server/control` 的「取消都不删它」对派到第二台、中途被停的那一次是假话（那一趟连编译目录一起删、组里删掉那一行）；另有三处「树」没写全「解开符号链接后」 |

四问的推翻条件各写在那一问末尾。模型的数一个都还没跑出，下面凡是模型的读数都是「推的，待主 agent 跑」。

## 复跑命令与模型文件

模型目录 `research/prompts/build-dir-clean-wording-governance-r2-opus-model/`，不编 Rust，只喂钩子与判法；跑法在文末「要主 agent 跑的命令」第 1、2 项。文件与 sha256（这一次 `sha256sum` 的原样输出）：

```
9e19d4367efe91f3ee689903c44c417455fd2b295680d51cff8fd87d73d419e3  tools/kept-scene-store-shape-probe.py
b4ed060c844b12cf353613d1b92581427313e98369b5dd211a687581df9f7934  tools/quoted-role-name-probe.sh
e4f31699d787d3650ebb6d98b1b362d4ec45a52c00d0c6e1cc77a0574c115361  tools/rerun-from-a-clean-work-directory.sh
6d0e00707662cec6d5c40a011526e7e3ef9671d3b46fc4ca38e561beadb5a5fe  tools/rerun.sh
21af1a577d4e1c055a06a842484aba58b46cb01e2f671f90b1f55424cdf327b5  feeds/feed-step9-mkdir.json
a56ea66241baa8333dadc0b33c73e8538d8b6f5632054a9ce5f4e41deb065375  feeds/feed-step9-submit.json
```

- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`（文件头会编声明留着）。它打四样：外层声明拿到的 `CARGO_TARGET_DIR` 是不是以运行目录为树的本机组（组名前缀照 `task-server/task_server_machines.py` 的 `build_directory_name` 现算）；四份被判文件的 sha256 与开工快照是否相同；第 1 问探针；第 3 问探针。
- `tools/rerun-from-a-clean-work-directory.sh`：第 9 步的外层，文件头第 2 行写会编声明，清空运行目录（目录本身留着）再调 `rerun.sh`。
- `tools/kept-scene-store-shape-probe.py`：第 1 问的两臂（见下）。
- `tools/quoted-role-name-probe.sh`：第 3 问的两臂（见下）。
- `feeds/`：第 2 问喂项目代码闸与范围声明闸的两条输入。

语法检查（`bash -n`、`python3 -m py_compile`）这一次都退 0（量的）；整份没跑。

## 第 1 问：完工清理闸新判的任务现场 —— 推不出（打中）

### 打中的那一处：尝试表的 session 列恒是空串

闸读服务库的那两句：

- 「("session = ?", (session,))」（`.claude/hooks/completion_cleanup_guard_decisions.py:1294`）
- 「SELECT task_id, record FROM attempts WHERE {where}」（`.claude/hooks/completion_cleanup_guard_decisions.py:1299`）

服务那一侧，每一次尝试的记录在起尝试时就把会话写成空串，库照记录写这一列，之后没有一处改它（`grep` 过 `task-server/task_server_scheduler_*.py` 与 `task-server/task_server.py` 里写尝试的地方，量的）：

- 「"spec": specification, "state": "starting", "session": ""」（`task-server/task_server_scheduler_starting.py:140`）
- 「state, session, command_key = record.get("state", ""), record.get("session", "")」（`task-server/task_server_store.py:268`）

所以真库上 `attempts WHERE session = <本会话>` 一行都取不到，`kept_scenes` 交空表，「任务现场」这一种在主 agent 收工时一个都列不出（推的，按代码推；探针待主 agent 跑）。任务表那一句（`tasks WHERE session = ?`）是对的：任务记录带会话，分片的子任务照抄父任务的会话（`task-server/task_server_scheduler_building.py` 的 `create_child`）。

五格自证看不见这一处：样本库自己建表、自己写尝试行，尝试行的 session 列填的是提交它的会话，与服务写的不一样：

- 「表与列照 task-server/task_server_store.py 的 Store」（`.claude/hooks/completion_cleanup_guard_selftest.py:153`）
- 「(task_id, owner_session, json.dumps({"task_id": task_id, "scene_directory": directory})))」（`.claude/hooks/completion_cleanup_guard_selftest.py:170`）

能照喂的输入：`tools/kept-scene-store-shape-probe.py` 建两份样本库（表与列照 `Store.initialize`），任务行一样（本会话、`finished`、`environment` 里 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`、尝试记的现场目录在盘上），只有尝试行的 session 列不同：selftest-shape 臂填会话，store-shape 臂填空串；各喂一次主 agent 的 Stop：

```json
{"hook_event_name": "Stop", "session_id": "s-kept-scene-store-shape", "transcript_path": "<臂目录>/projects/project-directory/s-kept-scene-store-shape.jsonl", "stop_hook_active": false}
```

样本库经 `COMPLETION_CLEANUP_TASK_SERVER_DATABASE` 交给钩子（不碰服务的库），工作根、数据根、会话临时目录根、判定库服务自证根照自证 `Scenario.environment` 那一套设到臂目录下。预期（推的，待主 agent 跑）：selftest-shape 臂 `blocked=true scene_listed=true clean_named=true guard_query_rows=1 joined_query_rows=1`；store-shape 臂 `blocked=false scene_listed=false clean_named=false guard_query_rows=0 joined_query_rows=1`。推翻它的观测：store-shape 臂照样拦下并列出现场。

同一个查法在删掉的那一支里就有：`git show HEAD:.claude/hooks/completion-cleanup-guard.sh` 第 1525 行同样是按会话列取尝试（量的，这一次 `grep -n` 的命中）。第一轮判决第二节第 1 条的依据「删掉的那一支是收工时唯一叫主 agent 起 task clean <号> 的地方」（转述），那一支在真库上大概也从没列出过东西（推的，没量）；这一条交辩方与主 agent 判，不归这条腿。

四句：
- 分不分辨臂：不分辨第一轮改法 1 选「另判任务现场」还是别的路（不判、判全部现场、登豁免），病根在读法里，哪条臂照这个读法写都一样落空；它只分辨「按尝试表 session 列取」与「按任务号接任务表取」两种读法。
- 系统当时看不看得到：看得到。收工那一刻库里有任务行的会话与尝试行的 `task_id`，接一下就取得到；闸看的是一列恒空的列。
- 满足的是哪一句：第一轮判决改法 1「另判本会话已有结局、设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`、现场目录还在的任务」（转述）在真库上一条都判不到；`.claude/skills/dispatch/SKILL.md:18` 那一句「本会话已有结局、设了留现场、现场目录还在的任务现场（删法 `task-server/task clean <号>`，不直接 `rm`）」（`.claude/skills/dispatch/SKILL.md:18`）因此也成了只在纸面上的话。
- 改法在打中的那一格上还中不中：改成「按任务号取尝试」（下面改法 A）在这一格上不再中；第一轮判决的改法 1 本身不涉读法，不起作用。

改法 A（只在我的模型上量过、被攻过零轮；其实还没量，模型待主 agent 跑）：尝试那一句改成 `SELECT a.task_id, a.record FROM attempts a JOIN tasks t ON a.task_id = t.id WHERE t.session = ?`（弄坏开关 `kept-scenes-session-unchecked` 下去掉 `WHERE`）；样本库的 `service_task` 尝试行 session 列照服务写空串；配一条新开关（例 `kept-scene-attempts-by-session-column`，照现在这一句取），红在 `main-kept-scene-left`。

### 别的几样

| 情形 | 结论 | 依据 |
|---|---|---|
| 分片子任务 | 成立 | 子任务会话照父任务（`create_child`）；闸把子任务归到父任务号、`task clean <父任务号>` 连子任务的现场删（`task-server/task_server_scheduler_building.py` 的 `remove_scenes` 按 `children_of` 取）；留不留照父任务的 `environment`，与服务的 `scene_due` 同一口径 |
| 人手 `--manual` | 成立 | 任务记无会话，从不等于主 agent 的会话，闸不列；这类只能人手 `task clean` |
| 「设了留现场」认法 | 成立 | 闸与 `scene_due` 都读父任务记录 `environment` 里这个变量是不是 `"1"`；内存包装也只认 1（`scripts/hosts/run-with-memory-cap.sh` 文件头第 121 行，转述）。写成 `-- env SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 <命令>` 的三处都不认，服务照到时删，不留孤儿 |
| 现场在第二台 | 成立 | 现场目录在共享工作根下，两台同一个绝对路径（`task-server/task_server_lib.py` 文件头「现场」），本机 `isdir` 看得到 |
| 取不到结局（在排、在跑、待确认） | 成立 | 只判 `finished`；主 agent 的 Stop 本来只在手里没有后台任务时判，下一回收工再判 |
| 主 agent 换过会话号 | 推不出（推的，没量） | 闸与服务都只认提交时的会话：换号之后旧号交的留现场任务闸不列，`task clean` 也被拒（「if task.get("session") != session and not lib.broken("clean-session-unchecked", self.environment):」（`task-server/task_server_scheduler_starting.py:728`）），而这类现场「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删。」（`task-server/task_server_lib.py:297`）。换号后只剩 `control clear-queue` 一条路。会话号什么时候换（`/clear`、续开）这一轮没核，闸的文件头没写这一种 |
| 服务库读不了 | 成立 | `sqlite3.Error` 记一处没判全、放行；载不进 `task_server_lib.py` 同样记。另有一处静默：`read_configuration` 抛错时退到 `default_values` 不记一句，那条路径不在就照「服务没装过」不判（推的；多机配置写了别处的状态目录、又有别的键写坏时才碰得到） |
| `COMPLETION_CLEANUP_TASK_SERVER_DATABASE` | 低 | 文件头说只给自证用、平常不设，但平常设了照读、不拒（弄坏开关有 `refuse_break_outside_selftest`，它没有）；会话环境里带着它就静默读另一份库（推的） |

五格自证与六条开关：读代码逐格对过，每条开关只改一处判法、各自那一格的断言正好落在那一处（推的，与第一轮复跑「82 个弄坏开关各红在点名的格」相符）。它们红在名字说的那件事，但全在同一个样本世界里：尝试行带会话。真库的形状一格都没有，所以「五格都绿」不说明真库上判得到。

推翻条件：主 agent 复跑第 2 项，store-shape 臂拦下且列出现场；或在服务真库里看到尝试行的 session 列非空（这条腿按项目代码闸不读真库）。

## 第 2 问：两份定义与复跑模板 —— 推不出（打中）

### 打中的那一处：`--cwd <运行目录>` 配上相对的模型目录，外层脚本找不到

定义给模型目录的形态是相对仓根的路径：「模型目录（形态 `research/prompts/<轮>-opus-model/`）」（`.claude/agents/three-way-attack.md:23`）。第 9 步这一版把交法改成在运行目录里起：「再 `task-server/task submit --cwd <运行目录> -- bash <模型目录>/tools/<外层脚本> <仓根> <冻结副本> <运行目录> <模型目录>`」（`.claude/agents/three-way-attack.md:47`）。服务认命令位置上的脚本照任务的工作目录解：「return os.path.realpath(os.path.join(task["cwd"], words[index]))」（`task-server/task_server_scheduler_starting.py:369`）；尝试也在 `--cwd` 里起。照字面把 `<模型目录>` 填成定义给的形态：

```
mkdir -p /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-relative-run
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-relative-run -- bash research/prompts/build-dir-clean-wording-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/build-dir-clean-wording-governance-r2-snapshot /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-relative-run research/prompts/build-dir-clean-wording-governance-r2-opus-model
```

运行目录下没有 `research/`：服务读不到那份脚本的文件头（不给 `CARGO_TARGET_DIR`），`bash` 找不到脚本退 127（推的，待主 agent 跑，文末第 4 项）。第一轮改之前第 9 步不写 `--cwd`，从仓根交，相对路径解得开；这一处是这一轮的修补带进来的。模板用法同一形：「task-server/task submit --cwd <新的草稿目录> -- bash <模型目录>/tools/rerun.sh」（`scripts/agents/three-way-rerun-template.sh:8`），`<模型目录>` 与 `--model` 也没说要绝对路径。第 9 步的外层脚本收的第四个参数 `<模型目录>` 同样照 `--cwd` 解。

四句：不涉臂；服务提交那一刻看得到的是字面的相对路径与 `--cwd`，两样都照用户写的；满足的是第 9 步「交法写两项」那一句照字面做走不通；第一轮改法 2 给的两种改法（会编声明、改成 `bash -c`）在这一格上都还中：`bash -c '…'` 里的相对路径同样按运行目录解。

改法 B（被攻过零轮，推的）：第 9 步与模板用法里 `<模型目录>`、`<仓根>`、`<冻结副本>` 写明「绝对路径」，或第 9 步的外层脚本路径写 `<仓根>/<模型目录>/tools/<外层脚本>`。

### 别的几样

| 问 | 结论 | 依据 |
|---|---|---|
| 外层写了会编声明之后服务给不给、给哪棵树的组 | 成立（推的，待主 agent 跑第 2 项的 `name=compiles_declaration` 行） | 命令位置上是外层脚本、文件头前 60 行有 `# task-server-compiles: yes`，`compiles_declared` 照 `command_script` 读到它；树照 `build_tree_root` 是运行目录本身：工作根往上家目录、本地数据父目录、`…/share`、`…/singlefs`、`…/work`、`/home`、`/` 都没有 `.git`（这一次 `ls -d <层>/.git` 都不在，量的） |
| `mkdir -p` 再 `--cwd <运行目录>` 那一形过不过两道闸 | 推不出（没喂） | 第一轮主 agent 喂的是 `bash -c` 那一形；这一形是这一轮新写的，喂法在文末第 5、6 项（输入 `feeds/feed-step9-submit.json`）。按代码推放行：范围声明闸的文件头没把模型复跑算成起验证的命令（`grep` `rerun`、`research/prompts`、`-model/` 在 `.claude/hooks/run-scope-guard.sh` 只命中别的两处，量的） |
| 外层清空运行目录、`rerun.sh` 认不认 | 成立 | 外层清空之后 `--work` 是空目录，`parse_arguments` 放行；内存包装的临时目录在 `/tmp` 下，不在 `--cwd` 里，清空不碰它（`scripts/hosts/run-with-memory-cap.sh` 文件头第 113 行，转述） |
| 输出目录对照对不对得上 cargo | 成立，缺两种写法 | `dev`、`test` 进 `debug`，`release`、`bench` 进 `release`，别的 profile 进同名目录，`--target` 前面多一层三元组，`--example` 进 `examples/`：与 cargo 的规则一致（cargo 文档，没在本项目跑过）；仓里 `.cargo/config.toml` 只有 `[env]`，没有 `build.target`（量的）。没写到的：`-r`（`--release` 的短写）与 `--profile=<名>` 等号写法，照表查不到（推的，低） |
| 模板开关改成只改摘录之后还测不测得到 | 成立 | 自证的拷贝只由一条 `cp` 从模板自己拷来，那一格读拷贝的前 60 行；模板文件头真少了这一行时拷贝也少，那一格照红。开关改的是摘录，走的是同一句 `grep -qx`。测不到的是腿自己改过的那份 `rerun.sh`（没有哪一格读它），漏了声明时 `"${CARGO_TARGET_DIR:?}"` 当场退 1，看得出来 |

范围外（交主 agent）：`.claude/agents/experiment-runner.md:29` 拿 replay.sh 当范例「`scripts/experiment-apparatus/replay.sh` 这样写」（`.claude/agents/experiment-runner.md:29`），而 replay.sh 文件头编译目录那一段还写「它归服务删（任务有了结局之后）」（`scripts/experiment-apparatus/replay.sh:38`），是同一旧说法；replay.sh 不在这一轮被判的 30 份里。

推翻条件：第 4 项那条照字面的交法跑成（退 0、`name=compiles_declaration` 行 `form=local-group-of-cwd-tree`）。

## 第 3 问：注释历史判法「」里不判角色名 —— 推不出（打中）

### 打中的那一处：来历写在「」外、「」里只放角色名

判法把收尾了的「」里的三方腿角色名换成占位符再判：「ROLE_NAME_WORDS = ("攻方腿", "辩方腿", "正推腿")」（`scripts/gate/comment-history-check.py:242`）、「for match in ROLE_NAME_PATTERN.finditer(piece, start, end):」（`scripts/gate/comment-history-check.py:507`）。词表里讲来历的词只有角色名、轮次、批号、「定案」这类；「打中」「指出」「报的」不在里面。所以只要把角色名套进「」，句子剩下的部分一个词都不中。共用约束要不许写的正是这一种：「不写日期、谁定的与哪一轮打中的」（`.claude/agent-common.md:47`）。

能照喂的样本（射程里任一份 `.sh` 的注释；`tools/quoted-role-name-probe.sh` 照这几行建样本根 `<输出目录>/comment-history-sample-root/scripts/gate/quoted-role-name-sample.sh`）：

```
# 这一支补的是「攻方腿」打中的那一格
# 「辩方腿」指出这里漏判，补了这一判
# 「正推腿」报的那一条照改
# 「第二轮攻方腿」打中的
# 「Bash写范围:三方腿:正推腿写别的轮的报告，拒」
# 正推腿报的那一条
```

按代码推（待主 agent 跑第 2 项的 `name=quoted_role_names` 两行）：判法此刻只列第 5 行（「第二轮」照判）与第 7 行（「」外的角色名），第 2 到 4 行放过；开 `quoted-role-names-judged`（「」里照判，等于改前）列第 2 到 7 行。正文举的「「第二轮攻方腿」打中的」这一种还拦得住，拦它的是「第二轮」，不是角色名。

文件头「管不到的」那一句（`scripts/gate/comment-history-check.py:177`，转述：「」里写的其实是来历、用的词正是角色名的，当格名放过，靠 review）讲的是格名里带出处的那一种（例子是「正推腿乙 Z8」）；上面三行不是格名，「」只是套在主语上，这一句没把它们写进去，也没有一格样本钉它。

四句：
- 分不分辨臂：分辨第一轮改法 4 的两种写法之间那一处细节，不分辨「修判法」对「登豁免」：登豁免表整份不判，这三行一样放过。
- 判法当时看不看得到：看得到。被放过的格名都有「<族>:<格>」的冒号，上面三行的「」里只有一个角色名。
- 满足的是哪一句：共用约束第 47 行「哪一轮打中的」那一类来历；判据字面只看词表，词在「」里被换掉了。
- 改法在打中的格上还中不中：第一轮改法 4 字面就叫「」里的「角色名不当历史词」（转述），照它做的这一版在这三行上照中；下面改法 C 不中。

改法 C（只在我的读法上核过、被攻过零轮）：只在「」里有 ASCII 冒号（格名 `<族>:<格>` 的分隔）时才换掉里面的角色名。这一次在检出钩子与派发闸里把带角色名的十三处逐处核过，每一处所在的「」里都有冒号（量的，`python3` 读文件、只读）；自证 `quoted-role-names` 那一格的三处样本也都带冒号。改法 C 还是挡不住「「x:攻方腿」打中」这种故意写的，只把门槛从「套一对「」」抬到「套一对带冒号的「」」。

### 别的几样

| 问 | 结论 | 依据 |
|---|---|---|
| 四条开关红在点名的格 | 成立（推的，与第一轮复跑「开关 31 条、对上 31 条」相符） | 逐格算过：`quoted-role-names-judged` 让第一格列出第 1、2 行，第二格照样列第 1 到 4 行；`nested-quotes-flattened` 下外层的「」配不上，第一格第 2 行的「攻方腿」露出来；`quoted-spans-unjudged` 下第二格第 1 行的「此前」、第 2 行的日期被盖住；`unclosed-quote-counted` 下第二格第 3 行的「正推腿」被盖住。每条只动自己那一格 |
| 全仓少报的只有检出钩子与派发闸 | 成立（推的，grep 近似） | 射程七个目录（加 `gpu-verification-server/`）里 `grep -rnE '「[^」]*(攻方腿|辩方腿|正推腿)'`（去掉 `fixtures/`）只命中三份：`.claude/hooks/bash-command-detector.sh` 7 行、`.claude/hooks/runner-dispatch-guard.sh` 4 行、`scripts/gate/comment-history-check.py` 4 行（量的）；判法自己在豁免表里整份不判（`.claude/gate.d/comment-history-exempt.tsv`）。grep 不分注释与字符串，精确的数要主 agent 在新旧两版上各跑一次 `--list` 比 |
| 括着的别的历史词与日期照判 | 成立 | 占位符只盖角色名；第二格样本钉住「此前」与日期 |

推翻条件：第 2 项 `arm=current` 那一行列出第 2、3、4 行。

## 第 4 问：写全的树与改过的出路、说明 —— 推不出（打中）

### 打中的那一处：「取消都不删它」对第二台中途被停的那一次是假话

改法 6 写进的几句：

- 「编译目录不删：它归这棵树的组，尝试停了就放回」（`task-server/task:67`），同一句前半写着取消删「派到第二台的连同那一趟的本地目录」（`task-server/task:66`，转述）
- 「任务记下结局、task clean、取消都不删它」（`task-server/attempt_runner.py:27`）
- 「任务记下结局、task clean、取消都不删它，树删了才跟着删」（`gpu-verification-server/control:392`）
- 「任务记下结局、task clean、取消都不删它」（`prune-server/control:163`，改法 3 在这一句里补了树）

第二台那一趟的本地目录就装着编译目录：「编译目录是它下面的 target」（`scripts/hosts/multi-host-run.sh:61`）；带 `--keep-build-directory` 的也只在等到退出码时留着：「没等到退出码的（中途失败、被信号停）照 ⑥ 删」（`scripts/hosts/multi-host-run.sh:12`）。服务这一侧照这样记：「没打的（中途失败、被停，⑥ 连它一起删了）组里删掉那一个」（`task-server/task_server_lib.py:276`），取消删产物那一段写「⑥ 没等到退出码连它一起删，收尾时组里删掉那一行」（`task-server/task_server_scheduler_building.py:248`）。

照喂的历史：会编的任务（例 `prune-server/control` 的编法提示那一条 `bash -c 'cargo build … && cp …'`）被服务派到第二台、在跑时主 agent `task-server/task cancel <号>`。按上面几行，第二台那一组里这一个编译目录被 ⑥ 删掉、组里删掉那一行；四句说明说「取消不删、放回」。同一棵树在第二台上下一条会编的任务从头编（推的，按代码与文件头推，没跑）。

这几句的来处是第一轮判决改法 6 点名的「照 task_server_lib.py:263、:286 改」（转述）；第 286 行本身就写「取消的照旧删它自己的现场，编译目录放回组里」（`task-server/task_server_lib.py:286`），与同一份文件头第 276 行对不上。`task-server/README.md` 第 170 行把例外写对了：「那一趟接着用组里那一个的，`multi-host-run.sh` 没等到退出码连它一起删」（`task-server/README.md:170`）；第 163 行的短句「编译目录归树的那一组，放回、不删」（`task-server/README.md:163`）没带这个例外。`.claude/skills/dispatch/SKILL.md:56` 的「服务当场删掉那一条的现场、编译目录放回它那棵树的组里」（`.claude/skills/dispatch/SKILL.md:56`）同一种，那一行这一轮没改，第一轮判过成立。`multi-host.env.example:114` 只讲本机的编译目录，本机取消确实不删，成立。

四句：不涉臂；说明写的是服务行为，服务当时看得到这一次在第二台、有没有等到退出码；满足的是正文第 4 问「有没有一处改出了新的假话」；第一轮改法 6「照服务现行说法改」照 :286 抄，在这一格上照中。

改法 D（被攻过零轮，推的）：这几句改成「本机的不删、放回组里；派到第二台而没等到退出码的（中途失败、被停）那一个连同那一趟的本地目录由 multi-host-run.sh 的 ⑥ 删，组里删掉那一行」，或只写「取消只删现场，编译目录照 task_server_lib.py 文件头「编译目录」」不再复述；`task_server_lib.py:286` 先改（不在这一轮被判的文件里，交主 agent）。

### 别的几样

| 处 | 结论 | 依据 |
|---|---|---|
| 「树」写全：kb `tooling` 三语第 42 行、`task-server/README` 三语第 224 行、`task-server/task_server_constants.py:289` | 推不出（低） | 判决与正文说「照同一句写全」（转述），这几处写成「没有就是 `--cwd` 本身」（`.claude/kb/tooling.md:42`）、「没有就是 --cwd 本身；它删了」（`task-server/task_server_constants.py:289`），少了「解开符号链接后」；原句是「树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层」（`task-server/task_server_lib.py:263`）。`--cwd` 是指向目录的符号链接、往上没有 `.git` 时，照这几处删掉链接本身，树不受影响、组不删（推的）。仓根与工作根此刻都不是链接（`realpath` 量的），平常碰不到 |
| 项目代码闸文件头的出路 | 低 | 「出路：不用自己删，删掉它的树，服务扫到树不在了就删它那一整组」（`.claude/hooks/project-code-guard.sh:212`）没带拒绝说明里那一句「主工作树那一组留给之后的任务接着编」（`.claude/hooks/project_code_guard_decisions.py:1214`）；树是主工作树时文件头照字面读是叫人删仓。拒绝说明本身是对的 |
| 「往上没有 .git 的删那个 --cwd 目录」 | 低（推的） | 树是 `--cwd` 本身而 `--cwd` 是工作根或数据根这种共用目录时，照字面是删工作根；检出钩子 ⑪ 拒 `rm -r` 工作根，走不通，那一组一直留着。出路没写「--cwd 是共用目录的不删、改在自己的子目录里交」 |
| 副本根上有 `.git` | 成立 | `scripts/hosts/minimal-copy.py` 拷完在副本根 `git init`（它文件头第 28 行，转述） |
| `task-server/task` clean 的说明与输出 | 成立 | 与 `clean`、`remove_scenes`、`shard_binaries_removal` 的实际删法一致：现场、分片父任务的测试二进制，编译目录不删；只许提交它的会话 |
| 完工清理闸文件头「现场」那一段 | 成立 | 逐句对过 `task-server/task_server_lib.py` 文件头「现场」；读法那一处的问题归第 1 问 |
| dispatch 技能第 18 行 | 见第 1 问 | 文字与闸的文件头对得上，但闸在真库上判不到 |

推翻条件：在第二台上跑一条会编的任务、跑到一半 `cancel`，之后 `task-server/task status` 或服务的组表里那一个编译目录还在、组里那一行还在。

## 几个改法各修哪一格

| 改法 | 第 1 问尝试表读法 | 第 2 问相对模型目录 | 第 3 问「」里只放角色名 | 第 4 问取消与第二台 |
|---|---|---|---|---|
| A 按任务号接任务表取尝试 | 推的，待主 agent 跑（第 2 项 `joined_query_rows` 只证查法取得到，不证改过的闸） | 不碰 | 不碰 | 不碰 |
| B 第 9 步与模板写明绝对路径 | 不碰 | 推的 | 不碰 | 不碰 |
| C 「」里有冒号才换掉角色名 | 不碰 | 不碰 | 推的（十三处现有命中都带冒号是量的） | 不碰 |
| D 改「取消都不删它」几句 | 不碰 | 不碰 | 不碰 | 推的 |

四个改法都只在这条腿的读法上核过、被攻过零轮，没有一个实现过。

## 要主 agent 跑的命令

1. 建运行目录（不交服务）：`mkdir -p /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-run`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-run -- bash /home/fy5090/code/singlefs/research/prompts/build-dir-clean-wording-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/build-dir-clean-wording-governance-r2-snapshot /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/build-dir-clean-wording-governance-r2-opus-model`
   - 在哪起：仓根（`--cwd` 已写在命令里）。
   - 看什么：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号> --all` 里：`name=compiles_declaration` 一行 `form=local-group-of-cwd-tree`（派到第二台的是 `form=peer-trip`，也算拿到）；`name=judged_file_hash` 四行都 `matches=true`；`name=kept_scene_store_shape arm=selftest-shape` 一行 `blocked=true scene_listed=true clean_named=true`；`arm=store-shape` 一行 `blocked=false`、`guard_query_rows=0 joined_query_rows=1`（第 1 问打中）；`name=quoted_role_names arm=current` 一行 `listed_lines=5,7`、`arm=quoted-role-names-judged` 一行 `listed_lines=2,3,4,5,6,7`（第 3 问打中；样本文件第 1 行是 shebang，第 2 到 7 行是第 3 问代码块里那六行）。每臂钩子的原样输出在运行目录 `kept-scene-<臂>-hook.stdout` / `.stderr`，判法的原样输出在 `comment-history-<臂>.txt`。
3. 建照字面那一形的运行目录（不交服务）：`mkdir -p /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-relative-run`
   - 在哪起：仓根。
4. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-relative-run -- bash research/prompts/build-dir-clean-wording-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/build-dir-clean-wording-governance-r2-snapshot /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus-relative-run research/prompts/build-dir-clean-wording-governance-r2-opus-model`
   - 在哪起：仓根。
   - 看什么：预期末行 `type=command-failed`、`exit=127`，日志里 `bash` 报找不到 `research/prompts/…/rerun-from-a-clean-work-directory.sh`（第 2 问打中）；跑成了就推翻第 2 问。
5. `task-server/task submit -- bash -c 'bash .claude/hooks/project-code-guard.sh < research/prompts/build-dir-clean-wording-governance-r2-opus-model/feeds/feed-step9-submit.json'`
   - 在哪起：仓根。
   - 看什么：`exit=0`，日志里没有 `✗` 行、没有 `permissionDecision`（放行）。
6. `task-server/task submit -- bash -c 'bash .claude/hooks/run-scope-guard.sh < research/prompts/build-dir-clean-wording-governance-r2-opus-model/feeds/feed-step9-submit.json'`
   - 在哪起：仓根。
   - 看什么：同第 5 项。
7. `task-server/task submit -- bash -c 'bash .claude/hooks/project-code-guard.sh < research/prompts/build-dir-clean-wording-governance-r2-opus-model/feeds/feed-step9-mkdir.json'`
   - 在哪起：仓根。
   - 看什么：同第 5 项。

第 5 到 7 项的喂法照第一轮主 agent 那几条（`{"tool_name": "Bash", "cwd": …, "tool_input": {"command": …}}`）写；第一轮实际怎么喂的这条腿没看到，主 agent 照自己那一套改。第 2、4 项判决引完数之后删两个运行目录，服务跟着删它们各自那一组。探针的内存与时间：没跑，估不出；两臂各起一次完工清理闸、一次注释历史判法，只扫各自的样本目录（推的，分钟以内）。

## 没打中的形状

- 第 1 问：分片子任务的会话与留现场口径、人手 `--manual`、第二台的现场路径、在排在跑待确认的任务、服务库读不了的三条路、五格自证与六条开关逐格对名字，都试过，读法那一处之外没打中。
- 第 2 问：会编声明的读法（`declared_header_words` 前 60 行、`startswith`）与模板自证 `grep -qx` 的松紧、`compiles_declared` 对 `--selftest` 与只列项参数的放过、工作根往上七层有没有 `.git`、外层清空与内存包装临时目录、cargo 输出目录的六种写法与仓里的 cargo 配置，都试过。服务撞内存上限重排时直接交 `rerun.sh`（不经外层）会因 `--work` 非空退 2，这是模板用法第 8 行那一形的老问题、外层就是为它写的，不算这一轮带进来的。
- 第 3 问：套着的「」、没收尾的「、「」里带轮次与日期、格名里带出处的那一种（文件头已登「靠 review」、正文已登去向），都试过。
- 第 4 问：`task clean` 的说明与输出、`TASK_SERVER_BYPASS_POLICY` 的两条出路、副本根上的 `.git`、`multi-host.env.example` 本机那一句、完工清理闸文件头「现场」一段，逐句对过，上表之外没打中。

## 这条腿自己的限度

- 模型一行都没跑：四问的打中都靠读代码推，模型只把推的那几行变成可核的输出。第 1 问最承重的一句（真库尝试行 session 列恒空）只读了写它的代码，没读真库（项目代码闸不许，第一次写探针时库文件名写进命令也被拒过）。
- 第 3 问全仓少报的范围是 grep 近似，不分注释与字符串。
- 第 4 问取消与第二台那一条没有在第二台上跑过。
- 写模型时有一次 `cd` 进了还不存在的目录，排他新建把探针落在了仓根，root-entry 钩子当场报出，已 `mv` 进模型目录，仓根这一次 `ls` 已不在；同一条命令里的 `rm -rf __pycache__` 在仓根执行，仓根此刻没有 `__pycache__`（量的），root-entry 钩子只报了那一个条目。

## 没做什么

- 不判辩方腿那两问；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数都没复跑（命令在上一节）。
- 没建仓副本、没建编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r2-opus/` 里只有几份 diff 摘录与拼 `rerun.sh` 用的一段文本，没有要删的副本与编译目录。
- 范围外交主 agent：`task-server/task_server_lib.py:286` 与 `remove_cancelled_artifacts` 文档串里同一句「取消…编译目录放回组里」；`scripts/experiment-apparatus/replay.sh:38` 的旧说法。
