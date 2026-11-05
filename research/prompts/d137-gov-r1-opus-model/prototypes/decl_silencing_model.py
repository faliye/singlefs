"""声明静音模型：看门狗（scripts/agents/agent-watch.py core_usage_report 第 3476 行）对任一条 declared_in_window>0 的低核读数判「只记不叫醒」。
造一段「单线程忙等 600 秒（只用 1 核，真卡住）」的样本，比三种声明下看门狗叫不叫醒。用法：python3 -BI decl_silencing_model.py <仓根>。只调 core_usage 的纯函数，不起 cargo、不碰盘。"""
import importlib.util, sys
repo = sys.argv[1]
spec = importlib.util.spec_from_file_location("core_usage", f"{repo}/scripts/hosts/core_usage.py")
cu = importlib.util.module_from_spec(spec); spec.loader.exec_module(cu)
registered = int(next(line.split()[0] for line in open(f"{repo}/.claude/heavy-thread-default") if line.strip() and not line.lstrip().startswith("#")))
window, threshold = cu.WINDOW_SECONDS, registered / cu.SHARE_DIVISOR
base = {"time": 0.0, "started": 0.0, "cpu": 0.0, "thread_cpu": {"1": 0.0}, "root": 999, "disk_waiting": 0, "computing": 1}
cur = {"time": 600.0, "started": 0.0, "cpu": 600.0, "thread_cpu": {"1": 600.0}, "root": 999, "disk_waiting": 0, "computing": 1}
samples = [base, cur]
def run(decl, tag):
    entries, problems = cu.parse_declarations(decl)
    v = cu.judge_series(samples, entries, problems, registered, window)
    silenced = (v["status"] == "low" and v["declared_in_window"] > 0)
    print(f"name=decl_silencing tag={tag} status={v['status']} cores={v['cores']:.1f} threshold={threshold:.1f} "
          f"declared_in_window={v['declared_in_window']:.0f} declared_fraction={v['declared_fraction']:.2f} watchdog_silenced={silenced}")
run("", "no-declaration-stuck")
run("0.0\t此刻串行：等待外部资源就绪中", "one-open-declaration-never-closed")
run("1.0\t此刻串行：短暂独占共享状态一下\n2.0\t此刻并行", "one-second-serial-then-closed")
