# GPU 服务接住 OOM、卡坏上报与借卡：治理文件轮第二轮（gpu-oom-card-governance-r2，2026-11-21）

这一轮攻的是第一轮判决（`research/prompts/gpu-oom-card-governance-r1-main-verification.md`）九条改法落成的修补，加第一轮之后换上、还没被任何一轮攻过的四样：借卡白名单、借卡先来后到排队与 `lease list`、`control install` 先停再清场再起、配置判法认九个清场键。改动都已在主工作区，两台 GPU 批处理验证服务跑的是照主工作区编的二进制。调度记录在 `records/2026-11-18-GPU服务OOM后整卡停用.md`。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 第一轮判决的十一条坐实、七条不成立，判得够不够得着：坐实的有没有判重了、不成立的有没有放过了（辩方腿复核）。
2. 九条改法各自落成的修补，与第一轮之后换上的四样，哪一格会错（攻方腿）。第一轮攻过而没打中的角度不重复攻：卡 0 OOM 那一刻卡 1 的账与驻留判器、同一块判两次或写错卡写错流、多机撞键漏台、两条算键登记的理由。

## 实现今天的样子

主 agent 的观测（读的是 2026-11-21 主工作区的文件；行号是那一刻的）：

- 这一轮没有改 `crates/` 下的产品代码与验证包；被判的 Rust 代码都在 `gpu-verification-server/`（它依赖验证包 `crates/singlefs-verification`，不在 `crates/` 下）。`crates/mutations.tsv` 里锚在 `gpu-verification-server/` 上的变异行跟着这几批补丁追加，最近一批七行（名字以「借卡排队」「借卡 lease list」起头）。
- **第一轮改法第 1 条（OOM 之后先做一次提交）**：`gpu-verification-server/src/card_worker.rs` 文件头第 6 到 14 行写四种情形；`JudgesOnACard::settle_after_running_out_of_memory` 做那一次提交并等它做完，没等成的报这张卡坏了；弄坏开关 card-not-settled-after-running-out-of-memory。
- **改法第 2 条（因显存不够丢的设备重开）**：真卡那一份 `after_a_panic` 在 `gpu-verification-server/src/gpu_judges_on_a_card.rs` 第 422 行；重开没成的答设备丢了（`card_worker.rs` 第 290 到 314 行拼原因）；开关 device-lost-for-want-of-memory-taken-for-a-card-failure。
- **改法第 3、4 条（判法 panic 与腾空卡上再 OOM 的块：报拒、放认领给另一台、报红）**：`ServiceRefusal::OutOfMemoryOnAnEmptiedCard` 与 `ServiceRefusal::JudgePanicked`（`gpu-verification-server/src/service_loop.rs` 第 94、96 行），`take_a_judge_panic`（第 1575 行）、`take_an_out_of_memory`（第 1480 行）、`refuse_the_block`（第 1356 行）；判不了的块每块一行 `GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE family= stream= block= card= why= claim= reason=`（`gpu-verification-server/src/serving_from_the_library.rs` 第 109、225 行），放掉认领让另一台取，开关 block-not-judgeable-claim-kept 下不放。
- **改法第 5 条（核对器借显存）**：`mebibytes_of_a_facts_verifier_under`（`gpu-verification-server/src/card_memory_ledger.rs`，调用在 `serving_from_the_library.rs` 第 964 行）。
- **改法第 6、7 条（撤回的块别的卡也能拿；大卡坏了只有它装得下的块当场报拒）**：`put_back_a_piece`（`service_loop.rs` 第 1385 行）、`put_back_the_whole_block`（第 1435 行）、`take_a_failed_card`（第 1257 行）、`plan_a_lone_block_on_an_emptied_card`（`gpu-verification-server/src/batch_scheduling.rs` 第 760 行）。
- **改法第 8 条（读不到不当成好了、去重落盘、卡号取不到报错）**：`gpu-verification-server/control` 的 `cards-failed` 对没读成的台另打 `GPU_VERIFICATION_SERVER_CARDS_FAILED_UNREAD host= state=`（文件头第 37 到 47 行）；任务服务拆分之后这条链在 `task-server/task_server_lib_configuration.py`（第 43 到 78 行解析与 `gpu_service_problems_merged`）、`task-server/task_server_scheduler_notifying.py`（第 725 行 `gpu_card_failures`、第 751 行 `read_gpu_cards_failed`）、`task-server/task_server_lib_constants.py` 第 486 到 490 行与 `task-server/task_server_constants.py` 第 586 到 593 行。
- **借卡白名单**：`gpu-verification-server/src/card-lease-borrowers.tsv` 经 `include_str!` 编进二进制；`lease take` 照 `/proc/<借的进程号>/exe` 的文件名认，读之前读之后各认一回借的进程，不在表里退 4、一把键都不认领（`gpu-verification-server/src/card_lease.rs` 文件头「借卡白名单」一段、`executable_name_of_the_borrower`、`admit_only_a_borrower_on_the_whitelist`）；开关 card-lease-borrower-whitelist-unchecked。
- **借卡先来后到与 `lease list`**：`lease take` 过白名单之后先认领一张等号 `gpu-claim/card-lease-waiting/<这一台>/<20 位毫秒>-<10 位进程号>-<20 位启动刻>`，每回认领请求键之前先列这一台的等号、自己排第一才去认领；等的那一段每过三分之一租期续一次等号，两把键到手、起续租进程之前放等号；等待行末尾加 `position=`。`lease list` 只读：每台一行 `GPU_CARD_LEASE_HELD … held_by=service|borrower|nobody`，请求键有人持着一行 `GPU_CARD_LEASE_REQUESTED`，每张等号一行 `GPU_CARD_LEASE_QUEUED … position=`。`gpu-verification-server/control` 的 `lease list` 原样转给这一台装着的二进制。开关 card-lease-waiting-order-ignored（二进制）与 lease-list-refused（control）。测试在 `gpu-verification-server/tests/card_lease_on_a_temporary_library.rs`。
- **`control install` 先停、再清场、再起**：`gpu-verification-server/control` 文件头第 21 行与「install 换版：先停、再清场、再起」一段（第 77 行起）、「开卡之前清场」（第 70 行起）、`quiesce_the_cards`（第 575 行）；清场没过，停掉的那几台照旧二进制起回来、退 78。
- **配置判法认九个清场键**：`scripts/crash/crash-point-replay-shard-configuration-check.sh` 第 77 到 89 行的说明与第 184 行 `MULTI_HOST_QUIESCE_SCRIPT_KEYS`（只认键名、不判值）。
- **判定键那一批之后服务不再有的东西**（`grep -n serves gpu-verification-server/control` 零命中）：`control serves` 子命令与版本键对账已删，判段不再问服务收不收；第一轮判决第 2 条里「判段只问 `control serves`」那半句说的是那时的样子。
- 已跑出的结局（都在调度记录里）：借卡那一个测试目标与库 39 条测试全绿；七行新变异「抓到 7 条、没红 0 条、无效 0 条」；`control --selftest`「cells=30 failed=0」；真服务上 `control lease list` 两台各一行 `held_by=service`。

