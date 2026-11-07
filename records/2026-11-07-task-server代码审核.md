# 2026-11-07 task-server 代码审核

审核范围覆盖 `task-server/task_server.py`（8144 行）、`task-server/task_server_lib.py`（4124 行）、`task-server/attempt_runner.py`（449 行）、`task-server/control`（406 行）、`task-server/task`（541 行）及 `task-server/tests/test_cleanup_protocol.py`（308 行）。早前报告（`records/2026-11-01-task-server深入审核.md`、`records/2026-11-03-task-server与prune-server深入对攻审核.md`、`records/2026-11-06-task-server代码审核.md`）所列缺陷均已核对现行代码确认落定，以下仅列本次新查实的五处代码漏洞。

---

## 汇总

| 编号 | 级别 | 位置 | 问题概要 |
|---|---|---|---|
| 缺陷一 | 高危 | `task-server/task_server.py:7576-7660, 4694-4749` | `cancel_sharded` 与 `cancel_estimating` 停完子任务单元后未等对账收尾就收尾父任务并删产物，`cancelled_attempt_artifacts` 整行写回旧尝试字典把已 `ended` 的尝试覆盖回 `running`，永久泄漏显卡、内存、占盘与机器占用 |
| 缺陷二 | 中危 | `task-server/task_server.py:1049-1064, 4625-4650, 4843-4844, 7388-7446` | 非排队线程调 `Store.update("attempts", ...)` 整行覆盖尝试记录，且 `clean` 未持 `build_removal_lock` 与 `finalize` 并发删编译目录与写库 |
| 缺陷三 | 中危 | `task-server/task_server.py:1819-1835, 1893-1897, 6620-6644` | `Resources.recheck` 漏调 `disk_problem`、`launch_recheck` 漏调 `peer_view`，导致起之前复核与同轮后续派活跳过 50 GiB 盘余量下限、第二台编译目录占盘检查与坏卡隔离过滤；`PassView.refresh` 未同步更新同轮另一张快照缓存 |
| 缺陷四 | 中危 | `task-server/task_server.py:3194, 3210, 3274, 4403-4427, 7661-7703` | `judge_stall_stop` 与 `judge_run_limit` 后台线程按 `task_id` 调 `stop_running`，在尝试撞上限自动重排后可能误杀新起的下一次尝试，且与对账线程并发执行 `--recover-cleanup` |
| 缺陷五 | 低危 | `task-server/task_server.py:1735-1736, 1752-1755, 1784-1793, 2896-2905` | `Scheduler.quarantined_cards`、`Resources.cached` 与 `Resources.peer` 在排队线程、NFS 重查线程与 HTTP 线程池之间无锁并发读写与遍历删除，存在字典迭代期修改抛 `RuntimeError` 的竞态 |

---

## 缺陷一（高危）：分片与估算任务取消时未等子任务对账收尾即删产物并写回旧尝试字典，导致尝试记录永久卡在 `running` 并泄漏资源

### 位置

- `task-server/task_server.py:7576-7600`（`Scheduler.cancel_estimating`）
- `task-server/task_server.py:7602-7635`（`Scheduler.cancel_sharded`）
- `task-server/task_server.py:7637-7659`（`Scheduler.cancel_children`）
- `task-server/task_server.py:4680-4749`（`Scheduler.remove_cancelled_artifacts`、`Scheduler.cancelled_attempt_artifacts`）
- `task-server/task_server.py:1049-1064`（`Store.update`）

### 根因与触发路径

对于不分片的普通在跑任务，`cancel`（`task-server/task_server.py:7530-7535`）在调用 `self.stop_running(task_id)` 停掉 systemd 单元后，会调用 `self.cancel_outcome_reply(task_id, reply)`（`task-server/task_server.py:7539-7561`），轮询等待排队线程的 `reconcile` 调用 `finish_attempt` 与 `settle_attempt` 完成外层 `.scope` 清理、写入尝试结局 `state = "ended"` 并将任务状态改为 `state = "finished"`，随后才调用 `self.with_cancelled_artifacts(task_id, before, reply)` 清理产物。

