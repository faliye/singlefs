# 根环实现第 10y 批代码轮第一轮：云端攻方（Opus）报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G1 -->

攻击面：正文第三节 K1 到 K4，立场找反例，重点 K1 / K2 的新读法与上界、重起、回调，与 K4 的 C694 B、C 和 T1 甲交界。代码行号引冻结副本（`~/.local/share/singlefs/work/m3-batch10y-code-r1/tree/`；被引的几份 crates 文件与主工作区今天逐字节相同，sha256 前 12 位核过）。数都是我在冻结副本拷贝上量的，不是入库装置上的数。没占显卡：`nvidia-smi` 只读跑，显存账那几形用造的报告喂真函数。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 甲 一次读出 | **打中（窄化、没消除）** | 一份 `-q` 报告里 FB 空闲与进程那一行是两次 NVML 查询；本机两张卡的两次读显存查询相隔 1347.9 µs。建上借据那一刻落在中间，模型上这一笔多给 8000 MiB（整笔借据） |
| K1-1 乙 没走卡池的设备 | **打中（窗口）** | 总线号在 `start_the_card` 编完管线之后才记，之前那一段读数把那个设备的占用当借据已建上；模型多给 500 / 1200 MiB |
| K1-1 乙附「只会少借」 | **打中（文档那句不成立，物理后果推的）** | 读不到或分不出时把驱动那一份 D 当已在卡上，驱动实占 x < D 时比列出来的那条路多给 D − x（模型 1960 MiB） |
| K1-1 丙 销账未还 | **部分复现** | R 照 m − D 往下压，x < D 时压过头，被压掉的那截当成活借据已建；模型多给 1960 / 2260 MiB，x = D 时 0 |
| K1-1 丁 上界 Total − Reserved | 本机没打中，有条件的一形 | 空卡上 Free = Total − Reserved 实测逐 MiB 相同；卡上有常驻的别的用户时，落在两者之间的量永远报本次未跑、不判红（模型 resident 400 MiB） |
| K1-1 同一个量 / 认不得的输出 | 没打中（量过） | `-q` 的 FB Free 与 `--query-gpu` 相同，进程 Used 与 `--query-compute-apps` 相同（一个活着的 wgpu 用例进程，C+G，2687 MiB）；缺 Reserved 报错判红，「Not available」记成没列（不当 0） |
| K1-3 OOM 归因 | **打中（读码，推的）** | 设备上的 OOM 计数整台设备共用：别的线程同一刻记了一次 OOM，这条线程上真正的 panic（比如校验错）被认成显存不够、报本次未跑 |
| K1-3 设备丢了重起 | **打中（读码，推的）** | 重起不另登记驱动那一份，触发重起的那一笔按「驱动那一份已在卡上」借，新设备的上下文没留地方 |
| K1-3 不缓存 Err | 没打中（读码） | `DeviceStartedOnDemand` 起不来不留着 |
| K2-2 两种结局出口 | 没打中（读码） | `decide_scratch_before_booking` 与 `decide_after_attempting_every_card` 两支出口与判决一致 |
| K2-1 / K2-2 | 同 K1-1、K1-3 那几形 | 给大：甲、乙、丙、重起；放得下却未跑：只有丁（有条件） |
| K3 改回去照绿 | **打中（读码加 grep，推的）** | 认出没走卡池设备的那两处接线（记总线号、交给读数器）没有一条测试、`crates/mutations.tsv` 里 0 行 |
| K4-1 落点掉出影子账 | 没打中新形 | 两臂同样中的只有已知 C554 那一形（C 的槽在之后的挂载里两遍都读坏） |
| K4-2 本该放行被拒 | **打中（量过，在卸载与会话写，不在可写挂载）** | C 的账挂上之后读坏：今天 84 / 84 段历史卸载被拒；照 T1 甲字面「照它记着的落点」那一臂 84 / 84 卸载做成，卸载后 I-7.4 0 |

## 复跑

模型目录 `research/prompts/m3-batch10y-code-r1-opus-model/`，文中 `out/…` 指它下面的 `out/`。

```
bash research/prompts/m3-batch10y-code-r1-opus-model/rerun.sh setup <空草稿目录>   # 拷冻结副本、打 opus-k4-arms.patch、放两个原型、release 编译
bash research/prompts/m3-batch10y-code-r1-opus-model/rerun.sh ledger <草稿目录>     # K1 / K2 那几形（OPUS10Y-TORN / OUTSIDE / RETURNED / CEILING / PARSE 行）
bash research/prompts/m3-batch10y-code-r1-opus-model/rerun.sh k4 <草稿目录>         # K4：b1 六批、b2 对照四批，并行，各经 run-with-memory-cap.sh 4G 与 capped.sh 1
bash research/prompts/m3-batch10y-code-r1-opus-model/rerun.sh agg <草稿目录>        # k4agg.py 汇总
bash research/prompts/m3-batch10y-code-r1-opus-model/rerun.sh shim <草稿目录>       # nvidia-smi 读显存那两次 NVML 调用的时刻（只读）
```

模型目录每个文件的 sha256：

