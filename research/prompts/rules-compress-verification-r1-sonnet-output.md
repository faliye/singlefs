# rules-compress-verification-r1 正推腿报告

日期：2026-11-16。轮：rules-compress-verification-r1，第一轮，正推腿（逐条核落点）。背景材料：`research/prompts/_rules-compress-verification-r1-body.md`。

这一条腿没写模型，没有复跑命令。行号都是用草稿目录里的辅助脚本在被引文件里按原文片段逐字查出来的（片段命中 0 次的不出表；辅助脚本与规格在 `~/.local/share/singlefs/work/rules-compress-verification-r1-sonnet/`）。合稿树照改名后的路径写：仓里的 `.claude/` 在树里是 `dot-claude/`。

被判文件此刻与快照相同（量的，这一次跑的原样输出）：

```
$ sha256sum -c research/prompts/rules-compress-verification-r1-snapshot/judged-sha256.txt
.claude/rules/verification.md: OK
.claude/rules/implementation-workflow.md: OK
.claude/rules/three-way-inference.md: OK
```

合稿树五份的 sha256（量的）与 `research/prompts/rules-compress-verification-r1-snapshot/merged-tree-sha256.txt` 里的同名行逐字相同：

```
2952a79101179033c2a8fe1c6c5b3e526a9179add633264196246c2a1567aef3  dot-claude/rules/verification.md
79f389ff6dd0ac81c88355773f627de385f32b37770ed09f9bca668161497f79  dot-claude/rules/implementation-workflow.md
007db7ea2aafa8310cbd48e7d5a66fd340431a4e6a831c2012701e86fcb1cb95  dot-claude/rules/three-way-inference.md
e0a04dc60f1f187748e6bb477b47e70c5bbf98b3798f6145c4d995bd03391527  dot-claude/rules/verification-reference.md
6d3735f0b92133b7368e113f56901a95e06d6462931bc905f5abf4a1bb196cca  dot-claude/skills/three-way/SKILL.md
```

## 一、逐条落点表

读法：一行是原件里的一条（或紧挨着的一组）规则。「原件」一栏是原件的 文件:行 与那一条开头的一段原话；「合稿树落点」是它在合稿树（或合稿指过去的脚本文件头）里的 文件:行 与落点那一句里的一段原话，几处落点用分号隔开。原件一行里带好几条细则的（`verification.md` 第 39、40、55、126 行这类），按细则拆成几行。备注里写「矛盾表」「用户定案」的，是做法有改动而已登记的；写「见格几」的在第三节展开。

### `.claude/rules/verification.md`（第 1 到 61 行）

