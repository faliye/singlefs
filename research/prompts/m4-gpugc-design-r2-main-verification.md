# m4-gpugc-design-r2：里程碑四七·2 GPU 垃圾回收设计轮，第二轮判决（2026-10-24）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 -->

被判的是正文 `research/prompts/_m4-gpugc-design-r2-body.md` 的八格 G1–G8。腿的报告：
- 辩方（Sonnet）：`research/prompts/m4-gpugc-design-r2-sonnet-output.md`，sha256 前缀 `f04217d5`，复核第一轮判决 `research/prompts/m4-gpugc-design-r1-main-verification.md`；
- 攻方（Opus）：`research/prompts/m4-gpugc-design-r2-opus-output.md`，sha256 前缀 `ab260957`，模型目录 `research/prompts/m4-gpugc-design-r2-opus-model/`。

这是这一题的第二轮。照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，第一、二轮都打中的算打穿；只在一轮打中的仍挂着，进第三轮。

开工快照：`research/prompts/m4-gpugc-design-r2-snapshot/`

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 开工快照 | `sha256sum -c research/prompts/m4-gpugc-design-r2-snapshot/sha256sums.txt` 与 `kb-sha256.txt` | 21 份与 kb 那份全 OK（腿跑着的时候快照里的文件没人改）。快照清单存成 `sha256sums.txt`，`scripts/agents/snapshot-tree.py` 认的是 `judged-sha256.txt`，判不了，照第一轮的做法直接 `sha256sum -c` |
| 两条腿的引文 | `python3 scripts/agents/cite-check.py <两份报告> --root . --background research/prompts/_m4-gpugc-design-r2-background.md` | 「✓ 核了 77 处引文、24 处裸行号，对上 76 处，没判 25 处（其中裸行号没有引文可比 24 处；逐处列在上面）」。非裸行号的那 1 处是攻方第 136 行引的仓外 `wgpu-types-30.0.1/src/limits.rs:441`，没核（仓外源码，判决不依赖它） |
| 判决要用的代码行（主 agent 逐行读主工作区） | 读 | `crates/singlefs-core/src/region_scan.rs` 第 286–289、309–312 行：每条记录在全部映射条目上 `.iter().filter` / `.iter().any`。<br>`crates/singlefs-core/src/compaction_plan.rs` 第 812–816 行：一趟发布预算 `(3 * root_ring_slots_per_region).saturating_sub(4).saturating_sub(publishes_already_passed).max(1)`。<br>`crates/singlefs-core/src/mounted_session.rs` 第 2704 行：`let mut last_completed_intent: Option<CompactionIntent> = None;`，一趟里的局部变量。<br>`crates/singlefs-core/src/transaction.rs` 第 9663–9670 行的文档注释：整理的搬迁「准入不归删除类」「搬迁副本不进重写的角色、不进需求，所以两判都不判」。<br>`crates/singlefs-core/src/checksum.rs` 第 3 行：「实现用 slicing-by-8」 |
| 本机显卡 | `lspci \| grep -iE 'vga\|display\|3d'`、`ls /usr/share/vulkan/icd.d/` | 三张：`01:00.0`、`10:00.0` 两张 NVIDIA，`79:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Device 13c0 (rev c1)`；ICD 有 `radeon_icd.json`、`lvp_icd.json`（软件 Vulkan）在内的 9 份。攻方「本机有第三张显卡，AMD 集显」坐实 |
| 第一段产物的口径 | 主 agent 逐行读 `research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out` 的 `host_cpu_per_submission_point` 行现算 | ① 行数 2684（`grep -c`）；`judgement` 那一列 losing 1137、not_losing 398、not_losing_lower_bound 95、unstable 1054。岔路单第 1 行先前写的「5368 行、含 unstable 1529 行」是把 `floor_judgement` 一起数了，主 agent 的错，这一轮改正。<br>② 候选 C 与 A 在同一 (f, P, K, 卡, 输入, 等法, 间隔, 钟, 门槛种类) 下的 `host_cpu_per_submission_milliseconds_per_round` 672 组逐字相同：C 与 A 只差门槛（省下的主机 CPU），不差每次提交的主机 CPU（攻方报 432 组，口径不同，结论同）。<br>③ 候选 A、机器比例门槛、线程钟：第 5 轮 ÷ 第 1 轮的中位，卡 0 0.528（108 格）、卡 1 0.958（108 格）。卡 0 的读数随轮次往下走，前后轮混着机器状态（辩方报终点校准比起点快 12.4%） |
| 第三段产物的口径 | 读 `research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out` 的八行 `function_split_summary`，把 `call`、`remainder` 之外各项 `_share_median` 相加 | 第一行 0.888（`remainder_share_median=0.110173`），其余七行 1.167–2.517、余项为负（例 2.517 与 `-1.516256`）。重放比真调用贵，份额不是对一次调用的划分（两条腿都指出）。岔路单第 10 行先前标的「够判」不成立，改回「开着」 |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-gpugc-design-r2-main-check/model/`，`bash scripts/hosts/run-with-memory-cap.sh 2G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <草稿>/run` | 退 0，15 行 `name=rerun_compare model=… identical=true`（pivot、pivot_wait、readings、decompose、judged_points、identity_index、c_inputs、a_per_intent、ratio_end、round_robin、drift、foreground_wall、hardware_crc、combined、g8） |
| 腿在副本或探针上量的数 | — | 攻方「单路 SSE4.2 比 slicing-by-8 快 3.98 倍」是它在草稿目录探针上量的；辩方「把乙换成 P=1024 的量，C 卡 0 的 not_losing 从 52/54 掉到 25/54」是代理数。照「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」，两样都不进 kb，判决只当线索用 |

## 二、判定与去向

两条腿在一件事上一致：**第一段里唯一成批不赔的候选 C，它的赢落在 CPU 模式今天的两处慢路径上**——区域扫描逐条在全部映射条目上线性找（`region_scan.rs` 第 286、309 行），与软件 slicing-by-8 的 CRC（`checksum.rs` 第 3 行）。C 与 A 的每次提交主机 CPU 逐字相同（第一节 ②），C 只赢在省下的那一份；那一份的大头是这两处。攻方把两处都换成更快的 CPU 写法之后（指令 CRC 量的、按身份查推的替换，都在它的模型上），间隔 50 ms 下 C 一格不赔都没有。这两样都没在入库装置上量过。

| 格 | 第一轮 | 第二轮辩方 | 第二轮攻方 | 判定 |
|---|---|---|---|---|
| G1 GPU 上算哪几段 | 打中三处，挂起 | ① 站得住、已量到；② 只打中 D；③ 打中 A–D 全部，被新读数加强（一次提交挂钟 5.46–8.05 ms） | 打中：C 稳赢的点赢在线性查找与软件 CRC | **打穿（两轮都中）**：G1-③（今天的形态下开 GPU 就进前台路径）与 G1-①（净收益）两处打穿。① 的结论从「符号不知道」收严成「候选 A、B、D 在登记读法下两张卡都出局；C 的赢依赖 CPU 模式今天的两处慢路径，那两处换掉之后符号没量过」 |
| G2 攒批形态 | 打中，挂起 | 站得住，几乎不决定什么（`merging_shape` 432 行只 1 行跨意图） | 打中：卡 0 的 9 条拟合 7 条不线性，跟着卡 0 的漂移走；墙钟的「固定开销」是核逐位串行的深度 | **打穿（两轮都中）**：跨批攒成不了，第一段读数里跨意图只 1 格；拟合在卡 0 上不可信，卡 1 上可用 |
| G3 多卡分活 | 没打中，脆弱 | 漏判一处：卡 1 在 16 MiB 一档线程 CPU 高于 CPU 自己算 | 反例：按窗口轮转让 C 有 4 格由不赔翻成赔、23 格翻成不稳定 | **一轮打中，挂起进第三轮**（第一轮没中，第二轮两条腿都中） |
| G4 两份核比对与 N | 打中两处，挂起 | ① 站得住，同样打中所有两核共用输入的方案；N 不由零次翻面 | 没打中：零次上界只罩两份 CRC 核 | **① 打穿（第一轮中，第二轮辩方复核站得住）**；② 与 N 留第三轮 |
| G5 控制接口 | 没打中，脆弱 | 没漏判，并进 G8 | 反例：先请求切到 cpu、这一批 GPU 随后出错、GC 停下，条款要的「人切回 cpu」已在出错之前发生 | **一轮打中，挂起进第三轮** |
| G6 三个默认值 | 打中，挂起 | 站得住：不挪出时「fsync 在等」那一条空真 | 改法够不着（推的） | **打穿（第一轮中，第二轮辩方复核站得住）**；改法留第三轮 |
| G7 代码住哪 | 没打中，脆弱 | 没漏判；另有共同因素：所有门槛按软件 CRC 算 | 反例（推的）：checker 档把 `wgpu` 钉成 `=30.0.1`，产品与 checker 档只能共用 naga 与驱动这条翻译链 | **一轮打中（推的），挂起进第三轮** |
| G8 GC 挪不挪出调用线程 | （第一轮判决新冒出） | 不挪出：前台每次净增 5.50–14.76 ms，占 CPU 模式一批挂钟 4.5%–118%，字面仍违 D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条 | 挪出之后丢三样保证：一趟发布预算的 `.max(1)` 加定时兜底，空闲时整理发布没有上限（模型 g8：200 拍 213 次，线是 20）；签下一条意图的 `last_completed_intent` 跨趟不留；整理批不过准入，挪出后能插进用户改动的抬 F 与再判之间 | **第一轮，挂起进第三轮**。两边都有代价：不挪出违 D24 已定项 2 第 2 条，挪出要补三样保证。挪出的主机 CPU 代价（c_rt 不因挪线程而变，辩方）没有数 |

第一轮推的五条改法（零轮 → 被攻一轮）：
- 按设备类型筛卡：**打中**（本机第三张卡是 AMD 集显，E185 只量了两张 NVIDIA）。
- 对拍从这一版起算：**不够**（推的：小池测试碰不到 `wgpu` 的派活与绑定上限）。
- 重挂时把不确定的条目交 CPU：**打中**（不确定的条目只记在内存里，持久化又撞「不写进盘」）。
- 挂载时用已知答案自检：**不够**（构造不出 crates 上的实例）。
- GC 挪出调用线程：**并进 G8**。

**条款与代码的冲突（与 GPU 无关，攻方 G8 顺带打中）**：D26（后台整理与放置回收） 已定项 3 第 2 句「整理不是释放路径，它走常规准入，批次最坏占用计入『已承诺预留』」（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 100 行），与 `transaction.rs` 第 9663–9670 行的注释「搬迁副本不进重写的角色、不进需求，所以两判都不判」对不上。不在这一轮判，记进 `.claude/kb/checks-owed.md` C906（整理批不过准入与 D26 已定项 3 对不上） 交用户。

## 三、下一步

1. **第三轮（`m4-gpugc-design-r3`）之前先量两样，在入库装置上**：照「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」与「岔路交用户定之前，每条路要有代价数」——
   - CPU 模式换成硬件 CRC 指令、区域扫描换成按身份查之后，一批的主机 CPU；拿它重算候选 A–D 的门槛，判第一段的「不赔」还剩多少；
   - 卡 0 的轮次漂移：第一段在卡 0 上换一种控住暖机与频率的跑法重量，判 1054 格「不稳定」里有多少是漂移；
   - 第三段换一种是划分的拆法（份额相加等于整次调用）。

   主 agent 先写问题单，交实验设计员写 E185 第 4 次重跑登记。
2. **第三轮攻法**（`.claude/rules/three-way-inference.md` 第三轮分工）：「已定」项撑不撑得住、未定项清单的完整性、这一轮报出的数字能不能核。挂着进第三轮的：G3、G5、G7、G8，G4 ② 与 N，G6 的改法。第三轮之后停。
3. **岔路单**：第 1 行的行数改正；第 10 行改回「开着」；新增第 11 行「GC 挪不挪出调用线程」（G8）。这一轮不交用户：G1 的结论挂在第三节第 1 条那两样没量的数上。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-10-24 不受影响：打穿的 G1、G2、G4 ①、G6 都是对七·2 设计候选的判定，已定项 13 的八条条款本身不改；第 7 条那把尺下 GPU 模式赚不赚，等第三节第 1 条的数 |
| D26（后台整理与放置回收） 已定项 3 | 备料 | 2026-10-24 不受影响：第 2 句与 `transaction.rs` 第 9663–9670 行对不上，登记欠账交用户，条款不在这一轮改 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-24 不受影响：第 2 条「不许进前台路径」是 G1-③ 与 G8 的依据，条款本身不改 |
| D13（验证路线） | 备料 | 2026-10-24 不受影响：G7 判器与产品共不共用 GPU 代码照现行；攻方的 `wgpu` 版本钉法那一条留第三轮 |
