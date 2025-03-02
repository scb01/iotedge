// Copyright (c) Microsoft. All rights reserved.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use anyhow::Context;
use git2::Repository;
use hex::encode;
use log::debug;

use crate::error::Error;

type RemoteUrl = String;
type CommitId = String;
type RemoteMap = HashMap<RemoteUrl, CommitId>;

#[derive(Debug)]
struct GitModule {
    remote: RemoteUrl,
    commit: CommitId,
    flag: bool,
}

impl fmt::Display for GitModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} : {} {}",
            self.remote,
            self.commit,
            if self.flag { "****" } else { "" }
        )?;
        Ok(())
    }
}

impl GitModule {
    pub fn new(remote: String, commit: String, flag: bool) -> Self {
        GitModule {
            remote,
            commit,
            flag,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Git2Tree {
    root: GitModule,
    children: Vec<Git2Tree>,
}

fn sanitize_url(url: &str) -> String {
    url.trim_end_matches(".git").replace("www.", "")
}

impl Git2Tree {
    fn format(&self, level: i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.root.flag {
            writeln!(
                f,
                "*** FAILURE ***  Line which follows has a mismatched commit"
            )?;
        }
        for _l in 0..level {
            write!(f, "  ")?;
        }
        write!(f, "|- ")?;
        writeln!(f, "{}", self.root)?;
        for child in &self.children {
            child.format(level + 1, f)?;
        }
        Ok(())
    }

    fn new_as_subtree(path: &Path, remotes: &mut RemoteMap) -> anyhow::Result<Self> {
        debug!("repo path {:?}", path);
        let repo = Repository::open(path).context(Error::Git)?;
        let remote = sanitize_url(
            repo.find_remote("origin")
                .context(Error::Git)?
                .url()
                .unwrap(),
        );
        debug!("remote = {:?}", remote);
        let commit = encode(
            repo.head()
                .context(Error::Git)?
                .peel_to_commit()
                .context(Error::Git)?
                .id(),
        );
        debug!("commit = {:?}", commit);
        let flag = remotes.get(&remote).map_or(false, |c| &commit != c);
        remotes
            .entry(remote.clone())
            .or_insert_with(|| commit.clone());

        let mut children: Vec<Git2Tree> = Vec::new();
        for sm in repo.submodules().context(Error::Git)? {
            let child = Git2Tree::new_as_subtree(path.join(sm.path()).as_path(), remotes)?;
            children.push(child);
        }
        Ok(Git2Tree {
            root: GitModule::new(remote, commit, flag),
            children,
        })
    }

    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let mut remotes: RemoteMap = HashMap::new();
        Git2Tree::new_as_subtree(path, &mut remotes)
    }

    pub fn count_flagged(&self) -> i64 {
        let count = if self.root.flag { 1 } else { 0 };
        count
            + self
                .children
                .iter()
                .fold(0, |acc, x| acc + x.count_flagged())
    }
}

impl fmt::Display for Git2Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format(0, f)
    }
}
