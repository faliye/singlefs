# m3-close-b19-code-r3：批 19 验证段代码轮第三轮（最终轮）正文（2026-10-29）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 -->

## 一、题面

前两轮判决：`research/prompts/m3-close-b19-code-r1-main-verification.md`、`research/prompts/m3-close-b19-code-r2-main-verification.md`。这一轮是最终轮：只攻前两轮站住的形态与第二轮判决的修补（改动 `research/prompts/_m3-close-b19-code-r3-diff.md`：`crates/singlefs-core/src/mount.rs`；修补报告 `~/.local/share/singlefs/work/m3-close-b19-code-fix2/result.md`），这一轮里新冒出来的零轮形态不再开轮，写进判决的交用户表。第三轮分工照 `.claude/rules/three-way-inference.md`「多轮」一节：已定的撑不撑得住、未定的清单完不完整、前两轮报出的数字能不能核。前两轮的改动全文在 `research/prompts/_m3-close-b19-code-r1-diff.md`、`_m3-close-b19-code-r2-diff.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-29）

方案按 `crates/` 今天的实现来谈。第二轮修补已打在主工作区（没验证；修补员在第二台跑过两个改到的测试二进制整份、退 0）：

- `crates/singlefs-core/src/mount.rs`：续做预演判两本账之前，从读坏账里剔掉读补写盘系统配置槽的那几条（新函数 `read_back_a_system_configuration_slot_of_a_device_rewritten_by_the_acquisition`）；读错账没剔（修补员理由：逐盘核只收两槽都读得出的盘，之后那两槽再报读错是新的故障）。
- 第一轮修补的形态照旧：逐盘核在续做预演之前（`establish_instance`），公开入口 `raise_rollback_floor_to_the_admission_ceiling` 交回 `NoValidRoot`，断言在 `push_one_floor_raise_within_the_admission_budget`。
- 新用例：`crates/singlefs-harness/tests/writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device.rs`（加了拒因点名树表、两种拒同时成立报逐盘核那一拒两条），`acquisition_rewriting_a_device_without_a_self_verified_system_configuration_rolls_back_on_a_write_failure.rs`（回卷后下一次挂载补写盘表为空）。

## 三、格

### V1　剔读坏账的改法撑不撑得住
剔掉的只是「读补写盘系统配置槽」那几条：认法（`read_back_a_system_configuration_slot_of_a_device_rewritten_by_the_acquisition`）会不会剔多（剔掉补写盘上别的块、别的盘的系统配置槽）或剔少；读错账不剔这一取舍成立不成立（逐盘核之后、取号之前那两槽读错是不是一定是新故障）。

### V2　逐盘核在预演之前这一形撑不撑得住
前两轮站住的 T4 改法一：两种拒同时成立报逐盘核那一拒；逐盘核读盘的读错读坏不进预演的两本账。攻这一形在多盘（两块以上补写盘、补写盘与读坏盘不同）上还成立不成立。

### V3　C929 照今天不判
第二轮定的候选一：回卷之后那块盘走落后支、补写闸三、四、五不再判。核它与 C874 已定的五件闸、D22 那一行有没有矛盾。

### V4　未定的清单完不完整
前两轮判决「条款没写」的那几句（D16 槽 0 钉值与平局、D23 已定项 14 两条前提与隐含前提、C874 取法与字段名、C690 越过不扫三形、C928 解冻后、逐盘核与预演的先后、C929 一句）是不是全了；有没有前两轮改了行为而没列进写回的。

### V5　前两轮的数能不能核
前两轮判决引的数（第一轮攻方探针五行输出、第二轮辩方 13 步、T5 的 420 次挂载第 366 行触发、U2 拒因前后的偏移与长度）：挑两三个在今天的树上能不能复现，复现不了的写明。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | V1–V5 | 逐格核代码与前两轮判决的结论一致，V4 列全写回清单，V5 复现两三个数 |
| 攻方腿（Opus） | V1–V5 | 攻站住的形态哪一处撑不住；不重复前两轮攻过的角度；新冒出的零轮形态只列，不要求实现 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建探针的，放 `research/prompts/m3-close-b19-code-r3-<模型>-model/`（带 `tools/rerun.sh`，接受 `--repo`、`--frozen`、`--work`），草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。
- 不跑重型测试、不起 checker 档的 cargo、不碰 GPU；探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，本机排不上就经 `scripts/hosts/multi-host-run.sh --peer` 起。
- 报告写进 `research/prompts/m3-close-b19-code-r3-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
