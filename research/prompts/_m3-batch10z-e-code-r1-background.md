# 根环实现第 10z 批第五块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 -->

## 一、题面

这一轮攻第 10z 批第五块（显存账与第 ① 段派活）。它照第 10z 批第三块代码轮判决 `research/prompts/m3-batch10z-c-code-r1-main-verification.md` 第三节那三条改。

GPU 用在崩溃放量与对拍上的总口径在 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

用户原话：「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」；2026-10-07 定 G1 二（`research/prompts/m3-batch10t-code-r1-forks.md`）；2026-10-08 定 G2「先回甲、同时量乙」（`research/prompts/m3-batch10z-a-code-r1-forks.md`，乙由 E174（借据丢掉时等待与显存回还） 在量，不在这一轮）。

改动全文：`research/prompts/_m3-batch10z-e-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-e/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | K1-5：叫停时卡只领判完的块在等的那几块（被交回的块），判完的块照块序全部入库、没判的留给下一趟；轮间状态抽成 harness 档 `RoundsOfTheCards`，原第 3081 行两条断言挪进去；`cards_used` 按卡序号去重 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |
| B | K1-4 (a)：设备丢失回调把 wgpu 给的那一句与 `wgpu::hal::DeviceError::OutOfMemory` 比对，相同即记显存不够；(b)：活交回那一刻记设备丢没丢，那一刻设备还在的不因之后同块别段的 Out of Memory 说成显存不够 | `gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` |
| C | K3：`notify_one` 挂住（60 秒超时）、范围里接到的先记再报到、先问叫停再领块三条，正推 `killers.rs` 五条入库；外层循环的报过卡过滤、下一轮只判没领走的、`cards_used` 抽出来测；第 D 条接线加一条 checker 档用例；变异追加 24、整行替换 5 | 三份 src、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/`，取在第四、五块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-e-code-r1-snapshot/crates-sha256.txt`。
- 合后验证（CPU 部分）：见调度记录第 843 行（与第四块同一趟）；clippy 带 gpu / verdict-store 特性 `--all-targets` 退 0、三种特性 `--no-run` 退 0。碰卡的 checker 档 lib 与 GPU 测试排在 E174 之后。
- 实现员证红：两次 prove-red `✓ 点名 8 条…都抓到了`、`✓ 点名 10 条…都抓到了`。
- 实现员自定、被攻过零轮的四处：
  1. 「判完的块在等的」按「比判完的最大块号小、还在队里」来判；
  2. 叫停、这一轮有卡报了显存不够、判完的块都已入库时，结局由「本次未跑」改成「照叫停收尾」（推翻上一块实现员钉的那一行）；
  3. 丢失缘由靠比对 wgpu 那一句的原文；
  4. 交回那一刻的记法留下两段窄窗口（读码推的）。
- 已知没做、不在这一轮被判范围的：G2 甲认的少给；设备丢了、账没被问到那一下读数多给（借之前必经重起判）；真卡上的 Out of Memory 与丢设备造不出。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态，一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　判决第三节三条做到没有
1. K1-5：叫停落在任一轮任一时点，判完的块是不是都入库、不判红、不判两遍、不漏判？「在等的」那条判法有没有一形把不在等的块当在等、或反过来？`cards_used` 对不对？
2. K1-4 (a)：比对 wgpu 原文——原文换了措辞、或别的错误恰好同文时会怎样？(b)：交回那一刻的两段窄窗口里，真 panic 会不会仍被说成显存不够，真的显存不够会不会判红？
3. 实现员第 2 处推翻上一块的结局，「照叫停收尾」时没判的块是不是都留给下一趟、库里没有缺块？

### K2　只防 OOM、不压并发两头（新代码引进来的）
1. 有没有一形让账给的量大于此刻真能用的？
2. 除甲认的少给之外，有没有一形卡放得下却报本次未跑、判红、或整趟弃卡？

### K3　测试判别力
新加的用例与变异行，能不能拦住这一批每一处被改回去？有没有一形「改回去」照绿？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。

### 小节清单：`.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D24 后台重活能不能卸给 GPU —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D24 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：暂缓，不做 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：三条硬约束 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：判据取什么形式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：重开判据 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：合法性的来源 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E174 借据丢掉时等待与显存回还 —— 装置写好、没上卡跑 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E174 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 读数：不起卡的几份（模型卡数据，不是 G2-乙 的读数） | 抄 | 默认全抄（脚本生成） |
| ### 单测与变异 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-10-08：建页，装置写好、没上卡跑 | 不抄 | 历史节的子节 |

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:1-4`（整段抄，未转述）**

```markdown
## D24 后台重活能不能卸给 GPU —— 已定

