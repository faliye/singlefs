# 崩溃一致性形式化验证：现有工作做到什么、花多少（七·1 岔路代价的线索）

批次：`records/2026-09-30-里程碑三开工第一轮.md#1`（1-2 外部事实）。取得日期：2026-09-30（全部现取，见「复核」）。
性质：**线索，不是证据**。下面每条都是「读论文 / 官方文档 / 仓库得到的」，**全部未在本项目验证**；没有一条是本工程里跑出来的。
口径约定：「行」= 该论文或仓库自己的口径（各家是否含空行、注释、trusted 部分不同，每条注明）；「挂钟」= 论文自报，硬件写在同格；「没查到」后面写查了哪。
本工程的已知情况（用来写差异格）取自 `.claude/kb/decisions/13-验证路线.md` 已定项 1 / 2 / 4 与 `.claude/kb/milestone/03-third-txn.md` 七·1 行：`crates/` 是 Rust；`Recover` 读盘上镜像；journal 记录带 CRC32C（D13 已定项 4 对碰撞「知情接受」）；崩溃枚举域是整写子集加撕裂第三态；提交先串行；M1 全性、M2 前缀不切发布、M3 幂等与单调。

## 0　总览（每家一行，细节与出处在后面各家的表里）

| 对象 | 证 / 查的性质 | 实现行 | spec 行 | 证明或标注行 | 一次验证挂钟（硬件） | 哈希 / 校验和 | spec 从哪来 | 与实现怎么绑 |
|---|---|---|---|---|---|---|---|---|
| Yggdrasil（OSDI 2016，Python + Z3） | crash refinement，逐函数，全自动 | Yxv6 1,500；YminLFS 150；Ylog 60；Ycp 45 | Yxv6 250；YminLFS 25；Ylog 35；Ycp 15 | 无证明；一致性不变式 5 行（Yxv6） | Yxv6+sync 不到 1 分钟（单核 4.0 GHz i7-4790K）；Yxv6+group_commit 1.6 小时（24 核 Xeon 2.2 GHz） | 论文里没有（0 次命中） | 手写 Python 状态机 | 被验的 Python 就是运行的那份（Cython 编成 C，编译器不验） |
| FSCQ（SOSP 2015，Coq，CHL） | 每个系统调用满足崩溃后可恢复的 spec | 没拆（与证明交织） | 没拆 | 合计 31,397 行「代码加证明」 | 检查证明 11 小时（i7-3667U 2.0 GHz，8 GB） | 论文里没有（0 次命中）；日志头 length 兼提交位 | 手写 CHL spec | 实现就在 Coq 里，抽取成 Haskell，Haskell 运行时进 TCB |
| DFSCQ（SOSP 2017，Coq） | metadata-prefix spec（fsync / fdatasync），树序列 | 没拆 | 没拆 | 合计 83,927 行「spec、代码、证明」 | 论文没给 | Hash 操作码：碰撞建模成不终止 | 手写 | 同 FSCQ |
| Kani（AWS，Rust 有界模型检查，CBMC） | 默认 UB 与 panic，加用户断言；合约可上无界 | 各项目自己的代码 | 无独立 spec（断言与合约在 harness 里） | Firecracker 34 个 harness；Hifitime 153 个 harness 加 125 条合约 | CI 总时：Firecracker 21 分钟，s2n-quic 23 分钟，verify-rust-std 69 分钟 | 官方文档：拿模型检查器反推密码哈希「不会终止」 | 用户写在 harness 里 | 直接编译 Rust 的 MIR |
| ShardStore（SOSP 2021，S3） | 功能正确、崩溃一致、并发线性化——**用测试与随机化，没用 Kani** | 44,048 | 参考模型 450（Rust） | 验证与测试 harness 8,422（4,860 加 2,661 加 901） | 没给挂钟；「上线前跑几千万条随机序列」 | 测试里当作「读盘校验必须拦住损坏」 | 与实现同语言的可执行参考模型 | 同一个 crate 里的 proptest / Loom / Shuttle harness |
| Verus persistent log（SOSP 2024，Azure） | 精化抽象无限日志、崩溃原子、元数据 CRC 内的损坏检测 | 739 | trusted 1,284（含 CRC 库 spec） | 证明 2,913 | 12 秒（EC2 m5.8xlarge，Xeon 8175M 2.5 GHz，16 核） | CRC 库写 spec 并标 trusted | 手写 trusted spec | Verus 直接验 Rust，rustc 只看到可执行部分 |
| CapybaraKV（OSDI 2025，Verus） | 崩溃一致（PoWER）加损坏检测（CRC 模型） | 5,531 | trusted 5,244 | spec 加证明 14,255 | 54 秒单线程 / 23 秒 8 线程（i7-11850H，8 物理核，32 GB） | CRC-64 公理：汉明距离 1 到 c 的字节串 CRC 不同 | 手写 trusted PM spec | Verus 直接验 Rust；PoWER 与 CHL 的对应在 Rocq 里证 |
| TLA+ 系（AWS、MongoDB、CCF、etcd） | 协议级安全性；实现靠 trace 验证 | CCF 共识实现 2,174 | CCF 共识 spec 1,134 | 无证明；trace spec 369 | CCF 共识 spec 模型检查约 10^6 状态每分钟、总计约 10^8 状态（Azure DC8s v3）；找到选举计票 bug 的一次是 128 核上穷举 48 小时；一条 consistency trace 用 DFS 验不到 1 秒（BFS 约 1 小时） | 都不建模 | 手写，或从代码逆向 | trace 验证（TLC）；MongoDB Server 一条线 10 周后放弃 |
| Apalache（符号有界） | 安全性，长度 k，归纳不变式 | — | — | — | 官方论文：Paxos / Raft 类 10 到 15 步后 Z3 变慢 | 无 | — | 没查到与实现 trace 对拍的公开案例 |

## 1　Yggdrasil（OSDI 2016，Sigurbjarnarson、Bornholt、Torlak、Wang；Python + Z3）

