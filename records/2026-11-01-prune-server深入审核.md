# prune-server 深入审核（2026-11-01）

## 范围、基线与结论

审核时 HEAD 为 `59ce5a86550de2c2c9bc6b51b34b4f881313682c`，prune-server 为工作树中的新增工程；结论针对实际文件内容，不以 HEAD 已包含这些文件为前提。本次只增加审核记录，未修改服务或客户端实现。

重点检查 server、connections、committer、storage、protocol、journal、client、reliable_client 的准入、持久化、去重、日志恢复、连接生命周期；辅查迁移、配置及既有测试。没有声称逐行穷尽整个工程。

确认 **7 项问题，P1 5 项、P2 2 项**。最重要的两项不是性能问题：日志整理后重启会让新写入被误去重；WAL 同步失败后重复请求可获得未经成功同步的确认。

| 编号 | 优先级 | 问题 | 证据类型 |
| --- | --- | --- | --- |
| PS-01 | P1 | journal 整理丢失请求号高水位，新写入被当作重复请求 | 当前源码直接编译 + 临时服务端到端 |
| PS-02 | P1 | WAL 同步失败后去重路径仍回成功 | 独立服务进程 LD_PRELOAD 同步失败注入 |
| PS-03 | P1 | 无界出站队列绕过准入额度 | 不读响应客户端、RSS 实测 |
| PS-04 | P1 | 帧体收取发生在准入前，未完成帧绕过内存限制 | 半帧连接、RSS 实测 |
| PS-05 | P1 | journal rename 与 open/flock 竞争，写入旧 inode 后丢失 | 当前 journal 源码、flock 前屏障 |
| PS-06 | P2 | 认领响应可超出 64 MiB 协议上限 | 当前协议源码直接编译 + 网络请求 |
| PS-07 | P2 | 阻塞 connect 不受重连总时限控制 | 当前客户端源码 + 填满 Unix backlog |

## PS-01：整理日志后复用旧请求号，静默丢掉新写入

**位置：** `journal.rs:178-204`、`243`（恢复请求号），`293-337`（compact），`reliable_client.rs:492`、`593-595`（初始化和分配请求号），`committer.rs:319-345`（去重）。

恢复时 `highest_request_number` 仅从 WRITE_RECORD 推导。compact 只写客户端身份和仍未确认的写帧，完全删除已确认请求的高水位；ACK_RECORD 也不推进恢复高水位。客户端身份在整理后保留。

当全部写帧确认并触发整理时，日志只剩身份。重启恢复出的 next_request_number=1；服务端的同一身份/request=1 已存在，因而将后续新写入当作重复请求返回确认，根本不写新键。

端到端步骤：

1. 临时服务、临时 journal；将整理阈值设为 1 字节以快速覆盖正常 compact 分支，所有 break switch 关闭。
2. 客户端写 `first=one`，收到确认后整理并关闭。
3. 同 journal 重开客户端，写 `second=two`，等待全部写确认，再读 second。

输出：

```text
first_request=1 compactions=1
second_request=1 acknowledged=true value=[None]
```

阈值只是缩短复现时间，默认超过 64 MiB 时走同一逻辑。即使整理时还有未确认低编号请求，也可能丢失已确认的更高编号，而不只是“全部确认”的情况。

**修复要求：**在日志中显式持久化请求号高水位，并在所有整理结果中保留；恢复必须取高水位与所有记录编号的最大值。只从未确认记录推导不够；简单保留最近 ACK 也需处理乱序确认。旧版已整理且无高水位的日志无法仅靠文件重建正确编号，需要明确迁移策略，不能假设 1 安全。

**回归验收：**完整确认后 compact/reopen 再写新键，以及“低号未确认、高号已确认”后 compact/reopen，两者都必须分配从未使用过的请求号，并确认新键真正存在。

## PS-02：失败的 WAL 同步之后，重复请求绕过同步仍获成功

**位置：** `storage.rs:370-387`（commit_group/同步），`committer.rs:319-345`（去重）、`500-515`（commit）、`519-546`（fail_group）。

组提交先 `write_opt(sync=false)`，再 `flush_wal(true)`。第一次写已经让数据和 applied-request 标记在进程内可见；如果随后同步失败，fail_group 返回 StorageFailed，但没有把该请求标记为“未确认持久化”，也没有停止服务。

客户端重新发送同一身份/请求号，`applied_request_entries` 查到标记，进入 duplicate 分支；这一组没有新写 batch，`commit_group` 直接返回 `Ok(None)`，无需再次同步，仍发成功确认。

