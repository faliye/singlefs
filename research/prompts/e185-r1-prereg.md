# E185 第 1 次重跑登记：GC一批搬迁的挂钟拆分（七·6 合入之后）

写于 2026-10-22，重跑之前（装置不改、这一趟一份产物都还没跑）。差分重跑登记：只写变了哪些输入、哪几行要重判、跑哪几段与怎么跟上一趟比；判据、阈值、作废条款、停机条款、臂、阳性对照、取样点、轮数照原登记 `research/prompts/e185-preregistration.md`（含第十二节 12.1–12.11 修订）原样沿用，一个字不放宽。原登记读的那一刻 sha256 `473493a455034a6f96c0c276e712fce3e0d423001040d5ba0360e0b46ccb8198`，721 行；下文写「同上一次登记」的节都指这一份。

## 一、问题

英文名：gc_batch_wall_clock_split

（照抄原登记第一节第 7 行；装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` 不另起，复跑行不动。）

**主 agent 给的问题**（派发提示「关的是哪几条」逐字）：

```markdown
E185（GC一批搬迁的挂钟拆分） 第 1 次重跑，写重跑登记 `research/prompts/e185-r1-prereg.md`（节名照你定义「登记的固定节名」）。实验号不占新号；装置不改，判据、阈值、作废条款照 `research/prompts/e185-preregistration.md`（含第十二节 12.1–12.11 修订）原样沿用，一个字不放宽。

1. 写明这次重跑为什么跑、改了什么输入：被测的 GC 路径在七·6（发布路径按身份查）三批补丁与后续批合入之后变了（`crates/singlefs-core/src/transaction.rs`，主工作区现 sha256 前缀 `3b81509b`；七·6 那一行验收写「改完重跑 E185（GC一批搬迁的挂钟拆分） 第二段」）。上一趟两段产物 `research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out`、`research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out` 是七·6 合入之前的基线。实验页 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` 现状写问题单第 1、2 行等七·6 合入之后两段重跑再判、第 4 行单跑那一半已判「单跑够」、第三段没跑。验收：登记里写明跑哪几段（至少第一段 wall-clock 与第二段 stage-split）、各回答问题单哪几行、产物文件名（带日期，不盖上一趟）。
2. 第三段（gpu-submission，问题单第 3 行）跑不跑、在什么条件下跑：照登记第五节分段表与问题单第 3 行的够判条件写成条件句（例如第一、二两段交回之后哪一格成立就跑第三段），由主 agent 按交回的产物判。验收：条件可由产物里的哪几行判出写明。
3. 与上一趟比的口径：重跑的数与 2026-10-20 那一趟并列报，不改判据；写明哪些行能拿来比七·6 前后（一批 CPU 随池大小的增长，七·6 那一行验收的「从平方降到接近线性」），比法跑前写死，并写一句「什么观测会让它不成立」。验收：比法每一格写清读产物的哪一行、哪个字段。
```

**问题单逐行抄**（`research/prompts/m4-gpugc-measure-forks.md` 第 5–12 行，原样；文件 sha256 `78eaec57b0a7f6fc2bbe3435458385ba84ef7beae9dce8d423d32e5454d55c8e`，四行状态此刻都是「开着」）：

```markdown
被测的是 `crates/` 今天的 GC 路径：`crates/singlefs-core/src/mounted_session.rs` 的 `run_compaction`（区域扫描 `region_scan.rs` → 选段与计划 `compaction_plan.rs` → 发布 `transaction.rs` 的 `publish_relocation_batch`）。

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | 一批搬迁的挂钟受谁限：盘 ／ CPU | 一批里 CPU 时间与等盘读写时间的比；同一批在内存设备与真盘（直接 I/O）上的挂钟差 | 至少两种批大小、两种段内活槽占比，在内存设备与真盘上各量到一批的挂钟与按阶段拆开的 CPU 时间 | 开着 |
| 2 | GPU 上算 GC 的哪几段：甲 只算校验和核对（搬之前核源单元）／ 乙 甲加区域扫描里的单元头解析与映射反查 ／ 丙 乙加选段打分（`choose_region` 对全部候选段） | 每一段在一批里的 CPU 时间占比与它处理的字节数（得出每段的 CPU 吞吐） | 第 1 行够判的那组取样点上，三段各自的 CPU 时间与字节数都量到 | 开着 |
| 3 | 攒批形态：甲 一批一交 ／ 乙 连续几批攒起来一次交，几批由显存上限定 ／ 丙 放宽一条意图的 R 到不止一个聚簇段 | 一批交给 GPU 的计算字节数；GPU 一次提交的固定开销与每字节开销 | 一批的计算字节数按批大小量到；GPU 一侧的两样开销量到（本机 RTX 5090 / 5060 Ti，经 `scripts/crash/gpu-quiesce-run.sh` 起） | 开着 |
| 4 | 定时兜底间隔与写带宽上限的默认值从什么量取：按一批在真盘上的写字节与挂钟 ／ 另跑前台负载量 GC 对前台 fsync 延迟的影响 | GC 单跑的写带宽；GC 与前台 fsync 并发时前台延迟的变化 | 第 1 行的真盘取样点上写带宽量到；要不要量并发那一项，设计员判（附带，够判后不跑） | 开着 |
```

**变了哪些输入**（差分重跑的第一样）：

| 输入 | 上一趟（2026-10-20） | 这一趟 | 改动一句话 |
|---|---|---|---|
| 被测的核心层 `crates/singlefs-core/src/` | 七·6 合入之前那棵树（产物不记源码哈希；批次记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 70 行记第一段跑完，第 72 行起才是七·6 实现员交回与合入） | 起跑那一刻的树；主工作区此刻 `transaction.rs` sha256 `3b81509b74d72a6f0e4b764aa9027d7d301033186b5a7f96f51472821dffeac2`、`mounted_session.rs` `d31503616b7c6fec47b189b4c424ebc039621078510af299c3d4cf121fa177c8` | 七·6 三批与后续批把发布与整理读版本的按身份查从 `iter().find` 换成按身份建一次的索引（第三节）；两次之间还合了别的批（同一份记录第 82–83 行：里程碑三的会话打了三份改 `transaction.rs` 的补丁），所以前后差别只归到「两棵树之间」，不单归七·6 |
| 装置 | `b3a50d50441f9f2e74bec6c941b74e632529688a78563995387708230b60c470`（批次记录第 64、70 行「日志首行 `b3a50d50…`」） | 同一份：主工作区此刻 sha256 `b3a50d50441f9f2e74bec6c941b74e632529688a78563995387708230b60c470` | 不改 |
| 判据、阈值、作废与停机条款、取样点、轮数 | 原登记 | 原登记 | 不改 |
| 被测条款 D26 已定项 13 | 原登记第二节抄的是六条、参数名 `gc_compute` 一族 | 今天是八条、参数名 `background_compute` 一族，多了第 7 条（值不值按省下的 CPU 时间算，D24 已定项 3 按挂钟比的判据不管它罩着的后台任务）与第 8 条（两份独立 GPU 核各算一次比对） | 条款正文变了（第二节整段抄今天的）；对 Q9 的影响见第四节末与第六节，判据不改、交主 agent |

**要重判的岔路行**（差分重跑的第二样）：第 1 行（Q1–Q4）、第 2 行（Q5–Q8，连同第三段跑了之后的 Q9）照原判据在新树上重判；第 3 行（Q10–Q13）在第三段跑的时候判；第 4 行单跑那一半（Q14–Q16）由第一段照常重量、照原判据并列报，判定与上一趟相反时第 4 行重开（第五节「第四段」条件）。另有一格不是岔路单的行：七·6 验收「一批 CPU 随池大小的增长从平方降到接近线性」的比法（第六节 C 组，附带，只从两段产物算、不另跑）。

**读法写死**：原登记第一节「读法写死」那张表整张沿用，只补下面三格：

| 词 | 口径 |
|---|---|
| 真盘 | 原登记写本机那块 NVMe；2026-10-20 那一趟两段都在第二台跑（批次记录第 53、63–66、70 行）。这一趟为与那一趟并列比，两段照旧在第二台、同一个镜像目录所在的盘上跑；第 1 行真盘臂的结论只说「第二台那块盘上 ext4 的 `O_DIRECT` 镜像文件」，盘的型号与逻辑块大小记进输入清单（第五节）。待主 agent 认：另一种是两段都在本机跑，那样真盘臂与内存臂的 CPU 都不跟 2026-10-20 那一趟比（第六节 C 组整组不算） |
| 七·6 前、七·6 后 | 前 = 2026-10-20 两份产物（sha256 见第五节）；后 = 这一趟两份产物。「前后差」只说两棵树之间，不单归七·6 |
| 一批的 CPU（比法用） | 第二段 `name=stage` 行的 `call_cpu_nanoseconds`，与第一段 `name=call` 行的 `cpu_nanoseconds`（`arm=memory`、`injection=none`），两条来源各算各的；取法见第六节 C1 |

