# kb 索引

**中文** · [English](INDEX.en.md) · [日本語](INDEX.ja.md)

**规则不进 kb，进 [CLAUDE.md](../../CLAUDE.md)。** kb 只放事实、决策、调研、实测数据。

| 文件 | 内容 |
|---|---|
| [decisions.md](decisions.md) | 决策索引：编号、简称、状态、进度、指向正文的链接 |
| [decisions/](decisions/) | 每个决策一个文件（`NN-简称.md`），正文与论证都在这里 |
| [decisions-history.md](decisions-history.md) | 全部决策的变更史 |
| [../../crates/singlefs-checker/invariants/README.md](../../crates/singlefs-checker/invariants/README.md) | 不变量清单，按类拆成同目录 9 份，README 放说明、状态与条数登记。checker 是它的可执行形式 |
| [prior-art.md](prior-art.md) | 他家方案调研，含来源与口径 |
| [pitfalls.md](pitfalls.md) | 避坑清单。每做一个设计决定回来对一遍 |
| [experiments.md](experiments.md) | 实验索引：编号、简称、状态、指向正文的链接 |
| [experiments/](experiments/) | 每个实验一个文件（`NN-简称.md`），测什么、判据、口径都在这里 |
| [experiments-history.md](experiments-history.md) | 全部实验的变更史 |
| [checks-owed.md](checks-owed.md) | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| [pending-rules.md](pending-rules.md) | 待议的规矩：做不成会红的检查、从共享 SOP 撤下来的条款，原文逐字留存，写明缺什么才能成规则 |
| [tooling.md](tooling.md) | 工具与环境事实：本地 LLM、Rust 工具链、QEMU harness 的现状与缺口 |
| [vm-harness.md](vm-harness.md) | 怎么把一个实验送进虚机在**真块设备**上跑：三个前置、卫生检查、虚机里多了哪条校验路径、屏蔽 CPU 特性那一档的已知答案闸、设备侧独立录制（blklogwrites）、附加根与盘镜像预分配、计时与块层计数的口径 |
| [verification-build.md](verification-build.md) | 三样验证手段（checker、事务层、崩溃点重放）怎么落地：各自消费哪些已定条款、被哪些未定项挡着、能从 research/ 抬走什么、第一版最小范围、当时待用户定案的问题（只剩 T1（常规层 挂钟预算） 一问没定）、崩溃放量的流程与用户定的两档 |
| [layout/](layout/) | 每个里程碑新写出哪些字节，一个里程碑一个文件：`01-first-txn.md` 新池新建文件（每段每字段指向一条决策分项，给宽度与取值；段序列登记表也在这里）、`02-second-txn.md` 覆盖写、释放、回退与复用新写出的形态、`03-third-txn.md` 删除新写出的形态 |
| [milestone/](milestone/) | 里程碑规划，每个里程碑一个文件（`NN-简称.md`）：每步写设想实现什么、预想的细节、验收标准、写出的字节、会碰到的决策点；按当时判断写，每步开工前回来改 |
| [feature-bits.md](feature-bits.md) | feature bit 记账表：incompat / compat_ro / compat 三张位图各 256 位，每分出去一位写一行；位号的唯一登记位在 D15（格式冻结政策） 已定项 4 |
| [freeze-layer-membership.md](freeze-layer-membership.md) | 冻结层归属登记表：一行一个结构，落 D15（格式冻结政策） 已定项 7 的哪一层、权威态还是派生态、退不退出冻结 |
| [term-renames.md](term-renames.md) | 术语改名对照表：旧名与新名的唯一权威对照，别处一律用新名 |
| [translation-glossary.md](translation-glossary.md) | 翻译术语表：中、英、日三语并排，仓根 README 与 `briefs/` 的三语版本照它用词 |

## 不是编号的记号

`doc-lint:not-numbers` 那一行里的记号与编号同形（大写字母 + 数字），
但它们是领域术语，不是本工程的编号，所以不要求带简称。新增同类术语加进那一行，别改门禁。
`F41` 是 CRC-32C 的 Castagnoli 多项式 `0x1EDC6F41` 末四位被切出来的那一段（D18（块里携带什么信息） 已定项 17 写死了这个参数），同理。

<!-- doc-lint:not-numbers L1 L2 L3 AES-256 AES256 SHA256 SHA-256 SHA512 CRC32 CRC32C CRC64 RAID5 RAID6 OCFS2 Z3 T10 M1 M2 M3 M4 M5 M6 M7 M8 M9 M12 M13 M25 M26 F41 -->

`M` 加数字（`doc-lint:not-numbers` 那一行里的 `M1`–`M9`、`M12`、`M13`、`M25`、`M26`）是**变异表里的行名**，不是本工程的编号。它们的登记位是
`scripts/mutation/tables/<bin>.tsv`——每张表的第一列，与那个二进制一一对应。
引用它们时要带上所属的二进制（例如「`e39_back_chain` 的 `M8_退回第一版哈希`」），
因为**同一个 `M8` 在不同表里指不同的东西**。
