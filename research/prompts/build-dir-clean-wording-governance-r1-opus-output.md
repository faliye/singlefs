# 编译目录旧说法收口治理文件轮第一轮：攻方腿报告（build-dir-clean-wording-governance-r1，2026-11-21）

被判的 28 份文件对 `research/prompts/build-dir-clean-wording-governance-r1-snapshot/judged-sha256.txt` 跑过 `sha256sum -c`，28 份都对上（输出只打不对上的行，空）。改前原样读的是正文点名的各草稿目录 `base/` 与 `git show HEAD:<路径>`。这条腿没写模型（没有 `rerun.sh`）；要核的只有三份喂钩子的输入，放在草稿目录，命令在「要主 agent 跑的命令」一节。

喂钩子的三份输入（`sha256sum` 原样输出）：

```
da2a3bceb3d7eabcce33a13bc99723b1b18e7f071fd6a2f6f3dba97fb47470aa  /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-experiment-runner.json
355db4da745a399e03d8ea4a44c16900284308373f1aa8728baefa360cf3b492  /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-prune-server-hint.json
586ea849417f260f0bd70dd423839548220d6cc97dbe5913317b1c30e5de7cdc  /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-three-way-attack.json
```

## 各格判定一览

| 问 | 判定 | 打中的东西（一句） | 严重程度（推的） |
|---|---|---|---|
| 第 1 问 完工清理闸删支后的漏拦 | 推不出（打中） | 删掉的那一支是主 agent 收工时唯一会叫它起 `task clean <号>` 的地方；设了留现场变量的任务现场只由 `task clean` 删，删支之后收工时谁都不提；`.claude/skills/dispatch/SKILL.md:18` 还写着完工清理闸拦下「这几样」（含临时现场）。另：新文件头把「树」说成「副本」，与服务的树定义差一种情形 | 中（现场残留占共享工作根）；文件头措辞低 |
| 第 2 问 两份定义新写的编与跑命令 | 推不出（打中） | 命令照字面交服务跑得起、第二台拿得到 `CARGO_TARGET_DIR`、闸按代码推都放行（三份喂钩子待主 agent 跑）；但 `.claude/agents/three-way-attack.md:35` 的取二进制写法搬进同一份定义第 8、9 步要求的 `rerun.sh` 里跑不起（那一条交服务时服务不给 `CARGO_TARGET_DIR`），定义没说那里怎么取；`<debug 或 release>` 的判法漏 `--profile`、`--example`、`--target` | 中低（失败是响的：退非 0，不出错数） |
| 第 3 问 新出路照做走不走得通 | 推不出（打中，低） | 三处出路对仓副本与主工作树走得通；`--cwd` 往上没有 `.git` 的任务（树就是 `--cwd` 本身，例：草稿目录），「删掉自己建的那份仓副本」删不掉它那一组，出路没写这一种 | 低 |
| 第 4 问 乙组标记与删行 | 推不出（打中，低） | 五处标记没有一处在讲来历；但检出钩子新加的四处标记的理由不在共用约束「特殊缘由只三类」之内，是给门禁词表的误判单行放过（共用约束第 79 行要修门禁或登常设豁免）；滞后表删四行、搬迁表删十四行成立 | 低 |

## 第 1 问：完工清理闸删掉「服务编译目录」那一支之后漏拦什么

### 删支本身该不该

该删。服务今天 `task clean` 不删组里的编译目录：「任务记下结局、task clean <号>、取消都不删组里的」（`task-server/task_server_lib.py:286`）。旧那一支照「本会话有结局、尝试记的编译目录还在盘上」列单（改前 `completion_cleanup_guard_decisions.py` 的 `service_builds`），给的删法是 `task clean <号>`；`task clean` 也不删任务行（`task-server/task_server_scheduler_starting.py` 的 `clean` 只删现场与分片测试二进制），留着那一支，每次收工都会列同一批删不掉的目录，第二次照「同一张单放行」交用户，等于永远拦一次。这一点不打。

### 打中：删支丢掉的是「收工时叫主 agent 起 `task clean`」这一推，丢下的对象是任务现场

