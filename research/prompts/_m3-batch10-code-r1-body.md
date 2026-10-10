# 根环实现第 10 批：代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 K7 RGall Q3 A2 F10 F11 C378 -->

## 一、题面

第 10 批到第 10f 批的补丁都合进了 `crates/`，各条交回与主 agent 的判定在 `records/2026-09-30-里程碑三开工第一轮.md` 第 10 批各条。这一轮是这一串改动的代码轮第一轮，攻的是合入之后的代码与它的测试。根环实现的前三轮代码轮判决：`research/prompts/m3-root-ring-code-r1-main-verification.md`、`-r2-`、`-r3-`。回收门槛那一轮判决：`research/prompts/m3-reclaim-window-r1-main-verification.md`。

改动全文在 `research/prompts/_m3-batch10-code-r1-diff.md`（每份补丁一节，各带实现员报告的路径），冻结副本 `/tmp/claude-1000/m3-batch10-code-r1/tree/`：

| # | 改动 | 主要落点 | 压着的条款或定案 |
|---|---|---|---|
| A | RGall：回收门槛与同取窗口的另外五处一起退回「环里最旧有效根」；A2（低于现行 F 的根跳过它的账）；C（重读读失败或读回不是那个头，这一轮都不删）；Q3（准入抬 F 的上限并进读不出与自证不过两类槽） | `crates/singlefs-core/src/allocator.rs`、`mount.rs`、`instance_table_sweep.rs`，`crates/singlefs-checker/src/walk.rs`，参考模型 `crates/singlefs-harness/src/model.rs`、`model_comparison.rs` | `.claude/kb/decisions/16-发布语义.md` 已定项 1「可再分配」「环里最旧有效根」「抬 F 的上限」「根槽这一次读坏」四行；`.claude/kb/decisions/18-块里携带什么信息.md` 已定项 11 |
| B | 读坏的槽钉在可能住过的最旧 txg（三处回收门槛），重读读出有效根或被抛弃的根才解开；全零的槽也钉住（`RootRingOccupant::AllZeroAtMount`），只是不并进准入抬 F 的上限 | `allocator.rs`、`mount.rs` | 用户 2026-10-04、2026-10-05 三条定案（条文还没写回 kb，原话在调度记录第 10b 批 core 修订交回、第 10c 批交回两条） |
| C | I-8.5 在有槽自证过而一块都不可择时报不适用，一个槽都没自证过的照旧判红；I-7.9 两支只留锚在镜像最新根上的窗口 | `walk.rs` | `.claude/kb/invariants.md` 的 I-8.5、I-7.9 两行；`InvariantVerdict` 文档「不适用必须带理由，不许静默算成立」 |
| D | 重建分配器那一遍一条不被抛弃的自证有效根都没读出，就整环重读一次，重读之前调 `BeforeTheOneReread` 钩子；重读还没有就在取号之前返回 `Recovery(NoValidRoot)` | `mount.rs` | 用户 2026-10-05 定的甲二改法：`research/prompts/m3-batch17-code-r1-forks.md` 第 2 行 |
| E | checker 档：历史生成加一步「第一个 journal 记录写到达时整池掉电再可写挂载」（留下孤儿）；四样工具搬进 `writable_mount_phases`；记录核对器按掉电接缝分发布（`RecordStreamContinuity::BrokenAtSeams`） | `crates/singlefs-harness/src/history.rs`、`memory_pool.rs`，`crates/singlefs-checker-tier/src/crash_injection.rs`、`crash.rs`、`writable_mount_phases.rs` | `.claude/rules/verification.md`「崩溃一致性只能靠崩溃点重放验证」；调查报告 `/tmp/claude-1000/m3-r1/batch10b-investigate-claimed-missing/report.md` |
| F | GPU 判器：每表树数上限 8 → 11、种类 9–11 的走读与映射条目数、两个内核的 I-9.16 ② 按 11 种、I-7.9 只留一个窗口 | `crates/singlefs-checker-tier/src/crash_judge_*.wgsl`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs` | 同 C；GPU 与 CPU 判器逐状态对拍 |
| G | 变异表：第 927 行判等价、留档用例与替换的两行；第 10d、10e、10f 批改锚点的十几行 | `crates/mutations.tsv`、`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` | `.claude/rules/mutation-sampling.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本取自 2026-10-05 主工作区：第 10 批到第 10f 批都已打上。删除与并发那条线第 17、18 批，形式化接入那条线第 26、27-2b、28、29 批也已合入。被判文件的 sha256 在 `research/prompts/m3-batch10-code-r1-snapshot/crates-sha256.txt`。
- 合入后验证（harness 档全部测试二进制、fmt、clippy）每一份都绿：`/tmp/claude-1000/merge-and-verify/` 下 `2026-10-05-3808358`（10b 四份）、`2026-10-05-413718`（10c 加删除与并发那条线第 18 批，那条线跑的）、`2026-10-05-510982`（10d）、`2026-10-05-1067524`（10e 与第 927 行）。
- checker 档提前跑过（`SINGLEFS_HEAVY_TESTS=user-request`，调度记录「第 10d 批合入后 checker 档」那一条）：lib 116 过；崩溃注入与记录核对器 8 份集成测试全绿；GPU 对拍 26 条在新开进程里全过（同一进程跑到第十几条之后 Vulkan 枚举不出 NVIDIA 卡，10b 之前就有，推的是没清场的环境问题）。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中，判决照原去向写）：E139 第 2 次重跑量出的 RGall 代价（disk_stuck 与 S = 4 坏槽上扣住的单元没有上界，F10、F11，`.claude/kb/experiments/` 里 E139 页）；钩子不区分实例表与根环两处调用；崩溃注入加换恢复策略的开关；`crash_injection.rs` 第一截上方注释过时。
- 用户已认的代价（腿报出来不算打中，只核「代价是不是只有这么大」）：钉最旧 txg 让 I-3.1 在合法盘面上红在记账多的一边（读故障下一直读不出时最长一圈；没有读故障时崩在记录之后、根槽之前、环没转过来的全零槽红一版）；读坏成全零的槽 F 可能越过住在里面的有效根（Q3 的代价，没洞那一份 F 抬到 18）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　回收门槛（A、B）
六处一起退回「环里最旧有效根」之后，还有没有一处仍按一圈窗口取（allocator 回收、抬 F 那一遍、checker 候选过滤、I-3.1 丁-defer 门槛、影子账候选、回退候选集、抬 F 上限）。钉在可能住过的最旧 txg 那三处（挂载时含 m 取不到那一支、进程内、抬 F 那一遍）：有没有一段合法历史在门槛越过一条盘上真住着、可回退到的有效根时回收了它引用的单元（不安全方向）。全零槽另算：上限不并、回收门槛钉住，两边用的是不是同一份读数与同一个分类（挂载那一刻读到的）。用户认下的 I-3.1 多记账：是不是只红在记账多的一边、时长是不是只到那一槽被写掉为止；有没有一形红在记账少的一边。

