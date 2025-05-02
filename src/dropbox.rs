use futures::future::BoxFuture;
use futures::Future;
use log::{debug, error, info};
use regex::Regex;
use std::sync::Arc;
use std::{fs, io::Error as IOError, path::PathBuf};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::file::{directory_exists, DirError};

const DEFAULT_SLEEP_INTERVAL: u64 = 5000;

pub struct DropBoxDirs<'a> {
    pub target: &'a str,
    pub error: &'a str,
    pub processing: &'a str,
    pub processed: &'a str,
}

#[derive(Clone)]
pub struct DropBoxes {
    pub target: PathBuf,
    pub error: PathBuf,
    pub processing: PathBuf,
    pub processed: PathBuf,
}

pub enum DropBoxDir {
    Target,
    Error,
    Processing,
    Processed,
}

#[derive(Debug)]
pub enum DropBoxError {
    Path(DirError),
    Regex(regex::Error),
    IO(IOError),
    Misc(String),
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
    dropboxes: Arc<DropBoxes>,
    target_filter: Option<Regex>,
    handler: Handler<T>,
    data: Arc<T>,
}

impl<T: Clone + 'static> DropBox<T> {
    //    fn new<H, Fut>(handler: H, data: T) -> Self
    pub fn new<H, Fut>(
        dirs: DropBoxDirs,
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
        Ok(DropBox {
            dropboxes: DropBoxes {
                target: directory_exists(dirs.target)?,
                error: directory_exists(dirs.error)?,
                processing: directory_exists(dirs.processing)?,
                processed: directory_exists(dirs.processed)?,
            }
            .into(),
            target_filter: rx,
            handler: Box::new(move |a, b, c| Box::pin(handler(a, b, c))),
            data: data.into(),
        })
    }
    pub async fn list(&self, dirtype: DropBoxDir) -> Result<Vec<String>, DropBoxError> {
        let dir = match dirtype {
            DropBoxDir::Target => &self.dropboxes.target,
            DropBoxDir::Error => &self.dropboxes.error,
            DropBoxDir::Processing => &self.dropboxes.processing,
            DropBoxDir::Processed => &self.dropboxes.processed,
        };
        let mut files = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(osstr) = path.file_name() {
                    if let Some(str) = osstr.to_str() {
                        let mut dirstr = dir.to_str().unwrap().to_string();
                        dirstr.push('/');
                        dirstr.push_str(str);
                        if let Some(r) = &self.target_filter {
                            if r.is_match(str) {
                                files.push(dirstr);
                            }
                        } else {
                            files.push(dirstr);
                        }
                    }
                }
            }
        }
        Ok(files)
    }

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
            "Dropbox monitor sleeping {} milliseconds for every iteration",
            sleep_time.as_millis()
        );
        loop {
            if let Some(ref token) = token {
                if token.is_cancelled() {
                    info!("Quitting dropbox monitor");
                    break Ok(());
                }
            }
            let files = self.list(DropBoxDir::Target).await?;
            if !files.is_empty() {
                info!("Processing {} files", files.len());
                debug!("Files:\n{:#?}", files);
                match (self.handler)(self.dropboxes.clone(), files, self.data.clone()).await {
                    // need a way to break the loop for fatal errors!!
                    Ok(()) => {}
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            }
            sleep(sleep_time).await;
        }
    }
}
