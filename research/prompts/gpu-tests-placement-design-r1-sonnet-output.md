# 产品 GPU 包的测试与 E202 装置归位（第一轮）：正推腿报告

轮名 gpu-tests-placement-design-r1；腿 three-way-forward（Sonnet）；今天 2026-11-18（`TZ=Asia/Tokyo date +%F`）。方案按 `crates/` 今天的实现来谈。
标记：「量的」是我自己跑只读命令或草稿目录里的只读分析脚本得到的；「读码」是读源码得到的；「推的」没量过。

## 0. 读法、模型与复跑

- 分析脚本在模型目录 `research/prompts/gpu-tests-placement-design-r1-sonnet-model/tools/`（只读源码，纯 `python3` 标准库）：`modrefs.py`（按文件的 use 链）、`symclosure.py`（按条目的闭包，名字匹配，是上界）、`group_compositions.py`（验证包判器键表里落在要搬模块里的单元）、`classify_refs.py`（引用分类）、`seeds.json`（根）、`rerun.sh`（整份）。复跑命令（与文末「要主 agent 跑的命令」第 1 条同一条）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/gpu-tests-placement-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-forward-rerun-run /home/fy5090/code/singlefs/research/prompts/gpu-tests-placement-design-r1-sonnet-model`；包装脚本只删工作根下名字以 `-run` 收尾的那一个运行目录，别的一律不删、退 2。
- 模型文件的 sha256（`sha256sum tools/*` 在模型目录里跑出来的原样）：

```
d020fa4299b147d79d33d6908a101b1f7d892a4d51eda8257259661db35d7d7d  tools/classify_refs.py
0439a95ab9d78593b550d50c408b879c34a65deb3b0ce588ee591fe321d66960  tools/group_compositions.py
bd059a4d6906efac569f476004f3c0e4595182e97f606997805c485df01f8fa7  tools/modrefs.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
b6d17c6766e08b68b49ffa6112cd3ac58d3084873b459e622d616c06696c86cb  tools/rerun.sh
9a4a0a2de301891c0074f1addae4d93604290104b9c10d833a399873c0b106c5  tools/seeds.json
4851961b8f28045da211d1df540ccf9aa3d2508ce042c5975f77a6ef4ea4016c  tools/symclosure.py
```

- 我没跑 `rerun.sh`（子 agent 不交任务，仓里脚本也不许直接起）。报告里的闭包数是我在草稿目录里用同一份脚本的副本直接跑的只读分析，`rerun.sh` 里的四种闭包与引用分类要主 agent 复跑后才算判决能引的数；所有数在复跑前一律当「推的」。
- 要留意：`target/release/build/singlefs-verification-*/out/judging_units_keys.tsv` 是 2026-11-17 编出来的旧构建产物（我读的那份文件时间是 2026-11-17），不是此刻树编出来的；第 1 格第 5 节里的「哪几组键会变」要用此刻树重编后再核。

## 0.1 与背景材料「实现今天的样子」对不上的几处（读码，逐文件现数）

| 背景写的 | 我数到的 | 命令 |
|---|---|---|
| 直接 `use singlefs_verification` 的有 9 个测试目标与 E202 的 bin | 测试目标文件 8 个；另有三个共用模块直接用它（`common_pairing_input_through_the_library`、`common_history_pool_through_the_library`、`common_checksum_batches_over_the_handed_over_gpus`），再加 E202 的 bin | `grep -l singlefs_verification crates/singlefs-background-gpu-tests/tests/*.rs \| wc -l` 得 8；`grep -l singlefs_verification crates/singlefs-background-gpu-tests/tests/*/mod.rs` 列三个 |
| `common_discrete_gpus_of_this_host` 被 10 个测试目标与 E202 的 bin 共用 | 11 个测试目标 + E202 的 bin | `grep -l "^mod common_discrete_gpus_of_this_host" crates/singlefs-background-gpu-tests/tests/*.rs \| wc -l` 得 11 |
| 用到的「只有」`PairingJudged` 等十余样 | 实际 `use` 到的还有：`crash_identity::{judging_units_key, judging_units_key_of_the_group_in_the_table_text, JudgingUnits}`、`crash_library::{CrashLibrary, CrashLibraryClass, ItemName, PairingConclusionSource, RecordingKeyOfThisRun, StageOutcome, toolchain_of_this_process, recording_key_of_the_pairing_input_and_the_reference_units, pairing_input_and_reference_through_the_library_under_the_recording_key}`、`gpu_card_configuration::CardLeaseChannelOfTheCaller`；测试里还调这些类型上的方法（`CrashLibrary::open`、`RunStage::of_this_process().judges()`、`DisposableLibrary::new` 与 `make_it_the_library_of_this_process`、`CardLeaseChannelOfTheCaller::read_from_the_environment_now` 与 `cards_leased_by_this_process`） | `crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:16-32`；`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/cards_of_the_command/tests.rs:191`、`:14060`、`:14070` |
| 全仓引包名 187 个文件 1261 处 | 此刻 191 个文件 1274 处；多出的 4 个文件 13 处正是这一轮自己的材料（调度记录 3 处，`_gpu-tests-placement-design-r1-` 的正文、背景、附录共 10 处） | 第 6 格 |
| `common_discrete_gpus_of_this_host` 第 542 行「按 `CARGO_MANIFEST_DIR` 读本包测试的文件头」 | 第 542 行附近读的是 `../../scripts/crash/crash-point-replay-shard-configuration-check.sh`（取这一台的地址），不是测试文件头 | `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:541-548` |

另一个背景没写的事实：测试包里已经有一份「自己写一份」的验证包算法：`common_discrete_gpus_of_this_host` 文件头写着「harness 一侧不依赖 验证包，这里自己写一份」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:7`）——按 PCI 总线号对 wgpu 适配器的那一段。

## 1. F1 判定库存取与算键放哪

### 1.1 闭包实际多大（量的：`modrefs.py`、`symclosure.py`；验证包 `src/` 共 166591 行，`wc -l crates/singlefs-verification/src/*.rs`）

读法：从测试包与 E202 实际 `use` 到的那几样出发，沿 `crate::` 引用往下读。三种粒度，数一次比一次细：

| 口径 | 模块数 | 行数 | 里头有没有 GPU 判器与崩溃放量 |
|---|---|---|---|
| 按文件（非测试代码里 `crate::<模块>` 的引用往下读，整文件算，含文件里的单测） | 26 | 100868 | 有：`crash_amplification` 36247、`gpu_memory_ledger` 9089、`gpu_unit_checks` 6231、`crash_judge_dispatch` 4175、`crash_judge_tables` 3729、`crash_judge_gpu` 2747、`crash_verify_gpu` 856 都在里面。**按文件搬不成立** |
| 按条目，整块 `impl` 算在内，不裁 | 14 | 13689 | 没有判器模块的一行；有崩溃放量的 1535 行（`crash_amplification` 里的 `RecordedFlow`、`CrashPlan`、`PlannedCrashPoint` 这些流与计划的类型）、`crash` 1384、`memory_pool` 677 |
| 按条目，整块 `impl` 算在内，把 `RecordedFlowAndItsPlan` 这一个类型截断 | 10 | 10225 | 判器 0 行，崩溃放量 0 行：`verdict_store` 6534、`crash_library` 2089、`crash` 402、`gpu_card_configuration` 366、`crash_identity` 348、`crash_judge_conclusions` 215、`sha256` 181、`crash_point_replay_progress` 74、`hexadecimal` 10、`judging_units_entries` 6 |
| 按条目，按方法裁（不在已留的类型上的方法不算），测试包与 E202 的全部标识符当根，截断同上 | 10 | 5215 | 判器 0 行，崩溃放量 0 行：`verdict_store` 2343、`crash_library` 1836、`gpu_card_configuration` 356、`crash_identity` 318、`sha256` 181、`crash` 86、`crash_point_replay_progress` 75、`hexadecimal` 10、`judging_units_entries` 6、`crash_judge_conclusions` 4 |

- 名字匹配的上界，不是真闭包：同名的会多算、`cfg` 与宏我没展开；测试代码（`#[cfg(test)] mod tests`）不算。判器模块（`crash_judge_gpu`、`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_dispatch`、`gpu_memory_ledger`、`crash_judge_tables`、`crash_given_states`、`crash_judge_cpu_blocks`）在三种按条目的口径里都是 0 行。
- 为什么按文件的口径拖出判器：`crates/singlefs-verification/src/crash_library.rs:51` 引 `crash_judge_dispatch::{PackedJudgeTables, PackedJudgeTablesReadBack}`（只被 `read_packed_judge_tables_back` 一族用，`:2128`-`:2223`）；`:947`-`:948` 的 `flows_read_back` 字段与 `:1585` 的 `recorded_flow_and_its_plan_read_back_once` 引 `crash_amplification::RecordedFlowAndItsPlan`；`:2656` 的 `STATES_PER_PAIRING_BLOCK` 引 `crash_amplification::STATES_PER_BLOCK`（`:2656`-`:3344` 是逐状态对拍的块，测试包一样不用）。测试包用的那一截（`:100`-`:2654`）里，真正需要截断的只有流缓存那一处。
- 同理 `gpu_card_configuration.rs` 的非测试部分（`:1`-`:1011`）只 `use std::…`（`:14`、`:15`），对 `crash_amplification`、`crash_library`、`gpu_unit_checks` 的引用都在注释里或 `#[cfg(test)]` 之后（`:1012`起），整份可原样搬。

### 1.2 候选甲：拆出一个不含判器、不依赖 wgpu 的包（成立，带四项代价）

做法（包名先写 `singlefs-verdict-library`，只是占位，主 agent 与用户定名）：

| 从哪搬 | 搬到 | 行数（读码） | 要动的地方 |
|---|---|---|---|
| `crates/singlefs-verification/src/sha256.rs` | 新包 `src/sha256.rs` | 234（非测试 197） | 无 |
| `src/hexadecimal.rs` | 新包 `src/hexadecimal.rs` | 26 | 无 |
| `src/verdict_store.rs` | 新包 `src/verdict_store.rs` | 10904（非测试 `:1`-`:6771`，测试 `:6772`起约 4132） | `:146` 引 `crash::{StateCount1280Bits, STATE_COUNT_BYTES}`：把 `crash.rs` 里 `StateCount1280Bits`、`Unsigned256Bits` 与那几个常量（`:1623`-`:2023`，约 400 行）与 `CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER`（`crash.rs:485`）一起搬；`:150`-`:152` 引 `crash_judge_conclusions` 的结论类型（`EncodedVerdictVector`、`BlockCountVector`、`RecoveryWordsOfState`、`LandedRootWords`、`AbandonedRootNamedByThePoolChecker`，整块 impl 口径下约 215 行，只是结论的编码，不含判法）一起搬 |
| `src/crash_library.rs` 的 `:1`-`:2654` | 新包 `src/crash_library.rs` | 约 1840-2090（按方法裁与不裁两种） | 截断流缓存：`OpenLibrary` 的 `flows_read_back` 字段（`:947`-`:948`）、`:1362` 的构造、`:1585` 的方法改成对任意值类型泛型（`ValuesReadBackOnceInThisProcess<Value>` 本来就是泛型，`:1216`起），具体类型由验证包那一侧指定；`:2080`-`:2223`（流与判表读回）、`:2232` 的 `facts_through_the_pairing_library`、`:2531` 的 `judge_tables_through_the_pairing_library`、`:2656`-`:3344`（逐状态对拍的块）留在验证包，新放一份 `crash_library_flows.rs` |
| `src/crash_identity.rs` 的键那一截 | 新包 `src/units_key.rs` | 约 250 | `JudgingUnitsKey`、`JudgingUnitFingerprint`、`ConclusionKind`、`ConclusionVersion`、`composition_of_the_compiled_group`、`judging_units_key_of_the_group_in_the_table_text`；`JudgingUnits` 枚举（它的成员是验证包判器组的名字）、`judging_units_key`、编进二进制的那份表（`:629` 的 `include_str!(…"/judging_units_keys.tsv")`）与 `judging_units_entries.rs`、`build.rs` 留在验证包 |
| `src/crash_point_replay_progress.rs` 的 `CrashPointReplayToolchainIdentity` | 新包 `src/toolchain_identity.rs` | 75 | `crash_library.rs:99` 的 `toolchain_of_this_process` 要它 |
| `src/gpu_card_configuration.rs` 的非测试部分 | 新包 `src/gpu_card_configuration.rs` | 1011 | 测试 `:1012`-`:1959` 引 `crash_amplification` 的环境守卫，留在验证包；`crash_amplification` 原来就 `pub use` 它，改成从新包 `pub use` |

新包 `Cargo.toml` 依赖（读码）：`singlefs-unit-graph`（读回键表要 `entry_groups::parse_entry_group_table` 一族，`crash_identity.rs` 第 18-21 行引它）、`prune-server`（`crates/singlefs-verification/Cargo.toml:33` 的同一行）、`libc`（不需要，除非借卡那一截用到，我没查）。不依赖 `wgpu`、`pollster`、`singlefs-checker`、`singlefs-reference-filesystem`。验证包 `Cargo.toml` 加新包依赖，`lib.rs` 用 `pub use` 把 `crash_library`、`verdict_store`、`sha256`、`hexadecimal`、`gpu_card_configuration` 的旧路径转出来，这样 `crash_library::` 与 `verdict_store::` 的 79 个引用文件（`grep -rln "crash_library::\|verdict_store::\|crash_identity::\|gpu_card_configuration::" crates/singlefs-verification/{src,tests,examples} \| wc -l`，量的）不用逐个改（推的，要编过才算）。

补上一段依赖那一句：新包不需要 `libc`（`grep -c "libc::"` 对 verdict_store、crash_library、crash_identity、gpu_card_configuration、sha256、crash_point_replay_progress 六份都是 0），也不需要 `singlefs-core`（这六份的非测试部分没有 `singlefs_core`；`crash_identity.rs:713`、`:728` 的核心层特性探针留在验证包）。

### 1.3 甲的代价数

| 项 | 数 | 出处与量法 |
|---|---|---|
| 搬动的非测试行 | 按条目整块 impl 口径约 10225，按方法裁约 5215（`VerdictStore` 的 `impl` 不能跨包拆成两半：它的 `impl` 在 `verdict_store.rs:3537`、`:6507`，所以按整块搬） | 1.1 的表（推的：名字匹配上界） |
| 随代码搬走的变异行 | 207 行（`verdict_store.rs` 89、`crash_library.rs` 66、`gpu_card_configuration.rs` 34、`crash_identity.rs` 17、`sha256.rs` 1）：第二列文件路径要改 | `awk -F'\t' '$2=="crates/singlefs-verification/src/<文件>"' crates/mutations.tsv \| wc -l`（量的） |
| 验证包里引这几个模块的文件 | 79 个（`pub use` 转出后可不改，推的） | 同上命令（量的） |
| 判器键（**最大的一项**） | 验证包判器键表 91 组里，成员落在要搬的模块里的有 3 组：`gpu_judge`（`sha256` 9 个单元、`hexadecimal` 2 个）、`gpu_facts_verifier`（`sha256` 2 个）、`observer_conclusion_composition`（`verdict_store` 137、`crash_library` 75、`crash_identity` 7、`sha256` 9、`hexadecimal` 2）。单元数里含 lib.rs 里 `pub mod <模块>;` 的声明单元 | `group_compositions.py <键表> sha256 hexadecimal verdict_store crash_library crash_identity gpu_card_configuration crash_point_replay_progress`（量的，读 2026-11-17 的旧构建产物，要重编后复核） |

为什么「最大」：D13（验证路线） 已定项 17 的键里，闭包里的每个单元进键的是（种类，去注释指纹），单元的名字与路径不进键，但闭包里「声明它所在文件的 mod」与「引进它用到的名字的 use」是单元、也进键（`crates/singlefs-unit-graph/src/closure.rs:11`「单元连模块头与声明它所在文件的 mod、连引进它用到的名字的 use」；键行的写法 `closure.rs:3430`-`:3484`：`entry-target-kind`、`cfg`、`unit` 逐行、`entry`、`toolchain`、`lock`）。所以文本一字不改地把 `sha256.rs` 搬进另一个包，函数单元的指纹不变，但 lib.rs 里那一行 `pub mod sha256;` 换成 `pub use …`，闭包里的单元集合就变了，`gpu_judge` 那一把键就变。而 GPU 判器那一组并进每一块崩溃判定的键（`crates/singlefs-verification/src/crash_identity.rs:885` 的 `judgement_units_key_with_the_gpu_judge`，文档 `:879`-`:882`「两条路线判过的块都换版本、全部现判」）。推的：搬一次，崩溃判定库里镜像判定与给定状态判定的每一块都换版本、`observer_conclusion_composition` 名下的观察者结论块都换版本。次数级照 D13（验证路线） 已定项 17「盘上的数」那一条（`.claude/kb/decisions/13-验证路线.md:453`「整库一把的判法摘要一换累计多判 3 935 242 934 态」）估是 10⁹ 量级，没量过。规则不拦这件事：同一条里「改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑」（`:441`）。**这是要交用户的代价，交之前要量**：在基线树与拆完的树各编一次验证包，比两份 `judging_units_keys.tsv` 里各组的 `key`（命令在末尾「要主 agent 跑的命令」）。

### 1.4 ⑧ 在甲下怎么样

- **理由那一句**（`.claude/gate.d/checker-independence-and-sync.sh:50`「⑧ 同一个理由落在 GPU 上：验证包的 GPU 判器与产品 GPU 包共用一段代码时，那段写错会让判器与被判的产品路径一起错，GPU 对拍照样全绿。」）：字面上不成立——新包里没有判器（1.1 的条目级口径判器模块 0 行），产品 GPU 包也没有任何一行被验证包引。但同一个机制挪到了新包上：崩溃判定与产品对拍两路共用的是同一份「存取、按键复用」代码，新包的键或复用写错（例如 `crash_library.rs:2594` 的 `pairing_conclusion_reused_or_judged` 把过期的「一致」当成复用），两路一起错、对拍照样绿。⑧ 对这一类没有罩（它只比 crate 名与别名，`checker-independence-and-sync.sh:45`）。我判：理由字面「不成立」，共因风险「成立，转到新包上」，罩它靠新包自己的变异行（随代码搬的 207 行）与第 7 格给的「新包不含判器」判据。
- **允许共用清单要加什么**：`checker-independence-and-sync.sh:640` 的 `internal_allowed = closure_without_dev({SHARED_CRATE_NAME, core_name})` 之外，交集（`:641`）会多出：新包、`singlefs-unit-graph`、`singlefs-unit-graph` 的外部依赖 `verus_syn`、`proc-macro2`、`quote`（`crates/singlefs-unit-graph/Cargo.toml` 的 `[dependencies]`）、`prune-server`（`crates/singlefs-verification/Cargo.toml:33`；新包的依赖，产品 GPU 包 dev 依赖新包之后进它的闭包）。F3 若走丙还要加那个替身包。这几个是名字，写成一份登记（包名、理由、「不许含 wgpu/naga/ash/pollster 依赖与 `.wgsl` 文件」），不写成通配。
- **要不要另判「新包不含判器」**：要。⑧ 的交集那一条只看名字，新包以后往里添判器 ⑧ 看不见。判据见第 7 格缺口 3 与下面 1.7。

### 1.5 候选乙：产品 GPU 包测试自写一份存取与算键（撞前提 3；第二个问题是对拍没有合法的家）

- 撞前提 3：「我们在录制入库部分的逻辑应该是一致的。 不要做第二套或者第三套东西。」（`records/2026-11-11-GPU两段与按需领卡核查.md:126` 那一段，背景正文前提 3）。乙就是第二套。能引来撑乙的只有同一份记录 `:141` 的「用户补充（键值）：原话「互不依赖 代码可以不同，但是输出的键值要一样。」」——它放宽的是「包之间互不依赖」，同一段后面写的是新包直接调验证包「键值出自同一份代码，自然相同」，没有授权第二套存取代码。
- 「逐字节对拍」没有合法的家：要在一个进程里同时算验证包那份与测试里自写那份，这个进程所在的包要同时依赖两边——就是要撤的那个测试包的形状。只能退成「黄金向量」：两边各断言同一批输入算出同一批键与值（向量放哪份文件，由哪边引，又是一次跨包引用）。
- 体量（推的）：按方法裁的上界约 4.2k 行（`verdict_store` 2343 + `crash_library` 1836），真写一个只服务对拍路径的子集会小得多但没量；变异行（对拍这一段的 `crash_library`、`verdict_store` 变异）要再写一遍。
- 键不换：验证包一行不动，判器键不变。这是乙对甲唯一的、也是实在的优势。

### 1.6 有没有第三条路

- 把存取与算键搬进 `prune-server` 的库：`prune-server/Cargo.toml` 只依赖 `rocksdb`、`libc`，自成工作区；算键要 `singlefs-unit-graph`，那会让服务工程依赖本仓的包。不成立（读码）。
- 搬进 `singlefs-unit-graph`：它的 description 写「只用 std……不依赖工作区任何 crate」（外加 `verus_syn`），加 `prune-server` 依赖就破了这句。不推荐。
- 产品测试经 Python 客户端 `scripts/prune-server/prune_server_client.py` 读写，键在 Rust 一侧另算：还是第二套。不成立。
- 两层：新包只放「存取」（`verdict_store` 的传输层、表、`ItemManifest`），算键与按键复用留在验证包，产品测试自己按键调存取：新包缩小，但「谁算键」又落回两边各一份，等于乙的一半。推不出更好。
- 反向的省法：`sha256` 的 `DIGEST_BYTES` 测试里要用，`singlefs-unit-graph` 已经有一份 `sha256`（`crates/singlefs-unit-graph/src/lib.rs:33` 的 `pub mod sha256`），产品测试只要 `DIGEST_BYTES` 可以不经新包；但仓里本来就有两份 SHA-256，这不是这一轮的题，只记一笔。

### 1.7 F1 判定与什么会推翻它

- 甲：**成立**（能落地：判器与崩溃放量一行不被拖出，按条目口径；代价见 1.3，最大的是判器键）。
- 乙：**不成立**（撞前提 3；逐字节对拍没有合法的家，退成黄金向量）。
- 第三条路：我读到的几条都不成立或不更好，**推不出**有更好的一条。
- 推翻甲「不含判器」的现象：新包搬完之后 `grep -rn "crash_judge\|gpu_unit_checks\|crash_verify_gpu\|gpu_memory_ledger\|wgsl" crates/<新包>/src` 有命中，或新包 `cargo tree -p <新包>` 里出现 `wgpu`、`naga`、`pollster`；名字匹配漏掉一条隐式依赖（宏、`include_str!`、`cfg`）。
- 推翻「判器键会变、全部现判」的现象：搬完后基线树与新树各编验证包，`judging_units_keys.tsv` 里 `gpu_judge`、`gpu_facts_verifier`、`observer_conclusion_composition` 三组的 `key` 逐字相同（那就是 unit-graph 把转出的 `pub use` 与新包的 `pub mod` 当成同一组单元，我读码推出相反的结论，没有现跑）。

## 2. F2 E202 的装置放哪

背景事实（读码）：E202 的 bin 有两处 `#[path]` 把 `tests/common_discrete_gpus_of_this_host/mod.rs` 与 `tests/common_pairing_input_through_the_library/mod.rs` 编进来（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:157`「#[path = "../../tests/common_discrete_gpus_of_this_host/mod.rs"]」、`:160`），非测试代码里用它们取卡与对拍（`:147`-`:155` 的 `use`、`:5826`-`:5840`、`:8924`）；bin 里 `mod tests`（`:9494`起，约 4600 行）另有内联真卡测试与一次性库的测试（`:12416`、`:14060`、`:14070`）。它用的验证包东西全在 1.2 搬进新包的那一截里（`PairingJudged`、`cards_handed_over_or_leased_on_demand`、`CardLeaseChannelOfTheCaller`、`RunStage`、`TheOneLibraryKeptForThisProcess`、`DisposableLibrary`、`JudgingUnitsKey`、`PairingAgreement`、`DIGEST_BYTES`），所以 E202 搬走以后不需要依赖验证包。

### 2.1 丙：验证包 `src/bin/`（不成立，撞 ⑧）

验证包要依赖产品 GPU 包，门禁 ⑧ 判红：`.claude/gate.d/checker-independence-and-sync.sh:38`「两边的传递闭包（dev-dependencies 也算）里都没有对方」，样本 `checker-implementation-disjoint-gpu-verification-depends-red`（`:99`）。这条标准位置（`.claude/rules/verification.md@定义与名字` 验证包一行写「真设备装置二进制（`src/bin/`）」）被 ⑧ 堵死。

### 2.2 甲：`crates/` 下新开一个只放「要产品 GPU 包的实现驱动装置」的装置包（成立，推荐）

| 规矩 | 撞不撞 | 依据 |
|---|---|---|
| ⑧ | 不撞：装置包依赖产品 GPU 包与新判定库包，不依赖验证包；谁都不依赖装置包；⑧ 只比验证包与产品 GPU 包两个闭包 | `checker-independence-and-sync.sh:38`、`:641` |
| `implementation-first.md` 第 4 条 | 不撞：条文管的是 `scripts/experiment-apparatus/` 与 `research/prompts/` 下的装置与模型「只用来独立验证：不与实现共用代码」（`.claude/rules/implementation-first.md@规矩` 里「不与实现共用代码，用来核实现、核推论」）；装置包不在这两处，它与验证包 `src/bin/` 里那 26 个实现驱动装置同一类 | `crates/singlefs-verification/Cargo.toml` 的 26 条 `[[bin]]` |
| `directory-layout.md` 的层数与「不放结果」 | 不撞，也不受它管：条文只管 `scripts/` 与 `research/`（`.claude/rules/directory-layout.md@scripts/：反复要用的脚本`）；装置包里不放产物 | 这正是调度记录缺口 1、2 |

装置读判定库那一段：装置包 → 新判定库包（1.2）；对拍的取卡与对拍共用模块经第 5 格的支撑包。

做法（文件到文件）：

1. `git mv crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs crates/<装置包>/src/bin/e202_gpu_cpu_gc_comparison.rs`（14096 行，目录深度不变，所以 `:13832`-`:13852` 里 `include_str!("../../../singlefs-background-gpu/Cargo.toml")` 这类相对路径照旧有效）。
2. bin 里改：`:142`-`:146` 的 `use singlefs_verification::…` 换成新判定库包；`:147`-`:161` 两处 `#[path]` 换成引支撑包的模块（第 5 格）；`:4140`-`:4147` 摘要要走的八处路径里最后两项（共用模块的文件）换成支撑包的文件。摘要范围要不要加支撑包与判定库包的 `src`，是登记第 15 条（`research/prompts/e202-preregistration.md:549`）的修订，实验设计员写，这一轮不判。
3. 新建 `crates/<装置包>/Cargo.toml`：`[dependencies]` 产品 GPU 包、核心层、`singlefs-format`、`singlefs-replay-prefix`、`singlefs-mount-runtime`、新判定库包、支撑包、`wgpu`（照 `crates/singlefs-background-gpu-tests/Cargo.toml` 现有那一行）；`[build-dependencies] singlefs-unit-graph`；`[features] tests-on-a-real-gpu = []`；`[[bin]] name = "e202_gpu_cpu_gc_comparison"`。E202 两组单元键（`reference_units_entries.rs` 里 `bin:` 那一行、`gpu_side_units_entries.rs:165` 起那一行）的登记表与构建脚本跟着它走（F4）。
4. 根 `Cargo.toml` 的 `members` 加装置包。
5. `.claude/gate.d/experiment-number-directories.tsv:17`「crates/singlefs-background-gpu-tests/src/bin」换成装置包的 `src/bin`（不登记，门禁 doc-experiments 的 experiment-orphans 格就不认这个目录里的实验号：它按这张表读，注释在 `.claude/gate.d/doc-experiments.sh:2193`）；`.claude/gate.d/comment-history-lag.tsv:11` 的路径列；`.claude/gate.d/path-moves.tsv` 加一行。
6. kb 页的三份：`.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md` `:7`（装置路径与「住在产品 GPU 的测试包 background-gpu-tests 里」那句）、`:21`（复跑命令，`-p` 换装置包名，`--features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison` 照旧）、`:49`（历史版本节里「搬进它的测试包」那一句是历史，不改，加一行说这次搬家）；`.en.md`、`.ja.md` 同样的 `:7`、`:21`、`:49`（`grep -n "background-gpu-tests" .claude/kb/experiments/202-*`，量的）；欠账 C1046（`.claude/kb/checks-owed.md:648` 与 en、ja 两份）里的装置路径。
7. 跑前登记 `research/prompts/e202-preregistration.md`：`:479`、`:499`、`:549`、`:551`、`:617`、`:718`、`:732` 带装置或测试包路径；`:774`-`:776` 三条起法写的还是更早一次搬家前的 `-p singlefs-background-gpu`，与 kb 页 `:21` 已经不一致，改之前先判以哪份为准（推的）。登记是路径引用，照 `.claude/rules/path-moves.md@文件与目录的路径不在冻结范围内`「全仓指向它的地方一次改成新路径」改；抄下来的命令与输出照 `怎么做` 第 5 步留旧。
8. `crates/mutations.tsv`：第二列是 E202 装置文件的 143 行（路径列）、第五列 `-p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- <测试名>` 的 `-p` 换包名（`awk -F'\t' '$2=="crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs"' crates/mutations.tsv \| wc -l` 得 143；第五列含旧包名的共 243 行，量的）。这些行的原文与替换文不变，行文本变了，crates 变异复判下一趟全部现判这 143 行。
9. `crates/singlefs-background-gpu/Cargo.toml` 的 `[features]` 注释（`:31`-`:33` 说特性留着是给测试包的构建脚本）第 4 节 F4 里一起改。

### 2.3 乙：`scripts/experiment-apparatus/` 下一个包（照 `e176_crates_oracle` 的接法）（不成立）

| 规矩 | 撞不撞 | 依据 |
|---|---|---|
| `implementation-first.md` 第 4 条 | **撞字面**：E202 量的是产品整理在两种模式下的对照，是用产品代码当被测对象（bin 里 `use singlefs_core::…` 20 行、`use singlefs_background_gpu::…`，`e202_gpu_cpu_gc_comparison.rs:62`-`:141`），不是「独立验证」。先例 E176 范围窄：「只有它以路径依赖引 crates/ 的 singlefs-core、singlefs-format」，只读、拿两个纯函数当参照（`scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:3`） | `.claude/rules/implementation-first.md@规矩` |
| `directory-layout.md` 层数 | 不撞：`scripts/experiment-apparatus/e202_gpu_cpu_gc_comparison/{Cargo.toml, main.rs}` 是 `scripts/<子目录>/<装置目录>/<文件>` 三层（不能有 `src/`） | `.claude/rules/directory-layout.md@scripts/：反复要用的脚本` |
| `directory-layout.md`「不放结果」 | 不撞，装置不放产物 | 同上 |
| ⑧ | 不撞（不在 `crates/` 下） | `checker-independence-and-sync.sh:38` |

乙自己的几项硬伤（读码）：
- 单元键：`singlefs-unit-graph` 的 `load_workspace`（`crates/singlefs-unit-graph/src/workspace.rs:1367`）只收 `crates/` 下的包与顺它们 path 依赖收到的包；装置包自成工作区，E202 两组 `bin:` 键（现在由测试包的 `build.rs` 算、`include_str!(OUT_DIR…)` 读）在这个位置没有构建脚本可挂，要改走 `scripts/experiment-apparatus/device-code-key.py` 的 `--code-key-file`（它「包目录」只认两种：`scripts/experiment-apparatus` 与 `crates/<名>`，`device-code-key.py:28`）。E202 里对拍取键的那一层要重写。
- 变异行：143 行用 `-p <包> --bin e202_gpu_cpu_gc_comparison`，包不在根工作区里 `-p` 点不到；E176 的清单自己写着这类包「不挂在 scripts/experiment-apparatus 的可选特性上……副本旁边没有 crates/，整个包每一张变异表的基线都编不过」（`e176_crates_oracle/Cargo.toml:4`-`:5`）。crates 变异门禁的复制方式我没读，推的。
- 判定库：路径依赖新判定库包的话，它的构建脚本读的是根 `Cargo.lock`，装置包用自己那份 `Cargo.lock`，两份对不上时键里的依赖行不同（推的）；走 `singlefs-device-library` 管道（`crates/singlefs-verification/Cargo.toml:34` 起的注释）只有 read、write、write-many 三个原始操作，E202 内联测试要起临时 prune-server（`DisposableLibrary`，`:14070`），管道给不了。
- 对拍的两份共用模块还是要经一个 crate 才能被装置包与产品测试同时引，装置包放在 `scripts/` 并没有省掉支撑包。

### 2.4 F2 判定

- 甲：**成立**，推荐。丙：**不成立**（撞 ⑧）。乙：**不成立**（撞第 4 条字面，键、变异行、判定库三处要重做）。
- 什么会推翻：①装置包搬出去以后 `cargo metadata` 里验证包或产品 GPU 包的依赖闭包出现装置包；②用户把 `implementation-first.md` 第 4 条改成放行「实现驱动装置」且 unit-graph 增加对 `scripts/experiment-apparatus` 包的发现——那样乙的两项硬伤就没了。

## 3. F3 核心层特质的 CPU 替身 `common_background_gpu_compute` 放哪

事实（读码、量的）：替身文件 310 行，只依赖 `singlefs-core`（`crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:15`-`:35` 的 `use`）；验证包里 8 个测试目标文件加 1 个共用模块 `common_compaction_relocation_on_the_background_compute` 用它（`grep -l common_background_gpu_compute crates/singlefs-verification/tests/*.rs crates/singlefs-verification/tests/*/mod.rs`，不算 `.background_compute_settings_and_switch.rs.lhuq544s.replacing` 那份别的会话留着的编辑中间文件）；产品一侧的一处在 `crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11`「#[path = "../../singlefs-verification/tests/common_background_gpu_compute/mod.rs"]」。这个测试拿它当对拍的另一边（同一文件头 `:4`「对拍的另一边是核心层测试用的假实现」）：替身自己数连续不确定、自己判（`mod.rs:6`-`:7`「只在这份实现里」），与产品那一份是两份独立写的逻辑，这正是它的用处。
`crates/mutations.tsv` 里点名它的有 2 行（第二列），搬了要改路径。

| 候选 | 撞 `fs-design.md`「运行时与 checker 必须用不同的算法」 | ⑧ 的盲区 | 核心层依赖方向 | 代价 | 判 |
|---|---|---|---|---|---|
| 甲 核心层带测试特性导出 | 替身住进运行时那个包：替身是产品实现的「对拍另一边」，对拍双方的一方放在被调用的核心层里，条文说的「审计与被审计用同一段代码」没被打破（替身调的是核心层已有的 CPU 函数，今天也是）但读起来别扭 | 在允许共用里（核心层），⑧ 不会再看它 | 不变（核心层不依赖别人） | 加特性要动三处：`crates/singlefs-core/build.rs:32` 一族的特性表、`crash_identity.rs:656` 的 `CORE_LAYER_FEATURES_RECOGNIZED_FROM_ITS_CODE`（它的长度 `[&str; 2]` 与两个特性探针）、`crates/singlefs-unit-graph/src/build_features.rs` 里比清单与脚本的单测；核心层里放 310 行测试代码，违背用户要的「测试就该在测试的地方」 | 不推荐 |
| 乙 并进 F1 甲那个新判定库包 | 不撞 | 新包本来在允许共用里 | 新包多依赖核心层（核心层在允许共用里） | 零个新包；但新包里放判定库存取与一个核心层替身，职责混了，「新包不含判器」判据要写成「不含判器、不含 wgpu」之外再点名替身 | 备选（最便宜） |
| 丙 单开一个只依赖核心层的测试支撑包 | 不撞 | 新包两边都依赖，⑧ 的允许清单加这一个名字（只依赖核心层，没有 wgpu/判器） | 不变 | 一个新包 310 行；验证包 8+1 个文件的 `mod common_background_gpu_compute;` 换成 `use <支撑包>::…`；产品那一处 `#[path]` 换成依赖；2 行变异路径 | **推荐** |
| 丁 两边 `#[path]` 引同一份 | 不撞算法 | **落在盲区**：产品测试跨包 `#[path]` 引验证包的源码，⑧ 只比 crate 名看不见（`checker-independence-and-sync.sh:45`），正是调度记录缺口 3 与原因 5 的那一处 | 不变 | 零改动 | **不成立**（前提「不许跨包 `#[path]`」；第 7 格缺口 3 的判据落地后这一处必红） |

