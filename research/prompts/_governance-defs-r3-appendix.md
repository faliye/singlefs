**出处 `.claude/main-agent.md:50-64`（整段抄，未转述）**

```markdown
## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派；不派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，派 `crash-verifier` 跑 QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：层 0 全量主 agent 跑，`crash-verifier` 跑 QEMU、herd7、crates 变异表 |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键），主 agent 自己在后台跑 `.claude/gate.d/54-layer0-replay.sh` 里 `print_staged_worktree_full_commands` 打印的那三行命令，三行放进同一次 Bash 调用（它们共用一个 shell 变量；建 HEAD + 暂存区的 worktree、带 `SINGLEFS_HEAVY_TESTS=commit` 经内存包装跑那棵树里的 `--full`、删 worktree；命令只在那一处写，不在这里抄）（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯；那次调用退 0（退出码是经内存包装的那条 `--full` 命令的，250–254 是包装自己的结局）、全绿标记写好之后才往下，不然整轮门禁里的 54 号必红 → 派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定，判据在 `research/scripts/stage-must-run.sh` 文件头；要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑；用户要求时各处都换成 `=user-request` |
| 改了一个数或格式常量、撤回一条结论、新立一条判据 | `sweep`（四种活与各自要给的输入见它的定义） |
| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ 红在 `research/mutations/` 的表：`experiment-runner` 只修锚点；红在 `crates/mutations.tsv`（`relabel-item.py` 改写了 `crates/` 下的源码）：派 `implementation-writer` 修锚点，走代码轮 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 要别家文件系统的事实 | `prior-art` |

```

**出处 `.claude/agent-common.md:40-58`（整段抄，未转述）**

```markdown
## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 重型测试（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准，逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 cargo test，按轻阶段对待）。子 agent 跑 `cargo test` / `cargo run` / `cargo bench` 与直接执行编出来的二进制，一律经内存包装：`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限照 systemd 写法，派发提示没给的用 `4G`；退出码 250 是撞了这一条的上限，照实报主 agent，不自己调大重跑；251（scope 起不来）与 252（内存不够排不上）是那条命令一行都没跑，照实报、不绕开包装去裸跑；253 是超过限时（`RUN_WITH_MEMORY_CAP_TIME_LIMIT`），照实报；254 是被总上限挤掉、结果不算数，重跑一次，再挤掉就照实报；2 是包装的用法写错，改写法再跑；退出码表以 `research/scripts/run-with-memory-cap.sh` 文件头为准），不经它的由 `heavy-test-guard.sh` 拒。重型测试里 `crash-verifier` 只跑 55、57、59 号那几道（54 号快档在 `gate.sh --staged` 里，全量由主 agent 跑），`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
- 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
- **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
- 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；几条活要在一条命令里并行，每个作业把退出码写进自己的文件、用不带参数的 `wait` 等齐、再逐个读那几个文件：`{ 甲; echo $? > <草稿>/甲.rc; } & { 乙; echo $? > <草稿>/乙.rc; } & wait; cat <草稿>/甲.rc <草稿>/乙.rc`（不带参数的 `wait` 自己恒返回 0，退出码只认文件，`.claude/singlefs-ai-sop/rules/command-safety.md`「并行不许把失败吃掉」；起了不止一个 `&` 时逐个 `wait "$pid"` 会被检出 hook 记成没等齐）。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
  等长活时不起缓存计时器，结束本轮直接等完成通知。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的 `progress.md`，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对七种写法在执行前拒绝（逐条的判据写在它的文件头「拒绝七种」一节）：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）、终止进程时不是一次点名一个自己起的进程号或任务号（按名字挑、按进程组或 cgroup 批量、在循环里逐个发信号、停 ssh 与会话相关的服务，都拒；要停就照「要停自己起的一条链」那一条逐个 `proc.py stop`）、在同一个 inode 上改一个已存在的脚本（`>`、不带 `-a` 的 `tee`、`cp` 覆盖、python 里 `open(…, "w")` 改 `.sh` / `.py`；改用 `research/scripts/replace-once.py`，或写临时文件再 `mv` 换上）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。

```

**出处 `.claude/agents/crash-verifier.md:22-30`（整段抄，未转述）**

```markdown
## 做什么

1. 每个阶段开跑前各照共用约束「不做」一节看一次负载。54 号不归你：快档在 `gate.sh --staged` 的 HEAD + 暂存区树上跑才算得对全绿标记的键（在主工作区跑，工作区与暂存区不同就判红），全量由主 agent 在 worktree 里跑。55、57、59 在主工作区跑，判的是工作区那一份：每个阶段开跑前取那一道自己的输入：55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`；跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- crates litmus .lkmm-static-only`（cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层，要没有输出；别处的未跟踪文件不挡）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（第一个事务的干跑） 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

```

**出处 `.claude/agents/crash-verifier.md:35-38`（整段抄，未转述）**

```markdown
## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行 / 日志路径；末尾「没做什么」。每个阶段的整份输出都写进 `<草稿目录>/<阶段>.log`（前台跑也写），59 号那一份是变异分诊员要的输入。

```

**出处 `.claude/agents/experiment-runner.md:24-36`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
2. 写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`；`research/e7-index-bench` 的 `[[bin]]` 的 `name` 全写连字符 `e<号>-<英文名里的下划线换成连字符>`，入库装置不写 `[[bin]]`、bin 名就是文件名 `e<号>_<英文名>`；中文简称只用在 kb 页），在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<英文名>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`research/e7-index-bench` 的 `name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
3. 入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」。`research/` 那一支写变异表 `research/mutations/e<号>_<英文名>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下（入库装置那一支编在仓根的 `target/` 下），用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
5. 在 `research/scripts/replay.sh` 里登记复跑（`research/` 那一支写一行登记；入库装置照 E156、E158 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 `E<号>|@driver_e<号>||<产物>|exact`），跑一次 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`（`replay.sh` 里执行编出来的二进制，不经内存包装会被 `heavy-test-guard.sh` 拒） 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152（按里程碑对比六家文件系统的文件性能） 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。
6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增；清单里一条都没有时 ② 不适用，① ③ 照做）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。

```

**出处 `.claude/agents/kb-scribe.md:16-25`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。
- 要不要记决策变更史：标题整行、日期、改前、改后、依据各一句（快查两行由主 agent 给定，不由你概括；标题里「（其N）」那一段由你在写之前照当月文件现取，其余逐字照给）。
- 分项翻状态的：决策号与分项号；另给这几样，缺了门禁必红而你不能自己补：分项那一行收尾的「**状态：已定。**」或「**状态：未定。**」（20 号），翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），N 写阿拉伯数字或一到十的汉字数字，20、75 号都认）；`.claude/kb/decisions.md` 状态列由第 3 步的 `21-decision-items-sync.sh --write` 重生成，规格只给预期的分项计数供回读核对，不手写。变更史标题与快查里的分项标签写翻完之后的：变更史用今天的编号（`.claude/kb/decisions-history.md` 第 8 行），写成翻之前的，第 3 步的 `relabel-item.py` 也会把它改掉。
- 欠账表要加或还的行：也写成逐条规格（加：新行与插在哪一整行之后；还：被还的那一行，与移进已还清表的新行、插在哪一整行之后）。
- 写决策正文（新立分项、或改一条分项的定案 / 射程 / 依据）的：规格要按 `.claude/rules/format-evolution.md` 那一节的形态逐块给全（四块、索引行、未定项的字节判决），形态本身照那一节，这份定义不抄第二份；缺哪一块就停下报告，不自己补。
- **规格改了某条分项的「**依据**」段时，同一份规格还要带上那个实验页 `### 影响的决策` 表里对应那几行的改动**（旧串、新串，一条分项一行）：门禁 75 号那条双向检查两侧要同时到位，而判关系是主 agent 的活、不是你的。规格里没带那几行就停下报告，列出缺哪几行。
- 报告路径与草稿目录，都在 `/tmp/claude-1000/` 下（写范围闸对你只放行 `.claude/kb/` 与那里）。

```

**出处 `.claude/agents/kb-scribe.md:26-35`（整段抄，未转述）**

```markdown
## 做什么

1. 开工时先记下规格点名的文件、当月变更史文件、`.claude/kb/decisions.md`、`.claude/kb/decisions-history.md` 的 `sha256sum`（这是开工时的，不是派发前的；文件带别的会话没提交的改动时照记）。把规格写成 `research/scripts/replace-batch.py` 的规格文件（放草稿目录）；规格里每个文件路径先喂写范围闸（`printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' | AGENT_HOOK_DETECTIONS=<草稿目录>/precheck.jsonl bash .claude/hooks/write-guard.sh`；检出记录指到草稿，预检被拒不会叫醒主 agent 的看门狗），有一条退出码 2 就整份不写、停下报告；再 `--dry-run` 核全部恰好命中一次，然后不带 `--dry-run` 实写并回读。用它，不用逐条 Edit。
2. 决策变更史：原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格；新条目放在哪、同一天多条怎么编「（其N）」，照当月文件与 `.claude/kb/decisions-history.md` 的文件头，不另定。然后先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：它列的「快查·… 还没写」里有不是这一轮的条目，就停在 `--write` 之前交回，不让 `--write` 往那些条目里写「（待补）」；都是这一轮的才跑 `--write`。跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次，只核新增的行：每条决策的卡片只留最近 3 次，被挤掉的旧行是按设计；新增的行里有这一轮之外的条目就停下交回，不自己收拾。
3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；改写了 `crates/**/*.rs` 的（实现的地盘，要走代码轮），同样逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。
3b. 规格里决策那几处与实验页那几行是一批，一起写、一起回读，不分两次。写完跑 75 号：它报「不对称」时看点名的那一对在不在这一份规格里——在，就是规格少给了一边，停下报告并写明缺哪一行；不在，按「不是这一轮的不修」照写。**判一个实验撑不撑一条分项不在你的活里**，规格没写的关系一个字不加。
每次写入之后，写入后钩子 `.claude/hooks/kb-scribe-followup.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你：红的是这个流程下一步会消掉的（21 号在第 3 步的 `21-decision-items-sync.sh --write` 之前、30 号在变更史条目写之前、49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。
4. 跑阶段归属表登记给你的阶段各一次并贴结果（共用约束 `.claude/agent-common.md`「门禁」一节）；阶段数用那条 `awk` 接 `| wc -l` 数出来贴原样，不手数。红的判它是不是这一轮写出来的（看点名的文件与行在不在这一轮改动里）；不是这一轮的不修，照写。30 号报的改动行数含别的会话没提交的改动，照抄，不据此判归属。另跑一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 贴末行；红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子。
5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。

```

**出处 `.claude/agents/mutation-triage.md:22-30`（整段抄，未转述）**

```markdown
## 做什么

1. 开跑前照共用约束「不做」一节看负载。
2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的与测试进程被杀的在「无效」一栏（第 5 步那一句分得开两者），「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」，两种都让整道判红；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列），其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`mutate.sh` 的 `💥` 不判失败，逐条列出交主 agent；与改动前的数比，「无效」变多要单列。
6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。

```

**出处 `.claude/agents/three-way-verifier.md:16-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 轮名、这一轮全部腿报告的路径（主 agent 确认都已交齐、腿不再写）。
- 背景材料路径（用来识别误写成背景材料行号的引用）。
- 云端腿交回里给的报告 `sha256sum`（有就给）。
- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时，拿腿开工时刻照第 2 步现查那个文件在腿开工之后有没有被改过：没改过记 ✗，改过或没给开工时刻记「分不清：文件可能在腿开工之后被改过」（第 6 步的「分不清」一栏）。
- 腿开工时刻（UTC）：第 2 步现查「快照清单里没有的文件」在腿开工之后有没有被改过要用；没给时，快照清单外的文件内容对不上记「分不清：文件可能在腿开工之后被改过」，不记 ✗。
- 报告路径（形态 `research/prompts/<轮>-verifier-output.md`）、草稿目录。

```

**出处 `.claude/agents/three-way-verifier.md:25-33`（整段抄，未转述）**

```markdown
## 做什么

1. 判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它（挑一条文件在快照里、或腿开工之后没被改过的），必须判 ✗；判不出就停下报告「核查方法不分辨」。
2. 每处「文件:行号 + 抄的原文」：先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件可能在腿开工之后被改过」，不记 ✗；给了快照、而这个文件不在清单里的，对主树核，并用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查它在腿开工之后有没有被改过，照实写，不一律记成「被改过」。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。
3. 每行引的产物：在产物文件里逐字找。
4. 每条复跑命令：把腿的模型目录拷到草稿目录，在副本里跑（加 `nice -n 19`），比输出与报告里抄的、比 sha256；不在腿的原目录里跑。复跑命令带着指向路径的环境变量的，路径一并换到你的草稿目录；输出里嵌着临时路径的，按字段比，不按整份哈希判 ✗。
5. 本地腿的转述核对表里每一处「原文文件:行」也逐条核：行号在不在、抄的是不是原文、英文有没有丢限定词，也有没有多加原文没有的限定词或括注。
6. 核不动的（要编译、要虚机、要网络）写「核不动」与原因；「核不动」与「分不清」（第 2 步与输入一节那几种，连同原因）各单列一栏，不算进 ✓ 也不算进 ✗；报告文件现在的 sha256 与交回里给的对不上，整份记「分不清：报告在腿交回之后被改过」。

```

**出处 `.claude/rules/implementation-workflow.md:24-29`（整段抄，未转述）**

```markdown
## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），连同派腿的时刻（`date -u`）交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、派核查员之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再交核查员——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、几点、被改了几处、腿引的行落没落在那几处。

```

**出处 `.claude/rules/mutation-sampling.md:78-84`（整段抄，未转述）**

```markdown
## 改了一个格式常量之后，要看「无效」那一栏有没有变多

变异表里一条本来好好的条目，会因为**别处改了一个常量**而从「被抓」变成「无效」，而没有任何东西报警。

⇒ **改完格式常量，重跑受影响的每张变异表，比对「抓到 / 无效 / 没红」三个数**，不能只看「没红」是不是 0（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`💥` 不判失败，照样单列、逐条交出去；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；门禁 59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」，两种都让整道判红；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列；`crates/mutations.tsv` 整表复跑是重型，由提交时的崩溃验证员跑 59 号）：
无效那一栏变多，等于有断言被关掉了，而它与「这一条本来就抓不到」在输出里长得一模一样。

```

**出处 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:176-190`（整段抄，未转述）**

```markdown
### 判据自己也会写错：打中之后先判是哪一种

跑前写死的判据挡住了「看完结果再改判据」，但它自己也可能写错，而且往往在打中的那一刻才露出来。四种形态：

| 形态 | 长什么样 | 怎么办 |
|---|---|---|
| **打中不分辨臂** | 一格打中让每一条臂一起出局，病根在各条臂共用的前提里 | 不拿它判臂：把那个共用前提另立一笔账先修，判臂只看分辨得出臂的那几格。写「全部出局 ⇒ 回落到某一条臂」这种条款时，同时写一格让所有臂一起出局时怎么记 |
| **判别子观测不到** | 判据要被判的系统区分两种情形，而那个系统在做决定的那一刻看得到的东西，两种情形逐条相同 | 要求这两种情形判得不同的两条判据互相矛盾，任何做法都只能满足其中一条。改判据，改完是新一轮 |
| **打中归错了判据** | 一条腿报「判据 X 打中」，而发生的事按字面属于门槛不同的另一条判据 | 逐字核打中满足的是哪一条判据的哪一个分句；归错会让一条臂在错的门槛下出局 |
| **改法碰不到打中的格** | 跑前的反向接受条款写「打中 ⇒ 在改法 A 与改法 B 之间选」，而其中一个改法在打中的那几格上照样中 | 写条款时每个改法都写明它修的是哪一格；打中之后先在打中的格上逐个核改法还中不中，还中的那个交出去时写明「对这几格不起作用」——不写，选它等于把误判留着 |



⇒ **打中之后先问四句**：它分不分辨臂？它要被判的系统区分的东西，系统当时看得到吗？它满足的是判据字面的哪一个分句？跑前条款给的每个改法，在打中的那几格上还中不中？四句都过，才按跑前条款判。

```

**出处 `research/prompts/governance-review-r2-sync.md:17-56`（整段抄，未转述）**

```markdown
## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/gate.d/54-layer0-replay.sh:232 | worktree add / apply 失败时第三行照跑 --full | 改了：丙 1：`layer0_tree_ready` 为 1 才跑，否则退 1；甲 A9：说明改成「经内存包装的那条命令的」 |
| .claude/main-agent.md:59 | 退出码就是 `--full` 的 | 改了：甲 A9 |
| .claude/gate.d/10-kb-rot.sh:29 | cd 之后按相对 $0 找共用库 | 改了：丙 2 |
| .claude/gate.d/10-kb-rot.sh:177 | 退 2 当「一份治理文档都没扫到」 | 改了：丙 2：3 是没有对象，其余非 0 是没跑成 |
| .claude/gate.d/lib-governance-refs.py:135 | 跳过的不计数 | 改了：丙 3：成功行报跳过的两种各几处 |
| .claude/gate.d/lib-governance-refs.py:35 | 「、」连着的号；「-」后面的号 | 改了：丙 6：「/」连着的也判，「-」「–」后面的不判 |
| .claude/gate.d/lib-governance-refs.py:18 | 带非 ASCII 字符 | 改了：丙 4；丙 5 嵌套「」写进文件头 |
| .claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md:10 | （样本） | 补了：丙 7：「/」连着的号、日期后的号、两种没判的 |
| .claude/gate.d/20-kb-shape.sh:46 | 历史类文件整份跳过的理由 | 改了：丙 10：写明是射程不是豁免 |
| .claude/gate.d/90-term-renames.sh:23 | 逐文件与冻结证据目录矛盾 | 改了：丙 9 |
| .claude/gate.d/75-decision-experiment-links.sh:226 | 「（3 项未定）」带空格判红 | 改了：丙 11 / 乙 3 |
| .claude/hooks/runner-dispatch-guard.sh:441 | 提交时的重阶段派 crash-verifier；gate.sh 整轮 | 改了：丙 12 |
| .claude/hooks/runner-dispatch-guard.sh:42 | 否定词与名字写进同一个分句 | 改了：丙 13 |
| .claude/hooks/heavy-test-guard.sh:767 | 崩溃验证员不经包装跑层 0 测试目标 | 改了：丙 14：换成非层 0 目标，内存包装那一道重新有判别力 |
| research/scripts/ask-local.sh:13 | 退出码表 | 补了：丙 15 |
| research/scripts/ask-local-selftest.sh:28 | 只证 UNCHECKED 那一半 | 补了：丙 16：带 VOID_SAVED=1 跑损坏那一格 |
| .claude/rules/mutation-sampling.md:82 | 输出里有编译错误记「无效」 | 改了：乙 1 / 甲 A3；乙 4 另外三栏 |
| .claude/agents/mutation-triage.md:28 | 同上 | 改了：乙 1、乙 4 |
| .claude/agents/mutation-triage.md:27 | 编不过的在「有变异无效」一栏 | 改了：乙 1 |
| .claude/agents/three-way-verifier.md:21 | 设计轮对不上一律分不清 | 改了：乙 2：拿开工时刻现查，没改过记 ✗ |
| .claude/agents/three-way-verifier.md:27 | 判别力自证 | 改了：乙 2 |
| .claude/agents/kb-scribe.md:20 | N 写阿拉伯数字或汉字数字都认 | 改了：乙 3 |
| .claude/agents/kb-scribe.md:32 | 49、75 号 | 补了：乙 8：21、30 号 |
| .claude/agents/experiment-runner.md:30 | 二进制编在 research/target/ | 补了：乙 5：入库装置编在仓根 target/ |
| .claude/agents/experiment-runner.md:32 | 登记复跑 | 补了：乙 5：入库装置写 driver_e<号> |
| .claude/agents/experiment-runner.md:27 | name 写连字符 e<号>-<英文名> | 改了：乙 6 |
| .claude/agents/crash-verifier.md:24 | 未跟踪文件按全部输入路径查 | 改了：乙 9：只查 crates litmus 与 .lkmm-static-only |
| .claude/agents/crash-verifier.md:37 | 产出表 | 补了：乙 7：日志路径 |
| .claude/rules/three-way-inference.md:163 | 报告带 `-s2` 后缀 | 改了：甲 A2 |
| .claude/rules/format-evolution.md:70 | 27 号不扫的只列脚本与产物 | 改了：甲 A4 |
| .claude/skills/crash-test/SKILL.md:12 | 门禁分诊员跑登记给自己的那几道 | 改了：甲 A5 |
| .claude/skills/gate/SKILL.md:12 | 其余时候不跑 | 改了：甲 A6 |
| .claude/agent-common.md:46 | 内存包装退出码与 cargo bench | 补了：甲 A7 |
| .claude/rules/implementation-workflow.md:52 | 主 agent 在提交流程里后台起 | 改了：甲 A8 |
| .claude/main-agent.md:59 | 这一批改了 54 号的输入…才跑全量 | 不改：甲 A1 不成立，只改文档的提交 54 号经 research/scripts/change-touches-crates.sh 退 77，不判红 |

