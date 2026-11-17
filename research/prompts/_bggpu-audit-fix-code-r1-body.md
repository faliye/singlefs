# bggpu-audit-fix-code-r1：产品 GPU 包审核三条修复（开卡量代价出错的泄漏、重复条目键、映射建缓冲的暂存进账）与只改名的产品文件，代码轮第一轮正文（2026-11-12）

<!-- doc-lint:not-numbers T1 T2 T3 T4 G01 G02 G03 G04 -->

## 一、题面

`records/2026-11-10-singlefs-background-gpu严格审核.md` 读码立了四条（G01–G04），2026-11-11 那一批改掉（调度记录 `records/2026-11-11-background-gpu审核四条修复.md`，实现员报告 `~/.local/share/singlefs/work/bggpu-audit-fix/result.md`，主工作区直接改、没有补丁目录）。这一轮判其中三条的代码做的是不是条款说的：

- G01：开卡量代价的那次提交出错、提交还在跑时，那张卡要整张泄漏、记进进程泄漏账，不析构（D26（后台整理与放置回收） 已定项 13 第 4 条）。
- G03：一批里同一个条目键出现两次，原来 `assert`，改成这一批按出错交回、计数不动，核心层收到一种不归哪张卡的出错（用户 2026-11-11 弹窗选「改成交回错误」）。
- G04：映射着建的缓冲在卡上另占一份同尺寸暂存，要进显存账、进切批的判装不下（同一项第 3 条「缓冲、暂存区一起算在里面」）；建到一半显存不足时，已建那几份的暂存要等一次提交冲掉才放账。

G02（开卡器期限）不在这一轮：它的现行形态（每卡期限取连同重开的最长等待）由另一题 `c996-code-r1` 判过（`research/prompts/c996-code-r1-main-verification.md`「这一轮被判过的文件」一节 `gpu_opener_of_this_process.rs` 那一行）。C996（开卡打开本机全部独显）的块同样归那一题，这一轮不攻；腿只攻本文第二节点名的落点与它们和周围代码的交叉。

这一轮另带一格：全仓命名回扫（`records/2026-11-11-全仓命名回扫.md`）在 25 份产品代码文件里只改了标识符的名字，判「改名不改行为」（T4）。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

