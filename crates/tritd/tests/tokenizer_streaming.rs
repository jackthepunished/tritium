//! Streaming detokenization against the real 2B4T vocabulary.
//!
//! `models/` is not in the repository, so these are `#[ignore]`d and run
//! explicitly, the same convention the real-checkpoint parity tests use:
//!
//! ```sh
//! cargo test -p tritd --release --test tokenizer_streaming -- --ignored
//! ```

use std::path::Path;

use trit_core::tokenizer::{Detokenizer, Tokenizer};
use tritd::tokenizer_hf::HfTokenizer;

const TOKENIZER: &str = "../../models/bitnet-2b4t/tokenizer.json";

fn load() -> Option<HfTokenizer> {
    let p = Path::new(TOKENIZER);
    p.exists().then(|| HfTokenizer::from_file(p).unwrap())
}

/// Round-trip text through encode -> per-token bytes -> Detokenizer.
///
/// The concatenation of `id_to_bytes` over a sequence must be exactly the
/// bytes of `decode` over the same sequence. Anything less means a token
/// boundary corrupted the stream, which is invisible in a one-shot decode and
/// very visible in `tritd serve`.
#[test]
#[ignore = "needs models/bitnet-2b4t/tokenizer.json"]
fn streaming_matches_batch_decoding() {
    let Some(tk) = load() else {
        panic!("tokenizer not found at {TOKENIZER}");
    };

    for text in [
        "The capital of France is Paris.",
        // Emoji: U+1F600 splits across two tokens on this vocabulary, and the
        // second carries a lone continuation byte.
        "hello 😀 world",
        "🇹🇷 flags and 👨‍👩‍👧‍👦 sequences",
        "日本語のテキスト",
        "café naïve — em-dash",
        "mixed 😀 日本 café 🚀 end",
    ] {
        let ids = tk.encode(text, false).unwrap();

        let mut streamed = String::new();
        let mut d = Detokenizer::new();
        for &id in &ids {
            streamed.push_str(&d.push(&tk, id).unwrap());
        }
        streamed.push_str(&d.flush());

        assert_eq!(
            streamed,
            tk.decode(&ids, false).unwrap(),
            "streamed text diverged from batch decode for {text:?}"
        );
        assert_eq!(streamed, text, "round trip lost data for {text:?}");
    }
}

/// The specific case that was broken: neither half of a split codepoint may
/// come back as U+FFFD.
#[test]
#[ignore = "needs models/bitnet-2b4t/tokenizer.json"]
fn a_split_codepoint_keeps_its_real_bytes() {
    let Some(tk) = load() else {
        panic!("tokenizer not found at {TOKENIZER}");
    };
    let ids = tk.encode("😀", false).unwrap();
    assert!(ids.len() > 1, "expected the emoji to split; got {ids:?}");

    let joined: Vec<u8> = ids
        .iter()
        .flat_map(|&id| tk.id_to_bytes(id).unwrap())
        .collect();
    assert_eq!(
        joined,
        "😀".as_bytes(),
        "per-token bytes must concatenate to the original, not to replacement characters"
    );
    assert!(
        !joined.windows(3).any(|w| w == [0xEF, 0xBF, 0xBD]),
        "a replacement character survived into the byte stream"
    );
}
