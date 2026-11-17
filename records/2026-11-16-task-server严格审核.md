# task-server 严格审核（2026-11-16）

对象：工作区里的 `task-server/`（`task_server.py`、`task_server_lib.py`、`attempt_runner.py`、`task`、`control`、`tests/test_cleanup_protocol.py`、`task_server_selftest.py`、`README.md`、`cleanup-protocol.md`），连同耦合的 `scripts/hosts/multi-host-run.sh` 与本机状态目录 `~/.local/state/singlefs/task-server/` 下的运行日志。行号以工作区现行源码为准。本次只新增这份审核记录，没有改代码或提交。

---

## 一、致命级问题（可致全库死锁、重启误杀在跑任务、漏做显卡回收）

### 1. `Store` 手工 `BEGIN IMMEDIATE` 事务在 `sqlite3.Error` 时未回滚，单次写库异常即可锁死全库
- **位置**：`task-server/task_server.py` 第 1495 行（`Store.connection`）、第 1716–1723 行（`delete_row`）、第 1732–1739 行（`delete_attempt`）、第 1774–1781 行（`health_write`）、第 1813–1824 行（`write_pause`、`clear_pause`），以及第 1583、1608、1640 行
- **现象与触发条件**：
  `Store.connection()` 以 `isolation_level=None` 打开连接，用 `connection.execute("BEGIN IMMEDIATE")` 与 `connection.execute("COMMIT")` 手工管理事务。在 `health_write()`、`delete_row()`、`delete_attempt()`、`write_pause()`、`clear_pause()` 中：
  ```python
  with self.write_lock:
      connection = self.connection()
      connection.execute("BEGIN IMMEDIATE")
      connection.execute("UPDATE meta SET value = ? WHERE name = 'health'", (str(time.time()),))
      connection.execute("COMMIT")
  ```
  当 `UPDATE`、`DELETE` 或 `COMMIT` 遇到瞬时 `SQLITE_BUSY`、磁盘满或 IO 抖动抛出 `sqlite3.Error` 时，外层捕获或向上抛出异常前**没有执行 `ROLLBACK`**，随即释放了 Python 层的 `self.write_lock`。
- **后果**：
  1. 该线程的 SQLite 连接一直留在未结束的 `BEGIN IMMEDIATE` 事务中，后续该线程再次调用 `BEGIN IMMEDIATE` 一律报 `OperationalError: cannot start a transaction within a transaction`。
  2. 因为该连接仍持有 SQLite 文件层的写锁，其他所有线程（含 `scheduler` 线程与 HTTP 工作线程）在 `BEGIN IMMEDIATE` 上等满 30 秒 `busy_timeout` 后全部报 `OperationalError: database is locked`，整个服务彻底卡死。
  3. 第 1583（`insert`）、1608（`update`）、1640（`change`）行的 `connection.execute("ROLLBACK")` 未包 `contextlib.suppress(sqlite3.Error)`，当 SQLite 已自动回滚事务时会抛出二次异常掩盖原始错误。
- **修法**：所有 `BEGIN IMMEDIATE` 块统一包 `try ... except BaseException:`，在 `except` 分支里用 `with contextlib.suppress(sqlite3.Error): connection.execute("ROLLBACK")` 回滚后再抛出或处理。

---

### 2. 服务重启时 `reconcile` 与 `inbox_loop` 早于 `card_lease_loop` 执行首轮，且 `card_take_waited_seconds` 等未从 `state.json` 惰性加载领卡账，导致重启瞬间误杀按需领卡任务并漏做卡回收
- **位置**：`task-server/task_server.py` 第 4367–4392 行（`waiting_for_card_take`、`card_take_waited_seconds`）、第 7914–7920 行（`card_lease_view`）、第 10318–10325 行（`held_cpu_anchor_at`）、第 10387–10389 行（`recent_window_cores`）、第 3856–3860 行（`Scheduler.loop`）、第 7980 行（`card_lease_loop`）、第 12139–12143 行（`serve`）
- **现象与触发条件**：
  1. `serve()` 启动各后台线程时，`scheduler.thread`（`loop` 调 `reconcile`）和 `inbox_thread`（`inbox_loop`）立即执行第一轮，而 `card_lease_thread`（`card_lease_loop`，第 7980 行）先执行 `while not self.stopping.wait(self.card_lease_poll_seconds):`（先等 1 秒）才跑第一次 `card_lease_round()`。
  2. 因此在服务重启后的首轮 `reconcile()` 与 `inbox_loop()` 中，内存表 `self.card_leases` 是空的 `{}`。
  3. `card_lease_state`（第 7906 行）与 `held_card_cores`（第 10299 行）在内存表找不到时会调 `self.card_lease_record(attempt)` 从磁盘 `state.json` 读回，但 `card_take_waited_seconds`（第 4387 行）、`waiting_for_card_take`（第 4372 行）、`card_lease_view`（第 7919 行）、`held_cpu_anchor_at`（第 10321 行）、`recent_window_cores`（第 10388 行）只调 `self.card_leases.get(attempt["id"])`，拿到 `None` 就直接返回 `0.0`、`False` 或 `""`。
