# 字段表字节与扇出（D8 已定项 11：含预留位的头 = 115 + 2 x key 宽；节点 16384）
NODE = 16384
def fanout(key, entry):
    header = 115 + 2 * key
    return header, (NODE - header) // entry
payload_compaction = 12 + 8 + 8 + 6      # R(设备4+起始槽6+槽数2) + c0 + segs0 + 续做点
payload_compaction_nocursor = 12 + 8 + 8
payload_destroy = 8 * 4                  # 头 lo hi P
for name, payload in (("整理(带续做点)", payload_compaction), ("整理(不带)", payload_compaction_nocursor), ("销毁快照", payload_destroy)):
    print("payload", name, payload)
width = max(payload_compaction, payload_destroy)
key = 2 + 8
entry = key + width
print("logged_ops key", key, "value", width, "entry", entry, "header/entries_per_node", fanout(key, entry))
width_nc = max(payload_compaction_nocursor, payload_destroy)
print("无续做点 value", width_nc, "entry", key + width_nc, fanout(key, key + width_nc))
print("分配记录树 key10 entry20", fanout(10, 20))
print("中央映射树 key27 叶条目55", fanout(27, 55), "内部条目113", fanout(27, 113))
print("记账树 key22 叶条目34", fanout(22, 34))
print("一个聚簇段字节数", 64 * 16384, "数据单元字节数", 2 * 16384)
