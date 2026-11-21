# 旧账本开着项当场修的治理文件轮第一轮：攻方腿报告（ledger-open-fixes-governance-r1，2026-11-19）

攻击面：正文「要腿判的」第 1 到 5 问，角度是过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。读的是主工作区此刻的那一版；只有这条腿读到的范围见末节「这条腿自己的限度」。

探针与喂法都在草稿目录 `/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/probe/`，一份都还没跑（子 agent 不交任务），下文凡写「探针会打」的都是推的，待主 agent 跑：

| 文件 | sha256 | 管哪一问 |
|---|---|---|
| `hook_swap_lock_probe.py` | `864ec87511820bb8db1d4e22b9ec419224dcbed3856a8636c542f3465ab12505` | 第 1 问，六臂 |
| `added-lines-error-feed.sh` | `39331eb6a15c435af5f4cb3db37e35ced8e0122e910f3f616cf32c23d1cc08ad` | 第 2 问（标准错误） |
| `evidence-mixed-replay-feed.sh` | `c0e1f3757ddad2625f9f26256512f5154f867fffbae17b5e305e5bd22bb2a2a0` | 第 2 问（复跑表 exact 行） |
| `deliverables_refusal_probe.py` | `4569868a7d93d9ea5c1754754a3d1c9234b1e7d784b8f83ccdad5a2cbc010bb1` | 第 3 问，四臂 |

没建模型目录、没写 `rerun.sh`：这一轮的探针是喂被判脚本的小树，派发提示叫放草稿目录。

## 各格判定一览

| 问 | 格 | 判 | 依据一句 |
|---|---|---|---|
| 1 | 每条进口在建树、写文件之前占住 | 不成立（打中） | `--seal-outcome` 往结局文件追加回读与完成，不占、也不看开始行那一趟还在不在；探针臂 `seal-outcome-while-swap-running` |
| 1 | 每条出口放掉 | 成立 | `finally` 与 TERM、HUP 处理；KILL 留下的记录靠同一台「进程没了就不挡」 |
| 1 | 同一台四样在进程号复用、容器、重启之后 | 成立（今天的部署）；条件句见正文 | 启动刻挡复用、开机号挡重启；任务服务经 `systemd-run --user` 起、不分进程命名空间；两台 machine-id 量得不同 |
| 1 | 另一台看修改时间那一支 | 成立（推的，条件：两台钟差小于约 285 秒） | 300 秒对 15 秒心跳；NFS 上 open 会重取属性 |
| 1 | 缝一（几趟同时摘旧记录） | 打中：给出复现；且它的后果句不成立 | 三趟才够得着；探针臂 `gap-three-contenders-on-stale-record`；「结局文件拦着」那半句被下一格推翻 |
| 1 | 缝二（link 回复丢失） | 后果够不着互斥；够不够得着推不出 | 后果只是一次点名自己的假退 10；探针臂 `gap-link-reply-lost` |
| 1 | 新格：结局文件按仓、占用锁按草稿目录 | 打中 | 拒绝说明叫并行的「各给一个 --scratch」，两趟真换在读结局文件与写开始行之间重叠时后写的盖掉先写的；探针臂 `outcome-overlap-different-scratch` |
| 2 | 报错写到标准错误之后格的结局与成功、拒绝句 | 成立（推的） | 只在取不到时红句前多几行；取到时 git 的标准错误被吞，原来照打，不改判 |
| 2 | `load_change_range` 与原五个 `load_*` 三种情形逐个相同 | 值与状态成立；标准错误不成立（打中，小） | 取到时标准错误被吞；`gate_added_lines` 失败不写「哪一条 git、退几」，与文件头那句不符；喂法 `added-lines-error-feed.sh` |
| 2 | 复跑表 exact 行一路 | 打中 | 一个实验只要有一条 exact 行，它的 timing 行改了也放过，与文件头「timing 行照旧比修改时刻」相反；喂法 `evidence-mixed-replay-feed.sh` |
| 3 | 多拒 | 打中 | 上层认领目录里别人写坏的一行拦下层子 agent 的交回；带空格的路径照文件头写法也过不了 |
| 3 | 漏拒 | 打中（轻） | 符号链接的 `.deliverables` 整份跳过、不拒；写在副本那一层的不读 |
| 3 | 挪场景之后 `s` 别处有没有被换掉 | 成立（推的） | 插进去的一段自成一格，后面每一格用之前都重新赋值 |
| 4 | 该不该守入口脚本那一节 | 成立 | 三份都必须带位置参数，规则第 4 行把它们排除在入口脚本之外 |
| 4 | 退 78 的准入 | 不成立（打中，小） | 虚机驱动要的可读内核镜像不在 run-condition 里，缺了退 1 当作废 |
| 4 | 派到第二台、产物落在哪 | 推不出 | 第二台的内核镜像、KVM 没核；删旧闸不分 `research/results/` 下未进 git 的产物 |
| 4 | 三份只差实验号、没漏改 | 成立 | 归一化之后只差设计内的几处；两处说明与实现不一 |
| 5 | 钉值与现算 sha256 是同一个量 | 成立（量的） | `sha256sum` 现算等于钉值；装置比的是编进去的源码原文的 sha256 |
| 5 | 钉值等于登记第十二节 | 不成立（打中） | 文件头说钉值就是登记第十二节那一个；r4 登记第十二节写 `9e813cbd…`，r3 登记写 `4368c0e7…`，钉值 `d312b9aa…`；重钉把停机条款 K-S6、J-S7 变成照跑 |
| 5 | 装置再改一次照样只在开跑时红 | 成立 | 三份自证把钉值换成自证给的那个，自证绿不证钉值 |

