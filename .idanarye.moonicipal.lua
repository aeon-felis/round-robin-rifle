local moonicipal = require'moonicipal'
local T = moonicipal.tasks_file()

-- require'idan'.unload_package('idan.project')
T = require'idan.project.rust.bevy'(T, {
    crate_name = 'round_robin_rifle',
})
