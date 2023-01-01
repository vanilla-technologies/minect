# Minect is library that allows a program to connect to a running Minecraft instance without
# requiring any Minecraft mods.
#
# © Copyright (C) 2021, 2022 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
#
# This file is part of Minect.
#
# Minect is free software: you can redistribute it and/or modify it under the terms of the GNU
# General Public License as published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
#
# Minect is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
# the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
# Public License for more details.
#
# You should have received a copy of the GNU General Public License along with Minect.
# If not, see <http://www.gnu.org/licenses/>.

execute as @e[type=area_effect_cloud,tag=minect_connector+-connection_id-] run function minect_internal:connect/remove_connector
setblock ~ ~ ~ redstone_block
setblock ~ ~1 ~ activator_rail
execute align xyz run summon command_block_minecart ~.5 ~1 ~.5 {Command: "function minect:enable_logging", Tags: [minect_connect_canceller], TrackOutput: false}
execute align xyz run summon command_block_minecart ~.5 ~1 ~.5 {CustomName: '{"text":"minect_connect"}', Command: "tag @s add minect_connect_cancelled", Tags: [minect_connect_canceller], TrackOutput: false}
execute align xyz run summon command_block_minecart ~.5 ~1 ~.5 {Command: "function minect:reset_logging", Tags: [minect_connect_canceller], TrackOutput: false}
execute align xyz run summon command_block_minecart ~.5 ~1 ~.5 {Command: "function minect_internal:connection/-connection_id-/connect/cancel_cleanup", Tags: [minect_connect_canceller], TrackOutput: false}