## 第 1 问：换上工具的草稿目录占用（`scripts/gate/hook-swap.py`）

读的是主工作区此刻那一版对暂存区的差（占用这一段只在工作区里，暂存区与 HEAD 里都没有 `swap.lock`：`grep -c 'swap.lock'` 对 HEAD、暂存区两份都是 0，量的）。

### 进口与出口

- `--dry-run`、真换、退回规格（反向规格当普通规格交）、只给脚本规格的四种都经 `swap()`：「held, busy_lines = hold_scratch(scratch, mode)   # 先占草稿目录」（`scripts/gate/hook-swap.py:2809`），读结局文件「unfinished = pending_refusal(root, outcome_file, mode)」（`scripts/gate/hook-swap.py:2821`）在它之后。成立。
- `--feed-only`、`--check-callers` 不收 `--scratch`，写的是 `tempfile` 的临时目录，不碰草稿目录，不占是对的。成立。
- `--seal-outcome`：文件头写明「草稿目录的占用（防同一个 --scratch 上并发；--read-outcome、--seal-outcome、--feed-only、--check-callers 不占）」（`scripts/gate/hook-swap.py:70`），而它要写文件：「    append_outcome(path, [("回读", state[0], state[3]) for state in states] + [("完成", len(states), int(time.time()), "补记")])」（`scripts/gate/hook-swap.py:2579`）。它也不看开始行记的那一趟还在不在（`pending_refusal` 看、退 9；`seal_outcome` 只看几份文件是不是都已换上）。可达的历史：一趟真换全部换上、正在换上之后再验（钩子规格要喂几十条，几秒到几十秒），这时另一条 `--seal-outcome` 补写了完成；随后那一趟被杀（任务服务撤销是 TERM，TERM 处理只放锁、不回滚）。结局文件有完成，下一趟不拦，`--read-outcome` 退 0 说「换完了」，而换上之后再验一次没过。探针臂 `seal-outcome-while-swap-running` 预期打 `pending_while_running=9 seal_exit=0 next_swap_pending_refusal=none read_outcome_exit=0`（推的，待主 agent 跑）。这一处不是这一次带进来的，但第 1 问把它列成要判的进口。
- 出口：正常返回、`BadInput`、Ctrl-C 走 `finally`；TERM、HUP 走处理函数；KILL、QUIT 留下记录，同一台靠「进程没了就不挡」。成立。

### 同一台四样（机器、开机、进程号、启动刻）

- 进程号复用：启动刻对不上判「不在」；重启：开机号变了判「不在」：「if outcome.boot != identity_file_value(BOOT_ID_PATH) or found is None:」（`scripts/gate/hook-swap.py:2373`）。成立。
- 容器：任务服务的尝试单元经 `systemd-run --user` 起（`task-server/attempt_runner.py` 文件头第 2 行，转述），`task-server/task_server.py` 里 `grep -nE 'Private[A-Za-z]+=|ProtectProc'` 零命中（量的），同一个进程命名空间，今天够不着。
- 会判错的条件：两台 machine-id 相同（克隆镜像）。那时另一台活着的持有者被读成「机器相同、开机不同」，判「不在」、立刻抢。探针臂 `same-machine-id-other-boot` 预期打 `live_holder_counts_as_occupying=False`（推的）。今天的两台：`.git/singlefs-hook-swap-outcome.tsv` 开始行的机器段是 16 位十六进制、不等于本机现算的那一个（量的，一次），说明上一趟在第二台跑、两台 machine-id 不同。今天够不着。

### 另一台看修改时间那一支