- **后果**：
  1. **重启瞬间误判四小时硬上限（`over-four-hours`）杀掉在跑任务**：按需领卡任务常在队里等卡数小时。服务一旦重启，首轮 `check_stall` 顶部的 `judge_run_hard_limit` 调 `run_limit_accumulated` 再调 `card_take_waited_seconds(attempt)` 拿到 `0.0`，等卡时长未扣除，挂钟满 4 小时的任务当场被判 `over-four-hours` 停掉打回且不重排。
  2. **重启瞬间误推两小时提醒（`long-run-unconfirmed`）**：`inbox_loop` 与 `long_run_sessionless_note` 首轮同样拿到 `card_take_waited_seconds == 0.0`，误发长跑未确认提醒。
  3. **重启空档内刚结束的按需领卡尝试漏做卡回收（`reclaim_cards`）**：若尝试在服务重启期间刚结束，`reconcile` 首轮调 `finish_attempt` 再调 `reclaim_cards`（第 6658 行），此时 `card_lease_view(attempt).get("card")` 返回 `""`（调 `card_lease_record` 的 `settle_card_lease` 在第 6738 行之后才跑），导致该次尝试持有的卡完全跳过 Xid 检查、残留计算进程清理与显存核查。
- **修法**：在 `card_take_waited_seconds`、`waiting_for_card_take`、`card_lease_view`、`held_cpu_anchor_at`、`recent_window_cores` 中，当尝试处于在跑态且 `self.card_leases.get(attempt["id"]) is None` 时统一调 `self.card_lease_record(attempt)` 从磁盘恢复；或在启动线程前先同步跑一次领卡账恢复。

---

### 3. 按需领卡任务正常调 `task lease return` 归还的显卡完全绕过卡回收（`reclaim_cards`）
- **位置**：`task-server/task_server.py` 第 6875–6878 行（`reclaim_cards`）、第 7914–7920 行（`card_lease_view`）、第 8091–8115 行（`return_card_lease`）
- **现象与触发条件**：
  - 当按需领卡任务跑完上卡阶段并调 `task-server/task lease return` 还卡时，`return_card_lease`（第 8091–8115 行）直接把卡从 `lease["held"]` 移进 `lease["history"]`，清空 `lease["cards"]`，并立即 `self.wake()` 让下一条任务领走这张卡，还卡时不跑 `reclaim_cards`。
  - 当该次尝试随后退出并进入 `finish_attempt` 调 `reclaim_cards`（第 6875–6876 行）时：
    ```python
    held = attempt.get("card_held")
    cards = [f"{address}:{index}" for address, index in card_keys(held if held else self.card_lease_view(attempt).get("card"))]
    ```
    `card_lease_view(attempt).get("card")` 只返回尝试退出那一刻仍持着的 `lease["cards"]`（此时已是 `""`），不看 `lease["history"]` 中领过并已归还的卡。
- **后果**：所有正常调 `task lease return` 还卡的按需领卡任务都绕过了卡回收：不查内核日志的 GPU Xid 故障、不杀残留的 GPU 子进程、不核显存是否回落到 32 MiB 以下，也不打印 `TASK_SERVER_CARD_GC` 日志。
- **修法**：按需领卡归还卡时（或尝试收尾连同 `lease["history"]` 里领过的卡一起）执行卡回收，回收通过后再放回空闲池。

---

## 二、高危级问题（调度失真、死锁、绕过门禁、状态丢失与并发竞态）

### 1. 按需领卡的等卡时长统计存在三处漏洞（前次尝试未扣除、未获卡即结束时丢失、并发领卡重复累加）
- **位置**：`task-server/task_server.py` 第 4376–4392 行（`ended_card_waited_seconds`、`card_take_waited_seconds`）、第 4701–4713 行（`run_limit_accumulated`）、第 7946–7965 行（`reclaim_card_lease`）、第 8099–8103 行（`return_card_lease`）
- **现象与触发条件**：
  1. **`run_limit_accumulated` 漏扣前几次已结束尝试的等卡时间**：
     `settled_run_seconds`（第 5285 行）对已结束的尝试减去了 `self.ended_card_waited_seconds(attempt)`；但 `run_limit_accumulated`（第 4707 行，用于判两小时提醒和四小时硬上限）只对当前在跑尝试减了 `card_take_waited_seconds(attempt)`，对 `ended` 的前几次尝试仍直接取 `float(record.get("seconds") or 0)`（含等卡挂钟秒数）。若第 1 次尝试等卡 3 小时后因 `peer-run-failed`、`killed-externally` 或 `card-fault` 重排，第 2 次尝试一启动就会背上第 1 次尝试的 3 小时等卡时长，跑满 1 小时即被 `over-four-hours` 误杀。（同理，`completed_runs` 第 9508 行、`jump_refusal` 第 8650 行、`finished_preface` 第 6996 行也未扣等卡时间。）
  2. **尝试在等卡期间结束或撤回时，已等待时间归零丢失**：
     `ended_card_waited_seconds` 只累加 `card_lease_history` 中 `kind == "granted"` 的 `waited_seconds`。若尝试在 `lease["waiting"]` 中等了 2 小时还没拿到卡就结束（或调无参 `task lease return` 撤回等待），`reclaim_card_lease`（第 7955–7957 行）与 `return_card_lease`（第 8100–8102 行）直接将 `lease["waiting"] = []` 清空，**没有往 `lease["history"]` 记录已等待的秒数**，导致这 2 小时等卡时间在收尾后全部算作实际运行时间，触发误报 `long-run-split` 或重试时误判 `over-four-hours`。
  3. **同一次尝试并发多路 `task lease take` 时把重叠的等待时长直接相加**：
     若同一次尝试内多个工作进程同时发起 `task lease take`（`README.md` 第 189 行允许），`card_take_waited_seconds` 与 `ended_card_waited_seconds` 用 `sum(...)` 把各路等待秒数直接相加而非取时间区间并集（`covered_seconds`），导致算出的等卡时长能超过尝试已跑的挂钟总时长；并且只要有一路还在 `waiting`，第 4430 行 `waiting_for_card_take` 就返回 `True`，屏蔽已拿到卡在跑的另一路的卡死检测。

