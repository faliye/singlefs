# build-dir-clean-wording-governance-r3：正推腿报告（Sonnet）

三问各自的结论：

| 问 | 结论 |
|---|---|
| 第 5 问 | 「实现今天的样子」七条里，六条的文件内容与正文一致（文件:行 与原文见第一节）；换上中断那一条与几处运行读数（自证 116 格、86 个开关、开关复跑几条逐格相同）静态只能核到个数，运行结局待主 agent 跑。另有三处正文没说、读文件读出来的不一致（第一节改法 6 末）。 |
| 第 6 问 | 剩下的 9 个旧命名目录：属 5 棵树，5 棵树此刻都还在盘上；9 个是否仍登在组里读不了服务库，是推的，但有旁证（第二节 6.1）。派发闸与检出钩子对两份定义新写的命令：读文件推出「不判」，没喂过，喂法与对照臂写进「要主 agent 跑的命令」（第五节）。 |
| 第 7 问 | 全仓还剩 2 处「取消都不删」没写第二台那一支（`task-server/README` 三语第 224 行、`multi-host.env.example:114`），1 处有意留着的旧说法样本（`prune-server/control:174`）；「wait 末行 build=」「task clean 删编译目录」「副本删了」在非豁免位置没有别的真命中（第三节）。 |

## 开工前提

- 读了：共用约束（第 1–98 行；后面的部分没读，本报告用到的条款都在读到的范围里）、`three-way-inference.md`、`implementation-first.md`、第二轮判决、第三轮正文、三轮正文里点名的各文件、`records/2026-11-17-subagent账本清欠调度记录.md` 第 545–551 行。
- 没读：禁读清单点名的另一条腿这一轮的报告；`evidence-discipline.md` 只看了节标题、没读正文（这一轮没有建模型）。
- 没写模型：这一轮三问都是读文件与读盘上现状，没有要起的样本进程。喂钩子的输入文件在草稿目录 `feeds/` 里，sha256 在第五节。
- 草稿目录：`~/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-sonnet/`（认领过）；旧命名目录逐个的对照表 `old-named-directories.tsv`（sha256 `6970e30d36e723dce837eb8dc58dde9e1495d98dee590eef2df718d741296dbe`）、引文表 `quotes.tsv`（sha256 `8f9474fde39987dc10c5c8f27456462ef0f86637e84b38d25007d364bd8127c9`，每一句都 `grep -nF` 命中过）。
- 条目里出现的任务号形状的目录名不抄进报告（共用约束），逐个的名字只在草稿目录那份表里。

## 一、第 5 问：「实现今天的样子」逐条

### 改法 1（完工清理闸接任务表）：成立

| 正文说的 | 文件:行与原文 | 判定 |
|---|---|---|
| 取尝试改成 `attempts JOIN tasks ON attempts.task_id = tasks.id WHERE tasks.session = ?` | `.claude/hooks/completion_cleanup_guard_decisions.py:1305` `attempt_query = ("SELECT attempts.task_id, attempts.record FROM attempts JOIN tasks ON attempts.task_id = tasks.id WHERE "`；`:1306` `+ ("1" if any_session else "tasks.session = ?"))` | 一致 |
| 服务写尝试行会话列是空串 | `task-server/task_server_scheduler_starting.py:140` `"spec": specification, "state": "starting", "session": "", "command_key": task["command_key"], "command": task.get("command"), "started_at": round(time.time(), 3),` | 一致（这是闸按任务表判的前提） |
| `scene_due` 两个不到期条件 | `task-server/task_server_scheduler_building.py:358` `if not task.get("collected") and (task.get("session") or lib.broken("scene-sessionless-kept", self.environment)):`；`:360` `if (task.get("environment") or {}).get("SINGLEFS_KEEP_TEST_TEMPORARY_FILES") == "1" and not lib.broken("scene-keep-ignored", self.environment):` | 一致 |
| 闸两个条件都判 | `.claude/hooks/completion_cleanup_guard_decisions.py:1341` `uncollected = not top_record.get("collected") and not broken("kept-scenes-uncollected-unchecked")`；`kept` 在它上一行取环境里的 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES`；`.claude/hooks/completion-cleanup-guard.sh:35` 文件头「与 task-server/task_server_scheduler_building.py 的 scene_due 同一口径，两个都判」（整句见 quotes.tsv） | 一致 |
| 出路：结局没取走写 `wait`、设了留现场写 `clean`、两样都占写 `clean` | `.claude/hooks/completion_cleanup_guard_decisions.py:1411` `by_wait = uncollected and (not kept or broken("kept-scene-removal-uncollected-first")) and not broken("uncollected-scene-removal-by-clean")`：两样都占时 `kept` 为真、`by_wait` 为假，写 clean | 一致 |
| 自证样本库尝试行会话列写空串 | `.claude/hooks/completion_cleanup_guard_selftest.py:170` `connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, 'finished', '', '', ?, '')",` | 一致 |
| `unkept-scene-not-listed` 样本改成结局已取走 | `.claude/hooks/completion_cleanup_guard_selftest.py:434` `unkept = s.service_task(10, "finished", keep=False, collected=True)` | 一致 |
| 新格 `uncollected-scene-left` | `.claude/hooks/completion_cleanup_guard_selftest.py:442` `check("uncollected-scene-left", blocked_with(outcome, uncollected, f"{TASK_SERVER_CLIENT} wait 16", KIND_KEPT_SCENE)` | 一致 |
| 四条新开关 | `.claude/hooks/completion_cleanup_guard_decisions.py:84` `("kept-scenes-uncollected-unchecked", "uncollected-scene-left"),`；`:85` `("uncollected-scene-removal-by-clean", "uncollected-scene-left"),`；`:86` `("kept-scene-removal-uncollected-first", "main-kept-scene-left"),`；`:87` `("kept-scene-attempts-by-session-column", "main-kept-scene-left"),` | 一致；四条各自点名的格与实现里它改的那一处对得上（`-unchecked` 去掉取没取走的判，`-by-clean`、`-uncollected-first` 改出路，`-by-session-column` 退回按尝试表会话列取） |
| 自证「判了 116 格；86 个弄坏开关」 | 弄坏开关表现数：`awk` 数 `BREAK_TABLE` 里 `("…", "…"),` 行得 86（文件头与登记表第 10 行也写八十六条）；自证格数：`check("` 字面调用 112 处，加两处循环（一处循环 2 格、一处循环 2 格）共 116，与「116 格」相符 | 静态对得上，运行结局待主 agent 跑 |

