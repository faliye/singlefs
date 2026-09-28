"""E142 第十九次跑登记里的锚点，逐格抄自 research/prompts/e142-r19-prereg.md：
7.1 A1–A6（第 378–383 行）、7.2 B1–B9（第 391–399 行）、7.4 D1–D6（第 674–679 行）、C1–C13（第 685–697 行）。
不读装置、不读 crates/、不读产物。"""

A3_MKFS = '[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]'
A3_ACQ = '[system_configuration_slot×2]'
A3_WARM = '[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]'
A3_TXN = '[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]'
A3_POST = ('[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|'
           '[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]')
D3_WARM = '[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]'
D3_TXN = '[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]'
D3_POST = ('[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|'
           '[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]')

# 主几何（臂 N19C）：7.4 D1–D4、C1–C5。字段：operations segments closed_form kinds（D 表，Q142.48）；
# closed_form_three_state in_place_per_segment in_place_overwrites in_place_kinds（C 表 / D4，Q142.47 与 Q142.48）。
N19C = {
    'mkfs': dict(operations='23', segments='12+1+1+1+4', closed_form='4114', kinds=A3_MKFS,
                 closed_form_three_state='4114', in_place_per_segment='0+0+0+0+0', in_place_overwrites='0', in_place_kinds='none', source='D1 D2 D3 D4 C1'),
    'instance_acquisition': dict(operations='2', segments='2', closed_form='4', kinds=A3_ACQ,
                 closed_form_three_state='9', in_place_per_segment='2', in_place_overwrites='2', in_place_kinds='[system_configuration_slot×2]', source='D1 D2 D3 D4 C2'),
    'warm_up': dict(operations='22', segments='2+1+2+2+1+2', closed_form='15', kinds=D3_WARM,
                 closed_form_three_state='25', in_place_per_segment='0+0+2+0+0+2', in_place_overwrites='4', in_place_kinds='[system_configuration_slot×4]', source='D1 D2 D3 D4 C3'),
    'transaction': dict(operations='35', segments='24+2+1+2', closed_form='16777223', kinds=D3_TXN,
                 closed_form_three_state='16777228', in_place_per_segment='0+0+0+2', in_place_overwrites='2', in_place_kinds='[system_configuration_slot×2]', source='D1 D2 D3 D4 C4'),
    'post_mkfs_stream': dict(operations='59', segments='2+2+1+2+2+1+2+24+2+1+2', closed_form='16777240', kinds=D3_POST,
                 closed_form_three_state='16777260', in_place_per_segment='2+0+0+2+0+0+2+0+0+0+2', in_place_overwrites='8', in_place_kinds='[system_configuration_slot×8]', source='D1 D2 D3 D4 C5'),
}

# G11（⑧ 关，= 臂 N19，R44）：7.1 A1–A4、7.2 B1–B5；操作数取 7.1 A2（23、2、18、33、53）。
G11 = {
    'mkfs': dict(operations='23', segments='12+1+1+1+4', closed_form='4114', closed_form_three_state='4114', in_place_per_segment='0+0+0+0+0', kinds=A3_MKFS, source='A1 A2 A3 A4 B1'),
    'instance_acquisition': dict(operations='2', segments='2', closed_form='4', closed_form_three_state='9', in_place_per_segment='2', kinds=A3_ACQ, source='A1 A2 A3 A4 B2'),
    'warm_up': dict(operations='18', segments='2+1+2+2+1+2', closed_form='15', closed_form_three_state='25', in_place_per_segment='0+0+2+0+0+2', kinds=A3_WARM, source='A1 A2 A3 A4 B3'),
    'transaction': dict(operations='33', segments='24+2+1+2', closed_form='16777223', closed_form_three_state='16777228', in_place_per_segment='0+0+0+2', kinds=A3_TXN, source='A1 A2 A3 A4 B4'),
    'post_mkfs_stream': dict(operations='53', segments='2+2+1+2+2+1+26+2+1+2', closed_form='67108885', closed_form_three_state='150994980', in_place_per_segment='2+0+0+2+0+0+2+0+0+2', kinds=A3_POST, source='A1 A2 A3 A4 B5'),
}

# 吞一步屏障的四点：kinds 按每段多重集比（D6、A6）；其余字段逐字比（C6–C9、B6–B9）。多重集写成每段一个 dict。
def segs(*segments):
    return [dict(segment) for segment in segments]

