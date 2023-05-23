use std::{
    hash::Hash,
    marker::PhantomData,
    collections::{HashMap, BTreeSet},
    sync::Arc,
    time::Duration,
    fmt::{Display, Debug},
};
use tokio::runtime::Runtime;
use krakenrs::{
    KrakenRestAPI, KrakenCredentials, KrakenRestConfig, LimitOrder,
    OrderFlag, BsType
};
use log::{info, error};
use exch_observer_types::{ExchangeBalance, ExchangeClient};


/// Interface to Kraken REST API.
pub struct KrakenClient<Symbol: Eq + Hash> {
    pub api: Arc<KrakenRestAPI>,
    pub runtime: Option<Arc<Runtime>>,
    marker: PhantomData<Symbol>,
}

impl<Symbol> KrakenClient<Symbol>
where Symbol: Eq + Hash + Clone + Display + Debug + Into<String>
{
    pub fn new(api_key: String, api_secret: String) -> Self {

        let creds = KrakenCredentials { key: api_key, secret: api_secret };
        let config = KrakenRestConfig {
            timeout: Duration::from_secs(10),
            creds: creds
        };

        let api = KrakenRestAPI::try_from(config).unwrap();

        Self {
            api: Arc::new(api),
            runtime: None,
            marker: PhantomData,
        }
    }

    pub fn new_with_runtime(
        api_key: String,
        api_secret: String,
        runtime: Arc<Runtime>,
    ) -> Self {

        let creds = KrakenCredentials { key: api_key, secret: api_secret };

        let config = KrakenRestConfig {
            timeout: Duration::from_secs(10),
            creds: creds
        };

        let api = KrakenRestAPI::try_from(config).unwrap();

        Self {
            api: Arc::new(api),
            runtime: Some(runtime),
            marker: PhantomData,
        }
    }

    pub fn set_runtime(&mut self, runtime: Arc<Runtime>) {
        self.runtime = Some(runtime);
    }

    pub fn has_runtime(&self) -> bool {
        self.runtime.is_some()
    }

    fn buy_order1(runner: &Runtime, api: Arc<KrakenRestAPI>, symbol: Symbol, qty: f64, price: f64) {
        let symbol: String = symbol.into();
        runner.spawn_blocking(move || {
            info!(
                "calling Buy order on symbol: {}; qty: {}; price: {}",
                &symbol, qty, price
            );

            let oflags = BTreeSet::from_iter(vec![OrderFlag::Fciq]);
            let res = api.add_limit_order(
                LimitOrder {
                    bs_type: BsType::Buy,
                    volume: qty.to_string(),
                    pair: symbol,
                    price: price.to_string(),
                    oflags: oflags,
                },
                None, // userref
                false
            );

            if let Ok(res) = res {
                info!("Buy order completed: {:?}", res);
            } else {
                error!("Error placing buy order: {:?}", res.err().unwrap());
            }

        });
    }

    fn sell_order1(runner: &Runtime, api: Arc<KrakenRestAPI>, symbol: Symbol, qty: f64, price: f64) {
        let symbol: String = symbol.into();
        runner.spawn_blocking(move || {
            info!(
                "calling Buy order on symbol: {}; qty: {}; price: {}",
                &symbol, qty, price
            );

            let oflags = BTreeSet::from_iter(vec![OrderFlag::Fcib]);
            let res = api.add_limit_order(
                LimitOrder {
                    bs_type: BsType::Sell,
                    volume: qty.to_string(),
                    pair: symbol,
                    price: price.to_string(),
                    oflags: oflags,
                },
                None, // userref
                false
            );

            if let Ok(res) = res {
                info!("Buy order completed: {:?}", res);
            } else {
                error!("Error placing buy order: {:?}", res.err().unwrap());
            }

        });
    }
}

impl<Symbol> ExchangeClient<Symbol> for KrakenClient<Symbol>
where
    Symbol: Eq + Hash + Clone + Display + Debug + Into<String>,
{
    fn symbol_exists(&self, symbol: &Symbol) -> bool {
        true
    }

    /// Fetches the balance for the given asset from Binance Account API
    fn get_balance(&self, asset: &String) -> Option<ExchangeBalance> {
        None
    }

    /// Sends Buy GTC limit order to Binance REST API
    fn buy_order(&self, symbol: &Symbol, qty: f64, price: f64) {
        let runtime = if let Some(runtime) = &self.runtime {
            runtime.clone()
        } else {
            panic!("No runtime set for BinanceClient, cannot execute buy order");
        };
        Self::buy_order1(&runtime, self.api.clone(), symbol.clone(), qty, price);
    }

    /// Sends Sell GTC limit order to Binance REST API
    fn sell_order(&self, symbol: &Symbol, qty: f64, price: f64) {
        let runtime = if let Some(runtime) = &self.runtime {
            runtime.clone()
        } else {
            panic!("No runtime set for BinanceClient, cannot execute sell order");
        };
        Self::sell_order1(&runtime, self.api.clone(), symbol.clone(), qty, price);
    }

    /// Fetches the balances for all assets from Binance Account API
    fn get_balances(&self) -> Result<HashMap<String, ExchangeBalance>, Box<dyn std::error::Error>> {
        Ok(HashMap::new())
    }
}