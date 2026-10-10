#!/usr/bin/env python3
"""K3：从冻结副本两份 checker 档源文件造扫源码的输入：gen-scan-inputs.py <冻结副本根> <输出目录>。
c* 给崩溃放量那一扫（crash_amplification.rs），g* 给卡池那一扫（gpu_unit_checks.rs）。名字里 -red- 是点名的绕法或阳性对照、
应当红；-legal- 是合法写法、应当绿（判红的去看是不是写明了故意拒）；-list- 是名单里写明拦不住的、照旧绿。"""
import sys
from pathlib import Path

frozen, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
src = frozen / "crates/singlefs-checker-tier/src"
crash = (src / "crash_amplification.rs").read_text()
gpu = (src / "gpu_unit_checks.rs").read_text()
OPENING = "#[cfg(test)]\nmod tests {"
assert crash.count(OPENING) == 1 and gpu.count(OPENING) == 1


def before_opening(text, insert):
    return text.replace(OPENING, insert + OPENING, 1)


KEEP_HIDDEN = ("macro_rules! keep_hidden { (#[cfg(test)] $($item:tt)*) => { $($item)* }; "
               "(const $name:ident : $ty:ty = $value:expr ; #[cfg(test)] $($item:tt)*) => { const $name: $ty = $value; $($item)* }; }\n")


def wrap_real_tests(text, before_inside="", brackets="()"):
    """S14：拿一个吃掉 #[cfg(test)] 的宏把真测试模块包进宏调用的圆括号（`brackets` 给 `[]` 就是方括号）里（宏定义在前，收尾补右括号与分号）。"""
    at = text.index(OPENING)
    close = text.index("\n}\n", at) + 3
    macro = KEEP_HIDDEN
    return text[:at] + macro + "keep_hidden!" + brackets[0] + "\n" + before_inside + text[at:close] + brackets[1] + ";\n" + text[close:]


LAYER = "pub fn atk_extra_layer(first_input: u8, stop_is_requested: &(dyn Fn() -> {ret} + Sync)) -> bool {{ let _ = stop_is_requested; first_input == 0 }}\n"
CHARS = ("const ATK_BRACKETS: [char; 4] = ['(', ')', '[', ']'];\nconst ATK_CLOSE: char = ')';\nconst ATK_BYTE: u8 = b']';\n"
         "const ATK_ESC: char = '\\'';\nconst ATK_U: char = '\\u{29}';\n")
LIFETIMES = "pub fn atk_longest<'a, 'b: 'a>(x: &'a str, _y: &'b str) -> &'a str { x }\npub fn atk_q<'a>(c: &'a char) -> bool { *c == '(' }\n"
STRINGS = ('const ATK_RAW: &str = r#"(("#;\nconst ATK_STR: &str = "\\")[";\nconst ATK_BSTR: &[u8] = b")";\n'
           'const ATK_CSTR: &core::ffi::CStr = c"(";\nconst ATK_RAWB: &[u8] = br#"]]"#;\n')
