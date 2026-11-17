# 2026-11-07 task-server 全量严格审核报告

审核范围覆盖 `task-server/` 下全部文件：
- `task-server/task_server.py`（8144 行）
- `task-server/task_server_lib.py`（4126 行）
- `task-server/attempt_runner.py`（449 行）
- `task-server/control`（406 行）
- `task-server/task`（541 行）
- `task-server/README.md`（187 行）
- `task-server/cleanup-protocol.md`（19 行）
- `task-server/singlefs-task-server.service.in`（18 行）
- `task-server/tests/test_cleanup_protocol.py`（308 行）

---

## 汇总

| 编号 | 级别 | 位置 | 问题概要 |
| :--- | :--- | :--- | :--- |
| 缺陷一 | 高危 | `task-server/task_server.py:999-1020, 1157-1166, 2650-2663, 3011-3020, 7715-7723` | `Scheduler.tampered` 与 `Store.write_row` 不更新 SQL `task_id` 关联列且不修正非法的 `parent_id`，遇到被篡改的 `attempts` 行也不将其置为 `ended`，导致写回后依然校验失败、引发无限告警风暴并永久泄漏机器资源账本 |
| 缺陷二 | 高危 | `task-server/attempt_runner.py:270-286` | `Sampler.scan_build_log` 在单轮日志增长达到 `LOG_SCAN_BYTES`（4 MiB）时未按末尾 `\n` 截断，将跨 4 MiB 边界的 `Finished` 行劈成两半漏判，导致编译段峰值 `build_phase_peak` 丢失为 `None` |
| 缺陷三 | 高危 | `task-server/task_server.py:4997-5031, 5114-5116, 7510-7536` | `Scheduler.cancel` 未限制仅 `role == "shard"` 的子任务可单独取消；单独取消 `role == "estimate"` 估算子任务会触发 `drive_item_estimate` 将父任务误放行进 `queued` 正式执行 |
| 缺陷四 | 中危 | `task-server/task_server.py:5929-5939, 6625-6650` | `Scheduler.launch_recheck`（起之前复核）重新计算 `fresh_prediction` 后未复核 `alternation.blocked` 交替封组条件，预测被兄弟片抬高成整片后可绕过封组直接启动 |
| 缺陷五 | 中危 | `task-server/task_server.py:1977, 5160, 5914, 5919, 6203, 6487, 6662` | `busy_cards`、`server_pending`、`server_disk_pending`、`room_by_the_formula` 等 7 处在跑尝试资源核算查询忽略行签名校验位 `valid`，未通过校验的脏行会永久霸占显卡与内存额度 |
| 缺陷六 | 中危 | `task-server/control:327-349` | `control clear-queue` 停服检查未核 `unit_processes()` 与 `server.sock`；删表后任务 `id` 从 1 重新编号而未轮换 `meta` 表 `tag`，新任务日志通过 `O_APPEND` 直接拼入清库前同名旧日志文件 |
| 缺陷七 | 中危 | `task-server/task_server.py:7282-7375` | `confirm` 与 `requeue` 在 `task_lock` 锁外执行 `counts_mismatch`，且缺乏会话级串行保护，同会话并发确认可绕过在跑任务数 `tasks` 核对 |
| 缺陷八 | 中危 | `task-server/task_server.py:2287-2309` | `prove_red_uses_gpu` 对 GPU 特性的识别漏判 `--all-features`、`-F`、`tests-on-a-real-gpu`、`<crate>/gpu` 及空格分隔特性，与 `cargo_feature_gpu` 和 `mutation_gate_uses_gpu` 不一致 |
| 缺陷九 | 低危 | `task-server/README.md:13, 25`、`task-server/task_server_lib.py:1117, 1829-1831`、`task-server/task:276-370` | 设计文档与注释中 `ITEM_ENTRIES` 仍写 3 条而代码实际为 5 条；`README.md` 漏列 `confirm`/`requeue`；`task status` 客户端未渲染服务端返回的 `item_estimate` 子任务详情；常驻内存字典随历史任务单调增长 |

---

## 一、 高危漏洞与状态机缺陷 (High)

