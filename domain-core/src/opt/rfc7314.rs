//! EDNS Options from RFC 7314

use crate::iana::OptionCode;
use crate::message_builder::OptBuilder;
use crate::octets::{Compose, OctetsBuilder, ShortBuf};
use crate::parse::{Parse, ParseError, Parser};
use super::CodeOptData;


//------------ Expire --------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Expire(Option<u32>);

impl Expire {
    pub fn new(expire: Option<u32>) -> Self {
        Expire(expire)
    }

    pub fn push<Target: OctetsBuilder>(
        builder: &mut OptBuilder<Target>,
        expire: Option<u32>
    ) -> Result<(), ShortBuf> {
        builder.push(&Self::new(expire))
    }

    pub fn expire(self) -> Option<u32> {
        self.0
    }
}


//--- Parse and Compose

impl<Ref: AsRef<[u8]>> Parse<Ref> for Expire {
    fn parse(parser: &mut Parser<Ref>) -> Result<Self, ParseError> {
        if parser.remaining() == 0 {
            Ok(Expire::new(None))
        }
        else {
            u32::parse(parser).map(|res| Expire::new(Some(res)))
        }
    }

    fn skip(parser: &mut Parser<Ref>) -> Result<(), ParseError> {
        if parser.remaining() == 0 {
            Ok(())
        }
        else {
            parser.advance(4)
        }
    }
}

impl Compose for Expire {
    fn compose<T: OctetsBuilder>(
        &self,
        target: &mut T
    ) -> Result<(), ShortBuf> {
        if let Some(value) = self.0 {
            value.compose(target)?;
        }
        Ok(())
    }
}


//--- OptData

impl CodeOptData for Expire {
    const CODE: OptionCode = OptionCode::Expire;
}

