use super::{AccessType, BitCollection, BitOrder};
use crate::Error;
use crate::Result as OrigenResult;
use crate::{Dut, LOGGER};
use indexmap::map::IndexMap;
use num_bigint::BigUint;
use std::cmp;
use std::sync::MutexGuard;

#[derive(Debug)]
pub struct Register {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    // TODO: What is this?!
    /// The dimension of the register, defaults to 1.
    pub dim: u32,
    /// Address offset from the start of the parent address block in address_unit_bits.
    pub offset: usize,
    /// The size of the register in bits.
    pub size: usize,
    pub access: AccessType,
    pub fields: IndexMap<String, Field>,
    /// Contains all bits implemented by the register, bits[i] will return None if
    /// the bit is unimplemented/undefined
    pub bits: Vec<usize>,
    // TODO: Should this be defined on Register, or inherited from address block/memory map?
    pub bit_order: BitOrder,
}

impl Default for Register {
    fn default() -> Register {
        Register {
            id: 0,
            name: "".to_string(),
            description: None,
            dim: 1,
            offset: 0,
            size: 32,
            access: AccessType::ReadWrite,
            fields: IndexMap::new(),
            bits: Vec::new(),
            bit_order: BitOrder::LSB0,
        }
    }
}

/// An iterator for a register's fields which yields them in offset order, starting from 0.
/// An instance of this iterator is returned by my_reg.named_bits().
pub struct RegisterFieldIterator<'a> {
    reg: &'a Register,
    field_names: Vec<String>,
    // Tracks index through the field_names array
    index: usize,
    // Keeps track of the last register bit position
    pos: usize,
}

impl<'a> RegisterFieldIterator<'a> {
    fn new(reg: &Register, include_spacers: bool) -> RegisterFieldIterator {
        // Derive the order of iteration when this iterator is created, then
        // store the names of the fields locally in the order that is required.
        // This can no doubt be done more elegantly, but for now the borrow checker
        // has broken me.
        let mut field_names: Vec<String>;

        let mut fields: Vec<&Field> = reg.fields.values().collect();
        fields.sort_by_key(|f| f.offset);
        if include_spacers {
            field_names = Vec::new();
            let mut pos = 0;
            for field in fields {
                if pos != field.offset {
                    field_names.push("_spacer_".to_string());
                }
                field_names.push(field.name.clone());
                pos = field.offset + field.width;
            }
            if pos != reg.size {
                field_names.push("_spacer_".to_string());
            }
        } else {
            field_names = fields.iter().map(|m| m.name.clone()).collect();
        }

        RegisterFieldIterator {
            reg: reg,
            field_names: field_names,
            index: 0,
            pos: 0,
        }
    }

    fn field_name_index(&self, ascending: bool) -> usize {
        if ascending {
            self.index
        } else {
            self.field_names.len() - self.index - 1
        }
    }

    fn field(&self, ascending: bool, next: bool) -> Option<&Field> {
        let mut i = self.field_name_index(ascending);
        if next {
            if ascending {
                i += 1;
                if i == self.field_names.len() {
                    return None;
                }
            } else {
                if i == 0 {
                    return None;
                }
                i -= 1;
            }
        }
        self.reg.fields.get(&self.field_names[i])
    }

    // Like next() but handles both forwards and backwards iteration - it was easier to implement
    // them in parallel instead of split accross next() and next_back()
    fn get(&mut self, ascending: bool) -> Option<SummaryField> {
        if self.index >= self.field_names.len() {
            None
        } else {
            if self.index == 0 {
                if ascending {
                    self.pos = 0;
                } else {
                    self.pos = self.reg.size;
                }
            }

            let summary: SummaryField;

            {
                let name = &self.field_names[self.field_name_index(ascending)];
                if name == "_spacer_" {
                    let width;
                    let offset;
                    if ascending {
                        offset = self.pos;
                        // Look ahead to the next field to work out how wide this spacer needs to be
                        width = match self.field(true, true) {
                            Some(x) => x.offset - offset,
                            None => self.reg.size,
                        };
                    } else {
                        offset = match self.field(false, true) {
                            Some(x) => x.offset + x.width,
                            None => 0,
                        };
                        width = self.pos - offset;
                    }
                    summary = SummaryField {
                        reg_id: self.reg.id,
                        name: "spacer".to_string(),
                        offset: offset,
                        width: width,
                        spacer: true,
                    };
                } else {
                    let field = self.field(ascending, false).unwrap();
                    summary = SummaryField {
                        reg_id: self.reg.id,
                        name: field.name.clone(),
                        offset: field.offset,
                        width: field.width,
                        spacer: false,
                    };
                }
            }

            self.index += 1;
            if ascending {
                self.pos = summary.offset + summary.width;
            } else {
                self.pos = summary.offset;
            }
            Some(summary)
        }
    }
}

