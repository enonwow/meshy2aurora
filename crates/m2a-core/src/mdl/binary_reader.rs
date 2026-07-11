use super::errors::ParseError;

pub(crate) struct BinaryReader<'a> {
    bytes: &'a [u8],
}

impl<'a> BinaryReader<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub(crate) fn read_u8(&self, offset: usize, context: &str) -> Result<u8, ParseError> {
        Ok(self.read_slice(offset, 1, context)?[0])
    }

    pub(crate) fn read_i8(&self, offset: usize, context: &str) -> Result<i8, ParseError> {
        Ok(self.read_u8(offset, context)? as i8)
    }

    pub(crate) fn read_u16(&self, offset: usize, context: &str) -> Result<u16, ParseError> {
        let bytes = self.read_slice(offset, 2, context)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn read_i16(&self, offset: usize, context: &str) -> Result<i16, ParseError> {
        let bytes = self.read_slice(offset, 2, context)?;
        Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn read_u32(&self, offset: usize, context: &str) -> Result<u32, ParseError> {
        let bytes = self.read_slice(offset, 4, context)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_i32(&self, offset: usize, context: &str) -> Result<i32, ParseError> {
        let bytes = self.read_slice(offset, 4, context)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_f32(&self, offset: usize, context: &str) -> Result<f32, ParseError> {
        let bytes = self.read_slice(offset, 4, context)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_fixed_string(
        &self,
        offset: usize,
        length: usize,
        context: &str,
    ) -> Result<String, ParseError> {
        let bytes = self.read_slice(offset, length, context)?;
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
    }

    pub(crate) fn read_slice(
        &self,
        offset: usize,
        length: usize,
        context: &str,
    ) -> Result<&'a [u8], ParseError> {
        let range = checked_range(offset, length, context)?;
        self.bytes
            .get(range)
            .ok_or_else(|| ParseError::pointer(offset, format!("{context} is outside input")))
    }
}

pub(crate) fn checked_range(
    offset: usize,
    length: usize,
    context: &str,
) -> Result<std::ops::Range<usize>, ParseError> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| ParseError::pointer(offset, format!("{context} range overflow")))?;
    Ok(offset..end)
}

pub(crate) fn checked_array_size(
    count: usize,
    element_size: usize,
    offset: usize,
    context: &str,
) -> Result<usize, ParseError> {
    count
        .checked_mul(element_size)
        .ok_or_else(|| ParseError::pointer(offset, format!("{context} size overflow")))
}

#[cfg(test)]
mod tests {
    use super::{checked_array_size, checked_range};
    use crate::mdl::errors::POINTER_OOB;

    #[test]
    fn offset_plus_size_overflow_is_rejected() {
        let error = checked_range(usize::MAX, 1, "overflow").unwrap_err();
        assert_eq!(error.code, POINTER_OOB);
    }

    #[test]
    fn count_times_element_size_overflow_is_rejected() {
        let error = checked_array_size(usize::MAX, 2, 7, "array").unwrap_err();
        assert_eq!(error.code, POINTER_OOB);
        assert_eq!(error.offset, 7);
    }
}