出处：<https://www.usenix.org/system/files/conference/osdi16/osdi16-sigurbjarnarson.pdf>，2026-09-30 下载全文读（读论文，没读源码仓，没跑）。下表「§」是论文小节号。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 证的性质 | crash refinement：实现能产生的磁盘状态集合（含崩溃）是 spec 允许集合的子集；对每个函数各出一条公式（Definition 4），无操作函数出恒等式（Definition 5） | §3.1、§3.2 | 论文原话：「the crash refinement theorem holds for all disks with up to 2^64 blocks, and for every trace of file system operations, regardless of its length」（§2.3） |
| 实现行数 | Yxv6 1,500；YminLFS 150；Ycp 45；Ylog 60；工具基础设施 1,500；FUSE 桩 250；合计「约 4,000 行 Python」 | Figure 5、§7 | 不含空行与注释 |
| spec 行数 | Yxv6 250；YminLFS 25；Ycp 15；Ylog 35 | Figure 5 | 同上 |
| 证明或标注行数 | 没有证明；只写 spec、实现与一致性不变式（Yxv6 5 条，YminLFS 5 条，Ycp 与 Ylog 0 条） | Figure 5、§1 | 论文自己对照：BilbyFs 13,000 行证明对 1,350 行代码、9.25 人月（转引 Amani 等，未核原文）；FSCQ 用了 1.5 年 |
| 一次验证挂钟与硬件 | YminLFS 修好后「under 30 seconds」（§2.3）；Yxv6+sync「under a minute on a single core」；Yxv6+group_commit「within 1.6 hours using 24 cores (Intel Xeon 2.2 GHz), approximately 36 hours on a single core」；贪心去 flush 优化器：mknod 五次 flush 去掉三次用了不到 3 分钟（§2.4） | §2.3、§2.4、§8「Verification performance」 | Yxv6+sync 的机器是 4.0 GHz 四核 Intel i7-4790K、Linux 4.4.0（§8 开头）；group_commit 慢的原因是事务大小上限 k 更大：sync 版 k = 10，group_commit 版 k = 511（§4.2）——挂钟随 k 变 |
| 循环与递归怎么处理 | 「self-finitizing」符号执行：循环与递归不设深度地展开；因此要求文件系统必须写成有限形态，「loops must be bounded」；符号值只支持布尔、定长整数、映射、具体长度的列表；不支持异常与协程，不能符号执行进 C 库 | §3.2 | 展开深度由代码有限性保证，不是用户给的界 |
| 磁盘怎么表示 | 未解释函数：（块地址，块内偏移）到 64 位整数的两层映射；试过「符号块惰性列表」与数组理论，都成瓶颈 | §6「Lessons learned」 | 每块假设一次读写原子（§2.2「a single block read/write is atomic」） |
| 哈希或校验和 | 论文里没有：`grep -a -c -i -E 'checksum\|hash\|crc'` 命中 0。日志用头块里的条目计数器，一致性不变式说恢复后计数器恒为零 | §4.2 第 2 层 | 「没查到」= 只查了这一篇全文 |
| spec 从哪来 | 手写 Python 状态机；磁盘模型是 trusted 公理（Axiom 1：块设备是异步磁盘 spec 的 crash refinement） | §2.1、§4.2 | 一致性不变式也手写 |
| 与实现怎么绑定 | 没有单独的「模型」：被符号执行验证的 Python 由 Cython 编成 C，经 FUSE 运行；TCB 含 spec、验证器、可视化器、编译器、Z3、Python、gcc、FUSE、Linux（优化器不在 TCB，改完重验） | §2.4、§2.5 | Cython 编译器未验证 |
| 已知限制 | 只推理单线程代码；一阶逻辑；Z3 本身要信（作者用 1,417 个 Z3 历史版本加 Boolector、MathSAT 交叉验证，未见不一致）；无 extents 与延迟分配；生成的 fsck 只检测不修复；定理只在文件系统独占磁盘时成立 | §2.3、§6 | — |
| 附带发现 | 对照 Arrakis 持久日志的实现发现两个 bug：恢复逻辑不幂等；恢复中再崩可能留垃圾 | §5「The Ylog persistent log」 | Ylog 用同步磁盘模型（电池后备缓存的 RAID 卡） |
| **与本工程的一处差异** | Yggdrasil 被验证的那份 Python 与运行的那份是同一份源码；本工程七·1 的候选 ① 里可符号执行的副本与 `crates/` 的 Rust 实现是两份，两份之间的对应论文里没有这一步可以借鉴。另外 Yggdrasil 的日志靠计数器提交、假设单块写原子、没有校验和；本工程恢复读 CRC32C 与 AEAD，枚举域含撕裂第三态（D13 已定项 4） | — | 差异是做法层面的：被验物与运行物是否同一份 |

## 2　FSCQ（SOSP 2015）与 DFSCQ（SOSP 2017），Coq，Crash Hoare Logic

出处：FSCQ <https://pdos.csail.mit.edu/papers/fscq:sosp15.pdf>；DFSCQ <https://pdos.csail.mit.edu/papers/dfscq.pdf>；仓库 <https://github.com/mit-pdos/fscq>（提交 2c7ef9c2，提交日 2022-10-21）。2026-09-30 读论文全文并浅克隆仓库自己数了行。同组后续的 DaisyNFS（OSDI 2022）<https://www.usenix.org/system/files/osdi22-chajed.pdf> 给了拆开的行数，放最后几行。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 证的性质 | FSCQ：每个过程带前条件、后条件、崩溃条件，顶层过程另带恢复过程与恢复条件；证明恢复后不丢数据，rename 原子。DFSCQ：metadata-prefix 规约（描述 fsync、fdatasync 的语义），用「树序列」表示崩溃后可能的状态；证明覆盖它实现的全部系统调用 | FSCQ §1、§3；DFSCQ 摘要与 §6 | 「Print Assumptions」核对无未证公理（FSCQ §6） |
| 行数（论文） | FSCQ 合计 31,397 行「代码加证明」，其中 CHL 基础设施 5,835、证明自动化 2,304、FscqLog 3,163；论文说证明与代码交织难以拆开；约为 xv6 文件系统（约 3,000 行 C）的 10 倍。DFSCQ 合计 83,927 行「spec、代码、证明」，其中基础设施 24,032、写前日志 11,791、Hashing semantics 458 | FSCQ Figure 18、§6、§7.4；DFSCQ Table 2 | **论文都没有把证明与实现拆开，所以「证明行数比实现行数」这个比值论文里没有** |
| 行数（我数的） | 仓库 `src/*.v` 共 98 个文件，非空行 83,866；从 `Proof.` 到 `Qed.` 之类结束符的行 51,509；其余非空行 32,357（含 spec 陈述、定义与代码，无法再拆） | 命令见「复核」；脚本 `/tmp/claude-1000/m3-r1/formal-prior-art/count_coq.py` | 粗口径：按正则切；总数与 DFSCQ Table 2 的 83,927 相近；**51,509 比 32,357 不是证明比实现**，因为分母里有 spec 与定义 |
| 一次验证挂钟与硬件 | FSCQ 检查证明 11 小时，Intel i7-3667U 2.00 GHz、8 GB；不检查证明、只编译出 FUSE 可执行文件不到 2 分钟。DFSCQ 论文没给检查时长 | FSCQ §6；DFSCQ 全文 `grep -a -i -E 'hours\|minutes\|GHz'` 无命中 | Yggdrasil 论文转述「Coq took 11 hours to check the proofs of FSCQ」（§6），与 FSCQ 自报一致 |
| 开发花费 | FSCQ：几位研究者一年半，多数花在证明与 spec；DFSCQ：5 人两年，都不是全职，「much less than 10 person-years」 | FSCQ §6；DFSCQ §6 | — |
| 循环怎么处理 | 循环是 CHL 里的 `for` 组合子，证明了 `for` 与 `if` 的标准 Hoare 规约；定理对任意输入成立（Coq 全称），不是有界 | FSCQ §6「CHL」 | 循环不变式由证明者写在规约里 |
| 哈希或校验和 | FSCQ 论文：`grep` 0 命中；日志头的 length 字段兼作提交位，恢复时 length 非零才重放（为省一次 sync，改了约 50 行代码与 100 行证明）。DFSCQ：新增 Hash 操作码，语义里记录所有 hash 过的输入，碰到「同 hash 不同输入」就让该操作码无限循环不返回（碰撞建模成不终止）；运行时用抗碰撞哈希；否掉了「概率推理」与「公理：无碰撞」两条路（后者用鸽笼原理会推出矛盾） | FSCQ §7.4；DFSCQ §5.6 | DFSCQ 说细节在 Wang 的硕士论文（没读） |
| 写原子性假设 | 「disk_write 的 spec：块整块原子写，崩溃后要么是最后写的值要么是以前的某个值，不允许部分块写」；如何去掉这个假设留给未来 | FSCQ §3.2 | 单块是一个扇区 |
| spec 从哪来 | 手写 CHL spec | FSCQ §3 | — |
| 与实现怎么绑定 | 实现就在 Coq 里，与 spec 同一语言；Coq 抽取成 Haskell，再加 400 行 Haskell 驱动与 350 行 Haskell（定长字、块操作、缓存替换策略）；Haskell 编译器与运行时进 TCB | FSCQ §6「FSCQ file server」「Limitations」 | DFSCQ 的 TCB：spec、执行语义、Coq 检查器、抽取、运行环境（Haskell 运行时、FUSE、Linux 磁盘驱动） |
| 已知限制 | 不支持多处理器与并发（两篇都说）；FscqLog 一次一个事务、整个事务日志放内存、不保证日志空间够；FSCQ 无 fsync、无硬链接 | FSCQ §1（fsync）、§5.3（硬链接）、§5.4（FscqLog）、§6「Limitations」、§7.2（多线程）；DFSCQ §6 | — |
| 同组后续拆开的数（DaisyNFS，OSDI 2022） | Figure 12 三列「proof / code / spec」：GoJournal 29,000 / 1,419；事务系统 10,000 / 250 / 932（Coq，Perennial）；文件系统 6,787 / 4,051 / 630（Dafny）；正文：事务系统证明对代码 20 倍，Dafny 部分约 2 倍（代码 = Dafny `/printMode:NoGhost` 抽出的行）。编译（检查）证明：慢机器 CI 12 分钟，8 核笔记本 4 分钟 | DaisyNFS 摘要、§1、§8 | 这是另一个系统（NFS 服务器，Go 加 Dafny），不是 FSCQ 的数 |
| **与本工程的一处差异** | FSCQ / DFSCQ 的实现本身写在 Coq 里（纯函数式，抽取到 Haskell），证明与实现同源；本工程实现在 Rust，走 Coq 一类路线等于另写一份。另外它假设整块原子写，本工程枚举含撕裂第三态；DFSCQ 把哈希碰撞写成不终止，本工程 D13 已定项 4 对 CRC32C 碰撞是「知情接受」，不进模型 | — | 差异：实现语言与被证物 |

