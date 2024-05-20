#![cfg(test)]
//! Reference data types.
//!
//! This module provides some reference instantiations of various data types which have an external,
//! language-independent interface (e.g. serialization or commitment scheme). Ports of the sequencer
//! to other languages, as well as downstream packages written in other languages, can use these
//! references objects and their known serializations and commitments to check that their
//! implementations are compatible with this reference implementation.
//!
//! The serialized form of each reference object, in JSON form, is available in the `data` directory
//! of the repo for this crate. These JSON files are compiled into the crate binary and checked
//! against the reference objects in this module to prevent unintentional changes to the
//! serialization format. Thus, these JSON objects can be used as test vectors for a port of the
//! serialiation scheme to another language or framework.
//!
//! To get the byte representation or U256 representation of a commitment for testing in other
//! packages, run the tests and look for "commitment bytes" or "commitment U256" in the logs.
//!
//! These tests may fail if you make a breaking change to a commitment scheme, serialization, etc.
//! If this happens, be sure you _want_ to break the API, and, if so, simply replace the relevant
//! constant in this module with the "actual" value that can be found in the logs of the failing
//! test.

use crate::{
    block::tables::NameSpaceTable, state::FeeInfo, ChainConfig, FeeAccount, Header, L1BlockInfo,
    Payload, Transaction, TxTableEntryWord, ValidatedState,
};
use async_compatibility_layer::logging::{setup_backtrace, setup_logging};
use committable::Committable;
use hotshot_types::traits::{
    block_contents::vid_commitment, block_contents::TestableBlock,
    signature_key::BuilderSignatureKey, BlockPayload, EncodeBytes,
};
use jf_merkle_tree::MerkleTreeScheme;
use sequencer_utils::commitment_to_u256;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::str::FromStr;

fn reference_ns_table() -> NameSpaceTable<TxTableEntryWord> {
    NameSpaceTable::from_bytes([0; 48])
}

const REFERENCE_NS_TABLE_COMMITMENT: &str = "NSTABLE~GL-lEBAwNZDldxDpySRZQChNnmn9vNzdIAL8W9ENOuh_";

fn reference_l1_block() -> L1BlockInfo {
    L1BlockInfo {
        number: 123,
        timestamp: 0x456.into(),
        hash: "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap(),
    }
}

const REFERENCE_L1_BLOCK_COMMITMENT: &str = "L1BLOCK~4HpzluLK2Isz3RdPNvNrDAyQcWOF2c9JeLZzVNLmfpQ9";

fn reference_chain_config() -> ChainConfig {
    ChainConfig {
        chain_id: 0x8a19.into(),
        max_block_size: 10240,
        base_fee: 0.into(),
        fee_contract: Some(Default::default()),
        fee_recipient: Default::default(),
    }
}

const REFERENCE_CHAIN_CONFIG_COMMITMENT: &str =
    "CHAIN_CONFIG~L6HmMktJbvnEGgpmRrsiYvQmIBstSj9UtDM7eNFFqYFO";