什么现象会推翻它：`check(` 字面调用或循环次数数出来不是 116；`BREAK_TABLE` 条目不是 86；真跑出来「判了 N 格」的 N 不同；探针 `store-shape` 臂 `blocked=false`。

### 改法 2（两份定义与复跑模板的路径写法）：成立

| 正文说的 | 文件:行与原文 | 判定 |
|---|---|---|
| `three-way-attack.md` 第 9 步外层脚本与模型目录写成 `<仓根>/research/prompts/<轮>-opus-model/…`，写明相对路径照 `--cwd` 解 | `.claude/agents/three-way-attack.md:47` `（脚本路径、模型目录与 `<仓根>`、`<冻结副本>`、`<运行目录>` 都写绝对路径：命令里的相对路径照 `--cwd` 解，`--cwd` 换成运行目录之后相对仓根的路径找不到）` | 一致 |
| 复跑模板用法同改 | `scripts/agents/three-way-rerun-template.sh:9` `#     脚本路径、模型目录与 <仓根>、<冻结副本>、<新的草稿目录> 都写绝对路径：命令里的相对路径照 --cwd 解，--cwd 换成草稿目录之后相对仓根的路径找不到（bash 退 127）。` | 一致 |
| 对照补 `-r` 与 `--profile=<名>` | `.claude/agents/three-way-attack.md:35` `带 `--release`（或短写 `-r`）、`--profile release` 或 `--profile bench` 的是 `release`；带 `--profile <别的名>` 的是那个名；`--profile=<名>` 与 `--profile <名>` 落在同一层` | 一致 |
| `experiment-runner.md` 同改并补「`--cwd` 不是仓根时，命令位置上的脚本写绝对路径」 | `.claude/agents/experiment-runner.md:29` `命令里的相对路径照 `--cwd` 解：`--cwd` 不是仓根时，命令位置上的脚本写绝对路径（`<仓根>/scripts/experiment-apparatus/…`）`；同一行含 `-r` 与 `--profile=<名>` 两种写法 | 一致 |
| 模板开关 3 条 | `scripts/agents/three-way-rerun-template.sh` 文件头开关表 3 条（`root-from-self-path`、`root-check-by-wrapper`、`compiles-declaration-dropped`）；登记表 `.claude/gate.d/break-switch-replay-files.tsv:27` 写「三条开关」 | 个数一致；「逐格相同」是运行结局，待主 agent 跑 |

### 改法 3（注释历史判法带冒号才放过）：成立

- `scripts/gate/comment-history-check.py:250` `CELL_NAME_SEPARATOR = ":"`；`:513` `if CELL_NAME_SEPARATOR not in piece[start:end] and "quoted-role-names-colon-unchecked" not in switches:`（「」里没有 ASCII 冒号就不换占位符、角色名照判）。
- 新格 `:1072` `cell("quoted-role-name-without-colon",`；新开关 `:1261` `"quoted-role-names-colon-unchecked": "quoted-role-name-without-colon",`；文件头开关表 32 条（`^  COMMENT_HISTORY_CHECK_BREAK=` 行数 32），登记表 `:33` 写「三十二条开关」。
- 「门禁 doc-text comment-history 格单跑只剩 `scripts/gate/large-file-git-hook.py` 一行滞后行」：滞后表里确有 `scripts/gate/large-file-git-hook.py	1`；「单跑只剩它」是运行读数，待主 agent 跑。

### 改法 4（「取消都不删它」写全）：成立，另见第三节两处没改到的

照 `task-server/task_server_lib.py:276` `    没打的（中途失败、被停，⑥ 连它一起删了）组里删掉那一个（task_server.py 的 settle_build_slot）。两台的组互不相干。` 写全的位置，今天都带了第二台那一支：

| 位置 | 原文（第二台那一半） |
|---|---|
| `task-server/task_server_lib.py:292` | `派到第二台、中途失败或被停的那一趟（没等到退出码），multi-host-run.sh 的 ⑥ 连它用的组里那一个一起删，收尾时组里删掉那一行` |
| `task-server/task_server.py:66`（起 `:65`） | `本机组里的编译目录不删：它归这棵树的组，尝试停了就放回（task_server_lib.py 文件头「编译目录」）。派到第二台的，照那一次尝试日志里 multi-host-run.sh` 接 `⑥ 没等到退出码连它一起删`（`:69`） |
| `task-server/attempt_runner.py:27` | `本机组里的归这棵树的组，服务收尾这一次尝试时放回组里，任务记下结局、task clean、取消都不删它，树删了服务才跟着删`（限定「本机组里的」，第二台那一支在文件头，不是假话） |
| `gpu-verification-server/control:392` | `（本机组里的，任务记下结局、task clean、取消都不删它，树删了才跟着删；派到第二台、中途失败或被停的那一趟，` |
| `prune-server/control:163` | `（本机组里的，任务记下结局、task clean、取消都不删它；派到第二台、中途失败或被停的那一趟，multi-host-run.sh 的 ⑥ 连它用的组里那一个一起删；` |
| `task-server/README.md:163` | `（本机组里的编译目录放回、不删；派到第二台、中途被停的那一趟由 `multi-host-run.sh` 连它一起删）` |
| `prune-server/README.md:27` | `（任务记下结局、`task-server/task clean`、取消都不删本机组里的那一个，派到第二台、中途失败或被停的那一趟连它一起删；树删了才跟着删整组）` |
| kb `tooling` 三语第 42 行 | 中文「记下结局、`clean`、取消都不删本机组里的那一个（派到第二台、中途失败或被停的那一趟连它一起删）」，英、日各自对应 |

