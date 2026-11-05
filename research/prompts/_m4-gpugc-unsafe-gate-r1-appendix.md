**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:1-8`（整段抄，未转述）**

```markdown
## E197 扫环胶水内存随盘数 —— 装置已写、产物未跑

**中文** · [English](197-ring-scan-glue-memory-by-device-count.en.md) · [日本語](197-リングスキャン接着部のメモリとディスク数.ja.md)

跑前登记 `research/prompts/e197-preregistration.md`（问法由主 agent 在装置写之前删定三处，见登记第 5 行）。回答岔路单 `research/prompts/m3-close-c653-forks.md` 的甲（上界）、乙（改法）两行；被测条款是 C653（扫环胶水内存随盘数涨）（见 `.claude/kb/checks-owed.md` 对应行）：扫环那一截进 Verus 之后，胶水 `scan_journal` 先把每块盘上每一份自证过的记录收进一个 Vec 再建表，峰值随盘数涨。

装置 `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs`（入库装置：直接调实现里的 `scan_journal` 与不改的 `first_copy_of_each_record_key`，另在装置里抄三份胶水当乙二臂与两份阳性对照，不另写模型），35 条单测（`grep -c '#\[test\]'` 数的，含一条子进程入口用例，不在子进程里时什么都不量）；驱动与出表脚本在 `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/`（`drive.py`、`report.py`）；变异行在 `crates/mutations.tsv` 里（登记修订三去掉计数分配器那一行、补两行、换一行的改法）。**这一页只记装置已写，产物一次没跑。**

```

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:9-16`（整段抄，未转述）**

```markdown
### 现状

- 装置已在主工作区，编译与单测是 checker 档（重型），归主 agent。装置要直接调扫环核，而 `singlefs-core` 没有转出它，所以 `crates/singlefs-checker-tier/Cargo.toml` 的 `[dependencies]` 里有 `singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }`。
- `JournalRecord` 的 `size_of`：登记第七节锚 6 手算 376，按结构体字段逐个数是 320（四条节点指针，不是五条）。装置单测按 320 钉，常驻页式子（登记修订三第四节）也按 320 代入；跑出来的读数为准，对不上照实写进登记修订，不改登记原文。
- 登记第七节锚 7 原来写 split_pc 的推算「同基线」，装置按分配器口径多算两个 Vec 的容量（4 盘差约 2.4%，3 盘上 K = 64 差约 −3%、K = 1 差约 −16%）。登记修订三改成常驻页口径之后，记录 Vec 与键表只算碰过的页，split_pc 与 baseline 同式，那两个 Vec 的容量差不再出现（推的，装置单测钉了两式相等）；split 在 1 盘上同 baseline。
- 驱动的自证曾跑绿（10 项）。装置与出表脚本照登记修订三改完之后（去掉计数分配器、进程内改量常驻页峰值 H），装置单测与 `report.py` 的自证、弄坏开关都还没跑，交主 agent。
- 产物一次没跑；岔路单甲、乙两行都还开着，没有任何一个判定行。路径登记里的行号取自主工作区里的源码。

```

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:17-32`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 仓根。入库装置的编译与单测归主 agent（checker 档，重型）；二进制编在草稿目录的 compile-then-swap/target/release/ 下。
task-server/task submit -- python3 scripts/hosts/compile-then-swap.py <草稿副本> crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs --scratch <草稿目录> --release
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e197_scan_journal_glue_memory_by_device_count 的单测; 范围=1 个测试目标; 为什么=核 E197 装置的 35 条单测' task-server/task submit -- cargo test --release -p singlefs-checker-tier --bin e197_scan_journal_glue_memory_by_device_count
# 产物（单测绿之后）：先停机条款与差分集（checks），再逐键逐值比对（compare），再三段（k64-full、k64-half、k1-full）；每格一次服务尝试（分片的一片），内存上限与哪台机器由服务定
BINARY=<草稿目录>/compile-then-swap/target/release/e197_scan_journal_glue_memory_by_device_count
task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py checks --binary $BINARY --out research/results/e197-<日期>-checks.out
task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py compare --binary $BINARY --out research/results/e197-<日期>-compare.out
# 逐行打出要交的 task-server/task submit 整条（不交、不跑）：第 0 批是 checks 与 compare，第 1–10 批各 6 件 segment（k64-full、k64-half、k1-full 三段，每段重复号 1–5，每个重复号四组）；逐条原样交
python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py list-submissions --binary $BINARY --date <日期>
# 每件有结局之后，主 agent 直接存服务状态：task-server/task status <父任务号> --json > research/results/e197-<日期>-status-<父任务号>.json；全部存好再出表（判决文件排他新建，已在就拒）
task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py --artifacts research/results/e197-<日期>-checks.out research/results/e197-<日期>-compare.out research/results/e197-<日期>-k*-r*-*.out --status research/results/e197-<日期>-status-*.json --out research/results/e197-<日期>-verdict.out
```

````

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:33-39`（整段抄，未转述）**

```markdown
### 口径

- 判的数是统一任务服务记下的每次尝试的内存峰值（服务取按轮采到的与内存包装退出时读自己 cgroup 的 `memory.peak` 里大的，含页缓存与外壳自己，偏大不偏小；驱动不套包装，主 agent 在每件有结局之后读 `task-server/task status <父任务号> --json` 存档，report.py 读存档），一格 5 次取中位数，减同几何空跑格的中位数；进程内的常驻页峰值 H（量窗开头往 `/proc/self/clear_refs` 写 5、读 `VmRSS` 当起点，交回之后读 `VmHWM`，两者之差）只用来互核，不判。
- 几何：默认环每盘 196 608 个记录槽，盘 4 GiB，盘数 1–4（3 盘是反向取样点），每条记录点名项 64 与 1，记录槽填满与半满；一格的键是（臂, 盘数, 填满档, 点名项数, 重复号），峰值取服务记下的那一次尝试的 `memory_peak`（结果行带 `task=` `attempt=`，report.py 按状态存档对上）。
- H 的量法：量窗开头（建池、编探针记录之后，调臂之前）往 `/proc/self/clear_refs` 写 5 把这个进程的 `VmHWM` 重置成此刻的常驻量，读 `VmRSS` 当起点；交回之后、放掉交回值之前（经读缓存的两臂在放掉缓存之前）读 `VmHWM`；H = (`VmHWM` − 起点) × 1024 字节。写不进或读不出的那一次作废（V11），不当 0。轨迹与期末值读 `VmRSS` 减起点。装置里没有计数分配器，也没有 `unsafe`；H 是整个进程的数，单测里凡要读 H 的，每一格各起一个子进程量。
- 量峰值的格里各臂交回的表不再跑一遍 `scan_journal`（那会把峰值翻倍），只打 64 位摘要（按键序折进每条记录 4096 字节编码的 CRC-32C），出表时与基线臂同格的摘要相比；逐键逐值相同另由不量峰值的专用格判：`--compare` 的 36 格（三份胶水臂 × 盘数 1–4 × 三个几何，每格先跑臂、再跑 `scan_journal`，用 `==` 比）与差分集的四例（小环上同样用 `==` 比）。

```

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:40-46`（整段抄，未转述）**

```markdown
### 岔路表

| 岔路 | 状态 | 还差什么 / 剩下的量能不能让它翻面 |
|---|---|---|
| 甲 上界 | 未判 | 产物一次没跑；登记里的量（K = 64 满槽与半满、K = 1 两个反向取样点、1–4 盘）跑完才有判定行 |
| 乙 改法 | 未判 | 产物一次没跑；乙二臂能否做到由差分集 D 与四档镜像格的摘要判，乙一的证明义务改动量不在这份装置里 |

```

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:47-53`（整段抄，未转述）**

```markdown
### 它答不了的

- 乙二的两遍读写法（先只收键调核、再读一遍只解首份）不在岔路单乙二的定义里，登记列为待认项，不量。
- 可写挂载扫环时 `ReadStageCache` 整份留着读到的记录字节，这一截不在 `scan_journal` 里，岔路单没有对应的行，不量。
- 乙一（核改成逐盘交）要一份改过的 Verus 核与证明，不是内存量，这份装置不量；乙二判做不到时由主 agent 另派。
- 盘数只量 1–4，登记的盘数上限大于 4 时甲二的字节数是外推；`crates/` 里没有池的盘数上限，取几由主 agent 定。

```

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:54-65`（整段抄，未转述）**

```markdown
### 路径与结论登记

结论「`scan_journal` 的峰值随盘数怎么涨」：同一个量三条路各算一遍再比（进程内常驻峰值 H、cgroup、按常驻页式子推）。

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 进程内常驻峰值 H | 量窗开头往 `/proc/self/clear_refs` 写 5 重置 `VmHWM`、读 `VmRSS` 当起点，交回之后（经读缓存的两臂在放掉缓存之前）读 `VmHWM`，H = 两者之差 × 1024；每格 5 次，几何取默认环的三档，不扫别的盘数 | `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:156 read_from` 与 `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:131 open_with` | 记录尺寸 4096、环槽数 196 608、`size_of`（单测钉）；与路径 2 共用「碰过、还在的页」这个口径（H 数进程常驻页的高水位，路径 2 数 scope 记账页的高水位），一处让页没碰到或没算进来的毛病两条会一起错，只剩路径 3 的式子看得出 |
| 2 | cgroup 峰值 | 服务每次尝试记下的 `memory_peak`（按轮采到的与内存包装退出时读 cgroup 的 `memory.peak` 取大），主 agent 存 `task-server/task status <父任务号> --json`，驱动与 report.py 读存档，减空跑格 | `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py:265 pinned_reading` | 与路径 1 不共用代码；共用的只有「每格一个进程、同一个二进制」 |
| 3 | 按常驻页式子推 | 登记修订三第四节 `resident_prediction.py` 的常驻页式子 R，按量到的 `size_of` 代入；出表脚本与装置里各写一份，两份都钉在登记修订三的设计数上 | `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:344 predicted_resident_peak_bytes` 与 `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py:129 predicted_resident_peak` | 记录尺寸、点名项大小、glibc 的块头与 16 字节对齐、B 树每叶留 6 个键（标准库升序插入的分裂点，推的）；这一条错了，路径 1 对推算的 ±5% 会一起错，所以另有路径 2 互核（出表脚本的 `S3_two_measures` 行）；路径 1、2 都只认碰过的页，一起错时只有这一条式子看得出 |

跨路径的比对断言：路径 1 对路径 3 钉在装置单测 `the_resident_peak_of_every_scanning_arm_stays_within_the_prediction`；路径 1 对路径 2 在出表脚本的 `S3_two_measures` 行（判的是 H 对 P），跑出产物之前还没有一次真实读数，这一条欠着。

```

**出处 `.claude/kb/experiments/197-扫环胶水内存随盘数.md:66-73`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 备料 | 2026-11-04 不受影响：登记修订三把进程内量法从计数分配器改成常驻页峰值 H，装置、出表脚本与路径登记的源码落点随改，判的 P、结论与产物都不变（产物没有一格进过判决）；此前 2026-11-03 不受影响：只改复跑写法，登记表的源码落点照现在的源码改，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-22 不受影响：该分项的「**欠**」段挂着 C653（扫环胶水内存随盘数涨），装置已写、产物未跑，依据段没有引 E197（扫环胶水内存随盘数），所以这一格只写备料；产物跑出来之后「该不该升成支撑」交主 agent 判 |
| D23（journal 的角色与格式） 已定项 19 | 备料 | 2026-11-04 不受影响：登记修订三只换进程内的量法，默认环 768 MiB 与「环 ≤ 设备容量 ÷ 4」仍只当几何来源用；此前 2026-11-03 不受影响：只改复跑写法，登记表的源码落点照现在的源码改，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-22 不受影响：默认环 768 MiB 与「环 ≤ 设备容量 ÷ 4」只当几何来源用，装置里的本地常量抄自它并与 `crates/` 回比 |
| D23（journal 的角色与格式） 已定项 12 | 备料 | 2026-11-04 不受影响：登记修订三只换进程内的量法，记录尺寸 4 KiB 与一条记录装 64 个点名项仍只当几何来源用；此前 2026-11-03 不受影响：只改复跑写法，登记表的源码落点照现在的源码改，结论、数与产物都不变；此前 2026-11-01 不受影响：只改复跑起法；此前 2026-10-22 不受影响：记录尺寸 4 KiB 与一条记录装 64 个点名项只当几何来源用，同样本地常量加回比 |

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:1-25`（整段抄，未转述）**

```markdown
## E183 容量斜率读缓存小盘准入与清扫阈值的代价数 —— 部分已跑（第一段 H4 够判；第二段 H2 已跑，R2e 发布两行不出类、登记字面不够判；第三段 H5 已跑，R5c 不出、登记字面不够判；第四段 H3 未跑）

**中文** · [English](183-cost-numbers-for-capacity-slope-read-cache-small-disk-admission-and-sweep-thresholds.en.md) · [日本語](183-容量勾配・読み取りキャッシュ・小容量ディスクの受け入れ・掃除しきい値のコスト.ja.md)

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 R2a R2b R2c R2d R2e R2f R3a R3b R3c R4a R4b R4c R4d R5a R5b R5c R5d X14 X15 K10 K30 K31 K32 K33 K34 A30 A31 A33 A34 A35 A36 A37 A38 A41 V30 V31 V32 V33 V34 V35 V36 V37 T2 T3 T4 T5 G1 G2 G3 G4 G5 G6 G7 G8 M60 M64 M65 M70 M71 M72 M73 M74 M75 M76 M77 M78 M79 PC1 S4 S8 S16 L1 X17 X19 X21 X3 X5 X22 X25 X27 R2g R2h V38 V40 G9 G10 G11 G12 T7 T8 T9 A43 Q8 Q13 Q14 -->

跑前登记 `research/prompts/e183-preregistration.md`（四段：第一段 H4，第二段 H2，第三段 H5，第四段 H3；第十二节的修订：第一段续做的三条与第二、三段装置写之后的七条都在对应产物之前）。回答的是问题单 `research/prompts/m3-close-cost-measurement-questions.md` 的 H2 至 H5 四行，第一段只答 H4：最小能 mkfs 并可写挂载的盘上，准入式子除容量、已分配、不可回收之外的六项各多大、谁占大头。被测的是 `crates/` 今天的 mkfs、可写挂载与会话发布，式子出处 D28（挂载期承诺量） 已定项 1、已定项 3。

装置是入库装置 `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs`（只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的文件；盘是内存稀疏盘，S 取 4、8、16，日志环 L 取 1 MiB，另有 S = 8、L = 4 MiB 一个取样点，默认盘是 4 GiB × 2、L = 768 MiB）。独立的一半是纯 Python 的锚点脚本（不读 `crates/`）。

**现状**：

- 第一段跑完：H4 的十项在一个进程里按序跑完（`all`），产物 459 行，A31 独立核对 56 份读数 0 处不符；复跑登记的产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`，核对输出 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`（第 1 次重跑，清扫阈值的本地常量改 365 之后重出，与 2026-10-23 那份逐行 diff 只差第 6 行，判决行逐字相同，见「第 1 次重跑」一节；2026-10-23 的两份旧产物留着）。
- 复跑登记的第一段产物是上游写序变化之后重出的一份（只修钉值，结论不变）：取号写两轮与欠账 C755（恢复与两种挂载触发退回的失败不同）那一处每次可写挂载问一次 F2，让每块盘的计数（`name=h4_mount_tally`，每项每种状态每块盘一行）多一次系统配置写、多四次系统配置读。重出的产物与 2026-10-18 那份 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18.out`（两遍逐字节相同的原产物，留着）行数、字节数相同（459 行、152434 字节；重出的也跑了两遍，`cmp` 逐字节相同），只差 28 行 `name=h4_mount_tally`（第 106、107、117、118、183、184、194、195、275、276、286、287、355、356、366、367、381、382、392、393、407、408、418、419、433、434、444、445 行），每行只差 `system_configuration=` 一个字段：`r26/106496,w3/12288` 变成 `r30/122880,w4/16384`；这一页引的判决行与读数行都不在这 28 行里，两份逐字相同。装置单测里钉的第一段 SHA-256 当时换成这一份的 `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`，2026-10-30 第 1 次重跑之后重钉成 `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870`（换进主工作区的状态见「第 1 次重跑」一节）。核对输出与 2026-10-18 那份 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18-anchor-check.out` 只差第 1 行的 `product_sha256`。
- 三个最小盘（R4a，单位 MiB）：S = 4 为 mkfs 18、空池挂载 18、有文件挂载 28；S = 8 为 18、18、34；S = 16 为 18、18、49；S = 8、L = 4 MiB 为 21、21、37。有文件挂载的最小盘是 mkfs 最小盘的 1.555 / 1.888 / 2.722 / 1.761 倍。三个最小盘的 C + 1 … C + 16 MiB 单调核都过。
- 阳性对照 PC-准入（登记原判据，只在空池 C_挂空(S) − 1 MiB 上判）四个小盘搜索项 0 个成立：C_挂空 − 1 MiB 上 mkfs 就败在 `UnitAreaBeyondDevice`，空池挂载没走到，报不出 `SpaceAdmissionRefusedBeforeAcquisition`。按 V33 原文记一次输：R4a–R4c 的空池一半不出，空池状态下最小盘归 mkfs 的几何，不归准入式子。登记修订补的阳性对照 PC-准入-文件（有文件状态）4 个小盘搜索项 4 个成立：C_挂文件 − 1 MiB 上可写挂载做成、准入结局 `StillShortAfterTheFloorRaises`。
- 有文件状态下，最小盘上准入预留占容量 906256 / 961401 / 969728 ppm（S = 4 / 8 / 16，R4b，只报数）；六项里环窗口主导，份额 692 / 812 / 888 千分之一（R4c），四个小盘点（含 L = 4 MiB 一点）主导项相同。
- 默认盘（R4d）有文件状态准入预留占容量 56305 / 77478 / 119824 ppm（S = 4 / 8 / 16）；主导项 S = 4 为挂载期承诺量（546 千分之一），S = 8、16 为环窗口（569、721 千分之一），跨 S 不稳定。空池可写挂载入口 99 ppm。
- K32 空池一半在 `before_a_publish` 入口不成立（7 个项各一份读数，环窗口被扣、条款句说空池为 0），按 G8 记「条款这一格可能错」；`writable_mount` 入口空池环窗口为 0，成立。
- 装置现有单测 87 个、E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 的变异行 64 条（2026-11-03 现数：`grep -c '#\[test\]'` 对装置源文件、`grep -c '^E183 ' crates/mutations.tsv`；第二段的单测与变异加在登记 `research/prompts/e183-r2-prereg.md` 第十二节，这一页没有它们的跑数）。下面是 2026-10-30 那一版的读数：装置单测 73 个，release 下 73 passed、0 failed 是主 agent 跑的读数；变异行 54 条（`crates/mutations.tsv` 里名字以 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 加一个空格起头的行，`grep -c` 数的），三个数待提交时门禁 checker-tier-crates-mutation-replay 跑出，这一段没跑变异整表。第一段写下时是单测 24 个、变异 13 条（M60、M64、M65 是登记第九节第一段的，M70 至 M79 是装置另补的），后面各段加的单测与变异在登记第十二节。
- 第三段（H5）跑完：产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out`（19067 行，26 片，本机 13 片、第二台 13 片，同一份程序文件），独立核对输出 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out`（24 格加对照 0 处不符）；读数在「第三段读数」各节。
- H5 的读数：R5a 已出；R5b 甲：θ = 64、365 不越过一片的上界，θ = 369、738 越过，六个几何点一致；PC-差分 原臂 24 格全打中，V35 成立，R5c、R5d 不出，新臂 PC-差分-读链 24 格全过；R5c 没有真比值，判别力自证做不出（V36）；G5 既没触发也没清除。登记第十一节的条件（R5c 要出）下 H5 不够判，岔路表在「岔路表（问题单 H5…）」一节。
- 第二段（H2）跑完（第 2 次重跑登记 `research/prompts/e183-r2-prereg.md`，经统一任务服务分 5 片、一片一轮，两台都派到）：产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out`（949 行，9 格 × 5 轮 = 45 个进程全部「阈值内跑完」）。R2a 一次分配的耗时对容量次线性（b 0.251 至 0.322），R2b 一次发布的耗时对容量线性（b 0.811 至 0.875），R2d 分配的峰值多占与容量基本无关（每次 112 字节）、发布的峰值多占线性（b 1.022；16 TiB − 1 MiB 上一次发布多占 15568569548 字节，约两份分配器拷贝），R2c 分配器拷贝线性（每盘每槽约 3.625 字节），R2e 分配耗时对记录数次线性、发布两行不出类（登记的 X5 造不出）。本机能造的最大稀疏盘（阈值 24 GiB 内整格跑完的最大一档）是阶梯端点 16 TiB − 1 MiB（进程高水位最大 23629737984 字节），阈值减半时是 4 TiB。G2 触发（Q8 推的 R2a 基本无关、R2e 分配线性都被推翻），G11 记一次输（两台各不足 5 轮、多数格负载第一列 ≥ 2.0）。登记第十一节的条件下 H2 字面不够判（差 R2e 发布两行，登记内补不上）；读数与岔路表在「第 2 次重跑」一节。
- 第四段（H3）装置没写；H3 一行开着，H2、H5 两行登记字面不够判。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:26-36`（整段抄，未转述）**

```markdown
### 做了什么

1. 起跑前提三样核过（登记第十一节停机条款「起跑前提不成立」那一条；登记第十二节「修订」第二条）；T2、T3、PC-堆 在跑产物之前用 `--item constants`、`--item anchors`、`--item positive-control-heap` 单独跑过，输出在执行员草稿目录，不是产物：`constants_compared=21 constants_mismatched=0`、`anchor_mismatches=0`、`positive_control_heap_holds=true`。
2. 官方二进制由 `scripts/hosts/compile-then-swap.py --release` 从装置终稿（sha256 `766d029c01fe54e885ef186b8caee481c99b5d8bde0eaa990449abdafa60af45`）编出，二进制 sha256 `97e5e01ea487719935bc44f368874db4c5a82a851f941269a72c9c50c8665eec`；`singlefs-core`、`singlefs-format`、`singlefs-harness` 的源文件在二进制之后没动过（`find crates -name '*.rs' -newer <二进制>` 只列出另一个实验的装置与 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 装置自己）。被测树 `git diff HEAD -- crates/ | sha256sum` 为 `7fd21f75d5e478c1342b53d0d54b123303ac7993b0e4af473d049dceea10d5ca`（`--stat` 末行「190 files changed, 73166 insertions(+), 4603 deletions(-)」），与登记第三节记的 `847e0e54…` 不同，按「起跑前提不成立」那条停机条款不停。
3. 产物跑两遍（内存上限 8G、线程上限 31，装置单线程），两遍退出码都是 0，`cmp` 逐字节相同（T4 的确定性核）；产物 sha256 `190704189a413410206bac906fd499b729abe0c376ad5a68a7b53ddc22aa1f24`。
4. A31 独立核对：`scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py` 用同目录 `anchors_e183.py`（登记第十三节命令一的锚点脚本，sha256 `d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d`）对产物每份读数现算单元区容量与环窗口字节再比。
5. 第三段（H5）：长跑脚本 `scripts/experiment-apparatus/e183-sweep-history-shards.sh` 起 26 片；第一趟两台各编装置、程序文件 sha256 不同，装置按登记在合并时停机，第二台 13 片作废移开；第二趟第二台用本机那一份程序文件现跑，合并出产物（细节在「第三段（H5）：跑法、产物与被测树」）。
6. H5 的独立核对与读表小工具：`sweep_history_anchor_check.py` 用同一份锚点脚本对产物 24 格逐格核（0 处不符），`sweep_cell_table.py` 从产物读 24 格表与每次挂载写字节的除法；都在 `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/`。
7. 2026-10-23 只修钉值：取号写两轮与欠账 C755（恢复与两种挂载触发退回的失败不同）落地之后，装置单测「第一段 `all` 输出与钉死的入库产物逐字节相同」红在 SHA-256 上（现跑 `b4ff64f1…`，钉的 `7ed2e16c…`）。在仓副本（主工作区当天的 `crates/`，不带 `target/`）里把钉值换成现跑的 SHA-256、来源注释补一句，release 编出装置，`all` 跑两遍（内存上限 8G、线程上限 31），两遍退 0、`cmp` 逐字节相同，sha256 `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`，与调查员在另一份副本里现跑的值相同；A31 核对脚本对新产物重跑，退 0、56 份读数 0 处不符。复跑登记两行与 `driver_e183_anchor_check` 读的产物改指新文件，旧产物与旧核对输出留着。
8. 2026-10-30 第 1 次重跑：装置里清扫阈值的本地常量跟上 C690（清扫触发阈值的默认值没定） 的默认值 365，第一段 `all` 重出产物，逐行 diff 只变第 6 行，判决行不变，A31 对得上；复跑登记改指新产物，经过与数在「第 1 次重跑」一节。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:37-80`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 仓根；入库装置，经 scripts/experiment-apparatus/replay.sh 的两行登记（driver_e183 编装置并跑 all，driver_e183_anchor_check 跑 A31 核对）；
# 重型（checker 档包要编），由主 agent 在提交时带 SINGLEFS_HEAVY_TESTS=commit 跑
bash scripts/experiment-apparatus/replay.sh E183

# 只看装置本身（同一份源码，不经复跑登记）
cargo run -q --release -p singlefs-checker-tier --bin e183_capacity_read_cache_reserve_and_sweep_costs -- all

# A31 独立核对，纯 Python
python3 -B -I scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out
```

登记行在 `scripts/experiment-apparatus/replay.sh` 里 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 名下两行：驱动 `@driver_e183`、产物 `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`、判法 exact；驱动 `@driver_e183_anchor_check`、产物 `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`、判法 exact。装置入口：`--list-items`、`--item <项>`、`all`；项名十个：`constants`、`positive-control-heap`、`anchors`、`h4-small-disk-slots-per-region-4`、`h4-small-disk-slots-per-region-8`、`h4-small-disk-slots-per-region-16`、`h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes`、`h4-default-disk-slots-per-region-4`、`h4-default-disk-slots-per-region-8`、`h4-default-disk-slots-per-region-16`。每项耗时与退出码打到标准错误，不进产物。

产物里每一类结果行的条数（`awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out | sort | uniq -c` 的原样输出；2026-10-23 那份同一条命令的输出逐字相同，两份输出的 `md5sum` 相同）：

```text
      5 name=anchor_chain_rewrite
      1 name=anchor_default_ring_unit_area_start
     15 name=anchor_geometry
      1 name=config
     21 name=constant_compare
      1 name=discrimination_self_proof
      1 name=done
      7 name=h4_config
      8 name=h4_dominance_across_points
     14 name=h4_instance_rows
      4 name=h4_minimum
     12 name=h4_monotonicity
     28 name=h4_mount_tally
     28 name=h4_reading
     28 name=h4_reading_checks
     56 name=h4_reading_device
    203 name=h4_trial
      4 name=positive_control_admission
      1 name=positive_control_heap
     10 name=shard_done
     11 name=verdict
```

登记的「臂 × 历史 × 几何」数一遍：四个小盘搜索项与三个默认盘项共 7 项，每项两种状态（空池、有文件）× 两个读数入口（`writable_mount`、`before_a_publish`）× 两块盘 = 56 条 `h4_reading_device`，28 条 `h4_reading` 与 28 条 `h4_reading_checks` 各一条一份读数；每个小盘搜索项一条 `h4_minimum`、一条 `positive_control_admission`、三条 `h4_monotonicity`（mkfs、空池挂载、有文件挂载）；203 条 `h4_trial` = 41 + 47 + 62 + 50 个尝试（四个小盘项）加默认盘 3 个，条数对得上，没有缺行、缺字段。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:81-126`（整段抄，未转述）**

````markdown
### 第 1 次重跑（清扫阈值的本地常量 369 改 365，只重跑第一段）

