use core::cmp;
use core::convert::{TryFrom, TryInto};
use core::fmt::{self, Debug, Display, Write};
use core::mem;
use core::ptr;
use core::slice;
use core::str;

use alloc::vec::Vec;

use hashbrown::HashMap;
use lazy_static::lazy_static;

use liblumen_arena::DroplessArena;

use liblumen_core::locks::RwLock;

use super::{AsTerm, Term, TypeError, TypedTerm};

/// The maximum number of atoms allowed
///
/// This is derived from the fact that atom values are
/// tagged in their highest 6 bits, so they are unusable.
pub const MAX_ATOMS: usize = usize::max_value() >> 6;

/// The maximum length of an atom (255)
pub const MAX_ATOM_LENGTH: usize = u16::max_value() as usize;

lazy_static! {
    /// The atom table used by the runtime system
    static ref ATOMS: RwLock<AtomTable> = Default::default();
}

/// An interned string, represented in memory as a tagged integer id.
///
/// This struct contains the untagged id
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Atom(usize);
impl Atom {
    pub const SIZE_IN_WORDS: usize = 1;

    /// Gets the identifier associated with this atom
    #[inline(always)]
    pub fn id(&self) -> usize {
        self.0
    }

    /// Returns the string representation of this atom
    #[inline]
    pub fn name(&self) -> &'static str {
        ATOMS.read().get_name(self.0).unwrap()
    }

    /// Creates a new atom from a slice of bytes interpreted as Latin-1.
    ///
    /// Returns `Err` if the atom name is invalid or the table overflows
    #[inline]
    pub fn try_from_latin1_bytes(name: &[u8]) -> Result<Self, AtomError> {
        Self::try_from_str(str::from_utf8(name).unwrap())
    }

    /// Like `try_from_latin1_bytes`, but requires that the atom already exists
    ///
    /// Returns `Err` if the atom does not exist
    #[inline]
    pub fn try_from_latin1_bytes_existing(name: &[u8]) -> Result<Self, AtomError> {
        Self::try_from_str_existing(str::from_utf8(name).unwrap())
    }

    /// Creates a new atom from a `str`.
    ///
    /// Returns `Err` if the atom name is invalid or the table overflows
    #[inline]
    pub fn try_from_str<S: AsRef<str>>(s: S) -> Result<Self, AtomError> {
        let name = s.as_ref();
        Self::validate(name)?;
        if let Some(id) = ATOMS.read().get_id(name) {
            return Ok(Atom(id));
        }
        let id = ATOMS.write().get_id_or_insert(name)?;
        Ok(Atom(id))
    }

    /// Creates a new atom from a `str`, but only if the atom already exists
    ///
    /// Returns `Err` if the atom does not exist
    #[inline]
    pub fn try_from_str_existing<S: AsRef<str>>(s: S) -> Result<Self, AtomError> {
        let name = s.as_ref();
        Self::validate(name)?;
        if let Some(id) = ATOMS.read().get_id(name) {
            return Ok(Atom(id));
        }
        Err(AtomError(AtomErrorKind::NonExistent))
    }

    /// Creates a new atom from its id.
    ///
    /// # Safety
    ///
    /// This function is unsafe because creating an `Atom`
    /// with an id that doesn't exist will result in undefined
    /// behavior. This should only be used by `Term` when converting
    /// to `TypedTerm`
    /// ```
    #[inline]
    pub unsafe fn from_id(id: usize) -> Self {
        Self(id)
    }

    fn validate(name: &str) -> Result<(), AtomError> {
        let len = name.len();
        if len > MAX_ATOM_LENGTH {
            return Err(AtomError(AtomErrorKind::InvalidLength(len)));
        }
        Ok(())
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(":'")?;
        self.name()
            .chars()
            .flat_map(char::escape_default)
            .try_for_each(|c| f.write_char(c))?;
        f.write_char('\'')
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = ATOMS.read().get_name(self.0) {
            f.write_str(":\"")?;
            name.chars()
                .flat_map(char::escape_default)
                .try_for_each(|c| f.write_char(c))?;
            f.write_char('\"')
        } else {
            f.debug_tuple("Atom").field(&self.0).finish()
        }
    }
}

unsafe impl AsTerm for Atom {
    #[inline]
    unsafe fn as_term(&self) -> Term {
        Term::make_atom(self.0)
    }
}
impl PartialOrd for Atom {
    #[inline]
    fn partial_cmp(&self, other: &Atom) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Atom {
    #[inline]
    fn cmp(&self, other: &Atom) -> cmp::Ordering {
        use cmp::Ordering;

        if self.0 == other.0 {
            return Ordering::Equal;
        }
        self.name().cmp(other.name())
    }
}

impl TryFrom<Term> for Atom {
    type Error = TypeError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        term.to_typed_term().unwrap().try_into()
    }
}