```
220e723ef2df1230b0f481ea1f333016be380a9f2df8a918600aae511cd76bb0  ./k4agg.py
4f2eabaf6af202703cf8a80196efc85690b2baf350f32c2cc1e4f7e0ec56c260  ./nvml-shim/logall.c
d472efa8b5135e868380c1fd56a06f0770c12194a1e168f79ceb30916f3bbba7  ./nvml-shim/shim.c
fe87648355c476d1d26ef474da70021e9db4bdbbf333117ceb23f24b5989e4c8  ./opus_10y_k4_probe.rs
2ae58a3188990a9a675d69cc004b9efc0b6e6da428d9f37475b823ce8b7102e3  ./opus_10y_ledger_probe.rs
60c1b9c75c25084b1661555974dec099ab8eb310a824a41c04a689c9776c8ff6  ./opus-k4-arms.patch
8d447c001109abdd3a7fd7aa2cad22a6066cdf66140f92b77cf8aa834dc1fbcd  ./out/b1-1.log
bae4f20a1da5a82ef9d4b88cb6888d1ce720d2e5393f4a327f6ff5e9c35b3900  ./out/b1-2.log
55472808b0db59c79dfef0a4da9ad5dc7f9c5dcde325423c64878291f98a43b2  ./out/b1-3.log
edec2bafdccfc05917a4e43cee41ddf49bb00c6daa5d318f3d66f0edbc3ca83e  ./out/b1-4.log
8bad16ff9991d85a82f2d2286486c42e1fa4c520711b2e99e7dc2a4fc40a537e  ./out/b1-5.log
bd0791913daba88abbe3c61c15bb8660ea7a4eda3e0b71d4747ceffd4bbc6f0a  ./out/b1-6.log
f100c2bdf8b7e9403f158638ee3cebd993ff225612a72a424b9ed91d425d0243  ./out/b2-1.log
7fffdcabeb66f85a16648f705ba0facdcc89fd48c51d6ce4e67ead9a51f7781b  ./out/b2-2.log
9e3d303072295836b48c9eedae4619f49083135ce9b734884b48b847f1471ff6  ./out/b2-3.log
a2fd667a376742ba88701a255430b0388bdf7596730a8f01f5a286d09bf5d4e4  ./out/b2-4.log
dec1d0605129a888306b449be647aeb2dd8aac57a0d072fa464b2128df7205e9  ./out/ledger-probe.log
e7edbc529ec121a4156df5061b597e147a58b06c21689fb3ff84a83009abec91  ./out/nvml-shim-dlsym-all.err
18a9df369fa3ca174560083d70d5404a56e083fdf2f0860d8ee2ae6d386e6c12  ./out/nvml-shim-run1.err
b74102e75581a173086dce882ef662719bfa1a685bd6cbbf7f312757b95815cf  ./out/nvsmi-apps-live-1.txt
3f35a352c3fe25fcb69c351f8654d82743e38f864cf01ed03ad476d0eca7c918  ./out/nvsmi-gpu-live-1.txt
cb6c2e7bae578c9be84684c1ca2c3f1c49a77f64c21dd61d6156d30eaaa2cb80  ./out/nvsmi-q-live-1.txt
9fb9d53be0cfb81770db8838ffe303f4c84b67175ad67f7870be62856ae6b161  ./out/nvsmi-q-memory-pids.txt
2cf20fd3bedb6f4eb05f4ada06e0f884fdbabaec1119195da75b411e494ddf3c  ./rerun.sh
```

`out/ledger-probe.log` 是 debug 档那一次（`rerun.sh ledger` 跑 release 档，两档算的是同一串整数）。

## K1-1：一次读 `nvidia-smi -q` 的解析与四形

### 读出来的是不是同一个量（量过，只读）

本机空闲时原样（`out/nvsmi-q-memory-pids.txt`）两张卡 FB 段是 `Total : 32607 MiB`、`Reserved : 497 MiB`、`Free : 32110 MiB` 与 `16311 / 462 / 15849`，`Processes : None`；同一刻 `nvidia-smi --query-gpu=pci.bus_id,memory.total,memory.reserved,memory.used,memory.free --format=csv` 原样：

```
00000000:01:00.0, 32607 MiB, 497 MiB, 2 MiB, 32110 MiB
00000000:10:00.0, 16311 MiB, 462 MiB, 2 MiB, 15849 MiB
```

主 agent 的一个 GPU 用例进程（`record_checker_judges_absence_by_the_persisted_set`，pid 293696）在卡 0 上时连读三次，三次相同（`out/nvsmi-q-live-1.txt`、`out/nvsmi-apps-live-1.txt`、`out/nvsmi-gpu-live-1.txt`）：`-q` 里 `Process ID : 293696`、`Type : C+G`、`Used GPU Memory : 2687 MiB`、FB `Free : 29404 MiB`；`--query-compute-apps` 是 `293696, 00000000:01:00.0, 2687 MiB`，`--query-gpu` 是 `00000000:01:00.0, 29404 MiB, 2708 MiB`。两种读法取的是同一个量。`nvidia-smi` 二进制里有 `M+C+G` 这一串（`strings /usr/bin/nvidia-smi`），同一个进程的图形与计算两类并成一行列出。

解析真报告（`OPUS10Y-PARSE real_report=`，`out/ledger-probe.log`）：两张卡 `free 32110 / total_less_reserved 32110 / NotListed` 与 `15849 / 15849 / NotListed`。认不得的输出（造的变体）：

