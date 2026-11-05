# watchdog-task-half-r1：看门狗删掉盯任务服务那一半——第一轮判决（2026-11-05）

## 一、这一轮的材料与证据

- 正文：`research/prompts/_watchdog-task-half-r1-body.md`；背景材料：`research/prompts/_watchdog-task-half-r1-background.md`（`three-way-materials.py` 带 `--files .claude/kb/tooling.md`，任务 6682）；开工快照：`research/prompts/watchdog-task-half-r1-snapshot/`（`judged-sha256.txt`、`kb-sha256.txt`、`batch-49.patch`、`batch-49-readme.patch`）。
- 正推腿（Sonnet，格 K1）：`research/prompts/watchdog-task-half-r1-sonnet-output.md`（227 行），没写模型。
- 攻方腿（Opus，格 K2、K3）：`research/prompts/watchdog-task-half-r1-opus-output.md`（sha256 3d976ebe…），模型 `research/prompts/watchdog-task-half-r1-opus-model/`（`old-watchdog-model.py` 400724413de7…、`overrun-probe.sh` ecea7aad5fcd…、`tools/rerun.sh` e0c1a4aa2b55…）。
- 倒推开工树与核引文（原样；倒推经 task-server 交，`--cwd` 给判决草稿目录）：

```
python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py watchdog-task-half-r1 --out <草稿目录>/watchdog-task-half-r1-tree --repository /home/fy5090/code/singlefs
python3 scripts/agents/cite-check.py research/prompts/watchdog-task-half-r1-sonnet-output.md research/prompts/watchdog-task-half-r1-opus-output.md --root <草稿目录>/watchdog-task-half-r1-tree --background research/prompts/_watchdog-task-half-r1-background.md
```

- 倒推结果（任务 6718）：「✓ 快照 watchdog-task-half-r1：清单点名 6 份，倒推出 6 份，sha256 对上 6 份」。腿跑着的时候快照里的文件没被改过。
- 引文核对：对开工树「✓ 核了 60 处引文、78 处裸行号，对上 27 处，没判 111 处」。没判的都是快照之外的文件（`task-server/`、`scripts/agents/watch.sh` 之外的脚本）。改用主工作区为根再核一遍：23 处对不上，其中 22 处写「原文在这份文件第 N 行」，1 处「行号指到标题行」（`task_server.py` 2093 → 2094）。全是腿交回之后主 agent 打了批 50、task-server 几份文件往下挪了几行，引文原文都在。
- 主 agent 复跑（都经 task-server）：
  - 攻方模型 `tools/rerun.sh`（任务 6787；`--frozen` 照腿写的给仓根，主工作区 `agent-watch.py` 自开工没改过，脚本开头核 sha256 过）：「name=before_sha256 got=f6ed737f…（与 want 相同）」，11 种情形「before=<改前报的告警> want=… verdict=pass」，「name=after-readers functions=无 lines=无 want=无 verdict=pass」，「name=done cells=12 failed=0」。之前两次（6761、6783）退 1，是主 agent 把 `--frozen` 给成了只有 6 份文件的快照树，不是模型的问题。
  - 服务自证两格（任务 6749）：`down-host-waits-not-never`、`card-fault-replaced-on-another-card`「2 格都对」。
  - 活探针（任务 6750 头一次 5 秒；6769 第二次）：第二次在第二台跑满 1511.052 秒、结局 `succeeded`，服务没停、没告警；`task status` 只写「已超过记录用时的两倍、在用核数不到线程数 31 的四分之一」。它是插空起的（预计 11.1 秒，挡路时长 96.2 秒）；排头 6754 在它起跑 96 秒后照样起了（探针只占一点内存），所以这一次插空没有挡住排头。
  - 批 49 的十八条（6639–6656）：两份整份自证绿；九条弄坏开关各红在点名的格；旧键 `任务:2284:排队过久` 被拒（6650「不是 --ack 认的形态」）；旧配置键 `task-queue-minutes` 被拒（6652「agent-watch.py 不认」）；旧选项 `--task-queue-minutes` 被拒（6654）；`类:` 键照收（6651）。
  - 正推腿推的 `--ack 本机:根分区余量过低`（任务 6712）：退 2，「「本机:根分区余量过低」不是 --ack 认的形态」。