---

### 2. `card_lease_round` 排序键写成了布尔比较 `self.lease_submitted_at(pair[0]) != 0.0`，破坏按任务提交先后分卡并可形成三方死锁
- **位置**：`task-server/task_server.py` 第 7996–7998 行（`card_lease_round`）、第 7854–7865 行（`lease_reserved_cards`）
- **现象与触发条件**：
  第 7996–7998 行写为：
  ```python
  waiting = sorted(((lease, entry) for lease in self.card_leases.values() for entry in lease["waiting"]
                    if not entry.get("quiescing") and now >= float(entry.get("retry_at") or 0)),
                   key=lambda pair: (self.lease_submitted_at(pair[0]) != 0.0, pair[1]["since"], pair[1]["sequence"]))
  ```
  - 对所有未 `promote` 的普通任务，`self.lease_submitted_at(pair[0]) != 0.0` 全都等于布尔值 `True`。结果所有普通任务的领卡请求都按 `pair[1]["since"]`（调 `task lease take` 的时刻）排序，而非按任务提交时刻 `lease_submitted_at` 排序，直接违背文件头第 655 行与 `README.md` 第 59 行的“按各自任务的提交时刻排”。
  - **三方死锁**：当单卡台上有按需领卡任务甲（提交时刻 10，在时刻 100 调 `lease take`）、起跑分卡任务乙（提交时刻 20，排满两小时，`aged = True`）、按需领卡任务丙（提交时刻 30，在时刻 90 调 `lease take`）时：排队线程判任务乙时，`lease_reserved_cards(..., before=20)` 因任务甲（`10 < 20`）在等而把卡留给领卡线程，不给乙；领卡线程跑 `card_lease_round` 时，因按 `since` 排而先看任务丙（`90 < 100`），在 `grant_card_lease` 中调 `queue_claimed_cards(..., before=30)` 时又因任务乙（`20 < 30`）占住该卡而拒给丙；同时 `lease_reserved_cards`（第 7855 行）未按 `lease_submitted_at` 排序，且多卡请求（`wanted > 1` 或 `all`）在第 7860–7864 行每次 `entry` 只预留 1 张卡（`break`），会让多卡请求被后交的单卡任务饿死。

---

### 3. `task promote` 的四处高危缺陷（不唤醒调度线程、起跑分卡任务仍按旧提交时刻让卡、挡住后方队列、不支持分片任务）
- **位置**：`task-server/task_server.py` 第 8271–8303 行（`aged`、`queue_order`、`promote`）、第 8541–8621 行（`place_queued_into`）、第 8666 行（`card_queue_claim`）、第 7461–7469 行（`create_child`）
- **现象与触发条件**：
  1. **`promote()` 不调 `self.wake()`**：`promote()`（第 8279–8303 行）改完库后没有调 `self.wake()`，被提前的任务最多还要干等 `TICK_SECONDS`（30 秒）才进入下一轮调度。
  2. **被 `promote` 的起跑分卡任务（`starts_with_cards`）在分卡时仍用原始 `submitted_at`**：`promote()` 返回说明写“领卡排在最前”，但只在 `lease_submitted_at`（第 7875 行，仅用于按需领卡）里把时间换成了 `0.0`。而在 `place_queued_into`（第 8541 行）和 `card_queue_claim`（第 8666 行）中，仍写 `submitted = float(task.get("submitted_at") or now)`，导致被 `promote` 的起跑分卡任务仍把空卡让给所有比它原始提交时刻早的按需领卡任务与排满两小时的任务。
  3. **`aged()` 与 `place_queued_into` 违背“不挡后面”并破坏 `aged` 规则 ①**：
     - `aged()`（第 8272 行）注释写“task promote 提前过的不在这里：它排最前、不过 CPU 闸，内存放得下就起，不挡后面”，但第 8273 行未排除 `task.get("promoted")`，一旦该任务距提交满两小时，`aged()` 返回 `True` 照样挡住全队。
     - 即便距提交未满两小时，被 `promote` 的任务排在 `position = 0`；若它因等卡或等内存暂时起不来，第 8569 或 8609 行将它加入 `earlier_blocked`。随后队里所有任务（含已排满两小时的 `aged` 任务）都被当作“插队”送进 `jump_refusal`（第 8616 行），没有上次运行时长或时长超限的一律被拒。更严重的是，当 `position = 1` 的 `aged` 任务在第 8617 行被 `jump_refusal` 拦下时，代码直接 `continue` 而未调 `behind(...)`，放任 `position >= 2` 的短任务越过 `aged` 任务先起，违反排队规则 ①。
  4. **分片任务无法 `promote`**：分片任务确认后父任务状态立即变为 `"sharded"`（第 11185 行），调 `task promote <父任务号>` 会因 `state != "queued"` 报 409 拒绝（第 8296 行）；即便直接 `promote` 其 `plan` 子任务，`create_child`（第 7461 行）创建 `shard` 和 `merge` 子任务时也不继承 `promoted`。

