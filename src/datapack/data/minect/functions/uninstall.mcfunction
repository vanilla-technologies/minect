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

tellraw @s [{"text":""},{"text":"[Info]","color":"blue","hoverEvent":{"action":"show_text","contents":"Minect"}},{"text":" Do you want to uninstall Minect or remove individual connections?\n "},{"text":"[Uninstall Minect]","clickEvent":{"action":"run_command","value":"/function minect:uninstall_completely"},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"aqua"},{"text":" "},{"text":"[Remove individual connections]","clickEvent":{"action":"run_command","value":"/function minect:disconnect"},"hoverEvent":{"action":"show_text","contents":"Click to execute"},"color":"aqua"}]