D24（后台重活能不能卸给 GPU） 管一件事：三个后台重活（批量压缩、全盘 scrub、大规模纠删码重建）要不要卸给 GPU。不管的：CPU 侧算法怎么选在 D9（加密） 已定项 3；scrub 本身的形态在 D9（加密） 已定项 5；后台整理与回收的队列与预算在 D26（后台整理与放置回收）；三格划分本身是 `.claude/rules/fs-design.md` 的记账纪律，D24（后台重活能不能卸给 GPU） 只引用它。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:15-27`（整段抄，未转述）**

```markdown
#### 已定项 1：暂缓，不做

**定案**：**暂缓，不做**——三个候选场景（批量压缩、全盘 scrub、大规模纠删码重建）都不排期。**暂缓不等于否掉**：算力受限那一格（批量压缩）仍然开着，只是不排期。**GPU 这条线不再继续测**，D24（后台重活能不能卸给 GPU） 停在「方向存疑、两侧都没坐实」这个状态——GPU 侧还缺一个真的 AES-GCM 核函数（要 `nvcc`）。**不要把「GPU 赢」当成已定结论去支撑别的决策。**

**射程**：承重的只有「非必需」这一条——三个候选场景全在 `.claude/rules/fs-design.md` 第三格（后台、可续做、非决策路径），不做它不影响任何硬承诺，不像 D1（数据可移动性 / 反向索引） 的缩容、D3（空间分配） 的常驻整理那样是承诺过的能力。**实测那一格不在候选清单里**：判掉的是「纯扫描」，而三个候选的 CPU 侧都要在扫描之上再算，所以标题里那句「一格实测为负收益」要读成「一个与候选无关的格」，不是「候选被实测判负」——那一格的数与判法在已定项 3。

**依据**：

- 用户定案两次（这件事非必需；GPU 这条线不再继续测），原话在变更史。
- 无实验：这条暂缓承重的只有用户定案，而实测判掉的那一格是纯扫描、不在候选清单里，那一格记在已定项 3；候选三格里两格至今无数据（纠删码重建与批量压缩），没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:28-44`（整段抄，未转述）**

```markdown
#### 已定项 2：三条硬约束

**定案**：三条，不满足就不许做——

1. **绝不能成为必需。** 文件系统必须在没有 GPU 时完整工作、且语义完全相同。⇒ **GPU 是可选加速器，一个比特都不许进格式。**
2. **不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。
3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，抽样复核。

**射程**：第 1 条管的是格式，第 2 条管的是路径，第 3 条管的是验证，三条各自独立，满足不了任何一条就不许做。它们不定「值不值得做」，那是已定项 1 与已定项 3。第 3 条要的两份实现今天一份都没有（D24（后台重活能不能卸给 GPU） 暂缓，不排期）。

**依据**：

- 用户定案（GPU 是可选加速器），原话在变更史。
- 无实验：三条都是政策与验证纪律的直接推论，没有可量的量；第 2 条引的 PCIe 往返量级是公开常识，本工程没测过前台路径上的 GPU 往返。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:45-57`（整段抄，未转述）**

```markdown
#### 已定项 3：判据取什么形式

