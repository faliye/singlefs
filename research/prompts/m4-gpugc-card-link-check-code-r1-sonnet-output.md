# m4-gpugc-card-link-check-code-r1：正推腿报告（2026-11-13）

腿：three-way-forward（Sonnet）。立场：从已定条款与冻结副本里的实现推出「应该是什么」，逐格比。只回 T1 与检查单八格；T2、T3 是攻方腿的格，这里不判。

## 读法与核过的前提

- 被判代码读冻结副本，下文引代码一律写绝对路径（前缀 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/`，后面接 `crates/…`；写绝对路径是因为交回闸拿主工作区的旧 `crates/` 核引文，写相对路径会对到旧代码）。表格里「某文件 第 N 行」指同一份冻结副本里的行，要核的引文在表后各节。
- 六份被判文件的 sha256 照 `research/prompts/m4-gpugc-card-link-check-code-r1-snapshot/crates-sha256.txt` 在冻结副本里核：`sha256sum -c` 六行全 OK（本腿跑的只读命令）。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` 第 253 行（已定项 13 第 4 条，整条在这一行）里从「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」到「这张卡再一次被判用不了时才再读一次。」那一段。我把那一段在该行里取出来逐句拆开，原文片段在 T1 表里整句引。
- 落点判法：「对」= 代码做的与那一句一致；「比条款宽」= 代码在条款没点名的时刻或对象上也做；「比条款窄」= 条款点名的时刻或对象上代码没做；「没落」= 找不到代码。
- 本腿没起任何程序、没跑测试：「罩着」写的是测试名、行号与它断言了什么（读源码得到），没有一条是本腿跑出的结果；变异行引的是冻结副本 `crates/mutations.tsv` 的行号（它在 `crates/` 里，随冻结副本带来）。

## T1　条款那一段，一句一行

