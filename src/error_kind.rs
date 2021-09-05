use std::{io, net};

#[derive(Debug)]
pub enum ModbusErrKind {
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
