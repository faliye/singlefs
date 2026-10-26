# m4-gpugc-design-r1：里程碑四七·2 GPU 垃圾回收设计轮，第一轮判决（2026-10-23）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 Q9 Q13 Q17 F7 -->

被判的是正文 `research/prompts/_m4-gpugc-design-r1-body.md` 的七格 G1–G7。腿的报告：
- 正推（Sonnet）：`research/prompts/m4-gpugc-design-r1-sonnet-output.md`，sha256 前缀 `004e39e6`；
- 攻方（Opus）：`research/prompts/m4-gpugc-design-r1-opus-output.md`，sha256 前缀 `d9a74da7`，模型目录 `research/prompts/m4-gpugc-design-r1-opus-model/`。

这是这一题的第一轮。照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，这一轮打中的先挂起，不写进 kb 正文。

开工快照：`research/prompts/m4-gpugc-design-r1-snapshot/`

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 开工快照 | `sha256sum -c research/prompts/m4-gpugc-design-r1-snapshot/sha256sums.txt` 与 `kb-sha256.txt` | `sha256sums.txt` 13 份全 OK。`kb-sha256.txt` 三份对不上：<br>两份是 E185 实验页的英日译文，主 agent 在两条腿跑着的时候装了新译文；清单里这两份全标「不抄」，两条腿都没引。<br>一份是 `.claude/kb/checks-owed.md`，别的会话改的；腿引的只有 C412 那一行，`git diff -U0 HEAD` 里没有以 `| C412 ` 起头的增删行，第 291 行仍是 C412 |
| 两条腿的引文 | `python3 scripts/agents/cite-check.py <两份报告> --root . --background research/prompts/_m4-gpugc-design-r1-background.md` | 「✓ 核了 96 处引文、99 处裸行号，对上 95 处，没判 100 处」。没对上的那 1 处与没判的 7 处是正推引的 `wgpu` 包源码（仓外）；加 `--root ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f` 单跑正推那一份：「✓ 核了 43 处引文、96 处裸行号，对上 43 处」 |
| 判决要用的代码行（主 agent 逐行读） | 读主工作区 | `crates/singlefs-core/src/mounted_session.rs`：<br>第 1928–1931 行，用户改动的发布函数里、返回之前起 `run_compaction_on`；<br>第 2641 行，开关在一趟开头读一次；<br>第 2975 行的文档注释，「每批之前都重扫」；<br>第 2989 行，`advance_compaction_by_one_publish`。<br>`crates/singlefs-core/src/region_scan.rs` 第 286–288 行与第 309–311 行，每条记录在全部映射条目上 `.iter().filter` / `.iter().any`。<br>`crates/singlefs-core/src/logged_operations.rs` 第 303 行，`slot_count != CLUSTER_SEGMENT_SLOTS` 拒收。<br>`.claude/kb/decisions/26-后台整理与放置回收.md` 第 64 行，「`R` 的槽数 … 恒 64（一个聚簇段），别的值拒收」。<br>`crates/singlefs-core/src/compaction_plan.rs`：第 808 行 `publish_budget_of_a_compaction_run`，`3S − 4 − 已经过去的发布数`；第 825 行，产品路径恒不起跑。<br>`crates/singlefs-core/src/transaction.rs` 第 10137 行，`relocation_checked_against_the_version` |
| 腿引的产物行 | 在产物里逐字找 | 正推的「一批最大 4.9067%」：`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` 第 989 行 `call_cpu_nanoseconds=13894421 crc_cpu_nanoseconds=259441.000000 … scan_increment_cpu_nanoseconds=77751.000000 … release_check_crc_cpu_nanoseconds=344572.000000`，主 agent 现算 (259441 + 344572 + 77751) ÷ 13894421 = 0.049067。攻方的每次提交固定挂钟：`research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out` 第 54 行 `intercept_seconds=0.004492`、第 136 行 `intercept_seconds=0.010300` |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-gpugc-design-r1-main-check/model/`，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <草稿>/run --model <那份>` | 退 0；g1、g2、g4、g6 四份输出与攻方 `out/` 逐字节相同（`cmp`），日志 `name=rerun_compare model=g2 identical=true` 等四行。判决用的数：<br>`name=g1_ledger_summary points=12 break_even_ms_min=0.2159 break_even_ms_max=0.7214 a_card0_ms=4.492 a_card1_ms=10.300`；<br>g4 两行 `name=g4_test_same_input_cpu_vs_gpu differing_entries=0` 与 `name=g4_end_to_end_modes differing_entries=3 gpu_dead_cpu_movable=3`（seed 185、`kept_only=3`）；<br>g2 `runs_needed_at_least=8`–`38`；<br>g6 真盘臂一批 `wall_ms_median=23.728`–`123.369` |
| 正推腿的读数 | — | 正推的 `tools/` 脚本没有单独复跑。判决里引到的正推数（4.59%、0.54 ms、φ ≥ 0.89、70–94 MiB）逐处标「正推量的」 |

