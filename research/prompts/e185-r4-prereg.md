# E185 第 4 次重跑登记：GC一批搬迁的挂钟拆分（CPU 模式四种写法当基线重判候选 A–D；卡 0 的轮次漂移控住暖机与频率重量；一批 CPU 按取样划分拆开）

写于 2026-10-24，装置改之前、这一趟一份产物都还没跑。不是差分重跑：问题单三行要的是三样新量，装置要加两个子命令、`scripts/experiment-apparatus/` 下加一份独立的重判脚本，原有子命令、原有产物行、原有判据与 V1–V154 一个字不改。下文写「r3 登记」指上一次登记 `research/prompts/e185-r3-prereg.md`（读的那一刻 sha256 `349105075c373e45806f7187fc12ce9e259f258483363b7564447714fbb00541`，680 行；它的第十二节 R3-1 至 R3-11 是装置作者的收严与补量），「r2 登记」「原登记」照 r3 登记文件头（`research/prompts/e185-r2-prereg.md` `7d74c089…`、`research/prompts/e185-preregistration.md` `c84bf038…`）。r3 登记的 R1–R8、原登记的 Q1–Q17、r2 登记的 N1–N15 这一趟不重判。

## 一、问题

英文名：gc_batch_wall_clock_split

（照抄 r3 登记第一节第 7 行；装置仍是 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`，不另起。重判脚本另起文件名 `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py`，英文名同上，前缀 `e185_r4_` 标这一趟。）

**主 agent 给的问题**（派发提示「关的是哪几条」逐字）：

```markdown
1. 问题单第 1 行：CPU 模式一批的主机 CPU 换四种写法当基线（今天、硬件 CRC 指令、区域扫描按身份查、两处都换）。装置里的替代写法只给计时用，不改 `crates/`；判据写死照第 3 次重跑的门槛算法逐点重算候选 A–D 的判定。验收：登记写明替代写法怎么造、与今天写法对拍逐字相同的检查、阳性对照、每个量对应问题单哪一行、什么数让判定翻面。
2. 问题单第 2 行：卡 0 的轮次漂移。验收：登记写死控住暖机与频率的做法、判「装置造的 / 卡的性质」的门槛，并写明第 3 次重跑第一段的「计时期间」规矩照用、装置在计时窗口写串行声明（`.claude/kb/checks-owed.md` C905（E185 装置的计时窗口不写串行声明））。
3. 问题单第 3 行：一种份额相加等于整次调用的按函数拆法。验收：登记写死容差与作废条款。
4. 登记里另写一节：要不要顺带关 C895（E185 重跑的比对对象没有单元表）——给出立单元表要改装置的哪几处；关不关由主 agent 定。
```

**问题单逐行抄**（`research/prompts/m4-gpugc-measure-r4-forks.md` 第 1–11 行，原样；文件 sha256 `2589bebad1d6aaf01e1d15d8f13c3d821b2665a644b189cd9941eb2e87555009`）：

```markdown
# m4-gpugc-measure-r4 问题单（2026-10-24）

七·2 设计轮第二轮判决 `research/prompts/m4-gpugc-design-r2-main-verification.md`「三、下一步」第 1 条要在入库装置上量的三样，一行一条：候选、翻面观测、够判条件、状态。是 E185（GC一批搬迁的挂钟拆分） 的第 4 次重跑；第 3 次重跑登记 `research/prompts/e185-r3-prereg.md`，产物在 `research/results/` 下 `*-r3-2026-10-24*`。格式照 `.claude/rules/three-way-inference.md`「交岔路时写岔路单，派实验时带上它」。

被测的仍是 `crates/` 今天的 GC 路径：`crates/singlefs-core/src/mounted_session.rs` 的 `run_compaction` → 区域扫描 `region_scan.rs` 第 204 行 `scan_region` → 选段与计划 `compaction_plan.rs` → 发布 `transaction.rs` 第 9676 行 `publish_relocation_batch`。CPU 侧 CRC-32C 在 `crates/singlefs-core/src/checksum.rs`（第 3 行写「实现用 slicing-by-8」）；区域扫描在 `region_scan.rs` 第 286–289、309–312 行对每条记录在全部映射条目上线性找。

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | CPU 模式一批的主机 CPU 取哪一种 CPU 写法当基线：甲 今天的写法（软件 slicing-by-8、线性找）／ 乙 CRC 换硬件指令、线性找不变 ／ 丙 CRC 不变、区域扫描换按身份查 ／ 丁 两处都换 | 四种基线下候选 A–D（`research/prompts/m4-gpugc-design-r1-forks.md` 第 1 行）各自的门槛与第 3 次重跑第一段的每次提交主机 CPU 相比，判定（不赔 / 赔 / 不稳定）变没变 | 四种基线在第 3 次重跑的 12 个点上各量到一批的主机 CPU（按阶段拆开），候选 A–D 的门槛照第 3 次重跑登记的算法逐点现算 | 开着 |
| 2 | 第 3 次重跑第一段卡 0 的轮次漂移是装置造的还是卡的性质：甲 暖机或频率造的（控住之后消失）／ 乙 卡 0 的真实性质（控住之后仍在） | 控住暖机与频率之后，卡 0 每格第 5 轮 ÷ 第 1 轮的中位（第 3 次重跑是 0.528，卡 1 0.958）；`unstable` 格数变化 | 卡 0 在第 3 次重跑第一段的全部格上重量，报每格的轮次比与判定，与卡 1 同样的跑法对照 | 开着 |
| 3 | 按函数拆开一批 CPU 的拆法：甲 第 3 次重跑第三段的重放法 ／ 乙 一种份额相加等于整次调用的拆法 | 第 3 次重跑第三段八行 `function_split_summary` 里七行各项份额之和 1.167–2.517、余项为负；新拆法下各项之和与整次调用之比 | 新拆法在第 3 次重跑第三段的四个 (f, P) 点上量到、各项之和与整次调用相差不超过登记写死的容差 | 开着 |
```

**这一趟补哪几行**：三行都补。第 1 行由「写法与拆分段」（量四种写法下一批的阶段 CPU）加「重判段」（重算门槛与判定）补；第 2 行由「控漂移段」补；第 3 行由「写法与拆分段」里 K = 8 那几个点补。三段的名字、在哪台、谁先谁后在第五节。

**读法写死**（第 1 行）：

| 词 | 口径 |
|---|---|
| 一批的主机 CPU（CPU 模式，按阶段拆开） | 照原登记第一节第 45–47 行与它 12.5 补的子项，一次 `run_compaction` 调用之后、计时窗口外照这一次调用的输入重放：甲段 `crc` = 区域扫描读到的每个单元整单元 CRC-32C；甲段子项 `release_check_crc` = 释放前读盘核那几份副本的整单元 CRC-32C；乙段增量 `scan_increment` = 整个 `scan_region` 调用 − 今天写法的 `crc`；丙段 `choose` = 签发那一步每次 `choose_region` 之和。每项 5 次重放取中位（装置 `median_stage_timing`，第 7672 行起），线程钟（`thread_cpu_nanoseconds`，第 716 行） |
| 写法甲（今天） | `crc` 与 `release_check_crc` 调 crates 的 `crc32_castagnoli`（`checksum.rs:42-61`，slicing-by-8）；`scan_region` 调 crates 那一份（`region_scan.rs:204`） |
| 写法乙（硬件 CRC，线性找不变） | `crc`、`release_check_crc` 换成装置自己的硬件指令 CRC-32C，两端各量：**单路**（每 8 字节一条 `_mm_crc32_u64`，尾巴逐字节 `_mm_crc32_u8`，初值 `0xFFFFFFFF`、出来取反）；**三路交错**（把一个单元切成三段，段长取 ⌊长度 ÷ 3 ÷ 8⌋ × 8、同值、余下的给第三段：32768 字节切成 10920、10920、10928，16384 字节切成 5456、5456、5472；三段在同一个循环里交错各走一条 `_mm_crc32_u64`，最后按「第一段的寄存器移过第二段长度的零字节、异或第二段，再移过第三段长度、异或第三段」合并，移位用开跑时为这四种段长预算好的 32×32 移位矩阵，每次合并一次矩阵乘向量；不用 PCLMULQDQ，也不许每次现算移位矩阵）。`scan_region` 仍调 crates 那一份，`scan_increment` 仍 = crates `scan_region` − 今天写法的 `crc`（乙段增量里那一份 CRC 是 `parse_data_unit` 调的 crates CRC，乙不改它，r2 拆法的前提「乙段增量 = 扫描减甲段」照用） |
| 写法丙（CRC 不变，区域扫描按身份查） | `crc`、`release_check_crc` 照甲；`scan_region` 换成装置里抄的一份（逐句照 `region_scan.rs:204-264` 的 `scan_region`、`:268-328` 的 `classify_allocation` 与 `:333-418` 的 `read_unit_header`，单元头解析调 crates 公开的 `parse_index_node`、`parse_data_unit`、`parse_packed_unit`、`mapping_key_for_node`、`mapping_key_for_data`），只把两处线性找换成查索引：`:286-289` 的「指着这一槽的映射条目」换成按（盘、槽）查一张索引，`:309-312` 的「头里那把 key 有没有映射条目」换成按 key 查一张索引。两张索引由这一版的映射条目（`version.mapping_entries`）建：（盘、槽）→ 条目号的表（每条条目两条位置项各进一次）、key → 条目号的表，都用 std 的 `HashMap`，条目号按映射条目的次序存，查出来再按次序排，保证与线性找交回的次序相同。建索引的 CPU 单独计时，两端各算：**计入**（每次重放建一次，加进扫描）、**不计**（只算查的那一份，等于「索引随版本增量维护、摊到每次接近零」的下界）。`scan_increment` = 丙的扫描（按端加或不加建索引）− 今天写法的 `crc` |
| 写法丁（两处都换） | `crc`、`release_check_crc` 照乙（两端），`scan_region` 照丙（两端），`scan_increment` 照丙；四种组合（单路／三路 × 计入／不计）各算一份 |
| 产物里的写法名 | 甲 `today`；乙 `hardware_crc_single`、`hardware_crc_three_way`；丙 `identity_index_with_build`、`identity_index_without_build`；丁 `both_single_with_build`、`both_single_without_build`、`both_three_way_with_build`、`both_three_way_without_build`（键名过 `scripts/gate/data-name-lint.py`，执行员为过 lint 改名的在第十二节记对照，不改量法） |
| 候选 A–D 的门槛 T_X | 照 r3 登记第一节「候选门槛 T_A、T_B、T_D」与 R3-9：装置 `candidate_thresholds_of_the_ledger`（第 9567 行）那个算法——按 (f, P, K, 轮) 分组、`kind=issue` 那一行算签发、其余算批、不滤 `valid`、各轮省下的取中位 ÷ 提交次数；A 省 `crc`（签发加各批），B 是 A 加签发的 `choose`，C 省 `crc + scan_increment`（签发加各批）加签发的 `choose`，D 省各批的 `release_check_crc`、提交次数 = 批数；A、B、C 的提交次数 = 批数 + 1。写法 X 下只换 `crc`、`scan_increment`、`release_check_crc` 三样的取值，`choose` 四种写法都不变（`choose_region` 不读映射条目、不算 CRC），提交次数不变 |
| 门槛来源（两端） | **r2 产物按比例换**（`ratio_on_the_r2_product`）：对 r2 第二段产物（`f70eb21b…`）的每一条 `name=stage` 行，三样各乘那一点的写法系数 φ_s,X(p)，再跑上一格那个算法；φ_s,X(p) = 这一趟写法与拆分段那一点各有效轮的（这条意图里进候选式子的各次调用 s_X 之和 ÷ 同几次调用 s_甲 之和）取中位，s ∈ {`crc`、`scan_increment`、`release_check_crc`}；X = 甲时 φ ≡ 1，门槛与 r3 逐字相同。**这一趟产物直接算**（`direct_from_the_r4_product`）：上一格那个算法直接跑在这一趟写法与拆分段的逐次调用读数上（三样取写法 X 那一列） |
| 门槛的机器（两端，照 r3） | `attack_ledger`：门槛原样；`machine_scaled`：门槛 × ρ，ρ = 量 c_rt 那一次本机的校准基线 ÷ 量阶段 CPU 那一次的校准基线（都取 `name=calibration_baseline … position=start` 的 `median_nanoseconds`）。门槛来源取「r2 产物按比例换」时分母是 r2 第二段产物那一行（与 r3 的 ρ_机器 同一个数）；取「这一趟产物直接算」时分母是写法与拆分段产物那一行。分子随 c_rt 来源：r3 第一段产物或这一趟控漂移段产物的那一行 |
| 候选 C 在写法甲下的门槛 | 照 R3-9 用 `out/g1.out` 原样那一个数（四位小数）；写法乙丙丁下用上面算法现算（r3 的 P7 已核两者在写法甲下到小数第 4 位相同） |
| c_rt 来源（两端） | **r3 第一段**：`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`（问题单第 1 行点名的）每一行 `name=host_cpu_per_submission_point` 的 `host_cpu_per_submission_milliseconds_per_round` 与 `floor_host_cpu_per_submission_milliseconds_per_round`；**这一趟控漂移段受控臂**：同名同义的那两列。第 1 行读 r3 第一段那一端（问题单原文）；另一端并列报，两端判定不同的记「随 c_rt 来源翻面」 |
| 判定 | 照 r3 登记 R1–R3：每轮 c_rt < 门槛判 `not_losing`（B 写 `not_losing_lower_bound`），≥ 判 `losing`；5 轮相同写那个，不全相同写 `unstable`；作废轮、门槛读不到照 R3-11 写 `rounds_insufficient`、`not_run`。c_floor 同法判 `floor_judgement`。「变没变」= 同一行（同一点、候选、卡、形态、等法、间隔、钟、门槛的机器）这一趟的判定与 r3 第一段产物那一行的 `judgement` 逐字比 |

**读法写死**（第 2 行）：

| 词 | 口径 |
|---|---|
| 轮次比 | 照第二轮判决第一节第 5 行 ③ 的取法：`name=host_cpu_per_submission_point` 里 `candidate=scan_checksum_only threshold_kind=machine_scaled clock=thread` 的行（每张卡 108 行：12 个点 × 常驻两种输入 × 三种等法 × 间隔 0，加共用输入三种等法 × 间隔 50 ms），每行第 5 轮 ÷ 第 1 轮的 `host_cpu_per_submission_milliseconds_per_round`，一张卡取中位（控漂移段里同口径、同键的行叫 `name=drift_control_point`，多一个 `arm=`）。另报格一级的同一个比（`name=submission_cell` 的 `thread_cpu_median_nanoseconds`，每张卡 80 格）与第 2–4 轮各 ÷ 第 1 轮的中位（轨迹） |
| 复现臂 | 照 r3 第一段的跑法原样量卡 0：起卡之后立刻开量（不另暖机），每格 10 次热身（`WARM_UP_SUBMISSIONS`，第 12569 行）、80 格按轮转、不绑核、只量卡 0 |
| 受控臂（「控住暖机与频率」的做法，写死） | ① **暖到稳**：每张卡第 1 轮之前，用探针格（n = 512、常驻、共用输入、阻塞等、间隔 0）每 20 次一窗连发，至少 60 秒，直到连着三窗的线程钟中位离三窗均值都不超过 3%，至多 600 秒；到顶没稳照样往下量，记 `warm_up_reached_the_cap=true`。② **每格暖到稳**：每格先 10 次热身，之后每 10 次一组接着热身，直到相邻两组的线程钟中位相差不超过 5%，热身至多 100 次（到顶照量，记次数）。③ **绑核**：整段把量的那条线程绑到一个固定的逻辑核（`sched_setaffinity`；取 `amd_pstate_prefcore_ranking` 最高的那一个，并列取号小的，读不到取允许集合里号最小的非 0 核），记核号与 `scaling_governor`。④ **频率读数**：每格前后各读一次那个核的 `scaling_cur_freq`、`cpuinfo_avg_freq` 与这张卡的 `clocks.sm`、`clocks.mem`、`pstate`、`temperature.gpu`、`power.draw`、`clocks_event_reasons.active`（并进 r3 每格本来就起的那一次 `nvidia-smi`，不多起一次进程），计时窗口外。⑤ **卡交替**：两张卡同时起着，每一轮里第奇数轮先量卡 0 全部 80 格、再量卡 1，第偶数轮先卡 1 后卡 0；一张卡的 80 格照 r3 轮转。本机没有不经 root 锁住 CPU 或 GPU 频率的办法（`sudo -n true` 要密码，第十三节），所以「控住频率」写成 ③④⑤：绑核去掉迁核、读数记下频率、交替让两张卡分到同样的时段 |
| 漂移探针格（阳性对照） | 每轮 80 格之后加两格（只在每轮末尾，不打乱 80 格的次序）：n = 8、常驻、共用输入、阻塞等、间隔 0；计时区间 ① 之前按线程钟自旋，**递减格**第 r 轮自旋 8.0 − 1.9 × (r − 1) 毫秒（8.0、6.1、4.2、2.3、0.4），**恒定格**每轮 2 毫秒。两条臂、两张卡都加（复现臂只有卡 0） |

**读法写死**（第 3 行）：

| 词 | 口径 |
|---|---|
| 整次调用 | 装置 `measure_one_intent`（第 3404 行）里第 3460、3462 行两次读线程钟之间那一次 `session.run_compaction(devices)`（第 3461 行），`cpu_nanoseconds` 就是它（r3 第三段 `call_cpu_nanoseconds` 同一个数）；只看 K = 8 的四个 (f, P) 点，签发一行、各批合起来一行，同 r3 的八行 `function_split_summary` |
| 拆法乙（取样划分） | 整次调用里按线程 CPU 取样：第 3460 行读线程钟之后立刻用 `timer_create(CLOCK_THREAD_CPUTIME_ID)`、`SIGEV_THREAD_ID` 指本线程、信号 `SIGPROF` 起一个周期 1 毫秒的计时器，首次到期取 (0, 1] 毫秒里按（点、轮、调用号）定种子的均匀随机相位；第 3462 行读线程钟之前撤掉。每次到期的权重 = 1 + `timer_getoverrun`。处理函数只往预先分好的缓冲里写：被打断处的 RIP、RSP、RBP、按帧指针链往外走（每一步要在本线程栈的上下界里、对齐 8、严格变大，至多 128 层）得到的返回地址。链走到锚点帧（`measure_one_intent` 在第 3461 行调用之前读到的本帧 RBP）算**直接成链**；走不到的，从 RSP 往上 16 KiB 里找第一个「当成帧指针走下去能走到锚点、且它上一格是本可执行文件代码段里的地址」的位置，从那里接着走，算**补回成链**；两种都不成的算**链断**。编译带 `-C force-frame-pointers=yes`（只给写法与拆分段那一次编译，第五节） |
| 取样落进哪一项 | 计时窗口外，用 `addr2line -f -i -C -a -e /proc/self/exe`（地址减掉 `/proc/self/maps` 里本可执行文件的装载基址）把每个地址展开成带内联的逻辑帧（返回地址先减 1）。从锚点往里找 `advance_compaction_by_one_publish` 那一帧（`mounted_session.rs:2989`，内联也认）；找不到的取样记 `outside_the_publish_step`；找到的，从它往里第一个落在下表名字上的逻辑帧定项（最外的那一个作数）：`TransactionOutput` 的 `clone` 且 `advance_compaction_by_one_publish` 那一帧的行号在 3002–3005 → `clone_before`，在 3165 或 3267 → `clone_after`；`central_mapping_as_the_compaction_reads_it` → `read_the_version_mapping`；`version_as_the_compaction_reads_it`（不在上一项里的部分）→ `read_the_version_other`；`scan_region` → `region_scan`；`choose_region` → `choose`；`compaction_publish_planned_for_the_region_scan` → `plan`；`publish_the_compaction_step_once`、`publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion`、`publish_the_planned_compaction_step_falling_back_on_the_region_check` → `publish`；别的 → `rest_of_the_publish_step`。逻辑帧里有 `??`、定不了项的记 `symbol_unknown`。与 r3 七项的对应：`clone_before`、`clone_after`、`read_the_version_mapping`、`read_the_version_other`（r3 的「钉住槽那一圈」）、`region_scan`、`choose` 各对一项，r3 的「发布与其余」对 `plan + publish + rest_of_the_publish_step + outside_the_publish_step` |
| 各项之和 | 除 `chain_broken`、`symbol_unknown` 之外各项的权重之和 × 1 毫秒 |
| 拆法甲（r3 第三段的重放法） | 照 r3 登记第五节「第三段怎么量」原样，在同一批调用上照跑（装置 `function_split_of_the_call`，第 14198 行）；各项之和 = 两次克隆、两样读这一版、`scan_region`、`choose` 的重放之和 |

未定项：

1. **第 2 行的三条线是读过 0.528、0.958 之后定的**：「漂移消失」取 [0.90, 1 ÷ 0.90]，「漂移仍在」取 < 0.80 或 > 1.25，之间不判（第六节 D1–D3）。0.90 与 1.10 同 r2 的校准作废线；0.80 落在 0.528 与 0.958 之间，是看过这两个数之后挑的，主 agent 认或改，在装置写之前定。
2. **第 3 行的容差**：各项之和 ÷ 整次调用落在 [0.95, 1.05] 判「是划分」（第六节 F1），计数核对 [0.97, 1.03]、成链 ≥ 95%（F2、F3）。5% 是政策线，读过 r3 的 1.167–2.517 与余项 −1.516256（问题单与第二轮判决）；主 agent 认或改。
3. **第 1 行读哪一端**：写法乙的单路／三路、写法丙丁的计入／不计建索引、门槛来源的两端、门槛的机器两端、c_rt 来源两端，都各报各的判定；一端不赔一端赔的点写「随 <旋钮> 翻面」，那一点第 1 行不当已判（照 r3 J4）。哪几个旋钮的两端主 agent 要先定读哪一端，在装置写之前定；不定就照这一句办。
4. **重判段走独立脚本**：第 1 行的重判是对已有产物的算术，按定义默认走 `scripts/experiment-apparatus/`（第五节「装置写在哪」）；它与 r3 的 Rust 判法不共用代码，靠停机 K-S3（写法甲下逐行复现 r3 第一段的判定）对上。主 agent 要它进入库装置的，在装置写之前改这一句。
5. **C895 顺带关不关**：第十一节末尾一小节列了要改的几处，由主 agent 定。

主 agent 定（2026-10-24，装置写之前）：

1. 第 2 行三条线照登记写的取（[0.90, 1 ÷ 0.90]、< 0.80 或 > 1.25、之间不判）。这三条线是读过第 3 次重跑的 0.528、0.958 之后定的，引这一段结论的判决与实验页要连这一句一起写。
2. 第 3 行容差照登记写的取（[0.95, 1.05]、计数核对 [0.97, 1.03]、成链 ≥ 95%）；同样是读过第 3 次重跑的 1.167–2.517 之后定的，引的时候连这一句一起写。
3. 第 1 行五个旋钮两端都报、各报各的判定；一端赔一端不赔的点记「随旋钮翻面」，不当已判、不挑一端。
4. 重判段走独立脚本 `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py`，照登记写的写法甲下逐行复现第 3 次重跑、复现不了停（K-S3）。
5. C895（E185 重跑的比对对象没有单元表） 这一次不关：要改的约 450–750 行，复跑那几条是重型，不为这次重跑扩范围。比对表那 5 行照第 3 次重跑的写法在 `.claude/gate.d/replay-comparison-units.tsv` 写「不登记的比对对象」，理由里写「欠在 C895（E185 重跑的比对对象没有单元表）」。
6. 第 3 行（按函数拆开的拆法）这一次不量，撤掉拆法乙整块（F1–F6、P-sample、FP 探针、K-S8、K-S9、V183–V198）与写法与拆分段里只为它加的取样。理由：第 3 行原为岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 10 行服务，第 10 行要的是「区域扫描换成按身份查之后一批的主机 CPU」，第 1 行的写法丙、丁直接量这个数；按函数拆的份额不再是任何一条岔路的输入。第 3 次重跑第三段的重放法份额照第二轮判决第一节记「不是划分」，不引作结论。（第 35 批执行员中途停在拆法乙的取样器上，报告 `~/.local/share/singlefs/work/m4-e185-r4-apparatus/result.md`。）

## 二、被测条款与它引的定义

两段，`scripts/kb/quote-kb.py` 整段抄（出口 `/home/fy5090/.local/share/singlefs/work/m4-e185-r4-designer/clauses.md`，回读逐字节一致，再整份追加到这里）：派发点名的 D26（后台整理与放置回收） 已定项 13 整节（第 7 条是第 1 行的尺：「收益是后台任务少占的主机 CPU 时间（CPU 模式下它的 CPU 时间减 GPU 模式下它的 CPU 时间……）」；第 1 行问的就是「CPU 模式下它的 CPU 时间」取哪一种 CPU 写法），与 D24（后台重活能不能卸给 GPU） 已定项 2 整节（第 2 条「不许进前台路径」是第二轮判决 G1-③、G8 的依据，第 3 条「GPU 路径与 CPU 路径必须是两份实现」与这一趟替代写法「与今天写法对拍逐字相同」同一个要求的 CPU 一侧）。已定项 13 今天在第 241 行（r3 登记抄时在第 232 行，中间多了几行，正文这一趟照现在的抄）。

定义逐字读过、不是答案：第 7 条只说「CPU 模式下它的 CPU 时间」，没说 CPU 模式取哪一种写法——这正是第 1 行的岔路，问法只有一种读法（四种写法各当一次基线，各报各的判定），不交回。

**出处 `.claude/kb/decisions/26-后台整理与放置回收.md:241-261`（整段抄，未转述）**

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

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:30-46`（整段抄，未转述）**