但在取消分片父任务（`cancel_sharded`）和取消估算父任务（`cancel_estimating`）时，`cancel_children`（`task-server/task_server.py:7644-7659`）对每个在跑子任务只做两件事：
1. 在 `task_lock(child["id"])` 下把子任务记上 `cancel_requested = True, cancel_origin = "parent"`（此时子任务的 `state` 仍是 `"running"`）；
2. 调用 `self.stop_running(child["id"])` 停掉 systemd 单元（并在 `task-server/task_server.py:7699` 调 `self.wake()` 唤醒排队线程），**完全没有等待排队线程对子任务执行 `finish_attempt` 与 `settle_attempt`**。

紧接着，`cancel_sharded`（`task-server/task_server.py:7629-7635`）与 `cancel_estimating`（`task-server/task_server.py:7599-7600`）立即执行 `self.finalize_parent(...)` 和 `self.with_cancelled_artifacts(parent_id, before, ...)`，由此引发两条连锁漏洞：

1. **旧尝试字典整行写回覆盖已收尾的尝试，导致尝试永久卡在 `state = "running"`**：
   - HTTP 取消线程进入 `remove_cancelled_artifacts`（`task-server/task_server.py:4696`），从库里读出子任务的尝试记录 `attempt`；因对账线程尚未跑完 `finish_attempt`，此时 HTTP 线程手里的 `attempt["state"]` 为 `"running"`。
   - 与此同时，被 `stop_running` 唤醒的排队线程执行 `reconcile` 并进入 `finish_attempt`（`task-server/task_server.py:4289-4316`），将该尝试更新为 `state = "ended", outcome = "cancelled", ended_at = ...` 并通过 `self.store.update("attempts", attempt)` 写回 SQLite，再经 `settle_attempt` 调用 `finalize` 将子任务更新为 `state = "finished"`（随后 `finalize` 里的 `remove_finished_build_directories` 在 `task-server/task_server.py:4625` 等待 HTTP 线程持有的 `self.build_removal_lock`）。
   - HTTP 取消线程在 `cancelled_attempt_artifacts`（`task-server/task_server.py:4737`）删完该尝试的编译目录后，执行 `task-server/task_server.py:4748-4749`：
     ```python
     attempt["build_directory_removed"] = True
     self.store.update("attempts", attempt)
     ```
     由于 `Store.update`（`task-server/task_server.py:1049-1064`）只检查 `table == "tasks"` 时不从 `finished` 退回，对 `attempts` 表是无条件整行 JSON 覆盖，HTTP 线程手里那份 `state == "running"` 的旧字典直接把库里已经 `state == "ended"` 的尝试记录覆盖回 `state = "running"`，并抹掉 `outcome`、`ended_at`、`memory_peak` 等收尾字段。
   - 由于子任务在 `tasks` 表里已经是 `state == "finished"`，后续每一轮 `reconcile` 在 `task-server/task_server.py:3015`（`if task.get("state") == "finished": continue`）都会直接跳过该子任务，永远不会再重查这条尝试（即使重启 `task-server` 也不会恢复）。
   - 于是这条尝试在 `attempts` 表中永久停留在 `state = "running"`。所有按 `SELECT ... FROM attempts WHERE state IN ('starting', 'running')` 统计在跑占用的逻辑都会永久把它的资源算作占用：
     - `busy_cards`（`task-server/task_server.py:5155-5158`）：该尝试分到的显卡永久被视为忙碌，无法再分配给任何后续任务；
     - `room_by_the_formula`（`task-server/task_server.py:5909`）与 `server_pending`（`task-server/task_server.py:6482`）：该尝试的 `memory_cap` 与 `gpu_booked` 永久从所在机器的可用内存与显存中扣除；
     - `server_disk_pending`（`task-server/task_server.py:6198`）：该尝试的 `disk_booked` 永久计入所在机器的编译目录待涨占盘；
     - `Alternation.__init__`（`task-server/task_server.py:1916`）、`blocking_seconds`（`task-server/task_server.py:2086`）与 `progress_smallest_fitting`（`task-server/task_server.py:5296`）：所在机器永久被视为有在跑尝试占着。