年龄是读的那一台的 `time.time()` 减记录的修改时间；记录在本地盘那一台用本地钟写、在 NFS 那一台由服务端定时间。两趟同时进要读的那一台把一份刚刷新的记录算成过期，得两台钟差超过「SCRATCH_LOCK_STALE_SECONDS = 300」（`scripts/gate/hook-swap.py:2586`）减一个心跳，约 285 秒。属性缓存：读记录先 `open`，NFS 的打开即重取属性，接着的 `stat` 拿的是新的。判「成立」，条件是两台钟差小于约 285 秒；第二台的钟没核（推的）。钟差落在反方向只会让死掉的持有者多挡一会儿，不破互斥。

### 工具员写明的两条缝

- 缝一：「留下的缝：几趟同时在摘同一份已经不挡的旧记录时，极窄的时机下两趟可能同时放成」（`scripts/gate/hook-swap.py:86`）。两趟够不着，要三趟：甲、乙都读到同一份过期的旧记录；甲摘走、放成自己的；乙随后改名摘走的其实是甲的记录，核出内容不同，要「os.link(removed, path)」（`scripts/gate/hook-swap.py:2698`）放回去；就在乙摘走与放回之间，丙读到没有记录、放成自己的；乙放回撞 EEXIST，接着删掉摘走的那份——甲的记录没了，甲、丙同时占着。探针臂 `gap-three-contenders-on-stale-record` 用线程与替进去的 `os.rename` 定死这个次序，预期打 `holding=A,C a_record_still_on_disk=False`（推的）。原始故障就是同一个草稿目录上并发三份 `--dry-run`，三趟正是这道锁要防的形状；上一趟被 KILL 留下的旧记录是旧记录的常见来路。
- 缝一的后果句「主工作区那一头仍有基线核对与结局文件拦着」（`scripts/gate/hook-swap.py:87`）推不出，见下面「结局文件按仓、占用锁按草稿目录」。
- 缝二：NFS 上 `link` 在服务端做成而回复丢了、重发报 EEXIST。走到 `hold_scratch` 的下一轮，读到的是自己的记录、持有者是自己这个活着的进程，判「占着」，退 10，拒绝说明点名的是自己的进程号；记录留下（没登记成持有，不放）。之后同一台按「进程没了」让路，另一台最多挡 300 秒。互斥不破，后果是一次误导人的退 10。够不够得着取决于挂载的 NFS 版本：4.1 起的会话有应答缓存，重发拿回原应答、不报 EEXIST；本机读不到 `/proc/fs/nfsd/versions`（`Permission denied`，量的），第二台没核，推不出。探针臂 `gap-link-reply-lost` 让 `link` 先做成再抛 EEXIST，预期打 `held=False refused=True refusal_names_own_pid=True record_left=True`（推的）。

### 结局文件按仓、占用锁按草稿目录（这一轮没被列出的一格，打中）

- 结局文件不在草稿目录里：「结局文件的位置：root 是 git 工作树的根就放它的 git-dir 下（下一趟不管用哪个草稿目录都看得见）」（`scripts/gate/hook-swap.py:2392`）。占用锁只锁同一个草稿目录。
- 新写的拒绝说明与文件头把并行的出路指到不同的草稿目录：「→ 怎么办：等那一趟结束再交，或换一个 --scratch（要并行的几条各给一个草稿目录）」（`scripts/gate/hook-swap.py:2666`）、「几条要并行就各给一个 --scratch。」（`scripts/gate/hook-swap.py:44`）。
- 结局文件挡「上一趟还在换」只在读的那一刻：读在 `scripts/gate/hook-swap.py:2821`，写开始行「write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)」（`scripts/gate/hook-swap.py:2933`）在草稿建树、喂完全部喂法、核调用方名字之后，中间是几秒到几十秒的窗口。两趟真换（各给一个草稿目录，或缝一里同一个草稿目录的两趟）都在窗口里读过结局文件，后写开始行的那一趟整份改名盖掉先写的。先写的那一趟换到一半被杀，结局文件里只有后一趟的开始行与完成行，下一趟不拦，几份文件停在半截没人提。
- 探针臂 `outcome-overlap-different-scratch`：甲（两份文件的脚本规格）在草稿验完之后让乙（另一个草稿目录、另一份文件）读结局文件，甲写开始行、换上第一份，再放乙写开始行、换完、写完成，然后甲被杀。预期打 `a_killed=True b_exit=0 x1_landed=True x2_landed=False next_swap_pending_refusal=none read_outcome_exit=0`；对照臂 `outcome-overlap-control`（没有乙）预期 `next_swap_pending_refusal=9`（推的）。
- 四句：不分臂（治理文件轮没有臂，病在共用前提「一个仓同一时刻只有一趟真换」）；系统做决定那一刻看不到判别它的东西（读结局文件时另一趟还没写开始行）；满足的是文件头第 87 行那半句的反面；工具员给的唯一改法（占用锁）在这一格上照中，锁的键是草稿目录。
- 改法（只在我的推导里、没实现没跑，被攻过零轮）：真换那一段（读结局文件到写完成或退回）另占一份按仓的锁，放在结局文件同一目录（git-dir 下）、用同一套 `link` 记录；`--seal-outcome` 也占它，并像 `pending_refusal` 那样看开始行那一趟还在不在、在就退 9。各修哪一格：按仓的锁修「各给一个草稿目录也会重叠」与缝一的后果；`--seal-outcome` 看进程修「补写盖住在跑的那一趟」。两格都「推的，待主 agent 跑」。

