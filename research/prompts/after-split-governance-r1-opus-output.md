# 拆后收口治理轮第一轮 攻方腿报告（after-split-governance-r1）

日期：2026-11-20。攻击面：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱；落到正文「判据格」第 2、3 格与「已经量的」一节。
被判对象以开工快照 `research/prompts/after-split-governance-r1-snapshot/judged-sha256.txt` 为准；开工时 `sha256sum -c` 这 34 份全部 OK（这一次现跑，没有一行不是 OK）。

## 复跑命令与模型文件

模型目录 `research/prompts/after-split-governance-r1-opus-model/`，三份探针都没跑过（子 agent 不交任务），读数一律「推的，待主 agent 跑」。
复跑（主 agent 交，见末尾「要主 agent 跑的命令」）：`tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结树> <运行目录> <模型目录>`，它删掉那一个 `-run` 收尾的运行目录再起 `tools/rerun.sh`。
这一轮没有主 agent 建的冻结副本，`--frozen` 填仓根；`rerun.sh` 开跑先拿开工快照核探针读的四份被判文件的 sha256，对不上就停。

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | cb46126b8e3847ff5b8723ef5f5b83b1f2d1be01876ce154a5d9e32025240585 |
| tools/rerun-from-a-clean-work-directory.sh | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |
| tools/measured_versus_judged_versions.py | e70cfca8dda1e6662adac479e0230291b9b576458dd2b2a3c08a80cfa60431c3 |
| probes/netting_probe.py | 6c6b3d33f0f9a8a1f766521a203501e78e9f9a66f2965a12387b7516f4854527 |
| probes/placement_probe.py | 1905ede38bac237c50625dce5db21c5edda28b31d09a86815b3d675ddaeaf157 |
| probes/key_and_classify_probe.py | 6c6b50256b5ee65a2a9f20c07e806bc96e2b2273a7430fda62779670e825af9a |

- `netting_probe.py`：拿被判的 `new_violation_hits` 判六种挪号、换号、换描述的写法，每种打 `name=netting_<写法> new_hits=…`。
- `placement_probe.py`：拿被判的放置类钩子判 12 问（选项都是中性的「是」「否」），每问打 `name=placement_<问名> exit=…`。
- `key_and_classify_probe.py`：建两棵四个包的小工作区，用真 cargo 跑五趟 `cargo check`（无环境、`RUSTFLAGS`、工作区上一层的 `.cargo/config.toml`、家目录里的 `.cargo/config.toml`、关调试断言），只调判法里的纯函数（`recorded_package_facts`、`package_key`、`Layout`），不连判定库；每包每趟打 `name=key_<趟>_<包> reuse=… facts_same=… verdict=<sound-reuse|false-reuse|rejudge>`，再打 `name=classify_delta …`。
- `measured_versus_judged_versions.py`：只读任务服务尝试目录的 `spec.json`、`result.json` 与被判文件的改动时间，逐趟标那一趟量的是不是被判的那一版；它打出的行带任务号，只给主 agent 现查，不抄进仓。
- 估时（推的）：五趟 cargo check 每趟几个无依赖的小包，加 12 次喂钩子，合计几分钟；内存是 cargo 编小包的量。没有崩溃枚举，3b 那一套不适用。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据在 |
|---|---|---|---|
| 第 2 格 放置类词表 | 不成立（误拦、漏拦都造得出） | 时间意义的「放在」加「包」被拦；「放到」「新开一个包」「单开包」「哪一个包」都放过 | 第二节 |
| 第 2 格 前像比对的其他生成方 | 成立（仓里没找到别的六段生成方） | 另有一条不过前像的写表路：拆分重归的整表 `--out` 换进 | 第二节 |
| 第 2 格 相抵按判据加字面计数 | 部分成立 | 删旧号加不同号照拒；同一字面挪行、拿描述指会话的那一类整句换成别的会话都不拒 | 第二节 |
| 第 2 格 dep-info 后缀认法 | 不成立 | `.md`、`.toml`、`.tsv` 当模块编进来、不属任何包的 `.inc`、构建脚本拷进 OUT_DIR 三条路都放过；放行表第 7 行的理由与代码不符 | 第二节 |
| 第 3 格 单元键收相对路径 | 不成立（按字面）；技术上必须收 | D13 已定项 17 把路径列进「一律不进键」、例外只有一个；去掉路径键就过窄，两句在这一格互相顶着 | 第三节 |
| 第 3 格 HOME 改按配置内容之后键里还变什么 | 不成立 | 随主机、编译目录变的成分没再找到；但生效构建环境没进键：13 个包里只有报了 HOME 的 4 个收家目录配置，RUSTFLAGS、工作区上一层的 cargo 配置、`CARGO_PROFILE_*` 一个都不收，换台、换配置会照旧复用错的事实 | 第三节 |
| 第 3 格 自证够不够当换台复用的证据 | 不够 | 假 cargo 在两个根下写的是同一份录好的 .d，事实相同是造出来的；真仓三趟都在第二台同一个编译目录 | 第三节 |
| 「已经量的」 | 多数核到；7 句量的不是被判的那一版或核不到 | 写闸、检出钩子、merge-and-verify、code-tooling 四份在那些量之后又被这一批之外的换上改过；门禁样本档与 8 条样本开关量在判法最后一次换上之前；「旧判法同样红」没量过 | 第四节 |

最脆弱的一条：第 3 格「生效构建环境不进键」。它让 crate-cross-references 的复用在第二台或改了 cargo 配置时交出与真编不符的绿判定，而判法文件头自己写着配置能改 rustflags（`scripts/gate/crate-cross-references-check.py:65`），代码只对报了 HOME 的包照做。

## 二、第 2 格：误伤与绕过

### 2.1 放置类词表

现行词表：「WHERE_WORDS = ("哪个包", "哪个目录", "哪个 crate", "新开包")」（`.claude/hooks/ask-user-placement-guard.sh:89`），「PLACEMENT_WORDS = ("放哪", "放在", "放进", "搬到", "搬进", "住进", "归位")」（`.claude/hooks/ask-user-placement-guard.sh:90`）。认法按字面子串，「包」按整词（排除 7 个复合词）。

