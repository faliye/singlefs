# m3-b44-code-r3 正推腿（Sonnet）报告：W13–W16 的代码做的是不是条款说的（2026-10-18）

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 L6 J1 J2 J3 J5 W13 W14 W15 W16 -->

## 0 前提与读法

- 立场：正推。从已定条款与 `crates/` 今天的实现推「应该是什么」，再逐格比；不替主 agent 采纳或出判决。
- 冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r3/repo/`；报告里 `crates/…` 的行号一律按它。我在 `sha256sum -c --quiet research/prompts/m3-b44-code-r3-snapshot/crates-sha256.txt` 上核过：冻结副本与我拷出的草稿副本（`~/.local/share/singlefs/work/m3-b44-code-r3-sonnet/repo/`）都对上 118 份（命令无输出、退 0）。kb、记录、判决的行号是仓里那份文件自己的行号，每一句都 `grep -nF` 过。
- 量法与复跑：模型目录 `research/prompts/m3-b44-code-r3-sonnet-model/`（探针两个、补丁一个、`tools/rerun.sh`、`out/`）。探针只在草稿副本里加，冻结副本一个字没改。
- 标记：**读码**＝读冻结副本推的；**量的**＝我在草稿副本上跑出的；**推的**＝由量的或读码再往外推一步。
- 判定三档：一致 / 冲突 / 规则没说。「冲突」把两边整行并排抄。
- 带 `（读码）` 的结论没造镜像。每条结论后写「推翻」＝什么现象出现就推翻它。

### 0.1 复跑过的事实（原样输出）

W13–W16 的用例在草稿副本（crates 与冻结副本逐份相同）上复跑，10 个测试目标全绿（4+2+1+4+2+6+4+5+12+42 条），另有 core 库单测 7 条绿（`data_unit_payload`、`deleted_inodes` 两族；筛选词在命令里，结果行 `7 passed; 0 failed`）（`out/existing-tests.txt`）：

```
journal_record_whose_new_root_segment_pointer_does_not_ascend_is_refused_by_the_writable_mount: 4 passed; 0 failed
newer_unreadable_root_counts_only_units_of_this_pool_as_overwriting_its_pointer_slots: 2 passed; 0 failed
writable_mount_refuses_a_deleted_inodes_root_whose_entries_are_wider_than_eight_bytes: 1 passed; 0 failed
writable_mount_rehearsal_and_the_real_run_read_each_block_once: 4 passed; 0 failed
writable_mount_reread_after_a_bad_read_reaches_the_disk_past_the_reads_shared_with_the_rehearsal: 2 passed; 0 failed
rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read: 6 passed; 0 failed
a_writable_mount_with_system_configuration_slot_zero_unreadable: 4 passed; 0 failed
acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics: 5 passed; 0 failed
instance_table_sweep_at_the_end_of_a_writable_mount: 12 passed; 0 failed
entries_after_mount_refuse_swapped_or_behind_devices: 42 passed; 0 failed
```

（用例绿只说明实现员写的断言成立；下面每格的判定靠读码对条款，用例是旁证。）

### 0.2 一句话总览

- 六格里「代码对得上条款原文」的占多数；对不上或条款没写的落点有三处，都附了可复现的量法：
  1. **L1 末句**：共用那一份不只罩预演，真做续做自己的读也记下，没有上界；量的斜率是被删每个 32 KiB 数据单元多读两份共 64 KiB 并长期占着（§L1.5）。
  2. **L2/L1 ⓑ**：「没走完就拒」只数块设备报错的读；读成功而字节坏造成的没走完不触发，真做因共用同一份坏字节，落盘之后才交同一个错（§L2.3，量的，探针 `probe_corrupt_bytes_window.rs`）。
  3. **L3 E2**：W16 没改的「进程内重读挂载时读坏的槽」在可写挂载里读的多半不是此刻读盘（§L3.3，读码）。
- L4、L5、L6 的判定对象都对得上；L6 有两处注释写得比今天的代码宽（§L6.3）。

## L1 读一次两边共用（W13、W15）

### L1.1 条款句对代码行

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | 「抬 F 与续做要读的东西在取号之前读一次，预演与真做共用这一份，预演的读不再穿到底下那块盘」（`research/prompts/m3-b44-code-r2-forks.md:9`） | 每块盘包一个 `DeviceReadOnceInThisMount`（`crates/singlefs-core/src/mounted_session.rs:571`），在 `establish_instance` 里一次建好（`crates/singlefs-core/src/mount.rs:6298` 起的 `devices`）；取号之前预演传 `&*pool.devices`（`crates/singlefs-core/src/mount.rs:5345`），取号之后真做传同一个 `devices`（`crates/singlefs-core/src/mount.rs:6499` 起、`crates/singlefs-core/src/mount.rs:6528` 起）。只进内存那一趟的盘 `DeviceKeepingWritesInMemory` 对底下那块盘「只读借成」（`crates/singlefs-core/src/mounted_session.rs:362`），没写过的块读到这一份上（`crates/singlefs-core/src/mounted_session.rs:487`） | **一致**（读码）。预演与真做读的是同一个对象里的同一张块表 |
| 2 | 翻面观测：「造出共用的那一份与真做取号之后读到的不同（取号之后有写改了它）」（`research/prompts/m3-b44-code-r2-forks.md:9`） | 写成的块记成写下的字节（`crates/singlefs-core/src/mounted_session.rs:762`）；写报错的块把原记着的丢掉（`crates/singlefs-core/src/mounted_session.rs:774`）；写先穿到底下那块盘、成功才记（「self.record_the_written_blocks(offset, bytes);」（`crates/singlefs-core/src/mounted_session.rs:873`）） | **一致**（读码）。取号之后的写改了的块，之后读到的是写下的字节；写失败的块下次穿到盘上 |
| 3 | 第二轮判决 J3 行：「读被预演吃掉并入岔路一」（`research/prompts/m3-b44-code-r2-main-verification.md:24`） | 预演读错不再被吃掉：见 L2 | 见 L2 |
| 4 | 第二轮判决 J2 行：条款没写「预演自己被别的错打断、没走到会不会撞 C841 那一步」（`research/prompts/m3-b44-code-r2-main-verification.md:23`） | ⓑ 是实现员补的那一句：`crates/singlefs-core/src/mount.rs:5362` 起 | **规则没说**（补的一句；见 L1.4 ⓑ、L2） |
| 5 | 岔路二：「先量再定」（`research/prompts/m3-b44-code-r2-forks.md:18`）；J3 行：「内存没上界交用户：岔路二」（`research/prompts/m3-b44-code-r2-main-verification.md:24`） | 记下的那一份「没有上限」（`crates/singlefs-core/src/mounted_session.rs:566`） | 与「先量再定」不冲突；但岔路二点名的是「只进内存的盘」，这里的记下那一份把无上界带进了真做（L1.5） |

推翻 1：任一次读在预演里穿到盘上读了一次、真做里又穿到盘上读了第二次同一块（不经重读那条路、也不在三段读盘里）；`out/` 里的读次数能量出（`rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read` 钉每个根环槽 8 次：读阶段与重建 6、判号 1、取号读半 1）。
推翻 2：写成功之后同一块的下一次共用路径上的读读到盘上而不是写下的字节。

### L1.2 `establish_instance` 里每一次读走哪一路（读码）

路的初值是共用：`crates/singlefs-core/src/mount.rs:6280` 的 `Cell::new(ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun)`。切到读盘只有一个入口 `with_reads_straight_to_the_disk`（`crates/singlefs-core/src/mount.rs:6824`），做完切回进来之前那一路。读盘那一路在 `read_by_the_route` 里直接 `self.disk.read_at`，不看也不改记下的那一份、读错也不进读错账（`crates/singlefs-core/src/mounted_session.rs:716`）。

| 序 | 调用（函数:行） | 读什么 | 走哪一路 |
|---|---|---|---|
| a | `PreviousVersion::units`（`crates/singlefs-core/src/mount.rs:6305` 调，`crates/singlefs-core/src/mount.rs:1166` 定义）：只有树表 0 条那一臂读盘，带文件那一臂借 `output.units` 不读 | 实例表第 0 片、树表、分配记录树节点 | `SharedBetweenTheRehearsalAndTheRealRun` |
| b | `next_instance_generation_to_acquire(&pool)`（`crates/singlefs-core/src/mount.rs:6316`；`crates/singlefs-core/src/transaction.rs:880`）：`highest_root_instance` 读全部根环槽（`crates/singlefs-core/src/transaction.rs:883`、`crates/singlefs-core/src/recovery.rs:3591`）加 `highest_system_configuration_instance` 读每盘两槽（`crates/singlefs-core/src/transaction.rs:771`） | 判号那一次 | 共用（记下） |
| c | `dry_run_of_the_publishes_after_acquisition`（`crates/singlefs-core/src/mount.rs:6383`）：写行与暖机预演；释放前读盘核跳过（`crates/singlefs-core/src/transaction.rs:6869`） | 发布路径落盘之前那一段的读 | 共用 |
| d | `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`（`crates/singlefs-core/src/mount.rs:6440`）：先清读错账（`crates/singlefs-core/src/mount.rs:5331`），第一趟分配器拷贝上走删除队列，空间不够才走第二趟 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`（`crates/singlefs-core/src/mount.rs:7110`），其中取号用 `ThroughTheDevicesOfTheRehearsal`（`crates/singlefs-core/src/mount.rs:7143`） | 取号读半、写行、暖机、抬 F、续做的读 | 共用 |
| e | `devices_without_the_selected_version`（`crates/singlefs-core/src/mount.rs:6465` 包在 `with_reads_straight_to_the_disk` 里；函数在 `crates/singlefs-core/src/mount.rs:5620`） | 每盘两槽系统配置；落后支逐个读所选那一版的单元；缺根那一半读根环槽「读一次重读一次」 | **读盘** |
| f1 | `acquire_then_publish_until_the_continuation` 的读半：`crates/singlefs-core/src/mount.rs:6869` 起包在 `with_reads_straight_to_the_disk(read_route, …)` 里调 `read_the_witness_before_the_expected_acquisition_writes`（`crates/singlefs-core/src/transaction.rs:1008`）：重算号、`highest_system_configuration_instance`（`crates/singlefs-core/src/transaction.rs:1082`）、屏障前逐盘核一遍（`crates/singlefs-core/src/transaction.rs:1084`）、屏障、屏障后再读见证值 | 取号写之前那几次读 | **读盘** |
| f2 | 同函数的写半：`write_the_witnessed_expected_acquisition`（`crates/singlefs-core/src/transaction.rs:1030`，`crates/singlefs-core/src/mount.rs:6882` 调）里每盘 `write_system_configuration_slot`（`crates/singlefs-core/src/transaction.rs:511`）读本盘两槽算世代号（`crates/singlefs-core/src/transaction.rs:520`）与算 F 生效值读根环（`crates/singlefs-core/src/transaction.rs:539`） | 写系统配置槽之前的读 | 共用（路在读半做完已切回：`crates/singlefs-core/src/mount.rs:6830`） |
| f3 | 写行、暖机、写行之后推抬 F（`crates/singlefs-core/src/mount.rs:7075` 起） | 发布路径的读 | 共用 |
| g | `continue_the_deletion_queue_until_it_is_empty`（`crates/singlefs-core/src/mount.rs:6528`） | 续做的读 | 共用 |
| h | 实例表清扫（`crates/singlefs-core/src/mount.rs:6540` 包在 `with_reads_straight_to_the_disk` 里） | 清扫判、抹之前重读、清扫发布的读 | **读盘** |

