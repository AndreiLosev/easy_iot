mod protocols;

use std::net;
use protocols::modbus::driver::ModbusDriver;
use protocols::modbus::request_types::ReadRequest;
use protocols::modbus::request_types::WriteRequest;
use protocols::modbus::request_types::MultipleWriteRequest;
use protocols::modbus::request_types::MRequestValues;

fn main() {
    // let transport = serial::open("/dev/pts/3").unwrap();
    let transport = net::TcpStream::connect("127.0.0.1:5020").unwrap();
    let mut x = ModbusDriver::new(transport).unwrap();
    let qwe = ReadRequest{
        start: 3,
        count: 4,
        buffer: Vec::with_capacity(8),
    };
    let ewq = WriteRequest{
        start: 3,
        value: MRequestValues::Coil(true),
        buffer: Vec::with_capacity(8),
    };

    let qwe1r = WriteRequest{
        start: 3,
        value: MRequestValues::Register(15),
        buffer: Vec::with_capacity(8),
    };

    let mewq = MultipleWriteRequest{
        start: 4,
        values: vec![
            MRequestValues::Coil(true),
            MRequestValues::Coil(false),
            MRequestValues::Coil(true),
        ],
        buffer: Vec::with_capacity(8),
    };

    let wqwe1r = MultipleWriteRequest{
        start: 4,
        values: vec![
            MRequestValues::Register(150),
            MRequestValues::Register(250),
            MRequestValues::Register(2150),
        ],
        buffer: Vec::with_capacity(8),
    };

    dbg!(x.read_coils(qwe.clone()).unwrap());
    dbg!(x.read_discrete_inputs(qwe.clone())).unwrap();
    dbg!(x.read_holed_registers(qwe.clone()).unwrap());
    dbg!(x.read_inputs_registers(qwe.clone()).unwrap());
    dbg!(x.write_single_coil(ewq.clone()).unwrap());
    dbg!(x.write_single_holding_register(qwe1r.clone()).unwrap());
    dbg!(x.write_multiple_coils(mewq).unwrap());
    dbg!(x.write_multiple_holding_register(wqwe1r).unwrap())

}

