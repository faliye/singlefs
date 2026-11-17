# 2026-11-07 prune-server 代码审核

审核范围覆盖 `prune-server/` 源码：`src/server.rs`（2309 行）、`src/committer.rs`（858 行）、`src/storage.rs`（751 行）、`src/connections.rs`（426 行）、`src/reliable_client.rs`（2117 行）、`src/segmented_values.rs`（1503 行）、`src/protocol.rs`（1563 行）、`src/oversized_values.rs`（310 行）、`src/journal.rs`（440 行）、`src/configuration.rs`（236 行）、`src/administration.rs`（314 行）、`src/fault_log.rs`（207 行）、`src/process_watch.rs`（191 行）、`prune-server/control`（1417 行），以及客户端脚本 `scripts/prune-server/prune_server_client.py`（1512 行）。

早前报告（`records/2026-11-06-prune-server全量代码审核.md`）所列四项历史缺陷（PS-01 认领租期缩短、PS-02 写帧存储出错放弃、PS-03 删前缀未先记配额、PS-04 删前缀未释放认领）均已核对现行代码确认修复落定。以下为本次代码深度审计新查实与确认的六处漏洞与设计不符缺陷。

---

## 汇总

| 编号 | 级别 | 位置 | 问题概要 |
|---|---|---|---|
| 缺陷一 | 高危 | `prune-server/src/committer.rs:303-306, 323-326` | 认领与删前缀无差别检查 `!group.batch.is_empty()` 拆散组提交，导致相同操作内部无法批处理并退化为每帧独立 WAL 同步 |
| 缺陷二 | 高危 | `prune-server/src/committer.rs:795-814`、`server.rs:385-406` | 组提交失败（`fail_group`）替换拒绝帧时未释放原预留配额并重复强记配额，导致连接与全局待处理字节额度虚高与双重记账 |
| 缺陷三 | 中危 | `prune-server/src/server.rs:1169-1173` | `ServiceInThisProcess::stop` 在 IPv6 监听环境下将 unspecified IP 误换为 IPv4 环回地址，导致唤醒连接失败与停机卡死 30 秒 |
| 缺陷四 | 中危 | `prune-server/src/oversized_values.rs:163-176` | 离线删除超大值（`drop-oversized-values`）未同步清理对应的认领记录，导致服务重启后旧认领幽灵复活并阻断写入 |
| 缺陷五 | 中危 | `prune-server/src/committer.rs:374-385, 137` | 组提交内存查重对同组重发请求使用后发帧的条数覆盖真实历史生效条数，破坏幂等回执一致性 |
| 缺陷六 | 低危 | `prune-server/src/connections.rs:165-190` | 未计费响应队满强关连接未在故障日志（`fault_log`）留痕，导致慢客户端静默断开且管理侧失察 |

---

## 缺陷一（高危）：认领与删前缀无差别检查 `!group.batch.is_empty()` 拆散组提交，导致批处理退化为每帧独立 WAL 同步

### 位置

- [`prune-server/src/committer.rs:303-315`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L303-L315)（`Committer::process_job` 认领分支）
- [`prune-server/src/committer.rs:323-334`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L323-L334)（`Committer::process_job` 删前缀分支）

### 根因与触发路径

1. 设计文档（`committer.rs:6-7` 与 `14-16`）明确规定设计契约：
   > “认领：判「已有结论」读库，所以认领帧之前已收进这一组的写先提交（同步、回确认），认领读的是已落盘的库。”
   > “删前缀：同样先把这一组里已收的写提交掉（删的是已落盘的库），再扫。”
   其设计本意极其清晰：只有当前攒的这一组中存在**修改数据表的写批（BatchedWrite）**时，才需要先执行 `commit` 落盘，从而使后续认领帧判“已有结论”或删前缀扫描数据键时能读到最新落盘数据。