| 号 | 原件 | 合稿树落点 | 备注 |
|---|---|---|---|
| V1 | `.claude/rules/verification.md:8`：「四个包，四个名字」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:5`：「四个包，全仓只用这四个名字称呼它们」 |  |
| V2 | `.claude/rules/verification.md:12`：「单元测试与集成测试，加上它们用的脚手架库」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:9`：「验证包的模块：录制器、内存池」 | 模块清单挪参考件 |
| V3 | `.claude/rules/verification.md:12`：「连同 QEMU（门禁 qemu-device-streams）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:10`：「验证包连同这几道门禁判的东西」 |  |
| V4 | `.claude/rules/verification.md:12`：「指定范围的主 agent 随时经任务服务交」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「提交时由主 agent 经任务服务交」 | 什么时候跑并进两种跑法那张表 |
| V5 | `.claude/rules/verification.md:13`：「判一个镜像的判决器库」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:10`：「判一个镜像的判决器库」 |  |
| V6 | `.claude/rules/verification.md:14`：「语义级的参照文件系统」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:11`：「语义级的参照文件系统」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:153`：「⑤⑥⑦ 判 RefFS 那三样」 | 用户定案 9 |
| V7 | `.claude/rules/verification.md:15`：「归测试基础设施、验证包依赖的一侧」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:12`：「归测试基础设施，不是产品代码」 | 少了半句说明 |
| V8 | `.claude/rules/verification.md:17`：「测试文件按它测什么起名」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:16`：「测试文件按它测什么起名，领域在前」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:152`：「test-file-names 格在真仓上跑」 |  |
| V9 | `.claude/rules/verification.md:18`：「不许用的叫法」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:17`：「不许用的叫法」 |  |
| V10 | `.claude/rules/verification.md:22`：「验证包的测试不分档，不带参数就是全部」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:21`：「验证包的测试不分档」；`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`：「内存上限、机器、卡、GPU 清场、排队次序」 | 谁交、怎么交指到共用约束 |
| V11 | `.claude/rules/verification.md:23`：「红了只重跑红的那几条」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:22`：「红了只重跑红的那几条」；`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`：「原样交、不再插」 |  |
| V12 | `.claude/rules/verification.md:24`：「调崩溃枚举函数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:23`：「调崩溃枚举函数」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:151`：「placement 那一样判写在别的包里」 |  |
| V13 | `.claude/rules/verification.md:25`：「一条用例一个场景照」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:24`：「本仓把每一轮新建一个池」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:155`：「的 one-scenario 格（在真仓上跑」 |  |
| V14 | `.claude/rules/verification.md:29`：「门禁只有一个，跑法两种：不指定范围」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:28`：「门禁只有一个，跑法两种，没有分档」 |  |
| V15 | `.claude/rules/verification.md:33`：「门禁阶段不带 `--check` 与 `--item`；」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「门禁阶段不带 `--check` 与 `--item`；」 |  |
| V16 | `.claude/rules/verification.md:34`：「（可给几次，`--list-items` 列可点的项）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:33`：「范围声明种类 `items` 或 `target`」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:130`：「只跑点名的项，给几次取并集」 |  |
| V17 | `.claude/rules/verification.md:36`：「判哪些、复用哪些由门禁照单元键自己选」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:35`：「判哪些、复用哪些由门禁照每条判定的单元键自己选」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:185`：「文件级排除：」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:186`：「边级排除：」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:187`：「python 模块有单元进闭包时」 | 两种排除只在参考件 |
| V18 | `.claude/rules/verification.md:37`：「判法运行期按字面尾巴整组收的数据文件」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:188`：「判法运行期按字面尾巴整组收的数据文件」 |  |
| V19 | `.claude/rules/verification.md:38`：「缩域：崩溃域只往小缩」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:38`：「崩溃域只往小缩」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:50`：「的是崩溃点重放入口、崩溃放量入口与」 |  |
| V20 | `.claude/rules/verification.md:39`：「不指定范围那一趟经服务交，服务照它文件头」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:18`：「经服务交的每一种跑法（不带参数」 | 参考件照门禁文件头写成每一种跑法 |
| V21 | `.claude/rules/verification.md:39`：「各格判什么：crash-case-replay 格起」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:19`：「crash-case-replay 格起」 |  |
| V22 | `.claude/rules/verification.md:39`：「feature-gated-tests 格再一趟」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:20`：「feature-gated-tests 格再起一趟」 |  |
| V23 | `.claude/rules/verification.md:39`：「挂特性的测试文件跑到加复用 0 条的」 | `.claude/gate.d/crash-point-replay.sh:249`：「挂特性的文件跑到加复用 0 条」 | 只剩门禁文件头 |
| V24 | `.claude/rules/verification.md:39`：「另有 product-gpu-real-card-tests 一格」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:146`：「的真卡对拍判红整格红」；`.claude/gate.d/crash-point-replay.sh:294`：「一个对拍目标跑到加复用 0 条也报」 | 本次未跑两种只剩门禁文件头 |
| V25 | `.claude/rules/verification.md:39`：「要上卡的三处」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:21`：「要上卡的三处」；`.claude/gate.d/crash-point-replay.sh:276`：「SINGLEFS_CRASH_AMPLIFICATION_STAGE=record、摘掉」 | 矛盾表 验证-1 |
| V26 | `.claude/rules/verification.md:39`：「领卡被拒的那一格判红、那一趟不起」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:21`：「领卡被拒的那一格判红、那一趟不起」 |  |
| V27 | `.claude/rules/verification.md:39`：「逐条按崩溃状态块跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:146`：「的崩溃状态块，判器结论与挂在这个点上的观察者结论」 |  |
| V28 | `.claude/rules/verification.md:39`：「跑完打 `CRASH_CASE_BLOCKS` 一行」 | `.claude/gate.d/crash-point-replay.sh:117`：「CRASH_CASE_BLOCKS blocks=<块数>」 | 只剩门禁文件头 |
| V29 | `.claude/rules/verification.md:39`：「的用例（故障注入之后的红由用例自己认下）」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:146`：「的退出码判、块那一行只核」 | 库里的红块不判一句没抄 |
| V30 | `.claude/rules/verification.md:39`：「按条复用那几趟里每条测试函数各是一条判定」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:42`：「按条复用那几趟每条测试函数一条」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:36`：「判红的照同一把键记红、撤掉这把键原有的绿，下一趟现判」；`.claude/gate.d/crash-point-replay.sh:90`：「复用：按条复用那几趟里每条测试函数各是一条判定」 |  |
| V31 | `.claude/rules/verification.md:39`：「崩溃放量的判器整套上了 GPU」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:30`：「判器整套上了 GPU」 |  |
| V32 | `.claude/rules/verification.md:40`：「不带参数跑登记表里全部分项」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:28`：「不带参数跑登记表里全部分项」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:29`：「退 248」 |  |
| V33 | `.claude/rules/verification.md:40`：「每块的键是块键」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:36`：「每块的键是块键」 |  |
| V34 | `.claude/rules/verification.md:42`：「各按判定入库：每条判定的键是它的单元键」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:42`：「按判定入库的单位：」 |  |
| V35 | `.claude/rules/verification.md:43`：「只读文本的静态检查格之外，凡起编译」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:36`：「按判定入库：凡起编译」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:44`：「prune-server 自己的自证是门禁 code-tooling」 |  |
| V36 | `.claude/rules/verification.md:44`：「判红的照同一把键记红、撤掉这把键原有的绿，键记着红的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:43`：「判红照同一把键记红、撤掉原有的绿」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:36`：「broad-mutation 的驱动只算翻面」 |  |
| V37 | `.claude/rules/verification.md:44`：「各道的库名是被判那棵树的 git common-dir」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:100`：「各道的库名是被判那棵树的 git common-dir」 |  |
| V38 | `.claude/rules/verification.md:46`：「崩溃判定库的键不读仓根」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:35`：「崩溃判定库的键不读仓根」 |  |
| V39 | `.claude/rules/verification.md:48`：「交；内存上限、哪台机器、哪几张卡、GPU 清场与排队都由服务定」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:42`：「一切跑项目代码的命令只认经」；`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:56`：「内存上限、机器、卡、GPU 清场、排队次序」 |  |
| V40 | `.claude/rules/verification.md:49`：「门禁红了必须修好或查清原因」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:40`：「门禁红了必须修好或查清原因」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:22`：「要知道一趟有多少条要现判：门禁」 |  |
| V41 | `.claude/rules/verification.md:50`：「提交时只走一条路」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:41`：「提交时只走一条路」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:149`：「不指定范围那一趟这一格自己先起驱动剪枝跑」 |  |
| V42 | `.claude/rules/verification.md:51`：「谁跑得了由」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:57`：「谁跑得了由」 |  |
| V43 | `.claude/rules/verification.md:52`：「起任何验证（崩溃点重放与崩溃放量入口」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:42`：「起任何验证的命令开头带范围声明」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:58`：「范围声明四段的写法」 |  |
| V44 | `.claude/rules/verification.md:53`：「同一份输入第二趟现判 0」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:64`：「自证里各有一格」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:37`：「每一趟真跑开跑前从键相同」 | 三问指到 test-script-discipline |
| V45 | `.claude/rules/verification.md:55`：「**凡用 GPU 先清场，跑完不复原**，清场由服务做」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:46`：「清场与分卡都由服务做」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:70`：「算在内的：」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:71`：「起跑就分卡的（手跑」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:74`：「要复原，手动跑」 | 矛盾表 验证-6 |
| V46 | `.claude/rules/verification.md:57`：「崩溃状态数压在 20 万量级」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:39`：「崩溃状态数压在 20 万量级」 |  |
| V47 | `.claude/rules/verification.md:58`：「算不出 n 缩到 5 个展开深度」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:51`：「算不出 n 缩到 5 个展开深度」 |  |
| V48 | `.claude/rules/verification.md:59`：「控制测试线程的只有变异那一路」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:81`：「控制测试线程的只有变异那一路」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:82`：「变异那一路：」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:83`：「测试二进制之间一个接一个上卡」 |  |
| V49 | `.claude/rules/verification.md:60`：「GPU 对拍与内存对拍不压并发」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:84`：「GPU 对拍与内存对拍不压并发」 |  |
| V50 | `.claude/rules/verification.md:61`：「派活（哪件上哪张卡）与判法拆开」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:75`：「派活（哪件上哪张卡）与判法拆开」 |  |

### `.claude/rules/verification.md`（第 63 到 146 行）

