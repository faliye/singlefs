# product-test-pairing-keys-design-r1：判决（2026-11-19）

正文 `research/prompts/_product-test-pairing-keys-design-r1-body.md`，开工快照 `research/prompts/product-test-pairing-keys-design-r1-snapshot/`。这一题的第一轮。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md`。被判文件快照 10 份派腿之前记，写判决前 `sha256sum -c --quiet` 退 0。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/product-test-pairing-keys-design-r1-sonnet-output.md`（sha256 d795cf29…） | 8 个候选，推荐丙（测试支撑包的构建脚本算键、产品包测试经 dev 依赖读）；甲、乙撞前提 4；13 个目标里 8 个以核心层或产品函数为参照；独立参照 CRC 约 40–60 行、活槽 30–50 行、区域分类 250–400 行（推的）；防再发六条 |
| 攻方（opus） | `research/prompts/product-test-pairing-keys-design-r1-opus-output.md`（sha256 8a2f0922…） | 正文「整包 385 条都过」说宽了；D26（后台整理与放置回收） 已定项 13「两份实现在测试里对拍」，独立参照该加不该换；真卡 CRC 只取到 4096 字节；显卡驱动与卡型、`--config` profile 不进键；区域分类比到拒因那一层只能照抄产品检查次序 |

- `sha256sum -c --quiet research/prompts/product-test-pairing-keys-design-r1-snapshot/judged-sha256.txt` 退 0，被判文件没改过，直接在主工作区核。
- `python3 scripts/agents/cite-check.py research/prompts/product-test-pairing-keys-design-r1-opus-output.md research/prompts/product-test-pairing-keys-design-r1-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_product-test-pairing-keys-design-r1-background.md`：「核了 69 处引文、99 处裸行号，对上 66 处，没判 102 处」。没判的多是拆包补丁快照里的路径，不在主工作区。
- 主 agent 现核坐实：D26（后台整理与放置回收） 已定项 13 原文「两份实现在测试里对拍」（`.claude/kb/decisions/26-后台整理与放置回收.md:261`）；产品的 CPU 参照是运行时裁决用的产品代码（`checksum_kernels.rs:256`、`live_slot_kernels.rs:527`、`region_record_kernels.rs:1421`）；真卡 CRC 取的长度是 0、1、7、8、9、4095、4096（`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:15`、`:48`），槽与索引节点是 16384 字节（`crates/singlefs-format/src/` 的 `SLOT_BYTES`、`NODE_BYTES`）；「整包 385 条」那一趟没带特性、5 个真卡目标没编。
- 腿报的「显卡驱动与卡型不进键」「`--config` 与 `.cargo/config.toml` 的 profile 不进键」「区域分类拒因次序只能照抄」三条是读码推的，主 agent 没有复跑，判决里当线索、交实现段核。

## 二、判决

用户在弹窗里定了两条（调度记录「用户问答（弹窗）」那一条，原话在那里）：

1. **对拍键：丙。** 另开一个测试支撑 crate，它的构建脚本经 `singlefs-unit-graph` 算对拍两侧的键、写进 `OUT_DIR`，产品包测试经 `[dev-dependencies]` 读；产品包的 `build.rs` 与 `[dependencies]`、`[build-dependencies]` 里不出现单元图与这个 crate，产品构建不编测试代码。用户原话：「只有在测试时候才有复用的概念。 因此这是测试功能 根本就不是产品功能。」这个 crate 要有两个依赖者（`.claude/rules/directory-layout.md`「crates/：各包放什么」测试支撑那一条），第二个是实验包里的 E202（它也要这类键）。入口登记表改成数据文件，由这个 crate 的构建脚本读，不经 `#[path]` 编别的包的 `.rs`。
2. **参照：加，三方比。** 产品 GPU 与产品 CPU 两份实现之间的对拍照 D26（后台整理与放置回收） 已定项 13 留着；另在产品包测试里写不出自产品的独立参照（CRC32C、每段活槽数、区域记录分类）判对错。区域分类那一份只比「收不收」与收下后的字段，拒因只比类别、不照抄产品的检查次序（攻方第 6 格采）。真卡 CRC 取样补 16384、32768 字节两档。
3. **攻方推的另两条（零轮）**：显卡驱动与卡型进不进对拍结论的键、`--config` profile 进不进键，交实现员读码核实：进不了键而会被跨驱动复用的，结论键加上 GPU 一侧的适配器信息（`wgpu` 适配器的驱动与设备名），量不到的写明。
4. **防再发**：产品包任何依赖表（含 `[build-dependencies]`）出现测试支撑角色的 crate 判红；测试支撑 crate 出现在产品包的 `[dependencies]` 或 `[build-dependencies]` 判红；放进 code-tooling 的 crate-roles 格，交治理工具员。
5. 正文「整包 385 条都过」那一句说宽了：那一趟不带特性，真卡目标没编，调度记录已更正。

这一轮的两条岔路由用户定，不再开第二、三轮；返工合入之后照代码轮三方攻。

## 三、改法

- 改法第 1–4 条都是零轮，交实现员与治理工具员；写进产品代码的只有测试，产品 `src/` 不改，不做 A/B。
A/B 读数：任务 <号>

## 交用户的

两条岔路用户已在弹窗里定了（调度记录）。

## 这一轮被判过的文件

- crates/singlefs-background-gpu/Cargo.toml：判了它的 `[build-dependencies]` 不再有单元图，测试支撑 crate 只进 `[dev-dependencies]`
- crates/singlefs-background-gpu/build.rs：判了它不算测试的键，回到拆包之前只交特性的那一版
- crates/singlefs-background-gpu/src/checksum_kernels.rs：判了 `cpu_reference_outcome` 是产品运行时裁决代码，测试另写独立 CRC32C 参照
- crates/singlefs-background-gpu/src/live_slot_kernels.rs：判了 `cpu_reference_live_slots` 是产品运行时裁决代码，测试另写独立活槽数参照
- crates/singlefs-background-gpu/src/region_record_kernels.rs：判了 `cpu_reference_classification` 是产品运行时裁决代码，测试另写独立分类参照，只比收不收与字段
- crates/singlefs-background-gpu/src/background_gpu_compute.rs：判了 CPU 替身对拍的另一边落在产品与核心层代码上，随独立参照一并补
- crates/singlefs-verification/src/crash_identity.rs：判了第 804 行「跑时不读盘上的树」是那段代码对自己的描述，键只在构建时算这一约束仍成立
- crates/singlefs-verification/build.rs：判了它的算键做法是测试支撑 crate 构建脚本照着写的样子，验证包这一份不动
- crates/singlefs-unit-graph/src/lib.rs：判了单元图只进测试那一侧（测试支撑 crate 的构建依赖），不进产品包
- .claude/rules/fs-design.md：判了第 22 行「审计与被审计用同一段代码」被禁止，落成产品包测试的独立参照

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-19 不受影响：两份实现在测试里对拍照留，另加测试侧独立参照 |
| D13（验证路线） 已定项 5 | 支撑 | 2026-11-19 不受影响：判对错的一方不与产品共用代码，落到产品包测试 |
