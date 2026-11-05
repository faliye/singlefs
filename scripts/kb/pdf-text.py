#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 它自己查要的抽取器，缺了逐份报出来，不交给调用方预判
"""把 PDF 的文字抽出来，供逐字复核外部文献引用。

    python3 scripts/kb/pdf-text.py <pdf> [> out.txt]

本机没有 poppler-utils，也没有 pip（`sudo -n` 要密码、`python3 -m pip` 不存在），
所以自带一个只依赖标准库 zlib 的抽取器。

⚠️ 射程：它只解 FlateDecode 的内容流，按 Tj/TJ/'/" 取字符串，并对
字体 /Differences 建单字节映射（不支持 /ToUnicode）。CID（两字节）字体不支持——
遇到那种 PDF 会抽出乱码，**乱码要当抽取失败处理，不许当成「原文没有这句」**。
判据在 --selftest：抽不出预期锚点句就判红。
"""
import re, sys, zlib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

def tokenize_objects(data):
    for m in re.finditer(rb'(\d+)\s+(\d+)\s+obj\b', data):
        start = m.end()
        end = data.find(b'endobj', start)
        if end < 0:
            continue
        yield int(m.group(1)), data[start:end]

def stream_bytes(body):
    m = re.search(rb'stream\r?\n', body)
    if not m:
        return None
    raw = body[m.end():]
    e = raw.rfind(b'endstream')
    if e >= 0:
        raw = raw[:e]
    if b'FlateDecode' in body[:m.start()]:
        try:
            return zlib.decompress(raw)
        except zlib.error as error:
            # 增量解压可能在输入截断时返回看似完整的文字；证据抽取不能接受半份流。
            raise ValueError(f'PDF FlateDecode 内容流损坏或不完整：{error}') from error
    return raw

STR_LIT = re.compile(rb'\((?:\\.|[^\\()])*\)', re.S)

def unescape(s):
    s = s[1:-1]
    out = bytearray(); i = 0
    while i < len(s):
        c = s[i]
        if c == 0x5c and i + 1 < len(s):
            n = s[i+1]
            mp = {0x6e: 10, 0x72: 13, 0x74: 9, 0x62: 8, 0x66: 12}
            if n in mp:
                out.append(mp[n]); i += 2
            elif n in (10, 13):
                i += 2
                if n == 13 and i < len(s) and s[i] == 10:
                    i += 1
            elif 0x30 <= n <= 0x37:
                j = i + 1; oct_ = b''
                while j < len(s) and len(oct_) < 3 and 0x30 <= s[j] <= 0x37:
                    oct_ += bytes([s[j]]); j += 1
                out.append(int(oct_, 8) & 0xFF); i = j
            else:
                out.append(n); i += 2
        else:
            out.append(c); i += 1
    return bytes(out)

def content_tokens(content):
    """内容流词法单元；字面字符串允许嵌套括号，注释里的操作符不参与抽取。"""
    i = 0
    while i < len(content):
        c = content[i:i + 1]
        if c.isspace() or c == b'\0':
            i += 1
            continue
        if c == b'%':
            while i < len(content) and content[i] not in (10, 13):
                i += 1
            continue
        if content[i:i + 2] in (b'<<', b'>>'):
            yield 'operator', content[i:i + 2]
            i += 2
            continue
        if c == b'/':
            i += 1
            while i < len(content) and content[i:i + 1] not in b' \t\r\n\x00()<>[]{}/%':
                i += 1
            continue
        if c == b'(':
            start, depth = i, 1
            i += 1
            while i < len(content) and depth:
                if content[i] == 92:
                    i += 2
                    continue
                if content[i] == 40:
                    depth += 1
                elif content[i] == 41:
                    depth -= 1
                i += 1
            if depth:
                raise ValueError('PDF 字面字符串没有闭合')
            yield 'string', unescape(content[start:i])
            continue
        if c == b'<' and content[i:i + 2] != b'<<':
            end = content.find(b'>', i + 1)
            if end < 0:
                raise ValueError('PDF 十六进制字符串没有闭合')
            value = re.sub(rb'\s', b'', content[i + 1:end])
            yield 'string', bytes.fromhex((value + (b'0' if len(value) % 2 else b'')).decode('ascii'))
            i = end + 1
            continue
        if c in (b'[', b']'):
            yield 'array', c
            i += 1
            continue
        match = re.match(rb"[^\s\x00()<>\[\]{}/%]+", content[i:])
        if match:
            token = match.group(0)
            yield ('number' if re.fullmatch(rb'[+-]?(?:\d+(?:\.\d*)?|\.\d+)', token) else 'operator'), token
            i += len(token)
        else:
            i += 1


