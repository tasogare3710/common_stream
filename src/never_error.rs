use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
#[error("invalid state")]
pub struct InvalidState;

/// エラーが発生する可能性のない`Iterator`のためのiterator adapter。
///
pub struct Iter<I, T>(I)
    where I: Iterator<Item=T>;

impl<I, T> Iter<I, T> where
    I: Iterator<Item=T>
{
    pub fn new(iter: I) -> Self {
        Self(iter)
    }
}

impl<I, T> Iterator for Iter<I, T> where
    I: Iterator<Item=T>
{
    type Item = Result<<I as Iterator>::Item, InvalidState>;

    // XXX: エラーが返ったらpanicしてはどうか？
    // XXX: `!`がまだ安定化してない
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use std::char;

    use super::super::{Stream, UnRead};

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
                buff: String::new(),
            }
        }
    }

    impl<I, E> Source<I, E> where
        I: Iterator<Item=Result<char, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        pub fn get(&mut self) -> Option<Result<char, E>> {
            let n = self.inner.next();
            if n.is_none() {
                n
            } else {
                let n = n.unwrap();
                Some(if n.is_err() {
                    n
                } else {
                    let ch = n.unwrap();
                    self.buff.push(ch);
                    Ok(ch)
                })
            }
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
    fn stream_by_chars_with_non_error_iter_adapter() {
        let s = Stream::new(super::Iter::new("music".chars()));

        assert_eq!(String::from("music"), s.map(it_is_ok).fold(String::new(), append));
    }

    #[test]
    fn stream_by_array_with_non_error_iter_adapter() {
        let s = super::Iter::new(['m', 'u', 's', 'i', 'c'].iter().cloned());
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

    #[test]
    fn stream_new_type_by_array_with_non_error_iter_adapter() {
        let s = super::Iter(['m', 'u', 's', 'i', 'c'].iter().cloned());
        let mut s = Source::new(s);

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