### 缺陷一：篡改检测收尾 (`Scheduler.tampered`) 不修复 `task_id` 关联列且不终结损坏的 `attempts` 行，导致无限告警与资源永久泄漏

#### 位置
- `task-server/task_server.py:999-1020`（`Store.verified`）
- `task-server/task_server.py:1157-1166`（`Store.write_row`）
- `task-server/task_server.py:2650-2663`（`Scheduler.tampered`）
- `task-server/task_server.py:3011-3020`（`Scheduler.reconcile`）
- `task-server/task_server.py:7715-7723`（`Scheduler.summary`）

#### 根因与触发路径

1. **写回后关联列仍然不匹配（`valid` 恒为 `False`）**：
   `Store.verified`（`task-server/task_server.py:1016-1019`）对 `tasks` 与 `attempts` 表校验关联列一致性：
   ```python
   if (record.get("task_id") or 0) != (column_task_id or 0):
       return False
   if table == "tasks" and record.get("parent_id") and record.get("parent_id") != record.get("task_id"):
       return False
   ```
   当某行因 `column_task_id` 与 JSON 正文 `record["task_id"]` 不一致（TS-05 场景）或 `parent_id != task_id` 而触发 `Scheduler.tampered`（`task-server/task_server.py:2650-2660`）时，`tampered` 调用 `self.store.update("tasks", record)` 将任务改为 `state="finished", outcome="record-tampered"`。
   然而 `Store.write_row`（`task-server/task_server.py:1161-1162`）执行的 SQL 是：
   ```python
   connection.execute(
       f"UPDATE {table} SET state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
       (state, session, command_key, text, self.signature(table, record["id"], state, session, command_key, text), record["id"]),
   )
   ```
   它**从不更新 SQL 的 `task_id` 列**，同时 `tampered` 也没有将 `record["task_id"]` 修正为 `column_task_id` 或修正 `parent_id`。因此 `tampered` 重签写回后，该行在之后每一次 `store.task()` 或 `store.select()` 读取时，`verified()` **依然返回 `False`**。

2. **每次调用 `summary` 重复触发 `tampered` 写入告警**：
   在 `Scheduler.summary`（`task-server/task_server.py:7720-7722`）中：
   ```python
   if not valid:
       self.tampered(task_id, task, "任务记录")
       task, valid = self.store.task(task_id)
   ```
   由于上一步写回后 `valid` 仍为 `False`，每次对该任务执行 `task status`、`wait` 或内部调用 `summary`，都会再次调用 `self.tampered(...)`，不断向 `alerts` 表和 `alerts.jsonl` 追加重复的 `record-tampered` 告警。

3. **`attempts` 行校验失败时未改 `attempts.state`，导致资源永久泄漏**：
   在 `Scheduler.reconcile`（`task-server/task_server.py:3011-3020`）中：
   ```python
   for attempt, valid in self.store.select("attempts", "state IN ('starting', 'running')"):
       ...
       if not valid or not task_valid:
           self.tampered(task["id"], task, "尝试记录" if not valid else "任务记录")
           continue
   ```
   当 `attempt` 记录本身校验不过（`not valid`）时，`tampered` 只修改了 `tasks` 表中父任务的状态为 `finished`，**并未将该 `attempt` 的 `state` 从 `'starting'`/`'running'` 移出**。下一轮 `reconcile` 在第 3016 行见 `task.get("state") == "finished"` 直接 `continue` 跳过，导致该条 `attempt` 永远留在 `state IN ('starting', 'running')` 中，被后续所有在跑资源统计函数（见缺陷五）永久计入占用。

#### 修复建议
- 在 `Store.write_row` 中对 `SIGNED_TABLES` 一并更新 `task_id = ?`（取 `int(record.get("task_id") or 0)`）；
- 在 `Scheduler.tampered` 中规范化 `record["task_id"] = int(record.get("parent_id") or 0)`（若非有效子任务则清零 `task_id` 与 `parent_id`），使写回后的记录能够通过 `verified()`；
- 当 `reconcile` 因 `not valid` 触发 `tampered` 时，同步将该 `attempt` 行修正并更新为 `state = "ended", outcome = "record-tampered"`。

---