## 二、被测条款与它引的定义

四段，`scripts/kb/quote-kb.py` 整段抄（出口 `/home/fy5090/.local/share/singlefs/work/m4-e185-r1-designer/clauses.md`，回读逐字节一致，再整份追加到这里）：里程碑四作业点表七·1、七·6 两行（派发点名），D26 已定项 13（派发点名，今天的正文，与原登记第二节抄的那一版不同），D24 已定项 3（派发点名，与原登记第二节第 108–121 行逐字相同，`diff` 核过）。原登记第二节另抄的 D26 已定项 1、D24 已定项 2、C412 这一次没重抄，照原登记第二节。

**出处 `.claude/kb/milestone/04-fourth-txn.md:141-141`（整段抄，未转述）**

```markdown
| 七·1 | 量：在 `crates/` 现有的 GC 路径上量一批搬迁的挂钟，拆成读写 I/O 与 CPU（解单元头、核校验和、查映射、分配记录树写放大）各几成；口径写明设备、负载、批大小（C412（整理的挂钟代价没量过） 的检查列） | C412（整理的挂钟代价没量过）；D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式 | 断言钉绝对值；批大小翻倍挂钟跟着涨（判别力自证） | 二·2 的发布路径 |
```

**出处 `.claude/kb/milestone/04-fourth-txn.md:142-142`（整段抄，未转述）**

```markdown
| 七·6 | 发布路径按身份查：`TransactionOutput::unit`、整理读版本时逐 key 查中央映射（`version_as_the_compaction_reads_it`）、发布照抄数据单元（`carried_unit`）、分配记录树结算里的 `mapping_key_carried_from`、`file_other_than_the_first` 五处由在 Vec 上 `iter().find` 改成按身份索引，行为不变（用户 2026-10-20 定「先修四处线性查找」）；另加两处同样改：整理发批核搬迁源落点 `relocation_checked_against_the_version` 按 key 线性找角色、`mapping_locations_of_a_mapped_unit` 线性找 key（三个调用点，含快照判定那一圈进 deadlist 那一支）（用户 2026-10-21 定并进，`research/prompts/m4-publish-lookup-code-r2-main-verification.md` 第五节）；增长判据与变异数产品调用点上的次数，不数测试自己另建的索引 | `.claude/rules/fs-design.md`「记账是事务的副产品」；七·1 第二段的发现（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 2026-10-20 调查员交回那一条） | 改前改后同一串操作写出的盘镜像逐字节相同；每处改法一条会红的变异；一批 CPU 随池大小的增长从平方降到接近线性（副本推翻实验：P = 4096 一批 217.47 → 62.38 ms），改完重跑 E185（GC一批搬迁的挂钟拆分） 第二段 | 七·1 第二段 |
```

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:232-252`（整段抄，未转述）**

```markdown
#### 已定项 13：后台任务的计算放在哪、怎么开、什么节奏

**定案**：八条。

1. **计算放在哪**：默认 CPU。挂载参数 `background_compute`（取 `cpu` 或 `gpu`，默认 `cpu`；命令行有了之后写成 `--background-compute=gpu`）选 GPU。这一组挂载参数（第 1、2、3 条）只有一份，管全部后台任务，不按任务分设：开了 `gpu`，凡有 GPU 路径的后台任务（GC 搬迁、打包搬迁，以后接上 GPU 路径的别的后台任务）都走 GPU，没有 GPU 路径的照旧在 CPU 上算。它只活在内存里：不写进系统配置、不占 feature bit（D24（后台重活能不能卸给 GPU） 已定项 2 第 1 条），每次挂载各给各的。挂着的时候可以经已挂载会话的控制接口切换（一次切全部后台任务），从下一批开始生效；一批之内不换（`.claude/rules/fs-design.md`「分支变量不许在操作中途改变」）。
2. **用哪几张卡**：挂载参数 `background_gpu_devices` 是一张卡的列表（可以点几张）；不给就用本机全部显卡。
3. **显存上限**：挂载参数 `background_gpu_memory_limit` 是一个带单位的字节数列表，按次序一张卡一个，与参与的卡逐个对应（给了 `background_gpu_devices` 就对它的列表，没给就对本机全部显卡的列表），不补齐：`background_gpu_devices` 点了三张卡，`background_gpu_memory_limit` 要么不给、要么正好给三个数，只给一个也报错；GPU 模式下全部后台任务在那张卡上合占的显存不超过它的那一个数：这个数是上限、不是预留，用得比它少没有关系（卡上此刻空闲的不到这个数也照常跑，只用空闲的那些），任何时刻都不许比它多，攒批、缓冲、暂存区一起算在里面。不给就不设上限，后台任务跑的时候卡上能用多少用多少。GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管。
4. **配错与 GPU 用不上时报错，不降级**：挂载或切换那一刻，`background_gpu_memory_limit` 给了而个数与参与的卡数不等、某一个数超过那张卡的显存总量、GPU 不在、驱动不对、点名的卡不存在，就拒绝挂载（或拒绝切换）并报错；挂着的时候 GPU 出错，正在算的那一批放弃（意图留在盘上，目标态单调，已定项 2），那个后台任务停下并报错，不换回 CPU；恢复要人切回 `cpu` 或修好 GPU 再切。唯一的例外在第 8 条：连续几轮状态不确定的条目按条交 CPU 判，后台任务照旧在 GPU 上跑。
5. **什么时候起跑**：每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数。两种模式起跑规则相同。
6. **起跑之后的节奏**：GC 占盘写带宽有一个上限（挂载参数）；前台有 fsync 在等时不发下一批。
7. **值不值按省下的 CPU 时间算**：GPU 模式的目的是少占主机 CPU 时间，数据量大时跑得更快只是副产物。收益是后台任务少占的主机 CPU 时间（CPU 模式下它的 CPU 时间减 GPU 模式下它的 CPU 时间，后者含准备数据、提交、等结果、两份结果的比对，以及第 8 条交 CPU 判的那几条），挂钟与 CPU 模式持平也做；D24（后台重活能不能卸给 GPU） 已定项 3 那条按挂钟比的判据不管这一项罩着的后台任务。实现之后跑一组对照：同一负载、同一池，CPU 模式与 GPU 模式各跑，报两边的 GC 挂钟、主机 CPU 时间、前台 fsync 延迟，并在几档盘容量上各跑一遍，看收益随容量怎么变。
8. **GPU 的结果怎么复核**：两份独立写的 GPU 核（不共用核函数）对同一批输入各算一次，逐条比两份的结果。两次相同的条目算确认，交 CPU 改位图（CPU 只改位图，不再抽样复核）；两次不同的条目写明状态不确定，这一批不回收、不搬，留到下一轮重算。同一个条目连续 N 轮都不确定，交 CPU 那一份实现单独判一次，按 CPU 的判定处理。每一批报确认、不确定、交 CPU 判的条目各多少，运行时看得出来。只有一张卡时两份核都在这张卡上跑，有几张卡时可以分开跑；不许要求多张卡，一张与几张都要能跑、判定相同。N 与几张卡时怎么分在里程碑四设计轮（七·2）定。

GPU 路径与 CPU 路径是两份实现（D24（后台重活能不能卸给 GPU） 已定项 2 第 3 条）：运行时的复核照第 8 条由两份 GPU 核各算一次比对，CPU 那一份管连续不确定的条目，两份实现在测试里对拍。GPU 路径默认写成 WGSL、经 `wgpu` 起（checker 档的崩溃判器已经这样用，`Cargo.lock` 里是 `wgpu 30.0.1`，不用另装 CUDA 工具链）；有更好的做法可以在设计轮比过之后换。改新池新建文件的字节：否——八条全是挂载参数与后台行为，不写进盘。

**射程**：管开关、选卡、显存、出错处理、起跑与节奏的形式。第 1 到 4 条与第 7、8 条管全部有 GPU 路径的后台任务，今天是 GC 搬迁与打包搬迁；第 5、6 条只管 GC，打包的起跑与限速在 D27（小数据打包容器） 已定项 6。D24（后台重活能不能卸给 GPU） 的三个候选仍暂缓（D24（后台重活能不能卸给 GPU） 已定项 1），重开并接上 GPU 路径时进同一组开关，值不值照哪条判据重开时定。不管：GPU 上算 GC 的哪几段（里程碑四设计轮定）、算打包的哪几段（里程碑七设计轮定）；定时兜底的间隔、写带宽上限的默认值（等 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理））。⚠️ 第 4 条运行中出错就停 GC：停着的时候空间只减不增，与 D3（空间分配） 已定项 14「后台整理常驻、自动」并存的读法是这只在用户显式开了 GPU 时才会发生，默认 `cpu` 不受影响。⚠️ 第 8 条的两份核抓得到只有一份出的错（一份核写错、没有 ECC 时的位翻转、不确定的竞争），抓不到两份一起错的：两份共用的那几样（同一个驱动与着色器编译器、同一份输入表的准备、对规则的同一种误读）出错时两份照样一致，那一类靠 GPU 与 CPU 两份实现在测试里的对拍和崩溃点重放抓。

