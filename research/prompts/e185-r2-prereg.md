# E185 第 2 次重跑登记：GC一批搬迁的挂钟拆分（占比 0.25 的几何改负载、机器状态读数、校准作废）

写于 2026-10-23，装置改之前、这一趟一份产物都还没跑。不是差分重跑：负载（第五节）、装置的读数（第六节）与作废、重试的做法（第十一节修订 R2-1 至 R2-4）都要改。问题单 Q1–Q17 每一行的算法、门槛、判定照原登记 `research/prompts/e185-preregistration.md`（含第十二节 12.1–12.11；读的那一刻 sha256 `473493a455034a6f96c0c276e712fce3e0d423001040d5ba0360e0b46ccb8198`，721 行）一个字不改；第 1 次重跑登记 `research/prompts/e185-r1-prereg.md`（sha256 `46736ef87c10932a8108e22c6f3fc557179d746081d0d0d25791d0b13046c354`，417 行）里管跑法、输入清单与停机的几条照下文点名沿用。下文写「原登记」「r1 登记」就指这两份。

## 一、问题

英文名：gc_batch_wall_clock_split

（照抄原登记第一节第 7 行；装置仍是 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`，不另起，复跑行同一份。）

**主 agent 给的问题**（派发提示「关的是哪几条」逐字）：

```markdown
这一趟要让问题单第 1、2 行够判（第 3、4 行第 1 次重跑已够判，这一趟不再判，写明沿用哪份产物）。
1. 负载：主 agent 定取调查报告的改法甲（只改装置覆盖写的次序，让占比 0.25 的几何造得出；乙用快照留旧单元，写成甲造不出时的后备，不并跑）。登记写明怎么判造没造成、造不出时停机交主 agent；写明「占比 0.75 的池与第 1 次重跑逐字节相同」怎么核（调查报告给的 `name=pool digests=` 对 W:237、W:515 那一条）。验收：第五节与第十一节各有对应行。
2. 读数：调查报告第四节列的每条意图的额外读数（排队时间、utime/stime、缺页、PSI、loadavg、CPU 号、单调钟秒数）、造池之后逐段打成分、每条意图之前的固定活校准，逐条写进第六节「只报不判」或接进已有的作废条款；接进作废条款的（例如校准比接进「机器被别的活挤」、作废后不立刻重试）写成登记修订，原判据原样保留并写明收严在哪。Q1–Q17 的算法、门槛、判定一个字不改。验收：第六节每个新量对应问题单一行或写明是作废用的读数。
3. 问题单第 2 行丙段（选段）第 1 次重跑是 `not_run`（原登记 F7 打中：装置只在批行上判、选段只在签发那一次发生）：写明这一趟怎么让丙段有读数，或者写明为什么这一趟仍判不了、第 2 行丙段怎么交主 agent。Q9 那一格照第 1 次重跑登记第六节「Q9 一格交主 agent」原样沿用。
4. 计时期间两台的清场怎么做、怎么记进产物（第 1 次重跑起跑记录没写清场，调查报告第四节末条）。
5. 装置 sha256：第 1 次重跑第三段已用 `eadb929d…`（与 `b3a50d50…` 只差 3 行 `quarantine_mark`，主 agent 已认）；这一趟装置要改（第 1 条），登记写明改前基线取哪一版、改完的 sha256 由执行员写进产物与输入清单。
6. 主 agent 此刻的约束：预开发会话在改所有起编译、拷库、跑重活的脚本（每趟自己的编译目录、跑完删，共用编译缓存会删），它通知之前不起 merge-and-verify.sh、变异门禁、crash-point-replay.sh、崩溃放量与双机驱动、multi-host-run.sh；登记里的跑法照现行脚本写，主 agent 起跑前再核脚本有没有变。
```

**问题单逐行抄**（`research/prompts/m4-gpugc-measure-forks.md` 第 5–12 行，原样；文件 sha256 `78eaec57b0a7f6fc2bbe3435458385ba84ef7beae9dce8d423d32e5454d55c8e`，与 r1 登记第 21 行记的相同，四行「状态」一格在文件里仍写「开着」）：

```markdown
被测的是 `crates/` 今天的 GC 路径：`crates/singlefs-core/src/mounted_session.rs` 的 `run_compaction`（区域扫描 `region_scan.rs` → 选段与计划 `compaction_plan.rs` → 发布 `transaction.rs` 的 `publish_relocation_batch`）。

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | 一批搬迁的挂钟受谁限：盘 ／ CPU | 一批里 CPU 时间与等盘读写时间的比；同一批在内存设备与真盘（直接 I/O）上的挂钟差 | 至少两种批大小、两种段内活槽占比，在内存设备与真盘上各量到一批的挂钟与按阶段拆开的 CPU 时间 | 开着 |
| 2 | GPU 上算 GC 的哪几段：甲 只算校验和核对（搬之前核源单元）／ 乙 甲加区域扫描里的单元头解析与映射反查 ／ 丙 乙加选段打分（`choose_region` 对全部候选段） | 每一段在一批里的 CPU 时间占比与它处理的字节数（得出每段的 CPU 吞吐） | 第 1 行够判的那组取样点上，三段各自的 CPU 时间与字节都量到 | 开着 |
| 3 | 攒批形态：甲 一批一交 ／ 乙 连续几批攒起来一次交，几批由显存上限定 ／ 丙 放宽一条意图的 R 到不止一个聚簇段 | 一批交给 GPU 的计算字节数；GPU 一次提交的固定开销与每字节开销 | 一批的计算字节数按批大小量到；GPU 一侧的两样开销量到（本机 RTX 5090 / 5060 Ti，经 `scripts/crash/gpu-quiesce-run.sh` 起） | 开着 |
| 4 | 定时兜底间隔与写带宽上限的默认值从什么量取：按一批在真盘上的写字节与挂钟 ／ 另跑前台负载量 GC 对前台 fsync 延迟的影响 | GC 单跑的写带宽；GC 与前台 fsync 并发时前台延迟的变化 | 第 1 行的真盘取样点上写带宽量到；要不要量并发那一项，设计员判（附带，够判后不跑） | 开着 |
```

**这一趟补哪几行**：第 1 行（Q1–Q4）、第 2 行（Q5–Q8，丙段的 Q7 照第六节「丙段怎么有读数」）。第 3、4 行不再判，沿用第 1 次重跑的产物（主 agent 判过够判，派发提示）：

| 行 | 沿用哪份产物 | sha256（现算） | 这一趟的数会不会让它翻面 |
|---|---|---|---|
| 3（Q10–Q13） | `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（a、b、φ、n*），连同它读的 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（B_批） | `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408`；`978ae7b5fe60ab8f5a87338b982e8989c4c0d0ba8f19994bfc8e1c39b3d437fd` | 推的，不会：占比 0.25 造成之后它的 B_批 ≤ 签发那一次的 262144 字节（原登记第七节乙类第 356 行），比占比 0.75 的小；φ = a ÷ (a + b·B_批) 随 B_批 变小只会变大，翻面要 φ 落到 50% 以下。第三段这一趟不跑 |
| 4（Q14–Q16 单跑那一半） | `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out` 的 `name=q16` | `978ae7b5…`（同上） | 第一段装置照样算 Q14–Q16，这一趟只并列报、不重判；这一趟 `name=q16` 判 `needs_the_concurrent_measurement` 的，写进报告交主 agent 定第 4 行要不要重开，第四段这一趟不跑 |

Q9（第 2 行与第 3 行共用）照 r1 登记第六节「Q9 一格交主 agent」原样沿用：照原登记算、照原门槛判、照原样报，第 2 行判的时候 Q9 算不算翻面量由主 agent 定。这一趟第三段不跑，所以 Q9 只有第 1 次重跑那 72 行（第三段产物），占比 0.25 新造成的点上没有 Q9；主 agent 定 Q9 算翻面量、又要这些点的 Q9 时，另跑第三段，跑不跑的条件照 r1 登记第五节「第三段跑不跑」三甲、三乙、三丙（产物换成这一趟的），三丁换成「主 agent 定要」。

**读法写死**：原登记第一节「读法写死」那张表与 r1 登记第一节补的「真盘」一格（第二台那块盘上 ext4 的 `O_DIRECT` 镜像文件，盘型号与逻辑块记进输入清单）照沿用；r1 登记补的「七·6 前、七·6 后」「一批的 CPU（比法用）」两格这一趟不用（这一趟没有七·6 前后的比法）。新补：

| 词 | 口径 |
|---|---|
| 改法甲（覆盖写次序） | 只对占比 0.25：造池第 2 步要覆盖写的单元，次序从「全部按旧槽号升序」（装置第 2263 行 `inodes_to_overwrite.sort_unstable()`）改成三组依次：① 不在「整段 32 个都是这次写的」段里的单元（混住段的全部，与配方累计点名的），组内按旧槽号升序；② 整段里要覆盖写的单元，除槽号最大的那一整段之外，按旧槽号升序；③ 槽号最大的那一整段里要覆盖写的 24 个，按旧槽号升序。占比 0.75 的次序不变（仍全部按旧槽号升序）。覆盖写哪些单元、写几次、写的内容一样不变，只换次序 |
| 改法乙（后备，不并跑） | 第五节「乙」那一段；只在主 agent 看了甲的单点核之后定用它时才改进装置 |
| 第 1 行够判（这一趟的读法，待主 agent 认） | 集合 S = 这一趟第一段里（f、P、K）三者的格，满足：那一点 `name=geometry` 写 `formed=true`，且那一格 `name=q3` 写 `judgement=compatible`。两个 f 在 S 里各有至少两个 K（P 取哪个、取几个都行）就够判；Q1、Q2 在 S 的格上报的判定（含「不稳定」，原登记 F4）就是第 1 行的读数。原登记 F5 打中的格（Q3 不是 `compatible`）不进 S |
| 第 2 行够判 | S 里的每一格，第二段 `name=stage_judgement` 的甲、乙两段判定与第六节新加的 `name=choose_judgement` 的丙段判定都不是 `not_run`，且对应 `name=stage` 行的 `crc_bytes`、选段的 `records_read_per_choice` 大于 0 |
| 校准比 | 第六节 N1：同一条线程按线程 CPU 钟量一遍固定的活（16 MiB 缓冲的 CRC-32C，连做 3 遍取中位）÷ 这一段开头那一块校准的中位 |

未定项：

1. **第 1 行够判的读法**（上表）是设计员按问题单第 1 行够判条件与原登记 F5、「几何没造成」两条写的，没有别的出处；主 agent 认或改。改了它不动任何判据，只改「够判」怎么从产物里数。
2. **Q3 在小批上的余量**（第四节第 9 行）：占比 0.25、P = 1024 一批只有十来毫秒 CPU，第 1 次重跑没打中的轮 Q3 在 0.0076–0.1167，离 20% 的门槛十来个百分点。这一趟可能再在这一格打中 F5；打中了这一格不进 S，第 1 行够不够判看 P = 8192 那几格。判据不改；是不是要为这一格另想办法，交主 agent。
3. **清场用哪个入口**：仓里清两台的入口只有 `scripts/crash/gpu-quiesce-run.sh`（照多机配置的私有清场命令，文件头第 33 行「第二台的回读不变」，推的：私有脚本也清第二台，没读私有脚本）；它是为起 GPU 的跑法写的，这一趟拿它包一条不起 GPU 的计时跑法（第五节「清场」）。主 agent 认用它，或给别的清场办法。
4. **校准门槛 1.10 的依据**只有本机上一个代用量（第四节第 11 行：zlib 的 CRC-32 在本机、负载 18 时的分布），第二台、装置自己的 CRC-32C 没量过；门槛在跑前写死，不看这一趟的数再改（R2-1）。

