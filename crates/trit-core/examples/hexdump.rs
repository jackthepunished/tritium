fn main() {
    let dir = std::path::Path::new("/tmp/tritspec");
    let path = dir.join("example.trit");
    let mut w = trit_core::tritfmt::TritWriter::create(&path, r#"{"hidden_size":3}"#).unwrap();
    // W = [[+1,-1,0],[0,+1,+1]], scale 0.5
    w.write_trit("w", &[2, 3], &[1, -1, 0, 0, 1, 1], 0.5).unwrap();
    w.finish().unwrap();
    let b = std::fs::read(&path).unwrap();
    println!("total {} bytes", b.len());
    for (i, chunk) in b.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|x| format!("{x:02x}")).collect();
        let asc: String = chunk.iter().map(|&c| if (32..127).contains(&c) { c as char } else { '.' }).collect();
        println!("{:04x}  {:<47}  |{}|", i * 16, hex.join(" "), asc);
    }
}