「几处『树』补『解开符号链接后』」：`.claude/hooks/lib_workspace_cargo_lock.py:230`、`task-server/task_server_constants.py:289`、kb `tooling` 三语第 42 行、`task-server/README` 三语第 224 行、`multi-host.env.example:114` 都带「解开符号链接后」；`往上第一层有 .git` 而不带「解开」的写法，全仓只剩与编译目录无关的三处（`.claude/hooks/handback-guard.sh:36`、`:802`、`.claude/hooks/project_code_guard_selftest.py:1466` 的断言串）。

### 改法 5：成立

`.claude/hooks/lib_workspace_cargo_lock.py:225` `lines.append("  （这一道堵的是主工作区里直接起的会编的 cargo 编进 <仓根>/target、删了又长的那条路）")`，前后已无「编译目录用完就删」；自证钉它不出现：`:309` `and "这一道堵的是主工作区里直接起的会编的 cargo" in message and "用完就删" not in message`。「自证与 `refusal-submit-dropped` 证红只红点名那一格」是运行结局，待主 agent 跑。

### 改法 6（服务清扫旧命名目录）：成立；读到三处正文没说的

| 正文说的 | 文件:行与原文 | 判定 |
|---|---|---|
| 加 `sweep_unpooled_attempt_label_directories`，由 `sweep_vanished_tree_pools` 扫完组以后接着调 | `task-server/task_server_scheduler_building.py:171` `            self.sweep_unpooled_attempt_label_directories()`（在 `sweep_vanished_tree_pools`，`:153` 起）；定义 `:195` | 一致 |
| 只删本库尝试标签、不在任何组里、不是在跑尝试用着、没有进程开着的目录，删了回读不在 | 组里的与在跑的取并集：`:212` `{os.path.realpath(attempt["build_directory"]) for attempt in self.store.running_attempts() if attempt.get("build_directory")})`；名字用 `re.compile(rf"task-{re.escape(self.store.tag)}-\d+-\d+")` 且 `fullmatch`；符号链接与非目录跳过；删走 `machine.remove_build_directory(directory)`（有进程开着不删）；没删成写告警 `build-directory-left`、删成才打「删了、回读不在了」 | 一致 |
| 三条新开关 | `task-server/task_server_lib.py:669` `  TASK_SERVER_BREAK=unpooled-label-directories-kept  本机编译根下不在任何组里、照尝试标签起名的目录不删（task_server.py 的 sweep_unpooled_attempt_label_directories）`，其后 `unpooled-label-directory-open-unchecked`、`unpooled-label-directories-pool-unread` 各一条；`task-server/task_server_lib_constants.py` 与 `task-server/task_server_scheduler_building.py` 各点名这三个 | 一致 |
| `task-server/README` 三语第 224 行补了这一支 | `task-server/README.md:224` 「同一扫另删本机编译根下不在任何组里、名字是 `task-<库标签>-<号>-<次>` 的目录（每次尝试一个目录那种旧命名；有进程开着的不删、写告警 `build-directory-left`、下一回再扫）」；英、日第 224 行对应 | 一致（但同一行前一句有一处没写全，见第三节） |
| 「每隔 10 分钟再扫」（README 同一行） | `task-server/task_server_constants.py:290` `BUILD_POOL_SWEEP_SECONDS = 600` | 一致 |
| 开关复跑「四条逐格相同」；重启后「删了 1 个、没删成 0 个」；12 个剩 11 个 | 重启后那一扫的读数是 `records/2026-11-17-subagent账本清欠调度记录.md:550` 的转述，不是我读到的运行结局；盘上今天仍是 11 个旧命名目录（`old-named-directories.tsv` 11 行），与「剩 11 个」相符 | 目录个数对上；其余运行结局待主 agent 跑 |
| 剩下里 2 个近 6 小时还有写入、是仍登在组里接着用的编译目录 | 盘上现查：11 个里恰有 2 个最近 6 小时内有文件在写，这 2 个的尝试记录的工作目录是主工作区；此刻一条在跑的 `cargo test` 的 `CARGO_TARGET_DIR` 就指到其中一个旧命名目录下的 `target`（读 `/proc/<pid>/environ` 所得，那条进程是别的会话在跑的） | 相符；「仍登在组里」的库内一半读不了，旁证见 6.1 |

没一致的三处（正文没提，读文件读出来的）：

