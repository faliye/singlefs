# 拆后收口治理轮第一轮：正推腿报告（判据格 1、4、5）

- 轮名 after-split-governance-r1，腿：正推（three-way-forward），日期 2026-11-20。
- 被判：正文「改了什么」四组改动，开工快照点名的 34 份。核快照：`sha256sum -c research/prompts/after-split-governance-r1-snapshot/judged-sha256.txt` 去掉 OK 行之后输出为空，34 份此刻的 sha256 都等于快照。
- 方案按 `crates/` 今天的实现来谈：这一批没改 `crates/`，本腿读的是判法、钩子、登记表与规则正文；没起任何脚本、没跑模型（命令都写在最后一节）。
- 范围：被判文件里还带着同期别的改动（`git diff HEAD` 比这四组大得多），不属这四组的我不判、不追查；「这一批改的句子」按正文四组各自点名的位置取。
- 引文：每一句引文都在被引文件里现查过原文，行号取命中的那一行；认不得原文的写「转述」。收口前跑 `scripts/agents/cite-check.py`，末行贴在报告尾。

## 模型与复跑（只有一个，管格 1 第 4 组那一问）

模型要回答的问题：门禁 crate-cross-references 的判法不在源码字面上认写法，靠「rustc 给每个编译单元写的 dep-info 列着它真读的每份文件」这一前提
（`scripts/gate/crate-cross-references-check.py` 文件头）。门禁样本档里的 dep-info 是录好的、由假 cargo 回放（同文件头「样本」一段），
它证明分类对，不证明真 rustc 真把这九种写法读到的文件写进 `.d`。模型用真 cargo、真 rustc 补这一格：
造 provider 与 16 个无外部依赖的消费包（各用一种写法读 provider 包的：`#[path]` 平写、cfg_attr 里的 path、改名导入的 include_str、方括号定界、原始字符串 path、行内模块里的 path、绝对路径、
经 `.inc` 中转、名叫 target 的目录、点起头的目录、读 `.md` 与 `.json` 数据文件、构建脚本自己源码里的 `#[path]`、库里 `include!`、构建脚本运行时读并报 rerun-if-changed、对照的同包 `#[path]`、编不过的 include_str），
`cargo check --offline --all-targets --all-features`，读 target 里每个 `.d`，逐个消费包问一句 provider 的文件列没列进去。

模型目录 `research/prompts/after-split-governance-r1-sonnet-model/`，文件与 sha256：

```
562794ae5b9a3869f6e5dace6df4c7bd884b5a2e6b909847e35e0cfd91aee49e  tools/make-workspace.sh
d45b160408ae2c5d9c11604a8072381f452d8a654edd1bb28b361ecf55d916e5  tools/run-dep-info-model.sh
69507cb058fddcacd582cede002b64e87230fc78434fa566a9bbb6ca8e7ac0fe  tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
a393b8b6f528d0563125bcd02de658ab1f40528ab9a366d44e2fbe8e42f0cd70  out/expected.txt
98ea6e4f216f2fb4b69fff9b3a44842c38686ca685f3f55dc48c5d3fb1107be4  frozen-stub/README.txt
```

