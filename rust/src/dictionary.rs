use std::{
	collections::{
		BTreeMap,
		HashMap,
	},
	fmt,
	hash::Hash,
	ops::Index,
	sync::Arc,
};

/// A de-duplicating dictionary.
///
/// This structure maintains a collection of all inserted items, as well as a
/// bidirectional mapping between them and `Identifier`s which can be used to
/// look them up later.
///
/// Items are stored inside `Arc`s, since the dictionary needs to maintain two
/// links to the same object. This means that users can choose between holding
/// the dictionary's `Identifier`s *or* other `Arc` handles.
pub struct Dictionary<T: ?Sized + Eq + Hash> {
	cached: HashMap<Arc<T>, usize>,
	idents: BTreeMap<usize, Arc<T>>,
}

impl<T: ?Sized + Eq + Hash> Dictionary<T> {
	/// Creates an empty dictionary.
	pub fn new() -> Self {
		Self {
			cached: HashMap::new(),
			idents: BTreeMap::new(),
		}
	}

	/// Tests if the dictionary is empty.
	pub fn is_empty(&self) -> bool {
		self.cached.is_empty()
	}

	/// Counts how many items are stored in the dictionary.
	pub fn len(&self) -> usize {
		self.cached.len()
	}

	/// Inserts an item into the dictionary, returning an opaque identifier that
	/// can be used to retrieve it later. If the item is already stored in the
	/// dictionary, then the existing identifier is returned.
	pub fn insert(&mut self, value: impl AsRef<T> + Into<Arc<T>>) -> Identifier {
		let next_ident = self.len();
		if let Some(&out) = self.cached.get(value.as_ref()) {
			return Identifier::new(out);
		}
		let arced = value.into();
		self.cached.insert(arced.clone(), next_ident);
		self.idents.insert(next_ident, arced);
		Identifier::new(next_ident)
	}

	/// Attempts to get a value out of the dictionary.
	///
	/// While `Identifier` can only be produced by inserting values into the
	/// dictionary, I currently have no way to tie identifiers to the dictionary
	/// that produced them, so an identifier from one dictionary might not work
	/// in another.
	pub fn lookup(&self, ident: Identifier) -> Option<Arc<T>> {
		self.idents.get(&ident.ident).cloned()
	}

	/// Attempts to get the identifier for a value if it is present in the
	/// dictionary.
	pub fn lookup_value(&self, value: impl AsRef<T>) -> Option<Identifier> {
		self.cached
			.get(value.as_ref())
			.map(|&ident| Identifier::new(ident))
	}

	/// Tests if a value is stored in the dictionary.
	pub fn contains(&self, value: impl AsRef<T>) -> bool {
		self.cached.contains_key(value.as_ref())
	}

	/// Tests if a key is stored in the dictionary.
	pub fn contains_key(&self, ident: Identifier) -> bool {
		self.idents.contains_key(&ident.ident)
	}
}

impl<T: ?Sized + Eq + Hash> Index<Identifier> for Dictionary<T> {
	type Output = T;

	fn index(&self, ident: Identifier) -> &Self::Output {
		&self.idents[&ident.ident]
	}
}

impl<T: ?Sized + Eq + Hash> Clone for Dictionary<T> {
	fn clone(&self) -> Self {
		Self {
			cached: self.cached.clone(),
			idents: self.idents.clone(),
		}
	}
}

impl<T: ?Sized + Eq + Hash + fmt::Debug> fmt::Debug for Dictionary<T> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_list().entries(self.idents.values()).finish()
	}
}

impl<T: ?Sized + Eq + Hash> Default for Dictionary<T> {
	fn default() -> Self {
		Self::new()
	}
}

/// An opaque identifier produced by a dictionary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
	ident: usize,
}

impl Identifier {
	fn new(ident: usize) -> Self {
		Self { ident }
	}
}

impl fmt::Display for Identifier {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&self.ident, fmt)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn build_dict() {
		let mut dict = Dictionary::<str>::new();
		let id1 = dict.insert("hello");
		let id2 = dict.insert("world");
		let id3 = dict.insert("hello");

		assert_eq!(id1, id3);
		assert_eq!(dict.len(), 2);

		assert_eq!(&dict[id2], "world");
	}
}