impl<'a> Iterator for RegisterFieldIterator<'a> {
    type Item = SummaryField;

    fn next(&mut self) -> Option<SummaryField> {
        self.get(true)
    }
}

// Enables reg.named_bits(true).rev()
impl<'a> DoubleEndedIterator for RegisterFieldIterator<'a> {
    fn next_back(&mut self) -> Option<SummaryField> {
        self.get(false)
    }
}

impl Register {
    /// Returns a path to this register like "dut.my_block.my_map.my_address_block.my_reg", but the map and address block portions
    /// will be inhibited when they are 'default'. This is to keep map and address block concerns out of the view of users who
    /// don't use them and simply define regs at the top-level of the block.
    pub fn friendly_path(&self, _dut: &MutexGuard<Dut>) -> String {
        format!("friendly.path.to.be.implemented.{}", self.name)
    }

    /// Returns the fully-resolved address taking into account all base addresses defined by the parent hierarchy
    pub fn address(&self, _dut: &MutexGuard<Dut>) -> u64 {
        0x1000
    }

    /// Returns an iterator for the register's fields which yields them (as SummaryFields) in offset order, starting from lowest.
    /// The caller can elect whether or not spacer fields should be inserted to represent un-implemented bits.
    pub fn named_bits(&self, include_spacers: bool) -> RegisterFieldIterator {
        RegisterFieldIterator::new(&self, include_spacers)
    }