**依据**：无实验：这一项是开关与运行政策的用户定案，数值不在这里（等 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理））。用户定案（2026-10-19 弹窗四问与追加的几句；2026-10-20 参数统一管全部后台任务、收益口径扩到全部后台任务；2026-10-21 两份 GPU 核各算一次比对、不确定的留到下一轮、连续不确定的交 CPU 判），原话在变更史。

**欠**：C412（整理的挂钟代价没量过）。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:47-59`（整段抄，未转述）**

```markdown
#### 已定项 3：判据取什么形式

**定案**：判据是 `CPU 16 线程耗时 ÷ (传输 + 计算 + 传回) 耗时`，小于 1 就否掉那一格。代进去的 CPU 侧数必须是**多核饱和吞吐**，不是「单核 × 核数」：线性外推与实测在 AES-GCM 上差近 4 倍（外推约 61 GB/s，实测饱和 17.4 GB/s）。GPU 侧的两个地板是**单程 56.69 GB/s**（锁页 H2D，≥64 MiB）与**来回 28.5 GB/s**（H2D + D2H，回传全部数据的场景走这一行）；CPU 侧的上限是**内存带宽墙 64.91 GB/s**。⇒ **GPU 只在「CPU 那侧的实际吞吐掉到传输地板以下」时才有机会。**

**射程**：⚠️ **「瓶颈是不是 CPU」不等价于「这个任务有没有硬件指令」**——有 AES-NI 的加密照样在 17.4 GB/s 就饱和，瓶颈仍然在 CPU 侧，**有硬件指令 ≠ 快到 GPU 追不上**。⚠️ **「CPU 够快所以 GPU 无用」这个前提在本工程不成立，不许拿它支撑任何结论**：它来自单核乘核数的线性外推，而多核实测饱和在 17.4 GB/s，差近 4 倍且方向相反——17.4 低于 GPU 的两个地板。代进去之后四个候选格各是什么：scrub（校验和 / MAC）有 AES-NI 时 CPU 侧多核饱和 17.4 GB/s（8 线程，16/32 线程 16.3/16.5；单核 3656.13 MiB/s）⇒ 按判据这一格 GPU 赢，⚠️ **两侧都没坐实，不许当已定结论用**（GPU 侧缺真的 AES-GCM 核函数）；无 AES-NI 的机器上单核 244.74 MiB/s ⇒ 16 核约 4.1 GB/s，与高端 NVMe 阵列同量级，这一档 GPU 可能有用，多核未测、饱和值只会更低；EC 重建无数据（Reed-Solomon over GF(2⁸)，重建要读全部幸存盘并算 parity，计算量随盘数 × 容量线性，这一格计算确实重）；批量压缩无数据（压缩比与吞吐的取舍，GPU 压缩库存在）。这两个数（17.4 与 244.74）来自 E6（加密算法选型），不是 D24（后台重活能不能卸给 GPU） 自己量的。⚠️ **不能读成「GPU 无用」**：计算段那一段的推论只对**带宽受限**的核函数成立，**压缩是算力受限的**，它的计算段可能远不止那一成，那一格仍然开着且仍要等 `nvcc`。⚠️ **还有一层压在上面**：三个候选场景全是盘限的后台活，盘带宽比 64.91 与 28.5 都低得多 ⇒ 整件事若是盘限的，CPU 本来就在空转，卸给 GPU 省不下任何东西——**这一层是推理不是实测**（本工程没有阵列）。⚠️ **一个未验的逃生口**：GPUDirect Storage 让 NVMe 直接 DMA 进显存、绕开主存与 PCIe 那一跳，**它会推翻「GPU 吃数据 56.69 GB/s」这个传输地板**，没测过，不许当它不存在。要定 D24（后台重活能不能卸给 GPU），先要量 EC 重建与批量压缩这两格的 CPU 上界，以及「PCIe 传输 + 核函数 + 传回」这条链路相对纯 CPU 的净收益——**数据要过一遍 PCIe，那笔搬运成本可能吃掉全部计算收益**。**本机可测**：NVIDIA RTX 5090（32 GB）+ RTX 5060 Ti（16 GB），驱动 595.84；`nvcc` 没有装（`command -v nvcc` 零命中，`/usr/local/cuda*/bin/nvcc` 也没有），那是 G24.3（计算段占比） 的前置。

**依据**：

- E21（GPU 卸载的净收益）：三条路径的吞吐（CPU 吃数据 64.91 GB/s、GPU 单程 56.69 GB/s、GPU 来回 28.5 GB/s）把判据的两个阈值钉住——GPU 光把数据搞进去就已经比 CPU 读它慢 13%，而这还没开始算；计算段用带宽受限算子给上界，显存内 1041 GB/s 而端到端（含传输）只有 50.3 GB/s ⇒ 传输占 90.3%、计算段几乎免费，对纯扫描那一格净收益 0.78×，小于 1 ⇒ GPU 输。**决定 GPU 赢不赢的是 CPU 那侧有没有对应的硬件指令，不是 GPU 有多快。**
- 用户定案（判据的正确形式是多核饱和吞吐），原话在变更史。

**欠**：无。