2. 然而在现行代码实现中：
   ```rust
   RequestView::Claim { ... } => {
       if !group.batch.is_empty() {
           self.commit(group);
           group = self.new_group();
       }
       self.add_claim(&mut group, job, sequence, client_identity, lease_milliseconds, &keys);
       group
   }
   ```
   代码检查的是 `!group.batch.is_empty()`。而 `add_claim` 在认领成功时会通过 `self.shared.library.add_claim(&mut group.batch, &claim, &record)` 向 `group.batch` 写入认领行！
3. 当连续收到多帧认领请求时（例如客户端并发提交多批判定向量编号登记，或多条连接并发认领）：
   - 处理第 1 帧认领：`group.batch` 初始为空，认领写入加入 `group.batch`，返回非空的 `group`；
   - 处理第 2 帧认领：执行 `if !group.batch.is_empty()` 判定为真，**第 1 帧认领被立刻强制触发 `self.commit(group)` 并调用 `flush_wal(true)` 执行一次磁盘同步！**
   - 处理第 3 帧认领：同样被第 2 帧打断并再次触发一次 `flush_wal(true)`。
4. 删前缀（`DeletePrefix`）分支存在完全相同的缺陷：连续的多帧删前缀之间无法组提交，每一帧都被前一帧的删除写批强制打断，各自进行独立的 WAL 同步。
5. 更严重的是行为分化：若第 1 帧认领中的键由于“已被持有”或“已有结论”导致未写入任何认领行（`group.batch` 为空），则第 2 帧认领可以与第 1 帧合批；而一旦认领成功，后一帧就被强制拆分。这种因业务结局偶发分化的组提交机制使得高并发认领与连续删前缀退化为最慢的串行磁盘同步瓶颈。

### 修复建议

在 `Group` 结构体中增加布尔标志（如 `has_data_modifications: bool`），仅在写入了实际数据行（`BatchedWrite` 或 `DeletePrefix` 删除了数据行）时置为真。在 `Claim` 或 `DeletePrefix` 到达时，改为 `if group.has_data_modifications { self.commit(group); group = self.new_group(); }`。连续的纯认领帧之间共享同一个组提交与单次 WAL 同步。

---

## 缺陷二（高危）：组提交失败（`fail_group`）替换拒绝帧时未释放原预留配额并重复强记配额，导致额度双重记账

### 位置