没有改动全文那一份：这三条与 C996 的两份补丁、命名回扫先后落在同一批文件上，都没提交，按文件切不出只属于这三条的 diff。腿照第二节的函数名在冻结副本里读现行代码；要看这一批之前的样子，`~/.local/share/singlefs/work/bggpu-audit-fix-code-r1/baseline/` 是提交 `8ed2ea484` 的同一批文件（那一版里没有的两份不在）。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 冻结副本 `~/.local/share/singlefs/work/bggpu-audit-fix-code-r1/repo/`：开工快照那一刻主工作区的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.claude/kb`、`.claude/rules`；被判文件的 sha256 在 `research/prompts/bggpu-audit-fix-code-r1-snapshot/crates-sha256.txt`。
- 被判的产品代码，前三格：`crates/singlefs-background-gpu/src/` 的 `background_gpu.rs`、`background_gpu_compute.rs`、`memory_limit_and_chunks.rs`、`two_kernel_reconciliation.rs`、`checksum_kernels.rs`、`region_record_kernels.rs`、`live_slot_kernels.rs`、`gpus_of_this_host.rs`、`waiting_for_the_gpu.rs`、`gpu_opener_of_this_process.rs`、`lib.rs`、`bin/e202_gpu_cpu_gc_comparison.rs`；`crates/singlefs-core/src/background_compute.rs`；`crates/singlefs-mount-runtime/src/lib.rs`。
- G01 的落点（`background_gpu.rs`）：
  - `failure_of_the_measurement_dropping_or_leaking_the_card(card, failure, &GpuMemoryLeakedInThisProcess)`：账上有等超过上限、没做完的提交就 `mem::forget` 那张卡，把这一次的字节（`bytes_held` 减开卡时已算上的之前挂载泄漏的那一片）记进进程泄漏账；没有就照常丢；两支都交回原错。它调 `gpus_of_this_host.rs` 的 `gpus_dropped_leaking_those_still_running` 与泄漏账的 `note_the_gpus_leaked_when_dropped`。
  - 判别子 `card_has_a_submission_running_past_its_wait`、记账 `card_leaked_when_dropped` 与 `BackgroundGpuOverTheCards` 的 `Drop` 共用。
  - 调用点：`started_gpu_with_its_cost_measured`（量代价出错那一支）。C996 那一批把开卡拆成 `opening_of_a_card_within_the_limit` 与 `started_gpu_in_use`，挂载与切换都经它们走到这里。
  - 开卡线程外面那一层等：`waiting_for_the_gpu.rs` 的 `outcome_of_opening_a_gpu_within_the_limit`。
- G03 的落点：
  - `two_kernel_reconciliation.rs`：`SameEntryKeyTwiceInOneBatch { position_of_the_first_occurrence, position_of_the_second_occurrence }`；`first_entry_key_repeated_in_the_batch`；`reconcile_the_two_kernel_outcomes` 在动计数之前整批查键，重复就交回 `Err`；`reconcile_unless_the_batch_failed` 要求 `BatchFailure: From<SameEntryKeyTwiceInOneBatch>`。
  - `background_gpu.rs`：`ChecksumBatchFailed`、`RegionRecordBatchFailed`、`LiveSlotBatchFailed` 各加成员 `SameEntryKeyTwiceInOneBatch(..)`。
  - `background_gpu_compute.rs`：`core_failure_of_a_repeated_entry_key` 把它报成核心层的 `BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch`（`crates/singlefs-core/src/background_compute.rs`，主 agent 自己加的成员，不归哪张卡）。
  - 实现员报告「走不到的理由」一节写的那一处 `unreachable!` 已经不在：`grep -n 'unreachable!' crates/singlefs-background-gpu/src/background_gpu_compute.rs crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs` 零命中（量的，2026-11-12）。
- G04 的落点：
  - 口径在 `memory_limit_and_chunks.rs`：`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`（2）、`COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS`（2 × 2 + 2）；`ChunkFootprint::bytes_held_by_a_chunk`；切批判装不下 `chunk_fits_with_the_staging_of_its_input`（把交给被验函数 `gpu_chunk_fits` 的上限先扣掉输入的暂存）；最大一片 `largest_piece_input_bytes`（交给被验函数 `largest_gpu_piece_input` 的上限是每段加每条再加剩下的一半）；建缓冲出错的结账 `GpuMemoryAccount::settle_the_chunk_whose_buffers_were_not_created`。被验函数在 `crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs`、`gpu_gc_proofs.rs`，这一批没改它们。
  - 三段的占用：`checksum_kernels.rs`、`region_record_kernels.rs`（连同 `mapping_table_bytes_held_with_their_staging`）、`live_slot_kernels.rs`（记录表每片条数的除数）。
  - 外壳 `background_gpu.rs`：三处 `bytes_held_by_this_chunk`；`buffers_and_both_readbacks_created_within_one_out_of_memory_scope` 的真建尺寸断言；`GpuInUse::buffers_created_within_the_free_memory` 建缓冲出错时交一次空提交、在等的上限之内等它，等到了放账、交回建缓冲那一次的错，等不到账留着、记成还在跑的提交、交回等的那一次的错。
  - 实现员读 wgpu-core 30.0.1 的依据（报告 G04 一节「改法四」，腿现查，源码在 `~/.cargo/registry/src/`）：`src/device/queue.rs` 的 `Queue::submit` 对空的命令缓冲照样 `pre_submit` 待写；`src/resource.rs` 的 `unmap_inner` 在 `BufferMapState::Init` 一支把暂存收进待写。
- 验证到的数（量的，任务号在调度记录「进展」一节）：库内单测 13208 过 158 条；显存容量契约 13211 过 5 条；形式化契约 13210 过 2 条；与 CPU 替身对拍 13212 过 11 条；验证包逐种注入 13218 过 12 条；点名 23 行变异 13243「抓到 23 条、没红 0 条、无效 0 条、锚点失效 0 行」；真卡 13329 区域扫描 6 过、13328 校验和 7 过 1 红（`the_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu` 读到 `recuts=0`，单跑重交 13333 过、`recuts=1`，判不稳定）。这些数都早于 C996 的两份补丁与它的写回，只当线索。
- 实现员停下、主 agent 收下的两处（攻击面）：
  - 建缓冲出错之后等不到冲掉待写的空提交时，交回的是等超过上限、走重交那条路，重交次数可能多算 1（与欠账 C952（单次抽样的条目与泄漏账持有方）记的那个洞同类）。
  - `GpuInUse` 里交空提交、等它的那一段要真设备，单测只罩了它调的结账函数。
- 没在真卡上量过的前提（推的，没量过）：暂存的优先分配落在设备本地堆。审核记录 G04 一节写明优先分配退到主机内存时这一份不占显存；现行口径一律按占着算，是往多里记。
- T4 的对象：`crates/singlefs-core/src/` 的 `admission.rs`、`block_device.rs`、`bytes.rs`、`checksum.rs`、`code_two_tree.rs`、`container_tree.rs`、`deleted_inodes.rs`、`inode_tree.rs`、`journal.rs`、`lineage_tree.rs`、`logged_operations.rs`、`pointer.rs`、`records.rs`、`region_scan.rs`、`retirement_tree.rs`、`root_record.rs`、`root_ring.rs`、`tombstone_tree.rs`、`truncate.rs`、`unit.rs`、`unlink.rs`、`write_request_split.rs`；`crates/singlefs-reference-filesystem/src/lib.rs`；`crates/singlefs-replay-prefix/src/` 的 `compaction_plan_proofs.rs`、`unlink_proofs.rs`。主 agent 的量法 `research/prompts/bggpu-audit-fix-code-r1-main-checks/rename-only.py`：每份文件取提交 `8ed2ea484` 那一版与快照那一版，去掉注释、空白与收尾逗号之后切成记号，两份记号数相等、非标识符记号逐个相同（字符串字面量在内）、标识符之间是一张前后一致的一一映射，25 份都判「只改名」；每份的改名表在同目录 `rename-only.out`。这个量法只在一份文件之内判一一映射。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13（`#### 已定项 13` 起到下一个 `####` 为止）第 3、4、8 条。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　开卡量代价出错：泄漏的判别子与账
量代价的那次提交在哪几种结局下会留下「还在跑」的标记，`failure_of_the_measurement_dropping_or_leaking_the_card` 的判别子读的是不是同一个标记；有没有一条路径提交已经交出去、等的那一步出错，而判别子读到「没有在跑」于是照常析构（析构里 wgpu-core 的 `Queue` 会不带时限地等）。泄漏时记进进程账的字节算得对不对：与卸载那一条（`BackgroundGpuOverTheCards` 的 `Drop`）是不是同一个算式，之前挂载泄漏的那一片有没有重算一遍或漏扣。量代价之外、开卡过程里别的出错出口（建设备之后、建管线、切换开卡那一支）有没有同样握着已提交的队列而照常丢的。开卡线程外面那一层到点之后，线程里这张卡走到哪一步、账记不记得上。