    pub fn console_display(
        &self,
        dut: &MutexGuard<Dut>,
        with_bit_order: Option<BitOrder>,
        fancy_output: bool,
    ) -> OrigenResult<String> {
        let bit_order = match with_bit_order {
            Some(x) => x,
            None => BitOrder::LSB0,
        };
        if bit_order != self.bit_order {
            LOGGER.warning(&format!(
                "Register displayed with {:?} numbering, but defined with {:?} numbering",
                bit_order, self.bit_order
            ));
            LOGGER.warning(&format!(
                "Access (and display) this register with explicit numbering like this:"
            ));
            LOGGER.warning(&format!(""));
            LOGGER.warning(&format!(
                "   reg('{}').with_msb0        # bit numbering scheme is msb0",
                self.name
            ));
            LOGGER.warning(&format!(
                "   reg('{}').with_lsb0        # bit numbering scheme is lsb0 (default)",
                self.name
            ));
            LOGGER.warning(&format!(
                "   reg('{}')                  # bit numbering scheme is lsb0 (default)",
                self.name
            ));
        }

        // This fancy_output option is passed in via option
        // Even better, the output could auto-detect 7-bit vs 8-bit terminal output and adjust the parameter, but that's for another day
        let horiz_double_line;
        let horiz_double_tee_down;
        //let horiz_double_tee_up;
        let corner_double_up_left;
        let corner_double_up_right;
        let horiz_single_line;
        //let horiz_single_tee_down;
        let horiz_single_tee_up;
        let horiz_single_cross;
        let horiz_double_cross;
        let corner_single_down_left;
        let corner_single_down_right;
        let vert_single_line;
        let vert_single_tee_left;
        let vert_single_tee_right;

        if fancy_output {
            horiz_double_line = "═";
            horiz_double_tee_down = "╤";
            //horiz_double_tee_up = "╧";
            corner_double_up_left = "╒";
            corner_double_up_right = "╕";
            horiz_single_line = "─";
            //horiz_single_tee_down = "┬";
            horiz_single_tee_up = "┴";
            horiz_single_cross = "┼";
            horiz_double_cross = "╪";
            corner_single_down_left = "└";
            corner_single_down_right = "┘";
            vert_single_line = "│";
            vert_single_tee_left = "┤";
            vert_single_tee_right = "├";
        } else {
            horiz_double_line = "=";
            horiz_double_tee_down = "=";
            //horiz_double_tee_up = "=";
            corner_double_up_left = ".";
            corner_double_up_right = ".";
            horiz_single_line = "-";
            //horiz_single_tee_down = "-";
            horiz_single_tee_up = "-";
            horiz_single_cross = "+";
            horiz_double_cross = "=";
            corner_single_down_left = "`";
            corner_single_down_right = "\'";
            vert_single_line = "|";
            vert_single_tee_left = "<";
            vert_single_tee_right = ">";
        }
        let bit_width = 13;
        let mut desc: Vec<String> = vec![format!(
            "\n{:#X} - {}",
            self.address(dut),
            self.friendly_path(dut)
        )];
        let r = self.size % 8;
        if r == 0 {
            let mut s = "  ".to_string()
                + corner_double_up_left
                + &(horiz_double_line.repeat(bit_width) + horiz_double_tee_down).repeat(8);
            s.pop();
            desc.push(s + corner_double_up_right);
        } else {
            let mut s = "  ".to_string()
                + &(" ".repeat(bit_width + 1)).repeat(8 - r)
                + corner_double_up_left
                + &(horiz_double_line.repeat(bit_width) + horiz_double_tee_down).repeat(r);
            s.pop();
            desc.push(s + corner_double_up_right);
        }

        let num_bytes = (self.size as f32 / 8.0).ceil() as usize;
        for byte_index in 0..num_bytes {
            let byte_number = num_bytes - byte_index;
            // The max bit number in the current byte row
            let max_bit: usize = (byte_number * 8) - 1;
            // The min bit number in the current byte row
            let min_bit: usize = max_bit + 1 - 8;

            // BIT INDEX ROW
            let mut line = "  ".to_string();
            for i in 0..8 {
                let bit_num = (byte_number * 8) - i - 1;
                if bit_num > self.size - 1 {
                    line += &" ".repeat(bit_width);
                } else {
                    if bit_order == BitOrder::LSB0 {
                        line += vert_single_line;
                        line += &format!("{: ^bit_width$}", bit_num, bit_width = bit_width);
                    } else {
                        line += vert_single_line;
                        line += &format!(
                            "{: ^bit_width$}",
                            self.size - bit_num - 1,
                            bit_width = bit_width
                        );
                    }
                }
            }
            line += vert_single_line;
            desc.push(line);

            // BIT NAME ROW
            let mut line = "  ".to_string();
            let mut first_done = false;
            for field in self.named_bits(true).rev() {
                if is_field_in_range(&field, max_bit, min_bit) {
                    if max_bit > (self.size - 1) && !first_done {
                        for _i in 0..(max_bit - (self.size - 1)) {
                            line += &" ".repeat(bit_width + 1);
                        }
                    }

                    if field.width > 1 {
                        if !field.spacer {
                            //      if contiguous
                            let bit_name = format!(
                                "{}[{}:{}]",
                                field.name,
                                max_bit_in_range(&field, max_bit, min_bit, &bit_order),
                                min_bit_in_range(&field, max_bit, min_bit, &bit_order),
                            );
                            let bit_span = num_bits_in_range(&field, max_bit, min_bit);

                            // This is legacy code that handled non-contiguous fields
                            //       else
                            //            upper = _max_bit_in_range(bit, max_bit, min_bit, options) + bitcounter - bit.size
                            //            lower = _min_bit_in_range(bit, max_bit, min_bit, options) + bitcounter - bit.size
                            //            if dolsb0
                            //              bit_name = "#{name}[#{upper}:#{lower}]"
                            //            else
                            //              bit_name = "#{name}[#{upper}:#{lower}]"
                            //            end
                            //            bit_span = upper - lower + 1
                            //        end

                            let width = (bit_width * bit_span) + bit_span - 1;
                            let txt = &bit_name.chars().take(width - 2).collect::<String>();
                            line += vert_single_line;
                            line += &format!("{: ^bit_width$}", txt, bit_width = width);
                        } else {
                            for i in 0..field.width {
                                if is_index_in_range(field.offset + i, max_bit, min_bit) {
                                    line += vert_single_line;
                                    line += &" ".repeat(bit_width);
                                }
                            }
                        }
                    } else {
                        let bit_name = &field.name;
                        let txt = &bit_name.chars().take(bit_width - 2).collect::<String>();
                        line += vert_single_line;
                        line += &format!("{: ^bit_width$}", txt, bit_width = bit_width);
                    }
                }
                first_done = true
            }
            line += vert_single_line;
            desc.push(line);

            // BIT STATE ROW
            let mut line = "  ".to_string();
            let mut first_done = false;
            for field in self.named_bits(true).rev() {
                if is_field_in_range(&field, max_bit, min_bit) {
                    if max_bit > (self.size - 1) && !first_done {
                        for _i in 0..(max_bit - self.size - 1) {
                            line += &" ".repeat(bit_width + 1);
                        }
                    }

                    if field.width > 1 {
                        if !field.spacer {
                            let bits = field.to_bit_collection(dut);
                            let mut value = "".to_string();
                            if bits.has_known_value() {
                                let v = bits
                                    .range(
                                        max_bit_in_range(&field, max_bit, min_bit, &bit_order),
                                        min_bit_in_range(&field, max_bit, min_bit, &bit_order),
                                    )
                                    .data()
                                    .unwrap();
                                value += &format!("{:#X}", v);
                            } else {
                                //          if bit.reset_val == :undefined
                                value += "X";
                                //          else
                                //            value = 'M'
                                //          end
                            }
                            //        value += _state_desc(bit)
                            let bit_span = num_bits_in_range(&field, max_bit, min_bit);
                            let width = bit_width * bit_span;
                            line += vert_single_line;
                            line += &format!(
                                "{: ^bit_width$}",
                                value,
                                bit_width = width + bit_span - 1
                            );
                        } else {
                            for i in 0..field.width {
                                if is_index_in_range(field.offset + i, max_bit, min_bit) {
                                    line += vert_single_line;
                                    line += &" ".repeat(bit_width);
                                }
                            }
                        }
                    } else {
                        if !field.spacer {
                            let bits = field.to_bit_collection(dut);
                            let mut value = "".to_string();
                            if bits.has_known_value() {
                                //  val = bit.val
                                value += &format!("{}", bits.data().unwrap());
                            } else {
                                //  if bit.reset_val == :undefined
                                value += "X";
                                //  else
                                //    val = 'M'
                                //  end
                            }
                            //value = "#{val}" + _state_desc(bit)
                            line += vert_single_line;
                            line += &format!("{: ^bit_width$}", value, bit_width = bit_width);
                        } else {
                            line += vert_single_line;
                            line += &" ".repeat(bit_width);
                        }
                    }
                }
                first_done = true;
            }
            line += vert_single_line;
            desc.push(line);

            if self.size >= 8 {
                let r = self.size % 8;
                if byte_index == 0 && r != 0 {
                    let mut s = "  ".to_string()
                        + corner_double_up_left
                        + &(horiz_double_line.repeat(bit_width) + horiz_double_tee_down)
                            .repeat(8 - r);
                    s.pop();
                    s += horiz_double_cross;
                    s += &horiz_single_line.repeat((bit_width + 1) * r);
                    s.pop();
                    desc.push(s + vert_single_tee_left);
                } else {
                    if byte_index == num_bytes - 1 {
                        let mut s = "  ".to_string()
                            + corner_single_down_left
                            + &(horiz_single_line.repeat(bit_width) + horiz_single_tee_up)
                                .repeat(8);
                        s.pop();
                        desc.push(s + corner_single_down_right);
                    } else {
                        let mut s = "  ".to_string()
                            + vert_single_tee_right
                            + &(horiz_single_line.repeat(bit_width) + horiz_single_cross).repeat(8);
                        s.pop();
                        desc.push(s + vert_single_tee_left);
                    }
                }
            } else {
                let mut s = "  ".to_string()
                    + &" ".repeat((bit_width + 1) * (8 - self.size))
                    + corner_single_down_left
                    + &(horiz_single_line.repeat(bit_width) + horiz_single_tee_up)
                        .repeat(self.size);
                s.pop();
                desc.push(s + corner_single_down_right);
            }
        }
        Ok(desc.join("\n"))
    }

