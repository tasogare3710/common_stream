use crate::Stream;
use std::{collections::VecDeque, iter::Iterator, marker::PhantomData};

pub struct CommonStream<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    iter: I,
    decode: F,
    backing_store: VecDeque<U>,
    phantom: PhantomData<Result<U, E>>,
}

impl<I, T, U, E, F> crate::UnRead<U> for CommonStream<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    fn unread(&mut self, token: U) {
        self.backing_store.push_front(token);
    }
}

impl<I, T, U, E, F> Iterator for CommonStream<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.backing_store.is_empty() {
            self.iter.next().map(|it| (self.decode)(it))
        } else {
            self.backing_store.pop_front().map(Ok)
        }
    }
}

impl<I, T, U, E, F> Stream<U, E> for CommonStream<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
    E: std::error::Error + Send + Sync + 'static,
{
}

#[derive(Debug)]
pub struct Builder<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    iter: Option<I>,
    decode: Option<F>,
    backing_store: Option<VecDeque<U>>,
}

impl<I, T, U, E, F> Builder<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    pub fn build(self) -> CommonStream<I, T, U, E, F> {
        if let Builder {
            iter: Some(iter),
            decode: Some(decode),
            backing_store,
        } = self
        {
            CommonStream {
                iter,
                decode,
                backing_store: backing_store.unwrap_or_else(VecDeque::new),
                phantom: Default::default(),
            }
        } else {
            // XXX: エラーを返す必要がある
            panic!();
        }
    }

    pub fn iter(mut self, iter: I) -> Self {
        self.iter.replace(iter);
        self
    }

    pub fn decode(mut self, decode: F) -> Self {
        self.decode.replace(decode);
        self
    }

    pub fn backing_store(mut self, backing_store: VecDeque<U>) -> Self {
        self.backing_store.replace(backing_store);
        self
    }
}

impl<I, T, U, E, F> CommonStream<I, T, U, E, F>
where
    I: Iterator<Item = T>,
    F: FnMut(T) -> Result<U, E>,
{
    pub fn new(iter: I, decode: F) -> Self {
        Self {
            iter,
            decode,
            backing_store: Default::default(),
            phantom: Default::default(),
        }
    }

    pub fn build() -> Builder<I, T, U, E, F> {
        Builder {
            iter: None,
            decode: None,
            backing_store: None,
        }
    }
}