```markdown
#### 已定项 2：三条硬约束

**定案**：三条，不满足就不许做——

1. **绝不能成为必需。** 文件系统必须在没有 GPU 时完整工作、且语义完全相同。⇒ **GPU 是可选加速器，一个比特都不许进格式。**
2. **不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。
3. **它是一个新的失败域，卸出去的结果必须复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，两份在测试里对拍。运行时怎么复核由用到 GPU 的那条决策定：后台任务照 D26（后台整理与放置回收） 已定项 13 第 8 条，两份独立的 GPU 核各算一次比对，确认的交 CPU 改位图，连续几轮不确定的条目交 CPU 那一份判。

**射程**：第 1 条管的是格式，第 2 条管的是路径，第 3 条管的是验证，三条各自独立，满足不了任何一条就不许做。它们不定「值不值得做」，那是已定项 1 与已定项 3。第 3 条要的两份实现一份都没有（D24（后台重活能不能卸给 GPU） 暂缓，不排期）。

**依据**：

- 用户定案（GPU 是可选加速器；2026-10-21 运行时的复核改成两份独立的 GPU 核各算一次比对、连续不确定的条目交 CPU 判），原话在变更史。
- 无实验：三条都是政策与验证纪律的直接推论，没有可量的量；第 2 条引的 PCIe 往返量级是公开常识，本工程没测过前台路径上的 GPU 往返。

**欠**：无。

```

## 三、实现今天的样子

现查于 2026-10-24，主工作区，文件加行号（设计员的观测，不是结论）。

| 一步 | 在哪 | 看到的事实 |
|---|---|---|
| CPU 那份 CRC-32C | `crates/singlefs-core/src/checksum.rs:3`、`:9-35`、`:42-61` | 第 3 行注释「实现用 slicing-by-8」；`castagnoli_tables` 八张 256 项表；`crc32_castagnoli` 每 8 字节查八次表、尾巴逐字节。`grep -rn '_mm_crc32\|crc32_u64\|sse4.2\|sse4_2' crates/ --include=*.rs` 零命中：crates 里没有硬件指令写法（写法乙丁那一半装置自己写） |
| 区域扫描里的 CRC | `crates/singlefs-core/src/unit.rs:391`、`:397-407`、`:581-587` | `parse_data_unit` 调 `check_common_prefix`（头的宽校验和）与 `check_payload_checksum`（`:403` 对 `bytes[header_end..]` 调 `crc32_castagnoli`）：扫描读一个数据单元时自己算一遍差不多整单元的 CRC，这是 r2 拆法「乙段增量 = 扫描 − 甲段」的来由 |
| 区域扫描 | `crates/singlefs-core/src/region_scan.rs:204-264`（`scan_region`）、`:268-328`（`classify_allocation`）、`:333-418`（`read_unit_header`）、`:36-40`（`points_at`） | `scan_region` 先在全部分配记录上过一遍（`:215` 起）挑出 R 那块盘、未释放、起点在 R 里的，按槽排序；每条记录 `classify_allocation`：钉住、隔离各一次 `range` 查；`:286-289` 在全部映射条目上 `.iter().filter(points_at(盘, 槽))`；读头；`:309-312` 在全部映射条目上 `.iter().any(key == 头里的 key)`。`grep -n 'HashMap\|BTreeMap' region_scan.rs` 只命中第 12、145、422、452 行（`:145` 是 `first_two_movable_units_sharing_a_mapping_key` 里按可搬 key 查重，不是映射条目的查找）：crates 里没有按身份查映射条目的索引 |
| 解头与 key | `crates/singlefs-core/src/unit.rs:426`、`:507`、`:581`；`crates/singlefs-core/src/records.rs:462`、`:474` | `parse_index_node`、`parse_packed_unit`、`parse_data_unit`、`mapping_key_for_data`、`mapping_key_for_node` 都是 `pub fn`；`RegionScan`、`ScannedAllocation`、`AllocationInRegion`、`MovableUnit`、`RegionScanInconsistency`、`UnitHeaderUnreadable` 都是 `pub` 且派生 `PartialEq`（`region_scan.rs:28`、`:67-125`）：装置抄的那一份扫描交回的东西能与 crates 那一份整个用 `==` 比 |
| 选段 | `crates/singlefs-core/src/compaction_plan.rs:88`（`choose_region`）、`:735`（`region_scans_of_the_segments_without_an_unreleased_record`）、`:660` | `choose_region` 读分配记录、单元区、聚簇段、钉住与隔离的槽、已扫的段，不读映射条目、不算 CRC：四种写法下丙段不变 |
| 一次 GC 调用 | `crates/singlefs-core/src/mounted_session.rs:1956`（`run_compaction`）、`:2588`、`:2615`、`:2696-2704`、`:2722` | 先删除续做，再按一趟发布预算循环调 `advance_compaction_by_one_publish`（私有，`:2989`）；装置把 `publishes_since_the_last_user_change` 设成留一次预算（装置第 3456–3457 行），所以每次调用一次发布 |
| 一次发布里各步 | `mounted_session.rs:3001-3005`、`:3026`、`:3028-3037`、`:3093`、`:3165`、`:3267`、`:2807`（`pub`）、`:2835`（私有）、`:2895`、`:2922`、`:3355`（私有）、`:3277`（私有） | `:3002-3005` 克隆现行那一版；`:3026` `version_as_the_compaction_reads_it`（里面先调公开的 `central_mapping_as_the_compaction_reads_it`，再走一遍单元表收钉住槽、再走一遍映射条目收用户数据槽）；`:3028-3037` 闭包 `scan_of` 调 `scan_region`；签发那一步 `:3093` 选段循环；`:3165`（签发）与 `:3267`（批）`published.clone()` 再复制一份发成的那一版；发布走 `publish_the_compaction_step_once`（`:2895`）或 `publish_the_planned_compaction_step_falling_back_on_the_region_check`（`:3355`），计划走 `compaction_publish_planned_for_the_region_scan`（`:3277`） |
| 发布 | `crates/singlefs-core/src/transaction.rs:9676`（`pub fn publish_relocation_batch`）、`:9776`（`pub`）、`:9852`（`pub(crate) fn publish_compaction_step_in_the_session`） | 会话第一次试的那一条（`mounted_session.rs:3369` 调 `publish_compaction_step_in_the_session`）是 `pub(crate)`：装置不能在 crates 外把一次发布原样重放，所以第 3 行的拆法乙不靠重放，在真调用里取样 |
| 可克隆 | `crates/singlefs-core/src/allocator.rs:1817-1818`、`compaction_plan.rs:615-616` | `PoolAllocator`、`CompactionOfThisMount` 派生 `Clone`（这一趟不靠它） |
| 编译配置 | `Cargo.toml:18-23`；`.cargo/config.toml` | release 档 `opt-level = 3`、`debug = true`、`overflow-checks = true`、`debug-assertions = true`：带调试信息，`addr2line` 认得出函数与内联；`.cargo/config.toml` 只有 `[env] BINDGEN_EXTRA_CLANG_ARGS`，没有 `rustflags`。`grep -rn 'timer_create\|SIGPROF\|force-frame-pointers' crates/ scripts/` 零命中 |

**装置今天的样子**（改前基线）：主工作区 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` sha256 `4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea`、20497 行、`#[test]` 174 条，即 r3 登记 R3-1 那一版；HEAD 那一版仍是 `b3a50d50…`（r2 那一版，r3 的改动还没提交）。`crates/mutations.tsv` 里 `E185 V` 起头 154 行（V1–V154）。

| 装置里的一处 | 行 | 看到的 |
|---|---|---|
| 串行声明 | 第 380–500 行（`SerialDeclarations`、`Emitter::declare`）；第 52–55 行文件头 | 只在环境变量 `SINGLEFS_SERIAL_DECLARATIONS` 设了时写，没设不写。r3 第一段每格开头声明串行（第 13154 行），段末校准声明串行（第 13455 行）。起跑脚本 `scripts/experiment-apparatus/e185-r3-segment-run.sh`（137 行）不设这个变量（`grep -c SINGLEFS_SERIAL_DECLARATIONS` 为 0），本机那两段经它起，声明一行都没写：这就是 C905 |
| 一次调用怎么计 | `measure_one_intent` 第 3404 行；第 3459–3462 行 | 挂钟、线程钟前读，`session.run_compaction(devices)`，线程钟后读；设备是装置的计时包装（`ledger`），它在每次设备调用里读钟，那一份 CPU 在整次调用里 |
| 阶段重放 | `replay_the_stages_of_a_call` 第 7740 行；`median_stage_timing` 第 7672 行；`crc_stage_over_the_units` 第 2052 行；`time_a_pure_stage` 第 2037 行 | 甲段对记下的单元字节调 `crc32_castagnoli`、乙段整个 `scan_region`、丙段签发那一步的 `choose_region`，各 5 次取中位；装置另核停机条款第 4 条（重放的可搬 key 与这一批搬的 key 相同） |
| 门槛账 | `stage_ledger_rows_of` 第 9486 行；`saved_nanoseconds_of_the_intent` 第 9520 行起；`candidate_thresholds_of_the_ledger` 第 9567 行；`machine_ratio` 第 9700 行 | r3 的门槛算法，读 r2 第二段产物的 `name=stage` 行；`input_file_checks` 写死 `g1.out` 与 r2 产物两个 sha256 |
| r3 第一段 | `mod host_cpu_per_submission_segment` 第 12559 行起；`measure_the_cells_of_the_card` 第 13127 行；热身 10 次第 12569 行；每格前 `wait_until_the_card_has_no_other_compute_process` 起两次 `nvidia-smi`（第 12601 行起） | 一张卡 5 轮量完再量另一张；不绑核；不读频率 |
| r3 第三段 | `function_split_of_the_call` 第 14198 行；`function_split_one_intent` 第 14322 行；`run_the_function_split_segment` 第 14419 行 | 调用返回之后在调用之前那一版上各项单独重放、5 次取中位；「发布与其余」取余数 |
| 子命令分派 | `run_the_subcommand` 第 14979 行 | 今天有 `wall-clock`、`geometry-check`、`stage-split`、`gpu-submission`、`foreground-fsync`、`host-cpu-per-submission`、`recompute-disagreement`、`function-split` |

`scripts/experiment-apparatus/replay.sh` 里没有 E185 的复跑行（`grep -n -i e185` 零命中）。

与问题单对不上的：没有。问题单第 1 行写的「`region_scan.rs` 第 286–289、309–312 行」与「`checksum.rs` 第 3 行」逐字对得上上表。

## 四、跑之前已经存在的数

派发点名要读问题单、第二轮判决第一节与第三节、r3 登记全文，所以下面这些数在写判据之前就读到了。设计员自己定的线（第六节「门槛旁边写它从哪来」）逐条写了它看没看过这些数。