**定案**：判据是 `CPU 16 线程耗时 ÷ (传输 + 计算 + 传回) 耗时`，小于 1 就否掉那一格。代进去的 CPU 侧数必须是**多核饱和吞吐**，不是「单核 × 核数」：线性外推与实测在 AES-GCM 上差近 4 倍（外推约 61 GB/s，实测饱和 17.4 GB/s）。GPU 侧的两个地板是**单程 56.69 GB/s**（锁页 H2D，≥64 MiB）与**来回 28.5 GB/s**（H2D + D2H，回传全部数据的场景走这一行）；CPU 侧的上限是**内存带宽墙 64.91 GB/s**。⇒ **GPU 只在「CPU 那侧的实际吞吐掉到传输地板以下」时才有机会。**

**射程**：⚠️ **「瓶颈是不是 CPU」不等价于「这个任务有没有硬件指令」**——有 AES-NI 的加密照样在 17.4 GB/s 就饱和，瓶颈仍然在 CPU 侧，**有硬件指令 ≠ 快到 GPU 追不上**。⚠️ **「CPU 够快所以 GPU 无用」这个前提在本工程不成立，不许拿它支撑任何结论**：它来自单核乘核数的线性外推，而多核实测饱和在 17.4 GB/s，差近 4 倍且方向相反——17.4 低于 GPU 的两个地板。代进去之后四个候选格今天各是什么：scrub（校验和 / MAC）有 AES-NI 时 CPU 侧多核饱和 17.4 GB/s（8 线程，16/32 线程 16.3/16.5；单核 3656.13 MiB/s）⇒ 按判据这一格 GPU 赢，⚠️ **两侧都没坐实，不许当已定结论用**（GPU 侧缺真的 AES-GCM 核函数）；无 AES-NI 的机器上单核 244.74 MiB/s ⇒ 16 核约 4.1 GB/s，与高端 NVMe 阵列同量级，这一档 GPU 可能有用，多核未测、饱和值只会更低；EC 重建无数据（Reed-Solomon over GF(2⁸)，重建要读全部幸存盘并算 parity，计算量随盘数 × 容量线性，这一格计算确实重）；批量压缩无数据（压缩比与吞吐的取舍，GPU 压缩库存在）。这两个数（17.4 与 244.74）来自 E6（加密算法选型），不是 D24（后台重活能不能卸给 GPU） 自己量的。⚠️ **不能读成「GPU 无用」**：计算段那一段的推论只对**带宽受限**的核函数成立，**压缩是算力受限的**，它的计算段可能远不止那一成，那一格仍然开着且仍要等 `nvcc`。⚠️ **还有一层压在上面**：三个候选场景全是盘限的后台活，盘带宽比 64.91 与 28.5 都低得多 ⇒ 整件事若是盘限的，CPU 本来就在空转，卸给 GPU 省不下任何东西——**这一层是推理不是实测**（本工程没有阵列）。⚠️ **一个未验的逃生口**：GPUDirect Storage 让 NVMe 直接 DMA 进显存、绕开主存与 PCIe 那一跳，**它会推翻「GPU 吃数据 56.69 GB/s」这个传输地板**，没测过，不许当它不存在。要定 D24（后台重活能不能卸给 GPU），先要量 EC 重建与批量压缩这两格的 CPU 上界，以及「PCIe 传输 + 核函数 + 传回」这条链路相对纯 CPU 的净收益——**数据要过一遍 PCIe，那笔搬运成本可能吃掉全部计算收益**。**本机可测**：NVIDIA RTX 5090（32 GB）+ RTX 5060 Ti（16 GB），驱动 595.84；`nvcc` 没有装（`command -v nvcc` 零命中，`/usr/local/cuda*/bin/nvcc` 也没有），那是 G24.3（计算段占比） 的前置。

**依据**：

- E21（GPU 卸载的净收益）：三条路径的吞吐（CPU 吃数据 64.91 GB/s、GPU 单程 56.69 GB/s、GPU 来回 28.5 GB/s）把判据的两个阈值钉住——GPU 光把数据搞进去就已经比 CPU 读它慢 13%，而这还没开始算；计算段用带宽受限算子给上界，显存内 1041 GB/s 而端到端（含传输）只有 50.3 GB/s ⇒ 传输占 90.3%、计算段几乎免费，对纯扫描那一格净收益 0.78×，小于 1 ⇒ GPU 输。**决定 GPU 赢不赢的是 CPU 那侧有没有对应的硬件指令，不是 GPU 有多快。**
- 用户定案（判据的正确形式是多核饱和吞吐），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:58-81`（整段抄，未转述）**

```markdown
#### 已定项 4：重开判据

