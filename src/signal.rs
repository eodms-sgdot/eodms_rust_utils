use tokio::sync::broadcast;    
use tokio::sync::broadcast::{Receiver, Sender};   

// Adapted from https://github.com/Samoxive/tcp-proxy/blob/main/src/main.rs

#[derive(Clone,Debug)]
pub enum SignalType {
	Shutdown,
	Pause,
}

#[derive(Clone,Debug)]
pub struct Signal {
	sender: Sender<SignalType>,
}

impl Signal {
	pub fn new() -> Self {
		let (sender, _) = broadcast::channel(1);
		Self { sender }
	}

	pub fn subscribe(&self) -> Receiver<SignalType> {
		self.sender.subscribe()
	}

	pub fn shutdown(&self) {
		let _ = self.sender.send(SignalType::Shutdown);
	}

	pub fn pause(&self) {
		let _ = self.sender.send(SignalType::Pause);
	}
}

impl Default for Signal {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Duration;
	#[tokio::test]
	async fn signal() {
		let signal = Signal::new();
		let signal2 = signal.clone();
		let r = tokio::spawn(async move {
			run(signal2).await;
		});
		tokio::time::timeout(Duration::from_secs(3), async {
			tokio::time::sleep(Duration::from_secs(1)).await;
			signal.pause();
			tokio::time::sleep(Duration::from_secs(1)).await;
			signal.shutdown();
			r.await.unwrap();
		}).await.unwrap();
	}
	async fn run(signal: Signal) {
		let mut signal_receiver = signal.subscribe();
		loop {
			tokio::select! {
				signal = signal_receiver.recv() => {
					let signal = signal.unwrap();
					match signal {
						SignalType::Shutdown => {
							println!("Shutting down");
							return;
						},
						SignalType::Pause => {
							println!("Paused");
						},
					}
				},
			};
			tokio::time::sleep(Duration::from_secs(1)).await;
		}
	}
}
