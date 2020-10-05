use crate::traits;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    Fichier(File<HANDLE>),
    Dossier(Dir<HANDLE>),
}

// TODO: Implement any useful helper methods on `Entry`.

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    // FIXME: Implement `traits::Entry` for `Entry`.
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match self {
            Entry::Fichier(f) => &f.name,
            Entry::Dossier(d) => &d.name,
        }
    }

    fn metadata(&self) -> &Self::Metadata {
        match self {
            Entry::Fichier(f) => &f.metadata,
            Entry::Dossier(d) => &d.metadata,
        }
    }

    fn as_file(&self) -> Option<&Self::File> {
        match self {
            Entry::Fichier(f) => Some(f),
            Entry::Dossier(_) => None,
        }
    }

    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            Entry::Fichier(_) => None,
            Entry::Dossier(d) => Some(d),
        }
    }

    fn into_file(self) -> Option<Self::File> {
        match self {
            Entry::Fichier(f) => Some(f),
            Entry::Dossier(_) => None,
        }
    }

    fn into_dir(self) -> Option<Self::Dir> {
        match self {
            Entry::Fichier(_) => None,
            Entry::Dossier(d) => Some(d),
        }
    }
}