注入方法：独立进程的 LD_PRELOAD 包装 fsync/fdatasync；只针对临时 `/db/*.log` 文件，在 arm 文件存在时持续返回 EIO，初始化完成后才 arm。未向真实服务注入故障。

实测：

```text
first_write: status=4, operation=2, 同步 WAL 失败 ... Input/output error
retry_same_write: status=0, operation=2, entries_written=1, is_already_applied=1
AUDIT failed sync /tmp/pr-sync-.../db/000004.log
```

arm 仍在，同步仍会失败，但第二次成功路径没有调用同步。**没有声称通过 kill -9 复现断电丢失**：页缓存存活会保留 write；这里验证的是“成功确认前必须已成功持久化”的契约被破坏，真实断电下存在数据丢失风险。

**修复要求：**区分进程内已应用与已确认持久化；同步错误后进入故障状态，禁止将不确定请求直接确认为成功。选择重试同步或终止服务重新恢复都需明确状态规则；不能仅重载 claims 便继续确认写请求。

**回归验收：**write 成功、flush 失败，保持存储同步故障后重发相同请求；不能回 Completed。恢复同步后才可确认，且仍必须幂等。

## PS-03：响应已入队即释放额度，慢客户端造成无界内存积压

**位置：** `connections.rs:30-37`（mpsc::channel/send），`server.rs:583-587`（工作线程释放 slot）、`896-921`（写线程），`committer.rs:561-563`。

准入额度只覆盖计算中的请求。响应进入无界 mpsc 队列后 slot 就被释放，实际 socket 可能仍堵在 write_all。客户端继续发送请求但不读响应时，服务继续生成并积累大响应；busy 拒绝帧同样走无界队列，不能依赖 pending limit 兜底。

独立服务设置全局及每连接 maximum-pending=1，先写一个 1 MiB 值。新连接发送 64 次点读，间隔 15 ms，完全不读响应：

```text
rss before unread reads:  84768 kB
rss after 64 unread reads: 152416 kB
pending total limit=1
```

内存增加约 66 MiB，与未发送完的大响应数量对应。没有把这次有限请求测试直接称为实际 OOM；无界性由当前源码的无界队列与提前释放额度共同证明。

**修复要求：**按出站字节限制响应队列，额度持有至实际发送/丢弃；限制慢写连接的存活时间。避免直接在全局 committer 内对满队列阻塞，否则单个慢客户端会阻塞所有写提交。忙响应及管理响应也要有明确上限/关闭策略。

**回归验收：**小请求读取大值、客户端只发不收；服务内存必须有可解释的上界，且其他客户端仍能读写。

## PS-04：在准入前分配并阻塞读取整个帧体

**位置：** `server.rs:650-676`（read_request_frame），`803-842`（先读取再 admit），`607/634/796`（每连接线程）。

收到合法长度后立即 `vec![0; declared_body_bytes]` 并 read_exact，完整接收后才进入 admission。socket 没有服务端读超时，连接数也没有此处的统一准入。多个只发送部分帧体的连接可以无限期占用这些不计账的 buffer 和线程。

实测将 `maximum-pending-bytes=1024`、`maximum-pending-total=1`，开 4 个连接，各声明 16 MiB 帧体、只发送 2 MiB 内容并保持连接：

```text
before incomplete frames: VmRSS 17296 kB
after 4 incomplete frames: VmRSS 29716 kB
pending_bytes_limit=1024
```

Linux 零页/延迟提交使声明的 64 MiB 不会立即全部计入 RSS，所以本记录不将声明字节数当成实测物理内存。已发送部分实际占用了远超 1 KiB 限制的内存，未完成请求无法进入正常 busy 处理。

**修复要求：**解析长度后、分配帧体前预留接收额度；总接收额度、连接数、帧读取期限都需受控。拿不到接收额度时必须有可维持帧边界的策略，不能为“先收完再拒绝”继续保留大 buffer。

**回归验收：**大量合法长度但不完成的帧、慢速逐字节帧，在请求处理额度为极小值时也不能绕过全局内存预算。

## PS-05：journal 原子替换破坏基于文件 inode 的排他锁

**位置：** `journal.rs:129-137`（open 后 flock），`298-309`（replacement 锁），`325-333`（rename 与替换 self.file）。

