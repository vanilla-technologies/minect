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

function minect:reset_logging
tag @e[type=area_effect_cloud,tag=minect_connection,tag=minect_inactive] remove minect_inactive

execute align xyz positioned ~ ~5 ~ positioned ~-15 ~-254 ~-15 at @e[type=area_effect_cloud,tag=minect_connection,dx=16,dy=255,dz=16] positioned ~ ~-5 ~ run fill ~ ~ ~8 ~15 ~254 ~15 stone replace #minect_internal:command_blocks
