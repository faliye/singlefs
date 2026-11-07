# task-server 与 prune-server 深入对攻审核（2026-11-03）

## 范围、基线与结论

本记录合并首轮逐行审查与三轮多 Agent 对攻审核结果，针对工作树实际源码。重点追踪 `task-server`（`task_server.py`、`attempt_runner.py`、`task_server_lib.py`、`multi-host-run.sh`）与 `prune-server`（`server.rs`、`committer.rs`、`storage.rs`、`journal.rs`、`reliable_client.rs`、`segmented_values.rs` 及 Python 客户端 `prune_server_client.py`）。

确认 **12 项深层问题，P1 3 项、P2 6 项、P3 3 项**。各项均包含红方攻击路径、蓝方防御抗辩、紫方最终裁决与最小修复定则。

| 编号 | 影响模块 | 优先级 | 问题简述 | 核心位置 | 证据与触发后果 |
| --- | --- | --- | --- | --- | --- |
| TS-01 | task-server | P1 | 分片失败未停运行中兄弟分片导致资源泄漏 | `task_server.py:2572-2580` | 单片失败父任务收尾，但 running 状态分片继续运行霸占 GPU/RAM |
| PS-02 | prune-server | P1 | 分段大值两阶段写非原子化致并发读崩退出 | `prune_server_client.py:774-775` | 覆写大值先写段再写清单，并发读新段旧清单触发 SHA-256 错配退出（code 3） |
| PS-04 | prune-server | P1 | Worker 线程在准入扣额前全额分配响应缓冲区 | `server.rs:1150-1156` | 32 个 Worker 线程并发前缀扫瞬时分配多 GiB 堆内存，击穿 cgroup 引发 OOM |
| TS-02 | task-server | P2 | 清理残余进程无异常隔离导致成功命令误判 lost | `attempt_runner.py:331-334` | cgroup 清理在写结果前抛异常打穿 runner，对账缺失 result.json 误判 launcher-lost |
| TS-03 | task-server | P2 | 取消操作脱离 task_lock 产生并发状态撕裂 | `task_server.py:4184-4199` | stop_running 在锁外执行，与自然成功结束发生竞态，接口回 stopped=True 但入库 succeeded |
| TS-04 | task-server | P2 | 启动复核失败虚假累加启动计数致第二台饥饿 | `task_server.py:2830-2837` | start_attempt 返回 None 仍自增 started_on，虚耗 PEER_STARTS_PER_PASS 配额 |
| TS-05 | task-server | P2 | children_of 读取时丢弃 HMAC 校验位绕过防篡改 | `task_server.py:2440-2442` | 解包丢弃 valid 标记，关联列被改或损坏的非法子任务直接装载进调度流程 |
| TS-06 | task-server | P2 | 跨机任务取消时 SSH 挂起导致远端物理机进程失管 | `task_server.py:4296-4303` | 本地 systemd SIGKILL 终止 multi-host-run.sh，远端 SSH 进程沦为孤儿进程继续执行 |
| PS-03 | prune-server | P2 | 重连无差别重发 Claim 请求导致租约非预期延长 | `reliable_client.rs:1241-1249` | 客户端重连重置 Claim 状态为 Unsent，服务端无请求号去重，单方刷新并推迟租约 |
| PS-06 | prune-server | P2 | 前缀扫超限截断丢弃条目并 256 叉递归致爆栈 | `prune_server_client.py:823-836` | 截断直接 del entries 并无界 256 叉递归探测，超大记录引发查询爆炸与 RecursionError |
| TS-07 | task-server | P3 | Store.task_locks 字典常驻无淘汰机制导致内存泄漏 | `task_server.py:458, 649-650` | 任务锁常驻字典永不删除，长周期守护运行下持有数百万锁对象，内存单调增长 |
| PS-01 | prune-server | P3 | 读线程在套接字 shutdown 前 join 写线程致半挂 | `server.rs:1714-1718` | 优雅停机顺序倒置，写端超时最长可阻塞 Reader 达 30 秒，延迟连接资源释放 |
| PS-05 | prune-server | P3 | Python 客户端缺少 100 万段数上限校验导致解析截断 | `prune_server_client.py:766-773` | 段号超 6 位产生 7 位字符串，破坏 segment_key_parts 倒数 6 位的解析约定 |