## 二、被测条款与它引的定义

三段，`scripts/kb/quote-kb.py` 整段抄（出口 `/home/fy5090/.local/share/singlefs/work/m4-e185-r2-designer/clauses.md`，回读逐字节一致，再整份追加到这里）：里程碑四作业点表七·1 那一行（派发点名的被测条款；今天在第 142 行，r1 登记抄时在第 141 行，正文逐字相同，`diff` 核过）；D26 已定项 13 与 D24 已定项 3（Q9 一格交主 agent 那一句引的两条；与 r1 登记第二节第 69–111 行逐字相同，`diff` 核过）。原登记第二节另抄的 D26 已定项 1、D24 已定项 2、C412 与 r1 登记抄的七·6 这一次没重抄，照原登记与 r1 登记第二节。

**出处 `.claude/kb/milestone/04-fourth-txn.md:142-142`（整段抄，未转述）**

```markdown
| 七·1 | 量：在 `crates/` 现有的 GC 路径上量一批搬迁的挂钟，拆成读写 I/O 与 CPU（解单元头、核校验和、查映射、分配记录树写放大）各几成；口径写明设备、负载、批大小（C412（整理的挂钟代价没量过） 的检查列） | C412（整理的挂钟代价没量过）；D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式 | 断言钉绝对值；批大小翻倍挂钟跟着涨（判别力自证） | 二·2 的发布路径 |
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

现查于 2026-10-23，主工作区，文件加行号（设计员的观测，不是结论）。原登记第三节与 r1 登记第三节两张表的事实，除下面几格外照旧，行号以今天的为准。

| 一步 | 在哪 | 看到的事实 |
|---|---|---|
| 显式入口 | `crates/singlefs-core/src/mounted_session.rs:1956` | `run_compaction` 还在（r1 登记写的是第 1939 行） |
| 整理读版本 | `mounted_session.rs:2807`、`2835` | `central_mapping_as_the_compaction_reads_it`、`version_as_the_compaction_reads_it` 还在 |
| 区域扫描的映射反查 | `crates/singlefs-core/src/region_scan.rs:204`、`286`、`309` | `scan_region` 仍对传进来的全部映射条目线性找两遍（`entries_pointing_here`、`header_key_is_mapped`） |
| 选段 | `crates/singlefs-core/src/compaction_plan.rs:88` | `choose_region` 还在 |
| 发布 | `crates/singlefs-core/src/transaction.rs:9661` | `publish_relocation_batch` 还在 |
| 覆盖写那一次就记释放 | `transaction.rs:2059` | `allocator.release(released, txg)`：旧落点在覆盖写那一次发布就记成已释放（调查报告第一节表第 2 行引的同一行，今天仍在第 2059 行） |
| 何时回到空闲 | `crates/singlefs-core/src/allocator.rs:2725`、`2744` | `record_root_written_by_this_process` 每写一条根按 `reclaim_floor(F_生效, 环里最旧有效根)` 回收（调查报告引 `:2725`、`:2744`，行号今天相同） |
| 环多长 | `crates/singlefs-format/src/lib.rs:291`、`309` | `ROOT_RING_REGIONS: u64 = 3`、`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8`：24 条根 |
| 用户数据落点 | `allocator.rs:954`、`964` | `lowest_user_data_slot`、`lowest_user_data_slot_outside_the_region` 还在 |
| 提交内生块落点 | `allocator.rs:2389` | `try_allocate_commit_generated` 还在 |
| 快照（改法乙要用） | `mounted_session.rs:118`、`124` | `UserChange::CreateSnapshot`、`UserChange::DestroySnapshot { snapshot }` 还在 |
| 线程 | `grep -rn 'thread::spawn\|thread::scope\|rayon' crates/singlefs-core/src \| wc -l` | 0 |
| 两棵树之间 | 第十三节 `sha256sum` 与 `git diff --stat HEAD` | 核心层 `transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs` 与 `Cargo.lock` 的 sha256 都与第 1 次重跑输入清单（`research/results/e185-r1-inputs-2026-10-22.txt` 第 2–9 行）记的不同，`checksum.rs`、`block_device.rs` 相同；`allocator.rs` 比 HEAD 多 354 行加、`singlefs-format/src/lib.rs` 35 行改动（第一次重跑的输入清单没记这两份）。这一趟起跑的树又会不同（预开发会话与别的实现批还在改），起跑那一刻的哈希记进输入清单（第五节） |

**装置今天的样子**（改前基线，第五节「装置」）：主工作区 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` sha256 `eadb929d6d8be037530f2ce94ee4a564202939f9c2d25e275023aa7ce091ae76`，10226 行，`#[test]` 87 条；HEAD 那一版 `git show HEAD:… | sha256sum` 是 `b3a50d50441f9f2e74bec6c941b74e632529688a78563995387708230b60c470`，差的就是 `git diff HEAD` 那 3 行加（第 6351 行 `quarantine_mark: record.quarantine_mark,`，单测里两行）。`crates/mutations.tsv` 里 `E185 V` 起头 55 行。

| 装置里的一处 | 行 | 看到的 |
|---|---|---|
| 造池 | `construct_the_pool` 第 2211 行；覆盖写排序第 2263 行 | 要覆盖写的单元全部按旧槽号升序（改法甲改这一行） |
| 几何核与后备 | `is_the_geometry_formed` 第 2592 行；`prepare_the_sampling_point` 第 2727 行；后备第 2788 行 | 造成 = R 里没有别的槽且活槽 = 64f；后备取「全部 f 段」里最前的一段，一段都没有就 `choice=none … formed=false` |
| 作废原因 | `invalid_reason_of` 第 3611 行 | 「未入账超过挂钟 10%」是 `unaccounted_time_above_ten_percent_of_the_wall` |
| 重试 | `measure_with_retries` 第 3898 行（第一段）；`stage_split_one_intent` 第 6578 行起的同形循环（第二段） | 作废之后当场 `continue` 重量，至多 3 次（`ATTEMPTS_PER_ROUND_AT_MOST`，第 287 行），没有等 |
| 计时窗口 | `enter_the_timing_window` 第 4243 行 | 先核本进程只剩一条线程，再声明串行段 |
| 池摘要行 | 第 5025 行 | `name=pool … digests=` |
| 丙段份额 | `emit_the_stage_shares` 第 6488 行；判定收集第 6653 行 | `choose_share_per_batch = 选段重放 CPU ÷ 这条意图的批数 ÷ 这条意图批 CPU 的中位`，只在签发那一次调用的重放里有选段 CPU；判定只从 `counts_as_a_batch` 的调用收（第 6653 行），签发那一行算出的值从不进 `name=stage_judgement`，所以那一格恒是 `not_run`（这就是 r1 F7 那一格「丙 not_run」的来源，读码的） |
| 读线程状态 | `grep -n '/proc/thread-self\|schedstat' …` 零命中 | 装置今天不读排队时间、切换、缺页（CPU 走 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`，原登记 12.2） |

与条款对不上的：无新的（r1 登记第三节记的七·6 两个函数名零命中那一处照旧）。

## 四、跑之前已经存在的数

这一趟是看过第 1 次重跑的结果才登记的（派发提示点名要读执行员与调查员的两份报告）：判据（Q1–Q17）没改一个字，所以下面这些数不影响任何门槛；改的是负载、读数与作废，这些改法就是照这些数定的，最后一列逐条写。编号给正文引用。

| # | 数 | 从哪读到 | 对判据与这一趟改法的影响 |
|---|---|---|---|
| 1 | 原登记第四节整张表、r1 登记第四节整张表的数 | 两份登记第四节 | 同那两份 |
| 2 | 第 1 次重跑的判决行：第一段 `positive_control_failures=0 … clause_wall_clock_not_rising_with_the_batch_triggered=9 clause_two_cpu_paths_disagree_points=5 gc_to_sequential_write_bandwidth_maximum_ratio=0.011019 gc_to_sequential_write_bandwidth_judgement=single_run_enough`；第二段 `positive_control_failures=0`；第三段 `card_failures=0 discrimination_holds=true`；比法 `g1_after_not_near_linear=1 g2_before_not_near_square=18 g3_two_sources_disagree_before=2 g3_two_sources_disagree_after=1 g4_different_work=0 g5_flipped=27` | 执行员报告 `/home/fy5090/.local/share/singlefs/work/m4-e185-r1-page/result.md` 第 76–82 行（整行抄的判决行） | 第 4 行沿用（第一节）；F1、F5 的触发数是上一趟的，这一趟照原判据重判 |
| 3 | 第一段 f = 0.75、P = 1024 三格 `name=q1` 五轮全 `both`（ρ 中位 1.163279、1.352170、1.293462）；f = 0.75、P = 8192 三格 `name=q3` `judgement=unstable`；Q1、Q2 合计 24 格「11 both / 10 cpu / 3 unstable」；第二段 28 行 `name=stage_judgement` 全「甲 not_worth、乙 not_worth、丙 not_run」，份额最大 f = 0.75 甲 0.020472、乙 0.006122、丙（签发行原值）0.013997，f = 0.25 丙 0.033060；第三段 Q9 72 行全 `gpu_loses`，Q13 56 行全 `merge_batches`，φ 最小 0.911860，两张卡 `intercept_seconds` 0.004492、0.010300 | 同上报告第 29–57 行 | 这些是第 1、2 行这一趟要重量的量在上一趟的值：门槛不由它们推出（门槛是原登记的）。第 3 行沿用的依据是 φ 最小 0.911860 离 50% 远（第一节） |
| 4 | 几何：第 1 次重跑 f = 0.25 两点 `geometry_formed_segments=0`、`choice=none … formed=false`，`overwritten_units=776`（P = 1024）、`6152`（P = 8192）；f = 0.75 两点 `geometry_formed_segments=3`、`choice=forced_fallback`，`overwritten_units` 280、2072；f = 0.25、P = 1024 `construction_seconds=85.543` | 调查员报告 `/home/fy5090/.local/share/singlefs/work/m4-e185-r1-investigation/result.md` 第 17–19、50–56 行 | 改法甲就是照这几个数与它推的机理定的；第 10 行的预测用了 `overwritten_units` |
| 5 | F5 三格逐轮原值（Q3 0.0194–1.2359，设备调用里两臂之差占内存臂挂钟 0.40%–1.7%）；慢段：同一份活 CPU 涨到 1.13–2.5 倍，连成一片 8 条意图，其中一条「未入账」10.8%、作废后当场重试的那条 2.316 倍被收下；第 1 次重跑第二段 60 条意图没有慢段；G3 三格的 α 每轮值 | 同上调查员报告第 92–137、182–209 行 | 这一趟的校准、作废后等常态、机器状态读数（第六节、第十一节 R2-1 至 R2-4）照这些定；Q3 的 20% 门槛不改 |
| 6 | 第 1 次重跑第一段第 237、515 行开头：`E7RESULT name=pool fraction=0.75 pool_data_units=1024 choice=forced_fallback geometry_formed=true speculative_rebuilds_voided=0 digests=423aa5bc004b3247287ae29c…`、`… pool_data_units=8192 … digests=e0910aa57c7de2b42dfd1026…`（只看了每行前 160 个字符） | `awk 'NR==237\|\|NR==515'` 那一条（第十三节） | 「占比 0.75 的池与第 1 次重跑逐字节相同」核的就是这两个 `digests=` 字段的整值（第五节「占比 0.75 的池」） |
| 7 | 第 1 次重跑两段的 `name=preparation` 行 `wall_seconds` 之和：第一段 4 行 4189.07 秒、第二段 2 行 4188.11 秒；`name=intent` 行第一段 141、第二段 60，第一段其中 1 条 `valid=false` | 第十三节的 `grep`/`awk` 命令 | 只用来估第十四节的挂钟 |
| 8 | 调度记录：第二台在第 1 次重跑起跑前「负载 1.09、可用 55G」；第一段准备时「第二台最近 605 秒平均 6.1 核」；本机那时有用户的 `fstrim` 与 IO 压力 92.16、「第二台没有 fstrim、IO 压力 0」；比法那一段「七·6 之后 … α 在 0.994926–1.030719 … 七·6 之前 1.179549–1.231472」 | `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 202、209、212、218 行（`grep E185` 命中整行打出） | 机器状态那一格的参考；α 那一句是比法的结果，这一趟没有比法，不用 |
| 9 | Q3 在小批上的余量：f = 0.25、P = 1024 两格没打中的八轮 Q3 在 0.0076–0.1167，打中的两轮 0.403424、0.221050，一批 CPU 十几毫秒 | 调查员报告第 169 行 | 第一节未定项 2；不改门槛 |
| 10 | 自己推的：改法甲下 f = 0.25 每个 P 剩下 1 个 f 段（槽号最大的那一整段）；f = 0.75 次序不变，照旧 3 个；覆盖写的副本落进这次造池打出的旧洞的份数 = `overwritten_units − 24`：f = 0.25 是 752（P = 1024）、6128（P = 8192），f = 0.75 是 256、2048 | 第七节「推的预测」的算式，用第 4 行的 `overwritten_units` 与第三节环长 24 | 只报、拿来核机理（第八节），不是判据也不是锚点；不对上不作废 |
| 11 | 校准的代用量（本机，负载 18.47，`/proc/pressure/cpu` some avg10 2.86）：zlib 的 CRC-32（不是装置那份 CRC-32C）按线程 CPU 钟 200 次，4 MiB 中位 448046 ns、p99 ÷ 中位 1.035、最大 1.064；16 MiB 中位 1772754 ns、p99 ÷ 中位 1.011、最大 1.113 | 第十三节的 python 命令 | 校准取 16 MiB、连做 3 遍取中位（压掉单次的离群），作废门槛 1.10（R2-1）；第二台与装置自己的 CRC-32C 没量过（第一节未定项 4） |
| 12 | 自己算的锚点：16 MiB「第 i 字节 = i mod 251」缓冲的 CRC-32C 是 `0x3430be3b`，32 MiB 同法是 `0x84b65aa5`；追指针校准 2^24 项、64 MiB、`x ← (2654435761 x + 1) mod 2^24` 从 0 走 2^20 步落在 9437184 | 第十三节的 python 命令（同一份 CRC 实现对「123456789」给 `0xe3069283`） | 钉校准做的活（第七节乙类），不是门槛 |
| 13 | 线程 CPU 钟连读中位步长 170 ns（原登记 12.2）；本机 `getconf CLK_TCK` = 100（utime、stime 以 10 ms 为一格） | 原登记第 478 行；第十三节命令 | 校准（16 MiB 约几毫秒到十几毫秒，推的）用线程 CPU 钟计，分辨率够；utime、stime 只报不判，10 ms 一格写进第六节 |
| 14 | 第 1 次重跑第三段的 f = 0.25 那几行 `batch_bytes=16384`、`merged_buffer_bytes=16` 是几何没造成的点 | 执行员报告第 53 行 | 第 3 行沿用时那几行不算（第一节） |

