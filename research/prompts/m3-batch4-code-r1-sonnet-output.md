# m3-batch4-code-r1 正推腿（Sonnet）报告：C1、C3、C4、C6（2026-10-02）

<!-- doc-lint:not-numbers C1 C2 C3 C4 C5 C6 R-独 R-换 -->

## 〇、前提与做法

- 读的是冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`（`crates/`、`.claude/kb/`）。下面的行号都取自它；报告里引到的 21 份文件，逐份比过 sha256，除 `crates/mutations.tsv` 外主工作区那份与冻结副本相同。`crates/mutations.tsv` 主工作区那份已不同，所以表里的行只写「表第 N 行」，不与引文并排。cite-check 以 `--root` 指向冻结副本跑。
- 判据取背景正文第五节。「一致 / 冲突 / 规则没说」逐格各报各的；冲突的两句并排抄。
- 复跑做在草稿副本里（冻结副本整份拷贝，开跑前后 `diff -rq crates` 无差）：只跑 harness 档，`prove-red.sh` 与自写探针经 `run-with-memory-cap.sh` 与 `capped.sh`（线程 5 至 7）。checker 档（`singlefs-checker-tier` 测试、层 0、GPU 对拍、变异整表）一条没跑，要的数写在「要主 agent 提交时跑的命令」。
- 探针与脚本放在 `research/prompts/m3-batch4-code-r1-sonnet-model/`：`drive.py`（探针定义）、`probe.py`、`combo.py`、`pr.sh`、`probe-tests-appended-to-checker_known_bad_images.rs`（三条自写探针用例）、`logs/`（每次运行的原样日志）。
- 这些探针是我提的新形态，按判据算零轮；「没打中」只抽了一次样。

## 一、总判（各格一句）

| 格 | 结论 |
|---|---|
| C1 | 16 条新立里 12 条判法与 kb 那一行逐句对得上，3 条在 kb 没写的地方自加了不适用或收窄口径（I-9.25、I-4.1、I-8.5），**1 条冲突**（I-8.1 的 F 格，checker 与 core 在同一张镜像上分开，命令实测）；4 处改写 checker 一侧都与 kb 对得上。 |
| C3 | 128 位两侧位序、偏移、字数逐位对得上，没有落进没人读的字；**但第 49 至 64 位（16 条新不变量）GPU 内核根本没有判**，位恒 0，层 0 的 GPU 路径对它们没有覆盖。判法摘要 v4 生产路径不读，真正的键是判法闭包摘要，旧判定不沿用。 |
| C4 | mkfs、C462 与条款对得上；I-8.1 core 同判按字面是 core 对着条款；C581 的 core 一侧只覆盖 3 类读者（journal 新根段、码 2 内部条目、根记录），checker 判 4 个入口，**同一张镜像上分开**（命令实测）；C463 只做了只读挂载那一半。 |
| C6 | 本批独立的变异行 54 条（追加 48、整行替换 6 加 1 条追加行改锚点）。能在 harness 档复跑的 45 条：43 条抓到，**2 条没红**（I-7.16 两行）。checker 档 9 条只能读：**1 条（违例位内核写出，表第 1661 行）按读法在它点名的测试上不会红**。另在 31 个弱化点上做探针：5 个被行点名的测试抓到，26 个存活，其中 4 个被别的用例顺带抓到，**22 个连整套 harness（跑过的 19 个测试二进制）都不红**（盲区）。 |

推翻它们的现象：见各节「推翻」一行。

## 二、C1 逐条

候选集、不适用、读不出的处置按代码读出，同 kb 那一行逐句对。16 条新立的下标是 `IMPLEMENTED_INVARIANTS` 的 49 至 64：

```
python3 - <<'EOF'
import re
s=open('crates/singlefs-checker/src/image.rs',encoding='utf-8').read()
m=re.search(r'IMPLEMENTED_INVARIANTS: \[&str; 65\] = \[(.*?)\];',s,re.S)
items=[x.strip().rstrip(',') for x in m.group(1).split('\n') if x.strip()]
print(len(items), items.index('"I-9.16"'), items[49], items[64])
EOF
65 48 POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
```
（工作目录是冻结副本根；输出依次：条数 65；`"I-9.16"` 的下标 48；下标 49 与 64 的两项。）清单声明在 `crates/singlefs-checker/src/image.rs:72`，「只往末尾追加」，前 49 条下标没动。

| 条 | 判法落点与口径 | 与 kb 那一行 |
|---|---|---|
| I-1.12 | 只在两处调用：实例表链上每一片（`crates/singlefs-checker/src/walk.rs:2025`）与 extent 数据单元（`crates/singlefs-checker/src/walk.rs:2190`）；码 1、码 3 取头偏移 43，码 2 不判，没有对象时不适用。 | 一致。kb 写「任一被走读的指针」，而 inode 叶容器指针按 kb 自己的话归 I-9.2，不重判。 |
| I-8.10 | 逐盘按 (实例, txg) 分组，带末条标志恰一条才判；与根环里同 (实例, txg) 的每条自证根比四段；一对都没比到报不适用。四段偏移 0..86、86..172、172..180、180..188 对根 36..122、256..342、122..130、130..138，与 kb 逐项相同。 | 一致。 |
| I-8.11 | 点名项每条位置条目，整单元校验和相符才比，不符不判；偏移 28 / 29 / 37 / 45 与 kb 一致（`crates/singlefs-checker/src/lib.rs:815`、`crates/singlefs-checker/src/lib.rs:818`）；出生树只判码 1、码 3；一条都没比到报不适用。 | 一致。 |
| I-4.9 | 候选集 = 走读里 `is_newest == false` 的候选根（`crates/singlefs-checker/src/walk.rs:6784`），另在「别的版本先走过的单元」上补判一处（`crates/singlefs-checker/src/walk.rs:749`）；候选集只剩最新根时报不适用。 | 一致，kb 行 184 写了这一句不适用。 |
| I-9.25 | 每个走到的版本（含记录施加出来的）判一次；树表单元被几版共享时只在第一个走到它的版本上判（`crates/singlefs-checker/src/walk.rs:1452` 的注释自认「判得更松」）。 | kb 行 311 写「走读到的任一根（候选集里每条）」：**规则没说**共享单元只判一次、也没说记录施加的版本算进来；推的：共享的树表单元里 p 落在 (旧根 txg, 新根 txg] 的镜像，旧根那一格漏判。 |
| I-3.13 | 只判走全了的记账树；版本含记录施加出来的。 | 一致，kb 行 155 明写「以及由记录施加出来的那几版」。 |
| I-9.26 | 偏移 8、88、73（`crates/singlefs-checker/src/walk.rs:150`）；73 是码 3 头里的块诞生代号（`crates/singlefs-checker/src/walk.rs:516` 起那一支），不是偏移 61 的容器出生代。 | 一致，kb 写「这片容器头的诞生代号（偏移 73）」。 |
| I-3.12 | 每条自证根各沿自己的实例表链；链上有片读不出的根跳过；一行都没判到报不适用。 | 一致。 |
| I-7.16 | 奇偶一格、差值一格。差值格的式子在 `crates/singlefs-checker/src/image.rs:772`（两槽都是 mkfs 写的 1，或相差恰为 1，二者取或），与 kb 行 70 的「两个槽都可择时世代号相差不超过 1，相等时只许都是 1」等价；core 写槽也是世代号对 2 取余（`crates/singlefs-core/src/transaction.rs:414`）。 | 一致。 |
| I-7.17 | 第 0 代只判槽 0；其余按 txg 取余算区域与槽（`crates/singlefs-checker/src/image.rs:805`），公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs:114`、`crates/singlefs-core/src/root_ring.rs:115`）。 | 一致。 |
| I-8.12 | 逐盘、逐实例按计数器看 txg 不减，换实例不判；没有可比的一对时不适用。 | 一致。 |
| I-7.15 | 各盘所选槽的设备数同 `chosen[0]` 比（`crates/singlefs-checker/src/walk.rs:6528`）。 | 一致。 |
| I-7.5 | 判定宽度 = 各盘所选槽 max(pbs, io_min) 里最大的（`crates/singlefs-checker/src/image.rs:892` 是那个 `.max()`），逐槽判起始偏移（`crates/singlefs-checker/src/image.rs:905`）。 | 一致（单一几何）。异构池各盘取最大这一点，kb 行 59 没写；只有 C47 对 physical_block_size 写了「mkfs 要按池内 max(physical_block_size) 划」(`.claude/kb/checks-owed.md:48`)，对 io_min 与 checker 取最大的聚合没写：**规则没说**，且没有测试盯着（C6 探针 J1）。 |
| I-4.1 | R-独：自己复刻 D23 已定项 14 的前缀规则（`crates/singlefs-checker/src/walk.rs:6372` 起），施加出来那一版另起一份走读，走读失败判红。**四处报不适用**：锚点认不定（行 6388）、所选根 txg 无后继（行 6392）、前缀停在点名单元验不过（行 6396）、一次都不施加（行 6402）。 | 起点含介质损坏报不适用，kb 行 176 写了；另三处 kb 没写，**规则没说**。其中「一次都不施加」是干净镜像上的常态：I-4.1 在没有记录压在最新根之上的镜像上永远是不适用，不会「成立」。 |
| I-8.1 | 每块盘每个可择槽（`SystemConfigurationSlotReading::Usable`，`crates/singlefs-checker/src/image.rs:830`）三格：F ≥ 2（行 838）、在飞上限等式、最坏占用等式；另一格读环：同 (盘, 实例, txg) 的记录数不多于**所选槽**记的上限（`crates/singlefs-checker/src/walk.rs:6145`）。 | **冲突**（F 格，见下）；另 kb 写「每个自证过的系统配置槽」，代码只判「可择」的槽（自证过而 incompat 位不认得、或 R / S 越界的槽不判），**规则没说**。 |
| I-8.5 | 全池没有一个槽自证过判红（`crates/singlefs-checker/src/image.rs:933`）；`NotChoosable` 拆成「不自证」与「自证过而不可择」两类，后一类计入自证过（行 921 起）。旁路报告 `system_configuration_redundancy_report`（行 965）不进违例通道。 | 与 kb 行 94「magic 与整槽校验和对」逐字对得上；越界值整池拒那一支照旧整池不适用、I-8.5 不判，kb 没写，**规则没说**。 |
| I-1.2 收窄 | 实例表链每一片按指针比出生身份（`crates/singlefs-checker/src/walk.rs:585` 抽出的函数），不进 I-4.2 / I-4.9。 | 一致。 |
| I-3.9 / B7 | 比最后引用它的那条根晚、又走不全的候选根才让这一条已释放记录跳过（`crates/singlefs-checker/src/walk.rs:3506`），跳过数大于 0 时标「部分没判」。 | 一致，kb 行 151。 |
| I-7.8 最新根 | 另判最新根水位大于全池最大树 ID（`crates/singlefs-checker/src/walk.rs:7183`）。core 读路径是「if entry.tree.0 >= root.tree_identifier_watermark {」(`crates/singlefs-core/src/recovery.rs:3617`)，水位要大于树 ID，两边是同一个不等号。 | 一致。 |
| I-2.4 extent 偏移 | checker 在 4 个入口判（见 C4）。 | 与 kb 行 133 的 checker 一半一致；core 一半见 C4。 |

### 冲突 A：I-8.1 的安全系数 F（C2 第 1 项，同时落在 C1 与 C4）

| 一边 | 原文 |
|---|---|
| kb 条款 | 「每块盘每个自证过的系统配置槽里三个字段各占一格：journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋，journal 最坏占用 = 在飞上限 × 4096，安全系数 F = JOURNAL_SAFETY_FACTOR（格式常量，读者要求 F ≥ 2），任一不等判红」(`.claude/kb/invariants.md:90`) |
| 字段表 | 「journal 安全系数 F」(`.claude/kb/layout/01-first-txn.md:136`)，该行宽 4、值 3、出处列写 I-8.1 |
| checker | 「safety_factor >= JOURNAL_SAFETY_FACTOR_LOWER_BOUND,」(`crates/singlefs-checker/src/image.rs:838`) |
| core | 「if u64::from(recorded.safety_factor) != JOURNAL_SAFETY_FACTOR {」(`crates/singlefs-core/src/recovery.rs:847`) |

按字面，条款的第三格是「F = 格式常量」的等式，括注「读者要求 F ≥ 2」是下界；core 照等式判，checker 照括注判。条款本身两读，字面更靠 core（「任一不等判红」对着的是三个等式）。我不替主 agent 定哪边对。

命令实测（草稿副本，自写探针 `zz_probe_safety_factor_two_with_consistent_fields`：干净镜像上四个系统配置槽 F 改成 2、在飞上限改成 ⌊环长 ÷ 4096 ÷ 2⌋、最坏占用跟着改、整槽重封）：

```
PROBE F=2 checker violated = []; I-8.1 = Holds; I-7.13 = Holds
PROBE clean core mount_read_only = Ok, open+read = Ok(Ok(1))
PROBE F=2-consistent core mount_read_only = Err(Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: JournalSafetyFactorNotTheFirstVersionValue { recorded_safety_factor: 2, first_version_safety_factor: 3 } }))
```

同一张镜像：checker 零红、I-8.1 成立；core 只读挂载拒。随之 I-4.1 的重放在飞上限用格式常量 F（`crates/singlefs-checker/src/walk.rs:6380`），I-8.1 用槽里的 F，两处在 F 不等于 3 时说的不是同一个环几何。

推翻：把 checker 的 F 格改成与格式常量相等后，上面那张镜像上 I-8.1 红；或条款改写成只要 F ≥ 2。

### C1 小结

12 条一致（I-1.12、I-8.10、I-8.11、I-4.9、I-3.13、I-9.26、I-3.12、I-7.16、I-7.17、I-8.12、I-7.15、I-7.5），3 条规则没说（I-9.25、I-4.1、I-8.5），1 条冲突（I-8.1）；4 处改写 checker 一侧都与 kb 对得上。
推翻 C1 的现象：在 core 写得出的合法镜像（或层 0 枚举到的状态）上这 16 条里任一条红，或某条只改一处的坏镜像它不红。我没跑层 0，合法镜像一侧只有 harness 档：每次 `prove-red.sh` 的基线（未改源码）都是绿的；副本里还把弱化点成组施加过，成组施加的结果见 5.4 末的组合表。

## 三、C3 违例位改宽

### 3.1 两侧位序、偏移、字数：逐位对得上

| 项 | Rust 一侧 | WGSL 一侧 | 判 |
|---|---|---|---|
| 违例位第一个字 | 「pub const POOL_CHECKER_VIOLATED_FIRST_WORD: usize = 8;」(`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:36`) | 内核里的同名常量由 `verdict_layout_wgsl_constants`（行 80）生成、接在三份内核源码最前面，不在 WGSL 里另写 | 一致 |
| 字数 | 「pub const POOL_CHECKER_VIOLATED_WORDS: usize = 4;」(`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:39`) | 「var<private> violated: array<u32, POOL_CHECKER_VIOLATED_WORDS>;」(`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:108`) | 一致 |
| 位序 | 「(u128::from(*word) << (32 * word_index))」(`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:74`)：第 k 个字占位 32k 起 | `violate` 写第 `invariant / 32` 字的第 `invariant % 32` 位（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:110`） | 一致 |
| 清零、读回、写出 | — | `clear_violated`（行 115）、`load_violated`（行 121）、`store_violated`（行 127）都对 `POOL_CHECKER_VIOLATED_WORDS` 个字循环；走读内核清零（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2847`）、末尾写出（行 3035）；扫描内核读回（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1748`）、写出（行 1752） | 一致 |
| 走到第几步、判不了原因 | `POOL_CHECKER_REACHED_WORD` = 8 + 4 = 12（行 43），`POOL_CHECKER_UNDECIDABLE_WORD` = 13（行 46），`VERDICT_WORDS` = 14（行 48） | 恢复、走读、扫描三个内核都用 `local * VERDICT_WORDS`（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:390`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2848`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1744`） | 一致 |
| 容量 | 编译期断言：条数不超过容量、容量不超过 128（行 50 至 57） | `violate` 的字下标 = 条号 ÷ 32，条号上限 127 时是字 3，数组有 4 个字 | 一致 |

旧写法的残留：`grep -rnE "violated_lo|violated_hi|local \* 12u" crates` 零命中（退出 1）。生成的字号常量与内核源码的拼接在 `recovery_kernel_source` 等三个函数里，三份内核都带它。

### 3.2 第 64 条以上有没有落进没人读的字：没有；但新立的 16 条 GPU 根本不判

- 没有没人读的字：下标 64（I-8.5，字 2 的最低位）到 127 都被 `pool_checker_violated_bits` 读进一个 `u128`。
- **第 49 至 64 位没人写**：WGSL 里 `INVARIANT_I_` 常量共 49 个，最大值 48（命令：`grep -c "^const INVARIANT_I_"` 输出 49；取最大值输出 48）。内核覆盖表停在 49 条：「pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[」(`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:61`)，其后列的是 0 至 48。所以 GPU 判过的状态上，新立 16 条（下标 49 至 64）的位恒为 0。
- 影响：层 0 放量走 GPU 路径时，`gpu_verdict_items` 对 65 条名字逐位出红项（「let violated = crate::crash_judge_gpu::pool_checker_violated_bits(words);」(`crates/singlefs-checker-tier/src/crash_amplification.rs:1686`)），但那 16 位永远空，**GPU 判过的状态对这 16 条没有覆盖**。对拍用例用 `& covered` 把它们屏蔽掉（「let gpu_checker_bits = pool_checker_violated_bits(&gpu_words) & covered;」(`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1101`)），所以对拍不会红。σ 那条用例拿 CPU 的全部红项与 GPU 的红项直接比（「if (gpu_items.as_ref() != Some(cpu_items) || gpu_recovery != *cpu_recovery)」(`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1209`)）：它只在 CPU 对那 16 条里某一条判红时才暴露这个缺口。
- 条款这一侧：a 轮判决第二节第 12 行的「两条腿」一列记着「每加一条要在层 0 的 GPU 判器里移一份 WGSL」(`research/prompts/m3-inv-a-r1-main-verification.md:32`)，「判」一列只采了「超了先改位宽」。本批 checker 档补丁只改宽、没移这 16 条的 WGSL，checker 补丁报告「没做什么」里也没认领。这一格报「规则没说本批要不要移；判决材料认为每加一条要移；本批没移」，由主 agent 定是下一批做还是登记欠账。
- 过期注释：`crates/singlefs-checker/src/image.rs:71` 仍写「下标 64 起的那一条在层 0 的 GPU 判器上没有位」；位宽已改，真正没有位的是下标 49 起的 16 条。`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 行 60 的文档注释「三批接完之后是全部 49 条」同。

### 3.3 判法摘要 v4 与旧判定

- 「let mut message = b"gpu-crash-judge-v4\n".to_vec();」(`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:141`) 与字号常量一起进 `gpu_judge_digest`。但全仓只有一处调用它：`grep -rn "gpu_judge_digest" crates` 命中的只有定义、文档注释与对拍用例里一句 `assert_ne!(gpu_judge_digest(), [0u8; 32]);`。**生产路径不读 v4。**
- 判定行的键是判法闭包摘要：「JudgeVersion(self.code.digest)」(`crates/singlefs-checker-tier/src/crash_amplification.rs:677`)。闭包入口含 `crash_judge_gpu.rs`（`crates/singlefs-checker-tier/src/crash_identity.rs:841`），五份内核按原文进摘要（行 1377 的断言）。`crash_judge_gpu.rs`、三份 WGSL、`walk.rs`、`image.rs` 都改了，摘要必变，旧库行不再被取用，不依赖 v4 那个字符串。`crash_judge_dispatch.rs` 在闭包之外（`FILES_THE_JUDGES_MUST_NOT_REACH`），补丁在那里只加了对拍用的入口，没改判定。
- 推翻：提交时 `crash-amplification` 流水线读到旧摘要下的判定行被复用，或 `crash_identity` 的闭包断言在合入后的树上红。要的数见末节命令 1。

### 3.4 状态列与 kb 不同步（报事实，不属本格判决）

- 「池级 checker（`walk::check_pool_image`）判 49 条」(`.claude/kb/invariants.md:12`)；「违例位是 64 位字，今天用了 49 位」(`.claude/kb/invariants.md:16`)；命令 `grep -c '^| I-.*已实现' .claude/kb/invariants.md` 输出 49，而 `IMPLEMENTED_INVARIANTS` 是 65 条。kb 那 16 行状态列还写「未实现（实现在途…）」，要在合入时写回。

## 四、C4 core 五条

### 4.1 mkfs 拒不对齐的几何（对 I-7.5）：一致

- 条款：「每个根槽的起始偏移是判定宽度的整数倍」(`.claude/kb/invariants.md:59`)；同一行「mkfs 在任何写之前先拒不按这个宽度对齐的几何」；D2 已定项 17 的射程是「根槽、系统配置槽、墓碑一类」。
- core：`first_slot_not_aligned_to_the_decision_width`（`crates/singlefs-core/src/make_filesystem.rs:322`）逐个判两个系统配置槽与 R × S 个根槽，调用排在重叠判之后、区域归属之前、任何写之前（行 444）；宽度是 max(pbs, io_min)（`crates/singlefs-core/src/system_configuration.rs:214`）。
- checker：I-7.5 只判根槽（`crates/singlefs-checker/src/image.rs:905`）。根槽偏移两边是同一个式子：core 的「region_start(target.region).0 + target.slot * u64::from(fixed_structure_slot_spacing),」(`crates/singlefs-core/src/root_ring.rs:103`) 与 checker 的「+ slot * geometry.slot_spacing;」(`crates/singlefs-checker/src/image.rs:718`)，区域起点的两个式子（`crates/singlefs-core/src/root_ring.rs:96`、`crates/singlefs-checker/src/image.rs:716`）按格式常量展开相同。mkfs 判得比 I-7.5 宽（多判系统配置槽），是 D2 已定项 17 的射程，不冲突；不对齐的系统配置槽在 checker 一侧只会经根槽偏移间接红。
- 复跑：变异表第 1803、1804 行（只取 io_min、不判对齐）prove-red 抓到，见 C6。

### 4.2 I-8.1 core 同判：对着条款字面；与 checker 在 F 格分开（冲突 A）

core 三格：F 等于格式常量（`crates/singlefs-core/src/recovery.rs:847`）、在飞上限等于现算、最坏占用等于上限乘 4096，经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒；在飞上限与最坏占用的式子与 checker 同（F 取 3 时）。同一发布的记录数那一格 core 不单判（恢复取前缀本来按上限截）。F 格见上节「冲突 A」，命令实测原样输出在那里。

### 4.3 指针 extent 偏移非 0 拒：core 只覆盖一部分读者，checker 判 4 个入口（冲突 B）

| 一边 | 原文 |
|---|---|
| kb 条款 | 「读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置」(`.claude/kb/invariants.md:133`)；同一行写「core 读指针头部的两个读者同改」(`.claude/kb/invariants.md:133`) |
| checker | 4 个入口都判：通用指针（`crates/singlefs-checker/src/walk.rs:941`）、inode 内部条目子指针（行 1663）、实例表链（行 2001）、extent 数据指针（行 2163） |
| core 判的 | journal 新根段两条指针（`crates/singlefs-core/src/journal.rs:313`、`crates/singlefs-core/src/journal.rs:316`）、码 2 内部条目（`crates/singlefs-core/src/code_two_tree.rs:154`）、根记录四条指针（`crates/singlefs-core/src/root_record.rs:133`、`crates/singlefs-core/src/root_record.rs:137`） |
| core 不判的 | 树表条目根指针（`crates/singlefs-core/src/records.rs:345`）、inode 内部条目子指针（行 374）、extent 记录数据指针（行 386）、实例表链指针（`crates/singlefs-core/src/instance_table.rs:206`）、extent 树下段根（`crates/singlefs-core/src/extent_tree.rs:327`）、分配记录树子指针（`crates/singlefs-core/src/allocation_record_tree.rs:774`） |

core 的写法与条款那句「两个读者」字面相符，MAC / nonce 本来就是同一形态的不对称（checker 4 个入口、core 2 个读者）。同一张镜像上的结果不同，命令实测（草稿副本，自写探针 `zz_probe_extent_offset_on_the_extent_data_pointer`：干净镜像上 extent 树根里那条数据指针的 extent 偏移 0 改 1、重封到根；journal 新根段不含它，不用同步）：

```
PROBE extent-data-pointer checker violated = ["I-2.4"]
PROBE clean core mount_read_only = Ok, open+read = Ok(Ok(1))
PROBE extent-data-pointer-offset-1 core mount_read_only = Ok, open+read = Ok(Ok(1))
```

checker 红 I-2.4，core 只读挂载与读第一个字节都成功。这是打中（判据「checker 与 core 在同一张镜像上对同一件事判得不同」），交主 agent 判：core 其余读者要不要也换成判，还是条款把 checker 收窄到两个读者。core 报告问题 2 已写同一件事。
推翻：把 core 那几处 `read_from` 换成 `read_judging_the_extent_offset_from` 后，上面那张镜像 core 一侧挂载失败。

### 4.4 根记录 extent 偏移非 0 按自证不过处置、最新根坏时退回上一条

- core：四条指针任一非 0，`parse_slot` 交 None（`crates/singlefs-core/src/root_record.rs:133` 的 `.ok()?`），择根不可择。先例：checker 的 `check_root_slot` 对根槽 flags 有未知位也返回 `NonZeroFlags`（`crates/singlefs-checker/src/lib.rs:337`）。
- 合法根不会被当坏：写者恒写 0（`crates/singlefs-core/src/pointer.rs:216` 是头部里 extent 偏移那一个 `put_u16(0)`）；副本里 core 一字没改，期间跑过的 harness 测试二进制（5.4 的组合表，含 `random_histories` 19 条）没有一条因为 core 拒了根而红。
- 与 checker 的分叉：checker 的 `valid_roots` 只用 `check_root_slot`（`crates/singlefs-checker/src/image.rs:1001`），extent 偏移非 0 的根仍是自证根，最新根照常走读并红 I-2.4；core 不选它。命令实测（探针 `zz_probe_newest_root_only_with_extent_offset_one`：只把 txg 最大的那一条根的实例表指针 extent 偏移改 1、重封）：

```
PROBE newest-root-only (txg 3, device 0) checker violated = ["I-2.4"]; I-7.2 = Holds
PROBE newest-root-only core mount_read_only = Ok, chosen_root txg CheckpointTxg(2), effective_root txg CheckpointTxg(3)
PROBE clean core chosen_root txg CheckpointTxg(3), effective_root txg CheckpointTxg(3)
```

core 退到 txg 2 的根，再靠 journal 重放回 txg 3，挂载成功，不报「根坏了」；checker 红 I-2.4。是静默退回一代还是拒，条款没写（core 报告问题 3）：**规则没说**。这一条上两边的差别是「一边报损坏、一边悄悄绕过」，不是「一边挂得上一边挂不上」，所以我不记打中，只列出来。

### 4.5 C462：一致

clause：「单列一个错误成员（约 30 行）」(`.claude/kb/checks-owed.md:378`)。core：「(None, None) => Err(RecoveryFailure::NoDeviceInThePool),」(`crates/singlefs-core/src/recovery.rs:1148`)。checker 一侧没有对应物。变异表第 1810 行抓到。

### 4.6 C463：只做了只读挂载那一半

- 条款：「2026-10-01 判决：只读与恢复路径报出来（新函数加一个字段）」(`.claude/kb/checks-owed.md:379`)。
- core：新函数 `choose_system_configuration_naming_devices_without_a_mountable_slot`（`crates/singlefs-core/src/recovery.rs:1059`）交回 `SystemConfigurationChoice`（行 1044），只读挂载把它放进 `MountedReadOnly`（`crates/singlefs-core/src/mounted_read.rs:545`）。`recover` 的 `RecoveryReport`（`crates/singlefs-core/src/recovery.rs:322`）没有这个字段，恢复路径要自己调新函数才拿得到。报告说明因为 checker 档 `e158_root_choice_repair.rs` 字面构造 `RecoveryReport`。判：部分一致，「恢复路径」那一半条款字面没做完，主 agent 定。
- 与 checker 的旁路报告对：core 的「两槽都不可择」包含读不出、自证不过、incompat 位不认得三类；checker 的 `devices_without_a_self_verified_slot` 只数 magic 与整槽校验和不对的（`crates/singlefs-checker/src/image.rs:965`）。两个槽都自证过但 incompat 位不认得的盘：core 点名，checker 两张表都不列。kb 两处各自的口径写法不同（C463 写「不可择」，I-8.5 写「自证过」），**规则没说**要不要统一。

## 五、C6 变异表

### 5.1 数与锚点

- 本批追加 48 行（checker 档 8、池级 checker 27、core 10、合入后 3），整行替换 7 行（checker 档 1、锚点 6；锚点里的 1779 就是追加行 I-3.13 换了锚点，所以独立的行是 54 条）。数法：五份补丁目录里 `mutations-append.tsv` 与 `mutations-replacements.tsv` 的非空行数（命令 `grep -c .`）：checker-tier 8 与 1、checker 27、core 10、postmerge 2、anchors 1 与 6。
- 锚点：这 54 行（表第 1660 至 1667、1776 至 1812、1819 至 1821 行，加整行替换里另外 6 行 173、174、177、867、1350、1595）在冻结副本源文件里原文都恰好命中一次（草稿脚本 `anchors.py` 逐行数子串，输出 54 行、命中数列全是 1，没有一行不是 1；原文里的 `\n` 还原成换行，同 mutation-tables 格数法）。

### 5.2 harness 档 45 行复跑（`prove-red.sh --copy`，经内存包装；每组先跑基线）

分九次跑完（日志在模型目录 `logs/pr-*.log`），汇总：

```
$ cat pr-a.log pr-b.log pr-core1.log pr-core2.log pr-s1.log pr-s2.log pr-s3.log pr-s4.log pr-s5.log | grep -c "<TAB>抓到<TAB>"
43
```
没红的 2 行（`pr-1782-1783.log`，去掉日志路径后的原样）：

```
✗ 二·1 I-7.16 判定恒真：系统配置两槽世代号差值不判	没红	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant 没判红
✗ 二·1 I-7.16 判定恒真：系统配置槽世代号奇偶不判	没红	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant 没判红
✗ 点名 2 条：跑了 2 条，跳过 0 条，有没红或无效的（逐条列在上面）
```

`pr-s4.log` 里有两行是内存包装的结局（退出码 252，命令没跑，不算结果）：表第 1819、1821 行，改在 `pr-s5.log` 重跑，两行都抓到。复跑结果与实现员各自报告里写的「抓到」一致（core 10 行、anchors 19 行、postmerge 2 行都复出）。第一段 checker 补丁报告写明「没单证」的 12 行里，I-3.13 一行 anchors 报告后来证了；其余 11 行（I-8.10、I-9.25、I-9.26、I-7.15、I-7.16 两行、I-7.17、I-2.4、I-8.11、I-8.12、I-7.8）是我这次首证：9 行抓到，**I-7.16 两行没红**。

**I-7.16 两行为什么不红**：那份坏镜像把没被择的那一槽的世代号改成被择那个减 2；减完 ≥ 2 时它与被择那个同奇偶、住错槽，**奇偶格与差值格同时红**。任一格单独恒真，另一格仍红，用例（只比「红的是 I-7.16」）照绿。两行单施加在整份 `checker_known_bad_images`（55 条）上也全绿（探针 M1、M2，见 5.4）。checker 补丁报告里已料到这一点（「单关一格可能不红」），留给了提交时的整表复跑；整表复跑跑的是同一条用例，结果会相同（推的）。要两行都红，坏镜像要拆成两份：只破奇偶（把两槽内容对调，差值仍是 1）、只破差值（槽 0 世代号 2、槽 1 世代号 5：奇偶都对、相差 3）。后面这两种造法是推的，没造过。

### 5.3 checker 档 9 行（只能读，不能跑）

| 表行 | 替换文是否真改坏被测行为 | 点名的测试能不能红 |
|---|---|---|
| 1660 Rust 读违例位只读前两个字 | 是：`..FIRST_WORD + WORDS` 改成 `..FIRST_WORD + 2`，位 64 与 127 读不到 | 能：lib 单测置字 2、3 后期望位 64、127（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:270` 起）。不要卡。 |
| 1661 走读与扫描内核写出违例位只写前两个字 | 是，内核两处共用的 `store_violated` 少写字 2、3 | **按读法不会红**，见下 |
| 1662 扫描内核读回只读前两个字 | 是：字 2、3 读回成 0，走读内核记的高位被扫描内核写回时抹掉 | 能，但要卡；没卡报 NOT_RUN 后返回，行活下来 |
| 1595（整行替换）编号低于 32 的违例不记 | 是：字 0 恒不写 | 主对拍能红，要卡且要有判红的状态 |
| 1663 R-换 读出内容不同也判相同 | 是 | lib 单测，能红（没跑） |
| 1664 二次崩溃之后不再挂 | 是：计数后立即返回 | lib 单测钉「判过」个数，能红（没跑） |
| 1665 做两次那边当没文件 | 是 | 能红（没跑） |
| 1666 登记表不在时照旧去读 | 是 | 能红（没跑） |
| 1667 登记表在时取不到用例 | 是 | 能红（没跑） |

**1661 的读法**：结论缓冲在派活前整个填 0xFFFFFFFF（「contents: &words_to_bytes(&vec![」(`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:731`) 与其后一行 `VERDICT_WORD_NOT_WRITTEN;`，常量定义在 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:118`）。变异让字 2、3 从不被写，没种假位与种过假位两次判出来的字 2、3 都还是全 1。名为 `a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels` 的用例比的是「种过的 = 没种的并上种的位」：「let expected_bits = pool_checker_violated_bits(unplanted_words) | planted_bits;」(`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1547`)，全 1 并上任何位还是全 1，两边相等，「if pool_checker_violated_bits(planted_words) != expected_bits || !other_words_agree {」(`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1556`) 不进。主对拍用 `& covered` 把位 49 以上屏蔽。所以表第 1661 行在它点名的测试上不会红，与表里写的「高 64 位那一半被截掉」对不上。真能抓到它的是 σ 那条用例（字 2 的最低位是下标 64，每个状态的 GPU 红项里会多出 I-8.5，与 CPU 红项不等），但变异复跑只跑行里点名的那一条。补法（推的）：让用例再断言没种的那一份里高于覆盖范围的位全是 0。
推翻：在有卡的机器上对这一行跑 `prove-red.sh`，点名的测试红了。命令在末节命令 2。

### 5.4 盲区探针：改了判法而没有行盯着的地方

做法：在副本里对新判法的一处弱化（删一格比较、删一个调用点、`>=` 改 `>` 一类）施加，先跑相关那一条测试（「行点名」一列），存活的再放进更宽的套件。探针定义在 `drive.py`，组合施加在 `combo.py`。弱化点共 31 个（30 个单独跑过，L6 只在组合里跑）。每个探针的最后一次运行摘要（命令：对 `logs/*__*.log` 逐份取第一行 `test result`，原样）：

```
A1_generic_ptr_site__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
A2_inode_child_site__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
A3_extent_data_site__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
A4_instance_chain_site__ptr | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 54 filtered
B1_i112_chain_site__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
B2_i112_data_site__ptr | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 54 filtered
C1_i925_inode_half__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
C2_i925_noninode_half__ptr | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 54 filtered
D1_i926_lower_bound__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
D2_i926_upper_bound__ptr | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 54 filtered
E1_i811_birth_tree__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
E1_i811_birth_tree__rel | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
E2_i811_class__rel | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
E3_i811_instance__rel | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
E4_i811_tail__rel | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
E5_i811_birth_txg__rel | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 54 filtered
F1_i810_tree_table__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
F2_i810_mapping_root__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
F3_i810_watermark__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
G1_i49_second_site__rel | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
H1_i717_region__full | test result: FAILED. 54 passed; 1 failed; 0 ignored; 0 measured; 0 filtered
H1_i717_region__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
H2_i717_slot__full | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
H2_i717_slot__ptr | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
I1_i812_strict__full | test result: FAILED. 53 passed; 2 failed; 0 ignored; 0 measured; 0 filtered
I1_i812_strict__rel | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
I2_i81_count_strict__full | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
I2_i81_count_strict__ring | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
J1_i75_min_width__full | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
J1_i75_min_width__ring | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
K1_i85_unchoosable_not_counted__full | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
K1_i85_unchoosable_not_counted__ring | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out
L1_i41_limit__i41 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out
L2_i41_follow__i41 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out
L3_i41_commit__i41 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out
L4_i41_open_publish__i41 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out
L5_i41_prefix_entry__i41 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out
M1_i716_diff__full | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
M2_i716_parity__full | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

组合施加的整套结果（同样来自模型目录 `logs/`）：

| 组合 | 弱化点 | 跑的测试二进制 | 结果 |
|---|---|---|---|
| `combo1-full.log` | A1 A2 A3 B1 C1 D1 E1 E3 E4 F1 F2 F3 G1（13 个） | `checker_known_bad_images` | 55 条全绿 |
| `combo1-g1.log` | 同上 13 个 | 另 10 个二进制 | 9 个全绿；`writable_mount_of_a_formatted_pool` 红 2 条：`I-1.12 登记的是真被评估过且成立，判的是 NotApplicable(…)` 与 `I-4.9 登记的是真被评估过且成立，判的是 NotApplicable(…)`，即 B1、G1 被这份「登记的判定」表抓住 |
| `combo2-g2.log` | 去掉 B1、G1 的 11 个 | 另 9 个二进制（含 `writable_mount_of_a_formatted_pool`、`random_histories` 19 条；`rollback_by_a_forward_publish` 全是 ignore） | 全绿 |
| `combo3-l.log` | L1 至 L6（I-4.1 重放六条规则同时拿掉） | `checker_known_bad_images` 55 条、`writable_mount_of_a_formatted_pool` 15 条、另 3 个 | 全绿 |

### 5.5 解读：哪些判法没有任何测试盯着

存活 = 在行点名的那条测试上不红。分三类：

- **被更宽的套件抓住**（4 个）：B1（I-1.12 实例表链那个调用点）与 G1（I-4.9 的第二处）被 `writable_mount_of_a_formatted_pool` 的判定登记表抓住；H1（I-7.17 区域格）被 `the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:2446`）抓住；I1（I-8.12 的 `>=` 改 `>`）被 `each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria` 等两条抓住。它们都不是变异表行盯着的，只是别的用例顺带守着。
- **整套仍不红的盲区**（22 个）：
  1. I-2.4 的通用指针、inode 内部条目子指针、extent 数据指针三个调用点（A1、A2、A3）；只有实例表链那个点被坏镜像钉住（A4 红）。这三个点正是 C4 冲突 B 里 core 不判的读者所对应的 checker 入口。
  2. I-9.25 的 inode 树那一半（C1）：坏镜像只有 extent 树条目那一支（C2 红）。
  3. I-9.26 的下界（D1，对象出生代不大于改动计数）：坏镜像只造了上界。
  4. I-8.11 点名项的类标签、实例代号、尾段、出生树（E1 至 E4）：坏镜像只改了诞生代号（E5 红）。
  5. I-8.10 新根段的树表指针、映射根指针、水位三段（F1 至 F3）：坏镜像只改了 F。合入后两行变异（表第 1819、1820 行）盯的是测试助手，不是 checker 的比较。
  6. I-7.17 的槽格（H2）：坏镜像把区域与槽一起放错。
  7. I-8.1 同一次发布记录数与在飞上限的相等边界（I2：`<=` 改 `<`）。
  8. I-7.5 的判定宽度取各盘最大（J1：`.max()` 改 `.min()`）：没有异构池的镜像。
  9. I-8.5 里「自证过而不可择」那一类算不算自证过（K1）：a 轮判决专门要求分开的那一类，没有任何镜像（各槽自证过而 incompat 位不认得）盯着；改成不算，整套 55 条不红，推的：这种池上 I-8.5 会误红（没造过这种镜像）。
  10. I-4.1 重放的六条规则（L1 在飞上限截断、L2 带标志那条之后有同发布记录、L3 末条要带提交标记、L4 一次发布内序号与事务号接得上、L5 记录能进前缀的反向链与断号、L6 计数器接得上）：表里只有两行盯着「走读失败判红」与「点名单元验不过」。条款要的 R-独 与 core `effective_root` 的逐状态差分没做，这几条规则目前没有任何东西核。E2 只在 `rel` 那一条上试过。
- **被行点名的测试抓到**（5 个）：A4、B2、C2、D2、E5。

推翻 C6：对上面任一盲区补一张只破该点的坏镜像后，对应弱化点在副本里被抓到（盲区消失）；或提交时的整表复跑里表第 1782、1783 行报「没红」（两条合起来的判断）。

## 六、要主 agent 提交时跑的命令（checker 档，我一条没跑）

工作目录仓根；都要带 `SINGLEFS_HEAVY_TESTS=commit` 前缀，经内存包装，线程数照当时的上限。

1. 违例位与判法摘要（C3）：`cargo test -p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests:: crash_identity::tests::`。看 `violated_bits_past_the_sixty_fourth_are_read_from_the_third_and_fourth_words` 过、`crash_identity` 的闭包断言过（行 1377 那条）。
2. 表第 1661 行（C6）：`bash research/scripts/prove-red.sh --copy <副本> --memory 12G singlefs-checker-tier "违例位改宽：checker 内核写出违例位只写前两个字（高 64 位那一半被截掉）"`，要在有卡的机器上。按我的读法它「没红」；红了就推翻 5.3 那一条。
3. σ 对拍（C3 影响）：`cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test record_checker_judges_absence_by_the_persisted_set -- the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times --nocapture`（要卡）。看 `GPU_JUDGE_SIGMA … mismatches=` 是 0；它在 CPU 对新立 16 条里任一条判红的状态上会报 GPU 与 CPU 不同。
4. 整表复跑（门禁 checker-tier-crates-mutation-replay）：看表第 1782、1783 行是否报「没红」（我在 harness 档已复出）。
5. 层 0 对新 16 条的覆盖：GPU 路径没有它们（C3），要主 agent 定：层 0 的 CPU 判器单独对这 16 条跑一次，还是先移 WGSL。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| C1 I-1.12、I-8.10、I-8.11、I-4.9、I-3.13、I-9.26、I-3.12、I-7.16、I-7.17、I-8.12、I-7.15、I-7.5 | 一致 | 候选集、不适用、偏移与 kb 逐句对得上 |
| C1 I-9.25 | 规则没说 | 共享树表单元只在第一个走到的版本上判；kb 写每条候选根 |
| C1 I-4.1 | 规则没说 | 三处自加的不适用；干净镜像上恒不适用 |
| C1 I-8.5 | 规则没说 | 越界值整池拒那一支 I-8.5 不判，kb 没写 |
| C1 I-8.1 | 冲突 | F 格：kb 字面是等式（core 照此），checker 判 ≥ 2；F=2 的一致镜像 checker 零红、core 拒挂（命令实测） |
| C1 I-1.2、I-3.9、I-7.8 改写 | 一致 | |
| C3 位序、偏移、字数 | 一致 | 两侧逐位对得上，没有没人读的字 |
| C3 第 49 至 64 位 | 规则没说 | 16 条新不变量 GPU 内核没有判，位恒 0；对拍屏蔽，σ 才暴露 |
| C3 判法摘要 | 一致 | v4 生产路径不读；键是闭包摘要，旧判定不沿用 |
| C4 mkfs 拒不对齐 | 一致 | mkfs 判得比 I-7.5 宽，是 D2 已定项 17 的射程 |
| C4 I-8.1 core 同判 | 冲突 | 同 C1 I-8.1：core 对着条款字面，与 checker 在 F 格分开 |
| C4 extent 偏移非 0 | 冲突 | checker 判 4 个入口，core 6 处读者不判；extent 数据指针偏移 1：checker 红 I-2.4、core 挂载与读都成功（命令实测） |
| C4 根记录 extent 偏移 | 规则没说 | 合法根不会被当坏；最新根坏时 core 退一代再靠 journal 重放回同版，条款没写拒还是退 |
| C4 C462 | 一致 | |
| C4 C463 | 规则没说 | 只读挂载有字段；恢复路径只有新函数、`RecoveryReport` 没字段 |
| C6 harness 档 45 行 | 43 抓到 / 2 没红 | 没红的是 I-7.16 两行（表第 1782、1783 行） |
| C6 checker 档 9 行 | 1 行按读法不红 | 表第 1661 行：缓冲初值全 1，种假位的用例看不出字 2、3 没写 |
| C6 盲区 | 22 个弱化点整套仍不红 | I-4.1 重放六条规则、I-8.5 自证过而不可择、I-9.26 下界、I-8.11 四字段、I-8.10 三段等 |

## 八、没做什么

- 没跑 checker 档任何东西（`singlefs-checker-tier` 测试、层 0、GPU 对拍、变异整表）：C3 的 GPU 一侧只能读，C6 的表第 1660 至 1667 行与整行替换的 1595 行只能读，没有它们的复跑数。
- 没替主 agent 判哪边对：冲突 A、冲突 B 都只并排抄了两边并给了推翻条件。
- 没判 C2、C5 的格；冲突 A 与冲突 B 落在 C2 第 1 项与 C4，我只在需要处用了命令实测，探针用例是我自写的新形态，按判据算零轮，没有对它们做过第二次抽样。
- 批次 5 的注释补丁（agent 名）不在我的格里；顺手核了一件事：补丁里除注释与 `#[ignore]` 理由外没有别的改动行（`grep -n '^[-+]'` 滤掉注释行后无输出）。
- 没碰 kb；kb 状态列与条数没同步（3.4）只报事实。
- 没做 R-独 与 core `effective_root` 的逐状态差分（报告里本来就欠，要层 0）。
- 草稿现场：仓副本 `/tmp/claude-1000/m3-batch4-code-r1-sonnet/tree/`（含 `target/`，14G）在交回前删掉，原因是已无用、重建 35 秒；留着的只有草稿目录里的脚本与日志，副本一份进模型目录。
