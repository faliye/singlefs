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

## 当前状态

能做的：在两块盘上 mkfs、写一个文件并提交；在同一个池里接着覆盖写、释放单元并在延迟之后重用它的落点、关闭再挂载成第二个可写实例、管理员回退；丢掉进程里的一切之后冷启动读回来。

不能做的：挂不到系统里（没有 FUSE、没有内核模块），没有目录与 POSIX 接口，没有快照、加密、压缩。**别拿它存任何数据。** 磁盘格式仍是软的，第一个外部用户出现之前随时可能推倒重来（[`.claude/rules/format-evolution.md`](.claude/rules/format-evolution.md)）。

验证到哪：

| 验证 | 现状 |
|---|---|
| 崩溃点重放（门禁 54 号） | 层 0 两条流：第一个事务；第二个事务的固定脚本（覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次）。另有 `.claude/gate.d/stage-inputs.tsv` 登记的崩溃枚举用例。固定脚本那条流的层 0 结果还没重核，暂不作数。**门禁全绿只构成第一个事务在模型层的崩溃一致性证据** |
| checker | 判 46 条不变量，数法见表后那条命令 |
| 模型对拍（门禁 74 号） | 随机历史的快档加五个偏向某种历史的取样点，每一步拿实现的结局与只住内存的理想模型比 |
| QEMU 真设备（门禁 55 号） | 两块 virtio 盘上跑第一个事务、发布 B、第二个实例、发布 D 与抬 F，设备侧独立录下的写与 FLUSH 和程序自己录的流逐项比；只有真实负载，没有崩溃注入 |
| 内存序（门禁 57 号） | herd7 判 `litmus/` 下每条 Never，每条都配一条去掉屏障的对照 |

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
| **QEMU / KVM** | 真实负载 + 崩溃注入下的端到端行为，是准入的最终判据。今天门禁 55 号只接了真实负载与设备侧录制，崩溃注入还没接，所以「最终判据」仍列在门禁的未实现清单里 |
| **herd7 / LKMM** | 并发路径的内存序——无锁结构、屏障、跨 CPU 可见性 |
| 崩溃点重放 | 写请求流按每块盘自己的屏障切段，段内任意一组整写持久（原地覆写多一种「新旧都读不出」的撕裂态），枚举出的每个崩溃状态都生成镜像、跑恢复 + checker。由门禁 54 号跑：两条层 0 流加登记的崩溃枚举用例，哪些结果作数见「当前状态」 |
| 模型对拍 | 功能正确性：随机操作序列与内存里的理想模型比对。由门禁 74 号跑：随机历史的快档加五个偏向某种历史的取样点，一共六段（段名以 74 号的 `SECTIONS` 为准），每一步拿实现的结局与只住内存的理想模型比 |

三条硬要求：

1. **改了 `crates/*/src/` 就必须带测试。** 没有例外，没有「下个 patch 补上」。
2. **新增的测试必须先证明它会红**——把被测代码改坏、确认测试失败、再改回来，
   并在 commit message 里写明怎么验的。不能证明会红的测试等于没写。
3. **未实现的验证不许假装通过。** 门禁会显式列出还没接进来的阶段；
   绿色只代表已实现的部分过了，不代表验证充分。

提交前跑门禁：

```bash
bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 快档与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑；层 0 全量不在里面）

cargo test --workspace                    # 平时的单测；登记的崩溃枚举用例（层 0 全量等）都标 ignored，这里只跑快档与缩小版
bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 快档（两条流里不标 ignored 的用例），再逐条核登记的崩溃枚举用例各自那一格全绿标记
bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>  # 层 0 全量（release）：暂存之后在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（建法见快档判红时的出路句），逐条崩溃枚举用例按它自己的输入指纹复用或重跑，判绿写那一条的全绿标记
bash .claude/gate.d/55-qemu-first-transaction.sh  # 单跑 QEMU 两块 virtio 盘上的第一个事务、发布 B、第二个实例、发布 D 与抬 F
bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
bash research/scripts/vm-bench.sh --selftest  # 单跑虚机装置自检（装置归项目）
```

`lkmm.sh` 要 `opam install herdtools7`，并用 `SINGLEFS_KERNEL_TREE=` 指一棵带
`tools/memory-model` 的 Linux 源码树。虚机装置 `research/scripts/vm-bench.sh` 要可读的内核镜像，
找不到会给出办法而**不会静默降级到软件模拟**。门禁 55 号要 `qemu-system-x86_64` 与 KVM，
前置条件见 [`.claude/kb/vm-harness.md`](.claude/kb/vm-harness.md)「三个前置」。

### 层 0 全量分到两台机器上跑（可选）

默认不分片：没有下面这份配置，或者配置判不过，门禁 54 号 `--full` 就在本机单机跑。

要分片，就在本机写一份配置。路径是 `${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}`，已被 git 忽略、不进仓；从仓根的模板 `layer0-shard.env.example` 拷一份改。一行一个 `KEY=值`，值不做 shell 展开，七个键都要写：

| 键 | 写什么 |
|---|---|
| `PEER_SSH_HOST` | 第二台的 ssh 别名，要能免密登录（`ssh -o BatchMode=yes <别名> true` 退 0） |
| `PEER_REPOSITORY_DIRECTORY` | 第二台上的专用目录（绝对路径）：每一趟在 `runs/` 下放树与编译目录、跑完删；两片的进度文件与账本在 `progress/<输入指纹>/` |
| `PEER_CARGO_BIN_DIRECTORY` | 第二台的 `~/.cargo/bin`（绝对路径；非登录 shell 的 PATH 里没有它） |
| `QUIESCE_STOP_COMMAND` | 开跑前在本机跑的清场命令，空串表示不清场 |
| `QUIESCE_STOPPED_CHECK_COMMAND` | 清场之后回读，退 0 才算清完；写了 STOP 就要写它 |
| `QUIESCE_START_COMMAND` | 跑完（跑红了也跑）在本机复原；写了 STOP 就要写它 |
| `QUIESCE_STARTED_CHECK_COMMAND` | 复原之后回读，退 0 才算复原了；写了 START 就要写它 |

