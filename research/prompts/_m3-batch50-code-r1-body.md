# 删除线 W3 + 第 50 批（M3、叶死亡退役树）：代码轮第一轮正文（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 K7 W3 M3 R1 R2 R3 X1 X2 X3 X4 Y1 Y2 N1 -->

## 一、题面

这一轮攻删除线合进主工作区的那一份补丁（W3 与第 50 批合起来，70 个 `crates/` 文件，变异表追加 71 行、整行替换 45 行、删 2 行）：

| 件 | 照什么改 | 主要落点 |
|---|---|---|
| W3 | 判决 `research/prompts/m3-w3-r3-main-verification.md` 第六节用户定案：Y1 Opt2（管理员回退那次的 inode 号水位 = max(cur 那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)；树 ID 水位照旧取环里全部根）；Y2 C2aL（记账在水位变了的每次发布留台阶行；谱系记录带 cur 水位、按「等于」判、从 cur 读；谱系记录 16 → 24 字节；新不变量；I-3.13 对台阶行的例外） | `crates/singlefs-core/src/mount.rs`（回退取水位、`highest_inode_number_watermark_in_the_ring`、`version_to_publish_the_rollback_after`、`roll_back_by_a_forward_publish`）、`write_accounting.rs`、`records.rs`、`lineage_tree.rs`、`recovery.rs`、`admission.rs`、`crates/singlefs-checker/src/walk.rs`、GPU 判器 |
| 第 50 批 M3 | 判决 `research/prompts/m3-leaf-death-r3-main-verification.md` 第五节 X1–X4 与借法均分；X4 N1：借时分隔 key 取孩子这一版的最小 key，删 I-9.4 第三句 | `crates/singlefs-core/src/inode_tree.rs`、`transaction.rs`、`crates/singlefs-harness/src/model.rs`、`model_comparison.rs`、`walk.rs`、`crash_judge_checker.wgsl` |
| 第 50 批退役树 | 判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1–10 条与第六节 R1–R3 用户定案（R2「两样都做」：ckpt_cost 照谱系单列一项，另加墓碑式预付与 W 加 6 + H_退役；R3 删空、吸收只重写内容变了的叶与祖先） | `records.rs`、`container_tree.rs`、`tombstone_tree.rs` 一族、`transaction.rs`、`admission.rs`、`mount.rs`、`recovery.rs`、`walk.rs`、GPU 判器四份 `.wgsl` 与 `crash_judge_gpu.rs`、`crash_judge_dispatch.rs` |