| 编号 | 条款那一句（kb 第 253 行） | 代码落在哪 | 落得 | 罩着它的测试 |
|---|---|---|---|---|
| T1-1 | 「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」 | pci_state_of_the_gpu.rs 第 63–79 行 `pci_state_of_the_gpu`：每调一次读一次、计数加一（第 67 行）；七个调用点见 T1-4、T1-5、T1-6 | 读一次本身对；各调用点宽窄分记在 T1-5、T1-6、T1-16 | `pci_state_of_the_gpu::tests::every_file_present_is_read_as_its_text_with_the_address_and_the_directory_read`（第 324 行，断言 `readings_taken() == 1`，第 366 行） |
| T1-2 | 「照这张卡的 PCI 地址」 | 卡标识就是 `device_pci_bus_id` 原样的字节（adapter_inventory.rs 第 138 行、background_gpu.rs 第 1844 行）；读 `/sys/bus/pci/devices/<地址>`（pci_state_of_the_gpu.rs 第 26 行、第 72 行）。标识当不了目录名时不读（第 126–134 行）：条款没写，是实现员加的一道（T2 第 6 处，不在这一格判） | 对（地址来源与目录）；守卫是条款之外多一道，不改读的时刻 | 第 324 行；`identifiers_not_usable_as_a_directory_name_are_not_read_outside_the_root`（第 465 行）只罩着 `../x`、`..`、非 UTF-8 三种；空串、`.`、NUL 三个分支没有测试罩着（变异行 7871 只改 `/` 那一支） |
| T1-3 | 「读的是卡在不在总线上」及后面的使能位、链路四样、出错计数三样 | pci_state_of_the_gpu.rs 第 151–160 行九个读数一一对着条款点名的文件（`enable`、`current_link_speed`、`current_link_width`、`max_link_speed`、`max_link_width`、`aer_dev_correctable`、`aer_dev_fatal`、`aer_dev_nonfatal`），目录在不在看第 167 行 `read_dir` | 对（九样齐、名字逐个对得上） | 第 324 行（九样全在）；`the_device_directory_not_present_is_read_as_off_the_bus_with_every_file_not_present`（第 371 行）；`the_device_enable_reading_zero_is_read_as_disabled_and_off_the_bus`（第 408 行）；`device_enable_texts_are_read_as_disabled_enabled_with_the_count_or_unrecognized`（第 493 行）；变异行 7864–7870 |
| T1-4 | 「挂着时一批在同一张卡上等超时、到判那张卡用不了」 | 门在 pci_state_of_the_gpu.rs 第 102–121 行（第 104 行 `UnusableAfterResubmissions` 判真）；两处调用：background_gpu.rs 第 3555 行（交活之前那一等用完）、第 3694 行（批里）；到核心层的转换 background_gpu_compute.rs 第 390–447 行 | 对 | `the_card_unusable_after_its_resubmissions_is_read_once_and_the_batch_failure_carries_the_reading`（background_gpu.rs 第 4857 行，3 次超时重交加第 4 次超时，读一次）；`the_card_unusable_while_waiting_before_the_batch_is_read_once_and_the_failure_carries_the_reading`（第 4949 行，交活之前那一等）。两条都只走校验和那一段；区域扫描与选段打分两段读的是同一处代码，可它们把读数带进 `GpuFailed` 的转换（background_gpu.rs 第 744–748 行、第 777–781 行）没有各自的测试，核心层转换里这两段（background_gpu_compute.rs 第 593–594 行、第 631–632 行）的测试只喂「没读」 |
| T1-5 | 「开卡重开用完都开不成」 | 挂载：gpus_of_this_host.rs 第 931–938 行；切换开卡线程重开用完：background_gpu.rs 第 1701–1702 行；起不了线程而次数用完：第 3263–3265 行；累计试满当场答开不成：第 1649–1650 行（线程里）、第 3130–3132 行（交设置那一刻） | 重开用完那一刻：对；另有三处比条款宽（见 T1 打中 A、B、C） | 挂载：`the_card_refusing_the_mount_carries_its_pci_state_read_once_its_reopenings_are_used_up`（gpus_of_this_host.rs 第 1576 行）、`the_card_opening_at_its_last_reopening_at_the_mount_is_not_read`（第 1594 行）；切换：`the_switch_not_opened_is_read_once_and_every_later_answer_and_batch_carries_that_reading`（background_gpu.rs 第 5999 行）、`the_card_opened_at_its_third_try_at_a_switch_reads_no_pci_state`（第 6044 行）。**没有断言读数的**：第 3265 行（起不了线程）、第 1650 行（线程里试满）两处，没有测试或变异行断言它们读了或带的是哪一份 |
| T1-6 | 「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」 | pci_state_of_the_gpu.rs 第 105 行 `UncapturedErrorOrDeviceLost` 判真；调用同 T1-4 的第 3694 行 | 批里这一条路对；另两处与条款不齐（见 T1 打中 D、E） | `the_card_lost_in_a_batch_is_read_once_and_the_batch_failure_carries_the_reading`（第 4897 行）；`a_batch_failing_without_judging_the_card_unusable_reads_no_pci_state`（第 4923 行，等提交没等成不读） |
| T1-7 | 「平常的批、每次超时重交之前不查」 | 门函数第 102–121 行把「一次超时（还要重交）」判假；开卡每一次没开成而还要重开时不读（background_gpu.rs 第 1695–1707 行只在 `Err` 整串出来之后读一次） | 对 | `the_batch_completing_after_three_timeouts_reads_no_pci_state`（第 4884 行）、第 4857 行（重交 3 次只读 1 次）、第 6044 行、gpus_of_this_host.rs 第 1594 行；变异行 7880、7881、7883 |
| T1-8 | 「读数写进报警与观测，行为不变」之「写进报警」 | core background_compute.rs 第 1534–1537 行：报警的读数从 `error_that_stopped_the_task` 现取 | 对 | `the_alarm_at_every_wake_while_stopped_carries_the_pci_state_read_when_the_gpu_was_judged_unusable`（core 第 3571 行）；变异行 7856 |
| T1-9 | 同句之「观测」 | 观测有两处：错里带读数（core 第 1402 行 `compute_error_that_abandoned_the_batch` 带的错）、读次数计数器（background_gpu.rs 第 2806 行 `pci_state_readings_taken`） | 对 | 计数器有断言（第 4879、6039 行等）；`compute_error_that_abandoned_the_batch` 那一路没有单独断言读数 |
| T1-10 | 同句之「行为不变：GC 照旧停下等人处理，照旧按占用在 CPU 上补跑，前台的空间拒绝报不加字段」 | 停下的判（core 第 1459 行 `reach_of_the_error`）、记停下（第 1730 行）、补跑的判（第 1867 行 `whether_the_task_runs_the_next_batch`、第 1885 行）三处在补丁全文里没有改动；非测试代码里没有拿两个出错比相等的（见 T3 的核对一并交攻方）；补丁没碰前台空间拒绝的类型（补丁全文 grep `space`、`ENOSPC`、`空间拒绝` 在新加行零命中） | 对（读代码；没有专为它新写的行为测试） | 原有停下与补跑的测试只是随新字段补了 `NotReadForThisError`；这一格我没有新测试可指 |
| T1-11 | 「内核日志里的 Xid 与 `nvidia-smi` 不读」 | pci_state_of_the_gpu.rs 产品部分（第 1–223 行）只调 `std::fs::read_dir` 与 `std::fs::read`（第 167、178 行），没起外部程序、没读内核日志；grep `nvidia-smi\|dmesg\|kmsg\|Command::new\|Xid\|journalctl` 在该文件零命中 | 对 | 没有测试专罩「不读」；grep 零命中是证据 |
| T1-12 | 「读不到的那一样（文件不存在、读不通）报」…「，不当成 0，也不当成正常」 | core 第 490–498 行 `PciFileReading` 三态；读文件第 177–185 行：不存在报 `FileNotPresent`、别的错报 `NotReadable` 带系统原话；读不到的那一样不折成数字也不折成「正常」 | 对 | 第 371 行（九份都不在）、`the_device_enable_not_readable_is_reported_with_the_system_error_and_not_judged_off_the_bus`（第 433 行）；变异行 7867、7870 |
| T1-13 | 「只有卡不在总线上（那个 PCI 地址的目录不在了，或使能位是关的）才判硬件坏」及「链路速度与宽度、出错计数只报不判」 | core 第 534–555 行 `hardware_judgement_of_the_pci_state`：目录不在、或 `enable` 读到 0 才判坏，别的（目录读不通、`enable` 不在或读不通、认不出的原文、链路、计数）不判 | 对；但这个判定函数没有产品代码调用，只有测试调它（函数名与 `hardware_judgement` 方法 grep 全仓，非测试零调用） | core 第 3491、3505、3517、3529、3555 行五条（目录不在判坏、使能关判坏、降速与出错计数不判、使能读不到不判、目录读不通看使能）；变异行 7857–7861 |
| T1-14 | 「挂载那一刻开卡重开用完、拒绝挂载的，读数写进挂载的报错」 | gpus_of_this_host.rs 第 936 行读、拒绝成员 `GpuUnusable` 带读数（background_gpu.rs 第 232–236 行）；挂载报错文本是 `format!("{refused:?}")`（gpu_opener_of_this_process.rs 第 194 行），经 `RefusedByTheOpening { description }` 进运行库，运行库 lib.rs 没改 | 对 | 读数进拒绝成员有测试（gpus_of_this_host.rs 第 1576 行，变异行 7882）；**成员 Debug 文本进 `description` 这一跳没有测试** |
| T1-15 | 「停着之后每次叫起交回的报警带停下那一刻读的同一份读数，不重读」 | core 第 1828–1829 行报警克隆停下记录里的错、读数现取（第 1534–1537 行）；GPU 包侧开不成之后每一问、每一批报的是同一份（background_gpu.rs 第 3403 行起、第 3458–3464 行），不重读 | 对 | 核心层：第 3571 行（停下那一趟加之后三次叫起都是同一份）；GPU 包：第 5999 行（读次数始终 1）；变异行 7856、7878 |
| T1-16 | 「这张卡再一次被判用不了时才再读一次」 | 没有缓存：每个判用不了的调用点（T1-4、T1-5、T1-6）每次都现读；换设置清掉上次的开不成（background_gpu.rs 第 2856 行）、停下记录先到先得（core 第 1748 行 `if !self.is_stopped_after_a_compute_error(task)`） | 对（字面）；「再一次」包含的几种重复读见 T1 打中 B、C，读次数没有上界的一支交 T2 第 4 处 | 「不重读」那一半有测试（第 5999、3571 行）；**「再一次判才再读」这一半没有断言读次数涨到 2 的测试**：第 6509 行那一条 `the_same_settings_handed_in_again_once_the_openings_of_the_card_are_used_up_open_it_no_more` 只断言第二次答出的读数内容 |

