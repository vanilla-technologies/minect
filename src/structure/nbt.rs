use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Structure {
    #[serde(rename = "DataVersion")]
    data_version: i32,
    size: Vec<i32>,
    // size: [i32; 3],
    palette: Vec<PaletteBlock>,
    blocks: Vec<StructureBlock>,
    entities: Vec<StructureEntity>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct PaletteBlock {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties", default)]
    properties: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct StructureBlock {
    state: i32,
    // state: usize,
    pos: Vec<i32>,
    // pos: [i32; 3],
    nbt: Option<nbt::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct StructureEntity {
    pos: Vec<f64>,
    // pos: [f64; 3],
    block_pos: Vec<i32>,
    // block_pos: [i32; 3],
    nbt: nbt::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::File,
        io::{self, BufReader, BufWriter},
    };

    #[test]
    fn it_works() -> io::Result<()> {
        let file = File::open("/mnt/c/Users/Adrian/AppData/Roaming/.minecraft/saves/Scribble/generated/minecraft/structures/bla.nbt")?;
        let reader = BufReader::new(file);
        let nbt: Structure = nbt::from_gzip_reader(reader)?;
        println!("{:?}", nbt);

        let structure = Structure {
            data_version: 2724,
            size: vec![1, 2, 3],
            palette: vec![PaletteBlock {
                name: "minecraft:stone".to_string(),
                properties: BTreeMap::new(),
            }],
            blocks: vec![StructureBlock {
                state: 0,
                pos: vec![0, 0, 0],
                nbt: None,
            }],
            entities: Vec::new(),
        };

        let file = File::create("/mnt/c/Users/Adrian/AppData/Roaming/.minecraft/saves/Scribble/generated/minecraft/structures/blub.nbt")?;
        let mut writer = BufWriter::new(file);
        nbt::to_gzip_writer(&mut writer, &structure, None).unwrap();

        assert_eq!(2 + 2, 4);

        Ok(())
    }
}
