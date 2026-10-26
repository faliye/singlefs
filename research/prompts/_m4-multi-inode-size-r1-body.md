# m4-multi-inode-size-r1：截断与变长从第一个文件推广到任意活 inode（2026-10-26）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 K1 K3 K5 K9 -->

## 一、题面

里程碑四待优化表「多 inode 大小变更」那一行（`.claude/kb/milestone/04-fourth-txn.md`，表头「待优化」下行首「多 inode 大小变更」），用户 2026-10-26 定这次做，并定推广范围：「任意活 inode 都能缩短与变长」，core 别处只认第一个文件的地方仍留里程碑十的一·3（C618）。这一轮要判的是：在这个射程下，别的文件的洞、截断续做与计划、截断条目至多一条、对拍对象、现有拒绝边界各怎么定，才能写成可实现、可验的条款。

被判的条款：`.claude/kb/decisions/08-核心索引结构.md` 已定项 5（含「截断入口边界」那一句）、已定项 14（标签 0 的定义句与两条用户定案）；截断判决 `research/prompts/m4-trunc-r1-main-verification.md`（K1、K3、K5、K6、K9 与第六节 S1–S5）、`research/prompts/m4-trunc-r2-main-verification.md`（R2–R4）。

这一行的边界是零轮定的：`crates/singlefs-harness/src/model_comparison.rs` 那一句注释「第一版只有第一个文件写数据……主 agent 定」（`records/2026-10-13-里程碑四预开发.md` 第 308 行「截甲交回」第 ① 处）；它的理由在按 inode 号写数据的两个入口落地后不成立，但撤回的理由失效不等于边界自动撤掉，放开要在这一轮重新论证。

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

背景事实来自调查员报告 `~/.local/share/singlefs/work/m4-close-optimization-three/result.md`「三　多 inode 大小变更」一节，下面几条主 agent 现查过：

- `crates/singlefs-core/src/transaction.rs:10457` `publish_size_change`；不是第一个文件而新 size ≠ 旧 size 拒成 `TruncationRefusal::SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses`（成员定义 `:5555`）。
- 截断续做只认第一个文件：`transaction.rs:10604` `let names_the_live_first_file = intent.inode.0 == FIRST_INODE_NUMBER`，`:10610` 不是就拒。
- 别的文件的版本结构 `FileOfAnInodeOtherThanTheFirst`（`transaction.rs:3673`，文档 `:3671`「截断与变长对它拒」）不许有洞；冷恢复对第一个文件之外的文件有洞同样拒（`crates/singlefs-core/src/recovery.rs` 里「第一版不支持」那一处，调查员报告点名 `:5939`–`:5945`）。
- 已泛化、可照搬的：unlink 的批按 inode 取剩下几个单元（调查员报告点名 `transaction.rs:10391`–`:10400`，判决 m3-multi-file-unlink-r1 M1）；池级 checker 截断那几判按每条 inode 记录判（`crates/singlefs-checker/src/walk.rs`，报告点名 `:12804`）；harness 模型已有按 inode 号写（`crates/singlefs-harness/src/model.rs`）。
- RefFS 只有第一个文件（`crates/singlefs-reference-filesystem/src/lib.rs` 的 `Operation`）；翻译层对按 inode 号写停下（`crates/singlefs-harness/src/reference_filesystem_translation.rs`「RefFS 第一版只有第一个文件」）。
- 钉住边界的用例与变异：`crates/singlefs-harness/tests/truncation_intent_batches_and_final_batch.rs`、`truncation_random_histories_against_the_model.rs`；`crates/mutations.tsv` 里锚在拒绝成员与 `is_the_first_file` 上的 3 行。
- 撞文件：`transaction.rs`、`mount.rs` 正被 GPU 垃圾回收会话第 39 批挪线程的实现员改，实现排在它交回之后；这一轮只判条款，不受影响。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### S2　别的文件的洞怎么表示
截断判决 K5 的标签 3、4、0 写法不分 inode；D8 已定项 14 的标签 0 定义句按「第一个文件」写，而别的文件 size 0 时上段里根本没有条目（`transaction.rs` 文档「只写过建 inode（size 0）的 inode 没有数据单元，不在这里」）。格里要问：别的 inode 从 size 0 变长到 size > 0 而一个单元都没有时，写不写一条标签 0 的上段条目；写或不写，三处读者（冷走读、可写重挂与冷恢复、挂载态读者）、池级 checker 与 GPU 恢复内核各怎么判；`FileOfAnInodeOtherThanTheFirst` 与冷恢复那一处拒绝怎么改，改后既有坏镜像会不会多红（截断判决 S3「只红自己」那一类）。

### S3　截断续做与截断计划对别的文件
截断 extent 计划（K3）是截断自己写的。格里要问：照 unlink 已泛化的按 inode 取单元那一份能不能直接搬；别的文件留下 144、288 个单元时罩着被重写单元的那片叶会不会漏重写（K9 那条叶边界搬到别的文件上）；续做入口按 inode 认领时，回退取 R_old 的截断条目（K1）怎么认。

### S4　「全池活截断条目至多一条」（K1）留不留
格里要问：串行下会话每次改动先走空，是否推得出同时最多一条；管理员回退取 R_old 的截断条目时能不能合出两条（两个不同 inode 各一条）；留的话写成哪条可判的规则，放开的话哪几处读者与 checker 要改。

### S5　对拍对象
验收写「多 inode 独立缩短/变长、快照共享、意图续做、关闭重挂与崩溃后读回对拍」。格里要问：只对 harness 模型够不够；RefFS 扩成多文件是不是这一行的前提，还是 C618（里程碑十的一·3）的一部分；只对模型时崩溃后读回的允许集合由谁给。

### S6　现有拒绝边界的用例与变异
待优化表写「不把现有拒绝边界直接当作回归」。格里要问：拒绝成员 `SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses` 删掉，还是留给别的情形；钉它的用例与 3 行变异改成正例的哪种写法才不丢判别力。

### S7　射程之外还剩什么只认第一个文件
用户把 core 别处只认第一个文件的地方留给里程碑十的一·3。格里要问：要让「任意活 inode 缩短与变长」在快照共享、意图续做、关闭重挂、崩溃恢复四样上都成立，`crates/singlefs-core/src/` 里哪几处只认第一个文件的代码必须一起改（不改就让这一行的验收不成立），哪几处可以留给一·3；给出清单与判法（`grep -rn FIRST_INODE_NUMBER crates/singlefs-core/src` 今天 61 行）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | S2–S7 | 从截断两轮判决、D8 已定项 5 与 14、今天的实现推每一格的候选与它要满足的条件，每格给推荐与它依赖的前提；S7 给清单；每条结论写文件:行号 |
| 云端攻方（Opus） | S2–S7 | 在冻结副本的拷贝上造反例：放开边界之后某个读者、checker 或 GPU 内核在合法镜像上判红，或在非法镜像上判绿；S4 造回退合出两条截断条目的历史；S7 找清单漏掉、会让验收不成立的那一处。每个打中给可复现的命令与原样输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；冻结副本路径在派发提示里给。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；拷冻结副本，不在冻结副本里改；每棵树各用一个编译目录，跑完删。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；线程上限照派发提示。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