2. **父任务早于子任务收尾，且本机外层内存包装 `.scope` 未经清理即回报 `stopped: True`**：
   - `drive_parent_phase`（`task-server/task_server.py:5031-5033`）与 `drive_item_estimate`（`task-server/task_server.py:4997-5000`）中明确规定 `phase == "cancelling"` 时必须等 `all(child.get("state") == "finished" for child in children)` 才调用 `finalize_parent`。而 `cancel_sharded` 与 `cancel_estimating` 在子任务仍处于 `state = "running"` 时就直接调用了 `finalize_parent`。
   - 对派在本机（`local`）的尝试，`run-with-memory-cap.sh` 会把实际负载放进 `singlefs-heavy.slice` 下的独立外层 `.scope` 单元运行。`stop_running`（`task-server/task_server.py:7686-7694`）只停止并回读内层 `.service` 单元，不检查也不杀外层 `.scope`；外层 `.scope` 的回读与 `SIGKILL` 清理是在 `finish_attempt` 调用 `stop_leftovers`（`task-server/task_server.py:4456-4475`）时完成的。`cancel_sharded` 与 `cancel_estimating` 不等子任务走完 `finish_attempt` 就向调用方返回 `200 stopped: True`，导致返回时子任务的外层 `.scope` 可能仍在运行甚至随后进入 `cleanup_pending`。

### 修复建议

1. 在 `cancel_children` 中，对每个调用 `self.stop_running(child["id"])` 成功的在跑子任务，继续等待其对账收尾完成（复用或调用与 `cancel_outcome_reply` 相同的等待逻辑，确认子任务在库中达到 `state == "finished"`，若有子任务超时未收尾或记了 `record-tampered` 则记入 `problems`），全部子任务都进入 `finished` 后再由 `cancel_sharded` / `cancel_estimating` 调用 `finalize_parent` 与 `with_cancelled_artifacts`。
2. 将 `cancelled_attempt_artifacts`、`remove_finished_build_directories_locked` 与 `clean` 中更新 `build_directory_removed` 的 `self.store.update("attempts", attempt)` 改为 `self.store.change("attempts", attempt["id"], {"build_directory_removed": True})`，并在 `Store.update` 中与 `tasks` 表一样增加对 `attempts` 表的状态保护（库里已是 `state == "ended"` 的尝试禁止被非 `ended` 的旧记录覆盖状态）。

---

## 缺陷二（中危）：`clean` 未持 `build_removal_lock` 且与 `finalize` 并发删目录写库

### 位置

- `task-server/task_server.py:4843-4844`（`Scheduler.finalize`）
- `task-server/task_server.py:4619-4650`（`Scheduler.remove_finished_build_directories`、`Scheduler.remove_finished_build_directories_locked`）
- `task-server/task_server.py:7388-7446`（`Scheduler.clean`）

### 根因与触发路径

1. 在 `finalize`（`task-server/task_server.py:4843-4844`）中：
   ```python
   self.notify_finished(task["id"])
   self.remove_finished_build_directories(task)
   ```
   服务先调用 `notify_finished` 唤醒阻塞在 `GET /tasks/{id}/wait` 上的客户端，然后才调用 `remove_finished_build_directories(task)` 删除编译目录。
2. 客户端从 `task wait <id>` 拿到结局后立即按标准流程执行 `task clean <id>`。`remove_finished_build_directories`（`task-server/task_server.py:4625`）与 `remove_cancelled_artifacts`（`task-server/task_server.py:4694`）都通过 `with self.build_removal_lock:` 互斥，唯独 `clean`（`task-server/task_server.py:7416-7434`）**没有获取 `self.build_removal_lock`**（同样地，服务启动时的后台线程 `sweep_finished_build_directories` 也持 `build_removal_lock`，而与 `clean` 不互斥）。
3. 当 `clean` 与 `finalize` 中的 `remove_finished_build_directories` 并发执行时：
   - 两者同时在 `task-server/task_server.py:4633` 与 `7409` 读出 `build_directory_removed == False` 的尝试记录，并对同一个编译目录并发调用 `remove_run_build_directory`；
   - 若一侧先把目录删完，另一侧在 `os.path.lexists(directory)` 或 `rmtree` 过程中可能观察到半删状态或报假失败；
   - 两侧随后各自调用 `self.store.update("attempts", attempt)`（`task-server/task_server.py:4645, 7427`）整行写回各自更早读出的 `attempt` 字典。

