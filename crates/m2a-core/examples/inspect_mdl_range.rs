use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use m2a_core::mdl::inspect_binary_mdl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let container = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("usage: inspect_mdl_range <container> <offset> <length>")?;
    let offset = arguments
        .next()
        .ok_or("missing offset")?
        .to_string_lossy()
        .parse::<u64>()?;
    let length = arguments
        .next()
        .ok_or("missing length")?
        .to_string_lossy()
        .parse::<usize>()?;
    if arguments.next().is_some() {
        return Err("unexpected extra argument".into());
    }

    let mut file = File::open(&container)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    let report = inspect_binary_mdl(&bytes)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
