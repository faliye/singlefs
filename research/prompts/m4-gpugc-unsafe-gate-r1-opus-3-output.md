# m4-gpugc-unsafe-gate-r1 攻方腿（Opus，Q2 登记理由、Q3 SAFETY 前提）（2026-11-05）

攻击面：正文第三节 Q2、Q3；Q1 不做。只读代码、登记表与本机缓存里的依赖源码，没写模型、没造样本、没交任何命令（所以没有复跑脚本与 sha256 清单）。
引代码用开工快照 `research/prompts/m4-gpugc-unsafe-gate-r1-snapshot/judged-sha256.txt` 里的路径；开工时核过那份清单前 9 行（判法、登记表与 7 份源码）与工作区逐个 `OK`，行号即快照的行号。

判据（登记表头）：`.claude/gate.d/unsafe-allowances.tsv:18`：「7 理由：为什么非放开不可、不放开的替代为什么不行」。Q2 判的是这一句的两个分句：「非放开不可」与「替代为什么不行」写的那条原因成不成立。

## 各格判定一览

### Q2　登记理由（逐行）

| 登记行 | 项 | 结论 | 一句话依据（详见下文） |
|---|---|---|---|
| 31 | `punch_hole_keeping_the_file_length` | 站得住 | `Cargo.lock` 146 个包里没有 rustix、nix（量的）；理由自己写明了两条零 unsafe 的路与用户的选择；HEAD 里就有这一行 |
| 32 | `crc32_castagnoli_on_route` | 站得住 | 唯一的安全写法是全工作区 `-C target-feature=+sse4.2`，代价是全部产品与装置换代码生成（推的） |
| 33 | `largest_device_local_memory_heap` | 站得住 | wgpu 30.0.1、wgpu-hal 30.0.1 公开的安全接口都不给内存堆（读了本机缓存的源码）；`as_hal` 本身就是 `unsafe fn` |
| 34 | `host_cpu_time_of_this_thread` | 站得住（推的） | 安全的 `/proc/thread-self/schedstat` 不算正在跑的那一截，短窗量不准 |
| 35 | `monotonic_raw_time_now` | 站得住 | std 与 `Cargo.lock` 里没有读 `CLOCK_MONOTONIC_RAW` 的安全写法 |
| 36 | `with_the_address_space_capped_beyond_the_present_size` | **站不住（字面；第二台没核）** | 「只能调 libc」不成立：读 `/proc/self/limits` 加子进程 `prlimit --pid <本进程> --as=…` 零 unsafe |
| 37 | `free_device_local_bytes_of_the_card` | 站得住 | 同 33；wgpu-hal 的 heap budget 是私有方法 |
| 41 | `nanoseconds_of_the_cpu_clock` | 站得住（理由里有一句不准） | 线程钟有现成安全封装（第 34 行那个函数），但它 panic、不交 `None`，进程钟没有封装 |
| 42 | `RawWallClockReading::now` | **站不住（字面）** | 「已有依赖没有安全封装」不成立：工作区包 `singlefs-background-gpu`（在 `Cargo.lock` 里）公开的 `monotonic_raw_time_now` 同一个钟、同样读不到就 panic；代价是 checker 档包多一条非可选依赖、把 wgpu 拉进编译闭包 |
| 43 | `drop_the_page_cache_of_the_file` | **站不住（字面）** | 理由只驳了 `drop_caches`；`dd if=<文件> iflag=nocache count=0` 零 unsafe、只扔这一个文件、失败反映在退出码（本机 coreutils 9.4 的说明原文） |
| 44 | `cpu_number_now` | **站不住** | 「多开一次文件、解析一整行」在主调用处不成立：同一次快照已经读了 `/proc/thread-self/stat` 并切好了栏，第 39 栏白拿 |
| 45 | `clock_ticks_per_second` | **站不住** | 理由只给了「sysconf 或写死 100」两条；`/proc/self/auxv` 的 `AT_CLKTCK`（内核交给进程的那个值）零 unsafe、不作假设；`getconf CLK_TCK` 也是 |
| 46 | `HardwareCrc::crc` | 站得住 | 同 32；核心层只有三路分块一种硬件写法，替不了装置的单路与整单元三切两臂 |
| 47 | `allowed_cores_of_this_thread` | **站不住** | 理由给的原因只是「要另解析区间列表」，是写法偏好不是做不到；替代与它的语义差只在有下线核时出现（推的） |
| 48 | `pin_this_thread_to` | **站不住（字面）** | 「std 也不给线程号」不成立：`std::fs::read_link("/proc/thread-self")` 交回 `<pid>/task/<tid>`；之后 `taskset -p -c <核> <tid>` |
| 53–57 | E183 计数分配器五行 | 登记与源码、SAFETY 对得上 | 处数 1、2、2、2、2 与源码相同；必提词逐处都在；理由与判决第 39 行同一件事 |

### Q3　SAFETY 前提（逐处，共 29 处 unsafe 关键字，数法见下文）

