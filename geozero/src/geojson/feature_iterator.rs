// Copyright 2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use geojson::Feature;
use std::io;
use std::marker::PhantomData;

pub struct FeatureIterator<R> {
    reader: R,
    skip: Option<u8>,
    skip_preamble: bool,
    skip_appendix: bool,
    marker: PhantomData<Feature>,
}

impl<R> FeatureIterator<R> {
    pub fn new(reader: R) -> Self {
        FeatureIterator {
            reader: reader,
            skip: Some(b'['),
            skip_preamble: true,
            skip_appendix: false,
            marker: PhantomData,
        }
    }
}

impl<R> FeatureIterator<R>
    where R: io::Read
{
    fn skip_past_byte(&mut self, byte: u8) -> io::Result<bool> {
        let mut one_byte = [0];
        loop {
            if self.reader.read_exact(&mut one_byte).is_err() {
                return Ok(false);
            }
            if one_byte[0] == byte {
                return Ok(true);
            }
            if one_byte[0] == b']' {
              self.skip_appendix = true;
            }
            if !self.skip_preamble && !self.skip_appendix && !(one_byte[0] as char).is_whitespace() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("byte {}", one_byte[0])));
            }
        }
    }
}

impl<R> Iterator for FeatureIterator<R>
    where R: io::Read
{
    type Item = io::Result<Feature>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(skip) = self.skip {
            match self.skip_past_byte(skip) {
                Ok(true) => {}
                Ok(false) => {
                    return None;
                }
                Err(err) => {
                    return Some(Err(err));
                }
            }
            self.skip = None;
        }
        let de = serde_json::Deserializer::from_reader(&mut self.reader);
        match de.into_iter().next() {
            Some(Ok(v)) => {
                self.skip = Some(b',');
                Some(Ok(v))
            }
            Some(Err(err)) => {
                Some(Err(err.into()))
            }
            None => None,
        }
    }
}