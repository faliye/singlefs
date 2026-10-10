# 删除线第 58 批：代码轮第一轮正文（2026-10-10）

<!-- doc-lint:not-numbers K1 K2 K3 J1 J2 J3 J4 -->

## 一、题面

这一轮攻第 58 批两份补丁，照判决 `research/prompts/m3-batch57-code-r1-main-verification.md` 第三节 J1–J4 改，已合进主工作区（底是第 57 批合入之后）。两份都只改测试，`crates/*/src/` 一行没动：

| 份 | 关的 | 落点 |
|---|---|---|
| 甲 | J3 旋钮支三片以上的坏镜像：三棵树各 [2,1,1]、[2,2,1,1]、[2,1,2,1]（后一种先转一圈根环、标慢档），只认这一棵自己那条不变量（I-9.22 / I-9.37）红；GPU 每棵树一条 [2,1,1] 对拍。J4 第 57 批墓碑产品支 GPU 对拍改造法：起点镜像取第 816 个 unlink 之前的整段录制流，写表只录那一次 unlink | harness `checker_known_bad_images.rs`、checker 档 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` |
| 乙 | J1 冷走读次序补钉：分配记录代晚于根 + 删映射根一条（只读恢复与可写挂载各一条）、分配记录代晚于根 + 这一版自己那行水位代晚于根（只读恢复一条）。J2 第 57 批 H4 两条换造法：删记账根 key 最小的一行，不碰标签与代 | harness `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` |

条款：`.claude/kb/invariants.md` I-1.10（判定顺序）、I-9.21、I-9.22；`.claude/kb/decisions/18-块里携带什么信息.md` 已定项 24 第 3 条；`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 4（统计量登记表）。I-9.36、I-9.37 定义在 `~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`。

改动全文：`research/prompts/_m3-batch58-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-batch58/knob-multi/report.md`、`~/.local/share/singlefs/work/m3-batch58/cold-walk-pins/report.md`。上一轮判决与腿报告：`research/prompts/m3-batch57-code-r1-main-verification.md`、`m3-batch57-code-r1-opus-output.md`、`m3-batch57-code-r1-sonnet-output.md`；攻方模型 `research/prompts/m3-batch57-code-r1-opus-model/`（可以拷来改臂）。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch58-code-r1/repo/`（主工作区合入第 58 批之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。被判文件与 `crates/mutations.tsv` 的 sha256 在 `research/prompts/m3-batch58-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主 agent 在主工作区跑过：`merge-and-verify.sh` 打两份补丁，「✓ 测试全绿（194 个测试二进制，共 2008 条测试）」、fmt、clippy、build 过；checker 档两趟与 8 行变异点名复跑的结果写进判决。
- 实现员自定、主 agent 接受、被攻过零轮的：甲加的第三种排布 [2,1,2,1] 与判之前转一圈根环（派发给的两种排布里少一条的都是倒数第二片）；乙 J1 ② 用「这一版自己那行水位代晚于根」造逐条判那一坏。
- 已知、记作欠账或下一批线索、报出来不算打中的：「分配记录判在条目宽之前」还没钉；旋钮中途改变条款没说；GPU 谱系记录表上限 256 条；产品支一片容器不判；跨版本容器集包含没判；「每盘一条」排在分配记录代之前；`crates/mutations.tsv` 文件头「门禁 59 号」；崩溃恢复下旋钮 ≥ 2 与抛弃段里写的容器那一形没跑到全域。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　旋钮多片坏镜像（甲 J3）
每条新用例是不是只因它要钉的那一种写错而红：有没有一条在正确判法下也会因别的不变量或别的原因红、而断言没分清？转一圈根环之后候选集里还剩哪几版，[2,1,2,1] 那一版是不是真被判到？GPU 那三条是不是真在比旋钮支？

### K2　J4 改造法
新起点镜像下崩溃状态集合是不是和原造法相同（只少了前 815 次 unlink 的写）？产品墓碑 [815, 1] 第二片改号那一形 GPU 与 CPU 判得一样吗？会不会撞内核别的读宽上限？

### K3　冷走读次序补钉与换造法（乙 J1、J2）
三张新盘是不是只由点名的那一次序分得出（有没有别的判先拒了）？删记账根 key 最小一行会不会碰到 inode 号水位行或别的判的前提，使用例在错的理由上绿？可写挂载那一条钉的是重建里的哪一个次序？

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K3 | 核「测试测的是不是条款与判决点名的那件事」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | K1–K3 | 在冻结副本的拷贝上造反例：用例在错的理由上绿或红、改坏照绿、新造法漏掉原造法罩到的状态 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 10G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；线程上限 15；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
