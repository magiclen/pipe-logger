mod cli;

use std::{io, io::Write};

use anyhow::{anyhow, Context};
use byte_unit::Byte;
use cli::*;
use pipe_logger_lib::{PipeLoggerBuilder, RotateMethod, Tee};

fn main() -> anyhow::Result<()> {
    let args = get_args();

    let mut builder = PipeLoggerBuilder::new(args.log_path);

    if let Some(r) = args.rotate {
        let byte = Byte::from_str(r)?;

        builder.set_rotate(Some(RotateMethod::FileSize(byte.get_bytes())));
        builder.set_compress(args.compress);
        builder.set_count(args.count);
    }

    if args.err {
        builder.set_tee(Some(Tee::Stderr));
    } else {
        builder.set_tee(Some(Tee::Stdout));
    }

    let mut logger = builder.build()?;

    let mut input = String::new();

    let stdin = io::stdin();

    loop {
        let c = stdin.read_line(&mut input).with_context(|| anyhow!("stdin"))?;

        if c == 0 {
            break;
        }

        logger.write_all(&input.as_bytes()[..c]).unwrap();
    }

    Ok(())
}
