#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{execute, instantiate, query};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Uint128};
    use euclid::error::ContractError;
    use euclid::msgs::router::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
    use euclid::token::{Pair, Token};
    // use euclid_ibc::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, STATE};

    struct TestToken {
        name: &'static str,
        token: Token,
        expected_error: Option<ContractError>,
    }

    struct TestInstantiateMsg {
        name: &'static str,
        msg: InstantiateMsg,
        expected_error: Option<ContractError>,
    }

    struct TestExecuteMsg {
        name: &'static str,
        msg: ExecuteMsg,
        expected_error: Option<ContractError>,
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let test_cases = vec![TestInstantiateMsg {
            name: "Valid instantiate message",
            msg: InstantiateMsg {
                vlp_code_id: 1,
                vcoin_code_id: 2,
            },
            expected_error: None,
        }];

        for test in test_cases {
            let res = instantiate(deps.as_mut(), env.clone(), info.clone(), test.msg.clone());
            match test.expected_error {
                Some(err) => assert_eq!(res.unwrap_err(), err, "{}", test.name),
                None => assert!(res.is_ok(), "{}", test.name),
            }
        }
    }
}