| 号 | 原件 | 合稿树落点 | 备注 |
|---|---|---|---|
| V51 | `.claude/rules/verification.md:65`：「判定库的内容：剪枝按单元键复用时要读的键」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:50`：「库里只放剪枝按单元键复用时要读的键」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:93`：「文件头登记的 29 张表」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:94`：「Python 一侧的五族」 |  |
| V52 | `.claude/rules/verification.md:65`：「不进库的：运行记录」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:51`：「不进库的：运行记录」；`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`：「两台挂在同一绝对路径，不经」 | 不经家目录取那半句只在共用约束 |
| V53 | `.claude/rules/verification.md:66`：「（服务状态目录下的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:50`：「全仓只有一份判定库」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:92`：「布局只一种」 |  |
| V54 | `.claude/rules/verification.md:67`：「三件按次序做完」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:54`：「三件按次序做完」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:106`：「第三列：写了判定库的行都写 prune-server」 |  |
| V55 | `.claude/rules/verification.md:68`：「那一族接完之后旧库由主 agent 删掉」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:54`：「旧库不原地打开、不并存」 | 由主 agent 删这半句找不到，见「找不到落点的」 |
| V56 | `.claude/rules/verification.md:69`：「新建自己的判定库」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:52`：「新建自己的判定库」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:110`：「中转库的例子」 |  |
| V57 | `.claude/rules/verification.md:70`：「不设开关：读写判定库只有经服务这一条路」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:53`：「不设开关：读写判定库只有经服务这一条路」 |  |
| V58 | `.claude/rules/verification.md:71`：「指定范围与不指定范围是同一套脚本、同一份库」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:28`：「两种跑法是同一套脚本、同一份判定库」 |  |
| V59 | `.claude/rules/verification.md:72`：「欠账 C948」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:55`：「欠账 C948」 |  |
| V60 | `.claude/rules/verification.md:76`：「没有一个任务是必须绑在一台机器上的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:61`：「派到两台里任何一台都要跑得起」 |  |
| V61 | `.claude/rules/verification.md:77`：「没有只派一台的名单」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:61`：「里加只派一台的名单，也不钉台」 |  |
| V62 | `.claude/rules/verification.md:78`：「不读会改变判定结局的本机独有状态；许读」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:62`：「不读会改变判定结局的本机独有状态：许读」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:63`：「只在一台有的，是那条测试的缺陷」 |  |
| V63 | `.claude/rules/verification.md:79`：「欠账 C949」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:64`：「欠账 C949」 |  |
| V64 | `.claude/rules/verification.md:83`：「机器、显卡、显卡数、进程数、线程数、CPU 数这类环境参数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:68`：「机器、显卡、显卡数、进程数、线程数、CPU 数这类环境参数」 |  |
| V65 | `.claude/rules/verification.md:84`：「测试确实要用别的值时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:69`：「测试确实要用别的值时」 | 矛盾表 验证-5 |
| V66 | `.claude/rules/verification.md:85`：「见到脚本新读一个配置文件里没有的环境变量」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:70`：「见到脚本新读一个配置文件里没有的环境变量」 |  |
| V67 | `.claude/rules/verification.md:89`：「每一个词都在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:74`：「每一个词都在」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:152`：「vague-names 格读」 |  |
| V68 | `.claude/rules/verification.md:93`：「把块层所有写请求记下来」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:78`：「把块层所有写请求记下来」 |  |
| V69 | `.claude/rules/verification.md:97`：「在内存里维护一个只管语义」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:82`：「在内存里维护一个只管语义」 |  |
| V70 | `.claude/rules/verification.md:101`：「每加一条，池级 checker 就加一个检查」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:86`：「每加一条，池级 checker 就加一个检查」 |  |
| V71 | `.claude/rules/verification.md:105`：「枚举了多少个崩溃点，是不是全部」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:90`：「枚举了多少个崩溃点，是不是全部」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:91`：「里对应那条的实现状态」 |  |
| V72 | `.claude/rules/verification.md:110`：「每道门禁都能全跑，也能逐格点名跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:95`：「每道门禁都能全跑，也能逐格点名跑」 |  |
| V73 | `.claude/rules/verification.md:112`：「每格一个函数，判一件事。门禁 source 共用库」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:116`：「门禁 source 共用库」 | crash-point-replay 那半句照门禁文件头改了，见格五 |
| V74 | `.claude/rules/verification.md:113`：「文件头写格名表：每格一行」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:117`：「文件头写格名表：每格一行」 |  |
| V75 | `.claude/rules/verification.md:118`：「不跑任何格，也不起任何重活」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:122`：「不跑任何格，也不起任何重活」 | 参数表四行都在 |
| V76 | `.claude/rules/verification.md:124`：「汇总：每格在自己的子 shell 里跑」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:128`：「汇总：每格在自己的子 shell 里跑」；`.claude/gate.d/lib/stage-cells.sh:55`：「跑到的格全退 77 ⇒ 整道退 77」 | 矛盾表 验证-4 |
| V77 | `.claude/rules/verification.md:125`：「样本要证某一格时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:129`：「样本要证某一格时」 |  |
| V78 | `.claude/rules/verification.md:126`：「证明每个有证明义务的单元按全路径」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:138`：「证明的项怎么切」；`scripts/gate/verus-proof-check.py:55`：「核的被验区域里有证明义务的单元各一项」 | 矛盾表 验证-7 |
| V79 | `.claude/rules/verification.md:127`：「顶层每道门禁（共用库在」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:131`：「顶层每道门禁（」 |  |
| V80 | `.claude/rules/verification.md:128`：「阶段归属表」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:132`：「阶段归属表」 |  |
| V81 | `.claude/rules/verification.md:134`：「崩溃点重放跑了、包里全部测试函数按单元键复用或现判都绿」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:146`：「崩溃点重放跑了；包里全部测试函数按单元键复用或现判都绿」 |  |
| V82 | `.claude/rules/verification.md:135`：「形式化验证：恢复重放取前缀、择根、扫环三截的核」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:148`：「形式化验证：恢复重放取前缀、择根、扫环三截的核」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:168`：「verus-proof 起 Verus 之前照信任清单」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:170`：「钉全由 E165 第二段驱动」 |  |
| V83 | `.claude/rules/verification.md:136`：「广谱变异：此刻源码上每一条变异都有单元键相同的结论、没有一条没红」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:149`：「广谱变异：此刻源码上每一条变异都有单元键相同的结论、没有一条没红」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:176`：「等七张表」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:177`：「驱动不给任何一步设内存上限」 |  |
| V84 | `.claude/rules/verification.md:137`：「模型对拍跑了、每段都判过」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:150`：「模型对拍跑了、每段都判过」 |  |
| V85 | `.claude/rules/verification.md:138`：「崩溃枚举用例住在验证包的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:151`：「崩溃枚举用例住在验证包的」 | 声明模块这条判据只在参考件，见格三 |
| V86 | `.claude/rules/verification.md:139`：「函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:152`：「函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名」 |  |
| V87 | `.claude/rules/verification.md:140`：「池级 checker 库不依赖实现；RefFS」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:153`：「池级 checker 库不依赖实现；RefFS」 | 用户定案 9 |
| V88 | `.claude/rules/verification.md:141`：「判器调用点（池级 checker、记录核对器、GPU 判器）落在判定段里」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:154`：「判器调用点（池级 checker、记录核对器、GPU 判器）落在判定段里」；`scripts/crash/cpu-judging-sites-check.py:18`：「包住它的某一个函数（最里层到最外层都算）前面写了标记」 | 标记写法只在参考件与判法脚本，见格三 |
| V89 | `.claude/rules/verification.md:142`：「验证包一条用例一个场景」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:155`：「验证包一条用例一个场景」 |  |
| V90 | `.claude/rules/verification.md:143`：「变异行点名的测试跑得到」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:156`：「变异行点名的测试跑得到」 |  |
| V91 | `.claude/rules/verification.md:144`：「每道阶段认第一个参数当项目根」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:159`：「每道阶段认第一个参数当项目根」 |  |
| V92 | `.claude/rules/verification.md:145`：「跑项目代码的命令经没经任务服务、写法认不认得出」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:160`：「跑项目代码的命令经没经任务服务、写法认不认得出」 |  |
| V93 | `.claude/rules/verification.md:146`：「登记的自证与样本档不起真工具」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:161`：「登记的自证与样本档不起真工具」 |  |

### `.claude/rules/implementation-workflow.md`

| 号 | 原件 | 合稿树落点 | 备注 |
|---|---|---|---|
| I1 | `.claude/rules/implementation-workflow.md:9`：「1 写代码」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:9`：「1 写代码」 | 用户定案 4 |
| I2 | `.claude/rules/implementation-workflow.md:10`：「2 多 agent 正反对抗推理」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:10`：「2 多 agent 正反对抗推理」 | 矛盾表 合稿-1、用户定案 2 |
| I3 | `.claude/rules/implementation-workflow.md:11`：「3 checker 测试」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:11`：「3 checker 测试」 |  |
| I4 | `.claude/rules/implementation-workflow.md:13`：「**次序不能倒**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:13`：「**次序不能倒**」 |  |
| I5 | `.claude/rules/implementation-workflow.md:17`：「**定义是工作流，不是说明**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:17`：「**定义是工作流，不是说明**」 |  |
| I6 | `.claude/rules/implementation-workflow.md:19`：「同规矩**：写完要走一轮三方」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:19`：「同规矩**：写完要走一轮三方」 |  |
| I7 | `.claude/rules/implementation-workflow.md:21`：「**治理文件同规矩**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:21`：「**治理文件同规矩**」 | 矛盾表 合稿-2 |
| I8 | `.claude/rules/implementation-workflow.md:23`：「**三步在定义上各取什么**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:23`：「**三步在定义上各取什么**」 | 矛盾表 合稿-3 |
| I9 | `.claude/rules/implementation-workflow.md:27`：「派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:27`：「派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:71`：「快照放这一轮的材料目录」 |  |
| I10 | `.claude/rules/implementation-workflow.md:29`：「**别的会话改了快照里的文件**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:29`：「**别的会话改了快照里的文件**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:72`：「被别的会话定点替换过的把那几处替换反着做一遍」 | 矛盾表 规则第二遍-1 |
| I11 | `.claude/rules/implementation-workflow.md:33`：「每一轮都按下表八格逐格点名」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:33`：「每一轮都逐格点名八格」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:78`：「与查过的范围」 |  |
| I12 | `.claude/rules/implementation-workflow.md:37`：「打补丁时核过列全」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:82`：「打补丁时核过列全」 |  |
| I13 | `.claude/rules/implementation-workflow.md:38`：「同一个量不同语义不共用裸整数」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:83`：「同一个量不同语义不共用裸整数」 |  |
| I14 | `.claude/rules/implementation-workflow.md:39`：「只为旧数据放宽的读法有没有」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:84`：「只为旧数据放宽的读法有没有」 |  |
| I15 | `.claude/rules/implementation-workflow.md:40`：「改动里新的判定存放处」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:85`：「改动里新的判定存放处」 |  |
| I16 | `.claude/rules/implementation-workflow.md:41`：「读不读会改变判定结局的本机独有状态、写不写死主机名与地址」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:86`：「读不读会改变判定结局的本机独有状态、写不写死主机名与地址」 |  |
| I17 | `.claude/rules/implementation-workflow.md:42`：「新测试与崩溃检测切片并行没有」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:87`：「新测试与崩溃检测切片并行没有」 |  |
| I18 | `.claude/rules/implementation-workflow.md:43`：「新项能被各条的」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:88`：「新项能被各条的」 |  |
| I19 | `.claude/rules/implementation-workflow.md:44`：「比已有的流多罩了哪些崩溃状态」 | `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:89`：「比已有的流多罩了哪些崩溃状态」 |  |
| I20 | `.claude/rules/implementation-workflow.md:48`：「改动之后一条判定要不要重判」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:37`：「改动之后一条判定要不要重判」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:186`：「每一条登记的边配三格自证」 |  |
| I21 | `.claude/rules/implementation-workflow.md:50`：「复用由阶段自己做、自己报」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:38`：「复用由阶段自己做、自己报」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:189`：「作废的判定报得出是哪几个函数单元变了」 |  |
| I22 | `.claude/rules/implementation-workflow.md:52`：「固定在一棵快照树上做，不在活的主工作区量」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:39`：「固定在一棵快照树上做，不在活的主工作区量」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:196`：「两趟都经服务在快照树里交」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:197`：「第二趟有现判的」 |  |
| I23 | `.claude/rules/implementation-workflow.md:57`：「当理由**（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:40`：「当理由**（」 |  |
| I24 | `.claude/rules/implementation-workflow.md:65`：「一条都不交（」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:50`：「一条都不交（」；`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:48`：「红了或证不红的新派实现员」 |  |
| I25 | `.claude/rules/implementation-workflow.md:66`：「不指定范围那一趟：主 agent 照」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:51`：「每次提交代码、提交之外要跑不指定范围的、其余任何时候」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:41`：「提交时只走一条路」 |  |
| I26 | `.claude/rules/implementation-workflow.md:67`：「主 agent 先弹窗问用户，同意了才交」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「提交之外要跑先弹窗问用户」；`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`：「主 agent 自己判要跑的，先弹窗问用户，同意了才跑」 | 两处说法不齐，见格二 |
| I27 | `.claude/rules/implementation-workflow.md:68`：「交，不弹窗；」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:52`：「交，不弹窗；」 |  |
| I28 | `.claude/rules/implementation-workflow.md:69`：「）主 agent 随时经任务服务交；子 agent 都写进交回报告」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:33`：「主 agent 随时经任务服务交，点这一次改到的项、格与目标」 |  |
| I29 | `.claude/rules/implementation-workflow.md:71`：「里面的同样算，命令位置上执行的脚本闸读进去逐行判」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:57`：「里的与命令位置上执行的脚本里的照判」 |  |
| I30 | `.claude/rules/implementation-workflow.md:73`：「不归上游 singlefs-ai-sop 管」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:54`：「不归上游 singlefs-ai-sop 管」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:157`：「缺 herd7 直接红，不静默跳过」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:158`：「两块 virtio 盘上跑固定负载」 |  |
| I31 | `.claude/rules/implementation-workflow.md:81`：「单跑一道不算跑过门禁」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:54`：「单跑一道不算跑过门禁」 |  |
| I32 | `.claude/rules/implementation-workflow.md:86`：「**写法**：切片并行」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:58`：「**写法**：切片并行」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:205`：「Rust 装置取登记的每台默认重活线程数的写法」 | 矛盾表 验证-8 |
| I33 | `.claude/rules/implementation-workflow.md:87`：「**进度间隔**：5 分钟」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:59`：「**进度间隔**：5 分钟」 |  |
| I34 | `.claude/rules/implementation-workflow.md:88`：「**分片**：经服务交的入口」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:60`：「**分片**：经服务交的入口」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:211`：「各道三步怎么做在各自文件头」；`.claude/gate.d/crates-mutation-replay.sh:144`：「merge  先核各片判的行按此刻的行键在服务里读得到」；`.claude/gate.d/crates-mutation-replay.sh:19`：「只核账，库里没有键相同记录的行不现判、整道判红」 | 矛盾表 验证-2、验证-3 |
| I35 | `.claude/rules/implementation-workflow.md:90`：「**门禁管哪一半**：崩溃点重放由门禁」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:61`：「**门禁管哪一半**：门禁判得出的那一半」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:146`：「用例跑了至少两片、没有显式把线程数设成 1」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:147`：「崩溃放量的分片驱动只经服务的分片协议起」 | 矛盾表 验证-9 |

