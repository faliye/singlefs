# E185 第 3 次重跑登记：GC一批搬迁的挂钟拆分（一次 GPU 提交在主机线程上的 CPU、它的固定与每字节两部分、重算之后仍不一致的概率；附带一批 CPU 按函数拆）

写于 2026-10-23，装置改之前、这一趟一份产物都还没跑。不是差分重跑：要回答的是三条新岔路（七·2 设计轮第一轮判决点名的缺口），装置要加两个带 `gpu` 特性的子命令与一个附带子命令，原有子命令、原有产物行与原有判据一个字不改。下文写「原登记」「r1 登记」「r2 登记」分别指 `research/prompts/e185-preregistration.md`（读的那一刻 sha256 `c84bf038fbfd16a960081d4e053aeda213b7119db87804fa53ad64ffa7437697`，721 行）、`research/prompts/e185-r1-prereg.md`（`7ce2419b70a82523061393986da2b47294c394cb78895f0bcd5c084f63c35de0`）、`research/prompts/e185-r2-prereg.md`（`7d74c089ea31e7f49e2762f4fa5ca8ec7aaacc5330afd69ae09d090f4995e52c`，569 行）。原登记的 Q1–Q17 与 r2 登记的 N1–N15 这一趟不重判、不重跑。

## 一、问题

英文名：gc_batch_wall_clock_split

（照抄原登记第一节第 7 行；装置仍是 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`，不另起。）

**主 agent 给的问题**（派发提示「关的是哪几条」逐字）：

```markdown
要回答的问题：七·2 设计轮第一轮判决 `research/prompts/m4-gpugc-design-r1-main-verification.md`「二、判定与去向」开头那一段与「三、下一步」第 1 条点名的缺口，对应岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 1、3、6 行：
1. 一次 GPU 提交在主机线程上花的 CPU（c_rt）：准备缓冲、提交、等结果、读回、两份核各算一次再比对，全算在内。按提交大小（128 KiB、256 KiB、512 KiB、1 MiB，另加一档 ≥ 16 MiB）、两张卡、阻塞等与睡眠等两种等法各量。读法写死：用线程 CPU 钟，不用挂钟。岔路单第 1 行的够判条件是「在 E185 的 12 个点上与攻方模型 g1 的门槛逐点比」，门槛在 `research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out`。
2. 把 c_rt 拟合成「固定部分 + 每字节部分」（岔路单第 3 行）。
3. 改身份索引之后一批 CPU 按函数拆开：读这一版、克隆、发布、区域扫描各占几成（判决第三节第 3 条与正推腿 G1-d）。
4. 真卡上单元级重算之后仍不一致的概率（岔路单第 6 行），两张卡、≥ 10⁶ 个单元次，写明零次时的上界。
```

**岔路单逐行抄**（`research/prompts/m4-gpugc-design-r1-forks.md` 第 3–15 行，原样；文件 sha256 `43b667a1de724126a73745f079c68bc6c77799d5c13036fc0aa4212e726b9385`）：

```markdown
判决 `research/prompts/m4-gpugc-design-r1-main-verification.md` 列出的岔路，一行一条：候选、翻面观测、够判条件、状态。不写倾向、不写已有的数。这一轮都还开着：每一行都缺一次 GPU 提交在主机线程上的 CPU（下称 c_rt）这个代价数，先建实验量。

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | GPU 上算哪几段：A 只放扫描里的校验和；B A 加选段打分；C 甲、乙、丙全放；D 只换整理发布里释放前读盘核那一段 | 每个候选一次调用省下的主机 CPU，与同一次提交的 c_rt 比 | c_rt 按提交大小（128 KiB–1 MiB 与一档 ≥ 16 MiB）、两张卡、阻塞等与睡眠等各量到，在 E185 的 12 个点上逐点比 | 开着 |
| 2 | GC 跑在哪：甲 留在用户改动的调用线程里（今天的形态）；乙 挪到另一条线程或另一个调用入口，批与批之间看前台 | GPU 往返是否落进用户改动的返回路径；「前台有 fsync 在等」这一判别子能不能观测到 | 两种形态下用户改动返回的挂钟分布与 fsync 等待分布各量一次（E185 第四段 Q17 的口径，P 多取几档） | 开着 |
| 3 | 攒批：甲 一批一交；乙-1 同一条意图内一次交；乙-2 跨意图预取几个候选段；丙 放宽 R（一条意图罩住多个聚簇段，改盘上槽数字段） | c_rt 拟合成「固定部分 + 每字节部分」之后，固定部分大到压过乙-1 的上界才轮到乙-2 或丙；乙-2 要不要放开每批重扫、至多一条活意图、一趟发布预算三条约束 | 第 1 行的 c_rt 量齐并拟合；用户对三条约束的答复 | 开着 |
| 4 | 每批的重验：甲 每批重验字节（今天）；乙 窗口开头验一次字节、每批只重扫状态 | 一个单元的字节在两批之间翻一位的注入下，两条路的结局是否都合 D19 已定项 5 的硬规则 | 注入跑过，两条路的结局逐项写出 | 开着 |
| 5 | 两份核的独立：甲 只不共用核函数（共用输入准备、驱动、编译器）；乙 输入也各备一份；丙 各一份且分在两张卡 | 「两种模式各读这一版」的对拍在带快照的流上红不红；乙、丙多出的主机 CPU 与第 7 条余量比 | 七·3 的对拍改成从这一版起算、流里带快照跑一遍；c_rt 量到 | 开着 |
| 6 | N：取 1 ／ 取 2 ／ 更大；不确定计数住内存还是住盘上（重挂清不清零） | 真卡上单元级重算之后仍不一致的概率；确定性不一致出不出现 | 两张卡上 ≥ 10⁶ 个单元次的重算，写明零次时的上界 | 开着 |
| 7 | 「这一批不回收、不搬」的读法：A 整批不搬 ／ B 只那几个条目不搬 ／ C 批内就地对那几个条目重算再判 | 一轮重算的代价；有条目一直不确定时意图能不能完成 | c_rt 量到之后三种读法下各量一次 | 开着 |
| 8 | 「全部显卡」：只认已验厂商的独显 ／ 任何独显 ／ 含集显与软件适配器；有卡起不来时拒绝挂载 ／ 跳过并报告 | 软件 Vulkan 适配器会不会被当成一张卡；同一台机器另有进程占卡时用户期望挂不上还是照常挂 | 在装着软件 Vulkan 的本机列一次适配器；用户定第 4 条的射程 | 开着 |
| 9 | 三个默认值：写带宽上限（不限 ／ 按一批写字节定）；定时兜底间隔（0 关 ／ 按 E10、C412 定）；显存上限不给时（能用多少用多少 ／ 按用途上限取窗口） | 兜底在闲置文件系统上能再发几次（发布预算 `3S − 4`）；写带宽上限压不压得住一批由 CPU 定的挂钟 | 第 2 行定了之后量 Q17；C412 的挂钟代价量到 | 开着 |
```

**这一趟补哪几行**：岔路单第 1 行（R1–R3）、第 3 行（R4、R5）、第 6 行（R6–R8）。主 agent 的第 3 问（一批 CPU 按函数拆）对不上岔路单任何一行（第六节「附带」那一格写为什么），照设计员定义标「附带，够判后不跑」，登记在第三段；主 agent 给岔路单补一行（例如照正推腿 G1.6 表的 G1-d 那一行）之后它才算正式量（第一节未定项 1）。第 2、4、5、7、8、9 行这一趟不补；第 5 行的「两份核共用输入 ／ 各备一份」第 1 行的读数里两种都量（第五节），第 5 行本身怎么判不归这一趟。

**读法写死**：

| 词 | 口径 |
|---|---|
| 一次 GPU 提交 | 主机线程上的一次往返：① 准备：把这一次的 n 个单元的字节（调用方已在内存里，同区域扫描读盘之后的样子）写进卡上的输入缓冲（`wgpu::Queue::write_buffer`），连同长度表与派活参数；② 编一个命令缓冲：核一、核二各派一次（每个单元一条线程），两份结果拷进同一块回读缓冲；③ 提交（`queue.submit`，一次）；④ 回读映射登记（`map_async`）；⑤ 等（下一格「等法」）；⑥ 读回两份结果、核每条结果的序号、逐条比两份（相同记「确认」，不同记「不确定」），再逐条与调用方手里的期望值比（同产品里与单元头里存着的校验和比，期望值在计时窗口外由 CPU 算好）；⑦ 解映射。①–⑦ 全在计时窗口里 |
| c_rt | 一次 GPU 提交在发起它的那条线程上花的 CPU：①–⑦ 前后各读一次本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` 之差（原登记 12.2 的同一个钟）。挂钟（`CLOCK_MONOTONIC_RAW`，原登记 12.4）包着它，只用于包含自检与物理下限，不进判定 |
| c_rt 的进程钟版 | 同一区间 `CLOCK_PROCESS_CPUTIME_ID` 之差：含驱动与 wgpu 在本进程里起的别的线程。只用于第十条 J5，不替代 c_rt |
| 等那一段 | ⑤ 单独再包一层线程钟与挂钟：`c_wait`、`t_wait`；`c_rt − c_wait` 记作 c_floor（与等法、核快慢无关的那一部分） |
| 等法 | 阻塞等：`device.poll(wgpu::PollType::Wait { submission_index: Some(这一次), timeout: None })`，与 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:881-884` 同一个调用。睡眠等：循环「`device.poll(wgpu::PollType::Poll)`；回读回调到了就出；没到 `std::thread::sleep(间隔)`」，间隔取 100 µs 与 1 ms 两档（第八节；没有条款给这个数，第一节未定项 3），每次另报 poll 了几次 |
| 核一、核二 | 两份独立写的 CRC-32C 核，不共用核函数（D26 已定项 13 第 8 条）。核一：装置照 `gpu_unit_checks.rs:80-118` 的逐位算法另抄一份 WGSL（那份源码是私有常量，装置读不到；行为照第十一节停机 J-S1 与 crates 的 `GpuCard::crc32c_of_ranges` 逐条对拍）；核二：装置新写，按字节查 256 项表（表在工作组共享内存里）。两份每个单元一条线程、工作组 64 |
| 输入形态 | 共用：两核读同一块输入缓冲（岔路单第 5 行甲）；分开：每核一块、各写一次（第 5 行乙）。第 5 行丙（分在两张卡）不量 |
| 常驻写法 | 管线、缓冲（按这一趟最大一档开）、绑定组每张卡建一次，在计时窗口外；每次提交只做上面 ①–⑦。这是支持 GPU 模式的人认的写法（正推腿 G1.7 那一段指出今天 crates 的写法每次都建缓冲） |
| 今天的写法 | `GpuCard::crc32c_of_ranges` 调一次（一份核、每次建五个缓冲与绑定组、阻塞等，`gpu_unit_checks.rs:774-911`），同一个钟计。真实基线，第五节 |
| 提交大小 n | 单元数，每单元 32768 字节：4、8、12、16、20、24、32、512（128 KiB、256 KiB、384 KiB、512 KiB、640 KiB、768 KiB、1 MiB、16 MiB）。主 agent 给的五档全在里面；384、640、768 KiB 三档是 12 个点上真会出现的大小（第四节第 6 行），量到了就不用插值 |
| 间隔 | 两次提交之间计时窗口外睡多久：0（连发）与 50 ms（产品里两批之间隔着一批的 CPU，推的）；50 ms 只量共用输入的三种等法（第八节） |
| 12 个点 | E185 第 2 次重跑第二段的 (f, P, K)：f ∈ {0.25, 0.75}、P ∈ {1024, 8192}、K ∈ {4, 8, 16}，与 `out/g1.out` 的 12 行一一对应 |
| 一个点的提交序列 | 一条意图签发那一次加每批各一次，每次交那一刻 R 里还剩的单元（每批之前重扫整个 R，原登记第七节乙类第 357–358 行同一个前提）：f = 0.25（m = 8）：K = 4 → (8, 8, 4)，K = 8、16 → (8, 8)；f = 0.75（m = 24）：K = 4 → (24, 24, 20, 16, 12, 8, 4)，K = 8 → (24, 24, 16, 8)，K = 16 → (24, 24, 8)。与 P 无关。候选 D（只换释放前读盘核）每批一次、交这一批搬的单元两块盘各一份：f = 0.25：K = 4 → (8, 8)，K = 8、16 → (16)；f = 0.75：K = 4 → 六个 8，K = 8 → 三个 16，K = 16 → (32, 16) |
| 一个点的 c_rt | 那个点的提交序列里每次提交取它那一档 n 的 c_rt，取平均（按一条意图的总 CPU ÷ 提交次数，与 g1 的门槛同一个分母：`g1_cpu_saved_ledger.py` 第 35、46 行）；一档 n 的 c_rt 取那一格那一轮全部计时提交的中位 |
| 门槛 T（g1） | `out/g1.out` 那一点的 `break_even_host_cpu_per_submission_ms`：甲、乙、丙全部交 GPU、主机一点不花时一条意图能省的上界 ÷ 提交次数。它是候选 C 的门槛，也是任何候选的上界 |
| 候选门槛 T_A、T_B、T_D | 装置照 `g1_cpu_saved_ledger.py` 第 13–47 行逐句的取法（不滤 `valid`、按 (fraction, pool_data_units, batch_units, round) 分组、`kind=issue` 那一行算签发、其余算批、各轮取中位）从 r2 第二段产物现算：A 只省 `crc_cpu_nanoseconds`（签发加各批）；B 是 A 加签发那一行的 `choose_cpu_nanoseconds`；C 照 g1 原式（装置也算一遍，与 `g1.out` 逐点比到小数第 4 位，第十一节停机 J-S3）；D 只省各批的 `release_check_crc_cpu_nanoseconds`，提交次数 = 批数。A、B、C 的提交次数 = 批数 + 1 |
| 机器换算 | `g1.out` 与 r2 第二段量 CPU 的是第二台，GPU 在本机。装置在段开头照 r2 登记 N2 做 21 遍 16 MiB CRC-32C 校准取中位，÷ r2 第二段产物 `name=calibration_baseline segment=stage_split position=start` 那一行的 `median_nanoseconds`，得 ρ_机器；T × ρ_机器 是「换到本机」的门槛，与 T 各判一行（第一节未定项 2） |
| 单元次（第 6 行） | 一个单元被核一、核二各算一次、比一次。天然单元次不含注入阳性对照的那几个单元 |

未定项：

1. **第 3 问没有岔路行**：正推腿 G1.6 表 G1-d 那一行（「要不要先补一条 CPU 侧测量：改身份索引之后一批 CPU 按函数拆开」）是正推腿自己的岔路，主 agent 的岔路单 `m4-gpugc-design-r1-forks.md` 没收它；按岔路单，它决定不了第 1 行（第 7 条按省下的 CPU 绝对值判，不按份额，第六节「附带」那一格）。主 agent 认「附带、够判后不跑」，或给岔路单补一行再派第三段。
2. **门槛用哪一台的 CPU**：派发点名的门槛是 g1 的（第二台的 CPU 量出来的省下），c_rt 只能在有卡的本机量。两种都判（R1、R3、R5 各两行，`threshold_kind=attack_ledger` 与 `machine_scaled`）；两种判定不同的点，第 1 行在那一点读哪一种由主 agent 定。ρ_机器 只用 CRC 那份活校准，乙的映射反查是吃内存的活，换算对它是推的。
3. **三个装置旋钮没有条款给值**：睡眠间隔（100 µs、1 ms）、提交间隔（0、50 ms）、核二的写法（按字节查表）。前两个各取方向相反的两点（第八节），核二的写法只影响等那一段的长短，c_floor 与它无关（第六节 R2）。
4. **第 5 行甲、乙两种输入形态都量**：哪一种算第 1 行的读数，等第 5 行定；两种判定不同的点写「随输入形态翻面」。

主 agent 定（2026-10-23，装置写之前）：

1. 第 3 问要量：岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 补第 10 行（区域扫描按身份查映射条目并不并进七·3：并 ／ 不并；翻面观测是按函数拆开的一批 CPU 里区域扫描占几成），它是 `.claude/kb/milestone/04-fourth-txn.md` 第 174 行那条延后项的进入条件。第三段照第五节写法在第二台跑，不再标「附带、够判后不跑」。
2. 门槛：第 1 行读 `threshold_kind=machine_scaled`（换到本机 CPU 的门槛），`attack_ledger` 那一行并列报；两种判定不同的点照 J10 写「随机器翻面」，那一点第 1 行不当已判。
3. 三个装置旋钮照第八节的取值认下。
4. 第 5 行两种输入形态都量，照写。

## 二、被测条款与它引的定义

两段，`scripts/kb/quote-kb.py` 整段抄（出口 `/home/fy5090/.local/share/singlefs/work/m4-e185-r3-designer/clauses.md`，回读逐字节一致，再整份追加到这里）：派发点名的里程碑四作业点表七·2 那一行（今天在第 144 行），与 D26 已定项 13 整节（派发点名第 7、8 条；第 7 条是 c_rt 的口径「后者含准备数据、提交、等结果、两份结果的比对」，第 8 条是两份核、不确定的留到下一轮、连续 N 轮交 CPU）。D24 已定项 2、3 这一趟不判，照 r2 登记第二节。

**出处 `.claude/kb/milestone/04-fourth-txn.md:144-144`（整段抄，未转述）**

```markdown
| 七·2 | 设计轮：GPU 上算哪几段；攒批的形态（几批连起来交、还是放宽 R）与显存上限的关系；多张卡怎么分活；GPU 结果两份核比对的形态（第二份核怎么写才算独立；一张卡时两份都在这张卡上跑，几张卡时怎么分，不许要求多张卡；比对放在 GPU 上还是 CPU 上）与连续几轮不确定才交 CPU 判的 N（D26（后台整理与放置回收） 已定项 13 第 8 条）；控制接口（挂着切换）的形态；定时兜底间隔、写带宽上限、显存上限的默认值；GPU 代码住哪（推的：单开一个可选的包，core 不带它也能编，守 D24（后台重活能不能卸给 GPU） 已定项 2 第 1 条）；GPU 路径默认 WGSL 经 `wgpu`（现成，checker 档的崩溃判器在用），有更好的做法在七·2 的设计轮比过再换 | D26（后台整理与放置回收） 已定项 13；D24（后台重活能不能卸给 GPU） 已定项 2 | 判决落 `research/prompts/`，定案写回 kb；有实验才有决策 | 七·1 |
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

