攻方腿推的结局（没跑：门禁格子 agent 不起，要主 agent 跑 run-gate-samples.sh）：
退 1。点名的只有两处对照：crates/demo/src/device_control.rs 第 2 行、scripts/experiments/own_control.py 第 3 行（「读自己的线程变量 …：capped.sh 不设它，文件里也没回退到 SINGLEFS_THREAD_CAP 或登记值」）。
不点名（漏判）的三份：
  scripts/hosts/env_prefix.sh            四行各自钉 8 个线程（命令前缀赋值、-j 8、--test-threads 8、:= 默认），THREAD_DEFAULT_FORMS 只认 ${X:-N}
  crates/demo/src/device_after_cfg_test.rs  第 1 行行首 #[cfg(test)] 只罩一条 use，判「读自己的线程变量」那一路在这里 break，后面的 SAMPLE_WORKER_THREADS 看不到
  scripts/experiments/mentions_capped.py    SAMPLE_OWN_JOBS 没设就 KeyError、不回退；文件里另有一处字符串 "scripts/hosts/capped.sh"（kept 留下），CAP_OR_REGISTRY 按整份文件认成「回退了」
