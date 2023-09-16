// SPDX-License-Identifier: MPL-2.0

// Copyright (C) 2023  Soc Virnyl Estela

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

pub mod compress;
pub mod decompress;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

use crate::cli::Opts;
use crate::vendor;
use crate::vendor::vendor;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn, Level};

pub fn copy_dir_all(src: impl AsRef<Path>, dst: &Path) -> Result<(), io::Error> {
    debug!("Copying sources");
    debug!(?dst);
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        trace!(?entry);
        trace!(?ty);
        if ty.is_dir() {
            trace!(?ty, "Is directory?");
            copy_dir_all(&entry.path(), &dst.join(&entry.file_name()))?;

        // Should we respect symlinks?
        // } else if ty.is_symlink() {
        //     debug!("Is symlink");
        //     let path = fs::read_link(&entry.path())?;
        //     let path = fs::canonicalize(&path).unwrap();
        //     debug!(?path);
        //     let pathfilename = path.file_name().unwrap_or(OsStr::new("."));
        //     if path.is_dir() {
        //         copy_dir_all(&path, &dst.join(pathfilename))?;
        //     } else {
        //         fs::copy(&path, &mut dst.join(pathfilename))?;
        //     }

        // Be pedantic or you get symlink error
        } else if ty.is_file() {
            trace!(?ty, "Is file?");
            fs::copy(&entry.path(), &mut dst.join(&entry.file_name()))?;
        };
    }
    Ok(())
}

pub fn process_src(args: &Opts, prjdir: &Path, target_file: &OsStr) -> Result<(), io::Error> {
    info!("Guessed project root at uwu {}", prjdir.display());
    let pathtomanifest = prjdir.join(target_file);
    debug!(?pathtomanifest);
    if pathtomanifest.exists() {
        if let Ok(isworkspace) = vendor::is_workspace(&pathtomanifest) {
            if isworkspace {
                info!(?pathtomanifest, "Project uses a workspace!");
            } else {
                info!(?pathtomanifest, "Project does not use a workspace!");
            };

            match vendor::has_dependencies(&pathtomanifest) {
                Ok(hasdeps) => {
                    if hasdeps && isworkspace {
                        info!("Workspace has dependencies!");
                        vendor(args, prjdir, None)?;
                    } else if hasdeps && !isworkspace {
                        info!("Non-workspace manifest has dependencies!");
                        vendor(args, prjdir, None)?;
                    } else if !hasdeps && isworkspace {
                        info!("Workspace has no global dependencies. May vendor dependencies from member crates.");
                        vendor(args, prjdir, None)?;
                    } else {
                        // This is what we call a "zero cost" abstraction.
                        info!("No dependencies, no need to vendor!");
                    };
                }
                Err(err) => return Err(err),
            };

            if args.cargotoml.is_empty() {
                info!(?args.cargotoml, "No subcrates to vendor!");
            } else {
                info!(?args.cargotoml, "Found subcrates to vendor!");
                vendor::cargotomls(args, prjdir)?;
            }
        }
    } else {
        warn!("Project does not have a manifest file at the root of the project!");
        if args.cargotoml.is_empty() {
            info!(?args.cargotoml, "No subcrates to vendor!");
        } else {
            info!(?args.cargotoml, "Found subcrates to vendor!");
            vendor::cargotomls(args, prjdir)?;
        }
    }
    Ok(())
}