读盘三段（e、f1、h）里每一次读都真到盘上：`StraightToTheDisk` 臂直接 `return self.disk.read_at(offset, buffer)`，不经过记下的那张表（`crates/singlefs-core/src/mounted_session.rs:716`）。有没有漏进共用那一份：三段的闭包都在 `with_reads_straight_to_the_disk` 里，路的切回只在闭包返回之后（`crates/singlefs-core/src/mount.rs:6830`）；三段里的写与发布路径的读也走同一个 `Cell`，所以也读盘。有没有多读：读次数由 W15 的钉值守（根环槽 8 次，读阶段与重建 6、判号 1、取号读半 1），我复跑绿。

推翻：任一读盘段里出现一次读命中记下的那张表（读次数少于钉值）；或读盘段之外的 `read_route.set` 改了路。

### L1.3 三段各自要求现读盘的条款原文（现查，不照抄 W15 报告的编号）

W15 报告写的「D18 已定项 11」编号是对的：这三处都在 `#### 已定项 11` 的正文里（标题行在 `.claude/kb/decisions/18-块里携带什么信息.md:253`，标题写的是「单元类型标签的值域…」，但这一项正文装着实例表那一类、它的回收、可写挂载的顺序与逐盘核）。

| 段 | 条款原文 | 它要求现读盘的是哪一句 | 判定 |
|---|---|---|---|
| 逐盘核（e） | 「世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn」（`.claude/kb/decisions/18-块里携带什么信息.md:320`）；「那一槽读一次、重读一次都不是它」（`.claude/kb/decisions/18-块里携带什么信息.md:320`）。代码文档：「读阶段的见证读两槽都读得出、到这一遍才读不出：两次读之间的瞬时读错」（`crates/singlefs-core/src/mount.rs:5608`） | 「读一次、重读一次」要两次真读盘；「到这一遍才读不出」是拿读阶段那次读与此刻这次读比，此刻这次不能取读阶段记下的那一份 | **一致**（推）：条款字面写读一次重读一次，没有一句明写「不许取记下的那一份」，要求现读是由两次读之间的瞬时读错这个动机推出来的 |
| 取号读半（f1） | 「在第一道屏障之后、第一个取号写之前读一次」（`.claude/kb/decisions/23-journal的角色与格式.md:394`）；「判不出就在第一个取号写之前拒」（`.claude/kb/decisions/23-journal的角色与格式.md:394`） | 「第一道屏障之后…读一次」本身就是一次现读：屏障前后各读一次的意义在读盘 | **一致**（字面） |
| 实例表清扫（h） | 「抹头之前那次重读也算：重读报设备错、或读回的不是那个头，这次清扫都不删」（`.claude/kb/decisions/18-块里携带什么信息.md:316`）；开关开着的后台路另有「后台抹之前在同一串行段里重读那一槽的头」（`.claude/kb/decisions/18-块里携带什么信息.md:316`） | 「抹头之前那次重读」是判完之后再读一次头，不能取判那一遍读进记下的那一份 | **一致**（字面）。挂载末尾立刻判、抹那一路（开关关着）条款只写了「抹头之前那次重读也算」，没有单写「挂载里抹之前重读」一句；W15 把它当同一件事 |

推翻：在这三段里放一次读取记下的那张表仍过全部用例；或条款里有一句明写这三段可取读阶段的缓存（我在 D16、D18、D23 全文里按「重读」「再读」「见证」没有找到）。

另有一处值得并排看：条款对读阶段缓存写的是「只收读成且不是全零的落点」（`.claude/kb/decisions/23-journal的角色与格式.md:394`），今天共用那一份记下**每一次读成功的块**，全零的也记（`crates/singlefs-core/src/mounted_session.rs:680`）。读阶段缓存与共用那一份不是同一个东西（前者是 C554 乙-配置续「读阶段重做一遍」用，后者是岔路一），条款对后者没有对应一句：**规则没说**。后果见 L3.3 的 E2 一格（读回全零的槽记下之后不再被「重读」重读到盘上）。

### L1.4 ⓐ–ⓓ 每一样对条款

| 读法 | 对得上的那一句 / 条款没写的地方 | 代码 | 预演与真做结局会不会不同 | 会不会改掉无故障时写出的字节 |
|---|---|---|---|---|
| ⓐ 共用只罩抬 F、续做与它们的预演；逐盘核、取号写之前、清扫读盘 | 对得上：「预演的读不再穿到底下那块盘」（`research/prompts/m3-b44-code-r2-forks.md:9`）与上表三段 | 实际罩得比 ⓐ 的文字宽：判号（序 b）、写行与暖机的真做读（f3）、取号写半（f2）也走共用，因为写行、暖机的预演（c）要与真做同一份才能让「取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同」那条断言成立（`crates/singlefs-core/src/mount.rs:7044`） | 不会（同一张表）。但读盘三段与预演不是同一次读：瞬时读错能让真做的取号读半失败而预演过得去——这是条款要的（`.claude/kb/decisions/23-journal的角色与格式.md:394`） | 不会（读码）：无故障时共用读到的就是盘上字节 |
| ⓑ 两趟预演读错与没走到结局同时出现就拒，被另一份兜住的读错也算 | **规则没说**：岔路单把「没走完就拒」列成另一个没选的候选（`research/prompts/m3-b44-code-r2-forks.md:7`），选的那一条只写了共用；判决 J2 行承认条款没写「预演自己被别的错打断、没走到会不会撞 C841 那一步」（`research/prompts/m3-b44-code-r2-main-verification.md:23`） | 触发条件：`StepRefused` 不是 C841 或 `StoppedBeforeTheEndOfTheContinuation`（`crates/singlefs-core/src/mount.rs:5362`），且读错账非空（`crates/singlefs-core/src/mount.rs:5365` 的反面）；读错账只在底下那块盘的 `read_at` 报 `Err` 时记（`crates/singlefs-core/src/mounted_session.rs:667`、`crates/singlefs-core/src/mounted_session.rs:695`） | 见 L2.3：读成功而字节坏不入账 | 不会 |
| ⓒ 报成功的写记成写下的字节 | 对得上翻面观测「取号之后有写改了它」（`research/prompts/m3-b44-code-r2-forks.md:9`）；「写报成功而没落盘」条款没写 | `crates/singlefs-core/src/mounted_session.rs:873` | 说谎设备（`InjectedFault::WriteIsSwallowed`，`crates/singlefs-harness/src/fault_injection.rs:143`）上，挂载内之后的共用读读到写下的字节、不是盘上的；等于「只有被吞的那几个字节不同、其余照常」，与说谎设备的模型同构（推的） | 无故障时不会 |
| ⓓ 写行与暖机预演的读错走原来的错误成员 | 无新条款；原成员见 `crates/singlefs-core/src/mount.rs:6398`、`crates/singlefs-core/src/mount.rs:6411`、`crates/singlefs-core/src/mount.rs:6420` | 读错账在续做预演开头清空（`crates/singlefs-core/src/mount.rs:5331`），所以这两串里的读错不会变成新拒因 | 不会 | 不会 |

ⓒ 的「这次挂载之后读到的与真盘分不分得开」：分得开。记下那张表是 `establish_instance` 的局部变量（`crates/singlefs-core/src/mount.rs:6298`），交回的 `Mounted` 不含它（`crates/singlefs-core/src/mount.rs:6550`），之后的读直接到盘上。挂载内：被吞的写之后，共用路径读到写下的字节，读盘三段读到真盘；两者在说谎设备上可以不同（推的，没造）。
推翻 ⓒ：说谎设备上挂载做成，而挂载之后读到的字节与「吞掉那几个写、其余照常」的模型对不上。

### L1.5 记下那一份的内存有没有上界（量的 + 推的）

代码自己写「没有上限」（「记下的那一份活到这次挂载交回为止，读过的、写过的块都在里面，没有上限」，`crates/singlefs-core/src/mounted_session.rs:566`）。岔路二问的是只进内存那一趟记下的写（「只进内存那一趟记下的写超过一个登记的上限」，`research/prompts/m3-b44-code-r2-forks.md:16`），而 W13 把同样的无上界带进了**真做**：真做续做自己读的块、自己写的块都记下，不止预演。这在没有空间不够、预演第二趟根本不走的挂载里也成立。

量的（探针 `probe_bytes_read_to_the_disk.rs`，草稿副本上给 `DeviceReadOnceInThisMount` 加了个 `Drop` 打印，补丁 `patches/drop-print.patch`；窄盘、删除批大小 1、不带故障；原样输出 `out/rerun.stdout`）：

