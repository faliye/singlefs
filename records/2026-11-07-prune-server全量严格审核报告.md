# 2026-11-07 prune-server 全量严格审核报告

审核范围覆盖 `prune-server/` 下全部源码与相关运维、客户端脚本：
- `prune-server/src/server.rs`（2309 行）
- `prune-server/src/committer.rs`（858 行）
- `prune-server/src/storage.rs`（751 行）
- `prune-server/src/connections.rs`（426 行）
- `prune-server/src/reliable_client.rs`（2117 行）
- `prune-server/src/segmented_values.rs`（1503 行）
- `prune-server/src/protocol.rs`（1563 行）
- `prune-server/src/oversized_values.rs`（310 行）
- `prune-server/src/journal.rs`（440 行）
- `prune-server/src/configuration.rs`（236 行）
- `prune-server/src/administration.rs`（314 行）
- `prune-server/src/fault_log.rs`（207 行）
- `prune-server/src/process_watch.rs`（191 行）
- `prune-server/src/bin/prune_server.rs`（380 行）
- `prune-server/src/bin/prune_server_bench.rs`（312 行）
- `prune-server/control`（1417 行）
- `scripts/prune-server/prune_server_client.py`（1512 行）
- `scripts/prune-server/prune_server_client_selftest.py`（1356 行）
- `scripts/prune-server/prune-server-load-simulation.py`（640 行）

早前报告（`records/2026-11-06-prune-server全量代码审核.md`）所列四项历史缺陷（PS-01 认领租期缩短、PS-02 写帧存储出错放弃、PS-03 删前缀未先记配额、PS-04 删前缀未释放认领）经复核已在最新代码中确认修复落定。以下为本次代码深度全量审查新查实与确认的六处漏洞与设计不符缺陷。

---

## 汇总

| 编号 | 级别 | 位置 | 问题概要 |
| :--- | :--- | :--- | :--- |
| **缺陷一** | **高危** | `prune-server/src/committer.rs:303-306, 323-326` | 认领与删前缀无差别检查 `!group.batch.is_empty()` 拆散组提交，导致相同操作内部无法批处理并退化为每帧独立 WAL 同步 |
| **缺陷二** | **高危** | `prune-server/src/committer.rs:795-814`、`server.rs:385-406` | 组提交失败（`fail_group`）替换拒绝帧时未释放原预留配额并重复强记配额，导致连接与全局待处理字节额度虚高与双重记账 |
| **缺陷三** | **中危** | `prune-server/src/server.rs:1169-1173` | `ServiceInThisProcess::stop` 在 IPv6 监听环境下将 unspecified IP 误换为 IPv4 环回地址，导致唤醒连接失败与停机卡死 30 秒 |
| **缺陷四** | **中危** | `prune-server/src/oversized_values.rs:163-176` | 离线删除超大值（`drop-oversized-values`）未同步清理对应的认领记录，导致服务重启后旧认领幽灵复活并阻断写入 |
| **缺陷五** | **中危** | `prune-server/src/committer.rs:374-385, 137` | 组提交内存查重对同组重发请求使用后发帧的条数覆盖真实历史生效条数，破坏幂等回执一致性 |
| **缺陷六** | **低危** | `prune-server/src/connections.rs:165-190` | 未计费响应队满强关连接未在故障日志（`fault_log`）留痕，导致慢客户端静默断开且管理侧失察 |

---

## 一、 高危漏洞与并发状态机缺陷 (High)

### 缺陷一：认领与删前缀无差别检查 `!group.batch.is_empty()` 拆散组提交，导致批处理退化为每帧独立 WAL 同步

#### 位置
- [`prune-server/src/committer.rs:303-315`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L303-L315)（`Committer::process_job` 认领分支）
- [`prune-server/src/committer.rs:323-334`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L323-L334)（`Committer::process_job` 删前缀分支）
- [`prune-server/src/committer.rs:6-7, 14-16`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L6-L7)（设计契约）

#### 根因与触发路径