重跑登记 `research/prompts/e183-r1-prereg.md`（sha256 `9417c63368fbe1b6f6d0e04844214d07a98059ca1a2cd6bc409f608b5076fd59`；第十二节「修订」一段没有写）；问题单 `research/prompts/m3-close-b19-e183-threshold-365-questions.md` 两行。起因：C690（清扫触发阈值的默认值没定） 那一行写的默认值定为 365（用户 2026-10-29 定），`crates/singlefs-core/src/instance_table_sweep.rs` 第 74 行已是 `Self(365)`，装置里的本地常量 `SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 还是 369，两条快档用例在 2026-10-30 红（登记第四节读数：回比用例报「本地 369 实现 365」，逐字节用例同时红）。只改了装置两行（常量值 369 改 365，文档注释改抄新的登记位；常量名与产物里的键名 `sweep_threshold_placeholder_rows` 不改），然后重出第一段 `all` 产物；第二段、第三段、第四段这一次不碰。

跑法与前提（都是这一趟现查的）：

- 起跑前提四样成立：`instance_table_sweep.rs:74` 是 `Self(365)`；装置 `grep -n SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 命中第 78、1879 两行；两份 2026-10-23 旧产物的 `sha256sum` 与登记相同（`b4ff64f1…`、`da09f826…`）；`anchors_e183.py` 与 `small_disk_anchor_check.py` 的 `sha256sum` 与登记相同。
- 装置由 `scripts/hosts/compile-then-swap.py --release` 编过再整份换进主工作区（退 0）；二进制 sha256 `4130ccac6b3f66d0817e2f318362f558e65ac658de28fe3abc685e898078d190`；源文件在产物那一刻 sha256 `56a6b495d92699d42eb06cccd8843f54557c9b06ba893be452000bc23a168810`，与草稿副本逐字节相同，比换之前主工作区那一份（`b3b37608…`）只多那两行。`find crates -name '*.rs' -newer <二进制>` 在 `singlefs-core`、`singlefs-format`、`singlefs-harness` 下零命中（只列出 checker 档包里另几个文件）。
- 被测树：`git diff HEAD -- crates/ | sha256sum` 为 `e5f1218c991320c5456abbd300932d2aa1937a8790b7848fb2712de5deff0f0f`，`--stat` 末行「193 files changed, 54733 insertions(+), 5377 deletions(-)」，与登记第三节写的 `97a92443…`（182 个文件）不同，原因是别的会话在同一棵树上继续改；`all` 用到的 `singlefs-core` 没有比二进制新的源文件。
- `all` 跑两遍（并行，各经内存包装 8G、线程上限 31，装置单线程），都退 0，`cmp` 逐字节相同，sha256 `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870`；两遍一起 226 秒。落成产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`，459 行、152434 字节、`grep -vc '^E7RESULT '` 为 0、末字节是换行、末行 `E7RESULT name=done emitted=459`。
- A31 核对：`python3 -B -I …/small_disk_anchor_check.py …/anchors_e183.py <新产物>` 退 0，输出落成 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`（58 行，sha256 `eaecf66cbfa6057b56bdf71e470d55998061388c6f5ad52c7fd307cc7b144c00`），末行 `E7RESULT name=verdict item=anchor_check readings_checked=56 mismatches=0`。

逐行差（问题单第 1 行；`diff 2026-10-23 产物 2026-10-30 产物` 的原样输出，1 个 hunk）：

```text
6c6
< E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=369 implementation=369 equal=true
---
> E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=365 implementation=365 equal=true
```

判定（比较脚本 `compare_e183_first_rerun.py` 的输出，脚本与输出在执行员草稿目录，不入库；纯 Python，十二条自检用例全过）：

| 量 | 判定 |
|---|---|
| 逐行差 | 只变第 6 行：恰好一个 hunk，头 `6c6`，旧行逐字等于登记引的旧第 6 行，新行逐字等于登记钉的新行 |
| 第 6 行新值 | 对：`local=365 implementation=365 equal=true` |
| 行数与字节数 | 459 行、152434 字节，与旧产物相同；非 `E7RESULT` 起头的行 0 条 |
| 回比判决行 | 对：`name=verdict item=constants constants_compared=21 constants_mismatched=0` |
| 判决行（下面「判决」一节那 11 行） | 11 行逐字段比，全部没变：H4 的判决行没变 |
| 两遍确定性 | 两遍逐字节相同 |
| A31 核对 | 对得上：退 0，56 份读数 0 处不符 |
| A31 输出差 | 只变产物 sha 那一行：1 个 hunk `1c1`，只差 `product_sha256=` 的值，新值等于新产物的 `sha256sum` |
| 两边是否一致 | 一致：产物只变第 6 行、核对输出只变头一行，没有互相对不上的地方 |
| 旧产物未被覆盖 | 对：两份 2026-10-23 产物的 `sha256sum` 跑完之后仍是登记里记的 |

产物里每一类结果行的条数与旧产物逐字相同（见「复跑」一节那条命令）；新产物里 `name=verdict` 共 11 条，整行与「判决」一节抄的那 11 行逐字相同，行号相同（第 23、26、49、120、197、289、369、395、421、447、458 行）。11 条里取值为 `false` 的字段是：第 120、197、289、369 行的 `positive_control_admission_holds=false` 与 `reading_checks_hold=false`，第 395、421、447 行的 `reading_checks_hold=false`，第 458 行的 `dominant_item_stable_across_default_disk_points=false`；原因与旧产物一样（空池一半 mkfs 先败、K32 空池 `before_a_publish` 入口按 G8 记、默认盘主导项跨 S 不稳定，见本页第一节「现状」），都是登记预期内的；没有 `not_run`，违例、不匹配、歧义、失败类的整数计数都是 0（`panicked_trials=0`、`anchor_mismatches=0`、`constants_mismatched=0`、`invalid_device_conditions=0`）。

答问题单第 1 行：常量改 365 之后第一段 `all` 产物只变第 6 行那一行比对，别的行不变，H4 的 11 条判决行不变。登记 P6 推的（只变第 6 行）与量出的一致。

换钉值与复跑登记：判决行没变、第 6 行对、A31 对得上，登记放行，所以复跑登记两行和 `driver_e183_anchor_check` 读的产物改指新文件（`scripts/experiment-apparatus/replay.sh` 第 1811、1812、3275 行），比对对象登记两行进 `.claude/gate.d/replay-comparison-units.tsv`；装置里钉的第一段 SHA-256 与行数（`SEGMENT_ONE_PINNED_SHA256` 改 `7c5add85…`，行数 459 不变）在草稿副本里改好，由主 agent 经 `compile-then-swap.py`（整条经 `run-with-memory-cap.sh 24G`）换进主工作区，退 0（换上之前主工作区那一份钉的是 `b4ff64f1…`，而装置的输出已是 `7c5add85…`）。旧的两份产物留着，不改名不删。

问题单第 2 行：A31 对得上（上表）。两条快档用例（`local_constants_equal_the_implementation_constants`、`segment_one_all_output_equals_the_registered_product_byte_for_byte`）是 checker 档重型测试，执行员不跑，钉值换进主工作区之后由主 agent 在主工作区跑，结果「2 passed; 0 failed」（调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 2026-10-30「E183 U3 先编后换」那一条）；改前的红是上面登记第四节读数的那两条。

变异：这一次不新加变异行。装置里变异行 54 条，改完之后每一行的原文在装置里仍恰好命中一次（草稿副本与主工作区两份各数一遍，不是 1 次的行 0 条）；变异表里没有一行的原文含常量那一行或钉值。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:127-152`（整段抄，未转述）**

````markdown
### 判决（整行抄自 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` 第 23、26、49、120、197、289、369、395、421、447、458 行；2026-10-30 的产物这 11 行逐字相同，见上一节）

```text
E7RESULT name=verdict item=constants constants_compared=21 constants_mismatched=0
E7RESULT name=verdict item=positive-control-heap positive_control_heap_holds=true
E7RESULT name=verdict item=anchors anchor_mismatches=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-4 positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-8 positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-16 positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-default-disk-slots-per-region-4 both_states_admitted=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-default-disk-slots-per-region-8 both_states_admitted=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-default-disk-slots-per-region-16 both_states_admitted=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4_summary dominant_item_stable_across_small_disk_points=true dominant_item_stable_across_default_disk_points=false discrimination_self_proof_holds=true invalid_device_conditions=0
```

`grep -n 'name=verdict' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` 列出判决行 11 条。没过的字段逐个点名：

| 字段 | 产物行 | 取值 | 为什么是这个值 | 登记预期内吗 |
|---|---|---|---|---|
| `positive_control_admission_holds=false` | 120、197、289、369（四个小盘搜索项） | false | PC-准入 的空池一半没成立：`name=positive_control_admission` 行的 `empty_mount_one_step_below=not_reached`，C_挂空 − 1 MiB（17 MiB 或 L = 4 MiB 时的 20 MiB）上 mkfs 先败在 `UnitAreaBeyondDevice`，空池挂载没走到 | 不在原登记预期内；按 V33 原文记一次输（见「作废、失败、停机条款逐条」），修订另补的 PC-准入-文件 4 项成立 |
| `reading_checks_hold=false` | 120、197、289、369（小盘）、395、421、447（默认盘） | false | 这 7 项的 `name=h4_reading_checks` 里 `state=empty entry=before_a_publish` 的 `k32_ring_window_by_the_clauses=false`：发布入口上空池读数的环窗口被扣（`ring_window_bytes` 非 0），条款句（K10）说空池为 0；其余 K30、K31、容量、有文件状态的 K32 全是 true | 登记修订第三条预期内的结果，记 G8 |
| `dominant_item_stable_across_default_disk_points=false` | 458 | false | 默认盘有文件状态下主导项 S = 4 为挂载期承诺量、S = 8、16 为环窗口，`name=h4_dominance_across_points group=default_disks state=file` 两行 `stable=false`（产物第 455、456 行） | 登记预期内的一种结果（第八节 8.2「主导项跨点不同 ⇒ 不稳定」） |

其余判决字段：`constants_mismatched=0`、`anchor_mismatches=0`、`panicked_trials=0`（7 处）、`invalid_device_conditions=0` 都是 0；`positive_control_heap_holds=true`、`every_minimum_disk_monotone_over_the_window=true`（4 处）、`discrimination_self_proof_holds=true`、`dominant_item_stable_across_small_disk_points=true`、`both_states_admitted=true`（3 处）都是 true；`grep -c 'not_run'` 对产物为 0，没有空字段（`grep -c '=\( \|$\)'` 为 0）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:153-206`（整段抄，未转述）**

````markdown
### 读数：三个最小盘（R4a）与阳性对照

整行抄自 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` 第 93、97、170、174、262、266、342、346 行（每个小盘搜索项一条 `h4_minimum`、一条 `positive_control_admission`）：

```text
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-4 make_filesystem_mebibytes=18 empty_mount_mebibytes=18 file_mount_mebibytes=28 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=25 file_mount_over_make_filesystem=1555permille trials=41 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-4 mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=17 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-8 make_filesystem_mebibytes=18 empty_mount_mebibytes=18 file_mount_mebibytes=34 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=31 file_mount_over_make_filesystem=1888permille trials=47 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-8 mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=17 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-16 make_filesystem_mebibytes=18 empty_mount_mebibytes=18 file_mount_mebibytes=49 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=46 file_mount_over_make_filesystem=2722permille trials=62 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-16 mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=17 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes make_filesystem_mebibytes=21 empty_mount_mebibytes=21 file_mount_mebibytes=37 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=34 file_mount_over_make_filesystem=1761permille trials=50 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=20 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
```

- 最小盘（MiB）：`make_filesystem_mebibytes`（C_mkfs）= `empty_mount_mebibytes`（C_挂空），有文件挂载 `file_mount_mebibytes`（C_挂文件）是 28 / 34 / 49 / 37，`file_mount_over_make_filesystem` 是 1555 / 1888 / 2722 / 1761 千分之一；搜索从 4 MiB 起按 1 MiB 步进，到最大的最小盘之后 16 MiB 止（`name=h4_config` 的 `window_mebibytes=16`），上沿 1024 MiB 没碰到，极值都不是端点。
- 「挂上但推满仍不够」的最小盘 `file_mount_still_short_mebibytes` 是 25 / 31 / 46 / 34，比 C_挂文件 小 3 MiB；空池状态下这一栏 `empty_mount_still_short_mebibytes=not_found`（空池挂载在搜索范围内一次也没出现推满仍不够）。
- PC-准入（空池一半）：`mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch`（只供测试的开关在 mkfs 最小盘上挂得上，这一半成立）；`empty_mount_one_step_below=not_reached`（17 MiB、L = 4 MiB 时 20 MiB 上 mkfs 败在 `UnitAreaBeyondDevice`，空池挂载没走到）。
- PC-准入-文件：`file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises` 4 项都是，四项的 `file_mount_mebibytes` 都找得到。数法：

```bash
P=research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out
grep 'name=positive_control_admission' $P | grep -c 'file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises'
grep 'name=positive_control_admission' $P | grep -c 'empty_mount_one_step_below=failed:SpaceAdmissionRefusedBeforeAcquisition'
grep 'name=h4_minimum' $P | grep -vc 'file_mount_mebibytes=not_found'
```

原样输出依次为 `4`、`0`、`4`。

每个小盘搜索项里尝试按盘大小分段的结局（`name=h4_trial` 行，同一结局的连续 MiB 合并；逐 MiB 读法：`grep 'name=h4_trial item=h4-small-disk-slots-per-region-8 ' $P | sed -e 's/.*device_mebibytes=\([0-9]*\) make_filesystem=\([^ ]*\) empty_mount=\([^ ]*\) first_file=\([^ ]*\) unmount=[^ ]* file_mount=\([^ ]*\) .*/\1 \2 \3 \4 \5/'`，`$P` 取 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out`，换项名读另三个项）：

| 项 | 盘大小 MiB | mkfs | 空池挂载 | 首个文件 | 有文件挂载 |
|---|---|---|---|---|---|
| S = 4 | 4–7 | `RootRingBeyondDevice` | 未走到 | 未走到 | 未走到 |
| S = 4 | 8–17 | `UnitAreaBeyondDevice` | 未走到 | 未走到 | 未走到 |
| S = 4 | 18 | 成 | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | 未走到 |
| S = 4 | 19–24 | 成 | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | 未走到 |
| S = 4 | 25–27 | 成 | `AdmittedBeforeAcquisition` | 成 | `StillShortAfterTheFloorRaises` |
| S = 4 | 28–44 | 成 | `AdmittedBeforeAcquisition` | 成 | `AdmittedBeforeAcquisition` |
| S = 8 | 18 | 成 | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | 未走到 |
| S = 8 | 19–30 | 成 | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | 未走到 |
| S = 8 | 31–33 | 成 | `AdmittedBeforeAcquisition` | 成 | `StillShortAfterTheFloorRaises` |
| S = 8 | 34–50 | 成 | `AdmittedBeforeAcquisition` | 成 | `AdmittedBeforeAcquisition` |
| S = 16 | 18 | 成 | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | 未走到 |
| S = 16 | 19–45 | 成 | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | 未走到 |
| S = 16 | 46–48 | 成 | `AdmittedBeforeAcquisition` | 成 | `StillShortAfterTheFloorRaises` |
| S = 16 | 49–65 | 成 | `AdmittedBeforeAcquisition` | 成 | `AdmittedBeforeAcquisition` |
| S = 8、L = 4 MiB | 21 | 成 | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | 未走到 |
| S = 8、L = 4 MiB | 22–33 | 成 | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | 未走到 |
| S = 8、L = 4 MiB | 34–36 | 成 | `AdmittedBeforeAcquisition` | 成 | `StillShortAfterTheFloorRaises` |
| S = 8、L = 4 MiB | 37–53 | 成 | `AdmittedBeforeAcquisition` | 成 | `AdmittedBeforeAcquisition` |

读法：C_挂空 之上的 1 MiB 起空池挂载全程放行；有文件状态的链上，首个文件发布先被准入拒（`SpaceAdmissionRefused`，S = 8 为 19 至 30 MiB），再是挂载做成但推满仍不够（31 至 33 MiB），到 34 MiB 才放行，C_挂文件 上的可用(d) 只剩 32692 字节（`name=h4_trial` 第 S = 8 项 34 MiB 行的 `file_available_minimum_bytes=32692`，33 MiB 行是 −1015884）。mkfs 最小盘那一格（18 MiB）首个文件败在 `EmptyClusterSegmentsBelowTheFloor`，不是准入拒。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:207-247`（整段抄，未转述）**

````markdown
### 读数：最小盘上的各项与总和（R4b）、谁主导（R4c）

有文件状态、`writable_mount` 入口、设备 0 的读数（整行抄自 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` 第 110、187、279、359 行，依次是 S = 4、S = 8、S = 16、S = 8 且 L = 4 MiB，盘分别取各自的 C_挂文件）：

```text
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-4 state=file entry=writable_mount device=0 capacity_bytes=11534336 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=2883584 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=229376 ring_window_bytes=7241728 admission_reserve_bytes=10453068 available_bytes=425908 reserve_over_capacity_parts_per_million=906256 share_permille_mount_time_commitment=275 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=9 share_permille_checkpoint_reserve_pool_share=21 share_permille_ring_window=692 dominance=dominant:ring_window:692permille
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-8 state=file entry=writable_mount device=0 capacity_bytes=17825792 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=2883584 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=229376 ring_window_bytes=13926400 admission_reserve_bytes=17137740 available_bytes=32692 reserve_over_capacity_parts_per_million=961401 share_permille_mount_time_commitment=168 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=5 share_permille_checkpoint_reserve_pool_share=13 share_permille_ring_window=812 dominance=dominant:ring_window:812permille
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-16 state=file entry=writable_mount device=0 capacity_bytes=33554432 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=3276800 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=262144 ring_window_bytes=28901376 admission_reserve_bytes=32538700 available_bytes=360372 reserve_over_capacity_parts_per_million=969728 share_permille_mount_time_commitment=100 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=3 share_permille_checkpoint_reserve_pool_share=8 share_permille_ring_window=888 dominance=dominant:ring_window:888permille
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes state=file entry=writable_mount device=0 capacity_bytes=17825792 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=2883584 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=229376 ring_window_bytes=13926400 admission_reserve_bytes=17137740 available_bytes=32692 reserve_over_capacity_parts_per_million=961401 share_permille_mount_time_commitment=168 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=5 share_permille_checkpoint_reserve_pool_share=13 share_permille_ring_window=812 dominance=dominant:ring_window:812permille
```

两块盘逐项相同、两个入口在有文件状态下逐项相同（只差 `entry` 字段）：`grep 'name=h4_reading_device' $P | sed 's/ device=[01] / device=_ /' | sort | uniq -c | awk '$1!=2' | wc -l` 输出 `0`（不重复的 28 份读数每份恰两条，两块盘各一）；`grep 'name=h4_reading_device' $P | grep 'state=file' | sed 's/ entry=[a-z_]* / entry=_ /; s/ device=[01] / device=_ /' | sort | uniq -c | awk '{print $1}' | sort | uniq -c` 输出 `7 4`（7 份有文件状态读数，每份 4 条）。

| 点 | 容量（单元区字节） | 挂载期承诺量 | checkpoint 保留池份额 | 环窗口 | 准入预留（六项和） | 可用(d) | 准入预留 ÷ 容量（ppm） | 主导项 |
|---|---|---|---|---|---|---|---|---|
| S = 4，C = 28 MiB | 11534336 | 2883584 | 229376 | 7241728 | 10453068 | 425908 | 906256 | 环窗口 692 千分之一 |
| S = 8，C = 34 MiB | 17825792 | 2883584 | 229376 | 13926400 | 17137740 | 32692 | 961401 | 环窗口 812 千分之一 |
| S = 16，C = 49 MiB | 33554432 | 3276800 | 262144 | 28901376 | 32538700 | 360372 | 969728 | 环窗口 888 千分之一 |
| S = 8，L = 4 MiB，C = 37 MiB | 17825792 | 2883584 | 229376 | 13926400 | 17137740 | 32692 | 961401 | 环窗口 812 千分之一 |

六项里没单列的三项：被抛弃根独占量、待删占用份额在全部 56 条读数里都是 0（`grep 'name=h4_reading_device' $P | grep -c 'abandoned_root_exclusive_bytes=0 '` 与 `pending_delete_share_bytes=0 ` 都输出 `56`）；已承诺预留份额在有文件状态的读数里全是 98380 字节（28 条）、空池状态全是 0（28 条），两个数的核法是 `grep 'name=h4_reading_device' $P | grep -o 'state=[a-z]* \|committed_reservation_share_bytes=[0-9]*' | paste - - | sort | uniq -c`（输出 `28 state=empty … =0`、`28 state=file … =98380` 两行）。表里「准入预留（六项和）」已含这 98380 字节。R4b 的比例在最小盘上近乎同义反复（可用(d) 刚好 ≥ 0，比例约 1 − 已分配 ÷ 容量），登记已写「照报、不算判据」；翻面靠 R4c、R4d。

R4c 跨点汇总与判别力自证（整行抄自同一产物第 449 至 457 行、第 459 行；第 449、450 行的 `state=empty` 是 V33 记输之后不出的空池一半，留档不当 R4a–R4c 的结果）：

```text
E7RESULT name=h4_dominance_across_points group=small_disks state=empty entry=writable_mount points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:mount_time_commitment:952permille;h4-small-disk-slots-per-region-8=dominant:mount_time_commitment:952permille;h4-small-disk-slots-per-region-16=dominant:mount_time_commitment:952permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:mount_time_commitment:952permille stable=true
E7RESULT name=h4_dominance_across_points group=small_disks state=empty entry=before_a_publish points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:ring_window:951permille;h4-small-disk-slots-per-region-8=dominant:ring_window:974permille;h4-small-disk-slots-per-region-16=dominant:ring_window:986permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:ring_window:974permille stable=true
E7RESULT name=h4_dominance_across_points group=small_disks state=file entry=writable_mount points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:ring_window:692permille;h4-small-disk-slots-per-region-8=dominant:ring_window:812permille;h4-small-disk-slots-per-region-16=dominant:ring_window:888permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:ring_window:812permille stable=true
E7RESULT name=h4_dominance_across_points group=small_disks state=file entry=before_a_publish points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:ring_window:692permille;h4-small-disk-slots-per-region-8=dominant:ring_window:812permille;h4-small-disk-slots-per-region-16=dominant:ring_window:888permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:ring_window:812permille stable=true
E7RESULT name=h4_dominance_across_points group=default_disks state=empty entry=writable_mount points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:mount_time_commitment:952permille;h4-default-disk-slots-per-region-8=dominant:mount_time_commitment:952permille;h4-default-disk-slots-per-region-16=dominant:mount_time_commitment:952permille stable=true
E7RESULT name=h4_dominance_across_points group=default_disks state=empty entry=before_a_publish points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:ring_window:995permille;h4-default-disk-slots-per-region-8=dominant:ring_window:997permille;h4-default-disk-slots-per-region-16=dominant:ring_window:998permille stable=true
E7RESULT name=h4_dominance_across_points group=default_disks state=file entry=writable_mount points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:mount_time_commitment:546permille;h4-default-disk-slots-per-region-8=dominant:ring_window:569permille;h4-default-disk-slots-per-region-16=dominant:ring_window:721permille stable=false
E7RESULT name=h4_dominance_across_points group=default_disks state=file entry=before_a_publish points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:mount_time_commitment:546permille;h4-default-disk-slots-per-region-8=dominant:ring_window:569permille;h4-default-disk-slots-per-region-16=dominant:ring_window:721permille stable=false
E7RESULT name=discrimination_self_proof judgement=r4c_dominance readings=28 flipped=28 holds=true
E7RESULT name=done emitted=459
```

- 有文件状态（R4c 的结果行是第 451、452 行）：四个小盘取样点的主导项都是环窗口，份额 692 / 812 / 888 / 812 千分之一，`stable=true`（S 越大份额越高，L = 4 MiB 与 L = 1 MiB 同项）。过半门槛 500 千分之一，四点都过。
- 判别力自证（第 457 行）：把 R4c 的门槛挪到每份读数最大份额的另一侧，28 份读数 28 份由「主导」转面，`holds=true`，V36 不触发。
- 空池状态（留档）：`writable_mount` 入口主导项挂载期承诺量 952 千分之一（容量只有 1 MiB、六项和 344064 字节、可用(d) 655360 字节）；`before_a_publish` 入口主导项环窗口 951 / 974 / 986 / 974 千分之一，可用(d) 为负（−6160384 / −12451840 / −25034752 / −12451840 字节），即首个文件那次发布的准入在这四个盘上读数就是拒的。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:248-268`（整段抄，未转述）**

````markdown
### 读数：默认盘（R4d）

整行抄自同一产物第 374、385、400、411、426、437 行（`writable_mount` 入口、设备 0；每个 S 先空池、后有文件，盘是 4 GiB × 2、L = 768 MiB，单元区容量 3472883712 字节）：

```text
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-4 state=empty entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=49152 unreclaimable_bytes=0 mount_time_commitment_bytes=327680 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=0 checkpoint_reserve_pool_share_bytes=16384 ring_window_bytes=0 admission_reserve_bytes=344064 available_bytes=3472490496 reserve_over_capacity_parts_per_million=99 share_permille_mount_time_commitment=952 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=47 share_permille_ring_window=0 dominance=dominant:mount_time_commitment:952permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-4 state=file entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=819200 unreclaimable_bytes=0 mount_time_commitment_bytes=106889216 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=8896512 ring_window_bytes=79659008 admission_reserve_bytes=195543116 available_bytes=3276521396 reserve_over_capacity_parts_per_million=56305 share_permille_mount_time_commitment=546 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=45 share_permille_ring_window=407 dominance=dominant:mount_time_commitment:546permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-8 state=empty entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=49152 unreclaimable_bytes=0 mount_time_commitment_bytes=327680 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=0 checkpoint_reserve_pool_share_bytes=16384 ring_window_bytes=0 admission_reserve_bytes=344064 available_bytes=3472490496 reserve_over_capacity_parts_per_million=99 share_permille_mount_time_commitment=952 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=47 share_permille_ring_window=0 dominance=dominant:mount_time_commitment:952permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-8 state=file entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=819200 unreclaimable_bytes=0 mount_time_commitment_bytes=106889216 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=8896512 ring_window_bytes=153190400 admission_reserve_bytes=269074508 available_bytes=3202990004 reserve_over_capacity_parts_per_million=77478 share_permille_mount_time_commitment=397 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=33 share_permille_ring_window=569 dominance=dominant:ring_window:569permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-16 state=empty entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=49152 unreclaimable_bytes=0 mount_time_commitment_bytes=327680 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=0 checkpoint_reserve_pool_share_bytes=16384 ring_window_bytes=0 admission_reserve_bytes=344064 available_bytes=3472490496 reserve_over_capacity_parts_per_million=99 share_permille_mount_time_commitment=952 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=47 share_permille_ring_window=0 dominance=dominant:mount_time_commitment:952permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-16 state=file entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=819200 unreclaimable_bytes=0 mount_time_commitment_bytes=106889216 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=8896512 ring_window_bytes=300253184 admission_reserve_bytes=416137292 available_bytes=3055927220 reserve_over_capacity_parts_per_million=119824 share_permille_mount_time_commitment=256 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=21 share_permille_ring_window=721 dominance=dominant:ring_window:721permille
```

| S | 空池准入预留 ÷ 容量（ppm） | 有文件准入预留 ÷ 容量（ppm） | 有文件主导项 |
|---|---|---|---|
| 4 | 99 | 56305 | 挂载期承诺量 546 千分之一 |
| 8 | 99 | 77478 | 环窗口 569 千分之一 |
| 16 | 99 | 119824 | 环窗口 721 千分之一 |

默认盘有文件状态下，环窗口的字节与 A31 的锚点值相同（S = 4 / 8 / 16 为 79659008 / 153190400 / 300253184，核对输出见 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out`）；挂载期承诺量 106889216 字节随 S 不变（S = 4 时它比环窗口大，所以主导项在 S = 4 与 S = 8 之间换手）。空池状态 `before_a_publish` 入口准入预留 ÷ 容量是 23036 / 44209 / 86555 ppm（第 378、404、430 行；这一入口的环窗口在空池上被扣，K32 的空池一半不成立，见「作废、失败、停机条款逐条」）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:269-288`（整段抄，未转述）**

