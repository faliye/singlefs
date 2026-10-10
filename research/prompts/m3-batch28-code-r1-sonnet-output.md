# m3-batch28-code-r1 正推腿报告（Sonnet，2026-10-06）

立场：正推。格：K1（H-B、H-V、T1–T3 与变异行）、K3（5 份每份一格）。方案按 `crates/` 今天的实现来谈；引代码写冻结副本 `/tmp/claude-1000/m3-batch28-code-r1/tree/` 里的路径与行号（下文路径省去这个前缀，`cite-check` 用 `--root` 解析），引 kb 写 kb 文件自己的行号。

## 0 材料核对

- 冻结副本 5 份被判文件的 sha256 与 `research/prompts/m3-batch28-code-r1-snapshot/crates-sha256.txt` 逐份相同（命令 `sha256sum` 五份，输出五行与快照五行一致：walk.rs 1130acbc…、crash_judge_checker.wgsl 9a96e6dd…、GPU 对拍测试 90268f77…、checker_known_bad_images.rs 748807f7…、mutations.tsv 124fb019…）。
- kb：`.claude/kb/invariants.md` 的 sha256 c88645bd… 与 `kb-sha256.txt` 一致（该文件在工作区另有未提交改动，本报告引的是与快照同 sha 的版本）。
- 条款：I-9.2 在 `.claude/kb/invariants.md:289`、I-9.17 在 :304、I-9.23 在 :310。三条都有同一小句，且都以「其余判该条目损坏」收尾：
  - I-9.2：「子指针头部的出生树 == 身份引用的出生树」（`.claude/kb/invariants.md:289`）
  - I-9.17：「子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 5」（`.claude/kb/invariants.md:304`）
  - I-9.23：「子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1」（`.claude/kb/invariants.md:310`）
  - 三条的字面都没写「孩子读不出时怎么判」；判决第三节第 4 条是「写回」，不在第 28 批。
- 判决：第三节第 1 条「读不出孩子时，I-9.2 / I-9.17 / I-9.23 里不看孩子就判得了的小句」（`research/prompts/m3-batch26-code-r1-main-verification.md:38`）；第 2 条「两份都看，有一份自证成另一类就交回它」（`research/prompts/m3-batch26-code-r1-main-verification.md:39`）；第 3 条是三个测试尖度线索（同文件第 40 行）。

## 1 K1：H-B（读不出孩子时照判出生树那一小句）

### 1.1 CPU 三棵树

| 树 | 读不出孩子的那一支 | 判什么 | 与读得出时同一小句的对照 |
|---|---|---|---|
| 墓碑树、谱系树 | `crates/singlefs-checker/src/walk.rs:2379-2396` | `identity_invariant`（墓碑 I-9.23、谱系 I-9.17）判 `child.birth_tree == identity_birth_tree`，`identity_birth_tree` 取身份引用第一段 | 读得出时 `container_entry_holds` 里的同一小句「&& child.birth_tree == identity.0;」（`crates/singlefs-checker/src/walk.rs:2406`） |
| inode 树类型段 2 | 调用点 `crates/singlefs-checker/src/walk.rs:2590-2595`（新多传 `child.birth_tree`），判在 `crates/singlefs-checker/src/walk.rs:4207-4218` | I-9.2 判 `birth_tree_in_the_child_pointer_head == identity_birth_tree` | 读得出时「&& child.birth_tree == identity.0;」（`crates/singlefs-checker/src/walk.rs:2611`） |
| inode 树类型段 0 | `crates/singlefs-checker/src/walk.rs:4219-4227`，本批没改 | 身份三段为 0 | 同上，没改 |

