//! Decoding and encoding of Base 16 a.k.a. hex digits.
//!
//! The Base 16 encoding is defined in [RFC 4648]. It really is just a normal
//! hex-encoding using the (case-insensitive) letters ‘A’ to ‘F’ as
//! additional values for the digits.
//!
//! The module defines the type [`Decoder`] which keeps the state necessary
//! for decoding. The various functions offered use such a decoder to decode
//! and encode octets in various forms.
//!
//! [RFC 4648]: https://tools.ietf.org/html/rfc4648

use crate::base::octets::{EmptyBuilder, FromBuilder, OctetsBuilder};
use core::fmt;
#[cfg(feature = "std")]
use std::string::String;

//------------ Re-exports ----------------------------------------------------

pub use super::base64::DecodeError;

//------------ Convenience Functions -----------------------------------------

/// Decodes a string with Base 16 encoded data.
///
/// The function attempts to decode the entire string and returns the result
/// as an `Octets` value.
pub fn decode<Octets>(s: &str) -> Result<Octets, DecodeError>
where
    Octets: FromBuilder,
    <Octets as FromBuilder>::Builder:
        OctetsBuilder<Octets = Octets> + EmptyBuilder,
{
    let mut decoder = Decoder::<<Octets as FromBuilder>::Builder>::new();
    for ch in s.chars() {
        decoder.push(ch)?;
    }
    decoder.finalize()
}

/// Encodes binary data in Base 16 and writes it into a format stream.
///
/// This function is intended to be used in implementations of formatting
/// traits:
///
/// ```
/// use core::fmt;
/// use domain::utils::base16;
///
/// struct Foo<'a>(&'a [u8]);
///
/// impl<'a> fmt::Display for Foo<'a> {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         base16::display(&self.0, f)
///     }
/// }
/// ```
pub fn display<Octets, Target>(octets: &Octets, f: &mut Target) -> fmt::Result
where
    Octets: AsRef<[u8]> + ?Sized,
    Target: fmt::Write,
{
    for octet in octets.as_ref() {
        f.write_char(ENCODE_ALPHABET[(octet >> 4) as usize])?;
        f.write_char(ENCODE_ALPHABET[(octet & 0x0F) as usize])?;
    }
    Ok(())
}

/// Encodes binary data in Base 16 and returns the encoded data as a string.
#[cfg(feature = "std")]
pub fn encode_string<B: AsRef<[u8]> + ?Sized>(bytes: &B) -> String {
    let mut res = String::with_capacity(bytes.as_ref().len() * 2);
    display(bytes, &mut res).unwrap();
    res
}

/// Returns a placeholder value that implements `Display` for encoded data.
pub fn encode_display<Octets: AsRef<[u8]>>(
    octets: &Octets,
) -> impl fmt::Display + '_ {
    struct Display<'a>(&'a [u8]);

    impl<'a> fmt::Display for Display<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            display(self.0, f)
        }
    }

    Display(octets.as_ref())
}

/// Serialize and deserialize octets Base 16 encoded or binary.
///
/// This module can be used with Serde’s `with` attribute. It will serialize
/// an octets sequence as a Base 16 encoded string with human readable
/// serializers or as a raw octets sequence for compact serializers.
#[cfg(feature = "serde")]
pub mod serde {
    use crate::base::octets::{
        DeserializeOctets, EmptyBuilder, FromBuilder, OctetsBuilder,
        SerializeOctets,
    };
    use core::fmt;

