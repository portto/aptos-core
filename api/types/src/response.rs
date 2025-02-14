// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    mime_types::{BCS, JSON},
    Error, LedgerInfo,
};
use anyhow::Result;
use serde::Serialize;
use warp::{
    http::header::{HeaderValue, CONTENT_TYPE},
    hyper::StatusCode,
};

pub const X_APTOS_CHAIN_ID: &str = "X-Aptos-Chain-Id";
pub const X_APTOS_EPOCH: &str = "X-Aptos-Epoch";
pub const X_APTOS_LEDGER_VERSION: &str = "X-Aptos-Ledger-Version";
pub const X_APTOS_LEDGER_TIMESTAMP: &str = "X-Aptos-Ledger-TimestampUsec";

pub struct Response {
    pub ledger_info: LedgerInfo,
    pub body: Vec<u8>,
    pub is_bcs_response: bool,
}

impl Response {
    pub fn new<T: Serialize>(ledger_info: LedgerInfo, body: &T) -> Result<Self, Error> {
        Ok(Self {
            ledger_info,
            body: serde_json::to_vec(body)?,
            is_bcs_response: false,
        })
    }

    pub fn new_bcs<T: Serialize>(ledger_info: LedgerInfo, body: &T) -> Result<Self, Error> {
        Ok(Self {
            ledger_info,
            body: bcs::to_bytes(body).map_err(|_| {
                Error::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "data serialization error".to_string(),
                )
            })?,
            is_bcs_response: true,
        })
    }
}

impl warp::Reply for Response {
    fn into_response(self) -> warp::reply::Response {
        let mut res = warp::reply::Response::new(self.body.into());
        let headers = res.headers_mut();

        if self.is_bcs_response {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(BCS));
        } else {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(JSON));
        }
        headers.insert(X_APTOS_CHAIN_ID, (self.ledger_info.chain_id as u16).into());
        headers.insert(
            X_APTOS_LEDGER_VERSION,
            self.ledger_info.ledger_version.into(),
        );
        headers.insert(
            X_APTOS_LEDGER_TIMESTAMP,
            self.ledger_info.ledger_timestamp.into(),
        );
        headers.insert(X_APTOS_EPOCH, self.ledger_info.epoch.into());

        res
    }
}
