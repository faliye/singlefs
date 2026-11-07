# task-server 深入审核（2026-11-01）

## 范围、基线与结论

本记录合并首轮审核与继续深挖的结果，针对工作树实际源码，而非仅审查 HEAD 差异。审核时 HEAD 为 `59ce5a86550de2c2c9bc6b51b34b4f881313682c`，目录包含已暂存新增文件及未暂存修改。源码未因本次审核而修改，未提交修复。

重点追踪 `task_server.py` 的 Store、调度线程、HTTP 线程、启动/取消/完成/恢复状态转换、分片推进、签名校验，辅查 `task_server_lib.py`、`attempt_runner.py` 和 `control`。不是全文件逐行覆盖证明，也不是现网任务事故统计。

确认 **6 项问题，P1 4 项、P2 2 项**。P1 表示应优先修复的执行正确性/恢复问题；P2 表示有具体触发条件的错误行为。以下各项都有明确状态序列；初轮动态证据使用真实 Store/SQLite 与生产方法，外部启动和停止通过桩替代。同日复核增加了真实临时 systemd 单元验证，见“逐项复核结果”；未启动或停止用户的既有任务。

| 编号 | 优先级 | 问题 | 证据类型 |
| --- | --- | --- | --- |
| TS-01 | P1 | 已成功取消的任务被启动线程复活 | 两线程确定性交错、真实 SQLite |
| TS-02 | P1 | 子任务完成与父任务推进之间的崩溃无法恢复 | 持久化崩溃状态构造、真实 reconcile |
| TS-03 | P1 | attempt 插入与 task 更新分开提交，恢复后重复启动同一尝试 | 提交边界故障注入、真实恢复/启动方法 |
| TS-04 | P2 | 分片取消失败后重试谎报 stopped | 停止失败注入、两次真实 cancel |
| TS-05 | P2 | task_id 索引列不受签名验证，能错配任务与尝试 | 单列 SQL 修改、真实签名验证 |
| TS-06 | P1 | systemd-run 超时被当作确定未启动，可能遗留失管单元 | TimeoutExpired 注入、读回调用计数 |

## TS-01：取消成功后仍启动，并清掉取消标记

**位置：** `task_server.py:2683-2742`（`start_attempt`），`2961-3002`（`cancel`），`438-457`（`Store.update/write_row`），`3210-3217`（HTTP 取消入口）。

`start_attempt` 只在开头读一次最新状态，确认 `queued` 且未取消；随后创建目录、构造环境、写 spec、插入 attempt，最后使用调用方持有的旧 task 字典覆盖数据库。HTTP 取消在另一线程执行。Store 的锁只保护单次 UPDATE，不保护“读取状态—决定启动—写入状态”这一整段。

可达交错：

1. 调度线程读到任务 queued，越过启动前检查。
2. 暂停在 `ensure_private_directory`，HTTP 线程执行取消；数据库变成 `finished/cancelled`，返回 `200, stopped=True`。
3. 调度线程继续，用旧 task 写回 `running`，旧字典的 `cancel_requested=False` 也写回，随后调用 launch。

实测关键输出：

```text
cancel_response=(200, {'stopped': True, 'state': 'finished', 'outcome': 'cancelled', ...})
after_start={'state': 'running', 'cancel_requested': False, 'outcome': None} launched=[1]
```

影响不是状态显示延迟：任务可在调用方已获取消成功后开始产生外部副作用。相同的整行旧记录覆盖模式也应检查 finish/retry 与 cancel 之间的交错；这里不把同一根因的变体重复计数。

**修复要求：**引入任务级串行状态机或带版本/CAS 的事务转换，将启动意图与 attempt 创建关联起来。若启动已进入不可撤销阶段，取消必须记录取消意图并等待/核实单元停止，不能简单返回“排队时取消，没起”。仅在 launch 前再查一次仍会留下新的检查—使用窗口。

**回归验收：**精确阻塞在启动前检查之后再取消；取消成功后必须没有启动，或者取消接口必须等已启动单元停止才确认。任务不得由 finished 回到 running。

## TS-02：分片完成通知不是可恢复的持久化状态转换

**位置：** `task_server.py:1668-1675`（`finalize`），`1721-1766`（`advance_parent`），`1277-1339`（恢复），另见 `2914-2918`（父任务提交与计划子任务创建）。

`finalize` 先将子任务 `finished` 提交，再调用 `advance_parent`。如果在两者之间退出，或 `advance_parent` 自身抛错，子任务已不是恢复扫描的 running 任务，attempt 也已 ended。`settle_ended_attempts` 只扫描 running task；`reconcile` 只扫描 starting/running attempt。两者都不会补推进仍为 sharded 的父任务。