| # | 数 | 从哪读到 | 对判据的影响 |
|---|---|---|---|
| 1 | 第 2 行：卡 0 第 5 轮 ÷ 第 1 轮的中位 0.528（108 格）、卡 1 0.958（108 格）；第 3 行：r3 第三段八行 `function_split_summary` 第一行各项之和 0.888（`remainder_share_median=0.110173`），其余七行 1.167–2.517、余项为负（例 2.517 与 `-1.516256`） | 问题单第 10、11 行；`research/prompts/m4-gpugc-design-r2-main-verification.md` 第 21、22 行 | 第 2 行的「漂移仍在」线 0.80 是看过 0.528 与 0.958 之后挑在两者之间的（第一节未定项 1）；第 3 行的 5% 容差看过这几个数（未定项 2） |
| 2 | r3 第一段产物 `host_cpu_per_submission_point` 2684 行，`judgement` 那一列 losing 1137、not_losing 398、not_losing_lower_bound 95、unstable 1054；C 与 A 在同一格下每次提交主机 CPU 672 组逐字相同；辩方报终点校准比起点快 12.4% | 第二轮判决第 21 行 | 2684 是重判段要读的行数（第七节乙类核的是其中线程钟那 1776 行，另算的，与 2684 不比）；1054 是第 2 行「unstable 格数变化」的对照，D5 只报 |
| 3 | 攻方：单路 SSE4.2 比 slicing-by-8 快 3.98 倍（它的探针，没进 kb）；把两处换成更快写法之后间隔 50 ms 下 C 一格不赔都没有（它的模型）；辩方：把乙换成 P = 1024 的量，C 卡 0 的 not_losing 从 52/54 掉到 25/54（代理数）；一次提交挂钟 5.46–8.05 ms；G8 的 5.50–14.76 ms、4.5%–118%；g8 模型 200 拍 213 次；`merging_shape` 432 行只 1 行跨意图；本机第三张卡是 AMD 集显 | 第二轮判决第 24、28–42 行 | 3.98 倍让写法乙「硬件 CRC 比今天快几倍」有了量级（第 9 行推的）；别的数不进判据。这一趟就是照判决「副本上的数要在入库装置上重做」把前两样在入库装置上重做 |
| 4 | r3 登记里的数：`out/g1.out` 12 个点的门槛 0.2159–0.7214 ms、`saved_upper_ms` 0.591–3.961、`submissions_per_intent`（3、2、2、3、2、2、7、4、3、7、4、3）；正推的「一批 CPU 13.4 ms 或 111.1 ms」；r1 第二段准备 4188.11 秒；线程钟一次前进 170 ns 中位；两张卡的链路（5.0 x16、4.0 x8）；r3 的挂钟估算 | `research/prompts/e185-r3-prereg.md` 第 155–167、295、301、679 行 | 门槛算法与提交序列照用；13.4、111.1 ms 与 4188.11 秒只用来估第十四节的挂钟与第 3 行每个点的取样数 |
| 5 | r3 第一段（带 `-2` 那一套）从输入清单写好到产物最后一次改动 1769 秒（含编译） | `stat -c '%W %Y'` 两份文件的时间差（第十三节命令；没读正文） | 只用来估控漂移段的挂钟 |
| 6 | 两台机器：本机与第二台都是 AMD Ryzen 9 9950X、32 个逻辑核、`scaling_governor` = performance；`perf_event_paranoid` 都是 4（不经 root 用不了 `perf`）；内核 `CONFIG_HZ=1000`、`CONFIG_HIGH_RES_TIMERS=y`，两台 `nohz_full` 都为空；两台都有 `sse4_2` 与 `pclmulqdq`，都装着 `nm` 与 `addr2line`（`/usr/bin`）；本机 `sudo -n true` 要密码；两张卡空闲时 `pstate` P8、`clocks.sm` 195 与 180 MHz，`clocks.max.sm` 3105 与 3090 MHz，persistence mode 开着 | 第十三节命令（第二台的主机名不抄） | 第 3 行不能用 `perf`，改在装置里自己取样（第一节）；取样周期取 1 ms 是因为线程 CPU 计时器按 1000 Hz 的节拍判到期，更短的周期只多出 `overrun` 权重、不多出取样的位置；第 2 行锁不住频率，写成绑核、读频率与卡交替；两台同一种 CPU，写法乙的硬件指令两台都有，三路合并不用 PCLMULQDQ |
| 7 | 自己算的锚点：三路切段 32768 → 10920、10920、10928，16384 → 5456、5456、5472；「第 i 字节 = i mod 251」的 32768 字节 CRC-32C `0x2ad60827`、16384 字节 `0xeafca51d`，三路合并与整段相同；「123456789」`0xe3069283`；漂移探针递减格 8.0、6.1、4.2、2.3、0.4 ms，c_rt 0.05、1、3、6、14、30 ms 时第 5 轮 ÷ 第 1 轮是 0.0559、0.1556、0.3091、0.4571、0.6545、0.8000；r3 第一段线程钟那一半的点行 1776 行，每张卡候选 A、`machine_scaled` 108 行；取样的份额标准误（130 个样本、份额 0.1 与 0.3：0.0263、0.0402；2000 个：0.0067、0.0102）；随机相位计数的相对标准差（10 次调用共 130 ms：0.00702；20 次共 2220 ms：0.00058） | 第十三节 python 命令与输出 | 第七节乙类锚点与第六节的分辨率 |
| 8 | 推的（没量）：写法乙三路比 slicing-by-8 快约 8–12 倍（`crc32q` 每拍一条、延迟 3 拍，三路接近每拍 8 字节；slicing-by-8 每字节约 1–2 拍），单路约 3–4 倍（与第 3 行的 3.98 同量级）；一个 32 KiB 单元今天约 11–22 µs、三路约 1–2 µs。线性找：R 里至多 32 个码 1 单元，每条记录在全部映射条目上走两遍；P = 8192 时映射条目约 8200 多条，一次扫描约 24 × 2 × 8200 次比较 ≈ 0.4–1 ms；P = 1024 约 0.05–0.1 ms；建两张索引每条约 50–100 ns，P = 8192 约 0.4–1.6 ms、P = 1024 约 0.05–0.2 ms：P = 8192 时「计入建索引」那一端省下的可能与建索引花的同量级，「不计」那一端省下线性找的全部 | 设计员推的，依据是第 3 行的 3.98 倍与 `region_scan.rs:286-312` 的循环形状 | 这是 3a 第 ② 步要的「被测臂上预期的差」：写法乙下 A、D 的门槛预计降到今天的约 1/10–1/3，C 降的多少取决于乙段增量占 C 门槛几成；丙「计入」那一端在 P = 8192 可能与甲差不多、「不计」那一端明显低。差都远大于线程钟分辨率（170 ns）与重放计时的重复粒度；判据是绝对值比较（c_rt < T_X），不在哪一臂上退化成零分母。所以两端都登记 |
| 9 | 推的（没量）：取样划分里链断的份额。Rust 这一侧带帧指针；std 里没在这一次编进来的那几个函数与 glibc（`malloc`、`memcpy`）不带，打断在 glibc 叶函数里时 RBP 仍是调用者的帧、链照样走到锚点；打断在 `_int_malloc` 这类把 RBP 当通用寄存器用的函数里才断，要靠「补回成链」。克隆整版要大量分配，断的份额不知道（推 1%–10%） | 设计员推的 | 第 3 行最可能的失败来源；F3 单列「直接成链」「补回成链」「链断」三样，P-sample 的分配一项专门照这一支；成链 < 95% 走第十条 L6 |

第 3a 步第 ③ 样（登记点上要触发的事件，只对「触发是前提」的做）：

- **取样计时器到期**：每次调用的 CPU ≥ 约 13 ms（第 4 行），周期 1 ms，每次调用期望取样数 = CPU ÷ 1 ms ≥ 13；四个 (f, P) 点每点 5 轮里签发与各批共 10–20 次调用（f = 0.25：签发 + 1 批；f = 0.75：签发 + 3 批，K = 8，r3 第一节「一个点的提交序列」），一点合计约 130–2200 个取样。会发生。
- **12 个点上都有签发与批**：`out/g1.out` 12 行的 `submissions_per_intent` 是 2–7（第 4 行），r2 第二段在这 12 个点上都有签发与批；写法与拆分段照 r2 第二段同一份配方与几何核造池（第五节），几何核不过照原条款作废重造。会发生；没造成的点照 r2 的「几何没造成」条款判，不当结果。
- **硬件 CRC 指令在**：两台都有 `sse4_2`（第 6 行）；装置开跑照 `is_x86_feature_detected!("sse4.2")` 再核一次，不在退 78。
- **索引里有东西**：P ≥ 1024，映射条目 ≥ 1024 条；每次重放报 `identity_index_entries`，为 0 照「读不到」作废那一次。

设计员自己定的线（第六节）与上面的关系：D1–D3 的 0.80 看过第 1 行；F1 的 5% 看过第 1 行；D4 的 1.5 与 2、F2 的 3%、F3 的 95%、F6 的 2% 与 5%、P-sample 的 ±0.03、PW1 的 [0.9, 1.1]、PW2 的 [1.8, 2.2] 没有引用上面任何一个数（后两者照 r2 与 r3 的阳性对照线）。

## 五、臂、阳性对照、真实基线

### 装置写在哪（写死）

- **控漂移段、写法与拆分段：入库装置**。仍是 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`，加两个子命令 `host-cpu-drift-control`、`cpu-writings-and-function-partition`，原有八个子命令、原有产物行、原有判据与 V1–V154 不动。理由：第 2 行问的是这台机器上 `wgpu` 30.0.1 经 checker 档起卡之后一次提交的主机 CPU 随轮次怎么走，只能在 r3 第一段那一套 GPU 栈上量；第 1 行要的是 crates 这份 GC 代码在同一批真调用上各阶段的 CPU、第 3 行要的是 crates 这份代码一次真调用的 CPU 按函数怎么分，都要直接调 crates 的函数、在 crates 的调用里取样。替代写法（硬件 CRC、按身份查的扫描）只住在装置里、只给计时用，不改 `crates/`，每一次重放都与 crates 那一份对拍（停机 K-S1、K-S2）。**改前基线取主工作区今天那一版 sha256 `4368c0e7a7870a09810fa5667173f9cf14625f4d64c4e0287ee10ec7ef80e6ea`**（第三节）。
- **重判段：`scripts/experiment-apparatus/` 下的独立脚本** `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py`（带同名 `_test.py` 自证与 `.mutations.tsv` 变异表，形照同目录 `e185_r1_compare.py`）。它只读产物做算术，不调 crates、不与装置共用代码（`.claude/rules/implementation-first.md` 第 4 条）；它与装置的 Rust 判法是两份写法，靠停机 K-S3（写法甲、门槛来源「r2 产物按比例换」、c_rt 来源 r3 时逐行复现 r3 第一段产物的判定）对上。

入库装置这一趟新读 crates 的东西，逐个改成本地常量加回比、或按行为对拍，不直接引：

| 读什么 | 怎么读 | 对不上 |
|---|---|---|
| `singlefs_format::DATA_UNIT_BYTES`（32768）、`NODE_BYTES`（16384） | 已有本地常量 `LOCAL_DATA_UNIT_BYTES`、`LOCAL_NODE_BYTES`（第 112、114 行）与它们的回比（`constants_disagreeing_with_the_crates`，第 149 行），三路切段的段长由它们算 | 停机 J-S2（r3 原有） |
| `UNIT_CLASS_DATA`、`UNIT_CLASS_INDEX_NODE`、`UNIT_CLASS_PACKED`（`crates/singlefs-core/src/unit.rs:17-19`，1、2、3） | 抄的那份扫描用到的类标签另立本地常量，进 `constants_disagreeing_with_the_crates` 回比 | 停机 J-S2 |
| `region_scan.rs` 的分类逻辑（`:204-418`） | 不引常量，按行为对拍：每次重放，抄的那份扫描（两张索引）交回的 `Result<RegionScan, RegionScanRefusal>` 与 crates `scan_region` 交回的整个 `==` | 停机 K-S2 |
| `crc32_castagnoli`（`checksum.rs:42-61`） | 按行为对拍：每次重放，单路、三路两种硬件写法对每个单元（扫描读到的与释放前读盘核那几份）的值与 crates 那一份逐个相同 | 停机 K-S1 |
| `mounted_session.rs` 第 3002–3005、3165、3267 行（取样分项认的克隆调用处） | 本地常量记这三处的行号与那一行的原文（`.clone();`、`*current = PoolVersion::of_a_version_of_trees(published.clone());` 两种），开跑时 `include_str!` 读 crates 那份源码逐行核原文 | 停机 K-S8 |
| 取样分项认的函数名（第一节「取样落进哪一项」那张名单） | 本地常量；开跑时对编出来的可执行文件跑一次 `nm -C`（计时窗口外），名单里每个名字至少命中一处（内联掉的认 `addr2line -i` 的内联帧：在 P-sample 之后的头一次真调用里核每个名字在取样里出现过，没出现的那一项报 `name_never_sampled`，不停） | 名字一处都找不到：停机 K-S9 |

### 三段（按岔路排序；每段交回之后主 agent 对着问题单判续不续）

| 段 | 子命令 / 脚本 | 在哪 | 补问题单哪一行 | 补完那一行够不够判 |
|---|---|---|---|---|
| 控漂移段 | `host-cpu-drift-control --cards 0,1 --stage-split-product <r2 第二段产物> --attack-ledger <g1.out>`（`gpu` 特性；没编特性退 78 报「本次未跑」，同 r3 第一段） | 本机 | 第 2 行（D1–D7）；另给第 1 行的「c_rt 来源」另一端 | 第 2 行：复现臂卡 0、受控臂两张卡都有轮次比与判定（含「不稳定」）就够判 |
| 写法与拆分段 | `cpu-writings-and-function-partition --point 0.25:1024 --point 0.25:8192 --point 0.75:1024 --point 0.75:8192`（不带 `gpu` 特性；这一次编译带 `-C force-frame-pointers=yes`） | 第二台（同 r2 第二段与 r3 第三段，真盘不用） | 第 1 行的阶段 CPU（B1、B2）；第 3 行（F1–F6） | 第 3 行：四个 (f, P) 点签发与各批各有 F1 判定（含「不稳定」）就够判；第 1 行还差重判段 |
| 重判段 | `python3 scripts/experiment-apparatus/e185_r4_baseline_rejudge.py --host-cpu-product <r3 第一段 -2 产物> --drift-control-product <控漂移段产物> --stage-split-product <r2 第二段产物> --writings-product <写法与拆分段产物> --attack-ledger <g1.out>` | 本机，前两段都交回之后 | 第 1 行（B3–B6） | 九种写法 × 两种门槛来源 × 两种门槛的机器在 12 个点上都有门槛、r3 第一段每一行都有重判结果，就够判；控漂移段没跑成时只报 c_rt 来源 r3 那一端，第 1 行照样够判（问题单原文点名 r3 那一端） |

控漂移段与写法与拆分段在两台机器上，可以同时跑（计时窗口各在各的机器上）；写法与拆分段先起，它的 `multi-host-run.sh` 拷完树、打出第二台开跑那一行之后再起控漂移段，控漂移段计时的时候本机不跑拷树。写法与拆分段末尾取回产物（几个文件的 rsync，秒级）若落在控漂移段的计时里，照控漂移段每格前后的校准比判「被挤」，不另立条款。重判段只读产物，几秒。

**计时期间**（r3 登记第五节「跑法」最后一条照用，原文：「第一段起跑前主 agent 告诉别的会话不往本机两张卡派活（清场之外的那一半）；第一段每格前后的校准比与线程数就是这一段的「机器被别的活挤」读数。」）：控漂移段起跑前主 agent 告诉别的会话不往本机两张卡派活、本机不起重活；每格前后的校准比、线程数与新加的频率读数就是「被挤」的读数。写法与拆分段照 r2 第二段：经 `gpu-quiesce-run.sh` 包着的 `multi-host-run.sh --peer` 起，第二台计时期间不跑别的重活。

**串行声明（C905）**：两个新子命令都照写，执行员落地：① 段一开头（时钟自检之前）声明串行，理由照 r3 登记第十四节「串行」那一句的意思写（控漂移段：「E185 第 4 次重跑控漂移段：一次 GPU 提交在发起它的那条线程上花的 CPU，要串行量，别的线程抢 CPU 与 PCIe 会混进来」；写法与拆分段的计时窗口照原有的 `TimingWindow` 理由）；② 每个计时窗口（段开头校准与 P8、P1、暖到稳、每格、漂移探针格、段末校准；写法与拆分段的每次调用、每次重放、P-sample、取样开销探针）进之前声明串行，出来之后若接着的是并行的准备或多线程的活，声明并行；段末声明并行；③ 起跑脚本在 `SINGLEFS_SERIAL_DECLARATIONS` 没设时把它设成这一趟运行目录里的 `serial-declarations` 并 `export`（`detached-run.sh` 与 `multi-host-run.sh --peer` 设了的照它的），跑完把那份文件拷进 `research/results/e185-r4-serial-<段>-<日期>.txt`；④ 判别力：一条单测用临时文件跑控漂移段的一个假格（假卡、假钟），声明文件里要有一行串行、理由 ≥ 8 个字，再把那份文件交 `python3 scripts/hosts/core_usage.py judge --incoming <假样本> --series <假样本> --declarations <它>`（照 `core_usage.py` 文件头的用法，在单测之外的自证里跑），判出来要标出这段串行；变异 V181 去掉 ① 那一句，单测红。

### 第 1 行的臂（「怎么做」推得出「做完会怎样」：门槛 T_X 由那几项阶段 CPU 相加，各臂改的是哪一项、怎么改）

| 臂 | 怎么做（支持它的人认的样子） | 做完会怎样 |
|---|---|---|
| 甲 今天的写法（支持「拿产品今天的 CPU 模式比」的人认的） | CPU 模式就是 crates 今天这份：slicing-by-8、线性找；门槛照 r3 原样 | 门槛与 r3 逐字相同，判定与 r3 第一段逐行相同（这是 K-S3 的核法，不是结果） |
| 乙 硬件 CRC（支持「CPU 模式该用现成的硬件指令」的人认的） | 两项 CRC 换成 `crc32q` 指令；三路交错是这类实现的常见写法，单路是最直的写法，两端都量 | A、D 的门槛只剩硬件 CRC 那一份（一份 CRC 变快多少，A、D 的门槛就小多少）；B 多一份不变的 `choose`；C 的门槛少了 CRC 那一块，乙段增量不变 |
| 丙 按身份查（支持「线性找是没优化的写法」的人认的） | 两处线性找换成两张 `HashMap`；建索引的 CPU 两端都算：计入（每次扫描现建，上界）、不计（索引随版本增量维护的下界） | 只有 C 的门槛变：乙段增量少了线性找、多了建索引（计入那一端）；A、B、D 不变 |
| 丁 两处都换 | 乙与丙合起来，四种组合 | A、D 照乙；C 两样都少 |

各臂量的是同一批调用、同一批记下的单元字节（写法与拆分段每次调用之后在计时窗口外一并重放），哪一点落在哪一边是结果。

### 第 2 行的臂

| 臂 | 怎么做（支持它的人认的样子） | 做完会怎样 |
|---|---|---|
| 甲 装置造的（暖机或频率） | 支持它的人认的反证就是把暖机与频率控住再量（第一节「受控臂」①–⑤）；同一趟里先照 r3 原样量一遍卡 0（复现臂），证明这一天这台机器上漂移还在 | 复现臂卡 0 的轮次比照旧远小于 1，受控臂卡 0 回到 1 附近 |
| 乙 卡 0 的真实性质 | 支持它的人认的：同样控住之后再量，若卡 0 照样一轮比一轮快（或慢），那就是卡 0 在这份负载下的样子，与卡 1 对照（卡 1 同一套跑法） | 受控臂卡 0 的轮次比仍远离 1，卡 1 在 1 附近 |

两条臂量的是同一组格（r3 第一段的 80 格 × 5 轮），哪一条成立是结果；第三种结局「这一天复现不出」（复现臂卡 0 也在 1 附近）照第十条 L5 记。

### 第 3 行的臂

| 臂 | 怎么做（支持它的人认的样子） | 做完会怎样 |
|---|---|---|
| 甲 重放法（r3 第三段原样） | 每项在计时窗口外单独重放、5 次取中位，没有取样噪声，量的是那个函数单跑时自己的 CPU | 各项单跑的 CPU；与整次调用相比的「和」不受约束（重放与调用里的缓存、分配器状态不同） |
| 乙 取样划分 | 在真调用里按线程 CPU 每 1 ms 取一个样，按调用链把每个样分到恰好一项（第一节「取样落进哪一项」）；每个样只进一项，所以各项之和 = 落进各项的样本数 × 1 ms，与整次调用的差只来自计数误差与链断、认不出的那几样 | 各项之和 ÷ 整次调用 ≈ 1，偏多少由 F2（计数）、F3（成链）两行单独看得出 |

### 阳性对照（每条臂都跑）

| # | 对哪几条臂 | 对照 | 判得出的样子 |
|---|---|---|---|
| PW1 每种写法的计时罩得住 | 第 1 行九种写法里的五个计时件（今天的 CRC、单路、三路、抄的扫描查索引那一份、建索引）各自，写法与拆分段每个点每轮第一批（同 r2 第二段 `with_the_controls` 那一批） | 计时区间里加 2 ms 按线程钟自旋 | 增量在 2 ms 的 [0.9, 1.1] 倍（同 r2、r3 的 2 ms 对照线） |
| PW2 每种写法分得出两倍的活 | 同上五个计时件 | 同一批输入喂两遍（单元表接两份、扫描连调两次） | 两遍 ÷ 一遍落在 [1.8, 2.2] |
| PW3 替代写法算对 | 单路、三路、抄的扫描 | 每次重放与 crates 那一份对拍（K-S1、K-S2）；另在开跑时核「123456789」→ `0xe3069283`、i mod 251 的 32768 与 16384 字节 → `0x2ad60827`、`0xeafca51d`、0 到 64 字节随机长度（种子写死）两种写法与 crates 逐个相同；抄的扫描另在一份小池（单测里手搭）上把钉住、隔离、死、可搬、头读不了、类标签不对、key 与位置不对、跨边界、两条同 key 九种结局各走一次，与 crates 相同 | 一个都不差 |
| PR1 重判复现 r3 | 重判段 | 写法甲、门槛来源「r2 产物按比例换」、c_rt 来源 r3：每一行的门槛、`judgement_per_round`、`judgement`、`floor_judgement` 与 r3 第一段产物那一行逐字相同 | 2684 行一行不差（不同走停机 K-S3） |
| PR2 重判分得出门槛动了 | 重判段的判定函数 | 单测：把门槛挪到同一行两个量到的值之间，判定由一种变成另一种；九种写法的门槛账各喂一份合成产物，A、B、C、D 只在该变的那几项上变 | 判定翻、该变的变、不该变的不变 |
| P1（r3 原样） | 控漂移段两条臂、每张卡、全部七种提交形态 | r3 第五节 P1 原样（2 ms 自旋与 2 ms 睡） | r3 原样 |
| P-drift 漂移看得出 | 控漂移段两条臂、每张卡（复现臂只有卡 0） | 第一节「漂移探针格」递减格 | 递减格的第 5 轮 ÷ 第 1 轮 ≤ 0.80（c_rt ≤ 30 ms 时第七节乙类算出 ≤ 0.80）；恒定格只报 |
| P2、P3、P4、P7、P8、J-S1（r3 原样） | 控漂移段 | r3 第五节原样 | r3 原样 |
| r3 第三段的 2 ms 对照（原样） | 第 3 行甲 | r3 第五节「附带段的阳性对照」原样（克隆、读这一版两个重放各加 2 ms） | r3 原样 |
| P-sample 取样划分分得清 | 第 3 行乙 | 写法与拆分段开头与末尾各一遍：装置里一棵假调用树，锚点函数下面依次调 `item_alpha`（线程钟自旋 3 ms）、`item_beta`（1 ms）、`item_gamma`（6 ms）、`item_delta`（1 ms 的分配与释放，大小 16 字节到 64 KiB 随机、种子写死，走 glibc 的 `malloc`／`free`），四个都 `#[inline(never)]`，连做 300 遍（约 3300 个取样，份额 6/11 的标准误约 0.0087，±0.03 约 3.4 个标准误）；取样、补链、符号化、分项与真调用同一套代码，名单换成这四个名字 | 四项份额各在 3/11、1/11、6/11、1/11 的 ±0.03 里；计数（全部权重 × 1 ms ÷ 线程钟）在 [0.97, 1.03]；成链（直接加补回）≥ 95%，`item_delta` 那一项另报直接成链与补回成链各几成 |
| FP 探针 | 第 3 行乙 | 开跑时三层 `#[inline(never)]` 的探针函数自己走一遍帧指针链 | 三个返回地址符号化之后依次是这三个函数；不是就说明这一次编译没带帧指针，第 3 行乙整段作废（第十一节） |

