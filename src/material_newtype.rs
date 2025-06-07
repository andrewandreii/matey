use material_colors::color::Argb;
use material_colors::scheme::Scheme;

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! sametype {
    (
        #[same_as = $other:ident]
        #[field_type = $tname:ident]
        $(#[$battr:meta])*
        $sv:vis struct $sname:ident {
            $($fv:vis $fname:ident,)+
        }
    ) => {
        $(#[$battr])*
        $sv struct $sname {
            $($fv $fname: $tname,)+
        }

        impl From<$other> for $sname {
            fn from(other: $other) -> $sname {
                $sname {
                    $($fname: other.$fname.into()),+
                }
            }
        }

        impl From<$sname> for $other {
            fn from(other: $sname) -> $other {
                $other {
                    $($fname: other.$fname.into()),+
                }
            }
        }
    };

    (
        #[iter]
        #[same_as = $other:ident]
        #[field_type = $tname:ident]
        $(#[$battr:meta])*
        $sv:vis struct $sname:ident {
            $($fv:vis $fname:ident,)+
        }
    ) => {
        sametype!(
            #[same_as = $other]
            #[field_type = $tname]
            $(#[$battr])*
            $sv struct $sname {
                $($fv $fname,)+
            }
        );

        mod internal {
            pub use std::array::IntoIter;
        }

        impl IntoIterator for $sname {
            type Item = (&'static str, $tname);
            type IntoIter = internal::IntoIter<Self::Item, {count!($($fname)+)}>;

            fn into_iter(self) -> Self::IntoIter {
                [
                    $((stringify!($fname), self.$fname)),+
                ].into_iter()
            }
        }

        impl<'a> IntoIterator for &'a $sname {
            type Item = (&'static str, &'a $tname);
            type IntoIter = internal::IntoIter<Self::Item, {count!($($fname)+)}>;

            fn into_iter(self) -> Self::IntoIter {
                [
                    $((stringify!($fname), &self.$fname)),+
                ].into_iter()
            }
        }
    };
}

sametype!(
	#[same_as = Argb]
	#[field_type = u8]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
	#[repr(C)]
	pub struct MateyArgb {
		pub alpha,
		pub red,
		pub green,
		pub blue,
	}
);

impl MateyArgb {
	pub fn to_hex(&self) -> String {
		format!("{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
	}
}

sametype!(
	#[iter]
	#[same_as = Scheme]
	#[field_type = MateyArgb]
	#[derive(Debug, Clone, Copy, Default)]
	#[repr(C)]
	pub struct MateyScheme {
		pub primary,
		pub on_primary,
		pub primary_container,
		pub on_primary_container,
		pub inverse_primary,
		pub primary_fixed,
		pub primary_fixed_dim,
		pub on_primary_fixed,
		pub on_primary_fixed_variant,
		pub secondary,
		pub on_secondary,
		pub secondary_container,
		pub on_secondary_container,
		pub secondary_fixed,
		pub secondary_fixed_dim,
		pub on_secondary_fixed,
		pub on_secondary_fixed_variant,
		pub tertiary,
		pub on_tertiary,
		pub tertiary_container,
		pub on_tertiary_container,
		pub tertiary_fixed,
		pub tertiary_fixed_dim,
		pub on_tertiary_fixed,
		pub on_tertiary_fixed_variant,
		pub error,
		pub on_error,
		pub error_container,
		pub on_error_container,
		pub surface_dim,
		pub surface,
		pub surface_tint,
		pub surface_bright,
		pub surface_container_lowest,
		pub surface_container_low,
		pub surface_container,
		pub surface_container_high,
		pub surface_container_highest,
		pub on_surface,
		pub on_surface_variant,
		pub outline,
		pub outline_variant,
		pub inverse_surface,
		pub inverse_on_surface,
		pub surface_variant,
		pub background,
		pub on_background,
		pub shadow,
		pub scrim,
	}
);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MateyTheme {
	pub light: MateyScheme,
	pub dark: MateyScheme,
}

impl MateyTheme {
	pub fn new(light: MateyScheme, dark: MateyScheme) -> Self {
		MateyTheme { light, dark }
	}
}
