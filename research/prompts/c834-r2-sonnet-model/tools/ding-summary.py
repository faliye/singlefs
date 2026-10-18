"""汇总 probe-ding-<候选>.out：只列那一判走到了的状态（序号、盘序、判、M2 结局）。用法：python3 ding-summary.py <目录> <候选…>"""
import re, sys
directory = sys.argv[1]
for candidate in sys.argv[2:]:
    print("==", candidate)
    for line in open(f"{directory}/probe-ding-{candidate}.out"):
        if not line.startswith("name=c834_state"):
            continue
        judge = re.search(r"judge=(\S+)", line).group(1)
        if judge == "not_reached":
            continue
        ordinal = re.search(r"ordinal=(\d+)", line).group(1)
        order = re.search(r"order=(\w+)", line).group(1)
        verdict = re.search(r"verdict=(\w+)", judge).group(1)
        m2 = "refused" if "m2=refused" in line else "mounted(overwrote=%s)" % re.search(r"r_units_overwritten_before_row_root=(\d+)", line).group(1)
        print(ordinal, order, verdict, m2, "later_records=" + re.search(r"later_records=(\d)", line).group(1))
