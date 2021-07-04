pub mod nbt;
pub mod placement;

pub struct StructureBuilder {}

impl StructureBuilder {
    pub fn new() -> StructureBuilder {
        StructureBuilder {}
    }
    // pub fn add_block(&self, block: B) {}
    // pub fn build(self) -> nbt::Structure {
    //     Structure {
    //         data_version: 2724,
    //         size,
    //         palette,
    //         blocks,
    //         entities: Vec::new(),
    //     }
    // }
}

trait Block {}