## 第 2 问：`changed-paths.sh` 与三道文档门禁

### 报错写到标准错误之后，结局与成功、拒绝句

`gate_changed_paths` 的六处调用方（`grep -rn gate_changed_paths` 在 `.claude/gate.d/` 下非样本的六份：doc-process-records、doc-experiments、doc-decisions、doc-text、doc-registries、checker-independence-and-sync，量的）没有一处把标准错误并进标准输出取值，取到的路径表不变。变的只有标准错误：取不到时多出「取不到改动路径：git … 退出 …」与缩进的原文；取到时 git 写到标准错误的东西（例：重命名探测超上限的 warning）进了临时文件就删，原来照打。门禁的退出码与成功句、拒绝句都不随它变。判「成立」（推的，读代码）。附带一处新的退 1：`error_file="$(mktemp)" || return 1`，`$TMPDIR` 写不进时原来照取、现在退 1。

### `load_change_range` 与原来五个 `load_*`

HEAD 那一版五个函数（`git show HEAD:.claude/gate.d/doc-process-records.sh` 第 836–866 行）与现在第 843–870 行逐个对：

| 情形 | 值 | 状态 | 标准错误 |
|---|---|---|---|
| 取不到 | 同（半截输出照存） | 同（1） | 原来：命令自己的标准错误在第一次取的那一刻打一次；现在：存进 `<前缀>_error`，每调一次打一次，父进程预取那一次也打 |
| 取到空 | 同（空） | 同（0） | 原来照打；现在吞掉 |
| 取到多行 | 同 | 同 | 原来照打；现在吞掉 |

值与状态成立；标准错误一列不成立。另一处说法与实现不一：「取法命令是 gate_changed_paths 或 gate_added_lines：它们失败时把哪一条 git、退几、git 的报错原文写到标准错误。」（`.claude/gate.d/doc-process-records.sh:841`）——这一次只改了 `gate_changed_paths`；`gate_added_lines` 的 git 照旧直接起（「git "${quote_option[@]}" diff --no-renames --unified=0」，`scripts/gate/changed-paths.sh:228`），失败时只有 git 自己的原文、没有「哪一条、退几」，awk 解析失败（路径被 git 加引号）时什么都不写，格里打的是「取改动范围的命令没有留下报错」。用到它的两格（knowledge-sync、deferred-items-registered）没有 git 报错样本。喂法 `added-lines-error-feed.sh` 在 `diff.renames` 写坏的仓里各起一次两个函数，预期打 `gate_changed_paths exit=1 header_line=1` 与 `gate_added_lines exit=1 header_line=0`（推的）。

### evidence-in-repo 的「复跑表 exact 行」一路（打中）