---

### 4. 第二台（`PeerMachine`）未实现 `card_busy` 与 `card_utilizations`，导致第二台上的 GPU 任务卡死检测与疑似卡死通知完全失效
- **位置**：`task-server/task_server.py` 第 2083–2098 行（`Machine.card_busy`、`Machine.card_utilizations`）、第 2358–2378 行（`PeerMachine`）、第 4415–4525 行（`check_stall`、`judge_stall_stop`）
- **现象与触发条件**：
  `multi-host-run.sh` 每轮已在日志中打出 `MULTI_HOST_RUN_PEER_GPU ... utilization=<pct> on_cards=<...>`，且 `log_content`（第 6466–6472 行）已解析这两项。但 `PeerMachine` 没有重写基类 `Machine.card_busy`（恒返回 `True`）和 `Machine.card_utilizations`（恒返回 `None`）。
- **后果**：
  派到第二台的所有 GPU 任务（或持卡阶段的按需领卡任务）：
  - `self.card_load(attempt)` 永远为 `True`（哪怕 `on_cards=empty` 卡上根本没有进程）；
  - `note_card_load` 的 `utilization_unread` 永远为 `True`，`card_idle`（第 4495 行）永远为 `False`；
  - 一旦在第二台上发生死锁（日志不涨、CPU 为 0、GPU 利用率 `utilization=0` 甚至卡上无进程），**永远不会被判 `stalled` 或 `suspected-stall`**，一直白占显卡直到四小时硬上限。

---

### 5. 按需领卡任务在静默窗口中途还卡后，`self.card_loads` 残留旧状态，让回到 CPU 阶段死锁的任务被误当成“GPU 在算”免停
- **位置**：`task-server/task_server.py` 第 4412–4413、4467–4468、4491–4525 行
- **现象与触发条件**：
  按需领卡尝试在静默窗口前半段持卡运行，`note_card_load` 在 `self.card_loads[attempt["id"]]` 中记下了 `window_started_at == seen[2], every_poll_busy = True, utilization_busy = True`。随后尝试在窗口中途调 `task lease return` 还卡，回到 CPU 阶段并发生死锁（日志不涨、CPU 不涨）。
  在后续轮询中，因 `attempt["card"]` 变成了 `""`，第 4467 行 `if task.get("gpu") and attempt.get("card"):` 为假，既不更新也不清除 `self.card_loads[attempt["id"]]`。当满 10 分钟判 `judge_stall_stop` 时，第 4491 行取出的 `loaded` 仍是还卡前的记录（`every_poll_busy == True`），触发第 4516 行的 `GPU 在算，不停，再看一个窗口`。

---

### 6. `reclaim_cards` 检出 GPU Xid 故障并停用显卡时，未将失败尝试的结局改为 `card-fault` 重排
- **位置**：`task-server/task_server.py` 第 6613、6658、6901–6910 行（`reclaim_cards`）、第 7006–7066 行（`settle_attempt`）
- **现象与触发条件**：
  文件头第 142–143 行与 `reclaim_cards`（第 6905 行）告警均写明：检出 Xid 时停用坏卡 60 分钟，“写一条 `card-fault` 告警，这条任务重排时换别的卡”。
  但当 GPU 进程因 Xid 崩溃退非 0（如退 1 或 101）时，`finish_attempt` 在第 6613 行（调 `reclaim_cards` 之前）已将 `outcome` 判为 `"command-failed"`；第 6658 行 `reclaim_cards` 检出 Xid 后只设置了 `attempt["card_gc_quarantined"]`，并未把 `outcome` 改成 `"card-fault"` 或设置 `attempt["faulty_cards"]`。随后 `settle_attempt` 按 `"command-failed"` 交回失败结局，**不会换卡重排**。

