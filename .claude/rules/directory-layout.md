# 仓里的目录各放什么

## scripts/：反复要用的脚本

- 工程反复要跑的脚本、它们读的输入表、自证与样本，一律放 `scripts/` 下按用途分的子目录，从那里跑；分哪几类、各放什么写在 `scripts/README.md`。
- `scripts/` 下最多三层，最深是 `scripts/<子目录>/<装置目录>/<文件>`，再深的拆成平级的装置目录。
- `scripts/` 顶层只放 `README.md` 与子目录；每个子目录有一份 `README.md`，写它放什么、从哪跑。新加一个脚本，放进用途对得上的子目录，在那份 `README.md` 的表里加一行。
- 不放结果：产物、日志、实测表不进 `scripts/`。实验产物写进 `research/results/`；运行记录（峰值表这类）放多机配置 `NFS_SHARED_ROOTS` 点名的共享根下（`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」）。
- 两台主机跑同一份脚本：资源分发、编译、跑测试都是同一份，差异只走参数与本地配置 `multi-host.env`，不为第二台另写一份。

## research/：可以整个删掉的研究目录

- 只有 `prompts/`（三方论证与冻结证据）与 `results/`（实验产物）两个子目录，加一份 `README.md`。
- 里面每一个文件都可以删。要用某份旧证据或旧产物，从 git 历史里取，或照复跑登记重跑；它不作为权威，结论另行验证。
- 反复要用的脚本、装置与输入不放 `research/`，放 `scripts/`。

## crates/：各包放什么

- `crates/` 下每个包在 `.claude/gate.d/crate-roles.tsv` 登记一行角色，四选一：产品（随产品发的代码）、验证（验证包、池级 checker、RefFS）、测试支撑（只给测试与装置用的库）、实验装置（实验包）。新加一个包，同一次改动登记它那一行。
- 一个包的集成测试放它自己的 `tests/`，单元测试放 `src/` 里的 `#[cfg(test)]`；不为测试另开包。验证包的跨包验证与崩溃枚举用例住验证包的 `tests/`（崩溃枚举用例写在别的包里，`scripts/crash/crash-case-check.py` 判红）。只给一个包的测试用的支撑代码写成那个包的 `tests/common_<名>/mod.rs`；至少两个包都无条件依赖的才单开测试支撑包。
- 实验装置二进制只放实验装置包；没有包依赖实验装置包。
- 验证包只装测试、测试的脚手架，与门禁、崩溃入口自己起的工具二进制。
- 产品包不放实验装置与测试二进制；产品自己的可执行入口不禁，逐个登进放行表 `.claude/gate.d/crate-roles-allowed.tsv`。
- 不跨包用 `#[path = …]`、`include!`、`include_str!`、`include_bytes!` 引另一个包目录下的源码：后缀是 `.toml`、`.md`、`.tsv` 的算数据文件，读它不在此列；别的后缀一律照源码算（`.rs`、`.wgsl`，也包括 `.inc` 这类中转文件）。要用另一个包的代码，在 `Cargo.toml` 里依赖它，或在自己的包里写一份；确要读别的后缀的数据文件，登进放行表 `.claude/gate.d/crate-cross-references-allowed.tsv` 写明理由。
- 弹窗问放置类问题（东西放哪个包、哪个目录，要不要新开包）：问句点名 `.claude/rules/directory-layout.md`「crates/：各包放什么」；每个选项的说明以「标准：」或「偏离：」起头，「偏离：」后面用「」照抄它偏离的「crates/：各包放什么」那一条的原话，去掉空白至少 4 个字、是那一节正文里的一段；至少一个选项以「标准：」起头。
- 放置类问题怎么认（钩子 `.claude/hooks/ask-user-placement-guard.sh` 的词表与下面每组词逐组相同、次序相同）：问句、选项标题、选项说明各算一段，一段里有下面三样之一就是放置类。问处词「哪个包」「哪一个包」「哪个目录」「哪个子目录」「哪个 crate」紧跟在放置词或单字「放」「归」后面（中间可隔空白）；放置词「放在」「放进」「放到」「搬到」「搬进」「移到」「挪到」「挪进」「住进」「并进」后面、同一小句里有落点词「包」「crate」之一，小句断在「，」「。」「？」「！」「；」「：」「、」「,」「?」「!」「;」与换行上；开包词「新开」「单开」「另开」后面同一小句里隔 2 个字以内有「包」。「包」按整词认，不算整词的有「包括」「包含」「包装」「包裹」「包办」「包容」「打包」；用「」括着的原文（轮名、引的原话）不算。问处词前面不是放置词或单字的（日志在哪个目录里找、每个文件落在哪个包）、放置词后面同一小句里没有落点词的（复跑的输出放在草稿目录）不算放置类。

