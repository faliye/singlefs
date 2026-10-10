# 第 37 批代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers B1 B2 B3 B4 -->

## 一、题面

第 37 批（`records/2026-10-01-里程碑三形式化接入.md#37`）是主 agent 自己写的几处小改，只动注释与 checker 档装置二进制的内联单测，改动全文 `research/prompts/_m3-formal-code-b37-r1-diff.md`：

- `crates/singlefs-checker/src/image.rs` 第 211–212 行：I-4.1 常量的文档注释里点名的函数 `version_the_next_mount_applies` 仓里不存在，改成 `walk.rs` 里 I-4.1 那份只读重放的真名 `replay_the_next_mount_would_do`（判决 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 交下一批的那一条）。
- `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的内联单测 `every_criterion_has_a_positive_control_and_every_control_is_red`：全表那一圈原来只取 r3 档（记录表 `RecordTableSource::Direct`）的读数，`Criterion::ALL` 里 J8 五行只在 r5 档判（`judges_j8`），T_table、T_row 只在两份记录表都取的那一档判（`compares_the_two_record_tables`），所以这条单测在主工作区必红（根环线报的日志 `~/.local/share/singlefs/work/m3-r1/merge-10z-i/checker-quick.log` 第 407–419 行）。改成 r3、r5、两份表都取的 r3 三档读数并起来判全表，文档注释随改。
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 两条内联单测改钉值：五段操作数第一段 23 → 25（harness `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 第 347–349 行已改，装置没跟）；`publish_writes` 那一行加已删 inode 树、墓碑树、谱系树节点三种写（各 0），放在记账树与中央映射树之间（`crates/singlefs-core/src/write_accounting.rs` 第 103–107 行 `WrittenStructureKind` 的报告次序）。
- `crates/mutations.tsv` 追加两行（主工作区第 3383、3407 行）：「E165 全表对照只取 r3 一档（J8 五行无对照）」「E165 全表对照不取两份表都取的那一档（T_table、T_row 无对照）」。

条款：E165（恢复重放有界穷举与变异杀伤界）的阳性对照条款，`research/prompts/e165-preregistration.md` 第 5.4 节与作废条款 V2、`research/prompts/e165-r2-prereg.md` 第 5.5 节、`research/prompts/e165-r5-prereg.md` 第 5.4 节；I-4.1（重放后树可遍历）。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree/`（取自主工作区，清单同目录 `crates-sha256.txt`，336 行）。树里另有根环线第 10z 批、删除线第 37–45 批等别的线的改动，不在这一轮被判；同一文件里别的线的块（例：`image.rs` 第 282 行）不判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。
- `replay_the_next_mount_would_do` 在冻结副本 `crates/singlefs-checker/src/walk.rs` 第 8878 行。
- 主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 跑过：E165 那条单测 `1 passed; 0 failed`；nff 装置 17 条内联单测全绿；两行新变异在副本上 `prove-red.sh` 都「抓到」（日志 `~/.local/share/singlefs/work/e165-control-test-fix/`）；`crates-mutation-rows.py static-check` 整表 `CHECKED 3405 322 10`。
- 这几条都是 checker 档装置二进制的内联测试，不在 checker 档快档里；点名它们的已有变异行（`crates/mutations.tsv` 第 72 行等），在它们红着期间的「抓到」不作数。

方案按 `crates/` 今天的实现来谈。

## 三、格

### B1　E165 全表对照
三档并起来是不是恰好罩住 `Criterion::ALL` 每一行、而且每一行的对照都在**判它的那一档**里红（一行在 A 档有读数、在 B 档才判，并起来会不会让一条没判的读数顶替）；三档与 E165 正式跑用的档与 `--table` 取法对不对得上（读三份登记与驱动 `research/scripts/` 里起这个装置的那份）；r1、r2 那两段原样，r2 那条「不判 J6Qn 拆行」的断言还成不成立；`Criterion::ALL` 里有没有一行三档都不判、这条单测因此仍会红或被放过。

### B2　nff 钉值
25 与 harness 那份是不是同一个量（同一条路、同一段切法）；三种写的名字与次序与 `describe_publish_writes` 现在打的、与 `WrittenStructureKind` 的报告次序是不是逐个相同；这两处之外同一文件的别的内联单测还有没有对着旧值的。

### B3　image.rs 注释
改后的函数名与那一句说的行为（一条记录都不施加时报不适用、前缀停在点名单元验证不过时报不适用）对不对得上 `walk.rs` 里 I-4.1 的判法。

### B4　变异行
两行锚点各恰好命中一次、点名的测试筛得到、改回去红在断言上；三档里的第一档（r3 Direct）被改掉、或 r1、r2 两段被改坏，有没有变异罩着；nff 两处钉值有没有变异罩着、要不要补。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | B1–B4 | 从条款与驱动正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | B1–B4，重点 B1 | 找反例：一种坏法让这条单测放过（判据没对照、对照在不判它的那一档里读数），或三档之外还漏了一档 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮；条款没写、要用户定的，交用户。
- 重型测试不跑：这一轮被判的全是 checker 档装置二进制的内联测试，腿一样都不跑，读码判，要跑的写成命令交主 agent；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b37-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b37-r1-<腿名>-model/`（有就放，带一份 `tools/rerun.sh`，每条 cargo 经 `research/scripts/run-with-memory-cap.sh`）；草稿放 `~/.local/share/singlefs/work/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
