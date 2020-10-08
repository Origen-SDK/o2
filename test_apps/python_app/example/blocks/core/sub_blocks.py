SubBlock("adc0", "adc.16_bit")
SubBlock("adc1", "adc.8_bit", offset=0x1000)
SubBlock("ram", "origen.memories", class_name="RAM", offset=0x1_0000, sb_options={"length": 0x1000})