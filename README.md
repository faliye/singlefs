# singlefs

[中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

---
大硬盘是公司的，好睡眠是自己的。
---
**一个从零设计的写时复制（COW）文件系统，Rust 实现。**

## 为什么再造一个

我查看了很多fs系统的代码，我觉得他们都很优秀。但是过去的艰苦岁月，让它们不够浪费，太过耦合，在如此富裕的日子里竟然无法享福。

所以我决定我们来享福吧。允许singlefs丑陋，允许浪费存储，也允许拥有很多并行臃肿的实现分支。

我希望很多年以后，当有人形容singlefs的这些代码时，不要说它是一坨屎，而是说它是很多条屎。

实现上，修改一个地方可能意味着很多分支都需要一起修改。但这没关系——我们可以很轻松地再加上另一条不一样的——即使它臭得别有风味。

嗯，虽然我现在还在验证，但是俺寻思应该可以。
waaagh！

现有 COW 文件系统的几个痛点是**格式级的**，补丁补不掉，只能在设计时避开：

| 痛点 | 根因 | 本项目的选择 |
|---|---|---|
| ENOSPC 荒谬（有空间却写不进、删文件也报没空间） | 两级分配 + chunk 分类僵化 | 不分 data/metadata，全局统一分配器 |
| RAID5/6 write hole | 定宽条带需读改写 | 条带宽度可变，永不读改写 |
| 配额记账又慢又错 | 记账是事后遍历，还要走反向索引 | 记账是事务的副产品，提交时增量维护 |
| 反向索引成了复杂度黑洞 | 它是后来长出来的，不是设计的 | day-1 设计进格式 |
| 修复工具没人敢用 | 可重建性不是设计目标 | 元数据块自描述 + checker 与格式同步演进 |

决策与依据在 [`.claude/kb/decisions.md`](.claude/kb/decisions.md)，避坑清单在 [`.claude/kb/pitfalls.md`](.claude/kb/pitfalls.md)。

### 性能预估

数都来自跑过的实验，各按自己的口径读，不互相换算：

- 格式化之后能用多少：16 GiB 盘上单盘可用 95.2%，两盘镜像 47.6%；六家文件系统默认格式化在同一档（E152（按里程碑对比六家文件系统的文件性能），表在 [`scripts/experiments/perf-by-milestone.md`](scripts/experiments/perf-by-milestone.md)）。
- 小文件：不足 32 KiB 的数据一律补齐成一个 32 KiB 单元（E66（补齐之后小文件怎么落））。小对象打包由 D27（小数据打包容器）定了、还没实现，实现后补齐浪费最坏 11.1%（E120（档表取等比时公比怎么定））。
- 元数据自己占的：分配记录树在 32 KiB 落点、90% 填充下约占容量的 0.055%（E74（分配记录的条目数与整理开销），算术模型）。

## 当前状态

当前里程碑是「共享与回收」（[`.claude/kb/milestone/04-fourth-txn.md`](.claude/kb/milestone/04-fourth-txn.md)）；上一个「删除与多次 COW」（[`.claude/kb/milestone/03-third-txn.md`](.claude/kb/milestone/03-third-txn.md)）主体已完结，还开着的是段 4「优化」（二·6、六·4）。

能做的：两块盘上 mkfs；写一个文件并提交，顺序写满多个数据单元、按偏移读回；一次发布建多个文件（没有目录）；覆盖写、释放、抬过回退下界之后复用落点；关闭再挂载成第二个可写实例；挂着的时候做管理员回退；冷启动读回；按 inode 号覆盖写、顺序写与删除文件（删除先发布删除意图，再分批回收它的数据单元）；截断（改小时先发布意图、再分批回收 size 之外的数据单元，改大与等长各一次发布）；建与删只读快照。

不能做的：挂不到系统里（没有 FUSE、没有内核模块），没有目录与 POSIX 接口，没有可写快照、加密、压缩；整理（GC）的入口在，批大小条款还没定，产品路径在签整理意图之前就停下。**别拿它存任何数据。** 磁盘格式仍是软的，第一个外部用户出现之前随时可能推倒重来（[`.claude/rules/format-evolution.md`](.claude/rules/format-evolution.md)）。

验证到哪：

| 验证 | 判什么 | 门禁 |
|---|---|---|
| 崩溃点重放 | 写请求流按每块盘的屏障切段，枚举段内每一种落盘状态，逐个生成镜像、跑恢复加池级 checker | checker-tier-crash-point-replay；全量只在用户要求时跑 |
| 崩溃放量 | 同一套枚举放到几十亿态：流先录进判定库，再从库读回判，判器在 GPU 上，两台机器分片 | `scripts/crash/crash-amplification.sh`，用户要求时跑 |
| 池级 checker | 不变量清单（`crates/singlefs-checker/invariants/`）里「已实现」的 102 条，与代码里的清单同数 | 被上两样调用 |
| 模型对拍 | 随机操作序列与只住内存的理想模型逐步比 | harness-model-differential-and-scenarios |
| QEMU 真设备 | 两块 virtio 盘上跑固定负载，设备侧录下的写与 FLUSH 和程序录的流逐项比；没有崩溃注入 | checker-tier-qemu-device-streams |
| 内存序 | herd7 判 `litmus/` 下每条 Never，各配一条去掉屏障的对照 | checker-tier-lkmm |
| 形式化验证 | Verus 验恢复重放取 journal 前缀、择根、扫环的纯函数核（`crates/singlefs-replay-prefix`），编进产品的就是这一份 | checker-tier-formal-verification |
| 变异 | crates 变异表逐行改坏一处、点名的测试必须变红；广谱变异（cargo-mutants）每一条都要有结论、没有一条没红 | checker-tier-crates-mutation-replay、checker-tier-broad-mutation |

判了多少：2026-10-16 到 2026-10-18 那一轮全量罩 14 条崩溃枚举用例，日志打了态数的 12 条合计 5,149,586,107 个崩溃状态（约 51.5 亿），每一态都有键相同的判定、违例 0；判定存在崩溃判定库里按函数单元键复用，判器在两台机器的 GPU 上跑。第 6 条没在最终那棵树上重跑，第 12 条的快档判法最后一天改过。2026-10-15 六道重型门禁过了一趟：herd7 与 QEMU（6 次虚机、123 项检查）绿，形式化与变异表跑的是轻型（变异 172 行里 2 行没红），广谱变异对这一版代码还没有结论；各项测试的数汇总在 2026-10-18 那份简报「测试数据汇总」一节。

**门禁全绿只构成模型层的崩溃一致性证据**，不是「验过了」。每一样跑到哪、哪几项记本次未跑、还没罩到什么，看 [`briefs/`](briefs/) 最新一份。

## 系统配置

挂载从每块盘偏移 0 与 4096 两个 4096 字节的槽读起，那里是**系统配置**：一块 489 字节的结构，每盘一份、两槽轮换。它是整条链的起点（系统配置 → 根环 → 根记录 → 树表 → 各棵索引树 → 数据单元），也是盘上唯一救不回来的一小块：别的都能从自描述的单元扫回来，journal 环在哪、单元多大这些几何丢了没有第二处写着。

内容分四类：mkfs 时由用户设、改了要重建文件系统的不可变配置（389 字节）；改了只重建索引、不丢数据的可变配置（4 字节）；每次挂载可以重设的运行配置（36 字节）；文件系统自己维护、用户从来不能设的运行量（60 字节，每次发布都在动，别当常量缓存）。字段表在 [`.claude/kb/layout/01-first-txn.md`](.claude/kb/layout/01-first-txn.md)。

## 贡献者治理（Contributor Governance）

> **Implementation That Favors the AI, Review That Favors the Human（实现上 AI 友好，审核上人类友好）.**
>
> **Make every submitted patch review-worthy（让每一份提交都值得被 review）.**
>
> **Contribution throughput may be unbounded; acceptance throughput is evidence-bound（投稿吞吐可以无限，接收吞吐受证据约束）.**

门禁存在的目的不是把谁挡在外面，是把每一份提交抬到值得花人的时间去看那条线上：机械的部分交给脚本，人的注意力用在只有人能判的地方。

**准入判据是自动化验证。** 本项目不按来源区分提交者，只分一类：带着证据的提交，和不带的。证据判据对所有人是同一把尺子，发出去之前跑一遍门禁，就知道自己站在哪。

> 披露：本项目的代码、文档与门禁脚本由人与 AI 协作产出，并会长期保持这种方式。这是对我们自己做法的说明，不构成对提交者的任何分类。

## 贡献

三条硬要求：

1. **改了 `crates/*/src/` 就必须带测试。** 没有例外，没有「下个 patch 补上」。
2. **新增的测试必须先证明它会红**：把被测代码改坏、确认测试失败、再改回来，在 commit message 里写明怎么验的。不能证明会红的测试等于没写。
3. **未实现的验证不许假装通过。** 门禁显式列出还没接进来的阶段；绿只代表已实现的部分过了。

```bash
bash .claude/scripts/gate.sh                                  # 整轮门禁：共享阶段加 .claude/gate.d/ 的项目阶段
cargo test -p singlefs-harness                                # harness 档：单元与集成测试，改了代码随时跑
bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick   # 轻型崩溃点重放
```

checker 档（崩溃枚举用例、QEMU、herd7、变异表、形式化验证、广谱变异）默认只在提交时跑，命令带 `SINGLEFS_HEAVY_TESTS=commit`；崩溃点重放全量与崩溃放量全量只在用户要求时跑。哪些算重型、轻型与重型两档怎么分，见 [`.claude/rules/verification.md`](.claude/rules/verification.md)。

前置：herd7 要 `opam install herdtools7`，并用 `SINGLEFS_KERNEL_TREE=` 指一棵带 `tools/memory-model` 的 Linux 源码树；QEMU 那一道要 KVM 与一份打了 blklogwrites 补丁的 QEMU（`scripts/devices/build-patched-qemu.sh` 编好装进私有目录，再照 [`qemu.env.example`](qemu.env.example) 在仓根写 `qemu.env`），缺了就拒跑、不退回系统 QEMU。虚机装置的前置见 [`.claude/kb/vm-harness.md`](.claude/kb/vm-harness.md)。

### 多机与 GPU（可选）

照仓根模板 [`multi-host.env.example`](multi-host.env.example) 在本机写一份 `multi-host.env`（git 忽略）：两台分片、GPU 判器、显卡的先后与额度、清场命令都在里面，每个键的说明在模板里；写完 `bash scripts/crash/crash-point-replay-shard-configuration-check.sh` 判一次。凡用 GPU 先清场，跑完不复原（`scripts/crash/gpu-quiesce-run.sh`）。把一件结果是文件的活挪到第二台：`bash scripts/hosts/multi-host-run.sh --peer -- <命令>`。各脚本放哪见 [`scripts/README.md`](scripts/README.md)。

门禁脚本与规则由 singlefs-ai-sop 统一分发，所有参与者跑的是同一套。

## 开工

规矩与门禁在独立仓 singlefs-ai-sop，同一份规范有中文、英文、日文三个仓（[`singlefs-ai-sop-zh`](https://github.com/faliye/singlefs-ai-sop-zh)、[`-en`](https://github.com/faliye/singlefs-ai-sop-en)、`-ja`），对外以 `-en` 为准；本项目接的是 `-zh`。项目里那一份是**拷贝**，不带上游的 `.git` 与 `CLAUDE.md`：

```bash
git clone https://github.com/faliye/singlefs.git
git clone https://github.com/faliye/singlefs-ai-sop-zh.git
cd singlefs
rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/
diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop   # 没有输出才算拷全
bash .claude/singlefs-ai-sop/install.sh     # 刷版本戳 .singlefs-ai-sop-version
bash .claude/scripts/gate.sh
```

上游发版之后照同样的 `rsync` 再拷一遍、再跑 `install.sh`。**改规则要改上游仓**，不许在 `.claude/singlefs-ai-sop/` 里就地改。

## 目录

| 路径 | 内容 |
|---|---|
| [`crates/singlefs-format`](crates/singlefs-format) | 格式常量，每个宽度的值来自 kb 的决策分项 |
| [`crates/singlefs-core`](crates/singlefs-core) | mkfs、分配器、事务层、可写挂载与会话、恢复、挂载态的读、各棵索引树、块设备后端 |
| [`crates/singlefs-harness`](crates/singlefs-harness) | harness 档：单元与集成测试，加上录制器、内存池、理想模型、随机历史、故障注入这些脚手架 |
| [`crates/singlefs-checker-tier`](crates/singlefs-checker-tier) | checker 档：崩溃态枚举、判定库、崩溃放量、GPU 判器、真设备装置；默认只在提交时跑 |
| [`crates/singlefs-checker`](crates/singlefs-checker) | 池级 checker：判一个镜像的判决器库，与实现只共享格式常量 |
| [`crates/singlefs-replay-prefix`](crates/singlefs-replay-prefix) | 恢复重放取前缀、择根、扫环的纯函数核，Verus 验的就是这一份 |
| [`crates/singlefs-unit-graph`](crates/singlefs-unit-graph) | 函数单元图：Rust 与 WGSL 源码按条目切成单元、取去注释指纹，从入口收闭包、算复用键；只用 std |
| [`crates/singlefs-reference-filesystem`](crates/singlefs-reference-filesystem) | RefFS：语义级的参照文件系统，只在内存里按 POSIX 操作走 |
| [`crates/mutations.tsv`](crates/mutations.tsv) | crates 的变异表 |
| [`scripts/`](scripts/) | 反复要用的脚本，按用途分子目录，各放什么见它的 README |
| [`litmus/`](litmus/) | herd7 的 litmus 测试 |
| [`.claude/kb/`](.claude/kb/) | 决策与变更史、实验、欠账表、字节布局表、里程碑规划、调研与避坑 |
| [`.claude/rules/`](.claude/rules/) | 项目本地规则；共享规则在 `.claude/singlefs-ai-sop/rules/` |
| [`.claude/gate.d/`](.claude/gate.d/) | 项目本地的门禁阶段与它们的登记表 |
| [`.claude/agents/`](.claude/agents/)、[`.claude/hooks/`](.claude/hooks/)、[`.claude/skills/`](.claude/skills/) | subagent 定义、Claude Code 钩子、skill |
| `.claude/singlefs-ai-sop/` | 共享规范与门禁脚本的拷贝，git 忽略，「开工」那一步取来 |
| [`research/`](research/) | 三方论证的材料与判决（`prompts/`）、实验留存产物（`results/`），都可以整个删掉重取 |
| [`records/`](records/) | 建设过程 |
| [`briefs/`](briefs/) | 每次更新的简报，按日期一份：那一版能做什么、验到哪、还没罩到什么。每份简报有中文（默认）、英文（`.en.md`）、日语（`.ja.md`）三个版本 |

## 许可

双许可：[Apache-2.0](LICENSE-APACHE) 或 [MIT](LICENSE-MIT)，任选其一。

任何提交并被本项目采纳的贡献，按 Apache-2.0 的定义，都将按上述双许可授权，不附加任何额外条款。
