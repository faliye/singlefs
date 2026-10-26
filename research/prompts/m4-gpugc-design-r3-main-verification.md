# m4-gpugc-design-r3：里程碑四七·2 GPU 垃圾回收设计轮，第三轮判决（2026-10-26）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 M1 M2 M3 M4 M5 M6 M7 M8 M9 -->

被判的是第一轮正文 `research/prompts/_m4-gpugc-design-r1-body.md` 的 G1–G7 与第二轮正文新加的 G8，这一轮的正文 `research/prompts/_m4-gpugc-design-r3-body.md`。腿的报告：
- 辩方（Sonnet）：`research/prompts/m4-gpugc-design-r3-sonnet-output.md`（770 行，sha256 前缀 `eeef1324`），复核第二轮判决 `research/prompts/m4-gpugc-design-r2-main-verification.md`；
- 攻方（Opus）：`research/prompts/m4-gpugc-design-r3-opus-output.md`（510 行，sha256 前缀 `46d27a99`），模型目录 `research/prompts/m4-gpugc-design-r3-opus-model/`。

两条腿都是续做：同立场的前一条腿各写到 200、249 行时中途停下没交回，新派的腿读交接摘要接着写完（调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 37、38 批）。

这是第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。这一轮之后不再开轮；这一轮新冒出来的改法算零轮，进第四节交用户表。

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 开工快照 | `sha256sum -c research/prompts/m4-gpugc-design-r3-snapshot/sha256sums.txt`、`crates-sha256.txt`（事后补记本批合并后的被判文件哈希）与 `kb-sha256.txt` | 23 份里三份对不上：`crates/singlefs-core/src/compaction_plan.rs`、`transaction.rs`、`crates/singlefs-checker-tier/Cargo.toml`；kb 两份对不上：`.claude/kb/checks-owed.md`、`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md`。都是别的会话在腿跑着时改的、没提交（三份代码 HEAD、快照、此刻的 sha256 前缀各不相同，例 `compaction_plan.rs` HEAD `deebf340`、快照 `826d7d3d`、此刻 `16f08bba`），倒推不出快照那一刻的原样。`checks-owed.md` 恢复验证审计那条线报了改法（插 C898、C907 两行）。两条腿都写明引 `crates/` 行号取此刻工作区，所以引文以主工作区为根核；这几份里腿引到的行主 agent 逐处人工读（下一行） |
| 两条腿的引文 | `python3 scripts/agents/cite-check.py <两份报告> --root . --background research/prompts/_m4-gpugc-design-r3-background.md` | 「✓ 核了 106 处引文、41 处裸行号，对上 106 处，没判 41 处（其中裸行号没有引文可比 41 处；逐处列在上面）」 |
| 判决要用的代码与门禁行（主 agent 逐行读主工作区） | 读 | `crates/singlefs-core/src/compaction_plan.rs` 第 855–863 行：`publish_budget_of_a_compaction_run` 收尾 `.max(1)`（第二轮判决报的第 815 行整体后移，内容不变）。`.claude/gate.d/checker-independence-and-sync.sh` 第 24 行：checker-implementation-disjoint 格 ① 「singlefs-checker 与 singlefs-core 各取传递闭包求交」，只判这两个包——checker 档 GPU 判器与以后的产品 GPU 包共用代码时这一格不红（攻方 G7 坐实） |
| 产物现数（主 agent） | 读 `research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out` 与 `research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz` | ① `merging_shape` 648 行：`one_submission_per_batch` 510、`one_submission_per_intent` 80、`unstable` 56、`fit_not_linear` 2，跨意图合并 0 行（辩方坐实）。② 候选 B 与 C 的每次提交主机 CPU 1040 组逐字相同（辩方坐实）。③ 按间隔拆开的不赔格数（受控臂 c_rt、门槛照第 4 次重跑产物直接算、机器比例、线程钟，`baseline_rejudge_point` 行）见第二节表 |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-gpugc-design-r3-main-check/model/`，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <那份>/tools/rerun.sh --repo … --frozen … --work …` | 退 0，5 行 `name=rerun_compare model=… identical=true`（crt_shift、round_spread、rejudge_arms、numbers_check、hanging_checks）；判决第二节表是主 agent 从入库产物另外现数的 |
| 正文里主 agent 自己的错 | 两条腿点出 | ① 正文第四节 G1 把「全部显卡」写成 D26（后台整理与放置回收） 已定项 13 第 4 条，它在第 2 条。② 正文第三节「C 卡 0 仍多半不赔」把间隔 0 与 50 ms 两种读数并着数，单看间隔 50 ms 在两处都换的写法下 C 一格不赔都没有（第二节表）。③ 第三节「C 卡 0 120 格」里有 12 格是今天的写法只报的那几格。三处都是主 agent 写正文时数错，以第二节表为准 |

