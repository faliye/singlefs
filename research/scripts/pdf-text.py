#!/usr/bin/env python3
"""把 PDF 的文字抽出来，供逐字复核外部文献引用。

    python3 research/scripts/pdf-text.py <pdf> [> out.txt]

本机没有 poppler-utils，也没有 pip（`sudo -n` 要密码、`python3 -m pip` 不存在），
所以自带一个只依赖标准库 zlib 的抽取器。

⚠️ 射程：它只解 FlateDecode 的内容流，按 Tj/TJ/'/" 取字符串，并对
字体 /Differences 与 /ToUnicode 建单字节映射。CID（两字节）字体不支持——
遇到那种 PDF 会抽出乱码，**乱码要当抽取失败处理，不许当成「原文没有这句」**。
判据在 --selftest：抽不出预期锚点句就判红。
"""
import re, sys, zlib

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
        except zlib.error:
            try:
                return zlib.decompressobj().decompress(raw)
            except zlib.error:
                return None
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

def extract_text(content, cmap):
    out = []
    # 按操作符切：字符串后面跟 Tj / TJ / ' / "，以及 Td/TD/T* 这些换行信号
    for m in re.finditer(rb'(\[(?:[^\[\]\\]|\\.)*\]|\((?:\\.|[^\\()])*\)|<[0-9A-Fa-f\s]*>)\s*(TJ|Tj|\'|\")|(T\*|Td|TD|ET)', content, re.S):
        if m.group(3):
            out.append('\n'); continue
        blob, op = m.group(1), m.group(2)
        pieces = []
        if blob.startswith(b'['):
            for sm in re.finditer(rb'\((?:\\.|[^\\()])*\)|<[0-9A-Fa-f\s]*>|-?\d+(?:\.\d+)?', blob, re.S):
                t = sm.group(0)
                if t.startswith(b'('):
                    pieces.append(unescape(t))
                elif t.startswith(b'<'):
                    h = re.sub(rb'\s', b'', t[1:-1])
                    pieces.append(bytes.fromhex(h.decode('ascii')) if len(h) % 2 == 0 else b'')
                else:
                    # 负的字距调整 ⇒ 词间空格。阈值 100 是经验值，宁可多空格不许粘词
                    if float(t) < -100:
                        pieces.append(b' ')
        elif blob.startswith(b'<'):
            h = re.sub(rb'\s', b'', blob[1:-1])
            pieces.append(bytes.fromhex(h.decode('ascii')) if len(h) % 2 == 0 else b'')
        else:
            pieces.append(unescape(blob))
        for p in pieces:
            out.append(''.join(cmap.get(b, OT1.get(b, chr(b) if 32 <= b < 127 else ('�' if b else ''))) for b in p))
        if op in (b"'", b'"'):
            out.append('\n')
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
    data = open(path, 'rb').read()
    cmap = build_cmap(data)
    parts = []
    for _, body in tokenize_objects(data):
        s = stream_bytes(body)
        if s and (b'Tj' in s or b'TJ' in s):
            parts.append(extract_text(s, cmap))
    return '\n'.join(parts)

# 抽取器自己也要能失败：喂已知文献，抽不出锚点句就判红
SELFTEST = [
    ('ostep-45-file-integrity.pdf', 'the old block likely has a matching'),
    ('naclcrypto.pdf', 'polynomial root-finding'),
    ('nist-sp800-38d.pdf', 'Galois/Counter Mode'),
]

if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == '--selftest':
        import os
        d = sys.argv[2] if len(sys.argv) > 2 else '/home/fy5090/code/fs-refs/docs'
        bad = 0
        for name, anchor in SELFTEST:
            p = os.path.join(d, name)
            if not os.path.exists(p):
                print(f'  ✗ {name} 不在本机'); bad += 1; continue
            t = ' '.join(pdf_text(p).split())
            hit = anchor in t
            print(f'  {"✓" if hit else "✗"} {name}  锚点句「{anchor}」{"命中" if hit else "抽不出 ⇒ 抽取器对这份 PDF 失效"}')
            bad += 0 if hit else 1
        sys.exit(1 if bad else 0)
    sys.stdout.write(pdf_text(sys.argv[1]))