最小崩溃状态：父任务 `state=sharded, phase=plan`；其唯一计划子任务 `state=finished, outcome=succeeded`；没有活动 attempt。调用真实 `reconcile` 后：

```text
after_reconcile_parent=sharded children=1
```

父任务一直等待，不会生成分片。最后一个分片完成后、创建 merge 之前，以及 merge 完成后、父任务完成之前也存在同类窗口。提交父任务后尚未创建 plan 子任务，或生成多个 shard 的中途退出，则还会留下不完整的子任务集合。此处合并为一个“分片状态机不可恢复”问题。

**修复要求：**持久化父任务期望阶段和完整子任务集合/创建意图，启动恢复及每轮对账都检查非终态父任务。生成子任务必须幂等，建议对 `(parent_id, role, shard_index)` 设置业务唯一性约束，并在事务里推进阶段。不能只重新调用 `advance_parent`：部分子任务已创建时会重复生成或过早判断全部完成。

**回归验收：**在父任务插入后、子任务终态提交后、每个 shard 插入后、merge 插入前后分别断点退出；重启必须最终推进，且每个逻辑子任务只有一份。当前最小复现直接构造崩溃后的合法数据库状态，未声称执行过上述全部 kill 点。

## TS-03：创建 attempt 与 task 转 running 不原子

**位置：** `task_server.py:2738-2740`，`1329-1332`，`2754-2766`。

attempt 插入和 task 更新是两次独立事务。第一次提交之后进程退出，会留下 `task=queued, attempt=starting`，任务的 `attempt_count` 仍为 0。

重启后 `reconcile` 能找到 starting attempt，并在单元不存在、未有 started 标记时调用 launch。但 launch 只更新 attempt，不把对应 task 修成 running。随后正常排队仍会看到这个 queued task，按 `attempt_count+1` 再建一次编号为 1 的 attempt，使用相同单元名、日志及 spec 路径。

故障注入在 task UPDATE 之前抛出模拟进程终止，真实 Store 保留前一个提交。恢复第一次启动成功，第二次启动桩模拟 systemd 的同名单元拒绝：

```text
after_crash=queued attempts=1
after_reconcile=queued
after_second_start=unit-start-failed attempts=[(1, 'running'), (1, 'ended')] same_unit=True
```

**影响边界：**systemd 的同名单元机制通常会阻止两份同名单元同时启动，所以不把本问题描述为必然“双进程并发执行”。已经确认的是重复 attempt、共享同一 spec、以及把仍有活动尝试的任务提前记为失败；如果旧单元已经结束并被 collect，第二次也可能实际再执行命令。

**修复要求：**在同一个事务中校验 queued 状态、分配尝试号、插入 attempt 并更新 task；恢复逻辑也应检查并修复 task/attempt 一致性。对 `(task_id, number)` 增加唯一约束，作为防线而非替代事务。

**回归验收：**在 attempt INSERT 已提交、task UPDATE 未执行的位置注入退出；重启只恢复原 attempt，任务不再进排队分支，也不能因为同名单元存在而提前终态。

## TS-04：分片取消第一次失败，第二次返回虚假的停止成功

**位置：** `task_server.py:2973-2976`，`2980-2998`。

分片取消先调用 `finalize_parent` 将父任务改成 finished，然后逐个停止子任务。子任务停止失败时第一次返回 500，但父任务终态已经提交。再次取消走“已经 finished”分支，直接返回 stopped=True，不再访问活动子任务。

真实 cancel、真实 SQLite，只有 `stop_running` 注入失败：

```text
first parent cancel=(500, {'stopped': False, ...})
second parent cancel=(200, {'stopped': True, 'state': 'finished', 'outcome': 'cancelled', ...})
stop calls=[4] child state=running
```

`cancel_requested` 本身不会让 reconcile 主动再次停止单元，因而失败后的执行仍可继续。父任务提前 finished 也会让 wait 使用者过早认为整件任务已经停止。

**修复要求：**保留 cancelling 阶段；所有活动子任务停止并完成读回后再确认父任务停止。即使保留“逻辑取消先完成”的模型，后续 cancel 也必须重新检查活动子任务，且返回值要区分取消意图已接受与物理执行已停止。

**回归验收：**第一次 systemctl stop 失败，第二次成功；第二次 cancel 必须真的重试停止，不能仅根据父任务 finished 返回成功。

## TS-05：未签名的 task_id 索引列可绕过记录校验

**位置：** `task_server.py:375-378`（列），`396-408`（verified），`477-486`（select/attempts_of），`3007`（停止所选 attempts）。

签名正文含 `record.task_id`，但 SQL 的独立 `task_id` 列没有参与 select 返回值，也没有与正文比较。`attempts_of(task_id)` 和 `children_of(parent_id)` 却用这一列决定关联关系。