- 乙与丙的取舍是「少一个包」与「一个包一种职责」。调度记录措施 M1「不为测试另开包」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:72`：「不为测试另开包」）要写成「一个包的测试放它自己的 `tests/`」，两个以上的包共用的测试支撑另说：新判定库包、支撑包、替身包都是这种，M1 的措辞要把这个例外写进去，否则丙与第 5 格的支撑包都撞 M1。
- 判：甲「不成立（不推荐）」、乙「成立」、丙「成立」、丁「不成立」。推荐丙；用户若要最少的包，退乙。
- 什么会推翻丙：验证包 8+1 个文件换成 `use` 之后，门禁 ⑧ 报交集里多出的不止支撑包一个名字；或替身里有一处用到验证包或产品的东西（`mod.rs` 的 `use` 当前只有 core，`grep -n "singlefs_" crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs` 命中的都该是 `singlefs_core`）。

更正第 3 节事实里的两处行号：替身文件的 `use` 在 `crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:12`-`:35`（不是 `:15`）；「只在这份实现里」那半句在 `:6`（不是 `:6`-`:7`）。核心层特性表的单测是 `crates/singlefs-unit-graph/src/build_features.rs:747` 的 `the_core_build_script_lists_every_feature_of_its_manifest`。

## 4. F4 对拍两侧的单元键在哪算

今天（读码、量的）：测试包 `build.rs`（218 行）用 `load_workspace`（`crates/singlefs-background-gpu-tests/build.rs:124`）装工作区，两张登记表经 `#[path]` 编进来（`build.rs:8`-`:11`），写成 `OUT_DIR/reference_units_keys.tsv` 与 `gpu_side_units_keys.tsv`；测试用 `include_str!(concat!(env!("OUT_DIR"), …))` 读回（`tests/common_pairing_input_through_the_library/mod.rs:36`-`:40`）。登记表里：参照侧 7 组、GPU 侧 31 组（`grep -c "group_name:"` 对 `gpu_side_units_entries.rs` 得 31，对 `reference_units_entries.rs` 得 8 含结构体字段那一行），入口路径都以 `singlefs_background_gpu_tests::` 起头（`grep -c` 得 31 与 9 行），其中各有一组 `bin:e202_gpu_cpu_gc_comparison`（`reference_units_entries.rs:65`、`gpu_side_units_entries.rs:165`）。

