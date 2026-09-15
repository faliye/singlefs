# E152 跑前登记：按里程碑对比六家文件系统的文件性能

写于 2026-09-15 00:28 JST，装置写之前。

## 一、问的是什么

用户 2026-09-15 定向：拿 XFS、ext4、F2FS、Bcachefs、Btrfs、OpenZFS 六家和 singlefs 按文件性能对比，
维度是大文件顺序写、大文件顺序读、4K 随机读、4K 随机写、元数据 / 小文件，主 agent 再补几维；
结果放 `research/` 下一份按里程碑对比的文件，用来找差距。

这一轮只出观测，两件：

1. 六家在同一台虚机、同一种虚拟盘、同一组负载下每一维的数，外加裸盘一行当天花板。
2. singlefs 在里程碑「第一个事务」上能跑的那几维的数；跑不了的维写明缺什么能力。

「这些差距说明 singlefs 该怎么改」是推论，不在这一轮里。跑之前不预设谁快谁慢，也没看过任何一家在这个测试台上的数。

## 二、被测对象

| 配置 | 内核侧（6.17.0-1028-oem） | 用户态 | 格式化 | 挂载 |
|---|---|---|---|---|
| raw（裸盘，天花板） | — | — | 不格式化 | 不挂载，fio 直接打 `/dev/vda` |
| ext4 | 编进内核 | e2fsprogs 1.47.0（本机） | `mkfs.ext4 -F -q`，带本机 `/etc/mke2fs.conf` | `-o noinit_itable`（不让 lazyinit 线程在量的时候后台写 inode 表） |
| xfs | 模块 | xfsprogs 6.6.0（本机） | `mkfs.xfs -f -q` | 默认 |
| f2fs | 模块（依赖 lz4_compress、lz4hc_compress） | f2fs-tools 1.16.0（本机） | `mkfs.f2fs -f -q` | 默认 |
| btrfs | 模块（依赖 raid6_pq、xor） | btrfs-progs 6.6.3（本机） | `mkfs.btrfs -f -q`（单盘默认：数据 single、元数据 DUP） | 默认 |
| bcachefs | 模块（6.17 树内；依赖 raid6_pq、xor、两个 lz4） | bcachefs-tools 24+really1.3.4-2build2（Ubuntu 24.04 的包，`bcachefs version` 报 1.3.3，比内核旧） | `bcachefs format -f -q` | 默认 |
| zfs | 模块 2.3.4-1ubuntu2（OEM 内核自带；依赖 spl） | zfsutils-linux 2.3.4-1ubuntu2（Ubuntu 25.10 的包，与模块版本逐字相同） | `zpool create -f -m /mnt bench /dev/vda`（默认属性：recordsize 128K、压缩 on） | 由 zpool 挂；重挂 = export 再 import |
| singlefs | — | `crates/` 里门禁 55 号用的静态二进制 `first_transaction_on_device` | 它自己的 mkfs | 它自己的冷恢复 |

参数一律取各家默认，只有 ext4 那一个挂载选项例外；不调优。

## 三、测试台

- 宿主：本机（32 核、60 GiB、NVMe Kingston SNV3S4000G）；QEMU 8.2.2 + KVM，`-cpu host`，4 vCPU、4 GiB 内存。
- 虚机内核：6.17.0-1028-oem，与宿主正在跑的同一个版本；模块取 `/lib/modules/6.17.0-1028-oem` 解压后的 `.ko`。
  **不用 `/boot` 下唯一可读的 lockdep 内核**：锁校验的开销因家而异，会把比较本身弄歪（`.claude/kb/vm-harness.md` 已写明它量出来的时间不可比）。
- 盘：六家各一块 virtio-blk，16 GiB；singlefs 两块各 16 GiB（D2（RAID 条带策略） 已定项 9 定第一版跑 2 块盘）。
  宿主上是 TMPDIR 里的 raw 文件，`cache=none,aio=native`，fallocate 预分配（不让宿主 ext4 在大文件顺序写时边写边分配）。每一轮新建。
- initramfs：busybox + fio 3.36（本机）+ 各家用户态与依赖库 + 模块 + E152 二进制；`/dev` 挂 devtmpfs（zpool 给整盘分区后要看得到分区节点）。
- ⚠️ 这是虚拟盘上的数：宿主页缓存被 `cache=none` 绕开，宿主 NVMe 自己的缓存、宿主 ext4 的未写区段转换都算在里面。
  八个配置同一个测试台，比的是相对位置；绝对值不许拿去和外面公开的数比。