- 判的恰是「子指针头部出生树 == 身份引用出生树」这一小句：容器那支的比较是「child.birth_tree == identity_birth_tree,」（`crates/singlefs-checker/src/walk.rs:2385`），取值「let identity_birth_tree = identity.0;」（`crates/singlefs-checker/src/walk.rs:2382`）；inode 那支同形（`crates/singlefs-checker/src/walk.rs:4208-4211`）。
- 看孩子头的小句照旧不判：容器那支读不出之后只推走读失败「.push(format!("{name}的容器两份都读不到对得上的"));」（`crates/singlefs-checker/src/walk.rs:2394`）；容器头四段、类标签 3、记录类型都在读得出之后的合取里（`crates/singlefs-checker/src/walk.rs:2403-2406`）。inode 类型段 2 那支同样只有一个 `judge`，子头四段与打包记录类型 == 2 没进去（`crates/singlefs-checker/src/walk.rs:4195-4197` 注释与 4207-4218 代码相符）。
- 三棵树记的不变量号：容器那支用传进来的 `identity_invariant`，调用链 `crates/singlefs-checker/src/walk.rs:2252` 起；谱系树落 I-9.17、墓碑树落 I-9.23 由两条坏镜像钉（见 3.1）。

### 1.2 GPU 两支

- inode 根：「if (record_type == 2u) {」下判「w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2664-2666`），在 `s.version == NONE` 那支内（第 2657 行起）；类型段 0 那句三段为 0 没动（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2661-2663`）。
- 容器树：「w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3114`），`rd64(n.base, o + 8u)` 就是 CPU 的 `identity.0`（条目偏移 8 起的身份引用第一段，同文件 3120-3122 的读得出那支用同一个读法）。`identity_invariant` 由记录类型定（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3018-3021`）。
- 孩子指针出生树 CPU 在指针偏移 34（`crates/singlefs-checker/src/image.rs:1349`「birth_tree: read_u64(bytes, 34),」）；GPU 的 `child.birth_tree` 与读得出那支同名同用。

### 1.3 结论与「会推翻它」

- 一致：三棵树与 GPU 两支判的就是 I-9.2 / I-9.17 / I-9.23 里只比父条目两段的那一小句；看孩子头的小句照旧不判；类型段 0 的身份三段为 0 照旧。
- 为什么合法状态上不会因这一句误红（推的，读码）：这一小句只看父条目自己的两段字节，读得出孩子的同一条目上同一小句本来就每个合法状态都判（`crates/singlefs-checker/src/walk.rs:2406`、`:2611`）；读不出孩子不改变父条目的字节。实测旁证：合法流上 9 个 harness 测试二进制 108 条全绿（5.3），三次从空目录跑一致。
- 推翻条件：合法写者写出的父条目里，子指针出生树与身份出生树不等（kb I-9.2 字面要求相等）；或者有合法崩溃状态里父条目指向的孩子读不出、而父条目的两段本来就不等。这两样 K2 攻方在合法流上扫，我这里只有 5.3 那一批旁证。

## 2 K1：H-V（认孩子的类，两份都看）

### 2.1 CPU 与 GPU 的写法

- CPU：`crates/singlefs-checker/src/walk.rs:4161-4190`。累加器「let mut class_other_than_the_entry_type_names = None;」（`crates/singlefs-checker/src/walk.rs:4166`），循环过两条位置条目（`child.locations` 恰两条：`crates/singlefs-checker/src/image.rs:1283`「pub locations: [PointerLocation; 2],」）；一份要先读得出头、类标签登记过、按那一类长度的整单元校验和对得上才算自证（第 4170、4174、4180 行三个 `continue`），自证过的类若不是类型段说的、且还没记下别类，才记下（`crates/singlefs-checker/src/walk.rs:4183-4187`）；循环走完交回累加器（第 4189 行）。
- GPU：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2986-3007`，先第一份后第二份，同一套自证（`w_unit_verifies_at_the_length_of_its_class`，第 2969 行起，本批没改），第二份只在累加器还是 NONE 时才记。
- 次序：CPU 与 GPU 都是位置条目 0 在前（CPU 位置条目从指针偏移 50 起、64 起：`crates/singlefs-checker/src/image.rs:1351`；GPU 的 `device0` 取偏移 50：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:545`）。

### 2.2 真值表（不碰盘，全部状态）

`research/prompts/m3-batch28-code-r1-sonnet-model/h-v-truth-table.py`：一份副本的状态 = 读不出头，或（类标签 0–9 × 自证过与否）；类型段说的类取 None、1、2、3；两份副本全部组合共 1764 个。三次复跑输出相同：

```
状态总数 1764
CPU新 与 GPU新 不同的状态数 0
CPU新 与 按条款写的规格 不同的状态数 0
旧写法与新写法交回不同的状态按形分：
   ('first=named+v', 'second verifies other') 旧→新 [(None, 1), (None, 2), (None, 3)]
变异写法与新写法交回不同的状态数（0 = 在全部状态上等价）：
   cpu_old_form 0
   cpu_row_2930 12
   gpu_old_form 0
   gpu_row_2936 12
```

- 「按条款写的规格」= 任一份自证过且类 ≠ 类型段说的就交回它（位置条目在前的先）。这是判决第三节第 2 条的字面：「两份都看，有一份自证成另一类就交回它」（`research/prompts/m3-batch26-code-r1-main-verification.md:39`）。CPU 新写法与它在全部状态上同值。
- CPU 新与 GPU 新（照 wgsl 逐行抄进脚本）在全部状态上同值，包括「两份各自证成两个不同的别类」（都交回位置条目在前那份的）。
- 旧写法（在第一份自证过的副本处停）与新写法不同的状态，只有一形：第一份自证成类型段说的那一类、第二份自证成别类，旧交回 None、新交回那一类。别的状态一概不变。所以「本来判对的现在判错」在这个抽象里没有：新写法只在这一形上改了结论，而这一形按判决正是要红的。
- 抽象的前提（读码）：结果只取决于每份副本的（读得出头吗、头里的类、按那一类长度自证吗）与类型段说的类，这与 `crates/singlefs-checker/src/walk.rs:4167-4188` 的循环体一一对得上；`MappingKeyUnitClass::of` 把 1、2、3 以外的标签归 `Unregistered`（`crates/singlefs-checker/src/walk.rs:815-823`），它没有长度（`unit_bytes_of_the_class` 交 None：`crates/singlefs-checker/src/walk.rs:4132-4139`）。
- 没判的：两份各自证成不同别类时取哪一类、一份本类一份别类时槽数按哪一类登记——已记 C739：「认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款」（`.claude/kb/checks-owed.md:588`），本轮不判；两边今天的取法相同（真值表）。

### 2.3 调用点

- 墓碑 / 谱系：`crates/singlefs-checker/src/walk.rs:2209`，类型段 0 说码 2、类型段 = 记录类型说码 3，别类就判 `identity_invariant` 红并 `continue`（第 2214-2222 行）。
- inode：`crates/singlefs-checker/src/walk.rs:2528`，别类判「self.judgements.judge("I-9.2", false, || {」（`crates/singlefs-checker/src/walk.rs:2541`）并 `continue`。
- GPU：inode 根 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2637`，容器树两处 3057（类型段 0）、3103（类型段 = 记录类型）。

### 2.4 结论与「会推翻它」

- 一致：两份都看的次序、长度（按自证那一类的长度）、两份都自证成本类 / 一份本类一份别类 / 两份各是别类时交回什么，CPU 与 GPU 同值，且与判决字面同值（真值表三次相同）。
- 推翻条件：真值表抽象漏了一个状态维度（例如 GPU `read_at` 读 16 KiB 头成功而 32 KiB 校验和槽位判定与 CPU 的整单元读不同：GPU 那一支 `slice_crc_matches` 在 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:574-582`，本批没改，我读了没发现差异，但没有 GPU 实跑）；或者对拍流上出现 CPU 与 GPU 不同的状态（提交时 GPU 对拍，我没跑）。

## 3 K1：T1–T3、13 条坏镜像、4 条对拍、变异行

### 3.1 13 条坏镜像实跑（`checker_known_bad_images`，harness 档）

新代码上 19 条（13 条新增 + 第 26 批 6 条陈旧副本）`19 ok / 0 FAILED`；把本批对 walk.rs 的改动反向打回去（`patch -R`，`research/prompts/m3-batch28-code-r1-sonnet-model/walk-rs-of-batch28.patch`）再跑同样 19 条：`10 ok / 9 FAILED`。那 9 条红的恰是 3 条 H-B 一格加一（inode、墓碑、谱系各一）与 6 条 H-V；T2 三条、T3 一条与第 26 批 6 条在旧码上也绿（它们钉的是本来就该成立的行为，见 3.3）。

| 组 | 条数 | 测试 | 期望写了什么 |
|---|---|---|---|
| H-B 一格加一 | 3 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:11158`（inode，身份出生树加 1）、`:11170`（墓碑，子指针出生树加 1）、`:11184`（谱系，身份出生树加 1） | 共用判法 `:11110` 起：先对照「对照：条目不改、容器两份都读不出，{entry_invariant} 不红：{:?}」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:11124`），再加一后红、说明含「两份都读不出」与两段出生树的值 |
| T3 不过判 | 1 | `crates/singlefs-harness/tests/checker_known_bad_images.rs:11202` | I-2.1 红（钉孩子真读不出）、I-9.2 不红 |
| T2 只改身份一段 | 3 | `:11050`、`:11060`、`:11070`，共用 `:11017` 起 | 类型段 0 身份只一段为 1：I-9.2 红、说明含「孩子两份都读不出」与 `{expected_three_fields}（要三段为 0）` |
| H-V 两份都看 | 6 | `:11304`、`:11316`（inode：条目不改 / 改成类型段 0）、`:11335`、`:11349`（墓碑）、`:11370`、`:11385`（谱系），共用 `:11224` 造镜像、`:11263` 判 | 两种盘序（码 2 那一份在位置条目 0 / 1）各判一次：条目不变量红、说明点名孩子头里的类（`IndexNode` 或 `PackedRecordUnit`），两种盘序红的同一批 |

### 3.2 T1：GPU 第二份副本那一支的对拍与变异（GPU 不跑，读码）

- 对拍：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7600`（盘 1 那份换成码 2 节点字节、位置条目对得上它，期望 `["I-9.2"]`）与 `:7616`（位置条目不改，第二份自证不过，断言 CPU 红 I-2.1、不红 I-9.2，且逐状态不一致为空：「!comparison.cpu_checker_red_invariants.contains("I-9.2"),」在 `:7635`）。
- 变异：`crates/mutations.tsv:2975`（第一份自证过就 `return`，叶容器盘 1 那份自证成码 2 那条流上 CPU 红 I-9.2、GPU 不红）与 `:2976`（第二份副本不自证、只认头里的类）。锚点在 wgsl 里各恰好命中 1 次（我用脚本数了 2936、2973–2976 五行，都是 1 次、替换文与锚点不同）；是否红要 GPU 对拍，我没跑。
- 判断：`:7616` 钉的正是「第二份只认头里的类」这一格（第 26 批线索①）；`:7600` 钉「第二份自证得过」那一侧。H-B 两条对拍（`:7469` 期望 `["I-9.2", "I-2.1"]`、`:7510` 期望 `["I-9.23", "I-2.1"]`）逐位比 CPU 与 GPU。

### 3.3 变异行（CPU 全跑，GPU 4 行不跑）

`mutrun.py` 把每行的锚点施加（先数锚点命中，≠1 记 ANCHOR）、编、跑点名那几条测试、还原；每一行都先在还原后的原树上跑过基线（全绿）。从空目录跑了三次（`rerun.sh`，细节见 5.2）：前两次 summary 逐行相同（72 行），第三次在其上多 9 条合法流 LEGIT 行与 X11–X14 四行，其余逐行相同。

| 行 | 内容 | 结局（RED = 点名测试红了；第三列是红了几条测试） |
|---|---|---|
| `crates/mutations.tsv:2883`（替换，锚点断） | 孩子的类与类型段不符也当相符，改成 `false && …` | RED，点名测试红，1 条 |
| `:2930`–`:2933`（替换，4 行） | 头里的类就是类型段那一类时，在验校验和之前 `break` | RED，点名测试红，各 4 条 |
| `:2934`（替换，锚点断） | inode 读不出孩子时整个不判 | RED，点名测试红，4 条 |
| `:2935`（第 27 批行，一并跑） | 类型段不认得那支按 16 KiB 读 | RED，点名测试红，1 条 |
| `:2960`、`:2961` | inode H-B 不判 / T3 过判 | RED，点名测试红，1、2 条 |
| `:2962`、`:2963` | 容器树读不出孩子时恒成立（墓碑、谱系两个点名） | RED，点名测试红，各 2 条 |
| `:2964`–`:2966` | T2 三行：身份三段各一段不判 | RED，点名测试红，各 1 条 |
| `:2967`–`:2972` | H-V 在第一份自证过的副本处停（三棵树 × 条目不改 / 改成类型段 0） | RED，点名测试红，各 6 条 |
| `:2973`–`:2976` | GPU 四行 | SKIP（要 GPU） |

- 替换 7 行里 4 行「等价变异」：旧写法「if !verifies_at_the_length_of_its_class\n && Some(class_in_the_header) != class_the_entry_type_names」搬到新代码上（我的额外行 X1）19 条全绿（GREEN），真值表上与新写法在 1764 个状态上有 0 个不同（`cpu_old_form 0`）：按 `.claude/rules/mutation-sampling.md` 的判据是「在所有输入上同值」的等价（理由：同类副本不验校验和只改「同类时是否 continue」，而同类时下一句 `Some(class) != named` 本来就是假，所以什么都不改，读码即得，真值表是穷举佐证），不是取样点不敏感。
- 换上的写法真改行为：`:2930`–`:2933` 新文「在验校验和之前，头里的类 == 类型段那一类且还没记下别类就 break」，真值表上与新写法在 12 个状态不同（`cpu_row_2930 12`），实跑点名测试红 4 条。
- 额外变异（我加的，`research/prompts/m3-batch28-code-r1-sonnet-model/extra-mutations.tsv`，14 行）：

| 行 | 改什么 | 点名那 19 条 / 整份 149 条 |
|---|---|---|
| X1 | 旧式变异搬到新写法（等价） | GREEN（19 条） |
| X2、X11 | inode 读不出孩子时类型段 2 的比较 `==` 改 `>=` | **GREEN（19 条）；X11 整份 149 条也 GREEN** |
| X3 | 同上改 `<=` | RED 1 条 |
| X4、X5 | 容器树读不出孩子时 `>=`、`<=` | RED 各 1 条 |
| X6、X7 | 拿身份引用的容器号当出生树比（inode、容器树） | RED 各 2 条 |
| X8 | 容器树读不出孩子时一律记 I-9.23 | RED 1 条 |
| X9 | 认孩子的类不要求自证 | RED 4 条 |
| X10 | 容器树读不出孩子时那一小句恒判红（过判；本批变异表只有「恒成立」两行，没有这一向） | RED 2 条 |
| X12 | 认孩子的类：校验和对不上就 `break`（不再看下一份） | RED 4 条（整份 149 条） |
| **X13** | 认孩子的类：头里的类标签不在登记表里就 `break` | **GREEN（整份 149 条）** |
| **X14** | 认孩子的类：这一份头读不出就 `break` | **GREEN（整份 149 条）** |

### 3.4 缺口（本轮 K1 的发现，正推到的、没有测试盯着的格）

1. **X2 / X11：inode 读不出孩子、「子指针头部出生树比身份出生树大」那个方向没有测试。** 机理（读码）：inode H-B 那条坏镜像只把身份出生树加 1（`crates/singlefs-harness/tests/checker_known_bad_images.rs:11158`，GPU 对拍 `:7469` 的流也一样），于是子指针出生树小于身份出生树；把比较改成 `>=` 时这一形仍判红、所有现有测试都看不出。容器树两个方向各有一条（墓碑加子指针一侧、谱系加身份一侧），所以 X4（`>=`）、X5（`<=`）都红；inode 只有一个方向，X2 活下来、整份 149 条坏镜像也抓不到（X11）。这是取样点不敏感（换一个取样点——子指针出生树加 1——它就不同值），不是等价变异。补法：inode 那条再造「子指针出生树加 1」一形，加一行 `>=` 变异。
2. **X13 / X14：认孩子的类时「第一份头的类标签不在登记表里 / 头读不出、第二份自证成别类」没有测试。** 这两个 `continue` 是旧码就有的结构，但本批把「读到第一份自证过就停」改成「两份都看」之后，第二份才是被看的对象，这一形成了新行为的一部分：改成 `break` 后，第一份坏、第二份是别类的状态会漏判；整份 149 条坏镜像全绿。真值表（H-V 状态里 `(类 7, 不自证)` 加 `(类 2, 自证)` 之类）上两种写法不同，所以不是等价变异。X12（校验和对不上）有 4 条抓住，这一格只缺类标签坏、头读不出两形。
3. 第 28 批没补、已记 C739 的两个状态（两份各是不同别类；一份本类一份别类的槽数）没有坏镜像，本轮不判。
4. 推翻条件：补上对应一形的测试后 X2/X11、X13、X14 变红；或者找到一条现有测试（本轮之外的整个 harness）在这三个变异下红——我只跑了 `checker_known_bad_images` 整份 149 条，没有跑全 harness（那一档太长，且 X2 的变异只落在 inode 读不出孩子那支，别的二进制要造到这支才会红）。

### 3.5 合法流上不误红的旁证（5.3 的 `legit` 单元）

合法流（新池建文件、删除、回退、容器树长高）上 9 个 harness 测试二进制整份跑，108 条全绿：container_trees_grow_a_level_when_the_root_is_full 6、deletion_scenario_writes 5、new_pool_file_creation_publish 9、pool_checker_judges_the_deletion_state_of_every_walked_version 45、rollback_lineage_record_and_its_checkpoint_cost_term 4、test_only_switches_force_every_branch 6、unlink_mount_continuation_and_the_emptied_version 23、unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches 1、unlink_single_file_publishes 9。这是旁证，不是对 H-B、H-V 在所有合法崩溃状态上不误红的证明（那是 K2 与提交时层 0 的活）。

## 4 K3：被判的 5 份文件，每份一格

| # | 文件（冻结副本） | 判定 | 一句话 |
|---|---|---|---|
| 1 | `crates/singlefs-checker/src/walk.rs` | 一致 | H-B 三处（`:2379-2396`、`:2590-2595`、`:4200-4229`）判的是条款只比父条目两段的那一小句；H-V `:4161-4190` 与条款字面同值，真值表 1764 态；没找到冲突 |
| 2 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | 一致（GPU 不跑） | H-B 两支（`:2664-2666`、`:3114`）与 H-V（`:2986-3007`）逐行对 CPU，H-V 真值表 CPU 与 GPU 抄写版 0 个不同 |
| 3 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 一致（GPU 不跑） | 4 条对拍（`:7469`、`:7510`、`:7600`、`:7616`）的期望与 CPU 一侧读码相符；对拍本身是否绿复核不了 |
| 4 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 一致，另有缺口 | 13 条实跑绿、旧码上恰 9 条红；缺口见 3.4（X2/X11、X13/X14 活下来）|
| 5 | `crates/mutations.tsv` | 一致 | 17 条新行中 13 条 CPU 行实跑都红且点名测试红；替换 7 行的口径对（4 行等价、新文真改行为）；4 条 GPU 行没跑 |

### 4.1 一处冲突（两边整行并排）

- 判据一侧：`.claude/rules/mutation-sampling.md:87`「一条变异没被抓时，问一句：换一个取样点，它还同值吗？」；任务一侧：`research/prompts/_m3-batch28-code-r1-body.md:33`「13 条坏镜像与 4 条对拍的期望写没写全」。
- 实现一侧：`crates/singlefs-harness/tests/checker_known_bad_images.rs:11164`「INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,」——inode 那条只给身份出生树加 1；`>=` 变异 X2 / X11 在这张表之外的整份 149 条里都不红。换一个取样点（子指针出生树加 1）就不同值：取样点不敏感，期望没写全。
- 推翻条件：补一条「inode 子指针出生树加 1」的坏镜像后 X2/X11 变红则缺口是真的；若能证明 inode 那支的 `>=` 与 `==` 在某个合法前提下同值（要证子指针出生树恒不大于身份出生树），则缺口不成立，但条款（I-9.2 字面「==」）没有这个前提。

## 5 模型与复跑

### 5.1 目录与用法

`research/prompts/m3-batch28-code-r1-sonnet-model/`：`rerun.sh`、`mutrun.py`、`h-v-truth-table.py`、`extra-mutations.tsv`、`walk-rs-of-batch28.patch`（本批对 walk.rs 的改动原样，取自 `_m3-batch28-code-r1-diff.md` 第 6–124 行，反向打回去用）。

```
SINGLEFS_REPO=<冻结副本树根> [RERUN_BUDGET_SECONDS=<预算秒，默认 0 = 不限>] [RERUN_THREADS=15] \
  [RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<同会话另有一条在跑时的理由>] bash rerun.sh <目录> all
bash rerun.sh compare <summary-a> <summary-b>
```

一次只起一条 cargo；一份树副本、一个编译目录，都在 `<目录>` 里；编译 `run-with-memory-cap.sh 6G`、跑 `4G`、`capped.sh <线程>`、`nice -n 19`。单元次序 truth、new、old、rows、extra、legit，每个单元（变异按行）完成写标记，预算用完在边界退 75，再跑同一条命令续（前台命令不超过 240 秒，所以我分段跑）。

### 5.2 跑了什么、结果

- 第一次跑（`/tmp/claude-1000/m3-batch28-code-r1-sonnet/run1`，已删树与编译目录，留 `run1-summary-prefix.txt`）与第二次（`run2-summary-prefix.txt`）summary 逐行相同：`bash rerun.sh compare` 输出「compare: 两份 summary 排序后逐行相同（72 行）」。
- 第一次跑发现一个脚本毛病：还原 walk.rs 用 `cp -a` 会带回旧 mtime，cargo 把变异那一版的产物当成现成，基线格出现「RED rc=101 13 passed 6 failed」。改成新 mtime 的拷贝后重跑（第二、第三次）；变异行本身每次写入新内容、不受影响，所以第一次的 72 行与第二次相同。
- 第三次（`run3`，从空目录、分 12 段）最终汇总 85 行，在 72 行之上多 LEGIT 9 行与 X11–X14 四行，其余逐行相同。收尾输出：`SUITE new: 19 ok / 0 FAILED；old: 10 ok / 9 FAILED`，基线格六条全 ok（含整份 `149 passed; 0 failed`）。
- 复跑中途主 agent 把线程上限从 10 改成 15：第三次前 7 段用 10、后 5 段用 15；`rerun.sh` 现在读 `RERUN_THREADS`（默认 15）。结果与线程数无关，但最终这版脚本没有再从空目录整条跑一遍。
- 同会话主 agent 一直另有 cargo 在跑，我的每条都设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（会话之间的并行，我自己的一次只起一条）；整机负载 35–106，分段耗时被拉长到 2–5 分钟一段；包装退出码 250–254 一次都没出现（脚本遇到会退 4 停下）。

### 5.3 `legit` 单元

见 3.5：9 个 harness 测试二进制整份跑；每个二进制「test result: ok. N passed; 0 failed」记一行 LEGIT。

## 6 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 H-B CPU 三棵树 | 一致 | 读不出孩子时判的就是条款里只比父条目两段的那一小句，看孩子头的小句仍不判（`walk.rs:2379-2396`、`:4200-4229`）；X3–X8、X10 与 `:2960`–`:2966` 实跑都红 |
| K1 H-B GPU 两支 | 一致（读码） | wgsl `:2664-2666`、`:3114` 与 CPU 同取值同判；对拍与四条 GPU 变异我没跑 |
| K1 H-V CPU / GPU | 一致 | 真值表 1764 态：CPU 新 = GPU 新 = 按判决字面写的规格；旧新只在「一份本类自证、另一份别类自证」一形上不同；`:2967`–`:2972` 实跑红 |
| K1 H-V 两份各是不同别类 / 一份本类一份别类的槽数 | 规则没说 | 已记 C739，今天两边同取位置在前那份；本轮不判 |
| K1 替换 7 行 | 一致 | 4 行旧写法是真等价（X1 绿、真值表 0/1764），新文真改行为（12 态不同、点名 4 条红）；2 行锚点断的新锚点命中 1 次且 RED；GPU 1 行锚点命中 1 次，没跑 |
| K1 T1（GPU 第二份副本对拍与变异） | 一致（读码） | `:7600`、`:7616` 与 `mutations.tsv:2975`、`:2976`；GPU 没跑，复核不了 |
| K1 T2（只改身份一段三形） | 一致 | 三条实跑绿、旧码也绿（钉既有行为）；`:2964`–`:2966` 各红 1 条 |
| K1 T3（类型段 2 读不出孩子不过判） | 一致 | `:11202` 实跑绿；`:2961`（恒判红）RED 2 条；容器树的不过判由 `:11124` 对照与 X10 钉 |
| K1 13 条坏镜像 / 4 条对拍的期望写没写全 | 冲突 | inode 读不出孩子那条只有「身份出生树加 1」一个方向：`>=` 变异 X2/X11 整份 149 条都不红；第一份头类标签不登记 / 头读不出而第二份别类（X13/X14）也没测试；见 3.4、4.1 |
| K3 walk.rs | 一致 | 同 K1 |
| K3 crash_judge_checker.wgsl | 一致（GPU 不跑） | 同 K1 |
| K3 GPU 对拍测试文件 | 一致（GPU 不跑） | 4 条的期望读码相符，实绿复核不了 |
| K3 checker_known_bad_images.rs | 一致，附缺口 | 13 条绿；缺口见 3.4 |
| K3 mutations.tsv | 一致 | 17 条新行 CPU 13 条全红，GPU 4 条未跑；表里缺对应 3.4 缺口的行 |

## 7 没做什么

- 不判别的腿的格（K2 攻方格：误红、漏判、CPU 与 GPU 不同的新形，上一轮「没打中」的第二次抽样）；不替主 agent 采纳、不出判决。
- GPU 一概没跑：4 条对拍与 GPU 4 行变异、`slice_crc_matches` 在 GPU 上与 CPU 整单元读的一致性，都只读码；提交时由主 agent 跑。wgsl 变异替换文是否编得过（第八类「无效」）也没验：我查到本机 registry 里有 naga 30.0.1，但没起静态校验，这一项留给 GPU 对拍。
- 不重型测试：没跑全 harness、没跑 checker 档、没跑 `crates` 变异整表；`checker_known_bad_images` 之外的 harness 只跑了 3.5 那 9 个二进制。
- 没写回 kb、没改任何 `crates/` 或 `research/prompts/` 里别人的文件；模型目录与本报告是我唯一写的仓内路径。
- 第三次复跑中途线程上限从 10 改成 15，最终这版 `rerun.sh` 没有再整条从空目录跑一遍（见 5.2）。
- 草稿与编译目录：`/tmp/claude-1000/m3-batch28-code-r1-sonnet/`（含 run3 的树副本与编译目录，交回前删；留各段日志与 summary）。
- 模型目录里另存本次第三次复跑的 `summary-run3.txt`（85 行）与 `truth-table-run3.txt`，供 `rerun.sh compare` 与主 agent 复跑比对；它们是 `/tmp/claude-1000/m3-batch28-code-r1-sonnet/run3/` 里同名文件的拷贝。
