# singlefs

**一个从零设计的写时复制（COW）文件系统，Rust 实现。**

---
大硬盘是公司的，好睡眠是自己的。
---

我查看了很多fs系统的代码，我觉得他们都很优秀。但是过去的艰苦岁月，让它们不够浪费，太过耦合，在如此富裕的日子里竟然无法享福。

所以我决定我们来享福吧。允许singlefs丑陋，允许浪费存储，也允许拥有很多并行臃肿的实现分支。

我希望很多年以后，当有人形容singlefs的这些代码时，不要说它是一坨屎，而是说它是很多条屎。

实现上，修改一个地方可能意味着很多分支都需要一起修改。但这没关系——我们可以很轻松地再加上另一条不一样的——即使它臭得别有风味。

嗯，虽然我现在还在验证，但是俺寻思应该可以。
waaagh！



## 为什么再造一个

现有 COW 文件系统的几个痛点是**格式级的**，补丁补不掉，只能在设计时避开：

| 痛点 | 根因 | 本项目的选择 |
|---|---|---|
| ENOSPC 荒谬（有空间却写不进、删文件也报没空间） | 两级分配 + chunk 分类僵化 | 不分 data/metadata，全局统一分配器 |
| RAID5/6 write hole | 定宽条带需读改写 | 条带宽度可变，永不读改写 |
| 配额记账又慢又错 | 记账是事后遍历，还要走反向索引 | 记账是事务的副产品，提交时增量维护 |
| 反向索引成了复杂度黑洞 | 它是后来长出来的，不是设计的 | day-1 设计进格式 |
| 修复工具没人敢用 | 可重建性不是设计目标 | 元数据块自描述 + checker 与格式同步演进 |

完整的决策记录与依据在 [`.claude/kb/decisions.md`](.claude/kb/decisions.md)，
避坑清单在 [`.claude/kb/pitfalls.md`](.claude/kb/pitfalls.md)。

### 性能预估

数都来自已经跑过的实验，各条按各自的口径读，不互相换算；实验之后实现变了、没重跑的，以那次实验的日期为准。

- **空间占用与浪费率**
  - 格式化之后能用多少：16 GiB 盘上，可用容量的上限是一块盘的 95.2%；两盘镜像之后是两块盘总量的 47.6%。六家默认格式化，单盘可用 90.6%（Bcachefs）到 98.1%（F2FS），镜像配置 46.3%（ext4 md raid1）到 49.2%（Btrfs raid1），和 singlefs 在同一档。来源：E152（按里程碑对比六家文件系统的文件性能）2026-09 的虚机正式跑，表在 [`scripts/experiments/perf-by-milestone.md`](scripts/experiments/perf-by-milestone.md)。
  - 小文件：不足 32 KiB 的数据一律补齐成一个 32 KiB 单元，32 KiB 及以下每一档实测都占 32 768 字节（E66（补齐之后小文件怎么落））。一个 3000 字节的文件因此占 32 768 字节，约 91% 是补齐（没单独量）。
  - 小对象打包之后：把 ≤ 4 KiB 的小对象搬进共享打包容器，档宽按公比 1.125 等比切（36 档），补齐浪费最坏 11.1%，与对象大小怎么分布无关；三种分布下实测 +8.26%（均匀）、+11.24%（对数均匀）、+1.91%（离散）。来源：E120（档表取等比时公比怎么定），纯算术模型。打包由 D27（小数据打包容器）定了，`crates/` 里还没有，所以今天的小文件仍按上一条算。
  - 元数据自己占的：分配记录树（每个已分配落点一条）在 32 KiB 落点、90% 填充下约占容量的 0.055%，16 TiB 盘上约 9 GiB；E97（记账与分配记录的条目编码）按两级扇出复核是 553 ppm。来源：E74（分配记录的条目数与整理开销），纯算术模型，条目宽按20 字节算。

## 当前状态

里程碑「覆盖写、释放、回退与复用」（[`.claude/kb/milestone/02-second-txn.md`](.claude/kb/milestone/02-second-txn.md)）的步 0 到步 7 与四条并行线都已落地，2026-09-29 判定结束；当前里程碑是「删除与多次 COW」（[`.claude/kb/milestone/03-third-txn.md`](.claude/kb/milestone/03-third-txn.md)），里程碑二没做完的（「验证到哪」里的已知红等）转在它的项五。

能做的：在两块盘上 mkfs（根环与 journal 环整环清零）；写一个文件并提交，一个文件可以顺序写满多个数据单元、挂着的时候按偏移读回；一次发布建多个文件（多个 inode，没有目录）；在同一个池里接着覆盖写、释放单元并在回退下界 F 抬过之后复用它的落点、关闭再挂载成第二个可写实例、挂着的时候做管理员回退（回退是一次普通的向前发布，不换实例、不抛弃任何根）；丢掉进程里的一切之后冷启动读回来；按 inode 号删除单个文件：先发布删除意图，再分批回收它的数据单元，直到删除队列清空（`crates/singlefs-core/src/transaction.rs` 的 `publish_unlink_intent`、`publish_unlink_steps_until_the_queue_is_empty`）。

