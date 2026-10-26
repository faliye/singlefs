# singlefs

**中文** · [English](README.en.md) · [日本語](README.ja.md)

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

几个促使 singlefs 从格式与架构着手的设计问题，以及已选的方向：这些是设计目标，各项实现范围见「当前状态」。

| 痛点 | 根因 | 本项目的选择 |
|---|---|---|
| ENOSPC 荒谬（有空间却写不进、删文件也报没空间） | 两级分配 + chunk 分类僵化 | 不分 data/metadata，全局统一分配器 |
| RAID5/6 write hole | 定宽条带需读改写 | 条带宽度可变，永不读改写 |
| 配额记账又慢又错 | 记账是事后遍历，还要走反向索引 | 记账是事务的副产品，提交时增量维护 |
| 反向索引成了复杂度黑洞 | 它是后来长出来的，不是设计的 | day-1 设计进格式 |
| 修复工具没人敢用 | 可重建性不是设计目标 | 元数据块自描述 + checker 与格式同步演进 |

决策与依据在 [`.claude/kb/decisions.md`](.claude/kb/decisions.md)，避坑清单在 [`.claude/kb/pitfalls.md`](.claude/kb/pitfalls.md)。

### 容量与开销数据

已有实验包含实测与算术模型，数值只适用于各自的设备、负载与格式口径。它们不是当前完整文件系统的吞吐、延迟或通用性能承诺；完整对照与历史版本限制见性能表。

- 容量上限：E152（按里程碑对比六家文件系统的文件性能）的 16 GiB/盘配置中，单元区占单盘容量的 95.2%；两盘镜像后的池容量上限是两盘总量的 47.6%。树与记账还要继续占空间，实际可用量低于这个上限；这不表示当前支持单盘 mkfs。对照结果见 [`scripts/experiment-apparatus/perf-by-milestone.md`](scripts/experiment-apparatus/perf-by-milestone.md)。
- 小文件：数据单元占盘固定 32 KiB，当前扣除 105 字节头和 29 字节预留后，净荷最多 32634 字节；未装满的部分补零，每个单元独立占空间（E66（补齐之后小文件怎么落）；实现见[数据单元构造](crates/singlefs-core/src/unit.rs)）。 D27（小数据打包容器）已定但尚未实现；E120（档表取等比时公比怎么定）的 11.1% 是选定档表的补齐浪费上界，不是当前实现或完整容器开销的实测结果。
- 分配记录树：32 KiB 落点、90% 填充的算术模型约占容量的 0.055%（E74（分配记录的条目数与整理开销））；只计这棵树，不含其他索引、journal、快照与保留空间。

## 当前状态

当前里程碑是「共享与回收」（[里程碑四](.claude/kb/milestone/04-fourth-txn.md)）。工程已有可读写的双盘文件系统库原型，核心机制存在不等于里程碑全部验收完成；前一阶段的移交见[里程碑三](.claude/kb/milestone/03-third-txn.md)。

| 能力 | 已有实现 | 当前边界 |
|---|---|---|
| 格式化与挂载 | 两盘 mkfs、可写挂载、关闭后重挂、冷启动读回 | mkfs 当前要求恰好两盘；不表示任意盘数、RAID5/6 或热加盘已可用 |
| 文件与内容读写 | 一次发布可建多个文件；按 inode 号写入、按字节偏移读回 | 没有目录；覆盖写和顺序写均从头替换整份内容，没有任意偏移部分写入口 |
| 删除与复用 | 先发布删除意图，再分批回收；越过回退下界后复用落点 | 首文件删完后才新建的 inode 不能写数据；删除前已存在的其他文件可继续写 |
| 截断与变长 | 缩短先发意图、分批回收；变长与等长请求各一次发布 | 真正改变大小仅支持 inode 1，其他活 inode 只接受等长请求 |
| 只读快照 | 创建、销毁；快照引用的旧块进入 deadlist | 同时最多 10 个；现成快照读取接口只读首文件；没有可写快照或克隆 |
| 回退与恢复 | 挂载中的管理员回退、日志重放、恢复后续做意图 | 由当前支持的库调用与测试负载验证，尚无完整 POSIX 使用路径 |
| 事务发布 | 用户变更同步发布，单次请求内可包含多个单元事务 | 时间/脏字节自动 checkpoint、跨请求攒批和挂载阈值设置尚未接通 |
| 整理与回收 | 意图、区域扫描、搬迁和发布后 discard 的内部路径已有 | 默认产品路径因批大小未定，在签意图前停止；测试开关能推进不等于自动后台 GC 可用 |
| 索引扫描重建 | checker 可扫描得到中央映射、extent 和 inode 的内存表并比对 | 尚无全树重建并写回磁盘的通用修复工具；livelist 与稀疏旁表仍无条目写者 |

