# gpu-oom-card-governance-r2：判决（2026-11-21）

正文 `research/prompts/_gpu-oom-card-governance-r2-body.md`，开工快照 `research/prompts/gpu-oom-card-governance-r2-snapshot/`。这是这一题的第二轮，治理文件轮，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」收；第一轮判决是 `research/prompts/gpu-oom-card-governance-r1-main-verification.md`。这一轮有坐实的打中，改完再攻第三轮，第三轮是最后一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方腿（复核第一轮判决，第 1 到 4 问） | `research/prompts/gpu-oom-card-governance-r2-sonnet-output.md` | 第 1 问成立（重开设备与真卡外壳没在卡上执行过，推不出）；第 2 问「有变成成立的」；第 3 问每一环字面对得上、三处文字不配；第 4 问三处各自成立、一个口子打中 |
| 攻方腿（第 5 到 13 问） | `research/prompts/gpu-oom-card-governance-r2-opus-output.md` | 第 5 问推不出；第 6、8、11 问成立；第 7、9、10、12、13 问部分打中；撤回自己第一轮三处说法 |

核查：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py gpu-oom-card-governance-r2 --out /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-verdict/gpu-oom-card-governance-r2-tree --repository /home/fy5090/code/singlefs`：结局 type=succeeded exit=0；20 份被判文件逐份「此刻工作区那一份 sha256 与快照相同」，腿跑着的时候没有一份被改过。
- `python3 scripts/agents/cite-check.py research/prompts/gpu-oom-card-governance-r2-sonnet-output.md research/prompts/gpu-oom-card-governance-r2-opus-output.md --root /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-verdict/gpu-oom-card-governance-r2-tree --background research/prompts/_gpu-oom-card-governance-r2-body.md`：「✓ 核了 109 处引文、97 处裸行号，对上 85 处，没判 121 处（其中裸行号没有引文可比 78 处；逐处列在上面）」，没有对不上的；下面判决引到的行主 agent 逐处在文件里现查过。
- 辩方腿两条探针，主 agent 在冻结副本的拷贝上复跑（两份被判的测试文件与 `card_lease.rs` 的 sha256 与快照相同），结局 type=command-failed exit=101，两条断言都红，与它的预计相同：
  - 「R2_DEFENSE_PROBE name=facts_job_put_back_lands cards_that_loaded=[0, 0, 1, 1] answered_card=[CardIndex(1)] verified_card_and_judges_on_it=[(1, 0)] out_of_memory_caught=1 facts_jobs_put_back=1」
  - 「R2_DEFENSE_PROBE name=put_back_block_cut_by_the_other_card out_of_memory_caught=2 faults_raised_on_the_card_that_ran_out=2 waves=5 judged_states=64 judged_more_than_once=0」
- 攻方腿五格探针，同样在拷贝上复跑，结局 type=command-failed exit=1，末行「name=rerun_steps_mismatched count=2」：
  - 「OOM_ATTACK_PROBE name=put_back_piece_back_on_its_card retracted=[0..64] waves=20 retracted_runs_on_card0=1 retracted_runs_on_card1=1 states_judged=256 judged_once=true」，这一步 matched=yes；
  - 「step=waiting-order-test-under-keys-held-switch … cargo_exit=101 predicted=FAILED seen=FAILED matched=yes」「step=two-borrowers-test-under-waiting-order-switch … cargo_exit=0 predicted=ok seen=ok matched=yes」；
  - 借卡的两条探针（借方进程停住、直接认领的借方被 `lease list` 写成服务）「seen=none matched=no」：没编过或没跑到，不是推错。这两格没有复跑读数，下面照主 agent 读代码的结果判，并写明。
- 核不动的：攻方腿照 wgpu-core 30.0.1 源码推的两句（丢了的旧设备压着的显存不在账上、重开 4 次之后第 5 回报卡坏）与「那一次提交的等是 `timeout: None`」，要在真卡上量或读依赖源码，这一轮没核，不当核过；第 11 问「清场过了之后第二台 install 失败没有起回来那一支」与第 12 问「值写错要到清场那一刻才红」主 agent 没逐支追，只当线索。

## 二、判决

坐实的（主 agent 现查过代码或复跑过探针）：

1. **放回的核对组落到好卡上、把它的判器换光**（辩方第 2 问）。复跑读数 `answered_card=[CardIndex(1)] verified_card_and_judges_on_it=[(1, 0)]`。第一轮判决「不成立」里「攻方『放回的核对组把卡 1 的判器全部换出』：复跑推翻，重核派回卡 0」这一条在今天的代码上翻面：改法第 6 条（撤回的块别的卡也能拿）落成之后，放回的组只在空着的卡里挑，显存不够的那张卡那一刻还不算空着。与用户定案「OOM 需要倍catch 不能影响好的卡」相违。
2. **撤回的块剩下的几截又派回它显存不够过的那张卡**（辩方第 2 问新发现、攻方第 7 问，两腿各自的探针都中）。读数 `faults_raised_on_the_card_that_ran_out=2`、`retracted_runs_on_card0=1 retracted_runs_on_card1=1`；`judged_more_than_once=0`、`judged_once=true`，没有一态判两次、结论没丢。坏处是多一回本可避免的显存不够。
3. **接住之后那一次提交没等成，好卡被报成卡坏**（攻方第 5 问头一格）。`gpu-verification-server/src/card_worker.rs` 第 269 到 275 行：`settle_after_running_out_of_memory` 回错就走 `failed(…)`，不再问设备怎样；那一次提交自己显存不够时卡没坏，照样停到服务重起、`control check` 叫人重置卡。辩方腿第 1 问点名同一处：落成的是一次单元校验和、要建缓冲，不是第一轮改法第 1 条写的空提交。
4. **判不了的块只在进程里记着，服务一重起两台各再试一回、各发一条告警**（攻方第 6 问）。`gpu-verification-server/src/serving_from_the_library.rs` 第 25 行「记下名字，这个进程此后不再认领它（待判行留在库里」；`control install` 会让两台都重起。同一个进程里不来回弹、放认领与写结论之间不判两遍，这两格不成立。
5. **`control check` 与告警写死「已放给另一台去试」**（辩方第 3 问）。`gpu-verification-server/control` 第 1062 行不看那一行的 `claim=`，开关 block-not-judgeable-claim-kept 下没放、单台配置、两台都报的时候照这样写。
6. **告警去重在任务服务重起后有一个口子**（辩方第 4 问）。`task-server/task_server_scheduler_notifying.py` 第 771 到 775 行：头一回读完就把 `gpu_cards_failed_known` 记成真，而那一回有台没读成时没有上一回可留；第 791 到 797 行据此把那一台记过的「发过」当成好了删掉，那一台读成之后重发。
7. **借卡白名单只认可执行文件名，同名拷贝放在哪都放行；借方变量写别人的进程号与启动刻也放行**（攻方第 8 问）。这是定案时写明的认法（`gpu-verification-server/src/card_lease.rs` 文件头「不看目录、不看包名、不看命令行」），测试自己就是把 `sleep` 拷成表里的名字起的（`gpu-verification-server/tests/card_lease_on_a_temporary_library.rs` 第 1049 到 1051 行）。它拦得住的是误用（别的测试进程顺手借卡），拦不住有意冒名。攻方另两条路（借到之后 exec 成别的程序、不经 `lease take` 直接认领请求键）同理。收不收严交用户。
8. **谁拿到借方的进程号与启动刻，都能 `lease return` 放掉它的两把键，而 `lease list` 会把这两样打出来**（攻方第 8 问后一半）。持有者身份是 `card-lease/<这一台>/<进程号>-<启动刻>`（`card_lease.rs` 第 320 行），`lease list` 的 `holder=` 原样打（第 661 行）；`lease return` 的身份只认环境变量。同属合作前提下的缺口，与第 7 条一起交用户。
9. **排队的次序用的是墙上钟**（攻方第 9 问）。`card_lease.rs` 第 356、357 行 `SystemTime::now().duration_since(UNIX_EPOCH)`：时钟往回拨之后起等的排到前面。同一台上的事，换成不回拨的钟就行。
10. **借方进程被停住（SIGSTOP）时算在跑，卡借出去没有上限**（攻方第 9 问）。`gpu-verification-server/src/service_process.rs` 第 122 到 134 行的判法只看进程在不在、启动刻对不对，不看状态；续租进程照它一直续。这一格没有复跑读数（探针没编过），照主 agent 读代码判。
11. **`lease list` 把身份不以 `card-lease/` 起头的持有者一律写成服务**（攻方第 10 问）。`card_lease.rs` 第 628 行按前缀分。没有复跑读数，照读代码判。过期的不列、服务重起、列认领截断三格不成立。
12. **`control install` 停了两台之后被信号打断，两台都停着、没人被告知**（攻方第 11 问）。`gpu-verification-server/control` 里没有接信号的代码（`grep -n "signal\.\|SIGTERM\|KeyboardInterrupt"` 零命中）。旧二进制被盖掉、`--dry-run` 真停服务两格不成立。
13. **两条借卡开关点名的格有一边重叠**（攻方第 13 问）。复跑：先来后到那一格在 card-lease-keys-held-by-another-taken-as-claimed 下也红；反过来 `two_borrowers_on_one_host_get_the_cards_one_after_the_other` 在 card-lease-waiting-order-ignored 下是 ok。`.claude/singlefs-ai-sop/rules/show-me-test.md`「每一个弄坏开关都必须会红」要的是各自红在点名那一格，这一条满足；重叠本身记下，不改。

不成立、不采或只当线索的：

- 辩方第 1 问：三处改法做的是第一轮判决写的那件事，文件头四种情形与代码逐句对得上：成立。重开设备、真卡那一份 `after_a_panic` 的外壳没在卡上执行过，仍是第一轮判决第 6 条那笔欠的测试，范围比那时小（纯函数 `device_after_a_panic` 有了测试）。
- 辩方第 2 问其余各条（卡 0 显存不够那一刻卡 1 的账；判两次、写错卡、写错流；多机撞键）：仍不成立。
- 辩方第 4 问「一直读不成的台没人知道」「判不了行按块发、一条坏流一堆条」：前一条与判决第 12 条同根（服务没在跑不算毛病），后一条并进判决第 4 条的改法。
- 攻方第 5 问其余三格（等不回来、旧设备压着显存、连着显存不够不告诉谁）：前两格核不动，写在「一、腿与核查」；第三格（设备没丢的显存不够没有次数上限、只打接住那一行）属实但不改：那一行已在 journal 里，卡照常判，用户定的是不影响好卡，没定要告警。
- 攻方第 9 问 `lease take` 停住或被杀至多挡一个认领租期、等号不动服务那一半：不成立。
- 攻方第 12 问：键名改一个字自证格会红，成立；值写错红得晚，没逐支核，当线索，不为它改配置判法（值归私有清场脚本判）。
- 辩方腿判第一轮「判重」两处（第 11 条的 README 一句是第 5 条的文档面；第 6 条在坐实与不成立里各记一次）：属实，不影响改法。第一轮判决第 2 条里「判段只问 `control serves`」已随那条子命令删掉而过时。

## 三、改法

下面七条都是这一轮判决里推的，攻过零轮；写进服务代码之前先做成替身判器或临时判定库上「没有这一改动就红」的用例与弄坏开关，主 agent 经任务服务跑出两臂读数。

1. 判决第 1 条（零轮）：放回的核对组优先等它显存不够的那张卡腾空；只在那张卡坏了或装不下时才给别的卡，给别的卡时不为它换出驻留的判器。
2. 判决第 2 条（零轮）：派一截之前只要这一块在这张卡上显存不够过就不派给它（不看这一块此刻在不在等）；别的卡都装不下时才回到那张卡腾空单独判。
3. 判决第 3 条（零轮）：那一次提交没等成时再问一次设备怎样：因显存不够丢了走重开，设备没丢照「腾空的卡上又显存不够」处置这一波的块，只有为别的原因丢了才报卡坏。
4. 判决第 4、5 条（零轮）：判不了的行与 `control check`、告警的出路照那一行的 `claim=` 写（放了才说放给另一台）；告警的键去掉进程号、同一条流并成一条，服务重起之后同一块再判不了不重发。
5. 判决第 6 条（零轮）：头一回读有台没读成时，不把那一台记过的「发过」当成好了：`gpu_cards_failed_known` 按台记。
6. 判决第 9、11 条（零轮）：等号的排序段换成这一台开机以来不回拨的钟；`held_by` 认整个 `card-lease/<台>/<进程号>-<启动刻>` 的形状，服务的身份照它自己的认，两样都不是写 other。
7. 判决第 12 条（零轮）：`control install` 停了之后接 SIGTERM、SIGHUP、SIGINT，被打断时把停掉的几台起回来再退。

8. 用户定的两条（零轮）：借卡客户端收严——`lease take`、续租与 `lease return` 都核起它的进程就是借方变量点名的那一个或它的子孙，续租时照样核可执行文件名；借方进程状态是停住的当不在，续租进程放掉两把键。

被打中那一支的读数是「一、腿与核查」里两腿探针那两趟；改了之后那一臂由实现员补丁里的用例与弄坏开关在副本上跑出，合入之前记进调度记录。
A/B 读数：任务 25854、任务 25864

## 交用户的

- 借卡白名单收不收严（判决第 7、8 条）：今天只认可执行文件名，拦得住误用、拦不住有意冒名；别人拿到进程号与启动刻能还掉借方的卡。候选：甲，维持（本机只有自己的 agent 与脚本，白名单只防顺手借卡），`lease list` 的 `holder=` 照打；乙，客户端收严（核起 `lease take` 的进程就是变量点名的那一个或它的子孙、续租时照样核文件名、`lease return` 同样核），不动判定库服务；丙，在判定库服务那一侧认身份（要给 prune-server 加认身份的办法，改动大）。主 agent 推荐甲。用户 2026-11-21 弹窗选「客户端收严」，照乙做。
- 借方进程停住时卡借出去没有上限（判决第 10 条）：候选：甲，不设上限（E202 一趟多久由它自己定）；乙，状态是停住的借方当不在、到点放键。主 agent 推荐乙。用户 2026-11-21 弹窗选「停住的借方当不在 (Recommended)」，照乙做。
- 判不了的块在服务重起后再试一回（判决第 4 条）：改法第 4 条只管不重发告警；要不要连「再试」也拦掉（在判定库里记一笔、重起后照它不取）交用户定。主 agent 推荐不拦：重起多半是换了版，再试一回是对的。用户 2026-11-21 弹窗选「不拦，重起后再试 (Recommended)」。
- 欠的测试：真卡上跑 `after_a_panic` 与重开设备（第一轮判决第 6 条，代跑接口待定）；攻方腿核不动的两句 wgpu 行为要在真卡上量。

## 这一轮被判过的文件

- research/prompts/_gpu-oom-card-governance-r2-body.md：正文十三问逐问有判定；「实现今天的样子」里改法第 1 条写成「做那一次提交」属实，但那不是空提交（判决第 3 条）。
- gpu-verification-server/src/card_worker.rs：判了文件头四种情形与 `card_worker_loop` 逐句对照，成立；那一次提交没等成就报卡坏（判决第 3 条）。
- gpu-verification-server/src/gpu_judges_on_a_card.rs：判了 `after_a_panic` 三处改法做的是第一轮判决写的事，成立；重开设备与外壳仍没在卡上执行过。
- gpu-verification-server/src/service_loop.rs：判了撤回、放回、报拒各支；放回的核对组换光好卡的判器（判决第 1 条）、剩下的几截派回显存不够过的卡（判决第 2 条）。
- gpu-verification-server/src/batch_scheduling.rs：判了大卡坏了当场报拒用的是开卡那一刻的配置额度、之后登记的流走同一个判法，成立。
- gpu-verification-server/src/card_memory_ledger.rs：判了核对器借显存的算法与调用处（三个变四个），与第一轮改法第 5 条一致。
- gpu-verification-server/src/serving_from_the_library.rs：判了判不了行的字段、放认领、进程里记名；重起之后再试再告警（判决第 4 条）。
- gpu-verification-server/src/break_switches.rs：判了这几批新开关各自红在点名的格；两条借卡开关一边重叠（判决第 13 条）。
- gpu-verification-server/src/card_lease.rs：判了白名单认法、等号与次序、`lease list` 三种 `held_by`、`lease return` 的身份；判决第 7 到 11 条。
- gpu-verification-server/src/card-lease-borrowers.tsv：判了表里只有 E202 那一项、认的是文件名；认法的强弱归判决第 7 条。
- gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs：判了它的替身判器罩到的情形；判决第 1、2 条两种情形要补用例（两腿的探针接在它末尾跑出的读数）。
- gpu-verification-server/tests/batch_scheduling_requirements.rs：判了「只有它装得下的块当场报拒」那几条用例与代码一致。
- gpu-verification-server/tests/card_lease_on_a_temporary_library.rs：判了先来后到、被杀让位、`lease list` 三格与白名单三格；借方是改名的 `sleep`（判决第 7 条），停住的借方与直接认领的持有者两种情形要补用例。
- gpu-verification-server/README.md：判了借卡、日志、起停几节与代码一致；`control check` 那一句出路随判决第 5 条改。
- gpu-verification-server/control：判了 `cards-failed` 的没读成行、`lease list` 原样转、install 先停再清场再起；写死「已放给另一台」（判决第 5 条）、不接信号（判决第 12 条）。
- task-server/task_server_lib_configuration.py：判了 `gpu_service_problems_merged` 留着没读成那一台上一回读到的、卡号取不到报错，成立。
- task-server/task_server_scheduler_notifying.py：判了 `read_gpu_cards_failed` 与 `watch_health` 的去重；重起后头一回读有台没读成的口子（判决第 6 条）。
- task-server/task_server_lib_constants.py：判了几种行的前缀与 control 逐字相同。
- task-server/task_server_constants.py：判了判不了的块那条告警的出路文字；写死放给另一台随判决第 5 条改。
- scripts/crash/crash-point-replay-shard-configuration-check.sh：判了九个清场键只认键名、键名改一个字自证格会红，成立；值写错红得晚只当线索。

## 回看决策

不涉及决策：这一轮判的是 GPU 批处理验证服务、借卡通道与任务服务告警的运行行为，不推翻、不确立 `.claude/kb/decisions/` 下的任何一条。
