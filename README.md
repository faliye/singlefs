# singlefs

**一个从零设计的写时复制（COW）文件系统，Rust 实现。**

我查看了很多fs系统的代码，我觉得他们都很优秀。但是它们不够浪费，太过耦合。最初的设计影响来后来的拓展性。

所以我决定允许它丑陋，允许浪费存储，也允许拥有很多并行的实现分支。

我希望很多年以后，当有人形容这些代码时，不要说它是一坨屎，而是说它是很多条屎。

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

## 当前状态

**格式设计阶段。还没有磁盘格式，也还没有代码。**

先定 `decisions.md` 里的待定项，再动第一行实现——其中校验和位置与核心索引结构
决定「第一段代码长什么样」，不定就写会返工。

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
| 崩溃点重放 | 任意断电点能否恢复（`dm-log-writes` 截断 + 重放 + checker） |
| 模型对拍 | 功能正确性：随机操作序列与内存里的理想模型比对 |

三条硬要求：

1. **改了 `crates/*/src/` 就必须带测试。** 没有例外，没有「下个 patch 补上」。
2. **新增的测试必须先证明它会红**——把被测代码改坏、确认测试失败、再改回来，
   并在 commit message 里写明怎么验的。不能证明会红的测试等于没写。
3. **未实现的验证不许假装通过。** 门禁会显式列出还没接进来的阶段；
   绿色只代表已实现的部分过了，不代表验证充分。

提交前跑门禁：

```bash
bash .claude/scripts/gate.sh              # 规范版本 / 文档 / 有无测试 / 构建单测 / LKMM
GATE_QEMU=1 bash .claude/scripts/gate.sh  # 再加 QEMU harness 自检（要两次虚机启动）

bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
bash .claude/scripts/qemu.sh --selftest   # 单跑 QEMU harness 自检
```

`lkmm.sh` 要 `opam install herdtools7`，并用 `SINGLEFS_KERNEL_TREE=` 指一棵带
`tools/memory-model` 的 Linux 源码树。`qemu.sh` 要可读的内核镜像，
找不到会给出办法而**不会静默降级到软件模拟**。

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
| `.claude/kb/` | 设计决策、不变量清单、他家方案调研、避坑清单 |
| `.claude/scripts/` | 门禁包装（逻辑在 singlefs-ai-sop） |
| `records/` | 建设过程 |

## 许可

双许可：[Apache-2.0](LICENSE-APACHE) 或 [MIT](LICENSE-MIT)，任选其一。

选双许可是为了不挡路：Apache-2.0 带显式专利授权，MIT 极简且与 GPL 项目兼容。
这样 bootloader、initramfs、嵌入式固件这些 GPL 之外的场景也能直接用——
而这正是 GPL 实现进不去的地方。

除非你明确另行声明，任何你有意提交并被本项目采纳的贡献，
按 Apache-2.0 的定义，都将按上述双许可授权，不附加任何额外条款。