flock 锁的是打开的 inode，路径会在 compact 时换到另一个 inode。虽然 replacement 在 rename 前已加锁，但无法保护已经打开旧路径、尚未来得及 flock 的竞争者。

确定性交错：

1. A 持有旧 journal 的锁。
2. B 执行 open(path) 取得旧 inode 的 fd，在 flock 前暂停。
3. A compact，锁住新 inode，rename 到 path，替换 self.file，释放旧 inode 锁。
4. B 继续 flock(old_fd) 成功，读写已不再由 path 指向的旧 inode。

用当前 journal.rs 原样编译，只在真实 libc flock 调用前给 B 加屏障。B 在 A 仍持有新文件时成功打开并追加 request=2，双方关闭后从原路径恢复：

```text
second_open_and_append_succeeded_while_first_still_open=true
recovered_pending=0 (expected second write)
```

这不是仅有“双开锁没挡住”的形式问题：B 已成功 append 的未确认写不在路径指向的日志中，断线/崩溃后无法重放。前提是有第二个客户端竞争同一日志；API 明确承诺这种误用应被排他锁拒绝。

**修复要求：**使用不参与 rename 的独立 lock 文件，并在持有该锁后打开/整理数据文件；或实现带 inode 核对与重试的等价协议。只给 replacement 加 flock 已经存在，不能解决此交错。

**回归验收：**在第二个 opener 的 open 与 flock 之间阻塞，第一个客户端 compact 后再放行；第二个客户端必须被拒绝或在正确的当前文件上受同一把排他锁保护，不能成功写孤立 inode。

## PS-06：合法认领请求产生超长响应，服务与客户端协议自相矛盾

**位置：** `committer.rs:415-457`（outcomes 构造），`protocol.rs:998-1029`（encode_claim_outcomes），`client.rs:324`、`reliable_client.rs:1119`（响应长度拒绝）。

每个 HeldByOther 结果都携带 holder_identity，而请求只携带短键。认领编码没有像点读响应一样核算 64 MiB 上限。

复现：身份长度 255 的 A 认领键 k，B 在同一个合法请求中查询 260,000 次 k。请求体 1,560,019 字节，服务响应体 68,900,014 字节，大于 67,108,864 字节上限。

```text
claim request body bytes=1560019 response body bytes=68900014 protocol max=67108864
current_source_response_bytes=68900014 accepted_by_client=false
```

后一行直接编译当前 protocol.rs 并调用编码器和客户端共用长度检查，不依赖服务二进制的新旧。重复键当前协议未禁止，服务也接受。大批互不相同的短键同样存在响应膨胀，只是构造成本较高。

**修复要求：**认领结果构造前/期间执行响应字节预算，超限时给合法拒绝响应；明确多键认领在拒绝时是否允许部分副作用，不能只在所有认领已经写进 batch 后丢弃响应。若限制键数，需要按最大身份长度计算。

**回归验收：**长 holder、短键、大批量 HeldByOther 和混合 Claimed/HeldByOther，任何响应都不能越过帧长上限。

## PS-07：连接建立不受 maximum_total 和 write_timeout 控制

**位置：** `client.rs:53-70`（阻塞 connect 及报到），`reliable_client.rs:400-412`（连接后才设写超时），`806-869`（重连循环）。

重连总时限只在两次尝试之间检查，`open_announced_stream` 内部使用阻塞 UnixStream::connect/TcpStream::connect；报到 write_all 也发生在 reliable_client 设置 write_timeout 之前。因此一次连接尝试就可能长时间阻塞，stop_check 和重连总时限无法介入。TCP keepalive 在连接完成后设置，不覆盖建立连接阶段。

验证直接编译当前客户端相关模块，在临时 Unix socket 上 listen(1) 而不 accept，先用两个连接占满队列。设置 `maximum_total=50ms`、`write_timeout=50ms`，调用 ReliableClient::open：

```text
open still blocked after 1000ms despite maximum_total=50ms
```

外部测试超时终止了该测试客户端；未让它永久挂住。此复现不需要修改防火墙、不需要依赖外部黑洞地址。

**修复要求：**连接、地址解析、报到都纳入同一个截止时间；TCP 用有期限的 connect 或非阻塞连接，Unix socket 也要有可中断的非阻塞连接策略；报到前设置写期限。每次阻塞操作应使用剩余预算而非重新给完整预算。

**回归验收：**Unix backlog 满、TCP 建连迟迟不完成、连接后不读报到，配置的总时限均能生效，并能响应 stop_check。