## 三、实现今天的样子

现查于 2026-10-23，主工作区，文件加行号（设计员的观测，不是结论）。

| 一步 | 在哪 | 看到的事实 |
|---|---|---|
| 产品里的 GPU 代码 | `grep -rn wgpu crates/singlefs-core \| wc -l` | 0：产品路径上没有 GPU 代码，D26 已定项 13 第 7、8 条描述的 GPU 模式还没有实现 |
| 仓里唯一的 GPU CRC 核 | `grep -rn 'CRC32C_REFLECTED_POLYNOMIAL\|0x82F63B78' crates/ --include=*.rs` | 只命中 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:87`、`:107`（逐位算法，`:80-118`，每段一条线程、工作组 64）与 `crates/singlefs-checker/src/lib.rs:56` 的注释（那是池级 checker 的 CPU 查表实现，不是 GPU 核）。第 8 条要的第二份 GPU 核仓里没有 |
| 一次调用今天怎么做 | `gpu_unit_checks.rs:774-911` | `GpuCard::crc32c_of_ranges`：主机上分一块 `count × 32 KiB` 的缓冲并拷贝（`:779-789`）；每次新建四个带初值的缓冲（`:798`、`:806`、`:814`、`:823`）、一个回读缓冲（`:831`）、一个绑定组（`:838`）、一个命令编码器（`:860`）；`queue.submit`（`:876`）、`map_async`（`:879`）、`device.poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: None })` 阻塞等（`:881-884`）；读回时核每条结果的序号（`:898-907`） |
| 起卡 | `gpu_unit_checks.rs:508-569`、`:575-621` | 按卡支持的上限要 limits（`:521`）、`MemoryHints::Performance`（`:523`）；`gpu_cards_by_priority(&[(序号, 额度)])` 只起点名的卡，空闲显存低于 `MINIMUM_FREE_MEMORY_MEBIBYTES`（2048，`:71`）的不起。`GpuCard::device()`、`queue()` 公开（`:723`、`:729`），装置能在同一个设备上另建管线与缓冲 |
| 一次调用的显存 | `gpu_unit_checks.rs:669-673` | `bytes_crc32c_of_ranges_needs(n) = n × (32768 + 4 + 2 × 8) + 16` |
| wgpu | `Cargo.lock:1091-1092`；`crates/singlefs-checker-tier/Cargo.toml:27`、`:40` | `wgpu` 30.0.1，Vulkan；checker 档特性 `gpu` |
| 计数特性 | `crates/singlefs-core/Cargo.toml:22` | `lookup-work-counters`：r1 登记作废条款「计数特性被并进来」盯的那一个 |
| GC 一批在调用线程上 | `crates/singlefs-core/src/mounted_session.rs:1928`、`:2989` | 用户改动的发布在返回之前调 `run_compaction_on`；`advance_compaction_by_one_publish` 推一次发布 |
| 克隆 | `mounted_session.rs:3003-3005`；`:3165`、`:3267` | 每推一次先 `current.version_of_trees()…clone()` 整份复制现行那一版；发成之后 `PoolVersion::of_a_version_of_trees(published.clone())` 再复制一份发成的那一版 |
| 读这一版 | `mounted_session.rs:2807-2833`（公开）、`:2835`（私有）、`:3026` | `central_mapping_as_the_compaction_reads_it` 是 `pub fn`，七·6 之后按叶读、每把 key 查一次；`version_as_the_compaction_reads_it` 私有，另加钉住槽那一圈 |
| 区域扫描 | `crates/singlefs-core/src/region_scan.rs:204`、`:286-289`、`:309-311`；`mounted_session.rs:3029` | `scan_region` 仍对全部映射条目线性找两遍 |
| 发布 | `mounted_session.rs:2895`（私有）；`crates/singlefs-core/src/transaction.rs:9675`（`pub fn publish_relocation_batch`） | 装置不单独重放发布（第五节第三段「发布与其余」取余数） |
| CPU 那份 CRC | `crates/singlefs-core/src/checksum.rs:43` | `crc32_castagnoli`，软件实现 |

**装置今天的样子**（改前基线）：主工作区 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` sha256 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`、13187 行、`#[test]` 118 条，即 r2 登记第十二节「打进主工作区之后，以这一份为准」那一版；HEAD 那一版是 `b3a50d50…`（10223 行），差的是 r2 的改动，还没提交。`crates/mutations.tsv` 里 `E185 V` 起头 94 行。

| 装置里的一处 | 行 | 看到的 |
|---|---|---|
| 第三段 | `mod gpu_submission` 第 8727–9176 行；分派第 9655 行 | 只计挂钟（`time_the_submissions` 第 8814–8843 行，`Instant`），不读线程钟；只调 `crc32c_of_ranges` 一份核 |
| 线程钟 | `thread_cpu_nanoseconds` 第 706 行（clock id 3，第 688 行） | 现成，新子命令照用 |
| 计时窗口核线程数 | `enter_the_timing_window` 第 5647–5650 行 | 核本进程只剩一条线程；起了 Vulkan 设备之后驱动有自己的线程，新的两个 GPU 子命令不能照搬这一条（第五节改成「装置自己不起线程」） |
| 校准 | `calibration_baseline` 第 4080 行；产物行第 4602–4615 行 | `name=calibration_baseline segment=… position=… runs=21 median_nanoseconds=…` |
| 映射条目取法 | `version_as_the_replay_reads_it` 第 2582 行 | 装置自己那一份；七·6 之后 crates 的 `central_mapping_as_the_compaction_reads_it` 已公开，第三段两份对拍（停机 J-S6） |
| 子命令 | 第 9617–9679 行 | `wall-clock`、`geometry-check`、`stage-split`、`gpu-submission`、`foreground-fsync` |

`replay.sh` 里没有 E185 的复跑行（`grep -n 'e185\|E185' scripts/experiment-apparatus/replay.sh` 零命中）。

与条款对不上的：第 8 条的第二份 GPU 核仓里没有（上表第 2 行）；这一趟装置自己写一份，不改 crates。

## 四、跑之前已经存在的数

派发提示点名要读判决、岔路单、`out/g1.out` 与两条腿给定义的那几节，所以下面这些数在写判据之前就读到了。门槛 T 本身是派发点名的（g1），设计员没有挑；设计员自己定的几条线（第六节「门槛旁边写它从哪来」）逐条写了它看没看过这些数。