### 缺陷二：`attempt_runner.py` 构建日志扫描在单次增长满 4 MiB 时截断行边界，漏判 `Finished` 导致 `build_phase_peak` 丢失

#### 位置
- `task-server/attempt_runner.py:270-286`（`Sampler.scan_build_log`）

#### 根因与触发路径

`Sampler.scan_build_log` 每秒增量读取尝试日志以寻找 cargo 输出的 `Finished ... target(s) in ...` 行：
```python
with open(self.log_path, "rb") as handle:
    handle.seek(self.log_offset)
    chunk = handle.read(LOG_SCAN_BYTES)
if not chunk:
    return
complete = chunk[:chunk.rfind(b"\n") + 1] if len(chunk) < LOG_SCAN_BYTES else chunk
if not complete:
    return
self.log_offset += len(complete)
```
对比 `task_server.py:4208`（`Scheduler.log_content`）中的正确写法：
```python
complete = chunk[:chunk.rfind(b"\n") + 1] or (chunk if len(chunk) == LOG_SCAN_BYTES else b"")
```
`attempt_runner.py:279` 的三元表达式把条件写反了：
- 当日志在 1 秒采样间隔内增长不足 4 MiB（`len(chunk) < LOG_SCAN_BYTES`）时，代码按最后一个 `\n` 截断；
- **当日志在 1 秒内增长达到或超过 4 MiB（`len(chunk) == LOG_SCAN_BYTES`）时**，代码直接令 `complete = chunk`，**完全不按最后一个 `\n` 截断**，并将 `self.log_offset` 直接推进 4 MiB！
- 这导致 4 MiB 边界处的一行被硬生生劈成两半：前半截在本次循环中检测，后半截在下次循环中检测。如果 cargo 的 `Finished ... target(s) in ...` 行恰好跨在 4 MiB 边界上，前后两半都无法匹配正则 `self.finished_line`，`self.build_finished_seen` 保持 `False`，最终 `result.json` 中的 `build_phase_peak` 变为 `None`，同编译族（`build_family`）无法借用该次尝试的编译段内存峰值。

#### 修复建议
将 `task-server/attempt_runner.py:279` 改为：
```python
complete = chunk[:chunk.rfind(b"\n") + 1] or (chunk if len(chunk) == LOG_SCAN_BYTES else b"")
```
即优先截取到最后一个换行符为止；仅当整整 4 MiB 内连一个换行符都没有（超长单行）时才整块推进。

---

### 缺陷三：未限制单独取消非分片子任务，单独 `task cancel` 估算子任务 (`role="estimate"`) 会误将父任务放行进队执行

#### 位置
- `task-server/task_server.py:7510-7536`（`Scheduler.cancel`）
- `task-server/task_server.py:5114-5116`（`Scheduler.cancelled_alone`）
- `task-server/task_server.py:4997-5031`（`Scheduler.drive_item_estimate`）

#### 根因与触发路径

1. 根据设计规范（`task-server/task_server_lib.py:926-930`），只有分片阶段的一片（`role == "shard"`）允许被会话单独取消（记 `cancel_origin = "session"`，在合并阶段作为 `missing_shards` 跳过）。
2. 但 `Scheduler.cancel`（`task-server/task_server.py:7511`）在判断是否为子任务单独取消时，只检查了 `if task.get("parent_id")`，没有检查 `task.get("role") == "shard"`：
   ```python
   alone = {"cancel_origin": "session"} if task.get("parent_id") and not lib.broken("shard-cancel-ends-whole", self.environment) else {}
   ```
3. 若对按项估算子任务（`role == "estimate"`）调用 `task cancel <估算子任务号>`：
   - 估算子任务被取消并以 `state = "finished", outcome = "cancelled"` 收尾；
   - 父任务的 `phase` 仍为 `"estimate"`（并非 `"cancelling"`，因为没有调父任务的 `cancel_estimating`）；
   - 随后 `drive_item_estimate`（`task-server/task_server.py:5011-5030`）见 `estimate.get("state") == "finished"`，将其视作“估算子任务没跑成、没有按项预计用时”，直接把父任务的状态改为 `state = "queued"` 排进正式执行队列！