---

### 7. 尝试结束、被取消或调 `task lease return` 时若正处于 `quiesce_for_card_lease` 清场中，卡立即被放回空闲池而 `gpu-quiesce-run.sh` 仍在跑
- **位置**：`task-server/task_server.py` 第 7832–7844 行（`leased_card_keys`）、第 7946–7965 行（`reclaim_card_lease`）、第 8099–8105 行（`return_card_lease`）、第 8178–8240 行（`quiesce_for_card_lease`）
- **现象与触发条件**：
  `grant_card_lease` 挑中卡后在后台起 `quiesce_for_card_lease` 线程跑 `gpu-quiesce-run.sh`。若在此期间尝试结束、被取消（`reclaim_card_lease` 删掉 `self.card_leases[attempt_id]`）或调了无参 `task lease return`（清空 `lease["waiting"]`），后台的 `gpu-quiesce-run.sh` 进程并未被终止，但 `leased_card_keys` 与 `busy_cards()` 立即认为该卡已空闲并分给下一个任务，造成两个清场或测试进程在同一张卡上相撞。

---

### 8. 第二台盘余量低于 50 GiB（`DISK_FLOOR_KIND`）时，预算超过本机上限的任务被当场误判为 `never-fits` 永久打回
- **位置**：`task-server/task_server.py` 第 9186–9207 行（`hosts_short_of_record`）、第 9958–10012 行（`judge_placement`）、第 9886–9899 行（`place`）
- **现象与触发条件**：
  当第二台在线（`snapshot.reachable[PEER] == True`），但盘余量低于 50 GiB 时，`PEER` 不在 `snapshot.reports` 中，且 `snapshot.down_kind[PEER] == DISK_FLOOR_KIND`。
  在 `judge_placement` 中：
  1. 第 9960 行 `if snapshot.down_kind.get(host) != DISK_FLOOR_KIND:` 让 `PEER` 既不进 `verdicts` 也不进 `unusable`（`unusable` 为空 `{}`）；
  2. 第 9975 行 `hosts_short_of_record` 因 `limits[PEER] is None` 将 `elsewhere` 设为非空（`["第二台这一轮给不出可用上限"]`），若任务的记录峰值大于本机上限 `limits[LOCAL]`，就把 `verdicts[LOCAL]` 标为 `never=True`；
  3. 第 10010 行 `if all(verdict.never for verdict in verdicts.values()) and not unusable:` 成立，且 `place()`（第 9893 行）检查 `not snapshot.reachable.get(host)` 为假，直接返回 `Placement("never")`，把任务以 `never-fits` 永久打回。

---

### 9. `check_stall` 在调 `log_content(attempt)` 之前读取第二台的 CPU 与线程数，且同轮多次调 `log_content` 会覆盖清掉前一次的 `peer_io_wait` 与 `peer_threads`
- **位置**：`task-server/task_server.py` 第 4403–4426 行（`check_stall`）、第 6410–6490 行（`log_content`）、第 6106 行（`audit_card_bookings`）
- **现象与触发条件**：
  1. **先读旧缓存后解析日志**：`PeerMachine.cpu_usec` 和 `PeerMachine.thread_reading` 依赖 `log_content(attempt)` 增量扫描日志更新缓存。但 `check_stall` 在第 4403–4408 行先调 `cpu_usec`、`note_peak_segment`、`held_card_cores`、`note_threads`，到第 4414、4415、4425 行才调 `log_content(attempt)`。这导致每次用的都是上一轮（甚至持卡前 CPU 阶段）的旧读数配当前时间戳 `time.time()`；且在 10 分钟宽限期内（第 4417 行提前返回），若不要卡的第二台尝试已进 `wrong_cards`，整段宽限期内 `log_content` 一次都不会被调用。
  2. **多次调用互相覆盖**：`log_content(attempt)`（第 6483–6489 行）每次只用本次新增字节 `[offset, size)` 内解析到的行覆盖 `self.peer_io_wait[attempt["id"]]` 和 `self.peer_threads[attempt["id"]]`，且无锁。`check_stall` 一轮内最多调 3 次 `log_content`（第 4414、4415、4425 行），加上后台 `booking_audit_thread`（第 6106 行）并发调用，后一次调用会把前一次读到的 `blocked=0`（非 IO 等待）或更高的 `runnable` 线程数覆盖丢弃，还会重复累加 `add_card_usage_round`。

---

### 10. 命令解析与提交拒绝（`submit_refusal`）的三处绕过与误拒漏洞
- **位置**：
  - `task-server/task_server_lib.py` 第 5238–5267 行（`shell_inner_commands`）与 `task-server/task_server.py` 第 11103–11105 行（`inner_nested`）
  - `task-server/task_server_lib.py` 第 3461–3504 行（`command_word_index`）
  - `task-server/task_server_lib.py` 第 2154–2158、5485–5495 行（`whole_package_cargo_test`）