## T1 打中：条款与代码不齐的地方，两边整行并排

每一条都是「条款一句」对「代码一行」；引文都在被引文件里逐字找过一次（交回前还跑 `cite-check.py`）。「推」= 读代码推出、没起程序核；每条末尾写什么现象会推翻它。路径前缀 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/` 在下文引文里原样写出。

### A　开卡被叫停（卸载、切到 `cpu`）时也读一次：比条款宽，没有测试罩着

- 条款：「开卡重开用完都开不成」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。叫停不是卡开不成：代码自己也把叫停那一次从「连着开不成的次数」里排除，见「GpuFailureCause::OpeningCalledOff」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1585`）。
- 代码，一条线。重开之前的睡眠只在没叫停时睡：「if !called_off.load(Ordering::SeqCst) {」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1663`）。叫停之后每一次开卡都当场交回叫停的原因：「Err(opening_called_off())」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1669`）。重开的循环不因此提前退出：「for reopenings_so_far in 0..=reopenings.reopenings_at_most {」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:399`），原因一个个收进整串：「Err(cause) => causes_of_each_opening.push(cause),」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:405`）。整串一出来就读，不看里面是什么原因：「Err(causes_in_the_thread) => {」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1695`），读在同文件第 1702 行。
- 后果（推）：卸载或切到 `cpu` 时，开卡线程上还在开、还没开成的那一张卡，线程尾巴上读一次 PCI 状态；结局送不出去、被丢掉：「let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3220`）。读数不进任何报警，读次数计数器却加一，「三种时刻之外不读」被破。读的是普通文件、没有副作用，不影响行为不变。
- 没有测试罩着：叫停那一条 `fn no_gpu_taking_part_while_a_card_opens_does_not_sleep_out_the_interval_before_a_reopening() {`（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:8964`）断言的是开卡次数，不断言读次数。
- 什么现象会推翻它：照那条测试的装置、叫停之后在 `pci_state_readings_taken()` 上断言 0 而它是绿的，就是我读错了第 1663、1669、1695 行的流程。

### B　累计试满：同一组设置每交一次读一次，读次数没有上界；条款对「再一次」没分清

- 条款：「开卡重开用完都开不成」与「这张卡再一次被判用不了时才再读一次」（都在 `.claude/kb/decisions/26-后台整理与放置回收.md:253`）。
- 代码：交设置那一刻，点到的卡在这次挂载里连着开不成的次数已到上限，就当场答开不成并读一次，不记「这次试满已读过」：「self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2864`），读在第 3132 行；开卡线程里开到这一刻已试满同样读一次（读在第 1650 行）。
- 两种读法：把「再一次被判用不了」读作「又发生一次新的用不了」，这一支每交一次设置读一次、同一次试满读 N 次，比条款宽；读作「又判一次」，字面对。条款没分。这是主 agent 要定的读法，T2 第 4 处的上界问题落在这里。
- 同一次试满读两次的一个例子（推）：`the_same_settings_handed_in_again_once_the_openings_of_the_card_are_used_up_open_it_no_more`（同文件第 6509 行）里，线程开完 4 次读一次（第 1702 行），之后同一组设置又交一次读第二次（第 3132 行）；那条测试只断言第二次答出的读数内容，不断言读次数。
- 什么现象会推翻它：一条测试交同一组设置 N 次、断言读次数仍是 1，而它是绿的。

### C　开卡结局被更新的交设置盖掉、没被采用时，读已经做了

- 代码：线程尾巴上读（第 1702 行），收结局的一头只在次序号对得上时才用：「if settings_waiting.handing_in_number == handing_in_number {」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3050`）。对不上的那一份读数不进任何报警。比条款宽一次读，同 T2 第 3 处，这里只记事实。
- 什么现象会推翻它：盖掉的那一次换设置之后，`pci_state_readings_taken()` 断言没涨而它是绿的。

### D　切换时量代价出错：卡当场报出的错照样没读，比条款窄

- 条款：「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。
- 代码：量代价出错的卡之后每一批报成它的 GPU 出错，读数一律是没读：「Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3466`），第 3472 行交 `NotReadForThisError`；第 3467 行的注释自己写着「换设置时量代价出错不是判一张卡用不了的那三种时刻」。
- 推：量代价那一次提交走的是同一条等完提交与读回的路（`measured_cost_of_one_submission` 经 `both_kernel_outcomes_of_a_checksum_chunk`），那条路把 wgpu 记下的错报成「return Err(GpuFailureCause::UncapturedErrorOrDeviceLost { reason });」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2251`）。所以量代价时设备丢了，报出的 `cause` 就是条款点名的那一种，任务照样因此停下，却没读。我没逐层追到量代价那一支一定走进第 2251 行那个函数，这一步标「推」。
- 什么现象会推翻它：量代价那一支的出错在到 `cause` 之前被换成别的原因（那时这一条只是文字上的窄）。条款没有把「量代价时卡报出错」排出去，这是 T2 第 2 处要对的读法。

### E　读发生在 GPU 包，停不停由核心层定：切换请求挂着时读了而不停下

- 条款：「卡当场报出错……、后台任务因此停下」（同上第 253 行）。
- 代码：核心层收到出错后，出错之前就挂着还没交给实现的切换请求的，当场解除、任务不记成停着：「if self.requested_from_the_next_batch.is_some() {」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1741`）。读在更早的 GPU 包里已经做了（background_gpu.rs 第 3694 行）。所以这一格读了而没停下，比条款多一次读；错和读数照样报在这一批的记账里（core 第 1402 行）。
- 什么现象会推翻它：切换请求挂着时出错的那一批，`compute_error_that_abandoned_the_batch` 里读数是没读（那说明读被挡在前面；我没在代码里看到挡）。

### F　不算打中、但要记下的两处

- 「等提交没等成」「回读映射不成」同样让任务停下，代码不读：「| GpuFailureCause::UncapturedErrorOrDeviceLost { .. } => true,」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:105`）之外的原因在第 106–119 行全判假。条款点名的是「设备丢失、驱动报的未捕获的错」，字面上代码对；这是 T2 第 1 处。
- 核心层的判硬件坏函数「pub fn hardware_judgement_of_the_pci_state(」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:534`）没有产品调用点：条款「才判硬件坏」落成了一个只给读数的人调的纯函数，报警里不带判定结果。条款没说判的结果放哪，所以不算没落，但「判」只在测试里发生。

## 检查单八格

### 走不到的理由二次查实：成立

- 方法：对这一批改到的六份产品源文件，读补丁全文里每一个新加行，按 hunk 头分「mod tests 区块」与「不在测试里」，在后者里搜 `todo!`、`unreachable!`、`assert!`、`assert_eq!`、`assert_ne!`、`debug_assert`、`panic!`、`.expect(`、`.unwrap(`、`_ =>`、`Undecided`、`Unsupported`、`OutsideTheFirstVersion`。脚本 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1-sonnet/scan-added-lines.py`（只读，主 agent 可直接跑：`python3 <脚本> research/prompts/_m4-gpugc-card-link-check-code-r1-diff.md`）。
- 结果：除新文件 `pci_state_of_the_gpu.rs` 外，五份文件不在测试里的新加行零命中；新文件整份是一个 hunk，脚本把它的命中都归到「不在测试里」，我另用 `head -257 pci_state_of_the_gpu.rs | grep -nE 'todo!|unreachable!|assert|panic!|\.expect\(|\.unwrap\(|_ =>|Undecided|Unsupported|OutsideTheFirstVersion'` 核它的产品部分（第 1–257 行，测试区块从第 258 行起）：零命中、退出码 1。脚本列出的 24 行命中全在第 280 行以后，即测试区块里。新加行里没有以 Undecided、Unsupported、OutsideTheFirstVersion 收尾的名字。
- 实现员那句「新加的 `assert!` 都在 `#[cfg(test)]` 模块里」：核过，对。
- 什么现象会推翻它：脚本在「不在 mod tests 区块」那一段列出 `pci_state_of_the_gpu.rs` 以外文件的一行，或 `head -257` 那一条有命中。

### 类型与名字：成立（一处要主 agent 知道）

- 同一个量不同语义共用裸整数：新加的整数只有三处，各是各的量：使能计数「enable_count: u64,」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:504`，内核记的使能计数）、读次数「readings_taken: Arc<AtomicU64>,」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:33`）、对外的 `pci_state_readings_taken() -> u64`（background_gpu.rs 第 2806 行，同一个量）。链路速度与宽度读成 `String`、没折成裸整数。
- 缩写：新加的非注释非字符串代码里，三个字母及以下的词除登记过的 `pci` 外，只有标准库与语言自带的名字（`buf` 在 `PathBuf`、`fs`、`dir` 在 `create_dir_all`、`err`、`std`、`io`、`env`、`seq` 在 `SeqCst`、`id` 在 `std::process::id`、`cfg`、`eq` 在 `assert_eq`）。`pci` 登记行：「peripheral component interconnect：外设总线标准」（`.claude/abbreviations:9`）。内核的文件名 `aer_dev_*` 只出现在字符串与注释里，代码名字写全成 `advanced_error_reporting_*`。要主 agent 知道的一处：`gpu`、`cpu` 不在 `.claude/abbreviations` 里，新名字（`PciStateOfTheGpu` 等）沿用存量里几百处同样的用法；若评审按登记表字面判，那是存量问题，不是这一批新增的。
- `_ =>`：新加行零命中（同上脚本）。穷举：「GpuFailureCause::NotOpenedAfterTheReopenings { .. } => false,」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:119`）收尾，判一个出错是不是读的时刻把每个成员逐个列出；核心层 `pci_state_when_judged_unusable`（core 第 330–354 行）、`hardware_judgement_of_the_pci_state` 同样逐个列。
- `expect`：新加的 `expect` 全在测试辅助里（pci_state_of_the_gpu.rs 第 280、287、288、299 行，gpus_of_this_host.rs 的测试里），消息写的是前提（「临时目录下建得了用例自己的目录」「点名本机的一张卡：校验合法」「第四次开成：挂载照常」）；产品部分新加零个 `expect`。
- 什么现象会推翻它：产品部分有一个新加的整数既当计数又当别的量用，或有一个新名字含登记表外、标准库外的缩写。

### 不兼容旧代码与旧格式：成立

- 补丁全文新加行 grep `legacy|compat|deprecated|旧|兼容|向后|backward|Default::default|unwrap_or_default|serde(default|version|VERSION|migrat|迁移`：零命中。
- 改到的 11 个文件全是代码与测试，没有盘上格式、版本号常量、迁移路径。
- 没有为旧调用方留缺省参数：读的那一份是必填参数「pci_state_reader: &PciStateReaderOfTheGpus,」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/gpus_of_this_host.rs:897`）；`NotReadForThisError` 是「这个错不是那三种时刻」的语义成员，产品代码在用（background_gpu.rs 第 3472 行、core 第 351 行），不是给旧调用方留的占位。
- 什么现象会推翻它：补丁里有一个新加的 `Option` 或缺省值，让不传读器的老写法照样编得过。

### 不新建判定库：成立

- 新模块只读 sysfs，产品部分不写任何文件；补丁新加行里写盘的只有测试辅助在临时目录建夹具文件（pci_state_of_the_gpu.rs 第 280–299 行，`#[cfg(test)]`），夹具不是判定。grep `rocksdb|sqlite|\.tsv|OpenOptions|File::create|prune-server` 新加行零命中。
- 什么现象会推翻它：新加行里出现另开的判定存放处（TSV、sqlite、另一份 RocksDB）。

### 所有任务多机可用：成立；第二台复核不了

- 没有任何新测试读本机真 sysfs：产品读器 `PciStateReaderOfTheGpus::of_this_host()` 只在产品开卡处造（background_gpu.rs 第 1135 行），grep 全仓，测试里零处用它；读真目录的用例在临时目录里现造文件：「let root = std::env::temp_dir().join(format!(」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:276`）；「不在的根」是固定绝对路径：「PathBuf::from("/singlefs-background-gpu-tests-pci-device-directories-never-created")」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:226`）。没有一条新测试的判定随有没有显卡、sysfs 长什么样而变。
- 固定路径在本机读成目录不在：`ls -d` 报「No such file or directory」，`os.listdir('/singlefs-background-gpu-tests-pci-device-directories-never-created/<某个 PCI 地址>')` 报 `ENOENT`（本腿跑的两条只读命令）；在根目录下建目录要超级用户，普通测试进程建不了。第二台上有没有人建过这个目录：复核不了，原因是本腿不连第二台；要核可在第二台上 `ls -d` 那条路径。
- 本机真 sysfs 的实情（只读，`/sys/bus/pci/devices` 下三张显示类设备）：两张英伟达卡九个文件全在（`enable` 读到 `1`），一张 AMD 卡有 `enable`、链路四样，没有三份 `aer_dev_*`。这正是条款要求「读不到报读不到」的那一格，测试里的 `FileNotPresent` 与它对得上。
- 什么现象会推翻它：有一条新测试在没显卡的机器上红，或固定路径在某台机器上存在。

### 测试优先多线程：成立

- 每条用例各造各的目录：目录名带进程号、用例名与进程内序号：「static DIRECTORIES_MADE_IN_THIS_PROCESS: AtomicU32 = AtomicU32::new(0);」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:274`），那个静态量只用来取不重复的序号，丢掉时整个删掉。读次数的计数器每造一份读器就新造一个：「readings_taken: Arc::new(AtomicU64::new(0)),」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:50`），别的包里的用例每条自己调 `pci_state_reader_under_a_root_not_present_for_tests()` 造新的，没有跨用例共用的计数器。
- 什么现象会推翻它：两条用例拿同一份读器或同一个目录名，在并行时读次数互相串。我 grep 了所有调用点，没找到。

### 带 --item 的命令与项：推不出（我不跑命令），静态上站得住

- 冻结副本 `crates/mutations.tsv` 第 7856–7885 行是这一批的 30 条变异行；我逐行核了：每条的「原串」在它指的源文件里恰好命中一次，每条点名的测试函数在对应包的源码里存在（30 行全过）。「11 行整行替换」与「都能被 `--item` 点名」要真跑才知道，这是主 agent 在快照上点名跑过的事，我不重复；这一格写「推不出」。
- 与 T1 的关系：这 30 条变异没有一条罩着 background_gpu.rs 第 1650、3265 行的读，也没有一条罩着 background_gpu_compute.rs 第 593–594、631–632 行带读数的转换，也没有罩着挂载报错的 Debug 文本那一跳（T1-5、T1-4、T1-14）。
- 什么现象会推翻它：`crates-mutation-replay.sh --list-items` 点不到第 7856–7885 行的某一行。

### 新崩溃流多罩了什么：成立

- 补丁全文新加行 grep `crash|Crash|崩溃|fsync|sync_all|unsafe|CrashPoint` 零命中；读数只在内存里带，不写盘，没有新的崩溃点重放流或用例。
- 什么现象会推翻它：补丁里有一处新加的写盘或 fsync。

## T1 各行什么现象会推翻

- T1-1、T1-2、T1-3、T1-4、T1-7：「要主 agent 跑的命令」第一条里点名的用例有一条红（说明我读的断言与实际不符），或条款第 253 行被改了字。T1-3 另有一条：真机上某个文件名读出来不是条款写的那一个（本机三张卡已对过，见检查单「所有任务多机可用」）。
- T1-5、T1-6、T1-16 的「对」：`pci_state_readings_taken()` 在叫停、交同一组设置多次、被盖掉的开卡这几种情形里的数，与我在 A、B、C 里推的不同。
- T1-8、T1-15：核心层测试第一条红。
- T1-9、T1-10：有哪一处非测试代码拿两个出错比相等（我 grep 了出错类型与几个常用变量名，零命中；grep 认不出的写法是这一条的射程之外，交 T3）。
- T1-11：模块里出现起外部程序或读内核日志的调用。
- T1-12、T1-13：核心层里判定那五条测试有一条红；或有产品代码调判定函数却把「读不到」当成坏。
- T1-14：`BackgroundGpuOpenRefused::GpuUnusable` 的 Debug 文本里没有读数（那一跳没有测试，我只能从 `derive(Debug)` 推它有）。

## 要主 agent 的读法

- B 的「再一次被判用不了」到底是「又发生一次新的用不了」还是「又判一次」：条款没分，两种读法下 T2 第 4 处的上界结论相反。
- D 与实现员第 2 处：条款把「卡当场报出错」列成一种时刻；量代价那一次提交出的同一种错，条款没排除，代码排除了。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1-1 查一次 | 对 | 每次调用读一次、计数加一；七个调用点的宽窄分记在后面几行 |
| T1-2 照 PCI 地址 | 对 | 地址取自 `device_pci_bus_id` 原样；守卫是条款外多一道；空串、`.`、NUL 三支没测试 |
| T1-3 九样 | 对 | 九个文件名与条款逐个对上，本机真 sysfs 核过 |
| T1-4 等超时用完 | 对 | 两处读（交活之前那一等、批里），只有校验和那一段有测试 |
| T1-5 开卡重开用完 | 对，另三处比条款宽 | 叫停也读、试满每交一次读一次、被盖掉的结局已读（A、B、C）；两处读没有测试断言 |
| T1-6 卡当场报出错 | 批里对；两处不齐 | 量代价出错同一种错不读（比条款窄，D）；挂着切换请求时读了而不停（E） |
| T1-7 平常不查 | 对 | 四条测试加变异行罩着 |
| T1-8 写进报警 | 对 | 报警的读数从停下的那个错现取 |
| T1-9 写进观测 | 对 | 错里带读数、读次数有计数器；出错那一路没有单独断言 |
| T1-10 行为不变 | 对（读代码） | 停下、补跑、前台空间拒绝三处补丁没碰；没有专为它新写的行为测试 |
| T1-11 不读 Xid 与 nvidia-smi | 对 | 只调 `read_dir` 与 `read`，grep 零命中 |
| T1-12 读不到不当 0 不当正常 | 对 | 三态读数，不折成数字 |
| T1-13 只有不在总线上才判坏 | 对 | 判定函数只有测试调用，没有产品调用点 |
| T1-14 挂载报错带读数 | 对 | 读数进拒绝成员有测试；Debug 文本进 `description` 那一跳没测试 |
| T1-15 停着之后同一份不重读 | 对 | 现取自停下的错；两层各有测试 |
| T1-16 再一次判才再读 | 对（字面） | 「不重读」有测试；「再读」那一半没有断言读次数的测试 |
| 走不到的理由二次查实 | 成立 | 新加行里产品部分零命中，实现员那句话对 |
| 类型与名字 | 成立 | 无共用裸整数、无新缩写（`gpu`、`cpu` 不在登记表是存量）、无 `_ =>`、产品部分无新 `expect` |
| 不兼容旧代码与旧格式 | 成立 | 新加行零命中，读器是必填参数 |
| 不新建判定库 | 成立 | 产品部分不写盘 |
| 所有任务多机可用 | 成立（第二台复核不了） | 测试不读真 sysfs；固定路径本机读成 `ENOENT` |
| 测试优先多线程 | 成立 | 目录与计数器各用例各造各的 |
| 带 --item 的命令与项 | 推不出 | 30 行变异行静态上站得住；真点名要主 agent 跑 |
| 新崩溃流多罩了什么 | 成立 | 没有写盘与崩溃点 |

## 要主 agent 跑的命令

以下两条是核 T1 表里「罩着它的测试」那一列点名的测试真的存在且绿；跑在带这一批的树上（合入后的主工作区仓根，或主 agent 铺好的开工快照树的仓根，`--cwd` 写那一个）。本腿没跑，表里一切关于测试的话都是读源码得到的。

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试里本批新加的读数用例; 范围=点名 1 个模块加 11 个测试函数; 为什么=核 T1 表点名的测试存在且绿，不跑整包' task-server/task submit --cwd <带这一批的仓根> -- cargo test -p singlefs-background-gpu --lib -- pci_state_of_the_gpu::tests the_card_unusable_after_its_resubmissions_is_read_once_and_the_batch_failure_carries_the_reading the_batch_completing_after_three_timeouts_reads_no_pci_state the_card_lost_in_a_batch_is_read_once_and_the_batch_failure_carries_the_reading a_batch_failing_without_judging_the_card_unusable_reads_no_pci_state the_card_unusable_while_waiting_before_the_batch_is_read_once_and_the_failure_carries_the_reading the_switch_not_opened_is_read_once_and_every_later_answer_and_batch_carries_that_reading the_card_opened_at_its_third_try_at_a_switch_reads_no_pci_state the_same_settings_handed_in_again_once_the_openings_of_the_card_are_used_up_open_it_no_more the_card_refusing_the_mount_carries_its_pci_state_read_once_its_reopenings_are_used_up the_card_opening_at_its_last_reopening_at_the_mount_is_not_read the_pci_state_read_by_the_shell_is_carried_into_the_core_failure_of_the_three_moments`。在哪里起：`--cwd` 那一个仓根。看哪几行：`TASK_SERVER_RESULT` 那一行 `type=` 与 `exit=0`；`task-server/task log <号>` 里 `test result: ok. 18 passed`（模块里 7 条加点名的 11 条；名字都是整名，不会多带别的）。
2. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试里本批新加的读数与判定用例; 范围=点名 7 个测试函数; 为什么=核 T1 表点名的核心层测试存在且绿，不跑整包' task-server/task submit --cwd <带这一批的仓根> -- cargo test -p singlefs-core --lib -- the_alarm_at_every_wake_while_stopped_carries_the_pci_state_read_when_the_gpu_was_judged_unusable errors_not_judging_a_gpu_unusable_carry_no_pci_state_read the_gpu_whose_device_directory_is_not_present_is_judged_off_the_bus the_gpu_whose_device_enable_reads_disabled_is_judged_off_the_bus the_gpu_enabled_with_a_degraded_link_and_errors_counted_is_not_judged_off_the_bus the_gpu_whose_device_enable_is_not_read_is_not_judged_off_the_bus the_gpu_whose_device_directory_is_not_readable_is_judged_by_its_device_enable`。在哪里起：同上。看哪几行：`test result: ok. 7 passed`。

没有要跑的程序或模型：A、B、C 要核得新写一条测试（把读器换成假目录、叫停一次开卡、交同一组设置 N 次，断言读次数），那是实现员的活，这一轮不写。

## 没做什么

- 没判 T2、T3 的格，没替主 agent 采纳或出判决；没起任何程序，没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 第二台上固定路径存不存在、30 行变异行真点名红不红、测试真绿不绿：复核不了或待主 agent 跑，已逐处写明。
- 本腿的草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1-sonnet/` 里留着 `scan-added-lines.py`（检查单第一格引的只读脚本），别的中间文件交回前已删；整目录 108K、没有副本或编译目录。
