use tracing_log::{log::LevelFilter, AsTrace};

use crate::Error;

pub fn attach_logger(max_level: LevelFilter) -> Result<(), Error> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(max_level.as_trace())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    ::tracing::info!("tracing initialized");

    Ok(())
}

// use std::{
//     fs::{self, File},
//     path::{Path, PathBuf},
// };
// use tracing_subscriber::util::SubscriberInitExt;

// LogLevel,
// RotationStrategy, TimezoneStrategy
// pub fn _get_log_file_path(
//     dir: &impl AsRef<Path>,
//     file_name: &str,
//     rotation_strategy: &RotationStrategy,
//     timezone_strategy: &TimezoneStrategy,
//     max_file_size: u128,
// ) -> Result<PathBuf, Error> {
//     let path = dir.as_ref().join(format!("{file_name}.log"));

//     if path.exists() {
//         let log_size = File::open(&path)?.metadata()?.len() as u128;
//         if log_size > max_file_size {
//             match rotation_strategy {
//                 RotationStrategy::KeepAll => {
//                     let to = dir.as_ref().join(format!(
//                         "{}_{}.log",
//                         file_name,
//                         timezone_strategy
//                             .get_now()
//                             .format(&time::format_description::parse(
//                                 "[year]-[month]-[day]_[hour]-[minute]-[second]"
//                             )?)?,
//                     ));
//                     if to.is_file() {
//                         // designated rotated log file name already exists
//                         // highly unlikely but defensively handle anyway by adding .bak to filename
//                         let mut to_bak = to.clone();
//                         to_bak.set_file_name(format!(
//                             "{}.bak",
//                             to_bak
//                                 .file_name()
//                                 .map(|f| f.to_string_lossy())
//                                 .unwrap_or_default()
//                         ));
//                         fs::rename(&to, to_bak)?;
//                     }
//                     fs::rename(&path, to)?;
//                 }
//                 RotationStrategy::KeepOne => {
//                     fs::remove_file(&path)?;
//                 }
//             }
//         }
//     }

//     Ok(path)
// }