## 验证覆盖、构建限制与未扩大结论

既有测试运行：

```sh
PRUNE_SERVER_SELFTEST_CELLS=busy-backpressure-bounded,claim-exclusive-and-lease-expiry \
  prune-server/target/release/prune-server --selftest
```

结果 `cells=2 passed=2 failed=0`。这只覆盖正常背压/租约互斥，未覆盖无接收客户端、半帧、日志高水位、同步失败后重复请求。

网络集成测试使用已有 `target/release/prune-server`（时间戳当天），未重新编译整个 RocksDB 服务。源码 connections.rs 时间戳晚于这个二进制；因此服务动态测试只作为所列代码路径的佐证，不宣称全工作树的新构建回归。无界队列、slot 释放、分配前准入缺失等同时由当前源码确认。

客户端、journal、protocol 的关键验证直接用 rustc 编译当前源码：客户端验证只替换连接登记时间的 `committer::now_milliseconds` 为固定值；journal 高水位单测只固定随机身份生成；锁竞争验证只在真实 flock 前加入调度屏障。这些替代不改变被验证的日志、请求号、连接或协议逻辑。PS-02 的注入针对真实二进制/真实 RocksDB 的 WAL 同步系统调用，保持注入开关后才重发请求。

所有服务均使用 TemporaryDirectory、临时 Unix socket、TCP 127.0.0.1:0 和临时库，结束后关闭测试进程。没有访问现网判定库，没有做真实机器掉电、全盘耗尽或整套自测。

以下观察未单独计数：服务 TCP 无认证是 README 已明示的部署边界；同一身份/请求号携带不同内容的客户端违规重用本身不算服务器漏洞，但 PS-01 证明官方客户端会自行制造这种重用；日志不 fsync 的客户端整机掉电语义已在 journal 文件头声明，不能把它混作 PS-02 的服务端持久化确认问题。

## 修复顺序与基线指纹

先修 PS-01/02，阻止“写入成功但数据未生效/未成功持久化”；并行设计 PS-05 的稳定日志锁。再分别限制 PS-03 出站内存、PS-04 接收内存，最后补 PS-06 响应预算和 PS-07 建连期限。入站与出站是两种不同的未计账生命周期，只修一个不能解决另一个。

```text
journal.rs         56455f6a11041b0d0e0cc21f7ece98a9673161ca46838951e05f315a458d1816
committer.rs       e02a6c6e6b34ec4ad1a7a76f84231bdb414f3732429ced3f42c2d2d0609e4f62
storage.rs         76e8ce828ddd5e06028dd818c2ce0ac62a2128898a9a6ce66ea2097a1e309aad
connections.rs     31f746ed6d99c846b8b269a12a1cc0c85bbf881e2646e272327d29c016056681
server.rs          283399bc406d09d2c32b4a56df490f7489f96de1b1f7ef01cad11f075055ec2a
protocol.rs        7a719dee2adff457e560e8d67d9ed6fcdefc299ddb08526aa79ae6cd481deb28
client.rs          fb34bf9ecd61c1f23ad8dcb73ffd28410d6a7a8bb7f49e5eae345b040826ee1e
reliable_client.rs afc6b1e1dc6f76a7fa5f2a43cbad682b54edc8c924a223844141731f2e5faa5f
```

所有条目在本次审核结束时均未修复。后续复核应记录逐项修复提交和新回归结果，不把本记录中的复现结果当作修复后的结论。

## 逐项复核结果（同日追加）

用户要求完成后验证，本轮先验证预编 RocksDB 的环境，再从当前工作树重新构建服务，随后从本文提取并重放全部复现。**7 项均再次复现，未撤销条目，仍未修复。** 前文首轮使用旧二进制的限制现已解除；这里保留首轮说明作为历史，不再把它当作最新验证状态。

构建命令与结果：

```sh
python3 scripts/hosts/rocksdb-prebuilt.py entry
cargo build --release --offline --locked --manifest-path prune-server/Cargo.toml --bin prune-server -j 2
```

预编库入口检查通过；Cargo 重新编译 prune-server，用时 7.46 秒。当前源码重新构建出的服务 SHA-256：

```text
b9984e24f631882b44c9a56be33531a1d260f2434351ed89648e6e4f90829cc5
```

所有服务端集成复现均改用该产物；客户端/协议验证也重新编译了本文所含的当前源码 harness。

