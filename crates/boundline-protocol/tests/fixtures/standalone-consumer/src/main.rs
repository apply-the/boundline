use boundline_protocol::{
    ContractLine, MutationRequestEnvelope, OperationId, ProtocolVersion, RequestDigest, RequestId,
    Revision,
};

fn main() {
    let request = MutationRequestEnvelope {
        protocol_version: ProtocolVersion::V1,
        contract_line: ContractLine::new("boundline.protocol"),
        operation: OperationId::new("consumer.check"),
        request_id: RequestId::new("request-001"),
        canonical_request_digest: RequestDigest::new("sha256:consumer"),
        expected_state_revision: Revision::new(0),
        payload: (),
    };
    std::hint::black_box(request);
}
