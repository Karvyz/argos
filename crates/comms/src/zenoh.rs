use std::marker::PhantomData;

use zenoh::{
    Config, Session,
    bytes::ZBytes,
    handlers::FifoChannelHandler,
    pubsub::{Publisher as ZenohPublisher, Subscriber as ZenohSubscriber},
    sample::Sample,
};

use crate::{Error, Topic};

#[derive(Clone)]
pub struct Comms {
    session: Session,
}

impl Comms {
    pub async fn open() -> Result<Self, Error> {
        zenoh::init_log_from_env_or("error");
        let session = zenoh::open(Config::default()).await?;
        Ok(Self { session })
    }

    pub async fn publisher<T: Topic>(&self) -> Result<Publisher<T>, Error> {
        let inner = self.session.declare_publisher(T::KEY).await?;
        Ok(Publisher {
            inner,
            topic: PhantomData,
        })
    }

    pub async fn subscriber<T: Topic>(&self) -> Result<Subscriber<T>, Error> {
        let inner = self.session.declare_subscriber(T::KEY).await?;
        Ok(Subscriber {
            inner,
            topic: PhantomData,
        })
    }
}

pub struct Publisher<T: Topic> {
    inner: ZenohPublisher<'static>,
    topic: PhantomData<T>,
}

impl<T: Topic> Publisher<T> {
    pub async fn send(&self, message: T::Message) -> Result<(), Error> {
        self.inner.put(ZBytes::from(T::encode(&message))).await?;
        Ok(())
    }
}

pub struct Subscriber<T: Topic> {
    inner: ZenohSubscriber<FifoChannelHandler<Sample>>,
    topic: PhantomData<T>,
}

impl<T: Topic> Subscriber<T> {
    pub async fn recv(&self) -> Result<T::Message, Error> {
        let sample = self.inner.recv_async().await?;
        T::decode(&sample.payload().to_bytes())
    }
}