| 编号 | 本轮结果 | 判定 |
| --- | --- | --- |
| PS-01 | 首次 request=1/compactions=1；重开后 request=1/acknowledged=true，读 second 为 None | 新构建上确认静默跳过新写入 |
| PS-02 | 首次返回 status=4 WAL 同步 EIO；保持注入后重发返回 status=0/is_already_applied=1 | 确认同步故障未解除仍能获得成功确认；未声称做过真实断电测试 |
| PS-03 | pending limit=1，不读 64 个大响应，RSS 85,732 → 153,352 KiB | 确认响应积压不受该额度约束 |
| PS-04 | pending bytes=1,024，4 条半帧连接，RSS 17,888 → 29,504 KiB | 确认接收阶段的实际内存绕过准入 |
| PS-05 | A 仍开着日志时 B 成功打开并追加；重开路径 recovered_pending=0 | 当前 journal 源码再次确认旧 inode 写入遗失 |
| PS-06 | 合法请求得到 68,900,014 字节响应；当前协议长度检查 false | 服务端与客户端两侧确认 |
| PS-07 | maximum_total=50ms、write_timeout=50ms，open 在 1,000ms 外部期限到达时仍未返回 | 当前客户端源码再次确认建连阻塞绕过期限 |

PS-01 另做反向对照：仅将测试配置中的 journal_compaction_bytes 从 1 改为 `u64::MAX`，同样先写、关闭、重开、再写，不修改任何生产源码：

```text
first_request=1 compactions=0
second_request=2 acknowledged=true value=[Some([116, 119, 111])]
```

`[116,119,111]` 即 `two`。因此不是测试键构造、读取顺序或服务未写成功造成的假象，触发点确实是整理后高水位丢失。这个配置只是实验对照，**不是修复建议**：永久关闭整理会导致日志无限增长。

本轮相关既有测试：

```sh
PRUNE_SERVER_SELFTEST_CELLS=busy-backpressure-bounded,claim-exclusive-and-lease-expiry,group-commit-coalesces-syncs,replay-after-disconnect-and-server-kill \
  prune-server/target/release/prune-server --selftest
```

结果 `cells=4 passed=4 failed=0`。组提交、正常互斥、常规背压与断线/进程退出重放仍正常，但没有覆盖“已确认日志 compact 后重开产生新写”“同步失败后 duplicate 确认”“半帧/不读响应”等触发条件。未执行全量自测，不把四项通过误记为缺陷已经消除。

## 附录：可重放的审计脚本快照

以下是实际使用的最小复现源码，保存于本记录中，不依赖审计会话的 `/tmp` 文件继续存在。在仓根运行下面的提取命令，会将快照重建到 `/tmp/singlefs-*`，并将审计时的绝对仓路径替换成当前仓根。不会安装服务，也不会修改业务源码。复现成功的含义是观察到缺陷，修复后断言或预期输出应当改变。

