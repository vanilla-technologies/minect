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

scoreboard objectives add minect_version dummy
execute if score version minect_version matches 1 run function minect_internal:v1_uninstall
execute if score version minect_version matches 2 run function minect_internal:v2_uninstall
execute unless score version minect_version matches 3.. run function minect_internal:v3_install

# TODO: Instead of using function tags we could patch this function. That way there is a bit less clutter that is alphabetically before the functions in the minect namespace.
scoreboard players set connect_prompt minect_global 1
function #minect_internal:connect/prompt
