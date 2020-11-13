SubBlock("core0", "core")
SubBlock("core1", "core", offset=0x1000_0000)
SubBlock("core2", "core", offset=0x2000_0000)
SubBlock("core3", "core", offset=0x3000_0000)
SubBlock("dac", "python_plugin.dac", offset=0x8000_0000)
SubBlock("arm_debug",
         "origen.arm_debug",
         sb_options={
             "mem_aps": {
                 "sys": {},
                 "core1": {
                     "ap": 1
                 },
                 "core2": {
                     "ap": 2
                 }
             }
         })
SubBlock("flash",
         "origen.memories",
         "Flash",
         offset=0x0,
         sb_options={"length": 0x1_0000})
SubBlock("shared_ram",
         "origen.memories",
         "RAM",
         offset=0x1_0000,
         sb_options={"length": 0x1_0000})
