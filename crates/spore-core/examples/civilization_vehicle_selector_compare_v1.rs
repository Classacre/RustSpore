//! Compare retained eight-row synthetic selector output, not live gameplay.
use spore_core::civilization_vehicle_selector_v1::{decode_model_keys, select};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("pass retained stdout path")?;
    let bytes = std::fs::read(path)?;
    if bytes.len() != 256 { return Err("expected eight 32-byte rows".into()); }
    let records = [0x018e_a1eb,0x018e_b106,0x018e_a2cc,0x01a5_6aba,0,0x018e_a2cc,0x018e_a2cc,0x018e_a2cc];
    // The matching original host guards this synthetic stream and every loaded
    // vector word; native stdout emits only the subsequent selector rows.
    let mut stream = 16u32.to_be_bytes().to_vec();
    for i in 0..16u32 {
        for w in [102 + 3*i, 101 + 3*i, 100 + 3*i] { stream.extend_from_slice(&w.to_be_bytes()); }
    }
    let triples: [[u32; 3]; 16] = decode_model_keys(&stream, 16)?.try_into().map_err(|_| "count")?;
    for (case, record) in records.into_iter().enumerate() {
        let mut selectors = [0u32; 16];
        selectors[..4].copy_from_slice(&[0x99e9_2f05,0x4e3f_7777,0x47c1_0953,0x72c4_9181]);
        if case >= 5 { selectors[2] = 0; }
        if case == 6 { selectors[4] = 0x47c1_0953; selectors[8] = 0x47c1_0953; }
        if case == 7 { selectors[15] = 0x47c1_0953; }
        let (index, triple) = select(record, &selectors, &triples);
        let mut row = [0u32; 8];
        for (j, word) in row.iter_mut().enumerate() {
            let at = case*32+j*4;
            *word = u32::from_le_bytes(bytes[at..at+4].try_into()?);
        }
        if row[..4] != [index as u32, triple[0], triple[1], triple[2]]
            || row[4] & 0xffff != 0x027f || (row[4] >> 16) & 0x3840 != 0
            || row[5] & !0x3f != 0x1f80 || row[6..] != [0,0] {
            return Err(format!("row {case} differs").into());
        }
    }
    println!("matched eight controlled selector rows; no live vehicle or asset claim");
    Ok(())
}
