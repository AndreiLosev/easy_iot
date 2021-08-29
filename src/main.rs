mod RequestTypes;

use std::str::FromStr;
use std::{io, time::Duration, net};
use std::io::prelude::*;
use rmodbus::{
    client::ModbusRequest,
    guess_response_frame_len,
    ModbusProto,
};

use RequestTypes::{ReadRequest, MRequestValues, WriteRequest, MultipleWriteRequest};

struct ModbusDriver {
    transport: net::TcpStream,
    m_request: ModbusRequest,
}

enum ModbusFunc {
    ReadCoilds(ReadRequest),
    ReadDiscreteInputs(ReadRequest),
    ReadHoldingRegisters(ReadRequest),
    ReadInputRegisters(ReadRequest),
    WriteSingleCoil(WriteRequest),
    WriteSingleHoldingRegister(WriteRequest),
    WriteMultipleCoils(MultipleWriteRequest),
    WriteMultipleHoldingRegisters(MultipleWriteRequest),
}

#[derive(Debug)]
enum ModbusErrKind {
    Io(io::Error),
    Net(net::AddrParseError),
    Rmodbus(rmodbus::ErrorKind),
}

impl From<io::Error> for ModbusErrKind {
    fn from(err: io::Error) -> ModbusErrKind {
        ModbusErrKind::Io(err)
    }
}

impl From<net::AddrParseError> for ModbusErrKind {
    fn from(err: net::AddrParseError) -> ModbusErrKind {
        ModbusErrKind::Net(err)
    }
}

impl From<rmodbus::ErrorKind> for ModbusErrKind {
    fn from(err: rmodbus::ErrorKind) -> ModbusErrKind {
        ModbusErrKind::Rmodbus(err)
    }
}

impl ModbusDriver {
    fn new(host: &str, port: u16, timeout: u64) -> Result<Self, ModbusErrKind> {

        let soket_addr =  net::SocketAddr::from((
            net::IpAddr::from_str(host)?,
            port,
        ));
        let transport = net::TcpStream::connect(soket_addr)?;
        transport.set_read_timeout(Some(Duration::from_millis(timeout)))?;
        transport.set_write_timeout(Some(Duration::from_millis(timeout)))?;
        let m_request = ModbusRequest::new(1, ModbusProto::TcpUdp);
        return Ok(Self { transport, m_request });
    }

    fn read_coils(&mut self, req_data: ReadRequest) -> Result<Vec<bool>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadCoilds(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_bool(&response, &mut result)?;
        Ok(result)
    }

    fn read_discrete_inputs(&mut self, req_data: ReadRequest) -> Result<Vec<bool>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadDiscreteInputs(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_bool(&response, &mut result)?;
        Ok(result)
    }

    fn read_holed_registers(&mut self, req_data: ReadRequest) -> Result<Vec<u16>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadHoldingRegisters(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_u16(&response, &mut result)?;
        Ok(result)
    }

    fn read_inputs_registers(&mut self, req_data: ReadRequest) -> Result<Vec<u16>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadInputRegisters(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_u16(&response, &mut result)?;
        Ok(result)
    }

    fn write_single_coil(&mut self, req_data: WriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteSingleCoil(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    fn write_single_holding_register(&mut self, req_data: WriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteSingleHoldingRegister(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    fn write_multiple_coils(&mut self, req_data: MultipleWriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteMultipleCoils(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    fn write_multiple_holding_register(&mut self, req_data: MultipleWriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteMultipleHoldingRegisters(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    fn generate_request(&mut self, func_type: &mut ModbusFunc) -> Result<Vec<u8>, rmodbus::ErrorKind> {
        match func_type {
            ModbusFunc::ReadCoilds(r) => {
                self.m_request.generate_get_coils(r.start, r.count, &mut r.buffer)?;
                Ok(r.buffer.clone())
            },
            ModbusFunc::ReadDiscreteInputs(r) => {
                self.m_request.generate_get_discretes(r.start, r.count, &mut r.buffer)?;
                Ok(r.buffer.clone())
            }
            ModbusFunc::ReadHoldingRegisters(r) => {
                self.m_request.generate_get_holdings(r.start, r.count, &mut r.buffer)?;
                Ok(r.buffer.clone())
            },
            ModbusFunc::ReadInputRegisters(r) => {
                self.m_request.generate_get_inputs(r.start, r.count, &mut r.buffer)?;
                Ok(r.buffer.clone())
            },
            ModbusFunc::WriteSingleCoil(r) => {
                let value = r.value.get_bool()?;
                self.m_request.generate_set_coil(r.start, value, &mut r.buffer)?;
                Ok(r.buffer.clone())
            }
            ModbusFunc::WriteSingleHoldingRegister(r) => {
                let value = r.value.get_register()?;
                self.m_request.generate_set_holding(r.start, value, &mut r.buffer)?;
                Ok(r.buffer.clone())
            }
            ModbusFunc::WriteMultipleCoils(r) => {
                let values = r.values.iter().map(|i| i.get_bool()).collect::<Result<Vec<_>,_>>()?;
                self.m_request.generate_set_coils_bulk(r.start, &values, &mut r.buffer)?;
                Ok(r.buffer.clone())
            }
            ModbusFunc::WriteMultipleHoldingRegisters(r) => {
                let values = r.values.iter().map(|i| i.get_register()).collect::<Result<Vec<_>, _>>()?;
                self.m_request.generate_set_holdings_bulk(r.start, &values, &mut r.buffer)?;
                Ok(r.buffer.clone())
            },
        }
    }

    fn networking(&mut self, func_type: &mut ModbusFunc) -> Result<Vec<u8>, ModbusErrKind> {
        let buff = self.generate_request(func_type)?;
        self.transport.write(&buff)?;
        let mut buf = [0u8; 7];
        self.transport.read_exact(&mut buf)?;
        let mut response: Vec<u8> = Vec::new();
        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, ModbusProto::TcpUdp)?;
        if len > 7 {
            let mut rest = vec![0u8; (len - 7) as usize];
            self.transport.read_exact(&mut rest)?;
            response.extend(rest);
        };
        Ok(response)
    }
}

fn main() {

    let mut x = ModbusDriver::new("127.0.0.1", 5020, 1000).unwrap();

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