反例输入（每问的选项写「是」「否」，不带四组词；读数推的，待主 agent 跑 `placement_probe.py`）：

| 问名 | 问句 | 按意思是不是放置类 | 推的结局 | 错在哪 |
|---|---|---|---|---|
| scheduling-put-at-with-package | 合入后验证放在提交前做，还是现在就对验证包跑一趟？ | 不是（问先后） | 拦 | 「放在」当时间用，加「包」就拦 |
| result-file-into-scratch-directory | 复跑的输出放在草稿目录还是 research/results/ 下？ | 不是 crates/ 的放置 | 拦 | 要求点名「crates/：各包放什么」、偏离选项引那一节原话，这一问引不出 |
| batch-title-unquoted | 产品GPU包测试与E202装置归位那一批先关哪几件？ | 不是 | 拦 | 调度记录标题没加「」就带「归位」与「包」 |
| where-is-the-log-directory | 这一趟的日志在哪个目录里找？ | 不是 | 拦 | 问处词「哪个目录」单独成立 |
| put-to-package | E202 的装置放到实验包还是验证包？ | 是 | 放 | 「放到」不在放置词里 |
| open-one-new-package | 要不要为 E202 新开一个包？ | 是 | 放 | 「新开包」中间夹「一个」就不认 |
| separate-package-or-merge | 配对键那个包该单开包，还是并进验证包？ | 是 | 放 | 「单开包」「并进」都不在表里（crate-roles 判法自己的红句就写「至少 2 个才单开包」） |
| which-one-package | E202 归哪一个包？ | 是 | 放 | 「哪一个包」不是「哪个包」 |
| move-into-crate-path | E202 的装置移到 crates/singlefs-implementation-experiments 行吗？ | 是 | 放 | 落点词 crate 认得出（复数照认），「移到」不在放置词里 |
| which-subdirectory | 这批装置挪进哪个子目录？ | 是 | 放 | 「哪个子目录」不是「哪个目录」 |
| control-put-where-package | E202 放哪个包？ | 是 | 拦 | 对照 |
| control-whole-package-run | 整包跑哪几个包？ | 不是 | 放 | 对照（必放样本那一问） |

判定：不成立，误拦与漏拦都造得出。已经写明的限度只盖住一部分：钩子文件头写「管不到的：换了说法的放置类问题」（`.claude/hooks/ask-user-placement-guard.sh:69`，后面举的三个例子是挪到、落在哪里、搁哪），规则写「放置类问题换了词表外的说法没被认出来」（`.claude/rules/directory-layout.md:45`）归 review。没写明的两类：

- 误拦：问句里有时间意义的「放在」（「放在提交前做」）、标题里的「归位」，或者问日志在哪个目录，这些都拦，而且出路不对：拦下之后要点名 crates/ 那一节、给「偏离：」选项引那一节的原话，非 crates/ 的问题引不出合格的原话，只能硬套。
- 漏拦：「放到」「新开一个包」「单开包」「哪一个包」都是和词表只差一两个字的常见说法，不是「换了说法」。

四句：分辨臂——不涉及臂（这一格只判钩子拦得准不准）；钩子当时看不看得到——看得到，判别它的就是问句里的字；满足的判据分句——第 2 格「会不会拦下与放置无关的弹窗、放过换了说法的放置类问题」两句都中；改法在打中的几格上还中不中——见第五节的表，加词只修漏拦，误拦要另改认法。

### 2.2 前像比对：两个生成方之外

查的范围：`grep -rn mutations-replacements` 扫 `scripts`、`.claude/hooks`、`.claude/scripts`、`.claude/skills`、`.claude/agents`、`.claude/rules`、`.claude/main-agent.md`、`task-server`（不含 `fixtures/`）。写这张表的只有 `scripts/gate/merge-and-verify.sh` 的 MERGE_PATCH_PROGRAM、`scripts/gate/patch-of-the-current-tree.py` 与 `scripts/gate/merge-and-verify-selftest-lib.sh` 的样本，三处都写十二段；读它的 `scripts/gate/merge-and-verify-items-lib.sh` 用 awk 取第一段、要求 NF 不少于 6，十二段的行照读。实现员手写的那一份照 `.claude/agents/implementation-writer.md:71` 写十二段。在这个范围里没找到还写六段替换的地方：成立。

另一条写变异表、不过前像的路（打中旁边一格，推的）：拆分时重归变异行的工具把改好的整份表写到 `--out`，出路写「其余行用 --out 那份整表换进 crates/mutations.tsv」（`scripts/gate/lib_crates_split_tables.py:226`）。整表一换，补丁写好到换进之间别的会话改过的行会被一并盖掉，前像比对拦不到这条路。它不在这一批的 34 份里，算越出这一批的线索，交主 agent 判要不要另开。

### 2.3 相抵按「判据加命中字面」计数

现行：「相抵的单位是违例本身：(判据行名字, 命中的串) 按整份文件计数，写后某一种比写前多出几处就交几处，在哪一行不论」（`.claude/gate.d/lib-forbidden-notations.py:1177`）；共用约束的说法是「按判据与命中的那段字面整份计数，写后比写前多出来的违例就拒」（`.claude/agent-common.md:45`）。

| 写法 | 推的新违例数 | 判 |
|---|---|---|
| 同一行只改路径，号不动 | 0 | 放，符合本意 |
| 把一个旧号从一句挪到另一句（同一字面） | 0 | 放；字面照计数没多，但它现在挂在另一件事上 |
| 删一个旧号、同一处写一个不同的号 | 1 | 拒，符合本意 |
| 同一个号多写一处 | 1 | 拒，符合本意 |
| 判据行 session-descriptions 认的那句描述，整句改写成指另一个会话（命中的字面没变） | 0 | 放；判据要拦的就是拿描述指别的会话，新写的这一句指的是另一个会话 |
| 同上，再挪到另一行 | 0 | 放 |