- 主 agent 现查：
  - `task-server/task_server.py:77`、`:301`、`:2094`：超跑停要「已跑超过记录用时的 5 倍、日志又满 STALL_SECONDS 秒没涨」才停，日志在涨的不停。
  - `.claude/main-agent.md`、`.claude/agent-common.md` 里 `control check`、`control watch` 零命中。
  - `.claude/gate.d/break-switch-replay-files.tsv:63` 写「一百零二条」，`grep -cP '^\s+AGENT_WATCH_BREAK=' scripts/agents/agent-watch.py` 现数 79。
- 核不动的：攻方 K3「`--ack 类:` 经真起的看门狗子进程那条路零格」靠一条变异（`acknowledgements()` 滤掉 `类:` 之后整套自证照绿）才量得到，这一轮没做，算线索。攻方「cleanup-pending、shard-advance-failed 没有结局、wait 不醒」只读了代码，模型那两格是按代码造的情形，没在真服务上触发过。

## 二、逐条判决

「采」是主 agent 现查或复跑坐实的；「线索」是腿读出来、主 agent 没量到的。

| 格 | 判 | 依据 |
|---|---|---|
| K1 改后定义与脚本一致 | 采 | 正推腿逐句判第 63、67、73 行（S1–S13），与 `agent-watch.py`、`watch.sh`、`watch.conf` 一致；批 49 十八条复跑都照预期 |
| K1 定义里漏了服务健康怎么知道 | 采 | 删掉「修好后 `control check` 回读到退 0」之后，定义里 `control check`、`control watch` 零命中；两样在 `task-server/control` 里都有 |
| K1 补丁前就有的三处说不清（漏「根分区余量过低」、只记不叫醒与叫醒的混写、别的叫醒告警读到之后怎么办） | 采 | 正推腿引 `agent-watch.py:513`（`INFO_ALERT_NAMES`）、`:1924`、`watch.conf:23`；不是批 49 带来的，但批 49 把句首收成「它只管 agent」之后更不合 |
| `watch.conf:24` 推荐的 `--ack 本机:根分区余量过低` 过不了 | 采 | 任务 6712 退 2；补丁前就这样 |
| K2 排队过久没人报 | 采 | 模型两格 pass；服务自证 `down-host-waits-not-never` 绿 |
| K2 跑得远超预计（日志在涨）没人报 | 采 | 活探针 6769 跑到记录用时的 136 倍，服务不停、不告警；模型那一格 pass |
| K2 主 agent 不在 `task wait` 上时异常结局没人报 | 采（有条件） | 模型那一格 pass；改前也只在看门狗在跑的那一段报 |
| K2 服务连得上却不健康没人报 | 采 | 模型那一格 pass；连不上的那一半照旧经 `wait` 退 78 |
| K2 反复重试 | 不算漏报 | 重试都有上限，到顶有结局（攻方「没打中的形状」第一条） |
| K2 服务告警没人读 | 采 | 模型「after-readers functions=无 lines=无」；服务自证 `card-fault-replaced-on-another-card` 绿：换卡重排跑成时结局是 `succeeded`，`.claude/main-agent.md` 那条「末行是 card-fault 就弹窗」接不到 |
| K2 cleanup-pending、shard-advance-failed | 线索 | 只读了代码，没在真服务上触发 |
| K2 定义里别处还在引已删的东西 | 采 | 射程内只有 `.claude/hooks/selftest-registry.tsv:32` 的说明；射程外 `break-switch-replay-files.tsv:63` 条数不对（现数 79） |
| K3 删掉的 24 + 1 条开关 | 采：删得对 | 两腿都没找到还管着留下一半的；`ack`、`ackscope` 点名的新格核的还是原来那件事（攻方第 105 行一节） |
| K3 `--ack 类:` 端到端零格 | 线索 | 没做变异，见「核不动的」 |

