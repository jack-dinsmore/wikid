use crate::constants::*;
use clap::{ArgMatches};
use crate::root::Root;
use std::str::FromStr;

pub fn build<'a>(matches: &ArgMatches<'a>) -> MyResult<()> {
    let root = Root::summon()?;

    if matches.is_present("public") {
        println!("Public build");
        if root.public_url == "" {
            return Err("You must first set the public url");
        }
    }

    println!("{:?}", root);

    Ok(())
}
