# Minect is library that allows a program to connect to a running Minecraft instance without
# requiring any Minecraft mods.
#
# © Copyright (C) 2021-2023 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
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

execute if block ~ ~ ~ chain_command_block[facing=east] run scoreboard players add @s minect_cursor_x 1
execute if block ~ ~ ~ chain_command_block[facing=east] positioned ~1 ~ ~ run function minect_internal:cursor/place
execute if block ~ ~ ~ chain_command_block[facing=west] run scoreboard players remove @s minect_cursor_x 1
execute if block ~ ~ ~ chain_command_block[facing=west] positioned ~-1 ~ ~ run function minect_internal:cursor/place
execute if block ~ ~ ~ chain_command_block[facing=up] run scoreboard players add @s minect_cursor_y 1
execute if block ~ ~ ~ chain_command_block[facing=up] positioned ~ ~1 ~ run function minect_internal:cursor/place
execute if block ~ ~ ~ chain_command_block[facing=down] run scoreboard players remove @s minect_cursor_y 1
execute if block ~ ~ ~ chain_command_block[facing=down] positioned ~ ~-1 ~ run function minect_internal:cursor/place
execute if block ~ ~ ~ chain_command_block[facing=south] run scoreboard players add @s minect_cursor_z 1
execute if block ~ ~ ~ chain_command_block[facing=south] positioned ~ ~ ~1 run function minect_internal:cursor/place
execute if block ~ ~ ~ chain_command_block[facing=north] run scoreboard players remove @s minect_cursor_z 1
execute if block ~ ~ ~ chain_command_block[facing=north] positioned ~ ~ ~-1 run function minect_internal:cursor/place