```markdown
### 作废、失败、停机条款逐条（第一段）

| 条款 | 状态 | 读数 |
|---|---|---|
| 起跑前提不成立（停机） | 过 | 登记第十二节「修订」第二条；`grep -n '问法待'` 只剩这条停机条款自己引它的那一句，命令一重跑输出 sha256 与登记相同 |
| T2 本地常量回比 | 过 | 第 23 行 `constants_compared=21 constants_mismatched=0` |
| T3 装置现算几何与锚点脚本 | 过 | 第 49 行 `anchor_mismatches=0`；`name=anchor_geometry` 15 行、`name=anchor_chain_rewrite` 5 行 |
| T4 确定性两遍 | 过 | 产物与第二遍输出 `cmp` 无输出，退出码都是 0 |
| T5、C0 补丁命中不是 1 次或 A41 不符（停机） | 不适用 | 属于第三段、第四段 |
| V30 PC-堆 | 过 | 第 25 行 `E7RESULT name=positive_control_heap peak_over_baseline_bytes=67108864 lower_bytes=67108864 upper_bytes=67174400 holds=true` |
| V33 PC-准入 | **空池一半打中，记一次输；有文件一半（修订补的 PC-准入-文件）不触发** | 空池：4 个小盘项 `empty_mount_one_step_below=not_reached`，0 项成立，R4a–R4c 的空池一半不出，最小盘归 mkfs 的几何；有文件：4 项 `file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises`，R4a 有文件一半、R4b、R4c 有文件状态照出 |
| V36 判别力自证 | 过 | 第 457 行 `flipped=28 holds=true` |
| V37 读不到不当 0 | 没触发 | 产物没有空字段（`grep -c '=\( \|$\)'` 为 0）；`none` 只出现在「没走到」的字段（`file_available_minimum_bytes=none` 等） |
| G4 最小盘不单调 | 没触发 | 12 条 `name=h4_monotonicity` 都是 `monotone=true`、`failures_in_the_window_mebibytes=none` |
| G7 K31 两边 c 不等 | 没触发 | 28 份读数的 `k31_mount_time_commitment_by_the_clauses=true`，`checkpoint_cost_blocks_by_the_pool` 与 `checkpoint_cost_blocks_by_the_commitment` 两边相同（取值 1 有 14 份读数、14 有 6 份、16 有 2 份、543 有 6 份，`grep 'name=h4_reading_checks' $P \| grep -o 'checkpoint_cost_blocks_by_the_pool=[0-9]* checkpoint_cost_blocks_by_the_commitment=[0-9,]*' \| sort \| uniq -c` 列出） |
| G8 条款这一格可能错 | **K32 空池 `before_a_publish` 入口 7 份读数触发** | `grep 'name=h4_reading_checks' $P \| grep 'k32_ring_window_by_the_clauses=false' \| grep -c 'state=empty entry=before_a_publish'` 输出 `7`，而 `k32_ring_window_by_the_clauses=false` 总数也是 7；其余 21 份 K32 是 true（空池 `writable_mount` 入口 7 份环窗口为 0，有文件状态两个入口 14 份与条款句现算相等）；K30（待删占用为 0）28 份都是 true |
| G1、G2、G3、G5、G6 | 不适用 | 属于第二到四段 |

K32 的位置：条款句 D28（挂载期承诺量） 已定项 1 的环窗口一条只写「常驻扣项、按几何现算」，登记 K32 的空池为 0 一半出自 K10（`crates/singlefs-core/src/admission.rs` 注释引的「环窗口从第一个文件版本那次发布起计」，只罩可写挂载那一判）。实现里 `before_a_publish` 入口对空池读数照扣环窗口（`ring_window_bytes` 为 S = 4 / 8 / 16 时 6815744 / 13107200 / 25690112 字节），产物里这一格 `ring_window_by_the_clauses_if_deducted_bytes` 与之相同。脚本（A31 核对里空池 `before_a_publish` 按「扣」读，与实现一致）与装置现算都没错，实现与条款这一格对不上，记「条款的这一格可能错」，没有判哪一边错。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:289-311`（整段抄，未转述）**

```markdown
### 单测与变异

- 单测：`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` 输出 `24`；主 agent 跑 `cargo test --release` 的读数是「test result: ok. 24 passed; 0 failed」。
- 变异表：入库装置的变异行追加进 `crates/mutations.tsv` 末尾，数 `crates/mutations.tsv` 里名字以 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 加一个空格起头的行（`grep -c`），输出 `13`；每条的原文在装置源文件里恰好命中一次、点名的测试函数都存在，不含 `\n` 之外的反斜杠转义。抓到 / 无效 / 没红三个数待提交时门禁 checker-tier-crates-mutation-replay 整表复跑，这一段没跑变异。

| 变异 | 改什么 | 点名的单测 |
|---|---|---|
| M60 | 计数分配器的峰值更新由取大改成直接存此刻 | `counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed` |
| M64 | 最小盘搜索把推满仍不够当成成 | `a_mount_that_stays_short_after_raising_the_floor_is_not_counted_as_admitted` |
| M65 | 准入预留求和漏加环窗口 | `admission_reserve_adds_all_six_items_including_the_ring_window` |
| M70 | 读写计数包装把系统配置区的上沿算进系统配置区 | `fixed_regions_classify_offsets_by_their_landing_zone` |
| M71 | 单调核的窗口少核最后一个点 | `monotonicity_lists_every_failure_in_the_sixteen_mebibyte_window_and_nothing_beyond` |
| M72 | 主导判定把恰好过半算成没过半 | `dominance_needs_at_least_half_of_the_total` |
| M73 | 链重写的片数式子丢掉 N_switch | `chain_rewrite_bytes_cross_to_a_second_page_exactly_at_the_registered_row` |
| M74 | 单元区槽数少数一槽 | `local_geometry_reproduces_every_registered_script_row` |
| M75 | 搜索停得早一步 | `search_stops_sixteen_mebibytes_after_the_largest_of_the_three_minima` |
| M76 | 挂载期承诺量少乘一份写行那次的元数据 | `mount_time_commitment_by_the_clauses_matches_the_hand_computed_value` |
| M77 | 空池的环窗口期望值改成非零 | `reading_checks_hold_on_a_reading_built_by_the_clauses_and_fail_when_one_item_is_off` |
| M78 | 有文件状态不要求首个文件与卸载都成 | `the_file_state_needs_every_step_of_the_chain_to_have_succeeded` |
| M79 | 判别力自证的门槛没挪过最大份额 | `moving_the_dominance_threshold_across_the_largest_share_flips_the_class` |

取样点：M64 在「推满仍不够」比「成」先出现的盘上才改变输出，产物的四个小盘项都出现（`file_mount_still_short_mebibytes` 比 `file_mount_mebibytes` 小 3 MiB），产物上敏感；M72 只在最大份额恰为 500 千分之一时不同，产物 56 条读数的最大份额没有一个是 500（`grep 'name=h4_reading_device' $P \| grep -o 'dominance=[a-zA-Z_:0-9]*' \| sort \| uniq -c` 列出 13 种），只在单测的合成点上敏感；M65 在有文件状态与默认盘上敏感，空池 `writable_mount` 入口环窗口为 0 时不敏感。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:312-322`（整段抄，未转述）**

```markdown
### 岔路表（问题单 H4；登记第十一节 H4 的够判条件是 R4a、R4c、R4d 都出了数）

| 问题单行 | 现状 | 还差什么 | 剩下的量能不能让它翻面 | 算出它的命令（$P 取 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out`） |
|---|---|---|---|---|
| H4 · R4a 三个最小盘 | 已够判（有文件那一半）：C_mkfs、C_挂空、C_挂文件、推满仍不够的最小盘四个点都出数；空池那一半 V33 打中，最小盘归 mkfs 几何，不出 | 无 | 登记里没跑的量（H2、H3、H5）与 H4 无关，不能让它翻面 | `grep 'name=h4_minimum\|name=positive_control_admission' $P` |
| H4 · R4b 准入预留 ÷ 容量（只报数） | 已出：有文件状态最小盘上 906256 / 961401 / 969728 ppm；空池那一半不出 | 无 | 不会翻面（最小盘上近乎同义反复，登记已写） | `grep 'name=h4_reading_device' $P \| grep 'state=file entry=writable_mount device=0 '` 取 `reserve_over_capacity_parts_per_million` |
| H4 · R4c 谁主导 | 已够判（有文件那一半）：小盘四点环窗口主导（692 / 812 / 888 / 812 千分之一）；默认盘有文件状态跨 S 不稳定；判别力自证 28 / 28 | 无 | 不会翻面 | `grep 'name=h4_dominance_across_points\|name=discrimination_self_proof' $P` |
| H4 · R4d 默认盘比例 | 已出：有文件状态 56305 / 77478 / 119824 ppm（S = 4 / 8 / 16），空池 99 ppm | 无 | 不会翻面（只报数） | `grep 'name=h4_reading_device' $P \| grep 'item=h4-default'` 取同一字段 |

H4 够判。登记里剩下的量（第二段 H2、第四段 H3）标「够判后未跑」属于问题单别的行，不属于 H4；H5 在第三段跑了（「第三段（H5）：跑法、产物与被测树」起的各节）；H2、H3 两行仍开着，是否续跑由主 agent 定。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:323-354`（整段抄，未转述）**

````markdown
### 第三段（H5）：跑法、产物与被测树

产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out`：19067 行，末行 `E7RESULT name=done emitted=19067`，sha256 `b25c75f5ae5fdba2181ce0fb44962fbe15b69062d709403ba4c81bd9581c3216`。独立核对输出 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out`（28 行，纯 Python，见下「复跑」）。

- 26 片：24 格阈值历史（S 取 4、8、16 × θ 取 64、365、369、738 × 卸载方式取正常卸载、崩溃）、「交给后台」对照 1 片、首片的确定性重跑 1 片。本机 13 片（12 个正常卸载格加对照）、第二台 13 片（12 个崩溃格加首片重跑），两台是同一份程序文件，sha256 都是 `ac1799e786281c7d2f8335b0358eb9a47d8b27711df7704b50ed755a06f926e5`（产物里出现 27 次，26 条 `h5_config` 加 1 条 `config`，没有第二个值）：

```text
     27 binary_sha256=ac1799e786281c7d2f8335b0358eb9a47d8b27711df7704b50ed755a06f926e5
```

- 起法（产物合并之后补记，不进跑前登记）：第一趟两台各编一份装置，程序文件里编进了编译树的绝对路径，两台 sha256 不同，装置按登记在合并时停机；第二台那 13 片整份移开作废（`sha256` 前 12 位 `d3f1a82da397`，留着不删），第二趟本机 13 片沿用，第二台用本机送去的同一份程序文件现跑 13 片，合并出这份产物。长跑脚本 `scripts/experiment-apparatus/e183-sweep-history-shards.sh`，sha256 `80501eb704b4d8a76e4d6027ee3840f557cbd8b15549dd182ac1386fbed15469`。
- 被测树：程序文件编自装置源文件 `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs`（sha256 `a17a554d5d8ae7c8b1a9a040e9a0bc54c1648084d3c8ba019ea2245c092f16d8`，73 个单测，主 agent 跑的读数是 `test result: ok. 73 passed; 0 failed`）。「起跑前提不成立」那条停机条款的第二样（登记第三节记的 `847e0e54…` 不同时在第十二节记新值）：此刻 `git diff HEAD -- crates/ | sha256sum` 为 `d471364a6f08c3f38e8091ea13a038891d81ae64b81122cf107b5ea334383acb`，`git diff HEAD --stat -- crates/ | tail -1` 末行「202 files changed, 108398 insertions(+), 4742 deletions(-)」（这是写这一节时的读数，不是产物跑那一刻的树）；`find crates -name '*.rs' -newer <程序文件>` 此刻只列出 `crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/tests/` 下两个测试文件与 `common_truncation_crash_streams/mod.rs`、`crates/singlefs-harness/tests/common_compaction_prefix/mod.rs`，`singlefs-core`、`singlefs-format` 的源文件在程序文件之后没动过。
- 产物每一类结果行的条数（`awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' $P | sort | uniq -c` 的原样输出，`$P` 取 H5 产物）：

```text
      1 name=config
      1 name=discrimination_self_proof
      1 name=done
     25 name=h5_anchor_model
     26 name=h5_cell_summary
     25 name=h5_chain_read_differential
     26 name=h5_config
     75 name=h5_differential
      4 name=h5_geometry_sensitivity
  18852 name=h5_mount
      6 name=h5_point
     24 name=h5_threshold_verdicts
      1 name=verdict
```

数一遍：26 条 `h5_config`、26 条 `h5_cell_summary`（24 格加对照加重跑）；`h5_anchor_model` 与 `h5_chain_read_differential` 各 25 条（24 格加重跑，对照没有模型与差分）；`h5_differential` 75 条（25 片 × 3：没越过阈值的第一次挂载、两次清扫挂载）；`h5_threshold_verdicts` 24 条、`h5_point` 6 条（3 个 S × 2 种卸载方式）、`h5_geometry_sensitivity` 4 条（4 个 θ）；`h5_mount` 18852 条，没有缺行、缺字段（`grep -c '=\( \|$\)'` 对产物输出 `0`，`grep -c 'not_run'` 输出 `0`）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:355-369`（整段抄，未转述）**

````markdown
### 复跑（第三段 H5）

```bash
# 仓根。H5 产物是长跑（本机加第二台），重型，由主 agent 起；程序文件用 --binary 钉住，不钉会重编 target/release
bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --binary <装置程序文件>

# 独立核对，纯 Python（不读 crates/、不 import 装置）：24 格逐格核挂载次数、两次清扫在第几次挂载、各删几行、清扫后剩几行、行数峰值、片数峰值、终了 txg，加对照一格
python3 -B -I scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_history_anchor_check.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out

# 经复跑登记（只含独立核对那一行）
bash scripts/experiment-apparatus/replay.sh E183
```

H5 产物本身不登记逐字节复跑：`h5_differential` 与 `h5_cell_summary` 里有挂钟字段（`mount_nanoseconds`、`sweep_mount_nanoseconds`），两趟本来就不逐字节相同，`replay.sh` 的 `strip_timing` 不抹这两个字段名，收尾行也没有 `loadavg1`、`cpus`（`timing` 档会判红）。登记的是读这份产物的独立核对（`replay.sh` 里 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 名下驱动 `@driver_e183_h5_anchor_check`、产物 `e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out`、判法 exact 那一行），它的输出是确定的。重跑的确定性核（T4）另在产物里：首片的重跑在第二台、原片在本机，去掉挂钟、进程高水位、片名与程序文件 sha256 之后逐字段相同（判决行 `repeat_of_the_first_shard_equal_outside_timing=true`），这一页再核一遍：把两片的行抄出来、片名换成同一个字、`*nanoseconds=` 抹成同一个记号再 `diff`，输出 `无差异`（两边都是 135 行）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:370-449`（整段抄，未转述）**

````markdown
### 第三段读数：行数轨迹（R5a）与片数峰值（R5b 甲）

24 格每格连挂到清扫发布成过两次。逐次挂载的行（`name=h5_mount`）共 18852 条，每条带装置读到的行数、写行之后的行数、清扫结局、删的行数与模型里同一次挂载的值。行数轨迹的几条计数，原样输出（`$P` 取 H5 产物）：

```bash
grep 'name=h5_mount' $P | grep -o ' k33=[a-z_]*\| k34=[a-z_]*\| a36=[a-z_]*\|trajectory_matches_the_model=[a-z]*' | sort | uniq -c
grep 'name=h5_anchor_model' $P | grep -o 'matches_the_registered_script_output=[a-z]*' | sort | uniq -c
grep 'name=h5_mount' $P | grep -o 'sweep=[a-z_]*' | sort | uniq -c
```

```text
  18802  a36=not_applicable
     50  a36=true
  18852  k33=true
  18802  k34=not_applicable
     50  k34=true
  18852 trajectory_matches_the_model=true
     25 matches_the_registered_script_output=true
    400 sweep=handed_to_the_background
  18402 sweep=not_swept_at_or_below_the_threshold
     50 sweep=rows_deleted
```

- K33（发了清扫发布 ⟺ rows0 > θ）：18852 次挂载全 `k33=true`。K34（删掉的行 = 根环里一条根都没有的实例的行，一行不多不少）与 A36（抹的头数为 0）：50 次清扫（24 格各 2 次，加首片重跑的 2 次）全 `true`，其余挂载 `not_applicable`。逐次的行数轨迹与模型一致：18852 次全 `trajectory_matches_the_model=true`，25 条 `h5_anchor_model` 全 `matches_the_registered_script_output=true`。T5 没触发。
- 独立核对（`sweep_history_anchor_check.py`，不 import 装置、不读 `crates/`，用登记第十三节命令一的锚点脚本 `anchors_e183.py`，sha256 `d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d`）：24 格 × （挂载次数、两次清扫在第几次挂载、各删几行、清扫后剩几行、行数峰值、片数峰值、终了 txg）加对照一格，不符 0 处。末几行原样（整份输出是 `…-h5-2026-10-19-anchor-check.out`）：

```text
E7RESULT name=anchor_check_control shard=h5-control-slots-per-region-8-handed-to-the-background-normal_unmount mounts=400 rows0_is_the_mount_number=True never_swept=True equal=True
E7RESULT name=verdict item=h5_anchor_check cells_checked=24 control_cells_checked=1 mismatches=0
E7RESULT name=done emitted=28
```

  `grep -c 'equal=True'` 对这份输出为 `25`（24 格加对照 1 格）。把产物里一格的 `rows_left` 改一个数再喂给核对脚本，退 1 并点名那一格 `equal=False`（草稿里的试红，没存）。
- A34（rows0 峰值 = θ + 1；片数峰值 θ = 64、365 为 1，369 为 2，738 为 3）：24 格全相符，见 R5a 与 R5b 甲 24 格读数表的 `行数峰值`、`片数峰值` 两列。A35（两次清扫在第几次挂载、各删几行、剩几行、共挂几次）登记里的例子 S = 8、θ = 369、正常卸载 `mounts=736 sweeps=[366, 366] residuals=[4, 4] mounts_at_sweep=[370, 736]` 在产物里的整行是第 7929 行：

```text
E7RESULT name=h5_anchor_model shard=h5-slots-per-region-8-threshold-369-normal_unmount model_mounts=736 model_sweeps_mount_deleted_left=370:366:4,736:366:4 model_final_txg=4418 matches_the_registered_script_output=true
```

R5a 与 R5b 甲在 24 格上的读数（`摘要行` 是 `name=h5_cell_summary` 在 H5 产物里的行号；写字节与读字节是 `h5_cell_summary` 的 `sweep_write_bytes`、`sweep_read_bytes` 两个字段，按 V35 不当 R5c、R5d 的结果，见「第三段读数：阳性对照 PC-不扫 与 PC-差分（V34、V35）」一节；表由 `python3 -I -B scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_cell_table.py $P table` 从产物里逐行读出，`$P` 取 H5 产物）：

| S | 卸载 | θ | 摘要行 | 挂载次数 | 行数峰值 | 片数峰值 | R5b 甲 | 清扫次数 | 期末行数 | 两次清扫的写字节 | 两次清扫的读字节 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 4 | 崩溃 | 64 | 270 | 127 | 65 | 1 | 不越过 | 2 | 3 | 344576 / 344576 | 436109312 / 436107264 |
| 4 | 崩溃 | 365 | 1743 | 729 | 366 | 1 | 不越过 | 2 | 3 | 344576 / 344576 | 436109312 / 436107264 |
| 4 | 崩溃 | 369 | 3232 | 737 | 370 | 2 | 越过 | 2 | 3 | 344576 / 344576 | 436205568 / 436201472 |
| 4 | 崩溃 | 738 | 6197 | 1475 | 739 | 3 | 越过 | 2 | 3 | 344576 / 344576 | 436291584 / 436287488 |
| 4 | 正常卸载 | 64 | 136 | 128 | 65 | 1 | 不越过 | 2 | 2 | 344576 / 344576 | 435462144 / 435462144 |
| 4 | 正常卸载 | 365 | 1007 | 730 | 366 | 1 | 不越过 | 2 | 2 | 344576 / 344576 | 435462144 / 435462144 |
| 4 | 正常卸载 | 369 | 2488 | 738 | 370 | 2 | 越过 | 2 | 2 | 344576 / 344576 | 435558400 / 435558400 |
| 4 | 正常卸载 | 738 | 4715 | 1476 | 739 | 3 | 越过 | 2 | 2 | 410112 / 344576 | 435718144 / 435652608 |
| 8 | 崩溃 | 64 | 6460 | 123 | 65 | 1 | 不越过 | 2 | 7 | 344576 / 344576 | 437692416 / 437690368 |
| 8 | 崩溃 | 365 | 7927 | 725 | 366 | 1 | 不越过 | 2 | 7 | 344576 / 344576 | 437692416 / 437688320 |
| 8 | 崩溃 | 369 | 9410 | 733 | 370 | 2 | 越过 | 2 | 7 | 344576 / 344576 | 437788672 / 437786624 |
| 8 | 崩溃 | 738 | 12369 | 1471 | 739 | 3 | 越过 | 2 | 7 | 410112 / 344576 | 437932032 / 437864448 |
| 8 | 正常卸载 | 64 | 6330 | 126 | 65 | 1 | 不越过 | 2 | 4 | 344576 / 344576 | 435480576 / 435480576 |
| 8 | 正常卸载 | 365 | 7195 | 728 | 366 | 1 | 不越过 | 2 | 4 | 344576 / 344576 | 435480576 / 435480576 |
| 8 | 正常卸载 | 369 | 8670 | 736 | 370 | 2 | 越过 | 2 | 4 | 344576 / 344576 | 435576832 / 435576832 |
| 8 | 正常卸载 | 738 | 10891 | 1474 | 739 | 3 | 越过 | 2 | 4 | 410112 / 344576 | 435736576 / 435671040 |
| 16 | 崩溃 | 64 | 12620 | 115 | 65 | 1 | 不越过 | 2 | 15 | 410112 / 344576 | 442494976 / 442429440 |
| 16 | 崩溃 | 365 | 14075 | 717 | 366 | 1 | 不越过 | 2 | 15 | 344576 / 344576 | 442429440 / 442427392 |
| 16 | 崩溃 | 369 | 15546 | 725 | 370 | 2 | 越过 | 2 | 15 | 410112 / 344576 | 442591232 / 442523648 |
| 16 | 崩溃 | 738 | 18493 | 1463 | 739 | 3 | 越过 | 2 | 15 | 344576 / 344576 | 442585088 / 442580992 |
| 16 | 正常卸载 | 64 | 12498 | 122 | 65 | 1 | 不越过 | 2 | 8 | 344576 / 344576 | 435517440 / 435517440 |
| 16 | 正常卸载 | 365 | 13351 | 724 | 366 | 1 | 不越过 | 2 | 8 | 344576 / 344576 | 435517440 / 435517440 |
| 16 | 正常卸载 | 369 | 14814 | 732 | 370 | 2 | 越过 | 2 | 8 | 344576 / 344576 | 435613696 / 435613696 |
| 16 | 正常卸载 | 738 | 17023 | 1470 | 739 | 3 | 越过 | 2 | 8 | 410112 / 344576 | 435773440 / 435707904 |

R5b 甲的判读（`name=h5_threshold_verdicts` 24 条，按 θ 与片数峰值归类的原样输出）：

```text
      6 threshold_rows=365 peak_chain_pages=1 r5b_reading_one_page=not_crossing
      6 threshold_rows=369 peak_chain_pages=2 r5b_reading_one_page=crossing
      6 threshold_rows=64 peak_chain_pages=1 r5b_reading_one_page=not_crossing
      6 threshold_rows=738 peak_chain_pages=3 r5b_reading_one_page=crossing
```

- θ = 64、365 的 12 格片数峰值都是 1，行数峰值 65 与 366 都不超过 366：不越过读法甲的上界；θ = 369、738 的 12 格片数峰值是 2 与 3，行数峰值 370 与 739：越过，每次可写挂载的切换预留每块盘多 4 × 32768 字节（X19）。六个几何点（S 三档 × 卸载方式两种）上同一个 θ 的 R5b 判定相同：`name=h5_geometry_sensitivity` 4 条 `r5b_stable_across_points=true`，判决行 `geometry_unstable_thresholds=none`。
- 行数峰值只由 θ 定（θ + 1），与 S、卸载方式无关；S 和卸载方式只改清扫之后剩几行（期末行数：正常卸载 S = 4 / 8 / 16 为 2 / 4 / 8，崩溃为 3 / 7 / 15）。按式子推，读法甲下 θ 的上界是 365（行数项 ≤ 一片 ⟺ rows0 ≤ 366，rows0 峰值 = θ + 1，登记 X17）；量到的是 365 不越过、369 越过，θ = 366 至 368 没量（登记的候选只有四档）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:450-508`（整段抄，未转述）**

````markdown
### 第三段读数：阳性对照 PC-不扫 与 PC-差分（V34、V35）

**PC-不扫（修订后的判据：清扫发布 0 次、rows0 逐次 +1、期末 rows0 = 400）过。** 「交给后台」对照一片，400 次挂载，逐次挂载 `sweep=handed_to_the_background` 400 条、`rows0` 等于第几次挂载。对照片的 `h5_config` 与 `h5_cell_summary` 整行（H5 产物第 18494、18895 行）：

```text
E7RESULT name=h5_config shard=h5-control-slots-per-region-8-handed-to-the-background-normal_unmount slots_per_region=8 threshold_rows=none unmount_kind=normal_unmount mounts_at_most=400 device_bytes=4294967296 journal_ring_bytes=805306368 binary_sha256=ac1799e786281c7d2f8335b0358eb9a47d8b27711df7704b50ed755a06f926e5
E7RESULT name=h5_cell_summary shard=h5-control-slots-per-region-8-handed-to-the-background-normal_unmount slots_per_region=8 threshold_rows=none unmount_kind=normal_unmount writable_mounts=400 sweeps_done=0 peak_rows0=400 mounts_with_rows0_above_the_threshold=0 mounts_with_rows0_above_366=34 peak_chain_pages=2 mounts_with_two_or_more_chain_pages=34 final_rows=400 total_write_bytes=757080576 sweep_write_bytes=none sweep_read_bytes=none sweep_mount_nanoseconds=none positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true final_rows0=400 registered_final_rows0=400 original_registered_final_rows0=401 final_rows0_equals_the_registered_value=true
```

对照格挂载次数 400 次里有 34 次 rows0 > 366、片数峰值 2，没有一次清扫：`sweeps_done=0`、`final_rows0=400`、`registered_final_rows0=400`（登记第十二节修订把原来的 401 改成 400）、`original_registered_final_rows0=401` 各印一次。V34 没触发，R5a、R5b 甲出数。

**PC-差分原臂：24 格全打中，V35 成立，记一次输。** 登记 5.3 原判据：没越过阈值的第一次挂载，阈值那次与「交给后台」那次两者的设备写字节之差、读字节之差都 = 0；越过的那一次写字节之差 > 0。按字面判，不放宽（登记第十二节修订，依据是派发提示）。产物里 24 格的 `h5_cell_summary` 的四个字段取值归类的原样输出：

```bash
grep 'name=h5_cell_summary' $P | grep -v 'shard=h5-repeat\|shard=h5-control' | grep -o 'positive_control_differential_below_the_threshold=[a-z]* positive_control_differential_at_the_first_sweep=[a-z]* differential_below_the_threshold_write_bytes=[0-9]* differential_below_the_threshold_read_bytes=[0-9]*' | sort | uniq -c
```

```text
     24 positive_control_differential_below_the_threshold=false positive_control_differential_at_the_first_sweep=true differential_below_the_threshold_write_bytes=0 differential_below_the_threshold_read_bytes=32768
```

写字节之差 0（没越过的一次）、越过的那次写字节之差 > 0，这两半 24 格都成立；读字节之差是 32768、不是 0，24 格全打中：原臂的「没越过的那一次读字节之差 = 0」在每一格上都不成立，`positive_control_differential_below_the_threshold=false` 24 格。V35 条款字面：那一格的 R5c、R5d 不出；24 格全打中，R5c、R5d 整体不出。「每一格上第一次没越过阈值的那一次挂载」就是第 1 次挂载（rows0 = 1），整行样例（S = 8、θ = 369、正常卸载，H5 产物第 7931、8302、8669 行）：

```text
E7RESULT name=h5_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=1 swept=false write_bytes_difference=0 read_bytes_difference=32768 mount_nanoseconds=3492017
E7RESULT name=h5_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=370 swept=true write_bytes_difference=344576 read_bytes_difference=435576832 mount_nanoseconds=92570354
E7RESULT name=h5_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=736 swept=true write_bytes_difference=344576 read_bytes_difference=435576832 mount_nanoseconds=76328517
```

**新臂 PC-差分-读链（修订补的一条臂，与原臂各自判）：24 格全过。** 判据：读字节之差 = 阈值那一次读实例表链的字节数 = 片数 × 32768，片数取锚点模型这一次的 rows0（rows0 = 1 ⇒ 32768）。`name=h5_chain_read_differential` 的 `read_bytes_difference_equals_the_chain_read` 取值归类的原样输出（不含首片重跑那一条，`grep -v 'shard=h5-repeat'`）：

```bash
grep 'name=h5_chain_read_differential' $P | grep -v 'shard=h5-repeat' | grep -o 'read_bytes_difference_equals_the_chain_read=[a-z]*' | sort | uniq -c
```

```text
     24 read_bytes_difference_equals_the_chain_read=true
