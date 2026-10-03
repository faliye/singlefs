# m3-code-r1 云端正推腿报告（Z1：条款对实现、条款对测试，含 ①②③ 三问）

日期 2026-10-01。立场：正推。只读与推导，没编没跑测试；唯一跑过的非读命令是 `/tmp` 草稿目录里的一段 python ctypes 调 `fallocate`（第 A3 节第 7 行）。
被判对象读冻结副本 `/tmp/claude-1000/m3-code-r1/tree/`，引代码写的是副本里的行号；引 kb 写 kb 文件自己的行号（下面每处都 `grep -nF` 过一次）。

## 0　被读对象与背景材料对上了没有

命令与原样输出：

```
$ diff <(sha256sum 六份 kb 文件取第一列) <(kb-sha256.txt 取第一列) && echo KB_SAME
KB_SAME
$ (cd /tmp/claude-1000/m3-code-r1/tree && sha256sum -c crates-sha256.txt | 取末列 | sort | uniq -c)
     12 OK
```

kb 六份（checks-owed、D16、D17、D18、D22、D23）与开工快照逐字节相同，副本里 12 个被判文件与登记的 sha256 相同。仓里主工作区的 `crates/` 还没打这批补丁，行号与副本对不上，所以引代码一律写副本的绝对路径加副本行号（交回闸的 cite-check 只认仓根，绝对路径照样能判）；引 kb 与规则写仓内路径。

## A1　C585（题面第 1 行）：条款对实现

条款落点只有 D18 已定项 11 一条（`.claude/kb/decisions/18-块里携带什么信息.md` 第 314、323、337–340 行），加欠账行 C585（`.claude/kb/checks-owed.md` 第 459 行）与 D16 已定项 1 的「根槽这一次读坏」一行（`.claude/kb/decisions/16-发布语义.md` 第 37 行）。逐句比：