对真实签名的 attempt 只执行：

```sql
UPDATE attempts SET task_id = <另一个任务号> WHERE id = <attempt号>;
```

无需修改正文或签名。实测：

```text
unsigned_index={'selected_for': 2, 'record_owner': 1, 'signature_valid': True, 'unit': 'other-tasks-unit'}
```

任务 2 查询尝试时得到了实际属于任务 1 的记录，而且 verified=True。停止、清理、完成对账、分片归属使用这些查询结果时，可能操作错误任务的单元或产物。

**前提与边界：**需要数据库索引列被外部工具/损坏/绕过服务的写操作修改。本项不是远程越权漏洞，也不声称同 UID 的恶意进程无法直接破坏服务；问题是现有 HMAC 防篡改契约未覆盖决定执行对象的关键字段，当前 `record-tampered` 保护不会检出这种不一致。

**修复要求：**签名和验证同时覆盖索引列；至少在读取时比较 SQL `task_id` 与正文 `task_id`。关联查询还需核对记录实际 owner，停止/清理必须拒绝未验证的记录。

**回归验收：**只改 attempts.task_id 或子任务 tasks.task_id，所有关联读取都应检出不一致，不能停止或清理原 owner 的资源。

## TS-06：启动命令响应超时被误当成确定未启动

**位置：** `task_server.py:2770-2788`（start_unit），`2759-2763`（launch），`1294-1300`（reconcile）。

`subprocess.run(systemd-run, timeout=60)` 抛 TimeoutExpired 时直接返回 False，随后 attempt=ended、task=finished/unit-start-failed。systemd-run 是向用户级 systemd 管理器发请求的客户端；调用端超时不能证明管理器没有接受并启动单元。杀掉超时的 systemd-run 进程也不等于停止已交给管理器的服务。

在“管理器已接受，但调用端回执超时”这一前提下，任务从账上消失，既不再计入活动尝试，也不会被常规 reconcile 收尾；用户根据失败结果重交可能与仍在运行的第一次重叠。

注入 TimeoutExpired，并提供 `unit_states -> ('loaded','active')` 的可读回状态，执行真实 start_attempt/launch/reconcile：

```text
ambiguous_start=unit-start-failed active_unit_readbacks=0
```

这证明当前路径没有尝试消除歧义。**验证限制：**没有在真实 systemd 上制造 D-Bus 回执丢失，使用的是故障注入；不把每一次超时都断言为实际已启动。

**修复要求：**把超时归类为启动状态未知，保留 starting 与资源预留，通过预先持久化的唯一单元名读取状态、started/result 标记；确认没起后才记确定失败。无法判定时保留可恢复状态并告警。

**回归验收：**启动请求已创建活动单元后注入响应超时，任务必须仍受对账和取消管理，不得提前释放账面资源。

## 验证记录与不足

已运行的既有测试：

```sh
TASK_SERVER_SELFTEST_CELLS=cancel-queued,cancel-sharded-running,ended-attempt-settled-after-restart \
  python3 -B task-server/task_server_lib.py --selftest
```

结果 3/3 通过。测试日志的结尾包含通用覆盖说明，不代表跑过整套测试。上述三个正常场景未覆盖本记录的新故障窗口；自测通过不能反证本记录的问题。

独立复现使用 `Scheduler.__new__` 配置最小依赖，真实 Store 在 TemporaryDirectory 建库；对启动/停止、资源发现使用桩以隔离用户环境。TS-01 使用 Event 屏障控制两个线程；TS-02 直接构造事务提交后的恢复状态；TS-03 在 task UPDATE 前注入 BaseException；TS-04/06 注入外部操作失败；TS-05 只改未签名索引列。未执行全量 task-server 自测，未开展真实断电或真实 systemd 管理器失联测试。

未列为额外缺陷的观察：`/health` 展示 heartbeat 年龄但仅按文档约定的数据库写入/饥饿问题计算 ok；同 UID 的会话推断不是操作系统安全边界；采样峰值不是精确记账。这些需要产品契约或进一步证据，不能仅凭实现形态扩充问题数量。

## 修复顺序与基线指纹

先修 TS-01/03 的任务状态原子性与 TS-06 的启动不确定态，再补 TS-02 分片恢复状态机；随后让 TS-04 取消读回可重试，并补 TS-05 关联字段完整性验证。修复需要新增对应故障窗口测试，不能只重跑既有 happy path。

```text
task-server/task_server.py
d56a23cb49823398f7060a913e65a46697a5b830adfd81a68b6160e00d20bf18
task-server/task_server_lib.py
be7a3c0d93c7d0d8c9d99d635d1fe50286367b66454ddc294c0cbe2ed86fbb5a
```