```

**出处 `.claude/gate.d/59-crates-mutation-replay.sh:1-459`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
#
# 判据：`crates/mutations.tsv` 里每一条变异（文件、原文、替换文、cargo test 参数、必须红的测试名），
# 原文在文件里恰好命中一次，文件落在每片拷的范围之内（COPIED_INTO_EACH_SHARD，派活之前判）；
# 把仓拷到临时目录、改坏那一处、跑点名的测试，那条测试必须判红；跑完还原再下一条。
# 一条锚点腐化、一条指到拷贝范围之外、一条没红，整道红。
#
# 为什么：show-me-test.md 要「存进仓的变异清单，交给门禁反复复跑」——commit message 里的叙述只被读一次。
# 2026-09-16 发布 B 三方第二轮攻方腿把「空闲独立维护」那处改法整个撤回，全仓零判红零警告：
# 改法本身没有任何东西守着。这张表让每一处三方打中之后的改法都留一条「改回去它就红」的变异，撤回时这里先响。
#
# 分片并发跑（command-safety.md「一个脚本里的检测项，能并行就并行」）：每条变异都要起 cargo 编译 + 跑测试，
# 彼此不依赖，而串行时一条约 8 秒、191 条约 25 分钟。并发的障碍是同一篇里写着的「共用一份可写状态」——
# 所有变异改同一份源码副本、共用一个 CARGO_TARGET_DIR（cargo 的文件锁会把它们重新串行化）。
# 所以每个分片各给一份源码副本与一个编译目录。分片按 crate 切：改 singlefs-core 要重编译它自己加全部下游，
# 改 singlefs-harness 只重编译它自己，同一分片里连着改同一个 crate，增量编译的命中率才稳。
#
# 工作进程数取下面两项的最小值、至少 1（worker_count_for）；实际开几个、各项给几个、卡在哪一项，打在 stderr 的头一行（stdout 要与进程数无关，见下一段）：
#   ① 核数的一半、至多 16；设了 GATE_MUTATION_WORKERS 就用它顶掉 ①（设成 1 就是原来的串行跑法）；
#   ② 表里的条数。
#   内存不在这里收：每条经 research/scripts/run-with-memory-cap.sh 跑，它起跑之前先判整机放不放得下（slice 已占的加这一条要的，不超过 slice 的总上限才起），
#   放不下的在它那里排队，几条合起来撞顶也只在 slice 里杀（records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。按可用内存收进程数只看得见开跑那一刻，
#   看不见别的重活同时来抢，治标不治本（用户 2026-09-25 定）。这一道自己在包装之外的进程（Python 父进程与工作进程、包装的 bash）不在 slice 里，
#   从包装的余量里出：2026-09-25 量进程池工作进程每个约 12 MiB、父进程约 15 MiB。
# 活是**动态领的**（谁先跑完谁再领下一条），不按条数预先切片：`singlefs-harness` 那几条点名的是几十秒的重测试，
# 等分之下拿到它们的那一片会独自拖住整道（实测：7 片跑完、最后 1 片又跑了 4 分钟）。
# ⚠️ **输出与进程数无关**：判定行按变异表的行号排序再打印到 stdout，GATE_MUTATION_WORKERS=1 与 =8 的 stdout 要逐字相同，
# 这一条是并发化之后重新证明它还红得出来的那一半（command-safety.md「改成并行之后要重新证明它红得出来」）。
# 进度实时打到 stderr，不进 stdout，不参与那条比对。
#
# 编译产物放 ${GATE_MUTATION_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-crates-mutation-target}（跨轮复用，第一次要整编一遍）；
# 分片各自的编译目录是它加 -w<片号>，第一次从它拷一份种子，省掉每片各冷编译一遍。
#
# 每条变异的 cargo test 放进内存上限里跑（research/scripts/run-with-memory-cap.sh：systemd 的临时 scope，MemoryMax=<上限>、MemorySwapMax=0），
# 撞上限只杀这一条的进程，不把整机拖进 OOM（2026-09-25 一条无界分配的变异两次把整机拖进 OOM，records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。
#   上限从三处取，先到先用：变异表里单起一行「# 每条变异的内存上限：<上限>」（判别力样本用它把上限压到 512M）、GATE_MUTATION_MEMORY_MAX、默认 4G。
#   默认 4G 的依据：本机 60 GiB 内存，本地模型服务（vllm 与 ray）连同会话常驻约 12 GiB（2026-09-25 `free -g` 的 used 列是 12），
#   同时可能有 3 件重活 ⇒ 每件 16 GiB；这一道同时开好几个工作进程，4G 给正常的编译与测试留余量。2026-09-25 量过编译那一半：
#   HEAD 的四个 crate 全部测试目标在 4G 上限、2 个并行编译下从零编得过，匿名内存峰值 0.60 GiB（每 0.2 秒取一次样）；点名的测试跑起来要多少没量，
#   包装的峰值表（research/scripts/memory-peaks.tsv）跑过一遍之后按条记着。换机器要重算（这些数只在本机成立）。
#   几条同时跑合起来放不放得下由包装排队判、slice 的总上限兜底，这里不按内存收进程数。
#   撞了这一条自己的上限（包装退 250）记「内存撞顶」：这条破坏让被测代码无界分配，点名的测试没来得及红，不算抓到也不算没红，单列一栏、整道判红。
#   被总上限挤掉（包装退 254：整个 slice 满了、内核在 slice 里挑了这一条杀）记「被总上限挤掉」，排队等满包装的等待上限还放不下（包装退 252）记「排不上没跑」：
#   两栏都不是这条变异的结论，单列、整道判红，出路是重跑。
#   带上限的 scope 或 slice 的总上限起不来、设不上（没有用户级 systemd、D-Bus 连不上）就整道判红，一条都不跑，不退回无上限去跑。
#
# 每条变异的 cargo test 限时（交给包装：RUN_WITH_MEMORY_CAP_TIME_LIMIT，给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间；限时里含这一条要做的重编译）：
#   秒数从三处取，先到先用：变异表里单起一行「# 每条变异的超时秒数：<秒>」（判别力样本用它压到 20 秒）、GATE_MUTATION_TIMEOUT、默认 1800。
#   到点 systemd 给 scope 里每个进程（cargo 与测试进程）发 TERM，TIMEOUT_KILL_GRACE_SECONDS 秒还不退再发 KILL，包装按 scope 的 Result=timeout 退 253。
#   不在包装外面套 timeout：排队的时间会算进限时里，排得久的变异会被误判成超时。
#   超过限时记「超时」：这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久。不算抓到、不算没红，与「内存撞顶」「没红」「无效」分开单列一栏、各自计数，整道判红。
#   默认 1800 秒的依据：点名随机历史测试的变异在 16 个工作进程并发时，一条卡在那个测试文件上超过 6–8 分钟（没记到跑完的挂钟），串行单次 1.5–3 分钟
#   （.claude/kb/checks-owed.md 的 C457（被测试自己的线程池与外层进程池叠加超订阅），2026-09-21 在副本上量的）；四个 crate 的全部测试目标在 4G 上限、
#   2 个并行编译下冷编译 23–25 秒（2026-09-25 量）。1800 秒约是 8 分钟的 4 倍，给没量到的尾巴留余地；代价是一条真挂死的变异让一个工作进程多占 30 分钟。
#   这些数只在本机成立，换机器要重算。
#
# 判别力：fixtures/59-crates-mutation-replay.sh/red 的变异表把上限压到 512M、限时压到 20 秒：加一条把 1 MiB 的缓冲改成 1536 MiB 的变异，必须报「内存撞顶」；
#   加一条把步长 1 改成 0、循环永远走不到上界的变异，必须报「超时」；计数行里没红、无效、内存撞顶、超时各数各的。
# green 同样压到 512M，留着一行撤掉了的「# 给整机留的内存余量：1T」：头一行必须报开 2 个（两条、核数的一半多于 2）、卡在「表里的条数」那一项，
#   不许再按内存收进程数（按内存收的那一版在这份样本上只开 1 个）；两条正常变异必须照旧红在点名的测试上。
#
#   bash .claude/gate.d/59-crates-mutation-replay.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
# 这一道原先没有任何范围判定，每趟跑满：2026-09-22 实测，一批 27 个路径里 crates/ 零个，它照跑不误。
reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
reuse_rc=$?
if [[ "$reuse_rc" != 0 ]]; then
  echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
  echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
  exit 77
fi

TABLE="crates/mutations.tsv"
if [[ ! -f "$TABLE" ]]; then
  echo "  ✗ 没有 $TABLE"
  echo "     → 怎么办：建一张六段制表符分隔的表（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），每一处三方打中之后的改法留一条。"
  exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "  ✗ 没有 cargo，变异复跑不了"
  echo "     → 怎么办：装 Rust 工具链（scripts/env.sh 会报），再跑这一道。"
  exit 1
fi
export GATE_MUTATION_TARGET_DIR="${GATE_MUTATION_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-crates-mutation-target}"
# 带内存上限跑一条命令的包装在这份阶段所在的仓里（样本仓里没有 research/）
GATE_MUTATION_MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
export GATE_MUTATION_MEMORY_CAP_RUNNER
python3 - "$ROOT" "$TABLE" <<'PY'
import atexit
import concurrent.futures
import fnmatch
import multiprocessing
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

root, table = sys.argv[1], sys.argv[2]
memory_cap_runner = os.environ["GATE_MUTATION_MEMORY_CAP_RUNNER"]
MEMORY_CAP_HIT_EXIT = 250            # run-with-memory-cap.sh：撞了这一条自己的上限（scope 的 Result 是 oom-kill、自己的 oom 计数不为 0）
MEMORY_CAP_UNAVAILABLE_EXIT = 251    # run-with-memory-cap.sh：带上限的 scope 或 slice 的总上限起不来
MEMORY_ADMISSION_REFUSED_EXIT = 252  # run-with-memory-cap.sh：排队等满还放不下，命令没跑
TIME_LIMIT_HIT_EXIT = 253            # run-with-memory-cap.sh：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT（scope 的 Result 是 timeout）
SLICE_TOTAL_HIT_EXIT = 254           # run-with-memory-cap.sh：被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀）
DEFAULT_MEMORY_MAX_PER_MUTATION = "4G"   # 依据见文件头
MEMORY_MAX_DIRECTIVE = re.compile(r"^#\s*每条变异的内存上限[：:]\s*(\S+)\s*$")
DEFAULT_TIMEOUT_SECONDS_PER_MUTATION = "1800"   # 依据见文件头
TIMEOUT_DIRECTIVE = re.compile(r"^#\s*每条变异的超时秒数[：:]\s*(\S+)\s*$")
TIMEOUT_KILL_GRACE_SECONDS = 30          # 到点发 TERM 之后再等这么久，还不退就发 KILL（交给包装的 RUN_WITH_MEMORY_CAP_KILL_GRACE）


def unescape(text):
    return text.replace("\\n", "\n")


rows = []
memory_max_from_table = None
timeout_seconds_from_table = None
with open(os.path.join(root, table), encoding="utf-8") as handle:
    for line_number, line in enumerate(handle, 1):
        line = line.rstrip("\n")
        directive = MEMORY_MAX_DIRECTIVE.match(line)
        if directive:
            memory_max_from_table = directive.group(1)
        directive = TIMEOUT_DIRECTIVE.match(line)
        if directive:
            timeout_seconds_from_table = directive.group(1)
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 6 or any(field == "" for field in fields):
            print(f"  ✗ {table}:{line_number} 不是六段（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）：{line[:80]}")
            print("     → 怎么办：六段用制表符分隔，一段都不能空；原文 / 替换文里的换行写成 \\n。")
            sys.exit(1)
        rows.append((line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5]))
if not rows:
    print(f"  ✗ {table} 里一条成形的变异都没有")
    print("     → 怎么办：至少一条：三方打中之后的每一处改法，留一条「改回去它就红」的变异。")
    sys.exit(1)
memory_max = memory_max_from_table or os.environ.get("GATE_MUTATION_MEMORY_MAX", "").strip() or DEFAULT_MEMORY_MAX_PER_MUTATION
if not re.fullmatch(r"[1-9][0-9]*[KMGT]?", memory_max):
    print(f"  ✗ 每条变异的内存上限写成了「{memory_max}」，不是 systemd 的写法")
    print("     → 怎么办：写成 正整数[K|M|G|T]，例 4G、512M（表头那一行「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。")
    sys.exit(1)
timeout_seconds_text = timeout_seconds_from_table or os.environ.get("GATE_MUTATION_TIMEOUT", "").strip() or DEFAULT_TIMEOUT_SECONDS_PER_MUTATION
if not re.fullmatch(r"[1-9][0-9]*", timeout_seconds_text):
    print(f"  ✗ 每条变异的限时写成了「{timeout_seconds_text}」，不是正整数秒")
    print("     → 怎么办：写成正整数秒，例 1800（表头那一行「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT）。")
    sys.exit(1)
timeout_seconds = int(timeout_seconds_text)
stale = []
for line_number, name, path, old, new, _args, _expected in rows:
    full = os.path.join(root, path)
    if not os.path.isfile(full):
        stale.append(f"{table}:{line_number} {name}：文件 {path} 不存在")
        continue
    count = open(full, encoding="utf-8").read().count(old)
    if count != 1:
        stale.append(f"{table}:{line_number} {name}：原文在 {path} 里命中 {count} 次，要恰好 1 次")
if stale:
    print("  ✗ 变异表的锚点腐化：")
    for item in stale:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：改代码时把变异表里的原文一起改到今天的写法（锚点腐化的那条变异等于没跑过，mutation-sampling.md 第七类）。")
    sys.exit(1)

COPIED_INTO_EACH_SHARD = ["crates", "Cargo.toml", "Cargo.lock", "litmus", "research/results"]
# 拷目录时跳过的名字（各 crate 自己的编译目录）。拷贝范围由这两样一起定：prepare_shard 照它们拷，下面那道路径检查照它们判。
IGNORED_WHEN_COPYING_INTO_EACH_SHARD = ["target"]
shared_target_directory = os.environ["GATE_MUTATION_TARGET_DIR"]


def is_copied_into_each_shard(path):
    """表里「文件」那一段在不在每片的源码副本里：落在 COPIED_INTO_EACH_SHARD 的某一项之内，
    而且那一项底下的每一段路径都不叫 IGNORED_WHEN_COPYING_INTO_EACH_SHARD 里的名字。绝对路径、用 `..` 跳出去的都不在。"""
    normalized = os.path.normpath(path)
    if os.path.isabs(normalized):
        return False
    for item in COPIED_INTO_EACH_SHARD:
        if normalized == item:
            return True
        if normalized.startswith(item + os.sep):
            below_item = normalized[len(item) + 1:].split(os.sep)
            return not any(fnmatch.fnmatch(part, pattern) for part in below_item for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD)
    return False


# 锚点只在真仓上核过：文件落在拷贝范围之外时，工作进程在自己那份副本里打不开它，要到派活之后才炸成一个没有出路的 traceback。
# 所以派活之前先判，有一行在外面就一个工作进程都不起。
rows_outside_shard_copy = [f"{table}:{line_number} {name}：{path}"
                           for line_number, name, path, _old, _new, _args, _expected in rows if not is_copied_into_each_shard(path)]
if rows_outside_shard_copy:
    print("  ✗ 变异表有几行的文件不在每片拷的范围里（锚点在真仓上核得过，工作进程在自己的源码副本里却打不开它）：")
    for item in rows_outside_shard_copy:
        print(f"      {item}")   # gate-lint:detail
    print(f"     → 怎么办：把那个路径（或它所在的目录）加进本阶段的 COPIED_INTO_EACH_SHARD（现在是 {'、'.join(COPIED_INTO_EACH_SHARD)}），"
          "或者把这条变异改到拷贝范围之内的文件上；这一轮一个工作进程都没起。")
    sys.exit(1)

# 派活之前先试一次带上限的 scope 起不起得来：起不来就一条都不跑，不退回无上限
memory_cap_check = subprocess.run(["bash", memory_cap_runner, "--check", memory_max], capture_output=True, text=True)
if memory_cap_check.returncode != 0:
    print(f"  ✗ 带内存上限（每条 {memory_max}）的 systemd scope 起不来（{memory_cap_runner} --check 退 {memory_cap_check.returncode}），一条变异都没跑：")
    for detail_line in (memory_cap_check.stdout + memory_cap_check.stderr).splitlines():
        print(f"      {detail_line}")   # gate-lint:detail
    print("     → 怎么办：照上面那几行修用户级 systemd / D-Bus（systemctl --user status）再跑这一道；不许拿掉上限去跑，无界分配的变异会把整机拖进 OOM。")
    sys.exit(1)


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：分片按它切，同一片里连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def worker_count_for(row_count):
    """开几个工作进程，连同头一行要报的话（各项给几个、卡在哪一项）。两项的意思与依据见文件头「工作进程数取下面两项的最小值」那一段。

    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，而**测试那一段是单线程的**。
    实测 8 个进程、不限 cargo 的编译并行度时，峰值才 16 个 rustc——编译的并行度本来就用不满
    （依赖链是串的），多出来的核闲着。所以开得比「核数 ÷ 4」多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住，让总并行度落回核数。
    内存不在这里收（文件头那一段）：每条经 run-with-memory-cap.sh 排队，放不下的在它那里等。
    """
    bounds = []   # (这一项叫什么, 给几个, 头一行里怎么说)
    requested = os.environ.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        bounds.append(("GATE_MUTATION_WORKERS", int(requested), f"GATE_MUTATION_WORKERS 给 {int(requested)} 个"))
    else:
        processor_count = os.cpu_count() or 4
        by_processor = max(1, min(16, processor_count // 2))
        bounds.append(("核数", by_processor, f"核数 {processor_count} 的一半、至多 16，给 {by_processor} 个"))
    bounds.append(("表里的条数", row_count, f"表里 {row_count} 条"))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}；内存不按进程数收：每条经 run-with-memory-cap.sh 排队，放不下的等（slice 的总上限兜底）")
    return worker_count, head_line


def rows_grouped_by_crate(all_rows):
    """按 crate 归堆、堆内保持表序，再把几堆首尾相接：切片时同一个 crate 才会落在同一片里。"""
    groups = {}
    for row in all_rows:
        groups.setdefault(crate_of(row[2]), []).append(row)
    ordered = []
    for crate_name in sorted(groups, key=lambda name: (-len(groups[name]), name)):
        ordered.extend(groups[crate_name])
    return ordered


# 每个工作进程自己的那一份：副本目录与编译目录，进程起来时建一次、这一轮里一直用
WORKER_STATE = {}


def init_worker(sequence_counter, shard_root):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。

    片号从共享计数器领，不是按任务分——任务是动态领的（谁先跑完谁再领下一条），
    按条数预先切片会让拿到重测试的那一片独自拖住整道（实测：7 片跑完、最后 1 片又跑了 4 分钟）。
    """
    with sequence_counter.get_lock():
        sequence_counter.value += 1
        worker_sequence = sequence_counter.value
    work, target_directory = prepare_shard(worker_sequence, shard_root)
    WORKER_STATE["work"] = work
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = target_directory
    WORKER_STATE["environment"] = environment
    # 源码副本建在父进程的 shard_root 底下、由父进程收尾时整个删；编译目录留着跨轮复用。
    # 不在这里挂 atexit：进程池的工作进程走 os._exit 退出，atexit 在这里一次都不跑（实测 /tmp 里堆了 131 个没删的副本）。


def judge_row(row):
    """进程池里的一格活：一条变异。副本与编译目录是这个工作进程独有的。"""
    return judge_one(WORKER_STATE["work"], WORKER_STATE["environment"], row)


def prepare_shard(worker_sequence, shard_root):
    """一个工作进程自己的一份源码副本加一个编译目录，进程起来时建一次。"""
    work = tempfile.mkdtemp(prefix=f"shard-{worker_sequence}-", dir=shard_root)
    for item in COPIED_INTO_EACH_SHARD:
        source = os.path.join(root, item)
        if not os.path.exists(source):
            continue
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns(*IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy2(source, target)
    # 各片一个编译目录，第 0 片用共用的那个（跨轮热着），其余片自己编译起来。
    # ⚠️ 不从共用目录拷种子：实测 7 片拷 16 GiB 要 7 分钟还没拷完，期间只有 1 个 rustc 在跑——
    # 那是拿瓶颈资源（磁盘 I/O）去省空闲资源（32 核 CPU）。各片自己冷编译一次，编译走 CPU、彼此不争。
    target_directory = (shared_target_directory if worker_sequence == 1
                        else f"{shared_target_directory}-w{worker_sequence}")
    return work, target_directory


def judge_one(work, environment, row):
    """一条变异的判定：改坏那一处、跑点名的测试、还原。回的是 (行号, 档, 那一条要打印的话)。"""
    line_number, name, path, old, new, args, expected = row
    full = os.path.join(work, path)
    pristine = open(full, encoding="utf-8").read()
    assert pristine.count(old) == 1, (path, name)
    open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
    # 限时交给包装（RuntimeMaxSec，从起跑算，不算排队）：不在外面套 timeout，排队的时间不算进限时
    run_environment = dict(environment, RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(timeout_seconds), RUN_WITH_MEMORY_CAP_KILL_GRACE=str(TIMEOUT_KILL_GRACE_SECONDS))
    try:
        run = subprocess.run(["bash", memory_cap_runner, memory_max, "cargo", "test", "--offline"] + args.split(),
                             cwd=work, env=run_environment, capture_output=True, text=True)
    finally:
        open(full, "w", encoding="utf-8").write(pristine)
    output = run.stdout + run.stderr
    tail = "\n".join(output.splitlines()[-8:])
    # 包装报的几种结局先判：被停、被杀的测试二进制可能已经打出了半截输出，不许拿它判抓到或没红
    if run.returncode == TIME_LIMIT_HIT_EXIT:
        return line_number, "timeout", f"{table}:{line_number} {name}：{timeout_seconds} 秒没跑完（超时），点名的测试 {expected} 没来得及红\n{tail}"
    if run.returncode == MEMORY_CAP_HIT_EXIT:
        return line_number, "memory", f"{table}:{line_number} {name}：撞了内存上限 {memory_max}（内存撞顶），点名的测试 {expected} 没来得及红\n{tail}"
    if run.returncode == SLICE_TOTAL_HIT_EXIT:
        return line_number, "squeezed", f"{table}:{line_number} {name}：被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀），这一条的结果不算数\n{tail}"
    if run.returncode == MEMORY_ADMISSION_REFUSED_EXIT:
        return line_number, "refused", f"{table}:{line_number} {name}：内存不够排不上（包装等满了等待上限），这一条没跑\n{tail}"
    if run.returncode == MEMORY_CAP_UNAVAILABLE_EXIT:
        return line_number, "unavailable", f"{table}:{line_number} {name}：带内存上限的 scope 起不来，这一条没跑\n{tail}"
    red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)
    ran_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. ", re.M)
    if run.returncode != 0 and red_pattern.search(output):
        return line_number, "caught", f"  ✓ {name}：{expected} 红了"
    if ran_pattern.search(output):
        return line_number, "failure", f"{table}:{line_number} {name}：{expected} 没红（退出码 {run.returncode}）\n{tail}"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        # 替换文写进源码之后编不过：既不算被抓也不算没红，是一条无效变异，
        # 它要证明的那个行为今天零变异覆盖（.claude/rules/mutation-sampling.md 第八类）。
        # 报成「没红」会把排查指向「去补一条用例」，而要改的是这一行替换文。
        return line_number, "invalid", f"{table}:{line_number} {name}：替换文写进源码之后编不过（退出码 {run.returncode}）\n{tail}"
    return line_number, "failure", f"{table}:{line_number} {name}：点名的测试 {expected} 没跑到（退出码 {run.returncode}）\n{tail}"


ordered_rows = rows_grouped_by_crate(rows)
worker_count, worker_count_head_line = worker_count_for(len(rows))
# 实际开几个、各项给几个、卡在哪一项打在头一行，走 stderr：stdout 要与进程数无关（文件头）
print(worker_count_head_line, file=sys.stderr, flush=True)
# 每个 cargo 的编译并行度：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢
cargo_jobs = os.environ.get("GATE_MUTATION_CARGO_JOBS", "").strip() \
    or str(max(1, (os.cpu_count() or 4) // worker_count))
os.environ["CARGO_BUILD_JOBS"] = cargo_jobs
judgements = []
# 用进程池不用线程池：工作进程起来时要把源码拷一份，`shutil.copytree` 是纯 Python 循环、一路持着 GIL，
# 线程池下实测 8 个分片只有 1 个真在跑、其余 7 个卡在 futex 上等 GIL，九分钟一条判定都没出。
# chunksize=1 是动态调度：谁先跑完谁再领下一条，不预先按条数切片。条目按 crate 归过堆再发，
# 于是同一个工作进程连着拿到的多半是同一个 crate，增量编译的命中率还在。
sequence_counter = multiprocessing.Value("i", 0)
# 各工作进程的源码副本都建在这个总目录底下，父进程退出时整个删掉（父进程正常退出与 sys.exit 都走 atexit）。
shard_root = tempfile.mkdtemp(prefix="singlefs-mutation-replay-")
atexit.register(shutil.rmtree, shard_root, ignore_errors=True)
with concurrent.futures.ProcessPoolExecutor(max_workers=worker_count,
                                            initializer=init_worker,
                                            initargs=(sequence_counter, shard_root)) as pool:
    # 进度实时打到 stderr：判定行走 stdout、最后按表的行号排序统一打印，两边不混。
    # 只有 stdout 参与「输出与分片数无关」那条比对，进度是给盯着跑的人看的。
    try:
        for judgement in pool.map(judge_row, ordered_rows, chunksize=1):
            # 工作进程里抛出来的异常在这里原样炸出去：并行不许把失败吃掉（command-safety.md）
            judgements.append(judgement)
            print(f"  … 已判 {len(judgements)}/{len(rows)} 条", file=sys.stderr, flush=True)
    except concurrent.futures.process.BrokenProcessPool:
        # 一个工作进程被硬杀（OOM、段错误）时 pool.map 当场抛这个，而不是安静地少产出几项——
        # 下面那道「派多少收多少」的闸因此永远轮不到，真实的失败路径是一条没有出路的裸 traceback。
        print(f"  ✗ 有工作进程中途死了（已判 {len(judgements)}/{len(rows)} 条），这一轮的变异没跑全")
        print("     → 怎么办：多半是内存不够被 OOM 杀的——GATE_MUTATION_WORKERS 调小再跑一遍；")
        print("       单跑 GATE_MUTATION_WORKERS=1 能跑完就是并发度的问题，还死就去看那一条变异本身。")
        sys.exit(1)

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿（command-safety.md）
if len(judgements) != len(rows):
    print(f"  ✗ 派出去 {len(rows)} 条变异，只收回 {len(judgements)} 条判定")
    print("     → 怎么办：这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，")
    print("       是就去查领活与收束那一段（init_worker、judge_row 与 pool.map 那几行）。")
    sys.exit(1)

# 按变异表的行号排序再打印：输出与分片数无关，GATE_MUTATION_WORKERS=1 与 =8 逐字相同
judgements.sort(key=lambda judgement: judgement[0])
invalid = [text for _, verdict, text in judgements if verdict == "invalid"]
failures = [text for _, verdict, text in judgements if verdict == "failure"]
memory_hits = [text for _, verdict, text in judgements if verdict == "memory"]
timeouts = [text for _, verdict, text in judgements if verdict == "timeout"]
unavailable = [text for _, verdict, text in judgements if verdict == "unavailable"]
squeezed = [text for _, verdict, text in judgements if verdict == "squeezed"]
refused = [text for _, verdict, text in judgements if verdict == "refused"]
caught = [text for _, verdict, text in judgements if verdict == "caught"]
for text in caught:
    print(text)
# 各栏各数各的：超时、内存撞顶、被总上限挤掉、排不上既不算抓到也不算没红，与无效一样单列；这一行与进程数无关，进 stdout
print(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、内存撞顶 {len(memory_hits)} 条、超时 {len(timeouts)} 条、"
      f"被总上限挤掉 {len(squeezed)} 条、排不上没跑 {len(refused)} 条、scope 起不来没跑 {len(unavailable)} 条（共 {len(rows)} 条）")
if invalid:
    print(f"  ✗ 有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：")
    for item in invalid:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：改这一行替换文，不是去补用例。先看反斜杠——表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；")
    print("       别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。")
if failures:
    print(f"  ✗ 有变异没红（没红 {len(failures)} 条）：")
    for item in failures:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。")
if memory_hits:
    print(f"  ✗ 有变异撞了内存上限（内存撞顶 {len(memory_hits)} 条，每条上限 {memory_max}）：")
    for item in memory_hits:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：撞顶 = 这条破坏让被测代码无界分配，点名的测试没来得及红——不算抓到，也不是没红。换一处改法让它红在点名的测试上，")
    print("       或者给被测代码加一道先红的界（循环步数上限、分配量的断言）；正常的测试确实要这么多内存，才调大上限（表头「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。")
if timeouts:
    print(f"  ✗ 有变异超时（超时 {len(timeouts)} 条，每条限 {timeout_seconds} 秒）：")
    for item in timeouts:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：超时 = 这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久——不算抓到，也不是没红。先在副本里只改坏这一处、")
    print("       不限时单跑它那一行的 cargo test 参数，看它结束不结束：不结束就换一处改法让它红在点名的测试上，或者给被测代码加一道先红的界（循环步数上限）；")
    print("       结束得了就调大限时（表头「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT），别把它记成抓到。")
if squeezed:
    print(f"  ✗ 有变异被 slice 的总上限挤掉（被总上限挤掉 {len(squeezed)} 条；这几条没撞各自的上限 {memory_max}，是 slice 里合起来满了）：")
    for item in squeezed:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：这几条的结果不算数，整道重跑；常这样说明包装的峰值表记少了（同时跑的几条实际占的比表里记的多），")
    print("       跑的时候 bash research/scripts/run-with-memory-cap.sh --status 看 slice 里是谁在占，别拿掉包装去跑。")
if refused:
    print(f"  ✗ 有变异排不上没跑（排不上没跑 {len(refused)} 条；包装等满了等待上限，slice 还是放不下）：")
    for item in refused:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，等那几件重活跑完再整道重跑；")
    print("       别拿掉包装去跑——排不上说明此刻整机放不下。")
if unavailable:
    print(f"  ✗ 跑到一半带内存上限的 scope 起不来了（{len(unavailable)} 条没跑）：")
    for item in unavailable:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：修好用户级 systemd / D-Bus（systemctl --user status）整道重跑；不许拿掉上限去跑。")
if invalid or failures or memory_hits or timeouts or squeezed or refused or unavailable:
    sys.exit(1)
# ⚠️ 进程数与 cargo 编译并行度**不写进 stdout**：它们是两次跑唯一不同的输入，写进成功行就让
# 「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」这句话当场为假（独立复核实测：191 行判定行逐字相同，
# 只有这一行收尾不同）。要看用了几个进程、为什么，看 stderr 的头一行。
print(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次；每条在内存上限 {memory_max} 里跑、内存撞顶 0 条，"
      f"每条经 run-with-memory-cap.sh 排队、被总上限挤掉与排不上 0 条，每条限时 {timeout_seconds} 秒、超时 0 条；"
      f"工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数无关）")
print(f"  … 这一轮 {worker_count} 个工作进程，每个 cargo 编译并行度 {cargo_jobs}，每条变异内存上限 {memory_max}、限时 {timeout_seconds} 秒", file=sys.stderr)
PY
```

