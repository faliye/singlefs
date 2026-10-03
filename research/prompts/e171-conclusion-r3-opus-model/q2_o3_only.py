"""e171-conclusion-r3 攻方：拿比对器自己的 Q2 读法（bad_images_report.py 的读入与并表函数，原样调用），在一份 O3 坏镜像判定上数各族「只有 O3 红」的张数。
用法：python3 q2_o3_only.py <比对器目录> <known-bad 目录> <E170 bad-states 目录> <标签>=<o3_bad_image_judge 产物> …
"""
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, sys.argv[1])
import bad_images_report as report  # noqa: E402

known_bad_dir, expected_dir = sys.argv[2], sys.argv[3]
rows = report.read_known_bad_rows(known_bad_dir) + report.read_e170_rows(known_bad_dir, expected_dir)
for argument in sys.argv[4:]:
    label, path = argument.split("=", 1)
    attached = report.attach_o3(rows, report.read_image_judgements(path))
    parts = []
    for family in ("P", "B", "K", "N"):
        members = [row for row in attached if row.family == family]
        only = [row for row in members if row.o3_strict_is_red() and not row.o2_is_red()]
        parts.append(f"{family}:o3_red={sum(row.o3_strict_is_red() for row in members)}/o3_only={len(only)}")
    only_all = [f"{row.family}:{row.stream[:3]}:{row.name}" for row in attached if row.o3_strict_is_red() and not row.o2_is_red()]
    print(f"name=q2_o3_only variant={label} " + " ".join(parts) + f" o3_only_total={len(only_all)} o3_only_rows={','.join(only_all) or 'none'}")