    pub fn add_field(
        &mut self,
        name: &str,
        description: &str,
        offset: usize,
        width: usize,
        access: &str,
        reset: &BigUint,
    ) -> OrigenResult<&mut Field> {
        let acc: AccessType = match access.parse() {
            Ok(x) => x,
            Err(msg) => return Err(Error::new(&msg)),
        };
        let f = Field {
            name: name.to_string(),
            description: description.to_string(),
            offset: offset,
            width: width,
            access: acc,
            reset: reset.clone(),
            enums: IndexMap::new(),
        };
        self.fields.insert(name.to_string(), f);
        Ok(&mut self.fields[name])
    }
}

//def _state_desc(bits)
//state = []
//unless bits.readable? && bits.writable?
//  if bits.readable?
//    state << 'RO'
//  else
//    state << 'WO'
//  end
//end
//state << 'Rd' if bits.is_to_be_read?
//state << 'Str' if bits.is_to_be_stored?
//state << 'Ov' if bits.has_overlay?
//if state.empty?
//  ''
//else
//  "(#{state.join('|')})"
//end
//end

fn max_bit_in_range(field: &SummaryField, max: usize, _min: usize, bit_order: &BitOrder) -> usize {
    let upper = field.offset + field.width - 1;
    if *bit_order == BitOrder::MSB0 {
        field.width - (cmp::min(upper, max) - field.offset) - 1
    } else {
        cmp::min(upper, max) - field.offset
    }
}

