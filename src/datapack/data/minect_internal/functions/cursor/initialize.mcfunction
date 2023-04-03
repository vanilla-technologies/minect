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

execute positioned ~1 ~-2 ~ run summon area_effect_cloud ~ ~ ~ {Tags: [minect_cursor]}
scoreboard players add @e[type=area_effect_cloud,tag=minect_cursor] minect_cursor_x 1
setblock ~1 ~-2 ~ chain_command_block[facing=east]{Command: "function minect:enable_logging", auto: true}
setblock ~2 ~-2 ~ chain_command_block[facing=east]{auto: true}