## 3　Kani（Rust 有界模型检查）与 AWS ShardStore

出处：Kani 官方文档 <https://model-checking.github.io/kani/>（源文件取自 `github.com/model-checking/kani` 的 `docs/src/`，2026-09-30 取）；最新发布标签 `kani-0.68.0`，发布日 2026-09-16（GitHub API 查）；Kani 论文 arXiv 2607.01504（Delmas 等，ASE 2026，2026-07-01 上传）<https://arxiv.org/pdf/2607.01504>；Firecracker 博客 <https://model-checking.github.io/kani-verifier-blog/2023/08/31/using-kani-to-validate-security-boundaries-in-aws-firecracker.html>；ShardStore <https://www.cs.utexas.edu/~bornholt/papers/shardstore-sosp21.pdf>（SOSP 2021，Bornholt 等）。

**先说一个与题面有出入的事实**：ShardStore 的 SOSP 2021 论文里**没有用 Kani**——`grep -a -c -i kani` 该论文文本命中 0（本文件没查 Kani 的开源日期，只查了该论文文本里没有这个词）。论文用的是：proptest 属性测试加同语言可执行参考模型；崩溃一致性用「参考模型加 DirtyReboot 操作」的属性测试；并发用 Loom（可靠、有界偏序归约）与 Shuttle（随机）；Miri 查 UB；Crux（有界符号求值）证反序列化不 panic；Prusti 只做了参考模型自身性质的早期实验。下面 3A 是 Kani 本身，3B 是 ShardStore 实际用的手段。

### 3A　Kani

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 是什么 | 编译 Rust 的 MIR 成 GOTO 程序交给 CBMC，SAT 或 SMT 求解（默认 MiniSat，可选 Kissat、CaDiCaL、Z3、cvc5、Bitwuzla）；harness 形如单元测试，`kani::any()` 产生非确定值，`kani::assume` 收窄 | ASE'26 论文 §2、§3 | 自动检查的默认性质：UB、panic（溢出、越界、unwrap None）等 |
| 用过的项目与 CI 挂钟 | Firecracker 34 个 harness 21 分钟；s2n-quic 102 个 23 分钟；Hifitime 153 个 42 分钟；verify-rust-std 16,748 个 69 分钟；zerocopy 10 个 3 分钟；lading 22 个 2 分钟；x86_64 6 个不到 1 分钟；rust-sel4 1 个 4 分钟 | ASE'26 论文 Table 1、§5.3 | 「Runtime」是 CI 时长；**硬件没查到**（该论文文本 `grep -a -i -E 'vCPU\|GHz\|EC2'` 无相关命中）；八个项目**无一个是存储或文件系统**（Table 1 自称「Subset of open-source projects」，是子集） |
| Firecracker 早期博客数 | 27 个 harness，3 套验证，CI 约 15 分钟（硬件没写）；TokenBucket 用 `#[kani::unwind(1)]`，VirtIO 队列用 `#[kani::unwind(2)]`；桩掉 `Instant::now`、`libc::clock_gettime`、`libc::sysconf`；找到 6 个问题（限速器 5 个、VirtIO 1 个） | 博客（2023-08-31） | 与论文的 34 个 / 21 分钟是不同时间点的数 |
| 循环 | 展开界要「比循环次数多一」，有时要多二或多三；「it's not uncommon for some proofs to scale poorly even to 5 iterations」；示例 LIMIT 100 用作者机器约 3.8 秒；界不够报 unwinding assertion 失败，不会悄悄通过；找不到界就一直展开。论文里 Firecracker 的 GCD（u64）：给足界（94）全域有界验证「一小时超时未结束」；改用循环合约（实验特性）后 0.54 秒（Z3，202 项检查） | 文档 tutorial-loop-unwinding、debugging-slow-proofs；ASE'26 §2 | 循环合约要写循环不变式，等于回到手写标注 |
| 堆与集合 | `Box`、`Rc`、`Arc` 在支持表里是「Yes」；文档把「大的有界集合，如很长的 Vec」与字符串列为让证明变慢的常见原因，示例把 Vec 缩到 2 个元素；规约表达式不许堆分配 | 文档 rust-feature-support、debugging-slow-proofs；ASE'26 §4.2 | 没查到 Kani 对 HashMap 的专门说明 |
| trait 对象 | 支持表里「Trait object types」「Closure types」「Type parameters」「Function pointer types」都是「Partial」；论文说合约的 spec 语言不处理动态 trait 分派，合约按单态化逐个验证 | 文档 rust-feature-support；ASE'26 §5 末「Threats to validity」 | 「Partial」文档没展开范围 |
| 并发与异步 | 「Concurrent features are currently out of scope for Kani」，遇到并发代码只报警并按顺序代码编译；await 表达式与数据竞争在支持表里「No」；官方建议并发用 Loom（可靠）与 Shuttle（随机、可扩展） | 文档 rust-feature-support、tool-comparison | 论文补充：成功的运行只在「顺序执行（无线程与 async）、无指针别名（Stacked / Tree Borrows）建模、FFI 在 CBMC 内存模型外除非桩掉」等前提下保证 |
| 文件与 I/O | 文档 overrides 页说需要建模的典型是「system calls and I/O operations」，并列了被覆盖的符号：打印宏、assert 宏、`std::process::{abort, exit}`；支持表里没有 `std::fs` 的行 | 文档 overrides、rust-feature-support | **没查到官方对 `std::fs`、块设备读写怎么处理的说法**（查了这两页的全文） |
| 哈希 | 官方比较页：「you can easily try to reverse a cryptographic hash with a model checker, but good luck getting it to terminate」；调慢证明的建议里有「大值上的乘除取模，尤其 u64」 | 文档 tool-comparison、debugging-slow-proofs | 没查到 Kani 对 CRC 或 AEAD 的官方处理办法 |
| 其他不支持 | 不支持栈展开（panic 的 unwinding）；不支持内联汇编；浮点里的三角函数与 sqrt 是过近似 | 文档 rust-feature-support | — |
| 与验证工具的对照 | 论文说 Verus 的 SOSP 2024 案例约每行实现 5 行证明（6.1K 实现、31K 证明）；Kani 的 Hifitime 案例给 9.5K 行的库加了 125 条合约、153 个 harness | ASE'26 §6 | 论文自己的比较，Hifitime 是时间库 |
| spec 从哪来 / 怎么绑 | 断言与合约由用户写在 harness 或函数属性里，同一个 crate、同一份源码；`cargo kani` 与 `cargo test` 用法一致；s2n-quic 用 Bolero 让同一个 harness 既当模糊测试又当 Kani 证明 | ASE'26 §3、§5.3 | 没有独立于实现的 spec 文件 |
| 已知限制 | 界外不说话（无合约的 harness 只有有界保证）；假设写错会让证明空真；不建模指针别名；合约按单态化验证 | ASE'26 §2.1、§4.3、§5 | — |
| **与本工程的一处差异** | 已有案例（Firecracker 限速器与 VirtIO 队列、s2n-quic 编码边界、Hifitime）的被验对象是算术与小状态；本工程 `Recover` 读的是盘上镜像字节并验 CRC32C 与 AEAD，镜像大小与哈希是有界模型检查里文档点名会拖慢的两类东西。`Recover` 的读写抽象、线程与 trait 对象的实际情况归 1-1 事实表，本文件没查 | — | 差异：被验对象的形态与输入规模 |

