#[macro_export]
macro_rules! get_reg {
    ( $dut:expr, $ab_id:expr, $reg_name:expr ) => {{
        let ab = $dut.get_address_block($ab_id)?;
        let r_id = ab.get_register_id($reg_name)?;
        $dut.get_register(r_id)?
    }};
}

#[macro_export]
macro_rules! get_bc_for {
    ( $dut:expr, $ab_id:expr, $reg_name:expr, $field_name:expr ) => {{
        let ab = $dut.get_address_block($ab_id)?;
        let r_id = ab.get_register_id($reg_name)?;
        let reg = $dut.get_register(r_id)?;
        match reg.fields.get($field_name) {
            Some(f) => Ok(f.bits($dut)),
            None => Err(crate::Error::new(&format!(
                "Could not find bitfield '{}' in register '{}' (address block: '{}')",
                $field_name,
                $reg_name,
                ab.name
            )))
        }
    }};
    ( $dut:expr, $reg:expr, $field_name:expr ) => {{
        match $reg.fields.get($field_name) {
            Some(f) => Ok(f.bits($dut)),
            None => Err(crate::Error::new(&format!(
                "Could not find bitfield '{}' in register '{}'",
                $field_name,
                $reg.name,
            )))
        }
    }}
}

#[macro_export]
macro_rules! get_reg_as_bc {
    ( $dut:expr, $ab_id:expr, $reg_name:expr ) => {{
        let ab = $dut.get_address_block($ab_id)?;
        let r_id = ab.get_register_id($reg_name)?;
        let reg = $dut.get_register(r_id)?;
        reg.bits($dut)
    }};
}

#[macro_export]
macro_rules! add_reg_32bit {
    ( $dut:expr, $address_block_id:expr, $name:expr, $offset:expr, $access:expr, $resets:expr, $fields:expr, $description:expr ) => {{
        crate::core::model::registers::register::Register::add_reg(
            $dut,
            $address_block_id,
            None,
            $name,
            $offset,
            Some(32),
            "LSB0",
            None,
            None,
            Some($description.to_string()),
            $access,
            $resets,
            $fields
        )?
    }};
}

#[macro_export]
macro_rules! add_reg {
    ( $dut:expr, $address_block_id:expr, $name:expr, $offset:expr, $size:expr, $access:expr, $resets:expr, $fields:expr, $description:expr ) => {{
        crate::core::model::registers::register::Register::add_reg(
            $dut,
            $address_block_id,
            None,
            $name,
            $offset,
            Some($size),
            "LSB0",
            None,
            None,
            Some($description.to_string()),
            $access,
            $resets,
            $fields
        )?
    }};
}

#[macro_export]
macro_rules! field {
    ( $name:expr, $offset:expr, $width:expr, $access:expr, $enums:expr, $resets:expr, $description:expr ) => {{
        crate::core::model::registers::register::FieldContainer::internal_new($name, $offset, $width, $access, $enums, $resets, $description)
    }};
}

#[macro_export]
macro_rules! some_hard_reset_val {
    ( $val:expr ) => {{
        Some(vec!(crate::core::model::registers::register::ResetVal::new("hard", $val, None)))
    }};
}