1. 头注释的标记没跟着改。`task-server/task_server_lib.py:670`、`:672`、`:674` 三行都写 `「✗ task_server_lib.py 自证：unpooled-label-directories-removed：」（新开关，推的，没复跑）`，而正文说开关复跑过。要么标记过期、要么「开关复跑」没跑到这三条。
2. 登记表条数没跟着改。`.claude/gate.d/break-switch-replay-files.tsv:77` 写 `现数五百三十条开关`，`grep -c '^  TASK_SERVER_BREAK=' task-server/task_server_lib.py` 得 533（`task_server_lib.py` 文件头开关表行首的那一种写法）。差 3，正好是这三条新开关。这一列只是备注（门禁读它取路径与限时），不会红，但它自己写的「现数」不是现数。
3. 在跑尝试那一支没有自证格钉它。清扫的保护有三层：组里记着的、在跑尝试记着的、有进程开着的；自证格 `task-server/task_server_selftest_lifecycle.py` 的 `cell_unpooled_label_directories_removed` 摆的五个目录是「不在组里的、有进程开着的、组里记着的、别的库标签的、别的名字的」，没有一个是只靠「在跑尝试记着」才被保护的；三条开关里也没有一条拆这一层。把 `:212` 那一半删掉，这一格仍绿（推的，没跑）。

什么现象会推翻第 1–2 条：主 agent 照 `task_server_lib.py` 自证复跑那三条开关得到逐格红在 `unpooled-label-directories-removed`，并且头注释的标记与登记表条数是在读我这一份之后才改的。推翻第 3 条：自证里另有一格放一条在跑尝试、编译目录只记在尝试记录里，且那一格在去掉 `:212` 那一半时红。

### 换上中断一次：转述，一致

`records/2026-11-17-subagent账本清欠调度记录.md:549` 与 `:550` 写了第一次尝试撞内存上限、重排、第二次换上工具拒绝、主 agent 直接跑验证再补写完成，以及根子（服务只认命令那一层的写工作区工具）另派工具员；正文那一条与这两行一致。这一条是过程记录，不是文件内容，我没有别的来源能核，也不核。「task_server_lib.py 自证 4 格都对」是运行结局，待主 agent 跑。

### 第 5 问小结

- 改法 1–6 与记录行：文件内容部分逐条一致；运行读数（自证 116 格、开关复跑几条、清扫「删了 1 个」、`task_server_lib.py` 自证 4 格）我只能核到个数或目录现数，结局待主 agent 跑。
- 第 5 问没有「不成立」的条；三处不一致（头注释标记、登记表条数、在跑尝试没格钉）是正文没点名的附带发现。

## 二、第 6 问：前两轮只抽查、推的、没读的地方

找到的「推的」「没读」只有两处：第二轮判决第 7 条后面的 9 个旧命名目录（`records/2026-11-17-subagent账本清欠调度记录.md:550` `（其余 9 个属哪棵树、那棵树在不在，要读服务库，主 agent 不读，推的）`），与第一轮判决只写了项目代码闸与范围声明闸「两道闸都放行」、第一轮正文问了派发闸而判决没回（`research/prompts/_build-dir-clean-wording-governance-r1-body.md` 第 2 问写「项目代码闸、范围声明闸、派发闸会不会拒它」）。其余三份正文与两份判决里没有再写「只抽查」「没读」的地方（用 `grep -nE '没看|没量|没跑|没核|抽样|只看了|只抽|没复跑|未复跑|推的|没读|没有读|读不到|没喂|没试'` 扫过，命中都在上面两处或与这一问无关）。

### 6.1 剩下 9 个旧命名目录属哪棵树：只读能查到的查完了，「仍登在组里」那一半读不了

查法（只读）：每个目录的尝试记录 `~/.local/state/singlefs/task-server/attempts/<目录名>/spec.json` 里的 `"cwd"`；编出来的 `target` 里 `.d` 文件的源码绝对路径作旁证（主工作区那三个的 `.d` 里是 `<仓根>/crates/…`，其余用相对路径、绝对路径只在 `merge-and-verify/…/repo`、`judging-keys-input-data-only/copy`、`unlink-prefix-proofs-2026-11-19/repo` 的构建输出里出现，与 spec 的 cwd 一致；合入后验证槽里有一个目录的 `target` 里读不到绝对路径，只靠 spec）。逐个目录的结果在 `old-named-directories.tsv`。

| 树（--cwd） | 旧命名目录个数 | 树此刻还在盘上 | 近 6 小时有写入 |
|---|---|---|---|
| 主工作区（仓根） | 3 | 是 | 2 个有、1 个没有 |
| `~/.local/share/singlefs/work/unlink-prefix-proofs-2026-11-19/repo` | 1 | 是 | 没有 |
| `~/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo` | 1 | 是 | 没有 |
| `~/.local/share/singlefs/work/merge-and-verify/slot.kyz05duu/repo`（合入后验证的槽） | 3 | 是 | 没有 |
| `~/.local/share/singlefs/work/judging-keys-input-data-only/copy` | 3 | 是 | 没有 |

合计 11 个，其中 2 个有近期写入（正文说的 2 个）、9 个没有（正文说的 9 个）。9 个里主工作区 1 个、其余 4 棵树共 8 个。

