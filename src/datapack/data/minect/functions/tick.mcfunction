kill @e[type=command_block_minecart,tag=minect_impulse,nbt=!{LastExecution: 1L}]

scoreboard players add reload_timer minect_global 1
execute if score reload_timer minect_global >= reload_delay minect_config run function minect:reload