- **现象与触发条件**：
  1. **`lexer.whitespace_split = True` 废掉了 `shlex(..., punctuation_chars=True)`**：
     `shell_inner_commands` 构造 `shlex(words[index + 1], posix=True, punctuation_chars=True)` 后紧接着设了 `lexer.whitespace_split = True`，让 `shlex` 退化为纯空白切词：
     - `bash -c "true;cargo test -p singlefs-verification"` 或 `bash -c "(cargo test ...)"` 的首词变成 `"true;cargo"` 或 `"(cargo"`，绕过整包测试检查、全仓编译拒绝与内嵌包装检查；
     - `bash -c "cargo test -p singlefs-verification 2>&1"` 中的 `"2>&1"` 不会被拆成 `['2', '>&', '1']`，而是作为位置参数（测试名过滤词）放进 `names`，**直接绕过 `whole-package-unconfirmed` 整包测试拒绝**，或在带 `--exact` 时把 `"2>&1"` 当成第二个测试名引发误拒。
  2. **`command_word_index` 算错包装参数偏移**：
     `env -C <dir> <cmd>`、`nice -n10 <cmd>`、`nice --adjustment 10 <cmd>`、`timeout -k 5 60 <cmd>`、`python3 -W ignore <script>` 等写法会让 `command_word_index` 停在选项值上，从而绕过 `submit_refusal` 并漏判 `declared_compiles`。
  3. **`CARGO_VALUE_OPTIONS` 漏掉 `--exclude`，且连写 `-p<pkg>` 命中死代码**：
     - `CARGO_VALUE_OPTIONS`（第 2154 行）漏了 `"--exclude"`，`cargo test -p pkg --exclude other` 会把 `other` 当作测试过滤词，直接绕过 `whole-package-unconfirmed`；
     - 第 5487 行 `name = word.split("=", 1)[0] if word.startswith("--") else word` 让 `-psinglefs-verification` 的 `name` 保留全串，导致第 5490 行 `name in ("-p", ...) and len(word) > 2` 成为永远不成立的死代码，不仅绕过 `whole_package_cargo_test`，还会在 `whole_workspace_cargo_refusal`（第 8848 行）中被误拒为“没带 `-p`”。

---

### 11. `attempt_runner.py`：写 `started` 标记后未捕获 `subprocess.Popen` 异常导致误报 `launcher-lost`（238），且短于 1 秒的任务漏采退出前 CPU 与线程数
- **位置**：`task-server/attempt_runner.py` 第 381–434 行
- **现象与触发条件**：
  1. **误报 `launcher-lost`**：第 398 行先写出 `started` 文件，第 401 行 `subprocess.Popen` 却在 `try...except` 块之外。若命令不存在（`FileNotFoundError`）、无执行权限（`PermissionError`）或格式错误（`OSError`），`attempt_runner.py` 未写 `result.json` 就崩溃退出；`task_server.py` 见到有 `started` 而无 `result.json`，将其误判为 `launcher-lost`（退 238，“跑中整个 scope 被外头掐了”）。此外信号处理器在 `Popen` 之后才注册（第 407 行），存在早期 `SIGTERM` 竞态窗口。
  2. **短任务 CPU 核数与线程数恒为初始值**：第 419 行循环在 `Popen` 后 `t ≈ 0` 立即跑第一次 `sampler.sample()`，此时 `sampler.memory_peak` 被赋上初始几 MiB（非 `None`），`usage` 接近 0，`threads` 为 1；若任务在 `time.sleep(1.0)` 期间（短于 1 秒）跑满多核并退出，循环结束后的 `if sampler.memory_peak is None:`（第 425 行）为 `False`，**不再调用 `sampler.sample()`**，导致短任务的 `cpu_usec` 和 `threads` 永远停留在 `t ≈ 0` 的初始值。

---

