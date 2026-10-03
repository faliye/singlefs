use model_core::pack::pack;
use model_core::word::Word;

#[test]
fn pack_puts_high_above_low() {
    assert_eq!(pack(1, 2), 0x102u64 as Word);
}