## 五、臂、阳性对照、真实基线

**同上一次登记**：第 1、2 行的臂（对面那条臂写成支持它的人认的样子）、阳性对照（第一段第 1 行两条臂各在内存、真盘上跑，第二段甲、乙、丙三个阶段的 2 ms 对照与甲两遍），真实基线（`CompactionSwitches::PRODUCT` 只换批大小）、计时与计数怎么做、取样点（设备两种、K = 4、8、16、f = 0.25、0.75、P = 1024、8192、丙段盘大小两点）、轮数 5、并行与挂钟（12.10、12.11 的全局队列与串行段声明）照原登记第五节（第 193–312 行）与第十二节，一个字不改。第三、四段这一趟不跑（第一节）。下面只写这一趟改的。

### 装置写在哪（写死）

**入库装置**：仍是 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`（原登记第五节「装置住哪」写死的那一份）。理由同原登记：问题单问的是 `crates/` 这份 GC 代码的性质。**改前基线取主工作区今天那一版 sha256 `eadb929d6d8be037530f2ce94ee4a564202939f9c2d25e275023aa7ce091ae76`**（第三节；比 HEAD 的 `b3a50d50…` 多 3 行 `quarantine_mark`，第 1 次重跑第三段已用它，主 agent 已认）。改完、单测与变异跑完、第一份产物之前，执行员把改完的 sha256 写进第十二节，并由每一趟的输入清单第一行记下；两处不同就停（第十一节停机 7）。r1 登记第十一节「装置变了」那一条（写死 `b3a50d50…`）这一趟由停机 7 取代。

装置读 `crates/` 的常量照原登记第五节常量表回比，这一趟新加一个（执行员改成本地常量，开跑回比，对不上走原登记停机条款第 1 条）：

| 本地常量 | 回比的对象 | 值 |
|---|---|---|
| 根环条数（只用来打第七节「推的预测」那一栏） | `singlefs_format::ROOT_RING_REGIONS × singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM` | 24 |

### 这一趟装置改什么（执行员照写；每一处的产物行与键名在第六节）

| # | 改什么 | 为了哪一条 |
|---|---|---|
| 改动一 | 改法甲：把第 2263 行的排序换成一个纯函数（输入：占比、按段分好的要覆盖写的单元与它们的旧槽号、哪些段整段是这次写的；输出：覆盖写的次序），次序照第一节「改法甲」那一格；占比 0.75 时它交回的次序与 `sort_unstable` 逐项相同 | 派发第 1 条 |
| 改动二 | 新子命令 `geometry-check --point <活槽占比>:<池的数据量>… [--overwrite-order slot-order]`：只造池、核几何，打 `name=geometry_attempt`、`name=geometry`、`name=segment_composition`、`name=overwrite_landing`、`name=pool` 几行与末行 `name=done`，不量；`--overwrite-order slot-order` 照第 1 次重跑那一版造（全部按旧槽号升序，不数副本落点，第六节 N12 那一行写 `status=not_run`），给单点核与占比 0.75 的核（下面两小节） | 派发第 1 条 |
| 改动三 | 造池之后逐段打成分（N11）与覆盖写副本的落点数（N12）；`wall-clock`、`stage-split`、`geometry-check` 三个子命令都打，只打每个点头一次造池（`name=geometry_attempt` 那几次），第 2–5 轮的重造不打 | 派发第 2 条 |
| 改动四 | 每条意图前后的线程与机器读数（N3–N9）、每段开头、每个取样点开头与段末的机器状态（N10） | 派发第 2、4 条 |
| 改动五 | 校准（N1、N2、N15）与作废、重试的做法（第十一节 R2-1 至 R2-4）；`wall-clock`、`stage-split` 两段都照做，阳性对照那几轮也照做 | 派发第 2 条 |
| 改动六 | 丙段的判定：签发那一次 `name=stage` 行已经算出的 `choose_share_per_batch` 每轮收一个，按（f、P、K）五轮打一行 `name=choose_judgement`（第六节「丙段怎么有读数」） | 派发第 3 条 |
| 改动七（附带） | 「照原作废规则」的并列判定一行（N14），只报不判 | 派发第 2 条「原判据原样保留」 |

不改的：Q1–Q17 的算法、门槛、判定函数，原有的每一种产物行（`name=call`、`name=intent`、`name=stage`、`name=stage_judgement`、`name=q1` … 的键与取值），阳性对照、锚点、停机条款的判法，准备阶段的全局队列。新东西一律另起一行（新的 `name=`），不往已有的行里加键。

### 改法乙（后备，不并跑）

甲的单点核判「没造成」（下一小节）时停机交主 agent（第十一节停机 8）；主 agent 定改用乙，执行员才把乙改进装置，在第十二节写一条修订（时点在产物之前），再从单点核重来。乙只对占比 0.25：造池第 1 步（顺序写完 P 个单元）之后发一次 `UserChange::CreateSnapshot`（`mounted_session.rs:118`），第 2 步全部覆盖写之后、第 3 步补满前沿之前发 `UserChange::DestroySnapshot { snapshot }`（`mounted_session.rs:124`），之后照旧第 3、4 步。快照留住旧单元，覆盖写期间旧洞不释放，副本全落在前沿；删快照之后旧单元一起释放，释放轮里一起回收，之后没有数据写（调查员报告第 72 行）。推的结果：每一整段都是 f 段（f = 0.25：P = 1024 约 31 段、P = 8192 约 255 段）。代价写在这里，用乙时照报：负载多了建快照、删快照的几次发布；`mapped_units` 与节点段的穿插会变；锚点（第七节乙类）只依赖 R 的成分，不变。

### 单点核：甲造没造成（第一、二段之前先跑一次，连编约十几分钟）

在起跑那棵树里、编完之后，先跑两条（各自一份产物，见「产物」）：

1. `geometry-check --point 0.25:1024 --point 0.75:1024`（改法甲）；
2. `geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order`（第 1 次重跑那种造法，对照）。

判法（读第 1 条的产物）：

| 格 | 读哪一行 | 成立的样子 | 不成立怎么办 |
|---|---|---|---|
| 甲造成 | `name=geometry fraction=0.25 pool_data_units=1024 …` 的 `formed`；同一点 `name=segment_composition` 里 `region_start_slot` 等于 `name=geometry` 点名的那一段（`choice=greedy` 时是 `region_start_slot`，`choice=forced_fallback` 时是 `fallback_region_start_slot`）的那一行 | `formed=true`；那一行 `live_slots=16 slots_of_this_workload=16 other_slots=0` | 停（第十一节停机 8），后面的段一段都不跑，交主 agent 定用不用乙 |
| 占比 0.75 在同一棵树里没被改动带歪 | 两份产物里 `name=pool fraction=0.75 pool_data_units=1024` 那一行的 `digests=` 整值 | 两份相同 | 停（第十一节停机 9） |
| 对照：第 1 次重跑的「没造成」在今天的树上重现 | 第 2 条产物 `name=geometry_attempt fraction=0.25 pool_data_units=1024` 的 `geometry_formed_segments` | 推的：0（与第 1 次重跑第 39 行相同） | 不停：照实记「旧造法在今天的树上也造得出 f 段」，机理那一句（第八节）记不成立，交主 agent |

单点核只核 P = 1024：P = 8192、f = 0.25 造一次约 2000 秒（第四节第 1 行引原登记 12.11 的数），放进单点核会把它拖成半小时以上；改法甲留下的那一段与 P 无关（第七节「推的预测」），P = 8192 那一点在第一、二段里照「几何没造成」原条款判。

### 占比 0.75 的池与第 1 次重跑逐字节相同：怎么核

两道，各报各的，第二道兜住「两棵树本来就不同」：

1. **与第 1 次重跑比（派发点名的那一道）**：第 1 次重跑第一段第 237、515 行的 `digests=` 整值，与这一趟 `name=pool fraction=0.75` 两行（P = 1024 取单点核第 1 条产物与第一段产物各一次，P = 8192 取第一段产物）的 `digests=` 整值逐字比。命令（只取 P 与摘要两样，摘要是整值、不截）：
   `awk 'NR==237||NR==515' research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'`，对这一趟的产物换成 `grep '^E7RESULT name=pool fraction=0.75 ' <产物> | sed -E '同上'`，两份 `diff`。
