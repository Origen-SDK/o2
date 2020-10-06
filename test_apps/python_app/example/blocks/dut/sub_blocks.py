SubBlock("core0", "core")
SubBlock("core1", "core", offset=0x1000_0000)
SubBlock("core2", "core", offset=0x2000_0000)
SubBlock("core3", "core", offset=0x3000_0000)

SubBlock("dac", "python_plugin.dac", offset=0x8000_0000)
