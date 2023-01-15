use reqwest::{Client, ClientBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::common::{OptionsContract, TradeDirection};
// TDAMERITRADE

// Place order EXAMPLE
// payload = dict(complexOrderStrategyType="NONE", orderType="LIMIT", session="NORMAL", price=str(matching_tda_option_price),
//                                    duration="DAY", orderStrategyType="SINGLE",
//                                    orderLegCollection=[
//                                        {
//                                            "instruction": "SELL_TO_CLOSE",
//                                            "quantity": i["qty"],
//                                            "instrument": {
//                                                "symbol": i['contractName'],
//                                                "assetType": "OPTION"
//                                            }
//                                        }
//                                    ])
pub struct OptionsOrderStrategy {
    // the price used for the order
    price: f32,
    // quantity of units
    quantity: usize,
    // ticker symbol to trade
    ticker_symbol: String,
    // determines a few of the parameters
    options_contract: OptionsContract,
    // determines instruction
    trade_direction: TradeDirection,
}

impl OptionsOrderStrategy {
    // build the simplified order strategy payload
    pub fn build_order(&self) -> PlaceOrderBody {
        PlaceOrderBody {
            complex_order_strategy_type: "NONE".to_string(),
            order_type: "LIMIT".to_string(),
            session: "NORMAL".to_string(),
            price: self.price,
            duration: "DAY".to_string(),
            order_strategy_type: "SINGLE".to_string(),
            order_leg_collection: vec![OrderLeg {
                instruction: self.order_instruction().to_string(),
                quantity: self.quantity,
                instrument: OptionsInstrument {
                    symbol: self.options_contract_id(),
                    asset_type: "OPTION".to_string(),
                },
            }],
        }
    }

    // produce the order instruction for buying/selling
    fn order_instruction(&self) -> &str {
        match self.trade_direction {
            TradeDirection::Buy => "BUY_TO_OPEN",
            TradeDirection::Sell => "SELL_TO_CLOSE",
        }
    }

    // produces the string that is used to buy/sell an options contract => orderLegCollection[0]instrument.symbol
    fn options_contract_id(&self) -> String {
        // python
        // c_option_contract = f'{static_ticker_names[0]}_{cal["optionsDate"]}C{call_price}'
        let option_initial = match self.options_contract {
            OptionsContract::Call => "C",
            OptionsContract::Put => "P",
        };
        // TODO price needs to be calculated based on the price of the order example python
        // put_price = (av_close - (av_close % 2.5) - 2.5)
        // if put_price % 1 == 0:
        //     put_price = int(put_price)
        // else:
        //     put_price = round(put_price, 1)

        let delta = self.strike_price_delta();
        // the strike price of the options contract
        // TODO number of delta steps could be configurable =>       delta * n
        let strike_price = (self.price - (self.price - delta) - delta).round() as i32;
        let id = format!(
            "{}_{}{}{}",
            self.ticker_symbol,
            self.strike_date(),
            option_initial,
            strike_price
        );
        id
    }

    // TODO the delta will need to be determined based on ticker
    fn strike_price_delta(&self) -> f32 {
        2.5
    }

    // TODO use a config param to calculate the strike date based on a delta from current date '090122'
    fn strike_date(&self) -> &str {
        return "090122";
    }
}

// TODO add json serializable functionality such as serde_json
// A Summary can be found here: https://developer.tdameritrade.com/account-access/apis/post/accounts/%7BaccountId%7D/orders-0
#[derive(Deserialize, Serialize, Debug)]
pub struct PlaceOrderBody {
    // complicated stuff
    complex_order_strategy_type: String,
    // limit always
    order_type: String,
    // not sure
    session: String,
    // price to trade
    price: f32,
    // expiration of order
    duration: String,
    // not sure
    order_strategy_type: String,
    // the order details such as ticker, trade instrument, quantity, for simplicity vector can just be size of 1
    order_leg_collection: Vec<OrderLeg>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderLeg {
    // instructions depend on asset type
    instruction: String,
    // quantity to order
    quantity: usize,
    // symbol and type
    instrument: OptionsInstrument,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OptionsInstrument {
    // name of the of ticker symbol
    symbol: String,
    // asset type
    asset_type: String,
}

// TODO separate the file structure out more

const TD_BASE_URL: &str = "https://api.tdameritrade.com/v1/";

// TODO Place Order starting template

pub async fn place_order(
    account_id: &str,
    headers: reqwest::header::HeaderMap,
    body: PlaceOrderBody,
) -> Result<()> {
    // r = re.post(f"https://api.tdameritrade.com/v1/accounts/{accountId}/orders", headers=headers, json=json)
    // TODO turn this into a util
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new()
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let place_order_url = format!(
        "{base_url}/accounts/{account_id}/orders",
        base_url = TD_BASE_URL,
        account_id = account_id
    );
    // TODO re use the same client if this were to become a deamon or server
    let response = Client::new()
        .post(place_order_url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    // error handling
    Ok(())
}