2. **同一棵树里比**：单点核第 1、2 条两份产物 `name=pool fraction=0.75 pool_data_units=1024` 的 `digests=` 相同（单点核表第 2 格）；另有两条单测（第九节 V57、V61 盯着）：占比 0.75 时改动一的次序函数与 `sort_unstable` 逐项相同；P = 128 的小池（装置单测已用的规模）在占比 0.75 下开着与关着改动三的计数，摘要逐块相同。

判法：第 1 道相同就记「占比 0.75 的池与第 1 次重跑逐字节相同」。第 1 道不同而第 2 道相同，记「占比 0.75 的池与第 1 次重跑不同，同一棵树里改动没有带歪它；差在两棵树（第三节：核心层五份源文件与 `Cargo.lock` 的 sha256 与第 1 次重跑输入清单不同）」，不停，占比 0.75 的点照量，这一趟的数不与第 1 次重跑的数混用。第 2 道不同就停（第十一节停机 9）。推的：核心层这几天改过（第三节），第 1 道不相同的可能不小；它不相同不是停机的理由。

### 清场：计时期间两台怎么清、怎么记

- **怎么清**：每一趟（单点核、第一段、第二段各一趟）整条命令经 `bash scripts/crash/gpu-quiesce-run.sh -- …` 起：它照多机配置的私有清场命令停白名单里的服务并回读，回读没过退 78、命令不起，那一趟记「本次未跑」；跑完不复原（`.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」）。它清不清第二台，读的是 `gpu-quiesce-run.sh` 第 33 行「第二台的回读不变」（推的，私有脚本没读；第一节未定项 3，主 agent 起跑前核）。
- **计时期间不往第二台派活**：主 agent 起跑前告诉别的会话与在跑的子 agent（同第 1 次重跑，调度记录第 206 行的做法），连同不往第二台的卡派活（崩溃放量双机分项的 `peer:` 卡）；装置照原登记第十一节「机器被别的活挤」在每轮开轮前看第二台上有没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。本机上的驱动每 5 秒经 ssh 看一次、每 300 秒在第二台跑一次 `core_usage.py sample`（`scripts/hosts/multi-host-run.sh` 文件头第 26–32 行），这两样照旧，它们落在哪一刻由 N9 的单调钟秒数对得上。
- **怎么记进产物**：① 包装打到标准错误的 `GPU_QUIESCE step=quiesce …`、`GPU_QUIESCE step=command …`、`GPU_QUIESCE step=restore …`、`GPU_QUIESCE step=exit …` 四行原样存进 `research/results/e185-r2-quiesce-<段>-<日期>.txt`（主 agent 起跑时把驱动的标准错误另存一份，跑完 `grep '^GPU_QUIESCE'`；私有服务名不在这几行里，`gpu-quiesce-run.sh` 文件头第 15 行）；② 装置在段开头、每个取样点开头与段末各打一行 `name=machine_state`（N10：整机忙的比例与本进程占的比例，其余只记总量、写「其余不归 singlefs」，不打进程名）；③ 每条意图前后的线程与机器读数（N3–N9）。

### 跑法（照现行脚本写；主 agent 起跑前核这几份脚本有没有变）

派发第 6 条：预开发会话在改起编译、拷库、跑重活的脚本，主 agent 收到它的通知之前不起 `multi-host-run.sh`。下面照今天的 `scripts/hosts/multi-host-run.sh`、`scripts/crash/gpu-quiesce-run.sh`、`scripts/hosts/run-with-memory-cap.sh` 写；起跑前主 agent 核这三份与它们的用法（`--peer`、`--memory-cap`、`--label`、`--fetch`，编译目录怎么给）有没有变，变了照新的写法起，改的只是起法、不是装置的参数，记进第十二节一行。

- **机器**：三趟都在第二台、先后跑、不同时跑；真盘臂的镜像放第二台那块盘（第 1 次重跑输入清单第 24 行：型号 CT2000T500SSD8，逻辑块 512）上的 ext4、不放开机清空的目录（同第 1 次重跑）。
- **每一趟一条命令**（`<段>` 取 `geometry-check`、`wall-clock`、`stage-split`；`<日期>` 是那一趟起跑当天 `TZ=Asia/Tokyo date +%F`）：
  `bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap 46G --label e185-r2-<段>-<日期> --fetch research/results/<产物名>… -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段>`
  `scripts/experiment-apparatus/e185-r2-segment-run.sh`（随树拷过去），做四步：写输入清单 → 在那棵树自己的编译目录里 `cargo build --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split`（不带 `gpu` 特性，不带 `--all-targets`、`--tests`）→ 跑那一段 → 产物写进 `research/results/`（`--fetch` 逐个点名取回）。树与编译目录随 `multi-host-run.sh` 第 ⑦ 步删掉。
- **上限与线程**：同第 1 次重跑：外层 `--memory-cap 46G`、`E185_MEMORY_CAP=40G`；`E185_THREADS` 不设，装置取登记值 `.claude/heavy-thread-default`（今天 31）；`SINGLEFS_SERIAL_DECLARATIONS` 由 `multi-host-run.sh --peer` 设。
- **单测与变异**：装置改完由主 agent 带提交前缀在主工作区跑（checker 档，`cargo test --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split` 与 `crates/mutations.tsv` 里 `E185 V` 那几行），全绿、变异全抓之后才起第一趟；条数写进第十二节。
- **次序**：单点核 → 第一段 → 第二段。单点核停机（停机 8、9）就不起后两趟。第一段先于第二段的理由同 r1 登记第五节：第二段的判定挂在第一段的 S 上（第一节），次序不影响任何判定。

### 输入清单（每一趟一份，起跑前在起跑的那棵树里写，跑完不改）

`research/results/e185-r2-inputs-<段>-<日期>.txt`，逐行：装置 sha256（第一行）；`crates/singlefs-core/src/` 下 `transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs`、`checksum.rs`、`block_device.rs`、`allocator.rs` 与 `crates/singlefs-format/src/lib.rs`、`Cargo.lock` 的 sha256（比第 1 次重跑多 `allocator.rs` 与 `singlefs-format/src/lib.rs` 两份：几何靠它们）；树的来历（`multi-host-run.sh` 拷树之后 `git init` 那一次的提交号，写明树拷自哪棵）；`cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline` 原样；镜像目录所在盘的型号与逻辑块大小；`nproc`；`getconf CLK_TCK`；两样上限。第二台的主机名、路径照第 1 次重跑的写法换成「〔第二台的目录〕」。

### 产物（带日期，不盖已有的；同一天同一段再跑加 `-2`、`-3`）

| 段 | 产物 | 补问题单哪一行 | 补完那一行够不够判 |
|---|---|---|---|
| 单点核 | `research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out`、`research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out` | 不补（停机用，第 1 行的前提） | 不适用 |
| 第一段 `wall-clock` | `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-<日期>.out` | 第 1 行（Q1–Q4）；第 4 行的 Q14–Q16 只并列报 | 第 1 行照第一节「第 1 行够判」那一格数 |
| 第二段 `stage-split` | `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-<日期>.out` | 第 2 行（Q5–Q8，丙段的 Q7 读 `name=choose_judgement`） | 第 2 行照第一节「第 2 行够判」那一格数；Q9 照第一节交主 agent |
| 第三、四段 | 不跑 | 第 3、4 行沿用（第一节） | 已够判 |

分段不是「全部做完」的计划：单点核交回之后主 agent 判续不续；第一段交回之后，第 1 行若因 f = 0.25 两点都没造成而注定不够判（S 里没有 f = 0.25 的格），主 agent 判第二段还跑不跑（第二段的判定挂在 S 上）。

## 六、报哪些量与各自的判据

**同上一次登记**：Q1–Q17 每一行怎么算、门槛、判定、对应问题单哪一行与翻面取值照原登记第六节（第 314–338 行）与 12.5、12.6 的产物键名，一个字不改；每个量照旧各报各的判定，5 轮判定不全相同写「不稳定」。Q9 照第一节（r1 登记第六节「Q9 一格交主 agent」原样沿用，这一趟不重算）。r1 登记第六节的「并列报」C0–C3（七·6 前后的比法）这一趟没有：这一趟不比七·6 前后，问题单也没有一行靠它翻面。

### 丙段怎么有读数（派发第 3 条）

Q7 的算法照原登记第六节第 326 行逐字：「签发那一步全部 `choose_region` CPU ÷ 这条意图的批数，再 ÷ 一批 CPU；另报签发那一次的原值」，门槛「摊到每批 ≥ 10% 判「值得」；< 10% 判「不值」」。装置第 6488 行起已经照它算了（「一批 CPU」取这条意图批 CPU 的中位，这是装置从原登记起就用的读法，这里写死、不改），值在签发那一次 `name=stage` 行的 `choose_share_per_batch`；第 1 次重跑没有判定，是因为判定只从批行收（第三节）。这一趟（改动六）：每轮取那一轮有效意图签发那一行的 `choose_share_per_batch`，按原门槛判，五轮收成一行

`name=choose_judgement fraction=… pool_data_units=… batch_units=… rounds=… choose_share_per_batch_per_round=… choose_share_judgement_per_round=… choose_share_judgement=…`

五轮相同写那个判定，不全相同写 `unstable`，某轮读不到写那一轮 `not_run`（不当 0）。`name=stage_judgement` 行照旧打、格式不变（它的丙段那一格仍是 `not_run`，不读它）；第 2 行与原登记 F7 的丙段都读 `name=choose_judgement`。第 2 行丙段的翻面照原登记：Q6 已判值得时，Q7 从 < 10% 越过 10% 让选择从乙翻到丙。所以这一趟丙段有读数，不交主 agent；交主 agent 的只剩 Q9 那一格（第一节）。

### 这一趟新加的量（全部在计时窗口外读；读不到写 `none`，不当 0）

| 量 | 怎么算（产物行与键） | 门槛与判定 | 对应问题单哪一行，或作废用 |
|---|---|---|---|
| N1 校准比（CRC） | 每条意图（含阳性对照那几轮、第二段的意图）调用 `measure_*` 之前与刚返回之后，测量线程对一块 16 MiB 缓冲（第 i 字节 = i mod 251，段开头建一次、之后不改）连做 3 遍 `singlefs_core::checksum::crc32_castagnoli`，按线程 CPU 钟各计一遍、取中位；÷ N2 段开头那一块的中位。产物新行 `name=intent_machine_readings`（键同那条意图的 `name=intent`：`arm`、`fraction`、`pool_data_units`、`batch_units`、`round`、`attempt`、`injection`）里的 `calibration_before_nanoseconds`、`calibration_after_nanoseconds`、`calibration_before_ratio`、`calibration_after_ratio`；每次核 CRC 值（第七节） | 前后两个比有一个 > 1.10，那一条意图作废（R2-1）；≤ 1.10 不作废 | 作废用（原登记第十一节「机器被别的活挤」，R2-1 修订） |
| N2 校准的基线与漂移 | 段开头（时钟分辨率自检之后、准备之前）与段末各连做 21 遍 N1 那份活，各取中位；`name=calibration_baseline position=start\|end runs=21 median_nanoseconds=… ratio_of_the_end_to_the_start=…`（末那一行带比） | 末 ÷ 头 不在 [0.90, 1.10] 里，写 `drifted=true`（R2-4） | 作废用（R2-4） |
| N3 排队等 CPU | `/proc/thread-self/schedstat` 第二个字段在一条意图前后之差：`run_queue_wait_nanoseconds`（同一行） | 只报不判 | 作废用的读数：分开「机器被别的活挤」的两种（别的进程抢 CPU ／ 内核回收），F5 打中时先看它（第十条 H3） |
| N4 非自愿切换 | `/proc/thread-self/status` 的 `nonvoluntary_ctxt_switches` 之差：`involuntary_switches` | 只报不判 | 同 N3 |
| N5 用户态与系统态 | `/proc/thread-self/stat` 第 14、15 个字段（从最后一个 `)` 之后数，进程名里有空格与括号也不错位）之差：`user_ticks`、`system_ticks`，另报 `clock_ticks_per_second`（`sysconf(_SC_CLK_TCK)`；本机 100，一格 10 ms） | 只报不判 | 同 N3 |
| N6 缺页 | 同一行第 10、12 个字段之差：`minor_faults`、`major_faults` | 只报不判 | 同 N3 |
| N7 压力与负载 | 意图前后各读 `/proc/pressure/cpu`、`/proc/pressure/memory` 的 `some avg10` 与 `/proc/loadavg` 第一个数：`cpu_pressure_some_ten_seconds_before`、`…_after`、`memory_pressure_some_ten_seconds_before`、`…_after`、`load_average_one_minute_before`、`…_after` | 只报不判 | 同 N3 |
| N8 在哪个 CPU 上 | 意图前后各调一次 `sched_getcpu`：`cpu_number_before`、`cpu_number_after` | 只报不判 | 同 N3 |
| N9 时刻 | 意图前后相对段开头的单调钟秒数（`Instant`，三位小数）：`seconds_since_the_segment_start_before`、`…_after` | 只报不判 | 同 N3；与第二台的系统日志、驱动的采样对时 |
| N10 机器状态 | 段开头、每个取样点开头（准备完、量之前）、段末各一行 `name=machine_state position=…`：读 `/proc/stat` 首行与本进程 `/proc/self/stat` 的 utime + stime，隔 1 秒再读一次，报 `machine_busy_share`（全机非空闲 jiffies 增量 ÷ 全部 jiffies 增量）、`this_process_share`（本进程增量 ÷ 全部）、`others_share`（两者之差，「其余不归 singlefs」只记这个总量，不打进程名）、`memory_available_bytes`（`/proc/meminfo`）、N7 那三样 | 只报不判 | 派发第 4 条（清场之后计时期间机器上还剩什么）；作废用的读数 |
| N11 逐段成分 | 每个点头一次造池之后、对单元区里每一段（有未释放记录的段）打一行 `name=segment_composition fraction=… pool_data_units=… overwrite_order=… region_start_slot=… live_slots=… slots_of_this_workload=… other_slots=… formed=…`，取法同装置第 2592 行用的成分；另打一行 `name=segment_composition_summary … segments=… formed_segments=…`（`formed_segments` 应等于 `name=geometry_attempt` 的 `geometry_formed_segments`） | 只报不判；单点核用它判甲造成（第五节） | 停机用（停机 8）；「几何没造成」作废条款的读数 |
| N12 副本落点 | 造池第 2 步每次覆盖写之后，看这一份新副本的槽是不是这次造池前面某次覆盖写空出来的旧槽：`name=overwrite_landing fraction=… pool_data_units=… overwrite_order=… overwritten_units=… copies_landing_in_holes_of_this_construction=… copies_landing_elsewhere=… predicted_copies_landing_in_holes=…`（预测 = `overwritten_units − 24`，24 是第五节的本地常量）；`--overwrite-order slot-order` 时不数，写 `status=not_run` | 只报不判 | 核机理（第八节）；不是作废条件 |
| N13 丙段判定 | 上一小节 `name=choose_judgement` | 原登记 Q7 的门槛（10%） | 第 2 行丙段（翻面取值照原登记 Q7） |
| N14（附带）照原作废规则的并列判定 | 每个（f、P、K）格一行 `name=judgements_under_the_original_voiding`：每轮取「照原登记第十一节（不加 R2-1）会被收下的那一次」——即只因 N1 作废的那一次照原规则算有效——再照原判定函数算 `rho_judgement`、`wall_ratio_judgement`、`reconciliation_judgement`（第一段）或 `crc_share_judgement`、`scan_increment_share_judgement`、`choose_share_judgement`（第二段） | 只报不判 | 不对应岔路行；给主 agent 看 R2-1 动了哪几格。够判后可以不读 |
| N15 校准比（追指针） | 与 N1 同时、同一条线程：64 MiB 的 `u32` 下标表（第 x 项 = (2654435761 x + 1) mod 2^24，段开头建一次），从 0 起每步取表里那一项当下一个下标，走 2^20 步，按线程 CPU 钟计；÷ 段开头 21 遍的中位。同一行 `chase_before_ratio`、`chase_after_ratio`，核终点下标（第七节） | 只报不判 | 作废用的读数：CRC 那份活吃不到内存时延，N1 在 1 附近而对象 CPU 翻倍时，看它是不是也涨了（第十条 H3） |

门槛旁边写它从哪来：N1、N2 的 1.10 与 0.90 是登记时定的政策线，取法同原登记作废条款已有的「未入账超过挂钟 10%」那一条的 10%；第四节第 11 行的代用量里单次的 p99 ÷ 中位 1.011、最大 1.113，取 3 遍的中位后单次离群压得掉（推的）。没有一个门槛从臂的定义推得出（这几个量都不判臂）。

## 七、钉绝对值的断言

**同上一次登记**：甲类（出自被测条款本身：Q4 批大小翻倍挂钟跟着涨、Q9 取 16 线程饱和吞吐，不符走原登记 F1、F2）与乙类（独立算出、命令核过，不符才作废）照原登记第七节（第 340–368 行）与 12.5 补的那一条，一个字不改；装置逐批核；乙类只在「几何造成了」的点上判（R 里全是码 1 数据单元、活槽 = 64f）。f = 0.25 造成之后，原登记乙类里 f = 0.25 那几格第一次真的被判到：签发那一步扫 R 读 262144 字节；一条意图的批数 K = 4 → 2、K = 8 → 1、K = 16 → 1；签发加各批扫描读字节之和 655360、524288、524288；第一批搬迁副本字节 262144、524288、524288；一条意图搬迁副本之和 524288（原登记第 356–360 行，算式与输出在原登记第 681–719 行）。

**新加，乙类**（独立算出、第十三节的 python 命令核过；不符时那一份校准读数作废——N1 不符那一条意图作废重量，N2、R2-3 不符那一段停，见第十一节）：

| 断言 | 值 | 在哪核 |
|---|---|---|
| N1 每一遍校准算出的 CRC-32C（16 MiB，第 i 字节 = i mod 251） | `0x3430be3b` | 装置每一遍都核（校准的活确实做了、没被优化掉） |
| R2-3 加倍那一遍（32 MiB，同法） | `0x84b65aa5` | 段开头那一次 |
| N15 追指针从 0 走 2^20 步的终点下标（2^24 项，`x ← (2654435761 x + 1) mod 2^24`） | 9437184 | 装置每一遍都核 |
| 同一份 CRC 实现对「123456789」 | `0xe3069283`（与原登记第 365 行同值） | 第十三节命令（核 python 那份 CRC 实现本身） |
| 单点核：改法甲下 f = 0.25、P = 1024 选中那一段的成分 | `live_slots=16 slots_of_this_workload=16 other_slots=0`（= 原登记乙类的前提「R 全是码 1、活槽 = 64f」，64 × 0.25 = 16） | 单点核产物（第五节） |
| `name=segment_composition_summary` 的 `formed_segments` 与同一点 `name=geometry_attempt` 的 `geometry_formed_segments` | 相等 | 每一个点（两条路数同一个东西：前者按 N11 逐段数，后者是几何核原来那条路） |

**推的预测（不是锚点，不符不作废；拿来核调查员推的机理，第八节）**：设每次覆盖写恰好一次发布、每次发布写一条根、环 24 条根、第 j 次覆盖写打出的洞从第 j + 24 次发布起分得到、用户数据取最低的空槽对（第三节各行）。那么覆盖写到第 25 次起每一份副本都落进 24 次之前打出的洞，全程落进旧洞的副本 = `overwritten_units − 24`；造完时还空着的洞正好是最后 24 次覆盖写打出的。改法甲下 f = 0.25 最后 24 次正是槽号最大那一整段的 24 个，所以那一段剩 16 个活槽、全是这次写的，f 段 = 1（与 P 无关）；f = 0.75 每段 8 个洞，最后 24 次是最后三整段，f 段 = 3（第 1 次重跑量到的也是 3，第四节第 4 行）。按第四节第 4 行的 `overwritten_units`：落进旧洞的副本 f = 0.25 是 752、6128，f = 0.75 是 256、2048。改法甲那一段比环长没有余量（24 个洞 = 24 条根）；余量只来自「最后一段挑槽号最大的」：第 N 次发布时更低的那一段的洞先被分走（调查员报告第 68 行）。

互比旁边的绝对值：N1 是比值，旁边是每一遍的 CRC 值；N13（Q7）是份额，旁边是原登记乙类「一条意图的批数」（份额的分母之一）。

## 八、轨迹与几何敏感性

**同上一次登记**：取样点、轨迹表（B_批、每批写带宽、每批 ρ、每批等盘的峰值、为正的批数、期末值）、几何敏感性（Q1、Q2、Q4、Q5、Q6、Q7、Q13、Q16 在每个旋钮两端各报一次）与判别力自证照原登记第八节（第 370–401 行），不改。f = 0.25 造成之后，「f 旋钮两端」第一次两端都有数。

**新加的轨迹**（被作废谓词消费的量，每段各报一行 `name=calibration_trajectory`）：

| 量 | 峰值 | 为正的条数 | 期末值 |
|---|---|---|---|
| N1 前后两个比里大的那个 | 全段最大 | > 1.10 的意图条数（另报 > 1.05、> 1.20 的条数：门槛挪一档会多作废、少作废几条，只报） | 段里最后一条意图的 |
| R2-2 等常态的秒数 | 单次最长 | 等过的次数 | 全段累计（上限 600 秒） |

**机理核对（只报，第十条 H2）**：N12 的 `copies_landing_in_holes_of_this_construction` 与 `predicted_copies_landing_in_holes` 并列；f = 0.25 改法甲下 `formed_segments` 与预测的 1 并列；单点核对照那条 f = 0.25 `geometry_formed_segments` 与预测的 0 并列。三样都对上，记「调查员推的机理在这几个点上对上了」；有一样对不上，写哪一样差多少，机理那一句不当已证。

**作废门槛的敏感性**：N14 那一行就是「不加 R2-1」那一端；S（第 1 行够判那一格）照 N14 再数一遍并列报，两种数法 S 不同，写出差在哪几格。

## 九、变异

**同上一次登记**：`crates/mutations.tsv` 里 `E185 V` 起头的 55 行（V1–V55，`grep -c 'E185 V' crates/mutations.tsv` 输出 55）照原登记第九节与 12.1–12.11，不增不删；改动一至七碰到它们锚点的那几行，锚点改到新源码里逐字命中一次（`.claude/rules/mutation-sampling.md` 第七类），点名的单测不换。

**新加**（改的都是装置自己的代码；追加进 `crates/mutations.tsv` 的 E185 行之后，名字接着编 V56 起；每条点名一条会红的单测；常量断言写加法不写减法）。执行员可以补，不许删。

| 变异 | 改哪 | 在哪个取样点改变输出 | 该红的 |
|---|---|---|---|
| V56 甲不生效 | 次序函数对占比 0.25 也交回按旧槽号升序 | 单测里手搭的分段输入（两个混住段、三个整段，占比 0.25）：最后 24 项应全是槽号最大那一整段的，变异下最后几项是混住段的 | 次序函数单测；真跑时单点核「甲造成」那一格 |
| V57 甲漏到 0.75 | 次序函数对占比 0.75 也照甲排 | 同一份手搭输入、占比 0.75 | 「占比 0.75 时与 `sort_unstable` 逐项相同」单测；真跑时单点核第 2 格 |
| V58 留到最后的是槽号最小的整段 | 第 ③ 组取槽号最小的那一整段 | 同一份手搭输入（三个整段起点不同） | 次序函数单测 |
| V59 混住段排到最后 | 第 ① 组放到第 ③ 组之后 | 同上（有混住段的那一份） | 次序函数单测 |
| V60 落点把前沿也数进旧洞 | `copies_landing_in_holes_of_this_construction` 不看「是不是这次造池前面空出的旧槽」 | 单测里合成的一串（旧槽集合、每次副本的新槽，前 24 份落前沿） | 落点计数单测 |
| V61 计数改动带歪了池 | 改动三的计数在造池途中多发一次发布（例：数落点时 `publish_while_constructing` 一个空改动） | P = 128、占比 0.75 的小池 | 「开着与关着计数，摘要逐块相同」单测 |
| V62 成分两栏对调 | `slots_of_this_workload` 与 `other_slots` 互换 | 手搭一段 16 个这次写的槽、2 个别的槽 | 成分单测；真跑时第七节「两条路数同一个东西」 |
| V63 校准比倒过来 | 比 = 基线 ÷ 这一次 | 合成：基线 1 ms、这一次 1.2 ms（应 1.2 作废，变异 0.833 不作废） | 作废判法单测 |
| V64 等号含错一侧 | `> 1.10` 写成 `>= 1.10` | 合成比正好 1.10（应不作废） | 作废判法边界单测 |
| V65 只看前一个比 | 作废只看 `calibration_before_ratio` | 合成：前 1.00、后 1.30（应作废） | 作废判法单测 |
| V66 作废后当场重试 | R2-2 的等去掉，直接 `continue` | 假的校准源：作废后头两次读 1.30、1.25，第三、四次 1.02、1.03（应等到第四次才重试） | 等常态的单测（假钟与假校准源，不真睡） |
| V67 等不设上限 | 去掉全段 600 秒的上限 | 假校准源一直读 1.30 | 「累计满 600 秒退 5」单测 |
| V68 校准算的不是那份活 | 缓冲第 i 字节写成 i mod 256 | 段开头那一遍 | CRC 值锚点 `0x3430be3b` |
| V69 漂移不判 | R2-4 的区间判法去掉 | 合成头 1 ms、末 1.2 ms | 漂移单测 |
| V70 丙段判定还从批行收 | `name=choose_judgement` 读批行的 `choose_share_per_batch` | 合成一条意图：签发行 0.15、批行空 | 丙段判定单测（应 `worth`，变异 `not_run`） |
| V71 丙段取成签发那一次的原值 | 读 `choose_share_of_the_issue` | 合成：签发行 `choose_share_per_batch` 0.05、`choose_share_of_the_issue` 0.15（应 `not_worth`） | 丙段判定单测 |
| V72 排队时间读错字段 | schedstat 读第一个字段 | 合成 schedstat 文本「100 200 3」（应 200） | 读法单测 |
| V73 stat 字段错位 | 从第一个空格起数字段（不从最后一个 `)` 之后数） | 合成 stat 行，进程名是 `(a b) c)` | 读法单测 |
| V74 读不到当 0 | 文件不在时交回 0 | 不在的路径 | 读法单测（应 `none`） |
| V75 并列判定也剔掉校准作废的那一次 | N14 取第一份「照新规则有效」的 | 合成一轮：第 1 次只因 N1 作废、第 2 次有效，两次 CPU 差一倍 | N14 单测（应取第 1 次） |
| V76 加倍对照的区间判反 | R2-3 判 `[1.8, 2.2]` 之外为过 | 合成 1 ms 与 2 ms | 加倍对照单测 |

单测是新的计数实验，照 `.claude/singlefs-ai-sop/rules/show-me-test.md` 先证明会红（每条变异打开时红在点名的那一条）。

## 十、失败条款

**同上一次登记**：F1–F8 照原登记第十条（第 427–440 行），不改；F7 的丙段读 `name=choose_judgement`（第六节）。r1 登记的 G1–G6 只管七·6 前后的比法，这一趟没有比法，不适用。

**新加**（每条紧跟触发的观测）：

| 条款 | 判成什么 | 什么观测会让它触发 |
|---|---|---|
| H1 甲造不出 | 停（第十一节停机 8），交主 agent 定用不用乙；不算负载的结论，也不算第 1 行的读数 | 单点核第 1 条产物里 `name=geometry fraction=0.25 pool_data_units=1024` 写 `formed=false`，或它点名那一段的 `name=segment_composition` 不是 `live_slots=16 slots_of_this_workload=16 other_slots=0` |
| H2 机理对不上 | 不作废、不停：照实记哪一样对不上（第八节「机理核对」），「洞被这次写的副本填回」那一句不当已证；几何造成了的点照量 | `copies_landing_in_holes_of_this_construction ≠ predicted_copies_landing_in_holes`；或改法甲下 f = 0.25 的 `formed_segments ≠ 1`；或单点核对照那条 f = 0.25 的 `geometry_formed_segments ≠ 0` |
| H3 F5 又打中、而机器读数平静 | 正当结果，不作废：那一格照原登记 F5 不进 S；照调查员报告第 163 行写的推翻条件记「机器那一刻的状态」这一解释在这一格上不成立，要回到对象自身查，交主 agent | 某格 Q3 有一轮 > 20%，而那一轮两条臂的 N1、N15 前后比都在 [0.95, 1.05]，N3、N5 的 `system_ticks`、N6、N7 与同格别的轮同量级（各在同格中位的 2 倍以内），对象一批的 CPU 却是同格中位的 1.5 倍以上 |
| H4 F5 又打中、而设备调用里的差大 | 正当结果：照调查员报告第 164 行，「内存盘拷贝开销或系统调用」那一支回来，交主 agent | 某格 Q3 打中的那一轮，设备调用里的 CPU 两臂之差超过内存臂挂钟的 20%（`name=call` 行已有的 `device_cpu_nanoseconds`，两臂整条意图之和相减） |
| H5 轮数不足 | 那一格写「轮数不足」，不下判定、不进 S，不当阴性；交主 agent 定补不补 | 某格的 `name=round_invalid` 行（三次都作废）使有效轮少于 5 |
| H6 第 1 行仍不够判 | 正当结果，照实记 S 缺哪几格、各因哪一条（几何没造成、F5、H5），交主 agent | 数完 S，某个 f 在 S 里不到两个 K |
| H7 第 4 行单跑那一半翻面 | 不重判第 4 行（第一节），照实并列报，交主 agent 定要不要重开 | 这一趟 `name=q16` 判 `needs_the_concurrent_measurement` |
| H8 校准基线漂移 | 不作废：那一段 N1 作废依据的基线不代表整段，写进报告；那一段的 N14 并列判定与正式判定都照报，主 agent 定信哪一个 | N2 `drifted=true` |

没有一条把「没测出差别」写成实现有 bug：实现对不对不是这一实验判的；装置与实现对不上走第十一节停机条款。

## 十一、作废条款与停机条款

### 作废

**同上一次登记**：原登记第十一节作废八条（阳性对照红、GPU 结果错、GPU 计时没等到、读不到、机器被别的活挤、起始镜像不同、几何没造成、包装出局）与 12.10 第 ③ 条（扔页缓存不成）照沿用，原文不改；r1 登记第十一节「计数特性被并进来」（输入清单 `cargo tree` 那一段出现 `lookup-work-counters` 就两段产物整段作废）照沿用。

**修订（相对原登记第十一节；写于 2026-10-23，装置改之前、产物之前；只收严，原判据与原作废条款的原文都保留）**：

| 修订 | 改了什么 | 收严在哪 | 依据 |
|---|---|---|---|
| R2-1 校准比接进「机器被别的活挤」 | 原条款「那一轮作废、重跑」的触发观测原文两样（「包含自检里「挂钟 − CPU − 等盘」超过挂钟 10%」；「开轮前 `ps` 看到 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio` 在跑」）照旧，再加第三样：「这条意图前后的 N1 校准比有一个 > 1.10」。作废的那一条照打 `valid=false invalid_reason=calibration_ratio_above_the_registered_edge`，另报两个比 | 多一种作废：线程在 CPU 上、却跑得比段开头慢一成以上的那几条意图（第 1 次重跑第 95 条那种「未入账 3.1%、CPU 2.316 倍」被收下的）不再收下。判据、门槛一个不动；改的是哪几轮进判定 | 调查员报告第 135、250 行；第四节第 5 行 |
| R2-2 作废之后先等常态再重试 | 因「未入账超过挂钟 10%」或 R2-1 作废的，不当场重量：每 10 秒做一次 N1 的校准，连着两次 ≤ 1.10 才重量；单次至多等 300 秒，等满照样重量（重量的那一次照样受 R2-1 判）；一段里累计等满 600 秒，打 `name=stopped reason=machine_not_back_to_normal_within_the_waiting_cap`、收尾、退 5（装置文件头已有的「等满上限仍没等到」那个码），那一段产物不完整，交主 agent。别的作废原因（挂载失败、钟读不到、扫描读为 0、拷回不成）照旧当场重量 | 原来作废后紧接着重量，慢段里的第二次多半还慢；改了之后重量那一次落在常态里。重试次数仍至多 3（`ATTEMPTS_PER_ROUND_AT_MOST`），轮数仍 5 | 调查员报告第 250 行（甲） |
| R2-3 校准自己的阳性对照 | 每段开头、N2 头一块之后做一遍加倍的活（32 MiB，同法），它的线程 CPU ÷ N2 头一块的中位要落在 [1.8, 2.2]，CRC 要是 `0x84b65aa5`；产物 `name=positive_control row=calibration ratio=… crc_holds=… holds=…`。不过就打 `name=stopped reason=calibration_cannot_tell_twice_the_work`、退 4，那一段一个点都不量 | 多一种停：校准分不出两倍的活时，R2-1 判不了，不拿它作废 | 调查员报告第 249 行「把校准的活加倍，比值要跟着翻倍」 |
| R2-4 校准基线漂移 | N2 段末那一块 ÷ 段开头那一块不在 [0.90, 1.10]，写 `drifted=true`（第十条 H8） | 只多一个标记，不放宽 | 段开头若正落在慢段里，R2-1 的基线就偏大 |

