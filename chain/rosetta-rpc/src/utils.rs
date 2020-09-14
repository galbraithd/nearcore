use actix::Addr;
use futures::StreamExt;

use near_client::ViewClientActor;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, PartialEq, derive_more::AsRef, derive_more::From)]
pub(crate) struct BorshInHexString<T: BorshSerialize + BorshDeserialize>(T);

impl<T> BorshInHexString<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> paperclip::v2::schema::TypedData for BorshInHexString<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl<T> serde::Serialize for BorshInHexString<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(
            self.0.try_to_vec().expect("borsh serialization should never fail"),
        ))
    }
}

impl<'de, T> serde::Deserialize<'de> for BorshInHexString<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let blob = hex::decode(&<String as serde::Deserialize>::deserialize(deserializer)?)
            .map_err(|err| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Other(&format!(
                        "signed transaction could not be decoded due to: {:?}",
                        err
                    )),
                    &"base64-encoded transaction was expected",
                )
            })?;
        Ok(Self(T::try_from_slice(&blob).map_err(|err| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Other(&format!(
                    "signed transaction could not be deserialized due to: {:?}",
                    err
                )),
                &"a valid Borsh-serialized transaction was expected",
            )
        })?))
    }
}

#[derive(Debug, Clone, PartialEq, derive_more::AsRef, derive_more::From)]
#[as_ref(forward)]
pub(crate) struct BlobInHexString<T: AsRef<[u8]> + From<Vec<u8>>>(T);

impl<T> paperclip::v2::schema::TypedData for BlobInHexString<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl<T> BlobInHexString<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> serde::Serialize for BlobInHexString<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.as_ref()))
    }
}

impl<'de, T> serde::Deserialize<'de> for BlobInHexString<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::from(
            hex::decode(&<String as serde::Deserialize>::deserialize(deserializer)?).map_err(
                |err| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other(&format!(
                            "the value could not be decoded due to: {:?}",
                            err
                        )),
                        &"hex-encoded value was expected",
                    )
                },
            )?,
        )))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct SignedDiff<T>
where
    T: Copy + PartialEq,
{
    is_positive: bool,
    absolute_difference: T,
}

impl<T> paperclip::v2::schema::TypedData for SignedDiff<T>
where
    T: Copy + PartialEq,
{
    fn data_type() -> paperclip::v2::models::DataType {
        paperclip::v2::models::DataType::String
    }
}

impl From<u64> for SignedDiff<u64> {
    fn from(value: u64) -> Self {
        Self { is_positive: true, absolute_difference: value }
    }
}

impl From<u128> for SignedDiff<u128> {
    fn from(value: u128) -> Self {
        Self { is_positive: true, absolute_difference: value }
    }
}

impl<T> SignedDiff<T>
where
    T: Copy + PartialEq + std::ops::Sub<Output = T> + std::cmp::Ord,
{
    pub fn cmp(lhs: T, rhs: T) -> Self {
        if lhs <= rhs {
            Self { is_positive: true, absolute_difference: rhs - lhs }
        } else {
            Self { is_positive: false, absolute_difference: lhs - rhs }
        }
    }

    pub fn is_positive(&self) -> bool {
        self.is_positive
    }

    pub fn absolute_difference(&self) -> T {
        self.absolute_difference
    }
}

impl<T> std::fmt::Display for SignedDiff<T>
where
    T: Copy + PartialEq + std::string::ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if self.is_positive { "" } else { "-" },
            self.absolute_difference.to_string()
        )
    }
}

impl<T> std::ops::Neg for SignedDiff<T>
where
    T: Copy + PartialEq,
{
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.is_positive = !self.is_positive;
        self
    }
}

impl<T> serde::Serialize for SignedDiff<T>
where
    T: Copy + PartialEq + std::string::ToString,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> serde::Deserialize<'de> for SignedDiff<T>