**定案**：说不出什么现象会让它重开，「暂缓」就是无限期搁置。四个闸——

<!-- doc-lint:registry name-col=2 -->

| 闸 | 简称 | 判什么 | 现在测得了吗 |
|---|---|---|---|
| G24.1 | 缺硬件指令 | 目标平台的 CPU **缺**对应硬件指令，使该任务的 CPU 侧吞吐低于 **50.3 GB/s**（单程场景）或 **28.5 GB/s**（要回传的场景） | 能，量一下那个任务在目标 CPU 上的吞吐即可 |
| G24.2 | 不是盘限 | 该任务**不是盘限**的——即 CPU 侧吞吐低于盘带宽，CPU 真的成了瓶颈 | 需要阵列，本工程没有 |
| G24.3 | 计算段占比 | 压缩那一格：真核函数的计算段占比 | **不能**，本机无 `nvcc`，前置是 CUDA 工具链 |
| G24.4 | GPUDirect 可用 | GPUDirect Storage 可用——它绕开主存与 PCIe 那一跳，会推翻「GPU 吃数据 56.69 GB/s」这个传输地板 | 未验 |

⚠️ **G24.1（缺硬件指令） 与 G24.2（不是盘限） 必须同时成立。** 只满足 G24.1（缺硬件指令）（CPU 没指令所以慢）而 G24.2（不是盘限） 不成立（整件事是盘限的），那 CPU 慢也无所谓——它本来就在等盘。

**射程**：四个闸是重开的充分条件，不是排期的判据；G24.3（计算段占比） 与 G24.4（GPUDirect 可用） 今天测不了，前置分别是 CUDA 工具链与 GPUDirect Storage，G24.2（不是盘限） 要阵列。已定项 4 定的是「什么现象会让它重开」，不定重开之后按什么次序做。

**依据**：

- E21（GPU 卸载的净收益）：G24.1（缺硬件指令） 的两个阈值（50.3 GB/s 单程、28.5 GB/s 来回）与 G24.4（GPUDirect 可用） 要推翻的那个地板（56.69 GB/s）都是它量出来的；判据 1 的形式也出自它。
- 用户定案（说不出什么现象会让它重开，暂缓就是无限期搁置），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:82-94`（整段抄，未转述）**

```markdown
#### 已定项 5：合法性的来源

**定案**：三个候选场景全部落在 `.claude/rules/fs-design.md` 记账纪律的**第三格**（后台、可续做、非决策路径，明写「无戒律」）⇒ 引入 GPU 不与任何已定纪律冲突。**三格划分是 D24（后台重活能不能卸给 GPU） 合法性的全部依据——一旦某个场景挪进第一格（运行时决策路径），D24（后台重活能不能卸给 GPU） 对它当场失效。**

**射程**：与别的决策的连锁三条：D2（RAID 条带策略） 那边，EC 重建是 D24（后台重活能不能卸给 GPU） 最可能有收益的一格；D3（空间分配） 那边，常驻后台整理属第三格，是 GPU 的合法作用域；D9（加密） 那边，scrub 的校验和与 MAC 计算量由它决定，**无 AES-NI 那一档是 GPU 唯一有明确数据支持的入口**。已定项 5 不判这三格值不值得做，那是已定项 1 与已定项 3。

**依据**：

- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张三格表：第三格逐字「后台、可续做、非决策路径（销毁快照、整理、scrub）」，明写「无戒律」。
- 无实验：一个场景落在哪一格是按那条纪律自己的判据判的（「被问到那一刻有没有答案」），没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:1-10`（整段抄，未转述）**

```markdown
## E174 借据丢掉时等待与显存回还 —— 装置写好、没上卡跑

