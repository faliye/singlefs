"""m4-i315-r3 辩方腿：给副本里 harness 测试共用的搭建（tests/common/mod.rs）加两样，供固定盘面造「同一个身份、不同载荷」与「别的池同一个 txg」的旧住户：
1. build_pool_with_first_file(tag, content)：build_pool 的全部步骤，第一个文件的内容由调用方给；build_pool(tag) 照旧用 file_content()。
2. set_fsid_tweak(n)：之后 parameters() 返回的文件系统身份前 8 字节里第 4 字节异或 n（造「重新 mkfs 的池」，n 为 0 还原）。
用法：python3 -I apply_common_r3.py <副本根>"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
path = root / "crates/singlefs-harness/tests/common/mod.rs"
t = path.read_text()
def once(old, new, label):
    global t
    if t.count(old) != 1:
        raise SystemExit(f"{label}: 旧串命中 {t.count(old)} 次")
    t = t.replace(old, new)
once('''pub fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,''', '''static FSID_TWEAK: AtomicU64 = AtomicU64::new(0);
/// 辩方腿副本：之后 parameters() 的文件系统身份第 4 字节异或 n（0 还原）。
pub fn set_fsid_tweak(n: u64) {
    FSID_TWEAK.store(n, Ordering::SeqCst);
}
pub fn parameters() -> MakeFilesystemParameters {
    let mut filesystem_identifier = E142_FILESYSTEM_IDENTIFIER;
    filesystem_identifier[3] ^= u8::try_from(FSID_TWEAK.load(Ordering::SeqCst)).expect("异或值装得进一个字节");
    MakeFilesystemParameters {
        filesystem_identifier,''', "params")
once('''pub fn build_pool(tag: &str) -> BuiltPool {
    let parameters = parameters();''', '''pub fn build_pool(tag: &str) -> BuiltPool {
    build_pool_with_first_file(tag, &file_content())
}

pub fn build_pool_with_first_file(tag: &str, content: &[u8]) -> BuiltPool {
    let parameters = parameters();''', "build")
once('''    let content = file_content();
    let warm_up_operation_count;''', '''    let warm_up_operation_count;''', "content")
once('''            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up.last_record_bytes,''', '''            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up.last_record_bytes,''', "firstfile")
once('''pub fn build_pool_with_first_file(tag: &str, content: &[u8]) -> BuiltPool {
    let parameters = parameters();''', '''pub fn build_pool_with_first_file(tag: &str, content: &[u8]) -> BuiltPool {
    build_pool_with_first_file_as_instance(tag, content, 1)
}

/// 辩方腿副本：取号 `acquisitions` 次，之后的暖机与第一个文件都以最后取到的实例代号写（造「旧实例留下的单元」与「后一个实例写的单元」）。
pub fn build_pool_with_first_file_as_instance(tag: &str, content: &[u8], acquisitions: u32) -> BuiltPool {
    let parameters = parameters();''', "instances")
once('''        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));''', '''        let mut instance = acquire_instance(&mut pool).expect("取号");
        for _ in 1..acquisitions {
            instance = acquire_instance(&mut pool).expect("再取号");
        }
        assert_eq!(instance, InstanceGeneration(acquisitions));''', "acquire")
path.write_text(t)
print("ok common")