def extract_text(content, cmap):
    out, operands, arrays = [], [], []
    in_text = False
    for kind, token in content_tokens(content):
        if kind == 'array':
            if token == b'[':
                arrays.append([])
            elif arrays:
                value = arrays.pop()
                (arrays[-1] if arrays else operands).append(value)
            continue
        if kind in ('string', 'number'):
            (arrays[-1] if arrays else operands).append(token if kind == 'string' else float(token))
            continue
        if token == b'BT':
            in_text = True
        elif token == b'ET':
            if in_text:
                out.append('\n')
            in_text = False
        elif in_text and token in (b'T*', b'Td', b'TD'):
            out.append('\n')
        elif in_text and token in (b'Tj', b'TJ', b"'", b'"') and operands:
            if token in (b"'", b'"'):
                out.append('\n')
            value = operands[-1]
            for piece in value if isinstance(value, list) else [value]:
                if isinstance(piece, float):
                    if piece < -100:
                        out.append(' ')
                elif isinstance(piece, bytes):
                    out.append(''.join(cmap.get(b, OT1.get(b, chr(b) if 32 <= b < 127 else ('�' if b else ''))) for b in piece))
        operands.clear()
    return ''.join(out)

# TeX 的 OT1 编码把连字放在 0x0B-0x0F，没有 /Differences 也要认——
# 不认的话 root-finding 会抽成 root-�nding，逐字复核当场失效。
OT1 = {0x0B: 'ff', 0x0C: 'fi', 0x0D: 'fl', 0x0E: 'ffi', 0x0F: 'ffl'}

GLYPH = {'ff':'ff','fi':'fi','fl':'fl','ffi':'ffi','ffl':'ffl','quotesingle':"'",
         'quoteright':'’','quoteleft':'‘','quotedblleft':'“','quotedblright':'”',
         'endash':'–','emdash':'—','hyphen':'-','bullet':'•','fraction':'/',
         'periodcentered':'·','minus':'−','circumflex':'^','tilde':'~'}

def build_cmap(data):
    """从 /Differences 建单字节码 → 字符的映射。多个字体的映射会合并——
    这在同一份 PDF 里可能撞车，所以只补 32..126 之外的码位，不覆盖 ASCII。"""
    cmap = {}
    for _, body in tokenize_objects(data):
        m = re.search(rb'/Differences\s*\[(.*?)\]', body, re.S)
        if not m:
            continue
        code = 0
        for tok in re.finditer(rb'(\d+)|/([A-Za-z0-9._]+)', m.group(1)):
            if tok.group(1):
                code = int(tok.group(1))
            else:
                name = tok.group(2).decode('latin-1')
                if code not in cmap and not (32 <= code < 127):
                    if name in GLYPH:
                        cmap[code] = GLYPH[name]
                    elif re.fullmatch(r'uni[0-9A-Fa-f]{4}', name):
                        cmap[code] = chr(int(name[3:], 16))
                code += 1
    return cmap

