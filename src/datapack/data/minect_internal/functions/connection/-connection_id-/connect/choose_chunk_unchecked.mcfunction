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

execute as @e[type=area_effect_cloud,tag=minect_connector+-connection_id-] run function minect_internal:connect/remove_connector
summon area_effect_cloud ~ ~ ~ {Duration: 2147483647, Tags: [minect_connector+-connection_id-]}
execute as @e[type=area_effect_cloud,tag=minect_connector+-connection_id-] run function minect_internal:connect/align_to_chunk

execute at @e[type=area_effect_cloud,tag=minect_connector+-connection_id-] run setblock ~ ~ ~ structure_block{name: "minect:-connection_id-/-structure_id-", mode: LOAD, showboundingbox: true, sizeX: 16, sizeY: 136, sizeZ: 16}

tellraw @s [{"text":""},{"text":"[Info]","color":"blue","hoverEvent":{"action":"show_text","contents":"Minect"}},{"text":" This chunk will be force loaded to keep the connection active when no player is around.\n "},{"text":"[Confirm]","clickEvent":{"action":"run_command","value":"/execute as @e[type=area_effect_cloud,tag=minect_connector+-connection_id-] run function minect_internal:connection/-connection_id-/connect/confirm_chunk"},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"green"},{"text":" "},{"text":"[Choose different chunk]","clickEvent":{"action":"suggest_command","value":"/execute positioned ~ ~ ~ run function minect:connect/choose_chunk"},"hoverEvent":{"action":"show_text","contents":"Click for suggestion"},"color":"yellow"},{"text":" "},{"text":"[Cancel]","clickEvent":{"action":"run_command","value":"/function minect_internal:connection/-connection_id-/connect/cancel"},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"red"}]

# Only one choose_chunk at a time
scoreboard players reset connect_choose_chunk minect_global
