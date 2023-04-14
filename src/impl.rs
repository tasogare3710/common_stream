use std::{collections::VecDeque, iter::Iterator, marker::PhantomData};

/// 入力イテレータ`Iterator<Item = T>`と変換関数`FnMut(T) -> Result<U, E>`を使用して`U`の値を返す出力イテレータの実装です。
/// 型変数`T`と`E`が`?Sized`ではない事に注意してください。このイテレータが`Box<dyn std::error::Error>`を返す事は出来ません。
pub struct Transformed<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    iter: I,
    transform: F,
    backing_store: VecDeque<U>,
    phantom: PhantomData<Result<U, E>>,
}

impl<I, T, U, E, F> crate::UnRead<U> for Transformed<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    fn unread(&mut self, token: U) {
        self.backing_store.push_back(token);
    }

    fn unread_from_tokens(&mut self, iter: impl IntoIterator<Item = U>) {
        self.backing_store.extend(iter)
    }
}

impl<I, T, U, E, F> Iterator for Transformed<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.backing_store.is_empty() {
            self.iter.next().map(|it| (self.transform)(it))
        } else {
            self.backing_store.pop_back().map(Ok)
        }
    }
}

use std::fmt;

impl<I, T, U, E, F> fmt::Debug for Transformed<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
    T: fmt::Debug,
    U: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommonStream")
            .field("backing_store", &self.backing_store)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<I, T, U, E, F> Transformed<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    pub fn new(iter: I, transform: F) -> Self {
        Self {
            iter,
            transform,
            backing_store: Default::default(),
            phantom: PhantomData,
        }
    }

    pub fn with_backing_store(iter: I, transform: F, backing_store: impl IntoIterator<Item = U>) -> Self {
        Self {
            iter,
            transform,
            backing_store: VecDeque::from_iter(backing_store),
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use ::{
        std::{
            collections::VecDeque,
            io::{self, BufRead as _, BufReader},
            iter::Iterator,
            string::FromUtf8Error,
        },
        thiserror::Error,
    };

    #[cfg(test)]
    struct BufReadIter {
        reader: BufReader<VecDeque<u8>>,
    }

    #[cfg(test)]
    impl FromIterator<u8> for BufReadIter {
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
            Self {
                reader: BufReader::new(VecDeque::from_iter(iter)),
            }
        }
    }

    #[cfg(test)]
    impl Iterator for BufReadIter {
        type Item = Result<Vec<u8>, io::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            let mut buf = Vec::with_capacity(80);
            match self.reader.read_until(0xA, &mut buf) {
                Ok(len) => {
                    if len == 0 {
                        None
                    } else {
                        let last = buf.pop();
                        assert!(last.is_some());
                        assert!(!buf.is_empty());
                        Some(Ok(buf))
                    }
                }
                Err(err) => Some(Err(err.into())),
            }
        }
    }

    #[cfg(test)]
    #[derive(Error, Debug)]
    pub enum TransformError {
        #[error("{0:?}")]
        IoError(#[from] io::Error),
        #[error("{0:?}")]
        FromUtf8Error(#[from] FromUtf8Error),
    }

    #[cfg(test)]
    fn tranform(bytes: Result<Vec<u8>, io::Error>) -> Result<String, TransformError> {
        match bytes {
            Ok(bytes) => String::from_utf8(bytes).map_err(TransformError::from),
            Err(err) => Err(TransformError::from(err)),
        }
    }

    #[test]
    fn utf8_encoded_stream_to_multi_string_stream() {
        let stream = "apple\u{0A}grape\u{0A}banana\u{0A}";
        let stream = BufReadIter::from_iter(stream.bytes());
        let mut stream = super::Transformed::new(stream, tranform);

        assert_eq!("apple", stream.next().unwrap().unwrap().as_str());
        assert_eq!("grape", stream.next().unwrap().unwrap().as_str());
        assert_eq!("banana", stream.next().unwrap().unwrap().as_str());
        assert!(stream.next().is_none());
    }
}