COMMENTS = "// )\n/* ( [ */\n/// ]\nconst ATK_AFTER_COMMENTS: u8 = 0;\n"
crash_inputs = {
    "c00-legal-frozen": crash,
    "c01-red-plain-new-layer": before_opening(crash, LAYER.format(ret="bool")),
    "c02-red-S14-real-tests-wrapped-hiding-a-layer": wrap_real_tests(crash.replace(OPENING + "\n", OPENING + "\n" + LAYER.format(ret="bool"), 1)),
    "c03-red-S14b-wrapped-after-an-item-ending-in-a-semicolon": wrap_real_tests(crash, "const BEFORE_THE_TESTS: u8 = 0;\n"),
    "c04-red-S19-wrapped-fake-tests-hiding-a-layer": before_opening(crash, "macro_rules! keep_hidden { (#[cfg(test)] $($item:tt)*) => { $($item)* }; }\nkeep_hidden!(\n" + OPENING + "\n" + LAYER.format(ret="bool") + "}\n);\n"),
    "c09-red-S14b-in-square-brackets": wrap_real_tests(crash, "const BEFORE_THE_TESTS: u8 = 0;\n", "[]"),
    "c05-red-N01-core-primitive-bool": before_opening(crash, LAYER.format(ret="::core::primitive::bool")),
    "c06-red-N02-bool-in-parentheses": before_opening(crash, LAYER.format(ret="(bool)")),
    "c07-list-N03-alias-of-bool": before_opening(crash, "type AtkFlag = bool;\n" + LAYER.format(ret="AtkFlag")),
    "c08-list-N15-type-macro": before_opening(crash, "macro_rules! atk_bool { () => { bool }; }\n" + LAYER.format(ret="atk_bool!()")),
    "c10-legal-char-literals": before_opening(crash, CHARS),
    "c11-legal-lifetimes": before_opening(crash, LIFETIMES),
    "c12-legal-strings": before_opening(crash, STRINGS),
    "c13-legal-comments": before_opening(crash, COMMENTS),
    "c14-legal-attribute-before-the-opening": before_opening(crash, "#[allow(clippy::all)]\n"),
    "c15-legal-macro-rules-before-the-opening": before_opening(crash, "macro_rules! atk_m { ($($t:tt)*) => {}; }\n"),
}
POOL = "    mod card_pool {\n"
DROP = "        impl Drop for GpuCardLease {\n"
assert gpu.count(POOL) == 1 and gpu.count(DROP) == 1
drop_start = gpu.index(DROP)
drop_end = gpu.index("\n        }\n", drop_start) + len("\n        }\n")
pinned_drop = gpu[drop_start:drop_end]
bad_drop = "        impl Drop for GpuCardLease {\n            fn drop(&mut self) {}\n        }\n"
entry_head = "            pub fn lease_the_first_card_with_room_now("
entry_start = gpu.index(entry_head)
entry_end = gpu.index("\n            }\n", entry_start) + len("\n            }\n")
pinned_entry = gpu[entry_start:entry_end]
bad_entry = pinned_entry.replace("                self.take_in_the_devices_found_idle();\n", "", 1)
assert bad_entry != pinned_entry
uses_end = gpu.index("        };\n", gpu.index(POOL)) + len("        };\n")
clean_pool_head = gpu[gpu.index(POOL):uses_end]
gpu_inputs = {
    "g00-legal-frozen": gpu,
    "g01-red-S14-real-tests-wrapped": wrap_real_tests(gpu),
    "g02-red-S14b-wrapped-after-an-item-ending-in-a-semicolon": wrap_real_tests(gpu, "const BEFORE_THE_TESTS: u8 = 0;\n"),
    "g03-red-S19-wrapped-fake-tests-hiding-a-bad-drop": before_opening(gpu, "macro_rules! keep_hidden { (#[cfg(test)] $($item:tt)*) => { $($item)* }; }\nkeep_hidden!(\n" + OPENING + "\n" + bad_drop.replace("GpuCardLease", "card_pool::GpuCardLease", 0) + "}\n);\n"),
    "g09-red-S14b-in-square-brackets": wrap_real_tests(gpu, "const BEFORE_THE_TESTS: u8 = 0;\n", "[]"),
    "g04-red-G14-clean-drop-under-cfg-then-bad": gpu[:drop_start] + "        #[cfg(any())]\n" + pinned_drop + bad_drop + gpu[drop_end:],
    "g05-red-G15-bad-drop-only": gpu[:drop_start] + bad_drop + gpu[drop_end:],
    "g06-red-G16-clean-entry-under-cfg-then-bad": gpu[:entry_start] + "            #[cfg(any())]\n" + pinned_entry + bad_entry + gpu[entry_end:],
    "g07-red-G17-bad-entry-only": gpu[:entry_start] + bad_entry + gpu[entry_end:],
    "g08-red-clean-pool-under-cfg-then-real-with-mod-wgpu": gpu.replace(POOL, "    #[cfg(any())]\n" + clean_pool_head + "    }\n" + POOL + "        mod wgpu {}\n", 1),
    "g10-legal-char-literals": before_opening(gpu, CHARS),
    "g11-legal-lifetimes": before_opening(gpu, LIFETIMES),
    "g12-legal-strings": before_opening(gpu, STRINGS),
    "g13-legal-comments": before_opening(gpu, COMMENTS),
    "g14-legal-attribute-before-the-opening": before_opening(gpu, "#[allow(clippy::all)]\n"),
    "g15-legal-macro-rules-before-the-opening": before_opening(gpu, "macro_rules! atk_m { ($($t:tt)*) => {}; }\n"),
    "g16-legal-pool-module-under-a-feature": gpu.replace(POOL, '    #[cfg(feature = "gpu")]\n' + POOL, 1),
    "g17-legal-another-module-named-card-pool": before_opening(gpu, "mod atk_other {\n    pub mod card_pool {}\n}\n"),
    "g18-legal-pinned-words-in-comments-and-strings": before_opening(gpu, '/// 见 fn lease_card、impl Drop for GpuCardLease、mod card_pool\nconst ATK_NOTE: &str = "fn lease_card Drop for GpuCardLease mod card_pool";\n'),
    "g19-legal-another-fn-lease-card-outside-the-pool": before_opening(gpu, "struct AtkMock;\nimpl AtkMock {\n    fn lease_card(&self) {}\n}\n"),
    "g20-legal-drop-under-a-cfg-attr": gpu.replace(DROP, '        #[cfg_attr(feature = "trace", allow(dead_code))]\n' + DROP, 1),
}
for name, text in {**crash_inputs, **gpu_inputs}.items():
    (out / (name + ".rs")).write_text(text)
    print(name, "opening_literal_count", text.count(OPENING))
