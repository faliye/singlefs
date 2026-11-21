# 治理文件轮第一轮（ledger-topdown-governance-r1）攻方腿报告

日期：2026-11-18。被判文件按开工快照 `research/prompts/ledger-topdown-governance-r1-snapshot/judged-sha256.txt` 读：开工时逐份 `sha256sum -c`，不对的 0 份（读的那一刻与快照逐字相同）。

复跑命令与模型：这条腿没写模型（没有崩溃状态枚举、没有模型目录），只在草稿目录 `/home/fy5090/.local/share/singlefs/work/governance-r1-attack/` 里写了喂钩子的喂法与两棵探针小树（`feeds/`、`q2/`、`q3/`、`q6/`），全部交主 agent 跑，命令在末尾「要主 agent 跑的命令」。报告里凡没标「量的」的读数都是推的。

## 各格判定一览

| 问 | 结论 | 打中几条 | 一句话 |
|---|---|---|---|
| 1 写闸第十一道 | 推不出（打中） | 3 | Bash 写两个服务 `src/` 不拦；同一理由的 `Cargo.toml` 不在罩里；出路里的 `--verify` 对 50 份要临时判定库服务的自证用不了 |
| 2 换上工具退 8、退 9、按键认库 | 推不出（打中） | 4 | 键取的库在本文件里没存过时静默不判；只用 `from 库 import` 的调用方整份不核（早于这一轮，104 份真调用方）；退 9 只看已写开始行的那一趟，两趟并发互盖结局文件；退 8 把坏的旧钩子诊断成喂法失效 |
| 2 补点名三格对不对得上 | 成立 | 0 | 任务 19596 日志里 7 个格名与文件头 7 格相同（量的，读日志）；调度记录写的「红 9 格」与日志 12 行、7 格都对不上，是记录的数错 |
| 3 规格工具登进直接起的名单 | 推不出（打中） | 2 | 设 `SINGLEFS_WORK_ROOT` 就能让它把 `--out` 写进仓里任一目录；检出钩子 ⑫ 不认它的 `--out` |
| 4 本次未跑记号行 | 推不出（打中） | 2 | 文档与提醒三处都没写「不带 `--nocapture` 读不到」；接在 libtest 结果行起头后面的记号漏认，而仓里门禁的读法认它 |
| 5 复跑入口按内存取并行度 | 推不出（条件打中） | 1 | 两台 `SLICE_MEMORY_CAP_BY_HOST` 不同 → 并行行数不同 → 计时行开跑时残留负载不同 → 「本次不判」与「判了」可以不同；只读函数那一种「取法被盖」在其余 16 格里没有 |
| 6 注释历史的四个例外 | 推不出（打中） | 2 句 | 「以前面临」「以前面向」两句真讲过去，命中「面」那个例外；「除以前登记的」较弱 |
| 9 自证与弄坏开关 | 推不出（打中） | 3 | 规格工具 4 类新拒绝分支没有开关也没有自证格；项目代码闸两条开关的点名格自标「推的，没跑」；换上工具 22 条开关不进开关复跑表，后改的代码没有复跑兜底 |

第一轮三个攻击面落在哪：过度外推与越位——问 1 的 L2、问 3、问 4 的漏认；内部矛盾与循环依赖——问 1 的出路 O1、问 3（名单的理由与工作根可改相矛盾）、问 4（任务服务与门禁对同一记号两种认法）；最脆弱的一处——问 2 的并发与问 9 的换上工具开关没有复跑兜底（判在第 9 节末）。

## 问 1：写闸第十一道罩两个服务的 `src/`

结论：推不出。三条打中，两条是漏放（L1、L2），一条是出路过不了（O1）。误拒没找到（见末节「没打中的形状」）。

### L1 漏放：工具员在 Bash 里改两个服务 `src/` 下已存在的文件，没有一道闸拦

