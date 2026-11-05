# d137-ts-defs-r1：统一任务服务那两批定义改动——第一轮判决（2026-10-31）

## 一、这一轮的材料与证据

- 正文：`research/prompts/_d137-ts-defs-r1-body.md`；开工快照：`research/prompts/d137-ts-defs-r1-snapshot/files-sha256.txt`（30 份）。
- 正推腿（Sonnet，格 F）：`research/prompts/d137-ts-defs-r1-sonnet-output.md`（441 行，sha256 d3803490…）。
- 攻方腿（Opus，格 A）：`research/prompts/d137-ts-defs-r1-opus-output.md`（295 行，sha256 66c87022…），模型 `research/prompts/d137-ts-defs-r1-opus-model/`。
- 倒推开工树与核引文（原样，经 task-server 交，`--cwd` 给本机草稿目录）：

```
python3 scripts/agents/snapshot-tree.py d137-ts-defs-r1 --out <草稿目录>/tree --repository <仓根>
python3 scripts/agents/cite-check.py research/prompts/d137-ts-defs-r1-sonnet-output.md research/prompts/d137-ts-defs-r1-opus-output.md --root <草稿目录>/tree --background research/prompts/_d137-ts-defs-r1-body.md
```

- 倒推结果（任务 1831）：清单 30 份，倒推出 27 份、sha256 都对上；倒推不出 3 份，主 agent 另行倒推：
  - `.claude/agent-common.md`（快照 4f111ee5，此刻 8858e562）：快照之后只改过一处，是主 agent 在腿跑着时写进用户 2026-10-31「命令必须要精确……不允许单次放宽或者一次豁免」那一句（违「腿跑着的时候主 agent 不改这些文件」）；把那一次替换反着做一遍，sha256 回到 4f111ee5，原样存在判决草稿目录。两条腿都写明引它的行是变后现取的，引到第 74 行那一句的逐处人工核。
  - `.claude/skills/crash-test/commit-stages.md`（快照 21973c2f，此刻 7f32fae5）：暂存区那一份就是快照原样；快照之后别的会话照用户「工具里不限时」改了第 2b 步 ③ 的两行（删超时升档），腿没有引那两行。
  - `.claude/hooks/lib_heavy_tests.py`（快照 d3e72828，此刻 cc11e40f）：别的会话换上，改的是 `prove_red_use` 拆包，不碰这一题的格；喂钩子的复跑用的是换上之后那一份，读数与腿报告逐格相同。
- 引文核对（任务 1832）：「✓ 核了 135 处引文、183 处裸行号，对上 77 处，没判 241 处（其中裸行号没有引文可比 135 处；逐处列在上面）」，没有对不上的。
- 主 agent 复跑腿的模型与样本（都经 task-server、`--cwd` 本机草稿目录）：
  - 正推腿三份喂钩子脚本（任务 1827–1829）：C01 rc=2、C02 rc=0、C03 rc=2、C09 rc=2、C11 rc=2、D01 rc=2、D03 rc=2、D04 rc=2、D12 rc=2、E01 rc=2、E02 rc=2、E03 rc=2、E07 rc=2、E11 rc=0，与腿报告逐格相同。
  - 攻方腿 `tools/rerun.sh`（任务 1837，副本照腿给的 rsync 重建 190M、跑完删）：`name=h1-light-crash-in-copy hook=heavy-test-guard exit=2 expected=0`、`name=w2-kb-scribe-precheck-out-of-scope hook=write-guard exit=0 expected=2`、`name=w3-precheck-input-missing hook=write-guard exit=1 expected=not-2`，其余 7 条与期望相同，`name=feed_done samples=10`。
- 主 agent 自己撞上的两处读数：任务 1822（`--cwd` 仓根）被派到第二台、在那边建 `/home/fy5090/…` 没权限退 1；任务 1801（`cargo clippy … --all-features`）提交回显 `gpu=1`「要卡：带 gpu 特性的 cargo」。攻方腿任务 1790 `succeeded` 而本机没有它写的 `out/peer-probe-4.txt`（`task status 1790`：第 1 次在第二台）。
- 核不动的：A5（`--no-run` 那一条会不会记成要卡）的提交按判法是重型，这一轮没交；F12 同一件，判为线索。

## 二、逐条判决

「采」是主 agent 现查坐实的；「线索」是腿读出来、主 agent 没量到的；所有改法被攻过零轮。

### F　正推腿（定义写的与钩子、服务对不对得上）