| # | 条款句 | 实现 | 判定 |
|---|---|---|---|
| 1 | 「用户定改法 C——落后且缺现行那一版的单元，或现行那一版的根写在这块盘上而那一槽里不是它，才拒，四个入口同一判——里程碑三实现，被攻过零轮」（`.claude/kb/decisions/18-块里携带什么信息.md:323`） | 共用判法 `what_a_device_behind_the_version_lacks`：先调缺单元那一判，单元都在再看根。四个入口经 `devices_behind_the_current_version` 调它（「pub(crate) fn devices_behind_the_current_version<Reader: PoolReader + ?Sized>(」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3464`）），本体在「fn what_a_device_behind_the_version_lacks<Reader: PoolReader + ?Sized>(」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3350`）；会话发布另在 `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:287` 调它 | 一致 |
| 2 | 「挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前核三样」（`.claude/kb/decisions/18-块里携带什么信息.md:314`） | 抬 F、卸载、回退三条走 `caller_inputs_agreeing_with_the_disk`，落后支在「devices_behind_the_current_version(devices, &parameters_of_the_pool, current);」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3834`）；缺根那一半只读盘至多两次（见 3 与 5）。副本的 `mounted_session.rs` 在补丁里只有文档注释（`m3-code-r1-diff.patch` 第 930–964 行都是 `//!` 与 `///` 行） | 一致 |
| 3 | 「现行那一版的根写在这块盘上」：条款没定义「写在」是「该写的槽归这块盘」还是「实际写过」 | 槽由 `target_for_publish(txg)` 算、盘取区域归属表：「if device_of_the_ring_slot != device {」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3418`）不归这块盘就交回 `None`、不读盘。「实际写过」由调用方保证：四个入口恒传 `Written`，依据是会话每次发布成功才换 `current`（「*current_file_version = published;」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:312`）） | 一致（实现取前一种读法；条款没定义） |
| 4 | 「那一槽里不是它」 | 读到的与现行那一版的 (实例, txg) 比：「if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3429`）；不等有三种：读不出、自证不过（`Bad`）、自证得过而是另一条根（「RootRingSlotReading::SelfVerified(another_root) => {」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3433`）） | 一致；条款只写了「不是它」，三种都算是实现的读法 |
| 5 | 条款 323 没写重读。D16 那一行写的是：「挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜」（`.claude/kb/decisions/16-发布语义.md:37`） | 读一次、不是它就重读一次、仍不是才交回两次读数：「let first_reading = reading_other_than_the_version_root()?;」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3443`）、「let reread = reading_other_than_the_version_root()?;」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3444`） | **条款没说而代码做了**：借用 D16 那一行的 R = 1，没把被借规则的参数绑死（见 B 节 K7） |
| 6 | 「交进来的盘有一块不带就在取号之前、任何写之前拒可写」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）；改法 C 一句只写「四个入口」（第 323 行，同第 1 行） | 可写挂载也过缺根那一半，新成员「BehindTheSelectedVersionAndItsRootNotInItsRingSlot {」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:707`），传值在 3993–4003 行（`Written` 或 `NotWritten`） | **条款没说而代码做了**（三问 ① 专答） |
| 7 | 「只落后、单元都在的不拒，只剩一槽自证、那一槽停在上一次轮换的盘由此不误拒」（`.claude/kb/decisions/18-块里携带什么信息.md:314`） | 缺根那一半拒的恰是「只落后、单元都在」里根槽不是它的那一形；注释自己写明放行的只有「只落后、单元都在、根槽里是它」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3349`） | **冲突**（与 314 字面）；与 323 一致；条款自己前后没对上，见第 8 行 |
| 8 | 并排：第 314 行「现行那一版有单元在这块盘上它的落点读不出或字节与这个进程验得过的那一份不同，才拒」（`.claude/kb/decisions/18-块里携带什么信息.md:314`） 对 第 323 行第 1 行那句改法 C | 实现按 323 | 条款内部不一致：314 还是旧判法（「才拒」是必要条件），323 与 C585 行写了改法 C，欠账行第 340 行仍挂「改法 C 的实现与会红的检查（里程碑三）」（`.claude/kb/decisions/18-块里携带什么信息.md:340`）。以 323 为准（用户 2026-09-28 选 C，`records/2026-09-24-里程碑二收尾调度.md` 第 317 行）；314 与 340 要主 agent 同步，不是代码的错 |

## A2　C586（题面第 2 行）：条款对实现

| # | 条款句 | 实现 | 判定 |
|---|---|---|---|
| 1 | 「inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去」（`.claude/kb/decisions/23-journal的角色与格式.md:383`） | 树 ID 一半：「.fold(current.root.tree_identifier_watermark, u64::max);」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:5386`），环里读得出的根并进内存里现行那一版的；这一批没改这一行（`m3-code-r1-diff.patch` 的 mount.rs 段没有 5386 附近的 hunk） | 一致 |
| 2 | 「补一条回退时树 ID 水位只在内存里的现行那一版上更高的用例」（`.claude/kb/checks-owed.md:460`） | 新用例 `the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it`，断言「回退那次发布的树 ID 水位 = max(现行那一版内存里的 19, 环里读得出的根的 11)」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:1675`） | 一致 |
| 3 | 欠账行要的变异行，名字是回退的树 ID 水位只取环里读得出的根（`.claude/kb/checks-owed.md:460`） | `/tmp/claude-1000/m3-code-r1/tree/crates/mutations.tsv` 副本第 1483 行「C586：回退那次的树 ID 水位只取环里读得出的根，不与内存里现行那一版的取 max」，替换文 `.fold(0, u64::max)`（`m3-code-r1-diff.patch` 第 27 行） | 一致 |
| 4 | 「litmus 那一处注释改成现在的路径」（`.claude/kb/checks-owed.md:460`） | 副本第 14 行：「块层那一半在录制流上判（crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs）」（`/tmp/claude-1000/m3-code-r1/tree/litmus/publish-returns-after-the-rotation-barrier.litmus:14`）；该文件在副本里存在（`ls` 命中），第 10 行点名的 harness 档那份也存在 | 一致 |

C586 这一笔只有「内存那一操作数」有新用例钉；max 的另一个操作数（环里读得出的根）新用例没钉（第 B 节 L2）。

## A3　打洞（题面第 3 行）：条款对实现

打洞没有 kb 条款：全仓 kb 里 `打洞`、`PUNCH_HOLE`、`fallocate` 的命中只有 vm-harness 与实验页里别的用途（`grep -rn '打洞\|PUNCH_HOLE\|fallocate' .claude/kb --include=*.md` 命中 7 行，没有一行在说文件后端写零）。所以下面的「条款」是 D17、D22、两笔欠账与用户原话（记录文件）。

| # | 条款句 | 实现 | 判定 |
|---|---|---|---|
| 1 | 「两个后端：普通文件与 `O_DIRECT` 打开的裸设备」（`.claude/kb/decisions/17-实现分层与第三方管道.md:82`） | 文件后端 `write_zeroes_at` 改打洞，`O_DIRECT` 后端那一个没动：副本文件头第 16 行写明「`O_DIRECT` 后端照旧真写 0」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:16`），diff 里没有动它的 hunk | 一致 |
| 2 | 「journal 环也整段写 0：两者都由 mkfs 自己经块设备的写零动作发出」（`.claude/kb/decisions/22-单元原子性怎么合成.md:176`）；「每块盘 4 步、两块盘共 8 步」（同行） | 录制层照旧记一步：「self.inner.write_zeroes_at(offset, length)?;」之后才推一步（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/src/lib.rs:368`），步数与长度不随后端落地方式变；唯一的生产调用方 `make_filesystem.rs` 第 497、499 行没动 | 一致（读代码推的，没重跑 mkfs 步数用例） |
| 3 | 「在一块装过上一份 singlefs 的盘上重 mkfs」（`.claude/kb/decisions/22-单元原子性怎么合成.md:184`）：整段写 0 要真让旧根读回 0 | 打洞后读回全 0：核心单测先填脏再打洞再读回（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:823`）；mkfs 层没有「在脏镜像上重 mkfs」的用例（新 harness 用例建的是新镜像，读回全 0 本来就成立，分不出清过与没写过） | 一致（块层有钉，mkfs 层没钉，见 B 节 P2） |
| 4 | 用户 2026-09-30、10-01 定只在不支持时退（`records/2026-09-30-里程碑三开工第一轮.md:128`：「别的错照报」）；「ENOSPC 照报错，补丁不改」（`records/2026-09-30-里程碑三开工第一轮.md:184`） | 分路：`EOPNOTSUPP` 或 `ENOSYS` 走「=> HolePunchFailure::PunchingUnsupported,」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:173`）、别的与没有 errno 的全走 `OtherFailure`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:174`）；别的错在「HolePunchFailure::OtherFailure => Err(BlockDeviceError::InputOutput(punch_error)),」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:457`）交回，不写 0；`ENOSPC` 落在 `Some(_)` 里 | 一致 |
| 5 | 欠账 C603：「查实的浪费形态逐个立检查（第一形：往测试镜像真写整段零）」（`.claude/kb/checks-owed.md:477`） | 第一形的检查：新 harness 用例量本线程 `wchar` 等于 mkfs 真写的字节，且每盘退回写零 0 次（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs:92`、`:104`）；核心单测同样量 `wchar`。写零唯一的生产调用方是 mkfs，所以这一形被盖住 | 一致 |
| 6 | C603 的另一半：「每类测试与每道重活跑完报内存峰值与写盘字节数、登记成表」（`.claude/kb/checks-owed.md:477`）；C486：「每次都跑那一档的挂钟超过写死的上界就判红，上界随这两个数定」（`.claude/kb/checks-owed.md:397`）与「冷挂载的耗时要有口径与上界」（`.claude/kb/checks-owed.md:397`） | 这一批没有对应的代码：只做了「写零真写」那一半（题面第 3 行与第 13 行也这么限定）；C486 的 ① 上界与 ② 冷挂载口径、C603 的写盘字节登记表都没动 | 条款说了而代码没做（欠账仍开着，不是这一批的出口） |
| 7 | 块设备契约：「写零：把 `[offset, offset + length)` 整段变成 0。偏移与长度同样要按物理块对齐、不越过设备末尾。」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:127`）——长度 0 满足这句 | 旧路径：「while written < length {」对长度 0 一次都不循环、交回 `Ok`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:349`）；新路径把长度原样交给 `fallocate`（「libc::FALLOC_FL_PUNCH_HOLE」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:381`）、「length_for_the_system_call,」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:383`）），内核对长度 0 报 `EINVAL`，`EINVAL` 走 `OtherFailure`，交回 `InputOutput` | **冲突**：契约允许的长度 0，旧实现 `Ok`，新实现 `Err`；稀疏盘 `zero_fill` 对长度 0 也是 `Ok`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:85`），稀疏盘与文件后端的对拍请求里没有长度 0（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs:25`） |

第 7 行的复核命令与原样输出（`/tmp` 草稿目录，镜像放在 ext4 的 `/` 上，调的是同一个 `fallocate(2)`、同一组标志 `PUNCH_HOLE | KEEP_SIZE` = 3）：

```
$ df -T /tmp/claude-1000/m3-code-r1-sonnet | tail -1
/dev/nvme0n1p2 ext4 3843466296 1826356116 1821798052  51% /
$ python3 (ctypes 调 libc.fallocate(fd, 3, off, len)，64 KiB 文件先填满 0xC3)
off=0 len=0 -> rc=-1 errno=EINVAL
off=0 len=512 -> rc=0 errno=0
off=512 len=0 -> rc=-1 errno=EINVAL
off=65536 len=512 -> rc=0 errno=0
off=0 len=131072 -> rc=0 errno=0
```

长度 0 在这台机器的 ext4 上确实报 `EINVAL`；越过文件末尾的两格返回 0，所以越界由 `check_request`（「self.check_request(offset, length)?;」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:428`））挡，不由内核挡。长度 0 的调用方今天没有（`make_filesystem.rs` 两处都是非零整段），所以这条冲突不会被现有流量踩到，但契约字面允许它。

其余几条：

- unsafe 与登记表：条款是门禁表头写的判法「函数里 unsafe 关键字恰好 1 处、上方连着的 // 注释里有一行 // SAFETY:，且那段注释写出依赖的前提（提到文件描述符、偏移、长度）。」（`.claude/gate.d/unsafe-allowances.tsv:15`）。实现：core 根「#![deny(unsafe_code)]」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/lib.rs:10`），放开挂在「#[allow(」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:361`）下的 `punch_hole_keeping_the_file_length` 一个函数，`unsafe` 关键字全 crate 一处（「let return_code = unsafe {」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:378`）），上方 `// SAFETY:` 起于第 374 行，写了描述符、偏移、长度三样。登记表第 19 行函数名与文件与这三处一致。判：一致。每条前提成不成立归 Z3 ⑤，这里不判。
- 依赖：`libc = "0.2"`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/Cargo.toml:10`），锁文件里 `libc` 是 0.2.189（`Cargo.lock:355`），补丁只给 core 的依赖表加一行：一致。
- 14 行变异：副本 `/tmp/claude-1000/m3-code-r1/tree/crates/mutations.tsv` 第 1477–1482 行是 C585 的 6 行，第 1483 行 C586，第 1484–1490 行打洞 7 行，数目与题面一致（`m3-code-r1-diff.patch` 第 21–34 行）。每行点名的用例见 B 节。
- 注释没跟上的三处（不是条款冲突，只是旧句还在）：文件头第 7 行「一次调用覆盖 `[offset, offset + length)` 一整段，真设备上对应 WRITE ZEROES。」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:7`）；「后端把一段写零拆成多大的块交给底层：4 MiB。」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:106`）现在只管 `O_DIRECT` 后端与退路；`model_comparison.rs` 第 780 行的注释还写「又缺它的单元」，没跟 C585（该文件不在被判清单里）。
- 持久：文件头第 14–15 行说打洞的持久靠 `fdatasync` 与 `ext4_punch_hole` 记进日志事务，依据是「本机 6.17 内核源码」。这台机器上没有内核 `.c` 源码（`ls /usr/src` 只有 headers 目录），我核不了这句；xfs、btrfs、tmpfs 上的持久代码里一句也没写。块设备契约写的是「持久由后面的屏障保证」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:134`），打洞走的 `barrier()` 是 `sync_data`（「self.file.sync_data().map_err(BlockDeviceError::InputOutput)」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:462`））；这在 ext4 上成不成立靠那句没核的源码引用。判：复核不了。

## B　条款对测试：新用例各钉条款里哪一句，条款里有而没人钉的逐句列出

### B1　每条新用例钉的

文件缩写：E = `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`；U = `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`；M = `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs`；B = `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs`（`mod tests`）。

| 用例（行号） | 钉住的条款句 | 钉得对不对 |
|---|---|---|
| E:886 会话发布、E:894 正常卸载、E:900 抬 F、E:908 回退，各一条「P 的根住的那块盘换成切在根槽 FUA 之前」 | 改法 C 的缺根那一支（A1 第 1 行）与「在任何写之前拒、盘面逐扇区不变」（欠账行：「断言会话发布在任何写之前拒、盘面逐扇区不变；改回今天的判法它必须红」（`.claude/kb/checks-owed.md:459`））：断言点名的盘、`lacking` 整体相等、两块盘逐字节不变（「"{entry:?}：两块盘逐字节不变"」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:876`））、会话现行版本不动 | 钉的是欠账行要的那一形。`lacking` 里两次读数都是 `Bad(NotSelfVerified)`（E:843、E:846），即那一槽 P 之前没写过的全 0 槽；`AnotherRoot` 与 `Unreadable` 两种读数没有用例（K3、K4） |
| E:918 切在根槽 FUA 之后，会话照常发布 | 「只落后、单元都在……不误拒」里根槽是它的那一形（A1 第 7 行）；做完之后 checker 不红 | 钉了会话一个入口；其余三个入口没有这一形（K5） |
| E:947 切在根槽之前，可写挂载照常挂上 | 可写挂载的 `NotWritten`：所选那一版是施加 P 的记录得来的，不核缺根（A1 第 6 行，条款没写的那一句）；断言所选那一版是 P（「"所选那一版是施加 P 的记录得来的 P"」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:961`）） | 钉的是「施加过记录就不核」；可写挂载「核了并拒」那一支没有用例（K10） |
| E:462 Y2-c（原有，没改语义） | 「只剩一槽自证、那一槽停在上一次轮换的盘由此不误拒」（`.claude/kb/decisions/18-块里携带什么信息.md:314`） | 这条用例里被写坏的是盘 1，而现行版本 txg 5 的根槽在区域 2、归属表第三项是盘 0（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/common/mod.rs:49`：「region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],」；`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/root_ring.rs:114` 区域号取 txg 对 3 的余数）：缺根那一半对盘 1 在 `device_of_the_ring_slot != device` 那句就返回 `None`、没读根槽。改法 C 之后「只剩一槽自证而根槽恰在这块盘」那一形没有用例（K5 的另一半） |
| U:1592 回退的树 ID 水位 | D23 水位一句里「cur 那一版内存里的水位」那一个操作数（A2 第 1、2 行） | 钉对；环里读得出的那一个操作数没钉（L2） |
| M:38 mkfs 在两块镜像文件上 | C603 第一形（A3 第 5 行）、D22 8.3「journal 环也整段写 0」经写零动作发出（A3 第 2 行）、fs-design 第 4 条的计数（「punched_calls,」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs:111`）对录制层记的写零步数） | 钉对；它不读回内容（新镜像本来全 0），分不出「清过」与「没写过」；「清过」由 B:823 在块层钉（P2） |
| B:823 打洞：非块边界的两段读回全 0、邻居不动、长度不变、`wchar` 为 0 | D22 8.3 的「整段写 0」效果；`KEEP_SIZE` | 钉对（先填脏再清，能分出清过） |
| B:891 强制「不支持」走退路 | 用户定「不支持才退」的退这一支；fs-design 第 2 条 | 钉对；开关造的是 `EOPNOTSUPP`，经 `classify_hole_punch_failure` 走 |
| B:971 强制 `EIO` 照报 | 同一句的「别的错照报」这一支 | 钉对 |
| B:1029 errno 分路表 | 「ENOSPC 照报错」（A3 第 4 行）、`ENOSYS` 退 | 钉的是纯函数；`ENOSPC`、`ENOSYS` 经 `write_zeroes_at` 的整条路没有用例（P3） |

### B2　条款里有、没有任何新用例（也没有我查得到的旧用例）钉的句子

编号 K 管 C585，L 管 C586，P 管打洞。「没有」的意思：我读了这两份 diff 与副本里 entries 文件、`grep` 过 `RootNotInItsRingSlot` 等名字，没找到；没逐份翻全仓用例，凡全仓别处可能有的，写明。

- **K3　「那一槽里不是它」里自证得过的另一条根。** 新用例的盘面里，P 是 txg 6，根槽是区域 0 的第 2 槽（`slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/root_ring.rs:115`）），环深下限 S ≥ 4（「每区槽数 S ≥ k_tol + 2 = 4」（`.claude/kb/decisions/16-发布语义.md:44`）），txg 6 在转第一圈之前，那一槽只有 mkfs 清的全 0，所以读数只会是 `NotSelfVerified`。`SelfVerified(another_root)` 那一臂（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3433`）没有用例走到。
- **K4　读不出（`Unreadable`）。** 没有往那一槽注入读错的用例。
- **K5　「根槽里是它」不误拒，在抬 F、卸载、回退三个入口，以及「只剩一槽自证而根槽恰在这块盘」。** 只有 E:918 钉会话一个入口。
- **K6　根槽不在被判的这块盘上就不核、不读。** 四条拒用例与 E:918 都是换根槽所在的盘；没有「换的是另一块盘、根槽在对面」的格。该句的守卫是 `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3418`，变异表 14 行里没有一行改它。
- **K7　重读一次。** 没有「第一次读坏、重读读得出」的用例；去掉重读不会让任何新用例变红（实现员自己写在「被攻过零轮」第 2 条）。
- **K10　可写挂载上缺根那一支「核了并拒」。** 新成员 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:707`）在 `/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness`、`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-checker-tier`、`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-checker` 的 `.rs` 里 `grep` 零命中，没有任何用例构造它；因为所选那一版就是择到的根时它的槽刚在读阶段读得出，要两次紧接着的读都坏才走到（A1 第 6 行与三问 ①）。
- **K11　归档的两份大扫描。** D18 第 338 行写「改法 C 只在代码三方第三轮攻方的模型上量过」（`.claude/kb/decisions/18-块里携带什么信息.md:338`），其中「那 58 格拒、一槽坏的 1728 串不误拒」（`.claude/kb/decisions/18-块里携带什么信息.md:338`）；实现员把攻方的扫描包搬进草稿副本跑过（报告里 `STALE accepted=254 refused=1101 flagged=0`），没进仓：入库的只有 E 里 6 条单点用例。这两份数我复核不了（没编没跑）。
- **L2　`max` 的另一个操作数。** U:1592 的环里读得出的根都小于内存里的（11 对 19）；「环里读得出的根更高、内存里更低」那一格，我只翻了 `rollback_by_a_forward_publish.rs` 第 434–435 行与第 912 行两处断言（比的是回退版与第三版的水位相等），没有逐份核它们是不是钉的这个操作数。
- **L3　读次序假设。** U:1592 用「第 3 次读」注入读错（`&[3],`，第 1596 行附近），写死了「判候选 2 次、算水位 1 次」的读次序；任何一次改动让这几个槽多读或少读，用例要么红在 `expect`、要么静默变成没钉水位。这是用例的脆，不是条款句。
- **P2　脏镜像上重 mkfs。** 见 B1 的 M:38 行。
- **P3　`ENOSPC`、`ENOSYS` 经整条 `write_zeroes_at` 的路。** 只钉了纯函数。
- **P4　`O_DIRECT` 后端仍真写 0。** 本批没有用例钉它；文件头说它被 checker 档 `device_log.rs` 的折叠依赖（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:16`），那一档是重型测试，我没跑，也没核它有没有钉。
- **P5　写零长度 0。** 没有用例（A3 第 7 行的冲突）。
- **P6　打洞的持久。** 没有用例，也不可能在单测里造掉电；见 A3 末段。

## C　三问

### C①　可写挂载按「`effective_root` 与 `chosen_root` 是不是同一条」传 `Written` / `NotWritten`：与 D18 已定项 11 的字面合不合，条款要不要补句

**判：不合，也没有一条字面可以读出这个传法；条款要补一句，补哪一句归用户定（判据表第 5 行「Z1 ① 那一句，交用户」）。**

1. 字面只到四个入口。D18 把「入口」定义成挂着之后收盘表那四个（「挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前核三样」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）），改法 C 那一句写的是「四个入口同一判」（`.claude/kb/decisions/18-块里携带什么信息.md:323`），欠账行同样是「才拒，四个入口同一判」（`.claude/kb/checks-owed.md:459`）。可写挂载的落后支在 314 里是另一句：「可写挂载核施加前缀之后那一版」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）。改法 C 的原型当初也只改了挂着之后的入口：「原型 B、C 只改了挂着之后的入口那一处（`devices_behind_the_current_version`），可写挂载那一处没改」（`research/prompts/m2-closeout-code-r3-opus-output.md:483`）。用户选的是这个原型（「我选C 但是放到里程碑三去」（`records/2026-09-24-里程碑二收尾调度.md:317`））。所以把缺根那一半接到可写挂载上，是实现员的延伸，不是用户定的改法 C 的字面。
2. 「现行那一版的根写在这块盘上」套到可写挂载的「所选那一版」上没有指称：所选那一版若是在择到的根上施加 journal 记录得来的，它的根（D23 已定项 15：由记录重建）要么没写进根环，要么写了但读不出；条款对这两形都没说。实现的传法是：没施加记录（`effective_root` 与 `chosen_root` 的实例与 txg 都相等，「== start.chosen_root.instance」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3994`））传 `Written`，施加过就传 `NotWritten`（「RootOfTheVersionInTheRing::NotWritten => return None,」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3410`））。
3. 这个传法与字面的两处不合：(a) 施加过记录、而那次发布的根槽其实写过又读坏的盘面，字面（根写在这块盘上而那一槽里不是它）要拒，实现不拒；这一形转给 D23 已定项 14 的 C554 乙-配置续那一段管（「读阶段（择根、扫 journal、重放）读完，读池里每块盘两槽的系统配置」（`.claude/kb/decisions/23-journal的角色与格式.md:389`），「仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常」（`.claude/kb/decisions/23-journal的角色与格式.md:389`））。(b) 反过来，若按字面把「施加过记录」也核根槽，崩在根槽 FUA 之前的合法崩溃状态会被拒：攻方那一格的原样观测是「可写挂载（崩了再挂）也放行，择 (1,7)（靠 journal 记录施加），之后 checker 0」（`research/prompts/m2-closeout-code-r3-opus-output.md:117`），E:947 与变异表第 1482 行（把 `NotWritten` 换成 `Written` 的那一行）钉的就是不能拒这一形。
4. 推导：`Written` 那一支在无故障的历史上是空的。择到的根是读阶段从它的根槽读出来的，那一槽在同一块盘上；一块被换成旧快照的盘上读不出这条根，就择不到它（会择更老的根、再施加记录，即走 `NotWritten`）。所以走到「`Written` ∧ 根槽在被判的盘上 ∧ 不是它」只能是读阶段刚读得出、紧接着连读两次都坏：只有两次连续瞬时读错才会拒。它对旧快照零检出，只多出一个双故障下的新拒因。这一条是推的，没造过盘面。推翻它的现象：造出一块盘面，可写挂载 `effective_root == chosen_root`、单元都在、根槽在被换盘上且不是所选根、之后 checker 红或丢写。
5. 候选补句（只列，不替用户选，也不写进 kb）：
   - 甲：「可写挂载的落后支保持落后且缺所选那一版的单元，不核根槽；改法 C 只管挂着之后的四个入口。」（贴字面；代码里 `root_of_the_selected_version` 那一路、新成员 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:707`）与 E:947 那条用例的一半随之去掉。）
   - 乙：实现员报告的那一句：「可写挂载上，所选那一版就是择到的那条根才算写进了根环，施加过记录的不核缺根」。（保留现状，补条款；代价是一条要两次连续读错才走到、没有用例钉的分支，见 B 节 K10。）
   要连同 D16 已定项 1 那一行一起看：那一行对可写挂载写的是「可写挂载的择根按 C554」……（`.claude/kb/decisions/16-发布语义.md:37`），乙会让可写挂载多一条与它并列的读根槽机制。我推甲更贴条款与用户选的原型；这是推的，用户定。

### C②　`DevicesBehindTheCurrentVersionAndMissingItsUnits` 现在也覆盖缺根，按 code-discipline「名字」一节判要不要改

**判：要改。**

- 判据：「判据：不看注释、不看调用点、不看实现，光看这个名字，说不说得出它是什么、做什么、不做什么。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:15`）。这个名字说「缺它的单元」；现在它在单元一个不缺时也触发。看名字的人会得出「单元都在就不会进这个成员」，实现正好相反。
- 同一节的另两句：「| **换了含义就换名字** | 同名遮蔽只在含义不变时用；原始数据变成验过的、字节数变成块数，就起新名字 |」（`.claude/singlefs-ai-sop/rules/code-discipline.md:33`）与「一个语义概念全仓一个名字」（`.claude/singlefs-ai-sop/rules/code-discipline.md:31`）。这批里同一个概念「一块落后的盘缺那一版的什么」，在 `VersionLackingOnADeviceBehindIt` 与 `SelectedVersionLackingOnDevice` 里已经分成「缺单元」「缺根」两个成员，而三个错误枚举里的成员名仍只有「缺单元」，一个概念两个名字。
- 实现员自己在文档注释里写了「成员名只说了缺单元那一半」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:105`），即承认名字没说全。
- 「单元」在 kb 里并不一致：D18 把根槽称作「自证单元」（「自证单元（journal 记录 / 根槽 / 系统配置槽）不用这张表」（`.claude/kb/decisions/18-块里携带什么信息.md:184`）），而代码里「单元」指 `TransactionUnit` 清单（`units_missing: Vec<(TransactionUnit, SlotNumber)>`，`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/mount.rs:3331`），根不在清单里。名字按代码里的含义读是错的；按 kb 的宽读也读不出「单元都在仍拒」这一层，所以两种读法下名字都要换。
- 代价：副本里这个名字出现在 8 个文件、30 处（`grep -rF` 数出），另有 D18 第 314 行三处引它；`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/src/history.rs:2392` 把它当字符串标签写出，按标签比对的金样要同步（没查有没有）。规则里没有「改名太贵可以不改」的例外：「长度不设限」，判据只看名字说不说得清。清单外五个文件的改动不构成理由，只是这一批的范围问题，交主 agent 排下一批。
- 同一节另见两处小的，不单独判红：`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:429` 的 `let punched = match` 装的是 `Result`，失败时名字说假话；`HolePunchFailure::OtherFailure` 没说调用方要做的决定（对照 `PunchingUnsupported`）。

### C③　打洞失败分路与 fs-design「五条硬要求」逐条对

条款：`.claude/rules/fs-design.md` 第 144–153 行。三条分支：A 打洞成功；B 不支持（`EOPNOTSUPP`、`ENOSYS`）退回逐块写 0；C 别的错照报 `InputOutput`。

| 要求 | A 打洞成功 | B 退路 | C 照报 | 判 |
|---|---|---|---|---|
| 1「切换判据必须显式、可计算、写下来。」（`.claude/rules/fs-design.md:144`） | 判据是 `fallocate` 的返回 errno，写在「按 errno 判一次打洞失败该退回逐块写 0，还是照常报错。」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:170`）与分路表「别的错（读写错、`EBADF`、`EINVAL`、`ENOSPC`，以及没有 errno 的）：照常报成块设备 I/O 错，不被退路吞掉。」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:166`），用 libc 常量、没有魔数 | 同左 | 同左 | 满足；只写在代码与记录里，kb 没有条款（A3 前言） |
| 2「每条分支必须能被测试强制进入。」（`.claude/rules/fs-design.md:146`），要求「留一个只供测试使用的开关，能无视判据直接走指定分支。」（`.claude/rules/fs-design.md:147`） | **没有开关**：只有文件系统支持打洞时才进得去；不支持的环境里 B:823 与 M:38 都红（M 文件头自己写了）。「进不去就等于没测」（`.claude/rules/fs-design.md:148`） | `ActAsIfPunchingWereUnsupported` 强制进入（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:433`） | `ActAsIfPunchingFailedWithAnInputOutputError` 强制进入，只有 `EIO` 一个 errno，别的 errno 只在分路表纯函数上 | **B、C 满足，A 部分满足**。两个开关不是「无视判据」而是把一个假 errno 送进判据，比字面更强；它们是 `pub`、没有 `cfg(test)` 门，生产代码也调得到 |
| 3「每条分支单独验，包括崩溃点重放。」（`.claude/rules/fs-design.md:149`） | B:823、M:38 单独验 | B:891 单独验 | B:971 单独验 | 单验满足；**崩溃点重放没有**：崩溃状态由录制流在内存镜像上重建（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:336`），录制层在后端成功之后才记一步（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/src/lib.rs:368`），文件后端只是沉点；`FileBackedBlockDevice` 在 checker 档 `src` 里零命中。所以分支不同在崩溃枚举里看不出来，这一格靠这个推导（推的）免掉，不靠一条用例 |
| 4「分支必须可观测」（`.claude/rules/fs-design.md:151`） | `punched_calls` | `written_as_zero_bytes_calls` | 返回 `Err`，两个计数不动 | 满足，但计数只在拿得到具体类型 `FileBackedBlockDevice` 时读（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:219`、`wrapped_device()`（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/src/lib.rs:265`）），经 `BlockDevice` trait 的生产路径读不到；悬崖（退路真写 768 MiB）只有 M:104 那条断言守着 |
| 5「分支变量不许在操作中途改变。」（`.claude/rules/fs-design.md:153`），判据是「只在操作期间不会变的输入上分支。」（`.claude/rules/fs-design.md:123`） | 变量是每次调用 `fallocate` 的 errno，调用间不缓存；`zero_fill_landing` 字段只能经按值消耗 `self` 的 `with_test_only_zero_fill_landing` 改（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-core/src/block_device.rs:208`），不能在 `&mut self` 的一次 mkfs 中途改 | 同 | 同 | 满足（推：errno 随文件系统与文件固定。推翻它的现象：同一个镜像文件上、同一个文件系统里，同一种范围先成功后 `EOPNOTSUPP`） |

总判：第 1、4、5 条满足；第 2 条（A 分支无开关）与第 3 条（崩溃点重放）部分满足；前者随环境，后者靠推导免掉。我不建议给 A 加开关：它会是一个「假装打洞成功」，反而放过 M:38 本来要钉的真打洞。

## 判定一览

| 格 | 判定 | 一句话 | 什么现象会推翻它 |
|---|---|---|---|
| A1 C585 条款对实现，改法 C 四个入口 | 一致 | 共用判法先缺单元、后缺根，四个入口同一判，在任何写之前拒（只读盘） | 在副本上造一格：四个入口之一放过了「落后、单元都在、根住在这块盘而那一槽两次读都不是它」的盘，或在写之后才拒 |
| A1 缺根那一半的重读一次 | 条款没说、代码做了 | 借 D16 第 37 行的 R = 1，被借规则的参数没绑全（`AnotherRoot` 与重读后仍不是它的归宿都没写） | 在 D18 第 323 行或 C585 行找到「重读」一词 |
| A1 可写挂载延伸 | 条款没说、代码做了 | 改法 C 字面只到四个入口；延伸的那一支在无故障历史上空转 | 见 C① 第 4 点 |
| A1 D18 第 314 行与第 323 行 | 条款内部冲突，代码跟 323 | 314 仍是「才拒」的旧判法，340 的欠还挂着；不是代码的错 | 主 agent 同步 314 之后这一行消失 |
| A2 C586 | 一致 | 第 5386 行的 max 与 D23 水位一句相同，没动；新用例、变异行、litmus 路径三样欠账行要的都在 | 水位那一行在副本上被改掉而新用例不红 |
| A3 打洞条款对实现（D17、D22、用户定、C603 第一形） | 一致 | 文件后端打洞、`O_DIRECT` 不动、只在 `EOPNOTSUPP` 与 `ENOSYS` 退、`ENOSPC` 照报；唯一生产调用方的步数不变 | 副本上 mkfs 的录制步数或写零长度变了，或 `ENOSPC` 落进退路 |
| A3 C486 ①②、C603 登记表 | 条款说了、代码没做 | 这批只做「写零真写」那一半，欠账仍开 | 在副本里找到那几道检查 |
| A3 块设备契约对长度 0 | **冲突** | 契约允许长度 0 且旧实现与稀疏盘都 `Ok`，打洞实现报 `EINVAL`；今天没有调用方踩到 | 目标文件系统上内核对长度 0 的 `fallocate` 返回 0（我的探针只在 ext4 上）；或契约被改成禁止长度 0 |
| A3 打洞的持久 | 复核不了 | 代码引的是本机内核源码，这台机器没有源码；非 ext4 无一句 | 在有源码的机器上读 `ext4_punch_hole` 与 xfs、btrfs 对应路径 |
| B1 新用例钉的句子 | 钉对，范围有限 | 8 条 harness 用例与 4 条核心单测各钉了表里写的那一句 | 在副本上把任一条的目标行改坏而它不红（那是 Z4 的事） |
| B2 没人钉的句子 | 列出 14 条 | K3、K4、K5、K6、K7、K10、K11、L2、L3、P2–P6，其中 K7（重读）与 K10（可写挂载拒）两条在实现里是活的分支却无用例 | 全仓 `grep` 或旧用例里找到钉其中某一句的断言 |
| C① | 不合，交用户 | 字面只到四个入口；两个候选补句甲（不延伸）、乙（实现员那句），我推甲，推的 | 见 C① 第 4 点 |
| C② | 要改名 | 名字说「缺单元」，实现在单元都在时也进；代码自己承认名字只说一半 | `.claude/singlefs-ai-sop/rules/code-discipline.md` 加「改名代价大可不改」的例外 |
| C③ | 五条里 1、4、5 满足，2、3 部分满足 | A 分支无开关（随环境）、崩溃点重放靠「录制流与文件后端无关」的推导免掉 | 找到崩溃枚举经文件后端跑的路径；或不支持打洞的环境里 B:823、M:38 之外也有用例要强制进 A |

## 没做什么

- 没编没跑任何测试（含 harness 档与 core 的 lib 单测）：实现员的证红与测试结果、`STALE accepted=254 refused=1101 flagged=0`、变异表 14 行里哪几行红，都是照它们的报告转述，我复核不了。
- 没判 Z2、Z3、Z4 的格（攻方的活）；A3 第 7 行（长度 0）与 C③ 第 2、3 条里的几句是正推顺手读到的，那几格的反例造盘归 Z3 ① 与 Z3 ③。
- 没查 xfs、btrfs、tmpfs 上 `fallocate` 的真 errno，没造不支持打洞的文件系统；`ENOSPC` 的后果没量。
- 没读内核 `.c` 源码（这台机器没有）。
- 没逐份翻全仓用例核 L2（环里更高的水位）、P4（`O_DIRECT` 仍真写 0）有没有别处的钉；写的是「我没找到」，不是「没有」。
- 没改任何 kb、代码、变异表、门禁，没替主 agent采纳或出判决；D18 第 314、340 行的同步归主 agent。
- 一次抽样：这份是正推一条腿的一次观测，「一致」「满足」两类结论不支撑否定类断言（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。
