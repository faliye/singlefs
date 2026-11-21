# 产品 GPU 包的测试与实验装置归位（第二轮）

方案按 `crates/` 今天的实现来谈。标「量的」的是主 agent 跑命令得到的，标「读码」的是主 agent 读源码得到的，标「推的」的没量过。调度记录在 `records/2026-11-19-产品GPU包测试与E202装置归位.md`；第一轮判决在 `research/prompts/gpu-tests-placement-design-r1-main-verification.md`。

## 问题

第一轮的四条岔路已由用户定案（判决「二、判决」第 1–4 条）。这一轮推定案之后怎么落地：产品 GPU 包的测试回 `crates/singlefs-background-gpu/tests/`、验证包 `src/bin/` 下的装置与 E202 进新开的实验包、测试包 `crates/singlefs-background-gpu-tests` 撤掉，每一步写到文件与函数。不重议定案。

## 已定的前提（用户原话与定案，不在这一轮里重议）

1. 「按照标准的要求严格的来。 测试就该在测试的地方   装置就该在装置的地方。 干净的拆开解决问题。」
2. F1「11-11 说的「不要做第二套或者第三套东西」 这个不能违背不公用的前提。如果有冲突删掉。」：产品 GPU 包的测试自己写存取代码，不依赖验证包；录入键照同一套算法、不许特例（第一轮判决第 1 条）；「互不依赖 代码可以不同，但是输出的键值要一样。」（`records/2026-11-11-GPU两段与按需领卡核查.md:141`）
3. F2「按照和 E185 这些装置放在一起。 这是验证包还是实验包，如果是实验包依赖产品代码很正常。 如果checker 那么绝对不行」：新开实验包；验证包 `src/bin/` 下的装置这一轮一起搬进去（弹窗定）；`.claude/rules/verification.md`「定义与名字」验证包那一行跟着改。
4. F3「按照其他流的方式走 如果没有就创建 类似 singlefs-verification中有相应的东西吧」：产品 GPU 包在自己的 `tests/` 下另建 CPU 替身。
5. F4「singlefs-unit-graph应该是共用的 为什么会有额外的？」：产品 GPU 包的 `build.rs` 经 `singlefs-unit-graph` 算对拍两侧的键；⑧ 的允许共用清单加 `singlefs-unit-graph` 与 `prune-server`（弹窗定「两个都加」），另判这两个不依赖 wgpu、不含判器。
6. 不跨包 `#[path]`、`include!` 引另一个包目录下的源码（措施 M3 要立成门禁）。

## 压着这件事的条款

- 门禁 checker-independence-and-sync 的 ⑧（`.claude/gate.d/checker-independence-and-sync.sh:38-40`）。
- D13（验证路线） 已定项 17（单元键由什么组成，路径、名字不进键）。
- `.claude/rules/verification.md`「定义与名字」「判定库只有一份」「卡只归 GPU 服务，任务不拿卡」。
- `.claude/rules/implementation-first.md` 第 4 条；`.claude/rules/directory-layout.md`；`.claude/rules/path-moves.md`「怎么做」。

## 实现今天的样子

主 agent 的观测（量的）：

- 验证包 `src/bin/` 26 个装置二进制（`Cargo.toml` 26 条 `[[bin]]`），每一份都引验证包自己的模块，2 到 56 处（逐份 `grep -c '(crate|singlefs_verification)::'`）。6 份经 `#[path]` 引验证包 `tests/` 下的共用模块：e161、e170、e171 两版、e185 引 `tests/common/mod.rs`（1416 行，验证包 340 个测试目标 `mod common;`），e184 另引 `common_crash_point_observers`、`common_crash_points`、`common_snapshot_prefix`。
- 不带实验号的 5 份与点名它们的地方（`git grep -l`）：
  - `crash_library_reader`：`scripts/crash/crash-amplification.sh`、`crash-point-replay.sh`、`crash-library-totals.sh`、`lib_crash_point_replay_items.py`、`scripts/gate/merge-and-verify.sh`、`.claude/gate.d/verdict-library-scripts.tsv`。
  - `new_pool_file_creation_on_device`、`new_pool_file_creation_device_log_check`：门禁 `.claude/gate.d/qemu-device-streams.sh` 与它的 12 个样本档目录；前者另被 E152 的 `scripts/devices/e152-run.sh`、`e152-stage-root.sh`、`scripts/experiment-apparatus/e152_file_system_benchmark.rs` 点名。
  - `deletion_write_dump`：`scripts/experiment-apparatus/e172_deletion_write_dry_run.rs`、`replay.sh`、`scripts/experiment-apparatus/Cargo.toml`、`.claude/gate.d/stage-inputs.tsv`。
  - `new_pool_file_creation_region_bytes`：`scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs`、`replay.sh`。