不能做的：挂不到系统里（没有 FUSE、没有内核模块），没有目录与 POSIX 接口（删除只能按 inode 号删），没有快照、加密、压缩。**别拿它存任何数据。** 磁盘格式仍是软的，第一个外部用户出现之前随时可能推倒重来（[`.claude/rules/format-evolution.md`](.claude/rules/format-evolution.md)）。

验证到哪：

| 验证 | 现状 |
|---|---|
| 崩溃点重放（门禁 checker-tier-layer0-replay） | 崩溃点重放的两条流：新池新建文件；覆盖写、释放、回退与复用的固定脚本（覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次）。另有 `.claude/gate.d/stage-inputs.tsv` 登记的 13 条崩溃枚举用例。门禁的快档跑两趟：不带特性一趟，再带 `tests/` 里 cfg 点到的特性一趟（feature-gated-tests 格，带 `gpu` 特性的那一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）。崩溃点重放这一套与崩溃放量已经合成一张合表 `.claude/gate.d/crash-point-replay-items.tsv`（13 项，一项一条流）、一个入口 `scripts/crash/crash-point-replay.sh`：轻型走这个入口；新入口接进提交那一趟之前，提交时照旧跑快档与全量（[`.claude/rules/verification.md`](.claude/rules/verification.md)「两档：轻型与重型」）。固定脚本那条流的崩溃点重放结果还没重核，暂不作数。**门禁全绿只构成新池新建文件在模型层的崩溃一致性证据** |
| 崩溃放量（`scripts/crash/crash-amplification.sh`，用户要求时跑） | 先录后核、按块复用、两台机器合并，判器整套在 GPU 上（与 CPU 判器逐状态对拍过）。2026-09-30 清场之后两台五张卡、展开上限 28 全枚举：固定脚本到 D、到 E 再卸载、多记录发布、树分裂、按位置寻址、抬 F、σ 七个分项全域合计 2 615 501 475 态（这一趟实判 2 563 727 926 态，到 E 那一项沿用到 D 那一段），0 红，一趟 7 小时 37 分（到 E 那一项 1 662 648 564 态、4.5 小时）。GPU 判器对不对只靠与 CPU 判器逐状态相同来证，对拍过的是展开上限 16 的 17 条流 525 365 态与 σ 的 65 548 态，更大的域没对拍过；逐项与经过见 [`briefs/2026-09-28.md`](briefs/2026-09-28.md)。这一格的数都是 2026-09-30 那一趟的；入口今天列 8 个分项（`--list-items`），多出的「多记录发布之后 unlink 第一个文件」不在那一趟里 |
| checker | kb 里记为已实现的不变量 90 条，数法见表后那条命令；代码里的清单 `IMPLEMENTED_INVARIANTS`（`crates/singlefs-checker/src/image.rs`）列 92 个，多出的 I-9.36、I-9.37 在 kb 里还没有行 |
| 模型对拍（门禁 harness-model-differential-and-scenarios） | 随机历史的快档加五个偏向某种历史的取样点，每一步拿实现的结局与只住内存的理想模型比 |
| QEMU 真设备（门禁 checker-tier-qemu-device-streams） | 两块 virtio 盘上跑新池新建文件、发布 B、第二个实例、发布 D 与抬 F，设备侧独立录下的写与 FLUSH 和程序自己录的流逐项比；只有真实负载，没有崩溃注入 |
| 内存序（门禁 checker-tier-lkmm） | herd7 判 `litmus/` 下每条 Never，每条都配一条去掉屏障的对照 |
| 形式化验证（门禁 checker-tier-formal-verification） | Verus 验 `crates/singlefs-replay-prefix`（恢复重放取 journal 前缀的纯函数核，编进产品、被恢复路径调用的就是这一份）：工具链钉版、证明、空真、必红世界四格，登记 56 个 verified（`.claude/gate.d/verus-pins.tsv`）；另两格判 E165（恢复重放有界穷举与变异杀伤界）的红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv`（360 行）钉全、逐行重放 |
| 广谱变异（门禁 checker-tier-broad-mutation） | cargo-mutants 生成全量变异，驱动 `scripts/mutation/broad-mutation.py` 逐条跑、可续跑、可点名；门禁判驱动库目录那一版就是这一次的暂存树、每一条都有结论、没有一条没红 |
| crates 变异表（门禁 checker-tier-crates-mutation-replay） | `crates/mutations.tsv` 3954 行（其中 5 行注释），每行改坏一处、点名一条必须变红的测试 |
| 轻型与重型 | 崩溃点重放、形式化验证、crates 变异表三样各分两档（用户 2026-10-10 定）：轻型三条命令带 `--quick`，跑核心部分加按改动点名的项、不剪枝，给实现员与合入后验证用；重型是提交时的总门闸，跑全部，红着不提交。轻型是重型的子集，保留到事务功能做全 |
| 测试规模（源码里数的 `#[test]`，不是跑出来的条数） | harness 档 182 个测试文件，全包 1768 个，其中 69 条标 harness 慢档用例；checker 档 47 个测试文件，全包 1133 个；另有 singlefs-core 248 个、池级 checker 32 个、RefFS 24 个、singlefs-format 7 个、singlefs-replay-prefix 7 个 |
| 已知红 | 里程碑二收尾留下的三样（checker 档快档 3 条红、实验装置 e158 与 e161 的内联测试 10 条红、`crates/mutations.tsv` 5 行锚点对不上源码），按记录已经清掉：删除线第 56 批合入后 checker 档快档 CPU 一趟 659 过、0 红，带 `gpu`、`verdict-store` 特性一趟 1020 过、0 红；e161 的内联测试 14 过，最近几次合入后验证都记 e158 95 过；那 5 行锚点已对上（`records/2026-10-01-删除与并发事务.md`、`records/2026-09-30-里程碑三开工第一轮.md`）。这些是记录里的数；2026-10-10 那一批提交绕过了门禁、没跑 checker 档，要等下一次提交时 checker 档实跑才算数。大档故障注入 2 条新发现（C600（吞写之后可写挂载被拒而模型说该成）：设备在起点段吞掉一次写之后，模型说可写挂载该成、实现拒绝）还开着，条款没定，归里程碑三五·6 |

