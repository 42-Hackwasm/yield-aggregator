use crate::error::ContractError;
use cosmwasm_std::Coin;


/// Modified assert_sent_sufficient_coin function just so it has to be == the required amount
pub fn assert_sent_exact_coin(sent: &[Coin], required: Option<Coin>) -> Result<(), ContractError> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let mut received_amounts = vec![];
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                received_amounts.push(coin.amount.u128().to_string() + " " + &coin.denom);
                coin.denom == required_coin.denom && coin.amount.u128() == required_amount
            });

            if sent_sufficient_funds {
                return Ok(());
            } else {
                return Err(ContractError::InsufficientFundsSend {
                    needed: required_amount.to_string(),
                    received: convert_vector_of_string_slices_to_string(received_amounts),
                });
            }
        }
    }
    Ok(())
}

fn convert_vector_of_string_slices_to_string(vector: Vec<String>) -> String {
    let mut string = String::new();
    for s in vector {
        string.push_str(&s);
        string.push_str(", ");
    }
    string
}


/// You send enough OR MORE coins with the correct denom(s)
/// https://github.com/InterWasm/cw-contracts/blob/main/contracts/nameservice/src/coin_helpers.rs
pub fn assert_sent_sufficient_coin(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<(), ContractError> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                coin.denom == required_coin.denom && coin.amount.u128() >= required_amount
            });

            if sent_sufficient_funds {
                return Ok(());
            } else {
                return Err(ContractError::InsufficientFundsSend {});
            }
        }
    }
    Ok(())
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use cosmwasm_std::{coin, coins};

//     #[test]
//     fn assert_sent_sufficient_coin_works() {
//         match assert_sent_sufficient_coin(&[], Some(coin(0, "token"))) {
//             Ok(()) => {}
//             Err(e) => panic!("Unexpected error: {:?}", e),
//         };

//         match assert_sent_sufficient_coin(&[], Some(coin(5, "token"))) {
//             Ok(()) => panic!("Should have raised insufficient funds error"),
//             Err(ContractError::InsufficientFundsSend {}) => {}
//             Err(e) => panic!("Unexpected error: {:?}", e),
//         };

//         match assert_sent_sufficient_coin(&coins(10, "smokin"), Some(coin(5, "token"))) {
//             Ok(()) => panic!("Should have raised insufficient funds error"),
//             Err(ContractError::InsufficientFundsSend {}) => {}
//             Err(e) => panic!("Unexpected error: {:?}", e),
//         };

//         match assert_sent_sufficient_coin(&coins(10, "token"), Some(coin(5, "token"))) {
//             Ok(()) => {}
//             Err(e) => panic!("Unexpected error: {:?}", e),
//         };

//         let sent_coins = vec![coin(2, "smokin"), coin(5, "token"), coin(1, "earth")];
//         match assert_sent_sufficient_coin(&sent_coins, Some(coin(5, "token"))) {
//             Ok(()) => {}
//             Err(e) => panic!("Unexpected error: {:?}", e),
//         };
//     }
// }