```
OPUS10Y-PARSE no_reserved=Err("nvidia-smi 的报告里总线 01:00.0 那张卡的 Reserved 读不出")
OPUS10Y-PARSE not_available=Ok([CardMemoryReading { free_mebibytes: 20000, total_less_reserved_mebibytes: 32110, occupied_by_this_process: NotListed }])
OPUS10Y-PARSE twice=Ok([CardMemoryReading { free_mebibytes: 20000, total_less_reserved_mebibytes: 32110, occupied_by_this_process: OnlyThroughTheBooks { mebibytes: 6000 } }])
```

缺 Reserved（老驱动没有这一行）报错，卡池那一路 panic 判红，不悄悄当 0；`Used GPU Memory` 读不出记成没列，走「借着的当没建、驱动那一份当已在卡上」那一支（见乙附）；同一个进程号列两行就加起来（「(Some(Ok(earlier)), Some(used)) => Ok(earlier.saturating_add(used)),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:737`）），两行是两份上下文时对，是同一份用量的两种列法时算两遍——驱动今天并成一行（上面 C+G），没见过拆开列的，推的。

### 甲：一份报告不是一个时刻（打中，窄化没消除）

卡池每借一次跑一次 `nvidia-smi`（「let report = run_the_report()?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:792`）），文件头说空闲与这个进程占着的「出自同一份报告」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:13`）。同一份报告里这两样是 `nvidia-smi` 先后两次问 NVML 拿的。量法：`LD_PRELOAD` 一个截 `dlsym` 的小库进 `nvidia-smi`（`nvml-shim/shim.c`），把读显存的 NVML 函数换成先记时刻再转发：

```
NVMLCALL nvmlDeviceGetMemoryInfo_v2 dev=0x70ffacc80898 t_us=112227489255.4..112227489266.2
NVMLCALL nvmlDeviceGetMemoryInfo_v2 dev=0x70ffaccf5298 t_us=112227490614.1..112227490622.0
```

（`out/nvml-shim-run1.err`；输出的报告与不截时逐字相同，`diff` 只差 Timestamp 那一行。）两张卡读显存相隔 112227490614.1 − 112227489266.2 = 1347.9 µs；进程表不经 `dlsym` 取（`out/nvml-shim-dlsym-all.err` 列出的 17 个名字里没有列进程的函数，走 `nvmlInternalGetExportTable`），它的时刻量不到。卡 0 的进程表按输出次序在卡 0 读显存之后、卡 1 读显存之前，窗口 ≤ 1.35 ms 是推的；整次调用 0.06 秒（`/usr/bin/time` 五次都是 `0.06 s`）。

这个进程别的线程借到之后在锁外建缓冲；另一条线程借的时候在锁里跑这一次 `nvidia-smi`。建上那一笔落在两次查询之间时：空闲是建之前的，占着的是建之后的，同一笔算两遍——第 10x 批 K1-1 那一形，窗口从两条命令之间缩到一条命令里两次查询之间。模型（`opus_10y_a_torn_report_counts_a_lease_built_between_the_two_queries_twice`，真的解析与容量函数）：

```
OPUS10Y-TORN configured=Some(32000) outsider=10000 x=600 lease=8000 room_before=11550 room_after=11550 room_torn=19550 over_by=8000 free_after_build=13510 truly_usable=11550 torn_minus_truly=8000
OPUS10Y-TORN configured=None outsider=10000 x=600 lease=8000 room_before=6021 room_after=6021 room_torn=12021 over_by=6000 free_after_build=13510 truly_usable=11550 torn_minus_truly=471
OPUS10Y-TORN configured=Some(32000) outsider=16000 x=600 lease=4000 room_before=9550 room_after=9550 room_torn=13550 over_by=4000 free_after_build=11510 truly_usable=9550 torn_minus_truly=4000
OPUS10Y-TORN configured=Some(32000) outsider=0 x=600 lease=8000 room_before=21440 room_after=21440 room_torn=21440 over_by=0 free_after_build=23510 truly_usable=21550 torn_minus_truly=-110
```

配置的额度不卡住时（卡上有别的用户），撕开的那份报告多给整笔借据（8000、4000 MiB），比卡上此刻真能再给的大同样多；额度卡住或 3/4 那一份卡住时少一些或为 0。这不是用户认的两进程竞态：两样都是这个进程自己的。

四句：分辨「一次读」与「两次读」两臂——两臂都中，一次读的窗口窄得多（不分辨）；系统看不看得到：看不到，一份报告里没有时刻；字面：判决第三节 K1-1 第一条「同一次 `nvidia-smi` 调用，或者用一次读出两个量的读法」——前一个分句做到了，后一个分句（一次读出两个量）没做到，`-q` 是两次查询；改法：判决给的两个改法里「同一次调用」对这一格不起作用。我的零轮改法（只在我的模型上推过、没实现、被攻过零轮）：一次借读两份报告，空闲与占着的各取两份里小的那个——建上落在第一份窗口里时第二份的空闲已扣掉它，落在两份之间时第一份占着的没有它，落在第二份窗口里时两份占着的取小没有它，三种都不算两遍（推的）；或者把「这个进程能占的」封顶到 Total − Reserved（只挡住超过整卡的那一截，推的）。

### 乙：没走卡池的设备，记总线号之前那一段（打中，窗口）

`start_the_card` 先 `request_device`（「.map_err(|error| failure_of_a_device_request(&error, &which_card))?;」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:351`）），再编单元校验和的管线，最后才记总线号（「.insert(discovered.pci_bus_identifier.clone());」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:386`））。这中间卡池读一次，这个进程那一行已经含新设备的上下文，读数器认成都走这份账（`OnlyThroughTheBooks`），那一份当成借据已建上。

同一个进程里两样同时起是常态：崩溃放量取卡时只看卡池起没起（「if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5263`）），同一个测试二进制里别的用例还没起池时，它自己在卡上起设备；之后有用例起池、借卡，两件在 cargo 的几条测试线程上并行。`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里量速度那一条直接取整张卡（`first_usable_card_or_report_not_run`，`#[ignore]`，带 `--include-ignored` 时与卡池用例同进程并行）。

