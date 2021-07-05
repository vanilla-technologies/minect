pub mod geometry3;
pub mod structure;

use structure::StructureBuilder;

pub struct InjectionConnection {}

impl InjectionConnection {
    pub fn new() -> InjectionConnection {
        InjectionConnection {}
    }

    // pub fn inject_group(group: Vec<Command>) {
    //     let builder = StructureBuilder::new();
    //     for command in group {
    //         builder.add_block()
    //     }
    //     let structure = builder.build();
    // }
}

pub struct Command {
    command: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
