# 删除线第 56 批：代码轮第一轮正文（2026-10-10）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 G1 G2 G3 G4 G5 G6 M3 -->

## 一、题面

这一轮攻第 56 批三份补丁，照判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G1–G6 改，已合进主工作区（底是第 52–55 批与根环线第二十六块合入之后）：

| 份 | 关的 | 落点 |
|---|---|---|
| 甲 | G1 恢复两个读者里逐条判记账条目代与 seq 排到数条目之前；G5 回退取树 ID 水位抽成私有函数 `tree_identifier_watermark_of_the_rollback` 与构造态单测 | `crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/mount.rs`、harness `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` |
| 乙 | G2 墓碑（I-9.21 / I-9.22）、谱系与退役（I-9.36 / I-9.37）容器号三判：首片 1、相邻 +1（替掉严格递增）、每片至少一条，CPU 与 GPU 同判；G3 补会红的用例；G6 旋钮写成 WGSL 常量传进 GPU（墓碑、谱系、退役三个），新入口 `JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs` | `crates/singlefs-checker/src/walk.rs`、`image.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crash_judge_checker_common.wgsl`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs`，测试三份 |
| 丙 | G4 M3 独立参照历史五段、比分隔 key、五形各至少一次；两条等价留档单测 | harness `inode_leaf_merge_and_borrow_against_an_independent_reference.rs`、`crates/singlefs-core/src/inode_tree.rs`（只加单测） |

条款：
- `.claude/kb/invariants.md` I-1.10（第 35 行）、I-9.21、I-9.22；I-9.36、I-9.37 还没写回 kb，定义在第 53 批实现员报告 `~/.local/share/singlefs/work/m3-batch53/gaps/report.md`，第 56 批改过的判法在乙的报告里。
- `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24（容器号从 1 起、最大号 + 1、零根时 1）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（树 ID 水位照旧取环里全部根）；`.claude/kb/decisions/08-核心索引结构.md` 已定项 6（M3）；`.claude/kb/layout/03-third-txn.md`。

改动全文：`research/prompts/_m3-batch56-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch56/recovery-rollback/report.md`、`~/.local/share/singlefs/work/m3-batch56/container-judges/report.md`、`~/.local/share/singlefs/work/m3-batch56/m3-reference/report.md`。上一轮判决与腿报告：`research/prompts/m3-batch52-54-code-r1-main-verification.md`、`m3-batch52-54-code-r1-opus-output.md`、`m3-batch52-54-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch52-54-code-r1-opus-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch56-code-r1/repo/`（主工作区合入第 56 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch56-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主 agent 在主工作区跑过：`merge-and-verify.sh` 打三份补丁，「✓ 测试全绿（194 个测试二进制，共 1973 条测试）」、fmt、clippy、build 过；checker 档带 GPU 那一趟与快档的结果写进判决。根环线在同一棵树上跑的 GPU 测试 12 份全绿。
- 三棵容器树写侧只经 `crates/singlefs-core/src/container_tree.rs` 的 `append_record_to_the_open_container`、`append_records_to_the_open_container` 两个函数追加，四份文件里 `remove`、`retain`、`drain`、`truncate` 零命中（主 agent 现查）：G2 三判压在这一条上。
- 实现员自定、主 agent 接受、被攻过零轮的：甲把冷走读的分配记录代与跨度也挪到两个数之前，两读者次序统一为每盘一条 → 分配记录代与跨度 → 条目宽 → 逐条判代与 seq → 数记账条目 → 映射条目数；`tree_identifier_watermark_of_the_rollback` 收一个不读的 cur 实例表参数（为了「只取没被抛弃的根」写得出）；乙产品支只有一片容器时三判照旧不判（报不适用）、墓碑旋钮一起接进 GPU、GPU 扫描内核展开 387 470 → 388 815；丙等价留档的族（1–4 片、每片记录数取 {1, 2, 116, 117, 118, 233}、相邻和 ≥ 234）。
- 已知、记作下一批线索、报出来不算打中的：「每盘一条」排在分配记录代之前，单盘代坏先报每盘一条；产品支一片容器不判；GPU 谱系记录表上限 256 条。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　恢复的新次序（甲 G1）
1. 新次序下，哪种坏法交回的错误成员（按 `detail` 读）变了，而它不该变？合法镜像（只剩台阶行、空发布、回退那次、多盘）被拒没有？
2. 重建、冷走读、可写挂载三条路的次序真的一致吗？checker 档坏盘战役（`bad_disk_input_campaign.rs`）钉的成员受不受影响？

### K2　容器号三判（乙 G2）
1. 合法镜像上误报：崩溃恢复抛弃段里写的容器、回退照抄、根长高、旋钮各异、墓碑树在删除与重写的历史上，有没有一条写侧路径让容器号不从 1 起、不连号或出现空容器？
2. 坏镜像漏报：三判之外还有哪种容器号或记录数的坏法漏？
3. CPU 与 GPU：三判逐位一样吗；旋钮常量为 0 的那一份（层 0 与放量）是不是就是产品入口？

### K3　旋钮进 GPU（乙 G6）
旋钮写成内核源码前的 WGSL 常量：编出来的内核与旧的产品内核在常量为 0 时是不是同一个判法；展开量钉值是不是从原理算的；带旋钮的对拍是不是真在比 GPU 与 CPU 新入口。

### K4　M3 参照与等价留档（丙 G4）
1. 五段历史与「各至少一次」断言能不能拦住 M3 的左右、借、借后吸收、分隔 key 的改坏？参照仍不调 `inode_tree.rs` 吗？
2. 两条等价测试的族罩不罩得住走得到的形状：有没有满足「相邻两片之和 ≥ 234」、而两种写法结局不同的可达形？

### K5　回退取树 ID 水位（甲 G5）
抽出前后 `roll_back_by_a_forward_publish` 行为相同吗；单测钉的是不是 D23 已定项 14 那半句；不读的参数是不是在给错误写法开门。

### K6　判别力
第 56 批新加与替换的变异行能不能拦住改回去？还有哪几种改坏照绿？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K6 | 核「代码做的是不是条款说的」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | K1、K2、K4、K5（K3、K6 顺带） | 在冻结副本的拷贝上造反例：合法镜像误报、坏镜像漏报、错误成员变了、CPU 与 GPU 判得不同、等价留档的族漏了可达形 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 10G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
