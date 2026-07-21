mod cli;

use std::io;

use anyhow::Context;
use byte_unit::Byte;
use cli::*;
use pipe_logger_lib::{CompressionMethod, PipeLoggerBuilder, RotateMethod, Tee};

fn main() -> anyhow::Result<()> {
    let args = get_args();

    let mut builder = PipeLoggerBuilder::new(args.log_path);

    if let Some(r) = args.rotate {
        let byte = Byte::parse_str(&r, true)
            .with_context(|| format!("failed to parse the rotation size {r:?}"))?;

        builder.set_rotate(Some(RotateMethod::FileSize(byte.as_u64())));
        builder.set_compression(args.compress.then_some(CompressionMethod::Xz(9)));
        builder.set_count(args.count);
    }

    if args.quiet {
        builder.set_tee(None);
    } else if args.err {
        builder.set_tee(Some(Tee::Stderr));
    } else {
        builder.set_tee(Some(Tee::Stdout));
    }

    let mut logger = builder.build().context("failed to build the logger")?;
    let mut stdin = io::stdin().lock();

    io::copy(&mut stdin, &mut logger).context("failed to copy stdin to log outputs")?;

    logger.finish().context("failed to finish logger")?;

    Ok(())
}