### T2　重复条目键：交回的是不是「这一批出错」
`reconcile_the_two_kernel_outcomes` 查出重复时计数、连续不确定的轮数、CPU 参照的调用是不是都一条没动；三段调用方（校验和、区域扫描、选段打分）交回的出错经 `background_gpu_compute.rs` 到核心层是哪个成员，核心层对这个成员怎么处置（停不停、报不报警、重不重交、归不归卡），与第 4 条「一批算出错…这一批作废」「GPU 用不上时重交、报警、停下」是不是同一件事，会不会把调用方的缺陷当成卡坏了去重交三次或停掉一张好卡，或反过来悄悄吞掉。选段打分那一段在比对之前另有一处键与整段清单相等的断言，它与新出错谁先到。`From` 的每个实现有没有把位置报反。

### T3　暂存进账：口径在每一处是不是同一个数
逐段核「映射着建的各乘 2、回读各 1」：校验和、区域扫描（头片与映射片）、选段打分三段各建哪几份缓冲、哪几份是映射着建的，`ChunkFootprint` 的两个系数、外壳的 `bytes_held_by_this_chunk`、真建尺寸断言三处算出来是不是同一个数，有没有哪一份缓冲映射着建而系数里只算了一份，或不映射而算了两份。`chunk_fits_with_the_staging_of_its_input` 与 `largest_piece_input_bytes` 把上限改小之后交给被验函数：被验函数的 `ensures` 在改小的上限上给出的结论，推不推得出「连同暂存不超过原上限」；输入按 4 字节对齐、上限减不下去（下溢）、每段加每条已经超过上限这几个边角各交回什么。建到一半显存不足那一支：空提交真能冲掉待写吗（读 wgpu-core 30.0.1 源码，不靠报告）；等到了放账那一刻，失败那一次已建的缓冲与暂存是不是真的都放了；等不到时账留着、记成还在跑，后面重交与减半重切会不会在账上还记着这一段时又去记一段；重交次数多算 1 会不会让「超过 3 次停卡」早一次或晚一次。这一格最脆弱的是哪一句，写出来。