- 文件头：「timing 行、复跑表里没有这个实验的行、点名的产物不在盘的，照旧比修改时刻。」（`.claude/gate.d/doc-experiments.sh:821`）
- 实现按实验号放：「if replay_row and replay_row.group(5) == "exact" and replay_row.group(4) in product_path_by_file_name:」（`.claude/gate.d/doc-experiments.sh:970`）把实验号记进 `replay_exact_product_by_number`，判的时候只问「if (number not in on_request_numbers and not fingerprint_stale and newest is not None and number in replay_exact_product_by_number」（`.claude/gate.d/doc-experiments.sh:1017`）。一个实验只要有一条 exact 行、点名的产物在盘，它的装置怎么改都放过，哪怕改的是只出 timing 产物的那一段；timing 行不进门禁的逐字节复跑（`research-build-and-replay.sh` 文件头 experiment-replay 一格写的是只派 exact 行，转述）。改了 timing 那一段：原来要一份比装置新的产物，现在既不要产物、也没有复跑。
- 今天复跑表里同一个实验既有 exact 又有 timing 行的：E162（2 条 exact、3 条 timing）、E174（4 条、2 条）、E184（1 条、1 条）（对 `scripts/experiment-apparatus/replay.sh` 按实验号分组数的，量的）。E162、E174 的装置是 `scripts/experiment-apparatus/e162_*.rs`、`e174_*.rs`，落在 evidence-in-repo 认的装置源码形里；三者 exact 行点名的产物今天都不在 `research/results/`（`find` 零命中，量的）——今天够不着，等哪一轮把 E162 的 anchors 重跑入库，之后改 `e162_crash_verdict_block_store.rs` 的 S1 计时段就放过。
- 另两处更宽：变异表（`scripts/mutation/tables/e<号>_*.tsv`）改了同样按实验号放过；产物按文件名（不带目录）对，`research/results/` 底下任何一层有同名文件都算在盘。
- 喂法 `evidence-mixed-replay-feed.sh`：拿样本 `evidence-in-repo-red` 原样起，只在提交基准之前给 E913（它在样本里只有一条 timing 行）加一条 exact 行与一份比装置旧的产物。照文件头 E913 应仍在红行里；照实现 E913 进「按复跑表判」、红行里少了 E913 那一行。预期打 `按复跑表判 1 份…E913` 且没有 `e913_replay_timing.rs（装置源码，改于 2026-09-05）` 那一行（推的）。
- 改法（零轮，推的）：按行而不按实验号放——装置这一次改动碰到的产物都落在 exact 行上才放；做不到按行的，同一个实验有 timing 行就不走这一路。

## 第 3 问：完工清理闸「交回时 `.deliverables` 写法不合当场拒」

新拒的范围：「交回时（PreToolUse）认领目录、草稿目录与合入后验证那一层的 .deliverables 里有写法不合的行（读不了、不是三段、种类不在五种里、理由太短、路径落在目录外）」（`.claude/hooks/completion-cleanup-guard.sh:108`），实现是「if problems and event == "PreToolUse" and not broken("handback-declaration-unchecked"):」（`.claude/hooks/completion-cleanup-guard.sh:1905`），`problems` 来自「exempt, problems = declarations([*(directory for root in roots for directory in (root[0], root[1])), *layers], role)」（`.claude/hooks/completion-cleanup-guard.sh:1904`）。

多拒（探针 `deliverables_refusal_probe.py`，四臂预期都是推的）：

1. 上层认领目录里别人写坏的一行，拦下层子 agent 的交回。子 agent 的草稿目录是派发提示点名的下一层时，`roots` 那一项是（下一层，顶层），「        roots.append((path, top, set()))」（`.claude/hooks/completion-cleanup-guard.sh:718`），声明读两层。主 agent 或同一会话的别的子 agent 在顶层 `.deliverables` 写了一行种类不对的（例 `snapshot 留底 …`），下一层的子 agent 一个副本都没留也交不回，而那份文件不在它的写范围里，改不了，拒绝说明只叫它「改对再交回」。臂 `parent-directory-bad-line-blocks-child-handback` 预期 `action=block named=True`。共用约束「写」一节说子目录另派给别的子 agent，正是这种形状。
2. 路径带空格的报告。头里写法「<相对那个目录的路径>  <种类>  <为什么留>      种类五选一：报告、补丁、备份、交接、基线」（`.claude/hooks/completion-cleanup-guard.sh:99`），实现「            parts = text.split(None, 2)」（`.claude/hooks/completion-cleanup-guard.sh:747`），之后才剥引号。`"final report.md" 报告 …` 被切成四段、第二段不是种类，判写法不合；没有改对的写法，只能改文件名。臂 `quoted-path-with-space-blocks` 预期 `action=block named=True`。

漏拒：

3. `.deliverables` 是符号链接（指过去的文件里种类写错）：「        if os.path.islink(path) or not os.path.isfile(path):」（`.claude/hooks/completion-cleanup-guard.sh:736`）整份跳过，不算「读不了」，不拒、不点名，里面的声明也不算。臂 `symlinked-declaration-with-bad-line` 预期 `action=pass named=False`。轻：声明不算，留着的副本照样在单子上被拦。
4. 写在副本所在那一层（不是认领目录那一层）的 `.deliverables`：只读 `roots` 的两层，不读、不拒。臂 `nested-declaration-with-bad-line` 预期 `action=pass`。同样轻。
5. 子 agent 写「基线」：种类在五种里，不算写法不合，交回不拒；头里说基线只认主 agent。按头里列的五种不合，这不算漏拒，记一笔。

挪场景之后 `s`：插进去的一段（`s = scenario("bad-declaration")` 起的那一段）在 `s = scenario("hand")` 之前收完，自己赋的 `s`、`directory`、`declaration_path`、`outcome` 在下一格开头都重新赋值；往后各格（hand、hand-library、hand-main…）用到的名字在本格里先赋再用。判「成立」（推的，读了第 2540–2660 行）。

## 第 4 问：E07、E53、E159 三份入口

### 该不该守、守没守

- 入口脚本那一节（`--list-items`、`--item`、不带参数跑全部）：不该守。规则的射程句把它们排除在外：「必须带位置参数、对一张表或一份输入跑的工具与只读日志的辅助脚本不算」（`.claude/rules/test-script-discipline.md:4`）；三份都必须给 `<输出文件>`。成立。
- 退 78 的准入（不成立，小）：规则要「整个脚本都离不开的工具、设备、硬件写成 `run-condition`」（`.claude/rules/test-script-discipline.md:30`）。三份的「# run-condition: command cargo rustc qemu-system-x86_64 busybox cpio」（`scripts/experiment-apparatus/e07-vm-bench-run.sh:3`）加 musl 与 `/dev/kvm` 两条，漏了虚机驱动自己离不开的可读内核镜像：「KERNEL="$(find_kernel)" || die "找不到可读的内核镜像。」（`scripts/devices/vm-bench.sh:301`），`die` 退 1（`scripts/devices/vm-bench.sh:52`），入口照「if ((driver_exit != 0)) && ! break_is driver-exit-code-ignored; then」（`scripts/experiment-apparatus/e07-vm-bench-run.sh:372`）报「虚机驱动退出码 1，这一趟作废」退 1，不是 78。本机 `/boot/vmlinuz-*` 三份里两份是 600（`ls -la`，量的），驱动按通配次序取第一份可读的，本机够得着；第二台没核。可达的输入：派到 `/boot/vmlinuz-*` 全是 600、又没设 `SINGLEFS_KERNEL` 的那一台，退 1 而不是 78。
- 派到第二台跑不跑得起：编译目录由服务给、二进制取 cargo 报的、输出路径按调用方工作目录（共享的仓）算，这几样两台都行；KVM、musl、QEMU、busybox、cpio 与内核镜像第二台有没有，推不出。
- 产物落在哪：调用方给。删旧闸「rm -f -- "$output_file"   # 删旧闸：跑前删本轮输出」（`scripts/experiment-apparatus/e07-vm-bench-run.sh:347`）在构建之前，不分输出是不是 `research/results/` 下已存在、没进 git 的产物；同一天按同一个带日期的名字重跑，旧产物先被删，这一趟作废也回不来。检出钩子 ⑤ 只判 Bash 命令里的整份覆盖，看不见脚本里的 `rm`。推的。

### 三份是不是只差实验号

把实验号、二进制名、实验名换成同一个记号之后两两 `diff`（草稿目录 `d07-53.txt` 44 行、`d07-159.txt` 56 行，量的），差的只有：用法与例子、参数说明、`VM_*` 的例值、E159 多的 `-p e7-index-bench`、E159 的用法行。没漏改。两处说明与实现不一（轻）：

- E53：「e53_ring_failure 不收参数：盘的路径（四块盘 /dev/vda … /dev/vdd）由虚机里 scripts/devices/vm-bench.sh 放在参数前面传给它，复跑不给参数」（`scripts/experiment-apparatus/e53-vm-bench-run.sh:12`），自证那一格喂的是「selftest_invoke "$cell" e53.out selfcheck 4」（`scripts/experiment-apparatus/e53-vm-bench-run.sh:171`）。另外 E53 要 `VM_DISKS=4`、脚本不设也不核；装置只要两块以上（`scripts/experiment-apparatus/e53_ring_failure.rs:168` 是 `if device_paths.len() < 2 {`），给 2、3 块照跑出一份 `devices=2` 的产物。
- E159：「USAGE_LINE="用法：e159-vm-bench-run.sh <输出文件> <子命令> [选项…]"」（`scripts/experiment-apparatus/e159-vm-bench-run.sh:73`），实现只核输出文件：「if [[ -z "${1:-}" ]] && ! break_is missing-output-accepted; then」（`scripts/experiment-apparatus/e159-vm-bench-run.sh:327`），不给子命令照起虚机。

E159 的结果行前缀核过：装置打 `E159RESULT …` 经 `Emitter::emit_raw`，它在前面加 `E7RESULT `（`scripts/experiment-apparatus/e7_index.rs` 外的库里 `format!("E7RESULT {body}")`，转述），入口按 `^E7RESULT ` 抓得到。

## 第 5 问：E185 三份钉值

- 同一个量：成立（量的）。`sha256sum crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs` 得 `d312b9aa71d0518976fd383b48300705f4598d2e79d4a3d59488657e684a51eb`，HEAD 与暂存区那一份同值；三份脚本的 `registered_apparatus_sha256` 都是它。装置自己比的是编进去的源码原文（`OWN_SOURCE_TEXT`）的 sha256，与脚本现算的是同一份字节。
- 钉值等于登记第十二节：不成立（打中）。脚本自己的文件头：「装置源码的 sha256 要等于登记第十二节写的那一个（下面 registered_apparatus_sha256）；不等就写完输入清单、退 3（停机 K-S6），不编、不跑。」（`scripts/experiment-apparatus/e185-r4-segment-run.sh:18`）。登记第十二节 R4-1 写的是 `9e813cbd36900f1b0c7001e6de0f98de01080bbf7ac6aab39d8e7c9854bd97f8`（`research/prompts/e185-r4-prereg.md:579`），停机条款「K-S6 装置变了：输入清单第一行、第十二节执行员写的、产物 `name=apparatus` 行三处的装置 sha256 有一处不同」（`research/prompts/e185-r4-prereg.md:538`）。r3 登记已归档（删于提交 `a24379563`），它第十二节 R3-1 写 `4368c0e7…`、停机 J-S7 比的就是它（`git show a24379563^:research/prompts/e185-r3-prereg.md` 第 451、477 行，量的）。所以「开跑退 3」本来就是登记的停机条款在起作用；重钉成现算值，把停机变成照跑，而脚本自己的出路要两处一起改：「→ 怎么办：核装置是不是换过；换过就交主 agent 改登记第十二节与这一份的 registered_apparatus_sha256，不照跑」（`scripts/experiment-apparatus/e185-r3-segment-run.sh:467`）。登记第十二节没改（这一轮的改动里没有 `research/prompts/e185-r4-prereg.md`）。r2 的脚本没有登记第十二节那句，只把钉值交给装置当停机 7 比。
- 四句：不分臂；系统看得到（起跑脚本读得到登记文件，只是不读）；满足的是 r4 脚本第 18 行「要等于登记第十二节写的那一个」的反面；正文给的改法（重钉）在这一格上照中。
- 「三份自证都绿」不证钉值：自证把钉值换成自证给的：「registered_apparatus_sha256=$E185_R3_SELFTEST_REGISTERED_SHA256」（`scripts/experiment-apparatus/e185-r3-segment-run.sh:104`），r2、r4 同写法。钉值对不对只有真开跑那一刻判。
- 装置再改一次：成立，照样只在开跑时退 3。改装置的那次提交里没有任何一道把钉值对着源码或登记核。
- 以后怎么拦（零轮，推的）：把钉值从三份脚本里拿掉，起跑脚本从它点名的登记文件第十二节读那一个 sha256（单一出处），再加一格门禁：凡有 `registered_apparatus_sha256` 或从登记读钉值的起跑脚本，装置源码在这次改动里变了而登记第十二节没新加一行修订的，判红、出路写「补登记第十二节一行修订（新 sha256、改了什么）」。这样装置改动在提交时红、不在开跑时红，也不再绕过停机条款。

## 要主 agent 跑的命令

四条互不依赖，都在仓根起；运行目录事先不能存在（探针自己建），重跑先删那一个运行目录。跑完运行目录里有临时 git 仓，读完结果请删。

1. 第 1 问六臂。在 `/home/fy5090/code/singlefs` 起：

   `task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/probe/hook_swap_lock_probe.py /home/fy5090/code/singlefs/scripts/gate/hook-swap.py /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/hook-swap-lock-run`

   看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志末行 `probe-done arms=6`；打中的判法：`arm=outcome-overlap-different-scratch` 那一行 `next_swap_pending_refusal=none` 且 `x1_landed=True x2_landed=False`（对照臂同位置是 `9`）；`arm=seal-outcome-while-swap-running` 那一行 `seal_exit=0` 且 `read_outcome_exit=0`；`arm=gap-three-contenders-on-stale-record` 那一行 `holding=A,C`；`arm=gap-link-reply-lost` 那一行 `refusal_names_own_pid=True`；`arm=same-machine-id-other-boot` 那一行 `live_holder_counts_as_occupying=False`。哪一行不是这样，那一格按「没打中」改判。

2. 第 3 问四臂。在 `/home/fy5090/code/singlefs` 起：

   `task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/probe/deliverables_refusal_probe.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/deliverables-refusal-run`

   看：`exit=0`、末行 `probe-done arms=4`；打中的判法：`arm=parent-directory-bad-line-blocks-child-handback action=block named=True`、`arm=quoted-path-with-space-blocks action=block named=True`（多拒）；`arm=symlinked-declaration-with-bad-line action=pass`、`arm=nested-declaration-with-bad-line action=pass`（漏拒）。

3. 第 2 问复跑表 exact 行。在 `/home/fy5090/code/singlefs` 起：

   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 evidence-in-repo 一格，跑在攻方腿改过的样本根上; 范围=1 格; 为什么=核同一个实验既有 exact 行又有 timing 行时这一格是否照文件头仍比修改时刻' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/probe/evidence-mixed-replay-feed.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/evidence-mixed-replay-run`

   看：日志里 `replay-rows-for-E913:` 下两行（一条 timing、一条 exact）；打中的判法：有一行以「按复跑表判 1 份」起头、点名 E913，且没有「scripts/experiment-apparatus/e913_replay_timing.rs（装置源码，改于 2026-09-05）」那一行。末行 `stage-exit=` 是样本里其余红例照红的退出码（预期 1），不作这一格的判据。

4. 第 2 问 `gate_added_lines` 的标准错误。在 `/home/fy5090/code/singlefs` 起：

   `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/probe/added-lines-error-feed.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-attack/added-lines-error-run`

   看：`gate_changed_paths exit=1 header_line=1` 与 `gate_added_lines exit=1 header_line=0` 两行；后一行 `header_line=1` 就是 `.claude/gate.d/doc-process-records.sh` 第 841 行那句成立、这一格改判没打中。

第 3、4 条起的是门禁阶段与共用库，若被范围声明闸或项目代码闸拒，照拒绝说明改声明再交；第 1、2 条只起草稿目录里的 python 探针，探针在进程里载入被判脚本。

## 没打中的形状

- 第 1 问：两趟摘同一份旧记录的各种先后（两趟到不了双持，要第三趟在摘走与放回之间）；进程号复用、重启（启动刻、开机号挡住）；两台钟差在 285 秒内的修改时间；NFS 属性缓存（打开即重取）；放成记录与装信号处理之间的 TERM（记录留下、同一台立刻让路）；同一个草稿目录经符号链接写成两个字符串（落到同一个文件）。
- 第 2 问：六份调用方里有没有把标准错误并进取值（没有）；`set -u` 下没初始化的 `<前缀>_error`（带默认值，不炸）；取到空与多行的值与状态。
- 第 3 问：全角空格分隔（Python 的 `split()` 认）；`..` 与绝对路径（照拒）；种类后面带冒号（照拒）；挪场景之后的名字复用。
- 第 4 问：E159 结果行前缀（经 `emit_raw` 加了 `E7RESULT `）；二进制路径取 cargo 报的；相对输出路径。
- 第 5 问：钉值与装置自比的是不是两个量（是同一个量）。

## 这条腿自己的限度

- 四份探针一份都没跑，上面所有「预期打」都是推的；探针本身没被攻过。
- 读到的范围：`scripts/gate/hook-swap.py` 的占用一段、`swap` 与结局文件几个函数、`main`；`scripts/gate/changed-paths.sh` 的差与 `gate_added_lines`；三道文档门禁的改动与预取段；`.claude/hooks/completion-cleanup-guard.sh` 的差、`declarations`、`subagent_roots` 尾部与自证第 2540–2660 行；E07 入口整份、E53 与 E159 经归一化的差；E185 三份的钉值段、r4 登记第十一、十二节与归档的 r3 登记两行；`.claude/rules/test-script-discipline.md` 整份；`scripts/devices/vm-bench.sh` 的内核与抓取段。没读：`.claude/hooks/project-code-guard.sh`、`scripts/hosts/memory_peaks_lib.py`、`CLAUDE.md`、`.claude/main-agent.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/investigator.md`、`.claude/rules/code-discipline-by-review.md`、`.claude/gate.d/code-tooling.sh` 第 2344 行起那几项（第 6 到 10 问分给正推腿）。
- 第二台：machine-id 只从一份结局文件的开始行推出不同；钟、NFS 版本、内核镜像、KVM 都没核。
- 第 1 问的「新格」与第 5 问的「登记第十二节」是正文没点名的角度，判决要不要收由主 agent 定。

## 没做什么

不判第 6 到 10 问；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数（四份探针的读数）没复跑，命令在上一节；探针与喂法在草稿目录，不是入库装置，读数不算入库装置上的数。草稿目录里没建仓副本、没建编译目录，没有要删的；探针跑出的四个运行目录（`*-run`）由跑它的主 agent 读完删。

