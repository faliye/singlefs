# 删除线第 52–54 批：代码轮第一轮正文（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 F1 F2 F3 F4 F5 M3 M4 M5 W3 -->

## 一、题面

这一轮攻三批补丁，都打在主工作区现在的样子（已含 W3 + 第 50 批）之上：

| 批 | 照什么改 | 落点 |
|---|---|---|
| 第 52 批 | 合后在副本上跑 checker 档打出的红：恢复里「核记账条目宽」排到「数记账条目」之前（W3 让坏宽度先报成条目数不对）；I-9.31 评估数；六处钉值随退役树与台阶行变 | `crates/singlefs-core/src/recovery.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 与六份 checker 档测试、一份 harness 测试 |
| 第 53 批 | 判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1–F5：M3 的独立参照与「先右」单测；新立 I-9.36（产品镜像里非最右的谱系、退役容器装满）、I-9.37（不超测试旋钮值），CPU 与 GPU 同判；Opt2 环那一项抽成私有函数 `inode_number_watermark_of_the_rollback` 与构造态单测；三张坏镜像；措辞 | `crates/singlefs-checker/src/walk.rs`、`image.rs`、GPU 判器、`crates/singlefs-core/src/mount.rs`、`inode_tree.rs`、harness 测试 |
| 第 54 批 | 第 53 批之后的两处钉值：分片用例打印行与进度文件哈希、`reuse_after_raising_the_floor` 的不适用豁免表 | 两份测试 |

条款：
- `.claude/kb/invariants.md` I-9.21、I-9.22（墓碑容器的两条，I-9.36、I-9.37 照它们的形）、I-9.31–I-9.35、I-1.7（第 32 行）；I-9.36、I-9.37 还没写回 kb，定义在第 53 批实现员报告 `~/.local/share/singlefs/work/m3-batch53/gaps/report.md`。
- `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（回退取 inode 号水位）；`.claude/kb/decisions/08-核心索引结构.md` 已定项 6（M3）；`.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24（谱系容器、测试旋钮）；`.claude/kb/layout/03-third-txn.md`（谱系 1359、退役 959）。

改动全文：`research/prompts/_m3-batch52-54-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch52/fix-reds/report.md`、`~/.local/share/singlefs/work/m3-batch53/gaps/report.md`、`~/.local/share/singlefs/work/m3-batch54/repin/report.md`。上一轮判决与腿报告：`research/prompts/m3-batch50-code-r1-main-verification.md`、`m3-batch50-code-r1-opus-output.md`、`m3-batch50-code-r1-opus-s2-output.md`、`m3-batch50-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch50-code-r1-opus-model/`、`m3-batch50-code-r1-opus-s2-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-09）

- 冻结副本 `~/.local/share/singlefs/work/m3-fixes-code-r1/repo/`（主工作区现在的样子加三批补丁；`crates/` 清单 `~/.local/share/singlefs/work/m3-fixes-code-r1/tree-files.sha256`）。被判的 24 份加 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt`。下面与腿报告里的代码行号都按冻结副本。
- 主 agent 在另一份副本（同样三批，只差 `crates/mutations.tsv` 末尾两行别的会话的 E165 变异与 E165 装置注释）上跑过：harness 档 `merge-and-verify.sh` 189 个测试二进制全绿、fmt、clippy 过；checker 档快档 61 个目标，红的只剩一条别的会话的装置钉值（`new_pool_file_creation_on_device.rs` 第 1971 行，不归这三批）；带 gpu 特性那一趟 20 个目标 493 过、0 红；`reuse_after_raising_the_floor --include-ignored` 16 过。
- 实现员自定、被攻过零轮的：恢复两处（重建、冷走读）各加一次 `every_accounting_entry_is_as_wide_as_its_field_table`；I-9.36、I-9.37 的旋钮经新入口 `check_pool_image_given_the_container_capacity_knobs` 从镜像外给，旧入口三棵容器树都当没装旋钮，每棵树按自己的旋钮判；GPU 扫描内核展开 387 154 → 387 470；`mount.rs` 抽出的私有函数；M3 参照用例在固定种子的历史上逐次比；钉值的算法（各份报告里写着）。
- 用户已定、报出来不算打中的：上一轮判决第二节那一列（Opt2、C2aL、X1–X4、R1–R3）；上一轮判决取的 K5-2 候选丙（不做贴边 unlink 用例）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　恢复里两判的次序（第 52 批）
1. 先核条目宽之后，有没有一种别的坏法（条目数不对、代段晚于根、seq 为 0、台阶行写错）交回的错误成员变了，或合法镜像（只剩台阶行、空发布、回退那次）被拒？
2. 重建与冷走读两个读者次序一致吗？可写挂载那条路呢？

### K2　I-9.36、I-9.37（第 53 批）
1. 合法镜像上误不误报：最右容器没满、根长高之后、回退照抄 cur、崩溃恢复抛弃段里写的容器、测试旋钮压小的镜像、旋钮只给了一棵树、谱系与退役混在一起。
2. 坏镜像漏不漏：提早关、容器号不连、中间一片多出一条。
3. CPU 与 GPU 判得一样吗（树表 12 条、容量上限、旋钮怎么传给 GPU）？

### K3　Opt2 环那一项（第 53 批）
抽出 `inode_number_watermark_of_the_rollback` 前后，`roll_back_by_a_forward_publish` 的行为是不是逐字节相同？三条构造态单测钉的是不是条款（D23 已定项 14）说的那件事：没被抛弃的根算、被抛弃的不算、树 ID 水位照旧取全部？

### K4　M3 的独立参照（第 53 批）
1. 参照用例真不调 `inode_tree.rs` 的合并、借、摘片吗？它与实现共用了哪些前提（阈值常量、读侧）？共用的前提错了两边会不会一起错？
2. 它在固定种子上覆盖到借后吸收、连锁吸收、先右与先左结局不同的那几形了吗？

### K5　钉值（第 52、54 批）
各处新钉值是从原理算的还是抄实测值？有没有一处钉值的变化背后是行为变了而被当成该变的（AccountingLeafSplit 形状、unlink 全量两个闭式、分片进度文件、dispatch 草稿区）？

### K6　判别力
三批新加与替换的变异行能不能拦住改回去？还有哪几种改坏的写法照绿？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K6 | 核「代码做的是不是条款说的」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | K1、K2、K3、K4、K5（K6 顺带） | 在冻结副本的拷贝上造反例：合法镜像误报、坏镜像漏报、错误成员变了、CPU 与 GPU 判得不同、钉值掩盖的行为变化 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 10G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 12；用自己的编译目录，几份变异副本各用各的。第二台机器空着，可经 `research/scripts/multi-host-run.sh --peer` 用。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