where
    T: Copy + PartialEq + std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string_value = <String as serde::Deserialize>::deserialize(deserializer)?;
        let mut chars_value = string_value.chars();
        if let Some(first_char) = chars_value.next() {
            let (is_positive, absolute_difference) = if first_char == '-' {
                (false, chars_value.as_str())
            } else {
                (true, string_value.as_str())
            };
            Ok(Self {
                is_positive,
                absolute_difference: absolute_difference.parse().map_err(|err| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other(&format!(
                            "the value could not be decoded due to: {:?}",
                            err
                        )),
                        &"an integer value was expected in range of [-u128::MAX; +u128::MAX]",
                    )
                })?,
            })
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Other("empty value is not a valid number"),
                &"a non-empty value was expected",
            ))
        }
    }
}

fn get_liquid_balance_for_storage(
    mut account: near_primitives::account::Account,
    runtime_config: &near_runtime_configs::RuntimeConfig,
) -> near_primitives::types::Balance {
    account.amount = 0;
    near_runtime_configs::get_insufficient_storage_stake(&account, &runtime_config)
        .expect("get_insufficient_storage_stake never fails when state is consistent")
        .unwrap_or(0)
}

pub(crate) struct RosettaAccountBalances {
    pub liquid: near_primitives::types::Balance,
    pub liquid_for_storage: near_primitives::types::Balance,
    pub locked: near_primitives::types::Balance,
}

impl RosettaAccountBalances {
    pub fn zero() -> Self {
        Self { liquid: 0, liquid_for_storage: 0, locked: 0 }
    }

    pub fn from_account<T: Into<near_primitives::account::Account>>(
        account: T,
        runtime_config: &near_runtime_configs::RuntimeConfig,
    ) -> Self {
        let account = account.into();
        let amount = account.amount;
        let locked = account.locked;
        let liquid_for_storage = get_liquid_balance_for_storage(account, runtime_config);

        Self {
            liquid_for_storage,
            liquid: amount
                .checked_sub(liquid_for_storage)
                .expect("liquid balance for storage cannot be bigger than the total balance"),
            locked,
        }
    }
}

pub(crate) async fn query_accounts(
    account_ids: impl Iterator<Item = &near_primitives::types::AccountId>,
    block_id: &near_primitives::types::BlockReference,
    view_client_addr: &Addr<ViewClientActor>,
) -> Result<
    std::collections::HashMap<
        near_primitives::types::AccountId,
        near_primitives::views::AccountView,
    >,
    crate::errors::ErrorKind,
> {
    account_ids
        .map(|account_id| {
            async move {
                let query = near_client::Query::new(
                    block_id.clone(),
                    near_primitives::views::QueryRequest::ViewAccount {
                        account_id: account_id.clone(),
                    },
                );
                let account_info_response =
                    tokio::time::timeout(std::time::Duration::from_secs(10), async {
                        loop {
                            match view_client_addr.send(query.clone()).await? {
                                Ok(Some(query_response)) => return Ok(Some(query_response)),
                                Ok(None) => {}
                                // TODO: update this once we return structured errors in the
                                // view_client handlers
                                Err(err) => {
                                    if err.contains("does not exist") {
                                        return Ok(None);
                                    }
                                    return Err(crate::errors::ErrorKind::InternalError(err));
                                }
                            }
                            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
                        }
                    })
                    .await??;

                let kind = if let Some(account_info_response) = account_info_response {
                    account_info_response.kind
                } else {
                    return Ok(None);
                };

                match kind {
                    near_primitives::views::QueryResponseKind::ViewAccount(account_info) => {
                        Ok(Some((account_id.clone(), account_info)))
                    }
                    _ => Err(crate::errors::ErrorKind::InternalInvariantError(
                        "queried ViewAccount, but received something else.".to_string(),
                    )
                    .into()),
                }
            }
        })
        .collect::<futures::stream::FuturesUnordered<_>>()
        .collect::<Vec<
            Result<
                Option<(near_primitives::types::AccountId, near_primitives::views::AccountView)>,
                crate::errors::ErrorKind,
            >,
        >>()
        .await
        .into_iter()
        .filter_map(|account_info| account_info.transpose())
        .collect()
}