### 3B　ShardStore（AWS S3 存储节点，SOSP 2021）实际用的手段

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 规模 | 实现 44,048 行 Rust；单元与集成测试 19,540；参考模型 450；功能正确检查 4,860；崩溃一致检查 2,661；并发检查 901；合计 72,460 | Figure 6 | 「All lines are Rust code」；验证工件合计占总代码 13%、实现的 20%，对照「形式验证 3 到 10 倍」的开销（§8.2） |
| 人力 | 两位形式化专家全职九个月，第三位专家三个月，之后主要由没有形式化背景的工程师维护；测试 harness 里 18% 的行最后一次修改来自非专家（git blame） | §8.2 | 「crude measure」 |
| 崩溃一致性怎么查 | 参考模型加崩溃时可丢的「最近变更」；操作字母表加 `DirtyReboot(RebootType)`；重启恢复后检查两条性质：persistence（依赖对象说已持久的必须读得到）与 forward progress（干净重启后每个依赖都持久）；块级穷举崩溃状态的变体「has not found additional bugs and is dramatically slower to test, so we do not use it by default」 | §5 | 默认粒度粗：每个组件选「整体刷盘或不刷」 |
| 挂钟 | 没给单次时长；「we routinely run tens of millions of random test sequences before every ShardStore deployment」 | §4.1 | 没给硬件 |
| 拦下的问题 | 16 个（Figure 5 目录），含崩溃一致与并发 | §1、§8.1 | 属性测试查功能与崩溃，Loom / Shuttle 查并发 |
| 校验和 | 被当作「读盘数据不可信，组件须检测并使操作失败」；序列化代码用 Crux 证明：对任意长度不超过上界的盘上字节序列，反序列化不会因越界或逻辑错 panic，更大输入改用模糊测试；崩溃一致 bug #10 的成因是块帧头尾的 UUID 与魔数在崩溃后撞上 | §4.4、§5、§7 | 界：「up to a size bound」，论文没写数 |
| 为什么不用建模语言 | 「We could instead have used a language intended for modeling (Alloy, Promela, P, etc.)…however, we found that by writing reference models in the same language as the implementation, we make them easier for engineers to keep up to date」 | §3.2 | 模型即 mock，兼作单元测试的替身 |
| 已知限制 | 检查通过不等于正确；资源耗尽没有测（没有正确性 oracle）；崩溃粒度粗 | §4.4、§5、§8.3 | — |
| **与本工程的一处差异** | ShardStore 的崩溃模型是「用户态依赖对象 + 软更新」，规约是「已持久必可读」，落点在存储节点的软更新协议上；本工程是 journal 记录加 checkpoint 发布，性质是前缀不切发布与重放幂等，且已经有全域枚举（崩溃点全量，不是随机） | — | 差异：崩溃一致协议不同，枚举方式不同 |

## 4　Verus，及基于它的存储系统（Azure 持久日志；CapybaraKV）

出处：Verus SOSP 2024 论文 <https://www.andrew.cmu.edu/user/bparno/papers/verus-sys.pdf>（Lattuada、Hance 等，2026-09-30 下载全文）；PoWER（OSDI 2025，LeBlanc、Lorch、Hawblitzel、Huang、Tao、Zeldovich、Chidambaram）<https://people.csail.mit.edu/nickolai/papers/leblanc-power.pdf>；Verus 官方指南 <https://verus-lang.github.io/verus/guide/features.html>、<https://verus-lang.github.io/verus/guide/while.html>（2026-09-30 取）。CapybaraKV 的仓库 `github.com/microsoft/verified-storage` 只在 Verus 官方项目页里见到链接，没读。

### 4A　SOSP 2024 论文里的持久日志（Azure Storage，持久内存上的循环日志）

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 证的性质 | 实现精化一个抽象的无限日志；所有操作对崩溃原子；日志元数据「protected from corruption up to CRC」；只在 `advance_head` 释放过的空间上覆盖 | §4.2.5 | 面向持久内存；日志已并入 Azure 的生产代码库，作为一个普通 crate |
| 行数 | trusted 1,284；证明 2,913；实现 739；证明对代码比 3.9 | Figure 9「P. log」行 | 论文正文「Verus verifies the log implementation in 12s with a proof-to-code ratio of 3.9」；论文在 IronKV 一段说过 trusted 列「noisy」（口径是把原协议层重新解释成 spec），日志一段没有这句 |
| 一次验证挂钟与硬件 | 12 秒（Figure 9 里 1 核 12、8 核 6；列头顺序是我从抽取的文本推的，12 与正文吻合） | §4.2.5、Figure 9 | 除非另注，实验在 AWS EC2 m5.8xlarge（Xeon Platinum 8175M 2.50 GHz，16 核、关超线程，128 GiB）（§4 开头） |
| 五个大案例合计 | trusted 4,692；证明 31,231；实现 6,119；比 5.1；各案例比：IronKV 2.9、NR 7.1、页表 13.3、mimalloc 4.3、日志 3.9 | Figure 9 | 摘要口径「6.1K 实现、31K 证明」 |
| 哈希或校验和 | 「For the crates that the verified code depends on, such as a CRC crate, we write a specification and mark it trusted」 | §4.2.5 | 校验和的正确性归 trusted spec |
| 循环 | 用户写循环不变式；官方指南：「verifying loops requires programmer effort to write loop invariants」，不变式过弱或过强都会验不过 | 指南 while 页 | — |
| spec 从哪来 / 怎么绑 | 手写 spec，元数据字节布局用 `Serializable` trait 的 spec 方法写；Verus 直接验 Rust 实现，「Verus erases all but Rust content for tools other than the verifier」，rustc 只见可执行代码 | §4.2.5 | 无独立模型与实现之间的对拍 |
| 已知限制 | 结论受 TCB 限制：顶层 spec、Verus、求解器、Rust 编译器（§1）；官方指南列为「不支持」：async、`Drop`、`Mutex` 与 `RwLock`、打印与 I/O、函数指针、`Pin`、`transmute`、硬件内建函数；「部分支持」：trait 对象、浮点、迭代器、指针、panic 展开、多 crate | §1；指南 features 页 | 支持面以指南当天的页面为准 |

