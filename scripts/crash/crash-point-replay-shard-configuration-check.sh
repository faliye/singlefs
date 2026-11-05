#!/usr/bin/env bash
# 本地多机配置的判法：在不在、键齐不齐、写得对不对、第二台连不连得上、它上面有没有 cargo，双机与 GPU 两个开关开没开。
# 调用方：scripts/hosts/peer-host-lib.sh 的 peer_host_load_configuration（scripts/hosts/multi-host-run.sh --peer、scripts/mutation/mutation-shard-run.sh 经它读）、
#   scripts/hosts/gpu-card-health.py 与几份实验装置（--emit-hosts）。挑台、上限、分片、分卡与清场都归 task-server（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03），
#   这一份只判配置，不分活、不挑台。
# 按台的三个键 CARGO_BIN_DIRECTORY_BY_HOST、BUILD_ROOT_BY_HOST、SLICE_MEMORY_CAP_BY_HOST 都要写，HOSTS 里每一台恰好一项（文件头「按台的三个键」）；
# GPU_KERNEL_SERIAL_COMPILE_HOSTS 可以不写（文件头「GPU 内核一个一个编的台」）。
#
#   crash-point-replay-shard-configuration-check.sh [--configuration <配置文件>] [<仓根>]
#       --configuration 可与下面几种用法合用，写在别的参数前面（文件头「两台对称」）。
#       退 0 双机：开（stdout 一句：双机与 GPU 开没开、配置在哪、第二台是谁）；
#       退 3 双机：关（配置判得过，但 ENABLE_ACROSS_MACHINES 不是 1：调用方照单机跑；stdout 一句「双机：关」与为什么，不连第二台）；
#       退 1 判不过（stdout 一句原因，下一行是出路）。
#   crash-point-replay-shard-configuration-check.sh --emit-assignments [<仓根>]
#       判法同上；双机开着时 stdout 改打配置里每个键的 bash 赋值（printf %q，调用方 eval；两个开关键没写的打成 0），不打那一句；
#       双机关着时照样打「双机：关」那一句、退 3，不打赋值。
#   crash-point-replay-shard-configuration-check.sh --emit-hosts [<仓根>]
#       判法同上，只是不连任何一台（第二台也不连）。双机开着时 stdout 打 --emit-assignments 那几行，另打除本机外的各台（主机数组里不是自己的各项，次序同数组）：
#       MULTI_HOST_COUNT=<台数>，以及每台 MULTI_HOST_<k>_SSH_HOST（那一项的地址）、MULTI_HOST_<k>_REPOSITORY_DIRECTORY、MULTI_HOST_<k>_CARGO_BIN_DIRECTORY、
#       MULTI_HOST_<k>_MEMORY_CAP（依次取 BUILD_ROOT_BY_HOST、CARGO_BIN_DIRECTORY_BY_HOST、SLICE_MEMORY_CAP_BY_HOST 里那一台地址的那一项）、
#       MULTI_HOST_<k>_IS_PEER（头一台是 1，别的是 0），k 从 2 起。
#       赋值里带 NFS_SHARED_ROOTS（下面「NFS 共享的根」）与主机数组的三个赋值（下面「主机数组」）。
#   crash-point-replay-shard-configuration-check.sh --emit-addresses [<仓根>]
#       判法同上（整份判，不连任何一台），判得过就打主机数组的三个赋值（HOSTS、THIS_HOST_ADDRESS、PEER_HOST_ADDRESS）与 GPU_KERNEL_SERIAL_COMPILE_HOSTS（没写打空串），
#       printf %q，退 0，不看双机开关：取这一台与第二台的地址用（scripts/hosts/multi_host_addresses.py 经它取；双机关着也照打）。判不过退 1。
#   crash-point-replay-shard-configuration-check.sh --selftest
#       拿临时目录里的假配置与假 ssh 走一遍各格，末尾一轮逐个打开下面点名「本份 --selftest」的弄坏开关、核它红在点名的格。退 0 全对，1 有格判错。
#   别的 -- 起头的参数是用法错，退 2。
#
# 两个开关键只在这一份里定义（别处照这里读，不另写一份）：
#   ENABLE_ACROSS_MACHINES  双机：写 1 才开；没写或写 0 算关（用户 2026-09-28 定：不设参数不主动开）。关着时不连第二台、退 3。
#   ENABLE_GPU              GPU：写 1 算开，没写或写 0 算关；这里只判写法、报开没开，用不用它归各调用方。
#   两个键只许写 0 或 1，写别的值判不过（退 1，说清是哪个键）。别的键缺一个都判不过，与开关开没开无关。
#
# 主机数组 HOSTS=<地址> <地址>[ <地址>…]（要写；用户 2026-11-02 定「多机就设置 一个数组 把多机的ip写进去」）：各台有线直连段的 IP，空格分隔，至少两项、不许重复，
#   每项是 IPv4 的点分写法或 IPv6（照 ip -o addr show 打的写法）；此刻只认两台：恰好一项是自己、恰好一项是第二台。哪一台是自己照这一台网卡上的地址认（scripts/hosts/peer-host-lib.sh 的 peer_host_resolve_addresses，
#   python 一侧 scripts/hosts/multi_host_addresses.py 的 resolve 经这一份取），不写死主机名：数组里恰好一项是这一台网卡上的地址，命中 0 项或不止 1 项判不过；
#   第二台是数组里不是自己的那一项（PEER_HOST_ADDRESS，--emit-hosts 的 MULTI_HOST_2）。ssh 直接连那个地址，用户名归各台自己的 ~/.ssh/config。
#   --emit-assignments、--emit-hosts 打 HOSTS（原样）、THIS_HOST_ADDRESS（自己那一项）、PEER_HOST_ADDRESS。两台判同一份配置时各认出自己那一项，判的结论相同，
#   第二台是谁两边互为对方。
#   判定库服务的地址 PRUNE_SERVER_TCP_ADDRESS=<地址>:<端口>（IPv6 写 [<地址>]:<端口>；可以不写）写了的，地址要是主机数组里的一项。
#
# 按台的三个键（要写；模板 multi-host.env.example）：值是空白分隔的若干项 <地址>=<值>，从第一个 = 切；HOSTS 里每一台恰好一项、
#   每项的地址都在 HOSTS 里：多一项（同一地址写两遍，或地址不在 HOSTS 里）、少一项（HOSTS 里有一台没写）、某项没有 = 或值是空的都判不过。
#   用法按「要的那一台的地址」取那一项：ssh 去对面查取对面那一项，这一台自己用取自己那一项。
#   CARGO_BIN_DIRECTORY_BY_HOST  那一台的 ~/.cargo/bin（绝对路径）。双机开着、不带 --emit-hosts 时两台都查：这一台照自己那一项本地 test -x <目录>/cargo，
#                                第二台照它那一项经 ssh test -x（两台判同一份配置查的是同一对目录，结局相同）。
#   BUILD_ROOT_BY_HOST           那一台本地盘上每一趟编译目录的根（绝对路径；scripts/hosts/multi-host-run.sh 在它下面建每一趟的目录，task-server 按它所在盘判余量）。
#   SLICE_MEMORY_CAP_BY_HOST     那一台 singlefs-heavy.slice 的总上限（scripts/hosts/run-with-memory-cap.sh 拿它设 slice 的 MemoryMax；task-server 派那一台不超过它），
#                                写法照 run-with-memory-cap.sh：正整数加 K / M / G / T，单位必写。
#   --emit-assignments、--emit-hosts 把三个键原样打出（两台打的逐字相同），--emit-hosts 另按台取那一项打 MULTI_HOST_<k>_*。
#
# GPU 内核一个一个编的台 GPU_KERNEL_SERIAL_COMPILE_HOSTS=<地址>[ <地址>…]（可以不写；不写或写空串，两台都一起编）：空白分隔，每项是 HOSTS 里的一项、不许重复，
#   不合的判不过。用它的一方照 --emit-addresses 打的 THIS_HOST_ADDRESS 在不在这个值里，定这一台编判器内核时一个一个编还是一起编
#   （scripts/crash/crash-amplification-two-hosts.sh 的每一片）；--emit-addresses、--emit-assignments、--emit-hosts 把它原样打出（没写打空串）。
#
# 显卡的两个键（可以不写；用户 2026-10-10 定多机配置合成一份，私有显卡配置并进这一份）：GPU_CARDS_BY_PRIORITY、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED。
#   这里只认这两个键名、不判值（task-server 照 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 挑卡；值每项写 <HOSTS 里的地址>:<卡号>:<额度>，从右切两刀，
#   判法在 crates/singlefs-verification/src/gpu_card_configuration.rs 与 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_card_list_problem），
#   --emit-assignments、--emit-hosts 不打它们。
#
# NFS 共享的根 NFS_SHARED_ROOTS=<绝对路径>[:<绝对路径>…]（要写、不许空；ENABLE_ACROSS_MACHINES 写 0 也要写）：两台在同一绝对路径上共享的目录，跑出来的结果两台都读得到靠它
#   （用户 2026-11-01 定「NFS 的配置要写在 task-server 的 readme 中。不要遗漏了。这部分也要作为检测项目。没有 nfs 不启动。要求用户配置。配置相关来自多机配置的配置文件中 不允许增加额外的配置文件」）。
#   这里判写法与本机的目录：冒号分隔、至少一段、每段以 / 起头、不以 / 结尾、不含空白、不许空段，不合的判不过（退 1），点名第几段；每段还要是这一台已在的目录
#   （两台挂在同一绝对路径上，派到哪一台判的都是同一个结论）。导出了没有、对方挂上没有、两向读写通不通不在这里判，归 task-server
#   （服务起来时判，control check 与 /health 也判；写法与核法在 task-server/README.md「## NFS 共享」）。--emit-assignments、--emit-hosts 把它原样打出。
#
# 共享数据根 SHARED_DATA_ROOT=<绝对路径>（这里认这个键名、不判值）：两台共享的数据根（工作根、上次红的清单、运行记录这几样所在的那一层），
#   环境里没设 SINGLEFS_DATA_ROOT 时 .claude/hooks/lib_work_root.py 照它取；写法（绝对路径、落在 NFS_SHARED_ROOTS 某一段下）由那份库判，没写而又要用时那份库报错、不补缺省。
#   --emit-assignments、--emit-hosts 不打它。
#
# task-server 的四个键 TASK_SERVER_STATE_DIRECTORY、TASK_SERVER_RUNTIME_DIRECTORY、TASK_SERVER_SIGNING_KEY_FILE、TASK_SERVER_BUILD_DIRECTORY（可以不写；这里只认键名、不判值）：
#   task-server 的状态目录、运行目录、签名密钥文件与编译目录，意思与缺省见仓根 multi-host.env.example，读法与判法归 task-server/。--emit-assignments、--emit-hosts 不打它们。
#
# 两台对称（records/2026-10-31-task-server巡检.md 问题 94；任务 3925：同一份配置、同一条命令派到两台，结局要相同）：哪一台是自己只照 HOSTS 与这一台网卡上的地址认，
#   HOSTS 里另一项就是第二台；不读随台变的角色变量。两台判同一份配置，退出码相同（双机开着时各连对方），
#   只有那一句里的第二台、--emit-* 打的 THIS_HOST_ADDRESS 与 PEER_HOST_ADDRESS 互为对方。--configuration <配置文件> 给了就判这一份，不看 SINGLEFS_MULTI_HOST_CONFIG、不找主工作树。
# 配置文件：${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}。主工作树的根按 git common-dir 取（在 HEAD + 暂存区的
# 临时 worktree 里跑也读主工作树那一份）；文件 git 忽略、不进仓，模板是仓根的 multi-host.env.example。
# 写法：一行一个 KEY=值（值不做 shell 展开；两头成对的单引号或双引号去掉一层），# 起头的行与空行不算；认不出的行、不认得的键、
# 缺键都判不过。必写的是 CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS 那四个、HOSTS、按台的三个键与 NFS_SHARED_ROOTS，意思见 multi-host.env.example；两个开关键见上。
# 清场一对（QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND）与复原一对（QUIESCE_START_COMMAND、QUIESCE_STARTED_CHECK_COMMAND）各自要么都写、要么都空；
#   两对之间不要求成对：清场之后都不复原，复原那一对只给用户手动复原用，写不写都判得过。
# 弄坏开关 MULTI_HOST_CONFIGURATION_BREAK=<项>（只在自证里认：第一个参数是 --selftest，或祖先进程里有命令行带 --selftest 的算自证起的；平常跑设了就在读参数之前退 2、点名它只在自证里认）。
# 逐项写开关名、弄坏哪一处、打开后哪一份自证的哪几格红（「本份 --selftest 的」那几格由这一份末尾那一轮核；别的写的是调用方自证的格名，括号里抄细节的句首）：
#   MULTI_HOST_CONFIGURATION_BREAK=switch-off-is-on  ENABLE_ACROSS_MACHINES 没写或写 0 也按开判
#       本份 --selftest 的 complete-configuration-off、gpu-card-keys-accepted、data-root-key-accepted、task-server-keys-accepted、prune-address-in-hosts-accepted、serial-compile-hosts-accepted
#       multi-host-run.sh 的「Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录」（「退 0，命令跑了」）
#   MULTI_HOST_CONFIGURATION_BREAK=switch-value-unchecked  两个开关键写了 0、1 之外的值也不报错
#       本份 --selftest 的 switch-value-refused
#   MULTI_HOST_CONFIGURATION_BREAK=switch-missing-defaults-zero  两个开关键没写时照改前当 0（不兜底之前的样子）
#       本份 --selftest 的 switch-missing-refused
#   MULTI_HOST_CONFIGURATION_BREAK=gpu-cards-optional  ENABLE_GPU=1 而显卡的两个键没写也判得过
#       本份 --selftest 的 gpu-on-cards-required
#   MULTI_HOST_CONFIGURATION_BREAK=by-host-keys-optional  按台的三个键没写也判得过
#       本份 --selftest 的 by-host-keys-required
#   MULTI_HOST_CONFIGURATION_BREAK=by-host-duplicate-accepted  按台的键里同一地址写两遍也判得过（取后写的那一项）
#       本份 --selftest 的 by-host-item-duplicated-refused
#   MULTI_HOST_CONFIGURATION_BREAK=by-host-missing-accepted  按台的键里 HOSTS 有一台没写也判得过
#       本份 --selftest 的 by-host-item-missing-refused
#   MULTI_HOST_CONFIGURATION_BREAK=by-host-outside-hosts-accepted  按台的键里地址不在 HOSTS 里的项也判得过
#       本份 --selftest 的 by-host-address-outside-hosts-refused
#   MULTI_HOST_CONFIGURATION_BREAK=by-host-value-form-unchecked  按台的键每项的写法（有 =、值不空、目录是绝对路径、上限带单位）不判
#       本份 --selftest 的 by-host-item-form-refused、by-host-value-form-refused
#   MULTI_HOST_CONFIGURATION_BREAK=unknown-keys-accepted  不认得的键也判得过（不报认不得）
#       本份 --selftest 的 unknown-key-refused
#   MULTI_HOST_CONFIGURATION_BREAK=cargo-item-of-the-wrong-host  ssh 去第二台查 cargo 时取这一台自己那一项（两台对称之前问错台的样子）
#       本份 --selftest 的 cargo-checked-with-each-host-item、cargo-missing-judged-the-same-on-both-hosts
#   MULTI_HOST_CONFIGURATION_BREAK=this-host-cargo-unchecked  不查这一台自己那一项下的 cargo（只经 ssh 查第二台）
#       本份 --selftest 的 cargo-missing-judged-the-same-on-both-hosts
#   MULTI_HOST_CONFIGURATION_BREAK=emit-hosts-this-host-items  --emit-hosts 的 MULTI_HOST_<k>_* 取这一台自己那一项（不按那一台的地址取）
#       本份 --selftest 的 emit-hosts-from-by-host-items
#   MULTI_HOST_CONFIGURATION_BREAK=serial-compile-hosts-unknown  GPU_KERNEL_SERIAL_COMPILE_HOSTS 当不认得的键
#       本份 --selftest 的 serial-compile-hosts-accepted、serial-compile-hosts-outside-refused、serial-compile-hosts-duplicate-refused、emit-addresses-serial-compile-hosts
#   MULTI_HOST_CONFIGURATION_BREAK=serial-compile-hosts-unchecked  GPU_KERNEL_SERIAL_COMPILE_HOSTS 的项不核在不在 HOSTS 里、重没重复
#       本份 --selftest 的 serial-compile-hosts-outside-refused、serial-compile-hosts-duplicate-refused
#   MULTI_HOST_CONFIGURATION_BREAK=serial-compile-hosts-not-emitted  --emit-addresses、--emit-assignments、--emit-hosts 不打 GPU_KERNEL_SERIAL_COMPILE_HOSTS
#       本份 --selftest 的 emit-addresses-serial-compile-hosts
#   MULTI_HOST_CONFIGURATION_BREAK=data-root-key-unknown  SHARED_DATA_ROOT 当不认得的键（认它之前的样子）
#       本份 --selftest 的 data-root-key-accepted
#   MULTI_HOST_CONFIGURATION_BREAK=task-server-keys-unknown  task-server 的四个键当不认得的键（认它们之前的样子）
#       本份 --selftest 的 task-server-keys-accepted
#   MULTI_HOST_CONFIGURATION_BREAK=gpu-card-keys-unknown  显卡的两个键当不认得的键（并进多机配置之前的样子）
#       本份 --selftest 的 gpu-card-keys-accepted
#       scripts/crash/crash-amplification-item.sh --selftest 的第 22 格（「多机配置写了显卡的两个键却判不过…」）
#   MULTI_HOST_CONFIGURATION_BREAK=peer-is-this-machine-unmarked  没有调用方自证的标记也认只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（不连第二台）
#       本份 --selftest 的 peer-is-this-machine-needs-the-mark
#   MULTI_HOST_CONFIGURATION_BREAK=nfs-roots-optional  NFS_SHARED_ROOTS 没写或写成空串也判得过（改成必写之前的样子）
#       本份 --selftest 的 nfs-roots-required
#   MULTI_HOST_CONFIGURATION_BREAK=nfs-roots-form-unchecked  NFS_SHARED_ROOTS 的写法（绝对路径、不以 / 结尾、无空白、无空段）不判
#       本份 --selftest 的 nfs-roots-form
#   MULTI_HOST_CONFIGURATION_BREAK=nfs-roots-exist-unchecked  不判每段是不是这一台已在的目录
#       本份 --selftest 的 nfs-roots-exist
#   MULTI_HOST_CONFIGURATION_BREAK=nfs-roots-not-emitted  --emit-assignments、--emit-hosts 不打 NFS_SHARED_ROOTS
#       本份 --selftest 的 emit-assignments-nfs-roots、emit-hosts-nfs-roots
#   MULTI_HOST_CONFIGURATION_BREAK=hosts-optional  HOSTS 没写也判得过（第二台没处取，PEER_HOST_ADDRESS 打成空的）
#       本份 --selftest 的 hosts-required
#   MULTI_HOST_CONFIGURATION_BREAK=hosts-count-unchecked  HOSTS 只写一项、写了重复的项、不是自己的不止一项也判得过
#       本份 --selftest 的 hosts-single-refused、hosts-duplicate-refused、hosts-three-refused
#   MULTI_HOST_CONFIGURATION_BREAK=hosts-self-unchecked  不照这一台网卡上的地址认自己，一律当数组头一项是自己（写死次序的样子）
#       本份 --selftest 的 cargo-checked-with-each-host-item、emit-addresses-when-off、hosts-peer-is-the-other-item、hosts-self-absent-refused、hosts-self-twice-refused、hosts-three-refused
#   MULTI_HOST_CONFIGURATION_BREAK=prune-address-unchecked  PRUNE_SERVER_TCP_ADDRESS 的地址不核在不在主机数组里
#       本份 --selftest 的 prune-address-outside-hosts-refused
#   MULTI_HOST_CONFIGURATION_BREAK=configuration-argument-ignored  不认 --configuration，照 SINGLEFS_MULTI_HOST_CONFIG 与主工作树找配置（只照环境判的样子）
#       本份 --selftest 的 arguments-judge-the-same-on-both-hosts
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 时不连第二台、不查它的 cargo（这一份与调用方的 --selftest 用，
# 那时「第二台」是本机上的另一个目录）。只在调用方的自证里认，认法同 scripts/hosts/peer-host-lib.sh 的 peer_host_inside_selftest：
# PEER_HOST_SELFTEST_MARK 指的文件在、首行是 PEER_HOST_SELFTEST_MARK_CONTENT（自证建、export）；没有标记设了它照连第二台。
#
# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判
# run-condition: command git bash ip
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
crash_point_replay_shard_check_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# 主机数组里认哪一项是自己（文件头「主机数组」）：共用取法在 peer-host-lib.sh 的 peer_host_resolve_addresses（那份库只定义函数与几个全局变量）
source "$crash_point_replay_shard_check_script_directory/../hosts/peer-host-lib.sh"

CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS=(QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)
MULTI_HOST_BY_HOST_KEYS=(CARGO_BIN_DIRECTORY_BY_HOST BUILD_ROOT_BY_HOST SLICE_MEMORY_CAP_BY_HOST)  # 要写，HOSTS 里每一台恰好一项 <地址>=<值>（文件头「按台的三个键」）
MULTI_HOST_BY_HOST_DIRECTORY_KEYS=(CARGO_BIN_DIRECTORY_BY_HOST BUILD_ROOT_BY_HOST)  # 值是那一台上的绝对路径
MULTI_HOST_BY_HOST_CAP_KEY=SLICE_MEMORY_CAP_BY_HOST  # 值是那一台 slice 的总上限，写法照 MEMORY_CAP_PATTERN
MULTI_HOST_SERIAL_COMPILE_KEY=GPU_KERNEL_SERIAL_COMPILE_HOSTS  # 可以不写：空白分隔的地址，每项在 HOSTS 里（文件头「GPU 内核一个一个编的台」）
MULTI_HOST_SWITCH_KEYS=(ENABLE_ACROSS_MACHINES ENABLE_GPU)  # 要写，各写 0 或 1；没写判不过（用户：「配置配错 没有配置 就报错 不允许自动补齐。 一定要明确的配置。 不兜底」）
MULTI_HOST_GPU_CARD_KEYS=(GPU_CARDS_BY_PRIORITY GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)  # ENABLE_GPU=1 时两个都要写（这里不判值），=0 时可以不写（文件头「显卡的两个键」）
MULTI_HOST_SERVICE_KEYS=(PRUNE_SERVER_TCP_ADDRESS)  # 可以不写：判定库服务 prune-server 的 TCP 地址，这里只核地址在主机数组里（文件头「主机数组」；别的读法与判法归 prune-server/）
MULTI_HOST_HOSTS_KEY=HOSTS  # 要写：各台有线直连段的 IP，空格分隔，至少两项（文件头「主机数组」）
MULTI_HOST_ADDRESS_PATTERN='^(([0-9]{1,3}[.]){3}[0-9]{1,3}|[0-9A-Fa-f]*:[0-9A-Fa-f:.]*)$'   # 主机数组的一项：IPv4 点分或 IPv6
MULTI_HOST_DATA_ROOT_KEY=SHARED_DATA_ROOT  # 可以不写，这里不判值：共享数据根（文件头「共享数据根」；读法与判法归 .claude/hooks/lib_work_root.py）
MULTI_HOST_TASK_SERVER_KEYS=(TASK_SERVER_STATE_DIRECTORY TASK_SERVER_RUNTIME_DIRECTORY TASK_SERVER_SIGNING_KEY_FILE TASK_SERVER_BUILD_DIRECTORY)  # 可以不写，这里不判值（文件头「task-server 的四个键」；读法与判法归 task-server/）
MULTI_HOST_NFS_KEY=NFS_SHARED_ROOTS  # 要写：两台在同一绝对路径上共享（NFS）的目录，冒号分隔（文件头「NFS 共享的根」）
NFS_README_POINTER='task-server/README.md「## NFS 共享」'
MEMORY_CAP_PATTERN='^[1-9][0-9]*[KMGT]$'
configuration_break="${MULTI_HOST_CONFIGURATION_BREAK:-}"
# 弄坏开关只在自证里认（文件头「弄坏开关」）：第一个参数是 --selftest，或祖先进程里有一个命令行带 --selftest（调用方的自证设着开关起它；沿父进程往上至多 16 层，只读 /proc）；
# 平常跑设了 MULTI_HOST_CONFIGURATION_BREAK（不管值是什么）就在读参数之前退 2，点名它只在自证里认。
configuration_inside_selftest() { # [<第一个参数>]：这一趟是不是自证起的（第一个参数是 --selftest，或祖先进程的命令行带 --selftest）
  local ancestor=$PPID
  [[ "${1:-}" == --selftest ]] && return 0
  for _ in {1..16}; do
    [[ "$ancestor" =~ ^[0-9]+$ ]] && (( ancestor > 1 )) || return 1
    if tr '\0' ' ' < "/proc/$ancestor/cmdline" 2>/dev/null | grep -q -- '--selftest'; then return 0; fi
    ancestor="$(sed 's/^.*) //' "/proc/$ancestor/stat" 2>/dev/null | awk '{ print $2 }')"
  done
  return 1
}
# 只供自证：CONFIGURATION_SELFTEST_LOCAL_ADDRESSES 换掉这一台网卡上的地址（文件头「主机数组」那几格造「这一台占两项」）；不是自证起的设了照 ip 读、不认它
configuration_local_addresses=""
if [[ -n "${CONFIGURATION_SELFTEST_LOCAL_ADDRESSES:-}" ]] && configuration_inside_selftest "${1:-}"; then configuration_local_addresses="$CONFIGURATION_SELFTEST_LOCAL_ADDRESSES"; fi
if [[ -n "$configuration_break" ]]; then
  if ! configuration_inside_selftest "${1:-}"; then
    echo "  ✗ MULTI_HOST_CONFIGURATION_BREAK 只在自证里认（弄坏开关只给 --selftest 证红用），这一趟不是自证起的，一步都没判" >&2
    echo "     → 怎么办：去掉 MULTI_HOST_CONFIGURATION_BREAK 再跑；要证红就跑 MULTI_HOST_CONFIGURATION_BREAK=<项> bash scripts/crash/crash-point-replay-shard-configuration-check.sh --selftest（文件头「弄坏开关」那一段写了各开关红在哪一份自证的哪几格）" >&2
    exit 2
  fi
