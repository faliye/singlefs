"""e171-conclusion-r3 攻方：第二轮判决第三节 C407 改写那一形的判官，照字面写成代码（只在攻方模型上，被攻过零轮）。
字面：「判官读已入库的 E171 比对器产物（不重跑）：①『三者全部放行』的行集合等于登记集合 …；② 每一列至少有一样（坏实现行或坏镜像）只有它红，
O3 那一列用 Q2 的 O3 独有坏镜像」。字面没写的几处，这里取最宽的读法并在输出里点名：
- 读法取甲（O1 = 对拍或崩溃判官、O2、O3 严格），批 1；计入分母的行取 `name=row … category=counted`，补位行照 `name=substitution` 换。
- ② 的 O1、O2 两列在计入分母的行里找「只有它红」的行；O3 那一列取 `name=q2_family` 的 P、B 两族 `o3_only_strict` 之和 > 0。
- 不读：分母 D、rows_run、冻结核对、阳性 / 阴性对照、批 2、丙读法、跑的是哪棵树（产物里没有树的摘要）。
用法：python3 c407_judge.py <产物> <登记集合，逗号分隔>
"""
import re
import sys

product, registered = sys.argv[1], {int(x) for x in sys.argv[2].split(",") if x}
category, matrix, q2_o3_only = {}, {}, 0
for line in open(product, encoding="utf-8"):
    m = re.match(r"E7RESULT name=row row=(\d+) .*category=([a-z_]+)", line)
    if m:
        category[int(m.group(1))] = m.group(2)
    m = re.match(r"E7RESULT name=substitution row=(\d+) .*category=([a-z_]+)", line)
    if m:
        category[int(m.group(1))] = m.group(2)
    m = re.match(r"E7RESULT name=row_matrix row=(\d+) batch=1 O1=(\w+) \S+ \S+ O2=(\w+) O3甲=(\w+)", line)
    if m:
        matrix[int(m.group(1))] = {"O1": m.group(2) == "red", "O2": m.group(3) == "red", "O3": m.group(4) == "red"}
    m = re.match(r"E7RESULT name=q2_family family=([PB]) .* o3_only_strict=(\d+) ", line)
    if m:
        q2_o3_only += int(m.group(2))
counted = sorted(n for n, c in category.items() if c == "counted" and n in matrix)
quiet = {n for n in counted if not any(matrix[n].values())}
only = {o: [n for n in counted if matrix[n][o] and not any(v for k, v in matrix[n].items() if k != o)] for o in ("O1", "O2", "O3")}
first = quiet == registered
second = {"O1": bool(only["O1"]), "O2": bool(only["O2"]), "O3": q2_o3_only > 0}
verdict = "green" if first and all(second.values()) else "red"
print(f"name=c407_judge verdict={verdict} counted_rows={len(counted)} all_quiet={','.join(map(str, sorted(quiet))) or 'none'} "
      f"equals_registered={str(first).lower()} only_O1_rows={len(only['O1'])} only_O2_rows={len(only['O2'])} only_O3_rows={len(only['O3'])} "
      f"q2_o3_only_images={q2_o3_only} column_has_a_unique_red={','.join(k for k, v in second.items() if v) or 'none'}")