def pdf_text(path):
    with open(path, 'rb') as handle:
        data = handle.read()
    cmap = build_cmap(data)
    parts = []
    objects = dict(tokenize_objects(data))
    consumed = set()
    # 一页的 /Contents 数组是一段连续的内容程序，BT/ET 不能在每个流间重置。
    for body in objects.values():
        header = body.split(b'stream', 1)[0]
        if not re.search(rb'/Type\s*/Page\b', header):
            continue
        contents = re.search(rb'/Contents\s*(\[[^\]]*\]|\d+\s+\d+\s+R)', header, re.S)
        if not contents:
            continue
        references = contents.group(1)
        if not references.startswith(b'['):
            number = int(references.split()[0])
            indirect = objects.get(number, b'').strip()
            if indirect.startswith(b'['):
                references = indirect
        streams = []
        for number in re.findall(rb'(\d+)\s+\d+\s+R', references):
            number = int(number)
            stream = stream_bytes(objects.get(number, b''))
            if stream is None:
                raise ValueError(f'PDF 页内容流 {number} 缺失或无法解压')
            consumed.add(number)
            streams.append(stream)
        parts.append(extract_text(b'\n'.join(streams), cmap))
    for number, body in objects.items():
        if number in consumed:
            continue
        header = body.split(b'stream', 1)[0]
        if re.search(rb'/Subtype\s*/Image\b', header):
            continue
        s = stream_bytes(body)
        # PDF 操作符可直接邻接分隔符，例如 BT(hello)Tj；不要求两侧都是空白。
        if s and re.search(rb'(?:^|[\x00\t\n\f\r ()<>\[\]/%])BT(?=$|[\x00\t\n\f\r ()<>\[\]/%])', s):
            parts.append(extract_text(s, cmap))
    return '\n'.join(parts)

# 抽取器自己也要能失败：喂已知文献，抽不出锚点句就判红
SELFTEST = [
    ('ostep-45-file-integrity.pdf', 'the old block likely has a matching'),
    ('naclcrypto.pdf', 'polynomial root-finding'),
    ('nist-sp800-38d.pdf', 'Galois/Counter Mode'),
]

if __name__ == '__main__':
    preflight(__file__)
    if len(sys.argv) > 1 and sys.argv[1] == '--selftest':
        import os
        # 文献目录缺省取共享数据根下的 fs-refs/docs（.claude/hooks/lib_work_root.py 的 shared_data_path，两台同一个绝对路径；不经 ~ 取：第二台的家目录是另一个）
        if len(sys.argv) > 2:
            d = sys.argv[2]
        else:
            import importlib.util
            specification = importlib.util.spec_from_file_location(
                'lib_work_root_for_pdf_text', os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', '..', '.claude', 'hooks', 'lib_work_root.py'))
            work_roots = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(work_roots)
            try:
                d = os.path.join(work_roots.shared_data_path('fs-refs'), 'docs')
            except ValueError as error:
                print(f'  ✗ 取不到共享数据根下的 fs-refs：{error}')  # gate-lint:detail
                print(f'  → 怎么办：{getattr(work_roots, "DATA_ROOT_HOWTO", "配好数据根")}；或把文献目录写在 --selftest 后面')
                sys.exit(2)
        bad = 0
        for name, anchor in SELFTEST:
            p = os.path.join(d, name)
            if not os.path.exists(p):
                print(f'  ✗ {name} 不在本机'); bad += 1; continue  # gate-lint:detail
            try:
                t = ' '.join(pdf_text(p).split())
            except (OSError, ValueError) as error:
                print(f'  ✗ {name} 抽取失败：{error}')  # gate-lint:detail
                bad += 1
                continue
            hit = anchor in t
            print(f'  {"✓" if hit else "✗"} {name}  锚点句「{anchor}」{"命中" if hit else "抽不出 ⇒ 抽取器对这份 PDF 失效"}')  # gate-lint:detail
            bad += 0 if hit else 1
        if bad:
            print(f'  ✗ 自检未通过：{bad} 处')
            print('     → 怎么办：本机没有的文献先跑 scripts/kb/fetch-refs.sh 固定下来；'
                  '锚点抽不出的，去看这份 PDF 是不是用了双字节 CID 字体（抽取器只支持单字节），或者原文措辞变了要换一句还在的锚点。')
        sys.exit(1 if bad else 0)
    if len(sys.argv) != 2:
        print('用法：pdf-text.py <pdf> 或 --selftest [文献目录]', file=sys.stderr)
        sys.exit(2)
    try:
        sys.stdout.write(pdf_text(sys.argv[1]))
    except (OSError, ValueError) as error:
        print(f'抽取失败：{error}', file=sys.stderr)
        sys.exit(1)