### `.claude/rules/three-way-inference.md`

| 号 | 原件 | 合稿树落点 | 备注 |
|---|---|---|---|
| T1 | `.claude/rules/three-way-inference.md:9`：「以及任何用来推翻或确立」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:5`：「以及用来推翻或确立」 |  |
| T2 | `.claude/rules/three-way-inference.md:11`：「查一个事实（跑命令、读文件、看源码）、算术」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:5`：「查一个事实（跑命令、读文件、看源码）、算术」 |  |
| T3 | `.claude/rules/three-way-inference.md:17`：「不构成免走三方的理由」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:5`：「不构成免走三方的理由」 |  |
| T4 | `.claude/rules/three-way-inference.md:21`：「一轮派两条推论腿」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:9`：「一轮派两条推论腿」 |  |
| T5 | `.claude/rules/three-way-inference.md:28`：「指的是各条腿要拿到不同的切入角」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:11`：「指的是各条腿要拿到不同的切入角」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:11`：「辩方腿替被判出局或被攻的一方辩护」 |  |
| T6 | `.claude/rules/three-way-inference.md:35`：「**射程是三处，不止背景材料**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:15`：「**射程是三处，不止背景材料**」 |  |
| T7 | `.claude/rules/three-way-inference.md:44`：「整条抄进去，连同它的 ⚠️ 与括注」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:17`：「整条抄进去，连同它的 ⚠️ 与括注」 |  |
| T8 | `.claude/rules/three-way-inference.md:48`：「有没有一个字是引用处没有的」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:18`：「有没有一个字是引用处没有的」 |  |
| T9 | `.claude/rules/three-way-inference.md:51`：「**别用手抄**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:23`：「**别用手抄**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:14`：「与源文件逐字节比对」；`scripts/kb/quote-kb.py:99`：「回读比对不一致」 | 退出码只剩脚本文件头 |
| T10 | `.claude/rules/three-way-inference.md:57`：「取，不用 `:行区间` 取」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:23`：「取，不用 `:行区间` 取」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:15`：「例外是分项索引表」 |  |
| T11 | `.claude/rules/three-way-inference.md:62`：「**材料要带一张小节清单。**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:24`：「**材料要带一张小节清单。**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:16`：「这张表本身进材料」 |  |
| T12 | `.claude/rules/three-way-inference.md:66`：「生成，不要临时用 awk 抓标题行」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:24`：「生成，不要临时用 awk 抓标题行」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:16`：「开头那段正文与一级标题下的正文各单独占一行」；`scripts/kb/kb-sections.py:45`：「KB_SECTIONS_NO_PREAMBLE=1」 | 自证与弄坏开关只剩脚本文件头 |
| T13 | `.claude/rules/three-way-inference.md:71`：「清单与附录由脚本绑住」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:25`：「清单与附录由脚本绑住」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:17`：「取法从清单里标」；`scripts/kb/quote-kb.py:103`：「反向核对，C320」；`scripts/agents/checklist-specs.py:27`：「CHECKLIST_SPECS_SPLIT_ON_SPACE=1」 | 退出码 4 到 8 只剩脚本文件头 |
| T14 | `.claude/rules/three-way-inference.md:77`：「机器生成的清单不许再过滤」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:24`：「机器生成的清单不许再过滤」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:16`：「机器生成的清单整份进材料」 |  |
| T15 | `.claude/rules/three-way-inference.md:80`：「⚠️ **射程按」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:24`：「⚠️ **射程按」 |  |
| T16 | `.claude/rules/three-way-inference.md:82`：「正文没点名的枢纽条款」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:25`：「正文没点名的枢纽条款」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:18`：「拿它的动词全仓 grep 一遍」 |  |
| T17 | `.claude/rules/three-way-inference.md:85`：「的理由要能被核」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:25`：「的理由要能被核」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:19`：「写之前去数一眼那个小节在不在」 |  |
| T18 | `.claude/rules/three-way-inference.md:88`：「判据表也要照这条办」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:26`：「判据表也要照这条办」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:20`：「都摘成一格」 |  |
| T19 | `.claude/rules/three-way-inference.md:93`：「只读结论句、不读它自己的射程句」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:20`：「只读结论句、不读它自己的射程句」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:21`：「而引用处没连」 |  |
| T20 | `.claude/rules/three-way-inference.md:99`：「**门禁拦不住这一条**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:28`：「**门禁拦不住这一条**」 |  |
| T21 | `.claude/rules/three-way-inference.md:104`：「**三方一致 → 继续。**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:34`：「**三方一致 → 继续。**」 |  |
| T22 | `.claude/rules/three-way-inference.md:107`：「**腿要跑的都交主 agent**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:35`：「**腿要跑的都交主 agent**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:26`：「腿要跑的（模型、复跑命令」 |  |
| T23 | `.claude/rules/three-way-inference.md:108`：「一条腿在副本装置上量出来的数」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:36`：「一条腿在副本装置上量出来的数」 |  |
| T24 | `.claude/rules/three-way-inference.md:110`：「攻方腿自己提的收严」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:37`：「攻方腿自己提的收严」 |  |
| T25 | `.claude/rules/three-way-inference.md:112`：「写进产品代码之前先在探针里对它做 A/B」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:38`：「写进产品代码之前先在探针里对它做 A/B」 |  |
| T26 | `.claude/rules/three-way-inference.md:113`：「攻方腿的装置把用户动作写死时」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:39`：「攻方腿的装置把用户动作写死时」 |  |
| T27 | `.claude/rules/three-way-inference.md:116`：「不是再当一条推论」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:32`：「不是再当一条推论」 |  |
| T28 | `.claude/rules/three-way-inference.md:120`：「一轮攻击打中的东西先挂起」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「打中的东西先挂起」 | 用户定案 2，见格二 |
| T29 | `.claude/rules/three-way-inference.md:122`：「**第三轮之后停，不开第四轮。**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:45`：「**第三轮之后停，不开第四轮。**」 |  |
| T30 | `.claude/rules/three-way-inference.md:124`：「**用户叫停、不写判决的轮**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:51`：「**用户叫停、不写判决的轮**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:35`：「四列用制表符分隔」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:39`：「恰好命中一次的」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:40`：「出处不许指这一轮自己的材料」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:41`：「共用一份登记行判法」 | 矛盾表 规则第二遍-2、3 |
| T31 | `.claude/rules/three-way-inference.md:126`：「腿交齐之后、写判决之前，主 agent 自己逐条核」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:53`：「腿交齐之后、写判决之前，主 agent 自己逐条核」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:47`：「snapshot-tree.py <轮> --out」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:48`：「腿引的每一行产物，在产物文件里逐字找」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:49`：「比 sha256、比输出与腿报告」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:50`：「核不动的（要虚机」 |  |
| T32 | `.claude/rules/three-way-inference.md:133`：「**判决从骨架起笔**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:60`：「**判决从骨架起笔**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:56`：「排他新建」 |  |
| T33 | `.claude/rules/three-way-inference.md:135`：「三轮分工：第一轮过度外推与越位」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「第一轮过度外推与越位」 |  |
| T34 | `.claude/rules/three-way-inference.md:137`：「攻击结果要主 agent 逐条现查再采纳」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:34`：「攻击结果主 agent 逐条现查再采纳」 |  |
| T35 | `.claude/rules/three-way-inference.md:139`：「要三条互不共享前提的验证路径」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:47`：「要三条互不共享前提的验证路径」 |  |
| T36 | `.claude/rules/three-way-inference.md:141`：「**岔路交用户定之前，每条路要有代价数。**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:49`：「**岔路交用户定之前，每条路要有代价数。**」 |  |
| T37 | `.claude/rules/three-way-inference.md:145`：「**每段交回对着岔路单判够不够。**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:64`：「**每段交回对着岔路单判够不够。**」 |  |
| T38 | `.claude/rules/three-way-inference.md:156`：「**一次不算**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:68`：「**一次不算**」 |  |
| T39 | `.claude/rules/three-way-inference.md:158`：「至少再抽一次」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:70`：「用同一份提示再派一条同立场腿」 |  |
| T40 | `.claude/rules/three-way-inference.md:159`：「**几份样本一不一致」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:71`：「**几份样本一不一致」 |  |
| T41 | `.claude/rules/three-way-inference.md:164`：「每一次工具调用写进文件的内容不超过 150 行」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:75`：「每一次工具调用写进文件的内容不超过 150 行」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:63`：「按分段写的提示重派」 | 矛盾表 常驻-15 |
| T42 | `.claude/rules/three-way-inference.md:167`：「**中途停下、被停或报错的腿，新开一条接着做，不续原来那条。**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:77`：「**中途停下、被停或报错的腿，新开一条接着做，不续原来那条。**」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:64`：「agent-handover.py --agent」 |  |
| T43 | `.claude/rules/three-way-inference.md:167`：「**上下文多大不是停、不续的理由**」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:77`：「**上下文多大不是停、不续的理由」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:65`：「**上下文多大不是停、不续的理由**」 |  |
| T44 | `.claude/rules/three-way-inference.md:169`：「草稿与临时文件放腿自己的目录」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:75`：「草稿与临时文件放腿自己的目录」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:62`：「不放会话共用的暂存目录」 |  |
| T45 | `.claude/rules/three-way-inference.md:171`：「引 kb 条款时写 kb 文件自己的行号」 | `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:75`：「引 kb 条款时写 kb 文件自己的行号」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:62`：「行号去 kb 文件里现查，不从背景材料里数」 |  |

