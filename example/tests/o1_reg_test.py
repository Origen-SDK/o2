# These are the tests from o1's reg_spec.rb, converted to o2

import origen
import pdb
import pytest

#def read_register(reg, options={})
#  # Dummy method to allow the bang methods to be tested
#end
#
#def write_register(reg, options={})
#  # Dummy method to allow the bang methods to be tested
#end

@pytest.fixture(autouse=True)
def run_around_tests():
    global dut
    # Code that will run before each test
    origen.app.instantiate_dut("dut.falcon")
    dut = origen.dut
    yield
    # Code that will run after each test

def test_split_bits():
    with dut.add_reg("tcu", 0x0024, size=8) as reg:
        reg.Field("peter", offset=7, reset=0)
        reg.Field("mike",  offset=6, reset=0)
        reg.Field("mike",  offset=5, reset=0)
        reg.Field("mike",  offset=4, reset=0)
        reg.Field("peter", offset=3, reset=1)
        reg.Field("peter", offset=2, reset=1)
        reg.Field("pan",   offset=1, reset=0)
        reg.Field("peter", offset=0, reset=0)

    assert dut.reg("tcu").get_data() == 12
    assert dut.reg("tcu").field("peter").len() == 4
    assert dut.reg("tcu").field("peter").data() == 0b0110
    dut.reg("tcu").field("peter").set_data(0)
    assert dut.reg("tcu").data() == 0
    assert dut.reg("tcu").field("peter").data() == 0
    dut.reg("tcu").field("peter").set_data(7)
    assert dut.reg("tcu").data() == 0b1101
    assert dut.reg("tcu").get_data() == 0b1101
    assert dut.reg("tcu").field("peter").data() == 7
    dut.reg("tcu").reset()
    assert dut.reg("tcu").data() == 12

def test_more_split_bits():
    with dut.add_reg("tcu", 0x0024, size=8) as reg:
        reg.Field("peter", offset=7, reset=0)
        reg.Field("mike",  offset=4, width=3, reset=0)
        reg.Field("peter", offset=2, width=2, reset=3)
        reg.Field("pan",   offset=1, reset=0)
        reg.Field("peter", offset=0, reset=0)
        
    assert dut.tcu.data() == 12
    assert dut.tcu.peter.len() == 4
    assert dut.tcu.peter.data() == 0b0110
    dut.tcu.peter.set_data(0)
    assert dut.tcu.data() == 0
    assert dut.tcu.peter.data() == 0
    dut.tcu.peter.set_data(7)
    assert dut.tcu.data() == 0b1101
    assert dut.tcu.peter.data() == 7
    assert dut.tcu.reset()
    assert dut.tcu.data() == 12

def test_yet_more_split_bits():
    with dut.add_reg("tcu", 0x0070, size=16) as reg:
        reg.Field("mike",   offset=15, reset = 1)
        reg.Field("bill",   offset=14, reset = 0)
        reg.Field("robert", offset=13, reset = 1)
        reg.Field("james",  offset=12, reset = 0)
        reg.Field("james",  offset=11, reset = 1)
        reg.Field("james",  offset=10, reset = 0)
        reg.Field("paul",   offset=9,  reset = 1)
        reg.Field("peter",  offset=8,  reset = 0)
        reg.Field("mike",   offset=7,  reset = 1)
        reg.Field("mike",   offset=6,  reset = 0)
        reg.Field("paul",   offset=5,  reset = 1)
        reg.Field("paul",   offset=4,  reset = 0)
        reg.Field("mike",   offset=3,  reset = 1)
        reg.Field("robert", offset=2,  reset = 0)
        reg.Field("bill",   offset=1,  reset = 0)
        reg.Field("ian",    offset=0,  reset = 1)
        
    assert dut.tcu.data() == 43689
    #check sizes
    assert dut.tcu.bill.len() == 2
    assert dut.tcu.ian.len() == 1
    assert dut.tcu.james.len() == 3
    assert dut.tcu.mike.len() == 4
    assert dut.tcu.paul.len() == 3
    assert dut.tcu.peter.len() == 1
    assert dut.tcu.robert.len() == 2
    #check reset data
    assert dut.tcu.bill.data() == 0
    assert dut.tcu.ian.data() == 1
    assert dut.tcu.james.data() == 2
    assert dut.tcu.mike.data() == 13
    assert dut.tcu.paul.data() == 6
    assert dut.tcu.peter.data() == 0
    assert dut.tcu.robert.data() == 2
    #write register to all 1
        
    dut.tcu.set_data(0xFFFF)
    assert dut.tcu.data() == 65535
        
    #write :peter to 0 and james[1] to 0
    dut.tcu.peter.set_data(0b0)
    dut.tcu.james[1].set_data(0b0)
    assert dut.tcu.peter.data() == 0
    assert dut.tcu.james.data() == (0b101)
    assert dut.tcu.data() == 63231
  
    #write mike to 1010 and james[2] to 1
    dut.tcu.mike.set_data(0b1010)
    dut.tcu.james[2].set_data(0)
    assert dut.tcu.mike.data() == 10
    assert dut.tcu.james.data() == 0b001
    assert dut.tcu.data() == 58999
    assert dut.tcu.reset()
    assert dut.tcu.data() == 43689