后续行号漂移时按函数名和指纹核对。本文件只记录问题，所有条目在本次审核结束时均未修复。

## 逐项复核结果（同日追加）

用户要求完成后验证，故再次从本文提取原始复现，并增加真实 systemd 临时单元验证。**6 项均保留，未发现需要撤销的误报；仍未修复。** 本节是比前文首轮“只用外部操作桩”的证据更强的补充，不将前文首轮测试历史改写成真实 systemd 测试。

本轮 `task_server.py` SHA-256 仍为 `d56a23cb49823398f7060a913e65a46697a5b830adfd81a68b6160e00d20bf18`。三类启动测试使用真实 `start_unit`、真实 `systemd-run --user`、真实 `attempt_runner.py`，命令仅为 `/bin/sleep 20`；任务数据库、spec、日志均在临时目录。每个测试最后停止它自己生成的唯一单元，并读取 ActiveState 确认不再运行。没有启动或停止用户的既有任务。

| 编号 | 本轮强化验证 | 观察与判定 |
| --- | --- | --- |
| TS-01 | 真实调度启动，屏障安排 cancel 交错 | cancel 返回 200/stopped=True 后，真实单元为 loaded/active；确认执行层面的违约 |
| TS-02 | 真实 finalize 提交子任务后，在 advance_parent 入口注入终止 | 子任务 finished、父任务 sharded；恢复原方法并 reconcile 后仍只有计划子任务；确认状态可由真实完成路径产生 |
| TS-03 | attempt 已提交、task 更新前注入终止，随后真实 systemd 恢复和二次启动 | 两份 number=1 attempt；第二次被真实 systemd 以 Unit already loaded 拒绝，task=unit-start-failed，但第一份单元仍 active |
| TS-04 | 真实活动子单元，第一次停止注入失败，再取消父任务 | 首次 500，第二次 stopped=True；停止只调用一次，实际子单元仍 active |
| TS-05 | 只改 SQL task_id 后调用真实 stop_running，拦截外部命令避免误停 | 为 owner=2 发出的命令为 `systemctl --user stop audit-other-owner.service`，该 attempt 的正文 owner=1；确认不仅查询错配，还进入错误停止路径 |
| TS-06 | 包装器先调用真实 systemd-run 成功，再延迟退出，令调用端超时 | accepted 标记存在，task=unit-start-failed，而真实单元 loaded/active；确认“请求已生效、回执未知”会遗留失管执行 |

TS-06 包装器只延迟回执，不改变服务的状态机；为缩短测试，把进程内 START_TIMEOUT_SECONDS 临时设为 1，测试后恢复。它模拟的是启动响应不确定，不声称在真实 D-Bus 中制造过消息丢失。TS-03 也仍精确表述为“重复尝试及错误终态”，不夸大成 systemd 已允许同名双单元并发。

关键输出：

```text
TS01_REAL cancelled_response=(200, stopped=True, ...) actual_unit=('loaded', 'active')
TS02_REAL_COMMIT child=finished parent=sharded children=1
TS03_REAL outcome=unit-start-failed actual_unit=('loaded', 'active') attempts=[(1, 'running'), (1, 'ended')]
TS04_REAL first=500 second_stopped=True stop_calls=1 actual_unit=('loaded', 'active')
TS05_WRONG_STOP requested_owner=2 record_owner=1 command=['systemctl', '--user', 'stop', 'audit-other-owner.service'] response=200
TS06_REAL manager_accepted=True outcome=unit-start-failed actual_unit=('loaded', 'active')
```

相关既有自测重新运行并补入 start-once：

```sh
TASK_SERVER_SELFTEST_CELLS=cancel-queued,cancel-sharded-running,ended-attempt-settled-after-restart,start-once \
  python3 -B task-server/task_server_lib.py --selftest
```

结果 **4/4 通过**。其中 start-once 既有断点覆盖的是任务/尝试已记录后、启动单元前后，不覆盖 TS-03 的两次数据库事务之间；它通过与本次复现失败不矛盾。没有执行全套自测，也没有把正常路径通过记成上述缺陷已经修复。

下面附录追加了两份强化验证脚本，仍可用统一提取命令重建。运行前须有可用的用户级 systemd；两份脚本都会创建短暂的测试单元并在 finally 中清理：

```sh
python3 -B /tmp/singlefs-task-real-verification.py
python3 -B /tmp/singlefs-task-extra-verification.py
```

## 2026-11-02 现查（第十一批换上之后）