<!-- doc-lint:not-numbers G1 G2 Q1 Q1b Q2a Q2b Q2c Q3 Q3b Q3c K1 K2 K3 K4 K5 K6 K7 K8 K9 B1 B2 B3 B4 B5 B6 B7 F1 F2 F3 F4 F5 F6 F7 F8 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 P1 P2 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 M23 M24 M25 M26 M27 M28 -->

跑前登记 `research/prompts/e174-preregistration.md`（装置与单测写完、真卡产物一次没跑之前的 sha256 `b2d5cd0c56552ec1ad322d5d8cca22f1fcb705617cb876d3726635058edc0d92`；执行员在第十二节「修订」写了 1 条：补一条臂「在飞·中」、钉死几处登记没写死的读法，原判据一字没动）；岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md` 的 G2 一行里"乙"的两个数：借据丢掉时先等设备把这笔借据提交的活做完再销账（`device.poll` 用等待那一种），等一次要多久（G2-乙-等待）；等完、缓冲丢掉之后，下一次 `nvidia-smi -q -d MEMORY,PIDS` 读到这个进程占着的是否已经不含那笔缓冲，含多少、隔多久才不含（G2-乙-读数）。问法里"再丢掉缓冲"按登记的读法：缓冲对象在借据活着时先丢，借据丢掉时再 poll，poll 返回那一刻算销账。

装置是独立手写的，不是入库装置：`research/e7-index-bench/src/bin/e174_lease_drop_wait_memory_return.rs`（bin 名 `e174-lease-drop-wait-memory-return`，特性 `e174-gpu`，依赖沿用 E163（GPU多卡算单元校验和） 那份钉死的 `wgpu =30.0.1`、`pollster =1.0.1`），不依赖 `crates/`、不与实现共用代码。设备怎么起（Vulkan、`required_limits: adapter.limits()`、`MemoryHints::Performance`）、读数口径（`nvidia-smi -q -d MEMORY,PIDS` 按进程号把 `Used GPU Memory` 加起来）、`PollType::Poll` 的写法都抄自 `crates/` 今天的取法，开跑前用登记第十一节那 7 条 `grep -c` 回比过：输出依次 2、1、1、1、1、1、1，与登记写的值逐条相同；驱动版本现读是 595.84（`nvidia-smi --query-gpu=index,name,pci.bus_id,memory.total,driver_version --format=csv` 两张卡都报 595.84，Total 32607 MiB 与 16311 MiB）。

**现状**：装置、单测、变异表、判法（`judge` 子命令）、清单与续跑写好；真卡上的第一段与第二段都没跑，G2-乙 的两个数没有量。不起卡能做的都做了：单测 19 个、变异表 28 条全抓（无效 0、没红 0）、不起卡的自检 31 条全绿、父子流水线在模型卡上走通（续跑与一次跑完逐字节相同）。要上卡的命令在「复跑」一节。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:11-18`（整段抄，未转述）**