### 4.1 键会不会因为包名、路径变（读码）

不会。键行是 `entry-target-kind`、`cfg` 原子的真值、闭包里每个单元的（种类，去注释指纹）、数据单元、入口链的（种类，指纹）、工具链、依赖行（`crates/singlefs-unit-graph/src/closure.rs:3430`-`:3484`）；D13（验证路线） 已定项 17 写明「路径（仓根也算）、名字、种子串、行号、资源上限、限时」一律不进键（`.claude/kb/decisions/13-验证路线.md:436`）。登记表里入口路径的头一段只是用来在工作区里找包，不进键。

会变的是**文本变了的单元**：闭包把「引进它用到的名字的 use」与「声明它所在文件的 mod」连进来（`closure.rs:11`），所以测试文件里 `use singlefs_verification::…` 换成 `use <新判定库包>::…`、`mod common_…;` 换成 `use <支撑包>::…`、`#[path]` 去掉，凡是用到这些名字的单元指纹都变。

### 4.2 哪几组判定要现判（推的，要用 4.4 的命令量）

| 组 | 会不会变键 | 为什么 |
|---|---|---|
| GPU 侧 31 组（含 E202 那 1 组） | 会：每条持卡判入口都调 `judgement_of_the_assertions_on_the_cards`，那个函数与它引的 `PairingJudged` 住在 `common_pairing_input_through_the_library`，那里的 `use singlefs_verification::…`（`mod.rs:16`-`:32`）要换 | 31 个对拍结论全部上卡重判一次 |
| 参照侧 7 组（含 E202 那 1 组） | 预计不变：入口是 `reference_bytes_of_*`，闭包是产品的 CPU 参照与核心层函数，不经过要换的 `use` | 要在新旧树各编一次、比 `reference_units_keys.tsv` 里各组的键才算数 |
| 录入键（每一条对拍的「输入与参照」那一条记录） | **会，而且与 F4 选甲还是乙无关**：`crash_library.rs:1773` 与 `:2427` 取录入键时调 `judging_units_key(&JudgingUnits::Recording, …)`，`recording` 那一组的入口是验证包的流录入与 GPU 判表建表（`judging_units_entries.rs:122`起：`crash_facts::extract_flow_facts_with_slices`、`crash_judge_tables::build_judge_tables_…`、`PackedJudgeTables::of`、`RecordedFlow::to_bytes`），产品 GPU 包没有验证包，取不到这一组 | 搬了以后产品一侧的录入键要换成新判定库包自己的一组（见 4.3），所有对拍的输入与参照记录重录一遍（CPU，不上卡） |