| 被删文件的数据单元数 | 数据字节 | 挂载末尾共用那一份占（两盘合计） | 读到盘上的字节合计 | 32 KiB 读的不同（盘, 偏移）数 | 第一个写之后读到盘上的读数 |
|---|---|---|---|---|---|
| 4 | 131072 | 2863104 | 2252800 | 15 | 29 |
| 12 | 393216 | 6402048 | 3104768 | 31 | 45 |
| 20 | 655360 | 9359360 | 4218880 | 47 | 1 |

读：每多一个被删数据单元，32 KiB 读的不同（盘, 偏移）多 2（斜率 (47−15)/16 = 2）＝两份各读一次、共 64 KiB，释放之前读盘核每份都读（`crates/singlefs-core/src/transaction.rs:6706`）；共用那一份每单元多约 406016 字节（(9359360−2863104)/16），其中包括写成的块（删除批大小 1 时每步约 0.41 MB，与 J3 行「窄盘每步约 0.41 MB」同量级，`research/prompts/m3-b44-code-r2-main-verification.md:24`）。4、12 单元时第一个写之后还有读到盘上的读（29、45 次），说明真做续做自己的读也在往里记；16、20 单元时第一个写之后只剩 1 次读到盘上。后者的原因是推的：这张窄盘上 16 个单元的续做中途要推抬 F、取号之前的第二趟因此走了（用例文件头这么写：「续做中途推抬 F，取号之前那一趟因此在只进内存的盘上走」，`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:7`），预演把真做要读的块先记下了；4、12 单元我没单独查走没走第二趟。归因「每单元多读的两份是释放之前读盘核」也是读码推的（斜率 2 与这一项对得上，没有单独关掉那一项再量）。

推的：默认批大小是 144 个单元（「第一版 unlink：只立 deleted_inodes，从尾部按 144 单元分批，挂载里续做到空」，`.claude/kb/decisions/08-核心索引结构.md:26`），批大时写的部分被摊薄，读的部分不变：读是每个被释放的数据单元两份。线性外推到 1 TiB 的文件是 2 TiB 的读被记下（2^25 个 32 KiB 单元 × 64 KiB）。外推没有量过。斜率我先后量过 4、8、12、16、20 个单元五点，32 KiB 读的不同（盘, 偏移）数依次是 15、23、31、39、47（每多 4 个单元多 8，即每单元 2），`out/rerun.stdout` 里留的是 4、12、20 三点。

**判定：规则没说；按「会不会撑爆一次合法挂载」这条判据，答案是会（推的）。** 一次合法挂载 = 崩在 unlink 一个大文件之后的重挂，续做要把队列走到空；按上面的斜率，被删数据量超过可用内存的一半时这次挂载记下的字节就超过内存，而 W13 之前这些读是一步用完即丢。条款层面岔路二选了「先量再定」，没有给上界，所以这一格是**条款没写**，该交用户：上界定在预演的写、还是连真做的读一起（真做续做自己读的块不记也不影响预演，但要另开一路）。
推翻：同一探针在批大小 144 的开关下，共用那一份随被删数据量增长的斜率明显小于 2×（例如读不记在真做路上）；或真做路上读过的块被释放（`blocks_read_or_written_in_this_mount` 里只有一处 `remove`，在写报错的块：「recorded.remove(&block_index);」（`crates/singlefs-core/src/mounted_session.rs:781`），读过的块没有释放的路）。

## L2 预演读错在取号之前拒（W13 的新拒因、W16 重造的两条用例）

### L2.1 条款句对代码行

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | 候选「没走完就拒」的定义：「可写挂载在取号之前拒，交一个新的类型化错误成员，盘一个字节不变」（`research/prompts/m3-b44-code-r2-forks.md:7`；注意它是没选的候选，选的是第 9 行那条） | 拒因造在 `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition`（`crates/singlefs-core/src/transaction.rs:4101`），由 `refuse_before_acquisition_a_continuation_…` 在 `crates/singlefs-core/src/mount.rs:5369` 造、`crates/singlefs-core/src/mount.rs:5374` 起装进 `MountError::Publish`，三份写账全空（`crates/singlefs-core/src/mount.rs:5376`） | **一致**（读码）。成员、写账形状对得上定义的字面 |
| 2 | 判决 J2 行：条款没写「预演自己被别的错打断、没走到会不会撞 C841 那一步」（`research/prompts/m3-b44-code-r2-main-verification.md:23`） | ⓑ 的补法：`crates/singlefs-core/src/mount.rs:5362` 起 | **规则没说**（补的一句，见 L1.4） |
| 3 | 「拒了之后两块盘逐字节不变」 | 拒在 `crates/singlefs-core/src/mount.rs:6440` 的调用里 `?` 返回，之后才有逐盘核（`crates/singlefs-core/src/mount.rs:6465`）与取号（`crates/singlefs-core/src/mount.rs:6499`）；这之前的所有写：预演在分配器拷贝上（dry run）与 `DeviceKeepingWritesInMemory`（对盘只读借，类型上调不到改盘的三个动作，`crates/singlefs-core/src/mounted_session.rs:362`） | **一致**。旁证：`the_same_read_fault_window_on_the_unchanged_image_is_refused_before_acquisition_instead_of_being_eaten_by_the_rehearsal` 断言两块盘逐字节不变，复跑绿（§0.1） |
| 4 | 「再挂一次不带故障照常走完」 | 交回 `Err` 时 `devices_read_once_in_this_mount` 与记下那张表一起丢（局部变量，`crates/singlefs-core/src/mount.rs:6298`），没有跨挂载的状态；分配器每次挂载由盘重建 | **一致**。旁证：同一条用例最后一段 `remount…expect("不带故障再挂一次：没改坏的那一份挂载做成")`，续做 16 步走到空，绿 |

推翻 3：拒绝之后任一块盘与挂载之前有一个字节不同。推翻 4：不带故障再挂一次仍拒或没走完。

### L2.2 ⓑ 偏拒：会不会把只是被另一份兜住、真做本来走得通的挂载拒掉（读码）

拒的触发 = 〔只进内存那一趟结局是 `StepRefused`（不是 C841 或 ReadFailed）或 `StoppedBeforeTheEndOfTheContinuation`〕且〔读错账非空〕（`crates/singlefs-core/src/mount.rs:5362`；账空就 `return Ok(())`，`crates/singlefs-core/src/mount.rs:5365`）。几条读码的结论：

1. 读错账在续做预演开头清空（`crates/singlefs-core/src/mount.rs:5331`），第一趟（分配器拷贝上，文档说不读盘）跑时账是空的，所以第一趟的非空间错不会被 ⓑ 拒（它走 `return Ok(())`）。ⓑ 只可能在第二趟（只进内存的盘）走过之后触发，而第二趟只在第一趟报空间不够时才走（`crates/singlefs-core/src/mount.rs:5342`）。
2. 没走到结局的原因只有两类：镜像固有的错（同一张共用表，真做走到同一步也会报，只是落盘之后）和读错本身造成的错。前一类 ⓑ 拒早了是好事（真做也会失败，少写几步）；后一类就是候选「没走完就拒」要拒的（真做再读一次盘，读错那一块没记下，可能走出别的结局）。我没有找到第三类——预演没走完、真做能走完、又不是读错造成的——所以「被另一份兜住的读错 + 与它无关的没走完」只会换一个拒因，不会拒掉真做能做成的挂载（读码）。
3. 副作用（文字而非结局）：这类挂载拒的是最先那次读错（盘 0 那一份），不是镜像固有的那个错；用户看到的拒因会变成 `ReadFailedWhileRehearsing…`（`crates/singlefs-core/src/mount.rs:5369`）。

拒的比例（给 Opus 腿的量法）我没量：条款的翻面观测「量出不带坏镜像、只带短暂读错的挂载被这一拒拦下的比例高到不可接受」（`research/prompts/m3-b44-code-r2-forks.md:7`）是没选候选的观测，对 ⓑ 的补法同样适用。
**判定：规则没说，代码在补的这一句范围内自洽。** 推翻 2：造出一段历史，预演没走完的原因既不是读错、也不是真做会撞的错，读错账又非空，而真做走得通。

### L2.3 反例：读成功而字节坏，那一拒不触发，真做落盘之后才交同一个错（量的）

ⓑ 数的是「底下那块盘的 `read_at` 报 `Err`」：读错账只在 `Err` 分支里记（`crates/singlefs-core/src/mounted_session.rs:667`）。读成功而字节坏（`InjectedFault::ReadReturnsCorruptedBytes`，`crates/singlefs-harness/src/fault_injection.rs:147` 本来就是这个项目的读故障一类）不入账，**而且**那一块被记下（「recorded.insert(block_index, block.to_vec());」，`crates/singlefs-core/src/mounted_session.rs:680`），真做再读取的是同一份坏字节。W16 的 J3 用例（`the_same_read_fault_window_on_the_unchanged_image_…`）只试了块设备报错。

探针 `probe_corrupt_bytes_window.rs`（拷自 W13/W16 的用例文件，同一个窗口：只进内存那一趟头一次读的一份树表的盘 0、盘 1 两份，序数 358、359；只把「报错」换成「读成功、第一个字节取反」）。原样输出（`out/rerun.stdout`）：

```
PROBE2 window={358, 359} ...
PROBE2 outcome=Err FloorRaiseFailedAfterTheMountsPublishes(FloorRaiseFailedAfterTheMountsPublishes { cause: RollbackFloorCeilingNeedsUnreadableValidRootTreeTable ...
PROBE2 disk_changed=true writes_of_the_mount=420
PROBE2 remount_without_fault_ok=true
PROBE2 io_error_window_outcome=Err Publish(PublishSequenceFailed { cause: ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(ReadFailedOnTheDisk { device: DeviceIdentity(0), offset: DeviceOffsetInBytes(25067520), length_in_b disk_changed=false
```

两行并排：

