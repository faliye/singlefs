# m4-gpugc-shared-bytes-code-r1：里程碑四七节 GPU 垃圾回收，单元字节共享与产品 GPU 包撞名修复，代码轮第一轮正文（2026-11-06）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节交用户之后，用户 2026-11-06 定核心层「先甲档，量完再定」（原话与出处在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 874 行）。照这一句改的是第 86 批（`~/.local/share/singlefs/work/m4-gpugc-shared-unit-bytes-writer/patch/`，续改第 89、90 批合进同一份整补丁 `~/.local/share/singlefs/work/m4-gpugc-shared-unit-bytes-writer-r3/patch/`）：`PublishedUnit.bytes` 由 `Vec<u8>` 换成 `Arc<[u8]>`、`FileVersionUnits.data_units` 换成 `Vec<Arc<[u8]>>`，`PoolVersion` 两员与版本列表、挂载交回值换成 `Arc<TransactionOutput>`，要改的地方一律显式 `Arc::make_mut`。同一段里另有第 88 批（`~/.local/share/singlefs/work/m4-gpugc-name-collisions-writer/patch/`）：用户 2026-11-06 定「撞名必须修复，不得撞名。」，产品 GPU 包里同一编译目标的同名条目去重或改名，其中产品代码 `low_and_high_words_of_the_slot` 留 `live_slot_kernels.rs` 一份改成 `pub(crate)`，`region_record_kernels.rs` 改为 use 它。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-gpugc-shared-bytes-code-r1-diff.md`（两份补丁里产品代码的块按打进主工作区的次序原样抄录，1981 行、13 个文件块；测试、变异表的块不在里面，整份补丁在各自的目录里）。第 86 批那一份取第 91 批补丁目录 `~/.local/share/singlefs/work/m4-gpugc-shared-unit-bytes-r4/patch/`：它的 `mount.rs` 一节是「主工作区打补丁前 → 与别的会话末尾新加的测试模块合好」，所以那一节的上下文里有别的会话的块，照第二节当上下文读。

## 二、实现今天的样子（主 agent 的观测，2026-11-06）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/`（全部补丁打进主工作区之后那一刻的主工作区，含别的会话这期间合入、没提交的改动；去掉了 `.git`、编译目录、私有配置 `multi-host.env` 与 `.env`）。被判的 13 份产品代码文件的 sha256 在 `research/prompts/m4-gpugc-shared-bytes-code-r1-snapshot/crates-sha256.txt`，冻结副本逐份对上。
- 被判的产品代码：
  - 第 86 批：`crates/singlefs-core/src/` 的 `transaction.rs`、`mount.rs`、`mounted_session.rs`、`recovery.rs`、`instance_table_sweep.rs`；`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`（只改了两条测试名）。
  - 第 88 批：`crates/singlefs-background-gpu/src/` 的 `adapter_inventory.rs`、`background_gpu_compute.rs`、`checksum_kernels.rs`、`gpus_of_this_host.rs`、`live_slot_kernels.rs`、`region_record_kernels.rs`、`two_kernel_reconciliation.rs`。
  - 这几份文件里不是这两批的块（别的会话的改动）当上下文读、不攻；腿只攻 `_m4-gpugc-shared-bytes-code-r1-diff.md` 里有的块。
