SubBlock("adc0", block_path="adc.16_bit")
SubBlock("adc1", block_path="adc.8_bit", offset=0x1000)
SubBlock("ram", mod_path=origen.sbb.memories, class_name="RAM", offset=0x1_0000, sb_options={"length": 0x1000})
