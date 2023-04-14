//! `Iterator<Item = Result<T, E>>`型に追加の制約を与えることで任意のストリームから任意のトークンの読み出しを実現するクレート。
//!
//! [抽象化](self::Stream)と[エラーに関する更なる追加の制約を与えるトレイト](self::SendSyncStream)と[単純で汎用のイテレータ実装](self::Transformed)で構成される。
mod r#impl;

pub use r#impl::Transformed;

use std::iter::{IntoIterator, Iterator};

/// 恒等関数
///
/// ```
/// use common_stream::{ident, Transformed};
///
/// let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
/// let s = Transformed::new(char::decode_utf16(s), ident);
///
/// assert_eq!(String::from("music"), s.map(Result::unwrap).collect::<String>());
/// ```
pub fn ident<X>(x: X) -> X {
    x
}

/// 抽象化のためのトレイト
pub trait Stream<TK, E>
where
    Self: Iterator<Item = Result<TK, E>>,
{
}

impl<T, TK, E> Stream<TK, E> for T
where
    E: std::error::Error,
    T: Iterator<Item = Result<TK, E>>,
{
}

/// `E`が[Send]と[Sync]を実装した[Stream](self::Stream)。
pub trait SendSyncStream<TK, E>
where
    Self: Iterator<Item = Result<TK, E>>,
{
}

impl<T, TK, E> SendSyncStream<TK, E> for T
where
    E: std::error::Error + Send + Sync + 'static,
    T: Iterator<Item = Result<TK, E>>,
{
}

/// プッシュバック機能を追加するトレイトです。
///
/// このトレイトのメソッドから戻ると次に読み込まれる値はプッシュバックされたトークンと同じになります。
pub trait UnRead<U> {
    /// 一つのトークンをプッシュバックします。
    ///
    /// # Parameters
    /// <dl>
    /// <dt>token</dt>
    /// <dd>プッシュバックするトークンです。</dd>
    /// </dl>
    fn unread(&mut self, token: U);

    /// 複数のトークンをプッシュバックします。
    ///
    /// # Parameters
    /// <dl>
    /// <dt>iter</dt>
    /// <dd>このイテレーターが返した個数だけ順序通りにトークンを返すようになります。</dd>
    /// </dl>
    fn unread_from_tokens(&mut self, iter: impl IntoIterator<Item = U>);
}

#[cfg(test)]
mod tests {
    use super::{ident, Transformed};
    use std::char::decode_utf16;

    #[test]
    fn unread_all() {
        use super::UnRead as _;

        let s = [0x006d, 0x0075, 0x0073, 0x0069, 0x0063].iter().cloned();
        let mut s = Transformed::new(decode_utf16(s), ident);
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
        let s = Transformed::new(decode_utf16(s), ident);

        assert_eq!(String::from("music"), s.map(Result::unwrap).collect::<String>());
    }

    #[test]
    fn stream_from_iter_infallible() {
        let s = Transformed::<_, char, _, std::convert::Infallible, _>::new("music".chars(), Result::Ok);

        assert_eq!(String::from("music"), s.map(Result::unwrap).collect::<String>());
    }

    #[cfg(test)]
    fn consume_send_sync_stream<TK, E>(_stream: impl super::SendSyncStream<TK, E>) {}

    #[test]
    fn pass_send_sync_stream_trait() {
        let s = Transformed::new(std::iter::empty::<Result<u8, std::convert::Infallible>>(), ident);
        consume_send_sync_stream(s);
    }
}
