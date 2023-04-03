gamerule commandBlockOutput false
gamerule doDaylightCycle false
gamerule doEntityDrops false
gamerule doMobLoot false
gamerule doMobSpawning false
gamerule doTileDrops false
gamerule doWeatherCycle false
time set day

forceload add 0 0

setblock 0 4 0 chain_command_block[facing=up]{Command: "setblock ~ ~-3 ~ stone", auto: true}
setblock 0 3 0 repeating_command_block[facing=up]{Command: "setblock ~ ~-2 ~ redstone_block", auto: true}
setblock 0 2 0 structure_block{name: "minect:test/0", mode: CORNER}
setblock 0 0 0 structure_block{name: "minect:test/0", mode: LOAD}

datapack disable "file/setup-minecraft-for-ci"
