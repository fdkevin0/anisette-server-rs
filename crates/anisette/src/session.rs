//! Transport-independent provisioning lifecycle; keep each instance on its backend worker.
use crate::protocol::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

/// Backends must bound execution and clean up partial starts on error.
/// No Send/Sync requirement: an emulator may be confined to its creating worker.
pub trait ProvisioningContext {
    type Error;
    /// Starts a backend exchange and returns its CPIM payload.
    ///
    /// # Errors
    /// Returns a backend error if provisioning cannot start; implementations must clean up partial starts.
    fn start(&mut self, spim: &[u8]) -> Result<Vec<u8>, Self::Error>;
    /// Completes the exchange and returns the resulting restorable state.
    ///
    /// # Errors
    /// Returns a backend error if completion, routing, or state export fails; pending handles must remain cancellable.
    fn finish(
        &mut self,
        ptm: &[u8],
        tk: &[u8],
        routing: Option<u64>,
    ) -> Result<Vec<u8>, Self::Error>;
    /// Must release a pending handle even after finish fails. Must not panic.
    fn cancel(&mut self);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Identifier,
    Start,
    End,
    Closed,
}

pub struct Session<B: ProvisioningContext> {
    phase: Phase,
    backend: Option<B>,
    pending: bool,
}

impl<B: ProvisioningContext> Default for Session<B> {
    fn default() -> Self {
        Self {
            phase: Phase::Identifier,
            backend: None,
            pending: false,
        }
    }
}

impl<B: ProvisioningContext> Session<B> {
    pub fn initial_message() -> Value {
        json!({"result":"GiveIdentifier"})
    }

    pub fn is_closed(&self) -> bool {
        self.phase == Phase::Closed
    }

    /// `create` is called exactly once, after a valid identifier, on the backend worker.
    /// A closed session never calls backend operations again.
    pub fn receive<F, E>(&mut self, bytes: &[u8], create: F) -> Option<Value>
    where
        F: FnOnce(Identifier) -> Result<B, E>,
    {
        let reply = match self.phase {
            Phase::Closed => return None,
            Phase::Identifier => {
                let identifier = parse_message::<IdentifierMessage>(bytes)
                    .and_then(|m| Identifier::decode(&m.identifier));
                match identifier {
                    Err(_) => self.fail("InvalidIdentifier", None),
                    Ok(id) => match create(id) {
                        Err(_) => self.fail(
                            "StartProvisioningError",
                            Some("backend initialization failed"),
                        ),
                        Ok(backend) => {
                            self.backend = Some(backend);
                            self.phase = Phase::Start;
                            json!({"result":"GiveStartProvisioningData"})
                        }
                    },
                }
            }
            Phase::Start => {
                let spim = parse_message::<StartMessage>(bytes)
                    .and_then(|m| decode_binary(&m.spim, MAX_STATE_BYTES));
                match spim {
                    Err(_) => {
                        self.fail("StartProvisioningError", Some("invalid provisioning data"))
                    }
                    Ok(spim) => {
                        let result = self
                            .backend
                            .as_mut()
                            .expect("backend established before start")
                            .start(&spim);
                        match result {
                            Err(_) => self
                                .fail("StartProvisioningError", Some("start provisioning failed")),
                            Ok(cpim) => {
                                self.pending = true;
                                if cpim.is_empty() || cpim.len() > MAX_STATE_BYTES {
                                    self.fail(
                                        "StartProvisioningError",
                                        Some("invalid backend output"),
                                    )
                                } else {
                                    self.phase = Phase::End;
                                    json!({"result":"GiveEndProvisioningData","cpim":STANDARD.encode(cpim)})
                                }
                            }
                        }
                    }
                }
            }
            Phase::End => {
                let end = parse_message::<EndMessage>(bytes).and_then(|m| {
                    Ok((
                        decode_binary(&m.ptm, MAX_STATE_BYTES)?,
                        decode_binary(&m.tk, MAX_STATE_BYTES)?,
                        m.routing,
                    ))
                });
                match end {
                    Err(_) => self.fail("EndProvisioningError", Some("invalid provisioning data")),
                    Ok((ptm, tk, routing)) => {
                        let result = self
                            .backend
                            .as_mut()
                            .expect("backend established before end")
                            .finish(&ptm, &tk, routing);
                        match result {
                            Err(_) => {
                                self.fail("EndProvisioningError", Some("end provisioning failed"))
                            }
                            Ok(state) => {
                                // A successful finish consumes the native provisioning handle.
                                self.pending = false;
                                if state.is_empty() || state.len() > MAX_STATE_BYTES {
                                    self.fail(
                                        "EndProvisioningError",
                                        Some("invalid backend output"),
                                    )
                                } else {
                                    self.close();
                                    json!({"result":"ProvisioningSuccess","adi_pb":STANDARD.encode(state)})
                                }
                            }
                        }
                    }
                }
            }
        };
        Some(reply)
    }

    pub fn timeout(&mut self) -> Option<Value> {
        if self.is_closed() {
            return None;
        }
        Some(self.fail("Timeout", None))
    }

    /// Called on disconnect/shutdown; Drop is a final guard for all exit paths.
    pub fn close(&mut self) {
        if self.pending {
            self.pending = false;
            if let Some(backend) = self.backend.as_mut() {
                backend.cancel();
            }
        }
        self.backend = None;
        self.phase = Phase::Closed;
    }

    fn fail(&mut self, result: &str, message: Option<&str>) -> Value {
        self.close();
        match message {
            Some(message) => json!({"result":result,"message":message}),
            None => json!({"result":result}),
        }
    }
}

impl<B: ProvisioningContext> Drop for Session<B> {
    fn drop(&mut self) {
        self.close();
    }
}