fn reference_fee_info() -> FeeInfo {
    FeeInfo::new(
        FeeAccount::from_str("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap(),
        0,
    )
}

const REFERENCE_FEE_INFO_COMMITMENT: &str = "FEE_INFO~xCCeTjJClBtwtOUrnAmT65LNTQGceuyjSJHUFfX6VRXR";

fn reference_header() -> Header {
    let builder_key = FeeAccount::generated_from_seed_indexed(Default::default(), 0).1;
    let fee_info = reference_fee_info();
    let ns_table = reference_ns_table();
    let payload = <Payload<_> as TestableBlock>::genesis();
    let payload_commitment = vid_commitment(&payload.encode(), 1);
    let builder_commitment = payload.builder_commitment(&ns_table);
    let builder_signature = FeeAccount::sign_fee(
        &builder_key,
        fee_info.amount().as_u64().unwrap(),
        &ns_table,
        &payload_commitment,
    )
    .unwrap();

    let state = ValidatedState::default();

    Header {
        height: 42,
        timestamp: 789,
        l1_head: 124,
        l1_finalized: Some(reference_l1_block()),
        payload_commitment,
        builder_commitment,
        ns_table,
        block_merkle_tree_root: state.block_merkle_tree.commitment(),
        fee_merkle_tree_root: state.fee_merkle_tree.commitment(),
        fee_info,
        chain_config: reference_chain_config().into(),
        builder_signature: Some(builder_signature),
    }
}

const REFERENCE_HEADER_COMMITMENT: &str = "BLOCK~6Ol30XYkdKaNFXw0QAkcif18Lk8V8qkC4M81qTlwL707";

fn reference_transaction() -> Transaction {
    let payload: [u8; 1024] = std::array::from_fn(|i| (i % (u8::MAX as usize)) as u8);
    Transaction::new(12648430.into(), payload.to_vec())
}

const REFERENCE_TRANSACTION_COMMITMENT: &str = "TX~jmYCutMVgguprgpZHywPwkehwXfibQx951gh4LSLmfwp";

macro_rules! load_reference {
    ($name:expr) => {
        serde_json::from_str(include_str!(std::concat!("../../data/", $name, ".json"))).unwrap()
    };
}

fn reference_test<T: Committable + Serialize + DeserializeOwned>(
    serialized: Value,
    reference: T,
    commitment: &str,
) {
    setup_logging();
    setup_backtrace();

    // Check that the reference object matches the expected serialized form.
    let actual = serde_json::to_value(&reference).unwrap();
    assert_eq!(
        serialized,
        actual,
        r#"
Serialized object does not match expected JSON. If you intended to make a breaking change to the
API, you may replace the reference JSON file in /data with the "actual" output below. Otherwise,
revert your changes which have caused a change in the serialization of this data structure.

Expected:
{}

Actual:
{}
"#,
        serde_json::to_string_pretty(&serialized).unwrap(),
        serde_json::to_string_pretty(&actual).unwrap()
    );

    // Check that we can deserialize from the reference JSON object.
    let parsed: T = serde_json::from_value(serialized).unwrap();
    assert_eq!(
        reference.commit(),
        parsed.commit(),
        "Reference object commitment does not match commitment of parsed JSON. This is indicative of
        inconsistency or non-determinism in the commitment scheme.",
    );

    // Print information about the commitment that might be useful in generating tests for other
    // languages.
    let actual = reference.commit();
    let bytes: &[u8] = actual.as_ref();
    let u256 = commitment_to_u256(actual);
    tracing::info!("actual commitment: {}", actual);
    tracing::info!("commitment bytes: {:?}", bytes);
    tracing::info!("commitment U256: {}", u256);

    // Check that the reference object matches the expected serialized object.
    let expected = commitment.parse().unwrap();
    assert_eq!(
        actual, expected,
        r#"
Commitment of serialized object does not match commitment of reference object. If you intended to
make a breaking change to the API, you may replace the reference commitment constant in the
reference_tests module with the "actual" commitment below. Otherwise, revert your changes which
have caused a change to the commitment scheme.

Expected: {expected}
Actual: {actual}
"#
    );
}

#[test]
fn test_reference_ns_table() {
    reference_test(
        load_reference!("ns_table"),
        reference_ns_table(),
        REFERENCE_NS_TABLE_COMMITMENT,
    );
}

#[test]
fn test_reference_l1_block() {
    reference_test(
        load_reference!("l1_block"),
        reference_l1_block(),
        REFERENCE_L1_BLOCK_COMMITMENT,
    );
}

#[test]
fn test_reference_chain_config() {
    reference_test(
        load_reference!("chain_config"),
        reference_chain_config(),
        REFERENCE_CHAIN_CONFIG_COMMITMENT,
    );
}

#[test]
fn test_reference_fee_info() {
    reference_test(
        load_reference!("fee_info"),
        reference_fee_info(),
        REFERENCE_FEE_INFO_COMMITMENT,
    );
}

#[test]
fn test_reference_header() {
    reference_test(
        load_reference!("header"),
        reference_header(),
        REFERENCE_HEADER_COMMITMENT,
    );
}

#[test]
fn test_reference_transaction() {
    reference_test(
        load_reference!("transaction"),
        reference_transaction(),
        REFERENCE_TRANSACTION_COMMITMENT,
    );
}