- 属哪棵树：成立（上表）。树的认法照 `task-server/task_server_lib.py` 文件头「编译目录」第一行 `按「哪一棵树 × 哪一台」给：树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层`；cwd 在盘上，它往上那一层就也在，所以表里五棵树都不是「树没了」那一类。
- 那棵树在不在：五棵都在。所以 `sweep_vanished_tree_pools` 不会因树没了去删它们的组；这几个也没被 `sweep_unpooled_attempt_label_directories` 删（那一扫删过 1 个、今天仍是这 11 个）。
- 是不是仍登在组里：读不了。`build_slots` 在服务库里，服务没有列组的接口（`grep -n 'build_slots\|build-pool\|build_pool' task-server/task` 只有说明文字没有命令），库只由服务读写、闸拒子 agent 载入它；写成「推的」。旁证三条：（一）每棵树的个数是 3、1、1、3、3，都不超过 `task-server/task_server_constants.py:286` `BUILD_POOL_SLOTS_PER_TREE = 3`，三棵树恰好 3 个，是满组的样子；（二）清扫的保护条件（`task_server_scheduler_building.py:212` 那三层）下，这 9 个名字都符合本库标签、没有进程把工作目录放在里面（`ls -l /proc/[0-9]*/cwd` 里指到这 11 个旧命名目录下的 0 个），既然没被删，剩下的解释只有组里记着它们或在跑尝试记着它们；（三）此刻一条在跑的 `cargo test` 的 `CARGO_TARGET_DIR` 指到主工作区那 3 个里的一个（读 `/proc/<pid>/environ`），这说明主工作区的组确实还在用旧命名目录。
- 一个附带的观察：五棵树的路径各自算 `sha256` 前 8 位（`tree-9f6ec4dd`、`tree-539f2f3f`、`tree-1851f1a5`、`tree-592bbd43`、`tree-1ea10679`），都不在编译根下今天的 15 个 `tree-*` 目录名里。含义：这五棵树自服务改成按树命名以后没有再给过新目录，是它们的组仍停在旧名字上的另一个旁证。
- 另：`merge-and-verify/slot.kyz05duu/repo` 是合入后验证留着接着编的槽（`.claude/agent-common.md:37` 「照 `scripts/gate/merge-and-verify.sh` 文件头「槽位」留给下一趟接着编」），那 3 个目录只要槽在就不会因树没了被删；`unlink-prefix-proofs-2026-11-19/repo` 与 `judging-keys-input-data-only/copy` 是草稿目录里的副本，副本一删、服务下一回扫就删它们的组（`task-server/task_server_lib.py:283` `  删（「服务build之后必须删除历史」「自己用完自己删」照树算）：树删了编译目录跟着删——`）。

什么现象会推翻：服务库里 `build_slots` 没有这 9 个的行（那就是清扫该删而没删，是清扫的缺陷，要查为什么）；或 `old-named-directories.tsv` 里的 cwd 与 spec 不符。

### 6.2 第一轮没喂过的闸对两份定义新写的命令判不判

被判的命令一共四种形（写在 `.claude/agents/three-way-attack.md:35`、`:47` 与 `.claude/agents/experiment-runner.md:29`）：`mkdir -p <运行目录>`；`task-server/task submit --cwd <运行目录> -- bash <仓根>/research/prompts/<轮>-opus-model/tools/<外层脚本> <仓根> <冻结副本> <运行目录> <仓根>/research/prompts/<轮>-opus-model`；`task-server/task submit --cwd <副本> -- bash -c 'cargo build <参数…> && "${CARGO_TARGET_DIR:?}/<输出目录>/<bin 名>" <参数…>'`；`task-server/task submit --cwd scripts/experiment-apparatus -- bash -c '…'`（及它的 `cargo run` 那一形）。它们都是写给主 agent 跑的命令（腿只把它们写进报告「要主 agent 跑的命令」一节，腿自己不起）。

| 闸 | 它判的输入 | 对四种形我读出来的结局 | 凭什么 | 状态 |
|---|---|---|---|---|
| 派发闸 `.claude/hooks/runner-dispatch-guard.sh` | 主 agent 的派发提示（文件头第 2 行 `# PreToolUse hook（Agent）：派发那一刻查派发提示齐不齐、写得对不对`），不读 agent 定义文件 | 定义里新写的这几条命令本身不经它：定义正文随 agent 起来时读进腿的上下文，不走 Agent 工具的提示。只有主 agent 把这几句抄进派发提示时才轮到它 | 文件头 ①（叫子 agent 跑项目代码的句子）以「提示正文」里的句子为对象，对「交回的写法」放行（`:12` `交回的写法（要跑的命令写进报告「要主 agent 跑的命令」一节）`） | 推的；若抄进提示，「交法写两项：先 …，再 `task-server/task submit …`」这一句不带「写进报告」字样，有可能撞 ①（句子形与它同形），推不出，要喂 |
| 检出钩子 `.claude/hooks/bash-command-detector.sh` | 每一次 Bash 调用（`:497` `# 从 stdin 读 hook 的 JSON；拒绝七种之一退出 2（拒绝），其余退出 0`） | 四种形都不判：对主 agent 适用的几格（①看门狗、②前台等待循环、③放出追踪、④后台 `&`、⑤覆盖 `research/results/` 产物、⑥终止进程、⑦就地改脚本、⑪`rm -r` 作用在工作根、⑮⑯⑱git 与账本、⑰⑲往仓内文件写字面内容、⑳手拷副本、㉑前台超时、㉒绕开 git 钩子）没有一格的形状是 `mkdir -p`、`task submit … -- bash <脚本>`、`bash -c '… && …'`；`&&` 不是 ④ 要的单独 `&`；⑧⑨⑩⑫⑬⑭只判子 agent，而这几条命令腿不起 | 文件头 ①–㉒ 各条的形（`grep -nE '^# [①②…]'` 看得到每格开头） | 推的，没喂；喂法与对照臂见第五节 |
| 交回闸 `.claude/hooks/handback-guard.sh`（第一轮与这一问都没点名，但腿的报告里会写这几条命令，它判报告里的命令行） | 子 agent 报告「要主 agent 跑的命令」一节：一行最多一条 `task-server/task submit`，引号外不用 `;`、`&&`、`||` 串别的命令 | 第 3 种、第 4 种形里的 `&&` 在 `bash -c '…'` 引号里，不算（`:75` `#      同一条 submit 后面 bash -c '…' 引号里的 && 与 echo $? > 不算。只判含 task-server/task 的行，代码块里另判含 $? 的行（只看 echo $? >）。`）；`mkdir -p` 那一行不含 `task-server/task`、不判；自证里有同形的放行格（`:147` `「交回命令:同一条交任务命令里 bash -c 引号内的 && 与 echo $? > 放行 应当返回 0，实际 2」`） | 头注释写明 + 自证格名 | 推的，没喂（写进报告的命令一行一条即可过） |