```

## 三、实现今天的样子

现查于 2026-10-22，主工作区，文件加行号（设计员的观测，不是结论）。原登记第三节那张表的事实除下面几格外照旧，行号以今天的为准：

| 一步 | 在哪 | 看到的事实 |
|---|---|---|
| 显式入口 | `crates/singlefs-core/src/mounted_session.rs:1939` | `run_compaction` 还在，装置调的就是它 |
| 整理读版本 | `mounted_session.rs:2790-2796`、`2818` | `central_mapping_as_the_compaction_reads_it` 用 `CentralMappingOfAVersionReadLeafByLeaf::of_the_version` 每片中央映射叶只解一次、每把 key 查一次；`version_as_the_compaction_reads_it` 调它（七·6）。原登记第三节写的「对每个进映射单元在映射树里查一次位置」那一步换了做法 |
| 按身份查的索引 | `crates/singlefs-core/src/transaction.rs:3527-3540`、`3574-3590`、`4069`、`4078`；`4712-4720` | `FirstEntryOfEachIdentity`：走一遍表、按身份排一次序，之后每次查是折半查找，同一身份取最先那一项；`TransactionOutput::unit`、`units_by_role` 经它；文件头注释写「在表上 `iter().find` 是 P 的平方」 |
| 发布 | `transaction.rs:9541`、`10003`、`11787`、`3982`、`4623` | `publish_relocation_batch`、`relocation_checked_against_the_version`、`carried_unit`、`file_other_than_the_first` 都在；查找处经 `lookup_work::entering(LookupSite::…)` 记账（`transaction.rs:4331`、`4628`、`4897`、`4973`、`6118`、`6207`、`9784`、`11975`、`12108`、`14919` 与 `mounted_session.rs:2794`） |
| 七·6 那一行点名的另两处函数名 | `grep -rn 'mapping_locations_of_a_mapped_unit\|mapping_key_carried_from' crates --include=*.rs` | 零命中：今天的代码里已经没有这两个名字（改名或并进了别处），不影响装置（装置不调它们） |
| 查找计数的特性 | `crates/singlefs-core/Cargo.toml:19-22`；`transaction.rs:3426-3440`；`crates/singlefs-harness/Cargo.toml` 的 `[dev-dependencies]` | `lookup-work-counters` 默认关，关着时 `count` 什么都不做；只有 harness 档的 dev 依赖打开它。`cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline` 的输出里核心层只有 `feature "default"`（第十三节原样）：只编装置那个二进制时不开计数 |
| 区域扫描 | `crates/singlefs-core/src/region_scan.rs:204`、`286`、`309-310` | `scan_region` 的映射反查仍是对全部映射条目线性找两遍（R 里至多 64 条记录，每条一遍 O(映射条目数)）；七·6 没改这里，`git status` 里这份文件没有改动 |
| 线程 | `grep -rn 'thread::spawn\|thread::scope\|rayon' crates/singlefs-core/src \| wc -l` | 0：整理整条路仍在调用线程上跑 |
| 两棵树之间的改动量 | `git diff --stat HEAD -- crates/singlefs-core/src` | 自 HEAD（`0b926bd3f`，2026-10-21）以来 9 份文件 719 行加、156 行删，其中 `transaction.rs` 684 行、`mounted_session.rs` 31 行；2026-10-20 那一趟的树早于 HEAD、又不是任何一次提交，起止哈希拿不到，只说「七·6 合入之前」 |
| 装置自己的映射条目取法 | 装置 `e185_gc_batch_wall_clock_split.rs`（原登记第五节「阶段拆分」） | 装置照原登记写时 `mounted_session.rs` 的取法另写了一份（逐 key 查）。实现那一边换成按叶解之后，装置那一份仍是独立的一份：停机条款第 4 条照旧比两边取出的条目集合，对不上就停（第十一节） |

**与条款对不上的一处**：七·6 那一行写的 `mapping_locations_of_a_mapped_unit`、`mapping_key_carried_from` 两个函数名今天零命中（上表）。只记在这里，不改里程碑表（不归设计员）。

## 四、跑之前已经存在的数

读原登记、条款、批次记录与实现时已经知道的数，照实列；判据照原登记、一个字不改，最后一列写它们对这一次新写的比法（第六节 C 组）有没有影响。

| 数 | 从哪读到 | 对判据与比法的影响 |
|---|---|---|
| 原登记第四节整张表的数（E21、D24 的吞吐，`O_DIRECT` 写零，锚点等） | 原登记第 179–191 行 | 同原登记 |
| 第一段第一趟（2026-10-19，装置 `ab45503e…`）的三行原样：CPU 臂阳性对照内存格 `wait_change=-0.530736`、真盘格 `wait_change=0.264753`、`cpu_increment_nanoseconds=352395055.0`、`354602470.0`；`name=verdict … positive_control_failures=2 … clause_wall_clock_not_rising_with_the_batch_triggered=10 clause_two_cpu_paths_disagree_points=2 gc_to_sequential_write_bandwidth_maximum_ratio=0.002852 gc_to_sequential_write_bandwidth_judgement=single_run_enough stored_intents=140`；调查员的数（基线等盘中位 −119583 ns、配对增量 47972223 ns、0.5238 等） | 原登记 12.9（第 559–574 行），读原登记时读到 | 那一趟是 12.9 之前的判法，不是 2026-10-20 的基线。比法不用其中任何一个数 |
| 造池用时：P = 8192、f = 0.25 每份 2022.6–2219.6 秒，f = 0.75 每份 1057.5–1107.0 秒，P = 1024 每份 9.4–20.7 秒，串行合计 16045.9 秒；时钟口径的数（`CLOCK_MONOTONIC` 慢 7.036 ppm 等） | 原登记 12.4、12.11（第 482、605、312 行） | 只用来估第十四节的挂钟 |
| 2026-10-20 那一趟：第一段 `name=verdict segment=wall_clock constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 …`（只读到这一截）；第二段一句「第 2 行的判读站不住；派调查员定位。这也是造池随池大小超线性的同一条线索」；第二台装置进程 VmRSS 10459704 kB、约 5.0 核 | 批次记录第 64、66、70 行（`grep` 定位「第二台」时整行打出来） | 是上一趟的结局与机器状态，读到了。比法的门槛（1.25、1.75）不由它们推出；「第 2 行站不住」那一句没有数 |
| 七·6 验收「副本推翻实验：P = 4096 一批 217.47 → 62.38 ms」 | 被测条款七·6 那一行（第二节） | 是副本上另一个装置的数，P = 4096 不是 E185 的取样点（装置 `POOL_DATA_UNITS` 只收 1024、8192，`e185_gc_batch_wall_clock_split.rs:281`），不拿它比；比法只在 E185 自己的两点上算 |
| 「七·6 之后一批 CPU 仍随池大小线性涨」 | `.claude/kb/milestone/04-fourth-txn.md` 第 172 行（「优化项」表，`grep 七·6` 命中） | 是对七·6 之后的预期，读到在写门槛之前。它说「线性」没给数；比法门槛按「平方」「线性」两个形状的四分之一处定（第六节），没按它调 |
| 七·6 代码轮的增长百分数 609%、359%、357%、362% | 批次记录第 85、87 行（`grep 七·6` 命中） | 量的是查找计数，不是 E185 的 CPU；不进比法 |
| 实验页现状：问题单第 1、2 行等七·6 合入之后两段重跑再判；第 4 行单跑那一半已判「单跑够」；第三段没跑 | 派发提示（实验页本身没读） | 第 4 行这一趟照常重量、并列报；翻面时第 4 行重开（第五节） |
| 自己算的比法锚点：8^1.25 = 13.454342644059432，8^1.75 = 38.05462768008707；α(1, 8) = 1.0，α(1, 64) = 2.0 | 第十三节的 python 命令 | 钉比法脚本的绝对值（第七节），不是门槛 |
| 读条款时推的一句：D26 已定项 13 第 7 条写「D24 已定项 3 那条按挂钟比的判据不管这一项罩着的后台任务」，而原登记 Q9 的门槛出自 D24 已定项 3、Q9 在第 2 行的用法是「Q9 < 1 那一段不进 GPU 集合」 | 第二节两段条款并读，推的、没量 | Q9 照原登记算、照原判定报，一个字不改；它在第 2 行还算不算翻面量，交主 agent 定（第六节、第十节 G6） |

## 五、臂、阳性对照、真实基线

**同上一次登记**：第五节「装置住哪」「负载」「计时与计数怎么做」「各行的臂」「阳性对照（每条臂都跑）」「真实基线」「GPU 一侧怎么量」「并行与挂钟」整节照原登记（第 193–312 行，含 12.1–12.11），装置是入库装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`，一个字不改；阳性对照照旧每条臂都跑（第一段第 1 行两条臂、第二段三个阶段，第三段两张卡）。这一次只补跑法、产物名、输入清单、分段与第三、四段的条件。

**装置写在哪**（写死）：不新写装置。原登记写死的入库装置照用；比法（第六节 C 组）是只读两段产物的算术，住 `scripts/experiment-apparatus/e185_r1_compare.py`（独立手写，只读产物文本，不 import `crates/` 任何东西），变异见第九节。

### 跑法（与 2026-10-20 那一趟同形）

- **机器**：两段都在第二台跑，与 2026-10-20 那一趟同一台（第一节「真盘」那一格，待主 agent 认）；镜像目录与那一趟同一块盘上的 ext4。
- **编法**：在起跑的那棵树里 `cargo build --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split`（第一、二段不带 `gpu` 特性；第三段另加 `--features gpu` 编一次），不带 `--all-targets`、`--tests`，不与 harness 档的测试一起编（免得并进 `lookup-work-counters`）。
- **上限与线程**：同那一趟：外层内存包装 `40G`、`E185_MEMORY_CAP=40G`；`E185_THREADS` 不设，装置取登记值 `.claude/heavy-thread-default`（今天 31）；`SINGLEFS_SERIAL_DECLARATIONS` 照 `scripts/hosts/multi-host-run.sh --peer` 设。
- **输入清单**（起跑前、在起跑的那棵树里写，跑完不改）：`research/results/e185-r1-inputs-<日期>.txt`，逐行：装置 sha256；`crates/singlefs-core/src/` 下 `transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs`、`checksum.rs`、`block_device.rs` 与 `Cargo.lock` 的 sha256；`git rev-parse HEAD` 与 `git diff --cached --stat -- crates/singlefs-core/src | tail -1`（树是副本、不是 git 仓时写「副本，不是 git 仓」与它从哪棵树拷来）；`cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline` 原样；镜像目录所在盘的型号与逻辑块大小（`/sys/block/<盘>/device/model`、`queue/logical_block_size`）；`nproc`；两样上限。机器名、第二台上的路径不写（私有值，`.claude/agent-common.md`「写」一节）。
- **上一趟两份产物**（基线，只读不改）：`research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out` sha256 `a6afc7bf2f380d86f3b51a070c91465c3206a2476c216af6ab59fbe700a7d87b`；`research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out` sha256 `e087a1b6834a381322c87ebd0a53a242b66ce035812fab39d7628b79e3a9e125`（与批次记录第 66、70 行记的前缀 `e087a1b6`、`a6afc7bf` 相同）。比法开算前再核一次，不同就停（第十一节）。

### 分段（这一次跑哪几段）

产物文件名里的 `<日期>` 是那一段起跑当天的 `TZ=Asia/Tokyo date +%F`；带 `-r1`，与 2026-10-20 的文件名不撞；同一天同一段要再跑一次的，文件名末尾再加 `-2`、`-3`，不盖已有的（`.claude/agent-common.md`「执行前拒绝的写法」⑤）。