1. 只由 `task clean` 删的现场有一类：「设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的只由 task clean 删。」（`task-server/task_server_lib.py:297`）。没人 `wait` 的任务现场同样「一直留到 task clean」（`task-server/task_server_lib.py:301`），这一类服务自己会反复推结局（「每翻一倍（60、120 分钟…）再推一回，推到取走为止」（`task-server/task_server_lib_constants.py:136`）），取走之后一小时自动删，不算漏；设了留现场变量的那一类取走之后不会自动删，也没有人推。
2. 改前：会编的任务一有结局，旧那一支就把它的编译目录列进收工单，删法 `task clean <号>`，主 agent 照删时连同现场一起删了（`clean` 里「各次尝试的现场一并删」那一段）。设了留现场变量、又会编的任务（`cargo test` 一类，正是这个变量常用的场合），收工时就这样被带着清掉。
3. 改后：完工清理闸文件头里一个「现场」都没有（`grep -n '现场' .claude/hooks/completion-cleanup-guard.sh` 零命中，跑过、退 1、无输出），新写那一段只说「主 agent 收工时该删的是自己建的副本，归上面认领目录与临时草稿目录那几处判。」（`.claude/hooks/completion-cleanup-guard.sh:28`）。现场在 `<工作根>/task-server-scenes/<尝试标签>`，不在任何认领目录里，迁库遗留那一扫只认库拷贝、旧文件库与库备份。
4. 同一轮改过的技能还说它拦：「要留测试的临时现场，在 submit 前面设 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`。完工清理闸 `.claude/hooks/completion-cleanup-guard.sh`（挂 Stop）在结束一轮时拦下还留着的这几样」（`.claude/skills/dispatch/SKILL.md:18`）。照字面读，「这几样」含刚说的临时现场；闸不判它。
5. 现存状态（量的，`ls -la ~/.local/share/singlefs/work/task-server-scenes`）：13 个现场目录，修改日期最早的三个是 11 月 4 日、11 月 6 日、11 月 12 日，都远过「取走后一小时」。`ls` 原样带钟点与带任务号的尝试标签，照共用约束不贴原样，只转述。这三份早于这一轮改动，说明旧那一支在时也没清干净（多半是不编的任务或别的会话的），只当「现场会长期残留」的旁证，不当「删支造成」的证据。

能照喂的历史（主 agent 在一个会话里做；推的，没跑）：

1. `task-server/task submit -- env SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 cargo test -p singlefs-verification --lib -- <任一测试全名> --exact`（前面照范围声明闸带 `SINGLEFS_RUN_SCOPE`）。
2. `task-server/task wait <号>` 取走结局。
3. 结束这一轮（手里没有后台任务与子 agent）。完工清理闸放行、什么都不打。
4. 一小时之后 `ls ~/.local/share/singlefs/work/task-server-scenes/` 还看得到那一次尝试的目录，直到有人 `task-server/task clean <号>`。改前同一段历史第 3 步会被拦、单子上点名 `task-server/task clean <号>`。

四句：

- 分不分辨臂：分辨。「留旧支」与「删支」两臂在第 3 步不同（前者拦、后者放）；「删支并改判现场」一臂（没人提）在第 3 步拦、在第 4 步干净。
- 系统当时看不看得到：看得到。旧那一支读的就是服务库里本会话的任务与尝试记录（只读打开）；现场路径记在尝试记录的 `scene_directory`（`task_server_lib.py` 文件头「现场」那一段）。
- 满足判据的哪一句：第 1 问「主 agent 收工时还留着、本该被拦下的东西（任务现场、设了留现场变量的现场…）有没有一样因此漏拦」的「设了留现场变量的现场」那一分句。「本该被拦」这一前提，依据是用户原话「用完就要删掉。不要残留了」（`task-server/task_server_lib.py` 文件头「现场」那一段起头）与共用约束「写」一节「用完就删」；工具员的反依据是 `:297`（设了的是有意留的）。有意留不等于一直留，这一处推不出谁对，所以记「推不出」。
- 跑前条款的改法在这一格上中不中：正文没给跑前改法。工具员「不改成判任务现场」那一判，在这一格上照样中（它就是这一格本身）。

### 次要：新文件头那一段与服务文件头对不对得上

- 「树删了（副本删了）服务跟着删它那一整组」（`.claude/hooks/completion-cleanup-guard.sh:27`）把树等同副本。服务的树是「往上找到的第一层有 .git（目录或文件，主仓与工作树都算）的那一层」（`task-server/task_server_lib_configuration.py:606`）、「哪一层都没有交解开之后的 directory 本身」（`task-server/task_server_lib_configuration.py:607`）。仓副本经 `minimal-copy.py`「在副本根 git init」（`scripts/hosts/minimal-copy.py:28`），对副本说得通；`--cwd` 往上没有 `.git` 的（草稿目录、`rerun.sh` 拷出的 `$WORK/tree`，后者 `rsync --exclude .git`）树就是 `--cwd` 本身，删副本删不到它。闸不判服务编译目录，所以这一处只是说明不准，不是漏拦。
- 「TASK_SERVER_BUILD_DIRECTORY 下按树分的组」只说了本机；第二台那一组在多机配置 `BUILD_ROOT_BY_HOST` 那一项下的 `builds/<标签>/target`（`task-server/task_server_lib.py` 文件头「编译目录」两台同一段）。闸只在本机跑，不影响判，只是定义窄了。
- 合入后验证快照与副本那几处判法没动（文件头「主 agent 另判合入后验证留下的快照」那一段与认领目录那一段逐字未改，`diff` 动的是第 27–28 行那一段、「删不掉的」那一段删去「与服务编译目录」、退役判据删去服务编译目录一句与开关表删五条），不漏。

## 第 2 问：两份定义新写的「编与跑写成一条交」

### 照字面交给服务：跑得起（推的，按代码读）

- 服务认得它会编：`compiles` 调 `cargo_invocations(shlex.join(task["command"]), …)`（`task-server/task_server_scheduler_starting.py:295`），`cargo_invocations` 把剥出来的那条「里面的 bash -c、cd 照 lib_shell_words 跟着走」（`.claude/hooks/lib_workspace_cargo_lock.py:85`），自证里有一格「编译目录：bash -c 里起：拒」（`.claude/hooks/lib_workspace_cargo_lock.py:299`）钉着 `bash -c` 里的 `cargo build` 被认出。认出之后本机给 `environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")`（`task-server/task_server_scheduler_starting.py:102`）。
- 第二台：`multi-host-run.sh` 每一趟给那条命令 `build_text="CARGO_TARGET_DIR=$(printf '%q' "$peer_scratch/target") "`（`scripts/hosts/multi-host-run.sh:1419`），命令的词经「${wrapper_text}$(printf '%q ' "$@")」（`scripts/hosts/multi-host-run.sh:1453`）逐词转义带过去，`bash -c` 那个词里的 `${CARGO_TARGET_DIR:?}` 到第二台才展开，取的是第二台那一个。`--peer` 命令里写了 `CARGO_TARGET_DIR=` 才算用法错，判法是「(export[[:space:]]+)?CARGO_TARGET_DIR=」（`scripts/hosts/multi-host-run.sh:932`），前面要接空白或 shell 符号、后面要接等号；`${CARGO_TARGET_DIR:?}` 前面是 `{`、后面是 `:`，不中。编与跑在同一次尝试、同一台上，不会编在一台跑在另一台。
- 项目代码闸：经 submit 交的那一条「主 agent 交什么都放」、`--` 之后那条「只为认出里面绕开服务的写法（③）」（`.claude/hooks/project-code-guard.sh:200`）；③ 管的是点到库、密钥、socket、准入变量与删服务编译目录，这条命令都不点。
- 范围声明闸：`cargo build` 与跑编出来的二进制都在「不要声明的形态」里：「cargo 的别的子命令（build、clippy、fmt、run、bench……）」（`.claude/hooks/run-scope-guard.sh:129`）、「上面几类之外的命令：钩子与研究脚本的 --selftest」（`.claude/hooks/run-scope-guard.sh:130`）。
- 交回闸 ⑧：判的是「有没有引号外的 ; ;; && ||」（`.claude/hooks/handback-guard.sh:1148`），`bash -c '… && …'` 的 `&&` 在单引号里，不算串接。
- 服务的提交时警告（取仓根 `target/`）跳过带 `$` 的段：`if REPOSITORY_TARGET_NAME not in piece or "$" in piece:`（`task-server/task_server_lib_commands.py:238`），不误报。
- 派发闸：判的是派发提示，两份定义的这一句只叫子 agent 把命令写进报告，不叫它跑；推的，没喂。

