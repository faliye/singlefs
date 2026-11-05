# m3-close-b19-c699-r1 攻方腿承接（opus-b）报告（2026-11-05）

立场：攻（承接 `research/prompts/m3-close-b19-c699-r1-opus-output.md`）。这一段只修模型到编得过：模型格（skr-gap-fault、sw-crash-wrapped、rs-two-files 等）、它们的判读与预言一个字不改。子 agent 不交任务：没编、没跑，下面凡是「编得过」都是推的，待主 agent 复跑坐实。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| 任务 6816 编不过的原因 | 模型包清单少了 `singlefs-format` 一项依赖；两处报错都在拼进来的 E198（取号跳过认出生代号的前提） 装置源码里，不在攻方新加的那一段 | 读码坐实（build.log 两条 E0433 的行号对到装置源码同一行号） |
| 修法 | `model-crate-Cargo.toml` 加一行 `singlefs-format = { path = "../singlefs-format" }`，与 checker 档包清单同一条路径；源码不动 | 改了，sha256 见下 |
| 改后依赖解析得通 | `cargo tree --offline -p c699-attack-model --depth 1` 退 0，列出四项直接依赖 | 量的（草稿目录里的副本，已删） |
| 改后能编过 | 推的：原来只有这两条错；rustc 在路径解析出错之后一般照样做类型检查、一并报出，build.log 里没有别的错，所以预计补上依赖就过。不排除类型检查被挡住、补上之后冒出新错 | 推的，待主 agent 跑 |
| 模型格的预言 | 原样沿用上一条腿（下面「预言」一节抄过来） | 推的，待主 agent 跑 |

## 模型与复跑

模型目录 `research/prompts/m3-close-b19-c699-r1-opus-b-model/`（新目录，上一条腿的 `research/prompts/m3-close-b19-c699-r1-opus-model/` 没动），三份文件，sha256 是写完之后 `sha256sum` 现算的：

```
816f91b7052454355b59c89e4745bc42e9b73d88950239be8e0cb5f6dd61415e  m3-close-b19-c699-r1-opus-b-model/attack_model_appended.rs
53caa7321cfdfe4f0bb29650af1e71a920aeed3a2bbcbacdf73ecab298dce70f  m3-close-b19-c699-r1-opus-b-model/model-crate-Cargo.toml
90b19d94a7f11caa2e62456c1f5fb66b08e2a6453d1ce2ee0e833594be776195  m3-close-b19-c699-r1-opus-b-model/tools/rerun.sh
```

- `attack_model_appended.rs` 与 `tools/rerun.sh` 从上一条腿的模型目录原样拷，sha256 与上一条腿报告里写的两行一样（`816f91b7…`、`90b19d94…`）。
- `tools/rerun.sh` 不用改就指到新目录：它的模型目录默认取自己所在路径的上一层（脚本里那一句 `MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"`），从新目录起，拷进副本的就是新目录里的 `model-crate-Cargo.toml` 与 `attack_model_appended.rs`。
- 上一条腿的 `model-crate-Cargo.toml` 的 sha256 是 `e56da699…`，新的是 `53caa732…`。

### 改了什么

`diff` 上一条腿的清单与新清单的原样输出：

```
1c1
< # 攻方腿模型（三方轮 m3-close-b19-c699-r1）的包清单：tools/rerun.sh 把它拷进草稿副本的 crates/c699-attack-model/，只在副本里，不进仓。
---
> # 攻方腿模型（三方轮 m3-close-b19-c699-r1，承接腿修依赖）的包清单：tools/rerun.sh 把它拷进草稿副本的 crates/c699-attack-model/，只在副本里，不进仓。
10a11,12
> # E198 装置源码第 2981、2998 行引 singlefs_format::ROOT_RING_REGIONS 与 singlefs_format::INSTANCE_TABLE_PAGE_RECORDS；checker 档包依赖 singlefs-format，模型包照同一条路径依赖。
> singlefs-format = { path = "../singlefs-format" }
```

为什么是这一行：