**这一轮的结论**：批 49 改的那几句与脚本一致，删掉的开关和格也删得对；但删掉盯任务那一半之后，有五种情形此刻没有任何一处会报出来（排队过久、日志在涨的远超预计、不在 `wait` 上时的异常结局、服务连得上却不健康、服务写的告警），用户删它时的前提「任务系统自带了提醒」对这五种还不成立。

## 三、怎么办（改法都被攻过零轮）

不用交用户、照条款就能改的（并进下一批工具员）：

| 改法 | 改哪 |
|---|---|
| 定义「派出去之后」写明服务健康靠 `control check` 与后台起的 `control watch`；补上「根分区余量过低」；只记不叫醒的与叫醒的分开写；读到别的叫醒告警怎么处置写一句 | `.claude/main-agent.md`（与别的会话在同一份里的改动不碰） |
| `watch.conf:24` 与 `agent-watch.py:118` 推荐的 `--ack` 写法改成认得的 `类:根分区余量过低` | `scripts/agents/watch.conf`、`scripts/agents/agent-watch.py` |
| `--ack 类:` 经真起的看门狗那条路补回一格，配一条变异证红 | `scripts/agents/watch.sh` 的自证 |
| 两处过时的登记说明改成现状 | `.claude/hooks/selftest-registry.tsv:32`、`.claude/gate.d/break-switch-replay-files.tsv:63` |

要交用户定的岔路（五种没人报的情形补在哪）：

| 候选 | 是什么 | 代价（推的，没量过） |
|---|---|---|
| 甲 `task wait` 自己报 | 服务把这条任务的排队过久（带在等什么）、跑过预计 N 倍、换卡重排、清场没收尾这几样推给在等它的 `wait`；`wait` 打一行提醒、退一个专用的「要处置、任务还在」码，主 agent 处置完再起 `wait`；服务健康另由后台的 `control watch` 盯 | 改 task-server 的 `wait` 与服务的推送；不在 `wait` 上时照旧收不到 |
| 乙 `control watch` 一处盯全会话 | 每个会话后台起一只 `control watch`，它读服务的告警表与本会话全部任务的状态，按阈值叫醒 | 等于把看门狗删掉的那一半挪进 task-server；不在 `wait` 上也收得到 |
| 丙 不补 | 承认这五种没人报，主 agent 自己定时看 `task status` | 零改动；漏报照旧 |

另记一条（推的，没量过）：插空只信预计用时。插进去的任务若占了排头要的那份内存、预计用时又错得离谱、日志一直在涨，排头会被挡到它跑完，服务不停它。活探针这一次只占一点内存，没撞上。

## 回看决策

不涉及决策：这一轮判的是看门狗脚本与主 agent 定义里「派出去之后」一节，`.claude/kb/decisions/` 里没有一条压在它们上面。

## 这一轮被判过的文件

- `.claude/main-agent.md`
- `scripts/agents/agent-watch.py`
- `scripts/agents/watch.sh`
- `scripts/agents/watch.conf`
- `scripts/agents/README.md`

## 判决之后（2026-11-05）

用户在弹窗里选「task wait 自己报」：服务把这条任务的排队过久（带在等什么）、跑过预计 N 倍、换卡重排、清场没收尾推给正在等它的 `wait`，`wait` 打一行提醒、退专用的码，主 agent 处置完再起 `wait`；服务健康另由后台起的 `control watch` 盯。改法并进下一批：task-server 那一侧与定义、看门狗那一侧各派一个工具员，文件不相交。

定义与看门狗那一份补丁（a3aa1293…）打进主工作区之后，用户在弹窗里选「我逐份豁免」（不再为这几处开三方）。`.claude/main-agent.md` 里同时有别的会话「删轻型并入重型」那一批没审过的改动，照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 9 条不登记哈希：等那几处审过或提交之后，按那时整份的 sha256 登进 `.claude/agent-def-review-exempt`。