### 12. 停服务、跨机通信、分片推进与测试套件的高危缺陷
1. **`Scheduler.stop()` 漏 `join` `self.peer_probe_thread`（`task-server/task_server.py` 第 3880 行对比第 12145 行）**：停服务时 `peer_probe_thread` 未被 join 就执行 `stop_own_children()`，探测线程随后起的 `ssh` 子进程留在 `singlefs-task-server.service`（`KillMode=process`）的 cgroup 中，导致 `control stop` 核出残留进程而退 1 失败。
2. **主线程与收尾线程在第二台断连时直接调 `refresh_peer()` 或 `peer.run()` 阻塞（`task-server/task_server.py` 第 2385、2400、2743、5884 行）**：`recheck`、`attempt_presence_readings`、`card_gc_readings`、`process_listings` 以及 `PeerMachine.remove_build_directory`（第 2455 行，本机 `alerts.jsonl` 已记录因为第二台 SSH 瞬断导致 `crash-point-replay-shard-configuration-check.sh` 失败而报 `build-directory-left`）未检查 `resources.peer_failed` 冷却期，在第二台断连时直接阻塞或失败。
3. **`drive_parent` 与 `write_text_atomically` 跨线程无锁竞态（`task-server/task_server.py` 第 7509–7693、1269–1275 行）**：最后两片同时在后台收尾线程完成（或与 `reconcile` 并发）时，两个线程同时进入 `drive_parent_phase` 调 `write_outputs`，共用固定临时文件名 `outputs.tsv.partial`（`write_text_atomically` 也只带 `os.getpid()` 不带线程号），后执行 `os.replace` 的线程报 `FileNotFoundError` 并触发 `shard-advance-failed` 告警。
4. **`Store.update` 不校验原行 HMAC 且回写已结束尝试整行（`task-server/task_server.py` 第 1587–1609、10697 行）**：`start_attempt` 在换台删除上一次尝试留下的编译目录后调 `self.store.update("attempts", kept)`（且未拿 `build_removal_lock`），用旧快照覆盖已结束尝试整行，会抹掉并发的 `card_held` 释放更新。
5. **`runaway_targets` 查 `ORDER BY id DESC LIMIT 20` 未排除分片子任务（`task-server/task_server.py` 第 5982–5992 行）**：分片子任务行没有 `"inbox"` 字段；若会话最近交了一个不少于 20 片的分片任务，前 20 行全是子任务，`runaway_targets` 返回 `[]`，导致空转进程一次都不通知发起会话就被直接杀掉。
6. **`task` 客户端 `write_lease_request` 与服务端 `read_card_lease_requests` 竞态（`task-server/task` 第 628–647 行、`task-server/task_server.py` 第 8040–8058 行）**：尝试收尾删目录时 `write_lease_request` 抛未捕获的 `FileNotFoundError`（退 1 而非退 3）；且若 `request-1.json` 在 NFS 上瞬时读失败而 `request-2.json` 读成功，服务端将 `last_sequence` 推进到 `2`，永久跳过 `request-1.json`。
7. **`test_cleanup_protocol.py` 漏提取 `multi_host_remove_peer_temporary` 且漏设 `peer_started=1`（`task-server/tests/test_cleanup_protocol.py` 第 214–282 行）**：导致测试子 shell 中 `multi_host_remove_peer_run` 每次都因 `multi_host_remove_peer_temporary: command not found`（退出码 127）返回失败，`test_wrapper_cleanup_retries_remove_after_holder_exit` 本应验证重试删除成功（`confirmed`），却把报错后的 `status == "pending"` 当成预期结果写进了断言。

---

## 三、中危级问题（功能失效、二次确认摩擦、资源泄漏与多线程字典竞态）

1. **`task clean` 无法清理第二台保留的编译目录（`task-server/task_server.py` 第 11428 行）**：
   `Scheduler.clean()` 直接检查本机 `os.path.lexists(directory)` 并调 `shutil.rmtree(directory)`，未走 `self.resources.machine(host).remove_build_directory(directory)`，导致第二台保留的编译目录用 `task clean` 永远删不掉（恒报 `删了 0 份`）。
2. **批量提交任务时二次确认 `confirm --items <N> --tasks <M>` 产生连锁拒单（`operations.jsonl` 任务 18076–18084 实测）**：
   `awaiting-confirmation` 的任务不计入 `M`（在排加在跑数）。当调用方连交 9 条任务时，9 张确认单都打出 `tasks=205`；一旦确认第 1 条（18076），它进队让 `M` 变为 `206`，后 8 条照确认单写的 `--tasks 205` 确认全部被拒（`confirm-refused`），逼迫调用方逐条递增重算 `206` 到 `213` 或反复失败重试。
3. **分片任务失败或取消后永久泄漏 `output-<index>-attempt-<number>` 目录（`task-server/task_server.py` 第 7496–7508、7197–7289、11393–11454 行）**：
   `finalize_parent` 仅在 `outcome == "succeeded"` 时删除各片的 `output-*` 目录；失败、取消或跑 `task clean` 均不删除 `output-*`，且 `finalize_parent` 删 `test-binaries` 时未拿 `self.build_removal_lock`。
4. **多处跨线程共享容器无锁读写，可抛 `RuntimeError: dictionary/Set changed size during iteration`**：
   - `self.quarantined_cards` 与 `self.health_notified`（`task-server/task_server.py` 第 3571–3582、3690、3957、6285 行）；
   - `forget_settled_entries` 遍历并删除 `reminded_thresholds`、`reminder_histories`、`inbox_failed` 时未拿 `attention_lock` 与 `inbox_lock`（第 4292–4341 行）；
   - `booking_attempts` 无锁原地修改 `self.unit_stopped.intersection_update(...)`（第 10173 行）。
5. **暂停排队（`pause --allow-session`）期间破坏全局 `queue_positions` 并每轮清空其他会话的预测缓存（`task-server/task_server.py` 第 8493–8509 行）**：
   `place_queued_into` 将 `queued` 过滤为仅放行会话的任务后，直接用过滤后的列表覆写 `self.queue_positions` 并调用 `keep_predictions_of`，导致其他会话排着的任务队位丢失、预测缓存被清空。
