use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Structure {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
    pub size: Vec<i32>,
    // size: [i32; 3],
    pub palette: Vec<PaletteBlock>,
    pub blocks: Vec<StructureBlock>,
    pub entities: Vec<StructureEntity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PaletteBlock {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties", default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StructureBlock {
    pub state: i32,
    // state: usize,
    pub pos: Vec<i32>,
    // pos: [i32; 3],
    pub nbt: Option<nbt::Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StructureEntity {
    pub pos: Vec<f64>,
    // pos: [f64; 3],
    pub block_pos: Vec<i32>,
    // block_pos: [i32; 3],
    pub nbt: nbt::Value,
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
