use crate::config::AUTH_KEY;

use base64::{decode, encode};
use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

#[allow(non_snake_case, non_camel_case_types)]
pub fn get_serial_number() -> Result<bool, Box<dyn std::error::Error>> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    #[derive(Deserialize, Debug)]
    struct Win32_OperatingSystem {
        SerialNumber: String,
    }

    let results: Vec<Win32_OperatingSystem> = wmi_con.query()?;

    for os in results {
        let encrypted = enc_string(os.SerialNumber);
        if &encrypted == AUTH_KEY {
            return Ok(true)
        } else {
            println!("Your key {} is invalid", encrypted);
        }
    }

    Ok(false)
}

fn enc_string(input: String) -> String {
    let mut return_str = input;

    for _i in 0..2 {
        return_str = encode(
            return_str
                .chars()
                .filter(|c| *c != '-')
                .map(|c| std::char::from_u32(c as u32 + 10).unwrap_or(c))
                .collect::<String>(),
        );
    }

    return_str
}