### 真实基线

第 1 行：写法甲（crates 今天这份 CPU 模式），门槛与 r3 相同。第 2 行：r3 第一段那一趟（产物 `…host-cpu-per-submission-r3-2026-10-24-2.out`），复现臂是它在同一天同一台上的重做。第 3 行：拆法甲（r3 第三段的重放法，同一批调用上照跑）。

### 控漂移段怎么量（`host-cpu-drift-control`）

- **段开头**（不绑核）：r3 第一段段开头原样（时钟分辨率自检、校准基线 21 遍与 P8、读 r2 第二段产物与 `g1.out` 算候选门槛（P7、停机 J-S3、J-S4）与 ρ、起卡之前核本进程只剩一条线程）；声明串行。
- **复现臂**：起卡 0（`gpu_cards_by_priority(&[(0, 4096)])`），编核、开常驻缓冲、P2 与 J-S1、P1，然后立刻 5 轮；每轮 80 格照 r3 轮转、每格照 r3（前校准 → 10 次热身 → 计时提交（间隔 0 的格 100 次、50 ms 的格 50 次）→ 后校准；作废与重试照 r3 的 `ATTEMPTS_PER_ROUND_AT_MOST` 与 R2-2），每轮末尾两格漂移探针。每格前后读频率（第一节受控臂 ④ 的那几样，并进 r3 每格本来就起的 `nvidia-smi`）——复现臂只多读、不多起进程，别的一样不改。
- **受控臂**：绑核（第一节 ③）；起卡 1，编核、开常驻缓冲、P2 与 J-S1；两张卡各跑一遍 P1；暖到稳（第一节 ①）先暖卡 1、再暖卡 0（第 1 轮先量卡 0，后暖的先量）；然后 5 轮，第奇数轮卡 0 的 80 格加两格探针、再卡 1 的，第偶数轮反过来（第一节 ⑤）；每格照 r3 的格，只把 10 次热身换成第一节 ② 的「暖到稳」。
- **算点与判**：两条臂各自照 r3 第一段的算点与判定（R1–R5、J1–J11、R3-11 的翻面口径）出点行；每条臂、每张卡算第一节「轮次比」与 D1–D5（第六节）。
- **段末**：段末校准（r3 原样）；声明并行。

### 写法与拆分段怎么量（`cpu-writings-and-function-partition`，第二台）

- **造池、几何核、全局准备队列、每条意图前后的校准与作废**：照 r2 第二段原样（原登记 12.10、12.11，r2 登记 N1–N15、R2-1 至 R2-4），四个 (f, P) × K ∈ {4, 8, 16} × 5 轮；r2 第二段的饱和吞吐、丙段盘大小敏感性这一趟不跑（对不上问题单任何一行）。
- **段开头与段末**（计时窗口里，声明串行）：FP 探针；P-sample；取样开销探针（F6：同一份 16 MiB 软件 CRC-32C 校准活，取样计时器开着与关着交替各 10 遍，取两边中位之比）。开跑核 `is_x86_feature_detected!("sse4.2")`、`addr2line` 与 `nm` 在 PATH 上，任何一样不在退 78。
- **每次调用**：K = 8 的调用，第 3460 行读钟之后开取样计时器、第 3462 行读钟之前撤（第一节「拆法乙」），处理函数写进预分好的缓冲（一次调用至多 4096 个取样 × 每个 128 个返回地址，超了记 `sample_buffer_full`、照「读不到」作废那一次调用）；K = 4、16 的调用不开计时器。
- **每次调用之后、计时窗口外**：① r2 第二段的阶段重放原样（今天的写法），打 r2 原格式的 `name=stage` 行与停机条款第 4 条；② 同一批记下的输入上量八样新的计时件：`crc` 与 `release_check_crc` 的单路与三路、抄的扫描（查索引那一份、不含建索引）、建索引，各 5 次取中位（`median_stage_timing`），打 `name=cpu_writing_stage`；③ 对拍 K-S1、K-S2；④ 每个点每轮第一批另做 PW1、PW2；⑤ K = 8 的调用：r3 第三段的重放（拆法甲，打 r3 原格式的 `name=function_split` 行），再把这一次调用的取样符号化、分项，打 `name=sampled_partition_call`。
- **每个点收尾**：九种写法的系数 φ（`name=writing_factor`）；K = 8 的点：签发与各批各一行 `name=sampled_partition_summary`、r3 原格式的 `name=function_split_summary`、F1–F5 的判定行。

### 重判段怎么算（`e185_r4_baseline_rejudge.py`）

- 读五份输入、核 sha256（K-S5）；读 r3 第一段产物 2684 行点行；从 r3 与 r2 的 `calibration_baseline … position=start` 现算 ρ，与 r3 产物里打出的 `machine_ratio=` 逐位比（K-S4）。
- 九种写法 × 门槛来源两端 × 门槛的机器两端：每个点四个候选的门槛（`name=baseline_threshold`）；乙段增量为负的点那几种写法的门槛写 `not_run reason=scan_increment_below_zero`（读不到不当 0）。
- c_rt 来源两端 × 上面每一种门槛：对每一行点行重判（`name=baseline_rejudge_point`，带 r3 那一行的判定与 `changed=`），再按（写法、门槛来源、c_rt 来源、候选、卡、门槛的机器、钟）出 `name=baseline_rejudge_summary`，按点出 `name=baseline_rejudge_point_summary`，按旋钮出 `name=baseline_knob_flip`。PR1 先跑，不过就停（K-S3）。

### 跑法（照现行脚本写；主 agent 起跑前核这几份脚本有没有变、两段的清场包装能不能同时起，变了照新写法起、不能同时就先后起，记进第十二节一行）

- **起**：执行员照 `scripts/experiment-apparatus/e185-r3-segment-run.sh` 的形写 `scripts/experiment-apparatus/e185-r4-segment-run.sh <段>`（写输入清单 → 编 → 跑 → 产物写进 `research/results/` → 删编译目录；装置 sha256 不等退 3；`cargo tree` 见 `lookup-work-counters` 退 6；另加第五节「串行声明」③）。控漂移段编 `--features gpu`；写法与拆分段编时设 `RUSTFLAGS='-C force-frame-pointers=yes'`、不带特性。
  - 写法与拆分段（第二台，先起）：`bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/multi-host-run.sh --peer --memory-cap 46G --label e185-r4-cpu-writings-<日期> --fetch research/results/<产物名>… -- env E185_RUN_DATE=<日期> E185_MEMORY_CAP=40G bash scripts/experiment-apparatus/e185-r4-segment-run.sh cpu-writings-and-function-partition`。
  - 控漂移段（本机，上一条拷完树之后起）：`bash scripts/hosts/run-with-memory-cap.sh 16G env E185_RUN_DATE=<日期> bash scripts/experiment-apparatus/e185-r4-segment-run.sh host-cpu-drift-control`。
  - 重判段（本机，两段都交回之后）：`bash scripts/hosts/run-with-memory-cap.sh 2G nice -n 19 python3 -I scripts/experiment-apparatus/e185_r4_baseline_rejudge.py --host-cpu-product research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out --drift-control-product research/results/<控漂移段产物> --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out --writings-product research/results/<写法与拆分段产物> --attack-ledger research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out > research/results/e185-r4-baseline-rejudge-<日期>.out`。
- **上限与线程**：控漂移段 16G（同 r3 第一段）；写法与拆分段外层 46G、`E185_MEMORY_CAP=40G`（同 r3 第三段）；重判段 2G。`E185_THREADS` 不设，取登记值 `.claude/heavy-thread-default`（今天 31），只用于写法与拆分段的准备队列；编译经 `bash scripts/hosts/capped.sh 31`。显存额度每张卡 4096 MiB（同 r3）。
- **单测与变异**：装置改完由主 agent 带提交前缀在主工作区跑（checker 档）：不带特性 `cargo test --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split`，带 `gpu` 特性的那几条经 `gpu-quiesce-run.sh` 另跑一趟；取样那几条单测要帧指针，另带 `RUSTFLAGS='-C force-frame-pointers=yes'` 跑一趟只点名它们的；`crates/mutations.tsv` 里新加的 `E185 V` 行（第九节）。重判脚本的自证 `python3 scripts/experiment-apparatus/e185_r4_baseline_rejudge_test.py` 与它的变异表（形照 `e185_r1_compare.mutations.tsv`）。全绿、变异全抓之后才起；条数写进第十二节。

### 输入清单（每一段一份，起跑前写，跑完不改）

`research/results/e185-r4-inputs-<段>-<日期>.txt`：r3 输入清单那几行原样（装置 sha256 第一行；`gpu_unit_checks.rs`、`mounted_session.rs`、`region_scan.rs`、`transaction.rs`、`checksum.rs`、`Cargo.lock` 的 sha256；`wgpu` 版本；`nvidia-smi` 那一行（控漂移段）；`cargo tree` 原样；`nproc`；`getconf CLK_TCK`；上限与线程），另加：`crates/singlefs-core/src/unit.rs`、`compaction_plan.rs`、`records.rs` 的 sha256；`RUSTFLAGS` 原样；`addr2line --version`、`nm --version` 头一行；`/proc/cpuinfo` 的 `model name` 与 `flags` 里 `sse4_2`、`pclmulqdq` 在不在；`scaling_governor`；`perf_event_paranoid`；四份输入产物的 sha256（重判段另加两份 r4 产物的）。第二台的主机名、路径照 r3 的写法换成「〔第二台的目录〕」。

### 产物（带日期，不盖已有的；同一天同一段再跑加 `-2`、`-3`）

| 段 | 产物 | 补问题单哪一行 |
|---|---|---|
| 控漂移段 | `research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-<日期>.out`（标准错误同名加 `.err`） | 第 2 行 |
| 写法与拆分段 | `research/results/e185-gc-batch-wall-clock-split-cpu-writings-and-function-partition-r4-<日期>.out`（同上） | 第 1 行的阶段 CPU、第 3 行 |
| 重判段 | `research/results/e185-r4-baseline-rejudge-<日期>.out` | 第 1 行 |
| 每段 | `research/results/e185-r4-inputs-<段>-<日期>.txt`；控漂移段、写法与拆分段另有 `research/results/e185-r4-quiesce-<段>-<日期>.txt` 与 `research/results/e185-r4-serial-<段>-<日期>.txt` | — |

### 实现量与分段（3b；每段交回之后主 agent 对着问题单判续不续，一行开着的都不剩就停）

估的实现量（推的，按 r3 那一次的改动量类比）：控漂移段约 600–900 行加约 20 条单测（复用 r3 第一段的格、判定与翻面代码，新写绑核、频率读数、两种暖机、卡交替、探针格、轮次比与 D1–D5）；写法与拆分段约 1000–1500 行加约 30 条单测（两种硬件 CRC 与移位矩阵、抄的扫描与两张索引、计时件、系数、取样计时器与处理函数、帧指针链与补链、符号化、分项、P-sample、FP 探针、开销探针、F1–F5）；重判段约 500 行 Python 加约 300 行自证与变异表；起跑脚本约 180 行。一个执行员做不完，分三段，可以由不同的执行员并行写（行格式第六节写死了）：

- **实现第一段**：控漂移段与 C905 的串行声明、起跑脚本（两段的分支都写好）。补第 2 行；跑完第 2 行够判。
- **实现第二段**：写法与拆分段。补第 3 行与第 1 行的阶段 CPU；跑完第 3 行够判，第 1 行还差实现第三段。
- **实现第三段**：重判段脚本。补第 1 行；与实现第二段的产物一起，第 1 行够判。

## 六、报哪些量与各自的判据

每个量一行、各报各的判定，不合成一个真假。控漂移段每一格 5 轮各判一次，5 轮相同写那个判定，不全相同写 `unstable`（真设备、真时序，`test-discipline.md`「单次观测不算数」）；某轮读不到写那一轮 `not_run`，不当 0。写法与拆分段的阶段重放照 r2（每项 5 次重放取中位、5 轮各一条意图）。重判段是对产物的算术，确定性：跑几遍都一样，证据强度靠 PR1、PR2 与变异。

**门槛旁边写它从哪来**：B4 的门槛 T_X 是 r3 的算法（D26 已定项 13 第 7 条那把尺：CPU 模式省下的主机 CPU ÷ 提交次数）换写法，由 CPU 模式那一侧的阶段 CPU 定，与第 1 行四条臂「怎么做」的写法是两回事：臂定义只说换哪一项的写法，不说那一项在这台机器上要多少 CPU，c_rt 是另一边量的，所以 c_rt < T_X 不是臂定义的同义反复。D1–D3 的 [0.90, 1/0.90] 与 0.80、D4 的 1.5 与 2、F1 的 5%、F2 的 3%、F3 的 95%、F6 的 2% 与 5% 是设计员定的政策线（哪几条看过第四节的数写在第四节末尾），没有一条能从臂的定义推出：臂定义说的是「控住之后消失／仍在」「是不是划分」，不说消失到多近、差多少算不是。