---

## 第一轮：生命周期、进程控制与资源清理对攻

### TS-01 (P1)：分片失败导致兄弟运行中分片被遗弃为永久僵尸

- **位置：** `task-server/task_server.py:2572-2580`。
- **红方攻击路径：**
  1. 提交包含 8 个分片的并行任务（需占用 8 张卡）。
  2. 第 0 片因代码错误在第 5 秒以非 0 状态退出（`failed`）。
  3. `drive_parent_phase` 被调度唤醒，检测到 `failed is not None`。
  4. 触发收尾逻辑：
     ```python
     self.finalize_parent(parent, failed, ...)
     for sibling in shards:
         if sibling["id"] != failed["id"] and sibling.get("state") == "queued":
             self.finalize(sibling, "cancelled", None, ...)
     return
     ```
  5. 此时第 1 至第 7 分片处于 `running` 状态。由于判断条件严格限制为 `== "queued"`，所有处于 `running` 或 `starting` 状态的分片被完全跳过，既不发信号停止，也不在数据库中标记。父任务已进入 `finished` 状态，后续调度轮次直接忽略该父任务。
  6. 第 1 至第 7 分片在物理宿主机上继续执行数小时，独占 GPU/RAM，集群计算槽位耗尽。
- **蓝方抗辩：**
  行 2576 注释指明“父任务先收尾，兄弟片收尾时 `drive_parent` 见它已有结局就不再动它”。兄弟分片进程正常结束后会触发自己的 `finish_attempt` 并回收，不会导致任务系统死锁。
- **紫方裁决：**
  红方利用成立，定级 P1。在长时间运行的重型任务场景下，运行中子分片未能协同熔断，会在父任务已被判负后继续无效占用稀缺硬件算力数小时。违背调度系统“单片失败全组快速熔断”原则。
- **最小修复规范：**
  在遍历兄弟分片时，若 `state in ("running", "starting")`，调用 `self.stop_running(sibling["id"])` 并将其置为 `cancelled`。

---

### TS-02 (P2)：`stop_unit_leftovers` 缺少异常隔离致成功任务误判 `launcher-lost`

- **位置：** `task-server/attempt_runner.py:331-334`。
- **红方攻击路径：**
  1. 提交耗时编译任务，子进程执行完毕，`child.returncode == 0`。
  2. 执行到行 333：`unit_leftovers = stop_unit_leftovers()`。
  3. 函数内部进入 `stop_unit_leftovers()` 读取 `cgroup.procs`。
  4. 若此时 cgroup 处于权限变更、根目录迁移（如外层 systemd 单元回收竞态）或底层文件系统因密集 I/O 抛出 `PermissionError` / `OSError`（非 `open` 内部捕获的异常），未捕获异常导致进程中断。
  5. 行 334 的 `write_json_atomically(result.json)` 尚未执行，整个 Python 运行包装器异常退出。
  6. 调度服务端在对账时读取不到 `result.json`，在行 20 按“单元不在、没留结果”将其硬性判定为 `launcher-lost`（返回码 238），彻底覆盖子任务实际的退出码 0。
- **蓝方抗辩：**
  `stop_unit_leftovers` 内部对 open 拥有 `except OSError: return []` 保护。
- **紫方裁决：**
  红方利用成立，定级 P2。`stop_unit_leftovers` 中的 `cgroup_of`、路径拼接以及进程杀灭循环并未实现全封闭异常隔离。在写 `result.json` 前夕发生未捕获异常将导致契约文件丢失。落盘前执行非核心清理动作必须包含在顶级 `try...except Exception` 块内。
- **最小修复规范：**
  将行 333 用 `try...except Exception:` 包裹，发生异常时记录 warning 并令 `unit_leftovers = None`，保证行 334 的 `write_json_atomically` 必定执行。

---

### PS-01 (P3)：服务端读线程在连接关闭前同步等待写线程致套接字半挂