**出处 `research/scripts/run-with-memory-cap.sh:1-1006`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 给一条命令定内存上限，起跑之前先判整机放不放得下、放不下就排队（records/2026-09-16-subagent拆分提案.md 第四十节第 28、30 行）。三层：
#   ① 这一条的上限：放进 systemd 的临时 scope（MemoryMax=<上限>、MemorySwapMax=0、OOMPolicy=stop），撞上限只杀这个 scope 里的进程；
#   ② 总量兜底：每个 scope 都挂进同一个 slice（默认 singlefs-heavy.slice），slice 的 MemoryMax = 整机内存 − 余量。几条合起来撞顶也只在 slice 里杀，
#      杀不到 slice 外面的本地模型服务 vllm-prod 与 Claude Code 会话。slice 设不上就报错退出，不退回无总上限；
#   ③ 起跑前判内存量：拿一把锁，算「slice 里已占的 + 这一条要的」，不超过总上限才起；放不下就放锁、隔一会儿再判，等满上限报「排不上」退出。
# 变异跑道 research/scripts/mutate.sh 与门禁 59 号每条变异都经它跑；子 agent 跑 cargo test / cargo run / 实验二进制也要经它
# （.claude/hooks/heavy-test-guard.sh 在执行前拒不经它的）。提交时的重阶段整条经它跑，例：
#   SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash <根>/.claude/gate.d/54-layer0-replay.sh --full <根>
#
#   run-with-memory-cap.sh <上限> <命令> [参数…]   排队，在上限里跑那条命令；退出码见下
#   run-with-memory-cap.sh --check <上限>          只试 slice 的总上限设不设得上、带同样上限的 scope 起不起得来（在 slice 里跑 true，不排队）；起不来退 251
#   run-with-memory-cap.sh --status                现量：整机内存、余量、slice 总上限与已占、账上在跑的每一条、slice 外面的用量
#   run-with-memory-cap.sh --selftest              自证（每一例在自己的临时 slice 里，总上限几百 M，吃不满机器）
#
# 上限照 systemd 的 MemoryMax 写法：正整数加 K / M / G / T（1024 进制），例 16G、512M。默认值由调用方定，写在调用方的脚本头。
# 退出码：
#   0–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 250–254）；
#   250    撞了这一条自己的上限：scope 的 Result 是 oom-kill，scope 自己的 memory.events 里 oom 不为 0（或读不到）。
#          只认 Result、不认退出码：被杀的是 cargo 起的测试二进制时 cargo 退 101 或被停时退 143，被杀的是命令本身时退 137，外面 kill -9 同样退 137；
#   251    slice 的总上限设不上、systemd-run 起不来（没有用户级 systemd、D-Bus 连不上、上限写法 systemd 不认）、账与锁的目录建不了，或查不到 scope 的结局：
#          那条命令一行都没跑（或结局判不了），不退回无上限、无总上限去跑——那正是要防的事；
#   252    内存不够排不上：等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下，或这一条要的量比总上限还大（等多久都放不下，不等）。命令一行都没跑；
#   253    超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）：scope 的 Result 是 timeout，systemd 停掉了 scope 里的进程；
#   254    被总上限挤掉：scope 的 Result 是 oom-kill，而 scope 自己的 oom 计数是 0、oom_kill 不为 0——整个 slice 满了，内核在 slice 里挑了这一条杀，
#          不是它自己撞了它的上限；它的结果不算数，重跑它；
#   2      用法错（上限、余量、总上限不是 正整数[KMGT] 的写法，没给命令）。
#
# 余量（RUN_WITH_MEMORY_CAP_RESERVE，默认 20G）留给 slice 外面：本地模型服务 vllm-prod、各个 Claude Code 会话与它们起的工具、门禁在包装外面的进程、内核。
#   2026-09-25 10:1x UTC 量的（只在本机成立；换机器、vllm 换模型或换并行度都要重量）：MemTotal 60.1 GiB；slice 外面回收不掉的合计 13.4 GiB
#   （MemTotal − MemAvailable，那时 slice 里几乎是空的），其中 vllm-prod 7.4 GiB anon + 0.6 GiB shmem（它 cgroup 的 memory.stat；
#   `systemctl status vllm-prod` 报的 Memory 37.9G 里 30.3 GiB 是模型文件的页缓存，回收得掉），user.slice 2.4 GiB anon（Claude Code 会话与它们起的进程），
#   其余约 3 GiB 是内核（Slab 1.9 GiB、页表）与别的服务。20G 比 13.4 GiB 多约 6.6 GiB，给会话变多、vllm 在负载下涨、门禁在包装外面的 python 进程留着；
#   本机 slice 总上限因此约 40 GiB。怎么现量：bash research/scripts/run-with-memory-cap.sh --status 打出上面每一项此刻的数。
#   slice 外面涨过了余量，总上限挡不住那一部分，整机照样可能被外面涨满：那一类靠看门狗的常驻内存告警与 .claude/hooks/session-start.sh 的 OOM 报告。
#
# 排队（第 ③ 层）怎么算，判与记账在同一把锁（${状态目录}/lock，flock）里，几条同时来的一条一条判：
#   这一条要的量：峰值表里这条命令（键见下）上一次实测的峰值，不超过它的上限；表里没有的按它的上限算——scope 的 MemoryMax 就是它最多能占的量，按它算不会少算。
#   已占 = slice 里回收不掉的用量（memory.current 减 active_file 与 inactive_file：页缓存回收得掉，而结束了的 scope 的页缓存挂回 slice、一直算在 slice 的
#          memory.current 里；2026-09-25 实测 scope 里 dd 写 120 MiB 退出之后，slice 的 memory.current 还是 131 MiB，其中 inactive_file 126 MiB）
#        + 账上每条放行了还在跑的 max(0, 它要的量 − 它的 scope 此刻回收不掉的用量)：刚起跑的 cargo 还没涨到量，只看 slice 的用量，前后脚来的两条都判得下、合起来超。
#   已占 + 这一条要的量 ≤ 总上限就放行，在账上记一笔（${状态目录}/ledger/<unit>.json：包装的进程号与起始时刻、要的量、上限、键），scope 收尾之后删掉；
#   包装被 KILL、没删掉的那一笔，下一次判的时候按进程号与起始时刻认出来删掉。
#   放不下就放锁、隔 ADMISSION_POLL_SECONDS 秒再判，第一次等与之后每 WAITING_REPORT_EVERY_SECONDS 秒往 stderr 报一行在等什么；
#   等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（默认 3600）秒还放不下，列出账上占着的每一条，退 252。默认 3600 秒的依据：门禁 59 号一条变异限时 1800 秒，
#   排在后面的一般等前面一条跑完就轮到；再宽一倍，给别的重活占着 slice 的时候。
#
# 峰值表（RUN_WITH_MEMORY_CAP_PEAKS，默认主仓的 research/scripts/memory-peaks.tsv；从 git worktree 里跑的也读写主仓那一份）：一行一条命令，文件头的 # 行写口径，
#   由这个脚本建、读、写（写在同一把锁里，先写临时文件再改名；按 MiB 向上取整，与表里记的相同就不写）。口径：scope 里那个外壳 bash 在命令退出之后读自己 cgroup 的 memory.peak（字节）。
#   外壳不 exec 命令、留下来读，是因为 scope 一收尾它的 cgroup 就删了、systemd 也不留（2026-09-25 实测命令退出之后 MemoryPeak=[not set]）。
#   含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。撞了这一条自己的上限记成上限；超时、被停、被总上限挤掉的不记（量到的是半截）。
#   不进 git（.gitignore）：数只在本机成立；而且门禁 59 号跑的时候每条都写它，放在被跟踪的地方，gate.sh 开跑与收尾的工作区指纹就对不上、整轮判红
#   （.claude/singlefs-ai-sop/scripts/lib.sh 的 worktree_fingerprint 用 git add -A 算，被忽略的文件不算）。
#   键：RUN_WITH_MEMORY_CAP_KEY 给了就用它；不给就把命令的各个词用空格接起来（制表符、换行换成空格，测试二进制名里 -<16 位十六进制> 的哈希去掉，截到 300 字）。
#
# 限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT，秒；不设不限）：给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间；到点 systemd 给 scope 里每个进程发 TERM，
#   RUN_WITH_MEMORY_CAP_KILL_GRACE（默认 30）秒还不退再发 KILL，退 253。要限时就用这个变量，别在包装外面套 timeout：套在外面的把排队的时间也算进去。
# 等待上限（RUN_WITH_MEMORY_CAP_WAIT_SECONDS，秒，默认 3600）。这三个只管这一条：RUN_WITH_MEMORY_CAP_KEY、_TIME_LIMIT、_WAIT_SECONDS 传进命令之前清掉，
#   命令里再经包装跑的不会接着用。
# 其余环境变量有默认，只在自证与特殊场合设：
#   RUN_WITH_MEMORY_CAP_SLICE          slice 名，默认 singlefs-heavy.slice（名字里的 - 是 systemd 的层级：它挂在 singlefs.slice 底下）；只许 singlefs 开头的 .slice，别的退 2
#   RUN_WITH_MEMORY_CAP_SLICE_TOTAL    直接给总上限（systemd 写法），顶掉「整机内存 − 余量」；自证用它压到几百 M
#   RUN_WITH_MEMORY_CAP_STATE_DIR      锁与账的目录，默认 ${XDG_RUNTIME_DIR:-/run/user/<uid>}/singlefs-heavy-admission（tmpfs，重启清空，与 slice 同寿）
#   RUN_WITH_MEMORY_CAP_CGROUP_ROOT    cgroup v2 挂在哪，默认 /sys/fs/cgroup
#   RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE  判完放得下、记账之前停这么多秒：只给自证把「几条同时来」的竞态撑开
#   RUN_WITH_MEMORY_CAP_SELFTEST_TARGET       --selftest 测哪一份包装，默认自己：拿改前那一份跑新的自证，看它判错
# 包装里再经包装跑的（整条 gate.sh 经它跑，里面 59 号的每条再经它）：里层另起一个 scope、挪出外层，挂在同一个 slice 里各排各的队；外层照它要的量占着账。
#
# 弄坏开关 RUN_WITH_MEMORY_CAP_BREAK（只给 --selftest 证明它会红用）：nocap 不进 scope 直接跑（第一版之前的跑法）、fallback systemd-run 起不来时退回无上限跑、
#   exitcode 按退出码 137 / 143 判撞顶、noresult 不认 Result、noslice 不挂 slice 也不排队（加总上限之前的跑法）、slicefallback slice 设不上时不挂 slice 照跑、
#   nolock 判与记账不拿锁、noledger 已占只看 slice 的用量不算账上还没涨到量的、nodefault 峰值表里没有的按 0 算、ignoretable 不看峰值表一律按上限算、
#   noslicehit 被总上限挤掉的也报成撞了自己的上限、notimelimit 不设限时、keepstale 账上包装已经死了的那一笔不删。
#
# 发信号的地方，每一处只打得到这一条自己起的进程（用户 2026-09-25 定「后面的脚本不能终止前面的脚本」「不能动 ssh」；
# records/2026-09-16-subagent拆分提案.md 第四十节第 32 行）：
#   撞顶（OOMPolicy=stop）与超时（RuntimeMaxSec）由 systemd 停这一条自己开的 scope，scope 里只有这一条的外壳与它的命令；
#   查完结局 reset-failed 的是这一条自己的 unit；外壳里的 trap 只收信号、不发；包装被外层停时的 trap 只删自己的标记、账与峰值文件；
#   slice 名只许 singlefs 开头（case 前面那句核对）：包装给它设 MemoryMax、自证收尾停的是自证自己开的 singlefs_memory_selftest_<进程号>_<序号>.slice，
#   写成 user.slice、app.slice 这类就等于给别人的进程设上限；
#   自证里 kill -9 $$ 停的是那条命令自己；TERM 风暴那一例先核 cgroup 路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope，
#   不对就一个信号都不发——弄坏开关 nocap、fallback 不开 scope，命令跑在调用方的 cgroup 里（会话里就是 SSH 会话的 session-*.scope）。
set -uo pipefail

MEMORY_CAP_HIT_EXIT=250
MEMORY_CAP_UNAVAILABLE_EXIT=251
MEMORY_ADMISSION_REFUSED_EXIT=252
TIME_LIMIT_HIT_EXIT=253
SLICE_TOTAL_HIT_EXIT=254
MEBIBYTE=$((1024 * 1024))
DEFAULT_RESERVE="20G"                 # 依据见文件头「余量」一段
DEFAULT_WAIT_SECONDS=3600             # 依据见文件头「排队」一段
DEFAULT_KILL_GRACE_SECONDS=30
BROKEN_JUDGEMENT="${RUN_WITH_MEMORY_CAP_BREAK:-}"
SCOPE_RESULT_POLL_ATTEMPTS=50        # 查 scope 结局最多查这么多次，每次隔 0.1 秒：命令刚退出时 scope 可能还没收尾
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SELF_PATH="$SCRIPT_DIRECTORY/$(basename "${BASH_SOURCE[0]}")"
SLICE="${RUN_WITH_MEMORY_CAP_SLICE:-singlefs-heavy.slice}"
RESERVE="${RUN_WITH_MEMORY_CAP_RESERVE:-$DEFAULT_RESERVE}"
SLICE_TOTAL_OVERRIDE="${RUN_WITH_MEMORY_CAP_SLICE_TOTAL:-}"
CGROUP_ROOT="${RUN_WITH_MEMORY_CAP_CGROUP_ROOT:-/sys/fs/cgroup}"
STATE_DIRECTORY="${RUN_WITH_MEMORY_CAP_STATE_DIR:-${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/singlefs-heavy-admission}"
TIME_LIMIT_SECONDS="${RUN_WITH_MEMORY_CAP_TIME_LIMIT:-}"
KILL_GRACE_SECONDS="${RUN_WITH_MEMORY_CAP_KILL_GRACE:-$DEFAULT_KILL_GRACE_SECONDS}"
WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-$DEFAULT_WAIT_SECONDS}"
# 在 scope 里跑的外壳：命令不 exec，跑完读自己 cgroup 的 memory.peak 与 memory.events 写进峰值文件（scope 一收尾 cgroup 就删了）。
# TERM / INT 设成记一笔的处理函数而不是忽略：忽略会被命令继承（exec 之后忽略的信号仍忽略），处理函数不会；
# 撞顶之后 systemd 停 scope 发的 TERM 于是只打断命令，外壳读完、写完再退，退出码照传命令的。
# 命令退出之后改成忽略 TERM / INT（后面不再起子进程，没有谁会继承它），只用 bash 的内建命令读、写（read、printf），不起 cut / cat / grep / mv：
# systemd 停 scope 时给 scope 里每个进程都发 TERM，那一刻正在跑的外部命令会被它杀掉、内建的 read 会被打断，峰值文件就写不全，
# 被总上限挤掉的那一条会被报成撞了自己的上限（2026-09-25 门禁 47 号里撞上过一次）。
# 外壳自己的 stderr 指到 /dev/null、命令的 stderr 经 fd 4 接回原处：命令被信号杀掉时 bash 会补一行「line 6: … Killed "$@"」，那一行不是命令的输出。
SCOPE_HOLDER='trap "stop_requested=1" TERM INT
: > "$1"
peak_file="$2"
cgroup_root="$3"
shift 3
exec 4>&2 2>/dev/null
"$@" 2>&4 4>&-
command_exit=$?
trap "" TERM INT
own_cgroup_line="" peak="" own_oom="" oom_kill=""
IFS= read -r own_cgroup_line < /proc/self/cgroup
own_group="$cgroup_root${own_cgroup_line#0::}"
IFS= read -r peak < "$own_group/memory.peak"
while read -r event_name event_count; do
  case "$event_name" in
    oom) own_oom="$event_count" ;;
    oom_kill) oom_kill="$event_count" ;;
  esac
done < "$own_group/memory.events"
printf "peak %s\noom %s\noom_kill %s\n" "$peak" "$own_oom" "$oom_kill" > "$peak_file"
exit "$command_exit"'

reject_usage() {
  echo "  ✗ $1" >&2
  echo "  → 怎么办：$2" >&2
  exit 2
}

report_unavailable() {
  echo "  ✗ 带内存上限的 scope 起不来：$1" >&2
  echo "  → 怎么办：$2" >&2
  exit "$MEMORY_CAP_UNAVAILABLE_EXIT"
}

check_size_syntax() { # check_size_syntax <写法> <是什么>
  [[ "$1" =~ ^[1-9][0-9]*[KMGT]?$ ]] \
    || reject_usage "$2要写成 正整数[K|M|G|T]（例 16G、512M），收到的是「$1」" "照 systemd 的 MemoryMax 写法给；默认值与依据写在文件头"
}

