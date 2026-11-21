# 编译目录归树之后的旧说法收口：治理文件轮第三轮（最后一轮），攻第二轮判决的修补（build-dir-clean-wording-governance-r3，2026-11-21）

这一轮攻的是第二轮判决（`research/prompts/build-dir-clean-wording-governance-r2-main-verification.md`）「三、改法」已换上主工作区的修补；这是这一题的最后一轮，这一轮新冒出来的形态不再开第四轮，标零轮。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；与 `crates/` 有交点的只有两份 agent 定义与复跑模板里编 `crates/` 下装置时二进制取哪个目录的写法。前两轮攻过而没打中的角度不重复：两份定义 `bash -c` 那一形与 `mkdir -p` 交给服务跑不跑得起、过不过得了项目代码闸与范围声明闸（主 agent 两轮共喂九条都放行）、输出目录对照（主 agent 探针七种形态逐项坐实）、滞后表与搬迁表删行、派发闸那一处 `history-kept` 的类别、改法 4 选修判法而不选登常设豁免。

## 问题

1. 每一处修补做的是不是第二轮判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 前两轮正文或判决里只抽查过、没看过的地方（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 547–550 行）：

- 改法 1：完工清理闸取尝试改成 `attempts JOIN tasks ON attempts.task_id = tasks.id WHERE tasks.session = ?`；`scene_due` 两个不到期条件都判：本会话已有结局、结局没取走的出路 `task-server/task wait <号>`，设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 的出路 `task-server/task clean <号>`，两样都占的写 clean；自证样本库尝试行会话列写空串；`unkept-scene-not-listed` 样本改成结局已取走；新格 `uncollected-scene-left`；四条新开关（`kept-scenes-uncollected-unchecked`、`uncollected-scene-removal-by-clean`、`kept-scene-removal-uncollected-first`、`kept-scene-attempts-by-session-column`）。换上后攻方探针「name=kept_scene_store_shape arm=store-shape attempt_session_column=empty hook_exit=0 blocked=true scene_listed=true clean_named=true」；自证「判了 116 格；86 个弄坏开关各红在点名的格」，开关复跑 86 条逐格相同。
- 改法 2：`.claude/agents/three-way-attack.md` 第 9 步与复跑模板用法里外层脚本与模型目录写成 `<仓根>/research/prompts/<轮>-opus-model/…`，写明相对路径照 `--cwd` 解；对照补 `-r` 与 `--profile=<名>`；`.claude/agents/experiment-runner.md` 同改并补「`--cwd` 不是仓根时，命令位置上的脚本写绝对路径」。rules-lint 退 0，模板开关复跑 3 条逐格相同。
- 改法 3：注释历史判法 `scripts/gate/comment-history-check.py` 的 `judged_text` 改成那一层「」里有 ASCII 冒号（`CELL_NAME_SEPARATOR`）才放过角色名；新格 `quoted-role-name-without-colon`、新开关 `quoted-role-names-colon-unchecked`；判法开关复跑 32 条逐格相同；门禁 doc-text comment-history 格单跑只剩 `scripts/gate/large-file-git-hook.py` 一行滞后行（mainagent(A) 写的脚本，已转）。
- 改法 4：「取消都不删它」照 `task-server/task_server_lib.py` 文件头第 276 行（第二台中途失败、被停的那一趟「⑥ 连它一起删了」）写全：`task_server_lib.py` 文件头与 `remove_cancelled_artifacts` 文档串、`task-server/task` cancel 说明、`task-server/attempt_runner.py:27`、`task-server/task_server.py:66` 等、`gpu-verification-server/control:392`、`prune-server/control:163`、`task-server/README` 三语第 163 行、`prune-server/README.md:27`、kb `tooling` 三语第 42 行；几处「树」补「解开符号链接后」。
- 改法 5：`.claude/hooks/lib_workspace_cargo_lock.py:225` 去掉「编译目录用完就删」；自证与 `refusal-submit-dropped` 证红只红点名那一格。
- 改法 6：任务服务加 `sweep_unpooled_attempt_label_directories`（`task-server/task_server_scheduler_building.py`），由 `sweep_vanished_tree_pools` 扫完组以后接着调：只删名字是本库尝试标签、不在任何组里、不是在跑尝试用着、没有进程开着的目录，删了回读不在；三条新开关；`task-server/README` 三语第 224 行补了这一支。开关复跑四条逐格相同；重启服务后起来那一扫「删了 1 个、没删成 0 个」，旧命名目录由 12 个剩 11 个，剩下的里 2 个近 6 小时还有写入、是仍登在组里接着用的编译目录（其余 9 个属哪棵树、那棵树在不在，主 agent 没读服务库，推的）。第二轮判决第 7 条照辩方「81G 没有一条路删」说过头，这一轮判决改正。
- 换上中断一次：任务服务那份规格第一次尝试撞内存上限被杀、服务重排、第二次换上工具拒绝；主 agent 直接跑验证「task_server_lib.py 自证 4 格都对」再 `--seal-outcome` 补写完成。根子（服务只认命令那一层的写工作区工具、包在脚本与 `bash -c` 里的认不出）另派了工具员，不在这一轮的射程里。
- 改前原样：各草稿目录的 `base/`（`~/.local/share/singlefs/work/build-dir-governance-r2-fix-c/base/`、`build-dir-governance-r2-fix-d/base/`），第二轮开工快照 `research/prompts/build-dir-clean-wording-governance-r2-snapshot/judged-sha256.txt`；不经草稿的对着 `git diff HEAD -- <路径>` 看。
- 已知开着、各有会话在修、这一轮不重复报的：`.claude/agent-common.md:37` 同一旧说法由 mainagent(B) 改；kb `tooling.md` 第 42 行后半句的领卡写法登在 `records/2026-11-19-未做完的任务登记.md` 第 35 行。