| 量 | 怎么算（产物行与键） | 门槛与判定 | 问题单哪一行、翻面取值 | 可跑性（3a ①②） |
|---|---|---|---|---|
| B1 一批的阶段 CPU 按写法 | 写法与拆分段每次调用一行 `name=cpu_writing_stage fraction= pool_data_units= batch_units= round= call_index= kind= crc_today_cpu_nanoseconds= crc_hardware_single_cpu_nanoseconds= crc_hardware_three_way_cpu_nanoseconds= release_check_crc_today_cpu_nanoseconds= release_check_crc_hardware_single_cpu_nanoseconds= release_check_crc_hardware_three_way_cpu_nanoseconds= scan_today_cpu_nanoseconds= scan_identity_index_lookup_cpu_nanoseconds= identity_index_build_cpu_nanoseconds= identity_index_entries= identity_index_location_keys= choose_cpu_nanoseconds= crc_bytes= release_check_crc_bytes= hardware_crc_matches= identity_index_scan_matches=` | 不单判，喂 B2、B3 | 第 1 行：够判条件原文「四种基线在 12 个点上各量到一批的主机 CPU（按阶段拆开）」就是这一行 | 每次调用八个新计时件 × 5 次，每件微秒到毫秒（第四节第 8 行推的），每次调用 < 0.1 秒；60 条意图 × 2–7 次调用约 300 次，约 30 秒。分辨率：`time_a_pure_stage` 重复到够长才读钟，线程钟 170 ns，件与件之差在微秒以上 |
| B2 写法系数 φ | 每点每种写法每样阶段一行 `name=writing_factor fraction= pool_data_units= batch_units= writing= stage= factor_per_round= factor=`（第一节「门槛来源」） | 不单判，喂 B3 的「r2 产物按比例换」那一端；写法甲恒为 1 | 第 1 行（门槛来源那个旋钮的一端） | 算术，1 秒内 |
| B3 候选门槛 | 每（写法、门槛来源、门槛的机器、c_rt 来源、点、候选）一行 `name=baseline_threshold … saved_per_intent_milliseconds= submissions_per_intent= threshold_milliseconds= threshold_over_today=`；乙段增量为负的写 `status=not_run reason=scan_increment_below_zero` | 不单判，喂 B4 | 第 1 行：问题单原文「候选 A–D 的门槛照第 3 次重跑登记的算法逐点现算」 | 算术 |
| B4 重判（第 1 行的判定） | r3 第一段每一行点行（与控漂移段受控臂每一行）× 九种写法 × 门槛来源两端：`name=baseline_rejudge_point writing= threshold_source= crt_source= <那一行点行原有的点、候选、卡、形态、等法、间隔、钟、门槛的机器各键> threshold_milliseconds= judgement_per_round= judgement= r3_judgement= changed= floor_judgement_per_round= floor_judgement= r3_floor_judgement= floor_changed=` | 照 r3 R1–R3：每轮 c_rt < T_X 判 `not_losing`（B 写 `not_losing_lower_bound`），≥ 判 `losing`；5 轮相同写那个，不全相同 `unstable`；c_floor 同法。`changed` = 判定与 r3 那一行 `judgement` 不同；c_rt 来源取控漂移段时，r3 里没有同键那一行的（两趟进程钟那几格不一样多），写 `r3_judgement=absent changed=not_applicable` | 第 1 行：某行 `changed=true` 就是问题单原文「判定变没变」的「变」。某候选在某点某卡某形态下从 `not_losing` 变 `losing`，那一格在写法 X 下这个候选出局；某点上候选 C 的 9 种（形态、等法、间隔）全部 `losing`，那一点 C 在写法 X 下不再「下界不赔」（r3 R1 的读法）。选择从「拿甲当基线」翻到「拿乙／丙／丁当基线」看的是用户与主 agent 认哪一种写法是 CPU 模式，这一行给的是每种写法下各候选还剩几格不赔 | 2684 行 × 9 × 2（再乘 c_rt 来源两端），约 10 万次比较，秒级 |
| B5 重判汇总 | `name=baseline_rejudge_summary writing= threshold_source= crt_source= candidate= index= threshold_kind= clock= lines= not_losing= not_losing_lower_bound= losing= unstable= rounds_insufficient= not_run= changed= changed_to_losing= changed_to_unstable= changed_to_not_losing=` | 只报（由 B4 数出来） | 第 1 行（「第一段的不赔还剩多少」，第二轮判决第三节第 1 条） | 算术 |
| B6 每点还剩不赔的形态数 | `name=baseline_rejudge_point_summary writing= threshold_source= crt_source= candidate= index= threshold_kind= fraction= pool_data_units= batch_units= forms_not_losing= forms_losing= forms_unstable= forms_judged=` | 只报；`forms_not_losing` 从 > 0 变 0 是那一点那个候选翻面 | 第 1 行 | 算术 |
| B7 今天的写法这一趟 ÷ r2 | 写法与拆分段每点每样阶段：这一趟写法甲的中位 ÷ r2 第二段产物同一点同一样的中位，`name=today_over_r2 fraction= pool_data_units= batch_units= stage= ratio=` | 只报；不在 [0.80, 1.25] 走第十条 L9 | 不单独对应；它说明「这一趟产物直接算」那一端离 r2 有多远（这一次编译带帧指针、日子不同），两端判定不同的点照「随门槛来源翻面」 | 算术 |
| D1 受控臂卡 0 的轮次比 | 控漂移段 `name=round_ratio arm=controlled index=0 scope=point_lines lines= median_round_five_over_round_one= median_round_two_over_round_one= median_round_three_over_round_one= median_round_four_over_round_one=`；判定 `name=drift_judgement quantity=controlled_card_zero_round_ratio value= judgement=` | 在 [0.90, 1 ÷ 0.90] 判 `drift_gone`；< 0.80 或 > 1.25 判 `drift_remains`；之间判 `between` | 第 2 行：`drift_gone` 是「控住之后消失」（甲那一侧），`drift_remains` 是「控住之后仍在」（乙那一侧）；值从 < 0.80 越过 0.90 就从乙翻到甲 | 108 行；r3 卡 0 是 0.528、卡 1 是 0.958（第四节第 1 行），两条线之间差 0.1，比 r3 卡 1 离 1 的 0.042 大一倍以上 |
| D2 复现臂卡 0 的轮次比 | 同上 `arm=reproduction index=0`；`quantity=reproduction_card_zero_round_ratio` | ≤ 0.80 判 `drift_reproduced`；在 [0.90, 1 ÷ 0.90] 判 `drift_not_reproduced`；之间 `between` | 第 2 行：D1 的「消失」只在 D2 `drift_reproduced` 时才说明是控住造成的；D2 `drift_not_reproduced` 时这一天复现不出，走第十条 L5 | 同上 |
| D3 受控臂卡 1 的轮次比 | 同上 `arm=controlled index=1`；`quantity=controlled_card_one_round_ratio` | 同 D1 的三档 | 第 2 行：问题单原文「与卡 1 同样的跑法对照」；卡 1 `drift_remains` 时控住的跑法本身在两张卡上都造出漂移，第 2 行不判（第十条 L4） | 同上 |
| D4 受控臂 `unstable` 数卡 0 ÷ 卡 1 | `name=unstable_count arm= index= lines= unstable=`（那条臂那张卡全部 `name=drift_control_point` 行的 `judgement` 列，口径同第二轮判决数 1054 的那一列）；`name=drift_judgement quantity=controlled_unstable_card_zero_over_card_one value= judgement=` | ≤ 1.5 判 `card_zero_not_less_stable`；≥ 2 判 `card_zero_less_stable`；之间 `between`；两张卡都是 0 判 `both_zero` | 第 2 行：问题单原文「`unstable` 格数变化」；从 ≥ 2 落到 ≤ 1.5 支持甲 | 每张卡约 1342 行（2684 ÷ 2），数得清 |
| D5 `unstable` 数与 r3 并列 | 同上，另报复现臂卡 0、受控臂两张卡的 `unstable` 与 r3 第一段同一张卡的数（重判段读 r3 产物时数） | 只报 | 第 2 行（变化本身） | 算术 |
| D6 频率读数 | `name=clock_reading arm= index= <格的键> round= position=before\|after host_core= scaling_cur_freq_khz= cpuinfo_avg_freq_khz= sm_clock_megahertz= memory_clock_megahertz= pstate= temperature_celsius= power_watts= clocks_event_reasons=`；每条臂每张卡每轮一行中位 `name=clock_round_summary`；每张卡格一级的 c_rt 与 `sm_clock_megahertz` 的秩相关 `name=clock_correlation` | 只报 | 第 2 行：甲说的「频率造的」有没有在读数里露出来（卡 0 的 SM 时钟随轮次涨而 c_rt 随轮次降）；不单判 | 每格前后各一次，并进本来就起的 `nvidia-smi`；`sysfs` 读两个文件 |
| D7 暖到稳 | `name=warm_up_to_stable arm=controlled index= windows= seconds= window_medians_nanoseconds= reached_the_cap=`；每格 `name=cell_warm_up arm=controlled index= <格的键> round= warm_up_submissions=` | 只报；`reached_the_cap=true` 走第十条 L4 那一支的旁证 | 第 2 行（控住的做法真的做到了没有） | 每张卡 60–600 秒 |
| F1 拆法乙：各项之和 ÷ 整次调用 | 每个 K = 8 的点签发、各批各一行 `name=sampled_partition_summary fraction= pool_data_units= batch_units=8 kind= calls= samples= items_over_the_call_pooled= items_over_the_call_per_round= samples_per_round= rounds_outside_ten_percent_with_enough_samples= judgement=` | 五轮合起来的「各项权重之和 × 1 ms ÷ 各次调用线程钟之和」在 [0.95, 1.05] 判 `partition_holds`，在外判 `partition_fails`；合起来在里面、而取样 ≥ 100 的轮里有 ≥ 2 轮单轮的比落在 [0.90, 1.10] 之外，判 `unstable`。不按单轮判：P = 1024、f = 0.25 的签发一轮只一次调用、约 13 个取样，单轮的计数误差就有约 ±8%（第四节第 7 行） | 第 3 行：问题单原文「新拆法下各项之和与整次调用之比」与够判条件「相差不超过登记写死的容差」；从 `partition_fails` 变 `partition_holds` 让第 3 行从甲翻到乙 | 四点合计约 130–2200 个取样一点；计数相对标准差 0.06%–0.7%（第四节第 7 行），比 5% 细一个数量级以上 |
| F2 拆法乙：计数核对 | 同一行 `all_weights_over_the_call_pooled=`、`counting_judgement=` | 全部权重（含链断与认不出）× 1 ms ÷ 线程钟在 [0.97, 1.03] 判 `counting_holds`，在外 `counting_fails` | 第 3 行：F1 不在容差里时，F2 说明是不是计时器数错了（不单独翻面） | 同上 |
| F3 拆法乙：成链 | 同一行 `direct_chain_share=`、`recovered_chain_share=`、`chain_broken_share=`、`symbol_unknown_share=`、`coverage_judgement=`、`coverage_judgement_without_recovery=` | （直接 + 补回）÷ 全部 ≥ 0.95 判 `coverage_holds`，< 0.95 判 `coverage_fails`；只算直接成链的另判一次 | 第 3 行：F1 不在容差里时，F3 说明是不是链断了（不单独翻面）；补回那一端与不补回那一端判定不同记「随补链翻面」 | 同上 |
| F4 各项份额（附带，同一批取样，不另跑） | 同一行 `share_<项>=`、`share_<项>_interval=`（二项 95% 区间），项照第一节那张名单；另一行 `name=sampled_partition_against_the_replay` 把 r3 七项与拆法乙对应的那几项并列 | 只报 | 不对应问题单的行（区域扫描占几成是设计轮岔路单第 10 行要的，`research/prompts/m4-gpugc-design-r1-forks.md` 第 16 行）；F1 判成之后才读 | 同上 |
| F5 拆法甲：各项之和 ÷ 整次调用 | 同一批调用上 r3 第三段的重放，`name=replay_partition_summary fraction= pool_data_units= batch_units=8 kind= items_over_the_call_pooled= items_over_the_call_per_round= judgement=` | 与 F1 同一条线 [0.95, 1.05]；单轮照 r3 每一次调用都有读数，`unstable` 照 F1 的规则（轮的取样数换成调用数 ≥ 1） | 第 3 行：甲在这一趟也在容差里，两种拆法都能用；甲照旧在外（r3 那七行的样子），甲不是划分 | r3 第三段同样的重放，约每次调用 0.1–0.5 秒 |
| F6 取样的开销 | 段开头与段末 `name=sampler_overhead position= ratio=` | ≤ 1.02 判 `overhead_negligible`；(1.02, 1.05] 判 `overhead_noted`；> 1.05 走第十一节作废 | 不单独对应；它说明 F1 量的那次调用有没有被取样本身拖慢 | 20 遍 16 MiB CRC，约 1 秒 |

另照 r3 原样打、原样判、不对应这三行的：控漂移段两条臂的 R1–R5、J1–J11 行与 `name=knob_flip` 行（第 1 行 c_rt 来源那一端要用它们的 c_rt），写法与拆分段 r2 原格式的 `name=stage` 行与停机条款第 4 条。产物行的键名照 `scripts/gate/data-name-lint.py` 写全名；执行员为过 lint 改了键名的，在第十二节记新旧对照，不改量的算法。

## 七、钉绝对值的断言

### 甲类：出自被测条款本身的（不符的时候先查装置；这几条不符都不是条款错，走第十一节）

| 断言 | 出处（逐字） | 不符时 |
|---|---|---|
| 替代写法做的是同一份活：每次重放，写法乙丙丁的 `crc_bytes`、`release_check_crc_bytes` 与写法甲相同，交回的值（CRC 逐个、扫描整个 `RegionScan`）与 crates 那一份逐字相同 | D26 已定项 13 第 7 条「CPU 模式下它的 CPU 时间」（量的是同一个后台任务在 CPU 模式下的 CPU，换写法不许换活）；D24 已定项 2 第 3 条「两份在测试里对拍」（这一趟在 CPU 一侧照这一句对拍） | 装置抄错或写错：停机 K-S1、K-S2 |
| 控漂移段的 c_rt 照旧罩住准备、提交、等、两份比对：P1 甲的自旋插在 ① 之前，增量落在 2 ms 的 [0.9, 1.1] 倍；⑤ 那一段的线程钟 ≤ 整次提交的线程钟 | 第 7 条「后者含准备数据、提交、等结果、两份结果的比对」（r3 第七节甲类第 1 条原样） | 装置没罩全：第十一节「阳性对照红」 |
| 第 3 行的「整次调用」是那次调用在调用线程上的线程 CPU（不是挂钟、不是进程钟）；各项之和要与它比 | 第 7 条「收益是后台任务少占的主机 CPU 时间」（按函数拆的是这一份 CPU） | 拆法乙拿错了分母：装置错，第十一节作废 |

### 乙类：独立算出、用命令核过的（不符才作废，或照第十一节停机）

算式只用第一节的定义；第十三节的 python 命令算过，输出原样抄在那里。

| 断言 | 值 | 在哪核 |
|---|---|---|
| 两种硬件写法对「123456789」 | `0xe3069283` | 写法与拆分段开头（PW3） |
| 两种硬件写法对「第 i 字节 = i mod 251」 | 32768 字节 `0x2ad60827`，16384 字节 `0xeafca51d`；三路合并与整段一遍相同 | 同上；单测钉住 |
| 三路切段 | 32768 → (10920, 10920, 10928)；16384 → (5456, 5456, 5472) | 单测 |
| 校准活（r2 第七节原样） | 16 MiB `0x3430be3b`，32 MiB `0x84b65aa5` | 每次校准 |
| 一次重放里写法乙丙丁的字节 | `crc_bytes` = 这一批扫描读字节，`release_check_crc_bytes` = 这一批 `release_check_read_bytes`（r2 停机条款第 4 条那两条锚点原样，换写法照核） | 每次重放 |
| `cpu_writing_stage` 行数 | 等于同一段 `name=stage` 行数（每一次重放的调用各一行） | 写法与拆分段收尾 |
| r3 第一段线程钟那一半的点行数 | 1776（2 张卡 × 12 个点 × (4 个候选 × 9 种形态等法间隔 + 今天的写法 1 行) × 2 种门槛的机器）；每张卡 `candidate=scan_checksum_only threshold_kind=machine_scaled clock=thread` 108 行 | 重判段读入时；控漂移段每条臂每张卡的同口径行数也是 108 |
| 写法甲在门槛来源「r2 产物按比例换」下的门槛 | 与 r3 第一段产物每一行的 `threshold_milliseconds` 逐字相同；φ ≡ 1 | 重判段（PR1、K-S3） |
| 控漂移段每条臂每张卡每轮的格数 | 80 格 + 2 格探针 | 段收尾 |
| 漂移探针递减格 | 8.0、6.1、4.2、2.3、0.4 ms；c_rt 0.05、1、3、6、14、30 ms 时第 5 轮 ÷ 第 1 轮 0.0559、0.1556、0.3091、0.4571、0.6545、0.8000 | 单测（假钟）钉式子；真跑时 P-drift |
| P-sample 的份额与总量 | 3/11、1/11、6/11、1/11（0.2727、0.0909、0.5455、0.0909）；300 遍 × 11 ms = 3300 ms，期望权重之和 3300 | 段开头与段末 |
| 随机相位下一次调用的期望取样权重 | = 这次调用的线程 CPU ÷ 1 ms（单测用假计时器：13.5 ms 的调用，1000 次随机相位的平均权重在 13.5 ± 0.05） | 单测 |

互比旁边的绝对值：写法之间的互比（B2、B4 里乙丙丁对甲）旁边是 PW3 的逐字相同（活确实是同一份）、PW1 的 2 ms（钟没坏）与 PR1 的「写法甲逐行复现 r3」（门槛算法没走样）；九种写法一起错（例如计时件漏了一半的单元）时，PW2 的两倍与 `crc_bytes` 锚点先红。两条臂的互比（D1 对 D2、D1 对 D3）旁边是 P1 的 2 ms 与 P-drift 的 ≤ 0.80（钟与比值函数都分得出漂移）；两条臂一起错（例如轮次比读成了第 4 轮）时，P-drift 的锚点 0.0559–0.8000 先红。两种拆法的互比（F1 对 F5）旁边是 P-sample 的四个份额与 3300（取样与分项分得清）；两种拆法一起错（例如整次调用读成了挂钟）时，F2 的计数核对与甲类第 3 条先露出来。

## 八、轨迹与几何敏感性

### 轨迹（被谓词消费的量）

被判定当谓词输入的量：控漂移段的每格 c_rt（D1–D4）、写法与拆分段的单轮比（F1、F3、F5）、作废用的校准比。

| 量 | 峰值 | 为正的条数 | 期末值 |
|---|---|---|---|
| 每条臂每张卡逐轮的 c_rt 中位（÷ 第 1 轮） | 五轮里最大的那个比（`round_ratio_peak`） | 比落在 [0.90, 1/0.90] 之外的轮数（`rounds_off_by_more_than_ten_percent`） | 第 5 轮的比（就是 D1–D3 的值） |
| 受控臂暖到稳的窗中位 | `window_median_peak_nanoseconds` | 与前一窗相差 > 3% 的窗数 | 最后一窗的中位 |
| 每格的热身次数（受控臂） | 全段最多的那一格 | 热身到顶（100 次）的格数 | 段末那一格的 |
| 拆法乙的单轮比（每点每种调用） | 五轮里离 1 最远的那个 | 落在 [0.90, 1.10] 之外的轮数（取样 ≥ 100 的轮另数） | 第 5 轮的 |
| 拆法乙的单轮成链份额 | 五轮里最低的 | < 0.95 的轮数 | 第 5 轮的 |
| 两段的校准比（作废谓词，r3 原样） | 全段最大 | > 1.10 的格轮数或意图数（另报 > 1.05、> 1.20 的） | 段末那一格或那条意图的 |

只报期末值的那一格正文里不写「从来」「恒」。

### 几何敏感性（每个带跑前判决的量）

