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
	type ElementType = String;

	fn get_all(&self) -> Self::ElementType {
		self.to_hex()
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
			format!("{}", value)
		} else {
			format!("{:02X}", value)
		})
	}
}

pub enum IndexableVariable {
	Argb(MattyArgb),
	PlainString(String),
}

impl IndexableVariable {
	pub fn plain(value: String) -> Self {
		IndexableVariable::PlainString(value)
	}
}

impl From<MattyArgb> for IndexableVariable {
	fn from(value: MattyArgb) -> Self {
		IndexableVariable::Argb(value)
	}
}

impl CharIndex for IndexableVariable {
	type ElementType = String;

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
