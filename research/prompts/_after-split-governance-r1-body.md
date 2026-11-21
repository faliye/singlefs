# 拆后收口那一批治理改动（治理文件轮，第一轮）

方案按 `crates/` 今天的实现来谈：这一批没改 `crates/` 下的代码，判的都是门禁、钩子、判法脚本、登记表与定义规则。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md` 第 227 行起「拆后收口」那一段。开工快照 `research/prompts/after-split-governance-r1-snapshot/judged-sha256.txt` 点名的 34 份就是被判的文件。

## 问题

这一批换上了四组改动。判它们判得对不对、拦不拦得住它们要拦的那几次事、有没有误伤、绕过与重复，规则正文写得对不对。

## 改了什么（主 agent 的观测，工具员报告与换上结局）

1. 放置类治理轮第一轮判决的改法（`research/prompts/gpu-placement-measures-governance-r1-main-verification.md`「二、判决」第 1、2、4、5 条），照拆后的主工作区重出后换上：放置类弹窗钩子 `.claude/hooks/ask-user-placement-guard.sh` 改用 `.claude/rules/directory-layout.md`「放置类问题怎么认」那四组词表（问处词、放置词加落点词、「包」按整词认、「」括着的原文不算）；`scripts/gate/crate-roles-check.py` 照 cargo 的认法逐个二进制目标登记、加「产品包经 `[dependencies]`、`[build-dependencies]` 依赖测试支撑包判红」一格；三张 `crate-roles*.tsv` 照拆后的树现算；`.claude/gate.d/doc-process-records.sh` 的 code-round-checklist 格截止日改 2026-11-19、只判八格的名单搬进 `.claude/gate.d/code-round-checklist-eight-cells.tsv` 并核文件在不在；检出钩子 ⑭ 与写闸第十一道往已有门禁的样本档目录写都拒，认法抽进 `.claude/hooks/lib_write_scope.py`；`code-tooling.sh` crate-roles 格说明、`.claude/agent-common.md` ⑭、`.claude/rules/verification-reference.md` 两行、`.claude/agents/tooling-writer.md` 第 3 步那一句跟上。当时被旧词表误拦的「整包跑哪几个包」弹窗原文进了喂法、新版放行。
2. 打补丁那一关整行替换与删除先核前像：`mutations-replacements.tsv` 每行十二段（新行加写补丁时那一行），`mutations-delete.txt` 每行是前像整行；打之前对被打那棵树此刻的 `crates/mutations.tsv` 逐行整行比，一行对不上整份拒、一个字不改；段数不对的整份拒。生成方 `scripts/gate/merge-and-verify.sh` 的合补丁那一支与 `scripts/gate/patch-of-the-current-tree.py` 一起改；`.claude/agents/implementation-writer.md` 补丁目录那一段跟上。
3. 写入一侧判任务号、会话短号、批号的相抵单位，从整行原文换成违例本身（判据行名字加命中的那段字面、整份计数）：`.claude/gate.d/lib-forbidden-notations.py` 的 `new_violation_hits`，`scripts/kb/lib_notation_check.py` 加五格共用自证，写闸自证一格从「改了旧号那一行照判」改成放行、另加「旧号那一行新加第二个号照判」；`.claude/agent-common.md` 两处说法跟上。
4. 门禁 checker-independence-and-sync 的跨包源码引用（原 ⑨）拆成新格 crate-cross-references，判法 `scripts/gate/crate-cross-references-check.py` 读 `cargo check --all-targets --all-features` 写的 dep-info：别的包目录下的源码（后缀不是 `.toml`、`.md`、`.tsv` 的都照源码算）与不属任何包的 `.rs`、`.wgsl` 判红，构建脚本生成与依赖源码放行；一个包一条判定进 prune-server，按单元键复用。换上之后修了四处：rustc 给每份读到的文件写的空规则被当产物剔光、读包清单没带 `--filter-platform`、构建脚本单元的 `.d` 名字照 cargo 真写的 `build_script_build-<哈希>.d` 找、四个包的构建脚本报 `rerun-if-env-changed=HOME` 让键随主机变（改成按 cargo 家目录里 config 的内容进键，现判那一行报键里变了哪几样）。两张表照真树重写（放行 5 行、滞后 0 行）；`scripts/prune-server/prune_server_client.py` 文件头「键空间」与 `.claude/hooks/write-guard-decisions.py` 一处文档串跟上。

## 已经量的（结局行 type= exit= 那一段）

- 第 1 组：五份换上 type=succeeded exit=0；放置类钩子、检出钩子、写闸自证 exit=0；写闸开关 gate-fixtures-in-place-allowed 证红红在点名的五格；放置类钩子、检出钩子、crate-roles-check 开关复跑 exit=0（算键库修复版换上之后重量也是 exit=0）；doc-process-records 样本档与开关复跑 exit=0；真仓 code-round-checklist 格 exit=0；真仓 crate-roles 格 exit=1，只红「support-dependents：crates/singlefs-background-gpu-pairing-keys（测试支撑）只有 1 个无条件的依赖者」，旧判法同样红。
- 第 2 组：换上 exit=0；apply-writer-patch、patch-of-the-current-tree、merge-and-verify 三份自证 exit=0；patch-of-the-current-tree 开关复跑 exit=0。apply-writer-patch、merge-and-verify 两份开关复跑没跑成（前者复跑那一格认不出拆过的入口读开关，后者算键整格拒算，都归别的会话在修）。
- 第 3 组：两份换上 exit=0；replace-once、replace-batch、insert-row、append-record、ledger-append-section、检出钩子、写闸自证与 doc-text 样本档 exit=0；写闸五条开关证红红的格数与点名逐格相同；replace-once、replace-batch 开关复跑 exit=0；原先被拒写的 13 处只改路径的行都写进了。
- 第 4 组：判法自证 17 格 exit=0；17 条开关证红红的格与点名逐格相同；门禁样本档 exit=0，8 条样本开关判错的样本与点名逐份相同；真仓单格三趟 exit=0，第一趟每包「库里那一条认不得，照头一次现判」，第二、三趟「单元键：现判 0、复用 12、强制现判 1、拒算 0」。三趟都派在第二台同一个编译目录，换台复用没量到。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；打中的写反例输入。

1. **拦不拦得住它要拦的事**：第 1 组拿「整包跑哪几个包」那份原文与一份真的问「E202 放哪个包」的弹窗各喂一次；第 2 组拿「补丁写好之后、打之前，别的会话改了同名那一行」喂；第 3 组拿「行里原有任务号、只改同一行路径」与「同一行新加一个号」各喂；第 4 组拿 `#[path]`、`include_str!` 九种写法、构建脚本单元读别的包、编不过的写法各喂。每一条现在由哪一行判、判成什么；判错的就是打中。
2. **误伤与绕过**：放置类词表会不会拦下与放置无关的弹窗、放过换了说法的放置类问题；前像比对在合补丁那一支与 `patch-of-the-current-tree.py` 两个生成方之外，还有没有生成六段旧格式的地方（会被整份拒）；相抵按「判据加命中字面」计数，能不能把一个旧号挪到另一行、或删一个旧号同时加一个新号而不被拒；dep-info 判法「后缀不是 `.toml`、`.md`、`.tsv` 的照源码算」会不会误伤正当读别的包数据文件的写法，放行表 5 行的理由站不站得住。
3. **复用键与多机**：crate-cross-references 的单元键里收了相对路径（这一格判的就是文件落在哪个包），主 agent 判为判定自己的内容输入；对照 `.claude/kb/decisions/13-验证路线.md` D13（验证路线） 已定项 17 与 `.claude/rules/verification.md`「一个门禁：两种跑法」里「路径、名字、行号、资源上限、限时、提交与树不进键」那一句，这一判站不站得住。HOME、CARGO_HOME 改按 cargo 配置文件内容进键之后，键里还有没有别的随主机、编译目录变的成分；换台复用没量到这一点，自证 key-independent-of-compile-directory 那一格够不够当证据。
4. **不兼容旧格式与旧开关**：前像比对把旧的六段替换整份拒，判法没有为旧格式开读法；crate-cross-references 判定记录里没有 `components` 的一律当认不得、照头一次现判。逐处看这一批有没有为旧东西留的分支、开关或兼容读法（`.claude/agent-common.md`「写」一节「里程碑 17 收尾之前不兼容任何旧东西」）。
5. **规则与定义正文**：`.claude/rules/directory-layout.md`、`.claude/rules/verification-reference.md`、`.claude/agent-common.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/implementation-writer.md` 这一批改的句子，合不合 `.claude/singlefs-ai-sop/rules/rules-discipline.md`（只写怎么做、不写论证），与判法现行的行为对不对得上（`directory-layout.md` 的后缀说法与判法的 `DATA_SUFFIXES`、`SOURCE_SUFFIXES` 是不是一回事）。

## 分工

正推腿判第 1、4、5 格；攻方腿判第 2、3 格与「已经量的」那一节里每一句能不能从日志里核到。
