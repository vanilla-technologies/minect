// Minect is library that allows a program to connect to a running Minecraft instance without
// requiring any Minecraft mods.
//
// © Copyright (C) 2021-2023 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
//
// This file is part of Minect.
//
// Minect is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Minect is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Minect.
// If not, see <http://www.gnu.org/licenses/>.

use crate::{
    geometry3::{Coordinate3, Direction3, Orientation3},
    json::{create_json_text_component, escape_json},
    structure::{
        nbt::Structure, new_command_block, new_structure_block, Block, CommandBlockKind,
        StructureBuilder,
    },
    Command, NAMESPACE,
};
use log::warn;
use std::{collections::BTreeMap, iter::FromIterator};

pub(crate) fn generate_structure(
    identifier: &str,
    next_id: u64,
    commands: impl Iterator<Item = Command>,
    commands_len: usize,
) -> Structure {
    let mut builder = StructureBuilder::new();
    for block in generate_basic_structure(identifier, next_id) {
        builder.add_block(block);
    }
    for block in generate_command_blocks(commands, commands_len) {
        builder.add_block(block);
    }
    builder.build()
}

fn generate_basic_structure(connection_id: &str, next_structure_id: u64) -> Vec<Block> {
    Vec::from_iter([
        new_structure_block(
            format!("{}:{}/{}", NAMESPACE, connection_id, next_structure_id),
            "LOAD".to_string(),
            Coordinate3(0, 0, 0),
        ),
        Block {
            name: "minecraft:stone".to_string(),
            pos: Coordinate3(0, 1, 0),
            properties: BTreeMap::new(),
            nbt: None,
        },
        new_structure_block(
            format!("{}:{}/{}", NAMESPACE, connection_id, next_structure_id),
            "CORNER".to_string(),
            Coordinate3(0, 2, 0),
        ),
        // We replace the repeating command block of the previous structure with an impulse command
        // block to reset it's execution ordering. This way it is executed after the regular command
        // blocks (because it has a greater Y coordinate) and can execute the logged block commands
        // that were registered in the same tick. It then replaces itself with a repeating command
        // block that refreshes the connection entity and triggers logged command blocks registered
        // from scheduled functions.
        new_command_block(
            CommandBlockKind::Impulse,
            None,
            format!(
                "setblock ~ ~ ~ repeating_command_block[facing=east]{{Command:\"{}\",auto:true}}",
                escape_json(&summon_connection_entity_command(connection_id))
            ),
            false,
            true,
            Direction3::East,
            Coordinate3(0, 3, 0),
        ),
        Block {
            name: "minecraft:redstone_block".to_string(),
            pos: Coordinate3(0, 4, 0),
            properties: BTreeMap::new(),
            nbt: None,
        },
        Block {
            name: "minecraft:activator_rail".to_string(),
            pos: Coordinate3(0, 5, 0),
            properties: BTreeMap::new(),
            nbt: None,
        },
    ])
}

fn summon_connection_entity_command(connection_id: &str) -> String {
    format!(
        "execute \
        positioned ~ ~2 ~ \
        align xyz \
        unless entity @e[\
            type=area_effect_cloud,\
            dx=1,dy=1,dz=1,\
            tag=minect_connection,tag=minect_connection+{connection_id}\
        ] \
        run \
        summon area_effect_cloud ~.5 ~.5 ~.5 {{\
            Duration:2147483647,\
            CustomName:\"{custom_name}\",\
            Tags:[minect_connection,minect_connection+{connection_id}]\
        }}",
        connection_id = connection_id,
        custom_name = escape_json(&create_json_text_component(connection_id)),
    )
}

