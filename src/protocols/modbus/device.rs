use std::collections::BTreeMap;

use super::driver::{ModbusDriver, MBNetworks};

// ReadCoilds(ReadRequest),
// ReadDiscreteInputs(ReadRequest),
// ReadHoldingRegisters(ReadRequest),
// ReadInputRegisters(ReadRequest),
// WriteSingleCoil(WriteRequest),
// WriteSingleHoldingRegister(WriteRequest),
// WriteMultipleCoils(MultipleWriteRequest),
// WriteMultipleHoldingRegisters(MultipleWriteRequest),

enum RawValue {
    Bool(bool),
    Unit(u16),
    Int(u16),
    DUint([u16; 2]),
    DInt([u16; 2]),
    Real([u16; 2]),
}

enum PrepValue {
    Bool(bool),
    Unit(u16),
    Int(i16),
    DUint(u32),
    DInt(i32),
    Real(f32),
}

struct ValueParam {
    name: String,
    addr: u16,
    value: RawValue,
    little_endian: bool,
}

pub struct Device<T: 'static>
{
    driver: ModbusDriver<T>,
    coils: Vec<ValueParam>,
    discrete_inputs: Vec<ValueParam>,
    input_registers: Vec<ValueParam>,
    holding_registers: Vec<ValueParam>,
    prep_values: BTreeMap<String, PrepValue>,
}


    fn get_map(mut x: Vec<ValueParam>) -> Vec<u16>
    {
        x.sort_by_key(|a| a.addr);
        x.iter().map(|i| i.addr).collect::<Vec<_>>()
    }

#[cfg(test)]
mod tests {
    use super::get_map;
    use super::ValueParam;
    use super::RawValue;
    #[test]
    fn sort_val() {
        let mut ar = vec![
            ValueParam{
                addr: 56,
                name: "vasia".to_string(),
                little_endian: false,
                value: RawValue::Bool(true),
            },
            ValueParam{
                addr: 12,
                name: "Igar".to_string(),
                little_endian: false,
                value: RawValue::Bool(true),
            },
            ValueParam{
                addr: 999,
                name: "Kristia".to_string(),
                little_endian: false,
                value: RawValue::Bool(true),
            },
        ];

        let res = vec![12_u16, 56, 999];

        assert_eq!(get_map(ar), res);
    }
}