顺带的发现：今天对拍记录的录入键里带着验证包的流录入与 GPU 判表建表代码的指纹（`common_pairing_input_through_the_library/mod.rs:314` 同一句），验证包改判表建表，产品对拍的记录也跟着作废；换成新判定库包自己的一组是更干净的方向，没有用「同一份输入同一把键」去换（同一份输入算出的键今天就依赖验证包的编译结果）。

### 4.3 甲：产品 GPU 包的 `build.rs` 接过这一段（成立，推荐）

1. `crates/singlefs-background-gpu/Cargo.toml`：加 `[build-dependencies] singlefs-unit-graph = { path = "../singlefs-unit-graph" }`；`links = "singlefs-background-gpu"`（`:5`）保留（装置包的构建脚本还读 `DEP_SINGLEFS_BACKGROUND_GPU_ENABLED_FEATURES`）；`[features]` 的 `tests-on-a-real-gpu = []`（`:34`）上面三行注释（`:31`-`:33`「本包的源码里没有按这个特性编的代码……」）改成「要真显卡的测试目标挂它」。
2. `crates/singlefs-background-gpu/build.rs`：现在 23 行（只交出特性）；并入测试包 `build.rs` 的 218 行里除 `main` 开头以外的部分（`keys_of_the_entry_groups`、`InputsOfTheEntryGroupKeys`、`main` 里算键与写 `OUT_DIR` 那段），把 `:8`-`:11` 的 `#[path]` 改指产品包的登记表；`"crates/singlefs-background-gpu-tests/build.rs"` 那几条 watched 字符串（`build.rs:125`起的 `watched` 集合）换路径。`load_workspace` 加载整个工作区，只在编这个包时跑；今天没有别的包依赖产品 GPU 包（`grep -n "singlefs-background-gpu" crates/*/Cargo.toml gpu-verification-server/Cargo.toml` 只有测试包那一行，量的），所以不拖慢别的包的构建。
3. 两张登记表放 `crates/singlefs-background-gpu/tests/common_pairing_units_entries/{reference_units_entries.rs,gpu_side_units_entries.rs}`，不放 `src/`：`.claude/rules/directory-layout.md:19`：「不变量清单、登记表、钉表这类只被某一组测试读的定义与输入，放在那组测试所在的 crate 目录里」（小节「测试的定义与输入跟着测试放」）；目录没有 `main.rs`，cargo 不把它当测试目标。入口路径头一段 `singlefs_background_gpu_tests::` 换成 `singlefs_background_gpu::`（40 行，两张表）；E202 那两行（`bin:` 目标）挪到装置包自己的登记表（F2）。结构体 `PairingSideUnitsEntryGroup`（`reference_units_entries.rs:7`起，9 行）两处各一份，是数据类型的两份，不是逻辑；要消掉它得放进 `singlefs-unit-graph`，单独立项。
4. 新判定库包自己一组：新判定库包有自己的 `build.rs` 与一张只有一组的登记表（`pairing_library_recording`，入口是它自己的 `pairing_input_through_the_library_under_the_recording_key`、`pairing_input_and_reference_through_the_library_under_the_recording_key`、`recording_key_of_the_pairing_input_and_the_reference_units`、`ItemManifest::to_bytes` 一族），像验证包那样把键表编进自己；`crash_library` 里取录入键改成读这一组（`recording_key_of_the_inputs` 去掉对 `JudgingUnits::Recording` 的引用，由调用方给 `&Result<JudgingUnitsKey, String>`，`recording_key_of_the_inputs_under` 今天已经收这个参数，`crash_library.rs:1781`）。验证包那一侧自己的流录入仍用验证包的 `recording` 组。这是新判定库包的第二处构建脚本（验证包一份、测试包一份、新判定库包一份、产品包一份，驱动代码重复四份）：建议把「取特性、工具链行、`Cargo.lock`、watched」那一段（`build.rs:96`-`:183`）收进 `singlefs-unit-graph` 一个函数，四个 `build.rs` 各剩登记表与一行调用；这是额外一刀，不做也能落地，但前提「不要第二套」会被读成又多一份。
5. `tests/reference_units_entries_name_functions_that_exist.rs`（394 行）随产品测试搬进产品包，它的 `:173` 有一处 `include_str!("../src/bin/e202_gpu_cpu_gc_comparison.rs")`（读 E202 的源码核登记的入口名在不在）：E202 搬走以后这是**跨包 `include_str!` 引源码**；把 E202 那一组的核对拆到装置包自己的 `tests/`，两边各核各的登记表（第 7 格缺口 3 的判据要点到的就是这一种）。
6. ⑧：产品包多一个构建依赖 `singlefs-unit-graph`，闭包里多 `singlefs-unit-graph`、`verus_syn`、`proc-macro2`、`quote`，与验证包的闭包相交（验证包 `Cargo.toml:20`、`:40` 依赖它），要在允许清单里登记（与 1.4 同一批名字）。