现查时 `task-server/task_server.py` 与 `task_server_lib.py` 的 sha256 与「修复顺序与基线指纹」一节逐字相同（`d56a23cb…`、`be7a3c0d…`），六项都没修。四份复现照附录的提取命令抽到 `~/.local/share/singlefs/work/task-server-deep-review/`（`/tmp` 路径换成那个目录），经服务交、两台都跑到，结论与本记录相同：

| 编号 | 任务（落点） | 现查输出 |
| --- | --- | --- |
| TS-01 | 3004（第二台）、3006（第二台） | `cancel_response= (200, {'stopped': True, …, 'detail': '在队里时取消，没起'})`，`after_start= {'state': 'running', 'cancel_requested': False, 'outcome': None} launched= [1]`；真单元 `actual_unit= ('loaded', 'active')` |
| TS-02 | 3004、3007（本机） | `after_reconcile_parent= sharded children= 1`；`TS02_REAL_COMMIT child= finished parent= sharded children= 1` |
| TS-03 | 3005（本机）、3006 | `after_second_start= unit-start-failed attempts= [(1, 'running'), (1, 'ended')] same_unit= True`；真单元仍 `('loaded', 'active')` |
| TS-04 | 3004、3007 | `first parent cancel= (500, …)`，`second parent cancel= (200, {'stopped': True, …, 'detail': '已经有结局了，没什么可取消'})`，`stop calls= [4] child state= running` |
| TS-05 | 3005、3007 | `unsigned_index= {'selected_for': 2, 'record_owner': 1, 'signature_valid': True, …}`；`TS05_WRONG_STOP requested_owner= 2 record_owner= 1 command= ['systemctl', '--user', 'stop', 'audit-other-owner.service']` |
| TS-06 | 3005、3006 | `ambiguous_start= unit-start-failed active_unit_readbacks= 0`；`TS06_REAL manager_accepted= True outcome= unit-start-failed actual_unit= ('loaded', 'active')` |

`start_attempt` 的形状与 TS-01 描述相同：开头读一次最新记录判 `queued` 且没取消，之后 `self.store.insert("attempts", …)`、再拿调用方传进来的 `task` 整行 `self.store.update("tasks", task)`、再 `launch`。去向：并进 task-server 巡检第十二批（`records/2026-10-31-task-server巡检.md` 问题 85）。

## 附录：可重放的审计脚本快照

以下是实际使用的最小复现源码，保存于本记录中，不依赖审计会话的 `/tmp` 文件继续存在。在仓根运行下面的提取命令，会将快照重建到 `/tmp/singlefs-*`，并将审计时的绝对仓路径替换成当前仓根。不会安装服务，也不会修改业务源码。复现成功的含义是观察到缺陷，修复后断言或预期输出应当改变。