尚未提供系统挂载入口（FUSE / 内核模块）、目录与 POSIX 接口、加密、压缩和小对象打包。GPU 判器用于测试；产品 GPU GC 仍未实现。设计表里的变宽条带、自动整理和可重建性是目标，不能从这些目标推导当前产品能力。

**别拿它存任何数据。** 磁盘格式仍是软的，第一个外部用户出现之前随时可能推倒重来（[格式演进规则](.claude/rules/format-evolution.md)）。后续代码工作及验收条件登记在里程碑四末尾的「待优化项」。

库调用的实际用法可从[多 inode 内容写测试](crates/singlefs-harness/tests/writes_by_inode_number_of_several_files.rs)与[截断意图及批次测试](crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs)进入；这些是开发示例，不是面向日常使用的文件系统命令。

### 验证机制与证据范围

验证表列出已有机制和入口，不表示当前工作区已经通过全部门禁：

| 验证 | 判什么 | 门禁 |
|---|---|---|
| 崩溃点重放 | 写请求流按每块盘的屏障切段，枚举段内每一种落盘状态，逐个生成镜像、跑恢复加池级 checker | checker-tier-crash-point-replay；全量只在用户要求时跑 |
| 崩溃放量 | 同一套枚举放到几十亿态：流先录进判定库，再从库读回判，判器在 GPU 上，两台机器分片 | `scripts/crash/crash-amplification.sh`，用户要求时跑 |
| 池级 checker | 代码清单 `IMPLEMENTED_INVARIANTS` 有 108 项；按镜像及前置条件判适用范围，数量不等于每张镜像上完整判过 108 项 | 被上两样调用 |
| 模型对拍 | 随机操作序列与只住内存的理想模型逐步比 | harness-model-differential-and-scenarios |
| QEMU 真设备 | 两块 virtio 盘上跑固定负载，设备侧录下的写与 FLUSH 和程序录的流逐项比；没有崩溃注入 | checker-tier-qemu-device-streams |
| 内存序 | herd7 判 `litmus/` 下每条 Never，各配一条去掉屏障的对照 | checker-tier-lkmm |
| 形式化验证 | Verus 验恢复取前缀、择根、扫环的纯函数核，产品使用这份恢复核；另有整理计划/抽象模型证明，不覆盖 core 搬迁执行与崩溃，不是端到端文件系统证明 | checker-tier-formal-verification |
| 变异 | crates 变异表逐行改坏一处、点名的测试必须变红；广谱变异（cargo-mutants）每一条都要有结论、没有一条没红 | checker-tier-crates-mutation-replay、checker-tier-broad-mutation |

checker 的结果有「成立」「不成立」「不适用」「部分没判」四种；后两种不等于完整成立。零违例或 GPU 违例位为零，不能推出所有适用条款都已完整判过。

定向审计记录：2026-10-23 的工作区审计实际运行 6 个 harness 测试目标，60 条通过、0 失败，涉及多 inode 写、快照、截断字段/批次及整理内部行为。它没有复跑全工作区、GPU、Verus、QEMU 或全域崩溃验证，不能用作整体验收；命令、边界和日志见[代码先行审计报告](records/2026-10-23-代码先行的工程进度与决策核对.md)。