### 停机（装置与 `crates/` 对不上，或这一趟的前提不成立：两边都查，既不作废也不当结果，`.claude/rules/implementation-first.md` 第 4 条）

**同上一次登记**：原登记停机 1–6，12.2 的时钟分辨率自检、12.10 的线程核、12.11 的串行声明写不进，照原登记，不改。新加：

7. 装置变了：输入清单第一行的装置 sha256 与第十二节执行员写的那一个不同（取代 r1 登记第十一节写死 `b3a50d50…` 的那一条）。
8. 甲造不出（第十条 H1）：单点核第 1 条产物判「没造成」，第一、二段不起。
9. 占比 0.75 被改动带歪：单点核第 1、2 条产物的 `name=pool fraction=0.75 pool_data_units=1024` 两个 `digests=` 不同（第五节「怎么核」第 2 道），第一、二段不起。
10. 校准锚点不符：N1 某一遍的 CRC 不是 `0x3430be3b`，或 N15 终点下标不是 9437184 —— 那一条意图作废重量（照「读不到」那一条办）；同一段里不符的累计超过 3 次就停、退 3（校准那段代码坏了，不是机器忙）。

### 够判停机

问题单第 1、2 行都够判就停，没跑的量逐条标「够判后未跑」。第 3、4 行这一趟本来不判（第一节）：第三段与 Q9–Q13、第四段与 Q17 标「够判后未跑（沿用第 1 次重跑）」。第一段交回之后 S 里已经没有 f = 0.25 的格（两点都没造成或都打中 F5）时，第 1 行这一趟判不了，第二段跑不跑由主 agent 定（第五节「产物」末段）。