## 二、判定与去向

**决定 GPU 模式按 D26 已定项 13 第 7 条（省下的主机 CPU）赚还是赔的读数**（主 agent 现数，`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`）：

| CPU 写法 | 候选 | 卡 | 间隔 0 不赔 | 间隔 50 ms 不赔 |
|---|---|---|---|---|
| 今天（软件 CRC、线性找） | C（甲乙丙全放） | 卡 0（5090） | 84/84 | 29/36 |
| 今天 | C | 卡 1（5060 Ti） | 59/84 | 0/36 |
| 今天 | A（只放扫描校验和）、D（只放释放前核） | 卡 0 | 44/72、50/72 | 0/36、0/36 |
| 今天 | A、D | 卡 1 | 0/72 | 0/36 |
| 三路硬件 CRC 加按身份查（不计建索引） | C | 卡 0 | 64/84 | 0/36 |
| 同上 | C | 卡 1 | 10/84 | 0/36 |
| 同上 | A、D | 两张卡 | 0 | 0 |

| 格 | 前两轮 | 第三轮 | 最终判定 |
|---|---|---|---|
| G1-① GPU 上算哪几段 | 两轮打中 | 两条腿都判「随提交间隔翻面」 | **打穿，结论改写**：CPU 模式换成快写法之后，只有候选 C 在卡 0、提交连着发（间隔 0）时不赔；间隔 50 ms 下任何候选在任何卡上都赔；A、D 在快写法下两张卡全赔。用户前提「需要加入，不需要讨论」不变，交用户的是放哪几段、CPU 侧换不换快写法、什么间隔下用 GPU |
| G1-③ 今天的形态进前台路径 | 两轮打中 | 两条腿都判站得住（每次提交挂钟约 5.3 ms，代码没变；范围：产品路径今天恒不起跑） | **打穿，维持** |
| G2 攒批形态 | 两轮打中 | 两条腿都判「跨批攒不成」站得住（648 行跨意图 0 行）；卡 0 拟合不可信站得住、理由换了 | **打穿，维持**：一批一交或一条意图一交，跨意图攒不做 |
| G3 多卡分活 | 第二轮打中 | 两条腿都判打中：轮转让 C 在卡 0 不赔的格变赔或不稳定，同样打中一切用到卡 1 的分法 | **打穿（第二、三轮）**：「全部卡等权轮转」出局；怎么分交用户（第四节） |
| G4 ① 输入表准备那一类对拍抓不到 | 第一轮打中、第二轮辩方站得住 | 两条腿站得住 | **打穿，维持** |
| G4 ② CPU 只改位图与逐条核的关系 | 第一轮打中 | 两条腿都判打中，病在条款字面（四个候选一起中，分不出臂） | **打穿（第一、三轮）**：条款要改写，交用户 |
| G4 的 N | 第一轮挂着 | 两条腿都判构造不出 N 的反例；第二段零次不一致只罩同一单元重复的输入 | **没打中**：N 取几交用户，计数住哪（内存会被重挂清零）仍是第一轮那一处脆弱点 |
| G5 控制接口 | 第二轮打中 | 两条腿都判只在 G8 选「挪出」时打中 | **打穿，带条件**：挪出调用线程时要定「停着时收到切换请求」怎么办（第四节） |
| G6 三个默认值 | 两轮打中 | 站得住（只对「fsync 在等」那半句）；改法「挪出」够不着「等多久」那一格（一批主机 CPU 12.2–13.9 ms / 96.4–112.7 ms，攻方量的） | **打穿，维持**；默认值交用户 |
| G7 GPU 代码住哪 | 第二轮打中（推的） | 攻方打中、换了形态（门禁 ① 只判两个包，第一节第三行主 agent 坐实）；辩方判够不着第二轮那一形 | **打中，形态改写**：第二轮「只能共用同一份 naga」那一形不成立；成立的是产品 GPU 包与 checker 档 GPU 判器共用代码时现有门禁不红，要补一格 |
| G8 GC 挪不挪出调用线程 | 第二轮第一次判 | 两条腿都判两边都有代价、交用户；不挪出另撞 D24（后台重活能不能卸给 GPU） 已定项 2 定案句；发布预算与跨趟那两样不是挪出特有的（凡起跑不经 `publish_user_change` 的都中） | **交用户** |

