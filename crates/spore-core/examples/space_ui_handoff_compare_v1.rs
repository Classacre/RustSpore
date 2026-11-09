//! Compare only the two retained scripted native handoff rows.
use space_ui_handoff_v1::{events, Event};
use spore_core::space_ui_handoff_v1;

fn main() {
    let path = std::env::args_os().nth(1).expect("native stdout path");
    let bytes = std::fs::read(path).expect("retained native stdout");
    assert_eq!(bytes.len(), 96);
    for (index, row) in bytes.chunks_exact(48).enumerate() {
        let words: Vec<u32> = row.chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect();
        let modeled = events(index == 0);
        assert_eq!(words[0] as usize, modeled.len());
        for (i, event) in modeled.iter().enumerate() {
            let code = match event {
                Event::GetReceiver => 1, Event::Locale => 2, Event::SetText => 3,
                Event::SetId => 4, Event::Finish => 5,
            };
            assert_eq!(words[i + 1], code);
        }
        assert!(words[1 + modeled.len()..8].iter().all(|&word| word == 0));
        assert_eq!(words[8] & 0xffff, 0x027f);
        assert_eq!((words[8] >> 16) & 0x3840, 0);
        assert_eq!(words[9] & !0x3f, 0x1f80);
        assert_eq!(&words[10..], &[0, 0]);
    }
    println!("matched two scripted source handoff rows");
}