- **位置：** `prune-server/src/server.rs:1714-1718`。
- **红方攻击路径：**
  1. 客户端发起长连接，发送耗时批读/前缀扫请求。
  2. 客户端单向发送 `FIN` 包发起关闭（或网络中断导致 reader 超时退出循环），reader 退出 `read_request_frame` 循环。
  3. 执行行 1714-1717：
     ```rust
     shared.connections.unregister(record.connection);
     drop(response_sender);
     let _ = writer.join();
     frames.reader.get_ref().shut_down();
     ```
  4. 此时写线程正阻塞在 `write_responses` 的 `flush` 或某次网络写入上（客户端已不再接收数据，TCP 接收窗口满）。
  5. 写线程必须等满 30 秒的 `write_deadline_timeout` 才能超时解脱。
  6. 在 30 秒内，reader 线程卡在 `writer.join()`，底层的 `stream.shut_down()` 无法被调用，操作系统文件描述符与服务端 Reader 线程长期无法释放。
- **蓝方抗辩：**
  `writer` 会在收到通道断开信号或写入错误时主动触发 `writer.get_ref().shut_down()`，正常网络环境下能够在数毫秒内解脱。
- **紫方裁决：**
  红方利用成立，定级 P3。属于停机次序不当问题。Reader 退出循环后应先调用 `stream.shut_down()` 掐断两端，使得正在等待对端响应的写操作立刻收到 `EPIPE` 快速退出，避免管理线程阻塞在 `join()` 上。
- **最小修复规范：**
  在 `let _ = writer.join()` 之前，先执行一次 `frames.reader.get_ref().shut_down()`。

---

## 第二轮：事务一致性、状态机与并发冲突对攻

### PS-02 (P1)：分段大值覆写非原子化导致并发读客户端 SHA-256 校验崩溃

- **位置：** `scripts/prune-server/prune_server_client.py:774-775, 801-812`。
- **红方攻击路径：**
  1. 服务端存储了一个大值键 `K_large`（大小 32MB，分为 2 段：`K_large\0seg000000` 与 `K_large\0seg000001`，清单位于 `K_large`）。
  2. 写入客户端 W 准备对 `K_large` 覆写新值（大小仍为 32MB，内容完全变更）。
  3. W 调用 `write_many_large`:
     ```python
     self.write_batches(segment_entries) # 写入新的 seg000000 和 seg000001
     self.write_batches(final_entries)   # 写入新的 manifest
     ```
  4. 在第 1 步完成、第 2 步尚未提交到 prune-server 的纳秒级窗口内：
  5. 读取客户端 R 发起 `read_many([K_large])`：
     - R 读到的 `K_large` 的值仍是旧的 `encode_manifest`（包含旧内容的 SHA-256 签名）。
     - R 解析清单，读取段键 `seg000000` 和 `seg000001`。
     - R 读到了 W 刚刚覆盖写进数据库的新分段内容。
  6. R 执行行 810：
     ```python
     if hashlib.sha256(whole).hexdigest() != manifest["sha256"]:
         fail("segmented-read", f"...") # 调用 sys.exit(3)
     ```
  7. 读取客户端 R 发生哈希错配并强制崩溃退出（`exit(3)`）。若写入客户端在执行完行 774 后宕机，该键永久损坏，后续任何读取该键的客户端均失败。
- **蓝方抗辩：**
  大值分段机制依赖上层通过 `claim` 租约进行独占保护，避免在覆写过程中进行并发读。
- **紫方裁决：**
  红方利用成立，定级 P1。读请求不加锁；且写客户端进程在写分段与写清单之间发生崩溃（SIGKILL、机器断电）时，该键的分段数据便与清单发生永久性状态撕裂。
- **最小修复规范：**
  写分段时不得直接覆写同名段键，应使用带有版本号或临时哈希标记的段键（如 `K\0v2_seg000000`），待新清单写入生效后再异步回收旧段；或者在底层 prune-server 引入跨键原子批量写入事务机制。

---

### TS-03 (P2)：取消操作脱离 `task_lock` 产生状态不一致判定

