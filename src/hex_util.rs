pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait FromHex {
    fn from_hex(hex_str: &str) -> Self;
}

#[macro_use]
macro_rules! serialize_as_hex_str {
    ($( $t:ident )* ) => {
        $(
            impl serde::Serialize for $t {
                fn serialize<S>(&self, serializer: S) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error> where
                    S: serde::Serializer {
                        use crate::hex_util::ToHex;
                        serializer.serialize_str(&self.to_hex())
                }
            }
        )*
    };
}


#[macro_use]
macro_rules! deserialize_from_hex {
($( $t:ident )* ) => {
    $(
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as serde::Deserializer<'de>>::Error> where
                D: serde::Deserializer<'de> {
                use crate::hex_util::FromHex;
                let s: &str = serde::Deserialize::deserialize(deserializer)?;
                Ok($t::from_hex(s))
            }
        }
    )*
};
}