```

整行样例（同一格，H5 产物第 7932 行）：

```text
E7RESULT name=h5_chain_read_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=1 read_bytes_difference=32768 model_rows0=1 expected_chain_pages=1 expected_chain_read_bytes=32768 read_bytes_difference_equals_the_chain_read=true
```

判决行 `chain_read_differential_failed_cells=0` 与 `differential_control_failed_cells=24` 分开报，不合成一个真假。

怎么读这两条（量到的与推的分开）：

- 量到的：没越过阈值的第 1 次挂载上，阈值那次比「交给后台」那次多读 32768 字节（一片实例表链），24 格逐格一样；这个数等于「一片 × 32768」，新臂 24/24 成立。越过阈值的 50 次清扫挂载（24 格各 2 次加首片重跑 2 次）里，写字节之差是 344576 的有 44 次、410112 的有 6 次（`grep 'name=h5_differential' $P | grep 'swept=true' | grep -o 'write_bytes_difference=[0-9]*' | sort | uniq -c`）：

```text
     44 write_bytes_difference=344576
      6 write_bytes_difference=410112
```

- 推的（登记修订里已写成推的，这一页的数没有证实它）：阈值那次挂载末尾沿实例表链数行数，所以读 32768，「交给后台」那次在开关那一判就返回、不读。新臂只证实了「差 = 链读的字节数」，没有分离出「谁读、谁不读」。
- 失败形态：原臂把「差 = 0」当作每格都成立的前提，而阈值版的挂载末尾要沿链数行数（上面推的那一条），这个前提对每一条臂（每个 θ）一起不成立，不分辨臂。`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 的第一行形态是「打中不分辨臂」，出路是把共用前提另立一笔账先修；这一页只按登记记一次输，不把原臂改成新臂的读法、不把读那一半当预期内。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:509-569`（整段抄，未转述）**

````markdown
### 第三段读数：R5c、R5d 不出，判别力自证，G5 与几何敏感性

**R5c、R5d 不出（V35，24 格全部）。** `name=h5_threshold_verdicts` 24 条的 `r5c=` 取值归类的原样输出，与整行样例（S = 8、θ = 64 与 θ = 369、正常卸载，H5 产物第 19041、19043 行）：

```bash
grep 'name=h5_threshold_verdicts' $P | grep -o 'r5c=[a-z_0-9]*' | sort | uniq -c
```

```text
     24 r5c=void_v35
E7RESULT name=h5_threshold_verdicts cell=h5-slots-per-region-8-threshold-64-normal_unmount slots_per_region=8 unmount_kind=normal_unmount threshold_rows=64 peak_chain_pages=1 r5b_reading_one_page=not_crossing writable_mounts=126 total_write_bytes=237512192 ratio_to_the_smallest_write_per_mount_permille=none r5c=void_v35
E7RESULT name=h5_threshold_verdicts cell=h5-slots-per-region-8-threshold-369-normal_unmount slots_per_region=8 unmount_kind=normal_unmount threshold_rows=369 peak_chain_pages=2 r5b_reading_one_page=crossing writable_mounts=736 total_write_bytes=1388854784 ratio_to_the_smallest_write_per_mount_permille=none r5c=void_v35
```

R5c 的写放大比值（`ratio_to_the_smallest_write_per_mount_permille`）24 格全是 `none`。产物里每格仍打了两次清扫的写字节与读字节（「第三段读数：行数轨迹（R5a）与片数峰值（R5b 甲）」一节的 24 格读数表）和整段历史的设备写字节，它们按 V35 不当 R5c、R5d 的结果。

**判别力自证（登记第八节 ②，R5c）做不出，V36。** 产物 `name=discrimination_self_proof` 行（H5 产物第 19065 行）：

```text
E7RESULT name=discrimination_self_proof judgement=h5_r5c_write_amplification real_ratios=0 moved_doubled_boundary_permille=none flipped=0 holds=false
```

`real_ratios=0`：R5c 没有一个真比值，没有「把 1.10 挪到两个相邻真比值之间」可挪，装置因此判 `holds=false`、判决行 `discrimination_self_proof_holds=false`。这是 V35 的后果，不是第二处装置缺陷：装置里这条判定函数 `acceptance_boundary_moved_between_real_ratios` 的单测（`moving_the_acceptance_boundary_between_two_real_ratios_flips_a_verdict`，喂 [1000, 1050, 1300] 必须挪得出）在主 agent 跑的 73 passed 读数里，是绿的；不绿的是真产物上没有比值可喂。按登记 V36：R5c 的几何敏感性不出。

**G5 既没触发也没清除。** 判决行 `g5_points_without_a_candidate=6` 与六条 `name=h5_point`（`thresholds_passing_both=none g5_no_candidate_passes_both=true`）：

```text
E7RESULT name=h5_point slots_per_region=4 unmount_kind=normal_unmount cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=4 unmount_kind=crash cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=8 unmount_kind=normal_unmount cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=8 unmount_kind=crash cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=16 unmount_kind=normal_unmount cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=16 unmount_kind=crash cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
```

登记 G5 的触发观测是「每个 θ 要么 R5b 判越过、要么 R5c 判不可接受」。θ = 369、738 R5b 越过；θ = 64、365 R5b 不越过，它们的 R5c 是 `void_v35`，不是「不可接受」，条件判不出。装置把「R5c 不出」的格当成「没过两关」数进 `thresholds_passing_both=none`，所以 6 = 点数（3 个 S × 2 种卸载方式），不是 G5 打中的观测。交用户时不许写「四个候选里没有一个同时满足」；只许写：θ = 369、738 越过一片的上界，θ = 64、365 不越过，写放大这一关在登记字面下没有读数。

**几何敏感性。** 4 条 `name=h5_geometry_sensitivity`（H5 产物第 19061 至 19064 行）：

```text
E7RESULT name=h5_geometry_sensitivity threshold_rows=64 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
E7RESULT name=h5_geometry_sensitivity threshold_rows=365 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
E7RESULT name=h5_geometry_sensitivity threshold_rows=369 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
E7RESULT name=h5_geometry_sensitivity threshold_rows=738 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
```

`r5b_stable_across_points=true` 四条是读数：六个几何点上 R5b 的判定相同。`r5c_stable_across_points=true` 四条是空真：六点上 R5c 都是同一个值 `void_v35`，装置按「取值集合只有一个元素」判稳，登记 V36 又让 R5c 的几何敏感性不出，这四条不当结果引。

**产物里能做的除法（不是 R5c 的结果，只供主 agent 判要不要补臂重登记）。** R5c 的第二个量 W_θ（登记 X21 ②）= 整段历史全部设备写字节 ÷ 可写挂载次数，来自 `h5_cell_summary` 的 `total_write_bytes` 与 `writable_mounts`，不经 PC-差分；V35 条款字面让整个 R5c 不出，这份除法不算 R5c，不套 1.10 的门槛下判定。`python3 -I -B scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_cell_table.py $P write-per-mount` 的原样输出：

```text
S=4 crash: θ=64:每次挂载写942657字节:相对最小1005千分之一 θ=365:每次挂载写937965字节:相对最小1000千分之一 θ=369:每次挂载写938204字节:相对最小1000千分之一 θ=738:每次挂载写970459字节:相对最小1035千分之一
S=4 normal_unmount: θ=64:每次挂载写1885052字节:相对最小1000千分之一 θ=365:每次挂载写1886850字节:相对最小1001千分之一 θ=369:每次挂载写1887032字节:相对最小1001千分之一 θ=738:每次挂载写1919989字节:相对最小1019千分之一
S=8 crash: θ=64:每次挂载写930903字节:相对最小1005千分之一 θ=365:每次挂载写926363字节:相对最小1000千分之一 θ=369:每次挂载写926552字节:相对最小1000千分之一 θ=738:每次挂载写958993字节:相对最小1035千分之一
S=8 normal_unmount: θ=64:每次挂载写1885017字节:相对最小1000千分之一 θ=365:每次挂载写1886849字节:相对最小1001千分之一 θ=369:每次挂载写1887031字节:相对最小1001千分之一 θ=738:每次挂载写1920033字节:相对最小1019千分之一
S=16 crash: θ=64:每次挂载写944836字节:相对最小1001千分之一 θ=365:每次挂载写947102字节:相对最小1004千分之一 θ=369:每次挂载写943719字节:相对最小1000千分之一 θ=738:每次挂载写976771字节:相对最小1035千分之一
S=16 normal_unmount: θ=64:每次挂载写1884945字节:相对最小1000千分之一 θ=365:每次挂载写1886847字节:相对最小1001千分之一 θ=369:每次挂载写1887030字节:相对最小1001千分之一 θ=738:每次挂载写1920122字节:相对最小1019千分之一
```

六个几何点上四个 θ 的每次挂载写字节，相对最小的比都在 1000 至 1035 千分之一之间，θ = 738 最大（1019 至 1035），θ = 64 至 369 之间不超过 1005。这个量混着两样东西：每次挂载自己的写（正常卸载每次约 1.9 MB，崩溃约 0.9 MB），和清扫的写（每次 344576 或 410112 字节，两次清扫分摊到几百次挂载上）；历史长度因 θ 不同，θ = 738 的链最长到 3 片，每次挂载写行要重写链，多写的部分推的来自这里，没分离过。它没有分离出「清扫发布」本身的写放大。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:570-596`（整段抄，未转述）**

````markdown
### 第三段：判决行逐字段点名

`grep -n 'name=verdict' $P` 的原样输出（H5 产物只有这一条判决行，`$P` 取 H5 产物）：

```text
19066:E7RESULT name=verdict item=h5_summary positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true control_final_rows0=400 control_final_rows0_equals_the_revised_registered_value=true differential_control_failed_cells=24 chain_read_differential_failed_cells=0 r5b_crossing_thresholds=369,738 g5_points_without_a_candidate=6 geometry_unstable_thresholds=none discrimination_self_proof_holds=false repeat_of_the_first_shard_equal_outside_timing=true repeat_first_differing_line=none
```

整行（第 19066 行）：

```text
E7RESULT name=verdict item=h5_summary positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true control_final_rows0=400 control_final_rows0_equals_the_revised_registered_value=true differential_control_failed_cells=24 chain_read_differential_failed_cells=0 r5b_crossing_thresholds=369,738 g5_points_without_a_candidate=6 geometry_unstable_thresholds=none discrimination_self_proof_holds=false repeat_of_the_first_shard_equal_outside_timing=true repeat_first_differing_line=none
```

没过的字段逐个点名；取值是 `true`、`zero` 的与名字不表示失败的计数不点名：

| 字段 | 产物行 | 取值 | 为什么是这个值 | 登记预期内吗 |
|---|---|---|---|---|
| `differential_control_failed_cells=24` | 19066 | 24 | 24 格的 PC-差分 原臂全打中：没越过阈值的第 1 次挂载上读字节之差是 32768 不是 0（「第三段读数：阳性对照 PC-不扫 与 PC-差分（V34、V35）」一节 `positive_control_differential_below_the_threshold=false` 24 格，`differential_below_the_threshold_read_bytes=32768`） | 登记修订里推过「读字节之差不为 0」，并写明打中就记一次输（V35，R5c、R5d 不出）；不是原登记的预期 |
| `discrimination_self_proof_holds=false` | 19066（`name=discrimination_self_proof` 行 `holds=false` 在 19065） | false | R5c 的真比值 0 个（`real_ratios=0`），没有比值可挪门槛 | V35 的后果，V36 成立：R5c 的几何敏感性不出 |
| `g5_points_without_a_candidate=6` | 19066（六条 `h5_point` 在 19035、19040、19045、19050、19055、19060） | 6 | 装置把 R5c 不出的格数成「没有候选」；六个几何点各有 θ = 64、365 的 R5b 不越过、R5c 不出 | 不是 G5 打中的观测，G5 既没触发也没清除（「第三段读数：R5c、R5d 不出，判别力自证，G5 与几何敏感性」一节） |
| `r5b_crossing_thresholds=369,738` | 19066 | 369,738 | 列表，不是失败标志：θ = 369、738 的片数峰值是 2 与 3，越过读法甲的一片上界 | 登记预期内（A34 钉的就是 2 与 3）；拿不准它算不算「没过」，所以点名 |

产物里判决行之外的、值像失败的字段：`positive_control_differential_below_the_threshold=false`（`h5_cell_summary` 行，24 格加首片重跑共 25 条，同一件事，判决行逐字段表 `differential_control_failed_cells=24` 那一行）；`r5c=void_v35`（24 条 `h5_threshold_verdicts`，第 19031 至 19059 行之间）；`holds=false`（第 19065 行，判决行逐字段表 `discrimination_self_proof_holds=false` 那一行）。`g5_no_candidate_passes_both=true`（6 条 `h5_point`）名字像失败、值是 true，来由见判决行逐字段表 `g5_points_without_a_candidate=6` 那一行。

其余判决字段：`positive_control_no_sweep_publish=true`、`positive_control_rows0_plus_one_each_mount=true`、`control_final_rows0_equals_the_revised_registered_value=true`、`geometry_unstable_thresholds=none`、`repeat_of_the_first_shard_equal_outside_timing=true`、`repeat_first_differing_line=none` 都是过的取值；`control_final_rows0=400` 是判据值（修订后）；`chain_read_differential_failed_cells=0`。产物没有 `not_run` 字段、没有空字段。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:597-611`（整段抄，未转述）**

```markdown
### 第三段：作废、失败、停机条款逐条

| 条款 | 状态 | 读数 |
|---|---|---|
| 起跑前提不成立（停机） | 过（被测树此刻读数在「第三段（H5）：跑法、产物与被测树」一节） | 第三段起法在 H5 那一节「起法」一条补记，不进跑前登记 |
| T4 确定性核 | 过 | 首片重跑在第二台、原片在本机，去掉挂钟等字段后 135 行逐字段相同；判决行 `repeat_of_the_first_shard_equal_outside_timing=true repeat_first_differing_line=none`；这一页另核一遍 `diff` 输出 `无差异` |
| T5 H5 轨迹与锚点不等 | 没触发 | K33 18852 次全 true，K34 与 A36 的 50 次全 true，18852 次轨迹全与模型一致，25 条模型行全与登记脚本输出一致，独立核对 24 格加对照 0 处不符 |
| V34 PC-不扫 | 没触发 | 对照一格 400 次挂载清扫 0 次、`final_rows0=400`（修订后判据） |
| V35 PC-差分 | **24 格全打中** | 原臂读字节之差 32768 ≠ 0；R5c、R5d 24 格都不出；新臂 PC-差分-读链 24 格全过，分开报 |
| V36 判别力自证 | **R5c 触发** | `real_ratios=0 holds=false`；R5b 的稳定性是读数，R5c 的稳定性是空真 |
| V37 读不到不当 0 | 没触发 | 产物没有空字段、没有 `not_run` |
| V30 PC-堆 | 不适用 | H5 的量是设备读写字节与行数，产物里没有堆量字段（`grep -c 'peak_over_baseline'` 输出 `0`） |
| G5 | 既没触发也没清除 | 见「第三段读数：R5c、R5d 不出，判别力自证，G5 与几何敏感性」一节 |
| G1–G4、G6–G8、T2、T3、C0 补丁命中不是 1 次或 A41 不符（停机）、V31–V33 | 不适用 | 属第一、二、四段 |

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:612-624`（整段抄，未转述）**

```markdown
### 岔路表（问题单 H5；登记第十一节 H5 的够判条件是 R5a 全部格、R5b 与 R5c 都出了判定）

`$P` 取 H5 产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out`，`$T` 取 `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs`。

| 问题单行 | 现状 | 还差什么 | 剩下的量能不能让它翻面 | 算出它的命令 |
|---|---|---|---|---|
| H5 · R5a 行数轨迹 | 已出：24 格逐次轨迹、行数峰值 θ + 1（65 / 366 / 370 / 739）、与锚点一致 | 无 | 登记里 H5 的格全跑完了，剩下的量为零，不会翻面 | `python3 -I -B $T/sweep_cell_table.py $P table`；`python3 -B -I $T/sweep_history_anchor_check.py $T/anchors_e183.py $P` |
| H5 · R5b 甲 | 已出：θ = 64、365 不越过，θ = 369、738 越过（片数峰值 2、3），六个几何点一致 | 无 | 不会翻面（确定性的量，首片重跑逐字段相同） | `grep 'name=h5_threshold_verdicts' $P \| grep -o 'threshold_rows=[0-9]* peak_chain_pages=[0-9]* r5b_reading_one_page=[a-z_]*' \| sort \| uniq -c` |
| H5 · R5c 写放大 | 不出：V35，24 格全打中 | R5c 的判定（登记字面下要补臂或改判据并重跑 H5 的 24 格，是新一轮登记，不在这一份里） | 不能：登记里 H5 的格已全跑，V35 是每格的观测，同一装置重跑同样打中 | `grep 'name=h5_cell_summary' $P \| grep -v 'shard=h5-repeat\|shard=h5-control' \| grep -o 'positive_control_differential_below_the_threshold=[a-z]*' \| sort \| uniq -c`；`grep 'name=h5_threshold_verdicts' $P \| grep -o 'r5c=[a-z_0-9]*' \| sort \| uniq -c` |
| H5 · R5d 清扫的读 | 不出（同 V35）；产物照打每次清扫读字节 435462144 至 442591232 | 同上 | 同上 | 同上 |
| H5 · 几何敏感性与判别力自证 | R5b 稳定；R5c 与自证不出（V36） | 同 R5c | 同 R5c | `grep 'name=h5_geometry_sensitivity\|name=discrimination_self_proof' $P` |
| H5 · 问题单整行 | 登记第十一节的条件下**不够判**（R5c 不出）。问题单原文的够判条件「至少三个候选阈值上各跑一段连挂到越过阈值的历史，报行数峰值与清扫发布的写字节数」：四个候选 θ × 六个几何点已跑，行数峰值与每次清扫的写字节都在产物里，但写字节取自被 V35 打中的差分 | 够不够判由主 agent 对着登记与问题单定；要写放大这一关出数，要新登记 | 登记里没有未跑的量；只有新登记加重跑能让 R5c 翻面 | 这张岔路表的 H5 各行 |

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:625-688`（整段抄，未转述）**

````markdown
### 第 2 次重跑（第二段 H2 第一次跑：经统一任务服务分片，X3 改读进程高水位）

重跑登记 `research/prompts/e183-r2-prereg.md`（sha256 `76257619fe9f7a1da89a433fe794ab345fc7a8ac3230f8e57e57e1fb6af3119a`；第十二节修订 12.1 至 12.3 都写在这一段产物之前）。它沿用的上一次登记与问题单已移出工作区，取法 `git show 59ce5a865^:research/prompts/e183-preregistration.md`（sha256 `c4609ca35a24704e5d4e52b2fe92503cda2d73efe18c39d503a3d5cd4e1a7f21`）、`git show 59ce5a865^:research/prompts/m3-close-cost-measurement-questions.md`（sha256 `c5dec701293cb41b2f3a7b291b8e85a4e3f5e837c60024415f21c3023e1ba2d5`）。这一段只答问题单的 H2 一行：C567（分配与发布的代价随盘容量线性） 的改前基线，一次分配、一次发布的耗时与峰值内存对盘容量、对记录数的斜率。只有一条臂（今天的代码，候选丁）；9 格（容量阶梯 4 GiB、64 GiB、1 TiB、4 TiB、16 TiB − 1 MiB，n = 0；记录格 4 GiB 的 n = 1024、10240；占用格 64 GiB、1 TiB 的 n = 10240）× 5 轮，一格一轮一个进程，盘是文件后端的稀疏镜像（两块等大、L = 768 MiB、S = 8）。

跑法与前提（都是这一段现查的）：

- 起跑前提五样成立：上一次登记与问题单按取法取得到、`sha256sum` 与登记相同（819 行、13 行）；C567 那一行在 `.claude/kb/checks-owed.md` 第 345 行，在第 613 行「已还清」之前，仍开着；`crates/singlefs-core/src/allocator.rs` 里 `pub fn lowest_user_data_slot(`（第 955 行）与 `fn make_room_for_record_on_device(`（第 1144 行）各 `grep -c` 为 1，`crates/singlefs-core/src/transaction.rs` 里 `let allocator_before_this_publish = allocator.clone();` 为 2 处；跑法脚本 `grep -c 'run-with-memory-cap\|capped\.sh\|nice -n'` 为 0、装置 `grep -c 'WRAPPER_EXIT'` 为 0；`.claude/gate.d/stage-inputs.tsv` 第 47 行有键 `E183/h2`；未定项两项在登记 12.1 修-1、修-2。`allocator.rs` 这一批被别的补丁改过（`git diff HEAD --stat` 151 行增删），改动的 hunk 在 `PlacementOnDevice`、根环占用与 `PoolAllocator` 几处，两个锚点函数各仍恰好一处，按登记这一趟不作废。
- 被测树：HEAD `59ce5a865`。开跑那一刻的 `git diff HEAD -- crates/ | sha256sum` 没记下来（12.1 末尾要的那一样，跑的那一段没写进登记；产物跑过之后不改登记），读产物这一天现取为 `c9e492c4a6c026fa44773dd865c5f380776dab6049e0109b9542a60687277595`，`--stat` 末行「339 files changed, 89908 insertions(+), 16735 deletions(-)」。`find crates/singlefs-core crates/singlefs-format -name '*.rs' -newer <运行目录里的程序文件>` 为 0 个，被测的这两个包在程序文件编出之后没动过；装置源文件此刻 sha256 `eff3e9dd41ff88f187e2b011b27e1301e438827c35d06debfdacafe1bd0b2a87`，也不比程序文件新。
- 跑法：主 agent 交一条 `task-server/task submit -- bash scripts/experiment-apparatus/e183-capacity-cells.sh run`（任务 5382，结局 `type=succeeded exit=0`），服务拆成计划、5 片、合并。计划（子任务 5383）三次尝试：本机 4096M、6144M 两次退 250（撞上限），第三次在第二台 9216M 退 0，问准入放行、编出程序文件 sha256 `9016bb927d45ec09730735c405753921de6bf7f6075c3fde64fd2030224729e9`（235792472 字节）。第 1 轮那一片（子任务 5429）前五次在本机从 4096M 抬到 20736M 都退 250，一格都没做完（每次 3 至 13 秒），第六次在第二台 31104M 做完 9 格；其余四片各一次做完。轮与台：第 1、2、4 轮第二台，第 3、5 轮本机；第 3、5 轮在本机起的是第二台编的那一份程序文件，`h2 list-shards` 守卫过了，登记未定项第 2 项「一份二进制两台都起得来」这一次量到成立。合并（子任务 5470）在第二台跑（登记 5.3 写合并在本机；服务不钉台，合并只读运行目录里的片文件，判定不受台的影响）。
- 挂钟：五片做完的那一次尝试服务记的用时 121.9、119.7、135.9、119.7、137.8 秒；登记第十四节推的一片 30 至 50 分钟、占用格 1 TiB n = 10240 一格 17 至 34 分钟（Q14）都远大于实测（服务日志里那一格 5 至 7 秒）。
- 产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out`：949 行，sha256 `c07d13a209d8a4c178d52d16000cba1f63599b06de9d0d0750f17c8460088bd7`，第 1 行是准入头 `E7INPUT name=input_fingerprint key=E183/h2 sha256=dd3b75e7b8db0f070ee139ff62ea0b0fd9a3b5e9a7dc1f638351f3dadaad1dea units=4476`，末行 `E7RESULT name=done emitted=948`，产物里 `name=stop` 0 行。
- 运行目录 `<工作根>/e183-h2-run/dd3b75e7b8db0f07/` 里另留着一个 1.2G 的 `target/`：计划前两次在本机编译时撞上限被杀，跑法脚本「自己建的编译目录编完就删」那一步没走到；不影响产物（程序文件是第三次在第二台编的那一份）。

复跑（第二段 H2）：

```bash
# 仓根；重型（真造 4 GiB 至 16 TiB − 1 MiB 的稀疏镜像，单格进程高水位到约 22 GiB），由主 agent 交服务；
# 服务照脚本文件头的 task-server-shards 声明拆成计划、5 片、合并。准入键 E183/h2 没变时计划那一步退 77（已有同键产物）
task-server/task submit -- bash scripts/experiment-apparatus/e183-capacity-cells.sh run

# 列各轮各格做没做完；只合并运行目录里已有的片（不经服务拆）
bash scripts/experiment-apparatus/e183-capacity-cells.sh --list-items
bash scripts/experiment-apparatus/e183-capacity-cells.sh --merge-only --run-directory <运行目录>
```

这一趟的运行目录（片文件与各格日志，连同上面那个 `target/` 共 1.5G）照登记 5.3「跑完之后」在产物落盘、这一页写完之后删；删了之后 `--merge-only` 对这一趟无对象，要复核只能整趟重跑。

第二段不进 `scripts/experiment-apparatus/replay.sh` 的复跑表（登记 5.6 第 2 条：一趟经服务分 5 片、两台都可能派到，复跑表一条任务里整份跑装不下；产物带挂钟字段，逐字节比不了），`.claude/gate.d/replay-comparison-units.tsv` 不加行（登记第十一节「比对对象」：第二段没有上一次产物可比）。

产物里每一类结果行的条数（`awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out | sort | uniq -c` 的原样输出）：

```text
      1 name=config
      2 name=discrimination_self_proof
      1 name=done
    225 name=h2_allocate
     45 name=h2_allocator_copy
      9 name=h2_cell_determinism
      9 name=h2_cell_values
     18 name=h2_cell_values_by_host
     13 name=h2_class
     45 name=h2_config
      1 name=h2_largest_disk
      2 name=h2_occupancy_sensitivity
     45 name=h2_payload_constant
     45 name=h2_process
    225 name=h2_publish
      5 name=h2_round_hosts
      5 name=h2_service_attempt
     45 name=h2_shard_status
     65 name=h2_slope
     45 name=h2_steps
      1 name=h2_threshold_dependence
      3 name=h2_threshold_sensitivity
      1 name=input_fingerprint
     45 name=positive_control_flat
      1 name=positive_control_flat_slope
     45 name=positive_control_heap
      1 name=positive_control_linear
      1 name=verdict
```

登记的「臂 × 格 × 轮」数一遍：一条臂 × 9 格 × 5 轮 = 45 个进程，每个进程一条 `h2_config`、`h2_steps`、`h2_process`、`h2_shard_status`、`h2_allocator_copy`、`h2_payload_constant`、`positive_control_heap`、`positive_control_flat`，各 45 条；每个进程 5 次分配、5 次发布，各 225 条；13 行斜率 × 5 轮 = 65 条 `h2_slope`；9 格各一条 `h2_cell_values` 与 `h2_cell_determinism`，按台各一条 `h2_cell_values_by_host`（9 × 2 台 = 18）；5 片各一条 `h2_service_attempt`、5 轮各一条 `h2_round_hosts`。条数对得上，没有缺行；没有一格一轮缺 `h2_process` 的高水位（产物里 `=none` 只出在 R2e 发布两行的斜率、确定性核的 `differing_fields`、最大一档的空类与判别力自证 ④ 的 `largest_just_beyond_mebibytes` 上，都是登记预期内的空）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:689-710`（整段抄，未转述）**

````markdown
### 第二段：判决（整行抄自 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out` 第 849、853、864、943、944、945、946、947、948 行）

```text
E7RESULT name=h2_largest_disk threshold_bytes=25769803776 lower_edge_bytes=25702694912 completed_capacities_mebibytes=4096,65536,1048576,4194304,16777215 completed_count=5 largest_completed_mebibytes=16777215 largest_is_the_ladder_endpoint=true beyond_the_memory_threshold_mebibytes=none undeterminable_mebibytes=none filesystem_refused_mebibytes=none near_the_threshold_at_or_above_the_largest=none fewer_than_four_capacities=false
E7RESULT name=h2_threshold_dependence largest_at_half_mebibytes=4194304 largest_at_registered_mebibytes=16777215 largest_at_double_mebibytes=16777215 depends_on_the_threshold=true switches_between=half_and_registered
E7RESULT name=discrimination_self_proof judgement=h2_largest_disk capacity_mebibytes=4096 just_within_threshold_bytes=138219520 largest_just_within_mebibytes=4096 largest_just_beyond_mebibytes=none synthetic_high_water_marks_fall_on_two_sides=true used=real_high_water_marks holds=true
E7RESULT name=positive_control_linear copy_bytes_slope_at_least_0_8_every_round=true copy_time_slope_at_least_0_8_every_round=true
E7RESULT name=positive_control_flat_slope slope_below_0_2_every_round=true
E7RESULT name=h2_occupancy_sensitivity row=r2a near_empty_class=sublinear occupied_class=basically_unrelated agreement=unstable_depends_on_occupancy
E7RESULT name=h2_occupancy_sensitivity row=r2b near_empty_class=linear occupied_class=x5_as_registered_cannot_be_made agreement=not_judged_x5_as_registered_cannot_be_made
E7RESULT name=discrimination_self_proof judgement=h2_slope_class real_slopes=35 moved_boundary=0.812 flipped=1 synthetic_slopes_fall_in_two_classes=true used=real_slopes holds=true
E7RESULT name=verdict item=h2_summary r2a_class=sublinear r2b_class=linear r2d_allocate_class=basically_unrelated r2d_publish_class=linear r2e_allocate_class=sublinear r2e_publish_class=x5_as_registered_cannot_be_made r2e_allocate_peak_class=basically_unrelated r2e_publish_peak_class=x5_as_registered_cannot_be_made r2c_class=linear completed_capacities=5 fewer_than_four_capacities=false positive_control_linear_copy_bytes_holds=true positive_control_linear_copy_time_holds=true positive_control_flat_holds=true heap_control_failed_shards=0 unstable_slope_rows=none predictions_contradicted=r2a_not_basically_unrelated,r2e_allocate_not_linear discrimination_self_proof_holds=true hidden_state_cells=0 cell_rounds_within_the_threshold=45 cell_rounds_beyond_the_memory_threshold=0 cell_rounds_undeterminable=0 cell_rounds_filesystem_refused=0 cell_rounds_voided_high_water_mark=0 cell_rounds_near_the_threshold=0 ladder_rounds_across_hosts=0 unit_area_slots_differing_cell_rounds=0 largest_depends_on_the_threshold=true threshold_discrimination_self_proof_holds=true
```