条款与代码的冲突 C906（整理批不过准入与 D26 已定项 3 对不上）两条腿都列进交用户表（辩方补 9），去向照第二轮判决：交用户。

## 三、数能不能核

两条腿逐个现数过两轮判决与这一轮正文的数（辩方报告第四节、攻方第二节）。主 agent 认的更正：
- 第一轮判决第一节「g6 真盘臂一批 `wall_ms_median=23.728`–`123.369`」下端应为 21.671（攻方现数）。
- 第二轮判决第二节 G8「占 CPU 模式一批挂钟 4.5%–118%」的分子取自第 3 次重跑、分母取自第一轮攻方模型，是两次跑拼的比，引的时候要连这一句写。
- 第二轮判决第一节引文「对上 76 处」的行号，开工快照之后有三份代码被改，7 处行号今天对不上；内容没变。
- 第三轮正文三处（第一节最后一行）。

照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」，前两轮判决原文不改，更正以这一节为准。

## 四、交用户

三轮之后的岔路，合成一张交用户表写进岔路单 `research/prompts/m4-gpugc-design-r1-forks.md`（第 12 行起，两条腿补的行按内容合并：辩方补 1–补 10、攻方 M1–M9）。这一轮新冒出的改法都标「零轮」。

最要紧、挡着七·3 的四问，主 agent 弹窗交用户：
1. GPU 上算哪几段、什么时候用（G1、补 3）。
2. CPU 侧换不换快写法：区域扫描按身份查（岔路单第 10 行）、CRC 换硬件指令（补 1、补 2）。
3. GC 挪不挪出用户改动的调用线程（G8、岔路单第 11 行）。
4. C906 整理批过不过常规准入（补 9）。

其余（多卡怎么分、N 与计数住哪、控制接口停着时的切换、默认值、产品 GPU 包与 checker 档的共用边界与门禁那一格、条款第 8 条与逐条核的改写、钟的取法、带快照的池）在岔路单里，等这四问定了再逐条交。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-10-26 不受影响：三轮打穿的都在七·2 的设计候选与第 8 条的字面上，条款正文等用户定第四节四问之后由写回员一批改 |
| D26（后台整理与放置回收） 已定项 3 | 备料 | 2026-10-26 不受影响：C906 交用户 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-26 不受影响：第 2 条是 G1-③、G8 的依据；「几十微秒」与实测差约 100 倍（攻方），条款本身改不改随 G8 一起交用户 |
| D13（验证路线） | 备料 | 2026-10-26 不受影响：G7 要补的门禁格是 checker-implementation-disjoint 的射程，条款不改 |

## 这一轮被判过的文件