## 二、找不到落点的，与只剩指针的

### 找不到落点的（4 处）

1. 旧库由谁删。原件 `.claude/rules/verification.md:68`：「那一族接完之后旧库由主 agent 删掉，删法照」；合稿只留 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:54`：「只删那一族旧库所在的目录，别的族的目录不碰。旧库不原地打开、不并存」，「由主 agent 删」没有了。找过的文件：合稿树 `dot-claude/` 整棵与 `CLAUDE-merged.md`，`grep -rnF '由主 agent 删掉'` 命中 0 行（量的），`grep -rnF '旧库'` 命中 3 行（合稿 `agent-common.md` 两行、合稿 `verification.md` 一行），三行都没写谁删。
2. 三个欠账号的交叉引用。原件 `.claude/rules/three-way-inference.md:69`：「C273（小节清单不认一级标题）」、`.claude/rules/three-way-inference.md:75`：「C234（三方材料的小节清单只覆盖被判那一项所在的文件）」、`.claude/rules/three-way-inference.md:73`：「C320（小节清单标」；合稿树 `dot-claude/` 整棵 `grep -rlF` 三个号各命中 0 份文件（量的；同一节里的 C244、C192 各命中 1 份，还在）。这三个号此刻只在 `scripts/kb/quote-kb.py` 文件头的退出码表里找得到 C320、C234 两个（表格 T13 那一行），C273 在合稿树与那两份脚本文件头我查过的行里都没看到。它们是指路的编号，不是判据；按口径第 1 条「路径」算不算丢，交主 agent 判。

### 删掉而我判不算丢的（1 处）

- 原件 `.claude/rules/three-way-inference.md:21`：「历史与依据」（那一句指到 2026-09-28 的一份记录、并写了别读它）整句删了。它是来历指针，口径第 4 条许删。

### 只剩指针、我去指到的地方查过那句话在的（11 行）

表里落点一栏出现脚本路径的那几行：V23、V24、V25、V28（门禁 `crash-point-replay` 文件头）、V76（共用库 `stage-cells.sh` 文件头）、V78（`verus-proof-check.py` 文件头）、V88（`cpu-judging-sites-check.py` 文件头）、I34（门禁 crates-mutation-replay 文件头）、T9、T12、T13（`quote-kb.py`、`kb-sections.py`、`checklist-specs.py` 文件头）。每行的脚本片段都逐字查到了。另有一句 `.claude/rules/verification.md:39`：「库里的红块不判」 合稿两份规则里都没抄，门禁文件头有：`.claude/gate.d/crash-point-replay.sh:121`：「judged-by-test 的用例（故障注入之后的红块由用例自己认下」。

### 只剩指针、我只核了指到的小节名在、没逐句核的

见第四节「没核到的范围」。

## 三、五格的结论

### 格一 丢失：不成立（有 1 条判据少了半句，3 个指路编号没了）

173 行里 172 行有落点（合稿规则、参考件、技能、共用约束，或合稿指到的脚本文件头）。打中的是上一节「找不到落点的」第 1 处；第 2 处算不算由主 agent 判。

什么现象会推翻它：合稿树里另有一处写了接完之后旧库由谁删（我搜的是「旧库」两个字，换了说法的搜不到）。

### 格二 二义：不成立（给得出三处两种读法）

1. 参考件里有两个同名标题。合稿 `verification-reference.md` 第 140 行是二级标题「门禁管哪一半」，第 213 行是三级标题「门禁管哪一半」（`grep -n '门禁管哪一半'` 的原样输出里这两行分别是 `140:## 门禁管哪一半` 与 `213:### 门禁管哪一半`，量的）。按标题指过去的有：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:99`：「各条判据由哪道门禁、哪一格判，逐行在」（后面接的就是这个标题），`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:217`：「判工作线程的那一条在」（同一行里写的是「门禁管哪一半」一节）。读法一落到第 140 行那张表，读法二落到第 213 行那一段（只有工作线程一句）；`scripts/agents/cite-check.py` 对「标题命中不是恰好一次」也是判红的写法。
2. 提交之外、用户要求跑不指定范围的那一趟，要不要再弹窗。`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「提交之外要跑先弹窗问用户」 与 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:59`：「提交之外要跑先弹窗问用户」 读出来是「提交之外一律先弹窗」；`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`：「提交之外，用户要求的照跑」 读出来是「用户要求的不弹窗，主 agent 自己判要跑的才弹窗」；`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:12`：「只在提交时或用户要求时交」 把两种说法写在同一句里。原件这一句是 `.claude/rules/verification.md:33`：「）；提交之外要跑先弹窗问用户」，矛盾表「入口-1」只登了 `main-agent.md` 那一处改判据，`verification.md` 与共用约束这两处没跟着改。两种读法做法不同：一种多一次弹窗。
3. 第一轮打中之后开几轮。同一行里 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「同一个结论再攻两轮，三轮里多数打穿才算打穿」 读出来是「打中了就一定再攻两轮（第二、三轮都开）」；`research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「前两轮不一致、无法收敛的开第三轮」 读出来是「第二轮收敛了就不开第三轮」。第一轮打中、第二轮两条腿一致没打中时，两种读法一个开第三轮、一个不开。用户定案第 2 条的原话是 `records/2026-11-16-三方豁免取消与存量补轮.md:119`：「打中的按三轮多数判」，定案本身没说第二轮收敛时第三轮开不开；这是落法里的二义，不是定案该不该。
   同一行的 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「第一轮两条腿结论一致、没打中，就停」 与 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:70`：「两次都没打中才记」 不冲突（再抽一份样本是同一轮里的事），只提一句。