```sh
python3 - <<'PY'
from pathlib import Path
import re
report = Path('records/2026-11-01-task-server深入审核.md').read_text()
for name, code in re.findall(r'### 复现文件：`([^`]+)`\n\n```[^\n]*\n(.*?)\n```', report, re.S):
    code = code.replace('/home/fy5090/code/singlefs', str(Path.cwd()))
    Path(name).write_text(code + '\n')
    print(name)
PY
```

运行：

```sh
python3 -B /tmp/singlefs-review-repro.py
python3 -B /tmp/singlefs-task-deep-review.py
```

第一份覆盖 TS-01/02/04，第二份覆盖 TS-03/05/06；脚本分别建临时库，不连接正在运行的 task-server。外部操作桩已直接展示在代码中。

### 复现文件：`/tmp/singlefs-review-repro.py`

```python
import sys, tempfile, types, threading
sys.path.insert(0, '/home/fy5090/code/singlefs/task-server')
import task_server as m
from pathlib import Path
with tempfile.TemporaryDirectory(prefix='singlefs-review-') as root:
    s=m.Scheduler.__new__(m.Scheduler)
    s.environment={}; s.store=m.Store(root+'/db',b'x'*32,{})
    s.configuration=types.SimpleNamespace(attempts_directory=root+'/attempts',logs_directory=root+'/logs',socket=root+'/sock')
    s.peer_root=root; s.finished_listeners=[]
    s.operation=lambda *a,**k:None; s.forget_prediction=lambda *a:None
    s.attempt_shared_roots=lambda t:{}; s.attempt_command=lambda *a:['true']; s.threads_of=lambda t:1
    s.crash_point=lambda *a:None; s.launch=lambda t,a: launched.append(a['id'])
    s.wake=lambda:None
    t=s.store.insert('tasks',dict(state='queued',session='owner',command_key='x',environment={},cwd=root,attempt_count=0,cancel_requested=False))
    paused=threading.Event(); resume=threading.Event(); launched=[]
    original=m.lib.ensure_private_directory
    def gate(path):
        paused.set(); assert resume.wait(5); original(path)
    m.lib.ensure_private_directory=gate
    thread=threading.Thread(target=s.start_attempt,args=(t,m.Placement('start','local',1024,1024)))
    thread.start(); assert paused.wait(5)
    print('cancel_response=',s.cancel(t['id'],'owner',0))
    resume.set(); thread.join(); m.lib.ensure_private_directory=original
    final=s.store.task(t['id'])[0]
    print('after_start=',{k:final.get(k) for k in ['state','cancel_requested','outcome']},'launched=',launched)
    assert final['state']=='running' and not final['cancel_requested'] and launched
    # Simulate restart after child's finished commit, before advance_parent.
    for a in launched: s.store.delete_attempt(a)
    parent=s.store.insert('tasks',dict(state='sharded',session='owner',command_key='p',phase='plan'))
    child=s.store.insert('tasks',dict(state='finished',session='owner',command_key='c',task_id=parent['id'],parent_id=parent['id'],role='plan',outcome='succeeded'))
    s.reconcile()
    print('after_reconcile_parent=',s.store.task(parent['id'])[0]['state'],'children=',len(s.children_of(parent['id'])))
    child=s.store.insert('tasks',dict(state='running',session='owner',command_key='d',task_id=parent['id'],parent_id=parent['id'],role='shard'))
    stops=[]
    def fail_stop(task_id):
        stops.append(task_id);return 500,{'stopped':False,'detail':'simulated systemctl failure'}
    s.stop_running=fail_stop
    print('first parent cancel=',s.cancel(parent['id'],'owner',0))
    print('second parent cancel=',s.cancel(parent['id'],'owner',0))
    print('stop calls=',stops,'child state=',s.store.task(child['id'])[0]['state'])
```

### 复现文件：`/tmp/singlefs-task-deep-review.py`

```python
import sys,tempfile,types,subprocess
sys.path.insert(0,'/home/fy5090/code/singlefs/task-server')
import task_server as m

def scheduler(root):
 s=m.Scheduler.__new__(m.Scheduler);s.environment={};s.store=m.Store(root+'/db',b'x'*32,{})
 s.configuration=types.SimpleNamespace(attempts_directory=root+'/attempts',logs_directory=root+'/logs',socket=root+'/s')
 s.peer_root=root;s.finished_listeners=[];s.operation=lambda *a,**k:None;s.forget_prediction=lambda *a:None
 s.attempt_shared_roots=lambda t:{};s.attempt_command=lambda *a:['true'];s.threads_of=lambda t:1;s.crash_point=lambda *a:None;s.wake=lambda:None
 s.unit_checked_at={};s.unit_check_seconds=0;s.unit_gone_since={};s.lost_grace_seconds=0
 return s

def task(s,root):return s.store.insert('tasks',dict(state='queued',session='owner',command_key='x',environment={},cwd=root,attempt_count=0,cancel_requested=False))
with tempfile.TemporaryDirectory() as d:
 s=scheduler(d);t=task(s,d);update=s.store.update
 class Crash(BaseException):pass
 def crash_update(table,record):
  if table=='tasks':raise Crash()
  update(table,record)
 s.store.update=crash_update
 try:s.start_attempt(t,m.Placement('start','local',1024,1024))
 except Crash:pass
 s.store.update=update
 print('after_crash=',s.store.task(t['id'])[0]['state'],'attempts=',len(s.store.attempts_of(t['id'])))
 m.unit_states=lambda *a:('not-found','inactive')
 calls=[]
 def start(t,a):
  calls.append(a['unit']);return (True,'') if len(calls)==1 else (False,'unit already exists')
 s.start_unit=start
 s.reconcile()
 print('after_reconcile=',s.store.task(t['id'])[0]['state'])
 s.start_attempt(s.store.task(t['id'])[0],m.Placement('start','local',1024,1024))
 print('after_second_start=',s.store.task(t['id'])[0]['outcome'],'attempts=',[(a['number'],a['state']) for a,_ in s.store.attempts_of(t['id'])],'same_unit=',calls[0]==calls[1])
with tempfile.TemporaryDirectory() as d:
 s=scheduler(d);t1=task(s,d);t2=task(s,d)
 a=s.store.insert('attempts',dict(task_id=t1['id'],state='running',session='',command_key='x',unit='other-tasks-unit',number=1))
 s.store.connection().execute('UPDATE attempts SET task_id=? WHERE id=?',(t2['id'],a['id']))
 rows=s.store.attempts_of(t2['id'])
 print('unsigned_index=',{'selected_for':t2['id'],'record_owner':rows[0][0]['task_id'],'signature_valid':rows[0][1],'unit':rows[0][0]['unit']})
with tempfile.TemporaryDirectory() as d:
 from unittest.mock import patch
 s=scheduler(d);t=task(s,d);s.systemd_run='systemd-run';s.children=[]
 # Manager accepted the unit, but the local systemd-run acknowledgement timed out.
 with patch.object(m.subprocess,'run',side_effect=subprocess.TimeoutExpired('systemd-run',m.START_TIMEOUT_SECONDS)), patch.object(m,'unit_states',return_value=('loaded','active')) as states:
  s.start_attempt(t,m.Placement('start','local',1024,1024))
  s.reconcile()
  print('ambiguous_start=',s.store.task(t['id'])[0]['outcome'],'active_unit_readbacks=',states.call_count)
```


### 复现文件：`/tmp/singlefs-task-real-verification.py`

```python
import sys,os,tempfile,types,threading,subprocess,time,pathlib,shutil
sys.path.insert(0,str(pathlib.Path.cwd()/'task-server'))
import task_server as m

def scheduler(d):
 s=m.Scheduler.__new__(m.Scheduler);s.environment=dict(os.environ);s.store=m.Store(d+'/db',b'x'*32,s.environment)
 s.configuration=types.SimpleNamespace(attempts_directory=d+'/attempts',logs_directory=d+'/logs',socket=d+'/s')
 s.peer_root=d;s.finished_listeners=[];s.operation=lambda *a,**k:None;s.forget_prediction=lambda *a:None
 s.attempt_shared_roots=lambda t:{};s.attempt_command=lambda *a:['/bin/sleep','20'];s.threads_of=lambda t:1;s.crash_point=lambda *a:None;s.wake=lambda:None
 s.unit_checked_at={};s.unit_check_seconds=0;s.unit_gone_since={};s.lost_grace_seconds=0;s.systemd_run=shutil.which('systemd-run');s.children=[]
 return s

def task(s,d):return s.store.insert('tasks',dict(state='queued',session='owner',command_key='verification',environment={},cwd=d,attempt_count=0,cancel_requested=False))
def placement():return m.Placement('start','local',1024,1024)
def units(s):return sorted({a['unit'] for a,_ in s.store.select('attempts') if a.get('unit')})
def cleanup(s):
 for unit in units(s):subprocess.run(['systemctl','--user','stop',unit],capture_output=True,timeout=15)
 for unit in units(s):
  state=m.unit_states(unit,s.environment);assert state is not None and state[1] not in m.lib.RUNNING_UNIT_STATES,(unit,state)
 print('cleanup verified:',units(s))

with tempfile.TemporaryDirectory(prefix='ts-real-cancel-') as d:
 s=scheduler(d);t=task(s,d);paused=threading.Event();resume=threading.Event();original=m.lib.ensure_private_directory;errors=[]
 def gate(path):paused.set();assert resume.wait(5);original(path)
 def run():
  try:s.start_attempt(t,placement())
  except BaseException as error:errors.append(error)
 try:
  m.lib.ensure_private_directory=gate;thread=threading.Thread(target=run);thread.start();assert paused.wait(5)
  result=s.cancel(t['id'],'owner',0);resume.set();thread.join(8);assert not thread.is_alive() and not errors,errors
  state=m.unit_states(units(s)[0],s.environment)
  print('TS01_REAL cancelled_response=',result,'actual_unit=',state)
  assert result[1]['stopped'] and state[1]=='active'
 finally:
  resume.set();m.lib.ensure_private_directory=original;cleanup(s)

with tempfile.TemporaryDirectory(prefix='ts-real-crash-') as d:
 s=scheduler(d);t=task(s,d);update=s.store.update
 class Crash(BaseException):pass
 def crash_update(table,record):
  if table=='tasks':raise Crash()
  update(table,record)
 try:
  s.store.update=crash_update
  try:s.start_attempt(t,placement())
  except Crash:pass
  s.store.update=update;s.reconcile();assert s.store.task(t['id'])[0]['state']=='queued'
  s.start_attempt(s.store.task(t['id'])[0],placement());state=m.unit_states(units(s)[0],s.environment)
  print('TS03_REAL outcome=',s.store.task(t['id'])[0]['outcome'],'actual_unit=',state,'attempts=',[(a['number'],a['state']) for a,_ in s.store.attempts_of(t['id'])])
  assert s.store.task(t['id'])[0]['outcome']=='unit-start-failed' and state[1]=='active'
 finally:s.store.update=update;cleanup(s)

with tempfile.TemporaryDirectory(prefix='ts-real-timeout-') as d:
 s=scheduler(d);t=task(s,d);old_timeout=m.START_TIMEOUT_SECONDS;wrapper=pathlib.Path(d)/'delayed-systemd-run'
 wrapper.write_text('#!'+sys.executable+'\nimport subprocess,sys,time,pathlib\np=subprocess.run('+repr([s.systemd_run])+'+sys.argv[1:])\nif p.returncode:sys.exit(p.returncode)\npathlib.Path('+repr(d+'/accepted')+').touch()\ntime.sleep(10)\n')
 wrapper.chmod(0o700);s.systemd_run=str(wrapper)
 try:
  m.START_TIMEOUT_SECONDS=1;s.start_attempt(t,placement());s.reconcile();state=m.unit_states(units(s)[0],s.environment)
  print('TS06_REAL manager_accepted=',pathlib.Path(d+'/accepted').exists(),'outcome=',s.store.task(t['id'])[0]['outcome'],'actual_unit=',state)
  assert pathlib.Path(d+'/accepted').exists() and state[1]=='active' and s.store.task(t['id'])[0]['outcome']=='unit-start-failed'
 finally:m.START_TIMEOUT_SECONDS=old_timeout;cleanup(s)
```

### 复现文件：`/tmp/singlefs-task-extra-verification.py`

```python
import ast,pathlib,types,tempfile,sys,subprocess
# Reuse only the fixture functions, without executing the other tests.
text=pathlib.Path('/tmp/singlefs-task-real-verification.py').read_text()
module=ast.parse(text)
module.body=[node for node in module.body if isinstance(node,(ast.Import,ast.ImportFrom,ast.FunctionDef)) or (isinstance(node,ast.Expr) and isinstance(node.value,ast.Call) and ast.unparse(node.value).startswith('sys.path.insert'))]
exec(compile(module,'fixture','exec'))
from unittest.mock import patch

with tempfile.TemporaryDirectory(prefix='ts-finalize-') as d:
 s=scheduler(d)
 parent=s.store.insert('tasks',dict(state='sharded',session='owner',command_key='parent',phase='plan'))
 child=s.store.insert('tasks',dict(state='running',session='owner',command_key='plan',task_id=parent['id'],parent_id=parent['id'],role='plan'))
 class Crash(BaseException):pass
 def crash(_):raise Crash()
 s.advance_parent=crash
 try:s.finalize(child,'succeeded',0)
 except Crash:pass
 s.advance_parent=types.MethodType(m.Scheduler.advance_parent,s);s.reconcile()
 print('TS02_REAL_COMMIT child=',s.store.task(child['id'])[0]['state'],'parent=',s.store.task(parent['id'])[0]['state'],'children=',len(s.children_of(parent['id'])))
 assert s.store.task(child['id'])[0]['state']=='finished' and s.store.task(parent['id'])[0]['state']=='sharded' and len(s.children_of(parent['id']))==1

with tempfile.TemporaryDirectory(prefix='ts-real-cancel-fail-') as d:
 s=scheduler(d);parent=s.store.insert('tasks',dict(state='sharded',session='owner',command_key='parent'))
 child=s.store.insert('tasks',dict(state='queued',session='owner',command_key='child',environment={},cwd=d,attempt_count=0,cancel_requested=False,parent_id=parent['id'],task_id=parent['id'],role='shard'))
 s.shard_environment=lambda *a:''
 try:
  s.start_attempt(child,placement());calls=[]
  def stop_failure(task_id):calls.append(task_id);return 500,{'stopped':False,'detail':'injected systemctl failure'}
  s.stop_running=stop_failure
  first=s.cancel(parent['id'],'owner',0);second=s.cancel(parent['id'],'owner',0);actual=m.unit_states(units(s)[0],s.environment)
  print('TS04_REAL first=',first[0],'second_stopped=',second[1]['stopped'],'stop_calls=',len(calls),'actual_unit=',actual)
  assert first[0]==500 and second[1]['stopped'] and actual[1]=='active' and len(calls)==1
 finally:cleanup(s)

with tempfile.TemporaryDirectory(prefix='ts-index-') as d:
 s=scheduler(d);first=task(s,d);second=task(s,d)
 attempt=s.store.insert('attempts',dict(task_id=first['id'],state='running',session='',command_key='x',unit='audit-other-owner.service',number=1))
 s.store.connection().execute('UPDATE attempts SET task_id=? WHERE id=?',(second['id'],attempt['id']))
 with patch.object(m.subprocess,'run',return_value=types.SimpleNamespace(returncode=0,stdout='',stderr='')) as run, patch.object(m,'unit_states',return_value=('loaded','inactive')),patch.object(m,'unit_processes',return_value=[]):
  result=s.stop_running(second['id'])
  print('TS05_WRONG_STOP requested_owner=',second['id'],'record_owner=',first['id'],'command=',run.call_args.args[0],'response=',result[0])
  assert run.call_args.args[0][-1]=='audit-other-owner.service' and result[0]==200
```