- **位置：** `task-server/task_server.py:4184-4199, 2148-2155`。
- **红方攻击路径：**
  1. 用户向 HTTP 接口提交 `POST /tasks/{id}/cancel`。
  2. 服务端在 `task_lock` 内将 `task["cancel_requested"]` 更新为 `True`，随后退出 `with self.store.task_lock(task_id):` 代码块。
  3. 执行行 4199：`return self.stop_running(task_id)`。
  4. 在释放锁之后、执行 `self.stop_running(task_id)` 之前的交错点：
     - 底层尝试的实际进程已经正常完成（exit code 0）。
     - 调度线程收到通知，在行 2144 调用 `finish_attempt`：
       ```python
       exit_code = result.get("exit_code") if result else None
       cancelled = bool(task.get("cancel_requested")) and (result is None or bool(result.get("stopped_signal")))
       outcome = lib.classify_attempt(exit_code, judged_host, tail, cancelled, ...)
       ```
     - 由于进程是自然退出，`result["stopped_signal"] == 0`，因此计算出的 `cancelled` 为 `False`。
     - `finish_attempt` 将该尝试状态记为 `ended`，任务状态记为 `finished`，`outcome` 记为 `succeeded`。
  5. 此时 HTTP 线程才进入 `self.stop_running(task_id)`：
     - 查询 `attempts_of(task_id)`，发现已经没有 `state in ACTIVE_ATTEMPT_STATES` 的尝试。
     - `stop_running` 遍历列表为空，返回 `200, {"stopped": True, "state": "running", ...}`。
  6. 调用端通过 HTTP 收到 `stopped: True` 成功回执，但数据库与真实最终任务状态却是 `succeeded`。
- **蓝方抗辩：**
  取消接口只要最终使任务终止，返回 `stopped: True` 并不破坏单向终态不可逆性。
- **紫方裁决：**
  红方利用成立，定级 P2。客户端依据 HTTP 接口返回的 `stopped: True` 认为已成功阻断并丢弃产物，而后对账系统读取到的任务状态却是成功并采信了对应产物。`stop_running` 判定逻辑应与终态互斥。
- **最小修复规范：**
  将 `self.stop_running(task_id)` 移入 `with self.store.task_lock(task_id):` 内，或在 `stop_running` 后再次复核数据库中的真实终态并向调用方说明。

---

### TS-05 (P2)：`children_of` 忽略 HMAC 校验位导致子任务越权篡改

- **位置：** `task-server/task_server.py:2440-2442`。
- **红方攻击路径：**
  1. `task-server` 行记录存有 HMAC 签名，读取时通过 `Store.verified` 验证。
  2. 检查 `children_of` 的实现：
     ```python
     def children_of(self, parent_id):
         return [record for record, _ in self.store.select("tasks", "task_id = ? ORDER BY id ASC", (parent_id,))]
     ```
  3. `store.select` 返回的是 `[(record, valid), ...]` 元组，行 2441 直接用 `for record, _ in ...` 无条件解包并丢弃了布尔值 `valid`。
  4. 当某行子任务的 `task_id` 关联列或正文字段被直接改写时，`valid` 判定为 `False`。
  5. 其它流程（如 `cancel`）均严格拦截未校验记录并报错 `record-tampered`（409 拒绝）。但在 `children_of` 返回时，未通过 HMAC 校验的非法子任务记录被原样交给了分片调度逻辑（`drive_parent_phase`），绕过了防篡改壁垒。
- **蓝方抗辩：**
  在 `drive_parent` 的入口处有尝试记录的校验，通常外部不会直接修改底层 SQLite 库。
- **紫方裁决：**
  红方利用成立，定级 P2。`task-server` 在架构上明确将 HMAC 防篡改作为安全边界。丢弃 `valid` 直接导致非法关联子任务被装载进内存驱动推进。
- **最小修复规范：**
  修改为 `[record for record, valid in self.store.select(...) if valid]`，并对 `not valid` 记录触发 `self.tampered` 告警。

---