## 十二、修订

（留空。装置改完之后、第一份产物之前由执行员写：改完的装置 sha256、单测与变异条数，只许收严或补量；起法因脚本改了而换写法的，记一行。主 agent 定用改法乙的，修订写在这里。）

### 修订一（执行员，2026-10-23；装置改完、单测与变异都还没跑、一份产物都没跑）

依据：改装置时对着本登记逐条落地读出来的，没有单测读数（入库装置的编译与单测归主 agent 跑，派发提示「约束」第一条）。原判据、原作废与停机条款的原文都不动；下面每一条只收严或补。

| 修订 | 改了什么 | 收严或补在哪 | 依据 |
|---|---|---|---|
| R2-5 停机 10 收严到基线 | 段开头 21 遍基线（N2）里 CRC 那份的线程 CPU 钟读不到，打 `name=stopped reason=calibration_baseline_unreadable`；21 遍里有一遍 CRC 或追指针终点不是锚点，打 `name=stopped reason=calibration_anchor_mismatch_in_the_baseline`；都收尾、退 3，那一段一个点都不量 | 停机 10 原文只管一条意图前后的 N1、N15；基线坏了 R2-1 的分母就不可信，多一种停 | 第十一节停机 10「校准那段代码坏了，不是机器忙」 |
| R2-6 校准读不到照「读不到」作废 | 一条意图前或后那次校准，CRC 那份 3 遍里有一遍线程 CPU 钟读不到，那一条记 `valid=false invalid_reason=calibration_unreadable`、当场重量；两个校准比写 `none` | 第六节 N1 只写「读不到写 none，不当 0」；多一种作废 | 原登记第十一节「读不到」 |
| R2-7 停机 10 按遍累计 | 一次校准里 CRC 不是锚点的遍数，加追指针终点不符（算一遍），都进一段的累计；累计超过 3 遍就打 `name=stopped reason=calibration_anchor_mismatches_above_three_in_the_segment`、退 3 | 与停机 10 原文「某一遍」同口径：按遍、不按次，一次校准几遍都不符时停得更早 | 第十一节停机 10 |
| R2-8 停机 7 在装置里也核一遍 | `geometry-check`、`wall-clock`、`stage-split` 开头各打一行 `name=apparatus source_sha256=<编进二进制的源码 sha256> registered_sha256=<环境变量的值或 not_set>`；环境变量 `E185_REGISTERED_APPARATUS_SHA256` 设了而与编进来的不同（不分大小写），打 `name=stop_clause clause=7`、收尾、退 3。起跑脚本把这一节下面「装置 sha256」那一格设进这个变量 | 补：停机 7 原文由输入清单第一行核；这一条让每份产物自带编进来的源码哈希，输入清单、这一节与产物三处对拍 | 第十一节停机 7 |
| R2-9 补变异 V77–V94 | 第九节 V56–V76 之外补 18 条，每条点名一条会红的单测：V77 校准锚点只核前一次、V78 丙段读不到的那一轮当不值、V79 整机忙把 iowait 算成忙、V80 压力读成 avg60、V81 轨迹把读不到的比当 0、V82 签发行进甲乙段的批序号、V83 落点预测的环长、V84 `--overwrite-order` 认得别的值、V85 排队时间读错文件、V86 停机 7 判反、V87 两条路数的造成段数对不上也写相同、V88 Q3 每轮的对账拿错臂、V89 意图读数的增量减反、V90 根环条数本地常量、V91 停机 10 按次不按遍、V92 追指针乘数、V93 非自愿切换读成自愿切换、V94 可用内存读成内存总量 | 只补不删 | 第九节「执行员可以补，不许删」 |

