//! EDNS Options from RFC 7828

use crate::iana::OptionCode;
use crate::message_builder::OptBuilder;
use crate::octets::{Compose, OctetsBuilder, ShortBuf};
use crate::parse::{Parse, ParseError, Parser};
use super::CodeOptData;


//------------ TcpKeepalive --------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TcpKeepalive(u16);

impl TcpKeepalive {
    pub fn new(timeout: u16) -> Self {
        TcpKeepalive(timeout)
    }

    pub fn push<Target: OctetsBuilder>(
        builder: &mut OptBuilder<Target>,
        timeout: u16
    ) -> Result<(), ShortBuf> {
        builder.push(&Self::new(timeout))
    }

    pub fn timeout(self) -> u16 {
        self.0
    }
}


//--- Parse and Compose

impl<Ref: AsRef<[u8]>> Parse<Ref> for TcpKeepalive {
    fn parse(parser: &mut Parser<Ref>) -> Result<Self, ParseError> {
        u16::parse(parser).map(Self::new)
    }

    fn skip(parser: &mut Parser<Ref>) -> Result<(), ParseError> {
        u16::skip(parser)
    }
}

impl Compose for TcpKeepalive {
    fn compose<T: OctetsBuilder>(
        &self,
        target: &mut T
    ) -> Result<(), ShortBuf> {
        self.0.compose(target)
    }
}


//--- CodeOptData

impl CodeOptData for TcpKeepalive {
    const CODE: OptionCode = OptionCode::TcpKeepalive;
}