复跑命令（外包一层、先删运行目录再调同一份 `tools/rerun.sh`）写在最后一节「要主 agent 跑的命令」第 1 条。
探针的内存与时间：我没跑（子 agent 不起 cargo），没量过；推的：16 个包都没有外部依赖，一遍 cargo check 秒级到十几秒、内存几百兆以内。
`rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 的拷贝，只在 `rerun_steps` 里加了一段「模型目录里有 `tools/run-dep-info-model.sh` 才起」，模板自带的自证因此仍在拷出去的目录里跳过它。
我只做了 `bash -n`，没起过一次（要起 cargo）。

## 格 1：拦不拦得住它要拦的事

### 第 1 组（放置类弹窗钩子等）

输入甲：「整包跑哪几个包」那份弹窗原文（钩子自证里的原文：`.claude/hooks/ask-user-placement-guard.sh:291`「必放：只问测试范围、选项里点名测试包与实验包（整包跑哪几个包）」）。

- 现在由哪一行判、判成什么：`is_placement`（`.claude/hooks/ask-user-placement-guard.sh:168`「def is_placement(question):」）先剥「」括着的原文，再要问处词或放置词加落点词；
  问处词表在 `.claude/hooks/ask-user-placement-guard.sh:89`「WHERE_WORDS = ("哪个包", "哪个目录", "哪个 crate", "新开包")」，放置词表在 `.claude/hooks/ask-user-placement-guard.sh:90`
  「PLACEMENT_WORDS = ("放哪", "放在", "放进", "搬到", "搬进", "住进", "归位")」。
- 我把自证里那份原文（`.claude/hooks/ask-user-placement-guard.sh:291` 起的六行）逐词 grep：十一个问处词与放置词在里面一次都没有命中（「哪几个包」不是「哪个包」；原文里是「搬走」不是「搬到」「搬进」）。
  没有问处词、没有放置词，不是放置类，退 0 放行。判对。
- 事实核：另做了一份同文的 JSON 喂法 `$SINGLEFS_WORK_ROOT/after-split-governance-r1-forward/feeds/pack-run.json`（绝对路径在最后一节），要主 agent 经钩子真喂一次，期望退 0、stderr 空。

输入乙：一份真的问「E202 放哪个包」的弹窗。

- 问句「E202 的装置放在哪个包？」里有问处词「哪个包」，直接是放置类（同函数）；`question_problems`（`.claude/hooks/ask-user-placement-guard.sh:198`「def question_problems(question, section):」）再要齐四样：
  问句点名规则文件与小节、选项说明以「标准：」或「偏离：」起头、「偏离：」引那一节的原话、至少一个「标准：」。没点名、没起头的缺三样，退 2，stderr 逐问列出。自证对应格：`.claude/hooks/ask-user-placement-guard.sh:297`「必拒：放在哪个包」。判对。
- 合规写法（点名规则、一个「标准：」、一个引了原话的「偏离：」）退 0：自证格「必放：放置类，问句点名规则那一节」，喂法 `e202-well-formed.json`。
- 打中一处（不是钩子判错，是规则正文与钩子对不上，放格 5 报）：「偏离：」没用「」照抄那一节原话（例：「偏离：整份搬进测试包，偏离不为测试另开包那一条」）钩子拒，
  条件是「」里至少 `.claude/hooks/ask-user-placement-guard.sh:102`「QUOTE_MINIMUM = 4」个字且逐字是那一节的一段（`.claude/hooks/ask-user-placement-guard.sh:192` 的 `quotes_the_section`）。
  喂法 `e202-deviation-not-quoted.json`，期望退 2、点名 deviation。
- 管不到（规则正文自己也承认，放置类问题换了词表外的说法认不出）：「E202 的装置落在哪？」（喂法 `e202-reworded.json`）没有词表里的词，退 0 放行；`.claude/rules/directory-layout.md:45` 把它划给 review。不算打中，写在这里是为了判误放的边界。

判定：第 1 组主体（两个点名输入）成立；另有一处打中，在下面「第 1 组附带」。

第 1 组附带（打中）：只判八格的名单「只缩不长」没有会红的检查。

- 门禁文件头写「名单只缩不长」（`.claude/gate.d/doc-process-records.sh:502`「另判名单只缩不长：名单里每一行指的正文都要在」），判法只核每一行写得成「路径、TAB、# 理由」且指的正文在
  （`.claude/gate.d/lib/doc-process-records-rounds-and-ledger-cells.sh:1187`「[[ -f "$listed_path" ]] || list_problems+=」），没有拿基准那一版比；对照同一批的 `crate-roles-lag.tsv` 经 `lib-lag-baseline.py` 比基准、多出行判红。
- 反例输入：往 `.claude/gate.d/code-round-checklist-eight-cells.tsv` 加一行 `research/prompts/_新题-code-r1-body.md<TAB># 理由`（正文在、行写得对）：门禁不红，
  这份新正文标题日期不论多晚都只判八格（`.claude/gate.d/lib/doc-process-records-rounds-and-ledger-cells.sh:1166` 的 `code_round_checklist_wants_placement` 先看名单）。
  机器没拦「不往里加新正文」，只有文字写着。现象推翻它：门禁自证里有一格往名单加行而判红，或另有 lag 基准比对读这张表。我没找到。

### 第 2 组（打补丁前核前像）

输入：补丁写好之后、打之前，别的会话改了同名那一行。

