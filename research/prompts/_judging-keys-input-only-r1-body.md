# judging-keys-input-only-r1：判定键只有输入数据之后，什么改动会假绿（2026-11-20）

## 一、题面

用户 2026-11-19、11-20 定：判定与剪枝的键只看数据。崩溃判定的键只由块的输入数据组成，server、GPU 与 CPU 判定器（池级 checker、记录核对器）、观察者、录制管线的代码都不进键；改判定器不作废已判的块，判法改了对不对靠对拍与复用哨兵抓；录入键只取产品代码加场景（每条崩溃枚举用例各一把，在 `crates/singlefs-unit-graph` 按包过滤）；同一块上的几个观察者用判定项标识分开；判器那几组的入口登记整批删；旧库里新键下复用不到的判定与录入数据都删掉。定案原话与弹窗在 `records/2026-11-19-判定键只算产品与测试代码.md` 第 82–97 行，写回在 D13（验证路线） 已定项 17（整条在附录）。

这一轮只攻一件事：键里没有判定器、观察者、录制管线与测试服务代码之后，有哪些改动会让一条判定在键不变的情况下被复用、而它的结局本该变（假绿，或假红被当成绿复用），对拍与复用哨兵抓不抓得到。方案按 `crates/` 今天的实现与在写的补丁来谈。

## 二、实现今天的样子（主 agent 的观测，2026-11-20）

读过的路径与看到的事实（主工作区此刻，补丁还没合）：

- `crates/singlefs-verification/src/crash_identity/judging_units_key_and_composition.rs:10-38`：`enum JudgingUnits` 有 `GpuJudge`、`JudgementOfTheCrashImages`（CPU 参照判器 `crash::judge_crash_image` 的闭包，文档注释写「镜像判定的版本就是这一把，不论这一块是 CPU 判的还是 GPU 判的」）、`JudgementOfTheGivenStates`、`CpuFactsVerifier`、`GpuFactsVerifier`、`Recording`、`ObserverConclusionComposition`、`PairingComparison`、`ObserverCode` 九种。
- `crates/singlefs-verification/src/crash_library/item_manifests_and_two_stages.rs:44-52`：录入键 `recording_key_of_the_inputs` 是 `JudgingUnits::Recording` 那一组的单元键加录入输入摘要取 SHA-256；录入单元拒算时只拼输入摘要、照录不复用。
- `crates/singlefs-verification/src/verdict_store/gpu_pending_rows.rs:25`：GPU 待判行带 `expected_gpu_judge_units: JudgingUnitsFingerprint`，判段问 GPU 服务收不收（`crates/singlefs-verification/src/crash_amplification.rs:1460` 起 `exit_because_the_gpu_verification_server_does_not_serve`）。
- 在写的补丁（实现员草稿 `~/.local/share/singlefs/work/judging-keys-input-data-only/`，报告 `result.md`）：三种结论的版本只由结论种类定，核对键只取事实表摘要，`19` 对拍键是 `19` 加输入摘要；待判行格式号由 1 升到 2、不再带期待键；服务一侧（`~/.local/share/singlefs/work/judging-keys-service-side/`）认领待判行不按键挑。
- 调查员报告 `~/.local/share/singlefs/work/judging-keys-change-list-after-split/result.md`：照拆后文件重列 191 处改动；`checking_threads()` 定义之外四处调用，线程数只定一块切几段、不改判定结论（依据 `crash_amplification/cpu_checking.rs:559`、`observer_recording.rs:914`、`cpu_checking.rs:224`）。
- 复用哨兵与强制现判的判据在 `.claude/rules/verification.md`「一个门禁：两种跑法」：每一趟真跑开跑前从键相同、库里有判定的项里随机抽几条当哨兵、再抽一条强制现判，哨兵有一条被现判、强制现判的判定与库里那一条不同、或强制现判那一条仍复用，整道红。

推的、没量过的：每一趟抽几条哨兵、强制现判几条；抽中一条被改坏的判器判错的块的概率。

## 三、要回答的问题

1. 构造假绿：键里没有判定器代码之后，改池级 checker、记录核对器、GPU 判器或观察者的一处（漏判一条不变量、把红判成绿、把绿判成红），已判的块照键复用、结局不变。哪几类改动，对拍（GPU 对 CPU 参照）与复用哨兵、强制现判都抓不到？逐类写：改哪个文件哪一类判法、哪一份判定的结局该怎么变、为什么三样都抓不到（两边一起改错、哨兵抽不到那一块、强制现判只抽一条）。构造不出写「推不出反例」并写查过哪些路径。
2. 第 1 问里抓不到的那几类，规则要补什么才能抓到，且不把判定器代码放回键里：例如改判定器代码时强制全量重判一次、哨兵按改动过的判法加大抽样、判定器改动提交前必须跑一次不复用的全量对照。每条写成对实现的改动（改 `crates/` 或 `scripts/` 哪一处），写明代价（多跑多少）与它替代了什么。
3. 录入键只取产品代码加场景（每条用例一把，按包过滤）：录出来的数据会因为别的东西变而不变吗（工具链、依赖版本、`Cargo.lock`、特性开关、构建脚本、录制管线里改变写什么的代码）？D13（验证路线） 已定项 17 的新写法把工具链与依赖版本从块键里去掉了；录入键里留不留它们，留了算不算违反「数据和怎么录入没有关系」。
4. 观察者用判定项标识分开：同一个判定项标识下观察者的判法改了（判的意思变了），旧结论照复用，算不算第 1 问那一类；判定项标识要不要在判法意思变时换号，谁判、怎么钉住。
5. 删旧库：「新键下复用不到的都删」之后，有没有哪一类旧判定其实在新键下还复用得到（键恰好相同），删了等于白白重判；有没有哪一类删不干净会被新代码误读（格式号升了、旧行读不出照拒）。
6. 这一套与 D13（验证路线） 已定项 17 定案其余几条（键过窄算违反、收不全的拒算照跑、改键让旧库作废不算代价）有没有冲突；冲突的写出原句与改法。

## 四、压着的条款

D13（验证路线） 已定项 17 整条在附录；用户定案原话在 `records/2026-11-19-判定键只算产品与测试代码.md` 第 82–97 行。