判定：部分成立。「删一个旧号同时加一个新号」照拒；「把旧号挪到另一行」与「换描述」不拒。后两种合规则字面（整份计数没多），不合规则的本意：任务号的字面里带着号，挪行只是换了挂靠；会话描述的命中字面里没有会话身份，同一字面可以整句换成指别的会话，写闸认不出这是一次新写。改成按行文相抵的旧判法能拦住换描述那一种，却会把只改路径的那 13 处照拒，这一批就是为了放过它们才改的；我没找到两样都满足的认法，只能把会话描述那一类写进写闸与 `.claude/agent-common.md:45` 的「判不到」。

四句：不涉及臂；写闸看得到写前、写后两份全文，但看不到一句话指的是哪个会话，这一样判别子观测不到；满足的是第 2 格「能不能把一个旧号挪到另一行」分句；改法见第五节。

### 2.4 dep-info 判法的后缀认法与放行表

现行分类：别的包目录下「后缀是 .toml、.md、.tsv 的算数据文件，放行；别的一律算跨包源码，判红」（`scripts/gate/crate-cross-references-check.py:36`）；不属任何包的「后缀 .rs、.wgsl 的算源码，判红；别的算数据，放行」（`scripts/gate/crate-cross-references-check.py:37`）；OUT_DIR 里的一律放行（`scripts/gate/crate-cross-references-check.py:33`）。判红只看编译单元的 `sources`（「for path in unit["sources"]:」，`scripts/gate/crate-cross-references-check.py:509`），构建脚本报的 rerun-if-changed 只进键、不分类。

漏拦（三条路，反例都在 `key_and_classify_probe.py` 的 delta 包里，真 cargo 编得过；推的，待主 agent 跑，看 `name=classify_delta` 那几行）：

| 写法（delta 包的 `src/lib.rs`） | dep-info 里那一份 | 推的分类 | 判 |
|---|---|---|---|
| `#[path = "../../beta/src/code.md"] mod from_markdown;`，code.md 里是 Rust 源码 | crates/beta/src/code.md | data（别的包的数据文件） | 放：别的包的源码改个后缀就编进来了 |
| `include!("../../../shared/snippet.inc");`，snippet.inc 在仓里、不属任何包 | shared/snippet.inc | unpackaged-data | 放：不属任何包的源码只要不叫 .rs、.wgsl 就放 |
| 构建脚本 `std::fs::copy("../beta/src/x.rs", OUT_DIR/copied.rs)`，库里 `include!(concat!(env!("OUT_DIR"), "/copied.rs"))` | OUT_DIR:delta/copied.rs | generated | 放：别的包的 .rs 经 OUT_DIR 中转；构建脚本报了 rerun-if-changed=../beta/src/x.rs 也不分类 |

判法文件头写着「写法不论」（`scripts/gate/crate-cross-references-check.py:6`），后面列的中转只有「经别的后缀中转」；上面三条都是写法，判法放行。成功句末尾的「这一格判不了的：特性组合之外被 cfg 掉的文件（没编进来）、运行时读的文件、过程宏自己读的文件、构建脚本跑时读而没报 rerun-if-changed 的文件」（`scripts/gate/crate-cross-references-check.py:962`）只盖住构建脚本没报 rerun-if-changed 的那一种，报了的（上面第三行）照样放。

误拦：别的包的 `.json`、`.bin`、`.txt` 这类数据，用 `include_str!` / `include_bytes!` 读，判红（delta 包读 crates/beta/data/sample.json，推的分类是 cross）。按现行规则这算有意收严：「后缀是 `.toml`、`.md`、`.tsv` 的算数据文件，读它不在此列；别的后缀一律照源码算」（`.claude/rules/directory-layout.md:24`），出路是登放行表。但几处正文与它互相矛盾：
- 判法文件头第 5 行仍写判据「读别的包的数据文件不在此列」（`scripts/gate/crate-cross-references-check.py:5`），没有后缀限定，与第 36 行的分类不一致；
- 放行表表头写「确要把另一个包目录下的源码编进来的跨包引用」（`.claude/gate.d/crate-cross-references-allowed.tsv:1`），而规则让读 `.json` 数据的也登这张表；登进去之后，成功句把它数进「跨包源码与不属任何包的源码 N 处」。

判定：不成立。漏拦三条路有具体输入；误拦是规则选的取舍，但规则、判法文件头与表头三处的说法互相顶着。

放行表 5 行的理由：

| 行 | 理由站不站得住 | 依据 |
|---|---|---|
| 第 3、4 行 unit-graph 编进 core、background-gpu 的 build.rs | 站得住：被测的就是那两份构建脚本的特性表 | `crates/singlefs-unit-graph/src/build_features.rs:267` 与 `:278` 的 `#[path]` |
| 第 5、6 行 unit-graph 读验证包、配对键包 build.rs 的原文 | 站得住：是当文本读，核报 rerun-if-env-changed 的那个循环 | `crates/singlefs-unit-graph/src/build_toolchain.rs:1121` 与 `:1125` 的 `include_str!` |
| 第 7 行 verification 编进 core 的 build.rs | 理由与代码不符 | 见下 |

