# test-script-align-code-r1 攻方腿（Opus）报告：K2、K3（2026-10-02）

被判的是冻结副本 `/tmp/claude-1000/test-script-align-code-r1/tree/` 里四份补丁合入之后的代码。下文引代码都写冻结副本的绝对路径与行号；副本上量的数都是草稿副本（`/tmp/claude-1000/test-script-align-code-r1-opus/tree`、`tree2`）上原型跑出来的，不是入库装置的数。checker 档包的测试一条没跑。

## 复跑

```
bash research/prompts/test-script-align-code-r1-opus-model/rerun.sh <空的工作目录> [/tmp/claude-1000/test-script-align-code-r1/tree]
```

它从冻结副本拷两份仓（`tree` 原样，`tree2` 在 `draw_second_crash_points` 里改坏一处），照模型目录的源码建两个原型包（`proto` 依赖 `tree` 的四个包，开 `verdict-store,gpu`；`proto2` 依赖 `tree2`），编完跑七个二进制与两个换改动的脚本，原样输出落进 `<工作目录>/runs/`，末尾 grep 出全部 `K2…`、`K3KEYS`、`VARIANT` 行。都经 `research/scripts/run-with-memory-cap.sh`、`capped.sh 8`。本腿实跑的原样输出在模型目录 `outputs/` 下。

模型目录各文件 sha256（`SHA256SUMS` 同此）：

| 文件 | sha256 |
|---|---|
| `./appended.tsv` | `225d5198ff07bfa9db96cc930c901e3d5f562f8f21c020c3c5da3a3d4da88295` |
| `./apply_row.py` | `1dc3ea21f90353af1b13bf43287a55265291727db8706704b3c7331cd852f6ce` |
| `./fixA.py` | `980608c166dc8ed06024e38946dd18aa4d7126a3aca840f82c8d9489e1fcd167` |
| `./k3check.py` | `2bb90e3b534e9c6b940105747db17282dc4208162016ff3bcfc4cff4382858eb` |
| `./keys_variant.sh` | `aa77f720209f46bf6ffd0f1ea067671875ed45c2a6bb182c7a2ba843ad5f5641` |
| `./outputs/ci_repro.out` | `cbabd272e5453aa9338e348a50b697a5c4beaedcca39d57ea3bb238afe059364` |
| `./outputs/gpu_probe-lvp.out` | `cf69214b90e0a76e9fffa59ff56265c9a67f85e6bb82552a08ab5dacb1e637d1` |
| `./outputs/gpu_probe-missing.out` | `f55107a9a69e4c535a242aba00d969df199cf726e2fcdc941f18fbfb9c4a9686` |
| `./outputs/gpu_probe-normal.out` | `fc85819ef7b61982b03f53c128992e92d87a772bd2300d050dbdc6f856b3500f` |
| `./outputs/k3check.out` | `5534ed5795451000d6aef8bff41ac60f52d913db632cbc3eed4048446a58a649` |
| `./outputs/keys_probe-as-merged.out` | `afa4db947dc72e6641827153595c331689c4dc091247d0db041343c36bbe8a63` |
| `./outputs/keys_probe-mutation-key-without-write-table.out` | `afa4db947dc72e6641827153595c331689c4dc091247d0db041343c36bbe8a63` |
| `./outputs/repro_probe-6.out` | `f1be1f10326016149d8eb7d28be19d3c8a816cd1a43837178a48f3fac6e8bf5b` |
| `./outputs/reuse_key-fix-a-versions-in-the-plan-hash.out` | `f5310b37037fdd62e8722361d0aff0fba4b826e04f92b7283cdb6f377d44413b` |
| `./outputs/reuse_key-mutation-block-key-without-judge-version.out` | `2a73ab05b54f3d8a9988678d188b550b46ba69e86ddfba3bbe88f3192a42a74f` |
| `./outputs/reuse_key-mutation-key-without-write-table.out` | `498b128403779c47a87f9122784e6420e26bcf43a65874c685af6bbb7bc45535` |
| `./outputs/reuse_key.out` | `498b128403779c47a87f9122784e6420e26bcf43a65874c685af6bbb7bc45535` |
| `./outputs/reuse_key-recheck.out` | `498b128403779c47a87f9122784e6420e26bcf43a65874c685af6bbb7bc45535` |
| `./outputs/stopped_line.out` | `082b3633f5bdb010895f948840d2a02572c8e21dcad5ff58e45cca3d0922dcc9` |
| `./outputs/variants.out` | `90ad778280726eeccff8752cafadab9866f4d41fb347403a4ae9887e96dbe12e` |
| `./proto2/Cargo.toml` | `c4d407e9e694aa11dc21ddc3020931f861015f774e87d5675536b2e1cbaf4888` |
| `./proto2/src/bin/ci_repro.rs` | `0d5ad9a172955677f9400c82f018897bf69927ff99958b0dcb89a0b91b47e1d3` |
| `./proto/Cargo.toml` | `f76c78a9bb55a4d9f5de3fb92af56fa997277d99f774cd9a9cec13169abddd11` |
| `./proto/src/bin/gpu_probe.rs` | `2554fdea8b1fdfe08b0bda16c6a3f41f29f6adb4c1fb54b3b5b22fe3fdbb72e3` |
| `./proto/src/bin/keys_probe.rs` | `a50ba9bb58751023ced6243ab3657ec534d6bf860712c473dbad367b994b2a6f` |
| `./proto/src/bin/repro_probe.rs` | `2b99568dd24b5a1a6a9c7fd3716d502bc53525611798ff2e097e68cdaa5fcf5b` |
| `./proto/src/bin/reuse_key.rs` | `58c1a88de7fa79dd42a790b59d50171ab3220ad9f6d810afbb0989e6d970d56e` |
| `./proto/src/bin/stopped_line.rs` | `0597483274038783ae996b569b2d09e353080c6840c50076eb3fa7a5c39ef4be` |
| `./proto/src/common_crash_points/mod.rs` | `2a2174c9dd55b2114e4a083e26508f8b33dee93ec600bb20e38c1cefcb4da022` |
| `./proto/src/common/mod.rs` | `054ad25d3f02cc0862cd79bbf6683cc7f212f38295854f179a17e876399e1f92` |
| `./proto/src/flows.rs` | `0d7f052f040a215fbda9b4b20c86d3ce79c1043e70c6884e1691474d1c230e58` |
| `./proto/src/lib.rs` | `3f8dc05e600fcefee472a7a04825a7a9e860210a78428f86418b42caa35ac1d6` |
| `./rerun.sh` | `634c42956b84cc490a75319ceadd070058d45e35965c0c3f97d89570cfc86c78` |
| `./variants.sh` | `2da205578e4cb529656bdc8d8a66507b90f0f66246644d4c1e6c252a388f43d3` |

