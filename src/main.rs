use std::str::FromStr;
use alloy::primitives::Address;
use coins_bip32::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let xpub_str = "xpub6EeaXhbbgvtV6KF1fvBeEn7DZnd1Gd4xh36eMAAeBB4KA73ZV5pXmjyddjPziE5QqkcoHtRRpkce9UP5qxsd2Q9qi3zmeXtEz5sc7NFGcvN";

    let xpub = XPub::from_str(xpub_str)
        .expect("Invalid Xpub string");

    for i in 0..100 {
        let child_xpub = xpub.derive_child(i)?;
        let verifying_key = child_xpub.as_ref();

        let address = Address::from_public_key(&verifying_key);

        println!("address /{i}: {:#?}", address);
    }

    Ok(())
}
