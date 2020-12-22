use std::env;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::{DirEntry, metadata, read_dir, read_to_string};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, ExitStatus};
use std::string::FromUtf8Error;

use clap::Clap;
use thiserror::Error;

/// Load environment variables from DIR (or ~/.envdir).
///
/// For each non-directory entry in DIR, this will output a brief shell script
/// to set an environment variable with the same name. If the entry is an
/// executable program, it will be run (with the current environment) and its
/// output will be used as the value of the environment variable. Otherwise, it
/// will be read, and its content will be used. In either case, a single
/// trailing newline will be removed, if present.
///
/// This process skips any file which can't be run or read, as appropriate, and
/// outputs a warning on stderr.
///
/// The intended use case for this is in shell profiles, in the form:
///
///     eval "$(envdir-helper)"
///
/// The generated output is compatible with sh, and thus with bash and zsh.
#[derive(Clap)]
#[clap(version=env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Directory to read environment variables from [default: ~/.envdir]
    envdir: Option<PathBuf>,
    /// Export generated environment variables [default: true]
    #[clap(long)]
    export: Option<bool>,
}

#[derive(Error, Debug)]
enum EnvdirError {
    #[error("failed to locate default envdir")]
    NoDefaultEnvdir(#[from] DefaultDirError),
    #[error("failed to read envdir directory")]
    EnvdirListFailed(#[from] io::Error),
    #[error("failed to decode a filename")]
    PathStringError(#[from] PathStringError),
}

const SELF: &str = env!("CARGO_BIN_NAME");

fn main() -> Result<(), EnvdirError> {
    let opts: Opts = Opts::parse();

    let envdir = match opts.envdir {
        None => default_envdir()?,
        Some(envdir) => envdir,
    };

    let output_fn = match opts.export {
        None => detect_env_script(&envdir)?,
        Some(true) => export_env_script,
        Some(false) => no_export_env_script,
    };

    for path in read_dir(envdir)?
        .filter_map(skip_failing_direntry)
        .map(|entry| entry.path())
        .filter(|path| !path.is_dir())
    {
        let name = path_to_string(&path)?;
        match env_content(&path) {
            Ok(content) => println!("{}", output_fn(name, &content)),
            Err(e) => eprintln!("{}: error reading env value for {:?}: {:?}", SELF, name, e),
        };
    }

    Ok(())
}

fn skip_failing_direntry<E: Debug>(result: Result<DirEntry, E>) -> Option<DirEntry> {
    match result {
        Ok(direntry) => Some(direntry),
        Err(e) => {
            eprintln!("{}: error reading envdir: {:?}", SELF, e);
            None
        }
    }
}

#[derive(Error, Debug)]
#[error("a required environment variable was not set")]
struct DefaultDirError(#[from] env::VarError);

fn default_envdir() -> Result<PathBuf, DefaultDirError> {
    let mut envdir = PathBuf::from(env::var("HOME")?);
    envdir.push(".envdir");

    Ok(envdir)
}

type ExportScript = fn(&str, &str) -> String;

fn detect_env_script(path: &Path) -> Result<ExportScript, PathStringError> {
    let file_name = path_to_string(path)?;
    Ok(if file_name.ends_with("rc") {
        no_export_env_script
    } else {
        export_env_script
    })
}

fn no_export_env_script(name: &str, content: &str) -> String {
    let name = shlex::quote(name);
    let content= shlex::quote(content);
    format!("{}={}", name, content)
}

fn export_env_script(name: &str, content: &str) -> String {
    let name = shlex::quote(name);
    let content= shlex::quote(content);
    format!("{}={}; export {}", name, content, name)
}

#[derive(Error, Debug)]
enum PathStringError {
    #[error("path has no name: {0}")]
    NamelessPath(PathBuf),
    #[error("path has a non-unicode name: {0:?}")]
    NonUnicodePath(OsString),
}

fn path_to_string(path: &Path) -> Result<&str, PathStringError> {
    use PathStringError::*;
    let file_name = path.file_name()
        .ok_or_else(|| NamelessPath(path.into()))?;
    file_name.to_str()
        .ok_or_else(|| NonUnicodePath(file_name.into()))
}

#[derive(Error, Debug)]
enum EnvContentError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("program produced non-UTF-8 output: {0}")]
    NonUnicodeOutput(#[from] FromUtf8Error),
    #[error("program {0:?} exited with status: {1}")]
    ProgramFailed(PathBuf, ExitStatus)
}

fn env_content(path: &Path) -> Result<String, EnvContentError> {
    let mut content = if is_program(path)? {
        env_program_content(path)?
    } else {
        env_file_content(path)?
    };

    if content.ends_with("\n") {
        content.pop();
    }
    
    Ok(content)
}

fn is_program(path: &Path) -> io::Result<bool> {
    const EXEC_MASK: u32 = (libc::S_IXUSR | libc::S_IXGRP | libc::S_IXOTH) as u32;

    use std::os::unix::fs::PermissionsExt;

    let metadata = metadata(path)?;
    let permissions = metadata.permissions();

    Ok(permissions.mode() & EXEC_MASK != 0)
}

fn env_program_content(path: &Path) -> Result<String, EnvContentError> {
    use EnvContentError::*;

    let output = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    
    if output.status.success() {
        let output = String::from_utf8(output.stdout)?;
        Ok(output)
    } else {
        Err(ProgramFailed(path.to_path_buf(), output.status))
    }
}

fn env_file_content(path: &Path) -> Result<String, EnvContentError> {
    Ok(read_to_string(path)?)
}