## 四、每一轮做什么（一个虚机一轮，按这个次序）

六家：

1. 格式化（计时，F），挂载，`df` 取总容量与可用字节（F）。
2. 小文件持久化 P：在刚格式化的空文件系统上建 100 个新文件，每个写 3000 字节、fsync 文件、fsync 所在目录；每一次单独计时、单独读块层计数。
3. 大文件顺序写 W1：`fio --rw=write --bs=1M --size=4G --ioengine=libaio --iodepth=16 --direct=1 --end_fsync=1`，一个 4 GiB 文件。
4. 重挂（卸载、`/proc/sys/vm/drop_caches` 写 3、再挂；zfs 是 export 再 import），大文件顺序读 R1：同一个文件 `--rw=read --bs=1M --size=4G --ioengine=libaio --iodepth=16 --direct=1`。
5. 重挂，4K 随机读 R4：同一个文件 `--rw=randread --bs=4k --size=4G --ioengine=libaio --iodepth=32 --direct=1 --time_based --runtime=20`。
6. 对照：同一个文件的前 256 MiB 不带 direct 连读两遍（第六节）。
7. 4K 随机写 W4：同一个文件 `--rw=randwrite --bs=4k --size=4G --ioengine=libaio --iodepth=32 --direct=1 --time_based --runtime=20 --end_fsync=1`。
8. 元数据 M：一个空目录里 `fio --ioengine=filecreate --nrfiles=10000 --filesize=4k --openfiles=1 --file_service_type=sequential --create_on_open=1` 建 1 万个文件，
   再用同样参数的 `--ioengine=filestat`、`--ioengine=filedelete` 各走一遍；另一个空目录里小文件写
   `--ioengine=psync --rw=write --bs=4k --filesize=4k --nrfiles=2000 --openfiles=1 --file_service_type=sequential --fsync=1 --create_on_open=1`。
9. 卸载、清缓存、计时再挂一次：冷挂载 C。

裸盘：只跑 3、4、5、6、7，fio 直接打 `/dev/vda`（`--size=4G`，从偏移 0 起），不重挂。

singlefs：把 `first_transaction_on_device /dev/vda /dev/vdb direct` 当子进程跑，逐行记它每条结果行到达的时刻：

- 从起跑到 `name=transaction` 那一行 = mkfs + 取号 + 暖机 + 第一个事务的挂钟，记成 P 的上界（拆不开：这个二进制不报分段时间）；
- 从那一行到 `name=recover_cold` 那一行 = 冷恢复的挂钟，记成 C；
- 整个子进程前后两块盘的块层读字节，记成冷恢复读的量的上界（写路径上只有取号读超级块与根环）；
- P 的写请求数、写字节、屏障与 FUA 取录制流（门禁 55 号已证它与设备侧日志逐项相等），不另量。

fio 公共参数：`--thread --output-format=terse --terse-version=3`，其余取 fio 默认（`randrepeat=1`，每家同一串随机偏移）。

## 五、每一维报什么

| 维 | 报什么 |
|---|---|
| W1 / R1 | fio 带宽（MiB/s） |
| R4 / W4 | fio 每秒操作数、完成延迟 p99（µs） |
| M | 建 / stat / 删各自的每秒操作数；小文件写 + fsync 的每秒文件数 |
| P | 每次延迟的中位与 p99（µs）；每次的块层写请求数、写字节、FLUSH 数（100 次的平均，另报第一次单独的一份） |
| C | 挂载挂钟（ms）、挂载期间块层读字节与读请求数 |
| F | 格式化挂钟（ms）、格式化期间块层写字节与 FLUSH 数；挂载后 df 的总容量与可用字节占盘大小的比例 |

没量、只列在对比文件里的维：老化后的性能、满盘与 ENOSPC、并发扩展、损坏检出与自愈、崩溃后恢复时间。

## 六、校验路径与它自己的自证

每个 fio 作业、每次挂载、格式化的前后各读一次来宾 `/sys/block/vda/stat`（下标 0 读请求、2 读扇区、4 写请求、6 写扇区、15 FLUSH 请求）。

- 带 direct 的读作业（R1、R4）：块层读字节 < 0.9 × fio 报的读字节 ⇒ 那一格记「被缓存顶替」，不进中位。
  会让它触发的观测：某家把 O_DIRECT 读交给自家缓存接住（最可能是 ZFS 的 ARC）。
