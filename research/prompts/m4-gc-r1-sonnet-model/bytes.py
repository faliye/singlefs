# 一批搬迁新写出的字节：2 盘各写一份；P = 这次发布新写的提交内生节点数（推的，取 6 / 8 / 12 三档）
NODE = 16384
DATA = 2 * NODE
def batch_bytes(k, unit, p, record_bytes=4096, named_per_record=64):
    units = 2 * (k * unit + p * NODE)
    records = -(-(k + p) // named_per_record)
    journal = 2 * records * record_bytes
    return units, journal, units + journal
for unit_name, unit in (("数据单元 32768", DATA), ("节点 16384", NODE)):
    print("==", unit_name)
    for k in (1, 4, 16, 64):
        row = []
        for p in (6, 8, 12):
            u, j, t = batch_bytes(k, unit, p)
            row.append(f"P={p}: 单元{u} 记录{j} 合计{t}")
        print(f"K={k}", " | ".join(row))