## 二、判定与去向

两条腿在一件事上一致：**决定 GPU 模式按 D26（后台整理与放置回收） 已定项 13 第 7 条赚还是赔的那个数，E185 没量过**。那个数是一次 GPU 提交在主机线程上花的 CPU，包括准备、提交、等、读回与两份比对；正推记作 c_rt，攻方记作每次提交的主机 CPU。
- 攻方模型 g1 给出门槛：c_rt 低于 0.216–0.721 ms 才不赔（复跑）。
- 正推给出上界：甲乙丙加释放前核合计最多占一条意图主机 CPU 的 4.59%（正推量的；单批调用口径主机 CPU 现算 4.9067%）。
- E185 只量到 GPU 一侧的挂钟：每次提交固定 4.492 ms（卡 0）、10.300 ms（卡 1）。
- 因此七格里凡是要比「省多少」的推荐，这一轮都判不了。

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| G1 GPU 上算哪几段 | 第一版只放扫描里的校验和；净收益符号不知道；发布那一半不放；Q9 不算翻面量 | 打中三处（G1-① 余量、G1-② 发布那一半、G1-③ 前台路径） | **打中，挂起**：<br>① 净收益的符号这一轮判不了，两条腿一致，缺 c_rt，进实验。<br>② 发布那一半里没有能放 GPU、又不碰共享事务层的部分，两条腿一致；释放前读盘核住在共用的发布准备函数里（`transaction.rs` 第 11216 行起的文档注释）。<br>③ 今天的 GC 在用户改动的发布函数里、返回之前跑一批（`mounted_session.rs` 第 1928–1931 行，主 agent 读过）。照这个形态开 GPU，就把一次 GPU 往返放进了前台调用的返回路径，与 D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」相撞。GC 先挪出调用线程是 G1 任何推荐的前提，七·2 那一行没列这件事（攻方 G1-③）。<br>Q9：两条腿都判不算翻面量，理由都是 D26 已定项 13 第 7 条已写明挂钟判据不管它，主 agent 认 |
| G2 攒批形态 | 推荐乙-1（意图内一次交、窗口开头验一次字节、每批只重扫状态）；φ 压不下（≥ 0.89，正推量的） | 打中（乙撞三条串行约束；丙要改格式；让甲出局的 Q13 判的是挂钟份额） | **打中，挂起**：<br>每批之前都重扫（`mounted_session.rs` 第 2975 行）、全池至多一条活意图、一趟发布预算 `3S − 4`（`compaction_plan.rs` 第 808 行），这三条让跨批攒不成，攻方模型 g2 复跑：攒 125 批至少要 8–13 趟。<br>放宽 R 要改 `logged_operations.rs` 第 303 行拒收的槽数字段，与 D26 第 64 行「恒 64」相撞。<br>乙-1 在两条腿下都压不下固定开销，净赚与否同样看 c_rt |
| G3 多卡分活 | 推荐 PCI 总线号、已验厂商独显、按窗口轮转 | 脆弱：「全部显卡」里会混进软件 Vulkan 适配器（推的，没起 GPU）；一张卡与几张卡在显存翻位下判定不同 | **没打中，两处脆弱记进第二轮**。正推的推荐被攻过零轮 |
| G4 两份核比对与 N | 推荐两种算法、各一块输入缓冲、比对在 CPU、N = 2 | 打中两处：<br>① 输入表准备这一类，七·3 写的同输入对拍抓不到，模型 g4 复跑：同输入对拍 0 处不同，两种模式各读这一版再比才 3 处不同；<br>② 「CPU 只改位图，不再抽样复核」与今天发布路径逐条核计划（`transaction.rs` 第 10137 行 `relocation_checked_against_the_version`）的关系没写清；<br>另一处脆弱：N 的计数住内存，重挂清零 | **打中，挂起**。①要改的是七·3 那条对拍的起点（从这一版起算、流里带快照），是测试设计。② 是条款与代码对不上，下一轮核 |
| G5 控制接口 | 推荐会话方法、在 `advance_compaction_by_one_publish` 进门读一次 | 脆弱：开关在一趟开头读一次，GC 挪出调用线程之后「下一批」会退成「下一趟」 | **没打中**，脆弱那一处与 G1-③ 同一件事（GC 挪不挪出调用线程），并进 G1-③ 那一条岔路 |
| G6 三个默认值 | 推荐写带宽上限默认不限、兜底间隔默认 0（关）、显存上限按用途上限取窗口；兜底与预算 `B_gc` 冲突 | 打中：「前台有 fsync 在等」在今天的独占借用会话里观测不到；写带宽上限压不住一批由 CPU 定的挂钟 | **打中，挂起**。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那张表，这是「判别子观测不到」那一形：GC 在用户改动的调用里同步跑，没有并发的 fsync 可等。改法同样是 GC 挪出调用线程（G1-③），两件事一起定 |
| G7 代码住哪 | 推荐新包、核心层不依赖它、CPU 参照用核心层现成的、与 checker 档不共用；真卡对拍归 checker 档、假后端进快档 | 脆弱：两份核都要在 WGSL 里自拼 u64 与字节寻址，是两份核的共同前提 | **没打中**。正推指出七·3 验收「对拍进快档」与 `.claude/rules/verification.md` 两档定义相左（要真设备的归 checker 档），主 agent 读两处原文认这一处差，记进七·3 的条款修订 |