    pub fn serialize<Octets, S>(
        octets: &Octets,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        Octets: AsRef<[u8]> + SerializeOctets,
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(&super::encode_display(octets))
        } else {
            octets.serialize_octets(serializer)
        }
    }

    pub fn deserialize<'de, Octets, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Octets, D::Error>
    where
        Octets: FromBuilder + DeserializeOctets<'de>,
        <Octets as FromBuilder>::Builder: EmptyBuilder,
    {
        struct Visitor<'de, Octets: DeserializeOctets<'de>>(Octets::Visitor);

        impl<'de, Octets> serde::de::Visitor<'de> for Visitor<'de, Octets>
        where
            Octets: FromBuilder + DeserializeOctets<'de>,
            <Octets as FromBuilder>::Builder:
                OctetsBuilder<Octets = Octets> + EmptyBuilder,
        {
            type Value = Octets;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an Base16-encoded string")
            }

            fn visit_str<E: serde::de::Error>(
                self,
                v: &str,
            ) -> Result<Self::Value, E> {
                super::decode(v).map_err(E::custom)
            }

            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                value: &'de [u8],
            ) -> Result<Self::Value, E> {
                self.0.visit_borrowed_bytes(value)
            }

            #[cfg(feature = "std")]
            fn visit_byte_buf<E: serde::de::Error>(
                self,
                value: std::vec::Vec<u8>,
            ) -> Result<Self::Value, E> {
                self.0.visit_byte_buf(value)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Visitor(Octets::visitor()))
        } else {
            Octets::deserialize_with_visitor(
                deserializer,
                Visitor(Octets::visitor()),
            )
        }
    }
}

//------------ Decoder -------------------------------------------------------

/// A Base 16 decoder.
///
/// This type keeps all the state for decoding a sequence of characters
/// representing data encoded in Base 16. Upon success, the decoder returns
/// the decoded data.
pub struct Decoder<Builder> {
    /// A buffer for the first half of an octet.
    buf: Option<u8>,

    /// The target or an error if something went wrong.
    target: Result<Builder, DecodeError>,
}

impl<Builder: EmptyBuilder> Decoder<Builder> {
    /// Creates a new, empty decoder using the *base32hex* variant.
    pub fn new() -> Self {
        Decoder {
            buf: None,
            target: Ok(Builder::empty()),
        }
    }
}

impl<Builder: OctetsBuilder> Decoder<Builder> {
    /// Finalizes decoding and returns the decoded data.
    pub fn finalize(self) -> Result<Builder::Octets, DecodeError> {
        if self.buf.is_some() {
            return Err(DecodeError::ShortInput);
        }

        self.target.map(OctetsBuilder::freeze)
    }

    /// Decodes one more character of data.
    ///
    /// Returns an error as soon as the encoded data is determined to be
    /// illegal. It is okay to push more data after the first error. The
    /// method will just keep returning errors.
    pub fn push(&mut self, ch: char) -> Result<(), DecodeError> {
        let value = match ch.to_digit(16) {
            Some(value) => value as u8,
            None => {
                self.target = Err(DecodeError::IllegalChar(ch));
                return Err(DecodeError::IllegalChar(ch));
            }
        };
        if let Some(upper) = self.buf.take() {
            self.append(upper | value);
        } else {
            self.buf = Some(value << 4)
        }
        match self.target {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Appends a decoded octet to the target.
    fn append(&mut self, value: u8) {
        let target = match self.target.as_mut() {
            Ok(target) => target,
            Err(_) => return,
        };
        if let Err(err) = target.append_slice(&[value]) {
            self.target = Err(err.into());
        }
    }
}

impl<Builder: EmptyBuilder> Default for Decoder<Builder> {
    fn default() -> Self {
        Self::new()
    }
}

//------------ Constants -----------------------------------------------------

/// The alphabet used for encoding.
///
/// We have to have this because `char::from_digit` prefers lower case letters
/// while the RFC prefers upper case.
const ENCODE_ALPHABET: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', // 0x00 .. 0x07
    '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', // 0x08 .. 0x0F
];

//============ Test ==========================================================

#[cfg(test)]
#[cfg(feature = "std")]
mod test {
    use super::*;
    use std::string::String;

    #[test]
    #[cfg(feature = "bytes")]
    fn decode_str() {
        use super::DecodeError;

        fn decode(s: &str) -> Result<std::vec::Vec<u8>, DecodeError> {
            super::decode(s)
        }

        assert_eq!(&decode("").unwrap(), b"");
        assert_eq!(&decode("F0").unwrap(), b"\xF0");
        assert_eq!(&decode("F00f").unwrap(), b"\xF0\x0F");
    }

    #[test]
    fn test_display() {
        fn fmt(s: &[u8]) -> String {
            let mut out = String::new();
            display(s, &mut out).unwrap();
            out
        }

        assert_eq!(fmt(b""), "");
        assert_eq!(fmt(b"\xf0"), "F0");
        assert_eq!(fmt(b"\xf0\x0f"), "F00F");
    }
}