### PS-03 (P2)：客户端断线重连对 `Claim` 请求无幂等去重重发

- **位置：** `prune-server/src/reliable_client.rs:1241-1249`，`prune-server/src/committer.rs:422-430`。
- **红方攻击路径：**
  1. 客户端通过 `reliable_client` 提交认领请求（`OperationKind::Claim`），租约时间设为 10 秒。
  2. 服务端 `committer` 收到并在内存成功记录该认领，将租约过期时间设为 $T_1 + 10$ 秒。
  3. 服务端在把确认帧刷入网络套接字时发生 TCP 拥塞丢包，连接异常重置。
  4. `reliable_client` 检测到断连，触发重连逻辑（行 1241-1249）：
     ```rust
     OperationKind::Write { .. }
     | OperationKind::Read
     | OperationKind::Scan
     | OperationKind::Claim
     | OperationKind::Statistics => {
         operation.state = OperationState::Unsent;
         self.diagnostics.replayed_frames += 1;
     }
     ```
  5. `Claim` 请求在无需分配全局自增 `request_number` 的情况下直接重发。重连耗时 8 秒，服务端在 $T_1 + 8$ 秒重新接收到该认领帧。
  6. 服务端 `add_claim`（`committer.rs:454-463`）见持有者仍是该客户端，直接将其过期时间刷新为 $(T_1 + 8) + 10 = T_1 + 18$ 秒。
  7. 原本在 $T_1 + 10$ 秒应当过期的租约，由于客户端单方面的断线重发，在没有任何分布式锁重新协调的情况下被静默续约延长了 8 秒，导致其它等待锁超时的并发客户端发生长时间非预期阻塞。
- **蓝方抗辩：**
  认领具备重入性（同一个 client 再次 claim 自己持有的 key 属于合法续租操作），重发不会造成键所有权被第三方篡夺。
- **紫方裁决：**
  红方利用成立，定级 P2。写入操作拥有严格的 `request_number` 和 journal 去重机制，而 `Claim` 操作被无条件列入重播且缺乏全局请求序列号防护。网络断续时，已发出的 `Claim` 会在不可预知的时间点重复续租，破坏租约期限控制预期。
- **最小修复规范：**
  为 `Claim` 操作绑定唯一的请求标识，或者在重连重发前由客户端检查本地超时时限，超时则标记失败而不盲目重播。

---

## 第三轮：协议边缘、网络高并发与多机对攻

### PS-04 (P1)：Worker 线程准入扣额前全额分配响应缓冲区引发 OOM 崩溃

- **位置：** `prune-server/src/server.rs:1150-1156, 1945-1950`。
- **红方攻击路径：**
  1. prune-server 的 Worker 线程池默认配置拥有 32 个 Worker 线程。
  2. 同时建立 32 条 TCP 连接，每条连接并发发送一个前缀扫（`scan_prefix`）或成批点读（`read_many`）请求。
  3. 每个请求命中大量数据，返回载荷接近协议单帧上限 64 MiB（`MAXIMUM_FRAME_PAYLOAD_BYTES`）。
  4. 审计行 1150-1152 的执行次序：
     ```rust
     let mut response = process_frame(&shared, &body, &connection);
     // 回应记进额度才交出去...记不进就换成忙
     if !slot.reserve_response(response.len()) {
         // 替换为 encode_busy
     }
     ```
  5. `process_frame` 是在扣减内存额度之前在当前 Worker 线程的堆上完整把 64 MiB 的 `Vec<u8>` 创建并拼装完毕。
  6. 32 个 Worker 线程同时执行 `process_frame`，瞬时在堆上无节制分配 $32 \times 64 \text{ MiB} = 2,048 \text{ MiB (2 GiB)}$。
  7. 如果宿主机配置的 cgroup 内存上限为 1.5 GiB，尽管 `slot.reserve_response` 在事后会判定额度超限并替换为 `busy` 响应，但这 2 GiB 的物理内存分配已经在此之前发生，直接触发操作系统内核 OOM Killer 杀掉 `prune-server` 进程。