fn min_bit_in_range(field: &SummaryField, _max: usize, min: usize, bit_order: &BitOrder) -> usize {
    let lower = field.offset;
    if *bit_order == BitOrder::MSB0 {
        field.width - (cmp::max(lower, min) - lower) - 1
    } else {
        cmp::max(lower, min) - field.offset
    }
}

/// Returns true if some portion of the given bit Field falls within the given range
fn is_field_in_range(field: &SummaryField, max: usize, min: usize) -> bool {
    let upper = field.offset + field.width - 1;
    let lower = field.offset;
    !((lower > max) || (upper < min))
}

//# Returns the number of bits from the given field that fall within the given range
fn num_bits_in_range(field: &SummaryField, max: usize, min: usize) -> usize {
    let upper = field.offset + field.width - 1;
    let lower = field.offset;
    cmp::min(upper, max) - cmp::max(lower, min) + 1
}

/// Returns true if the given index number is in the given range
fn is_index_in_range(i: usize, max: usize, min: usize) -> bool {
    !((i > max) || (i < min))
}

//def _bit_rw(bits)
//if bits.readable? && bits.writable?
//  'RW'
//elsif bits.readable?
//  'RO'
//elsif bits.writable?
//  'WO'
//else
//  'X'
//end
//end

#[derive(Debug)]
/// Named collections of bits within a register
pub struct Field {
    pub name: String,
    pub description: String,
    /// Offset from the start of the register in bits.
    pub offset: usize,
    /// Width of the field in bits.
    pub width: usize,
    pub access: AccessType,
    // Contains any reset values defined for this field.
    //pub resets: Vec<Reset>,
    // Just went with a simple reset value initially
    pub reset: BigUint,
    pub enums: IndexMap<String, EnumeratedValue>,
}

impl Field {
    pub fn add_enum(
        &mut self,
        name: &str,
        description: &str,
        value: &BigUint,
    ) -> OrigenResult<&EnumeratedValue> {
        //let acc: AccessType = match access.parse() {
        //    Ok(x) => x,
        //    Err(msg) => return Err(Error::new(&msg)),
        //};
        let e = EnumeratedValue {
            name: name.to_string(),
            description: description.to_string(),
            value: value.clone(),
        };
        self.enums.insert(name.to_string(), e);
        Ok(&self.enums[name])
    }
}

#[derive(Debug)]
/// A lightweight version of a Field that is returned by the my_reg.named_bits iterator,
/// and which is also used to represent gaps in the register (when spacer = true).
pub struct SummaryField {
    pub reg_id: usize,
    pub name: String,
    pub offset: usize,
    /// Width of the field in bits.
    pub width: usize,
    pub spacer: bool,
}

impl SummaryField {
    pub fn to_bit_collection<'a>(&self, dut: &'a MutexGuard<Dut>) -> BitCollection<'a> {
        let mut bits: Vec<usize> = Vec::new();
        let reg = dut.get_register(self.reg_id).unwrap();

        for _i in 0..self.width {
            bits.push(reg.bits[self.offset]);
        }

        BitCollection::for_bit_ids(bits, dut)
    }
}

//#[derive(Debug)]
//pub struct Reset {
//    pub reset_type: String,
//    // TODO: Should this be vector of tuples instead?
//    /// The size of this vector corresponds to the size of the parent field.
//    /// A set bit indicates a reset values of 1.
//    pub value: Vec<bool>,
//    /// The size of this vector corresponds to the size of the parent field.
//    /// A set bit indicates that the bit has a reset value defined by the
//    /// corresponding value.
//    pub mask: Vec<bool>,
//}

#[derive(Debug)]
pub struct EnumeratedValue {
    pub name: String,
    pub description: String,
    //pub usage: Usage,
    pub value: BigUint,
}
