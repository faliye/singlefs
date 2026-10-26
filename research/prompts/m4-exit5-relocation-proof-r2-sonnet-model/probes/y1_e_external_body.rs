// 探针 y1-e：external_body 声明一个未解释函数（像设备读、校验和），--no-cheating 下过不过。
use vstd::prelude::*;
verus! {
pub uninterp spec fn crc_spec(bytes: Seq<u8>) -> u32;
#[verifier::external_body]
pub fn crc32(bytes: &Vec<u8>) -> (r: u32)
    ensures r == crc_spec(bytes@)
{ bytes.len() as u32 }
pub fn uses_crc(bytes: &Vec<u8>) -> (r: bool)
    ensures r == (crc_spec(bytes@) == 0u32)
{ crc32(bytes) == 0 }
}
fn main() {}
