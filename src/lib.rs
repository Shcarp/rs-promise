use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Error, Debug)]
pub enum PromiseError {
    #[error("error: {0}")]
    PromiseResolve(String),
    #[error("error: {0}")]
    PromiseReject(String),
    #[error("Promise error")]
    Unknown,
}

#[derive(Debug)]
pub enum PromiseResult<T> {
    Resolved(T),
    Rejected(T),
}

unsafe impl<T: Send> Send for PromiseResult<T> {}
unsafe impl<T: Sync> Sync for PromiseResult<T> {}

#[derive(Debug, Clone)]
pub struct Promise<T: Send + Sync>(
    Sender<PromiseResult<T>>,
    Arc<Mutex<Receiver<PromiseResult<T>>>>,
);

unsafe impl<T: Send + Sync> Send for Promise<T> {}
unsafe impl<T: Send + Sync> Sync for Promise<T> {}
impl<T: Send + Sync> Unpin for Promise<T> {}

impl<T: Send + Sync + Debug> Promise<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(1);
        Promise(sender, Arc::new(Mutex::new(receiver)))
    }

    pub async fn resolve(&mut self, value: T) -> Result<(), PromiseError> {
        self.0
            .send(PromiseResult::Resolved(value))
            .await
            .or_else(|error| Err(PromiseError::PromiseResolve(error.to_string())))
    }

    pub async fn reject(&mut self, value: T) -> Result<(), PromiseError> {
        self.0
            .send(PromiseResult::Rejected(value))
            .await
            .or_else(|error| Err(PromiseError::PromiseReject(error.to_string())))
    }
}

impl<T: Send + Sync + Clone> Future for Promise<T> {
    type Output = PromiseResult<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.1.lock().unwrap().try_recv() {
            Ok(value) => Poll::Ready(value),
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let promise = Promise::<String>::new();
        let mut promise_clone = promise.clone();
        let send_data = "111";
        tokio::spawn(async move {
            promise_clone.resolve(send_data.to_string()).await.unwrap();
            promise_clone.reject(send_data.to_string()).await.unwrap();
        });

        if let PromiseResult::Resolved(value) = promise.clone().await {
            assert_eq!(value, send_data);
        } else {
            panic!("error");
        }

        if let PromiseResult::Rejected(value) = promise.clone().await {
            assert_eq!(value, send_data);
        } else {
            panic!("error");
        }
    }
}
