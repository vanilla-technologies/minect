scoreboard objectives add minect_version dummy
execute unless score version minect_version matches 1.. run function minect:install