- `.claude/agents/experiment-runner.md`：实验执行员定义与入库装置运行约束汇总判读
- `.claude/agents/mutation-triage.md`：变异分拣定义与编译目录隔离约束汇总判读
- `crates/singlefs-background-gpu/src/adapter_inventory.rs`：产品 GPU 包适配器枚举与独显过滤边界
- `crates/singlefs-background-gpu/src/background_gpu.rs`：产品 GPU 包计算通道与显存留账边界
- `crates/singlefs-background-gpu/src/background_gpu_compute.rs`：产品 GPU 包实现核心后台计算特征与设置切换边界
- `crates/singlefs-background-gpu/src/checksum_kernels.rs`：甲段两份独立校验和核的派发与比对
- `crates/singlefs-background-gpu/src/gpus_of_this_host.rs`：挂载时开独显、量代价与跳过非点名坏卡边界
- `crates/singlefs-background-gpu/src/lib.rs`：产品 GPU 包对外入口与模块导出边界
- `crates/singlefs-background-gpu/src/live_slot_kernels.rs`：丙段两份独立活槽计数核的派发与比对
- `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs`：显存上限切块与超时提交留账计算
- `crates/singlefs-background-gpu/src/region_record_kernels.rs`：乙段两份独立区域记录分类核的派发与比对
- `crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs`：两份独立 GPU 核结果比对与连续不确定计数
- `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs`：等待 GPU 提交完成的超时控制
- `crates/singlefs-background-gpu/src/work_split_by_measured_cost.rs`：按挂载时实测代价切分多卡工作量
- `crates/singlefs-core/src/admission.rs`：整理批走常规准入与发布预算边界
- `crates/singlefs-core/src/allocation_record_tree.rs`：分配记录树按身份查与结算路径
- `crates/singlefs-core/src/allocator.rs`：分配器空洞与保留区边界
- `crates/singlefs-core/src/background_compute.rs`：后台计算设置校验、切换生效与错误上浮类型
- `crates/singlefs-core/src/block_device.rs`：块设备接口与安全属性说明
- `crates/singlefs-core/src/checksum.rs`：三路硬件指令 CRC32C 与软件回退实现
- `crates/singlefs-core/src/code_two_tree.rs`：码 2 树表键区间与子树覆盖边界
- `crates/singlefs-core/src/compaction_plan.rs`：整理计划、发布预算、活槽统计拆分与 GPU 特征接入
- `crates/singlefs-core/src/extent_tree.rs`：上段叶标签 0 与多文件 extent 树边界
- `crates/singlefs-core/src/journal.rs`：日志环写入与恢复重放边界
- `crates/singlefs-core/src/lib.rs`：核心包导出与模块边界
- `crates/singlefs-core/src/make_filesystem.rs`：建池根环槽与系统配置写入边界
- `crates/singlefs-core/src/mount.rs`：挂载入口收后台计算句柄、会话内回滚清意图与取号两轮边界
- `crates/singlefs-core/src/mounted_read.rs`：挂载态读路径对多文件与标签 0 的判定
- `crates/singlefs-core/src/mounted_session.rs`：后台整理挪出调用线程、每次用户改动清已完成意图与每版建一次索引
- `crates/singlefs-core/src/records.rs`：记录编解码与隔离记号边界
- `crates/singlefs-core/src/recovery.rs`：恢复路径按身份查找与落点公式核对
- `crates/singlefs-core/src/region_scan.rs`：区域扫描按身份索引查找与分类实现
- `crates/singlefs-core/src/root_record.rs`：根记录回收门槛字段编解码
- `crates/singlefs-core/src/root_ring.rs`：根环落点公式核对与最旧有效根计算
- `crates/singlefs-core/src/system_configuration.rs`：系统配置槽距与根环区长度校验
- `crates/singlefs-core/src/transaction.rs`：事务发布按身份查找、多 inode 变长截断与整理批准入
- `crates/singlefs-core/src/truncate.rs`：任意活 inode 截断计划与下段节点替换
- `crates/singlefs-core/src/unit.rs`：单元头解析与校验和核对辅助接口
- `crates/singlefs-core/src/write_request_split.rs`：通用写请求按单元边界切分
- `crates/singlefs-format/src/lib.rs`：盘格式常数、根记录宽度与系统配置判据
- `crates/singlefs-reference-filesystem/src/lib.rs`：参考文件系统语义对照边界
- `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`：整理计划前缀重放形式化证明
- `crates/singlefs-replay-prefix/src/gc_crash_proofs.rs`：GC 崩溃重放形式化证明
- `crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs`：GC 根环区间形式化证明
- `crates/singlefs-replay-prefix/src/lib.rs`：形式化证明包入口与导出边界