### 4.4 乙：随新判定库包提供算法、测试运行时算（不成立）

- 键「跑时不读盘上的树，也不收仓根（仓根不是键的输入）」（`crates/singlefs-verification/src/crash_identity.rs:804`），是编进二进制的（`:629` 的 `include_str!(concat!(env!("OUT_DIR"), "/judging_units_keys.tsv"))`）。运行时算要读 `crates/**/*.rs` 的源码树，测试运行时没有构建时的 `CARGO_ENCODED_RUSTFLAGS`、`TARGET`、`$RUSTC -vV` 这些输入（测试包 `build.rs` 取它们靠 cargo 起构建脚本时给的环境）。不成立。
- 乙若读成「新判定库包提供构建脚本用的驱动函数、各包 `build.rs` 调它」，那是甲的一种做法（4.3 第 4 点的收进 `singlefs-unit-graph`），可以并。

### 4.5 F4 判定与怎么量

- 甲：**成立**；乙（运行时算）：**不成立**。
- 要主 agent 量的三样（命令在末尾）：①基线树里编测试包，留下 `reference_units_keys.tsv`、`gpu_side_units_keys.tsv` 各组的键；②拆完的树里编产品包，比对，参照侧 7 组应当键不变、GPU 侧 31 组应当全变；③验证包判器键表前后比（1.3）。
- 什么会推翻「参照侧 7 组不变」：新树里 `reference_units_keys.tsv` 的参照侧某组键变了——说明那一组的闭包里有一个被改过的 `use` 或 `mod` 单元，要点名是哪一个再判。

更正第 4 节几处行号（读码，`grep -n` 现取）：测试包 `build.rs` 里 `environment_variable` 在 `:41`、`output_of` 在 `:45`、`InputsOfTheEntryGroupKeys` 在 `:60`、`keys_of_the_entry_groups` 在 `:72`、`main` 在 `:109`、`watched` 集合在 `:161`（4.3 第 2、4 点写的 `:96`-`:183`、`:125` 起都错了：驱动代码是 `:41`-`:107` 的辅助函数加 `:109`-`:218` 的 `main`）；`PairingSideUnitsEntryGroup` 在 `reference_units_entries.rs:8`；`include_str!` 的两个常量在 `tests/common_pairing_input_through_the_library/mod.rs:35`-`:40`。

## 5. 共用模块怎么分

前提：F1 甲（新判定库包，下称判定库包）、F3 丙（替身包）、F2 甲（装置包，下称装置包）；E202 搬走以后既要用产品测试的取卡、又要用对拍共用模块，所以这两份必须放在一个**库包**里让产品测试与装置包同时依赖，不能 `#[path]`、不能抄两份。这个库包下称支撑包：依赖产品 GPU 包、核心层、判定库包；产品 GPU 包把它放进 `[dev-dependencies]`，产生一个 dev 依赖环（支撑包 → 产品包，产品包的测试 → 支撑包）。cargo 允许 dev 依赖成环（集成测试用同一份产品库），我没编过，门禁 ⑧ 的 `closure_of` 带已见集合（`checker-independence-and-sync.sh:339`起），不会死循环；unit-graph 对环的行为我没读，推的，主 agent 要先编过 `cargo check -p singlefs-background-gpu --tests` 再往下做。

