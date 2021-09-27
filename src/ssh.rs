use crate::error::Error;
use ssh2::Session;
use std::net::TcpStream;

pub const DOMAINS: &[&str] = &["remote11.chalmers.se:22", "remote12.chalmers.se:22"];

pub fn ask_for_user() -> Result<String, Error> {
    let mut rl = rustyline::Editor::<()>::new();
    match rl.readline("username: ") {
        Ok(line) => Ok(line),
        Err(_) => return Err(Error::user("no username specified")),
    }
}

pub fn ask_for_pass() -> Result<String, Error> {
    match rpassword::read_password_from_tty(Some("password: ")) {
        Ok(pass) => Ok(pass),
        Err(_) => return Err(Error::user("no password specified")),
    }
}

pub fn connect(remote: &str) -> Result<Session, Error> {
    // Connect to the local SSH server
    let tcp = TcpStream::connect(remote)?;
    let mut session = Session::new()?;
    session.set_timeout(5000);
    session.set_tcp_stream(tcp);
    session.handshake()?;

    let user = ask_for_user()?;
    let pass = ask_for_pass()?;

    session.userauth_password(&user, &pass)?;
    assert!(session.authenticated());

    Ok(session)
}