```markdown
### 做了什么

- **一笔借据的步骤**（登记第五节）写成对着 `CardHost` 这个 trait 的一份代码，有两份实现：真卡（`WgpuCardHost`）与模型卡（`SimulatedCardHost`，虚拟时钟，读一份报告记 50 毫秒，小于 256 MiB 的缓冲共用一块 256 MiB 的块、块用过就留着，更大的单独拿，释放要等缓冲被丢掉、提交做完、之后有一次 poll）。登记第九节的 M1–M7 都改在对着 trait 的那份代码上，所以不起卡也有单测可红。模型卡只复现步骤的算术，不是 wgpu 与驱动的行为，不拿来当 G2-乙 的读数。
- **子命令**：`run`（父进程，按清单一个接一个起子进程，等它退出后整份读、不在转发输出的循环里打时间戳；续跑、补轮、原文压成 `.raw.gz`、扣住不丢组出现 F1 就停）、`child`（一个装置进程）、`judge`（读产物判，确定性）、`checks`（不起卡的自检，确定性）、`plan`（清单各阶段的格数与子进程数，确定性）、`simulate`（在模型卡上跑某一格几轮，试 `judge` 用）；`run --simulated` 让每个子进程跑在模型卡上，`--limit-jobs N` 冒烟，`--skip-light` 先不跑在飞·轻。
- **清单**（`research/results/e174-lease-drop-wait-memory-return-plan-first-2026-10-08.out`、`…-plan-second-2026-10-08.out`）：第一段 128 格、首轮 640 个子进程，第二段 24 格、首轮 120 个子进程；"扣住不丢"最先，在飞·轻最后。
- **修订 1**：登记第九节 K9 要 e < 0.8 W，而 e 至少含 U_peak 那一次 -q 读与 B5 另读的一次 compute-apps（各约 0.05 秒），在飞·轻 W = 50 毫秒的 0.8 倍只有 40 毫秒，K9 在登记自己的数下必红。在飞·轻照登记保留、K9 照登记判，另补一条臂「在飞·中」（空转目标 250 毫秒）给每个用在飞·轻的组；已经作废的借据不再读窗口。其余是读法钉死：一轮有效的定义、补轮从 6 起至多 3 次、次序、B5 字面读法、F1 停机、判别力自证怎么取新 τ。
- **判法**：`judge --input 产物` 对每格按"第 1 笔"与"第 2、3 笔"两栏、取轮号最小的 5 个有效轮，逐轮列出 Q1（等多久）、Q2a / Q2b（t0 之后第一份是否含）、Q2c（隔多久不含）、Q3（窗口末留着）、Q3b（按块多拿）、Q3c（wgpu 分配器），再给一行 `name=verdict` 汇总 F1–F8 与几何敏感性；汇总里"…_ok"类字段写成通过的样子，`false` 就是那条没过（F2、F3、F4 打中是登记认的正当结果，不是装置有错）。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:19-37`（整段抄，未转述）**

````markdown
### 复跑

```
# 仓根；装置在 research/（不依赖 crates/）。真卡上的命令经 gpu-quiesce-run.sh 清场、经内存包装、线程上限 8
cd /home/fy5090/code/singlefs/research
bash scripts/capped.sh 8 cargo build --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return
# 不起卡：单测 19 条、变异表 28 条、自检、清单
bash scripts/capped.sh 8 bash scripts/run-with-memory-cap.sh 12G cargo test --release -p e7-index-bench --features e174-gpu --bin e174-lease-drop-wait-memory-return
MUTATE_JOBS=9 MUTATE_MEMORY_MAX=6G MUTATE_TIMEOUT=200 bash scripts/capped.sh 8 bash scripts/mutate.sh e174-lease-drop-wait-memory-return e7-index-bench/src/bin/e174_lease_drop_wait_memory_return.rs mutations/e174_lease_drop_wait_memory_return.tsv
bash scripts/capped.sh 8 bash scripts/run-with-memory-cap.sh 8G ./target/release/e174-lease-drop-wait-memory-return checks
bash scripts/capped.sh 8 bash scripts/run-with-memory-cap.sh 8G ./target/release/e174-lease-drop-wait-memory-return plan --segment first
# 上卡：先冒烟两个子进程，再第一段（不含在飞·轻），再在飞·轻，再第二段；同一条命令重起就是续跑
bash scripts/gpu-quiesce-run.sh -- bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 8 ./target/release/e174-lease-drop-wait-memory-return run --segment first --skip-light --limit-jobs 2 --output results/e174-lease-drop-wait-memory-return-segment-one-<日期>.out
# 判（确定性，读产物）
bash scripts/capped.sh 8 bash scripts/run-with-memory-cap.sh 8G ./target/release/e174-lease-drop-wait-memory-return judge --input results/e174-lease-drop-wait-memory-return-segment-one-<日期>.out --scope first
```

`research/scripts/replay.sh` 登记两行（`E174|@driver_e174_checks` 与 `E174|@driver_e174_simulated_judge`，只登记不起卡、不含计时字段的两份；真卡产物含计时与显存读数，两次跑不同，等产物出来再登记它的 `judge` 输出）。replay.sh 整份目前一开跑就在准入表检查上退 2（报 E142 在复跑表里有 2 行），与这两行无关，所以这两行没经 replay.sh 复跑，是手工跑同一条命令与产物 `cmp` 逐字节相同。