def test_has_an_address():
    dut.add_simple_reg("tr1", 0x10)
    assert dut.tr1.address() == 0x10

def test_has_a_reset_data_value():
    dut.add_simple_reg("tr1", 0x10, reset=0)
    assert dut.tr1.data() == 0
    with dut.add_reg("tr2", 0x0) as reg:
        reg.Field("b0", offset=0, reset = 1)
        reg.Field("b1", offset=1, reset = 1)
    assert dut.tr2.data() == 3
    with dut.add_reg("tr3", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)
        reg.Field("b2", offset=16, reset = 1)
    assert dut.tr3.data() == 0x1AA55

def test_stores_reset_data_at_bit_level():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)
        reg.Field("b2", offset=16, reset = 1)

    assert dut.tr1.b0.data() == 0x55
    assert dut.tr1.b1.data() == 0xAA
    assert dut.tr1.b2.data() == 1

def test_fields_can_be_accessed_via_field_or_fields():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)
        reg.Field("b2", offset=16, reset = 1)

    assert dut.tr1.fields["b0"].data() == 0x55
    assert dut.tr1.fields["b1"].data() == 0xAA
    assert dut.tr1.fields["b2"].data() == 1
    assert dut.tr1.field("b0").data() == 0x55
    assert dut.tr1.field("b1").data() == 0xAA
    assert dut.tr1.field("b2").data() == 1

def test_bits_can_be_accessed_via_position_number():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)
        reg.Field("b2", offset=16, reset = 0)

    assert dut.tr1[0].data() == 1
    assert dut.tr1[1].data() == 0
    assert dut.tr1[2].data() == 1

def test_bits_can_be_written_directly():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)
        reg.Field("b2", offset=16, reset = 1)

    assert dut.tr1.b1.data() == 0xAA
    assert dut.tr1.b1.set_data(0x13)
    assert dut.tr1.b1.data() == 0x13
    assert dut.tr1.b2.data() == 1

def test_bits_can_be_written_indirectly():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)
        reg.Field("b2", offset=16, reset = 1)

    dut.tr1.set_data(0x1234)
    assert dut.tr1.b0.data() == 0x34
    assert dut.tr1.b1.data() == 0x12
    assert dut.tr1.b2.data() == 0

#    # Add read/write coverage for all ACCESS_CODES
#    specify "access_codes properly control read and writability of individual bits" do
#      load_target
#      dut.nvm.reg(:access_types).data.should == 0x0000_0000
#      dut.nvm.reg(:access_types).write(0xFFFF_FFFF)
#      # Bits 31,29,28,4 not writable, bit 25,22,21,14,10 clear on write or write of 1'b1
#      # CORRECT: dut.nvm.reg(:access_types).data.should == 0b0100_1101_1001_1111_1011_1011_1110_0000
#      # NOTE: bits 22, 21, and 14 are broken - :wcrs, :w1c, and :w1crs do not clear on write!
#      # TEMP Expectation until above bug is fixed:
#      dut.nvm.reg(:access_types).data.should == 0b0100_1101_1111_1111_1111_1011_1110_0000
#      dut.nvm.reg(:access_types).read!
#      # Bits 29,27,23,15,13,4 clear on a read
#      # CORRECT: dut.nvm.reg(:access_types).data.should == 0b0100_0101_0111_1111_0111_1011_1110_0000
#      # NOTE: Bits 27, 23, and 15 are broken - :wrc, :wsrc, and :w1src do not clear on read!
#      # TEMP Expectation until above bug is fixed:
#      dut.nvm.reg(:access_types).data.should == 0b0100_1101_1111_1111_1111_1011_1110_0000
#    end

def test_only_defined_bits_capture_state():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=4, reset=0x55)
        reg.Field("b1", offset=8, width=4, reset=0xAA)

    dut.tr1.set_data(0xFFFF) 
    assert dut.tr1.data() == 0x0F0F

def test_bits_can_be_reset_indirectly():
    with dut.add_reg("tr1", 0x0) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=8, reset=0xAA)

    dut.tr1.set_data(0xFFFF) 
    assert dut.tr1.data() == 0xFFFF
    dut.tr1.reset()
    assert dut.tr1.data() == 0xAA55

def test_len_method():
    dut.add_simple_reg("tr1", 0x10, size=16)
    dut.add_simple_reg("tr2", 0x10, size=36)

    assert dut.tr1.len() == 16
    assert dut.tr2.len() == 36

def test_it_can_shift_out_left():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=0, width=4, reset=0x5)
        reg.Field("b1", offset=4, width=4, reset=0xA)
    
    reg = dut.tr1
    expected = [1,0,1,0,0,1,0,1]
    x = 0
    for bit in reg.shift_out_left():
      assert bit.data() == expected[x]
      x += 1
    reg.set_data(0xF0)
    expected = [1,1,1,1,0,0,0,0]
    x = 0
    for bit in reg.shift_out_left():
      assert bit.data() == expected[x]
      x += 1

