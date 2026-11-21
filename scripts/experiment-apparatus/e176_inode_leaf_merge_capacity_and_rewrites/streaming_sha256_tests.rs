use super::*;

fn one_shot_digest(bytes: &[u8]) -> String {
    let mut hasher = StreamingSha256::new();
    hasher.absorb_bytes(bytes);
    hasher.hexadecimal_digest()
}

#[test]
fn sha256_matches_the_published_test_vectors_whether_fed_at_once_or_in_pieces() {
    assert_eq!(one_shot_digest(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(one_shot_digest(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let two_blocks = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    assert_eq!(one_shot_digest(two_blocks), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    let mut in_pieces = StreamingSha256::new();
    for piece in two_blocks.chunks(9) {
        in_pieces.absorb_bytes(piece);
    }
    assert_eq!(in_pieces.hexadecimal_digest(), one_shot_digest(two_blocks), "分块喂与一次喂同一个摘要");
}