没算打中的：技能与派发表里写「各格判什么照 `verification.md` 的两种跑法一节」的三处（`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/SKILL.md:18`：「各格判什么在」、`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:24`：「各片与合并各判什么」、`research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:96`：「次序与各道判什么照那一份与」），那些内容已挪到参考件，要经 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:44`：「起一道门禁或入口、照它的写法单跑一趟」 再跳一次才到。跳两次到得了，读法只有一种。

什么现象会推翻它：第 1 条，换上时两个标题有一个改了名；第 2 条，主 agent 认定 `verification.md` 那一句的「要跑」本来就只指主 agent 自己判要跑的；第 3 条，用户说过第二轮收敛时第三轮开不开。

### 格三 常驻够不够：不成立（有一类判据只剩在参考件里，常驻的入口句罩不到写代码的那一步）

入口句本身都在：合稿 `verification.md` 第 14、44、46、57、95、99 行，合稿 `implementation-workflow.md` 第 29、33、42、63 行，合稿 `three-way-inference.md` 第 22、51、53、60、77 行，各有一句「做某事之前先读某份某节」（我逐行读过，行号是那几句所在的行）。

打中的是原件「门禁管哪一半」那张表：表的左栏「判什么」里有几条判据，原件别处没再写一遍，整张表挪进参考件之后常驻文件里就没有这几条了，而常驻的入口句 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:99`：「查一条判据有没有门禁管、红了归哪一格之前先读它」 说的是「查门禁」时才读，写测试、写代码的人不在这句话的射程里：