| 位置 | 结论 |
|---|---|
| `crates/singlefs-core/src/block_device.rs:750` fallocate | 成立 |
| `crates/singlefs-core/src/checksum.rs:136` 硬件 CRC | 成立（凭据类型只有一处造法，现查） |
| `crates/singlefs-background-gpu/src/background_gpu.rs:862`、`:865` hal 读内存堆 | 成立 |
| `crates/singlefs-background-gpu/src/background_gpu.rs:2420`、`:2449` 两个钟 | 成立 |
| `crates/singlefs-background-gpu/src/background_gpu.rs:4279` getrlimit | 成立 |
| `crates/singlefs-background-gpu/src/background_gpu.rs:4286` setrlimit 压软上限 | **写了而代码不保证**：「软上限不超过原来的硬上限」没有夹到硬上限（不涉内存安全，后果是 EINVAL、断言 panic） |
| `crates/singlefs-background-gpu/src/background_gpu.rs:4290` setrlimit 放回 | 成立 |
| 真卡测试 `:296`、`:310`（VK_EXT_memory_budget） | 成立（实例 apiVersion 照 wgpu-hal 源码核过） |
| E185 `:646`、`:677` 两个钟 | 成立 |
| E185 `:3719` posix_fadvise | 成立（「上一行打开」措辞差一行，`File::open` 在第 3715 行，中间隔着 `sync_all`） |
| E185 `:4381` sched_getcpu、`:4391` sysconf | 成立 |
| E185 `:8892`、`:8895` 硬件 CRC | 成立（`HardwareCrc {` 字面量全文件只一处，第 8879 行） |
| E185 `:14162`、`:14192` 绑核的 `cpu_set_t` 强转 | 成立；断言比内存安全要的还强（见下文） |
| E183 `:217` `unsafe impl GlobalAlloc`（线程本地那一半） | **成立，靠两条没写出的前提**：目标平台是原生线程本地存储；计数路径不 unwind（release 档开着 overflow-checks，第 142 行有一处不饱和的加法） |
| E183 `:223`、`:225`、`:237`、`:239`、`:251`、`:253`、`:263`、`:265` | 成立 |

## Q2　登记理由

### 先查的：`Cargo.lock` 与本机缓存里有什么（量的，2026-11-05 本机）

```
$ grep -c '^name = ' Cargo.lock
146
$ grep -nE '^name = "(rustix|nix|singlefs-background-gpu|libc|wgpu|wgpu-hal|ash)"' Cargo.lock
36:name = "ash"
360:name = "libc"
800:name = "singlefs-background-gpu"
1116:name = "wgpu"
1185:name = "wgpu-hal"
```