对「不判」的补充：项目代码闸与范围声明闸（第一轮判决已喂过、第二轮判决又喂了 `mkdir -p` 与第 9 步 submit）、两道闸对这四种形放行，已有读数，不重复。

什么现象会推翻它：把 `feeds/` 里四条 JSON 经检出钩子逐条喂，任一条退 2（拒）或在 stderr 点名一格；把 `feeds/dispatch-with-step-nine-sentences.txt` 经派发闸 `--check-prompt` 喂，退 2 而对照臂 `feeds/dispatch-control.txt` 退 0（那就说明抄进提示的这两句会被拒，要在定义里换成带「写进报告」的整句，或在提示骨架里不抄这两句）。

### 6.3 这一问附带读到的一条检出（不是被判对象）

为核 6.1 的目录归属，我在 Bash 里写过一条 `git show HEAD:task-server/task_server_li` 接 `b.py`（文件名整个写出来）再 `| grep -c` 的命令，被项目代码闸拒：`✗ 绕开 task-server 的写法被拒：在代码里载入 task-server/task_server_lib.py（它管库与签名密钥）`。命令只是读 git 对象里的文件、没有载入它；闸文件的出路写「命令里只是搜这几个名字的，把名字拆开写」。这是闸对只读命令的误认，已按它的出路换了写法；是否算机制缺陷交主 agent 判，不在这一轮射程里。


## 三、第 7 问：全仓还剩的旧说法

范围：`git ls-files` 里去掉 `research/`、`records/` 之后的全部文本文件（草稿目录里 `files.nul` 存了这份清单，约 1.4 MB）；已知开着的 `.claude/agent-common.md:37` 与 kb `tooling.md` 第 42 行后半句领卡写法按正文不报。变更史（`.claude/kb/decisions-history.md`、`.claude/kb/experiments-history.md`）在清单内、没有命中。

扫的四类写法，对清单逐类 `xargs -0 -a files.nul grep -nIE`：「wait 末行 build=」用 `末行[^。；]{0,12}build=|build=<|build=目录|wait[^。；]{0,20}build=`；「取消都不删」用 `取消.{0,8}都?不删|取消.{0,12}不删`（另有英、日各一条）；「副本删了」用 `副本删了|副本被删|（副本删`；「task clean 删编译目录」用 `task clean[^。；]{0,12}删|编译目录[^。；]{0,30}(task clean|--clean|clean <)`；再按「树」的定义句扫 `往上第一层有 .git` 不带「解开」的。

### 命中，要改的

| 位置 | 原文 | 为什么是旧说法 | 轻重 |
|---|---|---|---|
| `task-server/README.md:224` | `任务记下结局、`task clean`、取消都不删组里的编译目录。` | 没写第二台那一支：照 `task-server/task_server_lib.py:292` 派到第二台、中途失败或被停的那一趟，`multi-host-run.sh` 的 ⑥ 连它用的组里那一个一起删，而这一句的主语是「组里的编译目录」，没限定本机。同一份 `:163` 的取消说明已写全 | 低到中（假话的那种；第二轮判决只判了这一行的「树」写全，没判这一句） |
| `task-server/README.en.md:224` | `Recording an outcome, `task clean` and a cancel do not remove the pool's build directories.` | 同上 | 同上 |
| `task-server/README.ja.md:224` | `結末の記録、`task clean`、取り消しでは組のビルドディレクトリを消さない。` | 同上 | 同上 |

### 命中，措辞不齐但不算假话

- `multi-host.env.example:114` `同一棵树先后交的任务接着用这一组，task clean、取消都不删，树删了服务才跟着删）`：这一行的主语是「本机的编译目录放在它下面」（同一行开头），限定了本机，对本机组里的成立；与别处写全的措辞不齐，改不改由主 agent 定。这份文件不在这一轮的被判清单里（`judged-sha256.txt` 没有它）。

### 命中，但是有意留着的

- `prune-server/control:174` `"task-server/task wait <号> 的末行 build=<目录>，二进制是 <目录>/release/prune-server（README.md「装」）")`：是弄坏开关 `build-hint-leaves-build-directory` 要造出来的错提示（`prune-server/control:172` `# 弄坏开关 build-hint-leaves-build-directory 下的提示：只让取编译目录里的二进制、没拷到共享根（task wait 的末行不点名编译目录，提示却叫人读末行取）`，被 `:239`、`:240` 选用），不是现行说法。

### 命中形状相近，核过不是旧说法

- `.claude/hooks/handback-guard.sh:1395` `副本删了就写补丁里那一份的路径`：副本没了、文件:行写补丁里那一份，与编译目录归树无关。
- `scripts/hosts/clean-session-scratch.py:490` 「克隆外的副本、编译目录与会话临时目录根本会话目录里的副本删了」：这个脚本自己清会话目录里手建的副本与编译目录，不是服务的组。
- `task-server/task_server_scheduler_admission.py:684` `不用了的仓副本删掉，它的编译目录组在服务下一回扫树时跟着删`：与 `task_server_lib.py:283` 一致。
- `scripts/mutation/crates_mutation_rows_stage.py:745` 「经服务交的编译目录按树给、task clean 不删它」：正确。
- `task-server/task_server_lib.py:293` `wait 末行不带 build=（编译目录归树、不归任务）`、`task-server/task_server_selftest_lifecycle.py` 第 715、734、756 行与 `task-server/README` 三语第 53 行「`wait` 末行不带 `build=`」：这些是现行说法。
- 「编译目录用完 `task clean` 删」「`wait` 末行 `build=`」的各种写法（含 `task clean … 编译目录`、`编译目录 … clean <`、`--clean` 与 `compile-then-swap.py`）：除上面几条，非豁免位置没有别的命中；`scripts/hosts/compile-then-swap.py` 的 `--clean` 删的是它自己在草稿目录里建的副本与编译目录，不是服务给的。