- 测试文件要声明模块。原件 `.claude/rules/verification.md:138`：「那一样声明模块」；合稿只在 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:151`：「那一样声明模块」。合稿实现员定义的开工先读一行还在括号里把它算作 `verification.md` 那三节的内容：`research/prompts/rules-compress-r1-merged/dot-claude/agents/implementation-writer.md:16`：「测试文件第一行声明模块」，可合稿 `verification.md` 整份 `grep -nF '声明模块'` 命中 0 行（量的）。实现员照定义读完那三节，读不到这一条，新测试文件不声明模块，门禁 `crash-point-replay` 开跑前那一步就红。
- 判器调用点的标记写在哪。原件 `.claude/rules/verification.md:141`：「标记写在包住调用点的那个函数前面」；合稿只在参考件那张表里，写法还再指到判法脚本：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:154`：「标记写在哪才算在它文件头」。`gpu-exempt` 在合稿树里只命中参考件这一行（量的）。
- 变异行点名的测试要筛得到。合稿只在 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:156`：「变异行点名的测试跑得到」。
- 每道阶段认第一个参数当项目根。合稿只在参考件同一张表（落点表 V91）；合稿工具员定义点名读的是参考件「门禁的结构」与复用那一节（`research/prompts/rules-compress-r1-merged/dot-claude/agents/tooling-writer.md:16`：「写门禁、改算键的代码时另读」），不含这张表。

什么现象会推翻它：这四条在合稿树别的常驻文件或对应定义里另有一句写全了（我按「声明模块」「gpu-exempt」「cpu-reference-for-gpu-comparison」「筛选词」「第一个参数当项目根」五个词搜的整棵 `dot-claude/` 与 `CLAUDE-merged.md`，命中都只在参考件与上面引的实现员定义那一行）。

### 格四 子 agent 读得到：成立，带一处要改的指针

- 这一组五份文件都读得到：三份规则与参考件在 `.claude/rules/` 下，共用约束写的是 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`：「点到就整份读」，定义点到哪一节都整份读，所以规则正文里「先读参考件某节」「先读技能某节」的入口句子 agent 看得见；技能是普通文件，被规则正文点名了节。
- 两条腿的定义点名的节都在：`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-forward.md:17`：「开工先读：」（这一行点的三份规则的节标题在合稿里都还在，我逐个对过合稿 `three-way-inference.md`、`implementation-first.md`、`evidence-discipline.md` 的标题），`research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-attack.md:17`：「开工先读：」。
- 共用约束指进这一组的两处节名在：`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:28`：「写测试、门禁、入口脚本、实验装置的，开工另读」（后面点的两个节标题与合稿 `verification.md` 的两个二级标题逐字相同）。
- 要改的一处就是格三第一条：实现员定义括号里说 `verification.md` 那三节有「测试文件第一行声明模块」，指到的地方没有这句话。文件读得到，话不在。