- 带 direct 的写作业（W1、W4）：块层写字节 < 0.9 × fio 报的写字节 ⇒ 那一格记「写被吸收」，不进中位。
  会让它触发的观测：某家把 O_DIRECT 写降级成缓冲写，重复覆盖的块在内存里合并掉了。
- 自证：每一轮第 6 步的第二遍读，块层读字节必然接近 0，同一个判定函数必须把它判成「被缓存顶替」；判不出来，这一轮所有读格作废。

## 七、轮数与稳定性

- 每个配置 5 轮，每轮一个新虚机、一块新盘；八个配置轮流跑（第 1 轮八个全跑完再跑第 2 轮），宿主上的漂移平摊到每一家。
- 报中位，另报最小、最大与离散 = (最大 − 最小) ÷ 中位；离散 ≤ 15% 记「稳定」，> 15% 记「不稳定」，照报不删。
- 每轮开跑前记宿主 1 分钟负载（同一台机器上可能有别的会话在跑）；负载高的那一轮照跑照记、不删，汇总时逐轮列出负载。

## 八、作废条款（每条后面写会让它触发的观测）

1. vm-bench 的条数闸不过或退出码非 0 ⇒ 那一轮作废，同参数重跑一次；再不过 ⇒ 那一格记「跑不起来」并附来宾 dmesg 末尾。
   触发：某家格式化或挂载报错、fio 崩溃、控制台吞行。
2. fio 作业的 error 字段非 0 ⇒ 那一格作废。触发：某家不支持 O_DIRECT，或 libaio 返回 EINVAL。
3. 第 6 步的第二遍没判成「被缓存顶替」⇒ 那一轮所有读格作废。触发：判定函数的阈值或块层字段下标取错。
4. 某家格式化或挂载失败 ⇒ 那一家记「这一轮测不了」，附 dmesg 末尾；不改参数重试，改参数就是新一轮、另写登记。
   触发：bcachefs-tools 1.3.3 格式化的盘 6.17 内核挂不上；zpool 在没有 udev 的 initramfs 里等不到分区节点。

## 九、它答不了的

1. 虚拟盘上的相对位置，不是真 NVMe 上的绝对性能。
2. 六家单盘、singlefs 两盘；六家要不要也跑两盘镜像，收尾交用户定（预想：先单盘）。
3. 参数全取默认，一种尺寸（4 GiB 文件、1 万个文件、4 KiB 随机块），不老化、不并发（每个 fio 作业一个线程）。
4. singlefs 在第一个事务上只有 P、C、F 三维能跑，而且 P 只量得到上界。
5. 结论只到「差多少」，不到「为什么差」「该怎么改」。

## 十、跑产物之前的修订（留痕）

### 修订一（2026-09-15 JST，E152 二进制还没写、一格产物都还没跑）

- 改了什么：第四节第 8 步的 `filestat`、`filedelete` 两遍去掉 `--create_on_open=1`，其余参数不变；`filecreate` 保留它。
- 依据：宿主上 fio 3.36 的工具探针（scratchpad 里 20 个空文件，不是任何一家在测试台上的数）：带 `--create_on_open=1` 时 `filestat` 与 `filedelete` 都报 error=2、零次操作，`filedelete` 之后 20 个文件一个没少；不带它时两个引擎都走完 20 个文件、error=0，`filedelete` 之后目录空了。
- 为什么不算看了数再改：这是工具参数写错，照原样跑，第八节第 2 条会让这两格在每一家上都作废；改的只是作业跑不跑得起来，判据、阈值与作废条款一个字没动。

### 修订二（2026-09-15 JST，E152 二进制还没写、一格产物都还没跑）：撤回修订一

- 修订一的依据是一个同时换了两个变量的探针：那一次除了给 `filestat` / `filedelete` 加上 `--create_on_open=1`，作业名也从三个作业共用的一个名字换成了各自的引擎名；
  fio 默认按作业名给文件起名（`<作业名>.<作业号>.<文件号>`），stat 与 delete 找的是另一批从没建过的文件，error=2 是找不到文件，不是 `--create_on_open=1` 造成的。