判决行逐字段点名（`grep -n 'name=verdict' <产物>` 只有第 948 行一条）：

- `fewer_than_four_capacities=false`（第 948 行，第 849 行同名字段同值）：取值是布尔 false，意思是「跑完的容量档不少于四」不成立的那一面，即 G6 没触发；登记预期内。
- `r2e_publish_class=x5_as_registered_cannot_be_made`、`r2e_publish_peak_class=x5_as_registered_cannot_be_made`（第 948 行）：不是 `not_run`，拿不准算不算「没过」，点名：登记的 X5（覆盖写第 0 个数据单元以内的 4096 字节）在有 n 个单元的文件上造不出，上一次登记第十二节 2026-10-19 那条修订定了这两行宁缺不出，所以是登记预期内的不出类，不是 0、不是「基本无关」。
- `predictions_contradicted=r2a_not_basically_unrelated,r2e_allocate_not_linear`（第 948 行）：不是布尔也不是计数，拿不准，点名：上一次登记第四节 Q8 推的「R2a 基本无关、R2e 分配随记录数线性」被量到的类推翻，G2 触发（见下面条款逐条）；条款里预先写了这一种观测，不是装置的错。
- 其余计数（`heap_control_failed_shards`、`hidden_state_cells`、`cell_rounds_beyond_the_memory_threshold`、`cell_rounds_undeterminable`、`cell_rounds_filesystem_refused`、`cell_rounds_voided_high_water_mark`、`cell_rounds_near_the_threshold`、`ladder_rounds_across_hosts`、`unit_area_slots_differing_cell_rounds`）都是 0；判决行里没有 `not_run`。
- 判别力自证 ④（第 864 行）用的是真高水位：把下沿挪到 4 GiB 那一格 5 轮高水位的最大值（`138219520 − 67108864 = 71110656` 字节）上，最大一档是 4 GiB，再挪低 1 字节就一档都没有，换了一档；另有阈值敏感性（第 853 行）在 T ÷ 2 与 T 之间真换档（4 TiB 与 16 TiB − 1 MiB）。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:711-735`（整段抄，未转述）**

```markdown
### 第二段读数：斜率的类（R2a、R2b、R2c、R2d、R2e）与阳性对照

每轮的 b 取自产物第 865 至 941 行（`grep -n 'name=h2_slope' <产物>`），类取自第 870 至 942 行的 `name=h2_class`；类的门槛同上一次登记第六节（b ≥ 0.8 线性、0.2 ≤ b < 0.8 次线性、b < 0.2 基本无关，5 轮同一类才写那一类）。

| 行 | 量 | 横轴 | 第 1 至 5 轮的 b | 类 |
|---|---|---|---|---|
| R2a | 一次分配的耗时 | 容量（五档） | 0.284、0.258、0.322、0.309、0.251 | 次线性 |
| R2b | 一次发布的耗时 | 容量 | 0.813、0.811、0.875、0.812、0.830 | 线性 |
| R2d 分配 | 一次分配的峰值多占 | 容量 | 0.000（5 轮） | 基本无关（每次 112 字节） |
| R2d 发布 | 一次发布的峰值多占 | 容量 | 1.022（5 轮） | 线性 |
| R2c | 分配器拷贝字节 | 容量 | 1.023（5 轮） | 线性（照报、不算判据） |
| R2e 分配 | 一次分配的耗时 | 记录数（三档） | 0.591、0.523、0.591、0.646、0.475 | 次线性 |
| R2e 分配峰值 | 一次分配的峰值多占 | 记录数 | −0.000（5 轮） | 基本无关 |
| R2e 发布、R2e 发布峰值 | — | 记录数 | 不出（`points=0`） | `x5_as_registered_cannot_be_made` |
| PC-线性（拷贝耗时） | 分配器拷贝耗时 | 容量 | 1.306、1.306、1.330、1.311、1.333 | 线性，PC-线性 两半都过 |
| PC-平 | 1 MiB 丢弃分配的峰值多占 | 容量 | 0.000（5 轮） | 基本无关，PC-平 过 |
| 几何敏感性 R2a（占用格） | 一次分配的耗时 | 容量（4 GiB、64 GiB、1 TiB，n = 10240） | 0.039、0.029、0.060、0.035、0.103 | 基本无关；与近空盘的次线性不同，记「不稳定（依赖占用）」 |
| 几何敏感性 R2b（占用格） | — | — | 不出 | `x5_as_registered_cannot_be_made` |

贴着门槛的：R2b 五轮里有三轮的 b 在 0.811 至 0.813，离线性的下沿 0.8 不到 0.02；R2a 五轮在 0.251 至 0.322，离 0.2 最近的一轮差 0.051。类按门槛照判，没有一行跨门槛（`unstable_slope_rows=none`）。

R2c 的每盘每槽字节（`name=h2_allocator_copy` 的 `bytes_per_slot_per_device_millionths`，`grep -o 'name=h2_allocator_copy .*millionths=[0-9]*' <产物> | grep -o 'millionths=.*' | sort | uniq -c` 每个值 5 轮相同）：阶梯五档 3.634529、3.625487、3.625033、3.625008、3.625002，都在 A37 的 [3.5, 4.0] 里；记录格与占用格 4 GiB n = 1024 为 3.759416，4 GiB n = 10240 为 4.822029（在区间外），64 GiB n = 10240 为 3.686226，1 TiB n = 10240 为 3.628789。A37 推的 3.625 是位图与段计数，没算记录表；记录多的小盘上多出来的那一份推的是记录表（推的，没拆开量）。

一次分配交回的落点离单元区起点的槽数（`name=h2_allocate` 的 `slots_from_unit_area_start`，附带，解释 R2a）：9 格 225 次全是 2，包括 n = 10240 的三格。上一次登记 8.2 推的「记录格 n = 10240 那一状态首个空槽离起点约 2 万槽」没有成立，占用格与近空盘的落点相同；占用格上分配耗时的差来自记录数（4 GiB 上 94、2264、21034 条记录时分配中位数 640、1810、12260 纳秒，`name=h2_cell_values` 第 815、835、839 行），不是首个空槽离起点更远。所以几何敏感性那一行「依赖占用」的读法要按「依赖记录数」理解。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:736-763`（整段抄，未转述）**

```markdown
### 第二段读数：绝对值（R2f，按台）

`name=h2_cell_values_by_host`（第 813 至 846 行）与 `name=h2_cell_values`（标「跨台」，第 815 至 847 行）；本机 2 轮（第 3、5 轮）、第二台 3 轮（第 1、2、4 轮），两台都不足 5 轮，下表的中位数不叫「5 轮中位数」（G11）。耗时单位纳秒，堆单位字节。

| 格（容量 / 单元数） | 台（轮数） | 分配中位数（最小 至 最大） | 发布中位数（最小 至 最大） | 发布峰值多占中位数 | 分配器拷贝字节 |
|---|---|---|---|---|---|
| 4 GiB / 0 | 本机（2） | 740（380 至 3060） | 5085880（4687854 至 7278243） | 3111920 | 1540808 |
| 4 GiB / 0 | 第二台（3） | 460（320 至 2530） | 6220286（6139425 至 6286766） | 3111920 | 1540808 |
| 4 GiB / 0 | 跨台（5） | 640（320 至 3060） | 6219807（4687854 至 7278243） | 3111920 | 1540808 |
| 64 GiB / 0 | 本机（2） | 1690（1440 至 5100） | 16512061（14824776 至 20534581） | 60128240 | 30048968 |
| 64 GiB / 0 | 第二台（3） | 2060（1370 至 5660） | 15576916（15373824 至 19886910） | 60128240 | 30048968 |
| 64 GiB / 0 | 跨台（5） | 1880（1370 至 5660） | 15577281（14824776 至 20534581） | 60128240 | 30048968 |
| 1 TiB / 0 | 本机（2） | 4470（3420 至 7770） | 357879878（353842369 至 475754672） | 972392556 | 486180008 |
| 1 TiB / 0 | 第二台（3） | 3810（2540 至 9510） | 291380102（287646092 至 394520740） | 972392556 | 486180008 |
| 1 TiB / 0 | 跨台（5） | 4080（2540 至 9510） | 353842369（287646092 至 475754672） | 972392556 | 486180008 |
| 4 TiB / 0 | 本机（2） | 5070（3850 至 7730） | 1457141942（1445782358 至 2023847045） | 3891628140 | 1945797800 |
| 4 TiB / 0 | 第二台（3） | 4040（2880 至 6720） | 1122337793（1115925036 至 1541548423） | 3891628140 | 1945797800 |
| 4 TiB / 0 | 跨台（5） | 4600（2880 至 7730） | 1445782358（1115925036 至 2023847045） | 3891628140 | 1945797800 |
| 16 TiB − 1 MiB / 0 | 本机（2） | 6760（5570 至 7390） | 4896734085（4796853448 至 6293868215） | 15568569548 | 7784268504 |
| 16 TiB − 1 MiB / 0 | 第二台（3） | 5821（4390 至 7230） | 3962762545（3895826976 至 5208832836） | 15568569548 | 7784268504 |
| 16 TiB − 1 MiB / 0 | 跨台（5） | 6190（4390 至 7390） | 4796853448（3895826976 至 6293868215） | 15568569548 | 7784268504 |
| 4 GiB / 1024 | 跨台（5） | 1810（1610 至 5950） | 6748361（5216130 至 17213996），不是 X5 | 3418328 | 1593752 |
| 4 GiB / 10240 | 跨台（5） | 12260（11701 至 16440） | 11428831（9906053 至 364095938），不是 X5 | 5941756 | 2044232 |
| 64 GiB / 10240 | 跨台（5） | 14730（13290 至 31840） | 15503175（14935119 至 371069809），不是 X5 | 62958076 | 30552392 |
| 1 TiB / 10240 | 跨台（5） | 15540（13100 至 35120） | 80801087（75143331 至 381817553），不是 X5 | 975223160 | 486683720 |

记录格、占用格按台的那两行在产物第 833 至 846 行，这里只列跨台。读法（量的数，商是算的）：16 TiB − 1 MiB 上一次发布多占 15568569548 字节，是分配器一份拷贝 7784268504 字节的 2 倍再多 32540 字节；阶梯五档上发布多占是拷贝字节的 2.0197、2.0010、2.0001、2.0000、2.0000 倍。堆量（分配器拷贝、峰值多占）在同一格里两台逐字段相同，第二台的发布耗时中位数在 1 TiB、4 TiB、16 TiB − 1 MiB 三档比本机少 18.6%、23.0%、19.1%。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:764-801`（整段抄，未转述）**

````markdown
### 第二段读数：进程高水位与「本机能造的最大稀疏盘」（R2g、X22）

阈值 T = 25769803776 字节（24 GiB），下沿 25702694912。45 格轮全是「阈值内跑完」（`name=h2_shard_status` 45 行 `status=within_the_threshold class=within_the_threshold`，`near_the_threshold=false`，`grep 'name=h2_shard_status ' <产物> | grep -o 'status=[^ ]* class=[^ ]*' | sort | uniq -c` 输出一行 `45 status=within_the_threshold class=within_the_threshold`）。每格 5 轮的 H（`vm_high_water_mark_kibibytes` × 1024，`name=h2_process`）：

| 格（容量 / 单元数） | H 最小 | H 最大 |
|---|---|---|
| 4 GiB / 0 | 70328320 | 71110656 |
| 4 GiB / 1024 | 292634624 | 292831232 |
| 4 GiB / 10240 | 2798190592 | 2799054848 |
| 64 GiB / 0 | 101654528 | 102383616 |
| 64 GiB / 10240 | 2858577920 | 2859716608 |
| 1 TiB / 0 | 1490620416 | 1491107840 |
| 1 TiB / 10240 | 3783000064 | 3783700480 |
| 4 TiB / 0 | 5944557568 | 5945462784 |
| 16 TiB − 1 MiB / 0 | 23627563008 | 23629737984 |

16 TiB − 1 MiB 那一档 5 轮的状态行（第 772、781、790、799、808 行）里最大的一条：

```text
E7RESULT name=h2_shard_status cell=h2-device-16777215-mebibytes-units-0 round=4 status=within_the_threshold class=within_the_threshold high_water_mark_bytes=23629737984 high_water_mark_before_kibibytes=4212 high_water_mark_after_kibibytes=68964 near_the_threshold=false host=第二台 image_filesystem=ext2/ext3 service_attempt=3/5:1 unit_area_slots=1073691584 registered_unit_area_slots=1073691584
```

离下沿差 2072956928 字节（约 1.93 GiB，算的），「贴着阈值」的界是程序文件字节数加 64 MiB = 302901336，不贴。**本机能造的最大稀疏盘**：在阈值 T 内、在这一次派到的两台上整格跑完的最大一档是阶梯端点 16 TiB − 1 MiB（登记 X3「落在阶梯端点照写「≤ 端点」」：阶梯外没量，比它大的盘在阈值内跑不跑得完不知道）。阈值敏感性（第 850 至 853 行）：T ÷ 2 时最大一档是 4 TiB（16 TiB − 1 MiB 的 H 过了 T ÷ 2 的下沿），2T 时同 T，`depends_on_the_threshold=true`，在 T ÷ 2 与 T 之间换档。

PC-高水位（V40）：45 个进程 PC-堆 前后的高水位差 × 1024 在 66109440 至 66961408 字节，都 ≥ 62914560（`positive_control_heap` 45 行 `high_water_mark_control_holds=true`）。PC-堆（V30）45 行 `peak_over_baseline_bytes=67108864 holds=true`。

外包内（R2h、X27）：服务记的那一次尝试的峰值 P(a) 取 `task-server/task status 5382 --json` 每个子任务每次尝试的 `memory_peak`；装置那一边取第 854 至 858 行的 `service_peak_needed_at_least_bytes`（= H 最大(a) − 程序文件字节数 − 64 MiB）。

| 片 / 尝试 | 台 | P(a) | 要求不小于 | 成立 |
|---|---|---|---|---|
| 0/5 第 6 次 | 第二台 | 25304449024 | 23325726632 | 成立 |
| 1/5 第 1 次 | 第二台 | 25298878464 | 23326214056 | 成立 |
| 2/5 第 1 次 | 本机 | 25304514560 | 23325476776 | 成立 |
| 3/5 第 1 次 | 第二台 | 25298677760 | 23326836648 | 成立 |
| 4/5 第 1 次 | 本机 | 25303404544 | 23324661672 | 成立 |

第 1 轮那一片前五次尝试（本机，上限 4096M 至 20736M，退 250）一格都没做完，H 最大(a) 无对象。服务记的峰值（约 23.57 GiB）比进程高水位最大值（约 22.01 GiB）高约 1.5 GiB，与登记 Q13 推的页缓存量级相符（推的，没拆开量）；按 X22 服务峰值不进判类。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:802-825`（整段抄，未转述）**

```markdown
### 第二段：作废、失败、停机条款逐条

| 条款 | 观测 | 判 |
|---|---|---|
| T7 起跑前提 | 五样都成立（上面「跑法与前提」） | 不停 |
| T8 服务那一侧的跑法前提 | 没有一个子任务退 78；子任务的退出码只有 250（撞上限，服务放大重排）与 0 | 不停 |
| T9 几何与 A43 | `unit_area_slots_differing_cell_rounds=0`；45 行状态行的 `unit_area_slots` 都等于 `registered_unit_area_slots` | 不停 |
| T4 确定性的量两轮不同（另罩跨台） | 9 格 `name=h2_cell_determinism` 都是 `rounds_compared=5 equal_across_rounds=true differing_fields=none`，5 轮跨两台 | 不停 |
| T2 本地常量回比 | 产物里 45 行 `name=h2_payload_constant` 都是 `equal=true`；装置回比单测这一段没跑、读数不在这一页 | 产物这一侧不停；单测那一侧没核 |
| V30 PC-堆 | 45 行都过 | 不作废 |
| V32 PC-线性、PC-平 | 第 943、944 行都 true | 不作废 |
| V37 读不到 ≠ 0 | 每格每轮的量都在；`=none` 只出在登记预期的地方 | 不作废 |
| V38 外包内 | 5 次尝试都成立 | 不作废 |
| V40 PC-高水位 | 45 个进程都过 | 不作废 |
| G1 某行 5 轮不同类 | `unstable_slope_rows=none` | 不触发 |
| G2 第四节 Q8 推错 | R2a 是次线性（推的是基本无关）、R2e 分配是次线性（推的是线性）；R2c 的 b = 1.023 ≥ 0.8，阶梯五档的每盘每槽字节在 A37 区间里；记录格 4 GiB n = 10240 的 4.822029 在 A37 区间外（A37 写在 R2c「每档」那一行下，这一格是不是 A37 的对象拿不准，照列） | **触发**：推的与量出的如实并列（上一节表），正文按量出的写 |
| G6 跑完的容量档少于四 | 5 档 | 不触发 |
| G9 最大一档靠着阈值 | `near_the_threshold_at_or_above_the_largest=none` | 不触发 |
| G10 一轮跨台 | 5 轮 `name=h2_round_hosts` 都是 `one_host=true` | 不触发 |
| G11 挂钟不再单机（预先记的一次输） | 5 轮的台不全相同（第 1、2、4 轮第二台，第 3、5 轮本机）；各片日志里每格起跑、收尾各一次 `/proc/loadavg`，45 格轮里 32 格轮至少一次第一列 ≥ 2.0（范围 1.35 至 28.20；两次都低于 2.0 的 13 格轮，第 2 轮 7 格、第 4 轮 6 格；第 1、3、5 轮 9 格全部 ≥ 2.0） | **触发**：R2f 按台报（上一节），跨台的不叫「单机 5 轮中位数」；负载高的格照判类（X25 的理由：一轮之内同一台，只改截距） |
| G12 判不出 | 没有服务结局文件（没有子任务记 `memory-cap-at-maximum`） | 不触发 |

G11 的负载读数取自各片的服务日志（`task-server/task log 5429` 至 `5433 --all` 里以「格 」起头、带「负载」的行，90 行），不进产物；这一页不贴各行原样（行里带纪元秒），数法是对每格轮取起跑、收尾两次的第一列，任一次 ≥ 2.0 记一格轮。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:826-838`（整段抄，未转述）**

```markdown
### 岔路表（问题单 H2；登记第十一节 H2 的够判条件是 R2a、R2b、R2d、R2e 都出了类或「不稳定」且跑完的容量档 ≥ 4）

`$P` 取 H2 产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out`。

| 问题单行 | 现状 | 还差什么 | 剩下的量能不能让它翻面 | 算出它的命令 |
|---|---|---|---|---|
| H2 · R2a 分配耗时对容量 | 已出：次线性（5 轮 b 0.251 至 0.322） | 无 | 登记里第二段的格全跑完、没有跳过的格，剩下的量为零 | `grep -n 'name=h2_slope row=r2a \|name=h2_class row=r2a ' $P` |
| H2 · R2b 发布耗时对容量 | 已出：线性（5 轮 b 0.811 至 0.875） | 无 | 同上 | `grep -n 'name=h2_slope row=r2b \|name=h2_class row=r2b ' $P` |
| H2 · R2d 峰值多占对容量 | 已出：分配基本无关，发布线性（b 1.022） | 无 | 同上（确定性的量，5 轮跨两台逐字段相同） | `grep -n 'name=h2_class row=r2d_' $P` |
| H2 · R2e 对记录数 | 分配两行已出：耗时次线性、峰值基本无关；发布两行不出类（`x5_as_registered_cannot_be_made`） | 发布两行的类：登记的 X5 在有 n 个单元的文件上造不出（上一次登记第十二节 2026-10-19 修订），要出类得另立读法、是新登记 | 不能：登记里没有剩下的量能补这两行 | `grep -n 'name=h2_class row=r2e' $P` |
| H2 · 跑完的容量档 | 5 档（≥ 4），最大一档 16 TiB − 1 MiB（阶梯端点） | 无 | 不能 | `grep -n 'name=h2_largest_disk' $P` |
| H2 · 问题单整行 | 登记第十一节字面：R2a、R2b、R2d 出了类，容量档 5，R2e 只出了分配两行，**字面不够判**（差 R2e 发布两行）。问题单原文的够判条件「至少四档盘容量（含本机能造的最大稀疏盘与默认几何的盘）各量一次分配与一次发布的挂钟与峰值内存，记录数另扫三档」：五档容量各量了 5 轮分配、发布的耗时与峰值多占，记录数扫了三档（94、2264、21034 条），其中三档上的发布不是登记的 X5 | 够不够判由主 agent 对着登记与问题单定 | 登记里没有未跑的量；R2e 发布两行只有新登记能补 | 这张岔路表的 H2 各行 |

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:839-851`（整段抄，未转述）**

```markdown
### 它答不了的

- 空池状态下的最小盘：V33 打中，空池挂载在 C_挂空 − 1 MiB 上没走到（mkfs 先败），这个最小盘归 mkfs 的几何，R4a–R4c 的空池一半不出；产物里空池读数（设备 0 的行在第 99、103、176、180、268、272、348、352 行，设备 1 的行紧随其后）只留档。
- PC-准入-文件 只证明有文件状态下 C_挂文件 − 1 MiB 上「挂载做成、推满仍不够」，即最小盘被准入式子顶住；它不分是六项里哪一项把盘顶满（那是 R4c 的事），也不证明 1 MiB 以内的粒度（搜索步进 1 MiB，最小盘是「MiB 级」的数）。
- 取样点：S 取 4、8、16，L 取 1 MiB（另有 S = 8、L = 4 MiB 一点），默认盘只取 4 GiB × 2、L = 768 MiB 一个容量；盘是内存稀疏盘，文件后端与别的 L 没量。
- 只量一个文件的有文件状态；多文件、有快照、有删除队列时的各项没量（环窗口的「删除队列非空时挂载前那一判」是另一条路径）。
- 候选甲（收紧到实际几何）之后最小盘能小多少、三块盘以上：射程外（登记第十一节）。
- K32 空池 `before_a_publish` 入口为什么扣环窗口、条款与实现哪一边该改：没判，记「条款的这一格可能错」。
- 第四段（H3 读缓存）这一页没有读数。
- H2 答不了的：候选甲、乙、丙改完之后的代价（只量了今天的代码）；R2e 发布两行（登记的 X5 造不出）；比 16 TiB − 1 MiB 更大的盘（阶梯端点，最大一档只知道不小于它）；「本机能造的最大稀疏盘」只在登记的阈值 T 与这一次派到的两台上成立，T ÷ 2 时是 4 TiB；某档只在一台上的读数不外推到另一台（这一次每格两台都有轮）；R2f 的绝对耗时两台都不足 5 轮、45 格轮里 32 格轮起跑或收尾时负载第一列 ≥ 2.0，不是空闲单机的数；占用格的首个空槽没有离开起点（落点全是 2），几何敏感性量到的是记录数的影响，不是首个空槽远近的影响；服务记的尝试峰值含页缓存，没拆开量；镜像所在的本地文件系统 `stat -f -c %T` 两台都报 ext2/ext3（45 行 `image_filesystem=ext2/ext3`），别的文件系统没量；装置单测与变异三个数这一段没自己跑，单测数现为 87 个、E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 的变异行 64 条（`grep -c '#\[test\]'` 对装置源文件、`grep -c '^E183 ' crates/mutations.tsv`）。
- H5 答不了的：R5c、R5d（V35，24 格）；清扫发布本身的写放大（产物里每次挂载写字节混着每次挂载自己的写）；阈值那一次挂载为什么多读 32768 字节（推的是沿链数行数，新臂只证实了差等于链读的字节数）；挂载之间写用户数据的历史；开关开着（后台清扫）那一支；坏扇区让清扫一行都删不掉的历史；θ 在 64 与 365、365 与 369 之间的取值，θ = 366 至 368 没量（登记候选只有四档）；每格只连挂到第二次清扫，第三次及以后没量；4 GiB × 2、L = 768 MiB 一个容量，内存稀疏盘，没有文件后端；第二台与本机是同一份程序文件，但硬件与文件系统不同的影响没有单独量，只有首片重跑一例逐字段相同。
- 变异三个数与全部 73 个单测的绿，这一段没自己跑；单测的绿是主 agent 的读数，变异整表待提交时跑。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:852-867`（整段抄，未转述）**

```markdown
### 路径与结论登记