### 修复建议

1. 在 `finalize` 中先完成 `self.remove_finished_build_directories(task)`，再调用 `self.notify_finished(task["id"])` 唤醒 `wait`。
2. 在 `clean` 删除编译目录与现场目录的区段外层加 `with self.build_removal_lock:`，并在锁内现读尝试记录，用 `self.store.change("attempts", attempt["id"], {"build_directory_removed": True})` 替代整行 `self.store.update("attempts", attempt)`。

---

## 缺陷三（中危）：`Resources.recheck` 漏调 `disk_problem`、`launch_recheck` 漏调 `peer_view`，且 `PassView.refresh` 未同步更新同轮另一张快照

### 位置

- `task-server/task_server.py:1772-1779, 1796-1835`（`Resources.snapshot`、`Resources.disk_problem`、`Resources.recheck`）
- `task-server/task_server.py:1856-1861, 1893-1897`（`PassView.snapshot`、`PassView.refresh`）
- `task-server/task_server.py:6205-6223`（`Scheduler.build_disk_problem`）
- `task-server/task_server.py:6620-6644`（`Scheduler.launch_recheck`）

### 根因与触发路径

1. **`Resources.recheck` 未调用 `disk_problem`，导致起之前复核及同轮后续派活跳过 50 GiB 盘余量下限与第二台编译目录占盘检查**：
   - 在 `Resources.snapshot`（`task-server/task_server.py:1772-1775`）中，每台机器的 `report` 都会调用 `self.disk_problem(machine, report, fake)`：
     - 对于第二台（`peer`），`PeerMachine.probe` 返回的报告中不含 `DISK_REPORT_KEY`，全靠 `disk_problem` 调用 `machine.disk_free` 并将可用字节数写入 `report[DISK_REPORT_KEY]`（`task-server/task_server.py:1810-1814`）；
     - 若本机或第二台可用盘空间低于 `DISK_FLOOR_BYTES`（50 GiB）或查询失败，`disk_problem` 返回原因，`snapshot` 将该台从 `reports` 中移除并记入 `down`（本轮不派该台）。
   - 然而 `Resources.recheck`（`task-server/task_server.py:1819-1835`）只执行了 `report = machine.probe(self, gpu)[0] if machine.reachable(self)[0] else None`，**完全没有调用 `self.disk_problem(machine, report, fake)`**：
     - 当任务被选定派往第二台并进入 `launch_recheck`（`task-server/task_server.py:6633`）时，`recheck` 返回的第二台 `report` 中没有 `DISK_REPORT_KEY`。随后 `launch_recheck` 经 `self.host_admission` 调用 `self.build_disk_problem`（`task-server/task_server.py:6211-6213`）：
       ```python
       available = (snapshot.reports.get(host) or {}).get(DISK_REPORT_KEY)
       if not isinstance(available, int) or isinstance(available, bool):
           return ""
       ```
       因 `available` 为 `None`，`build_disk_problem` 直接返回空串放行。不仅当前任务在复核时跳过了第二台的编译目录占盘检查，而且 `view.refresh` 将该快照存入 `PassView` 后，**同一轮后续排队的所有任务在评估第二台时也全部跳过 `build_disk_problem`**。
     - 同样地，若本机或第二台的盘余量在排队初判后跌破 50 GiB（`DISK_FLOOR_BYTES`），`recheck` 不会拦截，任务仍照起。

2. **`launch_recheck` 未经过 `self.peer_view`**：
   - `PassView.snapshot`（`task-server/task_server.py:1859`）会调用 `self.scheduler.peer_view(...)`，其中 `cards_in_service` 会过滤掉 `self.quarantined_cards` 中仍在隔离期的坏卡，并在 `peer-nfs-missing` 等状态下从快照中移除第二台。
   - 而 `launch_recheck`（`task-server/task_server.py:6633`）直接写：
     ```python
     snapshot = self.unstarted_slices_filled(self.resources.recheck(snapshot_gpu, host, view.snapshot(snapshot_gpu)))
     view.refresh(host, snapshot_gpu, snapshot)
     ```
     既未经过 `peer_view`（虽然 `base` 经过了 `peer_view`，但若在 `base` 与 `recheck` 之间有其他改变或 `fake` 复核更新了 `reports[PEER]`，不会重新过滤），也未像 `PassView.snapshot` 那样保持一致的视图处理。