| 段 | 跑不跑 | 补问题单哪一行 | 产物 | 补完那一行够不够判 |
|---|---|---|---|---|
| 第一段 `wall-clock` | 跑，全部取样点（不带 `--point`，同那一趟） | 第 1 行（Q1–Q4）；第 4 行单跑那一半（Q14–Q16）重量、并列报；第 3 行要的 B_批（Q10） | `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-<日期>.out` | 第 1 行照原登记第五节分段表那一格；第 4 行：这一趟 `name=q16` 的 `judgement=single_run_enough` 时单跑那一半照旧够判，不是时见下面「第四段」 |
| 第二段 `stage-split` | 跑，全部取样点 | 第 2 行（Q5–Q8，饱和吞吐）；七·6 验收的比法（C 组） | `research/results/e185-gc-batch-wall-clock-split-stage-split-r1-<日期>.out` | 第 2 行的份额（Q5–Q7）够判；Q9 要第三段 |
| 比法 | 两段都交回、都有效之后算 | 不补岔路行（附带） | `research/results/e185-r1-comparison-<日期>.out`（`e185_r1_compare.py` 的输出） | 不适用 |
| 第三段 `gpu-submission` | 条件成立才跑（下面） | 第 3 行（Q10–Q13）；第 2 行的 Q9 | `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-<日期>.out` | 第 3 行够判；Q9 交主 agent 定还算不算第 2 行的翻面量（第六节） |
| 第四段 `foreground-fsync`（附带） | 条件成立才跑（下面） | 第 4 行并发那一半（Q17） | `research/results/e185-gc-batch-wall-clock-split-foreground-fsync-r1-<日期>.out` | 第 4 行够判 |

次序：第一段先、第二段后，同一台上先后跑、不同时跑（第十四节「串行：」）。第一段先，是因为它单独就能让第 1 行够判，第二段单独让不了任何一行够判（第 2 行要「第 1 行够判的那组取样点」）；次序不影响任何判定。

### 第三段跑不跑（条件句，由主 agent 读产物判）

第一、二两段交回之后，**下面四格全成立就跑第三段**，`--wall-clock-product` 与 `--stage-split-product` 只给这一趟（`-r1-`）的两份产物，不给 2026-10-20 的（两棵树的产物不混用）：

| 格 | 读哪份产物的哪一行、哪个字段 | 成立的样子 |
|---|---|---|
| 三甲 第一段有效 | 第一段 `-r1-` 产物：末行 `name=done`；`name=verdict segment=wall_clock` 一行的 `constant_or_anchor_mismatches`、`stop_clause_findings`、`positive_control_failures`；全文 `name=stopped` 的行数 | 末行是 `name=done`；三个字段都是 `0`；`name=stopped` 0 行 |
| 三乙 第二段有效 | 第二段 `-r1-` 产物：末行 `name=done`；`name=verdict segment=stage_split` 一行的同三个字段；每一行 `name=saturation` 的 `distinct_threads_holds` | 末行是 `name=done`；三个字段都是 `0`；`distinct_threads_holds=true` 行数等于 `name=saturation` 行数 |
| 三丙 B_批 读得出 | 第一段 `-r1-` 产物里 `name=call` 且 `arm=memory`、`injection=none`、`round=1`、`kind` 为 `batch` 或 `final_batch` 的行的 `scan_read_bytes_in_the_region`（即第三段 `batch_bytes_of_the_wall_clock_product` 读的那一组，装置第 7010–7038 行） | 每个取样点、每个 K 至少一行，且值都大于 0 |
| 三丁 第 3 行还开着 | 问题单 `research/prompts/m4-gpugc-measure-forks.md` 第 3 行「状态」那一格（主 agent 读两段产物时的样子） | 是「开着」 |

不跑的三种：三丁不成立（第 3 行已经「够判」或「用户已定」）就不跑，第三段与 Q9–Q13 标「够判后未跑」；三甲或三乙不成立的，先照原登记第十一节把那一段重跑或停机交回，第三段等它，不拿 2026-10-20 的产物顶；三丙不成立的走原登记第十一节停机条款第 2 条（扫描读字节对不上）。第二段 `name=stage_judgement` 每一行的 `crc_share_judgement`、`scan_increment_share_judgement`、`choose_share_judgement` 都是 `not_worth`（原登记第十条 F7）时，第 2 行不再要 Q9，但第 3 行仍要 a、b，第三段照样按上表判跑不跑。清场没过、取卡没过（退 78）的，第三段记「本次未跑」，第 3 行照旧开着。

### 第四段跑不跑（条件句）

照原登记第五节分段表与第十一节「够判停机」：第一段 `-r1-` 产物 `name=q16` 一行的 `judgement` 是 `single_run_enough` 就不跑，第四段与 Q17 标「够判后未跑」；是 `needs_the_concurrent_measurement`（`maximum_ratio` > 0.1）才跑，取样点与 K 取 `name=q16_point` 里 `ratio` 最大的那一行。2026-10-20 那一趟的 Q16 不参与这一判（这一判只看这一趟）。

## 六、报哪些量与各自的判据

**同上一次登记**：Q1–Q17 每一行的算法、门槛、判定、对应的问题单行与翻面取值照原登记第六节（第 314–338 行）与 12.5、12.6 的产物键名，一个字不改；每个量照旧各报各的判定，5 轮判定不全相同写「不稳定」。

**Q9 一格交主 agent**：Q9 照原登记算、照原门槛（D24 已定项 3「小于 1 就否掉那一格」）判、照原样报。今天的 D26 已定项 13 第 7 条写「D24（后台重活能不能卸给 GPU） 已定项 3 那条按挂钟比的判据不管这一项罩着的后台任务」（第二节），而原登记第六节 Q9 那一行在第 2 行的用法是「某段 Q5–Q7 判值得而 Q9 < 1，那一段不进 GPU 集合」。这一次不改 Q9 的门槛与用法；第 2 行判的时候 Q9 还算不算翻面量，由主 agent 定（第十条 G6）。

**并列报（新加，附带：只从已有产物算，不另跑；不改任何一条判据）**。下面每一格写读哪份产物、哪一行、哪个字段；「前」= 2026-10-20 那一份，「后」= 这一趟 `-r1-` 那一份，两份用同一条规则取。

| 量 | 读哪一行、哪个字段 | 怎么算 | 门槛与判定 | 对应哪一行与翻面取值 |
|---|---|---|---|---|
| C0 前后是不是同一份活 | 两段产物各自的 `name=geometry` 行（键 `fraction`、`pool_data_units`）的 `choice`、`formed`；`name=geometry_attempt` 最后一次的 `mapped_units`；`name=intent`（`arm=memory`、`injection=none`、`valid=true`）的 `batches`；`name=call`（同上、`round=1`）逐个 `call_index` 的 `kind`、`scan_read_bytes_in_the_region`、`release_check_read_bytes`、`relocation_copy_bytes`，另报 `write_bytes`、`metadata_write_bytes` | 前后逐字段比 | `choice`、`formed`、`mapped_units`、`batches` 与每个调用的 `kind` 和三样读与搬迁字节都相同判「同一份活」；有一样不同判「活不同」，那个（`fraction`、`pool_data_units`、`batch_units`）不进 C1、C2；`write_bytes`、`metadata_write_bytes` 只报差，不判 | 不对应岔路行；决定 C1、C2 那一格算不算数 |
| C1 一批 CPU 随池大小的指数（第一段来源 W） | 第一段产物 `name=call`，`arm=memory`、`injection=none`，每轮取 `name=intent` 为 `valid=true` 的那一次 `attempt`，`kind` 为 `batch` 或 `final_batch` 里 `call_index` 最小那一行的 `cpu_nanoseconds` | 每个（`fraction`、`batch_units`）：c(P) = 有效轮的中位；α = ln(c(8192) ÷ c(1024)) ÷ ln 8；另按轮号配对报每轮的 α | α ≤ 1.25 判「两点之间接近线性」；α ≥ 1.75 判「两点之间近平方」；中间判「介于两者」；每轮 α 的判定不全相同写「不稳定」。前、后各判各的，各占一行 | 七·6 验收「从平方降到接近线性」：前 ≥ 1.75 而后 ≤ 1.25 是那一句在这一格上看得到的样子；合不合成一句由主 agent 判 |
| C1 同上（第二段来源 S） | 第二段产物 `name=stage`，`kind` 为 `batch` 或 `final_batch` 里 `call_index` 最小那一行的 `call_cpu_nanoseconds`，每轮一个（作废轮第二段不打 `name=stage`） | 同上 | 同上 | 同上 |
| C1b 「其余」的指数（来源 S） | 同一行的 `call_cpu_nanoseconds − scan_cpu_nanoseconds`（一批 CPU 减甲与乙增量；这一批行没有选段，原登记第一节「其余」那一格的取法） | 同 C1 | 同 C1 | 同 C1；七·6 改到的查找都在这一份里 |
| C2 前后比 | C1 两个来源的 c(P)，每个（`fraction`、`pool_data_units`、`batch_units`） | 后 ÷ 前 | 不判，只报 | 不翻面；给主 agent 看 P = 8192 那几格降了几成 |
| C3 岔路量并列 | 第一段：`name=q1` 的 `rho_median`、`judgement`；`name=q2` 的 `intent_wall_ratio_median`、`judgement`；`name=q3` 的 `judgement`；`name=q4` 的 `median`、`judgement`；`name=q14` 的 `write_bytes_per_second_median`；`name=q15` 的 `direct_with_fdatasync_bytes_per_second_median`；`name=q16` 的 `maximum_ratio`、`judgement`；第二段：`name=stage_judgement` 的 `crc_share_judgement`、`scan_increment_share_judgement`、`choose_share_judgement`；`name=stage` 的 `crc_share`、`scan_increment_share`、`choose_share_per_batch`、`crc_bytes_per_second`、`scan_increment_bytes_per_second`（各轮中位）；`name=saturation` 的 `bytes_per_second`（键 `stage`、`threads`）；`name=choose_disk_size` 的 `cpu_nanoseconds_per_full_segment` | 按各行自己的键（`arm`、`fraction`、`pool_data_units`、`batch_units`、`batch_units_pair`、`batch_index`、`call_index`、`stage`、`threads`）把前后对上，逐字段并列 | 不新判：判定照原登记、两趟各用各的；同一格前后判定不同记「前后翻面（两棵树之间）」 | 第 1、2、4 行：岔路行按这一趟的判定判，上一趟的只并列 |

