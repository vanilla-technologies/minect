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

execute store result score @s minect_entity_pos run data get entity @s Pos[0] 1
scoreboard players operation @s minect_chunk_pos = @s minect_entity_pos
scoreboard players operation @s minect_entity_pos %= 16 minect_const
scoreboard players operation @s minect_chunk_pos -= @s minect_entity_pos
execute store result entity @s Pos[0] double 1 run scoreboard players get @s minect_chunk_pos

execute store result score @s minect_entity_pos run data get entity @s Pos[2] 1
scoreboard players operation @s minect_chunk_pos = @s minect_entity_pos
scoreboard players operation @s minect_entity_pos %= 16 minect_const
scoreboard players operation @s minect_chunk_pos -= @s minect_entity_pos
execute store result entity @s Pos[2] double 1 run scoreboard players get @s minect_chunk_pos