结论是「装置里算出的环窗口与单元区容量，与独立算出的锚点一致」这一类：同一个量用三条互不共享代码的路径各算一遍再比对。

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 核心层读数 | 装置在 mkfs 与可写挂载交回的状态上调 `admission_reading_of_a_writable_mount`、`admission_reading_before_a_publish`，读 `crates/` 里现算的六项与容量（环窗口是 `ring_window_on_one_device`） | `crates/singlefs-core/src/admission.rs:487 ring_window_on_one_device` | D28（挂载期承诺量） 已定项 1 的环窗口式子（(3S + 1) × W）与 W 的上界；单元区几何由 mkfs 写出；常量 812、169、144 取自 `singlefs_format`。条款句读错时三条路径一起错 |
| 2 | 装置本地移植 | 装置把锚点的几何算术（单元区首末槽、根层级、各层位置数 N、W、环窗口）移植成 Rust，在 `anchors` 项里与登记的脚本输出逐项比，在单测里钉 15 行 | `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:315 ring_window_by_the_clauses` | 本地常量（抄自 kb 登记位，用 `constants` 项与 `crates/` 回比）；同一条条款句 |
| 3 | 锚点脚本 | 纯 Python，只用条款句与 kb 常量现算同一批量，不 import 装置、不读 `crates/`；A31 核对脚本再用它对产物每份读数现算一遍 | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py:28 ring_window_slots` | 同一条条款句；kb 常量 812、169、144、槽宽 16384、journal 环起点槽 1024；A31 核对脚本对空池 `before_a_publish` 的预期按实现读（扣），所以那一格的脚本预期不独立于实现 |
| 4 | H5 实测轨迹 | 装置驱动 `mount_writable_with_instance_table_sweep_configuration` 与卸载，逐次读行数与清扫结局（`h5_mount`），内存稀疏盘 | `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:5107 mount_once_in_the_history` | D18（块里携带什么信息） 已定项 11 的触发谓词与「回收」一条、D28（挂载期承诺量） 已定项 3 的片数式子；`crates/` 的挂载实现 |
| 5 | H5 装置内模型 | 装置把锚点脚本的 `h5()` 移植成 Rust，每片先算一遍模型、与实测逐次比，并与登记脚本的输出逐格比（`matches_the_registered_script_output`） | `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:4386 sweep_history_by_the_anchor_model` | 同一批条款句：每次可写挂载写一行、暖机补根、正常卸载推空发布的次数、清扫删根环里一条根都没有的实例的行 |
| 6 | H5 锚点脚本 | 纯 Python 现算同一批量（挂载次数、清扫在第几次挂载、各删几行、剩几行、行数峰值、片数峰值、终了 txg） | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py:38 h5` | 同一批条款句；kb 常量 369、32768、N_switch = 3 |
| 7 | H5 独立核对 | 核对脚本对产物 24 格逐格比路径 6 的输出，加「交给后台」对照一格 | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_history_anchor_check.py:14 fields_of` | 路径 6 的脚本；产物里的逐次挂载行 |

跨路径的比对断言钉在源码和产物里：`anchors` 项运行时比路径 2 与路径 3（`anchor_mismatches=0`）；单测 `local_geometry_reproduces_every_registered_script_row`（装置源文件 2660 行）钉路径 2 复现脚本的 15 行；A31 核对脚本比路径 1 与路径 3（`mismatches=0`，56 份读数）。结论的取值是枚举点，不是扫遍定义域：7 个取样点 × 2 种状态 × 2 个入口 × 2 块盘，理由是登记只要求这几个几何点（第八节 8.2）。条款句读错、三条路径一起错的情形，这三条路径看不出来；条款本身的对错靠 K30 至 K34 逐条对拍，这一段里 K32 空池 `before_a_publish` 入口那一格与实现对不上。路径 4 至 7 是 H5 的：结论是 24 个枚举点（3 个 S × 4 个 θ × 2 种卸载方式）加一个对照格上逐次相符，不是扫遍定义域；路径 5、6、7 共用路径 4 读的同一批条款句，条款句读错时它们一起错，K33、K34 是对着条款字面逐次判的，不靠这几条路径互比；独立核对（路径 7）读的是路径 4 打出来的产物，与路径 6 同源。

```

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:868-875`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-11-03 不受影响：第 2 次重跑只跑第二段 H2，量的是一次分配、一次发布的耗时与峰值内存随盘容量与记录数的斜率（C567（分配与发布的代价随盘容量线性） 的改前基线），不读准入式子的六项，第一段、第三段的产物与读数没动；2026-10-30 不受影响：第 1 次重跑只把装置里清扫阈值的本地常量 369 改成 365 并重出第一段产物，逐行 diff 只变第 6 行，11 条判决行逐字相同，R4a 至 R4d、K31、K32 的读数没变，第三段没重跑；2026-10-23 不受影响：上游写序变化（取号写两轮、C755（恢复与两种挂载触发退回的失败不同）每次可写挂载问一次 F2）之后第一段重出产物、只修钉值，只有 28 行 `name=h4_mount_tally` 的系统配置读写数变了，R4a 至 R4d、K31、K32 的读数与判决行逐字相同，第三段没重跑；2026-10-21 不受影响：正文只补了编号的简称、停机条款改用条款名、位置指代换成节名、源码落点改成现行行号，数、结论与产物不变；2026-10-19 不受影响：该分项的依据段没引 E183（容量斜率读缓存小盘准入与清扫阈值的代价数），这一页只备料；环窗口在最小盘上占准入预留的 692 至 888 千分之一、K32 空池 `before_a_publish` 入口与条款句字面对不上，这两点交主 agent 判是否升成支撑或推翻、是否要补条款句，决策正文这一次没有改；第三段（H5）没碰这一分项 |
| D28（挂载期承诺量） 已定项 3 | 备料 | 2026-11-03 不受影响：第 2 次重跑只跑第二段 H2，量分配与发布随容量、记录数的代价，不碰切换预留与实例表的片数，第一段、第三段的产物与读数没动；2026-10-30 不受影响：第 1 次重跑只把装置里清扫阈值的本地常量 369 改成 365 并重出第一段产物，逐行 diff 只变第 6 行，11 条判决行逐字相同，R4a 至 R4d、K31、K32 的读数没变，第三段没重跑；2026-10-23 不受影响：上游写序变化（取号写两轮、C755（恢复与两种挂载触发退回的失败不同）每次可写挂载问一次 F2）之后第一段重出产物、只修钉值，只有 28 行 `name=h4_mount_tally` 的系统配置读写数变了，R4a 至 R4d、K31、K32 的读数与判决行逐字相同，第三段没重跑；2026-10-21 不受影响：正文只补了编号的简称、停机条款改用条款名、位置指代换成节名、源码落点改成现行行号，数、结论与产物不变；2026-10-19 不受影响：该分项的依据段没引 E183（容量斜率读缓存小盘准入与清扫阈值的代价数），挂载期承诺量里的切换预留与 checkpoint 保留池在 K31 对拍里 28 份读数全部相符，没有需要改正文的冲突；第三段（H5）量了链重写的片数台阶：行数峰值 370 与 739 时片数峰值是 2 与 3，装置模型与锚点脚本都给同一个片数（A34，24 格相符），与这一分项「链重写」一条的 max(1, ⌈(rows0 + N_switch) ÷ 369⌉) 一致，该不该升成支撑交主 agent |
| D18（块里携带什么信息） 已定项 11 | 备料 | 2026-11-03 不受影响：第 2 次重跑只跑第二段 H2，被测的是分配器与发布路径，H2 每格只做两次可写挂载，实例表行数到不了清扫阈值（按上一次登记 5.2 的格步骤推的），第三段的产物与读数没动；2026-10-30 不受影响：第 1 次重跑只把装置里清扫阈值的本地常量 369 改成 365 并重出第一段产物（第一段的挂载走产品路径，历史的实例表行数到不了阈值，阈值只在回比那一行读），逐行 diff 只变第 6 行，11 条判决行逐字相同，第三段没重跑；2026-10-23 不受影响：上游写序变化（取号写两轮、C755（恢复与两种挂载触发退回的失败不同）每次可写挂载问一次 F2）之后第一段重出产物、只修钉值，只有 28 行 `name=h4_mount_tally` 的系统配置读写数变了，R4a 至 R4d、K31、K32 的读数与判决行逐字相同，第三段没重跑；2026-10-21 不受影响：正文只补了编号的简称、停机条款改用条款名、位置指代换成节名、源码落点改成现行行号，数、结论与产物不变；2026-10-19 不受影响：该分项的依据段没引 E183（容量斜率读缓存小盘准入与清扫阈值的代价数），这一页只备料；第三段（H5）在 θ = 64、365、369、738 上量了「实例表行数超过阈值就在这次挂载里立刻判、抹、发清扫发布」：18852 次挂载全部与条款的触发谓词相符（K33），50 次清扫删掉的行都是根环里一条根都没有的实例的行（K34），该不该升成支撑交主 agent；清扫发布的写放大这一关 R5c 不出，对该分项现行文字没有需要改的冲突 |

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:1-8`（整段抄，未转述）**

```markdown
## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑

**中文** · [English](202-gpu-vs-cpu-gc-comparison.en.md) · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

跑前登记 `research/prompts/e202-preregistration.md`。回答问题单 `research/prompts/m4-gpugc-compare-questions.md`：比较同一负载、同一池下 `background_compute=cpu` 与 `background_compute=gpu` 两种模式的 GC 挂钟、占用的主机 CPU 时间与省下的时间、前台 fsync 延迟分布，以及省下的 CPU 时间随盘容量的变化。

装置 `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`（入库装置，住在 background-gpu 包里）。**这一页只记跑前登记与装置骨架已写，产物一次没跑。**

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:9-14`（整段抄，未转述）**

```markdown
### 现状

- 跑前登记已写（2026-11-03），装置骨架已立，产物一次没跑。
- 盘容量采用定占用读法（容量涨，活数据占单元区比例固定 50%）。
- 装置约束：GPU 模式每张卡上限 1 GiB、按卡的顺序填满；一张卡与多张卡各跑；判定逐条与 CPU 参照相同才计一组数。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:15-21`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 入库装置编译与运行归主 agent；经 task-server 调度
task-server/task submit -- cargo run --release -p singlefs-background-gpu --features gpu --bin e202_gpu_cpu_gc_comparison
```

````

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:22-30`（整段抄，未转述）**

```markdown
### 口径

- 原始输出未留存：实验未跑。
- 一次整理：一次跑里 GC 线程从第一次调后台入口到最后一次调用返回为止做的全部整理。
- GC 挂钟：GC 线程上每一次后台入口调用前后读 `std::time::Instant` 之差相加。
- GC 的主机 CPU 时间：每次 GC 调用前后读 `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` 之差减去同一窗口前台线程增量。
- 省下的 CPU 时间：同一档同一次重复里 CPU 臂的进程口径减 GPU 臂的进程口径。
- 重复次数：每档每臂 5 次，报中位、最小、最大与中位绝对偏差。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:31-35`（整段抄，未转述）**

```markdown
### 判据与失败条款

- 判据：每一档容量上「CPU 模式减 GPU 模式」的差有数、有正负、有离散度；阳性对照下省下的 CPU 时间必须少于正常对照。
- 失败条款：若 GPU 模式省下的主机 CPU 时间在各容量档位均恒 ≤ 0，或前台 fsync 延迟劣化超限，则触发失败条款。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:36-41`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-03 不受影响：未跑，跑前登记已写，装置未跑 |

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:1-51`（整段抄，未转述）**

````markdown
## E185 GC一批搬迁的挂钟拆分 —— 第 2 次重跑已跑（占比 0.25 的几何改负载后造成了，单点核与第一、二段跑完；问题单四行都够判：第 1、2 行读这一趟，第 3、4 行沿用第 1 次重跑的产物；第三、四段够判后不跑）

**中文** · [English](185-wall-clock-breakdown-of-a-batch-of-gc-migrations.en.md) · [日本語](185-GC-一バッチ分の移動の壁時計分解.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 Q15 Q16 Q17 G1 G2 G3 G4 G5 G6 V1 V15 V16 V21 V22 V23 V26 V27 V30 V31 V33 V34 V55 V56 V94 F1 F2 F3 F4 F5 F7 F8 N4 N5 N6 N7 N8 N9 N10 N11 N12 N13 N14 N15 H1 H2 H3 H4 H5 H6 H7 H8 R2 -->

跑前登记 `research/prompts/e185-preregistration.md`（四段：第一段 `wall-clock`、第二段 `stage-split`、第三段 `gpu-submission`、第四段 `foreground-fsync`，第四段附带、够判后不跑）。回答问题单 `research/prompts/m4-gpugc-measure-forks.md` 的四行：一批搬迁的挂钟受盘还是 CPU 限、GPU 上算 GC 的哪几段（甲 只算校验和核对 ／ 乙 加区域扫描里的解头与映射反查 ／ 丙 再加选段打分）、攒批形态（一批一交 ／ 几批攒起来一次交 ／ 放宽 R）、定时兜底间隔与写带宽上限的默认值从什么量取。被测的是 `crates/` 今天的 GC 路径 `MountedSession::run_compaction`；背景是欠账 C412（整理的挂钟代价没量过），被测条款是 D26（后台整理与放置回收） 已定项 13、已定项 1 与 D24（后台重活能不能卸给 GPU） 已定项 3 的判据形式。

正文是第 2 次重跑的结果：重跑登记 `research/prompts/e185-r2-prereg.md`（sha256 前缀 `7d74c089`，写于 2026-10-23，装置改之前、这一趟的产物一份都还没跑）。起因：第 1 次重跑里 f = 0.25 两点几何没造成，问题单第 1、2 行按字面不够判；第一段里还有一段连成一片的慢段（同一份活 CPU 涨到 1.13–2.5 倍，作废后当场重试的那条 2.316 倍被收下，登记第四节第 5 行，第 161 行）。登记改了三样：负载（改法甲：只换造池第 2 步覆盖写的次序，让占比 0.25 的几何造得出）、装置的读数（每条意图前后的校准、线程与机器状态读数，造池之后逐段打成分）、作废与重试的做法（校准比接进「机器被别的活挤」，作废后先等机器回到常态再重试，第十一节修订 R2-1 至 R2-4，原判据原样保留）；另有执行员在单测之前、产物之前写的修订一（R2-5 至 R2-9，只收严或补量，登记第 415–425 行）。Q1–Q17 每一行的算法、门槛、判定照原登记（含第十二节 12.1–12.11）一个字不改。问题单第 3、4 行第 1 次重跑已够判，这一趟不再判，沿用第 1 次重跑的产物（岔路表写明哪几份）；第三、四段这一趟不跑。第 1 次重跑的数与七·6 前后的比法（重跑登记 `research/prompts/e185-r1-prereg.md`）已挪进变更史，正文只写这一趟。

装置是入库装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`（一个二进制、五个子命令：`wall-clock`、`stage-split`、`gpu-submission`、`foreground-fsync` 与这一趟新加的 `geometry-check`，只读驱动 `singlefs-core` 的公开入口，不改 `crates/` 里别的文件）。决定工作量、几何与判据门槛的常量都是本地常量，值抄自登记第五节常量表，每个子命令开跑先与 `crates/` 同名常量回比（停机条款第 1 条）；这一趟新加一个本地常量 `LOCAL_ROOT_RING_ENTRIES = 24`，回比 `singlefs_format::ROOT_RING_REGIONS × ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`（装置第 131 行、第 178–180 行），只用来打机理核对那一栏。

**现状**：

- 装置现在是 sha256 `a5b48b734f034114a6e2fe3a81f17ecd171dafba5e5b5ec1082e1b150fa5f082`，25496 行，`grep -c '#\[test\]'` 对它 = 226；三份起跑脚本的 `registered_apparatus_sha256` 都是它。第 2 次重跑那三趟用的是 `0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d`，13187 行、118 条（改前基线是第 1 次重跑第三段用的 `eadb929d…`，10226 行、87 条）。三趟产物的 `name=apparatus` 行里，编进二进制的源码哈希与起跑脚本设给装置的登记哈希都是 `0bd22743…`（停机条款第 7 条与修订 R2-8 没触发），单点核两份与第二段同形，第一段如下（第一段第 2 行）：

```text
E7RESULT name=apparatus source_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d registered_sha256=0bd2274383f12e88beab3f96f354148698cfcd2043263b5d5709e32e1946784d
```

- 单测与变异（登记第十二节第 427–431 行，量的是主 agent 在一次性编译目录里跑的，这一页没重跑）：单测 `118 passed; 0 failed`，clippy 退 0；`crates/mutations.tsv` 里 E185（GC一批搬迁的挂钟拆分） `V` 起头的行现在 94 条（在 `crates/mutations.tsv` 里按该前缀 `grep -c` 输出 94）：V1–V15 照原登记第九节，V16–V55 是第 1 次重跑执行员补的，V56–V94 共 39 行是这一趟补的（V56–V76 登记第九节，V77–V94 修订一 R2-9），这 39 行的轻型变异全部抓到（登记第 431 行：计数「抓到 314 条、没红 0 条、无效 0 条」，39 行都在抓到里）；整表的三个数待提交时门禁 checker-tier-crates-mutation-replay。
- 这一趟的产物（都在 `research/results/`，行数用 `grep -c ''` 数，末行都是 `name=done`）：
  - 单点核：`research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`（125 行，sha256 `c47db8bf3a7841d65aa209c3a3fb4ba941e9c30ec68672cde1b0fc480c206e00`，末行 `E7RESULT name=done emitted=125`），对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（125 行，sha256 `0add58dbc58f0948d0b115d6575173494aca801550508c2d5d732ad38885a5c2`，末行 `E7RESULT name=done emitted=125`），输入清单 `research/results/e185-r2-inputs-geometry-check-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-geometry-check-2026-10-23.txt`。
  - 第一段：`research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（2103 行，sha256 `18191ec3a8afbb27405987f99d3e36e5ae2538d3c9ab4f2717738495e767096b`，末行 `E7RESULT name=done emitted=2103`），输入清单 `research/results/e185-r2-inputs-wall-clock-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-wall-clock-2026-10-23.txt`。
  - 第二段：`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（1803 行，sha256 `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a`，末行 `E7RESULT name=done emitted=1803`），输入清单 `research/results/e185-r2-inputs-stage-split-2026-10-23.txt`，清场记录 `research/results/e185-r2-quiesce-stage-split-2026-10-23.txt`。
  - 沿用的（第 3、4 行，不重算）：第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（sha256 `978ae7b5fe60ab8f5a87338b982e8989c4c0d0ba8f19994bfc8e1c39b3d437fd`），输入清单 `research/results/e185-r1-inputs-gpu-submission-2026-10-22.txt`、`research/results/e185-r1-inputs-2026-10-22.txt`。
  - 存档不引（第 1 次重跑与 2026-10-20 那一趟的产物，数在变更史）：`research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out`、`research/results/e185-r1-comparison-2026-10-22.out`（比法判决行 `output_void=false`，即那一份没作废）、`research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out`、`research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out`。
  - 2026-10-24 第 3 次与第 4 次重跑产物（三方论证备料，出处见 `research/prompts/e185-r3-prereg.md` 与 `research/prompts/e185-r4-prereg.md`）：`research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out.err`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24.out.err`、`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out`、`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out.err`、`research/results/e185-r3-inputs-function-split-2026-10-24.txt`、`research/results/e185-r3-inputs-host-cpu-per-submission-2026-10-24-2.txt`、`research/results/e185-r3-inputs-host-cpu-per-submission-2026-10-24.txt`、`research/results/e185-r3-inputs-recompute-disagreement-2026-10-24.txt`、`research/results/e185-r3-quiesce-host-cpu-per-submission-2026-10-24-2.txt`、`research/results/e185-r3-quiesce-host-cpu-per-submission-2026-10-24.txt`、`research/results/e185-r3-quiesce-recompute-disagreement-2026-10-24.txt`、`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`、`research/results/e185-r4-inputs-cpu-writings-2026-10-24.txt`、`research/results/e185-r4-inputs-host-cpu-drift-control-2026-10-24.txt`、`research/results/e185-r4-quiesce-host-cpu-drift-control-2026-10-24.txt`、`research/results/e185-r4-serial-cpu-writings-2026-10-24.txt`、`research/results/e185-r4-serial-host-cpu-drift-control-2026-10-24.txt`。
- 跑在哪：单点核、第一段、第二段在第二台先后跑，不同时跑，真盘臂的镜像放在同一块盘的 ext4 上（输入清单第 22–26 行：型号 CT2000T500SSD8，逻辑块 512，文件系统 ext4；第一段配置行 `image_directory_filesystem=ext4 image_directory_is_cleared_at_boot=false image_device=64512`），不带 `gpu` 特性编（三份输入清单里 `grep -c lookup-work-counters` 都输出 0），外层内存上限 46G、`E185_MEMORY_CAP=40G`、装置线程 31（第一段配置行 `worker_threads=31`）。三份输入清单第 1–11 行（装置、`transaction.rs`、`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`unit.rs`、`checksum.rs`、`block_device.rs`、`allocator.rs`、`singlefs-format/src/lib.rs`、`Cargo.lock` 的 sha256）逐行相同（`diff` 核过），第 12 行是各自拷树后的提交号。每趟整条命令经 `scripts/crash/gpu-quiesce-run.sh` 起，清场记录原样：

```text
GPU_QUIESCE step=quiesce result=stopped-and-read-back holders=0
GPU_QUIESCE step=command result=exit-0
GPU_QUIESCE step=restore result=not-restored holders=1
GPU_QUIESCE step=exit code=0
```

  单点核那一份 `step=quiesce result=stopped-and-read-back holders=0`、`step=restore result=not-restored holders=0`；第二段那一份第一行是 `step=quiesce result=joined-and-read-back holders=1`（别的会话已登记同用卡，加入而不是新停）；三份都是 `step=command result=exit-0`、`step=exit code=0`。清场回读没过会退 78、命令不起，三趟都没有。
- 判决行（单点核两份、第一段、第二段，整行抄）：

```text
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=highest_full_segment_last points=2 geometry_formed_points=2
E7RESULT name=verdict segment=geometry_check constant_mismatches=0 overwrite_order=slot_order points=2 geometry_formed_points=1
E7RESULT name=verdict segment=wall_clock constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none discrimination_failures=0 clause_wall_clock_not_rising_with_the_batch_triggered=0 clause_two_cpu_paths_disagree_points=0 gc_to_sequential_write_bandwidth_maximum_ratio=0.015977 gc_to_sequential_write_bandwidth_judgement=single_run_enough stored_intents=140
E7RESULT name=verdict segment=stage_split constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none
```

  四行里取值 `false` 的字段没有，也没有 `not_run`，名字表示违例、不匹配、失败的计数（`constant_mismatches`、`constant_or_anchor_mismatches`、`stop_clause_findings`、`positive_control_failures`、`discrimination_failures`、两条失败条款的触发数）都是 0；大于 0 的整数只有 `stored_intents=140` 与 `points`/`geometry_formed_points`：旧造法对照那一份 `geometry_formed_points=1`（两点里只造成了占比 0.75 那一点）是对照的预期，它重现了第 1 次重跑的「没造成」，改法甲那一份是 `geometry_formed_points=2`。`gc_to_sequential_write_bandwidth_maximum_ratio=0.015977` 是 Q16 的值不是失败计数。