- [`prune-server/src/committer.rs:795-814`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L795-L814)（`Committer::fail_group`）
- [`prune-server/src/committer.rs:850-853`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L850-L853)（`send_now`）
- [`prune-server/src/server.rs:385-406`](file:///home/fy5090/code/singlefs/prune-server/src/server.rs#L385-L406)（`PendingSlot` 配额伸缩）

### 根因与触发路径

1. 一帧在提交线程排队时，通过 `queue`（写帧强制记账数十字节）或 `queue_reserved`（认领与删前缀提前通过 `job.slot.reserve_response(...)` 预留响应字节）将回应帧与其占用的待处理额度（`queued.slot`）保存在 `QueuedResponse` 中。
2. 当发生写批失败（`GroupCommitFailure::WriteFailed` 或测试注入失败）时，提交线程执行 `fail_group`：
   ```rust
   for queued in responses {
       let rejection = encode_rejection(
           queued.sequence,
           ResponseStatus::StorageFailed,
           queued.operation_code,
           message,
       );
       send_now(queued.responder, rejection, queued.slot);
   }
   ```
3. 在 `send_now` 中：
   ```rust
   fn send_now(responder: Responder, response: Vec<u8>, mut slot: PendingSlot) {
       slot.force_response(response.len());
       send_reserved(responder, response, slot);
   }
   ```
   `send_now` 无条件调用 `slot.force_response(rejection.len())`，将拒绝消息字节（通常几百字节）强制追加到 `slot` 内部的 `ByteReservation` 中。
4. **致命漏洞在于：原本存放在 `queued.slot` 中预先划给正常回应的字节数（认领与删前缀帧可达几万至数十万字节）完全没有调用 `slot.release_response(...)` 予以释放！**
5. `queued.response` 正常帧被就地 drop，但其对应的内存记账依然锁在 `slot` 中。拒绝帧又强行追加了一笔新的记账，导致 `slot.bytes` 变成了 `原请求体 + 原预留响应体 + 新错误拒绝响应体`。
6. 在写线程将拒绝帧写出套接字之前，`admission.pending_bytes` 与连接的 `budget.held_bytes` 发生严重双重记账与额度虚高，极易触发后续请求误报 `Busy` 拒绝。

### 修复建议

在 `fail_group` 替换为 `rejection` 之前，或在 `send_now` 接手已预留 slot 时，先按 `queued.response.len()` 调用 `slot.release_response(queued.response.len())` 归还原响应预留额度，再调用 `slot.force_response(rejection.len())` 记入拒绝帧字节。

---

## 缺陷三（中危）：`ServiceInThisProcess::stop` 在 IPv6 监听环境下将 unspecified IP 误换为 IPv4 环回地址导致唤醒失败与停机卡死 30 秒

### 位置

- [`prune-server/src/server.rs:1164-1183`](file:///home/fy5090/code/singlefs/prune-server/src/server.rs#L1164-L1183)（`ServiceInThisProcess::stop_and_wait_for_the_library_to_close`）

### 根因与触发路径

1. `ServiceInThisProcess::stop` 用于停掉进程内启动的 prune-server 服务（测试实例或内嵌服务）。为了唤醒正在阻塞等待客户端连接的 `TcpListener::accept` 线程，代码需要发起一次主动连接：
   ```rust
   let mut wake_address = self.bound_network_address;
   if wake_address.ip().is_unspecified() {
       wake_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
   }
   let _ = TcpStream::connect(wake_address);
   ```
2. 当服务端配置并绑定在 IPv6 未指定地址（如 `[::]:port`）时，`wake_address` 的类型为 `SocketAddr::V6`，且 `wake_address.ip().is_unspecified()` 同样判定为 `true`。
3. 代码无条件执行 `wake_address.set_ip(IpAddr::V4(Ipv4Addr::LOCALHOST))`，将目标地址强制改成 IPv4 回环地址 `127.0.0.1:port`。
4. 在纯 IPv6 环境，或内核开启了 `net.ipv6.bindv6only=1`（禁用 IPv4-mapped IPv6 双栈回退）的生产容器中，向 `127.0.0.1:port` 发起连接直接返回 `ECONNREFUSED`（错误被 `let _` 静默忽略）。
5. 阻塞在 IPv6 `[::]:port` 上的 `network_accept` 线程从未收到唤醒连接，持续阻塞在 `listener.accept()`。随后执行的 `accept_thread.join()` 陷入永久阻塞，直至触发 `STOP_OF_A_SERVICE_IN_THIS_PROCESS_TIMEOUT`（30 秒）强行超时报错。

### 修复建议

根据 `bound_network_address` 的 IP 协议族分别选择回环唤醒地址：
```rust
if wake_address.ip().is_unspecified() {
    match wake_address.ip() {
        std::net::IpAddr::V4(_) => wake_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
        std::net::IpAddr::V6(_) => wake_address.set_ip(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
    }
}
```

---

## 缺陷四（中危）：离线删除超大值（`drop-oversized-values`）未同步清理认领记录，导致服务重启后旧认领幽灵复活

### 位置

- [`prune-server/src/oversized_values.rs:163-176`](file:///home/fy5090/code/singlefs/prune-server/src/oversized_values.rs#L163-L176)（`drop_rows_then_read_them_back_absent`）

### 根因与触发路径

1. `prune-server drop-oversized-values` 在服务停机时离线扫描 RocksDB 库，把值过长以致单键点读帧无法装下的旧数据行直接删除。
2. 在执行写批删除时：
   ```rust
   let mut batch = WriteBatch::default();
   for oversized in oversized_values {
       batch.delete(&oversized.key);
   }
   database.write_opt(batch, &unsynced)?;
   ```
   代码仅删除了数据行本身。
3. 系统的认领记录（Claim）同样保存在 RocksDB 的默认列族中，键前缀为 `0xF1`（即 `claim_key = [0xF1, key...]`）。若某把超大键在停机前曾被认领且租期未到期，该工具删除了数据键，却将对应的 `0xF1` 认领记录遗留在库中。
4. 服务重新拉起时，`load_claims` 从 RocksDB 中扫描全部 `0xF1` 认领键并加载到内存 `self.claims` 中。此时数据虽然已被离线删除，但旧的认领记录仍然有效。新的客户端尝试重新写入该键时，认领请求将被幽灵认领判定为“别人持有（HeldByOther）”，造成写入被不合理长期阻断。

### 修复建议

在 `drop_rows_then_read_them_back_absent` 中，删除 `oversized.key` 的同时，调用 `crate::storage::claim_key(&oversized.key)` 将对应的认领行一并加入写批删除：
```rust
batch.delete(&oversized.key);
batch.delete(crate::storage::claim_key(&oversized.key));
```

---

## 缺陷五（中危）：组提交内存查重对同组重发请求使用后发帧的条数覆盖真实历史生效条数

### 位置

- [`prune-server/src/committer.rs:374-385`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L374-L385)（`Committer::add_write` 去重分支）
- [`prune-server/src/committer.rs:137`](file:///home/fy5090/code/singlefs/prune-server/src/committer.rs#L137)（`Group::applied_in_group` 定义）

### 根因与触发路径

1. 在 `Committer::add_write` 中，服务对已生效请求号进行查重。当请求已在库中生效时，`applied_request_entries` 从记账键中读出当初落盘的真实条目数 `earlier_entries`；
2. 但如果该请求号是**在当前组提交批次中刚刚生效的**：
   ```rust
   let entries_written = u32::try_from(entries.len()).expect("条数来自帧里的 u32 字段");
   let earlier = if group.applied_in_group.contains(&request) {
       Ok(Some(entries_written))
   } else {
       self.shared.library.applied_request_entries(client_identity, request_number)
   };
   ```
   `Group::applied_in_group` 的类型被定义为 `HashSet<(Vec<u8>, u64)>`，只存了请求元组，没有存第一次生效时的条数！
3. 当客户端因网络抖动重发请求，且重发帧中的条目列表异常（例如重放了空 entries 探针或部分重试帧）时，`earlier` 取到的是后发请求自身的 `entries_written`，并将其封装在回执 `encode_write_acknowledgement(sequence, earlier_entries, true)` 中返回。
4. 这导致回执中声明的已生效条数与真正持久化写入的条目数不一致，违反了协议幂等去重契约。

### 修复建议

将 `Group::applied_in_group` 改为 `HashMap<(Vec<u8>, u64), u32>`，在第 448 行记录第一次生效时的实际写入条数；同组查重命中时直接取该映射中的历史条数。

---

## 缺陷六（低危）：未计费响应队满强关连接未在故障日志留痕，导致慢客户端静默断开且管理侧失察

### 位置

- [`prune-server/src/connections.rs:165-190`](file:///home/fy5090/code/singlefs/prune-server/src/connections.rs#L165-L190)（`Responder::send` 溢出检查）

### 根因与触发路径

1. 未计费响应通道（`Responder::send`）用于发送忙回执、协议拒绝、存储出错等不带预留配额的响应帧。为了防止慢客户端导致服务端内存无界膨胀，设有 `UNBUDGETED_OUTBOUND_BYTES_PER_CONNECTION`（256 KiB）上限。
2. 当积压字节超限时，代码调用 `(self.account.shut_down)()` 关闭套接字两个方向，并将计数器 `totals.overflow_closures` 加 1。
3. 整个过程**没有在 `shared.fault_log` 中记录任何故障行（`FaultKind`）**，标准错误也没有输出任何上下文信息（包括连接编号、对端 IP、报到任务号）；且在上层调用方（如 `committer.rs:850` 的 `send_now`），返回的 `Err(WriterGone)` 也被静默忽略。
4. 运维人员与监控脚本（`control watch`）无法在故障日志中感知到慢客户端被剔除的事件，无法定位被杀连接对应的任务或机器。

### 修复建议

在 `Responder::send` 发生溢出关闭时，向 `FaultLog` 写入一行类型为连接异常中断的故障记录，记明该连接积压的未计费字节数与连接身份信息。