fi

# ── 自证（文件头 --selftest）：每格起这一份自己，拿临时目录里的假配置判；PATH 最前面放一个假 ssh（被起一次记一行、退 255），判的那几格都不该连第二台
configuration_selftest() {
  local work self="$crash_point_replay_shard_check_script_directory/crash-point-replay-shard-configuration-check.sh" failed=() cells=0
  work="$(mktemp -d "${TMPDIR:-/tmp}/configuration-check-selftest.XXXXXX")" || { echo "  ✗ 自证建不了临时目录"; echo "     → 怎么办：看 TMPDIR（${TMPDIR:-/tmp}）能不能写，再跑"; return 1; }
  configuration_selftest_work="$work"
  trap 'rm -rf "${configuration_selftest_work:?}"' EXIT
  mkdir -p "$work/bin" "$work/bin-running" "$work/tree" "$work/cargo-this" "$work/cargo-peer"
  # 调用方自证的标记（文件头「只供测试的开关」）：只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 只在它在时认
  printf '%s\n' "$PEER_HOST_SELFTEST_MARK_CONTENT" > "$work/peer-host-selftest-mark"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "ssh $*" >> "${CONFIGURATION_SELFTEST_SSH_LOG:?}"' 'exit 255' > "$work/bin/ssh"
  chmod +x "$work/bin/ssh"
  # 查 cargo 那几格的假 ssh：被起一次记一行，把要在对面跑的那条命令（末一个参数）就地跑（两台的目录都在这一台的临时目录里）
  printf '%s\n' '#!/usr/bin/env bash' 'echo "ssh $*" >> "${CONFIGURATION_SELFTEST_SSH_LOG:?}"' 'exec bash -c "${*: -1}"' > "$work/bin-running/ssh"
  chmod +x "$work/bin-running/ssh"
  printf '%s\n' '#!/usr/bin/env bash' 'exit 0' > "$work/cargo-this/cargo"
  cp "$work/cargo-this/cargo" "$work/cargo-peer/cargo"
  chmod +x "$work/cargo-this/cargo" "$work/cargo-peer/cargo"
  # 主机数组（文件头「主机数组」）：127.0.0.1 在每一台的网卡上（lo），192.0.2.x 是文档用的地址段、不在任何一台上：样本在两台判得一样，不靠假的网卡地址
  # 自证造的假配置：写进文件的是一行行文字（每行一个引号里的字面量），不是这个脚本里的赋值
  #   按台的三个键两台各写不同的值：取错台的一眼看得出（文件头「按台的三个键」）
  printf '%s\n' 'HOSTS=127.0.0.1 192.0.2.1' 'CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.1=/selftest/peer-cargo' \
    'BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.1=/selftest/peer-builds' 'SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.1=1G' \
    'QUIESCE_STOP_COMMAND=' 'QUIESCE_STOPPED_CHECK_COMMAND=' 'QUIESCE_START_COMMAND=' 'QUIESCE_STARTED_CHECK_COMMAND=' \
    'ENABLE_ACROSS_MACHINES=0' 'ENABLE_GPU=0' "NFS_SHARED_ROOTS=$work/tree:$work/bin" > "$work/complete.env"
  # with_key <新文件> <键> <值或空> [<底稿>]：底稿（缺省 complete.env）去掉这个键那一行；值不空就换成这个值，空表示整行不写
  with_key() {
    grep -v "^$2=" "${4:-$work/complete.env}" > "$1"
    [[ -z "${3:-}" ]] || echo "$2=$3" >> "$1"
  }
  # with_nfs_roots <新文件> <值或空>：complete.env 去掉 NFS_SHARED_ROOTS 那一行；值不空就换成这个值，空串表示整行不写
  with_nfs_roots() {
    grep -v '^NFS_SHARED_ROOTS=' "$work/complete.env" > "$1"
    [[ -z "${2:-}" ]] || echo "NFS_SHARED_ROOTS=$2" >> "$1"
  }
  # judge_output <配置文件> [这一份的参数…]：照本机的条件起这一份（配置经 SINGLEFS_MULTI_HOST_CONFIG 给），打它的输出、退它的码
  judge_output() {
    local configuration="$1"
    shift
    env PATH="$work/bin:$PATH" CONFIGURATION_SELFTEST_SSH_LOG="$work/ssh.log" SINGLEFS_MULTI_HOST_CONFIG="$configuration" \
      SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 PEER_HOST_SELFTEST_MARK="$work/peer-host-selftest-mark" ${configuration_selftest_role[@]+"${configuration_selftest_role[@]}"} \
      bash "$self" "$@" "$work/tree" 2>&1
  }
  # judge <格名> <配置文件> <期望退出码> <输出里要有的一段或空> [这一份的参数…]：照期望核；判错的格名记进 failed、打一行 ✗
  judge() {
    local cell="$1" configuration="$2" wanted_exit="$3" wanted_text="$4" exit_code=0 output
    shift 4
    cells=$((cells + 1))
    : > "$work/ssh.log"
    output="$(judge_output "$configuration" "$@")" || exit_code=$?
    if [[ "$exit_code" != "$wanted_exit" ]] || [[ -n "$wanted_text" && "$output" != *"$wanted_text"* ]] || [[ -s "$work/ssh.log" ]]; then
      [[ " ${failed[*]-} " == *" $cell "* ]] || failed+=("$cell")   # 一个格名下可以有几条样本，判错的格名只记一次
      echo "  ✗ 自证 $cell：退 $exit_code（应当 $wanted_exit），输出 $(head -c 300 <<< "$output" | tr '\n' ' ')$( [[ -n "$wanted_text" ]] && echo "（要有「$wanted_text」）")；连了 ssh $(grep -c . "$work/ssh.log") 次"  # gate-lint:detail
    fi
  }
  local -a configuration_selftest_role=()
  judge complete-configuration-off "$work/complete.env" 3 "双机：关"
  # judge_both_roles <同 judge 的参数>：这一台的条件与第二台的条件（自证换掉网卡地址，192.0.2.1 是自己）各判一趟，两趟照同一个期望核（两台判同一份配置结局相同）
  judge_both_roles() {
    judge "$@"
    configuration_selftest_role=(CONFIGURATION_SELFTEST_LOCAL_ADDRESSES=192.0.2.1)
    judge "$@"
    configuration_selftest_role=()
  }
  # 按台的三个键（文件头「按台的三个键」）：每个键各一份缺键、多一项（同一地址两项；地址不在 HOSTS 里）、少一项、项的写法、值的写法
  local by_host_key by_host_sample
  for by_host_key in CARGO_BIN_DIRECTORY_BY_HOST BUILD_ROOT_BY_HOST SLICE_MEMORY_CAP_BY_HOST; do
    by_host_sample=/selftest/sample
    [[ "$by_host_key" == SLICE_MEMORY_CAP_BY_HOST ]] && by_host_sample=4G
    with_key "$work/$by_host_key-missing-key.env" "$by_host_key" ""
    judge_both_roles by-host-keys-required "$work/$by_host_key-missing-key.env" 1 "缺键 $by_host_key"
    with_key "$work/$by_host_key-duplicated.env" "$by_host_key" "127.0.0.1=$by_host_sample 192.0.2.1=$by_host_sample 192.0.2.1=$by_host_sample"
    judge_both_roles by-host-item-duplicated-refused "$work/$by_host_key-duplicated.env" 1 "$by_host_key 里地址「192.0.2.1」写了两项：多了一项"
    with_key "$work/$by_host_key-outside.env" "$by_host_key" "127.0.0.1=$by_host_sample 192.0.2.1=$by_host_sample 192.0.2.9=$by_host_sample"
    judge_both_roles by-host-address-outside-hosts-refused "$work/$by_host_key-outside.env" 1 "$by_host_key 里地址「192.0.2.9」不在 HOSTS 里"
    with_key "$work/$by_host_key-missing-item.env" "$by_host_key" "127.0.0.1=$by_host_sample"
    judge_both_roles by-host-item-missing-refused "$work/$by_host_key-missing-item.env" 1 "$by_host_key 里没有 HOSTS 的「192.0.2.1」那一项：少了一项"
    with_key "$work/$by_host_key-no-separator.env" "$by_host_key" "127.0.0.1 192.0.2.1=$by_host_sample"
    judge_both_roles by-host-item-form-refused "$work/$by_host_key-no-separator.env" 1 "$by_host_key 的一项「127.0.0.1」不是 <地址>=<值>"
    with_key "$work/$by_host_key-empty-value.env" "$by_host_key" "127.0.0.1=$by_host_sample 192.0.2.1="
    judge_both_roles by-host-item-form-refused "$work/$by_host_key-empty-value.env" 1 "$by_host_key 的一项「192.0.2.1=」不是 <地址>=<值>"
  done
  with_key "$work/cargo-relative.env" CARGO_BIN_DIRECTORY_BY_HOST "127.0.0.1=/selftest/this-cargo 192.0.2.1=relative/cargo"
  judge_both_roles by-host-value-form-refused "$work/cargo-relative.env" 1 "CARGO_BIN_DIRECTORY_BY_HOST 里 192.0.2.1 那一项「relative/cargo」不是绝对路径"
  with_key "$work/build-relative.env" BUILD_ROOT_BY_HOST "127.0.0.1=relative/builds 192.0.2.1=/selftest/peer-builds"
  judge_both_roles by-host-value-form-refused "$work/build-relative.env" 1 "BUILD_ROOT_BY_HOST 里 127.0.0.1 那一项「relative/builds」不是绝对路径"
  with_key "$work/slice-without-unit.env" SLICE_MEMORY_CAP_BY_HOST "127.0.0.1=2G 192.0.2.1=50"
  judge_both_roles by-host-value-form-refused "$work/slice-without-unit.env" 1 "SLICE_MEMORY_CAP_BY_HOST 里 192.0.2.1 那一项「50」：要写那一台 slice 的总上限"
  # GPU 内核一个一个编的台（文件头「GPU 内核一个一个编的台」）：写一台、两台、空串都判得过；地址不在 HOSTS 里、写两遍判不过；--emit-addresses 原样打出
  local serial_value
  for serial_value in "127.0.0.1" "192.0.2.1 127.0.0.1" '""'; do
    { cat "$work/complete.env"; echo "GPU_KERNEL_SERIAL_COMPILE_HOSTS=$serial_value"; } > "$work/serial-accepted.env"
    judge_both_roles serial-compile-hosts-accepted "$work/serial-accepted.env" 3 "双机：关"
  done
  { cat "$work/complete.env"; echo "GPU_KERNEL_SERIAL_COMPILE_HOSTS=192.0.2.1 192.0.2.9"; } > "$work/serial-outside.env"
  judge_both_roles serial-compile-hosts-outside-refused "$work/serial-outside.env" 1 "GPU_KERNEL_SERIAL_COMPILE_HOSTS 里「192.0.2.9」不在 HOSTS 里"
  { cat "$work/complete.env"; echo "GPU_KERNEL_SERIAL_COMPILE_HOSTS=192.0.2.1 192.0.2.1"; } > "$work/serial-duplicate.env"
  judge_both_roles serial-compile-hosts-duplicate-refused "$work/serial-duplicate.env" 1 "GPU_KERNEL_SERIAL_COMPILE_HOSTS 里「192.0.2.1」写了两遍"
  { cat "$work/complete.env"; echo "GPU_KERNEL_SERIAL_COMPILE_HOSTS=192.0.2.1"; } > "$work/serial-emitted.env"
  judge emit-addresses-serial-compile-hosts "$work/serial-emitted.env" 0 $'PEER_HOST_ADDRESS=192.0.2.1\nGPU_KERNEL_SERIAL_COMPILE_HOSTS=192.0.2.1' --emit-addresses
  with_nfs_roots "$work/no-nfs.env" ""
  judge nfs-roots-required "$work/no-nfs.env" 1 "缺键 NFS_SHARED_ROOTS"
  { cat "$work/no-nfs.env"; echo NFS_SHARED_ROOTS=; } > "$work/nfs-empty.env"
  judge nfs-roots-required "$work/nfs-empty.env" 1 "缺键 NFS_SHARED_ROOTS"
  with_nfs_roots "$work/nfs-trailing-slash.env" "$work/tree/"
  judge nfs-roots-form "$work/nfs-trailing-slash.env" 1 "NFS_SHARED_ROOTS 第 1 段「$work/tree/」以 / 结尾"
  with_nfs_roots "$work/nfs-relative.env" "relative/directory"
  judge nfs-roots-form "$work/nfs-relative.env" 1 "NFS_SHARED_ROOTS 第 1 段「relative/directory」不是绝对路径"
  with_nfs_roots "$work/nfs-empty-segment.env" "$work/tree::$work/bin"
  judge nfs-roots-form "$work/nfs-empty-segment.env" 1 "NFS_SHARED_ROOTS 第 2 段「」是空的"
  with_nfs_roots "$work/nfs-trailing-colon.env" "$work/tree:$work/bin:"
  judge nfs-roots-form "$work/nfs-trailing-colon.env" 1 "NFS_SHARED_ROOTS 第 3 段「」是空的"
  with_nfs_roots "$work/nfs-whitespace.env" "$work/tree:$work/a b"
  judge nfs-roots-form "$work/nfs-whitespace.env" 1 "NFS_SHARED_ROOTS 第 2 段「$work/a b」含空白"
  with_nfs_roots "$work/nfs-absent.env" "$work/tree:$work/absent-directory"
  judge nfs-roots-exist "$work/nfs-absent.env" 1 "NFS_SHARED_ROOTS 第 2 段「$work/absent-directory」不是本机已在的目录"
  with_nfs_roots "$work/nfs-file.env" "$work/complete.env"
  judge nfs-roots-exist "$work/nfs-file.env" 1 "NFS_SHARED_ROOTS 第 1 段「$work/complete.env」不是本机已在的目录"
  # 认得的键之外的任何键一律不认得：用一个从没用过的名字（不为哪个旧键单开一格）
  { cat "$work/complete.env"; echo 'SELFTEST_NEVER_USED_KEY=1'; } > "$work/unknown-key.env"
  judge_both_roles unknown-key-refused "$work/unknown-key.env" 1 "键 SELFTEST_NEVER_USED_KEY 不认得"
  { grep -v '^ENABLE_GPU=' "$work/complete.env"; echo ENABLE_GPU=2; } > "$work/switch-value.env"
  judge switch-value-refused "$work/switch-value.env" 1 "ENABLE_GPU 只许写 0 或 1"
  grep -v '^ENABLE_GPU=' "$work/complete.env" > "$work/switch-missing.env"
  judge switch-missing-refused "$work/switch-missing.env" 1 "ENABLE_GPU 没写：开关要明确写 0 或 1"
  { grep -v '^ENABLE_GPU=' "$work/complete.env"; echo ENABLE_GPU=1; } > "$work/gpu-no-cards.env"
  judge gpu-on-cards-required "$work/gpu-no-cards.env" 1 "ENABLE_GPU=1 而 GPU_CARDS_BY_PRIORITY 没写"
  { cat "$work/complete.env"; echo "SHARED_DATA_ROOT=$work/tree/data-root"; } > "$work/data-root.env"
  judge data-root-key-accepted "$work/data-root.env" 3 "双机：关"
  { cat "$work/complete.env"; printf '%s\n' "TASK_SERVER_STATE_DIRECTORY=$work/tree/service-state" "TASK_SERVER_RUNTIME_DIRECTORY=$work/tree/service-runtime" \
      "TASK_SERVER_SIGNING_KEY_FILE=$work/tree/service-key-sample" "TASK_SERVER_BUILD_DIRECTORY=$work/tree/service-builds"; } > "$work/service-keys.env"
  judge task-server-keys-accepted "$work/service-keys.env" 3 "双机：关"
  { grep -v '^ENABLE_GPU=' "$work/complete.env"; echo ENABLE_GPU=1; echo 'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:15G"'; echo 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"'; } > "$work/cards.env"
  judge gpu-card-keys-accepted "$work/cards.env" 3 "GPU：开"
  { grep -v '^ENABLE_ACROSS_MACHINES=' "$work/complete.env"; echo ENABLE_ACROSS_MACHINES=1; } > "$work/across.env"
  judge emit-hosts-from-by-host-items "$work/across.env" 0 $'MULTI_HOST_COUNT=1\nMULTI_HOST_2_SSH_HOST=192.0.2.1\nMULTI_HOST_2_REPOSITORY_DIRECTORY=/selftest/peer-builds\nMULTI_HOST_2_CARGO_BIN_DIRECTORY=/selftest/peer-cargo\nMULTI_HOST_2_MEMORY_CAP=1G\nMULTI_HOST_2_IS_PEER=1' --emit-hosts
  # 主机数组（文件头「主机数组」）：缺、只一项、重复、这一台不在里面、这一台占了两项、写了旧键各判不过；第二台是不是自己的那一项（写在头一项也认得出）
  with_hosts() { # <新文件> <HOSTS 的值或空>：complete.env 换掉 HOSTS 那一行；值空表示整行不写
    grep -v '^HOSTS=' "$work/complete.env" > "$1"
    [[ -z "${2:-}" ]] || echo "HOSTS=$2" >> "$1"
  }
  with_hosts "$work/hosts-missing.env" ""
  judge hosts-required "$work/hosts-missing.env" 1 "缺键 HOSTS"
  with_hosts "$work/hosts-single.env" "127.0.0.1"
  judge hosts-single-refused "$work/hosts-single.env" 1 "HOSTS 要写至少两项"
  with_hosts "$work/hosts-three.env" "127.0.0.1 192.0.2.1 192.0.2.3"
  judge hosts-three-refused "$work/hosts-three.env" 1 "HOSTS 里不是这一台的有 2 项"
  # --emit-addresses：双机关着也照打主机数组的三个赋值（第二台写在头一项也认得出）
  with_hosts "$work/hosts-other-first-off.env" "192.0.2.1 127.0.0.1"
  judge emit-addresses-when-off "$work/hosts-other-first-off.env" 0 $'THIS_HOST_ADDRESS=127.0.0.1\nPEER_HOST_ADDRESS=192.0.2.1' --emit-addresses
  with_hosts "$work/hosts-duplicate.env" "127.0.0.1 192.0.2.1 192.0.2.1"
  judge hosts-duplicate-refused "$work/hosts-duplicate.env" 1 "HOSTS 里「192.0.2.1」写了两遍"
  with_hosts "$work/hosts-form.env" "127.0.0.1 peer-machine-alias"
  judge hosts-form-refused "$work/hosts-form.env" 1 "HOSTS 的第 2 项「peer-machine-alias」不是 IP"
  with_hosts "$work/hosts-self-absent.env" "192.0.2.2 192.0.2.1"
  judge hosts-self-absent-refused "$work/hosts-self-absent.env" 1 "HOSTS 里没有一项是这一台网卡上的地址"
  # 这一台占两项：这一台网卡上的地址换成自证给的那一份（CONFIGURATION_SELFTEST_LOCAL_ADDRESSES，只在自证起的那几趟里认），两台都有的地址只有 127.0.0.1 一个
  with_hosts "$work/hosts-self-twice.env" "127.0.0.1 192.0.2.7 192.0.2.1"
  configuration_selftest_role=(CONFIGURATION_SELFTEST_LOCAL_ADDRESSES="127.0.0.1 192.0.2.7")
  judge hosts-self-twice-refused "$work/hosts-self-twice.env" 1 "HOSTS 里有 2 项都是这一台网卡上的地址"
  configuration_selftest_role=()
  { grep -v '^ENABLE_ACROSS_MACHINES=' "$work/hosts-self-absent.env" | grep -v '^HOSTS='; echo "HOSTS=192.0.2.1 127.0.0.1"; echo ENABLE_ACROSS_MACHINES=1; } > "$work/hosts-other-first.env"
  judge hosts-peer-is-the-other-item "$work/hosts-other-first.env" 0 $'THIS_HOST_ADDRESS=127.0.0.1\nPEER_HOST_ADDRESS=192.0.2.1' --emit-assignments
  { cat "$work/complete.env"; echo PRUNE_SERVER_TCP_ADDRESS=192.0.2.9:7311; } > "$work/prune-outside.env"
  judge prune-address-outside-hosts-refused "$work/prune-outside.env" 1 "PRUNE_SERVER_TCP_ADDRESS 的地址「192.0.2.9」不在 HOSTS 里"
  { cat "$work/complete.env"; echo PRUNE_SERVER_TCP_ADDRESS=192.0.2.1:7311; } > "$work/prune-inside.env"
  judge prune-address-in-hosts-accepted "$work/prune-inside.env" 3 "双机：关"
  # 两台对称（文件头「两台对称」）：同一份配置，本机的条件（环境给配置）与第二台的条件（家目录不在、环境指到一份不在的配置，
  #   参数给 --configuration）判得一样：退出码与输出逐字相同
  local same_local same_peer same_local_exit=0 same_peer_exit=0
  cells=$((cells + 1))
  : > "$work/ssh.log"
  same_local="$(judge_output "$work/across.env" --emit-hosts)" || same_local_exit=$?
  same_peer="$(configuration_selftest_role=(HOME=/nonexistent/x); judge_output "$work/absent-configuration.env" --configuration "$work/across.env" --emit-hosts)" || same_peer_exit=$?
  if [[ "$same_local_exit" != 0 || "$same_peer_exit" != "$same_local_exit" || "$same_peer" != "$same_local" || -s "$work/ssh.log" ]]; then
    failed+=(arguments-judge-the-same-on-both-hosts)
    echo "  ✗ 自证 arguments-judge-the-same-on-both-hosts：本机的条件退 $same_local_exit、第二台的条件（参数给配置、家目录不在）退 $same_peer_exit（都应当 0、输出逐字相同）；第二台那一趟的输出 $(head -c 300 <<< "$same_peer" | tr '\n' ' ')"  # gate-lint:detail
  fi
  # 查 cargo 那几格与下一格用的配置：两台那一项各指临时目录里的一份假 cargo，或第二台那一项指一个不在的目录
  with_key "$work/cargo-both.env" CARGO_BIN_DIRECTORY_BY_HOST "127.0.0.1=$work/cargo-this 192.0.2.1=$work/cargo-peer" "$work/across.env"
  with_key "$work/cargo-peer-absent.env" CARGO_BIN_DIRECTORY_BY_HOST "127.0.0.1=$work/cargo-this 192.0.2.1=$work/cargo-absent" "$work/across.env"
  # 只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 没有调用方自证的标记：不认，照连第二台
  #   （这一台那一项的 cargo 在；假 ssh 退 255 ⇒ 判不过、退 1，ssh 记了一次以上）
  local unmarked unmarked_exit=0
  cells=$((cells + 1))
  : > "$work/ssh.log"
  unmarked="$(env -u PEER_HOST_SELFTEST_MARK PATH="$work/bin:$PATH" CONFIGURATION_SELFTEST_SSH_LOG="$work/ssh.log" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/cargo-both.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 bash "$self" "$work/tree" 2>&1)" || unmarked_exit=$?
  if [[ "$unmarked_exit" != 1 || "$unmarked" != *"连不上"* || ! -s "$work/ssh.log" ]]; then
    failed+=(peer-is-this-machine-needs-the-mark)
    echo "  ✗ 自证 peer-is-this-machine-needs-the-mark：没有自证的标记、设了 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 退 $unmarked_exit（应当 1、照连第二台），连了 ssh $(grep -c . "$work/ssh.log") 次；输出 $(head -c 300 <<< "$unmarked" | tr '\n' ' ')"  # gate-lint:detail
  fi
  judge emit-hosts-nfs-roots "$work/across.env" 0 "NFS_SHARED_ROOTS=$work/tree:$work/bin" --emit-hosts
  # 两台都查 cargo（文件头「按台的三个键」）：双机开着、不带 --emit-hosts、不带只供测试的开关，假 ssh 把对面那条命令就地跑。
  #   这一台的条件（127.0.0.1 是自己）与第二台的条件（自证换掉网卡地址，192.0.2.1 是自己）判同一份配置：
  #   都退 0；ssh 去 192.0.2.1 问的是它那一项的目录，去 127.0.0.1 问的是 127.0.0.1 那一项的目录
  cargo_judge() { # <配置文件> <这一台网卡上的地址>：两台都查 cargo 的那一趟，打输出与 ssh 记下的调用，退它的码
    local cargo_exit=0
    : > "$work/ssh.log"
    env -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE PATH="$work/bin-running:$PATH" CONFIGURATION_SELFTEST_SSH_LOG="$work/ssh.log" \
      SINGLEFS_MULTI_HOST_CONFIG="$1" CONFIGURATION_SELFTEST_LOCAL_ADDRESSES="$2" bash "$self" "$work/tree" 2>&1 || cargo_exit=$?
    echo "ssh 调用：$(tr '\n' '|' < "$work/ssh.log")"
    return "$cargo_exit"
  }
  local cargo_this cargo_peer cargo_this_exit=0 cargo_peer_exit=0
  cells=$((cells + 1))
  cargo_this="$(cargo_judge "$work/cargo-both.env" 127.0.0.1)" || cargo_this_exit=$?
  cargo_peer="$(cargo_judge "$work/cargo-both.env" 192.0.2.1)" || cargo_peer_exit=$?
  if [[ "$cargo_this_exit" != 0 || "$cargo_peer_exit" != 0 || "$cargo_this" != *"192.0.2.1 test -x $work/cargo-peer/cargo"* \
        || "$cargo_peer" != *"127.0.0.1 test -x $work/cargo-this/cargo"* ]]; then
    failed+=(cargo-checked-with-each-host-item)
    echo "  ✗ 自证 cargo-checked-with-each-host-item：127.0.0.1 是自己时退 $cargo_this_exit、192.0.2.1 是自己时退 $cargo_peer_exit（都应当 0），ssh 去哪台要问那一台那一项的目录；输出 $(head -c 400 <<< "$cargo_this" | tr '\n' ' ') ‖ $(head -c 400 <<< "$cargo_peer" | tr '\n' ' ')"  # gate-lint:detail
  fi
  #   192.0.2.1 那一项的目录下没有 cargo：两种条件都退 1，都点名 192.0.2.1 那一项（第二台那一侧是本地查出的，这一侧是 ssh 查出的）
  local cargo_missing_text="CARGO_BIN_DIRECTORY_BY_HOST 里 192.0.2.1 那一项「$work/cargo-absent」下没有可执行的 cargo"
  cargo_this_exit=0 cargo_peer_exit=0
  cells=$((cells + 1))
  cargo_this="$(cargo_judge "$work/cargo-peer-absent.env" 127.0.0.1)" || cargo_this_exit=$?
  cargo_peer="$(cargo_judge "$work/cargo-peer-absent.env" 192.0.2.1)" || cargo_peer_exit=$?
  if [[ "$cargo_this_exit" != 1 || "$cargo_peer_exit" != 1 || "$cargo_this" != *"$cargo_missing_text"* || "$cargo_peer" != *"$cargo_missing_text"* ]]; then
    failed+=(cargo-missing-judged-the-same-on-both-hosts)
    echo "  ✗ 自证 cargo-missing-judged-the-same-on-both-hosts：127.0.0.1 是自己时退 $cargo_this_exit、192.0.2.1 是自己时退 $cargo_peer_exit（都应当 1，都要有「$cargo_missing_text」）；输出 $(head -c 400 <<< "$cargo_this" | tr '\n' ' ') ‖ $(head -c 400 <<< "$cargo_peer" | tr '\n' ' ')"  # gate-lint:detail
  fi
  judge emit-assignments-nfs-roots "$work/across.env" 0 "NFS_SHARED_ROOTS=$work/tree:$work/bin" --emit-assignments
  # 子进程那一轮（开关轮起的）只打判错的格，不再起开关轮
  if [[ -n "${CONFIGURATION_SELFTEST_WHEEL_CHILD:-}" ]]; then
    printf 'CONFIGURATION_SELFTEST_FAILED %s\n' "${failed[*]-}"
    return 0
  fi
  if (( ${#failed[@]} > 0 )); then
    echo "  ✗ --selftest 没通过：${#failed[@]} 格判错（共 $cells 格）：${failed[*]}"  # gate-lint:summary
    echo "     → 怎么办：看上面每一格的 ✗ 那一行，改判法（这一份认键、判写法那一段）或格的期望，再跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --selftest"
    return 1
  fi
  # 开关轮：文件头里点名「本份 --selftest 的」每条开关逐个打开、整份自证再跑一趟，判错的格要与点名的格逐格相同
  local switch_value named_line named wheel_output wheel_failed wheel_problems=() wheel_count=0
  while IFS=$'\t' read -r switch_value named_line; do
    wheel_count=$((wheel_count + 1))
    named="$(tr '、' ' ' <<< "$named_line" | xargs -n1 | LC_ALL=C sort | xargs)"
    wheel_output="$(CONFIGURATION_SELFTEST_WHEEL_CHILD=1 MULTI_HOST_CONFIGURATION_BREAK="$switch_value" bash "$self" --selftest 2>&1)"
    wheel_failed="$(sed -n 's/^CONFIGURATION_SELFTEST_FAILED //p' <<< "$wheel_output" | xargs -n1 2>/dev/null | LC_ALL=C sort | xargs)"
    if [[ "$wheel_failed" != "$named" ]]; then
      wheel_problems+=("MULTI_HOST_CONFIGURATION_BREAK=$switch_value 红的格「$wheel_failed」，点名的是「$named」")
    fi
  done < <(awk '/^#   MULTI_HOST_CONFIGURATION_BREAK=[a-z]/ { split($2, part, "="); value = part[2]; next }
                value != "" && /^#       本份 --selftest 的 / { line = $0; sub(/^#       本份 --selftest 的 /, "", line); print value "\t" line; value = ""; next }
                /^#   MULTI_HOST_CONFIGURATION_BREAK=/ { value = "" }' "$self")
  if (( wheel_count == 0 || ${#wheel_problems[@]} > 0 )); then
    printf '  ✗ 开关轮：%s\n' "${wheel_problems[@]:-文件头里一条点名本份 --selftest 的开关都没读到}"
    echo "     → 怎么办：开关红的格与文件头点名的对不上，改点名那一行或开关所在那一处，再跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --selftest"
    return 1
  fi
  echo "  ✓ 多机配置判法 --selftest：$cells 格都对；开关轮 $wheel_count 条开关各红在点名的格"
  return 0
}
if [[ "${1:-}" == --selftest ]]; then
  configuration_selftest
  exit $?
fi

emit_assignments=0
emit_hosts=0
emit_addresses=0
configuration_argument=""
while [[ "${1:-}" == --* ]]; do
  case "$1" in
    --emit-assignments) emit_assignments=1; shift ;;
    --emit-hosts) emit_hosts=1; shift ;;
    --emit-addresses) emit_addresses=1; shift ;;
    --configuration)
      if (( $# < 2 )) || [[ -z "$2" ]]; then
        echo "  ✗ $1 后面要写值（--configuration <配置文件>）"
        echo "     → 怎么办：照文件头的用法改参数再跑"
        exit 2
      fi
      configuration_argument="$2"
      shift 2 ;;
    *)
      echo "  ✗ 认不出的参数 $1（认 --emit-assignments、--emit-hosts、--emit-addresses、--configuration、--selftest，或不带）"
      echo "     → 怎么办：照文件头的用法改参数再跑"
      exit 2 ;;
  esac
done
(( emit_assignments + emit_hosts + emit_addresses > 1 )) && { echo "  ✗ --emit-assignments、--emit-hosts、--emit-addresses 只给一个"; echo "     → 怎么办：照文件头的用法改参数再跑"; exit 2; }
[[ "$configuration_break" == configuration-argument-ignored ]] && configuration_argument=""
repository_root="${1:-$crash_point_replay_shard_check_script_directory/../..}"

refuse() { # refuse <原因>：打原因与出路，退 1
  echo "  ✗ 多机配置判不过：$1"
  echo "     → 怎么办：照仓根 multi-host.env.example 建本地配置 multi-host.env（git 忽略），或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；第二台连不上先修 ssh 免密登录"
  exit 1
}

if [[ -n "$configuration_argument" ]]; then
  configuration_file="$configuration_argument"
elif [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
  configuration_file="$SINGLEFS_MULTI_HOST_CONFIG"
else
  if ! common_directory="$(git -C "$repository_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$common_directory" ]]; then
    refuse "$repository_root 不是 git 工作树，找不到主工作树的根（配置默认在那里）"
  fi
  configuration_file="$(dirname "$common_directory")/multi-host.env"
fi
[[ -f "$configuration_file" ]] || refuse "没有配置文件 $configuration_file"

declare -A configuration=()
line_number=0
while IFS= read -r configuration_line || [[ -n "$configuration_line" ]]; do
  line_number=$((line_number + 1))
  [[ "$configuration_line" =~ ^[[:space:]]*(#.*)?$ ]] && continue
  [[ "$configuration_line" =~ ^([A-Z][A-Z0-9_]*)=(.*)$ ]] || refuse "$configuration_file 第 $line_number 行认不出（要是 KEY=值）"
  configuration_key="${BASH_REMATCH[1]}"
  configuration_value="${BASH_REMATCH[2]}"
  if [[ "$configuration_value" =~ ^\'(.*)\'$ || "$configuration_value" =~ ^\"(.*)\"$ ]]; then configuration_value="${BASH_REMATCH[1]}"; fi
  known_key=0
  for expected_key in "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[@]}" "${MULTI_HOST_SWITCH_KEYS[@]}" "${MULTI_HOST_BY_HOST_KEYS[@]}" "$MULTI_HOST_NFS_KEY" "$MULTI_HOST_HOSTS_KEY" "${MULTI_HOST_SERVICE_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  if [[ "$configuration_break" != serial-compile-hosts-unknown && "$configuration_key" == "$MULTI_HOST_SERIAL_COMPILE_KEY" ]]; then known_key=1; fi
  if [[ "$configuration_break" != gpu-card-keys-unknown ]]; then
    for expected_key in "${MULTI_HOST_GPU_CARD_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  fi
  if [[ "$configuration_break" != data-root-key-unknown && "$configuration_key" == "$MULTI_HOST_DATA_ROOT_KEY" ]]; then known_key=1; fi
  if [[ "$configuration_break" != task-server-keys-unknown ]]; then
    for expected_key in "${MULTI_HOST_TASK_SERVER_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  fi
  [[ "$configuration_break" == unknown-keys-accepted ]] && known_key=1
  (( known_key )) || refuse "$configuration_file 第 $line_number 行的键 $configuration_key 不认得（认的是 ${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[*]} $MULTI_HOST_HOSTS_KEY ${MULTI_HOST_BY_HOST_KEYS[*]} $MULTI_HOST_SERIAL_COMPILE_KEY $MULTI_HOST_NFS_KEY ${MULTI_HOST_SWITCH_KEYS[*]} ${MULTI_HOST_GPU_CARD_KEYS[*]} ${MULTI_HOST_SERVICE_KEYS[*]} $MULTI_HOST_DATA_ROOT_KEY ${MULTI_HOST_TASK_SERVER_KEYS[*]}；写法照仓根 multi-host.env.example，不认的键删掉或改成那里的写法）"
  [[ -z "${configuration[$configuration_key]+set}" ]] || refuse "$configuration_file 里 $configuration_key 写了两遍"
  configuration[$configuration_key]="$configuration_value"
done < "$configuration_file"
missing_keys=()
for expected_key in "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[@]}" "$MULTI_HOST_HOSTS_KEY" "${MULTI_HOST_BY_HOST_KEYS[@]}"; do
  [[ " ${MULTI_HOST_BY_HOST_KEYS[*]} " == *" $expected_key "* && "$configuration_break" == by-host-keys-optional ]] && continue
  [[ "$expected_key" == "$MULTI_HOST_HOSTS_KEY" && "$configuration_break" == hosts-optional ]] && continue
  [[ -n "${configuration[$expected_key]+set}" ]] || missing_keys+=("$expected_key")
done
(( ${#missing_keys[@]} == 0 )) || refuse "$configuration_file 缺键 ${missing_keys[*]}（不清场、不手动复原的那一对写成空串，键照样要写）"
# 主机数组（文件头「主机数组」）：至少两项、不重复、每项是 IP；这一台网卡上的地址恰好命中一项（peer-host-lib.sh 的 peer_host_resolve_addresses）
hosts_text="${configuration[$MULTI_HOST_HOSTS_KEY]-}"
hosts_how_to_write="写法 $MULTI_HOST_HOSTS_KEY=<各台有线直连段的 IP，空格分隔，至少两项>（模板 multi-host.env.example；哪一台是自己照这一台网卡上的地址认，不写主机名）"
THIS_HOST_ADDRESS="" PEER_HOST_ADDRESS="" PEER_HOST_OTHER_ADDRESSES=()
if [[ -n "${configuration[$MULTI_HOST_HOSTS_KEY]+set}" ]]; then
  read -r -a hosts_items <<< "$hosts_text"
  if [[ "$configuration_break" != hosts-count-unchecked ]]; then
    (( ${#hosts_items[@]} >= 2 )) || refuse "$MULTI_HOST_HOSTS_KEY 要写至少两项（读到 ${#hosts_items[@]} 项「$hosts_text」）；$hosts_how_to_write"
    declare -A hosts_seen=()
    for hosts_item in "${hosts_items[@]}"; do
      [[ -z "${hosts_seen[$hosts_item]+set}" ]] || refuse "$MULTI_HOST_HOSTS_KEY 里「$hosts_item」写了两遍；$hosts_how_to_write"
      hosts_seen[$hosts_item]=1
    done
  fi
  for hosts_index in "${!hosts_items[@]}"; do
    [[ "${hosts_items[$hosts_index]}" =~ $MULTI_HOST_ADDRESS_PATTERN ]] \
      || refuse "$MULTI_HOST_HOSTS_KEY 的第 $((hosts_index + 1)) 项「${hosts_items[$hosts_index]}」不是 IP（IPv4 的点分写法或 IPv6）；$hosts_how_to_write"
  done
  if [[ "$configuration_break" == hosts-self-unchecked ]]; then
    THIS_HOST_ADDRESS="${hosts_items[0]}" PEER_HOST_ADDRESS="${hosts_items[1]:-}" PEER_HOST_OTHER_ADDRESSES=("${hosts_items[@]:1}")
  else
    peer_host_resolve_addresses "$hosts_text" "$configuration_local_addresses"
    case "$?" in
      0) ;;
      2) refuse "读不出这一台网卡上的地址（ip -o addr show 跑不起来），认不出 $MULTI_HOST_HOSTS_KEY 里哪一项是自己" ;;
      *) if (( peer_host_self_matches == 0 )); then
           refuse "$MULTI_HOST_HOSTS_KEY 里没有一项是这一台网卡上的地址（读到「$hosts_text」）：这一台不在多机配置里；$hosts_how_to_write"
         fi
         refuse "$MULTI_HOST_HOSTS_KEY 里有 $peer_host_self_matches 项都是这一台网卡上的地址（读到「$hosts_text」）：认不出哪一项是自己；一台只写一个地址" ;;
    esac
    if (( ${#PEER_HOST_OTHER_ADDRESSES[@]} != 1 )) && [[ "$configuration_break" != hosts-count-unchecked ]]; then
      refuse "$MULTI_HOST_HOSTS_KEY 里不是这一台的有 ${#PEER_HOST_OTHER_ADDRESSES[@]} 项（读到「$hosts_text」）：此刻只认两台，写自己与第二台各一项"
    fi
  fi
fi
# 判定库服务的地址要在主机数组里（文件头「主机数组」）；没写不判
prune_address_text="${configuration[PRUNE_SERVER_TCP_ADDRESS]-}"
if [[ -n "$prune_address_text" && "$configuration_break" != prune-address-unchecked ]]; then
  if [[ "$prune_address_text" =~ ^\[([^]]*)\]:[0-9]+$ || "$prune_address_text" =~ ^([^:]*):[0-9]+$ ]]; then
    prune_address="${BASH_REMATCH[1]}"
  else
    refuse "PRUNE_SERVER_TCP_ADDRESS 要写 <地址>:<端口>（IPv6 写 [<地址>]:<端口>），读到「$prune_address_text」"
  fi
  [[ " $hosts_text " == *" $prune_address "* ]] || refuse "PRUNE_SERVER_TCP_ADDRESS 的地址「$prune_address」不在 HOSTS 里（读到「$hosts_text」）：判定库服务要开在两台之一的有线直连段地址上"
fi
address_in_hosts() { # <地址>：是不是主机数组里的一项（逐项比字面）
  local hosts_item
  for hosts_item in ${hosts_items[@]+"${hosts_items[@]}"}; do [[ "$hosts_item" == "$1" ]] && return 0; done
  return 1
}
# 按台的三个键（文件头「按台的三个键」）：每项 <地址>=<值>，从第一个 = 切；地址在 HOSTS 里、不重复，HOSTS 里每一台都有一项。读进 by_host_item["<键> <地址>"]
declare -A by_host_item=()
by_host_value() { # <键> <地址>：那一台在这个键里的那一项的值（没有打空串）
  printf '%s' "${by_host_item[$1 $2]-}"
}
if [[ -n "${configuration[$MULTI_HOST_HOSTS_KEY]+set}" ]]; then
  for by_host_key in "${MULTI_HOST_BY_HOST_KEYS[@]}"; do
    [[ -n "${configuration[$by_host_key]+set}" ]] || continue
    by_host_how_to_write="写法 $by_host_key=<地址>=<值>[ <地址>=<值>…]，HOSTS 里每一台恰好一项（文件头「按台的三个键」，模板 multi-host.env.example）"
    read -r -a by_host_entries <<< "${configuration[$by_host_key]}"
    for by_host_entry in ${by_host_entries[@]+"${by_host_entries[@]}"}; do
      by_host_address="${by_host_entry%%=*}" by_host_entry_value="${by_host_entry#*=}"
      if [[ "$configuration_break" != by-host-value-form-unchecked ]]; then
        [[ "$by_host_entry" == *=* && -n "$by_host_address" && -n "$by_host_entry_value" ]] \
          || refuse "$by_host_key 的一项「$by_host_entry」不是 <地址>=<值>（没有 =，或 = 前后有一边是空的）；$by_host_how_to_write"
        if [[ "$by_host_key" == "$MULTI_HOST_BY_HOST_CAP_KEY" ]]; then
          [[ "$by_host_entry_value" =~ $MEMORY_CAP_PATTERN ]] \
            || refuse "$by_host_key 里 $by_host_address 那一项「$by_host_entry_value」：要写那一台 slice 的总上限（正整数加 K / M / G / T，例 50G，单位必写）；$by_host_how_to_write"
        elif [[ "$by_host_entry_value" != /* ]]; then
          refuse "$by_host_key 里 $by_host_address 那一项「$by_host_entry_value」不是绝对路径（要写那一台上的绝对路径）；$by_host_how_to_write"
        fi
      fi
      if ! address_in_hosts "$by_host_address" && [[ "$configuration_break" != by-host-outside-hosts-accepted ]]; then
        refuse "$by_host_key 里地址「$by_host_address」不在 HOSTS 里（读到 HOSTS「$hosts_text」）：多了一项；$by_host_how_to_write"
      fi
      if [[ -n "${by_host_item[$by_host_key $by_host_address]+set}" && "$configuration_break" != by-host-duplicate-accepted ]]; then
        refuse "$by_host_key 里地址「$by_host_address」写了两项：多了一项；$by_host_how_to_write"
      fi
      by_host_item[$by_host_key $by_host_address]="$by_host_entry_value"
    done
    for hosts_item in ${hosts_items[@]+"${hosts_items[@]}"}; do
      if [[ -z "${by_host_item[$by_host_key $hosts_item]+set}" && "$configuration_break" != by-host-missing-accepted ]]; then
        refuse "$by_host_key 里没有 HOSTS 的「$hosts_item」那一项：少了一项；$by_host_how_to_write"
      fi
    done
  done
  # GPU 内核一个一个编的台（文件头「GPU 内核一个一个编的台」）：可以不写；写了的每项在 HOSTS 里、不重复
  if [[ -n "${configuration[$MULTI_HOST_SERIAL_COMPILE_KEY]:-}" && "$configuration_break" != serial-compile-hosts-unchecked ]]; then
    serial_how_to_write="写法 $MULTI_HOST_SERIAL_COMPILE_KEY=<HOSTS 里的地址>[ <地址>…]，不写或写空串就是两台都一起编（文件头「GPU 内核一个一个编的台」）"
    declare -A serial_seen=()
    read -r -a serial_items <<< "${configuration[$MULTI_HOST_SERIAL_COMPILE_KEY]}"
    for serial_item in "${serial_items[@]}"; do
      address_in_hosts "$serial_item" || refuse "$MULTI_HOST_SERIAL_COMPILE_KEY 里「$serial_item」不在 HOSTS 里（读到 HOSTS「$hosts_text」）；$serial_how_to_write"
      [[ -z "${serial_seen[$serial_item]+set}" ]] || refuse "$MULTI_HOST_SERIAL_COMPILE_KEY 里「$serial_item」写了两遍；$serial_how_to_write"
      serial_seen[$serial_item]=1
    done
  fi
fi
serial_compile_hosts_text="${configuration[$MULTI_HOST_SERIAL_COMPILE_KEY]-}"
# NFS 共享的根（文件头「NFS 共享的根」）：必写；写法按段判，另判每段是这一台已在的目录
nfs_roots_text="${configuration[$MULTI_HOST_NFS_KEY]-}"
nfs_how_to_write="写法 $MULTI_HOST_NFS_KEY=<绝对路径>[:<绝对路径>…]（两台在同一绝对路径上共享的目录，模板见 multi-host.env.example），怎么导出、怎么挂载、怎么核见 $NFS_README_POINTER"
if [[ -z "$nfs_roots_text" ]]; then
  [[ "$configuration_break" == nfs-roots-optional ]] \
    || refuse "$configuration_file 缺键 $MULTI_HOST_NFS_KEY（没写，或写成了空串）：没有共享的目录，task-server 不启动，两台跑出来的结果也互相读不到；$nfs_how_to_write"
else
  nfs_rest="$nfs_roots_text:"
  nfs_index=0
  while [[ -n "$nfs_rest" ]]; do
    nfs_segment="${nfs_rest%%:*}"
    nfs_rest="${nfs_rest#*:}"
    nfs_index=$((nfs_index + 1))
    nfs_problem=""
    if [[ "$configuration_break" != nfs-roots-form-unchecked ]]; then
      if [[ -z "$nfs_segment" ]]; then nfs_problem="是空的（冒号连着写，或开头、结尾多了冒号）"
      elif [[ "$nfs_segment" != /* ]]; then nfs_problem="不是绝对路径（要以 / 起头）"
      elif [[ "$nfs_segment" =~ [[:space:]] ]]; then nfs_problem="含空白（路径里不许有空白）"
      elif [[ "$nfs_segment" == */ ]]; then nfs_problem="以 / 结尾（去掉结尾的 /）"
      fi
    fi
    if [[ -z "$nfs_problem" && "$configuration_break" != nfs-roots-exist-unchecked && ! -d "$nfs_segment" ]]; then
      nfs_problem="不是本机已在的目录（先按 $NFS_README_POINTER 建目录、导出与挂载）"
    fi
    [[ -z "$nfs_problem" ]] || refuse "$MULTI_HOST_NFS_KEY 第 $nfs_index 段「$nfs_segment」$nfs_problem；读到的整值「$nfs_roots_text」；$nfs_how_to_write"
  done
fi
for command_pair in "QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND" "QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND"; do
  read -r action_key check_key <<< "$command_pair"
  if [[ -n "${configuration[$action_key]}" && -z "${configuration[$check_key]}" ]] || [[ -z "${configuration[$action_key]}" && -n "${configuration[$check_key]}" ]]; then
    refuse "$action_key 与 $check_key 要么都写、要么都空：改了状态的命令跑完要回读一次"
  fi
done

# 两个开关：没写算 0；只许 0 或 1
for switch_key in "${MULTI_HOST_SWITCH_KEYS[@]}"; do
  if [[ -z "${configuration[$switch_key]+set}" ]]; then
    # 没写不算 0：开关一定要明确写出（用户：「配置配错 没有配置 就报错 不允许自动补齐」）；弄坏开关 switch-missing-defaults-zero 下照改前当 0
    [[ "$configuration_break" == switch-missing-defaults-zero ]] || refuse "$configuration_file 里 $switch_key 没写：开关要明确写 0 或 1，没写不算 0（要开写一行 $switch_key=1、要关写 $switch_key=0）"
    configuration[$switch_key]=0
  fi
  if [[ "${configuration[$switch_key]}" != 0 && "${configuration[$switch_key]}" != 1 && "$configuration_break" != switch-value-unchecked ]]; then
    refuse "$configuration_file 里 $switch_key 只许写 0 或 1，读到「${configuration[$switch_key]}」"
  fi
done
if [[ "${configuration[ENABLE_GPU]}" == 1 && "$configuration_break" != gpu-cards-optional ]]; then
  for card_key in "${MULTI_HOST_GPU_CARD_KEYS[@]}"; do
    [[ -n "${configuration[$card_key]+set}" ]] || refuse "$configuration_file 里 ENABLE_GPU=1 而 $card_key 没写：开了 GPU 两份显卡先后都要明确写出，不猜用哪几张卡"
  done
fi
# --emit-addresses（文件头用法）：整份判过之后只打主机数组的三个赋值，不看双机开关
if (( emit_addresses )); then
  printf '%s=%q\n' "$MULTI_HOST_HOSTS_KEY" "$hosts_text" THIS_HOST_ADDRESS "$THIS_HOST_ADDRESS" PEER_HOST_ADDRESS "$PEER_HOST_ADDRESS"
  [[ "$configuration_break" == serial-compile-hosts-not-emitted ]] || printf '%s=%q\n' "$MULTI_HOST_SERIAL_COMPILE_KEY" "$serial_compile_hosts_text"
  exit 0
fi
gpu_text="GPU：关"
[[ "${configuration[ENABLE_GPU]}" == 1 ]] && gpu_text="GPU：开"
if [[ "${configuration[ENABLE_ACROSS_MACHINES]}" != 1 && "$configuration_break" != switch-off-is-on ]]; then
  # 双机关着：配置判得过，不连第二台；调用方照单机跑
  echo "双机：关（配置 $configuration_file 里 ENABLE_ACROSS_MACHINES 不是 1；要开就把那一行改成 ENABLE_ACROSS_MACHINES=1），${gpu_text}"
  exit 3
fi

peer_description="第二台 $PEER_HOST_ADDRESS"
if (( emit_hosts )); then
  :   # 不连任何一台：连不连得上由调用方报（文件头 --emit-hosts）
elif [[ "${SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE:-0}" == 1 ]] && { peer_host_inside_selftest || [[ "$configuration_break" == peer-is-this-machine-unmarked ]]; }; then
  # 只供测试的开关只在调用方的自证里认（文件头「只供测试的开关」）；弄坏开关 peer-is-this-machine-unmarked 下没有标记也认
  peer_description="第二台是本机（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）"
else
  # 两台都查 cargo（文件头「按台的三个键」）：这一台取自己那一项本地查，第二台取它那一项经 ssh 查；两台判同一份配置查的是同一对目录
  this_cargo_directory="$(by_host_value CARGO_BIN_DIRECTORY_BY_HOST "$THIS_HOST_ADDRESS")"
  peer_cargo_directory="$(by_host_value CARGO_BIN_DIRECTORY_BY_HOST "$PEER_HOST_ADDRESS")"
  [[ "$configuration_break" == cargo-item-of-the-wrong-host ]] && peer_cargo_directory="$this_cargo_directory"
  cargo_missing() { # <地址> <目录>：那一台的 cargo 不在时的那一句（两台判出的同一处缺口说的是同一句）
    echo "CARGO_BIN_DIRECTORY_BY_HOST 里 $1 那一项「$2」下没有可执行的 cargo（那一项要指到那一台的 ~/.cargo/bin）"
  }
  if [[ "$configuration_break" != this-host-cargo-unchecked && ! -x "$this_cargo_directory/cargo" ]]; then
    refuse "这一台 $THIS_HOST_ADDRESS 上：$(cargo_missing "$THIS_HOST_ADDRESS" "$this_cargo_directory")"
  fi
  ssh -o BatchMode=yes -o ConnectTimeout=10 "$PEER_HOST_ADDRESS" true </dev/null >/dev/null 2>&1 \
    || refuse "ssh -o BatchMode=yes $PEER_HOST_ADDRESS true 连不上（用户名照这一台 ~/.ssh/config 里 Host $PEER_HOST_ADDRESS 那一条）"
  ssh -o BatchMode=yes -o ConnectTimeout=10 "$PEER_HOST_ADDRESS" "test -x $(printf '%q' "$peer_cargo_directory")/cargo" </dev/null >/dev/null 2>&1 \
    || refuse "第二台 $PEER_HOST_ADDRESS 上：$(cargo_missing "$PEER_HOST_ADDRESS" "$peer_cargo_directory")"
fi

if (( emit_assignments || emit_hosts )); then
  for expected_key in "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[@]}" "${MULTI_HOST_SWITCH_KEYS[@]}" "${MULTI_HOST_BY_HOST_KEYS[@]}"; do
    printf '%s=%q\n' "$expected_key" "${configuration[$expected_key]}"
  done
  [[ "$configuration_break" == serial-compile-hosts-not-emitted ]] || printf '%s=%q\n' "$MULTI_HOST_SERIAL_COMPILE_KEY" "$serial_compile_hosts_text"
  [[ "$configuration_break" == nfs-roots-not-emitted ]] || printf '%s=%q\n' "$MULTI_HOST_NFS_KEY" "${configuration[$MULTI_HOST_NFS_KEY]-}"
  printf 'CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE=%q\n' "$configuration_file"
  printf '%s=%q\n' "$MULTI_HOST_HOSTS_KEY" "$hosts_text" THIS_HOST_ADDRESS "$THIS_HOST_ADDRESS" PEER_HOST_ADDRESS "$PEER_HOST_ADDRESS"
  if (( emit_hosts )); then
    printf 'MULTI_HOST_COUNT=%s\n' "${#PEER_HOST_OTHER_ADDRESSES[@]}"
    for other_index in "${!PEER_HOST_OTHER_ADDRESSES[@]}"; do
      machine_number=$((other_index + 2))
      printf 'MULTI_HOST_%s_SSH_HOST=%q\n' "$machine_number" "${PEER_HOST_OTHER_ADDRESSES[$other_index]}"
      # 那一台的三项按它的地址取（文件头「按台的三个键」）；弄坏开关 emit-hosts-this-host-items 下取这一台自己那一项
      machine_address="${PEER_HOST_OTHER_ADDRESSES[$other_index]}"
      [[ "$configuration_break" == emit-hosts-this-host-items ]] && machine_address="$THIS_HOST_ADDRESS"
      printf 'MULTI_HOST_%s_REPOSITORY_DIRECTORY=%q\n' "$machine_number" "$(by_host_value BUILD_ROOT_BY_HOST "$machine_address")"
      printf 'MULTI_HOST_%s_CARGO_BIN_DIRECTORY=%q\n' "$machine_number" "$(by_host_value CARGO_BIN_DIRECTORY_BY_HOST "$machine_address")"
      printf 'MULTI_HOST_%s_MEMORY_CAP=%q\n' "$machine_number" "$(by_host_value SLICE_MEMORY_CAP_BY_HOST "$machine_address")"
      printf 'MULTI_HOST_%s_IS_PEER=%s\n' "$machine_number" "$(( other_index == 0 ? 1 : 0 ))"
    done
  fi
else
  echo "双机：开，${gpu_text}；配置 $configuration_file，$peer_description"
fi