## 各格判定一览

| 格 | 子项 | 判定 | 一句话 |
|---|---|---|---|
| K2 | E165 交叉核五对列、8 段与别的段数分支 | 没打中 | 五对列与红例账表头的列定义逐一对上；今天钉表 195 行逐行核五对列，0 行不同；8 段走专门的报错，别的段数走通用报错 |
| K2 | 驱动行字段：零态 | 没打中（够不着） | 计划里最后一个点总带全持久那一个状态，造不出零态的流 |
| K2 | 驱动行字段：全复用 | **打中（L1）** | 没叫停时照实（`states_reused=57`）；同一份库第三趟全复用、跑完时叫停文件在：`stopped_early=true states_checked_here=0 states_reused=0`，录入时已判 57 态 |
| K2 | 驱动行字段：ignore 全跳过 | 没打中 | `exhaustive` 按设计不算 ignore，另报 `ignored_points`、`states_skipped_by_ignore`；读代码，没造流跑 |
| K2 | 展开上限的 panic 被吞 | 没打中 | 读这个变量的三处都经 `from_environment_or`；线索：GPU 对拍自己的规模变量 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 照旧写错就悄悄退回默认（不在 C606 的射程里） |
| K2 | 复用对照真关了复用没有 | **打中（R1）** | 版本表不进复用键，也不进判法版本：写表相同、只差版本表的两条流在同一库同一判法版本下互相复用，两个方向都给错结局（12 态该绿判红、12 态该红判绿）；合入的对照照它自己的输入是绿的，看不见这一处 |
| K2 | GPU 取卡：有卡但驱动坏 | **打中（G1）** | 本机 PCI 上 2 个 NVIDIA 显示控制器；让 Vulkan 找不到 NVIDIA 的驱动（ICD 指到不存在的文件、或只给 llvmpipe），判定是 `NotRunNoGpuAdapter`，用例报「本次未跑」、退 0；C620 要的是判红 |
| K2 | GPU 取卡：非 NVIDIA 独显、集显 | 没打中 | 判「本次未跑」，与「只用 NVIDIA 独显」的设计一致 |
| K2 | 复现行喂回入口：随机历史 | 没打中 | 六段取样各 6 个种子，取样那一路与照抄复现行那一路逐项相同：36 段 0 段不同 |
| K2 | 复现行喂回入口：崩溃注入 | **打中（S2）** | 历史与参数逐项相同，但大档入口只判 `new_findings`：小快档红在结构断言上（副本里改坏一处、二次崩溃一个都不摆），照抄它打的重放行跑大档入口判绿 |
| K3 | 34 行的锚点、点名测试、特性 | 没打中 | 34 行都在表里、原文都恰好命中一次、点名的测试各有恰好一处定义、参数里的特性罩得住目标 |
| K3 | 替换文改的是不是那一判 | **打中（M3）** | 第 1587 行「复用键不罩写表」：在我照合入对照造的副本上改坏，对照仍逐态相同（0 态不同）——A、B 尾巴上有状态的三个点父镜像本来就不同，写表进不进键都分得开；点名的测试推的不会红 |
| K3 | 同上，其余 33 行 | 没打中 | 逐行读替换文推得出点名的那条断言红；第 1588 行在副本对照上量过会红（12 态不同）；第 1575 行红在防御用的 `assert_eq!` panic 上，不是红在 `is_err` 那一判 |
| K3 | E165 删的那一行 | 没打中 | 删的那行点名的测试已改名；同锚点、同替换文的新行（第 1575 行）换上了点名的新测试 |

