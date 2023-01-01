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

tellraw @a [{"text":""},{"text":"[Info]","color":"blue","hoverEvent":{"action":"show_text","contents":"Minect"}},{"text":" An external application wants to establish a connection with identifier '-connection_id-'. You can click on the colored text below to choose a chunk in which to generate the connection structure. The chunk may be cleared by the connection, so make sure it does not contain anything important.\n "},{"text":"[Choose a chunk]","clickEvent":{"action":"suggest_command","value":"/execute positioned ~ ~ ~ run function minect:connect/choose_chunk"},"hoverEvent":{"action":"show_text","contents":"Click for suggestions"},"color":"aqua"},{"text":" "},{"text":"[Cancel]","clickEvent":{"action":"run_command","value":"/function minect_internal:connection/-connection_id-/connect/cancel"},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"aqua"}]

# Only one prompt at a time
scoreboard players reset connect_prompt minect_global
