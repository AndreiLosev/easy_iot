use rmodbus::ErrorKind;
use serial;
use std::net;
use std::io::{Read, Write};
use rmodbus::{
    client::ModbusRequest,
    guess_response_frame_len,
    ModbusProto,
};
use std::any;

use super::request_types::{ReadRequest, WriteRequest, MultipleWriteRequest};
use super::error_kind::ModbusErrKind;

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

pub trait MBNetworks: Write + Read {}

impl MBNetworks for net::TcpStream {}

impl MBNetworks for serial::SystemPort {}


pub struct ModbusDriver<T: 'static> {
    transport: T,
    m_request: ModbusRequest,
    proto: ModbusProto,
}

impl<T: MBNetworks> ModbusDriver<T> {
    pub fn new(transport: T) -> Result<Self, ModbusErrKind> {

        let proto = if any::TypeId::of::<T>() == any::TypeId::of::<net::TcpStream>() {
            Ok(ModbusProto::TcpUdp)
        } else if any::TypeId::of::<T>() == any::TypeId::of::<serial::SystemPort>() {
            Ok(ModbusProto::Rtu)
        } else {
            Err(ModbusErrKind::Rmodbus(ErrorKind::Acknowledge)) // TODO
        }?;

        let m_request = ModbusRequest::new(1, proto);
        return Ok(Self {transport, m_request, proto});
    }

    pub fn read_coils(&mut self, req_data: ReadRequest) -> Result<Vec<bool>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadCoilds(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_bool(&response, &mut result)?;
        Ok(result)
    }

    pub fn read_discrete_inputs(&mut self, req_data: ReadRequest) -> Result<Vec<bool>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadDiscreteInputs(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_bool(&response, &mut result)?;
        Ok(result)
    }

    pub fn read_holed_registers(&mut self, req_data: ReadRequest) -> Result<Vec<u16>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadHoldingRegisters(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_u16(&response, &mut result)?;
        Ok(result)
    }

    pub fn read_inputs_registers(&mut self, req_data: ReadRequest) -> Result<Vec<u16>, ModbusErrKind> {
        let mut request = ModbusFunc::ReadInputRegisters(req_data);
        let response = self.networking(&mut request)?;
        let mut result = Vec::with_capacity(8);
        self.m_request.parse_u16(&response, &mut result)?;
        Ok(result)
    }

    pub fn write_single_coil(&mut self, req_data: WriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteSingleCoil(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    pub fn write_single_holding_register(&mut self, req_data: WriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteSingleHoldingRegister(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    pub fn write_multiple_coils(&mut self, req_data: MultipleWriteRequest) -> Result<(), ModbusErrKind> {
        let mut request = ModbusFunc::WriteMultipleCoils(req_data);
        let response = self.networking(&mut request)?;
        let result= self.m_request.parse_ok(&response)?;
        Ok(result)
    }

    pub fn write_multiple_holding_register(&mut self, req_data: MultipleWriteRequest) -> Result<(), ModbusErrKind> {
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
        let mut buf = [0u8; 6];
        self.transport.read_exact(&mut buf)?;
        let mut response: Vec<u8> = Vec::new();
        response.extend_from_slice(&buf);
        let len = guess_response_frame_len(&buf, self.proto)?;
        if len > 6 {
            let mut rest = vec![0u8; (len - 6) as usize];
            self.transport.read_exact(&mut rest)?;
            response.extend(rest);
        };
        Ok(response)
    }
}