**C 组能拿来比七·6 前后的只有内存臂的 CPU 那几格**（C1、C1b、C2，C3 里第二段的份额与吞吐）：七·6 只改 CPU 上的查找、声明写出的盘镜像逐字节相同（七·6 那一行验收），盘上读写的量前后应当相同（C0 核），所以真盘臂的等盘、写带宽与 ρ 前后有差，归不到七·6（机器那一刻的状态也在里面），只并列、不拿来说七·6。

**C1 只有两个 P 点的限制**（跑前写下）：两点只给一个指数。一批 CPU 里有不随 P 变的那一份（CRC、发布里的定长活）时，两点指数比主导项的指数小，所以「后 ≤ 1.25」只说这两点之间涨得不比 P^1.25 快，不证明更大的池上也如此；「前 ≥ 1.75」则说明这两点之间至少涨得像 P^1.75。装置只收 P = 1024、8192（`e185_gc_batch_wall_clock_split.rs:281`），要第三个点就是改装置、另起一次，这一次不做。

**C 组什么观测会让它不成立**：比法本身作废——输入清单里 `cargo tree` 的输出含 `lookup-work-counters`、装置 sha256 不是 `b3a50d50…`、两份基线产物 sha256 与第五节不同、两趟的 `name=configuration segment=wall_clock` 的 `image_directory_filesystem` 或 `image_device` 不同（只作废真盘那几格）、C0 判「活不同」（只作废那一格）。七·6 那一句在 E185 的点上不成立——任何一个 C0 判「同一份活」的（`fraction`、`batch_units`）上，后一趟 W 或 S 的 α > 1.25（或「不稳定」），记「降到接近线性在这一格上没看到」；前一趟 α < 1.75，记「从平方在这一格上没看到」（第十条 G1、G2）。

## 七、钉绝对值的断言

**同上一次登记**：甲类（出自被测条款本身，不符走原登记第十条 F1、F2）与乙类（独立算出、命令核过，不符才作废）两张表照原登记第七节（第 340–368 行）与 12.5 补的那一条，一个字不改；装置逐批核。

**比法脚本 `scripts/experiment-apparatus/e185_r1_compare.py` 的锚点**（新加；乙类：独立算出、第十三节的 python 命令核过，不符这一份比法输出作废、修脚本重算，不碰产物）：

| 断言 | 值 | 在哪核 |
|---|---|---|
| 指数函数 α(c1024, c8192) | α(1, 8) = 1.0；α(1, 64) = 2.0；α(2, 16) = 1.0 | 脚本自证 |
| 门槛两条边含在哪一侧 | α(1, 13.454342644059432) = 1.25 判「两点之间接近线性」；α(1, 38.05462768008707) = 1.75 判「两点之间近平方」 | 脚本自证 |
| 比法读到的第一批是搬了 K 个单元的那一批（两份产物都核，只在 `name=geometry` 为 `formed=true` 的点上） | `relocation_copy_bytes` = `2 × min(K, m) × 32768`：f = 0.75 时 K = 4、8、16 各 262144、524288、1048576；f = 0.25 时 K = 4 是 262144、K = 8 与 16 都是 524288（原登记第七节乙类第 359 行） | 真产物，前后两份 |
| 比法读到的签发那一次 | `kind=issue` 那一行的 `scan_read_bytes_in_the_region`：f = 0.25 是 262144，f = 0.75 是 786432（原登记第七节乙类第 356 行） | 真产物，前后两份 |

互比旁边的绝对值：C2（前后互比）与 C1（P 两点互比）旁边是上表后两行：前后两份产物与两个 P 点一起读错了行（例如都取成签发那一次），`relocation_copy_bytes` 那一条先红。

## 八、轨迹与几何敏感性

**同上一次登记**：取样点（设备两种、K = 4、8、16、f = 0.25、0.75、P = 1024、8192、丙段盘大小两点）、轨迹表、几何敏感性与判别力自证照原登记第八节（第 370–401 行），不改。

**C1、C1b 的几何敏感性**（新加）：P 是 α 自己的自变量，敏感性看另两个旋钮：每个 α 的判定在 f = 0.25 与 0.75（方向相反的两点：f 低时一条意图批少、每批扫描字节小）、K = 4 与 16 的两端各报一次；某一判定在一个旋钮两端翻面，写「随 <旋钮> 翻面」并列两点的值，不写成七·6 的性质。f = 0.25 时 K = 8、16 两格第一批都是 8 个单元（原登记第八节第 382 行），这两格的 α 照报、不算两个独立点。**判别力自证**：脚本单测把 1.25 挪到两个量到的 α 之间（合成值），判定必须由「两点之间接近线性」变成「介于两者」；1.75 同样。

**轨迹**：C 组不被任何条款当谓词输入，不报轨迹；被谓词消费的量（每批写带宽、B_批）照原登记第八节报。

## 九、变异

**同上一次登记**：装置的变异 E185 V1–V55（`crates/mutations.tsv` 里以「E185 V」起头的 55 行，`grep -c 'E185 V' crates/mutations.tsv` 输出 55）照原登记第九节与 12.1–12.11，不增不删；装置不改，锚点照旧逐字命中。

**比法脚本的变异**（新加；脚本是独立手写的读产物算术，变异表 `scripts/experiment-apparatus/e185_r1_compare.mutations.tsv`，原文在脚本里恰好命中一次）：

| 变异 | 改哪 | 在哪个取样点改变输出 | 该红的 |
|---|---|---|---|
| M1 对数底错 | `ln(比) ÷ ln 8` 写成 `ln(比) ÷ ln 2` | 合成 c(1024) = 1、c(8192) = 64（α 应 2.0，变异给 6.0） | 指数函数锚点 |
| M2 第一批取成签发那一次 | 取 `call_index` 最小那一行时不滤 `kind` | 任何真产物的点，例 f = 0.75、K = 8（签发那一行 `relocation_copy_bytes` = 0，不是 524288） | 「第一批是搬了 K 个单元的那一批」 |
| M3 作废轮不剔 | 第一段来源 W 不看 `name=intent` 的 `valid` | 合成五轮，三轮 `valid=false` 且 CPU 大十倍（中位应取有效两轮的，变异取到作废轮的） | 剔除作废轮的单测 |
| M4 线性那条边不含等号 | `α ≤ 1.25` 写成 `α < 1.25` | 合成 α = 1.25（c = 1、13.454342644059432） | 门槛两条边的锚点 |
| M5 不滤内存臂 | 第一段来源 W 不看 `arm=memory` | 合成同一点一行 `arm=real_disk` 的 CPU 大一倍 | 只取内存臂的单测 |
| M6 C0 漏比释放前读盘核那一栏 | 前后比时去掉 `release_check_read_bytes` | 合成两份产物只差这一栏（应判「活不同」，变异判「同一份活」） | C0 的单测 |
| M7 按轮配对错位 | 每轮 α 用第 r 轮的 c(1024) 配第 r + 1 轮的 c(8192) | 合成五轮各不相同、只有同轮配对才都给 α = 1.0 | 每轮 α 的单测 |
| M8 中位写成平均 | 中位换成算术平均 | 合成五轮 1、1、1、1、100（中位 1，平均 20.8） | 中位单测 |
| M9 门槛挪动不翻面 | 判定函数忽略传入的门槛、写死 1.25 | 判别力自证那一格（门槛挪到两个合成 α 之间） | 判别力自证 |

