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

forceload add ~ ~
setblock ~ ~ ~ air
setblock ~ 1 ~ structure_block{name: "minect:-connection_id-/-structure_id-", mode: LOAD}
setblock ~ 2 ~ redstone_block

# Protect the activator rail
setblock ~1 6 ~ stone
setblock ~-1 6 ~ stone
setblock ~ 6 ~1 stone
setblock ~ 6 ~-1 stone
setblock ~ 7 ~ stone

kill @s
tellraw @a [{"text":""},{"text":"[Info]","color":"blue","hoverEvent":{"action":"show_text","contents":"Minect"}},{"text":" Added connection -connection_id-"}]

# This loads the removal of the connect functions on disk
schedule function minect_internal:reload 1t