- 两处报错（`~/.local/share/singlefs/work/m3-close-b19-c699-r1-main/opus-rerun/build.log`，拼好的 `main.rs` 第 2981、2998 行）都落在装置源码里。`rerun.sh` 只在装置的 `fn main() {` 前面插一行 `#[allow(dead_code)]`，装置的 `fn main() {` 在第 4765 行，所以拼好的源码前 4764 行与装置行号一样。装置原文：`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2981`：「implementation: singlefs_format::ROOT_RING_REGIONS,」，`crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs:2998`：「implementation: singlefs_format::INSTANCE_TABLE_PAGE_RECORDS,」。
- 装置原本在 checker 档包里编，那个包依赖 `singlefs-format`：`crates/singlefs-checker-tier/Cargo.toml:11`：「singlefs-format = { path = "../singlefs-format" }」。上一条腿把装置搬进只依赖 core、checker、harness 的新包，漏了这一项。常量本身在 `crates/singlefs-format/src/lib.rs:291`：「pub const ROOT_RING_REGIONS: u64 = 3;」。
- 编译器给的提示是把这两处改成 `singlefs_core::…`。这里没照它改：那就改了装置源码，与「装置原样复用」不合；加依赖之后两处的代码还是装置原来那一份。

### 依赖与语法层面核了什么（量的）

1. 拼好的源码里引到的顶层路径：`grep -hoE '\b[a-z_][a-z0-9_]*::'` 在装置源码与 `attack_model_appended.rs` 上去重，带 crate 名的只有 `singlefs_checker`、`singlefs_core`、`singlefs_format`、`singlefs_harness`、`std`（另有 `super::`，在装置第 4813 行 `#[cfg(test)]` 的测试模块里，编二进制不进）。四个 singlefs crate 现在都在清单里。
2. 新包清单用 `python3` 的 `tomllib` 读得出，`dependencies` 四项：`singlefs-checker`、`singlefs-core`、`singlefs-format`、`singlefs-harness`。
3. 照 `rerun.sh` 的做法在草稿目录拼了一棵副本（`crates/`、`prune-server/`、`Cargo.toml`、`Cargo.lock`、`.cargo/`，接进 `crates/c699-attack-model` 这个成员），起 `cargo tree --offline -p c699-attack-model --depth 1`（不编），退 0，输出原样（草稿目录路径照抄）：

```
c699-attack-model v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus-b/tree/crates/c699-attack-model)
├── singlefs-checker v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus-b/tree/crates/singlefs-checker)
├── singlefs-core v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus-b/tree/crates/singlefs-core)
├── singlefs-format v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus-b/tree/crates/singlefs-format)
└── singlefs-harness v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus-b/tree/crates/singlefs-harness)
```

   这棵副本 48M，核完已删。
4. 拼好的 `main.rs`（`fn main() {` 恰好一行，是模型那一段的）用 `rustfmt --edition 2021 --check`（rustfmt 1.9.0-stable）解析：0 条 `error`，28 处 `Diff in`（只是排版不同），退 1 是因为有排版差异，不是解析失败。
5. 编出来不会被警告挡住：`.cargo/config.toml` 里只有 `[env]` 的 `BINDGEN_EXTRA_CLANG_ARGS`，没有 `-D warnings`；新包清单没有 `[lints] workspace = true`。装置的 `preflight()` 用 `env!("CARGO_MANIFEST_DIR")`，只由改名后的 `premise_apparatus_main` 调，模型那一段没调，最多出一条没用到的警告。

## 上一条腿的预言（原样抄，供主 agent 复跑后对）

下面几条从 `research/prompts/m3-close-b19-c699-r1-opus-output.md` 原样抄，没改一个字；都是推的、待主 agent 跑。格的定义、判读与「什么会推翻它」都在那份报告里，这里不重写。