4. 同理，若对 `role == "plan"` 或 `role == "merge"` 的分片子任务单独调用 `task cancel <子任务号>`，父任务未进入 `cancel_sharded`，却因计划或合并子任务 `outcome == "cancelled"` 而直接结束，绕过了整件取消的兄弟片停止与取消产物清理（`remove_cancelled_artifacts`）。

#### 修复建议
- 在 `Scheduler.cancel` 开头若 `task.get("parent_id")` 且 `task.get("role") != "shard"`，直接返回 `409` 拒绝单独取消，提示调用方取消父任务 `task cancel <parent_id>`；
- 在 `Scheduler.drive_item_estimate` 中，若估算子任务的 `outcome == "cancelled"`，将父任务以 `cancelled` 收尾而不放行进 `queued`。

---

## 二、 中危逻辑缺陷、竞态与资源记账漏洞 (Medium)

### 缺陷四：`launch_recheck`（起之前复核）更新预测后未复核交替封组 (`Alternation.blocked`)

#### 位置
- `task-server/task_server.py:6625-6650`（`Scheduler.launch_recheck`）
- `task-server/task_server.py:5929-5939`（`Scheduler.shard_cap_fitted_to_room`）

#### 根因与触发路径

在 `place_queued_into`（`task-server/task_server.py:5442-5456`）中，当某条排队任务通过 `self.place(...)` 后，会调用 `self.launch_recheck(task, placement, view)`：
1. `launch_recheck` 调用 `self.fresh_prediction(task)` 重新计算该任务的内存预测（例如对于 `role == "shard"` 的分片子任务，若在上一轮预测与本次起之前之间，同一计划的兄弟片刚刚跑完或撞了上限，`shard_family_peak` 会把该片的预测上限与记账 `booked` 大幅抬高）。
2. `launch_recheck` 随后仅调用了 `self.host_admission(task, prediction, host, snapshot, view, uses_card)` 检查内存、编译目录占盘与 CPU 闸，**完全没有重新调用 `alternation.blocked(task, host, on_host.memory_booked, snapshot)`**！
3. 如果某任务在 `place()` 时还是碎片（或因 `booked` 较小可塞进当前碎片组），而在 `launch_recheck()` 中因 `fresh_prediction` 抬高了 `booked`（或者因 `snapshot` 刷新后该机器 `margin` 变大导致 `whole_threshold` 变小），让新的 `booked` 跨过了碎片组剩余额度甚至变成了整片，它会直接绕过 `Alternation` 封组检查被启动，并在随后 `alternation.started()`（`task-server/task_server.py:5456`）中破坏该机器的交替账本。

#### 修复建议
在 `launch_recheck`（或 `place_queued_into` 调用 `launch_recheck` 后）若 `alternation is not None`，使用更新后的 `placement.booked` 与新 `snapshot` 复核一次交替封组条件。

---

### 缺陷五：在跑尝试资源核算查询忽略行签名校验 (`valid`)，被篡改或损坏的 `attempts` 行会永久污染调度账本

#### 位置
- `task-server/task_server.py:5160-5163`（`Scheduler.busy_cards`）
- `task-server/task_server.py:6487-6495`（`Scheduler.server_pending`）
- `task-server/task_server.py:6203-6208`（`Scheduler.server_disk_pending`）
- `task-server/task_server.py:5914`（`Scheduler.room_by_the_formula`）
- `task-server/task_server.py:5919-5920`（`Scheduler.beside_whole_capped`）
- `task-server/task_server.py:6662-6663`（`Scheduler.cpu_problem`）
- `task-server/task_server.py:1977`（`Alternation.__init__`）

#### 根因与触发路径

上述 7 处函数在遍历 `self.store.select("attempts", "state IN ('starting', 'running')")` 时，全部丢弃了第二个返回值 `valid`（写作 `for attempt, _ in ...`）。一旦库中存在签名校验不过（`valid == False`）且 `state` 处于 `'starting'` 或 `'running'` 的尝试记录，调度器不仅不忽略它，反而无条件信任其中未经校验的 `card`、`memory_cap`、`gpu_booked`、`disk_booked`、`threads` 字段，导致显卡、内存、磁盘与 CPU 额度被脏记录永久扣减。