D3_POST_FIRST_EIGHT = segs(
    {'system_configuration_slot': 2, 'barrier': 2}, {'journal_record': 2, 'barrier': 2}, {'root_record_fua': 1},
    {'system_configuration_slot': 2, 'barrier': 2}, {'journal_record': 2, 'barrier': 4}, {'root_record_fua': 1},
    {'system_configuration_slot': 2, 'barrier': 2}, {'unit_write': 24, 'barrier': 2})
A3_POST_FIRST_SEVEN = segs(
    {'system_configuration_slot': 2, 'barrier': 2}, {'journal_record': 2, 'barrier': 2}, {'root_record_fua': 1},
    {'system_configuration_slot': 2, 'barrier': 2}, {'journal_record': 2, 'barrier': 2}, {'root_record_fua': 1},
    {'unit_write': 24, 'system_configuration_slot': 2, 'barrier': 2})
G10A_TXN = segs({'unit_write': 24, 'barrier': 2}, {'journal_record': 2, 'barrier': 3, 'root_record_fua': 1, 'system_configuration_slot': 2})
G10B_TXN = segs({'unit_write': 24, 'barrier': 2}, {'journal_record': 2, 'barrier': 1, 'root_record_fua': 1}, {'system_configuration_slot': 2, 'barrier': 2})
G12A_TXN = segs({'unit_write': 24, 'barrier': 2}, {'journal_record': 2, 'barrier': 1, 'root_record_fua': 1, 'system_configuration_slot': 2})
G12B_TXN = segs({'unit_write': 24, 'barrier': 2}, {'journal_record': 2, 'barrier': 1, 'root_record_fua': 1}, {'system_configuration_slot': 2})

SWALLOWED = {
    ('G10a', 'transaction'): dict(operations='34', segments='24+5', closed_form='16777247', closed_form_three_state='16777287', in_place_per_segment='0+2', kinds=G10A_TXN, source='C6 D6'),
    ('G10a', 'post_mkfs_stream'): dict(operations='58', segments='2+2+1+2+2+1+2+24+5', closed_form='16777264', closed_form_three_state='16777319', in_place_per_segment='2+0+0+2+0+0+2+0+2', kinds=D3_POST_FIRST_EIGHT + G10A_TXN[1:], source='C7 D6'),
    ('G10b', 'transaction'): dict(operations='34', segments='24+3+2', closed_form='16777226', closed_form_three_state='16777231', in_place_per_segment='0+0+2', kinds=G10B_TXN, source='C8 D6'),
    ('G10b', 'post_mkfs_stream'): dict(operations='58', segments='2+2+1+2+2+1+2+24+3+2', closed_form='16777243', closed_form_three_state='16777263', in_place_per_segment='2+0+0+2+0+0+2+0+0+2', kinds=D3_POST_FIRST_EIGHT + G10B_TXN[1:], source='C9 D6'),
    ('G12a', 'transaction'): dict(operations='32', segments='24+5', closed_form='16777247', closed_form_three_state='16777287', in_place_per_segment='0+2', kinds=G12A_TXN, source='B6 A6'),
    ('G12a', 'post_mkfs_stream'): dict(operations='52', segments='2+2+1+2+2+1+26+5', closed_form='67108909', closed_form_three_state='150995039', in_place_per_segment='2+0+0+2+0+0+2+2', kinds=A3_POST_FIRST_SEVEN + G12A_TXN[1:], source='B7 A6'),
    ('G12b', 'transaction'): dict(operations='32', segments='24+3+2', closed_form='16777226', closed_form_three_state='16777231', in_place_per_segment='0+0+2', kinds=G12B_TXN, source='B8 A6'),
    ('G12b', 'post_mkfs_stream'): dict(operations='52', segments='2+2+1+2+2+1+26+3+2', closed_form='67108888', closed_form_three_state='150994983', in_place_per_segment='2+0+0+2+0+0+2+0+2', kinds=A3_POST_FIRST_SEVEN + G12B_TXN[1:], source='B9 A6'),
}

WRITE_LIST = dict(writes='29', barriers='6', fua='1', source='D5')
BEFORE_WINDOW = dict(operations='47', writes='31', source='C10')
WINDOW = dict(operations='35', writes='29', barriers='6', fua='1', source='C11')
LAYER0_CLOSED_FORM = '16777240'  # C12