| 条 | 腿的判 | 主 agent 核 | 判 |
|---|---|---|---|
| F1 | 执行员第 42、59 行与工具员第 35 行写「命名检查同样交主 agent」，共用约束第 104 行与交回闸要它们自己交 | 读原文坐实（同 A3） | 采 |
| F2 | `rerun-failed-tests.py` 打出的命令已带 `task-server/task submit --`，共用约束第 59 行与三份定义还让插一次 | 读 `scripts/gate/rerun-failed-tests.py:9` 坐实；E11 rc=0，钩子放两层 submit | 采 |
| F3 | `build=` 是 `target` 的上一层，执行员第 29、38 行当成 `target` | 读 `task-server/task_server_lib.py:122` 坐实 | 采 |
| F4 | 共用约束第 63、77、82 行教子 agent 直接跑 `proc.py`，钩子拒 | C01、C03 rc=2 坐实 | 采 |
| F5 | 执行员「非 succeeded 的输出不算产物」与 `mutate.sh` 有没红退 1、两份 skill 把 command-failed 当有结果矛盾 | 读的；判法上成立 | 采 |
| F6 | M07 被悄悄定成「要前缀」（共用约束第 59 行方括号、第 61 行写「带」），钩子文字写「不要」 | 读的；M07 用户没定 | 采，交用户 |
| F7 | 共用约束第 73 行「卡住」漏了服务判法里的宽限与「CPU 也没涨」 | 读的 | 线索，改成指到服务文件头 |
| F8 | 「服务对子 agent 交的重型当场拒」：拒的是钩子，服务不分主 agent 与子 agent | 读的（同 A7） | 采 |
| F9 | 钩子对认不出的写法给的出路是交回，与共用约束第 74 行、用户原话「改写法」相反 | 读的 | 采，钩子出路归别的会话 |
| F10 | 主 agent 定义等处十一处本仓脚本命令（外加 `stage-mine.py`）写直接跑，钩子拒 | D01、D03、D04、D12、E01、E02、E03、E07 rc=2 坐实 | 采 |
| F11 | 要整份日志时多数定义没写 `task log <号> --all` | 读的 | 线索，措辞改 |
| F12 | GPU 手跑先交 `--no-run`：服务对 `--no-run` 没有例外，只编的那条照样要卡 | 没量（同 A5） | 线索 |
| F13 | 定义漏了 `SINGLEFS_THREAD_CAP` 被客户端拒 | 没复跑 | 线索，措辞补 |
| F14 | 实现员定义里重型默认不交回那一处 | 读的 | 线索 |
| F15 | 执行员「耗时类的量交主 agent 跑」与 M01 不一致 | 读的；件 18b 留它是有意的（测时要独占机器） | 不采：留，交用户 |
| F16 | 「直接执行编出来的二进制」字面被拒（C09），换读法（交二进制）对得上 | C09 rc=2 坐实 | 采，措辞改成交 |
| F17 | `--cwd` 给仓根可能被派去第二台 | 坐实（同 A1） | 采 |
| F18 | 子 agent 与主 agent 同一会话号，`status`、`cancel`、`clean` 碰得到别人的号 | 读的（同 A9） | 采，措辞改 |
| M05 | 被定成「提交时不写线程、一律登记值」，与服务一致 | 读的 | 采，交用户确认 |
| M09 | 没被定 | 读的 | 撑住 |

### A　攻方腿（找漏洞）

| 条 | 腿的判 | 主 agent 核 | 判 |
|---|---|---|---|
| A1 | 从仓根交、不带 `--cwd` 的命令被派到第二台，写出的文件不回本机，任务照样 `succeeded`；21 份没有一处提到 | 坐实两次：任务 1790、主 agent 的任务 1822 | 采 |
| A2 | 实现员在副本里交崩溃点重放轻型，重型测试闸当重型拒，出路叫它标「重型」交回，与「轻型自己交」矛盾 | h1 复跑 exit=2 坐实 | 采 |
| A3 | 同 F1 | 同 F1 | 采 |
| A4 | 实现员 clippy 带 `--all-features`，被服务记成要卡：占卡、先清场、跑完不复原 | 主 agent 任务 1801 回显 `gpu=1` 坐实 | 采 |
| A5 | 「手跑的 GPU 测试分两条交」两条都占卡、第二条从头编 | 没量（提交是重型） | 线索 |
| A6 | `task wait` 起头连不上退 78，出路叫子 agent 重交，原任务还在跑，同一件跑两份 | 读的 | 采，措辞改 |
| A7 | 同 F8 | 同 F8 | 采 |
| A8 | 经 submit 交的 cargo 不认合入锁 | 读的 | 线索，归服务与钩子 |
| A9 | 同 F18 | 同 F18 | 采 |
| A10 | 写回员预检样本没写工作目录，任务工作目录不是仓根时范围外的路径放行；输入读不到退 1 被当成过 | w2 exit=0、w3 exit=1 复跑坐实 | 采 |
| A11 | 分片任务的 `task log <号>` 给的不是没成的那一片 | 读 `task-server/task` 末行 `shard=` 一段 | 采，措辞改 |
| A12 | 服务不通时出路是 `control start`，子 agent 起它被拒；定义没写 78 怎么办 | 读的 | 采 |
| A13 | 共用约束「副本只拷要编的」拷不进 `scripts/`，轻型三条要在副本里起 `scripts/crash/…` | 读的 | 采 |

## 三、怎么办

做法本身（经服务交、服务定资源）撑住，没有一条打到做法。打中的都是写出来的定义与钩子、服务对不上的地方。下面的改法都是零轮。