模型（`opus_10y_an_outside_device_not_yet_marked_and_the_driver_share_taken_as_on_the_card`）：

```
OPUS10Y-OUTSIDE outsider=10000 x=600 y=500 lease=8000 free=21010 room_unmarked=11550 room_marked=13010 room_not_listed=13010 truly_usable=11050 unmarked_minus_truly=500 marked_minus_truly=1960
OPUS10Y-OUTSIDE outsider=10000 x=600 y=1200 lease=8000 free=20310 room_unmarked=11550 room_marked=12310 room_not_listed=12310 truly_usable=10350 unmarked_minus_truly=1200 marked_minus_truly=1960
OPUS10Y-OUTSIDE outsider=10000 x=2560 y=500 lease=8000 free=19050 room_unmarked=11550 room_marked=11050 room_not_listed=11050 truly_usable=11050 unmarked_minus_truly=500 marked_minus_truly=0
```

没记上那一刻多给的正好是那个设备此刻占的 y。四句：分辨「起之前记」与「起之后记」两臂（推的，没实现）；系统看得到——它就是起设备的那个进程，起之前就知道要起；字面：判决第三节 K1-1 第二条「本进程没走卡池的设备算进『本进程占着的』」，有一段时间没算进；改法：先记总线号、再 `request_device`（起不来也照「分不出」算，只会少借，推的，零轮）。

### 乙附：「读不到 / 分不出就当驱动那一份已在卡上，只会少借」不成立

文件头那一句：「借着的当一点都没建、起了设备的驱动那一份当已经在卡上，只会少借。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:26`）。实现是「mebibytes_left_for_the_driver.min(promised_mebibytes)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:168`）：把 D = 2560 整份当已占在卡上。列了这个进程的那一路只把实占的 x 算进去，D − x 留给驱动往上长（判器内核这类管线算在 D 里）。上面第一、二行 `room_marked` 比列出来的那一路（`room_unmarked` 减 y）多给 D − x = 1960 MiB；x = D 时（第三行）为 0。物理上撞不撞：要看驱动之后会不会真长到 D——上面那个活着的用例进程实占 2687 MiB，其中多少是上下文、多少是缓冲分不出，没量（推的）。这一形与「驱动固定开销 2560 MiB 没上卡量」不是一回事：那条是 D 的值，这条是同一个 D 在两条路上一条当许出去、一条当已占上。

### 丙：销了账、卡上还没还的那一份（部分复现）

每读一次照「.min(mebibytes.saturating_sub(driver_on_the_card_mebibytes))」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:197`）往下压，驱动那一份按 D 扣（「DriverShareOnTheCard::DeviceStarted => mebibytes_left_for_the_driver,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:192`））。x < D 时 m − D 比卡上还占着的还没还的那一份小，压过头的那截在下一笔里当成活借据已建上。真的 `GpuMemoryLedger` 走一遍（借 L1、记设备起好、丢 L1、读一次、再借）：

```
OPUS10Y-RETURNED x=600 l1=4000 free_now=17510 room_first_read=17510 room_second_read=17510 truly_usable_while_l1_still_on_card=15550 over_by=1960
OPUS10Y-RETURNED x=300 l1=6000 free_now=15810 room_first_read=15810 room_second_read=15810 truly_usable_while_l1_still_on_card=13550 over_by=2260
OPUS10Y-RETURNED x=2560 l1=4000 free_now=15550 room_first_read=15550 room_second_read=15550 truly_usable_while_l1_still_on_card=15550 over_by=0
```

多给的就是 D − x，与乙附同一个根：D 在压 R 那一步当已占上。四句：分辨 x = D 与 x < D（量过：0 对 1960 / 2260）；系统看不看得到 x：看不到，`nvidia-smi` 不拆上下文与缓冲；字面：判决第三节 K1-1 第三条「已销账、卡上还没还的缓冲不当别人的已建部分」——x < D 时还有一截被当成了；改法：压 R 时驱动那一份按 0 扣（只会把 R 压得更少、少借，推的，零轮）。

### 丁：空着时能借的上界取 Total − Reserved（本机没打中，有条件）

上界照「reading.total_less_reserved_mebibytes,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:229`）算，Total − Reserved 照「total_less_reserved_mebibytes: total_mebibytes.saturating_sub(reserved_mebibytes),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:775`）。本机两张空卡上它逐 MiB 等于报的空闲（上面 `--query-gpu` 原样）。卡上有常驻的别的用户（显示服务、常驻的计算服务）时，空卡也到不了这个数：

```
OPUS10Y-CEILING resident=400 room_with_only_the_resident=21221 ceiling=21521 need=21222 decision=NoRoomNow
```