| # | 数 | 从哪读到 | 对判据的影响 |
|---|---|---|---|
| 1 | `out/g1.out` 全文 14 行：12 个点的 `intent_cpu_ms` 22.875–694.101、`saved_upper_ms` 0.591–3.961、`saved_share` 0.0037–0.0312、`submissions_per_intent`（3、2、2、3、2、2、7、4、3、7、4、3）、`break_even_host_cpu_per_submission_ms` 0.2159–0.7214、`break_even_over_a_card0/1`、`release_check_crc_ms` 0.165–0.496；汇总行 `points=12 break_even_ms_min=0.2159 break_even_ms_max=0.7214 a_card0_ms=4.492 a_card1_ms=10.300` | `research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out` 第 1–14 行（sha256 `61b210296634f2f20fa0465a4e4f09859cd9236522c54d4fb18301e75f243025`） | 门槛 T 与 R5 用的 `saved_upper_ms` 就是它（派发点名）。`submissions_per_intent` 与第一节「一个点的提交序列」逐点相同，是第七节乙类锚点 |
| 2 | 判决里的数：卡 0、卡 1 的 `intercept_seconds` 0.004492、0.010300；g4 `differing_entries=0` 与 `=3`；g2 `runs_needed_at_least=8`–`38`；g6 一批 `wall_ms_median=23.728`–`123.369`；正推的 4.59%、0.54 ms、φ ≥ 0.89、70–94 MiB；r2 第二段产物第 989 行 `call_cpu_nanoseconds=13894421 crc_cpu_nanoseconds=259441.000000 … scan_increment_cpu_nanoseconds=77751.000000 … release_check_crc_cpu_nanoseconds=344572.000000` 与现算的 4.9067% | `research/prompts/m4-gpugc-design-r1-main-verification.md` 第 20–22、26–29 行 | 截距是今天写法（每次建缓冲）的挂钟，不是 c_rt；它让「阻塞等若在驱动里转，c_wait 有几毫秒」成了要量的一支（第 9 行）。别的数不进判据 |
| 3 | 正推腿：第 207 行（丙每整段 70.05 ns、推到 1 TiB 每次 73.0–77.2 ms、64125 段）、第 233 行 G1-a、第 234 行 G1-b（P = 1024 +34%–36%、+67%–72%，P = 8192 约 +4%）、第 236 行 G1-d（「现 ≤ 4.59%」）、第 240 行 G1.7、第 316 行（φ ≥ 0.89）、第 320 行 G2-a（「乙-1 的上界（每条意图 0.76–4.6 ms）」）、第 391、473（一批 CPU 13.4 ms 或 111.1 ms）、741、744 行 | `research/prompts/m4-gpugc-design-r1-sonnet-output.md`（`grep -n` 命中整行打出） | G1-a 是 c_rt 的定义来源；G2-a 那句「0.76–4.6 ms」是正推自己的口径，R5 不用它，用 g1 的 `saved_upper_ms`（派发点名的门槛文件），两者口径不同照实并列 |
| 4 | 攻方腿：第 45 行（0.216–0.721 ms、卡 0 的 4.8%–16.1%、卡 1 的 2.1%–7.0%）、第 63–85 行（g1 那张表与「0.37%–3.12%、0.591–3.961 ms」）、第 112 行（P 从 1024 到 8192 `intent_cpu_ms` 涨 8.02–8.20 倍）、第 113 行（释放前核每条意图 0.165–0.495 ms）、第 135、137、208（E185 页 56 行 `merge_batches`）、219 行 | `research/prompts/m4-gpugc-design-r1-opus-output.md`（同上） | 第 135 行是第 1 行够判条件的来源（「两张卡、批字节 16 KiB–1 MiB、阻塞等与睡眠等两种等法，各量提交前后的线程 CPU（含准备与比对）」）；数不进门槛 |
| 5 | E185 页第 1 行标题（第 2 次重跑的状态）与第 275–330 行（「做了什么」「复跑」「口径」）：Q4 12 对的中位 1.015102–1.074115；两段 140 + 60 条意图 `invalid_reason=none`、`stop_clause_findings=0` | `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` | 是第 2 次重跑的结果，读到了；这一趟不判 Q1–Q17，不影响 |
| 6 | 自己算的：12 个点的提交序列（第一节）与每次提交的平均字节：f = 0.25：K = 4 → 218453.3，K = 8、16 → 262144；f = 0.75：K = 4 → 505563.4，K = 8 → 589824，K = 16 → 611669.3；候选 D 的序列（第一节）；提交次数与 g1 逐点相同 | 第十三节 python 命令 | 第七节乙类锚点；n 的取值（第一节）照它补了 384、640、768 KiB 三档 |
| 7 | 自己算的：零次时单侧 95% 上界 `1 − 0.05^(1/n)`：n = 1048560 → 2.856992e-06（3/n 是 2.861067e-06）；n = 2097120 → 1.428497e-06；PCIe 链路峰值 5.0 x16 63015384615 B/s、4.0 x8 15753846154 B/s，512 MiB 至少 0.00852 s、0.03408 s | 同上 | R6 的上界式与第七节锚点 |
| 8 | 本机两张卡：`nvidia-smi` 序号 0 是 RTX 5090（PCIe 5 x16，32607 MiB），序号 1 是 RTX 5060 Ti（PCIe 4 x8，16311 MiB），此刻各用 2 MiB | 第十三节命令 | 原登记第四节写的 x8 峰值按 5.0 代算；5060 Ti 报的是 4.0 x8，装置照 `nvidia-smi` 现查（原装置 `pcie_links_reported_by_nvidia_smi` 就这样做），不影响判据 |
| 9 | 推的（没量）：常驻写法下除等之外的主机 CPU（c_floor）：写入 128 KiB–768 KiB 按 10–20 GB/s 拷贝约 7–79 µs，编命令、提交、映射登记与读回按 wgpu 的校验约几十到一百多微秒，合计约 0.03–0.2 ms，与 T（0.2159–0.7214 ms）同一个量级；阻塞等若在驱动里转，c_wait ≈ t_wait，按核一每线程逐位算 32 KiB 约 0.5 ms 起到今天写法截距的几毫秒，远大于 T；若睡着，c_wait 几十微秒；睡眠等 1 ms 约 poll 2–6 次、100 µs 约 10–50 次，每次 poll 的 CPU 不知道（推 5–50 µs） | 设计员推的，依据是第 2 行的截距与 `gpu_unit_checks.rs` 的核写法 | 这就是 3a 第 ② 步要的「被测臂上预期的差」：各臂预期值都正，跨着 T，判据是绝对值比较（c_rt < T），不在任何一臂上退化成零分母；线程钟一次前进 170 ns 中位（原登记 12.2 实测），比要分的差（微秒以上）细三个数量级以上 |
| 10 | 原登记、r1 登记、r2 登记全文里的数（各自第四节与 12.9、修订一读到的产物行、校准代用量、锚点 `0x3430be3b`、`0x84b65aa5`、9437184） | 三份登记 | 机器换算照用 r2 的校准做法与锚点 `0x3430be3b`；别的不进这一趟的判据 |
| 11 | 多机配置的两台与清场命令的键（`--emit-hosts` 打出的那几行；私有值不抄） | 第十三节命令 | 只用来写第十四节「两段 GPU 在本机、附带段在第二台」 |

设计员自己定的线（第六节）与上面的关系：第 9 行推的量级让 n 的取值与等法的两档间隔有了依据；50 ms 的提交间隔取在第 1 行 `intent_cpu_ms ÷ submissions_per_intent`（11.44–99.16 ms，第十三节现算）与第 3 行「一批 CPU 13.4 ms 或 111.1 ms」之间的低端，读过这些数之后定的；J5 的 10%、R4 的 20%、阳性对照的 [0.9, 1.1] 与 0.1 没有引用上面任何一个数。

## 五、臂、阳性对照、真实基线

### 装置写在哪（写死）

**入库装置**：仍是 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`，加三个子命令，原有五个子命令、原有产物行、原有判据与 V1–V94 不动。理由：第 1、3、6 行问的是这台机器上 `wgpu` 30.0.1 经 checker 档起卡（`gpu_unit_checks.rs` 的 `gpu_cards_by_priority`）之后一次提交在主机上要花多少 CPU、真卡算错的概率，只能在 crates 现有的 GPU 栈上量；附带那一段问的是 crates 这份 GC 代码一批 CPU 的拆分，要直接调 crates 的函数。**改前基线取主工作区今天那一版 sha256 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`**（第三节）。改完、单测与变异跑完、第一份产物之前，执行员把改完的 sha256 写进第十二节；每一趟输入清单第一行与产物 `name=apparatus` 行（r2 修订 R2-8 那一行）各记一次，三处不同就停（停机 J-S7）。

装置读 crates 的东西照原登记第五节常量表回比（SLOT、数据单元、`bytes_crc32c_of_ranges_needs` 三点等，原样留着）；这一趟新读的只有两样，都不引常量、按行为回比：

| 读什么 | 怎么回比 | 对不上 |
|---|---|---|
| 核一的算法（`gpu_unit_checks.rs:80-118`，私有常量） | 每张卡起好之后，同一份输入（n = 8 与 512，第一节的单元字节）同时交 crates 的 `GpuCard::crc32c_of_ranges` 与装置的核一，逐条相同 | 停机 J-S1 |
| `central_mapping_as_the_compaction_reads_it`（附带段用，公开函数） | 每批在计时窗口外，crates 那一份与装置的 `version_as_the_replay_reads_it` 取出的映射条目集合相同 | 停机 J-S6 |

### 三段（按岔路排序；每段交回之后主 agent 对着岔路单判续不续）

| 段 | 子命令 | 在哪 | 补岔路单哪一行 | 补完那一行够不够判 |
|---|---|---|---|---|
| 第一段 | `host-cpu-per-submission --cards 0,1 --stage-split-product <r2 第二段产物> --attack-ledger <g1.out>`（`gpu` 特性；没编特性退 78 报「本次未跑」，同原 `gpu-submission`） | 本机，两张卡先后量 | 第 1 行（R1–R3）、第 3 行（R4、R5） | 第 1 行：12 个点 × 两张卡 × 三种等法都有判定（含「不稳定」）就够判；第 3 行：拟合与每点的攒批类别都有了，另欠用户对三条约束的答复（岔路单原文），那一半不归实验 |
| 第二段 | `recompute-disagreement --cards 0,1`（`gpu` 特性） | 本机，两张卡先后算 | 第 6 行（R6–R8） | 每张卡两种输入形态各 1048560 个天然单元次跑完、阳性对照过，就够判 |
| 第三段（附带，够判后不跑） | `function-split --point 0.25:1024 --point 0.25:8192 --point 0.75:1024 --point 0.75:8192`（不带 `gpu` 特性） | 第二台（同 r2 第二段，真盘不用） | 不对应岔路单任何一行（第一节未定项 1） | 不适用；主 agent 给岔路单补行才跑 |

第一、二段同一次编译（带 `gpu` 特性）、先后跑、不同时跑；第二段的内容生成与 CPU 参照用 31 条线程，会搅第一段的计时。第三段与前两段不在同一台，可以与它们同时跑（计时窗口各在各的机器上）。

### 第 1 行的臂（「怎么做」推得出「做完会怎样」：c_rt 由 ①–⑦ 那几步的主机 CPU 相加，各臂改的是其中哪一步、改多少）

| 臂 | 怎么做 | 做完会怎样 |
|---|---|---|
| 不赔（支持 GPU 模式的人认的样子） | 常驻写法：管线、缓冲、绑定组一次建好；每次只把字节写进去、派两次、交一次、等、读回比对；等的时候线程睡着（睡眠等，或驱动的阻塞等本身就睡）；共用一块输入 | 每次提交的主机 CPU 只剩拷字节与几次 API 调用，在 12 个点上低于那一点省下的（T） |
| 赔（支持 CPU 模式的人认的样子） | 同一个常驻写法，但两核各一块输入（多一次拷贝）、驱动的阻塞等在主机上转着等、两批之间 GPU 闲下来（间隔 50 ms）再交时更慢 | 等那一段的 CPU 与 GPU 一次往返的挂钟同量级，每次提交的主机 CPU 高于 T |

两条臂量的是同一组格（第一节的 n × 两张卡 × 三种等法 × 两种输入 × 两种间隔），哪一格落在哪一边是结果。c_floor 一列把「等法」从两条臂里拿掉：它高于 T 的点，两条臂一起判赔。

### 第 3 行的臂（R5 的四类）

| 臂 | 怎么做 | 做完会怎样 |
|---|---|---|
| 甲 一批一交 | 每批（与签发）各交一次 | 一个点的 c_rt < T |
| 乙-1 意图内一次交 | 窗口开头把 R 里 m 个单元交一次（m = 8 或 24），之后每批只重扫状态 | c_rt(m) < `saved_upper_ms` |
| 乙-2 ／ 丙 跨意图攒 | 攒 W 条意图交一次，固定部分摊成 c0 ÷ W | c1 × m × 32768 < `saved_upper_ms`，W ≥ ⌈c0 ÷ (`saved_upper_ms` − c1 × m × 32768)⌉ |
| 怎么攒都赔 | — | c1 × m × 32768 ≥ `saved_upper_ms`：每字节那一部分已经吃掉省下的 |

### 第 6 行的臂

| 臂 | 怎么做 | 做完会怎样 |
|---|---|---|
| N = 1（不确定就交 CPU） | 一次不一致就交 CPU 判 | 不一致若是确定性的（同一输入每次都不一致），等几轮也不会好，N = 1 少付几轮重算 |
| N ≥ 2（先重算） | 不一致的条目下一轮重算 | 不一致若是暂态的（重算一次就一致），N ≥ 2 让 GPU 自己消化，少交 CPU |

### 阳性对照（每条臂、每张卡都跑）

| # | 对哪几条臂 | 对照 | 判得出的样子 |
|---|---|---|---|
| P1 线程钟分得出转与睡 | 第 1 行全部 7 种提交形态（常驻 × 三种等法 × 两种输入，加今天的写法），每张卡，n = 8、间隔 0 | 计时区间开头（① 之前）插一段：(甲) 按线程钟自旋 2 ms；(乙) `std::thread::sleep(2 ms)`。每轮里基线、甲、乙各 30 次挨着量，次序按轮号轮换（同原登记 12.9），5 轮 | 甲：c_rt 增量中位在 2 ms 的 [0.9, 1.1] 倍；乙：c_rt 增量中位 ≤ 2 ms × 0.1，挂钟增量中位 ≥ 2 ms × 0.9 |
| P2 两核与 CPU 逐条相同 | 第 1 行全部形态、第 6 行两种输入，每张卡 | 9 字节「123456789」交两核；每一格每次提交的每条结果与计时窗口外 CPU 的 `crc32_castagnoli` 比 | 两核都回 `0xe3069283`；第 1 行每格每条都相同（第 6 行的天然不一致按 R6 计，不算 P2 红） |
| P3 比对分得出一条不同 | 第 1 行常驻两种输入，每张卡，n = 8、阻塞等 | 一次提交里把核二回读的第 3 条结果异或 1（在 ⑥ 读回之后、比对之前） | 那一次比对报恰好 1 条「不确定」、是第 3 条 |
| P4 没快过物理下限 | 第 1 行全部形态，每张卡 | 每次提交的挂钟 ≥ 这一次写进卡的字节 ÷ `nvidia-smi` 报的链路峰值（分开输入按两份字节算） | 没有一次快过 |
| P5 确定性不一致认得出 | 第 6 行两种输入，每张卡 | 第 1–8 批的第 7 个单元：分开输入时核二那一份输入第 100 字节异或 1；共用输入时每次（含重算）把核二回读的那一条异或 1 | 8 个全记「确定性」（3 次重算都不一致），不进天然计数 |
| P6 暂态不一致认得出 | 第 6 行两种输入，每张卡 | 第 1–8 批的第 11 个单元：只在头一次计算时把核二回读的那一条异或 1 | 8 个全记「暂态」（第 1 次重算就一致），不进天然计数 |
| P7 门槛读对了 | 第 1 行 | 装置照 g1 的取法从 r2 第二段产物算 T（候选 C） | 12 个点与 `g1.out` 逐点相同到小数第 4 位（不同走停机 J-S3） |
| P8 校准分得出两倍的活 | 第一、二段的机器换算与作废用的校准 | 段开头做一遍 32 MiB（同法），÷ 16 MiB 那 21 遍的中位（r2 修订 R2-3 同一个） | 落在 [1.8, 2.2]，CRC 是 `0x84b65aa5` |