### K2　重读再拒（D）与 10c 之后改成被拒的那两段
重读只坏一部分槽、读出的根与择根那一遍不一致时会怎样（实现员读代码推的方向保守，造了一条「重读只坏盘 1」）。`Recovery(NoValidRoot)` 被 `crates/singlefs-harness/src/mount_transparency.rs` 里那一处排在 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 那张单子之外：连坏两遍那一形的写前拒，挂载透明该不该判成违例、条款怎么写。删除与并发那条线的攻方装置在 10c 之后的树上，`192:9:3:1` n = 8 的 `w:but1/all:z` 与 `w:all/but1:z` 两段改成再挂在任何写之前被拒（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`，10c 之前坏 8195）：为什么拒、该不该拒。

### K3　I-7.9 与 I-8.5（C、F 的 I-7.9 一半）
I-7.9 两支只留锚在镜像最新根上的窗口：有没有一张坏镜像（F 越过上限）因此漏红。删除与并发那条线的攻方装置在 10c 之前的树上，`192:9:3:1` n = 8 `w:all/but1:z` 两盘都在时池级 checker 判 I-7.9 红 4116 个状态（它自己的判法 0 坏）：是实现越了上限，还是 checker 判错（拷贝 `/tmp/claude-1000/-home-fy5090-code-singlefs/e594a7a0-d003-4a02-9455-a9dc350ed330/scratchpad/m3b17-today/tree`，攻方模型 `research/prompts/m3-batch17-code-r1-opus-model/` 的 `rerun.sh` 的 l0-5）。I-8.5 改报不适用的那一格，边界对不对。

### K4　checker 档装置（E）
掉电步的接缝只在落在崩溃段第一个写之前时才交给记录核对器：接缝不在段首的那种状态会不会照旧假红，随机历史里有没有这种状态。孤儿历史、`writable_mount_phases` 搬家之后，三段（写行、暖机、抹头）每段都摆得出二次崩溃吗。记录核对器的判据（`claimed_state_missing_unit` 那一条）在 `.claude/kb/invariants.md` 里零命中：它压着哪条条款。

### K5　GPU 与 CPU 同口径（F）
树数上限 11 的守门用例能不能被一条写得更长的树表绕过；种类 9–11 的走读、映射条目数、I-9.16 ② 的 11 种与 CPU（`walk.rs` 的 `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`、`recovery.rs` 同名表）逐要素对得上吗；种类 9、10 与容器树长高没有一条流走得到，对拍等于没测这几支。

### K6　数能不能核
I-3.1 多记账的 `Some(3145728)` 与 `1441792`、前洞那条只红 txg 20 那一版、`k2-zeros` 那一形 F 不越过 21（有洞）与抬到 18（没洞）、草稿区 `SCRATCH_WORDS` 与展开大小、第 927 行「等价」的论证：每一个从哪一段代码推出来、钉它的测试与变异行改回旧值时红不红；有没有一个数只在实现员的临时探针上核过。

### K7　欠账与读法清单全不全
第 10 批到第 10f 批、第 927 行、锚点那几份报告里「我取了读法的」「做不出会红用例的」「要主 agent 判的」每一条，在调度记录、`.claude/kb/checks-owed.md` 或判决里都有去向吗；有没有一条被无声丢了。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K5、K6、K7 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行 |
| 云端攻方（Opus） | K1、K2、K3、K4，重点 K1 与 K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；不重复前几轮判决第二节已攻过的角度；用户认下的代价只攻「是不是只有这么大」 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；腿提的收严记「零轮」，交用户或登记实验。
- 第二节列为已知没做、用户已认的几样，腿报出来不算打中，判决照原去向写。

交付：腿的报告写 `research/prompts/m3-batch10-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-batch10-code-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的行号。方案按 `crates/` 今天的实现来谈。
