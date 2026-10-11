# 第 30 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers M1 M2 M3 M4 E1 B1 B2 G1 KX1 KX2 -->

## 一、题面

第 30 批（`records/2026-10-01-里程碑三形式化接入.md#30`）照判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第三节第 1 条与用户 2026-10-05 的定案（T_time 盘上单位改毫秒）改，三份补丁，改动全文 `research/prompts/_m3-formal-code-b30-r1-diff.md`：

- 30-1：新文件 `crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs` 三条用例（建到第 4 片；回退到第 3 片被重写过的那一版；回退到没有第 4 片的那一版），四个变体（胶水封顶第 3 片、模型封顶第 3 片、回退只取回第 2 片、回退只丢第 3 片）各一行变异（B1、B2）。
- 30-2：池级 checker `crates/singlefs-checker/src/walk.rs` 墓碑树 / 谱系树容器子指针判 extent 偏移（E1）、I-5.4 按版本记走没走到叶；GPU 走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 同一处 E1；GPU 恢复内核 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 种类 ≥ 32 的重复逐条比、加密 / 压缩几段（MAC / nonce、算法类型、压缩码、压后长度）照 core 补 8 处；`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 文件头；坏镜像进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，对拍进 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`。
- 30-3：`crates/singlefs-core/src/system_configuration.rs` T_time 改毫秒（`TIME_THRESHOLD_MILLISECONDS = 5000`），挂载透明 G1 期望常量（`crates/singlefs-harness/src/mount_transparency.rs`）照改，三个 harness 钉值重取。

条款：判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md`；`.claude/kb/invariants.md` I-2.4、I-5.4、I-9.16 三行；`.claude/kb/decisions/16-发布语义.md`「#### 已定项 5」；`.claude/kb/layout/01-first-txn.md` 系统配置字段表（第 149 行 T_time、第 158 行偏移说明）；用户定案原文「改成毫秒：字段表写明「单位毫秒」，默认值 5000；core 与 G1 一起改。D16 的 200 ms 写得出」。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b30-r1/tree/`（主仓第 30 批合入后的 `crates/`、`Cargo.toml`、`Cargo.lock`，sha256 清单同目录 `crates-sha256.txt`，309 行）。树里另有删除线第 20 批（`walk.rs` 删除队列、`crash_judge_common.wgsl` 找第一个文件补 nlink 等），不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录。
- 合入验证 `/tmp/claude-1000/merge-and-verify/2026-10-05-2363396/`：harness 档 164 个测试二进制全绿；fmt、`--all-targets --all-features` clippy、单开 gpu / verdict-store 两种 clippy 都过。checker 档一条没跑：GPU 对拍新加的 9 条与 GPU 变异 9 行只编过（30-2 用 naga 静态校验过）；`crash_enumeration_sharded_across_processes.rs` 第 844、850、851 行三个钉值读码会因 T_time 字节变（30-3 报告第六节），等提交时重取。
- 30-2 报告：harness 档全量约 1571 次 `check_pool_image` 新旧 I-5.4 判法只在 3 个故意坏的镜像上不同；extent 内部条目子指针与实例表链指针两处加密 / 压缩判定造不出流、没有变异行；新诊断码 65、66 只当诊断。

方案按 `crates/` 今天的实现来谈。

## 三、格

### M1　30-1：第 4 片与回退用例
三条用例罩不罩得住它们要证的行为：再换一个取样点（第 5 片及以后、回退到更旧的根、回退后不建或多建）四个变体还红不红；四行变异的锚点与已有 KX1 / KX2 那几行会不会互相掩护（同一原文、不同替换文）。

### M2　30-2：E1、种类 ≥ 32、I-5.4 按版本、GPU 加密 / 压缩对齐
E1 补的那一判有没有让合法历史误红（墓碑树、谱系树在删除进行中、回退之后、多实例下）；墓碑树 / 谱系树之外还有没有走读跟随、却不判 extent 偏移的指针（`.claude/kb/invariants.md` I-2.4 那一行说每条都判）；GPU 恢复内核种类 ≥ 32 逐条比与 core 在同一盘面上结局是不是逐格相同；I-5.4 按版本在合法历史上会不会多出「部分没判」、在坏盘上会不会放过；加密 / 压缩 8 处与 core 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 是不是同一组字段、同一判法，有没有一处 GPU 拒而 core 收或反过来。GPU 跑不了的读码、在 CPU 侧造盘面，写「推的」。

### M3　30-3：T_time 改毫秒
core 写出与读取、G1 期望与字段表第 149 行三处是不是同一个单位、同一个值；全仓还有没有按秒解这 4 字节的地方（`crates/`、`research/` 的装置）；三个新钉值的出处站不站得住；checker 档读码判不变的那些钉值，判得对不对（池级 checker 真不读偏移 433？崩溃枚举的计数、镜像摘要还有哪些吃整份系统配置槽）。

### M4　主 agent 两处
变异表那一行只改名、锚点与替换文没动，会不会让按名字复用的判定记录（`scripts/mutation/crates-mutation-rows.py`）对不上；字段表第 149 行的写法与 D16 已定项 5 的定案句对不对得上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | M1–M4 | 从判决、条款与用户定案正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | M1–M4，重点 M2、M3 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b30-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b30-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