### T4　只改名的 25 份产品文件：改名不改行为
主 agent 的量法只在一份文件之内判一一映射。腿攻它罩不到的：改了名的条目跨文件、跨包的引用是不是都跟着改了（编得过是量的：`cargo build` 任务 13941、14022 退 0，早于快照，只当线索）；有没有名字进了行为——`Debug` / `Display` 的输出被解析或被比较、`stringify!` 与宏按名字拼、序列化的字段名、盘上格式里的名字、`#[cfg]` 与特性名、Verus 的触发器与 `spec fn` 名被登记表按名字点到（`.claude/gate.d/verus-pins.tsv`、`crates/mutations.tsv`、`.claude/gate.d/stage-inputs.tsv` 这类按名字找的登记）；改名之后有没有在某个作用域里遮蔽或撞上另一个已有的名字，让同一个调用解析到另一个条目（trait 方法与固有方法同名这类）。量法本身：去掉收尾逗号、不看注释这两步会不会放过真改动（文档注释里的 doctest、宏里逗号有语义的地方）。

### 检查单：走不到的理由二次查实
实现员报告「走不到的理由」一节只有一处：`crates/singlefs-background-gpu/src/background_gpu_compute.rs:423 core_failure_of_a_repeated_entry_key 的 unreachable!：只在核心层交来的一批里条目键重复时走到`（理由全文在报告同一节）。这一处已由主 agent 换成报 `BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch`，`unreachable!` 不在了（第二节的 grep）。腿另查前三格的落点里新加的 `expect`、`unreachable!`、`panic!`、`assert!` 各依赖什么、调用点追得到追不到；G03 改的就是一处「调用方的 bug 所以 assert」，同类的断言还有没有（选段打分那一处键清单断言算一处）。

### 检查单：类型与名字
新名字：`SameEntryKeyTwiceInOneBatch`（结构体与四个枚举里的同名成员）、`first_entry_key_repeated_in_the_batch`、`core_failure_of_a_repeated_entry_key`、`failure_of_the_measurement_dropping_or_leaking_the_card`、`card_has_a_submission_running_past_its_wait`、`card_leaked_when_dropped`、`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`、`COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS`、`chunk_fits_with_the_staging_of_its_input`、`largest_piece_input_bytes`、`mapping_table_bytes_held_with_their_staging`、`settle_the_chunk_whose_buffers_were_not_created`。显存字节、条数、份数这几个量在这些函数的签名里是裸 `u64` 还是各有类型，腿判。T4 的新名字在 `rename-only.out`，腿抽查有没有缩写、空泛词。

### 检查单：不兼容旧代码与旧格式
主 agent 读前三格的落点，没看到认旧版本的分支、旧布局号或迁移路径；这一批不碰盘上格式。T4 的 25 份里字符串字面量逐个相同（量法判的）。腿现查。

### 检查单：不新建判定库
主 agent 读前三格的落点，没看到新的判定存放处。腿现查。

### 检查单：所有任务多机可用
新单测用替身卡（`CardComputedOnTheCpuForTests`）与纯函数，不要卡；真卡用例只改了上限的数。腿查新测试与落点有没有读本机独有状态、写死地址或卡号。

### 检查单：测试优先多线程
本轮改动的新测试都是单条纯函数或替身卡用例，各自一个场景；没有新的切片枚举（主 agent 读的）。腿判有没有该并行而串行的。

### 检查单：带 --item 的命令与项
实现员报告「要主 agent 跑的命令」第 4 组：crates-mutation-replay 带 `--item` 点名 22 行（12 行新写、10 行改锚），formal-verification 点名 4 项（被验的切批函数）；产品 GPU 包没有崩溃枚举用例与崩溃放量分项。腿核这句话，并核那 12 行新变异在冻结副本的 `crates/mutations.tsv` 里原文各命中一次（读，不跑）。

### 检查单：新崩溃流多罩了什么
本轮改动没有这一类：查过 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行与这一批改动的文件清单，没动崩溃枚举用例（腿现查）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 与检查单八格，T3、T4 主攻 | 核「代码今天做的是不是条款与改法声称的那件事」：三段每一份缓冲写文件:行号与它在三处口径里各算几份；wgpu-core 那两处写路径与行号；T4 逐类写查过哪些登记表、命中什么；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T4 与检查单八格，T1、T2、T3 主攻 | 在冻结副本的拷贝上造反例：提交已交出去而卡被析构的路径；重复键被当成卡坏或被吞掉的输入；连同暂存超过上限的一段（给出上限、条目尺寸与算式）；账上重复记一段的时序；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改。读码推出来、没量过的写「推的，没量过」。
- wgpu、wgpu-core、wgpu-hal、gpu-allocator 的源码在 `~/.cargo/registry/src/` 下，读，不改。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令）。要上卡的命令写明要卡、写明点名的测试目标，不写卡号。
- C996 的块（只开参与的卡、重开、切换开卡与关卡）不攻；攻到的缺陷落在那些块里的，单列一节「落在别的题里的」，不计进这一轮的打中。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