三份喂钩子的输入（项目代码闸与范围声明闸各喂一次，共六条）写在「要主 agent 跑的命令」，结局出来之前上面「闸放行」几条只算推的。

### 打中一：`three-way-attack.md` 第 35 行的取法搬进第 8、9 步的 `rerun.sh` 跑不起，定义没交代

- 第 35 行要原型「二进制取这一条命令里服务交下来的 `CARGO_TARGET_DIR`」（`.claude/agents/three-way-attack.md:35`）。同一份定义第 8 步要模型的复跑写进从模板拷来的 `rerun.sh`、「只改 `rerun_steps` 那一段」（`.claude/agents/three-way-attack.md:46`），第 9 步要「`<模型目录>/tools/rerun.sh` 整份跑一次」（`.claude/agents/three-way-attack.md:47`）。复跑里自然要再编那个原型。
- 模板的交法是「task-server/task submit -- bash <模型目录>/tools/rerun.sh --repo <仓根>」（`scripts/agents/three-way-rerun-template.sh:7`）。命令位置上是 `bash <脚本>`，`cargo` 在脚本正文里；服务认会编只看命令那一层与脚本文件头的声明：「命令位置上起的脚本文件头前 DECLARED_HEADER_LINES 行里写」（`task-server/task_server_lib.py:278`）那一行声明。模板文件头没有这一行（`grep -n 'task-server-compiles' scripts/agents/three-way-rerun-template.sh` 零命中），第 8 步又只许改 `rerun_steps`，加不上；外面那层包装（第 9 步的范例）也没有。
- 所以照第 35 行写进 `rerun_steps` 的 `cargo build … && "${CARGO_TARGET_DIR:?}/release/<bin>"`，在复跑里 `cargo build` 编进 `$WORK` 下那棵拷贝自己的 `target/`，接着 `${CARGO_TARGET_DIR:?}` 报「parameter null or not set」退 1，复跑整份红。
- 已入库的范例绕开了这一点，用的是兜底写法：「local binary="${CARGO_TARGET_DIR:-$WORK/copy/target}/release/floor-raise-precheck-r3-probe"」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun.sh:99`）。第 35 行的 `:?` 与它相反。

能照喂的输入（推的，没跑）：腿把 `rerun_steps` 写成 `( cd "$WORK/tree" && cargo build --release -p <原型包> && "${CARGO_TARGET_DIR:?}/release/<原型>" )`，主 agent 照第 9 步交 `task-server/task submit -- bash <模型目录>/tools/rerun-from-a-clean-work-directory.sh <运行目录>`；末行 `TASK_SERVER_RESULT … type=command-failed exit=1`，日志末尾是 `CARGO_TARGET_DIR: parameter null or not set`。

四句：分辨臂（「`:?`」与「`:-` 兜底」两种写法在同一段复跑上一红一绿）；系统看得到（服务给不给 `CARGO_TARGET_DIR` 只看命令那一层与文件头声明，脚本正文里的 `cargo` 它不读，这是设计，不是缺陷）；满足的是第 2 问「交给服务之后，跑不跑得起」那一分句；正文没给跑前改法。

### 打中二（低）：`<debug 或 release>` 的判法写窄了

第 35 行写「`<debug 或 release>` 照 `cargo build` 带没带 `--release`」（`.claude/agents/three-way-attack.md:35`）。cargo 的目录名照 profile 定：`--profile release` 与 `--profile bench` 不带 `--release` 也进 `release/`，`dev` 与 `test` 进 `debug/`；`--example <名>` 的在 `<profile>/examples/<名>`；带 `--target <三元组>` 的多一层 `<三元组>/`。仓里今天只有 `release`、`dev` 两个 profile（`grep -n '^\[profile' Cargo.toml` 两行），没有自定义名，所以实际会撞上的只有 `--profile release` 与原型写成 example 这两种。`experiment-runner.md` 第 29 行写死 `--release` 与 `release/`，不受影响。失败都是响的（找不到文件退 127），不出错数。（cargo 的目录规则是我印象里的，没在这一轮现查 cargo 文档或跑 cargo，标「推的」。）

### 没打中的

`.claude/agents/experiment-runner.md:29` 的 `--cwd scripts/experiment-apparatus` 是主工作区里的相对路径：树取 `.git` 所在的仓根，编译目录是主工作树那一组，在仓外，不撞「编进主仓」那一判（那一判不判交给服务的那一条，`.claude/hooks/project-code-guard.sh:201`）。`e176_crates_oracle/Cargo.toml` 新注释的「编」那一条单独交也无害（同一棵树同一台接着用），「跑」那一条用 `cargo run` 编与跑在一起，第二台也对。

旁边看到、不在被判文件里的：服务切词出异常时退回「只看命令本身那一层」（`task-server/task_server_scheduler_starting.py:297`），那一支判的是「return bool(words) and os.path.basename(words[0]) == "cargo"」（`task-server/task_server_scheduler_starting.py:301`），`bash -c '…'` 的头一个词是 `bash`，判成不会编、不给 `CARGO_TARGET_DIR`；`.claude/agents/experiment-runner.md:29` 那一条在仓里起，这时 `cargo build` 会编进 `scripts/experiment-apparatus/target`（主仓里）再被 `:?` 拦下。只在切词库出事时走到，交主 agent 看要不要另记一笔。

## 第 3 问：新出路照做走不走得通

| 出路 | 出处 | 照做 | 过不过得了别的闸（推的） |
|---|---|---|---|
| 直接删服务编译目录被拒，改「删掉自己建的仓副本」 | 「服务给的编译目录不用自己删：要它没，删掉自己建的那份仓副本，服务下一回扫到树不在了就删它那一整组；主工作树那一组留给之后的任务接着编。」（`.claude/hooks/project_code_guard_decisions.py:1213`），共用库同句「树删了（自己建的仓副本删了）服务跟着删它那一整组）」（`.claude/hooks/lib_project_code_runs_base.py:1127`） | 对仓副本走得通：副本根有 `.git`，树就是副本，删了之后服务每 600 秒扫一回（`task-server/task_server_constants.py` 的 `BUILD_POOL_SWEEP_SECONDS = 600`）删那一组；对主工作树写明了不删 | 删副本是 `rm -rf <副本>`，落在自己认领的目录里，检出钩子 ⑪ 只拒工作根本身与带通配的第一层，项目代码闸 ③ 只拒点到服务编译目录的删；过 |
| 主仓里直接起 cargo 被拒，改经 submit | 「服务按这棵树给一个仓外的编译目录、经 CARGO_TARGET_DIR 交给命令」（`.claude/hooks/lib_workspace_cargo_lock.py:228`） | 走得通：照抄的那条再过项目代码闸（主 agent 交的放）；带 `cargo test` 的要在 submit 前面加范围声明，出路已写「范围声明写在 submit 前面」 | 过（自证格「编译目录：拒绝说明给出经 task-server 提交的出路」钉着串） |
| `prune-server/control install` 缺二进制的编法提示 | `BUILD_COMMAND`（`prune-server/control:165-169`） | 走得通：编与 `cp` 同一次尝试、同一台，`${CARGO_TARGET_DIR:?}` 两台都有；`<共享根下的目录>` 照提示取在 `NFS_SHARED_ROOTS` 下，`task submit` 不会因共享根外的绝对路径退 3 | 喂钩子待跑（输入 `feed-prune-server-hint.json`） |

打中（低）：第一行那条出路只写了两种树。`--cwd` 往上没有 `.git` 的任务，树就是 `--cwd` 本身（`task-server/task_server_lib_configuration.py:607`）。能照喂的历史：腿或主 agent 在草稿目录里写了一个独立的小 cargo 工程（共用约束允许草稿目录里写自己的东西），交 `task-server/task submit --cwd <草稿目录> -- bash -c 'cargo build --release --manifest-path <草稿目录>/probe/Cargo.toml && "${CARGO_TARGET_DIR:?}/release/probe"'`。这一组挂在草稿目录名下；之后想腾盘，照出路删「自己建的那份仓副本」，那一组不动，要删的其实是草稿目录本身，出路没说。工作根下一层此刻有 3208 个目录（`ls -la ~/.local/share/singlefs/work` 列 `.` 那一行的链接数 3210 减二，量的），草稿目录大多长期留着。四句：分辨「删副本」「删 `--cwd` 那一层」两种出路；服务看得到（组里记着树路径）；满足第 3 问「照做之后走不走得通」那一分句；没有跑前改法。

## 第 4 问：乙组的标记与删行

### 五处 `history-kept`：没有一处在讲来历

- `.claude/hooks/runner_dispatch_guard_locks_and_verdicts.py:672`：「history-kept: 被判的标题写法示例，日期与重跑次数本身是数据」。那一行是文档串里列它认的实验页标题形态，日期是被认的数据，正是共用约束列的第一类（「解析历史条目的代码讲它认的形态」）。成立。
- `.claude/hooks/bash-command-detector.sh` 第 636、804、965、1027 行：「history-kept: 自证格名里的三方腿角色名，不是来历」（`.claude/hooks/bash-command-detector.sh:636`）。这几行列的是弄坏开关红在哪几格，格名里有「正推腿写别的轮的报告，拒」，命中的是词表里的「正推腿」（「定案  攻方腿  辩方腿  正推腿」（`scripts/gate/comment-history-check.py:37`））。它是现行角色名（`three-way-forward`），不是来历。内容上成立。

打中（低）：这四处标记的理由不在许用的类别里。共用约束写「特殊缘由只三类」（`.claude/agent-common.md:47`），门禁的出路也只列这三类：「确属特殊缘由（日期本身是数据、引外部文档写读到它的那一天、绕开外部工具缺陷写从哪个版本起）的」（`scripts/gate/comment-history-check.py:768`）。「词表把现行角色名当成了历史词」是门禁判错，共用约束对这一种的出路是「是机制或门禁判错的，修那道机制或门禁、或登进它的常设豁免表，不单次放宽、不只豁免这一次」（`.claude/agent-common.md:79`）；逐行加标记就是逐行放宽。同一份文件里早有三处同一类用法：「history-kept: 自证用例标签原文，按原文匹配」（`.claude/hooks/bash-command-detector.sh:582`，另两处在第 726、1133 行），说明这是现行做法与条文对不上，不是这一轮独有。门禁只判理由长度，不判类别，所以门禁不会红。

四句：不分辨「这一句讲不讲来历」（五处都不讲），分辨「标记」与「修词表或登常设豁免」两种处置；门禁看得到格名在 `「」` 里，词表那一判当时看不到；满足的是第 4 问「标记理由站不站得住」那一分句（理由真，但类别不合条文），不满足「把真在讲来历的句子豁免掉了」那一分句；没有跑前改法。

另两处改写：

- `.claude/hooks/write-guard.sh:107`「被改那几份样本此刻的样子拷进 <草稿目录>/fixtures-base/」：说的是运行时先后（改之前先拷一份），意思没变。成立。共用约束第 94 行同一件事还写「改前的拷进 `fixtures-base/`」，那一份不在注释射程里，不算。
- `.claude/hooks/write_guard_selftest.py:602` 删掉「攻方第二轮 3f」：删的就是来历。成立。

### 滞后表删的四行

四份文件在 `git show HEAD:.claude/gate.d/comment-history-lag.tsv` 里各 0 行、此刻表里各 0 行（`grep -cF` 量的），基准里没有就不许加，删得对。删了之后要四份文件注释里命中是 0；我拿词表的正则对四份文件整份 `grep -P`（不分注释与字符串）：检出钩子 7 行全带标记，派发闸那一份 2 行里 1 行带标记、另 1 行在字符串里（第 961 行拒绝说明的串），写闸 0 行，写闸自证 46 行都在测试输入字符串里。按这个读法注释里没有不带标记的命中；我的 `grep` 分不出注释，这一句是推的，以门禁单跑为准（正文说已单跑过，我没复跑）。

### 搬迁表删的十四行

十四行是七份钩子的部分文件从连字符名改成下划线名时登的；连字符名在 HEAD 里一份都没有（`git show HEAD:.claude/hooks/completion-cleanup-guard-decisions.py` 退非 0），旧路径不在基准里就不算搬迁。删了之后旧名还有没有人引：拿十四个连字符文件名（取自 `~/.local/share/singlefs/work/split-gov-hooks-to-imports/old-hyphen-parts/` 下的十四份备份）对 `.claude`、`scripts`、`task-server`、`prune-server`、`crates` 与 `CLAUDE.md` 做 `grep -rlE`，零命中。不漏判。成立。

## 改法（都是推的、被攻过零轮，没在模型上量过）

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 完工清理闸主 agent 收工时另判本会话有结局、现场目录还在、设了留现场变量的任务（读法照删掉的那一支：服务库只读打开，取尝试记录的 `scene_directory`），删法 `task-server/task clean <号>`；没人取的照服务的推送办，不列 | 第 1 问「设了留现场变量的现场」 | 推的，待实现 |
| 只改说法：`.claude/skills/dispatch/SKILL.md:18` 写明完工清理闸不判任务现场，设了留现场变量的读完日志自己 `task clean <号>` | 第 1 问技能那一句；对「漏拦」本身不起作用 | 推的 |
| 三处「树删了（副本删了）」改成「树（`--cwd` 往上第一层有 `.git` 的那一层，仓副本就是这一层；哪一层都没有的是 `--cwd` 本身）删了」，出路补「`--cwd` 往上没有 `.git` 的，删那一层」 | 第 1 问次要那一条、第 3 问 | 推的 |
| `.claude/agents/three-way-attack.md` 第 35 行补一句：`rerun.sh` 里取 `${CARGO_TARGET_DIR:-<拷贝>/target}`（`rerun.sh` 整条以 `bash` 交，服务不给），或在第 9 步的包装脚本文件头写会编的声明 | 第 2 问打中一 | 推的；探针待主 agent 跑 |
| 第 35 行的 `<debug 或 release>` 改成「`--release`、`--profile release`、`--profile bench` 是 `release`，别的是 `debug`；`--example` 另加 `examples/`」 | 第 2 问打中二 | 推的 |
| 词表里的角色名在自证格名（`「」` 里）不算历史，或把这类行登进常设豁免；去掉检出钩子那四处（连同第 582、726、1133 行）的标记。或者反过来，共用约束「特殊缘由」加一类「自证格名与用例标签原文」 | 第 4 问 | 推的；改门禁或共用约束都不归这条腿 |

## 要主 agent 跑的命令

在仓根 `/home/fy5090/code/singlefs` 起（`--cwd` 不写即取此刻目录；探针那两条的 `--cwd` 是草稿目录）。喂钩子的六条各看 `TASK_SERVER_RESULT` 的 `exit=`：0 是放行（推的预期），2 是拒，拒的那一条 `task-server/task log <号>` 里 `✗` 行就是打中；范围声明闸那三条另看日志里有没有 `permissionDecision`，有就是弹窗（推的预期是没有）。

- `task-server/task submit -- env AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/detections.jsonl bash -c 'bash .claude/hooks/project-code-guard.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-experiment-runner.json'`
- `task-server/task submit -- env AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/detections.jsonl bash -c 'bash .claude/hooks/project-code-guard.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-three-way-attack.json'`
- `task-server/task submit -- env AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/detections.jsonl bash -c 'bash .claude/hooks/project-code-guard.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-prune-server-hint.json'`
- `task-server/task submit -- env AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/detections.jsonl bash -c 'bash .claude/hooks/run-scope-guard.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-experiment-runner.json'`
- `task-server/task submit -- env AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/detections.jsonl bash -c 'bash .claude/hooks/run-scope-guard.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-three-way-attack.json'`
- `task-server/task submit -- env AGENT_HOOK_DETECTIONS=/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/detections.jsonl bash -c 'bash .claude/hooks/run-scope-guard.sh < /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/feed-prune-server-hint.json'`

第 2 问打中一的前提探针（两条对照，`--cwd` 是草稿目录 `/home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus`；`cargo build --help` 只打帮助、不编）。看日志里 `name=rerun_shape_probe` 那一行：推的预期是脚本正文那一条打 `cargo_target_dir=unset`、`bash -c` 那一条打一个仓外的路径；脚本那一条打出路径，打中一的前提就倒了。

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus -- bash /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/probe-rerun-shape.sh`
- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus -- bash -c 'cargo build --help > /dev/null && echo "name=rerun_shape_probe shape=bash-c cargo_target_dir=${CARGO_TARGET_DIR-unset}"'`

探针脚本的 `sha256sum`：`1f09b6be65234e407b0e192c4d9361e524b9d68cd507e7cc8acb3141dccaec68  /home/fy5090/.local/share/singlefs/work/build-dir-clean-wording-governance-r1-opus/probe-rerun-shape.sh`。这两条会让服务给草稿目录这棵树开一组编译目录（第 3 问那一种）；`cargo build --help` 不往里写东西，组是空的，草稿目录删了服务跟着删。

## 顺带看到的（不在四问里，交主 agent）

- `.claude/gate.d/break-switch-replay-files.tsv:10` 写「服务编译目录那一支连同它的五条开关去掉的那一版，换上之后还没复跑」，正文「实现今天的样子」说开关复跑 76 条逐格相同；两处对不上，归正推腿第 6 问。
- 任务现场长期残留（第 1 问第 5 条）与删支无关的那一部分，在服务一侧：设了留现场变量的现场没有到期删。

## 没打中的形状

- 第 1 问：合入后验证快照、认领目录里的副本与编译目录、子 agent 的「交主 agent 删」一行这几处的判法逐行比过，删支没碰；分片父任务的测试二进制在父任务有结局时由服务删（`task-server/task_server_scheduler_building.py` 的 `shard_binaries_removal`），不靠 `task clean`；服务编译目录不在任何认领目录里，也不在数据根的迁库遗留扫描里被认成库。
- 第 2 问：第二台的转义链（`printf '%q '` 逐词）、`--peer` 拒 `CARGO_TARGET_DIR=` 的正则、交回闸 ⑧ 的引号内 `&&`、服务提交时警告的 `$` 段，四处各按代码读过，没找到拒它的地方。
- 第 3 问：`prune-server/control` 的编法提示、`lib_workspace_cargo_lock.py` 的照抄出路，各照它们的字面串推了一遍会碰到的闸（项目代码闸、范围声明闸、共享根外路径退 3），没找到拒。
- 第 4 问：搬迁表删行之后的旧名引用按六处目录搜过；滞后表四行按 HEAD 与此刻两版各数过。

## 这条腿自己的限度

- 没写模型，没起任何东西；「闸放行」「服务给不给 `CARGO_TARGET_DIR`」都是按代码读的推断，等上面八条结局。
- 第 4 问滞后表那一句用整份 `grep -P` 代替门禁的注释切分，分不出注释与字符串。
- cargo 的 profile 目录规则凭印象，没现查。
- 第 1 问「本该被拦」取决于「有意留的现场要不要收工时提醒」这一设计判断，我只给了两边的依据，没判。
- 没读：禁读清单里的另一条腿报告；`records/` 里这一批的经过只读了搬迁表那两行。

## 没做什么

- 不判正推那三格（第 5 到 7 问）；不替主 agent 采纳；没往统一任务服务交任务，上面的命令都没跑。
- 没建仓副本、没建编译目录：草稿目录里只有 diff 文本、三份喂钩子的 JSON、一份探针脚本与写者锁，没有要删的副本或编译目录（`du -sh` 见交回）。