| 同一个窗口 | 结局 | 盘 |
|---|---|---|
| 报块设备错（W16 用例的形） | 取号之前拒：`MountError::Publish(… ReadFailedWhileRehearsingTheContinuationBeforeAcquisition …)` | 不变 |
| 读成功、字节坏 | 取号、写行、暖机、推抬 F 都落了盘（共 420 个写）之后交 `FloorRaiseFailedAfterTheMountsPublishes(… RollbackFloorCeilingNeedsUnreadableValidRootTreeTable …)` | 变了 |

不带故障再挂一次做成（`remount_without_fault_ok=true`）。W13 之前的树我没复原，「W13 之前这一形会做成」是推的：W13 之前真做的读是独立的第二次采样。

读：预演里读到的两份坏字节被记下，抬 F 算上限读不出，只进内存那一趟停在续做之前，账空，ⓑ 放行（「这两趟读盘没出过读错：照旧留给续做」，`crates/singlefs-core/src/mount.rs:5365`）；真做取号之后抬 F 读同一份记下的坏字节，同一个错落盘之后才交。拒因的文档把前提写成「读错的那一块没记进共用的那一份，真做再读可能走出别的结局」（`crates/singlefs-core/src/mount.rs:6439`）——对报错的读成立，对读成功而字节坏的读不成立。
**判定：规则没说。** 用户定的第 9 行只定了共用；ⓑ 的补法把「读错」当成块设备报错。该交主 agent / 用户的两条出路（我不替采纳）：①读错账也记「读成功但读的一方判为自证不过」的读（`note_that_the_bytes_read_self_verify` 只报自证过的那一半，`crates/singlefs-core/src/recovery.rs:150`，要补对称的一半）；②改用「没走完就拒」那一行（`research/prompts/m3-b44-code-r2-forks.md:7`）：没走完一律拒，翻面观测是被拒比例。
推翻：同一个窗口换成字节坏之后挂载在取号之前拒、或做成。

### L2.4 冻结副本里每一处匹配 `MountError::Publish` 或 `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition` 的调用方

命令（原样）：`grep -rn "MountError::Publish\|ReadFailedWhileRehearsingTheContinuationBeforeAcquisition" crates litmus --include=*.rs`，在冻结副本根目录下跑，56 行命中；下表去掉纯文档注释（另列）。**这一拒的写账全空、取号还没发**：凡是把 `Publish` 当「取号之后」的假设在这一拒上不成立，下面逐处看它成不成立。