def test_it_can_shift_out_right():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=0, width=4, reset=0x5)
        reg.Field("b1", offset=4, width=4, reset=0xA)
        
    reg = dut.tr1
    expected = [1,0,1,0,0,1,0,1]
    x = 0
    for bit in reg.shift_out_right():
        assert bit.data() == expected[7-x]
        x += 1
    reg.set_data(0xF0)
    expected = [1,1,1,1,0,0,0,0]
    x = 0
    for bit in reg.shift_out_right():
        assert bit.data() == expected[7-x]
        x += 1

def test_it_can_shift_out_with_holes_present():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=1, width=2, reset=0b11)
        reg.Field("b1", offset=6, width=1, reset=0b1)
    reg = dut.tr1

    expected = [0,1,0,0,0,1,1,0]
    x = 0
    for bit in reg.shift_out_left():
        assert bit.data() == expected[x] 
        x += 1
    expected = [0,1,1,0,0,0,1,0]
    x = 0
    for bit in reg.shift_out_right():
        assert bit.data() == expected[x] 
        x += 1

def test_read_method_tags_all_bits_for_read():
    dut.add_simple_reg("tr1", 0x10, size=16, reset=0)
    reg = dut.tr1
    reg.read()
    for i in range(16):
        assert reg[i].is_to_be_read() == True

# This test added due to problems shifting out buses (in o1)
def test_it_can_shift_out_left_with_holes_and_buses():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=5)
        reg.Field("b1", offset=0, width=4)

    reg = dut.tr1
    reg.set_data(0xFF)
    assert reg.data() == 0b00101111
    expected = [0,0,1,0,1,1,1,1]
    x = 0
    for bit in reg.shift_out_left():
        assert bit.data() == expected[x]
        x += 1

def test_bits_mark_as_update_required_correctly():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=5, reset=1)
        reg.Field("b1", offset=0, width=4, reset=3)
    reg = dut.tr1

    assert reg.is_update_required() == False
    reg.set_data(0x23)
    assert reg.is_update_required() == False
    reg.set_data(0x0F)
    assert reg.is_update_required() == True
    reg.set_data(0x23)
    assert reg.is_update_required() == False

def test_update_device_state_clears_update_required():
    with dut.add_reg("tr1", 0x0, size=8) as reg:
        reg.Field("b0", offset=5, reset=1)
        reg.Field("b1", offset=0, width=4, reset=3)
    reg = dut.tr1
    reg.set_data(0x0F)
    assert reg.is_update_required() == True
    reg.update_device_state()
    assert reg.is_update_required() == False

def test_can_iterate_through_fields():
    with dut.add_reg("tr1", 0x0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=4, reset=0xA)
        reg.Field("b2", offset=14, width=2, reset=1)
    reg = dut.tr1

    for name, field in reg.fields.items():
        if name == "b0":
            field.set_data(0x1)
        elif name == "b1":
            field.set_data(0x2)
        elif name == "b2":
            field.set_data(0x3)
            
    assert reg.data() == 0xC201
    assert reg.field("b1").data() == 0x2

def test_can_use_fields_with_bit_ordering():
    with dut.add_reg("tr1", 0x0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0x55)
        reg.Field("b1", offset=8, width=4, reset=0xA)
        reg.Field("b2", offset=14, width=2, reset=1)
    reg1 = dut.tr1
    with dut.add_reg("tr2", 0x0, bit_order="msb0", size=8) as reg:
        reg.Field("msb0", offset=6, width=2)
        reg.Field("msb1", offset=4, width=2, reset=0x3)
        reg.Field("msb2", offset=0, reset=1)
    reg2 = dut.tr2

    assert list(reg1.fields.keys())[0] == "b0"
    assert list(reg2.fields.keys())[0] == "msb0"

# TODO: Should be supported, but via the more generic access= attribute rather than w1c=
#    specify "can set bitw1c attribute and query w1c status" do
#        reg = Reg.new(self, 0x10, 16, :dummy, b0: {pos: 0, bits: 8, res: 0x55}, 
#                                              b1: {pos: 8, bits: 4, res: 0xA},
#                                              b2: {pos: 12, bits: 1, w1c: true},
#                                              b3: {pos: 13, bits: 1, w1c: false},
#                                              b4: {pos: 14,bits: 1, res: 1})
#
#        reg.bit(:b2).w1c.should == true
#        reg.bit(:b3).w1c.should == false
#        reg.bit(:b4).w1c.should == false
#    end

# TODO: What's the Python equivalent of this?
#    it "should respond to bit collection methods" do
#        reg = Reg.new(self, 0x10, 16, :dummy, b0: {pos: 0, bits: 8, res: 0x55}, 
#                                              b1: {pos: 8, bits: 4, res: 0xA},
#                                              b2: {pos: 14,bits: 2, res: 1})
#        reg.respond_to?("data").should == true
#        reg.respond_to?("read").should == true
#        reg.respond_to?("some_nonsens").should_not == true
#    end