## 十、失败条款

**同上一次登记**：F1–F8 照原登记第十条（第 427–440 行），不改。

**新加，只管 C 组**（每条紧跟触发的观测）：

| 条款 | 判成什么 | 什么观测会让它触发 |
|---|---|---|
| G1 降到接近线性在这一格上没看到 | 正当结果，不作废：照实记「七·6 验收那一句的后半在（`fraction`、`batch_units`）这一格上没看到」，交主 agent | C0 判「同一份活」的一格上，后一趟 W 或 S 的 α > 1.25，或每轮判定不全相同 |
| G2 从平方在这一格上没看到 | 正当结果：照实记「前一趟这一格不是近平方」，那一格的「降」无从说起 | 前一趟 W 或 S 的 α < 1.75 |
| G3 两条来源对不上 | 那一格不下判定，写「两源不一致」，两边都查（两段各自造池、各自量，取法应当相同） | 同一趟同一格 W 与 S 的 α 判定不同 |
| G4 前后活不同 | 那一格不进 C1、C2，两份产物对不上的字段原样列出，交主 agent | C0 判「活不同」 |
| G5 岔路量前后翻面 | 不是失败：这一趟的判定照原登记判岔路行，并列写「前后翻面（两棵树之间）」，不单归七·6 | C3 同一格前后判定不同 |
| G6 Q9 的用法与今天的 D26 已定项 13 第 7 条相左 | 不作废、不改：Q9 照原判定报；那一段进不进 GPU 集合不由 Q9 一格定，交主 agent | 第三段跑了，且有一段 Q5–Q7 判 `worth` 而它的 Q9 判 `gpu_loses`（原登记第 2 行用法下会把那一段剔掉的那种） |

没有一条把「没测出差别」写成实现有 bug：七·6 对不对由七·6 自己的测试与代码轮判，这一次只报两棵树之间量到的差。

## 十一、作废条款与停机条款

**同上一次登记**：作废八条、停机六条（连同 12.2 的时钟分辨率自检、12.10 的线程核、12.11 的串行声明写不进）照原登记第十一节（第 442–470 行）与第十二节，不改；装置照旧在产物里打 `name=stop_clause`、`name=stopped`，退出码照装置文件头。

**新加**（每条紧跟触发的观测）：

| 条款 | 作废或停什么 | 什么观测会让它触发 |
|---|---|---|
| 装置变了 | 停：重跑的前提「装置不改」不成立，不起跑，交主 agent | 起跑那棵树里装置 sha256 不是 `b3a50d50441f9f2e74bec6c941b74e632529688a78563995387708230b60c470` |
| 计数特性被并进来 | 这一趟两段产物整段作废（CPU 里混了计数的开销），照第五节编法重编重跑 | 输入清单里 `cargo tree` 那一段出现 `lookup-work-counters` |
| 基线产物变了 | 停比法（C 组不算），岔路行照这一趟判；交主 agent | 两份 2026-10-20 产物的 sha256 与第五节不同 |
| 真盘不是同一块 | C 组与 C3 里真盘臂那几格不比（岔路行照这一趟判） | 两趟第一段 `name=configuration segment=wall_clock` 的 `image_directory_filesystem` 或 `image_device` 不同，或输入清单的盘型号与那一趟所在的盘不同 |
| 比法锚点不符 | 那一份比法输出作废，修脚本重算（产物不动） | 第七节比法锚点任一条不符 |
| 实现与装置对不上 | 照原登记第十一节停机条款（`.claude/rules/implementation-first.md` 第 4 条：两边都查，既不作废也不当结果）；第 4 条（装置另写的映射条目取法与会话判成可搬的 key 对不上）在七·6 把实现换成按叶解之后照旧判 | 原登记停机条款 1–6 任一条 |

**够判停机**：问题单四行每一行都够判就停，没跑的量逐条标「够判后未跑」。具体：第一、二两段交回后主 agent 对着问题单逐行判；第 3 行已不开着时第三段与 Q9–Q13 标「够判后未跑」；这一趟 `name=q16` 判 `single_run_enough` 时第四段与 Q17 标「够判后未跑」；C 组是附带，两段有效就算、不为它续跑任何一段，也不为它加第三个 P 点。

## 十二、修订

（装置不改；比法脚本写之后、两段产物之前由执行员写，只许收严或补量。原判据、门槛、变异九行原样保留。）

写于比法脚本 `scripts/experiment-apparatus/e185_r1_compare.py` 与它的单测 `scripts/experiment-apparatus/e185_r1_compare_test.py` 跑绿之后、这一趟两段产物一次都没跑之前（单测 15 项绿，九行变异 9 抓到、0 无效、0 没红），派发提示「关的是哪几条」写的是「连同它的单测」，下面几条都是把第十一节、第六节已有的条件落成脚本里的检查，只收严、不加判据：

1. 单测放同目录 `e185_r1_compare_test.py`，不放脚本里的 `--selftest`：依据是 `--selftest` 要先登记进 `.claude/hooks/selftest-registry.tsv`，执行员没有写那张表的范围，闸拒了第一次跑（原话「这份自证没在登记表 … 里」）；第七节「脚本自证」两行由这份单测的「指数函数锚点」「门槛两条边含在哪一侧」两项落实。登记要不要补那一行，交主 agent。
2. 脚本开算前核「前」两份的 sha256（第五节两个值），不同就停、退 3，一行比较都不打；「后」那一份的 sha256 与上一趟相同（把基线当成新产物给）退 2。依据：第十一节「基线产物变了」，自证项「前的产物 sha256 不对就停」「输入清单与后的产物不能是上一趟的」。
3. 脚本可选读输入清单（`--inputs-list`）：清单里没有装置 sha256 `b3a50d50…` 全串，或出现 `lookup-work-counters`，输出末行 `output_void=true` 并退 4。依据：第十一节「装置变了」「计数特性被并进来」，自证项「输入清单与后的产物不能是上一趟的」。不给清单时该格写 `problems=not_run`，不当通过。
4. 两趟第一段 `name=configuration` 的 `image_directory_filesystem` 或 `image_device` 不同时，C3 里靠真盘的几格（q1、q2、q3、q14、q15、q16 与 q4 里 `arm=real_disk` 的键）写 `different_disk_not_compared`，不并列判。依据：第十一节「真盘不是同一块」。
5. 第一段 `cpu_unreadable=true` 的轮不进 W 的中位与配对，另数；指数里 c 不为正的当读不到（`alpha=none`、`judgement=not_readable`），不当 0。依据：`.claude/singlefs-ai-sop/rules/test-discipline.md`「读不到 ≠ 读到 0」，自证项「读不到不当 0」。
6. 第七节比法锚点（`relocation_copy_bytes` 与签发那一次的 `scan_read_bytes_in_the_region`）在脚本里对每个 `formed=true` 的点、每个有效轮都核，任一不符 `anchor_mismatches` 大于 0、`output_void=true`、退 4；`formed=false` 的点写 `status=not_checked`，不当通过。上一趟（2026-10-20）两份产物上，f = 0.75 的 6 个格每个核 5 轮、0 不符，f = 0.25 的 6 个格 `not_checked`（`name=geometry` 写 `formed=false`）。

## 十三、读过的文件与跑过的命令

### 读过的文件（行号区间；grep 命中行也算）