6. **`shard_cap_fitted_to_room` 会把分片任务的 `memory_cap` 压到 1 GiB 下限（`MEMORY_FLOOR_BYTES`）以下且非 MiB 对齐（`task-server/task_server.py` 第 9382–9393 行）**。
7. **`pin_cap`（`POST /pins`）在 `SLICE_MEMORY_CAP_BY_HOST` 缺失或写坏时未拒单，反而返回 HTTP 200 并写入空钉值（`task-server/task_server.py` 第 9771–9785 行）**。
8. **`progress_smallest_fitting`（`/health` 的 10 分钟无进展检查）未考虑 `gpu_waiting_hosts`、按需领卡预留（`lease_reserved_cards`、`queue_claimed_cards`）及单卡显存额度，导致正常让卡等待满 10 分钟时 `/health` 误报 `no-progress` 不健康（`task-server/task_server.py` 第 8424–8464 行）**。
9. **`LocalMachine.card_busy` 与 `card_processes` 只认 `runner` 的 PPID 子孙进程，漏掉脱离父进程但仍在尝试 cgroup 或 scope 内的 GPU 进程（`task-server/task_server.py` 第 2221、2272 行）**。
10. **`Scheduler.log_content()` 的 `noise_marks` 漏掉 `MULTI_HOST_RUN_PEER_MEMORY_PEAK`、`MULTI_HOST_RUN_PEER_BUILD_PEAK`、`MULTI_HOST_RUN_PEER_BUILD_KEPT` 与 `TASK_SERVER_CARD_GC`（`task-server/task_server.py` 第 6433 行）**。
11. **`control` 的 `clear-queue`、`drop-history`、`records`、`key-stats` 在库文件尚不存在时不走 `fail()` 而直接抛未捕获异常，且 `drop-history` 同秒内备份重名覆盖（`task-server/control` 第 476–680 行）**。
12. **`completed_runs`（用于 `jump_refusal` 的上次运行时长）与 `threads_by_record` 对分片子任务未检查 `record_fits_task`（片数相同），导致不同片数的记录混用（`task-server/task_server.py` 第 9501、10150 行）**。
13. **`acknowledge_anomalies`（`confirm --why`）在服务重启后只查父任务自己的 `stored_reminded_thresholds(task_id)`，漏查分片子任务的 `oversized-budget` 记录，导致重启后无法确认分片子任务的预算提醒（`task-server/task_server.py` 第 5460–5479 行）**。

---

## 四、文档、注释与实现不一致

1. **CPU 闸的三处文档与文件头说明同现行代码相反**：
   - `task-server/README.md` 第 29 行与 `task-server/task_server.py` 第 533–555 行写：在跑尝试按“到此刻各段取最大的峰值段核数”记、刚重启未量满一段的按历史记录记、无历史记录的新命令按 `1` 核记；
   - `task-server/task_server.py` 第 10163–10259 行实际默认按**最近一段**（`latest_running_cores`）记、重启后未量满一段按累计平均记、无记录按登记线程数（31 核）记（旧行为分别在弄坏开关 `cpu-running-by-peak`、`cpu-restart-unmeasured-by-record`、`cpu-unrecorded-by-one-core` 下），且 `latest_running_cores` 还让弄坏开关 `cpu-running-cores-over-registered` 与 `cpu-cores-by-average` 失效。
2. **在跑内存记账说明与 `BOOKED_WHILE_YOUNG` 实现不符**：
   - `task-server/README.md` 第 49 行写“在跑的内存记账只看最近 2 分钟里服务读到的用量：编译冲高之后回落，两分钟后账就跟着降”，文件头第 469 行写有 scope 即按 `min(上限, 实测)` 记；
   - `task-server/task_server.py` 第 9221–9227 行实际实现的是起跑前 2 分钟内一律按整份上限 `cap` 记（`BOOKED_WHILE_YOUNG = "young"`），满 2 分钟后按当轮瞬时实测记（无 2 分钟滑动窗口），且 `audit_memory_bookings`（第 6070 行）在算漂移总和时把 `young` 换成了 `used` 却未改明细行的 `booked`，导致告警正文的总和与明细加总不一致。
3. **`task-server/README.md` 第 194 行前后矛盾**：前半段写“脚本永远不报自己要多少内存，服务也不认这样的行”，同一段后半却仍写 `TASK_SERVER_MEMORY_NEEDED bytes=<字节数>`；第 27、60 行仍写乘 `1.5` 的旧公式。
4. **`task-server/cleanup-protocol.md` 与现行代码脱节**：第 16–34 行仍写不存在的 `MULTI_HOST_RUN_CLEANUP` 日志标记、漏列 `peer-cleanup.json` 的 `scope_unit` 与 `scope_leftovers` 字段，且未反映 `card_held` 先结案尝试再后台回收卡的逻辑。
5. **CLI 帮助与文档漏列子命令**：`task promote <号>` 与 `task lease take [<张数>|all]` 未写入 `task` 文件头、`usage()` 及 `README.md`；`control` 的 `usage` 漏列 `records`、`drop-history`。
