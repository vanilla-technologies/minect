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

scoreboard players operation @s minect_even_y = @s minect_cursor_y
scoreboard players operation @s minect_even_y %= 2 minect_const

scoreboard players operation @s minect_even_z = @s minect_cursor_z
scoreboard players operation @s minect_even_z %= 2 minect_const

execute if score @s minect_even_z = @s minect_even_y run function minect_internal:cursor/try_place_facing_east
execute unless score @s minect_even_z = @s minect_even_y run function minect_internal:cursor/try_place_facing_west