- 判的链：`plan_with_three_way`（`scripts/gate/apply_writer_patch_plan.py:951`）读被打那棵树此刻的 `crates/mutations.tsv`，`mutation_edits`（`scripts/gate/apply_writer_patch_plan.py:893`）把替换读成「新行六段、前像六段」、
  把删除读成前像六段，`preimage_problems`（`scripts/gate/apply_writer_patch_plan.py:926`）拿被打那棵树此刻同名那一行逐字比前像：
  `scripts/gate/apply_writer_patch_plan.py:933`「current.setdefault(line.split」取此刻的行，`scripts/gate/apply_writer_patch_plan.py:943`「if now != expected:」判不等；
  `scripts/gate/apply_writer_patch_plan.py:1008`「problems += preimage_problems(」并进问题清单，`scripts/gate/apply-writer-patch.py:1004`「if problems:」之后一律走
  `scripts/gate/apply-writer-patch.py:1005` 的「return 2, ([f"✗ {problem}" for problem in problems]」：退 2、逐行写行名与两侧原文、一个字不改。
- 判成什么：整份拒、退 2、不改主工作区。输入对应的自证格：`scripts/gate/apply-writer-patch.py:1358`「前像比对（文件头「补丁目录里的文件」）」那一段里的 preimage-replace-mismatched 与 preimage-delete-mismatched：
  `fixed` 那行此刻是 `o-fixed`、前像是 `o`，期望退 2、树不动、同一份里对得上的 `swap` 或 `drop` 也不打。判对。
- 比对到打之间还有一层：换上之前再核一次主工作区每份文件的 sha256 仍是核的时候的（`scripts/gate/apply-writer-patch.py:719`「主工作区此刻的 sha256 已经不是核的时候的改前」），变异表合并出的全文是要换的文件之一
  （`scripts/gate/apply-writer-patch.py:679` 的 `if merged is not None:` 把它写进临时目录的新全文），所以核与换之间别的会话再动那一行，换上这一步也拒。判对。
- 比对里有三处我逐个看过没发现错：只比此刻表里「名字恰好出现一次」的行（`scripts/gate/apply_writer_patch_plan.py:940`「if names.count(name) != 1:」，名字不是一行的由 1006 行那一句另报）；
  比的是去掉行尾换行符的整行（制表符算在里面）；变异表里没有 `\r` 与 Unicode 行分隔符（我 grep 过全表，8669 行都没有），`splitlines` 与逐行读的切法在这张表上一致。
- 合补丁那一支（`scripts/gate/merge-and-verify.sh:1930`「"mutations-replacements.tsv": [line if preimage_dropped else f"{line}\t{old_by_name[name]}"」）写十二段，
  `scripts/gate/patch-of-the-current-tree.py:262`「if len(fields) != REPLACEMENT_FIELDS:」拒六段；两个生成方与判法的段数、前像取法三处一致。生成方之外还有没有写六段旧格式的地方，归攻方腿（格 2）。

判定：成立。

### 第 3 组（写入一侧判号，相抵单位换成违例本身）

输入甲：行里原有任务号、只改同一行路径。输入乙：同一行新加一个号。

- 判的链：写闸 `.claude/hooks/write-guard-decisions.py:388`「library.new_violation_hits(form_key, before, after, relative)」、检出钩子 `.claude/hooks/bash-command-detector-literals-and-signals.py:380`
  「found = library.new_violation_hits(form_key, before, after, relative)」、三份定点写入脚本与两份追加工具经 `scripts/kb/lib_notation_check.py:122`「hits = library.new_violation_hits(form_key, before_text, new_text, relative)」，
  都落在 `.claude/gate.d/lib-forbidden-notations.py:1175`「def new_violation_hits(form_key, before_text, after_text, relative_path=''):」。
- 相抵：`.claude/gate.d/lib-forbidden-notations.py:1206`「remaining[key] -= 1」按 `(判据行名字, 命中的串)` 计数，写后某一种比写前多出几处才交几处。
  输入甲：写后那一行的任务号字面与写前相同，计数相抵，一处都不交，放行。输入乙：新加的号字面不同（或同字面但计数多出一处），多出来的交出来，拒，行号是写后文件的行号。
- 自证格：`scripts/kb/lib_notation_check.py:273`「('行里已有任务号、同一行只改路径'」期望放行（红在 violations-netted-by-line-text 或 violations-not-netted），
  `scripts/kb/lib_notation_check.py:275`「('行里已有任务号、同一行新加第二个任务号'」期望拒（红在 violations-netted-by-line-number），
  `scripts/kb/lib_notation_check.py:281`「('整份多出一行带同一个旧号'」期望拒且只点名新写的那一行。三份脚本的自证共用这一个函数，所以三份各判一遍。判对。
- 两个调用口各自的边角我看过：写闸写前读不了文件时放行并记一条没判（`.claude/hooks/write-guard-decisions.py:379`），文件整个被清空时不判；这是显式的「判不了放行但必记」，不是静默放行。

判定：成立。

### 第 4 组（跨包源码引用拆成 crate-cross-references 格）

先说读法：这一格不在源码字面上认写法，读 `cargo check` 写的 dep-info。每个编译单元的 `sources` 逐份过 `classify`
（`scripts/gate/crate-cross-references-check.py:462`「def classify」）：同包放行；构建脚本生成与依赖源码放行；落在别的本地包目录下的，后缀属 `DATA_SUFFIXES`
（`scripts/gate/crate-cross-references-check.py:190`「DATA_SUFFIXES = (".toml", ".md", ".tsv")」）放行，别的一律判红（`scripts/gate/crate-cross-references-check.py:476`「return "data" if path.endswith(DATA_SUFFIXES) else "cross"」）；
不属任何包的，只有后缀属 `SOURCE_SUFFIXES`（`scripts/gate/crate-cross-references-check.py:191`「SOURCE_SUFFIXES = (".rs", ".wgsl")」）才红。红的再拿两张表放过。

逐个输入：

1. `#[path]`、`include_str!` 九种写法。门禁样本档给了十种（平写对照加九种变体，名字在 `.claude/gate.d/checker-independence-and-sync.sh:142` 起）：
   不论哪种写法，rustc 把读进来的文件写进 `.d`，判法只看 `.d`，所以每一种都落进 `classify`：别的包目录下的 `.rs`、`.wgsl`、`.inc` 都是 `cross`，红。
   这一前提样本档只能靠录好的 `.d` 回放，没有真 rustc 的数。所以本腿做了模型（上面）：真 rustc 编十五个写法，读 `.d`。结果待主 agent 跑（第 1 条命令）。在模型出结果之前，这一条我只能写「推出，待模型」。
2. 构建脚本单元读别的包。构建脚本单元自己的源码（`build.rs`、它 `#[path]` 的文件）在 `build_script_build-<哈希>.d` 里，判法有专门一格（样本档与自证 build-script-dep-info-read），模型 c12 对应，期望列进。成立。
   但构建脚本**运行时**读别的包、并报 `rerun-if-changed` 的，不在 `.d` 里，也不被判红：`scripts/gate/crate-cross-references-check.py:509`「for path in unit["sources"]:」只遍历编译单元的 `sources`，
   构建脚本报的路径只进 `fact["build"]["inputs"]`（`scripts/gate/crate-cross-references-check.py:410`「fact["build"]["inputs"].append」）当单元键的成分，不进 `classify`。
   判法自己的话是「这一格判不了的：……构建脚本跑时读而没报 rerun-if-changed 的文件」（`scripts/gate/crate-cross-references-check.py:962`），读起来像是「报了 rerun-if-changed 的就判得了」，实际报了也不判。
   **打中**。反例输入：模型 c14，`build.rs` 里 `std::fs::read_to_string("../provider/src/k.wgsl")` 并 `println!("cargo:rerun-if-changed=../provider/src/k.wgsl")`：
   `.d` 里没有 provider，构建脚本的 `output` 里有那一行，门禁判绿。规则正文 `.claude/rules/directory-layout.md:45` 把「运行时读的文件与过程宏自己读的文件跨没跨包」划给 review，没把「报了 rerun-if-changed 的」划出来，
   两处说法对不上。现象推翻它：模型 c14 的 `provider-listed` 是 yes，或门禁真仓上有一格把 `fact["build"]["inputs"]` 过了 `classify`。
3. 编不过的写法（例：include_str 一份不存在的文件，模型 c15）。`scripts/gate/crate-cross-references-check.py:892`「succeeded = code == 0」一旦 cargo 退非 0 或没有 build-finished，
   `scripts/gate/crate-cross-references-check.py:894`「run.red(f"cargo check 退」整组现判的包红、不入库。编不过的单元没有 `.d`，所以不会因为「没事实」变成绿。成立。
   边角：一次 `cargo check -p` 点了几个包，只要有一个编不过，这几个包一起红、一起不入库（`scripts/gate/crate-cross-references-check.py:897`），过严但不漏。

## 格 4：不兼容旧格式与旧开关

依据的规矩：`.claude/agent-common.md:46`「里程碑 17 收尾之前不兼容任何旧东西，代码与格式尤其如此」（该条同一行写了「`research/` 下是当时的记录，豁免」，门禁与脚本里只为读 research 留的放宽照留）。逐组看：

第 1 组。
- 放置类钩子：词表、整词认法、「」原文剥除都是新规则下的判据，没有认旧词表或旧问句写法的分支；`rule_word_list_mismatches` 只核规则与钩子两份词表一致，不是兼容读法。
- `crate-roles-check.py`：二进制目标逐个按 cargo 认法登记，没有「整包行」的兼容读法（整包行被判红，`scripts/gate/crate-roles-check.py:14` 同文件头写「整包目录与别的目录不收」）。
  文件头提到 edition 2015 显式写目标会关掉自动发现，「这里照开着认，多认不少认」，那是 cargo 语义的近似，不是旧格式。
- 只判八格的名单与第九格截止日：这是给 `research/prompts/` 里第九格立起之前写好的代码轮正文留的豁免，落在上面那条「research 豁免」里，成立；名单的行数没有基准比对，见第 1 组附带。
- 检出钩子 ⑭ 与写闸第十一道往已有门禁的样本档目录写都拒：认法在 `lib_write_scope.py`，没有为旧位置留的读法。

第 2 组。
- 判法本身没有为六段旧格式开读法：`mutation_edits`（`scripts/gate/apply_writer_patch_plan.py:893`）只认十二段的替换与六段的删除，别的段数走 `wrong` 那一条整份拒
  （`scripts/gate/apply_writer_patch_plan.py:914`「wrong = [f」），出路都是同一句 `PREIMAGE_WAY_OUT`，没有「按旧格式补前像」的路径。成立。
- 边界一（交主 agent 定）：弄坏开关里留了读旧格式的分支，`scripts/gate/apply_writer_patch_plan.py:904`「elif len(row) == MUTATION_ROW_FIELDS and BROKEN == "replacement-preimage-optional":」
  （六段替换照名字换）与 `scripts/gate/apply_writer_patch_plan.py:909`「elif len(row) == 1 and BROKEN == "deletion-preimage-optional":」（只写名字的删除照名字删）。
  它们只在自证里认（`scripts/gate/apply-writer-patch.py:1986` 那一段平常跑设了开关就退 2），用来证明「段数不对整份拒」那条检查真会红。
  按 `.claude/agent-common.md:46` 的字面，「只为旧行放宽的读法」与「造旧形态输入的测试与它们的变异行」不留；这几处恰好是造旧形态输入（六段、一段）的测试与读旧形态的变异。
  我的读法：它们测的是「段数不对」这条通用拒绝，六段与一段只是挑了最像旧格式的两个段数；但名字叫 `*-preimage-optional`、文字里写「没带前像」，字面上像旧格式的变异行。是否算违例是规矩的解释，我不替判。
  对应的自证格：`scripts/gate/apply-writer-patch.py:1389` 与 `scripts/gate/apply-writer-patch.py:1401` 起的 preimage-replace-without-preimage、preimage-delete-names-only，
  与 `scripts/gate/patch-of-the-current-tree.py` 的 cell_replacement_without_preimage（替换行只有六段）。
- 生成方的一致：`patch-of-the-current-tree.py:262` 对六段替换也是整份拒，没有另开「旧补丁目录」读法。

第 3 组。
- 「文件里已有的旧号不挡这一次写」是对存量内容的宽容，不是对旧格式的读法：号的写法没有分新旧两种判，判的只是这一次写比写前多出来的违例。成立。
- 边界二：`hits_netted_by_line_text`（`.claude/gate.d/lib-forbidden-notations.py:1159`「def hits_netted_by_line_text」）整个函数是改动之前的相抵算法，现在只当弄坏开关 violations-netted-by-line-text 的实现留着。
  它没有被平常的路径调用（`.claude/gate.d/lib-forbidden-notations.py:1175` 起的 `new_violation_hits` 只在开关设着时才进它），属于「旧做法留作变异」，与第 2 组边界一同交主 agent 定。

第 4 组。
- 判定记录没有 `components` 一律认不得：`decode_record`（`scripts/gate/crate-cross-references-check.py:609`）要求 `fact` 与 `components` 都是字典，否则交 `None`；
  `scripts/gate/crate-cross-references-check.py:835`「reasons[package] = "库里那一条认不得，照头一次现判"」。复用那条路读 `keys/<键>.json` 也走同一个 `decode_record`，没有第二条放宽的读法。成立。
- 边界三：滞后表机制（`.claude/gate.d/crate-cross-references-lag.tsv` 加 `--lag-base-*` 三态与 `lag_growth`）是给存量留的通路，此刻 0 行（我数过：非注释行 0）。
  它不是旧格式读法，是这个仓对存量违例的通用写法（别的格同款）；但新格从一开始就带一条零行的通路，是否值得，交主 agent。
- 放行表 5 行是确要编进别的包源码的登记，不是旧数据；本腿不判理由站不站得住（格 2）。

判定（格 4）：成立。三处边界（两处弄坏开关里的旧形态变异、一处零行滞后通路）写在上面，不是判法为旧东西留的读法。
推翻它的现象：在 `crate-roles-check.py`、`apply_writer_patch_plan.py`、`crate-cross-references-check.py` 的平常路径里找到按版本号、按「没有某一段」分叉的读法。我 grep 了这三份和 `lib-forbidden-notations.py`，没有。

## 格 5：规则与定义正文

对照 `.claude/singlefs-ai-sop/rules/rules-discipline.md`（第 1 条「怎么做，不写论证」，第 2、3、4、5、7 条）与判法现行的行为，逐句对到常量或函数。
先答正文点名的那一问：`directory-layout.md` 的后缀说法与判法的两个常量是不是一回事。

- `.toml`、`.md`、`.tsv`：`.claude/rules/directory-layout.md:24`「后缀是 .toml、.md、.tsv 的算数据文件，读它不在此列；别的后缀一律照源码算」与 `DATA_SUFFIXES = (".toml", ".md", ".tsv")`
  （`scripts/gate/crate-cross-references-check.py:190`）逐个相同。对「落在别的本地包目录下」的文件，判法也是不属数据后缀就红（`scripts/gate/crate-cross-references-check.py:476`「return "data" if path.endswith(DATA_SUFFIXES) else "cross"」），与规则「别的后缀一律照源码算（.rs、.wgsl，也包括 .inc）」一致。
- `.rs`、`.wgsl`：`SOURCE_SUFFIXES` 只管「不属任何包的文件」那一支（`scripts/gate/crate-cross-references-check.py:477`），规则里对应的是门禁表那一行 `.claude/rules/directory-layout.md:42`「或是不属任何包的 .rs、.wgsl」。
  两处合起来与判法一致；但规则的约束句（第 24 行）只写「另一个包目录下」，不属任何包的 `.rs`、`.wgsl` 在约束句里没有对应条款，只在「门禁管哪一半」的表里出现。属于约束句缺一半，建议约束句补一句。
- 后缀比较区分大小写（`path.endswith(DATA_SUFFIXES)`）：别的包里的 `README.MD` 红，不属任何包的 `x.RS` 放。规则没写大小写；真仓上不碰这个，只记一笔。

逐句对照，对不上的（打中）：

1. 弹窗「偏离：」要引原话，规则没写。`.claude/rules/directory-layout.md:25`「写明偏离」只说「写明偏离『crates/：各包放什么』的哪一条」；钩子要的是「」括着的一段、去掉空白至少 4 个字（`.claude/hooks/ask-user-placement-guard.sh:102`「QUOTE_MINIMUM = 4」）、
   且必须逐字是那一节正文的一段（`.claude/hooks/ask-user-placement-guard.sh:192` 的 `quotes_the_section`）。照规则字面写「偏离：整份搬进测试包，偏离不为测试另开包那一条」会被钩子拒
   （喂法 `e202-deviation-not-quoted.json`，期望退 2、点名 deviation）。钩子的拒绝说明里有这句话（`.claude/hooks/ask-user-placement-guard.sh:107`「它偏离的那一条的原话（至少 {QUOTE_MINIMUM} 个字）」），规则正文没有。**对不上**。
2. 「库为空」缺「库文件不在也算空」。`.claude/rules/directory-layout.md:41` 把「库为空」写成内容上的定义
   （「库为空（去掉注释、属性、use 再导出与只装这些的 mod 之后没有条目）」），`.claude/rules/verification-reference.md:156` 只写「库为空而有测试目标的只许是测试支撑」；判法还有一支：库文件不在也算空（`scripts/gate/crate-roles-check.py:328`「if not os.path.isfile(path):」，文件头 24 行同写）。
   只有二进制没有库文件、又带 `tests/` 的包（角色不是测试支撑）会被判红，规则读不出。**对不上**。
3. 代码轮检查单「更早的只判前八格」说得过宽。`.claude/rules/verification-reference.md:157`「名单里的与标题日期更早的只判前八格」，判法是：标题日期早于 `CODE_ROUND_CHECKLIST_CUTOFF`（`.claude/gate.d/doc-process-records.sh:775`「CODE_ROUND_CHECKLIST_CUTOFF="2026-11-07"」）的一格都不查
   （`.claude/gate.d/doc-process-records.sh:495`「更早的不查」，`.claude/gate.d/lib/doc-process-records-rounds-and-ledger-cells.sh:1199` 的 `before_cutoff+=` 之后 `continue`）；
   只有标题日期在 2026-11-07 与第九格截止日之间（或名单里、日期不早于 11-07）的才判八格。规则的「更早」会被读成「早于第九格截止日的都判八格」。`.claude/rules/directory-layout.md:45`「就只判八格或整份不查」是对的。**对不上**（verification-reference 那一处）。
4. `.claude/agent-common.md:94` ⑭ 与 `.claude/agents/tooling-writer.md:31` 的样本补丁出路少了一句关键写法。⑭ 写「改前的拷进 fixtures-base/、改后的放进 fixtures-new/」，没写要「照仓里的相对路径」放；
   钩子的拒绝说明写了（`.claude/hooks/write-guard-decisions.py:590`「照仓里的相对路径，改前的样本拷进」）。`git apply -p2` 只剥掉 `a/fixtures-base/` 这两层，
   所以 `fixtures-base/` 底下要是 `.claude/gate.d/fixtures/<门禁>/…` 的完整相对路径，只拷 `<门禁>/…` 补丁会指到仓根下的错位置。照 ⑭ 字面做的人会做错，拒绝说明会纠正，但定义该自足。**对不上（少一句）**。
5. 「至少两个包」与判法数的依赖者不是一回事。`.claude/rules/directory-layout.md:20`「至少两个包都无条件依赖的才单开测试支撑包」，判法数依赖者时还把根 Cargo.toml 的 members 与 exclude 点名、在 `crates/` 之外的别的 cargo 工程算进去
   （`scripts/gate/crate-roles-check.py:506`「for directory in other_projects:」）。真树上 `singlefs-device-library` 的两个依赖者是实验包与 `scripts/experiment-apparatus`（后者在 `crates/` 之外，
   两处 Cargo.toml 的依赖行我看过：`crates/singlefs-implementation-experiments/Cargo.toml:24` 与 `scripts/experiment-apparatus/Cargo.toml:254`），按规则字面是一个包、判法判绿。**对不上（少一句）**；门禁 `crate-roles` 格的说明（`.claude/gate.d/code-tooling.sh:39`）同样写「至少两个无条件的依赖者」，没有这一半。
6. 弹窗词表的细节规则没写。钩子认英文词时「crate」前后不接字母、数字与下划线、带复数 s 也认、不分大小写、词里的空格认任意空白
   （`.claude/hooks/ask-user-placement-guard.sh:16`「英文词（crate）前后不接字母、数字与下划线」）；`.claude/rules/directory-layout.md:26` 只列了词。词表逐组相同由自证核（`rule_word_list_mismatches`，
   `.claude/hooks/ask-user-placement-guard.sh:179`「def rule_word_list_mismatches(section):」），列的词一致；这是规则少写的细则，不是列错。**补充项，不算对不上**。

与 `rules-discipline.md` 的对照（这一批改的句子）：

- 第 1、2、3 条（只写怎么做，不写论证、历史）：五份文件的改动句我按 `rules-lint.sh` 的词表（`PARAGRAPH_LABELS`、`HALF_SENTENCE`、`LEXICAL_EXPLANATIONS`、日期、历史链接）逐类 grep：
  反引号与「」之外没有「为什么、依据、理由、实测、经过、原因、因为、之所以、为了、所以」起头的解释，没有日期，没有指向 `records/` 的链接。真跑 `rules-lint.sh` 的命令在最后一节第 4 条，退出码以它为准。
- 第 7 条（不用位置指代、不自称）：五份文件里没有「本节、本章节、本表、本文档、本条」；指同一份文件别的小节都用小节名（例：`.claude/rules/directory-layout.md:25` 的「crates/：各包放什么」）。符合。
- 第 8 条只判本包自己的规则，项目自己的 `.claude/rules/` 不判，不适用。
- 一处疑似论证（我读不准，交主 agent）：`.claude/rules/directory-layout.md:45` 的「代码轮正文的标题先写一个更早的日期……就只判八格或整份不查，标题日期写对没有靠人核」，
  前半句讲这个绕法是怎么回事、后果是什么，属于解释门禁为什么管不到；判据只剩「标题日期写对没有靠人核」一句。`rules-lint.sh` 的词表不会命中它。
- 一处行文问题（不是论证，是句子断了）：`.claude/agents/implementation-writer.md:71` 把前像规矩整段插在补丁目录文件列表的中间，列表的收尾「……改名写成删一行加追加一行与 result.md（这份报告）」读起来像「追加一行」与「result.md」并列。
  这一段是这一批改的句子，该把规矩另起一句，文件列表保持原样。**行文要改**。

判定（格 5）：不成立。按「这一批改的句子」逐句对到判法，有四处与判法现行的行为对不上（第 1 到第 4 点），一处少一半（第 5 点）；
论证、日期、历史链接、位置指代这四类规矩的字面检查没有命中。推翻它的现象：把第 1 点那份「偏离」不带原话的弹窗喂钩子而它退 0，或第 3 点那个日期早于 2026-11-07 的正文被门禁判了八格。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 格 1 第 1 组：「整包跑哪几个包」原文 | 成立 | 十一个词一个都没命中，不是放置类，退 0；自证有同一份原文 |
| 格 1 第 1 组：真的「E202 放哪个包」 | 成立 | 问处词命中，缺规则点名与「标准：」「偏离：」就退 2 |
| 格 1 第 1 组：只判八格的名单只缩不长 | 打中 | 只核每行指的正文在，没有基准比对，往里加一行就免掉第九格 |
| 格 1 第 2 组：打之前别的会话改了同名行 | 成立 | 前像逐字比对，对不上整份拒、退 2、不改；换上前再核 sha256 |
| 格 1 第 3 组：旧号那一行只改路径 / 新加一个号 | 成立 | 按（判据行名字、命中字面）计数相抵，前者放行、后者拒 |
| 格 1 第 4 组：九种写法与 `#[path]`、`include_str!` | 待主 agent 跑 | 判法读 dep-info，分类对；真 rustc 是否都写进 `.d` 要跑模型 |
| 格 1 第 4 组：构建脚本单元读别的包 | 成立 + 打中 | 构建脚本自己的源码判得到；运行时读并报 rerun-if-changed 的不判，说明与判法对不上 |
| 格 1 第 4 组：编不过的写法 | 成立 | cargo 非 0 或没有 build-finished 即红，不入库 |
| 格 4 不兼容旧格式与旧开关 | 成立 | 平常路径没有旧格式读法；三处边界（两处弄坏开关里的旧形态变异、一处零行滞后通路）交主 agent |
| 格 5 规则与定义正文 | 不成立 | 弹窗「偏离」要原话、库文件不在算空、检查单更早日期不查、样本补丁路径、依赖者算法五处规则少于或不同于判法 |

## 要主 agent 跑的命令

每条都由主 agent 交；我一条都没起。下面每条三样：命令原样、在哪个目录起、跑完看哪几行。

1. 模型复跑（真 cargo 编 provider 与十六个消费包、读 dep-info；外包一层，先删运行目录再调同一份 `tools/rerun.sh`）：

```
task-server/task submit -- bash research/prompts/after-split-governance-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/after-split-governance-r1-sonnet-model/frozen-stub /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward-model-run /home/fy5090/code/singlefs/research/prompts/after-split-governance-r1-sonnet-model
```

   在仓根 `/home/fy5090/code/singlefs` 起。看 `task-server/task log <号>` 里的 `MODEL_RESULT identical` 与结局行 `type=succeeded exit=0`。
   `MODEL_RESULT differs` 时，上面的 diff 逐行点名哪个写法的 `provider-listed` 与期望不同：期望 c01 到 c13 都是 yes（c11 是数据文件、c12 是构建脚本自己的源码）、c00 是 no（对照）、
   c14 是 `provider-listed=no rerun-if-changed-in-build-output=yes`（我对第 4 组构建脚本那一问的结论就押在这一行上）、c15 是 `cargo-exit-nonzero=yes`。
   任何一个 c01 到 c13 出 no，格 1 第 4 组那一条就从「推出」改成「打中」；c14 出 `provider-listed=yes`，我那一处打中撤回。

2. 模板自证（核拷出去的 `rerun.sh` 仍照模板的规矩跑）：

```
task-server/task submit -- bash research/prompts/after-split-governance-r1-sonnet-model/tools/rerun.sh --selftest
```

   在仓根起。看结局行 `type=succeeded exit=0` 与日志里「自检通过」一句。

3. 放置类钩子喂法（五份 JSON 在草稿目录 `/home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward/feeds/`，各交一条；都在仓根起）：

```
task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward/feeds/pack-run.json'
task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward/feeds/e202-where.json'
task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward/feeds/e202-well-formed.json'
task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward/feeds/e202-deviation-not-quoted.json'
task-server/task submit -- bash -c 'bash .claude/hooks/ask-user-placement-guard.sh < /home/fy5090/.local/share/singlefs/work/after-split-governance-r1-forward/feeds/e202-reworded.json'
```

   期望的结局行（`exit=` 那一段）：pack-run 为 0（stderr 空）；e202-where 为 2，stderr 点名三样缺项（问句没点名规则那一节、选项说明没以「标准：」「偏离：」起头、一个「标准：」选项都没有）；
   e202-well-formed 为 0；e202-deviation-not-quoted 为 2，stderr 点名「选项没用「」引……原话」那一条（格 5 第 1 点）；e202-reworded 为 0（词表外的说法认不出，规则正文已承认）。
   任何一条与期望不同，格 1 第 1 组或格 5 第 1 点的判定改写。这是真喂钩子的入口（stdin 的 JSON），钩子读不到规则那一节时会放行并记一条检出，那种情况日志里会有「放置类弹窗闸没判」一句，要先排掉。

4. 规则纪律的字面检查（五份文件里这一批改的句子，格 5 的论证、日期、历史链接那一半）：

```
task-server/task submit -- env RULES_LINT_DIR=.claude/rules RULES_LINT_FILES='.claude/agents/*.md .claude/agent-common.md' bash .claude/singlefs-ai-sop/scripts/rules-lint.sh /home/fy5090/code/singlefs
```

   在仓根起。看结局行 `type=succeeded exit=0`；红的话看输出里点名的文件与行是不是这五份里这一批改的句子。我按它的词表 grep 过没有命中，以它为准。

## 没做什么

- 不判别的腿的格：格 2（误伤与绕过）、格 3（复用键与多机）与「已经量的」那一节每一句能不能从日志里核到，归攻方腿，本腿没碰。第 1 组放置类词表会不会拦下无关弹窗、前像比对在两个生成方之外还有没有六段旧格式生成点、相抵能不能被挪号绕过，都没判。
- 不替主 agent 采纳或出判决；格 4 的三处边界、格 5 的行文与论证一处，都只写了我的读法，是否违例交主 agent。
- 没往统一任务服务交任务，判决要引的数没复跑；要跑的命令都在上一节，由主 agent 交。模型只做了 `bash -n`，没起过一次，模型里每个文件写法对不对要等第 1 条的结果。
- 没读禁读清单里的文件；没改仓里 `crates/`、规则、钩子、门禁与登记表；写的只有这份报告、模型目录 `research/prompts/after-split-governance-r1-sonnet-model/` 与草稿目录里的喂法。
- 开工快照里的文件还带着同期别的改动，那些没有逐一判；只看了正文四组点名的位置。
- 观察到但不属本腿格的几处，不判，交主 agent：门禁 `code-tooling` 的 crate-roles 样本档（`.claude/gate.d/code-tooling.sh:120` 起）没有「产品包经 [dependencies] 依赖测试支撑包」那一格的样本，只有判法自己的 `--selftest` 覆盖；
  `.claude/gate.d/break-switch-replay-files.tsv` 本批新加的行末注释里带日期与「此前……那一版」这类改动经过，对照 `.claude/agent-common.md:47` 的「只写现状」，是否在 comment-history 格射程里我没判。
- 草稿目录里的 `judged.diff`、`pop.txt`、`files.txt` 是我的中间文件，已不再用；交回前删，大小写进交回正文。

## 引文核对（交回前）

`python3 scripts/agents/cite-check.py research/prompts/after-split-governance-r1-sonnet-output.md --background research/prompts/_after-split-governance-r1-background.md` 末行：

```
✓ 核了 55 处引文、42 处裸行号，对上 55 处，没判 42 处（其中裸行号没有引文可比 42 处；逐处列在上面）
```

42 处裸行号没有引文可比；写进来之前我对着文件读过被引行，但脚本没核，当「读过、没机器核」。
