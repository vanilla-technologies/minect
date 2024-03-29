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

scoreboard players set version minect_version 3

scoreboard objectives add minect_chunk_pos dummy
scoreboard objectives add minect_cursor_x dummy
scoreboard objectives add minect_cursor_y dummy
scoreboard objectives add minect_cursor_z dummy
scoreboard objectives add minect_even_y dummy
scoreboard objectives add minect_even_z dummy
scoreboard objectives add minect_global dummy
scoreboard objectives add minect_self_pos dummy

scoreboard objectives add minect_const dummy
scoreboard players set 2 minect_const 2
scoreboard players set 16 minect_const 16

scoreboard objectives add minect_config dummy
scoreboard players set update_delay minect_config 1

gamerule commandBlockOutput false
gamerule sendCommandFeedback true