1. **设计契约约定**：
   在 `committer.rs` 模块设计说明（`committer.rs:6-7` 与 `14-16`）中，明确界定了组提交在遇到认领与删前缀时的提前提交条件：
   > “认领：认领记录在内存里有一份（服务起来时从库里读，之后只由这个线程改），判「别人持有」不读库；判「已有结论」读库，所以认领帧之前已收进这一组的写先提交（同步、回确认），认领读的是已落盘的库。”
   > “删前缀：同样先把这一组里已收的写提交掉（删的是已落盘的库），再扫；回应（带这一帧最后一把键）先编好、照认领的办法先记进待处理额度……”
   该契约的设计初衷十分清晰：仅当当前这一组积攒了**修改数据表的常规写批（`BatchedWrite`）**时，为了保证认领判断“已有结论”以及删前缀扫描数据键时能看到已落盘的最新事实，才需要把之前攒下的写批提前提交落盘。

2. **现行代码实现缺陷**：
   在现行代码的帧分派调度中：
   ```rust
   RequestView::Claim {
       client_identity,
       lease_milliseconds,
       keys,
   } => {
       if !group.batch.is_empty() {
           self.commit(group);
           group = self.new_group();
       }
       self.add_claim(
           &mut group,
           job,
           sequence,
           client_identity,
           lease_milliseconds,
           &keys,
       );
       group
   }
   ```
   代码并不区分当前写批是普通数据写，还是认领行写入，仅仅粗暴判断 `if !group.batch.is_empty()`。
   而在 `add_claim`（`committer.rs:574-576`）中，只要认领成功，就会调用：
   ```rust
   self.shared.library.add_claim(&mut group.batch, &claim, &record);
   ```
   把认领记录追加到 `group.batch` 中。

3. **连锁破坏后果**：
   - 当多个客户端并发或同一个流水线连接连续发送认领帧时（如判定向量编号快速登记）：
     - 第 1 帧认领到达：`group.batch` 初始为空，认领写入加入 `group.batch`，返回非空的 `group`；
     - 第 2 帧认领到达：执行 `if !group.batch.is_empty()` 判定为真，**第 1 帧认领被强行触发 `self.commit(group)`，直接调用 `flush_wal(true)` 执行一次昂贵的磁盘同步（fdatasync）！**
     - 第 3 帧认领到达：同样判定 `group.batch` 非空，再次强行提交第 2 帧！
   - 同样的问题存在于 `RequestView::DeletePrefix`（`committer.rs:323-326`）：连续的删前缀请求帧之间完全无法合并进同一个组提交批次，每一帧都会被前一帧留下的写批打断并强制执行一次独立同步。
   - 极具讽刺意味的是：如果第 1 帧认领的所有键均由于“已有结论”或“别人持有”而未认领成功（`group.batch` 恰好为空），第 2 帧认领反而能够与它合在同一个批次；而一旦认领成功，组提交就会立刻断裂。这种取决于前一帧业务结局而分化的不可预测行为，使服务端在高并发认领与连续删前缀下退化为完全串行的低效同步。

#### 修复建议与验收

1. 在 `Group` 结构体（`committer.rs:134-140`）中引入显式标记字段：
   ```rust
   struct Group {
       batch: GroupBatch,
       responses: Vec<QueuedResponse>,
       applied_in_group: HashMap<(Vec<u8>, u64), u32>,
       has_data_modifications: bool,
       has_selftest_storage_failure: bool,
   }
   ```
2. 仅在 `add_write` 写入数据行、或 `add_delete_prefix` 删除了数据行时将 `has_data_modifications` 置为 `true`。
3. 在 `RequestView::Claim` 与 `RequestView::DeletePrefix` 分支中，将拆批判断改为：
   ```rust
   if group.has_data_modifications {
       self.commit(group);
       group = self.new_group();
   }
   ```
   让连续的纯认领帧之间可以安全复用同一组提交与单次 `flush_wal`。

---

### 缺陷二：组提交失败（`fail_group`）替换拒绝帧时未释放原预留配额并重复强记配额，导致额度双重记账与虚高

