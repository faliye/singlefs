# 删除线第 57 批：代码轮第一轮正文（2026-10-10）

<!-- doc-lint:not-numbers K1 K2 K3 K4 H1 H2 H3 H4 -->

## 一、题面

这一轮攻第 57 批两份补丁，照判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H1–H4 改，已合进主工作区（底是第 56 批与根环线 G、G2 合入之后）：

| 份 | 关的 | 落点 |
|---|---|---|
| 甲 | H1 装了旋钮那一支（墓碑 I-9.22、谱系与退役 I-9.37）加判「不是最右那片记录数 == 旋钮值」，CPU 与 GPU（三棵树共用的 `d_judge_containers_under_the_capacity_knob`）同判，扫描内核展开 395 434 → 395 566；H2 产品支（I-9.21、I-9.36）三判的会红用例：CPU 直接调判法的单测，GPU 只补了墓碑产品支一条对拍（谱系 / 退役产品支调用点补不上：`D_MAX_LINEAGE_RECORDS` 256，主 agent 判并进已有欠账）；H3 首片容器号 0 的坏镜像 | `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crash_judge_dispatch.rs`，测试三份 |
| 乙 | H4 冷走读「分配记录代与跨度」排在两个数之前：一张只有这一次序分得出的坏镜像（台阶行标签改成没登记的 13 让记账条目数对不上，同时两盘那一槽分配记录代晚于根），只读恢复与可写挂载各一条 | harness `corrupt_on_disk_content_is_refused_instead_of_panicking.rs`（`recovery.rs` 没改） |

条款：
- `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24 第 3 条（满了才关、关闭路径加只供测试的容量旋钮）与第 1、5 条（容器号从 1 起）。
- `.claude/kb/invariants.md` I-1.10（判定顺序）、I-9.21、I-9.22；I-9.36、I-9.37 还没写回 kb，定义在 `~/.local/share/singlefs/work/m3-batch53/gaps/report.md`、`~/.local/share/singlefs/work/m3-batch56/container-judges/report.md`，第 57 批改过的在 `~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`。

改动全文：`research/prompts/_m3-batch57-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`、`~/.local/share/singlefs/work/m3-batch57/cold-walk-order/report.md`。上一轮判决与腿报告：`research/prompts/m3-batch56-code-r1-main-verification.md`、`m3-batch56-code-r1-opus-output.md`、`m3-batch56-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch56-code-r1-opus-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch57-code-r1/repo/`（主工作区合入第 57 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch57-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主 agent 在主工作区跑过：`merge-and-verify.sh` 打两份补丁，「✓ 测试全绿（194 个测试二进制，共 1999 条测试）」、fmt、clippy、build 过；checker 档两趟与 GPU 4 行变异的结果写进判决。
- 实现员自定、主 agent 接受、被攻过零轮的：甲 H1 只加在三棵树共用的那一个 GPU 函数里；乙用没登记的统计量标签 13 造「条目数对不上」（读者今天不判标签）。
- 已知、记作欠账或下一批线索、报出来不算打中的：GPU 谱系记录表上限 256 条使产品容量谱系树 I-9.36 GPU 判不了（含 H2 谱系 / 退役产品支调用点没有对拍）；产品支只有一片容器时不判；跨版本容器集包含没判；「每盘一条」排在分配记录代之前；`crates/mutations.tsv` 文件头「门禁 59 号」；崩溃恢复抛弃段里写的容器那一形没跑到全域。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　旋钮支装满（甲 H1）
1. 合法镜像上误报：旋钮下最右那片没满、根长高、回退照抄、换实例重挂、旋钮各异、只给一棵树装旋钮，新判红不红？
2. 坏镜像漏报：旋钮下早关、多一条、最右之外有一片少一条，三棵树各漏不漏？CPU 与 GPU 判得一样吗？

### K2　产品支三判与首片 0（甲 H2、H3）
新用例能不能拦住产品支三判每一句的改坏、首片判 ≤ 1 的改坏？直接调判法的单测与整镜像的判法走的是不是同一段代码（会不会单测绿、整镜像那条路却没判）？

### K3　冷走读次序（乙 H4）
那张坏镜像是不是真只由次序分得出（有没有别的判先把它拒了，使用例在错的理由上绿）？用没登记的标签 13 造数不对，会不会让用例钉住的是标签而不是次序？重建那一条钉的是哪一个次序？

### K4　判别力
第 57 批新加的变异行能不能拦住改回去？还有哪几种改坏照绿？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K4 | 核「代码做的是不是条款说的」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | K1、K2、K3（K4 顺带） | 在冻结副本的拷贝上造反例：合法镜像误报、坏镜像漏报、用例在错的理由上绿、CPU 与 GPU 判得不同 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 10G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