### 范围外顺带读到的一处不一致（不在这一问的旧说法里，只记）

`.claude/agents/three-way-forward.md:32`（同样 `.claude/agents/three-way-defense.md:32`）写 `收运行目录参数，开头先删掉那一个运行目录，再调同一个 `<模型目录>/tools/rerun.sh``，而 `.claude/agents/three-way-attack.md:47` 这一轮改成 `收运行目录参数，开头先清空那一个运行目录（目录本身留着，它是交这一条的 `--cwd`）`；范例 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh:22` 是 `rm -rf -- "${work:?}"`（整个删掉）。两份定义给的复跑外层脚本一个删目录、一个清空，一个要求交之前先 `mkdir -p`、一个靠 `rerun.sh` 重建。这两份定义没有被这一轮判，只当线索交主 agent。

什么现象会推翻第 7 问的「要改的」三行：`task-server/task_server_lib.py:292` 写的第二台那一支在实现里并不成立（`scripts/hosts/multi-host-run.sh:12` 写「没等到退出码的（中途失败、被信号停）照 ⑥ 删」，它是第二台那一支的实现依据）；或者正文把「组里的」限定成了本机组。推翻「全仓没有别的」：用别的写法（不含 `build=`、`取消`、`副本删了` 三个词）说同一件事的句子，我的正则扫不到。


## 四、只读命令的原样输出（我自己跑的）

```
$ ls -d ~/.local/share/singlefs/task-server-builds/task-* | wc -l
11
$ ls -d ~/.local/share/singlefs/task-server-builds/tree-* | wc -l
15
$ grep -nE '^BUILD_POOL_SLOTS_PER_TREE|^BUILD_POOL_SWEEP_SECONDS' task-server/task_server_constants.py
286:BUILD_POOL_SLOTS_PER_TREE = 3
290:BUILD_POOL_SWEEP_SECONDS = 600
$ ls -l /proc/[0-9]*/cwd | grep -c 'task-server-builds/task-'
0
$ cut -f3 old-named-directories.tsv | sort | uniq -c
     11 yes
```

头注释开关表现数 533 那一条（第一节改法 6 的第 2 点）我第一次跑得 533，没有存下原样输出，第二次重跑时被项目代码闸按「载入服务文件」拒了（见 6.3）；列进下面第 7 项由主 agent 复跑。

## 五、要主 agent 跑的命令

草稿目录 `D` = `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-sonnet`；喂的输入文件 sha256：

```
7a34c651261647bafe686d6c57f67bebbd5381700f6ba62625f266331b884808  feeds/detector-apparatus-bash-c.json
7ec780700f859ace3d38f6d3cbf979b8c7fde0eb54e63d73800ae058e1a745dc  feeds/detector-bash-c-prototype.json
2affb156c6c9745cdb860a3506a7068784ca3aeb86b82262708c02d83d534c68  feeds/detector-mkdir.json
65574fa7e1d027291b789b6b58c874e214eed3125be1f4314a4c9dbf6417654d  feeds/detector-wrapper-submit.json
f7bb32f23fe241c2d42c66903edd62656169e6713a2709f98b213934a083563a  feeds/dispatch-control.txt
04bbff2ced87988b6754c98ec5008894b51e27735016437f3039a5181a7b5074  feeds/dispatch-with-step-nine-sentences.txt
```

范围声明的种类与「跑什么」我按 `.claude/hooks/run-scope-guard.sh` 的形写，主 agent 交上去被拒就照拒绝说明改；我没有起过它们（子 agent 不起钩子与服务）。检出钩子的喂法是把 JSON 经 `bash -c '… < 文件'` 送进钩子的 stdin：

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=喂检出钩子一条 JSON（mkdir -p 运行目录）; 范围=1 条输入; 为什么=核定义新写的命令检出钩子判不判' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-sonnet -- bash -c 'bash /home/fy5090/code/singlefs/.claude/hooks/bash-command-detector.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-sonnet/feeds/detector-mkdir.json'`。在哪起：`--cwd` 已给。看：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0` 算放行（与我的推相符）；`exit=2` 或 stderr 点名某一格算拒（推翻 6.2 节检出钩子那一行）。
2. 命令：同 1，`跑什么` 改「（外层脚本 submit）」，把 `detector-mkdir.json` 换成 `detector-wrapper-submit.json`。看同 1。
3. 命令：同 1，换成 `detector-bash-c-prototype.json`（第 3 种形）。看同 1。
4. 命令：同 1，换成 `detector-apparatus-bash-c.json`（`experiment-runner` 那一形）。看同 1。
5. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=派发闸 --check-prompt 对照臂; 范围=1 份提示; 为什么=核派发闸对不含两句定义命令的骨架判不判，作对照' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-sonnet -- bash /home/fy5090/code/singlefs/.claude/hooks/runner-dispatch-guard.sh --check-prompt /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r3-sonnet/feeds/dispatch-control.txt --type three-way-attack`。看：对照臂必须 `exit=0`；退 2 说明我造的骨架本身缺一样输入，这一对比作废、改骨架重做。
6. 命令：同 5，把文件换成 `dispatch-with-step-nine-sentences.txt`。看：退 0 则抄进提示也放行（派发闸那一行「有可能撞 ①」不成立）；对照臂退 0 而这一臂退 2 且拒绝说明点名 ① 的那几句，则成立，交主 agent 判要不要改定义的写法。
7. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=任务服务头注释开关表现数; 范围=只读 grep; 为什么=核登记表现数五百三十与文件头现数是否差三' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash -c "grep -c '^  TASK_SERVER_BREAK=' task-server/task_server_li[b].py"`（方括号写法只是让命令文本不带整个文件名）。看：日志里单独一行数字，533 即与我读到的相同（登记表 `.claude/gate.d/break-switch-replay-files.tsv:77` 写五百三十，差 3）。
8. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=完工清理闸自证; 范围=整份自证; 为什么=复核正文「判了 116 格；86 个弄坏开关各红在点名的格」' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/hooks/completion-cleanup-guard.sh --selftest`。看：末行含「判了 116 格」与「86 个弄坏开关各红在点名的格」算与正文一致，`✗` 行 0。
9. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=门禁 doc-text 的 comment-history 格; 范围=单格; 为什么=复核正文「只剩 large-file-git-hook 一行滞后行」' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/doc-text.sh --check comment-history`。看：`✗` 行只点名 `scripts/gate/large-file-git-hook.py` 一份算与正文一致（我只核到滞后表里有这一行，没跑这一格）。
10. 服务库里 `build_slots` 的行（6.1「是否仍登在组里」）：我没有读它的办法（服务没有列组的接口、闸拒子 agent 载入库），写「复核不了」；主 agent 要核的话要走服务认可的路径，核不了的登进 `records/2026-11-19-未做完的任务登记.md` 作一条待核。