| 模块 | 行数（量的） | 谁用（量的：`grep -l "^mod …"`） | 住哪 | 要改什么 |
|---|---|---|---|---|
| `common_discrete_gpus_of_this_host` | 859 | 11 个测试目标 + E202（引用它的行：测试 40、E202 11） | 支撑包 `src/discrete_gpus_of_this_host.rs`（整份，去掉 `:22` 的 `#![allow(dead_code…)]`，改 `pub mod`） | 产品测试 11 个 `mod common_discrete_gpus_of_this_host;` 换成 `use singlefs_gpu_test_support::discrete_gpus_of_this_host;`，调用处前缀跟着改（40 行）；E202 的 `#[path]`（`:157`）换成同一个 `use`（11 行）；`mod.rs:542` 的 `env!("CARGO_MANIFEST_DIR")` 取的是编它的那个包的目录，支撑包与产品、装置包同在 `crates/<包>/`，`../../scripts/…` 的深度不变 |
| 它的两个单测目标 `real_gpu_tests_open_only_the_gpus_the_task_server_handed_over`（378 行）、`real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus`（287 行） | 665 | 只测这个模块 | 搬进支撑包自己的 `tests/`（测一个包的东西住那个包的 `tests/`），不留在产品包里 | 两个文件里的 `mod common_discrete_gpus_of_this_host;` 换成 `use`；它们不挂特性 |
| `common_pool_on_memory_devices` | 432 | 4 个产品测试目标（只依赖核心层） | 留产品包 `tests/common_pool_on_memory_devices/mod.rs` | 无；E202 不用（`grep -c` 得 0） |
| `common_pairing_input_through_the_library` | 493 | 9 个测试目标 + 两个共用模块 + E202（引用行：测试 32、E202 11） | **拆成两层**：通用的那一层（造输入字节过库、读回、对拍结论入库、按需领卡、`judgement_of_the_assertions_on_the_cards` 一族）进支撑包 `src/pairing_input_through_the_library.rs`，两张键表以参数交进来；包里留一个薄壳 `tests/common_pairing_input_through_the_library/mod.rs`（产品）与 `src/pairing_tables.rs`（装置包），只放两条 `include_str!(concat!(env!("OUT_DIR"), …))` 与把表交给通用层的转调 | 今天 `:35`-`:40` 的两条常量是包专有的（OUT_DIR 在各自的包里），所以不能整份进支撑包；通用层的函数多一个 `tables` 参数，薄壳里的同名函数把它填上，9 个目标的调用点不变。`:314` 的 `judging_units_key(&JudgingUnits::Recording, …)` 换成判定库包那一组（4.3 第 4 点）。`pairing_conclusion_on_the_cards…` 目标用的 `GPU_SIDE_UNITS_KEYS_TEXT`（`:39`）留在薄壳里 |
| `common_history_pool_through_the_library` | 158 | 3 个产品测试目标（用对拍共用模块与造池模块） | 留产品包 | `use crate::common_pairing_input_through_the_library::…` 不变（引的是薄壳） |
| `common_checksum_batches_over_the_handed_over_gpus` | 322 | 3 个产品测试目标 | 留产品包 | 它引的 `crate::common_discrete_gpus_of_this_host::…` 换成支撑包的 `use`（13 行里的几行） |

- 每一处有没有跨包 `#[path]`：没有。产品包测试只剩包内 `mod`；E202 在装置包里经依赖用支撑包与判定库包；验证包经依赖用判定库包与替身包。有没有抄成两份：支撑包的两个模块各一份；薄壳是「把包专有的 OUT_DIR 表接给通用层」的转调，没有逻辑，装置包与产品包各一份。数据类型 `PairingSideUnitsEntryGroup` 在 F4 里已写是两份（登记表只用 `std`，不能依赖别的包）。
- 支撑包里放的是真卡测试的取卡与对拍共用代码，不含判器；它依赖产品 GPU 包，所以**不能**进验证包的闭包。验证包不依赖它，⑧ 的两边闭包里验证包那一边没有它，产品那一边有，不在交集里，不用登记进允许清单。
- 判：这一格**成立**（五个模块都有去处，没有跨包 `#[path]`；抄两份的只有 `PairingSideUnitsEntryGroup` 一个 9 行的数据类型与两个薄壳）。
- 什么会推翻：①`cargo check -p singlefs-background-gpu --tests` 报 dev 依赖环不被接受或 `links` 键冲突（产品包与支撑包同时在图里）；②薄壳在 9 个目标里要加的 `tables` 参数不止转调——那就是通用层的函数签名没收拢。

## 6. 撤包之后的引用（量的：`git -c core.quotepath=false grep -c "singlefs-background-gpu-tests\|singlefs_background_gpu_tests"`，输出在 `rerun.sh` 的 `out/refs-classes.txt`）

此刻 191 个文件、1274 处（开工时 187、1261，多出的是这一轮自己的材料：调度记录 3 处、`_gpu-tests-placement-design-r1-` 三份 10 处）。

| 类 | 文件 | 处 | 怎么办 |
|---|---|---|---|
| `research/prompts` | 97 | 786 | 见下 |
| `crates/mutations.tsv` | 1 | 244 | 活的表：第二列文件路径（E202 的 143 行、搬走的产品测试文件 40 行、其余指向测试包的）与第五列 `-p` 包名（243 行）改；行文本变了，下一趟这些行全部现判 |
| `records` | 16 | 64 | 见下 |
| 测试包自己 | 9 | 62 | 随包搬走或删除 |
| `.claude/gate.d/fixtures`（样本档） | 35 | 49 | 门禁常量的样本世界：`crash-point-replay.sh/product-gpu-*` 十几个目录里每个都有 `Cargo.toml` 与 `crates/singlefs-background-gpu-tests/Cargo.toml`，`expect`、`setup.sh` 写着包名；常量改成产品包名之后样本世界要照新布局改（`path-moves.md` 第 7 步） |
| `.claude/gate.d`（常量与登记表） | 8 | 25 | `crash-point-replay.sh`（7 处，`:794` 的 `PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu-tests"` 与格标题 `:12`、`:265`、`:271` 起法 `-p singlefs-background-gpu-tests`、`:792`、`:587`、`:596`）、`stage-inputs.tsv:31`（`tests-of:singlefs-background-gpu-tests`）、`stage-owners.tsv:6`、`doc-experiments.sh:2193`、`experiment-number-directories.tsv:17`、`comment-history-lag.tsv:11`、`unsafe-allowances.tsv`、`path-moves.tsv`（12 处，历史行与这一次要加的行） |
| `.claude/kb` | 12 | 18 | 中、英、日三份 × （E202 页、`checks-owed`、`milestone/04-fourth-txn`、`verification-build`）；E202 页的行号见 2.2 第 6 点；`verification-build.md:6` 的「`crates/` 下十一个 crate」要数新包（4 个新包、撤 1 个） |
| `scripts` | 3 | 14 | `scripts/gate/merge-and-verify.sh`（12 处：`:3425` 同名常量与自证里造的假树 `:3730`-`:3738`、`:4355`-`:4407`）；`scripts/mutation/crates-mutation-rows.py:391`（文档串，**进判法单元指纹**：同一份文件的另一处 docstring 改动换过 crates 变异的指纹，下一趟整张表现判，`records/2026-11-11-GPU两段与按需领卡核查.md:154`，所以这一行要与别的会换键的改动放在同一个窗口）；`scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:248`（注释里引测试包的共用模块路径，装置源码，改了装置的代码键会变，E163 的留存产物是否 exact 复跑我没查） |
| `crates/别的包` | 6 | 8 | 产品包 `Cargo.toml:32`、`src/lib.rs:21`、`tests/gpu_opener_of_this_process_gates_a_hung_opening_and_limits_the_instances.rs`、`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:93`、`:112`、`crates/singlefs-unit-graph/src/build_toolchain.rs:544`（单测读测试包的 `build.rs`，要换成产品包的）；`crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:226` 是一个「永不存在的目录」的字面量名字（不是对包的引用，撤包后可以留，也可以改名让搜索归零，改它是改产品源码） |
| 仓根与别处 | 3 | 3 | 根 `Cargo.toml:15` 的 `members`、`Cargo.lock`（重生成）、`CLAUDE.md:7`（一行里的包名） |
| `.claude/` 其余 | 1 | 1 | `.claude/main-agent.md:68` |

`research/prompts` 与 `records` 合起来 850 行、113 个文件（`git grep -n … -- research records`，分类脚本是我写的行级正则，量的）：命令行（`cargo test|run|build|clippy`、`task-server/task submit`、`-p singlefs-background-gpu-tests`）437 行；带 `crates/singlefs-background-gpu-tests/` 路径的非命令行 292 行；只写包名或下划线名的 121 行。文件按名字分：腿的产出 31、材料 29、判决 4、跑前登记 1、记录 16、别的 32。
- **随 `path-moves.md` 改的**：带路径的非命令行 292 行里，指向已经搬到新位置的文件的（测试目标、共用模块、E202 装置）。依据：`.claude/rules/path-moves.md` 开头一句「全仓指向它的地方一次改成新路径，`research/prompts/`、`research/results/`、`records/`、变更史与历史版本节、别的会话的记录都算」。
- **冻结、留旧的**：第 5 步「抄下来的命令和它的输出」「说搬迁这件事本身的那一行」（命令行 437 行与讲搬迁经过的行）。指向**撤掉的包本身**（没有后继路径）的那些，没有「新路径」可改，留旧名。
- **出口三「再搜 0 命中」在字面上做不到**：至少 `.claude/gate.d/path-moves.tsv` 的历史行（12 处）、这一轮自己的材料（13 处）与第 5 步留旧的行会留下；要么出口改成「除 `.claude/gate.d/dead-path-kept.tsv` 登记的与第 5 步留的之外 0 命中」（`.claude/gate.d/dead-path-kept.tsv` 已有，`path-moves.md` 第 5 步点名它），要么逐条登记。我读到的门禁只认治理文档与 kb 中文页（`path-moves.md@门禁管哪一半`），`research/`、`records/` 里留旧的不会被它拦，也不会被它登记。
- **门禁的常量与样本档**：`crash-point-replay.sh:794` 与 `scripts/gate/merge-and-verify.sh:3425` 两处 `PRODUCT_GPU_PACKAGE_NAME` 改成 `singlefs-background-gpu`，`stage-inputs.tsv:31` 改 `tests-of:singlefs-background-gpu`；该格的起法 `cargo test --release --no-fail-fast -p <包> --features tests-on-a-real-gpu --test <目标>…`（`crash-point-replay.sh:271`）不再需要「包带特性才编」的假设之外的改动；样本档 18 个目录 35 个文件里的包名与 `crates/<包>/Cargo.toml` 路径跟着换，每个弄坏开关仍要判红。
- 判：这一格**成立**（分类与改法可落地）；出口三的「0 命中」**推不出**（与第 5 步冲突，要主 agent 与用户定怎么表述）。

