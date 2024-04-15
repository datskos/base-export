// All code below is adapted from reth (https://github.com/paradigmxyz/reth)

// The MIT License (MIT)
//
// Copyright (c) 2022-2024 Reth Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use alloy_rlp::{Decodable, Encodable};
use reth_primitives::{
    bytes::{Buf, BytesMut},
    Block,
};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

/// An error that can occur when constructing and using a [`FileClient`].
#[derive(Debug, Error)]
pub enum FileClientError {
    /// An error occurred when opening or reading the file.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// An error occurred when decoding blocks, headers, or rlp headers from the file.
    #[error(transparent)]
    Rlp(#[from] alloy_rlp::Error),
}

pub(crate) struct BlockFileCodec;

impl Decoder for BlockFileCodec {
    type Item = Block;
    type Error = FileClientError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let buf_slice = &mut src.as_ref();
        let body = Block::decode(buf_slice)?;
        src.advance(src.len() - buf_slice.len());
        Ok(Some(body))
    }
}

impl Encoder<Block> for BlockFileCodec {
    type Error = FileClientError;

    fn encode(&mut self, item: Block, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst);
        Ok(())
    }
}