### 4B　CapybaraKV（Verus）与 CapybaraNS（Dafny），PoWER 方法

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 证的性质 | 崩溃一致（PoWER：把恢复要求写成存储 API 写操作的前条件）；损坏检测（读返回的字节可能与写入的不同，须做 CRC 检查才能证明没损坏） | 摘要、§3、§4.1 | 面向持久内存 |
| 行数 | CapybaraKV：trusted 5,244；spec 加证明 14,255；实现 5,531；证明对代码 2.6。其中通用 redo journal 组件：trusted 0，spec 加证明 2,234，实现 611。CapybaraNS（Dafny）：trusted 414；spec 加证明 673；实现 278；比 2.4 | Table 2、§6.1 | 「Trusted / Spec+Proof / Impl」三列，CapybaraKV 的 trusted 含 Windows 与 Linux 的 PM 后端与一个字节向量模拟区 |
| 一次验证挂钟与硬件 | CapybaraKV 54 秒（1 线程）、23 秒（8 线程）；CapybaraNS 12 秒（Dafny 不支持多线程验证）；硬件 Linux 6.9.3、Intel Core i7-11850H、8 物理核、32 GB | §6.1「Verification time」 | 验证本身不需要特殊硬件；复现性能实验要求至少 128 GB 的 Optane 持久内存（或 DRAM 模拟），并建议 128 GB 内存与 32 核（附录 A「Requirements」） |
| 开发花费 | CapybaraKV 约 1.5 年（专家加新手组成的团队）；CapybaraNS 在 spec 与 C# 包装写好后约 9 小时；PM spec 移植到 Dafny 不到一人时 | §6.1 | — |
| 哈希或校验和 | 损坏模型：每个存储位一个损坏掩码，掩码为 1 的位读回任意值；掩码里 1 的个数不超过一个对被验代码不透明的常数 c；公理：汉明距离在 [1, c] 内的两个字节序列 CRC 不同；用的 ECMA CRC-64，任意长度 c = 1（能抓单比特错误），短于约 1 GiB 时 c = 3（引 Koopman），实现只用了 c = 1；另设「corruption-detecting Boolean」（CDB，8 字节）做持久内存上崩溃原子的 CRC 更新，因为 CRC 不保证与数据一起原子落盘 | §4.1、§4.2 | 与 VeriBetrKV 的公理「corruption cannot produce a block with a valid checksum」不同：VeriBetrKV 的公理限定了校验和的位置与范围（§2.3，转述，没读 VeriBetrKV 原文） |
| 崩溃模型 | 持久内存写可被重排，模型还包含同一缓存行内 8 字节块重排产生的状态（对当前 Intel 平台是过近似）；不绑定具体的 flush 指令 | §5 | 「PoWER is not tied to any particular storage model」 |
| 恢复幂等 | 「recovery is idempotent. If the system crashes while recovering from an earlier crash, all modifications made during the first recovery will be overwritten by the second recovery」是 PoWER 恢复类写的证明策略之一 | §3.3 | 原地写（in-place）没有库支持，作者说没有相关经验 |
| spec 从哪来 / 怎么绑 | 手写 trusted 的 PM 规约；PoWER 与 CHL 的对应用 Rocq 写机械检查的证明，但是「metalogical，依赖一个 trusted 的翻译」；与崩溃不变式（Perennial 式）的对应在 Verus 里机械检查 | §3.2 | 作者说明没有在 Rocq 里实现 PoWER |
| 已知限制 | PoWER 库不支持原地写；面向持久内存的模型（8 字节原子）；并发靠工具自带（Verus 有原子不变式，Dafny 没有） | §3.3、§3.4 | — |
| **与本工程的一处差异** | 两个案例的崩溃模型是持久内存（字节可寻址、8 字节原子、缓存行重排）加日志或 KV 结构；本工程是块设备、整写子集加撕裂第三态枚举、journal 记录加 checkpoint 发布。且 Verus 要求被验代码写成 Verus 方言并附证明（上表证明对代码 2.6 到 13.3），本工程实现已经是无标注的 Rust | — | 差异：崩溃模型与被验代码要不要改写 |

## 5　TLA+ / TLC / Apalache 在提交协议、日志、恢复上的公开案例，及与实现的绑定

先说取得情况：AWS 2025 年 CACM 文章「Systems Correctness Practices at Amazon Web Services」（Brooker、Desai）两个官方地址都返回 403，**没读到全文**，只见搜索结果摘要（提到 TLA+、P、属性测试、故障注入、事件模拟与 ShardStore），不当事实用。下面各家都读了全文或仓库。

### 5A　AWS，「Use of Formal Methods at Amazon Web Services」（Newcombe 等，2014-09-29 署名日期，后发 CACM 2015）

出处：<https://lamport.azurewebsites.net/tla/formal-methods-amazon.pdf>，2026-09-30 下载全文读。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 做了什么 | DynamoDB 复制与容错机制的 TLA+ spec：作者「几周」写成；分布式 TLC 跑在十台 cc1.4xlarge（每台 8 核加超线程、23 GB）上；找到一个会丢数据的 bug，最短错误轨迹 35 个高层步骤；工程师自学 TLA+ 得到有用结果需 2 到 3 周 | 正文「DynamoDB」段与「Learning」段 | 设计层的 spec，不是实现 |
| 与实现怎么绑 | 没有绑：「While we would like to verify that the executable code correctly implements the high-level specification, or even generate the code from the specification, we are not aware of any such tools that can handle distributed systems as large and complex as those we are building」 | 「How do we know that the executable code correctly implements」一节 | 2014 年的说法，不代表今天；同段说「用 TLC 为代码找测试边角」这一路「seems promising」但还没在软件上试过 |
| 哈希或校验和 | 没有（该文全文相关词只命中 1 处，与建模无关） | — | — |
| 已知限制 | 论文自陈：spec 验的是设计，代码是否忠实实现要靠别的办法 | 同上 | — |

### 5B　MongoDB，「eXtreme Modelling in Practice」（Davis、Hirschhorn、Schvimer，PVLDB 13(9)，2020）

出处：<https://arxiv.org/abs/2006.00915>（PDF 2026-09-30 下载全文读）。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 做了什么 | 两个案例：Server 复制协议的 model-based trace-checking（MBTC）；Realm Sync 的 OT 算法 model-based test-case generation（MBTCG） | 摘要、§4、§5 | Server 已开发 12 年、「hundreds of thousands of lines」C++；Realm Sync 4 年、「tens of thousands of lines」 |
| MBTC 花费与结果 | 两位工程师 10 周：事件埋点 4 周 570 行 C++；改 `RaftMongo.tla` 3 周 252 行 TLA+；Python 后处理 3 周 484 行；只对 5 个手写测试与 1 个随机测试做了 trace 检查，只有 1 个手写测试的 trace 通过，其余 4 个因两处实现与 spec 的差异违反 spec；没接 CI、没量覆盖；「abandoned the project after 10 weeks」 | §4.1、§4.2 | 原因：并发与加锁下难以在同一时刻记录整个状态（oplog）、模型与实现有差异、TLC 对 trace 检查支持不全 |
| spec 大小影响 | 为贴近实现对 `RaftMongo.tla` 做的有限改动使状态空间从 42,034 增到 371,368，模型检查从 2 秒增到 14 分钟；3 节点、至多 3 个 term、oplog 至多 3 条时 TLC 发现 371,368 个不同状态；trace 几百个事件可用，几千个事件「impractically slow」 | §4.2.3、§4.2.4 | 硬件没写（没查到） |
| MBTCG 花费与结果 | 一位工程师约 4 周：`array_ot.tla` 795 行 TLA+（2 周）；Golang 生成器 755 行（2 周）；3 元素初始数组、3 个客户端各一次操作，生成 4,913 个 C++ 测试；被测实现是自包含函数、约 1,000 行 | §5 | 成功的条件：spec 写成时就与实现「close correspondence via direct transcription」，且对称性使模型检查可行 |
| 哈希或校验和 | 没有（全文相关词 0 命中） | — | — |
| 已知限制 | 作者结论：MBTC 对 Server 不实际，因为 spec 与实现缺少紧密对应；每多一份 spec，trace 检查的边际成本不会降到很低 | §6 | — |