## K2 第一节：复用对照与复用键（R1）

### 打中的历史

1. 冻结副本里块键由四样拼成：「input_fingerprint: InputFingerprint(*self.reuse_key.as_bytes()),」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:156`）、节点名、「enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:159`）、块起点。复用键只罩两样：「pub write_table_with_barriers: [u8; DIGEST_BYTES],」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:93`）与「pub parent_output: [u8; DIGEST_BYTES],」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:94`）；计划哈希只写段的状态数与展开方式。版本表（`CrashFlow::versions`）与被判的根槽写下标（`judged_root_index`）是判器的输入，四样里都没有。
2. 判法版本也不罩它：「到进度模块与 crate 根门前停）。判法改了每个点的复用键都变、树不动；设点的测试文件、流水线自己变了，摘要不变。」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:339`）。版本表是测试文件里写的（两流用例的 `record_flow` 按传进来的内容造），改测试文件里的版本表，判法版本不变。
3. 库默认跨轮留着：「库默认 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library（开机不清空，重跑同一条命令就是续跑），」（`research/scripts/crash-amplification-item.sh:28`）。

可达的历史：拿一条流跑一趟进库；改测试文件里这条流的版本表（修一个写错的期望内容，或写错一个），写表、判器代码都不动；同一条命令再跑一趟。第二趟每个块都按键命中，读回的是第一趟的判定。

### 量到的（草稿副本上的原型 `reuse_key`，流照两流用例的 `record_flow` 造，盘换成内存稀疏盘；这一次 57 态，层 0 全展开的闭式数 33554462 态，本原型只按展开上限 6 跑）

同一段写表、两张版本表（一张对，一张把 txg 4 那一版改坏一个字节，与合入对照里的 `with_the_overwritten_version_miswritten` 相同）：15 个点的复用键、身份、计划哈希逐个相同（`research/prompts/test-script-align-code-r1-opus-model/outputs/reuse_key.out:2`：「K2R1 name=keys points=15 every_point_same_reuse_key_identity_plan_hash=true versions_differ=true」）。不经库逐态现判，健康的 0 态红、写错的 12 态红。

`research/prompts/test-script-align-code-r1-opus-model/outputs/reuse_key.out` 原样：

```
K2R1 name=order_miswritten_then_healthy first_red_states=12 healthy_blocks_reused=13 healthy_states_checked=0 healthy_stored_red_states=12 healthy_fresh_red_states=0 states_differing_from_fresh=12 first_differing=Some(45)
K2R1 name=order_healthy_then_miswritten first_red_states=0 miswritten_blocks_reused=13 miswritten_states_checked=0 miswritten_stored_red_states=0 miswritten_fresh_red_states=12 states_differing_from_fresh=12 first_differing=Some(45)
K2R1 name=control_as_merged a_red_states=12 older_b_red_states=12 b_blocks_reused=10 states_differing_from_fresh=0
K2R1 name=control_with_the_miswritten_b_under_the_same_version miswritten_b_red_states=12 b_blocks_reused=13 states_differing_from_fresh=12 first_differing=Some(45)
```

- 写错的先进库、健康的后进：健康的 13 块全复用、一态不判，库里读回 12 态红，现判 0 态红（该绿判红）。
- 健康的先进库、写错的后进：写错的 13 块全复用，库里读回 0 态红，现判 12 态红（该红不红）。
- 合入对照的输入原样（A 尾巴红 → 旧判法版本下的 B 红 → B）：0 态不同，对照绿。
- 对照只改一处：库里先放的「版本表写错的 B」从旧判法版本换成同一判法版本：12 态不同，对照会红。合入的对照把「只差版本表」的那一对放在不同的判法版本下，正好避开了这一处。

`reuse_key-recheck.out` 是换完改动之后在副本换回原样、整份重编再跑的一次，与上面逐行相同。

### 四句

- 分不分辨：分得开「复用开着」与「复用关着」。关着（空库、或逐态现判）在这两条流上都对；开着的两个方向都错。它不罚合入的对照用例本身写错了什么断言，罚的是对照的输入挑法：它看不见键漏掉的这一项。
- 系统看不看得到：看得到。`plan_crash_points` 拿到的 `CrashFlow` 里就有 `versions` 与 `judged_root_index`，算键的那一刻都在手边。
- 满足哪一句：跑前判据第二条「造出一个输入让改后的判法给错结局（该红不红、该绿不绿…）⇒ 打中 K2」，两个方向都有。对应的上游条款：「每一份判定按它依赖的全部输入算键存下：被测代码、判法代码、参数、工具链。键相同的直接复用，不重算。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:58`）；对照用例对应「剪枝要自证：在小规模输入上关掉剪枝与复用跑一遍，判定与开着时逐项相同。剪枝自证进快档。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:63`）。欠账表里已经记着这一假定是推的：「推的没跑过：合入后提交时红，先定版本表进不进键」（`.claude/kb/checks-owed.md:474`）；这里是它的第一次实测，结果是对照不会红、复用会错。
- 改法各修哪一格：跑前条款只写了「写回代码」，没给改法。我提的两条见下表，都只在我的模型上量过、被攻过零轮。

| 改法 | 修「复用给错结局」那两格 | 修「对照看不见」那一格 | 代价 | 量过还是推的 |
|---|---|---|---|---|
| A：版本表与 `judged_root_index` 整份写进每个点的计划哈希 | 修：两个方向都 0 态不同（`outputs/reuse_key-fix-a-versions-in-the-plan-hash.out` 第 4、5 行） | 不相干 | 前缀共享全丢：合入对照里 B 复用的块从 10 掉到 0（同文件第 6 行「b_blocks_reused=0」），合入用例里「B 的前缀全部复用」那条断言会红 | 量过（草稿副本） |
| A2：只把这个点的状态够得着的那几版（txg 不超过这个点发布到的最大 txg）写进键 | 推的：修 | 不相干 | 前缀共享推的保住 | 推的 |
| B：对照里加一对「同一判法版本、只差版本表」的输入 | 不修 | 修：今天就会红（上面第四行，12 态不同） | 无 | 量过（草稿副本） |

推翻条件：在入库装置上照「同一判法版本、写表相同、只差版本表」的两条流重做一次，后进的那条读回的判定与现判逐态相同，这一条就不成立。

## K2 第二节：驱动行字段（L1）

### 打中的历史

被叫停的那一支不管这一趟复用了多少，一律写 0：判完第 ① 段之后「if stop_is_requested_by_the_environment() {」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:3030`），交回的结果里「states_reused: 0,」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:3063`），理由写在上一行：「// 被叫停的那一趟报 0：同一份库上接着跑的那一趟把它复用的与它判过的一起数进复用（reuse_and_scale_fields_of_the_line）」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:3062`）。复用数在这之前已经数出来了（`states_reused_unless_stopped`），不是取不出。