| 旋钮 | 两端 | 哪几个判定要两端都报 |
|---|---|---|
| 写法乙的硬件 CRC | 单路 ／ 三路 | B4（写法乙丁） |
| 建索引 | 计入 ／ 不计 | B4（写法丙丁） |
| 门槛来源 | r2 产物按比例换 ／ 这一趟产物直接算 | B4 |
| 门槛的机器 | `attack_ledger` ／ `machine_scaled` | B4；控漂移段的 R1–R5 照 r3 |
| c_rt 来源 | r3 第一段 ／ 控漂移段受控臂 | B4 |
| 卡 | 序号 0 ／ 序号 1 | B4、D1 对 D3、D4 |
| 池的数据量 P | 1024 ／ 8192（写法丙下乙段增量里线性找的份量随 P 变，两端方向相反：P 大线性找贵、建索引也贵） | B4、F1、F3 |
| 活槽占比 f | 0.25 ／ 0.75 | B4、F1 |
| 批大小 K | 4 ／ 8 ／ 16 | B4 |
| 补链 | 补 ／ 不补 | F1、F3 |
| 控漂移段的臂 | 复现 ／ 受控 | D1 对 D2 |
| 受控臂里一张卡在一轮里先量还是后量 | 先 ／ 后（第奇数轮卡 0 先，第偶数轮卡 1 先） | D1、D3：每张卡另按「先量的轮」「后量的轮」各算一次第 5 轮 ÷ 第 1 轮之外的同口径比（`name=card_order_effect index= first_rounds_median= second_rounds_median= ratio=`），只报 |

某个判定在一个旋钮两端翻面，写「随 <旋钮> 翻面」并把两端的值都列出来，不写成那条臂的性质；不翻写「这几个点上不翻」并列点（照 r3 J4 与 R3-7、R3-11 的口径：一端不是判定的对不算）。

**判别力自证**：判 B4、D1–D4、F1–F3、F5 的函数各有单测：把门槛挪到同一格两个量到的值之间，判定必须由一种变成另一种；合成输入喂 D1–D3 的三档、D4 的四档、F1 的三档各出一次；重判脚本的自证同样各喂一份。

## 九、变异

V1–V154 照原登记、r2 登记、r3 登记与它们的修订，不增不删；这一趟改到它们锚点的，锚点改到新源码里逐字命中一次（`.claude/rules/mutation-sampling.md` 第七类），点名的单测不换。装置的新变异追加进 `crates/mutations.tsv` 的 E185 行之后，名字接着编 V155 起；重判脚本的变异进 `scripts/experiment-apparatus/e185_r4_baseline_rejudge.mutations.tsv`（名字 R4J1 起）。每条点名一条会红的单测（要卡的带 `gpu` 特性、经清场跑；要帧指针的带 `RUSTFLAGS`）；常量断言写加法不写减法。执行员可以补，不许删。

| 变异 | 改哪 | 在哪个取样点改变输出 | 该红的 |
|---|---|---|---|
| V155 单路初值不取反 | 单路硬件 CRC 的初值 `0xFFFFFFFF` 写成 0 | 「123456789」（应 `0xe3069283`） | PW3 校验值单测 |
| V156 三路合并移位用错段长 | 合并第三段时用第二段的段长 | 32768 字节（第二、三段 10920 与 10928 不等；应 `0x2ad60827`） | 三路锚点单测；16384 字节另一条 |
| V157 三路切段不取 8 的倍数 | 段长取 ⌊长度 ÷ 3⌋ | 32768（应 10920、10920、10928） | 切段单测 |
| V158 单路尾巴不算 | 尾巴那几个字节跳过 | 0–64 字节随机长度（长度不是 8 的倍数的那几条） | PW3 随机长度单测 |
| V159 释放前读盘核用了扫描的单元 | `release_check_crc` 的硬件写法读 `units_read_by_the_scan` | 合成一批：扫描单元与释放前副本字节不同 | `release_check_crc_bytes` 锚点单测 |
| V160 槽索引不看盘 | （盘、槽）索引的键只取槽 | 手搭池：两条映射条目同槽号、不同盘 | 抄的扫描九种结局单测（K-S2） |
| V161 key 索引只收指着这一槽的 | 「头里那把 key 有没有映射条目」改成只在指着这一槽的条目里找 | 手搭池：头里的 key 有条目、条目不指这一槽（应 `Inconsistent(HeaderKeyDisagreesWithTheMappingLocation)`） | 同上 |
| V162 索引查出来不排序 | 条目号不按映射条目的次序排 | 手搭池：一槽两条条目、次序与插入次序相反 | 同上 |
| V163 计入那一端漏了建索引 | `identity_index_with_build` 不加建索引 | 合成读数：建索引 1 ms、查 0.1 ms | 写法读数单测（计入 1.1、不计 0.1） |
| V164 乙的乙段增量减了硬件 CRC | 写法乙的 `scan_increment` = 扫描 − 硬件 `crc` | 合成：扫描 10、今天 crc 4、硬件 crc 1（应 6） | 写法读数单测 |
| V165 系数取平均 | φ 的各轮取平均不取中位 | 合成五轮比 1、1、1、1、5 | 系数单测（应 1） |
| V166 系数乘到了选段 | 「r2 产物按比例换」把 `choose` 也乘 φ | 合成：B − A 应等于签发的 `choose` | 门槛单测 |
| V167 写法甲的系数不是 1 | φ 用今天写法这一趟 ÷ r2 | 合成：这一趟与 r2 不同 | PR1 单测（脚本） |
| V168 D 的门槛含签发 | D 的省下加签发那一行的 `release_check_crc` | 合成：签发行的 `release_check_crc` 非 0 | 门槛单测 |
| R4J1 重判的等号错一侧 | `c_rt < T` 写成 `<=` | 合成 c_rt 正好等于 T（应 `losing`） | 重判判定单测 |
| R4J2 重判读成了 c_floor | 判定读 `floor_host_cpu_per_submission_milliseconds_per_round` | 合成一行两列不同 | PR1 单测 |
| R4J3 ρ 倒过来 | ρ = 分母 ÷ 分子 | 合成：本机 1 ms、第二台 2 ms（应 0.5） | ρ 单测与 K-S4 |
| R4J4 直接算那一端乘了两次 ρ | `machine_scaled` 再乘一次 | 合成 | 门槛单测 |
| R4J5 B 的不赔不标下界 | `not_losing_lower_bound` 写成 `not_losing` | 合成 c_rt < T_B | 判定单测 |
| R4J6 负的乙段增量照算 | 负数不写 `not_run` | 合成增量 −0.1 ms | 门槛单测 |
| R4J7 c_rt 来源读成复现臂 | 控漂移段那一端读 `arm=reproduction` | 合成产物两条臂都有 | c_rt 来源单测 |
| R4J8 变没变比的是 `floor_judgement` | `changed` 拿 r3 的 `floor_judgement` 比 | 合成一行两列不同 | 重判单测 |
| V169 暖到稳拿第一窗比 | 「连着三窗离均值 3%」改成「离第一窗 3%」 | 假钟：窗中位 10、9、8、8、8 | 暖到稳单测（应在第 5 窗稳） |
| V170 每格热身只做 10 次 | 去掉「相邻两组差 ≤ 5%」那一圈 | 假钟：头 30 次每次少 2%、之后不变 | 每格热身单测（应在第 50 次停，变异停在第 10 次） |
| V171 受控臂不绑核 | 去掉 `sched_setaffinity` | 单测里绑完读本线程的亲和掩码 | 绑核单测（掩码只有一个核） |
| V172 卡不交替 | 每轮都先卡 0 | 次序单测（第 2 轮应先卡 1） | 次序单测 |
| V173 复现臂也绑核 | 复现臂调绑核 | 次序单测（复现臂的掩码应是全集） | 绑核单测 |
| V174 轮次比取第 4 轮 | 第 5 轮写成第 4 轮 | 合成五轮 10、9、8、7、5（应 0.5） | 轮次比单测 |
| V175 轮次比把进程钟的行算进来 | 不滤 `clock=thread` | 合成：线程钟行比 1、进程钟行比 0.5 | 轮次比单测 |
| V176 三档的边写成开区间 | 0.90 写成 `>` | 合成比正好 0.90（应 `drift_gone`） | D1 判定单测 |
| V177 漂移探针不递减 | 递减格每轮都自旋 8 ms | 假钟：c_rt 3 ms（应 0.3091） | P-drift 单测 |
| V178 `unstable` 数把 `not_run` 算进来 | 数 `unstable` 时把 `not_run`、`rounds_insufficient` 也算 | 合成一张卡两种都有 | D4 单测 |
| V179 频率读数进了计时区间 | `nvidia-smi` 那一次挪进计时区间 | 假提交步骤：读数一步走 50 ms，整次应不含 | 区间单测 |
| V180 卡 1 是 0 时 D4 除零 | 去掉 `both_zero` 那一支 | 合成两张卡都 0 | D4 单测 |
| V181 段开头不声明串行（C905） | 去掉控漂移段开头那一句 `declare(Serial(…))` | 单测里的假格 | 串行声明单测（声明文件里应有一行串行） |
| V182 段末不声明并行 | 去掉段末 `declare(Parallel)` | 同上 | 串行声明单测（最后一行应是并行） |
| V183 首次到期不随机 | 首次到期写死 1 ms | 假计时器：13.5 ms 的调用 1000 次（应平均 13.5 ± 0.05，变异 13） | 计数单测 |
| V184 不认 overrun | 权重恒 1 | 假计时器：一次到期带 overrun 3 | 计数单测 |
| V185 帧指针链不核栈界 | 去掉上下界检查 | 手造的栈：第 3 层指出栈外 | 链单测（应停在第 3 层、记链断） |
| V186 不核走到锚点 | 走完就算成链 | 手造的栈：链走不到锚点 | 成链单测 |
| V187 补链不核走到锚点 | 找到第一个代码段地址就收 | 手造的栈：那个位置往下走不到锚点 | 补链单测 |
| V188 分项取最里面的那一个 | 「最外的那一个作数」改成最里 | 合成逻辑帧：`publish` 里面调 `clone` | 分项单测（应 `publish`） |
| V189 克隆前后按行号分反 | 3002–3005 记成 `clone_after` | 合成逻辑帧行号 3003 | 分项单测 |
| V190 不减装载基址 | 地址原样交 `addr2line` | FP 探针（应符号化出三个探针函数名） | FP 探针单测（要帧指针） |
| V191 不展开内联 | `addr2line` 去掉 `-i` | 合成的 `addr2line` 输出带两层内联 | 分项单测 |
| V192 各项之和漏了 `rest_of_the_publish_step` | 求和不含那一项 | 合成一次调用：那一项 20% | F1 单测 |
| V193 F1 拿单轮比的平均 | 合起来的比改成五轮比的平均 | 合成五轮取样数 13、13、13、13、1300 | F1 单测 |
| V194 计数核对用挂钟 | F2 的分母换成挂钟 | 假钟：挂钟是线程钟的 1.2 倍 | F2 单测 |
| V195 P-sample 的容差放宽 | ±0.03 写成 ±0.05 | 合成份额偏 0.04 | P-sample 单测（应红） |
| V196 计时器撤得晚 | 撤计时器挪到第 3462 行读钟之后 | 假计时器：窗口外 1 ms 里有一次到期 | 窗口单测 |
| V197 开销比倒过来 | F6 的比 = 关 ÷ 开 | 合成开 1.1、关 1.0 | F6 单测 |
| V198 「unstable」不看取样数 | F1 的「取样 ≥ 100 的轮」不滤 | 合成两轮各 13 个取样、比 0.85 | F1 单测（应 `partition_holds`） |

单测是新的计数实验，照 `.claude/singlefs-ai-sop/rules/show-me-test.md` 先证明会红（每条变异打开时红在点名的那一条）。

## 十、失败条款

r3 登记的 J1–J11 照管控漂移段两条臂各自的 R1–R5 行（原样，不改）；原登记 F1–F8、r1 登记 G1–G6、r2 登记 H1–H8 管的是这一趟不重判的量，不适用。新加（每条紧跟触发的观测）：

| 条款 | 判成什么 | 什么观测会让它触发 |
|---|---|---|
| L1 换了写法之后候选成批变赔 | 正当结果，照实记：那几格在写法 X 下那个候选出局；问题单第 1 行交主 agent 定拿哪种写法当 CPU 模式 | B5 里某写法 X 下 `changed_to_losing` > 0（r3 判 `not_losing` 或 `not_losing_lower_bound` 的行，写法 X 下判 `losing`） |
| L2 写法的两端定输赢 | 那一点记「随 <旋钮> 翻面」，两端的值都列出，第 1 行在那一点不当已判，交主 agent | 同一行在单路与三路、或计入与不计建索引两端的 `judgement` 不同 |
| L3 门槛来源定输赢 | 同上，记「随门槛来源翻面」 | 同一行在「r2 产物按比例换」与「这一趟产物直接算」两端的 `judgement` 不同 |
| L4 控住的跑法本身造出漂移 | 第 2 行不判，交主 agent：控住的做法在两张卡上都动了轮次（暖到稳到顶、或卡交替让卡 1 也一轮比一轮快） | D3 判 `drift_remains`；或受控臂两张卡都 `reached_the_cap=true` |
| L5 这一天复现不出 | 第 2 行记「复现不了」，不当甲也不当乙；r3 第一段卡 0 的读数照旧可疑（它的漂移这一天没再出现），c_rt 来源两端照报，交主 agent | D2 判 `drift_not_reproduced` |
| L6 拆法乙不是划分 | 正当结果，照实记：那一点那种调用拆法乙不在容差里，第 3 行在那一点不能拿拆法乙的份额；F2、F3 指出是计数还是链断 | F1 判 `partition_fails`（四个点签发与各批八行里任何一行） |
| L7 拆法甲这一趟也在容差里 | 正当结果：两种拆法在那一点都能用，并列两者的份额（F4 里 `name=sampled_partition_against_the_replay`），哪种拆法交主 agent | F5 判 `partition_holds` |
| L8 两种拆法份额差得远 | 不作废：照实记差多少，交主 agent；两种拆法都在容差里而区域扫描的份额差 > 0.10 时，说明重放与真调用里那一项的 CPU 不一样，第 3 行要的是真调用里的，取拆法乙 | F1、F5 都 `partition_holds`，`share_region_scan` 两边差 > 0.10 |
| L9 今天的写法这一趟离 r2 远 | 不作废：「这一趟产物直接算」那一端照报，但它与「r2 产物按比例换」判定不同的点照 L3 记；写明这一趟编译带帧指针（第五节） | B7 某点某样阶段的比不在 [0.80, 1.25] |
| L10 取样的开销看得出 | 不作废：F1 照判，那一点写明开销比 | F6 判 `overhead_noted` |

没有一条把「没测出差别」写成实现有 bug：门槛换了写法之后是多少、卡 0 控住之后漂不漂、拆法乙是不是划分，都是正当结果；装置与 crates 对不上走第十一节停机。

## 十一、作废条款与停机条款

### 作废

| 条款 | 作废什么 | 什么观测会让它触发 |
|---|---|---|
| 阳性对照红 | PW1 或 PW2：那一种写法在整段作废，它的 B1–B4 全写 `void_positive_control`；P1、P3：照 r3，那一条臂那张卡那一种提交形态整段作废；P-drift：那一条臂那张卡的 D 值写 `void_positive_control`；P-sample 或 FP 探针：拆法乙整段作废（F1–F4 写 `void_positive_control`，第 1 行那一半不受影响）；r3 第三段的 2 ms 对照：拆法甲整段作废 | 第五节阳性对照表任何一格判不出（例：PW1 三路那一件的增量 2.4 ms；P-drift 递减格比 0.93；P-sample 的 `item_gamma` 份额 0.50；FP 探针符号化出来不是那三个函数） |
| GPU 结果错、GPU 计时没等到 | 照 r3（P2、P4），控漂移段那条臂那张卡整段作废 | r3 第十一节原样 |
| 读不到 | 那一次读数作废、重量，不当 0：线程钟、进程钟、挂钟读失败；`identity_index_entries=0`；`sample_buffer_full`；`addr2line` 对一次调用的地址一个都认不出（那一次调用的 F 行作废）。频率读数（D6）读不到只写 `none`，不作废那一格 | 对应的读失败或计数为 0 |
| 机器被别的活挤 | 照 r3（控漂移段每格前后校准比 > 1.10，那一格那一轮作废、等常态再重量）与 r2（写法与拆分段每条意图前后校准比 > 1.10，那条意图作废重量）；控漂移段开格前看到别的计算进程照 r3 等它 | r3 与 r2 第十一节原样 |
| 取样的开销太大 | 拆法乙整段作废 | F6 段开头或段末的比 > 1.05 |
| 计数特性被并进来 | 那一段整段作废 | 输入清单 `cargo tree` 那一段出现 `lookup-work-counters`（r3 原样） |
| 清场没过、卡起不来、运行条件不在 | 那一段（或那张卡）记「本次未跑」，不当结果 | `gpu-quiesce-run.sh` 退 78；`gpu_cards_by_priority` 没交回点名的卡；写法与拆分段开跑核 `sse4.2`、`addr2line`、`nm` 有一样不在（退 78） |
| 包装出局 | 那一次输出不算结果，命令与退出码写进报告 | `run-with-memory-cap.sh` 退 245 或 250–254 |

### 停机（装置与 crates 实现对不上，或这一趟的前提不成立：两边都查，既不作废也不当结果，`.claude/rules/implementation-first.md` 第 4 条）

任何一条触发，装置打 `name=stop_clause clause=<编号>`、收尾、退 3（重判脚本同样打这一行、退 3），执行员把装置的数、实现的数、两边读的代码行写进报告交回主 agent：

- K-S1 单路或三路硬件 CRC 对某个单元的值与 crates `crc32_castagnoli` 不同（含开跑时的校验串、i mod 251 两条与随机长度）。
- K-S2 抄的那份扫描（两张索引）交回的 `Result<RegionScan, RegionScanRefusal>` 与 crates `scan_region` 不 `==`。
- K-S3 重判段在写法甲、门槛来源「r2 产物按比例换」、c_rt 来源 r3 下，有一行的 `threshold_milliseconds`、`judgement_per_round`、`judgement`、`floor_judgement` 与 r3 第一段产物那一行不同；或线程钟那一半的点行不是 1776 行。
- K-S4 重判段现算的 ρ 与 r3 第一段产物打出的 `machine_ratio=` 到打出的位数不同。
- K-S5 输入文件变了：`out/g1.out` 不是 `61b210296634f2f20fa0465a4e4f09859cd9236522c54d4fb18301e75f243025`，r2 第二段产物不是 `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a`，r3 第一段产物不是 `840b68352e6d849e9a397dd71c3fb9d7a1f30c0a5b7f7f362e620b2a7ef69993`，r3 第三段产物不是 `31fd33a3a7672b9db7e13b54990ef4533db137a1c0f93bd521b66a07b8596851`；两份这一趟的产物与它们输入清单里记的 sha256 不同。
- K-S6 装置变了：输入清单第一行、第十二节执行员写的、产物 `name=apparatus` 行三处的装置 sha256 有一处不同（取代 r3 停机 J-S7 写死的那一个）。
- K-S7 拆法甲那一半：`central_mapping_as_the_compaction_reads_it` 与装置 `version_as_the_replay_reads_it` 取出的映射条目集合不同（r3 J-S6 原样）。
- K-S8 `mounted_session.rs` 第 3002–3005、3165、3267 行的原文与本地常量记的不同（取样分项认克隆的那几处挪了）。
- K-S9 取样分项名单里某个函数名在编出来的可执行文件里一处都找不到（crates 改了名）。
- 控漂移段另照 r3 的 J-S1 至 J-S5、J-S8 与时钟分辨率自检（原样）；写法与拆分段另照 r2 第二段的停机条款（常量回比、停机条款第 4 条、几何核那几条，原样）。