没有写模型，没有 `rerun.sh`，没有外层复跑脚本要复跑。探针的内存与时间：没起探针，估不出；上面各条都是几秒的单文件喂入或只读 grep。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 5：改法 1（接任务表、两条件都判、出路、自证样本、新格、四条新开关） | 成立 | 文件内容逐条一致（`completion_cleanup_guard_decisions.py:1305`、`:1341`、`:1411`、`:84`–`:87`，自证 `:170`、`:434`、`:442`）；116 格与 86 条开关静态数对上，运行结局待主 agent 跑 |
| 5：改法 2（定义与模板的绝对路径、`-r`、`--profile=<名>`） | 成立 | `three-way-attack.md:47`、`:35`，`experiment-runner.md:29`，模板 `:9` 与正文一致；模板开关 3 条个数对上 |
| 5：改法 3（带冒号才放过） | 成立 | `comment-history-check.py:250`、`:513`、`:1072`、`:1261`；开关 32 条个数对上；「单跑只剩一行」待主 agent 跑 |
| 5：改法 4（「取消都不删它」写全、「树」补解开符号链接后） | 成立 | 正文点名的位置都带第二台那一支；另有两处没改到，见第 7 问 |
| 5：改法 5（`lib_workspace_cargo_lock.py:225`） | 成立 | 前后矛盾的那句已去掉，自证钉 `:309`；自证只红点名那一格待主 agent 跑 |
| 5：改法 6（服务清扫旧命名目录） | 成立 | 函数、调用次序、三条开关、README 三语第 224 行都在；但头注释三行仍写「推的，没复跑」、登记表第 77 行条数差 3、在跑尝试那一层没有自证格钉它 |
| 5：换上中断一次 | 成立（转述） | 与 `records/…:549`、`:550` 一致；过程记录，不核文件 |
| 6：剩下 9 个旧命名目录属哪棵树 | 成立 | 5 棵树，3+1+1+3+3 个（含主工作区那 1 个），五棵树此刻都在盘上 |
| 6：9 个是否仍登在组里 | 推的（复核不了） | 库内 `build_slots` 读不了；旁证三条（个数不超 3、没被删且没有进程开着、一条在跑的 cargo 用其中一个） |
| 6：派发闸对定义新写的命令 | 推的（待主 agent 喂） | 它判提示、不判定义；抄进提示时「交法写两项…」那一句可能撞 ①，要喂 |
| 6：检出钩子对定义新写的命令 | 推的（待主 agent 喂） | ①–㉒ 没有一格的形状对得上这四种形 |
| 7：`task-server/README` 三语第 224 行「取消都不删组里的编译目录」 | 不成立（旧说法还在） | 没写第二台那一支，与 `task_server_lib.py:292` 对不上 |
| 7：`multi-host.env.example:114` | 推不出（措辞不齐） | 主语限定了本机，不算假话；这份文件不在被判清单里 |
| 7：`prune-server/control:174` 的 `build=<目录>` | 成立（有意留着） | 是弄坏开关造的错提示，不是现行说法 |
| 7：别的「wait 末行 build=」「task clean 删编译目录」「副本删了」 | 没有别的命中 | 非豁免位置只剩上面几处；核过形状相近的 5 处都不是旧说法 |

## 七、没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务（子 agent 不交任务），判决要引的数没复跑（命令在「要主 agent 跑的命令」一节，状态「待主 agent 跑」）。
- 没读服务库、没起钩子与门禁、没有起任何项目代码；`build_slots` 那一半「复核不了」。
- 没读禁读清单里另一条腿的报告，没读第二台上的任何东西（只读本机盘）。
- 没改任何被判文件；`three-way-forward.md` 与 `three-way-defense.md` 那条运行目录「删掉 / 清空」不一致只记在第三节末，没有当结论。
