use fuels::{contract::call_response::FuelCallResponse, prelude::*};

abigen!(
    Exchange,
    "./project/contracts/exchange-contract/out/debug/exchange-contract-abi.json"
);

pub struct ContractBalances {
    pub asset_a: u64,
    pub asset_b: u64,
}

pub struct ExchangeContract {
    pub asset_a: AssetId,
    pub asset_b: AssetId,
    pub instance: Exchange,
    pub liquidity_pool_asset: AssetId,
}

pub struct LiquidityParameters {
    pub amount_a: u64,
    pub amount_b: u64,
    pub deadline: u64,
    pub liquidity: u64,
}

pub struct WalletBalances {
    pub asset_a: u64,
    pub asset_b: u64,
    pub liquidity_pool_asset: u64,
}

pub mod paths {
    pub const EXCHANGE_CONTRACT_BINARY_PATH: &str = "./out/debug/exchange-contract.bin";
}

pub mod abi_calls {
    use super::*;

    pub async fn add_liquidity(
        contract: &Exchange,
        call_params: CallParameters,
        tx_params: TxParameters,
        desired_liquidity: u64,
        deadline: u64,
    ) -> FuelCallResponse<u64> {
        contract
            .methods()
            .add_liquidity(desired_liquidity, deadline)
            .call_params(call_params)
            // `add_liquidity` adds liquidity by using up at least one of the assets
            // one variable output is for the minted liquidity pool asset
            // the other variable output is for the asset that is not used up
            .append_variable_outputs(2)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn constructor(
        contract: &Exchange,
        pair: (AssetId, AssetId),
    ) -> FuelCallResponse<()> {
        contract
            .methods()
            .constructor((ContractId::new(*pair.0), ContractId::new(*pair.1)))
            .call()
            .await
            .unwrap()
    }

    pub async fn deposit(contract: &Exchange, call_params: CallParameters) -> FuelCallResponse<()> {
        contract
            .methods()
            .deposit()
            .call_params(call_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn remove_liquidity(
        contract: &Exchange,
        call_params: CallParameters,
        min_asset_a: u64,
        min_asset_b: u64,
        deadline: u64,
    ) -> FuelCallResponse<RemoveLiquidityInfo> {
        contract
            .methods()
            .remove_liquidity(min_asset_a, min_asset_b, deadline)
            .call_params(call_params)
            .tx_params(TxParameters::new(None, Some(10_000_000), None))
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap()
    }

    pub async fn swap_exact_input(
        contract: &Exchange,
        call_params: CallParameters,
        min_output: Option<u64>,
        deadline: u64,
    ) -> FuelCallResponse<u64> {
        contract
            .methods()
            .swap_exact_input(min_output, deadline)
            .call_params(call_params)
            .tx_params(TxParameters::new(None, Some(10_000_000), None))
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn swap_exact_output(
        contract: &Exchange,
        call_params: CallParameters,
        output: u64,
        deadline: u64,
    ) -> FuelCallResponse<u64> {
        contract
            .methods()
            .swap_exact_output(output, deadline)
            .call_params(call_params)
            .tx_params(TxParameters::new(None, Some(10_000_000), None))
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap()
    }

    pub async fn withdraw(
        contract: &Exchange,
        amount: u64,
        asset: AssetId,
    ) -> FuelCallResponse<()> {
        contract
            .methods()
            .withdraw(amount, ContractId::new(*asset))
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn balance(contract: &Exchange, asset: AssetId) -> FuelCallResponse<u64> {
        contract
            .methods()
            .balance(ContractId::new(*asset))
            .call()
            .await
            .unwrap()
    }

    pub async fn pool_info(contract: &Exchange) -> FuelCallResponse<PoolInfo> {
        contract.methods().pool_info().call().await.unwrap()
    }

    pub async fn preview_add_liquidity(
        contract: &Exchange,
        call_params: CallParameters,
        tx_params: TxParameters,
        amount: u64,
        asset: AssetId,
    ) -> FuelCallResponse<PreviewAddLiquidityInfo> {
        contract
            .methods()
            .preview_add_liquidity(amount, ContractId::new(*asset))
            .call_params(call_params)
            .tx_params(tx_params)
            .call()
            .await
            .unwrap()
    }

    pub async fn preview_swap_exact_input(
        contract: &Exchange,
        exact_input: u64,
        input_asset: AssetId,
    ) -> FuelCallResponse<PreviewSwapInfo> {
        contract
            .methods()
            .preview_swap_exact_input(exact_input, ContractId::new(*input_asset))
            .tx_params(TxParameters::new(None, Some(10_000_000), None))
            .call()
            .await
            .unwrap()
    }

    pub async fn preview_swap_exact_output(
        contract: &Exchange,
        exact_output: u64,
        output_asset: AssetId,
    ) -> FuelCallResponse<PreviewSwapInfo> {
        contract
            .methods()
            .preview_swap_exact_output(exact_output, ContractId::new(*output_asset))
            .tx_params(TxParameters::new(None, Some(10_000_000), None))
            .call()
            .await
            .unwrap()
    }
}

pub mod test_helpers {
    use super::*;
    use abi_calls::{add_liquidity, balance, constructor, deposit};
    use paths::EXCHANGE_CONTRACT_BINARY_PATH;

    pub async fn contract_balances(exchange: &ExchangeContract) -> ContractBalances {
        let asset_a = balance(&exchange.instance, exchange.asset_a).await.value;
        let asset_b = balance(&exchange.instance, exchange.asset_b).await.value;
        ContractBalances { asset_a, asset_b }
    }

    pub async fn wallet_balances(
        exchange: &ExchangeContract,
        wallet: &WalletUnlocked,
    ) -> WalletBalances {
        let asset_a = wallet.get_asset_balance(&exchange.asset_a).await.unwrap();
        let asset_b = wallet.get_asset_balance(&exchange.asset_b).await.unwrap();
        let liquidity_pool_asset = wallet
            .get_asset_balance(&exchange.liquidity_pool_asset)
            .await
            .unwrap();
        WalletBalances {
            asset_a,
            asset_b,
            liquidity_pool_asset,
        }
    }

    pub async fn deposit_but_do_not_add_liquidity(
        amounts: &LiquidityParameters,
        exchange: &ExchangeContract,
    ) {
        deposit(
            &exchange.instance,
            CallParameters::new(Some(amounts.amount_a), Some(exchange.asset_a), None),
        )
        .await;

        deposit(
            &exchange.instance,
            CallParameters::new(Some(amounts.amount_b), Some(exchange.asset_b), None),
        )
        .await;
    }

    pub async fn deposit_and_add_liquidity(
        amounts: &LiquidityParameters,
        exchange: &ExchangeContract,
    ) -> u64 {
        deposit_but_do_not_add_liquidity(&amounts, &exchange).await;

        let added = add_liquidity(
            &exchange.instance,
            CallParameters::new(Some(0), None, None),
            TxParameters::new(None, Some(100_000_000), None),
            amounts.liquidity,
            amounts.deadline,
        )
        .await;

        added.value
    }

    pub async fn setup() -> (Exchange, WalletUnlocked, AssetId, AssetId, AssetId, AssetId) {
        // setup wallet and provider
        let mut wallet = WalletUnlocked::new_random(None);
        let num_assets = 3;
        let coins_per_asset = 10;
        let amount_per_coin = 1_000_000;
        let (coins, asset_ids) = setup_multiple_assets_coins(
            wallet.address(),
            num_assets,
            coins_per_asset,
            amount_per_coin,
        );
        let (provider, _socket_addr) = setup_test_provider(coins.clone(), vec![], None, None).await;
        wallet.set_provider(provider);

        // setup exchange contract
        let exchange_contract_id = Contract::deploy(
            EXCHANGE_CONTRACT_BINARY_PATH,
            &wallet,
            TxParameters::default(),
            StorageConfiguration::default(),
        )
        .await
        .unwrap();
        let exchange_instance = Exchange::new(exchange_contract_id.clone(), wallet.clone());

        let liquidity_pool_asset_id = AssetId::from(*exchange_contract_id.hash());

        (
            exchange_instance,
            wallet,
            liquidity_pool_asset_id,
            asset_ids[0],
            asset_ids[1],
            asset_ids[2],
        )
    }

    pub async fn setup_and_initialize() -> (
        ExchangeContract,
        WalletUnlocked,
        LiquidityParameters,
        AssetId,
    ) {
        let (exchange_instance, wallet, liquidity_pool_asset, asset_a, asset_b, asset_c) =
            setup().await;
        constructor(&exchange_instance, (asset_a, asset_b)).await;

        let exchange = ExchangeContract {
            asset_a,
            asset_b,
            instance: exchange_instance,
            liquidity_pool_asset,
        };

        let amounts = LiquidityParameters {
            amount_a: 100,
            amount_b: 400,
            deadline: 1000,
            liquidity: 200,
        };

        (exchange, wallet, amounts, asset_c)
    }

    pub async fn setup_initialize_and_deposit_but_do_not_add_liquidity() -> (
        ExchangeContract,
        WalletUnlocked,
        LiquidityParameters,
        AssetId,
    ) {
        let (exchange, wallet, amounts, asset_c) = setup_and_initialize().await;

        deposit_but_do_not_add_liquidity(&amounts, &exchange).await;

        (exchange, wallet, amounts, asset_c)
    }

    pub async fn setup_initialize_deposit_and_add_liquidity() -> (
        ExchangeContract,
        WalletUnlocked,
        LiquidityParameters,
        AssetId,
        u64,
    ) {
        let (exchange, wallet, amounts, asset_c) = setup_and_initialize().await;

        let added_liquidity = deposit_and_add_liquidity(&amounts, &exchange).await;

        (exchange, wallet, amounts, asset_c, added_liquidity)
    }
}