- 把两个变量分开重做（scratchpad 的 fioprobe4，20 个空文件）：三个作业名字不同、都加 `--filename_format=metadata.$filenum` 时，带与不带 `--create_on_open=1` 两组都是 error=0、`filedelete` 之后目录空了；
  三个作业同名、都带 `--create_on_open=1`、不设 `--filename_format` 时也是 error=0。
- 所以撤回修订一：第 8 步三个元数据作业照第四节原文都带 `--create_on_open=1`；另给三个作业都加 `--filename_format=metadata.$filenum`，让 stat 与 delete 作用在 create 建的那 1 万个文件上
  （第四节写的「stat 它们、删它们」本来就是这个意思，原文漏了让它成立的那个参数）。判据、阈值与作废条款仍一个字没动。

### 修订三（2026-09-15 JST，冒烟跑之后、正式跑之前）：测试台的两处毛病

- 冒烟跑：八个配置各 1 轮（scratchpad 的 `e152-smoke.out`），只用来找测试台的毛病；它的数不进结论，也没拿来比。
- 找出两处，都是搭建与收集的毛病，不碰负载、判据、阈值与作废条款：
  1. 附加根里的库按 0644 装，动态加载器 `/lib64/ld-linux-x86-64.so.2` 没有执行位，来宾里每个动态程序（fio、各家格式化工具、zpool）一执行就 Permission denied；
     七个配置两次都卡在第一个外部命令（产物里的 `name=failure` 行是 `stage=fio_start` 或 `stage=format`，detail 末尾都是 `Permission_denied_(os_error_13)`）。
     singlefs 那个二进制是静态链接的，不走加载器，这一配置跑通了。
     改法：库一律按 0755 装；搭建脚本加一道自检，用搭好的加载器与库实际跑一次搭好的 fio。
     自检的判别力：拿修之前那份搭建目录照做，加载器没有执行位、直接执行报 Permission denied；修之后重搭，自检通过。
  2. 驱动脚本用不锚行首的 grep 收结果行，失败时 vm-bench.sh 缩进着重打的控制台末尾也被收进产物，每行重复一次；改成只收行首的 `E7RESULT` 行。

### 修订四（2026-09-15 JST，第二次冒烟跑之后、正式跑之前）：对照的两遍读关掉 fio 的 invalidate

- 第二次冒烟跑（scratchpad 的 `e152-smoke-2.out`）里先跑完的五个配置，对照那一行全是 `control_has_teeth=false`：第二遍不带 direct 的读，块层照样读满，判定函数判「算数」。
- 原因不是第八节第 3 条写的两种（判定函数的阈值、块层字段下标），在 fio：`invalidate` 默认是 1（`fio --cmdhelp=invalidate` 逐字 `default: 1`），
  每个作业开始前先对文件发一次 `posix_fadvise(DONTNEED)` 丢掉页缓存；宿主上 strace 一个读作业，默认时 1 次 `POSIX_FADV_DONTNEED`，加 `--invalidate=0` 后 0 次。
  第二遍从来读不到缓存，对照没测到它要测的那种情形。
- 改法：第 6 步的两遍读都加 `--invalidate=0`，别的作业不动。判据与阈值不动：对照要的本来就是「第二遍读的是缓存」，原文漏了让它成立的那个参数。
  第八节第 3 条照原样执行——冒烟跑那几轮的读格按它都作废，而冒烟跑的数本来就不进结论。
- 装置加一条单测钉住对照两遍都带 `--invalidate=0`，变异表加一条去掉它的变异。

### 修订五（2026-09-15 JST，第二次冒烟跑之后、正式跑之前）：zfs 的 vdev 换成事先分好的一个分区

- 第二次冒烟跑里 zfs 两次都在格式化那一步失败，`name=failure` 行的 detail 原样是
  `cannot_label_'vda':_failed_to_detect_device_partitions_on_'/dev/vda1':_19_Error_preparing/labeling_disk.`——正是第八节第 4 条写下的那种触发。
- 原因：给 zpool 一整块盘时，它自己打 GPT 分区，再等 udev 报告分区就绪；initramfs 里没有 udev，等不到（`/dev` 挂的是 devtmpfs，分区节点其实在）。
- 改法（只改 zfs 一家）：格式化那一步先用 sfdisk 在盘上建一张 GPT、一个从 1 MiB（第 2048 扇区）起到盘尾的分区，再 `zpool create -f -m /mnt bench /dev/vda1`；
  给的是分区，zpool 不再自己打标签，也就不等 udev。计时照旧从这一步开头算到 zpool 返回（分区本来就是 zpool 自己做的那一半）。