装置 sha256（交回时）：`d6e1cb04218afe4db6459367719a3768bc79f5809b2ee4e735f57d986528a4b3`，13178 行，`#[test]` 118 条（改前 87 条，新加 31 条），`crates/mutations.tsv` 里 `E185 V` 起头的行追加之后是 94 行（V1–V55 原有，锚点在改后的源码里仍各逐字命中一次；V56–V94 追加）。单测与变异的跑数待主 agent 跑完写在这一格下面；编译或单测要改装置的，改完重写这个 sha256，以最后一份为准（停机 7 与 R2-8 比的都是它）。

装置 sha256（打进主工作区之后，以这一份为准）：`0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`，13187 行，`#[test]` 118 条。合入后验证的 clippy 报 `empty_line_after_doc_comments`（执行员把停机 7 那一段插在 `preflight` 的文档注释与函数之间），主 agent 把那两行文档注释挪回 `preflight` 头上，代码不变；挪之前那一份是 `19d449423c0a93de13f0f316e4c81d35c1f09b721941719e4c6341c2479c955a`。与交回时那一份只差别的会话的测试临时文件守卫：单测模块里三处临时路径改由 `singlefs_harness::temporary_files::TemporaryPathRemovedOnDrop` 认领、删掉三处手工删除，量的路径不变（主 agent 拿 HEAD 加执行员补丁与主工作区逐行 diff 核过，40 行改动全在 `#[cfg(test)]` 里或是基线 `eadb929d…` 已有的 `quarantine_mark` 两处）。交回那一份在临时副本里编过、单测「test result: ok. 118 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」（`~/.local/share/singlefs/work/m4-e185-r2-unit/run.log`）；这一份的单测与 39 行变异由主 agent 照验证段跑。

单测与变异（主 agent 照验证段跑，量的）：这一份装置在一次性编译目录里单测「test result: ok. 118 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.08s」，clippy（带 `-D warnings` 与 check.sh 七条纪律 lint）`clippy_rc=0`；变异轻型 `~/.local/share/singlefs/work/m4-b17-postmerge/mutation-quick-2.log` 点名 V56–V94 共 39 行，第 1233 行「计数：抓到 314 条、没红 0 条、无效 0 条、内存撞顶 0 条、超时 0 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条、锚点失效 0 行（共 314 条）」，39 行都在抓到里。

起法：没变（主 agent 起跑前照第五节「跑法」核过 `multi-host-run.sh`、`gpu-quiesce-run.sh`、`run-with-memory-cap.sh` 的用法；跑法脚本由主 agent 写，三趟起跑时在仓根 `e185-r2-tools/run-<段>.sh`（产物落树里的 `e185-r2-out/`），用户要仓根不放脚本与产物，三趟起完之后搬进 `scripts/experiment-apparatus/e185-r2-segment-run.sh`、产物直接写 `research/results/`，三趟跑的是搬前那几份，每段做的事相同）。改法乙：没改进装置（主 agent 没定用乙）。

## 十三、读过的文件与跑过的命令

### 读过的文件（行号区间；grep 命中行也算）