- 锁文件里没有 rustix、nix，也没有别的包封装 `clock_gettime`、`sched_*affinity`、`sched_getcpu`、`sysconf`、`posix_fadvise`、`getrlimit` / `setrlimit`（146 个包名逐个看过：tokio、parking_lot、getrandom、rocksdb、gpu-allocator 这些都不给）。登记行里「已有依赖没有安全封装」对**外部包**这一半成立。
- 但锁文件里有工作区自己的 `singlefs-background-gpu`，它公开两个安全函数：`crates/singlefs-background-gpu/src/background_gpu.rs:2413`：「pub fn host_cpu_time_of_this_thread() -> std::time::Duration {」，`crates/singlefs-background-gpu/src/background_gpu.rs:2442`：「pub fn monotonic_raw_time_now() -> std::time::Duration {」。这是第 41、42 行判法的关键（下文）。
- 本机 cargo 下载缓存（`~/.cargo/registry/cache/`）里有 `nix-0.29.0`、`nix-0.30.1`、`nix-0.31.3`、`rustix-0.38.44` 到 `rustix-1.1.4` 的 `.crate`（量的，`ls`）；`nix-0.31.3` 里 `sched_setaffinity`、`sched_getaffinity`、`sched_getcpu`、`clock_gettime`（含 `CLOCK_MONOTONIC_RAW`、`CLOCK_THREAD_CPUTIME_ID`）、`posix_fadvise`、`sysconf`（`CLK_TCK`）、`getrlimit` / `setrlimit` 都是安全函数（`tar -xzO` 读源码 grep 到的）。它们不在锁文件里，按派发的口径不算「已有依赖」；要用就是新加一个依赖（离线能不能解析、第二台缓存里有没有，都没核）。这一条覆盖第 34–36、41–45、47、48 行，作为「不在本题口径里、但存在的一条路」交主 agent。

### 第 31 行（`punch_hole_keeping_the_file_length`）：站得住

HEAD 里就有这一行（`git show HEAD:.claude/gate.d/unsafe-allowances.tsv` 的第 19 行，现查），这一批没改理由。理由本身已经写明「不放开的路是有的」两条（rustix、记写过的区间）与用户知情之后的选择，不靠「只能」立论。std 唯一的别路是起 `fallocate(1)` 子进程，产品写零、discard 每次请求一次进程，代价不可接受（推的）。推翻它的现象：锁文件里出现带安全 `fallocate` 的包。

### 第 32 行（`crc32_castagnoli_on_route`）与第 46 行（`HardwareCrc::crc`）：站得住

- 「std 没有查过特性再调的安全封装」成立。另一条安全写法有：全工作区编译带 `-C target-feature=+sse4.2`（写进 `.cargo/config.toml` 的 rustflags），这时调带同一特性的函数不用 unsafe。代价（推的，没量）：① 全部产品与装置的代码生成换了，E185 这类量挂钟的实验在被测树上换了编译条件；② 运行时探测与核心层「查到才造得出凭据」的设计变成空的；③ rustflags 进复用键与编译目录，全量重编。两台机器都有 SSE4.2：本机 `grep -c sse4_2 /proc/cpuinfo` 得 32（量的），第二台没核。
- 第 46 行另一句「crates 的 crc32_castagnoli 只有一种写法、替不了这两臂」成立：核心层的硬件路只有 `HardwareCrcInstructionThreeWay`（每路 512 字节分块三路，余下单路，`crates/singlefs-core/src/checksum.rs:176`：「pub const HARDWARE_CRC_LANE_BYTES: usize = 512;」），装置要的单路整段 `crc32q` 与整单元三切（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:8725`：「段长取 ⌊长度 ÷ 3 ÷ 8⌋ × 8、前两段同值、余下的给第三段。」）两样都不在核心层里。

### 第 33、37 行（hal 读显存）：站得住

- 读了本机缓存的 wgpu-30.0.1 与 wgpu-hal-30.0.1 源码：wgpu 公开 API 里与内存有关的只有 `Device::generate_allocator_report`（wgpu-30.0.1 的 src/api/device.rs 第 630 行），报的是 wgpu 自己分出去的块，不是整张卡的堆；wgpu-hal 的 `PhysicalDeviceProperties` 只公开 `properties()`、`supports_extension`、`is_driver`，不给内存属性；heap budget 在 wgpu-hal 的 vulkan/device.rs 第 275 行是私有方法 `fn heap_budget`。`Adapter::as_hal` 本身是 `pub unsafe fn`（wgpu-30.0.1 的 src/api/adapter.rs 第 131 行）。所以经 wgpu 走不出零 unsafe 的路。
- std 的别路：起 `nvidia-smi --query-gpu=pci.bus_id,memory.total,memory.free`（本机 `/usr/bin/nvidia-smi` 在，量的）。代价（推的）：只认 NVIDIA；NVML 的 total / free 与 Vulkan 设备本地堆、`VK_EXT_memory_budget` 的「本进程预算减用量」不是同一个量；产品包里起外部进程。第 37 行要的是「本进程此刻还能建多少」，nvidia-smi 不分进程预算，替不了。两行理由站得住。

### 第 34、35 行（产品 GPU 包的两个钟）：站得住

- 线程 CPU 钟：std 没有；`getrusage(RUSAGE_THREAD)` 同样要 libc。零 unsafe 的只有 `/proc/thread-self/schedstat` 第一栏（本线程上 CPU 的纳秒）与 `/proc/thread-self/stat` 的 utime、stime（节拍，本机 100 Hz，10 ms 一格，量的 `getconf CLK_TCK`）。前者按我对内核的印象（推的，没读内核源码）只是 `sum_exec_runtime` 原值、不把正在跑的那一截补到此刻，读的线程自己正在跑，差可到一个调度节拍；E202 这类亚毫秒的一次提交量不准。后者更粗。站得住。
- `CLOCK_MONOTONIC_RAW`：std 的 `Instant` 读 `CLOCK_MONOTONIC`；procfs 里没有零 unsafe 的等价钟（`/proc/uptime` 是 10 ms 粒度的开机钟）。站得住。
- 旁注（不改结论）：两处写的漂移数对不上。第 42 行与 E185 源码写「慢约 7 ppm」，`crates/singlefs-background-gpu/src/background_gpu.rs:2433`：「两样成对相减时差出几十 ppm；要拿挂钟与线程 CPU 钟比大小的读这一个。」；E202 第四段执行员读到 −3.921、−4.764 ppm（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:796`：「本机读到 −3.921、−4.764 ppm」）。NTP 调频随时间与机器变，符号都会翻；「7 ppm」是 2026-10-19 本机的一次读数，写进理由当常量是过度外推。理由真正要的只是「`CLOCK_MONOTONIC` 受调频、RAW 不受」，那一句成立。

### 第 36 行（`with_the_address_space_capped_beyond_the_present_size`）：站不住（字面；第二台没核）

- 理由：`.claude/gate.d/unsafe-allowances.tsv:36`：「std 没有 getrlimit/setrlimit，只能调 libc」。「只能」不成立。
- 替代写法（推的，没实现没跑）：读 `/proc/self/limits` 的「Max address space」一行拿原来的软、硬上限（`std::fs::read_to_string`）；压的那一步起子进程 `prlimit --pid <std::process::id()> --as=<软>:<硬>`（util-linux；同一用户对自己的进程调 `prlimit(2)` 不要特权）；子进程是 posix_spawn 起的，不另占本进程的地址空间，起它时上限还没压。放回那一步可以照样起一次，或者不放回（这个进程只跑这一条用例，函数文档 `crates/singlefs-background-gpu/src/background_gpu.rs:4254`：「压的是整个进程：只在地址空间压住的那个子进程里调（那个进程只跑一条用例），不在跑别的用例的进程里调。」）。注意放回时起的那个子进程本身受压住的上限管，`prlimit` 自己的 VmSize 远小于上限，推的够用。
- 代价：准——同一个「此刻 VmSize 加 32 MiB」，精度不变；快——多一到两次 fork/exec（毫秒级，推的），只在这一条用例里；两台机器——本机 `/usr/bin/prlimit`（util-linux 2.39.3，量的），第二台没核。测试里依赖一个外部命令，是不是比一处 unsafe 好，是取舍，交主 agent；但「只能」那一句要么改成写明这条替代与为什么不用，要么换写法。

### 第 41 行（`nanoseconds_of_the_cpu_clock`）：站得住（理由里有一句不准）

- 线程钟那一半有现成安全封装：`singlefs_background_gpu::background_gpu::host_cpu_time_of_this_thread`。但它读不到就 `assert_eq!` panic，E185 要的是读不到交 `None`、那一轮作废（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:620`：「本线程到此刻在 CPU 上跑过的纳秒；系统调用报错、读出负数的交回 `None`，调用方把那一轮作废」）；进程钟（`CLOCK_PROCESS_CPUTIME_ID`）没有封装。这一项照样非放开不可（不加新依赖的前提下）。
- 不准的一句：理由「已有依赖里只有 libc」——工作区包也是已有依赖，有线程钟封装；应写成「工作区包的线程钟封装读不到就 panic，进程钟没有」。

### 第 42 行（`RawWallClockReading::now`）：站不住（字面）

- 理由：`.claude/gate.d/unsafe-allowances.tsv:42`：「读 CLOCK_MONOTONIC_RAW 只能经 libc 调 clock_gettime(2)，已有依赖没有安全封装」。
- 反例：`monotonic_raw_time_now`（`background_gpu.rs:2442`）读同一个钟、读不到同样 panic——E185 这一处也是读不到就 panic：`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:685`：「"CLOCK_MONOTONIC_RAW 在 Linux 上恒可读；读不到说明平台不对"」。换成 `RawWallClockReading(u64::try_from(monotonic_raw_time_now().as_nanos()).expect(…))`，读数与语义逐位相同（推的）。
- 代价（推的）：`singlefs-checker-tier` 今天不依赖 `singlefs-background-gpu`（它的 `Cargo.toml` 里零命中，现查），而后者的 wgpu 不是可选依赖（`crates/singlefs-background-gpu/Cargo.toml:11`：「wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }」），checker 档包今天的 wgpu 只挂在 `gpu` 特性下（`crates/singlefs-checker-tier/Cargo.toml:39`：「gpu = ["dep:wgpu", "dep:pollster"]」）。照搬就把 wgpu、ash、naga 拉进 checker 档每个二进制的编译闭包，判法单元的键（D13（验证路线） 已定项 17）多半跟着变。另一条路：把两个钟的安全函数挪进两包都依赖的 `singlefs-core`，删掉产品 GPU 包与 E185 各两处，核心层加两处（净少两处，第 41 行的进程钟另算）；这与核心层「恰好一处」的旧决定（第 31 行理由里的 2026-10-01）有关，交用户。理由要么写出这条替代与不用的原因（编译闭包），要么换写法。

### 第 43 行（`drop_the_page_cache_of_the_file`）：站不住（字面）

- 理由只驳了一条替代：`.claude/gate.d/unsafe-allowances.tsv:43`：「写 /proc/sys/vm/drop_caches 要 root、扔的是全机的缓存」。
- 没驳的那条：GNU dd 的 `iflag=nocache count=0`。本机 `info coreutils 'dd invocation'` 原文（量的，coreutils 9.4）：「Request to discard the system data cache for a file.  When count=0 all cached data for the file is specified」「Also when count=0, failure to discard the cache is diagnosed and reflected in the exit status.」，例子 `dd if=ifile iflag=nocache count=0`。页缓存挂在 inode 上，dd 用自己的描述符扔，效果与本进程的描述符相同（推的）；脏页照旧要先 `sync_all`，现有代码已经先同步。
- 代价：准——同一个「整个文件、干净页」（按我对 coreutils 的印象它内部就是 `posix_fadvise(DONTNEED)`，推的）；快——每次一次 fork/exec（毫秒级，推的），调用处在每次测量前恢复工作件、整份算摘要之后（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:3829`：「drop_the_page_cache_of_the_file(&self.work_paths[device_index])」），相对整份摘要可以忽略；两台——本机 `/usr/bin/dd` 在，第二台没核（coreutils 是基础包，推的在）。

### E185 三项（派发点名逐项审「为什么没用替代」）

#### 第 44 行 `cpu_number_now`：站不住

- 理由给的不用替代的原因：`.claude/gate.d/unsafe-allowances.tsv:44`：「/proc/thread-self/stat 第 39 列也报核号，但每读一次要多开一次文件、解析一整行」。
- 这个原因在主调用处不成立。`cpu_number_now` 有两个调用者：
  - 意图前后的快照 `thread_and_machine_snapshot`：同一次快照里已经读了这个文件、并把整行切成栏——`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4419`：「Path::new("/proc/thread-self/stat"),」，`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4420`：「thread_counters_of_the_counter_record,」，切栏的函数从最后一个 `)` 之后数（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4304`：「从最后一个 `)` 之后数（进程名里有空格与括号也不错位）」），再取第 39 栏不多开文件、不多切一次行；同一个结构里紧接着才调 `cpu_number: cpu_number_now()`（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4434`：「cpu_number: cpu_number_now(),」）。而且这两次快照在计时窗口里（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4699`：「都在调用方进的计时窗口里、在 `measure` 之外」），换成第 39 栏反而少一次 `sched_getcpu`，窗口里的开销不增反减。
  - `host_core_reading_now`（`:14143`）：GPU 一格一轮的前后各一次（`:14412`、`:14420`），都在量的那一格之外；它本来就读两份 sysfs 文件，多一份 `/proc/thread-self/stat` 是 3 份对 2 份，微秒级（推的）。
- 语义：proc(5) 说第 39 栏是「CPU number last executed on」；读的线程自己正在跑，内核取的就是此刻这个核（推的，按我对内核的印象，没读内核源码）。`sched_getcpu` 交回之后线程也可能马上换核，两者同样只是「读的那一刻」。本机一次读数：`/proc/thread-self/stat` 第 39 栏得 14（量的，awk 切栏）。
- 替代写法（推的，没实现没跑）：快照里把读 stat 的那一次改成交回（四个计数、第 39 栏）；`host_core_reading_now` 读一次 stat 取第 39 栏；删掉 `cpu_number_now` 与它那一行登记。

#### 第 45 行 `clock_ticks_per_second`：站不住

- 理由：`.claude/gate.d/unsafe-allowances.tsv:45`：「写死 100 是替平台作假设」。它把替代只列成「sysconf 或写死 100」，漏了两条不作假设、零 unsafe 的：
  - `/proc/self/auxv` 里类型 17（`AT_CLKTCK`）那一项：内核在 exec 时交给进程的节拍数。`/usr/include/linux/auxvec.h:26`：「#define AT_CLKTCK 17	/* frequency at which times() increments */」；libc 0.2.189 也有安全常量 `pub const AT_CLKTCK: c_ulong = 17;`（本机缓存的 linux_l4re_shared.rs 第 863 行）。本机 `LD_SHOW_AUXV=1 /bin/true` 打出 `AT_CLKTCK:            100`（量的）。按我对 glibc 的印象，`sysconf(_SC_CLK_TCK)` 交回的就是这个值（推的，没读 glibc 源码）。
  - `getconf CLK_TCK` 子进程：本机得 `100`（量的）。
- 代价：准——同一个数（auxv 是内核给的原值）；快——每个进程读一次、按 8 字节一对拆（约十几行），调用处一次（`:4672`）；两台——任何 Linux 都有 `/proc/self/auxv`，第二台没核。

#### 第 47 行 `allowed_cores_of_this_thread`：站不住

- 理由给的不用替代的原因：`.claude/gate.d/unsafe-allowances.tsv:47`：「/proc/thread-self/status 的 Cpus_allowed_list 也报这个集合，要另解析区间列表」。「要另解析」是写法多几行，不是做不到；这份文件已经在解析 `/proc/thread-self/status`（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4295`：「`/proc/thread-self/status` 的 `nonvoluntary_ctxt_switches`（N4）。」）。本机读数（量的）：`Cpus_allowed_list:	0-31`，`taskset -pc` 得 `current affinity list: 0-31`，两者相同。
- 理由该写而没写的那条真差别（推的，按我对内核的印象）：`sched_getaffinity` 交回的是允许掩码与「活着的核」的交，`Cpus_allowed_list` 是允许掩码原样；有下线核时后者多列下线的。本机 `/sys/devices/system/cpu/offline` 是空的（量的），第二台没核。这条差别对 E185 的后果：`preferred_core` 可能挑到下线核，`sched_setaffinity` 报错，`pin_this_thread_to` 交 `false`，受控臂不绑——结局可见、不静默。
- 替代写法（推的）：读 `Cpus_allowed_list`，按 `,` 切、每段 `a` 或 `a-b` 展开；或读 `Cpus_allowed` 的十六进制掩码。调用处是每条臂开头与测试（`:14233`、`:14247`、`:24501`–`:24515`），不在计时窗口里。

#### 第 48 行 `pin_this_thread_to`：站不住（字面）

- 理由：`.claude/gate.d/unsafe-allowances.tsv:48`：「起 taskset 子进程要先拿本线程的线程号，std 也不给」。std 没有专门的接口，但零 unsafe 拿得到：`std::fs::read_link("/proc/thread-self")` 交回 `<pid>/task/<tid>`，本机 `readlink /proc/thread-self` 得 `1804098/task/1804098`（量的）；或读 `/proc/thread-self/stat` 第一栏。
- 替代写法（推的）：拿到 tid 后起 `taskset -p -c <核> <tid>`，看退出码判绑成没有（`taskset -p` 不带 `-a` 只改这一个线程）；调用处每条受控臂一次（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:14241`：「preferred_core(&rankings, cores).filter(|core| pin_this_thread_to(*core))」）。
- 代价：准——同一个系统调用落在同一个线程上，线程在等子进程时睡着，醒来就在新掩码里（推的）；快——一次 fork/exec，毫秒级，在臂开头、计时窗口外；两台——本机 `/usr/bin/taskset`（util-linux 2.39.3，量的），第二台没核；另有一个后果：起子进程时本进程多一个子进程而不是多一条线程，不碰「计时窗口前只剩一条线程」那条核对（推的）。这一行与第 47 行一起改，E185 的绑核就整段零 unsafe。

### E183 五行（第 53–57 行）：登记与源码、SAFETY 对得上

- 处数：`unsafe impl GlobalAlloc for CountingAllocator`（`:217`）项头 1 处；四个方法各「`unsafe fn` 签名 1 + 块 1」= 2 处（`:223`/`:225`、`:237`/`:239`、`:251`/`:253`、`:263`/`:265`），与第 5 段 1、2、2、2、2 相同（现查 `grep -nE '\bunsafe\b'`）。
- 必提词：impl 上那段 SAFETY（`:213`–`:216`）有「System」「原子」「线程本地」「析构」；每个方法上下两处 SAFETY 都有「layout」「System」，`dealloc`、`realloc` 两处另有「pointer」，`realloc` 另有「new_size」。逐处都在。
- 理由与判决对得上：登记表第 53–57 行理由里那一句与 `research/prompts/m3-close-b19-reason-r3-main-verification.md:39`：「**E183 离不开（三份一致）**：量的是一次调用前后线程本地的 Layout 字节，VmHWM 是页粒度，`Cargo.lock` 与本机缓存里没有安全计数库」逐字同一件事；同一行判的「SAFETY 说计数是原子的，加了线程本地计数之后只对一半」已改：现在的 SAFETY 把两份计数分开写。「本机缓存里没有安全计数库」我现查成立：下载缓存里带 alloc 字样的只有 `allocator-api2`、`gpu-allocator`（量的）。
- 线程本地那一半的前提另见 Q3。

### Q2 打中的几格先答四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）

对第 36、42、43、44、45、47、48 行：

1. 分不分辨臂：这里的「臂」是「放开」与「替代」两条。每一格打中的是那一行自己写的「替代为什么不行」，替代真实存在、按行各不相同，分辨得出；不是所有行一起中（第 31–35、37、41、46 行没中）。
2. 判别子当时看不看得到：看得到。第 44 行要的 `/proc/thread-self/stat` 在同一份文件里已经读着；第 42 行要的安全函数在同一批（E202 修订第 10 条）刚登进第 35 行；auxv、`dd`、`prlimit`、`/proc/thread-self` 都是写理由那一刻本机就有的东西。
3. 满足的是判据哪个分句：表头第 7 段「不放开的替代为什么不行」——理由给出的那条原因不成立（44、47 是原因不成立或只是偏好；36、42、43、45、48 是「只能」「没有」这类全称句被一个反例打穿）。「为什么非放开不可」那一分句随之不成立。不归「改法碰不到」：这一题没有跑前条款给的改法。
4. 改法在打中的格上中不中：见下面「改法各修哪一格」表，每格都是推的。

## Q3　SAFETY 前提（逐处）

数法：快照里 7 份源码 `grep -nE '\bunsafe\b'` 去掉注释与 reason 字符串里的字样，剩 29 处关键字：核心层 2、产品 GPU 包 9（`background_gpu.rs` 7、真卡测试 2）、E185 9、E183 9。与登记表第 5 段合计 1+1+2+1+1+3+2+1+1+1+1+1+2+1+1+1+2+2+2+2 = 29 相同。

### 成立的（逐处一句核据）

- `crates/singlefs-core/src/block_device.rs:750`：四个整数参数、不碰内存；`&File` 借用到调用返回。两个调用者都先过 `check_request`、长度 0 先交回（现查 `:845`、`:854`、`:940`、`:942`）。
- `crates/singlefs-core/src/checksum.rs:136`：凭据 `CrcInstructionDetected` 的字段私有，全仓字面量只一处（`:78`–`:80`，`grep -rn '_constructed_only_after_the_runtime_detection' crates/` 只命中这份文件，现查）。
- `background_gpu.rs:862`：与 wgpu 文档的第一条安全要求（不销毁守卫借出的对象）对得上。`:865`：`vkGetPhysicalDeviceMemoryProperties` 是 1.0 核心命令，句柄与实例同出一个守卫，守卫活到函数末尾。
- `background_gpu.rs:2420`、`:2449`：栈上初始化过的 `timespec`、`&raw mut` 不造引用，钟号非法时内核报 EINVAL 不写。
- `background_gpu.rs:4279`、`:4290`：同上形。
- 真卡测试 `:296`、`:310`：`vkGetPhysicalDeviceMemoryProperties2` 要实例按 1.1 以上建。注释写的判据是 `instance_api_version()`；我核了 wgpu-hal 30.0.1 的 vulkan/instance.rs：那个数是 `vkEnumerateInstanceVersion` 的结果，≥ 1.1 时建实例的 `apiVersion` 写 1.3（同文件第 593–624 行），所以判据蕴含建实例的版本 ≥ 1.1，成立。`push_next` 挂的 `budget` 生命周期由借用检查管着。
- E185 `:646`、`:677`：同 `background_gpu.rs:2420` 的形。
- E185 `:3719`：不传指针。措辞：「上一行打开」实际是两行之上（`:3715` 打开、`:3716` `sync_all`），不影响前提。
- E185 `:4381`、`:4391`：不带指针参数。
- E185 `:8892`、`:8895`：`HardwareCrc` 的字段私有，字面量 `HardwareCrc {` 全文件只在 `:8879`（`grep -nE 'HardwareCrc \{'` 现查，`:8851` 是定义），没有 derive 能造。
- E183 `:223`–`:265` 八处：原样转给 `System`，计数在成功之后改；`realloc` 失败（null）时旧块照旧算在用，与契约一致。

### E185 绑核两处的 `cpu_set_t` 强转（`:14162`、`:14192`）：成立

- 断言（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:14129`：「std::mem::size_of::<libc::cpu_set_t>() == AFFINITY_MASK_WORDS * std::mem::size_of::<u64>()」）与 libc 0.2.189 的定义对得上：x86_64 上 `cpu_set_t` 是 `bits: [u64; 16]`（本机缓存 linux_l4re_shared.rs 第 80–85 行），128 字节、对齐 8。
- 内存安全真正要的比断言弱：长度是 `size_of_val(&mask)` 照实传的，内核与 glibc 只按这个长度读写；Rust 这边只造裸指针、不造 `&cpu_set_t`，对齐只对 C 那边有意义，而 glibc 的包装与内核都按字节拷。断言是充分条件，不缺前提。
- 措辞可补一处（不改结论）：`sched_getaffinity` 那条写「内核至多写这么多」，按我对 glibc 的印象（推的）是内核写 `min(len, cpumask_size())` 字节、glibc 包装再把剩下的清零到 `len`，写进来的是「内核与 glibc 合起来正好 `len` 字节」，仍不越界。
- 不是内存安全的、注释没写的前提：按位解码（`mask[core / 64] & (1 << core % 64)`）靠内核的 cpumask 是小端的 64 位 `unsigned long` 数组；x86_64 上成立。CPU 超过 1024 个时内核报 EINVAL（长度不够），交 `None`，不越界。

### `background_gpu.rs:4286`（setrlimit 压软上限）：写了而代码不保证

- 注释：`crates/singlefs-background-gpu/src/background_gpu.rs:4285`：「软上限不超过原来的硬上限（只压软的那一个）」。
- 代码：`crates/singlefs-background-gpu/src/background_gpu.rs:4282`：「rlim_cur: present_size_in_kibibytes * 1024 + room_beyond_the_present_size,」，`crates/singlefs-background-gpu/src/background_gpu.rs:4283`：「rlim_max: limit_before.rlim_max,」——没有与 `limit_before.rlim_max` 取小，也没有与原来的软上限取小。硬上限有限且小于「VmSize + 32 MiB」时软上限大于硬上限；原来的软上限更低时这一步其实是抬高软上限，「只压」也不对。
- 后果：不是内存安全问题（`setrlimit` 只读这个结构，参数不合法交 EINVAL），下一行 `assert_eq!(cap_status, 0, …)` 在子进程里 panic，用例判红。所以这条前提对 unsafe 本身是多余的，写了反而是一句不成立的话。
- 补法（二选一，推的）：① 代码夹住：`rlim_cur: (present_size_in_kibibytes * 1024 + room_beyond_the_present_size).min(limit_before.rlim_cur).min(limit_before.rlim_max)`，注释照旧；② 不改代码，把那半句从 SAFETY 删掉（它不是 setrlimit 的安全前提），挪进函数文档写成「假定原来的软、硬上限都比 VmSize + 32 MiB 大，不然 setrlimit 报错、用例 panic」。

### E183 `:217` `unsafe impl GlobalAlloc`（线程本地计数那一半）：成立，靠两条没写出的前提

注释说的部分我逐句核了：`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:127`：「static THIS_THREAD_IN_USE_BYTES: Cell<i64> = const { Cell::new(0) };」——常量初始化；`Cell<i64>` 没有析构；只由所在线程读写（`thread_local!` 给的）。没写出、但它靠着的两条：

1. **目标平台用原生线程本地存储。**「在分配器里碰它不再分配」按我对 std 的印象（推的；本机工具链没装 rust-src，没读到源码）只在有原生 `#[thread_local]` 的目标上成立：这时常量初始化、不要析构的线程本地量编成一次直接的 TLS 访问，不注册、不分配；在没有原生 TLS、走系统键的目标上，std 第一次访问时要在堆上建这一份，会递归进这个分配器。x86_64-unknown-linux-gnu 是原生 TLS，两台都是（推的）。还要它是可执行文件自己的静态 TLS 块，不是 dlopen 进来的库（后者 glibc 可能在 `__tls_get_addr` 里 malloc）；这是 bin 根，成立。补法：注释加一句「目标平台（x86_64 Linux gnu）有原生线程本地存储，常量初始化、无析构的线程本地量访问时不分配；这份装置是可执行文件，线程本地块是静态的」。
2. **计数路径不 unwind。** `GlobalAlloc` 的契约不许分配器 unwind。线程本地那几处都是饱和运算，不会 panic；全进程那一份有一处不饱和的加法：`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:142`：「let in_use_now = self.in_use_bytes.fetch_add(bytes, Ordering::Relaxed) + bytes;」，而 release 档开着溢出检查（`Cargo.toml:23`：「overflow-checks = true」），溢出就 panic。可达性（推的）：在用字节等于活着的字节（每一块都经这个分配器分、经它放，`fetch_sub` 不会减过头），不超过地址空间（约 2⁴⁷），单块 `Layout` 不超过 `isize::MAX`，两者相加到不了 2⁶⁴，不可达。补法：注释加「计数路径不 panic：线程本地那份全是饱和运算，全进程那份的和不超过活着的字节」；或者把那一处改成 `saturating_add`，前提就不用靠可达性论证。

这两条都不改「成立」：今天的平台与写法上前提为真。推翻它的现象：E183 换到无原生 TLS 的目标、或改成动态库、或计数改成会溢出的量（例如累计分配总字节而不是在用字节）。

## 改法各修哪一格（全部只在我读代码推的、没实现没跑，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 第 39 栏替 `sched_getcpu`（快照里复用已读的 stat；`host_core_reading_now` 多读一份） | Q2 第 44 行 | 推的 |
| 读 `/proc/self/auxv` 的 `AT_CLKTCK`（或 `getconf CLK_TCK`） | Q2 第 45 行 | 推的（本机 auxv 与 getconf 都读到 100 是量的） |
| 读 `Cpus_allowed_list` | Q2 第 47 行 | 推的（本机与 taskset 读数相同是量的） |
| `/proc/thread-self` 拿 tid 加 `taskset -p -c` | Q2 第 48 行 | 推的 |
| `dd iflag=nocache count=0` | Q2 第 43 行 | 推的 |
| `/proc/self/limits` 加 `prlimit --pid` | Q2 第 36 行；顺带消掉 Q3 `background_gpu.rs:4286` 那一格 | 推的 |
| E185 改用 `monotonic_raw_time_now`（或两个钟挪进 `singlefs-core`） | Q2 第 42 行（挪进核心层还连带第 34、35 行） | 推的 |
| 只改理由字面：写出上面的替代与不用的真原因 | Q2 第 36、41、42、43、45、47、48 行；对第 44 行不起作用（它的原因在主调用处就是反的） | 推的 |
| setrlimit 夹上限，或把那半句挪出 SAFETY | Q3 `background_gpu.rs:4286` | 推的 |
| E183 SAFETY 补两条前提，或第 142 行改饱和加 | Q3 E183 `:217` | 推的 |
| 加依赖 nix（本机下载缓存里有） | Q2 第 34–36、41–45、47、48 行一起 | 推的；离线解析、第二台缓存没核；不在「已有依赖」口径里 |

## 超出攻击面、交主 agent 的两条观测（没判，只报）

- 登记行第 8 段：第 33–37 行写「用户 2026-11-03 弹窗定」，指到的记录里那次弹窗定的是门禁写法（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:761`：「弹窗问用户，用户定」，定的是「改门禁，一处一行」），不是逐处「非放开不可」。这几行的必要性因此只靠第 7 段理由撑着、不靠用户的裁定；第 36 行的理由又是上面打穿的那一条。第 41–48 行的出处只写了记录路径，没写谁定的。
- `crates/singlefs-core/src/checksum.rs:298` 与 `:300` 各有一个 `#[allow(unsafe_code, …)]`（Verus 的 `assume_specification`），核心层 `src/` 下这类属性共 41 处（`grep -rn 'allow(unsafe_code' crates/singlefs-core/src | wc -l`，量的），而登记表核心层只两行。门禁对它们怎么判归 Q1，这条腿不判。

## 没打中的形状（试过、没打穿）

- 第 31、32、33、34、35、37、46 行：逐行找过「锁文件里的包有没有安全接口」（146 个包名；wgpu、wgpu-hal、ash 读了本机缓存源码）与「std 加 procfs / 系统命令有没有等价写法」，前者全无；后者有的（nvidia-smi、schedstat、全局 target-feature、fallocate 子进程）各有准度或范围上的代价，理由站得住。第 41 行的进程钟与「读不到交 None」两样别处都没有。
- Q3 的 26 处「成立」：逐处找过四种形状——指针指向的东西在调用期间活不活、长度与缓冲是否照实、凭据类型能不能绕过检测被造出来（全文件 grep 字面量）、FFI 声明与 libc 的布局是否一致（`cpu_set_t`、实例版本）。都没打穿。
- E183 五行的登记与源码：处数、必提词、理由与判决逐字段对过，没有不一致。

## 这条腿自己的限度

- 只读，没写模型、没造样本、没跑一行项目代码；上面的替代写法都没实现过，「零 unsafe」「语义相同」两句是推的。
- 几处内核、glibc、std 的行为是我的印象，不是读源码核的，文中都标了「推的」：`/proc/<tid>/stat` 第 39 栏与 `sched_getcpu` 等价、`sched_getaffinity` 与 `Cpus_allowed_list` 的下线核差别、schedstat 不含正在跑的那一截、glibc 的 `sysconf(_SC_CLK_TCK)` 取 `AT_CLKTCK`、dd 的 nocache 内部用 `posix_fadvise`、std 原生 TLS 访问不分配。本机工具链没装 rust-src，内核与 glibc 源码也不在本机。
- 第二台一处都没核（工具在不在、有没有下线核、缓存里有没有 nix）。
- 「站得住」是一次抽样的「没打中」，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不单独支撑结论。

## 要主 agent 跑的命令

无。这条腿没有模型与样本；判决要引的读数都是只读命令（`grep`、`ls`、`getconf`、`readlink`、`LD_SHOW_AUXV=1 /bin/true`、`taskset -pc`、`info coreutils`），主 agent 现查即可。

## 没做什么

- 不判 Q1（判法放过与误拒），不判正推腿的格；没读禁读清单里的正推腿报告与模型。
- 不替主 agent 采纳任何改法；没往统一任务服务交任务；没改登记表与源码。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-unsafe-gate-r1-opus-3` 认领了，里面只有 `cite-check.txt`（交回前跑的引文核对输出）；没建仓副本与编译目录，没有要删的。
