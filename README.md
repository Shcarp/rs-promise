### rs-promise
A simple promise implementation in rust

### Usage
```rs
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
```