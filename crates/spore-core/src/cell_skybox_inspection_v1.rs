//! Select the Cell skybox screen from an explicitly supplied EFFDIR.
//!
//! This is an inspection join, not runtime directory or texture selection.
//! The caller must separately establish the identity of the directory bytes.

use crate::cell_effdir_framing_v3::{Directory, Key, ScreenRef, Target};

pub const CELL_SKYBOX: Key = Key {
    group: 0,
    instance: 0x05c7_a4d0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenSelection {
    pub component_index: u32,
    pub mode: u8,
    pub flags: u32,
    pub texture: Key,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    ExportCount(usize),
    ExportNotLocalVisual,
    ScreenCount(usize),
    ScreenNotLocal,
    UnsupportedScreen { mode: u8, flags: u32 },
}

/// Select exactly one local screen from exactly one local Cell skybox export.
/// The mode and flags are restricted to the inspected candidate path; all
/// other components and the directory's opaque sections remain uninterpreted.
pub fn select(directory: &Directory<'_>) -> Result<ScreenSelection, Error> {
    let targets = directory.resolve(CELL_SKYBOX);
    if targets.len() != 1 {
        return Err(Error::ExportCount(targets.len()));
    }
    let Target::Visual(visual) = targets[0] else {
        return Err(Error::ExportNotLocalVisual);
    };
    let screens: Vec<_> = visual.blocks.iter().filter(|b| b.type_id == 9).collect();
    if screens.len() != 1 {
        return Err(Error::ScreenCount(screens.len()));
    }
    let block = screens[0];
    let Some(ScreenRef::Screen(screen)) = directory.screen_ref(block) else {
        return Err(Error::ScreenNotLocal);
    };
    if screen.mode != 5 || screen.flags != 1 {
        return Err(Error::UnsupportedScreen {
            mode: screen.mode,
            flags: screen.flags,
        });
    }
    Ok(ScreenSelection {
        component_index: block.index,
        mode: screen.mode,
        flags: screen.flags,
        texture: screen.texture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_effdir_framing_v3::{Block, Export, Screen, Screens, Visual};

    fn fixture() -> Directory<'static> {
        Directory {
            sections: vec![],
            screens: Screens::V1(vec![Screen {
                mode: 5,
                flags: 1,
                texture: Key {
                    group: 0,
                    instance: 0xbd08_bd8a,
                },
            }]),
            visuals: vec![Visual {
                flags: 0,
                blocks: vec![Block {
                    type_id: 9,
                    index: 0,
                }],
            }],
            exports: vec![Export {
                index: 0,
                key: CELL_SKYBOX,
            }],
            assignments_word: u32::MAX,
            imports: vec![],
        }
    }

    #[test]
    fn joins_the_selected_export_to_its_screen_texture() {
        let selected = select(&fixture()).unwrap();
        assert_eq!(
            selected.texture,
            Key {
                group: 0,
                instance: 0xbd08_bd8a
            }
        );
        assert_eq!(selected.component_index, 0);
    }

    #[test]
    fn rejects_ambiguous_or_nonlocal_exports_and_screens() {
        let mut d = fixture();
        d.exports.push(Export {
            index: 0,
            key: CELL_SKYBOX,
        });
        assert_eq!(select(&d), Err(Error::ExportCount(2)));
        d.exports.pop();
        d.exports[0].index = 0x7f00_0000;
        assert_eq!(select(&d), Err(Error::ExportNotLocalVisual));
        d.exports[0].index = 0;
        d.visuals[0].blocks.push(Block {
            type_id: 9,
            index: 0,
        });
        assert_eq!(select(&d), Err(Error::ScreenCount(2)));
        d.visuals[0].blocks.pop();
        d.visuals[0].blocks[0].index = 0x7f00_0000;
        assert_eq!(select(&d), Err(Error::ScreenNotLocal));
    }

    #[test]
    fn rejects_other_screen_modes_without_assuming_their_rendering() {
        let mut d = fixture();
        let Screens::V1(ref mut screens) = d.screens else {
            unreachable!()
        };
        screens[0].mode = 3;
        assert_eq!(
            select(&d),
            Err(Error::UnsupportedScreen { mode: 3, flags: 1 })
        );
    }
}
