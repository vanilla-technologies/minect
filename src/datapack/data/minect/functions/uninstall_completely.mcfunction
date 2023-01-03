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

execute as @e[type=area_effect_cloud,tag=minect_connection] run function minect:disconnect_self

scoreboard objectives remove minect_chunk_pos
scoreboard objectives remove minect_config
scoreboard objectives remove minect_const
scoreboard objectives remove minect_entity_pos
scoreboard objectives remove minect_global

scoreboard objectives remove minect_version
datapack disable "file/minect"

tellraw @s [{"text":""},{"text":"[Info]","color":"blue","hoverEvent":{"action":"show_text","contents":"Minect"}},{"text":" Uninstalled Minect from Minecraft. To fully uninstall Minect, you need to delete the following directories in your world directory and then execute "},{"text":"reload","clickEvent":{"action":"run_command","value":"/reload"},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"aqua"},{"text":" (or restart Minecraft):\n - datapacks/minect\n - generated/minect\n Alternatively you can reinstall Minect by executing "},{"text":"datapack enable \"file/minect\"","clickEvent":{"action":"run_command","value":"/datapack enable \"file/minect\""},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"aqua"}]