- 与 zpool 整盘默认布局的差别：zpool 自己打标签时数据分区同样从 1 MiB 起，另在盘尾留一个 8 MiB 的第 9 分区；这里没有第 9 分区。
  数据区的起点与对齐一样；对比文件里 zfs 那一列要带一句「vdev 是分区不是整盘」。
- 第八节第 4 条的「不改参数重试」管的是正式跑；冒烟跑找出的毛病在正式跑之前改，按修订留痕。sfdisk 取本机 util-linux 的那个，装进附加根。

### 修订六（2026-09-15 JST，第三次冒烟跑途中、正式跑之前）：裸盘的对照，两遍之间由 E152 一直开着目标

- 第三次冒烟跑（带修订四的 `--invalidate=0`，scratchpad 的 `e152-smoke-3.out`）里 ext4、xfs、f2fs 的对照都成了 `control_has_teeth=true`，
  裸盘仍是 `second_pass_verdict=counted control_has_teeth=false`。
- 原因：裸盘上两遍读是两个 fio 进程各自打开、关闭 `/dev/vda`，块设备最后一个打开者关闭时内核丢掉它的页缓存。
  本机内核源码树 `/home/fy5090/kbuild/linux-om`（7.2.0）的 `block/bdev.c` 第 763–764 行：`atomic_dec_and_test(&bdev->bd_openers)` 成立就调 `blkdev_flush_mapping`，
  它在第 757 行 `kill_bdev(bdev)`。虚机跑的是 6.17，6.17 的源码没逐行核，冒烟跑的现象与这个读法一致。文件系统上的文件不走这一步，所以六家不受影响。
- 改法：第 6 步两遍读之前由 E152 自己以只读打开目标、第二遍读完再关，块设备走不到「最后一个关闭」；文件上同样这么做，对它们没有影响。判据与阈值不动。
- 这一条改动没有单测能盯住（要真块设备），它的证据是下一次冒烟跑里裸盘的对照翻成 `true`；变异表不加条目。

### 第四次冒烟跑与正式跑的起点

- 第四次冒烟跑（裸盘、zfs 各 1 轮，scratchpad 的 `e152-smoke-4.out`）：裸盘 `second_pass_verdict=cache_substituted control_has_teeth=true`，修订六的读法坐实；
  zfs 格式化建池成功，十个作业 error 都是 0，O_DIRECT 读写两格都判「算数」，对照有牙，修订五坐实。另外五家的对照在第三次冒烟跑里已经有牙（修订四）。
- 四次冒烟跑的数都不进结论。正式跑用修订一到六之后的装置从头跑 5 轮，产物 `research/results/e152-file-system-benchmark-2026-09-15.out`，起跑时刻记在产物首行。

## 十一、第二次正式跑：六家也跑两盘镜像（2026-09-15 JST，第一次正式跑之后、装置改动之前写）

用户 2026-09-15 看过第一次正式跑之后定：六家也跑两盘镜像，与 singlefs 同配置再比。
第六、七、八节的判据、阈值、作废条款、轮数与稳定性照旧，这一节只写变了什么。跑之前不预设镜像下谁快谁慢。

### 配置（每个配置两块 16 GiB virtio 盘；除镜像本身之外参数一律默认）

| 配置 | 镜像怎么做 | 格式化 | 挂载 |
|---|---|---|---|
| raw-md（天花板） | md raid1：`mdadm --create /dev/md0 --level=1 --raid-devices=2 --metadata=1.2 --bitmap=none --assume-clean --run /dev/vda /dev/vdb` | 不格式化，fio 直接打 `/dev/md0` | 不挂载 |
| ext4-md | 同 raw-md | `mkfs.ext4 -F -q /dev/md0` | `-o noinit_itable /dev/md0` |
| xfs-md | 同 raw-md | `mkfs.xfs -f -q /dev/md0` | 默认 |
| f2fs-md | 同 raw-md | `mkfs.f2fs -f -q /dev/md0` | 默认 |
| btrfs-raid1 | 文件系统自己的镜像 | `mkfs.btrfs -f -q -d raid1 -m raid1 /dev/vda /dev/vdb` | `-o device=/dev/vda,device=/dev/vdb /dev/vda` |
| bcachefs-replicas2 | 文件系统自己的镜像 | `bcachefs format -f -q --replicas=2 /dev/vda /dev/vdb` | `/dev/vda:/dev/vdb` |
| zfs-mirror | 文件系统自己的镜像 | 两块盘各用 sfdisk 分一个从 1 MiB 起的分区（修订五），`zpool create -f -m /mnt bench mirror /dev/vda1 /dev/vdb1` | export 再 import |
| singlefs | 它自己的两盘镜像（D2（RAID 条带策略） 已定项 9） | 同第一次正式跑 | 同第一次正式跑 |

