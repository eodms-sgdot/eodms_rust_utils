use futures::future::BoxFuture;
use futures::Future;
use log::{debug, error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Error as IOError,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::file::{create_dest_path, directory_exists, DirError};

const DEFAULT_SLEEP_INTERVAL: u64 = 5000;

pub struct DropBoxDirs<'a> {
    pub target: &'a str,
    pub error: &'a str,
    pub processing: &'a str,
    pub processed: &'a str,
    pub other: Option<HashMap<&'a str, &'a str>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FilePaths {
    pub filename: String,
    pub error: PathBuf,
    pub processing: PathBuf,
    pub processed: PathBuf,
}

#[derive(Clone)]
pub struct DropBoxes {
    pub target: PathBuf,
    pub error: PathBuf,
    pub processing: PathBuf,
    pub processed: PathBuf,
    pub other: Option<HashMap<String, PathBuf>>,
}

impl DropBoxes {
    /// # Errors
    ///
    /// Can return an error if there is a path to string conversion issue or from
    /// `check_dest` or `move_to_error`
    pub fn generate_filepaths(&self, file: &Path) -> Result<FilePaths, DropBoxError> {
        let Some(basename) = file.file_name() else {
            return Err(DropBoxError::Misc("basename from filename".to_string()));
        };
        let binding = basename.to_owned();
        let Some(filename) = binding.to_str() else {
            return Err(DropBoxError::Misc("Converting OsStr to &str".to_string()));
        };
        let Ok(error) = create_dest_path(&self.error, file) else {
            return Err(DropBoxError::Misc("Creating error path".to_string()));
        };
        let Ok(processing) = create_dest_path(&self.processing, Path::new(&file)) else {
            move_to_error(file, &error)?;
            return Err(DropBoxError::Misc("Creating processing path".to_string()));
        };
        let Ok(processed) = create_dest_path(&self.processed, &processing) else {
            return Err(DropBoxError::Misc("Creating processed path".to_string()));
        };
        check_dest(&processed, &error)?;
        Ok(FilePaths {
            filename: filename.to_string(),
            error,
            processing,
            processed,
        })
    }
}

#[non_exhaustive]
pub enum DropBoxDir {
    Target,
    Error,
    Processing,
    Processed,
    Other(PathBuf),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DropBoxError {
    Path(DirError),
    Regex(regex::Error),
    IO(IOError),
    Misc(String),
    Time(SystemTimeError),
}

impl From<regex::Error> for DropBoxError {
    fn from(e: regex::Error) -> DropBoxError {
        DropBoxError::Regex(e)
    }
}

impl From<DirError> for DropBoxError {
    fn from(e: DirError) -> DropBoxError {
        DropBoxError::Path(e)
    }
}

impl From<IOError> for DropBoxError {
    fn from(e: IOError) -> DropBoxError {
        DropBoxError::IO(e)
    }
}

type DropBoxResult = Result<(), DropBoxError>;
type Handler<T> = Box<
    dyn Fn(Arc<DropBoxes>, Vec<String>, Arc<T>) -> BoxFuture<'static, DropBoxResult> + Send + Sync,
>;

pub struct DropBox<T> {
    name: String,
    dropboxes: Arc<DropBoxes>,
    target_filter: Option<Regex>,
    handler: Handler<T>,
    data: Arc<T>,
}

impl<T: Clone + 'static> DropBox<T> {
    /// # Errors
    ///
    /// Can return an error if the given regex is bad or if a dropbox directory does not exist
    pub fn new<H, Fut>(
        name: String,
        dirs: &DropBoxDirs,
        rxstr: Option<String>,
        handler: H,
        data: T,
    ) -> Result<DropBox<T>, DropBoxError>
    where
        H: Fn(Arc<DropBoxes>, Vec<String>, Arc<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = DropBoxResult> + Send + 'static,
    {
        let rx = match rxstr {
            Some(s) => Some(Regex::new(&s)?),
            None => None,
        };
        let other: Result<Option<HashMap<String, PathBuf>>, DropBoxError> = dirs
            .other
            .as_ref()
            .map(|hm| {
                hm.iter()
                    .map(|(k, v)| {
                        let pathbuf = directory_exists(v)?;
                        Ok(((*k).to_string(), pathbuf))
                    })
                    .collect::<Result<HashMap<_, _>, _>>()
            })
            .transpose();
        Ok(DropBox {
            name,
            dropboxes: DropBoxes {
                target: directory_exists(dirs.target)?,
                error: directory_exists(dirs.error)?,
                processing: directory_exists(dirs.processing)?,
                processed: directory_exists(dirs.processed)?,
                other: other?,
            }
            .into(),
            target_filter: rx,
            handler: Box::new(move |a, b, c| Box::pin(handler(a, b, c))),
            data: data.into(),
        })
    }

    /// # Errors
    ///
    /// Can return an error on various I/O issues
    pub fn list(&self, dirtype: &DropBoxDir) -> Result<Vec<String>, DropBoxError> {
        let dir = match dirtype {
            DropBoxDir::Target => &self.dropboxes.target,
            DropBoxDir::Error => &self.dropboxes.error,
            DropBoxDir::Processing => &self.dropboxes.processing,
            DropBoxDir::Processed => &self.dropboxes.processed,
            DropBoxDir::Other(p) => p,
        };
        let mut files = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(osstr) = path.file_name() {
                    if let Some(str) = osstr.to_str() {
                        let Some(dirstr) = dir.to_str() else {
                            return Err(DropBoxError::Misc(
                                "Converting PathBuf to &str".to_string(),
                            ));
                        };
                        let mut directory = dirstr.to_string();
                        directory.push('/');
                        directory.push_str(str);
                        if let Some(r) = &self.target_filter {
                            if r.is_match(str) {
                                files.push(directory);
                            }
                        } else {
                            files.push(directory);
                        }
                    }
                }
            }
        }
        Ok(files)
    }

    /// # Errors
    ///
    /// Can return an error if the target directory listing fails
    pub async fn monitor(
        &self,
        sleep_interval: Option<u64>,
        token: Option<CancellationToken>,
    ) -> Result<(), DropBoxError> {
        let sleep_time = if let Some(s) = sleep_interval {
            Duration::from_millis(s)
        } else {
            Duration::from_millis(DEFAULT_SLEEP_INTERVAL)
        };
        info!(
            "Dropbox monitor {} sleeping {} milliseconds for every iteration",
            self.name,
            sleep_time.as_millis()
        );
        loop {
            if let Some(ref token) = token {
                if token.is_cancelled() {
                    info!("Quitting dropbox monitor");
                    break Ok(());
                }
            }
            let files = self.list(&DropBoxDir::Target)?;
            if !files.is_empty() {
                info!("Processing {} files", files.len());
                debug!("Files:\n{files:#?}");
                match (self.handler)(self.dropboxes.clone(), files, self.data.clone()).await {
                    // need a way to break the loop for fatal errors!!
                    Ok(()) => {}
                    Err(e) => {
                        error!("{e:?}");
                    }
                }
            }
            sleep(sleep_time).await;
        }
    }
}

/// # Errors
///
/// Can return an error if `move_to_error` fails
pub fn check_dest(dstfile: &Path, errorfile: &Path) -> Result<(), DropBoxError> {
    if dstfile.exists() {
        move_to_error(dstfile, errorfile)?;
    }
    Ok(())
}

/// # Errors
///
/// Can return an error if a file rename fails
pub fn move_to_error(file: &Path, error: &Path) -> Result<(), DropBoxError> {
    let mut error = error.to_path_buf();
    if error.exists() {
        let mut tmp = error.as_mut_os_str().to_os_string();
        tmp.push(".");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(DropBoxError::Time)?
            .as_secs()
            .to_string();
        tmp.push(now);
        error.set_file_name(tmp);
    }
    match fs::rename(file, &error) {
        Ok(()) => {}
        Err(e) => {
            return Err(DropBoxError::IO(e));
        }
    }
    Ok(())
}
