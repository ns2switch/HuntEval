use std::io::{BufRead, Write};

use hunteval_domain::{MessageId, ProtocolVersion, RunId, UtcTimestamp};
use hunteval_protocol::{
    JsonlDecoder, MessageOrigin, ProtocolEnvelope, ProtocolPayload, ProtocolSession,
};

use crate::{
    ReferenceError, ReferenceTopology,
    framing::{MAX_PROTOCOL_LINE_BYTES, read_message, write_message},
    hunt,
};

pub(super) fn run_peer<R: BufRead, W: Write>(
    topology: ReferenceTopology,
    mut reader: R,
    writer: W,
) -> Result<(), ReferenceError> {
    let decoder = JsonlDecoder::new(MAX_PROTOCOL_LINE_BYTES)?;
    let started = read_message(&mut reader, decoder)?;
    require_runner_origin(&started)?;
    let mut session = ProtocolSession::new();
    session.accept(&started)?;
    let supported_version = ProtocolVersion::new(0, 3);
    let (seed, tables, max_agents) = match &started.payload {
        ProtocolPayload::RunStarted {
            supported_minimum,
            supported_maximum,
            tables,
            limits,
            seed,
            ..
        } if *supported_minimum <= supported_version
            && supported_version <= *supported_maximum
            && started.protocol_version == supported_version =>
        {
            (*seed, tables.clone(), limits.max_agents)
        }
        _ => return Err(ReferenceError::InvalidRunnerMessage),
    };

    let registration = topology.registration()?;
    registration.validate(max_agents)?;
    let mut peer = Peer {
        reader,
        writer,
        decoder,
        session,
        protocol_version: supported_version,
        run_id: started.run_id,
        timestamp: started.timestamp,
        seed,
        next_message: 1,
    };
    let registration_message = peer.send(
        None,
        ProtocolPayload::RegisterDeployment {
            selected_protocol_version: supported_version,
            deployment: registration,
        },
    )?;
    let accepted = peer.receive()?;
    match accepted.payload {
        ProtocolPayload::RegistrationAccepted {
            selected_protocol_version,
        } if selected_protocol_version == supported_version
            && accepted.caused_by_message_id.as_ref() == Some(&registration_message) => {}
        _ => return Err(ReferenceError::InvalidRunnerMessage),
    }

    hunt::execute(topology, &tables, &mut peer)?;
    let terminated = peer.receive()?;
    if !matches!(terminated.payload, ProtocolPayload::RunTerminated { .. }) {
        return Err(ReferenceError::InvalidRunnerMessage);
    }
    peer.session.finish()?;
    Ok(())
}

pub(super) struct Peer<R, W> {
    reader: R,
    writer: W,
    decoder: JsonlDecoder,
    session: ProtocolSession,
    protocol_version: ProtocolVersion,
    run_id: RunId,
    timestamp: UtcTimestamp,
    pub(super) seed: u64,
    next_message: u16,
}

impl<R: BufRead, W: Write> Peer<R, W> {
    pub(super) fn receive(&mut self) -> Result<ProtocolEnvelope, ReferenceError> {
        let message = read_message(&mut self.reader, self.decoder)?;
        require_runner_origin(&message)?;
        self.session.accept(&message)?;
        Ok(message)
    }

    pub(super) fn send(
        &mut self,
        caused_by_message_id: Option<MessageId>,
        payload: ProtocolPayload,
    ) -> Result<MessageId, ReferenceError> {
        if payload.origin() != MessageOrigin::Deployment {
            return Err(ReferenceError::InvalidRunnerMessage);
        }
        let message_id =
            MessageId::new(format!("deployment-{}-{:03}", self.seed, self.next_message))?;
        self.next_message = self
            .next_message
            .checked_add(1)
            .ok_or(ReferenceError::InvalidRunnerMessage)?;
        let envelope = ProtocolEnvelope {
            protocol_version: self.protocol_version,
            message_id: message_id.clone(),
            run_id: self.run_id.clone(),
            timestamp: self.timestamp,
            caused_by_message_id,
            payload,
        };
        self.session.accept(&envelope)?;
        write_message(&mut self.writer, &envelope)?;
        Ok(message_id)
    }
}

fn require_runner_origin(message: &ProtocolEnvelope) -> Result<(), ReferenceError> {
    if message.payload.origin() == MessageOrigin::Runner {
        Ok(())
    } else {
        Err(ReferenceError::InvalidRunnerMessage)
    }
}