size_in_bytes() { # size_in_bytes <正整数[KMGT]> → 字节（1024 进制，与 systemd 相同）；写法先由 check_size_syntax 验过，超过 64 位交非 0
  local number="${1%[KMGT]}" suffix="${1##*[0-9]}" multiplier=1
  case "$suffix" in
    K) multiplier=1024 ;;
    M) multiplier=$MEBIBYTE ;;
    G) multiplier=$((MEBIBYTE * 1024)) ;;
    T) multiplier=$((MEBIBYTE * MEBIBYTE)) ;;
  esac
  ((${#number} <= 18 && number <= 9223372036854775807 / multiplier)) || return 1
  echo $((number * multiplier))
}

# slice 的总上限（字节，按 MiB 向下取整：cgroup 的 memory.max 按页取整，取整到 MiB 之后读回来与写进去的逐字相同）；
# 算不出来（MemTotal 读不到、余量比整机还大）交非 0
slice_total_bytes() {
  local memory_total_kibibytes reserve_bytes total
  if [[ -n "$SLICE_TOTAL_OVERRIDE" ]]; then
    total="$(size_in_bytes "$SLICE_TOTAL_OVERRIDE")"
  else
    memory_total_kibibytes="$(awk '$1 == "MemTotal:" {print $2}' /proc/meminfo 2>/dev/null)"
    [[ "$memory_total_kibibytes" =~ ^[0-9]+$ ]] || return 1
    reserve_bytes="$(size_in_bytes "$RESERVE")"
    total=$((memory_total_kibibytes * 1024 - reserve_bytes))
  fi
  total=$((total / MEBIBYTE * MEBIBYTE))
  ((total > 0)) || return 1
  echo "$total"
}

# 把 slice 的总上限设上、确认 cgroup 里真的是这个数：成功时设好 SLICE_TOTAL_BYTES、SLICE_DIRECTORY；失败时设 SLICE_PROBLEM、交非 0。
# 已经是这个数就不再设（每条变异都经这里，免得一条一次 D-Bus 写）。
ensure_slice() {
  local cgroup_path control_output read_back
  SLICE_PROBLEM=""
  if ! SLICE_TOTAL_BYTES="$(slice_total_bytes)"; then
    SLICE_PROBLEM="总上限算不出来：/proc/meminfo 的 MemTotal 读不到，或整机内存减余量 $RESERVE 已不剩（RUN_WITH_MEMORY_CAP_SLICE_TOTAL=${SLICE_TOTAL_OVERRIDE:-没设}）"
    return 1
  fi
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  if [[ -n "$cgroup_path" && "$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)" == "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
    return 0
  fi
  if ! control_output="$(systemctl --user set-property --runtime "$SLICE" MemoryMax="$SLICE_TOTAL_BYTES" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user set-property --runtime $SLICE MemoryMax=$SLICE_TOTAL_BYTES 失败：$control_output"
    return 1
  fi
  if ! control_output="$(systemctl --user start "$SLICE" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user start $SLICE 失败：$control_output"
    return 1
  fi
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  if [[ -z "$cgroup_path" ]]; then
    SLICE_PROBLEM="$SLICE 起了却查不到它的 cgroup（systemctl --user show -p ControlGroup 是空的）"
    return 1
  fi
  read_back="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  if [[ "$read_back" != "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_PROBLEM="设了 MemoryMax=$SLICE_TOTAL_BYTES，读回 $CGROUP_ROOT$cgroup_path/memory.max 是「${read_back:-读不到}」（cgroup 的 memory 控制器没下放给用户级 systemd？）"
    return 1
  fi
  SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
}

prepare_state_directory() {
  mkdir -p "$STATE_DIRECTORY/ledger" "$STATE_DIRECTORY/peaks" 2>/dev/null && : >> "$STATE_DIRECTORY/lock" 2>/dev/null
}

default_peak_table() {
  local common_directory
  if common_directory="$(git -C "$SCRIPT_DIRECTORY" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" \
      && [[ "$(basename "$common_directory")" == ".git" && -d "$(dirname "$common_directory")/research/scripts" ]]; then
    echo "$(dirname "$common_directory")/research/scripts/memory-peaks.tsv"
  else
    echo "$SCRIPT_DIRECTORY/memory-peaks.tsv"
  fi
}

command_key() {
  local joined
  if [[ -n "${RUN_WITH_MEMORY_CAP_KEY:-}" ]]; then
    joined="$RUN_WITH_MEMORY_CAP_KEY"
  else
    joined="$*"
    joined="$(sed -E 's/-[0-9a-f]{16}\b//g' <<<"$joined")"
  fi
  joined="${joined//$'\t'/ }"
  joined="${joined//$'\n'/ }"
  printf '%s' "${joined:0:300}"
}

# 排队、记账、峰值表与 --status 的那一半（锁、JSON、算术在 python 里写得清楚）。第一个参数是动作：admit、finish、status。
admission() {
  RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED="$STATE_DIRECTORY" RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED="$PEAK_TABLE" \
    RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED="$WAIT_SECONDS" python3 /dev/fd/3 "$@" 3<<'PY'
import contextlib, fcntl, json, os, sys, time
from datetime import datetime, timezone

MEMORY_CAP_UNAVAILABLE_EXIT = 251
MEMORY_ADMISSION_REFUSED_EXIT = 252
ADMISSION_POLL_SECONDS = 1.0            # 放不下时隔多久再判一次
WAITING_REPORT_EVERY_SECONDS = 60       # 排队时隔多久往 stderr 报一行
MEBIBYTE = 1024 * 1024
state_directory = os.environ["RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED"]
ledger_directory = os.path.join(state_directory, "ledger")
lock_path = os.path.join(state_directory, "lock")
peak_table_path = os.environ["RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED"]
broken = os.environ.get("RUN_WITH_MEMORY_CAP_BREAK", "")
PEAK_TABLE_HEADER = [
    "# 每条命令上一次实测的内存峰值：research/scripts/run-with-memory-cap.sh 建、读、写（排队时按它算「这条要的量」），别手改。",
    "# 口径：scope 里的外壳 bash 在命令退出之后读自己 cgroup 的 memory.peak，按 MiB 向上取整的字节数；含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。",
    "#   撞了这一条自己的上限的记成上限；超时、被停、被总上限挤掉的不记（量到的是半截）。只在本机成立，换机器要重量，所以不进 git（.gitignore）。",
    "# 列（制表符分隔）：峰值字节、量的时刻（UTC）、那一次的上限、键（命令的各个词用空格接起来，测试二进制名里的 16 位哈希去掉；RUN_WITH_MEMORY_CAP_KEY 可以指定）。",
]


def human(byte_count):
    if byte_count >= 1024 ** 3:
        return f"{byte_count / 1024 ** 3:.1f} GiB"
    return f"{byte_count / 1024 ** 2:.0f} MiB"


def unreclaimable_bytes(cgroup_directory):
    """cgroup 里回收不掉的用量：memory.current 减 active_file 与 inactive_file（依据见文件头「排队」一段）。
    目录不在（scope 还没起，或已经收尾删了）交 0；目录在而文件读不了交 None。"""
    try:
        with open(os.path.join(cgroup_directory, "memory.current"), encoding="utf-8") as handle:
            current = int(handle.read())
        file_pages = 0
        with open(os.path.join(cgroup_directory, "memory.stat"), encoding="utf-8") as handle:
            for line in handle:
                name, value = line.split()
                if name in ("active_file", "inactive_file"):
                    file_pages += int(value)
    except FileNotFoundError:
        return None if os.path.isdir(cgroup_directory) else 0
    except (OSError, ValueError):
        return None
    return max(0, current - file_pages)


def process_start_time(process_id):
    """/proc/<pid>/stat 的第 22 段（进程起始时刻，开机以来的时钟滴答）：进程号被复用时它不同。进程不在交 None。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8") as handle:
            return handle.read().rsplit(")", 1)[1].split()[19]
    except (OSError, IndexError):
        return None


@contextlib.contextmanager
def admission_lock():
    if broken == "nolock":
        yield
        return
    with open(lock_path, "a", encoding="utf-8") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def live_ledger_entries():
    """账上还在跑的每一笔；包装已经死了（进程号不在、或起始时刻对不上）的那一笔删掉。"""
    entries = []
    for name in sorted(os.listdir(ledger_directory)):
        if not name.endswith(".json"):
            continue
        path = os.path.join(ledger_directory, name)
        try:
            with open(path, encoding="utf-8") as handle:
                entry = json.load(handle)
        except (OSError, ValueError):
            continue   # 刚删掉的；写的时候先写临时文件再改名，读不到半截
        if broken != "keepstale" and process_start_time(entry["pid"]) != entry["pid_start_time"]:
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)
            continue
        entries.append(entry)
    return entries


def read_peak_table():
    rows = {}
    try:
        with open(peak_table_path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return rows
    for line in lines:
        fields = line.split("\t")
        if line.startswith("#") or len(fields) != 4 or not fields[0].isdigit():
            continue
        rows[fields[3]] = (int(fields[0]), fields[1], fields[2])
    return rows


def write_peak_table(rows):
    lines = PEAK_TABLE_HEADER + [f"{peak}\t{measured}\t{cap}\t{key}" for key, (peak, measured, cap) in sorted(rows.items())]
    temporary = f"{peak_table_path}.{os.getpid()}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    os.replace(temporary, peak_table_path)


def need_of(key, cap_bytes, cap_text):
    """这一条要的量与一句为什么；依据见文件头「排队」一段。"""
    if broken == "ignoretable":
        return cap_bytes, f"按它的上限 {cap_text} 算（弄坏开关 ignoretable）"
    recorded = read_peak_table().get(key)
    if recorded is None:
        if broken == "nodefault":
            return 0, "峰值表里没有这条，按 0 算（弄坏开关 nodefault）"
        return cap_bytes, f"峰值表里没有这条，按它的上限 {cap_text} 算"
    need = min(recorded[0], cap_bytes)
    return need, f"按峰值表里上一次实测的峰值 {human(recorded[0])}（{recorded[1]} 量的）算"


def occupants_text(entries, slice_directory):
    if not entries:
        return ["      （账上没有在跑的；已占的全是 slice 里别的用量，多半是刚收尾还没放干净的，或不经这个包装挂进 slice 的进程）"]
    return [f"      {entry['unit']}：要 {human(entry['need'])}，此刻占 {human(unreclaimable_bytes(os.path.join(slice_directory, entry['unit'] + '.scope')) or 0)}，"
            f"包装进程 {entry['pid']}，{entry['admitted_at']} 起跑，键「{entry['key'][:120]}」" for entry in entries]


def admit(unit, cap_bytes, cap_text, total_bytes, slice_directory, key, wrapper_pid):
    os.makedirs(ledger_directory, exist_ok=True)
    wait_seconds = float(os.environ["RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED"])
    pause_seconds = float(os.environ.get("RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE") or 0)
    with admission_lock():
        need, need_text = need_of(key, cap_bytes, cap_text)
    if need > total_bytes:
        print(f"  ✗ 内存不够排不上：这一条要 {human(need)}（{need_text}），比 slice 的总上限 {human(total_bytes)} 还大，等多久都放不下；命令没跑", file=sys.stderr)
        print("  → 怎么办：上限写小一点再跑（真要这么多就先调小余量 RUN_WITH_MEMORY_CAP_RESERVE，依据写在 research/scripts/run-with-memory-cap.sh 文件头）", file=sys.stderr)
        return MEMORY_ADMISSION_REFUSED_EXIT
    started = time.monotonic()
    last_report = None
    while True:
        with admission_lock():
            entries = live_ledger_entries()
            slice_used = unreclaimable_bytes(slice_directory)
            if slice_used is None:
                print(f"  ✗ 读不了 slice 的用量（{slice_directory} 下的 memory.current / memory.stat），判不了放不放得下；命令没跑", file=sys.stderr)
                print("  → 怎么办：systemctl --user status 看用户级 systemd 还在不在、slice 还在不在；修好再跑，别绕开排队去跑", file=sys.stderr)
                return MEMORY_CAP_UNAVAILABLE_EXIT
            reserved = 0 if broken == "noledger" else sum(
                max(0, entry["need"] - (unreclaimable_bytes(os.path.join(slice_directory, entry["unit"] + ".scope")) or 0)) for entry in entries)
            occupied = slice_used + reserved
            if occupied + need <= total_bytes:
                if pause_seconds:
                    time.sleep(pause_seconds)
                entry = {"unit": unit, "pid": wrapper_pid, "pid_start_time": process_start_time(wrapper_pid), "need": need, "cap": cap_text,
                         "key": key, "admitted_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")}
                temporary = os.path.join(ledger_directory, f".{unit}.partial")
                with open(temporary, "w", encoding="utf-8") as handle:
                    json.dump(entry, handle, ensure_ascii=False)
                os.replace(temporary, os.path.join(ledger_directory, unit + ".json"))
                if last_report is not None:
                    print(f"run-with-memory-cap: 排了 {time.monotonic() - started:.0f} 秒，起跑（{unit}）", file=sys.stderr)
                return 0
        now = time.monotonic()
        if now - started >= wait_seconds:
            print(f"  ✗ 内存不够排不上：等了 {now - started:.0f} 秒（上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS={wait_seconds:g}），slice 已占 {human(occupied)}"
                  f"（回收不掉的用量 {human(slice_used)} + 账上还没涨到量的 {human(reserved)}），这一条要 {human(need)}（{need_text}），总上限 {human(total_bytes)}；命令没跑。账上占着的：",
                  file=sys.stderr)
            for line in occupants_text(entries, slice_directory):
                print(line, file=sys.stderr)   # gate-lint:detail
            print("  → 怎么办：等上面那几条跑完再跑这一条（或把 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 设长）；这一条的上限写得比它真要的大，就写小一点；"
                  "别绕开这个包装去跑——排不上说明此刻整机放不下，硬起就是再来一次整机 OOM", file=sys.stderr)
            return MEMORY_ADMISSION_REFUSED_EXIT
        if last_report is None or now - last_report >= WAITING_REPORT_EVERY_SECONDS:
            print(f"run-with-memory-cap: 排队（已等 {now - started:.0f} 秒，至多 {wait_seconds:g} 秒）：slice 已占 {human(occupied)}，这一条要 {human(need)}"
                  f"（{need_text}），总上限 {human(total_bytes)}；账上在跑 {len(entries)} 条", file=sys.stderr)
            last_report = now
        time.sleep(ADMISSION_POLL_SECONDS)


def finish(unit, key, cap_text, peak_text):
    """scope 收尾之后：账上这一笔删掉，量到了峰值就记进峰值表。
    峰值按 MiB 向上取整记，与表里这一行记的相同就不写：整份重写要先写临时文件再改名，盘忙的时候一次要一两秒
    （2026-09-25 实测 IO 压力 full avg10 约 14% 时，写临时文件加改名 0.03–1.8 秒），而撞了上限的那几条每次记的都是上限。"""
    with contextlib.suppress(FileNotFoundError):
        os.remove(os.path.join(ledger_directory, unit + ".json"))
    if not peak_text:
        return 0
    peak = -(-int(peak_text) // MEBIBYTE) * MEBIBYTE
    try:
        with admission_lock():
            rows = read_peak_table()
            if key in rows and rows[key][0] == peak:
                return 0
            rows[key] = (peak, datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), cap_text)
            write_peak_table(rows)
    except OSError as error:
        print(f"run-with-memory-cap: 峰值没记进 {peak_table_path}（{error}），这一条的退出码照旧；下一次它按上限排队", file=sys.stderr)
    return 0


def meminfo():
    values = {}
    with open("/proc/meminfo", encoding="utf-8") as handle:
        for line in handle:
            name, rest = line.split(":", 1)
            values[name] = int(rest.split()[0]) * 1024
    return values


def cgroup_stat(path, names):
    try:
        with open(path, encoding="utf-8") as handle:
            stat = dict(line.split() for line in handle)
        return sum(int(stat.get(name, 0)) for name in names)
    except (OSError, ValueError):
        return None


def status(slice_name, total_text, reserve_text, slice_directory, cgroup_root):
    memory = meminfo()
    total_memory, available = memory["MemTotal"], memory["MemAvailable"]
    print(f"  … 整机内存 {human(total_memory)}（/proc/meminfo 的 MemTotal），此刻 MemAvailable {human(available)}")
    total_bytes = int(total_text) if total_text.isdigit() else None
    print(f"  … slice {slice_name} 的总上限 {human(total_bytes) if total_bytes else '算不出来'}（整机内存 − 余量 {reserve_text}；RUN_WITH_MEMORY_CAP_RESERVE 改余量）")
    slice_used = unreclaimable_bytes(slice_directory) if slice_directory else 0
    entries = live_ledger_entries() if os.path.isdir(ledger_directory) else []
    print(f"  … slice 里回收不掉的用量 {human(slice_used or 0)}{'' if slice_directory else '（slice 还没起）'}；账上在跑 {len(entries)} 条"
          f"{'：' if entries else ''}")
    for line in occupants_text(entries, slice_directory) if entries else []:
        print(line)
    outside = total_memory - available - (slice_used or 0)
    vllm = cgroup_stat(os.path.join(cgroup_root, "system.slice/vllm-prod.service/memory.stat"), ("anon", "shmem"))
    user = cgroup_stat(os.path.join(cgroup_root, "user.slice/memory.stat"), ("anon",))
    print(f"  … slice 外面回收不掉的合计约 {human(outside)}（MemTotal − MemAvailable − slice 里的用量），其中 vllm-prod 的 anon + shmem "
          f"{human(vllm) if vllm is not None else '读不到'}、user.slice 的 anon {human(user) if user is not None else '读不到'}")
    if total_bytes is not None:
        reserve_bytes = total_memory - total_bytes
        if outside > reserve_bytes:
            print(f"  ✗ slice 外面的用量已经超过余量 {human(reserve_bytes)}：slice 的总上限挡不住整机被外面涨满")
            print("  → 怎么办：查外面是谁在涨（systemd-cgtop、ps -eo rss,pid,cmd --sort=-rss）；是常态就调大 RUN_WITH_MEMORY_CAP_RESERVE 并改文件头的依据")
            return 1
        print(f"  ✓ 余量比 slice 外面此刻的用量多 {human(reserve_bytes - outside)}")
    return 0


operation, arguments = sys.argv[1], sys.argv[2:]
if operation == "admit":
    unit, cap_bytes, cap_text, total_bytes, slice_directory, key, wrapper_pid = arguments
    sys.exit(admit(unit, int(cap_bytes), cap_text, int(total_bytes), slice_directory, key, int(wrapper_pid)))
if operation == "finish":
    sys.exit(finish(*arguments))
if operation == "status":
    sys.exit(status(*arguments))
print(f"  ✗ admission 不认识的动作 {operation}", file=sys.stderr)
print("  → 怎么办：这是 run-with-memory-cap.sh 自己的内部调用写错了，看调 admission 的那一行", file=sys.stderr)
sys.exit(2)
PY
}

# scope 的结局：命令退出之后查 systemd 记的 Result，直到 scope 收尾（不在了、inactive、failed）或查满次数。
# 打印 Result（oom-kill、timeout、success 等）；systemctl 本身出错时打印 unreadable。
# 读到 oom-kill 也要等 scope 收尾再走：它那时多半还在 deactivating，不等它落到 failed 就 reset-failed 不掉，撞顶的 scope 会一直挂着。
scope_result() {
  local unit="$1" attempt properties load_state active_state result="" oom_seen=""
  for ((attempt = 1; attempt <= SCOPE_RESULT_POLL_ATTEMPTS; attempt++)); do
    if ! properties="$(systemctl --user show -p LoadState -p ActiveState -p Result "$unit.scope" 2>/dev/null)"; then
      echo "unreadable"
      return
    fi
    load_state="$(sed -n 's/^LoadState=//p' <<<"$properties")"
    active_state="$(sed -n 's/^ActiveState=//p' <<<"$properties")"
    result="$(sed -n 's/^Result=//p' <<<"$properties")"
    [[ "$result" == "oom-kill" ]] && oom_seen="oom-kill"
    if [[ "$load_state" == "not-found" || "$active_state" == "inactive" || "$active_state" == "failed" ]]; then
      break
    fi
    sleep 0.1
  done
  [[ -n "$oom_seen" ]] && result="$oom_seen"
  if [[ "$active_state" == "failed" ]]; then
    systemctl --user reset-failed "$unit.scope" >/dev/null 2>&1   # 撞顶、超时的 scope 停在 failed，不清掉就一直挂在用户级 systemd 里
  fi
  echo "${result:-unreadable}"
}

peak_file_value() { # peak_file_value <峰值文件> <名字> → 那一行的数；没有就空
  [[ -f "$1" ]] && awk -v wanted="$2" '$1 == wanted {print $2}' "$1"
}

# run_capped <上限> <run|check> <命令…>：check 只试起不起得来，不排队、不记峰值、不限时
run_capped() {
  local cap="$1" mode="$2" cap_bytes unit started_marker peak_file key exit_code result use_slice=1 admission_status
  local peak_bytes own_oom_count oom_kill_count scope_properties
  shift 2
  if [[ "$BROKEN_JUDGEMENT" == "nocap" ]]; then
    "$@"
    return
  fi
  cap_bytes="$(size_in_bytes "$cap")" || cap_bytes=0   # 只有 --check 走得到 0：跑命令的入口先判过上限不超过 64 位
  prepare_state_directory || report_unavailable "账与锁的目录 $STATE_DIRECTORY 建不了（mkdir 或写 lock 失败），命令一行都没跑" \
    "查 \$XDG_RUNTIME_DIR（$STATE_DIRECTORY 的上一层）在不在、可写不可写；或设 RUN_WITH_MEMORY_CAP_STATE_DIR 到一个可写的目录。不许绕开排队去跑"
  if [[ "$BROKEN_JUDGEMENT" == "noslice" ]]; then
    use_slice=0
  elif ! ensure_slice; then
    if [[ "$BROKEN_JUDGEMENT" == "slicefallback" ]]; then
      use_slice=0
    else
      report_unavailable "slice $SLICE 的总上限设不上：$SLICE_PROBLEM；命令一行都没跑" \
        "照上面那句修（systemctl --user status、cat /sys/fs/cgroup/user.slice/user-\$(id -u).slice/user@\$(id -u).service/cgroup.subtree_control 里要有 memory）；修好之前别跑这一步，不许退回无总上限去跑"
    fi
  fi
  unit="singlefs-memory-cap-$$-$RANDOM$RANDOM"
  key="$(command_key "$@")"
  if [[ "$mode" == "run" && $use_slice -eq 1 ]]; then
    admission admit "$unit" "$cap_bytes" "$cap" "$SLICE_TOTAL_BYTES" "$SLICE_DIRECTORY" "$key" "$$"
    admission_status=$?
    if [[ $admission_status -eq $MEMORY_ADMISSION_REFUSED_EXIT ]]; then
      return "$MEMORY_ADMISSION_REFUSED_EXIT"
    fi
    if [[ $admission_status -ne 0 ]]; then
      report_unavailable "排队那一步退 $admission_status（上面是它的报错），命令一行都没跑" "照上面那几行修；不许绕开排队去跑"
    fi
  fi
  started_marker="${TMPDIR:-/tmp}/run-with-memory-cap-$$-$RANDOM$RANDOM.started"
  peak_file="$STATE_DIRECTORY/peaks/$unit"
  # 被外层 timeout 连同进程组一起停的时候，也把标记、账上这一笔与峰值文件删掉
  trap 'rm -f -- "$started_marker" "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"; exit 143' TERM
  trap 'rm -f -- "$started_marker" "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"; exit 130' INT
  scope_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0 -p OOMPolicy=stop)
  ((use_slice)) && scope_properties+=(--slice="$SLICE")
  if [[ "$mode" == "run" && -n "$TIME_LIMIT_SECONDS" && "$BROKEN_JUDGEMENT" != "notimelimit" ]]; then
    scope_properties+=(-p RuntimeMaxSec="$TIME_LIMIT_SECONDS" -p TimeoutStopSec="$KILL_GRACE_SECONDS")
  fi
  # 外壳里先建标记再跑命令：标记在，说明命令确实是在上限里起的；不在，说明 systemd-run 自己没起来。
  # 外面那层把本 shell 的 stderr 临时指到 /dev/null：外壳被 SIGKILL 时 bash 会补一行「… Killed  systemd-run …」，那一行不是命令的输出；
  # 命令自己的 stderr 经 fd 3 照旧接回原来的 stderr。只管这一条的三个变量在这里清掉，不传进命令
  { env -u RUN_WITH_MEMORY_CAP_KEY -u RUN_WITH_MEMORY_CAP_TIME_LIMIT -u RUN_WITH_MEMORY_CAP_WAIT_SECONDS \
      systemd-run --user --scope --unit="$unit" "${scope_properties[@]}" --quiet \
      bash -c "$SCOPE_HOLDER" run-with-memory-cap "$started_marker" "$peak_file" "$CGROUP_ROOT" "$@" 2>&3 3>&-; } 3>&2 2>/dev/null
  exit_code=$?
  if [[ ! -e "$started_marker" ]]; then
    rm -f -- "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"
    trap - TERM INT
    if [[ "$BROKEN_JUDGEMENT" == "fallback" ]]; then
      "$@"
      return
    fi
    report_unavailable "systemd-run --user --scope 退出码 $exit_code，命令一行都没跑（上面是 systemd-run 自己的报错）" \
      "查用户级 systemd 与 D-Bus（systemctl --user status、echo \$DBUS_SESSION_BUS_ADDRESS \$XDG_RUNTIME_DIR）；起不来就别跑这一步，不许退回无上限去跑"
  fi
  rm -f -- "$started_marker"
  trap - TERM INT
  result="$(scope_result "$unit")"
  peak_bytes="$(peak_file_value "$peak_file" peak)"
  own_oom_count="$(peak_file_value "$peak_file" oom)"
  oom_kill_count="$(peak_file_value "$peak_file" oom_kill)"
  rm -f -- "$peak_file" "$peak_file.partial"
  if [[ "$mode" != "run" ]]; then
    rm -f -- "$STATE_DIRECTORY/ledger/$unit.json"
  fi
  if [[ "$BROKEN_JUDGEMENT" == "exitcode" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$peak_bytes"
    if [[ $exit_code -eq 137 || $exit_code -eq 143 ]]; then
      echo "run-with-memory-cap: 撞了内存上限 $cap（弄坏开关 exitcode：按退出码判）" >&2
      return "$MEMORY_CAP_HIT_EXIT"
    fi
    return "$exit_code"
  fi
  if [[ "$result" == "unreadable" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    report_unavailable "命令退出码 $exit_code，之后查不到 scope $unit 的结局（systemctl --user show 出错），判不了是不是撞了上限" \
      "查用户级 systemd 还在不在（systemctl --user status）；这一条的结果不算数，修好再重跑"
  fi
  if [[ "$result" == "oom-kill" && "$BROKEN_JUDGEMENT" != "noresult" ]]; then
    if ((use_slice)) && [[ "$own_oom_count" == "0" && "${oom_kill_count:-0}" != "0" && "$BROKEN_JUDGEMENT" != "noslicehit" ]]; then
      [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
      echo "run-with-memory-cap: 被 slice $SLICE 的总上限挤掉了（scope $unit 的 Result=oom-kill，而它自己没撞它的上限 $cap：整个 slice 满了，内核在 slice 里挑了这一条杀；命令退出码 $exit_code）" >&2
      echo "run-with-memory-cap: → 这一条的结果不算数，重跑它；常这样说明排队按峰值表放进来的几条实际占的比表里记的多，bash $SELF_PATH --status 看 slice 里是谁" >&2
      return "$SLICE_TOTAL_HIT_EXIT"
    fi
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$cap_bytes"
    echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2
    return "$MEMORY_CAP_HIT_EXIT"
  fi
  if [[ "$result" == "timeout" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout，systemd 停掉了 scope 里的进程；命令退出码 $exit_code）" >&2
    return "$TIME_LIMIT_HIT_EXIT"
  fi
  [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$peak_bytes"
  return "$exit_code"
}

show_status() {
  local total cgroup_path slice_directory=""
  check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
  [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
  total="$(slice_total_bytes)" || total="unknown"
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  [[ -n "$cgroup_path" ]] && slice_directory="$CGROUP_ROOT$cgroup_path"
  admission status "$SLICE" "$total" "$RESERVE" "$slice_directory" "$CGROUP_ROOT"
}

# ── 自证 ──
# 吃内存的探针：每次拿 16 MiB 写满（真占常驻页），拿到 MEBIBYTES 就停下正常退出；上限 64M 时它在半路被杀
bounded_memory_hog_code() {
  printf '%s\n' "import sys" "held = []" "for _ in range(int(sys.argv[1]) // 16):" "    held.append(b'\\x01' * (16 * 1024 * 1024))" \
    "print('allocated', len(held) * 16, 'MiB')"
}

# 排队那几例用的作业：起跑时刻写进第一个文件，等一会儿、拿够内存、再占一会儿，收尾时刻写进第二个文件
selftest_job_code() {
  printf '%s\n' "import sys, time" "start_path, end_path = sys.argv[1], sys.argv[2]" \
    "delay_seconds, mebibytes, hold_seconds = float(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5])" \
    "open(start_path, 'w').write(repr(time.time()))" "time.sleep(delay_seconds)" \
    "held = [bytes([1]) * (1024 * 1024) for _ in range(mebibytes)]" "time.sleep(hold_seconds)" \
    "open(end_path, 'w').write(repr(time.time()))"
}

run_selftest() {
  local failures=0 checked=0 scratch output status marker hog_code job_code target slice_counter=0 slice context_directory
  local first_pid second_pid first_status second_status started_seconds elapsed_seconds
  local -a selftest_slices=() context=()
  target="${RUN_WITH_MEMORY_CAP_SELFTEST_TARGET:-$SELF_PATH}"
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/run-with-memory-cap-selftest-XXXXXX")"
  hog_code="$(bounded_memory_hog_code)"
  job_code="$(selftest_job_code)"
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail
  # 每一例一个新 slice（名字不带 -：- 是 systemd 的层级）、新的账与峰值表；收尾时 stop 加 revert 掉
  fresh_context() { # fresh_context <总上限>
    slice_counter=$((slice_counter + 1))
    slice="singlefs_memory_selftest_$$_${slice_counter}.slice"
    selftest_slices+=("$slice")
    context_directory="$scratch/context-$slice_counter"
    mkdir -p "$context_directory"
    context=(RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL="$1" RUN_WITH_MEMORY_CAP_STATE_DIR="$context_directory/state"
             RUN_WITH_MEMORY_CAP_PEAKS="$context_directory/peaks.tsv")
  }
  in_context() { env "${context[@]}" "$@"; }
  intervals_overlap() { # intervals_overlap <前一条起> <前一条收> <后一条起> <后一条收>：四个文件都在、两段时间有交集才退 0
    python3 -c 'import sys; a0, a1, b0, b1 = (float(open(path).read()) for path in sys.argv[1:5]); sys.exit(0 if a0 < b1 and b0 < a1 else 1)' "$@" 2>/dev/null
  }
  all_exist() { local path; for path in "$@"; do [[ -f "$path" ]] || return 1; done; }
  seed_peak() { # seed_peak <键> <字节>：往这一例的峰值表里写一行
    printf '%s\t%s\t%s\t%s\n' "$2" "2026-09-25T00:00:00Z" "200M" "$1" >> "$context_directory/peaks.tsv"
  }

  fresh_context 512M
  checked=$((checked + 1))
  if ! in_context bash "$target" --check 64M 2>"$scratch/check.err"; then
    fail "--check 64M 在这台机器上应当起得来，实际起不来：$(cat "$scratch/check.err")"
  fi

  checked=$((checked + 1))
  if in_context bash "$target" 64M python3 -c "$hog_code" 512 >"$scratch/hog.out" 2>"$scratch/hog.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/hog.err"; then
    fail "上限 64M 里分配 512 MiB 的探针应当退 $MEMORY_CAP_HIT_EXIT 并报「撞了内存上限 64M」，实际退 $status：$(cat "$scratch/hog.out" "$scratch/hog.err")"
  fi

  # 吃内存的是孙进程、父进程被杀之后自己还要退 101：照样按 Result 判撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'python3 -c "$1" 512; exit 101' hog-parent "$hog_code" >"$scratch/grandchild.out" 2>"$scratch/grandchild.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]]; then
    fail "吃内存的是孙进程时应当照样退 $MEMORY_CAP_HIT_EXIT（按 scope 的 Result 判，不按父进程的退出码），实际退 $status：$(cat "$scratch/grandchild.err")"
  fi

  # 不是撞顶的 SIGKILL（自己 kill -9 自己）退 137，不许判成撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'kill -9 $$' 2>"$scratch/sigkill.err"; then status=0; else status=$?; fi
  if [[ $status -ne 137 || "$(cat "$scratch/sigkill.err")" == *Killed* ]]; then
    fail "上限之内被 kill -9 的命令应当原样退 137、不算撞顶，stderr 里不多出外壳补的「Killed」那一行，实际退 $status：$(cat "$scratch/sigkill.err")"
  fi

  # 正常退出码原样传回
  for wanted in 0 7 101; do
    checked=$((checked + 1))
    if in_context bash "$target" 64M bash -c "exit $wanted"; then status=0; else status=$?; fi
    [[ $status -eq $wanted ]] || fail "命令退 $wanted 时应当原样传回 $wanted，实际 $status"
  done

  # 上限真的设上了：scope 里读自己 cgroup 的 memory.max 与 memory.swap.max
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max" "$group/memory.swap.max"' 2>&1)"
  if [[ "$output" != $'67108864\n0' ]]; then
    fail "scope 里的 memory.max、memory.swap.max 应当是 67108864 与 0，实际「$output」"
  fi

  # 总上限（第 ② 层）真的设上了：scope 挂在这一例的 slice 底下，slice 的 memory.max 是 512M
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; echo "$group"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>&1)"
  if [[ "$output" != *"/$slice/"*$'\n536870912' ]]; then
    fail "scope 应当挂在 slice $slice 底下、slice 的 memory.max 是 536870912（512M），实际「$output」"
  fi

  # slice 设不上（cgroup 里读回来的不是那个数，这里用一个不存在的 cgroup 根演）：退 251、报 slice、命令没跑，不退回无总上限
  checked=$((checked + 1))
  marker="$scratch/ran-without-slice"
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/noslice.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q 'slice' "$scratch/noslice.err"; then
    fail "slice 的总上限设不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 slice、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/noslice.err")"
  fi
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "slice 的总上限设不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # 余量比整机内存还大：总上限算不出来，同样退 251、命令没跑
  checked=$((checked + 1))
  marker="$scratch/ran-with-oversized-reserve"
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= RUN_WITH_MEMORY_CAP_RESERVE=1024T bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/reserve.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]]; then
    fail "余量 1024T 比整机内存还大时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/reserve.err")"
  fi

  # systemd-run 起不来：退 251、报错，而且那条命令一行都没跑（不退回无上限）
  checked=$((checked + 1))
  marker="$scratch/ran-without-cap"
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unavailable.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '起不来' "$scratch/unavailable.err"; then
    fail "D-Bus 连不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「起不来」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unavailable.err")"
  fi
  checked=$((checked + 1))
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "D-Bus 连不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # slice 设得上、而 systemd-run 自己起不来（systemd 不认这个上限：换成字节超过 64 位）：--check 退 251，不退回无上限去跑
  checked=$((checked + 1))
  if in_context bash "$target" --check 99999999999999T 2>"$scratch/scope-refused.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'systemd-run' "$scratch/scope-refused.err"; then
    fail "slice 设得上而 systemd-run 不认上限 99999999999999T 时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 systemd-run，实际退 $status：$(cat "$scratch/scope-refused.err")"
  fi

  # 两条同时来、都判放得下（第 ③ 层的锁）：总上限 256M，两条各要 200M（峰值表里没有、按上限算），判完停 0.6 秒再记账把竞态撑开；
  # 应当一条跑完另一条才起，两条都退 0。判与记账不在一把锁里时两条都放进来、合起来 300 MiB 超总上限
  fresh_context 256M
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=0.6 bash "$target" 200M python3 -c "$job_code" "$scratch/lock-1.start" "$scratch/lock-1.end" 0.2 150 1.0 \
    >/dev/null 2>"$scratch/lock-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=0.6 bash "$target" 200M python3 -c "$job_code" "$scratch/lock-2.start" "$scratch/lock-2.end" 0.2 150 1.0 \
    >/dev/null 2>"$scratch/lock-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/lock-{1,2}.{start,end} \
      || intervals_overlap "$scratch/lock-1.start" "$scratch/lock-1.end" "$scratch/lock-2.start" "$scratch/lock-2.end"; then
    fail "两条同时来（各要 200M、总上限 256M）应当一条一条跑、都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/lock-1.start" "$scratch/lock-1.end" "$scratch/lock-2.start" "$scratch/lock-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/lock-1.err" "$scratch/lock-2.err")"
  fi

  # 前一条放行了、还没涨到量（先睡 1 秒再拿 150 MiB）时后一条来：只看 slice 的用量两条都判得下、合起来超；
  # 账上还没涨到量的那部分算进已占，后一条就等前一条跑完
  fresh_context 256M
  checked=$((checked + 1))
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/ramp-1.start" "$scratch/ramp-1.end" 1.0 150 0.8 >/dev/null 2>"$scratch/ramp-1.err" &
  first_pid=$!
  sleep 0.4
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/ramp-2.start" "$scratch/ramp-2.end" 0 150 0.8 >/dev/null 2>"$scratch/ramp-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/ramp-{1,2}.{start,end} \
      || intervals_overlap "$scratch/ramp-1.start" "$scratch/ramp-1.end" "$scratch/ramp-2.start" "$scratch/ramp-2.end"; then
    fail "前一条还没涨到量时来的后一条应当等它跑完、两条都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/ramp-1.start" "$scratch/ramp-1.end" "$scratch/ramp-2.start" "$scratch/ramp-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/ramp-1.err" "$scratch/ramp-2.err")"
  fi

  # 峰值表里没有这条：按它的上限算。前一条（表里记 100 MiB）在跑，后一条表里没有、上限 200M：100 + 200 放不进 256M，
  # 等 1 秒排不上，退 252、命令没跑、说清是按上限算的
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-recorded $((100 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-recorded bash "$target" 200M python3 -c "$job_code" "$scratch/table-1.start" "$scratch/table-1.end" 0 10 2.5 \
    >/dev/null 2>"$scratch/table-1.err" &
  first_pid=$!
  sleep 0.6
  marker="$scratch/ran-unrecorded"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-unrecorded RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M bash -c ': > "$1"' touch-marker "$marker" \
      2>"$scratch/unrecorded.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$marker" ]] || ! grep -q '排不上' "$scratch/unrecorded.err" || ! grep -q '峰值表里没有这条，按它的上限 200M 算' "$scratch/unrecorded.err"; then
    fail "峰值表里没有的那一条应当按上限 200M 算、排不上退 $MEMORY_ADMISSION_REFUSED_EXIT、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unrecorded.err")"
  fi
  [[ $first_status -eq 0 ]] || fail "表里记着 100 MiB 的那一条应当照常跑完退 0，实际退 $first_status：$(cat "$scratch/table-1.err")"

  # 要的量比总上限还大：等多久都放不下，不等，立刻退 252
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-oversized bash "$target" 512M true 2>"$scratch/oversized.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || $elapsed_seconds -ge 5 ]] || ! grep -q '比 slice 的总上限' "$scratch/oversized.err"; then
    fail "要 512M 而总上限 256M 时应当不等、立刻退 $MEMORY_ADMISSION_REFUSED_EXIT，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/oversized.err")"
  fi

  # 峰值记进表，下一次按它算：跑一条拿 30 MiB 的，表里这个键记下的峰值在 30 MiB 与上限 200M 之间；
  # 之后两条同样的同时来，各按记下的量算，都放得下、同时在跑
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small bash "$target" 200M python3 -c "$job_code" "$scratch/small-0.start" "$scratch/small-0.end" 0 30 0 \
      2>"$scratch/small-0.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-small" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$output" =~ ^[0-9]+$ ]] || ((output < 30 * MEBIBYTE || output > 200 * MEBIBYTE)); then
    fail "跑完一条拿 30 MiB 的，峰值表里 selftest-small 应当记下 30 MiB 到 200M 之间的峰值，实际退 $status、记的是「$output」：$(cat "$scratch/small-0.err")"
  fi
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small bash "$target" 200M python3 -c "$job_code" "$scratch/small-1.start" "$scratch/small-1.end" 0 30 1.2 \
    >/dev/null 2>"$scratch/small-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small bash "$target" 200M python3 -c "$job_code" "$scratch/small-2.start" "$scratch/small-2.end" 0 30 1.2 \
    >/dev/null 2>"$scratch/small-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! intervals_overlap "$scratch/small-1.start" "$scratch/small-1.end" "$scratch/small-2.start" "$scratch/small-2.end"; then
    fail "峰值表里记着约 30 MiB 的两条（上限各 200M、总上限 256M）应当按记下的量一起放行、同时在跑，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/small-1.start" "$scratch/small-1.end" "$scratch/small-2.start" "$scratch/small-2.end" && echo '同时在跑' || echo '一条一条跑的')：$(cat "$scratch/small-1.err" "$scratch/small-2.err")"
  fi

  # 外壳写峰值那一步挨得住 systemd 停 scope 时发给 scope 里每个进程的 TERM：命令在 scope 里留一个进程，等命令退出之后 1 到 2 秒里
  # 不停地（不 sleep）给 scope 里除它以外的每个进程发 TERM——外壳那时起的任何外部命令活不过一轮；峰值照样要记进表。
  # 只在这一例自己开的 scope 里发：cgroup 路径要是 …/<这一例的 slice>/singlefs-memory-cap-*.scope（slice 名经 $1 传进去），不对就不发信号、退 0。
  # 不开 scope 的路径（弄坏开关 nocap）下命令就在调用方的 cgroup 里：2026-09-25 UTC 11:12 在 SSH 会话的 session-1.scope 里发过一轮 TERM，
  # 打掉了 VSCode 扩展宿主与 Claude 会话；只核最后一段名字时，外面包着一层别的 singlefs-memory-cap scope（整条门禁经包装跑）会打到外层那一整条
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-term-storm bash "$target" 64M bash -c '
      group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"
      case "$group" in
        */"$1"/singlefs-memory-cap-*.scope) ;;
        *) echo "不在这一例自己的 slice $1 底下自己开的 scope 里（$group），不发信号" >&2; exit 0 ;;
      esac
      command_pid=$$
      ( while kill -0 "$command_pid" 2>/dev/null; do :; done
        storm_end=$((SECONDS + 2))
        while ((SECONDS < storm_end)); do
          while read -r process_id; do [[ "$process_id" != "$BASHPID" ]] && kill -TERM "$process_id" 2>/dev/null; done < "$group/cgroup.procs"  # process-safety:own-scope 上面 case 核过路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope
        done ) >/dev/null 2>&1 &
      exit 0' term-storm "$slice" 2>"$scratch/term-storm.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-term-storm" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ ! "$output" =~ ^[0-9]+$ ]]; then
    fail "scope 里一直有进程收 TERM 时，外壳照样要把峰值记进表（selftest-term-storm 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/term-storm.err")"
  fi

  # 被总上限挤掉（第 ② 层兜住、退 254，不报成撞了自己的上限）：表里把这两条记成 10 MiB，排队一起放行，实际各拿 150 MiB，合起来超总上限 256M
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-underrecorded $((10 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-underrecorded bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-1.start" "$scratch/squeeze-1.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-underrecorded bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-2.start" "$scratch/squeeze-2.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne $SLICE_TOTAL_HIT_EXIT && $second_status -ne $SLICE_TOTAL_HIT_EXIT ]] || [[ $first_status -eq $MEMORY_CAP_HIT_EXIT || $second_status -eq $MEMORY_CAP_HIT_EXIT ]] \
      || ! grep -q '总上限挤掉' "$scratch/squeeze-1.err" "$scratch/squeeze-2.err"; then
    fail "两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 $SLICE_TOTAL_HIT_EXIT、报「被总上限挤掉」，实际退 $first_status 与 $second_status：$(cat "$scratch/squeeze-1.err" "$scratch/squeeze-2.err")"
  fi

  # 限时：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT 退 253，不跑满命令自己的时长
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 4 2>"$scratch/timelimit.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $TIME_LIMIT_HIT_EXIT || $elapsed_seconds -ge 4 ]] || ! grep -q '超过限时 1 秒' "$scratch/timelimit.err"; then
    fail "限时 1 秒跑 sleep 4 应当 4 秒之内退 $TIME_LIMIT_HIT_EXIT、报「超过限时 1 秒」，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/timelimit.err")"
  fi

  # 账上包装已经死了的那一笔（进程号是 pid_max，这台机器上不会有这个进程）不算：记着要 250M 的死账不挡一条要 200M 的
  checked=$((checked + 1))
  mkdir -p "$context_directory/state/ledger"
  printf '{"unit": "singlefs-memory-cap-dead", "pid": %s, "pid_start_time": "1", "need": %s, "cap": "250M", "key": "dead", "admitted_at": "2026-09-25T00:00:00Z"}' \
    "$(cat /proc/sys/kernel/pid_max)" $((250 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-dead.json"
  if in_context env RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M true 2>"$scratch/stale.err"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || -e "$context_directory/state/ledger/singlefs-memory-cap-dead.json" ]]; then
    fail "账上那一笔的包装已经死了，应当删掉它、放行这一条退 0，实际退 $status、那一笔$([[ -e "$context_directory/state/ledger/singlefs-memory-cap-dead.json" ]] && echo '还在' || echo '删了')：$(cat "$scratch/stale.err")"
  fi

  # 收尾之后不留账、不留 scope：每一例的账是空的，每个 slice 底下没有 scope
  checked=$((checked + 1))
  output=""
  for context_directory in "$scratch"/context-*; do
    if compgen -G "$context_directory/state/ledger/*.json" >/dev/null; then output+=" $context_directory 的账没清空"; fi
  done
  for slice in "${selftest_slices[@]}"; do
    if compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope" >/dev/null; then output+=" $slice 底下还有 scope"; fi
  done
  [[ -z "$output" ]] || fail "跑完应当不留账、不留 scope，实际：$output"

  # 用法错
  checked=$((checked + 1))
  if bash "$target" abc true 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq 2 ]] || fail "上限写成 abc 应当退 2，实际 $status"
  checked=$((checked + 1))
  if bash "$target" 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq 2 ]] || fail "没给命令应当退 2，实际 $status"
  # slice 名不是 singlefs 开头（别人的 slice）：退 2，不设上限、不起 scope。改前那一份会真去设，名字取一个不碍事的，收尾照样停掉、还原
  checked=$((checked + 1))
  foreign_slice="notsinglefs_selftest_$$.slice"
  selftest_slices+=("$foreign_slice")
  if env RUN_WITH_MEMORY_CAP_SLICE="$foreign_slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/foreign-state" \
      RUN_WITH_MEMORY_CAP_PEAKS="$scratch/foreign-peaks.tsv" bash "$target" --check 64M 2>"$scratch/foreign.err"; then status=0; else status=$?; fi
  if [[ $status -ne 2 ]] || ! grep -q 'singlefs 开头' "$scratch/foreign.err"; then
    fail "slice 名写成不是 singlefs 开头的 $foreign_slice 时应当退 2、报「singlefs 开头」，实际退 $status：$(cat "$scratch/foreign.err")"
  fi

  for slice in "${selftest_slices[@]}"; do
    systemctl --user stop "$slice" >/dev/null 2>&1
    systemctl --user revert "$slice" >/dev/null 2>&1
  done
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ run-with-memory-cap.sh 自检 $failures 处不对（查了 $checked 项；测的是 $target）"
    echo "  → 怎么办：照上面逐条改 run_capped / ensure_slice / scope_result / admission；RUN_WITH_MEMORY_CAP_BREAK 设着、或 RUN_WITH_MEMORY_CAP_SELFTEST_TARGET 指着改前那一份的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ run-with-memory-cap.sh 自检通过（查了 $checked 项：--check 起得来、64M 上限里分配 512 MiB 的探针与它当孙进程时都判撞顶退 $MEMORY_CAP_HIT_EXIT、\
上限之内 kill -9 原样退 137 且 stderr 不多一行、退出码 0 / 7 / 101 原样传回、scope 里 memory.max 与 memory.swap.max 设上了、scope 挂在 slice 底下且 slice 的总上限设上了、\
slice 设不上与余量比整机还大时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、D-Bus 连不上时跑命令与 --check 都退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、systemd-run 不认上限时 --check 退 $MEMORY_CAP_UNAVAILABLE_EXIT、\
两条同时来与前一条还没涨到量时都一条一条跑、峰值表里没有的按上限算排不上退 $MEMORY_ADMISSION_REFUSED_EXIT、要的量比总上限大立刻退 $MEMORY_ADMISSION_REFUSED_EXIT、\
峰值记进表之后两条按记下的量一起跑、scope 里一直收 TERM 时峰值照记（TERM 只在这一例自己的 slice 底下自己开的 scope 里发）、被总上限挤掉退 $SLICE_TOTAL_HIT_EXIT、超过限时退 $TIME_LIMIT_HIT_EXIT、死账不挡路、跑完不留账不留 scope、\
上限写错、没给命令与 slice 名不是 singlefs 开头退 2）"
}

# slice 名只许 singlefs 开头的 .slice（文件头「发信号的地方」一段）
[[ "$SLICE" =~ ^singlefs[-_A-Za-z0-9]*\.slice$ ]] \
  || reject_usage "slice 名 RUN_WITH_MEMORY_CAP_SLICE 要写成 singlefs 开头的 .slice，收到的是「$SLICE」" \
       "不设它（默认 singlefs-heavy.slice），或写成 singlefs_<用途>.slice；别的 slice 里是别人的进程，给它设上限、停它就打到别人"
case "${1:-}" in
  --selftest)
    run_selftest
    exit 0 ;;
  --status)
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
    show_status
    exit $? ;;
  --check)
    check_size_syntax "${2:-}" "内存上限"
    check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
    [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
    run_capped "$2" check true
    exit $? ;;
  "")
    reject_usage "没给上限与命令" "写成 run-with-memory-cap.sh <上限> <命令> [参数…]，上限例 16G" ;;
esac

cap="$1"
shift
check_size_syntax "$cap" "内存上限"
size_in_bytes "$cap" >/dev/null || reject_usage "内存上限 $cap 太大，换成字节超过 64 位" "写成真要的量，例 16G"
check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
[[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
[[ -z "$TIME_LIMIT_SECONDS" || "$TIME_LIMIT_SECONDS" =~ ^[1-9][0-9]*$ ]] \
  || reject_usage "限时 RUN_WITH_MEMORY_CAP_TIME_LIMIT 要写成正整数秒，收到的是「$TIME_LIMIT_SECONDS」" "写成例 1800；不限时就别设"
[[ "$WAIT_SECONDS" =~ ^[0-9]+$ ]] || reject_usage "等待上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 要写成非负整数秒，收到的是「$WAIT_SECONDS」" "写成例 3600"
[[ $# -gt 0 ]] || reject_usage "没给要在上限里跑的命令" "写成 run-with-memory-cap.sh $cap <命令> [参数…]"
PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
run_capped "$cap" run "$@"
exit $?
```

**出处 `.claude/gate.d/57-lkmm.sh:1-30`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# gate-stage: 内存序（herd7 + litmus/，本工程自己的阶段）
#
# 上游 singlefs-ai-sop 2026-09-16 起不再跑 LKMM（移交那次的记录与删前原样在提交 fbae43e 里，`git show fbae43e:.claude/handover/qemu-herd7/README.md`），
# 从此 litmus/ 下每条 Never 有没有对照组、绑没绑到代码、herd7 判定与声明符不符，只有这一道阶段在判。
# 逻辑全在 .claude/scripts/lkmm.sh（本工程接管的那份），这里只负责把它接进门禁并给出路。
#
# 判别力样本（89 号）拿 --static-only 喂：样本目录里放一个 .lkmm-static-only 标记文件，本阶段就只跑不需要 herd7 的那几层
# （fs-design.md 五条硬要求第 2 条：只供测试的开关；静态检查全过退 3，绿样本的 expect 写 exit=3）。
#
#   bash .claude/gate.d/57-lkmm.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LKMM="$(cd "$(dirname "$0")/../scripts" && pwd)/lkmm.sh"
[[ -f "$LKMM" ]] || { echo "  ✗ 找不到 $LKMM"; echo "     → 怎么办：它随仓走（.claude/scripts/lkmm.sh），被删了就从 git 找回来。"; exit 1; }
[[ -d "$ROOT/litmus" ]] || { echo "  ! $ROOT 下没有 litmus/，本阶段跳过"; exit 77; }
static_only=()
[[ -f "$ROOT/.lkmm-static-only" ]] && static_only=(--static-only)
bash "$LKMM" "$ROOT" "${static_only[@]}"
rc=$?
if [[ ${#static_only[@]} -gt 0 ]]; then
  exit "$rc"
fi
if [[ "$rc" -ne 0 ]]; then
  echo "  ✗ 内存序判红（细节在上面 lkmm.sh 的输出里）"
  echo "     → 怎么办：改 litmus 还是改代码先想清楚：判定与声明不符时先核 singlefs-models 指的那段代码今天发的次序，"
  echo "               再决定是 litmus 没跟上代码、还是代码把屏障漏了；缺 herd7 就跑 bash .claude/scripts/fetch-deps.sh --check。"
  exit 1
fi
echo "  ✓ 内存序：litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符"
```

**出处 `.claude/scripts/lkmm.sh:1-372`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 用 LKMM 判内存序结论：herd7 给模型判定。
#
# 这是本工程自己的一份：上游 singlefs-ai-sop 2026-09-16 起不再管 herd7 / LKMM（移交那次的记录与删前原样在提交 fbae43e 里，`git show fbae43e:.claude/handover/qemu-herd7/README.md`），
# 原文取自那里的 `sop-0.0.50-snapshot/scripts/lkmm.sh`，只改了 lib.sh 的路径、`litmus/` 改指本仓 `litmus/`。
# 门禁阶段 `.claude/gate.d/57-lkmm.sh` 调它；提交前必跑（`.claude/rules/implementation-workflow.md`）。
#
#   lkmm.sh [项目根]                  跑 <项目根>/litmus/*.litmus
#   lkmm.sh [项目根] --static-only    只跑不需要 herd7 的检查；全过也退 3——它不是通过
#
# 每个 .litmus 必须在文件里声明期望判定：
#
#   (* singlefs-expect: Never *)      坏结果必须不可能发生
#   (* singlefs-expect: Sometimes *)  坏结果可能发生（对照组用）
#
# 每条 Never 还要声明它模拟的是哪段代码，一行一个锚点（写者、读者各一行也行），
# 锚点后面空一格可以写注释：
#
#   singlefs-models: crates/<crate>/src/<文件>.rs::<函数名>
#   singlefs-models: none —— <为什么它不对应代码>
#
# 判定与声明不符 → 失败。没有声明 → 失败（不许「跑了但没人看结果」）。
#
# 会失败的检查（都是踩过的坑，做成拒绝执行而不是提醒句）：
#   1. 用了 rN 却没有 `int rN;` 声明（用法只在剥掉 (* … *) 注释的正文里认）—— herd7 不管，klitmus7 会在生成 C 之后
#      才报 undeclared，那时已经很难定位。这里提前拦。
#   2. init 块里给 atomic_t 形参赋初值不带类型 —— herd7 照跑且判定正确，
#      只有 klitmus7 会炸。也就是说这个错能一路混过模型判定，必须在这里拦。
#   3. 每条 Never 必须有**自己的**对照组：同名前缀 `<名>-*.litmus`、声明 Sometimes，
#      而且**内容就是它去掉屏障的形态**（判据在 is_fence_removal_of）。
#      全局数出一条 Sometimes 不算数——对抗测试实测：10 条互不相关的 Never
#      曾靠 1 条无关的 Sometimes 全部过闸。只按文件名认也不够：换一个 exists、换一个读者的
#      「对照组」照样过闸，名字前缀撞车时（a 与 a-b）a-b 的对照还会被算成 a 的。
#      「屏障挡住了」还是「本来就撞不上」，只有同一条测试去掉屏障才回答得了。
#   4. 每条 Never 都要绑到代码。herd7 只判 litmus 写下的那个形态：代码改了发布顺序而 litmus 没跟，
#      判定照样是 Never，门禁照样绿。所以锚点指的文件与 `fn` 要在，而且 crates/ 下要有一个 .rs
#      写出这个 litmus 的文件名——那是读它、拿它跟代码今天的形态比的测试。测得对不对门禁判不了，要人看。
#   5. 没有 herd7 直接失败，不静默跳过。
#
# 1–4 都排在 herd7 探测之前：它们不需要 herd7，
# 该红的先红出来（selftest 的样本也靠这一点才喂得进去）。
source "$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/lib.sh"

STATIC_ONLY=0; ROOT=""
for argument in "$@"; do
  case "$argument" in
    --static-only) STATIC_ONLY=1 ;;
    *) ROOT="$argument" ;;
  esac
done
ROOT="${ROOT:-$(project_root)}"
# 对照组判据靠内嵌的 python3。缺了它，$(is_fence_removal_of …) 失败被当成「不匹配」，
# 于是每条 Never 都报「没有配对的对照组」——症状指着 litmus，病根在环境（审计实测）。
command -v python3 >/dev/null 2>&1 || die "缺 python3 —— 对照组「是不是去掉屏障的那一份」靠它判" \
  "装 python3（Debian/Ubuntu: sudo apt install python3）再跑；没有它时这一阶段报的原因是错的。"
# 绝对化：失败信息里的 rel 要靠 ${f#$ROOT/} 剥前缀，而 FILES 是 readlink -f 出来的绝对路径。
# ROOT 是相对路径时剥不掉，报出来的就是一长串绝对路径——看的人得自己找哪一段是项目内路径。
[[ -d "$ROOT" ]] && ROOT="$(cd "$ROOT" && pwd)"
LITMUS_DIR="$ROOT/litmus"
KTREE="${SINGLEFS_KERNEL_TREE:-}"

head1 "LKMM（herd7）"

[[ -d "$LITMUS_DIR" ]] || { bad "没有 $LITMUS_DIR 目录"
  howto "并发相关的改动要有 litmus：照 litmus/ 那对现成的抄，" \
        "放进 <项目根>/litmus/。没有并发改动就不该跑到这条检查。"; exit 1; }
# 路径必须绝对化：下面要 cd 进内核树跑 herd7（模型文件是相对路径引的），
# 相对路径 cd 之后就找不着了
mapfile -t FILES < <(find "$LITMUS_DIR" -name '*.litmus' -exec readlink -f {} \; | sort)
[[ ${#FILES[@]} -gt 0 ]] || { bad "$LITMUS_DIR 下没有 .litmus 文件"
  howto "照 litmus/ 那对现成的抄一对进来（Never + 去屏障的 Sometimes 对照），" \
        "或者删掉空的 litmus/ 目录。"; exit 1; }

# ── 静态检查：期望声明 / 寄存器声明 / atomic_t 初值类型 ──
# 寄存器用法只在剥掉注释的正文里找：头部 (* … *) 注释里写 r1、r2 不是用了它，不能逼人往 litmus 里塞一个没人用的声明。
# 注释只认行首起的那种，与 is_fence_removal_of 同一条判据：代码里的 `WRITE_ONCE(*x, 1)` 也有「(*」。
litmus_code_without_comments() { # litmus_code_without_comments <litmus 文件>
  python3 - "$1" <<'PY'
import re
import sys

text = open(sys.argv[1], encoding='utf-8').read()
sys.stdout.write(re.sub(r'^[ \t]*\(\*.*?\*\)', '', text, flags=re.S | re.M))
PY
}
fails=0
declare -A EXPECT
for f in "${FILES[@]}"; do
  rel="${f#"$ROOT"/}"
  exp="$(sed -n 's/.*singlefs-expect:[[:space:]]*\([A-Za-z]*\).*/\1/p' "$f" | head -1)"
  case "$exp" in
    Never|Sometimes|Always) EXPECT["$f"]="$exp" ;;
    *) bad "$rel 缺 (* singlefs-expect: Never|Sometimes *) 声明"
       howto "在文件头注释块里写明期望判定。没有声明 = 跑了但没人看结果，不算验证。"
       fails=$((fails+1)); continue ;;
  esac
  if ! code_without_comments="$(litmus_code_without_comments "$f")"; then
    bad "$rel 剥不掉注释（python3 读它失败），寄存器声明没法查"
    howto "看上面 python3 的报错：多半是文件不是 UTF-8。照 litmus/ 现有文件的编码存一遍再跑。"
    fails=$((fails+1)); continue
  fi
  for r in $(printf '%s\n' "$code_without_comments" | grep -oE '\br[0-9]+\b' | sort -u); do
    grep -qE "^[[:space:]]*int[[:space:]]+$r[[:space:]]*;" <<<"$code_without_comments" \
      || { bad "$rel 用了 $r 但没有 'int $r;'（klitmus7 会在生成 C 之后才报）"
           howto "在用到 $r 的进程体开头补一行 'int $r;'。"
           fails=$((fails+1)); }
  done
  # 声明、init 块与 atomic_t 形参同样只在剥掉注释的正文里找：注释里单独一行 `int r2;` 不是声明过
  init="$(awk '/^\{/{f=1} f{print} f&&/\}/{exit}' <<<"$code_without_comments")"
  for v in $(grep -oE 'atomic_t[[:space:]]*\*[[:space:]]*[A-Za-z_][A-Za-z0-9_]*' <<<"$code_without_comments" \
             | sed -E 's/.*\*[[:space:]]*//' | sort -u); do
    printf '%s\n' "$init" | grep -qE "(^|[^[:alnum:]_])$v[[:space:]]*=" || continue
    printf '%s\n' "$init" | grep -qE "atomic_t[[:space:]]+$v[[:space:]]*=" \
      || { bad "$rel init 里 '$v = ...' 要写成 'atomic_t $v = ...'（只有 klitmus7 会炸）"
           howto "init 块里给 atomic_t 形参赋初值必须带类型，照 litmus/ 的写法。"
           fails=$((fails+1)); }
  done
done
[[ $fails -eq 0 ]] || { bad "静态检查未过：$fails 项"; exit 1; }   # gate-lint:summary

# ── 对照组是不是「这一条去掉屏障的形态」──
# 判据只写在这一处。去掉行首起的 (* … *) 注释、首行 `C <名>` 与空行之后逐行比，比的时候不看空白：
# 对照组只许比原测试少几行屏障，或者把 smp_store_release / smp_load_acquire 放宽成
# WRITE_ONCE / READ_ONCE，至少一处；exists、init、读者一个字都不许改。
# 注释只认行首起的那种：代码里的 `WRITE_ONCE(*x, 1)` 也有「(*」，当成注释开头会吞掉整段代码。
# 是 → 退 0；不是 → 退 1，stdout 打印第一处不同。
is_fence_removal_of() { # is_fence_removal_of <Never 那条> <候选对照组>
  python3 - "$1" "$2" <<'PY'
import re
import sys

FENCE_STATEMENT = re.compile(
    r'^(smp_mb|smp_wmb|smp_rmb|smp_mb__before_atomic|smp_mb__after_atomic'
    r'|smp_mb__after_spinlock|smp_mb__after_unlock_lock|synchronize_rcu)\(\);$')


def statements(path):
    text = open(path, encoding='utf-8').read()
    # 注释换成同样多的换行，报出来的行号才对得上原文件
    text = re.sub(r'^[ \t]*\(\*.*?\*\)', lambda comment: '\n' * comment.group(0).count('\n'),
                  text, flags=re.S | re.M)
    kept = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        shown = line.strip()
        if shown:
            kept.append((line_number, shown, re.sub(r'\s+', '', shown)))
    if kept and kept[0][1].startswith('C '):
        kept = kept[1:]
    return kept


def relaxed(compact):
    compact = re.sub(r'smp_store_release\((\w+),', r'WRITE_ONCE(*\1,', compact)
    return re.sub(r'smp_load_acquire\((\w+)\)', r'READ_ONCE(*\1)', compact)


never_statements = statements(sys.argv[1])
control_statements = statements(sys.argv[2])
never_position = control_position = weakened_count = 0
while never_position < len(never_statements):
    never_number, never_shown, never_compact = never_statements[never_position]
    control_statement = (control_statements[control_position]
                         if control_position < len(control_statements) else None)
    if control_statement and control_statement[2] == never_compact:
        never_position += 1
        control_position += 1
    elif FENCE_STATEMENT.match(never_compact):
        weakened_count += 1
        never_position += 1
    elif (control_statement and relaxed(never_compact) != never_compact
          and relaxed(never_compact) == control_statement[2]):
        weakened_count += 1
        never_position += 1
        control_position += 1
    else:
        where_in_control = (f'对照组第 {control_statement[0]} 行「{control_statement[1]}」'
                            if control_statement else '对照组已经到头')
        print(f'第一处不同：原测试第 {never_number} 行「{never_shown}」，{where_in_control}')
        sys.exit(1)
if control_position < len(control_statements):
    extra_number, extra_shown = control_statements[control_position][:2]
    print(f'对照组第 {extra_number} 行「{extra_shown}」在原测试里没有')
    sys.exit(1)
if weakened_count == 0:
    print('一处屏障都没去掉，内容与原测试相同')
    sys.exit(1)
PY
}

# ── 每条 Never 都要有自己的对照组 ──
n_never=0; n_some=0
for f in "${FILES[@]}"; do
  [[ "${EXPECT[$f]}" == Never ]] && n_never=$((n_never+1))
  [[ "${EXPECT[$f]}" == Sometimes ]] && n_some=$((n_some+1))
done
for f in "${FILES[@]}"; do
  [[ "${EXPECT[$f]}" == Never ]] || continue
  base="${f%.litmus}"; rel="${f#"$ROOT"/}"
  paired=0; rejected_candidates=()
  for g in "${FILES[@]}"; do
    [[ "$g" == "$base"-*.litmus && "${EXPECT[$g]}" == Sometimes ]] || continue
    if difference="$(is_fence_removal_of "$f" "$g")"; then paired=1; break; fi
    rejected_candidates+=("$(basename "$g")：$difference")
  done
  [[ $paired -eq 1 ]] && continue
  fails=$((fails+1))
  if [[ ${#rejected_candidates[@]} -eq 0 ]]; then
    bad "$rel  没有配对的对照组"
    say "        全局有几条 Sometimes 不算数：对照必须是这一条去掉屏障的形态，"
    say "        否则分不清「屏障挡住了」还是「这个模式本来就撞不上」。"
    howto "复制 $(basename "$f") 为 $(basename "$base")-nofence.litmus，删掉里面的屏障" \
          "（smp_wmb / smp_rmb 等），声明改成 (* singlefs-expect: Sometimes *)。" \
          "它判 Sometimes，原来那条的 Never 才有判别力。"
  else
    bad "$rel  没有配对的对照组：同名前缀的 Sometimes 都不是它去掉屏障的形态"
    for rejected_candidate in "${rejected_candidates[@]}"; do say "        $rejected_candidate"; done
    howto "对照组只许比原测试少几行屏障（smp_wmb / smp_rmb / smp_mb 等），或者把 smp_store_release /" \
          "smp_load_acquire 放宽成 WRITE_ONCE / READ_ONCE；exists、init、读者一个字都不许改——改了它回答的就是另一个问题。" \
          "从 $(basename "$f") 复制一份重新删屏障，别在旧的对照组上修。"
  fi
done
[[ $fails -eq 0 ]] || { bad "对照组检查未过：$fails 项"; exit 1; }   # gate-lint:summary

# ── 每条 Never 都要绑到代码 ──
n_bound=0; n_unbound=0
for f in "${FILES[@]}"; do
  [[ "${EXPECT[$f]}" == Never ]] || continue
  rel="${f#"$ROOT"/}"; litmus_name="$(basename "$f")"
  declarations=()
  while IFS= read -r declaration; do declarations+=("$declaration"); done \
    < <(sed -n 's/.*singlefs-models:[[:space:]]*//p' "$f" | sed -E 's/[[:space:]]*\*\)[[:space:]]*$//; s/[[:space:]]+$//')
  if [[ ${#declarations[@]} -eq 0 ]]; then
    bad "$rel  缺 singlefs-models 声明：说不出它模拟的是哪段代码"
    howto "在文件头注释块里写一行 singlefs-models: crates/<crate>/src/<文件>.rs::<函数名>，写者、读者各一行也行；" \
          "不对应任何代码就写 singlefs-models: none —— <为什么>。"
    fails=$((fails+1)); continue
  fi
  none_count=0; anchor_count=0; anchor_failed=0; none_reason=""
  for declaration in "${declarations[@]}"; do
    if [[ "$declaration" =~ ^none([^A-Za-z0-9_].*)?$ ]]; then
      none_count=$((none_count+1))
      none_reason="$(printf '%s' "${BASH_REMATCH[1]}" | sed -E 's/^[[:space:]—–:：-]+//')"
      continue
    fi
    anchor_count=$((anchor_count+1))
    anchor="${declaration%%[[:space:]]*}"      # 锚点后面空一格可以写注释
    anchor_path="${anchor%::*}"; anchor_function="${anchor##*::}"
    if [[ "$anchor" != *::* || -z "$anchor_path" || ! "$anchor_function" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
      bad "$rel  singlefs-models 写成了「$declaration」：认不出文件与函数"
      howto "格式： singlefs-models: <仓内路径>::<函数名>，例如 crates/core/src/transaction.rs::publish；" \
            "不对应代码就写 singlefs-models: none —— <为什么>。"
      anchor_failed=1; continue
    fi
    if [[ ! -f "$ROOT/$anchor_path" ]]; then
      bad "$rel  singlefs-models 指的 $anchor_path 不存在"
      howto "代码搬了家或改了名，就跟着改这一行；那段同步逻辑整个删了，这条 litmus 也该跟着删或改写。"
      anchor_failed=1; continue
    fi
    if ! grep -qE "(^|[^A-Za-z0-9_])fn[[:space:]]+$anchor_function([^A-Za-z0-9_]|$)" "$ROOT/$anchor_path"; then
      bad "$rel  $anchor_path 里没有 fn $anchor_function"
      howto "函数改了名就跟着改这一行。改名多半也动了同步的形态，回头对一遍 litmus 的写者和读者。"
      anchor_failed=1; continue
    fi
  done
  if [[ $anchor_failed -eq 1 ]]; then fails=$((fails+1)); continue; fi
  if [[ $none_count -gt 0 && $anchor_count -gt 0 ]]; then
    bad "$rel  singlefs-models 既写了 none 又写了锚点"
    howto "二选一：它对应代码，就删掉 none 那一行；不对应，就删掉锚点。"
    fails=$((fails+1)); continue
  fi
  if [[ $none_count -gt 0 ]]; then
    if [[ -z "$none_reason" ]]; then
      bad "$rel  singlefs-models: none 没写理由"
      howto "写成 singlefs-models: none —— <为什么它不对应代码>。理由不许省：" \
            "不对应代码的 Never 证明的只是一个抽象形态，读的人要知道它凭什么可以不对应。"
      fails=$((fails+1)); continue
    fi
    n_unbound=$((n_unbound+1)); continue
  fi
  # 锚点都在，还要有一个测试读这个 litmus。门禁按文件名认它
  if ! grep -rqF --include='*.rs' -- "$litmus_name" "$ROOT/crates" 2>/dev/null; then
    bad "$rel  crates/ 下没有一个 .rs 写出 $litmus_name —— 没有测试把它和代码对上"
    howto "写一个测试读这个 litmus，拿它的写者 / 读者次序跟代码今天实际发出的次序比" \
          "（例：录制写请求流，按步骤分类后逐项比）。测试里要写出文件名 $litmus_name，门禁按文件名认。" \
          "herd7 只判 litmus 写下的形态：代码改了顺序而 litmus 没跟，判定照样是 Never。"
    fails=$((fails+1)); continue
  fi
  n_bound=$((n_bound+1))
done
[[ $fails -eq 0 ]] || { bad "代码绑定检查未过：$fails 项"; exit 1; }   # gate-lint:summary

if [[ $STATIC_ONLY -eq 1 ]]; then
  say ""
  warn "静态检查全过：$n_never 条 Never 每条都有内容对得上的对照组，$n_bound 条绑到代码、$n_unbound 条声明不对应代码；共 $n_some 条 Sometimes"
  warn "herd7 判定没跑（--static-only），退出码 3——这不是通过"
  exit 3
fi

# ── herd7 ──
if ! command -v herd7 >/dev/null 2>&1 && command -v opam >/dev/null 2>&1; then
  export OPAMROOT="${OPAMROOT:-$HOME/.opam}"
  eval "$(opam env --root="$OPAMROOT" --set-root 2>/dev/null)" || true
fi
command -v herd7 >/dev/null 2>&1 || {
  bad "herd7 缺失"
  howto "opam install herdtools7" \
        "（装完若命令仍找不到，先 eval \"\$(opam env)\"）"
  exit 1
}

# ── 内核树（herd7 要在 tools/memory-model 里跑，模型文件是相对路径引的）──
if [[ -z "$KTREE" ]]; then
  for c in "$ROOT/../linux" "$HOME/linux" "$HOME/linux-bug-fix/linux"; do
    [[ -d "$c/tools/memory-model" ]] && { KTREE="$c"; break; }
  done
fi
# 按需取：委托给 fetch-deps.sh，取树的逻辑只有那一份实现
if [[ -z "$KTREE" || ! -d "$KTREE/tools/memory-model" ]]; then
  CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/singlefs/linux-memory-model"
  if [[ -d "$CACHE/tools/memory-model" ]]; then
    KTREE="$CACHE"
  elif [[ -z "${SINGLEFS_NO_FETCH:-}" ]]; then
    bash "$(dirname "${BASH_SOURCE[0]}")/fetch-deps.sh" --kernel || exit 1
    [[ -d "$CACHE/tools/memory-model" ]] && KTREE="$CACHE"
  fi
fi

[[ -n "$KTREE" && -d "$KTREE/tools/memory-model" ]] || {
  bad "找不到带 tools/memory-model 的内核树"
  howto "herd7 要在内核树的 tools/memory-model 里跑（模型文件是相对路径引的）。" \
        "指一棵 Linux 源码树：" \
        "SINGLEFS_KERNEL_TREE=/path/to/linux bash .claude/scripts/lkmm.sh"
  exit 1
}
KTREE="$(readlink -f "$KTREE")"
ok "内核树 $KTREE"
ok "herd7  $(herd7 -version 2>&1 | head -1)"

# ── 跑 ──
say ""
cd "$KTREE/tools/memory-model"
for f in "${FILES[@]}"; do
  rel="litmus/$(basename "$f")"   # 此时已 cd 进内核树，不能再算相对路径
  want="${EXPECT[$f]}"
  out="$(timeout 300 herd7 -conf linux-kernel.cfg "$f" 2>&1)" || {
    bad "$rel  herd7 执行失败"; printf '%s\n' "$out" | tail -5 | sed 's/^/        /'
    howto "看上面的报错。多半是 litmus 语法问题，照 litmus/ 现有文件的格式改；" \
          "超时（300s）则是状态空间太大，把进程数或变量数减下来。"
    fails=$((fails+1)); continue
  }
  got="$(printf '%s\n' "$out" | sed -n 's/^Observation[[:space:]]\+[^[:space:]]\+[[:space:]]\+\([A-Za-z]*\).*/\1/p' | head -1)"
  if [[ -z "$got" ]]; then
    bad "$rel  读不到 Observation 行 —— 判定不明，整条作废"
    printf '%s\n' "$out" | tail -5 | sed 's/^/        /'; fails=$((fails+1))
    howto "多半是 litmus 语法错。最常见的一处：进程签名后面不能跟行内注释，" \
          "注释只能写在文件头的 (* ... *) 块里。照 litmus/ 现有文件的格式改。"
  elif [[ "$got" == "$want" ]]; then
    ok "$rel  $got（符合声明）"
  else
    bad "$rel  期望 $want，实际 $got"
    printf '%s\n' "$out" | grep -E '^(States|Condition|Observation)' | sed 's/^/        /' || true
    howto "两种可能，别急着改声明：" \
      "① 代码或模型真的少了屏障 → 这正是这条测试要抓的东西，去补屏障" \
      "② 这条声明本来就写错了   → 改 (* singlefs-expect: ... *)" \
      "先想清楚是哪一种。直接把声明改成实际值，等于把测试关掉。"
    fails=$((fails+1))
  fi
done

say ""
[[ $fails -eq 0 ]] || { bad "LKMM 未通过：$fails 项"; exit 1; }   # gate-lint:summary
ok "LKMM 通过（$n_never 条 Never：每条都有内容对得上的对照组，$n_bound 条绑到代码、$n_unbound 条声明不对应代码；共 $n_some 条 Sometimes）"
```

**出处 `research/scripts/replay.sh:1-758`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 复跑已入库的实验，把今天的输出和 research/results/ 里那份逐字节比对。
#
#   bash research/scripts/replay.sh [实验号...]      # 不给参数就全跑
#
# 为什么要有它：kb-discipline.md「所有历史数据都只是参考」——每条实测都绑在当时那个
# 构建上，要拿它支撑新结论，先复跑一遍确认它今天还成立。手工比对会被抄第二遍，所以做成脚本。
#
# 三条完整性闸（command-safety.md「结果抓取要有完整性闸」）：
#   1. 跑前删本轮输出，不许跨轮复用；
#   2. 收尾行必须是 `name=done emitted=N`，且 N 必须等于本轮实际 E7RESULT 行数
#      （口径：emitted 把 config 行与 done 行自己都算进去，2026-08-29 对 8 份入库产物逐份核过）；
#   3. 退出码非 0 一律判红，不许当成「跑过了」。
set -uo pipefail
cd "$(dirname "$0")/.."
OUT_DIR="${REPLAY_OUT:-${TMPDIR:-/tmp}/singlefs-replay-$$}"
mkdir -p "$OUT_DIR"
# E9 要一个真设备/文件当后端（O_DIRECT）。镜像一律落临时目录，不许进仓
# （command-safety.md「测试镜像一律放临时目录」）。
REPLAY_DEV="${REPLAY_DEV:-$OUT_DIR/e9.img}"
# e9-keylayout 用 O_DIRECT 打开这个后端，自己不建文件。tmpfs 不支持 O_DIRECT，
# 所以 REPLAY_OUT 要落在真文件系统上（本机 /tmp 是 ext4，够用）。
[[ -e "$REPLAY_DEV" ]] || truncate -s 512M "$REPLAY_DEV"
export REPLAY_DEV
# E44 要一个 64 MiB 的 O_DIRECT 后端量本机 fsync 率
REPLAY_DEV45="${REPLAY_DEV45:-$OUT_DIR/e45.img}"
[[ -e "$REPLAY_DEV45" ]] || truncate -s 64M "$REPLAY_DEV45"
export REPLAY_DEV45
# ⚠️ **E58 的测试区不许预先 truncate 出来。** 稀疏洞读起来可能根本不碰设备，
# 而 e58 只看文件大小决定填不填 ⇒ 预创建一个空洞文件会让它去量「读洞要多久」。
# 交给二进制自己建、自己填。真读错了阳性对照会判红（读洞时 read_bytes 是 0）。
REPLAY_DEV58="${REPLAY_DEV58:-$OUT_DIR/e58.img}"
export REPLAY_DEV58
# E140 与 E58 同一套装置（8 GiB O_DIRECT 测试区，二进制自己建、自己填），同样不许预先 truncate。
REPLAY_DEV140="${REPLAY_DEV140:-$OUT_DIR/e140.img}"
export REPLAY_DEV140

# 实验号 | 二进制 | 参数 | 入库产物 | 判据（exact=应逐字节一致 / timing=含计时字段，只比结构）
TABLE=$(cat <<'TSV'
E14|e14-discrimination||e14-discrimination-2026-08-29.out|exact
E18|e18-branch||e18-branch-2026-08-28.out|exact
E19|e19-defer||e19-defer-2026-08-28.out|exact
E23|e23_journal_geom||e23-journal-geom-2026-08-29.out|exact
E24|e24_recovery||e24-recovery-2026-08-29.out|exact
E25|e25_journal_reserve||e25-journal-reserve-2026-08-29.out|exact
E26|e26_accounting||e26-accounting-2026-08-29.out|exact
E27|e27_snapshot_accounting_risk_paths||e27-d5-paths-2026-08-29.out|exact
E28|e28_map_rebuild||e28-map-rebuild-2026-08-29.out|exact
E29|e29_blast_radius||e29-blast-radius-2026-08-29.out|exact
E30|e30_range_rebuild||e30-range-rebuild-2026-08-29.out|exact
E31|e31-aad-snapshot||e31-aad-snapshot-2026-08-29.out|exact
E32|e32-journal-timeline||e32-journal-timeline-2026-08-29.out|exact
E33|e33-pin-rules||e33-pin-rules-2026-08-29.out|exact
E35|e35-head-forms||e35-head-forms-2026-08-29.out|exact
E36|e36-slot-mapping||e36-slot-mapping-2026-08-29.out|exact
E37|e37-log-epoch||e37-log-epoch-2026-08-29.out|exact
E38|e38_accounting_copy_on_write||e38-accounting-cow-2026-08-29.out|exact
E39|e39_back_chain||e39-back-chain-2026-08-29.out|exact
E42|e42_transaction_records||e42-txn-records-2026-08-29.out|exact
E44|e44_jsn_width|$REPLAY_DEV45|e44-jsn-width-2026-08-30.out|timing
E58|e58-csum-grain|$REPLAY_DEV58 1 none 4096 8192|e58-csum-grain-repro-2026-09-16.out|timing
E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192|e140-header-alignment-repro-2026-09-13.out|timing
E43|e43_extension_point_budget||e43-ext-budget-2026-09-24-h311.out|exact
E41|e41_root_ring_geom||e41-root-ring-geom-2026-08-30.out|exact
E71|e71-accounting-keys||e71-accounting-keys-2026-09-01.out|exact
E75|e75-record-size||e75-record-size-2026-09-01.out|exact
E76|e76-payload-checksum||e76-payload-csum-2026-09-01.out|exact
E77|e77-publish-order||e77-publish-order-2026-09-02.out|exact
E78|e78-replay-start||e78-replay-start-2026-09-02.out|exact
E79|e79-root-record||e79-root-record-2026-09-06.out|exact
E112|e112-old-writer-unknown-tree||e112-old-writer-unknown-tree-2026-09-06.out|exact
E113|e113-unknown-tree-full-arms||e113-unknown-tree-full-arms-2026-09-06.out|exact
E114|e114-pack-ledger||e114-pack-ledger-2026-09-12.out|exact
E117|e117-reserved-header||e117-reserved-header-2026-09-12.out|exact
E118|e118-single-disk-recovery||e118-single-disk-recovery-2026-09-07.out|exact
E122|e122-directory-locality||e122-dir-locality-2026-09-07.out|exact
E116|e116-pack-settle||e116-pack-settle-2026-09-24-h311.out|exact
E119|e119-slot-tiers||e119-slot-tiers-2026-09-12.out|exact
E120|e120-tier-ratio||e120-tier-ratio-2026-09-12.out|exact
E121|e121-capacity-tiers||e121-cap-tiers-2026-09-12.out|exact
E115|e115-system-configuration-completeness||e115-system-configuration-completeness-2026-09-07.out|exact
E80|e80-partial-stripe||e80-partial-stripe-2026-09-02.out|exact
E81|e81-commit-fixpoint||e81-commit-fixpoint-2026-09-02.out|exact
E82|e82-admission-overlay||e82-admission-overlay-2026-09-02.out|exact
E83|e83-tombstone-grain||e83-tombstone-grain-2026-09-03.out|exact
E84|e84-tombstone-pinning||e84-tombstone-pinning-2026-09-03.out|exact
E85|e85-unit-header||e85-unit-header-2026-09-02.out|exact
E86|e86-scan-step||e86-scan-step-2026-09-02.out|exact
E87|e87-fixed-placement||e87-fixed-placement-2026-09-02.out|exact
E88|e88-impostor-orphan||e88-impostor-orphan-2026-09-02.out|exact
E89|e89-interval-frontier||e89-interval-frontier-2026-09-03.out|exact
E90|e90-tree-aad||e90-tree-aad-2026-09-03.out|exact
E91|e91-ring-admission||e91-ring-admission-2026-09-03.out|exact
E92|e92-reuse-requirement||e92-reuse-requirement-2026-09-08.out|exact
E123|e123-reuse-window-versus-rollback-depth||e123-k-fork-cost-2026-09-09.out|exact
E124|e124-system-configuration-recompute||e124-system-configuration-recompute-2026-09-09.out|exact
E126|e126-system-configuration-slot-width||e126-system-configuration-slot-width-2026-09-09.out|exact
E127|e127-group-identity-under-split-merge||e127-group-identity-under-split-merge-2026-09-13-knobs.out|exact
E93|e93-aging-placement||e93-aging-placement-2026-09-03.out|exact
E95|e95-node-layout-arms||e95-node-layout-arms-2026-09-09.out|exact
E94|e94-move-touchset||e94-move-touchset-2026-09-03.out|exact
E96|e96-hybrid-consistency||e96-hybrid-consistency-2026-09-03.out|exact
E97|e97-entry-encoding||e97-entry-encoding-2026-09-07.out|exact
E98|e98-inode-record||e98-inode-record-2026-09-07.out|exact
E99|e99-writebuffer-sequence||e99-writebuffer-seq-2026-09-03.out|exact
E100|e100-system-configuration-slot||e100-system-configuration-slot-2026-09-03.out|exact
E101|e101-node-tag-reserve||e101-node-tag-reserve-2026-09-03.out|exact
E102|e102-unit-class-registry||e102-unit-class-registry-2026-09-12.out|exact
E103|e103-inode-update-cost||e103-inode-update-cost-2026-09-14-round2.out|exact
E104|e104-current-version||e104-current-version-2026-09-05.out|exact
E105|e105-extent-leaf-packed||e105-extent-leaf-packed-2026-09-12.out|exact
E106|e106-stripe-member-table||e106-stripe-member-table-2026-09-12.out|exact
E108|e108-plaintext-layer-cost||e108-plaintext-layer-cost-2026-09-06.out|exact
E107|e107-stripe-table-wa||e107-stripe-table-wa-2026-09-06.out|timing
E109|e109-position-authority||e109-position-authority-2026-09-06.out|exact
E110|e110-stripe-table-steady||e110-stripe-table-steady-2026-09-06.out|exact
E111|e111-stripe-table-key||e111-stripe-table-key-2026-09-06.out|exact
E34|e34-ring-iomin||e34-ring-iomin-2026-09-01.out|exact
E73|e73-key-range||e73-key-range-2026-09-07.out|exact
E74|e74-allocation-records||e74-alloc-records-2026-09-01.out|exact
E40|e40_checksum_width||e40-csum-width-2026-08-30.out|exact
E46|e46_region_spacing||e46-region-spacing-2026-08-30.out|exact
E47|e47_ring_loss||e47-ring-loss-2026-08-30.out|exact
E48|e48_ring_placement||e48-ring-placement-2026-08-30.out|exact
E50|e50_ring_slots||e50-ring-slots-2026-08-30.out|exact
E49|e49_chain_width||e49-chain-width-2026-09-25.out|exact
E51|e51_chain_chances||e51-chain-chances-2026-08-30.out|exact
E52|e52_head_mechanisms||e52-head-mechanisms-2026-08-30.out|exact
E54|e54_accounting_generations||e54-accounting-gen-2026-08-30.out|exact
E57|e57_field_authority||e57-field-authority-2026-08-31.out|exact
E59|e59_message_recompute||e59-msg-recompute-2026-08-31.out|exact
E61|e61-chain-hash||e61-chain-hash-2026-08-31.out|exact
E69|e69-backref-cost||e69-backref-cost-2026-08-31.out|exact
E60|e60-rebalance||e60-rebalance-2026-08-31.out|exact
E70|e70-ckpt-thresholds||e70-ckpt-thresholds-2026-08-31.out|exact
E62|e62-ring-home||e62-ring-home-2026-08-31.out|exact
E63|e63-width-rule||e63-width-rule-2026-08-31.out|exact
E67|e67-device-subset||e67-device-subset-2026-08-31.out|exact
E68|e68-inline-threshold||e68-inline-threshold-2026-08-31.out|exact
E8|e8-split||e8-split-2026-08-28.out|exact
E9|@driver_e9||e9-keylayout-2026-08-28.out|exact
E16|e16-journal||e16-journal-2026-08-31.out|exact
E16|e16-journal|bytes|e16-bytes-2026-09-03.out|exact
E17|e17-merge||e17-merge-2026-08-29-repro.out|timing
E20|e20-fanout||e20-poscontrol-2026-08-29.out|timing
E21|e21-cpu|2048 5|e21-cpu-2026-08-28.out|timing
E128|e128-pointer-birth-cost|2000000|e128-pointer-birth-cost-2026-09-10.out|timing
E130|e130_livelist_bounded_destroy||e130-livelist-bounded-destroy-2026-09-10.out|exact
E131|e131_livelist_carrier||e131-livelist-carrier-2026-09-16.out|exact
E132|e132_livelist_carrier_recount||e132-livelist-carrier-recount-2026-09-16.out|exact
E133|e133_map_key_format_cost||e133-map-key-format-cost-2026-09-11.out|exact
E134|e134_map_key_slot_baselines||e134-map-key-slot-baselines-2026-09-11.out|exact
E136|e136_fork_cost_rows||e136-fork-cost-rows-2026-09-11.out|exact
E138|e138_per_disk_floor||e138-per-disk-floor-2026-09-11.out|exact
E139|e139_tightened_floor||e139-tightened-floor-2026-09-12.out|exact
E141|e141_switch_reserve_mount_admission||e141-switch-reserve-mount-admission-2026-09-14-row-writing.out|exact
E142|@driver_e142||e142-first-txn-dry-run-2026-09-25-r16-combined.out|exact
E143|e143-one-unit-per-txn-journal||e143-one-unit-per-txn-journal-2026-09-13.out|exact
E145|e145-self-describing-node-header||e145-self-describing-node-header-2026-09-16-tree-table-200.out|exact
E146|e146-livelist-entry-width||e146-livelist-entry-width-2026-09-16-tree-table-200.out|exact
E147|e147-system-configuration-recompute-from-layout||e147-system-configuration-recompute-from-layout-2026-09-13.out|exact
E148|e148-commit-fixpoint-two-record-trees||e148-commit-fixpoint-two-record-trees-2026-09-13.out|exact
E150|e150-rollback-reuse-of-abandoned-roots||e150-rollback-reuse-of-abandoned-roots-2026-09-13-admission.out|exact
E151|e151-arrival-and-container-arms||e151-arrival-and-container-arms-2026-09-13-region.out|exact
E149|e149-pack-container-repair-options||e149-pack-container-repair-options-2026-09-13.out|exact
E144|e144-header-checksum-cost||e144-header-checksum-cost-2026-09-13.out|timing
E135|e135_rollback_floor||e135-rollback-floor-2026-09-11.out|exact
E137|e137_map_key_performance||e137-map-key-performance-2026-09-11.out|exact
E154|e154-two-gates-serial-rejudge-and-reclaim-timing||e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out|exact
E153|e153-ledger-shape-and-ring-holes||e153-ledger-shape-and-ring-holes-2026-09-24-rename.out|exact
E155|e155-fsync-write-volume||e155-fsync-write-volume-2026-09-25-h311-replay.out|exact
E155R2|e155-second-run-fsync-write-volume||e155-second-run-fsync-write-volume-2026-09-25-h311-replay.out|exact
E155R3|e155-third-run-release-cascade||e155-third-run-release-cascade-2026-09-25-h311-replay.out|exact
E155R4|e155-fourth-run-group-commit-concurrency||e155-fourth-run-group-commit-concurrency-2026-09-25-h311-replay.out|exact
E156|@driver_e156||e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out|exact
E157|e157-parallel-line-one-clauses||e157-parallel-line-one-clauses-2026-09-25-h311-replay.out|exact
E160|e160-random-small-read-share||e160-random-small-read-share-segment1-2026-09-24-realweight.out|exact
E158|@driver_e158||e158-root-choice-repair-2026-09-25-segment1-rerun.out|exact
E158|@driver_e158_q3_1_g0||e158-root-choice-repair-2026-09-25-q3-1-g0.out|exact
E158|@driver_e158_q3_1_s16||e158-root-choice-repair-2026-09-25-q3-1-s16.out|exact
E158|@driver_e158_q3_1_small_ring||e158-root-choice-repair-2026-09-25-q3-1-small-ring.out|exact
E158|@driver_e158_q3_1_s4||e158-root-choice-repair-2026-09-25-q3-1-s4.out|exact
E158|@driver_e158_q1_g0||e158-root-choice-repair-2026-09-25-q1-g0-today.out|exact
E158|@driver_e158_q2_1_g0||e158-root-choice-repair-2026-09-24-q2-1-g0-today.out|exact
E158|@driver_e158_q2_2a_g0||e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out|exact
E158|@driver_e158_q2_1_pc2||e158-root-choice-repair-2026-09-24-pc2-today.out|exact
E158|@driver_e158_q2_1_g0_session_s5||e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out|exact
E158|@driver_e158_q1_s16||e158-root-choice-repair-2026-09-25-q1-s16.out|exact
E158|@driver_e158_q1_s4||e158-root-choice-repair-2026-09-25-q1-s4.out|exact
E158|@driver_e158_q2_1_hc1||e158-root-choice-repair-2026-09-24-q2-1-hc1.out|exact
E158|@driver_e158_q2_1_hc1_lower_bound||e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out|exact
E159|e159-fsync-wait-group-commit|anchors|e159-fsync-wait-group-commit-2026-09-25-h311-replay.out|exact
TSV
)

# 计时字段：换机器、换负载就会变，比对时抹掉。抹掉的是**值**不是**字段名**——
# 字段整个消失属于结构变化，仍然会被抓。
# ⚠️ **判决字段一律不抹**：E128 那几行里 `verdict=` 与 `rounds_jia_slower=` 是结论不是计时，
# 留着逐字比 ⇒ 结论翻向会被这一步当场抓住，不用等下面的区间断言。
strip_timing() {
  sed -E 's/(per_sec_milli|median_per_sec_milli|spread_bp|min|max|ratio_bp|years_at_sync1|years_at_sync8|sync1_per_sec_milli|nosync_per_sec_milli|elapsed_ns|verify_ns|ns_per_op|ns_per_lookup|lookups_per_s|ns_small|ns_big|bing_median|jia_median|ratio_median|ratio_min|ratio_max|e128_median|deviation|ratio|one_shot_ns|two_phase_ns|two_phase_plus_5ms_ns|injected_recovered_ns|spread_one_shot|spread_two_phase|per_round_ns|best_ns|t1_ns|t16_ns|mibs|mib_per_s|entries_per_s|gbps|peak_gbps|speedup|threads16_speedup|dev|secs|copy_ns|r_rand_qd1|r_seq|random_ns_per_byte_padded|random_ns_per_byte_h133|seq_ns_per_byte_padded|seq_ns_per_byte_h133|crossover_random_share)=[^ ]*/\1=X/g'
}

# ── 结论区间断言 ──────────────────────────────────────────────────────────
# 计时实验复跑不出同样的字节，抹掉计时之后的「结构一致」又几乎什么都不证明——
# 三条臂一起漂到别处，结构照样一致（test-discipline.md「只让多条臂互相比，
# 测不出所有臂一起错」）。所以每个计时实验都要有一条把 kb 里那个数钉住的断言。
# 断言不中**不等于代码坏了**，也可能是 kb 里那个区间该改了——两种都要人来判，所以判红。
claim() { # claim <实验> <说的是什么> <实测值> <下界> <上界>
  local exp="$1" what="$2" got="$3" lo="$4" hi="$5"
  if [[ -z "$got" ]]; then
    printf '  ✗ %-5s %-46s 读不到这个值\n' "$exp" "$what"; return 1  # gate-lint:detail
  fi
  if awk -v g="$got" -v l="$lo" -v h="$hi" 'BEGIN{exit !(g>=l && g<=h)}'; then
    printf '  ✓ %-5s %-46s %s（kb 记的区间 %s–%s）\n' "$exp" "$what" "$got" "$lo" "$hi"; return 0
  fi
  printf '  ✗ %-5s %-46s %s 落在 kb 记的 %s–%s 之外\n' "$exp" "$what" "$got" "$lo" "$hi"; return 1  # gate-lint:detail
}
fld() { sed -n "s/.*$2=\([0-9.]*\).*/\1/p" "$1" | tail -1; }   # 取某行最后一个匹配字段

check_claims() {
  local exp="$1" f="$2" bad=0 v w x y
  case "$exp" in
  E17)
    # kb 记的是 29.9–31.8M 条目/秒（八轮独立运行：2026-08-28 起四轮、2026-09-16 四轮）。
    # 留 1% 余量给机器状态波动，超出就是该改 kb 那个区间了。
    # ⚠️ **下界 2026-09-16 从 30591000 放到 29636000**：当天四轮里三轮落在旧下界之外
    # （29.94 / 30.10 / 30.47 / 30.71M），不是一次抖动，所以按八轮的并集改区间。
    # 放宽的依据是那四轮的读数本身，不是为了让门禁变绿——同一天的并行加速与阳性对照
    # 一起偏低，而装置不记录跑时的机器负载，「漂移还是被别的负载挤」这一轮分不开，
    # 账在 C350（计时实验不记录跑时的机器负载）。
    v=$(grep 'arm=single' "$f" | sed -n 's/.*entries_per_s=\([0-9]*\).*/\1/p')
    claim E17 "单线程合并吞吐（条目/秒）" "$v" 29636000 32118000 || bad=1
    # 散射是并行度天花板：阳性对照 32 线程必须明显快过合并臂 32 线程，
    # 否则「散射吃掉 55%」这条读数没有判别力。
    w=$(grep 'arm=parallel threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    x=$(grep 'name=poscontrol threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    if awk -v a="$x" -v b="$w" 'BEGIN{exit !(a > b*1.5)}'; then
      printf '  ✓ %-5s %-46s 对照 %s× vs 合并臂 %s×\n' E17 "散射吃掉的那一半仍在（阳性对照更快）" "$x" "$w"
    else
      printf '  ✗ %-5s %-46s 对照 %s× 没比合并臂 %s× 快出 1.5 倍\n' E17 "阳性对照失去判别力" "$x" "$w"; bad=1  # gate-lint:detail
    fi ;;
  E20)
    # kb 的承重结论：超 L3 那一档 16 KiB 是唯一最小点，且 8 KiB 反常地差（五轮稳定，未解释）。
    for n in 2048 4096 8192 16384 32768 65536; do
      eval "v$n=\$(grep \"name=e20 node_bytes=$n keys=8388608 entry_bytes=40 \" '$f' | sed -n 's/.*ns_per_lookup=\\([0-9.]*\\).*/\\1/p')"
    done
    if awk -v a="$v16384" -v b="$v2048" -v c="$v4096" -v d="$v8192" -v e="$v32768" -v g="$v65536" \
         'BEGIN{exit !(a>0 && a<b && a<c && a<d && a<e && a<g)}'; then
      printf '  ✓ %-5s %-46s 16K=%s ns，其余 %s/%s/%s/%s/%s\n' E20 "超 L3 档 16 KiB 仍是唯一最小点" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"
    else
      printf '  ✗ %-5s %-46s 16K=%s，2K/4K/8K/32K/64K=%s/%s/%s/%s/%s\n' E20 "16 KiB 不再是最小点 ⇒ kb 那条要改" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"; bad=1  # gate-lint:detail
    fi
    if awk -v d="$v8192" -v c="$v4096" 'BEGIN{exit !(d>c)}'; then
      printf '  ✓ %-5s %-46s 8K=%s > 4K=%s\n' E20 "8 KiB 的未解释拐点又复现一次" "$v8192" "$v4096"
    else
      printf '  ✗ %-5s %-46s 8K=%s ≤ 4K=%s ⇒ kb 记的「五轮稳定」不再成立\n' E20 "8 KiB 拐点这次没出现" "$v8192" "$v4096"; bad=1  # gate-lint:detail
    fi ;;
  E144)
    # kb 记的是本机软件实现在 105 字节头上的纳秒数：crc32c 28、sha256 676（7 轮取最小）。留 ±30% 给机器状态波动，
    # 超出就是该改 kb 那个区间了；比值那一行由这两个数夹住，不另钉。
    v=$(grep 'name=cost arm=crc32c width=105 ' "$f" | sed -n 's/.*ns_per_op=\([0-9]*\).*/\1/p')
    claim E144 "CRC32C 算一个 105 字节头（ns）" "$v" 19 37 || bad=1
    w=$(grep 'name=cost arm=sha256 width=105 ' "$f" | sed -n 's/.*ns_per_op=\([0-9]*\).*/\1/p')
    claim E144 "SHA-256 算一个 105 字节头（ns）" "$w" 473 879 || bad=1
    # 判别力那一半是确定性的：阳性对照必须还漏得出双比特翻转，正式臂一个都不许漏。
    x=$(grep 'name=verdict' "$f" | sed -n 's/.*control_has_teeth=\([a-z]*\).*/\1/p')
    y=$(grep 'name=verdict' "$f" | sed -n 's/.*formal_double_bit_missed=\([0-9]*\).*/\1/p')
    if [[ "$x" == true && "$y" == 0 ]]; then
      printf '  ✓ %-5s %-46s 对照漏得出、正式臂零漏\n' E144 "判别力测试仍分得出差别"
    else
      printf '  ✗ %-5s %-46s control_has_teeth=%s formal_double_bit_missed=%s\n' E144 "判别力测试失去判别力" "$x" "$y"; bad=1  # gate-lint:detail
    fi ;;
  E128)
    # kb 的承重结论有两条，方向相反，所以两条都要钉——只钉一条会让「甲不慢」被读成
    # 「甲哪儿都不慢」，而记账那一对是慢的。
    ok67=$(grep 'name=xdev entry_bytes=67 ' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    ok111=$(grep 'name=xdev entry_bytes=111 ' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$ok67" == true && "$ok111" == true ]]; then
      printf '  ✓ %-5s %-46s 67 与 111 两档都落回 E20 的数\n' E128 "跨装置闸仍然成立"
    else
      printf '  ✗ %-5s %-46s ok67=%s ok111=%s\n' E128 "跨装置闸破了：这套装置与 E20 报不同的数" "$ok67" "$ok111"; bad=1  # gate-lint:detail
    fi
    ri=$(grep 'name=verdict pair=inode ' "$f" | sed -n 's/.*ratio_median=\([0-9.]*\).*/\1/p')
    rl=$(grep 'name=verdict pair=ledger ' "$f" | sed -n 's/.*ratio_median=\([0-9.]*\).*/\1/p')
    # ⚠️ **inode 那一对不钉方向。** 2026-09-10 四整轮里三轮判 `0/5 jia_never_slower`、
    # 第四轮判 `5/5 jia_slower_every_round`，方向相反 ⇒ kb 记的是「不稳定」，没有方向可钉。
    # 钉任何一边都是把一条不稳定的观测写成断言。这里只把观测值打出来，不判绿也不判红。
    # 真正兜着它的是上面那一步**逐字节结构比对**：`strip_timing` 有意不抹 `verdict=` 与
    # `rounds_jia_slower=`（它们是结论不是计时）⇒ 判决再翻一次，那一步就判红。
    # ⚠️ **E128 因此会间歇性判红，这是有意的。** 判红时该做的不是调宽容差，
    # 是把新跑那一次的比值与判决添进 E128 正文「结论一」那张逐轮表，让它变成五轮、六轮。
    # 每一次红都是一次新观测——对一格记着「不稳定」的实验，这正是复跑该有的行为。
    printf '  ! %-5s %-46s 109/93 = %s（不判，kb 记「不稳定」）\n' E128 "inode 那一对四轮里翻过一次向" "$ri"
    if awk -v r="$rl" 'BEGIN{exit !(r>1.00 && r<1.10)}'; then
      printf '  ✓ %-5s %-46s 97/81 = %s\n' E128 "记账树上甲仍慢一点点，量级不变" "$rl"
    else
      printf '  ✗ %-5s %-46s 97/81 = %s 掉出 (1.00, 1.10)\n' E128 "记账树那一对的方向或量级变了" "$rl"; bad=1  # gate-lint:detail
    fi ;;
  E44)
    # 本机 fsync 率：换机器会变，但**量级**要稳住，否则寿命折算整个塌掉
    v=$(grep 'name=arm arm=Sync1' "$f" | sed -n 's/.*median_per_sec_milli=\([0-9]*\).*/\1/p')
    claim E44 "本机 fsync 率（每秒千分之一次）" "$v" 500000 20000000 || bad=1
    # 阳性对照：不 fsync 必须至少快一倍，否则 fdatasync 没到设备
    w=$(grep 'name=poscontrol' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$w" == true ]]; then printf '  ✓ %-5s %-46s\n' E44 "阳性对照：fdatasync 确实到了设备"
    else printf '  ✗ %-5s %-46s ok=%s\n' E44 "阳性对照失败 ⇒ fdatasync 没到设备，整轮作废" "$w"; bad=1; fi
    # 48 位计数器在本机速率下的寿命：这是「48 位够不够」那条结论的落点
    x=$(grep 'name=lifetime bits=48' "$f" | sed -n 's/.*years_at_sync1=\([0-9]*\).*/\1/p')
    claim E44 "48 位计数器在本机撑多少年" "$x" 500 50000 || bad=1
    # 加宽 jsn 到 12 字节的代价：0..=100 项里一格都不该多占
    y=$(grep 'name=width unit=512 jsn_bytes=12 ' "$f" | sed -n 's/.*cost_unit_count_0_100=\([0-9]*\).*/\1/p')
    claim E44 "jsn 8→12 在 512 单元下多占几格" "$y" 0 0 || bad=1 ;;
  E58)
    # kb 的承重结论：32 KiB 在随机小读上比 16 KiB 贵约一成，而顺序侧只快 0.83%。
    # 只钉倍数不够（三条臂一起漂，倍数照样对），所以第三条钉的是绝对值。
    v=$(grep 'name=rand_g16384 ' "$f" | sed -n 's/.*ns_per_op=\([0-9.]*\).*/\1/p')
    w=$(grep 'name=rand_g32768 ' "$f" | sed -n 's/.*ns_per_op=\([0-9.]*\).*/\1/p')
    x=$(awk -v a="$w" -v b="$v" 'BEGIN{if(b>0) printf "%.4f", a/b}')
    claim E58 "随机小读 QD=1：32 KiB 是 16 KiB 的几倍" "$x" 1.03 1.25 || bad=1
    v=$(grep 'name=randq_g16384 ' "$f" | sed -n 's/.*user_mib_per_s=\([0-9.]*\).*/\1/p')
    w=$(grep 'name=randq_g32768 ' "$f" | sed -n 's/.*user_mib_per_s=\([0-9.]*\).*/\1/p')
    x=$(awk -v a="$v" -v b="$w" 'BEGIN{if(b>0) printf "%.4f", a/b}')
    claim E58 "QD=16 用户带宽：16 KiB 是 32 KiB 的几倍" "$x" 1.03 1.25 || bad=1
    # 绝对值：32 KiB 单元读 4 KiB 的放大恰为 8，由独立算术给出（32768/4096）
    x=$(grep 'name=meta_g32768 ' "$f" | sed -n 's/.*read_amp_4k=\([0-9.]*\).*/\1/p')
    claim E58 "32 KiB 单元的读放大（绝对值）" "$x" 8.0 8.0 || bad=1
    # 阳性对照：内核记的字节 ÷ ops×G，读到洞或读到缓存都会让它塌
    x=$(grep 'name=rand_g32768 ' "$f" | sed -n 's/.*pr_over_devbytes=\([0-9.]*\).*/\1/p')
    claim E58 "阳性对照：内核记的字节 ÷ (ops×G)" "$x" 0.98 1.02 || bad=1 ;;
  E140)
    # kb 的承重结论：含头随机页读比补齐慢 7.7%–9.1%，顺序读补齐少 7.6%–9.2% 带宽。复跑用的是种子 1、ops 4096 的单轮，
    # 抽样比第三轮少 4 倍，区间按第三轮五个种子的极差再各放 5 个百分点。
    v=$(grep 'name=verdict' "$f" | sed -n 's/.*r_rand_qd1=\([0-9.]*\).*/\1/p')
    claim E140 "随机 4 KiB 页读：含头是补齐的几倍" "$v" 1.03 1.15 || bad=1
    w=$(grep 'name=verdict' "$f" | sed -n 's/.* r_seq=\([0-9.]*\).*/\1/p')
    claim E140 "顺序读：补齐带宽是含头的几倍" "$w" 0.85 0.97 || bad=1
    # 绝对值：跨单元页占比由闭式 (4096 − gcd) / 32635 独立算出，与计时无关
    x=$(grep 'name=model_h133 ' "$f" | sed -n 's/.*straddle_fraction=\([0-9.]*\).*/\1/p')
    claim E140 "头 133 时跨单元页占比（闭式）" "$x" 0.125479 0.125479 || bad=1
    # 阳性对照：内核记的字节 ÷ 程序记账，读到缓存就塌（阴性对照那份产物里同一格是 0.0000）
    y=$(grep 'name=rand_h133 ' "$f" | sed -n 's/.*pr_over_devbytes=\([0-9.]*\).*/\1/p')
    claim E140 "阳性对照：内核记的字节 ÷ 程序记账" "$y" 0.98 1.02 || bad=1 ;;
  E107)
    # kb 的承重结论有两条，一条是字节、一条是延迟，两条都要钉。
    # 字节那条是纯算术、逐次相同，但仍然钉住——只钉延迟会让一个把字节模型改错的变异照样绿。
    v=$(grep 'name=bytes leaves=8 f=0 ' "$f" | sed -n 's/.*ratio_a_over_c=\([0-9.]*\).*/\1/p')
    claim E107 "主负载 8 叶 f=0：甲臂 ÷ 丙臂" "$v" 0.9503 0.9503 || bad=1
    v=$(grep 'name=crossover' "$f" | sed -n 's/.*first_leaves_where_arm_a_cheaper=\([0-9-]*\).*/\1/p')
    claim E107 "甲臂第一次比丙臂便宜的叶数" "$v" 8 8 || bad=1
    # 延迟那条：durable 语义下三段写序比一次提交慢多少倍
    a=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*one_shot_ns=\([0-9]*\).*/\1/p')
    b=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*two_phase_ns=\([0-9]*\).*/\1/p')
    x=$(awk -v a="$a" -v b="$b" 'BEGIN{if(a>0) printf "%.4f", b/a}')
    claim E107 "durable：三段写序 ÷ 一次提交" "$x" 1.15 2.10 || bad=1
    # 阳性对照：注入的 5 ms 必须被量回来，落在 ±10% 内
    x=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*injected_recovered_ns=\([0-9-]*\).*/\1/p')
    claim E107 "阳性对照：注入 5 ms 回收到的纳秒" "$x" 4500000 5500000 || bad=1
    # 作废条款 3：不 O_DIRECT 不 fdatasync 那条必须快一个数量级，否则 I/O 没真落盘
    a=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*one_shot_ns=\([0-9]*\).*/\1/p')
    b=$(grep 'name=nosync_control' "$f" | sed -n 's/.*per_round_ns=\([0-9]*\).*/\1/p')
    x=$(awk -v a="$a" -v b="$b" 'BEGIN{if(b>0) printf "%.2f", a/b}')
    claim E107 "作废条款 3：durable ÷ nosync" "$x" 10 500 || bad=1 ;;
  E21)
    # kb 的承重结论：CPU 扫描撞内存带宽墙（约 65 GB/s），16 线程几乎不加速 ⇒ GPU 传输地板已经更慢。
    v=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*peak_gbps=\([0-9.]*\).*/\1/p')
    claim E21 "CPU 扫描峰值带宽（GB/s）" "$v" 55 75 || bad=1
    w=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*threads16_speedup=\([0-9.]*\).*/\1/p')
    claim E21 "带宽受限：16 线程几乎不加速" "$w" 1.0 1.4 || bad=1
    # 阳性对照：同样 16 线程，计算受限的那条必须大幅加速，否则「不加速」分不清是带宽墙还是没跑起来
    x=$(grep 'name=poscontrol arm=compute' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    claim E21 "阳性对照（计算受限）16 线程加速" "$x" 10 20 || bad=1 ;;
  esac
  return $bad
}

# E9 的入库产物是 25 次运行拼起来的（5 种子 × 5 改名档），而这个循环从没被写进 kb。
# 2026-08-29 审计时按产物里的 config 行反推出来，重建结果与入库产物**逐字节一致**。
driver_e9() {
  local r s
  for r in 0 500 2000 5000 20000; do
    for s in 3 7 11 13 17; do
      ./target/release/e9-keylayout "$REPLAY_DEV" "$s" interleave 8 "$r" || return 1
    done
  done
}

# E142 第十五次跑步④（重跑登记 `research/prompts/e142-r15-prereg.md` 第六节）：装置↔crates/ 逐字节比对
# 要跨两个 cargo workspace（research/ 与仓根的 crates/ workspace，仓根 Cargo.toml 显式 exclude =
# ["research"]，两边互相看不到对方，不能合并成一次 cargo 调用）。旧的 `first_transaction_region_bytes`
# 读的是改位置寻址之前的旧布局（区域表写死八个单元，见该文件模块注释），已经比不出新写的五个分配记录树节点，
# 换成只读导出 `e142_first_transaction_write_dump`（不带任何布局知识，逐次写按 (设备, 偏移, 长度, sha256,
# 整段十六进制) 原样打出来）。装置的第一个命令行参数是这份导出的文件路径（做 Q142.1 真比对），第二个参数是
# arm O 的历史留存产物（`第十五次跑步④` 起 Q142.8 用它算「哪些区域从旧布局变到了新布局」，
# 交回报告 `research/prompts/e142-r15-step234-runner-report.md` 里写明这份参照为什么找不到能重新编译的
# 源码、只能用留存产物）。两段都各自有自己的 `name=done`，闸 2 逐段核过。
#
# E142 第十六次跑第一段（重跑登记 `research/prompts/e142-r16-prereg.md`；交回报告
# `/tmp/claude-1000/e142-r16-s1/report.md`）：α（内部/根节点一格）、β（根 largest_key）、ι（内部条目
# key 取什么）、γ（extent 上段叶 key 区间）三格 5.1 的变体开关整套删除，改成 D8（核心索引结构） 已定项 14
# 第 395/401 行、D18（块里携带什么信息） 已定项 2 射程写死的唯一写法（稀疏、整个 key 空间、按位置分片区间）；
# δ（盘上槽数）收口成 `slots_of_device_bytes` 一个 const fn。第二个命令行参数从「第十四次跑 arm O 参照」
# 改成「这一次步①现编现跑的臂 N15 参照」（`research/results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out`），
# 按 (设备, 偏移, 长度) 配对出 `name=old_new_region`（Q142.19），不再按名字配对出旧的 `name=old_new_region`
# 系列。新增 `name=g3_shape`（第八节 G3 五个几何点，不依赖 `crates/`）、`name=positive_control_p2`
# （四个点，锚点随稀疏改成第 0 条条目）、`name=g4_bytes_equal_summary`（第三个命令行参数给才跑，这一次
# `crates/` 一盘几何造不出（`make_filesystem.rs:194`，D2（RAID 条带策略） 已定项 9），不传，报
# `skipped=true`）。`code2_field_rows` 从「非空格与第一个空格」改成「每条都列 + 补齐区一行」。
# Q142.11（原 Q142.1）这一次判「全等」：29 个区域全部配上、29 个相等、0 个不等（三格上模型与 `crates/`
# 逐字节相同，只说明 kb 转写与 `crates/` 一致，不说明条款本身对，见第十二节修订与第 5.4 节）。
driver_e142() {
  local impl_snapshot="$OUT_DIR/e142-crates-write-dump.tmp"
  local arm_n15_reference="results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out"
  (cd .. && cargo run -q -p singlefs-harness --bin e142_first_transaction_write_dump) >"$impl_snapshot" || return 1
  ./target/release/e142-first-txn-dry-run "$impl_snapshot" "$arm_n15_reference" || return 1
  cat "$impl_snapshot"
}

# E156（入库装置，跑前登记「一」读法写死第 1 行；重跑登记 `research/prompts/e156-r2-prereg.md`「五、5.7」
# 第一、二、三段都在同一个二进制里，产物是累计的：2026-09-24 stage2.out 前 324 行是第一段（岔路 7），
# 之后是第二段（岔路 3，run_hf_single_cell）与第三段（岔路 1，run_hh_cell）新加的行。
# stage3.out（2026-09-24，重跑登记「十二」修订这一段）在 stage2.out 的基础上追加：Q1 诊断溯源
# （`q1_delta_debug`／已回收未覆盖记录过滤，见修订）、岔路 1 的 S = 4 第二个几何取样点（`q1_hh`／
# `q1d_*` 的 `s=` 字段、`anchor_k8`、`q1_geometry_sensitivity_s`）、K9 前提现核（`k9_precondition`）；
# 在 stage2.out 原有的 329 行共有格式上逐字节相同（`e156-s4/report.md` 的对拍命令），Q3e（X8-A）仍
# `status=not_done`，未做。stage4.out（2026-09-24，续派第二段）在 stage3.out 的基础上追加：岔路 1
# 步数对齐对照（`q1_step_matched_diff`，`run_hh_cell` 新增 `matched_to_holes` 参数）、岔路 3 的
# Q3e（X8-A/HY，`run_x8a_cell`，独立 128 槽小池）与 Q3d（`q3d_derived`）；在 stage3.out 原有的 323 行
# （K1/legal_state/s1d_step/Q7 全家/PC-检查三条/HK-HR-H0 构造）共有格式上逐字节相同。
# r3.out（2026-09-25，第 3 次重跑登记 `research/prompts/e156-r3-prereg.md`：分配记录树按位置寻址之后
# 重算钉着旧布局的常量）在 stage4.out 的基础上结构性改动，不是逐字节追加：分配记录树不再是单节点，
# 装置里的三处常量与另外六处改成第七节 7.2 算出的闭式与绝对值（K1-1 第 1 项 13→17、β0 走读引用 12→16、
# S1(c) 隔离槽数 34→54，其余变成随「这次改动的记录落在几片叶」现算的闭式，不再是常数）；新增
# `anchor_a_d8`、`anchor_a_d8_root_level`、`s1ef_step`／`s1ef_summary`（Q3r.2 逐次闭式核对）、
# `r5_empty_publish_closed_form`、`r7_hy_cap`／`r7_hy_condition`（HY 的覆盖写次数改成搜出满足
# e ≥ max(8, f) 的那一档，这一轮搜到的是 0：搜索过程见交回报告，判定按登记走）行；H0／HR 不再撞
# `AllocationRecordsExceedOneNode` 那道墙，跑满登记要求的全部步数（`baseline_workload_completed_full_length`
# 取代 stage4.out 的 `baseline_workload_truncated_by_write_failure`）。Q3r.4（岔路 1、3、7 判定变不变）
# 的比对命令与判定表见交回报告；核心结论：岔路 1（Q1d 在 S = 4 上从「单调」变「不单调」）与岔路 7
# （`q7d2_min_item5` 从「非 0」变「= 0」，F16 从「触发」变「不触发」）判定变了，岔路 3（Q3c/Q3e/K9）
# 在已核的量上不变。
# fork7-selfproof.out（2026-09-25，续派，E156 第 3 次重跑登记「十二」修订 4）在 r3.out 的基础上加
# 一个第 10 个基底 `beta_hr_rollback_row`：r3.out 的 `q7d2_min_item5` 第一次在可达状态上读到
# `min_item5=0 family=HR kind=rollback_row txg=76`（此前 7 个可达基底第 5 项最小是 1），这里把这一步
# 也捕成一个基底、并入既有的 `bases` 数组（9→10），让已有的 Q7a/Q7c①②/PC-检查循环再跑一遍，不改
# `q7c_self_test` 的公式、不新写判定逻辑。核心结论：Q7c① 在这个基底上第一次转色（`q7c1_not_subtracting_defer
# basis=beta_hr_rollback_row … flips_red_to_green=true`）——此前只有不可达的 β_syn 转过色；`basis_count`
# 9→10，`q7a_summary` 的 `all_red_count` 18→20（该基底 delta=1/8 各命中一次），`q7c1_flip_seen`/
# `q7c2_flip_seen` 字面不变（β_syn 已经让它们是 true）。在 r3.out 原有的 1076 行共有格式上逐字节相同，
# 只在旧的 `q7a_summary`/`integrity` 两行与新增 9 行（`basis_snapshot`/`q7a` ×3/`q7c1`/`q7c2`/
# `pc_check` ×3）上不同。
# 整个装置就活在 crates/singlefs-harness 里，没有 research/e7-index-bench 侧的配对二进制，先例同 E142（第 371 行注释）——
# 两个 cargo workspace 互相看不到对方，不能合并成一次调用。确定性：同一个二进制跑两遍逐字节一致（2026-09-25 现查：
# 这一段的产物两次跑出 `cmp` 逐字节一致）。
driver_e156() {
  (cd .. && cargo run -q -p singlefs-harness --bin e156_allocation_basis_counts)
}

# E158（第一段，入库装置，跑前登记「装置写在哪」写死第 5 行）：同 E156 的先例，两个 cargo workspace
# 互相看不到对方，不能合并成一次调用。`all` 模式跑 5.7 常量回比、第七节锚点、S5 独立解码对拍、
# H3（岔路 3）四个几何点不注入全枚举、Q3-2/Q2-3 算术、PC3。
# 2026-09-23 产物在 `crates/` 落地 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 定案（拿掉
# 「回退到无文件那一版」的拒绝）之后结构性对不上（50 行不同，原因见实验页与
# `/tmp/claude-1000/e158-s2/report.md` 第 1 节）；2026-09-24 主 agent 定这一行承重
# `e158-root-choice-repair-2026-09-24-segment1-rerun.out`（旧产物 `…-2026-09-23-segment1.out`
# 原样留着，对应 C512 落地之前的代码，不再是这一行比对的对象）。
# ⚠️ **2026-09-25（session s10）**：另一条并行线（主 agent 知会「实二六」）此刻在改
# `mount.rs`/`allocator.rs`/`allocation_record_tree.rs`（抬 F、挂载读、分配记录树根层），这一份
# `2026-09-24` 产物与今天重出的产物对不上，`pc3` 一行从 `recover_verdict=pass recover_root=
# Some((1, 6))` 变成 `recover_verdict=fail recover_root=Some((0, 0))`——按今天日期另存
# `e158-root-choice-repair-2026-09-25-segment1-rerun.out`，登记表这一行已改指向它；`…-09-24-…`
# 原样留着不删。**这不是本轮代码改动引起的**（e158 装置本身这一段只加了 op1 第三种变体，没碰
# `driver_e158`/`pc3` 这条路径），是不是要等 `crates/` 落定后再复核一遍交主 agent 定，详见跑前
# 登记「十二、修订」session s10 条目第 6 条与交回报告。
driver_e158() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- all)
}

# E158 第二段 Q3-1（岔路 3 候选 3 那一半，2026-09-24）：`q3-1-g0` 模式只跑 G0 几何上的
# H3 × Φ3（|F|≤2）违例枚举，不跑第一段的 H3 全枚举（那部分归 `driver_e158`）。
# ⚠️ **2026-09-25（session s10）：值得单独点名的一处现查**——G0/S16/小环三点，今天重出的
# `pairs_with_any_violation` 从旧产物的 213 变成 0（`cold_recover_with_fault_failed`/
# `mount_writable_with_fault_failed` 两点都仍是 0，不是新增了报错，是判定本身不再违例）；S4 那一点
# 除了 `pairs_with_any_violation`（1355→0）之外，`cold_recover_with_fault_failed`/`mount_
# writable_with_fault_failed` 还从 0 变成 239——即同一批构造里，以前是「成功但违例」，现在有 239
# 个变成了「调用直接报错」，是两种不同的失效形态，不只是数字变化。与 `driver_e158` 的 `pc3` 翻转
# 同一批文件改动引起（`mount.rs`/`allocator.rs`/`allocation_record_tree.rs`，另一条并行线，非本轮
# e158 装置改动）。**这条现查可能动到岔路单第 3 行「候选 3 = 今天」这个前提与 F2 的触发条件**——
# C332 正文钉的「2 个故障就能撤销回退」在今天这份 `crates/` 上现在测不出来了，交主 agent 判断是要
# 重新核对 F2、还是等这条并行线落定再复核；详见跑前登记「十二、修订」session s10 与交回报告。今天
# 重出的产物按日期另存 `e158-root-choice-repair-2026-09-25-{q3-1-g0,q3-1-s16,q3-1-small-ring,
# q3-1-s4}.out`，`…-09-24-…` 原样留着，登记表已改指向新文件。
driver_e158_q3_1_g0() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q3-1-g0)
}
# 第八节敏感性（行 3）三个取样点：S16、小环（环长现算，产物里的 `small_ring_search` 那行同时钉住取到的环长）、
# S4（`sigma_length_limit=4`，比其余三点多穷举一层，代价数量级最大，real 约 18 分钟，2026-09-24 现查）。
driver_e158_q3_1_s16() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q3-1-s16)
}
driver_e158_q3_1_small_ring() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q3-1-small-ring)
}
driver_e158_q3_1_s4() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q3-1-s4)
}
# E158 岔路单第 1 行（C393）：`q1-g0` 模式在今天的 `crates/`（候选 (c)）上跑 H1 家族 + Φ1 故障注入 +
# PC1-a/PC1-b，只跑 G0 几何（2026-09-24 session s3，见实验页）。候选 (a)（A1 副本）的同一份数只存产物
# `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，**不登记在这张表里**：它要在
# `research/mutations/e158_arms.tsv` 描述的副本上重新编译才跑得出来，这张表假设「跑这一行就等于跑今天
# committed 的 crates/」，副本不满足这个假设；复跑它的步骤见实验页「复跑」一节。session s9 起产物里
# 多了 171 格差集的分类诊断（`q1_2_subset_diff_pair*`，根因是 `RootRecord::instance_table` 没被走
# 全版本走到）与 op1 起三步挂载的持续/瞬时故障轨迹（`q1_1a_op1_trajectory`），纯增量追加。
# **2026-09-25（session s10）**：加了 op1 第三种变体（`MountWritableThenRaiseFloorTo`），新增
# `q1_1a_raise_floor_trigger_summary`/`q1_1a_raise_floor_by_aspect_severity`（G0：
# `pairs=96 trigger_count=32 history_nodes_without_room=26`，both_copies 两个指称各 16/16 触发、
# disk0_only/disk1_only 各 0/16，与既有 op1 两种同一批构造上的模式一致）——这一段是本轮新增、真实
# 数据，不是噪声。⚠️ 同一份产物里还混着另一条并行线（`mount.rs`/`recovery.rs`）造成的漂移：
# `q1_2_tree_table_crates_has_a_path`（`verify_named_units=false`）从「108+81」变成「189+0」（原来
# 能不碰物理拷贝复算出指针的 81 格现在全部走不到了）、`pc1_a` 的 `device_write_bytes` 从 157184 变
# 353792（`AllocationRecordTreeNode` 写得更多，与 `q2_2a_g0` 那条同一根因）——这两条与本轮 e158 装置
# 改动无关，是现查到的既有漂移（与 session s9 报告的方向一致）。今天重出的产物按日期另存
# `e158-root-choice-repair-2026-09-25-{q1-g0-today,q1-s16,q1-s4}.out`，`…-09-24-…` 原样留着。
driver_e158_q1_g0() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q1-g0)
}
# E158 岔路单第 2 行 ①（C331 修法，session s4，2026-09-24 session s5 起废弃，见下）：`q2-1-g0` 在
# 今天的 `crates/`（丙 = 甲-jsn）上跑 H2 主族（op2=`mount_writable`，只 n1∈{0,1,2,3}、n2=1）的穷举
# 下界搜索。**session s5 查出这条产物是在故障装配 bug 存在时跑出来的**（`attempt_rootback_probe_
# and_advance` 一块盘只装得上一个故障目标，权重 ≥ 2 就可能漏装——见跑前登记「十二、修订」session s5
# 条目第 2 条）：bug 修好之后同一个 n1 范围重跑给出不同结果（甲-txg 臂从「0 命中」变成「k_min=4」），
# 这条产物与它对应的 `driver_e158_q2_1_g0` 不能再当「今天/丙 0 命中」的依据引用，只留着当「bug 修前
# 长什么样」的历史对照。承重的是下面 `driver_e158_q2_1_g0_session_s5`。
driver_e158_q2_1_g0() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q2-1-g0)
}
# E158 岔路单第 2 行 ②（每条修法每次发布多写几字节，session s4）：`q2-2a-g0` 在今天的 `crates/` 上跑
# 固定脚本，按结构种类报每次发布写的字节。甲-txg 臂的同一份数只存产物，同上不登记在这张表里。
# ⚠️ **2026-09-25（session s10）**：`AllocationRecordTreeNode` 每次发布的 `write_calls`/
# `written_bytes` 全面上涨（例如 `second_mount_row_publish` 从 `write_calls=2 written_bytes=32768`
# 变成 `write_calls=10 written_bytes=163840`），与 `q1_g0` 的 `pc1_a` 字节变化同一根因
# （`allocator.rs`/`allocation_record_tree.rs`，另一条并行线，非本轮 e158 装置改动）。今天重出的
# 产物按日期另存 `e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out`，`…-09-24-…` 原样留着。
driver_e158_q2_2a_g0() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q2-2a-g0)
}
# E158 岔路单第 2 行 PC2（阳性对照，session s5）：复现判决 K2 的两个具体构造（甲-txg 4 个瞬时根槽
# 读失败、乙-只配置 0 个注入故障 + 1 个崩溃点），不靠穷举——见跑前登记「十二、修订」session s5。
# 乙-只配置候选的同一份数只存产物（副本上的数，副本没有 `published_txg` 字段就编不过，不登记在这张
# 表里，复跑步骤见实验页与 `research/mutations/e158_arms.tsv`）。
driver_e158_q2_1_pc2() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q2-1-pc2)
}
# E158 岔路单第 2 行 ①（C331 修法，2026-09-24 session s5，承重）：修好故障装配 bug 之后，`q2-1-g0`
# 在今天的 `crates/`（丙 = 甲-jsn）上重跑 H2 主族，n1 范围从 {0,1,2,3} 补齐到跑前登记 5.1 要求的
# {0,...,6}。甲-txg 臂的同一份数只存产物，同上不登记在这张表里（复跑步骤见实验页）。**session s9
# 撤掉了旧的计数上限（`SUBSET_ENUMERATION_CAP`，会在权重档中途停手）**，换成按权重档边界停的
# `weight_ceiling` 机制：n1=0..3 仍是完整穷举（`subsets_tried`==`full_space_subset_count`）；n1=4、
# 6 用新机制找到 `k_min=12`（此前因为撞旧计数上限报 `capped`，没有找到）；n1=5 如实报
# `k_min=not_found_up_to_weight_ceiling`（完整空间 131072、只搜到权重 12 为止，未截断，见跑前登记
# 「十二、修订」session s9）。**2026-09-25（session s10）复跑确认字节一致**——与 `q2_1_pc2`/
# `q2_1_hc1`/`q2_1_hc1_lower_bound` 三行一样不受另一条并行线这一刻改动的影响（那条线动的是
# `mount.rs`/`allocator.rs`/`allocation_record_tree.rs` 里 `q3-1`/`q1-g0`/`q2-2a-g0` 会读到的
# 路径，H2 主族的穷举下界搜索走的是 `first_txg_of_new_instance`/`next_counter`/根环读取，不经过
# 那几处改动），行 2「够判」的结论不受这一刻 crates/ 波动影响。
driver_e158_q2_1_g0_session_s5() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q2-1-g0)
}
# 第八节几何敏感性（行 1，2026-09-24 session s6）：岔路单第 1 行判决格 = Q1-1a 的 N_trig，S16（根环
# 大一倍）与 S4（根环小一半）两个方向相反的取样点，复用 H1 装置代码（`run_ledger_fault_family` 本身
# 就是几何参数化的）。两点都与 G0 逐字节等值（N_trig=798），判定不翻面，判别力自证「两点同值，自证
# 不适用」。
driver_e158_q1_s16() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q1-s16)
}
driver_e158_q1_s4() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q1-s4)
}
# H-C1 直接构造（2026-09-24 session s6，岔路单第 2 行 ①，丙的具体历史）：判决 K2 说丙需要 12 个
# 故障（4 根槽 + 4 条记录各两块盘）才打得中；`run_rootback_tolerance_family` 的穷举在 n1∈{4,5,6}
# 会撞 `SUBSET_ENUMERATION_CAP`，这里不靠穷举，直接按这个具体构造跑一次（hit=true, weight=12），
# 附一个只挡 4 条根槽、不挡记录的负对照（hit=false，证明「只挡根环挡不住丙」）。
driver_e158_q2_1_hc1() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q2-1-hc1)
}
# H-C1 下界探针（2026-09-24 session s6；session s9 改参数为 weight_ceiling）：在同一个 n1=4 节点
# 上，穷举权重 0..11 的全部组合（`subsets_tried=22558`，`full_space_subset_count=32768`，没有撞
# 顶）都没有命中，权重 12 上第一个尝试的组合就命中。**这是「恰好 12」的严格证据**：不是构造上界，是
# 穷举下界真正走到了 12 且之下全空。不传第二个参数时 `weight_ceiling=None`——n1=4 这个节点的完整
# 空间只有 2^15=32768，在 `FEASIBLE_FULL_SEARCH_SUBSET_BUDGET`=100000 预算内，`None` 就等于穷举到
# 完整空间的顶，与 session s6 当年手动调高到 60000 效果相同（60000 > 这段历史任何可能的权重值）。
driver_e158_q2_1_hc1_lower_bound() {
  (cd .. && cargo run -q --release -p singlefs-harness --bin e158_root_choice_repair -- q2-1-hc1-lower-bound)
}

ONLY=("$@")
# 替换表按 E103 这种带 E 的形态登记；裸数字会匹配 0 条并报全零（2026-09-05 在 E103 上踩过两次）。
for wanted in ${ONLY[@]+"${ONLY[@]}"}; do
  if [[ "$wanted" =~ ^[0-9]+$ ]]; then
    echo "  ✗ 实验号 $wanted 没带 E：这样匹配不到任何一行，跑出来全是零" >&2
    echo "     → 怎么办：写成 E$wanted，例：bash research/scripts/replay.sh E$wanted" >&2
    exit 2
  fi
done
want() { [[ ${#ONLY[@]} -eq 0 ]] && return 0; local e; for e in "${ONLY[@]}"; do [[ "$e" == "$1" ]] && return 0; done; return 1; }

# 产物列是 results/ 底下的纯文件名，不是仓库根起的路径：写成路径时下面拼出 results/research/results/… 指不到文件，
# 而 diff 失败会被报成「对不上，N 行不同」——看着像产物变了，其实是登记表坏了。2026-09-21 被一次全仓路径回写
# 的路径回写踩中一次（它把第 4 列升级成了仓库根路径）。
bad_rows=$(printf "%s\n" "$TABLE" | awk -F"|" '/^E[0-9]+\|/ && $4 ~ /\// {print "      " $1 "：第 4 列 " $4}')
if [[ -n "$bad_rows" ]]; then
  echo "  ✗ 登记表第 4 列（留存产物）写成了带斜杠的路径，应当是 results/ 底下的纯文件名：" >&2
  printf "%s\n" "$bad_rows" >&2   # gate-lint:detail
  echo "     → 怎么办：把那几行第 4 列改回纯文件名（例 e100-system-configuration-slot-2026-09-03.out）；" >&2
  echo "               产物搬过家就同时改文件名本身，别把目录写进这一列。" >&2
  exit 2
fi

cargo build --release --manifest-path e7-index-bench/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; exit 2; }

pass=0; drift=0; timing_only=0; broken=0; claim_bad=0; archived=0
CLAIM_QUEUE=()

printf '%-5s %-24s %-10s %s\n' 实验 二进制 判定 说明
printf '%s\n' "-------------------------------------------------------------------------"
# 一条实验的复跑：并发起的，所以计数、结果行与 claim 队列都落到自己的文件，
# 父进程按登记表的顺序回读（command-safety.md「并行不许把失败吃掉」「输出不许直接往 stdout 写」）。
replay_one() {
  # 第 2 个参数是这一条在登记表里的序号：同一个实验号可以有两行（E16 就是，一行比 e16-journal、
  # 一行带 args=bytes 比 e16-bytes），按实验号命名输出文件时两条会并发写同一个文件，
  # 后写完的那份整份留下、另一条的结果彻底消失，而「派多少收多少」那道闸只判文件存不存在、看不出来。
  local exp="$1" seq="$2" bin="$3" args="$4" stored="$5" kind="$6" tag fresh rc gate2 n
  tag="$exp.$seq"
  fresh="$OUT_DIR/$tag.out"
  rm -f "$fresh"                                    # 闸 1：不许跨轮复用
  if [[ "$bin" == @* ]]; then
    "${bin#@}" >"$fresh" 2>"$OUT_DIR/$tag.err"
  else
    # shellcheck disable=SC2086
    ./target/release/"$bin" $args >"$fresh" 2>"$OUT_DIR/$tag.err"
  fi
  rc=$?
  if [[ $rc -ne 0 ]]; then                          # 闸 3
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "退出码 $rc，见 $OUT_DIR/$tag.err" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 闸 2：**逐段**核。产物可能是多次运行拼起来的（E9 就是 25 段），
  # 只看最后一个 name=done 会让前 24 段的缺行全部漏过去。
  gate2=$(awk '/^E7RESULT/{n++}
               /^E7RESULT name=done emitted=/{
                 split($0,a,"emitted="); e=a[2]+0
                 segs++
                 if (e != n) { print "第 " segs " 段收尾行说 " e " 条，实收 " n " 条"; bad=1; exit }
                 n=0
               }
               END{ if (!bad) { if (segs==0) print "没有收尾行 name=done"; else if (n>0) print "最后一段没有收尾行，尾巴 " n " 条" } }' "$fresh")
  if [[ -n "$gate2" ]]; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "$gate2" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 留存产物已按「每次提交删上一次的实验记录」归档进版本库时，这一档不比对。
  # 不报成「对不上」：那与「装置真的改坏了、复跑出不同字节」长得一模一样，读的人会以为实验坏了
  # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
  if [[ ! -f "results/$stored" ]]; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 产物已归档 "$stored 不在树里；本次跑得出来，逐字节这一档不比对" >"$OUT_DIR/$tag.line"
    echo archived >"$OUT_DIR/$tag.verdict"; echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"; return
  fi
  if diff -q "$fresh" "results/$stored" >/dev/null 2>&1; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 字节一致 "$stored" >"$OUT_DIR/$tag.line"; echo pass >"$OUT_DIR/$tag.verdict"
    echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"; return
  fi
  if diff -q <(strip_timing <"$fresh") <(strip_timing <"results/$stored") >/dev/null 2>&1; then
    if [[ "$kind" == timing ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 仅计时不同 "$stored（结构一致，符合声明）" >"$OUT_DIR/$tag.line"; echo timing_only >"$OUT_DIR/$tag.verdict"
      echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"
    else
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 判据写错 "$stored 声明 exact 却只在抹掉计时后才一致" >"$OUT_DIR/$tag.line"; echo drift >"$OUT_DIR/$tag.verdict"
    fi
    return
  fi
  n=$(diff <(strip_timing <"$fresh") <(strip_timing <"results/$stored") | grep -c '^[<>]')
  printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 对不上 "$stored，$n 行不同 → diff $fresh results/$stored" >"$OUT_DIR/$tag.line"
  echo drift >"$OUT_DIR/$tag.verdict"
}

# 并发度：这些实验多是跑 release 二进制的 CPU 活，按核数定；有几个自己就是多线程的，所以不吃满。
# REPLAY_JOBS 压过它。各条实验的产物、镜像（e9/e45/e58/e140.img）与结果文件各用各的，没有共用的可写状态。
REPLAY_JOBS="${REPLAY_JOBS:-$(( $(nproc 2>/dev/null || echo 4) / 2 ))}"
[[ "$REPLAY_JOBS" -lt 1 ]] && REPLAY_JOBS=1
declare -a REPLAY_PIDS=() REPLAY_ORDER=()
running=0
row_sequence=0
while IFS='|' read -r exp bin args stored kind; do
  [[ -z "$exp" ]] && continue
  want "$exp" || continue
  args="${args//\$REPLAY_DEV140/$REPLAY_DEV140}"  # 先换长的，否则前缀会被短的吃掉
  args="${args//\$REPLAY_DEV58/$REPLAY_DEV58}"
  args="${args//\$REPLAY_DEV45/$REPLAY_DEV45}"
  args="${args//\$REPLAY_DEV/$REPLAY_DEV}"   # 表里写字面量 $REPLAY_DEV，这里才展开
  row_sequence=$((row_sequence+1))
  REPLAY_ORDER+=("$exp.$row_sequence")
  replay_one "$exp" "$row_sequence" "$bin" "$args" "$stored" "$kind" &
  REPLAY_PIDS+=($!)
  running=$((running+1))
  if (( running >= REPLAY_JOBS )); then wait -n 2>/dev/null || true; running=$((running-1)); fi
done <<<"$TABLE"
# 逐个 wait 写死的 pid，不写不带参数的 wait（它的退出码恒为 0，红了几个一个字都不说）
for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do wait "$pid" 2>/dev/null || true; done

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿
collected=0
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  [[ -f "$OUT_DIR/$tag.line" ]] && collected=$((collected+1))
done
if (( collected != ${#REPLAY_ORDER[@]} )); then
  echo "  ✗ 派出去 ${#REPLAY_ORDER[@]} 条复跑，只收回 $collected 条结果行"
  echo "     → 怎么办：这是并发收束自己的完整性闸红了，不是实验的问题；REPLAY_JOBS=1 再跑一遍看串行下全不全，"
  echo "       全的话去查 replay_one 与它的 pid 收束那一段。"
  exit 1
fi

# 按登记表的顺序回读：输出与并发度无关，REPLAY_JOBS=1 与 =16 逐字相同
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  cat "$OUT_DIR/$tag.line"
  if [[ -f "$OUT_DIR/$tag.verdict" ]]; then
    case "$(cat "$OUT_DIR/$tag.verdict")" in
      pass) pass=$((pass+1)) ;; timing_only) timing_only=$((timing_only+1)) ;;
      drift) drift=$((drift+1)) ;; broken) broken=$((broken+1)) ;; archived) archived=$((archived+1)) ;;
    esac
  fi
  [[ -f "$OUT_DIR/$tag.claim" ]] && CLAIM_QUEUE+=("$(cat "$OUT_DIR/$tag.claim")")
done

printf '%s\n' "-------------------------------------------------------------------------"
echo "结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）："
for q in "${CLAIM_QUEUE[@]}"; do
  check_claims "${q%%|*}" "${q#*|}" || claim_bad=$((claim_bad+1))
done
printf '%s\n' "-------------------------------------------------------------------------"
echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad ／ 产物已归档 $archived"
echo "本轮输出：$OUT_DIR"
if [[ $archived -ne 0 ]]; then
  echo "  ! 「产物已归档」$archived 行：留存产物按「每次提交删上一次的实验记录」归档进了版本库，逐字节这一档没有对照物。"
  echo "     这不是判红——这几行本次都跑得出来，结论区间断言照常判。要看当时的产物："
  echo "         git log --all --diff-filter=D --name-only -- \"*<产物文件名>\"     # 找到删它的那次提交"
  echo "         git show <提交>^:research/results/<产物文件名>                      # 读回当时的内容"
  echo "     读到的是当时的数、不是今天的结论；要拿它支撑新结论就重新跑一遍（evidence-discipline.md）。"
fi
if [[ $drift -ne 0 || $broken -ne 0 || $claim_bad -ne 0 ]]; then
  echo "  → 怎么办：「跑不了」看上面那一行给的 $OUT_DIR/<实验号>.err；「对不上」按上面给的 diff 命令看差在哪，" \
       "结构性差异是代码改动带来的就更新入库产物，不是就说明代码退化了；" \
       "「结论断言不中」逐条去 check_claims() 里对应实验号那一段读注释——是该改 kb 里记的区间，还是真的退化了，" \
       "两种都要人判，不许为了让这里变绿就调宽容差（test-discipline.md）。"
fi
if [[ $drift -eq 0 && $broken -eq 0 && $claim_bad -eq 0 ]]; then
  # 全绿且用的是自动分配的临时目录 ⇒ 收拾掉。有一条不绿就留着，上面的提示指着它。
  [[ -z "${REPLAY_OUT:-}" ]] && rm -rf "$OUT_DIR"
  exit 0
fi
exit 1
```

**出处 `.claude/hooks/agent-write-scope.tsv:1-23`（整段抄，未转述）**

```markdown
# 项目 subagent 用 Write / Edit 时的写范围：一个 agent 一个或几个路径模式，一行一个。由 agent-write-scope.sh 在 PreToolUse 里判。
# 三列用制表符分隔：agent 名、路径模式、出处。路径模式相对仓根，以 / 开头的是绝对路径；** 跨目录，* 不跨目录。
# ⚠️ 模式里只有 ** 与 * 两个元字符：write-guard.sh 的 glob_to_regex 把别的字符一律 re.escape，
#    写 e[0-9]*.rs 会被当成字面的「e[0-9]」、永远匹配不上，而闸只报「越出写范围」、不会说模式没写对（2026-09-22 踩过）。
# 只登记 tools 里有 Write 或 Edit 的定义；表与定义逐项一致由 63-agent-write-scope.sh 判。Bash 里的写不经过这道闸。
implementation-writer	crates/**	implementation-writer.md「写范围」
implementation-writer	litmus/**	implementation-writer.md「写范围」
implementation-writer	/tmp/claude-1000/**	报告文件与草稿目录（定义不许它写 kb 与 research/）
kb-scribe	.claude/kb/**	kb-scribe.md「写范围」；relabel-item.py 改写的别处文件只许那个脚本改，不经 Edit
kb-scribe	/tmp/claude-1000/**	报告文件与草稿目录
experiment-runner	research/e7-index-bench/src/bin/**	experiment-runner.md「写范围」：bin 源文件
experiment-runner	crates/singlefs-harness/src/bin/e*.rs	实验装置要驱动真实 crates/ 代码时的只读 bin（2026-09-22 E156 撞上：岔路问的是「今天这份代码的性质」，另写一份独立模型答不了）。限定 e 开头加数字，碰不到 first_transaction_* 那几个生产装置；这个 bin 不许改 core/checker，由代码三方与门禁 56 号管
experiment-runner	crates/mutations.tsv	入库装置的变异行只追加进末尾（experiment-runner.md 第 2 步与「写范围」；2026-09-23 E158 撞上：闸拒了那次追加，三行落进 research/mutations/ 下、门禁 59 号复跑不到）
experiment-runner	research/mutations/**	变异表
experiment-runner	research/results/**	这个实验的产物
experiment-runner	research/scripts/replay.sh	复跑登记行
experiment-runner	research/e7-index-bench/Cargo.toml	在末尾追加这个实验的 [[bin]]（agent-defs-r2 写范围探针：138 个实验 bin 有 100 个显式登记，副本里的试跑测不到这一处被拒）
experiment-runner	research/prompts/e*-preregistration.md	跑前登记的「修订」一段
experiment-runner	.claude/kb/experiments/**	实验页
experiment-runner	.claude/kb/experiments.md	实验索引行
experiment-runner	.claude/kb/experiments-history.md	重跑已有实验时这个实验的历史条目（agent-defs-r1 判决 D2）
experiment-runner	research/prompts/e*-r*-prereg.md	重跑已有实验的重跑登记（agent-defs-r1 判决 D2）
experiment-runner	/tmp/claude-1000/**	报告文件与草稿目录
```