### 5C　Microsoft CCF，「Smart Casual Verification of the Confidential Consortium Framework」（Howard、Kuppe、Ashton、Chamayou、Crooks）

出处：arXiv 2406.17455 <https://arxiv.org/pdf/2406.17455>，2026-09-30 下载全文读（论文文本里没有出现会议名，本文件不写会议）。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 查的性质 | 共识协议（含动态重配置）的安全性与自定义客户端一致性模型；TLC 穷举、随机模拟、trace validation 三种并用；找到 6 个 bug | 摘要、§7、Table 2 | CCF 实现总量 63 kLoC C++（§1） |
| 行数（Table 1） | 共识：spec 1,134 行、13 个变量；模型检查配置 158；模拟 69；trace validation 369；实现 2,174 行、25 个变量；功能测试 2,579；端到端测试 2,815。一致性：spec 375 行；模型检查 70；trace validation 111；功能测试 123 | Table 1 | 「All numbers measured on an Azure DC8s v3 VM」 |
| 状态量与挂钟 | 共识 spec 模型检查约每分钟 10^6 状态、总计约 10^8；找到选举计票 bug 的一次是「48 hours of exhaustive model checking of the consensus spec on a 128 core machine」；trace validation 用 TLC 深度优先后，一条 consistency spec 的 trace 从 BFS 约 1 小时降到不到 1 秒 | Table 1、§6.4、§7 Table 2 | DFS 的原理：只要找到一条与 trace 一致的行为即可 |
| 与实现怎么绑 | trace validation：测试驱动确定性串行化各节点，全局单一时钟，实现里加 15 条日志语句在无副作用的线性化点记录消息收发与节点高层状态转移；写一份 Trace spec（约 400 行，复用高层 spec 的动作，仅当 trace 当前一行匹配该动作才使能），用 TLC 检查 trace 与 spec 行为集合的交集非空；已并入 CCF 的 CI | §6.1、§6.2、§6.5 | 只对已有测试（含 13 个手写场景）产生的 trace 检查，不是任意执行 |
| 花费 | 测试驱动加日志约 1 天；得到能验过多数 trace 的 Trace spec 约 2 人月（跨 4 个月），主要是增强 TLC（动作组合、DFS、调试、状态图可视化）与分辨差异出在 spec 还是实现；写 Trace spec 本身是小事（88 次细粒度提交）；后来对 consistency spec 做 trace validation 约 1 人周（跨两周），且基本由 CCF 专家完成 | §6.5 | spec 是先读代码逆向写的，再靠 trace validation 纠正（Cirstea 等论文的描述，§4.4） |
| 哈希或校验和 | 没有建模（相关词 3 处命中，均与 spec 无关：账本摘要的用途说明、Q-Learning 的状态哈希函数、参考文献） | — | — |
| 已知限制 | trace 里的不确定性要靠增加日志或设计「建模模式」抵消阻抗失配；一致性 spec 假设知道所有客户端的事务，trace 只有单个客户端的，需加一个动作重构 | §6.2、§6.5 | 该文没有把盘上日志格式或部分持久化当作建模对象（没在文中看到，`grep` 只能证明相关词的位置，不能证明「没有」） |

### 5D　etcd raft 的 TLA+ spec 与 trace validation

出处：仓库 <https://github.com/etcd-io/raft> 的 `tla/` 目录，提交 `1c0011d2c6b7a0230f87bad38ad4c6e70d810f9e`（2026-09-30 浅克隆并读 `tla/README.md`）。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 规模 | `etcdraft.tla` 926 行；`Traceetcdraft.tla` 478 行；`MCetcdraft.tla` 119 行；仓库顶层目录非测试 `.go` 6,144 行、`*_test.go` 9,945 行 | `wc -l`（见复核） | 顶层目录口径，不含 `tracker`、`quorum` 等子目录 |
| 怎么绑 | 构建时加 `-tags=with_tla`，实现里在状态转移处埋 trace 点（SendAppendEntries、BecomeLeader 等），输出 NDJSON；`Traceetcdraft.tla` 把 trace 当状态空间约束；`validate.sh` 并行验多个 trace；所有实例须跑在同一台机器、写同一个文件以保持事件次序 | `tla/README.md` | 用 TLC，不是 Apalache |
| 与「日志前缀落盘」直接相关的已知问题 | 「etcdraft assumes atomic persisiting of states in Read action. This, however, may not apply in real-world application. For example, etcd may persist a prefix log when it crashes in the middle of saving data to disks. We will introduce non-deterministic validation method in future to address this issue.」 | README「Known Issues」1 | 即这份 spec 没有建模崩溃时只落一个前缀 |
| 挂钟 | 「It typically takes a few minutes to validate 3000 traces. And the time increases non-linearly after that」；模型检查要跑「at least hours」 | README「Known Issues」2、「Model Checking TLA+ Spec」 | 硬件没写 |

### 5E　Cirstea、Kuppe、Loillier、Merz，「Validating Traces of Distributed Programs Against TLA+ Specifications」（LORIA 与 Microsoft Research）

出处：arXiv 2404.16075 <https://arxiv.org/pdf/2404.16075>，2026-09-30 下载全文读。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 方法 | 把 trace 验证归约为受约束的模型检查，用 TLC；提供 Java 插桩 API、一批 TLA+ 算子、脚本；trace 里只记 spec 变量的更新，可只记部分变量 | 摘要、§4 | — |
| 结果 | 应用到几个分布式程序，「detecting discrepancies between the specifications and the implementations in all cases」；EWD 998（约 500 行实现）发现原子粒度不一致；CCF 与 etcd 已采用 | 摘要、§4.4、结论一节 | 作者自陈：「the technique does not provide formal correctness guarantees, mainly due to the necessary decisions on when to log an event corresponding to an atomic action」 |
| trace 信息量与代价 | 2PC 与 KV 基准：记全变量加事件参数（VEA）时 TLC 探索的状态数约等于 trace 长度（2PC 12 个 RM：trace 长 73，探索 74 个状态）；只记变量（V）时同一例 BFS 一小时超时、DFS 209 个状态；只记事件名（E）时 12 个 RM 为 BFS 2.5M、DFS 27k，16 个 RM 为 BFS 超时、DFS 557k；KV 基准 8 个客户端、20 个键、40 个值时 E 列 BFS 与 DFS 都超时 | Fig. 5（格式「BFS 数 / DFS 数」，单个数表示两者相同） | 一小时超时，硬件没写 |
| 与 Apalache | 「We have exclusively used the explicit-state model checker TLC in our experiments. In principle, symbolic model checkers such as TLA+'s Apalache tool could also be used, but we suspect that the overhead of generating and evaluating constraints would be prohibitive」 | 结论一节 | 作者的猜测，没有测 |
| 哈希或校验和 | 没有（0 命中） | — | — |

### 5F　Apalache（符号有界模型检查）

