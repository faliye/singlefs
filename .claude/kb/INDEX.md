# kb 索引

**规则不进 kb，进 [CLAUDE.md](../../CLAUDE.md)。** kb 只放事实、决策、调研、实测数据。

| 文件 | 内容 |
|---|---|
| [decisions.md](decisions.md) | 决策索引：编号、简称、状态、指向正文的链接 |
| [decisions/](decisions/) | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
| [decisions-history.md](decisions-history.md) | 全部决策的变更史 |
| [invariants.md](invariants.md) | 不变量清单。checker 是它的可执行形式 |
| [prior-art.md](prior-art.md) | 他家方案调研，含来源与口径 |
| [pitfalls.md](pitfalls.md) | 避坑清单。每做一个设计决定回来对一遍 |
| [experiments.md](experiments.md) | 实验索引：编号、简称、状态、指向正文的链接 |
| [experiments/](experiments/) | 每个实验一个文件（`NN-简称.md`），测什么、判据、口径都在这里 |
| [experiments-history.md](experiments-history.md) | 全部实验的变更史 |
| [checks-owed.md](checks-owed.md) | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| [tooling.md](tooling.md) | 工具与环境事实：本地 LLM、Rust 工具链、QEMU harness 的现状与缺口 |
| [vm-harness.md](vm-harness.md) | 怎么把一个实验送进虚机在**真块设备**上跑：三个前置、卫生检查、虚机里多了哪条校验路径 |
| [verification-build.md](verification-build.md) | 三样未实现的验证手段（checker、事务层、崩溃点重放）怎么落地：各自消费哪些已定条款、被哪些未定项挡着、能从 research/ 抬走什么、第一版最小范围、待用户定案的问题 |

## 不是编号的记号

`doc-lint:not-numbers` 那一行里的记号与编号同形（大写字母 + 数字），
但它们是领域术语，不是本工程的编号，所以不要求带简称。新增同类术语加进那一行，别改门禁。

<!-- doc-lint:not-numbers L1 L2 L3 AES-256 AES256 SHA256 SHA-256 SHA512 CRC32 CRC32C CRC64 RAID5 RAID6 OCFS2 Z3 T10 M1 M2 M3 M4 M5 M6 M7 M8 M9 -->

`M1`–`M9` 是**变异表里的行名**，不是本工程的编号。它们的登记位是
`research/mutations/<bin>.tsv`——每张表的第一列，与那个二进制一一对应。
引用它们时要带上所属的二进制（例如「`e39_back_chain` 的 `M8_退回第一版哈希`」），
因为**同一个 `M8` 在不同表里指不同的东西**。
