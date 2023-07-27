use serde::{Deserialize, Serialize};
use serde_json::json;
use server::rpc_core::error::RpcError;
use starknet_in_rust::services::api::contract_classes::compiled_class::CompiledClass::Deprecated;
use std::collections::HashMap;

use super::abi_entry::{AbiEntry, AbiEntryType};
use super::FeltHex;
use crate::api::json_rpc::error::ApiError;
use crate::api::json_rpc::RpcResult;

use crate::api::serde_helpers::base_64_gzipped_json_string::deserialize_to_serde_json_value_with_keys_ordered_in_alphabetical_order;
use starknet_in_rust::SierraContractClass as ImportedSierraContractClass;
use starknet_types::contract_class::ContractClass as TypesContractClass;
use starknet_types::felt::Felt;
use starknet_types::starknet_api::state::EntryPointType;
use starknet_types::starknet_api::state::{EntryPoint, FunctionIndex};

// TODO: move to types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ContractClass {
    Cairo0(DeprecatedContractClass),
    Sierra(SierraContractClass),
}

impl TryFrom<TypesContractClass> for ContractClass {
    type Error = ApiError;

    fn try_from(value: TypesContractClass) -> RpcResult<Self> {
        match value {
            TypesContractClass::Cairo0(value) => Ok(ContractClass::Cairo0(value.try_into()?)),
            TypesContractClass::Cairo1(value) => Ok(ContractClass::Sierra(value.try_into()?)),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct SierraContractClass {
    pub sierra_program: Vec<FeltHex>,
    pub contract_class_version: String,
    pub entry_points_by_type: HashMap<EntryPointType, Vec<EntryPoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<String>,
}

impl TryFrom<ImportedSierraContractClass> for SierraContractClass {
    type Error = ApiError;
    fn try_from(value: ImportedSierraContractClass) -> RpcResult<Self> {
        // let asd: Vec<RpcResult<FeltHex>> = value
        //     .sierra_program
        //     .into_iter()
        //     .map(|el| {
        //         let hex_str = format!("{:#x}", el.value);
        //         match Felt::from_prefixed_hex_str(&hex_str) {
        //             Ok(val) => Ok(FeltHex(val)),
        //             Err(err) => Err(val.into()),
        //         }
        //     })
        //     .collect();

        let sierra_program: Vec<FeltHex> =
            serde_json::from_str(&serde_json::to_string(&value.sierra_program)?)?;
        let mut map: HashMap<EntryPointType, Vec<EntryPoint>> = HashMap::new();

        for entry in value.entry_points_by_type.constructor {
            let selector = serde_json::from_str(&serde_json::to_string(&entry.selector)?)?;
            let function_idx = FunctionIndex(entry.function_idx);
            let con = EntryPointType::Constructor;
            match map.get_mut(&con) {
                Some(val) => val.push(EntryPoint { function_idx, selector }),
                None => map.insert(
                    EntryPointType::Constructor,
                    vec![EntryPoint { selector, function_idx }],
                ),
            }
        }

        for entry in value.entry_points_by_type.external {
            let selector = serde_json::from_str(&serde_json::to_string(&entry.selector)?)?;
            let function_idx = FunctionIndex(entry.function_idx);

            match map.get_mut(&EntryPointType::External) {
                Some(val) => val.push(EntryPoint { function_idx, selector }),
                None => map
                    .insert(EntryPointType::External, vec![EntryPoint { selector, function_idx }]),
            }
        }

        for entry in value.entry_points_by_type.l1_handler {
            let selector = serde_json::from_str(&serde_json::to_string(&entry.selector)?)?;
            let function_idx = FunctionIndex(entry.function_idx);

            match map.get_mut(&EntryPointType::L1Handler) {
                Some(val) => val.push(EntryPoint { function_idx, selector }),
                None => map
                    .insert(EntryPointType::L1Handler, vec![EntryPoint { selector, function_idx }]),
            }
        }

        Ok(Self {
            sierra_program,
            contract_class_version: value.contract_class_version,
            entry_points_by_type: map,
            abi: value.abi.map(|contract| contract.json()),
        })
        //let abi =
        // Self {
        //     sierra_program: value
        //         .sierra_program
        //         .into_vec()
        //         .into_iter(|el| serde_json::to_string(el)),
        //     abi: value.abi,
        // }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct DeprecatedContractClass {
    pub abi: Vec<ContractClassAbiEntryWithType>,
    /// A base64 encoding of the gzip-compressed JSON representation of program.
    #[serde(
        deserialize_with = "deserialize_to_serde_json_value_with_keys_ordered_in_alphabetical_order"
    )]
    pub program: serde_json::Value,
    /// The selector of each entry point is a unique identifier in the program.
    pub entry_points_by_type: HashMap<
        starknet_types::starknet_api::deprecated_contract_class::EntryPointType,
        Vec<starknet_types::starknet_api::deprecated_contract_class::EntryPoint>,
    >,
}

// impl TryFrom<Cairo0ContractClass> for DeprecatedContractClass {}

impl TryFrom<DeprecatedContractClass> for TypesContractClass {
    type Error = ApiError;

    fn try_from(value: DeprecatedContractClass) -> RpcResult<Self> {
        let abi_json = serde_json::to_value(value.abi).map_err(|_| {
            ApiError::RpcError(RpcError::invalid_params("abi: Unable to parse to JSON"))
        })?;
        let entry_points_json = serde_json::to_value(value.entry_points_by_type).map_err(|_| {
            ApiError::RpcError(RpcError::invalid_params(
                "entry_points_by_type: Unable to parse to JSON",
            ))
        })?;

        Ok(TypesContractClass::Cairo0(starknet_types::contract_class::Cairo0ContractClass::Json(
            json!({
                "program": value.program,
                "abi": abi_json,
                "entry_points_by_type": entry_points_json,
            }),
        )))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct ContractClassAbiEntryWithType {
    #[serde(flatten)]
    pub entry: AbiEntry,
    pub r#type: AbiEntryType,
}

#[cfg(test)]
mod tests {
    use starknet_types::felt::Felt;

    use crate::api::models::abi_entry::FunctionAbiEntry;

    #[test]
    fn deserialize_contract_class_abi_entry_with_type() {
        let json_str = r#"{
            "inputs": [],
            "name": "getPublicKey",
            "outputs": [
                {
                    "name": "publicKey",
                    "type": "felt"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        }"#;

        let obj = serde_json::from_str::<super::ContractClassAbiEntryWithType>(json_str).unwrap();
        assert_eq!(obj.r#type, super::AbiEntryType::Function);
        assert_eq!(
            obj.entry,
            super::AbiEntry::Function(FunctionAbiEntry {
                name: "getPublicKey".to_string(),
                inputs: vec![],
                outputs: vec![
                    starknet_types::starknet_api::deprecated_contract_class::TypedParameter {
                        name: "publicKey".to_string(),
                        r#type: "felt".to_string(),
                    }
                ],
                state_mutability: Some("view".to_string()),
            })
        );

        let json_str = r#"{
            "inputs": [
                {
                    "name": "newPublicKey",
                    "type": "felt"
                }
            ],
            "name": "setPublicKey",
            "outputs": [],
            "type": "function"
        }"#;

        let obj = serde_json::from_str::<super::ContractClassAbiEntryWithType>(json_str).unwrap();
        assert_eq!(obj.r#type, super::AbiEntryType::Function);
        assert_eq!(
            obj.entry,
            super::AbiEntry::Function(FunctionAbiEntry {
                name: "setPublicKey".to_string(),
                inputs: vec![
                    starknet_types::starknet_api::deprecated_contract_class::TypedParameter {
                        name: "newPublicKey".to_string(),
                        r#type: "felt".to_string(),
                    }
                ],
                outputs: vec![],
                state_mutability: None,
            })
        );

        let json_str = r#"{
            "inputs": [
                {
                    "name": "publicKey",
                    "type": "felt"
                }
            ],
            "name": "constructor",
            "outputs": [],
            "type": "constructor"
        }"#;

        let obj = serde_json::from_str::<super::ContractClassAbiEntryWithType>(json_str).unwrap();
        assert_eq!(obj.r#type, super::AbiEntryType::Constructor);
        assert_eq!(
            obj.entry,
            super::AbiEntry::Function(FunctionAbiEntry {
                name: "constructor".to_string(),
                inputs: vec![
                    starknet_types::starknet_api::deprecated_contract_class::TypedParameter {
                        name: "publicKey".to_string(),
                        r#type: "felt".to_string(),
                    }
                ],
                outputs: vec![],
                state_mutability: None,
            })
        );
    }
    #[test]
    fn deserialize_deprecated_contract_class() {
        let json_str = r#"{
            "abi": [
                {
                    "inputs": [],
                    "name": "getPublicKey",
                    "outputs": [
                        {
                            "name": "publicKey",
                            "type": "felt"
                        }
                    ],
                    "stateMutability": "view",
                    "type": "function"
                },
                {
                    "inputs": [
                        {
                            "name": "newPublicKey",
                            "type": "felt"
                        }
                    ],
                    "name": "setPublicKey",
                    "outputs": [],
                    "type": "function"
                },
                {
                    "inputs": [
                        {
                            "name": "publicKey",
                            "type": "felt"
                        }
                    ],
                    "name": "constructor",
                    "outputs": [],
                    "type": "constructor"
                }
            ],
            "program": "",
            "entry_points_by_type": {
                "EXTERNAL": [
                    {
                        "selector": "0xAAE3B5E8",
                        "offset": "0x1"
                    },
                    {
                        "selector": "0xAAE3B5E9",
                        "offset": "0x2"
                    }
                ]
            }
        }"#;

        let obj = serde_json::from_str::<super::DeprecatedContractClass>(json_str).unwrap();
        assert_eq!(obj.abi.len(), 3);
        assert_eq!(obj.entry_points_by_type.len(), 1);
        assert_eq!(obj.entry_points_by_type.get(&starknet_types::starknet_api::deprecated_contract_class::EntryPointType::External).unwrap().len(), 2);
    }

    #[test]
    fn deserialize_sierra_contract_class() {
        let json_str = r#"{
            "sierra_program": ["0xAA", "0xBB"],
            "contract_class_version": "1.0",
            "entry_points_by_type": {
                "EXTERNAL": [
                    {
                        "selector": "0xAAE3B5E8",
                        "function_idx": 1
                    },
                    {
                        "selector": "0xAAE3B5E9",
                        "function_idx": 2
                    }
                ]
            },
            "abi": "H4sIAAAAAAAA/8tIzcnJVyjPL8pJUQQAlQYXAAAA"
        }"#;
        let obj = serde_json::from_str::<super::SierraContractClass>(json_str).unwrap();
        assert_eq!(obj.sierra_program.len(), 2);
        assert_eq!(obj.contract_class_version, "1.0".to_string());
        assert_eq!(obj.entry_points_by_type.len(), 1);
        assert_eq!(
            obj.entry_points_by_type
                .get(&starknet_types::starknet_api::state::EntryPointType::External)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(obj.abi, "H4sIAAAAAAAA/8tIzcnJVyjPL8pJUQQAlQYXAAAA".to_string());
        assert_eq!(
            obj.entry_points_by_type
                .get(&starknet_types::starknet_api::state::EntryPointType::External)
                .unwrap()[0]
                .selector
                .0,
            starknet_types::starknet_api::hash::StarkFelt::from(
                Felt::from_prefixed_hex_str("0xAAE3B5E8").unwrap()
            )
        );
    }
}