- `--assume-clean`：两块盘都是新 fallocate 出来的全零镜像，本来就一致，不让 md 在量的时候后台做一次 16 GiB 的初始同步；
  `--bitmap=none` 与 mdadm 在这个尺寸上的默认一致，写死免得随版本变。mdadm 取本机 4.3，装进附加根，运行时设 `MDADM_NO_UDEV=1`（initramfs 里没有 udev）。
  md 核心编进了内核，raid1 是模块（`raid1.ko`，没有依赖）。

### 变了的口径

- **块层计数改成所有成员盘相加**：镜像下读请求会分到两块盘上，只看一块盘，第六节的校验会把每一家的读都误判成「被缓存顶替」；
  相加之后，读应当约等于 fio 的读量，写约等于两倍。单盘配置只有一块盘，相加等于不变，第一次正式跑的判定不受影响。
  md 配置只数两块成员盘，不数 `/dev/md0`（数了就重复）。小文件持久化、冷挂载、格式化的块层读数同样两盘相加，与 singlefs 的两盘合计同口径。
- 格式化的计时从建阵列（或分区）开始，到格式化命令返回为止；可用容量的分母换成两块盘的总字节，与 singlefs 的「两盘镜像后是总量的 47.6%」同口径。
- 冷挂载：md 配置只卸载、再挂文件系统，阵列不拆，所以阵列组装的时间不在里面；zfs 是 export 再 import，btrfs、bcachefs 挂载时自己组装两块盘。

### 多一条作废条款

- md 配置每轮在建完阵列与整轮结束时各读一次 `/proc/mdstat`，任何一次看到 `resync` 或 `recovery` ⇒ 那一轮作废，按第八节第 1 条同参数重跑一次。
  触发：`--assume-clean` 没生效，或者某块成员盘在跑的时候掉了。

### 次序

- 先冒烟：七个镜像配置各 1 轮，只找测试台的毛病，数不进结论；改了什么照第十节的写法追加修订。
- 再正式跑：八个配置按轮交错跑 5 轮，产物 `research/results/e152-file-system-benchmark-mirror-2026-09-15.out`。

### 镜像冒烟跑与第二次正式跑的起点

- 冒烟跑（七个镜像配置各 1 轮，scratchpad 的 `e152-mirror-smoke-1.out`）：七个都是第一次就跑通，没有 failure 行；对照 7 / 7 有牙；
  四个 md 配置两次读 `/proc/mdstat` 都是 `synchronizing=false`；带校验的作业没有一个被判「被缓存顶替」或「写被吸收」，fio 都没报错。没有要改的，这一节不加修订。
- 冒烟跑的数不进结论。第二次正式跑用这时的装置从头跑，起跑时刻记在产物首行。

### 第二次正式跑途中：第 1 轮 ext4-md 第一次跑失败（按第八节第 1 条重跑）

- 现象：`vm_exit=2`，那一次的输出只有一句 `scripts/vm-bench.sh: line 270: unexpected EOF while looking for matching` 加一个单引号。
- 原因：别的会话在正式跑进行中改了 `research/scripts/vm-bench.sh`（修改时间 UTC 2026-09-14 18:13:21，即东京 03:13:21）：开头三行注释换成了两行，文件变短。
  bash 按字节偏移边读边执行脚本，那一次正在跑的 vm-bench.sh 读到错位的内容，报语法错退出。改完之后 `bash -n` 通过；和 HEAD 比，那处改动只动注释，E152 加的几块原样在，测试台的行为没变。
- 处置：照第八节第 1 条同参数重跑一次。这次失败留在产物里（`E152RUN … attempt=1 … vm_exit=2`），汇总只取成功的那一次。
- 欠的防护：驱动脚本起跑时应把 vm-bench.sh 等脚本复制一份、之后只调副本，免得共写的仓里别人中途一改就把正在跑的实验弄坏；正在跑的就是这个驱动脚本，现在改它会把它自己弄坏，所以跑完再加。
