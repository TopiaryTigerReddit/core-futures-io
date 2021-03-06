use crate::{AsyncRead, AsyncWrite};
use _tokio::io::{AsyncRead as TAsyncRead, AsyncWrite as TAsyncWrite, Error, ErrorKind};
use core::{
    pin::Pin,
    task::{Context, Poll},
};

impl<T: Unpin + TAsyncRead> AsyncRead for Compat<T> {
    type Error = Error;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        TAsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
    }
}

impl<T: Unpin + TAsyncWrite> AsyncWrite for Compat<T> {
    type WriteError = Error;
    type FlushError = Error;
    type CloseError = Error;

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::WriteError>> {
        TAsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::FlushError>> {
        TAsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::CloseError>> {
        TAsyncWrite::poll_shutdown(Pin::new(&mut self.0), cx)
    }
}

pub struct Compat<T>(T);

impl<T> Compat<T> {
    pub fn new(input: T) -> Self {
        Compat(input)
    }
}

impl<T: Unpin + AsyncWrite> TAsyncWrite for Compat<T>
where
    T::WriteError: Into<Box<dyn std::error::Error + Sync + Send>>,
    T::FlushError: Into<Box<dyn std::error::Error + Sync + Send>>,
    T::CloseError: Into<Box<dyn std::error::Error + Sync + Send>>,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_close(Pin::new(&mut self.0), cx)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

impl<T: Unpin + AsyncRead> TAsyncRead for Compat<T>
where
    T::Error: Into<Box<dyn std::error::Error + Sync + Send>>,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