````

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:38-44`（整段抄，未转述）**

```markdown
### 读数：不起卡的几份（模型卡数据，不是 G2-乙 的读数）

- **自检**（`research/results/e174-lease-drop-wait-memory-return-checks-2026-10-08.out`，33 行，31 条自检）：第 32 行 `E7RESULT name=verdict scope=checks self_checks_total=31 self_checks_failed=0 void_conditions=0`。检查本身分得出差别：第 16–18 行三条负对照（模型卡被注入别的进程号、compute-apps 比 -q 多 3 MiB、标定不在带内）各自打出对应的作废理由。第 12 行 `E7RESULT name=self_check id=simulated_light_trips_k9_at_the_registered_numbers passed=true`：模型卡上在飞·轻按登记的数 K9 必红，复现的是修订 1 的算术。
- **清单**：`…-plan-first-2026-10-08.out` 第 10 行 `E7RESULT name=verdict scope=plan segment=first cells_total=128 first_pass_children_total=640 void_conditions=0`；`…-plan-second-2026-10-08.out` 第 3 行 `E7RESULT name=verdict scope=plan segment=second cells_total=24 first_pass_children_total=120 void_conditions=0`。
- **父子流水线走通**（`run --simulated --segment second --skip-light`，子进程都在模型卡上）：`…-simulated-second-2026-10-08.out`（243 行）第 242 行 `E7RESULT name=verdict scope=run segment=second cells_total=12 cells_incomplete=0 children_total=60 children_complete=60 children_failed=0 children_void=0 stopped_by=none`，原文压缩包 `…-simulated-second-2026-10-08.out.raw.gz`（`zcat` 读得出，每个子进程一段）。同一条命令用 `--limit-jobs 7`、`--limit-jobs 20` 分三次续跑，与一次跑完的产物和原文 `cmp` 逐字节相同（草稿读数，续跑文件不入库）。
- **判法在模型卡产物上**：`…-simulated-second-judge-2026-10-08.out`（26 行）第 25 行 `E7RESULT name=verdict scope=second columns_judged=24 cells_void=0 hold_control_holds=not_run wait_arm_next_reading_absent_everywhere=false wait_arm_end_of_window_not_kept_everywhere=false wait_always_returns=true poll_arm_also_absent_where_wait_arm_absent=false in_flight_cells_void_for_work_not_in_flight=0 long_window_clear_within_window=not_run geometry_same_as_main_grid=not_run done_wait_expensive_candidate_columns=0 discrimination_real_data_flipped_columns=12 discrimination_synthetic_flips=true`。这几个 `false` 来自模型卡自己的规则（小于 256 MiB 的借据让进程多占一块 256 MiB 的块、块用过就留着），模型卡上的数不是 wgpu 与驱动的数；`not_run` 是这份产物里没有对应的组。判别力自证：真数据里 τ 挪到含着的读数的中位数时 12 栏翻面，另用模型卡产物自证 `discrimination_synthetic_flips=true`。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:45-50`（整段抄，未转述）**

```markdown
### 单测与变异

单测 19 个：`grep -c '^test .* \.\.\. ok$' research/results/e174-lease-drop-wait-memory-return-unit-tests-2026-10-08-r2.out` 输出 19，同一份文件里 `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`。变异表 `research/mutations/e174_lease_drop_wait_memory_return.tsv` 28 行（`grep -c '' research/mutations/e174_lease_drop_wait_memory_return.tsv` 输出 28），`mutate.sh` 输出 `research/results/e174-lease-drop-wait-memory-return-mutate-2026-10-08-r2.log`：`grep -c '^✅' …` 输出 28，`grep -c '^⏭' …` 与 `grep -c '^❌' …` 都输出 0；第 30、31 行 `计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条` 与 `已还原，基线仍全绿`。单测与变异各有一份更早的输出（`…-unit-tests-2026-10-08.out`、`…-mutate-2026-10-08.log`，同样 19 个、28 条全抓），对的是之后又改了 `run` 续跑语义（已有 `name=done` 行但要补轮或换一组格时接着跑并追加新的 verdict 与 done）之前的源码，留着，正文引的数以 r2 为准。

