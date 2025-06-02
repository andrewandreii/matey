use crate::material_newtype::MattyArgb;

pub trait CharIndex {
	type ElementType;

	fn get_all(&self) -> Self::ElementType;

	fn get(&self, idx: char) -> Option<Self::ElementType>;
}

impl<T, ET> CharIndex for &T
where
	T: CharIndex<ElementType = ET>,
{
	type ElementType = ET;

	fn get_all(&self) -> Self::ElementType {
		(*self).get_all()
	}

	fn get(&self, idx: char) -> Option<Self::ElementType> {
		(*self).get(idx)
	}
}

impl CharIndex for MattyArgb {
	type ElementType = Vec<u8>;

	fn get_all(&self) -> Self::ElementType {
		self.to_hex().into_bytes()
	}

	fn get(&self, idx: char) -> Option<Self::ElementType> {
		let value = match idx.to_ascii_lowercase() {
			'r' => self.red,
			'g' => self.green,
			'b' => self.blue,
			'a' => self.alpha,
			_ => return None,
		};

		Some(if idx.is_uppercase() {
			format!("{}", value).into_bytes()
		} else {
			format!("{:02X}", value).into_bytes()
		})
	}
}

pub enum IndexableVariable {
	Argb(MattyArgb),
	PlainString(Vec<u8>),
}

impl IndexableVariable {
	pub fn plain(value: Vec<u8>) -> Self {
		IndexableVariable::PlainString(value)
	}
}

impl From<MattyArgb> for IndexableVariable {
	fn from(value: MattyArgb) -> Self {
		IndexableVariable::Argb(value)
	}
}

impl CharIndex for IndexableVariable {
	type ElementType = Vec<u8>;

	fn get_all(&self) -> Self::ElementType {
		match self {
			IndexableVariable::Argb(v) => v.get_all(),
			IndexableVariable::PlainString(v) => v.clone(),
		}
	}

	fn get(&self, idx: char) -> Option<Self::ElementType> {
		match self {
			IndexableVariable::Argb(v) => v.get(idx),
			IndexableVariable::PlainString(_) => None,
		}
	}
}
