use std::num::ParseIntError;

use radix_fmt::radix_36;

pub fn as_id<T: AsRef<str>>(chars: T) -> Result<u64, ParseIntError> {
    u64::from_str_radix(&chars.as_ref().to_string().to_uppercase(), 36)
}

pub fn as_str(id: u64) -> String {
    let ret = format!("{:0>2}", radix_36(id)).to_uppercase();
    if ret.len() == 1 {
        "0".to_owned() + &ret
    } else {
        ret
    }
}