数 checker 判了几条：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`。

## 系统配置

挂载从每块盘的固定位置读起，那里是**系统配置**——一块 489 字节的结构，住在偏移 0 与 4096 两个 4096 字节的槽里，**每盘一份、两槽轮换**，两块盘合起来全池四份。它是整条链的起点：系统配置 → 根环 → 根记录 → 树表 → 各棵索引树 → 数据单元。

它也是盘上**唯一救不回来**的那一小块。别的东西都能从单元自描述扫回来（每个单元自带「我是谁、我属于谁、我是第几代」），而 journal 环在哪、多长、单元多大这些几何，丢了没有第二个地方写着。

内容是**池级**的，每盘存一份副本、两槽同内容；唯一的盘级字段是「本盘设备号」——不是每块盘能配不同的值。

按**谁能设、改了要付什么**分四类：

| 类 | 字节 | 谁能设 | 改了付什么 |
|---|---|---|---|
| **系统不可变配置** | 389 | mkfs 时由用户设：fsid、journal 环长、单元大小、落点粒度、设备数、feature bits、加密参数 | 重建文件系统 |
| **系统可变配置** | 4 | mkfs 时设，今天只有节点大小一项 | 重建索引，**不丢数据**——索引是派生态，能从单元扫回来；第一版还没有重建索引这条路，所以今天实际改不了 |
| **系统运行配置** | 36 | **每次挂载都能重设，改了回写盘上**：checkpoint 周期 `T_time`、脏数据阈值 `T_dirty`、整理三水位。挂载时传值的通道与 `T_time`、`T_dirty` 的发布触发都还没做，今天按内置默认值写盘 | 无 |
| **系统运行量** | 60 | **用户从来不能设**，文件系统自己维护：槽世代号、整槽校验和、journal tail、实例代号、回退下界 F | 不适用 |

四类只差在两根轴上：**用户能不能设**，以及**值什么时候变**。前三类是配置，值只在有人改它的时候才变；第四类是文件系统写给自己看的量，每次发布都在动——所以别按名字把它当常量缓存起来。

字段逐行的清单是 [`.claude/kb/layout/01-first-txn.md`](.claude/kb/layout/01-first-txn.md) 里的「系统配置字段表」（46 行、489 字节），由 [`.claude/kb/decisions/22-单元原子性怎么合成.md`](.claude/kb/decisions/22-单元原子性怎么合成.md) 已定项 9 定为字段表；四类各多少字节在同一份决策的已定项 26。

## 贡献者治理（Contributor Governance）

> **实现上 AI 友好，审核上人类友好。**
>
> **Make every submitted patch review-worthy.**
>
> 让每一份提交都值得被 review。
>
> **Contribution throughput may be unbounded; acceptance throughput is evidence-bound.**
>
> 投稿吞吐可以无限，接收吞吐受证据约束。

门禁存在的目的不是把谁挡在外面，是**把每一份提交抬到值得花人的时间去看那条线上**。
机械的部分交给脚本，人的注意力才腾得出来用在只有人能判的地方。

**本项目的准入判据是自动化验证。** 每一个 patch 都应当经过严格测试，
**我们欢迎每一份认真测试、负责任的提交。**

本项目**不按来源区分提交者**，也不为任何一类单列规矩。
只有一类划分：**带着证据的提交，和不带的。**
按身份决定审查强度既不公平，也不管用——一个 patch 不会因为作者是谁而变好或变坏。

证据判据对所有人是同一把尺子，而且**你自己就能提前量**：
发出去之前跑一遍门禁，就知道自己站在哪。

> 披露：本项目的代码、文档与门禁脚本由人与 AI 协作产出，并会长期保持这种方式。
> 这是对我们自己做法的说明，不构成对提交者的任何分类。

## 贡献

**欢迎任何经过 QEMU、herd7、LKMM 充分验证的 request。**

| 工具 | 验什么 |
|---|---|
| **QEMU / KVM** | 真实负载 + 崩溃注入下的端到端行为，是准入的最终判据。今天门禁 checker-tier-qemu-device-streams 只接了真实负载与设备侧录制，崩溃注入还没接，所以「最终判据」仍列在门禁的未实现清单里 |
| **herd7 / LKMM** | 并发路径的内存序——无锁结构、屏障、跨 CPU 可见性 |
| 崩溃点重放 | 写请求流按每块盘自己的屏障切段，段内任意一组整写持久（原地覆写多一种「新旧都读不出」的撕裂态），枚举出的每个崩溃状态都生成镜像、跑恢复 + checker。由门禁 checker-tier-layer0-replay 跑：两条崩溃点重放流加登记的崩溃枚举用例，哪些结果作数见「当前状态」；统一入口 `scripts/crash/crash-point-replay.sh` 照合表跑，轻型带 `--quick` |
| 模型对拍 | 功能正确性：随机操作序列与内存里的理想模型比对。由门禁 harness-model-differential-and-scenarios 跑：随机历史的快档加五个偏向某种历史的取样点，一共六段，每段一格（段名以它文件头的格名表为准），每一步拿实现的结局与只住内存的理想模型比 |
| 形式化验证 | 恢复重放取 journal 前缀的那一段核（`crates/singlefs-replay-prefix`）用 Verus 证。由门禁 checker-tier-formal-verification 跑，连同 E165（恢复重放有界穷举与变异杀伤界）红例钉表的钉全与逐行重放 |
| 变异 | 测试抓不抓得到改坏。crates 变异表（门禁 checker-tier-crates-mutation-replay）逐行改坏一处、点名的测试必须变红；广谱变异（门禁 checker-tier-broad-mutation）用 cargo-mutants 生成全量变异，每一条都要有结论、没有一条没红 |

三条硬要求：

1. **改了 `crates/*/src/` 就必须带测试。** 没有例外，没有「下个 patch 补上」。
2. **新增的测试必须先证明它会红**——把被测代码改坏、确认测试失败、再改回来，
   并在 commit message 里写明怎么验的。不能证明会红的测试等于没写。
3. **未实现的验证不许假装通过。** 门禁会显式列出还没接进来的阶段；
   绿色只代表已实现的部分过了，不代表验证充分。

提交前跑门禁：

```bash
bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含 checker 档快档两趟与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑、Verus 形式化验证、广谱变异的判定；崩溃点重放全量与崩溃放量全量不在里面）

cargo test -p singlefs-harness            # harness 档：单元与集成测试，改了代码随时跑（.claude/rules/verification.md）
SINGLEFS_HEAVY_TESTS=user-request cargo test --release -p singlefs-checker-tier --lib --tests   # checker 档快档不带特性那一趟：崩溃枚举用例住这个包的 tests/，全量那几条标 ignored；默认只在提交时由门禁 checker-tier-layer0-replay 跑，门禁另跑一趟带 tests/ 里 cfg 点到的特性（带 gpu 的经 gpu-quiesce-run.sh 起）
bash .claude/gate.d/checker-tier-layer0-replay.sh   # 单跑 checker-tier-layer0-replay 的快档（checker 档包不标 ignored 的用例），再逐条核登记的崩溃枚举用例各自那一格全绿标记，不作数的报「本次未跑」
bash <worktree>/.claude/gate.d/checker-tier-layer0-replay.sh --full <worktree>  # 崩溃点重放全量（release）：暂存之后在 HEAD + 暂存区的 worktree 里用那棵树里的 checker-tier-layer0-replay 跑（建法见快档判红时的出路句），逐条崩溃枚举用例按它自己的输入指纹复用或重跑，判绿写那一条的全绿标记
bash .claude/gate.d/checker-tier-qemu-device-streams.sh  # 单跑 QEMU 两块 virtio 盘上的新池新建文件、发布 B、第二个实例、发布 D 与抬 F
bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
bash scripts/devices/vm-bench.sh --selftest  # 单跑虚机装置自检（装置归项目）
bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick   # 轻型崩溃点重放：核心部分，再带 --item <项> 加点名的项（--list-items 列）
bash scripts/hosts/run-with-memory-cap.sh <上限> bash .claude/gate.d/checker-tier-formal-verification.sh --quick <根>   # 轻型形式化验证
bash scripts/hosts/run-with-memory-cap.sh <上限> bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根>   # 轻型 crates 变异表
```

提交时跑六道重门禁：checker-tier-layer0-replay（快档）、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation，命令带 `SINGLEFS_HEAVY_TESTS=commit`；六道都登记在 `.claude/gate.d/stage-inputs.tsv`，输入没变就复用上一次的判定。崩溃点重放全量（`--full`）与崩溃放量全量（`scripts/crash/crash-amplification.sh`）只在用户要求时跑，前缀是 `SINGLEFS_HEAVY_TESTS=user-request`。崩溃点重放、形式化验证、crates 变异表三样另分轻型与重型：轻型就是上面带 `--quick` 的三条，守核心部分加按改动点名的项；重型是提交时的总门闸，红了修好或查清才提交（[`.claude/rules/verification.md`](.claude/rules/verification.md)「两档：轻型与重型」）。哪条命令算重型由 `.claude/hooks/lib_heavy_tests.py` 判。

`lkmm.sh` 要 `opam install herdtools7`，并用 `SINGLEFS_KERNEL_TREE=` 指一棵带
`tools/memory-model` 的 Linux 源码树。虚机装置 `scripts/devices/vm-bench.sh` 要可读的内核镜像，
找不到会给出办法而**不会静默降级到软件模拟**。门禁 checker-tier-qemu-device-streams 要 KVM 与一份打了 blklogwrites 补丁的 QEMU（经 `vm-bench.sh` 的 blklogwrites 一支起虚机）：`scripts/devices/build-patched-qemu.sh` 把它编好装进私有目录，再照仓根模板 `qemu.env.example` 在仓根写一份 `qemu.env`（git 忽略），一行 `SINGLEFS_QEMU_HOME=<绝对路径>`；缺了就拒跑，不退回系统 QEMU。
前置条件见 [`.claude/kb/vm-harness.md`](.claude/kb/vm-harness.md)「三个前置」。

### 崩溃点重放全量分到两台机器上跑（可选）

默认不分片：没有下面这份配置、配置判不过，或者配置里的双机开关 `ENABLE_ACROSS_MACHINES` 不是 1，门禁 checker-tier-layer0-replay 的 `--full` 就在本机单机跑。这份配置是多机配置：崩溃点重放分片、门禁 checker-tier-crates-mutation-replay 的双机分片与通用跨机脚本（「把一件活挪到第二台跑（可选）」那一节）共用一份。

要分片，就在本机写一份配置。路径是 `${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}`，已被 git 忽略、不进仓；从仓根的模板 `multi-host.env.example` 拷一份改。一行一个 `KEY=值`，值不做 shell 展开。前八个键都要写（不清场、不手动复原的那一对写成空串），后面的开关、显卡与机器清单可以不写，开关没写算关：

| 键 | 写什么 |
|---|---|
| `PEER_SSH_HOST` | 第二台的 ssh 别名，要能免密登录（`ssh -o BatchMode=yes <别名> true` 退 0） |
| `PEER_REPOSITORY_DIRECTORY` | 第二台上的专用目录（绝对路径）：每一趟在 `runs/` 下放树与编译目录、跑完删；两片的进度文件与账本在 `progress/<输入指纹>/` |
| `PEER_CARGO_BIN_DIRECTORY` | 第二台的 `~/.cargo/bin`（绝对路径；非登录 shell 的 PATH 里没有它） |
| `PEER_MEMORY_CAP` | 第二台上那一份活的内存上限（写法照 `scripts/hosts/run-with-memory-cap.sh`：正整数加 K / M / G / T），不大于第二台的内存减去它上面别的服务的占用 |
| `QUIESCE_STOP_COMMAND` | 开跑前在本机跑的清场命令，空串表示不清场 |
| `QUIESCE_STOPPED_CHECK_COMMAND` | 清场之后回读，退 0 才算清完；写了 STOP 就要写它 |
| `QUIESCE_START_COMMAND` | 手动复原用的命令（私有复原脚本），空串表示没有。清场之后各脚本都不复原：跑完、跑红、被打断都不复原，要复原由用户手动跑这一条 |
| `QUIESCE_STARTED_CHECK_COMMAND` | 手动复原之后在本机回读，退 0 才算复原了；写了 START 就要写它 |
| `ENABLE_ACROSS_MACHINES` | 双机开关：写 1 才开，没写或写 0 算关，写别的值判法报错；崩溃点重放分片、checker-tier-crates-mutation-replay 的双机分片、跨机脚本的 `--peer` 都要它是 1 |
| `ENABLE_GPU` | GPU 开关：写 1 才开，没写或写 0 算关，写别的值判法报错。开了就单元校验和、三段流第 ② 段的核对与判器三类计算全走 GPU，卡用不上就报错退 78、不退回 CPU；不走 GPU 的那几截一截一行登记在 `.claude/gate.d/gpu-exemptions.tsv` |
| `GPU_CARDS_BY_PRIORITY` | 平时（不清场）显卡的先后与每张卡的显存额度：空白隔开的若干项，每项 `<local\|peer>:<nvidia-smi 的序号>:<额度>`，同一台上写在前面的先装满。开了 GPU 而它与下一个键都没写，开跑前就报错退 78 |
| `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` | 清场之后显卡的先后与额度，写法同上；崩溃放量全量与崩溃点重放入口开跑就清场，整趟照这一份用卡 |
| `HOSTS` | 可选的机器清单：除本机外每一台的 ssh 别名，空格隔开；没写就只有 `PEER_SSH_HOST` 一台。按台查资源的 `scripts/hosts/host-resources.py` 与资源提醒钩子逐台查它；崩溃点重放分片、变异分片与 `multi-host-run.sh --peer` 仍只读 `PEER_*` |
| `HOST_<号>_REPOSITORY_DIRECTORY`、`HOST_<号>_CARGO_BIN_DIRECTORY`、`HOST_<号>_MEMORY_CAP` | 清单里第 `<号>` 台（从 2 起）的专用目录、`~/.cargo/bin` 与一条命令的内存上限，可选 |

写完先判一次：`bash scripts/crash/layer0-shard-configuration-check.sh [<仓根>]`：退 0 是双机开、能分片；退 3 是配置判得过但双机开关没开，照单机跑；退 1 是判不过。两个开关的定义只在这份判法里。环境变量 `SINGLEFS_LAYER0_SHARD`（`<i>/<n>` 或 `merge/<n>`）由驱动脚本 `scripts/crash/layer0-shard-run.sh` 自己设，一般不用手设。第二台的构建环境（`~/.cargo/config*`、`RUSTFLAGS` 这类）进输入指纹，与本机对不上时驱动脚本拒跑。

走分片的只有在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例行里登记了 `shard=across-machines` 的用例，今天是六条（`crash-case:layer0-first-stream`、`crash-case:layer0-second-stream`、`crash-case:layer0-multi-record-publish-stream`、`crash-case:layer0-unlink-after-the-multi-record-publish-stream`、`crash-case:layer0-tree-split-streams`、`crash-case:layer0-position-addressed-tree-streams`），别的用例照旧在本机单机跑。分片时本机跑第 0/2 片、第二台跑第 1/2 片，第二台的账本拷回本机后按 `merge/2` 并起来；merge 那一趟的输出照单机的判法判，判绿写同一格全绿标记。两片的进度文件各留在自己的进度目录里，被杀之后下一趟接着跑。

两台要装同一个 rustup 工具链：`rustc -Vv` 的前三行与 host 行、`cargo -V` 逐字相同，否则驱动脚本拒跑。本机要有 `rsync`、`ssh`，第二台要有 `python3` 与 `git`（它在树副本里算输入指纹）。

驱动脚本也能单独调：`bash scripts/crash/layer0-shard-run.sh <crash-case:键> <树根>`，树根是 HEAD + 暂存区那棵树（建法同 checker-tier-layer0-replay 的 `--full`）。它跑的是标了 ignore 的全量用例，和崩溃点重放全量一样只在提交时或用户要求时跑。`bash scripts/crash/layer0-shard-run.sh --selftest` 不碰第二台：本机起两个进程各跑一片，走同一条路（工具链与指纹比对、清场（不复原）、两片、merge、判、写标记）。

### 把一件活挪到第二台跑（可选）

本机的内存包装排队、或者 CPU 被别的长活占满时，结果是文件的确定性活（编译加单测、实验装置跑产物）可以挪到第二台跑：

```bash
bash scripts/hosts/multi-host-run.sh [--peer] [--memory-cap <上限>] [--label <标签>] [--fetch <树里的相对路径>]… -- <命令与参数…>
```

默认不跨机：不带 `--peer` 就在本机仓根经 `scripts/hosts/run-with-memory-cap.sh` 跑那条命令，不读多机配置。带 `--peer` 要多机配置判得过、而且 `ENABLE_ACROSS_MACHINES=1`，没开就拒、不偷偷退回本机。它比两台工具链（不同就拒），把 git 列得出的文件（不带 `.git`、编译目录与被 git 忽略的本地配置）拷到第二台 `runs/<标签>/`，在那边经那棵树里的内存包装跑（上限不大于 `PEER_MEMORY_CAP`），`--fetch` 点名的产物取回本机，跑完删掉第二台那一趟的目录；退出码就是那条命令的。输出里第二台的主机与目录一律换成「第二台」。重型测试的规矩照旧，不因为挪到第二台就能跑。`bash scripts/hosts/multi-host-run.sh --selftest` 不碰第二台（「第二台」是本机上的另一个目录）。

### 用 GPU 与按资源派活

- 凡起 GPU 都经 `bash scripts/crash/gpu-quiesce-run.sh -- <命令…>`：照多机配置清场并回读，再跑那条命令；跑完、跑红、被打断都不复原，要复原就手动跑 `QUIESCE_START_COMMAND` 那一条，再用 `QUIESCE_STARTED_CHECK_COMMAND` 回读到退 0。清场回读没过就不跑、退 78。崩溃放量入口、分项驱动与门禁 checker-tier-layer0-replay 的 feature-gated-tests 格自己经它起，不用在外面再套一层。
- `python3 scripts/hosts/host-resources.py [-- <命令…>]`：查多机配置里每一台此刻的核、占用、内存与卡；给了命令就带上它的历史，逐台判放不放得下。只读，不清场、不停进程、不拒命令。钩子 `.claude/hooks/host-resources-reminder.sh` 在会话里认出要起重活或 GPU 的命令时先替你跑它，把结论作为提醒交回，命令照常放行。
- `python3 scripts/hosts/host-dispatch.py [--gpu] -- <命令…>`：统一派发器，按各台此刻放不放得下这条命令预测要的量，定派到哪台、给多少内存上限；`--plan` 只查不派。

**门禁脚本与规则由 singlefs-ai-sop 统一分发**，
所有参与者跑的是同一套——判据一致，你才知道自己该验到什么程度。

## 开工

规矩与门禁在独立仓 singlefs-ai-sop，同一份规范有中文、英文、日文三个仓（[`singlefs-ai-sop-zh`](https://github.com/faliye/singlefs-ai-sop-zh)、[`-en`](https://github.com/faliye/singlefs-ai-sop-en)、`-ja`），对外以 `-en` 为准；本项目接的是 `-zh`。项目里那一份是**拷贝**，不带上游的 `.git` 与 `CLAUDE.md`（留着它会被当成目录级 CLAUDE.md 注入给读规则的 agent）：

```bash
git clone https://github.com/faliye/singlefs.git
git clone https://github.com/faliye/singlefs-ai-sop-zh.git
cd singlefs
rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/
diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop   # 没有输出才算拷全
bash .claude/singlefs-ai-sop/install.sh     # 刷版本戳 .singlefs-ai-sop-version
bash .claude/scripts/gate.sh
```

上游发版之后照同样的 `rsync` 再拷一遍、再跑 `install.sh`。[CLAUDE.md](CLAUDE.md) 用 `@` 引用 `.claude/singlefs-ai-sop/rules/` 里的共享规则——
**改规则要改上游仓**，不许在 `.claude/singlefs-ai-sop/` 里就地改：下次同步就没了，而且改它等于改所有用这套规范的项目。

## 目录

| 路径 | 内容 |
|---|---|
| [`crates/singlefs-format`](crates/singlefs-format) | 格式常量：每个宽度的值来自 kb 的决策分项，没定的带占位标记 |
| [`crates/singlefs-core`](crates/singlefs-core) | mkfs、分配器、事务层（封闭的提交步骤枚举）、写请求切分、可写挂载与挂着的会话（管理员回退、抬 F、准入）、恢复、挂载态的读、各棵索引树（inode 树、extent 树、分配记录树等）、按结构记写量、O_DIRECT 块设备后端 |
| [`crates/singlefs-harness`](crates/singlefs-harness) | harness 档：单元与集成测试，加上它们用的脚手架——录制流与崩溃镜像所在的内存池（`memory_pool`）、录制流切段（`segments`）、只住内存的理想模型与对拍、随机历史、故障注入、场景；改了代码随时跑 |
| [`crates/singlefs-checker-tier`](crates/singlefs-checker-tier) | checker 档：崩溃态枚举与记录核对器、断点续跑与双机分片、崩溃注入、坏盘输入、设备侧日志核对与真设备运行模式、崩溃放量（崩溃点身份与复用键、先录后核的流水线、判定存储、GPU 判器与核对内核）、实验与真设备装置二进制（`src/bin/`）；默认只在提交时跑（两档怎么分见 [`.claude/rules/verification.md`](.claude/rules/verification.md)） |
| [`crates/singlefs-checker`](crates/singlefs-checker) | 池级 checker：判一个镜像的判决器库，与实现只共享格式常量，解析、校验、遍历各写一份 |
| [`crates/singlefs-replay-prefix`](crates/singlefs-replay-prefix) | 恢复重放取 journal 前缀的纯函数核，只依赖 `vstd`，`singlefs-core` 依赖它；门禁 checker-tier-formal-verification 用 Verus 验的就是这一份 |
| [`crates/singlefs-reference-filesystem`](crates/singlefs-reference-filesystem) | RefFS：语义级的参照文件系统，只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里；依赖只有 `std`，被 harness 档与 checker 档调用 |
| [`crates/mutations.tsv`](crates/mutations.tsv) | crates 的变异表：每条改坏一处、点名一条必须变红的测试，门禁 checker-tier-crates-mutation-replay 复跑 |
| [`.claude/kb/`](.claude/kb/) | 设计决策与变更史、不变量清单、实验索引与正文、欠账表、每个里程碑写出的字节表（`layout/`）、里程碑规划（`milestone/`）、验证手段与虚机装置怎么落地、他家方案调研、避坑清单 |
| [`.claude/rules/`](.claude/rules/) | 项目本地规则：文件系统的设计纪律、格式演进、三方论证、实现流程等；共享规则在 `.claude/singlefs-ai-sop/rules/` |
| [`.claude/scripts/`](.claude/scripts/) | 门禁包装，多数只转发到 singlefs-ai-sop 的共享脚本；`lkmm.sh`（门禁 checker-tier-lkmm 的逻辑）、`fetch-deps.sh`（取测试依赖）、`gen-decision-items.py`（生成决策分项清单）、`preflight.sh` 与 `project_preflight.py`（在 git worktree 里也找得到规范副本的准入判定）是项目自己的 |
| [`.claude/gate.d/`](.claude/gate.d/) | 项目本地的门禁阶段；`stage-inputs.tsv` 登记每道阶段读哪些输入，以及崩溃枚举用例 |
| [`.claude/agents/`](.claude/agents/) | subagent 的定义；共用约束在 `.claude/agent-common.md`，主 agent 的职责与调度表在 `.claude/main-agent.md` |
| [`.claude/hooks/`](.claude/hooks/) | Claude Code 钩子：执行前拒绝重型测试、越界写、没超时的等待循环这几类写法，以及派发、续做、交回的闸；另有一道只提醒不拒的资源提醒（`host-resources-reminder.sh`，认出要起重活或 GPU 的命令时先查各台资源） |
| [`.claude/skills/`](.claude/skills/) | 三个 skill：跑验证套件（`crash-test`）、记决策（`decide`）、跑门禁（`gate`） |
| [`.claude/warnings/`](.claude/warnings/) | 对提议有异议、对方坚持时留的警告记录，按日期一份 |
| `.claude/singlefs-ai-sop/` | 共享规范与门禁脚本的拷贝，git 忽略，「开工」那一步取来 |
| [`research/`](research/) | 实验：装置（`e7-index-bench/`）、留存产物（`results/`）、实验的变异表（`mutations/`）、脚本（`scripts/`，含复跑、变异、虚机与崩溃点重放双机分片的驱动，崩溃点重放入口、崩溃放量、广谱变异驱动，GPU 清场与按资源派活）、三方论证的材料与各条腿的报告和判决（`prompts/`）、按里程碑的性能对比（`perf-by-milestone.md`） |
| [`.cargo/mutants.toml`](.cargo/mutants.toml) | 广谱变异（cargo-mutants）的配置 |
| [`multi-host.env.example`](multi-host.env.example) | 多机配置的模板（崩溃点重放双机分片、门禁 checker-tier-crates-mutation-replay 的双机分片、跨机脚本 `scripts/hosts/multi-host-run.sh` 共用；双机与 GPU 两个开关默认关）。显卡的先后与额度也写在这份里：平时一份 `GPU_CARDS_BY_PRIORITY`、清场之后一份 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`，开了 GPU 而两份都没写就报错退出；可选的机器清单 `HOSTS` 给按台查资源用。见「崩溃点重放全量分到两台机器上跑（可选）」「把一件活挪到第二台跑（可选）」「用 GPU 与按资源派活」 |
| [`qemu.env.example`](qemu.env.example) | 打了 blklogwrites 补丁的 QEMU 装在哪：照它在仓根写 `qemu.env`（git 忽略），门禁 checker-tier-qemu-device-streams 与 `scripts/devices/vm-bench.sh` 的 blklogwrites 一支读它 |
| [`litmus/`](litmus/) | herd7 的 litmus 测试，每条 Never 配一条去掉屏障的对照 |
| [`records/`](records/) | 建设过程 |
| [`briefs/`](briefs/) | 每次更新的简报，按日期一份：那一版能做什么、验到哪、还没罩到什么。`records/` 写过程，这里写现状 |

## 许可

双许可：[Apache-2.0](LICENSE-APACHE) 或 [MIT](LICENSE-MIT)，任选其一。

选双许可是为了不挡路：Apache-2.0 带显式专利授权，MIT 极简且与 GPL 项目兼容。
这样 bootloader、initramfs、嵌入式固件这些 GPL 之外的场景也能直接用——
而这正是 GPL 实现进不去的地方。

除非你明确另行声明，任何你有意提交并被本项目采纳的贡献，
按 Apache-2.0 的定义，都将按上述双许可授权，不附加任何额外条款。
