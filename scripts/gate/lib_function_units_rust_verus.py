"""verus! 里的规格与证明代码的识别与剥除（VerusGhostCode、verus_ghost_code_blanked），产品行为视图与 Rust 那一份对拍用的判断。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
from lib_function_units_lexer import IDENT, LITERAL, PUNCT, active_breaks, match_brackets, rust_lex


# 产品行为视图里只给 Verus 看的那几截（crates/singlefs-unit-graph 的 verus_ghost_code.rs 文件头列的那几种；那一份按 verus_syn 的语法树挑，
# 这里按词法单元认同样的写法，认出来的要逐单元核得上 Rust 那一份才用：RustWorkspace.product_view_file）
VERUS_GHOST_FUNCTION_MODES = frozenset(("spec", "proof", "axiom"))
VERUS_SIGNATURE_CLAUSES = frozenset(("requires", "recommends", "ensures", "default_ensures", "returns", "decreases", "opens_invariants",
                                     "no_unwind", "with", "by", "atomically"))
VERUS_LOOP_CLAUSES = frozenset(("invariant", "invariant_except_break", "invariant_ensures", "ensures", "decreases"))
VERUS_GHOST_CALLS = frozenset(("assume", "reveal", "reveal_with_fuel", "hide"))
VERUS_GHOST_WRAPPERS = frozenset(("Ghost", "Tracked"))
# verus! 里同层条目的起头词（crates/singlefs-unit-graph 的 is_verus_item_start）
VERUS_ITEM_START_WORDS = frozenset(("pub", "fn", "spec", "proof", "exec", "open", "closed", "broadcast", "struct", "enum", "union", "impl",
                                    "trait", "const", "static", "type", "use", "mod", "unsafe", "extern", "macro_rules", "global", "tracked",
                                    "ghost", "uninterp", "axiom", "async", "default"))
# 前一个词是这些时后面的 ( 不是调用的实参表
VERUS_NOT_CALLEES = frozenset(("if", "while", "match", "return", "in", "for", "let", "loop", "else", "move", "mut", "ref", "as", "fn", "impl",
                               "where", "dyn", "break", "continue", "unsafe", "await", "yield", "box"))
# 一对 { … } 之后接这些时，它还在循环头的子句里（不是循环体）
VERUS_CLAUSE_CONTINUATIONS = frozenset((",", "{", ".", "?", "=", "!", "<", ">", "&", "|", "+", "-", "*", "/", "%", "^", "@", "as", "implies",
                                        "is", "has", "else")) | VERUS_LOOP_CLAUSES


class VerusGhostCode:
    """一份 Rust 文件里每一块 verus! { … } 中只给 Verus 看的那几截（照 crates/singlefs-unit-graph 的 GhostCodeCollector 挑）：
    spec、proof、axiom 函数与常量整条（连属性与文档注释）、broadcast group、broadcast use、assume_specification；执行函数签名里的规格子句
    （requires 一族到函数体之前）、具名返回值的「(名:」与「)」、类型是 Ghost<…> / Tracked<…> 的参数；函数体与常量初值里的 proof { … }、assert(…)（连 by …）、
    assert forall、assume、reveal 一族、let ghost / let tracked、循环头的 invariant 一族与 ensures、decreases、for 循环给迭代器起的名；
    ghost / tracked 字段与类型是 Ghost<…> / Tracked<…> 的字段，调用、方法调用的实参与结构体字面量、结构体模式里整个是 Ghost(…) / Tracked(…) 的那几样
    （一列里剥几样时连逗号，照那一份 blank_the_chosen_elements_of_a_list）。宏调用里面不看（verus_syn 不解它）。
    collect() → [(首词法单元, 末词法单元之后, 连不连紧挨在前面的文档注释)]。"""

    def __init__(self, kinds, texts, pairs):
        self.kinds, self.texts, self.pairs = kinds, texts, pairs
        self.count = len(texts)
        self.openings = {close: opening for opening, close in pairs.items()}
        self.ranges = []

    def t(self, index):
        return self.texts[index] if 0 <= index < self.count else ""

    def ident(self, index):
        return 0 <= index < self.count and self.kinds[index] == IDENT

    def group(self, index):
        return self.t(index) in ("(", "[", "{") and index in self.pairs

    def blank(self, first, end, with_documents=False):
        if first < end:
            self.ranges.append((first, end, with_documents))

    def collect(self):
        index = 0
        while index + 2 < self.count:
            if self.ident(index) and self.t(index) == "verus" and self.t(index + 1) == "!" and self.t(index + 2) == "{" and (index + 2) in self.pairs:
                close = self.pairs[index + 2]
                self.items(index + 3, close)
                index = close + 1
                continue
            index += 1
        return self.ranges

    # 条目

    def is_item_start(self, index):
        text = self.t(index)
        if text == "#":
            return self.t(index + 1) == "["
        if not self.ident(index):
            return False
        if text in VERUS_ITEM_START_WORDS:
            return True
        if text == "assume_specification":
            return self.t(index + 1) in ("[", "<")
        return self.t(index + 1) == "!" and self.group(index + 2)

    def items(self, low, high):
        """low..high 里同层的条目：块首，或顶层 ; / } 之后接条目起头词或属性（crates/singlefs-unit-graph 的 verus_item_starts）。"""
        starts, boundary, index = [], True, low
        while index < high:
            if boundary and self.is_item_start(index):
                starts.append(index)
            if self.group(index):
                close = self.pairs[index]
                boundary = self.t(close) == "}"
                index = close + 1
                continue
            boundary = self.t(index) == ";"
            index += 1
        for position, start in enumerate(starts):
            self.item(start, starts[position + 1] if position + 1 < len(starts) else high)

    def skip_attributes(self, index, end):
        while index < end and self.t(index) == "#" and self.t(index + 1) == "[" and (index + 1) in self.pairs:
            index = self.pairs[index + 1] + 1
        return index

    def skip_visibility(self, index):
        if self.t(index) == "pub":
            index += 1
            if self.t(index) == "(" and index in self.pairs:
                index = self.pairs[index] + 1
        return index

    def skip_qualifiers(self, index):
        """→ (条目关键字的下标, 跳过的修饰词)；照 crates/singlefs-unit-graph 的 skip_qualifiers（verus 为真）。"""
        skipped = set()
        while True:
            word, following = self.t(index), self.t(index + 1)
            if word == "const":
                qualifier = following in ("fn", "unsafe", "async", "extern")
            elif word == "async":
                qualifier = following in ("fn", "unsafe")
            elif word == "unsafe":
                qualifier = following in ("fn", "impl", "trait", "extern", "auto")
            elif word == "extern":
                qualifier = following == "fn" or (0 <= index + 1 < self.count and self.kinds[index + 1] == LITERAL and self.t(index + 2) == "fn")
                if qualifier and following != "fn":
                    index += 1
            elif word == "default":
                qualifier = following in ("fn", "type", "const", "unsafe", "async", "impl")
            elif word == "safe":
                qualifier = following in ("fn", "static")
            elif word == "auto":
                qualifier = following == "trait"
            elif word in ("open", "closed", "spec", "proof", "exec", "tracked", "ghost", "uninterp", "axiom"):
                qualifier = True
            elif word == "broadcast":
                qualifier = following in ("proof", "fn", "spec", "axiom", "uninterp", "open", "closed")
            else:
                qualifier = False
            if not qualifier:
                return index, skipped
            skipped.add(word)
            index += 1

    def item(self, start, end):
        header = self.skip_attributes(start, end)
        keyword, modes = self.skip_qualifiers(self.skip_visibility(header))
        word = self.t(keyword)
        ghost = bool(modes & VERUS_GHOST_FUNCTION_MODES)
        if word in ("fn", "const", "static") and ghost:
            self.blank(start, end, True)
        elif word == "fn":
            self.function(keyword, end)
        elif word in ("const", "static"):
            self.expressions(keyword, end)
        elif (word == "broadcast" and self.t(keyword + 1) in ("use", "group")) or (word == "assume_specification" and self.t(keyword + 1) in ("[", "<")):
            self.blank(start, end, True)
        elif word in ("struct", "union", "enum") and self.ident(keyword + 1):
            self.data_type(word, keyword, end)
        elif word in ("impl", "trait"):
            body = self.first_brace(keyword + 1, end)
            if body is not None:
                self.items(body + 1, self.pairs[body])
        elif word == "mod" and self.t(keyword + 2) == "{" and (keyword + 2) in self.pairs:
            self.items(keyword + 3, self.pairs[keyword + 2])

    def first_brace(self, low, high):
        """low 起顶层第一个 {（先遇到 ; 交 None）。"""
        index = low
        while index < high:
            text = self.t(index)
            if text == "{" and index in self.pairs:
                return index
            if text == ";":
                return None
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        return None

    def after_angle_brackets(self, opening, end):
        """opening 那个 < 配到它的 >，交 > 之后的下标（-> 与 => 的 > 不算）；配不上交 None。"""
        depth = 0
        for position in range(opening, end):
            text = self.t(position)
            if text == "<":
                depth += 1
            elif text == ">" and self.t(position - 1) not in ("-", "="):
                depth -= 1
                if depth == 0:
                    return position + 1
                if depth < 0:
                    return None
        return None

    def single_colon(self, low, high):
        """low..high 顶层第一个单独的 :（不是 :: 的一半）。"""
        index = low
        while index < high:
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if self.t(index) == ":" and self.t(index - 1) != ":" and self.t(index + 1) != ":":
                return index
            index += 1
        return None

    def first_word(self, low, high, words):
        index = low
        while index < high:
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if self.ident(index) and self.t(index) in words and self.t(index - 1) not in (".", ":"):
                return index
            index += 1
        return None

    def elements(self, low, high, types):
        """low..high 按顶层逗号分开的几样 → [(起, 止, 后面那个逗号的下标或 None)]；types 为真时尖括号里的逗号不算（类型里的 Map<K, V>），
        否则只有 ::< 起的尖括号（turbofish）里的不算。"""
        parts, start, index, angles = [], low, low, 0
        while index < high:
            text = self.t(index)
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if text == "<" and (types or angles > 0 or (self.t(index - 1) == ":" and self.t(index - 2) == ":")):
                angles += 1
            elif text == ">" and angles > 0 and self.t(index - 1) not in ("-", "="):
                angles -= 1
            elif text == "," and angles == 0:
                parts.append((start, index, index))
                start = index + 1
            index += 1
        if start < high:
            parts.append((start, high, None))
        return parts

    def chosen_elements(self, low, high, is_chosen, types=False, with_documents=False):
        """一列里剥掉挑中的那几样连它们后面的逗号；挑中的有一样在列尾、后面没有逗号的，留下的那几样里最后一样后面的逗号也剥
        （crates/singlefs-unit-graph 的 blank_the_chosen_elements_of_a_list：剥完与压根没写那几样时逐个词法单元相同）。"""
        last_kept_comma, chosen_last_without_comma = None, False
        for first, end, comma in self.elements(low, high, types):
            if not is_chosen(first, end):
                last_kept_comma = comma
                continue
            self.blank(first, end, with_documents)
            if comma is None:
                chosen_last_without_comma = True
            else:
                self.blank(comma, comma + 1)
        if chosen_last_without_comma and last_kept_comma is not None:
            self.blank(last_kept_comma, last_kept_comma + 1)

    def is_wrapper_type(self, first, end):
        """first..end 整个是 Ghost<…> / Tracked<…>：路径（可带 :: 起头）末段是包装名、带尖括号参数，尖括号配到 end 为止。"""
        index = first + 2 if self.t(first) == ":" and self.t(first + 1) == ":" else first
        while self.ident(index) and self.t(index + 1) == ":" and self.t(index + 2) == ":":
            index += 3
        if not (self.ident(index) and self.t(index) in VERUS_GHOST_WRAPPERS and self.t(index + 1) == "<"):
            return False
        return self.after_angle_brackets(index + 1, end) == end

    def is_wrapper_parameter(self, first, end):
        """类型是 Ghost<…> / Tracked<…> 的参数（连模式）；self 不算。"""
        index = self.skip_attributes(first, end)
        colon = self.single_colon(index, end)
        if colon is None or any(self.t(position) == "self" for position in range(index, colon)):
            return False
        return self.is_wrapper_type(colon + 1, end)

    def is_wrapper_field(self, first, end, named):
        """ghost / tracked 模式的字段，或类型是 Ghost<…> / Tracked<…> 的字段。"""
        index = self.skip_visibility(self.skip_attributes(first, end))
        if named:
            mode = self.t(index) in ("ghost", "tracked") and self.ident(index + 1) and self.t(index + 2) == ":" and self.t(index + 3) != ":"
            name = index + 1 if mode else index
            if not (self.ident(name) and self.t(name + 1) == ":"):
                return False
            return mode or self.is_wrapper_type(name + 2, end)
        mode = self.t(index) in ("ghost", "tracked") and index + 1 < end
        return mode or self.is_wrapper_type(index, end)

    def is_wrapper_call(self, first, end):
        """first..end 整个是 Ghost(…) / Tracked(…)（可带 turbofish）：不带限定、只有一段路径、恰好一个实参。"""
        if not (self.ident(first) and self.t(first) in VERUS_GHOST_WRAPPERS):
            return False
        index = first + 1
        if self.t(index) == ":" and self.t(index + 1) == ":" and self.t(index + 2) == "<":
            index = self.after_angle_brackets(index + 2, end)
            if index is None:
                return False
        if self.t(index) != "(" or self.pairs.get(index) != end - 1:
            return False
        return len(self.elements(index + 1, end - 1, False)) == 1

    def is_wrapper_field_value(self, first, end):
        """结构体字面量里初值、结构体模式里子模式整个是 Ghost(…) / Tracked(…) 的字段。"""
        return (self.ident(first) and self.t(first + 1) == ":" and self.t(first + 2) != ":"
                and self.is_wrapper_call(first + 2, end))

    def data_type(self, word, keyword, end):
        index = keyword + 2
        if self.t(index) == "<":
            index = self.after_angle_brackets(index, end)
            if index is None:
                return
        if word == "struct" and self.t(index) == "(" and index in self.pairs:
            self.fields(index, False)
            return
        body = self.first_brace(index, end)
        if body is None:
            return
        if word != "enum":
            self.fields(body, True)
            return
        for first, variant_end, _comma in self.elements(body + 1, self.pairs[body], False):
            name = self.skip_attributes(first, variant_end)
            if self.t(name + 1) in ("{", "(") and (name + 1) in self.pairs:
                self.fields(name + 1, self.t(name + 1) == "{")

    def fields(self, opening, named):
        self.chosen_elements(opening + 1, self.pairs[opening], lambda first, end: self.is_wrapper_field(first, end, named),
                             types=True, with_documents=True)

    # 执行函数

    def function(self, keyword, end):
        index = keyword + 2
        if self.t(index) == "<":
            index = self.after_angle_brackets(index, end)
        if index is None or self.t(index) != "(" or index not in self.pairs:
            return
        parameters_close = self.pairs[index]
        self.chosen_elements(index + 1, parameters_close, self.is_wrapper_parameter, types=True)
        last = end - 1
        if self.t(last) == "}" and last in self.openings:
            body, signature_end = self.openings[last], self.openings[last]
        elif self.t(last) == ";":
            body, signature_end = None, last
        else:
            return
        after = parameters_close + 1
        if self.t(after) == "-" and self.t(after + 1) == ">" and self.t(after + 2) == "(" and (after + 2) in self.pairs:
            opening = after + 2
            colon = self.single_colon(opening + 1, self.pairs[opening])
            if colon is not None:
                self.blank(opening, colon + 1)
                self.blank(self.pairs[opening], self.pairs[opening] + 1)
        clause = self.first_word(after, signature_end, VERUS_SIGNATURE_CLAUSES)
        if clause is not None:
            self.blank(clause, signature_end)
        if body is not None:
            self.expressions(body + 1, last)

    def statement_end(self, index, high):
        """index 起顶层第一个 ; 之后（没有就是 high）。"""
        while index < high:
            if self.group(index):
                index = self.pairs[index] + 1
                continue
            if self.t(index) == ";":
                return index + 1
            index += 1
        return high

    def after_by(self, index, high):
        """assert(…) 之后的 by …：by 后面可带 (…)，再带 requires … 与 { … }，或直接 { … }。交整条之后的下标。"""
        if self.t(index) != "by":
            return index
        index += 1
        if self.t(index) == "(" and index in self.pairs:
            index = self.pairs[index] + 1
        if self.t(index) == "requires":
            while index < high:
                if self.t(index) == "{" and index in self.pairs:
                    return self.pairs[index] + 1
                if self.t(index) == ";":
                    return index
                index = self.pairs[index] + 1 if self.group(index) else index + 1
            return high
        if self.t(index) == "{" and index in self.pairs:
            return self.pairs[index] + 1
        return index

    def assert_forall_end(self, index, high):
        """assert forall … 到顶层的 ;（不含）或 by { … } 收尾。"""
        while index < high:
            text = self.t(index)
            if text == ";":
                return index
            if text == "by":
                after = index + 1
                if self.t(after) == "(" and after in self.pairs:
                    after = self.pairs[after] + 1
                if self.t(after) == "{" and after in self.pairs:
                    return self.pairs[after] + 1
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        return high

    def ghost_expression_end(self, index, high):
        """index 起是整条只给 Verus 看的表达式（proof { … }、assert、assert forall、assume、reveal 一族）就交它之后的下标，否则 None。"""
        text, following = self.t(index), self.t(index + 1)
        if text == "proof" and following == "{" and (index + 1) in self.pairs:
            return self.pairs[index + 1] + 1
        if text == "assert" and following == "(" and (index + 1) in self.pairs:
            return self.after_by(self.pairs[index + 1] + 1, high)
        if text == "assert" and following == "forall":
            return self.assert_forall_end(index + 2, high)
        if text in VERUS_GHOST_CALLS and following == "(" and (index + 1) in self.pairs:
            return self.pairs[index + 1] + 1
        return None

    def ends_a_loop_head(self, close, high):
        """循环头里一对 { … } 之后接的不像子句的接续（逗号、运算符、子句词……）：它就是循环体。"""
        following = close + 1
        return following >= high or self.t(following) not in VERUS_CLAUSE_CONTINUATIONS

    def clause_body(self, clause, high):
        """clause 起的子句之后那个体的 {（照 ends_a_loop_head 认）；先遇到 ; 或到 high 都没认出交 None。"""
        index = clause + 1
        while index < high:
            if self.t(index) == "{" and index in self.pairs:
                if self.ends_a_loop_head(self.pairs[index], high):
                    return index
                index = self.pairs[index] + 1
                continue
            if self.t(index) == ";":
                return None
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        return None

    def loop_head(self, keyword, high):
        """while / loop / for 的头：for 给迭代器起的名（in 之后的「名:」）与 invariant 一族、ensures、decreases 那几截到循环体之前。"""
        is_for, index, clause = self.t(keyword) == "for", keyword + 1, None
        while index < high:
            text = self.t(index)
            if (is_for and clause is None and text == "in" and self.ident(index + 1) and self.t(index + 2) == ":"
                    and self.t(index + 3) != ":"):
                self.blank(index + 1, index + 3)
            if clause is None and self.ident(index) and text in VERUS_LOOP_CLAUSES:
                clause = index
            if text == "{" and index in self.pairs:
                if clause is None or self.ends_a_loop_head(self.pairs[index], high):
                    break
                index = self.pairs[index] + 1
                continue
            if text == ";":
                return
            index = self.pairs[index] + 1 if self.group(index) else index + 1
        else:
            return
        if clause is not None:
            self.blank(clause, index)

    def is_call(self, opening):
        """opening 那个 ( 是调用或方法调用的实参表：前面是名字（不是关键字、不是 fn 名）、) 或 ]，或 ::<…> 的 >。"""
        before = self.t(opening - 1)
        if before in (")", "]"):
            return True
        if before == ">":
            depth, index = 0, opening - 1
            while index >= 0:
                if self.t(index) == ">":
                    depth += 1
                elif self.t(index) == "<":
                    depth -= 1
                    if depth == 0:
                        return self.t(index - 1) == ":" and self.t(index - 2) == ":"
                index -= 1
            return False
        return self.ident(opening - 1) and before not in VERUS_NOT_CALLEES and self.t(opening - 2) != "fn"

    def expressions(self, low, high):
        """函数体、常量初值这一段：剥整条只给 Verus 看的表达式与语句（当语句时连分号），循环头的子句，实参与字面量字段里的 Ghost(…) / Tracked(…)。"""
        index = low
        while index < high:
            text = self.t(index)
            if self.ident(index) and self.t(index + 1) == "!" and self.group(index + 2):
                index = self.pairs[index + 2] + 1
                continue
            if self.ident(index) and self.t(index - 1) not in (".", ":", "fn"):
                end = self.ghost_expression_end(index, high)
                if end is None and ((text == "let" and self.t(index + 1) in ("ghost", "tracked"))
                                    or (text == "broadcast" and self.t(index + 1) == "use")):
                    end = self.statement_end(index, high)
                elif end is not None and end < high and self.t(end) == ";" and (index == low or self.t(index - 1) in (";", "{", "}")):
                    end += 1
                if end is None and text in ("requires", "ensures"):
                    # 闭包的 requires / ensures（循环头里的已由 loop_head 剥过，这里再剥一遍是同一截）：到闭包体之前
                    end = self.clause_body(index, high)
                if end is not None:
                    self.blank(index, end)
                    index = end
                    continue
                if text in ("while", "loop", "for"):
                    self.loop_head(index, high)
            if self.group(index):
                close = self.pairs[index]
                if text == "(" and self.is_call(index):
                    self.chosen_elements(index + 1, close, self.is_wrapper_call)
                elif text == "{":
                    self.chosen_elements(index + 1, close, self.is_wrapper_field_value)
                self.expressions(index + 1, close)
                index = close + 1
                continue
            index += 1


VERIFIER_ATTRIBUTE_HEAD = "verifier"
OPENING_PUNCTUATION = ("{", "(", "[")
CLOSING_PUNCTUATION = ("}", ")", "]")


def verifier_attribute_token_ranges(kinds, texts):
    """外属性 #[verifier::…] 与括号写法 #[verifier(…)] 各从 # 到配对的 ] 的词法单元下标区间 [(# 的下标, ] 的下标)]，
    照 crates/singlefs-unit-graph 的 verifier_attribute_ranges_of：verus! 里外同一套认法；不认的有内属性 #![verifier::…]、#[cfg_attr(…, verifier::…)]、
    落在别的宏调用与 macro_rules! 的词法单元组里的（左括号前面紧挨 名 !，或 macro_rules ! 名；verus ! { 不算别的宏）、配不上 ] 的。"""
    if VERIFIER_ATTRIBUTE_HEAD not in texts:
        return []
    count = len(texts)

    def punctuation(index, wanted):
        return 0 <= index < count and kinds[index] == PUNCT and texts[index] == wanted

    def closing_bracket(opening):
        depth = 0
        for index in range(opening, count):
            if kinds[index] != PUNCT:
                continue
            if texts[index] in OPENING_PUNCTUATION:
                depth += 1
            elif texts[index] in CLOSING_PUNCTUATION:
                depth -= 1
                if depth == 0:
                    return index
        return None

    def attribute_closing_bracket(hash_index):
        opens_a_verifier_attribute = (punctuation(hash_index, "#") and punctuation(hash_index + 1, "[") and hash_index + 2 < count
                                      and kinds[hash_index + 2] == IDENT and texts[hash_index + 2] == VERIFIER_ATTRIBUTE_HEAD
                                      and ((punctuation(hash_index + 3, ":") and punctuation(hash_index + 4, ":")) or punctuation(hash_index + 3, "(")))
        return closing_bracket(hash_index + 1) if opens_a_verifier_attribute else None

    def macro_name_opening(opening):
        if opening >= 2 and punctuation(opening - 1, "!") and kinds[opening - 2] == IDENT:
            return texts[opening - 2]
        if (opening >= 3 and kinds[opening - 1] == IDENT and punctuation(opening - 2, "!") and kinds[opening - 3] == IDENT
                and texts[opening - 3] == "macro_rules"):
            return "macro_rules"
        return None

    # 每一层还开着的括号：这一层（连外层）落没落在别的宏的词法单元组里
    inside_another_macro, ranges, index = [], [], 0
    while index < count:
        if not (inside_another_macro and inside_another_macro[-1]):
            close = attribute_closing_bracket(index)
            if close is not None:
                ranges.append((index, close))
                index = close + 1
                continue
        if kinds[index] == PUNCT and texts[index] in OPENING_PUNCTUATION:
            name = macro_name_opening(index)
            opens_another_macro = name is not None and (name != "verus" or texts[index] != "{")
            inside_another_macro.append(bool(inside_another_macro and inside_another_macro[-1]) or opens_another_macro)
        elif kinds[index] == PUNCT and texts[index] in CLOSING_PUNCTUATION and inside_another_macro:
            inside_another_macro.pop()
        index += 1
    return ranges


def verus_ghost_code_blanked(text):
    """text 里每一块 verus! 中只给 Verus 看的那几截、verus! 里外的 #[verifier::…] / #[verifier(…)] 外属性（verifier_attribute_token_ranges）换成空格
    （换行留着，行号不变；VerusGhostCode 挑的那几截，各从首词法单元起到末词法单元止，整条剥的连紧挨在前面的文档注释）→ 剥过的源码；
    括号配不上的交 None（那份文件照原样那一份办）。弄坏开关 product-view-verifier-attributes-kept 下不剥属性。"""
    spans, document_spans = [], []
    kinds, texts, _lines, documents = rust_lex(text, spans, document_spans)
    pairs, _bad = match_brackets(kinds, texts)
    if pairs is None:
        return None
    ranges = VerusGhostCode(kinds, texts, pairs).collect()
    attributes = [] if "product-view-verifier-attributes-kept" in active_breaks() else [
        (spans[hash_index][0], spans[close][1]) for hash_index, close in verifier_attribute_token_ranges(kinds, texts)]
    if not ranges and not attributes:
        return text
    leading_documents = {}
    for (_line, _style, _rows, next_index, _raw), (start, _end) in zip(documents, document_spans):
        leading_documents[next_index] = min(start, leading_documents.get(next_index, start))
    characters = list(text)
    for first, end, with_documents in ranges:
        start = spans[first][0]
        if with_documents and first in leading_documents:
            start = min(start, leading_documents[first])
        for position in range(start, spans[end - 1][1]):
            if characters[position] != "\n":
                characters[position] = " "
    for start, end in attributes:
        for position in range(start, end):
            if characters[position] != "\n":
                characters[position] = " "
    return "".join(characters)


def product_view_agrees(rust_file, theirs_lines):
    """剥过的那一份重切出来的单元与 Rust 那一份交的这份文件的单元行逐个相同：种类、文件:起-止、指纹的多重集相同，
    Rust 那一份的限定名以这一份的嵌套路径收尾（前缀要等模块树连好才有，这里不比；文档测试不比名字）。"""
    theirs, names = collections.Counter(), {}
    for line in theirs_lines:
        kind, name, place, fingerprint = line.split("\t")
        theirs[(kind, place, fingerprint)] += 1
        names.setdefault((kind, place, fingerprint), []).append(name)
    ours = collections.Counter((unit.output_kind(), f"{unit.file}:{unit.first_line}-{unit.last_line}", unit.fingerprint) for unit in rust_file.units)
    if ours != theirs:
        return False
    return all(unit.kind == "doctest" or any(name == unit.name or name.endswith("::" + unit.name)
                                             for name in names[(unit.output_kind(), f"{unit.file}:{unit.first_line}-{unit.last_line}", unit.fingerprint)])
               for unit in rust_file.units)
