这一目录里的输出是 2026-10-19 在主工作区现跑的原样：时刻在另一位工具员换上 cargo run 那一批钩子之前（lib_heavy_tests.py 当时是 08efe8b7…，与开工快照相同），复跑 tools/rerun.sh 不再起它们（钩子自证不在副本里起）。
rerun-today*.log、rerun-preswap*.log 是第二轮攻方模型的四行比对：不带 -nolock 的那两份当时主工作区合入锁被别的会话占着；-nolock 的两份把锁指到不存在的文件。