- 起跑脚本：三趟当时跑的是仓根 `e185-r2-tools/` 下的 `run-geometry-check.sh`、`run-wall-clock.sh`、`run-stage-split.sh` 与它们共用的 `common.sh`；之后照用户要求，合成了 `scripts/experiment-apparatus/e185-r2-segment-run.sh`（登记第五节「跑法」已改写，第十二节「起法」一行写明），三趟跑的是合并之前那几份，每段做的事相同（写输入清单 → 在拷过去的树里编一次装置 → 跑这一段 → 产物写进 `research/results/`）。
- `scripts/experiment-apparatus/replay.sh` 还没有 E185（GC一批搬迁的挂钟拆分） 的登记行（在 `scripts/experiment-apparatus/replay.sh` 里按该实验号 `grep -c` 输出 0；照 E184（常规层与大负载层的代价计数） 的先例只登记确定性的一片），三份测量产物都带挂钟字段。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:52-128`（整段抄，未转述）**

````markdown
### 几何：f = 0.25 两点造成了（改法甲）

改法甲只换造池第 2 步覆盖写的次序（「做了什么」第 2 条），占比 0.75 的次序不变。单点核（登记第五节「单点核」，只核 P = 1024）有两份：改法甲 `research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out`，对照 `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out`（`--overwrite-order slot-order`，第 1 次重跑那种造法）。登记三格的读数：

| 格 | 读哪 | 读到的 | 判 |
|---|---|---|---|
| 甲造成 | 改法甲那一份第 64 行 `name=geometry`、第 55 行 `name=segment_composition` | `formed=true`；那一段 `live_slots=16 slots_of_this_workload=16 other_slots=0` | 成立，停机条款第 8 条（第十条 H1）不中 |
| 占比 0.75 没被同一棵树里的改动带歪 | 两份第 123 行 `name=pool fraction=0.75 pool_data_units=1024` 的 `digests=` 整值 | 相同（紧接的代码块里 `diff` 退 0） | 成立，停机条款第 9 条不中 |
| 对照：旧造法在今天的树上重现「没造成」 | 对照那一份第 3 行 `name=geometry_attempt`、第 64 行 `name=geometry` | `geometry_formed_segments=0`、`choice=none … formed=false` | 与登记推的 0 相同 |

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=segment_composition fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 region_start_slot=53824 live_slots=16 slots_of_this_workload=16 other_slots=0 formed=true
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.697 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1056 geometry_formed_segments=1 greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry_attempt fraction=0.25 pool_data_units=1024 workload_changes=0 construction_seconds=20.776 overwritten_units=776 filler_units=0 floor_raises_pushed_by_the_session=0 release_rounds=4 every_overwritten_slot_released=true mapped_units=1058 geometry_formed_segments=0 greedy_region_start_slot=50368 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=none fallback_region_start_slot=none formed=false reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

改法甲下 greedy 自己选中的那一段（第 3 行 `greedy_region_start_slot=51008 greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1 greedy_formed=false`）仍是装着别的单元的段，f = 0.25 的点跟占比 0.75 一样走登记好的后备（`choice=forced_fallback`，只供测试的强制选段点名那一个 f 段，第 64 行 `reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change`）。所以第一、二段量的是被强制选定的那一段，不是 greedy 自己选中的段。第一、二段八个 `name=geometry` 行（四个点 × 两段）全是 `choice=forced_fallback … formed=true`：

```text
$ grep -h 'name=geometry ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | grep -c 'choice=forced_fallback .* formed=true'
8
```

```text
E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=forced_fallback fallback_region_start_slot=53824 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
E7RESULT name=geometry fraction=0.25 pool_data_units=8192 choice=forced_fallback fallback_region_start_slot=78464 formed=true reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change
```

占比 0.75 的池与第 1 次重跑是不是逐字节相同，登记第五节两道，各报各的：

```text
$ diff <(awk 'NR==237||NR==515' research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
1,2c1,2
< 1024 423aa5bc004b3247287ae29cf0bd80de0508b33ec2724f654640e0475fdbc216,ec8a4ecb79383a2f66c6828c4ce64bc0234a708f10c09b7566a5d58b79200ca3
< 8192 e0910aa57c7de2b42dfd1026123435f3e84244105ef0067fb6b0aa44edc3cbb6,d177e4fc925f9c701acdd6e9a879bfff3017615db4c034e1d0c9e55ea6902e26
---
> 1024 a77b38d298bf0f30358b2011867845734bea60928bcb4b1e98b2ab13563d30c3,c43dcaf3fa8b68f1cf7b1d498569f1fa96a6220d07b428d8b5ceecb8b5ebc5d0
> 8192 7d26930931d792bf5859dfc1ceaa83a87ae4a2328ffce19af576986880642c3e,72de4e03f439421f3e56933997b33246b24351994fb457965e6d157778679ede
exit=1
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
$ diff <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/') <(grep '^E7RESULT name=pool fraction=0.75 pool_data_units=1024 ' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | sed -E 's/.* pool_data_units=([0-9]+) .* digests=([^ ]+).*/\1 \2/'); echo "exit=$?"
exit=0
```

第 1 道（与第 1 次重跑第一段第 237、515 行比）不同，第 2 道（同一棵树里：单点核两份、再加第一段第 912 行）相同。照登记第五节判法记：占比 0.75 的池与第 1 次重跑不同，同一棵树里改动没有带歪它；差在两棵树之间（核心层五份源文件与 `Cargo.lock` 的 sha256 与第 1 次重跑输入清单不同，登记第三节第 134 行）。不停，占比 0.75 的点照量，这一趟的数不与第 1 次重跑的数混用，所以岔路表与各量的判定只引这一趟。

机理核对（登记第八节，第十条 H2；只报，不是锚点，对不上不作废）：覆盖写副本落进这次造池打出的旧洞的份数，装置逐次数出来的与登记推的 `overwritten_units − 24` 并列，造成段数两条路数并列，紧接的两条命令从三份产物里数：

```text
$ grep -h 'name=overwrite_landing' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out | grep 'status=counted' | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .*copies_landing_in_holes_of_this_construction=([0-9]+) .*predicted_copies_landing_in_holes=([0-9]+).*/\1 \2 \3 \4/' | sort | uniq -c
      3 0.25 1024 752 752
      2 0.25 8192 6128 6128
      3 0.75 1024 256 256
      2 0.75 8192 2048 2048
$ grep -h 'name=segment_composition_summary' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out | sed -E 's/.*fraction=([0-9.]+) pool_data_units=([0-9]+) .* formed_segments=([0-9]+) geo.*/\1 \2 \3/' | sort | uniq -c
      2 0.25 1024 1
      2 0.25 8192 1
      2 0.75 1024 3
      2 0.75 8192 3
```

三样都对上：每个点落进旧洞的副本数 = 登记的预测（752、6128、256、2048）；改法甲下 f = 0.25 的 `formed_segments` 是 1（登记预测 1，与 P 无关），f = 0.75 是 3（预测 3）；对照那一份 f = 0.25 的 `geometry_formed_segments=0`（预测 0，上面对照格）。`name=segment_composition_summary` 共 12 行（第一、二段各 4、单点核两份各 2），`counts_agree=true` 12 行、`counts_agree=false` 0 行（`grep -h 'name=segment_composition_summary' <四份产物> | grep -c 'counts_agree=true'` 输出 12，`grep -c 'counts_agree=false'` 输出 0）。记「调查员推的机理在这几个点上对上了」；这一句只说这四个点与单点核那两点，不外推到别的 f、P。

```text
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=58 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=776 copies_landing_in_holes_of_this_construction=752 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=752 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=472 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.25 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=6152 copies_landing_in_holes_of_this_construction=6128 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=6128 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 segments=53 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=1024 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=280 copies_landing_in_holes_of_this_construction=256 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=256 root_ring_entries=24
E7RESULT name=segment_composition_summary fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 segments=418 formed_segments=3 geometry_formed_segments_of_the_attempt=3 counts_agree=true
E7RESULT name=overwrite_landing fraction=0.75 pool_data_units=8192 overwrite_order=highest_full_segment_last workload_changes=0 status=counted overwritten_units=2072 copies_landing_in_holes_of_this_construction=2048 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=2048 root_ring_entries=24
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:129-139`（整段抄，未转述）**

````markdown
### 问题单第 1 行够判的集合 S

登记第一节「第 1 行够判」的读法（设计员写的，主 agent 认下，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 381 行）：S = 第一段里（f、P、K）三者的格，那一点 `name=geometry` 写 `formed=true`，且那一格 `name=q3` 写 `judgement=compatible`；两个 f 在 S 里各有至少两个 K 就够判；Q1、Q2 在 S 的格上报的判定就是第 1 行的读数。

```text
$ grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true' research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
12
```

第一段 `name=geometry` 四行全 `formed=true`（上面），`name=q3` 12 格全是 `judgement=compatible row_one_judgeable_here=true`（第 1879、1883、1887、1891、1895、1899、1903、1907、1911、1915、1919、1923 行），所以 S = 全部 12 格：f = 0.25 与 f = 0.75 各 6 格（P = 1024、8192 × K = 4、8、16）。第二段 12 格同一批点。F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`），F4 没有（每格 5 轮判定一致）。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:140-192`（整段抄，未转述）**

````markdown
### 各量的判定（第 2 次重跑）

照原登记第六节各报各的判定，5 轮判定不全相同写「不稳定」。产物：第一段 = `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（这一页里行号叫「第一段第 N 行」），第二段 = `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（「第二段第 N 行」）。区间是对那一组行取最小与最大，两端都是产物里的原值。第一段 Q1、Q2 合计 24 格的判定数：

```text
$ W=research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
$ grep -E 'name=q[12] ' $W | sed -E 's/.* judgement=([a-z_]+)$/\1/' | sort | uniq -c
     12 both
     12 cpu
```

第一段 12 个（f、P、K）格，每格在 `name=q1`、`name=q2`、`name=q3`、`name=q14` 四行里各一行，四行连着排（第 1877–1880 行起，每格 4 行，到第 1924 行）。

| 量 | 读哪 | 这一趟的数 | 判定 |
|---|---|---|---|
| Q1 ρ_真盘 | 第一段 `name=q1` 12 行（第 1877、1881、1885、1889、1893、1897、1901、1905、1909、1913、1917、1921 行） | `rho_median`：P = 1024 六格 1.221978–1.493617，P = 8192 六格 8.441898–9.848870 | P = 1024 六格 `both`，P = 8192 六格 `cpu`；每格 5 轮判定全相同（`judgement_per_round`），没有 `disk`、没有 `unstable`；f = 0.25 的六格与 f = 0.75 的六格同判。随 P 翻面（第 1926 行六对全翻），不随 f（第 1925 行 `flips=0`）、不随 K（第 1927 行 `flips=0`） |
| Q2 挂钟比 | 第一段 `name=q2` 12 行（第 1878、1882、…、1922 行，每隔 4 行） | `intent_wall_ratio_median`：P = 1024 六格 1.723812–1.906832，P = 8192 六格 1.105487–1.118963 | 与 Q1 同判：P = 1024 六格 `both`，P = 8192 六格 `cpu`，每格 5 轮相同；随 P 翻面（第 1929 行 `flips=6`），不随 f（第 1928 行）、不随 K（第 1930 行）`flips=0` |
| Q3 两路对账 | 第一段 `name=q3` 12 行（第 1879、1883、…、1923 行） | 每轮 `reconciliation_per_round` 最大 0.103088（f = 0.75、P = 1024、K = 8 第 5 轮，第 1907 行）；f = 0.25 的格最大 0.055696（P = 1024、K = 16 第 2 轮，第 1887 行） | 12 格全 `compatible`、`row_one_judgeable_here=true`；F5 没触发（判决行 `clause_two_cpu_paths_disagree_points=0`）。离 20% 的门槛最近的一轮是 0.103088 |
| Q4 批大小翻倍 | 第一段 `name=q4` 12 对（第 1933–1944 行） | 每对 5 轮中位 1.015102–1.074115 | 内存臂 6 对、真盘臂 6 对全 `rises`，12 对的 `copy_bytes_double=true`、`clause_wall_clock_not_rising_with_the_batch_triggered=false`；判决行 F1 触发数 0。判据只要 > 1（且每轮 > 1），这一趟涨得不多：批大小翻倍，第一批挂钟涨 1.5%–7.4% |
| Q14 GC 单跑写带宽 | 第一段 `name=q14` 12 行 | `write_bytes_per_second_median` 7229989.048161–50509463.337147 | 不单判 |
| Q15 设备顺序写带宽 | 第一段第 1945 行 | `E7RESULT name=q15 direct_with_fdatasync_bytes_per_second_median=3161349388.1 page_cache_without_sync_bytes_per_second_median=13066869439.3 positive_control_row_four_holds=true` | 不单判 |
| Q16 带宽比 | 第一段第 1958 行 | `maximum_ratio=0.015977`（f = 0.75、P = 1024、K = 16，第 1954 行 `q16_point`） | 「单跑够」；三个旋钮都不翻（第 1959–1961 行 `flips=0`） |
| Q17 前台 fsync 延迟比 | 第四段 | 没跑 | 够判后未跑（第 4 行，Q16 ≤ 10%，原登记第十一节够判停机） |
| Q5 甲段份额 | 第二段 `name=stage` 批行（`kind` 是 `batch` 或 `final_batch`）的 `crc_share`；`name=stage_judgement` 30 行 | f = 0.75 最大 0.020433（第 947 行），f = 0.25 最大 0.006863（第 116 行） | 30 行 `crc_share_judgement=not_worth`，每行 5 轮都是 |
| Q6 乙段增量份额 | 同上的 `scan_increment_share` | f = 0.75 最大 0.006521（第 1065 行），f = 0.25 最大 0.002601（第 216 行） | 30 行 `scan_increment_share_judgement=not_worth`，每行 5 轮都是 |
| Q7 丙段增量份额 | 第二段 `name=choose_judgement` 12 行（第 248–250、869–871、1148–1150、1793–1795 行）的 `choose_share_per_batch_per_round` | f = 0.25 最大 0.028846（第 250 行），f = 0.75 最大 0.014279（第 1150 行）；签发那一次的原值 `choose_share_of_the_issue` 最大 f = 0.25 是 0.03894、f = 0.75 是 0.038388，也都低于 10% | 12 行 `choose_share_judgement=not_worth`，每行 5 轮都是。`name=stage_judgement` 里的丙段那一格 30 行都是 `not_run`，是登记第六节写明的（丙段改读 `name=choose_judgement`），不是漏判 |
| Q8 单线程吞吐 | 第二段批行 `crc_bytes_per_second`、`scan_increment_bytes_per_second`；签发行 `choose_record_bytes_per_second` | 甲 2875331797.740485–3189332554.68769 B/s；乙增量 1046057094.516405–12147544022.242819 B/s；丙 112157777.007656–892197239.242721 记录字节/秒 | 不单判，喂 Q9 |
| 饱和吞吐（Q9 的 CPU 一侧） | 第二段 `name=saturation` 108 行，`distinct_threads_holds=true` 108 行 | 16 线程：甲 48832371460.2–49032300970.9，乙 14532706506.7–37784969513.8，丙 1641866090.6–13741047928.6 B/s | 不单判 |

第二段几条数的命令与输出（`S` 是第二段产物路径）：

```text
$ S=research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
$ for f in 0.25 0.75; do for k in crc_share scan_increment_share; do echo -n "batch f=$f $k max: "; grep 'name=stage ' $S | grep -E "kind=(batch|final_batch)" | grep "fraction=$f " | grep -o " $k=[0-9.]*" | sed 's/.*=//' | sort -g | tail -1; done; done
batch f=0.25 crc_share max: 0.006863
batch f=0.25 scan_increment_share max: 0.002601
batch f=0.75 crc_share max: 0.020433
batch f=0.75 scan_increment_share max: 0.006521
$ for f in 0.25 0.75; do echo -n "choose_judgement f=$f max per_round: "; grep 'name=choose_judgement ' $S | grep "fraction=$f " | grep -o 'choose_share_per_batch_per_round=[^ ]*' | sed 's/.*=//' | tr ',' '\n' | sort -g | tail -1; done
choose_judgement f=0.25 max per_round: 0.028846
choose_judgement f=0.75 max per_round: 0.014279
$ grep 'name=stage_judgement ' $S | sed -E 's/.* (crc_share_judgement=[a-z_]+) .* (scan_increment_share_judgement=[a-z_]+) .* (choose_share_judgement=[a-z_]+)$/\1 \2 \3/' | sort | uniq -c
     30 crc_share_judgement=not_worth scan_increment_share_judgement=not_worth choose_share_judgement=not_run
$ grep 'name=choose_judgement ' $S | sed -E 's/.* (choose_share_judgement=[a-z_]+)$/\1/' | sort | uniq -c
     12 choose_share_judgement=not_worth
```

最大的那一行的第二段第 250 行（丙段）整行抄：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=16 rounds=5 choose_share_per_batch_per_round=0.028676,0.028423,0.028614,0.028705,0.028846 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:193-205`（整段抄，未转述）**

```markdown
### 沿用第 1 次重跑的量（问题单第 3、4 行；这一趟第三、四段不跑）

数取自第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（这一页叫「r1 第三段第 N 行」）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`。两个范围都含 f = 0.25 的两点，那两点在第 1 次重跑里几何没造成（量的是 greedy 选中的段），这一趟没有重算。

| 量 | 读哪 | 第 1 次重跑的数 | 判定 |
|---|---|---|---|
| Q9 16 线程对 GPU | r1 第三段 `name=q9` 72 行（两张卡 × 三段 × 12 格） | `ratio` 最大 0.007746（卡 0、乙增量、f = 0.75、P = 8192、K = 8） | 72 行 `judgement=gpu_loses`，单线程版 `judgement_one_thread=gpu_loses`，`clause_sixteen_thread_form_misfits_the_gc_triggered=false`（F2 没触发）。这一趟在 f = 0.25 造成了的点上没有 Q9。Q9 算不算问题单第 2 行的翻面量，交主 agent，并进七·2 设计轮定（登记第一节、调度记录第 393 行） |
| Q10 B_批 | r1 第三段 `name=q13` 的 `batch_bytes`（读第 1 次重跑第一段 `name=call`） | f = 0.75：131072–786432 字节；f = 0.25：16384 字节（几何没造成的点） | 不单判 |
| Q11 a、Q12 b | r1 第三段第 54、136 行 | 卡 0 `E7RESULT name=q11_q12 index=0 card_valid=true intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10 one_range_median_seconds=0.005244 largest_two_points_slope_seconds_per_byte=4.172332e-10`；卡 1 `E7RESULT name=q11_q12 index=1 card_valid=true intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9 one_range_median_seconds=0.005393 largest_two_points_slope_seconds_per_byte=1.191920e-9` | 不单判 |
| Q13 φ 与 n* | r1 第三段 `name=q13` 56 行 | 卡 0 `fixed_overhead_share` 0.932762–0.998500、`batches_to_merge` 125–5993；卡 1 0.911860–0.997990、94–4470 | 56 行 `judgement=merge_batches`（φ ≥ 50%，「要攒批」），两张卡同判（F8 没触发）。判别力第 201 行 `E7RESULT name=discrimination quantity=Q13 lower_value=0.911860 higher_value=0.998500 moved_threshold=0.955180 lower_judgement=one_batch_per_submission higher_judgement=merging_optional holds=true` |

推的（没量）：f = 0.25 造成之后，它的 B_批 ≤ 签发那一次的 262144 字节（原登记第七节乙类第 356 行），比 f = 0.75 的小；φ = a ÷ (a + b·B_批) 随 B_批变小只会变大，翻面要 φ 落到 50% 以下，第 1 次重跑 φ 最小的那一格是 0.911860。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:206-221`（整段抄，未转述）**

````markdown
### 阳性对照、自检与失败条款

- 第一段第 1 行阳性对照四格（第 1963–1966 行）：盘臂内存盘、真盘两格 `holds=true`，CPU 臂内存盘、真盘两格 `cpu_clause_holds=true write_wait_clause_holds=true`；CPU 臂内存盘那一格原判据那一句 `wait_clause_under_the_original_criterion_holds=false`，照 12.9 只报不判。整行抄：

```text
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=memory injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1795862.0 wait_increment_nanoseconds=362017597.0 write_wait_increment_nanoseconds=362030328.0 cpu_change=0.036631 wait_change=-3020.110262 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1701172.000000 write_wait_increment_paired_median_nanoseconds=362030518.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=memory injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=352496564.0 wait_increment_nanoseconds=68029.0 write_wait_increment_nanoseconds=67282.0 cpu_change=7.189988 wait_change=-0.567528 cpu_increment_over_injected=1.001411 cpu_clause_holds=true write_wait_increment_over_injected=0.000191 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=false original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352445233.000000 write_wait_increment_paired_median_nanoseconds=67472.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=disk device=real_disk injection=sleep_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=1782904.0 wait_increment_nanoseconds=361903509.0 write_wait_increment_nanoseconds=362318961.0 cpu_change=0.033885 wait_change=7.108353 paired_rounds=5 cpu_increment_paired_median_nanoseconds=1272218.000000 write_wait_increment_paired_median_nanoseconds=362373072.000000 holds=true
E7RESULT name=positive_control row=1 arm_of_the_question=cpu device=real_disk injection=spin_2_milliseconds_per_write baseline_rounds=5 control_rounds=5 write_calls_median=176.0 injected_nanoseconds=352000000.0 cpu_increment_nanoseconds=353173767.0 wait_increment_nanoseconds=4108344.0 write_wait_increment_nanoseconds=1711955.0 cpu_change=6.712162 wait_change=0.080694 cpu_increment_over_injected=1.003335 cpu_clause_holds=true write_wait_increment_over_injected=0.004864 write_wait_clause_holds=true wait_clause_under_the_original_criterion_holds=true original_criterion_is_reported_only=true paired_rounds=5 cpu_increment_paired_median_nanoseconds=352961316.000000 write_wait_increment_paired_median_nanoseconds=1684671.000000 holds=true
```

- 第二段第 2 行阳性对照四格（第 123–126 行）`holds=true`；两段 `name=clock_resolution_check` 都 `holds=true`（两段第 3 行）；第一段 `name=discrimination` 三行（Q1、Q2、Q16，第 1931、1932、1962 行）`holds=true`；两段 `name=anchor` 各 34 行全 `holds=true`（`grep 'name=anchor' <产物> | grep -c 'holds=true'` 两份都输出 34，`grep -vc` 输出 0）。
- 作废：第一段 `name=intent` 140 行全是 `valid=true invalid_reason=none`，第二段 60 行同；两段都没有 `name=round_invalid` 行（`grep -c 'name=round_invalid'` 输出 0），`stored_intents=140`。第 1 次重跑第一段作废过 1 轮，这一趟一轮都没作废、没重试。
- 失败条款：F1 没触发（Q4 12 对全 `rises`）；F2 这一趟不适用（Q9 沿用）；F3 打中：Q1 在 P = 1024 的六格落在 (0.5, 2)（1.221978–1.493617）、Q2 在 P = 1024 的六格落在 (1.5, 3)（1.723812–1.906832），照实记「两边都占」，是正当结果；F4 没有；F5 没触发；F7 打中：第二段 Q5、Q6、Q7 都 < 10%（最大分别 0.020433、0.006521、0.028846），照原登记记「三段都不到一批 CPU 的 10%，CPU 的大头在其余（读这一版、发布）」，交主 agent 判第 2 行的候选是否要补一个「发布那一半」的臂；F6、F8 沿用第 1 次重跑（F6：Q13 没有落在 (10%, 50%) 的格；F8：两张卡同判），这一趟没重算。
- 这一趟新加的失败条款（登记第十条）：H1（甲造不出）没触发；H2（机理对不上）没触发，见「几何」一节；H3、H4 以 F5 打中为前提，没打中，不适用；H5（轮数不足）没有，`name=round_invalid` 0 行；H6（第 1 行仍不够判）没有，S 里 f = 0.25 与 f = 0.75 各 6 格；H7（第 4 行单跑那一半翻面）没触发，`name=q16` 是 `single_run_enough`；H8（校准基线漂移）没触发，两段 `drifted=false`。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:222-241`（整段抄，未转述）**

````markdown
### 校准与机器状态读数（登记第六节新加的量，读数 1–N15）

校准比（读数 1）是测量线程每条意图前后对一块 16 MiB 缓冲连做 3 遍 CRC-32C 取中位，除以段开头 21 遍的中位；> 1.10 作废（修订 R2-1）；作废后先等机器回到常态再重试（R2-2）。其余读数只报不判。

| 读数 | 第一段 | 第二段 |
|---|---|---|
| 段开头 / 段末基线中位（读数 2） | 5243296.0 / 5227887.0 纳秒，末 ÷ 头 = 0.997061，`drifted=false`（第 5、2099 行） | 5224477.0 / 5222377.0 纳秒，0.999598，`drifted=false`（第 5、1799 行） |
| 校准自己的阳性对照（R2-3，加倍的活） | `ratio=2.003004 crc_holds=true holds=true`（第 6 行） | `ratio=2.003983 crc_holds=true holds=true`（第 6 行） |
| 前后两个校准比里大的那个的最大值（读数 1） | 1.015958；> 1.10 的意图 0 条，> 1.05 的 0 条，> 1.20 的 0 条；等常态 0 次（第 2100 行） | 1.077323；> 1.10 的 0 条，> 1.05 的 3 条，> 1.20 的 0 条；等常态 0 次（第 1800 行） |
| 追指针校准比（N15）的范围 | 1.226963–1.534854，每条意图都高于 1 | 1.279786–1.514878，同 |
| 整机其余部分占的忙比例（N10 `others_share`）最大 | 0.002502（第 4 行，段开头） | 0.001875（第 932 行） |
| 线程读数（读数 3–N9）最大 | 排队等 CPU 7508078 纳秒，非自愿切换 151，`major_faults` 全 0，CPU 号 4 个 | 排队等 CPU 914484 纳秒，非自愿切换 29，CPU 号 2 个 |

```text
E7RESULT name=calibration_trajectory segment=wall_clock intents=140 intents_with_a_readable_ratio=140 larger_ratio_peak=1.015958 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=0 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=0.998801 wait_longest_seconds=0 waits=0 wait_total_seconds=0
E7RESULT name=calibration_trajectory segment=stage_split intents=60 intents_with_a_readable_ratio=60 larger_ratio_peak=1.077323 intents_above_the_voiding_edge=0 voiding_edge=1.10 intents_above_the_lower_reported_edge=3 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=1.002871 wait_longest_seconds=0 waits=0 wait_total_seconds=0
```

追指针校准比（N15）全部高于 1（1.23–1.53）是装置报出来的，登记只要求报、不判，这一页没查原因；它没有用来作废任何意图。N14（照原作废规则的并列判定）两段各 12 行，`differs_from_the_registered=true` 0 行（`grep -c 'differs_from_the_registered=true'` 对两份产物都输出 0）：没有一条意图因校准比被作废，R2-1 在这一趟没有动任何一格的进入与否。

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:242-275`（整段抄，未转述）**

````markdown
### 问题单逐行（岔路表）

问题单 `research/prompts/m4-gpugc-measure-forks.md`（四行的「状态」一格已由主 agent 改成「够判」，调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 394 行）；够判条件照问题单原文，判定照这一趟的产物；S 的读法见「各量的判定」一节。

| 行 | 已够判 / 还差什么 | 这一趟读得出的 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 盘 ／ CPU | **已够判**。问题单够判条件：至少两种批大小、两种段内活槽占比，在内存设备与真盘上各量到一批的挂钟与按阶段拆开的 CPU 时间。f = 0.25 与 f = 0.75 的几何都造成了（第一段 `name=geometry` 四行 `formed=true`），S = 12 格（`grep -c 'name=q3 .*judgement=compatible row_one_judgeable_here=true'` 输出 12），两个 f 各 6 格、K = 4、8、16 都在，Q1、Q2 逐格 5 轮判定一致 | P = 1024 六格 Q1、Q2 都判 `both`（`rho_median` 1.221978–1.493617，`intent_wall_ratio_median` 1.723812–1.906832）；P = 8192 六格都判 `cpu`（8.441898–9.848870；1.105487–1.118963）。判定随 P 翻面（第一段第 1926、1929 行 `flips=6`），不随 f、不随 K 翻（第 1925、1927、1928、1930 行 `flips=0`）；按 f、P 报，不按 K 报。F3 打中（P = 1024 两边都占）。整行见紧接的第一块 | 登记里没有剩下没跑的量。能动它的只有 P 之外的点：装置只收 P = 1024、8192 两个池大小，两点之间与更大的池没量。推的（没量）：翻面轴是 P，池更大时 CPU 一侧只会更重 |
| 2 GPU 算哪几段 | **已够判**（第 1 行已够判）。登记第一节「第 2 行够判」：S 里每一格第二段 `name=stage_judgement` 的甲、乙判定与 `name=choose_judgement` 的丙判定都不是 `not_run`，且 `crc_bytes`、`records_read_per_choice` 大于 0：甲、乙 30 行批判定全 `not_worth`，丙 12 行全 `not_worth`，批行 `crc_bytes` 最小 131072，`records_read_per_choice` 最小 1068（命令见「各量的判定」一节） | 甲、乙、丙三段都 `not_worth`：甲份额最大 0.020433，乙增量最大 0.006521，丙每批最大 0.028846（签发那一次的原值最大 0.03894），都不到一批 CPU 的 10%。原登记 F7 打中：CPU 的大头在其余（读这一版、发布），交主 agent 判第 2 行的候选要不要补「发布那一半」的臂。Q9 沿用第 1 次重跑 72 行 `gpu_loses`，算不算翻面量交主 agent，并进七·2 设计轮定（Q9 是登记第一节写明的一格）。整行见紧接的第二块 | 登记里没有剩下没跑的量。Q9 在 f = 0.25 的点上没有（第三段不跑）；它不论算不算翻面量，都与 Q5–Q7 同向（第 1 次重跑 72 行全 `gpu_loses`，这一趟三段份额没有一格越过 10%） |
| 3 攒批形态 | **已够判**（沿用第 1 次重跑，这一趟第三段不跑）。依据：第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（sha256 `4041a1ca…`）与它读的第 1 次重跑第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`（`978ae7b5…`）。第三段装置 `eadb929d…` 比重跑登记写死的 `b3a50d50…` 多 3 行（`quarantine_mark`），主 agent 已认（本次登记第一节派发提示第 5 条，登记第 19 行） | 同第 1 次重跑：Q10 f = 0.75 每批 131072–786432 字节；两张卡 a、b 量到；Q13 56 行全 `merge_batches`，φ 最小 0.911860，两张卡同判：一批一交（甲）出局，选择在乙、丙之间，乙丙之间七·1 只给 n* 与每批字节，R 的语义与批间重扫由七·2 设计轮判（原登记第五节第 3 行臂表） | 没有剩下的量。翻面要 φ 落到 50% 以下，φ 最小的那一格也在 0.91 以上；f = 0.25 造成之后 B_批更小，φ 只会更大（推的，没量） |
| 4 兜底间隔与写带宽上限 | **单跑那一半已够判**（沿用第 1 次重跑 `name=q16`，这一趟第一段并列报、不重判）；第四段与 Q17 够判后未跑 | 第 1 次重跑第一段第 786 行 `E7RESULT name=q16 maximum_ratio=0.011019 judgement=single_run_enough …`；这一趟第一段第 1958 行 `maximum_ratio=0.015977 judgement=single_run_enough`，H7 没触发 | Q17 只在 Q16 > 10% 时跑，这一趟不跑 |

第一块（第 1 行的读数，第一段整行）：

```text
E7RESULT name=q1 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 rho_per_round=1.456034,1.438745,1.464884,1.404448,1.458415 rho_median=1.456034 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q2 fraction=0.25 pool_data_units=1024 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.772596,1.749635,1.720118,1.741935,1.760248 intent_wall_ratio_median=1.749635 per_batch_median_ratio=1.749635 judgement_per_round=both,both,both,both,both judgement=both
E7RESULT name=q3 fraction=0.25 pool_data_units=1024 batch_units=8 rounds=5 reconciliation_per_round=0.050004,0.031328,0.021439,0.016639,0.043347 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=q1 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 rho_per_round=9.819379,9.125596,9.851385,8.752512,9.908914 rho_median=9.819379 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q2 fraction=0.75 pool_data_units=8192 batch_units=8 geometry_formed=true rounds=5 intent_wall_ratio_per_round=1.108970,1.117088,1.108237,1.122220,1.114470 intent_wall_ratio_median=1.114470 per_batch_median_ratio=1.120025,1.116781,1.108746 judgement_per_round=cpu,cpu,cpu,cpu,cpu judgement=cpu
E7RESULT name=q3 fraction=0.75 pool_data_units=8192 batch_units=8 rounds=5 reconciliation_per_round=0.006159,0.006354,0.005763,0.006425,0.012056 judgement_per_round=compatible,compatible,compatible,compatible,compatible judgement=compatible row_one_judgeable_here=true
E7RESULT name=sensitivity quantity=Q1 knob=fraction pairs=6 flips=0 statement=does_not_flip_at_these_points flipped_pairs=none
E7RESULT name=sensitivity quantity=Q1 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
E7RESULT name=sensitivity quantity=Q2 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob flipped_pairs=0.25:1024:4:both->0.25:8192:4:cpu,0.25:1024:8:both->0.25:8192:8:cpu,0.25:1024:16:both->0.25:8192:16:cpu,0.75:1024:4:both->0.75:8192:4:cpu,0.75:1024:8:both->0.75:8192:8:cpu,0.75:1024:16:both->0.75:8192:16:cpu
```

第二块（第 2 行的读数，第二段整行：丙段判定、批判定的一行，份额最大的两行批行）：

```text
E7RESULT name=choose_judgement fraction=0.25 pool_data_units=1024 batch_units=4 rounds=5 choose_share_per_batch_per_round=0.014847,0.014637,0.014529,0.014800,0.014522 choose_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth choose_share_judgement=not_worth
E7RESULT name=stage_judgement fraction=0.25 pool_data_units=1024 batch_units=4 batch_index=0 rounds=5 crc_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth crc_share_judgement=not_worth scan_increment_share_judgement_per_round=not_worth,not_worth,not_worth,not_worth,not_worth scan_increment_share_judgement=not_worth choose_share_judgement_per_round=not_run,not_run,not_run,not_run,not_run choose_share_judgement=not_run
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=4 round=1 call_index=1 kind=batch call_cpu_nanoseconds=12578975 crc_cpu_nanoseconds=257032.000000 crc_wall_nanoseconds=257271.000000 crc_repetitions=1 scan_cpu_nanoseconds=332311.000000 scan_increment_cpu_nanoseconds=75279.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.020433 scan_increment_share=0.005985 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3059665722.555947 scan_increment_bytes_per_second=10446897541.146935 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=84920.000000 release_check_crc_wall_nanoseconds=85020.000000 release_check_crc_repetitions=1 release_check_crc_bytes=262144 crc_with_the_release_check_cpu_nanoseconds=341952.000000 crc_with_the_release_check_bytes=1048576
E7RESULT name=stage fraction=0.75 pool_data_units=1024 batch_units=16 round=3 call_index=1 kind=batch call_cpu_nanoseconds=13777990 crc_cpu_nanoseconds=247692.000000 crc_wall_nanoseconds=247821.000000 crc_repetitions=1 scan_cpu_nanoseconds=337532.000000 scan_increment_cpu_nanoseconds=89840.000000 choose_cpu_nanoseconds=none choose_calls=0 crc_bytes=786432 records_read_per_choice=1069 full_segments=3312 crc_share=0.017977 scan_increment_share=0.006521 choose_share_per_batch=none choose_share_of_the_issue=none crc_bytes_per_second=3175039968.993751 scan_increment_bytes_per_second=8753695458.593056 choose_record_bytes_per_second=none choose_records_per_second=none choose_full_segments_per_second=none snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds=330301.000000 release_check_crc_wall_nanoseconds=330562.000000 release_check_crc_repetitions=1 release_check_crc_bytes=1048576 crc_with_the_release_check_cpu_nanoseconds=577993.000000 crc_with_the_release_check_bytes=1835008
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:276-285`（整段抄，未转述）**

```markdown
### 做了什么

1. 计时包装设备：包住内存盘（`SparseBlockDevice`）或真盘（ext4 上的 `O_DIRECT` 镜像文件，`DirectInputOutputBlockDevice`），`BlockDevice` 的每一种调用前后各读一次 `Instant` 与本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（CPU 读数包着挂钟读数），按调用种类累计挂钟、CPU、等盘（挂钟减 CPU）、次数与字节；读按「这个计时窗口里第一次写之前 / 之后」分开，第一次写之前的读再分三栏：落在 R 那块盘 R 里的、扣掉搬迁源槽上的再读，归扫描读；这一批搬迁源的槽上（两块盘同号槽）扣掉 R 那块盘每个槽第一次读之后剩下的，归释放前读盘核（`release_check_read_bytes`，登记第十二节 12.5）；其余归别处；写按「落在这一批目的地 / 别的」分开。阳性对照的注入（每次 `write_at` 里睡 2 ms，或按线程 CPU 时钟自旋 2 ms）也在这里。
2. 造池：harness 默认参数 mkfs 两块 4 GiB 内存盘、第一个文件、可写挂载；建 P + 64 个空 inode，前 P 个各顺序写一个数据单元；整段都是这次写的段按槽序每 4 个覆盖写 3 个（f = 0.25）或 1 个（f = 0.75），混住的段这次写的单元全覆盖写掉；用多建的空 inode 把最高那一段补满；再「抬 F 到准入上限、建一个 inode」转到被覆盖写的旧落点都已释放。**覆盖写的次序（改法甲，装置 `overwrite_order_of`，第 2131 行）**：f = 0.75 仍全部按旧槽号升序；f = 0.25 改成三组依次：① 不在「整段 32 个单元都是这次写的」段里的单元（混住段的全部，与配方累计点名的），组内按旧槽号升序；② 整段里要覆盖写的单元，除槽号最大的那一整段之外，按旧槽号升序；③ 槽号最大的那一整段里要覆盖写的 24 个，按旧槽号升序。覆盖写哪些单元、写几次、写的内容不变，只换次序。机理（登记第七节「推的预测」）：每次覆盖写是一次发布、写一条根，根环 24 条根，旧落点在覆盖写那一次发布就记成已释放，覆盖写到第 25 次起每份副本都落进 24 次之前打出的洞，所以最后 24 次覆盖写打出的洞（槽号最大那一整段的 24 个）造完时还空着，那一段剩 16 个活槽、全是这次写的。
3. 几何核：在一份拷贝上可写挂载、照会话签发那一步的选段那一圈重放（`choose_region` 与「先扫」交替），看 greedy 将选哪一段；那一段全是这次写的数据单元、活槽 = 64f 就用 greedy，不是而它全是这次写的数据单元就把它们并进配方重造（至多三次），装着别的单元就走登记好的后备（只供测试的强制选段点名槽号最小的 f 段）。造池之后逐段打成分（`name=segment_composition`，N11）、造成段数两条路数并列（`name=segment_composition_summary` 的 `counts_agree`）、覆盖写副本的落点（`name=overwrite_landing`，N12）；这几行只打每个点头一次造池，第 2–5 轮的重造不打。
4. 量一条意图：可写挂载（`mount_writable_with_compaction_switches`，产品开关只把批大小换成只供测试的 `ForcedForTests(K)`），每次调用之前把会话「已经过去的发布数」设成 1000（发布预算落到 1），一次一次调 `run_compaction` 到最后一批删意图；每次调用前后读 `Instant` 与线程 CPU 时钟（挂钟读数包着 CPU 读数，与设备调用那一层反过来），返回之后在计时窗口外按发成的那一版判是签发、批还是最后一批。每条意图前后还在计时窗口外做一遍校准（读数 1、N15）和线程与机器读数（读数 3–N9，`name=intent_machine_readings`）：校准比 > 1.10 的那一条作废，作废之后先每 10 秒做一次校准、连着两次 ≤ 1.10 才重量，单次至多等 300 秒、一段里累计至多等 600 秒（修订 R2-1、R2-2）；每段开头做校准基线（读数 2，21 遍）与加倍活的自检（R2-3），段末再做一遍基线（R2-4）。段开头、每个取样点开头、段末各一行 `name=machine_state`（N10）。
5. 第二段的重放：调用之前取这一版的分配记录、映射条目（照会话的私有取法另写一份）、钉住与隔离的槽，取扫描读到的单元字节；三个阶段（甲 每个被读单元整单元 CRC-32C（另一个子项是释放前读盘核那几份副本的整单元 CRC-32C，副本字节在调用之前从两块盘同号槽各读一份）、乙 整个 `scan_region`、丙 签发那一步每次 `choose_region`）每次重放先不计时跑一遍热身、再计时跑一遍，5 次重放取中位；饱和吞吐 1、16、N 线程各 2 秒。丙段的判定（改动六）：每轮取那一轮有效意图签发那一行已经算出的 `choose_share_per_batch`，按原门槛（10%）判，五轮收成一行 `name=choose_judgement`（`name=stage_judgement` 行照旧打，它的丙段那一格仍是 `not_run`）。
6. 第三段（沿用，这一趟不跑）：每张卡用造池时写的一个真数据单元（32768 字节）、n 取 1 到 16384 段，`GpuCard::crc32c_of_ranges` 先丢一次再计 5 次取中位，核每段 CRC 与 CPU 相同、每次不快过 PCIe 链路峰值；最小二乘拟合 a、b，用第一段的 B_批 算 φ 与 n*，用第二段的 16 线程饱和吞吐算 D24（后台重活能不能卸给 GPU） 已定项 3 形式的比（Q9）。
7. 单点核（`geometry-check` 子命令，改动二）：只造池、核几何，打 `name=geometry_attempt`、`name=geometry`、`name=segment_composition`、`name=segment_composition_summary`、`name=overwrite_landing`、`name=pool` 几行与末行，不量；`--overwrite-order slot-order` 照第 1 次重跑那一版造（全部按旧槽号升序，不数副本落点）。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:286-315`（整段抄，未转述）**

````markdown
### 复跑（第 2 次重跑；命令照重跑登记第五节「跑法」，起法在 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 文件头）

```bash
# 仓根。装置在 checker 档包里，编与单测归主 agent 经 task-server 交（主工作区那一版 118 条）
SINGLEFS_RUN_SCOPE='…' task-server/task submit -- cargo test --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split
# 三趟（单点核、第一段、第二段）各一条命令，主 agent 在本机仓根经 task-server 交；派哪台由服务定，两台在同一棵共享的树里跑，产物直接写进这棵树的 `research/results/`、两台都读得到；`--disk-model` 填重跑登记第五节点名的那块盘的型号，派到的那一台没有那块盘时入口退 78、一步都不编不跑；
# <段> 取 geometry-check、wall-clock、stage-split，<日期> 是那一趟起跑当天，产物名见上面「这一趟的产物」
SINGLEFS_RUN_SCOPE='…' task-server/task submit -- env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r2-segment-run.sh <段> --disk-model <盘的型号>
```

`e185-r2-segment-run.sh` 在共享的树里、同一个绝对路径上做四步（编译目录取 `CARGO_TARGET_DIR`：派到第二台时是统一任务服务把这一趟派过去时在第二台本地盘上新建、跑完删的那一个（服务不设、只记下它在哪，`task-server/task status` 看得见），没设是树里的 `target`）：写输入清单 `research/results/e185-r2-inputs-<段>-<日期>.txt` → `cargo build --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split`（不带 `gpu` 特性，不带 `--all-targets`、`--tests`，免得并进 `lookup-work-counters`）→ 跑这一段（装置线程不设，取登记值 `.claude/heavy-thread-default`，环境 `E185_REGISTERED_APPARATUS_SHA256`、`E185_MEMORY_CAP=40G`）→ 产物写进 `research/results/`：

```bash
# geometry-check 段（单点核，两条）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 > research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-<日期>.out
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split geometry-check --point 0.25:1024 --point 0.75:1024 --overwrite-order slot-order > research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-<日期>.out
# wall-clock 段（第一段；真盘臂的镜像目录取 `E185_IMAGE_DIRECTORY`，没设取 `<MULTI_HOST_RUN_PEER_SCRATCH>/e185-images`：派到第二台时是统一任务服务把这一趟派过去时在第二台本地给的目录，不在 NFS 上、不在开机清空的目录里，跑完连目录删；没有这个变量（在本机跑）取 `${CARGO_TARGET_DIR:-target}/e185-images`；镜像目录要落在 `--disk-model` 点名的那块盘上）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split wall-clock --image-directory <镜像目录> > research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-<日期>.out
# stage-split 段（第二段；只用内存盘，与第一段同一台、先后跑）
${CARGO_TARGET_DIR:-target}/release/e185_gc_batch_wall_clock_split stage-split > research/results/e185-gc-batch-wall-clock-split-stage-split-r2-<日期>.out
```

三趟在第二台先后跑、不同时跑；装置线程不设，取登记值 `.claude/heavy-thread-default`（跑的那一刻是 31）；产物文件名的日期是那一趟起跑当天，同一天再跑加 `-2`、`-3`，不盖已有的。三趟当时跑的是合并之前仓根的 `e185-r2-tools/run-<段>.sh`，每段做的事与上面相同。

第三段与第四段这一趟不跑。主 agent 定要 f = 0.25 造成了的点上的 Q9 时另跑第三段（条件照重跑登记第五节「第三段跑不跑」，三丁换成「主 agent 定要」；命令形态照第 1 次重跑，`--wall-clock-product`、`--stage-split-product` 换成这一趟的两份 `-r2-` 产物，带 `gpu` 特性另编）：

```bash
task-server/task submit -- bash -c 'cargo run --release -p singlefs-checker-tier --features gpu --bin e185_gc_batch_wall_clock_split -- gpu-submission --wall-clock-product research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out --cards 0,1 > research/results/e185-gc-batch-wall-clock-split-gpu-submission-r2-<日期>.out'
```

````

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:316-330`（整段抄，未转述）**

```markdown
### 口径

- CPU 时间是本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` 两次读数之差（登记第一节，第十二节 12.2 修订；原来的 schedstat 读法对正在跑的线程约 1 ms 才前进一次，实测数在 12.2）。`wall-clock`、`stage-split`、`foreground-fsync` 开跑先在本线程上自旋 1 ms、连读这个钟，前进不到 100 次就停机退 3（产物行 `name=clock_resolution_check`）。设备调用那一层 CPU 读数包着挂钟读数，等盘 = 挂钟 − CPU，纯 CPU 的设备调用里可以是负的（差读钟的工夫）；一次 `run_compaction` 调用那一层挂钟读数包着 CPU 读数。这样嵌套，两个钟走得一样快时「挂钟 ≥ CPU + 等盘」不会因为读钟的先后被打破。
- 一批的挂钟是包住那一次 `run_compaction` 调用（含调用末尾的 discard）的 `Instant`；discard、屏障、写零各按调用种类单列。
- 真盘臂：每次测量之前把上一次改过的 4 MiB 块从起始件拷回工作件，再核整份镜像的分块摘要（每 4 MiB 一块的 SHA-256 接起来再算一次 SHA-256）与起始件相同；内存臂每轮按同一份配方重造、核两块盘的分块摘要与第一次造的相同。挂载时 journal 记录偏移的提示照抄单元区之前写过的扇区（同内存盘的答法），不在 `O_DIRECT` 镜像上把整个环逐槽读一遍；计时窗口里不照抄。
- 设备：内存臂是 harness 的稀疏内存盘；真盘臂是两个镜像文件，放在第二台一块 NVMe（型号 CT2000T500SSD8，逻辑块 512，`research/results/e185-r2-inputs-wall-clock-2026-10-23.txt` 第 22–26 行）上的 ext4 里，屏障是 `fdatasync`，结论只说「第二台那块盘上 ext4 的 `O_DIRECT` 镜像文件」（重跑登记第一节「真盘」）；原登记写的本机那块盘没用。
- 批：一次调用里搬了单元的那一次（与会话 `observations.batches_published` 同一口径）；签发单列，不算批。
- Q4 的 `rises` 只要求第一批挂钟的比 > 1（5 轮中位，且每轮 > 1）；这一趟 12 对的中位在 1.015102–1.074115，涨幅小，判据不看涨了多少。
- Q7 的「一批 CPU」取这条意图批 CPU 的中位（装置从原登记起的读法，登记第六节「丙段怎么有读数」写死）；丙段的值是签发那一次 `name=stage` 行的 `choose_share_per_batch`，每轮取那一轮有效意图的一个，所以 `name=choose_judgement` 每格只有 5 个值。
- 第一行阳性对照（登记第五节阳性对照表第 1 行，CPU 臂等盘那一句照第十二节 12.9）：取样点 f = 0.75、P = 1024、K = 8，每次 `write_at` 里睡 2 ms（盘臂）或按线程 CPU 时钟自旋 2 ms（CPU 臂）。增量一律是对照各轮的中位 − 基线各轮的中位。盘臂：整条意图等盘增量落在写调用次数 × 2 ms 的 [1.0, 1.5] 倍里、CPU 变化在 ±10% 以内（没改）。CPU 臂两句各自报：CPU 增量落在写调用次数 × 2 ms 的 ±10% 里（没改）；写调用（注入所在的那一种调用）的等盘增量的绝对值 ≤ 写调用次数 × 2 ms × 10%（12.9 的新判法，分母是注入量）。原判据「整条意图等盘相对基线的变化在 ±10% 以内」照算照报（`wait_change`、`wait_clause_under_the_original_criterion_holds`），不作判定用。对照点每一轮里，每个设备臂挨着量基线、盘臂对照、CPU 臂对照各一次，次序按轮号轮换（第 1、4 轮 基线 → 盘臂 → CPU 臂，第 2、5 轮 盘臂 → CPU 臂 → 基线，第 3 轮 CPU 臂 → 基线 → 盘臂），轮数 5；按轮号配对的增量中位另报、不判。
- 作废与停机（登记第十一节）：原登记八条作废、12.10 第 ③ 条与第 1 次重跑的「计数特性被并进来」照沿用；修订 R2-1 加第三样触发（这条意图前后的校准比有一个 > 1.10，`invalid_reason=calibration_ratio_above_the_registered_edge`），R2-2 作废后先等常态再重试，R2-3 校准自己的阳性对照不过就停（退 4），R2-4 基线漂移只打标记；停机条款第 7 条（装置变了，输入清单第一行与登记的哈希对不上，另有修订 R2-8 在装置里也核一遍）至第 10 条（校准锚点不符）与修订 R2-5、R2-7 把基线读不到、锚点不符的遍数累计超过 3 遍收严到停（退 3）；修订 R2-6 校准读不到那一条意图记 `invalid_reason=calibration_unreadable`。这一趟这些都没触发（两段判决行 `stop_clause_findings=0`，140 + 60 条意图 `invalid_reason=none`）。
- 第三段的 B_批（沿用）取第 1 次重跑第一段内存臂、第 1 轮、`kind` 为 `batch` 或 `final_batch` 那几行的 `scan_read_bytes_in_the_region`；φ 与 n* 按两张卡各自拟合的 a、b 算。
- 产物不带执行那台机器的路径：第一段 `name=configuration` 不打 `--image-directory` 的值，只打 `image_directory_filesystem`（目录所在设备在 `/proc/self/mountinfo` 里的文件系统种类）、`image_directory_is_cleared_at_boot`（tmpfs / ramfs，或真路径落在 `/tmp` 下为 `true`）与 `image_device`（设备号）；镜像建不成的报错只带文件名（登记第十二节 12.6）；输入清单里第二台的目录换成「〔第二台的目录〕」。
- 键名写全名，登记里的提问与条款编号只当索引：第二段甲、乙、丙三段的份额是 `name=stage` 的 `crc_share`、`scan_increment_share`、`choose_share_per_batch`（签发那一次另报 `choose_share_of_the_issue`，丙段判定读 `name=choose_judgement`）；Q16 是第一段判决行的 `gc_to_sequential_write_bandwidth_maximum_ratio` 与 `gc_to_sequential_write_bandwidth_judgement`；F1、F2、F5 是 `clause_wall_clock_not_rising_with_the_batch_triggered`、`clause_sixteen_thread_form_misfits_the_gc_triggered`、`clause_two_cpu_paths_disagree_points`；Q17 是第四段判决行的 `foreground_fsync_latency_judgement`。新旧对照表在登记 12.6。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:331-348`（整段抄，未转述）**

```markdown
### 它答不了的

- f = 0.25 量到的是被强制选定的那一段，不是 greedy 自己选中的段：改法甲下 greedy 选中的仍是装着别的单元的段（单点核第 3 行 `greedy_formed=false`），f = 0.25 与 f = 0.75 一样走只供测试的强制选段后备。这一页的第 1 行读数只对「被强制选定的、全是这次写的数据单元的段」成立，对 greedy 自己会选中的段没有读数。
- 改法甲造成的几何靠一条机理（根环 24 条根、旧落点在覆盖写那一次发布就记成已释放），机理核对只在四个点（f = 0.25、0.75 × P = 1024、8192）与单点核上对上；别的 f、P、别的覆盖写次序没试，也没试登记里的改法乙（快照留旧单元，装置没实现）。
- 占比 0.75 的池与第 1 次重跑不同（差在两棵树，「几何」一节），所以这一趟的数不与第 1 次重跑的数逐格比；七·6 前后的比法这一趟没有。
- 第 3、4 行沿用第 1 次重跑：第三段的 Q9–Q13 在 f = 0.25 的两点上量的是几何没造成的段（`batch_bytes=16384`、`merged_buffer_bytes=16` 不是显存占用：B_批 16384 字节不满一个 32768 字节的槽，装置把每批段数整除成 0 再算缓冲），f = 0.25 造成之后的 φ 与 n* 没量；第三段装置 `eadb929d…` 与重跑登记写死的 `b3a50d50…` 差的 3 行不在第三段路径上这一句是读 `git diff` 推的，没另量，主 agent 已认。
- Q9 照原登记算、照原样报；今天的 D26（后台整理与放置回收） 已定项 13 第 7 条写 D24（后台重活能不能卸给 GPU） 已定项 3 按挂钟比的判据不管它罩着的后台任务，Q9 在第 2 行还算不算翻面量，这一页不判，交主 agent，并进七·2 设计轮定（重跑登记第六节、第一节）。
- 第 1 行读数的 P 轴只有 1024、8192 两点（装置只收这两个池大小）：翻面发生在两点之间的哪里、更大的池上是什么，没量。
- Q4 的 `rises` 涨幅小（1.5%–7.4%）：「批大小是不是挂钟的主旋钮」这个问法这一页不下判断，F1 没触发只说明每一对都 > 1。
- 校准门槛 1.10 的依据只有本机上一个代用量（登记第四节第 11 行，zlib 的 CRC-32 在本机、负载 18 时的分布），第二台与装置自己的 CRC-32C 没量过；门槛在跑前写死，没看这一趟的数再改。这一趟没有一条意图越过它（最大 1.077323），所以它实际没动任何一格（N14 并列判定 24 行无一差异），也就不能说明门槛放在哪合适。追指针校准比（N15）每条意图都高于 1（1.23–1.53），没查原因。
- 校准只量测量线程此刻跑同一份活要多久，不排除对象自身（GC 路径）在个别意图里变慢；H3、H4 以 F5 打中为前提，这一趟没打中，两条没用上。F5 没触发也不能反推第 1 次重跑的 F5 是机器状态：这一趟的机器状态读数（`others_share` 最大 0.002502）与第 1 次重跑没有同一口径的对照读数。
- 第二段清场记录第一行是 `joined-and-read-back holders=1`：别的会话已登记同用卡，本趟加入而不是新停；计时窗口里别的会话有没有占第二台的 CPU，只能读 `name=machine_state` 的 `others_share`（最大 0.001875）与每条意图的排队等 CPU 读数，那是读数不是隔离。
- 释放前读盘核那一栏（登记第十二节 12.5）只认这一批搬迁源的槽。同一次发布对被换下的旧单元（重写的树节点这类）做的释放前读盘核走同一个函数、也在第一次写之前，落在 R 外，仍记在 `scan_read_bytes_elsewhere`：这一条是读 `transaction.rs` 推的，没量。分栏靠「区域扫描在释放前读盘核之前、扫描对 R 那块盘每个槽只读一遍」这个次序；调查员只在 f = 0.75、P = 128、K = 8 一个点上、只对 R 那块盘抓过调用方，另一块盘那一份是推的，单测端到端那一条在同一个点上钉住两块盘合计 2 × 搬的单元数 × 32768。甲段第二个子项不进 Q5、Q8、Q10 与第三段的 B_批，只报数。
- 几何后备：greedy 将选的段装着节点或打包单元时，登记只举了「多覆盖写那几段、让节点段的旧节点也被引用着」两种改法，装置只实现了前一种；竞争段装着别的单元时直接走后备、不试三次（登记第十二节 12.3 写死了这个读法），`name=geometry` 行写明原因；后一种改法没有试过。
- 第二段的饱和吞吐量的是重放（同一份输入反复跑），不是 GC 自己在多线程上跑：今天的 GC 整条路在调用线程上同步跑。
- 第一行阳性对照照 12.9 判「中位之差」，按轮配对的增量只报不判（这一趟 CPU 臂真盘格 `write_wait_increment_paired_median_nanoseconds=1684671.000000`，内存格 `67472.000000`）；交错之后配对判不判、怎么判没有定。
- 停机条款第 6 条（包含自检反过来）照登记字面判，但两个钟走得不一样快：线程 CPU 时钟不受校时调速，`Instant`（`CLOCK_MONOTONIC`）受。2026-10-19 执行员在本机用 python 量过（忙等、挂钟读数包着 CPU 读数）：200 ms 的窗口里 CPU 读数最多比挂钟多 1526 纳秒（比值最大 1.000007630），`CLOCK_MONOTONIC` 比 `CLOCK_MONOTONIC_RAW` 慢 7.036 ppm；挂钟换成 `CLOCK_MONOTONIC_RAW` 时 CPU 读数没有一次大过挂钟（200 ms 窗口最多 −194 纳秒）。几乎全在 CPU 上、又长到几百毫秒的调用（内存臂）因此可能让第 6 条触发，读钟那几十纳秒的余量盖不住；与线程 CPU 读数配对的挂钟已照 12.4 换成 `CLOCK_MONOTONIC_RAW`，这一趟两段判决行都是 `stop_clause_findings=0`，第 6 条没触发。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:349-368`（整段抄，未转述）**

```markdown
### 路径与结论登记

同一个量两条或以上互不共享代码的路径各算一遍再比对的，有五样：

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 扫描读字节：设备计数 | 计时包装记下的、第一次写之前落在 R 那块盘 R 里的读，扣掉这一批搬迁源槽上的再读（释放前读盘核，登记第十二节 12.5），长度相加 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:3267 split_the_reads_of_a_call` | 区域 R（会话签发的那一条意图）、槽字节 16384、这一批搬迁源的槽（会话交回的 `units_relocated_by_this_publish` 的 `source`）、读的次序（扫描在释放前读盘核之前） |
| 2 | 扫描读字节：分配记录 | 调用之前那一版 R 那块盘上未释放记录的 Σ 跨度 × 16384 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1429 expected_scan_read_bytes` | 同一个 R、同一个槽字节常量 |
| 3 | 批数：装置分类 | 每次调用发成的那一版里 logged_ops 还有没有意图、搬了几个单元 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1369 kind_of_the_call` | 会话交回的那一版 |
| 4 | 批数：会话计数 | 会话 `observations.batches_published` 的增量 | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:4937 stop_findings_of` | 无（会话自己数） |
| 5 | 批数：闭式 | ⌈m ÷ K⌉ | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1401 batches_of_one_intent` | 活槽占比定的 m |
| 6 | 一批的 CPU：真盘臂线程 CPU 时钟 | 真盘上一批的本线程 CPU | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:706 thread_cpu_nanoseconds` | 本线程 `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`（clock id 3） |
| 7 | 一批的 CPU：内存臂挂钟 | 内存盘上同一批的挂钟（等盘近 0） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:1561 cpu_reconciliation` | 同一份配方造的池；`Instant`（`CLOCK_MONOTONIC`），与路径 6 的钟走得不一样快（停机条款第 6 条那一条，口径里） |
| 8 | 造成段数：逐段成分 | 单元区里每一段（有未释放记录的段）逐段数成分，数「造成」为真的段（`name=segment_composition_summary` 的 `formed_segments`） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2901 construction_reading_lines` | `is_the_geometry_formed`（活槽 = 64f 且全是这次写的数据单元，第 2784 行）、`composition_of_the_segment`、同一份分配记录 |
| 9 | 造成段数：几何核那条路数 | 对着可写挂载后的那一版，按造池记下的数据单元槽所在的段起点筛「造成」为真的段（`geometry_formed_segments`） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2951 predict_the_greedy_choice` | 同路径 8：`is_the_geometry_formed`、`composition_of_the_segment`、同一份分配记录 |
| 10 | 落进旧洞的副本：逐次数 | 造池第 2 步每次覆盖写之后，看这一份新副本的槽是不是前面某次覆盖写空出来的旧槽（`copies_landing_in_holes_of_this_construction`） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2186 landings_of_the_copies` | 覆盖写的次序（改法甲的 `overwrite_order_of`）、每次覆盖写前后的槽号 |
| 11 | 落进旧洞的副本：闭式推的预测 | `overwritten_units − 24`（24 = 根环条数） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2210 predicted_copies_landing_in_holes` | 本地常量 `LOCAL_ROOT_RING_ENTRIES`（开跑与 `singlefs_format` 的两个常量之积回比）；机理前提：每次覆盖写一次发布、一条根，洞从第 j + 24 次发布起分得到，用户数据取最低的空槽对 |

源码落点的行号按主工作区那一份装置（sha256 前 16 位 `0bd2274383f12e88`）现查。跨路径的比对：1 与 2 由停机条款第 2 条逐批连起来（`stop_findings_of`，对不上就停）；3 与 4 由停机条款第 3 条连起来，3 与 5 只在几何造成了的点上连（同一函数）；6 与 7 是 Q3，> 20% 记第十条 F5，不停；8 与 9 由 `counts_agree` 连起来（`construction_reading_lines` 里 `formed_segments == prediction.geometry_formed_segments.len()`，单测 `construction_reading_lines_compare_the_two_counts_of_formed_segments_and_report_the_landing`，第 13124 行），对不上写 `counts_agree=false`、不停机，8 与 9 共用同一个判成造的函数，这个函数错了两边一起错，由「甲造成」那一格 `live_slots=16 slots_of_this_workload=16 other_slots=0` 的绝对值兜；10 与 11 只在 `name=overwrite_landing` 里并列打出，对不上记第十条 H2、不停机不作废，单测钉了闭式的四个值（第 12344–12347 行），没有一处断言把逐次数的与闭式的连起来，还欠这一条。取值是枚举点（两个 f、两个 P、三个 K、两个设备；8 至 11 是四个 f、P 点），理由是登记第八节只要求这几档；1 与 2 共用同一个 R，R 取错时两边一起错，由第七节「签发那一步扫 R 读到的字节」的绝对值锚点兜。

```

**出处 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:369-377`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 改了：第 3、4 次重跑的读数（一次 GPU 提交的主机 CPU，按 CPU 写法、卡、提交间隔拆开的不赔格数，主 agent 现数的表在 `research/prompts/m4-gpugc-design-r3-main-verification.md` 第二节）进了该分项依据段，撑第 1 条放上 GPU 的三段与射程里快写法下赔的卡与间隔；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：第 2 次重跑给出的是里程碑四设计轮要的读数（问题单第 1 行：池小（P = 1024）两边都占、池大（P = 8192）CPU 限；第 2 行：甲、乙、丙三段份额都不到 10%，F7 打中；第 3 行沿用第 1 次重跑 Q13 全判要攒批；第 4 行单跑够，比值 0.015977），该分项射程把「GPU 上算 GC 的哪几段」与定时兜底间隔、写带宽上限的默认值留给设计轮与 C412（整理的挂钟代价没量过），依据段是「无实验」、没引 E185（GC一批搬迁的挂钟拆分）；Q9 的用法与第 7 条是否相左，这一趟没重算（第三段沿用），Q9 算不算翻面量交主 agent，并进七·2 设计轮定；决策正文这一次没有改 |
| D26（后台整理与放置回收） 已定项 1 | 备料 | 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：这一趟装置照旧读它的停机谓词（R 按记录全空）判最后一批，不量三水位阈值；f = 0.25 几何这一趟用改法甲造成了，是负载次序的事，与触发判据无关；该分项依据段没引 E185（GC一批搬迁的挂钟拆分），决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 3 | 备料 | 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：Q9 沿用第 1 次重跑第三段（72 行全判 GPU 输、F2 没触发），这一趟没重算；第二段又量了 CPU 一侧的 16 线程饱和吞吐（甲 48832371460.2–49032300970.9 B/s），没有代进该分项的形式；该分项依据段没引 E185（GC一批搬迁的挂钟拆分）；Q9 算不算问题单第 2 行的翻面量交主 agent；决策正文这一次没有改 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 不影响 | 2026-11-03 不受影响：只改复跑写法，结论、数与产物都不变；此前 2026-11-02 不受影响：只改复跑一节的写法（派哪台、编译目录与镜像目录照共享的树改写，命令补上 `--disk-model`），结论、数与产物不变；此前 2026-11-01 不受影响：只改复跑命令写法，结论、数与产物不变；此前 2026-10-26 不受影响：该分项射程补了一句一次 GPU 提交的实测挂钟量级（第 4 次重跑约 5.3 ms），依据段没引 E185（GC一批搬迁的挂钟拆分），条款判据不改；此前 2026-10-24 不受影响：补上第 3、4 次重跑产物点名，数与结论不变；此前 2026-10-23 不受影响：只作定义引用（GPU 结果要复核、GPU 与 CPU 两份实现）：第三段逐段对拍 CRC 的 32 行 `crcs_agree=true` 是第 1 次重跑第三段的装置作废检查，这一趟第三段不跑，不量运行时两份 GPU 核各算一次比对的复核政策，结论不变 |

```
