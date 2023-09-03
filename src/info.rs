//! Relay metadata using NIP-11
/// Relay Info
use crate::config::Settings;
use serde::{Deserialize, Serialize};

pub const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
pub const UNIT: &str = "msats";

/// Limitations of the relay as specified in NIP-111
/// (This nip isn't finalized so may change)
#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Limitation {
    #[serde(skip_serializing_if = "Option::is_none")]
    payment_required: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    restricted_writes: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(unused)]
pub struct Fees {
    #[serde(skip_serializing_if = "Option::is_none")]
    admission: Option<Vec<Fee>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication: Option<Vec<Fee>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(unused)]
pub struct Fee {
    amount: u64,
    unit: String,
    // Cashu
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<PaymentMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kinds: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum PaymentMethod {
    Cashu { mints: Vec<String> },
}

impl PaymentMethod {
    pub fn cashu(mints: Vec<String>) -> Self {
        Self::Cashu { mints }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct RelayInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_nips: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limitation: Option<Limitation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fees: Option<Fees>,
}

/// Convert an Info configuration into public Relay Info
impl From<Settings> for RelayInfo {
    fn from(c: Settings) -> Self {
        let mut supported_nips = vec![1, 2, 9, 11, 12, 15, 16, 20, 22, 33, 40];

        if c.authorization.nip42_auth {
            supported_nips.push(42);
            supported_nips.sort();
        }

        let i = c.info;
        let p = c.pay_to_relay;
        let pc = c.pay_to_relay_by_cashu;

        let limitations = Limitation {
            payment_required: Some(p.enabled || pc.enabled),
            restricted_writes: Some(
                p.enabled
                    || pc.enabled
                    || c.verified_users.is_enabled()
                    || c.authorization.pubkey_whitelist.is_some()
                    || c.grpc.restricts_write,
            ),
        };

        let (mut payment_url, mut fees) = (None, None);
        if p.enabled || pc.enabled {
            let admission_fee = if p.enabled && p.admission_cost > 0 {
                Some(vec![Fee {
                    amount: p.admission_cost * 1000,
                    unit: UNIT.to_string(),
                    method: None,
                    kinds: None,
                }])
            } else {
                None
            };

            let mut post_fee = vec![];
            if p.enabled && p.cost_per_event > 0 {
                post_fee.push(Fee {
                    amount: p.cost_per_event * 1000,
                    unit: UNIT.to_string(),
                    method: None,
                    kinds: None,
                })
            }

            if pc.enabled {
                post_fee.push(Fee {
                    amount: pc.cost_per_event,
                    unit: pc.unit.to_string(),
                    method: Some(PaymentMethod::cashu(
                        pc.mints.iter().map(|m| m.as_str().to_owned()).collect(),
                    )),
                    kinds: pc.kinds.clone(),
                })
            }

            fees = Some(Fees {
                admission: admission_fee,
                publication: if post_fee.is_empty() {
                    None
                } else {
                    Some(post_fee)
                },
            });

            if p.enabled && i.relay_url.is_some() {
                payment_url = Some(format!(
                    "{}join",
                    i.relay_url.clone().unwrap().replace("ws", "http")
                ))
            }
        }

        RelayInfo {
            id: i.relay_url,
            name: i.name,
            description: i.description,
            pubkey: i.pubkey,
            contact: i.contact,
            supported_nips: Some(supported_nips),
            software: Some("https://git.sr.ht/~gheartsfield/nostr-rs-relay".to_owned()),
            version: CARGO_PKG_VERSION.map(std::borrow::ToOwned::to_owned),
            limitation: Some(limitations),
            payment_url,
            fees,
            icon: i.relay_icon,
        }
    }
}