- 第十一道只挂 Write / Edit / MultiEdit，文件头只对脚本说了 Bash 那一半看不见：「Bash 里用定点替换工具分几次改同一份脚本，这道闸看不见。」（`.claude/hooks/write-guard.sh:91`）。`src/` 下的 `.rs` 不是脚本，这一句没罩到它。
- 定义却说 `src/` 已存在的文件不在主工作区改：「同样不在主工作区改（写闸第十一道执行前拒）」（`.claude/agents/tooling-writer.md:31`）。
- Bash 一侧：⑦ 只认脚本、且放行追加：「放行：`>>`、`&>>`、`tee -a`、python 的 `'a'` 追加」（`.claude/hooks/bash-command-detector.sh:216`）；⑧ 只认 `sed -i`、`perl -i` 与 python 就地改，`scripts/kb/replace-once.py` 还是 ⑦ 拒绝说明里给的出路；⑫ 照写范围判，而工具员的写范围含整个服务目录：「tooling-writer prune-server/**」（`.claude/hooks/agent-write-scope.tsv:38`）。
- 最小输入（喂 `.claude/hooks/bash-command-detector.sh`，`agent_type` 是 `tooling-writer`，`cwd` 仓根）：
  1. `cat >> prune-server/src/protocol.rs <<'EOF2'` 加一行注释——预期退 0（放行）。
  2. `python3 scripts/kb/replace-once.py prune-server/src/protocol.rs 'pub ' 'pub '`——预期退 0。
  3. `cp <草稿目录>/q1/protocol.rs prune-server/src/protocol.rs`——预期退 0（`cp` 不换 inode，写到一半对别的会话可见，正是这道闸要挡的中间态）。
  对照：同一个 agent `cp` 盖 `prune-server/control`（带执行位）预期退 2（⑦）；`sed -i` 改 `protocol.rs` 预期退 2（⑧）。喂法在 `feeds/detector.json`，命令见末节第 1 条。
- 四句：分辨臂——分辨（这一轮把 `src/` 归进「只许经补丁换」，而 Bash 这一半没跟上）；判别子观测得到——⑫ 已经算出落点在 `prune-server/src/` 下，看得到；满足问 1 的「会不会漏放（别的写文件的写法绕得过）」那一分句；跑前没给改法。

### L2 漏放：与 `src/` 同一个理由的构建清单不在罩里

- 第十一道给 `src/` 的理由是：「源码改到一半，主工作区里的服务编不过，带临时判定库服务的自证都取不到二进制」（`.claude/hooks/write-guard.sh:81`）。
- 判法只看 `src/` 前缀，别处不带执行位、不是 `.sh` / `.py` 的一律放：「if not source and not relative.endswith(SWAP_ONLY_SCRIPT_SUFFIXES) and not executable:」（`.claude/hooks/write-guard.sh:739`）。
- 落在这个缝里的已存在文件（`git ls-files` 现列）：`prune-server/Cargo.toml`、`prune-server/Cargo.lock`、`gpu-verification-server/Cargo.toml`，与两个服务 `tests/` 下 11 份 `.rs`。分几次 Edit 改 `Cargo.toml` 同样让服务编不过；`gpu-verification-server/Cargo.toml` 还是仓根工作区的成员：「仓根工作区的成员」（`gpu-verification-server/Cargo.toml:3`），改到一半连 `crates/` 的编译一起坏。
- 最小输入（喂 `.claude/hooks/write-guard.sh`）：`{"tool_name":"Edit","agent_type":"tooling-writer","tool_input":{"file_path":"<仓根>/prune-server/Cargo.toml",…}}`——预期退 0；`gpu-verification-server/Cargo.toml` 同样退 0；对照 `prune-server/src/protocol.rs` 预期退 2。喂法在 `feeds/write-guard.json`。
- 四句：分辨臂——分辨（罩的范围是这一轮定的）；判别子观测得到——路径就在输入里；满足「会不会漏放」；跑前没给改法。`tests/` 那几份不影响服务二进制，算不算该罩交主 agent 判。

### O1 出路过不了：拒绝说明里的 `--verify` 对要临时判定库服务的自证用不了

- 拒绝说明的出路：「再去掉 --dry-run、加 --verify '<被换脚本的 --selftest 命令>' 换上。」（`.claude/hooks/write-guard.sh:756`）。
- 换上工具在主工作区里用 `bash -c` 照本进程的环境起这条命令：「done = subprocess.run(["bash", "-c", command], cwd=root, stdin=subprocess.DEVNULL, capture_output=True, text=True, env=environment,」（`scripts/gate/hook-swap.py:1421`）。
- `.claude/hooks/selftest-temporary-service.tsv` 里登记成 required 的 50 份（`awk` 现数；全在 `scripts/` 下，都在第十一道的目录里）裸写 `--selftest` 退 2；包装写法又要任务服务给的编译目录：「不经任务服务起、没有 CARGO_TARGET_DIR 时退 2，不现编」（`.claude/hooks/selftest-temporary-service.tsv:6`）。任务服务只在包装是命令词时认它、给编译目录：「if index >= len(words) - 1 or os.path.basename(words[index]) != TEMPORARY_SERVICE_SCRIPT or words[index + 1] != TEMPORARY_SERVICE_FLAG:」（`task-server/task_server_lib.py:2484`）；`--verify` 里那一串是 `hook-swap.py` 的一个参数，不是命令词。
- 推的结局：照拒绝说明写 `--verify` 的，换上之后验证退 2、回滚、整条退 5；能换上的只有不带 `--verify` 的写法，也就丢了「验不过就回滚」。这一轮就是这样换的：「`replay.sh` 两份规格不带验证连着真换 19770、19771」（`records/2026-11-17-subagent账本清欠调度记录.md:257`）。
- 四句：不分辨臂（`--verify` 与临时服务的冲突早于这一轮，这一轮把它写进了拒绝说明与定义）；判别子观测得到（登记表就在仓里，拒绝说明可以按表分两种出路）；满足「拒绝说明给的出路本身过不过别的闸」；跑前没给改法。
- 补丁那一条出路：拒绝说明与定义都没点名用什么打。`scripts/gate/apply-writer-patch.py` 文件头对 `crates.patch` 之外的补丁给的出路是：「出路是用 git apply <补丁文件> 直接打、回读核过」（`scripts/gate/apply-writer-patch.py:22`），`git apply` 逐份就地写，正是第十一道想挡的中间态，只是窗口短。这一条我只算弱点、不算打中。

### 我提的改法（只在我的读法上推过、被攻过零轮）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| ⑦ 另认工具员往 `SWAP_ONLY_SOURCE_DIRECTORIES` 写已存在文件（含 `>>`、`cp`、`replace-once.py`） | L1 | 推的，待主 agent 跑 |
| 第十一道的源码一类加上两个服务的 `Cargo.toml`、`Cargo.lock`（`tests/` 交主 agent 定） | L2 | 推的，待主 agent 跑 |
| 拒绝说明按 `selftest-temporary-service.tsv` 分两种：required 的写包装加 `--server-binary <两台都读得到的二进制>`，或另交一条自证任务、并写明丢了回滚 | O1 | 推的，待主 agent 跑 |

## 问 2：换上工具的退 8、退 9、按字符串键认调用方引到的库

结论：推不出。四条打中（K1、K3、R1、N1，K3 早于这一轮）；「补完没再复跑」那一次的点名，按日志成立。

### K1 按键认库：取的键在本文件里没存过时，既不跟也不列「本次未判」

- 文件头的承诺：「怎么认「载入了哪个库、绑在哪个变量上」写在 CALLER_SCAN_DIRECTORIES 前的注释；认不出去向的写法逐处列成「本次未判」，不当成查过。」（`scripts/gate/hook-swap.py:85`）。
- 代码：本文件里这个容器没有按字符串键存过库，就直接交空、不交未判的说明：「if not stores or (isinstance(index, ast.Constant) and not isinstance(index.value, str)):」（`scripts/gate/hook-swap.py:864`），下一行「return (("module", positional) if positional else None), None」（`scripts/gate/hook-swap.py:865`）。之后 `collect` 只在 `container_entry` 交了未判说明时才记，这一处两样都不记。
- 仓里真有这种写法：「canonical = sys.modules["project_preflight"].shared_preflight_directory()」（`scripts/gate/lib_function_units.py:18351`，第 18382 行同一句），这份文件载入那份库用的是「from project_preflight import preflight  # noqa: E402」（`scripts/gate/lib_function_units.py:1331`），没有往 `sys.modules` 存过；另一处「client = sys.modules["lib_verdict_library"].prune_server_client()」（`scripts/experiment-apparatus/e165-pin-red-examples.py:1516`）。
- 会判错的规格：一份脚本规格把 `.claude/scripts/project_preflight.py` 里的 `shared_preflight_directory` 改名（`.claude/scripts/` 在脚本规格目录里）。推的结局：引到的名字「缺 0 处」、这两处既不算缺也不进「本次未判」，照换；之后 `lib_function_units.py` 那两格一跑就 `AttributeError`。这两份真调用方实际是在更早一步就漏掉的（见下面 K3：文件里只有 `from … import`，没绑上库就提早退出），第 864 行这一支要在调用方另外绑过库时才轮到，探针照这一形造。
- 探针（草稿目录 `q2/`，交主 agent 跑，末节第 4、5 条）：`q2/root` 里库只剩 `preflight`，调用方 `scripts/gate/caller_by_key.py` 先 `import project_preflight as loaded`、用一次 `loaded.preflight`（让它不在第 808 行提早退出），再写 `sys.modules["project_preflight"].shared_preflight_directory()`；`q2/control` 同一库、调用方写 `import project_preflight` 再 `project_preflight.shared_preflight_directory()`。预期：`root` 那一条退 0、「1 处引用，缺 0 处，本次未判 0 处」；`control` 那一条退 1、点名 `caller_by_attribute.py` 引了 `project_preflight.shared_preflight_directory`。两条结局不同就坐实。
- 四句：分辨臂——分辨（按键认库是这一轮加的，这一支是它的缺口）；判别子观测得到——键是写死的字符串、折得出，库名也在要核的库里；满足问 2「把不该换的放过去」；跑前没给改法。

### K3 只用 `from 库 import 名字` 引的调用方整份不核（早于这一轮）

- 文件头的承诺：「凡是 <载入它的变量>.<名字>（含 from lib import 名字、getattr(库, "名字")）用到被换库的名字，换上之后那一版的模块层里都要有」（`scripts/gate/hook-swap.py:84`）。
- `from 目标库 import 名字` 不绑变量：「if base not in self.targets and alias.name in self.targets:」（`scripts/gate/hook-swap.py:1050`）；一份源码里没绑上任何库、也没有载入的调用，就在收集之前退出：「if not self.bound and not any(self.load_call_kind(node) for node in self.statements if isinstance(node, ast.Call)):」（`scripts/gate/hook-swap.py:808`），下一行「return    # 没载入任何目标库，也没有谁把库交进来」（`scripts/gate/hook-swap.py:809`）。记 `from … import` 那一处用法的「self.uses.append((node.lineno, base, alias.name))」（`scripts/gate/hook-swap.py:1206`）在 `collect` 里，走不到。自证的七份调用方样本没有一份只用 `from fake_lib import`。
- 仓里这样引的多：`from project_preflight import`、`from lib_atomic_replace import`、`from heavy_threads import` 三种在调用方扫描的五个目录下（去掉 `fixtures/`）现数 104 份 `.py`（`grep -rlE … | wc -l`，量的；只数 `.py`，`.sh` 里的 heredoc 没数），三份库都在脚本规格目录里。会判错的规格：从 `.claude/scripts/project_preflight.py` 删掉或改名 `preflight`。推的结局：引到的名字缺 0 处，照换，之后这些脚本一 import 就挂。
- 探针 `q2/fromimport`（末节第 15 条）：库里没有 `shared_preflight_directory`，调用方只写 `from project_preflight import shared_preflight_directory`。预期退 0、缺 0 处。
- 四句：不分辨臂（提早退出那一句在 HEAD 里就有，`git show HEAD:scripts/gate/hook-swap.py` 第 771 行；这一轮的按键认库建在它上面）；判别子观测得到；满足问 2「把不该换的放过去」；跑前没给改法。改法（零轮）：提早退出的条件里把「有 `from 目标库 import`」算作有绑定。

### R1 退 9：只认已经写了开始行的上一趟，两趟并发时互盖结局文件

- 退 9 的说法：「9 上一趟换上还没完：起它的进程还在这一台上跑（还在换或在验），拒绝再换，一个字节没写，等它写完成再来。」（`scripts/gate/hook-swap.py:106`）。
- 起手只读结局文件：「unfinished = pending_refusal(root, outcome_file, mode)」（`scripts/gate/hook-swap.py:1789`）；上一趟要到草稿验完、预演过之后才写开始行（`scripts/gate/hook-swap.py:1897-1901`）。上一趟在草稿里验（钩子规格要逐条喂，几十秒到几分钟）时，结局文件还是更早那一趟的、带完成行，这一趟照样往下走。几份文件全在换前时同样放：「return None   # 被杀在第一份换上之前（或已手工退回）：一份都没换，没有半截」（`scripts/gate/hook-swap.py:1724`）。
- 整个脚本里没有锁（`grep -c flock scripts/gate/hook-swap.py` 现数 0）；同一类的打补丁工具有：「同一个仓同一时刻只许一个在换：<git-dir>/singlefs-apply-writer-patch.lock 上 flock，不等，拿不到退 2（被杀时内核放锁）。」（`scripts/gate/apply-writer-patch.py:40`）。任务服务也不排它们的先后，只把它们认成改主工作区的命令、撞上限不重排：`task-server/task_server.py:969`「WORKSPACE_WRITERS = {"scripts/gate/hook-swap.py", "scripts/gate/apply-writer-patch.py", "scripts/gate/stage-mine.py"}」。
- 结局文件只有一份、每次整份重写：「位置：<仓根>/.git 的 git-dir 下的 singlefs-hook-swap-outcome.tsv（每次真换整份重写，只留最近一趟）」（`scripts/gate/hook-swap.py:49`）；之后的回读、完成行按路径追加：「with open(path, "a", encoding="utf-8") as handle:」（`scripts/gate/hook-swap.py:1620`）；判完没完只看最后一个完成或退回行：「ending = next((row[0] for row in reversed(rows) if row[0] in ("完成", "退回")), "")」（`scripts/gate/hook-swap.py:1653`）。
- 历史（两个会话、两份文件不重叠的脚本规格 甲、乙，都带 `--verify`）：甲过了起手、在草稿里验；乙起手读到旧的完成行，放行；甲写开始行、换上；乙写开始行（整份盖掉甲的开始与文件行）、开始换；甲回读、跑 `--verify`、往乙的那份文件追加甲的回读与完成行；乙在两份之间被杀（撞上限、断电）。推的结局：下一趟起手读到末尾的完成行（是甲的），照换，乙停在半截的那几份没人拦、也没人报，`--read-outcome` 按乙的文件行对照会把它报成 4（完成之后又被改）而不是 3。用户在这几步里不需要做什么特别的事，只要两个会话前后几十秒各交一条换上命令；两份规格碰同一份文件时还会被第 3 种退出码挡住，不重叠的挡不住。
- 四句：分辨臂——分辨（结局文件与退 9 都是这一轮加的；它承诺的「半截不再换」在并发下失效）；判别子观测不到——起手那一刻甲还没写开始行，单看结局文件分不出「没有别的一趟」与「别的一趟还在草稿里」，要靠锁；满足问 2「把不该换的放过去」；跑前没给改法。
- 这一条我没造得出能照跑的确定性探针（要卡准两条任务的时刻），全是推的；改法是起手到写完成之间持一把与打补丁工具同形的 `flock`，被杀时内核放锁。

### N1 退 8：坏的旧钩子被诊断成「喂法失效」

- 退 8 的判法是拿换上之前那一版喂标了 neutral 的喂法（`scripts/gate/hook-swap.py:1851`），对不上就说：「，是喂法失效，不是规格的毛病」（`scripts/gate/hook-swap.py:634`）。stderr 里有坏钩子的话也算对不上：「BROKEN_HOOK_MARKERS = ("没判成", "读不到共用模块")」（`scripts/gate/hook-swap.py:211`）。
- 会判错的写法：主工作区那一版钩子坏了（共用模块读不进、或把 `ls -la` 也拒了），要换上的正是修它的规格，喂法用内置那一份（内置里有「{"label": "主 agent 跑 ls -la 放行", "command": "ls -la", "exit": 0, "neutral": True},」，`scripts/gate/hook-swap.py:240`）。推的结局：退 8、一个字节不写，拒绝说明叫人改喂法；内置喂法改不了，要另写 `--feeds` 去掉 neutral 才换得上。文件头自己写着钩子坏掉是这条脚本要应付的情形：「钩子在 bash 这一层坏了时，每一条 Bash 调用都会被它拒（退 2），连改回去的那一条也起不来」（`scripts/gate/hook-swap.py:95`）。
- 四句：分辨臂——分辨（退 8 是这一轮加的）；判别子观测不到——只喂旧版，分不出「喂法写错」与「旧版本来就坏」，判据该改成两种都列、不下断语；满足问 2「把该换的挡住」；跑前没给改法。有出路（自写 `--feeds`），所以只算误诊加多一步。

### 补完没再复跑的那三格

- 记录：「补完没再复跑这一条」（`records/2026-11-17-subagent账本清欠调度记录.md:217`）。我读了任务 19596 的整份日志（`task-server/task log 19596 --all`，存在草稿目录 `log-19596.txt`）：`✗ hook-swap 自检：` 起头的 12 行，按格名去重是 7 格，与文件头 `swap-on-draft-red` 下现在点名的 7 格（`scripts/gate/hook-swap.py:111-112`）逐个对得上。19594、19595 两条（`subscript-key-ignored`、`skip-caller-names`）各 3 格，同样对得上。
- 对不上的是记录里的数：「`swap-on-draft-red` 19596 红 9 格」与日志的 12 行、7 格都不等。结论成立，但记录那一句的「9」该改。

## 问 3：规格工具登进直接起的名单

结论：推不出。两条打中（S1、S2），S1 是能照喂的越权写。

### S1 设 `SINGLEFS_WORK_ROOT` 把「工作根」挪到仓根，`--out` 就能写进仓里

- 名单登它的理由是只写工作根：「"scripts/gate/make-swap-spec.py": "any",」（`.claude/hooks/lib_project_code_runs.py:1107`，同一行注释写「只读两棵树、只写工作根下的 --out」），`scripts/gate/README.md:21` 同一句。`any` 只挡 `--selftest`：「if form == "any":」（`.claude/hooks/lib_project_code_runs.py:1197`）。
- 它认的工作根是现取的：「return module.scratch_roots()」（`scripts/gate/make-swap-spec.py:219`），而工作根照环境变量取：「工作根：环境变量 SINGLEFS_WORK_ROOT 设了就用它」（`.claude/hooks/lib_work_root.py:31`）。核 `--out` 只比这一组根：「roots = [os.path.realpath(root) for root in work_roots]」（`scripts/gate/make-swap-spec.py:166`）。
- Bash 一侧不认它的写：⑫ 认的定点写脚本只有五个：「KNOWN_WRITE_SCRIPTS = ("replace-once.py", "replace-batch.py", "insert-row.py", "ledger-append-section.py", "append-record.py")」（`.claude/hooks/bash-command-detector.sh:1887`）；共用约束也写明脚本里的写钩子判不到：「钩子判不到的写法（代码在变量里的 `eval "$c"`、脚本文件里的、python / perl / ruby / tclsh 起的脚本文件里起的）全靠自己守。」（`.claude/agent-common.md:32`）。
- 最小输入（任一项目子 agent；我用写范围最窄的 `three-way-attack`）：`SINGLEFS_WORK_ROOT=<仓根> python3 scripts/gate/make-swap-spec.py --base <草稿>/q3/base --new <草稿>/q3/new --out <仓根>/.claude/settings.json x.txt`。推的结局：项目代码闸退 0（名单上的文本工具）、检出钩子退 0（⑫ 不认 `--out`）、脚本自己核 `--out` 过（父目录 `<仓根>/.claude` 在「工作根」下），整份 `.claude/settings.json` 换成一个 JSON 数组。内容由 `--base`、`--new` 两棵树定，调用方说了算；落点是仓里根下任一层目录里的任一份文件（根本身那一层不收）。
- 怎么核：喂法在 `feeds/detector.json` 与 `feeds/project-code-guard.json`，只判不跑（末节第 1、2 条）；脚本那一层在草稿目录里的沙箱上演一遍（末节第 6、7 条：把工作根设成 `q3/fakework`，`--out` 写进它下面，预期写出；把工作根设成 `q3/otherroot`，同一个 `--out` 预期退 3）。我自己在草稿目录里直接起这一条被会话的权限分类器拒了，没跑成，所以这一节全是推的。
- 四句：分辨臂——分辨（登进名单是这一轮的改动）；判别子观测得到——项目代码闸切词时已经拿到命令前的赋值（`assignments`），⑫ 也看得到 `--out`；满足问 3「子 agent 借它能往哪里写、哪道闸本该拦而拦不到」；跑前没给改法。

### S2 不设环境变量：临时文件跟着符号链接写出去

- 写出用的是可预测名字的临时文件：「temporary = f"{out_path}.{os.getpid()}.tmp"」（`scripts/gate/make-swap-spec.py:201`），下一行「with open(temporary, "w", encoding="utf-8") as handle:」（`scripts/gate/make-swap-spec.py:202`）跟随符号链接。草稿目录里事先放一串 `spec.json.<进程号>.tmp` 指到仓里的链接（建链接落点在工作根下，⑫ 不判：「落点在本仓里而不在范围里的拒；落点算得出而在仓外的（/tmp、/dev/null、工作根下的草稿目录）不判」，`.claude/hooks/bash-command-detector.sh:250`），撞上进程号的那一个就把规格写进链接指到的仓内文件，随后 `os.replace` 只换掉链接本身。推的，要猜进程号，比 S1 难用；改法是建临时文件用 `O_CREAT | O_EXCL | O_NOFOLLOW`。

### 我提的改法（零轮）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 工作根改从多机配置读、不认命令前的 `SINGLEFS_WORK_ROOT` / `SINGLEFS_DATA_ROOT`（或把规格工具挪出 `TEXT_TOOLS_RUN_DIRECTLY`，只许主 agent 起） | S1 | 推的，待主 agent 跑 |
| ⑫ 的 `KNOWN_WRITE_SCRIPTS` 加 `make-swap-spec.py`，目标取 `--out` | S1 | 推的，待主 agent 跑 |
| 临时文件排他建、不跟链接 | S2 | 推的，待主 agent 跑 |

## 问 4：任务服务认「本次未跑」记号行

结论：推不出。两条打中（D1 文档缺口、M1 漏认形态）；误认形态没找到坐得实的（见末节）。

### D1 文档与提醒都没写「不带 `--nocapture` 读不到」

- 三处现状句都只说「行首」与「夹在中间不算」，一处都没提捕获：README「夹在别的输出中间的不算；分片的按片判，取消的不判），报 `gpu-cases-not-run`」（`task-server/README.md:149`）；异常表那一行「夹在别的输出中间的不算；退出码与结局种类照命令的，命令退 0 仍是成功｜依据里点名的这几条用例判段没执行、不算过」（`task-server/task_server_lib.py:1798`）；文件头「gpu-cases-not-run  任务（分片的一片按自己的；分片的父任务没有自己的尝试；取消的不判）出结局，最后那一次尝试的日志里有用例「本次未跑」的标记行：行首是 GPU_NOT_RUN_MARK」（`task-server/task_server.py:253`）。三份文件里 `nocapture` 现数各 0 处（`grep -c`）。
- 记号由测试用 `println!` 打：「println!("{}", not_run_line(test_name, &reason));」（`crates/singlefs-verification/src/gpu_unit_checks.rs:1906`），验证包自己的注释写明它进 libtest 捕获：「/// 打到标准输出：用 `println!`，与 [`GPU_NOT_RUN_MARKER`] 那一行同样进 libtest 捕获的这条用例的输出，用例红了 libtest 把它交出来。」（`crates/singlefs-verification/src/gpu_memory_ledger.rs:686`）。调度记录量过：「带 `--nocapture` 的 19471 服务发了 gpu-cases-not-run 提醒，不带的 19467 没有（标记行被 libtest 捕获）」（`records/2026-11-17-subagent账本清欠调度记录.md:172`）。
- 会让人把「没提醒」读成「都跑了」：共用约束教的点名测试写法本身不带 `--nocapture`：「task-server/task submit -- cargo test -p <crate> --test <目标> -- <测试全名…> --exact」（`.claude/agent-common.md:61`）。照规定写的每一条点名测试，用例没跑也收不到提醒，结局说明里也没有「这一趟有 N 条用例本次未跑」那一段。
- 四句：分辨臂——分辨（提醒是这一轮加的）；判别子观测不到——捕获之后日志里没有这一行，服务没法知道；满足问 4「这个缺口在文档与提醒里写没写明、会不会让人把没提醒读成都跑了」；跑前没给改法。改法只能补文档或改测试的打法（例如带 `--nocapture` 或 `--show-output` 才认、不带时在结局说明里写一句「这一趟点名的测试没带 `--nocapture`，本次未跑的记号读不到」）。

### M1 漏认：记号接在 libtest 结果行起头后面

- 服务只认行首：「if not line.startswith(GPU_NOT_RUN_MARK):」（`task-server/task_server.py:1119`），并明说「记号前面有别的字（夹在别的输出中间、前面有空格）或后面接着别的字的不是，交 None。」（`task-server/task_server.py:1118`）。
- 仓里门禁读同一个记号的库认这几种形态，并在自证里钉着：单线程「"test gpu_second ... NOT_RUN_NO_GPU_ADAPTER test=gpu_second reason=样本", "ok",」（`scripts/gate/lib_libtest_result_lines.py:508`）；几条同时跑、别的测试打的记号接在结果行起头后面「"test parallel_first ... NOT_RUN_NO_GPU_ADAPTER test=parallel_third reason=样本", "ok",」（`scripts/gate/lib_libtest_result_lines.py:526`）；它的认法是行内任一处：「NOT_RUN_MARKER_FORM = re.compile(r"NOT_RUN_[A-Z][A-Z0-9_]*(?=\s|$)")」（`scripts/gate/lib_libtest_result_lines.py:104`）。变异门禁那一份还认行首空白：「NOT_RUN_LINE = re.compile(r"^\s*NOT_RUN_NO_GPU_ADAPTER test=(\S+)(?: reason=(.*))?$", re.M)」（`scripts/mutation/crates-mutation-rows.py:1399`）。
- 最小输入：任务 19471 那一条命令末尾加 `--test-threads 1`（libtest 单线程时先打 `test <名> ... `、跑完才打 `ok`，测试的 `println!` 落在同一行里）。推的结局：退 0，日志里那一行是 `test a_file_read_through_the_session_counts_the_damaged_copy_and_the_next_change_relocates_the_unit ... NOT_RUN_NO_GPU_ADAPTER test=… reason=…`，服务不报 `gpu-cases-not-run`、结局说明里没有本次未跑那一段；多线程带 `--nocapture` 时同样的事偶尔出现（门禁那一格的样本就是这一形）。命令在末节第 9 条。
- 四句：分辨臂——分辨；判别子观测得到——那一行就在日志里，只是不在行首；满足问 4「另找会让它漏认的日志形态」；跑前没给改法。改法：照 `lib_libtest_result_lines.py` 的认法认行内记号（行首空白、`test <名> ... ` 之后都认），与门禁同一份判法。

### 记号行本身的一个旁支

- 原因不经接成一行：「format!("{GPU_NOT_RUN_MARKER} test={test_name} reason={reason}")」（`crates/singlefs-verification/src/gpu_memory_ledger.rs:652`），而同一份文件里设备丢失那一行专门用 `text_on_one_line` 把换行去掉。原因里带换行（wgpu 的错误文字常带）时，记号行照样认得到、条数不错，只是点名里的原因截在第一行。不算打中。

## 问 5：复跑入口按这一台能给的内存取并行度、报 host-cap、样本仓取法只读

结论：推不出（条件打中一条 P1）；「别的派行格里同一种取法被盖」不成立（17 格逐格列在下面）。

### 读到那个值的链（文件:行）

`scripts/experiment-apparatus/replay.sh:5346`（读数脚本 `read` 的 `host-cap` 那一行进 `replay_host_cap`）→ `scripts/experiment-apparatus/replay.sh:5351`（交给 `replay_settle_fit_memory`）→「REPLAY_FIT_MEMORY="$host_cap"」（`scripts/experiment-apparatus/replay.sh:526`）→ 派行比它：「if [[ -n "$REPLAY_FIT_MEMORY" ]] && (( running_bytes + $1 + REPLAY_ROW_BASE_BYTES > REPLAY_FIT_MEMORY ))」（`scripts/experiment-apparatus/replay.sh:508`）。读数那一头：「host_cap, host_text = host_cap_reading(os.environ)」（`scripts/experiment-apparatus/replay_row_costs.py:153`）→「return task_memory.host_slice_cap_bytes(environment)」（`scripts/experiment-apparatus/replay_row_costs.py:143`）→ 多机配置的键「by_host_text = admission_module().configuration_value(configuration, SLICE_MEMORY_CAP_KEY)」（`scripts/hosts/task_memory.py:140`）→ 按这一台的地址取那一项「address = multi_host_addresses.resolve(configuration).this」（`scripts/hosts/task_memory.py:125`）。这个值按台不同，是设计上就有的。

### P1 两台上限不同，计时行「判了」与「本次不判」可以不同（推的）

- 入口自己的说法：「只定哪几行一起起、谁先起，行键、判定与标准输出的结果行都不变」（`scripts/experiment-apparatus/replay.sh:119`）。
- 可并行的行之间线程不累计：「每行的线程不累计，每行都用满登记的每台默认重活线程数」（`scripts/experiment-apparatus/replay.sh:127`）。所以一起跑的行数（`min(REPLAY_JOBS, 这一台能给的内存 ÷ 每行留量)`）直接定可并行那一批的负载。
- 计时行排在可并行的之后，开跑前等负载落下来，但至多等 90 秒：「至多等 REPLAY_TIMING_SETTLE_SECONDS 秒（默认 90）」（`scripts/experiment-apparatus/replay.sh:134`）；等的时候拿登记的线程数当核数：「cpus="$(heavy_thread_default)"」（`scripts/experiment-apparatus/replay.sh:5426`）。计时行自己的负载关拿装置收尾行里的 `loadavg1` 与在线 CPU 数比，阈值「TIMING_LOAD_RATIO="${REPLAY_TIMING_LOAD_RATIO:-0.5}"」（`scripts/experiment-apparatus/replay.sh:400`），超过的记本次不判；整趟一行都判不了退 77。
- 历史：同一张复跑表、同样都没复用（第一次跑或复用哨兵强制现判），在 `SLICE_MEMORY_CAP_BY_HOST` 一台大一台小的两台上各跑一次。大的那台一起起的行多，可并行那一批收尾时一分钟负载高；1 分钟负载按 e 的负一点五次方衰减 90 秒之后仍可能超过 0.5 × CPU 数，计时行记本次不判；小的那台行少，计时行判了。推的：没量过两台的负载曲线，阈值附近才会翻。这不改库里的判定（本次不判不记库），改的是这一趟的结局行与 `load_skipped=`，计时行全部本次不判时整趟退 77 而不是 0。
- 撞上限那一条我也看过：放得下比的是这一台能给的内存，故意比这一次拿到的上限压得松（自证格「dispatch-task-cap-not-binding  三行量过、各 1 GiB，服务交的上限 3 GiB、多机配置这一台那一项 64G：三行一起跑（比这一台能给的内存，不拿这一次拿到的上限压）」，`scripts/experiment-apparatus/replay.sh:617`），撞了由服务放大重排。某一行被内核杀掉时，入口会把它记成跑不了并撤绿（`scripts/experiment-apparatus/replay.sh:79`），可是任务的 scope 是「scope_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0 -p OOMPolicy=stop)」（`scripts/hosts/run-with-memory-cap.sh:1345`），systemd 一见杀进程就停整个 scope，入口多半来不及写库。这一条只是竞争窗口，不算打中。
- 四句：分辨臂——分辨（按这一台能给的内存派行是这一轮改的；以前比这一次拿到的上限，两台在服务给的同一档上限下并行度相同）；判别子观测得到——入口知道自己比的是哪一台的值，也可以在计时行之前把等待改成按真 CPU 数、等到落下为止；满足问 5「两台的内存上限不同时判定结局会不会不同」；跑前没给改法。

### 样本仓的取法被共用库盖掉：别的派行格里还有没有

只读函数挡的是：样本仓里钉成 16 的 `heavy_thread_default`，被 `read_gpu_switch` 经符号链接读进的真仓那一份盖掉（钉的那一行：`scripts/experiment-apparatus/replay.sh:1395`）。入口里调它的地方只有四处：`default_replay_jobs`（`scripts/experiment-apparatus/replay.sh:487`）、E175 驱动（第 4415 行）、等负载（第 5426 行）、没量过的行的线程数（第 5477 行，只在弄坏开关 `threads-bound-kept` 下用）。每格用不用得到：

| 格 | 设了 `REPLAY_JOBS` | 有没量过的行 | 被盖了会不会改它判的东西 |
|---|---|---|---|
| dispatch-memory-bound | 8 | 无 | 不会 |
| dispatch-task-cap-not-binding | 8 | 无 | 不会 |
| dispatch-threads-unbounded | 8 | 无（量过 31 线程） | 不会 |
| dispatch-row-count-bound | 2 | 无 | 不会 |
| dispatch-exclusive-rows | 8 | 有 | 会，已设只读 |
| dispatch-gpu-rows-serial | 8 | 无 | 不会 |
| dispatch-gpu-rows-without-cards | 8 | 有（没种峰值） | 开关下没量过的行照 31 算会独占，独占也排在要卡那一段之前，比的起止串不变：不会 |
| dispatch-driver-leased-rows | 8 | 视种的峰值 | 同上：不会 |
| dispatch-oversized-row-refused | 8 | 无 | 不会（开跑前就退 250） |
| row-progress-lines | 8 | 有 | 只比进度行，不比同时几行：不会 |
| row-peak-recorded-then-shared | 8 | 头一趟有 | 头一趟只数记录份数，第二趟都量过：不会 |
| row-failed-peak-not-recorded | 8 | 有 | 单行：不会 |
| stopped-keeps-finished-rows | 8 | 有 | 比结果行与停的行数：不会 |
| e181-goes-through-long-run | 8 | 有 | 单行：不会 |
| dispatch-item-selects-one-row | 8 | 有 | 单行：不会 |
| dispatch-item-unknown-refused | 沿用上一格的 8 | 有 | 一行都不起：不会 |
| dispatch-deferred-row-not-run | 8 | 有 | 比起了哪几行：不会 |

17 格（与文件头「十七格」同数，`awk` 现数）。等负载那一处在这些格里由「REPLAY_TIMING_SETTLE_SECONDS=0 REPLAY_TIMING_LOAD_RATIO=1000」（`scripts/experiment-apparatus/replay.sh:1300`）关掉。结论：同一种「取法被盖」只在已经设了只读的那一格；`dispatch-gpu-rows-without-cards`、`dispatch-driver-leased-rows` 两格在开关 `threads-bound-kept` 下的判断是读代码推的，没跑。

## 问 6：注释历史判法「以前」的四个例外

结论：推不出。打中两句（命中「面」这一个例外），一句较弱（命中「除」）。「缀」「者」两个例外我没造出读得通的讲过去的句子。

- 判法：「BEFORE_PATTERN = r"(?<!除)以前(?![缀者面])"」（`scripts/gate/comment-history-check.py:158`），文件头的说法：「「以前」前面紧挨「除」、或后面紧跟「缀」「者」「面」的不算」（`scripts/gate/comment-history-check.py:39`）。它想放过的是「所以前面」里「以」接「前面」；可是「面」这个例外不看「以前」前面是不是「所」，「以前」接「面临」「面向」「面对」的都被放过。
- 被放过的句子（逐字，写在草稿目录 `q6/root/scripts/probe.sh` 的注释里）：
  1. 「这一段以前面临内存撞上限，所以按块读。」——讲过去（以前遇到过什么），命中「后面紧跟面」那个例外；这一行别的词也不在 `WORD_PATTERNS` 里。
  2. 「以前面向旧布局的那一支已删掉，现在只认新布局。」——讲过去与改动经过，命中「面」。
  3. 「除以前登记的三行之外，新写的不许加。」——「以前登记的」指过去那一次登记，命中「前面紧挨除」；这句同时在说现行规则，算弱。
  对照：「以前这里只认一种写法。」照判。
- 怎么核：末节第 8 条在探针小树上跑 `--list`。预期只列出对照那一行（第 6 行），第 2、3、4 行不列。
- 四句：分辨臂——分辨（四个例外是这一轮加的）；判别子观测得到——前一个字就在文本里（「所」）；满足问 6「会不会放过真讲过去的句子」；跑前没给改法。我提的改法（零轮）：「面」的例外只在前面紧挨「所」时成立，例如 `(?<!除)(?<!所)以前(?![缀者])` 再另放「所以前面」，推的，待主 agent 跑样本档。
- 超出四个例外的一处对不上（只记、不算这一问的打中）：共用约束把「后来改成」列为要判的历史写法：「改动经过（「此前」「原先」「起初」「最初」「后来改成」「改成了」「换成了」「修过」）」（`.claude/agent-common.md:42`），紧接着说门禁全仓判；判法的词表里没有它（`scripts/gate/comment-history-check.py:163`），文件头另写了「后来」靠 review。探针第 5 行「这张表后来改成按块读。」预期同样不列。

## 问 9：自证与弄坏开关

结论：推不出。三条打中（W1、W2、W3）。条数这一层对得上：我照 `.claude/gate.d/code-tooling.sh:1234-1267` 的 `switch_entries` 读法写了一份只读的数数脚本（`tools/count_switch_entries.py`，草稿目录里，量的），这一轮动过的登记行逐份现数与登记表行内写的数相同：写闸 2、`replay.sh` 60、`lib_project_code_runs.py` 72、注释历史判法 18、规格工具 5、E56 入口 12、`task_server_lib.py` 477、检出钩子 108、项目代码闸 142；换上工具 22 条不在表里（定义说它登不进）。

### 这一轮新加的会判红的分支，逐个对到开关与点名的格

| 分支 | 开关（文件:行） | 点名的格 | 跑过的证据 | 判 |
|---|---|---|---|---|
| 写闸第十一道的 `src/` 一支与它的出路 | `.claude/hooks/write-guard.sh:208` script-in-place-allowed | 第 215、216 行三格 | 调度记录 242 行「写闸开关复跑 19717」逐格相同 2 条 | 对上 |
| 换上工具退 8 | `scripts/gate/hook-swap.py:159` neutral-base-skipped | 第 160 行两格 | 调度记录 99 行任务 19093 | 对上（那一版） |
| 换上工具退 9 与认机器 | `scripts/gate/hook-swap.py:155`、`:157` | 第 156、158 行 | 调度记录 151 行，红 1、2 与期望同 | 对上（那一版） |
| 按键认库 | `scripts/gate/hook-swap.py:141` subscript-key-ignored | 第 142、143 行三格 | 任务 19594 日志（我读的）三格 | 对上 |
| 结局文件读法 | `scripts/gate/hook-swap.py:161` outcome-reader-trusts-file | 第 162、163 行四格 | 调度记录 154 行复跑 19251 红 4、点名 4 | 对上 |
| 规格工具五种拒绝 | `scripts/gate/make-swap-spec.py:33-42` 五条 | 每条一格 | 调度记录 250 行任务 19727 | 对上；另见 W1 |
| 本次未跑提醒 | `task-server/task_server_lib.py:1507` gpu-cases-not-run-unread | 第 1508 行一格 | 调度记录 160 行任务 19234 红 1 格 | 对上 |
| 注释历史的四个例外 | `scripts/gate/comment-history-check.py:73` prefix-compounds-flagged | 两份样本与一格自证 | 调度记录 121 行任务 19157、19160 | 对上 |
| 复跑入口按这一台的内存派行与 host-cap | `scripts/experiment-apparatus/replay.sh:716`、`:718`、`:720`、`:722` | 各自下一行 | 调度记录 257 行任务 19775，60 条逐格相同 | 对上 |
| perl 单独写的选项、查活动会话工具登名单 | `.claude/hooks/project-code-guard.sh:506`、`:508` | 第 507、509、510 行 | 没有：见 W2 | 对不上未核 |

### W1 规格工具新写的几种拒绝没有开关、也没有自证格

规格工具是这一轮新建的。它的自证只有六格（`grep -c 'def cell_'` 现数 6），开关五条；下面这几种退 2、退 3 的分支，自证格与开关都没有点到：

- 同一份点两次、绝对路径或带 `..`：「if os.path.isabs(written) or relative == ".." or relative.startswith(".." + os.sep) or relative == ".":」（`scripts/gate/make-swap-spec.py:393`）、「if len(set(normalized)) != len(normalized):」（`scripts/gate/make-swap-spec.py:395`）；
- `--max-context` 不是正整数（`scripts/gate/make-swap-spec.py:389`）；
- 文件不是 UTF-8（`scripts/gate/make-swap-spec.py:89`）；
- `--out` 是目录或所在目录不在：「if os.path.isdir(absolute) or not os.path.isdir(parent):」（`scripts/gate/make-swap-spec.py:164`）；
- 取不到工作根（`scripts/gate/make-swap-spec.py:408`）。

把这几支的条件改成恒假，现有六格照样全绿（推的，读自证六格的输入推出：没有一格喂这几种输入）。四句：分辨臂——分辨；判别子观测得到；满足问 9「每一处新加的会判红的分支，有没有开关或样本」；跑前没给改法。

### W2 项目代码闸的两条新开关点名的格自标「推的，没跑」，登记表没记它们没复跑

- 文件头两条都写着「红的格（推的，没跑）」（`.claude/hooks/project-code-guard.sh:506`、`:508`）；调度记录：「这一轮给它换上的两条新开关（`active-sessions-unlisted`、`perl-bare-options-take-next-word`）与补点名的四条算没判」（`records/2026-11-17-subagent账本清欠调度记录.md:110`）。
- 登记表那一行的理由列只写了「新加的 copy-without-full-gate-flag 与它牵到的格还没复跑」（`.claude/gate.d/break-switch-replay-files.tsv:74`），这两条与补点名的四条没写；表里 `grep -c` 这两个开关名现数 0。条数 142 对得上，「还没复跑」的清单对不上。
- 另：库那一侧给 `perl-bare-options-take-next-word` 写的选项是「-i、-l、-0、-x、-C、-d、-D」（`.claude/hooks/lib_shell_words.py:275`），钩子这一侧点名的三格只有 `-i`、`-l`、`-0`；四种没有格，开关下红不红没人量过。

### W3 最脆弱的一处：换上工具的 22 条开关没有复跑兜底

- 定义：「键算不出的脚本（例 `scripts/gate/hook-swap.py`）登不进那张表，它的开关照旧逐条写 `env <开关名>=<项> … --selftest`」（`.claude/agents/tooling-writer.md:35`）。所以每改一次换上工具，22 条开关要人记得逐条复跑；这一轮最后一次换上（按键认库那一版、规格 31 处）之后只复跑了三条（19594、19595、19596），退 8、退 9、结局文件那几条最后一次复跑在那之前的版本上。
- 它又是全仓脚本与钩子唯一的换上路径，第 2 节的 R1（没锁、结局文件互盖）也在它身上。两样叠在一起：它坏了，挡不住的是所有经它换上的脚本与钩子。
- 写闸那一行同类：表里写着「文件里别的开关写在「# … # 名字=值」两层井号的行里，这一格读不到」（`.claude/gate.d/break-switch-replay-files.tsv:61`），写闸文件头前 216 行里不同的 `*_BREAK=<项>` 写法 27 种、`WRITE_GUARD_DISABLE_*=1` 7 种（`grep -oE … | sort -u | wc -l` 现数），开关复跑格只读得到两条（这一轮新加的那条在里面，不算这一轮的错）。

### 表里「验证」写「没有」的几份

- 定义与共用约束四份：要走的是定义三方（这一轮就是），不算漏验。
- `scripts/gate/README.md:21` 与 `scripts/experiment-apparatus/README.md:11-12`：两份都在注释历史判法的射程里（`scripts/` 下的 `README.md`），我没找到在这一次改动之后跑过的注释历史那一格的记录（调度记录 250 行的 19726 早于还是晚于这两处改动，记录里看不出）；另外 `scripts/gate/README.md:21` 写「只写工作根下的 `--out`」，这一句被问 3 的 S1 推翻。
- `.claude/gate.d/stage-owners.tsv:139`：只改了第四列的说明；该跑的是门禁里核这张表写法的那一格，没记录，推的不会红。
- `scripts/kb/stale-candidates-record-carriers.tsv:3`：`scripts/kb/stale-candidates.py` 的 `--facts` 要核登记的路径在结束那一版或工作区里有；新账本已跟踪（`git ls-files` 现查在），推的不会退 2。换下的封存账本从此不进候选表，它里面还开着的欠账有没有都挪进现行账本，归正推腿判。

## 没打中的形状

- 问 1 误拒：试过新建文件（Write 不存在的路径，判法 `stat` 不到就放）、草稿目录与仓副本（落点在主工作区之外就放）、经符号链接从草稿目录指回主工作区 `src/`（解开链接照拒，是对的）、Claude Code 建在仓里的工作树（相对路径不以那几个目录起头，放行，属副本）、`scripts/`、`task-server/` 下带执行位而无后缀的文件（`git ls-files` 加 `-x` 现列只有 `task-server/control`、`task-server/task` 两份，都是该拒的）、`scripts/` 下别的样本目录（现列 0 个）。没找到该放而拒的。
- 问 2：退 9 在另一台起的那一形（机器标识对不上退 7、拒绝说明先叫看那一趟在不在跑），跨台的判法与说明自洽，任务服务的 `workspace-write-blocked` 提醒也点名上一趟的任务号，不算打中。按键认库的另几种写法：`sys.modules.setdefault(键, 库)` 存（交给认不出的调用，列成本次未判，对的）、键是循环变量（折不出，列成本次未判，对的）、同一个键先后存两份库（对上不止一份，列成本次未判，对的）、`sys.modules.get(键)` 取（不是下标，既不跟也不列，与 K1 同类；仓里扫描目录下现数 2 份文件有这种写法，没逐份核它们取的是不是要核的库，只记形状）。
- 问 3：`--out` 本身是指到仓里的符号链接（`os.replace` 换掉的是链接本身，写不进去）、`--out` 所在目录是指到仓里的链接（父目录先解开再比，拒），两种都挡住了。
- 问 4 误认：libtest 收尾时打的失败用例输出段（`---- <名> stdout ----` 之后记号在行首）会被认，可那条用例已经红了，说它本次未跑不误导；自证里假测试打的记号（`scripts/mutation/lib_broad_mutation_selftest_fixtures.py:54`）要是透到任务日志里会被误报，我没核那条自证有没有把子进程输出收起来，只记形状。
- 问 5：两台读到的配置项不同只改并行度这一点本身不算错；撞上限那一路见第 5 节，算竞争窗口不算打中。
- 问 6：「缀」「者」两个例外，试过「以前缀」「以前者」开头的讲过去的读法，造不出通顺的中文句子。
- 问 9：登记表条数与文件头开关数，逐份数过都相等（第 9 节开头）。

## 这条腿自己的限度

- 我没跑项目代码：钩子、自证、开关、探针全交主 agent，报告里的结局除了读日志（19591、19594、19595、19596）与读 `task status`（19471）之外都是推的。规格工具那一条沙箱演示我自己起过一次，被会话的权限分类器拒了，没跑成。
- R1（换上工具并发互盖）没造出确定性的探针，要两条任务卡准时刻；P1（两台上限不同改计时行的判）没量两台的负载，阈值附近才会翻。
- 问 2 的 K1 只扫了 `sys.modules[...]` 下标那一种取法里的两处；`sys.modules.get` 那一种没逐份核。
- 我读到的范围以这五问为主，正推腿那一半（每处改动是不是账本那一节说的、第 7、8、10 问）没做。

## 要主 agent 跑的命令

草稿目录（下称 `<草>`）是 `/home/fy5090/.local/share/singlefs/work/governance-r1-attack`；下面命令里都写全路径。喂法与探针只判、只在草稿目录的小树上跑，不改主工作区。每条看它自己的 `TASK_SERVER_RESULT` 与 `task-server/task log <号>`。

1. 喂检出钩子（问 1 L1、问 3 S1）。在仓根起。
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/bash-command-detector.sh --feeds /home/fy5090/.local/share/singlefs/work/governance-r1-attack/feeds/detector.json`
   看：日志里 7 行 `✓`/`✗`。喂法里写的 exit 是我预期钩子的结局：前三行（`Q1-L1`）与第六行（`Q3`）是 `✓` 就坐实漏放；是 `✗ 退 2` 就推翻那一条。两条对照（`cp` 盖 `prune-server/control`、`sed -i`）要是 `✓`（退 2），否则这次喂法本身不算数。全 `✓` 时 `exit=0`。
2. 喂项目代码闸（问 3 S1）。在仓根起。
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/project-code-guard.sh --feeds /home/fy5090/.local/share/singlefs/work/governance-r1-attack/feeds/project-code-guard.json`
   看：第一行 `✓ 退 0` 坐实（名单放行带 `SINGLEFS_WORK_ROOT=` 的那一条）；对照那一行要 `✓ 退 2`。
3. 喂写闸（问 1 L2）。在仓根起。
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --feed-only --hook .claude/hooks/write-guard.sh --feeds /home/fy5090/.local/share/singlefs/work/governance-r1-attack/feeds/write-guard.json`
   看：前三行 `✓ 退 0` 坐实（`Cargo.toml` 与 `tests/` 下的 `.rs` 放行）；对照 `prune-server/src/protocol.rs` 那一行要 `✓ 退 2`。
4. 按键认库的探针（问 2 K1）。在仓根起。
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q2/root`
   看：`exit=0`、末行「✓ 只核不换：…缺 0 处，本次未判 0 处」就坐实（库里没有调用方引的那个名字，照判过）；要是点名了 `caller_by_key.py` 或列了本次未判，K1 推翻。
5. 同一探针的对照。在仓根起。
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q2/control`
   看：`exit=1`、`✗` 行点名 `scripts/gate/caller_by_attribute.py` 引了 `project_preflight.shared_preflight_directory`。这一条不红，第 4 条的读数不算数。
6. 规格工具认工作根的沙箱（问 3 S1，只写草稿目录）。在仓根起。
   `task-server/task submit -- env SINGLEFS_WORK_ROOT=/home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/fakework python3 scripts/gate/make-swap-spec.py --base /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/base --new /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/new --out /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/fakework/out/settings.json x.txt`
   看：`exit=0`、日志「✓ 写出 …/q3/fakework/out/settings.json：1 份文件 1 处」。
7. 同一沙箱的对照。在仓根起。
   `task-server/task submit -- env SINGLEFS_WORK_ROOT=/home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/otherroot python3 scripts/gate/make-swap-spec.py --base /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/base --new /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/new --out /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q3/fakework/out/settings-control.json x.txt`
   看：`exit=3`、日志「不在工作根下」。第 6 条 0、第 7 条 3，就说明 `--out` 能不能写只看命令前的环境变量。
8. 注释历史的例外（问 6）。在仓根起。
   `task-server/task submit -- python3 scripts/gate/comment-history-check.py --list /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q6/root`
   看：只列出 `scripts/probe.sh:6`（对照句）就坐实第 2、3、4 行被放过、第 5 行（「后来改成」）也没判；第 2、3 行被列出则那一句推翻。
9. 单线程时记号接在结果行起头后面（问 4 M1）。在仓根起；会不会走崩溃枚举以范围声明闸为准，被拒照拒绝说明改。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired 里的 1 条测试; 范围=1 个测试; 为什么=核任务服务认不认单线程时接在 libtest 结果行起头后面的本次未跑记号，全域理由：与 19471 同一条用例同一个域，只换 libtest 的线程数' task-server/task submit -- cargo test -p singlefs-verification --test unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired -- a_file_read_through_the_session_counts_the_damaged_copy_and_the_next_change_relocates_the_unit --exact --nocapture --test-threads 1`
   看：`exit=0`；`task-server/task status <号>` 里没有「这一趟有 1 条用例本次未跑」，而 `task-server/task log <号> --all` 里有一行以 `test a_file_read_through_` 起头、同一行里带 `NOT_RUN_NO_GPU_ADAPTER`，就坐实。status 里有那一段则推翻。
10. 换上工具退 8 那条开关在现版上复跑（问 9 W3）。在仓根起。
    `task-server/task submit -- env HOOK_SWAP_BREAK=neutral-base-skipped python3 scripts/gate/hook-swap.py --selftest`
    看：`exit=1`，`✗ hook-swap 自检：` 起头的行按格名去重，等于文件头第 160 行点名的两格。
11. 退 9 那条。在仓根起。
    `task-server/task submit -- env HOOK_SWAP_BREAK=running-swap-not-checked python3 scripts/gate/hook-swap.py --selftest`
    看：同上，对第 156 行。
12. 认机器那条。在仓根起。
    `task-server/task submit -- env HOOK_SWAP_BREAK=swap-process-pid-only python3 scripts/gate/hook-swap.py --selftest`
    看：同上，对第 158 行。
13. 项目代码闸的 perl 那条开关（问 9 W2）。在仓根起。
    `task-server/task submit -- env LIB_SHELL_WORDS_BREAK=perl-bare-options-take-next-word bash .claude/hooks/project-code-guard.sh --selftest`
    看：`exit` 非 0，红的格等于 `.claude/hooks/project-code-guard.sh:507` 点名的三格。
14. 项目代码闸的查活动会话那条开关。在仓根起。
    `task-server/task submit -- env LIB_PROJECT_CODE_RUNS_BREAK=active-sessions-unlisted bash .claude/hooks/project-code-guard.sh --selftest`
    看：红的格等于第 509、510 行点名的那几格。
15. 只用 `from … import` 引的调用方（问 2 K3）。在仓根起。
    `task-server/task submit -- python3 scripts/gate/hook-swap.py --check-callers .claude/scripts/project_preflight.py --root /home/fy5090/.local/share/singlefs/work/governance-r1-attack/q2/fromimport`
    看：`exit=0`、末行「缺 0 处」就坐实（调用方引的名字库里没有，照判过）；`exit=1` 点名 `caller_by_from_import.py` 则推翻。

## 只读到没攻到的

- `.claude/gate.d/crash-point-replay.sh:314-318` 的注释（第 10 问，归正推腿）。
- 账本指针那一批：`.claude/hooks/handback-guard.sh:43`、`.claude/hooks/lib_run_scope.py:714`、`.claude/hooks/run-scope-guard.sh:31`、`.claude/hooks/runner-dispatch-guard.sh:183`、`scripts/agents/dispatch-prompt.py:300`、`.claude/gate.d/stage-owners.tsv:139`（第 8 问，归正推腿）。
- 定义与共用约束：`.claude/agent-common.md` 整读过，`.claude/agents/tooling-writer.md` 整读过；`.claude/agents/implementation-writer.md:32`、`.claude/main-agent.md:99` 没读（第 7 问，归正推腿）。
- 实验装置的注释与测试：`scripts/experiment-apparatus/e184-segment-one-long-run.sh:534`、`list_check.py`、`e185_r4_baseline_rejudge.py`、`e203-admission-scan-cost-run.sh`、`long-run-lib/binary-identity.sh`、E201 的三份 `.py` 与变异表、`replay-comparison-units.tsv`、`comment-history-lag.tsv`、`doc-process-records.sh:457-458`、`scripts/agents/agent-watch.py:591`：没读到攻的程度。
- `e56-vm-bench-run.sh`：只核了开关条数（12）与登记表相同，没攻它的判法。
- `task-server/task_server_selftest.py` 的新格与 `task_server.py:6025` 的记账：只读了认记号那一段（第 5535–5599 行一带与第 1116 行）。

## 只有这条腿读了的文件（不在 52 份里）

`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/selftest-temporary-service.tsv`、`.claude/hooks/lib_work_root.py`（头部）、`.claude/hooks/lib_shell_words.py:275`、`.claude/gate.d/code-tooling.sh:1234-1267`、`scripts/gate/apply-writer-patch.py`（文件头）、`scripts/gate/lib_function_units.py:18351`、`scripts/experiment-apparatus/e165-pin-red-examples.py:1516`、`scripts/gate/lib_libtest_result_lines.py`（自证样本那一段）、`scripts/mutation/crates-mutation-rows.py:1399`、`crates/singlefs-verification/src/gpu_memory_ledger.rs`（记号那一段）、`crates/singlefs-verification/src/gpu_unit_checks.rs:1906`、`scripts/hosts/task_memory.py`（这一台能给的内存那一段）、`scripts/hosts/run-with-memory-cap.sh`（scope 属性那一行）、`scripts/hosts/lib_repository_copy.py`（文件头）、任务日志 19591、19594、19595、19596 与任务 19471 的 status。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，钩子、自证、开关、探针一条没起，命令都在上一节；判决要引的数没复跑。
- 没写模型、没起崩溃状态枚举（这一轮是治理文件，没有要枚举的写流）。
- 草稿目录里没建仓副本与编译目录；留下的是喂法、探针小树、读日志的落盘与一份只读的数数脚本，主 agent 跑第 1–8 条要用，不删。