第 7 行原文「核心层是门禁 checker-implementation-disjoint 格 ⑧ 允许共用的一边，编进来的是只含特性表常量的构建脚本，被判的正是这张表」（`.claude/gate.d/crate-cross-references-allowed.tsv:7`）。两处不对：
- core 的 build.rs 不只含特性表常量：它有 `fn main`、`enabled_features_metadata_value`、`features_opened_by_the_rustflags_but_off_in_this_build_script`（`crates/singlefs-core/build.rs:49`、`:63`、`:106`），验证包的测试调的正是后一个函数：「singlefs_core_build_script::features_opened_by_the_rustflags_but_off_in_this_build_script(」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_lookup_by_identity/core_build_script_features.rs:42`）。被判的不止那张表。
- ⑧ 允许共用的是依赖闭包里的 core，`#[path]` 那条它明说归这一格：「`include_str!` 跨包目录引源码或着色器文件归 crate-cross-references 格判。」（`.claude/gate.d/checker-independence-and-sync.sh:55`）。拿 ⑧ 的允许共用当登这一行的理由，是把归这一格判的事推回给 ⑧。

四句：不涉及臂；判法看得到 dep-info 列的路径与后缀，看不到这一份是被当模块编进来还是当文本读进来（rustc 的 .d 不分），所以 `.md` 当模块那一条在 dep-info 里判别子观测不到，OUT_DIR 中转那一条在构建脚本的 rerun-if-changed 里看得到而没用；满足的判据分句是第 2 格「会不会误伤正当读别的包数据文件的写法，放行表 5 行的理由站不站得住」；改法见第五节。

## 三、第 3 格：复用键与多机

### 3.1 单元键收相对路径，对照 D13 已定项 17

键里带路径的成分：`package-file <路径> <sha256>`、`read <路径> <sha256>`、`build-input <路径> <sha256>`、`manifest <路径> <sha256>`，`build-input-named` 里还有包名（`scripts/gate/crate-cross-references-check.py:58`）；上一条判定按 `items/<包目录的 sha256>.json` 取。判法给的理由：「路径都相对 <根>：搬一份文件改变 rustc 找模块的结果，也改变它属于哪个包」（`scripts/gate/crate-cross-references-check.py:66`）。

条款原文：
- 「一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时」（`.claude/kb/decisions/13-验证路线.md:436`）
- 同一条接着写「只有一个例外：观察者结论的键里用判定项标识（观察者所在的文件加名字）分开同一块上的几个观察者，用户定」（`.claude/kb/decisions/13-验证路线.md:436`）
- 同一条又写「键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反」（`.claude/kb/decisions/13-验证路线.md:436`）
- 「内容输入」的原文：「加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）」（`.claude/kb/decisions/13-验证路线.md:435`）
- 规则：「路径、名字、行号、资源上限、限时、提交与树不进键（D13（验证路线） 已定项 17）」（`.claude/rules/verification.md:41`）

判定：主 agent 那一判按字面不成立。条款把路径明列进「一律不进键」，例外只开了观察者的判定项标识一处，并写明只有这一个；「内容输入」举的例子全是内容，没有一个是位置。把路径读成内容输入，是在条款之外开了第二个例外，治理文件轮没有资格开：D13 是决策，开例外要走设计推论轮、由用户定（观察者那一个例外就是用户定的）。

另一方面，这一格的判定确实离不开路径，去掉路径键就会过窄（同一条里的另一种违反）。反例（推的，没跑）：包 alpha 的 `src/m.rs` 里写 `#[path = "../../beta/src/x.rs"] mod q;`，包里另有一份内容相同的 `crates/alpha/beta/src/x.rs`；把 `src/m.rs` 原样改名成 `src/m/mod.rs`（`mod m;` 两种都认，内容一个字不改），`#[path]` 改成相对 `src/m/` 解，读的从 `crates/beta/src/x.rs`（跨包）换成 `crates/alpha/beta/src/x.rs`（同包）。不带路径的键里，包内文件的内容多重集合与读过的外部文件内容都没变，键相同、照旧复用「跨包」那条事实；带路径的现行键里 `package-file` 那几行变了，会现判。所以 D13 已定项 17 落到这一格上自相矛盾：不收路径是过窄，收路径是违反「一律不进键」。出路只能是用户在 D13 里为「判的就是文件落在哪」的判定开第二个例外（写明射程：只有判据本身取决于位置的判定），或者让这一格不按键复用、每趟现判（又与 `.claude/rules/verification.md` 同一节「凡起编译、跑自证、现算键的判定都按判定入库、按单元键复用」顶着）。两条都要用户定。

同一处的连带：包目录改名后 `items/` 取不到上一条，照头一次现判，这正是条款点名的「改名丢库」；按上面的理由这里也躲不开，一并写进那个例外。

四句：不涉及臂；判法看得到路径，问题不在看不看得到，在判据（D13 已定项 17）自己的两句在这一格上不能同时满足，属于「判据自己写错」那一类，要改的是判据；满足的分句是「路径（仓根也算）…一律不进键」与「只有一个例外」两句；改法见第五节。

### 3.2 HOME、CARGO_HOME 改按配置内容进键之后

逐个成分核随主机、编译目录变不变（按代码读，推的）：

| 成分 | 随编译目录变 | 随主机变 | 依据 |
|---|---|---|---|
| judge、command | 不变 | 不变 | 判法闭包键与常量参数 |
| toolchain | 不变 | 只在两台工具链不同时变（日志里第二台那一步打「两台的工具链相同」） | `scripts/gate/crate-cross-references-check.py:51` |
| manifest、package-file、read、build-input | 不变（相对路径加内容） | 不变（仓在共享根上） | 文件头第 53 到 57 行 |
| build-input-named | 不变（OUT_DIR 写成 `OUT_DIR:<包名>/<相对>`） | 不变 | 同上第 58 行 |
| environment | 取的是判法进程环境里的值 | 构建脚本报的 `CARGO_MANIFEST_DIR`、`CARGO_INCREMENTAL`、`CARGO_PROFILE_<名>_<键>` 这几个不在任务服务的白名单里，两台都是 unset；只有命令里写了 `-- env` 才变 | `crates/singlefs-unit-graph/src/build_toolchain.rs:168` 起那几行；`task-server/task_server_lib_constants.py:401` |
| cargo-home-config | 不变 | 两台配置内容不同就变（设计如此，换来现判，不会错复用） | `scripts/gate/crate-cross-references-check.py:547` |

结论一：键里随编译目录变的成分没再找到；随主机变的只剩有意收的配置内容与工具链。这一半成立。

结论二（打中，最脆弱的一条）：反过来，会改事实而没进键的生效构建环境还有好几样，第二台或换了配置时会照旧复用另一台的事实：
- 家目录里的 cargo 配置只在上一条事实的变量里有 HOME 或 CARGO_HOME 时才进键：「locations = set() if broken("home-paths-keyed") else variables & set(CARGO_HOME_LOCATION_VARIABLES)」（`scripts/gate/crate-cross-references-check.py:578`）。13 个包里报 HOME 的只有带那个 profile 循环的 4 个构建脚本，其余 9 个包的键与家目录配置无关；而判法自己写「配置内容不同才可能改 rustflags 与 profile，照内容进键」（`scripts/gate/crate-cross-references-check.py:65`）。这条理由对 13 个包一样成立，代码只用在 4 个上。
- 工作区上一层目录里的 cargo 配置（cargo 从起编的目录往上逐层读，构建脚本那一侧照同样的链读：「越深越高；家目录那一份落在这条链上的只读一次」，`crates/singlefs-unit-graph/src/build_toolchain.rs:200`）不进任何一个包的键：根上只收「ROOT_INPUTS = ("Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml", ".cargo/config", ".cargo/config.toml")」（`scripts/gate/crate-cross-references-check.py:189`）。仓在两台的同一个绝对路径上，仓的上一层是不是同一份、第二台的家目录是不是另一处，判法都不看。本机现查：仓上面几层与家目录下都没有 cargo 配置；第二台没核（私有主机信息，我也读不到）。
- `RUSTFLAGS`、`CARGO_ENCODED_RUSTFLAGS`、`CARGO_PROFILE_DEV_DEBUG_ASSERTIONS` 这类环境：不在任务服务白名单里，只能写进命令；一旦写了，9 个包的键不变。

这些东西改的是 cfg，cfg 决定 `#[cfg(...)] #[path = …] mod` 与 `#[cfg] include!` 编不编进来，也就决定 dep-info 列哪些文件。反例输入在 `key_and_classify_probe.py`：alpha 包 `#[cfg(attack_probe_flag)] #[path = "../../beta/src/x.rs"] mod stolen_under_flag;` 与 `#[cfg(not(debug_assertions))]` 同样一行；无环境那一趟的事实当库里那一条，另四趟各换一样，推的结局是 alpha 在四趟里都是 `reuse=yes facts_same=no verdict=false-reuse`（复用的事实 0 处判红，真编出来 1 处），gamma（报 HOME，只挂了 attack_probe_flag 那一行）在家目录配置那一趟是 `rejudge`、在 RUSTFLAGS 与上一层配置两趟是 `false-reuse`、在关调试断言那一趟是 `sound-reuse`；beta、delta 四趟都应是 `sound-reuse`（其中上一层配置那一趟换了根与编译目录，这几行就是真 cargo 上「键不随根与编译目录变」的读数）。读数待主 agent 跑。

本仓今天的暴露面（现查）：`crates/` 下挂在 `#[path]` 前面的 cfg 只见到 `#[cfg(feature = "gpu")]`，特性由 `--all-features` 定死，这几样环境翻不动它；所以今天的真仓结局不受影响，缺口是潜在的。但对照条款：其余这一类键要「加工具链（含 cargo 解析后的生效构建环境与外部工具版本）」（`.claude/kb/decisions/13-验证路线.md:435`），这一格只收了 `cargo -V` 与 `rustc -vV`，属于条款点名的「键过窄」。判法文件头说这一组键「这一组是 cargo 自己判一个编译单元新不新鲜要看的同一组输入」（`scripts/gate/crate-cross-references-check.py:67`）；cargo 自己判新鲜时还看 rustflags 与 profile（cargo 的行为，未在本项目验证；探针量的是判法这一侧，不是 cargo 的指纹），这句外推过了头。

四句：不涉及臂；判法起 cargo 时环境就在它手里（`run.environment`），配置文件也在它能读到的路径上，看得到而没收；满足的分句是第 3 格「键里还有没有别的随主机、编译目录变的成分」的反面——不是多收了，是该随主机变的没收；改法见第五节。

### 3.3 自证 key-independent-of-compile-directory 够不够当换台复用的证据

那一格的做法：「key-independent-of-compile-directory 同一份包树放在两个根下、各配一份编译目录（OUT_DIR 前缀不同）与家目录（HOME 不同）录同一份编译输出」（`scripts/gate/crate-cross-references-check.py:153`）。

判定：不够。理由：
1. 两边的 dep-info 与构建脚本 output 是同一份录好的文本，只把 `@ROOT@`、`@TARGET@`、`@OUT_DIR:<包名>@` 换成各自的路径，「两份事实相同」是喂法造出来的。它证的是事实的写法（OUT_DIR 写成相对、HOME 换成配置内容）不带路径，证不了第二台的真 cargo 写出同一组文件。
2. 它只变了根、编译目录与 HOME 三样；3.2 那几样（别的包不收家目录配置、上一层目录的配置、rustflags 一类环境）在两台之间可能不同，这一格一样都没变，变了它也不会红。
3. 它比的是两把键相等，没走「一台写库、另一台读库」：自证在临时服务底下，命名空间按库目录取，两个根各有各的 git common-dir，两边本来就不在同一个命名空间里。真服务底下命名空间是空的、主仓与副本读同一份（`scripts/prune-server/prune_server_client.py` 文件头「键空间」），这一点按代码读成立，但这一格没碰到它。
4. 真仓三趟都派在第二台、同一个编译目录（日志核到，见第四节），证的是同一台同一目录上的确定性。换台的那一次只有修 HOME 之前那两趟（本机编、再到第二台），当时 4 个包现判；修了之后没有一次真仓换台。

够用的证据要有一次真仓跨台：两趟单格分别落在两台、库里那一条是另一台写的，第二趟「现判 0」；任务服务不钉台，可以交几趟到两台各落一次为止，再按日志里第二台那几行认。我的探针只能补一半：它在同一台上用真 cargo、换根换编译目录，量 beta、delta 两包键与事实都不变（推的，待跑），量不到第二台。

## 四、「已经量的」一节逐句核

核法：任务服务尝试目录里每一趟的 `spec.json`（命令）与 `result.json`（`exit_code`），加 `/home/fy5090/.local/state/singlefs/task-server/logs/` 下那一趟日志的原文（只读）；判「量的是不是被判那一版」拿被判文件此刻的改动时间与那一趟收尾的时间比（`tools/measured_versus_judged_versions.py` 现算，它打的行带任务号，不抄进来）。`type=` 那一段存在任务服务的库里，我不读服务的库，只核到 `exit=`。

| 句 | 核到没有 | 说明 |
|---|---|---|
| 第 1 组：五份换上 exit=0 | 核到 | 五份规格（spec-detector-hooks、spec-placement-hook、spec-doc-process-records-stage、spec-code-tooling-stage、spec-crate-roles-script）被占退 10 之后重交的那五趟 exit 0 |
| 放置类钩子、检出钩子、写闸自证 exit=0 | 核到，检出钩子与写闸量的不是被判那一版 | 那两趟之后 `.claude/hooks/bash-command-detector.sh`、`.claude/hooks/write-guard.sh`、`.claude/hooks/write-guard-selftest.py` 又被这一批之外的换上改过（换上规格在工作根 split-gov-r2-fix-hooks/ 下）；被判那一版各有一趟 `--selftest` exit 0，是那几次换上自己交的，正文没引 |
| 写闸开关 gate-fixtures-in-place-allowed 证红红在点名的五格 | 核到，量的是更早一版 | 日志 5 行 `✗ 自检：整份换上:…` / `stdin:…`，与 `.claude/hooks/write-guard.sh:329` 起点名的五格逐字相同；写闸与它的自证文件后来都改过，被判那一版没有这条证红 |
| 放置类钩子、检出钩子、crate-roles-check 开关复跑 exit=0（算键库修复版换上之后重量也是 exit=0） | 核到 | 两次都 exit 0；检出钩子那一次重量在它最后一次被改之后，是被判那一版 |
| doc-process-records 样本档与开关复跑 exit=0 | 核到 | 样本档 exit 0；开关复跑头一次 exit 1「7 条里拒算 7 条」，重量 exit 0「复跑了 7 条开关…判错的样本与文件头点名的逐条相等」 |
| 真仓 code-round-checklist 格 exit=0 | 核到 | 日志「检查了 46 份：判九格的 2 份、判八格的 44 份」 |
| 真仓 crate-roles 格 exit=1，只红 support-dependents 那一行 | 核到，量的是更早一版的门禁文件 | 日志只有那一行 `✗ support-dependents：…` 与汇总行；`.claude/gate.d/code-tooling.sh` 在那之后被这一批之外的换上改过（规格在 split-gov-r2-fix-family/ 下），被判那一版没有再跑这一格 |
| 旧判法同样红 | 核不到 | 没有一趟用旧判法跑真仓；这句出自工具员报告里标「推的」的那一段（`/home/fy5090/.local/share/singlefs/work/after-split-placement-round-two-rebase/result.md` 第 159 行，转述：它写的是「推的是 exit=1…按旧判法这一行同样红」），进了「已经量的」就成了量的 |
| 第 2 组：换上 exit=0 | 核到 | 预演、换上两趟 exit 0 |
| apply-writer-patch、patch-of-the-current-tree、merge-and-verify 三份自证 exit=0 | 核到，merge-and-verify 量的是更早一版 | `scripts/gate/merge-and-verify.sh` 与 `scripts/gate/merge-and-verify-selftest-lib.sh` 后来被这一批之外的换上改过（规格在 merge-verify-unresolved-to-commit-gate/ 下）；被判那一版有一趟自证 exit 0，正文没引 |
| patch-of-the-current-tree 开关复跑 exit=0 | 核到 | 两次都 exit 0 |
| apply-writer-patch、merge-and-verify 两份开关复跑没跑成 | 核到 | 前者「代码里不读弄坏开关了」「复跑了 0 条、0 份」；后者「146 条里拒算 146 条」；merge-and-verify 被判那一版之后的开关复跑又交过三趟，退 1、143、1，仍没跑成 |
| 第 3 组：两份换上 exit=0 | 核到 | 预演两趟与换上两趟 exit 0 |
| replace-once…写闸自证与 doc-text 样本档 exit=0 | 核到，检出钩子与写闸量的是更早一版 | 八趟 exit 0；后两份同第 1 组那一句 |
| 写闸五条开关证红红的格数与点名逐格相同 | 核到，量的是更早一版 | 五趟各退 1，`✗` 行数 1、1、2、7、27；前四条的格名与 `.claude/hooks/write-guard.sh:173` 到 `:188` 点名的逐字相同，27 那一条只核了条数 |
| replace-once、replace-batch 开关复跑 exit=0 | 核到 | 两次都 exit 0 |
| 原先被拒写的 13 处只改路径的行都写进了 | 核到 | 拒写清单 12 行逐行现查：那一行里旧路径 0 处、新路径 1 处；另一处（records/2026-11-06-注释只写现状.md 第 109 行）现在带实验包的新路径 |
| 第 4 组：判法自证 17 格 exit=0 | 核到，被判那一版 | 日志「自证 17 格都过」 |
| 17 条开关证红红的格与点名逐格相同 | 核到，被判那一版 | 17 趟各退 1，逐趟把日志里 `✗ 自证 <格>：` 的格名集合与判法文件头那一条点名的集合比，17 条全相同 |
| 门禁样本档 exit=0，8 条样本开关判错的样本与点名逐份相同 | 量的不是被判那一版 | 这 9 趟都收尾在判法最后一次换上（换上规格在 after-split-cross-package-key-host-independent/ 下）之前；那次换上之后没有再跑门禁样本档与样本开关。抽核两条：dep-info-unread 那一条判错的 10 份样本与门禁文件头第 214 行起点名的 10 份逐份相同，out-dir-not-allowed 那一条只错 green 样本，与第 225、226 行相合 |
| 真仓单格三趟 exit=0；空库那一趟每包「库里那一条认不得」，另两趟「现判 0、复用 12、强制现判 1」 | 核到，被判那一版 | 空库那一趟「库里那一条认不得」13 行、成功句「现判 13、复用 0」；另两趟成功句原文相同，哨兵 3 条都复用，强制现判的那一包与库里相同 |
| 三趟都派在第二台同一个编译目录，换台复用没量到 | 核到 | 三趟都走第二台那一步，留着的本地编译目录是同一个标签 |

小结：上表 22 句里，核到而量的就是被判那一版的 13 句；核到、但量的是被判文件更早一版的 7 句（检出钩子与写闸的自证两处、写闸两组开关证红、crate-roles 真仓格、merge-and-verify 自证、门禁样本档与 8 条样本开关）；核不到的 1 句（旧判法同样红）；另 1 句（第 2 组两份开关复跑没跑成）如实写着没跑成。量的不是被判那一版，是因为开工快照记在别的会话又换上了 `.claude/hooks/write-guard.sh`、`.claude/hooks/write-guard-selftest.py`、`.claude/hooks/bash-command-detector.sh`、`scripts/gate/merge-and-verify.sh`、`scripts/gate/merge-and-verify-selftest-lib.sh`、`.claude/gate.d/code-tooling.sh` 之后；`scripts/gate/crate-cross-references-check.py` 则是这一批自己最后一次换上之后没重跑门禁样本档。

越位（第一个攻击面）：正文说「34 份就是被判的文件」，而这 34 份里上面那 6 份此刻的内容含有这一批四组改动之外的改动；判它们「判得对不对」等于连别的会话的改动一起判，正文没把那些改动列进「改了什么」，腿也没有分它们的依据。判决里要么把这 6 份的结论限定到这一批改动的那几段（`git diff` 按段认），要么照新快照把别的会话的改动写进正文。

## 五、改法与各修哪一格

全部是我提的，只在我的模型上量过——实际连模型都还没跑，读数都是推的；被攻过零轮。

| 改法 | 修的格 | 在打中的格上还中不中 |
|---|---|---|
| 放置词加「放到」「移到」「挪到」「挪进」「并进」，问处词改成认「哪一个包」「哪个子目录」、「新开」与「包」之间隔两字以内、加「单开包」 | 2.1 漏拦那 6 问 | 推的：漏拦 6 问都改成拦；误拦 4 问不变，加的词还会添新的误拦 |
| 落点词「包」只在前面紧挨着 `.claude/gate.d/crate-roles.tsv` 登记的包名或「验证包」「实验包」这类规则里的称呼时才算，「目录」只在同一问里出现 `crates/` 时才算 | 2.1 误拦的「草稿目录」那一问与标题那一问 | 推的：时间意义的「放在提交前做…验证包」仍拦；这一格要么接受、写进钩子「管不到的」，要么交用户定 |
| 写闸与 `.claude/agent-common.md:45` 的「判不到」补一句：同一字面的旧号挪行、判据行 session-descriptions 认的描述整句换成指别的会话，都不拦 | 2.3 | 不修行为，只把限度写明；要真拦就得对会话描述那一类按行文相抵，会让同一行只改路径的写法再被拒 |
| 不属任何包的文件只放行 `.toml`、`.md`、`.tsv` 三种后缀，与别的包同一套 | 2.4 的 `.inc` 那一条 | 推的：修；`.md` 当模块与 OUT_DIR 中转两条不修 |
| 构建脚本报的 rerun-if-changed 落在别的包目录、后缀不是三种数据后缀的，照跨包源码分类（放行表可登） | 2.4 的 OUT_DIR 中转那一条（报了 rerun-if-changed 的） | 推的：修报了的；没报的仍判不了。代价：验证包、配对键包这类为算键读别的包源码的构建脚本会一批判红，要逐条登放行表 |
| `.md`、`.toml`、`.tsv` 被 `#[path]` 当模块编进来：dep-info 分不出，另加一格按源码字面认 `#[path = "…(.md|.toml|.tsv)"]` 与 `include!(` 指这三种后缀 | 2.4 的 `.md` 当模块那一条 | 推的：修字面写法；经宏拼出来的路径仍漏 |
| 判法文件头第 5 行与放行表表头改成与 `.claude/rules/directory-layout.md:24` 同一个说法；放行表第 7 行的理由照实写（编进来的是整份构建脚本，测的是那个函数，⑧ 不管 `#[path]`） | 2.4 的正文矛盾 | 只改文字 |
| 键里加生效构建环境：每个包都收家目录配置（不看事实里有没有 HOME）、收根往上每一层的 `.cargo/config`/`config.toml` 内容、收 `RUSTFLAGS`、`CARGO_ENCODED_RUSTFLAGS`、`CARGO_BUILD_RUSTFLAGS`、`CARGO_PROFILE_*` 在环境里的取值 | 3.2 的 false-reuse 各行 | 推的，待主 agent 跑（照探针各趟逐行比：改了之后 alpha、gamma 各趟应从 `false-reuse` 变成 `rejudge`，beta、delta 不受影响） |
| 用户在 D13 已定项 17 为「判据本身取决于文件位置」的判定开第二个例外，写明这一格收相对路径与包目录、改名照头一次现判 | 3.1 | 要用户定、走设计推论轮；治理文件轮里不能自己开 |
| 「已经量的」里那 7 句在被判那一版上重量、引新结局；「旧判法同样红」改标推的或删掉 | 第四节 | 命令见下一节 |

## 没打中的形状

- 前像比对：在 2.2 列的范围里找写六段替换的生成方，没找到；也查了读替换表的 `scripts/gate/merge-and-verify-items-lib.sh`（取第一段、要求 NF 不少于 6），十二段照读，不误拒。
- 相抵：「删一个旧号、加一个不同的号」与「同一个号多写一处」都照拒，没造出绕法。
- 复用键：找了随编译目录变的成分（OUT_DIR、构建脚本 output 里的绝对路径、CARGO_TARGET_DIR、`CARGO_MANIFEST_DIR` 的取值），都没在键里；判定库名按代码读不带仓位置，主仓、副本与两台读同一份。
- 副本写 `items/` 盖掉主仓那一条：按代码推，只会让下一趟多现判，不会错复用（错复用要求键相同，键相同就要求包内文件与读过的外部文件都相同）。
- 13 处只改路径的行：逐行现查都已写进，没有一处旧路径留着。

## 这条腿自己的限度

- 三份探针都没跑，第二、三节里标「推的」的读数等主 agent 跑出来才算数；尤其 2.1 的 12 问，我是按词表手判的。
- 第二台上的东西（家目录与仓上一层有没有 cargo 配置、真 cargo 输出）我读不到，3.2 对第二台的说法都是推的。
- 第四节只核到 `exit=`；`type=` 存在任务服务的库里，我没读。
- 「量的不是被判那一版」按文件改动时间与那一趟收尾时间比，没逐份 diff 别的会话改了哪几段，也就没判那些改动会不会改变那几趟的结局。
- cargo 判新鲜时看 rustflags 与 profile，是 cargo 的行为，未在本项目验证；探针量的是判法这一侧，不量 cargo 的指纹。

## 要主 agent 跑的命令

每项一条交任务的命令；结局看那一条自己任务末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，再看 `task-server/task log <号> --all` 里点名的那几行。

1. 模型复跑（三份探针）：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-attack -- bash /home/fy5090/code/singlefs/research/prompts/after-split-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-attack/probe-run /home/fy5090/code/singlefs/research/prompts/after-split-governance-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/after-split-governance-r1-attack`。
   - 看：`exit=0`；`name=judged_files_check lines=4`；`name=netting_*` 六行的 `new_hits` 与 `verdict`；`name=placement_*` 十二行的 `exit` 与 `verdict`；`name=compile_*` 五行都是 `cargo_exit=0 missing_dep_info=0 problems=0`；`name=key_*` 二十行的 `verdict`；`name=classify_delta` 几行的 `category`。读数与第二、三节表里推的不一样的，以读数为准，判决里逐行写去向。
   - 跑完删 `/home/fy5090/.local/share/singlefs/work/after-split-governance-r1-attack/probe-run`（里面有探针的小编译目录，任务服务没给 CARGO_TARGET_DIR 时落在 `key-probe/targets` 下）。
2. 门禁样本档在被判那一版判法上重跑：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-independence-and-sync.sh 的样本档; 范围=1 道阶段的样本; 为什么=判法最后一次换上之后没有在被判那一版上跑过样本档' task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item checker-independence-and-sync.sh`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看：`exit=0`，没有「个用例判错」那一行。
3. 门禁样本开关在被判那一版上重量：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 stage-break-switch-replay 格点 checker-independence-and-sync.sh; 范围=1 格 1 项; 为什么=8 条样本开关量在判法最后一次换上之前，要在被判那一版上重量' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay --item checker-independence-and-sync.sh`
   - 在哪起：仓根。
   - 看：`exit=0`；「判错的样本与文件头点名的逐条相等」那一句与「拒算 0」。拒算不是 0 的，照它点名的文件:行办，不当过。
4. 写闸开关证红在被判那一版上重量：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格点写闸; 范围=1 格 1 项; 为什么=写闸与它的自证在这一批的证红之后又换过，要在被判那一版上重量' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/write-guard.sh`
   - 在哪起：仓根。
   - 看：`exit=0` 与复跑了几条；尝试目录里已有一趟在写闸最后一次换上之后跑的同一条、退 0，主 agent 也可以核它的日志直接引，不重交。
5. crate-roles 真仓格在被判那一版门禁文件上重判：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 crate-roles 格; 范围=1 格; 为什么=门禁文件在这一批量过之后又换过，要在被判那一版上重判真仓' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check crate-roles`
   - 在哪起：仓根。
   - 看：`exit=1`，`✗` 只有 support-dependents 那一行；多出别的红行就是门禁文件后来的改动改了结局。
6. 换台复用（同一条交到两台各落一次为止，任务服务不钉台）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-independence-and-sync.sh 的 crate-cross-references 格; 范围=1 格; 为什么=量换台复用：库里那一条由另一台写下之后这一台现判几个' task-server/task submit -- bash .claude/gate.d/checker-independence-and-sync.sh --check crate-cross-references`
   - 在哪起：仓根。
   - 看：日志开头有没有「第二台那里 cargo metadata --offline 过了」（有就是落在第二台）；成功句「单元键：现判 N、复用 M、强制现判 K」，两趟落在不同的台而第二趟 N 为 0，换台复用才算量到。

不经任务服务的一条：`tools/measured_versus_judged_versions.py` 读的是本机任务服务状态目录（不在共享根上，交 submit 会当场退 3），主 agent 现查时直接起或照第四节的核法手查：`python3 /home/fy5090/code/singlefs/research/prompts/after-split-governance-r1-opus-model/tools/measured_versus_judged_versions.py /home/fy5090/code/singlefs /home/fy5090/.local/state/singlefs/task-server/attempts`，看每行的 `earlier-version` / `judged-version`。

## 没做什么

- 不判正推腿那几格（第 1、4、5 格），不替主 agent 采纳。
- 没往统一任务服务交任务；三份探针、门禁样本档、开关复跑与换台复用都没跑，判决要引的数由主 agent 复跑（上一节）。
- 没读任务服务的库（`type=` 没核），没读第二台上的任何东西。
- 2.2 那条整表换进变异表的路、第四节那 6 份被别的会话改过的文件，只指出，没追是谁改的、改了哪几段。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/after-split-governance-r1-attack`（1.5M）只有我写的读日志辅助脚本与它们的输出，没有仓副本、编译目录与库拷贝，留着给主 agent 现查；用完由主 agent 删。
