use crate::constants::*;
use clap::{ArgMatches};
use crate::root::Root;

pub fn build<'a>(matches: &ArgMatches<'a>) -> MyResult<()> {
    let root = Root::summon()?;

    println!("{:?}", root);

    Ok(())
}