什么现象会推翻它：有一份定义的指针指到这五份之外、而那份定义又没被点名去读的文件（我只查了指进这五份的指针，共 32 行 grep 命中，逐行看过；指到别组文件的没查）。

### 格五 悄悄改了做法：不成立（有 3 处照脚本文件头改了字，矛盾总表里没登）

矛盾总表 `research/prompts/rules-compress-r1-merged/contradictions.md` 里按 `source`、`挂特性`、`无判定库` 三个词 `grep -cF` 都是 0（量的）。

1. 原件 `.claude/rules/verification.md:112`：「不 source 它，参数照同一张表自己解析」；合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:116`：「不 source 它跑格，参数照同一张表自己解析」。依据是门禁文件头 `.claude/gate.d/crash-point-replay.sh:237`：「不用共用库 lib/stage-cells.sh 跑格」（它 source 那份库，只是不用它跑格）。这一处是把说错的事实改对，照着做的人做法不变。
2. 可点的项多了一种。原件 `.claude/rules/verification.md:126`：「按崩溃枚举用例（」（后面只列崩溃枚举用例）；合稿在同一句里加了 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:130`：「与挂特性的测试文件（它文件头」。主 agent 与子 agent 能点的 `--item` 多了一类，是做法上的增加。
3. 登记表第三列许写什么。原件说的是现状：`.claude/rules/verification.md:67`：「（门禁样本档）一行」（只剩这一行）；合稿写成了规则：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:106`：「第三列不是 prune-server 的只许是」，后面许的是「无判定库：欠账号或待立」两种。依据是登记表表头 `.claude/gate.d/verdict-library-scripts.tsv:3`：「没有判定库的测试入口写」。许写的形态从一行现状放宽成一类。

另一处我判不算改做法：原件 `.claude/rules/verification.md:39`：「不指定范围那一趟经服务交，服务照它文件头」 说的是不指定范围那一趟，合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:18`：「经服务交的每一种跑法（不带参数」 说的是每一种跑法；原件另一份本来就写的是每一种（`.claude/rules/implementation-workflow.md:88`：「经服务交的每一种跑法都拆成计划、各片、合并」），门禁文件头也是（`.claude/gate.d/crash-point-replay.sh:135`：「让服务把经它交的每一种跑法」），合稿取了后一种。两份原件互相不齐的地方照脚本收成一种，矛盾表里没有这一条，要不要补登由主 agent 定。

用户定案十六条里落在这一组的，我核了落点在：第 2 条（落点表 T28、I2）、第 4 条（I1）、第 9 条（V6、V87）、第 11 条（三份开头的声明句都删了，合稿三份第 1 到 5 行我读过）。第 2 条的落法见格二第 3 条。第 13 到 16 条说的是脚本与门禁的后续活，不落在这五份文件里，没核。

什么现象会推翻它：矛盾总表之外另有一份登记（写手报告、调度记录）已经登了这三处；我照派发提示只认矛盾总表与用户定案。

## 四、核过的条数与没核到的范围

- 核过并确认有落点：落点表 173 行（`verification.md` 93 行、`implementation-workflow.md` 35 行、`three-way-inference.md` 45 行），其中 172 行落点完整，1 行（V55）少半句。行数是辅助脚本打的（量的）：三份规格 50、54、69 行，命中 0 次的片段 0 个。
- 三份原件我都从头到尾读了一遍（146、90、171 行）；合稿五份、合稿共用约束与矛盾总表整份读过。
- 没核到的：
  1. 参考件写「以某脚本文件头某段为准」而我只数了小节名在不在、没逐句对的：`formal-verification.sh` 文件头「复用」「分片」，`broad-mutation.sh` 与 `broad-mutation.py` 文件头「分片」「判定库」「内存与限时」，`lib_reuse_checks.py` 文件头「一」「二」「三」，`merge-and-verify.sh` 文件头「崩溃域上限」，`lib_crash_point_replay_items.py` 文件头「指定范围的崩溃域上限」，`lib_function_units.py`、`lib_verdict_library.py`、`prune_server_client.py`、`task_server_lib.py`、`task_server.py`、`replay.sh`、E165 第二段驱动文件头各一段，`doc-process-records.sh` 文件头 code-round-snapshot 一格。原件里对应的细则（各片起 `SHARE=<片号>/<N>`、哨兵抽法这类）有没有逐句在那些文件头里，这一格是「推不出」。
  2. 原件与合稿都写着的一个指针我看着不对、但不是压缩带来的：`.claude/rules/implementation-workflow.md:53`：「树里起命令要的多机配置照」 与 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:195`：「树里起命令要的多机配置照」 指到的那一行此刻是 `scripts/hosts/minimal-copy.py:25`：「字面量那一条（research/ 下的」，说的不是多机配置。原样照搬，没算进五格。
  3. 「不经家目录取」那半句：原件 `.claude/rules/verification.md:65`：「两台同一绝对路径，不经」，合稿规则里是 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:51`：「点名的共享根，两台读同一份」，后半句只在 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`：「两台挂在同一绝对路径，不经」。有落点，但挪到了共用约束；主 agent 起步读不读共用约束我没核。
  4. 合稿树里别组的文件（其余规则、其余定义、其余技能）只在追落点时按词搜过，没逐条读；写手对照表、`merge-result.md`、`references.md` 没读。攻方腿的产出没读。

## 要主 agent 跑的命令

没有。这条腿没写模型，也没有要起的脚本。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一 丢失 | 不成立 | 旧库由主 agent 删这半句没了；C273、C234、C320 三个指路编号没了；脚本文件头没逐句核的那一批推不出 |
| 二 二义 | 不成立 | 参考件两个同名标题；提交之外用户要求时弹不弹窗三处说法不齐；第一轮打中后第三轮开不开读得出两种 |
| 三 常驻够不够 | 不成立 | 声明模块、判器调用点标记、变异行筛选词、阶段第一个参数四条只在参考件的表里，入口句罩不到写代码的人 |
| 四 子 agent 读得到 | 成立 | 五份都读得到；实现员定义括号里说的声明模块在它点的节里没有 |
| 五 悄悄改了做法 | 不成立 | 三处照脚本文件头改了字而矛盾总表没登，其中两处放宽或增加了做法 |

## 没做什么

- 不判攻方腿的格，不替主 agent 采纳或出判决；没读禁读清单里的文件。
- 没往统一任务服务交任务，没跑任何脚本的自证；`scripts/agents/cite-check.py` 是直接跑的文本工具，交回前跑了一遍。
- 没改合稿树、没改规则与定义；格三、格四里要改的指针只写在这里。
- 用户定案第 13 到 16 条与矛盾总表里别组的条目没核。
