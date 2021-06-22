use peudo_write_ret::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let origin = OriginImpl{};
    match xyz_should_match::<ConfigImpl>(origin) {
        Ok(_) => Ok(()),
        Err(_) => {
           let err: Box<dyn Error + Send + Sync> = From::from("error");
           Err(err as Box<dyn Error>)
        },
    }
}