const CMD_BLOCK_OFFSET: Coordinate3<i32> = Coordinate3(0, 0, 8);
/// Minecraft limits the number of blocks that can be targeted by a fill command (which we use to
/// clean up) to 32768. X is limited to 16 and Z to 8 to stay in the chunk. The Y limit is
/// therefore calculated as: floor(32768 / 8 / 16) = 256.
/// Additionally there is a height limit in Minecraft before 1.18 of 256. Because we start at Y=1
/// (to avoid a hole in the bedrock layer) our height limit is 255.
/// The size is also hardcoded in the clean_up functions.
const MAX_SIZE: Coordinate3<i32> = Coordinate3(16, 255, 8);
const MAX_LEN: usize = MAX_SIZE.0 as usize * MAX_SIZE.1 as usize * MAX_SIZE.2 as usize;

fn generate_command_blocks(
    commands: impl Iterator<Item = Command>,
    commands_len: usize,
) -> impl Iterator<Item = Block> {
    if commands_len > MAX_LEN {
        warn!(
            "Attempted to execute {} commands. \
             Only the first {} commands will be executed. \
             The rest will be ignored.",
            commands_len, MAX_LEN
        );
    }

    const CURVE_ORIENTATION: Orientation3 = Orientation3::XZY;
    let max_size = CURVE_ORIENTATION.inverse().orient_coordinate(MAX_SIZE);
    let curve = CuboidCurve::new(max_size).map(|(coordinate, direction)| {
        (
            CURVE_ORIENTATION.orient_coordinate(coordinate),
            CURVE_ORIENTATION.orient_direction(direction),
        )
    });

    commands
        .zip(curve)
        .map(|(command, (coordinate, direction))| CommandBlock {
            command,
            coordinate,
            direction,
        })
        .map(|cmd_block| {
            let first = cmd_block.coordinate == Coordinate3(0, 0, 0);
            let kind = if first {
                CommandBlockKind::Impulse
            } else {
                CommandBlockKind::Chain
            };
            new_command_block(
                kind,
                cmd_block.command.get_name_as_json(),
                cmd_block.command.command,
                false,
                true,
                cmd_block.direction,
                cmd_block.coordinate + CMD_BLOCK_OFFSET,
            )
        })
}

pub(crate) struct CommandBlock {
    pub(crate) command: Command,
    pub(crate) coordinate: Coordinate3<i32>,
    pub(crate) direction: Direction3,
}

/// An [Iterator] producing a space filling curve with a zig zag pattern in the form of a cuboid.
/// Whenever possible the X axis is incremented or decremented, then the Y axis and if that is not
/// possible the Z axis.
pub(crate) struct CuboidCurve {
    next: Option<Coordinate3<i32>>,
    size: Coordinate3<i32>,
}

impl CuboidCurve {
    pub(crate) fn new(size: Coordinate3<i32>) -> CuboidCurve {
        CuboidCurve {
            next: Some(Coordinate3(0, 0, 0)),
            size,
        }
    }
}

impl Iterator for CuboidCurve {
    type Item = (Coordinate3<i32>, Direction3);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        let direction = direction_in_cuboid_curve(current, self.size);
        self.next = direction.map(|direction| current + direction.as_coordinate(1, 0));
        let direction = direction?; // Leave the last corner for the clean up command block
        Some((current, direction))
    }
}

fn direction_in_cuboid_curve(
    current: Coordinate3<i32>,
    size: Coordinate3<i32>,
) -> Option<Direction3> {
    // Try advance x
    {
        let forward0 = (current.1 % 2 == 0) == (current.2 % 2 == 0);
        let max0 = size.0 - 1;
        let limit0 = if forward0 { max0 } else { 0 };
        if current.0 != limit0 {
            return Some(if forward0 {
                Direction3::East
            } else {
                Direction3::West
            });
        }
    }
    // Try advance y
    {
        let forward1 = current.2 % 2 == 0;
        let max1 = size.1 - 1;
        let limit1 = if forward1 { max1 } else { 0 };
        if current.1 != limit1 {
            return Some(if forward1 {
                Direction3::Up
            } else {
                Direction3::Down
            });
        }
    }
    // Try advance z
    {
        let max2 = size.2 - 1;
        let limit2 = max2;
        if current.2 != limit2 {
            return Some(Direction3::South);
        }
    }
    None
}
