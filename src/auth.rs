use hex_literal::*;

use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};

type Aes256Cbc = Cbc<Aes128, Pkcs7>;

#[allow(non_snake_case, non_camel_case_types)]
pub fn get_serial_number() -> Result<(), Box<dyn std::error::Error>> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    #[derive(Deserialize, Debug)]
    struct Win32_OperatingSystem {
        SerialNumber: String,
    }

    let results: Vec<Win32_OperatingSystem> = wmi_con.query()?;

    for os in results {
        println!("{:#?}", os.SerialNumber);

        get_enc_key(&os.SerialNumber);
    }

    Ok(())
}

fn get_enc_key(encrypting: &str) -> String {
    let key = hex!("000102030405060708090a0b0c0d0e0f");
    let iv = hex!("f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0");
    let cipher = Aes256Cbc::new_var(&key, &iv).unwrap();
    let ciphertext = cipher.encrypt_vec(encrypting.as_bytes());

    println!("{:?}", ciphertext);

    "".to_string()
}
