scoreboard players set reload_timer minect_global 0
reload
execute at @e[type=area_effect_cloud,tag=minect_connection] positioned ~ ~-2 ~ run function minect:pulse_redstone
