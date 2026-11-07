# scripts/prune-server

判定库服务 prune-server（服务本体在仓根 `prune-server/`）的 Python 一侧：被三家判定库 import 的客户端、照三家读写形态压服务的模拟负载。两台都要跑得到。连哪一处只由多机配置 `multi-host.env` 的 `PRUNE_SERVER_TCP_ADDRESS` 定（本机有 socket 走 socket），配置里没有就报错、不补缺省；改连接地址与时限的四个环境变量只在临时服务底下认，平常跑设了退 3（客户端文件头「连哪一处」）。

| 文件 | 做什么 |
|---|---|
| `prune_server_client.py` | 客户端（被 `scripts/crash/lib_verdict_library.py`、`scripts/gate/admission.py`、`scripts/mutation/lib_mutation_units.py` 与广谱变异的驱动 `scripts/mutation/broad-mutation.py`（族 broad-mutation）按路径载入）：三家读写只有经它这一条路、本地库目录不建不读不写；成批点读、成批写（带身份与请求号）、前缀扫、统计、报到；连不上退 75（调用方照 75 整道退出，不当现判）；键空间、容错、退出码在它文件头；`check` 连一次；`selftest_service_problem()` 给三家自证核「这一趟连的是临时服务」；大过一帧的值透明地分段存取（清单、带版本的段键、版本登记键的写法在它文件头「键空间」大值分段，写方死在写清单之前读回的照旧是写之前的值，拼不回退 3、不当「不在」）；一条请求大过帧长上限又分不了段的当场退 3，不重试、不退 75 |
| `prune_server_client_selftest.py` | 客户端的自证与弄坏开关（`--selftest`）；起一份临时服务再起一条命令（`--with-temporary-service [--server-binary <二进制>] -- <命令>`，三家与旁路的原有自证、经三家写服务的别的自证（门禁与入口那几份、e165、lkmm、合入后验证、变异回归单测）只在它底下跑，不写真服务）；起一份临时服务整份重跑这几份自证（`--selftest-families`，名单是 `family_selftests()`）。与客户端分成两份，客户端被 import 时不把起服务、起 cargo 的边带进各家判法的单元闭包 |
| `prune-server-load-simulation.py` | 照三家一趟的读写形态量每家每秒多少条、失败几条 |