| 文件 | 读了哪些行 |
|---|---|
| `.claude/agent-common.md` | 1–109 |
| `.claude/agents/experiment-designer.md` | 派发时随定义给出（全文） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md`、`evidence-discipline.md`、`command-safety.md`、`writing-discipline.md`、`verify-before-claiming.md` | 全文（1–122、1–179、1–162、1–133、1–53） |
| `.claude/rules/three-way-inference.md`、`mutation-sampling.md`、`implementation-first.md`、`test-script-discipline.md` | 全文（1–166、1–93、1–19、1–106） |
| `research/prompts/m4-gpugc-measure-forks.md` | 1–12 |
| `research/prompts/e185-preregistration.md` | 1–721（全文，含第十二节 12.9 的第一段第一趟结局行） |
| `research/prompts/e185-r1-prereg.md` | 1–417（全文） |
| `/home/fy5090/.local/share/singlefs/work/m4-e185-r1-page/result.md` | 1–134（全文：第 1 次重跑的岔路表、判决行、G1–G6，读到的数列在第四节第 2、3、14 行） |
| `/home/fy5090/.local/share/singlefs/work/m4-e185-r1-investigation/result.md` | 1–293（全文；第四节第 4、5、9 行） |
| `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` | `grep -n '^#'` 的标题行（第 1 行标题带状态）；91–139（「做了什么」「复跑」「口径」）。「各量的判定」「比法」「岔路表」「它答不了的」各节没读 |
| `research/results/e185-r1-inputs-2026-10-22.txt` | 全文（打出时把用户名换掉） |
| `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out` | 第 237、515 行前 160 个字符；`name=preparation` 行的 `wall_seconds`、`name=intent` 行数、`valid=false` 行数、末行前 80 个字符（经 `grep`/`awk` 汇总，没读整行） |
| `research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out` | 同上汇总 |
| `research/results/e185*` | `ls -la` 与 `sha256sum`（产物 8 份） |
| `records/2026-10-19-里程碑四GPU垃圾回收开工.md` | `grep -n E185` 第 200 行之后的命中：202、205、206、209、211、212、217、218、225、228、281、284、286、291、297、299（每行前 600 字节） |
| `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` | 1–80；225–300；2195–2320；2585–2600；2727–2830；3600–3630；3895–3970；4160–4180；4240–4262；6440–6720；`grep -n` 命中（24、132、242、287、359、488、584、1560、1757、2040、2211、2263、2299、2592、2659、2727、2741、2758、2774、2783、2788、2790、2802、3021、3611、3623、3729、3730、3783、3898、3909、3931、3963、3970、4168、4243、5025、5329、5560、5644、5651、5795、5941、6003、6170、6380、6488、6534、6578、6595、6643、6653、6659、6748、6758、6840、6847、6849、7122、7133、7152、7312、7315、7448、7472、7486、7524、7749、7757、8154、8354、8964、9599、10198）；`git diff HEAD` 全文（3 行加） |
| `crates/singlefs-core/src/allocator.rs` | 2725–2760 里 `grep` 的命中；954、964、2389、2725、2744、2824 |
| `crates/singlefs-core/src/transaction.rs` | 2059、9661（`grep`） |
| `crates/singlefs-core/src/mounted_session.rs` | 118、124、1190、1191、1236、1237、1956、1974、2588、2615、2807、2835（`grep`） |
| `crates/singlefs-core/src/region_scan.rs` | 179、181、204、207、249、270、286、309、454、482、496、820、847（`grep`） |
| `crates/singlefs-core/src/compaction_plan.rs`；`checksum.rs` | 88；43（`grep`） |
| `crates/singlefs-format/src/lib.rs` | 291、309（`grep`） |
| `scripts/hosts/multi-host-run.sh` | 1–80；24–31；`grep` 命中 2–755 间与 `--peer`、`SERIAL` 有关的行 |
| `scripts/hosts/peer-host-lib.sh` | 2、7、23、30（`grep`）；120–160 |
| `scripts/crash/gpu-quiesce-run.sh` | 1–60 |
| `multi-host.env.example` | 20–27、57（`grep QUIESCE`；不是私有的那一份，私有的 `multi-host.env` 没读） |
| `.claude/hooks/lib_long_run.py` | 10、18、92（`grep CANONICAL_HOSTS_SENTENCE`） |
| `scripts/kb/replace-once.py` | `--help` 前 15 行 |
| `.claude/heavy-thread-default` | 全文 |
| `.claude/kb/milestone/04-fourth-txn.md`；`.claude/kb/decisions/26-后台整理与放置回收.md`；`.claude/kb/decisions/24-后台重活能不能卸给GPU.md` | 142；232–252；47–59（经 `quote-kb.py`） |

### 跑过的命令（原样；只读命令与写这份登记的命令，没编译、没跑装置、没跑门禁阶段）

```bash
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m4-e185-r2-designer --purpose 'E185 第 2 次重跑登记的起草'
sha256sum research/results/e185* research/prompts/e185* research/prompts/m4-gpugc-measure-forks.md crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs
git show HEAD:crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs | sha256sum      # b3a50d50…
git diff HEAD -- crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs              # 3 行加
sha256sum crates/singlefs-core/src/{transaction,mounted_session,region_scan,compaction_plan,unit,checksum,block_device,allocator}.rs Cargo.lock
git diff --stat HEAD -- crates/singlefs-core/src/allocator.rs crates/singlefs-format/src/lib.rs           # 354 行加…；35 行
grep -rn 'thread::spawn\|thread::scope\|rayon' crates/singlefs-core/src | wc -l                             # 0
grep -c 'E185 V' crates/mutations.tsv                                                                       # 55
grep -n '#\[test\]' crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs | wc -l         # 87
awk 'NR==237||NR==515{print NR": "substr($0,1,160)}' research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out
for f in <第 1 次重跑两段产物>; do grep -o 'name=preparation .*' "$f" | grep -o ' wall_seconds=[0-9.]*' | awk -F= '{s+=$2; n++} END{print "preparation lines",n,"sum wall_seconds",s}'; grep -c 'name=intent ' "$f"; done
getconf CLK_TCK                                                                                              # 100
D=/home/fy5090/.local/share/singlefs/work/m4-e185-r2-designer; nice -n 19 python3 scripts/kb/quote-kb.py $D/clauses.md '.claude/kb/milestone/04-fourth-txn.md~^\| 七·1 \|' '.claude/kb/decisions/26-后台整理与放置回收.md@#### 已定项 13：后台任务的计算放在哪、怎么开、什么节奏' '.claude/kb/decisions/24-后台重活能不能卸给GPU.md@#### 已定项 3：判据取什么形式'
diff <(sed -n 7,49p $D/clauses.md) <(sed -n 69,111p research/prompts/e185-r1-prereg.md) && echo D26-D24-same-as-r1
diff <(sed -n 3,5p $D/clauses.md) <(sed -n 59,61p research/prompts/e185-r1-prereg.md) && echo qi1-same
```

`quote-kb.py` 输出 `✓ 3 段整抄进 /home/fy5090/.local/share/singlefs/work/m4-e185-r2-designer/clauses.md，回读逐字节一致`；两条 `diff` 输出 `D26-D24-same-as-r1`、`qi1-same`。准备挂钟那条输出 `preparation lines 4 sum wall_seconds 4189.07`、`141`（第一段），`preparation lines 2 sum wall_seconds 4188.11`、`60`（第二段）。

校准代用量（第四节第 11 行；本机，那一刻 `/proc/loadavg` 是 `18.47 29.47 33.50`，`/proc/pressure/cpu` 首行 `some avg10=2.86 avg60=3.76 avg300=9.41`）：

```bash
nice -n 19 python3 -I -c '
import zlib, time, statistics, os
for size in (4<<20, 16<<20):
    buf = bytes((i*2654435761>>13)&0xff for i in range(size)) if size==(4<<20) else os.urandom(size)
    zlib.crc32(buf)
    ts=[]
    for _ in range(200):
        t0=time.thread_time_ns(); zlib.crc32(buf); ts.append(time.thread_time_ns()-t0)
    med=statistics.median(ts); ts.sort()
    print("size",size,"median_ns",med,"p01",ts[2]/med,"p50",ts[100]/med,"p95",ts[189]/med,"p99",ts[197]/med,"max",ts[-1]/med)
'
```

```text
size 4194304 median_ns 448046.0 p01 0.9865304008963366 p50 1.0003013083478036 p95 1.0176432776991648 p99 1.0348290130924056 max 1.0635760613865541
size 16777216 median_ns 1772754.0 p01 0.9964298486986914 p50 1.000011281881186 p95 1.007328145924364 p99 1.0108475287603356 max 1.112847580656989
```

第七节新锚点的算术（命令与输出原样）：

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
print("check", hex(crc(b"123456789")))
n=16*1024*1024
buf=bytes(i%251 for i in range(n))
print("calibration_16MiB_i_mod_251", hex(crc(buf)))
print("calibration_32MiB_i_mod_251", hex(crc(bytes(i%251 for i in range(2*n)))))
'
nice -n 19 python3 -I -c '
a=2654435761; m=1<<24
x=0
for _ in range(1<<20): x=(a*x+1)%m
print("chase_index_after_2^20_steps_from_0", x)
print("a_minus_1_mod_4", (a-1)%4, "c_odd", 1%2)
print("entries", m, "bytes", m*4)
'
```

```text
check 0xe3069283
calibration_16MiB_i_mod_251 0x3430be3b
calibration_32MiB_i_mod_251 0x84b65aa5
chase_index_after_2^20_steps_from_0 9437184
a_minus_1_mod_4 0 c_odd 1
entries 16777216 bytes 67108864
```

（`a − 1` 被 4 整除、增量是奇数，模 2^24 的这条递推走满全部 2^24 项才回到起点：追指针那张表是一个环，走 2^20 步不会提前绕回。）

其余只读命令（`sed -n`、`awk 'NR==…'`、`grep -n`、`wc -l`、`ls`）读的行号都列在上一张表里。

## 十四、并行与挂钟

原登记第五节「并行与挂钟」（12.10、12.11）管段内怎么并行，这一趟照旧；这里写三趟之间怎么排与新读数的代价。

- 独立单位：单点核、第一段、第二段三趟各自一条命令、各自拷树与编译，第二段不读第一段的产物（S 由主 agent 读两份产物时数）；单点核两条 `geometry-check` 各自两个点，点与点之间互相独立，走原有的全局准备队列；段内不计时的准备照原登记 12.11 的全局队列（各点造池、第 2–5 轮重造、真盘镜像互相独立）。
- 线程：`E185_THREADS` 不设，准备队列与第二段饱和吞吐的 N 取登记值 `.claude/heavy-thread-default`（今天 31）；计时窗口一条线程；N1、N2、N15 的校准与 N3–N10 的读数在测量线程上做，不另起线程；编译照 cargo 默认、外面经 `multi-host-run.sh` 的内存包装。
- 串行：三趟在第二台上先后跑、不同时跑，因为两段都有计时窗口，原登记第十一节「机器被别的活挤」要求计时时机器上没有别的测量，真盘臂又要第二台那一块盘；段内每个计时窗口串行（量的是一条线程的 CPU 与挂钟）；校准与等常态（R2-2）在测量线程上、计时窗口之外串行做，它们量的也是这一条线程此刻跑同一份活要多久，换到别的线程上就不是这条线程的状态了。
- 预计挂钟：单点核约 0.2 小时（推的：编一次约 10 分钟，同原登记第三段的估计；两条 `geometry-check` 各两个点并行，P = 1024 一次造池约 85 秒，第四节第 4 行）；第一段约 1.8–2.0 小时（推的：编约 0.17 小时 + 准备约 1.16 小时（第 1 次重跑量到 4189.07 秒，第四节第 7 行）+ 测量约 0.5 小时（原登记 12.11 推的）+ 新读数约 0.02 小时（N10 每次隔 1 秒共 6 次；N1 与 N15 每条意图前后各约 0.1–0.2 秒，按 16 MiB 软件 CRC-32C 约 1–3 GB/s 与 2^20 次各约 100 ns 的取内存推的，约 150 条意图）+ 等常态至多 600 秒）；第二段约 1.5–1.7 小时（推的：编约 0.17 + 准备约 1.16（4188.11 秒）+ 阶段重放与饱和吞吐约 0.2（原登记）+ 新读数与等常态同上）；三趟合计约 3.5–3.9 小时，每趟各自在 2 小时内，第一段贴着 2 小时的边。第一段要是超了，主 agent 改用 `detached-run.sh start --wall-clock over-2h` 时脚本文件头的三样：分片按段（三趟本身就是三片）；分不了：续跑与多机——一段里的轮次要在同一回造好的池与同一次清场之下连着量，中途换进程续不了，只能整段重跑；真盘臂要第二台那一块盘、计时要机器上没有别的测量，不能分到几台。
- 预计在用核数：编译那几分钟约 31 核；准备阶段约 5–15 核（第 1 次重跑第一段准备时第二台最近 605 秒平均 6.1 核，调度记录第 212 行，是 `E185_MEMORY_CAP=40G` 下内存预算只装得下 5 份左右）；计时窗口、校准与等常态 1 核；第二段饱和吞吐 16 核与 31 核两档各约 2 秒 × 每点；单点核两条各约 2 核（两个点）。