附带段的阳性对照：原第二段甲、乙、丙的 2 ms 对照照跑；新加的克隆、读这一版两个重放各在计时区间里加 2 ms 自旋，增量在 2 ms 的 [0.9, 1.1] 倍。

### 真实基线

第 1 行：今天的写法（`GpuCard::crc32c_of_ranges` 调一次，一份核、每次建缓冲、阻塞等），同一组 n、两张卡、间隔 0，与常驻写法同一个线程钟。它是仓里现在真有的 GPU 提交，不是 GPU 模式（只有一份核），判定只报、不当第 1 行的读数。第 6 行：CPU 的 `crc32_castagnoli`（每个单元都比）。附带段：`CompactionSwitches::PRODUCT` 只换批大小（原登记第五节「真实基线」）。

### 第一段怎么量（`host-cpu-per-submission`）

- **段开头**：原有时钟分辨率自检（原登记 12.2）；校准基线 21 遍与 P8（r2 登记 N2、R2-3 的同一份活与锚点 `0x3430be3b`、`0x84b65aa5`）；读 r2 第二段产物算候选门槛（P7）与 ρ_机器；起卡之前核本进程只剩一条线程（原有）。
- **每张卡**：`gpu_cards_by_priority(&[(序号, 4096)])` 起一张；核一、核二编管线；按 n = 512 两份输入开常驻缓冲；P2 的 9 字节、停机 J-S1 的对拍；然后 5 轮。起卡之后驱动有自己的线程：每格前后报本进程线程数（`/proc/self/task` 的条数），装置自己在这一段不起任何线程（单测核：段里没有 `thread::spawn`、`thread::scope`）。
- **每轮**：全部格按轮号轮换起点（第 r 轮从第 r 格起，格的次序：间隔、输入、等法、n 由外到里），每格：前校准（16 MiB × 3 遍取中位，÷ 段开头基线）→ 10 次热身不计 → 计时提交（间隔 0 的格 100 次，间隔 50 ms 的格 50 次）→ 后校准。今天的写法那几格同样按这个次序插在里面。
- **每次计时提交**：读挂钟、线程钟、进程钟 → ①–⑦（⑤ 前后另读线程钟与挂钟，睡眠等另数 poll 次数）→ 读进程钟、线程钟、挂钟（挂钟包着两个 CPU 钟，同原登记 12.4 的嵌套）。间隔 50 ms 的睡在下一次读挂钟之前，不在区间里。
- **作废与重试**：前后校准比有一个 > 1.10，或这一格有一次线程钟读不到，那一格那一轮作废、照 r2 修订 R2-2 等常态再重量（单次至多 300 秒、一段累计 600 秒，满了退 5）；一格至多重量 3 次（`ATTEMPTS_PER_ROUND_AT_MOST`），三次都作废打 `name=round_invalid`，那一格那一轮不进判定（第十条 J9）。
- **算点**：每格每轮取计时提交的中位；一个点 × 一张卡 × 一种形态 × 一种等法 × 一种间隔，每轮按第一节「一个点的提交序列」把各档的中位取平均，得那一轮那一点的 c_rt 与 c_floor；5 轮各判一次。

### 第二段怎么量（`recompute-disagreement`）

- 每张卡 × 两种输入：64 批，每批 16384 个单元；单元字节由种子定：起点 x = `0x9E3779B97F4A7C15` 异或 ((卡序号 << 48) | (输入形态 << 40) | (批号 << 20) | 单元号)（输入形态共用 0、分开 1；批号从 1 起），xorshift64*（`x ^= x >> 12; x ^= x << 25; x ^= x >> 27`，输出 `x × 0x2545F4914F6CDD1D mod 2^64`）每步出 8 字节、小端，截到单元长度（第七节乙类锚点），偶数号单元 32768 字节、奇数号 16384 字节；内容与 CPU 参照在计时以外用 `E185_THREADS` 条线程造（没设取 `.claude/heavy-thread-default`）。
- 每批：写入、核一核二各派一次、提交、阻塞等、读回；每个单元比三样：核一对核二、核一对 CPU、核二对 CPU。三样有一样不同的单元单独重算（n = 1，同一种输入），至多 3 次，每次照样比三样。
- P5、P6 的注入只在第 1–8 批；注入的 16 个单元不进天然计数，天然单元次 = 64 × 16384 − 16 = 1048560（每张卡每种输入）。
- 64 批之后用同样的种子把第 1 批整批再算一遍，两份 GPU 结果逐条比（「同一输入同一结果」，R8 一并报）。
- 另有一批「两张卡同一输入」：卡序号一栏填 `0xFFFF`、批号 0、16384 个单元，两张卡各算一遍（共用输入），两张卡的结果逐条比、各与 CPU 比（第七节甲类；不进天然计数）。
- 这一段不计时，只报每批挂钟（给第十四节核预计）。

### 第三段怎么量（`function-split`，附带）

r2 第二段同一套造池、几何核、全局准备队列与校准（原登记 12.11、r2 登记 N1–N15），只量 K = 8、四个 (f, P)、5 轮。每次 `run_compaction` 调用返回之后、计时窗口之外，在调用之前那一版上重放：克隆调用之前那一版（`TransactionOutput::clone`，即 `mounted_session.rs:3003-3005`）、克隆调用之后发成的那一版（`:3267`）、`central_mapping_as_the_compaction_reads_it`（crates 的公开函数）、装置那一份钉住槽那一圈（`version_as_the_replay_reads_it` 里与 crates `:2839-2840` 起那一圈同法的部分）、`scan_region`（原第二段乙）、签发那一次的 `choose_region`（原第二段丙）；各先不计时跑一遍、再计时 5 次取中位。「发布与其余」= 这一次调用的线程钟 CPU − 上面各项之和（含计划、发布、discard、记账；取余数，不单独重放）。

### 跑法（照现行脚本写；主 agent 起跑前核这几份脚本有没有变，变了照新写法起，记进第十二节一行）