- 第 86 批实现员报告（`~/.local/share/singlefs/work/m4-gpugc-shared-unit-bytes-writer/result.md`「设计取舍」一节）里写的落点：`Arc::make_mut` 五处——`mount.rs` 的 `unmount`、`continue_the_deletion_queue_until_it_is_empty`、`acquire_then_publish_until_the_continuation`，`mounted_session.rs` 的 `push_one_floor_raise_for_a_refusal_for_space`，`transaction.rs` 的 `PoolVersion::with_writes`（冻结重发那一支走它）；新加 `PoolVersion::of_a_shared_version_of_trees`、`shared_version_of_trees`、`into_shared_version_of_trees`；`into_file_version` / `into_version_of_trees` 经 `Arc::unwrap_or_clone` 交回；没改的同形深拷留着（整理那一趟 `CompactionRun::versions_published` 仍按值）。腿以冻结副本为准，报告只当线索。
- 验证段的数（量的，出处在调度记录第 86–90 批各行；合入后验证与按测试函数入库那一趟在开工快照之前跑完，数以那几行为准）：交接副本上三个包的 clippy、改到的测试目标、十五行变异复判（抓到 15）、推环那一行证红（抓到 1）；第 88 批 clippy、包测试、两行变异复判（抓到 2）、证红、撞名格（产品 GPU 包 0 处）。
- 压着的条款：
  - `.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 17（第 372 行起，到下一个 `####` 为止，含射程与依据里那句 ⚠️）：报错之后不许假设可以照旧复用同一份内存态重试；冻结重发那一支与 C381 的关系。
  - `.claude/kb/checks-owed.md` C381（根已落盘之后发布失败，分配器仍退回） 那一行（第 265 行）。
  - `.claude/rules/fs-design.md`「一个事务层，所有结构共用」（第 81 行起）与「记账是事务的副产品，不是事后的遍历」里「运行时与 checker 必须用不同的算法」（第 35 行）。
  - `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.1、Q2.2（甲档「要改的不只是类型」，`PartialEq`、录制流、写盘字节、checker 读的东西不变是推的；`Arc<TransactionOutput>` 可以独立做）。
  - `.claude/kb/milestone/04-fourth-txn.md` 第 140 行「## 七　GPU 垃圾回收」一节。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　过度外推与越位：改的真只是「持有方式」吗
第 86 批声称语义照旧、只换持有方式（相等、`Debug`、写盘与录制流照内容走）。逐处核：哪一处读到「是不是同一份」而不只是内容（`Arc::ptr_eq`、指针身份进键、按地址去重、`Arc::strong_count` 分支）；`Arc::make_mut` 在还有别人指着时先拷一份——五处里每一处，拷之前别的持有者（会话现行那一版、版本列表、挂载交回值、整理那一趟）各是谁，拷完之后该看到改动的那一方看没看到、不该看到的那一方是不是真没看到；`Arc::unwrap_or_clone` 交回时有没有一方仍以为自己拿着的是被交回的那一份。

### T2　内部矛盾：冻结重发与 D16 已定项 17
`PoolVersion::with_writes` 走 `Arc::make_mut`，报告写「那一刻只有一个引用，就地改」。核：冻结重发那一刻是不是真只有一个引用（版本列表、挂载交回值、调用方手里的那一份有没有还指着它）；多于一个时拷一份再改，会不会正好变成 D16（发布语义） 已定项 17 禁的「照旧复用同一份内存态重试」，或反过来让一方看到另一方的半截改动；与 C381 那一行写的「根已落盘之后发布失败，分配器仍退回」是什么关系。

### T3　哪一条最脆弱：没改的深拷与混用
报告留着没改的同形深拷（整理那一趟 `CompactionRun::versions_published` 按值、抬 F 那一串）与改成共享的并存：同一版在一处共享、在另一处深拷，之后一方 `make_mut` 改了，另一方以为相同而其实不同的格；`Arc<[u8]>` 的单元字节被测试或装置当 `Vec<u8>` 就地改坏（第 89、90 批改的那几处）时，改的是不是只有被测的那一份。

### T4　第 88 批：去重与改名改没改行为
`low_and_high_words_of_the_slot` 两份并成一份：两份原来逐字相同吗（46 的清单归它为「甲：正文相同」）；`pub(crate)` 之后可见性扩大有没有让别的模块误用；`region_record_kernels.rs` 改 use 之后 WGSL 拼出的源串、入口名与单元图的闭包（判法单元键）变没变；测试常量去重之后取的值与原来逐个相同吗。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4，T1、T4 主攻 | 核「代码今天做的是不是改法声称的那件事」：每处 `make_mut` / `unwrap_or_clone` / `Arc::clone` 写文件:行号、持有者是谁、钉它的测试或变异行名；T4 逐个去重项写两份原文是否逐字相同 |
| 云端攻方（Opus） | T1–T4，T2、T3 主攻 | 在冻结副本的拷贝上造反例：一方 `make_mut` 之后另一方看到或看不到改动的时序；冻结重发那一刻引用数多于一的路径；共享与深拷并存时内容分叉的输入；去重之后行为不同的输入 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；引补丁块写 `_m4-gpugc-shared-bytes-code-r1-diff.md` 那一节的补丁目录名，不写那份文件的行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改。读码推出来、没量过的写「推的，没量过」。
- 不起 GPU：真卡用例（特性 `tests-on-a-real-gpu`）不写进命令；不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