更正第 6 节：`.claude/gate.d/fixtures/crash-point-replay.sh/` 下名字以 `product-gpu` 起头的样本目录是 11 个（`ls … \| grep -c "^product-gpu"`，量的），不是 18 个（18 是 `records/2026-11-11-GPU两段与按需领卡核查.md` 当时的数）；35 个文件还含 `last-pass-runs-after-the-other-cells/expect`。第 6 节里「该格的起法……不再需要……之外的改动」那一句写得不清，意思是：起法本身（`cargo test … -p <包> --features tests-on-a-real-gpu --test <目标>…`）不用改，只改 `-p` 后面的包名。

## 7. 防再发：调度记录「没能避免的原因」五条缺口各一条会红的检查（只写判据，不改门禁）

出处：`records/2026-11-19-产品GPU包测试与E202装置归位.md@没能避免的原因`。下面「放哪」都写到门禁与格；新增的登记表 `.claude/gate.d/crate-roles.tsv`（包名、角色、目录、理由，TAB 分隔，角色四选一：产品、验证、测试支撑、装置）是缺口 1、2、4 共用的输入，写在门禁旁（`.claude/rules/directory-layout.md@测试的定义与输入跟着测试放` 末尾说的「只被门禁自己读的配置留在门禁旁边」）。

| 缺口 | 检查（判什么） | 拦哪种写法 | 放哪 |
|---|---|---|---|
| 1 没有规则说「一个包的测试放它自己的 `tests/`」「装置只放装置的位置」「产品包不放装置二进制」 | 逐包读 `crate-roles.tsv` 的角色，对每个 `crates/<包>/`：①角色是产品的包有 `[[bin]]` 或 `src/bin/*.rs`，且名字形如 `e<数字>_` → 红；②角色不是装置也不是验证的包，`src/bin/` 下有 `e<数字>_` 起头的文件 → 红；③包的 `src/lib.rs` 去掉注释与 `#![…]` 之后没有一个条目、而 `tests/` 下有测试目标，且角色不是测试支撑 → 红；④角色是测试支撑的包，被 `crates/*/Cargo.toml`（含 dev）依赖的包数不到 2 个 → 红（「不为单个包的测试另开包」；支撑包、判定库包、替身包各有 2 个以上依赖者，不会误红） | 原因 1 的写法（`crates/singlefs-background-gpu/src/bin/e202_…rs`，产品包里的装置）、原因 3 的写法（空库加测试目标的泛称 `-tests` 包） | 门禁 code-tooling 的新格 `crate-roles`（`.claude/gate.d/code-tooling.sh:38` 的 `directory-layout` 格射程写的是「research/ 与 scripts/ 的顶层、说明文件、层级与不放产物」，名字不含 `crates/`，所以另起一格，判法脚本 `scripts/gate/crate-roles-check.py`；规则同一轮写进 `.claude/rules/directory-layout.md` 新一节，措施 M1、M5 的文字也在那里）。样本：红放 `crates/x-tests/`（空 lib、两个测试目标、没有依赖者）与产品包里一个 `src/bin/e1_x.rs`；绿放支撑包被两个包依赖 |
| 2 没有门禁判 `crates/` 下每个包是什么角色 | ①`crates/*/Cargo.toml` 里每个包名在 `crate-roles.tsv` 里恰有一行，没有的红；②登记的目录不存在红（包搬了登记没跟）；③角色不在四个里红；④扫到 0 个包红（不静默：成功行要报「查了 N 个包」，N 为 0 或与 `crates/` 下的 `Cargo.toml` 数对不上都红，同 `.claude/agent-common.md` 「退出码 77 是本次未跑」那一条的路数）；⑤同一个包名两行红 | 新加一个包而没登记角色（今天新加包没有任何东西拦） | 同一格 `crate-roles`（登记覆盖与角色规矩共用一份脚本，分两个子检查，各带红样本） |
| 3 `#[path]`、`include!` 是 ⑧ 自己写明的盲区 | 扫 `crates/**/*.rs`（含 `build.rs`、`tests/`、`src/bin/`）里 `#[path = "<字面量>"]`、`include!("<字面量>")`、`include_str!("<字面量>")`、`include_bytes!("<字面量>")`，字面量按所在文件的目录解析，落在**另一个** `crates/<包>/` 目录里、且目标后缀是 `.rs`、`.wgsl` → 红；落在别的包但是数据文件（`.toml`、`.md`、`.tsv`）不红、只在成功行计数（E202 摘要读几个包的 `Cargo.toml` 就是这一种，`e202_gpu_cpu_gc_comparison.rs:13832`-`:13852`）；`concat!(env!("OUT_DIR"), …)` 不判。豁免表 `crate-includes-exempt.tsv`，每行点名文件、行、理由 | 今天的两处：`background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11` 的 `#[path = "../../singlefs-verification/tests/…"]`；以及搬完之后会冒出来的 `reference_units_entries_name_functions_that_exist.rs:173` 的 `include_str!("../src/bin/e202…rs")`（E202 搬到装置包之后） | 门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格在 ⑧ 后加一条 ⑨，同一份 Python 里扫源码的 `package_sources`（`checker-independence-and-sync.sh:596` 附近）已经在遍历两边的 `.rs`；并把 `:45` 的「⑧ 罩不到的……`#[path = …]`、`include!`」那一句改成「⑨ 判」。样本：沿用 `checker-implementation-disjoint-gpu-verification-depends-red` 的布局，红放产品包 `tests/x.rs` 里一行 `#[path = "../../singlefs-verification/tests/c/mod.rs"]`，绿放包内 `#[path = "c/mod.rs"]`。范围只管验证包与产品 GPU 包两边（⑧ 的射程），全仓版本由缺口 1 的 `crate-roles` 格按角色扫（支撑包以外的包不许跨包 `#[path]`） |
| 4 代码轮检查单八格里没有「放的位置」 | 代码轮正文 `_<题>-code-r<数字>-body.md` 逐格点名九格：第九格标题「### 检查单：放的位置」，底下非空，且（弱）至少点名一个 `crates/<包>/` 路径与该包在 `crate-roles.tsv` 里的角色；（强）对开工快照与此刻树的差里每个新增或搬动的 `crates/` 文件，它所在的包名在这一格的答案里逐个出现，并写「角色=…、该角色的位置标准=…」 | 只审内容不审放置：两轮代码轮与严格审核都没有这一格（原因 4） | 门禁 doc-process-records 的 `code-round-checklist` 格（`.claude/gate.d/doc-process-records.sh:26`）：八格改九格，格名逐字表在 `.claude/rules/implementation-workflow.md@代码轮正文的检查单`，同一轮改；它自己的自证样本（`code-round-checklist-red`、`-empty-red`、`-answers-unchecked` 开关）各加一格缺第九格的红样本 |
| 5 弹窗问放置类问题时选项没有先对着规则里的位置标准列 | 钩子读 `AskUserQuestion` 的 `question` 与 `options[].description`：含「放哪」「放进」「住进」「搬进」「归位」「位置」「新开包」「测试包」「装置包」之一即判放置类；放置类要求①`question` 里有 `.claude/rules/directory-layout.md` 加「小节名」的引用（位置标准出处）；②每个选项的 `description` 以「标准：」或「偏离：」起头，「偏离：」那句里要引被偏离的那一条规则；③至少一个选项以「标准：」起头。①缺、②有选项没起头、③没有标准选项都退 2 | 原因 4 与 5：弹窗里用户只能在偏离标准的几项里选，选项里写「整份搬进测试包」而没标它偏离哪一条 | 项目钩子，挂 `PreToolUse[AskUserQuestion]`，与 `.claude/hooks/ask-user-claim-guard.sh` 同事件，另起 `ask-user-placement-guard.sh`（那一份文件头写明两边判据没有交集、不并进去）；样本与自证照那一份的写法。不是门禁格 |

- 五条检查的依赖顺序：缺口 2 的登记表先立，缺口 1、4 读它，缺口 3 独立，缺口 5 独立。
- 什么会推翻缺口 1 ④（支撑包依赖者不到 2 个判红）：真有一个测试支撑只有一个依赖者而合理（比如该包的集成测试要的一份大夹具）——那就写成它的 `tests/common_…/mod.rs`，不开包；若仍要开，登记表加一列「例外」并写理由，不改判据。
- 什么会推翻缺口 3 的「数据文件不红」：E202 摘要读的 `Cargo.toml` 若被换成引另一个包的 `.rs` 源码——那一条按后缀判红，不会被放过去。
- 这五条都是推的（没有样本世界跑过）；每条落地时要各配一个会红的样本与一个弄坏开关，由工具员做。

更正第 7 节缺口 3 一行里的位置：扫两边源码的 `package_sources` 在 `.claude/gate.d/checker-independence-and-sync.sh:594`，`references_to` 在 `:597`（不是 `:596` 附近）。

## 8. 各岔路的推荐与代价

净账：F1 甲、F2 甲、F3 丙、F5 的支撑包合起来是新开 4 个包（判定库包、支撑包、替身包、装置包）、撤 1 个（测试包），净加 3 个。想少包就退 F3 乙（替身并进判定库包，净加 2 个）；F1 乙不新开判定库包，但违前提 3、对拍没有合法的家。

