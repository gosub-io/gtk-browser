use std::error::Error;
use std::future::Future;
use std::{cmp, mem};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_core::Stream as FStream;
use bytes::Bytes;

const DEFAULT_BUF_SIZE: usize = 1 * 1024;


type Stream = dyn FStream<Item=anyhow::Result<Bytes>>;


/// Converts the Error type (E) to an anyhow::Error (nothing more)
pub struct AsyncStreamWrap<S: FStream<Item=Result<Bytes, E>>, E: Error + Send + Sync + 'static>(S);

impl <S: FStream<Item=Result<Bytes, E>>, E: Error + Send + Sync + 'static> FStream for AsyncStreamWrap<S, E> {
    type Item = anyhow::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream =  unsafe { self.map_unchecked_mut(|this| &mut this.0) };

        match stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(val) => Poll::Ready(val.map(|v| v.map_err(|e| anyhow::Error::new(e)))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}


///
pub struct AsyncStream {
    stream: Pin<Box<Stream>>,
    /// Optional length given to the stream. Can be None when the stream length is not known (beforehand)
    len: Option<usize>,
}

impl AsyncStream {
    pub fn to_vec(self) -> ToVec {
        let (min_size, size) = self.stream.size_hint();
        let size = size.or(self.len).unwrap_or(cmp::max(DEFAULT_BUF_SIZE, min_size));

        let buf = Vec::with_capacity(size);

        ToVec {
            stream: self.stream,
            buf,
        }
    }

    pub fn new<E: Error + Send + Sync + 'static>(stream: impl FStream<Item=Result<Bytes, E>> + 'static, len: Option<usize>) -> Self {
        Self {
            stream: Box::pin(AsyncStreamWrap(stream)),
            len,
        }
    }
}

/// 
pub struct ToVec {
    stream: Pin<Box<Stream>>,
    buf: Vec<u8>, //TODO: this should be bytes and then in the end it can be copied to a single Vec<u8>
}

impl Future for ToVec {
    type Output = anyhow::Result<Vec<u8>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let stream = self.stream.as_mut();

        let poll = stream.poll_next(cx);

        let chunk = match poll {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(chunk) => chunk,
        };

        match chunk {
            Some(Ok(chunk)) => self.buf.extend_from_slice(&chunk.to_vec()),
            Some(Err(e)) => return Poll::Ready(Err(e)),
            None => return Poll::Ready(Ok(mem::take(&mut self.buf)))
        }


        Poll::Pending
    }
}