- **编**：第一、二段在本机，一次编：`cargo build --release -p singlefs-checker-tier --features gpu --bin e185_gc_batch_wall_clock_split`（不带 `--all-targets`、`--tests`，免得并进 `lookup-work-counters`；输入清单里的 `cargo tree` 那一段核）。编译目录归这一趟，跑完删（`scripts/hosts/run-build-directory.sh` 的做法）。第三段同 r2 第二段，在第二台另编（不带 `gpu` 特性）。
- **起**：执行员照 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 的形态写 `scripts/experiment-apparatus/e185-r3-segment-run.sh <段>`（写输入清单 → 编 → 跑那一段 → 产物写进 `research/results/` → 删编译目录）；带 GPU 的那一步在脚本里经 `bash scripts/crash/gpu-quiesce-run.sh -- …` 起（`.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」），清场回读没过退 78，那一段记「本次未跑」。
  - 第一、二段（本机，先后）：`bash scripts/hosts/run-with-memory-cap.sh 16G env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh host-cpu-per-submission`，跑完再起 `… recompute-disagreement`。
  - 第三段（第二台，附带）：照 r2 的形态 `bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap 46G --label e185-r3-function-split-<日期> --fetch research/results/<产物名>… -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r3-segment-run.sh function-split`。
- **上限与线程**：第一、二段内存上限 16G（推的：第二段一批 512 MiB 内容、分开输入时两份，加 CPU 参照与回读，峰值约 2 GiB）；`E185_THREADS` 不设，取登记值 `.claude/heavy-thread-default`（今天 31），只用于第二段造内容与 CPU 参照、第三段准备队列；编译经 `bash scripts/hosts/capped.sh 31`。显存额度每张卡 4096 MiB（同原第三段）。
- **单测与变异**：装置改完由主 agent 带提交前缀在主工作区跑（checker 档）：不带特性 `cargo test --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split`，带 `gpu` 特性的那几条经 `gpu-quiesce-run.sh` 另跑一趟；`crates/mutations.tsv` 里 `E185 V` 新加的那几行（第九节）。全绿、变异全抓之后才起第一段；条数写进第十二节。
- **次序**：第一段 → 第二段；第三段只在主 agent 定要之后起。
- **计时期间**：第一段起跑前主 agent 告诉别的会话不往本机两张卡派活（清场之外的那一半）；第一段每格前后的校准比与线程数就是这一段的「机器被别的活挤」读数。

### 输入清单（每一段一份，起跑前写，跑完不改）

`research/results/e185-r3-inputs-<段>-<日期>.txt`，逐行：装置 sha256（第一行）；`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-core/src/` 下 `mounted_session.rs`、`region_scan.rs`、`transaction.rs`、`checksum.rs` 与 `Cargo.lock` 的 sha256；`Cargo.lock` 里 `wgpu` 的版本；`nvidia-smi --query-gpu=index,name,driver_version,pcie.link.gen.max,pcie.link.width.max,memory.total --format=csv` 原样；`out/g1.out` 与 r2 第二段产物的 sha256（第一段）；`cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline` 原样；`nproc`；`getconf CLK_TCK`；内存上限与线程数。第二台的主机名、路径照 r2 的写法换成「〔第二台的目录〕」。

### 产物（带日期，不盖已有的；同一天同一段再跑加 `-2`、`-3`）

| 段 | 产物 | 补岔路单哪一行 |
|---|---|---|
| 第一段 | `research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-<日期>.out` | 第 1、3 行 |
| 第二段 | `research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-<日期>.out` | 第 6 行 |
| 第三段（附带） | `research/results/e185-gc-batch-wall-clock-split-function-split-r3-<日期>.out` | 不对应 |
| 每段 | `research/results/e185-r3-inputs-<段>-<日期>.txt`；`research/results/e185-r3-quiesce-<段>-<日期>.txt`（包装打到标准错误的 `GPU_QUIESCE` 四行，同 r2） | — |

分段不是「全部做完」的计划：第一段交回之后主 agent 对着岔路单判第 1、3 行；第二段交回之后判第 6 行；三行都不再开着就停，第三段标「够判后未跑」。

## 六、报哪些量与各自的判据

每个量一行、各报各的判定，不合成一个真假。第一段每一格 5 轮各判一次，5 轮相同写那个判定，不全相同写 `unstable`（真设备、真时序，`test-discipline.md`「单次观测不算数」）；某轮读不到写那一轮 `not_run`，不当 0。第二段是计数，每张卡每种输入一个数，不分轮：输入由种子定、设备是真的，「单次观测不算数」要的轮数在这里换成 2^20 个单元次，「同一输入同一结果」那一批（R8）证明没有隐藏状态。

**门槛旁边写它从哪来**：T、`saved_upper_ms` 是派发点名的 `out/g1.out`（攻方模型从 r2 第二段产物算的，D26 已定项 13 第 7 条那把尺：省下的主机 CPU）；T_A、T_B、T_D 是同一个算法换候选；它们都由 CPU 模式省下多少定，与第五节各臂「怎么做」（提交怎么写、怎么等、输入几份）无关，换哪一种提交形态门槛都不动，所以不是臂定义的同义反复。设计员自己定的线：R4 的 20%、J5 的 10%、作废的 1.10（r2 的政策线）、P1 的 [0.9, 1.1] 与 0.1，都没有引用第四节的数。

| 量 | 怎么算（产物行与键） | 门槛与判定 | 岔路单哪一行、翻面取值 | 可跑性 |
|---|---|---|---|---|
| R1 一个点的 c_rt 对 T | `name=host_cpu_per_submission_point fraction= pool_data_units= batch_units= candidate=c index= form= wait= gap_milliseconds= clock=thread threshold_kind=attack_ledger threshold_milliseconds= submission_units= host_cpu_per_submission_milliseconds_per_round= judgement_per_round= judgement=`；另一行 `threshold_kind=machine_scaled`（T × ρ_机器） | c_rt < T 判 `not_losing`；c_rt ≥ T 判 `losing` | 第 1 行。某点在某张卡、某种输入、某种间隔下三种等法都 `losing`：那一点 A、B、C 一起赔（T 是任何候选的上界），候选只剩 D 看 R3；有一种等法 `not_losing`：那一点 C 只到「下界不赔」（乙、丙的输入准备没量），A、B 之间看 R3。从 `not_losing` 变 `losing` 让那一点的选择从「GPU 模式（A/B/C）」翻到「不上 GPU 或只上 D」 | 第一段：每张卡 80 格（常驻 48 格 + 间隔 50 ms 24 格 + 今天的写法 8 格）、每轮 7600 次计时提交；单次 3–12 ms（推的：r1 截距 4.492、10.300 ms 是今天写法的上界，常驻写法少建缓冲），间隔格另加 50 ms；两张卡 5 轮约 950–1630 秒，加 P1 约 30–90 秒、校准约 30 秒，合计约 0.28–0.49 小时（第十三节现算）。分辨率：线程钟一次前进 170 ns 中位，T 在 0.2159–0.7214 ms |
| R2 一个点的 c_floor 对 T | 同 R1 那一行的 `floor_host_cpu_per_submission_milliseconds_per_round=`、`floor_judgement_per_round=`、`floor_judgement=` | 同 R1 | 第 1 行：c_floor `losing` 的点，赔与等法、核快慢都无关；R1 `losing` 而 R2 `not_losing` 的点，赔在等那一段，翻面看等法（第十条 J2） | 同 R1（同一批提交） |
| R3 候选 A、B、D 各对自己的门槛 | 同一行 `candidate=a\|b\|d`，`threshold_milliseconds` 取 T_A、T_B、T_D（第一节）；D 用第一节 D 的提交序列；`name=candidate_threshold` 每点每候选一行（`saved_per_intent_milliseconds`、`submissions_per_intent`、`threshold_milliseconds`） | 同 R1；B 的 `not_losing` 写成 `not_losing_lower_bound`（丙的输入是分配记录，展平它的 CPU 没量） | 第 1 行：A 与 D 两个方向都作数；某候选在某点从 `not_losing` 变 `losing` 让它在那一点出局 | 同 R1；门槛是算术，1 秒内 |
| R4 c_rt 的固定部分与每字节部分 | 每张卡 × 输入 × 等法 × 间隔：8 档 n 的 c_rt（各档 5 轮中位的中位）对字节最小二乘：`name=fixed_and_per_byte_fit index= form= wait= gap_milliseconds= fixed_milliseconds= per_mebibyte_milliseconds= maximum_relative_residual= points=8`，另报每轮各拟合一次的两个数 | 不单判，喂 R5；`maximum_relative_residual` > 20% 记 `fit_not_linear` | 第 3 行（c0、c1 本身是够判条件里的「拟合」） | 同一批提交；拟合是算术。分辨率：c1 由 128 KiB 到 16 MiB 的跨度定，16 MiB 那一档预计拷贝约 0.8–1.7 ms（推的，按 10–20 GB/s），远大于线程钟分辨率 |
| R5 一个点的攒批类别 | `name=merging_shape fraction= pool_data_units= batch_units= index= form= wait= gap_milliseconds= saved_per_intent_milliseconds= one_submission_per_intent_host_cpu_milliseconds= per_byte_part_milliseconds= intents_to_merge_at_least= category_per_round= category=` | 每轮按次序判：c_rt < T → `one_submission_per_batch`（甲）；否则 c_rt(m) < `saved_upper_ms` → `one_submission_per_intent`（乙-1，c_rt(m) 直接取 n = m 那一档量到的，不经拟合）；否则 c1 × m × 32768 < `saved_upper_ms` → `merge_across_intents`（乙-2 或丙），报 W* = ⌈c0 ÷ (`saved_upper_ms` − c1 × m × 32768)⌉；否则 `loses_however_merged`。后两类要 R4 不是 `fit_not_linear`，是的那几格写 `fit_not_linear` | 第 3 行：类别从甲变乙-1 是「一批一交」翻到「意图内一次交」；从乙-1 变 `merge_across_intents` 就是岔路单原文「固定部分大到压过乙-1 的上界才轮到乙-2 或丙」 | 同 R1 |
| R6 首轮天然不一致 | `name=disagreement_summary index= form= natural_unit_times= first_round_kernel_disagreements= first_round_rate= upper_bound_at_95_percent_when_zero=`；每批一行 `name=disagreement_batch` | 不设门槛：报次数与率；零次报单侧 95% 上界 `1 − 0.05^(1/n)`（n = 1048560 时 2.856992e-06），每张卡两种输入合起来另报一个（n = 2097120 时 1.428497e-06） | 第 6 行：它是「真卡上重算之后仍不一致的概率」的分母一侧；够判条件原文「≥ 10⁶ 个单元次的重算，写明零次时的上界」 | 第二段：每张卡每种输入 64 批 × 16384 单元；每批约 0.1–0.5 秒（推的：上传 512 MiB 在 5060 Ti 的 4.0 x8 上至少 0.034 秒，两核的逐线程 32 KiB 计算几十到几百毫秒），四组合计约 26–128 秒；计数无分辨率问题 |
| R7 重算之后还不一致 | 同一行 `still_disagreeing_after_one_recompute=`、`…_after_two_recomputes=`、`…_after_three_recomputes=`、`category=`；每个重算的单元一行 `name=recompute` | `zero`（首轮 0 次）／`transient_only`（首轮有、第 1 次重算全一致）／`deterministic_present`（有一个 3 次重算都不一致）／`mixed`（别的情形） | 第 6 行：`deterministic_present` 是「等几轮也不会好」的观测（翻向 N 取 1、直接交 CPU 那一侧）；`transient_only` 是「重算能消化」的观测（翻向 N ≥ 2）；`zero` 时 N 的取值不由这个量翻面，只报上界 | 同 R6 |
| R8 两核一起错与同一输入同一结果 | 同一行 `both_kernels_agree_and_differ_from_the_cpu=`、`kernel_one_differs_from_the_cpu=`、`kernel_two_differs_from_the_cpu=`；`name=determinism_replay index= form= units= identical=` | 不设门槛：报次数；`both_kernels_agree_and_differ_from_the_cpu` > 0 走第十条 J7 | 第 6 行（确定性不一致出不出现）；两核一起错是 D26 已定项 13 射程里「抓不到两份一起错」那一类的观测 | 同 R6 |
| 只报：今天的写法、进程钟、等那一段、同一线程上 CPU 算同一份字节 | `name=submission_cell` 的 `process_cpu_median_nanoseconds`、`wait_thread_cpu_median_nanoseconds`、`wait_wall_median_nanoseconds`、`polls_median`；今天的写法那几格同一行 `form=today_single_kernel`；`name=cpu_crc_of_the_same_bytes units= thread_cpu_median_nanoseconds=`（本机同一线程上 `crc32_castagnoli` 算 n 个单元，A 在本机能省下的参照） | 只报；今天的写法另按 R1 的式子判一行 `form=today_single_kernel`，不当第 1 行的读数 | 不单独对应；进程钟给 J5，等那一段给 J2，CPU 参照给机器换算的旁证（第十条 J10） | 同 R1 |
| 附带 A1 一批 CPU 按函数拆（附带，够判后不跑） | 第三段：`name=function_split fraction= pool_data_units= batch_units=8 round= call_index= kind= call_cpu_nanoseconds= clone_before_cpu_nanoseconds= clone_after_cpu_nanoseconds= read_the_version_mapping_cpu_nanoseconds= read_the_version_pinned_cpu_nanoseconds= region_scan_cpu_nanoseconds= choose_cpu_nanoseconds= remainder_cpu_nanoseconds=` 与各项占这一批 CPU 的份额；每点五轮中位一行 `name=function_split_summary` | 只报，不判 | 对不上岔路单任何一行：第 1 行按第 7 条比省下的 CPU 绝对值，读这一版、克隆、发布在 CPU 模式与 GPU 模式一样付（攻方第 112 行），不进省下的那一项，也不改提交次数，所以拆出来的份额让第 1、3、6 行哪一行都不翻面。正推 G1-d 要它是为了「甲乙丙的天花板离值得做还差多远」，那是份额口径 | 约 1.3–1.5 小时（推的：准备同 r2 第二段约 1.16 小时（r2 登记第四节第 7 行的 4188.11 秒），只量 K = 8，测量约 0.1 小时，编译约 0.17 小时） |

产物行的键名照 `scripts/gate/data-name-lint.py` 写全名；执行员为过 lint 改了键名的，在第十二节记新旧对照，不改量的算法。

## 七、钉绝对值的断言

### 甲类：出自被测条款本身的（不符走第十条「条款可能错」那一支，不作废）

| 断言 | 出处（逐字） | 不符时 |
|---|---|---|
| 计时区间罩住准备、提交、等、两份比对：P1 甲的自旋插在 ① 之前，增量落在 2 ms 的 [0.9, 1.1] 倍；⑤ 那一段的线程钟 ≤ 整次提交的线程钟 | D26 已定项 13 第 7 条「后者含准备数据、提交、等结果、两份结果的比对」 | 不是条款错，是装置没罩全：走第十一节「阳性对照红」 |
| 一次提交两份核各算一次：每次提交读回 2n 条结果，「确认」+「不确定」= n | 第 8 条「两份独立写的 GPU 核（不共用核函数）对同一批输入各算一次，逐条比两份的结果」 | 同上，装置错 |
| 同一输入在两张卡上判定相同：第二段「两张卡同一输入」那一批，两张卡逐条相同 | 第 8 条「一张与几张都要能跑、判定相同」 | 第十条 J11：不作废，照实记「这台机器上两张卡对同一输入给了不同的结果」，交主 agent（条款的前提在这台机器上不成立，或是卡坏了，两边都查） |

### 乙类：独立算出、用命令核过的（不符才作废，或照第十一节停机）

算式只用第一节的定义；第十三节的 python 命令算过，输出原样抄在那里。

| 断言 | 值 | 在哪核 |
|---|---|---|
| 两核对 9 字节「123456789」 | `0xe3069283` | 每张卡起好之后（P2） |
| 每个点一条意图的提交次数（A、B、C） | f = 0.25：K = 4 → 3，K = 8 → 2，K = 16 → 2；f = 0.75：K = 4 → 7，K = 8 → 4，K = 16 → 3（与 `g1.out` 的 `submissions_per_intent` 逐点相同） | 装置每点核（停机 J-S3） |
| 每个点每次提交的平均字节（A、B、C） | f = 0.25：K = 4 → 218453.3，K = 8、16 → 262144；f = 0.75：K = 4 → 505563.4，K = 8 → 589824，K = 16 → 611669.3 | 装置每点核 |
| 候选 D 的提交次数 | f = 0.25：K = 4 → 2，K = 8、16 → 1；f = 0.75：K = 4 → 6，K = 8 → 3，K = 16 → 2 | 装置每点核 |
| 每次提交写进卡的字节 | 共用 n × 32768，分开 2 × n × 32768；n = 512 时 16777216、33554432 | 每格核（`input_bytes_written`） |
| 物理下限 | 链路峰值按 `nvidia-smi` 现查；按今天读到的：5.0 x16 63015384615 B/s（16 MiB ≥ 0.000266 秒、32 MiB ≥ 0.000532 秒），4.0 x8 15753846154 B/s（16 MiB ≥ 0.001065 秒、32 MiB ≥ 0.002130 秒、512 MiB ≥ 0.034079 秒） | 每次提交（P4） |
| 候选 C 的门槛与 `g1.out` 相同 | 12 个点的 `break_even_host_cpu_per_submission_ms` 到小数第 4 位（0.2159 … 0.7214，第四节第 1 行） | 第一段开头（P7、停机 J-S3） |
| 第二段的内容 | (卡 0、共用、第 1 批、单元 0、32768 字节) 前 8 字节 `3ae4b0f4eaeac4c4`、CRC-32C `0xaac1beb6`；(卡 0、共用、第 1 批、单元 1、16384 字节) `57c11d0ac2056cc7`、`0x0dffe7b3`；(卡 1、分开、第 64 批、单元 16383) `972b7b7830e1c500`、`0x261c3840`；两张卡同一输入那一批单元 0：`7abae00a7fb36a3a`、`0x05f3bd54` | 第二段开头（内容造错了就是 CPU 参照与 GPU 一起错，这一条先红） |
| 第二段的天然单元次与注入 | 每张卡每种输入 1048560；注入 16 个，其中确定性 8、暂态 8 | 每组合收尾 |
| 零次时的上界 | n = 1048560 → 2.856992e-06；n = 2097120 → 1.428497e-06（`1 − 0.05^(1/n)`） | 单测钉式子；产物报的与它到 7 位有效数字相同 |
| 校准 | 16 MiB（第 i 字节 = i mod 251）CRC-32C `0x3430be3b`；32 MiB `0x84b65aa5`（r2 登记第七节） | 每次校准 |

互比旁边的绝对值：等法之间、输入形态之间、两张卡之间的互比，旁边是 P1 的 2 ms（钟没坏）、每格写进卡的字节（活的量没变）与每条结果对 CPU 相同（活确实做了）；三种等法一起错（例如区间漏了 ⑤）时，P1 照样过而「⑤ 那一段的线程钟 ≤ 整次」与 `wait_wall_median_nanoseconds` > 0 先露出来；R6–R8 三个计数一起错（例如注入的单元混进天然计数）时，「天然单元次 1048560」那一条先红。

## 八、轨迹与几何敏感性

### 轨迹（被谓词消费的量）

R1 的 c_rt 被第 1 行当「赔 ／ 不赔」的谓词输入；R6 的不一致次数被第 6 行当谓词输入。

| 量 | 峰值 | 为正的条数 | 期末值 |
|---|---|---|---|
| 每格的 c_rt（一格一轮的计时提交逐次） | `thread_cpu_maximum_nanoseconds`；另报第 90 百分位 | `submissions_at_or_above_the_smallest_threshold`（≥ 0.2159 ms 的次数）与 `submissions_at_or_above_the_largest_threshold`（≥ 0.7214 ms 的次数） | `thread_cpu_last_nanoseconds`；另报头一次 `thread_cpu_first_nanoseconds`（热身之后还在降就看得出来） |
| 第二段每批的首轮不一致 | 64 批里最多的一批 | 不一致 > 0 的批数 | 第 64 批的 |
| 第一段的校准比（作废谓词） | 全段最大 | > 1.10 的格轮数（另报 > 1.05、> 1.20 的） | 段末那一格的 |

只报期末值的那一格正文里不写「从来」「恒」。

### 几何敏感性（每个带跑前判决的量）

| 旋钮 | 两端 | 哪几个判定要两端都报 |
|---|---|---|
| 卡 | 序号 0（RTX 5090，5.0 x16）／ 序号 1（RTX 5060 Ti，4.0 x8） | R1、R2、R3、R5、R6、R7 |
| 等法 | 阻塞等 ／ 睡眠等；睡眠间隔 100 µs ／ 1 ms | R1、R3、R5 |
| 输入形态 | 共用 ／ 分开 | R1、R2、R3、R5、R6、R7 |
| 提交间隔 | 0 ／ 50 ms（共用输入的三种等法） | R1、R2、R3、R5 |
| 提交大小 | 128 KiB … 16 MiB 八档 | R4（拟合本身）；R1 的 12 个点自带 f、K 两个旋钮的两端 |
| 门槛的机器 | g1 原样 ／ × ρ_机器 | R1、R3、R5 |

某个判定在一个旋钮两端翻面，写「随 <旋钮> 翻面」并把两端的值都列出来，不写成那条臂的性质；不翻写「这几个点上不翻」并列点。第 1 行在一个点上的读数要把每个旋钮的两端都列出来，交主 agent 对着岔路单读。

**判别力自证**：判 R1 与 R5 的函数各有单测：把门槛挪到同一格两个量到的值之间，判定必须由一种变成另一种；合成值喂 R5（c_rt 高于 T 而 c_rt(m) 低于 `saved_upper_ms`；c0 很大而 c1 很小；c1 × m × 32768 高于 `saved_upper_ms`），四类各出一次；合成计数喂 R7，四类各出一次。

## 九、变异

V1–V94 照原登记、r2 登记与它们的修订，不增不删；这一趟改到它们锚点的，锚点改到新源码里逐字命中一次（`.claude/rules/mutation-sampling.md` 第七类），点名的单测不换。新加的追加进 `crates/mutations.tsv` 的 E185 行之后，名字接着编 V95 起；改的都是装置自己的代码；每条点名一条会红的单测（要卡的那几条单测带 `gpu` 特性、经清场跑）；常量断言写加法不写减法。执行员可以补，不许删。

| 变异 | 改哪 | 在哪个取样点改变输出 | 该红的 |
|---|---|---|---|
| V95 计时区间漏了等 | 线程钟的后读挪到 ⑤ 之前 | 单测里假的提交步骤（每步按假钟各走固定的纳秒）：⑤ 走 3 ms 时整次应 ≥ 3 ms | 「⑤ 那一段 ≤ 整次」的单测；真跑时 P1 照过，靠这一条 |
| V96 计时区间漏了准备 | 线程钟的前读挪到 ① 之后 | 同一份假步骤，① 走 1 ms | 区间单测；真跑时 P1 甲（自旋插在 ① 之前）红 |
| V97 睡眠等不睡 | 睡眠等循环里去掉 `sleep` | 假的轮询器：前 5 次 poll 没好、第 6 次好；应睡 5 次 | 等法单测（数睡的次数） |
| V98 睡眠间隔换了 | 1 ms 写成 100 µs | 配置行 `sleep_interval_nanoseconds=100000,1000000` | 配置单测 |
| V99 阻塞等不按这一次提交等 | `submission_index: Some(…)` 换成 `None` | 带卡单测：同一个设备上另交一次长的活，再交这一次；应只等这一次 | 带 `gpu` 特性的等法单测 |
| V100 分开输入只写一份 | 分开形态只 `write_buffer` 一次、两核读同一块 | 任何 n | `input_bytes_written = 2 × n × 32768` 的锚点单测 |
| V101 比对不比 | 两份结果不逐条比，「确认」恒 = n | 单测里回读第 3 条异或 1 | 比对单测；真跑时 P3 红 |
| V102 比对只看一份序号 | 核二的序号不核 | 单测里核二回读的序号错一条 | 序号单测 |
| V103 提交序列漏了签发 | 一个点的序列去掉签发那一次 | f = 0.25、K = 4（应 (8, 8, 4)） | 提交次数锚点（3） |
| V104 每批交的是这一批搬的而不是剩下的 | 序列里每批取 min(K, 剩下) 而不是剩下 | f = 0.75、K = 8（应 (24, 24, 16, 8)，变异 (24, 8, 8, 8)） | 平均字节锚点 589824 |
| V105 一个点按中位合 | 一个点的 c_rt 取各次提交的中位而不是平均 | 合成：各档 c_rt 随 n 线性，f = 0.75、K = 4 | 点的单测（平均值） |
| V106 赔的等号错一侧 | `c_rt < T` 写成 `c_rt <= T` | 合成 c_rt 正好等于 T（应 `losing`） | 门槛边界单测 |
| V107 g1 的分组漏了轮 | 按 (f, P, K) 分组不按轮 | 合成产物：同一点两轮、省下的不同 | 账本单测；真跑时停机 J-S3 |
| V108 候选 A 也省了乙 | T_A 加上 `scan_increment_cpu_nanoseconds` | 合成产物 | 候选门槛单测 |
| V109 候选 D 的提交数含签发 | D 的提交次数 = 批数 + 1 | f = 0.75、K = 16（应 2） | D 的提交次数锚点 |
| V110 B 的不赔不标下界 | `not_losing_lower_bound` 写成 `not_losing` | 合成 c_rt < T_B | 判定单测 |
| V111 机器换算倒过来 | ρ_机器 = 第二台 ÷ 本机 | 合成：本机 1 ms、第二台 2 ms（应 0.5） | 换算单测 |
| V112 拟合按 MiB 喂进去当字节 | 字节换成 MiB 再拟合、报的每 MiB 斜率没换回 | 合成 c = 0.1 ms + 0.2 ms/MiB | 拟合单测（还原到 1%） |
| V113 W* 取下整 | ⌈⌉ 写成 ⌊⌋ | 合成 c0 = 1 ms、余量 0.3 ms（应 4） | 攒批单测 |
| V114 乙-1 用平均字节不用 m | `one_submission_per_intent` 取平均提交字节那一档 | f = 0.75、K = 8（m = 24，平均 18） | 攒批单测 |
| V115 拟合不合也判后两类 | 去掉 `fit_not_linear` 的拦 | 合成二次曲线 | 攒批单测 |
| V116 注入的单元进了天然计数 | 天然单元次不减注入的 16 个 | 任何组合 | 锚点 1048560 |
| V117 暂态注入每次都注 | 第 11 个单元的异或在重算时也做 | 第 1 批 | P6 单测（应 `transient`） |
| V118 重算只算核一 | 重算时核二不派 | 第 1 批第 7 个单元 | P5 单测（分开输入时应 `deterministic`） |
| V119 两核一起错不计 | `both_kernels_agree_and_differ_from_the_cpu` 恒 0 | 合成：两核同为 X、CPU 为 Y | 计数单测 |
| V120 上界用 3/n | 式子换成 3/n | n = 1048560（2.861067e-06 ≠ 2.856992e-06） | 上界单测 |
| V121 零次判成暂态 | 首轮 0 次时类别写 `transient_only` | 合成 0 次 | 类别单测 |
| V122 内容的种子漏了输入形态 | 种子里不拼输入形态 | (卡 1、分开、第 64 批、单元 16383) | 内容锚点 `0x261c3840` |
| V123 物理下限分开输入只算一份字节 | 下限按 n × 32768 | 合成：分开、n = 512、挂钟 = 1.5 × 16 MiB ÷ 峰值（应判快过下限） | 下限单测 |
| V124 核一与 crates 的对拍不判 | 对拍结果不进停机 | 单测里换掉一条核一的结果 | 停机 J-S1 单测 |
| V125 间隔的睡落进区间 | 50 ms 的睡挪进计时区间 | 假钟单测：间隔格的挂钟中位应 < 50 ms | 区间单测 |
| V126 函数拆分余数漏减扫描（附带） | 余数 = 调用 − 克隆 − 读这一版（漏 `scan_region`） | 合成一批 | 附带段单测 |
| V127 克隆只算前一份（附带） | 只重放调用之前那一版的克隆 | 合成一批 | 附带段单测 |

单测是新的计数实验，照 `.claude/singlefs-ai-sop/rules/show-me-test.md` 先证明会红（每条变异打开时红在点名的那一条）。

## 十、失败条款

原登记 F1–F8、r1 登记 G1–G6、r2 登记 H1–H8 管的是这一趟不重判的量，不适用。新加（每条紧跟触发的观测）：

| 条款 | 判成什么 | 什么观测会让它触发 |
|---|---|---|
| J1 一个点上任何候选都赔、与等法无关 | 正当结果，照实记：那一点 A、B、C 出局，D 看 R3 | 某点在某张卡、某种输入、某种间隔下 R2（c_floor）判 `losing` |
| J2 等法定输赢 | 正当结果：那一点第 1 行的读数随等法翻面，两端的值与 `wait_wall_median_nanoseconds` 都列出（睡眠等多付的挂钟），交主 agent | 同一点、同一卡、同一输入、同一间隔，阻塞等 `losing` 而某一种睡眠等 `not_losing`（或反过来） |
| J3 两张卡判定相反 | 两张卡分开报，不合成一个（同原登记 F8） | 同一点同一形态 R1、R3 或 R5 在两张卡上不同 |
| J4 随旋钮翻面 | 那一点在那个旋钮上记「随 <旋钮> 翻面」，第 1 行在那一点不当已判，交主 agent | R1、R3、R5 在第八节某个旋钮的两端不同（输入形态、间隔、睡眠间隔、门槛的机器） |
| J5 线程钟低估 | 不作废：那一格另按进程钟算一遍 R1（`clock=process`）并列报；两种钟判定不同的点写「随钟翻面」，交主 agent（主 agent 写死线程钟，第 7 条说的是主机 CPU） | 某格 `process_cpu_median_nanoseconds − thread_cpu_median_nanoseconds` > `thread_cpu_median_nanoseconds` × 10% |
| J6 拟合不合 | R5 后两类写 `fit_not_linear`；前两类照判（它们不经拟合） | R4 的 `maximum_relative_residual` > 20% |
| J7 两核一起错 | 不作废：照实记次数与那几个单元的种子，交主 agent；它是第 8 条射程那句「抓不到两份一起错」的实例，第 5 行（两份核的独立）要用 | R8 `both_kernels_agree_and_differ_from_the_cpu` > 0 |
| J8 天然单元次不足 | 那一组合写「单元次不足」，不下判定、不当阴性，第 6 行在那张卡上不够判，交主 agent 定补不补 | 某张卡某种输入收尾时 `natural_unit_times` < 1048560（有批因读不到、卡掉了没算完） |
| J9 轮数不足 | 那一格写「轮数不足」，不下判定、不当阴性；交主 agent 定补不补 | 某格 `name=round_invalid`（三次都作废）使有效轮少于 5 |
| J10 机器换算离 1 远 | 不作废：两种门槛的判定都照报；R1 两种判定不同的点写「随机器翻面」（J4），交主 agent | ρ_机器 不在 [0.5, 2]，或本机同一线程算同一份字节的 CPU（只报那一行）÷ T_A 的比与 ρ_机器 差一倍以上 |
| J11 两张卡对同一输入结果不同 | 不作废：照实记，交主 agent（第七节甲类第 3 条） | 第二段「两张卡同一输入」那一批有一条两张卡不同 |

没有一条把「没测出差别」写成实现有 bug：c_rt 是多少、真卡错不错，都是正当结果；装置与 crates 对不上走第十一节停机。

## 十一、作废条款与停机条款

### 作废

| 条款 | 作废什么 | 什么观测会让它触发 |
|---|---|---|
| 阳性对照红 | 第一段：那一种提交形态在那张卡上整段作废（P1、P3）；第二段：那张卡那种输入整组作废（P5、P6） | 第五节阳性对照表 P1、P3、P5、P6 任何一格判不出（例：P1 甲的增量不在 2 ms 的 [0.9, 1.1] 倍里；P6 的 8 个里有一个没记成「暂态」） |
| GPU 结果错 | 第一段那张卡整段作废 | P2：9 字节那一段不是 `0xe3069283`，或第一段任何一格任何一条结果与 CPU 不同（第二段的不同是被量的量，不作废） |
| GPU 计时没等到 | 那张卡整段作废 | P4：任何一次提交快过物理下限 |
| 读不到 | 那一格那一轮作废、重量，不当 0 | 线程钟、进程钟、挂钟读失败；`nvidia-smi` 查不到链路；r2 产物里取不到 `calibration_baseline … position=start`（只让机器换算那几行写 `not_run`，不作废别的） |
| 机器被别的活挤 | 那一格那一轮作废，照 r2 修订 R2-2 等常态再重量 | 这一格前后校准比有一个 > 1.10（r2 修订 R2-1 同一个门槛）；或这一格开始前 `nvidia-smi --query-compute-apps` 在这张卡上看到本进程以外的计算进程（这时等它，不开格）；或 `ps -o pid,args -u "$(id -u)"` 看到 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio` |
| 计数特性被并进来 | 那一段整段作废 | 输入清单 `cargo tree` 那一段出现 `lookup-work-counters`（r1 登记第十一节同一条） |
| 清场没过、卡起不来 | 那一段（或那张卡）记「本次未跑」，不当结果 | `gpu-quiesce-run.sh` 退 78；`gpu_cards_by_priority` 没交回点名的卡（`GPU_CARD_NOT_USABLE`） |
| 包装出局 | 那一次输出不算结果，命令与退出码写进报告 | `run-with-memory-cap.sh` 退 245 或 250–254 |