- **蓝方抗辩：**
  服务端在接收帧体时有 `read_request_frame` 的 `reserve_bytes` 准入拦截，非法超长输入在入口已经被阻断。
- **紫方裁决：**
  红方利用成立，定级 P1。入口处的 `reserve_bytes` 仅拦截了“请求帧体的大小”，无法限制“查询生成的回应大小”。在全额分配出 64 MiB 巨型载荷之后再去检查 `reserve_response`，使得背压机制形同虚设。
- **最小修复规范：**
  在 `scan_prefix` 和 `read_many` 处理逻辑内部引入动态额度预占机制（流式分块累加或预先评估），超出直接报错，不得预先分配全额响应缓冲区。

---

### PS-06 (P2)：`raw_scan_stream` 在超限截断时触发 256 叉递归爆炸与爆栈

- **位置：** `scripts/prune-server/prune_server_client.py:823-836`，`prune-server/src/storage.rs:608-628`。
- **红方攻击路径：**
  1. 服务端 `storage.rs:626-631` 的前缀扫在遇超大记录时，由于首条记录体积即超出 `maximum_bytes`，返回 `entries = []` 且 `is_truncated = true`。
  2. 客户端 `prune_server_client.py:823-836`：
     ```python
     entries, truncated = self.raw_scan(prefix)
     if not truncated:
         yield from entries
         return
     del entries # 丢弃已有条目
     exact = self.read_many_raw([prefix])[0]
     if exact is not None:
         yield prefix, exact
     for byte_value in range(SCAN_SPLIT_BYTE_VALUES): # 256 叉循环展开
         yield from self.raw_scan_stream(prefix + bytes([byte_value]))
     ```
  3. 面对超限记录，由于该记录在其所有更长前缀下依然超出帧长限制，每个子分支均反复返回 `entries=[], truncated=True`。
  4. 递归树按 $256^D$ 呈指数级扩散，向 prune-server 发出大量高频 TCP 查询，最终客户端抛出 `RecursionError: maximum recursion depth exceeded` 崩溃。
- **蓝方抗辩：**
  该机制是为了兼容“无分页游标”协议设计的前缀树深度优先遍历算法，在键分布均匀且单值较小的情况下能正常完成遍历。
- **紫方裁决：**
  红方利用成立，定级 P2。客户端在截断时直接丢弃已有成果（`del entries`）且用 256 叉递归代替游标分页，面对超长记录必定引发雪崩式查询放大与 Python 爆栈。
- **最小修复规范：**
  在协议中增加基于游标的续扫语义；或者客户端必须消费并推进当前非空的 `entries`，并设置最大递归分支深度拦截。

---

### TS-04 (P2)：调度复核失败虚假累加启动计数致多机并发饥饿

- **位置：** `task-server/task_server.py:2830-2837`。
- **红方攻击路径：**
  1. 调度循环针对第二台机器配置了启动限流阈值：`PEER_STARTS_PER_PASS = 4`。
  2. 队列头部的任务 T 派发到第二台，执行行 2830：`attempt = self.start_attempt(task, placement)`。
  3. 假定此时发生突发状态变更（如任务被并发取消、Store 拒绝事务），`start_attempt` 返回 `None`。
  4. 行 2837 执行：
     ```python
     if attempt is not None and attempt.get("state") in ACTIVE_ATTEMPT_STATES:
         view.book(...)
     started_on[placement.host] = started_on.get(placement.host, 0) + 1
     ```
  5. `started_on[placement.host]` 的递增操作独立于 `attempt is not None`。即使任务实际未启动，第二台计数器依然自增。
  6. 4 次失败即可使 `started_on[host]` 达到上限，队列中排在后面的健康任务被行 2819 拦截，本轮调度丧失对第二台的派发能力。
- **蓝方抗辩：**
  下一轮 pass（数秒后）计数器会重新重置为 0，不会构成永久性死锁。
- **紫方裁决：**
  红方利用成立，定级 P2。计量指标与实际动作脱节，在队列发生异常时，调度器会因为虚假耗尽限流配额而在数秒内拒绝派发。