条款（kb 文件自己的行号，写回已经做完）：
- `.claude/kb/decisions/03-空间分配.md` 已定项 2（第 50 行起，标签 6 的式子）；`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 2（第 45 行起，台阶行例外）；`.claude/kb/decisions/08-核心索引结构.md` 已定项 6（第 152 行起）、已定项 9（第 295 行起，种类码 13）、已定项 11（第 341 行起，树 ID 65548、水位 65549）；`.claude/kb/decisions/18-块里携带什么信息.md` 已定项 11（第 253 行起，打包记录类型 3 宽 34）、已定项 24（第 572 行起，谱系 24 字节、树表 12 条）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（第 355 行起，回退水位、退役树随现行）；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1（第 14 行起，W 加 6 + H_退役）、已定项 4（第 109 行起，ckpt_cost 退役项）。
- `.claude/kb/invariants.md` 第 155 行 I-3.13（记账行代段等于这一版的 txg）、第 292 行 I-9.4（容器号不超最小 key）、第 318–323 行 I-9.30–I-9.35；`.claude/kb/layout/03-third-txn.md`（谱系 24 字节、退役容器、退役记录、最后一批写账）。
- 欠账：`.claude/kb/checks-owed.md` 第 477 行 C617（删后回退留下的墓碑没有预付罩着）、第 615–621 行 C766–C772。

改动全文：`research/prompts/_m3-batch50-code-r1-diff.md`。实现员报告：`research/prompts/m3-batch49-w3-implementer-report.md`、`research/prompts/m3-batch49-w3c-implementer-report.md`（W3）、`research/prompts/m3-batch50-m3-implementer-report.md`、`research/prompts/m3-batch50-m3b-implementer-report.md`（第 50 批）。设计轮攻方原型：`research/prompts/m3-retire-r1-opus-model/`、`research/prompts/m3-retire-r1-opus-s2-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-09）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch50-code-r1/tree/`（主工作区合入之后的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`litmus`；`crates/` 339 个文件的清单 `~/.local/share/singlefs/work/m3-batch50-code-r1/tree-files.sha256`）。被判的 71 份（70 个补丁文件加 `crates/mutations.tsv`）sha256 在 `research/prompts/m3-batch50-code-r1-snapshot/crates-sha256.txt`。下面与腿报告里的代码行号都按冻结副本。
- 合后验证：在主工作区的副本上跑 `merge-and-verify.sh`，「✓ 测试全绿（188 个测试二进制）」「✓ fmt 过」「✓ clippy 过」；主工作区 `crates-mutation-rows.py static-check` 退 0，`CHECKED	3706	325	10`，变异表 3711 行。E171 装置在「主工作区 + 补丁」上编过，`stream-constants --streams 1,2,3` 甲类全 holds=true（形式化线的前置核）。checker 档快档与 GPU 那一趟、checker 档 9 行变异证红在主 agent 手里跑，结果不进这一轮材料。
- 实现员自定、被攻过零轮的（报告原文在各自第五节与停下那一节）：
  - M3「借完从借出那片接着看合并」，取 E176 M3 那一臂；先查右邻、再查左邻；
  - R3 用 `InodeLeafContainerOrigin::{RewrittenThisPublish, CarriedFrom(上一版叶序)}`：挪了位置而内容没变的片照抄上一版落点、字节与映射 key；
  - `container_tree.rs` 一次追加多条（`append_records_to_the_open_container`），重写的码 2 节点 = 被碰到的每片容器的祖先并集；
  - I-9.35 的父版取法：谱系树里 T_rb = 本版 txg 取 T_old，否则 txg − 1；不在候选集不判；候选集滤不掉被抛弃根时整条报部分没判；
  - GPU：I-9.31–I-9.35 都上 GPU（扫描内核展开 387 154，余量 12 846），候选集里有一版退役树不是零根时扫描内核记码 8、交 CPU；
  - 谱系记录三段次序 T_rb | T_old | H_cur；
  - 恢复（`recovery.rs`）要求「代段 = 根 txg 的那一行」在；
  - ckpt_cost 记账那一项按这次发布之后的行数算；
  - 对拍模型照中央映射树给记账树分裂的占槽上界。
- 停下没做的：「贴边建满后 unlink 不超卖」那条用例（`m3-batch50-m3b-implementer-report.md` 第五节第 3 条与第八节第 1 条：删除类发布之后「可用」怎么读没定，三个候选甲、乙、丙是实现员推的）。
- 判决里的零轮形态：I-9.30 草案 ③ 在台阶行与那一行同 key 之下恒成立（C769）；I-9.35 退役完备（C770）；R2 两样合起来那一形；W 加 6 + H_退役 的上界；克隆头那一半（C767）；扫描重建读侧（C768）。
- 用户已定、报出来不算打中的：Opt2 与 C2aL 的选择本身；X1 甲、X2 读法 Z、X3 意图那次、X4 N1（「分裂时右半容器号给小了、仍大于左邻号」那一类镜像上没有判据拦）；R1 开池就建；R2 两样都做；R3 并进 M3；「删 → 回退」环下墓碑式预付照样超（C617）；退役树不回收。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　W3 回退水位（Opt2）

1. `crates/singlefs-core/src/mount.rs` 回退取 inode 号水位那几行：「按实例表没被崩溃恢复抛弃」的判法和 D23 已定项 14 写的是不是同一件事？有没有一形：被抛弃的根没被滤掉（重发 inode 号），或没被抛弃的根被滤掉（水位低于某个还会被选的根，跨回退重发）？树 ID 水位是不是仍取环里全部根？
2. 环里有读不出的根、实例表行不全、回退目标就是 cur、连续两次回退、回退之后崩溃，这几形各自取到的水位对不对？

### K2　W3 台阶行与谱系

1. 台阶行与每次发布那一行同 key（stat 12、inode 树、设备 0xFFFF_FFFF、generation = txg）：读者（`write_accounting.rs`、`recovery.rs`、checker、GPU 判器）有没有一处把台阶行当成这一版那一行、或反过来？
2. `recovery.rs` 要求「代段 = 根 txg 的那一行」在：有没有合法镜像（空发布、回退那次、零根、mkfs 之后第一版、只剩台阶行）被它判坏？
3. 谱系记录 24 字节 T_rb | T_old | H_cur：H_cur 从 cur 读，有没有一形取到别的版本的水位？I-9.30 的 ①②④ 在合法镜像上会不会误报，坏镜像（历史里台阶写错、谱系 H_cur 写错）抓不抓得到？③ 恒成立（C769）这一点本身攻方可以造一张它该红而不红的镜像，交代价。
4. ckpt_cost 记账那一项按这次之后的行数算：台阶跨叶、记账树长层那一次有没有少算？

### K3　退役树与退役记录

1. 登记（种类码 13、树 ID 65548、水位 65549、树表 12 条、声明长度 2400、头 ID 65537）与新池新建文件的字节：有没有一处还按 11 条、65548 读？
2. 死亡写侧：删空摘掉、左吸收右各写一条，借不写；死亡 txg = 这次发布的 checkpoint_txg；只追加不去重。同一次发布里分裂、合并、再分裂，或者一批删掉几片、几片连锁吸收时，退役记录点名的身份对不对、条数对不对？
3. 回退照抄 cur、崩溃恢复随所选根：有没有一形让被抛弃段里死掉、所选根里还活着的容器被判死，或反过来？
4. `container_tree.rs` 一次追加多条、祖先并集重写：满了才关、根满长高、跨容器边界追加，墓碑树与谱系树走的同一份代码有没有被改坏？

### K4　M3 合并、均分借、N1、R3

1. 合并与借的判定（≤ 233 吸收、< 117 借 ⌊(右 − 左) ÷ 2⌋、先右后左、借完从借出那片接着看）：有没有一形留下低于下限又没处理的叶、两片之和超 233 被吸收、分隔 key 与孩子最小 key 对不上、key 次序破？
2. R3：挪了位置没变内容的片照抄上一版落点。有没有一形内容其实变了却被照抄（或身份变了、映射 key 不对）？删空最左一片那次新写的槽数是不是真的不随右边片数涨？
3. N1 删 I-9.4 第三句之后，I-9.12 与别的判据还拦不拦得住分隔 key 写错的坏镜像？
4. 理想模型与胶水（`model.rs`、`model_comparison.rs`）对 M3 的描述与实现是不是两份独立的算法，还是抄了同一段（对拍就失去对照）？

### K5　准入（R2 两样都做）

1. ckpt_cost 的退役项（容器 2 + 根 1 + H_退役）、中央映射树删插数加的 2 + H_退役 把、标签 6 加的 (2 + H_退役 + 1) × 16384 + 35 × n_记录、W 加 6 + H_退役：各自取哪一版的 H_退役、n_记录？一次发布至多写几条退役记录，W 罩不罩得住（一批删掉多片、连锁吸收、跨层长高）？
2. 「贴边建满后 unlink 不超卖」：在实现上造。先给出你认为「删除类发布之后的可用」该怎么读（实现员推的甲、乙、丙三种或别的），再在那个读法下造一形超卖（unlink 或它的某一批被准入放过而之后写不下、或释放空间的操作被准入拒），或者造不出来；去掉退役那几项会不会让它红。
3. 开出第一片退役容器之后常驻 3 槽与实占同时在账上（实现员第八节第 1 条附注）：换一种盘面会不会让可用变负？

### K6　checker 与 GPU 判器

1. I-9.31–I-9.35 的 CPU 判法在合法镜像上误不误报：回退越过一片叶的出生、删 → 回退 → 再删、借后吸收、克隆头、候选集里有被抛弃根。I-9.35 的父版取法（T_rb = 本版 txg 取 T_old，否则 txg − 1）在空发布、回退那次、崩溃恢复抛弃过的段上对不对？
2. GPU 判器与 CPU 逐条同判：有没有一形两边判得不同（码 8 交 CPU 那一支、退役树非零根、树表 12 条容量）？
3. I-9.16 ④ 十二种、I-9.27 加种类 13、I-1.7 退役记录宽 34：坏镜像各自只红它自己吗？

### K7　判别力

追加 71 行、整行替换 45 行、删 2 行：能不能拦住改回去？哪几种改坏的写法照绿（特别是 M3 的阈值与方向、R3 照抄的判定、死亡 txg 的取法、Opt2 的滤法、ckpt_cost 那一项用这次之前还是之后的行数）？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K7 | 核「代码做的是不是条款说的」：每格逐条对条款原文与冻结副本的行，给出对得上 / 对不上与证据；对不上的写改法落在哪一处 |
| 云端攻方（Opus） | K1、K2、K3、K4、K5、K6（K7 顺带） | 在冻结副本的拷贝上造反例：误判死亡、复活、重发 inode 号、准入超卖、释放被准入拒、合法镜像误报、坏镜像漏报、CPU 与 GPU 判得不同；K5 第 2 条必做 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。用户决定的那几步（回退目标、删哪几个、什么时候崩）放开扫，不只扫一个固定后缀。
- 本机内存几条线同时在用：每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 10G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑，不直接执行；线程上限 8；用自己的编译目录。第二台机器空着，要用就经 `research/scripts/multi-host-run.sh --peer` 起（用法在它文件头）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
