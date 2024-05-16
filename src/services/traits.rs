// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};

/// Service methods.
///
/// A service is something that usually runs in its own thread, as a daemon, and
/// that communicates with client(s) by crossbeam channels. It accepts Inputs
/// and produces Events.
pub trait ProvidesService<I: core::fmt::Debug, E: core::fmt::Debug> {
    /// The sender side of the Input channel. Use this to send commands to the
    /// service.
    fn sender(&self) -> &Sender<I>;

    /// A convenience method to send Inputs to the service.
    fn send_input(&self, input: I) {
        if let Err(e) = self.sender().try_send(input) {
            eprintln!("While sending: {e:?}");
        }
    }

    /// The receiver side of the Event channel. Integrate this into a listener
    /// loop to respond to events.
    fn receiver(&self) -> &Receiver<E>;

    /// A convenience method to receive either Inputs or Events inside a
    /// crossbeam select loop.
    fn recv_operation<T>(
        oper: crossbeam_channel::SelectedOperation,
        r: &Receiver<T>,
    ) -> Result<T, crossbeam_channel::RecvError> {
        let input_result = oper.recv(r);
        if let Err(e) = input_result {
            eprintln!(
                "ProvidesService: While attempting to receive from {:?}: {}",
                *r, e
            );
        }
        input_result
    }
}