**这一轮没打中的**：G3、G5、G7 只有脆弱点，没有反例。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，「没打中」一次不算，第二轮另派一条同立场攻方腿再攻这三格。

## 三、下一步

1. **先量 c_rt，再开第二轮。** 照 `.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」：G1、G2、G4、G3 的岔路都挂在这一个数上，第二轮之前先建实验量它。要量的：
   - 一次 GPU 提交在主机线程上的 CPU，按提交大小、两张卡、阻塞等与睡眠等各量（正推 G1-a ①、攻方 G1 岔路第一行）；
   - 拟合成固定部分加每字节部分（正推 G2-a）；
   - 改身份索引之后一批 CPU 按函数拆开，读这一版、克隆、发布各占几成（正推 G1-d）。

   主 agent 先写问题单，交实验设计员写跑前登记。
2. **第二轮（`m4-gpugc-design-r2`）在实验交回后开。** 照第二轮分工攻：第一轮的修补本身、产生结论的方法、从未被看过的地方。辩方腿复核这份判决。要攻的：
   - 「GC 挪出调用线程」这一前提；
   - 两条腿各自的改法（按设备类型筛卡、对拍从这一版起算、重挂时把不确定的条目直接交 CPU、挂载时用已知答案自检）；
   - 第一轮没打中的 G3、G5、G7。
3. **CPU 侧可修的一处，记进延后项。** 正推报乙增量的绝对 CPU 随 P 线性涨（0.349 到 2.414 ms，正推量的），原因是 `region_scan.rs` 第 286、309 行每条记录在全部映射条目上线性找，主 agent 读过形状。它与 GPU 无关，是七·6 同一类的按身份查找问题，不挡七·2，登记进里程碑四的延后项。

岔路单：`research/prompts/m4-gpugc-design-r1-forks.md`。这一轮不交用户：每条路都还缺 c_rt 这个代价数。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-10-23 不受影响：第一轮打中的（第 3 条攒批与每批重扫相撞、第 6 条判别子观测不到、第 8 条输入准备那一类的对拍）挂起，三轮多数打穿之前不改正文 |
| D26（后台整理与放置回收） 已定项 1 | 备料 | 2026-10-23 不受影响：「R 槽数恒 64」照旧；放宽 R 是岔路的一个候选，没定 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-23 不受影响：第 2 条「不许进前台路径」是 G1-③ 打中的依据，条款本身不改 |
| D24（后台重活能不能卸给 GPU） 已定项 3 | 备料 | 2026-10-23 不受影响：Q9 不算翻面量，依据是 D26 已定项 13 第 7 条已写明不管 |
| D13（验证路线） | 备料 | 2026-10-23 不受影响：G7 判器与产品不共用 GPU 代码，照现行 |