叫停文件是「判完再看一眼」：这一趟全部复用、一块没判，只要那一刻文件在，也走这一支。可达的历史：同一份库上跑完一趟；第二趟开跑时叫停文件已经放着（上一回叫停之后没删，或驱动在另一项上放的）；第二趟全复用。

### 量到的（草稿副本上的原型 `stopped_line`，入口照 `run_from_environment`，单机 CPU、空的多机配置、库在草稿目录）

`research/prompts/test-script-align-code-r1-opus-model/outputs/stopped_line.out` 原样：

```
K2L1 name=first_run states_of_the_flow=57 stopped_early=false states_checked_here=57 states_already_judged_at_recording=0 fields=states_reused=0 exhaustive=false expand_up_to=6 ignored_points=0 states_skipped_by_ignore=0
K2L1 name=second_run_not_stopped stopped_early=false states_checked_here=0 states_already_judged_at_recording=57 fields=states_reused=57 exhaustive=false expand_up_to=6 ignored_points=0 states_skipped_by_ignore=0
K2L1 name=third_run_stop_file_present stopped_early=true states_checked_here=0 states_already_judged_at_recording=57 unjudged_blocks=0 fields=states_reused=0 exhaustive=false expand_up_to=6 ignored_points=0 states_skipped_by_ignore=0
```

第三趟：整条流 57 态都在库里（`unjudged_blocks=0`），行上写「判了 0、复用了 0」。按这一行汇总，这一项是「一态都没过手」。

### 四句