| 调用方（文件:行） | 认的是什么 | 在这一拒上 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs:5374` | 造：`Err(MountError::Publish(Box::new(PublishSequenceFailed {` ，三份写账全空 | 就是这一拒本身 |
| `crates/singlefs-core/src/mount.rs:5569` | 造：取号之后一次发布失败（`publish_failed_after_acquisition`） | 走不到这一拒（取号之后） |
| `crates/singlefs-core/src/mount.rs:6703` | 造：续做里一步没发成、不是空间不够 | 走不到 |
| `crates/singlefs-core/src/mount.rs:7162` | 读：`if let MountError::Publish(failed) = error_of_the_continuation` 把只进内存那一趟续做的 `cause` 当 `StepRefused` | 成立：这一拒由外层造，里层续做不造它（`crates/singlefs-core/src/mount.rs:5399` 另把它归「取号之前拒」那一类，走到也不会二次计） |
| `crates/singlefs-core/src/mount.rs:4254` | 读：`floor_raise_refused_for_space` 把 `Publish(_)` 归「不是空间不够」 | 成立：抬 F 路径看不到这一拒（它在 `establish_instance` 里先返回） |
| `crates/singlefs-core/src/mounted_session.rs:1364` | 读：`refusal_is_short_of_space` 对新成员判 `false` | 成立：新成员不是空间不够，不触发推抬 F |
| `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:798` | 读：`MountError::Publish(failed)` 当「取号之后那一串里有一次没做成」，先调 `counts_when_the_acquisition_barrier_arrived`（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:728`） | **假设不成立、降级安全**：这一拒没有发过写，那个函数交回「在可写挂载里一道屏障都没收到：取号那道屏障没发」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:739`）这个停下的原因，不会误判成通过。该二进制挂的池没有删除队列、故障计划是 `unarmed`（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1447`），走不到这一拒 |
| `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1241` | 读：抬 F 路径把 `Publish(_)` 列进「第一次空发布之前就停下」的无账一类 | 成立 |
| `crates/singlefs-checker-tier/src/crash_injection.rs:3204` | 读：`publishes_before_the_continuation_of_a_mount_that_failed`，用 `writes_of_persisted_publishes`、`writes_of_failed_publishes` 划续做段 | 成立：两份账都空，辅助函数交回 `None`（`crates/singlefs-checker-tier/src/crash_injection.rs:3271`），即「没有续做段」；崩溃轻型 8 项里带读故障那一项（`read-faulted-states-never-hold-the-witness-criterion`）上次 358 秒绿（`records/2026-10-13-代码欠账批.md:207`） |
| `crates/singlefs-harness/src/history.rs:3012` | 读：把错成员写成串 `MountError::Publish(<cause>)` | 成立：`crates/singlefs-harness/src/history.rs:2915` 给新成员单独一臂，串里带 `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(<块设备错>)` |
| `crates/singlefs-harness/tests/fault_injection_fast_tier.rs:979` | 读：`member.starts_with("MountError::Publish(")` 认注在可写挂载单元写上的那一格 | 成立（它认的是写故障那一格；新成员的串同前缀，不会改变这条断言） |
| `crates/singlefs-harness/src/mount_transparency.rs:2320` | 读：把 `Publish(_)` 归「不在 D23 拒因单子上」 | 成立：这一拒只在读错时出，挂载透明对「注过读故障的态不判」（`.claude/kb/decisions/23-journal的角色与格式.md:394`） |
| `crates/singlefs-harness/src/model_comparison.rs:735` | 读：把新成员映射成 `ObservedRefusalReason::Unexplained`，注释「健康的内存盘上不出读错」 | 成立：读错账只在盘报 `Err` 时记；越界在 `check_request` 里先挡掉，不进账（「self.check_request(offset, length)?;」，`crates/singlefs-core/src/mounted_session.rs:714`） |
| `crates/singlefs-harness/src/model_comparison.rs:819`、`crates/singlefs-harness/src/model_comparison.rs:934`、`crates/singlefs-harness/src/model_comparison.rs:1049` | 读：`Publish(failed)` 取 `failed.cause` 的理由（819）；两处「点名根环槽」「报的上限」的提取器把 `Publish(_)` 归无（934、1049） | 成立：新成员走 735 那一臂；提取器对它交空 |
| `crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:546` | 读：C841 用例，`failed.cause` 是容器号到顶，三份账空 | 成立（它认原因，不认「取号之后」）；这一拒只在读错账非空时抢在 C841 前，这些用例不注读故障 |
| `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:510` | 读：续做那一步的块设备错，`cause` 是 `BlockDevice(_)`，核写账合计 | 成立：写错在真做（预演的写在内存里），不抢先 |
| `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs:151` | 读：取号之后零单元发布轮换屏障报错，`cause` 是 `BlockDevice(_)` | 成立：屏障错不在预演里 |
| `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:1068` | 读：续做第二步写报错，`cause` 是 `BlockDevice(_)` | 成立 |
| `crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:460` | 读：W13/W16 自己的用例，要 `Publish` 且写账全空，`cause` 取出来判 | 成立（就是这一拒的用例；`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:511` 认 ReadFailed 成员） |

纯文档注释命中（不判）：`crates/singlefs-core/src/mount.rs:127`、`crates/singlefs-core/src/mount.rs:5307`、`crates/singlefs-core/src/mount.rs:5308`、`crates/singlefs-core/src/mount.rs:5386`、`crates/singlefs-core/src/mount.rs:6633`、`crates/singlefs-core/src/mount.rs:6849`、`crates/singlefs-core/src/mount.rs:7108`、`crates/singlefs-core/src/transaction.rs:4096`、`crates/singlefs-checker-tier/src/crash_injection.rs:2884`、`crates/singlefs-checker-tier/src/crash_injection.rs:6573`、`crates/singlefs-checker-tier/src/crash_injection.rs:6672`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2772`、`crates/singlefs-harness/tests/common_images_for_the_checker_tier/admission_raises_the_floor_before_refusing.rs:168`、`crates/singlefs-harness/tests/common_images_for_the_checker_tier/admission_raises_the_floor_before_refusing.rs:244`、`crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:562`、`crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:664`、`crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs:836`、`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:1003`、`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:3`、`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:4`、`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:457`、`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs:565`。

**判定：成立**，唯一的假设落空是 `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:798`（把 `Publish` 当取号之后），降级成「停下的原因」，且那个二进制走不到这一拒。第二轮判决 J4 行当时查的 8 处按 `MountError::Publish` 分路的调用方（`research/prompts/m3-b44-code-r2-main-verification.md:25`）这一轮新增了 `crates/singlefs-harness/src/history.rs:2915` 一臂与 `crates/singlefs-harness/src/model_comparison.rs:735` 一臂，都对。
推翻：任一调用方在这一拒上把「写账空」当成别的结局处理（例如把空账当「取号之后一次发布都没落盘的真失败」去核设备一层的写数）；`new_pool_file_creation_on_device.rs` 的故障计划不再是 `unarmed` 而池里有删除队列。

## L3 读坏当场重读穿到盘上、换掉记下的那一份（W16）

### L3.1 条款原文（现查）与主 agent 判定

- D19：「释放之前按位置项读盘核校验和，读盘本身失败或核出对不上，都先重读一次」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）；「只重读一次，不退避、不多次重试」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:115`）。
- D16：「这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退」（`.claude/kb/decisions/16-发布语义.md:37`，管「这个进程知道住着根的槽」）；「进程内记根时，根环表里这一项成了最小的那个，先重读那一槽」（`.claude/kb/decisions/16-发布语义.md:34`，E2）。
- 调度记录 W15 交回，主 agent 判定 ②：「那一读要绕过记下的那一份、直接读盘，读到自证过的就换掉记下的那一份」（`records/2026-10-13-代码欠账批.md:199`）。

### L3.1b 条款句对代码行（对照表）

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | 「释放之前按位置项读盘核校验和，读盘本身失败或核出对不上，都先重读一次」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`） | S6：第一次读走 `reader`，重读走 `rereading_reader`（`crates/singlefs-core/src/transaction.rs:3767`），读到对得上才报自证（`crates/singlefs-core/src/transaction.rs:3821`） | **一致** |
| 2 | 「只重读一次，不退避、不多次重试」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:115`） | 六处各一次重读（没有循环重试）；S5 重读那一遍整遍根环与实例表读一次（`crates/singlefs-core/src/recovery.rs:2035`） | **一致**（读码）。注意 S5 的「一次」是整遍根环读，不是单槽 |
| 3 | 「这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退」（`.claude/kb/decisions/16-发布语义.md:37`） | S2、S4：重读仍坏交 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread` 或 `RootRingSlotKnownToHoldARootStillBadAfterOneReread`（`crates/singlefs-core/src/mount.rs:3556` 起、`crates/singlefs-core/src/recovery.rs:1637` 起） | **一致** |
| 4 | 「进程内记根时，根环表里这一项成了最小的那个，先重读那一槽」（`.claude/kb/decisions/16-发布语义.md:34`） | U2：重读的是判号那一读记下的字节（L3.3 问 3） | **字面不一致**，方向安全，规则没说重读的时刻 |
| 5 | 判定 ②：「那一读要绕过记下的那一份、直接读盘，读到自证过的就换掉记下的那一份」（`records/2026-10-13-代码欠账批.md:199`） | S1–S6 全部：重读走 `reread_at_after_a_bad_read`（`crates/singlefs-core/src/mounted_session.rs:829`），自证后 `keep_the_self_verified_reread`（`crates/singlefs-core/src/mounted_session.rs:841`） | **一致** |

### L3.2 每一处「读坏当场重读」（文件:行）、走的新方法、换不换

链：`PoolReaderOfOneReadPass::rereading_after_a_bad_read`（`crates/singlefs-core/src/recovery.rs:186`）→ `PoolReader::reread_after_a_bad_read`（默认 `crates/singlefs-core/src/recovery.rs:130`；一遍读的读者 `crates/singlefs-core/src/recovery.rs:228`；盘切片 `crates/singlefs-core/src/recovery.rs:291`；`Vec` `crates/singlefs-core/src/recovery.rs:340`）→ `BlockDevice::reread_at_after_a_bad_read`（默认就是 `read_at`：`crates/singlefs-core/src/block_device.rs:162`；共用那一层 `crates/singlefs-core/src/mounted_session.rs:829` 走 `RereadStraightToTheDiskKeepingWhatSelfVerifies`，穿到盘上读并把读错记进读错账：`crates/singlefs-core/src/mounted_session.rs:686`；只进内存的盘 `crates/singlefs-core/src/mounted_session.rs:497`，写过的块取内存里那份）。换：读的一方核过自证就报 `note_that_the_bytes_read_self_verify`（`crates/singlefs-core/src/recovery.rs:150`）→ 重读那一遍转成 `keep_the_self_verified_reread`（`crates/singlefs-core/src/recovery.rs:245`）→ `BlockDevice::keep_the_self_verified_reread`（共用那一层 `crates/singlefs-core/src/mounted_session.rs:841` 把字节记进表；只进内存的盘 `crates/singlefs-core/src/mounted_session.rs:510` 只转交没写过的块，`crates/singlefs-core/src/mounted_session.rs:516`）。报自证的地方：根槽 `read_root_ring_slot`（`crates/singlefs-core/src/recovery.rs:1455`）、整单元 `read_unit_via_locations`（CRC-32C 等于位置项里的才报，`crates/singlefs-core/src/recovery.rs:573`）、释放核每份（`crates/singlefs-core/src/transaction.rs:3821`）。读盘三段（路是 `StraightToTheDisk`）里重读本来就穿盘，换的那一步是空操作（`crates/singlefs-core/src/mounted_session.rs:843` 的 `StraightToTheDisk` 臂）。

| 处 | 读什么 | 新方法 | 换掉记下的那一份 |
|---|---|---|---|
| S1 `crates/singlefs-core/src/mount.rs:1832` 起，用在 `crates/singlefs-core/src/mount.rs:1834`、`crates/singlefs-core/src/mount.rs:1836`（`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`，定义 `crates/singlefs-core/src/mount.rs:1816`） | 被抛弃根或较新可读根的账（分配记录树）读不出之后的重读 | `rereading_after_a_bad_read(devices)` | 换：账节点经 `read_unit_via_locations` 自证 |
| S2 `crates/singlefs-core/src/mount.rs:3520` 起，用在 `crates/singlefs-core/src/mount.rs:3544`、`crates/singlefs-core/src/mount.rs:3556`（抬 F 算上限 `roots_of_the_ring_for_the_ceiling`） | 根环槽：重建分配器两遍里只有一遍自证过的槽、这个进程知道住着根的槽 | `rereading_after_a_bad_read(reader)` | 换：`read_root_ring_slot` 自证 |
| S3 `crates/singlefs-core/src/mount.rs:4412` 起，用在 `crates/singlefs-core/src/mount.rs:4421`（抬 F 重算影子账 `readable_roots_for_raising_the_floor`） | 两遍里只有一遍自证过的槽 | 同上 | 换 |
| S4 `crates/singlefs-core/src/recovery.rs:1614` 起，用在 `crates/singlefs-core/src/recovery.rs:1637`（`readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once`，抬 F 重算影子账、管理员回退用） | 知道住着根的槽 | 同上 | 换 |
| S5 `crates/singlefs-core/src/recovery.rs:2035`（`effective_rollback_floor_rereading_the_newest_instance_table_once_over`，定义 `crates/singlefs-core/src/recovery.rs:1990`；三个调用方的读根环闭包 `crates/singlefs-core/src/recovery.rs:1848`、`crates/singlefs-core/src/recovery.rs:1893`、`crates/singlefs-core/src/recovery.rs:1937` 收「哪一遍」；实例表链 `crates/singlefs-core/src/recovery.rs:2007`） | 算 F 生效值：根环与最新那条根的实例表，重读那一遍两样都按重读读 | `in_the_pass(reader, RereadAfterABadRead)` | 换（根槽、实例表单元都自证）。每次发布末尾的系统配置轮换（续做在内）都经这一处（`crates/singlefs-core/src/transaction.rs:539`） |
| S6 `crates/singlefs-core/src/transaction.rs:3754` 起，重读用在 `crates/singlefs-core/src/transaction.rs:3767`（释放之前读盘核每份 `copies_of_a_released_unit_failing_the_release_checksum_check`） | 要释放的单元每份，D19 硬规则 1 的重读 | `rereading_after_a_bad_read(reader)` | 换，仅「对得上」（`CopyCheck::Intact`）时报 |
| U1 `crates/singlefs-core/src/recovery.rs:1683`（`readable_roots_with_ring_slots_rereading_unreadable_slots_once`），W16 没改 | 写系统配置槽之前算 F 生效值时，根槽「读不出」才立刻再读一次 | 不用：第二次读走同一个读者（`crates/singlefs-core/src/recovery.rs:1691` 的 `read_the_slot`） | 不用换，理由见 L3.3 U1 |
| U2 `crates/singlefs-core/src/mount.rs:1374`（`reread_of_a_root_ring_slot_bad_at_mount`，E2），W16 没改；调用方 `crates/singlefs-core/src/mount.rs:1415` 的包装，用在 `crates/singlefs-core/src/mount.rs:5107`、`crates/singlefs-core/src/mount.rs:6964`、`crates/singlefs-core/src/transaction.rs:1945` | 进程内记一条根时，重读挂载那一刻读坏的槽 | 不用 | 不换，理由见 L3.3 U2 |

我用名字 grep 过：`grep -rn "fn reread_at_after_a_bad_read" . --include=*.rs` 在冻结副本 `crates/` 下只有三处命中（`crates/singlefs-core/src/block_device.rs:162`、`crates/singlefs-core/src/mounted_session.rs:497`、`crates/singlefs-core/src/mounted_session.rs:829`）；grep 的是 `reread`、`once`、`twice`、`at_most_` 打头或结尾的函数名（`crates/singlefs-core/src/` 的 mount、recovery、transaction、allocator 四份），列出的 8 处就是共用段里的全部命中；`instance_table_of_the_newest_root_read_at_most_twice`（`crates/singlefs-core/src/mount.rs:1468`）只有 `crates/singlefs-core/src/mount.rs:2937` 一处调用，在重建分配器那一段（读阶段，读原盘，不经共用那一层）。名字 grep 的覆盖范围是我这里的边界，没有按「读一次、不对再读一次」的语义逐调用点扫。

### L3.3 逐问

**问 1：重读之后换掉的那一份，会不会让预演与真做看到不同字节？** 不会（读码），三个例外都落在条款之外：

1. 预演第二趟的重读换掉的是共用那张表（`crates/singlefs-core/src/mounted_session.rs:841`），真做第一次读这一块取到的就是换进去的；只进内存那一趟自己写过的块不转交（`crates/singlefs-core/src/mounted_session.rs:516`），所以不会把预演在内存里写的字节漏进真做的表。
2. 读盘三段里换是空操作（`crates/singlefs-core/src/mounted_session.rs:843`），它们的重读不动表。
3. 说谎设备（吞写）上，被吞的写记成了写下的字节（L1.4 ⓒ），之后的重读若落在这些块上读到盘上的旧字节，自证过就换掉写下的字节；这一形走得到的前提是读这个块第一次就读坏，而写下的字节是程序自己写的、自证得过，所以正常走不到（推的，没造）。

推翻：造出预演里重读换了、真做第一次读同一块却读到换之前的字节。

**问 2：坏字节第一次被记下、重读又失败时，后面那几读看到什么？** 记下的仍是第一次的坏字节（只有自证过的重读才换，`crates/singlefs-core/src/recovery.rs:1455`、`crates/singlefs-core/src/recovery.rs:573`、`crates/singlefs-core/src/transaction.rs:3821`；失败的重读读错入账、读到自证不过的不换）。之后：
- 每个有重读这一步的处（S1–S6）再读到它时，先取记下的坏字节、判坏、再穿到盘上重读一次（又一次真读盘，可能换掉）；
- 没有重读这一步的读者一直看到坏字节：`next_instance_generation_to_acquire` 的判号（序 b，但真做取号读半是读盘，不受影响）、U1 对「读得出而自证不过」的槽（按设计不重读，`crates/singlefs-core/src/recovery.rs:1680` 的文档）、续做推抬 F 算上限读有效根的树表（没有重读这一步）；
- 量的就是最后一类：L2.3 的两份坏字节在整个挂载里一直是坏的，真做落盘之后交同一个错。
条款只写了「根槽这一次读坏」「释放之前读盘核」两处要重读（D16、D19 的原文见 L3.1），对「读有效根的树表」没有一句：**规则没说**，反例就是 L2.3。

**问 3：W16 没改的两处，逐处为什么。**

- U1（`crates/singlefs-core/src/recovery.rs:1683`）：**不用改**（读码）。它只在第一次读「读不出」（设备报错、越界）时再读一次；读不出的块不入表（`crates/singlefs-core/src/mounted_session.rs:661` 报错就返回，不到 `recorded.insert`），第二次读照样穿到盘上，读得出就按共用路径记下。与改了的区别只是「重读读回自证不过的字节记不记」，而这一处按设计不重读自证不过的槽。边界：一次读跨几个块、其中一块已记下时，只有没记下的那一段穿盘（「unrecorded_run_start.get_or_insert(block_index);」，`crates/singlefs-core/src/mounted_session.rs:734`，分段读），读不出的那一块一定是没记下的，所以仍穿盘。推翻：读不出的一块（或它所在的整段）第二次读命中记下的表。
- U2（E2，`crates/singlefs-core/src/mount.rs:1374`）：**字面对不上，方向安全，该记一笔**（读码）。W16 报告说「这次挂载头一次读它照旧穿到盘上（读阶段读的不进共用的那一份），之后进程内重读取记下的那一份」；我查到的是更早一步：`next_instance_generation_to_acquire` 在判号时（序 b，`crates/singlefs-core/src/mount.rs:6316`）就用 `highest_root_instance` 把根环**每个槽**读了一遍（`crates/singlefs-core/src/recovery.rs:3591`），读成功的块（含读回全零的、字节坏的）当场记下。W15 钉的读次数同一句话：「写行、暖机、抬 F 与续做再读根环都取共用的那一份、不再读盘」（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs:48`），读根环每槽到盘上 8 次、之后不再增。所以 E2 在记根时的「重读」（D16 第 34 行「先重读那一槽」）读的是判号那一读记下的字节，不是此刻读盘：判号那一读读到好字节就照好字节算（等于这一读当了重读）；读到全零或字节坏就一直钉在可能住过的最旧 txg（回收少，不是越过盘上有效根，安全方向；代价是空间）。条款没写「重读与读坏之间隔多久、要不要是记根那一刻」：**规则没说**，W16 报告把它交主 agent，我的答案是：它不是「读坏当场」那一类，但今天的行为与 D16 第 34 行的字面（记根时再读一次盘）不同。推翻：造出判号时读到字节坏、此后读好的槽，进程内记根后 E2 得到 `ValidRoot`（说明重读真读了盘）；我没造。

**问 4：带读故障注入的设备上，默认实现走不走故障注入那一层？** 走（读码 + 复跑）。`FaultInjectingBlockDevice` 只实现 `read_at`（`crates/singlefs-harness/src/fault_injection.rs:979` 起的 `impl`），不覆写 `reread_at_after_a_bad_read`，默认实现就是 `self.read_at(offset, buffer)`（`crates/singlefs-core/src/block_device.rs:167`），所以重读是故障计划里多一次读调用，与 W13 之前的重读（本来就是 `read_at`）同数。共用那一层在注入层**之上**（包的是注入层）：重读穿到的是注入层。共用层之上没有别的包装（全仓 `impl BlockDevice for` 的包装都包在原盘下面）。旁证：`a_known_root_slot_read_back_garbage_into_the_shared_reads_is_reread_from_the_disk_by_the_floor_raise_and_the_mount_is_made` 用 `DeviceRecordingReads` 记账断言「那一槽第 10 次读到盘上的是抬 F 的重读」，复跑绿（§0.1）。
推翻：一个包在共用层之上的 `BlockDevice` 包装把 `read_at` 转给里层、而没覆写重读——那样重读会绕过它的注入；全仓今天没有。

**判定一览（L3）**：六处改到的：一致；两处没改的：U1 一致、U2 字面不一致（安全方向，规则没说）；条款外的 L2.3 一类（读有效根的树表）规则没说。

## L4 J1：journal 记录新根段指针在重新编码之前按 I-2.5 判（W13，W15 改了一条用例的造法）

### L4.1 条款句对代码行

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | I-2.5：「任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序」；「非全零的指针不享这条豁免」（`crates/singlefs-checker/invariants/04-校验和.md:10`） | 谓词：`NodePointer::location_entries_ascend_by_device`（`crates/singlefs-core/src/pointer.rs:313`），全零整条豁免（同函数里 `*self == Self::empty_root()`），其余按两条位置条目严格升序（`crates/singlefs-core/src/pointer.rs:84`） | **一致**（读码）：豁免范围就是条款写的「整条 86 字节全零」，非全零但位置条目全零的指针不豁免 |
| 2 | 判决处置第 1 条：「按 I-2.5 判那条记录新根段里的每条指针，坏的交类型化错误、两块盘不变」（`research/prompts/m3-b44-code-r2-main-verification.md:32`） | `new_root_segment_pointers_of_the_record_ascend_by_device`（`crates/singlefs-core/src/recovery.rs:5327`）逐条判，坏的报 `RecoveryFailure::InvariantViolated { invariant: "I-2.5" … }`（`crates/singlefs-core/src/recovery.rs:5300`）；在 `record.to_bytes()` 之前调（`crates/singlefs-core/src/recovery.rs:2752`，`to_bytes` 在 `crates/singlefs-core/src/recovery.rs:2753`） | **一致**：判在重新编码之前，错是类型化成员 |
| 3 | 判决 J1 行的反例：写者断言在重新编码时 panic（`research/prompts/m3-b44-code-r2-main-verification.md:22`，`pointer.rs` 第 325 行） | 写者断言今天在 `crates/singlefs-core/src/pointer.rs:326`，谓词与 J1 这一判是**同一个**（`self.location_entries_ascend_by_device().is_ok()`） | **一致**：J1 判过的记录，写者断言必过 |

### L4.2 逐问

**新根段里的指针全没全，还有哪类没进这一判？** `JournalRecord` 里 `NodePointer` 类型的字段恰四个：`new_tree_table`（`crates/singlefs-core/src/journal.rs:160`）、`new_mapping_root`（`crates/singlefs-core/src/journal.rs:161`）、`new_instance_table`（`crates/singlefs-core/src/journal.rs:166`）、`new_allocation_record_tree_root`（`crates/singlefs-core/src/journal.rs:167`），判的清单正是这四条（`crates/singlefs-core/src/recovery.rs:5327`）。没进这一判的一类：`named: Vec<NamedUnit>` 里每个点名项的 `locations: [LocationEntry; 2]`（`crates/singlefs-core/src/journal.rs:94`）——I-2.5 的条款射程里有「journal 点名项里的位置条目数组」，而这一函数不判它。不 panic：`NamedUnit::write_to` 只对写出的字节数断言（`crates/singlefs-core/src/journal.rs:113`），没有升序断言；全仓非测试代码里重新编码盘上读来的 journal 记录只有 `crates/singlefs-core/src/recovery.rs:2753` 这一处（另一处 `crates/singlefs-core/src/transaction.rs:1590` 是发布路径自己造的记录）。所以对 J1 要防的 panic 是全的；对 I-2.5 对点名项的那一半，核心这边没有判（池级 checker 一侧已实现，条款写「已实现」，`crates/singlefs-checker/invariants/04-校验和.md:10`），本批没动：**规则没说**这一判必须在核心恢复里也做。

**坏镜像在任何写之前交类型化错误、两块盘不变、不 panic？** 是。判在重建上一版里（`rebuild_previous_version` 在 `crates/singlefs-core/src/mount.rs:1251` 调它，可写挂载从 `crates/singlefs-core/src/mount.rs:7836` 进），那是取号之前的恢复段，没有写；回退的两处（`crates/singlefs-core/src/mount.rs:8432`、`crates/singlefs-core/src/mount.rs:8670`）调的是同一个函数。只读挂载是 `mount_read_only`（`crates/singlefs-core/src/mounted_read.rs:587`），不经 `rebuild_previous_version`。复跑：`journal_record_whose_new_root_segment_pointer_does_not_ascend_is_refused_by_the_writable_mount` 四条（树表、映射根、实例表、分配记录树根各一）绿（§0.1），断言可写挂载拒、只读挂载照旧挂上。W15 改的那一条（分配记录树根）造法：带文件的一版这一项本来是空根（86 字节全零），对调全零条目什么也改不了，W15 先换进一条不空的升序指针再对调（原因见 W15 报告「逐条定因」第 9 条，`~/.local/share/singlefs/work/m3-b44-w15/report.md`）；用例复跑绿，造法与断言的关系我读过用例文件，同意：造法是对的，断言没动。

**会不会误拒合法写路径写出的记录？** 不会：
- J1 的谓词与写者断言同一个（`crates/singlefs-core/src/pointer.rs:326`），发布路径造出来的记录 `to_bytes` 时已经过了这一判，没有哪条合法记录会被这一判拒；
- 空根：整条全零豁免（上表第 1 行），第一个文件版本的分配记录树根与暖机的映射根都是这一形；
- 全零位置条目但头不全零：写者自己的断言就拒，合法路径写不出；
- 单盘池：可写挂载在读任何一块盘之前要求不同设备身份数不少于下限（`crates/singlefs-core/src/mount.rs:6255`，下限 `crates/singlefs-core/src/mount.rs:787` 是 2），单盘池走不到这一判；只读挂载不重新编码记录。

**判定：一致。** 推翻：一条发布路径写出的记录在这一判上被拒（只能是写者断言与这一判的谓词不是同一个——今天是同一个函数）；或可写挂载在重新编码之前的路径上还有一处读盘上的指针写回而不经 I-2.5（我只查了 `JournalRecord::to_bytes` 的非测试调用点，其他重新编码盘上读来的指针的路径在 `pointers_of_the_rebuilt_version_ascend_by_device` 那一判里，`crates/singlefs-core/src/recovery.rs:3463`）。

## L5 比 fsid（W14）

### L5.1 条款句对代码行

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | 用户 2026-10-17 弹窗定：「单元头里的 fsid 与本池系统配置的 fsid 相同才算被更新单元盖掉」（`records/2026-10-13-代码欠账批.md:190`） | 比在 `instance_in_the_self_verified_unit_header_at`（`crates/singlefs-core/src/mount.rs:2214`）的末句：「(filesystem_identifier_in_the_header == filesystem_identifier_in_unit_headers)」（`crates/singlefs-core/src/mount.rs:2245`）`.then_some(instance)`；索引节点、数据单元、打包记录单元三类头都取头里的 fsid 一起比（`crates/singlefs-core/src/mount.rs:2227` 起的 `match`） | **一致** |
| 2 | C834 用户定案：「它两条位置条目指着的两个槽都读出自证过、实例代号比 R 大的单元头」（`research/prompts/c834-r3-main-verification.md:43`）；「不读系统配置里的号」（同行） | 实例代号仍取单元头里的（「instance > root.instance」，`crates/singlefs-core/src/mount.rs:2202`），两个槽都要过（「pointer.locations.iter().all(」，`crates/singlefs-core/src/mount.rs:2196`）；系统配置只出 fsid、不出实例代号 | **一致**：不读系统配置里的「号」，fsid 不是号 |
| 3 | 弹窗那句没写 fsid 取自哪一份系统配置 | `rebuilt_allocator` 传的是 `unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier)`（`crates/singlefs-core/src/mount.rs:3211`），低 8 字节（`crates/singlefs-core/src/unit.rs:44`，与单元头写入同一个函数） | **规则没说**取哪一份；取的是下一行那一份，见 L5.2 |

### L5.2 逐问

**本池 fsid 取自哪一份系统配置，那一份此刻是不是已经择定？** 取自可写挂载一开头择定的那一份：`let system_configuration = choose_system_configuration(&*devices)?;`（`crates/singlefs-core/src/mount.rs:7797`），之后才进 `rebuilt_allocator`（`crates/singlefs-core/src/mount.rs:7912`），所以此刻已经择定；不另读盘（`crates/singlefs-core/src/mount.rs:2045` 的文档）。几块盘的 fsid 不一致时择的那一步就拒（「交得出系统配置的几块盘之间 fsid 或设备数对不上」，`crates/singlefs-core/src/recovery.rs:1243`），所以这个 fsid 不随盘序变。

**合法写路径写出的单元头 fsid 是不是恒等于本池的（格式化、换盘、扩盘、回退之后）？**
- 写者：三类单元头都写 `unit_filesystem_identifier(filesystem_identifier)`（`crates/singlefs-core/src/unit.rs:99`、`crates/singlefs-core/src/unit.rs:208`、`crates/singlefs-core/src/unit.rs:283`），发布路径传 `pool.parameters.filesystem_identifier`（`crates/singlefs-core/src/transaction.rs:1579`、`crates/singlefs-core/src/transaction.rs:5964`），mkfs 传 `parameters.filesystem_identifier`（`crates/singlefs-core/src/make_filesystem.rs:509`、`crates/singlefs-core/src/make_filesystem.rs:520`）；可写挂载的 `parameters` 是按择到的系统配置建的、并核过调用方给的一致（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`）。
- 格式化：fsid 由调用方给（「而 fsid 是调用方给的（`MakeFilesystemParameters::filesystem_identifier`，mkfs 自己不生成）」，`crates/singlefs-core/src/make_filesystem.rs:531`）。**换一个 fsid 重新格式化**，旧单元留在单元区里、头里是旧 fsid：W14 之前按实例代号比会把它们当「更新实例盖掉」，现在不算——这就是要修的那一形。**同一个 fsid 重新格式化**，旧单元和本池不可分：按条款字面（「本池系统配置的 fsid」）它们算本池；这是 fsid 比对本来分不开的一形，W14 之前同样会算，不比以前坏。
- 换盘、扩盘：仓里没有这样的入口（`grep -rniE "fn (add|attach|replace|grow|expand)[a-z_]*(device|disk)" --include=*.rs singlefs-core/src singlefs-harness/src` 零命中）。
- 回退：`roll_back_by_a_forward_publish`（`crates/singlefs-core/src/mount.rs:8619`）是向前发布，用本池 `parameters` 写新单元、照抄的旧单元头本来就是本池的。
所以给不出本池单元头被误拒的合法历史；坏位改了头里的 fsid 会先被头校验和拒掉（那个槽不再「自证过」，本来就不算被盖掉）。

**别池的单元头放过、本池的误拒，各给不给得出可复现的镜像？**
- 别池的放过：给不出（64 位精确比；撞低 8 字节的概率是 2^-64）。同 fsid 重新格式化那一形放过的是同 fsid 的旧单元，我没造镜像，理由是它在条款字面上算本池。
- 本池的误拒：给不出（上一问）。
- 反方向的旁证（拒别池、放本池，各一条，复跑绿，§0.1）：`newer_root_whose_pointer_slots_hold_self_verified_units_of_another_pool_refuses_the_writable_mount_before_any_write_in_both_device_orders`、`newer_root_whose_pointer_slots_hold_the_same_planted_units_of_this_pool_is_passed_in_both_device_orders`。

**判定：一致。** 推翻：造出一段合法写路径的历史，其中某个本池写出的、自证过的单元头 fsid 的低 8 字节不等于择到的系统配置 fsid 的低 8 字节，而它又是 C834 那一判里「盖掉两份」的证据；或择系统配置的函数在 fsid 不一致时放行。

## L6 外部审核 ②⑥ 与 W13–W16 新写、改过的注释

### L6.1 ② `deleted_inodes` 根条目宽 `!= 8` 就拒（W13）

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | 审核 ②：「宽于 8 照收、截前 8 字节」是要改的现状，处置「② 改成 `!=`」（`records/2026-10-13-代码欠账批.md:167`） | 判在重建上一版读 `deleted_inodes` 根的地方：「entry.len() != DELETED_INODES_KEY_WIDTH_IN_BYTES」（`crates/singlefs-core/src/recovery.rs:2995`），报 `UnitMalformed`（不是 panic）；读回号的函数唯一调用点在这一判之后（`crates/singlefs-core/src/recovery.rs:3002`） | **一致** |
| 2 | D8：`deleted_inodes` 加 key = N（「8 字节、无 value、不带对象出生代」，`.claude/kb/decisions/08-核心索引结构.md:477`） | 写者只写 8 字节条目：「key.0.to_le_bytes().to_vec()」（`crates/singlefs-core/src/deleted_inodes.rs:57`，函数 `deleted_inodes_root_entries` 在 `crates/singlefs-core/src/deleted_inodes.rs:41`），条目宽常量 8（`crates/singlefs-core/src/deleted_inodes.rs:15`） | **一致**：合法写路径写出的根条目恒宽 8，不会被 `!= 8` 误拒 |

误拒的另一面：队列空时树表条目根指针置零、一个节点都不写（`crates/singlefs-core/src/deleted_inodes.rs:2` 的文档），读侧 `None => Vec::new()` 不经这一判；非空时 `level == 0` 且非空（同一个 `if`，`crates/singlefs-core/src/recovery.rs:2990` 起），这两条 W13 之前就在。旧判 `< 8` 对宽 0..7 的条目拒、对宽 > 8 的收下后截前 8 字节；新判两头都拒。复跑：`a_deleted_inodes_root_whose_entry_is_sixteen_bytes_wide_is_refused_by_the_writable_mount_before_any_write` 绿（§0.1）；`deleted_inodes::tests` 两条绿。审核的另一半（池级 checker 一侧、只读挂载）：审核报告与主 agent 判过（`records/2026-10-13-代码欠账批.md:167`），不在 W13 的射程，我没查。
推翻：合法写路径写出过宽不为 8 的 `deleted_inodes` 根条目；或一个宽不为 8 的根绕过 `crates/singlefs-core/src/recovery.rs:2995` 那一判进 `inode_numbers_of_the_deleted_inodes_root_entries`（今天唯一调用点在判之后）。

### L6.2 ⑥ `data_unit_payload` 的防护与 `parse_data_unit` 的判是不是同一个界

| # | 条款一句 | 代码处 | 判定 |
|---|---|---|---|
| 1 | 审核 ⑥：「⑥ 补一行防护」（`records/2026-10-13-代码欠账批.md:167`；同行说三处调用方都先经 `parse_data_unit` 核过声明长度） | `parse_data_unit`：「if payload_start + usize::from(declared_length) > bytes.len() {」（`crates/singlefs-core/src/unit.rs:567`）；`data_unit_payload`：「if payload_end > bytes.len() {」（`crates/singlefs-core/src/unit.rs:592`） | **一致，同一个界**：两处的起点都是 `DATA_UNIT_HEADER_BYTES`（105）加预留位 29 = 134，终点都是起点加声明长度；不等式化简后同为「起点 + 声明长度 > 单元长」 |

差别只在 `parse_data_unit` 先要单元长恰好 32768（`crates/singlefs-core/src/unit.rs:543`），`data_unit_payload` 收任意长度的切片；对 32768 字节的单元二者逐字相同，对更短的切片 `data_unit_payload` 现在报错而不是 panic（`bytes[payload_end..]` 之前已挡掉）。声明长度恰等于载荷容量（32634，终点 = 单元末尾）不拒，有单测（`data_unit_payload_with_a_declared_length_reaching_the_unit_end_exactly_is_the_whole_payload`，复跑绿）。两个调用方（`crates/singlefs-core/src/recovery.rs:5291`、`crates/singlefs-core/src/mounted_read.rs:948`）都在 `parse_data_unit` 之后；它们把 `data_unit_payload` 的任何 `Err` 都说成「补齐字节非零」（「detail: "补齐字节非零",」，`crates/singlefs-core/src/recovery.rs:5294`），所以新加的越界一支走到时报文会说错，今天走不到（因为 `parse_data_unit` 先拒），可记一笔。
**判定：一致。** 推翻：一个声明长度使 `parse_data_unit` 放行而 `data_unit_payload` 拒（或反过来）；谓词两头逐字相同，找不到。

### L6.3 注释说的是不是今天代码做的事

逐处读了 W13–W16 新写、改过的注释里承重的几处（diff 里 `///` 与 `//` 各处），下面是对不上或写宽的三处，其余的（`reread_at_after_a_bad_read` 的文档、`ReadRouteOfThisMount` 各臂、`ReadFailedWhileRehearsing…` 的文档、`MountError::Publish` 的文档、`unit.rs` 的越界文档、`deleted_inodes.rs` 的文档）与今天的代码对得上。

| 注释 | 代码做的 | 判定 |
|---|---|---|
| `crates/singlefs-core/src/mounted_session.rs:562`：「还是真做（取号、写行、暖机、抬 F、续做）——」接 `crates/singlefs-core/src/mounted_session.rs:563`：「都取记下的那一份，不再读盘」 | 取号读半在真做里读盘（`crates/singlefs-core/src/mount.rs:6869`），不取记下的那一份；同一段注释后文（`crates/singlefs-core/src/mounted_session.rs:567`）才说「读走哪一路看几块盘共用的 `read_route`」 | **写宽**：这一句把取号列进「都取记下的那一份」，与读盘三段之一冲突。`crates/singlefs-core/src/mount.rs:6276` 的「逐盘核、真做取号与实例表清扫三段照旧读盘」也写成整个真做取号，实际只有读半读盘、写半（系统配置槽算世代号与 F 生效值）取共用的（L1.2 序 f2） |
| `crates/singlefs-core/src/mount.rs:5302`：「只进内存那一趟与真做读同一份字节、走同一段，它报的错真做走到那一步照样报」；同样的意思在 `crates/singlefs-core/src/mount.rs:7104` | W16 之后读坏当场重读绕过记下的那一份（`crates/singlefs-core/src/mounted_session.rs:567` 的文档自己写的），重读是新的一次读盘，预演与真做各一次，可以不同 | **写宽**：对没有重读这一步的读成立（L2.3 量的），对 S1–S6 的重读不成立；W16 改读法时没回头改这两句 |
| `crates/singlefs-core/src/mounted_session.rs:564`：「读错的那一块不记，下一次读照样穿到盘上」，`crates/singlefs-core/src/mount.rs:6439`：「读错的那一块没记进共用的那一份，真做再读可能走出别的结局」 | 只对块设备报错的读成立；读成功而字节坏的读被记下（`crates/singlefs-core/src/mounted_session.rs:680`），真做不再读（L2.3） | **写宽**：「读错」在注释里没分「报错」与「读成功字节坏」，拒因的前提因此只对前一类成立 |

另一处小遗漏：`crates/singlefs-core/src/mount.rs:6277` 起列「读坏当场重读」的几处时少列了 S1（被抛弃根的账读不出那一次重读，`crates/singlefs-core/src/mount.rs:1832`）。不影响判。
推翻：把上面三处注释各配一个今天代码的反例复现为真（比如在取号读半里读到记下的那一份，或重读读到与预演的重读相同的字节）。我给的三处都是文字与代码并排对比出来的；没有改注释的意思，是否改并进修红批由主 agent 定。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| L1 共用的那一份：预演与真做读的是不是同一份 | 一致 | 同一个 `DeviceReadOnceInThisMount` 对象、同一张块表；写后读写下的字节（L1.1） |
| L1 三段照旧读盘 | 一致 | 逐盘核、取号读半、实例表清扫每次都读盘，不漏进共用那一份；取号**写半**与写行、暖机、判号（序 b）走共用（L1.2，ⓐ 的文字比代码窄，条款没冲突） |
| L1 三段要求现读盘的条款原文 | 一致（推） | D18 第 316、320 行，D23 第 394 行；W15 的编号对；条款没有一句明写「不许取记下的那一份」 |
| L1 ⓐ ⓒ ⓓ | 一致 | 无故障时不改写出的字节（读码 + 复跑钉值绿）；ⓒ 在说谎设备上挂载内读到写下的字节、挂载后读真盘 |
| L1 ⓑ | 规则没说 | 实现员补的一句，条款与判决都承认没写；触发集合是块设备报错的读（L2.3） |
| L1 记下那一份的内存有没有上界 | 规则没说；对「不撑爆合法挂载」这条判据：会（推的） | 无上界，且真做续做自己的读写都记下：量的斜率每个 32 KiB 数据单元多 64 KiB 读（两份）；1 TiB 外推 2 TiB 是推的；岔路二只点名了只进内存的盘 |
| L2 拒后盘不变、再挂照常 | 一致 | 拒在取号与任何写之前；复跑绿 |
| L2 偏拒 | 规则没说，自洽 | 拒的只会是读错造成的没走完（读码）；比例留给攻方腿 |
| L2 读成功而字节坏 | 规则没说（有反例，量的） | 窗口里两份树表读成功但字节坏：不拒，真做取号、写行、暖机、抬 F 落盘（420 个写）之后才交 `FloorRaiseFailedAfterTheMountsPublishes`；换成报错则取号之前拒、盘不变 |
| L2 按 `MountError::Publish` 认的调用方 | 成立 | 21 处调用方逐处查过；唯一落空的假设是 `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:798` 把 `Publish` 当取号之后，降级成停下的原因，且该二进制走不到这一拒 |
| L3 六处改到的重读 | 一致 | 都走 `reread_at_after_a_bad_read` 与自证后换；读盘三段里换是空操作 |
| L3 U1（只重读「读不出」） | 一致 | 读不出的块不记，第二次读本来就穿盘 |
| L3 U2（E2） | 字面不一致，方向安全，规则没说 | 判号那一读已把每个根环槽记下，E2 的「重读」读的是记下的字节，不是此刻读盘；坏/全零的槽钉在最旧 txg，回收少 |
| L3 默认实现走不走注入层 | 一致 | 走；全仓只有共用层与只进内存的盘覆写了重读 |
| L4 J1 | 一致 | 四条指针齐；谓词与写者断言是同一个函数；点名项的位置条目不在这一判里（不 panic，规则没说必须在核心做） |
| L5 比 fsid | 一致 | 取自开头择定的系统配置；合法写路径的头恒等；别池拒、本池放，各有用例 |
| L6 ② ⑥ | 一致 | `!= 8` 与写者条目宽 8 对得上；越界一判与 `parse_data_unit` 同一个界 |
| L6 注释 | 三处写宽 | `crates/singlefs-core/src/mounted_session.rs:562` 一句把取号列进都取记下的；`crates/singlefs-core/src/mount.rs:5302` 与 `crates/singlefs-core/src/mount.rs:7104` 的「读同一份字节」对重读不成立；「读错」不分报错与字节坏 |

## 没做什么

- 不判别的腿的格：没有替攻方腿造镜像去量「偏拒」的比例，没量 L2 ⓑ 被拒的比例；没有替主 agent 采纳或出判决，上面「该交主 agent / 用户」的两条出路只是并排列出来。
- 没复原 W13 之前的树，所以「W13 之前这一形会做成」「真做是独立的第二次采样」是推的。
- L1.5 的外推（1 TiB → 2 TiB）没量，斜率只来自窄盘 4、12、20 个单元三点；批大小 144 没在探针里开。
- L3.3 U2 没造镜像（要让判号那一读读到字节坏、之后读好）；L3 的名字 grep 没按语义逐调用点扫「读坏就再读」。
- 池级 checker 一侧（审核 ②）、`mount_transparency` 对新拒因的复算，没查。
- 没跑任何门禁阶段、checker 档测试、崩溃放量或第二台；cargo 只在本机草稿副本上跑，十几次前台命令（每条都经 `scripts/hosts/run-with-memory-cap.sh 14G` 与线程 31，没起后台）；没起 GPU。

## 要主 agent 跑的命令

- 复跑我的两个量法（草稿目录要新的）：
  `bash scripts/hosts/run-with-memory-cap.sh 14G bash research/prompts/m3-b44-code-r3-sonnet-model/tools/rerun.sh --repo ~/code/singlefs --frozen ~/.local/share/singlefs/work/m3-b44-code-r3/repo --work ~/.local/share/singlefs/work/<新目录>`，输出与 `research/prompts/m3-b44-code-r3-sonnet-model/out/rerun.stdout` 逐行比（用时行除外）。
- L1.5 的上界要真量：批大小取默认 144、被删数据量到 GiB 级的 E179，别用我的窄盘外推。
- L3 U2：造「判号那一读读到字节坏（读成功）、之后读好」的根环槽，看进程内记根后 E2 交 `ValidRoot` 还是钉着；我没造。

## 校验（第三步）与取舍

我做的是正推：读码对条款，加两个与产品代码不共用的探针（自己的读账包装、自己的 `Drop` 打印，都在草稿副本里加）。「校验」那一步按证据纪律要换一条独立的路子再做一遍同一个判断：L2.3 用「同一个窗口换两种读故障」当对照（报错 / 字节坏）；L1.5 用「读到盘上的 32 KiB 偏移数」与「共用那一份在 `Drop` 时的字节数」两个互不共用的计数器；两个探针的会红证明（往探针里塞已知故障看它会不会抓）只做了 L2.3 的一半——把窗口换成报错时探针确实走到取号之前那一拒，说明窗口真打中了；L1.5 的 `Drop` 计数器没单独证红。