3. **`PassView.refresh` 只更新 `self.snapshots[gpu]`，遗留 `self.snapshots[not gpu]` 旧快照**：
   - `PassView.snapshots` 按 `gpu` 布尔值分别缓存 `False` 和 `True` 两份 `Snapshot`（`task-server/task_server.py:1847`）。
   - 当同轮既有不要卡的任务（已填充 `self.snapshots[False]`）又有要卡的任务（已填充 `self.snapshots[True]`）时，某条任务在 `launch_recheck` 中调用 `view.refresh(host, snapshot_gpu, snapshot)`（`task-server/task_server.py:1893-1897`）只更新了 `self.snapshots[snapshot_gpu]`，而 `self.snapshots[not snapshot_gpu]` 仍保留着复核前的旧报告。若复核发现 `host` 已不可达或内存余量骤降，同轮后续 `gpu` 标志相反的任务调用 `view.snapshot(...)` 时仍会读到复核前的旧报告。

### 修复建议

1. 在 `Resources.recheck` 中拿到 `report` 后，与 `Resources.snapshot` 一样调用 `self.disk_problem(machine, report, fake)`；若返回非空问题字符串，则将 `report` 置为 `None` 并更新 `down` 与 `down_kind`。
2. 在 `PassView.refresh(self, host, gpu, snapshot)` 中，如果另一张快照 `self.snapshots.get(not gpu)` 已存在，同步将其 `reports`、`down`、`down_kind` 中 `host` 对应的项更新（或将其从 `self.snapshots` 中弹出使其在下次用到时重读）。

---

## 缺陷四（中危）：`judge_stall_stop` 与 `judge_run_limit` 后台线程按 `task_id` 调 `stop_running`，存在误杀重排新尝试及并发执行 `--recover-cleanup` 的竞态

### 位置

- `task-server/task_server.py:3194, 3210`（`Scheduler.judge_stall_stop`）
- `task-server/task_server.py:3274`（`Scheduler.judge_run_limit`）
- `task-server/task_server.py:7661-7703`（`Scheduler.stop_running`）
- `task-server/task_server.py:4403-4427`（`Scheduler.confirm_peer_cleanup`）

### 根因与触发路径

1. **后台线程按 `task_id` 查找活动尝试，可能误杀自动重排后的新尝试**：
   - 在 `judge_stall_stop`（`task-server/task_server.py:3194, 3210`）和 `judge_run_limit`（`task-server/task_server.py:3274`）中，排队线程通过：
     ```python
     threading.Thread(target=self.stop_running, args=(task["id"],), daemon=True).start()
     ```
     异步启动后台线程停单元，传入的参数是 `task["id"]` 而不是当前尝试的 `attempt["id"]`。
   - 当第 1 次尝试命中 `stalled_at_cap`（`task-server/task_server.py:3184-3195`）时，若该尝试恰好在此时自己退出（如刚被内核 OOM 终止），排队线程在随后一轮对账中会立即将第 1 次尝试收尾为 `memory-cap-hit`（`task-server/task_server.py:4243`），并在 `settle_attempt`（`task-server/task_server.py:4561-4586`）中把该任务乘 1.5 倍上限重排回 `queued`、紧接着在 `place_queued` 中拉起第 2 次尝试（写入 `attempts` 表，状态为 `starting` 或 `running`）。
   - 若刚才为第 1 次尝试启动的后台线程因线程调度延迟，此时才执行到 `stop_running` 的 `rows = self.store.attempts_of(task_id)`（`task-server/task_server.py:7666-7672`），它会把该任务名下所有处于 `ACTIVE_ATTEMPT_STATES` 的尝试都捞出来——即刚刚启动的第 2 次尝试——并对其执行 `systemctl --user stop` 将第 2 次尝试误杀。