要 21222 到 21521 MiB 的用例在这张卡上每一次都是本次未跑，永远不判红——第 10x 批「空卡读到的空闲比总量少 497 MiB，有一段要的量永远报本次未跑」那一形，差额从 Reserved 换成了常驻用户。两台机器今天都没有常驻用户（`host-resources.py`：「别的计算进程：没有」），够不着。

## K1-3：Out of Memory 接住之后、设备丢了重起

### OOM 按整台设备认，另一条线程的 OOM 把这条线程上真的 panic 吃掉（读码，推的）

卡线程上的活 panic 了、自己的错误范围没接到时，看设备上从活开始起记没记过 OOM：「(Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2016`），取法是「.out_of_memory_explaining_a_failure_since(out_of_memory_reports_before);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2040`），判的是「if self.out_of_memory_reports() > reports_before {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:159`）。计数在设备上，卡池里一张卡这个进程只有一个设备，所有借这张卡的用例、崩溃放量的卡线程与单元校验和那几条线程共用它；范围接到的那一次也往设备上记（「card.device_condition().record_an_out_of_memory(format!(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2042`））。

一段历史：同一个测试二进制里用例 A 在卡 0 上撞了 OOM（两进程竞态那一形，A 自己照规矩报本次未跑），同一刻崩溃放量在卡 0 上判一块，判器里一处 wgpu 校验错（真的 bug）让没接住的错误回调 panic；这条线程的范围是 Out of Memory 范围，接不到校验错，设备计数却在这段时间里涨了 → 这一块报「显存不够」本次未跑，不判红。另一支更宽：设备丢了、丢之前报过一次 OOM（「(Some(lost), Some(last_out_of_memory)) => Some(format!(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:166`）），不看 `reports_before`，之后在那个旧设备上的每一次失败都认成显存不够。

四句：分辨「按设备认」与「按线程认」两臂（推的）；系统看不看得到哪条线程记的：今天记的时候不带线程，看不到；字面：判决第三节 K1-3「这几处的 Out of Memory 都接住，报本次未跑」——这一形是把不是 OOM 的也接了，满足的不是那一句，归到 K2-2「判红」的反面（真红报成未跑）；结局是本次未跑，不是通过（`77` 不当绿），之后没有竞态的一趟会照红。改法（零轮）：记 OOM 时带起活那条线程给的一个号，判的时候只认自己这一趟（含它另起的编译线程）记的。

### 设备丢了重起，不另登记驱动那一份（读码，推的）

重起在「|started: &GpuCard| started.device_condition().lost_reason().is_some(),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1282`）判，起好了只把驱动那一份再记成已占（「.record_the_device_started_on_the_card(card_position);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1289`）），不像第一次那样要它连那一笔一起放得下。触发重起的那一笔是按「驱动那一份已在卡上」借的（`DeviceStarted`），新设备的上下文没留地方；旧设备还被别的借据的 `GpuCard` 克隆拿着时，卡上同时有两份上下文，账上一份。多给的上界是一份上下文（≤ D，推的）。新设备在 `request_device` 时撞满会报本次未跑（显存不够那一类），后面的缓冲撞满由 OOM 那一层接——结局多半是本次未跑而不是红，代价是账在那一刻给大了。

### 不缓存 Err（没打中）

`DeviceStartedOnDemand::started_or_start` 起不来不留着、下一次再起，单测 `a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced` 有变异行。

## K2：只防 OOM、不压并发两头

- K2-1 给大：上面甲（≤ 一笔借据）、乙（≤ 那个设备此刻的占用）、乙附与丙（≤ D − x）、重起（≤ 一份上下文）。没找到上界、回调引进来的另外的给大形。
- K2-2 放得下却未跑 / 判红 / 弃卡：只有丁（有条件、两台都够不着）。读码核了两种结局的出口：此刻带不了而空着时带得了走 `ScratchBeforeBooking::NoCardHoldsAStateNow` → 本次未跑；空着时也带不了才 `None` → 判红；借的那一刻每张都借不到走 `NoRoomOnAnyCardNow` → 本次未跑；借到的卡起设备显存不够那张不用、全不够报本次未跑。没找到「放得下却判红」的历史。
- 用户已定的两进程同读一份空闲不算，上面各形都是单进程里的。

## K3：改回去照绿（读码加 grep，推的）

认出没走卡池设备的接线有两处：起设备之后记总线号（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:386`）与把判的函数交给读数器（「Box::new(has_devices_started_outside_the_card_pool_on_the_bus),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1686`））。`grep -c 'BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL\|has_devices_started_outside_the_card_pool_on_the_bus\|insert(discovered.pci_bus_identifier' crates/mutations.tsv` 原样输出 `0`；`grep -rln` 同两个名字在 `crates/` 的 `.rs` 里只命中 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 自己。harness 档那几条变异（读数器认不认 `AlsoOutsideTheBooks`）在解析函数上，碰不到这两处：把记总线号那一句删掉、或在 1686 行交一个恒假的闭包，推的照绿。同一个盲区也罩着卡池用哪个读数器（`nvidia_smi_memory_and_processes_report` 在 `crates/mutations.tsv` 里 0 行）：把卡池改回两条 `--query-*` 读、拼成一份，harness 档的「一次读一份」单测照样绿（它测的是 `card_memory_reader_of_one_report_per_reading` 自己）。

## K4：C694 的 B、C 与 T1 甲

### 读码：抬 F 那一遍拿记着的根顶上之后，账照样从盘上再读

T1 甲那一句（`.claude/kb/decisions/16-发布语义.md` 第 37 行「重建分配器两遍里只有一遍自证过的槽」那一段）：「仍坏不拒，照挂载时读出、记在根环表里的那条根算：有效根照它本身进上限与候选集，被抛弃根照它记着的落点进影子账、不进上限（这一处置的实现被攻过零轮）」（`.claude/kb/decisions/16-发布语义.md:37`）。

实现：重读仍坏时顶上的是那条根记录（「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」（`crates/singlefs-core/src/mount.rs:3505`）），交回一串根（「let roots = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;」（`crates/singlefs-core/src/mount.rs:3668`）），影子账重算对每条被抛弃根照旧从盘上读账（「let placements = match placements_referenced_by_root(devices, unit_area_start, root) {」（`crates/singlefs-core/src/mount.rs:1536`）），读不出就走 C694 的 B：从映射现算、重读一次、看现行账有没有凭据（「if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {」（`crates/singlefs-core/src/mount.rs:1608`）），没有就拒（`crates/singlefs-core/src/mount.rs:1612`）。「记着的落点」在分配器根环表里（被抛弃根那一项「账读不出的、影子账关着（只供测试）的，这一串是空的：它什么都没隔离，离开根环时也什么都不清。」（`crates/singlefs-core/src/allocator.rs:1026`）说的就是它带着落点），抬 F 这一遍没用它；它只在进程内记根时补隔离用（`crates/singlefs-core/src/allocator.rs:2101`）。B 的文档写明抬 F 也拒：「没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬」（`crates/singlefs-core/src/mount.rs:1486`）。

挂载那一刻 C 的账读得出，C 引用的槽都隔离了，新实例分配不到，B 的凭据（现行账里有一条代大于 C 的记录罩住 C 的树表槽）在这次挂载里永远不成立。所以挂上之后 C 的账一读坏，抬 F 必拒——T1 甲那一槽（C 是两遍里只有一遍自证过的被抛弃根）也拒，与「仍坏不拒」「照它记着的落点」相反。

### 量：C 的账挂上之后读坏，正常卸载

起点照第 10i 批攻方的 `abandoned_start` 重写（after = 0）：崩溃恢复落到 C 之前那一条、取新实例，C 留在环里成被抛弃根，盘宽 384 槽与 4 GiB 两档、C 之前覆盖写 1 次或 3 次，共 4 个起点。每个起点挂上（实例 4，C 的账读得出、记下落点），用户覆盖写 j 次（j ∈ {0, 1, 2, 3, 5, 8, 12, 20}，放开扫），之后 C 的树表、中央映射树根、分配记录树根三处单元从此读坏（`FaultDevice`，两份位置都坏），正常卸载。C 的槽三种：平常；S1（第 10i 批 s1_plans 的 a 形：重建那一遍某一块盘上的有效根全读坏、整环重读，C 的槽从重读那一遍起一直坏；d0、d1 两种），共 4 × 3 × 8 = 96 段，两臂各一遍。臂：今天（冻结副本）；`recorded`（`opus-k4-arms.patch`：只在抬 F 那一遍，账读不出时照分配器根环表里记着的落点算，挂载那一遍不动）。被拒之后用户的下一步也放开走：接着写到 C 那一槽被这个进程写掉为止（至多 60 次）再卸载；崩了、账仍读不出时再可写挂载；故障消失之后再挂、覆盖写 6 次，每次核池级 checker 的 I-7.4。

`python3 k4agg.py out/b1-*.log` 原样（每行：段数、盘宽、臂、C 的槽、卸载之前 C 那一槽被写掉没有、卸载结局、卸载之后 I-7.4 条数、被拒之后接着写、第二次卸载、账仍坏时再挂、故障消失再挂与 6 次写之后 I-7.4 最多几条）：

```
2 384 prod S1 C-in-ring unmount=Err(AccountOfAnAbandonedRoot) after_unmount_I74=- writes_after_refusal=ok second_unmount=Ok remount_with_fault=Ok worst_I74_after_clean_remount=0
26 384 prod S1 C-in-ring unmount=Err(AccountOfAnAbandonedRoot) after_unmount_I74=- writes_after_refusal=refused second_unmount=Err(AccountOfAnAbandonedRoot) remount_with_fault=Ok worst_I74_after_clean_remount=0
4 384 prod S1 C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
1 384 prod plain C-in-ring unmount=Err(AccountOfAnAbandonedRoot) after_unmount_I74=- writes_after_refusal=ok second_unmount=Ok remount_with_fault=Ok worst_I74_after_clean_remount=0
13 384 prod plain C-in-ring unmount=Err(AccountOfAnAbandonedRoot) after_unmount_I74=- writes_after_refusal=refused second_unmount=Err(AccountOfAnAbandonedRoot) remount_with_fault=Err(AccountOfAnAbandonedRoot) worst_I74_after_clean_remount=0
2 384 prod plain C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
18 384 recorded S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
10 384 recorded S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=1
4 384 recorded S1 C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
14 384 recorded plain C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Err(AccountOfAnAbandonedRoot) worst_I74_after_clean_remount=0
2 384 recorded plain C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
28 4g prod S1 C-in-ring unmount=Err(AccountOfAnAbandonedRoot) after_unmount_I74=- writes_after_refusal=ok second_unmount=Ok remount_with_fault=Ok worst_I74_after_clean_remount=0
4 4g prod S1 C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
14 4g prod plain C-in-ring unmount=Err(AccountOfAnAbandonedRoot) after_unmount_I74=- writes_after_refusal=ok second_unmount=Ok remount_with_fault=Ok worst_I74_after_clean_remount=0
2 4g prod plain C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
22 4g recorded S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
6 4g recorded S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=1
4 4g recorded S1 C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
14 4g recorded plain C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Err(AccountOfAnAbandonedRoot) worst_I74_after_clean_remount=0
2 4g recorded plain C-left unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=0
合计 192
```

（`Err(AccountOfAnAbandonedRoot)` 是 `NewerStateStillUnreadableAfterOneReread` 里的那个成员，汇总时去掉了外层名；原行在 `out/b1-*.log`，例如 `out/b1-1.log` 里 `unmount=Err(NewerStateStillUnreadableAfterOneReread/AccountOfAnAbandonedRoot)`。）

读法：卸载之前 C 还在环里的 168 段（84 段今天、84 段 `recorded`），今天 84 / 84 卸载被拒，`recorded` 84 / 84 做成、卸载之后 I-7.4 都是 0。384 槽盘上更重：今天被拒之后接着写，42 段里 39 段每一次写都被拒（`FloorRaiseFailedWhilePushingForSpace`，里面同一个 `AccountOfAnAbandonedRoot`：准入要推抬 F，抬 F 读 C 的账被拒），写不出去 C 那一槽就永远不会被写掉，第二次卸载照拒；平常那一类里 13 段崩了再挂也被拒可写（B 在挂载那一道），账一直坏就一直只读。4 GiB 盘上准入不推抬 F，接着写 7 到 20 次写掉 C 那一槽（`grep -oh` 数 `then_writes=`，42 段），第二次卸载做成。C 那一槽在卸载之前已被写掉的（j = 20 那几段）两臂都做成。

对照（`out/b2-*.log`，同 96 段 × 两臂，不注账的读坏，`OPUS_K4_NO_ACCOUNT_FAULT=1`），`k4agg.py` 原样里 I-7.4 那几行：

```
10 384 prod S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=1
10 384 recorded S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=1
6 4g prod S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=1
6 4g recorded S1 C-in-ring unmount=Ok after_unmount_I74=0 writes_after_refusal=- second_unmount=- remount_with_fault=Ok worst_I74_after_clean_remount=1
```

其余 176 段 I-7.4 为 0，合计 192。b1 里 `recorded` S1 那 16 段的 I-7.4 与对照里两臂各 16 段同形：卸载做成之后，下一次挂载时 C 的槽两遍都读坏（S1 那条故障一直在），C 看不见、它的槽没隔离、被这次挂载写掉，故障消失再挂时 I-7.4 当场红（`first_I-7.4=Some("at=remount_clean I-7.4: 被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头…`，`out/b2-1.log`）。这是已知的 C554：「被抛弃实例的根在重开时全部暂时读不出 ⇒ 影子账读不到它们的账、隔离 0；新实例复用了它们的槽」（`.claude/kb/checks-owed.md:426`）。两臂一样中，不分辨臂；今天那一臂在 b1 里不中，只因为卸载与写都被拒、什么都没写。

四句（K4-2 卸载那一格）：
- 分辨臂：分辨。今天 84 / 84 拒，`recorded` 84 / 84 做成，卸载后 I-7.4 两臂都没有可比的红（今天没卸成），之后的红两臂同额（对照）。
- 判别子：系统看得到——记着的落点就在这个进程分配器的根环表里，抬 F 时不用读盘。
- 字面：S1 那 56 段满足 T1 甲「仍坏不拒…被抛弃根照它记着的落点进影子账」的反面（今天拒、且没照记着的落点）；平常那 28 段（C 不是 S1）今天的拒合 B 的代码文档「抬 F 拒这一次抬」，不违反 T1 甲，只与 C694 那一行的「正常卸载、会话写、再可写挂载三条路都该做得成（U1）」（`.claude/kb/checks-owed.md:545`）对不上，而 C694 那一行管的是账的单元被复用那一形，挂上之后读坏的这一形它没写——归到条款空白，不算打中。这一形的故障（挂上之后被抛弃根的账的单元读坏）：注入框架写得出（「OffsetExactly(DeviceOffsetInBytes),」（`crates/singlefs-harness/src/fault_injection.rs:430`）与「EveryMatchingCallFromTheNthOnward(u64),」（`crates/singlefs-harness/src/fault_injection.rs:462`）），有没有用例把它注在被抛弃根的账单元上我没查；随机历史里点名的那一形不是它，注册的那一形是「下一次可写挂载时根环里最新那条根（这个会话的现行版本）的根槽与它那次发布的记录点名的单元整次挂载读不出（读回全 0；」（`crates/singlefs-harness/src/history.rs:489`），挂载那一刻起就坏、不是挂上之后才坏。属不属于要防的故障由主 agent 判。
- 改法：`recorded`（只在我的模型上量过、被攻过零轮）修卸载与会话写两格；碰不到「账一直坏时崩了再可写挂载被拒」那 28 段——那一道在挂载里，新进程没有记着的落点，拒合 B 的字面。

| 改法 | 卸载被拒（S1） | 卸载被拒（平常） | 384 槽盘会话写全拒 | 账仍坏时再可写挂载被拒（平常） | C554 那一形 |
|---|---|---|---|---|---|
| `recorded`：抬 F 那一遍照根环表记着的落点 | 修（量过：56 → 0） | 修（量过：28 → 0） | 修（推的：写被拒的根就是这一次抬 F；`recorded` 那一臂卸载先做成，没再接着写） | 不修（量过：28 段里 28 段仍 Err） | 不修（量过：对照两臂同 16 段） |
| 只给 S1 那一类照记着的落点（T1 甲字面） | 修（推的） | 不修（推的） | 只修 S1 那 26 段（推的） | 不修（推的） | 不修（推的） |

### K4-1（落点掉出影子账）：没打中新形

试过的：上面 192 + 192 段里，卸载之后、再挂之后 6 次写，I-7.4 只在 C554 那一形红，两臂同额。读码：C 的预隔离（C694 的 C）只认水位之上的记录，这几个起点里 C 的记录在水位之下（C 是实例 2，挂载时所选那一版是实例 3 之后的），C 这一支没走到；B 的凭据在挂上之后 C 被隔离着时不会成立，不会把活着的 C 判成已被占。T1 甲顶上的根照样过 B 那一套，所以走不到「顶上了却漏掉」那一支：要么照账隔离、要么拒。

### K4-2（可写挂载被拒）：可写挂载那一道没打中

可写挂载被拒只在账在挂载时就读不出、又没有凭据时（上表「账仍坏时再可写挂载」那一列），合 B 的定案字面。

## 没打中的形状

- K1-1 读法：两种读法取的量（量过，相同）；BAR1、Conf Compute 两节的 Total / Used / Free 盖不掉 FB 那一节（真报告与造的报告都解析对，缩进 ≤ 4 才换节）；`Processes : None`；进程名一行里带冒号。
- K1-1 丁：本机两台没有常驻用户，够不着。
- K1-3：起设备失败不缓存（读码）；`request_device` 的 OOM 与别的失败分开（读码）。
- K2-2：「此刻带不了」与「没有要装的」分开、两种结局的出口（读码）；`pending_states` 为 0 走不到借草稿区（`pending.is_empty()` 先返回，读码）。
- K4-1：96 段 × 两臂 × 两种（注 / 不注账的读坏）里，I-7.4 只在 C554 那一形，两臂同额。没造到的：C 的记录在水位之上（C694 的 C 那一支）与 T1 甲一起的起点——这几个起点 C 的记录都在水位之下。
- K4-2：可写挂载那一道只在合 B 字面时拒。

## 取样、估时与跑的量

- 显存账：每形 1 到 4 个取样点（上面原样），不是扫描；结论只说「有这一形、量多大」，不说最坏。
- K4：先跑一小段（384 槽、j ∈ {0, 3}、debug 档）24 段 218.58 秒，估全量 192 段 debug 档约 29 分钟；改 release 档，4 GiB 一段约 3.6 秒（3 段 13.4 秒）。全量没缩：b1 六批（384 两批各 48 段，4 GiB 四批各 24 段）、b2 四批各 48 段，每批一个进程、`capped.sh 1`、内存上限 4G，十批分两次并行，各批 `rc=0`，`.rc` 文件数对上。
- 没调层 0 枚举：这一形是挂上之后的读故障，不是崩溃点，原型里没有写流要枚举，`closed_form_state_count` 没用上，状态数计 0。
- 跑之前看负载：别的会话有 `cargo test -p singlefs-harness -- --ignored`、checker 档 GPU 用例与另一条攻方原型在跑，没有性能测量；没等锁。

## 这条腿自己的限度

- K1 / K2 那几形的量都在造的报告与真函数上算，没在卡上真建缓冲去撞（派发说只读、不占卡）；物理上撞不撞 OOM 写了「推的」的那几处，取决于驱动实占 x 与 D 差多少，没量。
- 甲的窗口：只量到两张卡读显存相隔 1347.9 µs，进程表那一次的时刻量不到，窗口 ≤ 1.35 ms 是推的。
- K1-3 两形与 K3 都是读码加 grep，没跑 checker 档（重型，不归我）。
- K4 的 `recorded` 臂是我在副本里加的（`opus-k4-arms.patch`，只在抬 F 那一遍），只在我的模型上量过、被攻过零轮；它照槽号找记着的落点，没核那一槽里的根与记着的是同一条（这个进程写掉那一槽之后那一项就换类，推的）。
- K4 的起点是第 10i 批起点的重写，C 的字节在实例 3 挂完之后写回，是造出来的盘面；四个起点、一种用户写法（覆盖写），删除与新建 inode 没扫。
- 一次抽样：「没打中」那几格按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」要再抽一次才算。
- 模型源码 `opus_10y_k4_probe.rs` 第三段一次写进 161 行，超过每次 150 行那一条；报告各段都在 150 行以内。

## 没做什么

- 没跑重型测试（checker 档、门禁、变异整表）；没占显卡。
- 不判正推那几格，不替主 agent 采纳；副本上的数不算入库装置上的数。
- K4 没造 C 的记录在水位之上的起点（C694 的 C 那一支），没造删除类发布的用户动作。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10y-code-r1-opus/` 里的仓副本 `repo/`、编译目录 `target/`、比对用的 `cmp/` 交回前删掉；留下的是日志与 `nvml-shim/` 的两个 `.so`（模型目录里有源码与输出，`.so` 可由 `rerun.sh shim` 重编）。