出处：Apalache 文档源文件 `github.com/apalache-mc/apalache` 的 `docs/src/apalache/{index,features,theory,running,known-issues}.md`（2026-09-30 取 main 分支；最新发布 `v0.62.2`，发布日 2026-08-26，GitHub API 查）；OOPSLA 2019 论文「TLA+ Model Checking Made Symbolic」（Konnov、Kukovec、Tran）<https://ilyasergey.net/CS6213/_static/06-tla/symbolic-tla.pdf>（2026-09-30 下载全文读）。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 做什么 | 把 TLA+ 的验证问题翻成 SMT 约束（默认 Z3，cvc5 实验性），不逐状态枚举；文档说 TLC 是「mature」而 Apalache「is still experimental」 | index.md、theory.md | — |
| 前提 | 所有 spec 参数固定且有限；求值过程里的数据结构有限；「Only finite executions of bounded length are analyzed」；`--length` 默认 10；可查状态不变式、动作不变式、trace 不变式、归纳不变式，时序性质经 liveness-to-safety 转换 | index.md、running.md、features.md | — |
| 不支持或受限 | `Seq(S)`（无界序列，用 `Gen` 造有界序列代替）；递归算子与递归函数（0.23.1 之后不支持，用 `ApaFoldSet`、`ApaFoldSeqLeft`）；无界量词 `\A x : p`；非常量界的整数区间 `a..b`「the current encoding fails」；`Cardinality` 产生 O(n²) 约束 | features.md、known-issues.md | — |
| 性能（OOPSLA 2019） | 一核一实验，Xeon Gold 6130（2 颗、每颗 16 核）、192 GB，超时 23 小时；对类 Paxos 算法「Both TLC and our tool get stuck after 10-15 steps」；有界模型检查时 Z3「dramatically slows down」在 10 到 15 步之后，「especially noticeable on the specifications of fault-tolerant distributed algorithms such as Paxos and Raft」；检查归纳不变式相对 TLC 优势明显，有界模型检查时优势小 | 论文 §13.1、§13.3、§13.4 | 基准是 2019 年的分布式算法，不是日志重放 |
| 提交协议的具体数（OOPSLA 2019，两阶段提交 TwoPhase-7，7 个资源管理器） | 检查归纳不变式（Table 3）：Apalache 4 秒、608 MB；TLC 2 小时 44 分、探索 1.14M 状态。有界模型检查证明安全性（Table 5，性质成立）：Apalache 44 分钟（深度 10）；TLC 13 秒（探索 296K 状态，穷举到深度 23）。TwoPhase-3：Apalache 1 分 13 秒，TLC 1 秒 | Table 3、Table 5 | 同上硬件；「TO」= 23 小时超时；Apalache 的深度小于 TLC 时它覆盖的状态空间更小（论文原话） |
| 与实现的绑定 | 没查到：CCF 论文只在参考文献里提 Apalache；MongoDB 论文 0 命中；Cirstea 等只用 TLC 并写「we suspect」Apalache 的开销会过高（没测） | 见 5B、5C、5E | 见反证清单 |

### 5G　Azure Cosmos DB 的 TLA+ 公开 spec

出处：<https://github.com/Azure/azure-cosmos-tla> 的 `README.md`（2026-09-30 取）。

| 项 | 事实 | 口径 |
|---|---|---|
| 是什么 | 五个一致性级别（strong、bounded staleness、session、consistent prefix、eventual）的 TLA+ 与 PlusCal spec；README 说 Cosmos DB 工程团队「heavily relies on TLA+」；spec 只模型化客户端可见行为，「We refrain from modeling the Cosmos DB internals」 | 只有 README，spec 大小、检查时长、与实现的绑定：README 里没写（没查到，没读三个子目录） |

### 5H　相邻一笔：SquirrelFS（OSDI 2024，Rust typestate 加 Alloy 模型）

出处：arXiv 2406.09649 <https://arxiv.org/pdf/2406.09649>（LeBlanc、Taylor、Bornholt、Chidambaram），2026-09-30 下载全文读。不在题面五家里，因为它同时用了「独立的有界模型」与「Rust 里的检查」，放在这里当一份实例。

| 项 | 事实 | 出处 | 口径 |
|---|---|---|---|
| 崩溃协议怎么查 | 手写 Alloy 模型（Synchronous Soft Updates 协议）；模型检查的界：两个操作（可并发）、10 个持久对象、至多 30 步；不变式含四项一致性（合法链接计数、无指向未初始化对象的指针、已释放对象不含指针、rename 指针无环）；Alloy 找到两处设计问题（rename 后崩溃加目录项释放会让非法目录项重现，需加恢复转移；`.` 与 `..` 目录项不应持久化） | §4.2、§5.7 | 与实现同时开发，「Many transitions in the model could be translated directly into Rust typestate transitions」 |
| 挂钟与限制 | 「the formulas representing a large model can take days or weeks to solve」；为此写了并行运行多个独立实例的工具，把大谓词拆成小检查；Alloy 允许未被提及的状态任意变化，须写 frame condition，作者写了一个语法检查器查每个转移是否提及所有可变结构 | §4.3 | 硬件没查到 |
| 与实现的绑定 | 模型与 Rust 代码之间没有机械绑定：靠 Rust 编译器的 typestate 检查捕捉「从模型翻成 Rust 时的错误」，再用 Chipmunk 测崩溃（约 24 小时，未发现顺序类崩溃 bug，在未受检代码里发现 4 个：3 个在重建易失数据结构、1 个是 cache line flush 用错地址） | §4.1、§5.7 | 代码规模 7.5K 行，编译约 10 秒 |
| 校验和 | 全文没有校验和建模（相关词唯一命中是「不对名字做 hash」） | — | — |
| 论文对照的验证成本 | 「the developer has to write 7–13 lines of proof for every line of code」；BilbyFS 13k 行证明对 1k 行实现；VeriBetrKV 45K 行证明对 6k 行实现；FSCQ 约 11 小时、VeriBetrKV 1.8 小时（并行 10 分钟） | §1、§5.6 | 转述别家，没核原文 |
| **与本工程的一处差异** | 它的崩溃协议是持久内存上的软更新（对象状态与 store fence），Alloy 界是「两操作、十对象、三十步」；本工程是块设备上的 journal 与 checkpoint 发布，D13 已定项 4 已有整写子集的全量枚举，模型检查要补的是枚举取不到的那几维 | — | 差异：崩溃协议与已有枚举 |

## 6　反证清单（「没查到有人这样做」，各写查了哪几处、怎么查的）

口径：这些只说「这几处没查到」，不说「不存在」；举证责任的说法见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」。

