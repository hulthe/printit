use crate::Error;
use ssh2::Session;
use std::io::prelude::*;

pub fn print(session: &mut Session, print_command: &str) -> Result<(), Error> {
    let mut channel = session.channel_session()?;

    println!("{}", print_command);
    channel.exec(print_command)?;
    let mut cmd_output = String::new();
    channel.read_to_string(&mut cmd_output)?;
    //println!("{}", s);
    channel.wait_close()?;
    println!("{}", channel.exit_status()?);

    Ok(())
}