### 够判停机

问题单第 1、2、3 行都够判就停（第五节分段表「补完那一行够不够判」），没跑的量逐条标「够判后未跑」。控漂移段没跑成（清场没过、卡起不来）时：第 2 行不够判，交主 agent；重判段照跑，只报 c_rt 来源 r3 那一端（第 1 行照样够判）。写法与拆分段交回之后第 3 行已经够判而第 1 行还差重判段的，重判段照跑。

### 比对对象（这一次要与之比的上一次产物）

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out` | `840b68352e6d849e9a397dd71c3fb9d7a1f30c0a5b7f7f362e620b2a7ef69993` | 8007 | `replay.sh` 里没有 E185 的行；跑法第五节「跑法」控漂移段 | `e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out` | 第 2 行：复现臂、受控臂的轮次比与 `unstable` 数与它并列（D1–D5）；不带 `-2` 的那一份是 r3 停机作废的，不比 |
| 2 | `e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out` | `840b68352e6d849e9a397dd71c3fb9d7a1f30c0a5b7f7f362e620b2a7ef69993` | 8007 | 同上；跑法第五节「跑法」重判段 | `e185-r4-baseline-rejudge-2026-10-24.out.gz` | 第 1 行：c_rt 来源 r3 那一端，每一行点行重判、与它的判定逐行比（B4 的 `changed`；PR1、K-S3） |
| 3 | `e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` | `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a` | 1803 | 同上；跑法第五节「跑法」写法与拆分段 | `e185-gc-batch-wall-clock-split-cpu-writings-and-function-partition-r4-2026-10-24.out` | 写法甲这一趟的阶段 CPU 与它并列（B7） |
| 4 | `e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` | `f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a` | 1803 | 同上；跑法第五节「跑法」重判段 | `e185-r4-baseline-rejudge-2026-10-24.out.gz` | 门槛来源「r2 产物按比例换」那一端的底（B3） |
| 5 | `e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out` | `31fd33a3a7672b9db7e13b54990ef4533db137a1c0f93bd521b66a07b8596851` | 1258 | 同上；跑法第五节「跑法」写法与拆分段 | `e185-gc-batch-wall-clock-split-cpu-writings-and-function-partition-r4-2026-10-24.out` | 第 3 行：拆法甲在同一批调用上重跑的八行 `function_split_summary` 与它的八行并列（F5），只报 |

重跑产物名里的日期是写登记这一天；执行员起跑那一天不同的，在第十二节记新名，`.claude/gate.d/replay-comparison-units.tsv` 照新名登记。E185 没有单元表（C895），五行照 r3 的做法由执行员在那份表的文件头写「# 不登记的比对对象：research/prompts/e185-r4-prereg.md #<号>：<理由>；欠在 C895（E185 重跑的比对对象没有单元表）」；主 agent 定这一批顺带关 C895 的，照下一小节改登记成正式行。写之前门禁 doc-experiments 的 replay-comparison-units 格判这份登记红。

### C895（E185 重跑的比对对象没有单元表）顺带关不关（主 agent 定）

C895 要的（`.claude/kb/checks-owed.md` 第 574 行）：装置里立一张正式跑的单元表（照 E165 `formal_run_units` 的形），比对对象改登记成正式行、复跑列写 `replay.sh` 的驱动名；判别力是单元表少一条、格必须红。要关它，改这几处：

1. **装置的测试模块**（`mod tests`，第 15205 行起）新加一条测试，里面一个 `formal_run_units` 数组，照 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 第 18399 行的形：每条是（单元名字面、子命令、参数、留存产物名、这一单元跑的阳性对照名单）。列 r3 三条（第一段带 `-2` 那一份、第二段、第三段）与这一趟两条（控漂移段、写法与拆分段）；重判段不在装置里，它的单元写进 `scripts/experiment-apparatus/e185_r4_baseline_rejudge_test.py` 的同形数组。测试核三样：每个单元的子命令能被 `run_the_subcommand`（第 14979 行）认、参数能被 `options_of`（第 14944 行）与 `points_of`（第 14959 行）收下；每个单元点名的阳性对照在装置里都有判定函数，喂答案已知的合成读数时都判得出（复用 `positive_control_judges_hold_inside_their_bands_only` 第 15803 行、`thread_clock_control_judges_the_spin_and_the_sleep_at_the_registered_edges` 第 19354 行那一类）；全部单元并起来罩全 r3 登记与这一份登记的阳性对照表。
2. **子命令名立成常量**：`run_the_subcommand` 里的子命令字面量与单元表引同一份常量，不各写一份。
3. **装置文件头**（第 9–27 行的子命令清单）补两个新子命令。
4. **`scripts/experiment-apparatus/replay.sh` 的复跑表**（第 1492 行起的 `TABLE`）：E185 今天一行都没有，而 `.claude/gate.d/replay-comparison-units.tsv` 的正式行「复跑」列要写驱动名（写 `none:` 的行只缩不涨，新加的 `none:` 行格判红）。所以关 C895 还要给每个正式跑配一个驱动：重判段能做成逐字比的驱动（纯算术、几秒、产物逐字可复现）；控漂移段、写法与拆分段与 r3 那三段是真设备计时，要清场、要第二台、单趟 1–2 小时，复跑驱动只能比不随时序变的那几行（锚点、行数、对拍结果、停机条款），要另写一个比较器。
5. **登记表**：`.claude/gate.d/replay-comparison-units.tsv` 文件头 r3 那三行与这一趟的五行「不登记」换成正式行；`checks-owed.md` C895 由书记员还清。

代价（推的，没量）：1–3 约 150–250 行测试与常量；4 是大头，三到五个驱动加一个比较器约 300–500 行，外加它们自己的自证，复跑那几条是重型。不关的话，这一趟照 r3 的做法写五行「不登记」，格照 r3 那样判。

## 十二、修订

（装置改完之后、第一份产物之前由执行员写：改完的装置 sha256、重判脚本的 sha256、单测与变异条数、键名的新旧对照，只许收严或补量；起法因脚本改了而换写法、或两段改成先后起的，记一行。）

执行员写于 2026-10-24，三段的产物一份都还没跑。依据只有三样：第一节「主 agent 定」第 6 条（撤拆法乙）、派发提示「登记第十二节记装置 sha256、条数与前任定下的口径」「子命令名要改就在第十二节记一条修订」两句，与下面写明的单测、自证读数。

R4-1 改完的东西与 sha256（主工作区的装置由主 agent 打 `patch/crates.patch` 换上；sha256 是打上之后那一份）：
- 装置 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` `9e813cbd36900f1b0c7001e6de0f98de01080bbf7ac6aab39d8e7c9854bd97f8`（25850 行；改前 `4368c0e7…`、20497 行）。`#[test]` 174 条加到 230 条（新 56 条；`drift_control_without_the_gpu_feature_reports_not_run` 只在不带 `gpu` 特性时编）。
- 重判脚本 `scripts/experiment-apparatus/e185_r4_baseline_rejudge.py` `32f1421863eee648de35d756f59093bc3dadc2fa82e6ed30b1c9b56b7509f39b`（773 行）；自证 `e185_r4_baseline_rejudge_test.py` `81b888d413336bd7077810f232d7abb98fd76309757dd5f07dc68bf5b7c48675`（23 项）；变异表 `e185_r4_baseline_rejudge.mutations.tsv` `2192be951bbaabdb8b1c01bd85d59c3dbe0ed815ab62c7113b43c968fd204693`（22 行）。
- 起跑脚本 `scripts/experiment-apparatus/e185-r4-segment-run.sh`（登记的装置 sha256 写在它的 `registered_apparatus_sha256`，与上面那一个相同）。
- `crates/mutations.tsv` 追加 57 行：V155–V165、V169–V182（第九节表里那几行；V166–V168 见 R4-4，V183–V198 随拆法乙撤了，号不再用）与执行员补的 V199–V230（R4-5）。V1–V154 的锚点在新源码里逐条恰好命中一次。

R4-2 子命令改名（派发提示「子命令名要改就在第十二节记一条修订」）：`cpu-writings-and-function-partition` 改叫 `cpu-writings`（拆法乙撤了，名字只说它做什么）。产物名随之是 `research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-<日期>.out`，输入清单、串行声明是 `e185-r4-inputs-cpu-writings-<日期>.txt`、`e185-r4-serial-cpu-writings-<日期>.txt`；第十一节比对表 #3、#5 的重跑产物名照这个读。旧名在装置里不认（退 2，单测 `the_two_r4_subcommands_refuse_missing_and_unknown_options`）。

R4-3 随主 agent 定第 6 条撤掉的（不量、不打，第六节 F1–F6、第七节甲类第 3 条与乙类 P-sample、随机相位两行、第八节拆法乙两行与「补链」旋钮、第九节 V183–V198、第十条 L6–L8 与 L10、第十一节「取样的开销太大」与 `sample_buffer_full`、`addr2line` 两条「读不到」、停机 K-S7–K-S9）：写法与拆分段不再带 `-C force-frame-pointers=yes` 编；开跑只核 `sse4.2`（`addr2line`、`nm` 只给拆法乙用，不核）；输入清单不再记 `addr2line`、`nm` 的版本；不再在 K = 8 的调用上跑 r3 第三段的重放（F5 那一半）；重判段不读 r3 第三段产物（K-S5 少了 `31fd33a3…` 那一份；两段的输入清单照记它的 sha256）。

R4-4 前任定下、这一次照用的口径（派发提示「登记第十二节记…前任定下的口径」）：
- 控漂移段每格「后」的 GPU 读数取下一格开格前那一次 `nvidia-smi`（并进本来就起的那一次，不多起进程）；主机核那两项是量完当场读的。
- 产物键改名（过 `scripts/gate/data-name-lint.py`）：`scaling_cur_freq_khz` → `scaling_current_frequency_kilohertz`、`cpuinfo_avg_freq_khz` → `average_frequency_kilohertz`、`pstate` → `performance_state`（第六节 D6）。
- D2 的值大于 1 ÷ 0.90 时记 `above_the_registered_bands`，不当三档里的哪一档。
- B7（`name=today_over_r2`）与 V166–V168 落在重判脚本：门槛、系数乘法与比 r2 都是对产物的算术，装置不读 r2 产物、不算门槛。V166–V168 在脚本的变异表里，点名脚本自证的那一项。

R4-5 执行员补的（只补检查与判法的写死，不改量法）：
- 第七节乙类两条锚点进产物：控漂移段 `name=clock_round_summary` 带 `cells_expected=82 cells_hold=`，`name=round_ratio scope=point_lines` 带 `lines_expected=108`；写法与拆分段收尾打 `name=anchor item=cpu_writing_stage_lines value= expected=`（不等记停机）。
- 装置补 V199–V230 三十二行变异：新加的 56 条单测里第九节没点名的 32 条，各有一行改坏它要盯的那一处（show-me-test「新增的测试必须先证明它会红」）。
- 重判脚本加 R4J9–R4J19 十一行变异（输入 sha256、线程钟行数、B7 的边、旋钮翻面两端都要有判定、阳性对照红的写法整段作废、直接算那一端取写法 X 那一列、逐行复现不逐字比、D5、L1、B5 的数法、ρ 与 r3 打出的比）。
- 装置里 D1–D3 遇到 P-drift 没过写 `void_positive_control`、第十条 L4、L5 的触发抽成不带卡的纯函数（`drift_round_ratio_judgement`、`is_failure_clause_l4_triggered`、`is_failure_clause_l5_triggered`）各配单测；L4 那一行打 `card_one_judgement=` 与各张卡的 `warm_ups_reaching_the_cap=`。
- 重判脚本里「读不到」与「乙段增量为负」按（写法、门槛来源、点、候选）记 `status=not_run reason=…`，不让整份读不成；r3 的 Rust 算法遇到读不到是整份读不成，写法甲、r2 那一端两份算出来的一样（PR1 读数见下）。
- 候选 C 用 `out/g1.out` 原样那一个数只在（写法甲、门槛来源「r2 产物按比例换」）那一格；写法甲、「这一趟产物直接算」那一端照算法现算（g1 的数出自 r2 产物，不是这一趟的读数）。这一条是执行员对第一节「候选 C 在写法甲下的门槛」一句的读法，交主 agent 认或改。
- PW1、PW2 在每个点每轮头一条收下的意图的头一批上做（第五节阳性对照表「每个点每轮第一批」）；那一种写法红了，它的 `name=writing_factor` 写 `status=void_positive_control`，重判段那种写法两种门槛来源的门槛与判定全写 `void_positive_control`。
- 控漂移段与写法与拆分段段一开头（时钟自检之前）声明串行、段末声明并行，共用顶层 `within_a_declared_serial_segment`（V181、V182 改的就是它）；写法与拆分段每次调用之后的计时件另进计时窗口 `cpu_writing_pieces`（理由「E185 第 4 次重跑写法与拆分段：九种 CPU 写法的计时件与它们的 2 ms、两倍对照要在没有别的线程的进程里量」）。

R4-6 跑之前的读数（单测与自证，执行员在草稿目录与主工作区跑的；装置的 checker 档单测归主 agent，执行员没跑）：
- 重判脚本自证 23 项红 0；PR1 在三份已有产物上逐行复现 r3 第一段：`host_cpu_per_submission_point` 2684 行，门槛文字、每轮判定、判定、c_floor 判定逐字相同，对不上 0 条；现算 ρ 1.199658 与 r3 打出的 `machine_ratio` 相同。
- 重判脚本变异 22 行：抓到 22、无效 0、没红 0（每行红在点名那一项）。
- 装置 57 行变异只编不跑：锚点逐条恰好一次，57 条在 `cargo check --tests` 下全编得过；跑它们要 checker 档，交主 agent。

R4-7 起法没换：两段照第五节「跑法」那两条起；控漂移段与写法与拆分段的清场包装能不能同时起，主 agent 起跑前核，不能就先后起，再记一行。

## 十三、读过的文件与跑过的命令

### 读过的文件（行号区间；grep 命中行也算）