| 编号 | 没查到什么 | 查了哪几处、怎么查 |
|---|---|---|
| F-1 | Kani 用在文件系统、日志、KV 存储或崩溃一致性代码上的公开案例 | ① ShardStore SOSP 2021 全文文本 `grep -a -c -i kani` 命中 0；② Kani 论文（2026）Table 1 的 8 个项目：Firecracker、s2n-quic、Hifitime、verify-rust-std、zerocopy、lading、x86_64、rust-sel4，都不是存储或文件系统（表自称子集）；③ WebSearch 两次（问 Kani 加存储或文件系统或日志）与 GitHub 仓库搜索两次（`api.github.com/search/repositories`，关键词 kani 加 file system 或 journal），结果是无关仓库，搜索不穷举 |
| F-2 | TLA+ 或 Apalache 用在「写前日志 / journal 重放 / 崩溃恢复幂等」上的、有规模与挂钟数字的成熟公开案例 | ① WebSearch 两次（WAL 恢复与幂等，文件系统 journal）：结果是小型 GitHub issue、一篇 Isabelle 的论文、Ferrite 与 SquirrelFS（后者用 Alloy）；② 读了 AWS 2014、MongoDB 2020、CCF、etcd raft、Cosmos DB 五处，它们的模型是分布式协议（复制、共识、一致性级别），没有一处以盘上日志格式或重放为对象；③ etcd raft 的 spec 自陈假设日志原子持久化，崩溃只落一个前缀是「Known Issues」（见 5D） |
| F-3 | Apalache 与实现的 trace 对拍的公开案例 | CCF（Apalache 只在参考文献里出现 1 次）、MongoDB（0 次）、Cirstea 等（只用 TLC，「we suspect」Apalache 开销过高，没测）、etcd（TLC）；Apalache 文档里没查到 trace 对拍的说明（只读了 index、features、theory、running、known-issues 五页） |
| F-4 | TLA+ 案例里对 CRC 或校验和建模的 | AWS 2014、MongoDB、CCF、Cirstea 四篇全文 `grep -a -c -i -E 'checksum\|crc\b\|hash\|digest'`：1、0、3、0 处命中，逐处看过，均与建模无关；Cosmos DB README 与 etcd README 没有相关内容 |
| F-5 | Yggdrasil 与 FSCQ（2015）对校验和或哈希建模 | 两篇全文命中 0；对照：DFSCQ 有 Hash 操作码（36 处命中），PoWER 有 CRC 公理，Verus 持久日志把 CRC 库 spec 标 trusted |
| F-6 | FSCQ 或 DFSCQ 官方给出「证明行数比实现行数」 | 两篇论文都写明证明与代码交织未拆（FSCQ §6）；拆开的数只在同组后续 DaisyNFS（GoJournal 29,000 证明对 1,419 代码）里有 |
| F-7 | Verus 之外，在 Rust 实现上做崩溃一致性证明且不改写代码的 | 没有查到。Verus 案例（持久日志、CapybaraKV）与 SquirrelFS 都要求按各自方言或 typestate 写实现；本表只查了这两条线加 Kani，没系统查别的 Rust 验证器（Prusti、Creusot、Aeneas 等，只在 Kani 与 ShardStore 论文的相关工作里见到名字） |

## 7　没做什么

- 没论证任何一条路适不适合本工程，没推荐；每张表最后一行的「差异」只写做法层面的差异，不是评价。
- 没在本项目里验证任何一条：没跑 Kani、Verus、TLC、Apalache、Z3 或 Coq；没编译 `crates/`；所有数都是别家自报或我读、数出来的。
- 没读的原文：Yggdrasil 源码仓；DFSCQ 引的 Wang 硕士论文（Hash 操作码的细节）；VeriBetrKV（OSDI 2020）原文（只见 PoWER 与 SquirrelFS 转述）；GoJournal 与 Perennial 原文（数取自 DaisyNFS 的 Figure 12）；Verus 的 `verified-storage` 仓库；AWS 2025 CACM 文章（403）；Kani 官方文档只读了 `rust-feature-support`、`tutorial-loop-unwinding`、`debugging-slow-proofs`、`tool-comparison`、`overrides` 五页与首页；Cosmos DB 三个子目录的 spec。
- 抽取 PDF 文本用的是我装在草稿目录的 pypdf（`/tmp/claude-1000/m3-r1/formal-prior-art/libs/`），表格靠文本抽取还原：Verus Figure 9 的列头顺序、Yggdrasil Figure 5 的列对应，是从行内数字（总数与比值吻合）推的，已在对应格标注。
- 没查到的格（已在表里写「没查到」处）：Kani 论文 Table 1 各项目的 CI 硬件；ShardStore 的单次运行挂钟与硬件；DFSCQ 的证明检查时长；MongoDB 论文的 TLC 硬件；etcd 的 trace 验证硬件；Cosmos DB spec 的规模与检查时长；Kani 对 `std::fs`、CRC、AEAD 的官方处理。
- 没数今天 Firecracker 仓库里 harness 的实际个数（用的是论文与 2023 博客的数，两者不同）。
- 没做本工程实现侧的任何事实调查（`Recover` 的行数、依赖、读写抽象等归 1-1 事实表）。

**什么现象会推翻本表**：任一出处的原文与我抄的数不符（复核命令与 PDF 哈希见下）；FSCQ 仓库用别的口径再数得出不同的行数；Kani 或 Apalache 的官方文档在取得日期之后改了支持范围（文档是活的，以取得日期 2026-09-30 为准）。

## 8　复核（这一次真跑的原样输出）

命令脚本 `/tmp/claude-1000/m3-r1/formal-prior-art/verify.sh`，输出 `/tmp/claude-1000/m3-r1/formal-prior-art/verify.out`。下载的 PDF 与抽取文本都在同目录（`*.pdf`、`*.txt`），仓库副本在 `fscq-src/`、`etcdraft-src/`。

```
$ grep -a -c -i kani ss.txt        # ShardStore SOSP 2021 全文
0
$ for f in ygg fscq dfscq: grep -a -c -i -E 'checksum|hash|crc'
ygg 0
fscq 0
dfscq 36
$ for f in aws15 xm ccf trace sq: grep -a -c -i -E 'checksum|crc\b|hash|digest'
aws15 1
xm 0
ccf 3
trace 0
sq 1
$ grep -a -c -i apalache ccf.txt xm.txt trace.txt
ccf 1
xm 0
trace 2
$ FSCQ 仓库 count_coq.py（在 fscq-src 里跑）
2c7ef9c268fd79a81b26b44ef720f8e6a1e938a0 2022-10-21
files 98 nonblank_lines 83866 blank 10512 lines_from_Proof_to_Qed 51509 other_nonblank 32357 Admitted 2
$ etcd raft tla 行数
  926 etcdraft.tla
  478 Traceetcdraft.tla
  119 MCetcdraft.tla
 1523 total
$ etcd raft 顶层 .go 行数（非测试 / 测试）
1c0011d2c6b7a0230f87bad38ad4c6e70d810f9e 2026-09-22
6144
9945
$ sha256sum 下载的论文 PDF
7ded45e79a4b78aad7f8565dff41d83e1049c22cdcf845d42a1c4f7f7aa5bd16  ygg.pdf
24a77431800f9d771a97d7647039b3880b7aebdaa5814c78706786a3048c3613  fscq.pdf
7327ef67546b68e1695d8783786c4f4d1294f39f53f7287e2f61f592e4d0800b  dfscq.pdf
231eb1ed39613cc60a73cc3394f97e10f72349034c09da2f68de7782ee1f72d4  ss2.pdf
ab4b1f2a0f5fbc26ee59cc0cff30c271e15366e0e926df448cf8e506a5ea8144  kani.pdf
8d984c3ef389d468721627dd84ba8309247bd548ebecbaeb1df5c444916da3fc  power.pdf
f6ff8fa1b3e1bc5322e3b4c5a47b95dfd8c375c5220f7585d70318c261a772ff  verus.pdf
aedaa6702901f1b4d2e40788247b1b3e7e995d55712561983d1399d9c4d6d8d4  xm.pdf
c9e5a47fb1066fe97246f6e96b39f8790d5c044e5ba756c73e7db050134be1b4  ccf.pdf
229abdc0dac233c168dc086c9ae27593b8f87cadb46d65d2ee8f8750743e1965  trace.pdf
e4a3f8a897c1d8b3368b022fa7f1a18a2ea6abee673c7e05339729d35527a5f2  aws15.pdf
7dc5fa793d1fc98478b9dbf3b614b86ddb3e1fff18d4d5119d65f2327215ea07  daisy.pdf
8064a97eeb7c950c264cf6747c3b226d170beef4705628d6602be502dc7e7bf0  sym.pdf
a2c3ee31debb92aa86322567cc55ad4d8b3e8526752c4e7bdb5239254671c613  sq.pdf
```
