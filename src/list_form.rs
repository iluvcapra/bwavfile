use super::fourcc::{FourCC, ReadFourCC};
use byteorder::{ReadBytesExt, LittleEndian};
use std::io::{Cursor, Error, Read};

pub struct ListFormItem {
    pub signature : FourCC,
    pub contents : Vec<u8>
}

/// A helper that will accept a LIST chunk as a [u8]
/// and give you back each segment
/// 
pub fn collect_list_form(list_contents :& [u8]) -> Result<Vec<ListFormItem>, Error> {
    let mut cursor = Cursor::new(list_contents);
    let mut remain = list_contents.len();
    let _ = cursor.read_fourcc()?; // skip signature

    remain -= 4;
    let mut retval : Vec<ListFormItem> = vec![];

    while remain > 0 {
        let this_sig = cursor.read_fourcc()?;
        let this_size = cursor.read_u32::<LittleEndian>()? as usize;
        let mut content_buf = vec![0u8; this_size];

        cursor.read_exact(&mut content_buf)?;
        retval.push( ListFormItem { signature : this_sig, contents : content_buf } );
    }

    Ok( retval )
}