impl TryFrom<TypedTerm> for Atom {
    type Error = TypeError;

    fn try_from(typed_term: TypedTerm) -> Result<Self, Self::Error> {
        match typed_term {
            TypedTerm::Atom(atom) => Ok(atom),
            _ => Err(TypeError),
        }
    }
}

/// Produced by operations which create atoms
#[derive(Debug)]
pub struct AtomError(AtomErrorKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomErrorKind {
    TooManyAtoms,
    InvalidLength(usize),
    NonExistent,
}

impl Display for AtomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            AtomErrorKind::TooManyAtoms => write!(
                f,
                "exceeded system limit: maximum number of atoms ({})",
                MAX_ATOMS
            ),
            AtomErrorKind::InvalidLength(len) => write!(
                f,
                "invalid atom, length is {}, maximum length is {}",
                len, MAX_ATOM_LENGTH
            ),
            AtomErrorKind::NonExistent => {
                write!(f, "tried to convert to an atom that doesn't exist")
            }
        }
    }
}

struct AtomTable {
    ids: HashMap<&'static str, usize>,
    names: Vec<&'static str>,
    arena: DroplessArena,
}
impl AtomTable {
    fn new(names: &[&'static str]) -> Self {
        let len = names.len();
        let mut table = Self {
            ids: HashMap::with_capacity(len),
            names: Vec::with_capacity(len),
            arena: DroplessArena::default(),
        };
        let interned_names = &mut table.names;
        for name in names {
            table.ids.entry(name).or_insert_with(|| {
                let id = interned_names.len();
                interned_names.push(name);
                id
            });
        }
        table
    }

    fn get_id(&self, name: &str) -> Option<usize> {
        self.ids.get(name).cloned()
    }

    fn get_name(&self, id: usize) -> Option<&'static str> {
        self.names.get(id).cloned()
    }

    fn get_id_or_insert(&mut self, name: &str) -> Result<usize, AtomError> {
        match self.get_id(name) {
            Some(existing_id) => Ok(existing_id),
            None => unsafe { self.insert(name) },
        }
    }

    // Unsafe because `name` should already have been checked as not existing while holding a
    // `mut reference`.
    unsafe fn insert(&mut self, name: &str) -> Result<usize, AtomError> {
        let id = self.names.len();
        if id > MAX_ATOMS {
            return Err(AtomError(AtomErrorKind::TooManyAtoms));
        }

        let size = name.len();

        let s = if size > 0 {
            // Copy string into arena
            let ptr = self.arena.alloc_raw(size, mem::align_of::<u8>());
            ptr::copy_nonoverlapping(name as *const _ as *const u8, ptr, size);
            let bytes = slice::from_raw_parts(ptr, size);

            str::from_utf8_unchecked(bytes)
        } else {
            ""
        };

        // Push into id map
        self.ids.insert(s, id);
        self.names.push(s);

        Ok(id)
    }
}
impl Default for AtomTable {
    fn default() -> Self {
        let atoms = &["true", "false", "undefined", "nil", "ok", "error"];
        AtomTable::new(atoms)
    }
}

/// This is safe to implement because the only usage is the ATOMS static, which is wrapped in an
/// `RwLock`, but it is _not_ `Sync` in general, so don't try and use it as such in other situations
unsafe impl Sync for AtomTable {}

pub enum Encoding {
    Latin1,
    Unicode,
    Utf8,
}

impl TryFrom<Term> for Encoding {
    type Error = EncodingError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        match term.to_typed_term().unwrap() {
            TypedTerm::Atom(atom) => {
                let unicode_atom = Atom::try_from_str("unicode").unwrap();

                if atom == unicode_atom {
                    Ok(Encoding::Unicode)
                } else {
                    let utf8_atom = Atom::try_from_str("utf8").unwrap();

                    if atom == utf8_atom {
                        Ok(Encoding::Utf8)
                    } else {
                        let latin1_atom = Atom::try_from_str("latin1").unwrap();

                        if atom == latin1_atom {
                            Ok(Encoding::Latin1)
                        } else {
                            Err(EncodingError::NotAnEncodingName(term))
                        }
                    }
                }
            }
            _ => Err(EncodingError::NotAnAtom(term)),
        }
    }
}

pub enum EncodingError {
    NotAnAtom(Term),
    NotAnEncodingName(Term),
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodingError::NotAnAtom(term) => write!(f, "Encoding ({:#?}) is not an atom", term),
            EncodingError::NotAnEncodingName(term) => write!(
                f,
                "Encoding atom ({:#?}) is not one of the supported values (latin1, unicode, or utf8)",
                term
            ),
        }
    }
}
