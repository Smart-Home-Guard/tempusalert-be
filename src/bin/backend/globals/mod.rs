pub mod channels {
    use once_cell::sync::Lazy;
    use tokio::sync::{broadcast::{Sender, Receiver}, Mutex};

    pub type Publisher<T> = Sender<T>;
    pub type Subscriber<T> = Receiver<T>;

    #[derive(Clone)]
    pub enum UserEventKind {
        JOIN = 0,
        CANCEL = 1,
    }

    #[derive(Clone)]
    pub struct UserEvent {
        pub kind: UserEventKind,
        pub client_id: String,
    }

    static USER_CHANNEL: Lazy<Mutex<Sender<UserEvent>>> = Lazy::new(|| Mutex::new(tokio::sync::broadcast::channel::<UserEvent>(100).0));

    pub async fn get_user_publisher() -> Publisher<UserEvent> {
        let channel = USER_CHANNEL.lock().await;
        channel.clone()
    }

    pub async fn get_user_subscriber() -> Subscriber<UserEvent> {
        let channel = USER_CHANNEL.lock().await;
        channel.subscribe()
    }
}