### 停机（装置与 crates 实现对不上，或这一趟的前提不成立：两边都查，既不作废也不当结果，`.claude/rules/implementation-first.md` 第 4 条）

任何一条触发，打 `name=stop_clause clause=<编号>`、收尾、退 3（同原登记的停机退出码），执行员把装置的数、实现的数、两边读的代码行写进报告交回主 agent：

- J-S1 核一与 crates 的 `GpuCard::crc32c_of_ranges` 对同一份输入（n = 8、512）有一条不同。
- J-S2 原登记第五节常量表的本地常量与 crates 回比不等（原登记停机 1）。
- J-S3 装置照 g1 的取法从 r2 第二段产物算出的候选 C 门槛，与 `g1.out` 有一个点到小数第 4 位不同；或某点的提交次数与 `g1.out` 的 `submissions_per_intent` 不同。
- J-S4 输入的两份文件变了：`out/g1.out` 的 sha256 不是 `61b210296634f2f20fa0465a4e4f09859cd9236522c54d4fb18301e75f243025`，或 r2 第二段产物不是 `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a`。
- J-S5 包含自检反过来：某次提交的线程钟增量大于包着它的挂钟增量（计时口径错，同原登记停机 6）。
- J-S6 附带段：`central_mapping_as_the_compaction_reads_it` 与装置 `version_as_the_replay_reads_it` 取出的映射条目集合不同。
- J-S7 装置变了：输入清单第一行、第十二节执行员写的、产物 `name=apparatus` 行三处的装置 sha256 有一处不同（取代 r2 登记停机 7 写死的那一个）。
- J-S8 第二段内容锚点（第七节乙类）不符。
- 原有的时钟分辨率自检（原登记 12.2）不过，同样停。