## 范围

这一轮被判的文件（33 份，逐个列在 `research/prompts/build-dir-clean-wording-governance-r3-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 完工清理闸接任务表那一判：任务表的会话认得对不对（分片子任务、人手 `--manual`、换过会话号的）；结局没取走那一支的出路 `task wait <号>` 照做之后服务删不删那个现场、闸下一回还拦不拦；两样都占写 clean 对不对；四条新开关是不是红在名字说的那件事。
2. 注释历史判法「带冒号才放过」：有没有一种写法带了冒号也能把来历句藏进「」放过（例「「第二轮:攻方腿」打中的」）；冒号判的是 ASCII 冒号，全角冒号「：」在格名里出现时判得对不对。
3. 服务清扫旧命名目录：哪一种目录不该删而会被删（别的库标签的、第二台的、在排还没起的尝试要用的、组里登了而树刚换过路径的）；回读、告警、扫的频率与 `sweep_vanished_tree_pools` 的次序有没有缺口。
4. 改法 2、4、5 改过的说明与出路：照字面做走不走得通，与 `task-server/task_server_lib.py` 文件头「编译目录」「现场」两段逐句对不对得上；有没有一处改出了新的假话。

正推腿（逐条回「成立」「不成立」「推不出」）：

5. 「实现今天的样子」每一条：今天的文件是不是这样写的（写出 文件:行 与那一句原文）。
6. 前两轮正文与判决里写了「只抽查」「推的」「没读」的地方，逐条现查：第二轮判决第 7 条剩下 9 个旧命名目录属哪棵树（只读能查到的范围内）、第一轮判决里「两道闸都放行」以外没喂过的闸（派发闸、检出钩子）对两份定义新写的命令判不判。
7. 全仓还剩的「编译目录用完 task clean 删」「wait 末行 build=」「取消都不删」「副本删了」旧说法（`research/prompts/`、`records/`、变更史与已知开着那几处除外），给出 文件:行。

## 分工

攻方腿做第 1 到 4 问，正推腿做第 5 到 7 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
