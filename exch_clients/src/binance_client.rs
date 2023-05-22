use binance::{
    account::{Account, OrderSide, OrderType, TimeInForce},
    api::Binance,
    market::Market,
};
use exch_observer_types::{ExchangeBalance, ExchangeClient};
use log::info;
use std::sync::Arc;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};
use tokio::runtime::Runtime;

/// Client for the Binance REST API, implemented using
/// `https://github.com/wisespace-io/binance-rs.git` crate
pub struct BinanceClient<Symbol: Eq + Hash> {
    /// Account API
    pub account: Arc<Account>,
    /// Market API
    pub market: Arc<Market>,
    pub runtime: Option<Arc<Runtime>>,
    marker: PhantomData<Symbol>,
}

impl<Symbol: Eq + Hash + Clone + Display + Debug + Into<String>> BinanceClient<Symbol> {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            account: Arc::new(Account::new(api_key.clone(), secret_key.clone())),
            market: Arc::new(Market::new(api_key, secret_key)),
            runtime: None,
            marker: PhantomData,
        }
    }

    pub fn new_with_runtime(
        api_key: Option<String>,
        secret_key: Option<String>,
        async_runner: Arc<Runtime>,
    ) -> Self {
        Self {
            account: Arc::new(Account::new(api_key.clone(), secret_key.clone())),
            market: Arc::new(Market::new(api_key, secret_key)),
            runtime: Some(async_runner),
            marker: PhantomData,
        }
    }

    pub fn set_runtime(&mut self, runtime: Arc<Runtime>) {
        self.runtime = Some(runtime);
    }

    pub fn has_runtime(&self) -> bool {
        self.runtime.is_some()
    }

    /// Sends Buy GTC limit order to Binance REST API
    fn buy_order1(runner: &Runtime, account: Arc<Account>, symbol: Symbol, qty: f64, price: f64) {
        let mut symbol: String = symbol.into();
        symbol.make_ascii_uppercase();
        runner.spawn_blocking(move || {
            info!(
                "calling Buy order on symbol: {}; qty: {}; price: {}",
                &symbol, qty, price
            );
            let recipe = account
                .custom_order::<String, f64>(
                    symbol.clone().into(),
                    f64::try_from(qty).unwrap(),
                    f64::try_from(price).unwrap(),
                    None,
                    OrderSide::Buy,
                    OrderType::Limit,
                    TimeInForce::GTC,
                    None,
                )
                .unwrap();

            info!(
                "Trade [sym: {}, qty: {}, price: {}] successful, recipe: {:?}",
                symbol, qty, price, recipe
            );

            recipe
        });
    }

    /// Sends Sell GTC limit order to Binance REST API
    fn sell_order1(runner: &Runtime, account: Arc<Account>, symbol: Symbol, qty: f64, price: f64) {
        let mut symbol: String = symbol.into();
        symbol.make_ascii_uppercase();
        runner.spawn_blocking(move || {
            info!(
                "calling Sell order on symbol: {}; qty: {}; price: {}",
                &symbol, qty, price
            );
            let recipe = account
                .custom_order::<String, f64>(
                    symbol.clone().into(),
                    f64::try_from(qty).unwrap(),
                    f64::try_from(price).unwrap(),
                    None,
                    OrderSide::Sell,
                    OrderType::Limit,
                    TimeInForce::GTC,
                    None,
                )
                .unwrap();

            info!(
                "Trade [sym: {}, qty: {}, price: {}] successful, recipe: {:?}",
                symbol, qty, price, recipe
            );

            recipe
        });
    }
}

impl<Symbol> ExchangeClient<Symbol> for BinanceClient<Symbol>
where
    Symbol: Eq + Hash + Clone + Display + Debug + Into<String>,
{
    fn symbol_exists(&self, symbol: &Symbol) -> bool {
        self.market
            .get_depth(Into::<String>::into(symbol.clone()).to_ascii_uppercase())
            .is_ok()
    }

    /// Fetches the balance for the given asset from Binance Account API
    fn get_balance(&self, asset: &String) -> Option<ExchangeBalance> {
        let mut asset: String = asset.into();
        asset.make_ascii_uppercase();
        let rv = self
            .account
            .get_balance(asset.clone())
            .ok()
            .map(|v| Into::<ExchangeBalance>::into(v));

        info!("Fetching balance for {}: {:?}", asset, rv);
        rv
    }

    /// Sends Buy GTC limit order to Binance REST API
    fn buy_order(&self, symbol: &Symbol, qty: f64, price: f64) {
        let runtime = if let Some(runtime) = &self.runtime {
            runtime.clone()
        } else {
            panic!("No runtime set for BinanceClient, cannot execute buy order");
        };
        Self::buy_order1(&runtime, self.account.clone(), symbol.clone(), qty, price);
    }

    /// Sends Sell GTC limit order to Binance REST API
    fn sell_order(&self, symbol: &Symbol, qty: f64, price: f64) {
        let runtime = if let Some(runtime) = &self.runtime {
            runtime.clone()
        } else {
            panic!("No runtime set for BinanceClient, cannot execute sell order");
        };
        Self::sell_order1(&runtime, self.account.clone(), symbol.clone(), qty, price);
    }

    /// Fetches the balances for all assets from Binance Account API
    fn get_balances(&self) -> Result<HashMap<String, ExchangeBalance>, Box<dyn std::error::Error>> {
        let account_info = self.account.get_account()?;

        let mut balances: HashMap<String, ExchangeBalance> = HashMap::new();
        for balance in &account_info.balances {
            balances.insert(balance.asset.clone(), balance.clone().into());
        }

        Ok(balances)
    }
}
