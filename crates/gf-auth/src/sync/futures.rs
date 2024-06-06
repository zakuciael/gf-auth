#[derive(Debug, Default)]
pub struct Mutex<T: ?Sized>(futures::lock::Mutex<T>);

#[derive(Debug)]
#[allow(dead_code)]
pub struct LockError;

impl<T> Mutex<T> {
  pub fn new(val: T) -> Self {
    Self(futures::lock::Mutex::new(val))
  }

  #[allow(dead_code)]
  pub async fn lock(&self) -> Result<futures::lock::MutexGuard<'_, T>, LockError> {
    let val = self.0.lock().await;
    Ok(val)
  }
}
