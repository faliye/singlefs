#!/usr/bin/env python3
"""撇号类角标字符：门禁 12 号（收尾时全仓扫）与 .claude/hooks/write-guard.sh（写入那一刻拒绝）共用这一份，不各抄一份。

两处各抄一份，改一处漏一处，就会有门禁判红而 hook 放行的字符，或者反过来。
这份源码里几个字符写成转义，不出现字面：门禁 12 号扫全仓，也扫这一份。
规则在 .claude/rules/path-moves.md「变体起新名字，不用角标」；ASCII 单引号不在其内（机器分不开它与引号）。
"""

# U+2032 PRIME、U+2033 DOUBLE PRIME、U+2034 TRIPLE PRIME、U+02B9 MODIFIER LETTER PRIME、U+02BA MODIFIER LETTER DOUBLE PRIME
PRIME_MARKS = "\u2032\u2033\u2034\u02b9\u02ba"
MARKS_SHOWN = " ".join(PRIME_MARKS)