## 范围

被判的文件（读此刻主工作区那一版，清单与 sha256 在 `research/prompts/gpu-oom-card-governance-r2-snapshot/judged-sha256.txt`）：

- `gpu-verification-server/src/` 下 `card_worker.rs`、`gpu_judges_on_a_card.rs`、`service_loop.rs`、`batch_scheduling.rs`、`card_memory_ledger.rs`、`serving_from_the_library.rs`、`break_switches.rs`、`card_lease.rs`、`card-lease-borrowers.tsv`；
- `gpu-verification-server/tests/` 下 `service_loop_with_judges_off_the_card.rs`、`batch_scheduling_requirements.rs`、`card_lease_on_a_temporary_library.rs`；
- `gpu-verification-server/README.md`、`gpu-verification-server/control`；
- `task-server/task_server_lib_configuration.py`、`task-server/task_server_scheduler_notifying.py`、`task-server/task_server_lib_constants.py`、`task-server/task_server_constants.py`：只判 GPU 卡坏与判不了的块那条告警链；
- `scripts/crash/crash-point-replay-shard-configuration-check.sh`：只判九个清场键那一段与它的自证格。

不在这一轮：算键库的登记（第一轮判为成立；拆分之后挪进 `scripts/gate/lib_function_units_registry.py`，别的会话正在改，改完由它那一轮判）；真卡上跑 `after_a_panic`（第一轮判决第 6 条记成欠的测试，代跑接口待定）；`control serves`（已删）。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（替身判器的设定、一段假 journal、一条命令）或 文件:行。

辩方腿（复核第一轮判决）：