- E185 参照一侧的判法单元键取验证包构建脚本编进来的组 `e185_cpu_reference_crcs`（`crates/singlefs-implementation-experiments/src/bin/e185_gc_batch_wall_clock_split/main.rs:520`、`:803-810` 经 `singlefs_verification::crash_identity::judging_units_key`）；组的入口登记在验证包 `src/judging_units_entries.rs`（读码，没逐组数哪几组的目标是 `bin:`）。
- E202 从测试包的两个共用模块用到：`common_discrete_gpus_of_this_host` 的 `GpusThisProcessIsOn`、`gpus_this_process_is_on_judged_against_the_handed_over`、`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`；`common_pairing_input_through_the_library` 的 `pairing_concluded_on_the_cards_through_the_pairing_library`（`e202_gpu_cpu_gc_comparison.rs` 第 150–165 行起的 `#[path]` 与 `grep -oE` 计数）。
- 产品 GPU 包测试要的验证包东西（第一轮攻方腿报全包 24 个名字，`research/prompts/gpu-tests-placement-design-r1-opus-output.md` 「实现今天的样子」那一节）：录入与读回（`crash_library` 的 pairing 一族）、对拍结论入库（`pairing_conclusion_reused_or_judged`、`verdict_store::PairingAgreement`）、键（`crash_identity::JudgingUnitsKey`、`sha256`）、按需领卡（`gpu_card_configuration::cards_handed_over_or_leased_on_demand`）。
- `pci_bus_identifier_without_the_domain` 两份函数体逐字相同：测试包 `tests/common_discrete_gpus_of_this_host/mod.rs:206-209` 与验证包 `src/gpu_memory_ledger.rs:780-783`；前者第 204 行写「本包测试不依赖 验证包，自己留一份」，今天这句不对（测试包直接依赖验证包）。
- 判定库客户端在 `prune-server/src/lib.rs`（`prune_server::reliable_client`）；验证包经它读写判定库（`crates/singlefs-verification/Cargo.toml:33`）。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；写做法的写成对实现的改动。

1. **两侧录入键逐字节相同怎么证**：产品测试自写的录入与读回、对拍结论入库，与验证包那一套（`crash_library.rs` 的 `recording_key_of_the_inputs`、`verdict_store.rs` 第 `19` 号表与「对拍输入」表 `1C`）不共用代码。给出至少两种证法（例：两侧各钉一组同输入同字节的黄金值；一道门禁格两侧各算一次再比），各写它拦得住什么、拦不住什么，以及键的哪几段按定义就该两侧不同（录入函数单元指纹是各自那一份的指纹）。答这一套算不算「另开判定存放处」（`.claude/rules/verification.md`「判定库只有一份」）：写进的是同一份库、经同一个客户端。
2. **产品 GPU 包测试要的存取代码有多大、放哪**：照 `use` 链列出产品测试真要的最小一组（录入、读回、对拍结论、按需领卡），写成产品包 `tests/common_<名>/mod.rs` 的哪几份、各多少行（推的写明推的）；`[dev-dependencies]` 加什么（`prune-server`，还有没有别的）；按需领卡那一段在产品测试里怎么写（读 `gpu-verification-server/control` 的那一层），撞不撞「卡只归 GPU 服务，任务不拿卡」。
3. **实验包的接法**：包名、`Cargo.toml`（依赖验证包、产品 GPU 包、核心层；`gpu` 特性与验证包那一份怎么接）、`build.rs`（E185、E202 这类参照一侧的单元键今天由验证包构建脚本或测试包构建脚本编进去，搬过来之后入口登记表与算键在哪）；它在 ⑧ 下是什么（依赖两边，两边仍互不依赖）。
4. **26 个装置怎么分**：逐份判留验证包还是进实验包。判据先写死再逐份套：判据候选「门禁与崩溃入口自己起的工具（判定库读数、真设备流）留验证包；实验页与 `replay.sh` 登记表点名的装置进实验包」。给出逐份表（名字、谁点名它、去处）。
5. **装置用到的验证包 `tests/` 共用模块从哪来**：6 份经 `#[path]` 引 `tests/common/mod.rs` 等。候选：甲，把它们用到的那几样挪进验证包 `src/` 的公开模块；乙，实验包自己写一份（各写各的）；丙，别的。逐个答撞不撞前提第 6 条、`.claude/rules/verification.md`「定义与名字」、单元键（`tests/common/mod.rs` 被 340 个测试目标用，挪动它会不会换这些测试的键）。
6. **E202 的取卡与对拍那一层从哪来**：E202 进实验包，可以依赖验证包；它今天用测试包的两个共用模块（上一节列了用到的四样）。各写各的还是改用验证包已有的（`gpu_card_configuration`、`crash_library` 的对拍函数）；产品测试那一份取卡模块与 E202 那一份是不是两份、各自的测试住哪。
7. **`pci_bus_identifier_without_the_domain` 两份**：在前提第 2 条下，产品测试那一份是合法的各写各的，还是该另有一处；第 204 行那句注释改成什么。
8. **单元键与现判的量**：搬包之后哪几组判定的键会变、要现判多少（产品测试 GPU 侧与参照侧各几组，实验包里各装置的组，`crates/mutations.tsv` 改了行文本的行，`scripts/mutation/crates-mutation-rows.py:391` 的文档串换判法指纹）；写出主 agent 量它们的命令（`--list-items --with-keys` 一类）。
9. **措施的判据定稿**：第一轮判决「二、判决」第 7 条采用的五条检查，按这一轮的角色（产品、验证、测试支撑、实验装置）与第 4 格的分法重写判据；`crate-roles.tsv` 的初始行逐包列出；⑨（跨包 `#[path]`、`include!` 判红）对今天全仓已有的 16 处 `#[path]` 与 6 处跨包 `include_str!`（`grep -rnE` 量的）逐处判红还是放行、放行的理由。

## 分工

- 正推腿：逐格给出能落地的做法，每一步写到文件与函数；第 2、4、8 格的表与数照命令现量，命令写进报告。
- 攻方腿：攻第 1、3、5、6 格——两侧录入键的证法哪一种其实证不出逐字节相同；实验包的接法有没有让 ⑧ 失去意义（实验包依赖两边，会不会变成产品包经实验包间接用到验证包的代码）；共用模块的各候选里有没有藏着的跨包引、抄两份而不是各写各的、或换了 340 个测试的键；E202 那一层有没有绕回测试包的形状。再攻「实现今天的样子」里主 agent 写下的每一句全称句。不重复第一轮已攻过的角度（拆不拆得出不含判器的包、F2 三个候选撞哪条）。
