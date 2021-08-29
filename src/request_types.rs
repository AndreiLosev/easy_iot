#[derive(Clone)]
pub struct ReadRequest {
    pub start: u16,
    pub count: u16,
    pub buffer: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum MRequestValues {
    Coil(bool),
    Register(u16),
}

impl MRequestValues {
    pub fn get_bool(self) -> Result<bool, rmodbus::ErrorKind> {
        match self {
            Self::Coil(v) => Ok(v),
            _ => Err(rmodbus::ErrorKind::IllegalDataValue),
        }
    }

    pub fn get_register(self) -> Result<u16, rmodbus::ErrorKind> {
        match self {
            Self::Register(v) => Ok(v),
            _ => Err(rmodbus::ErrorKind::IllegalDataValue),
        }
    }
}

#[derive(Clone)]
pub struct WriteRequest {
    pub start: u16,
    pub value: MRequestValues,
    pub buffer: Vec<u8>,
}

pub struct  MultipleWriteRequest {
    pub start: u16,
    pub values: Vec<MRequestValues>,
    pub buffer: Vec<u8>,
}