def test_should_respond_to_data_as_an_alias_of_get_data():
    with dut.add_reg("r1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8)
        reg.Field("b1", offset=8, width=8)

    dut.r1.set_data(0x1234)
    assert dut.r1.get_data() == 0x1234
    assert dut.r1.data() == 0x1234
    
# TODO: Probably required, but putting as lower prior
#    specify "bits can be deleted" do
#        reg = Reg.new(self, 0x10, 16, :dummy, b0: {pos: 0, bits: 8}, 
#                                              b1: {pos: 8, bits: 8})
#        reg.has_bit?(:b1).should == true
#        reg.bits(:b1).delete
#        reg.has_bit?(:b1).should == false
#        reg.write(0xFFFF)
#        reg.data.should == 0x00FF
#    end

def test_reg_state_can_be_copied():
    with dut.add_reg("tr1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8)
        reg.Field("b1", offset=8, width=8)
    with dut.add_reg("tr2", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8)
        reg.Field("b1", offset=8, width=8)
    reg1 = dut.tr1
    reg2 = dut.tr2
    reg1.set_overlay("hello")
    reg1.set_data(0x1234)
    reg2.copy(reg1)
    assert reg2.overlay() == "hello"
    assert reg2.data() == 0x1234

def test_bit_collections_can_be_copied_to_other_bitcollections():
    with dut.add_reg("tr1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0)
        reg.Field("b1", offset=8, width=8, reset=0)
    with dut.add_reg("tr2", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0)
        reg.Field("b1", offset=8, width=8, reset=0)
    bits1 = dut.tr1.b0
    bits2 = dut.tr2.b0
    assert bits1.data() == 0
    assert bits1[1].is_to_be_read() == False
    bits2.set_data(0b0010).read()
    bits1.copy(bits2)
    assert bits1.data() == 0b0010
    assert bits1[1].is_to_be_read() == True
    assert bits1.is_to_be_read() == True

def test_status_string_methods_work():
    with dut.add_reg("tr1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0)
        reg.Field("b1", offset=8, width=8, reset=0)
    reg = dut.tr1
    reg[3:0].set_data(0x5)
    reg[7:4].set_overlay("overlayx")
    reg[15:8].set_data(0xAA)
    reg[10].set_overlay("overlayy")
    assert reg.status_str("write") == "A[1v10]V5"
    reg.reset()
    reg.clear_flags()
    reg.set_overlay(None)
    assert reg.status_str("write") == "0000"
    assert reg.status_str("read") == "XXXX"
    reg[7:4].set_data(5).read()
    assert reg.status_str("read") == "XX5X"
    reg[7:4].set_data(5).read()
    reg[14].set_data(0).read()
    assert reg.status_str("read") == "[x0xx]X5X"
    reg[3:0].capture()
    assert reg.status_str("read") == "[x0xx]X5S"
    reg[12:8].set_overlay("overlayx")
    reg[12:8].read()
    assert reg.status_str("read") == "[x0xv]V5S"
    reg[15].capture()
    assert reg.status_str("read") == "[s0xv]V5S"
    reg[7:4].set_undefined()
    assert reg.status_str("read") == "[s0xv]V?S"
    
def test_status_str_works_on_non_nibble_aligned_regs():
    with dut.add_reg("mr1", 0) as reg:
        reg.Field("b1", offset=0, width=11, reset=0)
    mr1 = dut.mr1
    assert mr1.b1.status_str("write") == "000"
    assert mr1.b1.status_str("read") == "[xxx]XX"
    mr1.b1.read()
    assert mr1.b1.status_str("read") == "000"
    mr1.b1.set_data(0xFFF).read()
    assert mr1.b1.status_str("read") == "7FF"

def test_the_flags_methods():
    with dut.add_reg("tr1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0)
        reg.Field("b1", offset=8, width=8, reset=0)
    reg = dut.tr1
    assert reg.read_enables() == 0
    assert reg.capture_enables() == 0
    assert reg.overlay_enables() == 0
    reg[7:4].read()
    assert reg.read_enables() == 0xF0
    reg.b1.set_overlay("blah")
    assert reg.overlay_enables() == 0xFF00
    reg[11:4].capture()
    assert reg.capture_enables() == 0x0FF0

def test_regs_are_correctly_marked_for_read():
    with dut.add_reg("tr1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0)
        reg.Field("b1", offset=8, width=8, reset=0)
    assert dut.tr1.is_to_be_read() == False
    dut.tr1.read()
    assert dut.tr1.is_to_be_read() == True

def test_reg_method_can_be_used_to_test_for_the_presence_of_a_register():
    with dut.add_reg("tr1", 0, size=16) as reg:
        reg.Field("b0", offset=0, width=8, reset=0)
        reg.Field("b1", offset=8, width=8, reset=0)
    assert dut.reg("tr1") != None
    assert dut.reg("tr2") == None

def test_registers_can_be_declared_in_block_format_with_descriptions():
    with origen.reg_description_parsing():
        # This is test reg 1
        with dut.add_reg("tr1", 0, size=16) as reg:
            # This is tr1 field b0
            reg.Field("b0", offset=0, width=8, reset=0)
            # This is tr1 field b1
            # With multi-lines
            reg.Field("b1", offset=8, width=8, reset=0)
        # This is test reg 2
        # With a 2 line description
        with dut.add_reg("tr2", 0, size=16) as reg:
            reg.Field("b0", offset=0, width=8, reset=0)
            reg.Field("b1", offset=8, width=8, reset=0)

    assert dut.tr1.description() == "This is test reg 1"
    assert dut.tr2.description() == "This is test reg 2\nWith a 2 line description"
    assert dut.tr1.b0.description() == "This is tr1 field b0"
    assert dut.tr1.b1.description() == "This is tr1 field b1\nWith multi-lines"
    
def test_register_descriptions_can_be_supplied_via_the_API():
    with dut.add_reg("tr1", 0, size=16, description="Hello, I'm tr1") as reg:
        reg.Field("b0", offset=0, width=8, reset=0, description="I'm b0")
        reg.Field("b1", offset=8, width=8, reset=0, description="I'm b1")

    assert dut.tr1.description() == "Hello, I'm tr1"
    assert dut.tr1.b0.description() == "I'm b0"
    assert dut.tr1.b1.description() == "I'm b1"

#    it "bit value descriptions work" do
#      Origen.app.unload_target!
#      nvm = OrigenCoreSupport::NVM::NVMSub.new
#      nvm.add_reg_with_block_format
#      nvm.reg(:dreg).bits(:bit15).bit_value_descriptions.size.should == 0
#      nvm.reg(:dreg).bits(:bit14).bit_value_descriptions.size.should == 2
#      nvm.reg(:dreg3).bits(:bit15).bit_value_descriptions.size.should == 0
#      nvm.reg(:dreg3).bits(:bit14).bit_value_descriptions.size.should == 2
#      nvm.reg(:dreg4).bits(:busy).bit_value_descriptions.size.should == 19
#      nvm.reg(:dreg4).bits(:busy).bit_value_descriptions(format: :hex).size.should == 19
#      nvm.reg(:dreg4).bits(:busy).bit_value_descriptions(format: :dec).size.should == 19
#      nvm.reg(:dreg).bits(:bit14).bit_value_descriptions[0].should == "Coolness is disabled"
#      nvm.reg(:dreg).bits(:bit14).bit_value_descriptions[1].should == "Coolness is enabled"
#      nvm.reg(:dreg3).bits(:bit14).bit_value_descriptions[0].should == "Coolness is disabled"
#      nvm.reg(:dreg3).bits(:bit14).bit_value_descriptions[1].should == "Coolness is enabled"
#      nvm.reg(:dreg4).bits(:busy).bit_value_descriptions[8].should == "Job8"
#      nvm.reg(:dreg4).bits(:busy).bit_value_descriptions(format: :dec)[1000].should == "Job8"
#      nvm.reg(:dreg4).bits(:busy).bit_value_descriptions(format: :hex)[4096].should == "Job8"
#      lambda { nvm.reg(:dreg4).bits(:busy).bit_value_descriptions(format: :octal) }.should raise_error
#      nvm.reg(:dreg).bits(:bit14).description(include_bit_values: false, include_name: false).should == ["This does something cool"]
#      nvm.reg(:dreg3).bits(:bit14).description(include_bit_values: false, include_name: false).should == ["This does something cool"]
#    end

def test_subset_method():
    with dut.add_reg("regx", 0) as reg:
        reg.Field("data", offset=0, width=32)

    reg = dut.regx
    reg.set_data(0x1111_224C)
    assert reg.data() == 0x1111_224C
    lower = reg.subset(0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15)
    assert lower.data() == 0x224C
    lower = reg.subset(15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0)
    assert lower.data() == 0x224C

def test_reg_and_bit_reset_data_can_be_fetched():
    with dut.add_reg("reset_test", 100) as reg:
        reg.Field("upper", offset=16, width=16, reset=0x5555)
        reg.Field("lower", offset=1, width=15, reset=0)
        reg.Field("bit0", offset=0, reset=1)

    assert dut.reset_test.reset_val() == 0x55550001
    assert dut.reset_test.upper.reset_val() == 0x5555
    assert dut.reset_test.lower.reset_val() == 0x0000
    assert dut.reset_test.bit0.reset_val() == 0x1 
    dut.reset_test.set_data(0xFFFF_FFFF)
    assert dut.reset_test.reset_val() == 0x55550001
    assert dut.reset_test.data() == 0xFFFF_FFFF

def test_reset_values_work_correct_in_real_life_case():
    with dut.add_reg("proth", 0x0024, size=32) as reg:
        reg.Field("fprot7", offset=24, width=8, reset=0xFF)
        reg.Field("fprot6", offset=16, width=8, reset=0xEE)
        reg.Field("fprot5", offset=8,  width=8, reset=0xDD)
        reg.Field("fprot4", offset=0,  width=8, reset=0x11)

    assert dut.proth.data() == 0xFFEE_DD11
    assert dut.proth.reset_val() == 0xFFEE_DD11
    assert dut.proth.fprot7.reset_val() == 0xFF
    assert dut.proth.fprot6.reset_val() == 0xEE
    assert dut.proth.fprot5.reset_val() == 0xDD
    assert dut.proth.fprot4.reset_val() == 0x11

def test_a_few_different_bit_names_can_be_tried():
    with dut.add_reg("multi_name", 0x0030) as reg:
        reg.Field("some_bit3", offset=3)
        reg.Field("some_bit2", offset=2)
        reg.Field("some_bit1", offset=1)
        reg.Field("some_bit0", offset=0)

    dut.multi_name.set_data(0)
    dut.multi_name.try_fields("blah1", "blah_blah1", "some_bit1").set_data(1)
    assert dut.multi_name.data() == 2
    # X chosen here specifically in the name so that when sorted it comes
    # after the name that will match a bit in this register
    dut.multi_name.try_fields("some_bit0", "xlah0", "xlah_blah0").set_data(1)
    assert dut.multi_name.data() == 3
    dut.multi_name.try_fields("some_bit2", "some_bit3", "some_bit4").set_data(3)
    assert dut.multi_name.data() == 0xF

#    it "the bits method accepts an array of bit ids" do
#      reg :tr, 0 do
#        bits 31..0, :data
#      end
#
#      reg(:tr).bits([4,5,6,7]).write(0xF)
#      reg(:tr).data.should == 0x0000_00F0
#    end
#
#    it "the Reg.read method should accept a mask option" do
#      reg :tr2, 0 do
#        bits 31..0, :data
#      end
#
#      reg(:tr2).read!(0x1234_5678, mask: 0x0000_00F0)
#      reg(:tr2).data.should == 0x1234_5678
#      reg(:tr2).bit(0).is_to_be_read?.should == false
#      reg(:tr2).bit(1).is_to_be_read?.should == false
#      reg(:tr2).bit(2).is_to_be_read?.should == false
#      reg(:tr2).bit(3).is_to_be_read?.should == false
#      reg(:tr2).bit(4).is_to_be_read?.should == true
#      reg(:tr2).bit(5).is_to_be_read?.should == true
#      reg(:tr2).bit(6).is_to_be_read?.should == true
#      reg(:tr2).bit(7).is_to_be_read?.should == true
#      reg(:tr2).bit(8).is_to_be_read?.should == false
#    end
#
#    specify "clear_read_flag clears is_to_be_read status " do
#      reg :tr3, 0 do
#        bits 31..0, :data
#      end
#
#        reg(:tr3).read(0x0F)
#        reg(:tr3).bit(0).is_to_be_read?.should == true
#        reg(:tr3).bit(0).clear_read_flag
#        reg(:tr3).bit(0).is_to_be_read?.should == false
#    end
#
#    specify "reset values can be set at register level" do
#      reg :reset2, 0, reset: 0x3 do
#        bit 3, :w
#        bits 2..1, :x
#        bit 0, :y
#      end
#      reg :reset3, 0, reset: :undefined do
#        bit 1, :x
#        bit 0, :y
#      end
#      reg :reset4, 0, reset: :memory do
#        bit 1, :x
#        bit 0, :y
#      end
#      reg :reset5, 0, reset: :memory do
#        bit 1, :x
#        bit 0, :y, reset: :undefined
#      end
#
#      reg(:reset2).data.should == 3
#      reg(:reset3).bit(:x).reset_val.should == :undefined
#      reg(:reset3).bit(:y).reset_val.should == :undefined
#      reg(:reset4).bit(:x).reset_val.should == :memory
#      reg(:reset4).bit(:y).reset_val.should == :memory
#      reg(:reset5).bit(:x).reset_val.should == :memory
#      reg(:reset5).bit(:y).reset_val.should == :undefined
#    end
#
#    specify "a memory location can be set on a register" do
#      reg :reset6, 0, memory: 0x1234_0000 do
#        bit 1, :x
#        bit 0, :y
#      end
#
#      reg(:reset6).bit(:x).reset_val.should == :memory
#      reg(:reset6).bit(:y).reset_val.should == :memory
#      reg(:reset6).memory.should == 0x1234_0000
#    end
#
#    specify "access can be set and tested at reg level" do
#      reg :access1, 0, access: :w1c do
#        bits 2..1, :x
#        bit  0,    :y
#      end
#
#      reg(:access1).bits(:x).w1c?.should == true
#      reg(:access1).bit(:y).w1c?.should == true
#      reg(:access1).w1c?.should == true
#      reg(:access1).w1s?.should == false
#      # Verify the access can be pulled for a mutli-bit collection
#      reg(:access1).bits(:x).access.should == :w1c
#    end
#
#    specify "sub collections of bits can be made from bit collections" do
#      reg :reg1, 0 do
#        bits 31..0, :data
#      end
#
#      reg(:reg1)[:data].size.should == 32
#      reg(:reg1)[31..0].size.should == 32
#      reg(:reg1).bits(:data).size.should == 32
#      reg(:reg1).bits(:data)[15..8].size.should == 8
#      reg(:reg1).bits(:data)[15..8].write(0xFF)
#      reg(:reg1).data.should == 0x0000_FF00
#      reg(:reg1).reset
#      # Verify that bits are stored in consistent order
#      reg(:reg1).to_a.map {|b| b.position }.should == 
#        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]
#      reg(:reg1)[].to_a.map {|b| b.position }.should == 
#        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]
#      reg(:reg1)[15..0].to_a.map {|b| b.position }.should ==
#        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
#      reg(:reg1)[][15..0].to_a.map {|b| b.position }.should ==
#        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
#      reg(:reg1)[15..0][15..8].to_a.map {|b| b.position }.should ==
#        [8, 9, 10, 11, 12, 13, 14, 15]
#      reg(:reg1)[15..0][15..8][3..0].to_a.map {|b| b.position }.should ==
#        [8, 9, 10, 11]
#
#      reg(:reg1)[15..0][15..8][3..0].write(0xF)
#      reg(:reg1).data.should == 0x0000_0F00
#
#      # When 1 bit requested just return that bit, this is consistent with the original
#      # behaviour before sub collections were added
#      reg(:reg1).bits(:data)[15].class.should == Origen::Registers::Bit
#      # Calling bits on a bit collection with no args should just return self
#      reg(:reg1).bits(:data).bits.size.should == 32
#    end
#
#    specify "indexed references to missing bits should return nil" do
#      reg :reg2, 0, size: 8 do
#        bits 7..0, :data
#      end
#      reg(:reg2)[7].should be
#      reg(:reg2)[8].should == nil
#    end
#
#    specify "regs can be deleted" do
#      class RegOwner
#        include Origen::Model
#        def initialize
#          reg :reg1, 0, size: 8 do
#            bits 7..0, :d1
#          end
#          reg :reg2, 0, size: 8 do
#            bits 7..0, :d1
#          end
#          reg :reg3, 0, size: 8 do
#            bits 7..0, :d1
#          end
#        end
#      end
#      top = RegOwner.new
#      top.has_reg?(:reg1).should == true
#      top.has_reg?(:reg2).should == true
#      top.has_reg?(:reg3).should == true
#      top.has_reg?(:reg4).should == false
#      top.del_reg(:reg2)
#      top.has_reg?(:reg1).should == true
#      top.has_reg?(:reg2).should == false
#      top.has_reg?(:reg3).should == true
#      top.has_reg?(:reg4).should == false
#      top.delete_registers
#      top.has_reg?(:reg1).should == false
#      top.has_reg?(:reg2).should == false
#      top.has_reg?(:reg3).should == false
#      top.has_reg?(:reg4).should == false
#    end
#
#    specify "block read/write method can set/read bits" do
#      add_reg :blregtest,   0x00,  4,  :y       => { :pos => 0},
#                                       :x       => { :pos => 1, :bits => 2 },
#                                       :w       => { :pos => 3 }
#      reg(:blregtest).data.should == 0x0
#      reg(:blregtest).write! do |r|
#        r.bits(:y).write(1)
#        r.bits(:x).write(0x2)
#        r.bits(:w).write(1)
#      end
#      reg(:blregtest).data.should == 0xD
#
#      reg(:blregtest).write(0)
#      reg(:blregtest).x.write! do |b|
#        b[1].write(1)
#      end
#      reg(:blregtest).data.should == 0b0100
#
#      reg(:blregtest).read! do |r|
#        r.bits(:y).read
#      end      
#      reg(:blregtest).bits(:y).is_to_be_read?.should == true
#      reg(:blregtest).bits(:x).is_to_be_read?.should == false
#      reg(:blregtest).bits(:w).is_to_be_read?.should == false
#    end
#
#    it "write method can override a read-only register bitfield with :force = true" do
#        reg :reg, 0x0, 32, description: 'reg' do
#            bits 7..0,   :field1, reset: 0x0, access: :rw
#            bits 15..8,  :field2, reset: 0x0, access: :ro
#            bits 23..16, :field3, reset: 0x0, access: :ro
#            bits 31..24, :field4, reset: 0x0, access: :rw
#        end
#        reg(:reg).bits(:field1).write(0xf)
#        reg(:reg).bits(:field2).write(0xf)
#        reg(:reg).bits(:field3).write(0xf)
#        reg(:reg).bits(:field4).write(0xf)
#        reg(:reg).bits(:field1).data.should == 0xf
#        reg(:reg).bits(:field2).data.should == 0x0
#        reg(:reg).bits(:field3).data.should == 0x0
#        reg(:reg).bits(:field4).data.should == 0xf
#
#        reg(:reg).bits(:field1).write(0xa, force: true)
#        reg(:reg).bits(:field2).write(0xa, force: true)
#        reg(:reg).bits(:field3).write(0xa, force: true)
#        reg(:reg).bits(:field4).write(0xa, force: true)
#        reg(:reg).bits(:field1).data.should == 0xa
#        reg(:reg).bits(:field2).data.should == 0xa
#        reg(:reg).bits(:field3).data.should == 0xa
#        reg(:reg).bits(:field4).data.should == 0xa
#    end
#
#    it 'regs with all bits writable can be created via a shorthand' do
#      class RegBlock
#        include Origen::Model
#        def initialize
#          reg :reg1, 0
#          reg :reg2, 4, size: 8
#          reg :reg3, 5, size: 8, reset: 0xFF
#        end
#      end
#
#      b = RegBlock.new
#      b.reg1.size.should == 32
#      b.reg2.size.should == 8
#      b.reg1.write(0xFFFF_FFFF)
#      b.reg1.data.should == 0xFFFF_FFFF
#      b.reg2.write(0xFF)
#      b.reg2.data.should == 0xFF
#      b.reg3.data.should == 0xFF
#    end

def test_regs_can_shift_left():
    dut.add_simple_reg("sr1", 0, size=4)
    sr1 = dut.sr1
    sr1.set_data(0xF)
    assert sr1.data() == 0b1111
    v = sr1.shift_left()
    assert sr1.data() == 0b1110
    assert v == 1
    v = sr1.shift_left()
    assert sr1.data() == 0b1100
    assert v == 1
    v = sr1.shift_left(1)
    assert sr1.data() == 0b1001
    assert v == 1
    v = sr1.shift_left(1)
    assert sr1.data() == 0b0011
    assert v == 1
    v = sr1.shift_left(1)
    assert sr1.data() == 0b0111
    assert v == 0

def test_regs_can_shift_right():
    dut.add_simple_reg("sr2", 0, size=4)
    sr2 = dut.sr2
    sr2.set_data(0xF)
    assert sr2.data() == 0b1111
    v = sr2.shift_right()
    assert sr2.data() == 0b0111
    assert v == 1
    v = sr2.shift_right()
    assert sr2.data() == 0b0011
    assert v == 1
    v = sr2.shift_right(1)
    assert sr2.data() == 0b1001
    assert v == 1
    v = sr2.shift_right(1)
    assert sr2.data() == 0b1100
    assert v == 1
    v = sr2.shift_right(1)
    assert sr2.data() == 0b1110
    assert v == 0

#    it "read only bits can be forced to write" do
#      add_reg :ro_test, 0, access: :ro
#      ro_test.write(0xFFFF_FFFF)
#      ro_test.data.should == 0
#      ro_test.write(0xFFFF_FFFF, force: true)
#      ro_test.data.should == 0xFFFF_FFFF
#      # Read requests apply force by default
#      ro_test.read(0x5555_5555)
#      ro_test.data.should == 0x5555_5555
#    end
#
#    it "inverse and reverse data methods work" do
#      add_reg :revtest, 0
#      revtest.write(0x00FF_AA55)
#      revtest.data.should == 0x00FF_AA55
#      revtest.data_b.should == 0xFF00_55AA
#      revtest.data_reverse.should == 0xAA55_FF00
#    end
#
#    it "multi-named bit collections work" do
#      add_reg :mnbit,   0x03,  8,  :d  => { pos: 6, bits: 2 },
#                                   :b  => { pos: 4, bits: 2 },
#                                   :c  => { pos: 2, bits: 2 },
#                                   :a  => { pos: 0, bits: 2 }
#
#      mnbit.data.should == 0
#      mnbit.bits(:d, :a).write(0b0110)
#      mnbit.d.data.should == 0b01
#      mnbit.a.data.should == 0b10
#      mnbit.data.should == 0b01000010
#      mnbit.write(0)
#      mnbit.bits(:b, :c).write(0b0110)
#      mnbit.b.data.should == 0b01
#      mnbit.c.data.should == 0b10
#      mnbit.data.should == 0b00011000
#    end
#
#    it "regs can be grabbed using regular expression" do
#      class RegOwner
#        include Origen::Model
#        def initialize
#          reg :adc0_cfg, 0, size: 8 do
#            bits 7..0, :d1
#          end
#          reg :adc1_cfg, 0, size: 8 do
#            bits 7..0, :d1
#          end
#          reg :dac_cfg, 0, size: 8 do
#            bits 7..0, :d1
#          end
#        end
#      end
#      top = RegOwner.new
#      top.regs.inspect.should == "[:adc0_cfg, :adc1_cfg, :dac_cfg]"
#      top.regs('/adc\d_cfg/').inspect.should == "[:adc0_cfg, :adc1_cfg]"
#      top.regs('/cfg/').inspect.should == "[:adc0_cfg, :adc1_cfg, :dac_cfg]"
#      expected_output = <<-EOT
#[
#0x0 - :dac_cfg
#  \u2552\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2564\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2555
#  \u2502      7      \u2502      6      \u2502      5      \u2502      4      \u2502      3      \u2502      2      \u2502      1      \u2502      0      \u2502
#  \u2502                                                    d1[7:0]                                                    \u2502
#  \u2502                                                      0x0                                                      \u2502
#  \u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518]
#  EOT
#      expect do
#        top.regs('/dac/').show
#      end.to output(expected_output).to_stdout
#    end
#  end
#end