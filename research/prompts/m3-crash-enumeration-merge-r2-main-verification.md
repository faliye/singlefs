# 崩溃点重放合并：设计轮第二轮判决（2026-10-10）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 L1 L2 L3 L4 L5 L6 K1 K2 K3 K4 K5 K6 K7 U1 U2 U3 U4 U5 U6 U7 U8 U9 W -->

正文 `research/prompts/_m3-crash-enumeration-merge-r2-body.md`（被判形态 G1–G14，格 L1–L6）。两腿报告：
- 攻方：`research/prompts/m3-crash-enumeration-merge-r2-opus-output.md`，sha256 d55bd889…8839，310 行；模型 `research/prompts/m3-crash-enumeration-merge-r2-opus-model/`；
- 辩方：`research/prompts/m3-crash-enumeration-merge-r2-sonnet-output.md`，sha256 b0036401…7e11，121 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | `research/prompts/m3-crash-enumeration-merge-r2-snapshot/crates-sha256.txt` 91 份两腿交齐后 `sha256sum -c --quiet` 全过；`kb-sha256.txt` 里 `.claude/kb/checks-owed.md` 不符，是主 agent 在两腿跑时改的（C650、C654、C657 三处必红世界表旧路径换新名，加 C773 一行，`git diff` 核过），两腿都没引这份 |
| 引文 | 辩方单份交回时 `cite-check.py`：「✓ 核了 60 处引文，对上 60 处，没判 0 处」；两份一起再核：「✗ 核了 136 处引文，2 处对不上」，两处都是辩方引 `.claude/hooks/lib_heavy_tests.py:585`「CRASH_AMPLIFICATION_SCRIPTS = frozenset」，今天在第 635 行：这份不在快照里，T3a 在两腿交回之间往它前面加了行（工作区对 HEAD `302 insertions(+), 13 deletions(-)`）。腿引的时候是对的，不算引错（`~/.local/share/singlefs/work/m3-crash-enumeration-merge-r2-main-rerun/cite-check.log`） |
| 复跑攻方模型 | 拷进 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r2-main-rerun/model/`，`run-with-memory-cap.sh 4G bash rerun.sh` 退 0，两份末行「MODEL_SELFCHECK compared=15 mismatched=0 []」「MODEL_SELFCHECK compared=11 mismatched=0 []」；`MODEL sharing=code_today unlink_shared=16777259 distinct_states=5133787951 … gpu_serial_item_walls_hours=10.8 … cpu_catch_up_hours=80..220`、`MODEL sharing_lost_if_versions_are_per_item states=1426456802`、`MODEL observer_edit_gpu_rejudge_hours=1.30..2.40` 与报告逐字相同 |
| 主 agent 现查的代码事实 | ① B、C 两步各在自己的测试文件里设点：`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1100`「let second = recorder.step(file!(), "sequential_write", \|\| {」与多记录流第 434 行同形，标签带设点文件（`crates/singlefs-checker-tier/src/crash_identity.rs:27` `code_file_in_repository`，身份串第 70–71 行拼进去）；另一条独立路：`records/2026-10-01-删除与并发事务.md:1292` 的「item=multi-record-publish exit=0 states=1342177375 reused=16777259」恰等于攻方按代码算的共享量。两条不共用代码与采样，L2-6 坐实。② GPU 段内序号只用低 32 位：`crash_judge_common.wgsl:3528`「var remaining = ordinal_lo - tab(T_SEGMENTS, s * 6u + 2u);」，只有第 ② 段核对器核段长（`crash_verify_gpu.rs:304`）。③ GPU 结论字只有违例位，没有逐条判到 / 不适用（`crash_judge_gpu.rs:10–14`）。④ `grep -c InPlaceSubsets` 在 `crash_judge_tables.rs`、`crash_amplification.rs`、`crash_judge_common.wgsl` 都是 0。⑤ `crates/singlefs-core/src/transaction.rs:1873` 写 `crate::mount::`，所以 `mount.rs` 在判法闭包里（辩方第 3 节读码，攻方第一轮探针汇总同向）。⑥ `crash_amplification.rs` 8122 行，`mod tests {` 在第 3575 行；契约测试的「不该动」表里有它（`crash_identity.rs:1464`）。⑦ 三份引擎文件 grep「等价」0 处 |
| 没跑的 | 攻方三条、辩方三条要主 agent 跑的命令都是 checker 档或要卡。L2-6 已由第 ① 格两条路坐实，不跑攻方第 3 条；判法闭包由第 ⑤ 格读码坐实，不跑辩方第 1 条；各项整项挂钟与 CPU 每态速率（攻方第 1、2 条，约 10–11 小时与 2 小时）没跑，第三节代价数照写「推的，没量过」；随机历史建判表（辩方第 3 条）并进合并实现前的事实核 |

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| L1-1、K3-1 改法（G8 按项取版本） | 攻方：按项取断前缀共享（多判 1 426 456 802 态），按节点取并就改一项重判一片；辩方 2.1 同 | **打中，两腿独立给出同一改法**：判法版本按节点算，取设这个点的文件出发的闭包 ∪ 判器闭包 |
| L1-2、K3-1 改法（G4、G8 把只在 CPU 上跑的代码放进 GPU 结论的键） | 攻方：改观察者 GPU 白重判 1.30–2.40 小时；辩方 2.2：引擎整份进版本过敏、违背用户 2026-09-29「派活和判法拆出来最好」与契约测试 | **打中**：一块存三样、各一把版本——GPU 结论（块键，GPU 判器版本）、CPU 结论（块键，CPU 判器版本）、观察者结果（块键，观察者版本）；引擎不整份进版本，改结论的几处（红项编码 `crash_amplification.rs:1162`、恢复字 `:1788`、GPU 结论换红项）搬进判器一侧的文件 |
| L1-3（单跑一项时节点上挂哪几个观察者） | 攻方 | **定义缺口**：观察者挂在哪个节点由登记表静态列出，单跑一项照表跑它经过的节点上登记的全部观察者 |
| L1-4、辩方 G7（逐条判到 / 不适用 GPU 给不出） | 两腿同 | **打中**：GPU 结论字加逐条「判到」位（甲，动内核，第三批）；第三批之前提交那一趟逐条计数报「本次未核」，违例、闭式、按发布分的态数照判 |
| L1-5、辩方 G9 第 2 条（等价剪枝不存在） | 两腿同 | **打中**：删掉「与已判等价的才剪」，这一版只按键与版本相同复用 |
| L1-6、辩方 K1-3（比的恢复字不全） | 两腿同 | **打中（小）**：G6 比两遍各自的落到的根、结局、失败点，共八个字；GPU 补不看 journal 那一遍的根两个字（第三批） |
| L1-7、辩方 G10（段级登记、u64 界） | 攻方：引擎只有点级 ignore；322 写那段 panic、跳过数装不进 u128；GPU 只拆低 32 位，33–64 写的段判错（今天最长 30 写，潜在）。辩方：u64 界偏松 | **打中**：内核按 lo / hi 算满 64 位余数，判表打包核段长、装不下就记 GPU 判不了（照 G6 判红）；引擎加段级「不展开、计入跳过」，跳过量按「段号 + 写数」报，不报十进制态数；登记的是整域按全量展开深度展不开的段 |
| L1-8、辩方 G9 第 3 条（GPU 与 CPU 不同趟、腿账） | 两腿同 | **定义缺口**：按（块键，腿，版本）记账；恢复字另表存（每态八个字），不进判定向量编号 |
| L2-1、L2-2、L2-3、L2-5（代价口径） | 攻方 | **打中**：第一轮拿一项的 GPU 速率乘全域；按 2026-09-30、10-08 量过的整项挂钟，整域 GPU 串跑约 9.8–10.8 小时、下界 4.9–7.2 小时；CPU 跟上（每态判 + 删除流逐态挂载）73–220 小时；E171 冻结前要这么长时间判法版本不变。都是推的，今天的树没量 |
| L2-6（删除流共享 13.6 亿态） | 攻方 | **打中，主 agent 两条路坐实**（第一节第 ① 格）：今天两项只共享取号暖机与 A，16 777 259 态；去重 37.9 亿 → 51.3 亿；第一轮判决 K1-2 用的 92.8% 在今天代码上不成立。K1-2 的结论（挂载观察者挂在标准前缀上）不受影响，它只在 B、C 进公共模块之后才有共享节点，交用户（第三节） |
| L2-4（提交只等 GPU 时整域上判的是 WGSL 那份恢复与 checker） | 攻方提醒 | 交用户（第三节）：只改 Rust 恢复或 Rust checker 的提交，整域上只在 CPU 那一截被测 |
| L3-1（两套「哪棵树、什么时候算版本」） | 攻方 | **打中**：版本在编译时算进二进制（`build.rs` 照同一套闭包算），跑时从盘上再算一遍，不等就整趟不写库、退 78；提交那一趟整域照门禁 `--full` 在 HEAD + 暂存区的 worktree 里跑 |
| L3-2（合表后 placement 认不出） | 攻方 | **打中**：`crash-case-check.py` 的 placement 改认合并后的项，与入口同批 |
| L3-3（模块表、没卡的语义） | 攻方 | **打中（小）**：modules 表与 19 份测试文件首行随改名同批；配置开 GPU 时没卡整道退 78，配置显式 `ENABLE_GPU=0` 时挂特性的那几格报「本次未跑」、整域不认这一趟 |
| L3-4（版本不含 GPU 驱动） | 攻方 | **打中（小）**：GPU 判器版本带驱动版本与卡型 |
| L3-5 | 攻方 | 没打中 |
| L6-1（G10 在不碰 crates 的第一批） | 攻方 | **打中**：G10 挪进第二批，第一批只改登记表形态 |
| L6-2（甲二上不了 GPU） | 攻方 | **打中**：判表加一列段的展开方式、两个内核照它拆（第三批）；第三批之前快档甲二那一半只 CPU 判，登记成豁免行 |
| L6-3、辩方 G5（随机历史两项上 GPU） | 攻方：没有批次管 `crash_injection.rs`；辩方：随机历史不是一种展开、判法也不是 `Layer0StateJudgement` | **打中**：加「给定状态表」一种展开，随机历史的池级 checker 与两遍恢复那一截上 GPU；对理想模型的比较与记录核对器照 G5 归 CPU、登记豁免行；`crash_injection.rs` 的录写与建判表进第二批，判表建得出与否进实现前事实核 |
| L6-4、辩方 G1（改名射程） | 两腿同 | **打中**：第四批文件清单写成 crates 标识符 + 脚本里认它们的地方（`capped.sh` 的 `SINGLEFS_LAYER0_THREADS`、`admission.py`、`stage-inputs.tsv`、门禁样本、`lib_heavy_tests.py`、`crash-case-check.py`），环境变量名、库目录默认路径同一提交改；仓外私有配置由主 agent 同时改 |
| L6-5、辩方 L6「条款先于做法」 | 两腿同 | **打中**：G12 的新条款在第四批合入后的第一次提交起生效；第一至第四批的提交照今天的条款（快档全绿，全量用户要求时跑），整域在第四批之后只跑一次 |
| 辩方 L6「撞在飞的 T3」 | 辩方 | **打中（小）**：第一批等 T3a、T3b 合入再派 |
| 辩方 K6-3（层表「层 0 冒烟」） | 辩方 | 采：层表三层的名字随 G1 换 |
| 辩方 K7-1 | 辩方 | 采：第一轮那一格过期；今天挡合并实现的是删除线第 56 批与这一轮的快照 |
| 辩方 G9 第 1 条（提交不等扩到要真跑实现的观察者） | 辩方 | 交用户（第三节） |
| 辩方 G11（前提大半成立、默认开 GPU 与今天相反） | 辩方 | 采事实：两个入口今天读同一份 `multi-host.env`；要合的是两套双机驱动与卡配置；照用户定的「默认开 GPU」，判法改成没写算开，私有配置主 agent 改 |

两腿各自提的改法（上表「改法」都是）都只在它们的模型或读码上成立，被攻过零轮。L1-1 与 L1-2 的改法两腿独立给出同一种，仍按零轮算。

## 三、交用户

| 问 | 事实（出处） |
|---|---|
| 逐状态可写挂载这类要真跑 Rust 实现的观察者，提交等不等 | 用户 2026-10-10 定的是「CPU 对拍」提交不等（第一轮判决第三节第 48 行）；G9 把挂载观察者一起放到日常（辩方第 4 节）；删除流整条逐态挂载约 50.8 小时（攻方模型 `cpu_unlink_mounts_alone_hours=50.8`，挂载速率出处不明，推的） |
| B、C 两步挪不挪进公共模块让两项共享 | 今天不共享，去重 5 133 787 951 态，整域 GPU 串跑约 10.8 小时、CPU 跟上 80–220 小时；挪进去 3 791 610 606 态，约 9.8 小时、73–175 小时（攻方模型 `MODEL sharing=code_today` 与 `sharing=records_assumption` 两行，速率是 2026-09-30、10-08 的树上量的，今天没量，推的）；挪进去之后删除流的挂载观察者挂在共享节点上 |
| 附：提交只等 GPU 时 Rust 恢复的改动整域上只经 CPU | 攻方 L2-4，读码：GPU 判器是 WGSL 另写的一份（`crash_judge_gpu.rs:3`） |

## 四、第三轮

收严后的形态写进第三轮正文，等第三节两问定了再派。第三轮是最终轮：只攻前两轮站住的形态；这一轮新冒出来的零轮形态（第二节各格的改法）照 `.claude/rules/three-way-inference.md`「多轮」不再为它们另开一轮，第三轮判决的交用户表里标「零轮」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 备料 | 2026-10-10 不受影响：两腿没动枚举域的定义 |
| D13（验证路线） 已定项 9 | 备料 | 2026-10-10 不受影响：改写等第三轮判完、第四批合入时做，生效时点按第二节 L6-5 那一格 |
| D13（验证路线） 已定项 15 | 备料 | 2026-10-10 不受影响：checker 档归属不变 |
