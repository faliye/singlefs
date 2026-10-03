# 诞生代号下界：第一轮正文（2026-10-01）

<!-- doc-lint:not-numbers V1 F1 F2 F3 -->

## 一、题面

m3-inv-b-r1 判决（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）：诞生代号下界（I-3.5 ② 按回退候选集判）两条腿都判出局。
- 正推模型上 379 条合法镜像红。
- 攻方：管理员回退之后回退目标离开候选集的镜像，两批随机历史 2 976 张里 173 张红，池级 checker 同一张上 0 条红。
- 攻方报的根因：I-3.5 ①「引用集合连续无洞」与「回退是一次向前发布」的定案对不上。

攻方提了收窄形态 **V1**：回退根继承整棵用户可见树时，下界不判（零轮）。用户 2026-10-01 定「另立一题论证 V1」。

这一轮要答：下界还立不立；立的话取 V1 还是别的形态；I-3.5 ① 那条要不要跟着改。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- 腿读冻结副本 `/tmp/claude-1000/m3-batch4-freeze/tree/`：主工作区 2026-10-01 的 `crates/`、`Cargo.*`、`.claude/kb/`，sha256 在 `research/prompts/m3-birth-floor-r1-snapshot/tree-sha256.txt`。
- 不变量 I-3.5 在冻结副本 `.claude/kb/invariants.md` 那一行（行号现查）。管理员回退是一次向前发布：D23（journal 的角色与格式） 已定项 14 与 D16（发布语义） 已定项 1 的现行条文。
- 实现在 `crates/singlefs-core/src/mount.rs` 的 `roll_back_by_a_forward_publish`，checker 的候选集在 `crates/singlefs-checker/src/walk.rs`。
- 上一轮两条腿的报告与模型：
  - `research/prompts/m3-inv-b-r1-sonnet-output.md`（B3 一节、`m3-inv-b-r1-sonnet-model/` 的 B3 模型）；
  - `research/prompts/m3-inv-b-r1-opus-output.md`（B3 一节、`m3-inv-b-r1-opus-model/` 的 `proto.diff`、`rerun.sh`）。

## 三、格

### F1　下界在哪类合法镜像上红
把上一轮两份红的镜像归类：哪些是「回退根继承整棵用户可见树」，哪些不是。V1 收窄之后还红不红。

### F2　V1 的措辞与判法
写全：「继承整棵用户可见树」在镜像上怎么认（看回退根与回退目标的哪几个字段）；checker 怎么判；一份只红它的坏镜像怎么造。

### F3　I-3.5 ① 要不要改
「引用集合连续无洞」与「回退是一次向前发布」的冲突是不是真冲突；改的话改成什么。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | F1–F3 | 从条款与冻结副本推 V1 的措辞、判法与 I-3.5 ① 的改法 |
| 云端攻方（Opus） | F1、F2 为主 | 在冻结副本的拷贝上，沿用上一轮攻方的历史生成器，造 V1 收窄之后仍在合法镜像上红的、或在坏镜像上漏红的 |

第一轮，不派辩方。

## 五、跑前写死的判据

- V1 在任一合法镜像上红 ⇒ V1 出局。
- V1 写不出一份只红它的坏镜像 ⇒ 下界不立，状态列写明。
- I-3.5 ① 的改法会让任一合法镜像红 ⇒ 不改。
- 腿提的新形态算被攻过零轮。

交付：
- 腿的报告写 `research/prompts/m3-birth-floor-r1-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；
- 攻方的副本与用例放 `research/prompts/m3-birth-floor-r1-opus-model/`；草稿放 `/tmp/claude-1000/<腿名>/`；
- 引 kb 条款写 kb 文件自己的行号（去冻结副本里那份 kb 现查）；方案按 `crates/` 今天的实现来谈。