### 够判停机

岔路单第 1、3、6 行都够判就停（第五节分段表「补完那一行够不够判」），没跑的量逐条标「够判后未跑」：第三段与 A1 默认标「够判后未跑（对不上岔路行）」，除非主 agent 补了岔路行。第一段交回之后第 1、3 行已经够判而第二段还没跑的，第二段照跑（它补的是第 6 行）。

### 比对对象（这一次要与之比的上一次产物）

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out` | `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408` | 203 | `replay.sh` 里没有 E185 的行；跑法第五节「跑法」第一段 | `e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-23.out` | 今天的写法那几格（n = 4、16，间隔 0，阻塞等）的 `wall_median_nanoseconds` 与 r1 `name=gpu_submission range_length=32768 range_count=4`、`=16` 的 `median_seconds` 并列，同一个函数的挂钟，只报不判 |
| 2 | `e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` | `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a` | 1803 | 同上，第一段 | `e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-23.out` | 门槛的来源：候选 C 的门槛照 g1 的取法重算、与 `g1.out` 逐点比（P7、停机 J-S3）；ρ_机器 的分母取它的 `calibration_baseline` |
| 3 | `e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` | `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a` | 1803 | 同上，第三段（附带） | `e185-gc-batch-wall-clock-split-function-split-r3-2026-10-23.out` | 同 (f, P, K = 8) 的 `scan_region` 重放 CPU 与 r2 第二段乙段并列，只报；第三段不跑时这一行标「够判后未跑」 |

重跑产物名里的日期是写登记这一天；执行员起跑那一天不同的，在第十二节记新名，`.claude/gate.d/replay-comparison-units.tsv` 照新名登记。E185 没有单元表，三行都要执行员在那份表的文件头写「# 不登记的比对对象：research/prompts/e185-r3-prereg.md #<号>：<理由>」；写之前门禁 doc-experiments 的 replay-comparison-units 格判这份登记红。

## 十二、修订

（留空。装置改完之后、第一份产物之前由执行员写：改完的装置 sha256、单测与变异条数、键名的新旧对照，只许收严或补量；起法因脚本改了而换写法的，记一行。）

执行员写于 2026-10-23，装置写完、一份产物都没跑；单测与变异此刻还没跑（入库装置，归主 agent），下面各条的依据是写装置时对着第五至十一节逐条落地看出的，不是单测读数。原判据一字不改。

| # | 修订 | 收严还是补量 |
|---|---|---|
| R3-1 | 装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` 改完的 sha256 `4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea`（R3-11 改过一次，那之前是 `ecc61b17…`；最早的改前 `0bd22743…`）；停机 J-S7 比的「第十二节写的那一个」就是它，起跑脚本 `scripts/experiment-apparatus/e185-r3-segment-run.sh` 的 `registered_apparatus_sha256` 同值。装置在草稿副本里写、交补丁，换进主工作区之后若因编不过再改，这一行跟着改、记一行 | — |
| R3-2 | 单测：`#[test]` 173 条（原 118 条，其中 `thread_cpu_time_advances_while_the_thread_spins` 改成按线程 CPU 钟转够 30 ms、或挂钟满 20 秒，断言不变；新加 55 条，53 条不带特性、2 条带 `gpu` 特性）。变异：`crates/mutations.tsv` 的 `E185 V` 行 94 → 151，新加 V95–V151（V95–V127 照第九节；V128–V151 是执行员补的，盯第七节甲类两条、J-S3、J-S4、J-S5、J-S6、J5、J8、J9、J10、P1、P3、第八节第二段轨迹与钟旋钮等），每行点名一条会红的单测，原文在装置里各命中一次；条数由一条 `grep -c` 数，主 agent 跑完单测与变异后照三个数回填 | 补量 |
| R3-3 | 第六节 R5 一行的键名：`one_submission_per_intent_host_cpu_milliseconds` → `one_submission_per_intent_host_cpu_milliseconds_per_round`，`per_byte_part_milliseconds` → `per_byte_part_milliseconds_per_round`，`intents_to_merge_at_least` → `intents_to_merge_at_least_per_round`（这三样每轮各一个，逗号接起来）；另加 `threshold_kind`、`threshold_milliseconds`、`one_submission_per_intent_units`、`fit_not_linear`。R4 一行另加 `fit_not_linear`、`fixed_milliseconds_per_round`、`per_mebibyte_milliseconds_per_round`。别的键与第六节逐字相同；`name=submission_cell` 另加 `wait_inside_the_submission`、`comparison_counts_hold` 与校准比、线程数各栏。为过 `scripts/gate/data-name-lint.py`（单字母与「字母加数字」的值判红）改了的值：`candidate=a`、`b`、`c`、`d` → `scan_checksum_only`、`scan_checksum_and_choice`、`every_stage`、`release_check_only`（第六节 R1 那一行写的 `candidate=c`、R3 的 `candidate=a\|b\|d` 照这张对照读）；翻面行 `name=knob_flip` 的条款值 J2、J3、J4、J5、J10 → `the_wait_method_decides`、`two_cards_judge_differently`、`flips_with_a_knob`、`thread_clock_underestimates`、`machine_ratio_decides`；`quantity=` 的值 → `host_cpu_per_submission`（R1、R3）、`floor_host_cpu_per_submission`（R2）、`merging_shape`（R5） | 补量 |
| R3-4 | 第七节甲类前两条每次提交都核（⑤ 那一段的线程钟 ≤ 整次；「确认」+「不确定」= n），有一次不符就照第十一节「阳性对照红」作废那张卡上那一种形态（`name=positive_control row=interval_coverage`） | 收严 |
| R3-5 | 停机 J-S1 除第五节写的那份输入（真数据单元连放）之外，另拿按种子造的 512 个各不相同的单元再对拍一次，任何一份有一条不同就停 | 收严 |
| R3-6 | R7 的「首轮 0 次」按三样比有一样不同计（两核一致而与 CPU 不同的也算首轮有，进重算）；R6 的率与零次上界按两核不一致计（`first_round_kernel_disagreements`）。两个数同一行都报（`first_round_any_difference` 另起一栏） | 收严 |
| R3-7 | 第十条 J5「两种钟判定不同的点写随钟翻面」写成第八节的一个旋钮：`name=knob_flip knob=clock clause=thread_clock_underestimates`；J2、J3、J4、J10 的翻面同样逐对一行 `name=knob_flip`、每个旋钮一行 `name=knob_flip_summary`（两端都有判定的对数、翻了几对） | 补量 |
| R3-8 | 第八节「第二段每批的首轮不一致」的轨迹写进 `name=disagreement_summary`：`first_round_disagreements_peak_of_a_batch`、`batches_with_first_round_disagreements`、`first_round_disagreements_of_the_last_batch` | 补量 |
| R3-9 | 读法里登记没写死、装置照写的：一格一轮的 c_floor 取每次提交 c_rt − c_wait 的中位；第一段的单元字节是造池时写的一个真数据单元连放 512 份（同原第三段 `one_data_unit_of_the_workload`）；候选 C 的门槛 T 判定用 `out/g1.out` 原样那一个数（四位小数），装置现算的只用于停机 J-S3；每格前后校准只做 16 MiB CRC 3 遍（第五节原文），段开头基线与 P8 照 r2 N2、R2-3（含追指针）；今天的写法只按 C 的门槛判一行 | 补量（写明，不改判据） |
| R3-10 | 起法：照第五节「跑法」写成 `scripts/experiment-apparatus/e185-r3-segment-run.sh <段>`，起跑命令与第五节相同；脚本另核装置 sha256（不等退 3）、输入清单 `cargo tree` 见 `lookup-work-counters` 退 6（第十一节「计数特性被并进来」那一段整段作废） | 补量 |
| R3-11 | 执行员写于 2026-10-24，第 3 次重跑一份产物都没跑。依据是单测 `host_cpu_point_lines_judge_each_threshold_kind_and_mark_voided_and_short_rounds` 的读数（主 agent 单跑红：期望 `knob=threshold_machine clause=machine_ratio_decides pairs=1 flipped=1`，装置出 `pairs=10 flipped=1`，另把一端是 `rounds_insufficient` 的对记成 `name=knob_flip`）。R3-7「两端都有判定的对数、翻了几对」里的「判定」照第六节「5 轮相同写那个判定，不全相同写 `unstable`」、第五节分段表「都有判定（含「不稳定」）」、第十条 J9「不下判定、不当阴性」与第十一节「作废」「读不到」两条读：`name=knob_flip` 与 `name=knob_flip_summary` 的 `pairs`、`flipped` 只算两端都是判定的对；一端是 `rounds_insufficient`、`void_` 开头的作废（`void_positive_control`、`void_card`）、门槛读不到的 `not_run` 或那一端没有这个量的 `not_applicable`（进程钟与今天的写法没有 c_floor）的对不算，`unstable` 算。R1、R2、R5 三个量、八个旋钮同一口径（装置 `is_a_judgement`）。R5 的 `fit_not_linear` 装置照旧当判定，登记没写死它算不算，交主 agent 定。装置 sha256 `ecc61b17…` → `4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea`（R3-1 与起跑脚本 `registered_apparatus_sha256` 一起改）；`#[test]` 173 → 174（新加 `knob_flip_lines_count_only_pairs_with_a_judgement_at_both_ends`）；`E185 V` 变异 151 → 154（新加 V152–V154），V1–V151 的原文在新源码里各命中一次，一条没改 | 收严（装置改回登记口径，判据不动） |

## 十三、读过的文件与跑过的命令

### 读过的文件（行号区间；grep 命中行也算）

| 文件 | 读了哪些行 |
|---|---|
| `.claude/agent-common.md` | 1–111 |
| `.claude/agents/experiment-designer.md` | 派发时随定义给出（全文）；`grep -n prereg` 命中 22、31、57、61 |
| `.claude/singlefs-ai-sop/rules/test-discipline.md`、`evidence-discipline.md` | 全文（1–122、1–179） |
| `.claude/rules/three-way-inference.md`、`mutation-sampling.md`、`implementation-first.md`、`test-script-discipline.md` | 全文（1–166、1–93、1–19、1–106） |
| `.claude/rules/verification.md` | 开头约 2 KB、`grep -n '^#'`、37、43–68（「两档」与「凡用 GPU 先清场，跑完不复原」） |
| `research/prompts/m4-gpugc-design-r1-forks.md` | 1–15 |
| `research/prompts/m4-gpugc-design-r1-main-verification.md` | 1–68 |
| `research/prompts/m4-gpugc-measure-forks.md` | 1–12 |
| `research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out`；`g1_cpu_saved_ledger.py` | 1–14；1–50 |
| `research/prompts/m4-gpugc-design-r1-sonnet-output.md` | `grep -n '^#'`；207、229、233、234、236、240、316、320、391、473、741、744（`grep -n 'G1-a\|G1-d\|G1\.6\|c_rt'` 命中整行） |
| `research/prompts/m4-gpugc-design-r1-opus-output.md` | `grep -n '^#'`；18、28、33、45、51、63、66–80、85、101、112、113、129、135、137、208、219（`grep -n 'g1\|主机 CPU\|每次提交'` 命中整行） |
| `research/prompts/e185-preregistration.md` | 1–721 |
| `research/prompts/e185-r2-prereg.md` | 1–569 |
| `research/prompts/e185-r1-prereg.md` | `grep -n '^#'` |
| `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` | `grep -n '^#'`（第 1 行标题带状态）；275–330（「做了什么」「复跑」「口径」）。「各量的判定」「沿用」「阳性对照」「校准」「岔路表」「它答不了的」「路径与结论登记」「影响的决策」各节没读 |
| `.claude/kb/milestone/04-fourth-txn.md`；`.claude/kb/decisions/26-后台整理与放置回收.md` | 142–147（`grep -n '^\| 七·'`）、144、134；232–252（经 `quote-kb.py`） |
| `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` | 60–169；495–934；`grep -n 'fn \|@compute\|PollType\|poll('` 命中 27–1905 |
| `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` | 8700–9176；5647–5662；`grep` 命中 17、38、42、119、140、141、202–208、688、693、700、702、706、714、746、1898、1914、1915、1927、1932、1943、1944、2283、2387、2494、2582、2702、2799、2963、3000、3363、3368、3869、3962、4023、4045、4061、4066、4080、4136、4601–4640、4861–4885、5254、5647、6294、7351–7359、8746、9565、9617、9631、9648、9655、9667、9679、9715–9741、9969–9990、10278、11976、12934、12938 |
| `crates/singlefs-core/src/mounted_session.rs` | `grep` 命中（492–4227 间）；1926–1932；2660–2690；2800–2870；2985–3035；3150–3175；3240–3270 |
| `crates/singlefs-core/src/region_scan.rs`；`transaction.rs`；`checksum.rs` | 204、284–290、306–312；3162–3165、9675（`grep`）；43（`grep`） |
| `Cargo.lock`；`crates/singlefs-checker-tier/Cargo.toml`；`crates/singlefs-core/Cargo.toml` | 1091–1092；27、40；8、22（`grep`） |
| `crates/singlefs-checker/src/lib.rs` | 56（`grep` 命中） |
| `scripts/crash/gpu-quiesce-run.sh` | 1–60 |
| `.claude/gate.d/replay-comparison-units.tsv` | 1–40（每行前 400 字节） |
| `.claude/gate.d/doc-experiments.sh` | `grep -n 'preregistration-parallelism\|replay-comparison-units\|prereg'` 命中 26–2384 间 |
| `.claude/heavy-thread-default` | 全文 |
| `scripts/kb/replace-once.py` | `--help` 前 8 行 |
| `research/results/e185*` | `ls -la`；三份产物的 `sha256sum` 与 `wc -l`（第十一节比对表），正文一行没读 |
| 多机配置 | `crash-point-replay-shard-configuration-check.sh --emit-hosts` 的输出（私有值不抄） |