| 文件 | 读了哪些行 |
|---|---|
| `.claude/agent-common.md` | 1–109 |
| `.claude/agents/experiment-designer.md` | 派发时随定义给出（全文） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 1–122 |
| `.claude/rules/three-way-inference.md` | 1–166 |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 1–179 |
| `.claude/rules/mutation-sampling.md` | 1–93 |
| `.claude/rules/implementation-first.md` | 1–19 |
| `.claude/rules/test-script-discipline.md` | 1–106 |
| `research/prompts/e185-preregistration.md` | 1–721（全文，含第十二节 12.9 里第一段第一趟的结局行） |
| `research/prompts/m4-gpugc-measure-forks.md` | 1–12 |
| `.claude/kb/milestone/04-fourth-txn.md` | 全部标题行；133–149；`grep 七·1\|七·6` 命中 20、38、68、141、142、143、161、168、172 |
| `.claude/kb/decisions/26-后台整理与放置回收.md` | 全部标题行 1–60 号以内；232–252（经 quote-kb） |
| `.claude/kb/decisions/24-后台重活能不能卸给GPU.md` | 全部标题行；47–59（经 quote-kb） |
| `records/2026-10-19-里程碑四GPU垃圾回收开工.md` | 1–16；标题行；`grep 七·6` 命中 69、70、72、73、75、76、79、80、82、83、85、86、87、90、92、93、94、96、97、103、104、107、109、110、111、128、130、134、140、142、150、153、155、159、160、161、162、164、165、170、171、172、174、176、179、180、181、182、183、184、185、186、188、191、198、199、201、202（每行只看到前 400 字节）；198、202（整行）；47–73 里含「第二台」「本机」「--peer」的片段（49、52、53、56、58、59、61、62、63、64、65、66、70、71） |
| `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` | 1–80；200–300；560–600 里的标签；882、945、1337、1474、2636、2835、4004 前后标签；1360–1366；1560–1640 的判定标签；2741–2760；3704–3790；5935–5960；6470–6532；6596–6650；7005–7100；`grep name=` 各行（471–7748 间） |
| `crates/singlefs-checker-tier/Cargo.toml` | 1–45 |
| `crates/singlefs-harness/Cargo.toml` | 1–40 |
| `crates/singlefs-core/Cargo.toml` | 1–30 |
| `crates/singlefs-core/src/transaction.rs` | 3415–3440；3525–3545；4700–4720；3982、4069、4078、9541、10003、11787；`grep` 命中 3261–3362 的特性行、3536–3586、4712–4766、`lookup_work::entering` 各行、`units: Vec<PublishedUnit>` 各行 |
| `crates/singlefs-core/src/mounted_session.rs` | 2785–2830；`grep` 命中 71、1939、1957、2794、2796、2818 |
| `crates/singlefs-core/src/region_scan.rs` | 268–330 里的 `grep` 命中；204、286、309、310 |
| `crates/singlefs-core/src/code_two_tree.rs` | `grep carried_from` 命中 331–409 |
| `scripts/gate/admission.py` | 1–40 |
| `scripts/kb/quote-kb.py` | `--help` 前 30 行 |
| `.claude/gate.d/doc-experiments.sh` | `grep prereg` 命中 26、63、64、94、118、2152、2293–2364 |
| `.claude/heavy-thread-default` | 全文 |

没读：`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md`（只 `ls` 了文件名）与 `.claude/kb/experiments/` 下别的实验页；`research/results/` 下任何产物的正文（两份 2026-10-20 产物只算了 sha256，用 `ls -la` 看了文件名与大小）。读到的已有的数列在第四节。

### 跑过的命令（原样）

只读命令与写这份登记的命令；没编译、没跑装置、没跑门禁阶段，没读产物正文：

```bash
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m4-e185-r1-designer --purpose 'E185 七·6 合入后第一次重跑登记'
sha256sum research/prompts/e185-preregistration.md research/prompts/m4-gpugc-measure-forks.md; grep -n '^#' research/prompts/e185-preregistration.md
grep -n '^#' .claude/kb/milestone/04-fourth-txn.md; grep -n '七·1\|七·6' .claude/kb/milestone/04-fourth-txn.md | cut -c1-300
grep -n '^#' .claude/kb/decisions/26-后台整理与放置回收.md | sed -n 1,60p; grep -n '^#' .claude/kb/decisions/24-后台重活能不能卸给GPU.md
sha256sum crates/singlefs-core/src/transaction.rs crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs crates/singlefs-core/src/mounted_session.rs crates/singlefs-core/src/region_scan.rs crates/singlefs-core/src/compaction_plan.rs
git log --format='%h %ad %s' --date=short -8 -- crates/singlefs-core/src/transaction.rs
git status --short -- crates/singlefs-core/src/ crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs
git diff --stat HEAD -- crates/singlefs-core/src | tail -12
grep -rn 'mapping_locations_of_a_mapped_unit\|mapping_key_carried_from' crates --include=*.rs | head -8     # 零命中
grep -rn 'thread::spawn\|thread::scope\|rayon' crates/singlefs-core/src | wc -l                             # 0
grep -c 'E185 V' crates/mutations.tsv                                                                       # 55
timeout 120 nice -n 19 cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline
D=/home/fy5090/.local/share/singlefs/work/m4-e185-r1-designer; nice -n 19 python3 scripts/kb/quote-kb.py $D/clauses.md '.claude/kb/milestone/04-fourth-txn.md~^\| 七·1 \|' '.claude/kb/milestone/04-fourth-txn.md~^\| 七·6 \|' '.claude/kb/decisions/26-后台整理与放置回收.md@#### 已定项 13：后台任务的计算放在哪、怎么开、什么节奏' '.claude/kb/decisions/24-后台重活能不能卸给GPU.md@#### 已定项 3：判据取什么形式'
diff <(sed -n 41,55p $D/clauses.md) <(sed -n 108,122p research/prompts/e185-preregistration.md) && echo D24-same
sha256sum research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out
```

`cargo tree` 的输出（原样）：

```text
singlefs-core v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-core)
└── singlefs-core feature "default"
    ├── singlefs-checker-tier v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-checker-tier)
    │   ├── singlefs-checker-tier feature "default" (command-line)
    │   └── singlefs-checker-tier feature "verdict-store"
    │       └── singlefs-checker-tier feature "default" (command-line)
    └── singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
        └── singlefs-harness feature "default"
            └── singlefs-checker-tier v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-checker-tier) (*)
```

`quote-kb.py` 的输出：`✓ 4 段整抄进 /home/fy5090/.local/share/singlefs/work/m4-e185-r1-designer/clauses.md，回读逐字节一致`；`diff` 输出 `D24-same`。

两份 2026-10-20 产物的 sha256（原样）：

```text
a6afc7bf2f380d86f3b51a070c91465c3206a2476c216af6ab59fbe700a7d87b  research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out
e087a1b6834a381322c87ebd0a53a242b66ce035812fab39d7628b79e3a9e125  research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out
```

第七节比法锚点的算术（命令与输出原样）：

```bash
python3 -I -c '
import math
print("8^1.25", 8**1.25, "8^1.75", 8**1.75)
for a,b in ((1,8),(1,64),(1,8**1.25),(1,8**1.75),(2,16)):
    print(a,b, math.log(b/a)/math.log(8))
print("ln-base-mutation e.g. log10:", math.log10(64))
'
```

```text
8^1.25 13.454342644059432 8^1.75 38.05462768008707
1 8 1.0
1 64 2.0
1 13.454342644059432 1.25
1 38.05462768008707 1.7500000000000002
2 16 1.0
ln-base-mutation e.g. log10: 1.806179973983887
```

其余只读命令（`sed -n`、`awk 'NR==…'`、`grep -n`、`wc -l`、`ls`）读的行号都列在上一张表里。

## 十四、并行与挂钟

原登记第五节「并行与挂钟」（12.10、12.11）管段内怎么并行，这一次照旧；这里写这一趟几段之间怎么排。

- 独立单位：第一段与第二段互相独立（各自造池、各自量，第二段不读第一段的产物）；第三段读这一趟前两段的产物，排在它们之后；第四段只在第五节的条件成立时跑；段内的独立单位照原登记第五节「并行与挂钟」那张表（不计时的准备各份互相独立，计时窗口不并行）。
- 线程：每段不计时的准备与第二段饱和吞吐的 N 都取 `E185_THREADS`，不设，由装置取登记值 `.claude/heavy-thread-default`（今天是 31）；计时窗口一条线程；比法脚本一条线程。
- 串行：第一、二两段在同一台（第二台）上先后跑、不同时跑，因为两段都有计时窗口，原登记第五节「并行与挂钟」与第十一节「机器被别的活挤」要求计时时机器上没有别的测量，而两段都要与 2026-10-20 那一趟同一台机器比，不能分到两台各跑一段；段内计时窗口串行（量的是一条线程的 CPU 与挂钟）；第三段要前两段的产物，排在最后。
- 预计挂钟：第一段约 1.5–1.7 小时、第二段约 1.1–1.3 小时（推的，照原登记第五节分段表 12.11 改写的两格与第 312 行的依据：造池串行合计 16045.9 秒，分批并行后第一段准备约 3500–4160 秒、第二段约 3330–3980 秒）；两段先后合计约 2.6–3.0 小时，每段各自在 2 小时内；比法脚本几秒；第三段（条件成立时）2 小时内，推的：带 `gpu` 特性另编一次约 10 分钟，每张卡 2 组 × 8 档 × 6 次不到 1 分钟。
- 预计在用核数：第一、二段的准备阶段约 15–20 核（推的：原登记 12.11 按估算 28G 下头一次派 15 份、64G 下 20 份；这一趟上限 40G，介于两者），计时窗口 1 核，第二段饱和吞吐 16 核与 31 核两档各约 2 秒 × 12 点；比法脚本 1 核；第三段 1 核加一张 GPU。
