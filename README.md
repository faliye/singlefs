# singlefs

**一个从零设计的写时复制（COW）文件系统，Rust 实现。**

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

## 系统配置

挂载从每块盘的固定位置读起，那里是**系统配置**——一块 481 字节的结构，住在偏移 0 与 4096 两个 4096 字节的槽里，**每盘一份、两槽轮换**，两块盘合起来全池四份。它是整条链的起点：系统配置 → 根环 → 根记录 → 树表 → 各棵索引树 → 数据单元。

它也是盘上**唯一救不回来**的那一小块。别的东西都能从单元自描述扫回来（每个单元自带「我是谁、我属于谁、我是第几代」），而 journal 环在哪、多长、单元多大这些几何，丢了没有第二个地方写着。

内容是**池级**的，每盘存一份副本、两槽同内容；唯一的盘级字段是「本盘设备号」——不是每块盘能配不同的值。

按**谁能设、改了要付什么**分四类：

| 类 | 字节 | 谁能设 | 改了付什么 |
|---|---|---|---|
| **系统不可变配置** | 389 | mkfs 时由用户设：fsid、journal 环长、单元大小、落点粒度、设备数、feature bits、加密参数 | 重建文件系统 |
| **系统可变配置** | 4 | mkfs 时设，今天只有节点大小一项 | 重建索引，**不丢数据**——索引是派生态，能从单元扫回来 |
| **系统运行配置** | 36 | **每次挂载都能重设，改了回写盘上**：checkpoint 周期 `T_time`、脏数据阈值 `T_dirty`、整理三水位 | 无 |
| **系统运行量** | 52 | **用户从来不能设**，文件系统自己维护：槽世代号、整槽校验和、journal tail、实例代号 | 不适用 |

四类只差在两根轴上：**用户能不能设**，以及**值什么时候变**。前三类是配置，值只在有人改它的时候才变；第四类是文件系统写给自己看的量，每次发布都在动——所以别按名字把它当常量缓存起来。

字段逐行的清单在 [`.claude/kb/decisions/22-单元原子性怎么合成.md`](.claude/kb/decisions/22-单元原子性怎么合成.md) 已定项 9 的字段表。

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
| **QEMU / KVM** | 真实负载 + 崩溃注入下的端到端行为，是准入的最终判据 |
| **herd7 / LKMM** | 并发路径的内存序——无锁结构、屏障、跨 CPU 可见性 |
| 崩溃点重放 | 屏障切段、段内任意一组写持久，枚举出的每个崩溃状态都生成镜像、跑恢复 + checker（第一个事务的全量在门禁 54 号） |
| 模型对拍 | 功能正确性：随机操作序列与内存里的理想模型比对。由门禁 74 号跑：随机历史五段取样点，每一步拿实现的结局与只住内存的理想模型比 |

三条硬要求：

1. **改了 `crates/*/src/` 就必须带测试。** 没有例外，没有「下个 patch 补上」。
2. **新增的测试必须先证明它会红**——把被测代码改坏、确认测试失败、再改回来，
   并在 commit message 里写明怎么验的。不能证明会红的测试等于没写。
3. **未实现的验证不许假装通过。** 门禁会显式列出还没接进来的阶段；
   绿色只代表已实现的部分过了，不代表验证充分。

提交前跑门禁：

```bash
bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 全量与 QEMU 真设备，要十几分钟）

cargo test --workspace                    # 平时的单测；层 0 全量标 ignored，这里只跑缩小版
bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 崩溃点重放全量（release）
bash .claude/gate.d/55-qemu-first-transaction.sh  # 单跑 QEMU 两块 virtio 盘上的第一个事务
bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
bash research/scripts/vm-bench.sh --selftest  # 单跑虚机装置自检（装置归项目）
```

`lkmm.sh` 要 `opam install herdtools7`，并用 `SINGLEFS_KERNEL_TREE=` 指一棵带
`tools/memory-model` 的 Linux 源码树。虚机装置 `research/scripts/vm-bench.sh` 要可读的内核镜像，
找不到会给出办法而**不会静默降级到软件模拟**。门禁 55 号要 `qemu-system-x86_64` 与 KVM，
前置条件见 [`.claude/kb/vm-harness.md`](.claude/kb/vm-harness.md)「三个前置」。

**门禁脚本与规则由 [singlefs-ai-sop](https://github.com/faliye/singlefs-ai-sop) 统一分发**，
所有参与者跑的是同一套——判据一致，你才知道自己该验到什么程度。

## 开工

规矩与门禁在独立仓 [singlefs-ai-sop](https://github.com/faliye/singlefs-ai-sop)，所有参与者共用同一份：

```bash
git clone https://github.com/faliye/singlefs.git
cd singlefs
git clone https://github.com/faliye/singlefs-ai-sop.git .claude/singlefs-ai-sop
bash .claude/singlefs-ai-sop/install.sh
bash .claude/scripts/gate.sh
```

[CLAUDE.md](CLAUDE.md) 用 `@` 引用 `.claude/singlefs-ai-sop/rules/` 里的共享规则——
**改规则要改上游仓**，不许在本项目就地改，否则一致性当场失效。

## 目录

| 路径 | 内容 |
|---|---|
| [`crates/singlefs-format`](crates/singlefs-format) | 格式常量：每个宽度的值来自 kb 的决策分项，没定的带占位标记 |
| [`crates/singlefs-core`](crates/singlefs-core) | mkfs、分配器、事务层（封闭的提交步骤枚举）、恢复、O_DIRECT 块设备后端 |
| [`crates/singlefs-harness`](crates/singlefs-harness) | 写请求录制器、层 0 崩溃状态枚举、设备侧日志核对，以及里程碑各步的验收用例 |
| [`crates/singlefs-checker`](crates/singlefs-checker) | checker：与实现只共享格式常量，解析、校验、遍历各写一份 |
| [`.claude/kb/`](.claude/kb/) | 设计决策与变更史、不变量清单、实验索引与正文、欠账表、第一个事务的字节表、里程碑规划、验证手段与虚机装置怎么落地、他家方案调研、避坑清单 |
| [`.claude/scripts/`](.claude/scripts/) | 门禁包装（逻辑在 singlefs-ai-sop） |
| [`.claude/gate.d/`](.claude/gate.d/) | 项目本地的门禁阶段 |
| [`research/`](research/) | 实验装置、留存产物与复跑脚本 |
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