- Q2.1（`skr-gap-fault`）：「预言的数（推的，跑前写下）：`skr-gap-fault` 一组里原样镜像上的洞对数等于 E198 skr 主点 26 对加 G3（S = 6）44 对 = 70 对、真值全是 `directly_bridged`；读坏那一张上同样 70 对，乙的 E2 = 70，今天的臂与甲的 E1、E2 都是 0。」
- Q2.1 乙计全零：「不加故障就丢掉全部 70 对（预言：原样那一半 E2 = 70，读坏那一半 E2 = 70，组合计 140）。」
- Q2.2（`sw-crash-wrapped`）：「实例 4 挂载之后第一张镜像上，洞对 (2@16, 3@20) 被今天的臂、甲、乙一致认作接得上，真值是中间有存在过的版本（3@17、3@18、3@19 的根槽都落过）。」「模型汇总行的 `all_three_candidates_e1` 数这一格，跑前预言它 ≥ 1。」
- Q2.3（`rs-two-files`）：「动作串 `second`、`empty+second`、`second+second`、`second+first` 四种在「写过 inode 2 之后、没再覆盖写第一个文件」的那几张上报 `reference_interval_verdict=violated`，每个 t 5 张，t = 4 … 9 合计 30 张」。
- 丁（攻方自提，被攻过零轮）：「sw-read-control、sw-crash-wrapped、nrb-recheck、rs1-recheck 四组 `accounting_generation_guarded_e1` 为 0，skr-gap-fault 的 `accounting_generation_guarded_e2` 为 0。」
- 锚（与 E198 两份产物同数才算模型接上了装置）：「`sw-read-control` 的 `today_e1` 与 `with_pointer_birth_e1` 各 315、`sw-crash-control` 的 `hole_pairs=0`、`nrb-recheck` 与 `rs1-recheck` 的 `today_e1` 6 与 285、`with_pointer_birth_e1` 0」。
- 全量末行：「末行 `OPUSMODEL name=done mode=full histories=261`」。

## 要主 agent 跑的命令

1. 模型整份复跑（编加跑；模型包不在 checker 档包里，不是重型；在新 shell 里起）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-c699-r1-opus-b-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-r1-opus-b/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。`--work` 那个目录现在不存在，`rerun.sh` 自己建；它已在本腿草稿目录的 `.deliverables` 里登成「交接」。`--frozen` 照上一条腿与任务 6816 用仓根（这一轮没有冻结副本）；任务 6816 打出的装置 sha256 `b57c3382…` 与此刻仓里那份一样（我现算过）。主 agent 若已有冻结副本，换成它。
   - 跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=model_source … sha256=816f91b7…`、`name=build exit=0`。不为 0 时日志里列着 `build.log` 里的 `error` 行（最多 40 行），整份在 `--work` 下的 `build.log`，新派承接腿接着修。再看 `OPUSMODEL name=estimate … within_the_limit=true`（`false` 时二进制退 3、打 `mode=stopped_over_the_estimate_limit`，不跑全量），七行 `OPUSMODEL name=group group=…`，末行 `OPUSMODEL name=done mode=full histories=261`。退非 0 的件数为 0 算过。
   - 跑完新派同立场的承接腿（带「承接：research/prompts/m3-close-b19-c699-r1-opus-b-output.md」与任务号），读 `task-server/task log <号> --all` 与 `--work` 下的 `model.out`，逐格对上一节抄的预言，把 `model.out` 拷进 `research/prompts/m3-close-b19-c699-r1-opus-b-model/out/model.out`。

## 没打中的形状

这一段只修模型，没攻新形状；上一条腿「没打中的形状」一节照旧有效。

## 这条腿自己的限度

- 没编、没跑：「补上依赖就编得过」是推的。路径解析出错之后被挡住的类型错，只有编了才看得见；冒出来就照上一节第 1 条新派承接腿。
- 依赖核只核了 crate 一层（引到的 crate 都在清单里、`cargo tree` 解析得通），没核每个被引的项在不在、是不是 `pub`、签名对不对。
- `--frozen` 用的是仓根，工作区还在变：复跑那一刻装置、`walk.rs` 等若与上一条腿开工时不同，`rerun.sh` 第一屏的 `name=frozen_source` 会打出来，要与任务 6816 打的六行对一遍。
- 上一条腿的限度（崩溃状态没有逐个展开、用户动作只放开了一部分、丁被攻过零轮）照旧，这一段没动。

## 没做什么

- 没往统一任务服务交任务，没编、没跑，判决要引的数一个都没复跑（命令在「要主 agent 跑的命令」一节）。
- 没改模型源码与 `rerun.sh`，没改上一条腿的模型目录与报告；不判正推、辩方那几格，不替主 agent 采纳。
- 草稿目录里建过两样：依赖核用的副本 `tree/`（48M，核完已删）、语法核用的 `syntax/`（312K，只有拼好的 `main.rs` 与 rustfmt 输出，不是仓副本，交回前删）。留着 `tree.txt`（`cargo tree` 输出）、`.deliverables`。