| 文件 | 读了哪些行 |
|---|---|
| `.claude/agent-common.md` | 1–111 |
| `.claude/agents/experiment-designer.md` | 派发时随定义给出（全文 1–70） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md`、`evidence-discipline.md` | 全文（1–122、1–179） |
| `.claude/rules/three-way-inference.md`、`mutation-sampling.md`、`implementation-first.md`、`test-script-discipline.md` | 全文（1–166、1–93、1–19、1–106） |
| `research/prompts/m4-gpugc-measure-r4-forks.md` | 1–11 |
| `research/prompts/m4-gpugc-design-r2-main-verification.md` | 1–68 |
| `research/prompts/e185-r3-prereg.md` | 1–680 |
| `research/prompts/e185-r2-prereg.md` | `grep -n '^#'`；`grep -n '甲\|乙\|丙\|crc_cpu\|scan_increment\|release_check\|choose_cpu'` 命中 15–433 间（15、17、31、32、36、40、43、49、50、52、132、140、146、159、160、166、168、174、190、195、200、202、204、208、215、219、259、268、274、290、292、293、300、302、310、313、315、328、340、341、354、355、366、372、373、394、403、413、425、433）；`grep -n 4188` 命中 163、492、567 |
| `research/prompts/e185-preregistration.md` | `grep -n '^#'`；`grep -n '甲段\|乙段\|丙段\|crc_cpu\|scan_increment\|release_check_crc\|重放'` 命中 45–605 间（45、46、47、49、214、223、224、282、289、304、310、312、316、324–327、329、362、380、420、464、480、482、484、488、489、507、511、578、586、588、599、605） |
| `research/prompts/m4-gpugc-design-r1-forks.md` | 16（`grep -n '^\| 10 '`） |
| `research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out` | 只算了 sha256，没读正文（数在 r3 登记第四节第 1 行读到） |
| `.claude/kb/checks-owed.md` | 574、584（`grep -n 'C905\|C895'`） |
| `.claude/kb/decisions/26-后台整理与放置回收.md` | 241（`grep`）；241–261（经 `quote-kb.py`） |
| `.claude/kb/decisions/24-后台重活能不能卸给GPU.md` | `grep -n '^#'`；30–46（经 `quote-kb.py`） |
| `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` | 没读 |
| `crates/singlefs-core/src/checksum.rs` | 1–120 |
| `crates/singlefs-core/src/region_scan.rs` | 12、20–35、36–48、60–150（`grep`）、150–420、145、422、452 |
| `crates/singlefs-core/src/mounted_session.rs` | `grep` 命中 34–3093 间；2588–2800；2800–2950；2985–3290；3355–3445；1956、1974、2615、2696、2722、2807、2835、2895、2922、2949、3277、3369、3443、3458、3488、3568（`grep`） |
| `crates/singlefs-core/src/unit.rs` | 11、17–19、119、231、294、385–410、426、507、581–600、720、757（`grep` 与 `sed`） |
| `crates/singlefs-core/src/compaction_plan.rs`；`records.rs`；`allocator.rs`；`transaction.rs` | 88、615–616、660、735；462、474；1815–1818；9676、9776、9852（`grep`） |
| `crates/singlefs-format/src/lib.rs` | 17、20（`grep`） |
| `crates/singlefs-core/src/` 下 `root_record.rs`、`instance_table_sweep.rs`、`system_configuration.rs`、`make_filesystem.rs`、`mounted_read.rs`、`journal.rs`、`recovery.rs`、`mount.rs` | `grep -rn 'crc32_castagnoli\|wide_checksum_field_holds'` 命中行 |
| `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` | 1–60；380–500；3440–3480；7630–7870；9440–9640；12559–12660；13117–13330；13450–13470；14188–14270；14322–14420；`grep` 命中 112、114、149、161、164、384、405、412、452、477、490、716、2037、2052、2638、2744、3392、3404、3461、3975、4156、4677、4691、4948、5723、5725、6322、7634、7900、8105–8296、9479–9552、9700–9767、10327、10934、10963、11004–11871、11968–12112、12569–12583、13003–14424、14542–15205、15209–19868 间的函数与测试名、19692 |
| `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` | 18380–18410；18760–18800 |
| `crates/mutations.tsv` | `grep -c 'E185 V'`；`grep 'E185 V' \| tail -2` |
| `Cargo.toml`；`.cargo/config.toml`；`Cargo.lock` | 18–23；全文；`grep -n '^name = "\(libc\|addr2line\|gimli\|object\|backtrace\)"'` 命中 360–361 |
| `scripts/experiment-apparatus/e185-r3-segment-run.sh` | 1–137 |
| `scripts/experiment-apparatus/replay.sh` | `grep` 命中 10、308、410–413、521、589、1310、1321–1323、1379、1382、1492、1515、1525、1946、1993、2006、2068 |
| `scripts/hosts/core_usage.py` | 1–80 里 `grep` 命中 2–70 间 |
| `scripts/hosts/multi-host-run.sh` | 1–60 里 `grep` 命中 2–54 间 |
| `scripts/agents/agent-watch.py`；`scripts/hosts/detached-run.sh` | `grep -rn SINGLEFS_SERIAL_DECLARATIONS` 命中行 |
| `.claude/hooks/lib_long_run.py` | 10–20（`grep -n CANONICAL_HOSTS_SENTENCE -A3`） |
| `.claude/gate.d/replay-comparison-units.tsv` | 1–30（每行前 300 字节）；27–29（`grep e185`） |
| `.claude/gate.d/doc-experiments.sh` | 27、34–39（`grep replay-comparison-units`） |
| `.claude/heavy-thread-default` | 全文 |
| `scripts/kb/quote-kb.py`；`scripts/kb/replace-once.py` | `--help` 前 30 行、前 20 行 |
| `research/results/e185*` | `ls`；四份产物的 `sha256sum` 与 `wc -l`、六份文件的 `stat` 时间（第十一节比对表、第四节第 5 行），正文一行没读 |
| 多机配置 | `crash-point-replay-shard-configuration-check.sh --emit-hosts` 的输出（私有值不抄） |

没读：`research/results/` 下任何产物的正文；E185 实验页。读到的结果数照第四节列着。

### 跑过的命令（原样；只读命令与写这份登记的命令，没编译、没跑装置、没跑门禁阶段）

第二台的主机名在命令里换成了〔第二台的主机名〕；`--emit-hosts` 打出的私有值不抄。

```bash
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m4-e185-r4-designer --purpose 'E185 第 4 次重跑登记的草稿'
which perf; cat /proc/sys/kernel/perf_event_paranoid; grep -o -m1 -w 'sse4_2\|pclmulqdq\|avx512f\|vpclmulqdq' /proc/cpuinfo | sort -u; lscpu | grep -E 'Model name|^CPU\(s\)|Thread|MHz'; cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor /sys/devices/system/cpu/cpu0/cpufreq/scaling_driver; cat /sys/devices/system/cpu/cpufreq/boost; ls /sys/devices/system/cpu/cpu0/cpufreq/; nvidia-smi --query-gpu=index,name,persistence_mode,pstate,clocks.sm,clocks.max.sm,clocks.mem,temperature.gpu,power.draw,clocks_event_reasons.active --format=csv; sudo -n true
grep -E '^CONFIG_HZ=|^CONFIG_NO_HZ_FULL=|^CONFIG_HIGH_RES_TIMERS=' /boot/config-$(uname -r)
timeout 20 ssh -o BatchMode=yes -o ConnectTimeout=8 〔第二台的主机名〕 'cat /proc/sys/kernel/perf_event_paranoid; which perf; grep -o -m1 -w "sse4_2" /proc/cpuinfo | head -1; lscpu | grep -E "Model name|^CPU\(s\)"; grep -E "^CONFIG_HZ=" /boot/config-$(uname -r); cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor'
timeout 20 ssh -o BatchMode=yes -o ConnectTimeout=8 〔第二台的主机名〕 'which nm addr2line; grep -o -m1 -w pclmulqdq /proc/cpuinfo; cat /sys/devices/system/cpu/nohz_full'
which nm addr2line; cat /sys/devices/system/cpu/nohz_full; cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_avg_freq; cat /sys/devices/system/cpu/cpu*/cpufreq/amd_pstate_prefcore_ranking | sort -n | uniq -c | tail -3
bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts
sha256sum crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs                 # 4368c0e7…
git show HEAD:crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs | sha256sum  # b3a50d50…
grep -c 'E185 V' crates/mutations.tsv; grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs   # 154；174
grep -rn '_mm_crc32\|crc32_u64\|sse4.2\|sse4_2' crates/ --include=*.rs | wc -l                     # 0
grep -n 'HashMap\|BTreeMap' crates/singlefs-core/src/region_scan.rs                                 # 12、145、422、452
grep -rn 'timer_create\|SIGPROF\|force-frame-pointers' crates/ scripts/ --include=*.rs --include=*.sh --include=*.toml | wc -l   # 0
grep -c 'SINGLEFS_SERIAL_DECLARATIONS' scripts/experiment-apparatus/e185-r3-segment-run.sh          # 0
grep -n -i 'e185' scripts/experiment-apparatus/replay.sh                                             # 零命中
sha256sum research/prompts/e185-r3-prereg.md research/prompts/e185-r2-prereg.md research/prompts/e185-preregistration.md research/prompts/m4-gpugc-measure-r4-forks.md research/prompts/m4-gpugc-design-r2-main-verification.md research/prompts/m4-gpugc-design-r1-forks.md research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out scripts/experiment-apparatus/e185-r3-segment-run.sh
cd research/results && sha256sum e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out; wc -l e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
for f in e185-r3-inputs-host-cpu-per-submission-2026-10-24-2.txt e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out e185-r3-inputs-function-split-2026-10-24.txt e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out e185-r3-inputs-recompute-disagreement-2026-10-24.txt e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out; do echo "$f $(stat -c '%W %Y' $f)"; done   # 再用 python 算时间差（第四节第 5 行；取回自第二台的那几份时间不作数）
D=/home/fy5090/.local/share/singlefs/work/m4-e185-r4-designer; nice -n 19 python3 scripts/kb/quote-kb.py $D/clauses.md '.claude/kb/decisions/26-后台整理与放置回收.md@#### 已定项 13：后台任务的计算放在哪、怎么开、什么节奏' '.claude/kb/decisions/24-后台重活能不能卸给GPU.md@#### 已定项 2：三条硬约束'
nice -n 19 python3 -I /home/fy5090/.local/share/singlefs/work/m4-e185-r4-designer/tools/anchors.py
TZ=Asia/Tokyo date +%F                                                                              # 2026-10-24
```

`quote-kb.py` 输出 `✓ 2 段整抄进 /home/fy5090/.local/share/singlefs/work/m4-e185-r4-designer/clauses.md，回读逐字节一致`；追加进本登记之后 `tail -n 47 登记 | diff - clauses.md` 无差（clauses.md 的 sha256 `711ed6db49a5c2a918566bf36de45ee354c6a88b6ca854efc0076df70f8e21ca`）。

本机与第二台的读数（原样；第二台那几行是设计员在行首加了「（第二台）」标出来，别的一个字不改；输出里没有第二台的主机名）：

```text
/usr/bin/perf
4
avx512f
pclmulqdq
sse4_2
vpclmulqdq
CPU(s):                                  32
Model name:                              AMD Ryzen 9 9950X 16-Core Processor
Thread(s) per core:                      2
performance
amd-pstate-epp
index, name, persistence_mode, pstate, clocks.current.sm [MHz], clocks.max.sm [MHz], clocks.current.memory [MHz], temperature.gpu, power.draw [W], clocks_event_reasons.active
0, NVIDIA GeForce RTX 5090, Enabled, P8, 195 MHz, 3105 MHz, 405 MHz, 30, 11.52 W, 0x0000000000000000
1, NVIDIA GeForce RTX 5060 Ti, Enabled, P8, 180 MHz, 3090 MHz, 405 MHz, 30, 9.66 W, 0x0000000000000000
sudo: a password is required
CONFIG_NO_HZ_FULL=y
CONFIG_HIGH_RES_TIMERS=y
CONFIG_HZ=1000
（第二台）4
（第二台）/usr/bin/perf
（第二台）sse4_2
（第二台）CPU(s):                                  32
（第二台）Model name:                              AMD Ryzen 9 9950X 16-Core Processor
（第二台）CONFIG_HZ=1000
（第二台）performance
（第二台）/usr/bin/nm
（第二台）/usr/bin/addr2line
（第二台）pclmulqdq
（第二台）(null)
```

本机 `nohz_full` 读出 `(null)`，`which nm addr2line` 是 `/usr/bin/nm`、`/usr/bin/addr2line`。r3 第一段带 `-2` 那一套的时间差：`host-cpu inputs_written_to_product_last_modified_seconds 1769`（另两段的产物取回自第二台或时间差不作数，不用）。四份产物的 `sha256sum` 与 `wc -l` 见第十一节比对表（`a56a0572…` 那一份是 r3 第二段，这一趟不用）。

锚点、轮次比与取样误差（第四节第 7 行，第七节乙类；`anchors.py` sha256 `0b665dda643c95521c2c891daa5b927a03b15136c70c787dfabbae336b35bec5`，脚本与输出原样）：

```python
import math
POLY=0x82F63B78
T=[]
for i in range(256):
    c=i
    for _ in range(8):
        c=(c>>1)^POLY if c&1 else c>>1
    T.append(c)
def crc_raw(data, c):
    for b in data:
        c=T[(c^b)&0xFF]^(c>>8)
    return c
def crc(data):
    return crc_raw(data,0xFFFFFFFF)^0xFFFFFFFF
# GF(2) shift of a raw register state by n zero bytes (zlib crc32_combine idea, Castagnoli polynomial)
def shift_zero_bytes(state, n):
    return crc_raw(bytes(n), state) if False else _shift(state, n)
def _mat_times(mat, vec):
    s=0; i=0
    while vec:
        if vec&1: s^=mat[i]
        vec>>=1; i+=1
    return s
def _mat_square(mat):
    return [_mat_times(mat, mat[n]) for n in range(32)]
def _shift(state, nbytes):
    odd=[POLY]+[1<<n for n in range(31)]
    even=_mat_square(odd)   # 2 zero bits
    odd=_mat_square(even)   # 4 zero bits
    n=nbytes
    while True:
        even=_mat_square(odd)
        if n&1: state=_mat_times(even,state)
        n>>=1
        if not n: break
        odd=_mat_square(even)
        if n&1: state=_mat_times(odd,state)
        n>>=1
        if not n: break
    return state
def lanes(length):
    base=(length//3)//8*8
    return [base, base, length-2*base]
def three_way(data):
    a,b,c=lanes(len(data))
    la=data[:a]; lb=data[a:a+b]; lc=data[a+b:]
    ra=crc_raw(la,0xFFFFFFFF); rb=crc_raw(lb,0); rc=crc_raw(lc,0)
    state=_shift(ra,b)^rb
    state=_shift(state,c)^rc
    return state^0xFFFFFFFF
print("check", hex(crc(b"123456789")))
for L in (32768,16384):
    buf=bytes(i%251 for i in range(L))
    print(f"length={L} lanes={lanes(L)} whole_crc32c={crc(buf):#010x} three_way_combined={three_way(buf):#010x} equal={crc(buf)==three_way(buf)}")
# drift control spins and the round-five over round-one ratio of the decreasing spin
spins=[8.0-1.9*(r-1) for r in range(1,6)]
print("decreasing_spin_milliseconds", [round(s,3) for s in spins])
for c in (0.05,1,3,6,14,30,32):
    print(f"c_rt_milliseconds={c} ratio_round5_over_round1={(c+spins[4])/(c+spins[0]):.4f}")
# r3 point lines on the thread clock (independent count)
cards=2; points=12; candidates=4; per_candidate=2*3+3; today=1; kinds=2
print("thread_clock_point_lines", cards*points*(candidates*per_candidate+today)*kinds, "per_card_candidate_a_machine_scaled", points*per_candidate)
# sampling: binomial standard error of a share
for n in (130,520,2000):
    for s in (0.1,0.3):
        print(f"samples={n} share={s} standard_error={math.sqrt(s*(1-s)/n):.4f}")
# random-phase counting error per point: calls x uniform phase variance 1/12
for calls,cpu_ms in ((10,26*5/10),(20,444*5/20)):
    total=calls*cpu_ms
    print(f"calls={calls} total_cpu_ms={total} counting_sd_relative={math.sqrt(calls/12)/total:.5f}")
```

```text
check 0xe3069283
length=32768 lanes=[10920, 10920, 10928] whole_crc32c=0x2ad60827 three_way_combined=0x2ad60827 equal=True
length=16384 lanes=[5456, 5456, 5472] whole_crc32c=0xeafca51d three_way_combined=0xeafca51d equal=True
decreasing_spin_milliseconds [8.0, 6.1, 4.2, 2.3, 0.4]
c_rt_milliseconds=0.05 ratio_round5_over_round1=0.0559
c_rt_milliseconds=1 ratio_round5_over_round1=0.1556
c_rt_milliseconds=3 ratio_round5_over_round1=0.3091
c_rt_milliseconds=6 ratio_round5_over_round1=0.4571
c_rt_milliseconds=14 ratio_round5_over_round1=0.6545
c_rt_milliseconds=30 ratio_round5_over_round1=0.8000
c_rt_milliseconds=32 ratio_round5_over_round1=0.8100
thread_clock_point_lines 1776 per_card_candidate_a_machine_scaled 108
samples=130 share=0.1 standard_error=0.0263
samples=130 share=0.3 standard_error=0.0402
samples=520 share=0.1 standard_error=0.0132
samples=520 share=0.3 standard_error=0.0201
samples=2000 share=0.1 standard_error=0.0067
samples=2000 share=0.3 standard_error=0.0102
calls=10 total_cpu_ms=130.0 counting_sd_relative=0.00702
calls=20 total_cpu_ms=2220.0 counting_sd_relative=0.00058
```

其余只读命令（`sed -n`、`awk 'NR==…'`、`grep -n`、`wc -l`、`ls`）读的行号都列在上一张表里。

第十四节的挂钟估算（推的；`wall_clock_estimate.py` sha256 `c9e01acfa995c1f652f843b4e46ba8d2776533b52977810cf87394686bcced5c`，命令 `nice -n 19 python3 -I /home/fy5090/.local/share/singlefs/work/m4-e185-r4-designer/tools/wall_clock_estimate.py`，脚本与输出原样）：

```python
# 第十四节的挂钟估算（推的）。依据：r3 登记第十三节的每张卡测量式（单次 3–12 ms、80 格、每轮 7600 次计时提交），
# r3 第一段带 -2 那一套从输入清单到产物 1769 秒（含编译），r1 第二段准备 4188.11 秒。
cards_measure=[(0.003,"lower"),(0.012,"upper")]
for per_sub,name in cards_measure:
    gap0_cells=8*3*2+8; gap50_cells=8*3; rounds=5
    per_card=rounds*(gap0_cells*110*per_sub+gap50_cells*60*(per_sub+0.05))
    probes=rounds*2*110*(per_sub+0.0042)
    extra_warm=(0 if name=="lower" else 90*80*rounds*per_sub)
    warm_to_stable=(60 if name=="lower" else 600)
    p1=(15 if name=="lower" else 45)
    smi=80*rounds*2*0.1
    build=0.17*3600; start=0.02*3600
    reproduction=per_card+probes+p1+smi
    controlled=2*(per_card+probes+extra_warm+warm_to_stable+p1+smi)
    total=build+start+reproduction+controlled
    print(f"drift_control {name}: per_card_measure_s={per_card:.0f} reproduction_s={reproduction:.0f} controlled_s={controlled:.0f} total_h={total/3600:.2f}")
for name,prep,measure in (("lower",4188.11,0.15*3600),("upper",4188.11*1.1,0.3*3600)):
    build=0.17*3600
    psample=2*300*0.011+20*0.05
    total=build+prep+measure+psample
    print(f"cpu_writings {name}: build_s={build:.0f} preparation_s={prep:.0f} measure_s={measure:.0f} controls_s={psample:.1f} total_h={total/3600:.2f}")
```

```text
drift_control lower: per_card_measure_s=474 reproduction_s=577 controlled_s=1274 total_h=0.70
drift_control upper: per_card_measure_s=816 reproduction_s=959 controlled_s=3982 total_h=1.56
cpu_writings lower: build_s=612 preparation_s=4188 measure_s=540 controls_s=7.6 total_h=1.49
cpu_writings upper: build_s=612 preparation_s=4607 measure_s=1080 controls_s=7.6 total_h=1.75
```

## 十四、并行与挂钟

- 独立单位：三段各自一条命令、各自一份产物；控漂移段（本机）与写法与拆分段（第二台）互相独立、可以同时跑，重判段读两段的产物、在两段之后；控漂移段里两条臂、两张卡、每张卡 82 格各自独立（每格自己热身、自己校准），只是一轮里按次序串着量；写法与拆分段的造池、几何核、第 2–5 轮重造照 r2 第二段的全局准备队列互相独立；重判段里每一行点行的重判互相独立。
- 线程：`E185_THREADS` 不设，取登记值 `.claude/heavy-thread-default`（今天 31），只用于写法与拆分段的准备队列；控漂移段整段一条线程（受控臂绑到一个核；驱动与 wgpu 在本进程里起的线程不归装置管，r3 原样由进程钟单报）；写法与拆分段的计时窗口一条线程；重判段一条线程；两段的编译经 `bash scripts/hosts/capped.sh 31`。
- 串行：控漂移段每次计时提交、每格都串行，量的是发起提交那一条线程的 CPU，别的线程抢 CPU 与 PCIe 会混进 c_rt；它的轮次比要的就是同一条线程上前后几轮的比，两条臂与两张卡只能先后量；写法与拆分段的每次调用、每次重放、P-sample 与开销探针串行，取样计时器只认本线程、同一时刻别的线程会把样本与整次调用的 CPU 一起搅乱；这些串行段都照第五节「串行声明」当场声明。两段在不同的机器上，计时窗口互不相扰，可以同时跑。
- 预计挂钟：控漂移段约 0.7–1.6 小时（推的：编约 0.17 小时；复现臂一张卡约 580–960 秒、受控臂两张卡约 1270–3980 秒，由 r3 的每张卡测量式加两格探针、热身到顶、暖到稳 60–600 秒算出，第十三节现算；r3 第一段两张卡连编实际约 1769 秒）；写法与拆分段约 1.5–1.8 小时（推的：编约 0.17 小时；准备 4188–4607 秒，r1 第二段实际 4188.11 秒、上沿多算一成；测量约 0.15–0.3 小时）；重判段几秒。两段同时跑时总挂钟约 1.5–1.8 小时，都在 2 小时内，不写分片、续跑、多机三样；主 agent 起跑前核出两段的清场包装不能同时起、要先后跑的，总挂钟约 2.2–3.3 小时，超过 2 小时，派发提示照「超过 2 小时」补「分片：」「续跑：」「多机：」三行（推的：分片单元是控漂移段的（臂、卡）与写法与拆分段的（点、轮），续跑入口要另写，交主 agent 定）。
- 预计在用核数：控漂移段编译那几分钟约 31 核，测量时 1 核加驱动线程（推的，1–2 核）；写法与拆分段编译约 31 核、准备 5–15 核（r2 登记第十四节，`E185_MEMORY_CAP=40G` 下约 5 份同时）、计时窗口 1 核；重判段 1 核。