#### 位置
- [`prune-server/src/committer.rs:795-814`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L795-L814)（`Committer::fail_group`）
- [`prune-server/src/committer.rs:850-853`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L850-L853)（`send_now`）
- [`prune-server/src/server.rs:385-406`](file:///home/fy5090/code/singlefs/prune-server/src/server.rs#L385-L406)（`PendingSlot` 配额伸缩）

#### 根因与触发路径

1. **配额记账生命周期**：
   在正常流程中，每个排入提交线程的请求帧都持有一个 `PendingSlot`。
   - `add_write` 通过 `queue(group, job, response, ...)` 调用 `job.slot.force_response(response.len())`，将写确认的数十字节记入 `slot`；
   - `add_claim`（`committer.rs:524`）与 `add_delete_prefix`（`committer.rs:619`）先计算正常回应的字节数 `response_frame_bytes`，调用 `job.slot.reserve_response(response_frame_bytes)` 预留数千至数十万字节的额度，再将 `job.slot` 存入 `QueuedResponse`。
2. **写失败处理逻辑**：
   当组提交发生写入失败（`GroupCommitFailure::WriteFailed` 或测试注入写失败）时，`commit` 调用 `fail_group`（`committer.rs:795-814`）：
   ```rust
   fn fail_group(&mut self, responses: Vec<QueuedResponse>, message: &str) {
       ...
       for queued in responses {
           let rejection = encode_rejection(
               queued.sequence,
               ResponseStatus::StorageFailed,
               queued.operation_code,
               message,
           );
           send_now(queued.responder, rejection, queued.slot);
       }
       ...
   }
   ```
3. **`send_now` 的二次叠加**：
   查看 `send_now` 实现：
   ```rust
   fn send_now(responder: Responder, response: Vec<u8>, mut slot: PendingSlot) {
       slot.force_response(response.len());
       send_reserved(responder, response, slot);
   }
   ```
   `send_now` 无条件执行 `slot.force_response(rejection.len())`，将新构造的拒绝帧字节数强制追加到 `slot.reservation` 中。
4. **配额泄漏与双重记账后果**：
   - `queued.response`（预留了大量字节的正常回应）被直接 drop 丢弃，但其预先划给该响应的额度**从未调用 `slot.release_response(...)` 释放**；
   - `slot` 中累积的记账不仅包含了请求体，还同时累加了“已被丢弃的正常响应预留”和“新追加的拒绝响应字节”；
   - 在写线程将拒绝帧真正发送到套接字并 drop `slot` 之前，`admission.pending_bytes` 与连接的 `budget.held_bytes` 处于严重的虚高叠加状态；
   - 在高并发或小额度限制下，这种突发性的双重记账会导致后续正常到来的并发连接或请求被错误判定为超过待处理字节上限，无辜返回 `503 Busy`。

#### 修复建议与验收

在 `fail_group` 中，对每个 `queued` 构造好 `rejection` 后，应先精确抵扣原响应的预留额度，或者直接调用专门的错误响应转发函数：
```rust
for queued in responses {
    let mut slot = queued.slot;
    slot.release_response(queued.response.len());
    let rejection = encode_rejection(...);
    send_now(queued.responder, rejection, slot);
}
```

---

## 二、 中危安全漏洞与可靠性缺陷 (Medium)

### 缺陷三：`ServiceInThisProcess::stop` 在 IPv6 监听环境下将 unspecified IP 误换为 IPv4 环回地址，导致唤醒连接失败与停机卡死 30 秒

#### 位置
- [`prune-server/src/server.rs:1164-1183`](file:///home/fy5090/code/singlefs/prune-server/src/server.rs#L1164-L1183)（`ServiceInThisProcess::stop_and_wait_for_the_library_to_close`）
- [`prune-server/src/server.rs:926-934`](file:///home/fy5090/code/singlefs/prune-server/src/server.rs#L926-L934)（`TcpListener::bind`）

#### 根因与触发路径

1. `ServiceInThisProcess::stop` 用于安全停服进程内启动的 prune-server 实例（广泛用于测试套件、基准测试和自证子进程）。
2. 为了打断并退出正阻塞在 `accept()` 上的 `network_accept` 线程，服务需要向自己实际绑定的网络地址发起一次主动连接：
   ```rust
   let mut wake_address = self.bound_network_address;
   if wake_address.ip().is_unspecified() {
       wake_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
   }
   let _ = TcpStream::connect(wake_address);
   ```
3. 当服务端配置在 IPv6 监听地址（例如 `--network-address [::]:0`）时，内核绑定的 `bound_network_address` 为 `SocketAddr::V6`，其 IP 地址为 `::`（IPv6 unspecified）。
4. 在此分支中，`wake_address.ip().is_unspecified()` 同样成立，但代码不分协议族一律强制调用 `wake_address.set_ip(IpAddr::V4(Ipv4Addr::LOCALHOST))`，导致 `SocketAddr::V6` 被错误变更为 IPv4 的 `127.0.0.1:port`！
5. 在纯 IPv6 环境、或者 Linux 系统开启了 `sysctl net.ipv6.bindv6only=1` 的容器环境下，向 `127.0.0.1:port` 发起 TCP 连接会立即遭到内核拒绝并返回 `ECONNREFUSED`（错误被 `let _` 忽略）。
6. 其直接后果是：监听在 IPv6 `[::]:port` 上的 `network_accept` 线程根本没有收到唤醒连接，依然死锁在 `listener.accept()` 中；随后的 `accept_thread.join()` 将被阻塞整整 30 秒，直到 `STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT` 耗尽报错退出。

#### 修复建议与验收

根据当前 `bound_network_address` 实际的协议族匹配对应的本地回环地址：
```rust
let mut wake_address = self.bound_network_address;
if wake_address.ip().is_unspecified() {
    match wake_address {
        SocketAddr::V4(_) => wake_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
        SocketAddr::V6(_) => wake_address.set_ip(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
    }
}
let _ = TcpStream::connect(wake_address);
```

---

### 缺陷四：离线删除超大值（`drop-oversized-values`）未同步清理对应的认领记录，导致服务重启后旧认领幽灵复活并阻断写入

#### 位置
- [`prune-server/src/oversized_values.rs:163-176`](file:///home/fy5090/code/singlefs/prune-server/src/oversized_values.rs#L163-L176)（`drop_rows_then_read_them_back_absent`）
- [`prune-server/src/storage.rs:56-57, 520-530`](file:///home/fy5090/code/singlefs/prune-server/src/storage.rs#L56-L57)（`CLAIM_KEY_TAG` 与 `load_claims`）

#### 根因与触发路径

1. `prune-server drop-oversized-values` 命令用于在服务停机状态下离线扫描并清理 RocksDB 库中早期因未分段写入而导致单帧无法点读回的异常超大行。
2. 在清理批处理逻辑中：
   ```rust
   let mut batch = WriteBatch::default();
   for oversized in oversized_values {
       batch.delete(&oversized.key);
   }
   database.write_opt(batch, &unsynced)?;
   database.flush_wal(true)?;
   ```
   代码仅仅向批处理中添加了数据键本身的删除操作。
3. 在同一份 RocksDB 默认列族中，服务使用 `0xF1` 前缀保存对应键的认领记录（`claim_key = [0xF1, key...]`）。如果某把超大值键在停服前曾被客户端执行过认领，且认领租约较长尚未在时间戳上到期：
   - 离线删除工具将数据行成功删除；
   - 对应的 `0xF1` 认领记录却被完整遗留在 RocksDB 磁盘中；
   - 运维启动服务后，`storage.load_claims`（`storage.rs:520`）重新扫描库中所有 `0xF1` 键，将该未过期的认领记录加载回服务内存中的 `self.claims` 映射中。
4. 此时产生严重的幽灵状态：数据库中已无此数据，但内存认领表依然记录该键被某个死去的客户端持有。其他合法客户端尝试按新的分段格式重新写入该键时，调用 `claim` 会持续返回 `HeldByOther`，导致整条写入链被幽灵认领长时间阻断。

#### 修复建议与验收

在 `drop_rows_then_read_them_back_absent` 中，同步把该键对应的认领键放进删除写批：
```rust
for oversized in oversized_values {
    batch.delete(&oversized.key);
    batch.delete(crate::storage::claim_key(&oversized.key));
}
```

---

### 缺陷五：组提交内存查重对同组重发请求使用后发帧的条数覆盖真实历史生效条数，破坏幂等回执一致性

#### 位置
- [`prune-server/src/committer.rs:374-385`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L374-L385)（`Committer::add_write` 去重分支）
- [`prune-server/src/committer.rs:137`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L137)（`Group::applied_in_group` 定义）

#### 根因与触发路径

1. 在 `Committer::add_write` 中，服务端实现幂等查重机制。对于已经在磁盘库中生效的请求，调用 `self.shared.library.applied_request_entries(...)` 从 `0xF0` 记账键中读出当初落盘的真实条目数 `earlier_entries`；
2. 但如果该请求号是**在当前正在攒的组提交批次中刚刚生效的**：
   ```rust
   let entries_written = u32::try_from(entries.len()).expect("条数来自帧里的 u32 字段");
   if !self.is_enabled(BreakSwitch::DuplicateRequestsReapplied) {
       let request = (client_identity.to_vec(), request_number);
       let earlier = if group.applied_in_group.contains(&request) {
           Ok(Some(entries_written))
       } else {
           self.shared.library.applied_request_entries(client_identity, request_number)
       };
       ...
       queue(
           group,
           job,
           encode_write_acknowledgement(sequence, earlier_entries, true),
           sequence,
           operation_code,
       );
   ```
3. 查看 `Group` 定义：
   `applied_in_group: HashSet<(Vec<u8>, u64)>`
   它仅记录了客户端身份和请求号，**根本没有保存第一次真正生效时的实际条数**！
4. 如果客户端网络抖动产生乱序重发，或重放时仅发送了探测空帧（`entries.len() == 0`）或部分分片帧，`earlier` 直接取了当前后发请求的 `entries_written`，在确认包中返回了错误的 `entries_written`，造成服务端回执与实际持久化数据状态割裂，破坏幂等重放的确定性。

#### 修复建议与验收

将 `Group::applied_in_group` 的类型调整为映射字典：
```rust
applied_in_group: HashMap<(Vec<u8>, u64), u32>
```
在第 448 行记录第一次生效时的实际写入条数；同组去重命中时直接读取该字典中记录的条目数。

---

## 三、 低危与可观测性缺陷 (Low)

### 缺陷六：未计费响应队满强关连接未在故障日志（`fault_log`）留痕，导致慢客户端静默断开且管理侧失察

#### 位置
- [`prune-server/src/connections.rs:165-190`](file:///home/fy5090/code/singlefs/prune-server/src/connections.rs#L165-L190)（`Responder::send` 溢出检查）
- [`prune-server/src/fault_log.rs:20-40`](file:///home/fy5090/code/singlefs/prune-server/src/fault_log.rs#L20-L40)（`FaultKind` 定义）

#### 根因与触发路径

1. `Responder::send` 用于投递忙回执、请求拒绝等不带预留额度的系统级消息帧。为了防止慢消费客户端导致服务端发送缓冲区积压耗尽内存，设有 `UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION`（256 KiB）配额保护。
2. 当积压超限时，代码调用 `(self.account.shut_down)()` 关闭连接，并将 `totals.overflow_closures` 原子加 1。
3. 然而，在此剔除连接的执行路径上，**完全没有向 `shared.fault_log` 写入任何故障日志行**，标准错误也没有输出任何上下文信息（包括连接编号、对端 IP、报到任务号）；且在上层调用方（如 `committer.rs:850` 的 `send_now`），返回的 `Err(WriterGone)` 也被静默忽略。
4. 运维人员与监控脚本（`control watch`）无法在故障日志中感知到慢客户端被踢掉的事件，无法定位被杀连接对应的任务或机器，可观测性缺失。

#### 修复建议与验收

在 `Responder::send` 发生溢出关闭时，通过回调或传递 `fault_log` 写入一行类型为连接异常中断的故障记录，记明该连接积压的未计费字节数与连接身份信息。
