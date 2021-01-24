mod r#impl;

pub use r#impl::{Builder, CommonStream};

use std::iter::{IntoIterator, Iterator};

/// 抽象化のためのトレイト
pub trait Stream<T, E>
where
    E: std::error::Error, // + Send + Sync + 'static
    Self: Iterator<Item = Result<T, E>>,
{
}

/// プッシュバックを可能とするトレイト。
pub trait UnRead<T> {
    /// 一つのトークンをプッシュバックします。
    /// このメソッドから返ると次に読み込まれる値はこのメソッドの引数`token`と同じになります。
    ///
    /// # Parameters
    /// ## `token`
    /// プッシュバックするトークンです。
    fn unread(&mut self, token: T);

    /// 複数のトークンをプッシュバックします。
    /// このメソッドから返るとそれ以降に読み込まれる値は`iter`が返したトークンと同じになります。
    ///
    /// # Parameters
    /// ## `iter`
    /// プッシュバックするトークンです。
    ///
    /// このイテレーターが返した個数だけ順序通りに値を返すようになります。
    fn unread_from_tokens(&mut self, iter: impl IntoIterator<Item = T>) {
        for c in iter {
            self.unread(c)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommonStream;
    use std::char;

    #[test]
    fn unread_all() {
        use super::UnRead as _;

        let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
        let mut s = CommonStream::new(char::decode_utf16(s), |x| x);
        let mut list = std::collections::VecDeque::<char>::new();

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

        s.unread_from_tokens(list);
        assert_eq!(
            String::from("cisum"),
            s.by_ref().map(Result::unwrap).collect::<String>()
        );

        assert!(s.next().is_none());
    }

    #[test]
    fn stream_from_iter_decode_identify() {
        let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
        let s = CommonStream::new(char::decode_utf16(s), |x| x);

        assert_eq!(String::from("music"), s.map(Result::unwrap).collect::<String>());
    }

    #[test]
    fn stream_from_iter_infallible() {
        let s = CommonStream::<_, char, _, std::convert::Infallible, _>::new("music".chars(), Result::Ok);

        assert_eq!(String::from("music"), s.map(Result::unwrap).collect::<String>());
    }
}