#### 修复建议
在所有读取活动 `attempts` 的循环中统一加上 `if not valid: continue` 过滤（或封装统一的 `self.active_attempts()` 辅助方法，只返回 `valid == True` 的活动尝试）。

---

### 缺陷六：`control clear-queue` 停服检查不完整，且删表后未轮换 `tag` 导致新旧任务日志与尝试目录混用

#### 位置
- `task-server/control:327-349`（`clear_queue`）

#### 根因与触发路径

1. **停服检查比 `stop` 弱**：
   `control stop`（`task-server/control:98-116`）会同时检查 `unit_active_state()` 与 `unit_processes()` 确保单元 cgroup 内无任何残留进程；而 `clear_queue`（`task-server/control:330`）仅检查了 `unit_active_state() not in ("inactive", "failed")`，既未检查 `unit_processes()`，也未探测 `configuration.socket` 是否有进程在监听。
2. **删表重置 `rowid` 但未轮换 `meta.tag` 导致旧日志追加污染**：
   `clear_queue` 执行 `DELETE FROM tasks` 与 `DELETE FROM attempts`，保留了 `meta` 表中的 `tag`（以及可能残留的 `pause` 状态）。由于 `tasks` 表定义为 `id INTEGER PRIMARY KEY`（无 `AUTOINCREMENT`），清空表后新提交的任务 ID 会重新从 `1` 开始分配。
   然而磁盘上的 `logs/`、`attempts/`、`shards/`、`scenes/` 下仍保留着清库前 `task-<tag>-1.attempt-1.log` 等文件。当新任务 `1` 启动第 `1` 次尝试时，`attempt_runner.py:374` 与 `append_log_lines`（`task-server/task_server.py:801-808`）均以追加模式（`"ab"` / `O_APPEND`）打开该日志文件，**新任务的输出会直接拼接在旧任务 1 的日志后面**！
   当新任务结束时，`log_tail` 与 `read_item_lines` 会读到旧任务留下的 `MULTI_HOST_RUN_COMMAND_EXIT`、`GPU_QUIESCE` 坏卡标记或 `ITEM_PLAN`/`ITEM_DONE` 行，直接导致退出码、结局分类或按项速率误判。

#### 修复建议
- 在 `clear_queue` 中补充 `unit_processes()` 非空检查与 socket 连通性检查；
- 在 `clear_queue` 的事务中生成新的 `tag = secrets.token_hex(4)` 更新 `meta` 表中的 `'tag'`，并删除 `meta` 表中的 `'pause'` 行，从而使清库后的新任务使用全新的 `<tag>` 命名空间，彻底隔离旧日志与旧目录。

---

### 缺陷七：`confirm` 与 `requeue` 的条数核对 (`counts_mismatch`) 存在 TOCTOU 竞态

#### 位置
- `task-server/task_server.py:7282-7324`（`Scheduler.confirm`、`Scheduler.counts_mismatch`）
- `task-server/task_server.py:7325-7375`（`Scheduler.requeue`）

#### 根因与触发路径

设计规范（`task-server/task_server.py:559-560`）要求 `task confirm <号> --items <N> --tasks <M>` 核对“这个会话此刻在排在跑的父任务条数 `M`”与服务现算值一致才允许进队，以防止同一会话在不知情下并发堆积多条任务。
但在实现中：
1. `self.counts_mismatch(...)` 在获取 `self.store.task_lock(task_id)` 之前执行；
2. 不同 `task_id` 各自持有独立的 `task_lock(task_id)`，没有会话级锁或确认锁。
如果同一会话连续提交了任务 A 和任务 B（此时在排在跑条数均为 `M`），随后并发执行 `task confirm A --tasks M` 和 `task confirm B --tasks M`，两个请求会同时通过 `counts_mismatch` 核对并双双进队，使该会话在排在跑任务数变为 `M + 2`，绕过了二次确认对并发任务数的约束。

#### 修复建议
在 `Scheduler` 中引入确认锁（如 `self.confirm_lock`），将 `confirm` 与 `requeue` 中的 `counts_mismatch` 核对及 `store.change` 状态切换放在同一临界区内执行。

