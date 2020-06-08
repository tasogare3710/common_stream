pub mod never_error;

use std::{
    collections::VecDeque,
    iter::IntoIterator,
};

/// プッシュバックを可能とするトレイト。
pub trait UnRead<C> {
    /// 一つのチャンクをプッシュバックします。
    /// このメソッドから返ると次に読み込まれる値はこのメソッドの引数`chunk`と同じになります。
    ///
    /// # Parameters
    /// ## `chunk`
    /// プッシュバックするチャンクです。
    fn unread(&mut self, chunk: C);

    /// 複数のチャンクをプッシュバックします。
    /// このメソッドから返るとそれ以降に読み込まれる値は`iter`が返したチャンクと同じになります。
    ///
    /// # Parameters
    /// ## `iter`
    /// プッシュバックするチャンクです。
    ///
    /// このイテレーターが返した個数だけ順序通りに値を返すようになります。
    fn unread_from_chunks(&mut self, iter: impl IntoIterator<Item=C>) {
        for c in iter { self.unread(c) }
    }
}

pub struct Stream<I, C, E> where
    I: Iterator<Item=Result<C, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    inner: I,
    backing_store: VecDeque<C>,
    _marker: ::std::marker::PhantomData<E>,
}

impl<I, C, E> Stream<I, C, E> where
    I: Iterator<Item=Result<C, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn new(inner: I) -> Self {
        Stream { inner, backing_store: VecDeque::new(), _marker: ::std::marker::PhantomData }
    }
}

impl<I, C, E> UnRead<C> for Stream<I, C, E> where
    I: Iterator<Item=Result<C, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    fn unread(&mut self, chunk: C) {
        self.backing_store.push_front(chunk);
    }
}

impl<I, C, E> Iterator for Stream<I, C, E> where
    I: Iterator<Item=Result<C, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<C, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.backing_store.pop_front().map_or_else(|| self.inner.next(), |b| Some(Ok(b)))
    }
}

#[cfg(test)]
mod tests {
    use std::char;

    use super::{
        never_error,
        Stream,
        UnRead,
    };

    struct Source<I, E> where
        I: Iterator<Item=Result<char, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        inner: Stream<I, char, E>,
        buff: String,
    }

    impl<I, E> Source<I, E> where
        I: Iterator<Item=Result<char, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        pub fn new(iter: I) -> Self {
            Source {
                inner: Stream::new(iter),
                buff: String::default(),
            }
        }
    }

    impl<I, E> Source<I, E> where
        I: Iterator<Item=Result<char, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        pub fn get(&mut self) -> Option<Result<char, E>> {
            self.inner.next().map(|item| {
                item.map(|ch| {
                    self.buff.push(ch);
                    ch
                })
            })
        }

        pub fn unget(&mut self, token: char) {
            self.inner.unread(token)
        }
    }

    fn append(mut acc: String, ch: char) -> String {
        acc.push(ch);
        acc
    }

    fn it_is_ok<T, E>(it: Result<T, E>) -> T where
        E: std::error::Error + Send + Sync + 'static,
    {
        it.ok().unwrap()
    }

    #[test]
    fn unread_all() {
        use std::collections::VecDeque;

        let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
        let mut s = Stream::new(char::decode_utf16(s));
        let mut list = VecDeque::<char>::new();

        let ch = s.next().map(Result::ok).unwrap().unwrap();
        assert_eq!('m', ch);
        list.push_back(ch);

        let ch = s.next().map(Result::ok).unwrap().unwrap();
        assert_eq!('u', ch);
        list.push_back(ch);

        let ch = s.next().map(Result::ok).unwrap().unwrap();
        assert_eq!('s', ch);
        list.push_back(ch);

        let ch = s.next().map(Result::ok).unwrap().unwrap();
        assert_eq!('i', ch);
        list.push_back(ch);

        let ch = s.next().map(Result::ok).unwrap().unwrap();
        assert_eq!('c', ch);
        list.push_back(ch);

        s.unread_from_chunks(list);
        assert_eq!(String::from("cisum"), s.by_ref().map(it_is_ok).fold(String::new(), append));

        assert_eq!(None, s.next());
    }

    #[test]
    fn stream_by_decode_utf16() {
        let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
        let s = Stream::new(char::decode_utf16(s));

        assert_eq!(String::from("music"), s.map(it_is_ok).fold(String::new(), append));
    }

    #[test]
    fn stream_by_chars_with_non_error_iter_adapter() {
        let s = Stream::new(never_error::Iter::new("music".chars()));

        assert_eq!(String::from("music"), s.map(it_is_ok).fold(String::new(), append));
    }

    #[test]
    fn stream_by_array_with_non_error_iter_adapter() {
        let s = never_error::Iter::new(['m', 'u', 's', 'i', 'c'].iter().cloned());
        let mut s = Stream::new(s);

        let first_char = s.next();
        assert_eq!(Some(Ok('m')), first_char);

        s.unread(first_char.map(Result::ok).unwrap().unwrap());

        assert_eq!(Some(Ok('m')), s.next());
        assert_eq!(Some(Ok('u')), s.next());
        assert_eq!(Some(Ok('s')), s.next());
        assert_eq!(Some(Ok('i')), s.next());
        assert_eq!(Some(Ok('c')), s.next());
        assert_eq!(None, s.next());
    }

    #[test]
    fn stream_new_type_by_decode_utf16() {
        let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
        let mut s = Source::new(char::decode_utf16(s));

        let first_char = s.get();
        assert_eq!(Some(Ok('m')), first_char);

        s.unget(first_char.map(Result::ok).unwrap().unwrap());

        assert_eq!(Some(Ok('m')), s.get());
        assert_eq!(Some(Ok('u')), s.get());
        assert_eq!(Some(Ok('s')), s.get());
        assert_eq!(Some(Ok('i')), s.get());
        assert_eq!(Some(Ok('c')), s.get());
        assert_eq!(None, s.get());
    }
}