- 分不分辨：分得开。不叫停的第二趟照实报 57；只有叫停那一支报 0。
- 系统看不看得到：看得到，`states_reused_unless_stopped` 在叫停判断之前就算好了。
- 满足哪一句：跑前判据第二条的字面是「给错结局」，这里错的是行上一个字段的数，不是判定；我按本轮正文第三节 K2 那一句「驱动行字段在零态、全复用、ignore 全跳过时是不是照实」报它。上游的原文：「两种都不写 0」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:31`）。主 agent 若按判据字面只认判定错，这一条归「字段不照实」，不归「判定错」。
- 改法：叫停那一支照样报 `states_reused_unless_stopped`（推的：改一行；会不会让同一份库上接着跑的那一趟重复计数，要看入口怎么汇总，我没读入口脚本的汇总）；或者叫停那一支写「取不出」。都没实现，推的，被攻过零轮。

零态：够不着。`plan_crash_points` 末尾把最后一个点的序号区间补到整条流的状态数，全持久那一个状态总在里面，造不出 0 态的计划（读代码，没跑）。ignore 全跳过：`exhaustive` 按文档不算 ignore，ignore 的点数与跳过的态数另报；读代码，没造流跑。

## K2 第三节：GPU 取卡（G1）

### 打中的历史

判「没卡」只看 Vulkan 枚举：「if nvidia_discrete_adapter_count == 0 {」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:64`）之下交 `NotRunNoGpuAdapter`，取卡函数在「GpuTestStart::NotRunNoGpuAdapter { reason } => {」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs:129`）那一支打本次未跑的行、交回 `None`，用例当场返回成功。判定函数的输入只有枚举结果与起好的卡数，没有 PCI。欠账表要的是：「PCI 上有卡而 Vulkan 枚举不出判红」（`.claude/kb/checks-owed.md:489`）。变异整表那一道又把这种返回记成本次未跑：「点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红，整道不因它判红」（`.claude/gate.d/checker-tier-crates-mutation-replay.sh:58`）。

可达的历史：内核驱动在、卡在 PCI 上，用户态的 NVIDIA Vulkan 驱动坏了或不在（ICD 文件丢了、版本对不上、只剩 Mesa 的 llvmpipe）。

### 量到的（草稿副本上的原型 `gpu_probe`，判定函数经 `#[path]` 原样引自冻结副本的拷贝；这台机器上真枚举）

三份输出各自的第 9 行原样：

```
K2G1 name=live vk_icd_filenames=None vk_driver_files=None pci_nvidia_display_controllers=2 vulkan_adapters=[EnumeratedAdapter { vendor_identifier: 4318, device_kind: DiscreteGpu }, EnumeratedAdapter { vendor_identifier: 4318, device_kind: DiscreteGpu }, EnumeratedAdapter { vendor_identifier: 65541, device_kind: SoftwareRenderingOnTheCpu }] usable_cards=1 decision=RunOnTheFirstUsableCard
K2G1 name=live vk_icd_filenames=Some("/nonexistent/nvidia_icd.json") vk_driver_files=Some("/nonexistent/nvidia_icd.json") pci_nvidia_display_controllers=2 vulkan_adapters=[] usable_cards=0 decision=NotRunNoGpuAdapter { reason: "Vulkan 枚举出 0 个适配器，没有一张 NVIDIA 独显：[]" }
K2G1 name=live vk_icd_filenames=Some("/usr/share/vulkan/icd.d/lvp_icd.json") vk_driver_files=Some("/usr/share/vulkan/icd.d/lvp_icd.json") pci_nvidia_display_controllers=2 vulkan_adapters=[EnumeratedAdapter { vendor_identifier: 65541, device_kind: SoftwareRenderingOnTheCpu }] usable_cards=0 decision=NotRunNoGpuAdapter { reason: "Vulkan 枚举出 1 个适配器，没有一张 NVIDIA 独显：[EnumeratedAdapter { vendor_identifier: 65541, device_kind: SoftwareRenderingOnTheCpu }]" }
```

（分别是 `outputs/gpu_probe-normal.out`、`gpu_probe-missing.out`、`gpu_probe-lvp.out`。）两种「驱动坏」都是 PCI 上 2 个显示控制器、判定为本次未跑。合成输入（同一批输出第 1–8 行）：零个适配器、只有 llvmpipe、NVIDIA 报成虚拟或其他类型、NVIDIA 集显、AMD 独显、Intel 集显都判本次未跑；只有「NVIDIA 独显、起来 0 张」判红。

### 四句

- 分不分辨：分得开「真没卡」与「有卡驱动坏」，判定把两者判成同一个结局。
- 系统看不看得到：今天的判定函数看不到（输入里没有 PCI），所以这是「判别子没接进来」，不是「观测不到」：`/sys/bus/pci/devices/*/vendor` 与 `class` 在用例进程里读得到，原型读到了 2。
- 满足哪一句：跑前判据第二条「该报本次未跑而判绿」的反面——该红而报了本次未跑；本轮正文表里这一组补丁压着的 C620 就是这一条。
- 改法：判定函数加一个入参「PCI 上 NVIDIA 显示控制器个数」，大于 0 而枚举里没有 NVIDIA 独显判红；补一条单测喂「PCI 2、枚举 0」。推的，没实现，被攻过零轮。代价：没有 sysfs 的环境（容器）里要另定，推的。

## K2 第四节：复现行喂回入口（S1 没打中、S2 打中）

### 随机历史（S1，没打中）

原型 `repro_probe`：六段取样（快档、复用、回退、分配记录墙、单元区墙关准入、单元区墙判准入，参数照 `random_histories.rs` 的六个常量）各取种子基起 6 个种子，一边按取样那一路跑（`run_history_campaign` 一段一个种子），一边把判红时会打的复现行原样解析、经 `random_history_to_reproduce_from_environment` 读回、`generate`、`execute_history_with` 跑，比种子、取样参数、计数、结局类别。`outputs/repro_probe-6.out` 末行原样：

```
K2S1 name=total histories=36 differing=0
```

取样范围：36 段，挂钟 534.9 秒（单线程）。36 段结局都是「完成」，没有一段落在已知红或新发现上，所以「复现出同一个红」这一半没有真红的样本；比的是历史、跑法与计数逐项相同。先跑每段 1 个种子量了 67 秒，按它定的 6 个。

### 崩溃注入（S2，打中）

判红时打的重放行都指到大档那条入口：「--exact crash_injection_large_tier_from_the_environment」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2071`），快档、小快档、回退取样那几条用例的每条断言都把它带进消息：「format!("重放：{}", report.replay_line())」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:74`）。而大档入口（「fn crash_injection_large_tier_from_the_environment() {」，`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:996`）跑完只判一条「report.new_findings.is_empty(),」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1006`）。小快档另有十几条结构断言，例如「"挂载途中一个二次崩溃都没摆出来；{replay}"」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:205`）；快档还有「每条路径都跑到」那一组。红在这些上的，照抄重放行跑出来的是同一批历史、同一份计数，判绿。

量到的（草稿副本 `tree2` 里改坏一处：`draw_second_crash_points` 一个二次崩溃都不摆；原型 `ci_repro` 照小快档的参数跑一批，按小快档的全部断言判，再把它打的「重放这一整批」那一行原样喂回大档入口的解析、跑、按大档的断言判）。`outputs/ci_repro.out` 第 7、15 行原样：

```
K2S2 name=small_fast_tier crash_points=16 second_crash_points=0 new_findings=0 small_tier_reds=["no_second_crash_point"]
K2S2 name=large_tier_entry_from_the_replay_line same_parameters=true crash_points=16 second_crash_points=0 new_findings=0 large_tier_red=false tally_equal=true
```

四句：

- 分不分辨：分得开。参数与计数逐项相同（`same_parameters=true`、`tally_equal=true`），只有判法不同。
- 系统看不看得到：看得到，结构断言用的计数都在大档那一趟的报告里。
- 满足哪一句：跑前判据第二条「该红不红」，发生在复现那一路上；上游原文「随机输入带种子。红了打印种子，凭种子能单独复现那一条。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:13`）。射程：只罩红在结构或覆盖断言上的那一类；红在新发现上的，大档入口照样红。按用例名重跑那条用例照样复现（种子写死），错的是打出来的那一行指错了入口。
- 改法：重放行指回出红的那条用例本身（用例名加同一组变量，用例照变量读参数），或大档入口另带「照哪一档的断言判」的变量。推的，没实现，被攻过零轮。

## K2 第五节：没打中的两格

E165 交叉核：五对列（账第 3、4、5、12、13 段对 ②③④⑦⑧）与红例账表头的列定义（「变体、表行、域、N、状态在枚举次序里的序号、…、原实现的逐态行、变体的逐态行」）逐一对上。今天的钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 用 awk 逐行核五对：195 行、0 行不同。8 段走「没整行抄红例账」那一支，0、1–7、9–20、22 段以上走通用报错；只含空白与制表符的行两边（装置与 `research/scripts/e165-pin-red-examples.py`）都当空行跳过，口径一致。CRLF 的表装置照收（`lines()` 去掉了 `\r`），Python 那边 ㉑ 带 `\r` 会判红，判得更严，不算该红不红。这几样是读代码与 awk 核的，没编装置跑。

展开上限：读 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 的地方只剩 `ExpandUpToWritesPerSegment::from_environment_or` 一处（两份固定脚本用例与公共模块都经它），没有别的路径兜住 panic。线索（不在 C606 的射程，不算打中）：GPU 对拍自己的规模变量照旧悄悄退回默认：「std::env::var("SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:54`）下面两行就是 `.and_then(|text| text.parse().ok())`。

## K3：变异表

### 静态核（34 行全过）

四份补丁目录里的 `mutations-append.tsv` 合起来 34 行（E165 8、崩溃放量 5、GPU 9、种子复现 12），在冻结副本 `crates/mutations.tsv` 第 1573–1610 行。`k3check.py` 逐行核：整行在表里、六段、原文（`\n` 换成换行）在冻结副本那份文件里恰好命中一次、必须红的测试名在冻结副本 `crates/` 里恰好一处 `fn` 定义。`outputs/k3check.out` 里 `in_table=True anchor_hits=1 cols=6 test_defs=1` 的行数用 grep 数出来是 34。参数里的特性逐行看过：挂 `verdict-store` 的两份测试文件与 `crash_amplification` 模块那几行带了 `--features verdict-store`，GPU 那七行带 `verdict-store,gpu`，`gpu_adapter_probe…` 与 `crash_injection_campaign` 两个目标不挂特性、参数也不带；E165 那个装置在 `Cargo.toml` 里没有 `[[bin]]` 段、没有 required-features。点名的测试都没有 `#[ignore]`。

### 打中：第 1587 行「崩溃放量 复用键不罩写表」改坏之后点名的对照不红（M3）

替换文把 `step_write_table_with_barriers(…)` 换成全零（锚点在 `/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:276`）。合入对照的设计说法：「/// 库里先放的两样都故意判红：A 的尾巴（版本表写错）与旧判法版本下的 B（版本表写错）。复用的键少罩一项输入——写表不进复用键，」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:469`）。

量到的（草稿副本上原样换上这一行，原型 `keys_probe` 逐点比 A、B 的键，`reuse_key` 跑对照的原样输入）：A、B 尾巴四个点里，复用键相同的只有第 11 点（`publish_overwrite.data_unit+index_node+packed_records.1`，0 个状态、0 块）；有状态的第 12–14 点改坏前后都不同，因为它们的父镜像里已经叠了第 11 点写下的、A 与 B 不同的数据单元。`outputs/keys_probe-mutation-key-without-write-table.out` 第 13 行原样：

```
K3KEYS point=12 name=publish_overwrite.journal_record.1 same_identity=false same_reuse_key=false same_plan_hash=true states_a=3 states_b=3 key_a=b9226432f619 key_b=a0ed8cb1e5ea
```

对照照样 0 态不同：`outputs/reuse_key-mutation-key-without-write-table.out` 第 6 行「K2R1 name=control_as_merged a_red_states=12 older_b_red_states=12 b_blocks_reused=10 states_differing_from_fresh=0」。合入用例的其余断言（前缀块数 10、A 红、旧版本 B 红、B 全绿）在这份输入上也都不受这一行影响，所以推的：门禁复跑这一行会记「没红」。同一副本上第 1588 行（判定块的键不接判法版本）量过会红：`outputs/reuse_key-mutation-block-key-without-judge-version.out` 第 6 行「K2R1 name=control_as_merged a_red_states=12 older_b_red_states=12 b_blocks_reused=13 states_differing_from_fresh=12」。

四句：分得开「写表进键」与「写表不进键」的只有「父镜像相同、写表不同、有状态」的一对点，对照里没有这样的一对；系统看得到（把 A 的尾巴设成与 B 同父镜像、写下不同字节的点，例如单元写之后才分叉）；满足跑前判据第三条「替换文改的不是那一判、或点名的测试跑不到」里的前一半（点名的测试跑得到，但这一判它分不出）；改法：对照里加一对只差写表、父镜像相同、有状态的点（推的，零轮），或这一行改点名 `crash_identity.rs` 的 `each_attribute_changes_the_reuse_key`（欠账 C605 原本要的就是这一条，推的）。

### 其余 33 行：替换文与点名那一判

逐行读替换文、对着点名的测试推（没有跑，checker 档的测试不归我跑；harness 档那 8 行主 agent 已在副本上证红）：

- 第 1585、1586、1589 行：复用数乘 0、展开上限退回默认、挂钟行永不到期，各自点名的测试直接断言这三样，推的会红。
- 第 1573、1574、1576–1580 行（E165）：去掉交叉核、8 段不另报、注释当数据、三对列号错位、⑧ 对错列，点名的测试各自断言「不收这一行」或「问题以某句起头」，推的会红。
- 第 1575 行（E165「一行不查是不是二十一段」）：替换成 `< 4` 之后，20 段的行越过段数检查，进 `check_that_the_copied_ledger_row_agrees_with_the_pinned_columns`，那里「"调用方 parse_a_pinned_red_example 先核过段数"」（`/tmp/claude-1000/test-script-align-code-r1/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1938`）那条 `assert_eq!` panic，点名的测试因 panic 红，不是因 `is_err()` 那条断言红。会红，但红的位置不是那一判；那条防御断言哪天删了，红就落到越界下标的 panic 上，照样红。不算打中，记下。
- 第 1590 行（GPU 取卡零适配器当跑过）：点名的纯函数用例喂零适配器，推的会红；它与 G1 无关（G1 是判定缺一个输入，这一行改的是已有的那一支）。
- 第 1591–1598 行（GPU 内核与两份宿主代码）：替换文改的都是判法本身（只比低 32 位、全当持久、读失败报无、违例不记、记账不判、根指针不比校验和、多项式错一位），点名的都是 GPU 对 CPU 的对拍，推的会红。射程：在 G1 那种机器上（PCI 有卡、Vulkan 驱动坏）这八行与第 1590 行之外的 GPU 用例一样报本次未跑，门禁记本次未跑、不判红。
- 第 1599–1610 行（种子复现）：替换文分别拿掉复现行的一项、写错种子、两档同名、认不得的值退回默认、重放行写反两个数，点名的「复现行喂回入口」自证或库内单测各自断言读回逐项相同，推的会红；其中 8 行 harness 档主 agent 已证红（正文第二节）。

### E165 删的那一行

被删的「E165 钉住的表一行不查是不是八段」点名的测试已经改名（旧名带「八段」），留着会让点名的测试找不到；新加的第 1575 行锚点与替换文与它相同、点名改名后的测试（实现员报告 `/tmp/claude-1000/impl-e165-whole-row/patch/report.md` 第 67 行的说法，我核了第 1575 行的锚点与替换文，没找到删之前的那一行原文：HEAD 的 `crates/mutations.tsv` 里没有任何 E165 行，它是还没提交的工作区里加的）。删得对，推的。

## 没打中的形状

| 攻的地方 | 试过的形状 | 取样范围 |
|---|---|---|
| E165 交叉核 | 五对列对表头；今天钉表逐行五对；8、0–7、9–20、22+ 段；只含空白与制表符的行；CRLF；行首空格加 `#` | 钉表 195 行（awk）；其余读代码 |
| 驱动行零态 | 造 0 态的计划 | 读 `plan_crash_points`，够不着 |
| 驱动行全复用（不叫停） | 同一份库第二趟 | 1 条流 57 态，`states_reused=57` 照实 |
| ignore 全跳过 | `exhaustive`、`states_skipped_by_ignore` 的算法 | 读代码，没造 ignore 的流跑 |
| 展开上限 panic 被吞 | 全仓读这个变量的地方 | grep，三处都经 `from_environment_or` |
| 挂钟进度行 | CPU 那一支每段收到即重置计时、而这一段的行在核对线程上已经打过；GPU 那一支收到块之前卡线程已打块行 | 读代码，没找到「重置了计时却没打行」的路径 |
| GPU 非 NVIDIA 独显、集显、虚拟、其他类型 | 合成输入 7 种 | 判本次未跑，与设计一致 |
| 随机历史复现 | 六段取样 × 6 种子，两路逐项比 | 36 段 0 段不同（结局都是完成，没有真红的样本） |
| 崩溃注入复现：新发现那一类 | 种子由历史种子定（`seed ^ 盐`），「复现这一段」只改种子区间 | 读代码；红在新发现上的大档照样红 |
| 33 行变异 | 逐行读替换文对点名的断言 | 第 1588 行在副本对照上量过会红，其余推的 |

## 这条腿自己的限度

- 所有数都是草稿副本上原型跑的，不是入库装置。R1、M3 用的流是照两流用例的 `record_flow` 拷来造的，盘换成内存稀疏盘，崩溃点的文件标签是原型自己的路径（`src/common_crash_points/mod.rs`），与仓里的标签不同、A 与 B 之间一致；合入用例本身没跑（checker 档）。M3「点名的测试不会红」是从副本对照推到合入用例的。
- G1 用 `VK_ICD_FILENAMES`、`VK_DRIVER_FILES` 让 Vulkan 加载器找不到 NVIDIA 的驱动来模拟「驱动坏」，没有真弄坏驱动。正常那一趟在 0 号卡上起了一次上下文（本机两张卡都被本地模型服务占着显存，没起计算）。
- S2 的红是我在 `tree2` 里自己改坏造出来的（二次崩溃一个都不摆），用来证明大档入口判不出结构断言那一类；没有拿现成的变异行证。
- 状态量：R1 每条流 57 态、每趟原型几条流；S2 两批各 16 个崩溃状态；全部加起来远低于 10⁶。没有跑任何层 0 全枚举，`closed_form_state_count` 在 R1 的流上是 33554462（全展开），原型只按展开上限 6 跑 57 态。
- 中途有一次 `variants.sh` 的旧写法用 `rsync -a` 把副本换回原样，保留了原来的修改时刻，cargo 没重编：`keys_probe` 第一次「原样」那一趟其实跑的是改法 A 的编译产物，那份输出改名 `keys_probe-as-merged-STALE-BUILD.out` 留在草稿目录、不进模型目录；改成不留修改时刻之后重跑，原样与改坏两份 `keys_probe` 与 `reuse_key-recheck.out` 才是作数的。`variants.sh` 三种改动那一趟每次都改了源码、cargo 照新修改时刻整份重编，推的作数（`reuse_key-recheck.out` 与第一次 `reuse_key.out` 逐行相同，说明换回之后的编译是对的）。
- 包装退过几次非 0：第一次 `reuse_key` 用 12G 排不上（退 252，整机账上已占 28.2 GiB），之后跑的都用 4G 或 `auto`；`repro_probe` 一次因同会话已有一条在跑退 252、一次上限超过峰值两倍退 251，`keys_variant.sh` 一次退 251，都没跑一行，按包装的出路改了上限或加了理由重跑。
- 等后台进程时有一次用了 `pgrep -f` 取进程号（只读，没停任何进程），之后改成 `ps` 读写死的进程号。

## 没做什么

- 没跑 checker 档包的任何测试，也没跑门禁阶段；K3 的 33 行只读替换文推，没证红。
- 没判 K1、K4、K5；读到的越格线索只记一条：`common_crash_points` 在没设库变量时自己往进程环境里设 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 再删，同一个测试二进制里两条用例并行调它会共用、互删同一个库（读代码，没跑，归 K5）。
- 没读入口脚本 `research/scripts/crash-amplification.sh` 的汇总逻辑，L1 改法对汇总的影响是推的。
- 改法 A2、G1 改法、S2 改法、M3 改法都没实现。