| 岔路 | 推荐 | 代价（有数的标出处） | 退路 |
|---|---|---|---|
| F1 判定库存取与算键 | 甲：新判定库包 | 搬动约 10225 非测试行（按方法裁约 5215）；207 行变异路径；⑧ 允许清单加判定库包、`singlefs-unit-graph`、`verus_syn`、`proc-macro2`、`quote`、`prune-server`；**验证包判器键表 91 组里 3 组的键会变，镜像判定与给定状态判定的每一块、观察者结论块全部现判一次**（数量级 10⁹ 态，推的，要先量，1.3） | 乙：键不变，违前提 3，对拍没有合法的家，退成黄金向量 |
| F2 E202 装置 | 甲：`crates/` 下装置包 | 143 行变异路径与 `-p`；kb 三语言页各 3 处加欠账 C1046 三语言；登记 `:479`、`:499`、`:549`、`:551`、`:617`、`:718`、`:732`、`:774`-`:776`；`experiment-number-directories.tsv:17`；根 `Cargo.toml` `members`；E202 摘要范围（登记第 15 条）要不要加支撑包与判定库包是登记修订 | 乙（`scripts/experiment-apparatus/`）：撞第 4 条字面，单元键、变异行、判定库三处重做；丙：撞 ⑧，不可行 |
| F3 CPU 替身 | 丙：只依赖核心层的替身包 | 一个新包 310 行；验证包 8 个目标文件加 1 个共用模块、产品 1 个目标换 `use`；2 行变异路径；⑧ 允许清单加一个名字；措施 M1 的措辞要给「两个以上的包共用的测试支撑」留出口 | 乙：并进判定库包，少一个包，职责混；甲：加核心层特性要动三处，核心层里放测试代码；丁：不成立 |
| F4 单元键在哪算 | 甲：产品包 `build.rs` 接过去；判定库包自己带一组录入键 | 产品包 `build.rs` 23 行变约 240 行；产品包加 `[build-dependencies] singlefs-unit-graph`；登记表 40 行入口路径头一段改名；**GPU 侧 31 组结论上卡重判一次**，参照侧 7 组预计键不变（要量）；所有对拍的录入记录重录一次（CPU）；驱动代码在四个 `build.rs` 里各一份，建议收进 `singlefs-unit-graph`（另立项） | 乙（运行时算）：不成立 |
| F5 共用模块 | 支撑包放取卡模块与对拍通用层，薄壳放包专有的 OUT_DIR 表 | 一个新包；dev 依赖环（要先用 `cycle-probe` 探）；产品测试 11 个目标 40 行、E202 11 行换 `use`；9 个目标不改调用点 | 环不被接受时：支撑包内容进产品包 `src/` 的测试特性（产品包里放测试代码，违「测试就该在测试的地方」），或取卡模块拆成不依赖产品包的两半 |
| F6 引用 | 按 `path-moves.md` 改活文件，历史按第 5 步留旧 | 191 文件 1274 处里：活的约 70 个文件（测试包自己 9、mutations 1、`.claude` 的 gate.d 8 与 kb 12 与 main-agent 1、scripts 3、crates 别的包 6、根 3、样本档 35 个文件）；`research/prompts` + `records` 113 个文件 850 行，其中命令行 437 行留旧、带路径 292 行改、121 行只写包名；`crates-mutation-rows.py:391` 的文档串会换 crates 变异的判法指纹，与别的换键改动同一窗口做 | 出口三「0 命中」改成「除登记留旧外 0 命中」 |
| F7 防再发 | 五条检查（第 7 节） | 新登记 `crate-roles.tsv`、新格 `crate-roles`、⑧ 加 ⑨、`code-round-checklist` 八格改九格、新钩子 `ask-user-placement-guard.sh`、规则 `directory-layout.md` 新一节 | 缺口 4 只做弱版（不读快照差） |

撞到「取卡」的地方（这一轮不改取卡）：支撑包里的取卡模块仍按 `GPU_CARDS_BY_PRIORITY` 当任务服务交来的卡（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:42` 起，`task-server/task_server_lib.py:2029`），产品测试与装置包的取卡语义不变；按需领卡（`gpu_card_configuration.rs` 的 `cards_handed_over_or_leased_on_demand`）搬进判定库包时，它读 `gpu-verification-server/control` 的路径常量（`CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY`）不变。「卡归 GPU 服务代跑（接口待定）」这一条与拆包无关，不在这里改。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 F1 甲（新判定库包） | 成立 | 判器与崩溃放量 0 行被拖出（条目口径）；代价最大的是判器键 3/91 组换键 |
| 1 F1 乙（产品测试自写） | 不成立 | 撞前提 3，逐字节对拍没有合法的家 |
| 1 F1 第三条路 | 推不出 | 搬进 prune-server 或 unit-graph、走 Python 客户端、分两层都不成立或不更好 |
| 1 ⑧ 理由在甲下 | 字面不成立、共因风险成立 | 新包没判器，但崩溃判定与对拍共用同一份按键复用，错了一起错 |
| 1 ⑧ 允许清单 | 要加 | 新包、`singlefs-unit-graph`、`verus_syn`、`proc-macro2`、`quote`、`prune-server`（F3 丙再加替身包） |
| 1 另判新包不含判器 | 要 | 判据见第 7 格缺口 3 与 1.7 |
| 2 F2 甲（装置包） | 成立 | 不撞 ⑧、第 4 条、`directory-layout.md`；推荐 |
| 2 F2 乙（`scripts/experiment-apparatus/`） | 不成立 | 撞第 4 条字面，单元键、变异行、判定库三处重做 |
| 2 F2 丙（验证包 `src/bin/`） | 不成立 | 验证包依赖产品包，⑧ 判红 |
| 3 F3 甲/乙/丙/丁 | 不成立（不推荐）/ 成立 / 成立（推荐）/ 不成立 | 丁是跨包 `#[path]`，落在 ⑧ 盲区；甲要改核心层三处 |
| 4 F4 甲 | 成立 | 产品包 `build.rs` 接过去；GPU 侧 31 组重判，参照侧 7 组预计不变 |
| 4 F4 乙 | 不成立 | 键编进二进制、跑时不读树 |
| 5 共用模块 | 成立 | 取卡与对拍通用层进支撑包，其余留产品包；没有跨包 `#[path]`；dev 依赖环待探 |
| 6 引用分类 | 成立；出口三「0 命中」推不出 | 191 文件 1274 处，历史留旧与出口三冲突 |
| 7 防再发 | 成立（判据已写，没跑过样本） | 五条各有放哪道门禁哪一格 |

## 要主 agent 跑的命令

1. 复跑模型（四种闭包、共用模块计数、引用分类、判器键表里要搬的单元）。在仓根起，不套内存包装；探针只读 `crates/` 源码与一份 63 MB 的旧键表，内存与时间没量（估内存不到 1 GiB、几十秒，推的）。没有冻结副本就像下面这样把仓根当 `--frozen`。
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/gpu-tests-placement-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-forward-rerun-run /home/fy5090/code/singlefs/research/prompts/gpu-tests-placement-design-r1-sonnet-model`
   看：`TASK_SERVER_RESULT` 行 `type=succeeded exit=0`；`task-server/task log <号>` 里应见 `name=module_level_closure modules=26 lines=100868`、`name=item-closure-whole-impl-with-flows TOTAL kept lines 13689`、`name=item-closure-whole-impl-flows-cut TOTAL kept lines 10225`、`name=item-closure-by-method-flows-cut TOTAL kept lines 5215`、`name=direct_verification_targets count=8`、`module=common_discrete_gpus_of_this_host count=11`、`name=refs_total files=191 hits=1274`（树被别的会话改动时这几个数会动，以复跑为准）、`group=gpu_judge`、`group=gpu_facts_verifier`、`group=observer_conclusion_composition` 三行（`path=none` 表示 `target/` 里没有验证包的构建产物，不算过）。
2. 探 dev 依赖环能不能编过（第 5 节）。在草稿目录里的小工作区起（两个只有几行的包，没有外部依赖）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=pa 的测试目标 it 里的 1 条测试; 范围=1 个测试; 为什么=探一个包的 dev 依赖指回包自己的环 cargo 接不接受' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-forward/cycle-probe -- cargo test -p pa --features tests-on-a-real-thing --test it -- the_support_crate_that_depends_on_the_package_under_test_sees_the_same_package --exact`
   看：`TASK_SERVER_RESULT` 末行 `exit=0` 且日志里 `test result: ok. 1 passed`；报「cyclic package dependency」之类就是 cargo 不接受，F5 的支撑包路线要换（第 8 节退路）。
3. 基线键表（F1 的 1.3 与 F4 的 4.2）：在仓根（基线树）里让两个包的构建脚本各跑一次，编译目录由服务给。
   `task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo check --release -p singlefs-verification -p singlefs-background-gpu-tests`
   看：`TASK_SERVER_RESULT` 末行 `build=<目录>` 与 `exit=0`；然后（只读命令，主 agent 直接跑）`ls -t <目录>/release/build/singlefs-verification-*/out/judging_units_keys.tsv`、`ls -t <目录>/release/build/singlefs-background-gpu-tests-*/out/reference_units_keys.tsv`、同目录的 `gpu_side_units_keys.tsv`，各取 `grep -P '^group\t' <表> | cut -f2-4` 存成「前」。拆完以后在补丁打上的树里对产品包与验证包同样跑 `cargo check --release -p singlefs-verification -p singlefs-background-gpu`，存成「后」，`diff` 前后：判器键表里 `gpu_judge` 等三组应当变、参照侧 7 组应当不变、GPU 侧 31 组应当全变；与此不符的逐条点名。（拆完的树现在还没有，这一条等实现员的补丁。）
4. 读引用数是只读 `git` 命令，主 agent 可直接跑：`git -c core.quotepath=false grep -c "singlefs-background-gpu-tests\|singlefs_background_gpu_tests"`，与 `rerun.sh` 的 `refs-classes.txt` 对。

## 没做什么

- 没起任何任务、没跑 `rerun.sh`、没跑门禁与测试；闭包数、引用分类、键表单元数是我在草稿目录里用同一份脚本的副本直接跑的只读分析（不经服务），判决引用前按「要主 agent 跑的命令」第 1 条复跑。
- 没读 `crates/singlefs-unit-graph` 对环与对 `pub use` 转出的收法，所以「判器键会变」「参照侧 7 组不变」「dev 依赖环可行」都是推的。
- 没读 crates 变异门禁的复制与起法，所以乙那一格里 `-p` 点不到包外装置包是推的。
- 没读 E202 登记（`research/prompts/e202-preregistration.md`）全文，只读了含装置路径的行；摘要范围（第 15 条）要不要加支撑包与判定库包没判。
- 没判别的腿的格：没替主 agent 采纳或出判决；攻方腿这一轮的产出（禁读清单里的文件）没读。
- 没改仓里任何文件：只写了本报告与模型目录 `research/prompts/gpu-tests-placement-design-r1-sonnet-model/`。别的会话的编辑中间文件 `.background_compute_settings_and_switch.rs.lhuq544s.replacing` 没动、没追。
- 交回前删的草稿：草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-forward` 里没有仓副本与编译目录；`cycle-probe` 在 `.deliverables` 声明成交接，其余是分析脚本的中间文件，776K。