- **最小修复规范：**
  将 `started_on[placement.host] += 1` 移动至 `if attempt is not None and attempt.get("state") in ACTIVE_ATTEMPT_STATES:` 代码块内部。

---

### TS-06 (P2)：跨机任务取消时 SSH 挂起导致远端物理机进程永久失管

- **位置：** `task-server/task_server.py:4296-4303`，`scripts/hosts/multi-host-run.sh:409-424`。
- **红方攻击路径：**
  1. 任务派发至第二台物理机，本地通过包装脚本 `multi-host-run.sh --peer` 起 systemd 单元，远端通过 SSH 执行真实进程。
  2. 取消指令进入 `stop_running`，执行 `systemctl --user stop <unit>`。
  3. 若此时主辅节点之间网络闪断或 SSH 守护进程无响应，`multi-host-run.sh` 的清理逻辑被阻塞。
  4. 本地 systemd 在等待 150 秒后超时，发送 `SIGKILL` 终止本地 `multi-host-run.sh` 进程。
  5. 本地单元退出，cgroup 变空。服务端回读认为任务已停止（`stopped: True`）。然而远端物理机上的 SSH 子进程因父进程暴毙丢失信道，转为由远端 PID 1 接管的孤儿进程，继续在第二台上永久运行。
- **蓝方抗辩：**
  `multi-host-run.sh` 内置了 trap 陷阱以及远端心跳超时探测。
- **紫方裁决：**
  红方利用成立，定级 P2。SIGKILL 会直接终止本地 trap 处理。跨机调度在取消时不能仅依赖本地 systemd 单元退出，必须有主控端直接对辅助节点的反向探活和主动追杀凭据。
- **最小修复规范：**
  在 `stop_running` 中对于跨机任务，若本地进程被杀或超时，必须向第二台直接发起一条带确定性超时限制的 `ssh <peer> kill` 保底命令。

---

### PS-05 (P3)：Python 客户端缺少段数上限约束导致 Rust 协议校验解析失败

- **位置：** `scripts/prune-server/prune_server_client.py:766-773, 904-905`，`prune-server/src/segmented_values.rs:26-36`。
- **红方攻击路径：**
  1. Rust 端对分段键定义了 `SEGMENT_INDEX_LIMIT = 1_000_000`，超限直接拒绝。
  2. Python 端实现为 `b"%0*d" % (SEGMENT_INDEX_DIGITS, index)`。
  3. 当段号达到 1,000,000 时，Python 格式化产生 7 位数字 `b"1000000"`。
  4. Python 自身及 Rust 客户端在调用 `segment_key_parts` 时固定截取末尾 6 位数字，导致解析出的键变成带 `1` 的残缺原键，段号变为 `000000`，造成键空间重叠覆盖。
- **蓝方抗辩：**
  单键达到该段数在当前场景下属于理论极端情况。
- **紫方裁决：**
  红方利用成立，定级 P3。属于跨语言协议规范实现不对齐。
- **最小修复规范：**
  在 Python 的 `write_many_large` 中加入段数断言：若 `count >= 10**SEGMENT_INDEX_DIGITS`，抛出 `fail("too-many-segments", ...)` 拒绝写入。

---

### TS-07 (P3)：`Store.task_locks` 映射表单调增长引发常驻内存泄漏

- **位置：** `task-server/task_server.py:458, 649-650`。
- **红方攻击路径：**
  1. `Store.task_lock(task_id)` 使用 `self.task_locks.setdefault(task_id, threading.Lock())`。
  2. 全仓代码中 `self.task_locks` 字典没有任何 `del`、`pop` 或淘汰机制。
  3. 系统连续运行并执行数百万批次任务后，字典积压大量已被清理销毁的历史任务锁对象，内存单调增长。
- **蓝方抗辩：**
  单个锁对象开销极小，数万次任务占用极少。
- **紫方裁决：**
  红方利用成立，定级 P3。作为长驻后台系统，资源池必须拥有生命周期闭环。
- **最小修复规范：**
  在任务被最终清理收尾（`mark_collected` 或归档）时，从 `task_locks` 中显式清除对应的 `task_id`。