写完先判一次：`bash research/scripts/layer0-shard-configuration-check.sh [<仓根>]`，退 0 就能分片。环境变量 `SINGLEFS_LAYER0_SHARD`（`<i>/<n>` 或 `merge/<n>`）由驱动脚本 `research/scripts/layer0-shard-run.sh` 自己设，一般不用手设。第二台的构建环境（`~/.cargo/config*`、`RUSTFLAGS` 这类）进输入指纹，与本机对不上时驱动脚本拒跑。

走分片的只有在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例行里登记了 `shard=across-machines` 的用例，今天是两条层 0 流（`crash-case:layer0-first-stream`、`crash-case:layer0-second-stream`），别的用例照旧在本机单机跑。分片时本机跑第 0/2 片、第二台跑第 1/2 片，第二台的账本拷回本机后按 `merge/2` 并起来；merge 那一趟的输出照单机的判法判，判绿写同一格全绿标记。两片的进度文件各留在自己的进度目录里，被杀之后下一趟接着跑。

两台要装同一个 rustup 工具链：`rustc -Vv` 的前三行与 host 行、`cargo -V` 逐字相同，否则驱动脚本拒跑。本机要有 `rsync`、`ssh`，第二台要有 `python3` 与 `git`（它在树副本里算输入指纹）。

驱动脚本也能单独调：`bash research/scripts/layer0-shard-run.sh <crash-case:键> <树根>`，树根是 HEAD + 暂存区那棵树（建法同 54 号 `--full`）。它跑的是标了 ignore 的全量用例，和层 0 全量一样只在提交时或用户要求时跑。`bash research/scripts/layer0-shard-run.sh --selftest` 不碰第二台：本机起两个进程各跑一片，走同一条路（工具链与指纹比对、清场与复原、两片、merge、判、写标记）。

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
| [`crates/singlefs-core`](crates/singlefs-core) | mkfs、分配器、事务层（封闭的提交步骤枚举）、可写挂载与挂着的会话（管理员回退、抬 F、准入）、恢复、挂载态的读、各棵索引树、O_DIRECT 块设备后端 |
| [`crates/singlefs-harness`](crates/singlefs-harness) | 写请求录制器、层 0 崩溃状态枚举（含断点续跑与双机分片）、设备侧日志核对、随机历史、崩溃注入与故障注入、坏盘输入、只住内存的理想模型，以及里程碑各步的验收用例 |
| [`crates/singlefs-checker`](crates/singlefs-checker) | checker：与实现只共享格式常量，解析、校验、遍历各写一份 |
| [`crates/mutations.tsv`](crates/mutations.tsv) | crates 的变异表：每条改坏一处、点名一条必须变红的测试，门禁 59 号复跑 |
| [`.claude/kb/`](.claude/kb/) | 设计决策与变更史、不变量清单、实验索引与正文、欠账表、每个里程碑写出的字节表（`layout/`）、里程碑规划（`milestone/`）、验证手段与虚机装置怎么落地、他家方案调研、避坑清单 |
| [`.claude/rules/`](.claude/rules/) | 项目本地规则：文件系统的设计纪律、格式演进、三方论证、实现流程等；共享规则在 `.claude/singlefs-ai-sop/rules/` |
| [`.claude/scripts/`](.claude/scripts/) | 门禁包装，多数只转发到 singlefs-ai-sop 的共享脚本；`lkmm.sh`（门禁 57 号的逻辑）、`fetch-deps.sh`（取测试依赖）、`gen-decision-items.py`（生成决策分项清单）是项目自己的 |
| [`.claude/gate.d/`](.claude/gate.d/) | 项目本地的门禁阶段；`stage-inputs.tsv` 登记每道阶段读哪些输入，以及崩溃枚举用例 |
| [`.claude/agents/`](.claude/agents/) | subagent 的定义；共用约束在 `.claude/agent-common.md`，主 agent 的职责与调度表在 `.claude/main-agent.md` |
| [`.claude/hooks/`](.claude/hooks/) | Claude Code 钩子：执行前拒绝重型测试、越界写、没超时的等待循环这几类写法，以及派发、续做、交回的闸 |
| [`.claude/skills/`](.claude/skills/) | 三个 skill：跑验证套件（`crash-test`）、记决策（`decide`）、跑门禁（`gate`） |
| [`.claude/warnings/`](.claude/warnings/) | 对提议有异议、对方坚持时留的警告记录，按日期一份 |
| `.claude/singlefs-ai-sop/` | 共享规范与门禁脚本的拷贝，git 忽略，「开工」那一步取来 |
| [`research/`](research/) | 实验：装置（`e7-index-bench/`）、留存产物（`results/`）、实验的变异表（`mutations/`）、脚本（`scripts/`，含复跑、变异、虚机与层 0 双机分片的驱动）、三方论证的材料与各条腿的报告和判决（`prompts/`）、按里程碑的性能对比（`perf-by-milestone.md`） |
| [`.cargo/mutants.toml`](.cargo/mutants.toml) | 广谱变异（cargo-mutants）的配置 |
| [`layer0-shard.env.example`](layer0-shard.env.example) | 层 0 双机分片本地配置的模板，见「层 0 全量分到两台机器上跑（可选）」 |
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