没读：`research/results/` 下任何产物的正文；E185 页的结果与结论各节。读到的结果数照第四节列着。

### 跑过的命令（原样；只读命令与写这份登记的命令，没编译、没跑装置、没跑门禁阶段之外的东西）

```bash
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m4-e185-r3-designer --purpose 'E185 第 3 次重跑跑前登记（量一次 GPU 提交的主机 CPU）'
sha256sum research/prompts/e185-preregistration.md research/prompts/e185-r1-prereg.md research/prompts/e185-r2-prereg.md research/prompts/m4-gpugc-design-r1-forks.md research/prompts/m4-gpugc-design-r1-main-verification.md research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out
sha256sum crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs crates/singlefs-checker-tier/src/gpu_unit_checks.rs   # 0bd22743…、93cf2ffd…
git show HEAD:crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs | sha256sum                                     # b3a50d50…（10223 行）
grep -rn 'CRC32C_REFLECTED_POLYNOMIAL\|0x82F63B78\|0x82f63b78' crates/ --include=*.rs
grep -rn 'wgpu' crates/singlefs-core/ crates/singlefs-core/Cargo.toml | wc -l                                                       # 0
grep -c 'E185 V' crates/mutations.tsv; grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs  # 94；118
grep -n 'E185\|e185' scripts/experiment-apparatus/replay.sh                                                                         # 零命中
nvidia-smi --query-gpu=index,name,pcie.link.gen.max,pcie.link.width.max,memory.total,memory.used --format=csv
bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts
for f in research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out; do [ -f "$f" ] && { sha256sum "$f"; wc -l < "$f"; }; done
D=/home/fy5090/.local/share/singlefs/work/m4-e185-r3-designer; nice -n 19 python3 scripts/kb/quote-kb.py $D/clauses.md '.claude/kb/milestone/04-fourth-txn.md~^\| 七·2 \|' '.claude/kb/decisions/26-后台整理与放置回收.md@#### 已定项 13：后台任务的计算放在哪、怎么开、什么节奏'
TZ=Asia/Tokyo date +%F                                                                                                              # 2026-10-23
```

`quote-kb.py` 输出 `✓ 2 段整抄进 /home/fy5090/.local/share/singlefs/work/m4-e185-r3-designer/clauses.md，回读逐字节一致`；追加进本登记之后 `diff <(tail -n 31 登记) clauses.md` 无差。`nvidia-smi` 输出（原样）：

```text
index, name, pcie.link.gen.max, pcie.link.width.max, memory.total [MiB], memory.used [MiB]
0, NVIDIA GeForce RTX 5090, 5, 16, 32607 MiB, 2 MiB
1, NVIDIA GeForce RTX 5060 Ti, 4, 8, 16311 MiB, 2 MiB
```

三份产物的 `sha256sum` 与 `wc -l`：`4041a1ca…` 203 行、`f70eb21b…` 1803 行、`18191ec3a8afbb27405987f99d3e36e5ae2538d3c9ab4f2717738495e767096b` 2103 行（第三份是 r2 第一段，这一趟不用）。

提交序列、上界与链路峰值（第四节第 6、7 行，第七节乙类；命令与输出原样）：

```bash
nice -n 19 python3 -I -c '
import math
UNIT=32768
def sizes(m,K):
    s=[m]; left=m
    while left>0:
        s.append(left); left-=min(K,left)
    return s
def release(m,K):
    out=[]; left=m
    while left>0:
        k=min(K,left); out.append(2*k); left-=k
    return out
for f,m in ((0.25,8),(0.75,24)):
    for K in (4,8,16):
        s=sizes(m,K); r=release(m,K)
        print(f"f={f} K={K} crc_submission_units={s} submissions={len(s)} mean_units={sum(s)/len(s):.4f} mean_bytes={sum(s)*UNIT/len(s):.1f} release_check_submission_units={r} release_submissions={len(r)}")
for n in (1048560, 1048576, 2097120):
    print("n",n,"upper95_exact",1-0.05**(1/n),"rule_of_three",3/n)
for gen,gts,w in ((5,32e9,16),(4,16e9,8)):
    bw=gts*w*128/130/8
    print("pcie gen",gen,"x",w,"peak_Bps",round(bw),"min_s_16MiB",16*2**20/bw,"min_s_32MiB",32*2**20/bw,"min_s_512MiB",512*2**20/bw)
print("units_in_set", sorted({4,8,12,16,20,24,32,512}), [u*UNIT for u in (4,8,12,16,20,24,32,512)])
'
```

```text
f=0.25 K=4 crc_submission_units=[8, 8, 4] submissions=3 mean_units=6.6667 mean_bytes=218453.3 release_check_submission_units=[8, 8] release_submissions=2
f=0.25 K=8 crc_submission_units=[8, 8] submissions=2 mean_units=8.0000 mean_bytes=262144.0 release_check_submission_units=[16] release_submissions=1
f=0.25 K=16 crc_submission_units=[8, 8] submissions=2 mean_units=8.0000 mean_bytes=262144.0 release_check_submission_units=[16] release_submissions=1
f=0.75 K=4 crc_submission_units=[24, 24, 20, 16, 12, 8, 4] submissions=7 mean_units=15.4286 mean_bytes=505563.4 release_check_submission_units=[8, 8, 8, 8, 8, 8] release_submissions=6
f=0.75 K=8 crc_submission_units=[24, 24, 16, 8] submissions=4 mean_units=18.0000 mean_bytes=589824.0 release_check_submission_units=[16, 16, 16] release_submissions=3
f=0.75 K=16 crc_submission_units=[24, 24, 8] submissions=3 mean_units=18.6667 mean_bytes=611669.3 release_check_submission_units=[32, 16] release_submissions=2
n 1048560 upper95_exact 2.856992441180317e-06 rule_of_three 2.861066605630579e-06
n 1048576 upper95_exact 2.85694884705201e-06 rule_of_three 2.86102294921875e-06
n 2097120 upper95_exact 1.4284972409406294e-06 rule_of_three 1.4305333028152895e-06
pcie gen 5 x 16 peak_Bps 63015384615 min_s_16MiB 0.00026624 min_s_32MiB 0.00053248 min_s_512MiB 0.00851968
pcie gen 4 x 8 peak_Bps 15753846154 min_s_16MiB 0.00106496 min_s_32MiB 0.00212992 min_s_512MiB 0.03407872
units_in_set [4, 8, 12, 16, 20, 24, 32, 512] [131072, 262144, 393216, 524288, 655360, 786432, 1048576, 16777216]
```

第二段内容锚点（第七节乙类；命令与输出原样）：

```bash
nice -n 19 python3 -I -c '
POLY=0x82F63B78
T=[]
for i in range(256):
    c=i
    for _ in range(8):
        c=(c>>1)^POLY if c&1 else c>>1
    T.append(c)
def crc(data):
    c=0xFFFFFFFF
    for b in data:
        c=T[(c^b)&0xFF]^(c>>8)
    return c^0xFFFFFFFF
M=(1<<64)-1
def unit_bytes(card,form,batch,unit,length):
    x=0x9E3779B97F4A7C15 ^ ((card<<48)|(form<<40)|(batch<<20)|unit)
    out=bytearray()
    while len(out)<length:
        x^=x>>12; x^=(x<<25)&M; x^=x>>27
        out+=((x*0x2545F4914F6CDD1D)&M).to_bytes(8,"little")
    return bytes(out[:length])
print("check", hex(crc(b"123456789")))
for card,form,batch,unit in ((0,0,1,0),(0,0,1,1),(1,1,64,16383),(0xFFFF,0,0,0)):
    L=32768 if unit%2==0 else 16384
    b=unit_bytes(card,form,batch,unit,L)
    print(f"card={card} form={form} batch={batch} unit={unit} length={L} first8={b[:8].hex()} crc32c={crc(b):#010x}")
'
```

```text
check 0xe3069283
card=0 form=0 batch=1 unit=0 length=32768 first8=3ae4b0f4eaeac4c4 crc32c=0xaac1beb6
card=0 form=0 batch=1 unit=1 length=16384 first8=57c11d0ac2056cc7 crc32c=0x0dffe7b3
card=1 form=1 batch=64 unit=16383 length=16384 first8=972b7b7830e1c500 crc32c=0x261c3840
card=65535 form=0 batch=0 unit=0 length=32768 first8=7abae00a7fb36a3a crc32c=0x05f3bd54
```

第四节「50 ms 取在哪」用的数与第十四节的挂钟估算（命令与输出原样）：

```bash
nice -n 19 python3 -I -c '
import re
rows=[dict(m.groups() for m in re.finditer(r"(\w+)=(\S+)",l)) for l in open("research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out") if l.startswith("name=g1_ledger ")]
v=[float(r["intent_cpu_ms"])/int(r["submissions_per_intent"]) for r in rows]
print("intent_cpu_per_submission_ms min",round(min(v),2),"max",round(max(v),2))
'
nice -n 19 python3 -I -c '
cards=2; rounds=5
gap0_cells=8*3*2+8   # 常驻 n×等法×输入，加今天的写法
gap50_cells=8*3      # 共用输入 × 三种等法
per_sub_hi=0.012; per_sub_lo=0.003
gap=0.05
for name,ps in (("upper",per_sub_hi),("lower",per_sub_lo)):
    t=cards*rounds*(gap0_cells*110*ps + gap50_cells*60*(ps+gap))
    pc=cards*rounds*7*3*30*(ps+0.002)
    cal=cards*rounds*(gap0_cells+gap50_cells)*2*3*0.006
    print(name,"cells_per_card",gap0_cells+gap50_cells,"submissions_per_card_per_round",gap0_cells*110+gap50_cells*60,"measure_s",round(t),"positive_control_s",round(pc),"calibration_s",round(cal),"total_h",round((t+pc+cal)/3600,3))
for name,per_batch in (("upper",0.5),("lower",0.1)):
    print("segment2",name,"seconds",64*2*cards*per_batch)
'
```

```text
intent_cpu_per_submission_ms min 11.44 max 99.16
upper cells_per_card 80 submissions_per_card_per_round 7600 measure_s 1632 positive_control_s 88 calibration_s 29 total_h 0.486
lower cells_per_card 80 submissions_per_card_per_round 7600 measure_s 948 positive_control_s 32 calibration_s 29 total_h 0.28
segment2 upper seconds 128.0
segment2 lower seconds 25.6
```

其余只读命令（`sed -n`、`awk 'NR==…'`、`grep -n`、`wc -l`、`ls`）读的行号都列在上一张表里。

## 十四、并行与挂钟

- 独立单位：第一段两张卡互相独立（各起各的设备、各开各的缓冲），但按卡先后量、不同时量；一张卡上 80 格各自独立（每格自己热身、自己校准），只是一轮里按轮转的次序串着量；第二段每张卡每种输入 64 批互相独立，一批里 16384 个单元的内容生成与 CPU 参照互相独立；第三段同 r2 第二段的全局准备队列（各点造池、第 2–5 轮重造互相独立）。三段各自一条命令、各自一份产物，第二段不读第一段的产物。
- 线程：`E185_THREADS` 不设，取登记值 `.claude/heavy-thread-default`（今天 31）；第一段整段一条线程（装置自己不起线程，驱动与 wgpu 在本进程里起的线程不归装置管，它们的 CPU 由进程钟单报）；第二段内容生成与 CPU 参照用 31 条线程、驱动 GPU 的是一条；第三段准备队列至多 31 条、计时窗口一条；编译经 `bash scripts/hosts/capped.sh 31`。
- 串行：第一段每次计时提交与每格都串行，量的是发起提交那一条线程的 CPU，别的线程抢 CPU、抢 PCIe 会混进 c_rt 与等那一段；两张卡先后量，同时量会让两张卡的提交在主机上互相抢；第一段与第二段不同时跑，第二段的 31 条线程会搅第一段的计时；第三段的计时窗口照原登记 12.11 串行。第三段在第二台，与本机的第一、二段可以同时跑。
- 预计挂钟：第一段约 0.5–0.7 小时（推的：带 `gpu` 特性编一次约 0.17 小时，同原登记第三段的估计；起卡与编管线约 0.03 小时；测量 0.28–0.49 小时，由每张卡 80 格、每轮 7600 次计时提交、单次 3–12 毫秒、间隔格另加 50 毫秒算出，第十三节现算）；第二段约 0.05–0.1 小时（推的：与第一段共用编译；每张卡每种输入 64 批、每批 0.1–0.5 秒，四组合计 26–128 秒，加起卡与两张卡同一输入那一批）；第三段（附带）约 1.3–1.5 小时（推的：准备同 r2 第二段约 1.16 小时，r2 登记第四节第 7 行的 4188.11 秒；只量 K = 8，测量约 0.1 小时；编译约 0.17 小时）。三段各自都在 2 小时内，不写分片、续跑、多机三样。
- 预计在用核数：第一段编译那几分钟约 31 核，测量时 1 核加驱动线程（推的，1–2 核）；第二段每批造内容与 CPU 参照时约 31 核、等卡时约 1 核；第三段编译约 31 核、准备 5–15 核（r2 登记第十四节，`E185_MEMORY_CAP=40G` 下约 5 份同时）、计时窗口 1 核。