```sh
python3 - <<'PY'
from pathlib import Path
import re
report = Path('records/2026-11-01-prune-server深入审核.md').read_text()
for name, code in re.findall(r'### 复现文件：`([^`]+)`\n\n```[^\n]*\n(.*?)\n```', report, re.S):
    code = code.replace('/home/fy5090/code/singlefs', str(Path.cwd()))
    Path(name).write_text(code + '\n')
    print(name)
PY
```

运行（要求已有可运行的 `prune-server/target/release/prune-server`、兼容当前 rustc 的 libc rlib，以及 C 编译器；若缺少，先按工程构建流程生成，不要用其他机器的旧二进制替代后宣称当前源码测试通过）：

```sh
audit_libc=$(rg --files --hidden --no-ignore prune-server/target/release/deps | rg '/liblibc-[^/]+\.rlib$' | head -n 1)
rustc --edition=2021 /tmp/singlefs-client-review.rs --extern "libc=$audit_libc" -L dependency=prune-server/target/release/deps -o /tmp/singlefs-client-review
rustc --edition=2021 /tmp/singlefs-journal-lock-review.rs -o /tmp/singlefs-journal-lock-review
rustc --edition=2021 /tmp/singlefs-claim-review.rs -o /tmp/singlefs-claim-review
cc -shared -fPIC /tmp/singlefs-review-sync.c -ldl -o /tmp/singlefs-review-sync.so
python3 -B /tmp/singlefs-prune-review.py
python3 -B /tmp/singlefs-prune-deep-review.py
python3 -B /tmp/singlefs-prune-sync-review.py
/tmp/singlefs-claim-review
python3 - <<'PY'
import tempfile, subprocess
with tempfile.TemporaryDirectory() as directory:
    subprocess.run(['/tmp/singlefs-journal-lock-review', directory + '/journal'], check=True)
PY
```

`prune-review.py` 覆盖 PS-03/06；`prune-deep-review.py` 覆盖 PS-01/04/07；`prune-sync-review.py` 覆盖 PS-02；journal-lock 单独覆盖 PS-05。RSS 数字受运行环境影响，不要求复现出完全相同的数字。编译客户端时唯一替代的服务模块是连接登记时间函数；未替代 journal 或可靠客户端实现。

### 复现文件：`/tmp/singlefs-prune-review.py`

```python
import tempfile,subprocess,socket,struct,time,pathlib
BIN='/home/fy5090/code/singlefs/prune-server/target/release/prune-server'
def u32(n):return struct.pack('<I',n)
def u64(n):return struct.pack('<Q',n)
def lp(b):return u32(len(b))+b
def send(s,seq,op,b):
    b=u64(seq)+bytes([op])+b;s.sendall(lp(b))
def exact(s,n):
    b=bytearray()
    while len(b)<n:
        p=s.recv(n-len(b))
        if not p:raise RuntimeError('EOF')
        b.extend(p)
    return bytes(b)
def recv(s):return exact(s,struct.unpack('<I',exact(s,4))[0])
with tempfile.TemporaryDirectory(prefix='pr-review-') as d:
    p=subprocess.Popen([BIN,'serve','--selftest-instance','--socket',d+'/run/s','--state-directory',d+'/state','--library',d+'/db','--tcp','127.0.0.1:0','--worker-threads','1','--maximum-pending-per-connection','1','--maximum-pending-total','1','--maximum-pending-bytes','16777216'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
    sockets=[]
    try:
        print(p.stdout.readline().strip())
        def conn():
            s=socket.socket(socket.AF_UNIX);s.settimeout(10);s.connect(d+'/run/s');sockets.append(s);return s
        s=conn()
        # Valid 255-byte claimant identity, then repeated small key queried by another identity.
        key=bytes([1])+lp(b'k');holder=b'a'*255
        send(s,1,5,bytes([255])+holder+u32(60000)+u32(1)+key);print('claim status=',recv(s)[8])
        count=260000
        send(s,2,5,b'\x01b'+u32(60000)+u32(count)+key*count)
        n=struct.unpack('<I',exact(s,4))[0]
        print('claim request body bytes=',9+2+4+4+len(key)*count,'response body bytes=',n,'protocol max=',64*1024*1024)
        exact(s,n)
        # Keep responses unread: response queue is outside admission accounting.
        value=b'z'*(1024*1024)
        send(s,3,2,b'\x01c'+u64(1)+u32(1)+bytes([1])+lp(b'v')+lp(value));print('write status=',recv(s)[8])
        slow=conn()
        def rss():
            return next(x for x in pathlib.Path(f'/proc/{p.pid}/status').read_text().splitlines() if x.startswith('VmRSS:'))
        print('rss before unread reads=',rss())
        for i in range(64):
            send(slow,100+i,1,u32(1)+bytes([1])+lp(b'v'))
            time.sleep(.015)
        time.sleep(.3)
        print('rss after 64 unread reads=',rss(),'pending total limit=1')
    finally:
        for s in sockets:s.close()
        p.terminate()
        try:p.wait(timeout=5)
        except subprocess.TimeoutExpired:p.kill();p.wait()
```

### 复现文件：`/tmp/singlefs-claim-review.rs`

```rust
#[allow(dead_code)]
#[path="/home/fy5090/code/singlefs/prune-server/src/protocol.rs"]
mod protocol;
fn main() {
    let outcomes=vec![protocol::ClaimOutcome::HeldByOther {holder_identity:vec![b'a';255],remaining_lease_milliseconds:60000};260000];
    let frame=protocol::encode_claim_outcomes(1,&outcomes);
    let declared=u32::from_le_bytes(frame[..4].try_into().unwrap());
    println!("current_source_response_bytes={declared} accepted_by_client={}",protocol::is_frame_length_acceptable(declared,protocol::RESPONSE_HEADER_BYTES));
    assert!(!protocol::is_frame_length_acceptable(declared,protocol::RESPONSE_HEADER_BYTES));
}
```

### 复现文件：`/tmp/singlefs-client-review.rs`

```rust
#![allow(dead_code)]
#[path="/home/fy5090/code/singlefs/prune-server/src/protocol.rs"] mod protocol;
#[path="/home/fy5090/code/singlefs/prune-server/src/break_switches.rs"] mod break_switches;
#[path="/home/fy5090/code/singlefs/prune-server/src/connections.rs"] mod connections;
#[path="/home/fy5090/code/singlefs/prune-server/src/client.rs"] mod client;
#[path="/home/fy5090/code/singlefs/prune-server/src/keepalive.rs"] mod keepalive;
#[path="/home/fy5090/code/singlefs/prune-server/src/journal.rs"] mod journal;
#[path="/home/fy5090/code/singlefs/prune-server/src/reliable_client.rs"] mod reliable_client;
mod committer {pub fn now_milliseconds()->u64 {0}}

use reliable_client::{ReliableClient,ReliableClientSettings};
use client::RouteAddress;use protocol::Table;
fn main(){
 let args:Vec<_>=std::env::args().collect();
 let mut settings=ReliableClientSettings::new(RouteAddress::UnixSocket(args[2].clone().into()),args[3].clone().into());
 settings.journal_compaction_bytes=1;
 settings.reconnect.maximum_total=std::time::Duration::from_millis(50);
 settings.write_timeout=std::time::Duration::from_millis(50);
 if args[1]=="open" {println!("open_returned={}",ReliableClient::open(settings).is_ok());return;}
 let mut first=ReliableClient::open(settings.clone()).unwrap();
 let (_,n)=first.submit_write_entries(&[(Table::Conclusions,b"first".to_vec(),b"one".to_vec())]).unwrap();
 first.wait_all_acknowledged().unwrap();println!("first_request={n} compactions={}",first.journal_statistics().compactions);drop(first);
 let mut second=ReliableClient::open(settings).unwrap();
 let (_,n)=second.submit_write_entries(&[(Table::Conclusions,b"second".to_vec(),b"two".to_vec())]).unwrap();
 second.wait_all_acknowledged().unwrap();println!("second_request={n} acknowledged=true value={:?}",second.read(&[(Table::Conclusions,b"second".to_vec())]).unwrap());
}
```

### 复现文件：`/tmp/singlefs-prune-deep-review.py`

```python
import socket,tempfile,subprocess,time,struct,pathlib
with tempfile.TemporaryDirectory(prefix='pr-deep-') as d:
 p=subprocess.Popen(['/home/fy5090/code/singlefs/prune-server/target/release/prune-server','serve','--selftest-instance','--socket',d+'/r/s','--state-directory',d+'/state','--library',d+'/db','--tcp','127.0.0.1:0','--worker-threads','1','--maximum-pending-total','1','--maximum-pending-bytes','1024'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
 streams=[]
 try:
  print(p.stdout.readline().strip())
  subprocess.run(['/tmp/singlefs-client-review','e2e',d+'/r/s',d+'/journal'],check=True,timeout=10)
  def rss():return next(l for l in pathlib.Path(f'/proc/{p.pid}/status').read_text().splitlines() if l.startswith('VmRSS:'))
  print('before incomplete frames:',rss())
  for _ in range(4):
   s=socket.socket(socket.AF_UNIX);s.settimeout(5);s.connect(d+'/r/s');streams.append(s)
   s.sendall(struct.pack('<I',16*1024*1024)+b'x'*(2*1024*1024))
  time.sleep(.2)
  print('after 4 incomplete frames:',rss(),'pending_bytes_limit=1024')
 finally:
  for s in streams:s.close()
  p.terminate();p.wait(timeout=5)
with tempfile.TemporaryDirectory(prefix='pr-connect-') as d:
 listener=socket.socket(socket.AF_UNIX);listener.bind(d+'/s');listener.listen(1)
 fillers=[]
 try:
  for _ in range(2):
   s=socket.socket(socket.AF_UNIX);s.connect(d+'/s');fillers.append(s)
  try:subprocess.run(['/tmp/singlefs-client-review','open',d+'/s',d+'/j'],timeout=1,check=True)
  except subprocess.TimeoutExpired:print('open still blocked after 1000ms despite maximum_total=50ms')
 finally:
  for s in fillers:s.close()
  listener.close()
```

### 复现文件：`/tmp/singlefs-journal-lock-review.rs`

```rust
extern crate self as libc;
pub const LOCK_EX:i32=2; pub const LOCK_NB:i32=4;
use std::sync::{Arc,Barrier,OnceLock};
static OPENED:OnceLock<Arc<Barrier>>=OnceLock::new();
static CONTINUE:OnceLock<Arc<Barrier>>=OnceLock::new();
extern "C" {#[link_name="flock"] fn real_flock(fd:i32,operation:i32)->i32;}
pub unsafe fn flock(fd:i32,operation:i32)->i32 {
 if std::thread::current().name()==Some("racing-opener") {OPENED.get().unwrap().wait();CONTINUE.get().unwrap().wait();}
 real_flock(fd,operation)
}
#[allow(dead_code)] mod client {pub fn random_client_identity()->Result<Vec<u8>,String>{Ok(b"audit-identity".to_vec())}}
#[allow(dead_code)] #[path="/home/fy5090/code/singlefs/prune-server/src/journal.rs"] mod journal;
fn main(){
 let path=std::path::PathBuf::from(std::env::args().nth(1).unwrap());
 let mut first=journal::Journal::open(&path,1,false,false).unwrap().journal;
 first.append_write(1,b"first").unwrap();
 OPENED.set(Arc::new(Barrier::new(2))).unwrap();CONTINUE.set(Arc::new(Barrier::new(2))).unwrap();
 let second_path=path.clone();
 let thread=std::thread::Builder::new().name("racing-opener".into()).spawn(move||{
  let mut second=journal::Journal::open(&second_path,100000,false,false).unwrap().journal;
  second.append_write(2,b"second-unacknowledged").unwrap();
  println!("second_open_and_append_succeeded_while_first_still_open=true");
 }).unwrap();
 OPENED.get().unwrap().wait();
 first.append_acknowledgement(1,std::iter::empty()).unwrap();
 CONTINUE.get().unwrap().wait();thread.join().unwrap();drop(first);
 let recovered=journal::Journal::open(&path,100000,false,false).unwrap();
 println!("recovered_pending={} (expected second write)",recovered.unacknowledged.len());
}
```

### 复现文件：`/tmp/singlefs-review-sync.c`

```c
#define _GNU_SOURCE
#include <dlfcn.h>
#include <unistd.h>
#include <errno.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
static int fail(int fd){char link[80],name[4096];const char *arm=getenv("AUDIT_SYNC_ARM");if(!arm||access(arm,F_OK))return 0;snprintf(link,sizeof(link),"/proc/self/fd/%d",fd);ssize_t n=readlink(link,name,sizeof(name)-1);if(n<0)return 0;name[n]=0;if(!strstr(name,"/db/")||!strstr(name,".log"))return 0;fprintf(stderr,"AUDIT failed sync %s\n",name);errno=EIO;return 1;}
int fsync(int fd){if(fail(fd))return -1;int (*real)(int)=dlsym(RTLD_NEXT,"fsync");return real(fd);}
int fdatasync(int fd){if(fail(fd))return -1;int (*real)(int)=dlsym(RTLD_NEXT,"fdatasync");return real(fd);}
```

### 复现文件：`/tmp/singlefs-prune-sync-review.py`

```python
import tempfile,subprocess,socket,struct,os,pathlib
exec(open('/tmp/singlefs-prune-review.py').read().split('with tempfile.TemporaryDirectory')[0])
with tempfile.TemporaryDirectory(prefix='pr-sync-') as d:
 env=dict(os.environ,LD_PRELOAD='/tmp/singlefs-review-sync.so',AUDIT_SYNC_ARM=d+'/arm')
 with open(d+'/server.log','w+') as out:
  p=subprocess.Popen([BIN,'serve','--selftest-instance','--socket',d+'/run/s','--state-directory',d+'/state','--library',d+'/db','--tcp','127.0.0.1:0','--worker-threads','1'],env=env,stdout=out,stderr=subprocess.STDOUT)
  try:
   for _ in range(100):
    if pathlib.Path(d+'/run/s').exists():break
    time.sleep(.02)
   s=socket.socket(socket.AF_UNIX);s.settimeout(5);s.connect(d+'/run/s')
   pathlib.Path(d+'/arm').touch()
   body=b'\x01x'+u64(1)+u32(1)+bytes([1])+lp(b'k')+lp(b'v')
   send(s,1,2,body);print('first_write=',recv(s)[8:])
   send(s,2,2,body);print('retry_same_write=',recv(s)[8:])
   s.close()
  finally:
   p.terminate();p.wait(timeout=5);out.seek(0);print(out.read())
```
