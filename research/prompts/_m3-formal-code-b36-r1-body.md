# 第 36 批代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 R1 R2 R3 -->

## 一、题面

第 36 批（`records/2026-10-01-里程碑三形式化接入.md#36`）照第 35 批代码轮判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第三节第 1 条改，改动全文 `research/prompts/_m3-formal-code-b36-r1-diff.md`：

- 36-1b：`crates/singlefs-checker/src/walk.rs`——R1：「下一次挂载先施加的那一版」改取 I-4.1 那份重放 `replay_the_next_mount_would_do` 施加的第一次发布、取它末条那一份新根段（`ReplayTheNextMountWouldDo` 的 `Applied` / `Stopped` 各多带第一次发布的末条；一次都不施加、锚点认不定、所选根 txg 没有后继时不认），I-3.10 与 I-5.4 共用；R3：FL 那段说明照 R1 重写，I-5.5 新理由收窄到「池里的盘上都没有数据单元」。`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`——R2：「拿掉根槽写」两条流改成合法造法录流。`crates/singlefs-harness/tests/checker_known_bad_images.rs` 两条新用例（断号；后面一次发布停在点名单元上仍取第一次）。`crates/mutations.tsv` 追加 2、替换 2。
- 36-2：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 走读内核的重放改名 `replay_the_next_mount_would_do`，多交回第一次施加的发布的末条，经交接段空着的第 31 个字交给扫描内核；`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `record_row_of_the_version_the_next_mount_applies_first` 读它，I-3.10、I-5.4 同用；`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 随改；对拍用例加两条流；`mutations.tsv` 追加 2、替换 2。

条款：`.claude/kb/invariants.md` I-3.10、I-4.1、I-5.4、I-5.5 几行；`.claude/kb/decisions/23-journal的角色与格式.md`「#### 已定项 14」「#### 已定项 15」；`.claude/kb/checks-owed.md` C735。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree/`（第 36 批合入后，清单同目录 `crates-sha256.txt`，316 行）。树里另有删除线第 28–29 批与根环线 10o / 10p，不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的工作目录（`~/.local/share/singlefs/work/<腿名>/`），每份副本用自己的编译目录，编完核日志里被测包确有 `Compiling`。
- 合入验证 `~/.local/share/singlefs/work/merge-and-verify/2026-10-06-1923410/`：harness 170 个测试二进制全绿，fmt、clippy 过；mutation-tables 只红 `crates/mutations.tsv` 第 2919 行（删除线第 26 批那一行，不归这一轮）。
- 实现员在副本上证红：36-1b harness 档 2 行抓到；36-2 的 2 行 GPU 变异与 4 行替换没证红（checker 档）；36-2 换基之后只跑了 fmt、clippy，合入验证补齐了编译与 harness 测试。GPU 没跑。
- 根环线报：各线合入后扫描内核 `crash-judge-checker-scan` 展开 700 783 个表达式、超预算 400 000，GPU 对拍编之前就拒；谁推过线根环线在量，不归这一轮判，涉及 GPU 的结论一律「推的」。

方案按 `crates/` 今天的实现来谈。

## 三、格

### P1　R1（CPU）
`version_the_next_mount_applies_first` 取的第一次发布与 core 恢复真施加的第一次发布是不是同一份（读 `crates/singlefs-core/src/recovery.rs` 施加那一段对照）：断号、点名单元停在第一次或后面某一次、锚点认不定、所选根 txg 没有后继、几个实例并存、回退之后几种历史各走哪一支；`Stopped` 带第一次的那一支在什么历史上走到；I-3.10 与 I-5.4 是否仍同一个认法；合法历史上多报少判。

### P2　R1（GPU）
36-2 选的乙（经交接段第 31 个字传下标）：写那个字的路径是不是先于扫描内核开工、所有能置 `H_VALID == 1` 的路径都写了它；走读内核 I-4.1 那一处行为是否不变；扫描内核读到的那一行与 CPU 认的是不是同一条记录（GPU 跑不了的读码，写「推的」）。

### P3　R2
两条流的新造法与合法历史逐字节一致吗（b35 攻方量过的 W 与系统配置槽两处）；末态判定与原造法相同吗；两条流名字说的那一形还在不在。

### P4　R3
三处说明与 I-5.5 理由的措辞与判定是否一致。

### P5　变异行
追加 4 行、替换 4 行：锚点各恰好命中一次、点名的测试跑得到、合入后基线绿、红在断言上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | P1–P5 | 从判决与条款正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | P1–P5，重点 P1、P2 | 找反例：一段合法历史让新认法误红或少判，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮；条款没写、要用户定的，交用户。
- 重型测试不跑：checker 档的测试、GPU 对拍与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b36-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b36-r1-<腿名>-model/`，带一份 `tools/rerun.sh`（每条 cargo 经 `scripts/hosts/run-with-memory-cap.sh`，上限按峰值表给或写 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`，同一时刻只起一条）；草稿放 `~/.local/share/singlefs/work/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