历史全量记录（不代表当前工作区的全量结论）：2026-10-16 到 2026-10-18 那一轮全量罩 14 条崩溃枚举用例，日志打了态数的 12 条合计 5,149,586,107 个崩溃状态（约 51.5 亿），每一态都有键相同的判定、违例 0；判定存在崩溃判定库里按函数单元键复用，判器在两台机器的 GPU 上跑。第 6 条没在最终那棵树上重跑，第 12 条的快档判法最后一天改过。2026-10-15 六道重型门禁过了一趟：herd7 与 QEMU（6 次虚机、123 项检查）绿，形式化与变异表跑的是轻型（变异 172 行里 2 行没红），广谱变异对当时那版代码没有结论；各项测试数据、口径与缺项见[2026-10-18 简报](briefs/2026-10-18.md)「测试数据汇总」。

**门禁全绿只构成模型层的崩溃一致性证据**，不是「验过了」。每一样跑到哪、哪几项记本次未跑、还没罩到什么，看 [最新简报（2026-10-25）](briefs/2026-10-25.md)（历史简报见 [`briefs/`](briefs/)）。

## 系统配置

挂载从每块盘偏移 0 与 4096 的两个 4096 字节槽读取**系统配置**，有效结构长 489 字节，两槽轮换。它提供系统配置 → 根环 → 根记录 → 树表 → 索引树 → 数据单元这条读取链所需的几何与身份；槽有冗余，不等于所有副本丢失后还能自动恢复配置。

字段在设计上分四类；格式里留了字段，不表示所有设置入口或重建机制已经实现：

| 格式类别 | 字节数 | 当前实现边界 |
|---|---:|---|
| 不可变配置 | 389 | 包含格式、布局和几何身份；仅部分值有 mkfs 参数，不能视为全部可调 |
| 可变配置 | 4 | 索引节点大小；当前按 16 KiB 常量实现，没有通用改大小并重建索引的产品入口 |
| 运行配置 | 36 | 时间、脏字节和整理水位字段使用默认值；尚无挂载设置通道和相应自动触发器 |
| 运行量 | 60 | 由文件系统维护的代号、水位等状态，随相应发布更新，用户不能直接设置 |

自描述单元支持扫描与核对；当前只读重建部分索引的内存表，不等于所有元数据都可由现成工具重建并写回。完整字段布局与兼容条件见[字节布局表](.claude/kb/layout/01-first-txn.md)。

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
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=多inode内容写测试; 范围=1 个测试目标; 为什么=验证README的库级使用入口' \
bash scripts/hosts/capped.sh default \
bash scripts/hosts/run-with-memory-cap.sh 12G \
bash scripts/hosts/run-build-directory.sh \
cargo test -p singlefs-harness --test writes_by_inode_number_of_several_files

bash scripts/crash/crash-point-replay.sh --list-items
```

示例只跑一个 harness 目标；12G 是该命令的内存上限示例，线程数由项目登记值提供，临时构建目录由脚本清理。`--list-items` 只列出已登记用例，不运行验证。其他目标按修改范围选择；提交前整轮门禁的资源前置、范围声明与运行方法见[提交流程](.claude/skills/gate/staged-gate.md)。

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
bash scripts/crash/crash-point-replay.sh --list-items
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
| [`research/`](research/) | 三方论证材料与判决（`prompts/`）、实验留存产物（`results/`），用于追溯设计和验证证据 |
| [`records/`](records/) | 建设过程 |
| [`briefs/`](briefs/) | 每次更新的简报，按日期一份：那一版能做什么、验到哪、还没罩到什么。每份简报有中文（默认）、英文（`.en.md`）、日语（`.ja.md`）三个版本 |

## 许可

双许可：[Apache-2.0](LICENSE-APACHE) 或 [MIT](LICENSE-MIT)，任选其一。

任何提交并被本项目采纳的贡献，按 Apache-2.0 的定义，都将按上述双许可授权，不附加任何额外条款。
