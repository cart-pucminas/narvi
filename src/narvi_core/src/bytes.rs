use std::array::TryFromSliceError;

pub trait ByteVecToPrimitive {
    fn to_u8(&self) -> Result<u8, TryFromSliceError>;
    fn to_u16(&self) -> Result<u16, TryFromSliceError>;
    fn to_u32(&self) -> Result<u32, TryFromSliceError>;
    fn to_u64(&self) -> Result<u64, TryFromSliceError>;
}

impl ByteVecToPrimitive for Vec<u8> {
    fn to_u8(&self) -> Result<u8, TryFromSliceError> {
        let arr: [u8; 1] = self.as_slice().try_into()?;
        Ok(u8::from_le_bytes(arr))
    }

    fn to_u16(&self) -> Result<u16, TryFromSliceError> {
        let arr: [u8; 2] = self.as_slice().try_into()?;
        Ok(u16::from_le_bytes(arr))
    }

    fn to_u32(&self) -> Result<u32, TryFromSliceError> {
        let arr: [u8; 4] = self.as_slice().try_into()?;
        Ok(u32::from_le_bytes(arr))
    }

    fn to_u64(&self) -> Result<u64, TryFromSliceError> {
        let arr: [u8; 8] = self.as_slice().try_into()?;
        Ok(u64::from_le_bytes(arr))
    }
}