---

### 缺陷八：`prove_red_uses_gpu` 识别 GPU 特性的规则不完整，与 `cargo_feature_gpu` / `mutation_gate_uses_gpu` 不一致

#### 位置
- `task-server/task_server.py:2287-2309`（`prove_red_uses_gpu`）
- 对比 `task-server/task_server.py:2252-2270`（`cargo_feature_gpu`）与 `task-server/task_server.py:2311-2336`（`mutation_gate_uses_gpu`）

#### 根因与触发路径

`prove_red_uses_gpu` 解析 `crates/mutations.tsv` 第 5 列参数时仅写了：
```python
arguments = row[4].split()
features = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == "--features"]
features += [word.split("=", 1)[1] for word in arguments if word.startswith("--features=")]
if any("gpu" in feature.split(",") for feature in features):
    return True
```
与 `cargo_feature_gpu` 和 `mutation_gate_uses_gpu` 相比，`prove_red_uses_gpu` 存在以下漏判：
1. 未识别 `--all-features`；
2. 未识别短选项 `-F gpu` 与 `-Fgpu`；
3. 未识别 `GPU_FEATURE_NAMES` 中的 `tests-on-a-real-gpu`；
4. 未识别带包名前缀的 `<pkg>/gpu`（`feature.split(",")` 未做 `.split("/")[-1]`）；
5. 未识别以空白分隔的 `--features "a gpu"`（仅按 `,` 切分，未用 `re.split(r"[\s,]+", ...)`）。

一旦变异表某行使用了上述任一写法，`prove-red.sh` 任务会被误判为 `gpu=False`（不分卡、不套 `gpu-quiesce-run.sh` 清场包装）。

#### 修复建议
在 `prove_red_uses_gpu` 中直接调用 `cargo_feature_gpu(["cargo"] + row[4].split())` 统一判定逻辑，并在提取 `names` 时跳过 `--copy` 的参数值。

---

## 三、 设计规范/文档不一致与低危问题 (Low)

### 缺陷九：文档滞后、`task status` 漏展示 `item_estimate` 及常驻内存字典未随任务回收

#### 位置
- `task-server/README.md:13, 25`
- `task-server/task_server_lib.py:1117, 1829-1831`
- `task-server/task_server.py:4330, 6056, 7757-7764, 2540-2575`
- `task-server/task:276-370`

#### 详情
1. **按项入口数量与子命令列表文档滞后**：
   - `task-server/README.md:25`、`task-server/task_server_lib.py:1117`、`task-server/task_server.py:4330, 6056` 均写“三条按项入口（`scripts/crash/crash-point-replay.sh`、`.claude/gate.d/formal-verification.sh`、`.claude/gate.d/crates-mutation-replay.sh`）”，而 `ITEM_ENTRIES`（`task-server/task_server_lib.py:1829-1831`）实际已扩展为 **5** 条（含 `.claude/gate.d/crash-point-replay.sh` 与 `scripts/gate/merge-and-verify.sh`）。
   - `task-server/README.md:13` 的文件职责表仅列出了 `submit`、`wait`、`status`、`cancel`、`log`、`clean`，漏列了 `confirm` 与 `requeue`。
2. **`task status` 客户端未渲染 `item_estimate` 子任务详情**：
   - `Scheduler.summary`（`task-server/task_server.py:7757-7761`）专门查询估算子任务并填入返回字典的 `summary["item_estimate"]`；
   - 但 `task` 客户端的 `_print_status`（`task-server/task:276-370`）只渲染了 `summary.get("children")`，从未读取 `summary.get("item_estimate")`，导致用户查 `task status <id>` 时看不到估算子任务的尝试记录、日志路径与资源峰值。
3. **常驻内存字典随历史任务数单调增长**：
   - `Scheduler` 中的 `suspect_reported`、`attention_reported`、`attention_histories`、`workspace_write_checks`、`advance_episodes`、`alerted_child_refusals`、`inbox_failed`、`unclaimed_repushed` 在任务完结并 `collected` 后从未清理，长时间运行会持续累积历史任务键。