| 改 | 改哪里 | 归谁 |
|---|---|---|
| 命名检查交回前自己交（F1/A3） | `.claude/agents/experiment-runner.md`、`tooling-writer.md` | 定义（这一会话） |
| `rerun-failed-tests.py` 的命令原样交、不再插 submit（F2） | `.claude/agent-common.md` 第 59 行附近与引它的定义 | 定义 |
| `build=` 下是 `target/release/<bin>`（F3） | `.claude/agents/experiment-runner.md` | 定义 |
| 子 agent 停任务用 `task cancel`，`proc.py` 只主 agent 用（F4） | `.claude/agent-common.md` 第 63、77、82 行附近 | 定义 |
| 结局按 `type=` 与命令自己的退出码分开读（F5） | `.claude/agents/experiment-runner.md`、两份 skill | 定义 |
| 卡住判法指到服务文件头（F7）、`task log --all`（F11）、`SINGLEFS_THREAD_CAP` 被拒（F13）、二进制交服务跑（F16）、分片读末行 `log=`/`shard=`（A11） | 共用约束与相关定义 | 定义 |
| 「服务当场拒」改「钩子拒」（F8/A7）、同一会话号的 cancel/clean 只碰自己交的号（F18/A9） | 共用约束、主 agent 定义 | 定义 |
| 主 agent 起本仓脚本一律经 submit（F10） | `.claude/main-agent.md`、`three-way-inference.md`、`commit-stages.md`、`experiment-designer.md` 点名的十一处 | 定义 |
| 要往仓里或草稿目录写文件的，`--cwd` 给本机草稿目录或写绝对输出路径；第二台写的文件不回本机（A1） | 共用约束「子 agent 不交重型」 | 定义；服务侧「可能要写就钉本机」归别的会话 |
| `task wait` 退 78：先 `task status <号>`，在跑就再起 wait、不重交（A6）；子 agent 遇 78 不起 `control`，写进进度交主 agent（A12） | 共用约束 | 定义 |
| 写回员预检样本带 `"cwd"`、判据改「`exit=2` 停、`succeeded exit=0` 算过、别的一律停」（A10） | `.claude/agents/kb-scribe.md` 第 31 行 | 定义 |
| 「副本只拷要编的」把轻型要起的 `scripts/` 列进去（A13） | 共用约束第 46 行附近、实现员定义 | 定义 |
| 副本里逐字节相同的 `crash-point-replay.sh --quick` 照轻型认（A2） | `.claude/hooks/lib_heavy_tests.py` | 钩子（别的会话） |
| clippy 只编不跑的不记要卡（A4）、`--no-run` 不记要卡（A5/F12）、submit 时认合入锁（A8）、钩子对认不出写法的出路改成「改写法」（F9） | `task-server/task_server.py`、钩子 | 服务与钩子（别的会话） |

交用户的：

| 问 | 依据 | 零轮 |
|---|---|---|
| M07：重型要不要 `SINGLEFS_HEAVY_TESTS` 前缀（定义多处写「带」，钩子文字写「不要」） | F6 | 是 |
| M05：「提交时不写线程、一律登记值」照现状定下来 | 正推腿第四节 4.1 | 是 |
| 执行员测时类的量照旧交主 agent 跑，还是也自己经服务交 | F15 | 是 |

## 回看决策

不涉及决策：被判的是项目的协作定义、规则与 skill，不碰 `.claude/kb/decisions/` 下任何一条。

## 这一轮被判过的文件

- `.claude/agent-common.md`：F2、F4、F6、F7、F8、F11、F13、F18、A1、A6、A12、A13 打中（零轮修补）；快照之后主 agent 改过一处，倒推成立
- `.claude/main-agent.md`：F10、F18 打中（零轮修补）
- `.claude/agents/clerk.md`：两条腿都没打中
- `.claude/agents/experiment-designer.md`：F10 一处（零轮修补）
- `.claude/agents/experiment-runner.md`：F1、F3、F5 打中（零轮修补）；F15 交用户
- `.claude/agents/implementation-writer.md`：A2（钩子侧）、A4（服务侧）、A13 牵到；F14 线索
- `.claude/agents/investigator.md`：两条腿都没打中
- `.claude/agents/kb-scribe.md`：A10 打中（零轮修补）
- `.claude/agents/mutation-triage.md`：两条腿都没打中
- `.claude/agents/three-way-attack.md`：F16 一处（零轮修补）
- `.claude/agents/three-way-defense.md`：两条腿都没打中
- `.claude/agents/three-way-forward.md`：两条腿都没打中
- `.claude/agents/tooling-writer.md`：F1 打中（零轮修补）
- `.claude/rules/implementation-workflow.md`：M05 那一句交用户确认
- `.claude/rules/verification.md`：F12/A5 线索
- `.claude/rules/three-way-inference.md`：F10 几处（零轮修补）
- `.claude/rules/test-script-discipline.md`：两条腿都没打中
- `.claude/skills/gate/SKILL.md`：两条腿都没打中
- `.claude/skills/gate/staged-gate.md`：F5 一处（零轮修补）
- `.claude/skills/crash-test/SKILL.md`：两条腿都没打中
- `.claude/skills/crash-test/commit-stages.md`：F5、F10、A11 打中（零轮修补）；快照之后别的会话改过两行，暂存区那一份是快照原样