2. **后台 `stop_running` 线程与排队对账线程并发执行 `confirm_peer_cleanup`**：
   - 对派在第二台（`peer`）的尝试，`stop_running` 在停掉本机包装单元后会直接调用 `self.confirm_peer_cleanup(attempt)`（`task-server/task_server.py:7696`）。
   - 同时，`stop_running` 停掉单元后（或 HTTP `cancel` 线程调 `stop_running` 时），排队线程在 `reconcile` 中看到 `result.json` 已写出或单元已停，也会进入 `finish_attempt` 调用 `self.confirm_peer_cleanup(attempt)`（`task-server/task_server.py:4263`）。
   - 当回执文件 `peer-cleanup.json` 状态为 `"pending"` 时，两个线程会同时拉起 `bash multi-host-run.sh --recover-cleanup <path>`（`task-server/task_server.py:4413`）并发连往第二台执行清理恢复，并且同时打开同一个日志文件 `attempt["log"]` 追加写入各自的 `stdout + stderr`（`task-server/task_server.py:4417-4418`）。

### 修复建议

1. 为 `stop_running` 增加可选参数 `attempt_id=None`（或在 `judge_stall_stop`、`judge_run_limit` 中传入 `attempt["id"]`），当指定了 `attempt_id` 时只停止该次尝试对应的单元，不触碰同一任务后续重排生成的新尝试。
2. 对同一尝试的 `confirm_peer_cleanup` 加按尝试号或回执路径的互斥锁（或在 `judge_stall_stop` / `judge_run_limit` 触发的后台停单元中只负责停本机 systemd 单元并 `wake()`，将远端回执确认统一留给排队线程的 `finish_attempt` 与取消接口同步等待的收尾路径完成）。

---

## 缺陷五（低危）：`quarantined_cards` 与 `Resources` 缓存在多线程间无锁并发读写与迭代修改

### 位置

- `task-server/task_server.py:2896-2905`（`Scheduler.cards_in_service`）
- `task-server/task_server.py:4496`（`Scheduler.settle_attempt`）
- `task-server/task_server.py:1735-1736, 1745-1794`（`Resources.refresh_peer`、`Resources.snapshot`）

### 根因与触发路径

1. `Scheduler.cards_in_service`（`task-server/task_server.py:2896-2905`）：
   ```python
   for entry, info in self.quarantined_cards.items():
       if now >= info["until"]:
           expired.append(entry)
   for entry in expired:
       self.quarantined_cards.pop(entry, None)
   ```
   该函数通过 `peer_view` 被排队线程（`task-server-scheduler`）、NFS 重查线程（`task-server-nfs-recheck`，`task-server/task_server.py:2877`）以及多个 HTTP 工作线程（如 `POST /tasks/{id}/confirm` 调 `confirmation` 再调 `budget_check`、`requeue_command_text` 调 `confirmation`）并发调用；同时排队线程在 `settle_attempt`（`task-server/task_server.py:4496`）中会向 `self.quarantined_cards` 插入新坏卡。当一个线程正在遍历 `self.quarantined_cards.items()` 时，另一个线程执行 `pop` 或插入新键，会直接触发 `RuntimeError: dictionary changed size during iteration`。
2. `Resources.snapshot`（`task-server/task_server.py:1745-1794`）与 `Resources.refresh_peer`（`task-server/task_server.py:1735-1736`）同样被排队线程、NFS 重查线程和 HTTP 工作线程（`/pins`、`/pins/show`、`/tasks/{id}`、`/tasks/{id}/confirm`）无锁并发调用，`self.cached` 与 `self.peer` 缺乏锁保护，且 `unstarted_slices_filled`（`task-server/task_server.py:6528`）会读取缓存快照中的 `reports`。

### 修复建议

1. 在 `cards_in_service` 及修改 `self.quarantined_cards` 的路径上加锁，或至少对 `list(self.quarantined_cards.items())` 做快照遍历。
2. 为 `Resources` 增加互斥锁保护 `self.cached` 与 `self.peer` 的读写与失效操作。