变异表的行（页内编号）：M1–M7 是登记第九节的七条（M1 等待臂的 `Wait` 换成 `Poll`、M2 拆缓冲张数向上取整换成向下取整、M3 把 BAR1 下的 `Used` 当成进程占着的、M4 建缓冲时每张取一半、M5 扣住不丢组在第 6 步照样丢缓冲、M6 t0 记在 poll 开始之前、M7 在飞档第 4 步也等完）；M8–M28 是执行员补的二十一条，改判定一侧的门槛与比较（M8–M13、M25）、读数解析与分配器比较（M15–M17）、外层计时与 K7、K9、扣住不丢组的窗口比较（M18、M22–M24）、续跑与清单（M14、M19–M21、M26–M28）。登记第九节说"判定脚本那几处门槛的变异由第八节的判别力自证代替"，这里另补了变异，不算替代。M5 登记写的取样点（"等待臂某一格 R_next 已经 ≤ τ 的 S"）在模型卡上就有（等待臂 2048 MiB 的 R_next 为 0），变异被抓，真卡上有没有这样的格要看产物。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:51-59`（整段抄，未转述）**

```markdown
### 它答不了的

- 没上卡：G2-乙 的两个数（等一次要多久、等完之后下一次读数含不含、隔多久不含）一个都没量；模型卡上的数只证明步骤与判法的算术。
- B5（-q 与 `--query-compute-apps` 逐 MiB 相等）依赖本机驱动把 Vulkan 进程列进 compute-apps；不起卡没法核。不列的话每个子进程都因此作废（登记 V7），所以先用 `--limit-jobs 2` 冒烟看前两个子进程。
- 在飞·轻的 K9 在登记的数下预计必红（修订 1 的算术，模型卡复现，真卡上 e 的值没量）；在飞·中 250 毫秒是否稳在 K9 之内取决于真卡上 nvidia-smi 一次读的时长，机器忙时读一次变长会让 e 增大。
- 第二段（同一设备两条线程）的线程 B 在模型卡上没有对应物：`run --simulated` 的第二段产物里 B 不存在，B 的真实行为只能上卡看。
- 没量的旋钮照登记：`MemoryHints` 只取 `Performance`，后端只取 Vulkan，驱动只取 595.84，一个进程里借据只连做 3 笔。
- 登记第四节自己推出的两条（gpu-allocator 的块大小、"被还在飞的提交用着的缓冲要等下一次 maintain"）是假设，装置没有按它们写任何门槛。

```

**出处 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:60-65`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-08 不受影响：被测条款三条硬约束不带显存读数或等待时长的数，装置只在测试与验证一侧、不进格式；决策依据段没引 E174（借据丢掉时等待与显存回还），真卡产物出来之后再判要不要升成支撑 |

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:6-14`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **做不做** | 暂缓，不排期；暂缓不等于否掉，算力受限那一格仍开着，GPU 这条线不再继续测 **状态：已定。** |
| 2 | **三条硬约束** | 绝不能成为必需（一个比特都不许进格式）、不许进前台路径、卸出去的结果必须能被 CPU 独立复核 **状态：已定。** |
| 3 | **判据取什么形式** | 取 CPU 侧的多核饱和吞吐与 GPU 的两个传输地板比，不是「单核 × 核数」，也不是「有没有硬件指令」 **状态：已定。** |
| 4 | **重开判据** | 四个闸 G24.1（缺硬件指令） 到 G24.4（GPUDirect 可用），前两个必须同时成立 **状态：已定。** |
| 5 | **合法性的来源** | 三个候选全在 `.claude/rules/fs-design.md` 第三格；某个场景挪进第一格，D24（后台重活能不能卸给 GPU） 对它当场失效 **状态：已定。** |

```
