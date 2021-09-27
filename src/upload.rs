use ssh2::Session;
use std::io;
use std::io::prelude::*;
use std::path::Path;

const FILE_NAME: &str = "printfile";

pub fn upload_print_file(session: &mut Session, file: &[u8]) -> io::Result<String> {
    let mut remote_file = session.scp_send(Path::new(FILE_NAME), 0o644, file.len() as u64, None)?;
    remote_file.write_all(file)?;

    Ok(FILE_NAME.to_string())
}