## 测试的定义与输入跟着测试放

- 不变量清单、登记表、钉表这类只被某一组测试读的定义与输入，放在那组测试所在的 crate 目录里，不放 `.claude/gate.d/`；门禁只读它们，不持有它们。
- 只被门禁自己读的配置（格名、阶段归属、排除表）留在门禁旁边。

## 门禁管哪一半

门禁 code-tooling 的 directory-layout 格（判法 `scripts/gate/directory-layout-check.py`）判：`research/` 顶层只有两个子目录与说明文件；`scripts/` 顶层只有说明文件与子目录、每个子目录有说明文件、层级不超过三层、没有产物与日志类的文件（`.out`、`.log`）。

「crates/：各包放什么」能按字面判的那一半：

| 判什么 | 谁判 |
|---|---|
| `crates/` 下任意深度的每个 `Cargo.toml` 的包在登记表里恰有一行、登记的目录在、角色认得、判到的包数不为 0 且与 `Cargo.toml` 数相同；实验装置包之外的每个二进制目标（照 cargo 的认法：`<包>/src/main.rs`、`src/bin/`、`examples/`、`[[bin]]` 与 `[[example]]` 的 `path` 指到哪都算，不看名字）都登在放行表或滞后表里；库为空（去掉注释、属性、`use` 再导出与只装这些的 `mod` 之后没有条目）而有测试目标的包只许是测试支撑；测试支撑包至少两个无条件的依赖者（`target.<条件>` 下的与 `optional = true` 的不算）；产品包不经 `[dependencies]`、`[build-dependencies]` 依赖测试支撑包（`[dev-dependencies]` 照放）；没有包依赖实验装置包 | 门禁 code-tooling 的 crate-roles 格（判法 `scripts/gate/crate-roles-check.py`）；存量逐个目标登在只缩不涨的 `.claude/gate.d/crate-roles-lag.tsv`（整包行不收），门禁与崩溃入口自己起的工具、产品自己的可执行入口登在放行表 `.claude/gate.d/crate-roles-allowed.tsv` |
| `crates/` 下各包的编译单元读的文件（`cargo check --all-targets --all-features` 写的 dep-info，`#[path]`、`include!` 一族怎么写都在里面）落在另一个包目录下、后缀不是 `.toml`、`.md`、`.tsv`，或是不属任何包的 `.rs`、`.wgsl` | 门禁 checker-independence-and-sync 的 crate-cross-references 格（判法 `scripts/gate/crate-cross-references-check.py`）；存量登在只缩不涨的 `.claude/gate.d/crate-cross-references-lag.tsv`，确要编进别的包的源码的登在放行表 `.claude/gate.d/crate-cross-references-allowed.tsv` |
| 放置类弹窗（照「放置类问题怎么认」那几组词认）的问句点名 `.claude/rules/directory-layout.md`「crates/：各包放什么」、每个选项以「标准：」或「偏离：」起头、至少一个「标准：」、「偏离：」引那一节的原话；钩子的词表与那几组词逐组相同，原话字数下限与开包隔字上限与钩子的常量相同 | 钩子 `.claude/hooks/ask-user-placement-guard.sh`（词表与两个数相同由它的自证核） |

靠 review 的：脚本放进的类对不对、说明文件写得准不准、有没有为第二台另写的脚本、测试的定义与输入放没放在测试旁边；包登的角色对不对、放行表登的二进制是不是真是门禁工具或产品自己的入口、运行时读的文件与过程宏自己读的文件跨没跨包、特性组合之外被 cfg 掉的引用（不编进来，dep-info 里没有）；弹窗里「标准：」「偏离：」写得对不对、放置类问题换了词表外的说法没被认出来；搬的范围超没超出用户定案（搬了定案没点名的东西、包名起得太泛），没有机器判得了的形态；代码轮正文的标题先写一个更早的日期（例如它攻的那份判决的日期），门禁 doc-process-records 的 code-round-checklist 格按标题里第一个日期判，就只判八格或整份不查，标题日期写对没有靠人核。