1. 判决第 1、4、5 条的三处改法（先做一次提交、重开设备、判法 panic 不算卡坏）各自在今天的代码里做的是不是判决「三、改法」写的那件事；`card_worker.rs` 文件头四种情形与 `card_worker_loop`、`gpu_judges_on_a_card.rs` 的 `after_a_panic` 逐句对。
2. 判决「不成立、推翻或不采的」七条：逐条在今天的代码上再判一次，有没有因为之后的改动（撤回的块别的卡也能拿、放认领给另一台）变成成立的。
3. 判决第 2 条与交用户的甲案（放掉认领、另一台去试、打判不了行、`control check` 判红、任务服务告警）：从服务打行到 `control check`、`control cards-failed`、任务服务解析与发告警，每一环与下一环的字面对得上没有；另一台也判不了时这一块最后停在哪、有没有人知道。
4. 判决第 9、10 条的修补：没读成的台留着上一回读到的、去重落盘、卡号取不到报错，各自对着 `gpu_service_problems_merged` 与 `read_gpu_cards_failed` 判。

攻方腿：

5. 先做一次提交那一步：那一次提交自己 OOM、自己丢设备、等不回来时各走哪一支；重开设备之后旧设备上的驻留账、在判的批、借出去的卡（`card_lease`）各是什么状态；连着 OOM 时会不会在「重开、再 OOM」里打转而不报任何人。
6. 放认领给另一台：两台都判不了的块会不会在两台之间来回弹（每台放、另一台取、再放）、每一圈各打一行、任务服务每圈发一条告警；放认领与写结论之间有没有一块被两台各判一次、结论写两遍的窗口。
7. 撤回的块别的卡也能拿之后：一块被卡 0 撤回、卡 1 拿去判的同时卡 0 腾空了，会不会两张卡各判一截而合不回去、或同一截判两次；大卡坏了「只有它装得下的块当场报拒」的判法用的是哪一刻的账，坏卡之后登记进来的新流的块照不照同一条判。
8. 借卡白名单：造得出一个不在表里的进程借到卡吗（借方进程号在读 exe 的前后被复用、`/proc/<pid>/exe` 指到已删文件带 ` (deleted)` 后缀、符号链接、同名文件放在别的目录、借方变量写别人的进程号与启动刻）；`lease return`、`lease renew` 不认白名单，能不能被用来占住或放掉别人的键。
9. 借卡排队：造得出后到的先拿到吗（两人同一毫秒起等、时钟回拨、等号在列与认领请求键之间失效、排第一的那个卡在认领卡键上而等号已放）；排第一的借方活着但 `lease take` 进程停住（SIGSTOP）时后面的人等多久、有没有上限；等号残留（`lease take` 被 SIGKILL）最长挡别人多久；服务那一半只看请求键，等号有没有可能让服务一直不放卡或一直不收回。
10. `lease list`：三种 `held_by` 的判法在租期刚过、服务重起换了持有者身份、请求键有人持着而卡键还在服务手里时各打什么，会不会把借方持着说成服务持着；列认领截断时输出是不是少行而不报。
11. `control install` 先停再清场再起：清场没过时「停掉的照旧二进制起回来」那一支，旧二进制已被新二进制盖掉了没有；发起台停了第二台而自己在清场时被杀，两台各停在什么状态、`control check` 怎么说；`install --dry-run` 会不会真的停服务。
12. 九个清场键只认键名不判值：写错值（单元名不存在、台地址不在 HOSTS 里）时哪一步才红、红在哪；配置判法的自证格在键名改了一个字时红不红。
13. 弄坏开关：这几批新加的开关（服务一侧 OOM、判不了、借卡白名单、借卡排队各几条，control 的 lease 那几条）有没有两条弄坏同一处、或点名的那一格靠别的开关也会红；card-lease-keys-held-by-another-taken-as-claimed 的判法挪进排队循环之后，它与 card-lease-waiting-order-ignored 各自点名的格是不是只认自己那一条。

## 分工

辩方腿做第 1 到 4 问，攻方腿做第 5 到 13 问。两条腿都读「范围」里的文件与第一轮的正文、两份腿报告和判决；只有一条腿读到的范围，在报告里写明。要跑的探针（替身判器的用例、喂 control 的假 journal、喂任务服务的假 cards-failed、借卡的小程序）不自己跑，写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条进报告「要主 agent 跑的命令」一节；要起 cargo 的，命令里只点名测试函数，不跑整个包；探针只对临时判定库服务与替身，不借真卡、不动两台在跑的服务。
